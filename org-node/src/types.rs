//! The value types org-node defines for what it holds that org-members does
//! not (SDD-swtd3w): the secrets (LLR-sz4xhc), the Organisation public key
//! (LLR-mmdu38), and the tag types for the chain account, the Persona
//! identifier, the epoch and the Sequence number (LLR-s7whrn). Seed to key
//! pair conversion lives in `keys.rs` (LLR-56hc77).
use core::fmt;

use ed25519_dalek::VerifyingKey;
use org_members::P2pMemberKey;
use serde::{Deserialize, Serialize};

use crate::error::OrgNodeError;

/// A secret type (LLR-sz4xhc): built infallibly from 32 bytes, gives them up
/// only through `expose_secret`, renders under `Debug` as its name and
/// `([REDACTED])` whatever it holds, has no `Display` and no `Copy`, and
/// serialises as the plain bytes it wraps. Wiping on drop is not done (owner
/// ruling 2026-10-04, RC-jjsz97 residual).
macro_rules! secret_type {
    ($(#[$meta:meta])* $ty:ident) => {
        $(#[$meta])*
        #[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $ty([u8; 32]);

        impl $ty {
            /// The secret bytes. The only way out: each call is a deliberate,
            /// searchable use (HAZ-uy8sxm residual).
            pub fn expose_secret(&self) -> &[u8; 32] {
                &self.0
            }
        }

        impl From<[u8; 32]> for $ty {
            fn from(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }
        }

        impl fmt::Debug for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($ty), "([REDACTED])"))
            }
        }
    };
}

secret_type!(
    /// The seed of a Member's signing key pair: the secret half of the
    /// Member-as-a-group key.
    MemberSeed
);
secret_type!(
    /// The seed of a device's signing key pair: the secret half of its Device
    /// key and of its iroh identity.
    DeviceSeed
);
secret_type!(
    /// The Organisation secret handed to a Member at admission.
    OrgSecret
);

/// The Organisation public key, parsed as a curve point (LLR-mmdu38). The
/// published signing key today (PR-szkat6).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrgPublicKey(VerifyingKey);

impl OrgPublicKey {
    /// Accepts exactly the 32 bytes that decompress to an Edwards point;
    /// refuses any other with `InvalidKey`.
    pub fn parse(bytes: &[u8; 32]) -> Result<Self, OrgNodeError> {
        VerifyingKey::from_bytes(bytes)
            .map(Self)
            .map_err(|_| OrgNodeError::InvalidKey)
    }

    /// The 32 bytes as given to `parse`.
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.0
    }
}

impl TryFrom<[u8; 32]> for OrgPublicKey {
    type Error = OrgNodeError;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        Self::parse(&bytes)
    }
}

/// The genesis value today: the founding administrator's Member key
/// (PR-szkat6).
impl From<&P2pMemberKey> for OrgPublicKey {
    fn from(key: &P2pMemberKey) -> Self {
        Self(*key.verifying_key())
    }
}

impl fmt::Debug for OrgPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.as_bytes();
        write!(f, "OrgPublicKey({:02x}{:02x}{:02x}{:02x}..)", b[0], b[1], b[2], b[3])
    }
}

impl Serialize for OrgPublicKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.as_bytes().serialize(s)
    }
}

impl<'de> Deserialize<'de> for OrgPublicKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let bytes = <[u8; 32]>::deserialize(d)?;
        Self::parse(&bytes).map_err(serde::de::Error::custom)
    }
}

/// A chain account (`AccountId32`): an Organisation's pure proxy or a
/// co-signatory. Tag type; converted to subxt's account type or raw bytes only
/// inside `chain_write`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChainAccount([u8; 32]);

impl ChainAccount {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for ChainAccount {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for ChainAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChainAccount(0x")?;
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        write!(f, ")")
    }
}

/// The identifier of a Persona in the Persona store. Tag type.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonaId(String);

impl PersonaId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for PersonaId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// An Organisation state's epoch. Tag type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Epoch(u64);

impl Epoch {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for Epoch {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// An Envelope's Sequence number. Tag type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SequenceNumber(u64);

impl SequenceNumber {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for SequenceNumber {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
