//! The Person device sub-trie's hashing: this unit's own domain keys and
//! empty-slot sentinel, none of them org-members'. LLR-edn55h.

use crate::device_trie::{DeviceTrieHasher, NodeHash};

/// `person::device-leaf`, padded with `_` to 32 bytes.
pub const PERSON_DEVICE_LEAF_KEY: &[u8; 32] = b"person::device-leaf_____________";

/// `person::device-node`, padded with `_` to 32 bytes.
pub const PERSON_DEVICE_NODE_KEY: &[u8; 32] = b"person::device-node_____________";

/// The Person device sub-trie's hasher: blake3 keyed by the leaf key over a
/// device key's 32 bytes (or the sentinel), and by the node key over two
/// child hashes concatenated. LLR-edn55h.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonDeviceHasher;

impl DeviceTrieHasher for PersonDeviceHasher {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_PERSON_DEVICE_V1";

    fn hash_device_leaf(data: &[u8]) -> NodeHash {
        NodeHash::new(blake3::keyed_hash(PERSON_DEVICE_LEAF_KEY, data).into())
    }

    fn hash_device_node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        let mut hasher = blake3::Hasher::new_keyed(PERSON_DEVICE_NODE_KEY);
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        NodeHash::new(hasher.finalize().into())
    }
}
