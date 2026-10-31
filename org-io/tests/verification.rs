#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! Verification (SDD-t4z8sz): `verify_organisation` reads the finalised block,
//! then the state, once each, takes one reading of the handle's clock and hands both to org-node's
//! verification, mapping its outcome (LLR-t2pve7); a failed read is unverified (LLR-ce42ce); the
//! clock is replaceable under test-support only (LLR-fzf2n3, LLR-6vtbn6).

use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use org_node::{DevicePublicKey, Epoch, OrgId, OrgState};
use org_io::send::Delivery;
use org_io::test_support::{dial_direct_as, FakeChain, Refusal, TestClock};
use org_io::transport::TransportMode;
use org_io::verify::{DiscardedChangeSummary, Inconsistency, UnverifiedReason, Verified};
use org_io::write::HeldTarget;
use org_io::{OrgIo, Received, WriteOutcome, WriteReport};

mod common;
use common::handles::{founded, fresh_keychain, handle, proposed, reopened_service, requested, store_bytes, store_dir, NET};
use common::journey::{
    self, admit, chain_moved_elsewhere, invited, keep_request, member_id_of, propose_admission,
    unknown_write, Admin,
};

/// A's current epoch of `org` sent while B (receiving, the only other Device) takes it with its
/// receive step, which writes what it keeps.
async fn epoch_kept(sender: &OrgIo, org: OrgId, receiver: &OrgIo) {
    let (received, sent) = journey::deliver_epoch(sender, org, receiver).await;
    assert!(
        matches!(received, Ok(Received::ProvisionalOrganisationReceived { .. } | Received::RevocationNoticeKept { .. })),
        "{received:?}"
    );
    assert!(sent.per_device.iter().all(|(_, delivery)| matches!(delivery, Delivery::Delivered)), "{sent:?}");
}

/// A holds an Organisation with B admitted at epoch 2: A wrote and verified it; B kept A's epoch
/// over the transport and adopted it by its own verification. Returns A, B, the Organisation and
/// B's Device key.
async fn two_members(tag: &str, chain: &FakeChain) -> (OrgIo, OrgIo, OrgId, DevicePublicKey) {
    let (alice, bob) = (handle(tag, "a", chain), handle(tag, "b", chain));
    let (org, _) = founded(&alice, "alice").await;
    proposed(&alice, org, &bob).await;
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap(), WriteReport::Included { .. }));
    assert!(matches!(alice.verify_organisation(HeldTarget::Organisation(org)).await, Verified::Adopted { .. }));
    let joiner = bob.personas().await[0].device_key;
    epoch_kept(&alice, org, &bob).await;
    assert!(matches!(bob.verify_organisation(HeldTarget::Organisation(org)).await, Verified::Adopted { .. }));
    (alice, bob, org, joiner)
}

/// A removes B in epoch 3, written, verified by A, and B's notice delivered to B.
async fn remove_b(alice: &OrgIo, bob: &OrgIo, org: OrgId, joiner: DevicePublicKey) {
    let bob_member = alice.organisation(org).await.unwrap().members.iter().find(|member| member.device_keys.contains(&joiner)).unwrap().id;
    alice.propose_revocation(org, bob_member).await.unwrap();
    alice.propose_updated_organisation(org).await.unwrap();
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap(), WriteReport::Included { .. }));
    assert!(matches!(alice.verify_organisation(HeldTarget::Organisation(org)).await, Verified::Adopted { .. }));
    epoch_kept(alice, org, bob).await;
}

// verifies: LLR-t2pve7
#[tokio::test]
async fn verify_organisation_reads_the_block_then_the_state_once_and_maps_each_outcome() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, bob, org, joiner) = two_members("verify-outcomes", &chain).await;
    let target = HeldTarget::Organisation(org);
    let (reads, finalised) = (chain.reads(), chain.finalised_reads());
    assert_eq!(alice.verify_organisation(target).await, Verified::InStep { epoch: Epoch::new(2) });
    assert_eq!((chain.reads() - reads, chain.finalised_reads() - finalised), (1, 1), "one block read, one state read");

    chain.epoch_ahead.store(true, Ordering::SeqCst);
    assert_eq!(alice.verify_organisation(target).await, Verified::Behind { chain_epoch: Epoch::new(3) });
    chain.epoch_ahead.store(false, Ordering::SeqCst);

    let slot = chain.slots().get(&org).unwrap();
    chain.slots().set(org, OrgState { epoch: Epoch::new(1), ..slot });
    assert_eq!(
        alice.verify_organisation(target).await,
        Verified::Inconsistent { reason: Inconsistency::ChainBehindRecord { chain_epoch: Epoch::new(1), record_epoch: Epoch::new(2) } }
    );
    chain.slots().set(org, slot);
    // org-node's behind marker, set by `Behind` in memory only, is cleared by an in-step verification.
    assert_eq!(alice.verify_organisation(target).await, Verified::InStep { epoch: Epoch::new(2) });

    remove_b(&alice, &bob, org, joiner).await;
    assert_eq!(bob.verify_organisation(target).await, Verified::Removed { org_id: org }, "B's kept notice, accepted against the chain");
    assert!(bob.organisations().await.is_empty());
}

// The order of the two reads: with the state read held open,
// the finalised block has already been read, once; released, the verification answers.
// verifies: LLR-t2pve7
#[tokio::test]
async fn the_finalised_block_is_read_before_the_state() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("verify-read-order", "a", &chain);
    let (org, _founder) = founded(&alice, "alice").await;
    let (reads, finalised) = (chain.reads(), chain.finalised_reads());
    let hold = chain.hold_next_read();
    let (verified, ()) = tokio::join!(alice.verify_organisation(HeldTarget::Organisation(org)), async {
        hold.arrived().await;
        assert_eq!(chain.finalised_reads() - finalised, 1, "the finalised block was read before the state read began");
        assert_eq!(chain.reads() - reads, 1, "the state read is the one held");
        hold.release();
    });
    assert_eq!(verified, Verified::InStep { epoch: Epoch::new(1) });
    assert_eq!(chain.finalised_reads() - finalised, 1, "and no block read after it");
}

// verifies: LLR-ce42ce
#[tokio::test]
async fn a_failed_read_or_parse_is_unverified_calls_no_org_node_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("verify-failed-read", "a", &chain);
    let (org, _founder) = founded(&alice, "alice").await;
    let target = HeldTarget::Organisation(org);
    let before = store_bytes("verify-failed-read", "a");
    chain.read_failing.store(true, Ordering::SeqCst);
    assert!(matches!(alice.verify_organisation(target).await, Verified::Unverified { reason: UnverifiedReason::Read(_), .. }));
    chain.read_failing.store(false, Ordering::SeqCst);
    chain.parse_refusing.store(true, Ordering::SeqCst);
    assert!(matches!(alice.verify_organisation(target).await, Verified::Unverified { reason: UnverifiedReason::Read(_), .. }));
    chain.parse_refusing.store(false, Ordering::SeqCst);
    assert_eq!(store_bytes("verify-failed-read", "a"), before, "nothing written");
    chain.state_hidden.store(true, Ordering::SeqCst);
    assert!(
        matches!(alice.verify_organisation(target).await, Verified::Unverified { reason: UnverifiedReason::OrgNotOnChain, .. }),
        "no state: org-node's refusal, unverified"
    );
    chain.state_hidden.store(false, Ordering::SeqCst);
    assert_eq!(store_bytes("verify-failed-read", "a"), before, "nothing written");
    assert_eq!(alice.verify_organisation(target).await, Verified::InStep { epoch: Epoch::new(1) }, "nothing judged by the failed reads");
}

// verifies: LLR-ce42ce
#[tokio::test]
async fn a_failed_read_keeps_the_readers_error_as_its_source() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("verify-read-source", "a", &chain);
    let (org, _founder) = founded(&alice, "alice").await;
    chain.read_failing.store(true, Ordering::SeqCst);
    let Verified::Unverified { reason: UnverifiedReason::Read(error) } = alice.verify_organisation(HeldTarget::Organisation(org)).await
    else {
        panic!("a failed read is unverified");
    };
    let source = std::error::Error::source(&error).expect("the reader's error is kept");
    assert!(matches!(source.downcast_ref::<org_node::OrgNodeError>(), Some(org_node::OrgNodeError::Chain(_))), "{source}");
    assert_eq!(error.to_string(), source.to_string(), "and its text is the reader's");
}

// verifies: LLR-ce42ce
#[tokio::test]
async fn verification_returns_the_last_outcome_and_unverified_before_any() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("verify-last", "a", &chain);
    let (org, _founder) = founded(&alice, "alice").await;
    let target = HeldTarget::Organisation(org);
    assert_eq!(alice.verification(target).await, Verified::Unverified { reason: UnverifiedReason::NotYetVerified });
    alice.verify_organisation(target).await;
    assert_eq!(alice.verification(target).await, Verified::InStep { epoch: Epoch::new(1) });
    chain.read_failing.store(true, Ordering::SeqCst);
    alice.verify_organisation(target).await;
    assert!(matches!(alice.verification(target).await, Verified::Unverified { reason: UnverifiedReason::Read(_), .. }));
    // The other Organisation is founded through a handle over the chain, and its data directory
    // opened again with no chain configured.
    let other_chain = FakeChain::new();
    let configured = handle("verify-last-other", "a", &other_chain);
    let (other, _) = founded(&configured, "alice").await;
    drop(configured);
    let unconfigured = OrgIo::open(&store_dir("verify-last-other", "a"), TransportMode::Loopback).unwrap();
    assert!(
        matches!(
            unconfigured.verify_organisation(HeldTarget::Organisation(other)).await,
            Verified::Unverified { reason: UnverifiedReason::Read(_), .. }
        ),
        "a handle with no chain"
    );
}

// verifies: LLR-fzf2n3
#[tokio::test(flavor = "multi_thread")]
async fn the_time_given_is_one_clock_reading_after_the_read() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, bob, org, joiner) = two_members("verify-clock", &chain).await;
    remove_b(&alice, &bob, org, joiner).await;
    let clock = TestClock::at(1_000_000);
    let bob = Arc::new(bob.with_clock_for_test(clock.clone()));
    let held = chain.hold_next_read();
    let verifying = {
        let bob = bob.clone();
        tokio::spawn(async move { bob.verify_organisation(HeldTarget::Organisation(org)).await })
    };
    held.arrived().await;
    assert_eq!(clock.readings(), 0, "no reading before the state read returns");
    held.release();
    assert!(matches!(verifying.await.unwrap(), Verified::Removed { .. }));
    // The removal's delivery task, started as `Removed` returns, reads the same clock at once, so
    // the count here is at least the verification's one reading (an exact count would race that
    // task); the exact count is checked on A below, whose verification starts no task.
    assert!(clock.readings() >= 1, "the verification read the clock");
    // The removal's deadline is the kept verification time plus 7 days (LLR-rav7dr): read through
    // the handle's report, not org-node's store.
    let report = bob.acknowledgement_report(org).await.expect("the removal is reported");
    assert_eq!(report.deadline, Some(1_000_000 + 604_800), "the time org-node kept is the clock's reading");

    // A verification that starts no delivery reads the clock exactly once, after the read.
    let a_clock = TestClock::at(2_000_000);
    let alice = Arc::new(alice.with_clock_for_test(a_clock.clone()));
    let held = chain.hold_next_read();
    let verifying = {
        let alice = alice.clone();
        tokio::spawn(async move { alice.verify_organisation(HeldTarget::Organisation(org)).await })
    };
    held.arrived().await;
    assert_eq!(a_clock.readings(), 0, "no reading before the state read returns");
    held.release();
    assert!(matches!(verifying.await.unwrap(), Verified::InStep { .. }));
    assert_eq!(a_clock.readings(), 1, "one reading per verification");
}

// verifies: LLR-fzf2n3
#[tokio::test]
async fn a_clock_before_the_epoch_is_unverified() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let clock = TestClock::at(0);
    let alice = handle("verify-before-epoch", "a", &chain).with_clock_for_test(clock.clone());
    // The founding adopted through the handle first, by a clock that reads; then it reads before
    // the epoch.
    let (org, _founder) = founded(&alice, "alice").await;
    clock.set_before_epoch(true);
    let before = store_bytes("verify-before-epoch", "a");
    let target = HeldTarget::Organisation(org);
    assert_eq!(alice.verify_organisation(target).await, Verified::Unverified { reason: UnverifiedReason::ClockBeforeEpoch });
    assert_eq!(store_bytes("verify-before-epoch", "a"), before, "nothing written");
}

/// B holds A's epoch 3 of `org`, received over the transport, while the chain is put back at
/// epoch 2: one provisional organisation of B's one epoch ahead of the chain. A's epoch 3 admits
/// C. Returns B, the Organisation and B's Device key.
async fn received_ahead_of_the_chain(tag: &str, chain: &FakeChain) -> (OrgIo, OrgId, DevicePublicKey) {
    let (alice, bob, org, joiner) = two_members(tag, chain).await;
    let at_two = chain.slots().get(&org).unwrap();
    let carol = handle(tag, "c", chain);
    let blob = alice.issue_invite(org, "Carol").await.unwrap();
    let invite_id = org_node::invitation::Invite::parse(&blob).unwrap().invite_id;
    requested(&alice, &carol, &blob, &["carol"]).await;
    alice.propose_admission(org, invite_id, None).await.unwrap();
    alice.propose_updated_organisation(org).await.unwrap();
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap(), WriteReport::Included { .. }));
    assert!(matches!(alice.verify_organisation(HeldTarget::Organisation(org)).await, Verified::Adopted { .. }));
    let (received, _) = journey::deliver_epoch(&alice, org, &bob).await;
    assert!(matches!(received, Ok(Received::ProvisionalOrganisationReceived { .. })), "{received:?}");
    chain.slots().set(org, at_two);
    (bob, org, joiner)
}

// Normal and boundary, under two limits: a received Organisation one epoch ahead of the chain is
// kept by a verification exactly the handle's ahead limit after the first verification that found
// it there, and discarded by one a second later. The same times give two outcomes under the two
// limits, so the limit verification used is the one the handle was given.
// verifies: LLR-22se2q
#[tokio::test]
async fn the_ahead_limit_the_handle_was_given_is_the_one_verification_uses() {
    let (_guard, _keychain) = fresh_keychain();
    for (tag, limit) in [("verify-ahead-600", 600), ("verify-ahead-1800", 1_800)] {
        let chain = FakeChain::new();
        let (bob, org, bob_device) = received_ahead_of_the_chain(tag, &chain).await;
        let clock = TestClock::at(1_000_000);
        let bob = bob.with_clock_for_test(clock.clone()).with_ahead_limit_for_test(Duration::from_secs(limit));
        let held = || reopened_service(tag, "b", &bob).provisional_organisations(&bob_device).unwrap().len();
        let target = HeldTarget::Organisation(org);
        assert_eq!(held(), 1, "{tag}: B holds the next epoch, received");
        assert!(matches!(bob.verify_organisation(target).await, Verified::InStep { .. }), "{tag}: found ahead, stamped");
        clock.set(1_000_000 + limit);
        assert!(matches!(bob.verify_organisation(target).await, Verified::InStep { .. }));
        assert_eq!(held(), 1, "{tag}: kept exactly the limit after the stamp");
        clock.set(1_000_000 + limit + 1);
        assert!(matches!(bob.verify_organisation(target).await, Verified::InStep { .. }));
        assert_eq!(held(), 0, "{tag}: discarded a second past the limit");
    }
}

// Abnormal: a handle with no chain holds no ahead limit, and its verification is refused as its
// read is, writing nothing; a handle over a substitute chain holds the test limit until replaced.
// verifies: LLR-22se2q
#[tokio::test]
async fn a_handle_with_no_chain_holds_no_ahead_limit_and_verifies_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let configured = handle("verify-no-limit", "a", &chain);
    assert_eq!(configured.ahead_limit_for_test(), Some(Duration::from_secs(1_200)), "the substitute chain's handle");
    let configured = configured.with_ahead_limit_for_test(Duration::from_millis(600_900));
    assert_eq!(configured.ahead_limit_for_test(), Some(Duration::from_secs(600)), "whole seconds, the fraction dropped");
    let (org, _) = founded(&configured, "alice").await;
    drop(configured);
    let unconfigured = OrgIo::open(&store_dir("verify-no-limit", "a"), TransportMode::Loopback).unwrap();
    assert_eq!(unconfigured.ahead_limit_for_test(), None, "a handle with no chain");
    let before = store_bytes("verify-no-limit", "a");
    let Verified::Unverified { reason: UnverifiedReason::Read(error) } =
        unconfigured.verify_organisation(HeldTarget::Organisation(org)).await
    else {
        panic!("refused as a read")
    };
    assert!(error.to_string().contains("chain not configured"), "the not-configured message: {error}");
    assert_eq!(store_bytes("verify-no-limit", "a"), before, "nothing written");
}

// Normal, boundary and abnormal: the one conversion `connect` and the test constructors give
// `ChainSettings::ahead_limit` through, read back from a handle given each limit, is whole
// seconds, the fraction dropped: a whole number of seconds is kept, a fraction is dropped, not
// rounded (to nothing, below a second), and the largest `Duration` keeps its whole seconds.
// verifies: LLR-22se2q
#[tokio::test]
async fn the_ahead_limit_is_the_durations_whole_seconds_the_fraction_dropped() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let mut held = handle("verify-limit-seconds", "a", &chain);
    for (given, whole_seconds, case) in [
        (Duration::from_secs(1_200), 1_200, "whole seconds kept"),
        (Duration::from_millis(600_999), 600, "the fraction dropped, not rounded"),
        (Duration::from_millis(999), 0, "under a second is none"),
        (Duration::ZERO, 0, "zero"),
        (Duration::MAX, u64::MAX, "the largest Duration's whole seconds"),
    ] {
        held = held.with_ahead_limit_for_test(given);
        assert_eq!(held.ahead_limit_for_test(), Some(Duration::from_secs(whole_seconds)), "{case}");
    }
}

// verifies: LLR-6vtbn6
#[tokio::test]
async fn a_test_clock_replaces_the_handles_clock_under_test_support_only() {
    let (_guard, _keychain) = fresh_keychain();
    let clock = TestClock::at(42);
    let dir = store_dir("verify-test-clock", "a");
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    let opened = OrgIo::open_with_clock_for_test(&dir, TransportMode::Loopback, clock.clone()).unwrap();
    assert_eq!(opened.clock_reading_for_test(), Some(42), "open_with_clock_for_test's handle reads the test clock");
    assert_eq!(handle("verify-test-clock", "b", &FakeChain::new()).with_clock_for_test(clock).clock_reading_for_test(), Some(42));
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "rs") || path.ends_with("test_support.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for replacing in ["fn with_clock_for_test", "fn open_with_clock_for_test", "TestClock"] {
            assert!(!text.contains(replacing), "{} replaces the clock outside test builds", path.display());
        }
    }
    // `test_support.rs` defines them: the module, and the handle's `impl` block in it that adds the
    // two constructors, are each compiled only under `test-support`.
    let gate = "#[cfg(feature = \"test-support\")]";
    let lib = std::fs::read_to_string(src.join("lib.rs")).unwrap();
    let lib_lines: Vec<&str> = lib.lines().map(str::trim).collect();
    let module = lib_lines.iter().position(|line| *line == "pub mod test_support;").expect("lib.rs declares the module");
    assert_eq!(lib_lines[module - 1], gate, "the test_support module is compiled only under test-support");
    let support = std::fs::read_to_string(src.join("test_support.rs")).unwrap();
    let support_lines: Vec<&str> = support.lines().map(str::trim).collect();
    for constructor in ["fn with_clock_for_test", "fn open_with_clock_for_test"] {
        let defined = support_lines.iter().position(|line| line.contains(constructor)).expect("test_support.rs defines it");
        let impl_line = (0..defined).rev().find(|&earlier| support_lines[earlier].starts_with("impl")).expect("inside an impl block");
        let handles = support_lines[impl_line].starts_with("impl crate::OrgIo") || support_lines[impl_line].starts_with("impl OrgIo");
        assert!(handles, "{constructor} is the handle's");
        let above = (0..impl_line).rev().find(|&earlier| !support_lines[earlier].starts_with("///")).expect("a line above the impl");
        assert_eq!(support_lines[above], gate, "{constructor}'s impl block is compiled only under test-support");
    }
    for entry in ["pub fn open(", "pub async fn connect("] {
        for (at, _) in lib.match_indices(entry) {
            let signature = &lib[at..at + lib[at..].find('{').unwrap()];
            assert!(!signature.contains("Clock"), "{entry} takes a clock: {signature}");
        }
    }
    let clock_rs = std::fs::read_to_string(src.join("clock.rs")).unwrap();
    assert!(clock_rs.contains("SystemTime::now()"), "the production clock is the system's");
}

// verifies: LLR-t2pve7, LLR-zey4mg
#[tokio::test(flavor = "current_thread")]
async fn a_verification_that_adopts_an_unknown_write_reports_it_adopted_and_clears_it() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("verify-unknown-adopted", "a", &chain);
    let (org, _) = founded(&alice, "alice").await;
    proposed(&alice, org, &handle("verify-unknown-adopted", "b", &chain)).await;
    let target = HeldTarget::Organisation(org);
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    tokio::time::pause();
    assert!(matches!(alice.write_organisation_to_chain(target).await, Ok(WriteReport::Unknown { .. })));
    tokio::time::resume();
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    // The adoption comes back with the unknown write's settlement.
    let Verified::Settled { outcome, write } = alice.verify_organisation(target).await else { panic!("not settled") };
    assert!(matches!(*outcome, Verified::Adopted { epoch, own: true, .. } if epoch == Epoch::new(2)), "{outcome:?}");
    assert_eq!(write, WriteOutcome::Adopted);
    assert!(
        !matches!(alice.write_organisation_to_chain(target).await, Err(org_io::WriteRefused::OutcomeUnknown)),
        "no longer refused as unknown"
    );
}

// verifies: LLR-t2pve7
#[tokio::test(flavor = "current_thread")]
async fn a_write_that_executed_before_a_restart_is_adopted_by_the_next_verification() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "verify-after-restart";
    let alice = handle(tag, "a", &chain);
    let (org, _) = founded(&alice, "alice").await;
    proposed(&alice, org, &handle(tag, "b", &chain)).await;
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    tokio::time::pause();
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await, Ok(WriteReport::Unknown { .. })));
    tokio::time::resume();
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    drop(alice);
    let reopened = OrgIo::open_for_test(&store_dir(tag, "a"), TransportMode::Loopback, &chain).unwrap();
    assert!(
        matches!(reopened.verify_organisation(HeldTarget::Organisation(org)).await, Verified::Adopted { epoch, .. } if epoch == Epoch::new(2))
    );
    assert_eq!(reopened.organisation(org).await.unwrap().epoch, Epoch::new(2));
}

// verifies: LLR-ce42ce
#[tokio::test]
async fn an_unreachable_chain_reports_every_target_unverified_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("verify-unreachable", "a", &chain);
    let (first, _) = founded(&alice, "alice").await;
    let (second, _) = founded(&alice, "alice").await;
    let before = store_bytes("verify-unreachable", "a");
    chain.read_failing.store(true, Ordering::SeqCst);
    for org in [first, second] {
        assert!(matches!(
            alice.verify_organisation(HeldTarget::Organisation(org)).await,
            Verified::Unverified { reason: UnverifiedReason::Read(_), .. }
        ));
    }
    assert_eq!(store_bytes("verify-unreachable", "a"), before, "nothing written");
}

// Adopted and Removed write org-node's store change; InStep, Behind and Inconsistent write nothing
// but a discard or stamp of a received organisation, shown by a behind verification that
// discards one.
// verifies: LLR-zcw3xv
#[tokio::test]
async fn adopted_and_removed_write_the_returned_store_and_the_others_write_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "verify-writes";
    let (alice, bob, org, joiner) = two_members(tag, &chain).await;
    let target = HeldTarget::Organisation(org);
    let before = store_bytes(tag, "a");
    assert!(matches!(alice.verify_organisation(target).await, Verified::InStep { .. }));
    chain.epoch_ahead.store(true, Ordering::SeqCst);
    assert!(matches!(alice.verify_organisation(target).await, Verified::Behind { .. }));
    chain.epoch_ahead.store(false, Ordering::SeqCst);
    let slot = chain.slots().get(&org).unwrap();
    chain.slots().set(org, OrgState { epoch: Epoch::new(1), ..slot });
    assert!(matches!(alice.verify_organisation(target).await, Verified::Inconsistent { .. }));
    chain.slots().set(org, slot);
    // org-node's behind marker, set by `Behind` in memory only, is cleared by an in-step verification.
    assert!(matches!(alice.verify_organisation(target).await, Verified::InStep { .. }));
    assert_eq!(store_bytes(tag, "a"), before, "InStep, Behind and Inconsistent with nothing to discard write nothing");

    remove_b(&alice, &bob, org, joiner).await;
    let reopened = OrgIo::open_for_test(&store_dir(tag, "a"), TransportMode::Loopback, &chain).unwrap();
    assert_eq!(reopened.organisation(org).await.unwrap().epoch, Epoch::new(3), "Adopted wrote the new record");
    drop(reopened);
    assert!(matches!(bob.verify_organisation(target).await, Verified::Removed { .. }));
    let file = common::handles::file_of(&store_dir(tag, "b"), &joiner);
    assert!(file.is_file(), "the Persona's store file stays");
    let reopened_b = OrgIo::open_for_test(&store_dir(tag, "b"), TransportMode::Loopback, &chain).unwrap();
    assert!(reopened_b.organisations().await.is_empty(), "the removed store holds no record");
    // Through the reopened handle's report, not org-node's store.
    assert!(reopened_b.acknowledgement_report(org).await.is_some_and(|report| !report.recipients.is_empty()), "it keeps the acknowledgements");

    // A behind verification that discards a received organisation writes the discard, and only
    // it: the record is unchanged.
    let tag = "verify-writes-discard";
    let chain = FakeChain::new();
    let (bob, org, bob_device) = received_ahead_of_the_chain(tag, &chain).await;
    let slot = chain.slots().get(&org).unwrap();
    chain.slots().set(org, OrgState { epoch: Epoch::new(4), ..slot });
    let before = store_bytes(tag, "b");
    let target = HeldTarget::Organisation(org);
    assert_eq!(bob.verify_organisation(target).await, Verified::Behind { chain_epoch: Epoch::new(4) });
    assert!(store_bytes(tag, "b") != before, "Behind wrote the discard");
    let reopened_b = reopened_service(tag, "b", &bob);
    assert!(reopened_b.provisional_organisations(&bob_device).unwrap().is_empty(), "the received organisation is discarded in the file");
    drop(reopened_b);
    assert_eq!(bob.organisation(org).await.map(|record| record.epoch), Some(Epoch::new(2)), "the record is unchanged");
}

// verifies: LLR-zcw3xv
#[tokio::test]
async fn a_failed_write_is_verified_store_and_reloads_the_persona() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("verify-write-fails", "a", &chain);
    let (org, _) = founded(&alice, "alice").await;
    proposed(&alice, org, &handle("verify-write-fails", "b", &chain)).await;
    alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap();
    org_io::test_support::fail_next_write_after_temporary();
    assert!(matches!(alice.verify_organisation(HeldTarget::Organisation(org)).await, Verified::Store { .. }));
    assert_eq!(alice.organisation(org).await.unwrap().epoch, Epoch::new(1), "reloaded from its file: not adopted");
    assert!(
        matches!(alice.verify_organisation(HeldTarget::Organisation(org)).await, Verified::Adopted { .. }),
        "the next verification adopts"
    );
}

// Abnormal: the store write of a Removed outcome fails. The
// verification is `Store`, the Persona is reloaded from its file (its record still held), and no
// acknowledgement's delivery starts; the next verification removes it and starts the delivery.
// verifies: LLR-zcw3xv
#[tokio::test]
async fn a_failed_write_of_a_removal_is_verified_store_and_starts_no_delivery() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, bob, org, joiner) = two_members("verify-removal-write-fails", &chain).await;
    remove_b(&alice, &bob, org, joiner).await;
    let target = HeldTarget::Organisation(org);
    org_io::test_support::fail_next_write_after_temporary();
    assert!(matches!(bob.verify_organisation(target).await, Verified::Store { .. }), "the failed write is the outcome");
    // The delivery is seen through the handle's report, not a count of its tasks. A delivery's start records the removal the report reads, and its first round sets the
    // next round's time.
    assert!(bob.acknowledgement_report(org).await.is_none(), "no delivery started while the file still holds the record");
    assert_eq!(bob.organisation(org).await.map(|record| record.epoch), Some(Epoch::new(2)), "reloaded from its file");
    assert_eq!(bob.verify_organisation(target).await, Verified::Removed { org_id: org }, "the next verification removes it");
    let started = tokio::time::timeout(NET, async {
        while !bob.acknowledgement_report(org).await.is_some_and(|report| report.next_round.is_some()) {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await;
    assert!(started.is_ok(), "and starts its delivery: its first round runs");
}

// Abnormal: on a behind verification, the store write of the discard org-node's verification
// returns fails. As an adoption's failed write, the verification is `Store`, not `Behind`, and the
// store file is as it was, the received organisation still in it; the next verification is
// `Behind` and writes the discard.
// verifies: LLR-zcw3xv
#[tokio::test]
async fn a_failed_write_of_a_discard_on_a_behind_verification_is_verified_store_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "verify-discard-write-fails";
    let (bob, org, bob_device) = received_ahead_of_the_chain(tag, &chain).await;
    let held = || reopened_service(tag, "b", &bob).provisional_organisations(&bob_device).unwrap().len();
    let slot = chain.slots().get(&org).unwrap();
    // The chain at epoch 4: B's record is at 2 and its received epoch 3 is below the chain's.
    chain.slots().set(org, OrgState { epoch: Epoch::new(4), ..slot });
    let target = HeldTarget::Organisation(org);
    let before = store_bytes(tag, "b");
    org_io::test_support::fail_next_write_after_temporary();
    assert!(matches!(bob.verify_organisation(target).await, Verified::Store { .. }), "the failed write is the outcome");
    assert_eq!(store_bytes(tag, "b"), before, "the store file is as it was");
    assert_eq!(held(), 1, "the received organisation is still held");
    assert_eq!(bob.verify_organisation(target).await, Verified::Behind { chain_epoch: Epoch::new(4) }, "the next verification");
    assert_eq!(held(), 0, "and it writes the discard");
}

// Abnormal: the Persona reloaded after that failed write keeps the behind marker the verification
// set, though its file never holds it: a revocation is still refused as behind until a
// verification clears the marker.
// verifies: LLR-zcw3xv
#[tokio::test]
async fn a_failed_write_of_a_discard_on_a_behind_verification_keeps_the_organisation_behind() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "verify-discard-write-fails-behind";
    let (bob, org, _) = received_ahead_of_the_chain(tag, &chain).await;
    let slot = chain.slots().get(&org).unwrap();
    chain.slots().set(org, OrgState { epoch: Epoch::new(4), ..slot });
    org_io::test_support::fail_next_write_after_temporary();
    assert!(matches!(bob.verify_organisation(HeldTarget::Organisation(org)).await, Verified::Store { .. }), "fixture: the write fails");
    let refused = bob.propose_revocation(org, org_node::MemberId::new([0x02; 32])).await;
    assert!(
        matches!(refused, Err(org_io::OperationError::Node(org_node::error::OrgNodeError::OrganisationBehind { chain_epoch, .. })) if chain_epoch == Epoch::new(4)),
        "still behind after the reload: {refused:?}"
    );
}

// verifies: LLR-4qyeut
#[tokio::test]
async fn a_founding_is_read_at_its_stored_proxy_and_none_yet_is_no_proxy_yet() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("verify-founding", "a", &chain);
    let founding = alice.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap();
    let target = HeldTarget::Founding(founding.founder);
    let reads = chain.reads();
    assert_eq!(alice.verify_organisation(target).await, Verified::Unverified { reason: UnverifiedReason::NoProxyYet });
    assert_eq!(chain.reads(), reads, "no read");
    assert!(matches!(alice.write_organisation_to_chain(target).await.unwrap(), WriteReport::Included { .. }));
    assert!(
        matches!(alice.verify_organisation(target).await, Verified::Adopted { epoch, own: true, .. } if epoch == Epoch::new(1)),
        "read at the stored proxy and adopted"
    );
    let org = alice.organisations().await[0].org_id;
    assert!(
        matches!(alice.verify_organisation(HeldTarget::Organisation(org)).await, Verified::InStep { .. }),
        "addressed by its Organisation from then on"
    );
}

// verifies: LLR-kq7z58
#[tokio::test]
async fn verify_all_verifies_every_target_in_store_file_order_whatever_fails() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("verify-all", "a", &chain);
    let (org, founder) = founded(&alice, "alice").await;
    let founding = alice.propose_founding_organisation("Carol", "Doe", "carol", "Other").await.unwrap();
    let invite = org_node::invitation::Invite {
        org_name: "Elsewhere".into(),
        org_id: OrgId::new([0x64; 20]),
        invitee_name: "Alice".into(),
        issuer: org_node::test_fixtures::device_key(0x65),
        issuer_addresses: org_node::invitation::EndpointAddresses::parse(vec![]).unwrap(),
        invite_id: org_node::invitation::InviteId::new([0x66; 32]),
    };
    let member = alice.propose_member(&invite.encode().unwrap(), "Alice", "Doe", &["alice"]).await.unwrap();
    let mut expected = [
        (founder, HeldTarget::Organisation(org)),
        (founding.founder, HeldTarget::Founding(founding.founder)),
        (member.member_device, HeldTarget::Organisation(OrgId::new([0x64; 20]))),
    ];
    expected.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let all = alice.verify_all().await;
    let targets: Vec<HeldTarget> = all.iter().map(|(target, _)| *target).collect();
    assert_eq!(targets, expected.iter().map(|(_, target)| *target).collect::<Vec<_>>(), "every target, in store-file order");
    for (target, verified) in &all {
        match *target {
            HeldTarget::Founding(_) => {
                assert!(matches!(verified, Verified::Unverified { reason: UnverifiedReason::NoProxyYet, .. }))
            }
            HeldTarget::Organisation(id) if id == org => assert!(matches!(verified, Verified::InStep { .. })),
            HeldTarget::Organisation(_) => assert!(
                matches!(verified, Verified::Unverified { reason: UnverifiedReason::OrgNotOnChain, .. }),
                "{verified:?}"
            ),
        }
    }
}

// verifies: LLR-kq7z58
#[tokio::test]
async fn verify_all_reports_every_target_in_store_file_order() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("verify-all-order", "a", &chain);
    let mut held_here = [founded(&alice, "alice").await, founded(&alice, "alice").await, founded(&alice, "alice").await];
    held_here.sort_by(|left, right| left.1.as_bytes().cmp(right.1.as_bytes()));
    let all = alice.verify_all().await;
    assert_eq!(all.len(), 3);
    for ((org, _), (target, verified)) in held_here.iter().zip(&all) {
        assert_eq!((*target, verified.clone()), (HeldTarget::Organisation(*org), Verified::InStep { epoch: Epoch::new(1) }));
    }
}

// verifies: LLR-fp2cat
#[tokio::test]
async fn a_provisional_member_reads_the_invites_organisation_and_adopts_nothing_itself() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "verify-member";
    let alice = handle(tag, "a", &chain);
    let (org, founder) = founded(&alice, "alice").await;
    let chain_b = FakeChain::sharing_slots_with(&chain);
    let bob = handle(tag, "b", &chain_b);
    let blob = alice.issue_invite(org, "Bob").await.unwrap();
    bob.propose_member(&blob, "Bob", "Builder", &["bob"]).await.unwrap();
    let target = HeldTarget::Organisation(org);
    assert_eq!(
        bob.verify_organisation(target).await,
        Verified::Behind { chain_epoch: Epoch::new(1) },
        "the Invite's Organisation read; nothing to adopt"
    );
    assert_eq!(chain_b.reads(), 1, "read at the Invite's Organisation");

    // An epoch that admits someone else reaches B: org-node adopts nothing that does not list B.
    let carol = handle(tag, "c", &chain);
    proposed(&alice, org, &carol).await;
    alice.write_organisation_to_chain(target).await.unwrap();
    alice.verify_organisation(target).await;
    // A plays at the wire: the epoch's message for C, as org-node gives
    // it for the store A's handle wrote, dialled from A's Device to B, whose receive step keeps it.
    let to_c = common::handles::reopened_service(tag, "a", &alice).messages_for_current_epoch(&founder).unwrap().into_iter().next().unwrap();
    bob.start_receiving().await.unwrap();
    let to = journey::address_of(&bob).await;
    let (read_back, received) = tokio::join!(dial_direct_as(&founder, to, &to_c.body), journey::next(&bob));
    assert_eq!(read_back.unwrap(), vec![0x00], "B's receive step accepted it: {received:?}");
    // org-node refuses the admitting epoch as `AdmissionNotOurs`, and the outcome is that
    // refusal, as org-node words it.
    assert_eq!(
        bob.verify_organisation(target).await,
        Verified::Refused { error: org_io::Cause::new(org_node::OrgNodeError::AdmissionNotOurs { org_id: org }) },
        "reported as org-node reports it"
    );
    assert!(bob.organisations().await.is_empty(), "nothing adopted");
}

// verifies: LLR-gaanm2
#[tokio::test]
async fn discarded_lists_each_left_out_change_with_its_kind_names_and_handle() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "verify-discarded";
    let alice = handle(tag, "a", &chain);
    let (org, _) = founded(&alice, "alice").await;
    let target = HeldTarget::Organisation(org);
    proposed(&alice, org, &handle(tag, "b", &chain)).await;
    assert!(matches!(alice.write_organisation_to_chain(target).await.unwrap(), WriteReport::Included { .. }));
    let late = org_node::invitation::Invite::parse(&alice.issue_invite(org, "Carl").await.unwrap()).unwrap();
    common::handles::requested(&alice, &handle(tag, "c", &chain), &late.encode().unwrap(), &["carl"]).await;
    alice.propose_admission(org, late.invite_id, None).await.unwrap();
    let own = alice.organisation(org).await.unwrap().members[0].clone();
    alice.propose_revocation(org, own.id).await.unwrap();
    let Verified::Adopted { discarded, .. } = alice.verify_organisation(target).await else { panic!("not adopted") };
    assert_eq!(
        discarded,
        vec![
            DiscardedChangeSummary::Admission {
                invite_id: late.invite_id,
                name: Some(org_node::Name::parse("Bob").unwrap()),
                surname: Some(org_node::Surname::parse("Builder").unwrap()),
                handle: org_node::Handle::parse("carl").unwrap(),
            },
            DiscardedChangeSummary::Revocation { member_id: own.id, handle: Some(own.handle) },
        ],
        "in org-node's order, each with its kind, names and Handle"
    );
}

// Boundary: an adoption that leaves nothing out lists no discarded
// change. The Admin holds only the change its written epoch carries, so org-node's adoption
// leaves none out and `discarded` is empty.
// verifies: LLR-gaanm2
#[tokio::test]
async fn an_adoption_that_leaves_nothing_out_lists_no_discarded_change() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "verify-discarded-none";
    let alice = handle(tag, "a", &chain);
    let (org, _) = founded(&alice, "alice").await;
    let target = HeldTarget::Organisation(org);
    proposed(&alice, org, &handle(tag, "b", &chain)).await;
    assert!(matches!(alice.write_organisation_to_chain(target).await.unwrap(), WriteReport::Included { .. }));
    let verified = alice.verify_organisation(target).await;
    assert!(matches!(&verified, Verified::Adopted { discarded, own: true, .. } if discarded.is_empty()), "{verified:?}");
}

/// The outcome and the settled write of a verification that found an
/// unknown write recorded; anything else fails the test.
fn settle_parts(verified: Verified) -> (Verified, WriteOutcome) {
    match verified {
        Verified::Settled { outcome, write } => (*outcome, write),
        other => panic!("no unknown write settled: {other:?}"),
    }
}

/// `tag`'s Admin with an epoch-2 admission proposed and its write left
/// unknown (`lands`: the chain moved before the bound elapsed). The caller
/// holds `fresh_keychain`'s lock.
async fn settle_unknown_update(tag: &str, lands: bool) -> Admin {
    let admin = journey::admin(tag).await;
    let bob = invited(&admin, tag, "b", "bob").await;
    keep_request(&admin, &bob).await;
    assert_eq!(propose_admission(&admin, &bob).await, Epoch::new(2));
    unknown_write(&admin.io, &admin.chain, HeldTarget::Organisation(admin.org), lands).await;
    admin
}

/// The Admin's provisional changes and provisional organisations, counted in the store file the
/// Admin's handle wrote (read apart from the handle).
fn settle_held(admin: &Admin) -> (usize, usize) {
    let node = common::handles::reopened_from(&admin.dir, &admin.io);
    (node.provisional_changes(&admin.device_key).unwrap().len(), node.provisional_organisations(&admin.device_key).unwrap().len())
}

// Normal and abnormal, one target each: an unknown write the chain shows
// executed is reported adopted and cleared; one past its window at the base
// epoch is not executed; one overtaken by another root is deprecated with the
// Organisation behind; one still within its window stays unknown and kept, so
// the next write is still refused.
// verifies: LLR-zey4mg
#[tokio::test]
async fn an_unknown_write_is_settled_adopted_not_executed_deprecated_or_still_unknown() {
    let (_guard, _keychain) = fresh_keychain();
    let landed = settle_unknown_update("settle-all-adopted", true).await;
    let (outcome, write) = settle_parts(landed.io.verify_organisation(HeldTarget::Organisation(landed.org)).await);
    assert!(matches!(outcome, Verified::Adopted { .. }), "{outcome:?}");
    assert_eq!(write, WriteOutcome::Adopted);
    let again = landed.io.verify_organisation(HeldTarget::Organisation(landed.org)).await;
    assert!(matches!(again, Verified::InStep { .. }), "cleared: {again:?}");

    let lapsed = settle_unknown_update("settle-all-not-executed", false).await;
    lapsed.chain.advance_finalised(65);
    let (outcome, write) = settle_parts(lapsed.io.verify_organisation(HeldTarget::Organisation(lapsed.org)).await);
    assert!(matches!(outcome, Verified::InStep { .. }), "{outcome:?}");
    assert_eq!(write, WriteOutcome::NotExecuted);

    let overtaken = settle_unknown_update("settle-all-deprecated", false).await;
    chain_moved_elsewhere(&overtaken.chain, overtaken.org, 2);
    let (outcome, write) = settle_parts(overtaken.io.verify_organisation(HeldTarget::Organisation(overtaken.org)).await);
    assert!(matches!(outcome, Verified::Behind { .. }), "{outcome:?}");
    assert_eq!(write, WriteOutcome::Deprecated);

    let pending = settle_unknown_update("settle-all-still", false).await;
    for _ in 0..2 {
        let (outcome, write) = settle_parts(pending.io.verify_organisation(HeldTarget::Organisation(pending.org)).await);
        assert!(matches!(outcome, Verified::InStep { .. }), "{outcome:?}");
        assert_eq!(write, WriteOutcome::StillUnknown, "kept, and reported again");
    }
    let refused = pending.io.write_organisation_to_chain(HeldTarget::Organisation(pending.org)).await;
    assert!(matches!(refused, Err(org_io::WriteRefused::OutcomeUnknown)), "{refused:?}");
}

// Boundary: a founding's unknown write has no window — far past any block
// count it stays unknown while the chain holds no genesis, and is adopted
// once the chain shows it. (The fake chain's founding never lands then
// hangs, so the proxy is stored by a refused first attempt, the retry's
// `found_at` hangs, and the genesis is put on the chain by hand.)
// verifies: LLR-zey4mg
#[tokio::test]
async fn a_foundings_unknown_write_is_settled_only_adopted_or_still_unknown() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let io = handle("settle-founding", "a", &chain);
    let proposed = io.propose_founding_organisation("Alice", "Admin", "alice", "Acme").await.unwrap();
    let founder = proposed.founder;
    let target = HeldTarget::Founding(founder);
    chain.found_at_failing.store(true, Ordering::SeqCst);
    let refused = io.write_organisation_to_chain(target).await.unwrap();
    chain.found_at_failing.store(false, Ordering::SeqCst);
    assert!(matches!(refused, WriteReport::Refused { .. }), "the proxy stored, the genesis not written: {refused:?}");
    unknown_write(&io, &chain, target, false).await;
    chain.advance_finalised(10_000);
    let (outcome, write) = settle_parts(io.verify_organisation(target).await);
    assert!(matches!(outcome, Verified::Unverified { .. }), "no genesis on the chain: {outcome:?}");
    assert_eq!(write, WriteOutcome::StillUnknown, "a founding is never not-executed");

    // The Organisation the handle lists for the founder (recorded with the stored proxy) and the founding's root and key as its proposal returned them.
    let org = io.personas().await.into_iter().find(|persona| persona.device_key == founder).and_then(|persona| persona.org_id).unwrap();
    chain.slots().set(org, OrgState { root_hash: proposed.root, org_pub_key: proposed.org_public_key, epoch: Epoch::new(1) });
    let (outcome, write) = settle_parts(io.verify_organisation(target).await);
    assert!(matches!(outcome, Verified::Adopted { .. }), "{outcome:?}");
    assert_eq!(write, WriteOutcome::Adopted);
}

// Abnormal, one cause after another on one Admin holding a provisional change
// and a provisional organisation: a refused write, an unreadable chain, a
// chain ahead on another root (Behind) and a chain at the record's epoch on
// another root (Inconsistent) each leave both held and the store file's
// bytes unchanged.
// verifies: LLR-pbng7d
#[tokio::test]
async fn nothing_is_discarded_on_behind_inconsistent_unverified_or_a_refused_write() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "settle-no-discard";
    let admin = journey::admin(tag).await;
    let bob = invited(&admin, tag, "b", "bob").await;
    keep_request(&admin, &bob).await;
    propose_admission(&admin, &bob).await;
    let (held, files) = (settle_held(&admin), store_bytes(tag, "a"));
    assert_eq!(held, (1, 1));
    let target = HeldTarget::Organisation(admin.org);

    admin.chain.write_failing.store(true, Ordering::SeqCst);
    let refused = admin.io.write_organisation_to_chain(target).await.unwrap();
    admin.chain.write_failing.store(false, Ordering::SeqCst);
    assert!(matches!(refused, WriteReport::Refused { .. }), "{refused:?}");
    assert_eq!((settle_held(&admin), store_bytes(tag, "a")), (held, files.clone()), "a refused write");

    admin.chain.read_failing.store(true, Ordering::SeqCst);
    let unverified = admin.io.verify_organisation(target).await;
    admin.chain.read_failing.store(false, Ordering::SeqCst);
    assert!(matches!(unverified, Verified::Unverified { .. }), "{unverified:?}");
    assert_eq!((settle_held(&admin), store_bytes(tag, "a")), (held, files.clone()), "unverified");

    let on_chain = admin.chain.slots().get(&admin.org).unwrap();
    chain_moved_elsewhere(&admin.chain, admin.org, 3);
    let behind = admin.io.verify_organisation(target).await;
    assert!(matches!(behind, Verified::Behind { .. }), "{behind:?}");
    assert_eq!((settle_held(&admin), store_bytes(tag, "a")), (held, files.clone()), "behind");

    chain_moved_elsewhere(&admin.chain, admin.org, 1);
    let inconsistent = admin.io.verify_organisation(target).await;
    assert!(matches!(inconsistent, Verified::Inconsistent { .. }), "{inconsistent:?}");
    assert_eq!((settle_held(&admin), store_bytes(tag, "a")), (held, files), "inconsistent");
    admin.chain.slots().set(admin.org, on_chain);
}

// The closure is called only on this node's removal and reads the keychain
// item of the Device org-node names, once: an in-step verification reads
// the keychain not at all. A locked keychain at the removal is
// `Verified::Keychain`, nothing written; unlocked, the next verification
// removes.
// verifies: LLR-8w2z64
#[tokio::test]
async fn the_secrets_closure_reads_the_keychain_only_to_sign_and_a_refusal_is_verified_keychain() {
    let (_guard, keychain) = fresh_keychain();
    let tag = "settle-secrets";
    let admin = journey::admin(tag).await;
    let bob = invited(&admin, tag, "b", "bob").await;
    admit(&admin, &bob).await;
    epoch_kept(&admin.io, admin.org, &bob.io).await;
    let adopted = bob.io.verify_all().await;
    assert!(matches!(adopted.as_slice(), [(_, Verified::Adopted { .. })]), "{adopted:?}");

    let calls = keychain.calls();
    let in_step = bob.io.verify_all().await;
    assert!(matches!(in_step.as_slice(), [(_, Verified::InStep { .. })]), "{in_step:?}");
    assert_eq!(keychain.calls(), calls, "no keychain read without a removal");

    let member = member_id_of(&admin, bob.device_key).await;
    admin.io.propose_revocation(admin.org, member).await.unwrap();
    admin.io.propose_updated_organisation(admin.org).await.unwrap();
    journey::write_and_adopt(&admin).await;
    epoch_kept(&admin.io, admin.org, &bob.io).await;
    let files = store_bytes(tag, "b");

    keychain.refuse_as(Some(Refusal::Locked));
    let refused = bob.io.verify_all().await;
    keychain.refuse_as(None);
    assert!(matches!(refused.as_slice(), [(_, Verified::Keychain { .. })]), "{refused:?}");
    assert_eq!(store_bytes(tag, "b"), files, "nothing written");
    assert!(bob.io.acknowledgement_report(admin.org).await.is_none(), "no acknowledgement delivery starts");

    let calls = keychain.calls();
    let removed = bob.io.verify_all().await;
    assert!(matches!(removed.as_slice(), [(_, Verified::Removed { .. })]), "{removed:?}");
    assert_eq!(keychain.calls(), calls + 1, "one read: the Persona's item, to sign");
}

// The chain moved to another root at the epoch the write expected.
// verifies: LLR-zey4mg
#[tokio::test]
async fn an_unknown_write_whose_chain_moved_elsewhere_is_deprecated() {
    let (_guard, _keychain) = fresh_keychain();
    let admin = settle_unknown_update("settle-deprecated", false).await;
    chain_moved_elsewhere(&admin.chain, admin.org, 2);
    let (outcome, write) = settle_parts(admin.io.verify_organisation(HeldTarget::Organisation(admin.org)).await);
    assert!(matches!(outcome, Verified::Behind { .. }), "{outcome:?}");
    assert_eq!(write, WriteOutcome::Deprecated);
    let next = admin.io.verify_organisation(HeldTarget::Organisation(admin.org)).await;
    assert!(matches!(next, Verified::Behind { .. }), "cleared: {next:?}");
}

// Abnormal: a failed read leaves an unknown write still unknown.
// verifies: LLR-zey4mg
#[tokio::test]
async fn a_failed_read_leaves_an_unknown_write_still_unknown() {
    let (_guard, _keychain) = fresh_keychain();
    let admin = settle_unknown_update("settle-failed-read", true).await;
    admin.chain.read_failing.store(true, Ordering::SeqCst);
    let (outcome, write) = settle_parts(admin.io.verify_organisation(HeldTarget::Organisation(admin.org)).await);
    admin.chain.read_failing.store(false, Ordering::SeqCst);
    assert!(matches!(outcome, Verified::Unverified { .. }), "{outcome:?}");
    assert_eq!(write, WriteOutcome::StillUnknown);
    let (_, write) = settle_parts(admin.io.verify_organisation(HeldTarget::Organisation(admin.org)).await);
    assert_eq!(write, WriteOutcome::Adopted, "kept, then settled by the next read");
}

// Boundary: at the window's last valid block the write may still execute.
// verifies: LLR-zey4mg
#[tokio::test]
async fn within_the_window_an_unknown_write_stays_still_unknown() {
    let (_guard, _keychain) = fresh_keychain();
    let admin = settle_unknown_update("settle-within-window", false).await;
    admin.chain.advance_finalised(64);
    let (_, write) = settle_parts(admin.io.verify_organisation(HeldTarget::Organisation(admin.org)).await);
    assert_eq!(write, WriteOutcome::StillUnknown, "block 64 is still the window's");
}

// Boundary: one block past the window, the chain still at the base epoch.
// verifies: LLR-zey4mg
#[tokio::test]
async fn past_the_window_at_the_base_epoch_the_write_is_not_executed() {
    let (_guard, _keychain) = fresh_keychain();
    let admin = settle_unknown_update("settle-past-window", false).await;
    admin.chain.advance_finalised(65);
    let (outcome, write) = settle_parts(admin.io.verify_organisation(HeldTarget::Organisation(admin.org)).await);
    assert!(matches!(outcome, Verified::InStep { .. }), "{outcome:?}");
    assert_eq!(write, WriteOutcome::NotExecuted);
    assert_eq!(settle_held(&admin), (1, 1), "org-io discards nothing itself: the proposal stays held");
    let retried = admin.io.write_organisation_to_chain(HeldTarget::Organisation(admin.org)).await.unwrap();
    assert!(matches!(retried, WriteReport::Included { .. }), "cleared, so a write is made again: {retried:?}");
}

// Abnormal: the chain holds exactly the unknown write, but the verification that read it could
// not write the adoption (the store write failed, the Persona reloaded). The write did not lose
// to another root, so it is not deprecated: it stays unknown and kept, and the next verification
// adopts it.
// verifies: LLR-zey4mg
#[tokio::test]
async fn an_unknown_write_on_chain_whose_adoption_failed_to_write_stays_still_unknown() {
    let (_guard, _keychain) = fresh_keychain();
    let admin = settle_unknown_update("settle-landed-store-fails", true).await;
    let target = HeldTarget::Organisation(admin.org);
    org_io::test_support::fail_next_write_after_temporary();
    let (outcome, write) = settle_parts(admin.io.verify_organisation(target).await);
    assert!(matches!(outcome, Verified::Store { .. }), "{outcome:?}");
    assert_eq!(write, WriteOutcome::StillUnknown, "its own write on chain is never deprecated");
    // Kept: the next verification still settles it.
    let (outcome, write) = settle_parts(admin.io.verify_organisation(target).await);
    assert!(matches!(outcome, Verified::Adopted { .. }), "{outcome:?}");
    assert_eq!(write, WriteOutcome::Adopted);
}

// Boundary: past the window, but the chain is below the epoch the write was based on (another
// history), not at it. Only a chain at the base epoch shows the write can no longer execute, so
// it stays unknown and kept.
// verifies: LLR-zey4mg
#[tokio::test]
async fn past_the_window_below_the_base_epoch_the_write_stays_still_unknown() {
    let (_guard, _keychain) = fresh_keychain();
    let admin = settle_unknown_update("settle-below-base", false).await;
    let target = HeldTarget::Organisation(admin.org);
    chain_moved_elsewhere(&admin.chain, admin.org, 0);
    admin.chain.advance_finalised(65);
    let (outcome, write) = settle_parts(admin.io.verify_organisation(target).await);
    assert!(!matches!(outcome, Verified::Adopted { .. } | Verified::InStep { .. }), "{outcome:?}");
    assert_eq!(write, WriteOutcome::StillUnknown, "not executed only at the base epoch");
    assert!(
        matches!(admin.io.write_organisation_to_chain(target).await, Err(org_io::WriteRefused::OutcomeUnknown)),
        "kept: a write is still refused as unknown"
    );
}

// A verification that adopts nothing discards nothing.
// verifies: LLR-pbng7d
#[tokio::test]
async fn a_verification_that_adopts_nothing_discards_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "settle-in-step";
    let admin = journey::admin(tag).await;
    let bob = invited(&admin, tag, "b", "bob").await;
    keep_request(&admin, &bob).await;
    admin.io.propose_admission(admin.org, bob.invite_id, None).await.unwrap();
    let files = store_bytes(tag, "a");
    let in_step = admin.io.verify_organisation(HeldTarget::Organisation(admin.org)).await;
    assert!(matches!(in_step, Verified::InStep { .. }), "{in_step:?}");
    assert_eq!(settle_held(&admin).0, 1, "the provisional change stays held");
    assert_eq!(store_bytes(tag, "a"), files, "nothing written");
}
