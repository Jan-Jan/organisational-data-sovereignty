#![cfg(feature = "test-support")]
//! The recipient each command chooses (LLR-q225ws); the submission itself is
//! org-io's. The chain is a substitute here (org-io's
//! `test_support::FakeChain`) under the org-io handle: its writes move the
//! slots its reads answer from.

use std::time::Duration;

use ods_poc_lib::commands::revoke_and_send;
use org_io::node::service::OrgService;
use org_io::node::store::PersonaStore;
use org_io::node::transport::endpoint::OrgEndpoint;
use org_io::node::transport::wire::WireMessage;
use org_io::node::{DeviceSeed, Epoch, Handle, Joiner, MemberId, MemberSeed, Name, Surname};
use org_io::test_support::FakeChain;
use org_io::OrgIo;
use rand::rngs::OsRng;

fn handle(tag: &str, chain: &FakeChain) -> OrgIo {
    let dir = std::env::temp_dir().join(format!("ods-app-submit-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    OrgIo::for_test(OrgService::new(PersonaStore::open(dir.join("store.bin"), "pw").unwrap()), chain)
}

fn persona(io: &mut OrgIo, handle: &str) -> org_io::node::PersonaId {
    io.node_mut()
        .create_persona(&mut OsRng, Handle::parse(handle).unwrap(), Name::parse("Test").unwrap(), Surname::parse("User").unwrap())
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

// LLR-q225ws: each command asks org-node for exactly one send, to exactly one
// DevicePublicKey, passing no key, secret or invite identifier; org-node
// chooses the kind from its committed record. The admission goes to the
// joiner's Device, which the record lists: Organisation information. The
// revocation goes to the removed Member's first Device, which it no longer
// lists: a revocation. Nothing else is sent.
// verifies: LLR-q225ws
#[tokio::test(flavor = "multi_thread")]
async fn each_command_sends_once_to_one_device_and_the_kind_follows_the_record() {
    let chain = FakeChain::new();
    let mut a = handle("kinds", &chain);
    let pid = persona(&mut a, "alice");
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x61, "bob");
    let sink = OrgEndpoint::bind(&DeviceSeed::from([0x62; 32]).signing_keypair()).await.unwrap();

    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    a.submit_commit_send(&mut OsRng, &update, bob.device_key, Some(addr)).await.unwrap();
    let (sink, got) = task.await.unwrap();
    assert!(matches!(got.unwrap().1, WireMessage::OrgInformation { .. }), "the joiner's Device is listed");

    let bob_id = a.node_for_test().list_orgs()[0].trie_members.iter().find(|m| m.member_key == bob.member_key).unwrap().id;
    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    revoke_and_send(&mut a, &mut OsRng, org, bob_id, Some(addr)).await.unwrap();
    let (sink, got) = task.await.unwrap();
    assert!(matches!(got.unwrap().1, WireMessage::Revocation(_)), "the removed Device is not listed");
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
    let chain = FakeChain::new();
    let mut a = handle("revoke-unknown", &chain);
    let pid = persona(&mut a, "alice");
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x61, "bob");
    let sink = OrgEndpoint::bind(&DeviceSeed::from([0x62; 32]).signing_keypair()).await.unwrap();
    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.node_mut().admit_member(&mut OsRng, org, &bob).unwrap();
    a.submit_commit_send(&mut OsRng, &update, bob.device_key, Some(addr)).await.unwrap();
    let (sink, _) = task.await.unwrap();

    let held: Vec<MemberId> = a.node_for_test().list_orgs()[0].trie_members.iter().map(|m| m.id).collect();
    let unknown = MemberId::new([0xee; 32]);
    assert!(!held.contains(&unknown), "the id is not one the record holds");
    let chain_epoch = chain.slots().get(&org).unwrap().epoch;

    let addr = sink.inner().addr();
    let err = revoke_and_send(&mut a, &mut OsRng, org, unknown, Some(addr)).await.unwrap_err();
    assert!(err.contains("names no member"), "{err}");
    assert_eq!(a.node_for_test().list_orgs()[0].epoch, Epoch::new(2), "nothing committed");
    assert!(a.node_for_test().provisional_updates(org).is_empty(), "no provisional update kept");
    assert_eq!(chain.slots().get(&org).unwrap().epoch, chain_epoch, "nothing written to the chain");
    assert!(
        tokio::time::timeout(Duration::from_secs(2), sink.recv_one()).await.is_err(),
        "nothing sent"
    );
}
