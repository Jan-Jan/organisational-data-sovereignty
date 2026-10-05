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
use org_node::{MemberSeed};
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::sequence::SeqGuard;
use org_node::test_fixtures::{admin_device, admit_member_delta, genesis_trie, Trie};
use org_node::verify::{verify_envelope_against_chain, VerifyContext};
use org_node::{Epoch, OrgNodeError, OrgPublicKey, SequenceNumber, SignedDeltaEnvelope};

fn setup() -> (SigningKeypair, OrgId, Trie, SignedDeltaEnvelope, RootHash) {
    let admin = MemberSeed::from([1u8; 32]).signing_keypair();
    let local = genesis_trie(&admin, &admin_device()); // receiver's mirror (epoch 1 state)
    // NOTE: admit_member_delta builds its own genesis internally from the same
    // admin and admin_device(); both genesis tries agree by construction
    // (deterministic fixtures).
    let (delta, new_trie) = admit_member_delta(&admin);
    let org = OrgId::new([5u8; 20]);
    let env = SignedDeltaEnvelope::build(org, SequenceNumber::new(2), &delta, &admin).unwrap();
    let new_root = new_trie.root_hash().unwrap();
    (admin, org, local, env, new_root)
}

/// The receiver's context: expects `org`, trusts `author`, has committed
/// `parent_seq` 1 at epoch 1.
fn ctx<'a>(org: OrgId, author: &'a ed25519_dalek::VerifyingKey) -> VerifyContext<'a> {
    VerifyContext {
        expected_org_id: org,
        author_member_key: author,
        seq_guard: SeqGuard::from_last_seen(SequenceNumber::new(1)),
        last_committed_epoch: Epoch::new(1),
    }
}

/// A chain whose state for `org` is `root` at `epoch`.
fn chain_at(org: OrgId, root: RootHash, epoch: Epoch) -> MockChain {
    let mut chain = MockChain::new();
    chain.set(org, OrgState { root_hash: root, org_pub_key: OrgPublicKey::parse(&[0u8; 32]).unwrap(), epoch });
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

// verifies: REQ-nhe2zu, LLR-8n95rf, LLR-d6kvbx, LLR-8hwqru
#[test]
fn happy_path_commits_when_root_matches_chain() {
    let (admin, org, local, env, new_root) = setup();
    let chain = chain_at(org, new_root, Epoch::new(2));
    let vk = admin.verifying_key();
    let ctx = ctx(org, &vk);
    let out = verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap();
    assert_eq!(out.epoch, Epoch::new(2));
    assert_eq!(out.seq_guard.last_seen(), SequenceNumber::new(2));
    assert_eq!(out.trie.root_hash().unwrap(), new_root);
}

// verifies: REQ-gju89b, LLR-4fbuy8
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

// verifies: REQ-ag6kqm, LLR-mcdh85
#[test]
fn rejects_bad_signature() {
    let (_admin, org, local, env, _) = setup();
    let imposter = MemberSeed::from([0xaa; 32]).signing_keypair();
    let chain = MockChain::new();
    let vk = imposter.verifying_key();
    let ctx = ctx(org, &vk);
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::BadSignature
    );
}

// verifies: REQ-6yu72z, LLR-xpbkp5, LLR-wx3php
#[test]
fn rejects_stale_seq() {
    let (admin, org, local, env, new_root) = setup();
    let chain = chain_at(org, new_root, Epoch::new(2));
    let vk = admin.verifying_key();
    let ctx = VerifyContext {
        seq_guard: SeqGuard::from_last_seen(SequenceNumber::new(2)), // env.parent_seq == 2, not > 2
        ..ctx(org, &vk)
    };
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::StaleSeq { got: 2, last_seen: 2 }
    );
}

// verifies: REQ-bvh8v6, LLR-8m99q2, LLR-rm9x4z
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

/// A `ChainReader` whose read FAILS, as distinct from one that reports the
/// Organisation absent. Added 2026-10-04 after review round 2: no reader
/// reachable at this gate could return `Err` — `MockChain::get_org_state`
/// always returns `Ok`, and `ChainOpsReader` wraps a state already read — so
/// the second clause of LLR-8m99q2 and the whole of LLR-rm9x4z's "a failure is
/// the `Err` arm" rested on nothing: changing `.map_err(OrgNodeError::Chain)`
/// in `verify.rs` to any other variant left the gate green.
struct FailingChain;

impl org_node::chain::ChainReader for FailingChain {
    fn get_org_state(&self, _org_id: &OrgId) -> Result<Option<OrgState>, String> {
        Err("registry read failed".into())
    }
}

// The abnormal-input case of SDD-pa6p7w: absence and failure are different
// answers and must not collapse into one rejection. A caller that cannot tell
// them apart would treat a transient registry failure as proof that the
// Organisation does not exist — the trusted-root oracle reporting "no anchor"
// when what happened is "no answer".
// verifies: REQ-bvh8v6, LLR-8m99q2, LLR-rm9x4z
#[test]
fn a_chain_read_that_fails_is_refused_as_chain_not_as_absence() {
    let (admin, org, local, env, _) = setup();
    let vk = admin.verifying_key();

    let failed = verify_envelope_against_chain(&local, &env, &ctx(org, &vk), &FailingChain)
        .unwrap_err();
    assert_eq!(
        failed,
        OrgNodeError::Chain("registry read failed".into()),
        "a failed chain read must be refused with Chain, carrying the reason"
    );

    // The same envelope against an EMPTY chain is a different rejection, so
    // the two answers are distinguished rather than merged.
    let absent = verify_envelope_against_chain(&local, &env, &ctx(org, &vk), &MockChain::new())
        .unwrap_err();
    assert_eq!(absent, OrgNodeError::OrgNotOnChain);
    assert_ne!(failed, absent, "a read failure and an absent slot must not be the same error");
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
// verifies: REQ-wp2nyc, REQ-nhe2zu, REQ-mr5abb, LLR-8n95rf, LLR-d6kvbx
#[test]
fn rejects_root_mismatch_when_chain_root_differs() {
    let (admin, org, local, env, _new_root) = setup();
    // Attacker-influenced delta but honest chain root that does NOT match.
    let chain = chain_at(org, RootHash::new([0xde; 32]), Epoch::new(2));
    let vk = admin.verifying_key();
    let ctx = ctx(org, &vk);
    assert_eq!(ctx.seq_guard.last_seen(), SequenceNumber::new(1), "the mark this test hands in");
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::RootMismatch
    );
    assert_eq!(
        ctx.seq_guard.last_seen(),
        SequenceNumber::new(1),
        "a rejection at the root match must leave the high-water mark where it was"
    );
}

// verifies: REQ-8gz8bu, LLR-vf5mjx
#[test]
fn rejects_stale_epoch() {
    let (admin, org, local, env, new_root) = setup();
    let chain = chain_at(org, new_root, Epoch::new(1)); // chain epoch 1 is not newer than committed 1
    let vk = admin.verifying_key();
    let ctx = ctx(org, &vk);
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::StaleEpoch { got: 1, last: 1 }
    );
}

// ---- relocated from src/sequence.rs ----------------------------------------

// verifies: REQ-6yu72z, LLR-wx3php
#[test]
fn rejects_equal_and_lower_seq() {
    let g = SeqGuard::from_last_seen(SequenceNumber::new(5));
    assert!(g.check(SequenceNumber::new(6)).is_ok());
    assert_eq!(g.check(SequenceNumber::new(5)), Err(OrgNodeError::StaleSeq { got: 5, last_seen: 5 }));
    assert_eq!(g.check(SequenceNumber::new(4)), Err(OrgNodeError::StaleSeq { got: 4, last_seen: 5 }));
}

// `advance` is LLR-uc7cej's. It never calls `from_last_seen`, so it carried
// LLR-duwz79 in name only: review round 8 measured that starting every guard
// at zero reddened six tests and none of them carried LLR-duwz79. The next test
// does.
// verifies: REQ-mr5abb, LLR-uc7cej
#[test]
fn advance_moves_high_water_mark_forward_only() {
    let mut g = SeqGuard::new();
    g.advance(SequenceNumber::new(3));
    assert_eq!(g.last_seen(), SequenceNumber::new(3));
    g.advance(SequenceNumber::new(2)); // ignored
    assert_eq!(g.last_seen(), SequenceNumber::new(3));
}

// ---- abnormal input: the cheap checks run before the delta is decoded ------

// verifies: REQ-gju89b, LLR-4fbuy8
#[test]
fn rejects_wrong_org_before_decoding_delta() {
    // Garbage delta bytes: if the org check did not come first, the error
    // would be MalformedDelta.
    let (admin, org, local, _env, _) = setup();
    let garbage = SignedDeltaEnvelope {
        org_id: org,
        parent_seq: SequenceNumber::new(2),
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

// verifies: REQ-ag6kqm, LLR-mcdh85
#[test]
fn rejects_bad_signature_before_decoding_delta() {
    // Garbage delta bytes signed by an imposter, for the expected org: if the
    // signature check did not precede decoding, the error would be
    // MalformedDelta.
    let (admin, org, local, _env, _) = setup();
    let imposter = MemberSeed::from([0xaa; 32]).signing_keypair();
    let garbage = SignedDeltaEnvelope {
        org_id: org,
        parent_seq: SequenceNumber::new(2),
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

// verifies: REQ-6yu72z, LLR-xpbkp5
#[test]
fn rejects_stale_seq_before_decoding_delta() {
    // Garbage delta bytes, honestly signed at parent_seq 1 against a guard
    // that has already seen 1: if the replay check did not precede decoding,
    // the error would be MalformedDelta.
    let (admin, org, local, _env, _) = setup();
    let garbage = SignedDeltaEnvelope {
        org_id: org,
        parent_seq: SequenceNumber::new(1),
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

// verifies: REQ-mr5abb, LLR-duwz79
#[test]
fn from_last_seen_starts_the_guard_at_the_given_mark() {
    for mark in [0u64, 1, 7, u64::MAX - 1] {
        let g = SeqGuard::from_last_seen(SequenceNumber::new(mark));
        assert_eq!(g.last_seen(), SequenceNumber::new(mark), "last_seen reports the mark the guard was started at");
        assert_eq!(g.check(SequenceNumber::new(mark)), Err(OrgNodeError::StaleSeq { got: mark, last_seen: mark }));
        assert!(g.check(SequenceNumber::new(mark + 1)).is_ok());
    }
}

// verifies: REQ-mr5abb, LLR-f5kq88
#[test]
fn check_does_not_advance_the_mark() {
    // `check` takes `&self`: a rejection (or an acceptance that is later
    // rejected downstream) can never move the high-water mark. The assertion
    // documents the contract the type system enforces.
    let g = SeqGuard::from_last_seen(SequenceNumber::new(5));
    g.check(SequenceNumber::new(6)).unwrap();
    assert_eq!(g.last_seen(), SequenceNumber::new(5));
}

// ---- added 2026-10-03 by the architecture tooth ----------------------------

// A delta whose base root is not the local trie's root is refused with the
// specific error, before the delta is applied and before the chain is read.
// verifies: REQ-wp2nyc, LLR-992mbf
#[test]
fn rejects_a_delta_whose_base_root_is_not_the_local_root() {
    let (admin, org, _local, env, new_root) = setup();
    // A local trie that is NOT the one the delta was built against: a genesis
    // with a different admin, so its root differs from the delta's base_root.
    // Two distinct seeds: org-members refuses an organisation in which one key
    // is held twice, a member key equal to its own device key included
    // (`DuplicateKey`, master `a547ff3`). Neither collides with the fixture
    // seeds [1], [2], [3] or the `admin_device()` seed [4].
    let stranger = MemberSeed::from([42u8; 32]).signing_keypair();
    let stranger_device = org_node::DeviceSeed::from([43u8; 32]).signing_keypair();
    let divergent = genesis_trie(&stranger, &stranger_device);

    let chain = chain_at(org, new_root, Epoch::new(2));
    let err = verify_envelope_against_chain(&divergent, &env, &ctx(org, &admin.verifying_key()), &chain)
        .unwrap_err();
    assert_eq!(err, OrgNodeError::DeltaBaseMismatch);
}

// The trie handed to verification is not mutated on the success path: the
// caller's value is still readable and still holds its original root.
// verifies: REQ-wp2nyc, LLR-8hwqru
#[test]
fn a_successful_verification_leaves_the_callers_trie_untouched() {
    let (admin, org, local, env, new_root) = setup();
    let root_before = local.root_hash().unwrap();

    let chain = chain_at(org, new_root, Epoch::new(2));
    let verified =
        verify_envelope_against_chain(&local, &env, &ctx(org, &admin.verifying_key()), &chain)
            .unwrap();

    assert_eq!(local.root_hash().unwrap(), root_before, "the caller's trie moved");
    assert_ne!(
        verified.trie.root_hash().unwrap(),
        root_before,
        "the returned trie must be the new one, not the old"
    );
}

// The same, on a rejection path.
// verifies: REQ-wp2nyc, LLR-8hwqru
#[test]
fn a_rejected_verification_leaves_the_callers_trie_untouched() {
    let (admin, org, local, mut env, new_root) = setup();
    let root_before = local.root_hash().unwrap();
    env.signature = [0u8; 64];

    let chain = chain_at(org, new_root, Epoch::new(2));
    assert!(
        verify_envelope_against_chain(&local, &env, &ctx(org, &admin.verifying_key()), &chain)
            .is_err()
    );
    assert_eq!(local.root_hash().unwrap(), root_before);
}

// `rejects_stale_epoch` above compares an epoch of 1 against a committed epoch
// of 1, so the two numbers in the error are equal and swapping them is
// invisible. The falsifiability sweep of 2026-10-03 found exactly that: a
// mutation exchanging `got` and `last` in verify.rs left the gate green. This
// case uses distinct numbers, so each field is pinned to the thing it names.
// verifies: REQ-8gz8bu, LLR-vf5mjx
#[test]
fn a_stale_epoch_names_the_chain_epoch_and_the_committed_epoch_the_right_way_round() {
    let (admin, org, local, env, new_root) = setup();
    // ctx() commits epoch 1; the chain is behind it at epoch 0.
    let chain = chain_at(org, new_root, Epoch::new(0));
    let vk = admin.verifying_key();
    let err = verify_envelope_against_chain(&local, &env, &ctx(org, &vk), &chain).unwrap_err();
    assert_eq!(err, OrgNodeError::StaleEpoch { got: 0, last: 1 });
}
