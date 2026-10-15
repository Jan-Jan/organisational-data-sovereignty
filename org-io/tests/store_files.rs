#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
#![allow(clippy::result_large_err, reason = "the handle's `CreateError`, which names a Device public key")]
//! The store files (SDD-egx8rv; S4, task T9 of
//! docs/plans/2026-10-06-org-io-transport.md): one sealed store file per
//! Persona, named for its Device key, written atomically, removed when its
//! Persona is forgotten, and opened at startup through each Persona's
//! keychain item; a failed write or removal reloads the Persona from its
//! file. LLR-nt8dja, LLR-b9bm36, LLR-murjr6, LLR-uzax55, LLR-n9dnm4,
//! LLR-amv4mb, REQ-6d9xv6 as narrowed (an ordinary save needs no keychain),
//! and LLR-85zque, moved here from the app (`state_assembly.rs`) and
//! rewritten per Persona.
//!
//! Every test runs over a fresh directory and over org-io's in-memory
//! keychain, one test at a time under `fresh_keychain`'s lock: keyring's
//! default credential builder is process-global.

use std::path::{Path, PathBuf};
use std::sync::MutexGuard;
use std::time::Duration;

use org_node::service::OrgService;
use org_node::store::PersonaStore;
use org_node::{DevicePublicKey, Handle, MemberId, Name, OrgId, PersonaId, ReceiveKind, Surname};
use org_io::store_file::{list_stores, store_path, StoreFileStep};
use org_io::test_support::{write_store, FakeChain, MemoryKeychain, Refusal};
use org_io::transport::TransportMode;
use org_io::{CreateError, KeychainError, OpenError, OperationError, OrgIo, ReceiveError, Received, StoreFailure};
use rand::rngs::OsRng;

mod common;
use common::handles::{at, file_of, fresh_keychain, held_store, joiner, joiner_of, receiving_addr, NET};

const SERVICE: &str = "ods-device-seed";

/// `fresh_keychain`'s lock and keychain, and a fresh directory for test
/// `tag` (not created).
fn fresh(tag: &str) -> (MutexGuard<'static, ()>, MemoryKeychain, PathBuf) {
    let (guard, keychain) = fresh_keychain();
    let dir = std::env::temp_dir().join(format!("org-io-store-files-{tag}-{}", std::process::id()));
    if dir.exists() {
        make_writable(&dir);
    }
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

/// `future`, run to completion on a runtime of its own. *Added 2026-10-13
/// (S4, task T11):* the handle's methods are `async`, and most tests here
/// are not.
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
}

async fn create_async(io: &OrgIo, handle: &str) -> Result<PersonaId, CreateError> {
    io.create_persona(&mut OsRng, Handle::parse(handle).unwrap(), Name::parse("Test").unwrap(), Surname::parse("User").unwrap())
        .await
}

fn create(io: &OrgIo, handle: &str) -> Result<PersonaId, CreateError> {
    block_on(create_async(io, handle))
}

fn device_key_of(io: &OrgIo, persona_id: &PersonaId) -> DevicePublicKey {
    io.node_for_test().persona_public_keys(persona_id).unwrap().1
}

/// The directory's file names, sorted.
fn names_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> =
        std::fs::read_dir(dir).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().to_string()).collect();
    names.sort();
    names
}

fn make_read_only(dir: &Path) {
    let mut permissions = std::fs::metadata(dir).unwrap().permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(dir, permissions).unwrap();
}

#[allow(clippy::permissions_set_readonly_false)]
fn make_writable(dir: &Path) {
    let mut permissions = std::fs::metadata(dir).unwrap().permissions();
    permissions.set_readonly(false);
    std::fs::set_permissions(dir, permissions).unwrap();
}

/// The store `persona_id`'s file holds, unsealed with its keychain item, as
/// its `Debug` shows it.
fn on_disk(io: &OrgIo, dir: &Path, persona_id: &PersonaId) -> String {
    let device_key = device_key_of(io, persona_id);
    let sealed = org_node::SealedStore::from_bytes(std::fs::read(file_of(dir, &device_key)).unwrap());
    let secret = io.device_secret_for_test(&device_key).unwrap();
    let store = PersonaStore::unseal(&sealed, device_key, secret).unwrap();
    let service = OrgService::new(vec![store]).unwrap();
    format!("{:?}", service.store_data(persona_id).unwrap())
}

// ---- LLR-nt8dja: one file per Persona, named for its Device key -------------

// verifies: LLR-nt8dja
#[test]
fn each_persona_has_its_own_file_named_for_its_device_key() {
    let (_guard, _keychain, dir) = fresh("one-file-each");
    let io = open(&dir);
    let alice = create(&io, "alice").unwrap();
    let bob = create(&io, "bob").unwrap();
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
    let alice = create(&io, "alice").unwrap();
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

// *Annotated 2026-10-14 (S4 review round 1, finding 16):* PR-n696wv, which
// this test reproduces, moved to org-io's ledger in task T9; it is cited here
// as the resolution names it.
// verifies: LLR-b9bm36, PR-n696wv
#[test]
fn the_file_holds_exactly_the_sealed_bytes() {
    let (_guard, _keychain, dir) = fresh("exact-bytes");
    std::fs::create_dir_all(&dir).unwrap();
    let mut service = OrgService::new(vec![]).unwrap();
    let (_, _secret, write) = service
        .create_persona(&mut OsRng, Handle::parse("alice").unwrap(), Name::parse("A").unwrap(), Surname::parse("U").unwrap())
        .unwrap();
    write_store(&dir, &write).unwrap();
    let file = file_of(&dir, &write.device_key);
    assert_eq!(std::fs::read(&file).unwrap(), write.sealed.as_bytes(), "exactly the sealed bytes");
    assert_eq!(names_in(&dir).len(), 1, "no temporary file is left: {:?}", names_in(&dir));
}

// Abnormal: the write is cut off after the temporary file is written and
// before it replaces the store file.
// *Annotated 2026-10-14 (S4 review round 1, finding 16):* PR-n696wv, which
// this test reproduces, moved to org-io's ledger in task T9; it is cited here
// as the resolution names it.
// verifies: LLR-b9bm36, PR-n696wv
#[test]
fn a_write_interrupted_after_the_temporary_leaves_the_old_store_unsealing() {
    let (_guard, _keychain, dir) = fresh("interrupted");
    std::fs::create_dir_all(&dir).unwrap();
    let mut service = OrgService::new(vec![]).unwrap();
    let (persona_id, secret, first) = service
        .create_persona(&mut OsRng, Handle::parse("alice").unwrap(), Name::parse("A").unwrap(), Surname::parse("U").unwrap())
        .unwrap();
    write_store(&dir, &first).unwrap();
    let second = service.expect_admission(&mut OsRng, &persona_id, OrgId::new([0x07; 20])).unwrap();

    org_io::test_support::fail_next_write_after_temporary();
    let failed = write_store(&dir, &second).unwrap_err();
    assert_eq!(failed.step, StoreFileStep::Rename, "{failed}");
    assert_eq!(names_in(&dir).len(), 1, "the temporary file is removed: {:?}", names_in(&dir));
    let file = file_of(&dir, &first.device_key);
    assert_eq!(std::fs::read(&file).unwrap(), first.sealed.as_bytes(), "the old store is in place");
    let reopened = PersonaStore::unseal(&first.sealed, first.device_key, secret).unwrap();
    assert!(reopened.data().expected_admissions.is_empty(), "and unseals to the old state");
}

// ---- LLR-murjr6: a failed write reloads the Persona from its file ------------

// Abnormal: the directory turns read-only after the first write.
// verifies: LLR-murjr6
#[cfg(unix)]
#[test]
fn a_failed_write_reports_failure_and_reloads_the_persona_from_its_file() {
    let (_guard, _keychain, dir) = fresh("failed-write");
    let io = open(&dir);
    let alice = create(&io, "alice").unwrap();
    let before = std::fs::read(file_of(&dir, &device_key_of(&io, &alice))).unwrap();
    make_read_only(&dir);
    let refused = block_on(io.expect_admission(&mut OsRng, &alice, OrgId::new([0x07; 20])));
    make_writable(&dir);
    match refused {
        Err(OperationError::Store(StoreFailure::Reloaded { failed })) => {
            assert_eq!(failed.step, StoreFileStep::CreateTemporary, "{failed}");
        }
        other => panic!("not reported as a failed write, reloaded: {other:?}"),
    }
    assert!(io.node_for_test().store_data(&alice).unwrap().expected_admissions.is_empty(), "the change is gone");
    assert_eq!(held_store(&io, &alice), on_disk(&io, &dir, &alice), "the service's state is the file's");
    assert_eq!(std::fs::read(file_of(&dir, &device_key_of(&io, &alice))).unwrap(), before);
}

// Abnormal, twice over: the write fails (a read-only directory) and so does
// the reload (the keychain refuses). The Persona is dropped from the running
// service, but nothing on disk is deleted: its file and its keychain item are
// unchanged, and the next open lists it again (owner ruling of 2026-10-13,
// option A: org-node's `drop_store` is its removal rule's second exempt
// place, verified by org-node's own scan and sealed_store tests).
// verifies: LLR-murjr6
#[cfg(unix)]
#[test]
fn a_dropped_persona_keeps_its_file_and_item_and_opens_again() {
    let (_guard, keychain, dir) = fresh("dropped");
    let io = open(&dir);
    let alice = create(&io, "alice").unwrap();
    let alice_key = device_key_of(&io, &alice);
    let file = file_of(&dir, &alice_key);
    let bytes = std::fs::read(&file).unwrap();
    let item = (SERVICE.to_string(), hex::encode(alice_key.as_bytes()));

    make_read_only(&dir);
    keychain.refuse_as(Some(Refusal::Unavailable));
    let refused = block_on(io.expect_admission(&mut OsRng, &alice, OrgId::new([0x07; 20])));
    keychain.refuse_as(None);
    make_writable(&dir);
    match refused {
        Err(OperationError::Store(StoreFailure::Dropped { reload, .. })) => {
            assert!(matches!(*reload, OpenError::Keychain { .. }), "{reload:?}");
        }
        other => panic!("not reported as dropped after a failed reload: {other:?}"),
    }
    assert!(io.node_for_test().store_data(&alice).is_none(), "dropped from the running service");
    assert_eq!(std::fs::read(&file).unwrap(), bytes, "the store file is unchanged");
    assert!(keychain.items().contains(&item), "the keychain item is unchanged");
    drop(io);

    let again = open(&dir);
    assert_eq!(block_on(again.persona_public_keys(&alice)).unwrap().1, alice_key, "the next open lists it again");
}

// The reproduction of S2's save-after-mutate report (org-node's ledger,
// `2026-10-08-save-after-mutate.md`, which names this test): a commit whose
// store write fails after the chain write says so, and the node reloads.
// verifies: LLR-murjr6
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_commit_whose_store_write_fails_says_the_commit_was_made_and_reloads() {
    let (_guard, _keychain, dir) = fresh("commit-write-fails");
    let chain = FakeChain::new();
    let io = OrgIo::for_test(OrgService::new(vec![]).unwrap(), &chain).with_data_dir_for_test(dir.clone());
    std::fs::create_dir_all(&dir).unwrap();
    let alice = create_async(&io, "alice").await.unwrap();
    let org = io.found_organisation(&mut OsRng, &alice).await.unwrap();
    let bob = joiner(9, "bob");
    let update = io.admit_member(&mut OsRng, org, &bob).await.unwrap();
    make_read_only(&dir);
    // *S4 task T12, 2026-10-13:* `submit_commit_send` names no recipient.
    let refused = io.submit_commit_send(&mut OsRng, &update, None).await.unwrap_err();
    make_writable(&dir);
    assert!(
        refused.contains("the chain write executed and the commit was made, but storing it failed; reloaded from the file"),
        "{refused}"
    );
    assert!(!refused.contains("nothing committed"), "{refused}");
    assert_eq!(held_store(&io, &alice), on_disk(&io, &dir, &alice), "the service's state is the file's");
}

// ---- LLR-uzax55, LLR-85zque: opening ---------------------------------------

// verifies: LLR-uzax55, LLR-85zque
#[test]
fn opening_creates_the_data_dir_and_unseals_every_store() {
    let (_guard, _keychain, dir) = fresh("open-unseals");
    let data_dir = dir.join("missing").join("parents");
    let io = open(&data_dir);
    assert!(data_dir.is_dir(), "the data directory and its parents are created");
    let alice = create(&io, "alice").unwrap();
    let bob = create(&io, "bob").unwrap();
    let (alice_held, bob_held) = (held_store(&io, &alice), held_store(&io, &bob));
    drop(io);
    let again = open(&data_dir);
    assert_eq!(block_on(again.personas()).len(), 2);
    assert_eq!(held_store(&again, &alice), alice_held);
    assert_eq!(held_store(&again, &bob), bob_held);
}

// Abnormal: the keychain holds no item for a listed store (REQ-fh5gsq).
// verifies: LLR-uzax55, LLR-85zque
#[test]
fn a_store_whose_item_is_missing_is_refused_and_left_untouched() {
    let (_guard, _keychain, dir) = fresh("item-missing");
    let io = open(&dir);
    let alice = create(&io, "alice").unwrap();
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
    let alice = create(&io, "alice").unwrap();
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

// Moved from the app's `state_assembly.rs` (S4, task T9), rewritten per
// Persona: the store is the Persona's file in the directory `open` created.
// verifies: LLR-85zque
#[test]
fn the_store_is_opened_inside_the_data_dir_it_creates() {
    let (_guard, _keychain, dir) = fresh("inside-data-dir");
    let data_dir = dir.join("a").join("b");
    assert!(!data_dir.exists());
    let io = open(&data_dir);
    assert!(data_dir.is_dir(), "the data directory and its parents are created");
    let alice = create(&io, "alice").unwrap();
    assert!(file_of(&data_dir, &device_key_of(&io, &alice)).is_file(), "the store is the Persona's file in the data directory");
    drop(io);
    // Reopened at the same place, it is the same store.
    assert_eq!(block_on(open(&data_dir).personas()).len(), 1);
}

// Moved from the app's `state_assembly.rs` (S4, task T9). Abnormal input:
// the data directory would sit under a regular file.
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

// ---- LLR-n9dnm4: the keychain item before the file ---------------------------

// Normal, then abnormal twice: a refused keychain write leaves no file and no
// Persona; a refused file write after the item leaves the item and no
// Persona, which shows the item was written first.
// verifies: LLR-n9dnm4
#[test]
fn create_persona_writes_the_item_before_the_file_and_nothing_when_the_keychain_refuses() {
    let (_guard, keychain, dir) = fresh("create-order");
    let io = open(&dir);
    let alice = create(&io, "alice").unwrap();
    let alice_key = device_key_of(&io, &alice);
    assert_eq!(keychain.items(), vec![(SERVICE.to_string(), hex::encode(alice_key.as_bytes()))]);
    assert!(file_of(&dir, &alice_key).is_file());

    keychain.refuse_as(Some(Refusal::Unavailable));
    match create(&io, "bob") {
        Err(CreateError::Keychain(KeychainError::Unavailable { .. })) => {}
        other => panic!("not refused by the keychain: {other:?}"),
    }
    keychain.refuse_as(None);
    assert_eq!(block_on(io.personas()).len(), 1, "the refused Persona is dropped from the service");
    assert_eq!(names_in(&dir).len(), 1, "no file is written");
    assert_eq!(keychain.items().len(), 1, "no item is written");

    #[cfg(unix)]
    {
        make_read_only(&dir);
        let refused = create(&io, "carol");
        make_writable(&dir);
        assert!(matches!(refused, Err(CreateError::Store(StoreFailure::Dropped { .. }))), "{refused:?}");
        assert_eq!(block_on(io.personas()).len(), 1, "the Persona whose file was not written is dropped");
        assert_eq!(names_in(&dir).len(), 1, "no file is written");
        assert_eq!(keychain.items().len(), 2, "the item was written before the file was tried");
    }
}

// ---- REQ-6d9xv6 as narrowed: an ordinary save needs no keychain --------------

// Ruling 2 (2026-10-07): saving a change to an open store needs no keychain
// (REQ-6d9xv6's narrowing note; org-node seals under the open store's key).
// verifies: REQ-6d9xv6
#[test]
fn an_ordinary_save_succeeds_while_the_keychain_is_locked() {
    let (_guard, keychain, dir) = fresh("save-locked");
    let io = open(&dir);
    let alice = create(&io, "alice").unwrap();
    let file = file_of(&dir, &device_key_of(&io, &alice));
    let before = std::fs::read(&file).unwrap();
    keychain.refuse_as(Some(Refusal::Locked));
    let calls_before = keychain.calls();
    block_on(io.expect_admission(&mut OsRng, &alice, OrgId::new([0x07; 20]))).unwrap();
    assert_eq!(keychain.calls(), calls_before, "no keychain call was made");
    assert_ne!(std::fs::read(&file).unwrap(), before, "the store file changed");
}

// ---- LLR-amv4mb: a forgotten Persona's file is removed -----------------------

/// A, over `chain`, has founded an Organisation; B has two Personas, Bob
/// (expecting the admission, its endpoint bound) and Dave, and writes its
/// stores to `dir`.
struct Pair {
    a: OrgIo,
    b: OrgIo,
    org: OrgId,
    bob: PersonaId,
    dave: PersonaId,
    addr_b: iroh::EndpointAddr,
}

async fn pair(dir: &Path) -> Pair {
    let chain = FakeChain::new();
    let a_dir = dir.join("a");
    std::fs::create_dir_all(&a_dir).unwrap();
    let a = OrgIo::for_test(OrgService::new(vec![]).unwrap(), &chain).with_data_dir_for_test(a_dir);
    let alice = create_async(&a, "alice").await.unwrap();
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    // *S4 task T10, 2026-10-13:* A's endpoint is bound on its first send.
    let b_dir = dir.join("b");
    std::fs::create_dir_all(&b_dir).unwrap();
    let chain_b = FakeChain::sharing_slots_with(&chain);
    let b = OrgIo::for_test(OrgService::new(vec![]).unwrap(), &chain_b).with_data_dir_for_test(b_dir);
    let bob = create_async(&b, "bob").await.unwrap();
    let dave = create_async(&b, "dave").await.unwrap();
    b.expect_admission(&mut OsRng, &bob, org).await.unwrap();
    let addr_b = receiving_addr(&b).await;
    Pair { a, b, org, bob, dave, addr_b }
}

/// A sends `update`, built for B's Bob, to B's receive; returns B's result.
/// *S4 task T11, 2026-10-13:* B receives through `next_received`, which
/// reports a store change it could not write as `ReceiveError::Store`.
/// *S4 task T12, 2026-10-13:* `submit_commit_send` names no recipient, and
/// sends in Loopback only to B's address, Bob's Device's (B's first
/// Persona).
async fn deliver_to_b(pair: &mut Pair, update: org_io::BuiltUpdate) -> Result<Received, ReceiveError> {
    let Pair { a, b, addr_b, .. } = pair;
    let (received, sent) = tokio::join!(
        async { tokio::time::timeout(NET, b.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            a.submit_commit_send(&mut OsRng, &update, at(addr_b.clone())).await
        }
    );
    sent.unwrap();
    received
}

/// A admits Bob, then revokes him; returns B's result of the revocation.
async fn admit_then_revoke_bob(pair: &mut Pair, before_the_revocation: impl FnOnce(&Pair)) -> Result<Received, ReceiveError> {
    let joiner_bob = joiner_of(&pair.b, &pair.bob);
    let admission = pair.a.admit_member(&mut OsRng, pair.org, &joiner_bob).await.unwrap();
    deliver_to_b(pair, admission).await.unwrap();
    let record = pair.a.node_for_test().list_orgs().into_iter().find(|record| record.org_id == pair.org).cloned().unwrap();
    let member_bob: MemberId =
        record.trie_members.iter().find(|member| member.device_keys.contains(&joiner_bob.device_key)).unwrap().id;
    let revocation = pair.a.revoke_member(&mut OsRng, pair.org, member_bob).await.unwrap();
    before_the_revocation(pair);
    deliver_to_b(pair, revocation).await
}

// verifies: LLR-amv4mb
#[tokio::test(flavor = "multi_thread")]
async fn a_forgotten_store_is_removed_and_listed_no_more() {
    let (_guard, keychain, dir) = fresh("forgotten");
    let mut pair = pair(&dir).await;
    let b_dir = dir.join("b");
    let (bob_key, dave_key) = (device_key_of(&pair.b, &pair.bob), device_key_of(&pair.b, &pair.dave));
    let dave_bytes = std::fs::read(file_of(&b_dir, &dave_key)).unwrap();

    let received = admit_then_revoke_bob(&mut pair, |_| {}).await.unwrap();
    let Received::Committed(outcome) = received else { panic!("not committed: {received:?}") };
    assert_eq!(outcome.kind, ReceiveKind::Revoked);
    assert!(!file_of(&b_dir, &bob_key).exists(), "Bob's store file is removed");
    assert_eq!(list_stores(&b_dir).unwrap(), vec![dave_key], "the listing holds no file for Bob's Device key");
    assert_eq!(std::fs::read(file_of(&b_dir, &dave_key)).unwrap(), dave_bytes, "Dave's file is byte-for-byte unchanged");
    assert!(
        keychain.items().contains(&(SERVICE.to_string(), hex::encode(bob_key.as_bytes()))),
        "Bob's keychain item is left in place"
    );
}

// Abnormal: the directory is read-only when the removal comes.
// verifies: LLR-amv4mb
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_removal_is_reported_and_the_persona_reloaded() {
    let (_guard, _keychain, dir) = fresh("failed-removal");
    let mut pair = pair(&dir).await;
    let b_dir = dir.join("b");
    let bob_key = device_key_of(&pair.b, &pair.bob);
    let refused = admit_then_revoke_bob(&mut pair, |_| make_read_only(&b_dir)).await;
    make_writable(&b_dir);
    match refused {
        Err(ReceiveError::Store(StoreFailure::Reloaded { failed })) => {
            assert_eq!(failed.step, StoreFileStep::Remove, "{failed}");
        }
        other => panic!("not reported as a failed removal, reloaded: {other:?}"),
    }
    assert!(file_of(&b_dir, &bob_key).is_file(), "Bob's file is still there");
    assert_eq!(held_store(&pair.b, &pair.bob), on_disk(&pair.b, &b_dir, &pair.bob), "Bob is reloaded from his file");
    assert_eq!(pair.b.node_for_test().store_data(&pair.bob).unwrap().orgs.len(), 1, "as the file holds him: admitted");
}
