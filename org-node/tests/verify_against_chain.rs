#![cfg(feature = "test-support")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! verify-against-chain: the receive-side commit rule (verify.rs) and the
//! replay guard it leans on (sequence.rs), tested at the crate's public
//! interface. Nine of these tests were relocated from the `#[cfg(test)]`
//! modules of `src/verify.rs` and `src/sequence.rs` and annotated; the rest
//! are the abnormal-input cases the org-node risk analysis added. What the
//! fix rounds added to the relocated bodies is recorded in
//! `docs/plans/2026-09-09-org-node-risk-analysis.md`.

use org_members::RootHash;
use org_node::chain::{MockChain, OrgState};
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::sequence::SeqGuard;
use org_node::test_fixtures::{admin_device, admit_member_delta, genesis_trie, Trie};
use org_node::verify::{verify_envelope_against_chain, VerifyContext};
use org_node::{OrgNodeError, SignedDeltaEnvelope};

fn setup() -> (SigningKeypair, OrgId, Trie, SignedDeltaEnvelope, RootHash) {
    let admin = SigningKeypair::from_seed([1u8; 32]);
    let local = genesis_trie(&admin, &admin_device()); // receiver's mirror (epoch 1 state)
    // NOTE: admit_member_delta builds its own genesis internally from the same
    // admin and admin_device(); both genesis tries agree by construction
    // (deterministic fixtures).
    let (delta, new_trie) = admit_member_delta(&admin);
    let org = OrgId::new([5u8; 20]);
    let env = SignedDeltaEnvelope::build(org, 2, &delta, &admin).unwrap();
    let new_root = new_trie.root_hash().unwrap();
    (admin, org, local, env, new_root)
}

/// The receiver's context: expects `org`, trusts `author`, has committed
/// `parent_seq` 1 at epoch 1.
fn ctx<'a>(org: OrgId, author: &'a ed25519_dalek::VerifyingKey) -> VerifyContext<'a> {
    VerifyContext {
        expected_org_id: org,
        author_member_key: author,
        seq_guard: SeqGuard::from_last_seen(1),
        last_committed_epoch: 1,
    }
}

/// A chain whose state for `org` is `root` at `epoch`.
fn chain_at(org: OrgId, root: RootHash, epoch: u64) -> MockChain {
    let mut chain = MockChain::new();
    chain.set(org, OrgState { root_hash: root, org_pub_key: [0u8; 32], epoch });
    chain
}

/// Transcript the envelope signs: org_id ‖ parent_seq (LE) ‖ delta_bytes.
fn sign_over(signer: &SigningKeypair, org: OrgId, seq: u64, delta_bytes: &[u8]) -> [u8; 64] {
    let mut t = Vec::new();
    t.extend_from_slice(org.as_bytes());
    t.extend_from_slice(&seq.to_le_bytes());
    t.extend_from_slice(delta_bytes);
    signer.sign(&t).to_bytes()
}

// ---- relocated from src/verify.rs ------------------------------------------

// verifies: REQ-nhe2zu
#[test]
fn happy_path_commits_when_root_matches_chain() {
    let (admin, org, local, env, new_root) = setup();
    let chain = chain_at(org, new_root, 2);
    let vk = admin.verifying_key();
    let ctx = ctx(org, &vk);
    let out = verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap();
    assert_eq!(out.epoch, 2);
    assert_eq!(out.seq_guard.last_seen(), 2);
    assert_eq!(out.trie.root_hash().unwrap(), new_root);
}

// verifies: REQ-gju89b
#[test]
fn rejects_wrong_org_id() {
    let (admin, _org, local, env, _) = setup();
    let chain = MockChain::new();
    let vk = admin.verifying_key();
    let ctx = ctx(OrgId::new([0xff; 20]), &vk);
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::OrgIdMismatch
    );
}

// verifies: REQ-ag6kqm
#[test]
fn rejects_bad_signature() {
    let (_admin, org, local, env, _) = setup();
    let imposter = SigningKeypair::from_seed([0xaa; 32]);
    let chain = MockChain::new();
    let vk = imposter.verifying_key();
    let ctx = ctx(org, &vk);
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::BadSignature
    );
}

// verifies: REQ-6yu72z
#[test]
fn rejects_stale_seq() {
    let (admin, org, local, env, new_root) = setup();
    let chain = chain_at(org, new_root, 2);
    let vk = admin.verifying_key();
    let ctx = VerifyContext {
        seq_guard: SeqGuard::from_last_seen(2), // env.parent_seq == 2, not > 2
        ..ctx(org, &vk)
    };
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::StaleSeq { got: 2, last_seen: 2 }
    );
}

// verifies: REQ-bvh8v6
#[test]
fn rejects_when_org_absent_from_chain() {
    let (admin, org, local, env, _) = setup();
    let chain = MockChain::new(); // empty
    let vk = admin.verifying_key();
    let ctx = ctx(org, &vk);
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::OrgNotOnChain
    );
}

// REQ-nhe2zu's commit rule is conditional, so withholding the commit when the
// recomputed root does not match the chain's is that requirement's
// abnormal-input case as well as REQ-wp2nyc's rejection.
//
// It is also REQ-mr5abb's last step: the root match is the final check, so an
// envelope rejected here has passed the sequence check and would have had its
// Sequence number committed had the root matched. The mark must not move.
// `verify_envelope_against_chain` takes the context by shared reference and
// returns the advanced guard only inside `Ok`, so the guard to assert on is
// the one this test handed in.
// verifies: REQ-wp2nyc, REQ-nhe2zu, REQ-mr5abb
#[test]
fn rejects_root_mismatch_when_chain_root_differs() {
    let (admin, org, local, env, _new_root) = setup();
    // Attacker-influenced delta but honest chain root that does NOT match.
    let chain = chain_at(org, RootHash::from_bytes([0xde; 32]), 2);
    let vk = admin.verifying_key();
    let ctx = ctx(org, &vk);
    assert_eq!(ctx.seq_guard.last_seen(), 1, "the mark this test hands in");
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::RootMismatch
    );
    assert_eq!(
        ctx.seq_guard.last_seen(),
        1,
        "a rejection at the root match must leave the high-water mark where it was"
    );
}

// verifies: REQ-8gz8bu
#[test]
fn rejects_stale_epoch() {
    let (admin, org, local, env, new_root) = setup();
    let chain = chain_at(org, new_root, 1); // chain epoch 1 is not newer than committed 1
    let vk = admin.verifying_key();
    let ctx = ctx(org, &vk);
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::StaleEpoch { got: 1, last: 1 }
    );
}

// ---- relocated from src/sequence.rs ----------------------------------------

// verifies: REQ-6yu72z
#[test]
fn rejects_equal_and_lower_seq() {
    let g = SeqGuard::from_last_seen(5);
    assert!(g.check(6).is_ok());
    assert_eq!(g.check(5), Err(OrgNodeError::StaleSeq { got: 5, last_seen: 5 }));
    assert_eq!(g.check(4), Err(OrgNodeError::StaleSeq { got: 4, last_seen: 5 }));
}

// verifies: REQ-mr5abb
#[test]
fn advance_moves_high_water_mark_forward_only() {
    let mut g = SeqGuard::new();
    g.advance(3);
    assert_eq!(g.last_seen(), 3);
    g.advance(2); // ignored
    assert_eq!(g.last_seen(), 3);
}

// ---- abnormal input: the cheap checks run before the delta is decoded ------

// verifies: REQ-gju89b
#[test]
fn rejects_wrong_org_before_decoding_delta() {
    // Garbage delta bytes: if the org check did not come first, the error
    // would be MalformedDelta.
    let (admin, org, local, _env, _) = setup();
    let garbage = SignedDeltaEnvelope {
        org_id: org,
        parent_seq: 2,
        delta_bytes: vec![0xff; 16],
        signature: sign_over(&admin, org, 2, &[0xff; 16]),
    };
    let vk = admin.verifying_key();
    let ctx = ctx(OrgId::new([0xee; 20]), &vk);
    assert_eq!(
        verify_envelope_against_chain(&local, &garbage, &ctx, &MockChain::new()).unwrap_err(),
        OrgNodeError::OrgIdMismatch
    );
}

// verifies: REQ-ag6kqm
#[test]
fn rejects_bad_signature_before_decoding_delta() {
    // Garbage delta bytes signed by an imposter, for the expected org: if the
    // signature check did not precede decoding, the error would be
    // MalformedDelta.
    let (admin, org, local, _env, _) = setup();
    let imposter = SigningKeypair::from_seed([0xaa; 32]);
    let garbage = SignedDeltaEnvelope {
        org_id: org,
        parent_seq: 2,
        delta_bytes: vec![0xff; 16],
        signature: sign_over(&imposter, org, 2, &[0xff; 16]),
    };
    let vk = admin.verifying_key();
    let ctx = ctx(org, &vk);
    assert_eq!(
        verify_envelope_against_chain(&local, &garbage, &ctx, &MockChain::new()).unwrap_err(),
        OrgNodeError::BadSignature
    );
}

// verifies: REQ-6yu72z
#[test]
fn rejects_stale_seq_before_decoding_delta() {
    // Garbage delta bytes, honestly signed at parent_seq 1 against a guard
    // that has already seen 1: if the replay check did not precede decoding,
    // the error would be MalformedDelta.
    let (admin, org, local, _env, _) = setup();
    let garbage = SignedDeltaEnvelope {
        org_id: org,
        parent_seq: 1,
        delta_bytes: vec![0xff; 16],
        signature: sign_over(&admin, org, 1, &[0xff; 16]),
    };
    let vk = admin.verifying_key();
    let ctx = ctx(org, &vk); // SeqGuard::from_last_seen(1)
    assert_eq!(
        verify_envelope_against_chain(&local, &garbage, &ctx, &MockChain::new()).unwrap_err(),
        OrgNodeError::StaleSeq { got: 1, last_seen: 1 }
    );
}

// verifies: REQ-mr5abb
#[test]
fn check_does_not_advance_the_mark() {
    // `check` takes `&self`: a rejection (or an acceptance that is later
    // rejected downstream) can never move the high-water mark. The assertion
    // documents the contract the type system enforces.
    let g = SeqGuard::from_last_seen(5);
    g.check(6).unwrap();
    assert_eq!(g.last_seen(), 5);
}
