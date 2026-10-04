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
use org_node::SignedDeltaEnvelope;
use org_node::chain::{MockChain, OrgState};
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
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

    let b_member = SigningKeypair::from_seed([2u8; 32]);
    let b_device = SigningKeypair::from_seed([3u8; 32]);
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

#[tokio::test]
async fn delivers_and_verifies_admit_over_relay_by_id() {
    // Admin MEMBER key signs the envelope.
    let admin = SigningKeypair::from_seed([1u8; 32]);
    // A's iroh identity, enrolled in the trie as the admin's device.
    let a_device = SigningKeypair::from_seed([10u8; 32]);
    let b_device = SigningKeypair::from_seed([11u8; 32]); // B's iroh identity
    let org = OrgId::new([5u8; 20]);

    let (genesis, new_trie, delta) = genesis_and_admit(&admin, &a_device);
    let new_root = new_trie.root_hash().unwrap();
    let env = SignedDeltaEnvelope::build(org, 2, &delta, &admin).unwrap();
    let msg = WireMessage {
        envelope: env.clone(),
        org_secret: Some([0xab; 32]),
        genesis_snapshot: None,
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
        a_device.device_key().as_bytes(),
        "authenticated remote device key must equal A's device key"
    );
    // 2. The WireMessage arrived intact over the relay.
    assert_eq!(got, msg, "received WireMessage must equal sent WireMessage");

    // 3. B verifies the received envelope against a MockChain seeded with the
    //    new root at epoch 2 (independent on-chain read).
    let mut chain = MockChain::new();
    chain.set(
        org,
        OrgState { root_hash: new_root, org_pub_key: [0u8; 32], epoch: 2 },
    );
    let ctx = VerifyContext {
        expected_org_id: org,
        author_member_key: &admin.verifying_key(),
        seq_guard: SeqGuard::from_last_seen(1),
        last_committed_epoch: 1,
    };
    let out = verify_envelope_against_chain(&genesis, &got.envelope, &ctx, &chain)
        .expect("verify_envelope_against_chain must succeed");
    assert_eq!(out.trie.root_hash().unwrap(), new_root, "committed root mismatch");
    assert_eq!(out.epoch, 2, "committed epoch must be 2");
}
