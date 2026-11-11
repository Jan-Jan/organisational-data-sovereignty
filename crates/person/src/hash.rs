//! The Person hash: an encoding of a definition under an encoding version,
//! hashed with blake3 keyed per unit and version, and the check of a supplied
//! hash. Free functions over their arguments; nothing is kept between calls.
//! SDD-v32mqh.

use alloc::vec::Vec;
use core::fmt;

use crate::definition::Person;
use crate::device_root::compute_device_root;
use crate::device_root_hasher::PersonDeviceRootHasher;
use crate::encoding::EncodingVersion;
use crate::error::IdentityError;
use crate::keyed_hash::keyed_hash;
use crate::name::{Name, Surname, MAX_NAME_LEN, MAX_SURNAME_LEN};
use crate::person_key::PersonPublicKey;
use sparse_merkle_tree::NodeHash;

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

/// The encoding of `person` under `version`; `DevicePositionTaken` only for
/// devices two of which share a position, which a `Person`'s `DeviceSlots`
/// never holds. LLR-qwp8d7.
pub fn encode(person: &Person, version: EncodingVersion) -> Result<Vec<u8>, IdentityError> {
    let device_root = compute_device_root::<PersonDeviceRootHasher>(person.devices().devices())?;
    Ok(match version {
        EncodingVersion::V1 => encode_v1(person.name(), person.surname(), person.person_key(), &device_root),
    })
}

/// Name length (u32 LE) and NFC bytes; surname likewise; `0x00`, or `0x01`
/// and the PersonPublicKey's 32 bytes; the 32-byte device root. LLR-qwp8d7.
pub(crate) fn encode_v1(
    name: &Name,
    surname: &Surname,
    person_key: Option<&PersonPublicKey>,
    device_root: &NodeHash,
) -> Vec<u8> {
    let name = name.as_str().as_bytes();
    let surname = surname.as_str().as_bytes();
    let mut encoding = Vec::with_capacity(4 + name.len() + 4 + surname.len() + 1 + 32 + 32);
    encoding.extend_from_slice(&(name.len() as u32).to_le_bytes());
    encoding.extend_from_slice(name);
    encoding.extend_from_slice(&(surname.len() as u32).to_le_bytes());
    encoding.extend_from_slice(surname);
    match person_key {
        None => encoding.push(0x00),
        Some(key) => {
            encoding.push(0x01);
            encoding.extend_from_slice(key.as_bytes());
        }
    }
    encoding.extend_from_slice(device_root.as_bytes());
    encoding
}

/// The keyed hash of an encoding under `version`'s domain key. LLR-4ebtn4.
pub(crate) fn hash_encoding(version: EncodingVersion, encoding: &[u8]) -> PersonHash {
    let key = match version {
        EncodingVersion::V1 => PERSON_DEFINITION_V1_KEY,
    };
    PersonHash(keyed_hash(key, &[encoding]))
}

/// The hash of `person` under the encoding version `version`, refused with
/// `UnsupportedEncodingVersion` for a version this unit does not implement.
/// LLR-4ebtn4, LLR-rde6tk.
pub fn person_hash(person: &Person, version: u16) -> Result<PersonHash, IdentityError> {
    let version = EncodingVersion::try_from(version)?;
    Ok(hash_encoding(version, &encode(person, version)?))
}

/// `Ok(true)` exactly when `hash` is the hash of `person` under `version`,
/// `Ok(false)` when it is not, and `Err(UnsupportedEncodingVersion)` only for
/// a version this unit does not implement. LLR-tf45kx.
pub fn verify(person: &Person, version: u16, hash: &PersonHash) -> Result<bool, IdentityError> {
    Ok(person_hash(person, version)? == *hash)
}
