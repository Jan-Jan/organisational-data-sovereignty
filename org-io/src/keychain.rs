//! The keychain (SDD-4fvypt): one OS keychain item per Device public key,
//! service `ods-device-seed`, user the key's 64 lower-case hex, holding that
//! Device's 32-byte seed (LLR-677fqq). The one module in org-io that calls
//! the keychain crate, and, with `test_support`, the only one that names it.
//!
//! A read builds a fresh `DeviceSecret` from the item's bytes, moved into
//! `Zeroizing` in the same expression and consumed by
//! `DeviceSecret::from_keychain_bytes`; a write lends the seed to the item
//! through `DeviceSecret::lend_to_keychain` and nothing else (LLR-n9dnm4).
//! A refusal names the Device key and, when unexplained, the keychain
//! error's variant name only, never a byte of the item (LLR-8erfbu).
//!
//! The functions that return or take a `DeviceSecret` are `pub(crate)`: no
//! public API hands out a device private key (plan §17.2 rule 2). Tests reach
//! them through `test_support::keychain`, compiled only with `test-support`.
//!
//! On a target with no native backend (Android), keyring 3.6.3 would fall
//! back to its non-persistent in-memory store silently; org-io refuses there
//! instead (`KeychainError::Unavailable`), calling no keyring (LLR-8erfbu).
#![allow(
    clippy::result_large_err,
    reason = "every refusal names its Device public key (LLR-8erfbu), a parsed point; refusals are rare and not on a hot path"
)]

use org_node::store::PersonaStore;
use org_node::{DevicePublicKey, DeviceSecret, OrgNodeError, SealedStore};

/// The keychain item's service name; the user is the Device key's hex.
const SERVICE: &str = "ods-device-seed";

/// Why the keychain gave no secret, or took none: each names the Device key
/// whose item it is, and nothing of the item's content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeychainError {
    /// No item for this Device key.
    ItemMissing { device_key: DevicePublicKey },
    /// The keychain is locked or its access was refused.
    Locked { device_key: DevicePublicKey },
    /// Any other keychain failure; `reason` is the keychain error's variant
    /// name (`NoNativeBackend` on a target without a keychain).
    Unavailable { device_key: DevicePublicKey, reason: &'static str },
    /// The item holds the secret of another Device key.
    KeyMismatch { device_key: DevicePublicKey },
    /// The item is not a 32-byte Device seed.
    Malformed { device_key: DevicePublicKey },
}

impl std::fmt::Display for KeychainError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ItemMissing { device_key } => write!(formatter, "no keychain item for DevicePublicKey {device_key:?}"),
            Self::Locked { device_key } => write!(formatter, "the keychain is locked (DevicePublicKey {device_key:?})"),
            Self::Unavailable { device_key, reason } => {
                write!(formatter, "the keychain is unavailable: {reason} (DevicePublicKey {device_key:?})")
            }
            Self::KeyMismatch { device_key } => {
                write!(formatter, "the keychain item of DevicePublicKey {device_key:?} holds another key's Device seed")
            }
            Self::Malformed { device_key } => {
                write!(formatter, "the keychain item of DevicePublicKey {device_key:?} is not a 32-byte Device seed")
            }
        }
    }
}

impl std::error::Error for KeychainError {}

/// The Device secret the item of `device_key` holds (LLR-677fqq).
pub(crate) fn read_device_secret(device_key: &DevicePublicKey) -> Result<DeviceSecret, KeychainError> {
    backend::read_device_secret(device_key)
}

/// Write `secret` into the item of `device_key` (LLR-n9dnm4).
pub(crate) fn write_device_secret(device_key: &DevicePublicKey, secret: &DeviceSecret) -> Result<(), KeychainError> {
    backend::write_device_secret(device_key, secret)
}

/// Lend the 32-byte seed of `device_key`'s item to `build`, once, and drop
/// the buffer before returning `build`'s result (LLR-677fqq). Called by the
/// endpoint build, `OrgEndpoint::bind_from_keychain` (LLR-d5y7ks).
pub(crate) fn with_endpoint_seed<R>(
    device_key: &DevicePublicKey,
    build: impl FnOnce(&[u8; 32]) -> R,
) -> Result<R, KeychainError> {
    backend::with_endpoint_seed(device_key, build)
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "windows", target_os = "linux"))]
mod backend {
    use org_node::{DevicePublicKey, DeviceSecret};
    use zeroize::Zeroizing;

    use super::{KeychainError, SERVICE};

    /// The keychain's refusal, as a `KeychainError` naming the variant only.
    fn refusal(device_key: &DevicePublicKey, error: &keyring::Error) -> KeychainError {
        let device_key = *device_key;
        match error {
            keyring::Error::NoEntry => KeychainError::ItemMissing { device_key },
            keyring::Error::NoStorageAccess(_) => KeychainError::Locked { device_key },
            keyring::Error::PlatformFailure(_) => KeychainError::Unavailable { device_key, reason: "PlatformFailure" },
            keyring::Error::BadEncoding(_) => KeychainError::Unavailable { device_key, reason: "BadEncoding" },
            keyring::Error::TooLong(..) => KeychainError::Unavailable { device_key, reason: "TooLong" },
            keyring::Error::Invalid(..) => KeychainError::Unavailable { device_key, reason: "Invalid" },
            keyring::Error::Ambiguous(_) => KeychainError::Unavailable { device_key, reason: "Ambiguous" },
            _ => KeychainError::Unavailable { device_key, reason: "Unknown" },
        }
    }

    /// The item of `device_key`. In a test build, only once org-io's
    /// in-memory credential store is installed, so a test never reaches the
    /// OS keychain.
    fn entry(device_key: &DevicePublicKey) -> Result<keyring::Entry, KeychainError> {
        #[cfg(feature = "test-support")]
        if !crate::test_support::in_memory_keychain_installed() {
            return Err(KeychainError::Unavailable { device_key: *device_key, reason: "NotInstalledForTest" });
        }
        keyring::Entry::new(SERVICE, &hex::encode(device_key.as_bytes())).map_err(|error| refusal(device_key, &error))
    }

    pub(super) fn read_device_secret(device_key: &DevicePublicKey) -> Result<DeviceSecret, KeychainError> {
        let entry = entry(device_key)?;
        let secret = DeviceSecret::from_keychain_bytes(Zeroizing::new(
            entry.get_secret().map_err(|error| refusal(device_key, &error))?,
        ))
        .map_err(|_| KeychainError::Malformed { device_key: *device_key })?;
        if secret.device_key() != *device_key {
            return Err(KeychainError::KeyMismatch { device_key: *device_key });
        }
        Ok(secret)
    }

    pub(super) fn write_device_secret(device_key: &DevicePublicKey, secret: &DeviceSecret) -> Result<(), KeychainError> {
        let entry = entry(device_key)?;
        secret.lend_to_keychain(|seed| entry.set_secret(seed)).map_err(|error| refusal(device_key, &error))
    }

    pub(super) fn with_endpoint_seed<R>(
        device_key: &DevicePublicKey,
        build: impl FnOnce(&[u8; 32]) -> R,
    ) -> Result<R, KeychainError> {
        let entry = entry(device_key)?;
        let item = Zeroizing::new(entry.get_secret().map_err(|error| refusal(device_key, &error))?);
        let seed: &[u8; 32] =
            item.as_slice().try_into().map_err(|_| KeychainError::Malformed { device_key: *device_key })?;
        Ok(build(seed))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows", target_os = "linux")))]
mod backend {
    use org_node::{DevicePublicKey, DeviceSecret};

    use super::KeychainError;

    pub(super) fn read_device_secret(device_key: &DevicePublicKey) -> Result<DeviceSecret, KeychainError> {
        Err(KeychainError::Unavailable { device_key: *device_key, reason: "NoNativeBackend" })
    }

    pub(super) fn write_device_secret(device_key: &DevicePublicKey, _secret: &DeviceSecret) -> Result<(), KeychainError> {
        Err(KeychainError::Unavailable { device_key: *device_key, reason: "NoNativeBackend" })
    }

    pub(super) fn with_endpoint_seed<R>(
        device_key: &DevicePublicKey,
        _build: impl FnOnce(&[u8; 32]) -> R,
    ) -> Result<R, KeychainError> {
        Err(KeychainError::Unavailable { device_key: *device_key, reason: "NoNativeBackend" })
    }
}

/// Why `Keychain::unseal` opened no store: the keychain gave no secret, or
/// org-node refused the sealed bytes.
pub(crate) enum UnsealFailure {
    Keychain(KeychainError),
    Store(OrgNodeError),
}

/// The Device secrets org-io's handle reads and writes, as the field of its
/// local store: each call goes to the OS keychain.
#[derive(Default)]
pub(crate) struct Keychain;

impl Keychain {
    /// The keychain write of a new Persona: `secret` is consumed and
    /// zeroised when it drops here.
    pub(crate) fn keep(&self, secret: DeviceSecret) -> Result<(), KeychainError> {
        write_device_secret(&secret.device_key(), &secret)
    }

    /// The secrets a signing path asks for (org-node's `device_secrets`
    /// closure): those of `device_keys` the keychain gives; one it refuses is
    /// left out, and org-node refuses the signing step for want of it.
    pub(crate) fn for_signing(&self, device_keys: &[DevicePublicKey]) -> Vec<DeviceSecret> {
        self.for_signing_noting_refusal(device_keys, &mut None)
    }

    /// `for_signing`, keeping the first refusal in `refused`, so the caller
    /// can report the keychain's refusal rather than org-node's want of the
    /// secret (REQ-6d9xv6).
    pub(crate) fn for_signing_noting_refusal(
        &self,
        device_keys: &[DevicePublicKey],
        refused: &mut Option<KeychainError>,
    ) -> Vec<DeviceSecret> {
        let mut secrets = Vec::new();
        for device_key in device_keys {
            match read_device_secret(device_key) {
                Ok(secret) => secrets.push(secret),
                Err(error) => {
                    refused.get_or_insert(error);
                }
            }
        }
        secrets
    }

    /// `sealed`, the store of `device_key`, opened with the secret its item
    /// holds (LLR-uzax55, LLR-murjr6): the secret is read here and consumed
    /// by org-node's `unseal`.
    pub(crate) fn unseal(&self, sealed: &SealedStore, device_key: DevicePublicKey) -> Result<PersonaStore, UnsealFailure> {
        let secret = read_device_secret(&device_key).map_err(UnsealFailure::Keychain)?;
        PersonaStore::unseal(sealed, device_key, secret).map_err(UnsealFailure::Store)
    }
}

impl crate::LocalStore {
    /// The secrets org-node's signing paths ask for (`Keychain::for_signing`).
    pub(crate) fn device_secrets_for_signing(&self, device_keys: &[DevicePublicKey]) -> Vec<DeviceSecret> {
        self.keychain.for_signing(device_keys)
    }
}

/// The keychain functions, for the tests of `tests/keychain.rs` (test builds
/// only; re-exported as `test_support::keychain`).
#[cfg(feature = "test-support")]
pub mod for_test {
    use org_node::{DevicePublicKey, DeviceSecret};

    use super::KeychainError;

    pub fn read_device_secret(device_key: &DevicePublicKey) -> Result<DeviceSecret, KeychainError> {
        super::read_device_secret(device_key)
    }

    pub fn write_device_secret(device_key: &DevicePublicKey, secret: &DeviceSecret) -> Result<(), KeychainError> {
        super::write_device_secret(device_key, secret)
    }

    pub fn with_endpoint_seed<R>(device_key: &DevicePublicKey, build: impl FnOnce(&[u8; 32]) -> R) -> Result<R, KeychainError> {
        super::with_endpoint_seed(device_key, build)
    }
}

#[cfg(feature = "test-support")]
impl crate::OrgIo {
    /// The device secret of `device_key`, read from the keychain as the
    /// handle reads it, for tests that unseal what the handle wrote (test
    /// builds only).
    pub fn device_secret_for_test(&self, device_key: &DevicePublicKey) -> Option<DeviceSecret> {
        read_device_secret(device_key).ok()
    }
}
