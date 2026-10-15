#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! One endpoint per Persona (SDD-xwft92; S4, task T10 of
//! docs/plans/2026-10-06-org-io-transport.md): each endpoint is bound from
//! its Persona's keychain item (LLR-d5y7ks), at most once per Device key
//! (LLR-d2pzzk), and every Persona's endpoint accepts into the one receive
//! channel (LLR-z35vbh). Also the endpoint lifecycle tests moved from
//! org-node's `service_lifecycle.rs` (LLR-6zjzn2, LLR-cns6q6, LLR-ecz9a6),
//! rewritten onto org-io's endpoints, annotations unchanged.
//!
//! The tests run one at a time under `LOCK`: one of them installs a keychain
//! of its own that refuses, and keyring's default credential builder is
//! process-global. Every other test installs the shared in-memory keychain
//! the handles' helpers use.

use org_node::{DevicePublicKey, Joiner, OrgId, PersonaId};
use org_io::test_support::{FakeChain, MemoryKeychain, Refusal};
use org_io::transport::{BindPlan, OrgEndpoint, TransportError};
use org_io::{KeychainError, OrgIo, ReceiveError, SendError, TerminalCause};
use rand::rngs::OsRng;
use std::time::Duration;

mod common;
use common::handles::{at, bind_seed, dead_addr, handle, joiner, keep_device_seed, persona, spawn_recv_one, NET};

static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// The keychain item's service name (LLR-677fqq).
const SERVICE: &str = "ods-device-seed";

// verifies: LLR-d5y7ks
#[tokio::test(flavor = "multi_thread")]
async fn the_bound_endpoint_id_is_the_device_key() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let device_key = keep_device_seed([0x31; 32]);
    let endpoint = OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await.unwrap();
    assert_eq!(endpoint.device_key(), device_key);
    assert_eq!(endpoint.inner().id().as_bytes(), device_key.as_bytes(), "the iroh EndpointId is the Device key");
    assert_eq!(endpoint.node_addr_for_dial().id.as_bytes(), device_key.as_bytes());
}

// Two binds from one item are two endpoints (each its own sockets) with one
// identity: the identity is the item's, not the bind's.
// verifies: LLR-d5y7ks
#[tokio::test(flavor = "multi_thread")]
async fn binding_twice_for_one_persona_yields_the_same_id() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let device_key = keep_device_seed([0x32; 32]);
    let first = OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await.unwrap();
    let second = OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await.unwrap();
    assert_ne!(first.bound_sockets(), second.bound_sockets(), "two binds, two endpoints");
    assert_eq!(first.inner().id(), second.inner().id(), "one identity");
}

// Abnormal: the item of one Device key holds another key's seed (an item
// another program or a copy left): the bind is refused, naming the key, and
// no endpoint is left open.
// verifies: LLR-d5y7ks
#[tokio::test(flavor = "multi_thread")]
async fn a_keychain_item_for_another_key_is_refused_and_the_endpoint_closed() {
    let _guard = LOCK.lock().await;
    let keychain = MemoryKeychain::shared();
    keychain.install();
    let device_key = keep_device_seed([0x33; 32]);
    let other_key = keep_device_seed([0x34; 32]);
    keychain.insert(SERVICE, &hex::encode(device_key.as_bytes()), [0x34; 32].to_vec());
    match OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await {
        Err(TransportError::KeychainKeyMismatch(named)) => assert_eq!(*named, device_key),
        Err(other) => panic!("refused for another reason: {other}"),
        Ok(endpoint) => panic!("bound under {:?}", endpoint.device_key()),
    }
    // The other key's own item still binds its own identity.
    let other = OrgEndpoint::bind_from_keychain(&other_key, BindPlan::Loopback).await.unwrap();
    assert_eq!(other.device_key(), other_key);
}

// Abnormal: binding is one of the operations that needs the keychain
// (REQ-6d9xv6, as narrowed 2026-10-07); a locked keychain refuses it, and a
// missing item does too, each naming the Device key.
// verifies: LLR-d5y7ks, REQ-6d9xv6
#[tokio::test(flavor = "multi_thread")]
async fn a_locked_keychain_refuses_the_bind() {
    let _guard = LOCK.lock().await;
    let keychain = MemoryKeychain::default();
    keychain.install();
    let device_key = keep_device_seed([0x35; 32]);
    keychain.refuse_as(Some(Refusal::Locked));
    match OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await {
        Err(TransportError::Keychain(refused)) => assert_eq!(*refused, KeychainError::Locked { device_key }),
        Err(other) => panic!("refused for another reason: {other}"),
        Ok(_) => panic!("bound from a locked keychain"),
    }
    keychain.refuse_as(None);
    let unknown = org_node::test_fixtures::device_key(0x36);
    match OrgEndpoint::bind_from_keychain(&unknown, BindPlan::Loopback).await {
        Err(TransportError::Keychain(refused)) => assert_eq!(*refused, KeychainError::ItemMissing { device_key: unknown }),
        Err(other) => panic!("refused for another reason: {other}"),
        Ok(_) => panic!("bound with no item"),
    }
    MemoryKeychain::shared().install();
}

/// A node, over `chain`, with two Personas, each the founder of an
/// Organisation of its own.
async fn two_founders(tag: &str, chain: &FakeChain) -> (OrgIo, [(PersonaId, OrgId); 2]) {
    let io = handle(tag, "a", chain);
    let first = persona(&io, "first").await;
    let second = persona(&io, "second").await;
    let org_1 = io.found_organisation(&mut OsRng, &first).await.unwrap();
    let org_2 = io.found_organisation(&mut OsRng, &second).await.unwrap();
    (io, [(first, org_1), (second, org_2)])
}

/// Admit `joiner` into `org_id` and send the admission to a sink; the
/// Device key the sink saw it come from.
async fn sender_of_the_admission(io: &OrgIo, org_id: OrgId, joiner: &Joiner, sink_seed: [u8; 32]) -> DevicePublicKey {
    let update = io.admit_member(&mut OsRng, org_id, joiner).await.unwrap();
    let (sink, sink_task) = spawn_recv_one(sink_seed).await;
    io.submit_commit_send(&mut OsRng, &update, at(sink)).await.unwrap();
    let (_sink_endpoint, sender, _message) = sink_task.await.unwrap();
    sender
}

// The inversion of org-node's pin of the endpoint bound once per service:
// a node administering two Organisations through two Personas sends each
// Organisation's update from its own Persona's endpoint, under that
// Persona's Device key, whichever was bound first.
// verifies: LLR-d2pzzk, PR-8qsnhx
#[tokio::test(flavor = "multi_thread")]
async fn a_second_organisations_message_goes_out_under_its_own_personas_key() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let (io, [(first, org_1), (second, org_2)]) = two_founders("endpoints-two-orgs", &chain).await;
    let first_key = io.node_for_test().persona_public_keys(&first).unwrap().1;
    let second_key = io.node_for_test().persona_public_keys(&second).unwrap().1;

    // *S4 task T12, 2026-10-13:* each sink is the joiner's own Device
    // (`joiner(seed, ..)`'s is the seed `[seed + 1; 32]`'s): a Loopback send
    // goes only to a recipient's address.
    let sent_1 = sender_of_the_admission(&io, org_1, &joiner(0x41, "bob"), [0x42; 32]).await;
    assert_eq!(sent_1, first_key, "the first Organisation's update goes out under its Persona's key");
    let sent_2 = sender_of_the_admission(&io, org_2, &joiner(0x43, "bea"), [0x44; 32]).await;
    assert_eq!(sent_2, second_key, "the second Organisation's update goes out under its own Persona's key");
    assert_ne!(sent_2, first_key, "and never under the first Persona's");
    assert_eq!(io.bound_endpoints_for_test().await, 2, "one endpoint per Persona");
}

// Each endpoint is bound lazily and once: asking for a Persona's endpoint
// again returns the same one (its sockets), and another Persona's is its own.
// verifies: LLR-d2pzzk
#[tokio::test(flavor = "multi_thread")]
async fn each_persona_has_one_endpoint_bound_lazily() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = handle("endpoints-lazy", "a", &chain);
    let first = persona(&io, "first").await;
    let second = persona(&io, "second").await;
    assert_eq!(io.bound_endpoints_for_test().await, 0, "nothing bound before it is needed");
    let first_sockets = io.endpoint_for_test(&first).await.unwrap().bound_sockets();
    assert_eq!(io.endpoint_for_test(&first).await.unwrap().bound_sockets(), first_sockets, "the same endpoint again");
    assert_eq!(io.bound_endpoints_for_test().await, 1);
    let second_endpoint = io.endpoint_for_test(&second).await.unwrap();
    assert_ne!(second_endpoint.bound_sockets(), first_sockets);
    assert_eq!(io.bound_endpoints_for_test().await, 2);
}

// A node with two Personas receives on both: a body sent to either
// Persona's Device reaches the one receive channel with the sender the
// handshake authenticated (S3a's member-sender rule needs it).
// verifies: LLR-z35vbh
#[tokio::test(flavor = "multi_thread")]
async fn a_message_to_either_personas_device_reaches_the_receive_channel() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = handle("endpoints-receive", "b", &chain);
    let first = persona(&io, "first").await;
    let second = persona(&io, "second").await;
    io.start_receiving().await.unwrap();
    assert_eq!(io.bound_endpoints_for_test().await, 2, "start_receiving binds every Persona's endpoint");

    let sender = bind_seed([0x61; 32]).await;
    for (persona_id, body) in [(&first, b"to the first".to_vec()), (&second, b"to the second".to_vec())] {
        let addr = io.endpoint_for_test(persona_id).await.unwrap().node_addr_for_dial();
        tokio::time::timeout(NET, sender.send_body(addr, &body)).await.unwrap().unwrap();
        let (from, received) = tokio::time::timeout(NET, io.next_inbound_for_test()).await.unwrap().unwrap();
        assert_eq!(from, sender.device_key(), "the handshake's sender");
        assert_eq!(received, body, "the body, unchanged");
    }
}

// ---- A closed endpoint is not reused (S4, task T20; PR-tcd2ta) -------------

/// Close `persona_id`'s endpoint, as a permanent transport failure would,
/// and wait until the receive channel reports every accept task ended.
async fn close_and_drain(io: &OrgIo, persona_id: &PersonaId) {
    io.endpoint_for_test(persona_id).await.unwrap().inner().close().await;
    match tokio::time::timeout(NET, io.next_inbound_for_test()).await.unwrap() {
        Err(TransportError::EndpointClosed) => {}
        Err(other) => panic!("the closed endpoint ended the receive channel for another reason: {other}"),
        Ok((from, _)) => panic!("a body from {from:?} arrived after the close"),
    }
}

/// Send `body` from a fresh sender to `persona_id`'s endpoint, take it off
/// the receive channel, check it came with the sender the handshake
/// authenticated, and return it.
async fn delivered_to(io: &OrgIo, persona_id: &PersonaId, sender_seed: [u8; 32], body: &[u8]) -> Vec<u8> {
    let sender = bind_seed(sender_seed).await;
    let addr = io.endpoint_for_test(persona_id).await.unwrap().node_addr_for_dial();
    tokio::time::timeout(NET, sender.send_body(addr, body)).await.unwrap().unwrap();
    let (from, received) = tokio::time::timeout(NET, io.next_inbound_for_test()).await.unwrap().unwrap();
    assert_eq!(from, sender.device_key(), "the handshake's sender");
    received
}

// After a Persona's endpoint has closed, `start_receiving` binds it again
// from the keychain item and accepts on it: a message to that Persona's
// Device reaches the receive channel, and one endpoint is bound, not two.
// verifies: LLR-z35vbh, LLR-d2pzzk, PR-tcd2ta
#[tokio::test(flavor = "multi_thread")]
async fn after_its_endpoint_closes_start_receiving_binds_it_again_and_receives() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = handle("endpoints-restart", "b", &chain);
    let pid = persona(&io, "restart").await;
    let device_key = io.node_for_test().persona_public_keys(&pid).unwrap().1;
    io.start_receiving().await.unwrap();
    let first_sockets = io.endpoint_for_test(&pid).await.unwrap().bound_sockets();
    close_and_drain(&io, &pid).await;

    io.start_receiving().await.unwrap();
    let endpoint = io.endpoint_for_test(&pid).await.unwrap();
    assert!(!endpoint.inner().is_closed(), "start_receiving binds the closed endpoint again");
    assert_eq!(endpoint.device_key(), device_key, "under the same Persona's Device key");
    assert_ne!(endpoint.bound_sockets(), first_sockets, "a new endpoint, not the closed one");
    assert_eq!(io.bound_endpoints_for_test().await, 1, "the closed endpoint is not kept beside the new one");

    let received = delivered_to(&io, &pid, [0x71; 32], b"after the restart").await;
    assert_eq!(received, b"after the restart".to_vec(), "the body reaches the receive channel");
}

// After a Persona's endpoint has closed, a send from that Persona binds the
// endpoint again and goes out under the Persona's Device key.
// verifies: LLR-d2pzzk, PR-tcd2ta
#[tokio::test(flavor = "multi_thread")]
async fn after_its_endpoint_closes_a_send_binds_it_again() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = handle("endpoints-resend", "a", &chain);
    let pid = persona(&io, "resend").await;
    let device_key = io.node_for_test().persona_public_keys(&pid).unwrap().1;
    let org_id = io.found_organisation(&mut OsRng, &pid).await.unwrap();
    io.endpoint_for_test(&pid).await.unwrap().inner().close().await;

    let update = io.admit_member(&mut OsRng, org_id, &joiner(0x73, "cy")).await.unwrap();
    let (sink, sink_task) = spawn_recv_one([0x74; 32]).await;
    let submitted = io.submit_commit_send(&mut OsRng, &update, at(sink)).await.unwrap();
    for (recipient, sent) in &submitted.sends.per_device {
        assert!(sent.is_ok(), "the send to {recipient:?} went out from a live endpoint: {sent:?}");
    }
    let (_sink_endpoint, sender, _message) = tokio::time::timeout(NET, sink_task).await.unwrap().unwrap();
    assert_eq!(sender, device_key, "under the sending Persona's Device key");
    assert!(!io.endpoint_for_test(&pid).await.unwrap().inner().is_closed(), "the send bound the endpoint again");
    assert_eq!(io.bound_endpoints_for_test().await, 1);
}

// Abnormal: the keychain refuses the rebind of a closed endpoint.
// `start_receiving` reports the failed bind, keeps nothing closed in its
// place, and binds once the keychain answers again.
// verifies: LLR-z35vbh, LLR-d2pzzk, PR-tcd2ta
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_rebind_is_a_failed_bind_and_the_next_start_binds() {
    let _guard = LOCK.lock().await;
    let keychain = MemoryKeychain::shared();
    keychain.install();
    let chain = FakeChain::new();
    let io = handle("endpoints-refused-rebind", "b", &chain);
    let pid = persona(&io, "refused").await;
    let device_key = io.node_for_test().persona_public_keys(&pid).unwrap().1;
    io.start_receiving().await.unwrap();
    close_and_drain(&io, &pid).await;

    keychain.refuse_as(Some(Refusal::Locked));
    let refused = io.start_receiving().await;
    keychain.refuse_as(None);
    match refused {
        Err(ReceiveError::Terminal(TerminalCause::BindFailed(TransportError::Keychain(refused)))) => {
            assert_eq!(*refused, KeychainError::Locked { device_key });
        }
        other => panic!("the refused rebind was not a failed bind: {other:?}"),
    }
    assert_eq!(io.bound_endpoints_for_test().await, 0, "the closed endpoint is not kept");

    io.start_receiving().await.unwrap();
    let received = delivered_to(&io, &pid, [0x75; 32], b"after the refusal").await;
    assert_eq!(received, b"after the refusal".to_vec());
}

// ---- A rebound endpoint accepts while the receiver runs (S4, task T21) -----

// One Persona's endpoint closes while the other's keeps accepting, so the
// receive loop does not end and nothing calls `start_receiving` again. A
// send from the first Persona binds its endpoint again; that endpoint
// accepts too: a message to its Device reaches the receive channel, and the
// other Persona's endpoint still delivers.
// verifies: LLR-z35vbh, LLR-d2pzzk, PR-47aycj
#[tokio::test(flavor = "multi_thread")]
async fn a_send_that_rebinds_a_closed_endpoint_while_the_receiver_runs_accepts_on_it() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let (io, [(closing, org_id), (open, _)]) = two_founders("endpoints-rebind-receiving", &chain).await;
    io.start_receiving().await.unwrap();
    let closed_sockets = io.endpoint_for_test(&closing).await.unwrap().bound_sockets();
    io.endpoint_for_test(&closing).await.unwrap().inner().close().await;

    let update = io.admit_member(&mut OsRng, org_id, &joiner(0x77, "dee")).await.unwrap();
    let (sink, sink_task) = spawn_recv_one([0x78; 32]).await;
    let submitted = io.submit_commit_send(&mut OsRng, &update, at(sink)).await.unwrap();
    for (recipient, sent) in &submitted.sends.per_device {
        assert!(sent.is_ok(), "the send to {recipient:?} went out from a live endpoint: {sent:?}");
    }
    tokio::time::timeout(NET, sink_task).await.unwrap().unwrap();
    let rebound = io.endpoint_for_test(&closing).await.unwrap();
    assert_ne!(rebound.bound_sockets(), closed_sockets, "the send bound the endpoint again");

    let sender = bind_seed([0x79; 32]).await;
    let delivered = tokio::time::timeout(Duration::from_secs(5), async {
        sender.send_body(rebound.node_addr_for_dial(), b"to the rebound").await.unwrap();
        io.next_inbound_for_test().await
    })
    .await;
    match delivered {
        Ok(Ok((from, body))) => {
            assert_eq!(from, sender.device_key(), "the handshake's sender");
            assert_eq!(body, b"to the rebound".to_vec(), "the body, unchanged");
        }
        Ok(Err(error)) => panic!("the receive channel ended instead: {error}"),
        Err(_) => panic!("the rebound endpoint has no accept task: nothing reached the receive channel"),
    }
    let received = delivered_to(&io, &open, [0x7a; 32], b"to the open one").await;
    assert_eq!(received, b"to the open one".to_vec(), "the other Persona's endpoint still delivers");
    assert_eq!(io.bound_endpoints_for_test().await, 2, "one endpoint per Persona");
}

// Abnormal: while the receiver runs, the keychain refuses the rebind a send
// asks for. The refusal is the send's, nothing closed is kept, and the next
// bind, once the keychain answers, accepts. The count of accept tasks stays
// exact: once both endpoints close, the receive channel ends.
// verifies: LLR-z35vbh, LLR-d2pzzk, PR-47aycj
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_rebind_while_the_receiver_runs_binds_and_accepts_on_the_next_send() {
    let _guard = LOCK.lock().await;
    let keychain = MemoryKeychain::shared();
    keychain.install();
    let chain = FakeChain::new();
    let io = handle("endpoints-refused-rebind-receiving", "b", &chain);
    let closing = persona(&io, "closing").await;
    let open = persona(&io, "open").await;
    let device_key = io.node_for_test().persona_public_keys(&closing).unwrap().1;
    let org = io.found_organisation(&mut OsRng, &closing).await.unwrap();
    let joining = joiner(0x60, "joining");
    let update = io.admit_member(&mut OsRng, org, &joining).await.unwrap();
    io.start_receiving().await.unwrap();
    io.endpoint_for_test(&closing).await.unwrap().inner().close().await;

    // The send path's bind, driven through the submission whose commit it
    // sends. *Moved 2026-10-15 (S4 TEST review round 1, finding-6; Seams 2):*
    // it called `endpoint_for_test`, the bind `send.rs` makes, directly.
    // Mutation check, that day: `send_committed` reporting a fixed reason in
    // place of the bind's refusal failed the reason assertion.
    keychain.refuse_as(Some(Refusal::Locked));
    let submitted = io.submit_commit_send(&mut OsRng, &update, at(dead_addr([0x61; 32]))).await;
    keychain.refuse_as(None);
    let submitted = submitted.expect("the chain write and the commit stand; only the send is refused");
    let locked = TransportError::Keychain(Box::new(KeychainError::Locked { device_key })).to_string();
    match submitted.sends.per_device.as_slice() {
        [(recipient, Err(SendError::SenderEndpoint(reason)))] => {
            assert_eq!(*recipient, joining.device_key, "the one recipient");
            assert_eq!(*reason, locked, "the keychain's refusal is the send's");
        }
        other => panic!("the send was not refused at the sender's endpoint: {other:?}"),
    }
    assert_eq!(io.bound_endpoints_for_test().await, 1, "the closed endpoint is not kept");

    // The next send, once the keychain answers, binds the endpoint again.
    // *Driven through a send 2026-10-15 (S4 TEST review round 2, finding-3):*
    // it rebound through `endpoint_for_test` directly; now the next
    // submission's send binds, and the endpoint is only looked up after.
    let second = joiner(0x62, "second");
    let update = io.admit_member(&mut OsRng, org, &second).await.unwrap();
    let submitted = io.submit_commit_send(&mut OsRng, &update, at(dead_addr([0x63; 32]))).await.unwrap();
    for (recipient, sent) in &submitted.sends.per_device {
        assert!(!matches!(sent, Err(SendError::SenderEndpoint(_))), "the send to {recipient:?} found no endpoint: {sent:?}");
    }
    assert!(
        submitted.sends.per_device.iter().any(|(recipient, sent)| *recipient == second.device_key
            && matches!(sent, Err(SendError::Transport(_)))),
        "the send dialled the addressed recipient from the rebound endpoint: {:?}",
        submitted.sends.per_device
    );
    assert_eq!(io.bound_endpoints_for_test().await, 2, "the send bound the endpoint again");
    let rebound = io.bound_endpoint_for_test(&closing).await.expect("the send bound the closing Persona's endpoint");
    let sender = bind_seed([0x7b; 32]).await;
    let delivered = tokio::time::timeout(Duration::from_secs(5), async {
        sender.send_body(rebound.node_addr_for_dial(), b"after the refusal").await.unwrap();
        io.next_inbound_for_test().await
    })
    .await;
    match delivered {
        Ok(Ok((from, body))) => {
            assert_eq!(from, sender.device_key(), "the handshake's sender");
            assert_eq!(body, b"after the refusal".to_vec());
        }
        Ok(Err(error)) => panic!("the receive channel ended instead: {error}"),
        Err(_) => panic!("the rebound endpoint has no accept task: nothing reached the receive channel"),
    }

    rebound.inner().close().await;
    io.endpoint_for_test(&open).await.unwrap().inner().close().await;
    match tokio::time::timeout(NET, io.next_inbound_for_test()).await {
        Ok(Err(TransportError::EndpointClosed)) => {}
        Ok(Err(other)) => panic!("the channel ended for another reason: {other}"),
        Ok(Ok((from, _))) => panic!("a body from {from:?} arrived after every endpoint closed"),
        Err(_) => panic!("every endpoint closed, yet the receive channel waits: the accept-task count is off"),
    }
}

// ---- A Persona created while the node receives (S4 review round 1, finding 2)

/// Send `body` from a fresh sender to the endpoint the node has already
/// bound for `persona_id`, binding nothing (so the test starts no accept
/// task the node did not), and return what reaches the receive channel.
async fn delivered_to_bound(io: &OrgIo, persona_id: &PersonaId, sender_seed: [u8; 32], body: &[u8]) -> Vec<u8> {
    let endpoint = io.bound_endpoint_for_test(persona_id).await.expect("the node bound the Persona's endpoint");
    let sender = bind_seed(sender_seed).await;
    let delivered = tokio::time::timeout(Duration::from_secs(5), async {
        sender.send_body(endpoint.node_addr_for_dial(), body).await.unwrap();
        io.next_inbound_for_test().await
    })
    .await;
    match delivered {
        Ok(Ok((from, received))) => {
            assert_eq!(from, sender.device_key(), "the handshake's sender");
            received
        }
        Ok(Err(error)) => panic!("the receive channel ended instead: {error}"),
        Err(_) => panic!("the endpoint has no accept task: nothing reached the receive channel"),
    }
}

// The receiver runs for the first Persona; a second Persona is created
// meanwhile. Its endpoint is bound and accepts at once: a message to its
// Device reaches the receive channel with no new `start_receiving`, and the
// first Persona's endpoint still delivers.
// verifies: LLR-z35vbh, PR-y6t5kt
#[tokio::test(flavor = "multi_thread")]
async fn a_persona_created_while_the_receiver_runs_is_received_on() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = handle("endpoints-created-receiving", "b", &chain);
    let first = persona(&io, "first").await;
    io.start_receiving().await.unwrap();

    let created = persona(&io, "created").await;
    assert_eq!(io.bound_endpoints_for_test().await, 2, "the new Persona's endpoint is bound at its creation");
    let received = delivered_to_bound(&io, &created, [0x7c; 32], b"to the new Persona").await;
    assert_eq!(received, b"to the new Persona".to_vec(), "the body reaches the receive channel");
    let received = delivered_to_bound(&io, &first, [0x7d; 32], b"to the first").await;
    assert_eq!(received, b"to the first".to_vec(), "the first Persona's endpoint still delivers");
}

// Abnormal: while the node receives, the replying Persona's endpoint cannot
// be bound (the keychain refuses its item). `produce_reply` hands out no
// reply and writes nothing, so no admission is invited that this node would
// not receive; once the keychain answers, the reply is produced and the
// Persona's endpoint accepts.
// verifies: LLR-z35vbh, PR-y6t5kt
#[tokio::test(flavor = "multi_thread")]
async fn no_reply_is_produced_while_receiving_for_a_persona_whose_endpoint_cannot_be_bound() {
    let _guard = LOCK.lock().await;
    let keychain = MemoryKeychain::shared();
    keychain.install();
    let chain = FakeChain::new();
    let inviter = handle("endpoints-reply-receiving", "a", &chain);
    let alice = persona(&inviter, "alice").await;
    let org_id = inviter.found_organisation(&mut OsRng, &alice).await.unwrap();
    let invite = inviter.issue_invite(&mut OsRng, org_id, "Acme", "Bob").await.unwrap();

    let io = handle("endpoints-reply-receiving", "b", &FakeChain::sharing_slots_with(&chain));
    io.start_receiving().await.unwrap();
    let bob = persona(&io, "bob").await;
    if let Some(endpoint) = io.bound_endpoint_for_test(&bob).await {
        endpoint.inner().close().await;
    }
    let before = common::handles::store_bytes("endpoints-reply-receiving", "b");

    keychain.refuse_as(Some(Refusal::Locked));
    let refused = io.produce_reply(&mut OsRng, &invite, &bob, true).await;
    keychain.refuse_as(None);
    let device_key = io.node_for_test().persona_public_keys(&bob).unwrap().1;
    match refused {
        Err(org_io::ExchangeError::Bind(TransportError::Keychain(refused))) => {
            assert_eq!(*refused, KeychainError::Locked { device_key });
        }
        Err(other) => panic!("refused for another reason: {other}"),
        Ok(_) => panic!("a reply was handed out for a Persona this node cannot receive on"),
    }
    assert_eq!(common::handles::store_bytes("endpoints-reply-receiving", "b"), before, "nothing written");

    io.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    let received = delivered_to_bound(&io, &bob, [0x7f; 32], b"the admission").await;
    assert_eq!(received, b"the admission".to_vec(), "the replying Persona's endpoint accepts");
}

// ---- A closing endpoint (S4 review round 2, A7) ----------------------------

// Abnormal: a Persona's endpoint has started closing but not finished (its
// close future polled once, still pending). The next call binds a new
// endpoint rather than handing out the closing one.
// verifies: LLR-d2pzzk
#[tokio::test(flavor = "multi_thread")]
async fn a_closing_endpoint_is_not_handed_out_again() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = handle("endpoints-closing", "a", &chain);
    let pid = persona(&io, "closing").await;
    let closing = io.endpoint_for_test(&pid).await.unwrap();
    let closing_sockets = closing.bound_sockets();

    let mut close = Box::pin(closing.inner().close());
    let polled = std::future::Future::poll(close.as_mut(),&mut std::task::Context::from_waker(std::task::Waker::noop()));
    assert!(polled.is_pending(), "the close finished at once: this run cannot show a closing endpoint");
    assert!(!closing.inner().is_closed(), "iroh reports it not yet closed");

    let next = io.endpoint_for_test(&pid).await.unwrap();
    assert_ne!(next.bound_sockets(), closing_sockets, "the closing endpoint was handed out again");
    close.await;
}

// ---- A stalled connection (S4 review round 2, finding 1; PR-d2p5y2) --------

/// A peer that completes the handshake with `persona_id`'s endpoint and
/// then opens no stream: the connection, held open by the caller.
async fn stalled_connection(io: &OrgIo, persona_id: &PersonaId, seed: [u8; 32]) -> (OrgEndpoint, iroh::endpoint::Connection) {
    let addr = io.endpoint_for_test(persona_id).await.unwrap().node_addr_for_dial();
    let staller = bind_seed(seed).await;
    let connection = tokio::time::timeout(NET, staller.inner().connect(addr, org_io::transport::ALPN)).await.unwrap().unwrap();
    (staller, connection)
}

// Abnormal: one peer completes the handshake and never opens a stream. A
// second peer's message to the same Persona's Device still reaches the
// receive channel while the first connection is held open.
// verifies: LLR-8uf9a5
#[tokio::test(flavor = "multi_thread")]
async fn a_stalled_connection_does_not_stop_another_peers_message() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = handle("endpoints-stalled", "b", &chain);
    let pid = persona(&io, "stalled-on").await;
    io.start_receiving().await.unwrap();
    let (_staller, _held) = stalled_connection(&io, &pid, [0x7e; 32]).await;

    let received = delivered_to_bound(&io, &pid, [0x7f; 32], b"past the stalled one").await;
    assert_eq!(received, b"past the stalled one".to_vec(), "the second peer's body reaches the receive channel");
}

// Abnormal: a connection that has given no body within the limit is
// dropped (closed by the receiving side), and its failure reaches the
// receive channel as one connection's, not as the end of the channel.
// verifies: LLR-8uf9a5
#[tokio::test(flavor = "multi_thread")]
async fn a_stalled_connection_is_dropped_after_the_limit() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let limit = Duration::from_secs(1);
    let io = handle("endpoints-stalled-dropped", "b", &chain).with_connection_time_limit_for_test(limit);
    let pid = persona(&io, "stalled-on").await;
    io.start_receiving().await.unwrap();
    let (_staller, held) = stalled_connection(&io, &pid, [0x7e; 32]).await;

    match tokio::time::timeout(limit * 10, io.next_inbound_for_test()).await {
        Ok(Err(TransportError::TimedOut(reported))) => assert_eq!(reported, limit, "the limit it was dropped at"),
        Ok(Err(other)) => panic!("the stalled connection failed for another reason: {other}"),
        Ok(Ok((from, _))) => panic!("a body from {from:?} arrived from a peer that sent none"),
        Err(_) => panic!("the stalled connection was not dropped within ten times the limit"),
    }
    assert!(tokio::time::timeout(limit * 10, held.closed()).await.is_ok(), "the receiving side closed the stalled connection");
}

// Boundary: an endpoint reads sixteen connections at once (LLR-8uf9a5's
// `CONNECTIONS_AT_ONCE`). With sixteen stalled, a seventeenth peer's body
// is not taken until a slot frees, when the stalled ones are dropped at the
// limit; then it arrives.
// verifies: LLR-8uf9a5
#[tokio::test(flavor = "multi_thread")]
async fn a_seventeenth_connection_waits_for_a_slot_while_sixteen_stall() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let limit = Duration::from_secs(2);
    let io = handle("endpoints-stalled-slots", "b", &chain).with_connection_time_limit_for_test(limit);
    let pid = persona(&io, "stalled-on").await;
    io.start_receiving().await.unwrap();
    let mut held = vec![];
    for seed in 0x40..0x50u8 {
        held.push(stalled_connection(&io, &pid, [seed; 32]).await);
    }
    let stalled_since = std::time::Instant::now();

    let addr = io.bound_endpoint_for_test(&pid).await.unwrap().node_addr_for_dial();
    let sender = bind_seed([0x7f; 32]).await;
    let (sent, received) = tokio::join!(tokio::time::timeout(NET, sender.send_body(addr, b"after a slot frees")), async {
        loop {
            match tokio::time::timeout(NET, io.next_inbound_for_test()).await.unwrap() {
                Ok(inbound) => return inbound,
                Err(TransportError::TimedOut(_)) => {}
                Err(other) => panic!("the receive channel gave another failure: {other}"),
            }
        }
    });
    sent.unwrap().unwrap();
    assert_eq!(received, (sender.device_key(), b"after a slot frees".to_vec()));
    assert!(stalled_since.elapsed() >= limit * 9 / 10, "the body was taken before a slot freed: {:?}", stalled_since.elapsed());
}

// ---- Moved from org-node's `service_lifecycle.rs` (S4, task T10) -----------

// *Moved 2026-10-13 (S4, task T10) from org-node's `service_lifecycle.rs`
// and rewritten onto org-io's endpoints:* the endpoint is bound once per
// Persona and the same one is returned after. The bound sockets tell one
// endpoint from two: a second bind takes a fresh ephemeral port.
// verifies: LLR-6zjzn2
#[tokio::test(flavor = "multi_thread")]
async fn the_endpoint_is_bound_once_and_the_same_one_is_returned_after() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = handle("endpoints-once", "a", &chain);
    let pid = persona(&io, "ep").await;
    let expected = io.node_for_test().persona_public_keys(&pid).unwrap().1;

    let first = io.endpoint_for_test(&pid).await.unwrap();
    let first_sockets = first.bound_sockets();
    assert_eq!(first.device_key(), expected);
    assert!(!first_sockets.is_empty(), "no sockets bound");

    let second = io.endpoint_for_test(&pid).await.unwrap();
    assert_eq!(second.bound_sockets(), first_sockets, "the second call bound a new endpoint instead of returning the first");
    assert_eq!(second.device_key(), expected);
}

// *Moved 2026-10-13 (S4, task T10) from org-node's `service_lifecycle.rs`
// and rewritten onto org-io's endpoints:* a node holding more than one
// Persona binds the endpoint of the Persona asked for, from that Persona's
// keychain item, not the first Persona's.
// verifies: LLR-cns6q6
#[tokio::test(flavor = "multi_thread")]
async fn the_endpoint_binds_the_named_personas_device_not_the_first_personas() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = handle("endpoints-named", "a", &chain);
    let first = persona(&io, "one").await;
    let second = persona(&io, "two").await;
    let first_key = io.node_for_test().persona_public_keys(&first).unwrap().1;
    let second_key = io.node_for_test().persona_public_keys(&second).unwrap().1;
    assert_ne!(first_key, second_key);

    let bound = io.endpoint_for_test(&second).await.unwrap().device_key();
    assert_eq!(bound, second_key, "the endpoint must authenticate the named persona's device key");
    assert_ne!(bound, first_key, "the endpoint bound the first persona's device key, not the named one");
}

// *Moved 2026-10-13 (S4, task T10) from org-node's `service_lifecycle.rs`
// and rewritten onto org-io's endpoints:* a node in Loopback mode binds
// loopback sockets and nothing else.
// verifies: REQ-db6s7q, LLR-ecz9a6
#[tokio::test(flavor = "multi_thread")]
async fn a_service_in_loopback_mode_binds_loopback_sockets_only() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = handle("endpoints-loopback", "a", &chain);
    let pid = persona(&io, "m").await;
    let sockets = io.endpoint_for_test(&pid).await.unwrap().bound_sockets();
    assert!(!sockets.is_empty(), "no sockets bound");
    for socket in sockets {
        assert!(socket.ip().is_loopback(), "bound a non-loopback address: {socket}");
    }
}
