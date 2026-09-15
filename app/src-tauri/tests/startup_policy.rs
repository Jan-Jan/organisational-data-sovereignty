//! Startup policy: passphrase resolution, data-directory resolution, and the
//! wording of the refusals.
//!
//! Requirements carried here: REQ-7g3k9a (passphrase), REQ-rxc8sp (data
//! directory), REQ-bmk2z2 (the refusal names both variables, supplier first).
//!
//! No test in this file sets an environment variable. The policy functions take
//! their inputs as parameters precisely so they need not: cargo runs
//! integration tests in threads of one process, and a test that mutated the
//! environment would race every other test in the binary.

use std::path::PathBuf;

use ods_poc_lib::policy::{self, StartupError};

// ---------------------------------------------------------------------------
// REQ-7g3k9a — the passphrase
// ---------------------------------------------------------------------------

// verifies: REQ-7g3k9a
#[test]
fn configured_passphrase_is_used() {
    assert_eq!(
        policy::resolve_passphrase(Some("correct horse battery staple"), false),
        Ok("correct horse battery staple".to_string())
    );
}

// verifies: REQ-7g3k9a
#[test]
fn absent_passphrase_is_refused() {
    assert_eq!(
        policy::resolve_passphrase(None, false),
        Err(StartupError::MissingPassphrase)
    );
}

// verifies: REQ-7g3k9a
#[test]
fn empty_passphrase_is_refused_not_honoured() {
    // A variable set to "" is a scripting accident, not a passphrase. Honouring
    // it would be the original hazard with an extra step.
    assert_eq!(
        policy::resolve_passphrase(Some(""), false),
        Err(StartupError::MissingPassphrase)
    );
}

// verifies: REQ-7g3k9a
#[test]
fn absent_passphrase_with_opt_in_uses_dev_default() {
    assert_eq!(
        policy::resolve_passphrase(None, true),
        Ok(policy::DEV_PASSPHRASE.to_string())
    );
}

// verifies: REQ-7g3k9a
#[test]
fn configured_passphrase_wins_over_opt_in() {
    let got = policy::resolve_passphrase(Some("operator supplied"), true)
        .expect("a configured passphrase is always usable");
    assert_eq!(got, "operator supplied");
    assert_ne!(got, policy::DEV_PASSPHRASE);
}

// ---------------------------------------------------------------------------
// REQ-rxc8sp — the data directory
// ---------------------------------------------------------------------------

// verifies: REQ-rxc8sp
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

// verifies: REQ-rxc8sp
#[test]
fn resolved_data_dir_is_used_when_no_override() {
    assert_eq!(
        policy::resolve_data_dir(None, Some(PathBuf::from("/platform/resolved")), false),
        Ok(PathBuf::from("/platform/resolved"))
    );
}

// verifies: REQ-rxc8sp
#[test]
fn unresolvable_data_dir_is_refused() {
    assert_eq!(
        policy::resolve_data_dir(None, None, false),
        Err(StartupError::UnresolvableDataDir)
    );
}

// verifies: REQ-rxc8sp
#[test]
fn unresolvable_data_dir_with_opt_in_uses_temp() {
    assert_eq!(
        policy::resolve_data_dir(None, None, true),
        Ok(PathBuf::from(policy::DEV_DATA_DIR))
    );
}

// verifies: REQ-rxc8sp
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

// verifies: REQ-bmk2z2
#[test]
fn passphrase_refusal_names_both_variables_supplier_first() {
    let msg = policy::resolve_passphrase(None, false).unwrap_err().to_string();
    let supplier = msg.find("ODS_PASSPHRASE").expect("names the supplying variable");
    let waiver = msg.find("ODS_ALLOW_DEV_DEFAULTS").expect("names the waiver");
    // Order matters: an operator who reads the waiver first sets the waiver,
    // which recreates HAZ-8ghmhn. REQ-bmk2z2 is satisfied by naming both; this
    // assertion is the register's reasoning made executable.
    assert!(supplier < waiver, "the supplying variable must be named first: {msg}");
}

// verifies: REQ-bmk2z2
#[test]
fn data_dir_refusal_names_both_variables_supplier_first() {
    let msg = policy::resolve_data_dir(None, None, false).unwrap_err().to_string();
    let supplier = msg.find("ODS_DATA_DIR").expect("names the supplying variable");
    let waiver = msg.find("ODS_ALLOW_DEV_DEFAULTS").expect("names the waiver");
    assert!(supplier < waiver, "the supplying variable must be named first: {msg}");
}
