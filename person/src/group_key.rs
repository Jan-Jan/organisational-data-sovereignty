//! The group-key rules a Person definition, and later a Member record, must
//! satisfy (docs/adr/2026-10-04-group-key-rules.md). LLR-sjkmr6, LLR-kbhc43.

use crate::error::IdentityError;
use crate::person_key::PersonPublicKey;
use crate::slots::DeviceSlots;

/// Checks a group key against the device slots it is held with: with no
/// device the key must be absent (`KeyWithoutDevice`); with one or more it
/// must be present (`MissingPersonKey`) and its 32 bytes must differ from
/// every device key's 32 bytes (`PersonKeyIsDeviceKey`).
///
/// The comparison is of raw bytes only (owner, 2026-10-06): it does not catch
/// a device's ed25519 key reused through its X25519 image, nor one X25519 key
/// under another of its encodings. LLR-sjkmr6, LLR-kbhc43.
pub fn check_group_key(
    group_key: Option<&PersonPublicKey>,
    devices: &DeviceSlots,
) -> Result<(), IdentityError> {
    match (group_key, devices.device_count()) {
        (None, 0) => Ok(()),
        (Some(_), 0) => Err(IdentityError::KeyWithoutDevice),
        (None, _) => Err(IdentityError::MissingPersonKey),
        (Some(key), _)
            if devices
                .devices()
                .iter()
                .any(|device| device.as_bytes() == key.as_bytes()) =>
        {
            Err(IdentityError::PersonKeyIsDeviceKey)
        }
        (Some(_), _) => Ok(()),
    }
}
