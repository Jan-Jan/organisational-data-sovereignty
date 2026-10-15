#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Two real iroh endpoints on loopback. A sends an admit envelope to B; B
//! authenticates A's device key from the connection and verifies against a
//! chain state passed as a value. Offline (no relay/internet — loopback
//! direct connect only).
//!
//! *Moved 2026-10-13 from org-node's `tests/transport_handshake.rs` (S4,
//! task T10), with the framing test of its `tests/wire_frame_bound.rs`:*
//! every endpoint is bound from a keychain item, org-io's in-memory one
//! (`bind_from_keychain`), where org-node bound from a key pair; the
//! assertions are unchanged. The trie fixtures are org-node's
//! (`test_fixtures`), where the original built them inline from
//! org-members.
use std::time::Duration;

use org_node::chain::OrgState;
use org_node::ids::OrgId;
use org_node::sequence::SeqGuard;
use org_node::test_fixtures::{admin_device, admit_member_delta, genesis_trie, ADMIN_DEVICE_SEED};
use org_node::verify::{verify_envelope_against_chain, VerifyContext};
use org_node::wire::{decode_message, WireMessage};
use org_node::{Envelope, Epoch, MemberSeed, OrgPrivateKey, SequenceNumber};
use org_io::test_support::MemoryKeychain;
use org_io::transport::{encode_frame, BindPlan, OrgEndpoint, TransportError, ALPN, MAX_FRAME};

mod common;
use common::handles::{bind_seed, keep_device_seed};

// verifies: LLR-v873fx
#[tokio::test]
async fn delivers_and_verifies_admit_over_iroh() {
    MemoryKeychain::shared().install();
    let member = MemberSeed::from([1u8; 32]).x25519_keypair();
    let org = OrgId::new([5u8; 20]);

    // Genesis + admit-bob delta; A's Device is the admin's, enrolled in the
    // trie (`admin_device`, seed `ADMIN_DEVICE_SEED`).
    let genesis = genesis_trie(&member, &admin_device());
    let (delta, new_trie) = admit_member_delta(&member);
    let new_root = new_trie.root_hash().unwrap();
    let env = Envelope::build(org, SequenceNumber::new(2), &delta).unwrap();
    let msg = WireMessage::OrgInformation { envelope: env.clone(), record_snapshot: vec![], org_private_key: OrgPrivateKey::from([0xab; 32]) };

    // Bind both endpoints (relay disabled, loopback only).
    let ep_a = bind_seed(ADMIN_DEVICE_SEED).await;
    let ep_b = bind_seed([11u8; 32]).await;
    let a_key = ep_a.device_key();
    let b_addr = ep_b.inner().addr();

    // B receives in a background task — spawn before A dials so accept() is
    // already waiting when A's connect() arrives.
    let recv_task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(10), ep_b.recv_one())
            .await
            .expect("recv_one timed out after 10 s")
    });

    tokio::time::timeout(Duration::from_secs(10), ep_a.send(b_addr, &msg))
        .await
        .expect("send timed out after 10 s")
        .expect("send failed");

    let (remote_device, got) = recv_task.await.expect("recv task panicked").expect("recv_one failed");

    // 1. The QUIC handshake authenticated A's device key.
    assert_eq!(remote_device, a_key, "authenticated remote device key must equal A's device key");
    assert_eq!(remote_device, admin_device().device_key().unwrap());

    // 2. The WireMessage arrived intact.
    assert_eq!(got, msg, "received WireMessage must equal sent WireMessage");

    // 3. B verifies the received envelope against the chain state holding
    //    the new root at epoch 2 (an independent on-chain read, as a value).
    let chain_state = Some(OrgState {
        root_hash: new_root,
        org_pub_key: OrgPrivateKey::from([9u8; 32]).x25519_keypair().org_public_key().unwrap(),
        epoch: Epoch::new(2),
    });
    let ctx = VerifyContext {
        expected_org_id: org,
        seq_guard: SeqGuard::from_last_seen(SequenceNumber::new(1)),
        last_committed_epoch: Epoch::new(1),
    };
    let out = verify_envelope_against_chain(&genesis, &env, &ctx, chain_state).expect("verify_envelope_against_chain must succeed");
    assert_eq!(out.trie.root_hash().unwrap(), new_root, "committed root must equal the expected new root");
    assert_eq!(out.epoch, Epoch::new(2), "committed epoch must be 2");
}

/// Assert the whole of what Loopback mode claims, for one endpoint.
///
/// Split out because the two tests below need identical checks and because
/// **the obvious assertion here does not discriminate.** `node_addr_for_dial()`
/// is `bound_sockets().map(TransportAddr::Ip)`, so matching its entries against
/// `TransportAddr::Relay` can never fire whatever the relay configuration is:
/// an arm that cannot execute asserts nothing. That was caught by review round
/// 2, which swapped `presets::Minimal` + `RelayMode::Disabled` for
/// `presets::N0` — full n0 relay, Pkarr publishing, DNS lookup — and watched
/// both tests stay green.
///
/// So the two off-machine escape routes are checked where they are actually
/// observable:
///
/// - **Address publication** — `address_lookup()` is empty. `presets::N0`
///   installs a Pkarr publisher and a DNS lookup, which would publish this
///   endpoint's address to `iroh.link`; `presets::Minimal` installs neither.
///   Synchronous, no network, and it reddens on the round-2 mutation.
/// - **Relay home** — `addr()` carries no relay URL.
///
/// What this still does not cover is recorded as a gap in the verification
/// record: iroh exposes no accessor for an `Endpoint`'s relay map, so a
/// mutation that enabled a relay while leaving address lookup empty would be
/// caught only once a relay home was actually acquired, which needs the
/// network this test refuses to depend on.
fn assert_confined_to_this_machine(ep: &OrgEndpoint, who: &str) {
    let bound = ep.inner().bound_sockets();
    assert!(!bound.is_empty(), "{who}: bound no socket at all");
    for sock in &bound {
        assert!(sock.ip().is_loopback(), "{who}: bound a non-loopback socket: {sock} (all bound: {bound:?})");
    }

    // Both accessors: node_addr_for_dial() is what production hands a peer,
    // addr() is what the handshake test dials.
    for (label, addr_set) in [("node_addr_for_dial()", ep.node_addr_for_dial()), ("addr()", ep.inner().addr())] {
        assert!(addr_set.ip_addrs().next().is_some(), "{who}: {label} advertised no dialable IP address");
        for sock in addr_set.ip_addrs() {
            assert!(sock.ip().is_loopback(), "{who}: {label} advertised a non-loopback address: {sock}");
        }
        assert!(
            addr_set.relay_urls().next().is_none(),
            "{who}: {label} advertised a relay home, so traffic can leave this machine: {:?}",
            addr_set.relay_urls().collect::<Vec<_>>()
        );
    }

    let lookup = ep.inner().address_lookup().expect("endpoint must be open during the test");
    assert!(
        lookup.is_empty(),
        "{who}: an address-lookup service is configured, so this endpoint's address is published off this machine ({} service(s))",
        lookup.len()
    );
}

/// Reproduction for org-node's loopback-timeout report (resolved there
/// before the move). `TransportMode::Loopback` binds `127.0.0.1:0` with relay disabled;
/// before that fix the builder named no bind address at all, so iroh bound
/// the wildcard and `Endpoint::addr()` advertised the host's LAN address.
/// This asserts the contract directly rather than through a timeout.
// verifies: REQ-db6s7q, LLR-wwunf4, LLR-wx77j5
#[tokio::test]
async fn loopback_mode_binds_and_advertises_loopback_only() {
    MemoryKeychain::shared().install();
    let device_key = keep_device_seed([42u8; 32]);
    let ep = OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await.unwrap();
    assert_confined_to_this_machine(&ep, "the endpoint");

    // LLR-wx77j5's other half: the identity reported is the endpoint's own,
    // and the sockets are the ones it bound. Added 2026-10-05 by review round
    // 8, which measured that reporting another identity left this test green.
    let dial = ep.node_addr_for_dial();
    assert_eq!(dial.id, ep.inner().id(), "node_addr_for_dial must report this endpoint's identity");
    assert_eq!(dial.id.as_bytes(), device_key.as_bytes());
    let mut advertised: Vec<_> = dial.ip_addrs().copied().collect();
    let mut bound = ep.inner().bound_sockets();
    advertised.sort();
    bound.sort();
    assert_eq!(advertised, bound, "and the sockets it currently has bound");
}

/// Robustness case for REQ-db6s7q: many endpoints alive at once in one
/// process, and two endpoints claiming the SAME device identity. It also
/// pins the asymmetry REQ-2wzfzv states between the two address families:
/// exactly one IPv4 loopback socket, which is required, and whatever IPv6
/// socket exists must be loopback. That an endpoint still comes up on a host
/// with no IPv6 at all cannot be driven from here: a stated gap.
// verifies: REQ-db6s7q, REQ-2wzfzv, LLR-wwunf4, LLR-6adc99
#[tokio::test]
async fn loopback_mode_holds_under_repeated_and_colliding_binds() {
    MemoryKeychain::shared().install();
    const COUNT: u8 = 8;
    let mut endpoints = Vec::new();
    for index in 0..COUNT {
        // The last two share one seed: same ed25519 key, so the same
        // EndpointId bound twice at once.
        let seed = if index >= COUNT - 2 { [99u8; 32] } else { [0x90 + index; 32] };
        let ep = OrgEndpoint::bind_from_keychain(&keep_device_seed(seed), BindPlan::Loopback)
            .await
            .expect("Loopback bind must succeed without depending on the host's network");
        endpoints.push(ep);
    }

    let mut v4_ports = Vec::new();
    for (index, ep) in endpoints.iter().enumerate() {
        assert_confined_to_this_machine(ep, &format!("endpoint {index}"));
        let bound = ep.inner().bound_sockets();
        let v4: Vec<_> = bound.iter().filter(|socket| socket.is_ipv4()).collect();
        assert_eq!(
            v4.len(),
            1,
            "endpoint {index} must have exactly one IPv4 loopback socket, the required half of the bind (REQ-2wzfzv); got {v4:?} from {bound:?}"
        );
        v4_ports.push(v4[0].port());
    }

    let mut distinct = v4_ports.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(distinct.len(), v4_ports.len(), "every endpoint must hold its own port; got {v4_ports:?}");
}

// verifies: LLR-ygn78w
#[tokio::test]
async fn endpoint_id_equals_device_key() {
    MemoryKeychain::shared().install();
    let device_key = keep_device_seed([7u8; 32]);
    let ep = OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await.expect("bind");
    // The iroh EndpointId bytes must equal the device key bytes: the endpoint's
    // identity IS the device's key, which is what makes a completed handshake
    // evidence of device-key custody.
    assert_eq!(ep.inner().id().as_bytes(), device_key.as_bytes());
    assert_eq!(ep.device_key(), device_key);

    // A different device yields a different endpoint identity.
    let ep2 = OrgEndpoint::bind_from_keychain(&keep_device_seed([8u8; 32]), BindPlan::Loopback).await.expect("bind");
    assert_ne!(ep2.inner().id().as_bytes(), ep.inner().id().as_bytes());
}

// A peer that never stops sending must not cause unbounded allocation in the
// receiver: `recv_one` reads at most MAX_FRAME + 4 bytes and then fails the
// stream, rather than growing a buffer to whatever the peer chooses to write.
// The bytes below are deliberately not a valid frame — what is under test is
// the read limit, which is reached before anything is decoded.
// *Moved 2026-10-13 (S4, task T10):* the annotation also named org-node's
// requirement on the Wire message's bound, which org-io cannot cite;
// LLR-k2y6nn is org-io's now, under REQ-szt6ea.
// verifies: LLR-k2y6nn
#[tokio::test(flavor = "multi_thread")]
async fn a_stream_longer_than_the_read_bound_is_refused_rather_than_buffered() {
    MemoryKeychain::shared().install();
    let receiver = bind_seed([0x71u8; 32]).await;
    let sender = bind_seed([0x72u8; 32]).await;
    let receiver_addr = receiver.node_addr_for_dial();

    let recv_task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(20), receiver.recv_one()).await.expect("recv_one timed out")
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Write one byte more than `recv_one` is willing to read.
    let conn = sender.inner().connect(receiver_addr, ALPN).await.expect("connect");
    let (mut send, _recv) = conn.open_bi().await.expect("open_bi");
    let oversize = vec![0u8; MAX_FRAME + 5];
    // A peer is free to ignore the protocol; write errors here are that peer's
    // problem, and the receiver's verdict below is what this test is about.
    let _ = send.write_all(&oversize).await;
    let _ = send.finish();

    // The verdict must be the READ's, not the decoder's: naming the variant
    // is what makes the bound itself the thing under test.
    match recv_task.await.unwrap() {
        Err(TransportError::Stream(_)) => {}
        other => panic!("expected the read to be refused as a stream error, got {other:?}"),
    }
}

// `recv_one` throws the 4-byte length prefix away without reading it, and a
// stream shorter than four bytes is decoded whole: a well-formed body behind
// a prefix that disagrees with it is accepted, and a stream of fewer than
// four bytes is handed to the decoder whole, which refuses it as Malformed.
// verifies: LLR-2smrvx
#[tokio::test(flavor = "multi_thread")]
async fn the_length_prefix_is_not_checked_against_the_body() {
    MemoryKeychain::shared().install();
    let receiver = bind_seed([0x73u8; 32]).await;
    let sender = bind_seed([0x74u8; 32]).await;
    let receiver_addr = receiver.node_addr_for_dial();

    let admin = MemberSeed::from([1u8; 32]).x25519_keypair();
    let (delta, _) = admit_member_delta(&admin);
    let env = Envelope::build(OrgId::new([5u8; 20]), SequenceNumber::new(2), &delta).unwrap();
    let msg = WireMessage::OrgInformation { envelope: env, record_snapshot: vec![], org_private_key: OrgPrivateKey::from([0xab; 32]) };
    let mut framed = encode_frame(&msg).unwrap();
    framed[0..4].copy_from_slice(&0u32.to_le_bytes());

    let recv_task = tokio::spawn(async move {
        let got = tokio::time::timeout(Duration::from_secs(20), receiver.recv_one()).await.expect("recv_one timed out");
        (receiver, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let conn = sender.inner().connect(receiver_addr, ALPN).await.expect("connect");
    let (mut send, _recv) = conn.open_bi().await.expect("open_bi");
    send.write_all(&framed).await.expect("write");
    send.finish().expect("finish");
    let (receiver, got) = recv_task.await.unwrap();
    let (_sender_key, back) = got.expect("a prefix of zero over a non-empty body is accepted");
    assert_eq!(back, msg);

    // Three bytes: no prefix to strip, so they are decoded as they stand.
    let receiver_addr = receiver.node_addr_for_dial();
    let short_task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(20), receiver.recv_one()).await.expect("recv_one timed out")
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let conn = sender.inner().connect(receiver_addr, ALPN).await.expect("connect");
    let (mut send, _recv) = conn.open_bi().await.expect("open_bi");
    send.write_all(&[1u8, 2, 3]).await.expect("write");
    send.finish().expect("finish");
    match short_task.await.unwrap() {
        Err(TransportError::Malformed) => {}
        other => panic!("a short stream must reach the decoder and be refused as Malformed, got {other:?}"),
    }
}

// The transport's framing, `len(u32 LE) ‖ encode_message(msg)`.
// *Moved 2026-10-13 from org-node's `tests/wire_frame_bound.rs` (S4, task
// T10):* the annotation also named org-node's requirement on the Wire
// message's bound and its round-trip item, which org-io cannot cite; both
// stay verified in org-node by the codec's own tests. LLR-fa7jt8 is
// org-io's now, under REQ-szt6ea.
// verifies: LLR-fa7jt8
#[test]
fn frame_round_trips() {
    let admin = MemberSeed::from([1u8; 32]).x25519_keypair();
    let (delta, _) = admit_member_delta(&admin);
    let msg = WireMessage::OrgInformation {
        envelope: Envelope::build(OrgId::new([3u8; 20]), SequenceNumber::new(2), &delta).unwrap(),
        record_snapshot: vec![1, 2, 3],
        org_private_key: OrgPrivateKey::from([9u8; 32]),
    };
    let framed = encode_frame(&msg).unwrap();
    // strip the 4-byte length prefix
    let len = u32::from_le_bytes(framed[0..4].try_into().unwrap()) as usize;
    assert_eq!(len, framed.len() - 4);
    let back = decode_message(&framed[4..]).unwrap();
    assert_eq!(back, msg);
}
