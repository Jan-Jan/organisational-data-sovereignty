//! OrgId = h160_of(P): the 20-byte contract storage key where an org's
//! OrgState lives. See spec §4.1. Also the invite identifier and the
//! endpoint addresses, which the Wire message carries (SDD-n2x3gb, SDD-42cjqk).
use serde::{Deserialize, Serialize};

/// The 20-byte H160 that keys an org's slot in the OrgRegistry contract.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrgId([u8; 20]);

impl OrgId {
    pub fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl core::fmt::Debug for OrgId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OrgId(0x")?;
        for b in self.0 {
            write!(f, "{:02x}", b)?;
        }
        write!(f, ")")
    }
}

/// The identifier an Invite carries: 32 bytes the inviting node drew at
/// random. Not secret. It binds an admission request to the Invite this node
/// issued (LLR-6gkxpg), the one Wire message that carries it (LLR-g3wmen).
/// Here, not in `invitation.rs`, so the Wire message, which no feature
/// gates, can name it; `invitation` re-exports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InviteId([u8; 32]);

impl InviteId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// The largest number of endpoint addresses one list holds (LLR-h4gwns).
pub const MAX_ENDPOINT_ADDRESSES: usize = 16;

/// The longest endpoint address, in bytes of UTF-8 (LLR-h4gwns).
pub const MAX_ENDPOINT_ADDRESS_BYTES: usize = 256;

/// Why `EndpointAddresses::parse` refused a list (LLR-h4gwns).
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EndpointAddressesError {
    /// More than sixteen entries.
    #[error("{count} endpoint addresses, at most {max} allowed", max = MAX_ENDPOINT_ADDRESSES)]
    TooMany { count: usize },
    /// An entry longer than 256 bytes.
    #[error("endpoint address {index} is {length} bytes, at most {max} allowed", max = MAX_ENDPOINT_ADDRESS_BYTES)]
    TooLong { index: usize, length: usize },
}

/// The endpoint addresses one Device gives for itself: zero to sixteen
/// entries, each at most 256 bytes of UTF-8 (LLR-h4gwns). Constructed only
/// through `parse`; org-node never interprets an entry. Here beside
/// `InviteId` for the same reason; `invitation` re-exports it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct EndpointAddresses(Vec<String>);

impl EndpointAddresses {
    /// Accepts zero to sixteen entries of at most 256 bytes; refuses any
    /// other list (LLR-h4gwns).
    pub fn parse(addresses: Vec<String>) -> Result<Self, EndpointAddressesError> {
        if addresses.len() > MAX_ENDPOINT_ADDRESSES {
            return Err(EndpointAddressesError::TooMany { count: addresses.len() });
        }
        if let Some((index, address)) =
            addresses.iter().enumerate().find(|(_, address)| address.len() > MAX_ENDPOINT_ADDRESS_BYTES)
        {
            return Err(EndpointAddressesError::TooLong { index, length: address.len() });
        }
        Ok(Self(addresses))
    }

    /// The entries, in the order given.
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}
