//! A proof's serde form: (depth, map, siblings[, ending]), every decode
//! routed through `from_parts`, the depth and the bounds checked first.

#![cfg(feature = "serde")]
#![allow(clippy::expect_used)]

mod common;

use common::{calculated, id, leaf, sample, TestHasher, TestLeaf};
use serde_json::json;
use sparse_merkle_tree::{AbsenceProof, InclusionProof, NodeHash, ProofEnding};

fn json_error<T: serde::de::DeserializeOwned + std::fmt::Debug>(json: serde_json::Value) -> String {
    serde_json::from_value::<T>(json).expect_err("refused").to_string()
}

/// verifies: LLR-77zy97
#[test]
fn proofs_round_trip_and_still_verify() {
    let tree = sample();
    let root = tree.root().expect("calculated");
    for probe in [id(9, 0x00), id(9, 0x41), id(9, 0xc1)] {
        let proof = tree.prove_absent(&probe).expect("absent");
        let bytes = postcard::to_allocvec(&proof).expect("encode");
        let back: AbsenceProof<8> = postcard::from_bytes(&bytes).expect("decode");
        assert_eq!(back, proof);
        assert_eq!(back.verify::<TestLeaf, TestHasher>(&root, &probe), Ok(()));
        let text = serde_json::to_string(&proof).expect("json");
        assert_eq!(serde_json::from_str::<AbsenceProof<8>>(&text).expect("json decode"), proof);
    }
    let (held, proof) = tree.prove_included(&id(2, 0x41)).expect("held");
    let bytes = postcard::to_allocvec(&proof).expect("encode");
    let back: InclusionProof<8> = postcard::from_bytes(&bytes).expect("decode");
    assert_eq!(back.verify::<TestLeaf, TestHasher>(&root, &held), Ok(()));
}

/// verifies: LLR-77zy97, REQ-dyjq33
///
/// The form opens with the depth: a depth-4 proof, whose map is one byte as a
/// depth-8 map is, parses at depth 4 only.
#[test]
fn a_proof_parses_at_its_own_depth_only() {
    let four = calculated::<4>(&[leaf(1, 0x40, 1)]).prove_absent(&id(9, 0x00)).expect("absent");
    let bytes = postcard::to_allocvec(&four).expect("encode");
    assert_eq!(bytes[0], 4, "the depth comes first");
    assert!(postcard::from_bytes::<AbsenceProof<4>>(&bytes).is_ok());
    assert_eq!(postcard::from_bytes::<AbsenceProof<8>>(&bytes), Err(postcard::Error::SerdeDeCustom));
    assert_eq!(json_error::<AbsenceProof<8>>(serde_json::to_value(&four).expect("json")), "malformed proof");
    let included = calculated::<4>(&[leaf(1, 0x40, 1)]).prove_included(&id(1, 0x40)).expect("held").1;
    let bytes = postcard::to_allocvec(&included).expect("encode");
    assert_eq!(postcard::from_bytes::<InclusionProof<8>>(&bytes), Err(postcard::Error::SerdeDeCustom));
}

/// verifies: LLR-77zy97, LLR-smf4zq, REQ-dyjq33
///
/// Decoding refuses what `from_parts` refuses, and a map or sibling list
/// longer than the depth allows at the first element past the bound.
#[test]
fn decoding_refuses_malformed_and_oversized_parts() {
    let hash = serde_json::to_value([1u8; 32]).expect("json");
    let empty = json!("Empty");
    assert_eq!(json_error::<AbsenceProof<8>>(json!([8, [3], [hash], empty])), "malformed proof");
    // A map of two bytes at depth 8: refused at the second byte.
    assert_eq!(json_error::<AbsenceProof<8>>(json!([8, [1, 0], [hash], empty])), "malformed proof");
    // Nine siblings at depth 8: refused at the ninth.
    let nine: Vec<_> = (0..9).map(|_| hash.clone()).collect();
    assert_eq!(json_error::<AbsenceProof<8>>(json!([8, [255], nine, empty])), "malformed proof");
    // A missing ending, a missing part.
    assert_eq!(json_error::<AbsenceProof<8>>(json!([8, [0], []])), "malformed proof");
    assert_eq!(json_error::<InclusionProof<8>>(json!([8, [0]])), "malformed proof");
    let ok: AbsenceProof<8> = serde_json::from_value(json!([8, [1], [hash], empty])).expect("well formed");
    assert_eq!(ok.siblings(), &[NodeHash::new([1; 32])]);
    assert_eq!(ok.ending(), &ProofEnding::Empty);
}

/// verifies: LLR-77zy97, REQ-dyjq33
///
/// A value of another shape than the tuple, or a map or sibling list that is
/// not a sequence, is refused.
#[test]
fn decoding_refuses_a_value_that_is_not_the_tuple() {
    for wrong_shape in [json!("proof"), json!(8), json!({ "depth": 8 })] {
        assert!(serde_json::from_value::<AbsenceProof<8>>(wrong_shape.clone()).is_err(), "{wrong_shape}");
        assert!(serde_json::from_value::<InclusionProof<8>>(wrong_shape).is_err());
    }
    assert!(serde_json::from_value::<AbsenceProof<8>>(json!([8, "map", [], "Empty"])).is_err());
    assert!(serde_json::from_value::<InclusionProof<8>>(json!([8, [1], "siblings"])).is_err());
}

/// verifies: LLR-77zy97
///
/// Encoding into a buffer too short for the form fails with the format's
/// error, whichever part it runs out at, and never yields a partial proof.
#[test]
fn encoding_into_a_short_buffer_fails() {
    let tree = sample();
    let absent = tree.prove_absent(&id(9, 0x41)).expect("absent");
    let full = postcard::to_allocvec(&absent).expect("encode");
    for length in 0..full.len() {
        let mut buffer = vec![0u8; length];
        assert!(postcard::to_slice(&absent, &mut buffer).is_err(), "absence proof into {length} bytes");
    }
    let mut buffer = vec![0u8; full.len()];
    assert_eq!(postcard::to_slice(&absent, &mut buffer).expect("fits").len(), full.len());

    let (_, included) = tree.prove_included(&id(2, 0x41)).expect("held");
    let full = postcard::to_allocvec(&included).expect("encode");
    for length in 0..full.len() {
        let mut buffer = vec![0u8; length];
        assert!(postcard::to_slice(&included, &mut buffer).is_err(), "inclusion proof into {length} bytes");
    }
}

/// verifies: LLR-77zy97, REQ-dyjq33
///
/// A declared sibling count of 2^32 - 1 with no siblings following fails
/// without reserving for the declared length.
#[test]
fn a_huge_declared_length_reserves_nothing() {
    // depth 8, map [0x01], then a sibling sequence of length 2^32 - 1.
    let bytes = [8u8, 1, 0x01, 0xff, 0xff, 0xff, 0xff, 0x0f];
    assert!(postcard::from_bytes::<AbsenceProof<8>>(&bytes).is_err());
}
