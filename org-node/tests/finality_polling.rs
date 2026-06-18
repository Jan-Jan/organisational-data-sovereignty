#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Exercises the PRODUCTION FinalitySink::settle polling loop against a
//! chopsticks fork. chain_genesis_e2e.rs substitutes its own synchronous
//! ChopsticksSink, so the real poll-until-newer-finalized + timeout logic
//! (and its at_current_block() call) is otherwise never executed. Hermetic:
//! chopsticks only, no secrets, no external network.
//!
//! Feature gate: this test is gated on `app` (not `chain`). `FinalitySink`
//! is defined under `chain`, but it lives in `org_node::service`, and the
//! `service` module is itself gated behind `app` in lib.rs (app => chain).
//! So the test/run command must enable `app`.
//!
//! Miner/finalization note: `mine_block` (chopsticks `dev_newBlock`) is
//! sufficient to advance the finalized cursor that `at_current_block()`
//! reads — chopsticks finalizes blocks it builds in dev mode by default,
//! which `chain_genesis_e2e` already relies on (it reads state back right
//! after `mine_block`). No extra finalization call was needed.
//!
//! Run: cargo test -p org-node --features app --test finality_polling -- --test-threads=1
//! Cleanup if a fork is orphaned: pkill -f "chopsticks.*--config"
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use org_node::chain_write::proxy::BlockSink;
use org_node::service::FinalitySink;

mod common;
use common::chopsticks_fork::spawn_fork;
use common::chopsticks_reorg::mine_block;
use common::conn::legacy_client;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn settle_waits_for_a_newer_finalized_block() {
    // Own the fork behind an Arc so the timer-miner task can share it without
    // taking ownership; the fork (and its child process) is torn down when the
    // last Arc drops at end of test.
    let fork = Arc::new(spawn_fork().await.expect("spawn chopsticks fork"));
    eprintln!("chopsticks fork ready at {}", fork.ws_url);
    let api = legacy_client(&fork.ws_url).await.expect("subxt client");

    // Snapshot the finalized hash settle() will compare against.
    let pre_hash = api
        .at_current_block()
        .await
        .expect("at_current_block")
        .block_ref()
        .hash()
        .0;

    // Background timer-miner: produce a new block every ~3 s so finality
    // advances on a timer (simulating live-chain non-instant finality).
    // `mine_block` takes `&ChopsticksHandle`; `&fork2` (an `&Arc<…>`) deref-
    // coerces to it.
    let stop = Arc::new(AtomicBool::new(false));
    let miner = {
        let stop2 = stop.clone();
        let fork2 = fork.clone();
        tokio::spawn(async move {
            while !stop2.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_secs(3)).await;
                if stop2.load(Ordering::Relaxed) {
                    break;
                }
                let _ = mine_block(&fork2).await;
            }
        })
    };

    let sink = FinalitySink {
        api: api.clone(),
        timeout: Duration::from_secs(30),
    };

    let t0 = Instant::now();
    let settled = sink.settle().await.expect("settle must succeed");
    let elapsed = t0.elapsed();

    stop.store(true, Ordering::Relaxed);
    let _ = miner.await;

    // It must have waited at least one ~2 s poll interval (i.e. it actually
    // polled rather than returning the snapshot immediately)...
    assert!(
        elapsed >= Duration::from_secs(2),
        "settle returned in {elapsed:?}; expected it to poll at least one ~2 s interval"
    );
    // ...and returned a STRICTLY NEWER finalized block than the snapshot.
    assert_ne!(
        settled, pre_hash,
        "settle must return a newer finalized hash than the pre-submit snapshot"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn settle_times_out_gracefully_when_chain_stalls() {
    let fork = spawn_fork().await.expect("spawn chopsticks fork");
    eprintln!("chopsticks fork ready at {}", fork.ws_url);
    let api = legacy_client(&fork.ws_url).await.expect("subxt client");

    let pre_hash = api
        .at_current_block()
        .await
        .expect("at_current_block")
        .block_ref()
        .hash()
        .0;

    // No miner: the chain never advances, so settle must hit its deadline and
    // return the pre-submit hash (graceful degrade) rather than hanging.
    let sink = FinalitySink {
        api: api.clone(),
        timeout: Duration::from_secs(4),
    };

    let t0 = Instant::now();
    let settled = sink.settle().await.expect("settle must return Ok on timeout");
    let elapsed = t0.elapsed();

    assert_eq!(
        settled, pre_hash,
        "on stall, settle must return the pre-submit snapshot hash"
    );
    assert!(
        elapsed >= Duration::from_secs(4),
        "settle returned in {elapsed:?}; expected it to wait out the 4 s timeout"
    );
}
