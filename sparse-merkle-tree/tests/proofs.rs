//! Absence and inclusion proofs: their shape, producing them, building them
//! from parts, and checking them against a root.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod common;

use common::{id, leaf, sample, TestHasher, TestLeaf, Tree};
use sparse_merkle_tree::{
    AbsenceProof, InclusionProof, Leaf, NodeHash, ProofEnding, TreeError, TreeHasher,
};

fn set_bits(map: &[u8]) -> usize {
    map.iter().map(|byte| byte.count_ones() as usize).sum()
}

/// Position bytes that are zero but for the last byte, `last`.
fn position_with_last_byte(last: u8) -> [u8; 32] {
    let mut position = [0; 32];
    position[31] = last;
    position
}

/// verifies: LLR-m5c3vw, REQ-s3unp5
///
/// The proof is a map of DEPTH bits, the siblings it marks, and an ending of
/// hashes only: no identity, no content.
#[test]
fn an_absence_proof_holds_a_map_its_siblings_and_an_ending() {
    let proof = sample().prove_absent(&id(9, 0x00)).expect("absent");
    assert_eq!(proof.map().len(), 1, "8 bits");
    assert_eq!(proof.siblings().len(), set_bits(proof.map()));
    assert_eq!(proof.ending(), &ProofEnding::Empty);
    let deep = Tree::<256>::from_leaves([leaf(1, 0x40, 1)]).expect("one").prove_absent(&id(9, 0x00)).expect("absent");
    assert_eq!(deep.map().len(), 32, "256 bits");
    assert_eq!(deep.siblings().len(), set_bits(deep.map()));
}

/// verifies: LLR-5tww69, LLR-53ezgd, REQ-uy4euz, REQ-ymgg6b
///
/// A position in an empty subtree ends `Empty`; so does a position beside a
/// compact leaf of another identity (the occupant's subtree is a sibling);
/// a position held by a leaf of another identity ends `Occupied` with that
/// leaf's two hashes. Each verifies against the root.
#[test]
fn prove_absent_ends_by_what_the_position_holds() {
    let tree = sample();
    let root = tree.root().expect("calculated");
    let empty = tree.prove_absent(&id(9, 0x00)).expect("absent");
    assert_eq!(empty.ending(), &ProofEnding::Empty);
    assert_eq!(empty.verify::<TestLeaf, TestHasher>(&root, &id(9, 0x00)), Ok(()));
    // Beside the compact leaf at 0xc0: 0xc1 differs only in the last bit.
    let beside = tree.prove_absent(&id(9, 0xc1)).expect("absent");
    assert_eq!(beside.ending(), &ProofEnding::Empty);
    assert_eq!(beside.verify::<TestLeaf, TestHasher>(&root, &id(9, 0xc1)), Ok(()));
    let occupied = tree.prove_absent(&id(9, 0x41)).expect("absent");
    assert_eq!(
        occupied.ending(),
        &ProofEnding::Occupied {
            identity_hash: TestLeaf::identity_hash(&id(2, 0x41)),
            content_hash: leaf(2, 0x41, 2).content_hash(),
        }
    );
    assert_eq!(occupied.verify::<TestLeaf, TestHasher>(&root, &id(9, 0x41)), Ok(()));
    let none = Tree::<8>::new();
    let proof = none.prove_absent(&id(9, 0x00)).expect("absent");
    assert_eq!(proof.verify::<TestLeaf, TestHasher>(&none.root().expect("root"), &id(9, 0x00)), Ok(()));
    assert!(proof.siblings().is_empty());
}

/// verifies: LLR-5tww69
#[test]
fn prove_absent_refuses_a_held_leaf_and_an_uncalculated_tree() {
    let tree = sample();
    assert_eq!(tree.prove_absent(&id(2, 0x41)).err(), Some(TreeError::LeafHeld));
    let pending = tree.insert(leaf(4, 0x80, 4)).expect("insert");
    assert_eq!(pending.prove_absent(&id(9, 0x00)).err(), Some(TreeError::HashesNotCalculated));
}

/// verifies: LLR-88rm99, LLR-tz3nfz, LLR-c7bdee, REQ-3szczq, REQ-fmx9e7
#[test]
fn prove_included_returns_the_leaf_and_a_proof_that_verifies() {
    let tree = sample();
    let root = tree.root().expect("calculated");
    for held in [leaf(1, 0x40, 1), leaf(2, 0x41, 2), leaf(3, 0xc0, 3)] {
        let (found, proof) = tree.prove_included(&held.id).expect("held");
        assert_eq!(*found, held);
        assert_eq!(proof.siblings().len(), set_bits(proof.map()));
        assert_eq!(proof.verify::<TestLeaf, TestHasher>(&root, &held), Ok(()));
    }
}

/// verifies: LLR-88rm99
#[test]
fn prove_included_refuses_an_absent_leaf_and_an_uncalculated_tree() {
    let tree = sample();
    assert_eq!(tree.prove_included(&id(9, 0x41)).err(), Some(TreeError::LeafNotHeld));
    assert_eq!(tree.prove_included(&id(9, 0x00)).err(), Some(TreeError::LeafNotHeld));
    let pending = tree.insert(leaf(4, 0x80, 4)).expect("insert");
    assert_eq!(pending.prove_included(&id(1, 0x40)).err(), Some(TreeError::HashesNotCalculated));
}

/// verifies: LLR-c7bdee, REQ-fmx9e7
///
/// An inclusion proof accepts its leaf only: another content, another
/// identity, another root are each a mismatch.
#[test]
fn an_inclusion_proof_refuses_any_other_leaf_or_root() {
    let tree = sample();
    let root = tree.root().expect("calculated");
    let (_, proof) = tree.prove_included(&id(1, 0x40)).expect("held");
    assert_eq!(proof.verify::<TestLeaf, TestHasher>(&root, &leaf(1, 0x40, 99)), Err(TreeError::ProofRootMismatch));
    assert_eq!(proof.verify::<TestLeaf, TestHasher>(&root, &leaf(7, 0x40, 1)), Err(TreeError::ProofRootMismatch));
    assert_eq!(
        proof.verify::<TestLeaf, TestHasher>(&NodeHash::new([0; 32]), &leaf(1, 0x40, 1)),
        Err(TreeError::ProofRootMismatch)
    );
}

/// verifies: LLR-53ezgd, REQ-ymgg6b
///
/// The root check comes first; a resolving proof ending in the asked
/// identity's own leaf is `ProofLeafHeld`.
#[test]
fn verify_absent_checks_the_root_first_then_the_ending() {
    let tree = sample();
    let root = tree.root().expect("calculated");
    let occupied = tree.prove_absent(&id(9, 0x41)).expect("absent");
    // Asked about the occupant itself: resolves, then refused as held.
    assert_eq!(occupied.verify::<TestLeaf, TestHasher>(&root, &id(2, 0x41)), Err(TreeError::ProofLeafHeld));
    // The same with a wrong root: the root check speaks first.
    assert_eq!(
        occupied.verify::<TestLeaf, TestHasher>(&NodeHash::new([0; 32]), &id(2, 0x41)),
        Err(TreeError::ProofRootMismatch)
    );
    // A proof for one position does not resolve for another.
    let empty = tree.prove_absent(&id(9, 0x00)).expect("absent");
    assert_eq!(empty.verify::<TestLeaf, TestHasher>(&root, &id(9, 0x80)), Err(TreeError::ProofRootMismatch));
    // An Empty ending where a leaf is held does not resolve.
    let forged = AbsenceProof::<8>::from_parts(
        occupied.map().to_vec(),
        occupied.siblings().to_vec(),
        ProofEnding::Empty,
    )
    .expect("well formed");
    assert_eq!(forged.verify::<TestLeaf, TestHasher>(&root, &id(2, 0x41)), Err(TreeError::ProofRootMismatch));
    // A tampered sibling does not resolve.
    let mut siblings = empty.siblings().to_vec();
    siblings[0] = NodeHash::new([7; 32]);
    let tampered = AbsenceProof::<8>::from_parts(empty.map().to_vec(), siblings, ProofEnding::Empty).expect("well formed");
    assert_eq!(tampered.verify::<TestLeaf, TestHasher>(&root, &id(9, 0x00)), Err(TreeError::ProofRootMismatch));
}

/// verifies: LLR-77zy97, REQ-dyjq33
#[test]
fn from_parts_refuses_a_map_or_siblings_that_do_not_fit_the_depth() {
    let one = vec![NodeHash::new([1; 32])];
    // A map of the wrong length.
    assert_eq!(AbsenceProof::<8>::from_parts(vec![], vec![], ProofEnding::Empty).err(), Some(TreeError::ProofMalformed));
    assert_eq!(AbsenceProof::<8>::from_parts(vec![1, 0], one.clone(), ProofEnding::Empty).err(), Some(TreeError::ProofMalformed));
    // More or fewer siblings than marked.
    assert_eq!(AbsenceProof::<8>::from_parts(vec![0b11], one.clone(), ProofEnding::Empty).err(), Some(TreeError::ProofMalformed));
    assert_eq!(AbsenceProof::<8>::from_parts(vec![0], one.clone(), ProofEnding::Empty).err(), Some(TreeError::ProofMalformed));
    // A bit marked at a level beyond the depth (depth 4: bits 4..7 of the byte).
    assert_eq!(AbsenceProof::<4>::from_parts(vec![0b1_0000], one.clone(), ProofEnding::Empty).err(), Some(TreeError::ProofMalformed));
    assert!(AbsenceProof::<4>::from_parts(vec![0b1000], one.clone(), ProofEnding::Empty).is_ok());
    assert_eq!(InclusionProof::<8>::from_parts(vec![0b11], one.clone()).err(), Some(TreeError::ProofMalformed));
    assert_eq!(InclusionProof::<4>::from_parts(vec![0b1_0000], one.clone()).err(), Some(TreeError::ProofMalformed));
    assert!(InclusionProof::<8>::from_parts(vec![0b1], one).is_ok());
    assert_eq!(InclusionProof::<256>::from_parts(vec![0; 31], vec![]).err(), Some(TreeError::ProofMalformed));
}

/// verifies: LLR-77zy97, REQ-dyjq33
///
/// Every refusal at one depth that is not a multiple of eight, where a map
/// byte has bits beyond the depth: a map of the wrong length, a mark beyond
/// the depth, and siblings not matching the marks.
#[test]
fn a_depth_four_map_is_refused_for_each_malformed_shape() {
    let one = vec![NodeHash::new([1; 32])];
    assert_eq!(AbsenceProof::<4>::from_parts(vec![0, 0], vec![], ProofEnding::Empty).err(), Some(TreeError::ProofMalformed));
    assert_eq!(AbsenceProof::<4>::from_parts(vec![], vec![], ProofEnding::Empty).err(), Some(TreeError::ProofMalformed));
    assert_eq!(AbsenceProof::<4>::from_parts(vec![0b1000_0000], vec![], ProofEnding::Empty).err(), Some(TreeError::ProofMalformed));
    assert_eq!(AbsenceProof::<4>::from_parts(vec![0b11], one.clone(), ProofEnding::Empty).err(), Some(TreeError::ProofMalformed));
    assert_eq!(AbsenceProof::<4>::from_parts(vec![0], one.clone(), ProofEnding::Empty).err(), Some(TreeError::ProofMalformed));
    assert!(AbsenceProof::<4>::from_parts(vec![0b1111], vec![one[0]; 4], ProofEnding::Empty).is_ok());
}

/// verifies: LLR-smf4zq, REQ-dyjq33
#[test]
fn every_refusal_is_its_own_variant_and_message() {
    let variants = [
        TreeError::Collision,
        TreeError::LeafNotHeld,
        TreeError::LeafHeld,
        TreeError::HashesNotCalculated,
        TreeError::HashesAlreadyCalculated,
        TreeError::LevelOutOfRange,
        TreeError::ProofMalformed,
        TreeError::ProofRootMismatch,
        TreeError::ProofLeafHeld,
    ];
    for (index, variant) in variants.iter().enumerate() {
        for other in &variants[index + 1..] {
            assert_ne!(variant, other);
            assert_ne!(variant.to_string(), other.to_string());
        }
    }
}

/// verifies: LLR-5tww69, LLR-53ezgd, LLR-88rm99, LLR-c7bdee
///
/// At depth 256, every held leaf proves included and every probe absent,
/// against the root, at the lowest and highest positions too.
#[test]
fn proofs_verify_at_depth_256_at_the_edges() {
    let leaves = [
        common::leaf_at(1, [0; 32], 1),
        common::leaf_at(2, [0xff; 32], 2),
        common::leaf_at(3, position_with_last_byte(1), 3),
    ];
    let tree = Tree::<256>::from_leaves(leaves.clone()).expect("distinct");
    let root = tree.root().expect("calculated");
    for held in &leaves {
        let (_, proof) = tree.prove_included(&held.id).expect("held");
        assert_eq!(proof.verify::<TestLeaf, TestHasher>(&root, held), Ok(()));
    }
    for probe in [common::Id { name: 9, position: [0; 32] }, common::Id { name: 9, position: [0x80; 32] }, common::Id { name: 9, position: position_with_last_byte(2) }] {
        let proof = tree.prove_absent(&probe).expect("absent");
        assert_eq!(proof.verify::<TestLeaf, TestHasher>(&root, &probe), Ok(()));
    }
    assert_eq!(TestHasher::empty_leaf(), tree.empty_subtree_hash(0).expect("level 0"));
}
