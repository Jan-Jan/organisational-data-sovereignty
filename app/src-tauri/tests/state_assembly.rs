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
use org_node::store::PersonaDetails;
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
        .service
        .blocking_lock()
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
    assert_eq!(again.service.blocking_lock().list_personas().len(), 1);
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
