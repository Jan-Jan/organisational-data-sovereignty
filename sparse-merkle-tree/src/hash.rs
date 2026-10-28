//! The 32-byte hash the tree is built from. LLR-jd4phg.

use core::fmt;

/// A 32-byte hash: of a leaf, a node or a root. Wraps any 32 bytes,
/// unvalidated: it is the one `NodeHash` of the workspace, which `person` and
/// `org-members` re-export. LLR-jd4phg.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeHash([u8; 32]);

impl NodeHash {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for NodeHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for NodeHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [byte0, byte1, byte2, byte3, ..] = self.0;
        write!(formatter, "NodeHash({byte0:02x}{byte1:02x}{byte2:02x}{byte3:02x}..)")
    }
}
