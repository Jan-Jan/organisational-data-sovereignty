//! The commands driven across the REAL Tauri IPC boundary.
//!
//! Carries: REQ-645jq9 (the transport mode crosses the boundary and is named
//! `transport_mode` on the wire), REQ-sjkp8z (the org-id boundary check is
//! reached by the real handler under the real argument name), REQ-vgr7s2 (an
//! empty peer address is genuinely accepted — the backend half of HAZ-n97v5g).
//!
//! NOT REQ-he8ejb. That requirement is about a check made BEFORE ANY COMMAND IS
//! INVOKED, and nothing in this file can observe one: every test here invokes a
//! command. The two member-id tests below are robustness tests over the
//! handler's own defence-in-depth checks and carry no requirement; REQ-he8ejb is
//! verified in app/tests/revoke.validate.test.ts.
//!
//! Everything here uses `tauri::test::mock_builder`, the same
//! `generate_handler!` list as `lib.rs`, and the same camelCase argument names
//! the frontend sends. What it catches that an extracted-logic test cannot is a
//! handler that is correct inside and unreachable, misnamed or mis-typed at its
//! boundary.
//!
//! `mock_context` installs an empty ACL (`Resolved::default()`). In Tauri 2 the
//! capability system gates PLUGIN commands, not app commands registered through
//! `generate_handler!`, so this suite reaches all twelve handlers without a
//! fixture `tauri.conf.json`.
//!
//! No test here sets an environment variable: `AppState::for_test` takes every
//! input as a parameter, exactly so that these tests cannot race each other.

use ods_poc_lib::commands;
use ods_poc_lib::policy::{ChainEndpoint, TransportModeName};
use ods_poc_lib::state::AppState;
use org_node::OrgNodeError;

use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::test::MockRuntime;
use tauri::webview::InvokeRequest;
use tauri::WebviewWindow;

/// A live mock application with a real `AppState` over a fresh store.
///
/// The `TempDir` is held so the store outlives the test; the `App` is held
/// because dropping it would tear down the webview the requests go to.
struct Harness {
    _dir: tempfile::TempDir,
    _app: tauri::App<MockRuntime>,
    webview: WebviewWindow<MockRuntime>,
}

fn harness_with(transport: TransportModeName, chain: Option<ChainEndpoint>) -> Harness {
    let dir = tempfile::tempdir().expect("tempdir");
    let state = AppState::for_test(dir.path().to_path_buf(), "ipc-suite-passphrase", chain, transport)
        .expect("AppState::for_test");

    // The SAME handler list as lib.rs. A command added there and not here makes
    // this suite stop covering it, which is why the list is spelled out rather
    // than shared through a helper that could silently drift.
    let app = mock_builder()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::create_persona,
            commands::create_organisation,
            commands::export_invite,
            commands::import_invite,
            commands::export_join_request,
            commands::import_join_request,
            commands::admit_member,
            commands::revoke_member,
            commands::list_personas,
            commands::list_orgs,
            commands::connection_status,
            commands::start_receiver,
        ])
        .build(mock_context(noop_assets()))
        .expect("build mock app");

    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("build webview");

    Harness {
        _dir: dir,
        _app: app,
        webview,
    }
}

fn harness() -> Harness {
    harness_with(TransportModeName::Networked, None)
}

fn request(cmd: &str, body: tauri::ipc::InvokeBody) -> InvokeRequest {
    InvokeRequest {
        cmd: cmd.into(),
        callback: tauri::ipc::CallbackFn(0),
        error: tauri::ipc::CallbackFn(1),
        // The ORIGIN matters, not just the spelling. `Webview::is_local_url`
        // compares against the platform's tauri protocol URL, and a request
        // that is not local is gated by the ACL — which `mock_context` leaves
        // empty — so the wrong literal here makes every command in this suite
        // fail with "not allowed. Plugin not found" rather than run.
        url: if cfg!(any(windows, target_os = "android")) {
            "http://tauri.localhost"
        } else {
            "tauri://localhost"
        }
        .parse()
        .unwrap(),
        body,
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    }
}

/// Invoke `cmd` with a JSON argument object and return the raw result.
fn invoke(
    h: &Harness,
    cmd: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    get_ipc_response(&h.webview, request(cmd, tauri::ipc::InvokeBody::Json(args)))
        .map(|b| b.deserialize::<serde_json::Value>().expect("response is JSON"))
}

/// Invoke `cmd` and require it to have failed, returning the error message.
fn invoke_err(h: &Harness, cmd: &str, args: serde_json::Value) -> String {
    match invoke(h, cmd, args) {
        Ok(v) => panic!("{cmd} unexpectedly succeeded: {v}"),
        Err(e) => e.as_str().map(str::to_string).unwrap_or_else(|| e.to_string()),
    }
}

fn org_id_40() -> String {
    "aa".repeat(20)
}

// ---------------------------------------------------------------------------
// REQ-645jq9 — the transport mode crosses the boundary
// ---------------------------------------------------------------------------

// verifies: REQ-645jq9
#[test]
fn connection_status_reports_transport_mode_over_ipc() {
    // Loopback, because Networked is what a hard-coded literal would return and
    // the test must be able to tell the two apart.
    let h = harness_with(TransportModeName::Loopback, None);
    let status = invoke(&h, "connection_status", serde_json::json!({})).expect("connection_status");
    assert_eq!(
        status.get("transport_mode"),
        Some(&serde_json::Value::String("loopback".into())),
        "the transport mode must cross the IPC boundary as `transport_mode`: {status}"
    );
}

// verifies: REQ-645jq9
#[test]
fn connection_status_is_registered_under_its_name() {
    let h = harness();
    let status = invoke(&h, "connection_status", serde_json::json!({}))
        .expect("connection_status must be registered under exactly this name");
    // The field names api.ts declares. A renamed field is a silently undefined
    // value in the UI, not a build error, so the wire shape is asserted here.
    for field in [
        "chain_configured",
        "chain_ws",
        "contract_h160",
        "transport_mode",
        "data_dir",
    ] {
        assert!(
            status.get(field).is_some(),
            "connection_status must report `{field}` on the wire: {status}"
        );
    }
}

// ---------------------------------------------------------------------------
// REQ-sjkp8z — the org-id boundary check is reached by the real handler
// ---------------------------------------------------------------------------

// verifies: REQ-sjkp8z
#[test]
fn export_invite_rejects_a_short_org_id() {
    let h = harness();
    // 39 characters: one short of the required width.
    let err = invoke_err(
        &h,
        "export_invite",
        serde_json::json!({ "orgId": "a".repeat(39) }),
    );
    assert!(
        err.contains("40 hex chars"),
        "a short org_id must be refused for its WIDTH, by the real handler under \
         the real `orgId` argument name, got: {err}"
    );
}

// verifies: REQ-sjkp8z
#[test]
fn export_invite_rejects_a_non_hex_org_id() {
    let h = harness();
    let err = invoke_err(
        &h,
        "export_invite",
        serde_json::json!({ "orgId": "zz".repeat(20) }),
    );
    assert!(
        err.contains("hex"),
        "a 40-character non-hex org_id must be refused for its ALPHABET, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// The Rust-side member-id checks — defence in depth, behind the frontend's
//
// These two carried `verifies: REQ-he8ejb` and could not bear it. REQ-he8ejb
// requires the member id to be checked BEFORE ANY COMMAND IS INVOKED; both
// tests below assert on the HANDLER's check, which by definition runs after the
// command was invoked, so deleting the frontend check — the behaviour the
// requirement actually names — leaves both green. REQ-he8ejb is verified in
// app/tests/revoke.validate.test.ts, against `validateRevokeInput`, which is
// where the pre-invocation decision lives.
//
// The tests stay: the handler's own width and prefix checks are worth keeping
// and worth gating. They simply carry no requirement of their own.
// ---------------------------------------------------------------------------

// robustness: no requirement of its own — REQ-he8ejb is a pre-invocation
// requirement and this asserts on the handler, which runs after invocation.
#[test]
fn revoke_member_rejects_a_short_member_id() {
    let h = harness();
    let err = invoke_err(
        &h,
        "revoke_member",
        serde_json::json!({
            "orgId": org_id_40(),
            "memberIdHex": "bb".repeat(31),
            "peerAddrBlob": ""
        }),
    );
    assert!(
        err.contains("32 bytes") || err.contains("64 hex"),
        "the backend must refuse an under-width member id on its own account, \
         not rely on the form having checked: {err}"
    );
}

// robustness: no requirement of its own — same reason as the test above. The
// doubled-prefix defect it guards is real and the handler's refusal is worth
// keeping; it is not, however, the pre-invocation check REQ-he8ejb requires.
#[test]
fn revoke_member_rejects_a_doubled_zero_x_prefix_on_the_member_id() {
    let h = harness();
    // `0x0x` + 64 hex characters. A REPEATED strip eats both prefixes and
    // hands 64 clean hex characters to the width check, so malformed input is
    // silently normalised into a valid member id and the revocation proceeds
    // against it. One optional strip leaves `0x` + 64 characters, which the
    // hex decode refuses — and the refusal must come from THIS handler, at
    // the IPC boundary, not from something further in.
    let err = invoke_err(
        &h,
        "revoke_member",
        serde_json::json!({
            "orgId": org_id_40(),
            "memberIdHex": format!("0x0x{}", "bb".repeat(32)),
            "peerAddrBlob": ""
        }),
    );
    assert!(
        err.contains("member_id hex"),
        "exactly one `0x` prefix is permitted on a member id; a doubled prefix \
         is malformed input and must be refused at the boundary rather than \
         normalised into a valid id, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// REQ-vgr7s2 — an empty peer address is not a ground for refusal (HAZ-n97v5g)
// ---------------------------------------------------------------------------

// verifies: REQ-vgr7s2
#[test]
fn revoke_member_is_not_refused_for_an_empty_peer_addr() {
    let h = harness_with(TransportModeName::Networked, None);
    let err = invoke_err(
        &h,
        "revoke_member",
        serde_json::json!({
            "orgId": org_id_40(),
            "memberIdHex": "bb".repeat(32),
            "peerAddrBlob": ""
        }),
    );
    // What this test establishes: the empty peer address is not the reason the
    // command fails. It CANNOT establish that the revocation succeeds, and the
    // name must not claim it does — `invoke_err` panics if the command returns
    // Ok, so this test positively REQUIRES the call to fail. It fails because
    // the harness store is fresh: there is no organisation under `org_id_40()`
    // and no member `bb…bb` in it, and a real success would need an org, a
    // member, a chain and a reachable peer, none of which this repository can
    // stand up. So the failure is a given; the only question this test asks is
    // WHAT it failed for, and the assertion below pins that to "not the
    // address".
    //
    // REQ-vgr7s2's acceptance clause — an empty peer address in Networked
    // transport is accepted — is established on the frontend side, in
    // app/tests/revoke.validate.test.ts, where `validateRevokeInput` genuinely
    // returns `{ ok: true }`. That is where the decision actually lives; this
    // test guards the Rust side against a redundant non-empty check creeping
    // back in behind it (HAZ-n97v5g).
    //
    // The assertion is POSITIVE, and that is the point. It used to read
    // `!err.to_lowercase().contains("peer_addr")`, which was keyed to one
    // spelling of one diagnostic: the original defect's message was
    // `Peer addr blob (hex) is required.`, and lowercased that does not contain
    // `peer_addr` — so reintroducing the exact defect this test exists to catch
    // left it green. Naming the reason the call MUST fail for instead makes any
    // earlier refusal, however worded, a failure.
    //
    // That reason is the store: this harness is fresh, so there is no
    // organisation under `org_id_40()` and `OrgService::revoke_member`'s first
    // act — `find_org` — reports `OrgNotOnChain`. Reaching it proves the call
    // got past org-id parsing, past member-id parsing and past address
    // handling, into the service. The expected text is taken from the ERROR
    // TYPE rather than written out, so an upstream rewording moves both sides
    // together and only a behavioural change can fail this.
    assert_eq!(
        err,
        OrgNodeError::OrgNotOnChain.to_string(),
        "an empty peer address must not be the ground of refusal in Networked \
         transport: the call must reach the service and fail there, because the \
         harness store holds no such organisation, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Robustness
// ---------------------------------------------------------------------------

// robustness: no requirement of its own — the plan's table records this test as
// carrying none. It holds the precondition every REQ above depends on: that the
// handler list is CLOSED, so a command renamed in lib.rs but not in api.ts is an
// error the frontend can see rather than a silent undefined.
#[test]
fn unknown_command_is_rejected() {
    let h = harness();
    let res = invoke(&h, "connectionStatus", serde_json::json!({}));
    assert!(
        res.is_err(),
        "the handler list must be closed: an unregistered command is an error, \
         not a silent success, got: {res:?}"
    );
}

// robustness: no requirement of its own — the plan's table records this test as
// carrying none. It holds the other precondition: that the managed state is
// reachable from a handler at all, so a failure above is a failure of the
// behaviour under test and not of this suite's wiring.
#[test]
fn list_personas_returns_an_empty_list_on_a_fresh_store() {
    let h = harness();
    let personas = invoke(&h, "list_personas", serde_json::json!({})).expect("list_personas");
    assert_eq!(
        personas,
        serde_json::json!([]),
        "a fresh store has no personas, and the managed state must be the one \
         the handler reads: {personas}"
    );
}

// ---------------------------------------------------------------------------
// Persona details are parsed at this boundary. The requirement for parsed
// Persona details is org-node's and is not exported to this unit; it is
// verified there (org-node/tests/persona_records.rs); these are this
// handler's robustness tests and carry no requirement of their own.
// ---------------------------------------------------------------------------

// robustness: the handler parses the handle, name and surname before
// org-node sees them, and stores the parsed (NFC) form.
#[test]
fn create_persona_stores_the_parsed_details() {
    let h = harness();
    invoke(
        &h,
        "create_persona",
        serde_json::json!({ "handle": "jose\u{0301}", "name": "Jose\u{0301}", "surname": "Smith" }),
    )
    .expect("create_persona");
    let personas = invoke(&h, "list_personas", serde_json::json!({})).expect("list_personas");
    assert_eq!(personas[0]["handle"], "jos\u{e9}");
    assert_eq!(personas[0]["name"], "Jos\u{e9}");
}

// robustness: an invalid field is refused at the boundary, named, and
// nothing is created.
#[test]
fn create_persona_refuses_an_invalid_field_naming_it_and_creates_nothing() {
    let h = harness();
    let long = "a".repeat(129);
    let cases = [
        ("Alice", "Alice", "Smith", "handle:"),
        ("alice", long.as_str(), "Smith", "name:"),
        ("alice", "Alice", long.as_str(), "surname:"),
    ];
    for (handle, name, surname, field) in cases {
        let err = invoke_err(
            &h,
            "create_persona",
            serde_json::json!({ "handle": handle, "name": name, "surname": surname }),
        );
        assert!(err.starts_with(field), "the refusal must name the field ({field}): {err}");
    }
    let personas = invoke(&h, "list_personas", serde_json::json!({})).expect("list_personas");
    assert_eq!(personas.as_array().map(Vec::len), Some(0), "a refused persona is not created");
}

// robustness: the Organisation secret is parsed from hex at this boundary.
#[test]
fn admit_member_refuses_an_org_secret_that_is_not_32_bytes() {
    let h = harness();
    let pid = invoke(
        &h,
        "create_persona",
        serde_json::json!({ "handle": "bob", "name": "Bob", "surname": "Jones" }),
    )
    .expect("create_persona");
    let blob = invoke(&h, "export_join_request", serde_json::json!({ "personaId": pid }))
        .expect("export_join_request");
    let err = invoke_err(
        &h,
        "admit_member",
        serde_json::json!({ "orgId": org_id_40(), "joinRequestBlob": blob, "orgSecretHex": "aa".repeat(31) }),
    );
    assert!(err.contains("org_secret must be 32 bytes"), "got {err}");
}
