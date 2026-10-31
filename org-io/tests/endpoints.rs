#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! One endpoint per Persona (SDD-xwft92): each endpoint is bound from
//! its Persona's keychain item (LLR-d5y7ks), at most once per Device key
//! (LLR-d2pzzk), and every Persona's endpoint accepts into the one receive
//! channel (LLR-z35vbh); and the endpoint's lifecycle (LLR-6zjzn2,
//! LLR-cns6q6, LLR-ecz9a6).
//!
//! The tests run one at a time under `LOCK`: one of them installs a keychain
//! of its own that refuses, and keyring's default credential builder is
//! process-global. Every other test installs the shared in-memory keychain
//! the handles' helpers use.

use org_node::{DevicePublicKey, OrgId, PersonaId};
use org_io::send::{Delivery, SendError, SendReport};
use org_io::test_support::{FakeChain, MemoryKeychain, Refusal};
use org_io::transport::{BindPlan, OrgEndpoint, TransportError};
use org_io::{HeldTarget, KeychainError, OrgIo, ReceiveError, Received, TerminalCause, Verified, WriteReport};
use std::time::Duration;

mod common;
use common::handles::{bind_seed, device_key_of, handle, keep_device_seed, persona, NET};
use common::journey;

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
// (REQ-6d9xv6, as narrowed); a locked keychain refuses it, and a
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

// The tests below send through `send_organisation`, the Organisations
// founded, admitted and adopted through the handle's own operations. A
// proposal's binding of its new Persona while the node receives is
// LLR-jvx6j9's, tested in `persona_creation.rs`.

// ---- A send goes out from its own Persona's endpoint (PR-8qsnhx, PR-tcd2ta, PR-47aycj)

/// Party `party` of test `tag`, over `chain`'s slots, admitted to `org` by
/// `io`, which must be receiving: it proposes its Member from an Invite `io`
/// issues and asks over the transport; `io` keeps the request, proposes,
/// writes and adopts the admitting epoch. The joiner's handle, receiving.
async fn admitted(io: &OrgIo, chain: &FakeChain, org: OrgId, tag: &str, party: &str) -> OrgIo {
    let joiner = handle(tag, party, &FakeChain::sharing_slots_with(chain));
    common::handles::proposed(io, org, &joiner).await;
    let written = io.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap();
    assert!(matches!(written, WriteReport::Included { .. }), "{written:?}");
    let verified = io.verify_organisation(HeldTarget::Organisation(org)).await;
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    joiner
}

/// `io` sends `org`'s adopted epoch while `to` receives it: the Device the
/// message came from, as `to`'s receive step reports it, and the report.
async fn sent_from(io: &OrgIo, org: OrgId, to: &OrgIo) -> (DevicePublicKey, SendReport) {
    let (received, sent) = tokio::join!(journey::next(to), io.send_organisation(org));
    match received {
        Ok(Received::ProvisionalOrganisationReceived { from, .. }) => (from, sent.unwrap()),
        other => panic!("the epoch was not received: {other:?}"),
    }
}

// The inversion of org-node's pin of the endpoint bound once per service: a
// node administering two Organisations through two Personas sends each
// Organisation's epoch from its own Persona's endpoint, under that Persona's
// Device key, whichever was bound first.
// verifies: LLR-d2pzzk, PR-8qsnhx
#[tokio::test(flavor = "multi_thread")]
async fn a_second_organisations_message_goes_out_under_its_own_personas_key() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let tag = "endpoints-two-orgs";
    let chain = FakeChain::new();
    let io = handle(tag, "a", &chain);
    let (org_1, first) = common::handles::founded(&io, "first").await;
    let (org_2, second) = common::handles::founded(&io, "second").await;
    io.start_receiving().await.unwrap();
    let bob = admitted(&io, &chain, org_1, tag, "b").await;
    let bea = admitted(&io, &chain, org_2, tag, "c").await;

    let (from_1, _) = sent_from(&io, org_1, &bob).await;
    assert_eq!(from_1, first, "the first Organisation's epoch goes out under its Persona's key");
    let (from_2, _) = sent_from(&io, org_2, &bea).await;
    assert_eq!(from_2, second, "the second Organisation's epoch goes out under its own Persona's key");
    assert_ne!(from_2, first, "and never under the first Persona's");
    assert_eq!(io.bound_endpoints_for_test().await, 2, "one endpoint per Persona");
}

// After a Persona's endpoint has closed, a send from that Persona binds the
// endpoint again and goes out under the Persona's Device key.
// verifies: LLR-d2pzzk, PR-tcd2ta
#[tokio::test(flavor = "multi_thread")]
async fn after_its_endpoint_closes_a_send_binds_it_again() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::shared().install();
    let tag = "endpoints-resend";
    let chain = FakeChain::new();
    let io = handle(tag, "a", &chain);
    let (org, founder) = common::handles::founded(&io, "resend").await;
    io.start_receiving().await.unwrap();
    let bob = admitted(&io, &chain, org, tag, "b").await;
    let pid = io.personas().await[0].persona_id.clone();
    let closed = io.endpoint_for_test(&pid).await.unwrap();
    let closed_sockets = closed.bound_sockets();
    closed.inner().close().await;

    let (from, report) = sent_from(&io, org, &bob).await;
    assert!(report.per_device.iter().all(|(_, sent)| matches!(sent, Delivery::Delivered)), "{report:?}");
    assert_eq!(from, founder, "under the sending Persona's Device key");
    let rebound = io.bound_endpoint_for_test(&pid).await.expect("the send bound the endpoint again");
    assert!(!rebound.inner().is_closed(), "the send bound the endpoint again");
    assert_ne!(rebound.bound_sockets(), closed_sockets, "a new endpoint, not the closed one");
    assert_eq!(io.bound_endpoints_for_test().await, 1, "the closed endpoint is not kept beside the new one");
}

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
    let tag = "endpoints-rebind-receiving";
    let chain = FakeChain::new();
    let io = handle(tag, "a", &chain);
    let (org, closing_key) = common::handles::founded(&io, "closing").await;
    let open_key = journey::propose_founding(&io, "open").await;
    io.start_receiving().await.unwrap();
    let bob = admitted(&io, &chain, org, tag, "b").await;
    let personas = io.personas().await;
    let pid_of = |key| personas.iter().find(|persona| persona.device_key == key).unwrap().persona_id.clone();
    let (closing, open) = (pid_of(closing_key), pid_of(open_key));
    let closed_sockets = io.endpoint_for_test(&closing).await.unwrap().bound_sockets();
    io.endpoint_for_test(&closing).await.unwrap().inner().close().await;

    let (from, report) = sent_from(&io, org, &bob).await;
    assert!(report.per_device.iter().all(|(_, sent)| matches!(sent, Delivery::Delivered)), "{report:?}");
    assert_eq!(from, closing_key);
    let rebound = io.bound_endpoint_for_test(&closing).await.expect("the send bound the endpoint again");
    assert_ne!(rebound.bound_sockets(), closed_sockets, "the send bound the endpoint again");

    let received = delivered_to_bound(&io, &closing, [0x79; 32], b"to the rebound").await;
    assert_eq!(received, b"to the rebound".to_vec(), "the rebound endpoint accepts");
    let received = delivered_to_bound(&io, &open, [0x7a; 32], b"to the open one").await;
    assert_eq!(received, b"to the open one".to_vec(), "the other Persona's endpoint still delivers");
    assert_eq!(io.bound_endpoints_for_test().await, 2, "one endpoint per Persona");
}

// Abnormal: while the receiver runs, the keychain refuses the rebind a send
// asks for. The refusal is the send's, for every recipient, nothing closed
// is kept, and the next send, once the keychain answers, binds the endpoint
// again and it accepts. The count of accept tasks stays exact: once both
// endpoints close, the receive channel ends.
// verifies: LLR-z35vbh, LLR-d2pzzk, PR-47aycj
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_rebind_while_the_receiver_runs_binds_and_accepts_on_the_next_send() {
    let _guard = LOCK.lock().await;
    let keychain = MemoryKeychain::shared();
    keychain.install();
    let tag = "endpoints-refused-rebind-receiving";
    let chain = FakeChain::new();
    let io = handle(tag, "a", &chain);
    let (org, closing_key) = common::handles::founded(&io, "closing").await;
    let open_key = journey::propose_founding(&io, "open").await;
    io.start_receiving().await.unwrap();
    let bob = admitted(&io, &chain, org, tag, "b").await;
    let personas = io.personas().await;
    let pid_of = |key| personas.iter().find(|persona| persona.device_key == key).unwrap().persona_id.clone();
    let (closing, open) = (pid_of(closing_key), pid_of(open_key));
    io.endpoint_for_test(&closing).await.unwrap().inner().close().await;

    keychain.refuse_as(Some(Refusal::Locked));
    let refused = io.send_organisation(org).await;
    keychain.refuse_as(None);
    let locked = TransportError::Keychain(Box::new(KeychainError::Locked { device_key: closing_key })).to_string();
    match refused.unwrap().per_device.as_slice() {
        [(recipient, Delivery::NotDelivered { reason: SendError::SenderEndpoint(reason) })] => {
            assert_eq!(*recipient, bob.personas().await[0].device_key, "the one recipient");
            assert_eq!(*reason, locked, "the keychain's refusal is the send's");
        }
        other => panic!("the send was not refused at the sender's endpoint: {other:?}"),
    }
    assert_eq!(io.bound_endpoints_for_test().await, 1, "the closed endpoint is not kept");

    let (from, report) = sent_from(&io, org, &bob).await;
    assert!(report.per_device.iter().all(|(_, sent)| matches!(sent, Delivery::Delivered)), "{report:?}");
    assert_eq!(from, closing_key, "from the rebound endpoint");
    assert_eq!(io.bound_endpoints_for_test().await, 2, "the send bound the endpoint again");
    let received = delivered_to_bound(&io, &closing, [0x7b; 32], b"after the refusal").await;
    assert_eq!(received, b"after the refusal".to_vec(), "the rebound endpoint accepts");

    io.bound_endpoint_for_test(&closing).await.unwrap().inner().close().await;
    io.bound_endpoint_for_test(&open).await.unwrap().inner().close().await;
    match tokio::time::timeout(NET, io.next_inbound_for_test()).await {
        Ok(Err(TransportError::EndpointClosed)) => {}
        Ok(Err(other)) => panic!("the channel ended for another reason: {other}"),
        Ok(Ok((from, _))) => panic!("a body from {from:?} arrived after every endpoint closed"),
        Err(_) => panic!("every endpoint closed, yet the receive channel waits: the accept-task count is off"),
    }
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
// handshake authenticated (the member-sender rule needs it).
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

// ---- A closed endpoint is not reused (PR-tcd2ta) ----------------------------

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
    let device_key = device_key_of(&io, &pid).await;
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
    let device_key = device_key_of(&io, &pid).await;
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

// ---- A bound endpoint's delivery ---------------------------------------------

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

// ---- A closing endpoint -----------------------------------------------------

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

// ---- A stalled connection (PR-d2p5y2) --------------------------------------

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

// ---- The endpoint's lifecycle ----------------------------------------------

// The endpoint is bound once per
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
    let expected = device_key_of(&io, &pid).await;

    let first = io.endpoint_for_test(&pid).await.unwrap();
    let first_sockets = first.bound_sockets();
    assert_eq!(first.device_key(), expected);
    assert!(!first_sockets.is_empty(), "no sockets bound");

    let second = io.endpoint_for_test(&pid).await.unwrap();
    assert_eq!(second.bound_sockets(), first_sockets, "the second call bound a new endpoint instead of returning the first");
    assert_eq!(second.device_key(), expected);
}

// A node holding more than one
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
    let first_key = device_key_of(&io, &first).await;
    let second_key = device_key_of(&io, &second).await;
    assert_ne!(first_key, second_key);

    let bound = io.endpoint_for_test(&second).await.unwrap().device_key();
    assert_eq!(bound, second_key, "the endpoint must authenticate the named persona's device key");
    assert_ne!(bound, first_key, "the endpoint bound the first persona's device key, not the named one");
}

// A node in Loopback mode binds
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
