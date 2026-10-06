//! The Person hash's encoding versions. LLR-rde6tk.

use crate::error::IdentityError;

/// An encoding version this unit implements. Closed: a version is added only
/// with its encoding and its own hash domain key (LLR-4ebtn4). LLR-rde6tk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodingVersion {
    /// Name, surname, optional PersonPublicKey (X25519), the device root over
    /// ed25519 DevicePublicKeys. LLR-edn55h.
    V1,
}

impl TryFrom<u16> for EncodingVersion {
    type Error = IdentityError;

    /// 1 is `V1`; every other value is refused before any encoding or hashing.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::V1),
            other => Err(IdentityError::UnsupportedEncodingVersion(other)),
        }
    }
}

impl From<EncodingVersion> for u16 {
    fn from(version: EncodingVersion) -> Self {
        match version {
            EncodingVersion::V1 => 1,
        }
    }
}
