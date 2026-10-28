//! The tree changes between two trees: `changes_between`.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use common::{calculated, leaf, Id, TestLeaf, Tree};
use proptest::prelude::*;

fn identity_names(removed: &[Id]) -> Vec<u32> {
    removed.iter().map(|id| id.name).collect()
}

fn names(upserted: &[Arc<TestLeaf>]) -> Vec<u32> {
    upserted.iter().map(|leaf| leaf.id.name).collect()
}

/// verifies: LLR-u2r4vj
#[test]
fn changes_between_reports_removed_added_and_replaced_in_position_order() {
    let before = calculated::<8>(&[leaf(1, 0x10, 1), leaf(2, 0x20, 1), leaf(3, 0x30, 1), leaf(4, 0xf0, 1)]);
    let after = calculated::<8>(&[leaf(2, 0x20, 2), leaf(4, 0xf0, 1), leaf(5, 0x05, 1), leaf(6, 0x31, 1)]);
    let (removed, upserted) = Tree::<8>::changes_between(&before, &after);
    assert_eq!(identity_names(&removed), [1, 3]);
    assert_eq!(names(&upserted), [5, 2, 6]);
    assert_eq!(upserted[1].content, 2, "a replaced leaf is reported with its new content");
}

/// verifies: LLR-u2r4vj
#[test]
fn equal_trees_differ_in_nothing() {
    let tree = calculated::<8>(&[leaf(1, 0x10, 1), leaf(2, 0x20, 1)]);
    assert_eq!(Tree::<8>::changes_between(&tree, &tree), (vec![], vec![]));
    let rebuilt = calculated::<8>(&[leaf(2, 0x20, 1), leaf(1, 0x10, 1)]);
    assert_eq!(Tree::<8>::changes_between(&tree, &rebuilt), (vec![], vec![]));
    let empty = Tree::<8>::new();
    assert_eq!(Tree::<8>::changes_between(&empty, &empty), (vec![], vec![]));
}

/// verifies: LLR-u2r4vj
///
/// The walk accepts an uncalculated tree, and a compact leaf facing an
/// internal node is lowered and compared, not reported whole.
#[test]
fn changes_between_walks_uncalculated_trees_and_lowers_compact_leaves() {
    let before = calculated::<8>(&[leaf(1, 0b0000_0000, 1)]);
    let after = before.insert(leaf(2, 0b0000_0001, 1)).expect("insert");
    assert!(!after.is_calculated());
    let (removed, upserted) = Tree::<8>::changes_between(&before, &after);
    assert!(removed.is_empty());
    assert_eq!(names(&upserted), [2]);
    let (removed, upserted) = Tree::<8>::changes_between(&after, &before);
    assert_eq!(identity_names(&removed), [2]);
    assert!(upserted.is_empty());
    // The same identity rewritten with equal content is no change.
    let same = before.insert(leaf(1, 0b0000_0000, 1)).expect("replace");
    assert_eq!(Tree::<8>::changes_between(&before, &same), (vec![], vec![]));
    // Two identities at one compact position: one removed, one added.
    let other = Tree::<8>::new().insert(leaf(3, 0b0000_0000, 1)).expect("insert");
    let (removed, upserted) = Tree::<8>::changes_between(&before, &other);
    assert_eq!((identity_names(&removed), names(&upserted)), (vec![1], vec![3]));
}

fn arb_set() -> impl Strategy<Value = BTreeMap<u8, (u32, u32)>> {
    proptest::collection::btree_map(any::<u8>(), (0u32..3, 0u32..3), 0..20)
}

fn tree_of(set: &BTreeMap<u8, (u32, u32)>) -> Tree<8> {
    let leaves: Vec<TestLeaf> = set.iter().map(|(first, (name, content))| leaf(*name, *first, *content)).collect();
    calculated::<8>(&leaves)
}

proptest! {
    /// verifies: LLR-u2r4vj
    ///
    /// Against a map model: removed is every identity of `before` absent from
    /// `after`; upserted every leaf of `after` not in `before` as it is; both
    /// strictly increasing by position.
    #[test]
    fn changes_between_matches_the_set_difference(before in arb_set(), after in arb_set()) {
        let (removed, upserted) = Tree::<8>::changes_between(&tree_of(&before), &tree_of(&after));
        let id_at = |first: &u8, name: &u32| leaf(*name, *first, 0).id;
        let expected_removed: Vec<Id> = before.iter()
            .filter(|(first, (name, _))| after.get(first).is_none_or(|(other, _)| other != name))
            .map(|(first, (name, _))| id_at(first, name))
            .collect();
        let expected_upserted: Vec<TestLeaf> = after.iter()
            .filter(|(first, value)| before.get(first) != Some(value))
            .map(|(first, (name, content))| leaf(*name, *first, *content))
            .collect();
        prop_assert_eq!(removed, expected_removed);
        prop_assert_eq!(upserted.iter().map(|leaf| (**leaf).clone()).collect::<Vec<_>>(), expected_upserted);
    }
}
