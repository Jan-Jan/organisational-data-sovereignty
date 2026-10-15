#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::result_large_err, reason = "the handle's `CreateError`, which names a Device public key")]
//! The transport preflight by the keychain path (LLR-h9tcat; S4, task T10 of
//! docs/plans/2026-10-06-org-io-transport.md, owner rulings 7 and 12): the
//! check binds each Device key's endpoint through its keychain item, and the
//! preflight finds the Device keys by listing the data directory's store
//! files, as the node does. No key or seed comes from the environment.
//!
//! Over org-io's in-memory keychain, one test at a time under `LOCK`:
//! keyring's default credential builder is process-global.

use std::path::PathBuf;
use std::time::Duration;

use org_node::{DevicePublicKey, Handle, Name, Surname};
use org_io::preflight::{check_transport, check_transport_for_data_dir, CheckResult};
use org_io::test_support::{MemoryKeychain, Refusal};
use org_io::transport::TransportMode;
use org_io::OrgIo;
use rand::rngs::OsRng;

mod common;
use common::handles::keep_device_seed;

static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Loopback needs no wait; the timeout is for Networked only.
const ONLINE_TIMEOUT: Duration = Duration::from_secs(1);

/// A fresh, not yet created directory for test `tag`.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("org-io-preflight-transport-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn hex_of(device_key: &DevicePublicKey) -> String {
    hex::encode(device_key.as_bytes())
}

/// Every row failed, and each one names `needle`.
fn assert_one_failure_naming(rows: &[CheckResult], needle: &str) {
    assert_eq!(rows.len(), 1, "one row: {rows:?}");
    assert!(!rows[0].ok, "the row fails: {rows:?}");
    assert!(rows[0].detail.contains(needle), "the row names {needle}: {rows:?}");
}

// verifies: LLR-h9tcat
#[tokio::test(flavor = "multi_thread")]
async fn the_transport_check_passes_for_a_device_key_whose_item_exists() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::default().install();
    let device_key = keep_device_seed([0x81; 32]);
    let row = check_transport(&device_key, TransportMode::Loopback, ONLINE_TIMEOUT).await;
    assert!(row.ok, "{row:?}");
    assert_eq!(row.name, "transport.bind");
    assert!(row.detail.contains(&hex_of(&device_key)), "the row names the Device key: {row:?}");
}

// Abnormal: no item, or a locked keychain, is one FAIL row naming the
// Device key; the check binds nothing, since the bind needs the item's seed.
// verifies: LLR-h9tcat
#[tokio::test(flavor = "multi_thread")]
async fn a_missing_or_locked_item_fails_the_check_naming_the_key_and_binds_nothing() {
    let _guard = LOCK.lock().await;
    let keychain = MemoryKeychain::default();
    keychain.install();
    let missing = org_node::test_fixtures::device_key(0x82);
    let row = check_transport(&missing, TransportMode::Loopback, ONLINE_TIMEOUT).await;
    assert_one_failure_naming(std::slice::from_ref(&row), &hex_of(&missing));
    assert!(row.detail.contains("no keychain item"), "{row:?}");

    let locked = keep_device_seed([0x83; 32]);
    keychain.refuse_as(Some(Refusal::Locked));
    let row = check_transport(&locked, TransportMode::Loopback, ONLINE_TIMEOUT).await;
    assert_one_failure_naming(std::slice::from_ref(&row), &hex_of(&locked));
    assert!(row.detail.contains("locked"), "{row:?}");
    keychain.refuse_as(None);
}

/// Create a Persona through a handle over `dir`, as the app does: its
/// keychain item, then its store file. Its Device key.
async fn create_persona(io: &OrgIo, handle_text: &str) -> DevicePublicKey {
    let persona_id = io
        .create_persona(&mut OsRng, Handle::parse(handle_text).unwrap(), Name::parse("Test").unwrap(), Surname::parse("User").unwrap())
        .await
        .unwrap();
    io.persona_public_keys(&persona_id).await.unwrap().1
}

// Two Personas' store files and items: one row per Device key, each bound
// through its own item.
// verifies: LLR-h9tcat
#[tokio::test(flavor = "multi_thread")]
async fn the_preflight_checks_every_persona_listed_in_the_data_dir() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::default().install();
    let dir = scratch("two");
    let io = OrgIo::open(&dir, TransportMode::Loopback).unwrap();
    let mut keys = vec![create_persona(&io, "alice").await, create_persona(&io, "bob").await];
    drop(io);

    let rows = check_transport_for_data_dir(&dir, TransportMode::Loopback, ONLINE_TIMEOUT).await;
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(rows.iter().all(|row| row.ok), "{rows:?}");
    let mut named: Vec<DevicePublicKey> = keys
        .iter()
        .copied()
        .filter(|key| rows.iter().any(|row| row.detail.contains(&hex_of(key))))
        .collect();
    named.sort();
    keys.sort();
    assert_eq!(named, keys, "each row names its own Device key");
}

// Abnormal: a data directory with no store file, or one that cannot be read,
// is one FAIL row naming the directory, and nothing is bound.
// verifies: LLR-h9tcat
#[tokio::test(flavor = "multi_thread")]
async fn a_data_dir_with_no_store_fails_the_check_naming_the_dir_and_binds_nothing() {
    let _guard = LOCK.lock().await;
    MemoryKeychain::default().install();
    let empty = scratch("empty");
    std::fs::create_dir_all(&empty).unwrap();
    std::fs::write(empty.join("notes.txt"), b"not a store").unwrap();
    let rows = check_transport_for_data_dir(&empty, TransportMode::Loopback, ONLINE_TIMEOUT).await;
    assert_one_failure_naming(&rows, &empty.display().to_string());

    let absent = scratch("absent");
    let rows = check_transport_for_data_dir(&absent, TransportMode::Loopback, ONLINE_TIMEOUT).await;
    assert_one_failure_naming(&rows, &absent.display().to_string());
}

// Abnormal: the binary with `ODS_DATA_DIR` unset, every other variable set,
// refuses and exits non-zero before any check: it names the variable and
// prints no checklist row (LLR-h9tcat as clarified 2026-10-13: no default).
// *Added 2026-10-14 (S4 review round 1, finding 12).* Mutation check, that
// day: with the binary's refusal replaced by a default (the temporary
// directory), this test failed ("the binary did not exit: it went on to the
// checks"); the refusal was restored.
// verifies: LLR-h9tcat
#[test]
fn the_binary_without_a_data_dir_exits_non_zero_before_any_check() {
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_preflight"))
        .env_remove("ODS_DATA_DIR")
        .env("ODS_TRANSPORT", "loopback")
        .env("ODS_CHAIN_WS", "ws://127.0.0.1:1")
        .env("ODS_CONTRACT_H160", format!("0x{}", "11".repeat(20)))
        .env("ODS_ADMIN_H160", format!("0x{}", "22".repeat(20)))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let started = std::time::Instant::now();
    while child.try_wait().unwrap().is_none() {
        if started.elapsed() > Duration::from_secs(30) {
            child.kill().unwrap();
            panic!("the binary did not exit: it went on to the checks");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let output = child.wait_with_output().unwrap();
    let (stdout, stderr) = (String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(!output.status.success(), "exits non-zero: {:?}", output.status);
    assert_eq!(stderr.trim(), "ODS_DATA_DIR not set", "names the variable");
    assert!(!stdout.contains("[PASS]") && !stdout.contains("[FAIL]"), "no check ran:\n{stdout}");
}

// Abnormal: the binary with `ODS_DATA_DIR` set to bytes that are not valid
// Unicode, every other variable set, refuses and exits non-zero before any
// check, saying what is wrong with the variable rather than that it is
// unset (LLR-h9tcat). Unix only: there an environment value can be any
// bytes.
// Mutation check, 2026-10-15: with the binary's two refusals merged back
// into one (`Err(_)` reporting "ODS_DATA_DIR not set"), this test failed on
// the stderr assertion; the split was restored.
// verifies: LLR-h9tcat
#[cfg(unix)]
#[test]
fn the_binary_with_a_non_unicode_data_dir_says_so_and_exits_before_any_check() {
    use std::os::unix::ffi::OsStrExt;
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_preflight"))
        .env("ODS_DATA_DIR", std::ffi::OsStr::from_bytes(b"/tmp/ods-\xff"))
        .env("ODS_TRANSPORT", "loopback")
        .env("ODS_CHAIN_WS", "ws://127.0.0.1:1")
        .env("ODS_CONTRACT_H160", format!("0x{}", "11".repeat(20)))
        .env("ODS_ADMIN_H160", format!("0x{}", "22".repeat(20)))
        .output()
        .unwrap();
    let (stdout, stderr) = (String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(!output.status.success(), "exits non-zero: {:?}", output.status);
    assert_eq!(stderr.trim(), "ODS_DATA_DIR is not valid Unicode", "says what is wrong with the variable");
    assert!(!stdout.contains("[PASS]") && !stdout.contains("[FAIL]"), "no check ran:\n{stdout}");
}
