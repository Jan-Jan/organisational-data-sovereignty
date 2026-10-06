#![cfg(feature = "test-support")]
//! REQ-nfr3n2: the app writes org-node's provisional update to the chain
//! through on-chain-client, and only once that write has executed asks
//! org-node to commit and send it; a failed or timed-out write asks for
//! neither and is reported (LLR-qhjp6g, LLR-be3zv9). The chain write is a
//! substitute here (`support::FakeWriter`) over org-node's `MockChainOps`,
//! which the service reads.

use std::sync::atomic::Ordering;
use std::time::Duration;

use ods_poc_lib::commands::revoke_and_send;
use ods_poc_lib::submit::{found_organisation, submit_commit_send};
use org_node::service::{MockChainOps, OrgService};
use org_node::store::PersonaStore;
use org_node::transport::endpoint::OrgEndpoint;
use org_node::transport::wire::WireMessage;
use org_node::{DeviceSeed, Epoch, Handle, Joiner, MemberId, MemberSeed, Name, Surname};
use rand::rngs::OsRng;

mod support;
use support::FakeWriter;

fn service(tag: &str, chain: &MockChainOps) -> OrgService {
    let dir = std::env::temp_dir().join(format!("ods-app-submit-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    OrgService::new(PersonaStore::open(dir.join("store.bin"), "pw").unwrap(), Box::new(chain.clone()))
}

fn persona(svc: &mut OrgService, handle: &str) -> org_node::PersonaId {
    svc.create_persona(&mut OsRng, Handle::parse(handle).unwrap(), Name::parse("Test").unwrap(), Surname::parse("User").unwrap())
        .unwrap()
}

fn joiner(seed: u8, handle: &str) -> Joiner {
    Joiner {
        handle: Handle::parse(handle).unwrap(),
        name: Name::parse("Test").unwrap(),
        surname: Surname::parse("Joiner").unwrap(),
        member_key: MemberSeed::from([seed; 32]).x25519_keypair().member_key().unwrap(),
        device_key: DeviceSeed::from([seed + 1; 32]).signing_keypair().device_key().unwrap(),
    }
}

// verifies: LLR-qhjp6g, LLR-be3zv9
#[tokio::test]
async fn founding_writes_the_chain_then_commits() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let mut a = service("found", &chain);
    let pid = persona(&mut a, "alice");
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap();
    let rec = a.list_orgs().iter().find(|o| o.org_id == org).cloned().unwrap();
    assert_eq!((rec.epoch, rec.last_seq.get()), (Epoch::new(1), 1));
    assert!(rec.proxy_account.is_some());
    assert_eq!(rec.org_private_key.x25519_keypair().org_public_key().unwrap(), rec.org_pub_key, "the founder holds the Organisation's key pair");
}

// verifies: LLR-qhjp6g
#[tokio::test]
async fn a_failed_genesis_write_commits_nothing_and_reports_the_failure() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    writer.failing.store(true, Ordering::SeqCst);
    let mut a = service("found-fails", &chain);
    let pid = persona(&mut a, "alice");
    let err = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap_err();
    assert!(err.contains("node unreachable"), "the failure is reported: {err}");
    assert!(a.list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.genesis_provisional_updates(&pid).len(), 1, "the update is kept for a retry");
}

// verifies: LLR-qhjp6g
#[tokio::test(flavor = "multi_thread")]
async fn an_admission_is_written_then_committed_then_sent() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let mut a = service("admit-a", &chain);
    let pid_a = persona(&mut a, "alice");
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid_a).await.unwrap();
    let mut b = service("admit-b", &chain);
    let pid_b = persona(&mut b, "bob");
    b.expect_admission(&mut OsRng, org).unwrap();
    let rec_b = b.list_personas()[0].clone();
    let (member_key, device_key) = b.persona_public_keys(&pid_b).unwrap();
    let bob = Joiner { handle: rec_b.handle, name: rec_b.name, surname: rec_b.surname, member_key, device_key };
    let ep_b = OrgEndpoint::bind(&rec_b.device_seed.signing_keypair()).await.unwrap();
    let addr_b = ep_b.inner().addr();
    let b = b.with_endpoint(ep_b);
    let task = tokio::spawn(async move {
        let mut b = b;
        let r = tokio::time::timeout(Duration::from_secs(30), b.receive_and_verify(&mut OsRng)).await.unwrap();
        (b, r)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.admit_member(&mut OsRng, org, &bob).unwrap();
    let out = submit_commit_send(&mut a, &writer, &mut OsRng, &update, device_key, Some(addr_b))
        .await
        .unwrap();
    assert_eq!(out.epoch, Epoch::new(2));
    let (b, received) = task.await.unwrap();
    assert_eq!(received.unwrap().epoch, Epoch::new(2));
    // A device holding no proxy account cannot submit: B was admitted, it
    // did not found the Organisation, so its record holds none.
    let mut b = b;
    let carol = joiner(0x71, "carol");
    let update = b.admit_member(&mut OsRng, org, &carol).unwrap();
    let err = submit_commit_send(&mut b, &writer, &mut OsRng, &update, carol.device_key, None)
        .await
        .unwrap_err();
    assert!(err.contains("proxy account"), "{err}");
    assert_eq!(b.list_orgs()[0].epoch, Epoch::new(2), "nothing committed on B");
}

// verifies: LLR-qhjp6g
#[tokio::test]
async fn a_failed_update_write_neither_commits_nor_sends() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let mut a = service("update-fails", &chain);
    let pid = persona(&mut a, "alice");
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x61, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).unwrap();
    writer.failing.store(true, Ordering::SeqCst);
    let err = submit_commit_send(&mut a, &writer, &mut OsRng, &update, bob.device_key, None)
        .await
        .unwrap_err();
    assert!(err.contains("node unreachable"), "{err}");
    assert_eq!(a.list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert_eq!(a.provisional_updates(org).len(), 1, "the update is kept");
    assert!(a.endpoint().is_none(), "nothing sent: no endpoint was bound");
}

// verifies: LLR-be3zv9
#[tokio::test(start_paused = true)]
async fn a_submission_that_never_finishes_times_out_and_nothing_is_committed() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::never(&chain);
    let mut a = service("never", &chain);
    let pid = persona(&mut a, "alice");
    let err = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap_err();
    assert!(err.contains("timed out"), "{err}");
    assert!(a.list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.genesis_provisional_updates(&pid).len(), 1, "the update is kept");
    // The update path is bounded the same way.
    let writer = FakeWriter::over(&chain);
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap();
    writer.hanging.store(true, Ordering::SeqCst);
    let bob = joiner(0x61, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).unwrap();
    let err = submit_commit_send(&mut a, &writer, &mut OsRng, &update, bob.device_key, None)
        .await
        .unwrap_err();
    assert!(err.contains("timed out"), "{err}");
    assert_eq!(a.list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert!(a.endpoint().is_none(), "nothing sent");
}

// LLR-q225ws: each command asks org-node for exactly one send, to exactly one
// DevicePublicKey, passing no key, secret or invite identifier; org-node
// chooses the kind from its committed record. The admission goes to the
// joiner's Device, which the record lists: Organisation information. The
// revocation goes to the removed Member's first Device, which it no longer
// lists: a revocation. Nothing else is sent.
// verifies: LLR-q225ws
#[tokio::test(flavor = "multi_thread")]
async fn each_command_sends_once_to_one_device_and_the_kind_follows_the_record() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let mut a = service("kinds", &chain);
    let pid = persona(&mut a, "alice");
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x61, "bob");
    let sink = OrgEndpoint::bind(&DeviceSeed::from([0x62; 32]).signing_keypair()).await.unwrap();

    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.admit_member(&mut OsRng, org, &bob).unwrap();
    submit_commit_send(&mut a, &writer, &mut OsRng, &update, bob.device_key, Some(addr)).await.unwrap();
    let (sink, got) = task.await.unwrap();
    assert!(matches!(got.unwrap().1, WireMessage::OrgInformation { .. }), "the joiner's Device is listed");

    let bob_id = a.list_orgs()[0].trie_members.iter().find(|m| m.member_key == bob.member_key).unwrap().id;
    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    revoke_and_send(&mut a, &writer, &mut OsRng, org, bob_id, Some(addr)).await.unwrap();
    let (sink, got) = task.await.unwrap();
    assert!(matches!(got.unwrap().1, WireMessage::Revocation { .. }), "the removed Device is not listed");
    assert!(
        tokio::time::timeout(Duration::from_secs(2), sink.recv_one()).await.is_err(),
        "nothing else is sent"
    );
}

// LLR-q225ws, abnormal input: a revocation of a member id the record does not
// hold is refused before anything is built, written, committed or sent: no
// provisional update kept, the chain and the record at the epoch they were,
// and nothing at the sink.
// verifies: LLR-q225ws
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let mut a = service("revoke-unknown", &chain);
    let pid = persona(&mut a, "alice");
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x61, "bob");
    let sink = OrgEndpoint::bind(&DeviceSeed::from([0x62; 32]).signing_keypair()).await.unwrap();
    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.admit_member(&mut OsRng, org, &bob).unwrap();
    submit_commit_send(&mut a, &writer, &mut OsRng, &update, bob.device_key, Some(addr)).await.unwrap();
    let (sink, _) = task.await.unwrap();

    let held: Vec<MemberId> = a.list_orgs()[0].trie_members.iter().map(|m| m.id).collect();
    let unknown = MemberId::new([0xee; 32]);
    assert!(!held.contains(&unknown), "the id is not one the record holds");
    let chain_epoch = chain.get(&org).unwrap().epoch;

    let addr = sink.inner().addr();
    let err = revoke_and_send(&mut a, &writer, &mut OsRng, org, unknown, Some(addr)).await.unwrap_err();
    assert!(err.contains("names no member"), "{err}");
    assert_eq!(a.list_orgs()[0].epoch, Epoch::new(2), "nothing committed");
    assert!(a.provisional_updates(org).is_empty(), "no provisional update kept");
    assert_eq!(chain.get(&org).unwrap().epoch, chain_epoch, "nothing written to the chain");
    assert!(
        tokio::time::timeout(Duration::from_secs(2), sink.recv_one()).await.is_err(),
        "nothing sent"
    );
}
