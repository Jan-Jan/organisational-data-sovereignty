#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The publish state (SDD-6mpmsw): a chain write whose outcome is unknown is
//! recorded (LLR-2yjuaj), settled by reading the chain (LLR-t98pk9) before
//! any other write for its Organisation (LLR-nqvbb6), and provisional
//! updates are discarded in the background: by each commit (LLR-fptp57,
//! LLR-36zp69) and by a settle that finds a write not executed (LLR-t98pk9).
//! The chain is `test_support::FakeChain`. A write that times out is driven
//! on a paused clock (`common::handles::timed_out`).

use std::sync::atomic::Ordering;
use std::time::Duration;

use on_chain_client::write::WRITE_MORTALITY_BLOCKS;
use org_io::node::transport::endpoint::OrgEndpoint;
use org_io::node::transport::wire::WireMessage;
use org_io::node::{DevicePublicKey, DeviceSeed, Epoch, OrgId, OrgState, RootHash};
use org_io::test_support::FakeChain;
use org_io::{BuiltUpdate, OrgIo, Reconciliation, Settled};
use rand::rngs::OsRng;

mod common;
use common::handles::{
    founded, handle, held, hung, joiner, joiner_of, persona, receiving_addr, reopened_service, store_bytes, store_path,
    timed_out, NET,
};

fn epoch_of(io: &OrgIo, org: OrgId) -> Epoch {
    io.view().organisation(org).unwrap().epoch
}

// The reproduction of the app's provisional-housekeeping problem report
// (its ID joins the verifies line when the report moves into org-io's
// ledger, plan task T17a): a write that executes after its 90-second bound
// is committed when the next submission for its Organisation settles it, and
// sent to the Device that submission named (C's), not the new one's; the new
// update (D's) is not written. On S2's code D's write went out and C's update
// was never committed.
// verifies: LLR-2yjuaj, LLR-t98pk9, LLR-nqvbb6, PR-924ftr
#[tokio::test]
async fn a_write_that_lands_after_its_timeout_is_committed_and_sent_to_its_own_recipient() {
    let (chain, mut a, org) = founded("publish-late-write").await;
    let carol = joiner(0x31, "carol");
    let carol_endpoint = OrgEndpoint::bind(&DeviceSeed::from([0x32; 32]).signing_keypair()).await.unwrap();
    let carol_addr = carol_endpoint.inner().addr();
    let c = a.node_mut().admit_member(&mut OsRng, org, &carol).unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&mut a, &c, carol.device_key, Some(carol_addr)).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    assert_eq!(chain.slots().get(&org).unwrap().root_hash, c.resulting_root, "C's write landed");
    assert_eq!(epoch_of(&a, org), Epoch::new(1), "but nothing was committed");
    let writes = chain.update_writes();

    let dave = joiner(0x33, "dave");
    let d = a.node_mut().admit_member(&mut OsRng, org, &dave).unwrap();
    let carol_receives = tokio::spawn(async move {
        let received = tokio::time::timeout(NET, carol_endpoint.recv_one()).await.unwrap().unwrap();
        (carol_endpoint, received)
    });
    let err = a.submit_commit_send(&mut OsRng, &d, dave.device_key, None).await.unwrap_err();
    assert!(err.contains("committed at epoch 2") && err.contains("this update was not written"), "{err}");
    assert_eq!(chain.update_writes(), writes, "D's update was not written");
    let (_endpoint, (_sender, message)) = carol_receives.await.unwrap();
    let WireMessage::OrgInformation { envelope, .. } = message else { panic!("Organisation information for C") };
    assert_eq!(envelope.org_id, org);
    let record = a.view().organisation(org).unwrap();
    assert_eq!((record.epoch, record.root_hash), (Epoch::new(2), c.resulting_root), "C committed");
    assert!(!held(&a, &c), "C is no longer provisional");
}

// Review round 1, finding-1: the app's only way to ask (its reconcile
// button, `reconcile_held`) commits a write that landed after its timeout,
// sends it to the Device its submission named, and clears the unknown write:
// the next settle reads nothing.
// verifies: LLR-k5pzsz, LLR-t98pk9
#[tokio::test]
async fn a_reconcile_that_commits_an_unknown_write_sends_it_to_its_recipient_and_clears_it() {
    let (chain, mut a, org) = founded("publish-reconcile-unknown").await;
    let carol = joiner(0x37, "carol");
    let carol_endpoint = OrgEndpoint::bind(&DeviceSeed::from([0x38; 32]).signing_keypair()).await.unwrap();
    let carol_addr = carol_endpoint.inner().addr();
    let c = a.node_mut().admit_member(&mut OsRng, org, &carol).unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&mut a, &c, carol.device_key, Some(carol_addr)).await;
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
// verifies: LLR-k5pzsz, LLR-nqvbb6
#[tokio::test]
async fn after_a_reconcile_commits_an_unknown_write_the_next_submission_is_written() {
    let (chain, mut a, org) = founded("publish-reconcile-then-submit").await;
    let bob = joiner(0x39, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&mut a, &c, bob.device_key, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let result = a.reconcile(&mut OsRng, org).await;
    assert!(matches!(result, Reconciliation::Committed { epoch, .. } if epoch == Epoch::new(2)), "{result:?}");
    let dave = joiner(0x3b, "dave");
    let d = a.node_mut().admit_member(&mut OsRng, org, &dave).unwrap();
    let writes = chain.update_writes();
    // Written and committed; the Loopback send has no address and fails after.
    let err = a.submit_commit_send(&mut OsRng, &d, dave.device_key, None).await.unwrap_err();
    assert!(err.starts_with("committed at epoch 3, but the send failed"), "{err}");
    assert_eq!(chain.update_writes(), writes + 1, "D was written");
}

// Review round 1, finding-1: a settle that finds its write executed and its
// update already committed (here by org-node's reconcile, driven out of
// org-io's sight) counts as settled: the submission that ran it writes.
// verifies: LLR-t98pk9, LLR-nqvbb6
#[tokio::test]
async fn a_settle_that_finds_its_write_already_committed_lets_the_submission_write() {
    let (chain, mut a, org) = founded("publish-settle-in-step").await;
    let bob = joiner(0x3d, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&mut a, &c, bob.device_key, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let landed = chain.slots().get(&org).unwrap();
    a.node_mut_for_test().reconcile(&mut OsRng, org, Some(landed)).unwrap();
    assert_eq!(epoch_of(&a, org), Epoch::new(2), "C committed");
    let dave = joiner(0x3f, "dave");
    let d = a.node_mut().admit_member(&mut OsRng, org, &dave).unwrap();
    let writes = chain.update_writes();
    let err = a.submit_commit_send(&mut OsRng, &d, dave.device_key, None).await.unwrap_err();
    assert!(err.starts_with("committed at epoch 3, but the send failed"), "{err}");
    assert_eq!(chain.update_writes(), writes + 1, "D was written");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");
}

// Review round 1, finding-3: once the record has moved past the epoch the
// write expected (here a reconcile committed another signatory's update),
// the write can never pass the contract's epoch check: the settle finds it
// not executed at once, within its window, reading nothing.
// verifies: LLR-t98pk9
#[tokio::test]
async fn an_unknown_write_whose_epoch_the_record_has_passed_is_settled_at_once() {
    let (chain, mut a, org) = founded("publish-epoch-passed").await;
    let bob = joiner(0x67, "bob");
    let carol = joiner(0x69, "carol");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    let d = a.node_mut().admit_member(&mut OsRng, org, &carol).unwrap();
    hung(&chain, &mut a, &c, bob.device_key).await;
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

// Review round 1, finding-5: resubmitting the very update whose write the
// settle finds not executed (and discards) is refused with a report saying
// so and that the update must be rebuilt; nothing is written.
// verifies: LLR-nqvbb6
#[tokio::test]
async fn resubmitting_an_update_whose_write_did_not_execute_says_it_must_be_rebuilt() {
    let (chain, mut a, org) = founded("publish-resubmit-not-executed").await;
    let bob = joiner(0x6b, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    hung(&chain, &mut a, &c, bob.device_key).await;
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    let writes = chain.update_writes();
    let err = a.submit_commit_send(&mut OsRng, &c, bob.device_key, None).await.unwrap_err();
    assert!(err.contains("did not execute") && err.contains("rebuild"), "{err}");
    assert_eq!(chain.update_writes(), writes, "nothing written");
    assert!(!held(&a, &c), "discarded");
}

// Review round 2, finding-2: resubmitting the update whose write the settle
// finds not executed because another update committed first (the record is
// past the epoch the write expected; its window has not passed) is refused
// with a report that is true here too: it says the write did not execute and
// the update must be rebuilt, and claims no passed window.
// verifies: LLR-nqvbb6
#[tokio::test]
async fn resubmitting_an_update_another_commit_overtook_says_it_must_be_rebuilt() {
    let (chain, mut a, org) = founded("publish-resubmit-overtaken").await;
    let bob = joiner(0x6d, "bob");
    let carol = joiner(0x6f, "carol");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    let d = a.node_mut().admit_member(&mut OsRng, org, &carol).unwrap();
    hung(&chain, &mut a, &c, bob.device_key).await;
    chain.slots().apply_update(org, d.resulting_root, d.org_pub_key, Epoch::new(1)).unwrap();
    let result = a.reconcile(&mut OsRng, org).await;
    assert!(matches!(result, Reconciliation::Committed { epoch, .. } if epoch == Epoch::new(2)), "{result:?}");
    let writes = chain.update_writes();
    let err = a.submit_commit_send(&mut OsRng, &c, bob.device_key, None).await.unwrap_err();
    assert!(err.contains("did not execute") && err.contains("rebuild"), "{err}");
    assert!(!err.contains("window"), "no window has passed: {err}");
    assert_eq!(chain.update_writes(), writes, "nothing written");
}

// The window is read once, before the write; a failed read writes nothing,
// records no unknown write and keeps the update.
// verifies: LLR-2yjuaj
#[tokio::test]
async fn a_failed_window_read_writes_nothing() {
    let (chain, mut a, org) = founded("publish-window-fails").await;
    let bob = joiner(0x41, "bob");
    let update = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    chain.write_window_failing.store(true, Ordering::SeqCst);
    let err = a.submit_commit_send(&mut OsRng, &update, bob.device_key, None).await.unwrap_err();
    assert!(err.contains("no write was made") && err.contains("node unreachable"), "{err}");
    assert_eq!(chain.update_writes(), 0, "nothing written");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(1));
    assert!(held(&a, &update), "the update is kept");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "no unknown write recorded");
}

// A write that failed (not timed out) records no unknown write: the next
// settle reads nothing.
// verifies: LLR-2yjuaj
#[tokio::test]
async fn a_failed_write_records_no_unknown_write() {
    let (chain, mut a, org) = founded("publish-write-fails").await;
    let bob = joiner(0x43, "bob");
    let update = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    chain.write_failing.store(true, Ordering::SeqCst);
    let err = a.submit_commit_send(&mut OsRng, &update, bob.device_key, None).await.unwrap_err();
    assert!(err.contains("chain write failed"), "{err}");
    let (reads, finalised_reads) = (chain.reads(), chain.finalised_reads());
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite);
    assert_eq!((chain.reads(), chain.finalised_reads()), (reads, finalised_reads), "nothing read");
}

// An unknown write refuses every later write for its Organisation while the
// outcome stays unknown, and no write for another Organisation.
// verifies: LLR-nqvbb6
#[tokio::test]
async fn an_unknown_write_blocks_a_second_write_for_that_organisation_only() {
    let (chain, mut a, first) = founded("publish-blocks-one").await;
    let bea = persona(&mut a, "bea");
    let second = a.found_organisation(&mut OsRng, &bea).await.unwrap();
    let bob = joiner(0x45, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, first, &bob).unwrap();
    hung(&chain, &mut a, &c, bob.device_key).await;
    let writes = chain.update_writes();

    let carol = joiner(0x47, "carol");
    let d = a.node_mut().admit_member(&mut OsRng, first, &carol).unwrap();
    let err = a.submit_commit_send(&mut OsRng, &d, carol.device_key, None).await.unwrap_err();
    assert!(err.contains("unknown outcome") && err.contains("this update was not written"), "{err}");
    assert_eq!(chain.update_writes(), writes, "no write for the Organisation with an unknown write");

    let e = a.node_mut().admit_member(&mut OsRng, second, &carol).unwrap();
    // Written and committed; the Loopback send has no address and fails after.
    let err = a.submit_commit_send(&mut OsRng, &e, carol.device_key, None).await.unwrap_err();
    assert!(err.starts_with("committed at epoch 2, but the send failed"), "{err}");
    assert_eq!(chain.update_writes(), writes + 1, "the other Organisation's write went out");
    assert_eq!(epoch_of(&a, second), Epoch::new(2));
    assert_eq!(epoch_of(&a, first), Epoch::new(1));
}

// The chain moved past the record, but not to the unknown write's root: the
// result is the reconcile's for that state — another held update committed,
// or behind — and the unknown write is cleared.
// verifies: LLR-t98pk9
#[tokio::test]
async fn an_unknown_write_whose_chain_moved_elsewhere_is_reconciled() {
    let (chain, mut a, org) = founded("publish-moved-elsewhere").await;
    let bob = joiner(0x51, "bob");
    let carol = joiner(0x53, "carol");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    let d = a.node_mut().admit_member(&mut OsRng, org, &carol).unwrap();
    hung(&chain, &mut a, &c, bob.device_key).await;
    // Another signatory wrote D, which this node also holds.
    chain.slots().apply_update(org, d.resulting_root, d.org_pub_key, Epoch::new(1)).unwrap();
    assert_eq!(
        a.settle_unknown(&mut OsRng, org).await,
        Settled::Reconciled(Reconciliation::Committed { org_id: org, epoch: Epoch::new(2), recipients: vec![carol.device_key] })
    );
    assert_eq!(a.view().organisation(org).unwrap().root_hash, d.resulting_root);
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");

    // Then to a root this node does not hold: behind.
    let dave = joiner(0x55, "dave");
    let e = a.node_mut().admit_member(&mut OsRng, org, &dave).unwrap();
    hung(&chain, &mut a, &e, dave.device_key).await;
    let current = chain.slots().get(&org).unwrap();
    chain.slots().set(org, OrgState { root_hash: RootHash::new([0xab; 32]), epoch: Epoch::new(3), ..current });
    assert_eq!(
        a.settle_unknown(&mut OsRng, org).await,
        Settled::Reconciled(Reconciliation::Behind { org_id: org, chain_epoch: Epoch::new(3) })
    );
    assert_eq!(epoch_of(&a, org), Epoch::new(2), "nothing committed");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");
}

// A failed read — of the finalised block or of the state — or a state the
// parse refuses leaves the outcome unknown, the record and the store as they
// were; once the chain answers, the write is settled.
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_failed_read_leaves_the_outcome_unknown_and_changes_nothing() {
    let tag = "publish-read-fails";
    let (chain, mut a, org) = founded(tag).await;
    let bob = joiner(0x57, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&mut a, &c, bob.device_key, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let before = store_bytes(tag, "a");
    for switch in [&chain.read_failing, &chain.parse_refusing] {
        switch.store(true, Ordering::SeqCst);
        let settled = a.settle_unknown(&mut OsRng, org).await;
        assert!(matches!(&settled, Settled::StillUnknown { org_id, .. } if *org_id == org), "{settled:?}");
        assert_eq!(epoch_of(&a, org), Epoch::new(1), "nothing committed");
        assert!(held(&a, &c), "the update is kept");
        assert_eq!(store_bytes(tag, "a"), before, "nothing written");
        switch.store(false, Ordering::SeqCst);
    }
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(matches!(settled, Settled::Committed { epoch, .. } if epoch == Epoch::new(2)), "{settled:?}");
}

// At the record's epoch, up to and including the window's last valid block,
// the write may still be included: the outcome stays unknown.
// verifies: LLR-t98pk9
#[tokio::test]
async fn within_the_window_the_outcome_stays_unknown() {
    let (chain, mut a, org) = founded("publish-within-window").await;
    let bob = joiner(0x59, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    hung(&chain, &mut a, &c, bob.device_key).await;
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
// verifies: LLR-t98pk9
#[tokio::test]
async fn past_the_window_at_the_records_epoch_the_write_is_not_executed() {
    let (chain, mut a, org) = founded("publish-past-window").await;
    let bob = joiner(0x5b, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    hung(&chain, &mut a, &c, bob.device_key).await;
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NotExecuted { org_id: org });
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite, "cleared");

    // Through the submission: the second unknown write settles as not
    // executed, then the next update is written.
    let carol = joiner(0x5d, "carol");
    let d = a.node_mut().admit_member(&mut OsRng, org, &carol).unwrap();
    let e = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    hung(&chain, &mut a, &d, carol.device_key).await;
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    let writes = chain.update_writes();
    let err = a.submit_commit_send(&mut OsRng, &e, bob.device_key, None).await.unwrap_err();
    assert!(err.starts_with("committed at epoch 2, but the send failed"), "{err}");
    assert_eq!(chain.update_writes(), writes + 1);
    assert_eq!(epoch_of(&a, org), Epoch::new(2));
}

// No unknown write, or an Organisation not held: nothing is read.
// verifies: LLR-t98pk9
#[tokio::test]
async fn settle_with_no_unknown_write_reads_nothing() {
    let (chain, mut a, org) = founded("publish-settle-none").await;
    let (reads, finalised_reads) = (chain.reads(), chain.finalised_reads());
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite);
    assert_eq!(a.settle_unknown(&mut OsRng, OrgId::new([9; 20])).await, Settled::NoUnknownWrite);
    assert_eq!((chain.reads(), chain.finalised_reads()), (reads, finalised_reads));
}

// A read that finds no state, or a state older than the record's, says
// nothing about the write: the outcome stays unknown, the unknown write is
// kept (the next settle reads again) and nothing changes.
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_settle_that_finds_no_state_or_an_older_state_stays_unknown() {
    let tag = "publish-settle-no-state";
    let (chain, mut a, org) = founded(tag).await;
    let bob = joiner(0x5f, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    hung(&chain, &mut a, &c, bob.device_key).await;
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
    assert_eq!(epoch_of(&a, org), Epoch::new(1));
    chain.slots().set(org, current);
    let finalised_reads = chain.finalised_reads();
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NotExecuted { org_id: org }, "the write was kept");
    assert_eq!(chain.finalised_reads(), finalised_reads + 1);
}

// The record is no longer held when the settle finds the chain at another
// root (this node was removed meanwhile): org-node's reconcile refuses it as
// not held, reported as a refused reconciliation, and the unknown write is
// cleared.
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_settle_whose_record_is_no_longer_held_is_refused() {
    let (chain, mut a, org) = founded("publish-settle-not-held").await;
    let bob = joiner(0x71, "bob");
    let admission = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    let err = a.submit_commit_send(&mut OsRng, &admission, bob.device_key, None).await.unwrap_err();
    assert!(err.starts_with("committed at epoch 2, but the send failed"), "{err}");
    let carol = joiner(0x73, "carol");
    let c = a.node_mut().admit_member(&mut OsRng, org, &carol).unwrap();
    let own_member = a.view().personas()[0].member_id.unwrap();
    let removal = a.node_mut().revoke_member(&mut OsRng, org, own_member).unwrap();
    hung(&chain, &mut a, &c, carol.device_key).await;
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
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_committed_late_write_whose_send_fails_keeps_the_commit() {
    let tag = "publish-late-send-fails";
    let (chain, mut a, org) = founded(tag).await;
    let bob = joiner(0x5d, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    // No address: the Loopback send refuses.
    timed_out(&mut a, &c, bob.device_key, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let settled = a.settle_unknown(&mut OsRng, org).await;
    let Settled::Committed { org_id, epoch, sent } = settled else { panic!("{settled:?}") };
    assert_eq!((org_id, epoch), (org, Epoch::new(2)));
    assert!(sent.unwrap_err().contains("Loopback send requires"));
    assert_eq!(epoch_of(&a, org), Epoch::new(2));
    assert_eq!(reopened_service(tag, "a").list_orgs()[0].epoch, Epoch::new(2), "the commit is saved");
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NoUnknownWrite);
}

// A settle that finds the write not executed discards its update itself,
// with the Organisation private key it holds: not held afterwards, and gone
// from the store reopened from disk; another update of the Organisation is
// kept.
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_write_known_not_executed_is_discarded_with_its_key() {
    let tag = "publish-not-executed-discarded";
    let (chain, mut a, org) = founded(tag).await;
    let bob = joiner(0x61, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    let d = a.node_mut().admit_member(&mut OsRng, org, &joiner(0x63, "carol")).unwrap();
    hung(&chain, &mut a, &c, bob.device_key).await;
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NotExecuted { org_id: org });
    assert!(!held(&a, &c), "the update is discarded");
    assert!(held(&a, &d), "the other update is kept");
    let on_disk = reopened_service(tag, "a").provisional_updates(org);
    assert!(on_disk.iter().all(|update| update.resulting_root != c.resulting_root), "C and its key are gone from disk");
    assert_eq!(on_disk.len(), 1);
}

// A settle whose discard cannot save the store reports the outcome as still
// unknown, with the reason, and keeps the unknown write: the next settle,
// with the store writable again, finds it not executed.
// verifies: LLR-t98pk9
#[tokio::test]
async fn a_settle_that_cannot_save_its_discard_stays_unknown() {
    use std::os::unix::fs::PermissionsExt;
    let tag = "publish-not-executed-unsaved";
    let (chain, mut a, org) = founded(tag).await;
    let bob = joiner(0x65, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    hung(&chain, &mut a, &c, bob.device_key).await;
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    let path = store_path(tag, "a");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o400)).unwrap();
    let settled = a.settle_unknown(&mut OsRng, org).await;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(
        matches!(&settled, Settled::StillUnknown { org_id, reason } if *org_id == org && reason.contains("discard")),
        "{settled:?}"
    );
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NotExecuted { org_id: org }, "the write was kept");
    assert!(!held(&a, &c));
}

/// An admission held for another Organisation of `io`, founded by a new
/// Persona: a commit to the first Organisation must keep it.
async fn held_elsewhere(io: &mut OrgIo) -> BuiltUpdate {
    let bea = persona(io, "bea");
    let other = io.found_organisation(&mut OsRng, &bea).await.unwrap();
    io.node_mut().admit_member(&mut OsRng, other, &joiner(0x7d, "erin")).unwrap()
}

// A submission's commit leaves no other provisional update for its
// Organisation (org-node's commit drops the orphans), on disk too; another
// Organisation's update is kept.
// verifies: LLR-fptp57
#[tokio::test]
async fn a_submitted_commit_leaves_no_other_update_for_the_organisation() {
    let tag = "publish-orphans-submitted";
    let (_chain, mut a, org) = founded(tag).await;
    let elsewhere = held_elsewhere(&mut a).await;
    let bob = joiner(0x81, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    let d = a.node_mut().admit_member(&mut OsRng, org, &joiner(0x83, "carol")).unwrap();
    let err = a.submit_commit_send(&mut OsRng, &c, bob.device_key, None).await.unwrap_err();
    assert!(err.starts_with("committed at epoch 2, but the send failed"), "{err}");
    assert!(!held(&a, &d), "the other update is dropped");
    assert!(held(&a, &elsewhere), "another Organisation's update is kept");
    assert!(reopened_service(tag, "a").provisional_updates(org).is_empty(), "on disk too");
}

// A settled late write's commit leaves no other provisional update for its
// Organisation; another Organisation's update is kept.
// verifies: LLR-fptp57
#[tokio::test]
async fn a_settled_late_write_leaves_no_other_update_for_the_organisation() {
    let (chain, mut a, org) = founded("publish-orphans-settled").await;
    let elsewhere = held_elsewhere(&mut a).await;
    let bob = joiner(0x85, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    let d = a.node_mut().admit_member(&mut OsRng, org, &joiner(0x87, "carol")).unwrap();
    chain.write_lands_then_hangs.store(true, Ordering::SeqCst);
    timed_out(&mut a, &c, bob.device_key, None).await;
    chain.write_lands_then_hangs.store(false, Ordering::SeqCst);
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(matches!(settled, Settled::Committed { epoch, .. } if epoch == Epoch::new(2)), "{settled:?}");
    assert!(!held(&a, &d), "the other update is dropped");
    assert!(held(&a, &elsewhere), "another Organisation's update is kept");
}

// A reconcile's commit (another signatory wrote C) leaves no other
// provisional update for the Organisation; another Organisation's update is
// kept.
// verifies: LLR-fptp57
#[tokio::test]
async fn a_reconciled_commit_leaves_no_other_update_for_the_organisation() {
    let (chain, mut a, org) = founded("publish-orphans-reconciled").await;
    let elsewhere = held_elsewhere(&mut a).await;
    let c = a.node_mut().admit_member(&mut OsRng, org, &joiner(0x89, "bob")).unwrap();
    let d = a.node_mut().admit_member(&mut OsRng, org, &joiner(0x8b, "carol")).unwrap();
    chain.slots().apply_update(org, c.resulting_root, c.org_pub_key, Epoch::new(1)).unwrap();
    let result = a.reconcile(&mut OsRng, org).await;
    assert!(matches!(result, Reconciliation::Committed { epoch, .. } if epoch == Epoch::new(2)), "{result:?}");
    assert!(!held(&a, &d), "the other update is dropped");
    assert!(held(&a, &elsewhere), "another Organisation's update is kept");
}

/// `sender` submits `update` and sends it to `recipient` at `addr` while
/// `receiver` receives it; the epoch `receiver` committed.
async fn submit_and_receive(
    sender: &mut OrgIo,
    receiver: &mut OrgIo,
    update: &BuiltUpdate,
    recipient: DevicePublicKey,
    addr: &iroh::EndpointAddr,
) -> Epoch {
    let (received, sent) = tokio::join!(
        async { tokio::time::timeout(NET, receiver.receive_and_verify(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            sender.submit_commit_send(&mut OsRng, update, recipient, Some(addr.clone())).await
        }
    );
    sent.unwrap();
    received.unwrap().epoch
}

// A received update's commit leaves no other provisional update the
// receiving node built for the Organisation; its update for another
// Organisation is kept.
// verifies: LLR-fptp57
#[tokio::test(flavor = "multi_thread")]
async fn a_received_update_leaves_no_other_update_for_the_organisation() {
    let tag = "publish-orphans-received";
    let chain = FakeChain::new();
    let mut a = handle(tag, "a", &chain);
    let alice = persona(&mut a, "alice");
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    let chain_b = FakeChain::sharing_slots_with(&chain);
    let mut b = handle(tag, "b", &chain_b);
    let bob_persona = persona(&mut b, "bob");
    b.node_mut().expect_admission(&mut OsRng, org).unwrap();
    let addr_b = receiving_addr(&mut b).await;
    let bob = joiner_of(&b, &bob_persona);
    let bob_admission = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    assert_eq!(submit_and_receive(&mut a, &mut b, &bob_admission, bob.device_key, &addr_b).await, Epoch::new(2));

    let elsewhere = held_elsewhere(&mut b).await;
    let x = b.node_mut().admit_member(&mut OsRng, org, &joiner(0x91, "xena")).unwrap();
    let y = b.node_mut().admit_member(&mut OsRng, org, &joiner(0x93, "yuri")).unwrap();
    let carol_admission = a.node_mut().admit_member(&mut OsRng, org, &joiner(0x95, "carol")).unwrap();
    assert_eq!(submit_and_receive(&mut a, &mut b, &carol_admission, bob.device_key, &addr_b).await, Epoch::new(3));
    assert!(!held(&b, &x) && !held(&b, &y), "B's other updates are dropped");
    assert!(held(&b, &elsewhere), "B's update for another Organisation is kept");
}

// Abnormal: no commit, no discard. A submission whose chain write fails, and
// a reconcile that finds the chain at a root this node does not hold
// (behind), commit nothing — and every provisional update of the
// Organisation is kept, the submitted one included, on disk too.
// verifies: LLR-fptp57
#[tokio::test]
async fn a_path_that_commits_nothing_discards_nothing() {
    let tag = "publish-orphans-no-commit";
    let (chain, mut a, org) = founded(tag).await;
    let bob = joiner(0x97, "bob");
    let c = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    let d = a.node_mut().admit_member(&mut OsRng, org, &joiner(0x99, "carol")).unwrap();

    chain.write_failing.store(true, Ordering::SeqCst);
    let err = a.submit_commit_send(&mut OsRng, &c, bob.device_key, None).await.unwrap_err();
    chain.write_failing.store(false, Ordering::SeqCst);
    assert!(err.contains("chain write failed"), "{err}");
    assert!(held(&a, &c) && held(&a, &d), "a failed write keeps both updates");

    let current = chain.slots().get(&org).unwrap();
    chain.slots().set(org, OrgState { root_hash: RootHash::new([0xcd; 32]), epoch: Epoch::new(2), ..current });
    assert_eq!(a.reconcile(&mut OsRng, org).await, Reconciliation::Behind { org_id: org, chain_epoch: Epoch::new(2) });
    assert_eq!(epoch_of(&a, org), Epoch::new(1), "nothing committed");
    assert!(held(&a, &c) && held(&a, &d), "a reconcile that is behind keeps both updates");
    assert_eq!(reopened_service(tag, "a").provisional_updates(org).len(), 2, "on disk too");
}

// A founding after a failed one leaves no genesis update of the Persona
// held: org-node's commit drops the failed founding's, on disk too.
// verifies: LLR-36zp69
#[tokio::test]
async fn a_founding_after_a_failed_one_leaves_no_genesis_update_held() {
    let tag = "publish-founding-drops-genesis";
    let chain = FakeChain::new();
    let mut a = handle(tag, "a", &chain);
    let pid = persona(&mut a, "alice");
    chain.write_failing.store(true, Ordering::SeqCst);
    a.found_organisation(&mut OsRng, &pid).await.unwrap_err();
    chain.write_failing.store(false, Ordering::SeqCst);
    assert_eq!(a.node_for_test().genesis_provisional_updates(&pid).len(), 1, "the failed founding's update is held");
    a.found_organisation(&mut OsRng, &pid).await.unwrap();
    assert!(a.node_for_test().genesis_provisional_updates(&pid).is_empty());
    assert!(reopened_service(tag, "a").genesis_provisional_updates(&pid).is_empty(), "gone from disk");
}
