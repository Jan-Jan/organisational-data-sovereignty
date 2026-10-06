#![cfg(feature = "test-support")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The Envelope (SDD-kk2y3e): what it binds together and how its Change set
//! is decoded.
//!
//! Relocated from the `#[cfg(test)]` module in `org-node/src/envelope.rs` so
//! the `verifies:` annotations sit under `test_paths`.
//!
//! *Merged 2026-10-05 into worktree-person-shared-types.* The Envelope carries
//! no signature (owner ruling; REQ-ag6kqm, amended in place): the chain's root
//! at a newer epoch decides, and nothing about the sender is checked. Master's
//! cases for the signed transcript and for a signature failing when the org,
//! sequence number, delta bytes or key change test code that no longer
//! exists. LLR-p8uu47, LLR-e7s4ye and LLR-pzde8b are amended in place to the
//! unsigned wire form and what `build` transmits, and the tests below verify
//! them as amended.

use org_node::ids::OrgId;
use org_node::test_fixtures::admit_member_delta;
use org_node::{Envelope, MemberSeed, OrgNodeError, SequenceNumber};

const ORG: [u8; 20] = [5u8; 20];
const SEQ: u64 = 7;

fn built() -> Envelope {
    let admin = MemberSeed::from([1u8; 32]).x25519_keypair();
    let (delta, _) = admit_member_delta(&admin);
    Envelope::build(OrgId::new(ORG), SequenceNumber::new(SEQ), &delta).unwrap()
}

// Relocated from `src/envelope.rs` (`build_binds_org_and_sequence_to_the_encoded_delta`).
// verifies: LLR-p8uu47
#[test]
fn build_binds_org_and_sequence_to_the_encoded_delta() {
    let admin = MemberSeed::from([1u8; 32]).x25519_keypair();
    let (delta, _) = admit_member_delta(&admin);
    let env = Envelope::build(OrgId::new(ORG), SequenceNumber::new(SEQ), &delta).unwrap();
    assert_eq!(env.org_id, OrgId::new(ORG));
    assert_eq!(env.parent_seq, SequenceNumber::new(SEQ));
    assert_eq!(env.delta_bytes, postcard::to_allocvec(&delta).unwrap());
}

// Was `build_signs_the_delta_bytes_it_transmits` (REQ-ag6kqm, LLR-p8uu47).
// The canonical-encoding half is kept; the signature half has nothing to
// check. LLR-p8uu47, as amended, states the half that is kept.
// verifies: LLR-p8uu47
#[test]
fn build_transmits_the_canonical_encoding_of_the_delta() {
    let env = built();
    // The transmitted bytes are the canonical encoding of the delta they
    // decode to: re-encoding what came out reproduces them exactly. `Delta`
    // has no `PartialEq`, so re-encoding is how the round trip is observed.
    let decoded = env.decode_delta().expect("the transmitted bytes must decode");
    let re_encoded = postcard::to_allocvec(&decoded).expect("a decoded delta must re-encode");
    assert_eq!(env.delta_bytes, re_encoded, "the encoding must be canonical");
}

// verifies: REQ-9g6as6, LLR-v2y6sw
#[test]
fn decode_delta_refuses_bytes_that_are_not_a_delta() {
    let mut env = built();
    env.delta_bytes = b"not a postcard delta at all".to_vec();
    assert_eq!(env.decode_delta().unwrap_err(), OrgNodeError::MalformedDelta);
}

// Relocated from `src/envelope.rs`. Since 2026-10-05 it also reads the bytes
// in order, not only their count, because the amended wire-form item states
// the order.
// verifies: LLR-e7s4ye, LLR-pzde8b
#[test]
fn the_wire_form_has_no_signature_field() {
    // OrgId is `[u8; 20]` (serde tuple, no length prefix) ‖ varint
    // parent_seq ‖ varint len ‖ bytes: nothing after.
    let env = Envelope { org_id: OrgId::new(ORG), parent_seq: SequenceNumber::new(1), delta_bytes: vec![9, 9] };
    let bytes = postcard::to_allocvec(&env).unwrap();
    assert_eq!(bytes.len(), 20 + 1 + 1 + 2);
    assert_eq!(&bytes[..20], &ORG, "the Organisation identifier comes first");
    assert_eq!(bytes[20], 1, "then the Sequence number");
    assert_eq!(&bytes[21..], &[2, 9, 9], "then the Change set bytes, and nothing after");
}
