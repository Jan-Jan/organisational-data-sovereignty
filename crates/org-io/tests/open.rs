#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `OrgIo::open` (LLR-rgdx22, LLR-uzax55): it creates the data directory with any missing parents,
//! opens every Persona's store file `persona-<hex Device key>.store` inside
//! it through that Persona's keychain item, and returns the unconfigured
//! handle in the transport mode it was given; it refuses a directory it
//! cannot create, naming it. Each Persona's store is sealed under its device
//! secret; `open` unseals every store file through its keychain item and
//! refuses with a typed `OpenError`. The store files' own tests are in
//! `store_files.rs`.

use std::path::PathBuf;

use org_io::test_support::{MemoryKeychain, TestClock};
use org_io::transport::TransportMode;
use org_io::{OpenError, OrgIo, Verified};

mod common;
use common::handles::{device_key_of, file_of, item_held, keep_device_seed, names_in, removed_file_of, store_dir};
use common::journey;

/// A fresh scratch directory for test `tag`.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("org-io-open-{tag}-{}", std::process::id()));
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn refusal(opened: Result<OrgIo, OpenError>) -> String {
    match opened {
        Ok(_) => panic!("expected a refusal"),
        Err(error) => error.to_string(),
    }
}

// verifies: LLR-rgdx22
#[tokio::test]
async fn open_creates_the_data_dir_and_opens_the_store_inside_it() {
    // The Persona's device secret goes to the keychain, here org-io's
    // in-memory one.
    org_io::test_support::MemoryKeychain::shared().install();
    let data_dir = scratch("normal").join("missing").join("parents");
    let io = OrgIo::open(&data_dir, TransportMode::Loopback).unwrap();
    assert!(io.personas().await.is_empty(), "an empty directory: no Persona");
    let persona_id = common::handles::create_named(&io, "alice", "Alice", "Doe").await;
    let device_key = device_key_of(&io, &persona_id).await;
    drop(io);
    let store = data_dir.join(format!("persona-{}.store", hex::encode(device_key.as_bytes())));
    assert!(store.is_file(), "the Persona's store is written to its file in the data directory");
    // Reopened, the store unseals through the keychain item: the same
    // Persona.
    let reopened = OrgIo::open(&data_dir, TransportMode::Loopback).unwrap();
    assert_eq!(device_key_of(&reopened, &persona_id).await, device_key);
}

// Normal: the handle runs in the transport mode `open` was given: the
// endpoints are org-io's, so the mode is the handle's endpoints'.
// verifies: LLR-rgdx22
#[cfg(feature = "test-support")]
#[test]
fn open_sets_the_transport_mode_it_was_given() {
    let networked = OrgIo::open(&scratch("networked"), TransportMode::Networked).unwrap();
    assert_eq!(networked.transport_mode_for_test(), TransportMode::Networked);
    let loopback = OrgIo::open(&scratch("loopback"), TransportMode::Loopback).unwrap();
    assert_eq!(loopback.transport_mode_for_test(), TransportMode::Loopback);
}

// verifies: LLR-rgdx22
#[test]
fn open_refuses_a_data_dir_it_cannot_create_naming_it() {
    let file = scratch("not-a-dir").join("plain-file");
    std::fs::write(&file, b"not a directory").unwrap();
    let data_dir = file.join("sub");
    let error = refusal(OrgIo::open(&data_dir, TransportMode::Loopback));
    assert!(error.starts_with(&format!("create data_dir {}", data_dir.display())), "{error}");
}

// A leftover `removed-<hex>.store` (its item still there) is finished at
// open — item deleted, file removed, nothing unsealed — and a removal kept
// past its deadline is deleted before the handle is returned; a Persona with
// no removal is untouched.
// verifies: LLR-upqj5g
#[tokio::test(flavor = "multi_thread")]
async fn open_finishes_removed_files_and_deletes_expired_or_empty_removals() {
    MemoryKeychain::shared().install();
    let tag = "open-finishes";
    let dir = store_dir(tag, "x");
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let leftover = keep_device_seed([0x71; 32]);
    let leftover_file = removed_file_of(&dir, &leftover);
    std::fs::write(&leftover_file, b"sealed under a secret about to go").unwrap();
    OrgIo::open(&dir, TransportMode::Loopback).unwrap();
    assert!(!leftover_file.exists(), "the leftover file removed");
    assert!(!item_held(MemoryKeychain::shared(), &leftover), "its item deleted");

    let verified_at = 6_000_000u64;
    let clock = TestClock::at(verified_at);
    let admin = journey::admin(tag).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    // Dead addresses for the Admin: Bob's delivery reaches no one, so its
    // acknowledgement stays kept until the deadline.
    let (dead, listening) = journey::soon_dead(admin.device_key).await;
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, dead).await;
    listening.close().await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let bob_device = bob.device_key;
    drop(bob);
    let bob_dir = store_dir(tag, "b");
    let bob_file = file_of(&bob_dir, &bob_device);
    let before = OrgIo::open_with_clock_for_test(&bob_dir, TransportMode::Loopback, clock.clone()).unwrap();
    assert!(bob_file.exists() && item_held(MemoryKeychain::shared(), &bob_device), "before the deadline the removal is kept");
    assert!(before.acknowledgement_report(admin.org).await.is_some(), "and held");
    drop(before);
    clock.set(verified_at + 604_800);
    let after = OrgIo::open_with_clock_for_test(&bob_dir, TransportMode::Loopback, clock.clone()).unwrap();
    let names = names_in(&bob_dir);
    assert!(!names.iter().any(|name| name.starts_with("persona-") || name.starts_with("removed-")), "{names:?}");
    assert!(!item_held(MemoryKeychain::shared(), &bob_device), "Bob's item deleted");
    assert!(after.acknowledgement_report(admin.org).await.is_none(), "the handle holds none");
    let reopened_admin = OrgIo::open(&store_dir(tag, "a"), TransportMode::Loopback).unwrap();
    assert_eq!(reopened_admin.organisations().await.len(), 1, "a Persona with no removal is untouched");
    assert!(item_held(MemoryKeychain::shared(), &admin.device_key), "nor its item");
}

// Abnormal: a `persona-<hex>.store` whose item is missing is still refused,
// and its bytes are left as they were.
// verifies: LLR-upqj5g
#[test]
fn a_persona_file_without_its_item_is_still_refused_and_untouched() {
    MemoryKeychain::shared().install();
    let dir = store_dir("open-item-missing", "x");
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let device_key = org_node::test_fixtures::device_key(0x72);
    let file = file_of(&dir, &device_key);
    std::fs::write(&file, b"a store whose item is gone").unwrap();
    let refused = OrgIo::open(&dir, TransportMode::Loopback);
    assert!(matches!(refused, Err(OpenError::KeychainItemMissing { .. })), "{:?}", refused.err());
    assert_eq!(std::fs::read(&file).unwrap(), b"a store whose item is gone", "untouched");
    assert!(removed_file_of(&dir, &device_key).symlink_metadata().is_err(), "not marked removed");
}
