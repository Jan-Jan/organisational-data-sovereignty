#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Verification of one Organisation against the chain (SDD-kegag2): `verify`
//! judges the chain state org-io read against the store's record and its
//! provisional organisations, adopts only a match, and otherwise changes no
//! sealed byte and asks no secret.

use std::cell::Cell;

use org_node::error::OrgNodeError;
use org_node::service::OrgService;
use org_node::store::{Origin, PersonaStore};
use org_node::test_fixtures::{device_key, device_secret, seal_for_test, store_for_test};
use org_node::{DiscardedChange, Epoch, Inconsistency, RootHash, Verified};
use rand::rngs::OsRng;

mod support;
use support::sweep;

/// Fixtures: stores holding a record of one Organisation, chain states,
/// and the Organisation information a listed Device sends.
mod stores {
    use std::cell::Cell;

    use org_node::chain::OrgState;
    use org_node::envelope::Envelope;
    use org_node::error::OrgNodeError;
    use org_node::service::{Checked, OrgService, StoreChange};
    use org_node::revocation::RevocationNotice;
    use org_node::store::{KeptAdmissionRequest, MemberSnapshot, OrgRecord, Origin, OutstandingInvite, RevokedDevice, StoreData};
    use org_node::test_fixtures::device_key;
    use org_node::wire::{encode_message, WireMessage};
    use org_node::{
        AheadLimit, DevicePublicKey, DeviceSecret, Epoch, Handle, MemberId, OrgPrivateKey, OrgPublicKey, RootHash,
        SequenceNumber, VerificationTime, Verified,
    };
    use rand::rngs::OsRng;

    pub use crate::support::fixtures::*;
    use crate::support::{held_store, leaf_of, snapshots_of, the_org, trie_of_members};

    /// `fixtures::record` with `members` in the trie's order, as an
    /// adoption leaves them.
    pub fn record_in_trie_order(epoch: u64, members: &[MemberSnapshot]) -> OrgRecord {
        crate::support::fixtures::record(epoch, &snapshots_of(&trie_of_members(members)))
    }

    /// Ada's store: a record at `epoch` listing `members` in trie order.
    pub fn ada_store_in_trie_order(epoch: u64, members: &[MemberSnapshot]) -> StoreData {
        let mut store_data = crate::support::fixtures::ada_store(epoch, members);
        store_data.record = Some(record_in_trie_order(epoch, members));
        store_data
    }

    /// `verify_as` on the store of the Device of seed `device`.
    pub fn verify(
        svc: &mut OrgService,
        device: u8,
        state: Option<OrgState>,
        asked: &Cell<bool>,
    ) -> Result<(Verified, Option<StoreChange>), OrgNodeError> {
        verify_as(svc, device_key(device), state, asked)
    }

    /// `verify` on the store `key` names, recording in `asked` whether the
    /// secret source was called.
    pub fn verify_as(
        svc: &mut OrgService,
        key: DevicePublicKey,
        state: Option<OrgState>,
        asked: &Cell<bool>,
    ) -> Result<(Verified, Option<StoreChange>), OrgNodeError> {
        verify_at(svc, key, state, (VerificationTime(1_760_000_000), AheadLimit(1200)), asked)
    }

    /// `verify` on the store `key` names at the time and with the ahead
    /// limit `when` gives, recording in `asked` whether the secret source
    /// was called.
    pub fn verify_at(
        svc: &mut OrgService,
        key: DevicePublicKey,
        state: Option<OrgState>,
        when: (VerificationTime, AheadLimit),
        asked: &Cell<bool>,
    ) -> Result<(Verified, Option<StoreChange>), OrgNodeError> {
        svc.verify(&mut OsRng, &key, state, when.0, when.1, |_: &[DevicePublicKey]| -> Vec<DeviceSecret> {
            asked.set(true);
            Vec::new()
        })
    }

    /// The Organisation information a listed Device sends for the epoch
    /// after a record at `base_epoch` listing `base`, admitting `added`
    /// and carrying the private key `key`: the body, the next record's
    /// root and the key's public half.
    pub fn information(
        base_epoch: u64,
        base: &[MemberSnapshot],
        added: &MemberSnapshot,
        key: [u8; 32],
    ) -> (Vec<u8>, RootHash, OrgPublicKey) {
        let base_trie = trie_of_members(base);
        let (next, delta) = base_trie.add_member(leaf_of(added)).unwrap().recalculate().unwrap();
        let envelope = Envelope::build(the_org(), SequenceNumber::new(base_epoch + 1), &delta).unwrap();
        let body = encode_message(&WireMessage::OrgInformation {
            envelope,
            record_snapshot: postcard::to_allocvec(&snapshots_of(&base_trie)).unwrap(),
            org_private_key: OrgPrivateKey::from(key),
        })
        .unwrap();
        (body, next.root_hash().unwrap(), public_key(key))
    }

    /// `receive_into` the store of the Device of seed `device`.
    pub fn receive(svc: &mut OrgService, device: u8, sender: u8, body: &[u8]) -> Epoch {
        receive_into(svc, device_key(device), sender, body)
    }

    /// `body` received by the store `key` names from the Device of seed
    /// `sender`, which must keep it as a provisional organisation; returns
    /// its epoch.
    pub fn receive_into(svc: &mut OrgService, key: DevicePublicKey, sender: u8, body: &[u8]) -> Epoch {
        match svc.check_received(&mut OsRng, &key, device_key(sender), body).unwrap() {
            (Checked::ProvisionalOrganisationKept { epoch }, Some(_sealed)) => epoch,
            _ => panic!("the Organisation information is not kept"),
        }
    }

    /// The request `member`'s Device sent for `invite_byte`, kept.
    pub fn request(invite_byte: u8, member: &MemberSnapshot, handles: &[&str]) -> KeptAdmissionRequest {
        KeptAdmissionRequest {
            org_id: the_org(),
            invite_id: invite_id_of(invite_byte),
            from: member.device_keys[0],
            name: member.name.clone(),
            surname: member.surname.clone(),
            preferred_handles: handles.iter().map(|handle| Handle::parse(handle).unwrap()).collect(),
            member_key: member.member_key,
            device_key: member.device_keys[0],
            addresses: addresses(&[&format!("{}.example:4433", member.handle.as_str())]),
        }
    }

    /// Ada's store at epoch 3 listing `members`, with Carol's request
    /// (invite `0x11`) and Dave's (invite `0x12`) kept and outstanding.
    pub fn admin_store(members: &[MemberSnapshot]) -> StoreData {
        let mut store_data = ada_store_in_trie_order(3, members);
        store_data.kept_admission_requests = vec![request(0x11, &carol(), &["carol"]), request(0x12, &dave(), &["carol", "dave"])];
        store_data.outstanding_invites = vec![
            OutstandingInvite { org_id: the_org(), invite_id: invite_id_of(0x11), invitee_name: "Carol".to_string() },
            OutstandingInvite { org_id: the_org(), invite_id: invite_id_of(0x12), invitee_name: "Dave".to_string() },
        ];
        store_data
    }

    /// Ada's store with Carol's admission recorded and proposed; the
    /// chain state that holds the proposal. The service holds the store,
    /// so the sealed writes are not read.
    pub fn own_proposal(members: &[MemberSnapshot]) -> (OrgService, OrgState) {
        let mut svc = service(ADA, admin_store(members));
        crate::support::unread(svc.record_admission(&mut OsRng, &device_key(ADA), invite_id_of(0x11), None).unwrap());
        crate::support::unread(svc.propose_updated_organisation(&mut OsRng, &device_key(ADA)).unwrap());
        let held = &held(&svc, ADA).provisional_organisations[0];
        let state = chain(held.epoch.get(), held.root, held.org_pub_key);
        (svc, state)
    }

    /// `svc`'s one store, given the `origin` its first provisional
    /// organisation was not kept with: a store org-io would unseal.
    pub fn with_origin(svc: &OrgService, device: u8, origin: Origin) -> OrgService {
        let mut moved = held(svc, device).clone();
        moved.provisional_organisations[0].origin = origin;
        service(device, moved)
    }

    /// A notice of a Member's absence, for a kept notice the store holds.
    pub fn absence_notice(members: &[MemberSnapshot]) -> RevocationNotice {
        let member_id = MemberId::new([0x09; 32]);
        let device = device_key(0x99);
        let proof = trie_of_members(members).prove_absent(&member_id, &device).unwrap();
        RevocationNotice { org_id: the_org(), member_id, device, proof }
    }

    pub fn revoked(member: &MemberSnapshot, slot: usize) -> RevokedDevice {
        RevokedDevice { member_id: member.id, device_key: member.device_keys[slot] }
    }

    /// The member snapshot listing the provisional Member's keys: MemberId
    /// `0x33`, Handle carol.
    pub fn provisional_member_listed(svc: &OrgService, key: DevicePublicKey) -> MemberSnapshot {
        let member = held_store(svc, &key).member.as_ref().unwrap();
        // Built here: every field but the id and Handle is the provisional Member's, read from its store.
        MemberSnapshot {
            id: MemberId::new([0x33; 32]),
            handle: Handle::parse("carol").unwrap(),
            name: member.name.clone(),
            surname: member.surname.clone(),
            member_key: member.member_key,
            device_keys: vec![member.device_key],
        }
    }

    /// Ada's founding store with its proxy recorded, its Device secret,
    /// and the chain state its write leaves at epoch 1; the service holds
    /// the store, so the sealed writes are not read.
    pub fn founding() -> (OrgService, DevicePublicKey, DeviceSecret, OrgState) {
        let mut svc = OrgService::new(Vec::new()).unwrap();
        let (key, secret, _sealed) = svc.propose_founding(&mut OsRng, "Ada", "Lovelace", "ada", "Acme".to_string()).unwrap();
        crate::support::unread(svc.record_founding_proxy(&mut OsRng, &key, the_org(), crate::support::the_proxy()).unwrap());
        let held = held_store(&svc, &key).founding.as_ref().unwrap();
        let state = chain(1, held.root, held.org_pub_key);
        (svc, key, secret, state)
    }
}

// ---- judgement without adoption, and the behind marker -------------------------

// No chain state is `OrgNotOnChain`; a founding store whose proxy is not
// recorded is `FoundingNotWritten` whatever state is passed, since it has
// no Organisation to read; neither asks a secret.
// verifies: LLR-rr558z
#[test]
fn no_chain_state_is_org_not_on_chain_and_a_founding_without_proxy_is_founding_not_written() {
    let asked = Cell::new(false);
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]));
    assert_eq!(stores::verify(&mut svc, stores::ADA, None, &asked).unwrap_err(), OrgNodeError::OrgNotOnChain);

    let mut founder = OrgService::new(Vec::new()).unwrap();
    let (key, _secret, store_change) = founder.propose_founding(&mut OsRng, "Ada", "Lovelace", "ada", "Acme".to_string()).unwrap();
    crate::support::unread(store_change);
    let (root, org_pub_key) = {
        let founding = founder.store_data(&key).unwrap().founding.as_ref().unwrap();
        (founding.root, founding.org_pub_key)
    };
    for state in [None, Some(stores::chain(1, root, org_pub_key))] {
        let refused = founder.verify(&mut OsRng, &key, state, org_node::VerificationTime(1), org_node::AheadLimit(1200), |_: &[org_node::DevicePublicKey]| {
            asked.set(true);
            Vec::<org_node::DeviceSecret>::new()
        });
        assert_eq!(refused.unwrap_err(), OrgNodeError::FoundingNotWritten);
    }
    assert!(!asked.get(), "no secret asked");
}

// A chain state behind the record, or at its epoch with another root, is
// `Inconsistent`, naming which.
// verifies: LLR-rr558z
#[test]
fn a_chain_behind_the_record_or_at_its_epoch_with_another_root_is_inconsistent() {
    let asked = Cell::new(false);
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]));
    let (root, org_pub_key) = (stores::record_root(&svc, stores::ADA), stores::public_key(stores::RECORD_KEY));
    let (behind, change) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(2, root, org_pub_key)), &asked).unwrap();
    assert_eq!(
        behind,
        Verified::Inconsistent {
            reason: Inconsistency::ChainBehindRecord { chain_epoch: Epoch::new(2), record_epoch: Epoch::new(3) }
        }
    );
    assert!(change.is_none());
    let (other, change) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(3, RootHash::new([0x5e; 32]), org_pub_key)), &asked).unwrap();
    assert_eq!(other, Verified::Inconsistent { reason: Inconsistency::OtherRoot { epoch: Epoch::new(3) } });
    assert!(change.is_none());
}

// The chain at the record's epoch holding the record's root is in step.
// verifies: LLR-rr558z
#[test]
fn the_records_epoch_and_root_are_in_step() {
    let asked = Cell::new(false);
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]));
    let root = stores::record_root(&svc, stores::ADA);
    let (verified, change) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(3, root, stores::public_key(stores::RECORD_KEY))), &asked).unwrap();
    assert_eq!(verified, Verified::InStep);
    assert!(change.is_none());
}

// A greater epoch that no held provisional organisation matches is
// `Behind`; so is any state but the founding's in a founding store with
// its proxy recorded.
// verifies: LLR-rr558z
#[test]
fn a_greater_epoch_with_no_match_is_behind() {
    let asked = Cell::new(false);
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]));
    let root = stores::record_root(&svc, stores::ADA);
    let (verified, store_change) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(4, root, stores::public_key([0x4c; 32]))), &asked).unwrap();
    crate::support::unread(store_change);
    assert_eq!(verified, Verified::Behind { chain_epoch: Epoch::new(4) });

    let (mut founder, key, _secret, founded) = stores::founding();
    let org_pub_key = founded.org_pub_key;
    for state in [stores::chain(1, RootHash::new([0x5e; 32]), org_pub_key), stores::chain(2, RootHash::new([0x5e; 32]), org_pub_key)] {
        let (verified, change) = founder
            .verify(&mut OsRng, &key, Some(state), org_node::VerificationTime(1), org_node::AheadLimit(1200), crate::support::none_held)
            .unwrap();
        assert_eq!(verified, Verified::Behind { chain_epoch: state.epoch });
        assert!(change.is_none());
    }
}

// `InStep`, `Behind`, `Inconsistent` and the refusal change no sealed
// byte, return no store change and never ask a secret.
// verifies: LLR-qvp7mq
#[test]
fn in_step_behind_inconsistent_and_refusals_change_no_sealed_byte_and_ask_no_secret() {
    let asked = Cell::new(false);
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]));
    let (root, org_pub_key) = (stores::record_root(&svc, stores::ADA), stores::public_key(stores::RECORD_KEY));
    let before = stores::encoded(&svc, stores::ADA);
    for state in [
        None,
        Some(stores::chain(2, root, org_pub_key)),
        Some(stores::chain(3, RootHash::new([0x5e; 32]), org_pub_key)),
        Some(stores::chain(3, root, org_pub_key)),
        Some(stores::chain(4, root, org_pub_key)),
    ] {
        match stores::verify(&mut svc, stores::ADA, state, &asked) {
            Ok((_verified, change)) => assert!(change.is_none(), "no store change for {state:?}"),
            Err(error) => assert_eq!(error, OrgNodeError::OrgNotOnChain),
        }
        assert_eq!(stores::encoded(&svc, stores::ADA), before, "no sealed byte changes for {state:?}");
    }
    assert!(!asked.get(), "no secret asked");
}

// `Behind` sets the marker and `InStep` clears it, changing no sealed
// byte; a store unsealed holds no marker.
// verifies: LLR-zk7fx2
#[test]
fn behind_sets_the_marker_in_step_clears_it_and_unseal_leaves_it_none() {
    let asked = Cell::new(false);
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]));
    let (root, org_pub_key) = (stores::record_root(&svc, stores::ADA), stores::public_key(stores::RECORD_KEY));
    let before = stores::encoded(&svc, stores::ADA);
    let (behind, change) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(4, root, org_pub_key)), &asked).unwrap();
    assert!(matches!(behind, Verified::Behind { .. }) && change.is_none(), "{behind:?}");
    assert_eq!(stores::held(&svc, stores::ADA).behind, Some(Epoch::new(4)));
    assert_eq!(stores::encoded(&svc, stores::ADA), before, "the marker is not sealed");
    let (in_step, change) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(3, root, org_pub_key)), &asked).unwrap();
    assert!(matches!(in_step, Verified::InStep) && change.is_none(), "{in_step:?}");
    assert_eq!(stores::held(&svc, stores::ADA).behind, None);

    let mut marked = stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]);
    marked.behind = Some(Epoch::new(4));
    let store = store_for_test(&device_secret([stores::ADA; 32]), marked);
    let write = seal_for_test(&store, &mut OsRng);
    let reopened = PersonaStore::unseal(&write.sealed, device_key(stores::ADA), device_secret([stores::ADA; 32])).unwrap();
    assert_eq!(reopened.data().unwrap().behind, None);
}

// An `Inconsistent` verification leaves the behind marker as it was: unset
// when unset, and at the earlier `Behind`'s epoch when set (merge review
// MR3, finding-5).
// verifies: LLR-zk7fx2, LLR-qvp7mq
#[test]
fn an_inconsistent_verification_leaves_the_behind_marker_as_it_was() {
    let asked = Cell::new(false);
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]));
    let (root, org_pub_key) = (stores::record_root(&svc, stores::ADA), stores::public_key(stores::RECORD_KEY));
    let inconsistent = [stores::chain(2, root, org_pub_key), stores::chain(3, RootHash::new([0x5e; 32]), org_pub_key)];
    for state in inconsistent {
        let (verified, change) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
        assert!(matches!(verified, Verified::Inconsistent { .. }) && change.is_none(), "{verified:?}");
        assert_eq!(stores::held(&svc, stores::ADA).behind, None, "an unset marker stays unset after {state:?}");
    }
    let (behind, _) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(4, root, org_pub_key)), &asked).unwrap();
    assert!(matches!(behind, Verified::Behind { .. }), "{behind:?}");
    for state in inconsistent {
        let (verified, change) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
        assert!(matches!(verified, Verified::Inconsistent { .. }) && change.is_none(), "{verified:?}");
        assert_eq!(stores::held(&svc, stores::ADA).behind, Some(Epoch::new(4)), "a set marker stays set after {state:?}");
    }
}

// A verification that adopts nothing leaves the record's kept Change set.
// verifies: LLR-qvp7mq
#[test]
fn a_refused_verification_keeps_the_earlier_change_set() {
    let asked = Cell::new(false);
    let mut store_data = stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]);
    store_data.record.as_mut().unwrap().kept_change_set = Some(vec![1, 2, 3]);
    let mut svc = stores::service(stores::ADA, store_data);
    let (root, org_pub_key) = (stores::record_root(&svc, stores::ADA), stores::public_key(stores::RECORD_KEY));
    for state in [None, Some(stores::chain(2, root, org_pub_key)), Some(stores::chain(3, RootHash::new([0x5e; 32]), org_pub_key)), Some(stores::chain(4, root, org_pub_key))] {
        // Each state is judged differently, some refused; whatever the
        // outcome, the kept Change set is the one the record held.
        let _outcome = stores::verify(&mut svc, stores::ADA, state, &asked);
        assert_eq!(stores::held(&svc, stores::ADA).record.as_ref().unwrap().kept_change_set, Some(vec![1, 2, 3]));
    }
}

// Each pair of chain epoch and root gives exactly the one outcome LLR-rr558z states.
// verifies: LLR-rr558z
#[test]
fn verify_reports_one_outcome_from_the_record_and_the_state() {
    let asked = Cell::new(false);
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]));
    let (root, org_pub_key) = (stores::record_root(&svc, stores::ADA), stores::public_key(stores::RECORD_KEY));
    let other = RootHash::new([0x5e; 32]);
    for (epoch, chain_root, expected) in [
        (1, root, Verified::Inconsistent { reason: Inconsistency::ChainBehindRecord { chain_epoch: Epoch::new(1), record_epoch: Epoch::new(3) } }),
        (3, other, Verified::Inconsistent { reason: Inconsistency::OtherRoot { epoch: Epoch::new(3) } }),
        (3, root, Verified::InStep),
        (5, other, Verified::Behind { chain_epoch: Epoch::new(5) }),
    ] {
        let (verified, store_change) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(epoch, chain_root, org_pub_key)), &asked).unwrap();
        crate::support::unread(store_change);
        assert_eq!(verified, expected, "epoch {epoch}");
    }
}

// A received provisional organisation for the chain's epoch is no match
// when the chain holds its root with another key, or its key with
// another root: `Behind`, and, since it can never be adopted, it is
// discarded in a sealed change (LLR-4tsgfh; until merge review MR3-N1 the
// store kept it).
// verifies: LLR-rr558z, LLR-4tsgfh
#[test]
fn verify_is_behind_when_no_provisional_organisation_matches_root_and_key() {
    let asked = Cell::new(false);
    let (body, next_root, next_key) = stores::information(3, &[stores::ada(), stores::bob()], &stores::carol(), [0x4c; 32]);
    for state in [stores::chain(4, next_root, stores::public_key([0x4d; 32])), stores::chain(4, RootHash::new([0x5e; 32]), next_key)] {
        let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob()]));
        assert_eq!(stores::receive(&mut svc, stores::ADA, stores::BOB, &body), Epoch::new(4));
        let (verified, change) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
        assert_eq!(verified, Verified::Behind { chain_epoch: Epoch::new(4) });
        assert!(change.is_some(), "the discard is sealed");
        assert!(stores::held(&svc, stores::ADA).provisional_organisations.is_empty(), "discarded for {state:?}");
    }
}

// ---- adopting a matched provisional organisation ------------------------------

// Of two received provisional organisations for epoch 4, the one whose
// epoch, root and key the chain holds is adopted; the other is dropped.
// verifies: LLR-8528ud
#[test]
fn the_match_is_the_one_provisional_organisation_with_the_chains_epoch_root_and_key() {
    let asked = Cell::new(false);
    let base = [stores::ada(), stores::bob()];
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &base));
    let (carol_body, carol_root, carol_key) = stores::information(3, &base, &stores::carol(), [0x4c; 32]);
    let (dave_body, _dave_root, _dave_key) = stores::information(3, &base, &stores::dave(), [0x4d; 32]);
    stores::receive(&mut svc, stores::ADA, stores::BOB, &carol_body);
    stores::receive(&mut svc, stores::ADA, stores::BOB, &dave_body);
    let (verified, change) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(4, carol_root, carol_key)), &asked).unwrap();
    assert_eq!(verified, Verified::Adopted { epoch: Epoch::new(4), discarded: Vec::new(), own: false });
    assert!(change.is_some());
    let record = stores::held(&svc, stores::ADA).record.as_ref().unwrap();
    assert_eq!((record.epoch, record.root_hash, record.org_pub_key), (Epoch::new(4), carol_root, carol_key));
    assert!(stores::held(&svc, stores::ADA).provisional_organisations.is_empty(), "the other, for the same epoch, is dropped");
}

// An own, a received and an answer provisional organisation are matched
// alike; only the own one reports `own`.
// verifies: LLR-8528ud
#[test]
fn own_received_and_answer_provisional_organisations_match_alike() {
    let asked = Cell::new(false);
    let base = [stores::ada(), stores::bob()];
    let (mut own, own_state) = stores::own_proposal(&base);
    let (verified, store_change) = stores::verify(&mut own, stores::ADA, Some(own_state), &asked).unwrap();
    crate::support::unread(store_change);
    assert_eq!(verified, Verified::Adopted { epoch: Epoch::new(4), discarded: Vec::new(), own: true });

    let mut received = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &base));
    let (body, root, org_pub_key) = stores::information(3, &base, &stores::carol(), [0x4c; 32]);
    stores::receive(&mut received, stores::ADA, stores::BOB, &body);
    let mut answer = stores::with_origin(&received, stores::ADA, Origin::Answer { from: device_key(stores::BOB) });
    for svc in [&mut received, &mut answer] {
        let (verified, sealed) = stores::verify(svc, stores::ADA, Some(stores::chain(4, root, org_pub_key)), &asked).unwrap();
        assert_eq!(verified, Verified::Adopted { epoch: Epoch::new(4), discarded: Vec::new(), own: false });
        assert!(sealed.is_some());
    }
}

// Adoption replaces the record's members, root, key pair and epoch with
// the match's, clears the marker, and returns one sealed change holding
// all of it.
// verifies: LLR-34c5y3, REQ-mr5abb
#[test]
fn adoption_replaces_the_record_and_reports_own_in_one_sealed_change() {
    let asked = Cell::new(false);
    let (mut svc, state) = stores::own_proposal(&[stores::ada(), stores::bob()]);
    let members = stores::held(&svc, stores::ADA).provisional_organisations[0].members.clone();
    let ahead = stores::chain(5, RootHash::new([0x5e; 32]), state.org_pub_key);
    let (behind, written) = stores::verify(&mut svc, stores::ADA, Some(ahead), &asked).unwrap();
    assert_eq!((behind, written.is_none()), (Verified::Behind { chain_epoch: Epoch::new(5) }, true));
    assert_eq!(stores::held(&svc, stores::ADA).behind, Some(Epoch::new(5)), "fixture: the marker is set");
    let (verified, change) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
    assert_eq!(verified, Verified::Adopted { epoch: Epoch::new(4), discarded: Vec::new(), own: true });
    let Some(org_node::service::StoreChange::Sealed(write)) = change else { panic!("one sealed change") };
    let reopened = PersonaStore::unseal(&write.sealed, device_key(stores::ADA), device_secret([stores::ADA; 32])).unwrap();
    let record = reopened.data().unwrap().record.as_ref().unwrap();
    assert_eq!((record.epoch, record.last_seq.get()), (Epoch::new(4), 4));
    assert_eq!((record.root_hash, record.org_pub_key), (state.root_hash, state.org_pub_key));
    assert_eq!(record.trie_members, members);
    assert_eq!(
        record.org_private_key_for_test().x25519_keypair().org_public_key().unwrap(),
        state.org_pub_key,
        "the record holds the private half of the adopted key"
    );
    assert_eq!(stores::held(&svc, stores::ADA).behind, None, "the adoption clears the marker");
}

// Changes recorded after the proposal (Dave's admission, Eve's
// revocation) are not in the adopted record: removed and reported in
// recorded order; the included ones are removed and not reported.
// verifies: LLR-d9x6xf
#[test]
fn changes_the_adopted_record_left_out_are_discarded_and_reported_in_order() {
    let asked = Cell::new(false);
    let (mut svc, state) = stores::own_proposal(&[stores::ada(), stores::bob(), stores::eve()]);
    crate::support::unread(svc.record_admission(&mut OsRng, &device_key(stores::ADA), stores::invite_id_of(0x12), None).unwrap());
    crate::support::unread(svc.record_revocation(&mut OsRng, &device_key(stores::ADA), stores::eve().id).unwrap());
    let (verified, store_change) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
    crate::support::unread(store_change);
    assert_eq!(
        verified,
        Verified::Adopted {
            epoch: Epoch::new(4),
            discarded: vec![
                DiscardedChange::Admit { invite_id: stores::invite_id_of(0x12), handle: org_node::Handle::parse("dave").unwrap() },
                DiscardedChange::Revoke { member_id: stores::eve().id },
            ],
            own: true,
        }
    );
    assert!(stores::held(&svc, stores::ADA).provisional_changes.is_empty());
}

// Abnormal, at the boundary of the inclusion rule: an `Admit` is included
// only when one adopted member holds both of the kept request's keys. With
// the kept request naming Carol's Member-as-a-group key and another
// DevicePublicKey, or Carol's DevicePublicKey and another Member-as-a-group
// key, the record that lists Carol does not include it, so it is removed
// and reported.
// verifies: LLR-d9x6xf
#[test]
fn an_admit_whose_request_matches_a_listed_member_on_one_key_only_is_discarded() {
    let asked = Cell::new(false);
    let carol_admit = DiscardedChange::Admit { invite_id: stores::invite_id_of(0x11), handle: org_node::Handle::parse("carol").unwrap() };
    for (case, other_device, other_member_key) in [("another Device", true, false), ("another member key", false, true)] {
        let (proposed, state) = stores::own_proposal(&[stores::ada(), stores::bob()]);
        let mut held = stores::held(&proposed, stores::ADA).clone();
        let request = held.kept_admission_requests.iter_mut().find(|request| request.invite_id == stores::invite_id_of(0x11)).unwrap();
        if other_device {
            request.device_key = device_key(0x7d);
        }
        if other_member_key {
            request.member_key = stores::dave().member_key;
        }
        let mut svc = stores::service(stores::ADA, held);
        let (verified, store_change) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
        crate::support::unread(store_change);
        assert_eq!(
            verified,
            Verified::Adopted { epoch: Epoch::new(4), discarded: vec![carol_admit.clone()], own: true },
            "{case}"
        );
        assert!(stores::held(&svc, stores::ADA).provisional_changes.is_empty(), "{case}: removed");
    }
}

// Abnormal, at the boundary of the inclusion rule: a `Revoke` is included
// exactly when the adopted record holds no member with its MemberId. Eve's
// revocation, proposed and adopted, is included and not reported; Bob's,
// recorded after the proposal so the adopted record still holds Bob, is
// removed and reported.
// verifies: LLR-d9x6xf
#[test]
fn a_revoke_whose_member_id_the_adopted_record_still_holds_is_discarded() {
    let asked = Cell::new(false);
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &[stores::ada(), stores::bob(), stores::eve()]));
    crate::support::unread(svc.record_revocation(&mut OsRng, &device_key(stores::ADA), stores::eve().id).unwrap());
    crate::support::unread(svc.propose_updated_organisation(&mut OsRng, &device_key(stores::ADA)).unwrap());
    let held = &stores::held(&svc, stores::ADA).provisional_organisations[0];
    let state = stores::chain(held.epoch.get(), held.root, held.org_pub_key);
    crate::support::unread(svc.record_revocation(&mut OsRng, &device_key(stores::ADA), stores::bob().id).unwrap());
    let (verified, store_change) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
    crate::support::unread(store_change);
    assert_eq!(
        verified,
        Verified::Adopted { epoch: Epoch::new(4), discarded: vec![DiscardedChange::Revoke { member_id: stores::bob().id }], own: true }
    );
    let record = stores::held(&svc, stores::ADA).record.clone().unwrap();
    assert!(!record.trie_members.iter().any(|member| member.id == stores::eve().id), "Eve's revocation is in the record");
    assert!(record.trie_members.iter().any(|member| member.id == stores::bob().id), "Bob is still held");
    assert!(stores::held(&svc, stores::ADA).provisional_changes.is_empty());
}

// Adoption at epoch 4 drops every provisional organisation at or below
// epoch 4 and every kept notice, and keeps the one for epoch 5.
// verifies: LLR-x8drb8
#[test]
fn adoption_drops_provisional_organisations_up_to_its_epoch_and_every_kept_notice() {
    let asked = Cell::new(false);
    let base = [stores::ada(), stores::bob()];
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &base));
    let (carol_body, root, org_pub_key) = stores::information(3, &base, &stores::carol(), [0x4c; 32]);
    let (dave_body, _dave_root, _dave_key) = stores::information(3, &base, &stores::dave(), [0x4d; 32]);
    stores::receive(&mut svc, stores::ADA, stores::BOB, &carol_body);
    stores::receive(&mut svc, stores::ADA, stores::BOB, &dave_body);
    let mut held = stores::held(&svc, stores::ADA).clone();
    held.provisional_organisations[1].epoch = Epoch::new(5);
    held.kept_revocation_notices.push(org_node::store::KeptNotice { notice: stores::absence_notice(&base), from: device_key(0x21) });
    let mut svc = stores::service(stores::ADA, held);
    let (adopted, sealed) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(4, root, org_pub_key)), &asked).unwrap();
    assert!(matches!(adopted, Verified::Adopted { .. }), "{adopted:?}");
    assert!(sealed.is_some(), "an adoption seals the store");
    let store_data = stores::held(&svc, stores::ADA);
    assert_eq!(store_data.provisional_organisations.len(), 1);
    assert_eq!(store_data.provisional_organisations[0].epoch, Epoch::new(5), "a later epoch's is kept");
    assert!(store_data.kept_revocation_notices.is_empty(), "every kept notice goes");
}

// A non-matching provisional organisation for the adopted epoch and a
// stale one below it go with the adoption.
// verifies: LLR-x8drb8
#[test]
fn an_adoption_discards_the_provisional_organisations_at_or_below_its_epoch() {
    let asked = Cell::new(false);
    let base = [stores::ada(), stores::bob()];
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &base));
    let (carol_body, root, org_pub_key) = stores::information(3, &base, &stores::carol(), [0x4c; 32]);
    let (dave_body, _dave_root, _dave_key) = stores::information(3, &base, &stores::dave(), [0x4d; 32]);
    stores::receive(&mut svc, stores::ADA, stores::BOB, &carol_body);
    stores::receive(&mut svc, stores::ADA, stores::BOB, &dave_body);
    let mut held = stores::held(&svc, stores::ADA).clone();
    let mut stale = held.provisional_organisations[1].clone();
    stale.epoch = Epoch::new(3);
    stale.root = RootHash::new([0x5f; 32]);
    held.provisional_organisations.push(stale);
    let mut svc = stores::service(stores::ADA, held);
    let (adopted, sealed) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(4, root, org_pub_key)), &asked).unwrap();
    assert!(matches!(adopted, Verified::Adopted { .. }), "{adopted:?}");
    assert!(sealed.is_some(), "an adoption seals the store");
    assert!(stores::held(&svc, stores::ADA).provisional_organisations.is_empty());
}

// An own and a received adoption each keep the Change set and the base
// snapshot of the provisional organisation they adopted.
// verifies: LLR-p8vpgz, LLR-34c5y3
#[test]
fn every_adoption_keeps_the_change_set_and_base_snapshot_that_produced_the_record() {
    let asked = Cell::new(false);
    let base = [stores::ada(), stores::bob()];
    let (mut own, state) = stores::own_proposal(&base);
    let held = stores::held(&own, stores::ADA).provisional_organisations[0].clone();
    let (adopted, sealed) = stores::verify(&mut own, stores::ADA, Some(state), &asked).unwrap();
    assert!(matches!(adopted, Verified::Adopted { .. }), "{adopted:?}");
    assert!(sealed.is_some(), "an adoption seals the store");
    let record = stores::held(&own, stores::ADA).record.clone().unwrap();
    assert_eq!((record.kept_change_set, record.kept_base_snapshot), (held.change_set, held.base_snapshot));

    let mut received = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &base));
    let (body, root, org_pub_key) = stores::information(3, &base, &stores::carol(), [0x4c; 32]);
    stores::receive(&mut received, stores::ADA, stores::BOB, &body);
    let held = stores::held(&received, stores::ADA).provisional_organisations[0].clone();
    let (adopted, sealed) = stores::verify(&mut received, stores::ADA, Some(stores::chain(4, root, org_pub_key)), &asked).unwrap();
    assert!(matches!(adopted, Verified::Adopted { .. }), "{adopted:?}");
    assert!(sealed.is_some(), "an adoption seals the store");
    let record = stores::held(&received, stores::ADA).record.clone().unwrap();
    assert_eq!(record.kept_change_set, held.change_set);
    assert_eq!(record.kept_base_snapshot.unwrap(), postcard::to_allocvec(&support::snapshots_of(&support::trie_of_members(&base))).unwrap());
}

// A provisional organisation with the chain's epoch and root but another
// key is no match: nothing is adopted, the record is as it was, and the
// unmatched one is discarded (LLR-4tsgfh; until merge review MR3-N1 the
// whole store was as it was).
// verifies: LLR-8528ud, LLR-4tsgfh
#[test]
fn a_provisional_organisation_whose_key_is_not_the_chains_is_no_match() {
    let asked = Cell::new(false);
    let base = [stores::ada(), stores::bob()];
    let mut svc = stores::service(stores::ADA, stores::ada_store_in_trie_order(3, &base));
    let (body, root, _key) = stores::information(3, &base, &stores::carol(), [0x4c; 32]);
    stores::receive(&mut svc, stores::ADA, stores::BOB, &body);
    let record_before = stores::held(&svc, stores::ADA).record.as_ref().unwrap().root_hash;
    let (verified, change) = stores::verify(&mut svc, stores::ADA, Some(stores::chain(4, root, stores::public_key([0x4e; 32]))), &asked).unwrap();
    assert_eq!(verified, Verified::Behind { chain_epoch: Epoch::new(4) });
    assert!(change.is_some(), "the discard is sealed");
    assert_eq!(stores::held(&svc, stores::ADA).record.as_ref().unwrap().root_hash, record_before, "nothing adopted");
    assert!(stores::held(&svc, stores::ADA).provisional_organisations.is_empty());
}

// The proposal's sealed store, unsealed after a restart, adopts the
// provisional organisation the chain holds.
// verifies: LLR-34c5y3
#[test]
fn verify_adopts_the_provisional_organisation_the_chain_holds_after_a_restart() {
    let asked = Cell::new(false);
    let mut svc = stores::service(stores::ADA, stores::admin_store(&[stores::ada(), stores::bob()]));
    crate::support::unread(svc.record_admission(&mut OsRng, &device_key(stores::ADA), stores::invite_id_of(0x11), None).unwrap());
    let org_node::service::StoreChange::Sealed(write) = svc.propose_updated_organisation(&mut OsRng, &device_key(stores::ADA)).unwrap();
    let held = &stores::held(&svc, stores::ADA).provisional_organisations[0];
    let state = stores::chain(4, held.root, held.org_pub_key);
    let reopened = PersonaStore::unseal(&write.sealed, device_key(stores::ADA), device_secret([stores::ADA; 32])).unwrap();
    // Built here: the service is reopened from the sealed write, as org-io restarts it.
    let mut restarted = OrgService::new(vec![reopened]).unwrap();
    let (verified, store_change) = stores::verify(&mut restarted, stores::ADA, Some(state), &asked).unwrap();
    crate::support::unread(store_change);
    assert_eq!(verified, Verified::Adopted { epoch: Epoch::new(4), discarded: Vec::new(), own: true });
}

// An adoption whose record still lists this Device never calls the
// secret source.
// verifies: LLR-34c5y3, LLR-5tvn7g
#[test]
fn an_adoption_that_keeps_this_device_never_asks_for_the_device_secrets() {
    let asked = Cell::new(false);
    let (mut svc, state) = stores::own_proposal(&[stores::ada(), stores::bob()]);
    let (verified, store_change) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
    crate::support::unread(store_change);
    assert!(matches!(verified, Verified::Adopted { .. }));
    assert!(!asked.get(), "no secret asked");
}

// ---- what an adoption settles --------------------------------------------------

// Carol's request, whose keys the adopted record lists, is removed with
// its outstanding pair and its addresses move into the address book;
// Dave's, not admitted, stays with its pair.
// verifies: LLR-rzqs2u
#[test]
fn an_admitted_requests_invite_is_settled_and_its_addresses_move_to_the_address_book() {
    let asked = Cell::new(false);
    let (mut svc, state) = stores::own_proposal(&[stores::ada(), stores::bob()]);
    let carol_addresses = stores::request(0x11, &stores::carol(), &["carol"]).addresses;
    let (adopted, sealed) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
    assert!(matches!(adopted, Verified::Adopted { .. }), "{adopted:?}");
    assert!(sealed.is_some(), "an adoption seals the store");
    let store_data = stores::held(&svc, stores::ADA);
    let kept: Vec<_> = store_data.kept_admission_requests.iter().map(|request| request.invite_id).collect();
    assert_eq!(kept, vec![stores::invite_id_of(0x12)], "Dave's request stays");
    let outstanding: Vec<_> = store_data.outstanding_invites.iter().map(|invite| invite.invite_id).collect();
    assert_eq!(outstanding, vec![stores::invite_id_of(0x12)], "Carol's pair is settled");
    assert_eq!(store_data.address_book, vec![(device_key(stores::CAROL), carol_addresses)]);
}

// After the settlement, an admission request carrying Carol's invite id is
// refused as not outstanding before any field past the identifiers is
// read (the body holds nothing after them).
// verifies: LLR-rzqs2u, LLR-6gkxpg
#[test]
fn a_settled_invite_refuses_a_later_request_as_not_outstanding() {
    let asked = Cell::new(false);
    let (mut svc, state) = stores::own_proposal(&[stores::ada(), stores::bob()]);
    let (adopted, sealed) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
    assert!(matches!(adopted, Verified::Adopted { .. }), "{adopted:?}");
    assert!(sealed.is_some(), "an adoption seals the store");
    let mut body = vec![3u8];
    body.extend_from_slice(support::the_org().as_bytes());
    body.extend_from_slice(stores::invite_id_of(0x11).as_bytes());
    body.extend_from_slice(&[0u8; 4]);
    let before = stores::encoded(&svc, stores::ADA);
    let refused = svc.check_received(&mut OsRng, &device_key(stores::ADA), device_key(stores::CAROL), &body);
    assert!(matches!(refused, Err(OrgNodeError::InviteNotOutstanding { invite_id, .. }) if invite_id == stores::invite_id_of(0x11)));
    assert_eq!(stores::encoded(&svc, stores::ADA), before);
}

// Bob has two Devices; revoking him drops both, and the adoption holds
// each as revoked, in slot order, except the one already held.
// verifies: LLR-bgj2ff
#[test]
fn an_adoption_holds_each_dropped_device_as_revoked_once() {
    let asked = Cell::new(false);
    let bob = stores::snapshot(0x02, "bob", 0x62, &[stores::BOB, 0xb2, 0xb3]);
    let mut store_data = stores::admin_store(&[stores::ada(), bob.clone()]);
    store_data.revoked_devices.push(stores::revoked(&bob, 1));
    let mut svc = stores::service(stores::ADA, store_data);
    crate::support::unread(svc.record_revocation(&mut OsRng, &device_key(stores::ADA), bob.id).unwrap());
    crate::support::unread(svc.propose_updated_organisation(&mut OsRng, &device_key(stores::ADA)).unwrap());
    let held = &stores::held(&svc, stores::ADA).provisional_organisations[0];
    let state = stores::chain(4, held.root, held.org_pub_key);
    let (adopted, sealed) = stores::verify(&mut svc, stores::ADA, Some(state), &asked).unwrap();
    assert!(matches!(adopted, Verified::Adopted { .. }), "{adopted:?}");
    assert!(sealed.is_some(), "an adoption seals the store");
    assert_eq!(
        stores::held(&svc, stores::ADA).revoked_devices,
        vec![stores::revoked(&bob, 1), stores::revoked(&bob, 0), stores::revoked(&bob, 2)]
    );
}

// `addresses_for` returns the address-book entry, else a kept request's
// addresses, else, in a provisional Member's store, the Invite's issuer
// addresses, else none; an unknown store is refused.
// verifies: LLR-pgy2rb
#[test]
fn addresses_for_prefers_the_address_book_then_a_kept_request_then_the_invite() {
    let carol = device_key(stores::CAROL);
    let from_book = stores::addresses(&["book.example:1"]);
    let mut booked = stores::admin_store(&[stores::ada(), stores::bob()]);
    booked.address_book.push((carol, from_book.clone()));
    let svc = stores::service(stores::ADA, booked);
    assert_eq!(svc.addresses_for(&device_key(stores::ADA), carol).unwrap(), Some(&from_book));

    let svc = stores::service(stores::ADA, stores::admin_store(&[stores::ada(), stores::bob()]));
    let requested = stores::request(0x11, &stores::carol(), &["carol"]).addresses;
    assert_eq!(svc.addresses_for(&device_key(stores::ADA), carol).unwrap(), Some(&requested));
    assert_eq!(svc.addresses_for(&device_key(stores::ADA), device_key(0x99)).unwrap(), None);

    let mut joiner = OrgService::new(Vec::new()).unwrap();
    let (key, _secret, store_change) = joiner.propose_member(&mut OsRng, &stores::invite_to_carol(), "Carol", "Coder", &["carol"]).unwrap();
    crate::support::unread(store_change);
    assert_eq!(
        joiner.addresses_for(&key, device_key(stores::ADA)).unwrap(),
        Some(&stores::addresses(&["ada.example:4433"]))
    );
    assert_eq!(joiner.addresses_for(&key, device_key(stores::BOB)).unwrap(), None);
    assert_eq!(joiner.addresses_for(&device_key(0x99), device_key(stores::ADA)).unwrap_err(),
        OrgNodeError::PersonaNotHeld { device_key: Box::new(device_key(0x99)) }
    );
}

// org-node holds endpoint addresses only as opaque text and dials,
// resolves and builds none (LLR-ds228z): no
// source file of `crates/org-node/src` names a transport address type, a peer
// address or a dial or resolve function, and `addresses_for` hands back
// exactly the strings a kept request carried, however little they look like
// an address.
// verifies: LLR-ds228z
#[test]
fn org_node_keeps_endpoint_addresses_as_opaque_text_and_dials_none() {
    let whole_word = |text: &str, word: &str| {
        text.match_indices(word).any(|(offset, _)| {
            let before = text[..offset].chars().next_back();
            let after = text[offset + word.len()..].chars().next();
            let ident = |neighbour: Option<char>| {
                neighbour.is_some_and(|character| character.is_alphanumeric() || character == '_')
            };
            !ident(before) && !ident(after)
        })
    };
    let sources = support::src_rust_files();
    assert!(sources.iter().any(|(path, _)| path.ends_with("service.rs")), "the scan reads crates/org-node/src");
    for (path, text) in &sources {
        for word in ["SocketAddr", "SocketAddrV4", "SocketAddrV6", "IpAddr", "EndpointAddr", "NodeAddr", "PeerAddr", "peer_addr", "node_addr_for_dial", "ToSocketAddrs"] {
            assert!(!whole_word(text, word), "{} names `{word}`: org-node holds no transport address", path.display());
        }
        for call in ["fn dial", "fn resolve", ".dial(", ".resolve(", "lookup_host", "TcpStream", "UdpSocket", "iroh::"] {
            assert!(!text.contains(call), "{} contains `{call}`: org-node dials and resolves nothing", path.display());
        }
    }

    let carol = device_key(stores::CAROL);
    let opaque = vec!["not an address at all".to_string(), "relay://whatever?x=1".to_string(), String::new()];
    let mut store_data = stores::admin_store(&[stores::ada(), stores::bob()]);
    let request = store_data.kept_admission_requests.iter_mut().find(|request| request.device_key == carol).unwrap();
    request.addresses = org_node::invitation::EndpointAddresses::parse(opaque.clone()).unwrap();
    let svc = stores::service(stores::ADA, store_data);
    let kept = svc.addresses_for(&device_key(stores::ADA), carol).unwrap().expect("the kept request's addresses");
    assert_eq!(kept.as_slice(), opaque.as_slice(), "the strings as given, unparsed and unchanged");
}

// ---- the first adoptions -------------------------------------------------------

// A provisional Member's store adopts the match listing its Member-as-a-group
// key and DevicePublicKey: the record is created, the Persona takes that
// member's MemberId and Handle, and the provisional Member and the
// expectation go.
// verifies: LLR-q4puca
#[test]
fn a_provisional_member_adopts_a_match_listing_its_keys_and_takes_its_member_id_and_handle() {
    let asked = Cell::new(false);
    let (mut svc, key) = stores::provisional_member();
    let listed = stores::provisional_member_listed(&svc, key);
    let (body, root, org_pub_key) = stores::information(1, &[stores::ada()], &listed, [0x4c; 32]);
    assert_eq!(stores::receive_into(&mut svc, key, stores::ADA, &body), Epoch::new(2));
    let (verified, change) = stores::verify_as(&mut svc, key, Some(stores::chain(2, root, org_pub_key)), &asked).unwrap();
    assert_eq!(verified, Verified::Adopted { epoch: Epoch::new(2), discarded: Vec::new(), own: false });
    assert!(change.is_some());
    let store_data = svc.store_data(&key).unwrap();
    assert_eq!(store_data.persona.member_id, Some(listed.id));
    assert_eq!(store_data.persona.handle, Some(listed.handle.clone()));
    assert!(store_data.member.is_none(), "the provisional Member goes");
    assert!(store_data.expected_admissions.is_empty(), "the expectation goes");
    let record = store_data.record.as_ref().unwrap();
    assert_eq!((record.org_id, record.epoch, record.root_hash), (support::the_org(), Epoch::new(2), root));
    assert_eq!(record.proxy_account, None);
}

// A match that does not list the provisional Member's keys is refused
// with `AdmissionNotOurs`, changing nothing.
// verifies: LLR-q4puca
#[test]
fn a_match_not_listing_the_provisional_member_is_admission_not_ours() {
    let asked = Cell::new(false);
    let (mut svc, key) = stores::provisional_member();
    let (body, root, org_pub_key) = stores::information(1, &[stores::ada()], &stores::dave(), [0x4c; 32]);
    stores::receive_into(&mut svc, key, stores::ADA, &body);
    let before = support::encoded_store(&svc, &key);
    let refused = stores::verify_as(&mut svc, key, Some(stores::chain(2, root, org_pub_key)), &asked);
    assert_eq!(refused.unwrap_err(), OrgNodeError::AdmissionNotOurs { org_id: support::the_org() });
    assert_eq!(support::encoded_store(&svc, &key), before);
    assert!(svc.store_data(&key).unwrap().member.is_some());
}

// The first record's Organisation public key is the chain's (equal to
// the key the match derived from its carried private key), and the
// record holds that private key.
// verifies: LLR-q4puca
#[test]
fn the_first_records_key_is_the_chains() {
    let asked = Cell::new(false);
    let (mut svc, key) = stores::provisional_member();
    let listed = stores::provisional_member_listed(&svc, key);
    let (body, root, org_pub_key) = stores::information(1, &[stores::ada()], &listed, [0x4c; 32]);
    stores::receive_into(&mut svc, key, stores::ADA, &body);
    let state = stores::chain(2, root, org_pub_key);
    let (adopted, sealed) = stores::verify_as(&mut svc, key, Some(state), &asked).unwrap();
    assert!(matches!(adopted, Verified::Adopted { .. }), "{adopted:?}");
    assert!(sealed.is_some(), "an adoption seals the store");
    let record = svc.store_data(&key).unwrap().record.as_ref().unwrap();
    assert_eq!(record.org_pub_key, state.org_pub_key);
    assert_eq!(record.org_private_key_for_test().x25519_keypair().org_public_key().unwrap(), state.org_pub_key);
}

// Regression: a provisional Member's first adoption moves the
// Invite issuer's DevicePublicKey and addresses into the address book in
// the adoption's own sealed change, so `addresses_for(issuer)` still
// answers once the provisional Member is gone.
// verifies: LLR-q4puca, LLR-h9d65p
#[test]
fn a_first_adoption_keeps_the_issuers_addresses_in_the_address_book() {
    let asked = Cell::new(false);
    let (mut svc, key) = stores::provisional_member();
    let listed = stores::provisional_member_listed(&svc, key);
    let (body, root, org_pub_key) = stores::information(1, &[stores::ada()], &listed, [0x4c; 32]);
    stores::receive_into(&mut svc, key, stores::ADA, &body);
    let (_adopted, change) = stores::verify_as(&mut svc, key, Some(stores::chain(2, root, org_pub_key)), &asked).unwrap();
    assert!(change.is_some());
    let issuer_addresses = stores::addresses(&["ada.example:4433"]);
    let store_data = svc.store_data(&key).unwrap();
    assert!(store_data.member.is_none(), "the provisional Member goes");
    assert_eq!(store_data.address_book, vec![(device_key(stores::ADA), issuer_addresses.clone())]);
    assert_eq!(svc.addresses_for(&key, device_key(stores::ADA)).unwrap(), Some(&issuer_addresses));
}

// A founding store adopts its founding at epoch 1: the record takes the
// founding's members, root and key pair, the proxy as `proxy_account`,
// no kept Change set or base snapshot; the Persona takes the founder's
// MemberId; the founding goes.
// verifies: LLR-gcm7bq
#[test]
fn a_founding_store_adopts_its_founding_at_epoch_one_with_its_proxy() {
    let asked = Cell::new(false);
    let (mut svc, key, _secret, state) = stores::founding();
    let members = svc.store_data(&key).unwrap().founding.as_ref().unwrap().members.clone();
    let (verified, change) = stores::verify_as(&mut svc, key, Some(state), &asked).unwrap();
    assert_eq!(verified, Verified::Adopted { epoch: Epoch::new(1), discarded: Vec::new(), own: true });
    assert!(change.is_some());
    let store_data = svc.store_data(&key).unwrap();
    let record = store_data.record.as_ref().unwrap();
    assert_eq!((record.org_id, record.epoch, record.last_seq.get()), (support::the_org(), Epoch::new(1), 1));
    assert_eq!((record.root_hash, record.org_pub_key), (state.root_hash, state.org_pub_key));
    assert_eq!(record.trie_members, members);
    assert_eq!(record.proxy_account, Some(support::the_proxy()));
    assert_eq!((record.kept_change_set.clone(), record.kept_base_snapshot.clone()), (None, None));
    let founder = members.iter().find(|member| member.device_keys.contains(&key)).unwrap().id;
    assert_eq!(store_data.persona.member_id, Some(founder));
    assert!(store_data.founding.is_none());
}

// Neither first adoption holds a Device as revoked.
// verifies: LLR-bgj2ff
#[test]
fn a_first_adoption_holds_no_device_as_revoked() {
    let asked = Cell::new(false);
    let (mut founder, founder_key, _secret, state) = stores::founding();
    let (adopted, sealed) = stores::verify_as(&mut founder, founder_key, Some(state), &asked).unwrap();
    assert!(matches!(adopted, Verified::Adopted { .. }), "{adopted:?}");
    assert!(sealed.is_some(), "an adoption seals the store");
    assert!(founder.store_data(&founder_key).unwrap().revoked_devices.is_empty());

    let (mut joiner, joiner_key) = stores::provisional_member();
    let listed = stores::provisional_member_listed(&joiner, joiner_key);
    let (body, root, org_pub_key) = stores::information(1, &[stores::ada(), stores::bob()], &listed, [0x4c; 32]);
    stores::receive_into(&mut joiner, joiner_key, stores::ADA, &body);
    let (adopted, sealed) = stores::verify_as(&mut joiner, joiner_key, Some(stores::chain(2, root, org_pub_key)), &asked).unwrap();
    assert!(matches!(adopted, Verified::Adopted { .. }), "{adopted:?}");
    assert!(sealed.is_some(), "an adoption seals the store");
    assert!(joiner.store_data(&joiner_key).unwrap().revoked_devices.is_empty());
}

// A chain state whose root is not the kept one's adopts nothing: `Behind`,
// no record, the provisional Member and its expectation kept; the kept
// information, which can never be adopted, is discarded (LLR-4tsgfh;
// until merge review MR3-N1 the whole store was as it was).
// verifies: REQ-eqt23j, LLR-q4puca, LLR-4tsgfh
#[test]
fn a_first_admission_whose_record_misses_the_chain_root_adopts_nothing() {
    let asked = Cell::new(false);
    let (mut svc, key) = stores::provisional_member();
    let listed = stores::provisional_member_listed(&svc, key);
    let (body, _root, org_pub_key) = stores::information(1, &[stores::ada()], &listed, [0x4c; 32]);
    stores::receive_into(&mut svc, key, stores::ADA, &body);
    let (verified, change) = stores::verify_as(&mut svc, key, Some(stores::chain(2, RootHash::new([0x5e; 32]), org_pub_key)), &asked).unwrap();
    assert_eq!(verified, Verified::Behind { chain_epoch: Epoch::new(2) });
    assert!(change.is_some(), "the discard is sealed");
    let store_data = svc.store_data(&key).unwrap();
    assert!(store_data.record.is_none());
    assert!(store_data.member.is_some() && !store_data.expected_admissions.is_empty(), "still waiting");
    assert!(store_data.provisional_organisations.is_empty());
}

// A chain past epoch 1 is behind for a founding store, which keeps its
// founding and changes no sealed byte.
// verifies: LLR-8528ud, LLR-qvp7mq
#[test]
fn a_founding_whose_chain_is_past_epoch_one_is_behind_and_keeps_the_founding() {
    let asked = Cell::new(false);
    let (mut svc, key, _secret, state) = stores::founding();
    let before = support::encoded_store(&svc, &key);
    let past = stores::chain(2, state.root_hash, state.org_pub_key);
    let (verified, change) = stores::verify_as(&mut svc, key, Some(past), &asked).unwrap();
    assert_eq!(verified, Verified::Behind { chain_epoch: Epoch::new(2) });
    assert!(change.is_none());
    assert_eq!(support::encoded_store(&svc, &key), before);
    assert!(svc.store_data(&key).unwrap().founding.is_some());
}

// A founding whose members do not rebuild the root it and the chain hold
// is not adopted.
// verifies: LLR-8528ud
#[test]
fn a_founding_whose_members_do_not_rebuild_the_chain_root_is_not_adopted() {
    let asked = Cell::new(false);
    let (svc, key, secret, state) = stores::founding();
    let mut tampered = svc.store_data(&key).unwrap().clone();
    tampered.founding.as_mut().unwrap().members[0].handle = org_node::Handle::parse("mallory").unwrap();
    let mut svc = stores::service_under(&secret, tampered);
    let (verified, change) = stores::verify_as(&mut svc, key, Some(state), &asked).unwrap();
    assert!(!matches!(verified, Verified::Adopted { .. }), "not adopted: {verified:?}");
    assert!(change.is_none());
    assert!(svc.store_data(&key).unwrap().record.is_none());
}

// Abnormal: a chain state at epoch 1 holding the founding's own root under
// another Organisation public key is no match: the founding store is
// behind, keeps its founding and changes no sealed byte.
// verifies: LLR-8528ud
#[test]
fn a_founding_whose_root_the_chain_holds_under_another_key_is_not_adopted() {
    let asked = Cell::new(false);
    let (mut svc, key, _secret, state) = stores::founding();
    let before = support::encoded_store(&svc, &key);
    let other_key = stores::chain(1, state.root_hash, stores::public_key([0x4e; 32]));
    let (verified, change) = stores::verify_as(&mut svc, key, Some(other_key), &asked).unwrap();
    assert!(!matches!(verified, Verified::Adopted { .. }), "not adopted: {verified:?}");
    assert!(change.is_none());
    assert_eq!(support::encoded_store(&svc, &key), before);
    assert!(svc.store_data(&key).unwrap().founding.is_some(), "the founding is kept");
}

// A founding is adopted once the chain carries its root: a chain state at
// epoch 1 with another root is no match, so the founding store reports
// Behind and writes nothing; the founding's own
// state is adopted, synchronously and with no send, as the record with the
// chain's root and key, epoch and mark 1, the founding's private key and
// its proxy; the Persona is Active with the founder's MemberId; the
// founding is consumed; and the sealed store holds all of it.
// verifies: REQ-tqap3r, LLR-gcm7bq, LLR-exaa9u, LLR-qw8tyu, LLR-x5asbx, LLR-g2jeqd
#[test]
fn a_founding_is_adopted_once_the_chain_carries_its_root() {
    use org_node::store::PersonaStatus;
    use org_node::{OrgState, SequenceNumber};

    let mut founder = sweep::found();
    let chain = sweep::founding_state(&founder);
    let elsewhere = OrgState { root_hash: RootHash::new([0x01; 32]), ..chain };
    let (verified, change) =
        founder.node.svc.verify(&mut OsRng, &founder.device, Some(elsewhere), support::story_time(), org_node::AheadLimit(1200), support::never_asked).unwrap();
    assert!(matches!(verified, Verified::Behind { .. }), "{verified:?}");
    assert!(change.is_none(), "Behind writes nothing");
    assert!(support::held_store(&founder.svc, &founder.device).founding.is_some(), "the founding is kept");

    let founder_id = support::held_store(&founder.svc, &founder.device).founding.as_ref().unwrap().members[0].id;
    let verified = sweep::verify(&mut founder, chain);
    assert!(
        matches!(&verified, Verified::Adopted { epoch, discarded, own: true } if *epoch == chain.epoch && discarded.is_empty()),
        "{verified:?}"
    );
    let record = sweep::record_of(&founder);
    assert_eq!(
        (record.org_id, record.root_hash, record.org_pub_key, record.epoch, record.last_seq),
        (support::the_org(), chain.root_hash, chain.org_pub_key, chain.epoch, SequenceNumber::new(1))
    );
    assert_eq!(record.org_private_key_for_test().x25519_keypair().org_public_key().unwrap(), chain.org_pub_key);
    assert_eq!(record.proxy_account, Some(support::the_proxy()));
    let persona = &support::held_store(&founder.svc, &founder.device).persona;
    assert_eq!(
        (persona.status, persona.org_id, persona.member_id),
        (PersonaStatus::Active, Some(support::the_org()), Some(founder_id))
    );
    assert!(support::held_store(&founder.svc, &founder.device).founding.is_none(), "the founding is consumed");
    let reopened = founder.reopened_service();
    let on_disk = support::held_store(&reopened, &founder.device);
    assert_eq!(
        on_disk.record.as_ref().map(|record| (record.root_hash, record.proxy_account)),
        Some((chain.root_hash, Some(support::the_proxy())))
    );
    assert!(on_disk.founding.is_none());
}

// An own provisional organisation the chain carries is adopted as a
// received one would be: the record, epoch, mark
// and members are written together with the provisional organisation's key
// pair, keeping no earlier key, and the sealed store returned with the
// outcome holds them; the adoption sends nothing.
// verifies: REQ-tqap3r, REQ-jy6ybw, LLR-cja9zv, LLR-g2jeqd
#[test]
fn an_own_provisional_organisation_the_chain_carries_is_adopted() {
    use org_node::SequenceNumber;

    let (mut founder, founding) = sweep::founded();
    let key_before = sweep::record_of(&founder).org_private_key_for_test().clone();
    let pending = sweep::admission(&mut founder, "Bob", "bob");
    let record = sweep::record_of(&founder);
    assert_eq!(
        (record.root_hash, record.epoch, record.last_seq, record.trie_members.len()),
        (pending.chain.root_hash, pending.chain.epoch, SequenceNumber::new(pending.chain.epoch.get()), 2)
    );
    assert_ne!(pending.chain.epoch, founding.epoch);
    assert_eq!(record.org_pub_key, pending.chain.org_pub_key, "the provisional organisation's public key");
    let key = record.org_private_key_for_test().clone();
    assert_eq!(key.x25519_keypair().org_public_key().unwrap(), pending.chain.org_pub_key, "and its private half");
    assert_ne!(key, key_before, "no earlier key is kept");
    assert!(matches!(sweep::verify(&mut founder, pending.chain), Verified::InStep), "adopted already: the same state is in step");
    let reopened = founder.reopened_service();
    let on_disk = support::held_store(&reopened, &founder.device).record.as_ref().unwrap();
    assert_eq!(
        (on_disk.root_hash, on_disk.epoch, on_disk.last_seq),
        (pending.chain.root_hash, pending.chain.epoch, SequenceNumber::new(pending.chain.epoch.get()))
    );
    assert_eq!(on_disk.org_private_key_for_test().clone(), key);
}

// The record takes a recorded revocation only through adoption: recording
// it and proposing the next epoch leave the record listing B; the adoption
// removes B; the messages built afterwards and never delivered undo
// nothing; the proxy account is neither read nor changed by any of it.
// verifies: REQ-tqap3r, LLR-34c5y3, LLR-x5asbx, LLR-g2jeqd
#[test]
fn the_record_takes_a_recorded_revocation_only_through_adoption() {
    let (mut founder, _) = sweep::founded();
    let mut bob = sweep::admission(&mut founder, "Bob", "bob");
    sweep::receive_and_adopt(&mut bob.member, founder.device, &bob.body, bob.chain);
    let b_id = sweep::member_id_of(&bob.member);
    let proxy_before = founder.svc.proxy_account(support::the_org()).unwrap();
    let lists_b = |side: &sweep::Side| sweep::record_of(side).trie_members.iter().any(|member| member.id == b_id);

    let change = founder.node.svc.record_revocation(&mut OsRng, &founder.device, b_id).unwrap();
    founder.keep(change);
    assert!(lists_b(&founder), "recording changes no record");
    let next = sweep::propose_next(&mut founder);
    assert!(lists_b(&founder), "proposing changes no record");
    sweep::adopt(&mut founder, next);
    assert!(!lists_b(&founder), "the adoption removes B");
    let messages = founder.svc.messages_for_current_epoch(&founder.device).unwrap();
    assert!(messages.iter().any(|message| message.recipient == bob.member.device), "B's revocation is built");
    drop(messages); // never delivered
    assert!(!lists_b(&founder), "an undelivered message undoes nothing");
    assert_eq!(founder.svc.proxy_account(support::the_org()).unwrap(), proxy_before);
}

// ---- discarding what can never be adopted (LLR-4tsgfh) ---------------------------

/// Fixtures: stores holding provisional organisations at chosen epochs,
/// with a record and with none, and a verification at a chosen time and
/// ahead limit.
mod ahead {
    use std::cell::Cell;

    use org_node::chain::OrgState;
    use org_node::error::OrgNodeError;
    use org_node::service::{OrgService, StoreChange};
    use org_node::store::{Origin, ProvisionalChange, ProvisionalOrganisation, ProvisionalOrganisationParts};
    use org_node::test_fixtures::{device_key, store_for_test};
    use org_node::{AheadLimit, DevicePublicKey, Epoch, MemberId, OrgPrivateKey, RootHash, VerificationTime, Verified};
    use rand::rngs::OsRng;

    pub use crate::stores::{chain, public_key, EVE};

    /// The time of the first verification in these tests.
    pub const AT: u64 = 1_760_000_000;
    /// The ahead limit these tests pass unless a test varies it.
    pub const LIMIT: u64 = 1_200;
    /// The seed of the root and key of every chain state `unmatched_chain`
    /// gives, which no provisional organisation here holds.
    pub const CHAIN_SEED: u8 = 0x5e;

    /// A provisional organisation for `epoch` whose root is `seed` bytes and
    /// whose key pair is drawn from `seed`, of `origin`, carrying `padding`
    /// bytes of Change set.
    pub fn held_at(epoch: u64, seed: u8, origin: Origin, padding: usize) -> ProvisionalOrganisation {
        ProvisionalOrganisation::from(ProvisionalOrganisationParts {
            epoch: Epoch::new(epoch),
            base_root: RootHash::new([0x0b; 32]),
            members: Vec::new(),
            root: RootHash::new([seed; 32]),
            org_pub_key: public_key([seed; 32]),
            org_private_key: OrgPrivateKey::from([seed; 32]),
            change_set: Some(vec![0x7a; padding]),
            base_snapshot: None,
            origin,
            applied: Vec::new(),
            first_ahead_at: None,
        })
    }

    /// Organisation information Eve's Device sent, for `epoch`, of seed `seed`.
    pub fn received(epoch: u64, seed: u8) -> ProvisionalOrganisation {
        held_at(epoch, seed, Origin::Received { from: device_key(EVE) }, 0)
    }

    /// An organisation answer Eve's Device sent, for `epoch`, of seed `seed`.
    pub fn answer(epoch: u64, seed: u8) -> ProvisionalOrganisation {
        held_at(epoch, seed, Origin::Answer { from: device_key(EVE) }, 0)
    }

    /// This node's own proposal for `epoch`, of seed `seed`.
    pub fn own(epoch: u64, seed: u8) -> ProvisionalOrganisation {
        held_at(epoch, seed, Origin::Own, 0)
    }

    /// The chain state at `epoch` holding no provisional organisation's root.
    pub fn unmatched_chain(epoch: u64) -> OrgState {
        chain(epoch, RootHash::new([CHAIN_SEED; 32]), public_key([CHAIN_SEED; 32]))
    }

    /// The chain state at `epoch` holding the root and key of seed `seed`.
    pub fn chain_of(epoch: u64, seed: u8) -> OrgState {
        chain(epoch, RootHash::new([seed; 32]), public_key([seed; 32]))
    }

    /// The one provisional change both stores hold.
    pub fn the_change() -> ProvisionalChange {
        ProvisionalChange::Revoke { member_id: MemberId::new([0x02; 32]) }
    }

    /// Ada's store holding a record at epoch 3, `organisations` and one
    /// provisional change.
    pub fn with_record(organisations: Vec<ProvisionalOrganisation>) -> (OrgService, DevicePublicKey) {
        let mut store_data = crate::stores::ada_store_in_trie_order(3, &[crate::stores::ada(), crate::stores::bob()]);
        store_data.provisional_organisations = organisations;
        store_data.provisional_changes = vec![the_change()];
        (crate::stores::service(crate::stores::ADA, store_data), device_key(crate::stores::ADA))
    }

    /// The chain state in step with `with_record`'s record.
    pub fn in_step_with_record(svc: &OrgService, key: DevicePublicKey) -> OrgState {
        let record = crate::support::held_store(svc, &key).record.as_ref().unwrap();
        chain(record.epoch.get(), record.root_hash, record.org_pub_key)
    }

    /// Carol's provisional Member store, which holds no record, holding
    /// `organisations` and one provisional change.
    pub fn without_record(organisations: Vec<ProvisionalOrganisation>) -> (OrgService, DevicePublicKey) {
        let mut svc = OrgService::new(Vec::new()).unwrap();
        let (key, secret, sealed) =
            svc.propose_member(&mut OsRng, &crate::stores::invite_to_carol(), "Carol", "Coder", &["carol"]).unwrap();
        crate::support::unread(sealed);
        let mut store_data = svc.store_data(&key).unwrap().clone();
        store_data.provisional_organisations = organisations;
        store_data.provisional_changes = vec![the_change()];
        svc.restore_store(store_for_test(&secret, store_data));
        (svc, key)
    }

    /// Both stores, each holding `organisations()`, by name.
    pub fn both(organisations: impl Fn() -> Vec<ProvisionalOrganisation>) -> [(&'static str, OrgService, DevicePublicKey); 2] {
        let (recorded, recorded_key) = with_record(organisations());
        let (unrecorded, unrecorded_key) = without_record(organisations());
        [("with a record", recorded, recorded_key), ("with no record", unrecorded, unrecorded_key)]
    }

    /// `verify` at `at` with the ahead limit `limit`; it must ask no secret.
    pub fn verify(
        svc: &mut OrgService,
        key: DevicePublicKey,
        state: Option<OrgState>,
        at: u64,
        limit: u64,
    ) -> Result<(Verified, Option<StoreChange>), OrgNodeError> {
        let asked = Cell::new(false);
        let outcome = crate::stores::verify_at(svc, key, state, (VerificationTime(at), AheadLimit(limit)), &asked);
        assert!(!asked.get(), "no secret asked");
        outcome
    }

    /// Each held provisional organisation's epoch, root seed and stamp, in
    /// held order.
    pub fn held_now(svc: &OrgService, key: DevicePublicKey) -> Vec<(u64, u8, Option<u64>)> {
        crate::support::held_store(svc, &key)
            .provisional_organisations
            .iter()
            .map(|held| (held.epoch.get(), held.root.as_bytes()[0], held.first_ahead_at.map(|at| at.0)))
            .collect()
    }
}

// Every verification against a chain at epoch C, whatever its outcome,
// discards each received or answer provisional organisation for an epoch
// below C, above C + 1, or at C not matching the chain, and keeps those at
// C + 1, stamped with the verification's time, in one sealed change; the
// provisional change is untouched. With a record: in step, behind and
// inconsistent; with none: behind.
// verifies: LLR-4tsgfh, RC-d4fc4s
#[test]
fn a_verification_discards_received_organisations_below_the_chain_beyond_the_next_epoch_or_unmatched_at_it() {
    let around = |c: u64| {
        vec![
            ahead::received(c - 1, 0x21),
            ahead::answer(c, 0x22),
            ahead::received(c, 0x23),
            ahead::received(c + 1, 0x24),
            ahead::answer(c + 1, 0x25),
            ahead::received(c + 2, 0x26),
            ahead::answer(c + 9, 0x27),
        ]
    };
    let (in_step, in_step_key) = ahead::with_record(around(3));
    let in_step_chain = ahead::in_step_with_record(&in_step, in_step_key);
    let other_root = ahead::chain(3, RootHash::new([ahead::CHAIN_SEED; 32]), in_step_chain.org_pub_key);
    let (behind, behind_key) = ahead::with_record(around(4));
    let (inconsistent, inconsistent_key) = ahead::with_record(around(3));
    let (unrecorded, unrecorded_key) = ahead::without_record(around(4));
    let cases = [
        ("in step", 3, in_step, in_step_key, in_step_chain),
        ("behind", 4, behind, behind_key, ahead::unmatched_chain(4)),
        ("inconsistent", 3, inconsistent, inconsistent_key, other_root),
        ("behind, no record", 4, unrecorded, unrecorded_key, ahead::unmatched_chain(4)),
    ];
    for (name, c, mut svc, key, state) in cases {
        let before = support::encoded_store(&svc, &key);
        let (verified, change) = ahead::verify(&mut svc, key, Some(state), ahead::AT, ahead::LIMIT).unwrap();
        assert!(!matches!(verified, Verified::Adopted { .. } | Verified::Removed { .. }), "{name}: {verified:?}");
        assert_eq!(
            ahead::held_now(&svc, key),
            vec![(c + 1, 0x24, Some(ahead::AT)), (c + 1, 0x25, Some(ahead::AT))],
            "{name}: only the next epoch's are kept, stamped"
        );
        assert!(change.is_some(), "{name}: the discard is sealed");
        assert_ne!(support::encoded_store(&svc, &key), before, "{name}");
        assert_eq!(svc.provisional_changes(&key).unwrap(), &[ahead::the_change()], "{name}: the change is untouched");
    }
}

// An adoption discards as before (LLR-x8drb8), and the same verification
// applies the rules to what is left: the next epoch's is kept and stamped,
// one beyond it discarded.
// verifies: LLR-4tsgfh, LLR-x8drb8
#[test]
fn an_adopting_verification_applies_the_rules_to_what_the_adoption_leaves() {
    let (mut svc, key) = ahead::with_record(vec![
        ahead::received(4, 0x31),
        ahead::received(4, 0x32),
        ahead::received(5, 0x33),
        ahead::received(6, 0x34),
    ]);
    let mut matched = svc.store_data(&key).unwrap().clone();
    // The match must list Ada's Device, or the adoption is her removal.
    matched.provisional_organisations[0].members = matched.record.as_ref().unwrap().trie_members.clone();
    svc.restore_store(org_node::test_fixtures::store_for_test(&device_secret([stores::ADA; 32]), matched));
    let (verified, change) = ahead::verify(&mut svc, key, Some(ahead::chain_of(4, 0x31)), ahead::AT, ahead::LIMIT).unwrap();
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    assert!(change.is_some());
    assert_eq!(ahead::held_now(&svc, key), vec![(5, 0x33, Some(ahead::AT))]);
}

// One held at C + 1 is kept by every verification up to `ahead_limit`
// seconds after its stamp, the stamp set by the first and never moved, and
// discarded by the first verification later than that while still at
// C + 1; with a record and with none.
// verifies: LLR-4tsgfh, RC-d4fc4s
#[test]
fn the_next_epochs_information_is_kept_up_to_the_ahead_limit_after_its_stamp_and_discarded_after() {
    for (name, mut svc, key) in ahead::both(|| vec![ahead::received(5, 0x41), ahead::answer(5, 0x42)]) {
        let state = Some(ahead::unmatched_chain(4));
        let stamped = vec![(5, 0x41, Some(ahead::AT)), (5, 0x42, Some(ahead::AT))];
        let (_, change) = ahead::verify(&mut svc, key, state, ahead::AT, ahead::LIMIT).unwrap();
        assert!(change.is_some(), "{name}: the stamp is sealed");
        assert_eq!(ahead::held_now(&svc, key), stamped, "{name}: stamped by the first verification");
        let before = support::encoded_store(&svc, &key);
        let (_, change) = ahead::verify(&mut svc, key, state, ahead::AT + 600, ahead::LIMIT).unwrap();
        assert!(change.is_none(), "{name}: nothing to seal");
        assert_eq!(support::encoded_store(&svc, &key), before, "{name}: the stamp does not move");
        ahead::verify(&mut svc, key, state, ahead::AT + ahead::LIMIT, ahead::LIMIT).unwrap();
        assert_eq!(ahead::held_now(&svc, key), stamped, "{name}: kept at exactly the limit");
        let (_, change) = ahead::verify(&mut svc, key, state, ahead::AT + ahead::LIMIT + 1, ahead::LIMIT).unwrap();
        assert!(change.is_some(), "{name}: the discard is sealed");
        assert!(ahead::held_now(&svc, key).is_empty(), "{name}: discarded one second after the limit");
    }
}

// The limit is the caller's: one stamp, verified at the same later time
// with two limits, is kept under the longer and discarded under the
// shorter.
// verifies: LLR-4tsgfh
#[test]
fn two_ahead_limits_give_two_outcomes_for_one_stamp() {
    for (name, limit, kept) in [("600 s", 600, false), ("1200 s", 1_200, true)] {
        for (store, mut svc, key) in ahead::both(|| vec![ahead::received(5, 0x41)]) {
            let state = Some(ahead::unmatched_chain(4));
            ahead::verify(&mut svc, key, state, ahead::AT, limit).unwrap();
            ahead::verify(&mut svc, key, state, ahead::AT + 900, limit).unwrap();
            assert_eq!(ahead::held_now(&svc, key).len() == 1, kept, "{store}, limit {name}");
        }
    }
}

// Once the chain reaches the epoch of a stamped one, it is judged by the
// match: adopted when it matches, discarded when it does not, whatever
// the time.
// verifies: LLR-4tsgfh, LLR-8528ud
#[test]
fn a_stamped_one_the_chain_reaches_is_judged_by_the_match() {
    let (mut svc, key) = ahead::with_record(vec![ahead::received(5, 0x51), ahead::received(5, 0x52)]);
    ahead::verify(&mut svc, key, Some(ahead::unmatched_chain(4)), ahead::AT, ahead::LIMIT).unwrap();
    assert_eq!(ahead::held_now(&svc, key).len(), 2);
    let (verified, _) = ahead::verify(&mut svc, key, Some(ahead::unmatched_chain(5)), ahead::AT + 60, ahead::LIMIT).unwrap();
    assert_eq!(verified, Verified::Behind { chain_epoch: Epoch::new(5) });
    assert!(ahead::held_now(&svc, key).is_empty(), "neither matches the chain at its epoch");
}

// This node's own proposals and its provisional changes are touched by no
// rule: below the chain, beyond the next epoch, unmatched at it, and at
// the next epoch long after any limit; none is stamped.
// verifies: LLR-4tsgfh
#[test]
fn own_proposals_and_provisional_changes_survive_every_rule() {
    let owned = || vec![ahead::own(3, 0x61), ahead::own(4, 0x62), ahead::own(5, 0x63), ahead::own(6, 0x64)];
    for (name, mut svc, key) in ahead::both(owned) {
        let before = support::encoded_store(&svc, &key);
        for at in [ahead::AT, ahead::AT + 1_000_000] {
            let (verified, change) = ahead::verify(&mut svc, key, Some(ahead::unmatched_chain(4)), at, ahead::LIMIT).unwrap();
            assert_eq!(verified, Verified::Behind { chain_epoch: Epoch::new(4) }, "{name}");
            assert!(change.is_none(), "{name}: nothing to seal");
        }
        assert_eq!(support::encoded_store(&svc, &key), before, "{name}: every proposal and the change kept, unstamped");
        assert_eq!(ahead::held_now(&svc, key).len(), 4, "{name}");
    }
}

// With no chain state nothing is discarded or stamped: the refusal
// changes no byte; with a record and with none.
// verifies: LLR-4tsgfh, LLR-qvp7mq
#[test]
fn no_chain_state_discards_nothing() {
    let around = || vec![ahead::received(1, 0x71), ahead::received(4, 0x72), ahead::answer(5, 0x73), ahead::received(9, 0x74)];
    for (name, mut svc, key) in ahead::both(around) {
        let before = support::encoded_store(&svc, &key);
        let refused = ahead::verify(&mut svc, key, None, ahead::AT + 1_000_000, ahead::LIMIT);
        assert_eq!(refused.unwrap_err(), OrgNodeError::OrgNotOnChain, "{name}");
        assert_eq!(support::encoded_store(&svc, &key), before, "{name}");
    }
}

// MR3-N1's flood: a store with no record, filled to the bound by
// information from any Device at several epochs, refuses the genuine
// information with `ProvisionalLimit`; a verification discards all of it
// but what is at the next epoch, which still fills the bound; the first
// verification after the limit discards that too; the genuine information
// is then kept, and the next verification adopts it. The Device waits
// throughout: nothing else of the store changes.
// verifies: LLR-4tsgfh, RC-d4fc4s, REQ-g2537c
#[test]
fn a_flood_of_unadoptable_information_delays_an_admission_only_until_the_ahead_limit() {
    use org_node::store::MAX_PROVISIONAL_BYTES;
    let eve = org_node::store::Origin::Received { from: device_key(stores::EVE) };
    // Small bodies below the chain, unmatched at it and beyond the next
    // epoch; two at the next epoch padded until the store is as full as
    // the bound allows.
    let flood = |filler: usize| {
        vec![
            ahead::held_at(1, 0x81, eve, 0),
            ahead::held_at(2, 0x82, eve, 0),
            ahead::held_at(3, 0x83, eve, 500_000),
            ahead::held_at(3, 0x84, eve, filler),
            ahead::held_at(7, 0x85, eve, 0),
        ]
    };
    let bound_len_of = |organisations: Vec<org_node::ProvisionalOrganisation>| {
        let (probe, probe_key) = ahead::without_record(organisations);
        stores::bound_len(probe.store_data(&probe_key).unwrap())
    };
    let mut filler = MAX_PROVISIONAL_BYTES - bound_len_of(flood(0));
    while bound_len_of(flood(filler)) > MAX_PROVISIONAL_BYTES {
        filler -= 1;
    }
    let (mut svc, key) = ahead::without_record(flood(filler));
    let listed = stores::provisional_member_listed(&svc, key);
    let (body, root, org_pub_key) = stores::information(1, &[stores::ada()], &listed, [0x4c; 32]);
    let genuine = stores::chain(2, root, org_pub_key);
    let (mut sized, sized_key) = ahead::without_record(Vec::new());
    let empty_len = stores::bound_len(sized.store_data(&sized_key).unwrap());
    stores::receive_into(&mut sized, sized_key, stores::ADA, &body);
    let genuine_len = stores::bound_len(sized.store_data(&sized_key).unwrap()) - empty_len;
    let next_epochs_len = bound_len_of(flood(filler).drain(2..4).collect());
    assert!(next_epochs_len + genuine_len > MAX_PROVISIONAL_BYTES, "fixture: the next epoch's bodies alone leave no room");
    let refuse = |svc: &mut OrgService| svc.check_received(&mut OsRng, &key, device_key(stores::ADA), &body).unwrap_err();
    let limit = OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES };
    assert_eq!(refuse(&mut svc), limit, "the flood blocks the genuine information");

    let (verified, _) = ahead::verify(&mut svc, key, Some(genuine), ahead::AT, ahead::LIMIT).unwrap();
    assert_eq!(verified, Verified::Behind { chain_epoch: Epoch::new(2) });
    assert_eq!(ahead::held_now(&svc, key), vec![(3, 0x83, Some(ahead::AT)), (3, 0x84, Some(ahead::AT))]);
    assert_eq!(refuse(&mut svc), limit, "the next epoch's bodies still fill the bound");
    ahead::verify(&mut svc, key, Some(genuine), ahead::AT + ahead::LIMIT, ahead::LIMIT).unwrap();
    assert_eq!(refuse(&mut svc), limit, "up to the limit");

    ahead::verify(&mut svc, key, Some(genuine), ahead::AT + ahead::LIMIT + 1, ahead::LIMIT).unwrap();
    assert!(ahead::held_now(&svc, key).is_empty(), "the flood is cleared");
    assert!(svc.store_data(&key).unwrap().member.is_some(), "still waiting for admission");
    assert_eq!(stores::receive_into(&mut svc, key, stores::ADA, &body), Epoch::new(2), "the genuine information is kept");
    let (verified, _) = ahead::verify(&mut svc, key, Some(genuine), ahead::AT + ahead::LIMIT + 60, ahead::LIMIT).unwrap();
    assert_eq!(verified, Verified::Adopted { epoch: Epoch::new(2), discarded: Vec::new(), own: false });
}

// A chain read whose epoch C is below the record's own epoch R, which the
// verification reports Inconsistent, runs none of the rules: a genuine
// R + 1 received organisation (beyond C + 1), and those below C and at C,
// are all kept unstamped, and nothing is written. At C = R with another
// root, also Inconsistent, the rules still run (owner ruling 2026-10-30,
// merge review MR4 finding-2).
// verifies: LLR-4tsgfh
#[test]
fn a_chain_read_behind_the_record_runs_no_rule_and_one_at_the_records_epoch_does() {
    let held = || vec![ahead::received(1, 0x91), ahead::received(2, 0x92), ahead::received(4, 0x93), ahead::answer(4, 0x94)];
    let (mut svc, key) = ahead::with_record(held());
    let before = support::encoded_store(&svc, &key);
    let held_before = ahead::held_now(&svc, key);
    let (verified, change) = ahead::verify(&mut svc, key, Some(ahead::unmatched_chain(2)), ahead::AT, ahead::LIMIT).unwrap();
    let behind_record = Inconsistency::ChainBehindRecord { chain_epoch: Epoch::new(2), record_epoch: Epoch::new(3) };
    assert_eq!(verified, Verified::Inconsistent { reason: behind_record });
    assert!(change.is_none(), "nothing to seal");
    assert_eq!(ahead::held_now(&svc, key), held_before, "nothing discarded, nothing stamped");
    assert_eq!(support::encoded_store(&svc, &key), before, "no byte of the store changes");

    let (mut svc, key) = ahead::with_record(held());
    let in_step = ahead::in_step_with_record(&svc, key);
    let other_root = ahead::chain(3, RootHash::new([ahead::CHAIN_SEED; 32]), in_step.org_pub_key);
    let (verified, change) = ahead::verify(&mut svc, key, Some(other_root), ahead::AT, ahead::LIMIT).unwrap();
    assert_eq!(verified, Verified::Inconsistent { reason: Inconsistency::OtherRoot { epoch: Epoch::new(3) } });
    assert!(change.is_some(), "the rules' discard is sealed");
    assert_eq!(ahead::held_now(&svc, key), vec![(4, 0x93, Some(ahead::AT)), (4, 0x94, Some(ahead::AT))]);
}

// A verification whose time is earlier than a stamp (the wall clock
// stepped back) keeps the stamped one, does not move the stamp and writes
// nothing; the stamp still ages from its own time; with a record and with
// none (MR4 finding-4).
// verifies: LLR-4tsgfh
#[test]
fn a_verification_earlier_than_the_stamp_keeps_the_next_epochs_information() {
    for (name, mut svc, key) in ahead::both(|| vec![ahead::received(5, 0xa1)]) {
        let state = Some(ahead::unmatched_chain(4));
        ahead::verify(&mut svc, key, state, ahead::AT, ahead::LIMIT).unwrap();
        let before = support::encoded_store(&svc, &key);
        let (_, change) = ahead::verify(&mut svc, key, state, ahead::AT - 10_000, ahead::LIMIT).unwrap();
        assert!(change.is_none(), "{name}: nothing to seal");
        assert_eq!(support::encoded_store(&svc, &key), before, "{name}: kept, the stamp unmoved");
        ahead::verify(&mut svc, key, state, ahead::AT + ahead::LIMIT + 1, ahead::LIMIT).unwrap();
        assert!(ahead::held_now(&svc, key).is_empty(), "{name}: still aged from the stamp");
    }
}

// A chain at epoch u64::MAX has no C + 1: every received or answer one not
// matching it at C is discarded, none is kept ahead, and the verification
// does not overflow; this node's own proposal is kept; with a record and
// with none (MR4 finding-4).
// verifies: LLR-4tsgfh
#[test]
fn a_chain_at_the_last_epoch_keeps_nothing_ahead_and_does_not_overflow() {
    let last = u64::MAX;
    let held = || vec![ahead::received(last - 1, 0xb1), ahead::received(last, 0xb2), ahead::answer(last, 0xb3), ahead::own(last, 0xb4)];
    for (name, mut svc, key) in ahead::both(held) {
        let (verified, change) = ahead::verify(&mut svc, key, Some(ahead::unmatched_chain(last)), ahead::AT, ahead::LIMIT).unwrap();
        assert_eq!(verified, Verified::Behind { chain_epoch: Epoch::new(last) }, "{name}");
        assert!(change.is_some(), "{name}: the discard is sealed");
        assert_eq!(ahead::held_now(&svc, key), vec![(last, 0xb4, None)], "{name}: only the own proposal is kept");
    }
}

// With an ahead limit of 0 one at C + 1 is kept, stamped, by the
// verification that stamps it and by any at that same second, and
// discarded by the first verification a second or more after the stamp;
// with a record and with none (MR4 finding-4).
// verifies: LLR-4tsgfh
#[test]
fn an_ahead_limit_of_zero_keeps_the_next_epochs_information_only_within_its_stamps_second() {
    for (name, mut svc, key) in ahead::both(|| vec![ahead::received(5, 0xc1)]) {
        let state = Some(ahead::unmatched_chain(4));
        let (_, change) = ahead::verify(&mut svc, key, state, ahead::AT, 0).unwrap();
        assert!(change.is_some(), "{name}: the stamp is sealed");
        assert_eq!(ahead::held_now(&svc, key), vec![(5, 0xc1, Some(ahead::AT))], "{name}: kept at its stamping verification");
        ahead::verify(&mut svc, key, state, ahead::AT, 0).unwrap();
        assert_eq!(ahead::held_now(&svc, key).len(), 1, "{name}: kept within the stamp's second");
        let (_, change) = ahead::verify(&mut svc, key, state, ahead::AT + 1, 0).unwrap();
        assert!(change.is_some(), "{name}: the discard is sealed");
        assert!(ahead::held_now(&svc, key).is_empty(), "{name}: discarded a second after the stamp");
    }
}
