#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Hermetic exercise of the preflight's chain.live check against chopsticks
//! (negative: stalled; positive: timer-miner). Moved from org-node's
//! tests/preflight.rs 2026-10-07; its two transport cases stay with org-node's
//! transport check, which needs a device key org-io does not hold (LLR-3zdw8v).
//! Not in verify_commands: it spawns a chopsticks fork.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use org_io::preflight::check_chain_live;

mod common;
use common::chopsticks_fork::spawn_fork;
use common::chopsticks_reorg::mine_block;
use common::conn::legacy_client;

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
    let miner_stop = stop.clone();
    let miner_fork = fork.clone();
    let miner = tokio::spawn(async move {
        while !miner_stop.load(Ordering::Relaxed) {
            let _ = mine_block(&miner_fork).await;
        }
    });
    let live = check_chain_live(&api, Duration::from_secs(12)).await;
    stop.store(true, Ordering::Relaxed);
    let _ = miner.await;
    assert!(live.ok, "live chain.live should pass with a miner: {live:?}");
}
