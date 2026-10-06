//! Property tests: no input makes a constructor, a decoder or a Person
//! operation panic, and every accepted value satisfies its type's rules.

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::{SigningKey, VerifyingKey};
use person::{
    check_group_key, check_successor, person_hash, verify, x25519, DevicePublicKey, DeviceSlots,
    Name, Person, PersonHash, PersonPublicKey, Surname, MAX_DEVICES, MAX_NAME_LEN,
};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    /// verifies: LLR-ayu93n, REQ-vxx8k3
    #[test]
    fn name_and_surname_parse_any_string(value in any::<String>()) {
        let name = Name::parse(&value).map(String::from);
        let surname = Surname::parse(&value).map(String::from);
        // Both bounds are 128 bytes, so both fields accept the same values.
        prop_assert_eq!(&name, &surname.map_err(|_| person::IdentityError::FieldTooLong {
            field: "name",
            max: MAX_NAME_LEN,
        }));
        if let Ok(stored) = name {
            prop_assert!(stored.len() <= MAX_NAME_LEN);
        }
    }

    /// verifies: LLR-7guspr, REQ-vxx8k3
    #[test]
    fn device_public_key_from_any_32_bytes(bytes in any::<[u8; 32]>()) {
        if let Ok(key) = DevicePublicKey::parse(&bytes) {
            let decoded = VerifyingKey::from_bytes(&bytes);
            prop_assert!(decoded.is_ok());
            prop_assert_eq!(key.as_bytes(), &bytes);
            prop_assert!(!key.verifying_key().is_weak());
            prop_assert!(key.verifying_key().to_edwards().is_torsion_free());
            prop_assert_eq!(key.verifying_key().to_edwards().compress().to_bytes(), bytes);
        }
    }

    /// PersonPublicKey and the exported check agree on every input.
    /// verifies: LLR-vs7etb, LLR-m75m7u, REQ-vxx8k3
    #[test]
    fn person_public_key_and_the_check_agree_on_any_32_bytes(bytes in any::<[u8; 32]>()) {
        let accepted = PersonPublicKey::parse(&bytes);
        prop_assert_eq!(accepted.is_ok(), x25519::is_valid_public_key(&bytes));
        if let Ok(key) = accepted {
            prop_assert_eq!(key.as_bytes(), &bytes);
        }
    }

    /// verifies: LLR-sjrh7z, REQ-vxx8k3
    #[cfg(feature = "serde")]
    #[test]
    fn decoding_any_bytes_does_not_panic(bytes in proptest::collection::vec(any::<u8>(), 0..300)) {
        let _ = postcard::from_bytes::<Name>(&bytes);
        let _ = postcard::from_bytes::<Surname>(&bytes);
        let _ = postcard::from_bytes::<DevicePublicKey>(&bytes);
        let _ = postcard::from_bytes::<PersonPublicKey>(&bytes);
        if let Ok(slots) = postcard::from_bytes::<DeviceSlots>(&bytes) {
            prop_assert!(slots.device_count() <= MAX_DEVICES);
            prop_assert!(slots.devices().windows(2).all(|pair| matches!(pair, [a, b] if a < b)));
        }
    }

    /// Byte strings shaped like a slots encoding — a small length, then keys —
    /// reach the per-key and ordering checks that random bytes rarely do.
    /// verifies: LLR-sjrh7z, REQ-vxx8k3
    #[cfg(feature = "serde")]
    #[test]
    fn decoding_slot_shaped_bytes_does_not_panic(
        length in 0u8..8,
        keys in proptest::collection::vec(any::<[u8; 32]>(), 0..6),
    ) {
        let mut bytes = vec![length];
        for key in &keys {
            bytes.extend_from_slice(key);
        }
        if let Ok(slots) = postcard::from_bytes::<DeviceSlots>(&bytes) {
            prop_assert!(slots.device_count() <= MAX_DEVICES);
        }
    }
}

/// The device key of a signing-key seed: every seed gives a valid key.
fn device_from_seed(seed: &[u8; 32]) -> Option<DevicePublicKey> {
    DevicePublicKey::try_from(SigningKey::from_bytes(seed).verifying_key()).ok()
}

/// A Person from small seeds, so that two draws share devices and keys often:
/// device seeds and the key seed index fixed signing and X25519 keys. `None`
/// when the fields break a rule.
fn person_from_small_seeds(key_seed: Option<u8>, device_seeds: &[u8]) -> Option<Person> {
    let devices: Vec<DevicePublicKey> = device_seeds
        .iter()
        .filter_map(|seed| device_from_seed(&[*seed; 32]))
        .collect();
    let key = key_seed.and_then(|seed| {
        PersonPublicKey::parse(&MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes()).ok()
    });
    let name = Name::parse("Alice").ok()?;
    let surname = Surname::parse("Example").ok()?;
    Person::new(name, surname, key, DeviceSlots::parse(devices).ok()?).ok()
}

/// The rules every accepted Person satisfies (REQ-ht3x78, REQ-7qgx2q, REQ-4szc22).
fn satisfies_the_person_rules(person: &Person) -> bool {
    let devices = person.devices().devices();
    let sorted = devices
        .windows(2)
        .all(|pair| matches!(pair, [a, b] if a < b));
    let key_rule = match person.person_key() {
        None => devices.is_empty(),
        Some(key) => {
            !devices.is_empty()
                && devices
                    .iter()
                    .all(|device| device.as_bytes() != key.as_bytes())
        }
    };
    devices.len() <= MAX_DEVICES && sorted && key_rule
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    /// Person::new on arbitrary fields never panics, accepts exactly what
    /// check_group_key accepts, and every Person it returns satisfies the rules.
    /// verifies: LLR-sjkmr6, LLR-kbhc43, LLR-4ku56h, LLR-3n3kxx
    #[test]
    fn person_new_on_any_fields(
        name in any::<String>(),
        surname in any::<String>(),
        key_bytes in proptest::option::of(any::<[u8; 32]>()),
        copy_a_device_key in any::<bool>(),
        seeds in proptest::collection::vec(any::<[u8; 32]>(), 0..6),
    ) {
        let devices: Vec<DevicePublicKey> = seeds.iter().filter_map(device_from_seed).collect();
        let key = if copy_a_device_key {
            devices.first().and_then(|device| PersonPublicKey::parse(device.as_bytes()).ok())
        } else {
            key_bytes.and_then(|bytes| PersonPublicKey::parse(&bytes).ok())
        };
        let (Ok(name), Ok(surname), Ok(slots)) =
            (Name::parse(&name), Surname::parse(&surname), DeviceSlots::parse(devices))
        else {
            return Ok(());
        };
        let expected = check_group_key(key.as_ref(), &slots);
        let built = Person::new(name, surname, key, slots);
        prop_assert_eq!(built.clone().map(|_| ()), expected);
        if let Ok(person) = built {
            prop_assert!(satisfies_the_person_rules(&person));
        }
    }

    /// check_successor never panics, and refuses exactly a device change that
    /// keeps a present key.
    /// verifies: LLR-wqha8d, LLR-3n3kxx
    #[test]
    fn check_successor_on_any_pair(
        previous_key in proptest::option::of(1u8..4),
        previous_devices in proptest::collection::vec(1u8..7, 0..5),
        next_key in proptest::option::of(1u8..4),
        next_devices in proptest::collection::vec(1u8..7, 0..5),
    ) {
        let (Some(previous), Some(next)) = (
            person_from_small_seeds(previous_key, &previous_devices),
            person_from_small_seeds(next_key, &next_devices),
        ) else {
            return Ok(());
        };
        let refused = previous.devices() != next.devices()
            && previous.person_key().is_some()
            && previous.person_key() == next.person_key();
        let expected = if refused {
            Err(person::IdentityError::PersonKeyNotRotated)
        } else {
            Ok(())
        };
        prop_assert_eq!(check_successor(&previous, &next), expected);
    }

    /// person_hash and verify never panic for any version and any supplied
    /// hash; only version 1 is implemented, and verify matches exactly the
    /// definition's own hash.
    /// verifies: LLR-tf45kx, LLR-4ebtn4, LLR-rde6tk
    #[test]
    fn hash_and_verify_on_any_version_and_hash(
        key in proptest::option::of(1u8..4),
        devices in proptest::collection::vec(1u8..7, 0..5),
        // Version 1, the one implemented, is drawn half the time; any::<u16>()
        // alone would reach it once in 65536 draws.
        version in prop_oneof![Just(1u16), any::<u16>()],
        supplied in any::<[u8; 32]>(),
    ) {
        let Some(definition) = person_from_small_seeds(key, &devices) else {
            return Ok(());
        };
        let supplied = PersonHash::new(supplied);
        match person_hash(&definition, version) {
            Ok(own) => {
                prop_assert_eq!(version, 1);
                prop_assert_eq!(verify(&definition, version, &own), Ok(true));
                prop_assert_eq!(verify(&definition, version, &supplied), Ok(own == supplied));
            }
            Err(error) => {
                prop_assert_ne!(version, 1);
                prop_assert_eq!(&error, &person::IdentityError::UnsupportedEncodingVersion(version));
                prop_assert_eq!(verify(&definition, version, &supplied), Err(error));
            }
        }
    }

    /// Decoding arbitrary bytes as a Person never panics, and every Person it
    /// returns satisfies the rules and re-encodes to the bytes it consumed.
    /// verifies: LLR-63pkfc
    #[cfg(feature = "serde")]
    #[test]
    fn decoding_any_bytes_as_a_person_does_not_panic(
        bytes in proptest::collection::vec(any::<u8>(), 0..400),
    ) {
        if let Ok((person, rest)) = postcard::take_from_bytes::<Person>(&bytes) {
            prop_assert!(satisfies_the_person_rules(&person));
            let consumed = bytes.len() - rest.len();
            prop_assert_eq!(postcard::to_allocvec(&person).ok(), Some(bytes[..consumed].to_vec()));
        }
    }

    /// Byte strings shaped like a Person encoding — two short strings, an
    /// option tag, a key, a slot count and keys from signing seeds — reach
    /// Person::new's rules, which random bytes rarely do.
    /// verifies: LLR-63pkfc, LLR-sjkmr6, LLR-kbhc43
    #[cfg(feature = "serde")]
    #[test]
    fn decoding_person_shaped_bytes_does_not_panic(
        name in "[a-z]{0,8}",
        surname in "[a-z]{0,8}",
        key_tag in 0u8..3,
        key_bytes in any::<[u8; 32]>(),
        copy_a_device_key in any::<bool>(),
        slot_count in 0u8..6,
        seeds in proptest::collection::vec(any::<[u8; 32]>(), 0..6),
    ) {
        let mut devices: Vec<[u8; 32]> = seeds
            .iter()
            .filter_map(device_from_seed)
            .map(|device| *device.as_bytes())
            .collect();
        devices.sort();
        let key = match (copy_a_device_key, devices.first()) {
            (true, Some(first)) => *first,
            _ => key_bytes,
        };
        let mut bytes = postcard::to_allocvec(&name).unwrap_or_default();
        bytes.extend(postcard::to_allocvec(&surname).unwrap_or_default());
        bytes.push(key_tag);
        bytes.extend_from_slice(&key);
        bytes.push(slot_count);
        for device in &devices {
            bytes.extend_from_slice(device);
        }
        if let Ok(person) = postcard::from_bytes::<Person>(&bytes) {
            prop_assert!(satisfies_the_person_rules(&person));
        }
    }
}
