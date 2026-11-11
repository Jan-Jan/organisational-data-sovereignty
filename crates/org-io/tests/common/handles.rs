//! Handles, Personas and transport helpers for the tests that drive `OrgIo`. The relay and sink
//! helpers are copied from org-node's `tests/support/mod.rs`. Endpoints are org-io's, each bound
//! from a keychain item of the installed in-memory keychain (`keep_device_seed`, `bind_seed`). A
//! device key pair in a test file is allowed: org-io's absence of device private keys
//! (LLR-3zdw8v) is a scan of `crates/org-io/src`.
//!
//! A Persona is made here as the handle makes one, a provisional founding. Every helper reaches
//! its state through the handle's own operations (`propose_founding_organisation`,
//! `write_organisation_to_chain`, `verify_organisation`, `request_admission` taken by
//! `next_received`); none calls org-node through the handle or writes a store change itself.
//! What one reads of org-node's store it reads from the files the handle wrote
//! (`reopened_from`).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use org_node::service::OrgService;
use org_node::store::{PersonaStore, SealedStore};
use org_node::{DevicePublicKey, DeviceSecret, OrgId, OrgNodeError, PersonaId};
use org_io::send::Delivery;
use org_io::test_support::{FakeChain, MemoryKeychain};
use org_io::transport::{BindPlan, OrgEndpoint};
use org_io::{HeldTarget, OrgIo, ReceiveError, Received, Verified, WriteReport};
use zeroize::Zeroizing;

/// How long a network step may take before the test gives up.
pub const NET: Duration = Duration::from_secs(30);

/// The data directory of `party`'s handle in test `tag`.
pub fn store_dir(tag: &str, party: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("org-io-{tag}-{party}-{}", std::process::id()))
}

/// `party`'s stores in test `tag`, opened again from its data directory with the device secrets
/// the keychain holds: what the handle persisted.
pub fn reopened_service(tag: &str, party: &str, io: &OrgIo) -> OrgService {
    reopened_from(&store_dir(tag, party), io)
}

/// The stores in `dir` that the handle `io` wrote, opened again with the device secrets the
/// keychain holds: what the handle persisted, read apart from the handle.
pub fn reopened_from(dir: &std::path::Path, io: &OrgIo) -> OrgService {
    let mut stores = vec![];
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let Some(hex_key) = name.strip_prefix("persona-").and_then(|rest| rest.strip_suffix(".store")) else { continue };
        let key_bytes: [u8; 32] = hex::decode(hex_key).unwrap().try_into().unwrap();
        let device_key = DevicePublicKey::parse(&key_bytes).unwrap();
        let sealed = SealedStore::from_bytes(std::fs::read(&path).unwrap());
        let secret = io.device_secret_for_test(&device_key).expect("the keychain holds the Persona's secret");
        stores.push(PersonaStore::unseal(&sealed, device_key, secret).unwrap());
    }
    OrgService::new(stores).unwrap()
}

/// A fresh handle for `party` in test `tag`, over `chain`, writing its stores to a fresh directory.
pub fn handle(tag: &str, party: &str, chain: &FakeChain) -> OrgIo {
    let dir = store_dir(tag, party);
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    OrgIo::for_test(OrgService::new(Vec::new()).unwrap(), chain).with_data_dir_for_test(dir)
}

/// The bytes of `party`'s store files in test `tag`, in file-name order.
pub fn store_bytes(tag: &str, party: &str) -> Vec<u8> {
    let mut paths: Vec<_> = std::fs::read_dir(store_dir(tag, party)).unwrap().map(|entry| entry.unwrap().path()).collect();
    paths.sort();
    paths.into_iter().flat_map(|path| std::fs::read(path).unwrap()).collect()
}

/// A Persona on the handle's node, its secret in the shared in-memory keychain.
pub async fn persona(io: &OrgIo, handle_text: &str) -> PersonaId {
    persona_named(io, handle_text, "Test", "User").await
}

/// `persona` with the name and surname given.
pub async fn persona_named(io: &OrgIo, handle_text: &str, name: &str, surname: &str) -> PersonaId {
    MemoryKeychain::shared().install();
    create_named(io, handle_text, name, surname).await
}

/// A Persona named Test User; its secret goes to whichever in-memory keychain the test installed.
pub async fn create(io: &OrgIo, handle_text: &str) -> PersonaId {
    create_named(io, handle_text, "Test", "User").await
}

/// A Persona as the handle makes one: a provisional founding of "Test Organisation", proposed
/// through `propose_founding_organisation` (its Device seed in the keychain item, then its store
/// file). Binds no endpoint unless the handle receives.
pub async fn create_named(io: &OrgIo, handle_text: &str, name: &str, surname: &str) -> PersonaId {
    let founding = io.propose_founding_organisation(name, surname, handle_text, "Test Organisation").await.unwrap();
    persona_of(io, &founding.founder).await
}

/// The Persona whose Device key is `device_key`, as the handle lists it.
pub async fn persona_of(io: &OrgIo, device_key: &DevicePublicKey) -> PersonaId {
    io.personas().await.into_iter().find(|persona| &persona.device_key == device_key).map(|persona| persona.persona_id).unwrap()
}

/// The Device key of the Persona `persona_id`, as the handle lists it.
pub async fn device_key_of(io: &OrgIo, persona_id: &PersonaId) -> DevicePublicKey {
    io.personas().await.into_iter().find(|persona| &persona.persona_id == persona_id).expect("a held Persona").device_key
}

/// The names of the entries in `dir`, sorted.
pub fn names_in(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> =
        std::fs::read_dir(dir).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().to_string()).collect();
    names.sort();
    names
}

/// Make `dir` read-only.
pub fn make_read_only(dir: &std::path::Path) {
    let mut permissions = std::fs::metadata(dir).unwrap().permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(dir, permissions).unwrap();
}

/// Make `dir` writable again.
#[allow(clippy::permissions_set_readonly_false)]
pub fn make_writable(dir: &std::path::Path) {
    let mut permissions = std::fs::metadata(dir).unwrap().permissions();
    permissions.set_readonly(false);
    std::fs::set_permissions(dir, permissions).unwrap();
}

/// Serialises the tests of one binary that install a keychain of their own.
static KEYCHAIN_LOCK: Mutex<()> = Mutex::new(());

/// The lock, and a fresh in-memory keychain installed for as long as it is held.
pub fn fresh_keychain() -> (MutexGuard<'static, ()>, MemoryKeychain) {
    let guard = KEYCHAIN_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let keychain = MemoryKeychain::default();
    keychain.install();
    (guard, keychain)
}

/// org-node's refusal of a receive; anything else fails the test.
pub fn refusal(received: Result<Received, ReceiveError>) -> OrgNodeError {
    match received {
        Err(ReceiveError::Refused(refused)) => refused,
        other => panic!("not refused by org-node: {other:?}"),
    }
}

/// The store file of `device_key` in `dir`, by the naming rule spelled out here.
pub fn file_of(dir: &std::path::Path, device_key: &DevicePublicKey) -> std::path::PathBuf {
    dir.join(format!("persona-{}.store", hex::encode(device_key.as_bytes())))
}

/// The removed name of `device_key`'s store file in `dir`: `removed-<hex>.store`.
pub fn removed_file_of(dir: &std::path::Path, device_key: &DevicePublicKey) -> std::path::PathBuf {
    dir.join(format!("removed-{}.store", hex::encode(device_key.as_bytes())))
}

/// The two names `device_key`'s store may have in `dir`: its store file and its removed file.
pub fn store_and_removed_files(dir: &std::path::Path, device_key: &DevicePublicKey) -> (std::path::PathBuf, std::path::PathBuf) {
    (file_of(dir, device_key), removed_file_of(dir, device_key))
}

/// Whether `keychain` holds `device_key`'s item.
pub fn item_held(keychain: &MemoryKeychain, device_key: &DevicePublicKey) -> bool {
    let user = hex::encode(device_key.as_bytes());
    keychain.items().iter().any(|(_, held)| *held == user)
}

/// What `io` shows of the Persona `persona_id` through its own queries — the Persona, the
/// Organisation it holds and the admission requests it keeps for it — as their `Debug` shows
/// them.
pub async fn held_store(io: &OrgIo, persona_id: &PersonaId) -> String {
    let persona = io.personas().await.into_iter().find(|persona| &persona.persona_id == persona_id).expect("the handle holds the Persona");
    let organisation = match persona.org_id {
        Some(org_id) => io.organisation(org_id).await,
        None => None,
    };
    let requests: Vec<_> =
        io.admission_requests().await.into_iter().filter(|request| Some(request.org_id) == persona.org_id).collect();
    format!("{persona:?} {organisation:?} {requests:?}")
}

/// The address `io` receives on: it starts receiving (every Persona's
/// endpoint bound and accepting) and gives its first Persona's endpoint's
/// address.
pub async fn receiving_addr(io: &OrgIo) -> iroh::EndpointAddr {
    io.start_receiving().await.unwrap();
    let first = io.personas().await[0].persona_id.clone();
    io.endpoint_for_test(&first).await.unwrap().inner().addr()
}

/// Put the Device seed `seed` in the in-memory keychain installed (by the
/// caller, or by `persona`), as a Persona's item; its Device key: org-io
/// binds an endpoint only from a keychain item.
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

/// A founding proposed through `io` with `handle_text`, written to its chain in its two steps
/// and adopted by its verification, all through the handle. Returns the Organisation and the
/// founder's Device key.
pub async fn founded(io: &OrgIo, handle_text: &str) -> (OrgId, DevicePublicKey) {
    let founding = io.propose_founding_organisation("Alice", "Doe", handle_text, "Acme").await.unwrap();
    let written = io.write_organisation_to_chain(HeldTarget::Founding(founding.founder)).await.unwrap();
    assert!(matches!(written, WriteReport::Included { .. }), "{written:?}");
    let verified = io.verify_organisation(HeldTarget::Founding(founding.founder)).await;
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    let org_id = io
        .personas()
        .await
        .into_iter()
        .find(|persona| persona.device_key == founding.founder)
        .and_then(|persona| persona.org_id)
        .expect("the founding's Organisation");
    (org_id, founding.founder)
}

/// `joiner` proposes itself as a Member from `blob` with `handles` and asks `issuer` for
/// admission over the transport, and `issuer`'s receive step keeps the request (both receiving).
/// Returns the joiner's Device key.
pub async fn requested(issuer: &OrgIo, joiner: &OrgIo, blob: &str, handles: &[&str]) -> DevicePublicKey {
    let member = joiner.propose_member(blob, "Bob", "Builder", handles).await.unwrap();
    issuer.start_receiving().await.unwrap();
    joiner.start_receiving().await.unwrap();
    let (kept, asked) = tokio::join!(tokio::time::timeout(NET, issuer.next_received()), joiner.request_admission(member.org_id));
    let kept = kept.expect("the request arrived in time");
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
    assert!(matches!(asked, Ok(Delivery::Delivered)), "{asked:?}");
    member.member_device
}

/// `io`'s next epoch of `org_id`, proposed: an Invite issued, `joiner`'s request kept, its admission
/// recorded, and the updated Organisation proposed.
pub async fn proposed(io: &OrgIo, org_id: OrgId, joiner: &OrgIo) -> org_io::ProposedOrganisation {
    let blob = io.issue_invite(org_id, "Bob").await.unwrap();
    let invite_id = org_node::invitation::Invite::parse(&blob).unwrap().invite_id;
    requested(io, joiner, &blob, &["bob"]).await;
    io.propose_admission(org_id, invite_id, None).await.unwrap();
    io.propose_updated_organisation(org_id).await.unwrap()
}
