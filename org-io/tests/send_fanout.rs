#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The fan-out of a committed update (SDD-v2bq89; S4, task T12): every
//! recipient org-node's commit names is attempted, in its order, and
//! reported per Device; a failed send leaves the commit in place
//! (LLR-znhmh2). In Loopback mode, which has no address book, the one
//! address the caller holds is sent to only when its endpoint identity is a
//! recipient's Device key, and nothing is sent without one (LLR-r9sh6k).
//! The chain is a substitute (`test_support::FakeChain`); the receivers are
//! endpoints bound from keychain items of the installed in-memory keychain.

use std::time::Duration;

use iroh::address_lookup::MemoryLookup;
use org_node::{DevicePublicKey, Epoch, Joiner, OrgId};
use org_io::peer_addr::PeerAddr;
use org_io::send::SendError;
use org_io::test_support::{FakeChain, MemoryKeychain};
use org_io::transport::{BindPlan, OrgEndpoint};
use org_io::OrgIo;
use rand::rngs::OsRng;
use tokio::sync::mpsc;

mod common;
use common::handles::{bind_seed, dead_addr, forwarding, handle, joiner, keep_device_seed, persona, reopened_service, NET};

/// How long a receiver waits before it concludes nothing was sent.
const QUIET: Duration = Duration::from_secs(2);

/// The keychain seed of `joiner(seed, ..)`'s Device (`device_key(seed + 1)`).
fn device_seed_of(joiner_seed: u8) -> [u8; 32] {
    [joiner_seed + 1; 32]
}

/// A founded Organisation on a fresh handle of test `tag`.
async fn founded(tag: &str, chain: &FakeChain) -> (OrgIo, OrgId) {
    let a = handle(tag, "a", chain);
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    (a, org)
}

/// Admit `joiner` into `org` and submit it, Loopback, at `peer_addr`.
async fn admit(a: &OrgIo, org: OrgId, joiner: &Joiner, peer_addr: Option<PeerAddr>) -> org_io::Submitted {
    let update = a.admit_member(&mut OsRng, org, joiner).await.unwrap();
    a.submit_commit_send(&mut OsRng, &update, peer_addr).await.unwrap()
}

/// The Devices of `org`'s committed record other than the sender's, in the
/// record's order: the order the commit builds its bodies in.
async fn recipients_in_record_order(a: &OrgIo, org: OrgId, sender: DevicePublicKey) -> Vec<DevicePublicKey> {
    let record = a.organisation(org).await.unwrap();
    record.members.iter().flat_map(|member| member.device_keys.iter().copied()).filter(|device| *device != sender).collect()
}

/// A Loopback receiver for `seed`'s Device that forwards every body it
/// receives, with its authenticated sender.
async fn listening(seed: [u8; 32]) -> (iroh::EndpointAddr, mpsc::UnboundedReceiver<(DevicePublicKey, Vec<u8>)>) {
    forwarding(bind_seed(seed).await)
}

async fn nothing_arrives(received: &mut mpsc::UnboundedReceiver<(DevicePublicKey, Vec<u8>)>) -> bool {
    tokio::time::timeout(QUIET, received.recv()).await.is_err()
}

// Networked (over the in-process relay, dialled by endpoint id): three
// recipients, one of which never bound an endpoint. Every one is attempted
// and reported, in the commit's order: two delivered, one failed.
// verifies: LLR-znhmh2, PR-3ue4va
#[tokio::test(flavor = "multi_thread")]
async fn networked_every_recipient_is_attempted_and_reported_in_order() {
    MemoryKeychain::shared().install();
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server().await.unwrap();
    let lookup = MemoryLookup::new();
    let plan = BindPlan::Relay(relay_map, lookup.clone());
    let chain = FakeChain::new();
    let a = handle("fanout-networked", "a", &chain).with_bind_plan_for_test(plan.clone());
    let pid = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid).await.unwrap();
    let sender = a.endpoint_for_test(&pid).await.unwrap();
    tokio::time::timeout(NET, sender.inner().online()).await.unwrap();
    lookup.add_endpoint_info(sender.inner().addr());
    let (_, sender_device) = a.persona_public_keys(&pid).await.unwrap();

    let mut receivers = vec![];
    for joiner_seed in [0x61, 0x63] {
        let device = keep_device_seed(device_seed_of(joiner_seed));
        let endpoint = OrgEndpoint::bind_from_keychain(&device, plan.clone()).await.unwrap();
        tokio::time::timeout(NET, endpoint.inner().online()).await.unwrap();
        lookup.add_endpoint_info(endpoint.inner().addr());
        receivers.push((device, forwarding(endpoint).1));
    }
    let (bob, carol, dave) = (joiner(0x61, "bob"), joiner(0x63, "carol"), joiner(0x65, "dave"));
    admit(&a, org, &bob, None).await;
    admit(&a, org, &carol, None).await;
    for (_, received) in receivers.iter_mut() {
        while received.try_recv().is_ok() {}
    }

    let submitted = tokio::time::timeout(Duration::from_secs(120), admit(&a, org, &dave, None)).await.unwrap();
    assert_eq!(submitted.epoch, Epoch::new(4), "committed");
    let reported: Vec<DevicePublicKey> = submitted.sends.per_device.iter().map(|(device, _)| *device).collect();
    assert_eq!(reported, recipients_in_record_order(&a, org, sender_device).await, "every recipient, in the commit's order");
    assert_eq!(reported.len(), 3);
    for (device, outcome) in &submitted.sends.per_device {
        if *device == dave.device_key {
            assert!(matches!(outcome, Err(SendError::Transport(_))), "dave never bound: {outcome:?}");
        } else {
            assert!(outcome.is_ok(), "{device:?}: {outcome:?}");
        }
    }
    for (device, received) in receivers.iter_mut() {
        let (from, _body) = tokio::time::timeout(NET, received.recv()).await.unwrap().unwrap();
        assert_eq!(from, sender_device, "{device:?} received the update from the sender's Device");
    }
}

// The org-io half of org-node's split item on a send that writes nothing: a send that fails is reported, and the commit,
// its record and its store file stay as they were committed.
// verifies: LLR-znhmh2
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_send_leaves_the_commit_in_place() {
    let chain = FakeChain::new();
    let (a, org) = founded("fanout-failed-send", &chain).await;
    let bob = joiner(0x61, "bob");
    // Bob's identity, with no transport address: the dial fails.
    let unreachable = PeerAddr::for_test(dead_addr(device_seed_of(0x61)));
    let submitted = admit(&a, org, &bob, Some(unreachable)).await;
    assert_eq!(submitted.sends.per_device.len(), 1);
    let (device, outcome) = &submitted.sends.per_device[0];
    assert_eq!(*device, bob.device_key);
    assert!(matches!(outcome, Err(SendError::Transport(_))), "{outcome:?}");
    assert_eq!(submitted.epoch, Epoch::new(2));
    let record = a.organisation(org).await.unwrap();
    assert_eq!(record.epoch, Epoch::new(2), "the commit stands");
    assert!(record.members.iter().any(|member| member.member_key == bob.member_key));
    assert!(a.node_for_test().provisional_updates(org).is_empty(), "the update was committed, not kept");
    let on_disk = reopened_service("fanout-failed-send", "a", &a);
    assert_eq!(on_disk.list_orgs()[0].epoch, Epoch::new(2), "the store file holds the commit");
}

// Loopback with no address: every recipient refused, nothing sent, no
// endpoint bound; the commit stands.
// verifies: LLR-r9sh6k, LLR-pw369n, PR-2dmjzj
#[tokio::test(flavor = "multi_thread")]
async fn loopback_without_an_address_is_refused_and_sends_nothing() {
    let chain = FakeChain::new();
    let (a, org) = founded("fanout-no-addr", &chain).await;
    let (_addr, mut bob_receives) = listening(device_seed_of(0x61)).await;
    let bob = joiner(0x61, "bob");
    let submitted = admit(&a, org, &bob, None).await;
    assert_eq!(submitted.sends.per_device.len(), 1);
    assert_eq!(submitted.sends.per_device[0].0, bob.device_key);
    assert!(matches!(submitted.sends.per_device[0].1, Err(SendError::NoLoopbackAddress)));
    assert_eq!(a.bound_endpoints_for_test().await, 0, "no endpoint bound");
    assert!(nothing_arrives(&mut bob_receives).await, "nothing sent");
    assert_eq!(a.organisation(org).await.unwrap().epoch, Epoch::new(2), "the commit stands");
}

// PR-2dmjzj's reproduction: a Loopback address whose identity is no
// recipient's Device key (an altered address) is refused: nothing reaches
// it, and nothing is sent to anyone.
// verifies: LLR-r9sh6k, PR-2dmjzj
#[tokio::test(flavor = "multi_thread")]
async fn loopback_to_an_address_of_no_recipient_is_refused_and_sends_nothing() {
    let chain = FakeChain::new();
    let (a, org) = founded("fanout-wrong-addr", &chain).await;
    let (_bob_addr, mut bob_receives) = listening(device_seed_of(0x61)).await;
    let (rogue_addr, mut rogue_receives) = listening([0x5c; 32]).await;
    let bob = joiner(0x61, "bob");
    let submitted = admit(&a, org, &bob, Some(PeerAddr::for_test(rogue_addr))).await;
    assert_eq!(submitted.sends.per_device.len(), 1);
    assert_eq!(submitted.sends.per_device[0].0, bob.device_key);
    assert!(matches!(submitted.sends.per_device[0].1, Err(SendError::AddressNotARecipient)), "{:?}", submitted.sends);
    assert_eq!(a.bound_endpoints_for_test().await, 0, "no endpoint bound");
    assert!(nothing_arrives(&mut rogue_receives).await, "the Organisation private key does not reach the address");
    assert!(nothing_arrives(&mut bob_receives).await, "nothing sent to the recipient either");
}

// Loopback with the address of one of two recipients: that one is dialled
// at the full address it names and receives its body from the sender's
// Device; the other is reported not attempted and receives nothing.
// verifies: LLR-r9sh6k, LLR-jn5jeh, PR-2dmjzj
#[tokio::test(flavor = "multi_thread")]
async fn loopback_sends_to_the_matching_recipient_only_and_reports_the_rest_not_attempted() {
    let chain = FakeChain::new();
    let (a, org) = founded("fanout-matching", &chain).await;
    let (bob_addr, mut bob_receives) = listening(device_seed_of(0x61)).await;
    let (carol_addr, mut carol_receives) = listening(device_seed_of(0x63)).await;
    let (bob, carol) = (joiner(0x61, "bob"), joiner(0x63, "carol"));
    let first = admit(&a, org, &bob, Some(PeerAddr::for_test(bob_addr))).await;
    assert!(first.sends.per_device[0].1.is_ok(), "{:?}", first.sends);
    tokio::time::timeout(NET, bob_receives.recv()).await.unwrap().unwrap();

    let submitted = admit(&a, org, &carol, Some(PeerAddr::for_test(carol_addr))).await;
    assert_eq!(submitted.sends.per_device.len(), 2, "{:?}", submitted.sends);
    for (device, outcome) in &submitted.sends.per_device {
        if *device == carol.device_key {
            assert!(outcome.is_ok(), "{outcome:?}");
        } else {
            assert_eq!(*device, bob.device_key);
            assert!(matches!(outcome, Err(SendError::NotAttemptedInLoopback)), "{outcome:?}");
        }
    }
    let pid = a.personas().await[0].persona_id.clone();
    let (_, sender_device) = a.persona_public_keys(&pid).await.unwrap();
    let (from, _body) = tokio::time::timeout(NET, carol_receives.recv()).await.unwrap().unwrap();
    assert_eq!(from, sender_device, "from the sender's Device");
    assert!(nothing_arrives(&mut bob_receives).await, "the other recipient is not dialled");
}
