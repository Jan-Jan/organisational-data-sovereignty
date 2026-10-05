#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Hermetic exercises of the preflight checks: transport (loopback PASS;
//! networked is only asserted to be BOUNDED — not hang — because it uses the
//! real presets::N0 builder whose relay reachability is environment-dependent)
//! and chain.live against chopsticks (negative: stalled; positive: timer-miner).
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use org_node::{DeviceSeed};
use org_node::preflight::{check_chain_live, check_transport};
use org_node::transport::TransportMode;

mod common;
use common::chopsticks_fork::spawn_fork;
use common::chopsticks_reorg::mine_block;
use common::conn::legacy_client;

#[tokio::test]
async fn transport_loopback_check_passes() {
    let device = DeviceSeed::from([7u8; 32]).signing_keypair();
    let res = check_transport(&device, TransportMode::Loopback, Duration::from_secs(5)).await;
    assert!(res.ok, "loopback transport check should pass: {res:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn transport_networked_check_is_bounded() {
    // The Networked check uses presets::N0 (real n0 relays); whether they are
    // reachable is environment-dependent, so we assert only that the check is
    // BOUNDED — it returns within ~2x its own timeout rather than hanging.
    // Either ok or !ok is acceptable; hanging is not.
    let device = DeviceSeed::from([42u8; 32]).signing_keypair();
    let inner_timeout = Duration::from_secs(5);
    let res = tokio::time::timeout(
        inner_timeout * 2 + Duration::from_secs(5),
        check_transport(&device, TransportMode::Networked, inner_timeout),
    )
    .await;
    assert!(res.is_ok(), "check_transport(Networked) hung beyond its timeout");
    let res = res.unwrap();
    assert_eq!(
        res.name.split('.').next(),
        Some("transport"),
        "unexpected check name: {res:?}"
    );
    eprintln!("networked check: ok={} {}", res.ok, res.detail);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn chain_live_check_passes_with_miner_and_fails_when_stalled() {
    let fork = Arc::new(spawn_fork().await.expect("spawn chopsticks fork"));
    let api = legacy_client(&fork.ws_url).await.expect("subxt client");

    // Stalled: no miner → chain.live must FAIL within the sample gap.
    let stalled = check_chain_live(&api, Duration::from_secs(2)).await;
    assert!(!stalled.ok, "stalled chain.live should fail: {stalled:?}");

    // Live: a continuous miner advances finality → chain.live must PASS.
    //
    // Timing reality of this harness: chopsticks `dev_newBlock` runs the real
    // Paseo-AssetHub wasm executor, and on a cold fork the FIRST block takes
    // ~20-30 s (runtime warmup) with steady-state blocks ~7-10 s after. So we:
    //   1. Pre-mine ONE block directly (awaited) to absorb the cold-start cost.
    //   2. Spawn a continuous miner (loop with no extra delay — each
    //      `dev_newBlock` already takes several seconds, so this is not a busy
    //      spin; it keeps fresh finalized blocks landing back-to-back).
    //   3. Run check_chain_live with a 12 s gap that comfortably exceeds the
    //      steady-state block time, so at least one new finalized head lands
    //      between the two samples.
    mine_block(&fork).await.expect("warmup mine_block");

    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let fork2 = fork.clone();
    let miner = tokio::spawn(async move {
        while !stop2.load(Ordering::Relaxed) {
            let _ = mine_block(&fork2).await;
        }
    });
    let live = check_chain_live(&api, Duration::from_secs(12)).await;
    stop.store(true, Ordering::Relaxed);
    let _ = miner.await;
    assert!(live.ok, "live chain.live should pass with a miner: {live:?}");
}
