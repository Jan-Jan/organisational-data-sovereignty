//! PersonPublicKey: a canonical, non-small-order X25519 public key.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use curve25519_dalek::constants::{ED25519_BASEPOINT_POINT, X25519_BASEPOINT};
use curve25519_dalek::montgomery::MontgomeryPoint;
use curve25519_dalek::scalar::Scalar;
use person::{IdentityError, PersonPublicKey};

/// The small-order X25519 u-coordinates libsodium rejects
/// (crypto_scalarmult/curve25519/ref10/x25519_ref10.c, `has_small_order`).
const SMALL_ORDER: [[u8; 32]; 7] = [
    [0; 32],
    {
        let mut bytes = [0; 32];
        bytes[0] = 1;
        bytes
    },
    [
        0xe0, 0xeb, 0x7a, 0x7c, 0x3b, 0x41, 0xb8, 0xae, 0x16, 0x56, 0xe3, 0xfa, 0xf1, 0x9f, 0xc4,
        0x6a, 0xda, 0x09, 0x8d, 0xeb, 0x9c, 0x32, 0xb1, 0xfd, 0x86, 0x62, 0x05, 0x16, 0x5f, 0x49,
        0xb8, 0x00,
    ],
    [
        0x5f, 0x9c, 0x95, 0xbc, 0xa3, 0x50, 0x8c, 0x24, 0xb1, 0xd0, 0xb1, 0x55, 0x9c, 0x83, 0xef,
        0x5b, 0x04, 0x44, 0x5c, 0xc4, 0x58, 0x1c, 0x8e, 0x86, 0xd8, 0x22, 0x4e, 0xdd, 0xd0, 0x9f,
        0x11, 0x57,
    ],
    field_prime_plus(-1),
    field_prime_plus(0),
    field_prime_plus(1),
];

/// Little-endian encoding of 2^255 - 19 + delta, for |delta| <= 18.
const fn field_prime_plus(delta: i8) -> [u8; 32] {
    let mut bytes = [0xff; 32];
    bytes[31] = 0x7f;
    bytes[0] = (0xed_i16 + delta as i16) as u8;
    bytes
}

fn x25519_public(seed: u8) -> [u8; 32] {
    MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes()
}

/// verifies: LLR-vs7etb, REQ-3vqs9b
#[test]
fn accepts_a_canonical_x25519_public_key_and_contains_its_bytes() {
    for seed in 1..=16 {
        let bytes = x25519_public(seed);
        let key = PersonPublicKey::parse(&bytes).expect("valid X25519 key");
        assert_eq!(key.as_bytes(), &bytes);
    }
}

/// p - 2 is the largest value below the field prime that is not on the
/// blocklist: the canonical bound is exactly p, not lower.
/// verifies: LLR-vs7etb, REQ-3vqs9b
#[test]
fn accepts_p_minus_two() {
    let bytes = field_prime_plus(-2);
    let key = PersonPublicKey::parse(&bytes).expect("canonical, not small order");
    assert_eq!(key.as_bytes(), &bytes);
}

/// verifies: LLR-vs7etb, REQ-3vqs9b
#[test]
fn rejects_every_small_order_u_coordinate() {
    for u_coordinate in SMALL_ORDER {
        assert_eq!(
            PersonPublicKey::parse(&u_coordinate),
            Err(IdentityError::InvalidPersonKey),
            "{u_coordinate:02x?}"
        );
    }
}

/// A fixture check, verifying no requirement: it checks this file's own
/// constant, not the library. The list above is checked, not trusted: each
/// canonical entry (0, 1, both order-8 points, p - 1) is confirmed of small
/// order on its own by the x-only Montgomery ladder of curve25519-dalek 4.1
/// (`MontgomeryPoint * Scalar`): multiplying by 8 reaches the point at
/// infinity, whose affine u-coordinate the ladder returns as all zeroes. The
/// ladder is first checked to be scalar multiplication, against the Edwards
/// base point, and a normal key is checked not to reach zero.
#[test]
fn the_small_order_list_is_what_it_claims() {
    let eight = Scalar::from(8u8);
    assert_eq!(
        X25519_BASEPOINT * eight,
        (ED25519_BASEPOINT_POINT * eight).to_montgomery()
    );
    for seed in 1..=16 {
        let normal = MontgomeryPoint(x25519_public(seed));
        assert_ne!((normal * eight).to_bytes(), [0; 32], "seed {seed}");
    }
    for u_coordinate in &SMALL_ORDER[..5] {
        let point = MontgomeryPoint(*u_coordinate);
        assert_eq!((point * eight).to_bytes(), [0; 32], "{u_coordinate:02x?}");
    }
}

/// verifies: LLR-vs7etb, REQ-3vqs9b
#[test]
fn rejects_non_canonical_encodings() {
    let mut top_bit = x25519_public(5);
    top_bit[31] |= 0x80;
    assert_eq!(
        PersonPublicKey::parse(&top_bit),
        Err(IdentityError::InvalidPersonKey)
    );
    for delta in 2..=18_i8 {
        assert_eq!(
            PersonPublicKey::parse(&field_prime_plus(delta)),
            Err(IdentityError::InvalidPersonKey),
            "p + {delta}"
        );
    }
}

/// verifies: LLR-vs7etb, REQ-3vqs9b
#[cfg(feature = "serde")]
#[test]
fn decoding_goes_through_parse() {
    let bytes = postcard::to_allocvec(&SMALL_ORDER[2]).expect("encode");
    assert!(postcard::from_bytes::<PersonPublicKey>(&bytes).is_err());
    let valid = PersonPublicKey::parse(&x25519_public(9)).expect("valid");
    let bytes = postcard::to_allocvec(&valid).expect("encode");
    assert_eq!(
        postcard::from_bytes::<PersonPublicKey>(&bytes).expect("decode"),
        valid
    );
}

/// verifies: LLR-vs7etb, REQ-3vqs9b
#[test]
fn try_from_goes_through_parse() {
    let valid = x25519_public(4);
    for bytes in [valid, SMALL_ORDER[1], field_prime_plus(3)] {
        assert_eq!(
            PersonPublicKey::try_from(bytes),
            PersonPublicKey::parse(&bytes)
        );
        assert_eq!(
            PersonPublicKey::try_from(&bytes),
            PersonPublicKey::parse(&bytes)
        );
    }
    assert!(PersonPublicKey::try_from(valid).is_ok());
}

fn hash_of<T: std::hash::Hash>(value: &T) -> u64 {
    use std::hash::{BuildHasher, BuildHasherDefault};
    BuildHasherDefault::<std::collections::hash_map::DefaultHasher>::default().hash_one(value)
}

/// verifies: LLR-78t363
#[test]
fn keys_compare_order_and_hash_by_their_bytes() {
    let keys: Vec<PersonPublicKey> = (1..=8)
        .map(|seed| PersonPublicKey::parse(&x25519_public(seed)).expect("valid"))
        .collect();
    for first in &keys {
        // Hashing the key is hashing its 32 bytes.
        assert_eq!(hash_of(first), hash_of(first.as_bytes()));
        for second in &keys {
            assert_eq!(first.cmp(second), first.as_bytes().cmp(second.as_bytes()));
            assert_eq!(first == second, first.as_bytes() == second.as_bytes());
            assert_eq!(
                hash_of(first) == hash_of(second),
                first.as_bytes() == second.as_bytes()
            );
        }
    }
}

/// verifies: LLR-a6krbh
#[test]
fn debug_is_the_type_name_and_a_hex_prefix() {
    // The u-coordinate of the X25519 base point, 9.
    let mut base_point = [0u8; 32];
    base_point[0] = 9;
    let key = PersonPublicKey::parse(&base_point).expect("valid");
    assert_eq!(format!("{key:?}"), "PersonPublicKey(09000000..)");
}

/// verifies: LLR-5za6mp
#[cfg(feature = "serde")]
#[test]
fn a_key_encodes_as_its_32_bytes() {
    let key = PersonPublicKey::parse(&x25519_public(8)).expect("valid");
    assert_eq!(
        postcard::to_allocvec(&key).expect("encode"),
        key.as_bytes().to_vec()
    );
}
