//! Property tests: no input makes a constructor or a decoder panic, and every
//! accepted value satisfies its type's rule.

use ed25519_dalek::VerifyingKey;
use person::{x25519, DevicePublicKey, Name, PersonPublicKey, Surname, MAX_NAME_LEN};
#[cfg(feature = "serde")]
use person::{DeviceSlots, MAX_DEVICES};
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
