//! DeviceSlots decoding: canonical order, the bound, huge declared lengths.

#![cfg(feature = "serde")]
// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use ed25519_dalek::SigningKey;
use person::{DevicePublicKey, DeviceSlots};

fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
        .expect("valid key")
}

fn sorted(mut devices: Vec<DevicePublicKey>) -> Vec<DevicePublicKey> {
    devices.sort();
    devices
}

/// postcard's varint encoding of the sequence length 2^32 - 1.
const HUGE_LENGTH_VARINT: [u8; 5] = [0xff, 0xff, 0xff, 0xff, 0x0f];

/// verifies: LLR-sjrh7z, REQ-tq4ms4
#[test]
fn decoding_accepts_only_strictly_increasing_sets() {
    let canonical = sorted(vec![device(1), device(2)]);
    let bytes = postcard::to_allocvec(&canonical).expect("encode");
    let slots = postcard::from_bytes::<DeviceSlots>(&bytes).expect("decode");
    assert_eq!(slots.devices(), canonical.as_slice());

    let reversed: Vec<_> = canonical.iter().rev().copied().collect();
    let bytes = postcard::to_allocvec(&reversed).expect("encode");
    assert!(postcard::from_bytes::<DeviceSlots>(&bytes).is_err());

    let duplicated = vec![device(1), device(1)];
    let bytes = postcard::to_allocvec(&duplicated).expect("encode");
    assert!(postcard::from_bytes::<DeviceSlots>(&bytes).is_err());

    let empty: Vec<DevicePublicKey> = vec![];
    let bytes = postcard::to_allocvec(&empty).expect("encode");
    assert_eq!(
        postcard::from_bytes::<DeviceSlots>(&bytes)
            .expect("decode")
            .device_count(),
        0
    );
}

/// verifies: LLR-sjrh7z, REQ-4szc22, REQ-vxx8k3
#[test]
fn decoding_rejects_a_set_over_the_bound() {
    let five = sorted(vec![device(1), device(2), device(3), device(4), device(5)]);
    let bytes = postcard::to_allocvec(&five).expect("encode");
    assert!(postcard::from_bytes::<DeviceSlots>(&bytes).is_err());
}

/// A declared length of 2^32 - 1 followed by no elements fails to decode, and
/// the decoder does not reserve capacity for the declared length (a 128 GiB
/// reservation would abort the test process). Any decoder that caps its
/// initial capacity passes this test, a plain `Vec` decoder included.
/// verifies: LLR-sjrh7z, REQ-vxx8k3
#[test]
fn decoding_a_huge_declared_length_with_no_elements_fails() {
    assert!(postcard::from_bytes::<DeviceSlots>(&HUGE_LENGTH_VARINT).is_err());
}

/// A declared length of 2^32 - 1 followed by five valid, strictly increasing
/// keys is rejected by the bound at the fifth key, not by the end of the
/// input. A decoder that reads the whole sequence before checking the bound
/// reaches the end of the input first and fails with
/// `DeserializeUnexpectedEnd` instead.
/// verifies: LLR-sjrh7z, REQ-vxx8k3
#[test]
fn decoding_a_huge_declared_length_stops_at_the_bound() {
    let five = sorted(vec![device(1), device(2), device(3), device(4), device(5)]);
    let encoded = postcard::to_allocvec(&five).expect("encode");
    // The encoding is the length varint 5 (one byte), then 5 x 32 key bytes.
    assert_eq!(encoded.len(), 1 + 5 * 32);
    let mut bytes = HUGE_LENGTH_VARINT.to_vec();
    bytes.extend_from_slice(encoded.get(1..).expect("keys follow the length"));

    // postcard maps every serde custom error, the bound's included, to
    // `SerdeDeCustom`; the keys are valid and increasing, so the only custom
    // error this input can produce is the bound's.
    assert_eq!(
        postcard::from_bytes::<DeviceSlots>(&bytes),
        Err(postcard::Error::SerdeDeCustom)
    );
}

/// verifies: LLR-sjrh7z
#[test]
fn encoding_round_trips() {
    let slots = DeviceSlots::parse(vec![device(3), device(1)]).expect("two");
    let bytes = postcard::to_allocvec(&slots).expect("encode");
    assert_eq!(
        postcard::from_bytes::<DeviceSlots>(&bytes).expect("decode"),
        slots
    );
}

/// verifies: LLR-5za6mp
#[test]
fn slots_encode_as_the_sequence_of_their_keys_in_sorted_order() {
    let slots = DeviceSlots::parse(vec![device(3), device(1), device(2)]).expect("three");
    let in_order = sorted(vec![device(1), device(2), device(3)]);
    let mut expected = vec![3u8];
    for key in &in_order {
        expected.extend_from_slice(key.as_bytes());
    }
    assert_eq!(postcard::to_allocvec(&slots).expect("encode"), expected);
}

/// A decoder asked for something other than a sequence names what it expects.
/// verifies: LLR-5za6mp, LLR-eeq89n
#[test]
fn decoding_a_non_sequence_names_the_expected_form() {
    use serde::de::{value, Deserialize, IntoDeserializer};
    let not_a_sequence: value::U32Deserializer<value::Error> = 5u32.into_deserializer();
    let error = DeviceSlots::deserialize(not_a_sequence).expect_err("not a sequence");
    assert!(
        error
            .to_string()
            .contains("at most 4 device keys in strictly increasing order"),
        "{error}"
    );
}
