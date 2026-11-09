#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The commands driven across the REAL Tauri IPC boundary.
//!
//! Carries: REQ-645jq9 (the transport mode crosses the boundary and is named
//! `transport_mode` on the wire), REQ-sjkp8z (the org-id boundary check is
//! reached by the real handler under the real argument name), REQ-vb5pag
//! (the Person, founding and Invite commands). Each test is annotated with
//! the low-level requirement of `app/docs/architecture/` it verifies.
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

use std::path::PathBuf;
use std::sync::Arc;

use ods_poc_lib::commands;
use ods_poc_lib::policy::{ChainEndpoint, TransportModeName};
use ods_poc_lib::state::AppState;
use org_io::node::{DevicePublicKey, InviteId, MemberId, OrgId};
use org_io::test_support::FakeChain;
use org_io::{Delivery, HeldTarget, OrgIo, Verified, WriteReport};

use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::test::MockRuntime;
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindow};

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
    data_dir: PathBuf,
    transport: TransportModeName,
    chain: Option<ChainEndpoint>,
) -> Harness {
    let state = AppState::for_test(data_dir, chain, transport).expect("AppState::for_test");
    harness_of(dir, state)
}

/// A harness over org-io's substitute chain `chain`, in Loopback mode,
/// where Organisations can be founded, written and verified.
fn harness_over(chain: &FakeChain) -> Harness {
    let dir = tempfile::tempdir().expect("tempdir");
    let state = AppState::for_test_over(dir.path().to_path_buf(), chain, TransportModeName::Loopback)
        .expect("AppState::for_test_over");
    harness_of(dir, state)
}

/// `harness_over` a fresh substitute chain.
fn harness_over_chain() -> Harness {
    harness_over(&FakeChain::new())
}

/// A live mock application managing `state`, whose store lies in `dir`.
fn harness_of(dir: tempfile::TempDir, state: AppState) -> Harness {
    // The SAME handler list as lib.rs, spelled out here because the macro needs
    // the paths at compile time. `product_registers_the_same_commands_as_the_harness`
    // fails if the two lists drift apart.
    let app = mock_builder()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::propose_person,
            commands::propose_founding_organisation,
            commands::issue_invite,
            commands::read_invite,
            commands::propose_member,
            commands::request_admission,
            commands::propose_admission,
            commands::propose_revocation,
            commands::cancel_invite,
            commands::propose_updated_organisation,
            commands::write_organisation_to_chain,
            commands::verify_organisation,
            commands::list_admission_requests,
            commands::list_removed_devices,
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
    harness: &Harness,
    cmd: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    get_ipc_response(&harness.webview, request(cmd, tauri::ipc::InvokeBody::Json(args)))
        .map(|response| response.deserialize::<serde_json::Value>().expect("response is JSON"))
}

/// Invoke `cmd` and require it to have failed, returning the error message.
fn invoke_err(harness: &Harness, cmd: &str, args: serde_json::Value) -> String {
    match invoke(harness, cmd, args) {
        Ok(answer) => panic!("{cmd} unexpectedly succeeded: {answer}"),
        Err(refusal) => refusal.as_str().map(str::to_string).unwrap_or_else(|| refusal.to_string()),
    }
}

/// The harness's shared org-io handle: for a test's set-up through org-io
/// itself, and to read back what a command left in org-io.
fn org_io_of(harness: &Harness) -> Arc<OrgIo> {
    Arc::clone(&harness._app.state::<AppState>().org_io)
}

/// Run one of the handle's futures to completion from a test thread.
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tauri::async_runtime::block_on(future)
}

/// `propose_founding_organisation` over IPC for "Alice Smith" with
/// `handle` and `org_name`; the new Persona's id.
fn propose_founding(harness: &Harness, handle: &str, org_name: &str) -> String {
    let proposed = invoke(
        harness,
        "propose_founding_organisation",
        serde_json::json!({ "name": "Alice", "surname": "Smith", "handle": handle, "orgName": org_name }),
    )
    .expect("propose_founding_organisation");
    proposed["persona_id"].as_str().expect("a Persona id").to_string()
}

/// The Device key org-io lists for the Persona `persona_id`.
fn device_key_of(harness: &Harness, persona_id: &str) -> DevicePublicKey {
    block_on(org_io_of(harness).personas())
        .into_iter()
        .find(|persona| persona.persona_id.as_str() == persona_id)
        .map(|persona| persona.device_key)
        .expect("org-io lists the Persona")
}

/// An Organisation "Acme" founded by the Persona `alice`: proposed over
/// IPC, then written and verified through the handle over the harness's
/// substitute chain. Its id as hex, and the founder's Device key.
fn founded_org(harness: &Harness) -> (String, DevicePublicKey) {
    let persona_id = propose_founding(harness, "alice", "Acme");
    let founder = device_key_of(harness, &persona_id);
    let io = org_io_of(harness);
    let written = block_on(io.write_organisation_to_chain(HeldTarget::Founding(founder)));
    assert!(matches!(written, Ok(WriteReport::Included { .. })), "the founding is written: {written:?}");
    let verified = block_on(io.verify_organisation(HeldTarget::Founding(founder)));
    assert!(matches!(verified, Verified::Adopted { .. }), "the founding is adopted: {verified:?}");
    let org_id = block_on(io.personas())
        .into_iter()
        .find(|persona| persona.device_key == founder)
        .and_then(|persona| persona.org_id)
        .expect("an adopted founding's Persona holds its org id");
    (hex::encode(org_id.as_bytes()), founder)
}

// ---------------------------------------------------------------------------
// REQ-645jq9 — the transport mode crosses the boundary
// ---------------------------------------------------------------------------

// verifies: LLR-8tzbzn
#[test]
fn connection_status_reports_transport_mode_over_ipc() {
    // Loopback, because Networked is what a hard-coded literal would return and
    // the test must be able to tell the two apart.
    let harness = harness_with(TransportModeName::Loopback, None);
    let status = invoke(&harness, "connection_status", serde_json::json!({})).expect("connection_status");
    assert_eq!(
        status.get("transport_mode"),
        Some(&serde_json::Value::String("loopback".into())),
        "the transport mode must cross the IPC boundary as `transport_mode`: {status}"
    );
}

// verifies: LLR-8tzbzn, LLR-ce4hxz
#[test]
fn connection_status_is_registered_under_its_name() {
    let harness = harness();
    let status = invoke(&harness, "connection_status", serde_json::json!({}))
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
// Robustness
// ---------------------------------------------------------------------------

// verifies: LLR-8tzbzn, LLR-ce4hxz
// It holds the precondition every REQ above depends on: that the handler list
// is CLOSED, so a command renamed in lib.rs but not in api.ts is an error the
// frontend can see rather than a silent undefined.
#[test]
fn unknown_command_is_rejected() {
    let harness = harness();
    let res = invoke(&harness, "connectionStatus", serde_json::json!({}));
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
        .filter(|command| !command.is_empty())
        .map(String::from)
        .collect()
}

/// `source` with every `//` comment cut off its line. It does not parse block
/// comments or string literals; the files it reads have neither around what
/// they check.
fn without_line_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

// verifies: LLR-ce4hxz
// The harness registers its own copy of the handler list; this test gates the
// product's copy against it. It reads both files as text and edits neither.
#[test]
fn product_registers_the_same_commands_as_the_harness() {
    let lib = read_source("src/lib.rs");
    // `Builder::invoke_handler` replaces any earlier handler, so a second call
    // would silently narrow what the product registers. Exactly one list, in
    // exactly one call, keeps the comparison below about the list that wins.
    let code = without_line_comments(&lib);
    assert_eq!(code.matches("generate_handler!").count(), 1, "lib.rs must have exactly one generate_handler!");
    assert_eq!(code.matches(".invoke_handler(").count(), 1, "lib.rs must call invoke_handler exactly once");
    let product = handler_list(&lib);
    let harness = handler_list(&read_source("tests/ipc.rs"));
    // LLR-ce4hxz's eighteen commands.
    assert_eq!(harness.len(), 18, "the harness registers eighteen commands: {harness:?}");
    assert_eq!(product, harness, "lib.rs and the ipc harness register different commands");
}

// verifies: LLR-ce4hxz
// It holds the other precondition: that the managed state is reachable from a
// handler at all, so a failure above is a failure of the behaviour under test
// and not of this suite's wiring.
#[test]
fn list_personas_returns_an_empty_list_on_a_fresh_store() {
    let harness = harness();
    let personas = invoke(&harness, "list_personas", serde_json::json!({})).expect("list_personas");
    assert_eq!(
        personas,
        serde_json::json!([]),
        "a fresh store has no personas, and the managed state must be the one \
         the handler reads: {personas}"
    );
}

// ---------------------------------------------------------------------------
// The Person (LLR-k57uqv)
// ---------------------------------------------------------------------------

// verifies: LLR-k57uqv
#[test]
fn propose_person_returns_the_device_key_names_and_provisional() {
    // A `handle` the caller still sends is not read: the Person has none.
    let harness = harness();
    let person = invoke(
        &harness,
        "propose_person",
        serde_json::json!({ "name": "Ada", "surname": "Lovelace", "handle": "ada" }),
    )
    .expect("propose_person");
    let mut keys: Vec<&str> =
        person.as_object().expect("an object").keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["device_public_key", "name", "status", "surname"], "no Handle crosses back: {person}");
    assert_eq!(person["name"], "Ada");
    assert_eq!(person["surname"], "Lovelace");
    assert_eq!(person["status"], "provisional");
    let held = block_on(org_io_of(&harness).person()).expect("org-io holds the Person");
    assert_eq!(person["device_public_key"], serde_json::json!(hex::encode(held.device_key.as_bytes())));
    assert_eq!(person["device_public_key"].as_str().map(str::len), Some(64));
    // A Person is no Persona.
    assert_eq!(invoke(&harness, "list_personas", serde_json::json!({})).expect("list_personas"), serde_json::json!([]));
}

// verifies: LLR-k57uqv
// The name and surname are stored in their parsed (NFC) form.
#[test]
fn propose_person_stores_the_parsed_details() {
    let harness = harness();
    let person = invoke(&harness, "propose_person", serde_json::json!({ "name": "Jose\u{0301}", "surname": "Smith" }))
        .expect("propose_person");
    assert_eq!(person["name"], "Jos\u{e9}");
    let held = block_on(org_io_of(&harness).person()).expect("org-io holds the Person");
    assert_eq!(held.name.to_string(), "Jos\u{e9}");
    assert_eq!(held.surname.to_string(), "Smith");
}

// verifies: LLR-k57uqv
// An invalid field is refused, named, and nothing is created: no Person is
// held and no Person file is written.
#[test]
fn propose_person_refuses_an_invalid_field_naming_it_and_creates_nothing() {
    let harness = harness();
    let long = "a".repeat(129);
    for (name, surname, field) in [(long.as_str(), "Smith", "name:"), ("Alice", long.as_str(), "surname:")] {
        let refusal = invoke(&harness, "propose_person", serde_json::json!({ "name": name, "surname": surname }))
            .expect_err("an invalid field is refused");
        assert_eq!(refusal["kind"], "refused", "{refusal}");
        let err = refusal["message"].as_str().expect("org-io's text");
        assert!(err.starts_with(field), "the refusal must name the field ({field}): {err}");
    }
    assert!(block_on(org_io_of(&harness).person()).is_none(), "a refused Person is not held");
    let person_files =
        org_io::store_file::list_person_stores(harness._dir.path()).expect("org-io lists the data directory's Person files");
    assert!(person_files.is_empty(), "no Person file is written: {person_files:?}");
}

// verifies: LLR-k57uqv
// A refusal crosses IPC as a kind and org-io's text, so the creation form branches on the kind, not
// on how org-io words it. A second Person is `person_already_held`; a
// refused field is `refused`, its text naming the field.
#[test]
fn a_second_person_is_refused_by_its_kind_beside_org_ios_text() {
    let harness = harness();
    let held = invoke(&harness, "propose_person", serde_json::json!({ "name": "Ada", "surname": "Lovelace" }))
        .expect("the first Person is proposed");
    let refusal = invoke(&harness, "propose_person", serde_json::json!({ "name": "Bob", "surname": "Builder" }))
        .expect_err("a second Person is refused");
    let device = held["device_public_key"].as_str().expect("the held Person's Device key");
    assert_eq!(
        refusal,
        serde_json::json!({
            "kind": "person_already_held",
            "message": format!("a Person is held already ({device}); nothing created"),
        })
    );
}

// ---------------------------------------------------------------------------
// The directory the store was opened in (LLR-vqkr5t)
// ---------------------------------------------------------------------------

// verifies: LLR-vqkr5t
#[test]
fn connection_status_reports_the_directory_the_store_was_opened_in() {
    let harness = harness();
    let status = invoke(&harness, "connection_status", serde_json::json!({})).expect("connection_status");
    assert_eq!(
        status["data_dir"],
        serde_json::json!(harness._dir.path().display().to_string()),
        "the reported directory is the one the store was opened in: {status}"
    );
}

// verifies: LLR-vqkr5t
#[test]
fn connection_status_reports_a_data_dir_it_had_to_create_verbatim() {
    // Abnormal input: a directory that does not exist yet, two levels deep,
    // with spaces and non-ASCII characters in its name. It is created, the
    // store is opened in it, and it is reported exactly as given.
    let dir = tempfile::tempdir().expect("tempdir");
    let data_dir = dir.path().join("odd dir Ωμέγα").join("nested");
    assert!(!data_dir.exists());
    let harness = harness_at(dir, data_dir.clone(), TransportModeName::Networked, None);
    assert!(data_dir.is_dir(), "the data directory is created");
    let status = invoke(&harness, "connection_status", serde_json::json!({})).expect("connection_status");
    assert_eq!(status["data_dir"], serde_json::json!(data_dir.display().to_string()));
}

// ---------------------------------------------------------------------------
// The Persona fields (LLR-pguhw5), over a founding's new Persona
// ---------------------------------------------------------------------------

// verifies: LLR-pguhw5
#[test]
fn list_personas_reports_exactly_the_persona_fields() {
    // No field beyond these six crosses IPC: in particular no seed.
    let harness = harness();
    propose_founding(&harness, "dave", "Acme");
    let personas = invoke(&harness, "list_personas", serde_json::json!({})).expect("list_personas");
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
    // Boundary value: a provisional founding's Persona belongs to no
    // Organisation until its proxy exists. The field is present and null,
    // not an empty string or a placeholder identifier.
    let harness = harness();
    propose_founding(&harness, "erin", "Acme");
    let personas = invoke(&harness, "list_personas", serde_json::json!({})).expect("list_personas");
    assert!(personas[0].get("org_id").is_some(), "{personas}");
    assert_eq!(personas[0]["org_id"], serde_json::Value::Null, "{personas}");
}

// ---------------------------------------------------------------------------
// The app holds and passes no randomness (LLR-vj5s28)
// ---------------------------------------------------------------------------

/// The crate names of every dependency table of `manifest`: `[dependencies]`,
/// `[dev-dependencies]`, `[build-dependencies]`, their `target.*` forms and
/// their dotted one-crate tables (`[dependencies.rand]`).
fn dependency_names(manifest: &str) -> Vec<String> {
    const TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
    let mut names = vec![];
    let mut in_table = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            let header = line.trim_matches(|bracket| bracket == '[' || bracket == ']');
            in_table = false;
            for table in TABLES {
                if header == table || header.ends_with(&format!(".{table}")) {
                    in_table = true;
                }
                if let Some(name) = header.strip_prefix(&format!("{table}.")) {
                    names.push(name.to_string());
                }
            }
            continue;
        }
        if !in_table || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            names.push(name.trim().trim_matches('"').to_string());
        }
    }
    names
}

// verifies: LLR-vj5s28
#[test]
fn the_app_manifest_names_no_rand_rand_core_or_getrandom() {
    let manifest = std::fs::read_to_string(format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"))).expect("read Cargo.toml");
    let names = dependency_names(&manifest);
    assert!(names.iter().any(|name| name == "org-io"), "the [dependencies] table was read: {names:?}");
    assert!(names.iter().any(|name| name == "tempfile"), "the [dev-dependencies] table was read: {names:?}");
    let named: Vec<&String> =
        names.iter().filter(|name| ["rand", "rand_core", "getrandom"].contains(&name.as_str())).collect();
    assert!(named.is_empty(), "the app's manifest names {named:?}; org-io holds the one random source");
}

/// The unit's sources under `src-tauri/src`, as (path, text).
fn app_sources() -> Vec<(PathBuf, String)> {
    let mut pending = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")];
    let mut sources = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read src dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = std::fs::read_to_string(&path).expect("read source");
                sources.push((path, text));
            }
        }
    }
    sources
}

// verifies: LLR-vj5s28
#[test]
fn no_app_source_names_a_generator_or_passes_an_rng() {
    let sources = app_sources();
    assert!(sources.iter().any(|(path, _)| path.ends_with("commands.rs")), "the sources were read");
    let mut found = vec![];
    for (path, text) in &sources {
        let code = without_line_comments(text);
        for name in ["OsRng", "RngCore", "CryptoRng", "rand::", "rand_core", "getrandom"] {
            if code.contains(name) {
                found.push(format!("{} names {name}", path.display()));
            }
        }
    }
    assert!(found.is_empty(), "the app names a generator: {found:#?}");
}

// ---------------------------------------------------------------------------
// The founding and the Invite (LLR-6ff735, LLR-y3zykm, LLR-b7ywy4)
// ---------------------------------------------------------------------------

// verifies: LLR-6ff735
#[test]
fn propose_founding_organisation_returns_the_new_personas_id_and_leaves_none_on_refusal() {
    let harness = harness();
    let founding = invoke(
        &harness,
        "propose_founding_organisation",
        serde_json::json!({ "name": "Alice", "surname": "Smith", "handle": "alice", "orgName": "Acme" }),
    )
    .expect("propose_founding_organisation");
    assert_eq!(founding["status"], "provisional_founding", "{founding}");
    let personas = invoke(&harness, "list_personas", serde_json::json!({})).expect("list_personas");
    assert_eq!(personas.as_array().map(Vec::len), Some(1), "{personas}");
    assert_eq!(founding["persona_id"], personas[0]["persona_id"], "the id org-io lists for the founder's Device key");
    // The founder's Device key, 64 hex: the key the founding's write takes
    // (LLR-6ff735).
    let persona_id = founding["persona_id"].as_str().expect("a Persona id");
    assert_eq!(founding["founder"], serde_json::json!(hex::encode(device_key_of(&harness, persona_id).as_bytes())), "{founding}");
    // A second founding is a new Persona, never the first again.
    let second = propose_founding(&harness, "bob", "Beta");
    assert_ne!(serde_json::json!(second), founding["persona_id"]);

    let before = invoke(&harness, "list_personas", serde_json::json!({})).expect("list_personas");
    let long = "a".repeat(129);
    for (name, surname, handle, field) in [
        ("Carol", "Jones", "Carol", "handle:"),
        (long.as_str(), "Jones", "carol", "name:"),
        ("Carol", long.as_str(), "carol", "surname:"),
    ] {
        let err = invoke_err(
            &harness,
            "propose_founding_organisation",
            serde_json::json!({ "name": name, "surname": surname, "handle": handle, "orgName": "Gamma" }),
        );
        assert!(err.starts_with(field), "the refusal names the field ({field}): {err}");
    }
    assert_eq!(
        invoke(&harness, "list_personas", serde_json::json!({})).expect("list_personas"),
        before,
        "a refused founding leaves no new Persona"
    );
}

// verifies: LLR-y3zykm
#[test]
fn issue_invite_returns_org_ios_blob_unchanged_and_takes_no_name_or_address() {
    let harness = harness_over_chain();
    let (org_id, founder) = founded_org(&harness);
    // An Organisation name and a peer address the caller still sends are not
    // read: the Invite carries the stored name and the issuing Device's
    // addresses.
    let blob = invoke(
        &harness,
        "issue_invite",
        serde_json::json!({ "orgId": org_id, "inviteeName": "Bob", "orgName": "Not Acme", "peerAddrBlob": "zz" }),
    )
    .expect("issue_invite");
    let blob = blob.as_str().expect("the Blob as text").to_string();
    let read = block_on(org_io_of(&harness).read_invite(&blob)).expect("org-io parses the Blob it built");
    assert_eq!(read.org_name.to_string(), "Acme", "the stored Organisation name");
    assert_eq!(read.invitee_name.to_string(), "Bob");
    assert_eq!(hex::encode(read.org_id.as_bytes()), org_id);
    assert_eq!(read.issuer, founder, "org-io supplies the issuing Device");
}

// verifies: LLR-y3zykm
// Abnormal. An Organisation this
// node does not hold is refused with org-io's text, unchanged, and no Blob;
// an empty invitee name is org-io's to refuse or take, and either way the
// command passes it on unchanged.
#[test]
fn issue_invite_for_an_organisation_not_held_is_org_ios_refusal_as_its_text() {
    let harness = harness_over_chain();
    let (org_id, _) = founded_org(&harness);
    let elsewhere = OrgId::new([0x5a; 20]);
    let expected = block_on(org_io_of(&harness).issue_invite(elsewhere, "Bob"))
        .expect_err("org-io holds no such Organisation")
        .to_string();
    let err = invoke_err(
        &harness,
        "issue_invite",
        serde_json::json!({ "orgId": hex::encode(elsewhere.as_bytes()), "inviteeName": "Bob" }),
    );
    assert_eq!(err, expected);
    let held = OrgId::new(hex::decode(&org_id).expect("hex").try_into().expect("20 bytes"));
    let direct = block_on(org_io_of(&harness).issue_invite(held, "")).map(|_| ()).map_err(|error| error.to_string());
    let over_ipc = invoke(&harness, "issue_invite", serde_json::json!({ "orgId": org_id, "inviteeName": "" }));
    match (direct, over_ipc) {
        (Err(direct), Err(over_ipc)) => assert_eq!(over_ipc, serde_json::json!(direct)),
        (Ok(()), Ok(blob)) => assert!(blob.is_string(), "{blob}"),
        (direct, over_ipc) => panic!("org-io {direct:?}, the command {over_ipc:?}"),
    }
}

// verifies: LLR-b7ywy4
#[test]
fn read_invite_returns_the_public_fields_and_stores_nothing() {
    let inviter = harness_over_chain();
    let (org_id, founder) = founded_org(&inviter);
    let blob = invoke(&inviter, "issue_invite", serde_json::json!({ "orgId": org_id, "inviteeName": "Bob" }))
        .expect("issue_invite");
    let blob = blob.as_str().expect("the Blob as text").to_string();

    let invitee = harness();
    let personas = invoke(&invitee, "list_personas", serde_json::json!({})).expect("list_personas");
    let orgs = invoke(&invitee, "list_orgs", serde_json::json!({})).expect("list_orgs");
    let dto = invoke(&invitee, "read_invite", serde_json::json!({ "blob": blob })).expect("read_invite");
    assert_eq!(
        dto,
        serde_json::json!({
            "org_name": "Acme",
            "org_id": org_id,
            "invitee_name": "Bob",
            "issuer_device_key": hex::encode(founder.as_bytes()),
        })
    );
    assert_eq!(invoke(&invitee, "list_personas", serde_json::json!({})).expect("list_personas"), personas);
    assert_eq!(invoke(&invitee, "list_orgs", serde_json::json!({})).expect("list_orgs"), orgs);

    // A Blob that does not parse is refused with org-io's own message.
    let io = org_io_of(&invitee);
    let cut = blob[..blob.len() / 2].to_string();
    for bad in ["x".to_string(), cut] {
        let err = invoke_err(&invitee, "read_invite", serde_json::json!({ "blob": bad }));
        let expected = block_on(io.read_invite(&bad)).expect_err("org-io refuses it").to_string();
        assert_eq!(err, expected, "{bad:?}");
        assert!(err.contains("invite"), "the message names the field: {err}");
    }
}

// ---------------------------------------------------------------------------
// The Member's commands (LLR-rg9vc7, LLR-q2k9z8, LLR-8cz828)
// ---------------------------------------------------------------------------

/// An inviter that founded "Acme" over the substitute chain and is
/// receiving, an Invite it issued to "Bob Builder", and an invitee (Loopback,
/// no chain) holding a provisional Member proposed from that Invite with
/// `handles`. `proposed` is `propose_member`'s answer.
struct Exchange {
    inviter: Harness,
    org_id: String,
    blob: String,
    invitee: Harness,
    proposed: serde_json::Value,
}

fn exchange(handles: &[&str]) -> Exchange {
    exchange_over(&FakeChain::new(), handles)
}

/// `exchange`, the inviter over `chain`.
fn exchange_over(chain: &FakeChain, handles: &[&str]) -> Exchange {
    let inviter = harness_over(chain);
    let (org_id, _) = founded_org(&inviter);
    invoke(&inviter, "start_receiver", serde_json::json!({})).expect("start_receiver");
    let blob = invoke(&inviter, "issue_invite", serde_json::json!({ "orgId": org_id, "inviteeName": "Bob Builder" }))
        .expect("issue_invite");
    let blob = blob.as_str().expect("the Blob as text").to_string();
    let invitee = harness_with(TransportModeName::Loopback, None);
    let proposed = invoke(
        &invitee,
        "propose_member",
        serde_json::json!({ "inviteBlob": blob, "name": "Bob", "surname": "Builder", "preferredHandles": handles }),
    )
    .expect("propose_member");
    Exchange { inviter, org_id, blob, invitee, proposed }
}

/// The inviter's kept admission requests once there are `n` of them, or
/// whatever is kept after five seconds.
fn kept_requests(inviter: &Harness, n: usize) -> Vec<org_io::KeptAdmissionRequestSummary> {
    let io = org_io_of(inviter);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let kept = block_on(io.admission_requests());
        if kept.len() >= n || std::time::Instant::now() >= deadline {
            return kept;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

// verifies: LLR-rg9vc7
#[test]
fn propose_member_hands_the_handles_in_order_and_sends_nothing() {
    let ex = exchange(&["bob", "builder", "bobb"]);
    assert_eq!(ex.proposed["status"], "provisional_member", "{}", ex.proposed);
    let personas = invoke(&ex.invitee, "list_personas", serde_json::json!({})).expect("list_personas");
    assert_eq!(personas.as_array().map(Vec::len), Some(1), "{personas}");
    assert_eq!(personas[0]["persona_id"], ex.proposed["persona_id"], "the Persona listed with the new Device key");
    assert_eq!(personas[0]["org_id"], serde_json::json!(ex.org_id), "the Invite's Organisation");
    // It sends nothing: the inviter, receiving, keeps no request.
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kept_requests(&ex.inviter, 0).is_empty(), "propose_member sent nothing");
    // The request the Persona later sends carries the Handles in the order given.
    let outcome = invoke(
        &ex.invitee,
        "request_admission",
        serde_json::json!({ "personaId": ex.proposed["persona_id"], "confirmed": true }),
    )
    .expect("request_admission");
    assert_eq!(outcome, serde_json::json!({ "result": "delivered" }));
    let kept = kept_requests(&ex.inviter, 1);
    assert_eq!(kept.len(), 1, "one kept request");
    let handles: Vec<String> = kept[0].preferred_handles.iter().map(ToString::to_string).collect();
    assert_eq!(handles, ["bob", "builder", "bobb"]);

    // A refused list is org-io's message, and leaves no new Persona.
    let io = org_io_of(&ex.invitee);
    for bad in [vec![], vec!["aa", "bb", "cc", "dd"], vec!["bob", "bob"]] {
        let err = invoke_err(
            &ex.invitee,
            "propose_member",
            serde_json::json!({ "inviteBlob": ex.blob, "name": "Bob", "surname": "Builder", "preferredHandles": bad }),
        );
        let expected = block_on(io.propose_member(&ex.blob, "Bob", "Builder", &bad))
            .expect_err("org-io refuses it")
            .to_string();
        assert_eq!(err, expected, "{bad:?}");
    }
    let err = invoke_err(
        &ex.invitee,
        "propose_member",
        serde_json::json!({ "inviteBlob": "x", "name": "Bob", "surname": "Builder", "preferredHandles": ["bob"] }),
    );
    assert_eq!(err, block_on(io.propose_member("x", "Bob", "Builder", &["bob"])).expect_err("not a Blob").to_string());
    assert_eq!(invoke(&ex.invitee, "list_personas", serde_json::json!({})).expect("list_personas"), personas);
}

// verifies: LLR-q2k9z8
#[test]
fn request_admission_unconfirmed_is_refused_and_asks_org_io_nothing() {
    let ex = exchange(&["bob"]);
    let err = invoke_err(
        &ex.invitee,
        "request_admission",
        serde_json::json!({ "personaId": ex.proposed["persona_id"], "confirmed": false }),
    );
    assert_eq!(err, "the user has not confirmed the admission request");
    std::thread::sleep(std::time::Duration::from_millis(1000));
    assert!(kept_requests(&ex.inviter, 0).is_empty(), "nothing was sent");
}

// verifies: LLR-q2k9z8
#[test]
fn request_admission_for_an_unknown_persona_is_refused() {
    let harness = harness();
    let err = invoke_err(&harness, "request_admission", serde_json::json!({ "personaId": "nobody", "confirmed": true }));
    assert_eq!(err, "no Persona nobody is held here");
    // A Persona that belongs to no Organisation yet (a founding with no
    // proxy) names none to request admission to.
    let founder = propose_founding(&harness, "alice", "Acme");
    let err = invoke_err(&harness, "request_admission", serde_json::json!({ "personaId": founder, "confirmed": true }));
    assert_eq!(err, format!("Persona {founder} belongs to no Organisation"));
}

// verifies: LLR-q2k9z8
#[test]
fn request_admission_refuses_without_confirmation_over_ipc() {
    // Refused before the Persona is looked up: an unknown one is refused
    // for the missing confirmation, not for being unknown.
    let harness = harness();
    let err = invoke_err(&harness, "request_admission", serde_json::json!({ "personaId": "p", "confirmed": false }));
    assert_eq!(err, "the user has not confirmed the admission request");
}

// verifies: LLR-8cz828
#[test]
fn request_admission_tags_each_outcome_and_never_reports_an_used_invite_as_refused() {
    // Every outcome over the real exchange: delivered, then already
    // received, then used by
    // another Device, then refused once the request's Device is a Member,
    // then not delivered once the inviter is gone.
    let ex = exchange(&["bob"]);
    let member = ex.proposed["persona_id"].clone();
    let ask = |harness: &Harness, persona: &serde_json::Value| {
        invoke(harness, "request_admission", serde_json::json!({ "personaId": persona, "confirmed": true }))
            .expect("request_admission")
    };
    assert_eq!(ask(&ex.invitee, &member), serde_json::json!({ "result": "delivered" }));
    assert_eq!(kept_requests(&ex.inviter, 1).len(), 1);
    assert_eq!(ask(&ex.invitee, &member), serde_json::json!({ "result": "already_received" }));
    let other = harness_with(TransportModeName::Loopback, None);
    let other_member = invoke(
        &other,
        "propose_member",
        serde_json::json!({ "inviteBlob": ex.blob, "name": "Eve", "surname": "Other", "preferredHandles": ["eve"] }),
    )
    .expect("propose_member")["persona_id"]
        .clone();
    let used = ask(&other, &other_member);
    assert_eq!(used, serde_json::json!({ "result": "invite_used_by_another_device" }));
    assert_ne!(used["result"], "refused");
    assert_ne!(used["result"], "not_delivered");

    // Admitted, Bob's Device asks again: the receiving Device refuses it.
    let invite_id = hex::encode(kept_requests(&ex.inviter, 1)[0].invite_id.as_bytes());
    invoke(&ex.inviter, "propose_admission", admission_args(&ex.org_id, &invite_id, serde_json::Value::Null, true))
        .expect("propose_admission");
    adopt_next_epoch(&ex.inviter, &ex.org_id);
    assert_eq!(ask(&ex.invitee, &member), serde_json::json!({ "result": "refused" }));

    // The inviter's endpoint closed: not delivered, with org-io's reason.
    let inviter_io = org_io_of(&ex.inviter);
    let inviter_persona = block_on(inviter_io.personas())[0].persona_id.clone();
    block_on(async {
        inviter_io.endpoint_for_test(&inviter_persona).await.expect("bound").inner().close().await;
    });
    let gone = ask(&ex.invitee, &member);
    assert_eq!(gone["result"], "not_delivered", "{gone}");
    let reason = gone["reason"].as_str().expect("a reason");
    let org_id = ods_poc_lib::parsing::parse_org_id(&ex.org_id).expect("an org id");
    match block_on(org_io_of(&ex.invitee).request_admission(org_id)) {
        Ok(Delivery::NotDelivered { reason: direct }) => assert_eq!(reason, direct.to_string(), "org-io's text unchanged"),
        other => panic!("org-io reports the closed inviter as not delivered: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// The Admin's commands (LLR-cr9nmw, LLR-gmb4p8, LLR-7n4hgr, LLR-vqz4s6,
// LLR-r8jeu8)
// ---------------------------------------------------------------------------

/// `exchange(handles)` whose provisional Member sent its admission request
/// and the inviter kept it; the kept request's invite id as hex.
fn requested(handles: &[&str]) -> (Exchange, String) {
    requested_over(&FakeChain::new(), handles)
}

/// `requested`, the inviter over `chain`.
fn requested_over(chain: &FakeChain, handles: &[&str]) -> (Exchange, String) {
    let ex = exchange_over(chain, handles);
    let outcome = invoke(
        &ex.invitee,
        "request_admission",
        serde_json::json!({ "personaId": ex.proposed["persona_id"], "confirmed": true }),
    )
    .expect("request_admission");
    assert_eq!(outcome, serde_json::json!({ "result": "delivered" }));
    let kept = kept_requests(&ex.inviter, 1);
    assert_eq!(kept.len(), 1, "the inviter keeps the request");
    let invite_id = hex::encode(kept[0].invite_id.as_bytes());
    (ex, invite_id)
}

/// The inviter's next epoch, proposed from its provisional changes, written
/// and verified through the handle over its substitute chain; the record
/// adopted.
fn adopt_next_epoch(harness: &Harness, org_id: &str) -> org_io::OrgSummary {
    let io = org_io_of(harness);
    let org = ods_poc_lib::parsing::parse_org_id(org_id).expect("an org id");
    block_on(io.propose_updated_organisation(org)).expect("the next epoch is proposed");
    let written = block_on(io.write_organisation_to_chain(HeldTarget::Organisation(org)));
    assert!(matches!(written, Ok(WriteReport::Included { .. })), "{written:?}");
    let verified = block_on(io.verify_organisation(HeldTarget::Organisation(org)));
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    block_on(io.organisation(org)).expect("the Organisation is held")
}

/// `propose_admission`'s arguments.
fn admission_args(org_id: &str, invite_id: &str, handle: serde_json::Value, confirmed: bool) -> serde_json::Value {
    serde_json::json!({ "orgId": org_id, "inviteId": invite_id, "handle": handle, "confirmed": confirmed })
}

// verifies: LLR-cr9nmw
#[test]
fn propose_admission_unconfirmed_is_refused_and_records_nothing() {
    let (ex, invite_id) = requested(&["bob"]);
    let err = invoke_err(&ex.inviter, "propose_admission", admission_args(&ex.org_id, &invite_id, serde_json::Value::Null, false));
    assert_eq!(err, "the Admin has not confirmed the admission");
    let org = ods_poc_lib::parsing::parse_org_id(&ex.org_id).expect("an org id");
    let nothing = block_on(org_io_of(&ex.inviter).propose_updated_organisation(org));
    assert!(nothing.is_err(), "no provisional change was recorded: {nothing:?}");
    // Confirmed, the admission is recorded, and its epoch admits the Member.
    invoke(&ex.inviter, "propose_admission", admission_args(&ex.org_id, &invite_id, serde_json::Value::Null, true))
        .expect("propose_admission");
    assert_eq!(adopt_next_epoch(&ex.inviter, &ex.org_id).members.len(), 2);
}

// verifies: LLR-gmb4p8
#[test]
fn a_blank_handle_is_handed_on_as_none_and_another_unchanged() {
    // Every case over the exchange. A blank Handle lets org-io choose the first preferred one free; a
    // chosen one, handed on when no preferred one is free (the founder holds
    // `alice`), is the Member's Handle unchanged.
    let cases: [(&[&str], serde_json::Value, &str); 6] = [
        (&["bob", "builder"], serde_json::Value::Null, "bob"),
        (&["bob", "builder"], serde_json::json!(""), "bob"),
        (&["bob", "builder"], serde_json::json!("   "), "bob"),
        (&["bob", "builder"], serde_json::json!("\t\n"), "bob"),
        (&["alice", "bob"], serde_json::json!(" "), "bob"),
        (&["alice"], serde_json::json!("builder"), "builder"),
    ];
    for (preferred, handle, member_handle) in cases {
        let (ex, invite_id) = requested(preferred);
        invoke(&ex.inviter, "propose_admission", admission_args(&ex.org_id, &invite_id, handle.clone(), true))
            .expect("propose_admission");
        let adopted = adopt_next_epoch(&ex.inviter, &ex.org_id);
        let handles: Vec<String> = adopted.members.iter().map(|member| member.handle.to_string()).collect();
        assert!(handles.contains(&member_handle.to_string()), "{handle} admits {member_handle}: {handles:?}");
    }
    // Abnormal: none chosen and none of the preferred free is org-io's
    // refusal, as its text; a non-blank Handle is handed on unchanged, so
    // one org-io cannot parse is refused by org-io, not trimmed here.
    for (preferred, handle) in [(&["alice"][..], serde_json::json!("  ")), (&["alice"][..], serde_json::json!(" bob "))] {
        let (ex, invite_id) = requested(preferred);
        let err = invoke_err(&ex.inviter, "propose_admission", admission_args(&ex.org_id, &invite_id, handle.clone(), true));
        let org = ods_poc_lib::parsing::parse_org_id(&ex.org_id).expect("an org id");
        let invite = InviteId::new(hex::decode(&invite_id).expect("hex").try_into().expect("32 bytes"));
        let chosen = handle.as_str().filter(|text| !text.trim().is_empty());
        let expected = block_on(org_io_of(&ex.inviter).propose_admission(org, invite, chosen))
            .expect_err("org-io refuses it")
            .to_string();
        assert_eq!(err, expected, "{handle}");
    }
}

// verifies: LLR-gmb4p8
#[test]
fn propose_admission_takes_no_organisation_secret_reply_or_peer_address() {
    // Whatever a caller sends under other names is not read: the
    // call ends in org-io's refusal for the four values alone.
    let harness = harness();
    let expected = block_on(org_io_of(&harness).propose_admission(OrgId::new([0xaa; 20]), InviteId::new([0xcc; 32]), None))
        .expect_err("no Organisation is held")
        .to_string();
    for stale in [serde_json::json!("zz".repeat(32)), serde_json::json!("aa"), serde_json::Value::Null] {
        let mut args = admission_args(&"aa".repeat(20), &"cc".repeat(32), serde_json::Value::Null, true);
        args["orgSecretHex"] = stale.clone();
        args["replyBlob"] = stale.clone();
        args["peerAddrBlob"] = stale.clone();
        assert_eq!(invoke_err(&harness, "propose_admission", args), expected, "{stale}");
    }
}

// verifies: LLR-7n4hgr
#[test]
fn propose_revocation_parses_the_member_id_and_takes_no_peer_address() {
    let (ex, invite_id) = requested(&["bob"]);
    invoke(&ex.inviter, "propose_admission", admission_args(&ex.org_id, &invite_id, serde_json::Value::Null, true))
        .expect("propose_admission");
    let adopted = adopt_next_epoch(&ex.inviter, &ex.org_id);
    let bob = adopted.members.iter().find(|member| member.handle.to_string() == "bob").expect("Bob is a Member").id;
    let org = ods_poc_lib::parsing::parse_org_id(&ex.org_id).expect("an org id");
    let io = org_io_of(&ex.inviter);
    // A Member id the record does not hold: org-io's refusal, for exactly
    // the id the text names, whatever its prefix or case; no address read.
    let expected = block_on(io.propose_revocation(org, MemberId::new([0xbb; 32])))
        .expect_err("not a Member")
        .to_string();
    for member in ["bb".repeat(32), format!("0x{}", "bb".repeat(32)), "BB".repeat(32)] {
        let err = invoke_err(
            &ex.inviter,
            "propose_revocation",
            serde_json::json!({ "orgId": ex.org_id, "memberId": member, "peerAddrBlob": "zz" }),
        );
        assert_eq!(err, expected, "{member}");
    }
    // Bob's id: recorded, and the next epoch can be proposed from it.
    let recorded = invoke(
        &ex.inviter,
        "propose_revocation",
        serde_json::json!({ "orgId": ex.org_id, "memberId": hex::encode(bob.as_bytes()) }),
    );
    assert_eq!(recorded, Ok(serde_json::Value::Null));
    assert!(block_on(io.propose_updated_organisation(org)).is_ok(), "the revocation is a provisional change");
}

// verifies: LLR-7n4hgr
#[test]
fn propose_revocation_rejects_a_short_member_id() {
    let harness = harness();
    let err = invoke_err(
        &harness,
        "propose_revocation",
        serde_json::json!({ "orgId": "aa".repeat(20), "memberId": "bb".repeat(31) }),
    );
    assert_eq!(
        err, "member_id must be 32 bytes (64 hex chars)",
        "the backend refuses an under-width member id on its own account"
    );
}

// verifies: LLR-7n4hgr
// `0x0x` + 64 hex characters: one strip leaves `0x` + 64, which the hex
// decode refuses, at this boundary.
#[test]
fn propose_revocation_rejects_a_doubled_zero_x_prefix_on_the_member_id() {
    let harness = harness();
    let err = invoke_err(
        &harness,
        "propose_revocation",
        serde_json::json!({ "orgId": "aa".repeat(20), "memberId": format!("0x0x{}", "bb".repeat(32)) }),
    );
    assert!(err.starts_with("member_id hex"), "exactly one `0x` prefix is permitted: {err}");
}

// verifies: LLR-7n4hgr
#[test]
fn propose_revocation_accepts_a_zero_x_prefixed_member_id() {
    let harness = harness();
    let expected = block_on(org_io_of(&harness).propose_revocation(OrgId::new([0xaa; 20]), MemberId::new([0xbb; 32])))
        .expect_err("no Organisation is held")
        .to_string();
    let err = invoke_err(
        &harness,
        "propose_revocation",
        serde_json::json!({ "orgId": "aa".repeat(20), "memberId": format!("0x{}", "bb".repeat(32)) }),
    );
    assert_eq!(err, expected, "one `0x` prefix is accepted and the call reaches org-io");
}

// verifies: LLR-vqz4s6
#[test]
fn cancel_invite_calls_org_io_once_and_returns_its_refusal_as_text() {
    let (ex, invite_id) = requested(&["bob"]);
    let org = ods_poc_lib::parsing::parse_org_id(&ex.org_id).expect("an org id");
    let io = org_io_of(&ex.inviter);
    let cancel = |invite: &str| invoke(&ex.inviter, "cancel_invite", serde_json::json!({ "orgId": ex.org_id, "inviteId": invite }));
    // An invite with no kept request: org-io's refusal, as its text.
    let expected = block_on(io.cancel_invite(org, InviteId::new([0xdd; 32]))).expect_err("no kept request").to_string();
    assert_eq!(cancel(&"dd".repeat(32)), Err(serde_json::json!(expected)));
    // The kept request's invite: cancelled, nothing returned, the request gone.
    assert_eq!(cancel(&invite_id), Ok(serde_json::Value::Null));
    assert!(block_on(io.admission_requests()).is_empty(), "the kept request is removed");
    // Once: cancelling again is org-io's refusal.
    let invite = ods_poc_lib::parsing::parse_invite_id(&invite_id).expect("an invite id");
    let again = block_on(io.cancel_invite(org, invite)).expect_err("already cancelled").to_string();
    assert_eq!(cancel(&invite_id), Err(serde_json::json!(again)));
}

// verifies: LLR-r8jeu8
#[test]
fn an_invite_id_that_is_not_hex_or_not_32_bytes_is_refused_before_org_io() {
    let harness = harness();
    for cmd in ["propose_admission", "cancel_invite"] {
        let args = |invite: &str| admission_args(&"aa".repeat(20), invite, serde_json::Value::Null, true);
        for not_hex in ["zz".repeat(32), format!("0x0x{}", "cc".repeat(32))] {
            let err = invoke_err(&harness, cmd, args(&not_hex));
            assert!(err.starts_with("invite_id hex"), "{cmd} {not_hex}: {err}");
        }
        for wrong_width in ["cc".repeat(31), "cc".repeat(33), String::new()] {
            let err = invoke_err(&harness, cmd, args(&wrong_width));
            assert_eq!(err, "invite_id must be 32 bytes (64 hex chars)", "{cmd} {wrong_width:?}");
        }
        // Well formed, with or without one `0x`: the call reaches org-io.
        for well_formed in ["cc".repeat(32), format!("0x{}", "cc".repeat(32))] {
            let err = invoke_err(&harness, cmd, args(&well_formed));
            assert!(!err.starts_with("invite_id"), "{cmd} {well_formed}: refused by the parser: {err}");
        }
    }
}

// ---------------------------------------------------------------------------
// Proposing the updated Organisation and writing it (LLR-2u3cga,
// LLR-rhc555)
// ---------------------------------------------------------------------------

// verifies: LLR-2u3cga
#[test]
fn propose_updated_organisation_returns_epoch_root_and_key_as_hex_and_writes_no_chain() {
    let chain = FakeChain::new();
    let (ex, invite_id) = requested_over(&chain, &["bob"]);
    let org = ods_poc_lib::parsing::parse_org_id(&ex.org_id).expect("an org id");
    let io = org_io_of(&ex.inviter);
    // No provisional change yet: org-io's refusal, as its text.
    let refusal = block_on(io.propose_updated_organisation(org)).expect_err("no provisional change").to_string();
    assert_eq!(invoke_err(&ex.inviter, "propose_updated_organisation", serde_json::json!({ "orgId": ex.org_id })), refusal);

    invoke(&ex.inviter, "propose_admission", admission_args(&ex.org_id, &invite_id, serde_json::Value::Null, true))
        .expect("propose_admission");
    let writes = chain.update_writes();
    let proposed = invoke(&ex.inviter, "propose_updated_organisation", serde_json::json!({ "orgId": ex.org_id }))
        .expect("propose_updated_organisation");
    let mut keys: Vec<&str> = proposed.as_object().expect("an object").keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["epoch", "org_public_key", "root"], "{proposed}");
    assert_eq!(proposed["epoch"], 2);
    assert_eq!(proposed["root"].as_str().map(str::len), Some(64));
    assert_eq!(proposed["org_public_key"].as_str().map(str::len), Some(64));
    assert_eq!(chain.update_writes(), writes, "no chain write");
    assert_eq!(block_on(io.organisation(org)).expect("held").epoch.get(), 1, "the record is unchanged");

    // What it reported is org-io's proposal: the epoch written and adopted
    // from it carries that root and key.
    let written = block_on(io.write_organisation_to_chain(HeldTarget::Organisation(org)));
    assert!(matches!(written, Ok(WriteReport::Included { .. })), "{written:?}");
    let verified = block_on(io.verify_organisation(HeldTarget::Organisation(org)));
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    let record = block_on(io.organisation(org)).expect("held");
    assert_eq!(record.epoch.get(), 2);
    assert_eq!(serde_json::json!(hex::encode(record.root_hash.as_bytes())), proposed["root"]);
    assert_eq!(serde_json::json!(hex::encode(record.org_pub_key.as_bytes())), proposed["org_public_key"]);
}

/// `write_organisation_to_chain`'s refusal of both or neither target.
const EXACTLY_ONE_TARGET: &str = "write_organisation_to_chain takes exactly one of org_id or founder";

// verifies: LLR-rhc555
#[test]
fn write_organisation_to_chain_takes_exactly_one_of_org_id_or_founder() {
    let harness = harness();
    let founder = hex::encode(device_key_of(&harness, &propose_founding(&harness, "alice", "Acme")).as_bytes());
    for args in [serde_json::json!({}), serde_json::json!({ "orgId": "aa".repeat(20), "founder": founder })] {
        assert_eq!(invoke_err(&harness, "write_organisation_to_chain", args.clone()), EXACTLY_ONE_TARGET, "{args}");
    }
    // Exactly one: the call reaches org-io, whatever it then reports.
    for args in [serde_json::json!({ "orgId": "aa".repeat(20) }), serde_json::json!({ "founder": founder })] {
        let outcome = invoke(&harness, "write_organisation_to_chain", args.clone());
        assert_ne!(outcome, Err(serde_json::json!(EXACTLY_ONE_TARGET)), "{args}");
    }
    // A founder that is not 32 bytes of hex is refused at the edge.
    let not_hex = invoke_err(&harness, "write_organisation_to_chain", serde_json::json!({ "founder": "zz".repeat(32) }));
    assert!(not_hex.starts_with("founder hex"), "{not_hex}");
    assert_eq!(
        invoke_err(&harness, "write_organisation_to_chain", serde_json::json!({ "founder": "aa".repeat(31) })),
        "founder must be 32 bytes (64 hex chars)"
    );
}

// ---------------------------------------------------------------------------
// Verification over IPC (LLR-jf63e4, LLR-g8wfy9)
// ---------------------------------------------------------------------------

/// Seed the store the harness will open in `data_dir` with an Organisation
/// founded, written and verified over a substitute chain; its id as hex.
/// The seeding handle is dropped before the harness opens the same files
/// with the chain not configured.
fn seed_store(data_dir: &std::path::Path) -> String {
    org_io::test_support::MemoryKeychain::shared().install();
    let chain = FakeChain::new();
    let io = OrgIo::open_for_test(data_dir, org_io::transport::TransportMode::Networked, &chain).expect("open");
    let founder = block_on(io.propose_founding_organisation("Test", "User", "alice", "Acme")).expect("proposed").founder;
    let written = block_on(io.write_organisation_to_chain(HeldTarget::Founding(founder)));
    assert!(matches!(written, Ok(WriteReport::Included { .. })), "{written:?}");
    let verified = block_on(io.verify_organisation(HeldTarget::Founding(founder)));
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    let org_id = block_on(io.personas())
        .into_iter()
        .find(|persona| persona.device_key == founder)
        .and_then(|persona| persona.org_id)
        .expect("the founding's org id");
    hex::encode(org_id.as_bytes())
}

/// A harness over a seeded store, chain not configured, and the seeded
/// Organisation's id as hex.
fn seeded_harness() -> (Harness, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let data_dir = dir.path().to_path_buf();
    let org_id = seed_store(&data_dir);
    (harness_at(dir, data_dir, TransportModeName::Networked, None), org_id)
}

// verifies: LLR-g8wfy9
#[test]
fn list_orgs_carries_each_verification_status_or_not_checked() {
    let chain = FakeChain::new();
    let harness = harness_over(&chain);
    let (org_id, _) = founded_org(&harness);
    let status = |harness: &Harness| invoke(harness, "list_orgs", serde_json::json!({})).expect("list_orgs")[0]["verification"].clone();
    // The founding was verified by its founder's key; the Organisation's own
    // target has not been verified on this handle.
    assert_eq!(status(&harness), serde_json::json!({ "status": "not_checked" }));
    invoke(&harness, "verify_organisation", serde_json::json!({ "orgId": org_id })).expect("verify_organisation");
    assert_eq!(status(&harness), serde_json::json!({ "status": "in_step", "epoch": 1 }));
    chain.epoch_ahead.store(true, std::sync::atomic::Ordering::SeqCst);
    let behind = invoke(&harness, "verify_organisation", serde_json::json!({ "orgId": org_id })).expect("verify_organisation");
    assert_eq!(status(&harness), serde_json::json!({ "status": "behind", "chain_epoch": behind[0]["chain_epoch"] }));
    chain.epoch_ahead.store(false, std::sync::atomic::Ordering::SeqCst);
    chain.read_failing.store(true, std::sync::atomic::Ordering::SeqCst);
    let unverified = invoke(&harness, "verify_organisation", serde_json::json!({ "orgId": org_id })).expect("verify_organisation");
    assert_eq!(status(&harness), serde_json::json!({ "status": "unverified", "reason": unverified[0]["reason"] }));
    // The other fields are unchanged.
    let org = &invoke(&harness, "list_orgs", serde_json::json!({})).expect("list_orgs")[0];
    assert_eq!(org["org_id"], serde_json::json!(org_id));
    assert_eq!(org["epoch"], 1);
    assert_eq!(org["member_count"], 1);
}

// verifies: LLR-jf63e4, LLR-g8wfy9
#[test]
fn verify_organisation_with_the_chain_not_configured_reports_every_organisation_unverified() {
    let (harness, org_id) = seeded_harness();
    let results = invoke(&harness, "verify_organisation", serde_json::json!({})).expect("verify_organisation");
    let results = results.as_array().expect("a list");
    assert_eq!(results.len(), 1, "one result per held Organisation: {results:?}");
    assert_eq!(results[0]["result"], "unverified", "{results:?}");
    assert_eq!(results[0]["org_id"], org_id.as_str());
    let reason = results[0]["reason"].as_str().expect("a reason");
    assert!(!reason.is_empty());
    let orgs = invoke(&harness, "list_orgs", serde_json::json!({})).expect("list_orgs");
    assert_eq!(orgs[0]["verification"], serde_json::json!({ "status": "unverified", "reason": reason }));
}

// verifies: LLR-jf63e4
#[test]
fn verify_organisation_on_an_empty_store_reports_nothing() {
    let harness = harness();
    let results = invoke(&harness, "verify_organisation", serde_json::json!({})).expect("verify_organisation");
    assert_eq!(results, serde_json::json!([]));
}

// verifies: LLR-g8wfy9
#[test]
fn list_orgs_reports_not_checked_before_any_reconcile() {
    let (harness, org_id) = seeded_harness();
    let orgs = invoke(&harness, "list_orgs", serde_json::json!({})).expect("list_orgs");
    assert_eq!(orgs[0]["org_id"], org_id.as_str());
    assert_eq!(orgs[0]["verification"], serde_json::json!({ "status": "not_checked" }));
}

// ---------------------------------------------------------------------------
// The kept admission requests, the handler set, the org-id parse
// (LLR-y7nc6x, LLR-ce4hxz, LLR-97hujy)
// ---------------------------------------------------------------------------

/// A source file of this crate, read as text.
fn read_source(rel: &str) -> String {
    std::fs::read_to_string(format!("{}/{rel}", env!("CARGO_MANIFEST_DIR"))).unwrap_or_else(|error| panic!("read {rel}: {error}"))
}

/// Malformed org ids: under width, over width, and the right width but not hex.
fn malformed_org_ids() -> [String; 3] {
    ["a".repeat(39), "a".repeat(42), "zz".repeat(20)]
}

/// The parser's own refusal of `org_id`, so the tests below require the
/// handler's refusal to be the parser's and no other.
fn parser_refusal(org_id: &str) -> String {
    ods_poc_lib::parsing::parse_org_id(org_id).expect_err("a malformed org id is refused")
}

/// LLR-ce4hxz's eighteen commands.
const EIGHTEEN: [&str; 18] = [
    "propose_person",
    "propose_founding_organisation",
    "issue_invite",
    "read_invite",
    "propose_member",
    "request_admission",
    "propose_admission",
    "propose_revocation",
    "cancel_invite",
    "propose_updated_organisation",
    "write_organisation_to_chain",
    "verify_organisation",
    "list_admission_requests",
    "list_removed_devices",
    "list_personas",
    "list_orgs",
    "connection_status",
    "start_receiver",
];

// verifies: LLR-y7nc6x
#[test]
fn list_admission_requests_returns_each_kept_request_with_the_events_fields_and_changes_nothing() {
    let fresh = harness();
    assert_eq!(invoke(&fresh, "list_admission_requests", serde_json::json!({})).expect("list"), serde_json::json!([]));

    let (ex, invite_id) = requested(&["bob", "builder"]);
    let member_device = device_key_of(&ex.invitee, ex.proposed["persona_id"].as_str().expect("an id"));
    let personas = invoke(&ex.inviter, "list_personas", serde_json::json!({})).expect("list_personas");
    let orgs = invoke(&ex.inviter, "list_orgs", serde_json::json!({})).expect("list_orgs");
    let listed = invoke(&ex.inviter, "list_admission_requests", serde_json::json!({})).expect("list_admission_requests");
    assert_eq!(
        listed,
        serde_json::json!([{
            "org_id": ex.org_id,
            "invite_id": invite_id,
            "invitee_name": "Bob Builder",
            "name": "Bob",
            "surname": "Builder",
            "preferred_handles": ["bob", "builder"],
            "device_public_key": hex::encode(member_device.as_bytes()),
        }])
    );
    // It changes nothing: the same list again, the same Personas and Organisations.
    assert_eq!(invoke(&ex.inviter, "list_admission_requests", serde_json::json!({})).expect("again"), listed);
    assert_eq!(invoke(&ex.inviter, "list_personas", serde_json::json!({})).expect("list_personas"), personas);
    assert_eq!(invoke(&ex.inviter, "list_orgs", serde_json::json!({})).expect("list_orgs"), orgs);
}

// verifies: LLR-ce4hxz
#[test]
fn the_harness_registers_exactly_the_eighteen_commands_and_refuses_the_old_names() {
    let expected: std::collections::BTreeSet<String> = EIGHTEEN.iter().map(|name| format!("commands::{name}")).collect();
    assert_eq!(handler_list(&read_source("tests/ipc.rs")), expected, "the IPC suite's list");
    assert_eq!(handler_list(&read_source("src/lib.rs")), expected, "the product's list");
    let harness = harness();
    let unknown = invoke_err(&harness, "no_such_command", serde_json::json!({}));
    for old in [
        "send_organisation",
        "request_organisation",
        "reconcile_organisations",
        "produce_invite_reply",
        "import_invite_reply",
        "admit_member",
        "revoke_member",
        "create_persona",
        "create_organisation",
        "export_invite",
        "import_invite",
    ] {
        let refused = invoke_err(&harness, old, serde_json::json!({}));
        assert_eq!(refused, unknown.replace("no_such_command", old), "{old} is refused as unregistered");
    }
    // The argument-free ones answer.
    for name in ["list_personas", "list_orgs", "connection_status", "list_admission_requests"] {
        invoke(&harness, name, serde_json::json!({})).unwrap_or_else(|refusal| panic!("{name} is registered: {refusal}"));
    }
}

// Abnormal, both: an Organisation the fresh store does not hold is refused
// as org-io refuses it, and a malformed Organisation identifier is refused
// at the IPC edge with the parser's text. The normal listing is the next
// test's.
// verifies: LLR-dmfvv6
#[test]
fn list_removed_devices_refuses_an_unheld_or_malformed_organisation() {
    let harness = harness();
    let unheld = invoke_err(&harness, "list_removed_devices", serde_json::json!({ "orgId": "ab".repeat(20) }));
    let org_io_refusal = block_on(org_io_of(&harness).removed_devices(OrgId::new([0xab; 20])))
        .expect_err("org-io refuses an unheld Organisation")
        .to_string();
    assert_eq!(unheld, org_io_refusal, "org-io's refusal as its text");
    for bad in malformed_org_ids() {
        assert_eq!(invoke_err(&harness, "list_removed_devices", serde_json::json!({ "orgId": bad })), parser_refusal(&bad), "{bad}");
    }
}

// verifies: LLR-dmfvv6
// The real exchange produces the listing's three backend.json answers (none,
// one_unacknowledged, one_acknowledged), so the webview's fake cannot drift
// from them.
#[test]
fn the_removed_devices_answers_have_the_fakes_shapes() {
    let chain = FakeChain::new();
    let inviter = harness_over(&chain);
    let (org_id, _) = founded_org(&inviter);
    let org = ods_poc_lib::parsing::parse_org_id(&org_id).expect("an org id");
    invoke(&inviter, "start_receiver", serde_json::json!({})).expect("start_receiver");
    let none = invoke(&inviter, "list_removed_devices", serde_json::json!({ "orgId": org_id })).expect("list_removed_devices");
    assert_backend_holds("answers", "list_removed_devices", "none", &none);

    let blob = invoke(&inviter, "issue_invite", serde_json::json!({ "orgId": org_id, "inviteeName": "Bob Builder" }))
        .expect("issue_invite");
    let invitee = harness_over(&FakeChain::sharing_slots_with(&chain));
    let member = invoke(
        &invitee,
        "propose_member",
        serde_json::json!({ "inviteBlob": blob, "name": "Bob", "surname": "Builder", "preferredHandles": ["bob"] }),
    )
    .expect("propose_member")["persona_id"]
        .clone();
    invoke(&invitee, "start_receiver", serde_json::json!({})).expect("start_receiver");
    let asked = invoke(&invitee, "request_admission", serde_json::json!({ "personaId": member, "confirmed": true }))
        .expect("request_admission");
    assert_eq!(asked, serde_json::json!({ "result": "delivered" }));
    let invite_id = hex::encode(kept_requests(&inviter, 1)[0].invite_id.as_bytes());
    invoke(&inviter, "propose_admission", admission_args(&org_id, &invite_id, serde_json::Value::Null, true))
        .expect("propose_admission");
    let admitted = adopt_next_epoch(&inviter, &org_id);
    let inviter_io = org_io_of(&inviter);
    block_on(inviter_io.send_organisation(org)).expect("the epoch admitting Bob is sent");
    let invitee_io = org_io_of(&invitee);
    within_seconds(30, "Bob adopts his admission", || block_on(invitee_io.organisation(org)).is_some());

    let bob = admitted.members.iter().find(|listed| listed.handle.to_string() == "bob").expect("Bob is a Member").id;
    invoke(&inviter, "propose_revocation", serde_json::json!({ "orgId": org_id, "memberId": hex::encode(bob.as_bytes()) }))
        .expect("propose_revocation");
    adopt_next_epoch(&inviter, &org_id);
    let unacknowledged = invoke(&inviter, "list_removed_devices", serde_json::json!({ "orgId": org_id })).expect("list_removed_devices");
    assert_backend_holds("answers", "list_removed_devices", "one_unacknowledged", &unacknowledged);
    block_on(inviter_io.send_organisation(org)).expect("the epoch removing Bob is sent");
    within_seconds(30, "the inviter receives Bob's acknowledgement", || {
        // The inviter holds the Organisation, so a refusal is a failure now,
        // not a "not yet".
        block_on(inviter_io.removed_devices(org))
            .expect("org-io lists the removed Devices of a held Organisation")
            .iter()
            .any(|removed| removed.acknowledged_epoch.is_some())
    });
    let listed = invoke(&inviter, "list_removed_devices", serde_json::json!({ "orgId": org_id })).expect("list_removed_devices");
    assert_backend_holds("answers", "list_removed_devices", "one_acknowledged", &listed);
}

// A list of one entry or none cannot tell org-io's order from any other, so
// two Members are removed here.
// verifies: LLR-dmfvv6
#[test]
fn list_removed_devices_answers_two_devices_in_org_ios_order() {
    let (ex, _) = requested(&["bob"]);
    let carol_blob = invoke(&ex.inviter, "issue_invite", serde_json::json!({ "orgId": ex.org_id, "inviteeName": "Carol Jones" }))
        .expect("issue_invite");
    let carol = harness_with(TransportModeName::Loopback, None);
    let carol_persona = invoke(
        &carol,
        "propose_member",
        serde_json::json!({ "inviteBlob": carol_blob, "name": "Carol", "surname": "Jones", "preferredHandles": ["carol"] }),
    )
    .expect("propose_member")["persona_id"]
        .clone();
    let asked = invoke(&carol, "request_admission", serde_json::json!({ "personaId": carol_persona, "confirmed": true }))
        .expect("request_admission");
    assert_eq!(asked, serde_json::json!({ "result": "delivered" }));
    let kept = kept_requests(&ex.inviter, 2);
    assert_eq!(kept.len(), 2, "the inviter keeps both requests");
    for request in &kept {
        let invite_id = hex::encode(request.invite_id.as_bytes());
        invoke(&ex.inviter, "propose_admission", admission_args(&ex.org_id, &invite_id, serde_json::Value::Null, true))
            .unwrap_or_else(|refusal| panic!("propose_admission {invite_id}: {refusal}"));
    }
    let admitted = adopt_next_epoch(&ex.inviter, &ex.org_id);
    for handle in ["bob", "carol"] {
        let member = admitted.members.iter().find(|listed| listed.handle.to_string() == handle).expect("a Member").id;
        invoke(&ex.inviter, "propose_revocation", serde_json::json!({ "orgId": ex.org_id, "memberId": hex::encode(member.as_bytes()) }))
            .expect("propose_revocation");
    }
    adopt_next_epoch(&ex.inviter, &ex.org_id);

    let org = ods_poc_lib::parsing::parse_org_id(&ex.org_id).expect("an org id");
    let in_org_ios_order: Vec<serde_json::Value> = block_on(org_io_of(&ex.inviter).removed_devices(org))
        .expect("org-io lists a held Organisation's removed Devices")
        .iter()
        .map(|removed| {
            serde_json::json!({
                "member_id": hex::encode(removed.member_id.as_bytes()),
                "device_key": hex::encode(removed.device_key.as_bytes()),
                "acknowledged_epoch": removed.acknowledged_epoch.map(|epoch| epoch.get()),
            })
        })
        .collect();
    assert_eq!(in_org_ios_order.len(), 2, "both Members' Devices are removed");
    let listed = invoke(&ex.inviter, "list_removed_devices", serde_json::json!({ "orgId": ex.org_id })).expect("list_removed_devices");
    assert_eq!(listed, serde_json::Value::Array(in_org_ios_order));
}

/// Wait until `holds`, polling, or fail naming `what` after `seconds`.
fn within_seconds(seconds: u64, what: &str, mut holds: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    while !holds() {
        assert!(std::time::Instant::now() < deadline, "{what}: not within {seconds} s");
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

// verifies: LLR-97hujy
#[test]
fn every_org_id_taking_command_parses_it_first_with_the_parsers_message() {
    // Every other argument is malformed too, so a refusal that is the
    // parser's message for the org id shows it was parsed first.
    let harness = harness();
    for bad in malformed_org_ids() {
        let refusal = parser_refusal(&bad);
        for (cmd, args) in [
            ("issue_invite", serde_json::json!({ "orgId": bad, "inviteeName": "Bob" })),
            ("propose_admission", serde_json::json!({ "orgId": bad, "inviteId": "zz", "handle": null, "confirmed": false })),
            ("propose_revocation", serde_json::json!({ "orgId": bad, "memberId": "zz" })),
            ("cancel_invite", serde_json::json!({ "orgId": bad, "inviteId": "zz" })),
            ("propose_updated_organisation", serde_json::json!({ "orgId": bad })),
            ("write_organisation_to_chain", serde_json::json!({ "orgId": bad, "founder": "zz" })),
            ("verify_organisation", serde_json::json!({ "orgId": bad })),
        ] {
            assert_eq!(invoke_err(&harness, cmd, args), refusal, "{cmd} {bad}");
        }
    }
}

// verifies: LLR-97hujy
#[test]
fn propose_revocation_refuses_a_malformed_org_id_with_the_parsers_message() {
    let harness = harness();
    for bad in malformed_org_ids() {
        let err = invoke_err(&harness, "propose_revocation", serde_json::json!({ "orgId": bad, "memberId": "bb".repeat(32) }));
        assert_eq!(err, parser_refusal(&bad), "{bad}");
    }
}

// verifies: LLR-97hujy
#[test]
fn propose_revocation_does_not_refuse_a_well_formed_org_id() {
    let harness = harness();
    let expected = block_on(org_io_of(&harness).propose_revocation(OrgId::new([0xaa; 20]), MemberId::new([0xbb; 32])))
        .expect_err("no Organisation is held")
        .to_string();
    for org_id in ["aa".repeat(20), format!("0x{}", "aa".repeat(20))] {
        let err = invoke_err(&harness, "propose_revocation", serde_json::json!({ "orgId": org_id, "memberId": "bb".repeat(32) }));
        assert_eq!(err, expected, "{org_id}: org-io's refusal, not the parser's");
    }
}

// verifies: LLR-97hujy
#[test]
fn issue_invite_rejects_a_short_org_id() {
    let harness = harness();
    // 39 characters: one short of the required width.
    let bad = "a".repeat(39);
    let err = invoke_err(&harness, "issue_invite", serde_json::json!({ "orgId": bad, "inviteeName": "Bob" }));
    assert_eq!(err, parser_refusal(&bad), "refused by the real handler under `orgId`");
    assert!(err.contains("40 hex chars"), "refused for its WIDTH, got: {err}");
}

// verifies: LLR-97hujy
#[test]
fn issue_invite_rejects_a_non_hex_org_id() {
    let harness = harness();
    let bad = "zz".repeat(20);
    let err = invoke_err(&harness, "issue_invite", serde_json::json!({ "orgId": bad, "inviteeName": "Bob" }));
    assert_eq!(err, parser_refusal(&bad));
    assert!(err.contains("hex"), "refused for its ALPHABET, got: {err}");
}

// verifies: LLR-97hujy
#[test]
fn issue_invite_does_not_refuse_a_well_formed_org_id() {
    // The fresh store holds no Persona of it, so org-io refuses, after the parser.
    let harness = harness();
    let expected = block_on(org_io_of(&harness).issue_invite(OrgId::new([0xaa; 20]), "Bob"))
        .expect_err("no Organisation is held")
        .to_string();
    for org_id in ["aa".repeat(20), format!("0x{}", "aa".repeat(20))] {
        let err = invoke_err(&harness, "issue_invite", serde_json::json!({ "orgId": org_id, "inviteeName": "Bob" }));
        assert_eq!(err, expected, "{org_id}");
    }
}

// verifies: LLR-97hujy
#[test]
fn propose_admission_refuses_a_malformed_org_id_with_the_parsers_message() {
    let harness = harness();
    for bad in malformed_org_ids() {
        let err = invoke_err(&harness, "propose_admission", admission_args(&bad, &"cc".repeat(32), serde_json::Value::Null, true));
        assert_eq!(err, parser_refusal(&bad), "{bad}");
    }
}

// verifies: LLR-97hujy
#[test]
fn propose_admission_does_not_refuse_a_well_formed_org_id() {
    let harness = harness();
    let expected = block_on(org_io_of(&harness).propose_admission(OrgId::new([0xaa; 20]), InviteId::new([0xcc; 32]), None))
        .expect_err("no Organisation is held")
        .to_string();
    for org_id in ["aa".repeat(20), format!("0x{}", "aa".repeat(20))] {
        let err = invoke_err(&harness, "propose_admission", admission_args(&org_id, &"cc".repeat(32), serde_json::Value::Null, true));
        assert_eq!(err, expected, "{org_id}");
    }
}

// ---------------------------------------------------------------------------
// The contract of the faked Tauri boundary. The rendered webview tests answer, refuse
// and emit through `app/tests/fake-tauri.ts`, which holds every value to
// the shape of an entry of `app/tests/contract/backend.json`. Each entry is
// held here to the shape of what the real command answers or refuses, or
// of the payload the real event mapping emits, under the same rule as
// `shapeMismatch` in `app/tests/contract.ts`.
// ---------------------------------------------------------------------------

const BACKEND: &str = include_str!("../../tests/contract/backend.json");

/// The entries of backend.json: (group, command or event, variant) and the value.
fn backend_entries() -> Vec<((String, String, String), serde_json::Value)> {
    let backend: serde_json::Value = serde_json::from_str(BACKEND).expect("backend.json is JSON");
    let mut entries = Vec::new();
    for group in ["answers", "refusals", "events"] {
        for (name, variants) in backend[group].as_object().expect("a group of names") {
            for (variant, value) in variants.as_object().expect("a name's variants") {
                entries.push(((group.to_string(), name.clone(), variant.clone()), value.clone()));
            }
        }
    }
    entries
}

/// The fixture's value for `group`/`name`/`variant`.
fn backend_entry(group: &str, name: &str, variant: &str) -> serde_json::Value {
    backend_entries()
        .into_iter()
        .find(|((entry_group, entry_name, entry_variant), _)| entry_group == group && entry_name == name && entry_variant == variant)
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("backend.json holds no {group} {name}.{variant}"))
}

const TAG_KEYS: [&str; 5] = ["result", "status", "kind", "transport_mode", "write"];

fn is_hex_id(text: &str) -> bool {
    (text.len() == 40 || text.len() == 64) && text.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Where `actual` departs from the shape of `expected`, or `None`: the rule
/// of backend.json's `about`, the same as `shapeMismatch` in contract.ts.
fn shape_mismatch(expected: &serde_json::Value, actual: &serde_json::Value, path: &str, key: &str) -> Option<String> {
    use serde_json::Value;
    match (expected, actual) {
        (Value::Null, Value::Null) => None,
        (Value::Bool(want), Value::Bool(got)) => (want != got).then(|| format!("{path}: {got} for {want}")),
        (Value::Number(_), Value::Number(_)) => None,
        (Value::String(want), Value::String(got)) => {
            if TAG_KEYS.contains(&key) && want != got {
                Some(format!("{path}: tag {got} for {want}"))
            } else if is_hex_id(want) && !(is_hex_id(got) && got.len() == want.len()) {
                Some(format!("{path}: {got} is not {} hexadecimal characters", want.len()))
            } else {
                None
            }
        }
        (Value::Array(want), Value::Array(got)) => {
            if want.is_empty() && !got.is_empty() {
                return Some(format!("{path}: elements where none are expected"));
            }
            got.iter().enumerate().find_map(|(index, item)| {
                let at = format!("{path}[{index}]");
                (!want.iter().any(|other| shape_mismatch(other, item, &at, key).is_none()))
                    .then(|| format!("{at}: no element of that shape is expected"))
            })
        }
        (Value::Object(want), Value::Object(got)) => {
            let want_keys: std::collections::BTreeSet<&String> = want.keys().collect();
            let got_keys: std::collections::BTreeSet<&String> = got.keys().collect();
            if want_keys != got_keys {
                return Some(format!("{path}: keys {got_keys:?} for {want_keys:?}"));
            }
            want.iter().find_map(|(name, inner)| shape_mismatch(inner, &got[name], &format!("{path}.{name}"), name))
        }
        _ => Some(format!("{path}: {actual} for {expected}")),
    }
}

/// backend.json's `group`/`name`/`variant` has the shape of `real`, what
/// the real command or event gave: the fake answers nothing the backend
/// does not.
fn assert_backend_holds(group: &str, name: &str, variant: &str, real: &serde_json::Value) {
    let fixture = backend_entry(group, name, variant);
    if let Some(mismatch) = shape_mismatch(real, &fixture, "$", "") {
        panic!("backend.json {group} {name}.{variant} is not what the backend gives ({mismatch}):\n fixture {fixture}\n real    {real}");
    }
}

/// Every entry of backend.json, as the tests below drive it: an entry with
/// no real producer here would be a fake answer nothing checks.
const BACKEND_PRODUCED: [(&str, &str, &str); 42] = [
    ("answers", "connection_status", "loopback_no_chain"),
    ("answers", "list_personas", "none"),
    ("answers", "list_personas", "provisional_member"),
    ("answers", "list_personas", "founder"),
    ("answers", "list_orgs", "none"),
    ("answers", "verify_organisation", "none_held"),
    ("answers", "verify_organisation", "in_step"),
    ("answers", "verify_organisation", "founding_with_proxy"),
    ("answers", "verify_organisation", "founding_no_proxy"),
    ("answers", "list_admission_requests", "none_kept"),
    ("answers", "list_admission_requests", "one_kept"),
    ("answers", "propose_person", "provisional"),
    ("answers", "propose_founding_organisation", "provisional_founding"),
    ("answers", "read_invite", "invite"),
    ("answers", "propose_member", "provisional_member"),
    ("answers", "request_admission", "delivered"),
    ("answers", "request_admission", "already_received"),
    ("answers", "request_admission", "invite_used_by_another_device"),
    ("answers", "request_admission", "refused"),
    ("answers", "request_admission", "not_delivered"),
    ("answers", "propose_admission", "recorded"),
    ("answers", "propose_revocation", "recorded"),
    ("answers", "write_organisation_to_chain", "included_founding"),
    ("answers", "write_organisation_to_chain", "refused"),
    ("refusals", "propose_person", "person_already_held"),
    ("refusals", "propose_person", "refused"),
    ("refusals", "*", "text"),
    ("events", "verification-result", "adopted_own_update"),
    ("events", "verification-result", "adopted_with_discarded"),
    ("events", "admission-requested", "kept"),
    ("answers", "list_orgs", "one_not_checked"),
    ("answers", "propose_updated_organisation", "proposed"),
    ("answers", "write_organisation_to_chain", "included_update"),
    ("answers", "cancel_invite", "cancelled"),
    ("answers", "verify_organisation", "adopted_reported"),
    ("events", "verification-result", "adopted_sends_reported"),
    ("events", "verification-result", "adopted_send_refused"),
    ("events", "verification-result", "behind_request_refused"),
    ("answers", "write_organisation_to_chain", "unknown"),
    ("answers", "list_removed_devices", "none"),
    ("answers", "list_removed_devices", "one_unacknowledged"),
    ("answers", "list_removed_devices", "one_acknowledged"),
];

// verifies: LLR-ce4hxz
// The fixture names only commands the surface registers and only entries
// the tests below drive.
#[test]
fn every_backend_entry_has_a_real_producer_here() {
    let mut fixture: Vec<(String, String, String)> = backend_entries().into_iter().map(|(at, _)| at).collect();
    fixture.sort();
    let mut produced: Vec<(String, String, String)> = BACKEND_PRODUCED
        .iter()
        .map(|(group, name, variant)| (group.to_string(), name.to_string(), variant.to_string()))
        .collect();
    produced.sort();
    assert_eq!(fixture, produced, "backend.json and the producers here name the same entries");
    let registered = handler_list(&read_source("src/lib.rs"));
    for (group, name, _) in &fixture {
        if group != "events" && name != "*" {
            assert!(registered.contains(&format!("commands::{name}")), "backend.json answers {name}, which the surface does not register");
        }
    }
}

// verifies: LLR-8tzbzn, LLR-jf63e4, LLR-y7nc6x, LLR-k57uqv, LLR-6ff735, LLR-pguhw5
// A fresh store's answers, the Person's and the founding's, and the refusals.
#[test]
fn a_fresh_stores_answers_and_refusals_have_the_fakes_shapes() {
    let loopback = harness_with(TransportModeName::Loopback, None);
    let status = invoke(&loopback, "connection_status", serde_json::json!({})).expect("connection_status");
    assert_backend_holds("answers", "connection_status", "loopback_no_chain", &status);
    for (command, variant) in [
        ("list_personas", "none"),
        ("list_orgs", "none"),
        ("verify_organisation", "none_held"),
        ("list_admission_requests", "none_kept"),
    ] {
        let answer = invoke(&loopback, command, serde_json::json!({})).expect(command);
        assert_backend_holds("answers", command, variant, &answer);
    }
    let field = invoke(&loopback, "propose_person", serde_json::json!({ "name": "a".repeat(129), "surname": "Smith" }))
        .expect_err("an over-long name is refused");
    assert_backend_holds("refusals", "propose_person", "refused", &field);
    let text = invoke(&loopback, "issue_invite", serde_json::json!({ "orgId": "zz", "inviteeName": "Bob" }))
        .expect_err("a malformed org id is refused");
    assert_backend_holds("refusals", "*", "text", &text);

    let harness = harness();
    let person = invoke(&harness, "propose_person", serde_json::json!({ "name": "Alice", "surname": "Smith" })).expect("propose_person");
    assert_backend_holds("answers", "propose_person", "provisional", &person);
    let held = invoke(&harness, "propose_person", serde_json::json!({ "name": "Alice", "surname": "Smith" }))
        .expect_err("a second Person is refused");
    assert_backend_holds("refusals", "propose_person", "person_already_held", &held);
    let founding = invoke(
        &harness,
        "propose_founding_organisation",
        serde_json::json!({ "name": "Alice", "surname": "Smith", "handle": "alice", "orgName": "Acme" }),
    )
    .expect("propose_founding_organisation");
    assert_backend_holds("answers", "propose_founding_organisation", "provisional_founding", &founding);
    // The founder is listed with the Handle the founding gave.
    let listed = invoke(&harness, "list_personas", serde_json::json!({})).expect("list_personas");
    assert_backend_holds("answers", "list_personas", "founder", &listed);
    // No chain: the founding, with no proxy, is unverified, named by its founder.
    let verified = invoke(&harness, "verify_organisation", serde_json::json!({})).expect("verify_organisation");
    assert_backend_holds("answers", "verify_organisation", "founding_no_proxy", &verified);
}

// verifies: LLR-jf63e4, LLR-rhc555, LLR-k4rhbs
// A founding written over the substitute chain, then verified; a refused write.
#[test]
fn the_writes_and_verifications_have_the_fakes_shapes() {
    let harness = harness_over_chain();
    let persona_id = propose_founding(&harness, "alice", "Acme");
    let founder = hex::encode(device_key_of(&harness, &persona_id).as_bytes());
    let written = invoke(&harness, "write_organisation_to_chain", serde_json::json!({ "founder": founder }))
        .expect("write_organisation_to_chain");
    assert_backend_holds("answers", "write_organisation_to_chain", "included_founding", &written);
    let verified = invoke(&harness, "verify_organisation", serde_json::json!({})).expect("verify_organisation");
    assert_backend_holds("answers", "verify_organisation", "in_step", &verified);

    let chain = FakeChain::new();
    chain.found_at_failing.store(true, std::sync::atomic::Ordering::SeqCst);
    let refusing = harness_over(&chain);
    let persona_id = propose_founding(&refusing, "alice", "Acme");
    let founder = hex::encode(device_key_of(&refusing, &persona_id).as_bytes());
    let refused = invoke(&refusing, "write_organisation_to_chain", serde_json::json!({ "founder": founder }))
        .expect("write_organisation_to_chain");
    assert_backend_holds("answers", "write_organisation_to_chain", "refused", &refused);
    // The refused write recorded the proxy: the founding is entered by its Organisation id and its founder.
    let verified = invoke(&refusing, "verify_organisation", serde_json::json!({})).expect("verify_organisation");
    assert_backend_holds("answers", "verify_organisation", "founding_with_proxy", &verified);
}

// verifies: LLR-tbtrb3, LLR-k4rhbs
// A write whose outcome is unknown: the chain never answers, so org-io's
// bound passes (on a paused clock) and the write may yet execute. The real
// command is called directly, not over IPC, because the IPC path runs on
// Tauri's own runtime, whose clock cannot be paused; its answer is
// serialised as IPC serialises it.
#[tokio::test(start_paused = true)]
async fn an_unknown_write_has_the_fakes_shape() {
    use std::sync::atomic::Ordering;
    let chain = FakeChain::new();
    let harness = harness_over(&chain);
    let io = org_io_of(&harness);
    let founder = io.propose_founding_organisation("Alice", "Smith", "alice", "Acme").await.expect("proposed").founder;
    // The proxy is stored by a write whose genesis is refused, as org-io's
    // founding-unknown tests do; then every write hangs.
    chain.found_at_failing.store(true, Ordering::SeqCst);
    let first = io.write_organisation_to_chain(HeldTarget::Founding(founder)).await;
    assert!(matches!(first, Ok(WriteReport::Refused { .. })), "the proxy stored, the genesis not: {first:?}");
    chain.found_at_failing.store(false, Ordering::SeqCst);
    chain.write_hanging.store(true, Ordering::SeqCst);
    let written = commands::write_organisation_to_chain(harness._app.state(), None, Some(hex::encode(founder.as_bytes())))
        .await
        .expect("write_organisation_to_chain");
    let unknown = serde_json::to_value(&written).expect("serialises");
    assert_eq!(unknown["result"], "unknown", "{unknown}");
    assert_backend_holds("answers", "write_organisation_to_chain", "unknown", &unknown);
}

// verifies: LLR-pguhw5, LLR-b7ywy4, LLR-rg9vc7, LLR-8cz828, LLR-y7nc6x, LLR-cr9nmw, LLR-7n4hgr, LLR-dp45mm, LLR-8grnbb
// The invitee's and the Admin's answers over the real exchange, and the two
// events the receiver loop and a verification emit, captured as the webview
// would receive them.
#[test]
fn the_exchanges_answers_and_events_have_the_fakes_shapes() {
    use tauri::Listener;
    let ex = exchange(&["bob", "builder"]);
    let (sender, arrived) = std::sync::mpsc::channel::<String>();
    ex.inviter._app.listen_any("admission-requested", move |event| {
        // A send fails only once the test has dropped `arrived`, after its
        // last read: an event after that is not this test's to check.
        let _ = sender.send(event.payload().to_string());
    });
    let invite = invoke(&ex.invitee, "read_invite", serde_json::json!({ "blob": ex.blob })).expect("read_invite");
    assert_backend_holds("answers", "read_invite", "invite", &invite);
    assert_backend_holds("answers", "propose_member", "provisional_member", &ex.proposed);
    // The provisional Member is listed with no Handle yet: `handle` null.
    let listed = invoke(&ex.invitee, "list_personas", serde_json::json!({})).expect("list_personas");
    assert_backend_holds("answers", "list_personas", "provisional_member", &listed);

    let member = ex.proposed["persona_id"].clone();
    let ask = |harness: &Harness, persona: &serde_json::Value| {
        invoke(harness, "request_admission", serde_json::json!({ "personaId": persona, "confirmed": true })).expect("request_admission")
    };
    assert_backend_holds("answers", "request_admission", "delivered", &ask(&ex.invitee, &member));
    let event = arrived.recv_timeout(std::time::Duration::from_secs(5)).expect("admission-requested emitted");
    let event: serde_json::Value = serde_json::from_str(&event).expect("a JSON payload");
    assert_backend_holds("events", "admission-requested", "kept", &event);
    assert_backend_holds("answers", "request_admission", "already_received", &ask(&ex.invitee, &member));
    let listed = invoke(&ex.inviter, "list_admission_requests", serde_json::json!({})).expect("list_admission_requests");
    assert_backend_holds("answers", "list_admission_requests", "one_kept", &listed);

    let other = harness_with(TransportModeName::Loopback, None);
    let other_member = invoke(
        &other,
        "propose_member",
        serde_json::json!({ "inviteBlob": ex.blob, "name": "Eve", "surname": "Other", "preferredHandles": ["eve"] }),
    )
    .expect("propose_member")["persona_id"]
        .clone();
    assert_backend_holds("answers", "request_admission", "invite_used_by_another_device", &ask(&other, &other_member));

    // The Admin admits Bob; the own adoption of that epoch is announced with its sends.
    let invite_id = hex::encode(kept_requests(&ex.inviter, 1)[0].invite_id.as_bytes());
    let recorded = invoke(&ex.inviter, "propose_admission", admission_args(&ex.org_id, &invite_id, serde_json::Value::Null, true))
        .expect("propose_admission");
    assert_backend_holds("answers", "propose_admission", "recorded", &recorded);
    let io = org_io_of(&ex.inviter);
    let org = ods_poc_lib::parsing::parse_org_id(&ex.org_id).expect("an org id");
    block_on(io.propose_updated_organisation(org)).expect("the next epoch is proposed");
    let written = block_on(io.write_organisation_to_chain(HeldTarget::Organisation(org)));
    assert!(matches!(written, Ok(WriteReport::Included { .. })), "{written:?}");
    let announced: Vec<ods_poc_lib::events::Emission> =
        block_on(commands::verification_outcomes(&io, Some(HeldTarget::Organisation(org))))
            .iter()
            .flat_map(ods_poc_lib::events::emissions_for)
            .collect();
    let result = announced.iter().find(|emission| emission.name == "verification-result").expect("a verification-result");
    assert_backend_holds("events", "verification-result", "adopted_own_update", &result.payload);

    // Bob, now a Member, asks again: refused by the receiving Device.
    assert_backend_holds("answers", "request_admission", "refused", &ask(&ex.invitee, &member));

    let bob = block_on(io.organisation(org))
        .expect("held")
        .members
        .iter()
        .find(|listed| listed.handle.to_string() == "bob")
        .expect("Bob is a Member")
        .id;
    let revoked = invoke(
        &ex.inviter,
        "propose_revocation",
        serde_json::json!({ "orgId": ex.org_id, "memberId": hex::encode(bob.as_bytes()) }),
    )
    .expect("propose_revocation");
    assert_backend_holds("answers", "propose_revocation", "recorded", &revoked);

    // The inviter's endpoint closed: not delivered.
    let inviter_persona = block_on(io.personas())[0].persona_id.clone();
    block_on(async {
        io.endpoint_for_test(&inviter_persona).await.expect("bound").inner().close().await;
    });
    assert_backend_holds("answers", "request_admission", "not_delivered", &ask(&other, &other_member));
}

// verifies: LLR-8grnbb, LLR-n64wvr
// An adoption another Admin built that left out an admission and a
// revocation (org-io's `Verified`, which the substitute chain does not
// produce for this node), through the app's real entry and event mapping.
#[test]
fn an_adoption_with_discarded_changes_is_announced_in_the_fakes_shape() {
    use org_io::node::{Epoch, Handle, Name, Surname};
    use org_io::verify::DiscardedChangeSummary;
    let org = OrgId::new([0xab; 20]);
    let verified = Verified::Adopted {
        epoch: Epoch::new(3),
        own: false,
        discarded: vec![
            DiscardedChangeSummary::Admission {
                invite_id: InviteId::new([0x1c; 32]),
                name: Some(Name::parse("Bob").unwrap()),
                surname: Some(Surname::parse("Builder").unwrap()),
                handle: Handle::parse("bob").unwrap(),
            },
            DiscardedChangeSummary::Revocation { member_id: MemberId::new([0x2d; 32]), handle: Some(Handle::parse("carol").unwrap()) },
        ],
    };
    let entry = ods_poc_lib::entry::entry_for(&ods_poc_lib::entry::EntryOrg::Id(org), &verified);
    let announced: Vec<ods_poc_lib::events::Emission> = ods_poc_lib::events::outcomes_for_verification(&entry, None)
        .iter()
        .flat_map(ods_poc_lib::events::emissions_for)
        .collect();
    let result = announced.iter().find(|emission| emission.name == "verification-result").expect("a verification-result");
    assert_backend_holds("events", "verification-result", "adopted_with_discarded", &result.payload);
}

// verifies: LLR-2u3cga, LLR-vqz4s6, LLR-rhc555, LLR-k4rhbs, LLR-g8wfy9
// The Admin's epoch commands over the real exchange: a held Organisation
// listed, the next epoch proposed from a recorded admission and written by
// its Organisation identifier, and a kept request's invite cancelled.
#[test]
fn the_epoch_commands_answers_have_the_fakes_shapes() {
    let (harness, _) = seeded_harness();
    let orgs = invoke(&harness, "list_orgs", serde_json::json!({})).expect("list_orgs");
    assert_backend_holds("answers", "list_orgs", "one_not_checked", &orgs);

    let chain = FakeChain::new();
    let (ex, invite_id) = requested_over(&chain, &["bob"]);
    invoke(&ex.inviter, "propose_admission", admission_args(&ex.org_id, &invite_id, serde_json::Value::Null, true))
        .expect("propose_admission");
    let proposed = invoke(&ex.inviter, "propose_updated_organisation", serde_json::json!({ "orgId": ex.org_id }))
        .expect("propose_updated_organisation");
    assert_backend_holds("answers", "propose_updated_organisation", "proposed", &proposed);
    let written = invoke(&ex.inviter, "write_organisation_to_chain", serde_json::json!({ "orgId": ex.org_id }))
        .expect("write_organisation_to_chain");
    assert_backend_holds("answers", "write_organisation_to_chain", "included_update", &written);

    let (other, other_invite) = requested(&["carol"]);
    let cancelled = invoke(&other.inviter, "cancel_invite", serde_json::json!({ "orgId": other.org_id, "inviteId": other_invite }))
        .expect("cancel_invite");
    assert_backend_holds("answers", "cancel_invite", "cancelled", &cancelled);
}

/// The ed25519 base point's canonical encoding: a valid Device public key.
const BASE_POINT: [u8; 32] = [
    0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
];

// verifies: LLR-8grnbb, LLR-k4rxyt, LLR-rywe8f, LLR-jf63e4
// The outcomes of a send and a request as a verification reports them,
// each Device's or the refusal of the whole, through the app's real entry
// mapping (`entry_for`, `attach_report`): as a `verification-result` event
// and as `verify_organisation`'s answer, which is these entries serialised.
// The substitute chain produces neither a discarded change nor a refusal
// of a whole send for this node (verification.rs, LLR-k4rxyt).
#[test]
fn the_send_and_request_reports_have_the_fakes_shapes() {
    use ods_poc_lib::entry::{attach_report, entry_for, EntryOrg, VerificationEntryDto};
    use ods_poc_lib::follow_up::VerificationFollowUp;
    use org_io::node::{Epoch, Handle, Name, Surname};
    use org_io::verify::DiscardedChangeSummary;
    use org_io::SendReport;
    let org = OrgId::new([0xab; 20]);
    let named = EntryOrg::Id(org);
    let device = DevicePublicKey::parse(&BASE_POINT).unwrap();
    let report = || SendReport { per_device: vec![(device, Delivery::Delivered), (device, Delivery::Refused)] };
    let event = |entry: &VerificationEntryDto| {
        ods_poc_lib::events::outcomes_for_verification(entry, None)
            .iter()
            .flat_map(ods_poc_lib::events::emissions_for)
            .find(|emission| emission.name == "verification-result")
            .expect("a verification-result")
            .payload
    };
    let own = || entry_for(&named, &Verified::Adopted { epoch: Epoch::new(3), discarded: vec![], own: true });

    let mut reported = own();
    attach_report(&mut reported, VerificationFollowUp::Send(org), Ok(report()));
    assert_backend_holds("events", "verification-result", "adopted_sends_reported", &event(&reported));

    let mut refused = own();
    attach_report(&mut refused, VerificationFollowUp::Send(org), Err("Organisation abab is not held here".to_string()));
    assert_backend_holds("events", "verification-result", "adopted_send_refused", &event(&refused));

    let mut behind = entry_for(&named, &Verified::Behind { chain_epoch: Epoch::new(4) });
    attach_report(&mut behind, VerificationFollowUp::Request(org), Err("no endpoint is bound".to_string()));
    assert_backend_holds("events", "verification-result", "behind_request_refused", &event(&behind));

    let discarded = entry_for(
        &named,
        &Verified::Adopted {
            epoch: Epoch::new(3),
            own: false,
            discarded: vec![
                DiscardedChangeSummary::Admission {
                    invite_id: InviteId::new([0x1c; 32]),
                    name: Some(Name::parse("Bob").unwrap()),
                    surname: Some(Surname::parse("Builder").unwrap()),
                    handle: Handle::parse("bob").unwrap(),
                },
                DiscardedChangeSummary::Revocation { member_id: MemberId::new([0x2d; 32]), handle: Some(Handle::parse("carol").unwrap()) },
            ],
        },
    );
    let answer = serde_json::to_value(vec![discarded, reported]).expect("the entries serialise");
    assert_backend_holds("answers", "verify_organisation", "adopted_reported", &answer);
}
