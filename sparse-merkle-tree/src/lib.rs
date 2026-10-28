//! One sparse Merkle tree of a fixed depth, generic over its leaf and its
//! hasher, with absence and inclusion proofs: SDD-x43d2z (the consumer
//! interfaces), SDD-rh9u6z (the tree store), SDD-ur6uvu (the proofs).
//! org-members builds the Membership record on it at depth 256, person a
//! device root tree at depth 4.

#![no_std]
#![deny(clippy::indexing_slicing)]

extern crate alloc;

mod bits;
mod defaults;
pub mod error;
pub mod hash;
pub mod leaf;
mod node;
pub mod proof;
pub mod tree;

pub use error::TreeError;
pub use hash::NodeHash;
pub use leaf::{leaf_hash, Leaf, TreeHasher};
pub use proof::{AbsenceProof, InclusionProof, ProofEnding};
pub use tree::SparseMerkleTree;
