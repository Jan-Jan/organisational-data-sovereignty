#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! Writing a provisional organisation to the chain, an update or a founding (SDD-mk39e5):
//! `write_organisation_to_chain` writes the latest own provisional organisation
//! through `ChainWriter` and reports included, refused or unknown (LLR-s7rq5x), refusing first
//! what it must not write (LLR-48me7m, LLR-mhn4n5); every call is bounded (LLR-d3farr); the window
//! is read once (LLR-a66ufr); an unknown write is recorded (LLR-7y46be).

use std::sync::atomic::Ordering;

use org_node::{ChainAccount, Epoch, OrgId};
use org_io::send::Delivery;
use org_io::test_support::FakeChain;
use org_io::transport::TransportMode;
use org_io::write::{HeldTarget, WriteFailure, WriteRefused, WRITE_TIMEOUT};
use org_io::{OrgIo, ProposedOrganisation, Received, Verified, WriteReport};

mod common;
use common::handles::{founded, fresh_keychain, handle, proposed, reopened_service, store_bytes, NET};

/// A's handle over `chain` holding an Organisation its founding write created (its proxy known
/// to the chain's writer) with its next epoch proposed.
async fn ready(tag: &str, chain: &FakeChain, seed: u8) -> (OrgIo, OrgId, ProposedOrganisation) {
    let alice = handle(tag, "a", chain);
    let (org, _) = founded(&alice, "alice").await;
    let next = proposed(&alice, org, &handle(tag, &format!("b{seed}"), chain)).await;
    (alice, org, next)
}

/// `write_organisation_to_chain(Organisation(org))` with the clock paused, so the 90-second bound
/// passes at once when the write never finishes.
async fn written_paused(alice: &OrgIo, org: OrgId) -> Result<WriteReport, WriteRefused> {
    tokio::time::pause();
    let written = alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await;
    tokio::time::resume();
    written
}

/// Whether A's store in test `tag` keeps `next` as a provisional organisation of `org`: read from
/// the store file A's handle wrote, apart from the handle.
async fn still_held(tag: &str, alice: &OrgIo, org: OrgId, next: &ProposedOrganisation) -> bool {
    let founder = alice.personas().await.into_iter().find(|persona| persona.org_id == Some(org)).unwrap().device_key;
    reopened_service(tag, "a", alice)
        .provisional_organisations(&founder)
        .unwrap()
        .iter()
        .any(|view| view.root == next.root && view.org_pub_key == next.org_public_key)
}
// verifies: LLR-s7rq5x
#[tokio::test]
async fn an_update_is_written_and_reported_included_with_no_state_read_and_no_adoption() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, next) = ready("write-included", &chain, 0x31).await;
    let (before, reads) = (store_bytes("write-included", "a"), (chain.reads(), chain.finalised_reads()));
    assert_eq!(
        alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap(),
        WriteReport::Included { epoch: Epoch::new(2) },
        "included, naming the epoch it wrote"
    );
    let slot = chain.slots().get(&org).unwrap();
    assert_eq!((slot.epoch, slot.root_hash, slot.org_pub_key), (Epoch::new(2), next.root, next.org_public_key), "the chain holds the next epoch");
    assert_eq!((chain.reads(), chain.finalised_reads()), reads, "no state read");
    assert_eq!(alice.organisation(org).await.unwrap().epoch.get(), 1, "no adoption");
    assert!(still_held("write-included", &alice, org, &next).await, "the provisional organisation stays held until verification");
    assert_eq!(store_bytes("write-included", "a"), before, "no store change");
}

// Each refusal comes before any chain call and writes nothing: no chain call, and the store
// files of the refused handle are byte for byte as they were.
// verifies: LLR-48me7m
#[tokio::test]
async fn not_held_no_proxy_and_stale_are_refused_before_any_chain_call() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "write-refused-first";
    let alice = handle(tag, "a", &chain);
    let bob = handle(tag, "b", &chain);
    let (org, founder) = founded(&alice, "alice").await;
    let files = store_bytes(tag, "a");
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(OrgId::new([0x77; 20]))).await, Err(WriteRefused::NotHeld)));
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await, Err(WriteRefused::NotHeld)), "no own provisional organisation");
    assert_eq!(store_bytes(tag, "a"), files, "NotHeld writes no store");

    // Epoch 2 admits B and is written and adopted by both: B's record is created by a first
    // admission and holds no proxy account.
    // Every step goes through the handles — A's write and verification, A's
    // send of the epoch taken by B's receive step, B's verification.
    proposed(&alice, org, &bob).await;
    let joiner = bob.personas().await[0].device_key;
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap(), WriteReport::Included { .. }));
    assert!(matches!(alice.verify_organisation(HeldTarget::Organisation(org)).await, Verified::Adopted { .. }));
    let (received, sent) = tokio::join!(tokio::time::timeout(NET, bob.next_received()), alice.send_organisation(org));
    assert!(matches!(received.expect("in time"), Ok(Received::ProvisionalOrganisationReceived { .. })));
    assert!(sent.unwrap().per_device.iter().all(|(_, delivery)| matches!(delivery, Delivery::Delivered)));
    assert!(matches!(bob.verify_all().await.as_slice(), [(_, Verified::Adopted { .. })]));
    let writes = (chain.update_writes(), chain.write_windows());

    let bob_member = bob.organisation(org).await.unwrap().members.iter().find(|member| member.device_keys.contains(&joiner)).unwrap().id;
    bob.propose_revocation(org, bob_member).await.unwrap();
    bob.propose_updated_organisation(org).await.unwrap();
    let files = store_bytes(tag, "b");
    assert!(matches!(bob.write_organisation_to_chain(HeldTarget::Organisation(org)).await, Err(WriteRefused::NoProxyAccount)));
    assert_eq!(store_bytes(tag, "b"), files, "NoProxyAccount writes no store");

    // A proposes epoch 3, then records another change: the proposal is no longer current.
    alice.propose_revocation(org, bob_member).await.unwrap();
    alice.propose_updated_organisation(org).await.unwrap();
    let own = alice.organisation(org).await.unwrap().members.iter().find(|member| member.device_keys.contains(&founder)).unwrap().id;
    alice.propose_revocation(org, own).await.unwrap();
    let files = store_bytes(tag, "a");
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await, Err(WriteRefused::Stale)), "a change recorded after the proposal");
    assert_eq!(store_bytes(tag, "a"), files, "Stale writes no store");
    assert_eq!((chain.update_writes(), chain.write_windows()), writes, "no chain call for any refusal");
}

// verifies: LLR-mhn4n5
#[tokio::test(flavor = "current_thread")]
async fn an_unknown_outcome_refuses_a_second_write_for_that_target_only() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, first, _) = ready("write-unknown-one", &chain, 0x33).await;
    let other_handle = handle("write-unknown-one", "c", &chain);
    let (second, _) = founded(&other_handle, "alice").await;
    proposed(&other_handle, second, &handle("write-unknown-one", "d", &chain)).await;
    chain.write_hanging.store(true, Ordering::SeqCst);
    assert!(matches!(written_paused(&alice, first).await, Ok(WriteReport::Unknown { .. })));
    chain.write_hanging.store(false, Ordering::SeqCst);
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(first)).await, Err(WriteRefused::OutcomeUnknown)));
    assert!(matches!(other_handle.write_organisation_to_chain(HeldTarget::Organisation(second)).await.unwrap(), WriteReport::Included { .. }), "another target writes");
}

// The other target on the same handle: the tests above write it
// through a second handle, which has an unknown-write map of its own. One handle holding two
// Organisations, the first's write left unknown, still writes the second.
// verifies: LLR-mhn4n5
#[tokio::test(flavor = "current_thread")]
async fn an_unknown_outcome_on_one_handle_never_refuses_its_other_target() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "write-unknown-same-handle";
    let chain = FakeChain::new();
    let alice = handle(tag, "a", &chain);
    let mut orgs = vec![];
    for seed in [0x43, 0x44] {
        let (org, _) = founded(&alice, "alice").await;
        proposed(&alice, org, &handle(tag, &format!("b{seed}"), &chain)).await;
        orgs.push(org);
    }
    chain.write_hanging.store(true, Ordering::SeqCst);
    assert!(matches!(written_paused(&alice, orgs[0]).await, Ok(WriteReport::Unknown { .. })));
    chain.write_hanging.store(false, Ordering::SeqCst);
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(orgs[0])).await, Err(WriteRefused::OutcomeUnknown)));
    let other = alice.write_organisation_to_chain(HeldTarget::Organisation(orgs[1])).await;
    assert!(matches!(other, Ok(WriteReport::Included { .. })), "the same handle's other target writes: {other:?}");
}

// verifies: LLR-d3farr, LLR-7y46be
#[tokio::test(flavor = "current_thread")]
async fn a_write_past_ninety_seconds_is_unknown_and_recorded() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, next) = ready("write-past-bound", &chain, 0x35).await;
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    let WriteReport::Unknown { reason } = written_paused(&alice, org).await.unwrap() else { panic!("not unknown") };
    assert!(reason.contains(&format!("{} s", WRITE_TIMEOUT.as_secs())) && reason.contains("may yet execute"), "{reason}");
    assert!(still_held("write-past-bound", &alice, org, &next).await, "the provisional organisation is kept");
    assert_eq!(alice.organisation(org).await.unwrap().epoch.get(), 1, "nothing else changed");
    // The record is seen at the handle, as the refusal it causes.
    assert!(
        matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await, Err(WriteRefused::OutcomeUnknown)),
        "recorded: the next write is refused as unknown"
    );
}

// verifies: LLR-a66ufr
#[tokio::test]
async fn the_window_is_read_once_and_passed_and_its_failure_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    chain.advance_finalised(500);
    let (alice, org, _) = ready("write-window", &chain, 0x36).await;
    alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap();
    assert_eq!(chain.write_windows(), 1, "one window read");
    assert_eq!(chain.last_update_window().unwrap().birth_number, 500, "the window read is the one passed to the write");

    let (other, other_org, _) = ready("write-window-fails", &chain, 0x37).await;
    chain.write_window_failing.store(true, Ordering::SeqCst);
    let writes = chain.update_writes();
    let WriteReport::Refused { failure } = other.write_organisation_to_chain(HeldTarget::Organisation(other_org)).await.unwrap() else {
        panic!("not refused")
    };
    let reason = failure.to_string();
    assert!(reason.contains("no write was made"), "{reason}");
    assert_eq!(chain.update_writes(), writes, "nothing written");
}

// Abnormal: the write window's read never returns. The bound
// holds for it too: past 90 s the write is refused, saying no write was made, with no update
// write asked and no unknown write recorded, and the provisional organisation kept.
// verifies: LLR-d3farr
#[tokio::test(flavor = "current_thread")]
async fn a_window_read_that_never_returns_is_bounded_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, next) = ready("write-window-hangs", &chain, 0x42).await;
    chain.write_window_hanging.store(true, Ordering::SeqCst);
    tokio::time::pause();
    let written = tokio::time::timeout(WRITE_TIMEOUT * 2, alice.write_organisation_to_chain(HeldTarget::Organisation(org))).await;
    tokio::time::resume();
    chain.write_window_hanging.store(false, Ordering::SeqCst);
    let WriteReport::Refused { failure } = written.expect("the window read is bounded").unwrap() else { panic!("not refused") };
    let reason = failure.to_string();
    assert!(reason.contains("write window timed out") && reason.contains("no write was made"), "{reason}");
    assert_eq!(chain.update_writes(), 0, "no update write asked");
    assert!(still_held("write-window-hangs", &alice, org, &next).await);
    let again = alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await;
    assert!(matches!(again, Ok(WriteReport::Included { .. })), "no unknown write: the next write is made: {again:?}");
}

// What the unknown write holds is seen where it is used, the verification's settlement of it
// (no read of the handle's memory): the window's last valid block is
// 1 000 + 64, since the write is still unknown at that block and not executed one block later
// with the chain at the epoch it expected; and its root and key are the provisional
// organisation's, since a chain holding exactly them one epoch on settles it adopted.
// verifies: LLR-7y46be
#[tokio::test(flavor = "current_thread")]
async fn the_unknown_write_holds_root_key_expected_epoch_and_last_valid_block() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    chain.advance_finalised(1_000);
    let (alice, org, _) = ready("write-unknown-record", &chain, 0x38).await;
    let target = HeldTarget::Organisation(org);
    chain.write_hanging.store(true, Ordering::SeqCst);
    written_paused(&alice, org).await.unwrap();
    chain.write_hanging.store(false, Ordering::SeqCst);
    chain.advance_finalised(64);
    let Verified::Settled { write, .. } = alice.verify_organisation(target).await else { panic!("not settled") };
    assert_eq!(write, org_io::WriteOutcome::StillUnknown, "block 1 064 is the window's last valid block");
    chain.advance_finalised(1);
    let Verified::Settled { write, .. } = alice.verify_organisation(target).await else { panic!("not settled") };
    assert_eq!(write, org_io::WriteOutcome::NotExecuted, "past it, the chain at the expected epoch 1");

    let (other, other_org, _) = ready("write-unknown-record-lands", &chain, 0x45).await;
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    written_paused(&other, other_org).await.unwrap();
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let Verified::Settled { write, .. } = other.verify_organisation(HeldTarget::Organisation(other_org)).await else { panic!("not settled") };
    assert_eq!(write, org_io::WriteOutcome::Adopted, "the chain holds the recorded root and key at the epoch after the expected one");
}

// verifies: LLR-a66ufr
#[tokio::test]
async fn a_failed_window_read_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, next) = ready("write-window-read-fails", &chain, 0x39).await;
    chain.write_window_failing.store(true, Ordering::SeqCst);
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await, Ok(WriteReport::Refused { .. })));
    assert_eq!((chain.update_writes(), chain.slots().get(&org).unwrap().epoch), (0, Epoch::new(1)), "nothing written");
    assert!(still_held("write-window-read-fails", &alice, org, &next).await);
    chain.write_window_failing.store(false, Ordering::SeqCst);
    let again = alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await;
    assert!(matches!(again, Ok(WriteReport::Included { .. })), "no unknown write: the next write is made: {again:?}");
}

// Only an elapsed bound records an unknown write (LLR-d3farr), and nothing but verification clears
// one (LLR-7y46be): a write the chain refused leaves none, so the next write is made.
// verifies: LLR-d3farr, LLR-7y46be
#[tokio::test]
async fn a_refused_write_records_no_unknown_write() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, _) = ready("write-refused-no-unknown", &chain, 0x3a).await;
    chain.write_failing.store(true, Ordering::SeqCst);
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await, Ok(WriteReport::Refused { .. })));
    chain.write_failing.store(false, Ordering::SeqCst);
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap(), WriteReport::Included { .. }), "the next write is not refused as unknown");
}

// verifies: LLR-mhn4n5
#[tokio::test(flavor = "current_thread")]
async fn an_unknown_write_refuses_a_second_write_for_that_target_only() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, _) = ready("write-unknown-second", &chain, 0x3b).await;
    chain.write_hanging.store(true, Ordering::SeqCst);
    written_paused(&alice, org).await.unwrap();
    chain.write_hanging.store(false, Ordering::SeqCst);
    let writes = chain.update_writes();
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await, Err(WriteRefused::OutcomeUnknown)));
    assert_eq!(chain.update_writes(), writes, "the refusal made no chain call");
    let (other, other_org, _) = ready("write-unknown-second-other", &chain, 0x3c).await;
    assert!(matches!(other.write_organisation_to_chain(HeldTarget::Organisation(other_org)).await.unwrap(), WriteReport::Included { .. }));
}

// verifies: LLR-s7rq5x
#[tokio::test]
async fn a_refused_update_write_keeps_the_provisional_organisation() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, next) = ready("write-refused-kept", &chain, 0x3d).await;
    chain.write_failing.store(true, Ordering::SeqCst);
    let WriteReport::Refused { failure } = alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap() else {
        panic!("not refused")
    };
    let reason = failure.to_string();
    assert!(reason.contains("node unreachable"), "{reason}");
    assert!(still_held("write-refused-kept", &alice, org, &next).await, "the provisional organisation stays held");
}

// verifies: LLR-d3farr
#[tokio::test(flavor = "current_thread")]
async fn a_write_that_never_finishes_times_out_as_unknown() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, _) = ready("write-never-finishes", &chain, 0x3e).await;
    chain.write_hanging.store(true, Ordering::SeqCst);
    let WriteReport::Unknown { reason } = written_paused(&alice, org).await.unwrap() else { panic!("not unknown") };
    assert!(reason.contains("timed out after 90 s"), "{reason}");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(1), "nothing landed");
}

// verifies: LLR-s7rq5x
#[tokio::test]
async fn a_handle_with_no_chain_refuses_every_write() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    // The Organisation and its next epoch are reached through a handle over the chain; the same data directory is then opened with no chain configured.
    let tag = "write-no-chain";
    let alice = handle(tag, "a", &chain);
    let (org, _) = founded(&alice, "alice").await;
    proposed(&alice, org, &handle(tag, "b", &chain)).await;
    drop(alice);
    let io = OrgIo::open(&common::handles::store_dir(tag, "a"), TransportMode::Loopback).unwrap();
    let WriteReport::Refused { failure } = io.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap() else {
        panic!("not refused")
    };
    let reason = failure.to_string();
    assert!(reason.contains("chain not configured"), "{reason}");
}

// verifies: LLR-48me7m
#[tokio::test]
async fn a_stale_provisional_organisation_is_refused_before_any_write() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, _) = ready("write-stale", &chain, 0x40).await;
    let member = alice.organisation(org).await.unwrap().members[0].id;
    alice.propose_revocation(org, member).await.unwrap();
    let files = store_bytes("write-stale", "a");
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(org)).await, Err(WriteRefused::Stale)));
    assert_eq!((chain.update_writes(), chain.write_windows()), (0, 0), "before any chain call");
    assert_eq!(store_bytes("write-stale", "a"), files, "nothing written to the store");
}

// verifies: LLR-s7rq5x
#[tokio::test]
async fn a_write_for_an_organisation_no_persona_holds_is_refused_as_not_held() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, _org, _) = ready("write-not-held", &chain, 0x41).await;
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Organisation(OrgId::new([0x55; 20]))).await, Err(WriteRefused::NotHeld)));
    assert_eq!((chain.update_writes(), chain.write_windows()), (0, 0));
}

/// A provisional founding proposed through `a`.
async fn founding(alice: &OrgIo) -> org_io::ProposedFounding {
    alice.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap()
}

/// The proxy and Organisation the founding store of `founder` holds, read from the store file A's
/// handle wrote in test `tag`; the Organisation the handle lists for the Persona is that file's
/// (no read of org-node's store through the handle).
async fn stored(tag: &str, alice: &OrgIo, founder: &org_node::DevicePublicKey) -> (Option<ChainAccount>, Option<OrgId>) {
    let stored = reopened_service(tag, "a", alice).store_data(founder).unwrap().clone();
    let held = alice.personas().await.into_iter().find(|persona| &persona.device_key == founder).expect("the Persona is held").org_id;
    assert_eq!(held, stored.persona.org_id, "the handle holds the Organisation its file holds");
    (stored.founding.as_ref().and_then(|founding| founding.proxy), stored.persona.org_id)
}

// verifies: LLR-5tpdxw
#[tokio::test]
async fn a_founding_creates_the_proxy_records_it_and_only_then_founds_at_it() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("found-two-steps", "a", &chain);
    let proposed = founding(&alice).await;
    assert_eq!(
        alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await.unwrap(),
        WriteReport::Included { epoch: proposed.epoch },
        "included, naming the founding's epoch"
    );
    let (proxy, org_id) = stored("found-two-steps", &alice, &proposed.founder).await;
    let (proxy, org_id) = (proxy.expect("the proxy recorded"), org_id.expect("the Organisation recorded"));
    assert_eq!(chain.founding_calls(), vec![("create_proxy", None), ("found_at", Some(proxy))], "create, then found at the stored proxy");
    let slot = chain.slots().get(&org_id).unwrap();
    assert_eq!((slot.epoch, slot.root_hash, slot.org_pub_key), (Epoch::new(1), proposed.root, proposed.org_public_key));
    let writes = org_io::test_support::store_writes_to(&common::handles::store_dir("found-two-steps", "a"), &proposed.founder);
    assert_eq!(writes.len(), 2, "the proposal's write and the proxy's, before found_at");
}

// Abnormal: the store write after `create_proxy` fails: the write stops there, names the proxy,
// and the Persona is reloaded.
// verifies: LLR-5tpdxw
#[tokio::test]
async fn a_failed_store_write_after_create_proxy_stops_and_names_the_proxy() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("found-store-fails", "a", &chain);
    let proposed = founding(&alice).await;
    // The proxy a fresh fake's first `create_proxy` gives: the one this write's call created.
    let (_, created) = org_io::write::ChainWriter::create_proxy(&FakeChain::new()).await.unwrap();
    org_io::test_support::fail_next_write_after_temporary();
    match alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await {
        Err(WriteRefused::Store { proxy, .. }) => {
            assert_eq!(proxy, created, "names the proxy create_proxy returned");
            assert_eq!(chain.founding_calls(), vec![("create_proxy", None)], "stopped before found_at");
        }
        other => panic!("not refused naming the proxy: {other:?}"),
    }
    assert_eq!(stored("found-store-fails", &alice, &proposed.founder).await, (None, None), "reloaded from its file: no proxy held");
}

// verifies: LLR-5tpdxw
#[tokio::test]
async fn a_founding_with_a_stored_proxy_skips_create_proxy() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("found-stored-proxy", "a", &chain);
    let proposed = founding(&alice).await;
    chain.found_at_failing.store(true, Ordering::SeqCst);
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await, Ok(WriteReport::Refused { .. })));
    chain.found_at_failing.store(false, Ordering::SeqCst);
    let (proxy, _) = stored("found-stored-proxy", &alice, &proposed.founder).await;
    assert!(matches!(alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await.unwrap(), WriteReport::Included { .. }));
    assert_eq!(
        chain.founding_calls(),
        vec![("create_proxy", None), ("found_at", proxy), ("found_at", proxy)],
        "one proxy for the founding: the retry founds at the stored one"
    );
}

// verifies: LLR-sq5pbq
#[tokio::test]
async fn a_refused_found_at_step_is_refused_naming_it_and_keeps_the_proxy() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("found-step-refused", "a", &chain);
    let proposed = founding(&alice).await;
    chain.found_at_failing.store(true, Ordering::SeqCst);
    let WriteReport::Refused { failure } = alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await.unwrap() else {
        panic!("not refused")
    };
    let reason = failure.to_string();
    let (proxy, _) = stored("found-step-refused", &alice, &proposed.founder).await;
    let proxy = proxy.expect("the proxy is kept");
    assert!(reason.contains("Fund") && reason.contains(&hex::encode(proxy.as_bytes())), "names the step and the proxy: {reason}");
    chain.found_at_failing.store(false, Ordering::SeqCst);
    let again = alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await;
    assert!(matches!(again, Ok(WriteReport::Included { .. })), "no unknown write: the next write is made: {again:?}");
}

// verifies: LLR-sq5pbq
#[tokio::test]
async fn a_failed_create_proxy_stores_no_proxy() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("found-create-fails", "a", &chain);
    let proposed = founding(&alice).await;
    chain.write_failing.store(true, Ordering::SeqCst);
    let WriteReport::Refused { failure } = alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await.unwrap() else {
        panic!("not refused")
    };
    let reason = failure.to_string();
    assert!(reason.contains("creating the pure proxy"), "{reason}");
    assert_eq!(stored("found-create-fails", &alice, &proposed.founder).await, (None, None), "no proxy stored");
}

// Abnormal: `create_proxy` never returns, so the 90-second bound elapses before any proxy is
// known. No unknown write is recorded for the founding, which has no proxy a verification could
// settle it at, so the next write is not refused and creates the proxy again.
// verifies: LLR-sq5pbq, LLR-d3farr, PR-8ghk9s
#[tokio::test(flavor = "current_thread")]
async fn a_create_proxy_timeout_records_no_unknown_write_and_the_next_write_creates_again() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("found-create-times-out", "a", &chain);
    let proposed = founding(&alice).await;
    chain.write_hanging.store(true, Ordering::SeqCst);
    tokio::time::pause();
    let timed_out = alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await.unwrap();
    tokio::time::resume();
    chain.write_hanging.store(false, Ordering::SeqCst);
    let WriteReport::Refused { failure } = timed_out else { panic!("not refused: {timed_out:?}") };
    assert_eq!(failure, WriteFailure::CreateProxyUnanswered, "an unanswered proxy creation");
    let reason = failure.to_string();
    assert!(
        reason.contains("the pure proxy creation went unanswered") && reason.contains(&format!("{} s", WRITE_TIMEOUT.as_secs())),
        "{reason}"
    );
    assert_eq!(stored("found-create-times-out", &alice, &proposed.founder).await, (None, None), "no proxy stored");
    assert_eq!(
        alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await.unwrap(),
        WriteReport::Included { epoch: proposed.epoch },
        "the next write proceeds"
    );
    let (proxy, _) = stored("found-create-times-out", &alice, &proposed.founder).await;
    assert_eq!(
        chain.founding_calls(),
        vec![("create_proxy", None), ("create_proxy", None), ("found_at", Some(proxy.expect("the new proxy stored")))],
        "the next write creates the proxy again and founds at it"
    );
}

// Abnormal: a founding's `found_at` at its stored proxy never returns, so its outcome is
// unknown and recorded. While it is, another write of that founding is refused
// `OutcomeUnknown` and makes no chain call; a founding of another Persona still writes.
// verifies: LLR-mhn4n5
#[tokio::test(flavor = "current_thread")]
async fn a_foundings_unknown_outcome_refuses_a_second_write_of_that_founding_only() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("found-unknown-second", "a", &chain);
    let proposed = founding(&alice).await;
    let target = HeldTarget::Founding(proposed.founder);
    chain.found_at_failing.store(true, Ordering::SeqCst);
    assert!(matches!(alice.write_organisation_to_chain(target).await, Ok(WriteReport::Refused { .. })), "the proxy stored");
    chain.found_at_failing.store(false, Ordering::SeqCst);
    chain.write_hanging.store(true, Ordering::SeqCst);
    tokio::time::pause();
    let hung = alice.write_organisation_to_chain(target).await;
    tokio::time::resume();
    chain.write_hanging.store(false, Ordering::SeqCst);
    assert!(matches!(hung, Ok(WriteReport::Unknown { .. })), "{hung:?}");
    let calls = chain.founding_calls().len();

    assert!(matches!(alice.write_organisation_to_chain(target).await, Err(WriteRefused::OutcomeUnknown)));
    assert_eq!(chain.founding_calls().len(), calls, "the refusal made no chain call");
    let other = founding(&alice).await;
    assert!(
        matches!(alice.write_organisation_to_chain(HeldTarget::Founding(other.founder)).await, Ok(WriteReport::Included { .. })),
        "another founding writes"
    );
}

// verifies: LLR-5tpdxw
#[tokio::test]
async fn a_founding_write_creates_the_proxy_records_it_then_founds_at_it() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("found-write", "a", &chain);
    let proposed = founding(&alice).await;
    alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await.unwrap();
    let (_, org_id) = stored("found-write", &alice, &proposed.founder).await;
    assert!(chain.slots().get(&org_id.unwrap()).is_some(), "the chain holds the genesis");
    assert!(alice.organisations().await.is_empty(), "nothing adopted by the write: verification adopts");
}

// verifies: LLR-sq5pbq
#[tokio::test]
async fn a_refused_genesis_step_is_reported_refused_naming_the_step() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let alice = handle("found-genesis-recorded", "a", &chain);
    let proposed = founding(&alice).await;
    alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await.unwrap();
    let WriteReport::Refused { failure } = alice.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await.unwrap() else {
        panic!("a second recording is refused by the contract")
    };
    let reason = failure.to_string();
    assert!(reason.contains("RecordGenesis"), "{reason}");
}

// The contract test of `FakeChain`'s `ChainWriter` (Doubles 3):
// `common::chain_writer_contract`'s founding assertions, over the fake. The same assertions run
// against the production writer on a chopsticks fork in `tests/chain_writer_contract.rs`,
// outside the gates.
// verifies: LLR-5tpdxw, LLR-sq5pbq
#[tokio::test]
async fn fake_chain_meets_the_founding_contract() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    common::chain_writer_contract::founding_contract(&handle("found-contract", "a", &chain)).await;
}

// The contract test of `FakeChain`'s state and finalised-block reads, its write window and
// update (Doubles 3):
// `common::chain_contract`'s assertions, over the fake. The same assertions run against the
// production halves on a chopsticks fork in `tests/chain_writer_contract.rs`, outside the gates.
// The fake's `state_hidden` and `parse_refusing` switches answer as the real reader does for no state and
// for a key org-node's parse refuses, and `epoch_ahead` as the chain does once another update at
// the same root and key landed.
// verifies: LLR-js7wgk, LLR-j4nkx5
#[tokio::test]
async fn fake_chain_meets_the_reader_and_writer_contract() {
    use common::chain_contract::{reader_and_writer_contract, ChainUnderTest};
    use org_io::chain_read::{org_state_from_chain, StateReader};

    let chain = FakeChain::new();
    let under = ChainUnderTest { reader: &chain, writer: &chain };
    let (org, proxy) = reader_and_writer_contract(&under, || {
        chain.advance_finalised(1);
        async {}
    })
    .await;

    chain.state_hidden.store(true, Ordering::SeqCst);
    assert_eq!(chain.read_state(org).await.unwrap(), None, "state_hidden reads as no state, clause 2");
    chain.state_hidden.store(false, Ordering::SeqCst);
    chain.parse_refusing.store(true, Ordering::SeqCst);
    let refused = chain.read_state(org).await.unwrap_err();
    let state = on_chain_client::OrgState {
        root_hash: on_chain_client::OnChainRootHash([0x31; 32]),
        org_pub_key: on_chain_client::OrgPubKey([0; 32]),
        epoch: on_chain_client::Epoch(1),
    };
    assert_eq!(refused, org_state_from_chain(state).unwrap_err(), "parse_refusing reads as the real reader's parse refuses");
    chain.parse_refusing.store(false, Ordering::SeqCst);

    chain.epoch_ahead.store(true, Ordering::SeqCst);
    let ahead = chain.read_state(org).await.unwrap().expect("a state one epoch on");
    chain.epoch_ahead.store(false, Ordering::SeqCst);
    let current = chain.read_state(org).await.unwrap().expect("the state");
    let window = org_io::write::ChainWriter::write_window(&chain).await.unwrap();
    org_io::write::ChainWriter::update(&chain, proxy, current.root_hash, current.org_pub_key, current.epoch, window).await.unwrap();
    assert_eq!(
        chain.read_state(org).await.unwrap(),
        Some(ahead),
        "epoch_ahead reads as the chain does once an update at its root and key landed, clause 4"
    );
}

// Each refusal before any chain call reads as text the app can show the user: its `Display` is
// text of its own, not its `Debug` form and not another refusal's; org-node's refusal and the
// store failure are shown by their own text, and the proxy a stopped founding created is named in
// hex (the one text an LLR states, LLR-5tpdxw's, pinned exactly; the others' wording is free).
// verifies: LLR-5tpdxw
#[test]
fn each_write_refusal_displays_human_readable_text() {
    use org_io::store_file::{StoreFileError, StoreFileStep};
    use org_io::StoreFailure;
    use org_node::error::OrgNodeError;

    let proxy = ChainAccount::new([0x5a; 32]);
    let failed = StoreFileError { step: StoreFileStep::Write, path: "/data/persona-ab.store".into(), reason: "disk full".into() };
    let failure = StoreFailure::Reloaded { failed };
    let failure_text = failure.to_string();
    let store = WriteRefused::Store { proxy, failure };
    assert_eq!(store.to_string(), format!("the proxy account {} was created, but recording it failed: {failure_text}", "5a".repeat(32)));
    let node = WriteRefused::Node(OrgNodeError::NoProvisionalFounding);
    assert!(node.to_string().contains(&OrgNodeError::NoProvisionalFounding.to_string()), "org-node's own text: {node}");
    let refusals = [WriteRefused::NotHeld, WriteRefused::NoProxyAccount, WriteRefused::Stale, WriteRefused::OutcomeUnknown, node, store];
    let texts: Vec<String> = refusals.iter().map(ToString::to_string).collect();
    for (refused, text) in refusals.iter().zip(&texts) {
        assert!(!text.trim().is_empty(), "{refused:?} has text");
        assert_ne!(*text, format!("{refused:?}"), "the Display text is not the Debug form");
        assert_eq!(texts.iter().filter(|other| *other == text).count(), 1, "each refusal says its own: {text}");
    }
}

// A refusal that has a cause keeps it as its `source()`, typed, besides
// naming it in its text: org-node's refusal and the store failure; the
// others have none.
// verifies: LLR-48me7m, LLR-5tpdxw
#[test]
fn a_write_refusal_keeps_its_cause_as_its_source() {
    use std::error::Error;

    use org_io::store_file::{StoreFileError, StoreFileStep};
    use org_io::StoreFailure;
    use org_node::error::OrgNodeError;

    let node = WriteRefused::Node(OrgNodeError::NoProvisionalFounding);
    let cause = node.source().and_then(|source| source.downcast_ref::<OrgNodeError>());
    assert_eq!(cause, Some(&OrgNodeError::NoProvisionalFounding));
    let failed = StoreFileError { step: StoreFileStep::Write, path: "/data/persona-ab.store".into(), reason: "disk full".into() };
    let store = WriteRefused::Store { proxy: ChainAccount::new([0x5a; 32]), failure: StoreFailure::Reloaded { failed } };
    let cause = store.source().and_then(|source| source.downcast_ref::<StoreFailure>());
    assert!(matches!(cause, Some(StoreFailure::Reloaded { .. })), "{cause:?}");
    for none in [WriteRefused::NotHeld, WriteRefused::NoProxyAccount, WriteRefused::Stale, WriteRefused::OutcomeUnknown] {
        assert!(none.source().is_none(), "{none:?}");
    }
}
