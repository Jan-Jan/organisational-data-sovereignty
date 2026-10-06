#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Encrypted-at-rest evidence for `PersonaStore` (REQ-hzm4kt): the store
//! round-trips through disk, a wrong passphrase yields an error rather than
//! data, and neither a persona secret nor an Organisation secret appears in
//! the clear in the file.

use std::path::PathBuf;

use org_members::{Handle, Name, RootHash, Surname};
use org_node::ids::OrgId;
use org_node::store::{OrgRecord, PersonaRecord, PersonaStatus, PersonaStore};
use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, OrgSecret, PersonaId, SequenceNumber};
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

/// An org record carrying `org_secret`, with the remaining fields fixed.
fn org_record(org_secret: Option<[u8; 32]>) -> OrgRecord {
    OrgRecord {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: OrgPrivateKey::from([0x22u8; 32]).x25519_keypair().org_public_key().unwrap(),
        epoch: Epoch::new(3),
        org_secret: org_secret.map(OrgSecret::from),
        last_seq: SequenceNumber::new(2),
        trie_members: Vec::new(),
        proxy_account: None,
        org_private_key: None,
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

// verifies: REQ-hzm4kt, LLR-s78sh7
#[test]
fn organisation_secret_does_not_appear_in_the_file() {
    let path = tmp_path("orgsecret");
    let mut s = PersonaStore::open(path.clone(), "pw").unwrap();
    let org_secret = [0x7eu8; 32];
    s.data_mut().orgs.push(org_record(Some(org_secret)));
    s.save(&mut OsRng).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, &org_secret), "org secret in clear");
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
