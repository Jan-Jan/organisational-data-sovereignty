#![cfg(feature = "test-support")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The signed delta envelope (SDD-kk2y3e): what the transcript binds together
//! and what breaks when any of it is altered.
//!
//! Relocated from the `#[cfg(test)]` module in `org-node/src/envelope.rs` so
//! the `verifies:` annotations sit under `test_paths`. The transcript-layout
//! case is new: `transcript` is private, so the layout is pinned by signing
//! the bytes the requirement names and comparing against what `build`
//! produced.

use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::test_fixtures::admit_member_delta;
use org_node::{OrgNodeError, SignedDeltaEnvelope};

const ORG: [u8; 20] = [5u8; 20];
const SEQ: u64 = 7;

fn built() -> (SigningKeypair, SignedDeltaEnvelope) {
    let admin = SigningKeypair::from_seed([1u8; 32]);
    let (delta, _) = admit_member_delta(&admin);
    let env = SignedDeltaEnvelope::build(OrgId::new(ORG), SEQ, &delta, &admin).unwrap();
    (admin, env)
}

// verifies: REQ-ag6kqm, LLR-e7s4ye
#[test]
fn the_signed_transcript_is_org_then_seq_little_endian_then_delta() {
    let (admin, env) = built();

    // Build the transcript the requirement names, independently of the module.
    let mut expected = Vec::new();
    expected.extend_from_slice(&ORG);
    expected.extend_from_slice(&SEQ.to_le_bytes());
    expected.extend_from_slice(&env.delta_bytes);
    assert_eq!(expected.len(), 20 + 8 + env.delta_bytes.len());

    assert_eq!(
        env.signature,
        admin.sign(&expected).to_bytes(),
        "the envelope's signature must be exactly a signature over org ‖ seq_le ‖ delta"
    );
}

// verifies: REQ-ag6kqm, LLR-e7s4ye
#[test]
fn the_sequence_number_is_little_endian_in_the_transcript() {
    let (admin, env) = built();
    // Big-endian for the same value is a different transcript, so a signature
    // over it must not be the one `build` produced.
    let mut big_endian = Vec::new();
    big_endian.extend_from_slice(&ORG);
    big_endian.extend_from_slice(&SEQ.to_be_bytes());
    big_endian.extend_from_slice(&env.delta_bytes);
    assert_ne!(env.signature, admin.sign(&big_endian).to_bytes());
}

// verifies: REQ-ag6kqm, LLR-p8uu47
#[test]
fn build_signs_the_delta_bytes_it_transmits() {
    let (admin, env) = built();
    // The transmitted bytes are the canonical encoding of the delta they
    // decode to: re-encoding what came out reproduces them exactly. Restored
    // 2026-10-04 after review round 1 — the relocation of
    // `decode_delta_round_trips` out of `src/envelope.rs` had reduced this to
    // `decode_delta().is_ok()`, which cannot see a lossy or non-canonical
    // encoding. `Delta` has no `PartialEq`, so re-encoding is how the round
    // trip is observed, exactly as the original did it.
    let decoded = env.decode_delta().expect("the transmitted bytes must decode");
    let re_encoded = postcard::to_allocvec(&decoded).expect("a decoded delta must re-encode");
    assert_eq!(env.delta_bytes, re_encoded, "the encoding must be canonical");

    // And the signature verifies over exactly those transmitted bytes.
    assert!(env.verify_signature(&admin.verifying_key()));
}

// verifies: REQ-9g6as6, LLR-v2y6sw
#[test]
fn decode_delta_refuses_bytes_that_are_not_a_delta() {
    let (_, mut env) = built();
    env.delta_bytes = b"not a postcard delta at all".to_vec();
    assert_eq!(env.decode_delta().unwrap_err(), OrgNodeError::MalformedDelta);
}

// verifies: REQ-ag6kqm, REQ-gju89b, LLR-ybn5pr
#[test]
fn altering_the_organisation_identifier_breaks_the_signature() {
    let (admin, mut env) = built();
    assert!(env.verify_signature(&admin.verifying_key()));
    env.org_id = OrgId::new([6u8; 20]);
    assert!(!env.verify_signature(&admin.verifying_key()));
}

// verifies: REQ-ag6kqm, LLR-cs4mpb
#[test]
fn altering_the_sequence_number_breaks_the_signature() {
    let (admin, mut env) = built();
    env.parent_seq = SEQ + 1;
    assert!(!env.verify_signature(&admin.verifying_key()));
}

// verifies: REQ-ag6kqm, LLR-9sknpa
#[test]
fn altering_the_delta_bytes_breaks_the_signature() {
    let (admin, mut env) = built();
    // Restored 2026-10-04 after review round 1: without this guard, a fixture
    // that produced an empty delta would make the mutation below a no-op and
    // the assertion vacuous.
    assert!(!env.delta_bytes.is_empty(), "the admit delta must be non-empty");
    env.delta_bytes[0] ^= 0xff;
    assert!(!env.verify_signature(&admin.verifying_key()));
}

// verifies: REQ-ag6kqm, LLR-pzde8b
#[test]
fn the_signature_does_not_verify_under_another_members_key() {
    let (_, env) = built();
    let stranger = SigningKeypair::from_seed([2u8; 32]);
    assert!(!env.verify_signature(&stranger.verifying_key()));
}
