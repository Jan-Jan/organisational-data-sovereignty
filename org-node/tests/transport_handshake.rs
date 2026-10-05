#![cfg(feature = "transport")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Two real iroh endpoints on loopback. A sends an admit envelope to B; B
//! authenticates A's device key from the connection and verifies against a
//! MockChain. Offline (no relay/internet — loopback direct connect only).
use std::time::Duration;

use org_node::chain::{MockChain, OrgState};
use org_node::{DeviceSeed, MemberSeed, OrgSecret};
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::sequence::SeqGuard;
use org_node::transport::endpoint::OrgEndpoint;
use org_node::transport::TransportMode;
use org_node::transport::wire::WireMessage;
use org_node::verify::{VerifyContext, verify_envelope_against_chain};
use org_node::{Epoch, OrgPublicKey, SequenceNumber, SignedDeltaEnvelope};
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::{Handle, MemberId, MemberLeaf, Name, Surname};

type Trie = OrgTrie<Blake3Hasher>;

// Inline genesis_and_admit — test_fixtures is lib-private to the crate.
// The admin's device is a keypair of its own: org-members refuses a leaf whose
// member key is also an enrolled device key (`DuplicateKey`).
// Returns (genesis_trie, new_trie, delta) where new_trie adds bob.
fn genesis_and_admit(
    admin: &SigningKeypair,
    admin_device: &SigningKeypair,
) -> (Trie, Trie, org_members::delta::Delta) {
    let admin_leaf = MemberLeaf::new(
        MemberId::new([1u8; 32]),
        Handle::parse("admin").unwrap(),
        admin.member_key(),
        Name::parse("Admin").unwrap(),
        Surname::parse("User").unwrap(),
        vec![admin_device.device_key()],
    )
    .unwrap();
    let genesis = Trie::genesis(vec![admin_leaf]).unwrap();

    let b_member = MemberSeed::from([2u8; 32]).signing_keypair();
    let b_device = DeviceSeed::from([3u8; 32]).signing_keypair();
    let b_leaf = MemberLeaf::new(
        MemberId::new([2u8; 32]),
        Handle::parse("bob").unwrap(),
        b_member.member_key(),
        Name::parse("Bob").unwrap(),
        Surname::parse("User").unwrap(),
        vec![b_device.device_key()],
    )
    .unwrap();
    let (new_trie, delta) = genesis.add_member(b_leaf).unwrap().recalculate().unwrap();
    (genesis, new_trie, delta)
}

// LLR-jn5jeh was removed from this annotation 2026-10-04 by review round 2:
// this test never calls `admit_member`, so it cannot observe how an admission
// dials its joiner. It is evidence for the endpoint handshake only.
// verifies: LLR-v873fx
#[tokio::test]
async fn delivers_and_verifies_admit_over_iroh() {
    // Admin keypair: MEMBER key signs the envelope.
    let admin = MemberSeed::from([1u8; 32]).signing_keypair();
    // A's iroh identity (device key = iroh EndpointId), enrolled in the trie
    // as the admin's device.
    let a_device = DeviceSeed::from([10u8; 32]).signing_keypair();
    // B's iroh identity.
    let b_device = DeviceSeed::from([11u8; 32]).signing_keypair();
    let org = OrgId::new([5u8; 20]);

    // Build genesis + admit-bob delta.
    let (genesis, new_trie, delta) = genesis_and_admit(&admin, &a_device);
    let new_root = new_trie.root_hash().unwrap();
    let env = SignedDeltaEnvelope::build(org, SequenceNumber::new(2), &delta, &admin).unwrap();
    let msg = WireMessage { envelope: env.clone(), org_secret: Some(OrgSecret::from([0xab; 32])), genesis_snapshot: None };

    // Bind both endpoints (relay disabled, loopback only).
    let ep_a = OrgEndpoint::bind(&a_device).await.unwrap();
    let ep_b = OrgEndpoint::bind(&b_device).await.unwrap();
    // Dial via inner().addr(). The note that used to sit here — that
    // node_addr_for_dial() "does not complete the dial in this setup" — was
    // true only because Loopback mode bound the wildcard, so bound_sockets()
    // reported `0.0.0.0`, which no peer can dial (PR-d4nye8). Both now report
    // the same loopback addresses, so either would serve; this one is left as
    // it was so the fix is not entangled with a change of API under test.
    let b_addr = ep_b.inner().addr();

    // B receives in a background task — spawn before A dials so accept() is
    // already waiting when A's connect() arrives.
    let recv_task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(10), ep_b.recv_one())
            .await
            .expect("recv_one timed out after 10 s")
    });

    // A sends to B's direct address.
    tokio::time::timeout(Duration::from_secs(10), ep_a.send(b_addr, &msg))
        .await
        .expect("send timed out after 10 s")
        .expect("send failed");

    let (remote_device, got) = recv_task
        .await
        .expect("recv task panicked")
        .expect("recv_one failed");

    // 1. The QUIC handshake authenticated A's device key.
    assert_eq!(
        remote_device.as_bytes(),
        a_device.device_key().as_bytes(),
        "authenticated remote device key must equal A's device key"
    );

    // 2. The WireMessage arrived intact.
    assert_eq!(got, msg, "received WireMessage must equal sent WireMessage");

    // 3. B verifies the received envelope against a MockChain seeded with
    //    the new root at epoch 2 (simulating an independent on-chain read).
    let mut chain = MockChain::new();
    chain.set(org, OrgState { root_hash: new_root, org_pub_key: OrgPublicKey::parse(&[0u8; 32]).unwrap(), epoch: Epoch::new(2) });
    let ctx = VerifyContext {
        expected_org_id: org,
        // admin.member_key().as_bytes() == admin.verifying_key().as_bytes()
        author_member_key: &admin.verifying_key(),
        seq_guard: SeqGuard::from_last_seen(SequenceNumber::new(1)),
        last_committed_epoch: Epoch::new(1),
    };
    let out = verify_envelope_against_chain(&genesis, &got.envelope, &ctx, &chain)
        .expect("verify_envelope_against_chain must succeed");

    assert_eq!(
        out.trie.root_hash().unwrap(),
        new_root,
        "committed root must equal the expected new root"
    );
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
        assert!(
            sock.ip().is_loopback(),
            "{who}: bound a non-loopback socket: {sock} (all bound: {bound:?})"
        );
    }

    // Both accessors: node_addr_for_dial() is what production hands a peer,
    // addr() is what the handshake test dials.
    for (label, addr_set) in [
        ("node_addr_for_dial()", ep.node_addr_for_dial()),
        ("addr()", ep.inner().addr()),
    ] {
        assert!(
            addr_set.ip_addrs().next().is_some(),
            "{who}: {label} advertised no dialable IP address"
        );
        for sock in addr_set.ip_addrs() {
            assert!(
                sock.ip().is_loopback(),
                "{who}: {label} advertised a non-loopback address: {sock}"
            );
        }
        assert!(
            addr_set.relay_urls().next().is_none(),
            "{who}: {label} advertised a relay home, so traffic can leave this \
             machine: {:?}",
            addr_set.relay_urls().collect::<Vec<_>>()
        );
    }

    let lookup = ep
        .inner()
        .address_lookup()
        .expect("endpoint must be open during the test");
    assert!(
        lookup.is_empty(),
        "{who}: an address-lookup service is configured, so this endpoint's \
         address is published off this machine ({} service(s))",
        lookup.len()
    );
}

/// Reproduction for PR-d4nye8. `TransportMode::Loopback` is documented as
/// binding `127.0.0.1:0` with relay disabled, and the three targets that use
/// it describe themselves as offline. Before the fix the builder named no bind
/// address at all, so iroh bound the wildcard (`0.0.0.0` and `[::]`) and
/// `Endpoint::addr()` advertised the host's LAN address — the one the dialling
/// side then used. Every "loopback" exchange left the machine over whatever
/// interface the host happened to have, which is what times out here.
///
/// This asserts the contract directly rather than through a ten-second
/// timeout, so it fails for the reason it names on any host and does not
/// depend on the network the machine is attached to.
// verifies: REQ-db6s7q, LLR-wwunf4, LLR-wx77j5
#[tokio::test]
async fn loopback_mode_binds_and_advertises_loopback_only() {
    let device = DeviceSeed::from([42u8; 32]).signing_keypair();
    let ep = OrgEndpoint::bind(&device).await.unwrap();
    assert_confined_to_this_machine(&ep, "the endpoint");

    // LLR-wx77j5's other half: the identity reported is the endpoint's own,
    // and the sockets are the ones it bound. Added 2026-10-05 by review round
    // 8, which measured that reporting another identity left this test green.
    let dial = ep.node_addr_for_dial();
    assert_eq!(dial.id, ep.inner().id(), "node_addr_for_dial must report this endpoint's identity");
    assert_eq!(dial.id.as_bytes(), device.device_key().as_bytes());
    let mut advertised: Vec<_> = dial.ip_addrs().copied().collect();
    let mut bound = ep.inner().bound_sockets();
    advertised.sort();
    bound.sort();
    assert_eq!(advertised, bound, "and the sockets it currently has bound");
}

/// Robustness case for REQ-db6s7q. Class C requires abnormal-input cases, and
/// the input surface of `bind` is one keypair — so the abnormal conditions
/// that exist are about repetition and collision rather than about malformed
/// arguments, and they are the ones under which a binding property plausibly
/// degrades: many endpoints alive at once in one process, and two endpoints
/// claiming the SAME device identity, which the production code never does and
/// which a confused caller or a duplicated persona could.
///
/// It also pins the asymmetry REQ-2wzfzv states between the two address
/// families: exactly one IPv4 loopback socket, which is required, and whatever
/// IPv6 socket exists must be loopback. The other half of that requirement —
/// that an endpoint still comes up on a host with no IPv6 at all — cannot be
/// driven from here, because nothing in a test can take `[::1]` away from the
/// machine it runs on. That is a stated gap, not a silent one.
// verifies: REQ-db6s7q, REQ-2wzfzv, LLR-wwunf4, LLR-6adc99
#[tokio::test]
async fn loopback_mode_holds_under_repeated_and_colliding_binds() {
    const COUNT: u8 = 8;
    let mut endpoints = Vec::new();
    for i in 0..COUNT {
        // The last two share one seed: same ed25519 key, so the same
        // EndpointId bound twice at once.
        let seed = if i >= COUNT - 2 { [99u8; 32] } else { [i; 32] };
        let ep = OrgEndpoint::bind(&DeviceSeed::from(seed).signing_keypair())
            .await
            .expect("Loopback bind must succeed without depending on the host's network");
        endpoints.push(ep);
    }

    let mut v4_ports = Vec::new();
    for (i, ep) in endpoints.iter().enumerate() {
        assert_confined_to_this_machine(ep, &format!("endpoint {i}"));

        let bound = ep.inner().bound_sockets();
        let v4: Vec<_> = bound.iter().filter(|s| s.is_ipv4()).collect();
        assert_eq!(
            v4.len(),
            1,
            "endpoint {i} must have exactly one IPv4 loopback socket, the required \
             half of the bind (REQ-2wzfzv); got {v4:?} from {bound:?}"
        );
        v4_ports.push(v4[0].port());
    }

    let mut distinct = v4_ports.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        v4_ports.len(),
        "every endpoint must hold its own port; got {v4_ports:?}"
    );
}

// Relocated 2026-10-03 from the `#[cfg(test)]` module in
// `org-node/src/transport/endpoint.rs`, so the annotation sits under
// `test_paths`. The assertion is unchanged; the Loopback mode is explicit
// because the bind in the original went through the default `bind`.
// verifies: REQ-ztdza4, REQ-xa6smf, LLR-ygn78w
#[tokio::test]
async fn endpoint_id_equals_device_key() {
    let device = DeviceSeed::from([7u8; 32]).signing_keypair();
    let ep = OrgEndpoint::bind_with_mode(&device, TransportMode::Loopback)
        .await
        .expect("bind");
    // The iroh EndpointId bytes must equal the device key bytes: the endpoint's
    // identity IS the device's key, which is what makes a completed handshake
    // evidence of device-key custody.
    assert_eq!(ep.inner().id().as_bytes(), device.device_key().as_bytes());
    assert_eq!(ep.device_key().as_bytes(), device.device_key().as_bytes());

    // A different device yields a different endpoint identity.
    let other = DeviceSeed::from([8u8; 32]).signing_keypair();
    let ep2 = OrgEndpoint::bind_with_mode(&other, TransportMode::Loopback)
        .await
        .expect("bind");
    assert_ne!(ep2.inner().id().as_bytes(), ep.inner().id().as_bytes());
}

// A peer that never stops sending must not cause unbounded allocation in the
// receiver: `recv_one` reads at most MAX_FRAME + 4 bytes and then fails the
// stream, rather than growing a buffer to whatever the peer chooses to write.
// The bytes below are deliberately not a valid frame — what is under test is
// the read limit, which is reached before anything is decoded.
// verifies: REQ-eg5j8u, LLR-k2y6nn
#[tokio::test(flavor = "multi_thread")]
async fn a_stream_longer_than_the_read_bound_is_refused_rather_than_buffered() {
    use org_node::transport::{ALPN, MAX_FRAME};

    let receiver_kp = DeviceSeed::from([0x71u8; 32]).signing_keypair();
    let sender_kp = DeviceSeed::from([0x72u8; 32]).signing_keypair();
    let receiver = OrgEndpoint::bind_with_mode(&receiver_kp, TransportMode::Loopback)
        .await
        .expect("bind receiver");
    let sender = OrgEndpoint::bind_with_mode(&sender_kp, TransportMode::Loopback)
        .await
        .expect("bind sender");
    let receiver_addr = receiver.node_addr_for_dial();

    let recv_task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(20), receiver.recv_one())
            .await
            .expect("recv_one timed out")
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Write one byte more than `recv_one` is willing to read.
    let conn = sender
        .inner()
        .connect(receiver_addr, ALPN)
        .await
        .expect("connect");
    let (mut send, _recv) = conn.open_bi().await.expect("open_bi");
    let oversize = vec![0u8; MAX_FRAME + 5];
    // A peer is free to ignore the protocol; write errors here are that peer's
    // problem, and the receiver's verdict below is what this test is about.
    let _ = send.write_all(&oversize).await;
    let _ = send.finish();

    let result = recv_task.await.unwrap();
    // The verdict must be the READ's — the stream exceeded what `recv_one` is
    // willing to take — and not the decoder's. `is_err()` alone would be
    // satisfied either way: with no read bound at all the bytes would still be
    // refused, by `decode_body` as FrameTooLarge, after being buffered in
    // full. Naming the variant is what makes the bound itself the thing under
    // test.
    match result {
        Err(org_node::transport::TransportError::Stream(_)) => {}
        other => panic!("expected the read to be refused as a stream error, got {other:?}"),
    }
}

// `recv_one` throws the 4-byte length prefix away without reading it, and a
// stream shorter than four bytes is decoded whole. The prefix is vestigial:
// the QUIC stream's end delimits the frame, and the bound is the read limit
// (LLR-k2y6nn) and `decode_body`'s. Stated by no requirement until review
// round 8, which measured that checking the prefix left the gate green. This
// test states what the code does: a well-formed body behind a prefix that
// disagrees with it is accepted, and (review round 9) a stream of fewer than
// four bytes is handed to the decoder whole, which refuses it as Malformed.
// verifies: LLR-2smrvx
#[tokio::test(flavor = "multi_thread")]
async fn the_length_prefix_is_not_checked_against_the_body() {
    use org_node::test_fixtures::admit_member_delta;
    use org_node::transport::wire::{encode_frame, WireMessage};
    use org_node::transport::ALPN;

    let receiver = OrgEndpoint::bind_with_mode(&DeviceSeed::from([0x73u8; 32]).signing_keypair(), TransportMode::Loopback)
        .await
        .expect("bind receiver");
    let sender = OrgEndpoint::bind_with_mode(&DeviceSeed::from([0x74u8; 32]).signing_keypair(), TransportMode::Loopback)
        .await
        .expect("bind sender");
    let receiver_addr = receiver.node_addr_for_dial();

    let admin = MemberSeed::from([1u8; 32]).signing_keypair();
    let (delta, _) = admit_member_delta(&admin);
    let env = org_node::SignedDeltaEnvelope::build(org_node::OrgId::new([5u8; 20]), SequenceNumber::new(2), &delta, &admin).unwrap();
    let msg = WireMessage { envelope: env, org_secret: None, genesis_snapshot: None };
    let mut framed = encode_frame(&msg).unwrap();
    framed[0..4].copy_from_slice(&0u32.to_le_bytes());

    let recv_task = tokio::spawn(async move {
        let got = tokio::time::timeout(Duration::from_secs(20), receiver.recv_one())
            .await
            .expect("recv_one timed out");
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
        tokio::time::timeout(Duration::from_secs(20), receiver.recv_one())
            .await
            .expect("recv_one timed out")
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let conn = sender.inner().connect(receiver_addr, ALPN).await.expect("connect");
    let (mut send, _recv) = conn.open_bi().await.expect("open_bi");
    send.write_all(&[1u8, 2, 3]).await.expect("write");
    send.finish().expect("finish");
    match short_task.await.unwrap() {
        Err(org_node::transport::TransportError::Malformed) => {}
        other => panic!("a short stream must reach the decoder and be refused as Malformed, got {other:?}"),
    }
}
