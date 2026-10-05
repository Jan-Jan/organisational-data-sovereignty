//! Decoding rejects through serde's error type with a message that names the
//! rule. postcard maps every custom error to the opaque `SerdeDeCustom`, so
//! these tests decode through serde's own value deserializers, whose error
//! keeps the message.

#![cfg(feature = "serde")]
// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use ed25519_dalek::SigningKey;
use person::{DevicePublicKey, DeviceSlots, Name, PersonPublicKey, Surname};
use serde::de::value::{Error, SeqDeserializer, StrDeserializer};
use serde::de::{Deserialize, IntoDeserializer};

fn bytes_deserializer(bytes: [u8; 32]) -> SeqDeserializer<std::array::IntoIter<u8, 32>, Error> {
    SeqDeserializer::new(bytes.into_iter())
}

fn slots_deserializer(keys: &[[u8; 32]]) -> SeqDeserializer<std::vec::IntoIter<Vec<u8>>, Error> {
    let keys: Vec<Vec<u8>> = keys.iter().map(|key| key.to_vec()).collect();
    SeqDeserializer::new(keys.into_iter())
}

fn device_bytes(seed: u8) -> [u8; 32] {
    *SigningKey::from_bytes(&[seed; 32])
        .verifying_key()
        .as_bytes()
}

fn sorted_device_bytes(seeds: &[u8]) -> Vec<[u8; 32]> {
    let mut keys: Vec<[u8; 32]> = seeds.iter().map(|seed| device_bytes(*seed)).collect();
    keys.sort();
    keys
}

/// verifies: LLR-eeq89n, LLR-7guspr, LLR-vs7etb
#[test]
fn a_rejected_key_names_its_rule() {
    let mut identity = [0u8; 32];
    identity[0] = 1;
    let error =
        DevicePublicKey::deserialize(bytes_deserializer(identity)).expect_err("small order");
    assert_eq!(error.to_string(), "invalid device public key");
    assert!(DevicePublicKey::deserialize(bytes_deserializer(device_bytes(3))).is_ok());

    let error =
        PersonPublicKey::deserialize(bytes_deserializer([0u8; 32])).expect_err("small order");
    assert_eq!(error.to_string(), "invalid person public key");
}

/// verifies: LLR-eeq89n, LLR-ayu93n
#[test]
fn a_rejected_name_names_the_field_and_the_bound() {
    let over = "a".repeat(129);
    let deserializer: StrDeserializer<Error> = over.as_str().into_deserializer();
    let error = Name::deserialize(deserializer).expect_err("over the bound");
    assert_eq!(
        error.to_string(),
        "field too long: name exceeds 128 bytes after NFC normalization"
    );
    let deserializer: StrDeserializer<Error> = over.as_str().into_deserializer();
    let error = Surname::deserialize(deserializer).expect_err("over the bound");
    assert_eq!(
        error.to_string(),
        "field too long: surname exceeds 128 bytes after NFC normalization"
    );
}

/// verifies: LLR-eeq89n, LLR-sjrh7z
#[test]
fn rejected_slots_name_the_bound_or_the_order() {
    let five = sorted_device_bytes(&[1, 2, 3, 4, 5]);
    let error = DeviceSlots::deserialize(slots_deserializer(&five)).expect_err("over the bound");
    assert_eq!(error.to_string(), "device slots exceed MAX_DEVICES");

    let mut reversed = sorted_device_bytes(&[1, 2]);
    reversed.reverse();
    let error =
        DeviceSlots::deserialize(slots_deserializer(&reversed)).expect_err("not increasing");
    assert_eq!(
        error.to_string(),
        "device slots must be strictly increasing (sorted, no duplicates)"
    );

    let duplicated = [device_bytes(1), device_bytes(1)];
    let error = DeviceSlots::deserialize(slots_deserializer(&duplicated)).expect_err("duplicate");
    assert_eq!(
        error.to_string(),
        "device slots must be strictly increasing (sorted, no duplicates)"
    );

    let mut identity = [0u8; 32];
    identity[0] = 1;
    let error = DeviceSlots::deserialize(slots_deserializer(&[identity])).expect_err("bad key");
    assert_eq!(error.to_string(), "invalid device public key");

    let four = sorted_device_bytes(&[1, 2, 3, 4]);
    let slots = DeviceSlots::deserialize(slots_deserializer(&four)).expect("within the bound");
    assert_eq!(slots.device_count(), 4);
}
