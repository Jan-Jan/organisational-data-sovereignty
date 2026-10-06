//! Decoding a Person: each field through its own decoder, then Person::new.
//! serde_json keeps a custom error's message, so the message tests decode
//! JSON; postcard, which maps every custom error to `SerdeDeCustom`, checks
//! the binary route.

#![cfg(feature = "serde")]
// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

mod common;

use common::{device, device_and_its_bytes_as_a_person_key, person_key};
use person::definition::Person;
use person::{DeviceSlots, IdentityError, Name, Surname};
use serde_json::{json, Value};

fn sorted_device_bytes(seeds: &[u8]) -> Vec<[u8; 32]> {
    let mut keys: Vec<[u8; 32]> = seeds.iter().map(|seed| *device(*seed).as_bytes()).collect();
    keys.sort();
    keys
}

/// A JSON Person with these fields, keys as arrays of 32 numbers.
fn person_json(name: &str, key: Option<[u8; 32]>, devices: &[[u8; 32]]) -> Value {
    json!({
        "name": name,
        "surname": "Example",
        "person_key": key,
        "devices": devices,
    })
}

fn decode_error(value: Value) -> String {
    serde_json::from_value::<Person>(value)
        .expect_err("refused")
        .to_string()
}

fn alice() -> Person {
    common::person("Alice", "Example", Some(9), &[2, 1])
}

/// verifies: LLR-63pkfc
#[test]
fn a_person_round_trips_through_both_formats() {
    let person = alice();
    let bytes = postcard::to_allocvec(&person).expect("encode");
    assert_eq!(
        postcard::from_bytes::<Person>(&bytes).expect("decode"),
        person
    );
    let text = serde_json::to_string(&person).expect("encode");
    assert_eq!(
        serde_json::from_str::<Person>(&text).expect("decode"),
        person
    );
    let revoked = common::person("Alice", "Example", None, &[]);
    let bytes = postcard::to_allocvec(&revoked).expect("encode");
    assert_eq!(
        postcard::from_bytes::<Person>(&bytes).expect("decode"),
        revoked
    );
}

/// The postcard form is the four fields in order: name, surname, the key as
/// an option, the slots as a sequence.
/// verifies: LLR-63pkfc
#[test]
fn the_binary_form_is_the_four_fields_in_order() {
    let person = alice();
    let mut expected = postcard::to_allocvec(person.name()).expect("name");
    expected.extend(postcard::to_allocvec(person.surname()).expect("surname"));
    expected.extend(postcard::to_allocvec(&person.person_key()).expect("key"));
    expected.extend(postcard::to_allocvec(person.devices()).expect("devices"));
    assert_eq!(postcard::to_allocvec(&person).expect("encode"), expected);
}

/// A decoded name is stored in NFC, as `Name::parse` stores it.
/// verifies: LLR-63pkfc
#[test]
fn a_decoded_name_is_normalised() {
    let value = person_json(
        "Jose\u{0301}",
        Some(*person_key(9).as_bytes()),
        &sorted_device_bytes(&[1]),
    );
    let person = serde_json::from_value::<Person>(value).expect("valid");
    assert_eq!(person.name().as_str(), "Jos\u{00e9}");
}

/// Each Person::new rule refuses through the decoder, with its message.
/// verifies: LLR-63pkfc, LLR-sjkmr6, LLR-kbhc43, LLR-3n3kxx
#[test]
fn a_definition_breaking_a_person_rule_is_refused_with_its_message() {
    let key = *person_key(9).as_bytes();
    assert_eq!(
        decode_error(person_json("Alice", Some(key), &[])),
        IdentityError::KeyWithoutDevice.to_string()
    );
    assert_eq!(
        decode_error(person_json("Alice", None, &sorted_device_bytes(&[1]))),
        IdentityError::MissingPersonKey.to_string()
    );
    let copied = *device_and_its_bytes_as_a_person_key().0.as_bytes();
    assert_eq!(
        decode_error(person_json("Alice", Some(copied), &[copied])),
        IdentityError::PersonKeyIsDeviceKey.to_string()
    );
}

/// Each field's own rule refuses through that field's decoder, with its message.
/// verifies: LLR-63pkfc, LLR-4ku56h
#[test]
fn a_field_breaking_its_own_rule_is_refused_with_its_message() {
    let key = Some(*person_key(9).as_bytes());
    let one = sorted_device_bytes(&[1]);
    assert_eq!(
        decode_error(person_json(&"a".repeat(129), key, &one)),
        "field too long: name exceeds 128 bytes after NFC normalization"
    );
    let mut identity = [0u8; 32];
    identity[0] = 1;
    assert_eq!(
        decode_error(person_json("Alice", key, &[identity])),
        "invalid device public key"
    );
    assert_eq!(
        decode_error(person_json("Alice", Some([0u8; 32]), &one)),
        "invalid person public key"
    );
    assert_eq!(
        decode_error(person_json(
            "Alice",
            key,
            &sorted_device_bytes(&[1, 2, 3, 4, 5])
        )),
        "device slots exceed MAX_DEVICES"
    );
    let mut reversed = sorted_device_bytes(&[1, 2]);
    reversed.reverse();
    assert_eq!(
        decode_error(person_json("Alice", key, &reversed)),
        "device slots must be strictly increasing (sorted, no duplicates)"
    );
}

/// verifies: LLR-63pkfc
#[test]
fn a_missing_or_unknown_field_is_refused() {
    let mut missing = person_json("Alice", None, &[]);
    missing
        .as_object_mut()
        .expect("an object")
        .remove("devices");
    assert!(serde_json::from_value::<Person>(missing).is_err());
    let mut extra = person_json("Alice", None, &[]);
    extra
        .as_object_mut()
        .expect("an object")
        .insert("epoch".into(), json!(1));
    assert!(serde_json::from_value::<Person>(extra).is_err());
}

/// postcard bytes whose every field decodes, but which break a Person rule,
/// are refused: the only custom error such bytes can produce is Person::new's.
/// verifies: LLR-63pkfc, LLR-sjkmr6
#[test]
fn binary_bytes_breaking_a_person_rule_are_refused() {
    let mut bytes = postcard::to_allocvec(&Name::parse("Alice").expect("name")).expect("name");
    bytes.extend(
        postcard::to_allocvec(&Surname::parse("Example").expect("surname")).expect("surname"),
    );
    bytes.extend(postcard::to_allocvec(&Some(person_key(9))).expect("key"));
    bytes.extend(
        postcard::to_allocvec(&DeviceSlots::parse(vec![]).expect("empty")).expect("devices"),
    );
    assert_eq!(
        postcard::from_bytes::<Person>(&bytes),
        Err(postcard::Error::SerdeDeCustom)
    );
}

/// Encoding into a buffer too short for it fails, whichever field the buffer
/// runs out in (JSON's opening brace included), and never writes past it.
/// verifies: LLR-63pkfc
#[test]
fn encoding_into_a_short_buffer_fails_at_every_field() {
    let person = alice();
    let text = serde_json::to_vec(&person).expect("encode");
    for length in 0..text.len() {
        let mut buffer = vec![0u8; length];
        let mut writer: &mut [u8] = &mut buffer;
        assert!(
            serde_json::to_writer(&mut writer, &person).is_err(),
            "{length} of {} bytes",
            text.len()
        );
    }
    let full = postcard::to_allocvec(&person).expect("encode");
    for length in 0..full.len() {
        let mut buffer = vec![0u8; length];
        assert!(
            postcard::to_slice(&person, &mut buffer).is_err(),
            "{length} of {} bytes",
            full.len()
        );
    }
    let mut buffer = vec![0u8; full.len()];
    assert_eq!(
        postcard::to_slice(&person, &mut buffer)
            .expect("fits")
            .to_vec(),
        full
    );
}
