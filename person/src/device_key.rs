use core::cmp::Ordering;
use core::fmt;

use ed25519_dalek::VerifyingKey;

use crate::error::IdentityError;

/// A device's ed25519 public key, canonically encoded, torsion-free and not
/// of small order: the device's identity. LLR-7guspr.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DevicePublicKey(VerifyingKey);

impl DevicePublicKey {
    /// Accepts the bytes when `VerifyingKey::from_bytes` does, they are the
    /// canonical encoding of the point, the point is torsion-free and it is
    /// not of small order: dalek decodes small-order points, points with a
    /// torsion component and non-canonical encodings. The identity is
    /// torsion-free and of small order, so the small-order check stays.
    pub fn parse(bytes: &[u8; 32]) -> Result<Self, IdentityError> {
        let key = VerifyingKey::from_bytes(bytes).map_err(|_| IdentityError::InvalidDeviceKey)?;
        let point = key.to_edwards();
        let is_canonical = point.compress().to_bytes() == *bytes;
        if is_canonical && point.is_torsion_free() && !key.is_weak() {
            Ok(Self(key))
        } else {
            Err(IdentityError::InvalidDeviceKey)
        }
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.0
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

impl TryFrom<[u8; 32]> for DevicePublicKey {
    type Error = IdentityError;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        Self::parse(&bytes)
    }
}

impl TryFrom<&[u8; 32]> for DevicePublicKey {
    type Error = IdentityError;

    fn try_from(bytes: &[u8; 32]) -> Result<Self, Self::Error> {
        Self::parse(bytes)
    }
}

/// A key dalek has decoded may be small-order, have a torsion component or be
/// non-canonically encoded, so it goes through `parse` too.
impl TryFrom<VerifyingKey> for DevicePublicKey {
    type Error = IdentityError;

    fn try_from(key: VerifyingKey) -> Result<Self, Self::Error> {
        Self::parse(key.as_bytes())
    }
}

impl PartialOrd for DevicePublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DevicePublicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl fmt::Debug for DevicePublicKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::write_hex_prefix(formatter, "DevicePublicKey", self.as_bytes())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DevicePublicKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_bytes().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DevicePublicKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = <[u8; 32]>::deserialize(deserializer)?;
        Self::parse(&bytes).map_err(serde::de::Error::custom)
    }
}
