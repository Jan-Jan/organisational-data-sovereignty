use core::fmt;

use crate::error::IdentityError;
use crate::x25519;

/// The key a grant to a Person or a Member is encoded against. Contains the 32
/// bytes it was constructed from. LLR-vs7etb.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PersonPublicKey([u8; 32]);

impl PersonPublicKey {
    /// Accepts a canonical, non-small-order X25519 public key. LLR-vs7etb.
    pub fn parse(bytes: &[u8; 32]) -> Result<Self, IdentityError> {
        if x25519::is_valid_public_key(bytes) {
            Ok(Self(*bytes))
        } else {
            Err(IdentityError::InvalidPersonKey)
        }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl TryFrom<[u8; 32]> for PersonPublicKey {
    type Error = IdentityError;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        Self::parse(&bytes)
    }
}

impl TryFrom<&[u8; 32]> for PersonPublicKey {
    type Error = IdentityError;

    fn try_from(bytes: &[u8; 32]) -> Result<Self, Self::Error> {
        Self::parse(bytes)
    }
}

impl fmt::Debug for PersonPublicKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::write_hex_prefix(formatter, "PersonPublicKey", &self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for PersonPublicKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for PersonPublicKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = <[u8; 32]>::deserialize(deserializer)?;
        Self::parse(&bytes).map_err(serde::de::Error::custom)
    }
}
