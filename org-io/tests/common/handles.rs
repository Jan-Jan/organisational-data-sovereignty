//! Handles, Personas and transport helpers for the tests that drive `OrgIo`
//! (`submit_flow`, `receive_order`, `signatory_rule`). The relay and sink
//! helpers are copied
//! from org-node's `tests/support/mod.rs` (`spawn_recv_one`, `deliver_from`,
//! `dead_addr`), with the endpoint calls going through org-io's transitional
//! re-export of org-node. A `DeviceSeed` in a test file is allowed: org-io's
//! absence of device private keys (LLR-3zdw8v) is a scan of `org-io/src`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use org_io::node::service::OrgService;
use org_io::node::store::PersonaStore;
use org_io::node::transport::endpoint::OrgEndpoint;
use org_io::node::transport::wire::WireMessage;
use org_io::node::{DeviceSeed, DevicePublicKey, Handle, Joiner, MemberSeed, Name, PersonaId, Surname};
use org_io::test_support::FakeChain;
use org_io::OrgIo;
use rand::rngs::OsRng;

/// How long a network step may take before the test gives up.
pub const NET: Duration = Duration::from_secs(30);

/// The store file of `party`'s handle in test `tag`.
pub fn store_path(tag: &str, party: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("org-io-{tag}-{party}-{}", std::process::id())).join("store.bin")
}

/// `party`'s store in test `tag`, opened again from disk: what it persisted.
pub fn reopened_service(tag: &str, party: &str) -> OrgService {
    OrgService::new(PersonaStore::open(store_path(tag, party), "pw").unwrap())
}

/// A fresh handle for `party` in test `tag`, over `chain` (its slots, its
/// read counter and its switches).
pub fn handle(tag: &str, party: &str, chain: &FakeChain) -> OrgIo {
    let path = store_path(tag, party);
    let dir = path.parent().unwrap();
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).unwrap();
    OrgIo::for_test(OrgService::new(PersonaStore::open(path, "pw").unwrap()), chain)
}

/// Create a Persona on the handle's node.
pub fn persona(io: &mut OrgIo, handle_text: &str) -> PersonaId {
    io.node_mut()
        .create_persona(
            &mut OsRng,
            Handle::parse(handle_text).unwrap(),
            Name::parse("Test").unwrap(),
            Surname::parse("User").unwrap(),
        )
        .unwrap()
}

/// A joiner with keys from `seed` that no handle holds.
pub fn joiner(seed: u8, handle_text: &str) -> Joiner {
    Joiner {
        handle: Handle::parse(handle_text).unwrap(),
        name: Name::parse("Test").unwrap(),
        surname: Surname::parse("Joiner").unwrap(),
        member_key: MemberSeed::from([seed; 32]).x25519_keypair().member_key().unwrap(),
        device_key: DeviceSeed::from([seed + 1; 32]).signing_keypair().device_key().unwrap(),
    }
}

/// The joiner the Persona `persona_id` of `io` is admitted as.
pub fn joiner_of(io: &OrgIo, persona_id: &PersonaId) -> Joiner {
    let record = io.node_for_test().list_personas().iter().find(|p| &p.persona_id == persona_id).cloned().unwrap();
    let (member_key, device_key) = io.node_for_test().persona_public_keys(persona_id).unwrap();
    Joiner { handle: record.handle, name: record.name, surname: record.surname, member_key, device_key }
}

/// The address `io` receives on: its endpoint, bound from its first Persona
/// when it has none yet.
pub async fn receiving_addr(io: &mut OrgIo) -> iroh::EndpointAddr {
    if io.node_for_test().endpoint().is_none() {
        let first = io.node_for_test().list_personas()[0].persona_id.clone();
        io.node_mut().ensure_endpoint(&first).await.unwrap();
    }
    io.node_for_test().endpoint().unwrap().inner().addr()
}

/// A well-formed peer identity, from `seed`, carrying no transport
/// addresses: a dial to it fails at once.
pub fn dead_addr(seed: [u8; 32]) -> iroh::EndpointAddr {
    iroh::EndpointId::from_bytes(DeviceSeed::from(seed).signing_keypair().device_key().unwrap().as_bytes())
        .map(|id| iroh::EndpointAddr::from_parts(id, std::iter::empty()))
        .unwrap()
}

/// Bind an endpoint from `seed` and spawn one receive on it. The task yields
/// the endpoint back (held open until the sender's `send` returned) with the
/// authenticated sender and the message: a sink that captures a message.
pub async fn spawn_recv_one(
    seed: [u8; 32],
) -> (iroh::EndpointAddr, tokio::task::JoinHandle<(OrgEndpoint, DevicePublicKey, WireMessage)>) {
    let endpoint = OrgEndpoint::bind(&DeviceSeed::from(seed).signing_keypair()).await.unwrap();
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
    let sender = OrgEndpoint::bind(&DeviceSeed::from(sender_seed).signing_keypair()).await.unwrap();
    tokio::time::timeout(NET, sender.send(addr, message)).await.unwrap().unwrap();
}

/// Send `message` to `addr` from a relay Device that no record lists.
pub async fn deliver(addr: iroh::EndpointAddr, message: &WireMessage) {
    deliver_from([0x5b; 32], addr, message).await;
}
