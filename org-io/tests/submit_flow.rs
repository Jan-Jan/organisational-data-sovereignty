#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Submission (SDD-erzj3m): org-io writes org-node's provisional update to
//! the chain, reads the state the write left, and only then asks org-node
//! to commit and send it; a failed or timed-out write asks for neither and
//! is reported (LLR-qhjp6g, LLR-be3zv9, moved from the app's
//! `tests/submit_flow.rs` with their annotations unchanged, ruling A), and
//! so does a read-back that fails after a write that executed (LLR-9r2bxd).
//! The chain is a substitute here (`test_support::FakeChain`): its writes
//! move the slots its reads answer from.
//!
//! *S4 task T12, 2026-10-13:* `submit_commit_send` names no recipient (the
//! commit names every one, and each is sent to and reported) and takes
//! org-io's `PeerAddr` (`at`). The app's two recipient tests moved here at
//! the end of the file.

use std::sync::atomic::Ordering;
use std::time::Duration;

use org_node::wire::WireMessage;
use org_node::{Epoch, MemberId};
use org_io::test_support::FakeChain;
use rand::rngs::OsRng;

mod common;
use common::handles::{at, bind_seed, dead_addr, handle, joiner, joiner_of, persona, receiving_addr, NET};

// verifies: LLR-qhjp6g, LLR-be3zv9
#[tokio::test]
async fn founding_writes_the_chain_then_commits() {
    let chain = FakeChain::new();
    let a = handle("submit-found", "a", &chain);
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let rec = a.node_for_test().list_orgs().into_iter().find(|o| o.org_id == org).cloned().unwrap();
    assert_eq!((rec.epoch, rec.last_seq.get()), (Epoch::new(1), 1));
    assert!(rec.proxy_account.is_some());
    assert_eq!(rec.org_private_key_for_test().x25519_keypair().org_public_key().unwrap(), rec.org_pub_key, "the founder holds the Organisation's key pair");
}

// verifies: LLR-qhjp6g
#[tokio::test]
async fn a_failed_genesis_write_commits_nothing_and_reports_the_failure() {
    let chain = FakeChain::new();
    chain.write_failing.store(true, Ordering::SeqCst);
    let a = handle("submit-found-fails", "a", &chain);
    let pid = persona(&a, "alice").await;
    let err = a.found_organisation(&mut OsRng, &pid).await.unwrap_err();
    assert!(err.contains("node unreachable"), "the failure is reported: {err}");
    assert!(a.node_for_test().list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.node_for_test().genesis_provisional_updates(&pid).len(), 1, "the update is kept for a retry");
}

// verifies: LLR-qhjp6g
#[tokio::test(flavor = "multi_thread")]
async fn an_admission_is_written_then_committed_then_sent() {
    let chain = FakeChain::new();
    let a = handle("submit-admit", "a", &chain);
    let pid_a = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid_a).await.unwrap();
    let b = handle("submit-admit", "b", &FakeChain::sharing_slots_with(&chain));
    let pid_b = persona(&b, "bob").await;
    b.expect_admission(&mut OsRng, &pid_b, org).await.unwrap();
    // *S4 task T10, 2026-10-13:* A's endpoint is bound from its Persona's
    // keychain item on its first send (org-io's endpoints).
    let bob = joiner_of(&b, &pid_b);
    let addr_b = receiving_addr(&b).await;
    let task = tokio::spawn(async move {
        let received = tokio::time::timeout(NET, b.next_received(&mut OsRng)).await.unwrap();
        (b, received)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let out = a.submit_commit_send(&mut OsRng, &update, at(addr_b)).await.unwrap();
    assert_eq!(out.epoch, Epoch::new(2));
    let (b, received) = task.await.unwrap();
    let org_io::Received::Committed(received) = received.unwrap() else { panic!("a committed admission") };
    assert_eq!(received.epoch, Epoch::new(2));
    // A device holding no proxy account cannot submit: B was admitted, it
    // did not found the Organisation, so its record holds none.
    let carol = joiner(0x71, "carol");
    let update = b.admit_member(&mut OsRng, org, &carol).await.unwrap();
    let err = b.submit_commit_send(&mut OsRng, &update, None).await.unwrap_err();
    assert!(err.contains("proxy account"), "{err}");
    assert_eq!(b.node_for_test().list_orgs()[0].epoch, Epoch::new(2), "nothing committed on B");
}

// verifies: LLR-qhjp6g
#[tokio::test]
async fn a_failed_update_write_neither_commits_nor_sends() {
    let chain = FakeChain::new();
    let a = handle("submit-update-fails", "a", &chain);
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x61, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.write_failing.store(true, Ordering::SeqCst);
    let err = a.submit_commit_send(&mut OsRng, &update, None).await.unwrap_err();
    assert!(err.contains("node unreachable"), "{err}");
    assert_eq!(a.node_for_test().list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert_eq!(a.node_for_test().provisional_updates(org).len(), 1, "the update is kept");
    assert_eq!(a.bound_endpoints_for_test().await, 0,"nothing sent: no endpoint was bound");
}

// verifies: LLR-be3zv9
#[tokio::test(start_paused = true)]
async fn a_submission_that_never_finishes_times_out_and_nothing_is_committed() {
    let chain = FakeChain::new();
    chain.write_hanging.store(true, Ordering::SeqCst);
    let a = handle("submit-never", "a", &chain);
    let pid = persona(&a, "alice").await;
    let err = a.found_organisation(&mut OsRng, &pid).await.unwrap_err();
    // A write that timed out may have executed: its outcome is unknown, not
    // failed (review round 1, finding 10); it is still not committed.
    assert!(err.contains("timed out") && err.contains("outcome unknown"), "{err}");
    assert!(!err.contains("failed"), "{err}");
    assert!(a.node_for_test().list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.node_for_test().genesis_provisional_updates(&pid).len(), 1, "the update is kept");
    // The update path is bounded the same way.
    chain.write_hanging.store(false, Ordering::SeqCst);
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    chain.write_hanging.store(true, Ordering::SeqCst);
    let bob = joiner(0x61, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let err = a.submit_commit_send(&mut OsRng, &update, None).await.unwrap_err();
    assert!(err.contains("timed out") && err.contains("outcome unknown"), "{err}");
    assert!(!err.contains("failed"), "{err}");
    assert_eq!(a.node_for_test().list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert_eq!(a.bound_endpoints_for_test().await, 0,"nothing sent");
}

// LLR-9r2bxd, normal case, on both operations: one read between the write
// and the commit, and the commit takes the state the write left.
// verifies: LLR-9r2bxd
#[tokio::test(flavor = "multi_thread")]
async fn each_submission_reads_the_chain_once_between_the_write_and_the_commit() {
    let chain = FakeChain::new();
    let a = handle("submit-read-back", "a", &chain);
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    assert_eq!(chain.reads(), 1, "one read between the genesis write and the commit");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(1));
    let bob = joiner(0x61, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    // *S4 task T10, 2026-10-13:* A's endpoint is bound from its Persona's
    // keychain item on its first send (org-io's endpoints).
    // *S4 task T12, 2026-10-13:* the sink is Bob's Device (`joiner(0x61,
    // ..)`'s is the seed `[0x62; 32]`'s): a Loopback send goes only to a
    // recipient's address.
    let (sink_addr, sink) = common::handles::spawn_recv_one([0x62; 32]).await;
    let outcome = a.submit_commit_send(&mut OsRng, &update, at(sink_addr)).await.unwrap();
    sink.await.unwrap();
    assert_eq!(chain.reads(), 2, "one read between the update write and the commit");
    let state = chain.slots().get(&org).unwrap();
    assert_eq!((outcome.epoch, outcome.root), (state.epoch, state.root_hash), "committed at the state read back");
}

// LLR-9r2bxd: a write that executed, then a read-back that failed: nothing
// committed, the update kept, both facts reported.
// verifies: LLR-9r2bxd
#[tokio::test]
async fn a_write_whose_read_back_fails_commits_nothing_and_says_the_write_executed() {
    let chain = FakeChain::new();
    let a = handle("submit-read-back-fails", "a", &chain);
    let pid = persona(&a, "alice").await;
    chain.read_failing.store(true, Ordering::SeqCst);
    let err = a.found_organisation(&mut OsRng, &pid).await.unwrap_err();
    assert!(err.contains("write executed") && err.contains("nothing committed"), "{err}");
    assert!(err.contains("node unreachable"), "the read's failure is reported: {err}");
    assert!(a.node_for_test().list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.node_for_test().genesis_provisional_updates(&pid).len(), 1, "the update is kept");
    assert_eq!(chain.geneses(), 1, "the write did execute");
}

// LLR-9r2bxd: a write that executed, then a read-back that finds no state:
// org-node's commit refuses with `OrgNotOnChain` and that refusal is
// returned — nothing committed, nothing sent, the update kept — on both
// operations.
// verifies: LLR-9r2bxd
#[tokio::test(flavor = "multi_thread")]
async fn a_read_back_that_finds_no_state_is_refused_with_org_not_on_chain() {
    let not_on_chain = org_node::OrgNodeError::OrgNotOnChain.to_string();
    let chain = FakeChain::new();
    let a = handle("submit-read-back-absent", "a", &chain);
    let pid = persona(&a, "alice").await;
    chain.state_hidden.store(true, Ordering::SeqCst);
    let err = a.found_organisation(&mut OsRng, &pid).await.unwrap_err();
    assert!(err.contains(&not_on_chain), "the commit's refusal is returned: {err}");
    assert_eq!(chain.reads(), 1, "the write was read back");
    assert_eq!(chain.geneses(), 1, "the write did execute");
    assert!(a.node_for_test().list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.node_for_test().genesis_provisional_updates(&pid).len(), 1, "the update is kept");

    chain.state_hidden.store(false, Ordering::SeqCst);
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x43, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.state_hidden.store(true, Ordering::SeqCst);
    let err = a
        .submit_commit_send(&mut OsRng, &update, at(dead_addr([0x44; 32])))
        .await
        .unwrap_err();
    assert!(err.contains(&not_on_chain), "the commit's refusal is returned: {err}");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(2), "the write did execute");
    assert_eq!(a.node_for_test().list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert_eq!(a.node_for_test().provisional_updates(org).len(), 1, "the update is kept");
    assert_eq!(a.bound_endpoints_for_test().await, 0,"nothing sent: no endpoint was bound to dial the dead address");
}

// LLR-9r2bxd on `submit_commit_send`: the update write executed, the
// read-back failed: nothing committed, nothing sent, the update kept.
// verifies: LLR-9r2bxd
#[tokio::test(flavor = "multi_thread")]
async fn an_update_whose_read_back_fails_commits_and_sends_nothing() {
    let chain = FakeChain::new();
    let a = handle("submit-update-read-back-fails", "a", &chain);
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x41, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.read_failing.store(true, Ordering::SeqCst);
    let err = a
        .submit_commit_send(&mut OsRng, &update, at(dead_addr([0x42; 32])))
        .await
        .unwrap_err();
    assert!(err.contains("write executed") && err.contains("nothing committed"), "{err}");
    assert_eq!(a.node_for_test().list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert_eq!(a.node_for_test().provisional_updates(org).len(), 1, "the update is kept");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(2), "the write did execute");
    assert_eq!(a.bound_endpoints_for_test().await, 0,"nothing sent: no endpoint was bound to dial the dead address");
}

// LLR-9r2bxd (clarified 2026-10-08, task T8b): a read-back that finds no
// state is reported "the same way" as a read-back that fails: the error
// says the write executed and nothing was committed, and carries org-node's
// `OrgNotOnChain` refusal as its cause; the update is kept, nothing sent.
// org-io's submission errors are strings, so the facts are matched as text.
// verifies: LLR-9r2bxd
#[tokio::test(flavor = "multi_thread")]
async fn a_read_back_that_finds_no_state_says_the_write_executed_and_nothing_was_committed() {
    let not_on_chain = org_node::OrgNodeError::OrgNotOnChain.to_string();
    let chain = FakeChain::new();
    let a = handle("submit-read-back-absent-reported", "a", &chain);
    let pid = persona(&a, "alice").await;
    chain.state_hidden.store(true, Ordering::SeqCst);
    let err = a.found_organisation(&mut OsRng, &pid).await.unwrap_err();
    assert!(err.contains("write executed") && err.contains("nothing committed"), "both facts reported: {err}");
    assert!(err.contains(&not_on_chain), "the OrgNotOnChain cause is named: {err}");
    assert_eq!(chain.geneses(), 1, "the write did execute");
    assert!(a.node_for_test().list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.node_for_test().genesis_provisional_updates(&pid).len(), 1, "the update is kept");

    chain.state_hidden.store(false, Ordering::SeqCst);
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x45, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.state_hidden.store(true, Ordering::SeqCst);
    let err = a
        .submit_commit_send(&mut OsRng, &update, at(dead_addr([0x46; 32])))
        .await
        .unwrap_err();
    assert!(
        err.contains("write executed") && err.contains("nothing committed or sent"),
        "both facts reported: {err}"
    );
    assert!(err.contains(&not_on_chain), "the OrgNotOnChain cause is named: {err}");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(2), "the write did execute");
    assert_eq!(a.node_for_test().list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert_eq!(a.node_for_test().provisional_updates(org).len(), 1, "the update is kept");
    assert_eq!(a.bound_endpoints_for_test().await, 0,"nothing sent: no endpoint was bound to dial the dead address");
}

// LLR-9r2bxd (clarified 2026-10-08, task T12c): a commit that refuses, after
// a write that executed and a read-back that found a state, for a reason
// other than `OrgNotOnChain` is reported as that one is: the error says the
// write executed and nothing was committed (or sent), and carries
// org-node's refusal as its cause; the update is kept, nothing sent. The
// read-back answers the slot one epoch past the write (`epoch_ahead`), a
// state whose root and key match the held update but whose epoch org-node
// refuses. org-io's submission errors are strings, so the facts are matched
// as text.
// verifies: LLR-9r2bxd
#[tokio::test(flavor = "multi_thread")]
async fn a_commit_refused_for_another_reason_after_a_write_says_the_write_executed_and_nothing_was_committed() {
    let not_on_chain = org_node::OrgNodeError::OrgNotOnChain.to_string();
    let chain = FakeChain::new();
    let a = handle("submit-other-refusal", "a", &chain);
    let pid = persona(&a, "alice").await;
    chain.epoch_ahead.store(true, Ordering::SeqCst);
    let err = a.found_organisation(&mut OsRng, &pid).await.unwrap_err();
    let seq_not_epoch = org_node::OrgNodeError::SeqNotEpoch { seq: 1, epoch: 2 }.to_string();
    assert!(err.contains("write executed") && err.contains("nothing committed"), "both facts reported: {err}");
    assert!(err.ends_with(&seq_not_epoch), "org-node's refusal is the cause: {err}");
    assert!(!err.contains(&not_on_chain), "the cause is the refusal itself: {err}");
    assert_eq!(chain.geneses(), 1, "the write did execute");
    assert!(a.node_for_test().list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.node_for_test().genesis_provisional_updates(&pid).len(), 1, "the update is kept");

    chain.epoch_ahead.store(false, Ordering::SeqCst);
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x47, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    chain.epoch_ahead.store(true, Ordering::SeqCst);
    let err = a
        .submit_commit_send(&mut OsRng, &update, at(dead_addr([0x48; 32])))
        .await
        .unwrap_err();
    let refusal = a
        .node_for_test()
        .commit_update(&mut OsRng, org, chain.slots().get(&org).map(|state| org_node::OrgState {
            epoch: Epoch::new(state.epoch.get() + 1),
            ..state
        }), |_| Vec::new())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("write executed") && err.contains("nothing committed or sent"),
        "both facts reported: {err}"
    );
    assert!(err.ends_with(&refusal), "org-node's refusal is the cause: {err}");
    assert!(!err.contains(&not_on_chain), "the cause is the refusal itself: {err}");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(2), "the write did execute");
    assert_eq!(a.node_for_test().list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert_eq!(a.node_for_test().provisional_updates(org).len(), 1, "the update is kept");
    assert_eq!(a.bound_endpoints_for_test().await, 0,"nothing sent: no endpoint was bound to dial the dead address");
}

// A handle with no chain configured (`OrgIo::not_configured`) refuses every
// submission at the write, so nothing is read back, committed or sent, and
// the update is kept (LLR-qhjp6g, clarified 2026-10-08, task T12b). The
// Organisation the update needs is founded first over a substitute chain,
// then the same store is opened again behind an unconfigured handle.
// verifies: LLR-qhjp6g
#[tokio::test]
async fn a_handle_with_no_chain_refuses_every_submission_at_the_write() {
    let not_configured = "chain not configured";
    let tag = "submit-not-configured";
    let chain = FakeChain::new();
    let founder = handle(tag, "a", &chain);
    let pid = persona(&founder, "alice").await;
    let org = founder.found_organisation(&mut OsRng, &pid).await.unwrap();
    let reopened = common::handles::reopened_service(tag, "a", &founder);
    drop(founder);
    let a = org_io::OrgIo::not_configured(reopened);

    // A second Persona, unbound, founds: refused at the genesis write.
    let unbound = persona(&a, "carol").await;
    let err = a.found_organisation(&mut OsRng, &unbound).await.unwrap_err();
    assert!(err.starts_with("chain write failed; nothing committed") && err.contains(not_configured), "{err}");
    assert_eq!(a.node_for_test().list_orgs().len(), 1, "nothing committed");
    assert_eq!(a.node_for_test().genesis_provisional_updates(&unbound).len(), 1, "the genesis update is kept");

    let bob = joiner(0x49, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let err = a.submit_commit_send(&mut OsRng, &update, None).await.unwrap_err();
    assert!(err.starts_with("chain write failed; nothing committed or sent") && err.contains(not_configured), "{err}");
    assert_eq!(a.node_for_test().list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert_eq!(a.node_for_test().provisional_updates(org).len(), 1, "the update is kept");
    assert_eq!(a.bound_endpoints_for_test().await, 0,"nothing sent");
    assert_eq!(chain.geneses(), 1, "no write reached any chain");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(1));
}

// Review round 3, finding 1: the handle's read surface (`NodeView`; since
// S4 task T11, 2026-10-13, `OrgIo`'s own query methods) gives the record's
// public fields, and only those: the summaries match what org-node holds
// field for field.
// verifies: LLR-3zdw8v
#[tokio::test]
async fn the_view_summarises_the_public_fields_org_node_holds() {
    let chain = FakeChain::new();
    let a = handle("submit-view", "a", &chain);
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let record = a.node_for_test().list_orgs()[0].clone();
    let summary = a.organisation(org).await.unwrap();
    assert_eq!(
        (summary.org_id, summary.root_hash, summary.org_pub_key, summary.epoch, &summary.members),
        (record.org_id, record.root_hash, record.org_pub_key, record.epoch, &record.trie_members)
    );
    assert_eq!(a.organisations().await, vec![summary]);
    assert_eq!(a.organisation(org_node::OrgId::new([9; 20])).await, None);
    let persona_record = a.node_for_test().list_personas()[0].clone();
    let persona_summary = a.personas().await.remove(0);
    assert_eq!(
        (persona_summary.persona_id, persona_summary.org_id, persona_summary.member_id, persona_summary.status),
        (persona_record.persona_id, persona_record.org_id, persona_record.member_id, persona_record.status)
    );
}

// Review round 3, finding 2: `submit_commit_send` writes only an update
// org-node holds, built on the record's current root, and refuses anything
// else before any write: a caller-altered update (another update's root
// under this one's key), and a stale one, built before another update
// committed (which could put a revoked Member's root back on chain).
// verifies: LLR-qhjp6g
#[tokio::test(flavor = "multi_thread")]
async fn an_update_org_node_does_not_hold_on_the_records_root_is_refused_before_any_write() {
    let chain = FakeChain::new();
    let a = handle("submit-not-held", "a", &chain);
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x51, "bob");
    let carol = joiner(0x53, "carol");
    let stale = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let current = a.admit_member(&mut OsRng, org, &carol).await.unwrap();

    let mut altered = current;
    altered.resulting_root = stale.resulting_root;
    let err = a.submit_commit_send(&mut OsRng, &altered, None).await.unwrap_err();
    assert!(err.starts_with("refused before any write"), "{err}");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(1), "nothing written");

    // *S4 task T10, 2026-10-13:* A's endpoint is bound from its Persona's
    // keychain item on its first send (org-io's endpoints).
    // *S4 task T12, 2026-10-13:* the sink is Carol's Device (`joiner(0x53,
    // ..)`'s is the seed `[0x54; 32]`'s): a Loopback send goes only to a
    // recipient's address.
    let (sink_addr, sink) = common::handles::spawn_recv_one([0x54; 32]).await;
    a.submit_commit_send(&mut OsRng, &current, at(sink_addr)).await.unwrap();
    sink.await.unwrap();
    let written = chain.slots().get(&org).unwrap();
    let err = a.submit_commit_send(&mut OsRng, &stale, None).await.unwrap_err();
    assert!(err.starts_with("refused before any write"), "{err}");
    assert_eq!(chain.slots().get(&org).unwrap(), written, "nothing written");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(2));
}

// Review round 3, finding 3: an update for an Organisation this node's
// store does not hold is refused before any write as not held
// (`OrgNotHeld`), not as missing from the chain.
// verifies: LLR-qhjp6g
#[tokio::test]
async fn an_update_for_an_organisation_the_store_does_not_hold_is_refused_as_not_held() {
    let chain = FakeChain::new();
    let a = handle("submit-org-not-held", "a", &chain);
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x55, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    let b = handle("submit-org-not-held", "b", &FakeChain::sharing_slots_with(&chain));
    let err = b.submit_commit_send(&mut OsRng, &update, None).await.unwrap_err();
    assert_eq!(err, org_node::OrgNodeError::OrgNotHeld { org_id: org }.to_string());
    assert_eq!(chain.slots().get(&org).unwrap().epoch, Epoch::new(1), "nothing written");
}

// *Moved 2026-10-13 from the app's `tests/submit_flow.rs` (S4, task T12),
// where it verified the retired one-send-per-command item (retired in S4
// task T12, superseded by the fan-out, LLR-znhmh2; see the S4 verification
// record). It ran with no `verifies:` line until 2026-10-15 (below); the
// fan-out's own tests are `tests/send_fanout.rs`.* Rewritten
// onto org-io's handle: the app's `revoke_and_send` chose one recipient;
// org-io sends to every recipient the commit names, which in Loopback is
// the one at the address held. The admission reaches the joiner's Device,
// which the record lists, as Organisation information; the revocation
// reaches the removed Device, which it no longer lists, as a revocation;
// nothing else is sent.
// *Pointed at the item it checks 2026-10-15 (S4 TEST review round 2,
// finding-2; the upgraded guardrails rule on retired IDs):* the retired
// item's definition is deleted, so only org-io's item is cited. Not the
// fan-out (LLR-znhmh2, Networked mode): the handle here is in Loopback
// mode, and what the test checks is LLR-r9sh6k's last clause — the send
// goes only to the recipient at the address held, and nothing else is sent.
// Which message each recipient gets is org-node's (REQ-jxyh9c), checked
// here only as the means of telling the two sends apart.
// verifies: LLR-r9sh6k
#[tokio::test(flavor = "multi_thread")]
async fn each_command_sends_once_to_one_device_and_the_kind_follows_the_record() {
    let chain = FakeChain::new();
    let a = handle("submit-kinds", "a", &chain);
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x61, "bob");
    let sink = bind_seed([0x62; 32]).await;

    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    a.submit_commit_send(&mut OsRng, &update, at(addr)).await.unwrap();
    let (sink, got) = task.await.unwrap();
    assert!(matches!(got.unwrap().1, WireMessage::OrgInformation { .. }), "the joiner's Device is listed");

    let bob_id = a.node_for_test().list_orgs()[0].trie_members.iter().find(|m| m.member_key == bob.member_key).unwrap().id;
    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.revoke_member(&mut OsRng, org, bob_id).await.unwrap();
    a.submit_commit_send(&mut OsRng, &update, at(addr)).await.unwrap();
    let (sink, got) = task.await.unwrap();
    assert!(matches!(got.unwrap().1, WireMessage::Revocation(_)), "the removed Device is not listed");
    assert!(
        tokio::time::timeout(Duration::from_secs(2), sink.recv_one()).await.is_err(),
        "nothing else is sent"
    );
}

// *Moved 2026-10-13 from the app's `tests/submit_flow.rs` (S4, task T12),
// where it verified the retired one-send-per-command item (as above, it
// ran with no `verifies:` line until 2026-10-15, below; the fan-out is
// LLR-znhmh2).* Abnormal input: a
// revocation of a member id the record does not hold is refused before
// anything is built, written, committed or sent: no provisional update
// kept, the chain and the record at the epoch they were, and nothing at the
// sink. Rewritten onto org-io's handle: the refusal is org-node's, from
// `revoke_member`, where the app's command refused first with its own text,
// which is gone.
//
// *Annotated 2026-10-15 (S4, owner ruling of that day):* it verifies
// LLR-nqdr97, minted for it; the abnormal case of that item.
// *Tightened 2026-10-15 (S4 TEST review round 3, notes):* the refusal is
// asserted to be org-node's for a Member id the record does not hold
// (`OrgNodeError::Trie(OrgMembersError::IdNotFound)`, as LLR-nqdr97's
// clarification of that day states), not only an error. Mutation check,
// that day (removed): `OrgIo::revoke_member` made to return
// `OrgNodeError::OrgNotHeld` for the Organisation in place of org-node's
// refusal failed this test at the refusal's assertion.
// verifies: LLR-nqdr97
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing() {
    let chain = FakeChain::new();
    let a = handle("submit-revoke-unknown", "a", &chain);
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x61, "bob");
    let sink = bind_seed([0x62; 32]).await;
    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    a.submit_commit_send(&mut OsRng, &update, at(addr)).await.unwrap();
    let (sink, _) = task.await.unwrap();

    let held: Vec<MemberId> = a.node_for_test().list_orgs()[0].trie_members.iter().map(|m| m.id).collect();
    let unknown = MemberId::new([0xee; 32]);
    assert!(!held.contains(&unknown), "the id is not one the record holds");
    let chain_epoch = chain.slots().get(&org).unwrap().epoch;

    let refused = a.revoke_member(&mut OsRng, org, unknown).await;
    assert!(
        matches!(
            refused,
            Err(org_io::OperationError::Node(org_node::OrgNodeError::Trie(org_node::OrgMembersError::IdNotFound)))
        ),
        "refused by org-node as a Member id the record does not hold, not {refused:?}"
    );
    assert_eq!(a.node_for_test().list_orgs()[0].epoch, Epoch::new(2), "nothing committed");
    assert!(a.node_for_test().provisional_updates(org).is_empty(), "no provisional update kept");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, chain_epoch, "nothing written to the chain");
    assert!(
        tokio::time::timeout(Duration::from_secs(2), sink.recv_one()).await.is_err(),
        "nothing sent"
    );
}
