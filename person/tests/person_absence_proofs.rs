//! Proofs that a DevicePublicKey is not held under a Person hash
//! (SDD-bx9mm6).

#![cfg(feature = "serde")]
#![allow(clippy::expect_used)]

mod common;

use common::{device, person};
use person::{person_hash, prove_device_absent, EncodingVersion, IdentityError, PersonAbsenceProof, PersonHash};
use proptest::prelude::*;

fn hash_v1(definition: &person::Person) -> PersonHash {
    person_hash(definition, 1).expect("version 1")
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

/// verifies: LLR-985v9t, LLR-cve3d6, REQ-5kvqcf
///
/// The proof holds the definition's fields, its device root and an absence
/// proof of depth 4, and no DevicePublicKey: neither a held one nor the one
/// it is about appears in its encoding.
#[test]
fn a_proof_holds_the_fields_and_no_device_public_key() {
    let alice = person("Alice", "Example", Some(9), &[1, 2, 3]);
    let probe = device(4);
    let proof = prove_device_absent(&alice, &probe).expect("absent");
    assert_eq!(proof.version(), EncodingVersion::V1);
    assert_eq!(proof.name(), alice.name());
    assert_eq!(proof.surname(), alice.surname());
    assert_eq!(proof.person_key(), alice.person_key());
    assert_eq!(
        Ok(*proof.device_root()),
        person::compute_device_root::<person::device_root_hasher::PersonDeviceRootHasher>(alice.devices().devices())
    );
    assert_eq!(proof.device_root_absence().map().len(), 1, "a depth-4 map");
    let bytes = postcard::to_allocvec(&proof).expect("encode");
    for key in [device(1), device(2), device(3), probe] {
        assert!(!contains(&bytes, key.as_bytes()), "the proof carries a DevicePublicKey");
    }
}

/// verifies: LLR-cve3d6, LLR-z8h2w8, REQ-5kvqcf, REQ-uh2bzg
#[test]
fn a_proof_for_a_key_not_held_verifies_against_the_person_hash() {
    for (seeds, probe) in [(&[1u8, 2][..], 5u8), (&[][..], 1), (&[1, 2, 3, 4, 5, 6, 7, 8][..], 9)] {
        let key = if seeds.is_empty() { None } else { Some(9) };
        let definition = person("Alice", "Example", key, seeds);
        let proof = prove_device_absent(&definition, &device(probe)).expect("absent");
        assert_eq!(proof.verify(&hash_v1(&definition), &device(probe)), Ok(()), "{seeds:?}");
    }
}

/// verifies: LLR-cve3d6, REQ-5kvqcf
#[test]
fn no_proof_is_made_for_a_key_the_person_holds() {
    let alice = person("Alice", "Example", Some(9), &[1, 2]);
    assert_eq!(prove_device_absent(&alice, &device(2)).err(), Some(IdentityError::DeviceStillHeld));
}

/// verifies: LLR-z8h2w8, REQ-uh2bzg
///
/// The Person hash is checked first: a hash of another definition, or of a
/// tampered field, is `PersonHashMismatch` whatever the device.
#[test]
fn verify_checks_the_person_hash_first() {
    let alice = person("Alice", "Example", Some(9), &[1, 2]);
    let proof = prove_device_absent(&alice, &device(5)).expect("absent");
    let bob = hash_v1(&person("Bob", "Example", Some(9), &[1, 2]));
    assert_eq!(proof.verify(&bob, &device(5)), Err(IdentityError::PersonHashMismatch));
    assert_eq!(proof.verify(&bob, &device(1)), Err(IdentityError::PersonHashMismatch), "before the device check");
    let mut tampered = serde_json::to_value(&proof).expect("json");
    tampered["name"] = serde_json::json!("Mallory");
    let tampered: PersonAbsenceProof = serde_json::from_value(tampered).expect("well formed");
    assert_eq!(tampered.verify(&hash_v1(&alice), &device(5)), Err(IdentityError::PersonHashMismatch));
}

/// verifies: LLR-z8h2w8, REQ-uh2bzg
///
/// Once the Person hash matches: a proof about another position does not
/// resolve to the device root; a proof ending in a held key at the asked
/// position shows the key held.
#[test]
fn verify_then_checks_the_device_root_and_the_ending() {
    // device(17) shares device(1)'s position: its proof ends in device(1).
    let alice = person("Alice", "Example", Some(9), &[1, 2]);
    let hash = hash_v1(&alice);
    let proof = prove_device_absent(&alice, &device(17)).expect("absent");
    assert_eq!(proof.verify(&hash, &device(17)), Ok(()));
    assert_eq!(proof.verify(&hash, &device(1)), Err(IdentityError::DeviceStillHeld));
    // device(5) is at 0b0101, device(9) at 0b1001: their paths part at the
    // root, so one's siblings do not fold to the root along the other's path.
    let elsewhere = prove_device_absent(&alice, &device(5)).expect("absent");
    assert_eq!(elsewhere.verify(&hash, &device(9)), Err(IdentityError::DeviceRootMismatch));
}

/// verifies: LLR-985v9t, LLR-3dvfjb, REQ-uh2bzg
#[test]
fn a_proof_survives_the_wire_and_malformed_ones_are_refused() {
    let alice = person("Alice", "Example", Some(9), &[1, 2]);
    let proof = prove_device_absent(&alice, &device(5)).expect("absent");
    let bytes = postcard::to_allocvec(&proof).expect("encode");
    let back: PersonAbsenceProof = postcard::from_bytes(&bytes).expect("decode");
    assert_eq!(back, proof);
    assert_eq!(back.verify(&hash_v1(&alice), &device(5)), Ok(()));
    let json = serde_json::to_value(&proof).expect("json");
    for (field, replacement) in [
        ("version", serde_json::json!(2)),
        ("name", serde_json::json!("a".repeat(129))),
        ("device_root", serde_json::json!([1, 2, 3])),
        ("absence", serde_json::json!([8, [0], [], "Empty"])),
    ] {
        let mut broken = json.clone();
        broken[field] = replacement;
        let error = serde_json::from_value::<PersonAbsenceProof>(broken).expect_err(field);
        assert_eq!(error.to_string(), "malformed proof", "{field}");
    }
    assert!(postcard::from_bytes::<PersonAbsenceProof>(&bytes[..bytes.len() - 1]).is_err());
}

/// verifies: LLR-3dvfjb
#[test]
fn the_proof_refusals_are_distinct_named_variants() {
    let messages = [
        (IdentityError::DevicePositionTaken, "device position taken by another device public key"),
        (IdentityError::DeviceStillHeld, "device public key still held"),
        (IdentityError::PersonHashMismatch, "proof does not resolve to the person hash"),
        (IdentityError::DeviceRootMismatch, "proof does not resolve to the device root"),
        (IdentityError::ProofMalformed, "malformed proof"),
    ];
    for (variant, message) in &messages {
        assert_eq!(variant.to_string(), *message);
    }
    let refusals: Vec<IdentityError> = messages.into_iter().map(|(variant, _)| variant).collect();
    let earlier = [
        IdentityError::FieldTooLong { field: "name", max: 128 },
        IdentityError::InvalidDeviceKey,
        IdentityError::InvalidPersonKey,
        IdentityError::DeviceSlotsFull,
        IdentityError::DuplicateDevice,
        IdentityError::DeviceNotFound,
        IdentityError::KeyWithoutDevice,
        IdentityError::MissingPersonKey,
        IdentityError::PersonKeyIsDeviceKey,
        IdentityError::PersonKeyNotRotated,
        IdentityError::UnsupportedEncodingVersion(2),
    ];
    for (index, refusal) in refusals.iter().enumerate() {
        for other in refusals[index + 1..].iter().chain(&earlier) {
            assert_ne!(refusal, other);
            assert_ne!(refusal.to_string(), other.to_string());
        }
    }
}

proptest! {
    /// verifies: LLR-3dvfjb, REQ-bczz87
    #[test]
    fn decoding_and_verifying_any_bytes_does_not_panic(
        bytes in proptest::collection::vec(any::<u8>(), 0..400),
        hash in any::<[u8; 32]>(),
        seed in 1u8..=15,
    ) {
        if let Ok(proof) = postcard::from_bytes::<PersonAbsenceProof>(&bytes) {
            let _ = proof.verify(&PersonHash::new(hash), &device(seed));
        }
    }

    /// verifies: LLR-cve3d6, LLR-z8h2w8, REQ-5kvqcf
    #[test]
    fn every_proof_made_verifies(held in proptest::collection::btree_set(1u8..=15, 0..8), probe in 1u8..=15) {
        let seeds: Vec<u8> = held.iter().copied().collect();
        let key = if seeds.is_empty() { None } else { Some(99) };
        let definition = person("Alice", "Example", key, &seeds);
        match prove_device_absent(&definition, &device(probe)) {
            Ok(proof) => prop_assert_eq!(proof.verify(&hash_v1(&definition), &device(probe)), Ok(())),
            Err(error) => {
                prop_assert_eq!(error, IdentityError::DeviceStillHeld);
                prop_assert!(held.contains(&probe));
            }
        }
    }
}
