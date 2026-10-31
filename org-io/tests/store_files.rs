#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The store files (SDD-egx8rv): one sealed store file per
//! Persona, named for its Device key, written atomically, removed when its
//! Persona is forgotten, and opened at startup through each Persona's
//! keychain item; a failed write or removal reloads the Persona from its
//! file. LLR-nt8dja, LLR-b9bm36, LLR-murjr6, LLR-uzax55, LLR-n9dnm4,
//! LLR-amv4mb, REQ-6d9xv6 as narrowed (an ordinary save needs no keychain),
//! and LLR-85zque, per Persona.
//!
//! Every test runs over a fresh directory and over org-io's in-memory
//! keychain, one test at a time under `fresh_keychain`'s lock: keyring's
//! default credential builder is process-global.
//!
//! A Persona is made as the handle makes one (a proposed founding,
//! `common::handles::create`), and the store change written through the
//! handle is the founding's proxy, recorded by the handle's own founding
//! write (`founder`, `open_over_chain`).

use std::path::{Path, PathBuf};
use std::sync::MutexGuard;

use org_node::service::OrgService;
use org_node::store::PersonaStore;
use org_node::{DevicePublicKey, OrgId, PersonaId};
use org_io::store_file::{list_stores, store_path, StoreFileStep};
use org_io::test_support::{write_store, FakeChain, MemoryKeychain, Refusal};
use org_io::transport::TransportMode;
use org_io::{HeldTarget, KeychainError, OpenError, OrgIo, StoreFailure, WriteRefused, WriteReport};
use rand::rngs::OsRng;

mod common;
use common::handles::{file_of, fresh_keychain, held_store, make_read_only, make_writable, names_in};

const SERVICE: &str = "ods-device-seed";

/// `fresh_keychain`'s lock and keychain, and a fresh directory for test
/// `tag` (not created).
fn fresh(tag: &str) -> (MutexGuard<'static, ()>, MemoryKeychain, PathBuf) {
    let (guard, keychain) = fresh_keychain();
    let dir = std::env::temp_dir().join(format!("org-io-store-files-{tag}-{}", std::process::id()));
    if dir.exists() {
        make_writable(&dir);
    }
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    (guard, keychain, dir)
}

fn open(dir: &Path) -> OrgIo {
    OrgIo::open(dir, TransportMode::Loopback).unwrap()
}

fn open_refused(dir: &Path) -> OpenError {
    match OrgIo::open(dir, TransportMode::Loopback) {
        Ok(_) => panic!("opened, and it should have been refused"),
        Err(error) => error,
    }
}

/// `future`, run to completion on a runtime of its own: the handle's methods
/// are `async`, and most tests here are not.
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
}

fn create(io: &OrgIo, handle: &str) -> PersonaId {
    block_on(common::handles::create(io, handle))
}

/// The Device key of the Persona `persona_id`, as the handle lists it.
fn device_key_of(io: &OrgIo, persona_id: &PersonaId) -> DevicePublicKey {
    block_on(common::handles::device_key_of(io, persona_id))
}

/// `open`, over a substitute chain the founding's write goes to.
fn open_over_chain(dir: &Path) -> OrgIo {
    open_over(dir, &FakeChain::new())
}

/// `open`, over `chain`.
fn open_over(dir: &Path, chain: &FakeChain) -> OrgIo {
    OrgIo::open_for_test(dir, TransportMode::Loopback, chain).unwrap()
}

/// A founding proposed through the handle: its Persona and founder Device key. Its write
/// records the proxy (`write_organisation_to_chain`), the store write the reload tests need.
fn founder(io: &OrgIo) -> (PersonaId, DevicePublicKey) {
    let proposed = block_on(io.propose_founding_organisation("Alice", "Doe", "alice", "Acme")).unwrap();
    (block_on(common::handles::persona_of(io, &proposed.founder)), proposed.founder)
}

// ---- LLR-nt8dja: one file per Persona, named for its Device key -------------

// verifies: LLR-nt8dja
#[test]
fn each_persona_has_its_own_file_named_for_its_device_key() {
    let (_guard, _keychain, dir) = fresh("one-file-each");
    let io = open(&dir);
    let alice = create(&io, "alice");
    let bob = create(&io, "bob");
    let (alice_key, bob_key) = (device_key_of(&io, &alice), device_key_of(&io, &bob));
    let mut expected = vec![
        format!("persona-{}.store", hex::encode(alice_key.as_bytes())),
        format!("persona-{}.store", hex::encode(bob_key.as_bytes())),
    ];
    expected.sort();
    assert_eq!(names_in(&dir), expected, "one file per Persona, and nothing else");
    assert_eq!(store_path(&dir, &alice_key), file_of(&dir, &alice_key));
    let mut listed = list_stores(&dir).unwrap();
    listed.sort_by_key(|key| *key.as_bytes());
    let mut keys = vec![alice_key, bob_key];
    keys.sort_by_key(|key| *key.as_bytes());
    assert_eq!(listed, keys, "the listing parses each name into its Device key");
}

// Abnormal input: names near the store file's form.
// verifies: LLR-nt8dja
#[test]
fn the_listing_ignores_every_other_name() {
    let (_guard, _keychain, dir) = fresh("listing-ignores");
    let io = open(&dir);
    let alice = create(&io, "alice");
    let alice_key = device_key_of(&io, &alice);
    let hex_key = hex::encode(alice_key.as_bytes());
    for name in [
        "persona-xyz.store".to_string(),
        "notes.txt".to_string(),
        format!("persona-{}.store", &hex_key[..63]),
        format!("persona-{}0.store", hex_key),
        format!("persona-{}.store", hex_key.to_uppercase()),
        format!("persona-{}.store.tmp-0123456789abcdef", hex_key),
        format!("persona-{}.store", "zz".repeat(32)),
        format!("persona-{}.bin", hex_key),
    ] {
        std::fs::write(dir.join(&name), b"not a store").unwrap();
    }
    assert_eq!(list_stores(&dir).unwrap(), vec![alice_key], "only the one store file is listed");
}

// ---- LLR-b9bm36: the atomic write -------------------------------------------

// It reproduces PR-n696wv.
// verifies: LLR-b9bm36, PR-n696wv
#[test]
fn the_file_holds_exactly_the_sealed_bytes() {
    let (_guard, _keychain, dir) = fresh("exact-bytes");
    std::fs::create_dir_all(&dir).unwrap();
    let mut service = OrgService::new(vec![]).unwrap();
    let (_device_key, _secret, change) = service.propose_founding(&mut OsRng, "A", "U", "alice", "Acme".to_string()).unwrap();
    let org_node::StoreChange::Sealed(write) = change;
    write_store(&dir, &write).unwrap();
    let file = file_of(&dir, &write.device_key);
    assert_eq!(std::fs::read(&file).unwrap(), write.sealed.as_bytes(), "exactly the sealed bytes");
    assert_eq!(names_in(&dir).len(), 1, "no temporary file is left: {:?}", names_in(&dir));
}

// Abnormal: the write is cut off after the temporary file is written and
// before it replaces the store file.
// It reproduces PR-n696wv.
// verifies: LLR-b9bm36, PR-n696wv
#[test]
fn a_write_interrupted_after_the_temporary_leaves_the_old_store_unsealing() {
    let (_guard, _keychain, dir) = fresh("interrupted");
    std::fs::create_dir_all(&dir).unwrap();
    let mut service = OrgService::new(vec![]).unwrap();
    let (device_key, secret, change) = service.propose_founding(&mut OsRng, "A", "U", "alice", "Acme".to_string()).unwrap();
    let org_node::StoreChange::Sealed(first) = change;
    write_store(&dir, &first).unwrap();
    let org_node::StoreChange::Sealed(second) = service
        .record_founding_proxy(&mut OsRng, &device_key, OrgId::new([0x07; 20]), org_node::ChainAccount::new([0x07; 32]))
        .unwrap();

    org_io::test_support::fail_next_write_after_temporary();
    let failed = write_store(&dir, &second).unwrap_err();
    assert_eq!(failed.step, StoreFileStep::Rename, "{failed}");
    assert_eq!(names_in(&dir).len(), 1, "the temporary file is removed: {:?}", names_in(&dir));
    let file = file_of(&dir, &first.device_key);
    assert_eq!(std::fs::read(&file).unwrap(), first.sealed.as_bytes(), "the old store is in place");
    let reopened = PersonaStore::unseal(&first.sealed, first.device_key, secret).unwrap();
    assert!(reopened.data().unwrap().founding.as_ref().unwrap().proxy.is_none(), "and unseals to the old state");
}

// ---- LLR-murjr6: a failed write reloads the Persona from its file ------------

// Abnormal: the directory turns read-only after the first write. The
// handle's own operation makes and writes the change — the founding's write
// records its proxy — and that store write fails.
// verifies: LLR-murjr6
#[cfg(unix)]
#[test]
fn a_failed_write_reports_failure_and_reloads_the_persona_from_its_file() {
    let (_guard, _keychain, dir) = fresh("failed-write");
    let chain = FakeChain::new();
    let io = open_over(&dir, &chain);
    let (alice, alice_key) = founder(&io);
    let before = std::fs::read(file_of(&dir, &alice_key)).unwrap();
    make_read_only(&dir);
    let refused = block_on(io.write_organisation_to_chain(HeldTarget::Founding(alice_key)));
    make_writable(&dir);
    match refused {
        Err(WriteRefused::Store { failure: StoreFailure::Reloaded { failed }, .. }) => {
            assert_eq!(failed.step, StoreFileStep::CreateTemporary, "{failed}");
        }
        other => panic!("not reported as a failed write, reloaded: {other:?}"),
    }
    assert_eq!(std::fs::read(file_of(&dir, &alice_key)).unwrap(), before);
    // Seen through the handle: what it holds is what
    // a fresh open of the file shows, and the next founding write starts with
    // a new proxy, so the reloaded Persona kept no proxy (the change is gone).
    assert_eq!(block_on(held_store(&io, &alice)), block_on(held_store(&open(&dir), &alice)), "the service's state is the file's");
    let calls = chain.founding_calls().len();
    assert!(matches!(block_on(io.write_organisation_to_chain(HeldTarget::Founding(alice_key))), Ok(WriteReport::Included { .. })));
    assert_eq!(chain.founding_calls()[calls].0, "create_proxy", "the change is gone: no proxy kept, a new one created");
}

// Abnormal, twice over: the write fails (a read-only directory) and so does
// the reload (the keychain refuses). The Persona is dropped from the running
// service, but nothing on disk is deleted: its file and its keychain item are
// unchanged, and the next open lists it again (owner ruling of 2026-10-13,
// option A: org-node's `drop_store` is its removal rule's second exempt
// place, verified by org-node's own scan and sealed_store tests). The
// failed write is the founding write's, as in the test above.
// verifies: LLR-murjr6
#[cfg(unix)]
#[test]
fn a_dropped_persona_keeps_its_file_and_item_and_opens_again() {
    let (_guard, keychain, dir) = fresh("dropped");
    let io = open_over_chain(&dir);
    let (alice, alice_key) = founder(&io);
    let file = file_of(&dir, &alice_key);
    let bytes = std::fs::read(&file).unwrap();
    let item = (SERVICE.to_string(), hex::encode(alice_key.as_bytes()));

    make_read_only(&dir);
    keychain.refuse_as(Some(Refusal::Unavailable));
    let refused = block_on(io.write_organisation_to_chain(HeldTarget::Founding(alice_key)));
    keychain.refuse_as(None);
    make_writable(&dir);
    match refused {
        Err(WriteRefused::Store { failure: StoreFailure::Dropped { reload, .. }, .. }) => {
            assert!(matches!(*reload, OpenError::Keychain { .. }), "{reload:?}");
        }
        other => panic!("not reported as dropped after a failed reload: {other:?}"),
    }
    assert!(!block_on(io.personas()).iter().any(|persona| persona.device_key == alice_key), "dropped from the running service");
    assert_eq!(std::fs::read(&file).unwrap(), bytes, "the store file is unchanged");
    assert!(keychain.items().contains(&item), "the keychain item is unchanged");
    drop(io);

    let again = open(&dir);
    assert_eq!(block_on(again.persona_public_keys(&alice)).unwrap().1, alice_key, "the next open lists it again");
}

// ---- LLR-uzax55, LLR-85zque: opening ---------------------------------------

// verifies: LLR-uzax55, LLR-85zque
#[test]
fn opening_creates_the_data_dir_and_unseals_every_store() {
    let (_guard, _keychain, dir) = fresh("open-unseals");
    let data_dir = dir.join("missing").join("parents");
    let io = open(&data_dir);
    assert!(data_dir.is_dir(), "the data directory and its parents are created");
    let alice = create(&io, "alice");
    let bob = create(&io, "bob");
    let (alice_held, bob_held) = (block_on(held_store(&io, &alice)), block_on(held_store(&io, &bob)));
    drop(io);
    let again = open(&data_dir);
    assert_eq!(block_on(again.personas()).len(), 2);
    assert_eq!(block_on(held_store(&again, &alice)), alice_held);
    assert_eq!(block_on(held_store(&again, &bob)), bob_held);
}

// Abnormal: the keychain holds no item for a listed store (REQ-fh5gsq).
// verifies: LLR-uzax55, LLR-85zque
#[test]
fn a_store_whose_item_is_missing_is_refused_and_left_untouched() {
    let (_guard, _keychain, dir) = fresh("item-missing");
    let io = open(&dir);
    let alice = create(&io, "alice");
    let device_key = device_key_of(&io, &alice);
    drop(io);
    let file = file_of(&dir, &device_key);
    let bytes = std::fs::read(&file).unwrap();
    let names = names_in(&dir);

    let empty = MemoryKeychain::default();
    empty.install();
    match open_refused(&dir) {
        OpenError::KeychainItemMissing { device_key: named, path } => {
            assert_eq!(named, device_key);
            assert_eq!(path, file);
        }
        other => panic!("not refused as a missing item: {other:?}"),
    }
    assert_eq!(std::fs::read(&file).unwrap(), bytes, "the file is untouched");
    assert_eq!(names_in(&dir), names, "no file is created");
    assert!(empty.items().is_empty(), "no item is created");
}

// Abnormal: the keychain is locked.
// verifies: LLR-uzax55, LLR-85zque
#[test]
fn a_locked_keychain_refuses_opening_naming_the_persona() {
    let (_guard, keychain, dir) = fresh("locked");
    let io = open(&dir);
    let alice = create(&io, "alice");
    let device_key = device_key_of(&io, &alice);
    drop(io);
    keychain.refuse_as(Some(Refusal::Locked));
    let refused = open_refused(&dir);
    match &refused {
        OpenError::Keychain { error: KeychainError::Locked { device_key: named }, .. } => assert_eq!(*named, device_key),
        other => panic!("not refused as a locked keychain: {other:?}"),
    }
    assert!(refused.to_string().contains(&format!("{device_key:?}")), "the refusal names the Device key: {refused}");
}

// The store is the Persona's file in the directory `open` created.
// verifies: LLR-85zque
#[test]
fn the_store_is_opened_inside_the_data_dir_it_creates() {
    let (_guard, _keychain, dir) = fresh("inside-data-dir");
    let data_dir = dir.join("a").join("b");
    assert!(!data_dir.exists());
    let io = open(&data_dir);
    assert!(data_dir.is_dir(), "the data directory and its parents are created");
    let alice = create(&io, "alice");
    assert!(file_of(&data_dir, &device_key_of(&io, &alice)).is_file(), "the store is the Persona's file in the data directory");
    drop(io);
    // Reopened at the same place, it is the same store.
    assert_eq!(block_on(open(&data_dir).personas()).len(), 1);
}

// Abnormal input: the data directory would sit under a regular file.
// verifies: LLR-85zque
#[test]
fn a_data_dir_that_cannot_be_created_is_refused_naming_it() {
    let (_guard, _keychain, dir) = fresh("cannot-create");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("plain-file");
    std::fs::write(&file, b"not a directory").unwrap();
    let data_dir = file.join("sub");
    let refused = open_refused(&data_dir);
    assert!(matches!(&refused, OpenError::CreateDataDir { path, .. } if *path == data_dir), "{refused:?}");
    assert!(refused.to_string().starts_with(&format!("create data_dir {}", data_dir.display())), "{refused}");
}

// ---- REQ-6d9xv6 as narrowed: an ordinary save needs no keychain --------------

// Ruling 2 (2026-10-07): saving a change to an open store needs no keychain
// (REQ-6d9xv6's narrowing note; org-node seals under the open store's key).
// The save is the founding write's proxy record, made and written by the
// handle.
// verifies: REQ-6d9xv6
#[test]
fn an_ordinary_save_succeeds_while_the_keychain_is_locked() {
    let (_guard, keychain, dir) = fresh("save-locked");
    let io = open_over_chain(&dir);
    let (_, alice_key) = founder(&io);
    let file = file_of(&dir, &alice_key);
    let before = std::fs::read(&file).unwrap();
    keychain.refuse_as(Some(Refusal::Locked));
    let calls_before = keychain.calls();
    let written = block_on(io.write_organisation_to_chain(HeldTarget::Founding(alice_key)));
    keychain.refuse_as(None);
    assert!(matches!(written, Ok(WriteReport::Included { .. })), "{written:?}");
    assert_eq!(keychain.calls(), calls_before, "no keychain call was made");
    assert_ne!(std::fs::read(&file).unwrap(), before, "the store file changed");
}
