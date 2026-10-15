//! The guard that decides whether a receiver loop may start.
//!
//! Requirements carried here: REQ-6hgm8r (at most one receiver loop runs at a
//! time) and REQ-3hfggn (the slot is released however the loop ends, so a
//! restart is possible).
//!
//! The previous code set a flag before spawning and never cleared it, so an
//! exited loop kept the slot for the life of the process (HAZ-cfp4jb).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};

use ods_poc_lib::events::StartGuard;

fn flag() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

// verifies: LLR-5zd6j8
#[test]
fn first_claim_succeeds() {
    let flag = flag();
    let guard = StartGuard::try_claim(&flag);
    assert!(guard.is_some(), "nothing is running, so a loop may start");
    assert!(flag.load(Ordering::Acquire), "the slot is marked as taken");
}

// verifies: LLR-5zd6j8
#[test]
fn second_claim_while_held_fails() {
    let flag = flag();
    let _first = StartGuard::try_claim(&flag).expect("first claim");
    assert!(
        StartGuard::try_claim(&flag).is_none(),
        "a second receiver loop must not be startable while the first holds the slot"
    );
}

// verifies: LLR-z66f27
#[test]
fn claim_succeeds_again_after_the_guard_drops() {
    let flag = flag();
    {
        let _first = StartGuard::try_claim(&flag).expect("first claim");
    }
    assert!(
        !flag.load(Ordering::Acquire),
        "dropping the guard releases the slot"
    );
    assert!(
        StartGuard::try_claim(&flag).is_some(),
        "a finished loop must be restartable"
    );
}

// verifies: LLR-z66f27
#[test]
fn guard_releases_when_dropped_by_panic() {
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let f = std::sync::Arc::clone(&flag);
    let _ = std::panic::catch_unwind(move || {
        let _g = StartGuard::try_claim(&f).expect("first claim");
        panic!("the receiver loop panicked");
    });
    assert!(
        StartGuard::try_claim(&flag).is_some(),
        "a panicking loop must not keep the slot claimed for the life of the process"
    );
}

// verifies: LLR-5zd6j8
#[test]
fn concurrent_claims_yield_exactly_one_guard() {
    // The abnormal-input case for REQ-6hgm8r: sixteen callers racing on one
    // flag. A read-then-write guard would let several through here, which is
    // exactly the double-spawning the rewiring must not reintroduce.
    let flag = flag();
    let barrier = Arc::new(Barrier::new(16));
    let handles: Vec<_> = (0..16)
        .map(|_| {
            let f = Arc::clone(&flag);
            let b = Arc::clone(&barrier);
            std::thread::spawn(move || {
                b.wait();
                StartGuard::try_claim(&f)
            })
        })
        .collect();
    let guards: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("no claimant panics"))
        .collect();
    assert_eq!(
        guards.iter().filter(|g| g.is_some()).count(),
        1,
        "exactly one of sixteen racing callers may start a receiver loop"
    );
}

// verifies: LLR-z66f27
#[test]
fn release_then_concurrent_claims_yield_exactly_one_guard() {
    // Release and re-claim must compose: after a loop ends, the next race is
    // decided the same way rather than letting every waiting caller through.
    let flag = flag();
    {
        let _first = StartGuard::try_claim(&flag).expect("first claim");
    }
    let barrier = Arc::new(Barrier::new(8));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let f = Arc::clone(&flag);
            let b = Arc::clone(&barrier);
            std::thread::spawn(move || {
                b.wait();
                StartGuard::try_claim(&f)
            })
        })
        .collect();
    let guards: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("no claimant panics"))
        .collect();
    assert_eq!(
        guards.iter().filter(|g| g.is_some()).count(),
        1,
        "a released slot is claimable exactly once"
    );
}

// ---------------------------------------------------------------------------
// S4, task T16: the receiver loop holds no app lock while it waits
// ---------------------------------------------------------------------------

use std::sync::mpsc;
use std::time::Duration;

use ods_poc_lib::commands;
use ods_poc_lib::policy::TransportModeName;
use ods_poc_lib::state::AppState;
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, MockRuntime, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindow};

/// Invoke `cmd` across the mock IPC boundary, as `tests/ipc.rs` does.
fn invoke(
    webview: &WebviewWindow<MockRuntime>,
    cmd: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    let request = InvokeRequest {
        cmd: cmd.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
        url: if cfg!(any(windows, target_os = "android")) { "http://tauri.localhost" } else { "tauri://localhost" }
            .parse()
            .expect("the local origin parses"),
        body: tauri::ipc::InvokeBody::Json(args),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    };
    get_ipc_response(webview, request).map(|body| body.deserialize::<serde_json::Value>().expect("response is JSON"))
}

// A Persona's endpoint is bound and the loop waits in org-io's
// `next_received` with no peer sending; a command that needs the handle
// still completes within a bound, and the loop is still running after it
// (so it was waiting, not stopped).
// verifies: LLR-5wb4vd
#[test]
fn the_receiver_holds_no_app_lock_while_waiting() {
    let dir = tempfile::tempdir().expect("tempdir");
    let state = AppState::for_test(dir.path().to_path_buf(), None, TransportModeName::Loopback)
        .expect("AppState::for_test");
    let app = mock_builder()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::create_persona,
            commands::list_personas,
            commands::start_receiver,
        ])
        .build(mock_context(noop_assets()))
        .expect("build mock app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().expect("build webview");

    invoke(&webview, "create_persona", serde_json::json!({ "handle": "ada", "name": "Ada", "surname": "Lovelace" }))
        .expect("create_persona");
    invoke(&webview, "start_receiver", serde_json::json!({})).expect("start_receiver");
    // Let the loop bind the Persona's endpoint and start waiting.
    std::thread::sleep(Duration::from_millis(500));

    let (listed_tx, listed_rx) = mpsc::channel();
    let listing_webview = webview.clone();
    std::thread::spawn(move || {
        let _ = listed_tx.send(invoke(&listing_webview, "list_personas", serde_json::json!({})));
    });
    let listed = listed_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("list_personas completes while the receiver waits for a message")
        .expect("list_personas");
    assert_eq!(listed.as_array().map(Vec::len), Some(1), "the one Persona is listed: {listed}");
    assert!(
        app.state::<AppState>().receiver_started.load(Ordering::Acquire),
        "the receiver loop is still running, so the command completed while it waited"
    );
}

// Abnormal: while the loop waits, a command org-io refuses (founding with a
// Persona the node does not hold) returns its refusal, and a command that
// changes the node (a second Persona) completes, each within the bound; the
// loop is still running after both (S4, task T23).
// verifies: LLR-5wb4vd
#[test]
fn a_refused_and_a_mutating_command_complete_while_the_receiver_waits() {
    let dir = tempfile::tempdir().expect("tempdir");
    let state = AppState::for_test(dir.path().to_path_buf(), None, TransportModeName::Loopback)
        .expect("AppState::for_test");
    let app = mock_builder()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::create_persona,
            commands::create_organisation,
            commands::list_personas,
            commands::start_receiver,
        ])
        .build(mock_context(noop_assets()))
        .expect("build mock app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().expect("build webview");

    invoke(&webview, "create_persona", serde_json::json!({ "handle": "ada", "name": "Ada", "surname": "Lovelace" }))
        .expect("create_persona");
    invoke(&webview, "start_receiver", serde_json::json!({})).expect("start_receiver");
    std::thread::sleep(Duration::from_millis(500));

    let (done_tx, done_rx) = mpsc::channel();
    let commands_webview = webview.clone();
    std::thread::spawn(move || {
        let refused = invoke(&commands_webview, "create_organisation", serde_json::json!({ "personaId": "nobody" }));
        let created = invoke(
            &commands_webview,
            "create_persona",
            serde_json::json!({ "handle": "grace", "name": "Grace", "surname": "Hopper" }),
        );
        let listed = invoke(&commands_webview, "list_personas", serde_json::json!({}));
        let _ = done_tx.send((refused, created, listed));
    });
    let (refused, created, listed) =
        done_rx.recv_timeout(Duration::from_secs(5)).expect("the commands complete while the receiver waits for a message");
    assert!(refused.is_err(), "a Persona the node does not hold founds nothing: {refused:?}");
    created.expect("the second Persona is created");
    assert_eq!(listed.expect("list_personas").as_array().map(Vec::len), Some(2), "both Personas are listed");
    assert!(
        app.state::<AppState>().receiver_started.load(Ordering::Acquire),
        "the receiver loop is still running after the commands"
    );
}

// ---------------------------------------------------------------------------
// S4, task T19: the real receive loop stops on a closed endpoint, and restarts
// ---------------------------------------------------------------------------

/// How long the test waits for the loop to react before concluding.
const SETTLE: Duration = Duration::from_millis(1500);

/// Whether the receiver slot reads `started` within `SETTLE`.
fn slot_reads(app: &tauri::App<MockRuntime>, started: bool) -> bool {
    let deadline = std::time::Instant::now() + SETTLE;
    while std::time::Instant::now() < deadline {
        if app.state::<AppState>().receiver_started.load(Ordering::Acquire) == started {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

/// Close the endpoint of the handle's first Persona, as a permanent
/// transport failure would.
fn close_the_endpoint(app: &tauri::App<MockRuntime>) {
    let io = Arc::clone(&app.state::<AppState>().org_io);
    tauri::async_runtime::block_on(async move {
        let persona_id = io.personas().await[0].persona_id.clone();
        io.endpoint_for_test(&persona_id).await.expect("the Persona's endpoint is bound").inner().close().await;
    });
}

/// A mock app whose real receiver loop ran over a bound Loopback endpoint
/// until that endpoint was closed, the `receiver-stopped` payloads it
/// announced from the start, and its webview.
struct Stopped {
    app: tauri::App<MockRuntime>,
    webview: WebviewWindow<MockRuntime>,
    stopped: mpsc::Receiver<String>,
}

/// Start the real loop, check it waits, close its endpoint, and check that it
/// stops once, naming the reason, and releases the start guard.
fn a_loop_stopped_by_a_closed_endpoint() -> Stopped {
    use tauri::Listener;

    let dir = tempfile::tempdir().expect("tempdir");
    let state = AppState::for_test(dir.path().to_path_buf(), None, TransportModeName::Loopback)
        .expect("AppState::for_test");
    let app = mock_builder()
        .manage(state)
        .invoke_handler(tauri::generate_handler![commands::create_persona, commands::start_receiver])
        .build(mock_context(noop_assets()))
        .expect("build mock app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().expect("build webview");
    let (stopped_tx, stopped) = mpsc::channel();
    app.listen_any("receiver-stopped", move |event| {
        let _ = stopped_tx.send(event.payload().to_string());
    });

    invoke(&webview, "create_persona", serde_json::json!({ "handle": "ada", "name": "Ada", "surname": "Lovelace" }))
        .expect("create_persona");
    invoke(&webview, "start_receiver", serde_json::json!({})).expect("start_receiver");
    std::thread::sleep(Duration::from_millis(500));
    assert!(slot_reads(&app, true), "the loop is running before the endpoint closes");
    assert!(stopped.try_recv().is_err(), "no stop is announced while the loop waits");

    close_the_endpoint(&app);
    let reason = stopped.recv_timeout(SETTLE).expect("the loop announces that it stopped");
    assert!(reason.contains("endpoint closed"), "the stop names its reason: {reason}");
    assert!(slot_reads(&app, false), "the start guard is released once the loop has stopped");
    assert!(stopped.recv_timeout(SETTLE).is_err(), "the stop is announced once");
    // The tempdir is dropped here; the store is in memory already and the
    // test writes nothing more to it.
    Stopped { app, webview, stopped }
}

// The loop runs org-io's real receive step over a bound Loopback endpoint.
// The endpoint is closed: the loop stops, announces the stop exactly once,
// naming the reason, and releases the start guard.
// verifies: LLR-yrb6th
#[test]
fn a_closed_endpoint_stops_the_loop_announces_it_once_and_releases_the_guard() {
    let stopped = a_loop_stopped_by_a_closed_endpoint();
    assert!(
        !stopped.app.state::<AppState>().receiver_started.load(Ordering::Acquire),
        "nothing holds the slot after the stop"
    );
}

// After that stop, `start_receiver` starts a loop that runs: it holds the
// slot and announces no stop. Written ignored in S4 task T19 as the
// reproduction of org-io's problem report on a closed endpoint that was kept
// and never bound again; un-ignored 2026-10-14 (S4, task T20), when org-io
// binds a closed endpoint again on the next `start_receiving`.
// verifies: LLR-yrb6th
#[test]
fn after_a_stop_start_receiver_starts_a_loop_that_runs() {
    let Stopped { app, webview, stopped } = a_loop_stopped_by_a_closed_endpoint();
    invoke(&webview, "start_receiver", serde_json::json!({})).expect("start_receiver, again");
    assert!(
        stopped.recv_timeout(SETTLE).is_err(),
        "the restarted loop runs: it announces no stop"
    );
    assert!(
        app.state::<AppState>().receiver_started.load(Ordering::Acquire),
        "the restarted loop still holds the slot, so it is waiting for a message"
    );
}
