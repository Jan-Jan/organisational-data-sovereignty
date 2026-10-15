//! Startup policy: no passphrase and the keychain refusal, data-directory
//! resolution, and the wording of the refusals.
//!
//! Requirements carried here: REQ-7g3k9a (as amended 2026-10-14: no
//! passphrase; a keychain refusal fails startup), REQ-rxc8sp (data
//! directory), REQ-bmk2z2 (the refusal names both variables, supplier first).
//!
//! No test in this file sets an environment variable. The policy functions take
//! their inputs as parameters precisely so they need not: cargo runs
//! integration tests in threads of one process, and a test that mutated the
//! environment would race every other test in the binary.

use std::path::PathBuf;

use ods_poc_lib::policy::{self, StartupError};

// *Deleted 2026-10-14 (S4, task T17):* `configured_passphrase_is_used`,
// `absent_passphrase_is_refused`, `empty_passphrase_is_refused_not_honoured`,
// `absent_passphrase_with_opt_in_uses_dev_default` and
// `configured_passphrase_wins_over_opt_in`, with `resolve_passphrase`, the
// behaviour they tested: the app reads no passphrase (LLR-tpkmh3 as amended
// 2026-10-14; its two tests follow).

// ---------------------------------------------------------------------------
// REQ-7g3k9a as amended 2026-10-14 (S4, task T17) — no passphrase; a
// keychain refusal fails startup, naming the condition
// ---------------------------------------------------------------------------

/// The unit's sources under `src-tauri/src`, as (path, text), comments
/// included: a passphrase variable named even in a comment would be a
/// reader's cue that one is still read.
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
    assert!(!sources.is_empty(), "no sources found");
    sources
}

// verifies: LLR-tpkmh3
#[test]
fn startup_reads_no_passphrase() {
    let naming: Vec<_> = app_sources()
        .into_iter()
        .filter(|(_, text)| text.contains("ODS_PASSPHRASE"))
        .map(|(path, _)| path)
        .collect();
    assert!(naming.is_empty(), "the app still names ODS_PASSPHRASE in {naming:?}");
}

// Abnormal case: the keychain refuses at startup. A Persona is created over
// org-io's in-memory keychain, which is then set to refuse as a locked
// keychain does; building the state again must fail, and the failure must
// name the Persona's store and the condition. Only this test in this binary
// touches the process-wide keychain.
// verifies: LLR-tpkmh3, REQ-7g3k9a
#[test]
fn a_keychain_refusal_fails_startup_naming_the_condition() {
    use org_io::node::{Handle, Name, Surname};
    use org_io::test_support::{MemoryKeychain, Refusal};
    use ods_poc_lib::policy::TransportModeName;
    use ods_poc_lib::state::AppState;

    let dir = tempfile::tempdir().expect("tempdir");
    let data_dir = dir.path().to_path_buf();
    let state = AppState::for_test(data_dir.clone(), None, TransportModeName::Loopback).expect("first start");
    tokio::runtime::Runtime::new()
        .expect("runtime")
        .block_on(state.org_io.create_persona(
            &mut rand::rngs::OsRng,
            Handle::parse("alice").unwrap(),
            Name::parse("Test").unwrap(),
            Surname::parse("User").unwrap(),
        ))
        .expect("persona");
    drop(state);
    let device_keys = org_io::store_file::list_stores(&data_dir).expect("list store files");
    assert_eq!(device_keys.len(), 1, "one Persona, one store file: {device_keys:?}");
    let store_files: Vec<String> = device_keys
        .iter()
        .map(|device_key| org_io::store_file::store_path(&data_dir, device_key))
        .map(|path| path.file_name().expect("store file name").to_string_lossy().into_owned())
        .collect();

    MemoryKeychain::shared().refuse_as(Some(Refusal::Locked));
    let refused = AppState::for_test(data_dir, None, TransportModeName::Loopback);
    MemoryKeychain::shared().refuse_as(None);

    let message = match refused {
        Ok(_) => panic!("started although the keychain refused"),
        Err(message) => message,
    };
    assert!(message.contains(&store_files[0]), "the refusal names the Persona's store: {message}");
    assert!(message.contains("locked"), "the refusal names the condition: {message}");
}

// ---------------------------------------------------------------------------
// REQ-rxc8sp — the data directory
// ---------------------------------------------------------------------------

// verifies: LLR-kfmng5
#[test]
fn override_data_dir_is_used() {
    assert_eq!(
        policy::resolve_data_dir(
            Some("/var/lib/ods"),
            Some(PathBuf::from("/platform/resolved")),
            false
        ),
        Ok(PathBuf::from("/var/lib/ods"))
    );
}

// verifies: LLR-kfmng5
#[test]
fn resolved_data_dir_is_used_when_no_override() {
    assert_eq!(
        policy::resolve_data_dir(None, Some(PathBuf::from("/platform/resolved")), false),
        Ok(PathBuf::from("/platform/resolved"))
    );
}

// verifies: LLR-kfmng5
#[test]
fn unresolvable_data_dir_is_refused() {
    assert_eq!(
        policy::resolve_data_dir(None, None, false),
        Err(StartupError::UnresolvableDataDir)
    );
}

// verifies: LLR-kfmng5
#[test]
fn unresolvable_data_dir_with_opt_in_uses_temp() {
    assert_eq!(
        policy::resolve_data_dir(None, None, true),
        Ok(PathBuf::from(policy::DEV_DATA_DIR))
    );
}

// verifies: LLR-kfmng5
#[test]
fn empty_override_is_ignored_not_used_as_path() {
    // An empty ODS_DATA_DIR must not become the relative path "", which would
    // put the persona store in the process's working directory.
    assert_eq!(
        policy::resolve_data_dir(Some(""), Some(PathBuf::from("/platform/resolved")), false),
        Ok(PathBuf::from("/platform/resolved"))
    );
    assert_eq!(
        policy::resolve_data_dir(Some(""), None, false),
        Err(StartupError::UnresolvableDataDir)
    );
}

// ---------------------------------------------------------------------------
// REQ-bmk2z2 — the refusals
// ---------------------------------------------------------------------------

/// Assert that `msg` names `supplier`, the variable that would supply the
/// value, before `ODS_ALLOW_DEV_DEFAULTS`, the variable that would waive it.
///
/// Order matters: an operator who reads the waiver first sets the waiver,
/// which recreates HAZ-8ghmhn. REQ-bmk2z2 is satisfied by naming both; this
/// assertion is the register's reasoning made executable.
fn assert_names_supplier_before_waiver(msg: &str, supplier: &str) {
    let supplier_at = msg.find(supplier).expect("names the supplying variable");
    let waiver_at = msg.find("ODS_ALLOW_DEV_DEFAULTS").expect("names the waiver");
    assert!(supplier_at < waiver_at, "the supplying variable must be named first: {msg}");
}

// *Deleted 2026-10-14 (S4, task T17):* `passphrase_refusal_names_both_variables_supplier_first`,
// with the passphrase refusal it read (LLR-97rww8, narrowed the same day to
// the data directory).

// verifies: LLR-97rww8
#[test]
fn data_dir_refusal_names_both_variables_supplier_first() {
    let msg = policy::resolve_data_dir(None, None, false).unwrap_err().to_string();
    assert_names_supplier_before_waiver(&msg, "ODS_DATA_DIR");
}

// verifies: LLR-97rww8
#[test]
fn refusals_for_empty_values_name_both_variables_supplier_first() {
    // Abnormal input: the variable is set, to the empty string. The value is
    // treated as absent, and the operator must be told the same thing, in the
    // same order, as when it is unset. *2026-10-14 (S4, task T17):* the
    // passphrase half left with the passphrase; the data directory remains.
    let msg = policy::resolve_data_dir(Some(""), None, false).unwrap_err().to_string();
    assert_names_supplier_before_waiver(&msg, "ODS_DATA_DIR");
}
