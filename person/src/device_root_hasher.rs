//! The Person device root tree's hasher: its own domain keys and empty-leaf
//! sentinel, none of them org-members'. LLR-qwp8d7.

use sparse_merkle_tree::{NodeHash, TreeHasher};

use crate::device_root::DeviceRootHasher;
use crate::keyed_hash::keyed_hash;

/// `person::device-leaf`, padded with `_` to 32 bytes.
pub const PERSON_DEVICE_LEAF_KEY: &[u8; 32] = b"person::device-leaf_____________";
/// `person::device-node`, padded with `_` to 32 bytes.
pub const PERSON_DEVICE_NODE_KEY: &[u8; 32] = b"person::device-node_____________";
/// `person::device-identity`, padded with `_` to 32 bytes.
pub const PERSON_DEVICE_IDENTITY_KEY: &[u8; 32] = b"person::device-identity_________";
/// `person::device-content`, padded with `_` to 32 bytes.
pub const PERSON_DEVICE_CONTENT_KEY: &[u8; 32] = b"person::device-content__________";

fn keyed(key: &[u8; 32], parts: &[&[u8]]) -> NodeHash {
    NodeHash::new(keyed_hash(key, parts))
}

/// The Person device root tree's hasher: blake3 keyed per domain. LLR-qwp8d7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonDeviceRootHasher;

impl TreeHasher for PersonDeviceRootHasher {
    fn leaf(identity_hash: &NodeHash, content_hash: &NodeHash) -> NodeHash {
        keyed(PERSON_DEVICE_LEAF_KEY, &[identity_hash.as_bytes(), content_hash.as_bytes()])
    }

    fn node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        keyed(PERSON_DEVICE_NODE_KEY, &[left.as_bytes(), right.as_bytes()])
    }

    fn empty_leaf() -> NodeHash {
        keyed(PERSON_DEVICE_LEAF_KEY, &[Self::DEVICE_EMPTY_SENTINEL])
    }
}

impl DeviceRootHasher for PersonDeviceRootHasher {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_PERSON_DEVICE_V1";

    fn device_identity_hash(key: &[u8; 32]) -> NodeHash {
        keyed(PERSON_DEVICE_IDENTITY_KEY, &[key])
    }

    fn device_content_hash() -> NodeHash {
        keyed(PERSON_DEVICE_CONTENT_KEY, &[])
    }
}
