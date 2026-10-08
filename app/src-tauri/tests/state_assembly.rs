//! Assembling the application state: where the persona store is opened, and
//! what is refused when it cannot be.
//!
//! Low-level requirement carried here: LLR-85zque (SDD-aq7m6b). Added
//! 2026-10-05 by the app architecture change.
//!
//! Every test builds its state through `AppState::for_test`, which takes each
//! input as a parameter and shares `assemble` with `AppState::init`. No test
//! here sets an environment variable.

use std::path::Path;

use ods_poc_lib::policy::TransportModeName;
use ods_poc_lib::state::AppState;
use org_io::node::store::PersonaDetails;
use rand::rngs::OsRng;

fn state_at(data_dir: &Path, passphrase: &str) -> Result<AppState, String> {
    AppState::for_test(
        data_dir.to_path_buf(),
        passphrase,
        None,
        TransportModeName::Networked,
    )
}

/// Create one persona, which saves the store.
fn add_persona(state: &AppState) {
    let PersonaDetails { handle, name, surname } =
        PersonaDetails::parse("alice", "Alice", "Smith").expect("valid details");
    state
        .org_io
        .blocking_lock()
        .node_mut()
        .create_persona(&mut OsRng, handle, name, surname)
        .expect("create_persona");
}

/// The error `for_test` returned. `AppState` is not `Debug`, so `expect_err`
/// is not available.
fn refusal(result: Result<AppState, String>) -> String {
    match result {
        Ok(_) => panic!("the state was assembled, and it should have been refused"),
        Err(e) => e,
    }
}

// verifies: LLR-85zque
#[test]
fn the_store_is_opened_inside_the_data_dir_it_creates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let data_dir = tmp.path().join("a").join("b");
    assert!(!data_dir.exists());

    let state = state_at(&data_dir, "first passphrase").expect("assembled");
    assert!(data_dir.is_dir(), "the data directory and its parents are created");
    add_persona(&state);
    assert!(
        data_dir.join("persona_store.bin").is_file(),
        "the store is the file persona_store.bin in the data directory"
    );
    drop(state);

    // Reopened at the same place under the same passphrase, it is the same store.
    let again = state_at(&data_dir, "first passphrase").expect("reopened");
    assert_eq!(again.org_io.blocking_lock().view().personas().len(), 1);
}

// verifies: LLR-85zque
#[test]
fn a_data_dir_that_cannot_be_created_is_refused_naming_it() {
    // Abnormal input: the data directory would sit under a regular file.
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("plain-file");
    std::fs::write(&file, b"not a directory").expect("write");
    let data_dir = file.join("sub");

    let err = refusal(state_at(&data_dir, "any passphrase"));
    assert!(
        err.starts_with(&format!("create data_dir {}", data_dir.display())),
        "the refusal names the directory: {err}"
    );
}

// `init` reads the environment and connects on Tauri's runtime, so it is not
// run here; its source is read instead (2026-10-08, task T9b). It opens the
// store once: a refused `OrgIo::connect` hands the service back, and the
// unconfigured handle is built from it, never from a second open.
// verifies: LLR-85zque
#[test]
fn init_opens_the_store_once_even_when_the_connect_is_refused() {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/state.rs"),
    )
    .expect("read state.rs");
    let start = source.find("pub fn init(").expect("init is defined");
    let end = start + source[start..].find("fn assemble(").expect("assemble follows init");
    let code: String = source[start..end]
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    // Since review round 2 (finding 2) the store is opened by `OrgIo::open`.
    let opens = code.matches("open_service(").count()
        + code.matches("PersonaStore::open(").count()
        + code.matches("OrgIo::open(").count();
    assert_eq!(opens, 1, "init opens the store {opens} times");
}

// verifies: LLR-85zque
#[test]
fn a_store_under_another_passphrase_is_refused() {
    // Abnormal input: an existing store, opened with the wrong passphrase. It
    // is refused, not replaced by an empty store.
    let tmp = tempfile::tempdir().expect("tempdir");
    let state = state_at(tmp.path(), "the right passphrase").expect("assembled");
    add_persona(&state);
    drop(state);

    let err = refusal(state_at(tmp.path(), "the wrong passphrase"));
    assert!(err.starts_with("open store: "), "{err}");
    assert!(
        tmp.path().join("persona_store.bin").is_file(),
        "the refused store is left where it was"
    );
}
