#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The publish state (SDD-6mpmsw): a chain write whose outcome is unknown is
//! recorded (LLR-2yjuaj), settled by reading the chain (LLR-t98pk9) before
//! any other write for its Organisation (LLR-nqvbb6), and provisional
//! updates are discarded in the background: by each commit (LLR-fptp57,
//! LLR-36zp69) and by a settle that finds a write not executed (LLR-t98pk9).
//! The chain is `test_support::FakeChain`. A write that times out is driven
//! on a paused clock (`common::handles::timed_out`).
//!
//! *Reconciled with S3b-io part 1, 2026-10-14 (merge into S4,
//! `worktree-org-io-transport`):* every test keeps its name, its scenario
//! and its `verifies:` line; its call sites are S4's. The handle takes
//! `&self` and its builders and views are its own `async` methods (S4, task
//! T11: no `node_mut`, no `view`); a submission names no recipient and takes
//! org-io's `PeerAddr` (S4, task T12), and a committed update whose send
//! fails is `Ok(Submitted)` with the failure in its per-Device `sends`, not
//! an `Err` (S4, task T12, LLR-znhmh2); a Loopback sink is bound as the
//! recipient's Device from its keychain item (S4, task T10); the receive
//! step is `next_received` (S4, task T11); `reopened_service` takes the
//! handle, whose keychain items unseal the store files (S4, task T9); a
//! store write that fails is injected with
//! `test_support::fail_next_write_after_temporary` (S4, task T9), since an
//! atomic write replaces a read-only file.

use std::sync::atomic::Ordering;
use std::time::Duration;

use on_chain_client::write::WRITE_MORTALITY_BLOCKS;
use org_io::test_support::FakeChain;
use org_io::{BuiltUpdate, OrgIo, Received, Reconciliation, Settled, Submitted};
use org_node::wire::WireMessage;
use org_node::{Epoch, OrgId, OrgState, RootHash};
use rand::rngs::OsRng;

mod common;
use common::handles::{
    at, bind_seed, founded, handle, held, hung, joiner, joiner_of, persona, receiving_addr, reopened_service, store_bytes,
    timed_out, NET,
};

async fn epoch_of(io: &OrgIo, org: OrgId) -> Epoch {
    io.organisation(org).await.unwrap().epoch
}

/// A submission that was written and committed at `epoch` and whose
/// Loopback send, with no address, failed for every recipient. *Reconciled
/// with S3b-io part 1, 2026-10-14:* S3b-io asserted the error "committed at
/// epoch N, but the send failed"; since S4 task T12 the commit is `Ok` and
/// the send's failure is reported per Device.
fn committed_with_failed_sends(submitted: Result<Submitted, String>, epoch: u64) {
    let submitted = submitted.unwrap();
    assert_eq!(submitted.epoch, Epoch::new(epoch), "committed");
    assert!(
        !submitted.sends.per_device.is_empty() && submitted.sends.per_device.iter().all(|(_, sent)| sent.is_err()),
        "the Loopback send has no address and fails after: {:?}",
        submitted.sends
    );
}

// The reproduction of the app's provisional-housekeeping problem report
// (its ID joins the verifies line when the report moves into org-io's
// ledger, plan task T17a): a write that executes after its 90-second bound
// is committed when the next submission for its Organisation settles it, and
// sent to the Device that submission named (C's), not the new one's; the new
// update (D's) is not written. On S2's code D's write went out and C's update
// was never committed.
// *Reconciled with S3b-io part 1, 2026-10-14:* C's late commit is sent as
// its submission would have been (S4, task T12): to its recipients, here C's
// Device alone, at the address that submission was given; C's sink is bound
// as C's Device.
// verifies: LLR-2yjuaj, LLR-t98pk9, LLR-nqvbb6, PR-924ftr
#[tokio::test]
async fn a_write_that_lands_after_its_timeout_is_committed_and_sent_to_its_own_recipient() {
    let (chain, a, org) = founded("publish-late-write").await;
    let carol = joiner(0x31, "carol");
    let carol_endpoint = bind_seed([0x32; 32]).await;
    let carol_addr = carol_endpoint.inner().addr();
    let c = a.admit_member(&mut OsRng, org, &carol).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&a, &c, at(carol_addr)).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    assert_eq!(chain.slots().get(&org).unwrap().root_hash, c.resulting_root, "C's write landed");
    assert_eq!(epoch_of(&a, org).await, Epoch::new(1), "but nothing was committed");
    let writes = chain.update_writes();

    let dave = joiner(0x33, "dave");
    let d = a.admit_member(&mut OsRng, org, &dave).await.unwrap();
    let carol_receives = tokio::spawn(async move {
        let received = tokio::time::timeout(NET, carol_endpoint.recv_one()).await.unwrap().unwrap();
        (carol_endpoint, received)
    });
    let err = a.submit_commit_send(&mut OsRng, &d, None).await.unwrap_err();
    assert!(err.contains("committed at epoch 2") && err.contains("this update was not written"), "{err}");
    assert_eq!(chain.update_writes(), writes, "D's update was not written");
    let (_endpoint, (_sender, message)) = carol_receives.await.unwrap();
    let WireMessage::OrgInformation { envelope, .. } = message else { panic!("Organisation information for C") };
    assert_eq!(envelope.org_id, org);
    let record = a.organisation(org).await.unwrap();
    assert_eq!((record.epoch, record.root_hash), (Epoch::new(2), c.resulting_root), "C committed");
    assert!(!held(&a, &c), "C is no longer provisional");
}

// Review round 1, finding-1: the app's only way to ask (its reconcile
// button, `reconcile_held`) commits a write that landed after its timeout,
// sends it to the Device its submission named, and clears the unknown write:
// the next settle reads nothing.
// *Reconciled with S3b-io part 1, 2026-10-14:* as the test above, the sink
// is bound as C's Device and the send goes where C's submission would have.
// verifies: LLR-k5pzsz, LLR-t98pk9
#[tokio::test]
async fn a_reconcile_that_commits_an_unknown_write_sends_it_to_its_recipient_and_clears_it() {
    let (chain, a, org) = founded("publish-reconcile-unknown").await;
    let carol = joiner(0x37, "carol");
    let carol_endpoint = bind_seed([0x38; 32]).await;
    let carol_addr = carol_endpoint.inner().addr();
    let c = a.admit_member(&mut OsRng, org, &carol).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&a, &c, at(carol_addr)).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let carol_receives = tokio::spawn(async move {
        let received = tokio::time::timeout(NET, carol_endpoint.recv_one()).await.unwrap().unwrap();
        (carol_endpoint, received)
    });
    assert_eq!(
        a.reconcile_held(&mut OsRng).await,
        vec![Reconciliation::Committed { org_id: org, epoch: Epoch::new(2), recipients: vec![carol.device_key] }]
    );
    let (reads, finalised_reads) = (chain.reads(), chain.finalised_reads());
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");
    assert_eq!((chain.reads(), chain.finalised_reads()), (reads, finalised_reads), "nothing read");
    let (_endpoint, (_sender, message)) = carol_receives.await.unwrap();
    let WireMessage::OrgInformation { envelope, .. } = message else { panic!("Organisation information for C") };
    assert_eq!(envelope.org_id, org);
}

// Review round 1, finding-1: after a reconcile committed a late write, the
// next submission for the Organisation is written, not refused by a settle
// of the write the reconcile already committed.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites; the commit
// is `Ok` with its failed send in `sends` (`committed_with_failed_sends`).
// verifies: LLR-k5pzsz, LLR-nqvbb6
#[tokio::test]
async fn after_a_reconcile_commits_an_unknown_write_the_next_submission_is_written() {
    let (chain, a, org) = founded("publish-reconcile-then-submit").await;
    let bob = joiner(0x39, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&a, &c, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let result = a.reconcile(&mut OsRng, org).await;
    assert!(matches!(result, Reconciliation::Committed { epoch, .. } if epoch == Epoch::new(2)), "{result:?}");
    let dave = joiner(0x3b, "dave");
    let d = a.admit_member(&mut OsRng, org, &dave).await.unwrap();
    let writes = chain.update_writes();
    // Written and committed; the Loopback send has no address and fails after.
    committed_with_failed_sends(a.submit_commit_send(&mut OsRng, &d, None).await, 3);
    assert_eq!(chain.update_writes(), writes + 1, "D was written");
}

// Review round 1, finding-1: a settle that finds its write executed and its
// update already committed (here by org-node's reconcile, driven out of
// org-io's sight) counts as settled: the submission that ran it writes.
// *Reconciled with S3b-io part 1, 2026-10-14:* org-node's reconcile takes
// the keychain's secrets (none needed here) and returns its store change,
// left unwritten: out of org-io's sight; D's commit writes the store after.
// verifies: LLR-t98pk9, LLR-nqvbb6
#[tokio::test]
async fn a_settle_that_finds_its_write_already_committed_lets_the_submission_write() {
    let (chain, a, org) = founded("publish-settle-in-step").await;
    let bob = joiner(0x3d, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&a, &c, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let landed = chain.slots().get(&org).unwrap();
    let (_, _unwritten) = a.node_for_test().reconcile(&mut OsRng, org, Some(landed), |_| Vec::new()).unwrap();
    assert_eq!(epoch_of(&a, org).await, Epoch::new(2), "C committed");
    let dave = joiner(0x3f, "dave");
    let d = a.admit_member(&mut OsRng, org, &dave).await.unwrap();
    let writes = chain.update_writes();
    committed_with_failed_sends(a.submit_commit_send(&mut OsRng, &d, None).await, 3);
    assert_eq!(chain.update_writes(), writes + 1, "D was written");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");
}

// Review round 1, finding-3: once the record has moved past the epoch the
// write expected (here a reconcile committed another signatory's update),
// the write can never pass the contract's epoch check: the settle finds it
// not executed at once, within its window, reading nothing.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-t98pk9
#[tokio::test]
async fn an_unknown_write_whose_epoch_the_record_has_passed_is_settled_at_once() {
    let (chain, a, org) = founded("publish-epoch-passed").await;
    let bob = joiner(0x67, "bob");
    let carol = joiner(0x69, "carol");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let d = a.admit_member(&mut OsRng, org, &carol).await.unwrap();
    hung(&chain, &a, &c).await;
    chain.slots().apply_update(org, d.resulting_root, d.org_pub_key, Epoch::new(1)).unwrap();
    assert_eq!(
        a.reconcile(&mut OsRng, org).await,
        Reconciliation::Committed { org_id: org, epoch: Epoch::new(2), recipients: vec![carol.device_key] }
    );
    let (reads, finalised_reads) = (chain.reads(), chain.finalised_reads());
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NotExecuted { org_id: org });
    assert_eq!((chain.reads(), chain.finalised_reads()), (reads, finalised_reads), "nothing read");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");
}

// PR-9q5ems (S4 review round 2, A3): the record past the epoch the write
// expected is no evidence the write did not execute when the record holds
// that write's own update — here committed out of org-io's sight by
// org-node's reconcile, as a commit the settle did not drive. The settle
// reads the chain, finds the update there, and reports the commit already
// made; a submission of another update then writes.
// verifies: LLR-t98pk9, PR-9q5ems
#[tokio::test]
async fn a_settle_whose_record_holds_its_executed_update_does_not_report_it_not_executed() {
    let (chain, a, org) = founded("publish-record-holds-executed").await;
    let bob = joiner(0x71, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&a, &c, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let landed = chain.slots().get(&org).unwrap();
    let (_, _unwritten) = a.node_for_test().reconcile(&mut OsRng, org, Some(landed), |_| Vec::new()).unwrap();
    assert_eq!(epoch_of(&a, org).await, Epoch::new(2), "C committed out of the settle's sight");
    assert_eq!(
        a.settle_unknown(&mut OsRng, org).await,
        Settled::Reconciled(Reconciliation::InStep { org_id: org, epoch: Epoch::new(2) })
    );
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");
}

// PR-9q5ems (S4 review round 2, A3): this node's write lands at e+1, another
// update at e+2 is committed on top of it before the settle runs (here by
// org-node's reconcile out of org-io's sight, as a received update would be
// committed). The record past e+1 says nothing about which update took e+1:
// the settle must not report the executed write as not executed, nor tell a
// resubmission to rebuild; it reads the chain and reconciles.
// verifies: LLR-t98pk9, LLR-nqvbb6, PR-9q5ems
#[tokio::test]
async fn a_settle_after_a_later_epoch_was_committed_does_not_report_its_executed_write_not_executed() {
    let (chain, a, org) = founded("publish-later-epoch-received").await;
    let bob = joiner(0x73, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&a, &c, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let landed = chain.slots().get(&org).unwrap();
    let (_, _unwritten) = a.node_for_test().reconcile(&mut OsRng, org, Some(landed), |_| Vec::new()).unwrap();
    let carol = joiner(0x75, "carol");
    let d = a.admit_member(&mut OsRng, org, &carol).await.unwrap();
    chain.slots().apply_update(org, d.resulting_root, d.org_pub_key, Epoch::new(2)).unwrap();
    let later = chain.slots().get(&org).unwrap();
    let (_, _unwritten) = a.node_for_test().reconcile(&mut OsRng, org, Some(later), |_| Vec::new()).unwrap();
    assert_eq!(epoch_of(&a, org).await, Epoch::new(3), "e+2 committed on top of C");

    let writes = chain.update_writes();
    let err = a.submit_commit_send(&mut OsRng, &c, None).await.unwrap_err();
    assert!(!err.contains("did not execute") && !err.contains("rebuild"), "C executed: {err}");
    assert_eq!(chain.update_writes(), writes, "nothing written");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "settled");
}

// Review round 1, finding-5: resubmitting the very update whose write the
// settle finds not executed (and discards) is refused with a report saying
// so and that the update must be rebuilt; nothing is written.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-nqvbb6
#[tokio::test]
async fn resubmitting_an_update_whose_write_did_not_execute_says_it_must_be_rebuilt() {
    let (chain, a, org) = founded("publish-resubmit-not-executed").await;
    let bob = joiner(0x6b, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    hung(&chain, &a, &c).await;
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    let writes = chain.update_writes();
    let err = a.submit_commit_send(&mut OsRng, &c, None).await.unwrap_err();
    assert!(err.contains("did not execute") && err.contains("rebuild"), "{err}");
    assert_eq!(chain.update_writes(), writes, "nothing written");
    assert!(!held(&a, &c), "discarded");
}

// Review round 2, finding-2: resubmitting the update whose write the settle
// finds not executed because another update committed first (the record is
// past the epoch the write expected; its window has not passed) is refused
// with a report that is true here too: it says the write did not execute and
// the update must be rebuilt, and claims no passed window.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-nqvbb6
#[tokio::test]
async fn resubmitting_an_update_another_commit_overtook_says_it_must_be_rebuilt() {
    let (chain, a, org) = founded("publish-resubmit-overtaken").await;
    let bob = joiner(0x6d, "bob");
    let carol = joiner(0x6f, "carol");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let d = a.admit_member(&mut OsRng, org, &carol).await.unwrap();
    hung(&chain, &a, &c).await;
    chain.slots().apply_update(org, d.resulting_root, d.org_pub_key, Epoch::new(1)).unwrap();
    let result = a.reconcile(&mut OsRng, org).await;
    assert!(matches!(result, Reconciliation::Committed { epoch, .. } if epoch == Epoch::new(2)), "{result:?}");
    let writes = chain.update_writes();
    let err = a.submit_commit_send(&mut OsRng, &c, None).await.unwrap_err();
    assert!(err.contains("did not execute") && err.contains("rebuild"), "{err}");
    assert!(!err.contains("window"), "no window has passed: {err}");
    assert_eq!(chain.update_writes(), writes, "nothing written");
}

// The window is read once, before the write; a failed read writes nothing,
// records no unknown write and keeps the update.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-2yjuaj
#[tokio::test]
async fn a_failed_window_read_writes_nothing() {
    let (chain, a, org) = founded("publish-window-fails").await;
    let bob = joiner(0x41, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.write_window_failing.store(true, Ordering::SeqCst);
    let err = a.submit_commit_send(&mut OsRng, &update, None).await.unwrap_err();
    assert!(err.contains("no write was made") && err.contains("node unreachable"), "{err}");
    assert_eq!(chain.update_writes(), 0, "nothing written");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(1));
    assert!(held(&a, &update), "the update is kept");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "no unknown write recorded");
}

// A write that failed (not timed out) records no unknown write: the next
// settle reads nothing.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-2yjuaj
#[tokio::test]
async fn a_failed_write_records_no_unknown_write() {
    let (chain, a, org) = founded("publish-write-fails").await;
    let bob = joiner(0x43, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.write_failing.store(true, Ordering::SeqCst);
    let err = a.submit_commit_send(&mut OsRng, &update, None).await.unwrap_err();
    assert!(err.contains("chain write failed"), "{err}");
    let (reads, finalised_reads) = (chain.reads(), chain.finalised_reads());
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite);
    assert_eq!((chain.reads(), chain.finalised_reads()), (reads, finalised_reads), "nothing read");
}

// An unknown write refuses every later write for its Organisation while the
// outcome stays unknown, and no write for another Organisation.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites; the other
// Organisation's commit is `Ok` with its failed send in `sends`.
// verifies: LLR-nqvbb6
#[tokio::test]
async fn an_unknown_write_blocks_a_second_write_for_that_organisation_only() {
    let (chain, a, first) = founded("publish-blocks-one").await;
    let bea = persona(&a, "bea").await;
    let second = a.found_organisation(&mut OsRng, &bea).await.unwrap();
    let bob = joiner(0x45, "bob");
    let c = a.admit_member(&mut OsRng, first, &bob).await.unwrap();
    hung(&chain, &a, &c).await;
    let writes = chain.update_writes();

    let carol = joiner(0x47, "carol");
    let d = a.admit_member(&mut OsRng, first, &carol).await.unwrap();
    let err = a.submit_commit_send(&mut OsRng, &d, None).await.unwrap_err();
    assert!(err.contains("unknown outcome") && err.contains("this update was not written"), "{err}");
    assert_eq!(chain.update_writes(), writes, "no write for the Organisation with an unknown write");

    let e = a.admit_member(&mut OsRng, second, &carol).await.unwrap();
    // Written and committed; the Loopback send has no address and fails after.
    committed_with_failed_sends(a.submit_commit_send(&mut OsRng, &e, None).await, 2);
    assert_eq!(chain.update_writes(), writes + 1, "the other Organisation's write went out");
    assert_eq!(epoch_of(&a, second).await, Epoch::new(2));
    assert_eq!(epoch_of(&a, first).await, Epoch::new(1));
}

// The chain moved past the record, but not to the unknown write's root: the
// result is the reconcile's for that state — another held update committed,
// or behind — and the unknown write is cleared.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-t98pk9
#[tokio::test]
async fn an_unknown_write_whose_chain_moved_elsewhere_is_reconciled() {
    let (chain, a, org) = founded("publish-moved-elsewhere").await;
    let bob = joiner(0x51, "bob");
    let carol = joiner(0x53, "carol");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let d = a.admit_member(&mut OsRng, org, &carol).await.unwrap();
    hung(&chain, &a, &c).await;
    // Another signatory wrote D, which this node also holds.
    chain.slots().apply_update(org, d.resulting_root, d.org_pub_key, Epoch::new(1)).unwrap();
    assert_eq!(
        a.settle_unknown(&mut OsRng, org).await,
        Settled::Reconciled(Reconciliation::Committed { org_id: org, epoch: Epoch::new(2), recipients: vec![carol.device_key] })
    );
    assert_eq!(a.organisation(org).await.unwrap().root_hash, d.resulting_root);
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");

    // Then to a root this node does not hold: behind.
    let dave = joiner(0x55, "dave");
    let e = a.admit_member(&mut OsRng, org, &dave).await.unwrap();
    hung(&chain, &a, &e).await;
    let current = chain.slots().get(&org).unwrap();
    chain.slots().set(org, OrgState { root_hash: RootHash::new([0xab; 32]), epoch: Epoch::new(3), ..current });
    assert_eq!(
        a.settle_unknown(&mut OsRng, org).await,
        Settled::Reconciled(Reconciliation::Behind { org_id: org, chain_epoch: Epoch::new(3) })
    );
    assert_eq!(epoch_of(&a, org).await, Epoch::new(2), "nothing committed");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");
}

// A failed read — of the finalised block or of the state — or a state the
// parse refuses leaves the outcome unknown, the record and the store as they
// were; once the chain answers, the write is settled.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites; the store
// is every Persona's store file (`store_bytes`).
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_failed_read_leaves_the_outcome_unknown_and_changes_nothing() {
    let tag = "publish-read-fails";
    let (chain, a, org) = founded(tag).await;
    let bob = joiner(0x57, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&a, &c, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let before = store_bytes(tag, "a");
    for switch in [&chain.read_failing, &chain.parse_refusing] {
        switch.store(true, Ordering::SeqCst);
        let settled = a.settle_unknown(&mut OsRng, org).await;
        assert!(matches!(&settled, Settled::StillUnknown { org_id, .. } if *org_id == org), "{settled:?}");
        assert_eq!(epoch_of(&a, org).await, Epoch::new(1), "nothing committed");
        assert!(held(&a, &c), "the update is kept");
        assert_eq!(store_bytes(tag, "a"), before, "nothing written");
        switch.store(false, Ordering::SeqCst);
    }
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(matches!(settled, Settled::Committed { epoch, .. } if epoch == Epoch::new(2)), "{settled:?}");
}

// At the record's epoch, up to and including the window's last valid block,
// the write may still be included: the outcome stays unknown.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-t98pk9
#[tokio::test]
async fn within_the_window_the_outcome_stays_unknown() {
    let (chain, a, org) = founded("publish-within-window").await;
    let bob = joiner(0x59, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    hung(&chain, &a, &c).await;
    for advance in [0, WRITE_MORTALITY_BLOCKS] {
        chain.advance_finalised(advance);
        let settled = a.settle_unknown(&mut OsRng, org).await;
        assert!(matches!(&settled, Settled::StillUnknown { reason, .. } if reason.contains("may yet")), "{settled:?}");
    }
    assert!(held(&a, &c));
}

// Past the window's last valid block with the chain still at the record's
// epoch, the write can no longer execute: not executed, cleared, and another
// update may be written (a submission whose settle is `NotExecuted` writes).
// *Amended 2026-10-13 (owner ruling of that day, background discard):* the
// second half resubmitted the same update, which the settle now discards; it
// submits another.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites; the commit
// is `Ok` with its failed send in `sends`.
// verifies: LLR-t98pk9
#[tokio::test]
async fn past_the_window_at_the_records_epoch_the_write_is_not_executed() {
    let (chain, a, org) = founded("publish-past-window").await;
    let bob = joiner(0x5b, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    hung(&chain, &a, &c).await;
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NotExecuted { org_id: org });
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");

    // Through the submission: the second unknown write settles as not
    // executed, then the next update is written.
    let carol = joiner(0x5d, "carol");
    let d = a.admit_member(&mut OsRng, org, &carol).await.unwrap();
    let e = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    hung(&chain, &a, &d).await;
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    let writes = chain.update_writes();
    committed_with_failed_sends(a.submit_commit_send(&mut OsRng, &e, None).await, 2);
    assert_eq!(chain.update_writes(), writes + 1);
    assert_eq!(epoch_of(&a, org).await, Epoch::new(2));
}

// No unknown write, or an Organisation not held: nothing is read.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-t98pk9
#[tokio::test]
async fn settle_with_no_unknown_write_reads_nothing() {
    let (chain, a, org) = founded("publish-settle-none").await;
    let (reads, finalised_reads) = (chain.reads(), chain.finalised_reads());
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite);
    assert_eq!(a.settle_unknown(&mut OsRng, OrgId::new([9; 20])).await, Settled::NoUnknownWrite);
    assert_eq!((chain.reads(), chain.finalised_reads()), (reads, finalised_reads));
}

// A read that finds no state, or a state older than the record's, says
// nothing about the write: the outcome stays unknown, the unknown write is
// kept (the next settle reads again) and nothing changes.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_settle_that_finds_no_state_or_an_older_state_stays_unknown() {
    let tag = "publish-settle-no-state";
    let (chain, a, org) = founded(tag).await;
    let bob = joiner(0x5f, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    hung(&chain, &a, &c).await;
    // Past the window, so only the state read keeps the outcome unknown.
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    let before = store_bytes(tag, "a");
    chain.state_hidden.store(true, Ordering::SeqCst);
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(matches!(&settled, Settled::StillUnknown { reason, .. } if reason.contains("no on-chain state")), "{settled:?}");
    chain.state_hidden.store(false, Ordering::SeqCst);
    let current = chain.slots().get(&org).unwrap();
    chain.slots().set(org, OrgState { epoch: Epoch::new(0), ..current });
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(matches!(&settled, Settled::StillUnknown { reason, .. } if reason.contains("older than")), "{settled:?}");
    assert_eq!(store_bytes(tag, "a"), before, "nothing written");
    assert!(held(&a, &c));
    assert_eq!(epoch_of(&a, org).await, Epoch::new(1));
    chain.slots().set(org, current);
    let finalised_reads = chain.finalised_reads();
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NotExecuted { org_id: org }, "the write was kept");
    assert_eq!(chain.finalised_reads(), finalised_reads + 1);
}

// The record is no longer held when the settle finds the chain at another
// root (this node was removed meanwhile): org-node's reconcile refuses it as
// not held, reported as a refused reconciliation, and the unknown write is
// cleared.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites; the first
// commit is `Ok` with its failed send in `sends`; the removal's reconcile
// removes the Persona's store file (S4, task T9).
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_settle_whose_record_is_no_longer_held_is_refused() {
    let (chain, a, org) = founded("publish-settle-not-held").await;
    let bob = joiner(0x71, "bob");
    let admission = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    committed_with_failed_sends(a.submit_commit_send(&mut OsRng, &admission, None).await, 2);
    let carol = joiner(0x73, "carol");
    let c = a.admit_member(&mut OsRng, org, &carol).await.unwrap();
    let own_member = a.personas().await[0].member_id.unwrap();
    let removal = a.revoke_member(&mut OsRng, org, own_member).await.unwrap();
    hung(&chain, &a, &c).await;
    // Another signatory wrote this node's removal, and this node reconciled
    // it: the record is forgotten.
    chain.slots().apply_update(org, removal.resulting_root, removal.org_pub_key, Epoch::new(2)).unwrap();
    assert_eq!(a.reconcile(&mut OsRng, org).await, Reconciliation::Removed { org_id: org });
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(
        matches!(&settled, Settled::Reconciled(Reconciliation::Refused { org_id, reason }) if *org_id == org && reason.contains("not held")),
        "{settled:?}"
    );
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");
}

// The late write is committed even though its send fails; the failure is
// reported and the commit stands, on disk too.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites;
// `reopened_service` unseals with the handle's keychain items.
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_committed_late_write_whose_send_fails_keeps_the_commit() {
    let tag = "publish-late-send-fails";
    let (chain, a, org) = founded(tag).await;
    let bob = joiner(0x5d, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    // No address: the Loopback send refuses.
    timed_out(&a, &c, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let settled = a.settle_unknown(&mut OsRng, org).await;
    let Settled::Committed { org_id, epoch, sent } = settled else { panic!("{settled:?}") };
    assert_eq!((org_id, epoch), (org, Epoch::new(2)));
    assert!(sent.unwrap_err().contains("Loopback send requires"));
    assert_eq!(epoch_of(&a, org).await, Epoch::new(2));
    assert_eq!(reopened_service(tag, "a", &a).list_orgs()[0].epoch, Epoch::new(2), "the commit is saved");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite);
}

// A settle that finds the write not executed discards its update itself,
// with the Organisation private key it holds: not held afterwards, and gone
// from the store reopened from disk; another update of the Organisation is
// kept.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_write_known_not_executed_is_discarded_with_its_key() {
    let tag = "publish-not-executed-discarded";
    let (chain, a, org) = founded(tag).await;
    let bob = joiner(0x61, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let d = a.admit_member(&mut OsRng, org, &joiner(0x63, "carol")).await.unwrap();
    hung(&chain, &a, &c).await;
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NotExecuted { org_id: org });
    assert!(!held(&a, &c), "the update is discarded");
    assert!(held(&a, &d), "the other update is kept");
    let on_disk = reopened_service(tag, "a", &a).provisional_updates(org);
    assert!(on_disk.iter().all(|update| update.resulting_root != c.resulting_root), "C and its key are gone from disk");
    assert_eq!(on_disk.len(), 1);
}

// A settle whose discard cannot save the store reports the outcome as still
// unknown, with the reason, and keeps the unknown write: the next settle,
// with the store writable again, finds it not executed.
// *Reconciled with S3b-io part 1, 2026-10-14:* the failed save is S4's
// injected store-file write failure (`fail_next_write_after_temporary`; a
// read-only file no longer fails an atomic write), after which the Persona
// is reloaded from its file, the update held again as on disk.
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_settle_that_cannot_save_its_discard_stays_unknown() {
    let tag = "publish-not-executed-unsaved";
    let (chain, a, org) = founded(tag).await;
    let bob = joiner(0x65, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    hung(&chain, &a, &c).await;
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    org_io::test_support::fail_next_write_after_temporary();
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(
        matches!(&settled, Settled::StillUnknown { org_id, reason } if *org_id == org && reason.contains("discard")),
        "{settled:?}"
    );
    assert!(held(&a, &c), "reloaded from its file, which still holds it");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NotExecuted { org_id: org }, "the write was kept");
    assert!(!held(&a, &c));
}

// Added 2026-10-14 (merge of S3b-io part 1 into S4): a settle that finds
// its write executed but whose commit cannot be stored reports the outcome
// still unknown and keeps the write — the Persona is reloaded from its file,
// so nothing is committed — and the next settle commits it.
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_settle_whose_commit_is_not_stored_stays_unknown_and_commits_next_time() {
    let (chain, a, org) = founded("publish-late-commit-unsaved").await;
    let bob = joiner(0x75, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&a, &c, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    org_io::test_support::fail_next_write_after_temporary();
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(
        matches!(&settled, Settled::StillUnknown { org_id, reason } if *org_id == org && reason.contains("not stored")),
        "{settled:?}"
    );
    assert_eq!(epoch_of(&a, org).await, Epoch::new(1), "reloaded from its file: nothing committed");
    assert!(held(&a, &c), "the update is held again");
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(matches!(settled, Settled::Committed { epoch, .. } if epoch == Epoch::new(2)), "{settled:?}");
}

// Added 2026-10-14 (merge of S3b-io part 1 into S4): org-node's reconcile
// returns its store change and saves nothing (S4), so a reconcile that
// commits an unknown write whose store write fails is undone by the reload
// and keeps the unknown write; the next reconcile commits the update again
// and sends it to its recipient. On S2's org-node the commit stayed in memory
// and the send was lost (org-node's reconcile-save problem report).
// verifies: LLR-k5pzsz
#[tokio::test]
async fn a_reconcile_whose_commit_is_not_stored_keeps_the_unknown_write_and_sends_next_time() {
    let (chain, a, org) = founded("publish-reconcile-unsaved").await;
    let carol = joiner(0x77, "carol");
    let carol_endpoint = bind_seed([0x78; 32]).await;
    let carol_addr = carol_endpoint.inner().addr();
    let c = a.admit_member(&mut OsRng, org, &carol).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&a, &c, at(carol_addr)).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    org_io::test_support::fail_next_write_after_temporary();
    let result = a.reconcile(&mut OsRng, org).await;
    assert!(
        matches!(&result, Reconciliation::Refused { org_id, reason } if *org_id == org && reason.contains("nothing committed")),
        "{result:?}"
    );
    assert_eq!(epoch_of(&a, org).await, Epoch::new(1), "reloaded from its file: nothing committed");
    let carol_receives = tokio::spawn(async move {
        let received = tokio::time::timeout(NET, carol_endpoint.recv_one()).await.unwrap().unwrap();
        (carol_endpoint, received)
    });
    assert_eq!(
        a.reconcile(&mut OsRng, org).await,
        Reconciliation::Committed { org_id: org, epoch: Epoch::new(2), recipients: vec![carol.device_key] }
    );
    let (_endpoint, (_sender, message)) = carol_receives.await.unwrap();
    let WireMessage::OrgInformation { envelope, .. } = message else { panic!("Organisation information for C") };
    assert_eq!(envelope.org_id, org);
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");
}

/// An admission held for another Organisation of `io`, founded by a new
/// Persona: a commit to the first Organisation must keep it.
async fn held_elsewhere(io: &OrgIo) -> BuiltUpdate {
    let bea = persona(io, "bea").await;
    let other = io.found_organisation(&mut OsRng, &bea).await.unwrap();
    io.admit_member(&mut OsRng, other, &joiner(0x7d, "erin")).await.unwrap()
}

// A submission's commit leaves no other provisional update for its
// Organisation (org-node's commit drops the orphans), on disk too; another
// Organisation's update is kept.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites; the commit
// is `Ok` with its failed send in `sends`.
// verifies: LLR-fptp57
#[tokio::test]
async fn a_submitted_commit_leaves_no_other_update_for_the_organisation() {
    let tag = "publish-orphans-submitted";
    let (_chain, a, org) = founded(tag).await;
    let elsewhere = held_elsewhere(&a).await;
    let bob = joiner(0x81, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let d = a.admit_member(&mut OsRng, org, &joiner(0x83, "carol")).await.unwrap();
    committed_with_failed_sends(a.submit_commit_send(&mut OsRng, &c, None).await, 2);
    assert!(!held(&a, &d), "the other update is dropped");
    assert!(held(&a, &elsewhere), "another Organisation's update is kept");
    assert!(reopened_service(tag, "a", &a).provisional_updates(org).is_empty(), "on disk too");
}

// A settled late write's commit leaves no other provisional update for its
// Organisation; another Organisation's update is kept.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-fptp57
#[tokio::test]
async fn a_settled_late_write_leaves_no_other_update_for_the_organisation() {
    let (chain, a, org) = founded("publish-orphans-settled").await;
    let elsewhere = held_elsewhere(&a).await;
    let bob = joiner(0x85, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let d = a.admit_member(&mut OsRng, org, &joiner(0x87, "carol")).await.unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&a, &c, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(matches!(settled, Settled::Committed { epoch, .. } if epoch == Epoch::new(2)), "{settled:?}");
    assert!(!held(&a, &d), "the other update is dropped");
    assert!(held(&a, &elsewhere), "another Organisation's update is kept");
}

// A reconcile's commit (another signatory wrote C) leaves no other
// provisional update for the Organisation; another Organisation's update is
// kept.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-fptp57
#[tokio::test]
async fn a_reconciled_commit_leaves_no_other_update_for_the_organisation() {
    let (chain, a, org) = founded("publish-orphans-reconciled").await;
    let elsewhere = held_elsewhere(&a).await;
    let c = a.admit_member(&mut OsRng, org, &joiner(0x89, "bob")).await.unwrap();
    let d = a.admit_member(&mut OsRng, org, &joiner(0x8b, "carol")).await.unwrap();
    chain.slots().apply_update(org, c.resulting_root, c.org_pub_key, Epoch::new(1)).unwrap();
    let result = a.reconcile(&mut OsRng, org).await;
    assert!(matches!(result, Reconciliation::Committed { epoch, .. } if epoch == Epoch::new(2)), "{result:?}");
    assert!(!held(&a, &d), "the other update is dropped");
    assert!(held(&a, &elsewhere), "another Organisation's update is kept");
}

/// `sender` submits `update`, sending it at `addr`, while `receiver`
/// receives it; the epoch `receiver` committed. *Reconciled with S3b-io part
/// 1, 2026-10-14:* no named recipient (S4, task T12: the Loopback send goes
/// to the recipient whose Device `addr` names); the receive step is
/// `next_received` (S4, task T11).
async fn submit_and_receive(sender: &OrgIo, receiver: &OrgIo, update: &BuiltUpdate, addr: &iroh::EndpointAddr) -> Epoch {
    let (received, sent) = tokio::join!(
        async { tokio::time::timeout(NET, receiver.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            sender.submit_commit_send(&mut OsRng, update, at(addr.clone())).await
        }
    );
    sent.unwrap();
    match received.unwrap() {
        Received::Committed(outcome) => outcome.epoch,
        other => panic!("a committed update, not {other:?}"),
    }
}

// A received update's commit leaves no other provisional update the
// receiving node built for the Organisation; its update for another
// Organisation is kept.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites; B's
// expectation is declared in B's Persona's store (S4, task T3).
// verifies: LLR-fptp57
#[tokio::test(flavor = "multi_thread")]
async fn a_received_update_leaves_no_other_update_for_the_organisation() {
    let tag = "publish-orphans-received";
    let chain = FakeChain::new();
    let a = handle(tag, "a", &chain);
    let alice = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    let chain_b = FakeChain::sharing_slots_with(&chain);
    let b = handle(tag, "b", &chain_b);
    let bob_persona = persona(&b, "bob").await;
    b.expect_admission(&mut OsRng, &bob_persona, org).await.unwrap();
    let addr_b = receiving_addr(&b).await;
    let bob = joiner_of(&b, &bob_persona);
    let bob_admission = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    assert_eq!(submit_and_receive(&a, &b, &bob_admission, &addr_b).await, Epoch::new(2));

    let elsewhere = held_elsewhere(&b).await;
    let x = b.admit_member(&mut OsRng, org, &joiner(0x91, "xena")).await.unwrap();
    let y = b.admit_member(&mut OsRng, org, &joiner(0x93, "yuri")).await.unwrap();
    let carol_admission = a.admit_member(&mut OsRng, org, &joiner(0x95, "carol")).await.unwrap();
    assert_eq!(submit_and_receive(&a, &b, &carol_admission, &addr_b).await, Epoch::new(3));
    assert!(!held(&b, &x) && !held(&b, &y), "B's other updates are dropped");
    assert!(held(&b, &elsewhere), "B's update for another Organisation is kept");
}

// Abnormal: no commit, no discard. A submission whose chain write fails, and
// a reconcile that finds the chain at a root this node does not hold
// (behind), commit nothing — and every provisional update of the
// Organisation is kept, the submitted one included, on disk too.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-fptp57
#[tokio::test]
async fn a_path_that_commits_nothing_discards_nothing() {
    let tag = "publish-orphans-no-commit";
    let (chain, a, org) = founded(tag).await;
    let bob = joiner(0x97, "bob");
    let c = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let d = a.admit_member(&mut OsRng, org, &joiner(0x99, "carol")).await.unwrap();

    chain.write_failing.store(true, Ordering::SeqCst);
    let err = a.submit_commit_send(&mut OsRng, &c, None).await.unwrap_err();
    chain.write_failing.store(false, Ordering::SeqCst);
    assert!(err.contains("chain write failed"), "{err}");
    assert!(held(&a, &c) && held(&a, &d), "a failed write keeps both updates");

    let current = chain.slots().get(&org).unwrap();
    chain.slots().set(org, OrgState { root_hash: RootHash::new([0xcd; 32]), epoch: Epoch::new(2), ..current });
    assert_eq!(a.reconcile(&mut OsRng, org).await, Reconciliation::Behind { org_id: org, chain_epoch: Epoch::new(2) });
    assert_eq!(epoch_of(&a, org).await, Epoch::new(1), "nothing committed");
    assert!(held(&a, &c) && held(&a, &d), "a reconcile that is behind keeps both updates");
    assert_eq!(reopened_service(tag, "a", &a).provisional_updates(org).len(), 2, "on disk too");
}

// A founding after a failed one leaves no genesis update of the Persona
// held: org-node's commit drops the failed founding's, on disk too.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-36zp69
#[tokio::test]
async fn a_founding_after_a_failed_one_leaves_no_genesis_update_held() {
    let tag = "publish-founding-drops-genesis";
    let chain = FakeChain::new();
    let a = handle(tag, "a", &chain);
    let pid = persona(&a, "alice").await;
    chain.write_failing.store(true, Ordering::SeqCst);
    a.found_organisation(&mut OsRng, &pid).await.unwrap_err();
    chain.write_failing.store(false, Ordering::SeqCst);
    assert_eq!(a.node_for_test().genesis_provisional_updates(&pid).len(), 1, "the failed founding's update is held");
    a.found_organisation(&mut OsRng, &pid).await.unwrap();
    assert!(a.node_for_test().genesis_provisional_updates(&pid).is_empty());
    assert!(reopened_service(tag, "a", &a).genesis_provisional_updates(&pid).is_empty(), "gone from disk");
}
