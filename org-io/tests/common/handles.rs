//! Handles, Personas and transport helpers for the tests that drive `OrgIo`
//! (`submit_flow`, `receive_order`, `signatory_rule`, `publish_path`,
//! `startup_reconcile`). The relay and sink helpers are copied
//! from org-node's `tests/support/mod.rs` (`spawn_recv_one`, `deliver_from`,
//! `dead_addr`). *Since S4 task T10 (2026-10-13)* the endpoints are org-io's,
//! each bound from a keychain item of the installed in-memory keychain
//! (`keep_device_seed`, `bind_seed`). A device key pair in a test file is allowed:
//! org-io's absence of device private keys (LLR-3zdw8v) is a scan of
//! `org-io/src`.
//!
//! *Adapted 2026-10-08 (S4, task T3):* each handle writes its store changes
//! to a data directory of its own (`persona-<hex>.store` per Persona), and
//! keeps each new Persona's device secret; `reopened_service` unseals what
//! reached the directory with the secrets the handle keeps.
//!
//! *Reconciled with S3b-io part 1, 2026-10-14:* S3b-io's helpers (`founded`,
//! `held`, `timed_out`, `hung`, `store_bytes`) are rewritten onto the `&self`
//! handle; a submission names no recipient (S4, task T12), so `timed_out`
//! and `hung` take none, and `store_bytes` reads every Persona's store file.
//!
//! *Deslop pass, 2026-10-14:* the helpers several test files had each
//! copied (`create`, `fresh_keychain`, `committed`, `refusal`, `file_of`,
//! `held_store`, `forwarding`) are here once.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::atomic::Ordering;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use org_node::service::OrgService;
use org_node::store::{PersonaStore, SealedStore};
use org_node::test_fixtures::{device_key, device_keypair};
use org_node::wire::WireMessage;
use org_node::{
    DevicePublicKey, DeviceSecret, Handle, Joiner, MemberSeed, Name, OrgId, OrgNodeError, PersonaId, ReceiveOutcome,
    Surname,
};
use org_io::test_support::{FakeChain, MemoryKeychain};
use org_io::transport::{BindPlan, OrgEndpoint};
use org_io::{BuiltUpdate, OrgIo, PeerAddr, ReceiveError, Received};
use rand::rngs::OsRng;
use tokio::sync::mpsc;
use zeroize::Zeroizing;

/// How long a network step may take before the test gives up.
pub const NET: Duration = Duration::from_secs(30);

/// The data directory of `party`'s handle in test `tag`.
pub fn store_dir(tag: &str, party: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("org-io-{tag}-{party}-{}", std::process::id()))
}

/// `party`'s stores in test `tag`, opened again from its data directory with
/// the device secrets `io` keeps: what it persisted.
pub fn reopened_service(tag: &str, party: &str, io: &OrgIo) -> OrgService {
    let mut stores = vec![];
    for entry in std::fs::read_dir(store_dir(tag, party)).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let Some(hex_key) = name.strip_prefix("persona-").and_then(|rest| rest.strip_suffix(".store")) else { continue };
        let key_bytes: [u8; 32] = hex::decode(hex_key).unwrap().try_into().unwrap();
        let device = DevicePublicKey::parse(&key_bytes).unwrap();
        let sealed = SealedStore::from_bytes(std::fs::read(&path).unwrap());
        let secret = io.device_secret_for_test(&device).expect("the handle keeps the Persona's secret");
        stores.push(PersonaStore::unseal(&sealed, device, secret).unwrap());
    }
    OrgService::new(stores).unwrap()
}

/// A fresh handle for `party` in test `tag`, over `chain` (its slots, its
/// read counter and its switches), writing its stores to a fresh directory.
pub fn handle(tag: &str, party: &str, chain: &FakeChain) -> OrgIo {
    let dir = store_dir(tag, party);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    OrgIo::for_test(OrgService::new(Vec::new()).unwrap(), chain).with_data_dir_for_test(dir)
}

/// The bytes of `party`'s store files in test `tag`, as they are on disk, in
/// file-name order. *Reconciled with S3b-io part 1, 2026-10-14:* one file
/// per Persona since S4 task T3, so every file is read.
pub fn store_bytes(tag: &str, party: &str) -> Vec<u8> {
    let mut paths: Vec<_> =
        std::fs::read_dir(store_dir(tag, party)).unwrap().map(|entry| entry.unwrap().path()).collect();
    paths.sort();
    paths.into_iter().flat_map(|path| std::fs::read(path).unwrap()).collect()
}

/// Party `a`'s handle over a fresh chain whose first Persona founded an
/// Organisation.
pub async fn founded(tag: &str) -> (FakeChain, OrgIo, OrgId) {
    let chain = FakeChain::new();
    let io = handle(tag, "a", &chain);
    let pid = persona(&io, "alice").await;
    let org = io.found_organisation(&mut OsRng, &pid).await.unwrap();
    (chain, io, org)
}

/// Whether org-node holds the provisional update `update` names.
pub fn held(io: &OrgIo, update: &BuiltUpdate) -> bool {
    io.node_for_test().holds_provisional(update.org_id, update.base_root, update.resulting_root, update.org_pub_key)
}

/// Submit `update` while every write never finishes, crossing the 90-second
/// bound on a paused clock: the write's outcome is unknown. The runtime is
/// current-thread; the clock is paused only while the bound is crossed, and
/// resumed before any network step.
pub async fn timed_out(io: &OrgIo, update: &BuiltUpdate, peer_addr: Option<PeerAddr>) {
    tokio::time::pause();
    let err = io.submit_commit_send(&mut OsRng, update, peer_addr).await.unwrap_err();
    tokio::time::resume();
    assert!(err.contains("outcome unknown"), "{err}");
}

/// Submit `update` as a write that never finishes and never lands.
pub async fn hung(chain: &FakeChain, io: &OrgIo, update: &BuiltUpdate) {
    chain.write_hanging.store(true, Ordering::SeqCst);
    timed_out(io, update, None).await;
    chain.write_hanging.store(false, Ordering::SeqCst);
}

/// Create a Persona on the handle's node. *Adapted 2026-10-13 (S4, task
/// T8):* its device secret goes to the keychain, org-io's in-memory one, which
/// every test of the binary shares (each Persona has an item of its own).
/// *Adapted 2026-10-13 (S4, task T9):* created by the handle itself.
/// *Adapted 2026-10-13 (S4, task T11):* the handle's methods are `async`
/// and take `&self`.
pub async fn persona(io: &OrgIo, handle_text: &str) -> PersonaId {
    persona_named(io, handle_text, "Test", "User").await
}

/// `persona` with the name and surname given.
pub async fn persona_named(io: &OrgIo, handle_text: &str, name: &str, surname: &str) -> PersonaId {
    MemoryKeychain::shared().install();
    create_named(io, handle_text, name, surname).await
}

/// Create a Persona named Test User on the handle's node; its device secret
/// goes to whichever in-memory keychain the test installed.
pub async fn create(io: &OrgIo, handle_text: &str) -> PersonaId {
    create_named(io, handle_text, "Test", "User").await
}

/// `create` with the name and surname given.
pub async fn create_named(io: &OrgIo, handle_text: &str, name: &str, surname: &str) -> PersonaId {
    let (handle, name, surname) =
        (Handle::parse(handle_text).unwrap(), Name::parse(name).unwrap(), Surname::parse(surname).unwrap());
    io.create_persona(&mut OsRng, handle, name, surname).await.unwrap()
}

/// Serialises the tests of one binary that install a keychain of their own
/// (keyring's credential builder is process-global).
static KEYCHAIN_LOCK: Mutex<()> = Mutex::new(());

/// The lock, and a fresh in-memory keychain installed for as long as it is
/// held.
pub fn fresh_keychain() -> (MutexGuard<'static, ()>, MemoryKeychain) {
    let guard = KEYCHAIN_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let keychain = MemoryKeychain::default();
    keychain.install();
    (guard, keychain)
}

/// The outcome of a receive that committed; anything else fails the test.
pub fn committed(received: Result<Received, ReceiveError>) -> ReceiveOutcome {
    match received {
        Ok(Received::Committed(outcome)) => outcome,
        other => panic!("not committed: {other:?}"),
    }
}

/// org-node's refusal of a receive; anything else fails the test.
pub fn refusal(received: Result<Received, ReceiveError>) -> OrgNodeError {
    match received {
        Err(ReceiveError::Refused(refused)) => refused,
        other => panic!("not refused by org-node: {other:?}"),
    }
}

/// The store file of `device_key` in `dir`, by the naming rule spelled out
/// here, independently of org-io's.
pub fn file_of(dir: &std::path::Path, device_key: &DevicePublicKey) -> std::path::PathBuf {
    dir.join(format!("persona-{}.store", hex::encode(device_key.as_bytes())))
}

/// The store `io` holds for `persona_id`, as its `Debug` shows it.
pub fn held_store(io: &OrgIo, persona_id: &PersonaId) -> String {
    format!("{:?}", io.node_for_test().store_data(persona_id).expect("the handle holds the Persona"))
}

/// `endpoint`'s address, and every body it receives, with its authenticated
/// sender, forwarded.
pub fn forwarding(endpoint: OrgEndpoint) -> (iroh::EndpointAddr, mpsc::UnboundedReceiver<(DevicePublicKey, Vec<u8>)>) {
    let addr = endpoint.inner().addr();
    let (forward, received) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Ok(body) = endpoint.recv_body().await {
            if forward.send(body).is_err() {
                return;
            }
        }
    });
    (addr, received)
}

/// A joiner with keys from `seed` that no handle holds.
pub fn joiner(seed: u8, handle_text: &str) -> Joiner {
    Joiner {
        handle: Handle::parse(handle_text).unwrap(),
        name: Name::parse("Test").unwrap(),
        surname: Surname::parse("Joiner").unwrap(),
        member_key: MemberSeed::from([seed; 32]).x25519_keypair().member_key().unwrap(),
        device_key: device_key(seed + 1),
    }
}

/// The joiner the Persona `persona_id` of `io` is admitted as.
pub fn joiner_of(io: &OrgIo, persona_id: &PersonaId) -> Joiner {
    let record = io.node_for_test().list_personas().into_iter().find(|p| &p.persona_id == persona_id).cloned().unwrap();
    let (member_key, device_key) = io.node_for_test().persona_public_keys(persona_id).unwrap();
    Joiner { handle: record.handle, name: record.name, surname: record.surname, member_key, device_key }
}

/// The address `io` receives on: it starts receiving (every Persona's
/// endpoint bound and accepting; S4, task T10) and gives its first Persona's
/// endpoint's address.
pub async fn receiving_addr(io: &OrgIo) -> iroh::EndpointAddr {
    io.start_receiving().await.unwrap();
    let first = io.node_for_test().list_personas()[0].persona_id.clone();
    io.endpoint_for_test(&first).await.unwrap().inner().addr()
}

/// `addr` as the peer address `submit_commit_send` takes. *Added 2026-10-13
/// (S4, task T12):* the handle takes org-io's `PeerAddr`, not iroh's type.
pub fn at(addr: iroh::EndpointAddr) -> Option<org_io::PeerAddr> {
    Some(org_io::PeerAddr::for_test(addr))
}

/// A well-formed peer identity, from `seed`, carrying no transport
/// addresses: a dial to it fails at once.
pub fn dead_addr(seed: [u8; 32]) -> iroh::EndpointAddr {
    iroh::EndpointId::from_bytes(device_keypair(seed).device_key().unwrap().as_bytes())
        .map(|id| iroh::EndpointAddr::from_parts(id, std::iter::empty()))
        .unwrap()
}

/// Put the Device seed `seed` in the in-memory keychain installed (by the
/// caller, or by `persona`), as a Persona's item; its Device key. *Added
/// 2026-10-13 (S4, task T10):* org-io binds an endpoint only from a keychain
/// item.
pub fn keep_device_seed(seed: [u8; 32]) -> DevicePublicKey {
    let secret = DeviceSecret::from_keychain_bytes(Zeroizing::new(seed.to_vec())).unwrap();
    let device_key = secret.device_key();
    org_io::test_support::keychain::write_device_secret(&device_key, &secret).unwrap();
    device_key
}

/// A Loopback endpoint bound from the keychain item of `seed`'s Device.
pub async fn bind_seed(seed: [u8; 32]) -> OrgEndpoint {
    OrgEndpoint::bind_from_keychain(&keep_device_seed(seed), BindPlan::Loopback).await.unwrap()
}

/// Bind an endpoint from `seed` and spawn one receive on it. The task yields
/// the endpoint back (held open until the sender's `send` returned) with the
/// authenticated sender and the message: a sink that captures a message.
pub async fn spawn_recv_one(
    seed: [u8; 32],
) -> (iroh::EndpointAddr, tokio::task::JoinHandle<(OrgEndpoint, DevicePublicKey, WireMessage)>) {
    spawn_recv_one_as(&keep_device_seed(seed)).await
}

/// `spawn_recv_one` for the Device `device_key`, whose keychain item exists
/// (a Persona's, or one `keep_device_seed` put there). *Added 2026-10-13
/// (S4, task T12):* a Loopback send goes only to an address whose identity
/// is a recipient's Device key, so a sink stands in for a recipient.
pub async fn spawn_recv_one_as(
    device_key: &DevicePublicKey,
) -> (iroh::EndpointAddr, tokio::task::JoinHandle<(OrgEndpoint, DevicePublicKey, WireMessage)>) {
    let endpoint = OrgEndpoint::bind_from_keychain(device_key, BindPlan::Loopback).await.unwrap();
    let addr = endpoint.inner().addr();
    let task = tokio::spawn(async move {
        let (sender, message) = tokio::time::timeout(NET, endpoint.recv_one()).await.unwrap().unwrap();
        (endpoint, sender, message)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    (addr, task)
}

/// Send `message` to `addr` from the Device of `sender_seed`.
pub async fn deliver_from(sender_seed: [u8; 32], addr: iroh::EndpointAddr, message: &WireMessage) {
    let sender = bind_seed(sender_seed).await;
    tokio::time::timeout(NET, sender.send(addr, message)).await.unwrap().unwrap();
}

/// Send `message` to `addr` from a relay Device that no record lists.
pub async fn deliver(addr: iroh::EndpointAddr, message: &WireMessage) {
    deliver_from([0x5b; 32], addr, message).await;
}
