//! The commands driven across the REAL Tauri IPC boundary.
//!
//! Carries: REQ-645jq9 (the transport mode crosses the boundary and is named
//! `transport_mode` on the wire), REQ-sjkp8z (the org-id boundary check is
//! reached by the real handler under the real argument name), REQ-vgr7s2 (an
//! empty peer address is genuinely accepted — the backend half of HAZ-n97v5g).
//! Since 2026-10-05 each test is annotated with the low-level requirements of
//! `app/docs/architecture/` that refine these; the annotations name them.
//!
//! NOT REQ-he8ejb. That requirement is about a check made BEFORE ANY COMMAND IS
//! INVOKED, and nothing in this file can observe one: every test here invokes a
//! command. The member-id tests below verify the handler's own
//! defence-in-depth check (LLR-6pmrma, derived); REQ-he8ejb is verified in
//! app/tests/revoke.validate.test.ts.
//!
//! Everything here uses `tauri::test::mock_builder`, the same
//! `generate_handler!` list as `lib.rs`, and the same camelCase argument names
//! the frontend sends. What it catches that an extracted-logic test cannot is a
//! handler that is correct inside and unreachable, misnamed or mis-typed at its
//! boundary.
//!
//! `mock_context` installs an empty ACL (`Resolved::default()`). In Tauri 2 the
//! capability system gates PLUGIN commands, not app commands registered through
//! `generate_handler!`, so this suite reaches every handler without a
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

mod support;
use support::reply_keys;

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
    let data_dir = dir.path().to_path_buf();
    harness_at(dir, data_dir, transport, chain)
}

/// A harness whose store is opened in `data_dir`, which may lie inside `dir`
/// and need not exist yet.
fn harness_at(
    dir: tempfile::TempDir,
    data_dir: std::path::PathBuf,
    transport: TransportModeName,
    chain: Option<ChainEndpoint>,
) -> Harness {
    let state = AppState::for_test(data_dir, "ipc-suite-passphrase", chain, transport)
        .expect("AppState::for_test");

    // The SAME handler list as lib.rs, spelled out here because the macro needs
    // the paths at compile time. `product_registers_the_same_commands_as_the_harness`
    // fails if the two lists drift apart.
    let app = mock_builder()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::create_persona,
            commands::create_organisation,
            commands::export_invite,
            commands::import_invite,
            commands::produce_invite_reply,
            commands::import_invite_reply,
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

fn member_id_64() -> String {
    "bb".repeat(32)
}

/// `revoke_member`'s arguments with a well-formed org id.
fn revoke_args(member_id_hex: &str, peer_addr_blob: &str) -> serde_json::Value {
    serde_json::json!({
        "orgId": org_id_40(),
        "memberIdHex": member_id_hex,
        "peerAddrBlob": peer_addr_blob
    })
}

/// The refusal a well-formed call on an Organisation ends in on this fresh
/// store: no record of it is found. Taken from the error type, so an upstream
/// rewording moves both sides together.
fn refused_for_an_unknown_org() -> String {
    OrgNodeError::OrgNotOnChain.to_string()
}

// ---------------------------------------------------------------------------
// REQ-645jq9 — the transport mode crosses the boundary
// ---------------------------------------------------------------------------

// verifies: LLR-8tzbzn
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

// verifies: LLR-8tzbzn, LLR-4wcyqy
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

/// Malformed org ids: under width, over width, and the right width but not hex.
fn malformed_org_ids() -> [String; 3] {
    ["a".repeat(39), "a".repeat(42), "zz".repeat(20)]
}

/// The parser's own refusal of `org_id`, so the tests below require the
/// handler's refusal to be the parser's and no other.
fn parser_refusal(org_id: &str) -> String {
    ods_poc_lib::parsing::parse_org_id(org_id).expect_err("a malformed org id is refused")
}

// verifies: LLR-vzf8j2
#[test]
fn revoke_member_refuses_a_malformed_org_id_with_the_parsers_message() {
    // The member id is well-formed and the peer address absent, so if the org
    // id were not refused the call would end in the unknown-org refusal.
    let h = harness();
    for bad in malformed_org_ids() {
        let err = invoke_err(
            &h,
            "revoke_member",
            serde_json::json!({ "orgId": bad, "memberIdHex": member_id_64(), "peerAddrBlob": "" }),
        );
        assert_eq!(err, parser_refusal(&bad), "{bad}");
    }
}

// verifies: LLR-vzf8j2
#[test]
fn revoke_member_does_not_refuse_a_well_formed_org_id() {
    let h = harness();
    for org_id in [org_id_40(), format!("0x{}", org_id_40())] {
        let err = invoke_err(
            &h,
            "revoke_member",
            serde_json::json!({ "orgId": org_id, "memberIdHex": member_id_64(), "peerAddrBlob": "" }),
        );
        assert_eq!(err, refused_for_an_unknown_org(), "{org_id}");
    }
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
// and worth gating. Since 2026-10-05 they verify LLR-6pmrma, a derived
// low-level requirement of the handler itself, not REQ-he8ejb.
// ---------------------------------------------------------------------------

// verifies: LLR-6pmrma
#[test]
fn revoke_member_rejects_a_short_member_id() {
    let h = harness();
    let err = invoke_err(&h, "revoke_member", revoke_args(&"bb".repeat(31), ""));
    assert_eq!(
        err, "member_id must be 32 bytes (64 hex chars)",
        "the backend must refuse an under-width member id on its own account, \
         not rely on the form having checked"
    );
}

// verifies: LLR-6pmrma
// The doubled-prefix defect it guards is real and the handler's refusal is
// worth keeping; it is not, however, the pre-invocation check REQ-he8ejb
// requires.
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
        revoke_args(&format!("0x0x{}", member_id_64()), ""),
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

// verifies: LLR-6pmrma, LLR-ty85xv
#[test]
fn revoke_member_is_not_refused_for_an_empty_peer_addr() {
    let h = harness_with(TransportModeName::Networked, None);
    let err = invoke_err(&h, "revoke_member", revoke_args(&member_id_64(), ""));
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
    // handling, into the service.
    assert_eq!(
        err,
        refused_for_an_unknown_org(),
        "an empty peer address must not be the ground of refusal in Networked \
         transport: the call must reach the service and fail there, because the \
         harness store holds no such organisation, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Robustness
// ---------------------------------------------------------------------------

// verifies: LLR-8tzbzn, LLR-4wcyqy
// It holds the precondition every REQ above depends on: that the handler list
// is CLOSED, so a command renamed in lib.rs but not in api.ts is an error the
// frontend can see rather than a silent undefined.
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

/// The command paths in the first `generate_handler![...]` list of `source`.
fn handler_list(source: &str) -> std::collections::BTreeSet<String> {
    let start = source.find("generate_handler![").expect("a generate_handler! list") + "generate_handler![".len();
    let end = start + source[start..].find(']').expect("the list is closed");
    source[start..end]
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// `source` with every `//` comment cut off its line. It does not parse block
/// comments or string literals; lib.rs has neither around its handler.
fn without_line_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

// verifies: LLR-4wcyqy
// The harness registers its own copy of the handler list; this test gates the
// product's copy against it. It reads both files as text and edits neither.
#[test]
fn product_registers_the_same_commands_as_the_harness() {
    let read = |rel: &str| {
        std::fs::read_to_string(format!("{}/{rel}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("read {rel}: {e}"))
    };
    let lib = read("src/lib.rs");
    // `Builder::invoke_handler` replaces any earlier handler, so a second call
    // would silently narrow what the product registers. Exactly one list, in
    // exactly one call, keeps the comparison below about the list that wins.
    let code = without_line_comments(&lib);
    assert_eq!(code.matches("generate_handler!").count(), 1, "lib.rs must have exactly one generate_handler!");
    assert_eq!(code.matches(".invoke_handler(").count(), 1, "lib.rs must call invoke_handler exactly once");
    let product = handler_list(&lib);
    let harness = handler_list(&read("tests/ipc.rs"));
    assert_eq!(harness.len(), 12, "the harness registers twelve commands: {harness:?}");
    assert_eq!(product, harness, "lib.rs and the ipc harness register different commands");
}

// verifies: LLR-4wcyqy
// It holds the other precondition: that the managed state is reachable from a
// handler at all, so a failure above is a failure of the behaviour under test
// and not of this suite's wiring.
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
// verified there (org-node/tests/persona_records.rs). These tests verify
// LLR-p7dfxb, this handler's own derived low-level requirement.
// ---------------------------------------------------------------------------

// verifies: LLR-p7dfxb
// The handler parses the handle, name and surname before org-node sees them,
// and stores the parsed (NFC) form.
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

// verifies: LLR-p7dfxb
// An invalid field is refused at the boundary, named, and nothing is created.
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

// ---------------------------------------------------------------------------
// Added 2026-10-05 by the app architecture change (T3), so that each low-level
// requirement of the command surface has a normal and an abnormal case.
//
// The store is fresh, so `revoke_member` fails on any well-formed input: the
// handler's first act after parsing is to look the Organisation's record up,
// and it reports `OrgNotOnChain` before the parsed arguments are used. A normal-case
// test of their argument parsing therefore observes one thing only: that the
// handler did NOT refuse the argument, because the call ended in that refusal
// and no other. It cannot observe what the handler passed on.
// ---------------------------------------------------------------------------

/// A valid iroh `EndpointId`: the compressed Ed25519 base point.
fn endpoint_id() -> iroh::EndpointId {
    let mut bytes = [0x66u8; 32];
    bytes[0] = 0x58;
    iroh::EndpointId::from_bytes(&bytes).expect("the base point is a valid key")
}

/// The postcard bytes of an id-only `EndpointAddr`, the form the app decodes.
fn encoded_endpoint_addr() -> Vec<u8> {
    let addr: iroh::EndpointAddr = endpoint_id().into();
    postcard::to_allocvec(&addr).expect("an EndpointAddr encodes")
}

// verifies: LLR-vqkr5t
#[test]
fn connection_status_reports_the_directory_the_store_was_opened_in() {
    let h = harness();
    let status = invoke(&h, "connection_status", serde_json::json!({})).expect("connection_status");
    assert_eq!(
        status["data_dir"],
        serde_json::json!(h._dir.path().display().to_string()),
        "the reported directory is the one the store was opened in: {status}"
    );
}

// verifies: LLR-85zque, LLR-vqkr5t
#[test]
fn connection_status_reports_a_data_dir_it_had_to_create_verbatim() {
    // Abnormal input: a directory that does not exist yet, two levels deep,
    // with spaces and non-ASCII characters in its name. It is created, the
    // store is opened in it, and it is reported exactly as given.
    let dir = tempfile::tempdir().expect("tempdir");
    let data_dir = dir.path().join("odd dir Ωμέγα").join("nested");
    assert!(!data_dir.exists());
    let h = harness_at(dir, data_dir.clone(), TransportModeName::Networked, None);
    assert!(data_dir.is_dir(), "the data directory is created");
    let status = invoke(&h, "connection_status", serde_json::json!({})).expect("connection_status");
    assert_eq!(status["data_dir"], serde_json::json!(data_dir.display().to_string()));
}

// verifies: LLR-6pmrma
#[test]
fn revoke_member_accepts_a_zero_x_prefixed_member_id() {
    let h = harness();
    let member = format!("0x{}", member_id_64());
    let err = invoke_err(&h, "revoke_member", revoke_args(&member, ""));
    assert_eq!(err, refused_for_an_unknown_org(), "one `0x` prefix is accepted");
}

// verifies: LLR-ty85xv
#[test]
fn revoke_member_treats_a_whitespace_only_peer_addr_as_absent() {
    // Abnormal input: whitespace, which a form field easily carries. It is an
    // absent address, not a malformed one.
    let h = harness();
    for blank in ["   ", "\n", "\t \r\n"] {
        let err = invoke_err(&h, "revoke_member", revoke_args(&member_id_64(), blank));
        assert_eq!(err, refused_for_an_unknown_org(), "{blank:?}");
    }
}

// verifies: LLR-n6twt7
#[test]
fn revoke_member_does_not_refuse_an_encoded_endpoint_addr() {
    let h = harness();
    let addr = hex::encode(encoded_endpoint_addr());
    let err = invoke_err(&h, "revoke_member", revoke_args(&member_id_64(), &addr));
    assert_eq!(err, refused_for_an_unknown_org(), "a decodable address is accepted");
}

// verifies: LLR-n6twt7
#[test]
fn revoke_member_refuses_a_peer_addr_that_is_not_hex() {
    let h = harness();
    for bad in ["zz", "abc", "not hex at all"] {
        let err = invoke_err(&h, "revoke_member", revoke_args(&member_id_64(), bad));
        assert!(err.starts_with("peer_addr_blob hex:"), "{bad:?}: {err}");
    }
}

// verifies: LLR-n6twt7
#[test]
fn revoke_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr() {
    // Abnormal input: well-formed hex whose bytes are not an EndpointAddr.
    // Dropping the address instead of refusing would revoke over discovery
    // when the operator supplied a dial address.
    let h = harness();
    for bad in ["00", "ff", "deadbeef"] {
        let err = invoke_err(&h, "revoke_member", revoke_args(&member_id_64(), bad));
        assert!(err.starts_with("peer_addr decode:"), "{bad:?}: {err}");
    }
}

// verifies: LLR-pguhw5
#[test]
fn list_personas_reports_exactly_the_persona_fields() {
    // No field beyond these six crosses IPC: in particular no seed.
    let h = harness();
    invoke(
        &h,
        "create_persona",
        serde_json::json!({ "handle": "dave", "name": "Dave", "surname": "Jones" }),
    )
    .expect("create_persona");
    let personas = invoke(&h, "list_personas", serde_json::json!({})).expect("list_personas");
    let mut keys: Vec<&str> = personas[0]
        .as_object()
        .expect("a persona is an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["handle", "name", "org_id", "persona_id", "status", "surname"]);
}

// verifies: LLR-pguhw5
#[test]
fn a_persona_in_no_organisation_reports_a_null_org_id() {
    // Boundary value: the persona belongs to no organisation. The field is
    // present and null, not an empty string or a placeholder identifier.
    let h = harness();
    invoke(
        &h,
        "create_persona",
        serde_json::json!({ "handle": "erin", "name": "Erin", "surname": "Jones" }),
    )
    .expect("create_persona");
    let personas = invoke(&h, "list_personas", serde_json::json!({})).expect("list_personas");
    assert!(personas[0].get("org_id").is_some(), "{personas}");
    assert_eq!(personas[0]["org_id"], serde_json::Value::Null, "{personas}");
}

// ---------------------------------------------------------------------------
// The invitation commands (T15 of docs/plans/2026-10-05-chain-authority.md):
// the Invite and its reply are the app's, parsed in `crate::invitation`.
//
// `admit_member` parses the org id, then the peer address, and only then
// the reply. Its normal-case tests hand it the reply `"x"`, which is not a
// Blob, so a call that got past every argument ends in the reply parser's
// refusal and no other.
// ---------------------------------------------------------------------------

/// `export_invite`'s arguments.
fn export_args(org_id: &str) -> serde_json::Value {
    serde_json::json!({ "orgId": org_id, "orgName": "Acme", "inviteeName": "Bob" })
}

/// `admit_member`'s arguments, with the reply `"x"`.
fn admit_args(org_id: &str, peer_addr_blob: &str) -> serde_json::Value {
    serde_json::json!({ "orgId": org_id, "replyBlob": "x", "peerAddrBlob": peer_addr_blob })
}

/// The refusal `admit_member` ends in once every argument before the reply
/// was accepted: the reply parser's own message for `"x"`.
fn refused_for_the_reply() -> String {
    ods_poc_lib::invitation::InviteReply::parse("x").expect_err("\"x\" is not a reply")
}

// verifies: LLR-vzf8j2
#[test]
fn export_invite_rejects_a_short_org_id() {
    let h = harness();
    // 39 characters: one short of the required width.
    let bad = "a".repeat(39);
    let err = invoke_err(&h, "export_invite", export_args(&bad));
    assert_eq!(err, parser_refusal(&bad), "refused by the real handler under `orgId`");
    assert!(err.contains("40 hex chars"), "refused for its WIDTH, got: {err}");
}

// verifies: LLR-vzf8j2
#[test]
fn export_invite_rejects_a_non_hex_org_id() {
    let h = harness();
    let bad = "zz".repeat(20);
    let err = invoke_err(&h, "export_invite", export_args(&bad));
    assert_eq!(err, parser_refusal(&bad));
    assert!(err.contains("hex"), "refused for its ALPHABET, got: {err}");
}

// verifies: LLR-vzf8j2
#[test]
fn export_invite_does_not_refuse_a_well_formed_org_id() {
    // The fresh store holds no Persona bound to it, so the call is refused
    // there, after the parser.
    let h = harness();
    for org_id in [org_id_40(), format!("0x{}", org_id_40())] {
        let err = invoke_err(&h, "export_invite", export_args(&org_id));
        assert_eq!(err, "no Persona of this device belongs to that Organisation", "{org_id}");
    }
}

// verifies: LLR-vzf8j2
#[test]
fn admit_member_refuses_a_malformed_org_id_with_the_parsers_message() {
    let h = harness();
    for bad in malformed_org_ids() {
        let err = invoke_err(&h, "admit_member", admit_args(&bad, ""));
        assert_eq!(err, parser_refusal(&bad), "{bad}");
    }
}

// verifies: LLR-vzf8j2
#[test]
fn admit_member_does_not_refuse_a_well_formed_org_id() {
    let h = harness();
    for org_id in [org_id_40(), format!("0x{}", org_id_40())] {
        let err = invoke_err(&h, "admit_member", admit_args(&org_id, ""));
        assert_eq!(err, refused_for_the_reply(), "{org_id}");
    }
}

// LLR-8krgzj as amended: `admit_member` takes no Organisation secret — an
// `orgSecretHex` a caller still sends is not read, whatever it holds — and a
// blank peer address is the absent one (LLR-ctrfz4).
// verifies: LLR-8krgzj, LLR-ctrfz4
#[test]
fn admit_member_takes_no_organisation_secret() {
    let h = harness();
    for stale in [serde_json::json!("zz".repeat(32)), serde_json::json!("aa"), serde_json::Value::Null] {
        let mut args = admit_args(&org_id_40(), "");
        args["orgSecretHex"] = stale.clone();
        let err = invoke_err(&h, "admit_member", args);
        assert_eq!(err, refused_for_the_reply(), "{stale}");
    }
}

// verifies: LLR-ctrfz4
#[test]
fn admit_member_does_not_refuse_a_blank_or_decodable_peer_addr() {
    let h = harness();
    let addr = hex::encode(encoded_endpoint_addr());
    for peer in ["", "   ", "\n", addr.as_str()] {
        let err = invoke_err(&h, "admit_member", admit_args(&org_id_40(), peer));
        assert_eq!(err, refused_for_the_reply(), "{peer:?}");
    }
}

// verifies: LLR-ctrfz4
#[test]
fn admit_member_refuses_a_peer_addr_that_is_not_hex() {
    let h = harness();
    for bad in ["zz", "abc", "not hex at all"] {
        let err = invoke_err(&h, "admit_member", admit_args(&org_id_40(), bad));
        assert!(err.starts_with("peer_addr_blob hex:"), "{bad:?}: {err}");
    }
}

// verifies: LLR-ctrfz4
#[test]
fn admit_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr() {
    let h = harness();
    for bad in ["00", "ff", "deadbeef"] {
        let err = invoke_err(&h, "admit_member", admit_args(&org_id_40(), bad));
        assert!(err.starts_with("peer_addr decode:"), "{bad:?}: {err}");
    }
}

/// A harness whose data directory holds `outstanding_invites.json` with the
/// one invite id `03…03`, issued for the Organisation `01…01`.
fn harness_with_outstanding_invite() -> Harness {
    let dir = tempfile::tempdir().expect("tempdir");
    let data_dir = dir.path().to_path_buf();
    std::fs::write(
        data_dir.join("outstanding_invites.json"),
        serde_json::json!([{ "org_id": hex::encode([1u8; 20]), "invite_id": hex::encode([3u8; 32]) }]).to_string(),
    )
    .expect("write outstanding_invites.json");
    harness_at(dir, data_dir, TransportModeName::Networked, None)
}

// verifies: LLR-pmus9f
#[test]
fn import_invite_reply_reports_the_reply_it_parses() {
    let h = harness_with_outstanding_invite();
    let (mk, dk) = reply_keys();
    let blob = ods_poc_lib::invitation::InviteReply::wire_for_test(&[1; 20], &[3; 32], &mk, &dk, "bob", "Bob", "Builder");
    let dto = invoke(&h, "import_invite_reply", serde_json::json!({ "blob": blob })).expect("import_invite_reply");
    assert_eq!(
        dto,
        serde_json::json!({
            "org_id": hex::encode([1u8; 20]),
            "handle": "bob",
            "name": "Bob",
            "surname": "Builder",
            "member_key": hex::encode(mk),
            "device_key": hex::encode(dk),
        })
    );
    // It stores nothing: still no Persona, no Organisation, and the same reply
    // parses again (its invite is still outstanding).
    assert_eq!(invoke(&h, "list_personas", serde_json::json!({})).expect("list_personas"), serde_json::json!([]));
    assert_eq!(invoke(&h, "list_orgs", serde_json::json!({})).expect("list_orgs"), serde_json::json!([]));
    invoke(&h, "import_invite_reply", serde_json::json!({ "blob": blob })).expect("still outstanding");
}

// verifies: LLR-pmus9f
#[test]
fn import_invite_reply_refuses_a_malformed_blob_or_an_unknown_invite() {
    let h = harness_with_outstanding_invite();
    let (mk, dk) = reply_keys();
    let malformed = [
        "not base64 !".to_string(),
        ods_poc_lib::invitation::InviteReply::wire_for_test(&[1; 20], &[3; 32], &mk, &dk, "Bob", "Bob", "Builder"),
    ];
    for bad in malformed {
        let err = invoke_err(&h, "import_invite_reply", serde_json::json!({ "blob": bad }));
        let parser = ods_poc_lib::invitation::InviteReply::parse(&bad).expect_err("refused by the parser");
        assert_eq!(err, parser, "the parse's own message");
        assert!(err.starts_with("reply"), "names the field: {err}");
    }
    let unknown = ods_poc_lib::invitation::InviteReply::wire_for_test(&[1; 20], &[4; 32], &mk, &dk, "bob", "Bob", "Builder");
    let err = invoke_err(&h, "import_invite_reply", serde_json::json!({ "blob": unknown }));
    assert_eq!(err, "this reply names no Invite this device has outstanding");
    // The outstanding invite id, under an Organisation it was not issued for
    // (review round 2, finding-1).
    let elsewhere = ods_poc_lib::invitation::InviteReply::wire_for_test(&[2; 20], &[3; 32], &mk, &dk, "bob", "Bob", "Builder");
    let err = invoke_err(&h, "import_invite_reply", serde_json::json!({ "blob": elsewhere }));
    assert_eq!(err, "this reply names no Invite this device has outstanding");
}

// verifies: LLR-w4mhd4
#[test]
fn produce_invite_reply_refuses_without_confirmation_over_ipc() {
    let h = harness();
    let err = invoke_err(
        &h,
        "produce_invite_reply",
        serde_json::json!({ "inviteBlob": "x", "personaId": "p", "confirmed": false }),
    );
    assert!(err.starts_with("confirm first"), "{err}");
}

