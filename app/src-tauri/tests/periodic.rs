#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The periodic verification (SDD-tgu7dp): when it runs (LLR-3ampqm), how
//! it announces (LLR-2267hj) and what a behind Organisation gets at each run
//! (LLR-fe7umz). Every test drives `run_periodic_verification`, the
//! interface those items name, over org-io's substitute chain under tokio's
//! paused clock, and observes only what it emits and when. A run that
//! outlasts its interval is a chain read the test holds open
//! (`FakeChain::hold_next_read`).

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ods_poc_lib::events::{self, Emission, ReceiverOutcome};
use ods_poc_lib::periodic;
use ods_poc_lib::policy::TransportModeName;
use ods_poc_lib::state::AppState;
use org_io::node::DevicePublicKey;
use org_io::test_support::FakeChain;
use org_io::{HeldTarget, Verified};
use tokio::time::Instant;

/// What the emitter was given: each emission with the second (since the
/// periodic verification started) it was emitted at.
type Emitted = Arc<Mutex<Vec<(u64, Emission)>>>;

/// A handle over `chain` in Loopback mode, its store in a fresh directory.
fn state_over(chain: &FakeChain) -> (tempfile::TempDir, AppState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let state = AppState::for_test_over(dir.path().to_path_buf(), chain, TransportModeName::Loopback).unwrap();
    (dir, state)
}

/// A provisional founding "Acme" proposed on `state`; its founder's Device key.
async fn propose_founding(state: &AppState, org_name: &str) -> DevicePublicKey {
    state.org_io.propose_founding_organisation("Alice", "Smith", "alice", org_name).await.unwrap().founder
}

/// A founding written to the chain and adopted by a verification, so every
/// later verification finds it in step.
async fn adopted_founding(state: &AppState) -> DevicePublicKey {
    let founder = propose_founding(state, "Acme").await;
    state.org_io.write_organisation_to_chain(HeldTarget::Founding(founder)).await.unwrap();
    let verified = state.org_io.verify_organisation(HeldTarget::Founding(founder)).await;
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    founder
}

/// Starts `run_periodic_verification` over `state`'s handle with an
/// emitter that records each emission and when it came.
fn spawn_periodic(state: &AppState, interval: Duration) -> (Emitted, tokio::task::JoinHandle<()>) {
    let t0 = Instant::now();
    let emitted: Emitted = Arc::new(Mutex::new(Vec::new()));
    let emitted_in = Arc::clone(&emitted);
    let org_io = Arc::clone(&state.org_io);
    let task = tokio::spawn(async move {
        periodic::run_periodic_verification(org_io, interval, move |emission: Emission| {
            emitted_in.lock().unwrap().push(((Instant::now() - t0).as_secs(), emission));
        })
        .await;
    });
    (emitted, task)
}

/// The `verification-result` payloads emitted so far, each with its second.
fn results(emitted: &Emitted) -> Vec<(u64, serde_json::Value)> {
    emitted
        .lock()
        .unwrap()
        .iter()
        .filter(|(_, emission)| emission.name == "verification-result")
        .map(|(at, emission)| (*at, emission.payload.clone()))
        .collect()
}

/// The seconds at which runs announced their results, once per run.
fn run_times(emitted: &Emitted) -> Vec<u64> {
    let mut times: Vec<u64> = results(emitted).into_iter().map(|(at, _)| at).collect();
    times.dedup();
    times
}

// verifies: LLR-3ampqm
// No run before one interval; then one run per interval, each verifying
// every Organisation and provisional founding held (no `org_id`).
#[tokio::test(start_paused = true)]
async fn no_run_before_one_interval_then_every_held_target_once_per_interval() {
    let chain = FakeChain::new();
    let (_dir, state) = state_over(&chain);
    let adopted = adopted_founding(&state).await;
    let proposed = propose_founding(&state, "Beta").await;
    let (emitted, task) = spawn_periodic(&state, Duration::from_secs(600));

    tokio::time::sleep(Duration::from_secs(599)).await;
    assert_eq!(results(&emitted), vec![], "no run before one interval has elapsed");

    tokio::time::sleep(Duration::from_secs(6)).await; // t = 605
    let first = results(&emitted);
    assert_eq!(first.iter().map(|(at, _)| *at).collect::<Vec<_>>(), vec![600, 600], "one run at 600: {first:?}");
    // The adopted founding is verified as its Organisation, the proposed one by its founder.
    let adopted_org = state
        .org_io
        .personas()
        .await
        .into_iter()
        .find(|persona| persona.device_key == adopted)
        .and_then(|persona| persona.org_id)
        .expect("the adopted founding's Organisation");
    let mut named: Vec<(serde_json::Value, serde_json::Value)> =
        first.iter().map(|(_, entry)| (entry["org_id"].clone(), entry["founder"].clone())).collect();
    named.sort_by_key(|(org_id, _)| org_id.is_null());
    assert_eq!(
        named,
        vec![
            (serde_json::json!(hex::encode(adopted_org.as_bytes())), serde_json::Value::Null),
            (serde_json::Value::Null, serde_json::json!(hex::encode(proposed.as_bytes()))),
        ],
        "every target held is verified at each run: {first:?}"
    );

    tokio::time::sleep(Duration::from_secs(1200)).await; // t = 1805
    assert_eq!(run_times(&emitted), vec![600, 1200, 1800], "one run per interval");
    assert_eq!(results(&emitted).len(), 6, "two results at each of three runs");
    task.abort();
}

// verifies: LLR-3ampqm
// A run that outlasts its interval: no second run starts while it is still
// going, the next starts when it ends, and the missed runs are not made up.
#[tokio::test(start_paused = true)]
async fn a_late_run_delays_the_next_rather_than_overlapping_or_bursting() {
    let chain = FakeChain::new();
    let (_dir, state) = state_over(&chain);
    adopted_founding(&state).await;
    let reads_before = chain.reads();
    let hold = chain.hold_next_read();
    let t0 = Instant::now();
    let (emitted, task) = spawn_periodic(&state, Duration::from_secs(60));

    hold.arrived().await;
    assert_eq!((Instant::now() - t0).as_secs(), 60, "the first run starts after one interval");
    // The first run's chain read is held for 200 s: three ticks are missed.
    tokio::time::sleep(Duration::from_secs(200)).await; // t = 260
    assert_eq!(chain.reads(), reads_before + 1, "no second run started while the first was going");
    assert_eq!(results(&emitted), vec![], "the held run has announced nothing yet");
    hold.release();

    tokio::time::sleep(Duration::from_secs(90)).await; // t = 350
    assert_eq!(
        run_times(&emitted),
        vec![260, 320],
        "the late run ends at 260, the next starts one interval later; the missed runs are not made up"
    );
    task.abort();
}

// verifies: LLR-2267hj
// An adopted result is announced as the receiver loop announces a
// verification: its `Updated` events (the record read back after adoption),
// then its `verification-result`.
#[tokio::test(start_paused = true)]
async fn an_adopted_result_is_announced_as_the_receiver_loop_announces_it() {
    let chain = FakeChain::new();
    let (_dir, state) = state_over(&chain);
    let founder = propose_founding(&state, "Acme").await;
    // Written, not yet verified: the periodic run is the verification that adopts it.
    state.org_io.write_organisation_to_chain(HeldTarget::Founding(founder)).await.unwrap();
    let (emitted, task) = spawn_periodic(&state, Duration::from_secs(60));
    tokio::time::sleep(Duration::from_secs(61)).await;
    task.abort();

    let emitted: Vec<Emission> = emitted.lock().unwrap().iter().map(|(_, emission)| emission.clone()).collect();
    assert_eq!(
        emitted.iter().map(|emission| emission.name).collect::<Vec<_>>(),
        vec!["membership-updated", "incoming-verified", "epoch-changed", "verification-result"],
        "the Updated events of an adopted result come before its verification-result: {emitted:?}"
    );
    let entry = emitted[3].payload.clone();
    assert_eq!(entry["result"], "adopted", "{entry}");
    assert_eq!(entry["founder"], serde_json::json!(hex::encode(founder.as_bytes())), "{entry}");

    // Exactly the events the receiver loop emits for this result.
    let org_id = state.org_io.personas().await.into_iter().find_map(|persona| persona.org_id).expect("adopted");
    let record = state.org_io.organisation(org_id).await.expect("the adopted record");
    // `outcomes_for_verification` takes the typed entry, which this test only sees as JSON; the outcomes
    // it gives an adopted entry with its record read (LLR-acb5g4) are built here.
    let expected: Vec<Emission> = [
        ReceiverOutcome::Updated {
            org_id: hex::encode(org_id.as_bytes()),
            epoch: record.epoch.get(),
            root: hex::encode(record.root_hash.as_bytes()),
        },
        ReceiverOutcome::VerificationResult { entry },
    ]
    .iter()
    .flat_map(events::emissions_for)
    .collect();
    assert_eq!(emitted, expected, "the periodic run announces exactly what the receiver loop announces");
}

// verifies: LLR-2267hj
// A result that is not adopted is announced by one `verification-result`
// alone, never by an `Updated` event.
#[tokio::test(start_paused = true)]
async fn an_in_step_result_is_announced_by_its_verification_result_alone() {
    let chain = FakeChain::new();
    let (_dir, state) = state_over(&chain);
    adopted_founding(&state).await;
    let (emitted, task) = spawn_periodic(&state, Duration::from_secs(60));
    tokio::time::sleep(Duration::from_secs(61)).await;
    task.abort();

    let emitted = emitted.lock().unwrap().clone();
    assert_eq!(emitted.iter().map(|(_, emission)| emission.name).collect::<Vec<_>>(), vec!["verification-result"], "{emitted:?}");
    assert_eq!(emitted[0].1.payload["result"], "in_step", "{emitted:?}");
}

// verifies: LLR-2267hj
// Abnormal. A run whose chain read
// fails announces that result, `unverified` with org-io's reason, as one
// `verification-result` and nothing else; the next run, the chain back,
// announces the Organisation in step.
#[tokio::test(start_paused = true)]
async fn a_run_whose_chain_read_fails_is_announced_unverified_and_the_next_run_recovers() {
    let chain = FakeChain::new();
    let (_dir, state) = state_over(&chain);
    adopted_founding(&state).await;
    chain.read_failing.store(true, Ordering::SeqCst);
    let (emitted, task) = spawn_periodic(&state, Duration::from_secs(60));
    tokio::time::sleep(Duration::from_secs(61)).await;
    let first = emitted.lock().unwrap().clone();
    assert_eq!(first.iter().map(|(_, emission)| emission.name).collect::<Vec<_>>(), vec!["verification-result"], "{first:?}");
    assert_eq!(first[0].1.payload["result"], "unverified", "{first:?}");
    assert!(first[0].1.payload["reason"].as_str().is_some_and(|reason| !reason.is_empty()), "{first:?}");
    chain.read_failing.store(false, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_secs(60)).await;
    task.abort();
    let all = results(&emitted);
    assert_eq!(all.len(), 2, "{all:?}");
    assert_eq!(all[1].1["result"], "in_step", "{all:?}");
}

// verifies: LLR-fe7umz
#[tokio::test(start_paused = true)]
async fn a_behind_organisation_is_requested_at_each_run_until_in_step() {
    let chain = FakeChain::new();
    let (_dir, state) = state_over(&chain);
    adopted_founding(&state).await;
    let (emitted, task) = spawn_periodic(&state, Duration::from_secs(60));

    // The chain holds the Organisation one epoch past every provisional organisation held.
    chain.epoch_ahead.store(true, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_secs(150)).await; // runs at 60 and 120
    // Once a run finds it in step, no later run requests it.
    chain.epoch_ahead.store(false, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_secs(120)).await; // runs at 180 and 240
    task.abort();

    let results = results(&emitted);
    assert_eq!(results.iter().map(|(at, _)| *at).collect::<Vec<_>>(), vec![60, 120, 180, 240], "{results:?}");
    for (at, entry) in &results[..2] {
        assert_eq!(entry["result"], "behind", "run at {at}: {entry}");
        // Requested at that run, and not refused: the report names every
        // other Device of the record, and a sole founder's record has none.
        assert_eq!(entry["requests"], serde_json::json!([]), "run at {at}: a behind result is requested at that run: {entry}");
        assert!(entry.get("requests_refused").is_none(), "run at {at}: the request was made, not refused: {entry}");
    }
    for (at, entry) in &results[2..] {
        assert_eq!(entry["result"], "in_step", "run at {at}: {entry}");
        assert!(entry.get("requests").is_none(), "run at {at}: no request after in step: {entry}");
    }
}
