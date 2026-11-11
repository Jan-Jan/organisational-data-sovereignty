//! Fuzz target: decoding arbitrary bytes as a Person, and every Person
//! operation on what decodes, must never panic; every decoded Person satisfies
//! the Person rules.
//!
//! Two shapes per input. Shape 1 decodes the bytes whole. Shape 2 puts them
//! after a valid name, surname and PersonPublicKey, where the device slots go,
//! so every iteration reaches the slot decoder, and every input whose slots
//! decode (a first byte of 0 always does) reaches Person::new's key rules;
//! random bytes alone rarely form two valid strings first.
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run it alone with
//! `cargo test -p person --test fuzz_person_decode`; deep-fuzz with
//! `cargo bolero test fuzz_person_decode --engine libfuzzer`.
//!
//! verifies: LLR-63pkfc, LLR-tf45kx, LLR-wqha8d

use bolero::check;
use person::definition::Person;
use person::group_key::check_group_key;
use person::hash::{person_hash, verify, PersonHash};
use person::successor::check_successor;

/// The postcard bytes of name "Alice", surname "Example" and a present
/// PersonPublicKey (u = 9): what precedes the device slots in shape 2.
fn prefix_before_the_slots() -> Vec<u8> {
    let mut prefix = vec![5];
    prefix.extend_from_slice(b"Alice");
    prefix.push(7);
    prefix.extend_from_slice(b"Example");
    prefix.push(1);
    let mut base_point = [0u8; 32];
    base_point[0] = 9;
    prefix.extend_from_slice(&base_point);
    prefix
}

fn exercise(person: &Person, bytes: &[u8]) {
    assert_eq!(
        check_group_key(person.person_key(), person.devices()),
        Ok(())
    );
    let mut supplied = [0u8; 32];
    for (slot, byte) in supplied.iter_mut().zip(bytes.iter()) {
        *slot = *byte;
    }
    let supplied = PersonHash::new(supplied);
    let _ = verify(person, 1, &supplied);
    let _ = verify(
        person,
        u16::from(bytes.first().copied().unwrap_or(2)),
        &supplied,
    );
    if let Ok(own) = person_hash(person, 1) {
        assert_eq!(verify(person, 1, &own), Ok(true));
    }
    assert_eq!(check_successor(person, person), Ok(()));
}

fn main() {
    let prefix = prefix_before_the_slots();
    check!().for_each(|bytes: &[u8]| {
        if let Ok(person) = postcard::from_bytes::<Person>(bytes) {
            exercise(&person, bytes);
        }
        let mut framed = prefix.clone();
        framed.extend_from_slice(bytes);
        if let Ok(person) = postcard::from_bytes::<Person>(&framed) {
            exercise(&person, bytes);
        }
    });
}
