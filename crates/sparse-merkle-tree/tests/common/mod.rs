//! A test hasher, a test leaf whose position the test chooses, two
//! oracles computing the classic tree's root without the crate's store, and
//! the sample tree the proof tests share.
#![allow(dead_code, clippy::expect_used, clippy::indexing_slicing)]

use sparse_merkle_tree::{leaf_hash, Leaf, NodeHash, SparseMerkleTree, TreeHasher};

fn hash(context: &str, parts: &[&[u8]]) -> NodeHash {
    let mut hasher = blake3::Hasher::new_derive_key(context);
    for part in parts {
        hasher.update(part);
    }
    NodeHash::new(hasher.finalize().into())
}

/// blake3 in derive-key mode, one context per domain.
#[derive(Debug, Clone, Copy)]
pub struct TestHasher;

impl TreeHasher for TestHasher {
    fn leaf(identity_hash: &NodeHash, content_hash: &NodeHash) -> NodeHash {
        hash("smt-test leaf", &[identity_hash.as_bytes(), content_hash.as_bytes()])
    }
    fn node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        hash("smt-test node", &[left.as_bytes(), right.as_bytes()])
    }
    fn empty_leaf() -> NodeHash {
        hash("smt-test leaf", &[b"empty"])
    }
}

/// A leaf identity: a name, and the position the test places it at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id {
    pub name: u32,
    pub position: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestLeaf {
    pub id: Id,
    pub content: u32,
}

impl Leaf<TestHasher> for TestLeaf {
    type Identity = Id;
    fn identity(&self) -> &Id {
        &self.id
    }
    fn position_bytes(identity: &Id) -> [u8; 32] {
        identity.position
    }
    fn identity_hash(identity: &Id) -> NodeHash {
        hash("smt-test identity", &[&identity.name.to_le_bytes(), &identity.position])
    }
    fn content_hash(&self) -> NodeHash {
        hash("smt-test content", &[&self.content.to_le_bytes()])
    }
}

pub type Tree<const D: usize> = SparseMerkleTree<TestLeaf, TestHasher, D>;

/// The identity `name` at the position whose first byte is `first`.
pub fn id(name: u32, first: u8) -> Id {
    let mut position = [0u8; 32];
    position[0] = first;
    Id { name, position }
}

pub fn leaf(name: u32, first: u8, content: u32) -> TestLeaf {
    TestLeaf { id: id(name, first), content }
}

pub fn leaf_at(name: u32, position: [u8; 32], content: u32) -> TestLeaf {
    TestLeaf { id: Id { name, position }, content }
}

pub fn bit(position: &[u8; 32], index: usize) -> bool {
    (position[index / 8] >> (7 - index % 8)) & 1 == 1
}

/// The empty-subtree hash of `level`, folded here from `empty_leaf`.
pub fn empty_at(level: usize) -> NodeHash {
    (0..level).fold(TestHasher::empty_leaf(), |hash, _| TestHasher::node(&hash, &hash))
}

/// Oracle 1, `tree_depth` <= 12: every one of the 2^tree_depth positions is a leaf slot,
/// folded pairwise to the root. No shortcut for empty subtrees.
pub fn classic_root_by_slots(tree_depth: usize, leaves: &[TestLeaf]) -> NodeHash {
    let mut level = vec![TestHasher::empty_leaf(); 1 << tree_depth];
    for leaf in leaves {
        let slot = (0..tree_depth).fold(0usize, |acc, index| (acc << 1) | usize::from(bit(&leaf.id.position, index)));
        level[slot] = leaf_hash::<TestLeaf, TestHasher>(leaf);
    }
    while level.len() > 1 {
        level = level.chunks(2).map(|pair| TestHasher::node(&pair[0], &pair[1])).collect();
    }
    level[0]
}

/// Oracle 2, any depth: the classic tree's subtree at `depth` holding
/// `leaves`, recursing on the position bit; an empty subtree is `empty_at`.
pub fn classic_root(tree_depth: usize, depth: usize, leaves: &[&TestLeaf]) -> NodeHash {
    if leaves.is_empty() {
        return empty_at(tree_depth - depth);
    }
    if depth == tree_depth {
        assert_eq!(leaves.len(), 1, "two leaves at one position");
        return leaf_hash::<TestLeaf, TestHasher>(leaves[0]);
    }
    let (right, left): (Vec<&TestLeaf>, Vec<&TestLeaf>) =
        leaves.iter().partition(|leaf| bit(&leaf.id.position, depth));
    TestHasher::node(&classic_root(tree_depth, depth + 1, &left), &classic_root(tree_depth, depth + 1, &right))
}

pub fn calculated<const D: usize>(leaves: &[TestLeaf]) -> Tree<D> {
    Tree::<D>::from_leaves(leaves.iter().cloned()).expect("no collision")
}

/// Three leaves of a depth-8 tree: 0x40 and 0x41 share seven bits; 0xc0 is
/// apart.
pub fn sample() -> Tree<8> {
    calculated::<8>(&[leaf(1, 0x40, 1), leaf(2, 0x41, 2), leaf(3, 0xc0, 3)])
}
