#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

extern crate alloc;

pub mod delta;
pub mod error;
pub mod hasher;
pub mod node;
pub mod normalize;
pub mod proof;
pub mod smt;
pub mod trie;
pub mod types;

pub use error::OrgMembersError;
pub use hasher::TrieHasher;
pub use proof::{AbsenceProof, ProofEnding};
pub use trie::OrgTrie;
pub use types::{
    DevicePublicKey, DeviceSlots, Handle, MemberId, MemberLeaf, Name, PersonPublicKey, RootHash,
    Surname,
};
