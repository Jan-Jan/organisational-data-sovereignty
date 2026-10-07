//! Story helpers shared by org-node's service-level test targets. Not a test
//! target: each target that uses it declares `mod support;`.
//!
//! The story operations — `found`, `prepare_to_join`, `joiner_of`, `admit`,
//! `revoke` — are the only place a test performs a story step, so a change to
//! the service API changes their bodies and not the tests that call them.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use org_members::{DevicePublicKey, Handle, MemberId, Name, Surname};
use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::service::{ChainOps, MockChainOps, OrgService, ReceiveOutcome, SelfDeleteOutcome};
use org_node::store::{MemberSnapshot, OrgRecord, PersonaRecord, PersonaStore, ProvisionalChange, ProvisionalUpdate};
use org_node::{ChainAccount, Joiner, OrgPrivateKey};
use org_node::transport::endpoint::OrgEndpoint;
use org_node::transport::wire::WireMessage;
use org_node::{DeviceSeed, Epoch, PersonaId};
use rand::rngs::OsRng;

pub const NET: Duration = Duration::from_secs(30);

/// The rogue relay's device seed — a third device, neither A's nor B's.
pub const ROGUE_SEED: [u8; 32] = [0x33u8; 32];

pub fn h(s: &str) -> Handle {
    Handle::parse(s).unwrap()
}
pub fn nm(s: &str) -> Name {
    Name::parse(s).unwrap()
}
pub fn sn(s: &str) -> Surname {
    Surname::parse(s).unwrap()
}

/// One party's store directory, unique per target, test and process.
pub fn store_dir(tag: &str, party: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ods-{}-{tag}-{party}-{}",
        env!("CARGO_CRATE_NAME"),
        std::process::id()
    ))
}

/// A fresh encrypted store, wiping anything an earlier run left there.
pub fn open_store(tag: &str, party: &str, password: &str) -> PersonaStore {
    let dir = store_dir(tag, party);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    PersonaStore::open(dir.join("store.bin"), password).unwrap()
}

/// Reopen a store without wiping it: what actually reached the disk.
pub fn reopen_store(tag: &str, party: &str, password: &str) -> PersonaStore {
    PersonaStore::open(store_dir(tag, party).join("store.bin"), password).unwrap()
}

/// The raw bytes of a party's store file (to prove nothing was written).
pub fn store_bytes(tag: &str, party: &str) -> Vec<u8> {
    std::fs::read(store_dir(tag, party).join("store.bin")).unwrap()
}

/// A persona's record, cloned.
pub fn persona_of(svc: &OrgService, persona_id: &PersonaId) -> PersonaRecord {
    svc.list_personas()
        .iter()
        .find(|p| &p.persona_id == persona_id)
        .expect("persona not found")
        .clone()
}

/// The device keypair of a persona, from its persisted `device_seed`.
pub fn device_kp(svc: &OrgService, persona_id: &PersonaId) -> SigningKeypair {
    persona_of(svc, persona_id).device_seed.signing_keypair()
}

/// `svc`'s record of `org_id`, cloned.
pub fn rec_of(svc: &OrgService, org_id: OrgId) -> OrgRecord {
    svc.list_orgs().iter().find(|o| o.org_id == org_id).expect("record").clone()
}

/// The record of `org_id` as `store` read it from disk, cloned.
pub fn disk_rec_of(store: &PersonaStore, org_id: OrgId) -> OrgRecord {
    store.data().orgs.iter().find(|o| o.org_id == org_id).expect("record on disk").clone()
}

/// The MemberId of the member with `handle` in `rec`.
pub fn id_by_handle(rec: &OrgRecord, handle: &str) -> MemberId {
    rec.trie_members.iter().find(|m| m.handle.as_str() == handle).expect("member in the record").id
}

/// A well-formed peer identity, from `seed`, carrying NO transport addresses.
/// The call sites say why a dial to it fails at once.
pub fn dead_addr(seed: [u8; 32]) -> iroh::EndpointAddr {
    iroh::EndpointId::from_bytes(DeviceSeed::from(seed).signing_keypair().device_key().unwrap().as_bytes())
        .map(|id| iroh::EndpointAddr::from_parts(id, std::iter::empty()))
        .expect("a well-formed endpoint identity")
}

/// Bind a fresh receiving endpoint for B (from B's device key), install it on
/// B's service and spawn `receive_and_verify`. Returns the address the sender
/// must dial and the task handle yielding `(svc_b, result)`.
pub async fn spawn_receive(
    svc_b: OrgService,
    b_device_kp: &SigningKeypair,
) -> (
    iroh::EndpointAddr,
    tokio::task::JoinHandle<(OrgService, Result<ReceiveOutcome, OrgNodeError>)>,
) {
    let ep_b = OrgEndpoint::bind(b_device_kp).await.unwrap();
    let b_addr = ep_b.inner().addr();
    let mut svc_b = svc_b.with_endpoint(ep_b);
    let handle = tokio::spawn(async move {
        let result = tokio::time::timeout(NET, svc_b.receive_and_verify(&mut OsRng))
            .await
            .expect("B receive_and_verify timed out");
        (svc_b, result)
    });
    // Let accept() be waiting before the sender dials.
    tokio::time::sleep(Duration::from_millis(50)).await;
    (b_addr, handle)
}

/// Bind a fresh receiving endpoint for B and spawn the **self-delete** receive
/// instead of the ordinary one. Mirrors `spawn_receive`.
pub async fn spawn_self_delete(
    svc_b: OrgService,
    b_device_kp: &SigningKeypair,
) -> (
    iroh::EndpointAddr,
    tokio::task::JoinHandle<(
        OrgService,
        Result<SelfDeleteOutcome, OrgNodeError>,
    )>,
) {
    let ep_b = OrgEndpoint::bind(b_device_kp).await.unwrap();
    let b_addr = ep_b.inner().addr();
    let mut svc_b = svc_b.with_endpoint(ep_b);
    let handle = tokio::spawn(async move {
        let result =
            tokio::time::timeout(NET, svc_b.receive_and_self_delete_if_revoked(&mut OsRng))
                .await
                .expect("B receive_and_self_delete_if_revoked timed out");
        (svc_b, result)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    (b_addr, handle)
}

/// Bind an endpoint from `seed` and spawn `recv_one`. The task yields the
/// endpoint back with the authenticated sender and the message it received:
/// a rogue relay R (`ROGUE_SEED`) relays the message; a sink (random seed)
/// holds the endpoint open until the sender's `send` has returned, since
/// dropping it on receipt can lose the acknowledgement the sender waits for.
pub async fn spawn_recv_one(
    seed: [u8; 32],
) -> (
    iroh::EndpointAddr,
    tokio::task::JoinHandle<(OrgEndpoint, DevicePublicKey, WireMessage)>,
) {
    let ep = OrgEndpoint::bind(&DeviceSeed::from(seed).signing_keypair()).await.unwrap();
    let addr = ep.inner().addr();
    let handle = tokio::spawn(async move {
        let (sender, msg) = tokio::time::timeout(NET, ep.recv_one())
            .await
            .expect("recv_one timed out")
            .expect("recv_one failed");
        (ep, sender, msg)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    (addr, handle)
}

/// The proxy account the app hands `commit_genesis` in these tests.
pub fn test_proxy() -> ChainAccount {
    ChainAccount::new([0x5a; 32])
}

/// The Organisation private key a provisional update holds.
pub fn private_key_of(update: &ProvisionalUpdate) -> OrgPrivateKey {
    match &update.change {
        ProvisionalChange::Genesis { org_private_key, .. } | ProvisionalChange::ChangeSet { org_private_key, .. } => {
            org_private_key.clone()
        }
    }
}

/// Story 1: `pid` founds an Organisation — the genesis update is built, the
/// app's chain write is stood in for by the mock, and the update committed.
pub async fn found(svc: &mut OrgService, chain: &MockChainOps, pid: &PersonaId) -> OrgId {
    let update = svc.create_organisation(&mut OsRng, pid).expect("build the genesis update");
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    svc.commit_genesis(&mut OsRng, pid, org_id, test_proxy()).await.expect("commit genesis");
    org_id
}

/// Story 2, the joining node's half: whatever it must hold before an
/// admission to `org_id` reaches it.
pub fn prepare_to_join(svc_b: &mut OrgService, org_id: OrgId) {
    svc_b.expect_admission(&mut OsRng, org_id).expect("expect the admission");
}

/// The joiner `pid` of `svc` is admitted as: the Persona's details and its
/// two public keys, as the app reads them from org-node (LLR-437fvx).
pub fn joiner_of(svc: &OrgService, pid: &PersonaId) -> Joiner {
    let p = persona_of(svc, pid);
    let (member_key, device_key) = svc.persona_public_keys(pid).unwrap();
    Joiner { handle: p.handle, name: p.name, surname: p.surname, member_key, device_key }
}

/// Story 3: admit `joiner` into `org_id`: build, write the chain (mock),
/// commit, send to `addr`. Returns the joiner's new MemberId.
pub async fn admit(
    svc: &mut OrgService,
    chain: &MockChainOps,
    org_id: OrgId,
    joiner: &Joiner,
    addr: iroh::EndpointAddr,
) -> Result<MemberId, OrgNodeError> {
    let update = svc.admit_member(&mut OsRng, org_id, joiner)?;
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, rec_of(svc, org_id).epoch)?;
    let outcome = svc.commit_update(&mut OsRng, org_id).await?;
    let id = rec_of(svc, org_id)
        .trie_members
        .iter()
        .find(|m| m.member_key == joiner.member_key)
        .map(|m| m.id)
        .expect("the joiner is in the committed record");
    tokio::time::timeout(NET, svc.send_update(&outcome, joiner.device_key, Some(addr)))
        .await
        .expect("send timed out")?;
    Ok(id)
}

/// Story 5: revoke `member_id`: build, write the chain (mock), commit, send
/// the committed revocation to the removed member's first device.
pub async fn revoke(
    svc: &mut OrgService,
    chain: &MockChainOps,
    org_id: OrgId,
    member_id: MemberId,
    addr: Option<iroh::EndpointAddr>,
) -> Result<(), OrgNodeError> {
    let recipient = rec_of(svc, org_id)
        .trie_members
        .iter()
        .find(|m| m.id == member_id)
        .and_then(|m| m.device_keys.first().copied())
        .expect("the removed member has a device");
    revoke_and_send(svc, chain, org_id, member_id, recipient, addr).await
}

/// Story 5, told to a Device the committed record still lists: build, write
/// the chain (mock), commit, and send the committed removal to `recipient`
/// at `addr`.
pub async fn revoke_and_tell(
    svc: &mut OrgService,
    chain: &MockChainOps,
    org_id: OrgId,
    member_id: MemberId,
    recipient: DevicePublicKey,
    addr: iroh::EndpointAddr,
) -> Result<(), OrgNodeError> {
    revoke_and_send(svc, chain, org_id, member_id, recipient, Some(addr)).await
}

/// Build the removal of `member_id`, write the chain (mock), commit, and
/// send the committed update to `recipient`.
async fn revoke_and_send(
    svc: &mut OrgService,
    chain: &MockChainOps,
    org_id: OrgId,
    member_id: MemberId,
    recipient: DevicePublicKey,
    addr: Option<iroh::EndpointAddr>,
) -> Result<(), OrgNodeError> {
    let update = svc.revoke_member(&mut OsRng, org_id, member_id)?;
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, rec_of(svc, org_id).epoch)?;
    let outcome = svc.commit_update(&mut OsRng, org_id).await?;
    tokio::time::timeout(NET, svc.send_update(&outcome, recipient, addr))
        .await
        .expect("send timed out")
}

/// `msg`, of the same kind, with its Envelope replaced.
pub fn with_envelope(msg: &WireMessage, envelope: org_node::Envelope) -> WireMessage {
    match msg {
        WireMessage::OrgInformation { record_snapshot, org_private_key, .. } => WireMessage::OrgInformation {
            envelope,
            record_snapshot: record_snapshot.clone(),
            org_private_key: org_private_key.clone(),
        },
        WireMessage::Revocation(_) => panic!("a revocation carries no Envelope"),
        WireMessage::Acknowledgement(_) => panic!("an acknowledgement carries no Envelope"),
    }
}

/// Organisation information carrying `envelope`, an empty snapshot and a
/// key no chain holds: for tests of the chain-free checks, which refuse the
/// Envelope before the snapshot or the key is looked at. (Until S3 T4 those
/// tests sent the Envelope as a revocation, which holds a notice now.)
pub fn carrying(envelope: org_node::Envelope) -> WireMessage {
    WireMessage::OrgInformation { envelope, record_snapshot: Vec::new(), org_private_key: OrgPrivateKey::from([0u8; 32]) }
}

/// The Envelope of the Organisation information `msg`; only that kind holds
/// one (LLR-js9dsu).
pub fn envelope_of(msg: &WireMessage) -> &org_node::Envelope {
    match msg {
        WireMessage::OrgInformation { envelope, .. } => envelope,
        other => panic!("Organisation information, got {other:?}"),
    }
}

/// The Organisation information `msg`, with its record snapshot replaced.
pub fn with_snapshot(msg: &WireMessage, record_snapshot: Vec<u8>) -> WireMessage {
    match msg {
        WireMessage::OrgInformation { envelope, org_private_key, .. } => WireMessage::OrgInformation {
            envelope: envelope.clone(),
            record_snapshot,
            org_private_key: org_private_key.clone(),
        },
        other => panic!("only Organisation information carries a snapshot, got {other:?}"),
    }
}

/// The Organisation information `msg`, with its Organisation private key
/// replaced.
pub fn with_key(msg: &WireMessage, org_private_key: OrgPrivateKey) -> WireMessage {
    match msg {
        WireMessage::OrgInformation { envelope, record_snapshot, .. } => WireMessage::OrgInformation {
            envelope: envelope.clone(),
            record_snapshot: record_snapshot.clone(),
            org_private_key,
        },
        other => panic!("only Organisation information carries a key, got {other:?}"),
    }
}

/// Stories 1–2 over the service API: A creates a persona and the organisation
/// (epoch 1); B creates a persona, declares it expects the admission (unless
/// the test says not to), and A reads the joiner it admits B as. A's endpoint is bound
/// from A's persona `device_seed`.
pub struct Setup {
    pub chain: MockChainOps,
    pub svc_a: OrgService,
    pub svc_b: OrgService,
    pub org_id: OrgId,
    pub pid_a: PersonaId,
    pub pid_b: PersonaId,
    pub b_device_kp: SigningKeypair,
    pub joiner_b: Joiner,
}

pub async fn setup(tag: &str) -> Setup {
    setup_with(tag, true).await
}

pub async fn setup_with(tag: &str, prepare: bool) -> Setup {
    setup_over_both(tag, prepare, |c| Box::new(c), |c| Box::new(c)).await
}

/// `setup_with`, with A's service reading the chain through `a_chain(chain)`
/// and B's through `b_chain(chain)`.
pub async fn setup_over_both(
    tag: &str,
    prepare: bool,
    a_chain: impl FnOnce(MockChainOps) -> Box<dyn ChainOps>,
    b_chain: impl FnOnce(MockChainOps) -> Box<dyn ChainOps>,
) -> Setup {
    let chain = MockChainOps::new();

    let mut svc_a = OrgService::new(open_store(tag, "a", "pw_a"), a_chain(chain.clone()));
    let mut svc_b = OrgService::new(open_store(tag, "b", "pw_b"), b_chain(chain.clone()));

    // Story 1: A creates persona + org.
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = found(&mut svc_a, &chain, &pid_a).await;
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(1));

    // A's outbound endpoint, bound from A's persona device seed.
    let ep_a = OrgEndpoint::bind(&device_kp(&svc_a, &pid_a)).await.unwrap();
    let svc_a = svc_a.with_endpoint(ep_a);

    // Story 2: B creates persona and prepares to join A's organisation.
    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    if prepare {
        prepare_to_join(&mut svc_b, org_id);
    }

    // The joiner A admits B as.
    let joiner_b = joiner_of(&svc_b, &pid_b);
    assert_eq!(joiner_b.handle.as_str(), "bob");

    let b_device_kp = device_kp(&svc_b, &pid_b);

    Setup { chain, svc_a, svc_b, org_id, pid_a, pid_b, b_device_kp, joiner_b }
}

/// Story 3+4 directly: A admits B, B receives from A's device and commits
/// epoch 2. Returns the setup with B's service carrying the committed record.
pub async fn admit_b_directly(mut s: Setup) -> Setup {
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;

    admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, b_addr)
        .await
        .expect("admit_member(B) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(2), "admitting B must bump to epoch 2");

    let (svc_b, outcome) = b_task.await.unwrap();
    let outcome = outcome.expect("B's direct admission from A must verify");
    assert_eq!(outcome.epoch, Epoch::new(2));
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(svc_b.list_orgs()[0].epoch, Epoch::new(2));
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 2, "A + B");
    s.svc_b = svc_b;
    s
}

/// A creates a further persona C and returns the joiner it is admitted as.
/// A's founding Persona stays the first bound to the Organisation, so A
/// still sends from its device.
pub fn joiner_for_c(svc_a: &mut OrgService) -> Joiner {
    let pid_c = svc_a.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let jr = joiner_of(svc_a, &pid_c);
    assert_eq!(jr.handle.as_str(), "carol");
    jr
}

/// A's genuine admission of `joiner`, captured by a sink rather than
/// delivered to B.
pub async fn captured_admission(s: &mut Setup, joiner: &Joiner) -> WireMessage {
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, joiner, sink_addr).await.expect("admit the joiner");
    sink.await.unwrap().2
}

/// A admits a further persona C; the admission is captured, not delivered
/// to B.
pub async fn captured_admission_of_c(s: &mut Setup) -> WireMessage {
    let joiner_c = joiner_for_c(&mut s.svc_a);
    captured_admission(s, &joiner_c).await
}

/// Send `msg` to `addr` from a relay device that is neither A nor B. No
/// record lists it, so since the owner rulings of 2026-10-07 (LLR-2r2fha,
/// LLR-kzgjz8, LLR-3aysup) a held Organisation's receiver refuses what it
/// sends: use it for first admissions and refusal cases, `deliver_from`
/// otherwise.
pub async fn deliver(addr: iroh::EndpointAddr, msg: &WireMessage) {
    deliver_from([0x5bu8; 32], addr, msg).await;
}

/// Send `msg` to `addr` from the Device of `sender_seed`.
pub async fn deliver_from(sender_seed: [u8; 32], addr: iroh::EndpointAddr, msg: &WireMessage) {
    let sender = OrgEndpoint::bind(&DeviceSeed::from(sender_seed).signing_keypair()).await.unwrap();
    tokio::time::timeout(NET, sender.send(addr, msg)).await.expect("deliver timed out").expect("deliver failed");
}

/// Send `body`, framed as `encode_frame` frames a message, to `addr` from a
/// relay device: bytes no `WireMessage` encodes to.
pub async fn deliver_raw(addr: iroh::EndpointAddr, body: &[u8]) {
    let relay = OrgEndpoint::bind(&DeviceSeed::from([0x5cu8; 32]).signing_keypair()).await.unwrap();
    let conn = relay.inner().connect(addr, org_node::transport::ALPN).await.expect("connect");
    let (mut send, _recv) = conn.open_bi().await.expect("open_bi");
    let mut framed = (body.len() as u32).to_le_bytes().to_vec();
    framed.extend_from_slice(body);
    send.write_all(&framed).await.expect("write");
    send.finish().expect("finish");
    // Hold the connection until the receiver has read the stream.
    let _ = tokio::time::timeout(NET, send.stopped()).await;
}

/// The encoded member snapshots of `trie`, as a first admission carries them.
pub fn snapshot_bytes(trie: &org_members::trie::OrgTrie<org_members::hasher::Blake3Hasher>) -> Vec<u8> {
    let members: Vec<MemberSnapshot> = trie
        .members()
        .iter()
        .map(|m| MemberSnapshot {
            id: *m.id(),
            handle: m.handle().clone(),
            name: m.name().clone(),
            surname: m.surname().clone(),
            member_key: *m.p2p_key(),
            device_keys: m.p2p_devices().to_vec(),
        })
        .collect();
    postcard::to_allocvec(&members).unwrap()
}

/// A `ChainOps` over a shared `MockChainOps` that counts `read_state` calls,
/// can hide an Organisation (answer `None` for it) and can fail every read.
#[derive(Clone)]
pub struct CountingChain {
    pub inner: MockChainOps,
    reads: Arc<AtomicUsize>,
    hidden: Arc<Mutex<Vec<OrgId>>>,
    failing: Arc<AtomicBool>,
}

impl CountingChain {
    pub fn over(inner: MockChainOps) -> Self {
        Self { inner, reads: Default::default(), hidden: Default::default(), failing: Default::default() }
    }
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }
    pub fn hide(&self, org: OrgId) {
        self.hidden.lock().unwrap().push(org);
    }
    pub fn fail_reads(&self) {
        self.failing.store(true, Ordering::SeqCst);
    }
}

#[async_trait::async_trait]
impl ChainOps for CountingChain {
    async fn read_state(&self, org: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        if self.failing.load(Ordering::SeqCst) {
            return Err(OrgNodeError::Chain("chain unreachable".into()));
        }
        if self.hidden.lock().unwrap().contains(&org) {
            return Ok(None);
        }
        self.inner.read_state(org).await
    }
}

/// `setup`, with B reading the chain through a `CountingChain`.
pub async fn setup_counted(tag: &str) -> (Setup, CountingChain) {
    let mut slot = None;
    let s = setup_over_both(tag, true, |c| Box::new(c), |c| {
        let counting = CountingChain::over(c);
        slot = Some(counting.clone());
        Box::new(counting)
    })
    .await;
    (s, slot.expect("setup_over_both calls its closures"))
}

/// `setup`, with A and B both reading the chain through one `CountingChain`:
/// its count is every chain read either node makes.
pub async fn setup_counting(tag: &str) -> (Setup, CountingChain) {
    let shared: Arc<Mutex<Option<CountingChain>>> = Default::default();
    let counting_for = |shared: Arc<Mutex<Option<CountingChain>>>| {
        move |c: MockChainOps| -> Box<dyn ChainOps> {
            let mut slot = shared.lock().unwrap();
            let counting = slot.get_or_insert_with(|| CountingChain::over(c)).clone();
            Box::new(counting)
        }
    };
    let s = setup_over_both(tag, true, counting_for(shared.clone()), counting_for(shared.clone())).await;
    let counting = shared.lock().unwrap().clone().expect("setup_over_both calls both closures");
    (s, counting)
}

/// The address `svc` receives on: its endpoint, bound from its first
/// Persona when it has none yet.
async fn receiving_addr(svc: &mut OrgService) -> iroh::EndpointAddr {
    if svc.endpoint().is_none() {
        let first = svc.list_personas()[0].persona_id.clone();
        svc.ensure_endpoint(&first).await.unwrap();
    }
    svc.endpoint().expect("an endpoint").inner().addr()
}

/// Send `msg` from an endpoint bound from `sender_seed` to `addr`, once the
/// receiver is waiting.
async fn send_from(sender_seed: [u8; 32], addr: iroh::EndpointAddr, msg: &WireMessage) {
    tokio::time::sleep(Duration::from_millis(50)).await;
    deliver_from(sender_seed, addr, msg).await;
}

/// Deliver `msg` to `svc`'s `receive_and_self_delete_if_revoked` from a
/// random relay Device no record lists, and return its result. Since the
/// owner rulings of 2026-10-07 (LLR-2r2fha, LLR-kzgjz8, LLR-3aysup) such a
/// sender is refused for every message about a held Organisation: use this
/// for the refusal cases, and `deliver_from_to_self_delete` from a listed
/// Device otherwise.
pub async fn deliver_to_self_delete(svc: &mut OrgService, msg: WireMessage) -> Result<SelfDeleteOutcome, OrgNodeError> {
    deliver_from_to_self_delete(svc, rand::random(), msg).await
}

/// Deliver `msg` to `svc`'s `receive_and_verify` from a random relay Device
/// no record lists, and return its result. Since the owner rulings of
/// 2026-10-07 (LLR-2r2fha, LLR-kzgjz8, LLR-3aysup) such a sender is refused
/// for every message about a held Organisation: use this for the refusal
/// cases and first admissions, and `deliver_from_to_receive` from a listed
/// Device otherwise.
pub async fn deliver_to_receive(svc: &mut OrgService, msg: WireMessage) -> Result<ReceiveOutcome, OrgNodeError> {
    deliver_from_to_receive(svc, rand::random(), msg).await
}

/// Deliver `msg` to `svc`'s `receive_and_self_delete_if_revoked` from the
/// Device of `sender_seed`, and return its result.
pub async fn deliver_from_to_self_delete(
    svc: &mut OrgService,
    sender_seed: [u8; 32],
    msg: WireMessage,
) -> Result<SelfDeleteOutcome, OrgNodeError> {
    let addr = receiving_addr(svc).await;
    let (result, ()) = tokio::join!(
        async { tokio::time::timeout(NET, svc.receive_and_self_delete_if_revoked(&mut OsRng)).await.expect("receive timed out") },
        send_from(sender_seed, addr, &msg)
    );
    result
}

/// Deliver `msg` to `svc`'s `receive_and_verify` from the Device of
/// `sender_seed`, and return its result.
pub async fn deliver_from_to_receive(
    svc: &mut OrgService,
    sender_seed: [u8; 32],
    msg: WireMessage,
) -> Result<ReceiveOutcome, OrgNodeError> {
    let addr = receiving_addr(svc).await;
    let (result, ()) = tokio::join!(
        async { tokio::time::timeout(NET, svc.receive_and_verify(&mut OsRng)).await.expect("receive timed out") },
        send_from(sender_seed, addr, &msg)
    );
    result
}

/// The device seed of `persona_id` in `svc`, as bytes: the Device a test
/// sends from when the message must come from that Persona's Device.
pub fn device_seed_of(svc: &OrgService, persona_id: &PersonaId) -> [u8; 32] {
    *persona_of(svc, persona_id).device_seed.expose_secret()
}

/// A calculated trie of one Member that lists neither `member` nor `device`:
/// an absence proof from it verifies under its root, not under any chain's.
pub fn trie_without(member: MemberId, device: DevicePublicKey) -> org_node::test_fixtures::Trie {
    use org_node::test_fixtures::{device_key, member_key};
    let other = MemberId::new([0x5e; 32]);
    assert_ne!(other, member);
    assert_ne!(device_key(0x5f), device);
    let leaf = org_members::MemberLeaf::new(other, h("elsewhere"), member_key(0x5e), nm("Else"), sn("Where"), vec![device_key(0x5f)])
        .unwrap();
    org_node::test_fixtures::Trie::genesis(vec![leaf]).unwrap()
}
