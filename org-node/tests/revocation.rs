#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Revocation notices and removal acknowledgements: the notices a removal
//! produces, the decision on a received notice, and the check of a received
//! acknowledgement. A store holds one current record (LLR-d925uu), and
//! `accept` decides but deletes nothing (the removal is `verify`'s).

mod support;

use org_members::hasher::Blake3Hasher;
use org_members::{DevicePublicKey, MemberId, MemberLeaf};
use org_node::ids::OrgId;
use org_node::keys::X25519Keypair;
use org_node::revocation::{
    accept, check_acknowledgement, check_notice, notices_for, Acknowledgement, RevocationNotice, Signature64, ACK_DOMAIN,
};
use org_node::store::{OrgRecord, OrgRecordParts, PersonaRecord, PersonaRecordParts, PersonaStatus, StoreData};
use org_node::test_fixtures::{admin_device, device_key, genesis_trie, member, Trie, BOB_DEVICE_SEED};
use org_node::{DeviceSecret, Epoch, MemberSeed, OrgNodeError, OrgPrivateKey, OrgState, PersonaId, RootHash};
use support::fixtures::{persona_parts, record_parts};
use support::{handle, parsed_name, parsed_surname};

fn org() -> OrgId {
    OrgId::new([7; 20])
}

fn bob_id() -> MemberId {
    MemberId::new([2; 32])
}

fn admin_member_keypair() -> X25519Keypair {
    MemberSeed::from([1; 32]).x25519_keypair()
}

/// The record of the calculated `trie` at `epoch`.
fn record_at(trie: &Trie, epoch: u64) -> OrgRecord {
    OrgRecord::from(OrgRecordParts {
        org_id: org(),
        root_hash: trie.root_hash().unwrap(),
        org_pub_key: org_node::test_fixtures::org_public_key(),
        org_private_key: OrgPrivateKey::from([9; 32]),
        ..record_parts(epoch, &support::snapshots_of(trie))
    })
}

/// A calculated trie of the admin and bob, bob holding `bob_devices`.
fn trie_with_bob(bob_devices: Vec<DevicePublicKey>) -> Trie {
    let genesis = genesis_trie(&admin_member_keypair(), &admin_device());
    let bob_fixture = org_node::test_fixtures::NodeFixture {
        member: MemberSeed::from([2; 32]).x25519_keypair(),
        device: org_node::test_fixtures::bob_device(),
        id: bob_id(),
    };
    let template = member(&bob_fixture, "bob");
    let bob = MemberLeaf::new(
        bob_id(),
        template.handle().clone(),
        *template.p2p_key(),
        template.name().clone(),
        template.surname().clone(),
        bob_devices,
    )
    .unwrap();
    genesis.add_member(bob).unwrap().recalculate().unwrap().0
}

/// The (previous, committed) records of deleting bob, the committed
/// calculated trie, and bob's Devices in slot order.
fn records_deleting_bob(
    bob_devices: Vec<DevicePublicKey>,
) -> (OrgRecord, OrgRecord, Trie, Vec<DevicePublicKey>) {
    let previous_trie = trie_with_bob(bob_devices);
    let slot_order = previous_trie.get(&bob_id()).unwrap().p2p_devices().to_vec();
    let committed_trie = previous_trie.delete_member(&bob_id()).unwrap().recalculate().unwrap().0;
    (record_at(&previous_trie, 1), record_at(&committed_trie, 2), committed_trie, slot_order)
}

fn record_pair_removing_one_device() -> (OrgRecord, OrgRecord, Trie, (MemberId, DevicePublicKey)) {
    let bob_device = org_node::test_fixtures::bob_device().device_key().unwrap();
    let (previous, committed, trie, _) = records_deleting_bob(vec![bob_device]);
    (previous, committed, trie, (bob_id(), bob_device))
}

fn record_pair_removing_a_member_with_two_devices() -> (OrgRecord, OrgRecord, Trie, Vec<DevicePublicKey>) {
    let (previous, committed, trie, slot_order) = records_deleting_bob(vec![device_key(5), device_key(6)]);
    assert_eq!(slot_order.len(), 2, "fixture: bob holds two Devices");
    (previous, committed, trie, slot_order)
}

fn sample_acknowledgement() -> Acknowledgement {
    Acknowledgement {
        org_id: org(),
        member_id: bob_id(),
        device: device_key(5),
        epoch: Epoch::from(3),
        root: RootHash::new([0x11; 32]),
        signature: Signature64([0x22; 64]),
    }
}

/// verifies: LLR-kr5t6f
///
/// Normal: one removed Device → one notice holding the committed record's
/// Organisation, B's pair, and a proof that verifies against the committed
/// root.
#[test]
fn notices_for_names_each_removed_device_with_a_proof_from_the_committed_record() {
    let (previous, committed, committed_trie, (member_id, device)) = record_pair_removing_one_device();
    let notices = notices_for(&previous, &committed, &committed_trie).unwrap();
    assert_eq!(notices.len(), 1);
    let notice = &notices[0];
    assert_eq!((notice.org_id, notice.member_id, notice.device), (committed.org_id, member_id, device));
    notice.proof.verify::<Blake3Hasher>(&committed.root_hash, &notice.member_id, &notice.device).unwrap();
}

/// verifies: LLR-kr5t6f
///
/// Normal: a removed Member's Devices come in slot order; nothing removed →
/// no notice.
#[test]
fn notices_for_orders_by_member_then_slot_and_is_empty_when_nothing_was_removed() {
    let (previous, committed, trie, devices) = record_pair_removing_a_member_with_two_devices();
    let order: Vec<_> = notices_for(&previous, &committed, &trie).unwrap().iter().map(|notice| notice.device).collect();
    assert_eq!(order, devices);
    assert!(notices_for(&committed, &committed, &trie).unwrap().is_empty());
}

/// verifies: LLR-kr5t6f
///
/// Abnormal: an uncalculated trie is refused with the trie's error.
#[test]
fn notices_for_refuses_an_uncalculated_trie() {
    let (previous, committed, _, (member_id, _)) = record_pair_removing_one_device();
    let uncalculated = trie_with_bob(vec![org_node::test_fixtures::bob_device().device_key().unwrap()])
        .delete_member(&member_id)
        .unwrap();
    assert!(!uncalculated.is_calculated(), "fixture: delete_member without recalculate");
    assert!(matches!(
        notices_for(&previous, &committed, &uncalculated),
        Err(org_node::OrgNodeError::Trie(org_members::OrgMembersError::HashesNotCalculated))
    ));
}

/// verifies: LLR-bs6yxf
///
/// The signed bytes are the domain then the postcard tuple; the signature
/// is not part of them.
#[test]
fn acknowledgement_signed_bytes_are_the_domain_then_the_postcard_tuple() {
    let ack = sample_acknowledgement();
    let mut expected = ACK_DOMAIN.to_vec();
    expected.extend(postcard::to_allocvec(&(ack.org_id, ack.member_id, ack.device, ack.epoch, ack.root)).unwrap());
    assert_eq!(ack.signed_bytes(), expected);
    let other = Acknowledgement { signature: Signature64([0xAA; 64]), ..ack.clone() };
    assert_eq!(other.signed_bytes(), expected);
    assert_eq!(ACK_DOMAIN, b"ods/org-node/revocation-acknowledgement/v1");
}

/// verifies: LLR-bs6yxf, PR-y6ay44
///
/// PR-y6ay44: the signed bytes are the domain, then exactly the tuple's
/// postcard encoding. Boundary: the epoch's varint at one byte (127), two
/// bytes (128, 300) and its longest (`u64::MAX`, ten bytes).
#[test]
fn signed_bytes_are_the_domain_then_the_tuple_at_each_varint_length() {
    for epoch in [127, 128, 300, u64::MAX] {
        let ack = Acknowledgement {
            org_id: org(),
            member_id: bob_id(),
            device: org_node::test_fixtures::bob_device().device_key().unwrap(),
            epoch: Epoch::new(epoch),
            root: RootHash::new([0x66; 32]),
            signature: Signature64([0; 64]),
        };
        let tuple = postcard::to_allocvec(&(ack.org_id, ack.member_id, ack.device, ack.epoch, ack.root)).unwrap();
        assert_eq!(ack.signed_bytes(), [ACK_DOMAIN, &tuple[..]].concat(), "epoch {epoch}");
    }
}

// --- Deciding a notice ---

/// `encodable`'s postcard encoding.
fn postcard_of<T: serde::Serialize>(encodable: &T) -> Vec<u8> {
    postcard::to_allocvec(encodable).unwrap()
}

/// A device seed as these tests hold it: its bytes. A secret is made from
/// the bytes as org-io's keychain hands one back.
#[derive(Clone, Copy)]
struct DeviceSeed([u8; 32]);

impl DeviceSeed {
    fn expose_secret(&self) -> &[u8; 32] {
        &self.0
    }

    fn secret(&self) -> DeviceSecret {
        org_node::test_fixtures::device_secret(self.0)
    }

    fn device_key(&self) -> DevicePublicKey {
        self.secret().device_key()
    }
}

/// The seed of the bound Persona's Device D (bob's Device).
fn bound_device_seed() -> DeviceSeed {
    DeviceSeed(BOB_DEVICE_SEED)
}

/// D: the bound Persona's DevicePublicKey.
fn bound_device() -> DevicePublicKey {
    bound_device_seed().device_key()
}

/// A store with one Persona bound to the Organisation as bob (M) with
/// Device D, and the record at epoch 4 listing M with D.
fn bound_store() -> StoreData {
    let persona = PersonaRecord::from(PersonaRecordParts {
        org_id: Some(org()),
        name: parsed_name("Bob"),
        surname: parsed_surname("B"),
        handle: Some(handle("bob")),
        member_id: Some(bob_id()),
        status: PersonaStatus::Active,
        ..persona_parts("bob", [2; 32], bound_device())
    });
    StoreData { record: Some(record_at(&trie_with_bob(vec![bound_device()]), 4)), ..StoreData::new(persona) }
}

/// The calculated trie of the record with D removed (bob deleted).
fn removed_trie() -> Trie {
    trie_with_bob(vec![bound_device()]).delete_member(&bob_id()).unwrap().recalculate().unwrap().0
}

fn removed_root() -> RootHash {
    removed_trie().root_hash().unwrap()
}

fn still_listing_root() -> RootHash {
    bound_store().record.unwrap().root_hash
}

fn genuine_notice() -> RevocationNotice {
    RevocationNotice {
        org_id: org(),
        member_id: bob_id(),
        device: bound_device(),
        proof: removed_trie().prove_absent(&bob_id(), &bound_device()).unwrap(),
    }
}

fn chain_at(epoch: u64, root: RootHash) -> OrgState {
    OrgState { root_hash: root, org_pub_key: org_node::test_fixtures::org_public_key(), epoch: Epoch::new(epoch) }
}

/// verifies: LLR-r7zm39
///
/// Abnormal, in order: an unheld Organisation, then another Device; normal:
/// the bound Persona is returned.
#[test]
fn check_notice_refuses_an_unheld_organisation_then_another_device() {
    let store = bound_store();
    let unheld = RevocationNotice { org_id: OrgId::new([9; 20]), device: device_key(77), ..genuine_notice() };
    assert!(matches!(check_notice(&store, &unheld), Err(OrgNodeError::RevocationNotHeld { .. })));
    let other = RevocationNotice { device: device_key(77), ..genuine_notice() };
    assert!(matches!(check_notice(&store, &other), Err(OrgNodeError::RevocationNotForThisDevice { .. })));
    let other_member = RevocationNotice { member_id: MemberId::new([8; 32]), ..genuine_notice() };
    assert!(matches!(check_notice(&store, &other_member), Err(OrgNodeError::RevocationNotForThisDevice { .. })));
    assert_eq!(check_notice(&store, &genuine_notice()).unwrap().member_id, Some(bob_id()));
}

/// verifies: LLR-rhsfw9
///
/// Abnormal, in order: an unheld notice; no chain state; a state older than
/// the record; a proof against another root; a Device the chain's record
/// still lists. Each returns the error alone. (`accept` borrows the store
/// immutably, so it cannot change it; that a refused notice stays kept in an
/// `OrgService` store is LLR-f9mcsq's, tested in own_removal.rs.)
#[test]
fn accept_refuses_in_order() {
    let store = bound_store();
    let unheld = RevocationNotice { org_id: OrgId::new([9; 20]), ..genuine_notice() };
    assert!(matches!(accept(&store, &unheld, None), Err(OrgNodeError::RevocationNotHeld { .. })));
    assert!(matches!(accept(&store, &genuine_notice(), None), Err(OrgNodeError::OrgNotOnChain)));
    assert!(matches!(
        accept(&store, &genuine_notice(), Some(chain_at(3, removed_root()))),
        Err(OrgNodeError::StaleChainState { .. })
    ));
    assert!(matches!(
        accept(&store, &genuine_notice(), Some(chain_at(5, RootHash::new([5; 32])))),
        Err(OrgNodeError::RevocationProofRefused { .. })
    ));
    assert!(matches!(
        accept(&store, &genuine_notice(), Some(chain_at(5, still_listing_root()))),
        Err(OrgNodeError::RevocationProofRefused { .. })
    ));
}

/// verifies: LLR-rhsfw9
///
/// Abnormal, each refusal in order: the refusal is returned, and `accept`
/// has no secret source to ask — a refused notice reads no Device secret.
#[test]
fn a_refused_notice_is_refused_in_order_with_no_secret_asked_for() {
    let store = bound_store();
    let unheld = RevocationNotice { org_id: OrgId::new([9; 20]), ..genuine_notice() };
    assert!(matches!(accept(&store, &unheld, None), Err(OrgNodeError::RevocationNotHeld { .. })));
    let other = RevocationNotice { device: device_key(77), ..genuine_notice() };
    assert!(matches!(accept(&store, &other, None), Err(OrgNodeError::RevocationNotForThisDevice { .. })));
    assert!(matches!(accept(&store, &genuine_notice(), None), Err(OrgNodeError::OrgNotOnChain)));
    assert!(matches!(
        accept(&store, &genuine_notice(), Some(chain_at(3, removed_root()))),
        Err(OrgNodeError::StaleChainState { .. })
    ));
    assert!(matches!(
        accept(&store, &genuine_notice(), Some(chain_at(5, RootHash::new([5; 32])))),
        Err(OrgNodeError::RevocationProofRefused { .. })
    ));
    assert!(matches!(
        accept(&store, &genuine_notice(), Some(chain_at(5, still_listing_root()))),
        Err(OrgNodeError::RevocationProofRefused { .. })
    ));
}

/// verifies: LLR-rhsfw9
///
/// Abnormal: a chain state at the record's own epoch whose root is not the
/// record's (here the root the notice's proof verifies against, and another)
/// → `ChainStateConflict`, before the proof is checked; the store is
/// unchanged.
#[test]
fn a_chain_state_at_the_records_epoch_with_another_root_is_a_conflict() {
    let store = bound_store();
    let before = postcard_of(&store);
    let record_epoch = 4;
    assert_eq!(store.record.as_ref().unwrap().epoch, Epoch::new(record_epoch), "fixture: the record is at epoch 4");
    for root in [removed_root(), RootHash::new([5; 32])] {
        assert_eq!(
            accept(&store, &genuine_notice(), Some(chain_at(record_epoch, root))),
            Err(OrgNodeError::ChainStateConflict { org_id: org() })
        );
    }
    assert_eq!(postcard_of(&store), before);
}

/// A Persona named `id`, bound to `org_id` as `member_id`, with the Device
/// of `device_seed`.
fn persona(id: &str, org_id: Option<OrgId>, member_id: Option<MemberId>, device_seed: DeviceSeed) -> PersonaRecord {
    PersonaRecord::from(PersonaRecordParts {
        org_id,
        name: parsed_name("Some"),
        surname: parsed_surname("One"),
        handle: Some(handle(id)),
        member_id,
        status: PersonaStatus::Active,
        ..persona_parts(id, [0x31; 32], device_seed.device_key())
    })
}

/// verifies: LLR-r7zm39
///
/// Normal: among Personas that are unbound, bound to another Organisation,
/// or bound to this one as another Member — each listed before it — the
/// Persona bound to the notice's Organisation as its Member on its Device is
/// the one returned. A store holds one Persona, so each decoy is the Persona
/// of a store of its own, which `check_notice` refuses.
#[test]
fn check_notice_returns_the_persona_the_notice_is_for() {
    let store = bound_store();
    for decoy in [
        persona("unbound", None, None, bound_device_seed()),
        persona("elsewhere", Some(OrgId::new([9; 20])), Some(bob_id()), bound_device_seed()),
        persona("othermember", Some(org()), Some(MemberId::new([8; 32])), bound_device_seed()),
    ] {
        let decoy_store = StoreData { persona: decoy, ..store.clone() };
        assert!(matches!(
            check_notice(&decoy_store, &genuine_notice()),
            Err(OrgNodeError::RevocationNotForThisDevice { .. })
        ));
    }
    let found = check_notice(&store, &genuine_notice()).unwrap();
    assert_eq!(found.persona_id, PersonaId::new("bob".into()));
    assert_eq!((found.org_id, found.member_id), (Some(org()), Some(bob_id())));
}

// --- Checking a received acknowledgement ---

/// An admin's store: the record of the Organisation at epoch 5, listing D
/// (bob still holds it) or not (bob deleted). No Persona is bound.
fn admin_store(listing_d: bool) -> StoreData {
    let trie = if listing_d { trie_with_bob(vec![bound_device()]) } else { removed_trie() };
    let admin = persona("admin", None, None, DeviceSeed([0x51; 32]));
    StoreData { record: Some(record_at(&trie, 5)), ..StoreData::new(admin) }
}

/// The acknowledgement of (the Organisation, bob, `seed`'s public key,
/// `epoch`, the removed root), signed with `seed`.
fn signed_ack(seed: &DeviceSeed, epoch: Epoch) -> Acknowledgement {
    let mut ack = Acknowledgement {
        org_id: org(),
        member_id: bob_id(),
        device: seed.device_key(),
        epoch,
        root: removed_root(),
        signature: Signature64([0; 64]),
    };
    let signing_key = ed25519_dalek::SigningKey::from_bytes(seed.expose_secret());
    ack.signature = Signature64(ed25519_dalek::Signer::sign(&signing_key, &ack.signed_bytes()).to_bytes());
    ack
}

/// A store whose record no longer lists bob's bound Device, and bob's
/// genuine acknowledgement at the record's epoch.
fn genuine_acknowledgement_fixture() -> (StoreData, Acknowledgement) {
    (admin_store(false), signed_ack(&bound_device_seed(), Epoch::new(5)))
}

/// verifies: LLR-5azhry
///
/// An acknowledgement delivered by any Device but the one it names is
/// refused before its signature is checked: a forged signature from the
/// wrong sender reports the sender, not the signature.
#[test]
fn an_acknowledgement_from_another_device_is_refused_before_its_signature() {
    let (store, ack) = genuine_acknowledgement_fixture();
    let before = postcard_of(&store);
    let other = device_key(0x61);
    assert_ne!(other, ack.device);
    let mut forged = ack.clone();
    forged.signature = Signature64([0; 64]);
    for candidate in [ack.clone(), forged] {
        assert_eq!(
            check_acknowledgement(&store, other, candidate).map(|verified| verified.into_inner()),
            Err(OrgNodeError::AcknowledgementNotFromItsDevice { org_id: ack.org_id })
        );
    }
    assert!(check_acknowledgement(&store, ack.device, ack.clone()).is_ok());
    assert_eq!(postcard_of(&store), before);
}

/// verifies: LLR-5azhry
///
/// Abnormal, order: an acknowledgement for an unheld Organisation from the
/// wrong sender reports the Organisation, not the sender; one from the wrong
/// sender that is also from the future and for a listed Device reports the
/// sender.
#[test]
fn the_sender_check_comes_after_the_held_lookup_and_before_the_rest() {
    let other = device_key(0x61);
    let mut future = signed_ack(&bound_device_seed(), Epoch::new(6));
    future.signature.0[0] ^= 1;
    let unheld = Acknowledgement { org_id: OrgId::new([9; 20]), ..future.clone() };
    assert!(matches!(
        check_acknowledgement(&admin_store(true), other, unheld),
        Err(OrgNodeError::AcknowledgementNotHeld { .. })
    ));
    assert!(matches!(
        check_acknowledgement(&admin_store(true), other, future),
        Err(OrgNodeError::AcknowledgementNotFromItsDevice { .. })
    ));
}

/// verifies: LLR-5azhry
///
/// Normal: a genuine acknowledgement of a Device the record no longer lists,
/// at an epoch equal to or below the record's, verifies; the store is
/// untouched.
#[test]
fn a_genuine_acknowledgement_verifies() {
    let store = admin_store(false);
    let before = postcard_of(&store);
    for epoch in [5, 4] {
        let ack = signed_ack(&bound_device_seed(), Epoch::new(epoch));
        assert_eq!(check_acknowledgement(&store, ack.device, ack.clone()).unwrap().into_inner(), ack);
    }
    assert_eq!(postcard_of(&store), before);
}

/// verifies: LLR-5azhry
///
/// Abnormal, cheapest first: unheld; from the future; for a listed Device;
/// a flipped signature byte; another key's signature. Each is checked with
/// every later check also failing, so the order is what decides the error.
#[test]
fn acknowledgements_are_refused_in_order() {
    let store = admin_store(false);
    let ack = signed_ack(&bound_device_seed(), Epoch::new(5));
    let mut unsigned_future = signed_ack(&bound_device_seed(), Epoch::new(6));
    unsigned_future.signature.0[0] ^= 1;
    let sender = ack.device;
    let unheld = Acknowledgement { org_id: OrgId::new([9; 20]), ..unsigned_future.clone() };
    assert!(matches!(
        check_acknowledgement(&admin_store(true), sender, unheld),
        Err(OrgNodeError::AcknowledgementNotHeld { .. })
    ));
    assert!(matches!(
        check_acknowledgement(&admin_store(true), sender, unsigned_future),
        Err(OrgNodeError::AcknowledgementFromFuture { .. })
    ));
    let mut unsigned = ack.clone();
    unsigned.signature.0[0] ^= 1;
    assert!(matches!(
        check_acknowledgement(&admin_store(true), sender, unsigned),
        Err(OrgNodeError::AcknowledgementForListedDevice { .. })
    ));
    let mut flipped = ack.clone();
    flipped.signature.0[0] ^= 1;
    assert!(matches!(
        check_acknowledgement(&store, sender, flipped),
        Err(OrgNodeError::AcknowledgementSignatureInvalid { .. })
    ));
    let foreign = Acknowledgement { signature: signed_ack(&DeviceSeed([99; 32]), Epoch::new(5)).signature, ..ack };
    assert!(matches!(
        check_acknowledgement(&store, sender, foreign),
        Err(OrgNodeError::AcknowledgementSignatureInvalid { .. })
    ));
}

/// verifies: LLR-bs6yxf
///
/// Abnormal: a signature by the named Device over the tuple without
/// `ACK_DOMAIN`, or over the tuple behind another domain, is not a signature
/// over the acknowledgement's signed bytes; `check_acknowledgement` refuses
/// it, and the genuine one verifies.
#[test]
fn a_signature_without_the_acknowledgement_domain_is_refused() {
    let store = admin_store(false);
    let genuine = signed_ack(&bound_device_seed(), Epoch::new(5));
    assert!(check_acknowledgement(&store, genuine.device, genuine.clone()).is_ok());
    let tuple = postcard_of(&(genuine.org_id, genuine.member_id, genuine.device, genuine.epoch, genuine.root));
    let mut other_domain = b"ods/org-node/some-other-message/v1".to_vec();
    other_domain.extend(&tuple);
    let signing_key = ed25519_dalek::SigningKey::from_bytes(bound_device_seed().expose_secret());
    for message in [tuple, other_domain] {
        let signature = Signature64(ed25519_dalek::Signer::sign(&signing_key, &message).to_bytes());
        let off_domain = Acknowledgement { signature, ..genuine.clone() };
        assert_eq!(
            check_acknowledgement(&store, off_domain.device, off_domain).map(|verified| verified.into_inner()),
            Err(OrgNodeError::AcknowledgementSignatureInvalid { org_id: org() })
        );
    }
}

/// verifies: LLR-5azhry
///
/// Abnormal: an acknowledgement whose epoch, root or Member was edited
/// after signing is refused; the signature covers each.
#[test]
fn an_acknowledgement_with_an_edited_field_is_refused() {
    let store = admin_store(false);
    let ack = signed_ack(&bound_device_seed(), Epoch::new(5));
    let edits = [
        Acknowledgement { epoch: Epoch::new(4), ..ack.clone() },
        Acknowledgement { root: RootHash::new([5; 32]), ..ack.clone() },
        Acknowledgement { member_id: MemberId::new([8; 32]), ..ack },
    ];
    for edited in edits {
        assert!(matches!(
            check_acknowledgement(&store, edited.device, edited),
            Err(OrgNodeError::AcknowledgementSignatureInvalid { .. })
        ));
    }
}

/// verifies: LLR-5azhry
///
/// Abnormal: Device bytes that are no curve point (y = 2 has no x on
/// edwards25519) never reach `check_acknowledgement`. An acknowledgement
/// that passes every earlier check, with only its Device bytes replaced by
/// them, does not decode, because `DevicePublicKey` parses its bytes; so no
/// such Device reaches the check's signature verification from safe code.
/// The store is unchanged.
#[test]
fn an_acknowledgement_whose_device_is_no_curve_point_does_not_decode() {
    let mut no_point = [0u8; 32];
    no_point[0] = 2;
    assert!(ed25519_dalek::VerifyingKey::from_bytes(&no_point).is_err());
    assert!(DevicePublicKey::try_from(no_point).is_err());

    let store = admin_store(false);
    let before = postcard_of(&store);
    let ack = signed_ack(&bound_device_seed(), Epoch::new(5));
    assert!(check_acknowledgement(&store, ack.device, ack.clone()).is_ok());
    let mut bytes = postcard_of(&ack);
    let device = *ack.device.as_bytes();
    let at = bytes.windows(32).position(|window| window == device).unwrap();
    bytes[at..at + 32].copy_from_slice(&no_point);
    assert!(postcard::from_bytes::<Acknowledgement>(&bytes).is_err());
    assert_eq!(postcard_of(&store), before);
}

mod acknowledgement_domain {
    use org_members::{MemberId, RootHash};
    use org_node::ids::OrgId;
    use org_node::revocation::{Acknowledgement, Signature64, ACK_DOMAIN};
    use org_node::test_fixtures::device_key;
    use org_node::Epoch;

    // verifies: LLR-bs6yxf
    #[test]
    fn acknowledgement_signed_bytes_are_the_domain_then_the_tuple() {
        let ack = Acknowledgement {
            org_id: OrgId::new([0xaa; 20]),
            member_id: MemberId::new([1; 32]),
            device: device_key(4),
            epoch: Epoch::new(4),
            root: RootHash::new([0x66; 32]),
            signature: Signature64([0x5a; 64]),
        };
        let mut expected = b"ods/org-node/revocation-acknowledgement/v1".to_vec();
        expected.extend(postcard::to_allocvec(&(ack.org_id, ack.member_id, ack.device, ack.epoch, ack.root)).unwrap());
        assert_eq!(ack.signed_bytes(), expected);
        assert_eq!(ACK_DOMAIN, b"ods/org-node/revocation-acknowledgement/v1");
        // No other message org-node signs or verifies begins with ACK_DOMAIN: every signature org-node
        // makes or checks is over an acknowledgement's `signed_bytes()` (the key wrapper in keys.rs
        // signs what it is handed), and the domain's bytes are written in the source once.
        let mut domain_literals = 0;
        for (path, text) in crate::support::src_rust_files() {
            let file = path.file_name().unwrap().to_string_lossy().to_string();
            let code: String =
                text.lines().map(|line| line.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n");
            domain_literals += code.matches("revocation-acknowledgement").count();
            for call in [".sign(", ".verify_strict(", ".verify("] {
                for (offset, _) in code.match_indices(call) {
                    let argument = &code[offset + call.len()..];
                    let argument = &argument[..argument.find(['\n', ';']).unwrap_or(argument.len())];
                    let wrapper = file == "keys.rs" && call == ".sign(" && argument.starts_with("message)");
                    assert!(
                        wrapper || argument.contains("signed_bytes()"),
                        "{file}: `{call}{argument}` signs or verifies a message other than an acknowledgement"
                    );
                }
            }
        }
        assert_eq!(domain_literals, 1, "the acknowledgement domain's bytes are written once, as ACK_DOMAIN");
    }
}

mod accept_decides_only {
    use super::{bound_store, chain_at, genuine_notice, postcard_of, removed_root};
    use org_node::revocation::accept;

    // A notice passing `check_notice`, against a chain state of a later epoch than the record's (4) whose
    // root no longer holds the Device, is accepted as `Ok(())`: nothing is returned but the decision, the
    // store passed in is unchanged, and `accept` takes no seed or secret source (LLR-rhsfw9): the call
    // below names its three arguments and compares its result with `Ok(())`, so the compiler pins both.
    // verifies: LLR-rhsfw9
    #[test]
    fn accept_returns_ok_for_a_later_epoch_without_the_device_and_takes_no_secret_source() {
        let store = bound_store();
        let before = postcard_of(&store);
        for epoch in [5, 9] {
            assert_eq!(accept(&store, &genuine_notice(), Some(chain_at(epoch, removed_root()))), Ok(()));
            assert_eq!(postcard_of(&store), before, "the store is unchanged");
        }
    }
}
