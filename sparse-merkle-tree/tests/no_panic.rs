//! Hostile input never panics: decoding arbitrary bytes, verifying what
//! decodes against arbitrary roots, and arbitrary operation sequences on a
//! tree whose positions collide often.

#![allow(clippy::expect_used)]

mod common;

#[cfg(feature = "serde")]
use common::Id;
use common::{id, leaf, TestHasher, TestLeaf, Tree};
use proptest::prelude::*;
#[cfg(feature = "serde")]
use sparse_merkle_tree::{AbsenceProof, InclusionProof, NodeHash};

#[derive(Debug, Clone)]
enum Op {
    Insert(u32, u8, u32),
    Remove(u32, u8),
    Calculate,
    ProveAbsent(u32, u8),
    ProveIncluded(u32, u8),
}

fn arb_op() -> impl Strategy<Value = Op> {
    // Positions differ in the first four bits only, so depth 4 collides often.
    let position = (0u8..16).prop_map(|nibble| nibble << 4);
    prop_oneof![
        3 => (0u32..4, position.clone(), any::<u32>()).prop_map(|(name, position, content)| Op::Insert(name, position, content)),
        2 => (0u32..4, position.clone()).prop_map(|(name, position)| Op::Remove(name, position)),
        2 => Just(Op::Calculate),
        1 => (0u32..4, position.clone()).prop_map(|(name, position)| Op::ProveAbsent(name, position)),
        1 => (0u32..4, position).prop_map(|(name, position)| Op::ProveIncluded(name, position)),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    /// verifies: LLR-smf4zq, LLR-77zy97, REQ-dyjq33
    #[cfg(feature = "serde")]
    #[test]
    fn decoding_and_verifying_any_bytes_does_not_panic(
        bytes in proptest::collection::vec(any::<u8>(), 0..600),
        root in any::<[u8; 32]>(),
        name in any::<u32>(),
        position in any::<[u8; 32]>(),
    ) {
        let identity = Id { name, position };
        let root = NodeHash::new(root);
        if let Ok(proof) = postcard::from_bytes::<AbsenceProof<4>>(&bytes) {
            let _ = proof.verify::<TestLeaf, TestHasher>(&root, &identity);
        }
        if let Ok(proof) = postcard::from_bytes::<AbsenceProof<256>>(&bytes) {
            let _ = proof.verify::<TestLeaf, TestHasher>(&root, &identity);
        }
        if let Ok(proof) = postcard::from_bytes::<InclusionProof<256>>(&bytes) {
            let _ = proof.verify::<TestLeaf, TestHasher>(&root, &TestLeaf { id: identity, content: 0 });
        }
    }

    /// verifies: LLR-smf4zq, LLR-ycf27m, LLR-3y5pq6, REQ-dyjq33
    ///
    /// Every operation returns a result, and every proof a calculated tree
    /// gives verifies against its root.
    #[test]
    fn any_operation_sequence_does_not_panic(ops in proptest::collection::vec(arb_op(), 0..40)) {
        let mut tree = Tree::<4>::new();
        for op in ops {
            match op {
                Op::Insert(name, first, content) => {
                    if let Ok(next) = tree.insert(leaf(name, first, content)) { tree = next; }
                }
                Op::Remove(name, first) => {
                    if let Ok(next) = tree.remove(&id(name, first)) { tree = next; }
                }
                Op::Calculate => {
                    if let Ok(next) = tree.calculate() { tree = next; }
                }
                Op::ProveAbsent(name, first) => {
                    if let (Ok(proof), Ok(root)) = (tree.prove_absent(&id(name, first)), tree.root()) {
                        prop_assert_eq!(proof.verify::<TestLeaf, TestHasher>(&root, &id(name, first)), Ok(()));
                    }
                }
                Op::ProveIncluded(name, first) => {
                    if let (Ok((held, proof)), Ok(root)) = (tree.prove_included(&id(name, first)), tree.root()) {
                        prop_assert_eq!(proof.verify::<TestLeaf, TestHasher>(&root, &*held), Ok(()));
                    }
                }
            }
        }
    }
}
