//! The Person hash: an encoding of a definition under an encoding version,
//! hashed with blake3 keyed per unit and version, and the check of a supplied
//! hash. Free functions over their arguments; nothing is kept between calls.
//! SDD-v32mqh.

use alloc::vec::Vec;
use core::fmt;

use crate::definition::Person;
use crate::device_hasher::PersonDeviceHasher;
use crate::device_trie::compute_device_root;
use crate::encoding::EncodingVersion;
use crate::error::IdentityError;
use crate::name::{MAX_NAME_LEN, MAX_SURNAME_LEN};

/// `person::definition::v1`, padded with `_` to 32 bytes. LLR-4ebtn4.
pub const PERSON_DEFINITION_V1_KEY: &[u8; 32] = b"person::definition::v1__________";

/// A name or surname length always fits the encoding's 4-byte length prefix,
/// so the `as u32` casts in `encode_v1` are lossless.
const _: () = assert!(MAX_NAME_LEN <= u32::MAX as usize && MAX_SURNAME_LEN <= u32::MAX as usize);

/// The 32-byte hash of a Person definition under one encoding version.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PersonHash([u8; 32]);

impl PersonHash {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for PersonHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for PersonHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::write_hex_prefix(formatter, "PersonHash", &self.0)
    }
}

/// The encoding of `person` under `version`. LLR-edn55h.
pub fn encode(person: &Person, version: EncodingVersion) -> Vec<u8> {
    match version {
        EncodingVersion::V1 => encode_v1(person),
    }
}

/// Name length (u32 LE) and NFC bytes; surname likewise; `0x00`, or `0x01`
/// and the PersonPublicKey's 32 bytes; the 32-byte Person device root.
/// LLR-edn55h.
fn encode_v1(person: &Person) -> Vec<u8> {
    let name = person.name().as_str().as_bytes();
    let surname = person.surname().as_str().as_bytes();
    let mut encoding = Vec::with_capacity(4 + name.len() + 4 + surname.len() + 1 + 32 + 32);
    encoding.extend_from_slice(&(name.len() as u32).to_le_bytes());
    encoding.extend_from_slice(name);
    encoding.extend_from_slice(&(surname.len() as u32).to_le_bytes());
    encoding.extend_from_slice(surname);
    match person.person_key() {
        None => encoding.push(0x00),
        Some(key) => {
            encoding.push(0x01);
            encoding.extend_from_slice(key.as_bytes());
        }
    }
    let device_root = compute_device_root::<PersonDeviceHasher>(person.devices());
    encoding.extend_from_slice(device_root.as_bytes());
    encoding
}

/// The hash of `person` under the encoding version `version`, refused with
/// `UnsupportedEncodingVersion` for a version this unit does not implement.
/// LLR-4ebtn4, LLR-rde6tk.
pub fn person_hash(person: &Person, version: u16) -> Result<PersonHash, IdentityError> {
    let version = EncodingVersion::try_from(version)?;
    let key = match version {
        EncodingVersion::V1 => PERSON_DEFINITION_V1_KEY,
    };
    let encoding = encode(person, version);
    Ok(PersonHash(blake3::keyed_hash(key, &encoding).into()))
}

/// `Ok(true)` exactly when `hash` is the hash of `person` under `version`,
/// `Ok(false)` when it is not, and `Err(UnsupportedEncodingVersion)` only for
/// a version this unit does not implement. LLR-tf45kx.
pub fn verify(person: &Person, version: u16, hash: &PersonHash) -> Result<bool, IdentityError> {
    Ok(person_hash(person, version)? == *hash)
}
