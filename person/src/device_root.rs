//! The device root (SDD-7r833z): the root of the device root tree, a sparse
//! Merkle tree of depth 4 over a set's DevicePublicKeys, built on the
//! `sparse-merkle-tree` unit, under the hash domains of the caller's hasher.

use sparse_merkle_tree::{AbsenceProof, Leaf, NodeHash, SparseMerkleTree, TreeHasher};

pub use crate::position::{device_position_bytes, DEVICE_POSITION_KEY};
use crate::device_key::DevicePublicKey;
use crate::error::IdentityError;

/// The device root tree's depth: 16 positions, above `MAX_DEVICES` (8).
pub const DEVICE_ROOT_DEPTH: usize = 4;

const _: () = assert!(DEVICE_ROOT_DEPTH <= 8, "a device position is read from its first byte");

/// The hashing a device root tree is built under: the tree's three domains, a
/// key's identity and its (empty) content, each under a domain the
/// implementor chooses, and the sentinel whose hash in the leaf domain is
/// `empty_leaf()`. LLR-9mjfjn.
pub trait DeviceRootHasher: TreeHasher {
    const DEVICE_EMPTY_SENTINEL: &'static [u8];
    fn device_identity_hash(key: &[u8; 32]) -> NodeHash;
    fn device_content_hash() -> NodeHash;
}

/// Whether two keys share the first `DEVICE_ROOT_DEPTH` bits of their
/// position bytes: the comparison of two keys' device positions. LLR-exwx4n.
pub(crate) fn same_device_position(first: &DevicePublicKey, second: &DevicePublicKey) -> bool {
    let [first_byte, ..] = device_position_bytes(first);
    let [second_byte, ..] = device_position_bytes(second);
    (first_byte ^ second_byte) >> (8 - DEVICE_ROOT_DEPTH) == 0
}

/// A device root tree's leaf: one DevicePublicKey. LLR-8y8tqr.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceLeaf(DevicePublicKey);

impl DeviceLeaf {
    pub fn new(key: DevicePublicKey) -> Self {
        Self(key)
    }

    pub fn public_key(&self) -> &DevicePublicKey {
        &self.0
    }
}

impl<H: DeviceRootHasher> Leaf<H> for DeviceLeaf {
    type Identity = DevicePublicKey;

    fn identity(&self) -> &DevicePublicKey {
        &self.0
    }

    fn position_bytes(identity: &DevicePublicKey) -> [u8; 32] {
        device_position_bytes(identity)
    }

    fn identity_hash(identity: &DevicePublicKey) -> NodeHash {
        H::device_identity_hash(identity.as_bytes())
    }

    fn content_hash(&self) -> NodeHash {
        H::device_content_hash()
    }
}

/// The device root tree; crate-private, so no `TreeError` crosses this
/// unit's own functions (LLR-3dvfjb).
pub(crate) type DeviceTree<H> = SparseMerkleTree<DeviceLeaf, H, DEVICE_ROOT_DEPTH>;

/// The calculated tree of `devices`, or `DevicePositionTaken` when two of
/// them share a position. LLR-8y8tqr.
fn device_tree<H: DeviceRootHasher>(devices: &[DevicePublicKey]) -> Result<DeviceTree<H>, IdentityError> {
    DeviceTree::<H>::from_leaves(devices.iter().copied().map(DeviceLeaf))
        // Only `Collision` arrives: a new tree is calculated once, at a depth in range.
        .map_err(|_| IdentityError::DevicePositionTaken)
}

/// The absence proof of `device` under the device root of `devices`;
/// `DevicePositionTaken` when two of `devices` share a position,
/// `DeviceStillHeld` when `device` is among them. LLR-8y8tqr.
pub fn prove_device_absent_in<H: DeviceRootHasher>(
    devices: &[DevicePublicKey],
    device: &DevicePublicKey,
) -> Result<AbsenceProof<DEVICE_ROOT_DEPTH>, IdentityError> {
    device_tree::<H>(devices)?
        .prove_absent(device)
        // Only `LeafHeld` arrives: the tree is calculated, at a depth in range.
        .map_err(|_| IdentityError::DeviceStillHeld)
}

/// The root of the device root tree of `devices`, or `DevicePositionTaken`
/// when two of them share a position.
/// LLR-8y8tqr.
pub fn compute_device_root<H: DeviceRootHasher>(devices: &[DevicePublicKey]) -> Result<NodeHash, IdentityError> {
    DeviceTree::<H>::root_of(devices.iter().copied().map(DeviceLeaf))
        // Only `Collision` arrives: `root_of` reads the root of a calculated tree.
        .map_err(|_| IdentityError::DevicePositionTaken)
}
