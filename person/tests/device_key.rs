//! DevicePublicKey: ed25519 validation, decoding, ordering.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use ed25519_dalek::{SigningKey, VerifyingKey};
use person::{DevicePublicKey, IdentityError};

fn verifying_key(seed: u8) -> VerifyingKey {
    SigningKey::from_bytes(&[seed; 32]).verifying_key()
}

fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(verifying_key(seed)).expect("valid key")
}

/// y = 2 is not on the curve: decompression fails.
fn off_curve_bytes() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0] = 2;
    bytes
}

/// verifies: LLR-7guspr, REQ-q6xkna
#[test]
fn parse_accepts_a_valid_key_and_rejects_what_dalek_rejects() {
    let valid = *verifying_key(7).as_bytes();
    let key = DevicePublicKey::parse(&valid).expect("valid ed25519 point");
    assert_eq!(key.as_bytes(), &valid);
    assert_eq!(key, device(7));

    let off_curve = off_curve_bytes();
    assert!(VerifyingKey::from_bytes(&off_curve).is_err());
    assert_eq!(
        DevicePublicKey::parse(&off_curve),
        Err(IdentityError::InvalidDeviceKey)
    );
}

/// Little-endian encoding of 2^255 - 19 + k, a non-canonical encoding of
/// y = k, for k <= 18.
fn field_prime_plus(k: u8) -> [u8; 32] {
    let mut bytes = [0xff; 32];
    bytes[31] = 0x7f;
    bytes[0] = 0xed + k;
    bytes
}

/// y = p + 3, the smallest non-canonical encoding of a point not of small
/// order, is rejected today by the torsion-free check, not the
/// canonical-encoding check: the point has a torsion component, which the test
/// asserts. Every non-canonical encoding dalek accepts decodes to a point of
/// small order or with a torsion component, so the canonical-encoding check is
/// not separately observable; it is kept as defence in depth against a change
/// in dalek's decoding.
/// verifies: LLR-7guspr, REQ-q6xkna
#[test]
fn parse_rejects_a_non_canonical_encoding_dalek_accepts() {
    let y_at_least_p = field_prime_plus(3);
    let key = VerifyingKey::from_bytes(&y_at_least_p).expect("dalek reduces y mod p");
    assert!(!key.is_weak());
    assert!(!key.to_edwards().is_torsion_free());
    assert_ne!(key.to_edwards().compress().to_bytes(), y_at_least_p);
    assert_eq!(
        DevicePublicKey::parse(&y_at_least_p),
        Err(IdentityError::InvalidDeviceKey)
    );
}

/// The identity point has four encodings dalek accepts: y = 1 and y = p + 1,
/// each with the sign bit clear and set (x = 0 either way). None reaches a
/// DeviceSlots.
/// verifies: LLR-7guspr, REQ-q6xkna
#[cfg(feature = "serde")]
#[test]
fn no_encoding_of_the_identity_reaches_device_slots() {
    let mut canonical = [0u8; 32];
    canonical[0] = 1;
    let y_is_p_plus_one = field_prime_plus(1);
    let mut negative_zero_x = canonical;
    negative_zero_x[31] |= 0x80;
    let mut p_plus_one_negative_zero_x = y_is_p_plus_one;
    p_plus_one_negative_zero_x[31] |= 0x80;
    let encodings = [
        canonical,
        y_is_p_plus_one,
        negative_zero_x,
        p_plus_one_negative_zero_x,
    ];
    for encoding in encodings {
        assert!(
            VerifyingKey::from_bytes(&encoding).is_ok(),
            "{encoding:02x?}"
        );
        assert_eq!(
            DevicePublicKey::parse(&encoding),
            Err(IdentityError::InvalidDeviceKey),
            "{encoding:02x?}"
        );
        let mut wire = vec![1u8];
        wire.extend_from_slice(&encoding);
        assert!(postcard::from_bytes::<person::DeviceSlots>(&wire).is_err());
    }
}

/// Records, rather than assumes, that ed25519-dalek's `VerifyingKey::from_bytes` accepts a
/// small-order point, which is why DevicePublicKey checks `is_weak()` itself.
/// The identity point encodes as y = 1. If the first assertion fails, dalek
/// now rejects it: record that in the SOUP row.
/// verifies: LLR-7guspr, REQ-q6xkna
#[test]
fn a_small_order_point_dalek_accepts_is_rejected() {
    let mut identity = [0u8; 32];
    identity[0] = 1;
    let key = VerifyingKey::from_bytes(&identity).expect("dalek accepts the identity point");
    assert!(key.is_weak());
    assert_eq!(
        DevicePublicKey::parse(&identity),
        Err(IdentityError::InvalidDeviceKey)
    );
}

/// 2^255 - 19, little-endian.
const FIELD_PRIME: [u8; 32] = {
    let mut bytes = [0xff; 32];
    bytes[0] = 0xed;
    bytes[31] = 0x7f;
    bytes
};

/// True when the y-coordinate of an ed25519 encoding (the sign bit cleared)
/// is below the field prime.
fn y_is_canonical(encoding: &[u8; 32]) -> bool {
    let mut y = *encoding;
    if let Some(top) = y.last_mut() {
        *top &= 0x7f;
    }
    y.iter().rev().lt(FIELD_PRIME.iter().rev())
}

/// The eight small-order points, taken from curve25519-dalek's
/// `EIGHT_TORSION` (the i-th entry is [i]P for a point P of order 8) and
/// compressed: the identity (y = 1), the point of order 2 (y = p - 1), the
/// two of order 4 (y = 0, both signs of x) and the four of order 8. Each
/// encoding is checked canonical, dalek is checked to accept it and to call
/// it weak, and then `parse` is checked to reject it.
/// verifies: LLR-7guspr, REQ-q6xkna
#[test]
fn parse_rejects_every_small_order_point_dalek_accepts() {
    let encodings: Vec<[u8; 32]> = curve25519_dalek::constants::EIGHT_TORSION
        .iter()
        .map(|point| point.compress().to_bytes())
        .collect();

    let mut identity = [0u8; 32];
    identity[0] = 1;
    let mut order_two = FIELD_PRIME;
    order_two[0] = 0xec;
    let mut order_four_negative = [0u8; 32];
    order_four_negative[31] = 0x80;
    let [p0, _, p2, _, p4, _, p6, _] =
        <[[u8; 32]; 8]>::try_from(encodings.clone()).expect("EIGHT_TORSION has eight points");
    assert_eq!(p0, identity);
    assert_eq!(p4, order_two);
    // [2]P and [6]P are the two points with y = 0; one has the sign bit set.
    let mut order_four = [p2, p6];
    order_four.sort();
    assert_eq!(order_four, [[0u8; 32], order_four_negative]);
    let mut distinct = encodings.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(distinct.len(), 8);

    for encoding in &encodings {
        assert!(y_is_canonical(encoding), "{encoding:02x?}");
        let key = VerifyingKey::from_bytes(encoding).expect("dalek accepts a small-order point");
        assert!(key.is_weak(), "{encoding:02x?}");
        assert_eq!(key.to_edwards().compress().to_bytes(), *encoding);
        assert_eq!(
            DevicePublicKey::parse(encoding),
            Err(IdentityError::InvalidDeviceKey),
            "{encoding:02x?}"
        );
    }
}

/// The base point plus each non-identity small-order point: a mixed-order
/// point, with a torsion component, that dalek accepts and does not call
/// weak. Each encoding is canonical, so only the torsion-free check can
/// reject it.
fn mixed_order_encodings() -> Vec<[u8; 32]> {
    curve25519_dalek::constants::EIGHT_TORSION
        .iter()
        .skip(1)
        .map(|torsion| {
            (curve25519_dalek::constants::ED25519_BASEPOINT_POINT + torsion)
                .compress()
                .to_bytes()
        })
        .collect()
}

/// verifies: LLR-7guspr, REQ-q6xkna
#[test]
fn parse_rejects_a_point_with_a_torsion_component() {
    let encodings = mixed_order_encodings();
    assert_eq!(encodings.len(), 7);
    for encoding in &encodings {
        let key = VerifyingKey::from_bytes(encoding).expect("dalek accepts a mixed-order point");
        assert!(!key.is_weak(), "{encoding:02x?}");
        assert!(!key.to_edwards().is_torsion_free(), "{encoding:02x?}");
        assert_eq!(key.to_edwards().compress().to_bytes(), *encoding);
        assert_eq!(
            DevicePublicKey::parse(encoding),
            Err(IdentityError::InvalidDeviceKey),
            "{encoding:02x?}"
        );
    }
    let base = curve25519_dalek::constants::ED25519_BASEPOINT_POINT
        .compress()
        .to_bytes();
    assert!(DevicePublicKey::parse(&base).is_ok());
}

/// `TryFrom<VerifyingKey>` takes a key dalek has decoded, which may be
/// small-order or non-canonically encoded; it applies the rule parse applies.
/// verifies: LLR-7guspr, REQ-q6xkna
#[test]
fn try_from_a_verifying_key_applies_the_rule_parse_applies() {
    let mut identity = [0u8; 32];
    identity[0] = 1;
    let small_order = VerifyingKey::from_bytes(&identity).expect("dalek accepts it");
    assert_eq!(
        DevicePublicKey::try_from(small_order),
        Err(IdentityError::InvalidDeviceKey)
    );
    let non_canonical = VerifyingKey::from_bytes(&field_prime_plus(3)).expect("dalek accepts it");
    assert_eq!(
        DevicePublicKey::try_from(non_canonical),
        Err(IdentityError::InvalidDeviceKey)
    );
    let valid = verifying_key(7);
    assert_eq!(
        DevicePublicKey::try_from(valid),
        DevicePublicKey::parse(valid.as_bytes())
    );
    assert!(DevicePublicKey::try_from(valid).is_ok());
}

/// verifies: LLR-7guspr, REQ-q6xkna
#[cfg(feature = "serde")]
#[test]
fn decoding_goes_through_parse() {
    let bytes = postcard::to_allocvec(&off_curve_bytes()).expect("encode");
    assert!(postcard::from_bytes::<DevicePublicKey>(&bytes).is_err());
    let bytes = postcard::to_allocvec(&field_prime_plus(3)).expect("encode");
    assert!(postcard::from_bytes::<DevicePublicKey>(&bytes).is_err());
    let valid = device(9);
    let bytes = postcard::to_allocvec(&valid).expect("encode");
    assert_eq!(
        postcard::from_bytes::<DevicePublicKey>(&bytes).expect("decode"),
        valid
    );
}

fn hash_of<T: std::hash::Hash>(value: &T) -> u64 {
    use std::hash::{BuildHasher, BuildHasherDefault};
    BuildHasherDefault::<std::collections::hash_map::DefaultHasher>::default().hash_one(value)
}

/// verifies: LLR-78t363
#[test]
fn keys_compare_order_and_hash_by_their_bytes() {
    let keys: Vec<DevicePublicKey> = (1..=8).map(device).collect();
    for first in &keys {
        // Hashing the key is hashing its 32 bytes.
        assert_eq!(hash_of(first), hash_of(first.as_bytes()));
        for second in &keys {
            assert_eq!(first.cmp(second), first.as_bytes().cmp(second.as_bytes()));
            assert_eq!(first == second, first.as_bytes() == second.as_bytes());
        }
    }
}

/// verifies: LLR-78t363
#[test]
fn verifying_key_returns_the_decoded_key() {
    let decoded = verifying_key(6);
    let key = DevicePublicKey::parse(decoded.as_bytes()).expect("valid");
    assert_eq!(key.verifying_key(), &decoded);
    assert_eq!(key.verifying_key().as_bytes(), key.as_bytes());
}

/// verifies: LLR-a6krbh
#[test]
fn debug_is_the_type_name_and_a_hex_prefix() {
    let key = device(7);
    assert_eq!(&key.as_bytes()[..4], &[0xea, 0x4a, 0x6c, 0x63]);
    assert_eq!(format!("{key:?}"), "DevicePublicKey(ea4a6c63..)");
}

/// verifies: LLR-5za6mp
#[cfg(feature = "serde")]
#[test]
fn a_key_encodes_as_its_32_bytes() {
    let key = device(8);
    assert_eq!(
        postcard::to_allocvec(&key).expect("encode"),
        key.as_bytes().to_vec()
    );
}

/// verifies: LLR-7guspr, REQ-q6xkna
#[test]
fn try_from_goes_through_parse() {
    let valid = *verifying_key(4).as_bytes();
    let mut identity = [0u8; 32];
    identity[0] = 1;
    for bytes in [valid, identity, off_curve_bytes()] {
        assert_eq!(
            DevicePublicKey::try_from(bytes),
            DevicePublicKey::parse(&bytes)
        );
        assert_eq!(
            DevicePublicKey::try_from(&bytes),
            DevicePublicKey::parse(&bytes)
        );
    }
    assert!(DevicePublicKey::try_from(valid).is_ok());
}
