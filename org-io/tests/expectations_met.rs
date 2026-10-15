#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The six expectations org-io declared on org-node, shown to hold through
//! org-io (S4, task T19 of docs/plans/2026-10-06-org-io-transport.md; the
//! owner's ruling 1 of 2026-10-14). Each REQ below was `expects: org-node`
//! and is met by a requirement org-node exports; under guardrails 0.5.1 a
//! met expectation is an ordinary REQ again and owes a test here, at org-io's
//! interface (the `OrgIo` handle, its store files and its endpoints):
//!
//! - REQ-x6kdse: a Persona's store opens from its sealed bytes and its
//!   keychain secret, and every change reaches its file whole;
//! - REQ-qza96g: a store file opens only under its own Persona's secret, and
//!   keeps the Device public key, never the seed;
//! - REQ-v6r2gd: the device secret is persisted nowhere but the keychain,
//!   and renders no bytes;
//! - REQ-tzr3dk: one receive path commits a first admission, an update and
//!   the node's own revocation, and a refused check reads and writes nothing;
//! - REQ-jxyh9c: each recipient of a commit gets the body org-node built for
//!   it, the removed Device included;
//! - REQ-txkm94: the Invite and reply Blobs, the outstanding identifier and
//!   the declared expectation reach the store files, and are settled there.
//!
//! The behaviour existed before these tests, so each carries a recorded
//! mutation check of the org-io path it depends on in place of a red-first
//! run (the plan's T19 report).
//!
//! Every test runs over fresh directories and a fresh in-memory keychain, one
//! test at a time under `fresh_keychain`'s lock: keyring's default
//! credential builder is process-global.

use std::path::{Path, PathBuf};
use std::time::Duration;

use iroh::address_lookup::MemoryLookup;
use org_node::store::{PersonaStore, SealedStore};
use org_node::wire::{decode_message, WireMessage};
use org_node::{DevicePublicKey, Epoch, MemberId, OrgId, OrgNodeError, PersonaId, ReceiveKind};
use org_io::test_support::{FakeChain, MemoryKeychain, Refusal};
use org_io::transport::{BindPlan, OrgEndpoint, TransportMode};
use org_io::{ExchangeError, KeychainError, OpenError, OrgIo, ReceiveError, Received};
use rand::rngs::OsRng;
use tokio::sync::mpsc;

mod common;
use common::handles::{
    at, bind_seed, committed, create, file_of, forwarding, fresh_keychain, handle, held_store, joiner, joiner_of,
    keep_device_seed, receiving_addr, reopened_service, store_bytes, NET,
};

const SERVICE: &str = "ods-device-seed";

/// A fresh, empty data directory for `party` of test `tag`.
fn fresh_dir(tag: &str, party: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("org-io-expectations-{tag}-{party}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// `OrgIo::open` over `dir`, Loopback, with `chain` substituted.
fn open(dir: &Path, chain: &FakeChain) -> OrgIo {
    OrgIo::open_for_test(dir, TransportMode::Loopback, chain).unwrap()
}

fn open_refused(dir: &Path) -> OpenError {
    match OrgIo::open(dir, TransportMode::Loopback) {
        Ok(_) => panic!("opened, and it should have been refused"),
        Err(error) => error,
    }
}

async fn device_key_of(io: &OrgIo, persona_id: &PersonaId) -> DevicePublicKey {
    io.persona_public_keys(persona_id).await.unwrap().1
}

/// The 32-byte Device seed the keychain holds for `device_key`.
fn seed_of(device_key: &DevicePublicKey) -> [u8; 32] {
    org_io::test_support::keychain::with_endpoint_seed(device_key, |seed| *seed).unwrap()
}

/// Whether `haystack` contains `needle` anywhere.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

/// Every file's bytes in `dir`, by name.
fn files_in(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<(String, Vec<u8>)> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            (path.file_name().unwrap().to_string_lossy().to_string(), std::fs::read(&path).unwrap())
        })
        .collect();
    files.sort();
    files
}

// ---- REQ-x6kdse: open from sealed bytes and the secret; every change whole ---

// Each store-changing operation of the handle — creating the Persona,
// founding, declaring an expectation, issuing an Invite — leaves a file that
// a fresh `OrgIo::open` unseals with the keychain item into exactly the
// store the handle holds: org-io wrote the complete sealed store each time.
// verifies: REQ-x6kdse
#[tokio::test(flavor = "multi_thread")]
async fn every_store_change_reaches_the_file_whole_and_reopens_as_held() {
    let (_guard, _keychain) = fresh_keychain();
    let dir = fresh_dir("whole", "a");
    let chain = FakeChain::new();
    let io = open(&dir, &chain);
    let alice = create(&io, "alice").await;
    let reopens_as_held = |step: &str| {
        let reopened = open(&dir, &chain);
        assert_eq!(held_store(&reopened, &alice), held_store(&io, &alice), "after {step}: the file holds the whole store");
    };
    reopens_as_held("create_persona");
    let org = io.found_organisation(&mut OsRng, &alice).await.unwrap();
    reopens_as_held("found_organisation");
    let carol = create(&io, "carol").await;
    io.expect_admission(&mut OsRng, &carol, OrgId::new([0x07; 20])).await.unwrap();
    assert_eq!(held_store(&open(&dir, &chain), &carol), held_store(&io, &carol), "after expect_admission");
    io.issue_invite(&mut OsRng, org, "Acme", "Bob Builder").await.unwrap();
    reopens_as_held("issue_invite");
    io.admit_member(&mut OsRng, org, &joiner(0x61, "bob")).await.unwrap();
    reopens_as_held("admit_member (a kept provisional update)");
    assert_eq!(files_in(&dir).len(), 2, "one store per Persona, and nothing else");
}

// Abnormal: a store file whose bytes org-node cannot unseal — empty,
// truncated, one byte flipped, or not a store at all — refuses the open,
// naming the file and its Device key, and the file is left as it was.
// verifies: REQ-x6kdse
#[tokio::test(flavor = "multi_thread")]
async fn a_store_file_org_node_cannot_unseal_is_refused_naming_it_and_left_untouched() {
    let (_guard, _keychain) = fresh_keychain();
    let dir = fresh_dir("unsealable", "a");
    let io = open(&dir, &FakeChain::new());
    let alice = create(&io, "alice").await;
    let device_key = device_key_of(&io, &alice).await;
    let file = file_of(&dir, &device_key);
    let sealed = std::fs::read(&file).unwrap();
    drop(io);
    let mut flipped = sealed.clone();
    let middle = flipped.len() / 2;
    flipped[middle] ^= 0x01;
    for (case, bytes) in [
        ("empty", Vec::new()),
        ("truncated", sealed[..sealed.len() / 2].to_vec()),
        ("one byte flipped", flipped),
        ("not a store", b"not a sealed store".to_vec()),
    ] {
        std::fs::write(&file, &bytes).unwrap();
        match open_refused(&dir) {
            OpenError::Unseal { device_key: named, path, .. } => {
                assert_eq!((named, path), (device_key, file.clone()), "{case}: the refusal names the file and its key");
            }
            other => panic!("{case}: not org-node's unseal refusal: {other:?}"),
        }
        assert_eq!(std::fs::read(&file).unwrap(), bytes, "{case}: the file is left as it was");
    }
}

// ---- REQ-qza96g: sealed under the Persona's own secret; no seed kept ---------

// A store file opens under its own Persona's keychain item, keeps that
// Persona's Device public key, and its unsealed contents hold the Device
// seed in no form.
// verifies: REQ-qza96g
#[tokio::test(flavor = "multi_thread")]
async fn a_store_file_opens_under_its_own_item_and_keeps_the_device_key_not_the_seed() {
    let (_guard, _keychain) = fresh_keychain();
    let dir = fresh_dir("own-item", "a");
    let chain = FakeChain::new();
    let io = open(&dir, &chain);
    let alice = create(&io, "alice").await;
    io.found_organisation(&mut OsRng, &alice).await.unwrap();
    let device_key = device_key_of(&io, &alice).await;
    let seed = seed_of(&device_key);

    let reopened = open(&dir, &chain);
    assert_eq!(device_key_of(&reopened, &alice).await, device_key, "the store keeps the Device public key");
    let sealed = SealedStore::from_bytes(std::fs::read(file_of(&dir, &device_key)).unwrap());
    let store = PersonaStore::unseal(&sealed, device_key, io.device_secret_for_test(&device_key).unwrap()).unwrap();
    assert_eq!(store.device_key(), device_key);
    let contents = format!("{:?}", store.data());
    assert!(!contents.contains(&hex::encode(seed)), "the unsealed store holds no seed, as hex");
    assert!(!contents.contains(&format!("{:?}", seed)), "the unsealed store holds no seed, as bytes");
    assert!(!contains(sealed.as_bytes(), &seed), "nor does the sealed file");
}

// Abnormal: another Persona's sealed store under this Persona's file name
// is refused by the unseal (the store key is this Persona's alone); an item
// holding another Device's valid seed is refused before any unseal; neither
// open writes anything.
// verifies: REQ-qza96g
#[tokio::test(flavor = "multi_thread")]
async fn a_store_file_opens_under_no_other_personas_secret() {
    let (_guard, keychain) = fresh_keychain();
    let dir = fresh_dir("other-secret", "a");
    let io = open(&dir, &FakeChain::new());
    let alice = create(&io, "alice").await;
    let bob = create(&io, "bob").await;
    let (alice_key, bob_key) = (device_key_of(&io, &alice).await, device_key_of(&io, &bob).await);
    let alice_file = file_of(&dir, &alice_key);
    let alice_sealed = std::fs::read(&alice_file).unwrap();
    drop(io);

    std::fs::write(&alice_file, std::fs::read(file_of(&dir, &bob_key)).unwrap()).unwrap();
    match open_refused(&dir) {
        OpenError::Unseal { device_key, .. } => assert_eq!(device_key, alice_key, "Bob's store does not open as Alice's"),
        other => panic!("not org-node's unseal refusal: {other:?}"),
    }

    std::fs::write(&alice_file, &alice_sealed).unwrap();
    keychain.insert(SERVICE, &hex::encode(alice_key.as_bytes()), seed_of(&bob_key).to_vec());
    match open_refused(&dir) {
        OpenError::Keychain { error: KeychainError::KeyMismatch { device_key }, .. } => assert_eq!(device_key, alice_key),
        other => panic!("not the keychain's key mismatch: {other:?}"),
    }
    assert_eq!(std::fs::read(&alice_file).unwrap(), alice_sealed, "nothing written");
}

// ---- REQ-v6r2gd: the secret is opaque and persisted only in the keychain -----

// After a Persona is created and its store changed several times, no file
// the handle wrote holds the Device seed; the keychain holds exactly one
// item per Persona; and the secret org-io reads back renders no bytes.
// verifies: REQ-v6r2gd
#[tokio::test(flavor = "multi_thread")]
async fn the_device_seed_is_in_the_keychain_alone_and_renders_no_bytes() {
    let (_guard, keychain) = fresh_keychain();
    let dir = fresh_dir("seed-nowhere", "a");
    let io = open(&dir, &FakeChain::new());
    let alice = create(&io, "alice").await;
    let org = io.found_organisation(&mut OsRng, &alice).await.unwrap();
    io.issue_invite(&mut OsRng, org, "Acme", "Bob Builder").await.unwrap();
    io.admit_member(&mut OsRng, org, &joiner(0x61, "bob")).await.unwrap();
    let device_key = device_key_of(&io, &alice).await;
    let seed = seed_of(&device_key);

    for (name, bytes) in files_in(&dir) {
        assert!(!contains(&bytes, &seed), "{name} holds the Device seed");
    }
    assert_eq!(keychain.items(), vec![(SERVICE.to_string(), hex::encode(device_key.as_bytes()))], "one item, by Device key");
    let rendered = format!("{:?}", io.device_secret_for_test(&device_key).unwrap());
    assert!(!rendered.contains(&hex::encode(seed)) && !rendered.contains(&format!("{:?}", seed)), "{rendered}");
}

// Abnormal: with the keychain's item gone, malformed, or the keychain
// locked, the store file alone opens nothing — nothing on disk stands in for
// the secret — and the file is left as it was.
// verifies: REQ-v6r2gd
#[tokio::test(flavor = "multi_thread")]
async fn without_its_keychain_item_a_store_file_opens_nothing() {
    let (_guard, keychain) = fresh_keychain();
    let dir = fresh_dir("no-item", "a");
    let io = open(&dir, &FakeChain::new());
    let alice = create(&io, "alice").await;
    let device_key = device_key_of(&io, &alice).await;
    let file = file_of(&dir, &device_key);
    let sealed = std::fs::read(&file).unwrap();
    drop(io);

    keychain.refuse_as(Some(Refusal::Locked));
    let locked = open_refused(&dir);
    keychain.refuse_as(None);
    assert!(matches!(locked, OpenError::Keychain { error: KeychainError::Locked { .. }, .. }), "{locked:?}");

    keychain.insert(SERVICE, &hex::encode(device_key.as_bytes()), vec![0x07; 31]);
    let malformed = open_refused(&dir);
    assert!(matches!(malformed, OpenError::Keychain { error: KeychainError::Malformed { .. }, .. }), "{malformed:?}");

    MemoryKeychain::default().install();
    match open_refused(&dir) {
        OpenError::KeychainItemMissing { device_key: named, path } => assert_eq!((named, path), (device_key, file.clone())),
        other => panic!("not a missing item: {other:?}"),
    }
    assert_eq!(std::fs::read(&file).unwrap(), sealed, "the file is left as it was");
}

// ---- REQ-tzr3dk: one receive path ---------------------------------------------

/// A has founded an Organisation over `chain`; B has a Persona, Bob, that
/// declared the admission, and accepts on its endpoint.
struct Pair {
    tag: &'static str,
    a: OrgIo,
    b: OrgIo,
    chain_b: FakeChain,
    org: OrgId,
    bob: PersonaId,
    addr_b: iroh::EndpointAddr,
}

async fn pair(tag: &'static str, b_declares: bool) -> Pair {
    let chain = FakeChain::new();
    let a = handle(tag, "a", &chain);
    let alice = create(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    let chain_b = FakeChain::sharing_slots_with(&chain);
    let b = handle(tag, "b", &chain_b);
    let bob = create(&b, "bob").await;
    if b_declares {
        b.expect_admission(&mut OsRng, &bob, org).await.unwrap();
    }
    let addr_b = receiving_addr(&b).await;
    Pair { tag, a, b, chain_b, org, bob, addr_b }
}

/// B's result while A submits `update` with B's address.
async fn sent_to_b(pair: &Pair, update: org_io::BuiltUpdate) -> Result<Received, ReceiveError> {
    let (received, sent) = tokio::join!(
        async { tokio::time::timeout(NET, pair.b.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            pair.a.submit_commit_send(&mut OsRng, &update, at(pair.addr_b.clone())).await
        }
    );
    sent.unwrap();
    received
}

/// The epoch of each record B's store files hold.
fn epochs_on_disk(pair: &Pair) -> Vec<Epoch> {
    reopened_service(pair.tag, "b", &pair.b).list_orgs().iter().map(|record| record.epoch).collect()
}

async fn member_id_of(io: &OrgIo, org: OrgId, member_key: org_node::PersonPublicKey) -> MemberId {
    io.organisation(org).await.unwrap().members.iter().find(|member| member.member_key == member_key).unwrap().id
}

// One path, `next_received`, commits the three kinds in turn — the first
// admission B declared, an update to the Organisation it holds, and its own
// revocation — reading the chain once for each, and each commit reaches B's
// store files before the outcome returns.
// verifies: REQ-tzr3dk
#[tokio::test(flavor = "multi_thread")]
async fn one_receive_path_commits_a_first_admission_an_update_and_the_own_revocation() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("one-path", true).await;
    let bob = joiner_of(&pair.b, &pair.bob);

    let admission = pair.a.admit_member(&mut OsRng, pair.org, &bob).await.unwrap();
    let admitted = committed(sent_to_b(&pair, admission).await);
    assert_eq!((admitted.org_id, admitted.kind, admitted.epoch), (pair.org, ReceiveKind::Admitted, Epoch::new(2)));
    assert_eq!(pair.chain_b.reads(), 1, "one read for the admission");
    assert_eq!(epochs_on_disk(&pair), vec![Epoch::new(2)], "the admission is in B's file");

    let update = pair.a.admit_member(&mut OsRng, pair.org, &joiner(0x71, "carol")).await.unwrap();
    let updated = committed(sent_to_b(&pair, update).await);
    assert_eq!((updated.kind, updated.epoch), (ReceiveKind::Updated, Epoch::new(3)));
    assert_eq!(pair.chain_b.reads(), 2, "one read for the update");
    assert_eq!(epochs_on_disk(&pair), vec![Epoch::new(3)], "the update is in B's file");

    let bob_member = member_id_of(&pair.a, pair.org, bob.member_key).await;
    let revocation = pair.a.revoke_member(&mut OsRng, pair.org, bob_member).await.unwrap();
    let revoked = committed(sent_to_b(&pair, revocation).await);
    assert_eq!((revoked.org_id, revoked.kind), (pair.org, ReceiveKind::Revoked));
    assert_eq!(pair.chain_b.reads(), 3, "one read for the revocation");
    let on_disk = reopened_service(pair.tag, "b", &pair.b);
    assert!(on_disk.list_orgs().is_empty() && on_disk.list_personas().is_empty(), "the revocation is in B's files");
}

// Abnormal: a body org-node's check refuses — bytes that do not decode, and
// a first admission B never declared — is returned as org-node's refusal,
// with no chain read and B's store files unchanged.
// verifies: REQ-tzr3dk
#[tokio::test(flavor = "multi_thread")]
async fn a_body_the_check_refuses_reads_no_chain_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("check-refuses", false).await;
    let files_before = store_bytes(pair.tag, "b");

    let stranger = bind_seed([0x5c; 32]).await;
    let (refused, ()) = tokio::join!(async { tokio::time::timeout(NET, pair.b.next_received(&mut OsRng)).await.unwrap() }, async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        tokio::time::timeout(NET, stranger.send_body(pair.addr_b.clone(), b"not a wire message")).await.unwrap().unwrap();
    });
    assert!(matches!(refused, Err(ReceiveError::Refused(OrgNodeError::MalformedMessage))), "{refused:?}");

    let bob = joiner_of(&pair.b, &pair.bob);
    let admission = pair.a.admit_member(&mut OsRng, pair.org, &bob).await.unwrap();
    let refused = sent_to_b(&pair, admission).await;
    assert!(
        matches!(refused, Err(ReceiveError::Refused(OrgNodeError::AdmissionNotExpected { org_id })) if org_id == pair.org),
        "{refused:?}"
    );
    assert_eq!(pair.chain_b.reads(), 0, "no chain read after a refused check");
    assert_eq!(store_bytes(pair.tag, "b"), files_before, "nothing written");
}

// ---- REQ-jxyh9c: each recipient gets the body org-node built for it ----------

async fn next_body(received: &mut mpsc::UnboundedReceiver<(DevicePublicKey, Vec<u8>)>) -> (DevicePublicKey, WireMessage) {
    let (sender, body) = tokio::time::timeout(NET, received.recv()).await.unwrap().unwrap();
    (sender, decode_message(&body).unwrap())
}

// Networked (the in-process relay, dialled by endpoint id): A revokes Bob
// from {Alice, Bob, Carol}. The report names Carol and Bob, never A's own
// Device; Carol, still listed, receives the Organisation's information and
// Bob, removed, receives the revocation — each from A's Device.
// verifies: REQ-jxyh9c
#[tokio::test(flavor = "multi_thread")]
async fn each_recipient_receives_the_body_org_node_built_for_it() {
    let (_guard, _keychain) = fresh_keychain();
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server().await.unwrap();
    let lookup = MemoryLookup::new();
    let plan = BindPlan::Relay(relay_map, lookup.clone());
    let chain = FakeChain::new();
    let a = handle("per-recipient", "a", &chain).with_bind_plan_for_test(plan.clone());
    let alice = create(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    let sender = a.endpoint_for_test(&alice).await.unwrap();
    tokio::time::timeout(NET, sender.inner().online()).await.unwrap();
    lookup.add_endpoint_info(sender.inner().addr());
    let sender_device = device_key_of(&a, &alice).await;

    let (bob, carol) = (joiner(0x61, "bob"), joiner(0x63, "carol"));
    let mut receivers = vec![];
    for device_seed in [[0x62; 32], [0x64; 32]] {
        let device = keep_device_seed(device_seed);
        let endpoint = OrgEndpoint::bind_from_keychain(&device, plan.clone()).await.unwrap();
        tokio::time::timeout(NET, endpoint.inner().online()).await.unwrap();
        lookup.add_endpoint_info(endpoint.inner().addr());
        receivers.push(forwarding(endpoint).1);
    }
    let (mut bob_receives, mut carol_receives) = (receivers.remove(0), receivers.remove(0));
    for joining in [&bob, &carol] {
        let update = a.admit_member(&mut OsRng, org, joining).await.unwrap();
        a.submit_commit_send(&mut OsRng, &update, None).await.unwrap();
    }
    next_body(&mut bob_receives).await;
    next_body(&mut bob_receives).await;
    next_body(&mut carol_receives).await;

    let bob_member = member_id_of(&a, org, bob.member_key).await;
    let revocation = a.revoke_member(&mut OsRng, org, bob_member).await.unwrap();
    let submitted = tokio::time::timeout(Duration::from_secs(120), a.submit_commit_send(&mut OsRng, &revocation, None))
        .await
        .unwrap()
        .unwrap();
    let mut reported: Vec<DevicePublicKey> = submitted.sends.per_device.iter().map(|(device, _)| *device).collect();
    reported.sort_by_key(|device| *device.as_bytes());
    let mut expected = vec![bob.device_key, carol.device_key];
    expected.sort_by_key(|device| *device.as_bytes());
    assert_eq!(reported, expected, "Carol, listed, and Bob, removed; never the sender's own Device");
    assert!(submitted.sends.per_device.iter().all(|(_, sent)| sent.is_ok()), "{:?}", submitted.sends);

    let (from, to_carol) = next_body(&mut carol_receives).await;
    assert_eq!(from, sender_device);
    assert!(matches!(to_carol, WireMessage::OrgInformation { .. }), "Carol, still listed, gets the Organisation's information");
    let (from, to_bob) = next_body(&mut bob_receives).await;
    assert_eq!(from, sender_device);
    assert!(matches!(to_bob, WireMessage::Revocation(_)), "Bob, removed, gets the revocation");
}

// Boundary (Loopback): revoking the last other Member leaves a record that
// lists the sender alone; the one recipient is the removed Device, which the
// record no longer names, and it receives the revocation.
// verifies: REQ-jxyh9c
#[tokio::test(flavor = "multi_thread")]
async fn the_last_other_member_removed_is_still_sent_its_revocation() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let a = handle("last-removed", "a", &chain);
    let alice = create(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    let sender_device = device_key_of(&a, &alice).await;
    let bob = joiner(0x61, "bob");
    let (bob_addr, mut bob_receives) = forwarding(bind_seed([0x62; 32]).await);
    let admission = a.admit_member(&mut OsRng, org, &bob).await.unwrap();
    a.submit_commit_send(&mut OsRng, &admission, at(bob_addr.clone())).await.unwrap();
    next_body(&mut bob_receives).await;

    let bob_member = member_id_of(&a, org, bob.member_key).await;
    let revocation = a.revoke_member(&mut OsRng, org, bob_member).await.unwrap();
    let submitted = a.submit_commit_send(&mut OsRng, &revocation, at(bob_addr)).await.unwrap();
    let record = a.organisation(org).await.unwrap();
    assert_eq!(record.members.len(), 1, "the record lists the sender alone");
    assert_eq!(submitted.sends.per_device.len(), 1, "{:?}", submitted.sends);
    assert_eq!(submitted.sends.per_device[0].0, bob.device_key, "the removed Device, which the record no longer names");
    assert!(submitted.sends.per_device[0].1.is_ok(), "{:?}", submitted.sends);
    let (from, to_bob) = next_body(&mut bob_receives).await;
    assert_eq!(from, sender_device);
    assert!(matches!(to_bob, WireMessage::Revocation(_)), "{to_bob:?}");
}

// ---- REQ-txkm94: Blobs, outstanding identifiers and expectations in the store

// The outstanding identifier and the declared expectation are in the store
// files: a reopened inviter accepts the reply and a reopened joiner holds
// the expectation; once the reply is acted on, a reopened inviter refuses
// it as no longer outstanding.
// verifies: REQ-txkm94
#[tokio::test(flavor = "multi_thread")]
async fn outstanding_invites_and_expectations_are_kept_and_settled_in_the_store_files() {
    let (_guard, _keychain) = fresh_keychain();
    let (dir_a, dir_b) = (fresh_dir("exchange", "a"), fresh_dir("exchange", "b"));
    let chain = FakeChain::new();
    let chain_b = FakeChain::sharing_slots_with(&chain);
    let a = open(&dir_a, &chain);
    let alice = create(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob Builder").await.unwrap();
    let b = open(&dir_b, &chain_b);
    let bob = create(&b, "bob").await;
    let reply = b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();

    let reopened_b = open(&dir_b, &chain_b);
    let expected: Vec<OrgId> = reopened_b.node_for_test().expected_admissions().iter().map(|expected| expected.org_id).collect();
    assert_eq!(expected, vec![org], "the declared expectation is in B's file");
    let reopened_a = open(&dir_a, &chain);
    let imported = reopened_a.import_reply(&reply).await.unwrap();
    assert_eq!((imported.org_id, imported.handle.as_str()), (org, "bob"), "the outstanding identifier is in A's file");

    reopened_a.admit_reply(&mut OsRng, org, &reply, None).await.unwrap();
    let settled = open(&dir_a, &chain);
    let refused = settled.import_reply(&reply).await.unwrap_err();
    assert!(matches!(refused, ExchangeError::Node(OrgNodeError::InviteNotOutstanding)), "settled in A's file: {refused:?}");
    assert_eq!(settled.organisation(org).await.unwrap().members.len(), 2, "and the admission with it");
}

// Abnormal: Blobs that do not parse are refused by org-node's parse, and a
// true reply admitted under another Organisation A holds is refused by the
// outstanding check; nothing is written to either side's files and nothing
// reaches the chain.
// verifies: REQ-txkm94
#[tokio::test(flavor = "multi_thread")]
async fn blobs_that_do_not_parse_and_a_reply_for_another_organisation_change_no_file() {
    let (_guard, _keychain) = fresh_keychain();
    let (dir_a, dir_b) = (fresh_dir("exchange-refused", "a"), fresh_dir("exchange-refused", "b"));
    let chain = FakeChain::new();
    let a = open(&dir_a, &chain);
    let alice = create(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    let carol = create(&a, "carol").await;
    let elsewhere = a.found_organisation(&mut OsRng, &carol).await.unwrap();
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob Builder").await.unwrap();
    let b = open(&dir_b, &FakeChain::sharing_slots_with(&chain));
    let bob = create(&b, "bob").await;
    let reply = b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    let (files_a, files_b, writes) = (files_in(&dir_a), files_in(&dir_b), chain.update_writes());

    for blob in ["", "not a blob", "00", &"ab".repeat(200), &invite[..invite.len() / 2]] {
        let as_invite = b.produce_reply(&mut OsRng, blob, &bob, true).await.unwrap_err();
        assert!(matches!(as_invite, ExchangeError::Node(_)), "{blob:?} as an Invite: {as_invite:?}");
        let as_reply = a.admit_reply(&mut OsRng, org, blob, None).await.unwrap_err();
        assert!(matches!(as_reply, ExchangeError::Node(_)), "{blob:?} as a reply: {as_reply:?}");
    }
    let misdirected = a.admit_reply(&mut OsRng, elsewhere, &reply, None).await.unwrap_err();
    assert!(matches!(misdirected, ExchangeError::Node(OrgNodeError::ReplyForAnotherOrganisation)), "{misdirected:?}");

    assert_eq!(files_in(&dir_a), files_a, "nothing written on A's side");
    assert_eq!(files_in(&dir_b), files_b, "nothing written on B's side");
    assert_eq!(chain.update_writes(), writes, "nothing written to the chain");
}
