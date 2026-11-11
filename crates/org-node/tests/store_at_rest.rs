#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Encrypted-at-rest evidence for `PersonaStore` (REQ-hzm4kt): the store
//! round-trips through its sealed bytes, another Device's secret yields an
//! error rather than data, and neither a persona secret nor the Organisation
//! private key appears in the clear in the sealed bytes.
//!
//! org-node writes no file and takes no passphrase. A store is sealed under
//! a key derived from its Persona's device secret (LLR-2qqqx6) and handed out
//! as a `StoreWrite`, whose bytes org-io writes; "the file" below is those
//! bytes, and "the wrong passphrase" is another Device's secret. The store
//! holds one Organisation (LLR-f5wrjx), and the write the tests read is
//! `build_invite`'s.

mod support;

use org_members::{Handle, MemberId, RootHash};
use org_node::ids::OrgId;
use org_node::service::StoreChange;
use org_node::store::{
    ExpectedAdmission, OrgRecord, OrgRecordParts, PersonaRecord, PersonaRecordParts, PersonaStatus, PersonaStore,
    SealedStore, StoreData, StoreRefusal, StoreWrite,
};
use org_node::test_fixtures::{device_secret, empty_store};
use org_node::{Epoch, OrgNodeError, OrgPrivateKey, OrgPublicKey, SequenceNumber};
use rand::rngs::OsRng;
use support::fixtures::{persona_parts, record_parts, service};
use support::{handle, parsed_name, parsed_surname};

/// The unbound Persona A U of the member seed `member_seed` and the Device
/// of `device_seed`.
fn persona(member_seed: [u8; 32], device_seed: [u8; 32]) -> PersonaRecord {
    PersonaRecord::from(PersonaRecordParts {
        name: parsed_name("A"),
        surname: parsed_surname("U"),
        ..persona_parts("p1", member_seed, device_secret(device_seed).device_key())
    })
}

/// `store_data`, as its Persona's store whose device seed is `[device; 32]`, opened
/// in an `OrgService` and written by one of its operations: the store change
/// `build_invite` returns (a store with no record is given one, as an
/// Invite needs it).
///
/// The bytes the tests read are those an operation hands org-io to write.
/// The store is opened from `store_data` with `store_for_test`: it stands for the
/// store org-io would have unsealed and handed to `OrgService::new`.
fn sealed(device: u8, mut store_data: StoreData) -> StoreWrite {
    if store_data.record.is_none() {
        store_data.record = Some(org_record([0x7d; 32]));
    }
    let device_key = store_data.persona.device_key;
    let mut svc = service(device, store_data);
    let (_, StoreChange::Sealed(write)) = svc.build_invite(&mut OsRng, &device_key, "Carol".into(), vec![]).unwrap();
    write
}

/// The sealed bytes of `write`, opened with the secret of `device_seed`.
fn unseal(write: &StoreWrite, device_seed: [u8; 32]) -> Result<PersonaStore, OrgNodeError> {
    PersonaStore::unseal(&write.sealed, write.device_key, device_secret(device_seed))
}

/// An org record holding the Organisation private key `key`, with the
/// remaining fields fixed.
fn org_record(key: [u8; 32]) -> OrgRecord {
    OrgRecord::from(OrgRecordParts {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: OrgPrivateKey::from(key).x25519_keypair().org_public_key().unwrap(),
        last_seq: SequenceNumber::new(2),
        org_private_key: OrgPrivateKey::from(key),
        ..record_parts(3, &[])
    })
}

// verifies: REQ-hzm4kt, LLR-wusj89, LLR-8mfjey
#[test]
fn round_trips_encrypted_through_disk() {
    let write = sealed(2, empty_store(persona([1u8; 32], [2u8; 32])));

    // Reopen with the right secret.
    let reopened = unseal(&write, [2u8; 32]).unwrap();
    assert_eq!(reopened.data().unwrap().persona.handle.as_ref().map(Handle::as_str), Some("alice"));

    // Another Device's secret must fail.
    assert!(unseal(&write, [3u8; 32]).is_err());
}

// verifies: REQ-hzm4kt, LLR-t4u66w
#[test]
fn wrong_passphrase_yields_error_not_data() {
    let write = sealed(8, empty_store(persona([7u8; 32], [8u8; 32])));
    let err = unseal(&write, [9u8; 32]).err().expect("must fail");
    assert_eq!(err, OrgNodeError::DeviceSecretMismatch, "got {err}");
    // The right secret over bytes that do not authenticate: an error too.
    let mut tampered = write.sealed.as_bytes().to_vec();
    tampered[24] ^= 0x80;
    let tampered = StoreWrite { device_key: write.device_key, sealed: SealedStore::from_bytes(tampered) };
    let err = unseal(&tampered, [8u8; 32]).err().expect("must fail");
    assert_eq!(err, OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic), "got {err}");
}

// verifies: REQ-hzm4kt, LLR-s78sh7
#[test]
fn seeds_do_not_appear_in_the_file() {
    let member_seed = [0x5au8; 32];
    let device = 0xa5u8;
    let device_seed = [device; 32];
    let bytes = sealed(device, empty_store(persona(member_seed, device_seed))).sealed.as_bytes().to_vec();
    assert!(!support::contains(&bytes, &member_seed), "member seed in clear");
    assert!(!support::contains(&bytes, &device_seed), "device seed in clear");
    assert!(!support::contains(&bytes, b"alice"), "handle in clear");
}

// The Organisation private key LLR-s78sh7 names is not in the sealed bytes.
// verifies: REQ-hzm4kt, LLR-s78sh7
#[test]
fn organisation_private_key_does_not_appear_in_the_file() {
    let key = [0x7eu8; 32];
    let mut store_data = empty_store(persona([1u8; 32], [2u8; 32]));
    store_data.record = Some(org_record(key));
    let bytes = sealed(2, store_data).sealed.as_bytes().to_vec();
    assert!(!support::contains(&bytes, &key), "Organisation private key in clear");
}

// ---- the length bound ------------------------------------------------------

/// Sealed bytes of `len` zero bytes, as the store of device seed `[2; 32]`.
fn zeros(len: usize) -> StoreWrite {
    StoreWrite { device_key: device_secret([2u8; 32]).device_key(), sealed: SealedStore::from_bytes(vec![0u8; len]) }
}

// A file too short to hold the nonce is refused as such, rather than being
// read as a nonce plus an empty ciphertext. The bound is the 24-byte nonce
// and the 16-byte authentication tag, 40 bytes; lengths below the nonce, the
// nonce itself and 39 are probed.
// verifies: REQ-hzm4kt, LLR-q5n28x
#[test]
fn a_file_shorter_than_the_nonce_is_refused() {
    for len in [0usize, 1, 23, 24, 39] {
        let err = unseal(&zeros(len), [2u8; 32])
            .err()
            .unwrap_or_else(|| panic!("a {len}-byte file opened as a store"));
        assert_eq!(err, OrgNodeError::StoreRefused(StoreRefusal::TooShort), "{len} bytes");
        let rendered = format!("{err}");
        assert!(
            rendered.contains("too short"),
            "a {len}-byte file must be refused as too short, got: {rendered}"
        );
    }
}

// Exactly the nonce and the tag, 40 bytes, and no ciphertext is NOT the "too
// short" case: it passes the length check and fails to decrypt, which is a
// different verdict. This is what makes the bound `< 40` rather than `<= 40`.
// verifies: REQ-hzm4kt, LLR-q5n28x
#[test]
fn a_file_of_exactly_the_nonce_length_is_refused_as_undecryptable_not_as_short() {
    let err = unseal(&zeros(40), [2u8; 32]).err().expect("opened a 40-byte file");
    let rendered = format!("{err}");
    assert!(!rendered.contains("too short"), "got the wrong verdict: {rendered}");
    assert_eq!(err, OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic), "got: {rendered}");
}

// Nonce freshness. XChaCha20-Poly1305 does not survive nonce reuse under the
// same key, and the key is derived from the Persona's device secret, so it IS
// the same key across seals of one store. A fixed all-zero nonce on every
// save fails this case. The seals are the writes four calls of
// `build_invite` return on one open store.
// verifies: REQ-hzm4kt, LLR-8mfjey
#[test]
fn each_save_draws_a_fresh_nonce() {
    let mut store_data = empty_store(persona([1u8; 32], [2u8; 32]));
    store_data.record = Some(org_record([0x7d; 32]));
    let device_key = store_data.persona.device_key;
    let mut svc = service(2, store_data);

    let mut nonces = Vec::new();
    for _ in 0..4 {
        let (_, StoreChange::Sealed(write)) =
            svc.build_invite(&mut OsRng, &device_key, "Carol".into(), vec![]).unwrap();
        nonces.push(write.sealed.as_bytes()[..24].to_vec());
    }

    for (first_index, first) in nonces.iter().enumerate() {
        for (second_index, second) in nonces.iter().enumerate() {
            if first_index != second_index {
                assert_ne!(first, second, "saves {first_index} and {second_index} reused a nonce");
            }
        }
    }
    // And none of them is the all-zero nonce a dropped draw would leave.
    assert!(nonces.iter().all(|nonce| nonce.iter().any(|byte| *byte != 0)));
}

// ---- the kept Change set -------------------------------------------------------

const ORG_1: [u8; 20] = [1; 20];

/// A valid Organisation public key; which one does not matter here.
fn org_public_key() -> OrgPublicKey {
    OrgPrivateKey::from([0x42; 32]).x25519_keypair().org_public_key().unwrap()
}

/// One store holds one Organisation. Its Persona is bound to org 1; the store holds org 1's record, whose kept Change set
/// is `[1]`, and an expectation of it.
fn one_organisation_store() -> StoreData {
    let persona = PersonaRecord::from(PersonaRecordParts {
        org_id: Some(OrgId::new(ORG_1)),
        name: parsed_name("N"),
        surname: parsed_surname("S"),
        handle: Some(handle("h1")),
        member_id: Some(MemberId::new([1; 32])),
        status: PersonaStatus::Active,
        ..persona_parts("p1", [1; 32], device_secret([11; 32]).device_key())
    });
    let record = OrgRecord::from(OrgRecordParts {
        org_id: OrgId::new(ORG_1),
        root_hash: RootHash::new([1; 32]),
        org_pub_key: org_public_key(),
        org_private_key: OrgPrivateKey::from([1; 32]),
        kept_change_set: Some(vec![1]),
        ..record_parts(4, &[])
    });
    StoreData {
        record: Some(record),
        expected_admissions: vec![ExpectedAdmission { org_id: OrgId::new(ORG_1) }],
        ..StoreData::new(persona)
    }
}

/// verifies: LLR-p8vpgz
///
/// `kept_change_set` round-trips through the sealed store, `None` and `Some`.
#[test]
fn kept_change_set_round_trips_through_the_store() {
    for kept in [None, Some(vec![1u8, 2, 3])] {
        let mut store_data = one_organisation_store();
        store_data.record.as_mut().expect("the fixture holds a record").kept_change_set = kept.clone();
        let write = sealed(11, store_data);
        let reopened = unseal(&write, [11; 32]).unwrap();
        assert_eq!(reopened.data().unwrap().record.as_ref().unwrap().kept_change_set, kept);
    }
}

/// Fixture of `the_kept_change_set_and_base_snapshot_are_the_adopted_ones`:
/// Ada's store holding a record of Ada and Bob, and the Organisation
/// information Bob sends admitting Carol at epoch 4.
mod adopted {
    use org_node::service::OrgService;
    use org_node::wire::{encode_message, WireMessage};
    use org_node::{Envelope, OrgPrivateKey, SequenceNumber};

    use super::support::fixtures::{ada, ada_store, bob, carol, service, ADA};
    use super::support::{leaf_of, snapshots_of, the_org, trie_of_members};

    /// Ada's service; the body Bob sends admitting Carol at epoch 4, the
    /// Change set it carries and the record snapshot it carries.
    pub fn setup() -> (OrgService, Vec<u8>, Vec<u8>, Vec<u8>) {
        let base = snapshots_of(&trie_of_members(&[ada(), bob()]));
        let base_trie = trie_of_members(&base);
        let svc = service(ADA, ada_store(3, &base));
        let (_next, delta) = base_trie.add_member(leaf_of(&carol())).unwrap().recalculate().unwrap();
        let envelope = Envelope::build(the_org(), SequenceNumber::new(4), &delta).unwrap();
        let change_set = envelope.delta_bytes.clone();
        let record_snapshot = postcard::to_allocvec(&base).unwrap();
        let body = encode_message(&WireMessage::OrgInformation {
            envelope,
            record_snapshot: record_snapshot.clone(),
            org_private_key: OrgPrivateKey::from([0x4c; 32]),
        })
        .unwrap();
        (svc, body, change_set, record_snapshot)
    }
}

/// verifies: LLR-p8vpgz
///
/// The record an adoption seals keeps, through the sealed store, the
/// adopted provisional organisation's Change set and base snapshot.
#[test]
fn the_kept_change_set_and_base_snapshot_are_the_adopted_ones() {
    use crate::support::fixtures::{ADA, BOB};
    use org_node::test_fixtures::{device_key, device_secret};
    let (mut svc, body, change_set, record_snapshot) = adopted::setup();
    let ada = device_key(ADA);
    let (kept, change) = svc.check_received(&mut OsRng, &ada, device_key(BOB), &body).unwrap();
    crate::support::unread(change);
    assert!(matches!(kept, org_node::service::Checked::ProvisionalOrganisationKept { .. }), "{kept:?}");
    let held = &svc.store_data(&ada).unwrap().provisional_organisations[0];
    let state = org_node::chain::OrgState { root_hash: held.root, org_pub_key: held.org_pub_key, epoch: Epoch::new(4) };
    let (adopted, change) = svc
        .verify(&mut OsRng, &ada, Some(state), org_node::VerificationTime(1), org_node::AheadLimit(1200), crate::support::none_held)
        .unwrap();
    assert!(matches!(adopted, org_node::Verified::Adopted { .. }), "{adopted:?}");
    let Some(org_node::service::StoreChange::Sealed(write)) = change else { panic!("adopted and sealed") };
    let reopened = PersonaStore::unseal(&write.sealed, ada, device_secret([ADA; 32])).unwrap();
    let record = reopened.data().unwrap().record.as_ref().unwrap();
    assert_eq!(record.kept_change_set, Some(change_set));
    assert_eq!(record.kept_base_snapshot, Some(record_snapshot));
}
