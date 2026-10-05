//! OrgId = h160_of(P): the 20-byte contract storage key where an org's
//! OrgState lives. See spec §4.1.
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

