//! A Member-as-a-group key that is a small-order X25519 u-coordinate or a
//! non-canonical one (≥ 2^255 − 19) is refused on every path a key enters
//! org-members by: the public constructor, and decoding a member key, a member
//! record and a change set off the wire. A canonical u-coordinate of a point on
//! the quadratic twist, or of a mixed-order point, is accepted (owner,
//! 2026-10-05).
//!
//! The constructors report `person::IdentityError::InvalidPersonKey`, which
//! `From` maps to `OrgMembersError::InvalidPersonKey`. A decode refusal is
//! serde's custom error carrying `person`'s message, "invalid person public
//! key". postcard, the wire format, keeps no message and reports
//! `SerdeDeCustom`; serde_json keeps it, so each decode test checks both.
//!
//! An X25519 key has no off-curve case: every u-coordinate below p is on the
//! curve or on its quadratic twist, and the twist is accepted.
//!
//! These are characterisation tests of a behaviour org-members inherits from
//! `person::PersonPublicKey::parse`: they pin that org-members reaches the
//! rule on each of its own entry points.

#![cfg(feature = "serde")]

use ed25519_dalek::{SigningKey, VerifyingKey};
use org_members::delta::Delta;
use org_members::types::{DevicePublicKey, MemberId, MemberLeaf, PersonPublicKey, RootHash};
use org_members::OrgMembersError;
use person::IdentityError;
use postcard::{from_bytes, to_allocvec};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// The message `person` gives a refused Member-as-a-group key.
const RULE: &str = "invalid person public key";

/// Decoding `wire` as `T` is refused for the key rule: postcard reports
/// serde's custom error, and serde_json, which keeps the message, names the
/// rule.
fn assert_refused_for_the_key<T: DeserializeOwned + Debug>(wire: &impl Serialize, case: &str) {
    let postcard_error = from_bytes::<T>(&to_allocvec(wire).unwrap()).expect_err(case);
    assert_eq!(postcard_error, postcard::Error::SerdeDeCustom, "{case}");
    let json_error =
        serde_json::from_str::<T>(&serde_json::to_string(wire).unwrap()).expect_err(case);
    assert!(json_error.is_data(), "{case}: {json_error}");
    assert!(json_error.to_string().starts_with(RULE), "{case}: {json_error}");
}

/// The little-endian encoding of a small whole number.
fn u_coordinate(value: u8) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0] = value;
    bytes
}

/// The little-endian encoding of 2^255 − 19 + delta, for |delta| ≤ 18.
fn field_prime_plus(delta: i8) -> [u8; 32] {
    let mut bytes = [0xff; 32];
    bytes[31] = 0x7f;
    bytes[0] = (0xed_i16 + i16::from(delta)) as u8;
    bytes
}

/// The u-coordinate of a mixed-order point: a prime-order point plus the
/// order-4 point (Edwards y = 0). Canonical and not of small order, with a
/// torsion component.
fn mixed_order_u() -> [u8; 32] {
    let prime = SigningKey::from_bytes(&[3; 32]).verifying_key().to_edwards();
    let order_four = VerifyingKey::from_bytes(&[0u8; 32]).expect("y = 0 is on the curve").to_edwards();
    let mixed = prime + order_four;
    assert!(prime.is_torsion_free());
    assert!(!mixed.is_torsion_free() && !mixed.is_small_order());
    mixed.to_montgomery().to_bytes()
}

/// Each accepted case, named: the X25519 base point (u = 9), a point on the
/// quadratic twist (u = 2, computed to be on the twist by `person`'s
/// `twist_u_coordinates_are_accepted`) and a mixed-order point.
fn accepted_keys() -> Vec<(&'static str, [u8; 32])> {
    vec![
        ("base point, u = 9", u_coordinate(9)),
        ("twist point, u = 2", u_coordinate(2)),
        ("mixed-order point", mixed_order_u()),
    ]
}

/// Each refused case, named: libsodium's `has_small_order` list (0, 1, the two
/// points of order 8, p − 1, p, p + 1), the largest 255-bit value, and u = 9
/// with bit 255 set (at least 2^255, so not canonical; RFC 7748 would mask the
/// bit, `person` refuses it).
fn refused_keys() -> Vec<(&'static str, [u8; 32])> {
    vec![
        ("all-zero u", u_coordinate(0)),
        ("u = 1", u_coordinate(1)),
        (
            "order-8 point e0eb..",
            [
                0xe0, 0xeb, 0x7a, 0x7c, 0x3b, 0x41, 0xb8, 0xae, 0x16, 0x56, 0xe3, 0xfa, 0xf1, 0x9f,
                0xc4, 0x6a, 0xda, 0x09, 0x8d, 0xeb, 0x9c, 0x32, 0xb1, 0xfd, 0x86, 0x62, 0x05, 0x16,
                0x5f, 0x49, 0xb8, 0x00,
            ],
        ),
        (
            "order-8 point 5f9c..",
            [
                0x5f, 0x9c, 0x95, 0xbc, 0xa3, 0x50, 0x8c, 0x24, 0xb1, 0xd0, 0xb1, 0x55, 0x9c, 0x83,
                0xef, 0x5b, 0x04, 0x44, 0x5c, 0xc4, 0x58, 0x1c, 0x8e, 0x86, 0xd8, 0x22, 0x4e, 0xdd,
                0xd0, 0x9f, 0x11, 0x57,
            ],
        ),
        ("p − 1", field_prime_plus(-1)),
        ("p", field_prime_plus(0)),
        ("p + 1", field_prime_plus(1)),
        ("2^255 − 1", {
            let mut bytes = [0xff; 32];
            bytes[31] = 0x7f;
            bytes
        }),
        ("u = 9 with bit 255 set", {
            let mut bytes = u_coordinate(9);
            bytes[31] = 0x80;
            bytes
        }),
    ]
}

/// A valid device key, so the member record differs only in its member key.
fn device_key() -> DevicePublicKey {
    let bytes = *SigningKey::from_bytes(&[1; 32]).verifying_key().as_bytes();
    DevicePublicKey::parse(&bytes).expect("a prime-order point")
}

/// The member record's wire shape with its member key as raw bytes, so a
/// payload can carry bytes `PersonPublicKey` would refuse.
#[derive(Serialize)]
struct WireLeaf<'a> {
    id: MemberId,
    handle: &'a str,
    p2p_key: [u8; 32],
    name: &'a str,
    surname: &'a str,
    p2p_devices: Vec<DevicePublicKey>,
}

fn wire_leaf(member_key: [u8; 32]) -> WireLeaf<'static> {
    WireLeaf {
        id: MemberId::new([7; 32]),
        handle: "alice",
        p2p_key: member_key,
        name: "Alice",
        surname: "Smith",
        p2p_devices: vec![device_key()],
    }
}

/// The change set's wire shape over `WireLeaf`.
#[derive(Serialize)]
struct WireDelta<'a> {
    base_root: RootHash,
    removed: Vec<MemberId>,
    upserted: Vec<WireLeaf<'a>>,
}

fn wire_delta(member_key: [u8; 32]) -> WireDelta<'static> {
    WireDelta {
        base_root: RootHash::new([0; 32]),
        removed: Vec::new(),
        upserted: vec![wire_leaf(member_key)],
    }
}

// verifies: LLR-a645bx
#[test]
fn the_public_constructor_accepts_a_valid_a_twist_and_a_mixed_order_member_key() {
    for (case, bytes) in accepted_keys() {
        let key = PersonPublicKey::parse(&bytes).expect(case);
        assert_eq!(key.as_bytes(), &bytes, "{case}");
        assert_eq!(PersonPublicKey::try_from(bytes), Ok(key), "{case}");
        assert_eq!(PersonPublicKey::try_from(&bytes), Ok(key), "{case}");
    }
}

// verifies: LLR-a645bx
#[test]
fn the_public_constructor_refuses_each_invalid_member_key() {
    let refused = Err(IdentityError::InvalidPersonKey);
    for (case, bytes) in refused_keys() {
        assert_eq!(PersonPublicKey::parse(&bytes), refused, "{case}");
        assert_eq!(PersonPublicKey::try_from(bytes), refused, "{case}");
        assert_eq!(PersonPublicKey::try_from(&bytes), refused, "{case}");
        assert_eq!(
            PersonPublicKey::parse(&bytes).map_err(OrgMembersError::from),
            Err(OrgMembersError::InvalidPersonKey),
            "{case}"
        );
    }
}

// verifies: LLR-a645bx
#[test]
fn decoding_a_member_key_accepts_a_valid_a_twist_and_a_mixed_order_member_key() {
    for (case, bytes) in accepted_keys() {
        let wire = to_allocvec(&bytes).unwrap();
        let key = from_bytes::<PersonPublicKey>(&wire).expect(case);
        assert_eq!(key.as_bytes(), &bytes, "{case}");
    }
}

// verifies: LLR-a645bx
#[test]
fn decoding_a_member_key_refuses_each_invalid_member_key() {
    for (case, bytes) in refused_keys() {
        assert_refused_for_the_key::<PersonPublicKey>(&bytes, case);
    }
}

// verifies: LLR-a645bx
#[test]
fn decoding_a_member_record_accepts_a_valid_a_twist_and_a_mixed_order_member_key() {
    for (case, bytes) in accepted_keys() {
        let wire = to_allocvec(&wire_leaf(bytes)).unwrap();
        let leaf = from_bytes::<MemberLeaf>(&wire).expect(case);
        assert_eq!(leaf.p2p_key().as_bytes(), &bytes, "{case}");
    }
}

// verifies: LLR-a645bx
#[test]
fn decoding_a_member_record_refuses_each_invalid_member_key() {
    for (case, bytes) in refused_keys() {
        assert_refused_for_the_key::<MemberLeaf>(&wire_leaf(bytes), case);
    }
}

// verifies: LLR-a645bx
#[test]
fn decoding_a_change_set_accepts_a_valid_a_twist_and_a_mixed_order_member_key() {
    for (case, bytes) in accepted_keys() {
        let wire = to_allocvec(&wire_delta(bytes)).unwrap();
        let delta = from_bytes::<Delta>(&wire).expect(case);
        assert_eq!(delta.upserted()[0].p2p_key().as_bytes(), &bytes, "{case}");
    }
}

// verifies: LLR-a645bx
#[test]
fn decoding_a_change_set_refuses_each_invalid_member_key() {
    for (case, bytes) in refused_keys() {
        assert_refused_for_the_key::<Delta>(&wire_delta(bytes), case);
    }
}
