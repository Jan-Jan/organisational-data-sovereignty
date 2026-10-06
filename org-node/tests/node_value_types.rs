#![cfg(feature = "test-support")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! org-node's value types at their own interface: the secret types
//! (LLR-sz4xhc) and the tag types (LLR-s7whrn), both SDD-swtd3w's; seed to key
//! pair (LLR-56hc77, SDD-sxp8hb); and the Organisation public key's parse
//! (LLR-mmdu38, SDD-swtd3w). Named `value_types.rs` until the 2026-10-05 merge
//! of master, which brought a `value_types.rs` of its own (SDD-swtd3w's
//! identifier and rejection vocabulary).

use ed25519_dalek::SigningKey;
use org_node::keys::{SigningKeypair, X25519Keypair};
use org_node::{implements_copy, implements_display};
use org_node::{
    ChainAccount, DeviceSeed, Epoch, MemberSeed, OrgNodeError, OrgPrivateKey, OrgPublicKey,
    OrgSecret, PersonaId, SequenceNumber,
};
use rand::rngs::OsRng;

/// A valid X25519 public key: the Organisation public key of a fixed secret.
fn x25519_key(seed: u8) -> [u8; 32] {
    *OrgPrivateKey::from([seed; 32]).x25519_keypair().org_public_key().unwrap().as_bytes()
}

/// y = 0 / u = 0: a point of small order. Decompresses as Edwards; refused
/// by the X25519 rule.
fn weak_key() -> [u8; 32] {
    [0u8; 32]
}

/// p + 1: a non-canonical encoding. Decompresses as Edwards; refused by the
/// X25519 rule.
fn non_canonical_key() -> [u8; 32] {
    let mut b = [0xffu8; 32];
    b[0] = 0xee;
    b[31] = 0x7f;
    b
}

fn plain<T: serde::Serialize>(v: &T) -> Vec<u8> {
    postcard::to_allocvec(v).unwrap()
}

/// verifies: LLR-sz4xhc
#[test]
fn secret_types_redact_debug_and_give_bytes_only_through_the_accessor() {
    let bytes = [0x5au8; 32];
    let member = MemberSeed::from(bytes);
    let device = DeviceSeed::from(bytes);
    let org = OrgSecret::from(bytes);
    assert_eq!(member.expose_secret(), &bytes);
    assert_eq!(device.expose_secret(), &bytes);
    assert_eq!(org.expose_secret(), &bytes);
    assert_eq!(format!("{member:?}"), "MemberSeed([REDACTED])");
    assert_eq!(format!("{device:?}"), "DeviceSeed([REDACTED])");
    assert_eq!(format!("{org:?}"), "OrgSecret([REDACTED])");
    assert_eq!(member.clone(), member);
    assert_ne!(OrgSecret::from([1u8; 32]), org);
    assert!(implements_display!(String) && implements_copy!(u8), "the probe itself works");
    assert!(!implements_display!(MemberSeed) && !implements_copy!(MemberSeed));
    assert!(!implements_display!(DeviceSeed) && !implements_copy!(DeviceSeed));
    assert!(!implements_display!(OrgSecret) && !implements_copy!(OrgSecret));
}

/// verifies: LLR-sz4xhc
#[test]
fn secret_debug_is_the_same_whatever_the_bytes() {
    // Boundary bytes, and bytes that spell the marker itself.
    for bytes in [[0u8; 32], [0xffu8; 32], *b"MemberSeed([REDACTED])0123456789"] {
        assert_eq!(format!("{:?}", MemberSeed::from(bytes)), "MemberSeed([REDACTED])");
        assert_eq!(format!("{:#?}", DeviceSeed::from(bytes)), "DeviceSeed([REDACTED])");
        assert_eq!(format!("{:?}", Some(OrgSecret::from(bytes))), "Some(OrgSecret([REDACTED]))");
    }
}

/// verifies: LLR-sz4xhc
#[test]
fn secret_types_serialise_as_the_plain_bytes() {
    let bytes = [0xa5u8; 32];
    assert_eq!(plain(&MemberSeed::from(bytes)), plain(&bytes));
    assert_eq!(plain(&DeviceSeed::from(bytes)), plain(&bytes));
    assert_eq!(plain(&OrgSecret::from(bytes)), plain(&bytes));
    let back: OrgSecret = postcard::from_bytes(&plain(&bytes)).unwrap();
    assert_eq!(back, OrgSecret::from(bytes));
    // Abnormal: 31 bytes are not a secret.
    assert!(postcard::from_bytes::<OrgSecret>(&plain(&bytes)[..31]).is_err());
}

// Adapted at the merge of master `1feb608` into worktree-person-shared-types:
// on this branch the Member-as-a-group key is X25519, so a member seed yields
// an `X25519Keypair` (`MemberSeed::x25519_keypair`, LLR-98ufry) and hands its
// seed back as a `MemberSeed`; a device seed yields a `SigningKeypair` as on
// master. LLR-56hc77's `MemberSeed::signing_keypair()` clause does not hold
// here; its rule — no seed passes through a plain byte array — does.
/// verifies: LLR-56hc77
#[test]
fn a_seed_yields_its_key_pair_and_a_key_pair_its_seed() {
    let member_kp = X25519Keypair::generate(&mut OsRng);
    assert_eq!(member_kp.member_seed().x25519_keypair().public_bytes(), member_kp.public_bytes());
    let device_kp = SigningKeypair::generate(&mut OsRng);
    assert_eq!(device_kp.device_seed().signing_keypair().verifying_key(), device_kp.verifying_key());
    // The device seed is the RFC 8032 secret key: any ed25519 implementation
    // derives the same key pair from it.
    assert_eq!(
        DeviceSeed::from([0x11; 32]).signing_keypair().verifying_key(),
        SigningKey::from_bytes(&[0x11; 32]).verifying_key()
    );
}

// Adapted at the same merge: master signed and verified with both seed types'
// key pairs; nothing signs on this branch (REQ-ag6kqm). Each seed at the byte
// bounds yields a key pair whose public key is valid in its role.
/// verifies: LLR-56hc77
#[test]
fn seeds_at_the_byte_bounds_yield_working_key_pairs() {
    for bytes in [[0u8; 32], [0xffu8; 32]] {
        let member = MemberSeed::from(bytes).x25519_keypair();
        let device = DeviceSeed::from(bytes).signing_keypair();
        assert!(member.member_key().is_ok(), "a valid Member-as-a-group key");
        assert!(device.device_key().is_ok(), "a valid DevicePublicKey");
    }
}

// The Organisation private key is a secret type like the three above, and the
// only way to its key pair and back. No requirement stated it until the
// merge of master `1feb608`. Normal case: a fresh key, its key pair, its
// public key and the round trip; boundary bytes still yield a valid
// Organisation public key.
/// verifies: LLR-322xfu, LLR-56hc77
#[test]
fn the_organisation_private_key_is_a_secret_type_and_the_only_way_to_its_key_pair() {
    let bytes = [0x5au8; 32];
    let key = OrgPrivateKey::from(bytes);
    assert_eq!(key.expose_secret(), &bytes);
    assert_eq!(format!("{key:?}"), "OrgPrivateKey([REDACTED])");
    assert_eq!(format!("{:#?}", Some(OrgPrivateKey::from([0u8; 32]))), "Some(\n    OrgPrivateKey([REDACTED]),\n)");
    assert_eq!(key.clone(), key);
    assert_ne!(OrgPrivateKey::from([1u8; 32]), key);
    assert!(!implements_display!(OrgPrivateKey) && !implements_copy!(OrgPrivateKey));
    assert_eq!(plain(&key), plain(&bytes), "serialises as the plain bytes");
    assert_eq!(postcard::from_bytes::<OrgPrivateKey>(&plain(&bytes)).unwrap(), key);
    assert!(postcard::from_bytes::<OrgPrivateKey>(&plain(&bytes)[..31]).is_err(), "31 bytes are not a key");

    let pair = key.x25519_keypair();
    assert_eq!(pair.org_private_key(), key, "the key pair hands back the same secret");
    assert_eq!(pair.org_private_key().x25519_keypair().public_bytes(), pair.public_bytes());
    assert_eq!(
        pair.org_public_key().unwrap().as_bytes(),
        &curve25519_dalek::montgomery::MontgomeryPoint::mul_base_clamped(bytes).to_bytes(),
        "the Organisation public key is the X25519 public key of the secret (RFC 7748)"
    );
    for boundary in [[0u8; 32], [0xffu8; 32]] {
        let public = OrgPrivateKey::from(boundary).x25519_keypair().org_public_key();
        assert!(public.is_ok(), "a secret of {boundary:?} yields a valid Organisation public key");
    }
}

// Was master's `org_public_key_accepts_every_curve_point_unchanged`
// (LLR-mmdu38's Edwards-point clause, which does not hold on this branch):
// the parse accepts what `person`'s X25519 rule accepts — here a key pair's
// public key and the canonical twist point u = 2, which master refused as off
// the Edwards curve — and keeps the bytes unchanged.
/// verifies: LLR-3jjgtw, LLR-mmdu38
#[test]
fn org_public_key_accepts_valid_x25519_keys_unchanged() {
    let mut twist = [0u8; 32];
    twist[0] = 2;
    for bytes in [x25519_key(0x11), twist] {
        let key = OrgPublicKey::parse(&bytes).unwrap();
        assert_eq!(key.as_bytes(), &bytes);
        assert_eq!(OrgPublicKey::try_from(bytes).unwrap(), key);
        assert_eq!(plain(&key), plain(&bytes), "serialises as the plain bytes");
        assert_eq!(postcard::from_bytes::<OrgPublicKey>(&plain(&bytes)).unwrap(), key);
    }
}

// Was master's `org_public_key_refuses_bytes_off_the_curve` (`InvalidKey`):
// the X25519 rule refuses the small-order u = 0 and the non-canonical p + 1,
// both of which master accepted, with `InvalidOrgPublicKey`, on decode too.
/// verifies: LLR-3jjgtw, LLR-mmdu38
#[test]
fn org_public_key_refuses_what_the_x25519_rule_refuses() {
    for bytes in [weak_key(), non_canonical_key()] {
        assert_eq!(OrgPublicKey::parse(&bytes), Err(OrgNodeError::InvalidOrgPublicKey));
        assert_eq!(OrgPublicKey::try_from(bytes), Err(OrgNodeError::InvalidOrgPublicKey));
        assert!(postcard::from_bytes::<OrgPublicKey>(&plain(&bytes)).is_err());
    }
}

/// verifies: LLR-mmdu38
#[test]
fn org_public_key_debug_is_its_name_and_first_four_bytes() {
    // RFC 8032 test 1's public key.
    let bytes: [u8; 32] = [
        0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64, 0x07,
        0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68, 0xf7, 0x07,
        0x51, 0x1a,
    ];
    let key = OrgPublicKey::parse(&bytes).unwrap();
    let rendered = format!("{key:?}");
    assert_eq!(rendered, "OrgPublicKey(d75a9801..)");
    assert_eq!(format!("{key:#?}"), "OrgPublicKey(d75a9801..)");
    let full: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    assert!(!rendered.contains(&full), "the full key is not rendered");
}

/// verifies: LLR-s7whrn
#[test]
fn tag_types_debug_renders_their_value() {
    let account: [u8; 32] = core::array::from_fn(|i| i as u8);
    assert_eq!(
        format!("{:?}", ChainAccount::new(account)),
        "ChainAccount(0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f)"
    );
    assert_eq!(format!("{:?}", PersonaId::new("p-alice".to_string())), "PersonaId(\"p-alice\")");
    assert_eq!(format!("{:?}", Epoch::new(3)), "Epoch(3)");
    assert_eq!(format!("{:?}", SequenceNumber::new(u64::MAX)), "SequenceNumber(18446744073709551615)");
}

/// verifies: LLR-s7whrn
#[test]
fn tag_types_hold_their_value_unchanged() {
    let account = [0x5au8; 32];
    assert_eq!(ChainAccount::new(account).as_bytes(), &account);
    assert_eq!(ChainAccount::from(account), ChainAccount::new(account));
    assert_eq!(PersonaId::new("p-alice".to_string()).as_str(), "p-alice");
    assert_eq!(PersonaId::from("p-alice".to_string()), PersonaId::new("p-alice".to_string()));
    assert_eq!(Epoch::new(3).get(), 3);
    assert_eq!(Epoch::from(3), Epoch::new(3));
    assert_eq!(SequenceNumber::new(2).get(), 2);
    assert_eq!(SequenceNumber::from(2), SequenceNumber::new(2));
    assert!(Epoch::new(2) < Epoch::new(3), "epochs order as their numbers");
    assert!(SequenceNumber::new(1) < SequenceNumber::new(2));
}

/// verifies: LLR-s7whrn
#[test]
fn tag_types_accept_every_boundary_value_and_serialise_as_it() {
    for bytes in [[0u8; 32], [0xffu8; 32]] {
        assert_eq!(ChainAccount::new(bytes).as_bytes(), &bytes);
        assert_eq!(plain(&ChainAccount::new(bytes)), plain(&bytes));
    }
    for id in [String::new(), "ünïcødé-persona".to_string(), "p".repeat(1024)] {
        assert_eq!(PersonaId::new(id.clone()).as_str(), id);
        assert_eq!(plain(&PersonaId::new(id.clone())), plain(&id));
    }
    for n in [0u64, 1, u64::MAX] {
        assert_eq!(Epoch::new(n).get(), n);
        assert_eq!(SequenceNumber::new(n).get(), n);
        assert_eq!(plain(&Epoch::new(n)), plain(&n));
        assert_eq!(plain(&SequenceNumber::new(n)), plain(&n));
    }
}
