#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `OrgIo::open` (LLR-rgdx22, clause added 2026-10-08, review round 3,
//! finding 3): it creates the data directory with any missing parents,
//! opens the Persona store `persona_store.bin` inside it under the
//! passphrase, and returns the unconfigured handle; it refuses a directory
//! it cannot create, naming it, and a store that does not open under the
//! passphrase, opening no other store in its place.

use std::path::PathBuf;

use org_io::node::transport::TransportMode;
use org_io::node::{Handle, Name, Surname};
use org_io::OrgIo;
use rand::rngs::OsRng;

/// A fresh scratch directory for test `tag`.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("org-io-open-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn refusal(opened: Result<OrgIo, String>) -> String {
    match opened {
        Ok(_) => panic!("expected a refusal"),
        Err(error) => error,
    }
}

// verifies: LLR-rgdx22
#[test]
fn open_creates_the_data_dir_and_opens_the_store_inside_it() {
    let data_dir = scratch("normal").join("missing").join("parents");
    let mut io = OrgIo::open(&data_dir, "pw", TransportMode::Loopback).unwrap();
    io.node_mut()
        .create_persona(&mut OsRng, Handle::parse("alice").unwrap(), Name::parse("Alice").unwrap(), Surname::parse("Doe").unwrap())
        .unwrap();
    drop(io);
    assert!(data_dir.join("persona_store.bin").is_file(), "the store is persona_store.bin in the data directory");
    let again = OrgIo::open(&data_dir, "pw", TransportMode::Loopback).unwrap();
    assert_eq!(again.view().personas().len(), 1, "reopened, it is the same store");
}

// Normal (LLR-rgdx22, review round 4, finding 1): the handle's service runs
// in the transport mode `open` was given, not org-node's Loopback default.
// verifies: LLR-rgdx22
#[cfg(feature = "test-support")]
#[test]
fn open_sets_the_transport_mode_it_was_given() {
    let networked = OrgIo::open(&scratch("networked"), "pw", TransportMode::Networked).unwrap();
    assert_eq!(networked.node_for_test().transport_mode(), TransportMode::Networked);
    let loopback = OrgIo::open(&scratch("loopback"), "pw", TransportMode::Loopback).unwrap();
    assert_eq!(loopback.node_for_test().transport_mode(), TransportMode::Loopback);
}

// verifies: LLR-rgdx22
#[test]
fn open_refuses_a_data_dir_it_cannot_create_naming_it() {
    let file = scratch("not-a-dir").join("plain-file");
    std::fs::write(&file, b"not a directory").unwrap();
    let data_dir = file.join("sub");
    let error = refusal(OrgIo::open(&data_dir, "pw", TransportMode::Loopback));
    assert!(error.starts_with(&format!("create data_dir {}", data_dir.display())), "{error}");
}

// verifies: LLR-rgdx22
#[test]
fn open_refuses_a_store_under_another_passphrase_and_opens_no_other() {
    let data_dir = scratch("passphrase");
    let mut io = OrgIo::open(&data_dir, "right", TransportMode::Loopback).unwrap();
    io.node_mut()
        .create_persona(&mut OsRng, Handle::parse("alice").unwrap(), Name::parse("Alice").unwrap(), Surname::parse("Doe").unwrap())
        .unwrap();
    drop(io);
    let store = std::fs::read(data_dir.join("persona_store.bin")).unwrap();
    let error = refusal(OrgIo::open(&data_dir, "wrong", TransportMode::Loopback));
    assert!(error.starts_with("open store: "), "{error}");
    assert_eq!(std::fs::read(data_dir.join("persona_store.bin")).unwrap(), store, "the store is left as it was");
}
