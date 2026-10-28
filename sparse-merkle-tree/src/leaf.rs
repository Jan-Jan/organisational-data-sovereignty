//! The interfaces a consumer implements: its hash domains and its leaf.
//! SDD-x43d2z.

use crate::hash::NodeHash;

/// The hash domains a tree is built under; each a pure function the consumer
/// implements under its own keys. The crate provides no implementation.
/// LLR-znpbf5.
pub trait TreeHasher {
    /// A leaf's hash from its identity hash and its content hash.
    fn leaf(identity_hash: &NodeHash, content_hash: &NodeHash) -> NodeHash;
    /// A node's hash from its left and right children's.
    fn node(left: &NodeHash, right: &NodeHash) -> NodeHash;
    /// The hash of a position holding no leaf.
    fn empty_leaf() -> NodeHash;
}

/// What a consumer's leaf supplies to the tree. Two leaves are the same leaf
/// for the tree when their identities are equal. LLR-t3e8pp.
pub trait Leaf<H: TreeHasher> {
    type Identity: Eq;

    fn identity(&self) -> &Self::Identity;
    /// The leaf's position bytes: the 32 bytes whose first `DEPTH` bits are
    /// its position.
    fn position_bytes(identity: &Self::Identity) -> [u8; 32];
    fn identity_hash(identity: &Self::Identity) -> NodeHash;
    fn content_hash(&self) -> NodeHash;
}

/// A leaf's hash: `H::leaf` of its identity hash and its content hash, and
/// nothing else of the leaf. LLR-9z6dc9.
pub fn leaf_hash<L: Leaf<H>, H: TreeHasher>(leaf: &L) -> NodeHash {
    H::leaf(&L::identity_hash(leaf.identity()), &leaf.content_hash())
}
