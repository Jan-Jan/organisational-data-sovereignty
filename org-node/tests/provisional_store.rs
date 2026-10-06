#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The provisional updates a Persona store keeps (REQ-xs4ab8, REQ-fwfku9):
//! identity, persistence and the 1 MiB bound (LLR-95753m, LLR-jq7qh7), where
//! a genesis update keeps the Organisation private key (LLR-qjz3q4,
//! REQ-ech45n), and the field-naming parse on open (LLR-8bum44).

use org_node::ids::OrgId;
use org_node::store::{
    ExpectedAdmission, MemberSnapshot, PersonaStore, ProvisionalChange, ProvisionalUpdate, StoreData,
    MAX_PROVISIONAL_BYTES,
};
use org_node::test_fixtures::{device_key, member_key, org_public_key};
use org_node::{Handle, InviteId, Name, OrgNodeError, OrgPrivateKey, PersonaId, RootHash, SequenceNumber, Surname};
use rand::rngs::OsRng;

fn change_set(org: u8, root: u8, bytes: usize) -> ProvisionalUpdate {
    ProvisionalUpdate {
        org_id: Some(OrgId::new([org; 20])),
        persona_id: PersonaId::new("p-alice".into()),
        base_root: Some(RootHash::new([0x33; 32])),
        resulting_root: RootHash::new([root; 32]),
        seq: SequenceNumber::new(3),
        org_pub_key: org_public_key(),
        change: ProvisionalChange::ChangeSet { change_set: vec![0xab; bytes] },
    }
}

const PRIVATE: [u8; 32] = [0x88; 32];

fn genesis(persona: &str, root: u8) -> ProvisionalUpdate {
    ProvisionalUpdate {
        org_id: None,
        persona_id: PersonaId::new(persona.into()),
        base_root: None,
        resulting_root: RootHash::new([root; 32]),
        seq: SequenceNumber::new(1),
        org_pub_key: OrgPrivateKey::from(PRIVATE).x25519_keypair().org_public_key().unwrap(),
        change: ProvisionalChange::Genesis {
            members: vec![MemberSnapshot {
                id: org_node::MemberId::new([1; 32]),
                handle: Handle::parse("alice").unwrap(),
                name: Name::parse("Alice").unwrap(),
                surname: Surname::parse("Smith").unwrap(),
                member_key: member_key(0x21),
                device_keys: vec![device_key(0x22)],
            }],
            org_private_key: OrgPrivateKey::from(PRIVATE),
        },
    }
}

fn store_path(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ods-provisional-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("store.bin")
}

fn contains(hay: &[u8], needle: &[u8]) -> usize {
    hay.windows(needle.len()).filter(|w| *w == needle).count()
}

// Normal: provisional updates and expected admissions reach the encrypted
// file and are read back, typed, when it is opened.
// verifies: REQ-xs4ab8, LLR-95753m, LLR-g76zqd
#[test]
fn provisional_updates_round_trip_through_the_encrypted_file() {
    let path = store_path("round-trip");
    let mut store = PersonaStore::open(path.clone(), "pw").unwrap();
    store.data_mut().insert_provisional(genesis("p-alice", 0x10)).unwrap();
    store.data_mut().insert_provisional(change_set(0xbb, 0x66, 4)).unwrap();
    let expectation = ExpectedAdmission { org_id: OrgId::new([0xcc; 20]), invite_id: InviteId::new([0xee; 32]) };
    store.data_mut().expected_admissions.push(expectation);
    store.save(&mut OsRng).unwrap();
    let reopened = PersonaStore::open(path, "pw").unwrap();
    assert_eq!(reopened.data().provisional_updates, vec![genesis("p-alice", 0x10), change_set(0xbb, 0x66, 4)]);
    assert_eq!(reopened.data().expected_admissions, vec![expectation]);
}

// Abnormal: the same identity replaces rather than duplicates; different
// identities — another root, another Persona's genesis — are all kept.
// verifies: REQ-xs4ab8, LLR-95753m
#[test]
fn an_update_with_the_same_identity_replaces_and_others_are_kept() {
    let mut data = StoreData::default();
    data.insert_provisional(change_set(0xbb, 0x66, 4)).unwrap();
    data.insert_provisional(change_set(0xbb, 0x66, 5)).unwrap();
    assert_eq!(data.provisional_updates, vec![change_set(0xbb, 0x66, 5)], "replaced, not added");
    data.insert_provisional(change_set(0xbb, 0x67, 4)).unwrap();
    data.insert_provisional(genesis("p-alice", 0x66)).unwrap();
    data.insert_provisional(genesis("p-bob", 0x66)).unwrap();
    assert_eq!(data.provisional_updates.len(), 4);
}

// Normal: a genesis update holds its Organisation private key, whose public
// key is the update's, once in the store's plaintext; abnormal: with the
// update gone, no copy of the key remains anywhere in the store.
// verifies: REQ-ech45n, LLR-qjz3q4
#[test]
fn the_genesis_private_key_lives_only_in_its_provisional_update() {
    let mut data = StoreData::default();
    let update = genesis("p-alice", 0x10);
    let ProvisionalChange::Genesis { org_private_key, .. } = &update.change else { panic!("genesis") };
    assert_eq!(org_private_key.x25519_keypair().org_public_key().unwrap(), update.org_pub_key);
    data.insert_provisional(update).unwrap();
    assert_eq!(contains(&postcard::to_allocvec(&data).unwrap(), &PRIVATE), 1);
    assert!(data.orgs.is_empty());
    data.provisional_updates.clear();
    assert_eq!(contains(&postcard::to_allocvec(&data).unwrap(), &PRIVATE), 0);
}

/// The postcard length of a one-update group whose change set is `n` bytes.
fn group_len(n: usize) -> usize {
    postcard::to_allocvec(&vec![change_set(0xbb, 0x66, n)]).unwrap().len()
}

// Normal (boundary): a group of exactly 1 MiB is kept; the bound is the
// wire frame's.
// verifies: REQ-fwfku9, LLR-jq7qh7
#[test]
fn a_group_of_exactly_the_bound_is_kept() {
    assert_eq!(MAX_PROVISIONAL_BYTES, org_node::transport::MAX_FRAME);
    let fixed = group_len(20_000) - 20_000; // the varint is three bytes from 16 384 up
    let n = MAX_PROVISIONAL_BYTES - fixed;
    assert_eq!(group_len(n), MAX_PROVISIONAL_BYTES);
    let mut data = StoreData::default();
    data.insert_provisional(change_set(0xbb, 0x66, n)).unwrap();
    assert_eq!(data.provisional_updates.len(), 1);
}

// Abnormal: one byte over, alone or as the sum of two, is refused with the
// limit named, and the store is exactly as it was; another Organisation's
// group is measured on its own.
// verifies: REQ-fwfku9, LLR-jq7qh7, LLR-mxskg9
#[test]
fn an_update_that_would_exceed_the_bound_is_refused_and_nothing_changes() {
    let fixed = group_len(20_000) - 20_000;
    let mut data = StoreData::default();
    assert_eq!(
        data.insert_provisional(change_set(0xbb, 0x66, MAX_PROVISIONAL_BYTES - fixed + 1)).unwrap_err(),
        OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES }
    );
    assert!(data.provisional_updates.is_empty());
    data.insert_provisional(change_set(0xbb, 0x66, 600_000)).unwrap();
    let before = postcard::to_allocvec(&data).unwrap();
    assert_eq!(
        data.insert_provisional(change_set(0xbb, 0x67, 600_000)).unwrap_err(),
        OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES }
    );
    assert_eq!(postcard::to_allocvec(&data).unwrap(), before, "StoreData exactly as it was");
    data.insert_provisional(change_set(0xdd, 0x67, 600_000)).unwrap();
    assert_eq!(data.provisional_updates.len(), 2, "another Organisation's group is its own");
}
