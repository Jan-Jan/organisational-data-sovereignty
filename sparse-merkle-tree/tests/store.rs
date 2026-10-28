//! The tree store: depth, positions, empty subtrees, compact nodes,
//! immutability, insert and remove, write-once hashes, order independence,
//! building and reading.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod common;

use common::{
    calculated, classic_root, classic_root_by_slots, empty_at, id, leaf, leaf_at, TestHasher,
    TestLeaf, Tree,
};
use proptest::prelude::*;
use sparse_merkle_tree::{TreeError, TreeHasher};

/// verifies: LLR-ggu9bh
///
/// The accepted boundary: depths 1 and 256 build, hold a leaf and give a
/// root. The refused side (0, and above 256) does not compile: the
/// `compile_fail` doctest on `SparseMerkleTree`, beside its compiling twin,
/// run by `cargo test -p sparse-merkle-tree`.
#[test]
fn depths_one_and_256_build() {
    let one = Tree::<1>::new().insert(leaf(1, 0, 1)).expect("insert").calculate().expect("calculate");
    assert!(one.root().is_ok());
    let deep = Tree::<256>::new().insert(leaf(1, 0, 1)).expect("insert").calculate().expect("calculate");
    assert!(deep.root().is_ok());
}

/// verifies: LLR-mm45ry
///
/// Bit 0 is the most significant bit of byte 0, and only the first `DEPTH`
/// bits are read: two identities differing in bit 4 alone collide at depth 4
/// and coexist at depth 5; two differing in bit 255 alone coexist only at 256.
#[test]
fn positions_are_the_first_depth_bits_most_significant_first() {
    let bit4_clear = leaf(1, 0b0000_0000, 1);
    let bit4_set = leaf(2, 0b0000_1000, 1);
    assert_eq!(Tree::<4>::from_leaves([bit4_clear.clone(), bit4_set.clone()]).err(), Some(TreeError::Collision));
    assert!(Tree::<5>::from_leaves([bit4_clear, bit4_set]).is_ok());

    let mut last = [0u8; 32];
    last[31] = 1;
    let bit255_clear = leaf_at(3, [0u8; 32], 1);
    let bit255_set = leaf_at(4, last, 1);
    assert_eq!(Tree::<255>::from_leaves([bit255_clear.clone(), bit255_set.clone()]).err(), Some(TreeError::Collision));
    assert!(Tree::<256>::from_leaves([bit255_clear, bit255_set]).is_ok());

    // At depth 1 the most significant bit of byte 0 says right.
    let right = Tree::<1>::root_of([leaf(5, 0x80, 1)]).expect("root");
    assert_eq!(right, classic_root_by_slots(1, &[leaf(5, 0x80, 1)]));
    assert_ne!(right, Tree::<1>::root_of([leaf(5, 0x7f, 1)]).expect("root"));
}

/// verifies: LLR-yuyn8f
#[test]
fn the_empty_subtree_hashes_and_their_bound() {
    let tree = Tree::<8>::new();
    assert_eq!(tree.empty_subtree_hash(0), Ok(TestHasher::empty_leaf()));
    for level in 1..=8 {
        let below = tree.empty_subtree_hash(level - 1).expect("in range");
        assert_eq!(tree.empty_subtree_hash(level), Ok(TestHasher::node(&below, &below)));
    }
    assert_eq!(tree.empty_subtree_hash(9), Err(TreeError::LevelOutOfRange));
    assert_eq!(tree.empty_subtree_hash(usize::MAX), Err(TreeError::LevelOutOfRange));
    assert_eq!(tree.root(), Ok(empty_at(8)));
    assert_eq!(Tree::<256>::new().root(), Ok(empty_at(256)));
}

fn spread(count: u32) -> Vec<TestLeaf> {
    (0..count).map(|name| leaf(name, (name as u8).wrapping_mul(37), name * 3)).collect()
}

/// verifies: LLR-kep2ju
///
/// The root equals the classic tree's, computed by both oracles, for sets of
/// 0 to 12 leaves at depth 8, adjacent and far apart.
#[test]
fn the_root_is_the_classic_trees_at_depth_8() {
    for count in 0..=12 {
        let leaves = spread(count);
        let refs: Vec<&TestLeaf> = leaves.iter().collect();
        let tree = calculated::<8>(&leaves);
        assert_eq!(tree.root(), Ok(classic_root_by_slots(8, &leaves)), "{count} leaves");
        assert_eq!(tree.root(), Ok(classic_root(8, 0, &refs)), "{count} leaves");
    }
    let neighbours = [leaf(1, 0b1010_1010, 1), leaf(2, 0b1010_1011, 2)];
    assert_eq!(calculated::<8>(&neighbours).root(), Ok(classic_root_by_slots(8, &neighbours)));
}

/// verifies: LLR-kep2ju
#[test]
fn the_root_is_the_classic_trees_at_depth_256() {
    let mut low = [0u8; 32];
    low[31] = 0b10;
    let mut high = [0xffu8; 32];
    high[0] = 0x7f;
    let leaves = [leaf_at(1, [0u8; 32], 1), leaf_at(2, low, 2), leaf_at(3, high, 3), leaf_at(4, [0xff; 32], 4)];
    for count in 0..=leaves.len() {
        let refs: Vec<&TestLeaf> = leaves[..count].iter().collect();
        assert_eq!(calculated::<256>(&leaves[..count]).root(), Ok(classic_root(256, 0, &refs)), "{count} leaves");
    }
}

/// verifies: LLR-kep2ju, LLR-ycf27m
///
/// Removing leaves gives the classic tree of the remaining set: a lone leaf
/// beside an emptied subtree rises, and the root follows.
#[test]
fn removal_gives_the_classic_tree_of_what_remains() {
    let leaves = spread(10);
    let mut tree = calculated::<8>(&leaves);
    for removed in 0..10 {
        tree = tree.remove(&leaves[removed].id).expect("held");
        // The last removal leaves no node at all: an empty tree is calculated.
        if !tree.is_calculated() {
            tree = tree.calculate().expect("changed");
        }
        assert_eq!(tree.root(), Ok(classic_root_by_slots(8, &leaves[removed + 1..])), "after {removed}");
    }
    assert_eq!(tree.root(), Ok(empty_at(8)));
}

/// verifies: LLR-8uem8c
#[test]
fn insert_and_remove_leave_the_receiver_unchanged() {
    let before = calculated::<8>(&spread(4));
    let root = before.root();
    let added = before.insert(leaf(99, 0x99, 9)).expect("insert");
    assert_eq!(before.root(), root);
    assert!(!before.contains(&id(99, 0x99)));
    assert!(added.contains(&id(99, 0x99)));
    let removed = before.remove(&spread(4)[0].id).expect("remove");
    assert_eq!(before.root(), root);
    assert!(before.contains(&spread(4)[0].id));
    assert!(!removed.contains(&spread(4)[0].id));
}

/// verifies: LLR-ycf27m, REQ-6cej5z
#[test]
fn insert_refuses_a_collision_and_replaces_the_same_identity() {
    let tree = calculated::<8>(&[leaf(1, 0x42, 1)]);
    // Another identity at the same position: refused, nothing returned.
    assert_eq!(tree.insert(leaf(2, 0x42, 1)).err(), Some(TreeError::Collision));
    let replaced = tree.insert(leaf(1, 0x42, 2)).expect("replace").calculate().expect("changed");
    assert_eq!(replaced.get(&id(1, 0x42)), Some(&leaf(1, 0x42, 2)));
    assert_eq!(replaced.root(), Ok(classic_root_by_slots(8, &[leaf(1, 0x42, 2)])));
    // A collision deeper in the tree, below an internal node.
    let two = calculated::<8>(&[leaf(1, 0x40, 1), leaf(2, 0x41, 1)]);
    assert_eq!(two.insert(leaf(3, 0x41, 1)).err(), Some(TreeError::Collision));
}

/// verifies: LLR-ycf27m
#[test]
fn remove_refuses_an_identity_not_held() {
    assert_eq!(Tree::<8>::new().remove(&id(1, 0)).err(), Some(TreeError::LeafNotHeld));
    let tree = calculated::<8>(&[leaf(1, 0x42, 1), leaf(2, 0x43, 1)]);
    // Another identity at a held position.
    assert_eq!(tree.remove(&id(3, 0x42)).err(), Some(TreeError::LeafNotHeld));
    // A position in an empty subtree.
    assert_eq!(tree.remove(&id(4, 0x80)).err(), Some(TreeError::LeafNotHeld));
}

/// verifies: LLR-3y5pq6
#[test]
fn hashes_are_write_once_and_the_root_waits_for_them() {
    let empty = Tree::<8>::new();
    assert!(empty.is_calculated());
    assert_eq!(empty.calculate().err(), Some(TreeError::HashesAlreadyCalculated));

    let pending = empty.insert(leaf(1, 1, 1)).expect("insert");
    assert!(!pending.is_calculated());
    assert_eq!(pending.root(), Err(TreeError::HashesNotCalculated));
    let done = pending.calculate().expect("calculate");
    assert!(done.is_calculated());
    assert!(done.root().is_ok());
    assert_eq!(done.calculate().err(), Some(TreeError::HashesAlreadyCalculated));
}

/// verifies: LLR-9qncuc
#[test]
fn from_leaves_builds_a_calculated_tree_or_refuses_a_collision() {
    let leaves = spread(6);
    let built = Tree::<8>::from_leaves(leaves.clone()).expect("no collision");
    assert!(built.is_calculated());
    let incremental = leaves
        .iter()
        .try_fold(Tree::<8>::new(), |tree, leaf| tree.insert(leaf.clone()))
        .expect("no collision")
        .calculate()
        .expect("changed");
    assert_eq!(built.root(), incremental.root());
    assert_eq!(Tree::<8>::root_of(leaves.clone()), built.root());
    assert_eq!(Tree::<8>::from_leaves(Vec::new()).expect("empty").root(), Ok(empty_at(8)));
    assert_eq!(
        Tree::<8>::from_leaves([leaf(1, 7, 1), leaf(2, 7, 1)]).err(),
        Some(TreeError::Collision)
    );
    assert_eq!(Tree::<8>::root_of([leaf(1, 7, 1), leaf(2, 7, 1)]), Err(TreeError::Collision));
}

/// verifies: LLR-639mz9
#[test]
fn get_contains_and_leaves_read_without_changing_the_tree() {
    let leaves = spread(6);
    let tree = calculated::<8>(&leaves);
    let root = tree.root();
    for leaf in &leaves {
        assert_eq!(tree.get(&leaf.id), Some(leaf));
        assert!(tree.contains(&leaf.id));
    }
    assert_eq!(tree.get(&id(99, leaves[0].id.position[0])), None, "another identity at a held position");
    assert_eq!(tree.get(&id(99, 0x01)), None, "an empty subtree");
    let mut by_position = leaves.clone();
    by_position.sort_by_key(|leaf| leaf.id.position);
    let listed: Vec<TestLeaf> = tree.leaves().into_iter().cloned().collect();
    assert_eq!(listed, by_position);
    assert!(Tree::<8>::new().leaves().is_empty());
    assert_eq!(tree.root(), root);
}

fn arb_leaves() -> impl Strategy<Value = Vec<TestLeaf>> {
    proptest::collection::btree_map(any::<u8>(), any::<u32>(), 0..24).prop_map(|by_position| {
        by_position
            .into_iter()
            .enumerate()
            .map(|(name, (first, content))| leaf(name as u32, first, content))
            .collect()
    })
}

proptest! {
    /// verifies: LLR-3zffc2, LLR-kep2ju
    #[test]
    fn the_root_depends_on_the_leaf_set_alone(leaves in arb_leaves(), extra in arb_leaves(), seed in any::<u64>()) {
        let forward = calculated::<8>(&leaves);
        let mut shuffled = leaves.clone();
        let len = shuffled.len().max(1);
        shuffled.rotate_left((seed as usize) % len);
        shuffled.reverse();
        let backward = calculated::<8>(&shuffled);
        prop_assert_eq!(forward.root(), backward.root());
        prop_assert_eq!(forward.root(), Ok(classic_root_by_slots(8, &leaves)));
        // Inserting and then removing leaves at free positions returns to the same root.
        let held: std::collections::BTreeSet<u8> = leaves.iter().map(|leaf| leaf.id.position[0]).collect();
        let free: Vec<TestLeaf> = extra.into_iter().filter(|leaf| !held.contains(&leaf.id.position[0]))
            .map(|leaf| TestLeaf { id: common::Id { name: leaf.id.name + 1000, ..leaf.id }, ..leaf }).collect();
        let mut tree = forward.clone();
        for leaf in &free { tree = tree.insert(leaf.clone()).expect("free position"); }
        for leaf in &free { tree = tree.remove(&leaf.id).expect("held"); }
        if !tree.is_calculated() { tree = tree.calculate().expect("pending"); }
        prop_assert_eq!(tree.root(), forward.root());
    }
}
