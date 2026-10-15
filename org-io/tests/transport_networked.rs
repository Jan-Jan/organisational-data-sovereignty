#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Two real iroh endpoints that find and reach each other ONLY via an
//! in-process relay + in-memory address lookup, dialled purely by EndpointId
//! (`send_to_id`). This is the hermetic stand-in for the two-laptop
//! `TransportMode::Networked` path: it exercises the relay datapath,
//! dial-by-EndpointId resolution, the frame codec, and the authenticated
//! receive. It does NOT (and cannot, on one machine) exercise real cross-NAT
//! hole-punching — that is the only thing left to the manual two-laptop run.
//!
//! *Moved 2026-10-13 from org-node's `tests/transport_networked.rs` (S4,
//! task T10):* both endpoints are bound from keychain items, org-io's
//! in-memory one, under the test-only `BindPlan::Relay`; the assertions are
//! unchanged. The trie fixtures are org-node's (`test_fixtures`).
use std::time::Duration;

use iroh::address_lookup::MemoryLookup;
use org_node::chain::OrgState;
use org_node::ids::OrgId;
use org_node::sequence::SeqGuard;
use org_node::test_fixtures::{admin_device, admit_member_delta, genesis_trie, ADMIN_DEVICE_SEED};
use org_node::verify::{verify_envelope_against_chain, VerifyContext};
use org_node::wire::WireMessage;
use org_node::{Envelope, Epoch, MemberSeed, OrgPrivateKey, SequenceNumber};
use org_io::test_support::MemoryKeychain;
use org_io::transport::{BindPlan, OrgEndpoint};

mod common;
use common::handles::keep_device_seed;

// verifies: LLR-v873fx
#[tokio::test]
async fn delivers_and_verifies_admit_over_relay_by_id() {
    MemoryKeychain::shared().install();
    let member = MemberSeed::from([1u8; 32]).x25519_keypair();
    let org = OrgId::new([5u8; 20]);

    let genesis = genesis_trie(&member, &admin_device());
    let (delta, new_trie) = admit_member_delta(&member);
    let new_root = new_trie.root_hash().unwrap();
    let env = Envelope::build(org, SequenceNumber::new(2), &delta).unwrap();
    let msg = WireMessage::OrgInformation {
        envelope: env.clone(),
        record_snapshot: vec![],
        org_private_key: OrgPrivateKey::from([0xab; 32]),
    };

    // In-process relay + shared in-memory address lookup. The Server is held
    // for the test's lifetime (dropping it stops the relay).
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server().await.expect("spawn in-process relay");
    let lookup = MemoryLookup::new();

    let a_key = keep_device_seed(ADMIN_DEVICE_SEED);
    let b_key = keep_device_seed([11u8; 32]);
    let ep_a = OrgEndpoint::bind_from_keychain(&a_key, BindPlan::Relay(relay_map.clone(), lookup.clone()))
        .await
        .expect("bind A against relay");
    let ep_b = OrgEndpoint::bind_from_keychain(&b_key, BindPlan::Relay(relay_map.clone(), lookup.clone()))
        .await
        .expect("bind B against relay");

    // Register both with the relay so a relay home is assigned, then publish
    // each endpoint's addr (including its relay home) into the shared lookup
    // so dial-by-EndpointId can resolve.
    tokio::time::timeout(Duration::from_secs(30), ep_a.inner().online()).await.expect("A did not come online within 30 s");
    tokio::time::timeout(Duration::from_secs(30), ep_b.inner().online()).await.expect("B did not come online within 30 s");
    lookup.add_endpoint_info(ep_a.inner().addr());
    lookup.add_endpoint_info(ep_b.inner().addr());

    let b_id = ep_b.inner().id();

    // B receives in a background task — spawn before A dials so accept() is
    // already waiting.
    let recv_task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(30), ep_b.recv_one()).await.expect("recv_one timed out after 30 s")
    });

    // A dials B PURELY BY EndpointId — resolution + relay routing are what we
    // are exercising here.
    tokio::time::timeout(Duration::from_secs(30), ep_a.send_to_id(b_id, &msg))
        .await
        .expect("send_to_id timed out after 30 s")
        .expect("send_to_id failed");

    let (remote_device, got) = recv_task.await.expect("recv task panicked").expect("recv_one failed");

    // 1. QUIC handshake authenticated A's device key.
    assert_eq!(remote_device, a_key, "authenticated remote device key must equal A's device key");
    // 2. The WireMessage arrived intact over the relay.
    assert_eq!(got, msg, "received WireMessage must equal sent WireMessage");

    // 3. B verifies the received envelope against the chain state holding
    //    the new root at epoch 2 (independent on-chain read, passed as a value).
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
    assert_eq!(out.trie.root_hash().unwrap(), new_root, "committed root mismatch");
    assert_eq!(out.epoch, Epoch::new(2), "committed epoch must be 2");
}
