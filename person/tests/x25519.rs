//! The exported X25519 public-key validity check, and PersonPublicKey's use of it.

use curve25519_dalek::montgomery::MontgomeryPoint;
use person::{x25519, PersonPublicKey};

fn canonical(seed: u8) -> [u8; 32] {
    MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes()
}

/// verifies: LLR-m75m7u, REQ-7gz72r
#[test]
fn accepts_canonical_non_small_order_keys() {
    for seed in 1..=16 {
        assert!(x25519::is_valid_public_key(&canonical(seed)));
    }
}

/// verifies: LLR-m75m7u, REQ-7gz72r
#[test]
fn rejects_small_order_and_non_canonical_bytes() {
    let mut top_bit = canonical(3);
    top_bit[31] |= 0x80;
    let mut field_prime = [0xff; 32];
    field_prime[0] = 0xed;
    field_prime[31] = 0x7f;
    let mut one = [0u8; 32];
    one[0] = 1;
    for invalid in [[0u8; 32], one, top_bit, field_prime] {
        assert!(!x25519::is_valid_public_key(&invalid), "{invalid:02x?}");
    }
}

/// Little-endian encoding of 2^255 - 19 + delta, for |delta| <= 18.
const fn field_prime_plus(delta: i8) -> [u8; 32] {
    let mut bytes = [0xff; 32];
    bytes[31] = 0x7f;
    bytes[0] = (0xed_i16 + delta as i16) as u8;
    bytes
}

/// libsodium's full `has_small_order` list
/// (crypto_scalarmult/curve25519/ref10/x25519_ref10.c): 0, 1, the two points
/// of order 8, p - 1, p and p + 1.
const LIBSODIUM_SMALL_ORDER: [[u8; 32]; 7] = [
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

/// verifies: LLR-m75m7u, REQ-7gz72r
#[test]
fn rejects_every_libsodium_small_order_entry() {
    for u_coordinate in LIBSODIUM_SMALL_ORDER {
        assert!(
            !x25519::is_valid_public_key(&u_coordinate),
            "{u_coordinate:02x?}"
        );
    }
}

/// p - 2 is the largest value below the field prime that is not on the
/// blocklist: the canonical bound is exactly p, not lower.
/// verifies: LLR-m75m7u, REQ-7gz72r
#[test]
fn accepts_p_minus_two() {
    assert!(x25519::is_valid_public_key(&field_prime_plus(-2)));
}

/// PersonPublicKey applies exactly this rule.
/// verifies: LLR-m75m7u, REQ-3vqs9b
#[test]
fn person_public_key_accepts_exactly_what_the_check_accepts() {
    let mut cases: Vec<[u8; 32]> = (0u8..=40).map(|byte| [byte; 32]).collect();
    cases.extend((1..=16).map(canonical));
    for bytes in cases {
        assert_eq!(
            PersonPublicKey::parse(&bytes).is_ok(),
            x25519::is_valid_public_key(&bytes),
            "{bytes:02x?}"
        );
    }
}

/// The field prime modulo `modulus`, computed from its little-endian bytes.
fn field_prime_mod(modulus: u64) -> u64 {
    field_prime_plus(0)
        .iter()
        .rev()
        .fold(0, |remainder, &byte| {
            (remainder * 256 + u64::from(byte)) % modulus
        })
}

/// The Jacobi symbol (a / n) for odd n > 0, by the textbook reciprocity
/// algorithm.
fn jacobi(mut a: u64, mut n: u64) -> i8 {
    let mut symbol = 1;
    a %= n;
    while a != 0 {
        while a % 2 == 0 {
            a /= 2;
            if n % 8 == 3 || n % 8 == 5 {
                symbol = -symbol;
            }
        }
        core::mem::swap(&mut a, &mut n);
        if a % 4 == 3 && n % 4 == 3 {
            symbol = -symbol;
        }
        a %= n;
    }
    if n == 1 {
        symbol
    } else {
        0
    }
}

/// The Legendre symbol (v / p) for 0 < v < 2^40, reduced to a Jacobi symbol
/// over v's odd part: p ≡ 5 (mod 8), so (2 / p) = −1, and p ≡ 1 (mod 4), so
/// (m / p) = (p mod m / m) for odd m.
fn legendre_mod_field_prime(mut value: u64) -> i8 {
    assert_eq!(field_prime_mod(8), 5);
    let mut symbol = 1;
    while value % 2 == 0 {
        value /= 2;
        symbol = -symbol;
    }
    symbol * jacobi(field_prime_mod(value), value)
}

/// +1 when u is the u-coordinate of a point on Curve25519, −1 when it is one
/// of a point on the quadratic twist: the Legendre symbol of
/// u³ + 486662·u² + u.
fn curve_or_twist(u: u64) -> i8 {
    legendre_mod_field_prime(u * u * u + 486_662 * u * u + u)
}

fn u_coordinate(u: u8) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0] = u;
    bytes
}

/// A canonical u-coordinate of a point on the quadratic twist is accepted by
/// both the check and PersonPublicKey, as RFC 7748 and libsodium accept it
/// (owner, 2026-10-05). Twist membership is computed here, not assumed; the
/// base point u = 9 is the control that the computation tells the curve from
/// the twist.
/// verifies: LLR-vs7etb, LLR-m75m7u
#[test]
fn twist_u_coordinates_are_accepted() {
    assert_eq!(curve_or_twist(9), 1);
    for u in [2, 3, 5] {
        assert_eq!(curve_or_twist(u64::from(u)), -1, "u = {u}");
        let bytes = u_coordinate(u);
        assert!(x25519::is_valid_public_key(&bytes), "u = {u}");
        assert_eq!(
            PersonPublicKey::parse(&bytes).map(|key| *key.as_bytes()),
            Ok(bytes)
        );
    }
}

/// The u-coordinate of the base point plus each non-identity 8-torsion point:
/// a mixed-order point, canonical and off the blocklist. Both the check and
/// PersonPublicKey accept it, by the owner's decision.
/// verifies: LLR-vs7etb, LLR-m75m7u
#[test]
fn accepts_mixed_order_u_coordinates() {
    use curve25519_dalek::constants::{ED25519_BASEPOINT_POINT, EIGHT_TORSION};
    for torsion in &EIGHT_TORSION[1..] {
        let point = ED25519_BASEPOINT_POINT + torsion;
        assert!(!point.is_torsion_free());
        let u_coordinate = point.to_montgomery().to_bytes();
        assert!(u_coordinate
            .iter()
            .rev()
            .lt(field_prime_plus(0).iter().rev()));
        assert!(
            x25519::is_valid_public_key(&u_coordinate),
            "{u_coordinate:02x?}"
        );
        assert_eq!(
            PersonPublicKey::parse(&u_coordinate).map(|key| *key.as_bytes()),
            Ok(u_coordinate)
        );
    }
}
