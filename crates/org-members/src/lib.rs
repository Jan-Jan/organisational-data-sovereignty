#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

extern crate alloc;

pub mod delta;
pub mod error;
pub mod hasher;
pub mod member_tree;
pub mod normalize;
pub mod proof;
pub mod trie;
pub mod types;

pub use error::OrgMembersError;
pub use hasher::MemberHasher;
pub use member_tree::MEMBER_DEPTH;
pub use proof::{AbsenceProof, ProvenMember};
pub use trie::OrgTrie;
pub use types::{
    DevicePublicKey, DeviceSlots, Handle, MemberId, MemberLeaf, Name, PersonPublicKey, RootHash,
    Surname,
};
