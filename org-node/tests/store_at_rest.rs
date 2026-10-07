#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Encrypted-at-rest evidence for `PersonaStore` (REQ-hzm4kt): the store
//! round-trips through disk, a wrong passphrase yields an error rather than
//! data, and neither a persona secret nor the Organisation private key appears in
//! the clear in the file.

use std::path::PathBuf;

use org_members::{Handle, MemberId, Name, RootHash, Surname};
use org_node::ids::OrgId;
use org_node::store::{
    ExpectedAdmission, OrgRecord, PersonaRecord, PersonaStatus, PersonaStore, ProvisionalChange, ProvisionalUpdate,
    StoreData,
};
use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, OrgPublicKey, PersonaId, SequenceNumber};
use rand::rngs::OsRng;

/// A fresh, per-test file path under the OS temp dir (any stale file removed).
fn tmp_path(suffix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ods-store-at-rest-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("store-{suffix}.bin"));
    let _ = std::fs::remove_file(&path);
    path
}

fn persona(member_seed: [u8; 32], device_seed: [u8; 32]) -> PersonaRecord {
    PersonaRecord {
        persona_id: PersonaId::new("p1".to_string()),
        org_id: None,
        handle: Handle::parse("alice").unwrap(),
        name: Name::parse("A").unwrap(),
        surname: Surname::parse("U").unwrap(),
        member_seed: MemberSeed::from(member_seed),
        device_seed: DeviceSeed::from(device_seed),
        member_id: None,
        status: PersonaStatus::Proposed,
    }
}

/// An org record holding the Organisation private key `key`, with the
/// remaining fields fixed.
fn org_record(key: [u8; 32]) -> OrgRecord {
    OrgRecord {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: OrgPrivateKey::from(key).x25519_keypair().org_public_key().unwrap(),
        epoch: Epoch::new(3),
        last_seq: SequenceNumber::new(2),
        trie_members: Vec::new(),
        proxy_account: None,
        org_private_key: OrgPrivateKey::from(key),
        kept_change_set: None,
    }
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

// verifies: REQ-hzm4kt, LLR-wusj89, LLR-8mfjey
#[test]
fn round_trips_encrypted_through_disk() {
    let path = tmp_path("roundtrip");

    let mut s = PersonaStore::open(path.clone(), "hunter2").unwrap();
    s.data_mut().personas.push(persona([1u8; 32], [2u8; 32]));
    s.save(&mut OsRng).unwrap();

    // Reopen with the correct passphrase.
    let s2 = PersonaStore::open(path.clone(), "hunter2").unwrap();
    assert_eq!(s2.data().personas.len(), 1);
    assert_eq!(s2.data().personas[0].handle.as_str(), "alice");

    // Wrong passphrase must fail.
    assert!(PersonaStore::open(path.clone(), "wrong").is_err());

    let _ = std::fs::remove_file(&path);
}

// verifies: REQ-hzm4kt, LLR-t4u66w
#[test]
fn wrong_passphrase_yields_error_not_data() {
    let path = tmp_path("wrongpw");
    let mut s = PersonaStore::open(path.clone(), "correct horse").unwrap();
    s.data_mut().personas.push(persona([7u8; 32], [8u8; 32]));
    s.save(&mut OsRng).unwrap();
    let err = PersonaStore::open(path.clone(), "wrong").err().expect("must fail");
    assert!(err.to_string().contains("decrypt failed"), "got {err}");
    let _ = std::fs::remove_file(&path);
}

// verifies: REQ-hzm4kt, LLR-s78sh7
#[test]
fn seeds_do_not_appear_in_the_file() {
    let path = tmp_path("plaintext");
    let mut s = PersonaStore::open(path.clone(), "pw").unwrap();
    let member_seed = [0x5au8; 32];
    let device_seed = [0xa5u8; 32];
    s.data_mut().personas.push(persona(member_seed, device_seed));
    s.save(&mut OsRng).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, &member_seed), "member seed in clear");
    assert!(!contains(&bytes, &device_seed), "device seed in clear");
    assert!(!contains(&bytes, b"alice"), "handle in clear");
    let _ = std::fs::remove_file(&path);
}

// *Renamed 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `organisation_secret_does_not_appear_in_the_file`; LLR-s78sh7 names the
// Organisation private key in the secret's place.
// verifies: REQ-hzm4kt, LLR-s78sh7
#[test]
fn organisation_private_key_does_not_appear_in_the_file() {
    let path = tmp_path("orgkey");
    let mut s = PersonaStore::open(path.clone(), "pw").unwrap();
    let key = [0x7eu8; 32];
    s.data_mut().orgs.push(org_record(key));
    s.save(&mut OsRng).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, &key), "Organisation private key in clear");
    let _ = std::fs::remove_file(&path);
}

// ---- added 2026-10-03 by the architecture tooth ----------------------------

// A file too short to hold the nonce is refused as such, rather than being
// read as a nonce plus an empty ciphertext. The nonce is 24 bytes, so every
// length below that is this case; the boundary and a few below it are probed.
// verifies: REQ-hzm4kt, LLR-q5n28x
#[test]
fn a_file_shorter_than_the_nonce_is_refused() {
    for len in [0usize, 1, 23] {
        let path = tmp_path(&format!("short-{len}"));
        std::fs::write(&path, vec![0u8; len]).unwrap();
        let err = PersonaStore::open(path.clone(), "pw")
            .err()
            .unwrap_or_else(|| panic!("a {len}-byte file opened as a store"));
        let rendered = format!("{err}");
        assert!(
            rendered.contains("too short"),
            "a {len}-byte file must be refused as too short, got: {rendered}"
        );
    }
}

// Exactly the nonce length and no ciphertext is NOT the "too short" case: it
// passes the length check and fails to decrypt, which is a different verdict.
// This is what makes the bound `< 24` rather than `<= 24`.
// verifies: REQ-hzm4kt, LLR-q5n28x
#[test]
fn a_file_of_exactly_the_nonce_length_is_refused_as_undecryptable_not_as_short() {
    let path = tmp_path("exact-nonce");
    std::fs::write(&path, vec![0u8; 24]).unwrap();
    let err = PersonaStore::open(path, "pw").err().expect("opened a 24-byte file");
    let rendered = format!("{err}");
    assert!(!rendered.contains("too short"), "got the wrong verdict: {rendered}");
    assert!(rendered.contains("decrypt failed"), "got: {rendered}");
}

// Nonce freshness. The falsifiability sweep of 2026-10-03 replaced
// `rng.fill_bytes(&mut nonce)` with nothing — a fixed all-zero nonce on every
// save — and the whole suite stayed green: nothing here was asserting that the
// nonce ever changed. XChaCha20-Poly1305 does not survive nonce reuse under
// the same key, and the key is derived from a fixed application salt and the
// passphrase, so it IS the same key across saves of one store. This case is
// what makes that mutation red.
// verifies: REQ-hzm4kt, LLR-8mfjey
#[test]
fn each_save_draws_a_fresh_nonce() {
    let path = tmp_path("nonce-freshness");
    let mut store = PersonaStore::open(path.clone(), "pw").unwrap();
    store.data_mut().personas.push(persona([1u8; 32], [2u8; 32]));

    let mut nonces = Vec::new();
    for _ in 0..4 {
        store.save(&mut OsRng).unwrap();
        let blob = std::fs::read(&path).unwrap();
        nonces.push(blob[..24].to_vec());
    }

    for (i, a) in nonces.iter().enumerate() {
        for (j, b) in nonces.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "saves {i} and {j} reused a nonce");
            }
        }
    }
    // And none of them is the all-zero nonce a dropped draw would leave.
    assert!(nonces.iter().all(|n| n.iter().any(|b| *b != 0)));
}

// ---- added 2026-10-07 by stage S3 (docs/plans/2026-10-06-org-io-commit-workflow.md, T2)

const ORG_1: [u8; 20] = [1; 20];
const ORG_2: [u8; 20] = [2; 20];

/// A valid Organisation public key; which one does not matter here.
fn org_public_key() -> OrgPublicKey {
    OrgPrivateKey::from([0x42; 32]).x25519_keypair().org_public_key().unwrap()
}

/// The postcard bytes of `value`. The store types hold secrets and so have no
/// `PartialEq`; two values are compared by their encoding.
fn encoded<T: serde::Serialize>(value: &T) -> Vec<u8> {
    postcard::to_allocvec(value).unwrap()
}

/// Two Organisations, each with a bound Persona, a record, one Change-set
/// provisional update and one expectation; plus an unbound genesis update
/// built by org 1's Persona.
fn two_organisation_store() -> StoreData {
    let persona = |number: u8, org_id: OrgId| PersonaRecord {
        persona_id: PersonaId::new(format!("p{number}")),
        org_id: Some(org_id),
        handle: Handle::parse(&format!("h{number}")).unwrap(),
        name: Name::parse("N").unwrap(),
        surname: Surname::parse("S").unwrap(),
        member_seed: MemberSeed::from([number; 32]),
        device_seed: DeviceSeed::from([number + 10; 32]),
        member_id: Some(MemberId::new([number; 32])),
        status: PersonaStatus::Active,
    };
    let record = |org_id: OrgId| OrgRecord {
        org_id,
        root_hash: RootHash::new([org_id.as_bytes()[0]; 32]),
        org_pub_key: org_public_key(),
        epoch: Epoch::new(4),
        last_seq: SequenceNumber::new(4),
        trie_members: vec![],
        proxy_account: None,
        org_private_key: OrgPrivateKey::from([org_id.as_bytes()[0]; 32]),
        kept_change_set: Some(vec![org_id.as_bytes()[0]]),
    };
    let update = |org_id: Option<OrgId>, number: u8| ProvisionalUpdate {
        org_id,
        persona_id: PersonaId::new(format!("p{number}")),
        base_root: org_id.map(|org| RootHash::new([org.as_bytes()[0]; 32])),
        resulting_root: RootHash::new([number + 50; 32]),
        seq: SequenceNumber::new(5),
        org_pub_key: org_public_key(),
        change: ProvisionalChange::ChangeSet {
            change_set: vec![number],
            org_private_key: OrgPrivateKey::from([number + 60; 32]),
        },
    };
    let (org_1, org_2) = (OrgId::new(ORG_1), OrgId::new(ORG_2));
    StoreData {
        personas: vec![persona(1, org_1), persona(2, org_2)],
        orgs: vec![record(org_1), record(org_2)],
        provisional_updates: vec![update(Some(org_1), 1), update(None, 1), update(Some(org_2), 2)],
        expected_admissions: vec![ExpectedAdmission { org_id: org_1 }, ExpectedAdmission { org_id: org_2 }],
    }
}

/// verifies: LLR-pba7yu
///
/// Normal: forgetting org 1 removes its record, provisional updates (the
/// genesis one its Persona built included), expectation and Persona with its
/// keys; org 2's data is unchanged and in order; nothing names org 1.
#[test]
fn forget_organisation_removes_everything_of_one_organisation_and_nothing_else() {
    let data = two_organisation_store();
    let input_bytes = encoded(&data);
    let after = data.forget_organisation(OrgId::new(ORG_1));
    assert_eq!(encoded(&after.personas), encoded(&vec![data.personas[1].clone()]));
    assert_eq!(encoded(&after.orgs), encoded(&vec![data.orgs[1].clone()]));
    assert_eq!(after.provisional_updates, vec![data.provisional_updates[2].clone()]);
    assert_eq!(after.expected_admissions, vec![ExpectedAdmission { org_id: OrgId::new(ORG_2) }]);
    let after_bytes = encoded(&after);
    assert!(!contains(&after_bytes, &ORG_1), "no tombstone, marker or copy names org 1");
    for forgotten_key in [[1u8; 32], [11u8; 32]] {
        assert!(!contains(&after_bytes, &forgotten_key), "the forgotten Persona's seeds are gone");
    }
    assert_eq!(encoded(&data), input_bytes, "the input is unchanged");
}

/// verifies: LLR-pba7yu
///
/// Abnormal: forgetting an Organisation the store does not hold returns an
/// equal copy.
#[test]
fn forget_organisation_of_an_unheld_organisation_changes_nothing() {
    let data = two_organisation_store();
    assert_eq!(encoded(&data.forget_organisation(OrgId::new([9; 20]))), encoded(&data));
}

/// verifies: LLR-pba7yu
///
/// Boundary: forgetting the only Organisation of a store leaves an empty
/// store; forgetting from an empty store is an empty store.
#[test]
fn forget_organisation_of_the_last_organisation_leaves_an_empty_store() {
    let only_org_1 = two_organisation_store().forget_organisation(OrgId::new(ORG_2));
    let empty = only_org_1.forget_organisation(OrgId::new(ORG_1));
    assert_eq!(encoded(&empty), encoded(&StoreData::default()));
    assert_eq!(encoded(&empty.forget_organisation(OrgId::new(ORG_1))), encoded(&StoreData::default()));
}

/// verifies: LLR-d9778a
///
/// `kept_change_set` round-trips through the sealed store, `None` and `Some`.
#[test]
fn kept_change_set_round_trips_through_the_store() {
    for (index, kept) in [None, Some(vec![1u8, 2, 3])].into_iter().enumerate() {
        let path = tmp_path(&format!("kept-change-set-{index}"));
        let mut data = two_organisation_store();
        data.orgs[0].kept_change_set = kept.clone();
        let mut store = PersonaStore::open(path.clone(), "pw").unwrap();
        *store.data_mut() = data;
        store.save(&mut OsRng).unwrap();
        let reopened = PersonaStore::open(path.clone(), "pw").unwrap();
        assert_eq!(reopened.data().orgs[0].kept_change_set, kept);
        assert_eq!(reopened.data().orgs[1].kept_change_set, Some(vec![2]));
        let _ = std::fs::remove_file(&path);
    }
}
