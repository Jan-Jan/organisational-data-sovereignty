//! The tree reaches a leaf only through `Leaf` and hashes only through
//! `TreeHasher`.

#![allow(clippy::expect_used)]

mod common;

use common::{leaf, Id, TestHasher, TestLeaf, Tree};
use sparse_merkle_tree::{leaf_hash, Leaf, NodeHash, SparseMerkleTree, TreeHasher};

/// verifies: LLR-znpbf5, LLR-t3e8pp, LLR-9z6dc9
///
/// At depth 1 a leaf on the left gives the root `node(leaf(idh, ch), empty_leaf)`,
/// one on the right `node(empty_leaf, leaf(idh, ch))`.
#[test]
fn a_depth_one_root_is_built_from_the_three_hasher_functions() {
    let left = leaf(1, 0x00, 7);
    let right = leaf(2, 0x80, 8);
    let hash_of = |leaf: &TestLeaf| {
        TestHasher::leaf(&TestLeaf::identity_hash(&leaf.id), &leaf.content_hash())
    };
    let tree = Tree::<1>::from_leaves([left.clone()]).expect("one leaf");
    assert_eq!(tree.root(), Ok(TestHasher::node(&hash_of(&left), &TestHasher::empty_leaf())));
    let tree = Tree::<1>::from_leaves([right.clone()]).expect("one leaf");
    assert_eq!(tree.root(), Ok(TestHasher::node(&TestHasher::empty_leaf(), &hash_of(&right))));
    assert_eq!(leaf_hash::<TestLeaf, TestHasher>(&left), hash_of(&left));
}

/// A leaf type carrying a field it does not hash.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Noted {
    inner: TestLeaf,
    note: u32,
}

impl Leaf<TestHasher> for Noted {
    type Identity = Id;
    fn identity(&self) -> &Id {
        &self.inner.id
    }
    fn position_bytes(identity: &Id) -> [u8; 32] {
        identity.position
    }
    fn identity_hash(identity: &Id) -> NodeHash {
        TestLeaf::identity_hash(identity)
    }
    fn content_hash(&self) -> NodeHash {
        self.inner.content_hash()
    }
}

/// verifies: LLR-9z6dc9, LLR-t3e8pp
///
/// Nothing of a leaf but its identity hash and content hash enters the tree:
/// two leaves differing only in an unhashed field give one root; a leaf of
/// another content gives another.
#[test]
fn only_the_identity_hash_and_content_hash_enter_the_tree() {
    let root = |note: u32, content: u32| {
        SparseMerkleTree::<Noted, TestHasher, 8>::root_of([Noted { inner: leaf(1, 0x42, content), note }])
            .expect("one leaf")
    };
    assert_eq!(root(1, 5), root(2, 5));
    assert_ne!(root(1, 5), root(1, 6));
}
