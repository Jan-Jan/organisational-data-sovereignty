#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Reconcile at start and on request (LLR-k5pzsz), the unverified state
//! (LLR-kp95ty), and no deletion but on a verified path (LLR-cc59ys, its
//! behavioural half). The chain is `test_support::FakeChain`; a restart is
//! the handle dropped and a new one built over the store reopened from disk,
//! with its own view of the same chain slots.
//!
//! *Reconciled with S3b-io part 1, 2026-10-14 (merge into S4,
//! `worktree-org-io-transport`):* every test keeps its name, its scenario
//! and its `verifies:` line; its call sites are S4's: the `&self` handle and
//! its `async` builders and views (S4, task T11), no named recipient and a
//! committed update whose failed send is reported in `Submitted::sends`
//! (S4, task T12), one receive path (`next_received`, S4, tasks T4 and T11),
//! Loopback sinks bound as the recipient's Device (S4, task T10). A restart
//! is `OrgIo::open_for_test` over the data directory, the app's own start
//! path (S4, tasks T9 and T15), in place of a service rebuilt from the
//! store file; the store on disk is read back with the keychain items.

use std::sync::atomic::Ordering;
use std::time::Duration;

use on_chain_client::write::WRITE_MORTALITY_BLOCKS;
use org_io::test_support::FakeChain;
use org_io::transport::TransportMode;
use org_io::{OrgIo, OrgSummary, PersonaSummary, Reconciliation, Settled, Verification};
use org_node::store::ProvisionalUpdate;
use org_node::wire::WireMessage;
use org_node::{Epoch, Joiner, MemberId, OrgId, OrgState, RootHash};
use rand::rngs::OsRng;

mod common;
use common::handles::{
    at, deliver, founded, held, hung, joiner, persona, receiving_addr, reopened_service, spawn_recv_one, store_bytes,
    store_dir, NET,
};

/// Admit `who` through the publish path: written and committed; the
/// Loopback send has no address and fails after the commit (S4, task T12:
/// reported per Device in `sends`, not as an error).
async fn admit(io: &OrgIo, org: OrgId, who: &Joiner) {
    let update = io.admit_member(&mut OsRng, org, who).await.unwrap();
    let submitted = io.submit_commit_send(&mut OsRng, &update, None).await.unwrap();
    assert!(submitted.sends.per_device.iter().all(|(_, sent)| sent.is_err()), "{:?}", submitted.sends);
}

async fn member_with_device(io: &OrgIo, org: OrgId, who: &Joiner) -> MemberId {
    io.organisation(org).await.unwrap().members.iter().find(|member| member.device_keys.contains(&who.device_key)).unwrap().id
}

/// The handle after a restart: the data directory as it is on disk, the
/// same chain.
fn restarted(tag: &str, chain: &FakeChain) -> OrgIo {
    OrgIo::open_for_test(&store_dir(tag, "a"), TransportMode::Loopback, &FakeChain::sharing_slots_with(chain)).unwrap()
}

/// What the store holds on disk: its Organisations, Personas and held
/// updates, public fields compared; unsealed with the keychain items `io`
/// reads.
fn on_disk(tag: &str, io: &OrgIo) -> (Vec<OrgSummary>, Vec<PersonaSummary>, Vec<ProvisionalUpdate>) {
    let service = reopened_service(tag, "a", io);
    let orgs: Vec<OrgSummary> = service.list_orgs().into_iter().map(OrgSummary::from).collect();
    let updates = orgs.iter().flat_map(|org| service.provisional_updates(org.org_id)).collect();
    (orgs, service.list_personas().into_iter().map(PersonaSummary::from).collect(), updates)
}

// The crash window of a publish: the write executed, then the node stopped
// before it committed. At the next start the held update is committed from
// the chain alone, and reported with the Devices it is for (the revoked
// Device's notice among them); nothing is sent.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites; the
// restart is `OrgIo::open_for_test` over the data directory.
// verifies: LLR-k5pzsz
#[tokio::test]
async fn a_write_that_executed_before_a_crash_is_committed_at_the_next_start() {
    let tag = "reconcile-crash";
    let (chain, a, org) = founded(tag).await;
    let bob = joiner(0x11, "bob");
    admit(&a, org, &bob).await;
    let bob_member = member_with_device(&a, org, &bob).await;
    let revocation = a.revoke_member(&mut OsRng, org, bob_member).await.unwrap();
    // The write executes; the node stops before its read-back.
    chain.slots().apply_update(org, revocation.resulting_root, revocation.org_pub_key, Epoch::new(2)).unwrap();
    drop(a);

    let a = restarted(tag, &chain);
    assert!(held(&a, &revocation), "the update was kept across the restart");
    let results = a.reconcile_held(&mut OsRng).await;
    let [Reconciliation::Committed { org_id, epoch, recipients }] = results.as_slice() else { panic!("{results:?}") };
    assert_eq!((*org_id, *epoch), (org, Epoch::new(3)));
    assert!(recipients.contains(&bob.device_key), "the revoked Device is told: {recipients:?}");
    let on_chain = chain.slots().get(&org).unwrap();
    let record = a.organisation(org).await.unwrap();
    assert_eq!((record.epoch, record.root_hash, record.org_pub_key), (on_chain.epoch, on_chain.root_hash, on_chain.org_pub_key));
    assert!(!held(&a, &revocation), "no longer provisional");
    assert_eq!(on_disk(tag, &a).0[0].epoch, Epoch::new(3), "the commit is saved");
}

// In step, behind, and each refusal (conflict, stale, no state, not held)
// are reported and change nothing: the store's bytes are as they were.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-k5pzsz
#[tokio::test]
async fn in_step_behind_and_refused_are_reported_and_change_nothing() {
    let tag = "reconcile-reports";
    let (chain, a, org) = founded(tag).await;
    let before = store_bytes(tag, "a");
    let current = chain.slots().get(&org).unwrap();
    let other_root = RootHash::new([0xab; 32]);

    assert_eq!(a.reconcile(&mut OsRng, org).await, Reconciliation::InStep { org_id: org, epoch: Epoch::new(1) });
    assert_eq!(store_bytes(tag, "a"), before);

    chain.slots().set(org, OrgState { root_hash: other_root, epoch: Epoch::new(2), ..current });
    assert_eq!(a.reconcile(&mut OsRng, org).await, Reconciliation::Behind { org_id: org, chain_epoch: Epoch::new(2) });
    assert_eq!(store_bytes(tag, "a"), before);

    for refused_state in [
        OrgState { root_hash: other_root, ..current },
        OrgState { epoch: Epoch::new(0), ..current },
    ] {
        chain.slots().set(org, refused_state);
        let result = a.reconcile(&mut OsRng, org).await;
        assert!(matches!(&result, Reconciliation::Refused { org_id, .. } if *org_id == org), "{result:?}");
        assert_eq!(store_bytes(tag, "a"), before);
    }
    chain.slots().set(org, current);

    chain.state_hidden.store(true, Ordering::SeqCst);
    let result = a.reconcile(&mut OsRng, org).await;
    assert!(matches!(&result, Reconciliation::Refused { reason, .. } if reason.contains("no on-chain state")), "{result:?}");
    chain.state_hidden.store(false, Ordering::SeqCst);

    let unheld = OrgId::new([9; 20]);
    let result = a.reconcile(&mut OsRng, unheld).await;
    assert!(matches!(&result, Reconciliation::Refused { org_id, .. } if *org_id == unheld), "{result:?}");
    assert_eq!(store_bytes(tag, "a"), before);
    assert_eq!(a.organisation(org).await.unwrap().epoch, Epoch::new(1));
}

// A reconciled commit that removes this node reports `Removed` and nothing
// else: the acknowledgements org-node signed are not returned. The record
// is forgotten, on disk too (org-node's verified removal path).
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites; the removal
// is signed with the keychain's secret and removes the Persona's store file
// (S4, tasks T9 and T11).
// verifies: LLR-k5pzsz
#[tokio::test]
async fn a_reconciled_removal_of_this_node_is_reported_removed_and_returns_no_acknowledgement() {
    let tag = "reconcile-removed";
    let (chain, a, org) = founded(tag).await;
    admit(&a, org, &joiner(0x13, "bob")).await;
    let own_member = a.personas().await[0].member_id.unwrap();
    let removal = a.revoke_member(&mut OsRng, org, own_member).await.unwrap();
    chain.slots().apply_update(org, removal.resulting_root, removal.org_pub_key, Epoch::new(2)).unwrap();
    assert_eq!(a.reconcile(&mut OsRng, org).await, Reconciliation::Removed { org_id: org });
    assert!(a.organisation(org).await.is_none());
    assert!(on_disk(tag, &a).0.is_empty(), "forgotten on disk");
}

// `reconcile_held` reconciles every held Organisation, once each, in the
// store's order.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites.
// verifies: LLR-k5pzsz
#[tokio::test]
async fn reconcile_held_reports_every_held_organisation_in_store_order() {
    let (chain, a, first) = founded("reconcile-held-order").await;
    let bea = persona(&a, "bea").await;
    let second = a.found_organisation(&mut OsRng, &bea).await.unwrap();
    let current = chain.slots().get(&second).unwrap();
    chain.slots().set(second, OrgState { root_hash: RootHash::new([0xab; 32]), epoch: Epoch::new(2), ..current });
    let store_order: Vec<OrgId> = a.organisations().await.iter().map(|org| org.org_id).collect();
    assert_eq!(store_order, vec![first, second]);
    let reads = chain.reads();
    assert_eq!(
        a.reconcile_held(&mut OsRng).await,
        vec![
            Reconciliation::InStep { org_id: first, epoch: Epoch::new(1) },
            Reconciliation::Behind { org_id: second, chain_epoch: Epoch::new(2) },
        ]
    );
    assert_eq!(chain.reads(), reads + 2, "one read per Organisation");
}

// A failed read or a refused parse is `Unverified` with its reason: no
// org-node operation runs, so a held update the chain shows is not
// committed and nothing is written; `verification` stays `Unverified` until
// a reconcile reads a state. A handle with the chain not configured is
// unverified for every Organisation.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites
// (`verification` is `async`: the publish state is behind its own lock); the
// handle with the chain not configured is `OrgIo::open` over the data
// directory, as the app opens it before it connects.
// verifies: LLR-kp95ty
#[tokio::test]
async fn an_unreachable_chain_marks_unverified_and_commits_and_deletes_nothing() {
    let tag = "reconcile-unverified";
    let (chain, a, org) = founded(tag).await;
    assert_eq!(a.verification(org).await, None, "not reconciled yet");
    let bob = joiner(0x15, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.slots().apply_update(org, update.resulting_root, update.org_pub_key, Epoch::new(1)).unwrap();
    let before = store_bytes(tag, "a");
    for (switch, cause) in [(&chain.read_failing, "node unreachable"), (&chain.parse_refusing, "not a valid X25519")] {
        switch.store(true, Ordering::SeqCst);
        let result = a.reconcile(&mut OsRng, org).await;
        let Reconciliation::Unverified { org_id, reason } = &result else { panic!("{result:?}") };
        assert_eq!(*org_id, org);
        assert!(!reason.is_empty() && reason.contains(cause), "{reason}");
        assert_eq!(a.verification(org).await, Some(Verification::Unverified { reason: reason.clone() }));
        assert_eq!(a.organisation(org).await.unwrap().epoch, Epoch::new(1), "nothing committed");
        assert!(held(&a, &update), "the update is kept");
        assert_eq!(store_bytes(tag, "a"), before, "nothing written");
        switch.store(false, Ordering::SeqCst);
    }
    let result = a.reconcile(&mut OsRng, org).await;
    assert!(matches!(result, Reconciliation::Committed { epoch, .. } if epoch == Epoch::new(2)), "{result:?}");
    assert_eq!(a.verification(org).await, Some(Verification::Verified));
    assert_eq!(a.reconcile(&mut OsRng, org).await, Reconciliation::InStep { org_id: org, epoch: Epoch::new(2) });
    assert_eq!(a.verification(org).await, Some(Verification::Verified));

    drop(a);
    let offline = OrgIo::open(&store_dir(tag, "a"), TransportMode::Loopback).unwrap();
    let before = store_bytes(tag, "a");
    let results = offline.reconcile_held(&mut OsRng).await;
    let [Reconciliation::Unverified { org_id, reason }] = results.as_slice() else { panic!("{results:?}") };
    assert_eq!(*org_id, org);
    assert_eq!(offline.verification(org).await, Some(Verification::Unverified { reason: reason.clone() }));
    assert_eq!(store_bytes(tag, "a"), before, "nothing written");
}

// No trigger but a verified one deletes anything: after each non-verified
// trigger the Organisation's record, the Personas and the other held updates
// are unchanged on disk.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's call sites. The sinks
// that capture Bob's messages are bound as Bob's Device (S4, task T10: a
// Loopback send goes only to a recipient's address). The two receive paths
// are one since S4 task T4, so both refused messages go through
// `next_received`.
// verifies: LLR-cc59ys
#[tokio::test]
async fn no_deletion_on_any_trigger_but_a_verified_one() {
    let tag = "reconcile-no-deletion";
    let (chain, a, org) = founded(tag).await;
    // Bob admitted, then revoked, each message captured for the relays below.
    let bob = joiner(0x17, "bob");
    let bob_device_seed = [0x18; 32];
    let admission = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let (sink_addr, sink) = spawn_recv_one(bob_device_seed).await;
    a.submit_commit_send(&mut OsRng, &admission, at(sink_addr)).await.unwrap();
    let (_, _, captured_admission) = sink.await.unwrap();
    let bob_member = member_with_device(&a, org, &bob).await;
    let revocation = a.revoke_member(&mut OsRng, org, bob_member).await.unwrap();
    let (sink_addr, sink) = spawn_recv_one(bob_device_seed).await;
    a.submit_commit_send(&mut OsRng, &revocation, at(sink_addr)).await.unwrap();
    let (_, _, captured_notice) = sink.await.unwrap();
    assert!(matches!(captured_notice, WireMessage::Revocation(_)), "{captured_notice:?}");
    let carol = joiner(0x19, "carol");
    let c = a.admit_member(&mut OsRng, org, &carol).await.unwrap();
    let d = a.admit_member(&mut OsRng, org, &joiner(0x1b, "dave")).await.unwrap();
    let before = on_disk(tag, &a);
    assert_eq!(before.2.len(), 2);
    let unchanged = |trigger: &str| assert_eq!(on_disk(tag, &a), before, "after {trigger}");

    for switch in [&chain.read_failing, &chain.parse_refusing] {
        switch.store(true, Ordering::SeqCst);
        a.reconcile(&mut OsRng, org).await;
        switch.store(false, Ordering::SeqCst);
    }
    unchanged("a failed read and a refused parse");

    let current = chain.slots().get(&org).unwrap();
    let other_root = RootHash::new([0xab; 32]);
    for (trigger, state) in [
        ("behind", OrgState { root_hash: other_root, epoch: Epoch::new(current.epoch.get() + 1), ..current }),
        ("stale", OrgState { epoch: Epoch::new(current.epoch.get() - 1), ..current }),
        ("conflict", OrgState { root_hash: other_root, ..current }),
    ] {
        chain.slots().set(org, state);
        a.reconcile(&mut OsRng, org).await;
        unchanged(trigger);
    }
    chain.slots().set(org, current);

    // An unknown write, then its settle within the window.
    hung(&chain, &a, &c).await;
    let settled = a.settle_unknown(&mut OsRng, org).await;
    assert!(matches!(settled, Settled::StillUnknown { .. }), "{settled:?}");
    unchanged("an unknown write");

    // A refused message on the receive path: relayed by a Device no record
    // lists.
    let addr_a = receiving_addr(&a).await;
    for captured in [&captured_admission, &captured_notice] {
        let (received, ()) = tokio::join!(
            async { tokio::time::timeout(NET, a.next_received(&mut OsRng)).await.unwrap() },
            async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                deliver(addr_a.clone(), captured).await;
            }
        );
        assert!(received.is_err(), "{received:?}");
    }
    unchanged("a refused message on the receive path");

    // The settle's own discard of a write known not executed removes that
    // update only. *Amended 2026-10-13 (owner ruling of that day, background
    // discard):* this was the app's discard of another update, removed.
    chain.advance_finalised(WRITE_MORTALITY_BLOCKS + 1);
    assert_eq!(a.settle_unknown(&mut OsRng, org).await, Settled::NotExecuted { org_id: org });
    let (orgs, personas, updates) = on_disk(tag, &a);
    assert_eq!((orgs, personas), (before.0, before.1), "after the settle's discard");
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].resulting_root, d.resulting_root, "the other held update is kept");
}
