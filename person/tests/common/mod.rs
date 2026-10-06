//! Key and definition fixtures shared by the integration test crates. Each
//! test crate uses a subset, hence the `dead_code` allowance.
#![allow(dead_code)]

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::SigningKey;
use person::{DevicePublicKey, DeviceSlots, Name, Person, PersonPublicKey, Surname};

pub fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
        .expect("valid key")
}

pub fn slots(seeds: &[u8]) -> DeviceSlots {
    DeviceSlots::parse(seeds.iter().map(|seed| device(*seed)).collect()).expect("valid set")
}

pub fn person_key(seed: u8) -> PersonPublicKey {
    PersonPublicKey::parse(&MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes())
        .expect("valid X25519 key")
}

/// The Person named `name surname` holding the PersonPublicKey of `key` and
/// the devices of `seeds`.
pub fn person(name: &str, surname: &str, key: Option<u8>, seeds: &[u8]) -> Person {
    Person::new(
        Name::parse(name).expect("valid name"),
        Surname::parse(surname).expect("valid surname"),
        key.map(person_key),
        slots(seeds),
    )
    .expect("valid definition")
}

/// A device key whose 32 bytes are also a valid X25519 public key, and that
/// key: the literal copy LLR-kbhc43 refuses. Found by search, not assumed.
pub fn device_and_its_bytes_as_a_person_key() -> (DevicePublicKey, PersonPublicKey) {
    (1..=u8::MAX)
        .find_map(|seed| {
            let candidate = device(seed);
            PersonPublicKey::parse(candidate.as_bytes())
                .ok()
                .map(|key| (candidate, key))
        })
        .expect("some device key's bytes are a valid X25519 key")
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
