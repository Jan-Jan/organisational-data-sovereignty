use core::fmt;

use crate::device_key::DevicePublicKey;
use crate::slots::DeviceSlots;

/// A 32-byte hash output.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeHash([u8; 32]);

impl NodeHash {
    pub fn new(bytes: [u8; 32]) -> Self {
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
        crate::write_hex_prefix(formatter, "NodeHash", &self.0)
    }
}

/// The hashing a device sub-trie is computed through. The implementor
/// chooses the domains and the empty-slot sentinel. LLR-4vsm8d.
pub trait DeviceTrieHasher {
    /// Hashed with `hash_device_leaf` to fill an unoccupied slot.
    const DEVICE_EMPTY_SENTINEL: &'static [u8];

    /// Hash a device public key (device-leaf domain).
    fn hash_device_leaf(data: &[u8]) -> NodeHash;

    /// Hash two device child hashes into a parent (device-node domain).
    fn hash_device_node(left: &NodeHash, right: &NodeHash) -> NodeHash;
}

/// The root of the depth-2, four-slot device sub-trie over `devices`, keys in
/// sorted order, unoccupied slots hashing the sentinel. LLR-6ezhw7.
pub fn compute_device_root<H: DeviceTrieHasher>(devices: &DeviceSlots) -> NodeHash {
    let empty_leaf = H::hash_device_leaf(H::DEVICE_EMPTY_SENTINEL);
    let leaf = |slot: Option<DevicePublicKey>| match slot {
        Some(device) => H::hash_device_leaf(device.as_bytes()),
        None => empty_leaf,
    };
    let [slot0, slot1, slot2, slot3] = devices.to_fixed_slots();
    let left = H::hash_device_node(&leaf(slot0), &leaf(slot1));
    let right = H::hash_device_node(&leaf(slot2), &leaf(slot3));
    H::hash_device_node(&left, &right)
}
