#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `OrgIo::open` (LLR-rgdx22, clause added 2026-10-08, review round 3,
//! finding 3): it creates the data directory with any missing parents,
//! opens every Persona's store file `persona-<hex Device key>.store` inside
//! it through that Persona's keychain item, and returns the unconfigured
//! handle in the transport mode it was given; it refuses a directory it
//! cannot create, naming it.
//!
//! *Corrected 2026-10-15 (S4 review round 2, finding-13):* the paragraph
//! above named the retired `persona_store.bin` and its passphrase (as the
//! dated notes below record, both went at S4's tasks T3 and T9); it now
//! states what `open` does on this tree.
//!
//! *Narrowed 2026-10-08 (S4, task T3 of
//! docs/plans/2026-10-06-org-io-transport.md; LLR-rgdx22's dated note):* no
//! passphrase. Each Persona's store is sealed under its device secret and
//! written to `persona-<hex Device key>.store`; until the keychain holds the
//! secrets (tasks T8, T9), `open` starts a service with no Persona over an
//! empty directory and refuses one holding a store file, naming it. The
//! passphrase test (`open_refuses_a_store_under_another_passphrase_and_opens_no_other`)
//! is deleted with the behaviour; the other three are adapted, annotations
//! unchanged.
//!
//! *Adapted 2026-10-13 (S4, task T9):* `open` unseals every store file
//! through its keychain item (LLR-uzax55) and refuses with a typed
//! `OpenError`; the reopened store is the one written, not refused. The
//! store files' own tests are in `store_files.rs`.

use std::path::PathBuf;

use org_node::{Handle, Name, Surname};
use org_io::transport::TransportMode;
use org_io::{OpenError, OrgIo};
use rand::rngs::OsRng;

/// A fresh scratch directory for test `tag`.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("org-io-open-{tag}-{}", std::process::id()));
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
    // *Adapted 2026-10-13 (S4, task T8):* the Persona's device secret goes to
    // the keychain, here org-io's in-memory one. *Adapted 2026-10-13 (S4,
    // task T11):* the handle's methods are `async`.
    org_io::test_support::MemoryKeychain::shared().install();
    let data_dir = scratch("normal").join("missing").join("parents");
    let io = OrgIo::open(&data_dir, TransportMode::Loopback).unwrap();
    assert!(io.personas().await.is_empty(), "an empty directory: no Persona");
    let persona_id = io
        .create_persona(&mut OsRng, Handle::parse("alice").unwrap(), Name::parse("Alice").unwrap(), Surname::parse("Doe").unwrap())
        .await
        .unwrap();
    let (_, device_key) = io.persona_public_keys(&persona_id).await.unwrap();
    drop(io);
    let store = data_dir.join(format!("persona-{}.store", hex::encode(device_key.as_bytes())));
    assert!(store.is_file(), "the Persona's store is written to its file in the data directory");
    // *Adapted 2026-10-13 (S4, task T9):* reopened, the store unseals through
    // the keychain item: the same Persona.
    let reopened = OrgIo::open(&data_dir, TransportMode::Loopback).unwrap();
    assert_eq!(reopened.persona_public_keys(&persona_id).await.unwrap().1, device_key);
}

// Normal (LLR-rgdx22, review round 4, finding 1): the handle's service runs
// in the transport mode `open` was given, not org-node's Loopback default.
// *Adapted 2026-10-13 (S4, task T10):* the endpoints are org-io's, so the
// mode is the handle's endpoints', not org-node's service's.
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
