//! Scenario steps over one Persona each: a [`Side`] is a [`Node`] holding
//! that Persona's store, with the chain stood in for by the `OrgState` a
//! step hands `verify`. Each step is one or a few `OrgService` calls, the
//! `Node` step it names where one exists, values in and values out.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// As in `support`: each target calls a subset of these steps.
#![allow(dead_code)]

use org_node::invitation::Invite;
use org_node::keys::X25519Keypair;
use org_node::revocation::RevocationNotice;
use org_node::service::{Checked, StoreChange};
use org_node::store::{OrgRecord, PersonaStore, StoreData, StoreWrite};
use org_node::test_fixtures::genesis_trie;
use org_node::verify_org::Verified;
use org_node::wire::{encode_message, WireMessage};
use org_node::{DevicePublicKey, DeviceSecret, Epoch, MemberId, OrgId, OrgService, OrgState, SigningKeypair};
use rand::rngs::OsRng;

use super::{handle, held_store, parsed_name, parsed_surname, the_org, Node, Party};

/// One Persona: a [`Node`] holding its store, and its Device key.
pub struct Side {
    pub node: Node,
    pub device: DevicePublicKey,
}

impl std::ops::Deref for Side {
    type Target = Node;
    fn deref(&self) -> &Node {
        &self.node
    }
}

impl std::ops::DerefMut for Side {
    fn deref_mut(&mut self) -> &mut Node {
        &mut self.node
    }
}

impl Side {
    /// The Persona an operation of `svc` just created, as org-io keeps it:
    /// `secret` in the keychain and `change` on the disk.
    pub fn new(svc: OrgService, device: DevicePublicKey, secret: DeviceSecret, change: StoreChange) -> Self {
        let mut side = Self { node: Node::over(svc, Party::default()), device };
        side.node.keychain.keep(&secret);
        side.keep(change);
        side
    }

    /// A Persona of a fresh node, from its identifier there.
    fn of(node: Node, persona_id: &org_node::PersonaId) -> Self {
        let device = node.device_of(persona_id);
        Self { node, device }
    }

    /// Keeps the write a store change carries, as org-io writes it.
    pub fn keep(&mut self, change: StoreChange) {
        let StoreChange::Sealed(write) = &change;
        assert_eq!(write.device_key, self.device, "a change for this Persona's store");
        self.node.disk.apply(change);
    }

    /// The last sealed write of this Persona's store.
    pub fn last_write(&self) -> StoreWrite {
        StoreWrite { device_key: self.device, sealed: self.node.disk.sealed_of(&self.device) }
    }

    /// The Device secret, read back from the keychain.
    pub fn secret(&self) -> DeviceSecret {
        self.node.keychain.secret(&self.device).expect("the Device secret")
    }

    /// The store of this side's Device as it reached its node's disk,
    /// unsealed.
    pub fn store_on_disk(&self) -> PersonaStore {
        self.node.store_on_disk(&self.device)
    }
}

/// The store data of `side`'s Persona.
pub fn store_of(side: &Side) -> &StoreData {
    held_store(&side.svc, &side.device)
}

/// The current record of `side`'s store.
pub fn record_of(side: &Side) -> &OrgRecord {
    store_of(side).record.as_ref().expect("a current record")
}

/// The MemberId an adoption gave `side`'s Persona.
pub fn member_id_of(side: &Side) -> MemberId {
    store_of(side).persona.member_id.expect("an adopted Persona")
}

/// A founder whose founding is proposed and whose proxy is recorded for
/// `org_id`, not yet adopted.
pub fn found_at(org_id: OrgId) -> Side {
    let mut node = Node::new(Party::default());
    let persona_id = node.propose_and_keep_founding(&mut OsRng, handle("ada"), parsed_name("Ada"), parsed_surname("Lovelace")).unwrap();
    let mut side = Side::of(node, &persona_id);
    let device = side.device;
    side.record_proxy(&device, org_id);
    side
}

pub fn found() -> Side {
    found_at(the_org())
}

/// The chain state a written founding leaves: its root and key at epoch 1.
pub fn founding_state(side: &Side) -> OrgState {
    let founding = store_of(side).founding.as_ref().expect("a provisional founding");
    OrgState { root_hash: founding.root, org_pub_key: founding.org_pub_key, epoch: Epoch::new(1) }
}

/// `Node::verify_and_keep` of `side`'s store against `chain`.
pub fn verify(side: &mut Side, chain: OrgState) -> Verified {
    let device = side.device;
    side.verify_and_keep(&device, Some(chain))
}

/// `Node::adopt` of `side`'s store against `chain`: it must adopt.
pub fn adopt(side: &mut Side, chain: OrgState) {
    let device = side.device;
    side.node.adopt(&device, chain);
}

/// A founder at `org_id` whose founding is adopted, and the chain state.
pub fn founded_at(org_id: OrgId) -> (Side, OrgState) {
    let mut side = found_at(org_id);
    let chain = founding_state(&side);
    let verified = verify(&mut side, chain);
    assert!(matches!(verified, Verified::Adopted { own: true, .. }), "{verified:?}");
    (side, chain)
}

pub fn founded() -> (Side, OrgState) {
    founded_at(the_org())
}

/// `Node::propose` from `side`'s store: the chain state a write of it
/// leaves.
pub fn propose_next(side: &mut Side) -> OrgState {
    let device = side.device;
    side.propose(&device)
}

/// The MemberId the provisional organisation for `chain` gives `device`.
pub fn proposed_id_of(side: &Side, chain: OrgState, device: &DevicePublicKey) -> MemberId {
    store_of(side)
        .provisional_organisations
        .iter()
        .find(|proposal| proposal.root == chain.root_hash && proposal.org_pub_key == chain.org_pub_key)
        .expect("the provisional organisation for that state")
        .members
        .iter()
        .find(|member| member.device_keys.contains(device))
        .expect("the Device is a member of it")
        .id
}

/// `Node::invite` from `inviter`'s store for `invitee`.
pub fn invite(inviter: &mut Side, invitee: &str) -> Invite {
    let device = inviter.device;
    inviter.node.invite(&device, invitee)
}

/// A provisional Member of `invite`'s Organisation, in a node of its own.
pub fn propose_member(invite: &Invite, name: &str, handles: &[&str]) -> Side {
    let mut node = Node::new(Party::default());
    let persona_id = node.propose_member(&mut OsRng, invite, name, "Member", handles).unwrap();
    Side::of(node, &persona_id)
}

/// `member`'s admission request, kept by `inviter` (`support::request_admission`).
pub fn request_admission(inviter: &mut Side, member: &Side) {
    let device = inviter.device;
    super::request_admission(&mut inviter.node, &device, &member.node, &member.device);
}

/// `Node::receive` of `body` from `from` in `side`'s store.
pub fn receive(side: &mut Side, from: DevicePublicKey, body: &[u8]) -> Checked {
    let device = side.device;
    side.node.receive(&device, from, body).unwrap()
}

/// Receives an Organisation-information `body` and verifies against
/// `chain`, which must adopt.
pub fn receive_and_adopt(side: &mut Side, from: DevicePublicKey, body: &[u8], chain: OrgState) -> Verified {
    let checked = receive(side, from, body);
    assert!(matches!(checked, Checked::ProvisionalOrganisationKept { .. }), "{checked:?}");
    let verified = verify(side, chain);
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    verified
}

/// `Node::message_for` from `inviter`'s store to `recipient`.
pub fn message_for(inviter: &Side, recipient: &DevicePublicKey) -> Vec<u8> {
    inviter.node.message_for(&inviter.device, recipient)
}

/// An admission adopted by its inviter and not yet received by the new
/// Member: the Member's side, the chain state, and the body it is sent.
pub struct Admission {
    pub member: Side,
    pub chain: OrgState,
    pub body: Vec<u8>,
}

/// `inviter` invites `name`, keeps the request, records the admission,
/// proposes and adopts the next epoch.
pub fn admission(inviter: &mut Side, name: &str, handle: &str) -> Admission {
    let invite = invite(inviter, name);
    let member = propose_member(&invite, name, &[handle]);
    request_admission(inviter, &member);
    let change = inviter.node.svc.record_admission(&mut OsRng, &inviter.device, invite.invite_id, None).unwrap();
    inviter.keep(change);
    let chain = propose_next(inviter);
    let verified = verify(inviter, chain);
    assert!(matches!(verified, Verified::Adopted { own: true, .. }), "{verified:?}");
    let body = message_for(inviter, &member.device);
    Admission { member, chain, body }
}

/// `inviter` records the revocation of `member_id`, proposes and adopts.
pub fn revocation(inviter: &mut Side, member_id: MemberId) -> OrgState {
    let change = inviter.node.svc.record_revocation(&mut OsRng, &inviter.device, member_id).unwrap();
    inviter.keep(change);
    let chain = propose_next(inviter);
    let verified = verify(inviter, chain);
    assert!(matches!(verified, Verified::Adopted { own: true, .. }), "{verified:?}");
    chain
}

/// A well-formed revocation notice body about `org_id` naming `member_id`
/// and `device`, its absence proof genuine under a trie no chain holds.
pub fn notice_about(org_id: OrgId, member_id: MemberId, device: DevicePublicKey) -> Vec<u8> {
    let trie = genesis_trie(&X25519Keypair::generate(&mut OsRng), &SigningKeypair::generate(&mut OsRng));
    let proof = trie.prove_absent(&member_id, &device).unwrap();
    encode_message(&WireMessage::Revocation(RevocationNotice { org_id, member_id, device, proof })).unwrap()
}
