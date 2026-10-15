#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The keychain module (SDD-4fvypt; LLR-677fqq, LLR-8erfbu, LLR-n9dnm4,
//! LLR-36jer8), driven through its `test_support` facade over org-io's
//! in-memory credential store, `MemoryKeychain`. The OS keychain is never
//! touched: every test installs a fresh `MemoryKeychain`, and the tests run
//! one at a time under `LOCK`, because keyring's default credential builder
//! is process-global.

use std::sync::{Mutex, MutexGuard};

use org_node::{DevicePublicKey, DeviceSecret};
use org_io::test_support::keychain::{read_device_secret, with_endpoint_seed, write_device_secret};
use org_io::test_support::{MemoryKeychain, Refusal};
use org_io::KeychainError;
use rand::rngs::OsRng;

static LOCK: Mutex<()> = Mutex::new(());

const SERVICE: &str = "ods-device-seed";

/// The lock, and a fresh in-memory keychain installed as keyring's default
/// credential builder for as long as the lock is held.
fn fresh_keychain() -> (MutexGuard<'static, ()>, MemoryKeychain) {
    let guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let keychain = MemoryKeychain::default();
    keychain.install();
    (guard, keychain)
}

fn user_of(device_key: &DevicePublicKey) -> String {
    hex::encode(device_key.as_bytes())
}

fn keychain_source() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/keychain.rs")).unwrap()
}

/// The source's lines with `//` comments removed (doc comments included).
fn code_lines(source: &str) -> Vec<String> {
    source.lines().map(|line| line.split("//").next().unwrap_or("").to_string()).collect()
}

// verifies: LLR-677fqq
#[test]
fn a_written_secret_reads_back_as_the_same_device_key() {
    let (_guard, _keychain) = fresh_keychain();
    let secret = DeviceSecret::generate(&mut OsRng);
    let device_key = secret.device_key();
    write_device_secret(&device_key, &secret).unwrap();
    let read = read_device_secret(&device_key).unwrap();
    assert_eq!(read.device_key(), device_key);
    // Compared inside two nested lends: the lend returns no bytes
    // (org-node's lend signature, 2026-10-15), so no seed is copied out.
    let same = read.lend_to_keychain(|read_seed| {
        secret.lend_to_keychain(|seed| if read_seed == seed { Ok(()) } else { Err(std::fmt::Error) })
    });
    assert!(same.is_ok(), "the item holds the seed, byte for byte");
}

// verifies: LLR-677fqq
#[test]
fn the_item_is_named_for_the_device_key() {
    let (_guard, keychain) = fresh_keychain();
    let secret = DeviceSecret::generate(&mut OsRng);
    let device_key = secret.device_key();
    write_device_secret(&device_key, &secret).unwrap();
    let user = user_of(&device_key);
    assert_eq!(user.len(), 64);
    assert_eq!(user, user.to_lowercase());
    assert_eq!(keychain.items(), vec![(SERVICE.to_string(), user)], "one item: service ods-device-seed, user the key's hex");
}

// verifies: LLR-677fqq
#[test]
fn a_secret_under_another_device_key_is_refused_as_key_mismatch() {
    let (_guard, _keychain) = fresh_keychain();
    let secret = DeviceSecret::generate(&mut OsRng);
    let other_key = DeviceSecret::generate(&mut OsRng).device_key();
    write_device_secret(&other_key, &secret).unwrap();
    match read_device_secret(&other_key) {
        Err(error) => assert_eq!(error, KeychainError::KeyMismatch { device_key: other_key }),
        Ok(_) => panic!("a secret of another Device key was handed out"),
    }
}

// verifies: LLR-677fqq
#[test]
fn with_endpoint_seed_lends_the_seed_once_and_refuses_a_short_item() {
    let (_guard, keychain) = fresh_keychain();
    let secret = DeviceSecret::generate(&mut OsRng);
    let device_key = secret.device_key();
    write_device_secret(&device_key, &secret).unwrap();
    let mut calls = 0;
    let endpoint_id = with_endpoint_seed(&device_key, |seed| {
        calls += 1;
        iroh::SecretKey::from_bytes(seed).public()
    })
    .unwrap();
    assert_eq!(calls, 1, "the build is called exactly once");
    assert_eq!(endpoint_id.as_bytes(), device_key.as_bytes(), "the lent seed is the Device key's");

    // Boundary values: one byte short, one byte long, empty.
    for length in [31usize, 33, 0] {
        let short_key = DeviceSecret::generate(&mut OsRng).device_key();
        keychain.insert(SERVICE, &user_of(&short_key), vec![0xA5; length]);
        let mut calls = 0;
        let refused = with_endpoint_seed(&short_key, |_| calls += 1).unwrap_err();
        assert_eq!(refused, KeychainError::Malformed { device_key: short_key }, "length {length}");
        assert_eq!(calls, 0, "a malformed item is never lent");
        assert!(!format!("{refused:?} {refused}").to_lowercase().contains("a5a5"), "no byte is echoed");
        match read_device_secret(&short_key) {
            Err(error) => assert_eq!(error, KeychainError::Malformed { device_key: short_key }),
            Ok(_) => panic!("a malformed item was read as a secret"),
        }
    }
}

// verifies: LLR-8erfbu
#[test]
fn a_missing_item_is_item_missing() {
    let (_guard, _keychain) = fresh_keychain();
    let device_key = DeviceSecret::generate(&mut OsRng).device_key();
    match read_device_secret(&device_key) {
        Err(error) => assert_eq!(error, KeychainError::ItemMissing { device_key }),
        Ok(_) => panic!("a missing item was read"),
    }
    assert_eq!(with_endpoint_seed(&device_key, |_| ()).unwrap_err(), KeychainError::ItemMissing { device_key });
}

// verifies: LLR-8erfbu
#[test]
fn a_locked_keychain_is_locked() {
    let (_guard, keychain) = fresh_keychain();
    let secret = DeviceSecret::generate(&mut OsRng);
    let device_key = secret.device_key();
    write_device_secret(&device_key, &secret).unwrap();
    keychain.refuse_as(Some(Refusal::Locked));
    match read_device_secret(&device_key) {
        Err(error) => assert_eq!(error, KeychainError::Locked { device_key }),
        Ok(_) => panic!("a locked keychain handed out a secret"),
    }
    assert_eq!(with_endpoint_seed(&device_key, |_| ()).unwrap_err(), KeychainError::Locked { device_key });
    let another = DeviceSecret::generate(&mut OsRng);
    let another_key = another.device_key();
    assert_eq!(write_device_secret(&another_key, &another).unwrap_err(), KeychainError::Locked { device_key: another_key });
    keychain.refuse_as(None);
    assert!(read_device_secret(&another_key).is_err(), "a refused write stored nothing");
}

// verifies: LLR-8erfbu
#[test]
fn any_other_failure_is_unavailable_with_no_secret_byte_in_its_reason() {
    let (_guard, keychain) = fresh_keychain();
    let device_key = DeviceSecret::generate(&mut OsRng).device_key();
    // A known item, so its bytes can be looked for in the refusal.
    keychain.insert(SERVICE, &user_of(&device_key), vec![0x5C; 32]);
    keychain.refuse_as(Some(Refusal::Unavailable));
    let refusals = [
        match read_device_secret(&device_key) {
            Err(error) => error,
            Ok(_) => panic!("an unavailable keychain handed out a secret"),
        },
        with_endpoint_seed(&device_key, |_| ()).unwrap_err(),
        write_device_secret(&device_key, &DeviceSecret::generate(&mut OsRng)).unwrap_err(),
    ];
    for refused in refusals {
        assert_eq!(refused, KeychainError::Unavailable { device_key, reason: "PlatformFailure" });
        let rendered = format!("{refused:?} {refused}");
        assert!(!rendered.to_lowercase().contains("5c5c"), "no secret byte: {rendered}");
        assert!(!rendered.contains("92, 92"), "no secret byte, in decimal: {rendered}");
        assert!(!rendered.contains("refused by the test"), "not the platform error's text: {rendered}");
    }
}

// *Added 2026-10-13 (T7's finding):* keyring 3.6.3 makes its non-persistent
// in-memory store the default on any target with no native backend
// (Android among them); org-io refuses there instead. No such target is
// installed here, so the test is a scan of the module.
// verifies: LLR-8erfbu
#[test]
fn a_target_without_a_native_backend_refuses_as_unavailable() {
    let native = "#[cfg(any(target_os = \"macos\", target_os = \"ios\", target_os = \"windows\", target_os = \"linux\"))]";
    let none = "#[cfg(not(any(target_os = \"macos\", target_os = \"ios\", target_os = \"windows\", target_os = \"linux\")))]";
    let source = keychain_source();
    let lines = code_lines(&source);
    // The block of `mod backend` under `gate`: its first and last line.
    let block_under = |gate: &str| -> (usize, usize) {
        let gate_line = lines.iter().position(|line| line.trim() == gate).unwrap_or_else(|| panic!("no `{gate}`"));
        assert!(lines[gate_line + 1].trim().starts_with("mod backend {"), "`{gate}` gates `mod backend`");
        let mut depth = 0i32;
        for (index, line) in lines.iter().enumerate().skip(gate_line + 1) {
            depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
            if depth == 0 {
                return (gate_line + 1, index);
            }
        }
        panic!("`mod backend` under `{gate}` is not closed");
    };
    let (native_start, native_end) = block_under(native);
    let (none_start, none_end) = block_under(none);

    for (index, line) in lines.iter().enumerate() {
        if line.contains("keyring") {
            assert!(
                (native_start..=native_end).contains(&index),
                "keychain.rs:{}: keyring is named outside the native backend: {line}",
                index + 1
            );
        }
    }
    let fallback = lines[none_start..=none_end].join("\n");
    for function in ["fn read_device_secret", "fn write_device_secret", "fn with_endpoint_seed"] {
        assert!(fallback.contains(function), "the fallback defines `{function}`");
    }
    assert_eq!(
        fallback.matches("KeychainError::Unavailable").count(),
        3,
        "each fallback function returns Unavailable:\n{fallback}"
    );
    // *Added 2026-10-14 (S4 review round 1, finding 12):* each with the
    // reason LLR-8erfbu's T8 note names. Mutation check, that day: the
    // source read with its first fallback reason replaced, in memory, by
    // "Unknown" failed this assertion; the replacement was removed.
    assert_eq!(
        fallback.matches("KeychainError::Unavailable { device_key: *device_key, reason: \"NoNativeBackend\" }").count(),
        3,
        "each fallback function's reason is NoNativeBackend:\n{fallback}"
    );
    assert!(!fallback.contains("keyring"), "the fallback calls no keyring");
}

// verifies: LLR-n9dnm4
#[test]
fn the_write_lends_the_bytes_only_through_lend_to_keychain() {
    let lines = code_lines(&keychain_source());
    let writes: Vec<&String> = lines.iter().filter(|line| line.contains("set_secret")).collect();
    assert_eq!(writes.len(), 1, "set_secret appears once: {writes:?}");
    assert!(writes[0].contains("lend_to_keychain("), "inside a lend_to_keychain call: {}", writes[0]);
}
