#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The Person (SDD-j7k844): `propose_person` writes the Person's Device seed to its
//! keychain item, then its sealed `person-<hex>.store` file (LLR-3cysjk); every summary is
//! provisional (LLR-mas8vc); `open` lists Person files apart from Persona files (LLR-z7kbf9); no
//! endpoint is bound for the Person's key (LLR-g6kaa8).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rand::rngs::StdRng;
use rand::{CryptoRng, RngCore, SeedableRng};

use org_node::OrgNodeError;
use org_io::test_support::{FakeChain, Refusal};
use org_io::transport::TransportMode;
use org_io::view::PersonStatus;
use org_io::{CreateError, OpenError, OrgIo, StoreFailure};

mod common;
use common::handles::{create, fresh_keychain, handle, make_read_only, make_writable, names_in};

const SERVICE: &str = "ods-device-seed";

fn dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("org-io-person-{tag}-{}", std::process::id()));
    if dir.exists() {
        make_writable(&dir);
    }
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn person_file(dir: &Path, device_key: &org_node::DevicePublicKey) -> PathBuf {
    dir.join(format!("person-{}.store", hex::encode(device_key.as_bytes())))
}

// verifies: LLR-3cysjk
#[tokio::test]
async fn propose_person_writes_the_keychain_item_then_the_person_file() {
    let (_guard, keychain) = fresh_keychain();
    let dir = dir("writes");
    let io = OrgIo::open(&dir, TransportMode::Loopback).unwrap();
    let person = io.propose_person("Ada", "Lovelace").await.unwrap();
    assert_eq!(keychain.items(), vec![(SERVICE.to_string(), hex::encode(person.device_key.as_bytes()))], "one item, named for the Person's key");
    assert_eq!(names_in(&dir), vec![format!("person-{}.store", hex::encode(person.device_key.as_bytes()))], "one Person file, no Persona file");
    assert!(io.personas().await.is_empty(), "the Person is no Persona");
    drop(io);
    let reopened = OrgIo::open(&dir, TransportMode::Loopback).unwrap();
    assert_eq!(reopened.person().await, Some(person), "the file reopens as the Person proposed");
}

// Abnormal: org-node refuses the name; the keychain refuses the item; the file write fails
// after the item was written, which shows the item came first.
// verifies: LLR-3cysjk
#[tokio::test]
async fn a_refused_name_writes_nothing_and_a_failed_file_write_leaves_the_item() {
    let (_guard, keychain) = fresh_keychain();
    let dir = dir("refused");
    let io = OrgIo::open(&dir, TransportMode::Loopback).unwrap();
    // A name over org-members' 128-character bound (an empty name is a valid `Name`).
    match io.propose_person(&"a".repeat(129), "Lovelace").await {
        Err(CreateError::Node(OrgNodeError::InvalidField { field, .. })) => assert_eq!(field, "person.name"),
        other => panic!("not org-node's refusal of the name: {other:?}"),
    }
    assert!(keychain.items().is_empty() && names_in(&dir).is_empty(), "nothing written");

    keychain.refuse_as(Some(Refusal::Unavailable));
    let refused = io.propose_person("Ada", "Lovelace").await;
    keychain.refuse_as(None);
    assert!(matches!(refused, Err(CreateError::Keychain(_))), "{refused:?}");
    assert!(keychain.items().is_empty() && names_in(&dir).is_empty(), "no item and no file");
    assert_eq!(io.person().await, None, "no Person held");

    #[cfg(unix)]
    {
        make_read_only(&dir);
        let refused = io.propose_person("Ada", "Lovelace").await;
        make_writable(&dir);
        assert!(matches!(refused, Err(CreateError::Store(StoreFailure::NotKept { .. }))), "{refused:?}");
        assert_eq!(keychain.items().len(), 1, "the item was written before the file was tried, and stays");
        assert!(names_in(&dir).is_empty(), "no file");
        assert_eq!(io.person().await, None, "the Person is dropped");
    }
}

// Abnormal: one Person per device (owner ruling 2026-10-22). A second proposal is refused
// naming the Person held, before any draw or write: nothing is drawn from the handle's random
// source, no keychain item and no file is added, and `person()` still returns the first.
// verifies: LLR-3cysjk
#[tokio::test]
async fn a_second_person_is_refused_naming_the_one_held_and_writes_nothing() {
    let (_guard, keychain) = fresh_keychain();
    let dir = dir("second");
    let io = OrgIo::open(&dir, TransportMode::Loopback).unwrap();
    let first = io.propose_person("Ada", "Lovelace").await.unwrap();
    let draws = Arc::new(AtomicUsize::new(0));
    let io = io.with_rng_for_test(Box::new(CountedDraws { inner: StdRng::seed_from_u64(7), draws: draws.clone() }));
    let (items, names) = (keychain.items(), names_in(&dir));
    match io.propose_person("Grace", "Hopper").await {
        Err(CreateError::PersonAlreadyHeld { device_key }) => assert_eq!(device_key, first.device_key, "names the Person held"),
        other => panic!("a second Person was not refused: {other:?}"),
    }
    assert_eq!(draws.load(Ordering::SeqCst), 0, "nothing drawn");
    assert_eq!(keychain.items(), items, "no keychain item added");
    assert_eq!(names_in(&dir), names, "no file added");
    assert_eq!(io.person().await, Some(first), "the one Person is still the first");
}

/// A seeded source counting every draw made from it.
struct CountedDraws {
    inner: StdRng,
    draws: Arc<AtomicUsize>,
}

impl RngCore for CountedDraws {
    fn next_u32(&mut self) -> u32 {
        self.draws.fetch_add(1, Ordering::SeqCst);
        self.inner.next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.draws.fetch_add(1, Ordering::SeqCst);
        self.inner.next_u64()
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.draws.fetch_add(1, Ordering::SeqCst);
        self.inner.fill_bytes(dest);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.draws.fetch_add(1, Ordering::SeqCst);
        self.inner.try_fill_bytes(dest)
    }
}

impl CryptoRng for CountedDraws {}

// Abnormal: a data directory holding two Person files (a corruption: `propose_person` refuses a
// second) is refused by `open` naming both Device keys; no file is unsealed, changed or removed.
// verifies: LLR-z7kbf9
#[tokio::test]
async fn open_refuses_two_person_files_naming_both_and_touches_nothing() {
    let (_guard, keychain) = fresh_keychain();
    let (dir_a, dir_b) = (dir("two-a"), dir("two-b"));
    let ada = OrgIo::open(&dir_a, TransportMode::Loopback).unwrap().propose_person("Ada", "Lovelace").await.unwrap();
    let grace = OrgIo::open(&dir_b, TransportMode::Loopback).unwrap().propose_person("Grace", "Hopper").await.unwrap();
    std::fs::copy(person_file(&dir_b, &grace.device_key), person_file(&dir_a, &grace.device_key)).unwrap();
    let before: Vec<(String, Vec<u8>)> =
        names_in(&dir_a).into_iter().map(|name| (name.clone(), std::fs::read(dir_a.join(&name)).unwrap())).collect();
    let items = keychain.items();
    let mut expected = vec![ada.device_key, grace.device_key];
    expected.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    match OrgIo::open(&dir_a, TransportMode::Loopback) {
        Err(OpenError::MoreThanOnePerson { device_keys }) => assert_eq!(device_keys, expected, "both Person files named"),
        Err(other) => panic!("not refused as more than one Person: {other:?}"),
        Ok(_) => panic!("opened a directory holding two Persons"),
    }
    let after: Vec<(String, Vec<u8>)> =
        names_in(&dir_a).into_iter().map(|name| (name.clone(), std::fs::read(dir_a.join(&name)).unwrap())).collect();
    assert_eq!(after, before, "no file changed, added or removed");
    assert_eq!(keychain.items(), items, "no keychain item changed");
}

// verifies: LLR-mas8vc
#[tokio::test]
async fn every_person_summary_is_provisional() {
    let (_guard, _keychain) = fresh_keychain();
    let io = handle("person-provisional", "a", &FakeChain::new());
    let proposed = io.propose_person("Ada", "Lovelace").await.unwrap();
    assert_eq!((proposed.name.as_str(), proposed.surname.as_str()), ("Ada", "Lovelace"));
    assert_eq!(proposed.status, PersonStatus::Provisional);
    assert_eq!(io.person().await.unwrap().status, PersonStatus::Provisional);
    let view = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/view.rs")).unwrap();
    let status = &view[view.find("pub enum PersonStatus").unwrap()..];
    let body = &status[status.find('{').unwrap() + 1..status.find('}').unwrap()];
    let variants: Vec<&str> = body.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with("//")).collect();
    assert_eq!(variants, ["Provisional,"], "the one status this change returns");
    for file in ["write.rs", "verify.rs", "person.rs"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
        let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{file} is scanned: {error}"));
        assert!(!text.contains("person_hash"), "{file} writes or verifies a Person hash");
    }
}

// Abnormal: after a refused second proposal, and once its data directory is opened again from
// the Person file, the Person's summary still carries its own name, surname and keys, and is
// provisional.
// verifies: LLR-mas8vc
#[tokio::test]
async fn a_refused_proposal_and_a_reopened_person_stay_provisional_with_their_fields() {
    let (_guard, _keychain) = fresh_keychain();
    let dir = dir("provisional-reopened");
    let io = OrgIo::open(&dir, TransportMode::Loopback).unwrap();
    let proposed = io.propose_person("Ada", "Lovelace").await.unwrap();
    assert!(io.propose_person("Grace", "Hopper").await.is_err(), "a second Person is refused");
    assert_eq!(io.person().await, Some(proposed.clone()), "after the refusal");
    drop(io);
    let reopened = OrgIo::open(&dir, TransportMode::Loopback).unwrap().person().await.expect("the Person, reopened");
    assert_eq!((reopened.name.as_str(), reopened.surname.as_str()), ("Ada", "Lovelace"));
    assert_eq!((reopened.device_key, reopened.person_key), (proposed.device_key, proposed.person_key));
    assert_eq!(reopened.status, PersonStatus::Provisional);
}

// verifies: LLR-z7kbf9
#[tokio::test]
async fn open_lists_person_files_apart_and_refuses_one_whose_item_is_missing() {
    let (_guard, keychain) = fresh_keychain();
    let dir = dir("open");
    let io = OrgIo::open(&dir, TransportMode::Loopback).unwrap();
    let person = io.propose_person("Ada", "Lovelace").await.unwrap();
    create(&io, "alice").await;
    drop(io);
    let reopened = OrgIo::open(&dir, TransportMode::Loopback).unwrap();
    assert_eq!(reopened.personas().await.len(), 1, "the Persona listing ignores the Person file");
    assert_eq!(reopened.person().await.unwrap().device_key, person.device_key, "the Person listing ignores the Persona file");
    drop(reopened);

    let persona_file = names_in(&dir).into_iter().find(|name| name.starts_with("persona-")).unwrap();
    std::fs::remove_file(dir.join(persona_file)).unwrap();
    let file = person_file(&dir, &person.device_key);
    let bytes = std::fs::read(&file).unwrap();
    let empty = org_io::test_support::MemoryKeychain::default();
    empty.install();
    match OrgIo::open(&dir, TransportMode::Loopback) {
        Err(OpenError::KeychainItemMissing { device_key, path }) => {
            assert_eq!((device_key, path), (person.device_key, file.clone()));
        }
        Err(other) => panic!("not refused as a missing item: {other:?}"),
        Ok(_) => panic!("opened without the Person's item"),
    }
    assert_eq!(std::fs::read(&file).unwrap(), bytes, "the file is untouched");
    assert!(empty.items().is_empty(), "nothing created in its place");
    keychain.install();
}

// verifies: LLR-g6kaa8
#[tokio::test(flavor = "multi_thread")]
async fn no_endpoint_is_bound_for_the_persons_device_key() {
    let (_guard, _keychain) = fresh_keychain();
    let io = handle("person-no-endpoint", "a", &FakeChain::new());
    let person = io.propose_person("Ada", "Lovelace").await.unwrap();
    let alice = create(&io, "alice").await;
    io.start_receiving().await.unwrap();
    assert_eq!(io.bound_endpoints_for_test().await, 1, "one endpoint, the Persona's");
    let bound = io.bound_endpoint_for_test(&alice).await.unwrap();
    assert_ne!(bound.inner().id().as_bytes(), person.device_key.as_bytes(), "not the Person's key");
}

// Abnormal: a message addressed to the Person's key
// is never accepted by this node. The node receives (its Persona's endpoint
// bound) before the Person is proposed; a Device dials the Person's key at
// every socket the node listens on, and no connection completes: no
// endpoint holds that identity, and the receive step takes no body.
// verifies: LLR-g6kaa8
#[tokio::test(flavor = "multi_thread")]
async fn a_message_addressed_to_the_persons_key_is_never_accepted() {
    let (_guard, _keychain) = fresh_keychain();
    let io = handle("person-not-accepted", "a", &FakeChain::new());
    let alice = create(&io, "alice").await;
    io.start_receiving().await.unwrap();
    let person = io.propose_person("Ada", "Lovelace").await.unwrap();
    assert_eq!(io.bound_endpoints_for_test().await, 1, "a Person proposed while receiving binds no endpoint");
    let bound = io.bound_endpoint_for_test(&alice).await.unwrap();
    let person_id = iroh::EndpointId::from_bytes(person.device_key.as_bytes()).unwrap();
    let to_person = iroh::EndpointAddr::from_parts(person_id, bound.bound_sockets().into_iter().map(iroh::TransportAddr::Ip));
    let sender = common::handles::keep_device_seed([0x6e; 32]);
    let (dialled, taken) = tokio::join!(
        tokio::time::timeout(std::time::Duration::from_secs(20), org_io::test_support::dial_direct_as(&sender, to_person, b"for the Person")),
        tokio::time::timeout(std::time::Duration::from_secs(3), io.next_received())
    );
    assert!(!matches!(dialled, Ok(Ok(_))), "no connection to the Person's key completes: {dialled:?}");
    // A refused handshake may surface as one failed inbound connection
    // (`ReceiveError::Connection`); no body is ever reported.
    assert!(
        matches!(taken, Err(_) | Ok(Err(org_io::ReceiveError::Connection(_)))),
        "the receive step takes no body: {taken:?}"
    );
}
