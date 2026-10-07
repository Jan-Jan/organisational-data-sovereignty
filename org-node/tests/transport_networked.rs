#![cfg(all(feature = "transport", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Two real iroh endpoints that find and reach each other ONLY via an
//! in-process relay + in-memory address lookup, dialled purely by EndpointId
//! (`send_to_id`). This is the hermetic stand-in for the two-laptop
//! `TransportMode::Networked` path: it exercises the Networked builder, the
//! relay datapath, dial-by-EndpointId resolution, the frame codec, and the
//! authenticated receive. It does NOT (and cannot, on one machine) exercise
//! real cross-NAT hole-punching — that is the only thing left to the manual
//! two-laptop run.
use std::time::Duration;

use iroh::address_lookup::MemoryLookup;
use org_node::Envelope;
use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, SequenceNumber};
use org_node::chain::{MockChain, OrgState};
use org_node::ids::OrgId;
use org_node::keys::{SigningKeypair, X25519Keypair};
use org_node::sequence::SeqGuard;
use org_node::transport::endpoint::OrgEndpoint;
use org_node::transport::wire::WireMessage;
use org_node::verify::{VerifyContext, verify_envelope_against_chain};
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::{Handle, MemberId, MemberLeaf, Name, Surname};

type Trie = OrgTrie<Blake3Hasher>;

// Inline genesis_and_admit. Written before `test_fixtures` was reachable from
// an integration test; since 2026-09-09 it is `pub mod` under `test-support`,
// which this target already requires, so this helper is kept inline only to
// leave the test unchanged. The admin's device is a keypair of its own:
// org-members refuses a leaf whose member key is also an enrolled device key
// (`DuplicateKey`). Returns (genesis_trie, new_trie_with_bob, delta).
fn genesis_and_admit(
    admin: &X25519Keypair,
    admin_device: &SigningKeypair,
) -> (Trie, Trie, org_members::delta::Delta) {
    let admin_leaf = MemberLeaf::new(
        MemberId::new([1u8; 32]),
        Handle::parse("admin").unwrap(),
        admin.member_key().expect("valid key"),
        Name::parse("Admin").unwrap(),
        Surname::parse("User").unwrap(),
        vec![admin_device.device_key().unwrap()],
    )
    .unwrap();
    let genesis = Trie::genesis(vec![admin_leaf]).unwrap();

    let b_member = MemberSeed::from([2u8; 32]).x25519_keypair();
    let b_device = DeviceSeed::from([3u8; 32]).signing_keypair();
    let b_leaf = MemberLeaf::new(
        MemberId::new([2u8; 32]),
        Handle::parse("bob").unwrap(),
        b_member.member_key().expect("valid key"),
        Name::parse("Bob").unwrap(),
        Surname::parse("User").unwrap(),
        vec![b_device.device_key().unwrap()],
    )
    .unwrap();
    let (new_trie, delta) = genesis.add_member(b_leaf).unwrap().recalculate().unwrap();
    (genesis, new_trie, delta)
}

// LLR-jn5jeh was removed from this annotation 2026-10-04 by review round 2:
// this is the dial-by-id case, which is what LLR-jn5jeh's Loopback clause
// excludes, and it never calls `admit_member`.
// verifies: LLR-v873fx
#[tokio::test]
async fn delivers_and_verifies_admit_over_relay_by_id() {
    let member = MemberSeed::from([1u8; 32]).x25519_keypair();
    // A's iroh identity, enrolled in the trie as the admin's device.
    let a_device = DeviceSeed::from([10u8; 32]).signing_keypair();
    let b_device = DeviceSeed::from([11u8; 32]).signing_keypair(); // B's iroh identity
    let org = OrgId::new([5u8; 20]);

    let (genesis, new_trie, delta) = genesis_and_admit(&member, &a_device);
    let new_root = new_trie.root_hash().unwrap();
    let env = Envelope::build(org, SequenceNumber::new(2), &delta).unwrap();
    let msg = WireMessage::OrgInformation {
        envelope: env.clone(),
        record_snapshot: vec![],
        org_private_key: OrgPrivateKey::from([0xab; 32]),
    };

    // In-process relay + shared in-memory address lookup. The Server is held
    // for the test's lifetime (dropping it stops the relay).
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server()
        .await
        .expect("spawn in-process relay");
    let lookup = MemoryLookup::new();

    let ep_a = OrgEndpoint::bind_with_relay(&a_device, relay_map.clone(), lookup.clone())
        .await
        .expect("bind A against relay");
    let ep_b = OrgEndpoint::bind_with_relay(&b_device, relay_map.clone(), lookup.clone())
        .await
        .expect("bind B against relay");

    // Register both with the relay so a relay home is assigned, then publish
    // each endpoint's addr (including its relay home) into the shared lookup
    // so dial-by-EndpointId can resolve.
    tokio::time::timeout(Duration::from_secs(30), ep_a.inner().online())
        .await
        .expect("A did not come online within 30 s");
    tokio::time::timeout(Duration::from_secs(30), ep_b.inner().online())
        .await
        .expect("B did not come online within 30 s");
    lookup.add_endpoint_info(ep_a.inner().addr());
    lookup.add_endpoint_info(ep_b.inner().addr());

    let b_id = ep_b.inner().id();

    // B receives in a background task — spawn before A dials so accept() is
    // already waiting.
    let recv_task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(30), ep_b.recv_one())
            .await
            .expect("recv_one timed out after 30 s")
    });

    // A dials B PURELY BY EndpointId — resolution + relay routing are what we
    // are exercising here.
    tokio::time::timeout(Duration::from_secs(30), ep_a.send_to_id(b_id, &msg))
        .await
        .expect("send_to_id timed out after 30 s")
        .expect("send_to_id failed");

    let (remote_device, got) = recv_task
        .await
        .expect("recv task panicked")
        .expect("recv_one failed");

    // 1. QUIC handshake authenticated A's device key.
    assert_eq!(
        remote_device.as_bytes(),
        a_device.device_key().unwrap().as_bytes(),
        "authenticated remote device key must equal A's device key"
    );
    // 2. The WireMessage arrived intact over the relay.
    assert_eq!(got, msg, "received WireMessage must equal sent WireMessage");

    // 3. B verifies the received envelope against a MockChain seeded with the
    //    new root at epoch 2 (independent on-chain read).
    let mut chain = MockChain::new();
    chain.set(
        org,
        OrgState { root_hash: new_root, org_pub_key: OrgPrivateKey::from([9u8; 32]).x25519_keypair().org_public_key().unwrap(), epoch: Epoch::new(2) },
    );
    let ctx = VerifyContext {
        expected_org_id: org,
        seq_guard: SeqGuard::from_last_seen(SequenceNumber::new(1)),
        last_committed_epoch: Epoch::new(1),
    };
    // The received message equals `msg` (2.), so its Envelope is `env`; only
    // Organisation information holds one (LLR-js9dsu).
    let out = verify_envelope_against_chain(&genesis, &env, &ctx, &chain)
        .expect("verify_envelope_against_chain must succeed");
    assert_eq!(out.trie.root_hash().unwrap(), new_root, "committed root mismatch");
    assert_eq!(out.epoch, Epoch::new(2), "committed epoch must be 2");
}
