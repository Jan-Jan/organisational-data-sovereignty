#![cfg(feature = "test-support")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! verify-against-chain: the receive-side commit rule (verify.rs) and the
//! replay guard it leans on (sequence.rs), tested at the crate's public
//! interface. Nine of these tests were relocated from the `#[cfg(test)]`
//! modules of `src/verify.rs` and `src/sequence.rs` and annotated; the rest
//! are the abnormal-input cases the org-node risk analysis added. What the
//! fix rounds added to the relocated bodies is recorded in
//! `docs/plans/2026-09-09-org-node-risk-analysis.md`.
//!
//! 2026-10-05: the Envelope carries no signature and nothing about its sender
//! is checked (REQ-ag6kqm, amended in place by owner ruling). After the org
//! binding, the replay check is the only check before the decode.

use org_members::RootHash;
use org_node::chain::{MockChain, OrgState};
use org_node::MemberSeed;
use org_node::ids::OrgId;
use org_node::sequence::SeqGuard;
use org_node::test_fixtures::{admin_device, admit_member_delta, genesis_trie, org_public_key, Trie};
use org_node::verify::{check_chain_free, verify_envelope_against_chain, VerifyContext};
use org_node::{DeviceSeed, Envelope, Epoch, OrgNodeError, SequenceNumber};
use std::cell::Cell;

fn setup() -> (OrgId, Trie, Envelope, RootHash) {
    let admin = MemberSeed::from([1u8; 32]).x25519_keypair();
    let local = genesis_trie(&admin, &admin_device()); // receiver's mirror (epoch 1 state)
    // NOTE: admit_member_delta builds its own genesis internally from the same
    // admin and admin_device(); both genesis tries agree by construction
    // (deterministic fixtures).
    let (delta, new_trie) = admit_member_delta(&admin);
    let org = OrgId::new([5u8; 20]);
    let env = Envelope::build(org, SequenceNumber::new(2), &delta).unwrap();
    let new_root = new_trie.root_hash().unwrap();
    (org, local, env, new_root)
}

/// The receiver's context: expects `org`, has committed `parent_seq` 1 at
/// epoch 1. It holds no key: verification decides from the Envelope, this
/// context and the chain reader alone (LLR-na7p4w).
fn ctx(org: OrgId) -> VerifyContext {
    VerifyContext {
        expected_org_id: org,
        seq_guard: SeqGuard::from_last_seen(SequenceNumber::new(1)),
        last_committed_epoch: Epoch::new(1),
    }
}

/// A chain whose state for `org` is `root` at `epoch`.
fn chain_at(org: OrgId, root: RootHash, epoch: Epoch) -> MockChain {
    let mut chain = MockChain::new();
    chain.set(org, OrgState { root_hash: root, org_pub_key: org_public_key(), epoch });
    chain
}

/// An envelope for `org` at `seq` whose Change set bytes do not decode.
fn garbage(org: OrgId, seq: u64) -> Envelope {
    Envelope { org_id: org, parent_seq: SequenceNumber::new(seq), delta_bytes: vec![0xff; 16] }
}

// ---- relocated from src/verify.rs ------------------------------------------

// REQ-nhe2zu's commit rule, and REQ-txvtm9's epoch rule with it: the envelope
// names the expected Organisation, carries a newer Sequence number and reaches
// the chain's root at a newer epoch. Nothing about the sender is checked and
// the context holds no key (REQ-ag6kqm, LLR-na7p4w). It is also LLR-9f5hmr's
// normal case (the Sequence number 2 is the chain's epoch 2).
// verifies: REQ-nhe2zu, REQ-txvtm9, REQ-ag6kqm, LLR-8n95rf, LLR-d6kvbx, LLR-8hwqru, LLR-na7p4w, LLR-9f5hmr
#[test]
fn happy_path_commits_when_root_matches_chain() {
    let (org, local, env, new_root) = setup();
    let chain = chain_at(org, new_root, Epoch::new(2));
    let ctx = ctx(org);
    let out = verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap();
    assert_eq!(out.epoch, Epoch::new(2));
    assert_eq!(out.seq_guard.last_seen(), SequenceNumber::new(2));
    assert_eq!(out.trie.root_hash().unwrap(), new_root);
}

// verifies: REQ-gju89b, LLR-4fbuy8, LLR-ybn5pr
#[test]
fn rejects_wrong_org_id() {
    let (_org, local, env, _) = setup();
    let chain = MockChain::new();
    let ctx = ctx(OrgId::new([0xff; 20]));
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::OrgIdMismatch
    );
}

// verifies: REQ-6yu72z, LLR-xpbkp5, LLR-wx3php, LLR-cs4mpb
#[test]
fn rejects_stale_seq() {
    let (org, local, env, new_root) = setup();
    let chain = chain_at(org, new_root, Epoch::new(2));
    let ctx = VerifyContext {
        seq_guard: SeqGuard::from_last_seen(SequenceNumber::new(2)), // env.parent_seq == 2, not > 2
        ..ctx(org)
    };
    assert_eq!(
        verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
        OrgNodeError::StaleSeq { got: 2, last_seen: 2 }
    );
}

// verifies: REQ-bvh8v6, LLR-8m99q2, LLR-rm9x4z
#[test]
fn rejects_when_org_absent_from_chain() {
    let (org, local, env, _) = setup();
    let chain = MockChain::new(); // empty
    let ctx = ctx(org);
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
    let (org, local, env, _) = setup();

    let failed = verify_envelope_against_chain(&local, &env, &ctx(org), &FailingChain)
        .unwrap_err();
    assert_eq!(
        failed,
        OrgNodeError::Chain("registry read failed".into()),
        "a failed chain read must be refused with Chain, carrying the reason"
    );

    // The same envelope against an EMPTY chain is a different rejection, so
    // the two answers are distinguished rather than merged.
    let absent = verify_envelope_against_chain(&local, &env, &ctx(org), &MockChain::new())
        .unwrap_err();
    assert_eq!(absent, OrgNodeError::OrgNotOnChain);
    assert_ne!(failed, absent, "a read failure and an absent slot must not be the same error");
}

// REQ-nhe2zu's commit rule, and REQ-txvtm9's after it, is conditional, so
// withholding the commit when the recomputed root does not match the
// chain's is that requirement's abnormal-input case as well as REQ-wp2nyc's
// rejection.
//
// It is also REQ-mr5abb's last step: the root match is the final check, so an
// envelope rejected here has passed the sequence check and would have had its
// Sequence number committed had the root matched. The mark must not move.
// `verify_envelope_against_chain` takes the context by shared reference and
// returns the advanced guard only inside `Ok`, so the guard to assert on is
// the one this test handed in.
// verifies: REQ-wp2nyc, REQ-nhe2zu, REQ-txvtm9, REQ-mr5abb, LLR-8n95rf, LLR-d6kvbx
#[test]
fn rejects_root_mismatch_when_chain_root_differs() {
    let (org, local, env, _new_root) = setup();
    // Attacker-influenced delta but honest chain root that does NOT match.
    let chain = chain_at(org, RootHash::new([0xde; 32]), Epoch::new(2));
    let ctx = ctx(org);
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
    let (org, local, env, new_root) = setup();
    let chain = chain_at(org, new_root, Epoch::new(1)); // chain epoch 1 is not newer than committed 1
    let ctx = ctx(org);
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

// verifies: REQ-gju89b, LLR-4fbuy8, LLR-ybn5pr
#[test]
fn rejects_wrong_org_before_decoding_delta() {
    // Garbage delta bytes: if the org check did not
    // come first, the error would be MalformedDelta.
    let (org, local, _env, _) = setup();
    let ctx = ctx(OrgId::new([0xee; 20]));
    assert_eq!(
        verify_envelope_against_chain(&local, &garbage(org, 2), &ctx, &MockChain::new())
            .unwrap_err(),
        OrgNodeError::OrgIdMismatch
    );
}

// Garbage Change set bytes for the expected org at a fresh Sequence number:
// no signature and no sender is checked before the decode (REQ-ag6kqm as
// amended), so the refusal is the decode's own.
// verifies: REQ-ag6kqm, LLR-mcdh85, LLR-9sknpa
#[test]
fn undecodable_change_set_bytes_are_refused_as_malformed_delta() {
    let (org, local, _env, _) = setup();
    assert_eq!(
        verify_envelope_against_chain(&local, &garbage(org, 2), &ctx(org), &MockChain::new())
            .unwrap_err(),
        OrgNodeError::MalformedDelta
    );
}

// verifies: REQ-6yu72z, LLR-xpbkp5
#[test]
fn rejects_stale_seq_before_decoding_delta() {
    // Garbage delta bytes at parent_seq 1, against a
    // guard that has already seen 1: if the replay check did not precede
    // decoding, the error would be MalformedDelta.
    let (org, local, _env, _) = setup();
    let ctx = ctx(org); // SeqGuard::from_last_seen(SequenceNumber::new(1))
    assert_eq!(
        verify_envelope_against_chain(&local, &garbage(org, 1), &ctx, &MockChain::new())
            .unwrap_err(),
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
// verifies: REQ-wp2nyc, LLR-992mbf, LLR-9sknpa
#[test]
fn rejects_a_delta_whose_base_root_is_not_the_local_root() {
    let (org, _local, env, new_root) = setup();
    // A local trie that is NOT the one the delta was built against: a genesis
    // with a different admin, so its root differs from the delta's base_root.
    // Two distinct seeds: org-members refuses an organisation in which one key
    // is held twice, a member key equal to its own device key included
    // (`DuplicateKey`, master `a547ff3`). Neither collides with the fixture
    // seeds [1], [2], [3] or ADMIN_DEVICE_SEED [4].
    let stranger = MemberSeed::from([42u8; 32]).x25519_keypair();
    let stranger_device = DeviceSeed::from([43u8; 32]).signing_keypair();
    let divergent = genesis_trie(&stranger, &stranger_device);

    let chain = chain_at(org, new_root, Epoch::new(2));
    let err = verify_envelope_against_chain(&divergent, &env, &ctx(org), &chain)
        .unwrap_err();
    assert_eq!(err, OrgNodeError::DeltaBaseMismatch);
}

// The trie handed to verification is not mutated on the success path: the
// caller's value is still readable and still holds its original root.
// verifies: REQ-wp2nyc, LLR-8hwqru
#[test]
fn a_successful_verification_leaves_the_callers_trie_untouched() {
    let (org, local, env, new_root) = setup();
    let root_before = local.root_hash().unwrap();

    let chain = chain_at(org, new_root, Epoch::new(2));
    let verified =
        verify_envelope_against_chain(&local, &env, &ctx(org), &chain)
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
    let (org, local, env, _new_root) = setup();
    let root_before = local.root_hash().unwrap();
    // Was `env.signature = [0u8; 64]` (a forged signature) before the
    // Envelope lost its signature (2026-10-05); the rejection is now a chain
    // root the Change set does not reach. LLR-8hwqru is about the caller's
    // trie on any rejection path, not about which check rejects.
    let chain = chain_at(org, RootHash::new([0xde; 32]), Epoch::new(2));
    assert!(verify_envelope_against_chain(&local, &env, &ctx(org), &chain).is_err());
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
    let (org, local, env, new_root) = setup();
    // ctx() commits epoch 1; the chain is behind it at epoch 0.
    let chain = chain_at(org, new_root, Epoch::new(0));
    let err = verify_envelope_against_chain(&local, &env, &ctx(org), &chain).unwrap_err();
    assert_eq!(err, OrgNodeError::StaleEpoch { got: 0, last: 1 });
}

// ---- added 2026-10-05 by review round 1 (finding-1) ------------------------

// The Sequence number must be the epoch of the Organisation state it is
// verified against. The envelope at 2 against a chain at epoch 3, with the
// root the delta reaches, is refused; so is u64::MAX against epoch 2, the
// number any sender would choose to jam the mark. The mark the test
// handed in stays where it was.
// verifies: REQ-txvtm9, REQ-mr5abb, LLR-9f5hmr, LLR-cs4mpb
#[test]
fn a_sequence_number_other_than_the_chain_epoch_is_refused() {
    let (org, local, env, new_root) = setup();
    let ctx = ctx(org);

    let behind = verify_envelope_against_chain(&local, &env, &ctx, &chain_at(org, new_root, Epoch::new(3))).unwrap_err();
    assert_eq!(behind, OrgNodeError::SeqNotEpoch { seq: 2, epoch: 3 });

    let jam = Envelope { parent_seq: SequenceNumber::new(u64::MAX), ..env.clone() };
    let ahead = verify_envelope_against_chain(&local, &jam, &ctx, &chain_at(org, new_root, Epoch::new(2))).unwrap_err();
    assert_eq!(ahead, OrgNodeError::SeqNotEpoch { seq: u64::MAX, epoch: 2 });
    assert_eq!(ctx.seq_guard.last_seen(), SequenceNumber::new(1), "a refusal must leave the mark where it was");
}

// ---- added 2026-10-05 by review round 2 (finding-6) ------------------------

// LLR-9f5hmr's order, both sides. The Sequence-number check runs after the
// stale-epoch check: an Envelope whose chain state is stale and whose number
// is not that state's epoch is refused as StaleEpoch. It runs before the
// root match: an Envelope whose number is not the chain's epoch and whose
// Change set misses the chain's root is refused as SeqNotEpoch.
// verifies: REQ-txvtm9, LLR-9f5hmr
#[test]
fn the_sequence_number_is_checked_after_the_stale_epoch_and_before_the_root_match() {
    let (org, local, env, new_root) = setup();
    let ctx = ctx(org);

    // Chain epoch 1 is not newer than the committed 1, and 2 is not 1.
    let stale = verify_envelope_against_chain(&local, &env, &ctx, &chain_at(org, new_root, Epoch::new(1))).unwrap_err();
    assert_eq!(stale, OrgNodeError::StaleEpoch { got: 1, last: 1 });

    // 2 is not the chain's epoch 3, and the chain's root is not the one the
    // Change set reaches.
    let wrong_root = chain_at(org, RootHash::new([0xde; 32]), Epoch::new(3));
    let both = verify_envelope_against_chain(&local, &env, &ctx, &wrong_root).unwrap_err();
    assert_eq!(both, OrgNodeError::SeqNotEpoch { seq: 2, epoch: 3 });
}

// ---- the chain-free half (T5 of docs/plans/2026-10-05-chain-authority.md) ---

/// A chain reader that counts its reads.
struct Counting {
    inner: MockChain,
    reads: Cell<usize>,
}
impl org_node::chain::ChainReader for Counting {
    fn get_org_state(&self, org: &OrgId) -> Result<Option<OrgState>, String> {
        self.reads.set(self.reads.get() + 1);
        self.inner.get_org_state(org)
    }
}

// Normal: an honest envelope passes the four chain-free checks.
// verifies: REQ-f2k4tr, LLR-fuq379
#[test]
fn check_chain_free_passes_an_honest_envelope() {
    let (org, local, env, _) = setup();
    assert_eq!(check_chain_free(&local, &env, &ctx(org)), Ok(()));
}

// Abnormal: each chain-free check refuses with its own error, in order.
// verifies: REQ-f2k4tr, LLR-fuq379
#[test]
fn check_chain_free_refuses_each_check_in_order() {
    let (org, local, _env, _) = setup();
    assert_eq!(check_chain_free(&local, &garbage(org, 2), &ctx(OrgId::new([0xee; 20]))), Err(OrgNodeError::OrgIdMismatch));
    assert_eq!(check_chain_free(&local, &garbage(org, 1), &ctx(org)), Err(OrgNodeError::StaleSeq { got: 1, last_seen: 1 }));
    assert_eq!(check_chain_free(&local, &garbage(org, 2), &ctx(org)), Err(OrgNodeError::MalformedDelta));
    let stranger = MemberSeed::from([0x5a; 32]).x25519_keypair();
    let (other_base, _) = admit_member_delta(&stranger);
    let wrong_base = Envelope::build(org, SequenceNumber::new(2), &other_base).unwrap();
    assert_eq!(check_chain_free(&local, &wrong_base, &ctx(org)), Err(OrgNodeError::DeltaBaseMismatch));
}

// Abnormal and normal: verify reads the chain only after the chain-free
// checks pass, and then exactly once.
// verifies: REQ-f2k4tr, LLR-fuq379
#[test]
fn verify_reads_the_chain_once_and_only_after_the_chain_free_checks() {
    let (org, local, env, new_root) = setup();
    let chain = Counting { inner: chain_at(org, new_root, Epoch::new(2)), reads: Cell::new(0) };
    let stale = Envelope { parent_seq: SequenceNumber::new(1), ..env.clone() };
    assert!(verify_envelope_against_chain(&local, &stale, &ctx(org), &chain).is_err());
    assert_eq!(chain.reads.get(), 0, "a chain-free refusal reads no chain");
    verify_envelope_against_chain(&local, &env, &ctx(org), &chain).unwrap();
    assert_eq!(chain.reads.get(), 1, "a passing envelope reads the chain once");
}

// ---- normal cases of LLR-9sknpa and LLR-mcdh85 (S3 robustness, 2026-10-07) --

// Normal: Change set bytes that decode and extend the receiver's record are
// refused neither as malformed nor as built on another base, at any fresh
// Sequence number, and at the chain's epoch they commit to the chain's root.
// verifies: LLR-9sknpa
#[test]
fn a_change_set_that_decodes_and_extends_the_record_is_taken() {
    let (org, local, env, new_root) = setup();
    let decoded = env.decode_delta().unwrap();
    assert_eq!(decoded.base_root(), &local.root_hash().unwrap(), "fixture: the Change set extends the record");
    for seq in [2, 7] {
        let fresh = Envelope { parent_seq: SequenceNumber::new(seq), ..env.clone() };
        assert_eq!(check_chain_free(&local, &fresh, &ctx(org)), Ok(()), "Sequence number {seq}");
    }
    let verified = verify_envelope_against_chain(&local, &env, &ctx(org), &chain_at(org, new_root, Epoch::new(2))).unwrap();
    assert_eq!(verified.trie.root_hash().unwrap(), new_root);
}

// Normal: an Envelope of exactly the Organisation, a Sequence number one past
// the mark and decodable Change set bytes — no signature, no sender, and a
// context that holds no key — is decoded and verified: nothing but the mark
// stands between the binding and the decode, and the chain is read once.
// verifies: LLR-mcdh85
#[test]
fn an_envelope_past_the_mark_is_decoded_and_verified_with_no_signature_or_sender() {
    let (org, local, env, new_root) = setup();
    let bare = Envelope { org_id: org, parent_seq: SequenceNumber::new(2), delta_bytes: env.delta_bytes.clone() };
    let chain = Counting { inner: chain_at(org, new_root, Epoch::new(2)), reads: Cell::new(0) };
    let verified = verify_envelope_against_chain(&local, &bare, &ctx(org), &chain).unwrap();
    assert_eq!(chain.reads.get(), 1);
    assert_eq!((verified.epoch, verified.seq_guard.last_seen()), (Epoch::new(2), SequenceNumber::new(2)));
    assert_eq!(verified.trie.root_hash().unwrap(), new_root);
}
