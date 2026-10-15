#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The contract test of org-io's keychain fake (S4 TEST review round 1,
//! finding-1, 2026-10-15): `test_support::MemoryKeychain` stands in for the
//! OS keychain behind keyring's credential interface
//! (`keyring::credential::CredentialBuilderApi`, LLR-36jer8), so the same
//! assertions run against it and against keyring's native store for this
//! target (macOS Keychain Services, the Windows Credential Manager, the Linux
//! Secret Service).
//!
//! The fake's run always runs. The real one runs only with
//! `ODS_REAL_KEYCHAIN_TESTS=1`, and only on a target with a native backend;
//! otherwise it prints why it skipped and passes. *Since 2026-10-15 (S4
//! review round 2, finding-8)* it is also `#[ignore]`d, so a plain run
//! reports it ignored rather than passed; run it with `--ignored` and the
//! variable. It writes under a service
//! named `ods-contract-test-<random>`, never `ods-device-seed`, and deletes
//! every item it wrote, on failure too. Neither run installs a default
//! credential builder: each builds its credentials from the builder it is
//! given, so this binary shares no process-global state with the others.
//!
//! *Mutation checks, 2026-10-15:* `MemoryCredential::delete_credential`
//! made to return `Ok(())` for a missing item failed the fake's run ("a
//! missing item is deleted"); `MemoryKeychain::build` made to give each
//! credential a map of its own failed it at the read through a second
//! credential. Both mutations were removed. The real run, the same day, in
//! the agent sandbox on macOS: the two missing-item checks passed against
//! Keychain Services, and the first write was refused by the sandbox
//! (`PlatformFailure`, "UNIX[Operation not permitted]"). *Updated
//! 2026-10-15 (S4 TEST review round 2, notes):* the owner ran it to the end
//! outside the sandbox that day, against Keychain Services: 2 passed.
//!
//! The real run is opt-in because it writes to the user's own keychain:
//! items under a test service, deleted when the run ends, but writes all the
//! same, and on macOS each may raise a keychain access prompt. A plain
//! `cargo test` touches only the fake.
//!
//! The fake's `Refusal::Locked` imitates a locked store; the locked store's
//! contract is `locked_contract`, below, whose native run cannot reach a
//! locked keychain in the test environment and is `#[ignore]`d with why.

use keyring::credential::{Credential, CredentialBuilderApi};
use org_io::test_support::MemoryKeychain;
use rand::RngCore;

/// The environment variable that lets the real run touch the OS keychain.
const REAL_KEYCHAIN: &str = "ODS_REAL_KEYCHAIN_TESTS";

const USERS: [&str; 2] = ["contract-test-a", "contract-test-b"];

/// The items one run may have written, deleted when the run ends, whatever
/// became of it. A delete of an item that is not there is ignored.
struct Cleanup<'a> {
    builder: &'a dyn CredentialBuilderApi,
    service: String,
}

impl Drop for Cleanup<'_> {
    fn drop(&mut self) {
        for user in USERS {
            if let Ok(credential) = self.builder.build(None, &self.service, user) {
                let _ = credential.delete_credential();
            }
        }
    }
}

/// What keyring's functional need asks of a store (soup.md, keyring row):
/// 32 arbitrary bytes kept byte-exact under a (service, user) pair and read
/// back through another credential for that pair; a missing item reported
/// as `NoEntry`, distinctly; one pair's item not another's; a write replacing
/// the item; a delete removing it.
fn contract(builder: &dyn CredentialBuilderApi, run: &str) {
    let mut suffix = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut suffix);
    let service = format!("ods-contract-test-{}", hex::encode(suffix));
    let cleanup = Cleanup { builder, service: service.clone() };
    let credential = |user: &str| -> Box<Credential> { builder.build(None, &service, user).unwrap() };
    let [user, other_user] = USERS;

    // A missing item is NoEntry, read or deleted.
    assert!(matches!(credential(user).get_secret(), Err(keyring::Error::NoEntry)), "{run}: a missing item is read");
    assert!(matches!(credential(user).delete_credential(), Err(keyring::Error::NoEntry)), "{run}: a missing item is deleted");

    // 32 bytes no text encoding accepts (a zero, 0xFF, a lone continuation
    // byte), byte-exact, through a second credential for the same pair.
    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);
    seed[0] = 0x00;
    seed[1] = 0xFF;
    seed[2] = 0x80;
    credential(user).set_secret(&seed).unwrap();
    assert_eq!(credential(user).get_secret().unwrap(), seed.to_vec(), "{run}: the item reads back byte for byte");

    // Another user under the same service is another item.
    assert!(matches!(credential(other_user).get_secret(), Err(keyring::Error::NoEntry)), "{run}: one item per pair");

    // A write replaces the item, whatever its length: the lengths org-io's
    // tests put in the fake to stand for a malformed item, and the seed's.
    for length in [31usize, 33, 32] {
        let bytes: Vec<u8> = (0..length).map(|index| (index as u8).wrapping_mul(37) ^ 0xA5).collect();
        credential(user).set_secret(&bytes).unwrap();
        assert_eq!(credential(user).get_secret().unwrap(), bytes, "{run}: a write of {length} bytes replaces the item");
    }

    // A delete removes the item, and the pair's other item is untouched.
    credential(other_user).set_secret(&seed).unwrap();
    credential(user).delete_credential().unwrap();
    assert!(matches!(credential(user).get_secret(), Err(keyring::Error::NoEntry)), "{run}: a deleted item is gone");
    assert!(matches!(credential(user).delete_credential(), Err(keyring::Error::NoEntry)), "{run}: deleted once");
    assert_eq!(credential(other_user).get_secret().unwrap(), seed.to_vec(), "{run}: the other user's item stays");
    drop(cleanup);
    assert!(matches!(credential(other_user).get_secret(), Err(keyring::Error::NoEntry)), "{run}: the run left no item");
}

// verifies: LLR-36jer8
#[test]
fn the_in_memory_keychain_keeps_the_credential_contract() {
    contract(&MemoryKeychain::default(), "MemoryKeychain");
}

// *Ignored 2026-10-15 (S4 review round 2, finding-8):* it returned "ok"
// without running unless the variable was set; now it is reported ignored,
// and a run by hand still needs the variable.
// verifies: LLR-36jer8
#[test]
#[ignore = "touches the OS keychain; run by hand with ODS_REAL_KEYCHAIN_TESTS=1 and --ignored"]
fn the_os_keychain_keeps_the_credential_contract() {
    if std::env::var(REAL_KEYCHAIN).as_deref() != Ok("1") {
        eprintln!("skipped: the OS keychain is touched only with {REAL_KEYCHAIN}=1");
        return;
    }
    real_run();
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "windows", target_os = "linux"))]
fn real_run() {
    contract(&*keyring::default::default_credential_builder(), "the OS keychain");
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows", target_os = "linux")))]
fn real_run() {
    eprintln!("skipped: this target has no native keychain backend (LLR-8erfbu)");
}

// ---- A locked store (S4 TEST review round 2, finding-4, 2026-10-15) --------

/// What a locked store answers, the content `Refusal::Locked` imitates:
/// every read, write and delete is refused as `NoStorageAccess`, the error
/// LLR-8erfbu maps to `KeychainError::Locked`. (*Corrected 2026-10-15, S4
/// TEST review round 3, finding-1:* the doc also said "and nothing is
/// written", which this function does not check; dropped.)
fn locked_contract(builder: &dyn CredentialBuilderApi, run: &str) {
    let mut suffix = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut suffix);
    let service = format!("ods-contract-test-{}", hex::encode(suffix));
    let _cleanup = Cleanup { builder, service: service.clone() };
    let credential = builder.build(None, &service, USERS[0]).unwrap();
    let locked = |result: &keyring::Result<()>| matches!(result, Err(keyring::Error::NoStorageAccess(_)));
    let read = credential.get_secret().map(|_| ());
    assert!(locked(&read), "{run}: a read from a locked store is NoStorageAccess, not {read:?}");
    let write = credential.set_secret(&[7u8; 32]);
    assert!(locked(&write), "{run}: a write to a locked store is NoStorageAccess, not {write:?}");
    let delete = credential.delete_credential();
    assert!(locked(&delete), "{run}: a delete from a locked store is NoStorageAccess, not {delete:?}");
}

// The fake told to refuse as locked keeps the locked store's contract.
// Mutation check, 2026-10-15: `Refusal::Locked` made to answer
// `PlatformFailure` failed this test at the read.
// *Re-annotated 2026-10-15 (S4 TEST review round 3, finding-1):* it cited
// LLR-8erfbu, org-io's mapping of `NoStorageAccess` to
// `KeychainError::Locked`, which it does not exercise: it checks the fake's
// fidelity to a locked store, LLR-36jer8, as this file's other contract
// tests do.
// verifies: LLR-36jer8
#[test]
fn the_in_memory_keychain_refuses_as_a_locked_store_does() {
    let keychain = MemoryKeychain::default();
    keychain.refuse_as(Some(org_io::test_support::Refusal::Locked));
    locked_contract(&keychain, "MemoryKeychain, locked");
}

// The same assertions against the native store, locked. It cannot run in
// the test environment: a locked keychain is not reachable there. On macOS
// the store a test reaches is the user's login keychain, which a test can
// lock only by locking the user's own session keychain (and the agent
// sandbox refuses every write anyway); the Windows Credential Manager has
// no locked state a process can enter; the Linux Secret Service needs a
// running, locked collection the test machine does not have. To run it by
// hand: lock the store (macOS: `security lock-keychain`), then
// `ODS_REAL_KEYCHAIN_TESTS=1 cargo test --test keychain_contract -- --ignored`,
// then unlock it. It writes nothing when the store is locked as stated, and
// deletes what it wrote when it is not.
// *Re-annotated 2026-10-15 (S4 TEST review round 3, finding-1):* it cited
// LLR-8erfbu, which it does not exercise (it runs keyring's native builder,
// not org-io's mapping); it is the native half of the fake's fidelity
// check, LLR-36jer8.
// verifies: LLR-36jer8
#[test]
#[ignore = "a locked native keychain is not reachable in the test environment; run by hand with the store locked"]
fn a_locked_os_keychain_refuses_as_no_storage_access() {
    if std::env::var(REAL_KEYCHAIN).as_deref() != Ok("1") {
        eprintln!("skipped: the OS keychain is touched only with {REAL_KEYCHAIN}=1");
        return;
    }
    real_locked_run();
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "windows", target_os = "linux"))]
fn real_locked_run() {
    locked_contract(&*keyring::default::default_credential_builder(), "the OS keychain, locked");
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows", target_os = "linux")))]
fn real_locked_run() {
    eprintln!("skipped: this target has no native keychain backend (LLR-8erfbu)");
}
