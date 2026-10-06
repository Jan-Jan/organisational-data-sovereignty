//! A DevicePublicKey that is a small-order point (among them the four points
//! of order 8 and y = p − 1), a non-canonical encoding of a point (among them
//! y = p), a mixed-order point, or bytes that decode to no point at all, is
//! refused on every path a key enters org-members by: the public constructor
//! and its `TryFrom` impls, and decoding a DevicePublicKey, a set of
//! DevicePublicKeys, a member record and a change set off the wire
//! (PR-b7khyw, LLR-z954wj).
//!
//! Every case but the off-curve one is one that
//! `ed25519_dalek::VerifyingKey::from_bytes` accepts, and `refused_keys`
//! asserts that, so the refusal shown is org-members' own, through
//! `person::DevicePublicKey::parse`.
//!
//! The constructors report `person::IdentityError::InvalidDeviceKey`, which
//! `From` maps to `OrgMembersError::InvalidDeviceKey`. A decode refusal is
//! serde's custom error carrying `person`'s message, "invalid device public
//! key". postcard, the wire format, keeps no message and reports
//! `SerdeDeCustom`; serde_json keeps it, so each decode test checks both.

#![cfg(feature = "serde")]

use ed25519_dalek::{SigningKey, VerifyingKey};
use org_members::delta::Delta;
use org_members::types::{
    DevicePublicKey, DeviceSlots, MemberId, MemberLeaf, PersonPublicKey, RootHash,
};
use org_members::OrgMembersError;
use person::IdentityError;
use postcard::{from_bytes, to_allocvec};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// The message `person` gives a refused DevicePublicKey.
const RULE: &str = "invalid device public key";

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

/// A prime-order point: a secret scalar times the base point.
fn prime_order_bytes(seed: u8) -> [u8; 32] {
    *SigningKey::from_bytes(&[seed; 32]).verifying_key().as_bytes()
}

/// The identity point, (0, 1), canonically encoded: small order.
fn small_order_identity() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    bytes
}

/// The identity encoded with x's sign bit set although x = 0.
fn identity_with_sign_bit() -> [u8; 32] {
    let mut bytes = small_order_identity();
    bytes[31] = 0x80;
    bytes
}

/// The identity encoded as y = p + 1 = 2^255 − 18, which is ≥ p.
fn identity_as_p_plus_one() -> [u8; 32] {
    let mut bytes = [0xff; 32];
    bytes[0] = 0xee;
    bytes[31] = 0x7f;
    bytes
}

/// A prime-order point plus the order-4 point (y = 0): canonically encoded,
/// not of small order, but with a torsion component.
fn mixed_order() -> [u8; 32] {
    let prime = VerifyingKey::from_bytes(&prime_order_bytes(3)).expect("prime-order point");
    let order_four = VerifyingKey::from_bytes(&[0u8; 32]).expect("y = 0 is on the curve");
    (prime.to_edwards() + order_four.to_edwards())
        .compress()
        .to_bytes()
}

/// The little-endian encoding of y = 2^255 − 19 + delta, sign bit clear.
fn y_is_field_prime_plus(delta: i8) -> [u8; 32] {
    let mut bytes = [0xff; 32];
    bytes[31] = 0x7f;
    bytes[0] = (0xed_i16 + i16::from(delta)) as u8;
    bytes
}

/// The canonical encoding of an order-8 point, checked to be one: eight times
/// it is the identity and four times it is not.
fn order_eight(encoding: &str) -> [u8; 32] {
    let bytes: [u8; 32] = hex::decode(encoding).unwrap().try_into().unwrap();
    let point = VerifyingKey::from_bytes(&bytes).expect("on the curve").to_edwards();
    assert_eq!(point.compress().to_bytes(), bytes, "canonical");
    let times_four = (point + point) + (point + point);
    let times_eight = times_four + times_four;
    assert_eq!(times_eight.compress().to_bytes(), small_order_identity());
    assert_ne!(times_four.compress().to_bytes(), small_order_identity());
    bytes
}

/// y = 2: no x satisfies the curve equation, so dalek decodes nothing.
fn off_curve() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0] = 2;
    assert!(VerifyingKey::from_bytes(&bytes).is_err(), "y = 2 is off the curve");
    bytes
}

/// Every refused case dalek decodes, named.
fn refused_keys() -> Vec<(&'static str, [u8; 32])> {
    let cases = vec![
        ("small-order identity", small_order_identity()),
        ("identity with sign bit set", identity_with_sign_bit()),
        ("identity as y = p + 1", identity_as_p_plus_one()),
        ("mixed-order point", mixed_order()),
        (
            "order-8 point c717..037a",
            order_eight("c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac037a"),
        ),
        (
            "order-8 point c717..03fa",
            order_eight("c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac03fa"),
        ),
        (
            "order-8 point 26e8..fc05",
            order_eight("26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc05"),
        ),
        (
            "order-8 point 26e8..fc85",
            order_eight("26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc85"),
        ),
        ("y = p − 1, the order-2 point", y_is_field_prime_plus(-1)),
        ("y = p, a non-canonical y = 0", y_is_field_prime_plus(0)),
    ];
    for (case, bytes) in &cases {
        assert!(VerifyingKey::from_bytes(bytes).is_ok(), "dalek decodes the {case}");
    }
    cases
}

/// Every refused case: those dalek decodes, and an off-curve encoding.
fn every_refused_key() -> Vec<(&'static str, [u8; 32])> {
    let mut cases = refused_keys();
    cases.push(("off-curve y = 2", off_curve()));
    cases
}

/// A member key valid as X25519, from the same scalar-times-base construction
/// as the device keys.
fn member_key() -> PersonPublicKey {
    (0u8..)
        .find_map(|seed| PersonPublicKey::parse(&prime_order_bytes(seed)).ok())
        .expect("about half of all seeds qualify")
}

/// The member record's wire shape with its device keys as raw bytes, so a
/// payload can carry bytes `DevicePublicKey` would refuse.
#[derive(Serialize)]
struct WireLeaf<'a> {
    id: MemberId,
    handle: &'a str,
    p2p_key: PersonPublicKey,
    name: &'a str,
    surname: &'a str,
    p2p_devices: Vec<[u8; 32]>,
}

fn wire_leaf(device: [u8; 32]) -> WireLeaf<'static> {
    WireLeaf {
        id: MemberId::new([7; 32]),
        handle: "alice",
        p2p_key: member_key(),
        name: "Alice",
        surname: "Smith",
        p2p_devices: vec![device],
    }
}

/// The change set's wire shape over `WireLeaf`.
#[derive(Serialize)]
struct WireDelta<'a> {
    base_root: RootHash,
    removed: Vec<MemberId>,
    upserted: Vec<WireLeaf<'a>>,
}

// verifies: LLR-z954wj
#[test]
fn the_public_constructor_refuses_each_invalid_device_key() {
    assert!(DevicePublicKey::parse(&prime_order_bytes(1)).is_ok());
    let refused = Err(IdentityError::InvalidDeviceKey);
    for (case, bytes) in every_refused_key() {
        assert_eq!(DevicePublicKey::parse(&bytes), refused, "{case}");
        assert_eq!(DevicePublicKey::try_from(bytes), refused, "{case}");
        assert_eq!(DevicePublicKey::try_from(&bytes), refused, "{case}");
        assert_eq!(
            DevicePublicKey::parse(&bytes).map_err(OrgMembersError::from),
            Err(OrgMembersError::InvalidDeviceKey),
            "{case}"
        );
    }
    for (case, bytes) in refused_keys() {
        let decoded_by_dalek = VerifyingKey::from_bytes(&bytes).expect(case);
        assert_eq!(DevicePublicKey::try_from(decoded_by_dalek), refused, "{case}");
    }
}

// verifies: LLR-z954wj
#[test]
fn decoding_a_device_key_refuses_each_invalid_device_key() {
    let valid = to_allocvec(&prime_order_bytes(1)).unwrap();
    assert!(from_bytes::<DevicePublicKey>(&valid).is_ok());
    for (case, bytes) in every_refused_key() {
        assert_refused_for_the_key::<DevicePublicKey>(&bytes, case);
    }
}

// verifies: LLR-z954wj
#[test]
fn decoding_a_device_key_set_refuses_each_invalid_device_key() {
    let valid = to_allocvec(&vec![prime_order_bytes(1)]).unwrap();
    assert!(from_bytes::<DeviceSlots>(&valid).is_ok());
    for (case, bytes) in every_refused_key() {
        assert_refused_for_the_key::<DeviceSlots>(&vec![bytes], case);
    }
}

// verifies: LLR-z954wj
#[test]
fn decoding_a_member_record_refuses_each_invalid_device_key() {
    let valid = to_allocvec(&wire_leaf(prime_order_bytes(1))).unwrap();
    assert!(from_bytes::<MemberLeaf>(&valid).is_ok());
    for (case, bytes) in every_refused_key() {
        assert_refused_for_the_key::<MemberLeaf>(&wire_leaf(bytes), case);
    }
}

// verifies: LLR-z954wj
#[test]
fn decoding_a_change_set_refuses_each_invalid_device_key() {
    let delta_with = |device: [u8; 32]| WireDelta {
        base_root: RootHash::new([0; 32]),
        removed: Vec::new(),
        upserted: vec![wire_leaf(device)],
    };
    let valid = to_allocvec(&delta_with(prime_order_bytes(1))).unwrap();
    assert!(from_bytes::<Delta>(&valid).is_ok());
    for (case, bytes) in every_refused_key() {
        assert_refused_for_the_key::<Delta>(&delta_with(bytes), case);
    }
}
