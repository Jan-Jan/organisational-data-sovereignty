#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests: who delivers a message to `OrgService::receive_and_verify`
//! (and to the self-delete path), over the service API, with real iroh
//! endpoints on loopback and a shared `MockChainOps`. Offline — no live chain,
//! no relay.
//!
//! The admin (A) always produces a chain-valid admission envelope. The
//! Envelope carries no signature; what varies is *who delivers it* to the
//! member (B). Since the owner's ruling of 2026-10-05 nothing about the sender
//! is checked (REQ-ag6kqm, REQ-xa6smf, REQ-ztdza4): a chain-valid first
//! admission or update is committed whichever device relays it, with or
//! without an imported Invite, and the chain decides.
//!
//! The same setup carries the MemberId tests: ids are random, not keys, and a
//! re-admission with the same keys gets a fresh id (REQ-d9g6nt).
//!
//! The gate:
//! `cargo test -p org-node --features app,test-support --test admission_sender`

use std::time::Duration;

use org_members::{Handle, MemberId, Name, Surname};
use org_node::blobs::JoinRequest;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::keys::{SigningKeypair, X25519Keypair};
use org_node::service::{ChainOps, MockChainOps, OrgService, SelfDeleteOutcome};
use org_node::store::{OrgRecord, PersonaRecord, PersonaStatus, PersonaStore};
use org_node::transport::endpoint::OrgEndpoint;
use org_node::transport::wire::WireMessage;
use org_node::{ChainAccount, DeviceSeed, Epoch, MemberSeed, OrgPublicKey, OrgSecret, PersonaId, RootHash, SequenceNumber};
use rand::rngs::OsRng;

const NET: Duration = Duration::from_secs(30);

/// The Organisation secret every admission in this file hands over.
fn org_secret() -> Option<OrgSecret> {
    Some(OrgSecret::from([0xffu8; 32]))
}

/// The rogue relay's device key — a third device, neither A's nor B's.
const ROGUE_SEED: [u8; 32] = [0x33u8; 32];

/// The store directory of one party in one test, unique per process.
fn store_dir(tag: &str, party: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "ods-admission-sender-{tag}-{party}-{}",
        std::process::id()
    ))
}

/// Fresh encrypted store under `temp_dir()`, unique per test and party.
fn open_store(tag: &str, party: &str, password: &str) -> PersonaStore {
    let dir = store_dir(tag, party);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    PersonaStore::open(dir.join("store.bin"), password).unwrap()
}

fn h(s: &str) -> Handle {
    Handle::parse(s).unwrap()
}
fn nm(s: &str) -> Name {
    Name::parse(s).unwrap()
}
fn sn(s: &str) -> Surname {
    Surname::parse(s).unwrap()
}

/// Reopen a store that `open_store` already created, WITHOUT wiping it — the
/// only way to see what actually reached the disk.
fn reopen_store(tag: &str, party: &str, password: &str) -> PersonaStore {
    PersonaStore::open(store_dir(tag, party).join("store.bin"), password).unwrap()
}

/// A persona's record, cloned.
fn persona_of(svc: &OrgService, persona_id: &PersonaId) -> PersonaRecord {
    svc.list_personas()
        .iter()
        .find(|p| &p.persona_id == persona_id)
        .expect("persona not found")
        .clone()
}

/// The device keypair of a persona, from its persisted `device_seed`.
fn device_kp(svc: &OrgService, persona_id: &PersonaId) -> SigningKeypair {
    persona_of(svc, persona_id).device_seed.signing_keypair()
}

/// The member and device keypairs of `svc`'s first persona — the
/// administrator in every fixture that calls this — from its persisted seeds.
fn admin_keys(svc: &OrgService) -> (X25519Keypair, SigningKeypair) {
    let admin = &svc.list_personas()[0];
    (admin.member_seed.x25519_keypair(), admin.device_seed.signing_keypair())
}

/// A persona's join request, exported and imported back as an administrator
/// receives it.
fn join_request_of(svc: &OrgService, persona_id: &PersonaId) -> JoinRequest {
    OrgService::import_join_request(&svc.export_join_request(persona_id).unwrap()).unwrap()
}

/// `svc`'s record of `org_id`, cloned.
fn rec_of(svc: &OrgService, org_id: OrgId) -> OrgRecord {
    svc.list_orgs().iter().find(|o| o.org_id == org_id).expect("record").clone()
}

/// The record of `org_id` as `store` read it from disk, cloned.
fn disk_rec_of(store: &PersonaStore, org_id: OrgId) -> OrgRecord {
    store.data().orgs.iter().find(|o| o.org_id == org_id).expect("record on disk").clone()
}

/// The MemberId of the member with `handle` in `rec`.
fn id_by_handle(rec: &OrgRecord, handle: &str) -> MemberId {
    rec.trie_members.iter().find(|m| m.handle.as_str() == handle).expect("member in the record").id
}

/// A well-formed peer identity, from `seed`, carrying NO transport addresses.
/// The call sites say why a dial to it fails at once.
fn dead_addr(seed: [u8; 32]) -> iroh::EndpointAddr {
    iroh::EndpointId::from_bytes(DeviceSeed::from(seed).signing_keypair().device_key().unwrap().as_bytes())
        .map(|id| iroh::EndpointAddr::from_parts(id, std::iter::empty()))
        .expect("a well-formed endpoint identity")
}

/// Stories 1–2 over the service API: A creates a persona and the organisation
/// (epoch 1); B creates a persona, imports A's invite (unless the test says
/// not to) and exports a join request that A imports. A's endpoint is bound
/// from A's persona `device_seed`.
struct Setup {
    chain: MockChainOps,
    svc_a: OrgService,
    svc_b: OrgService,
    org_id: OrgId,
    pid_b: PersonaId,
    b_device_kp: SigningKeypair,
    join_request_b: JoinRequest,
}

async fn setup(tag: &str) -> Setup {
    setup_with(tag, true).await
}

async fn setup_with(tag: &str, import_invite: bool) -> Setup {
    let chain = MockChainOps::new();

    let mut svc_a = OrgService::new(open_store(tag, "a", "pw_a"), Box::new(chain.clone()));
    let mut svc_b = OrgService::new(open_store(tag, "b", "pw_b"), Box::new(chain.clone()));

    // Story 1: A creates persona + org.
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = svc_a.create_organisation(&mut OsRng, &pid_a).await.unwrap();
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(1));

    // A's outbound endpoint, bound from A's persona device seed.
    let ep_a = OrgEndpoint::bind(&device_kp(&svc_a, &pid_a)).await.unwrap();
    let svc_a = svc_a.with_endpoint(ep_a);

    // Story 2: B creates persona; A exports the invite; B imports it.
    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    if import_invite {
        let invite_blob = svc_a.export_invite(org_id).unwrap();
        let invite = svc_b.import_invite(&mut OsRng, &invite_blob).unwrap();
        assert_eq!(invite.org_id, org_id);
    }

    // B exports a join request; A imports it.
    let join_request_b = join_request_of(&svc_b, &pid_b);
    assert_eq!(join_request_b.handle.as_str(), "bob");

    let b_device_kp = device_kp(&svc_b, &pid_b);

    Setup { chain, svc_a, svc_b, org_id, pid_b, b_device_kp, join_request_b }
}

/// Bind a fresh receiving endpoint for B (from B's device key), install it on
/// B's service and spawn `receive_and_verify`. Returns the address the sender
/// must dial and the task handle yielding `(svc_b, result)`.
async fn spawn_b_receive(
    svc_b: OrgService,
    b_device_kp: &SigningKeypair,
) -> (
    iroh::EndpointAddr,
    tokio::task::JoinHandle<(OrgService, Result<org_node::service::ReceiveOutcome, OrgNodeError>)>,
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

/// Bind an endpoint from `seed` and spawn `recv_one`. The task yields the
/// endpoint back with the authenticated sender and the message it received:
/// a rogue relay R (`ROGUE_SEED`) relays the message; a sink (random seed)
/// holds the endpoint open until the sender's `send` has returned, since
/// dropping it on receipt can lose the acknowledgement the sender waits for.
async fn spawn_recv_one(
    seed: [u8; 32],
) -> (
    iroh::EndpointAddr,
    tokio::task::JoinHandle<(OrgEndpoint, org_members::DevicePublicKey, WireMessage)>,
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

/// Story 3+4 directly: A admits B, B receives from A's device and commits
/// epoch 2. Returns the setup with B's service carrying the committed record.
async fn admit_b_directly(mut s: Setup) -> Setup {
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;

    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, b_addr, org_secret()),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(2), "admitting B must bump to epoch 2");

    let (svc_b, outcome) = b_task.await.unwrap();
    let outcome = outcome.expect("B's direct admission from A must verify");
    assert_eq!(outcome.epoch, Epoch::new(2));
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(svc_b.list_orgs()[0].epoch, Epoch::new(2));
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 2, "admin + B");
    assert_eq!(
        svc_b.list_orgs()[0].admin_member_key,
        s.svc_a.list_orgs()[0].admin_member_key,
        "B's record names the administrator the invite named, not a chain key"
    );
    s.svc_b = svc_b;
    s
}

/// A creates a further persona C and imports its join request. A's admin
/// persona is still found by member key, so A stays the org's admin.
fn join_request_for_c(svc_a: &mut OrgService) -> JoinRequest {
    let pid_c = svc_a.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let jr = join_request_of(svc_a, &pid_c);
    assert_eq!(jr.handle.as_str(), "carol");
    jr
}

// REQ-xa6smf as amended 2026-10-05: a first admission that verifies against
// the chain is committed whichever device delivers it. A rogue relay R
// forwards A's genuine admission of B; B commits it and consumes its Invite.
// verifies: REQ-xa6smf, LLR-j83kc8, LLR-9fvb3y
#[tokio::test(flavor = "multi_thread")]
async fn first_admission_from_a_device_other_than_the_invites_admin_is_committed() {
    let mut s = setup("first-rogue").await;

    // R waits for A's push; A "admits B" but is handed R's address.
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, r_addr, org_secret()),
    )
    .await
    .expect("admit_member(B via R) timed out")
    .expect("admit_member(B via R) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(2));

    let (ep_r, sender_seen_by_r, msg) = r_task.await.unwrap();
    assert_eq!(
        sender_seen_by_r,
        device_kp(&s.svc_a, &s.svc_a.list_personas()[0].persona_id).device_key().unwrap(),
        "R received the admission from A's device"
    );
    assert!(msg.org_secret.is_some() && msg.genesis_snapshot.is_some(), "a full admission message");

    // R relays the identical message to B.
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, ep_r.send(b_addr, &msg))
        .await
        .expect("R relay send timed out")
        .expect("R relay send failed");

    let (svc_b, result) = b_task.await.unwrap();
    let outcome = result.expect("a chain-valid first admission is committed whoever relays it");
    assert_eq!(outcome.org_id, s.org_id);
    assert_eq!(outcome.epoch, Epoch::new(2));
    assert_eq!(svc_b.list_orgs().len(), 1, "B committed the OrgRecord");
    assert_eq!(persona_of(&svc_b, &s.pid_b).status, PersonaStatus::Active);
    assert!(svc_b.list_pending_invites().is_empty(), "the Invite is consumed by the commit");
}

// Normal case of an update: after B has committed epoch 2, a further update
// (C's admission) pushed by the admin's own device is verified and committed.
// It is also LLR-9f5hmr's normal case (the Sequence number equals the chain's
// epoch, and the mark becomes that epoch).
// verifies: REQ-ztdza4, LLR-u6rq4s, LLR-cja9zv, LLR-9f5hmr
#[tokio::test(flavor = "multi_thread")]
async fn update_from_the_admin_after_admission_is_committed() {
    let mut s = admit_b_directly(setup("update-admin").await).await;
    let seq_at_2 = s.svc_b.list_orgs()[0].last_seq;

    let jr_c = join_request_for_c(&mut s.svc_a);

    // B waits for the next update; A admits C, pushing the envelope to B.
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, org_secret()))
        .await
        .expect("admit_member(C) timed out")
        .expect("admit_member(C) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(3), "admitting C must bump to epoch 3");

    let (svc_b, result) = b_task.await.unwrap();
    let outcome = result.expect("B must accept an update sent by the admin's own device");
    assert_eq!(outcome.org_id, s.org_id);
    assert_eq!(outcome.epoch, Epoch::new(3), "B must commit epoch 3");
    assert_eq!(
        outcome.root,
        s.chain.get(&s.org_id).unwrap().root_hash,
        "B's committed root must match the on-chain root"
    );
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(svc_b.list_orgs()[0].epoch, Epoch::new(3));
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 3, "admin + B + C");
    // The sequence mark advances with the commit. Added 2026-10-03: this is
    // the UPDATE branch of the commit, distinct from the first-admission
    // branch that `a_committed_admission_reaches_the_disk_and_consumes_the_invite`
    // covers, and the sweep found zeroing it here reddened nothing. Compared
    // against the mark held BEFORE this update, so the assertion cannot be
    // satisfied by the value it is reading.
    assert!(
        svc_b.list_orgs()[0].last_seq > seq_at_2,
        "the mark must advance with the commit: was {seq_at_2:?}, now {:?}",
        svc_b.list_orgs()[0].last_seq
    );
    assert_eq!(svc_b.list_orgs()[0].last_seq, SequenceNumber::new(3), "the mark is the epoch committed");
}

// REQ-ztdza4 and REQ-ag6kqm as amended 2026-10-05: an update that verifies
// against the chain is committed whichever device relays it. A rogue device R
// relays A's genuine admission of C; B commits epoch 3.
// verifies: REQ-ag6kqm, REQ-ztdza4, LLR-u6rq4s, LLR-37cj3n, LLR-9fvb3y
#[tokio::test(flavor = "multi_thread")]
async fn update_relayed_by_a_non_member_after_admission_is_committed() {
    let mut s = admit_b_directly(setup("update-rogue").await).await;
    let root_at_2 = s.svc_b.list_orgs()[0].root_hash;

    let jr_c = join_request_for_c(&mut s.svc_a);

    // R captures A's C-admission push.
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(NET, s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, r_addr, org_secret()))
        .await
        .expect("admit_member(C via R) timed out")
        .expect("admit_member(C via R) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(3));
    let (ep_r, _sender, msg) = r_task.await.unwrap();

    // R relays it to B.
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, ep_r.send(b_addr, &msg))
        .await
        .expect("R relay send timed out")
        .expect("R relay send failed");

    let (svc_b, result) = b_task.await.unwrap();
    let outcome = result.expect("a chain-valid update is committed whoever relays it");
    assert_eq!(outcome.epoch, Epoch::new(3));
    assert_eq!(svc_b.list_orgs()[0].epoch, Epoch::new(3));
    assert_eq!(svc_b.list_orgs()[0].last_seq, SequenceNumber::new(3), "the mark is the epoch committed");
    assert_ne!(svc_b.list_orgs()[0].root_hash, root_at_2, "B's root moved to the chain's");
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 3, "admin + B + C");
}

// ---- added 2026-10-03 by the architecture tooth ----------------------------

/// Bind a fresh receiving endpoint for B and spawn the **self-delete** receive
/// instead of the ordinary one. Mirrors `spawn_b_receive`.
async fn spawn_b_self_delete(
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

// On the removal path nothing about the sender is checked (REQ-ztdza4 and
// LLR-3q63zv, amended 2026-10-05 by owner ruling): a revocation relayed by a
// rogue device R is acted on, because the chain decides whether the removal
// is real. This branch refused it for a while (a sender check the owner
// retired the same day); this is master's test, restored.
// verifies: LLR-3q63zv, LLR-6p4pj2, LLR-9fvb3y
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_relayed_by_a_non_member_is_still_acted_on() {
    let mut s = admit_b_directly(setup("revoke-relay").await).await;
    let b_member_id = s.svc_b.list_personas()[0].member_id.expect("B has a member id");

    // R stands between A and B: A revokes B and sends the message to R's
    // address; R captures it and relays it on to B.
    let (rogue_addr, rogue_task) = spawn_recv_one(ROGUE_SEED).await;
    let (b_addr, b_task) = spawn_b_self_delete(s.svc_b, &s.b_device_kp).await;

    tokio::time::timeout(
        NET,
        s.svc_a.revoke_member(&mut OsRng, s.org_id, b_member_id, Some(rogue_addr)),
    )
    .await
    .expect("revoke_member timed out")
    .expect("revoke_member failed");

    let (rogue_ep, sender, captured) = rogue_task.await.unwrap();
    assert_eq!(
        sender.as_bytes(),
        s.svc_a.endpoint().expect("A endpoint").device_key().as_bytes(),
        "R must have received the revocation from A itself"
    );
    rogue_ep.send(b_addr, &captured).await.expect("rogue relay to B failed");

    let (svc_b, outcome) = b_task.await.unwrap();
    let outcome = outcome.expect("B must act on a chain-valid revocation whoever relayed it");
    assert!(
        matches!(outcome, SelfDeleteOutcome::SelfDeleted { .. }),
        "expected SelfDeleted, got {outcome:?}"
    );
    assert!(svc_b.list_orgs().is_empty(), "B's record of the org must be gone");
    assert_eq!(svc_b.list_personas()[0].status, PersonaStatus::Revoked);
}

// `receive_and_verify` reads the chain before it consults any local record, so
// a message naming an Organisation the chain knows nothing about is refused
// with OrgNotOnChain — not with a "no such record" error and not by falling
// through to the first-admission path.
// verifies: REQ-bvh8v6, LLR-3wb7th
#[tokio::test(flavor = "multi_thread")]
async fn a_message_for_an_organisation_absent_from_the_chain_is_refused() {
    let s = setup("absent-org").await;
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;

    // A well-formed message for an org that was never put on the mock chain.
    // B holds no record of it either, so if the chain read did NOT come first
    // this would be taken for a first admission.
    let stranger = MemberSeed::from([0x5au8; 32]).x25519_keypair();
    let (delta, _) = org_node::test_fixtures::admit_member_delta(&stranger);
    let absent_org = OrgId::new([0xeeu8; 20]);
    let envelope = org_node::Envelope::build(absent_org, SequenceNumber::new(1), &delta).unwrap();
    let msg = WireMessage { envelope, org_secret: None, genesis_snapshot: None };

    let relay = OrgEndpoint::bind(&DeviceSeed::from([0x5bu8; 32]).signing_keypair()).await.unwrap();
    relay.send(b_addr, &msg).await.expect("send to B failed");

    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert!(svc_b.list_orgs().is_empty(), "nothing may be committed");
}

// The committing half of `receive_and_verify`, which the falsifiability sweep
// of 2026-10-03 found almost unevidenced: deleting `self.store.save(rng)?`,
// writing a zero sequence mark, and never consuming the pending invite all
// left the suite green. Everything asserted here is read back from DISK, so a
// commit that only ever existed in memory fails it.
// verifies: REQ-nhe2zu, REQ-txvtm9, REQ-xa6smf, LLR-cja9zv, LLR-q8emds
#[tokio::test(flavor = "multi_thread")]
async fn a_committed_admission_reaches_the_disk_and_consumes_the_invite() {
    let s = setup("commit-persisted").await;
    // Before admission B holds the invite it imported from A.
    assert_eq!(
        s.svc_b.list_personas().len(),
        1,
        "B starts with exactly its own persona"
    );

    let s = admit_b_directly(s).await;

    // In memory, B committed epoch 2 with a non-zero sequence mark.
    let in_memory = s.svc_b.list_orgs()[0].clone();
    assert_eq!(in_memory.epoch, Epoch::new(2));
    assert_ne!(in_memory.last_seq, SequenceNumber::new(0), "the sequence mark must have been written");
    assert_eq!(in_memory.last_seq.get(), in_memory.epoch.get(), "the mark is the epoch committed (REQ-txvtm9)");

    // On disk, the same record — this is what `self.store.save(rng)?` is for.
    let reloaded = reopen_store("commit-persisted", "b", "pw_b");
    let orgs = &reloaded.data().orgs;
    assert_eq!(orgs.len(), 1, "the committed record must have reached the disk");
    assert_eq!(orgs[0].org_id, in_memory.org_id);
    assert_eq!(orgs[0].epoch, in_memory.epoch, "epoch must be persisted");
    assert_eq!(
        orgs[0].last_seq, in_memory.last_seq,
        "the sequence mark must be persisted, not left at its default"
    );
    assert_eq!(
        orgs[0].trie_members.len(),
        2,
        "admin + B must be persisted as snapshots"
    );

    // The pending invite is consumed, and that too reached the disk.
    assert!(
        reloaded.data().pending_invites.is_empty(),
        "the invite must be consumed once the first admission has committed"
    );
}

// The administrator's own side of an admission: A updates and persists its
// Organisation record. Added 2026-10-04 after review round 1, which found that
// deleting `self.store.save(rng)?` from `admit_member` left the whole gate
// green — the twin, on the write path, of the gap the sweep had already found
// and closed on the receive path.
// verifies: LLR-t4znbk, LLR-rb8r65, LLR-ghja3x
#[tokio::test(flavor = "multi_thread")]
async fn an_admission_reaches_the_administrators_disk() {
    let s = admit_b_directly(setup("admin-persisted").await).await;

    let in_memory = s.svc_a.list_orgs()[0].clone();
    assert_eq!(in_memory.epoch, Epoch::new(2), "A must hold the epoch it submitted");
    assert_eq!(in_memory.trie_members.len(), 2, "admin + B");
    assert_ne!(in_memory.last_seq, SequenceNumber::new(0), "A must hold the sequence it signed over");

    let reloaded = reopen_store("admin-persisted", "a", "pw_a");
    let orgs = &reloaded.data().orgs;
    assert_eq!(orgs.len(), 1, "A's record must reach the disk");
    assert_eq!(orgs[0].epoch, in_memory.epoch, "A's epoch must be persisted");
    assert_eq!(
        orgs[0].last_seq, in_memory.last_seq,
        "A's sequence mark must be persisted"
    );
    assert_eq!(
        orgs[0].trie_members.len(),
        2,
        "A must persist the member it just added"
    );
    // The member it added is B, not a placeholder.
    assert!(
        orgs[0].trie_members.iter().any(|m| m.handle.as_str() == "bob"),
        "B must be among A's persisted members"
    );

    // The root A records is the one it published. Added 2026-10-04 by review
    // round 6, which measured that leaving `org_rec.root_hash` unwritten was
    // green: nothing in the library reads it, so only a test can.
    let published = s.chain.get(&s.org_id).unwrap().root_hash;
    assert_eq!(in_memory.root_hash, published, "A's record must hold the root it published");
    assert_eq!(orgs[0].root_hash, published, "and so must A's disk");
}

// The ordering clause of LLR-t4znbk: A updates its record only AFTER the
// message has been sent, so a send that fails leaves A's record where it was.
// Added 2026-10-04 after review round 1, which found that moving the whole
// update-and-save block above the send left the gate green.
//
// The send is made to fail by dialling a well-formed peer identity carrying
// NO transport addresses, as the body below sets out.
//
// *Corrected 2026-10-04 by review round 4.* These two lines used to describe
// a loopback port with nothing bound to it — the discarded first draft of
// round 1's fix, left standing above the fix itself, and contradicted in terms
// by the comment ten lines down saying that mechanism does NOT work because
// QUIC retries it for thirty seconds. The revocation twin's comment was
// correct; this one was not.
// verifies: LLR-t4znbk
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_push_leaves_the_administrators_record_where_it_was() {
    let mut s = setup("admin-send-fails").await;
    let epoch_before = s.chain.get(&s.org_id).unwrap().epoch;
    let orgs_before = s.svc_a.list_orgs()[0].clone();

    // A well-formed peer identity with NO transport addresses at all. The
    // sending endpoint is Loopback mode, so the relay is disabled and there is
    // no address discovery: iroh has no path to try and refuses immediately.
    // (Dialling a loopback port nothing listens on does NOT work here — QUIC
    // retries it for thirty seconds, which is slower than this gate tolerates.)
    let result = tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, dead_addr([0x6fu8; 32]), org_secret()),
    )
    .await
    .expect("admit_member to a dead peer timed out");

    assert!(result.is_err(), "a push to a peer that is not listening must fail");

    // The chain moved — `submit_update` runs before the send, and that is the
    // documented order. What must NOT have moved is A's own record.
    assert_eq!(
        s.chain.get(&s.org_id).unwrap().epoch,
        Epoch::new(epoch_before.get() + 1),
        "the chain submission precedes the send and is expected to have landed"
    );
    let orgs_after = s.svc_a.list_orgs()[0].clone();
    assert_eq!(
        orgs_after.epoch, orgs_before.epoch,
        "A's recorded epoch must not advance on a failed push"
    );
    assert_eq!(
        orgs_after.last_seq, orgs_before.last_seq,
        "A's sequence mark must not advance on a failed push"
    );
    assert_eq!(
        orgs_after.trie_members.len(),
        orgs_before.trie_members.len(),
        "A must not record a member it failed to notify"
    );
}

// Normal case of REQ-d9g6nt: the founding admin's id and an admitted
// member's id are not their keys, and differ from each other.
// verifies: REQ-d9g6nt, LLR-ag9mgm, LLR-rjg3m2
#[tokio::test(flavor = "multi_thread")]
async fn member_ids_are_not_derived_from_keys() {
    let mut s = setup("ids-not-keys").await;

    let admin_snap = s.svc_a.list_orgs()[0].trie_members[0].clone();
    assert_eq!(admin_snap.member_key, s.svc_a.list_orgs()[0].admin_member_key);
    assert_ne!(admin_snap.id.as_bytes(), admin_snap.member_key.as_bytes(), "the admin's id is its member key");
    assert_ne!(admin_snap.id.as_bytes(), admin_snap.device_keys[0].as_bytes(), "the admin's id is its device key");

    // A admits B, delivered to B's own endpoint as in `admit_b_directly`.
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    let id_b = tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, b_addr, org_secret()),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");
    let (_svc_b, outcome) = b_task.await.unwrap();
    outcome.expect("B's admission from A must verify");

    assert_ne!(id_b.as_bytes(), s.join_request_b.member_key.as_bytes(), "B's id is its member key");
    assert_ne!(id_b.as_bytes(), s.join_request_b.device_key.as_bytes(), "B's id is its device key");
    assert_ne!(id_b, admin_snap.id, "B's id equals the admin's id");
    assert!(
        s.svc_a.list_orgs()[0].trie_members.iter().any(|m| m.id == id_b),
        "A's record holds B under the returned id"
    );
}

// Abnormal case of REQ-d9g6nt: admit B, revoke B by MemberId, admit the
// SAME join request (same member key and device key) again. Re-admission
// succeeds (owner ruling: same keys allowed), and each new id differs from
// every deleted id and from every key. Three rounds.
// verifies: REQ-d9g6nt, LLR-ag9mgm
#[tokio::test(flavor = "multi_thread")]
async fn readmission_with_same_keys_gets_a_fresh_member_id() {
    let mut s = setup("readmit-same-keys").await;
    let admin_id = s.svc_a.list_orgs()[0].trie_members[0].id;

    let mut ids: Vec<MemberId> = Vec::new();
    for round in 0..3 {
        // Admit the same join request.
        let (addr, sink) = spawn_recv_one(rand::random()).await;
        let id = tokio::time::timeout(
            NET,
            s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, addr, org_secret()),
        )
        .await
        .expect("admit_member(B) timed out")
        .unwrap_or_else(|e| panic!("admission round {round} with the same keys failed: {e:?}"));
        sink.await.unwrap();

        assert_ne!(id.as_bytes(), s.join_request_b.member_key.as_bytes(), "round {round}: id is B's member key");
        assert_ne!(id.as_bytes(), s.join_request_b.device_key.as_bytes(), "round {round}: id is B's device key");
        assert_ne!(id, admin_id, "round {round}: id is the admin's id");
        assert!(!ids.contains(&id), "round {round}: re-admission reused a deleted id");
        let members = &s.svc_a.list_orgs()[0].trie_members;
        assert!(members.iter().any(|m| m.id == id), "round {round}: A's record lacks the new id");
        for old in &ids {
            assert!(
                !members.iter().any(|m| m.id == *old),
                "round {round}: A's record still holds a deleted id"
            );
        }
        ids.push(id);

        // Revoke B by MemberId.
        let (addr, sink) = spawn_recv_one(rand::random()).await;
        tokio::time::timeout(NET, s.svc_a.revoke_member(&mut OsRng, s.org_id, id, Some(addr)))
            .await
            .expect("revoke_member(B) timed out")
            .expect("revoke_member(B) failed");
        sink.await.unwrap();
        assert!(
            !s.svc_a.list_orgs()[0].trie_members.iter().any(|m| m.id == id),
            "round {round}: revoked id still in A's record"
        );
    }
}

// Normal case of REQ-d9g6nt for the founding administrator: one persona
// founds two organisations with the same keys. Any id computed from those
// keys would be the same in both records; drawn at random, the two differ.
// verifies: REQ-d9g6nt, LLR-rjg3m2
#[tokio::test(flavor = "multi_thread")]
async fn same_persona_founding_two_organisations_gets_two_admin_ids() {
    let chain = MockChainOps::new();
    let mut svc = OrgService::new(open_store("two-orgs", "a", "pw_a"), Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();

    let org_1 = svc.create_organisation(&mut OsRng, &pid).await.unwrap();
    let org_2 = svc.create_organisation(&mut OsRng, &pid).await.unwrap();
    assert_ne!(org_1, org_2);

    let admin_of = |org_id: OrgId| {
        let rec = svc.list_orgs().iter().find(|o| o.org_id == org_id).unwrap();
        assert_eq!(rec.trie_members.len(), 1, "a new organisation holds its admin only");
        rec.trie_members[0].clone()
    };
    let (admin_1, admin_2) = (admin_of(org_1), admin_of(org_2));
    assert_eq!(admin_1.member_key, admin_2.member_key, "same persona, same member key");
    assert_eq!(admin_1.device_keys, admin_2.device_keys, "same persona, same device key");
    assert_ne!(
        admin_1.id, admin_2.id,
        "the same keys founding two organisations produced the same admin id"
    );
}

// Abnormal case of REQ-d9g6nt over the service: a fresh node (B, no record
// of the org) receives a first admission whose `genesis_snapshot` is absent.
// The envelope is A's genuine, chain-valid admission of B with the snapshot
// stripped, delivered from A's own device. B must refuse it with the
// explicit error — not reconstruct a record — and commit nothing.
// verifies: REQ-d9g6nt, LLR-j6j95z
#[tokio::test(flavor = "multi_thread")]
async fn first_admission_without_a_record_snapshot_is_refused() {
    let mut s = setup("no-snapshot").await;

    // Capture A's genuine admission of B.
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, r_addr, org_secret()),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");
    let (_ep_r, _sender, mut msg) = r_task.await.unwrap();
    assert!(msg.genesis_snapshot.is_some(), "A sent a snapshot to strip");
    msg.genesis_snapshot = None;

    // A's own endpoint delivers the stripped message.
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    let ep_a = s.svc_a.endpoint().expect("A's endpoint is bound");
    tokio::time::timeout(NET, ep_a.send(b_addr, &msg))
        .await
        .expect("send timed out")
        .expect("send failed");

    let (svc_b, result) = b_task.await.unwrap();
    assert!(
        matches!(&result, Err(OrgNodeError::Chain(m)) if m == "first admission without a record snapshot"),
        "B must refuse a snapshot-less first admission explicitly, got {result:?}"
    );
    assert!(svc_b.list_orgs().is_empty(), "B must not have committed an OrgRecord");
    let persona_b = persona_of(&svc_b, &s.pid_b);
    assert_ne!(persona_b.status, PersonaStatus::Active, "B's persona must not be Active");
    assert_eq!(persona_b.org_id, None);
}

// ---------------------------------------------------------------------------
// Added 2026-10-04 after review round 2.
//
// Round 1 found that `admit_member` and `create_organisation` could lose their
// `self.store.save(rng)?` with the whole gate staying green, and closed those
// two sites. Round 2 swept the rest of the class and found four more — the
// revocation path, both arms of the self-delete path, and `import_invite` —
// so the fix had closed the sites that were named rather than the class they
// belonged to. These tests close the class: every one of the unit's eight
// persistence sites is now read back from DISK by some named test.
// ---------------------------------------------------------------------------

// The administrator's own side of a revocation. Deleting `self.store.save`
// from `revoke_member` left the entire gate green: A's record of the removal
// never had to reach the disk, so an administrator that restarted recovered
// the member it had just removed on chain.
// verifies: LLR-6dc598, LLR-tax3pm, LLR-qg9utu, LLR-pw369n, LLR-8hdu9x
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_reaches_the_administrators_disk() {
    let mut s = admit_b_directly(setup("revoke-persisted").await).await;
    let before = s.svc_a.list_orgs()[0].clone();
    assert_eq!(before.trie_members.len(), 2, "admin + B before the revocation");
    let b_id = id_by_handle(&before, "bob");

    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    tokio::time::timeout(NET, s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id, Some(sink_addr)))
        .await
        .expect("revoke_member timed out")
        .expect("revoke_member failed");
    let (_sink_ep, _sender, msg) = sink.await.unwrap();

    // LLR-8hdu9x: the Wire message carries the member snapshots as they were
    // BEFORE the removal, and no Organisation secret. Added 2026-10-05 by
    // review round 8: emptying the snapshot was green, and the secret was
    // observed only by the PR-xwek5e pin.
    let sent: Vec<org_node::store::MemberSnapshot> =
        postcard::from_bytes(msg.genesis_snapshot.as_deref().expect("a snapshot is sent"))
            .expect("the snapshot decodes");
    assert_eq!(sent.len(), 2, "admin + B: the record before the removal");
    assert_eq!(
        msg.genesis_snapshot,
        Some(postcard::to_allocvec(&before.trie_members).unwrap()),
        "the snapshot is the record before the removal"
    );
    assert_eq!(msg.org_secret, None, "a revocation carries no Organisation secret");

    // LLR-tax3pm's sequence clause, read off the envelope A transmitted
    // rather than off A's own record. *Amended 2026-10-05 (review round 1,
    // finding-1):* LLR-tax3pm, as amended, states the Sequence number is the
    // epoch the revocation's chain update produced, which is past the record's
    // last. Its signing clause (the administrator's MEMBER key signs) does not
    // hold on this branch — the Envelope carries no signature (REQ-ag6kqm) — so the
    // two `verify_signature` assertions master had here are not carried.
    assert_eq!(
        msg.envelope.parent_seq,
        SequenceNumber::new(s.chain.get(&s.org_id).unwrap().epoch.get()),
        "the revocation envelope must carry the epoch its chain update produced"
    );
    assert!(msg.envelope.parent_seq > before.last_seq, "past the record's last mark");

    // LLR-6dc598: the chain moved before the message was sent, so the anchor
    // the receiver will check against existed when the message arrived.
    assert_eq!(
        s.chain.get(&s.org_id).unwrap().epoch,
        Epoch::new(before.epoch.get() + 1),
        "the on-chain root must advance for a revocation"
    );

    let in_memory = s.svc_a.list_orgs()[0].clone();
    assert_eq!(in_memory.trie_members.len(), 1, "A holds itself only after revoking B");
    assert_eq!(in_memory.epoch, Epoch::new(before.epoch.get() + 1), "the revocation advanced the epoch");
    assert_eq!(
        in_memory.last_seq,
        SequenceNumber::new(before.last_seq.get() + 1),
        "the revocation envelope's mark is one past the record's last"
    );

    let reloaded = reopen_store("revoke-persisted", "a", "pw_a");
    let orgs = &reloaded.data().orgs;
    assert_eq!(orgs.len(), 1, "A's record must still be on disk");
    assert_eq!(
        orgs[0].trie_members.len(),
        1,
        "the removal must reach the disk, not only memory"
    );
    assert!(
        !orgs[0].trie_members.iter().any(|m| m.id == b_id),
        "A's persisted record still holds the member it revoked"
    );
    assert_eq!(orgs[0].epoch, in_memory.epoch, "the advanced epoch must be persisted");
    assert_eq!(
        orgs[0].last_seq, in_memory.last_seq,
        "the advanced sequence mark must be persisted"
    );

    // As for the admission: the root A records after a revocation is the one
    // it published. Added 2026-10-04 by review round 6.
    let published = s.chain.get(&s.org_id).unwrap().root_hash;
    assert_eq!(in_memory.root_hash, published, "A's record must hold the root it published");
    assert_eq!(orgs[0].root_hash, published, "and so must A's disk");
}

// The revoked node's own side. Deleting `self.store.save` from the self-delete
// arm left the gate green, so on the evidence the gate produced, a revoked
// node that restarted recovered the Organisation record, the member snapshots
// and — with them — the Organisation secret it had just been cut off from.
// That is the material one of the four: LLR-6p4pj2, REQ-uxv2x2 and RC-wqgm2p
// are about a record ceasing to exist, and a deletion that reaches no disk has
// not happened.
// verifies: REQ-uxv2x2, LLR-6p4pj2
#[tokio::test(flavor = "multi_thread")]
async fn a_self_delete_reaches_the_revoked_nodes_disk() {
    let mut s = admit_b_directly(setup("selfdel-persisted").await).await;
    let b_member_id = s.svc_b.list_personas()[0].member_id.expect("B has a member id");
    assert_eq!(s.svc_b.list_orgs().len(), 1, "B holds the record before the revocation");

    let (b_addr, b_task) = spawn_b_self_delete(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a.revoke_member(&mut OsRng, s.org_id, b_member_id, Some(b_addr)),
    )
    .await
    .expect("revoke_member timed out")
    .expect("revoke_member failed");

    let (svc_b, outcome) = b_task.await.unwrap();
    assert!(
        matches!(
            outcome.expect("B must act on its own revocation"),
            SelfDeleteOutcome::SelfDeleted { .. }
        ),
        "B must self-delete"
    );
    assert!(svc_b.list_orgs().is_empty(), "B's record is gone in memory");
    assert_eq!(svc_b.list_personas()[0].status, PersonaStatus::Revoked);

    let reloaded = reopen_store("selfdel-persisted", "b", "pw_b");
    assert!(
        reloaded.data().orgs.is_empty(),
        "the deletion must reach the disk — a restart must not recover the record"
    );
    assert_eq!(
        reloaded.data().personas[0].status,
        PersonaStatus::Revoked,
        "the revoked status must reach the disk too"
    );
}

// The other arm of the same function: a revocation that removes somebody else
// is committed as an ordinary update, and that update must reach the disk as
// well — the `save` in the UpdatedNotRevoked branch was separately green.
//
// *Extended 2026-10-04 by review round 5*, which measured that dropping the
// sequence-mark write from that branch left the gate green: nothing compared
// `last_seq` before and after. A mark that does not move is a replay window —
// the same change offered again would pass the replay check.
// verifies: REQ-uxv2x2, LLR-jsx922, LLR-sxd3tg
#[tokio::test(flavor = "multi_thread")]
async fn an_update_that_does_not_revoke_us_reaches_the_disk() {
    let mut s = admit_b_directly(setup("update-persisted").await).await;
    let b_epoch_before = s.svc_b.list_orgs()[0].epoch;
    let b_seq_before = s.svc_b.list_orgs()[0].last_seq;

    // A admits C (a third member) and sends the change to B, which is still a
    // member of the trie the change applies to. B must take this through the
    // self-delete entry point as an ordinary update.
    let jr_c = join_request_for_c(&mut s.svc_a);
    let (b_addr, b_task) = spawn_b_self_delete(s.svc_b, &s.b_device_kp).await;
    let c_id = tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, org_secret()),
    )
    .await
    .expect("admit_member(C) timed out")
    .expect("admit_member(C) failed");

    let (svc_b, outcome) = b_task.await.unwrap();
    assert!(
        matches!(
            outcome.expect("B must commit a change that does not remove it"),
            SelfDeleteOutcome::UpdatedNotRevoked { .. }
        ),
        "B must take this as an update, not a self-delete"
    );
    let in_memory = svc_b.list_orgs()[0].clone();
    assert!(in_memory.epoch > b_epoch_before, "B's epoch must have advanced");
    assert!(in_memory.last_seq > b_seq_before, "B's sequence mark must have advanced");
    assert!(
        in_memory.trie_members.iter().any(|m| m.id == c_id),
        "B's record must now hold C"
    );
    assert_eq!(in_memory.trie_members.len(), 3, "admin + B + C");

    let reloaded = reopen_store("update-persisted", "b", "pw_b");
    let orgs = &reloaded.data().orgs;
    assert_eq!(orgs.len(), 1, "B's record must still be on disk");
    assert_eq!(orgs[0].epoch, in_memory.epoch, "the advanced epoch must reach the disk");
    assert_eq!(orgs[0].last_seq, in_memory.last_seq, "the advanced mark must reach the disk");
    // LLR-sxd3tg's root clause (review round 7: only a test traced elsewhere
    // observed it).
    let published = s.chain.get(&s.org_id).unwrap().root_hash;
    assert_eq!(in_memory.root_hash, published, "B's record must take the published root");
    assert_eq!(orgs[0].root_hash, published);
    assert_eq!(
        orgs[0].trie_members.len(),
        in_memory.trie_members.len(),
        "the updated membership must reach the disk"
    );
    assert!(
        orgs[0].trie_members.iter().any(|m| m.id == c_id),
        "the member the update added must reach B's disk, not only its memory"
    );
}

// The Invite `import_invite` stores is what a first admission consumes and
// takes its administrator key from (LLR-j83kc8, LLR-rys5nx). An invite that
// reaches no disk does not survive a restart, and deleting this `save` left
// the gate green.
// verifies: LLR-9zfnmb
#[tokio::test(flavor = "multi_thread")]
async fn an_imported_invite_reaches_the_joiners_disk() {
    let mut s = setup("invite-persisted").await;

    let in_memory = s.svc_b.list_pending_invites().to_vec();
    assert_eq!(in_memory.len(), 1, "B holds exactly one pending invite");
    assert_eq!(in_memory[0].org_id, s.org_id);

    let reloaded = reopen_store("invite-persisted", "b", "pw_b");
    let on_disk = &reloaded.data().pending_invites;
    assert_eq!(on_disk.len(), 1, "the imported invite must reach the disk");
    assert_eq!(on_disk[0].org_id, s.org_id);
    assert_eq!(
        on_disk[0].admin_device_key, in_memory[0].admin_device_key,
        "the imported admin device key must be persisted"
    );

    // Re-importing an invite for the same Organisation replaces the stored one
    // rather than appending a second, so a first admission has one answer.
    let again = s.svc_a.export_invite(s.org_id).unwrap();
    s.svc_b.import_invite(&mut OsRng, &again).unwrap();
    assert_eq!(s.svc_b.list_pending_invites().len(), 1, "one invite per organisation");
    let reloaded = reopen_store("invite-persisted", "b", "pw_b");
    assert_eq!(
        reloaded.data().pending_invites.len(),
        1,
        "the replacement must reach the disk as a replacement, not an addition"
    );
}

// LLR-ghja3x's second clause. Review round 2 found that changing
// `let parent_seq = last_seq + 1` to `+ 2` in `admit_member` left the whole
// gate green: both sides of the exchange take the mark from the same envelope,
// and `SeqGuard` only requires strict increase, so nothing compared the
// envelope's number against the administrator's record. This captures the
// envelope A actually sends and compares it against the mark A held before.
//
// *Amended 2026-10-05 (review round 1, finding-1):* the Sequence number is
// the epoch the admission's chain update produced (LLR-ghja3x, REQ-txvtm9),
// no longer one past the record's last. The genesis record is at epoch 1
// with mark 0, so the two rules give different numbers here (2 against 1).
// verifies: LLR-ghja3x, REQ-txvtm9
#[tokio::test(flavor = "multi_thread")]
async fn the_admission_envelope_carries_the_epoch_its_update_produced() {
    let mut s = setup("envelope-mark").await;
    let before = s.svc_a.list_orgs()[0].clone();

    // A "admits B" into a sink, which hands back the message A transmitted.
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, sink_addr, org_secret()),
    )
    .await
    .expect("admit_member timed out")
    .expect("admit_member failed");
    let (_sink_ep, _sender, msg) = sink.await.unwrap();

    assert_eq!(
        msg.envelope.parent_seq,
        SequenceNumber::new(s.chain.get(&s.org_id).unwrap().epoch.get()),
        "the envelope must carry the epoch its chain update produced"
    );
    assert_eq!(msg.envelope.parent_seq, SequenceNumber::new(before.epoch.get() + 1));
    assert!(msg.envelope.parent_seq > before.last_seq, "past the record's last mark");
    assert_eq!(msg.envelope.org_id, s.org_id);

    // Master asserted here that the envelope is signed by the administrator's
    // MEMBER key and not its DevicePublicKey (LLR-ghja3x's first clause). The
    // Envelope carries no signature on this branch (REQ-ag6kqm), so those
    // assertions are not carried; the record still names the administrator's
    // X25519 member key, rebuilt from the persona's persisted seed.
    let (member_kp, _admin_device_kp) = admin_keys(&s.svc_a);
    assert_eq!(
        &member_kp.public_bytes(),
        before.admin_member_key.as_bytes(),
        "the fixture's member key must be the one the record names"
    );

    // A's record then carries that same mark.
    assert_eq!(s.svc_a.list_orgs()[0].last_seq, msg.envelope.parent_seq);
}

// LLR-jn5jeh. Review round 2 found this requirement credited to
// `delivers_and_verifies_admit_over_iroh` and
// `delivers_and_verifies_admit_over_relay_by_id`, neither of which calls
// `admit_member` at all — they build an envelope by hand and send it over a
// bare endpoint — and the second of which is the by-id case the Loopback
// clause excludes. This test calls `admit_member` in Loopback mode and
// observes that the admission reaches the endpoint whose full `EndpointAddr`
// the call carried. The clause's falsification is the mutation
// `r2-jn5jeh-loopback-by-id` in the verification record's appendix: replacing
// the Loopback arm's `ep.send(peer_addr, …)` with a dial by `EndpointId`
// reddens this test, because in Loopback mode nothing resolves an identity to
// an address.
//
// The obvious second half — dial the SAME identity with its transports
// stripped and watch it fail — was written, measured and removed. It does not
// fail: with the listener up, iroh keeps retrying an `EndpointAddr` that
// carries an identity and no transports, past any timeout a gated test can
// afford (measured at >30 s, the suite's whole network budget). That is the
// same property PR-d4nye8's record names on the other side — a dead loopback
// port does not refuse, it retries — and it is why the negative is carried by
// a mutation rather than by an assertion here.
// verifies: LLR-jn5jeh
#[tokio::test(flavor = "multi_thread")]
async fn in_loopback_mode_the_joiner_is_dialled_at_the_full_address() {
    let mut s = setup("loopback-dial").await;

    // A peer whose endpoint is listening, named by the full address it reports.
    let (full_addr, sink) = spawn_recv_one(ROGUE_SEED).await;
    let dialled_id = full_addr.id;

    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, full_addr, org_secret()),
    )
    .await
    .expect("admit_member to the full address timed out")
    .expect("admit_member to the full address failed");

    // It arrived at the endpoint that address named, and it is the admission.
    let (ep, _sender, msg) = sink.await.unwrap();
    assert_eq!(
        ep.inner().id(),
        dialled_id,
        "the admission reached an endpoint other than the one the address named"
    );
    assert!(
        msg.org_secret.is_some() && msg.genesis_snapshot.is_some(),
        "a full admission message, not a bare update"
    );
    assert_eq!(msg.envelope.org_id, s.org_id);
}

// The revocation twin of `a_failed_push_leaves_the_administrators_record_where_it_was`.
// Round 1 found that clause unevidenced on the admission path and closed it
// there; round 2 found that `revoke_member` — the same shape, the same
// ordering, the same ~120 lines — was refined by no low-level requirement at
// all. Writing the twin is the point: a lesson learned on one side of a seam
// is not evidence about the other.
// verifies: LLR-6dc598, LLR-qg9utu
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_revocation_push_leaves_the_administrators_record_where_it_was() {
    let mut s = admit_b_directly(setup("revoke-send-fails").await).await;
    let before = s.svc_a.list_orgs()[0].clone();
    let epoch_before = s.chain.get(&s.org_id).unwrap().epoch;
    let b_id = id_by_handle(&before, "bob");

    // A well-formed identity with no transports: the dial fails at once rather
    // than waiting on the network, which a loopback port with no listener does
    // not do (QUIC retries it for thirty seconds).
    let result = tokio::time::timeout(
        NET,
        s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id, Some(dead_addr([0x7fu8; 32]))),
    )
    .await
    .expect("revoke_member to a dead peer timed out");
    assert!(result.is_err(), "a revocation push to a peer that is not listening must fail");

    // The chain moved — the submission precedes the send, and that is the
    // documented order (LLR-6dc598). What must NOT have moved is A's record.
    assert_eq!(
        s.chain.get(&s.org_id).unwrap().epoch,
        Epoch::new(epoch_before.get() + 1),
        "the chain submission precedes the send and is expected to have landed"
    );
    let after = s.svc_a.list_orgs()[0].clone();
    assert_eq!(after.epoch, before.epoch, "A's recorded epoch must not advance on a failed push");
    assert_eq!(
        after.last_seq, before.last_seq,
        "A's sequence mark must not advance on a failed push"
    );
    assert_eq!(
        after.trie_members.len(),
        before.trie_members.len(),
        "A must not record a removal it failed to deliver"
    );

    let reloaded = reopen_store("revoke-send-fails", "a", "pw_a");
    assert_eq!(
        reloaded.data().orgs[0].trie_members.len(),
        before.trie_members.len(),
        "nor may the undelivered removal reach the disk"
    );
}

// The three out-of-band functions whose behaviour review round 2 found refined
// by nothing. `import_invite` has its own test above (it is the one with a
// persistence site); these are the other three.
// verifies: LLR-zj88e6, LLR-437fvx, LLR-836z24, LLR-qezw3n
#[tokio::test(flavor = "multi_thread")]
async fn the_out_of_band_blobs_carry_the_keys_their_holders_are_pinned_by() {
    let s = setup("blob-contents").await;

    // LLR-zj88e6: the Invite names the DEVICE key of the administrator persona
    // of the Organisation it names. Since 2026-10-05 nothing compares it with
    // the first admission's sender (LLR-j83kc8 as amended).
    let (admin_member, admin_device) = admin_keys(&s.svc_a);
    let invite: org_node::blobs::Invite =
        org_node::blobs::decode(&s.svc_a.export_invite(s.org_id).unwrap()).unwrap();
    assert_eq!(invite.org_id, s.org_id);
    assert_eq!(
        invite.admin_device_key.as_bytes(),
        admin_device.verifying_key().as_bytes(),
        "the invite must name the administrator's device key"
    );
    assert_ne!(
        invite.admin_device_key.as_bytes(), invite.admin_member_key.as_bytes(),
        "the device key and the member key are two different keys"
    );
    assert_eq!(invite.admin_member_key.as_bytes(), &admin_member.public_bytes());

    // LLR-qezw3n: the Invite's dialling address is the bound endpoint's, and
    // it is empty on a service that has bound none. Added 2026-10-05 by
    // review round 8: emptying it was green.
    let bound = s.svc_a.endpoint().expect("setup binds A's endpoint").node_addr_for_dial();
    assert_eq!(
        invite.admin_node_addr,
        postcard::to_allocvec(&bound).unwrap(),
        "the invite must carry the bound endpoint's address"
    );
    let mut unbound = OrgService::new(open_store("blob-contents", "u", "pw_u"), Box::new(MockChainOps::new()));
    let pid_u = unbound.create_persona(&mut OsRng, h("unbound"), nm("Un"), sn("Bound")).unwrap();
    let org_u = unbound.create_organisation(&mut OsRng, &pid_u).await.unwrap();
    assert!(unbound.endpoint().is_none());
    let invite_u: org_node::blobs::Invite =
        org_node::blobs::decode(&unbound.export_invite(org_u).unwrap()).unwrap();
    assert!(invite_u.admin_node_addr.is_empty(), "no endpoint bound, no address");

    // LLR-437fvx: the JoinRequest carries the persona's own member and device
    // keys, as two distinct keys, each from that persona's own seed.
    let persona_b = persona_of(&s.svc_b, &s.pid_b);
    let jr: org_node::blobs::JoinRequest =
        org_node::blobs::decode(&s.svc_b.export_join_request(&s.pid_b).unwrap()).unwrap();
    assert_eq!(
        jr.member_key.as_bytes(),
        &persona_b.member_seed.x25519_keypair().public_bytes(),
        "the join request must carry the persona's member key"
    );
    assert_eq!(
        jr.device_key.as_bytes(),
        persona_b.device_seed.signing_keypair().verifying_key().as_bytes(),
        "the join request must carry the persona's device key"
    );
    assert_ne!(jr.member_key.as_bytes(), jr.device_key.as_bytes(), "the two keys must be distinct");
    assert_eq!(jr.handle, persona_b.handle);

    // LLR-836z24: importing a join request stores nothing. A's record is
    // identical on both sides of the call, on disk as well as in memory.
    let orgs_before = s.svc_a.list_orgs().to_vec();
    let personas_before = s.svc_a.list_personas().len();
    let invites_before = s.svc_a.list_pending_invites().len();
    let decoded = join_request_of(&s.svc_b, &s.pid_b);
    assert_eq!(decoded.handle, jr.handle, "the import must return what was exported");
    assert_eq!(s.svc_a.list_orgs().len(), orgs_before.len());
    assert_eq!(s.svc_a.list_orgs()[0].trie_members.len(), orgs_before[0].trie_members.len());
    assert_eq!(s.svc_a.list_personas().len(), personas_before);
    assert_eq!(s.svc_a.list_pending_invites().len(), invites_before);
}

// ---------------------------------------------------------------------------
// Added 2026-10-04 after review round 3.
//
// Round 2's finding-3 convicted `ensure_endpoint` for resting on "every gated
// test gives its service exactly one persona", and the fix wrote a two-persona
// test for that one call site. Round 3 found the same assumption — one
// Organisation and one Persona per store — holding up four more lookups:
// `find_org`, `find_org_mut`, `admin_persona_for_org` and
// `update_persona_status` could each ignore the identifier they were given and
// return whatever record came first, with the WHOLE gate green.
//
// The failure that makes it matter: an administrator holding two Organisations
// calls `admit_member(org_2, …)`, and the joiner is minted into org_1's
// record, org_1's root is submitted on chain, and the envelope is signed by
// org_1's administrator persona. Nothing in this unit said otherwise.
//
// This test is the whole class in one fixture: two personas, two
// Organisations, one admission into the second. Each of the four lookups is
// reddened by it.
// ---------------------------------------------------------------------------

// verifies: LLR-vdyu65, LLR-w3fhhg
#[tokio::test(flavor = "multi_thread")]
async fn a_second_organisation_is_admitted_into_without_touching_the_first() {
    let chain = MockChainOps::new();
    let mut svc_a = OrgService::new(open_store("two-orgs-admit", "a", "pw_a"), Box::new(chain.clone()));
    let mut svc_b = OrgService::new(open_store("two-orgs-admit", "b", "pw_b"), Box::new(chain.clone()));

    // TWO personas, so the administrator of org_2 is not the first persona in
    // the store. `admin_persona_for_org` must pick the second.
    let pid_1 = svc_a.create_persona(&mut OsRng, h("first"), nm("First"), sn("Admin")).unwrap();
    let pid_2 = svc_a.create_persona(&mut OsRng, h("second"), nm("Second"), sn("Admin")).unwrap();
    let org_1 = svc_a.create_organisation(&mut OsRng, &pid_1).await.unwrap();
    let org_2 = svc_a.create_organisation(&mut OsRng, &pid_2).await.unwrap();
    assert_ne!(org_1, org_2);

    // `create_organisation` binds each persona to the Organisation it founded,
    // which is `update_persona_status`'s job.
    assert_eq!(persona_of(&svc_a, &pid_1).org_id, Some(org_1), "persona 1 belongs to org 1");
    assert_eq!(persona_of(&svc_a, &pid_2).org_id, Some(org_2), "persona 2 belongs to org 2");

    let one_before = rec_of(&svc_a, org_1);
    let two_before = rec_of(&svc_a, org_2);
    assert_ne!(
        one_before.admin_member_key, two_before.admin_member_key,
        "two personas must give the two Organisations different administrators"
    );

    // A's endpoint, bound from persona 2's device seed — the administrator of
    // the Organisation being admitted into.
    let ep_a = OrgEndpoint::bind(&device_kp(&svc_a, &pid_2)).await.unwrap();
    let mut svc_a = svc_a.with_endpoint(ep_a);

    // B imports org_2's invite and offers a join request.
    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    let invite_blob = svc_a.export_invite(org_2).unwrap();
    let invite = svc_b.import_invite(&mut OsRng, &invite_blob).unwrap();
    assert_eq!(invite.org_id, org_2, "the invite must name the Organisation it was exported for");
    assert_eq!(
        invite.admin_member_key, two_before.admin_member_key,
        "the invite must carry org 2's administrator, not org 1's"
    );
    let jr_b = join_request_of(&svc_b, &pid_b);
    let b_device_kp = device_kp(&svc_b, &pid_b);

    // Admit B into org_2. B verifies the envelope against org_2's on-chain
    // state, so an envelope signed by org_1's administrator is refused.
    let (b_addr, b_task) = spawn_b_receive(svc_b, &b_device_kp).await;
    let new_member = tokio::time::timeout(
        NET,
        svc_a.admit_member(&mut OsRng, org_2, &jr_b, b_addr, org_secret()),
    )
    .await
    .expect("admit_member(org 2) timed out")
    .expect("admit_member(org 2) failed");
    let (svc_b, outcome) = b_task.await.unwrap();
    let outcome = outcome.expect("B must verify an admission signed by org 2's administrator");
    assert_eq!(svc_b.list_orgs()[0].org_id, org_2, "B committed the wrong Organisation");

    // org_2 moved: on chain, in A's record, and in B's.
    let two_after = rec_of(&svc_a, org_2);
    assert_eq!(two_after.epoch, Epoch::new(two_before.epoch.get() + 1), "org 2's epoch must advance");
    assert_eq!(two_after.trie_members.len(), 2, "org 2 holds its admin and B");
    assert!(two_after.trie_members.iter().any(|m| m.id == new_member));
    assert_eq!(outcome.epoch, two_after.epoch);
    assert_eq!(chain.get(&org_2).unwrap().epoch, two_after.epoch);

    // org_1 did NOT move — not in A's record, not on chain. This is the
    // assertion the four lookups rest on.
    let one_after = rec_of(&svc_a, org_1);
    assert_eq!(one_after.epoch, one_before.epoch, "org 1's epoch must not advance");
    assert_eq!(
        one_after.trie_members.len(),
        1,
        "org 1 must still hold its administrator alone"
    );
    assert!(
        !one_after.trie_members.iter().any(|m| m.id == new_member),
        "the joiner was minted into the Organisation the caller did not name"
    );
    assert_eq!(
        chain.get(&org_1).unwrap().epoch,
        one_before.epoch,
        "org 1's root must not have been submitted on chain"
    );
    assert_eq!(one_after.last_seq, one_before.last_seq, "org 1's mark must not move");

    // ...and on disk, which is where a lookup that ignored its argument would
    // leave the damage.
    let reloaded = reopen_store("two-orgs-admit", "a", "pw_a");
    let on_disk = |id: OrgId| disk_rec_of(&reloaded, id);
    assert_eq!(on_disk(org_2).trie_members.len(), 2, "org 2's admission must reach the disk");
    assert_eq!(on_disk(org_1).trie_members.len(), 1, "org 1 must be untouched on disk");
    assert_eq!(on_disk(org_1).epoch, one_before.epoch);
}

// LLR-pw369n's refusal half. `revoke_member` carries a rejection
// `admit_member` has no counterpart for: in Loopback mode there is no
// discovery to fall back on, so a revocation offered no peer address is
// refused with a typed error rather than attempted and lost — but only AFTER
// the new root has been submitted on chain (PR-b9wab3), which the epoch
// assertion below pins. Added 2026-10-04 after review round 3 found this
// clause — the fourth of SDD-72ddm6's interface — stated in no low-level
// requirement, although round 2 had just written the other three.
//
// *Corrected 2026-10-04 by review round 5.* This said "refused before anything
// is signed or submitted", the claim round 4 corrected in the decomposition and
// the risk file and left standing here, above an assertion that the chain
// epoch advanced.
//
// The dial half of the same requirement is carried by
// `a_revocation_reaches_the_administrators_disk`, which the mutation
// `r3-revoke-loopback-by-id` reddens.
// verifies: LLR-pw369n
#[tokio::test(flavor = "multi_thread")]
async fn a_loopback_revocation_with_no_peer_address_is_refused_and_records_nothing() {
    let mut s = admit_b_directly(setup("revoke-no-addr").await).await;
    let before = s.svc_a.list_orgs()[0].clone();
    let epoch_before = s.chain.get(&s.org_id).unwrap().epoch;
    let b_id = id_by_handle(&before, "bob");

    let result =
        tokio::time::timeout(NET, s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id, None))
            .await
            .expect("revoke_member(None) timed out");

    match &result {
        Err(OrgNodeError::Chain(m)) if m.contains("Loopback revoke requires") => {}
        other => panic!("expected a refusal naming the missing address, got {other:?}"),
    }

    // The chain HAS advanced, and that is PR-b9wab3 rather than a property
    // worth asserting as correct. `peer_addr == None` is knowable at the
    // function's first line, but the check sits after `submit_update`, so a
    // caller mistake costs an on-chain epoch and leaves the published root
    // without the member while A's own record still has them. This assertion
    // is written the way it is — pinning the defect — so that moving the check
    // earlier reddens it and the problem report gets closed deliberately
    // rather than drifting.
    assert_eq!(
        s.chain.get(&s.org_id).unwrap().epoch,
        Epoch::new(epoch_before.get() + 1),
        "PR-b9wab3: the submission still precedes this refusal"
    );
    let after = s.svc_a.list_orgs()[0].clone();
    assert_eq!(after.epoch, before.epoch, "A's recorded epoch must not move");
    assert_eq!(after.last_seq, before.last_seq, "A's sequence mark must not move");
    assert_eq!(
        after.trie_members.len(),
        before.trie_members.len(),
        "A must not record a removal it never sent"
    );
    assert!(
        after.trie_members.iter().any(|m| m.id == b_id),
        "B must still be in A's record"
    );
}

// ---------------------------------------------------------------------------
// Added 2026-10-04 after review round 4.
//
// Round 3 found four record lookups able to ignore the identifier they were
// given, and this file's `a_second_organisation_is_admitted_into_without_touching_the_first`
// closed them — on the ADMINISTRATOR's side. Round 4 found thirteen more on
// the two receive paths and in `import_invite`, all green, because that test
// gives the receiving service one Persona and one Organisation. It was a
// two-Organisation test of the sender and a one-Organisation test of everyone
// else; the class is "a lookup resting on one Organisation or one Persona per
// store", and the sweep covered `admit_member`'s call graph and stopped there.
//
// What the gate reported green includes: an update for org 2 writing org 2's
// root, epoch, sequence mark, member snapshots and ORGANISATION SECRET into
// org 1's record; a revocation from org 2 deleting EVERY Organisation record
// the node holds; and the first-admission lookup of the pending Invite (then
// a sender cross-check, retired 2026-10-05; now the Invite a first admission
// consumes and takes its administrator key from) reading whichever pending
// invite happens to come first.
//
// This fixture is the receiving half of the class: a node that holds two
// Organisations through two Personas, with both invites imported before
// either admission so the Invite lookup has something to get wrong.
// ---------------------------------------------------------------------------

/// A receiver holding two Organisations. Two independent administrators share
/// one chain; B joins both. Returns everything the tests below need.
struct TwoOrgReceiver {
    chain: MockChainOps,
    svc_a1: OrgService,
    svc_a2: OrgService,
    svc_b: OrgService,
    org_1: OrgId,
    org_2: OrgId,
    pid_b1: PersonaId,
    pid_b2: PersonaId,
    b1_device_kp: SigningKeypair,
    b2_device_kp: SigningKeypair,
}

async fn two_org_receiver(tag: &str) -> TwoOrgReceiver {
    let chain = MockChainOps::new();

    // Two administrators, each with its own store and its own Organisation.
    // They are separate services so that each can hold its own endpoint; what
    // is under test here is the RECEIVER's lookups, not the sender's.
    let mut svc_a1 = OrgService::new(open_store(tag, "a1", "pw_a1"), Box::new(chain.clone()));
    let mut svc_a2 = OrgService::new(open_store(tag, "a2", "pw_a2"), Box::new(chain.clone()));
    let pid_a1 = svc_a1.create_persona(&mut OsRng, h("admin1"), nm("Admin"), sn("One")).unwrap();
    let pid_a2 = svc_a2.create_persona(&mut OsRng, h("admin2"), nm("Admin"), sn("Two")).unwrap();
    let org_1 = svc_a1.create_organisation(&mut OsRng, &pid_a1).await.unwrap();
    let org_2 = svc_a2.create_organisation(&mut OsRng, &pid_a2).await.unwrap();
    assert_ne!(org_1, org_2);
    let a1_dev = device_kp(&svc_a1, &pid_a1);
    let a2_dev = device_kp(&svc_a2, &pid_a2);
    let mut svc_a1 = svc_a1.with_endpoint(OrgEndpoint::bind(&a1_dev).await.unwrap());
    let mut svc_a2 = svc_a2.with_endpoint(OrgEndpoint::bind(&a2_dev).await.unwrap());

    // B: two Personas, one per Organisation.
    let mut svc_b = OrgService::new(open_store(tag, "b", "pw_b"), Box::new(chain.clone()));
    let pid_b1 = svc_b.create_persona(&mut OsRng, h("bob-one"), nm("Bob"), sn("One")).unwrap();
    let pid_b2 = svc_b.create_persona(&mut OsRng, h("bob-two"), nm("Bob"), sn("Two")).unwrap();

    // **Order matters.** org 2's invite is imported FIRST, so a lookup that
    // takes "whichever invite comes first" rather than the one for this
    // Organisation picks the wrong administrator's key on the admission that
    // follows.
    let inv_2 = svc_a2.export_invite(org_2).unwrap();
    svc_b.import_invite(&mut OsRng, &inv_2).unwrap();
    let inv_1 = svc_a1.export_invite(org_1).unwrap();
    svc_b.import_invite(&mut OsRng, &inv_1).unwrap();
    assert_eq!(
        svc_b.list_pending_invites().len(),
        2,
        "two Organisations must leave two pending invites — one each"
    );
    assert_eq!(
        svc_b.list_pending_invites()[0].org_id, org_2,
        "org 2's invite must be the one a 'first invite' lookup would find"
    );

    let b1_device_kp = device_kp(&svc_b, &pid_b1);
    let b2_device_kp = device_kp(&svc_b, &pid_b2);

    // First admission into org 1, with org 2's invite sitting ahead of it.
    let jr_b1 = join_request_of(&svc_b, &pid_b1);
    let (addr, task) = spawn_b_receive(svc_b, &b1_device_kp).await;
    tokio::time::timeout(
        NET,
        svc_a1.admit_member(&mut OsRng, org_1, &jr_b1, addr, Some(OrgSecret::from([0x11u8; 32]))),
    )
    .await
    .expect("admit b1 timed out")
    .expect("admit b1 failed");
    let (svc_b, outcome) = task.await.unwrap();
    outcome.expect("B must accept its first admission to org 1");
    assert_eq!(
        svc_b.list_pending_invites().len(),
        1,
        "committing org 1 must consume org 1's invite and only that one"
    );
    assert_eq!(
        svc_b.list_pending_invites()[0].org_id, org_2,
        "org 2's invite must survive org 1's admission"
    );

    // First admission into org 2.
    let jr_b2 = join_request_of(&svc_b, &pid_b2);
    let (addr, task) = spawn_b_receive(svc_b, &b2_device_kp).await;
    tokio::time::timeout(
        NET,
        svc_a2.admit_member(&mut OsRng, org_2, &jr_b2, addr, Some(OrgSecret::from([0x22u8; 32]))),
    )
    .await
    .expect("admit b2 timed out")
    .expect("admit b2 failed");
    let (svc_b, outcome) = task.await.unwrap();
    outcome.expect("B must accept its first admission to org 2");
    assert_eq!(svc_b.list_orgs().len(), 2, "B now holds two Organisations");
    assert!(svc_b.list_pending_invites().is_empty(), "both invites are consumed");

    TwoOrgReceiver {
        chain,
        svc_a1,
        svc_a2,
        svc_b,
        org_1,
        org_2,
        pid_b1,
        pid_b2,
        b1_device_kp,
        b2_device_kp,
    }
}

// It also carries LLR-9zfnmb's "per Organisation" clause (the fixture asserts
// two invites leave two pending, one each) and LLR-e5c9ud's binding of the
// admitted Persona to the change's Organisation (b1 to org 1, b2 to org 2):
// the author's trace check after review round 7 found both falsified only by
// this test, which did not name them.
// verifies: LLR-cja9zv, LLR-q8emds, LLR-y2v8v2, LLR-9zfnmb, LLR-e5c9ud
#[tokio::test(flavor = "multi_thread")]
async fn a_receiver_holding_two_organisations_commits_into_the_one_the_change_names() {
    let mut s = two_org_receiver("two-org-recv").await;
    let one_before = rec_of(&s.svc_b, s.org_1);
    let two_before = rec_of(&s.svc_b, s.org_2);
    assert_ne!(one_before.org_secret, two_before.org_secret, "distinct secrets");

    // A2 admits C into org 2, and B receives it.
    let pid_c = s.svc_a2.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let jr_c = join_request_of(&s.svc_a2, &pid_c);
    let (addr, task) = spawn_b_receive(s.svc_b, &s.b2_device_kp).await;
    let c_id = tokio::time::timeout(
        NET,
        s.svc_a2.admit_member(&mut OsRng, s.org_2, &jr_c, addr, Some(OrgSecret::from([0x22u8; 32]))),
    )
    .await
    .expect("admit C timed out")
    .expect("admit C failed");
    let (svc_b, outcome) = task.await.unwrap();
    let outcome = outcome.expect("B must commit an update for an Organisation it holds");
    assert_eq!(outcome.org_id, s.org_2, "the outcome must name org 2");

    // org 2 moved.
    let two_after = rec_of(&svc_b, s.org_2);
    assert!(two_after.epoch > two_before.epoch, "org 2's epoch must advance");
    assert!(
        two_after.trie_members.iter().any(|m| m.id == c_id),
        "org 2's record must hold C"
    );

    // **org 1 did not.** This is the assertion the thirteen lookups rest on:
    // every field an update writes, compared against what org 1 held before.
    let one_after = rec_of(&svc_b, s.org_1);
    assert_eq!(one_after.epoch, one_before.epoch, "org 1's epoch must not move");
    assert_eq!(one_after.last_seq, one_before.last_seq, "org 1's mark must not move");
    assert_eq!(one_after.root_hash, one_before.root_hash, "org 1's root must not move");
    assert_eq!(
        one_after.org_secret, one_before.org_secret,
        "org 1's Organisation secret must not be overwritten by another Organisation's"
    );
    assert_eq!(
        one_after.trie_members.len(),
        one_before.trie_members.len(),
        "org 1's membership must not change"
    );
    assert!(
        !one_after.trie_members.iter().any(|m| m.id == c_id),
        "a member admitted to org 2 appeared in org 1's record"
    );

    // Personas: b2 is bound to org 2 and b1 is still bound to org 1.
    assert_eq!(persona_of(&svc_b, &s.pid_b1).org_id, Some(s.org_1));
    assert_eq!(persona_of(&svc_b, &s.pid_b2).org_id, Some(s.org_2));

    // ...and all of it on disk.
    let reloaded = reopen_store("two-org-recv", "b", "pw_b");
    let on_disk = |id: OrgId| disk_rec_of(&reloaded, id);
    assert_eq!(on_disk(s.org_2).epoch, two_after.epoch, "org 2's update must reach the disk");
    assert_eq!(on_disk(s.org_1).epoch, one_before.epoch, "org 1 must be untouched on disk");
    assert_eq!(on_disk(s.org_1).org_secret, one_before.org_secret);

    // org 2's record takes the root on chain. Added 2026-10-04 by review round
    // 6: leaving `existing.root_hash` unwritten on this branch was green.
    let org2_root = s.chain.get(&s.org_2).unwrap().root_hash;
    assert_eq!(two_after.root_hash, org2_root, "org 2's record must take the published root");
    assert_eq!(on_disk(s.org_2).root_hash, org2_root);
}

// The self-delete path's half of the same class. Review round 4 measured
// `self.store.data_mut().orgs.retain(|_o| false)` — delete EVERY Organisation
// record rather than the one being revoked from — as green, along with four
// other lookups on this path.
// verifies: REQ-uxv2x2, LLR-6p4pj2, LLR-jwhzh3
#[tokio::test(flavor = "multi_thread")]
async fn a_self_delete_removes_only_the_organisation_the_revocation_came_from() {
    let mut s = two_org_receiver("two-org-selfdel").await;
    let one_before = rec_of(&s.svc_b, s.org_1);
    let b2_member_id = persona_of(&s.svc_b, &s.pid_b2).member_id.expect("b2 has a member id in org 2");

    let (addr, task) = spawn_b_self_delete(s.svc_b, &s.b2_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a2.revoke_member(&mut OsRng, s.org_2, b2_member_id, Some(addr)),
    )
    .await
    .expect("revoke b2 timed out")
    .expect("revoke b2 failed");

    let (svc_b, outcome) = task.await.unwrap();
    assert!(
        matches!(
            outcome.expect("B must act on its own revocation from org 2"),
            SelfDeleteOutcome::SelfDeleted { org_id } if org_id == s.org_2
        ),
        "B must self-delete from org 2 and say so"
    );

    // org 2 is gone; **org 1 is not**.
    assert_eq!(svc_b.list_orgs().len(), 1, "exactly one Organisation must remain");
    let one_after = rec_of(&svc_b, s.org_1);
    assert_eq!(one_after.epoch, one_before.epoch, "org 1's record must be untouched");
    assert_eq!(one_after.org_secret, one_before.org_secret, "org 1's secret must survive");
    assert_eq!(one_after.trie_members.len(), one_before.trie_members.len());

    // The Persona bound to org 2 is revoked; the one bound to org 1 is not.
    assert_eq!(persona_of(&svc_b, &s.pid_b2).status, PersonaStatus::Revoked, "b2 is revoked");
    assert_eq!(
        persona_of(&svc_b, &s.pid_b1).status,
        PersonaStatus::Active,
        "b1 is in another Organisation and must not be revoked by org 2's removal"
    );

    let reloaded = reopen_store("two-org-selfdel", "b", "pw_b");
    assert_eq!(reloaded.data().orgs.len(), 1, "only org 1 may remain on disk");
    assert_eq!(reloaded.data().orgs[0].org_id, s.org_1);
    assert_eq!(reloaded.data().orgs[0].org_secret, one_before.org_secret);
}

// The two lookups on the self-delete path that the test above leaves green,
// found by probing the other nine of review round 4's thirteen rather than the
// four the round showed: the record the "still a member" branch commits into,
// and which Personas decide whether the node is still a member at all.
//
// Both need org 2's record to hold a device key belonging to B's **org 1**
// Persona. So org 2's administrator enrols b1's join request as well — B
// receives that change through the self-delete path, which is the first
// half — and then revokes b2. A "still a member" test that consults every
// Persona rather than org 2's finds b1's device in org 2's trie and keeps B in
// an Organisation it was just removed from: an administrator could pin any
// member in place by enrolling a key the member uses elsewhere.
// verifies: LLR-jsx922, LLR-6p4pj2, LLR-jwhzh3
#[tokio::test(flavor = "multi_thread")]
async fn membership_of_one_organisation_is_judged_by_that_organisations_personas_alone() {
    let mut s = two_org_receiver("two-org-mine").await;
    let one_before = rec_of(&s.svc_b, s.org_1);
    let two_before = rec_of(&s.svc_b, s.org_2);

    // Half 1 — an ordinary update arriving on the self-delete path. org 2's
    // administrator admits b1's device key into org 2.
    let jr_b1 = join_request_of(&s.svc_b, &s.pid_b1);
    let (addr, task) = spawn_b_self_delete(s.svc_b, &s.b2_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a2.admit_member(&mut OsRng, s.org_2, &jr_b1, addr, Some(OrgSecret::from([0x22u8; 32]))),
    )
    .await
    .expect("admit b1 into org 2 timed out")
    .expect("admit b1 into org 2 failed");
    let (svc_b, outcome) = task.await.unwrap();
    assert!(
        matches!(
            outcome.expect("B must commit an update it is still a member after"),
            SelfDeleteOutcome::UpdatedNotRevoked { org_id } if org_id == s.org_2
        ),
        "B is still in org 2 and must say so, naming org 2"
    );
    let b1_device = s.b1_device_kp.device_key().unwrap();
    let two_mid = rec_of(&svc_b, s.org_2);
    assert!(two_mid.epoch > two_before.epoch, "org 2's record must take the update");
    assert!(
        two_mid.trie_members.iter().any(|m| m.device_keys.contains(&b1_device)),
        "org 2's record must now hold b1's device key"
    );
    let one_mid = rec_of(&svc_b, s.org_1);
    assert_eq!(one_mid.epoch, one_before.epoch, "org 1's epoch must not move");
    assert_eq!(one_mid.last_seq, one_before.last_seq, "org 1's mark must not move");
    assert_eq!(one_mid.root_hash, one_before.root_hash, "org 1's root must not move");
    assert_eq!(
        one_mid.trie_members.len(),
        one_before.trie_members.len(),
        "org 2's update must not be written into org 1's record"
    );

    // Half 2 — org 2 revokes b2. b1's device is still in org 2's trie, but b1
    // is org 1's Persona; B's membership of org 2 was b2, and b2 is gone.
    let b2_member_id = persona_of(&svc_b, &s.pid_b2).member_id.expect("b2 has a member id in org 2");
    let (addr, task) = spawn_b_self_delete(svc_b, &s.b2_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a2.revoke_member(&mut OsRng, s.org_2, b2_member_id, Some(addr)),
    )
    .await
    .expect("revoke b2 timed out")
    .expect("revoke b2 failed");
    let (svc_b, outcome) = task.await.unwrap();
    assert!(
        matches!(
            outcome.expect("B must act on its own revocation from org 2"),
            SelfDeleteOutcome::SelfDeleted { org_id } if org_id == s.org_2
        ),
        "b2 was B's membership of org 2; another Organisation's Persona must not keep B in it"
    );
    assert_eq!(svc_b.list_orgs().len(), 1, "only org 1 may remain");
    assert_eq!(svc_b.list_orgs()[0].org_id, s.org_1);
}

// ---------------------------------------------------------------------------
// Review round 5.
// ---------------------------------------------------------------------------

// LLR-u6rq4s as amended 2026-10-05: the delivering device is compared with no
// Membership record. C relays its own removal to B on the ordinary receive
// path; C is in B's record before the change and not after it, and B commits
// the change because it matches the chain.
// verifies: REQ-ztdza4, LLR-u6rq4s
#[tokio::test(flavor = "multi_thread")]
async fn a_removal_relayed_by_the_member_it_removes_is_committed() {
    let mut s = admit_b_directly(setup("relay-by-removed").await).await;

    // C: a persona on A's device whose seed the test holds. The admission is
    // pushed to B, so B's record holds C.
    let pid_c = s.svc_a.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let c_seed = *persona_of(&s.svc_a, &pid_c).device_seed.expose_secret();
    let jr_c = join_request_of(&s.svc_a, &pid_c);
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    let c_id = tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, org_secret()),
    )
    .await
    .expect("admit C timed out")
    .expect("admit C failed");
    let (svc_b, out) = b_task.await.unwrap();
    out.expect("B commits C's admission");
    assert!(svc_b.list_orgs()[0].trie_members.iter().any(|m| m.id == c_id));
    let before = svc_b.list_orgs()[0].clone();

    // A revokes C and pushes the change to C's own endpoint.
    let (c_addr, c_task) = spawn_recv_one(c_seed).await;
    tokio::time::timeout(NET, s.svc_a.revoke_member(&mut OsRng, s.org_id, c_id, Some(c_addr)))
        .await
        .expect("revoke C timed out")
        .expect("revoke C failed");
    let (c_ep, _sender, msg) = c_task.await.unwrap();

    // C relays its own removal to B, on B's ordinary receive path.
    let (b_addr, b_task) = spawn_b_receive(svc_b, &s.b_device_kp).await;
    c_ep.send(b_addr, &msg).await.expect("relay to B");
    let (svc_b, result) = b_task.await.unwrap();
    let committed = result.expect("a chain-valid removal is committed whoever relays it");
    assert!(committed.epoch > before.epoch);
    let after = svc_b.list_orgs()[0].clone();
    assert!(!after.trie_members.iter().any(|m| m.id == c_id), "C is gone from B's record");
}

// Three things a first admission writes that no low-level requirement stated
// until review round 5: the administrator key on the member's record, the
// Organisation secret the message carried, and the admitted Persona's member
// id and status. The last two were already evidenced by mutation; the first
// was not — zeroing it left the gate green.
//
// *Amended 2026-10-05 (owner answer Q1 of the switch trim).* When an Invite
// was imported, the record's administrator key is the administrator's
// Member-as-a-group key the Invite named (LLR-rys5nx), not the chain's
// Organisation public key. LLR-xq9nrq states the case with no Invite, which
// `a_first_admission_with_no_imported_invite_records_the_chains_key_as_admin_member_key`
// covers. A member's record holds no Organisation private key (LLR-3fwykc):
// only the creating node's does.
// *Renamed 2026-10-05 by review round 2 (finding-21).* Master's name,
// `a_first_admission_records_the_signing_key_the_secret_and_the_member`,
// named a signing key the record no longer holds; master's ledgers cite it.
// verifies: LLR-ckk5nz, LLR-e5c9ud, LLR-rys5nx, LLR-3fwykc
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_records_the_invites_administrator_the_chains_key_the_secret_and_the_member() {
    let s = admit_b_directly(setup("first-admission-fields").await).await;
    let published = s.chain.get(&s.org_id).unwrap().org_pub_key;
    let admin_member_key = s.svc_a.list_orgs()[0].admin_member_key;
    let rec = s.svc_b.list_orgs()[0].clone();
    assert_eq!(
        rec.admin_member_key, admin_member_key,
        "the administrator key is the one the Invite named"
    );
    assert_ne!(rec.admin_member_key.as_bytes(), published.as_bytes(), "not the chain's Organisation public key");
    assert_eq!(rec.org_pub_key, published);
    assert_eq!(rec.org_secret, org_secret(), "the secret the message carried is stored");
    assert_eq!(rec.org_private_key, None, "a member's record holds no Organisation private key");

    let b_device = s.b_device_kp.device_key().unwrap();
    let b_member = rec
        .trie_members
        .iter()
        .find(|m| m.device_keys.contains(&b_device))
        .expect("B's device key is in the trie");
    let persona = persona_of(&s.svc_b, &s.pid_b);
    assert_eq!(persona.member_id, Some(b_member.id), "the Persona carries its member id");
    assert_eq!(persona.status, PersonaStatus::Active);
    assert_eq!(persona.org_id, Some(s.org_id));

    let reloaded = reopen_store("first-admission-fields", "b", "pw_b");
    assert_eq!(reloaded.data().orgs[0].admin_member_key, admin_member_key);
    assert_eq!(reloaded.data().orgs[0].org_secret, org_secret());
}

// PR-xwek5e — PINS A DEFECT. `revoke_member` sends `org_secret: None`, and
// `receive_and_verify` writes whatever the message carries into an existing
// record, so a member that receives somebody else's revocation loses the
// Organisation secret. This test asserts the behaviour as it is; correcting
// it reddens the last assertion, and that is the signal to rewrite this test
// against the ruled behaviour.
//
// It carries LLR-ckk5nz's second clause, which states this behaviour without
// endorsing it. Review round 6 found the clause traced only to the
// first-admission test, which cannot falsify it.
// It also carries LLR-8hdu9x's receiver clause (review round 9).
// verifies: LLR-ckk5nz, LLR-8hdu9x
#[tokio::test(flavor = "multi_thread")]
async fn pr_xwek5e_another_members_revocation_clears_the_receivers_secret() {
    let mut s = admit_b_directly(setup("pr-xwek5e").await).await;
    assert_eq!(s.svc_b.list_orgs()[0].org_secret, org_secret());

    let jr_c = join_request_for_c(&mut s.svc_a);
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    let c_id = tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, org_secret()),
    )
    .await
    .unwrap()
    .unwrap();
    let (svc_b, out) = b_task.await.unwrap();
    out.unwrap();

    let (b_addr, b_task) = spawn_b_receive(svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, s.svc_a.revoke_member(&mut OsRng, s.org_id, c_id, Some(b_addr)))
        .await
        .unwrap()
        .unwrap();
    let (svc_b, out) = b_task.await.unwrap();
    out.expect("B commits C's removal");
    assert_eq!(
        svc_b.list_orgs()[0].org_secret,
        None,
        "PR-xwek5e: B, still a member, has lost the Organisation secret"
    );
}

// PR-mdv38y — PINS A DEFECT. `receive_and_verify` picks the Persona to mark
// Active from every Persona whose device key is in the verified trie, with no
// regard to which Organisation that Persona is bound to, and then rebinds it.
// Org 2's administrator enrols b1 (B's org 1 Persona) and B receives that on
// the ordinary path: b1 is rebound to org 2. From then on an ordinary org 1
// update makes B delete its org 1 record, and org 2 revoking b2 leaves B in
// org 2 — the pin LLR-jwhzh3 states closed on the self-delete path alone.
// Found by review round 5. Each assertion below is the defect; correcting it
// reddens them.
#[tokio::test(flavor = "multi_thread")]
async fn pr_mdv38y_the_receive_path_rebinds_another_organisations_persona() {
    let mut s = two_org_receiver("pr-mdv38y").await;
    let b1_before = persona_of(&s.svc_b, &s.pid_b1);
    assert_eq!(b1_before.org_id, Some(s.org_1));

    // Org 2 enrols b1's device; B receives it on receive_and_verify.
    let jr_b1 = join_request_of(&s.svc_b, &s.pid_b1);
    let (addr, task) = spawn_b_receive(s.svc_b, &s.b2_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a2.admit_member(&mut OsRng, s.org_2, &jr_b1, addr, Some(OrgSecret::from([0x22u8; 32]))),
    )
    .await
    .unwrap()
    .unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert_eq!(outcome.unwrap().org_id, s.org_2);
    let b1_mid = persona_of(&svc_b, &s.pid_b1);
    assert_eq!(b1_mid.org_id, Some(s.org_2), "PR-mdv38y: org 1's Persona rebound to org 2");
    assert_ne!(b1_mid.member_id, b1_before.member_id, "PR-mdv38y: and its member id replaced");

    // An ordinary org 1 update: B is still in org 1's trie, and deletes org 1.
    let pid_c = s.svc_a1.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let jr_c = join_request_of(&s.svc_a1, &pid_c);
    let (addr, task) = spawn_b_self_delete(svc_b, &s.b1_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a1.admit_member(&mut OsRng, s.org_1, &jr_c, addr, Some(OrgSecret::from([0x11u8; 32]))),
    )
    .await
    .unwrap()
    .unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert!(
        matches!(
            outcome.unwrap(),
            SelfDeleteOutcome::SelfDeleted { org_id } if org_id == s.org_1
        ),
        "PR-mdv38y: B self-deletes from an Organisation it is still a member of"
    );

    // Org 2 revokes b2, B's own membership: B stays in org 2.
    let b2_member_id = persona_of(&svc_b, &s.pid_b2).member_id.unwrap();
    let (addr, task) = spawn_b_self_delete(svc_b, &s.b2_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a2.revoke_member(&mut OsRng, s.org_2, b2_member_id, Some(addr)),
    )
    .await
    .unwrap()
    .unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert!(
        matches!(
            outcome.unwrap(),
            SelfDeleteOutcome::UpdatedNotRevoked { org_id } if org_id == s.org_2
        ),
        "PR-mdv38y: B is pinned in an Organisation it was removed from"
    );
    assert!(svc_b.list_orgs().iter().any(|o| o.org_id == s.org_2));
}

// PR-8qsnhx — PINS A DEFECT. `ensure_endpoint` binds once per service, so a
// device administering two Organisations through two Personas sends the
// second Organisation's admissions under the FIRST Persona's device key. Found
// by review round 5; `a_second_organisation_is_admitted_into_without_touching_the_first`
// hides it by injecting the second Persona's endpoint by hand. No endpoint is
// injected here. Correcting the defect reddens the assertion marked below.
// *Amended 2026-10-05 (switch trim).* The joiner refused that push while its
// receive path checked the sender against the Invite; nothing about the
// sender is checked since the owner's ruling, so the joiner now commits it.
// What stays pinned is the key the push goes out under, captured by a relay.
#[tokio::test(flavor = "multi_thread")]
async fn pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key() {
    let chain = MockChainOps::new();
    let mut svc_a = OrgService::new(open_store("pr-8qsnhx", "a", "pw_a"), Box::new(chain.clone()));
    let pid_1 = svc_a.create_persona(&mut OsRng, h("first"), nm("First"), sn("Admin")).unwrap();
    let pid_2 = svc_a.create_persona(&mut OsRng, h("second"), nm("Second"), sn("Admin")).unwrap();
    let org_1 = svc_a.create_organisation(&mut OsRng, &pid_1).await.unwrap();
    let org_2 = svc_a.create_organisation(&mut OsRng, &pid_2).await.unwrap();

    let mut svc_b1 = OrgService::new(open_store("pr-8qsnhx", "b1", "pw_b1"), Box::new(chain.clone()));
    let mut svc_b2 = OrgService::new(open_store("pr-8qsnhx", "b2", "pw_b2"), Box::new(chain.clone()));
    let pid_b1 = svc_b1.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("One")).unwrap();
    let pid_b2 = svc_b2.create_persona(&mut OsRng, h("bea"), nm("Bea"), sn("Two")).unwrap();
    svc_b1.import_invite(&mut OsRng, &svc_a.export_invite(org_1).unwrap()).unwrap();
    svc_b2.import_invite(&mut OsRng, &svc_a.export_invite(org_2).unwrap()).unwrap();
    let jr_b1 = join_request_of(&svc_b1, &pid_b1);
    let jr_b2 = join_request_of(&svc_b2, &pid_b2);
    let b1_dev = device_kp(&svc_b1, &pid_b1);
    let b2_dev = device_kp(&svc_b2, &pid_b2);

    // The first Organisation's admission binds A's endpoint from Persona 1.
    let (addr, task) = spawn_b_receive(svc_b1, &b1_dev).await;
    tokio::time::timeout(NET, svc_a.admit_member(&mut OsRng, org_1, &jr_b1, addr, org_secret()))
        .await
        .unwrap()
        .unwrap();
    let (_svc_b1, out1) = task.await.unwrap();
    out1.expect("the first Organisation's admission is accepted");

    // The second goes out under Persona 1's device key: a relay R captures it.
    let epoch_before = chain.get(&org_2).unwrap().epoch;
    let (r_addr, r_task) = spawn_recv_one(rand::random()).await;
    tokio::time::timeout(NET, svc_a.admit_member(&mut OsRng, org_2, &jr_b2, r_addr, org_secret()))
        .await
        .unwrap()
        .expect("PR-8qsnhx: A reports success");
    let (r_ep, sender, msg) = r_task.await.unwrap();
    assert_eq!(
        sender,
        device_kp(&svc_a, &pid_1).device_key().unwrap(),
        "PR-8qsnhx: the second Organisation's admission goes out under Persona 1's device key"
    );
    assert_eq!(chain.get(&org_2).unwrap().epoch, Epoch::new(epoch_before.get() + 1), "the chain moved");

    // The joiner checks nothing about the sender, so it commits the admission.
    let (addr, task) = spawn_b_receive(svc_b2, &b2_dev).await;
    r_ep.send(addr, &msg).await.expect("relay to the joiner");
    let (svc_b2, out2) = task.await.unwrap();
    assert_eq!(out2.expect("a chain-valid first admission is committed whoever delivers it").org_id, org_2);
    assert_eq!(svc_b2.list_orgs().len(), 1, "the joiner holds the record");
}

// ---------------------------------------------------------------------------
// Review round 6. Three more pins, of the class round 5 named: state one path
// writes and another reads.
// ---------------------------------------------------------------------------

// PR-322qst — PINS A DEFECT. The node's OWN revocation, arriving on
// `receive_and_verify` rather than on the self-delete path, is committed as an
// ordinary update: B keeps its record of the Organisation it was removed
// from, and its Persona stays Active. REQ-uxv2x2 is not scoped to a path, and
// `revoke_member`'s own doc comment expects exactly this path to remove the
// member. Found by review round 6. Correcting it reddens the assertions below.
// It carries LLR-cja9zv, whose commit clause states this (review round 8).
// verifies: LLR-cja9zv
#[tokio::test(flavor = "multi_thread")]
async fn pr_322qst_an_own_revocation_on_the_ordinary_path_is_committed_not_self_deleted() {
    let mut s = admit_b_directly(setup("pr-322qst").await).await;
    let b_member_id = s.svc_b.list_personas()[0].member_id.expect("B has a member id");
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, s.svc_a.revoke_member(&mut OsRng, s.org_id, b_member_id, Some(b_addr)))
        .await
        .unwrap()
        .unwrap();
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.expect("PR-322qst: committed as an update").epoch, Epoch::new(3));
    let b_dev = s.b_device_kp.device_key().unwrap();
    let rec = svc_b.list_orgs()[0].clone();
    assert!(!rec.trie_members.iter().any(|m| m.device_keys.contains(&b_dev)), "B is not in the trie");
    assert_eq!(svc_b.list_orgs().len(), 1, "PR-322qst: B keeps the record it was removed from");
    assert_eq!(
        svc_b.list_personas()[0].status,
        PersonaStatus::Active,
        "PR-322qst: and its Persona stays Active"
    );
    let reloaded = reopen_store("pr-322qst", "b", "pw_b");
    assert_eq!(reloaded.data().orgs.len(), 1, "PR-322qst: on disk too");
}

// PR-mdv38y, second writer — PINS A DEFECT. `create_organisation` binds the
// founding Persona to the Organisation it founds (LLR-w3fhhg), overwriting the
// binding of a Persona that was a member elsewhere. The self-delete path reads
// that binding, so an ordinary update from the first Organisation makes B
// delete it while B is still in its trie. Scoping `receive_and_verify`'s
// selection, the cure PR-mdv38y first named, does not reach this writer.
// Found by review round 6. It carries LLR-w3fhhg's overwrite clause, which
// states this (review round 7), and LLR-q3aj8z's member-id clause (round 8).
// verifies: LLR-w3fhhg, LLR-q3aj8z
#[tokio::test(flavor = "multi_thread")]
async fn pr_mdv38y_founding_an_organisation_rebinds_a_member_persona() {
    let mut s = admit_b_directly(setup("pr-mdv38y-found").await).await;
    assert_eq!(s.svc_b.list_personas()[0].org_id, Some(s.org_id));
    let b_member_id_before = s.svc_b.list_personas()[0].member_id;
    assert!(b_member_id_before.is_some(), "B holds its org 1 member id");
    let org_y = s.svc_b.create_organisation(&mut OsRng, &s.pid_b).await.unwrap();
    assert_eq!(
        s.svc_b.list_personas()[0].org_id,
        Some(org_y),
        "PR-mdv38y: founding rebinds the member Persona"
    );
    // Founding leaves the Persona's member id as it was: here org 1's, which
    // names no member of org Y. Added 2026-10-05 by review round 8.
    assert_eq!(
        s.svc_b.list_personas()[0].member_id,
        b_member_id_before,
        "LLR-q3aj8z: create_organisation does not touch the member id"
    );

    let jr_c = join_request_for_c(&mut s.svc_a);
    let (b_addr, b_task) = spawn_b_self_delete(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, org_secret()))
        .await
        .unwrap()
        .unwrap();
    let (svc_b, outcome) = b_task.await.unwrap();
    let b_dev = s.b_device_kp.device_key().unwrap();
    let a_rec = rec_of(&s.svc_a, s.org_id);
    assert!(a_rec.trie_members.iter().any(|m| m.device_keys.contains(&b_dev)), "B is still in org 1");
    assert!(
        matches!(
            outcome.unwrap(),
            SelfDeleteOutcome::SelfDeleted { org_id } if org_id == s.org_id
        ),
        "PR-mdv38y: B self-deletes from an Organisation it is still a member of"
    );
    assert!(!svc_b.list_orgs().iter().any(|o| o.org_id == s.org_id));
}

// PR-8qsnhx, the reading side — PINS A DEFECT. `export_join_request` reads
// the address of whatever endpoint the service has bound, and names the
// device key of the Persona asked for. With the endpoint bound from another
// Persona — as the app's receiver binds it, from `personas.first()` — the
// blob names two different devices, and an administrator in Networked mode
// dials a device nothing is listening on. Found by review round 6.
// It carries LLR-437fvx's address clause, which states this (review round 7).
// verifies: LLR-437fvx
#[tokio::test(flavor = "multi_thread")]
async fn pr_8qsnhx_a_join_request_advertises_the_bound_endpoint_not_its_personas() {
    let mut svc = OrgService::new(open_store("pr-8qsnhx-jr", "b", "pw_b"), Box::new(MockChainOps::new()));
    let p1 = svc.create_persona(&mut OsRng, h("one"), nm("Persona"), sn("One")).unwrap();
    let p2 = svc.create_persona(&mut OsRng, h("two"), nm("Persona"), sn("Two")).unwrap();
    let p1_dev = device_kp(&svc, &p1);
    let p2_dev = device_kp(&svc, &p2);
    let svc = svc.with_endpoint(OrgEndpoint::bind(&p1_dev).await.unwrap());

    let jr = join_request_of(&svc, &p2);
    assert_eq!(*jr.device_key.as_bytes(), p2_dev.verifying_key().to_bytes(), "the blob names p2's device");
    let addr: iroh::EndpointAddr = postcard::from_bytes(&jr.node_addr).expect("an address");
    assert_eq!(
        addr.id.as_bytes(),
        p1_dev.verifying_key().as_bytes(),
        "PR-8qsnhx: but advertises p1's endpoint"
    );
}

// PR-8qsnhx, the second reader — PINS A DEFECT. `export_invite` writes the
// address of whatever endpoint the service has bound, beside the device key of
// the Organisation's administrator Persona. With the endpoint bound from
// another Persona the Invite names two devices. Written by review round 9,
// which measured that restricting the address to the administrator's own
// endpoint left the gate green. It carries LLR-qezw3n's "whichever Persona
// bound it" clause.
// verifies: LLR-qezw3n
#[tokio::test(flavor = "multi_thread")]
async fn pr_8qsnhx_an_invite_advertises_the_bound_endpoint_not_its_administrators() {
    let mut svc = OrgService::new(open_store("pr-8qsnhx-inv", "a", "pw_a"), Box::new(MockChainOps::new()));
    let p1 = svc.create_persona(&mut OsRng, h("one"), nm("Persona"), sn("One")).unwrap();
    let p2 = svc.create_persona(&mut OsRng, h("two"), nm("Persona"), sn("Two")).unwrap();
    let org = svc.create_organisation(&mut OsRng, &p2).await.unwrap();
    let p1_dev = device_kp(&svc, &p1);
    let p2_dev = device_kp(&svc, &p2);
    let svc = svc.with_endpoint(OrgEndpoint::bind(&p1_dev).await.unwrap());

    let invite: org_node::blobs::Invite = org_node::blobs::decode(&svc.export_invite(org).unwrap()).unwrap();
    assert_eq!(invite.admin_device_key, p2_dev.device_key().unwrap(), "the Invite names p2, the administrator");
    let addr: iroh::EndpointAddr = postcard::from_bytes(&invite.admin_node_addr).expect("an address");
    assert_eq!(
        addr.id.as_bytes(),
        p1_dev.verifying_key().as_bytes(),
        "PR-8qsnhx: but advertises p1's endpoint"
    );
}

// ---------------------------------------------------------------------------
// Review round 7.
// ---------------------------------------------------------------------------

// PR-u4c2vp, resolved by owner ruling 2026-10-05: nothing about the sender is
// checked on either Receive operation (REQ-ztdza4, LLR-3q63zv). A rogue relay
// R forwards the administrator's genuine admission of C to B's self-delete
// path, and B commits it as an ordinary update because it matches the chain.
// verifies: LLR-3q63zv, LLR-9fvb3y
#[tokio::test(flavor = "multi_thread")]
async fn pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path() {
    let mut s = admit_b_directly(setup("pr-u4c2vp").await).await;
    let epoch_before = s.svc_b.list_orgs()[0].epoch;

    // A admits C, but the push goes to R.
    let jr_c = join_request_for_c(&mut s.svc_a);
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(NET, s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, r_addr, org_secret()))
        .await
        .unwrap()
        .unwrap();
    let (r_ep, _sender, msg) = r_task.await.unwrap();

    // R relays it to B's self-delete path.
    let (b_addr, b_task) = spawn_b_self_delete(s.svc_b, &s.b_device_kp).await;
    r_ep.send(b_addr, &msg).await.expect("relay to B");
    let (svc_b, outcome) = b_task.await.unwrap();
    assert!(
        matches!(
            outcome.expect("a chain-valid update is committed whoever relays it"),
            SelfDeleteOutcome::UpdatedNotRevoked { .. }
        ),
        "committed as an ordinary update"
    );
    assert!(svc_b.list_orgs()[0].epoch > epoch_before, "and B's record moved");
}

// A first admission with NO imported Invite is committed on the chain anchor
// alone (REQ-xa6smf, LLR-mbjfq8, amended 2026-10-05): no Invite is required.
// verifies: REQ-xa6smf, LLR-mbjfq8
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_with_no_imported_invite_rests_on_the_chain_alone() {
    let mut s = setup_with("no-invite-chain-alone", false).await;
    assert!(s.svc_b.list_pending_invites().is_empty(), "B imports no invite");

    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, b_addr, org_secret()),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");

    let (svc_b, outcome) = b_task.await.unwrap();
    assert_eq!(outcome.expect("committed with no Invite").org_id, s.org_id);
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(persona_of(&svc_b, &s.pid_b).status, PersonaStatus::Active);
}

// LLR-xq9nrq, amended 2026-10-05 (owner answer Q1): on a first admission for
// which no Invite was imported, the record's admin_member_key is the
// Organisation public key read from the chain in the same operation, as a
// PersonPublicKey; there is no Invite to take an administrator's key from.
// verifies: LLR-xq9nrq
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_with_no_imported_invite_records_the_chains_key_as_admin_member_key() {
    let mut s = setup_with("no-invite-admin-key", false).await;

    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, b_addr, org_secret()),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");

    let (svc_b, outcome) = b_task.await.unwrap();
    outcome.expect("committed with no Invite");
    let published = s.chain.get(&s.org_id).unwrap().org_pub_key;
    let rec = rec_of(&svc_b, s.org_id);
    assert_eq!(rec.admin_member_key.as_bytes(), published.as_bytes(), "the chain's key, with no Invite");
    assert_eq!(rec.org_pub_key, published);
}

// The self-delete path refuses a change about an Organisation it holds no
// record of, with `OrgNotOnChain`, and writes nothing. That is the mechanism
// behind Gap 20 (the app receives only on this path, so a joiner's first
// admission cannot complete there). The error's name is wrong — the chain
// does hold the Organisation — and that is recorded for the fix change, not
// endorsed. Stated by no requirement until review round 7.
// verifies: LLR-379hnv
#[tokio::test(flavor = "multi_thread")]
async fn the_self_delete_path_refuses_an_organisation_it_holds_no_record_of() {
    let mut s = setup("selfdel-no-record").await;
    let (addr, task) = spawn_b_self_delete(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, addr, org_secret()),
    )
    .await
    .unwrap()
    .unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert_eq!(outcome.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert!(svc_b.list_orgs().is_empty(), "nothing is written");
    assert_eq!(svc_b.list_pending_invites().len(), 1, "the invite is not consumed");
    // Nor is any Persona touched. Added 2026-10-05 by review round 8: marking
    // every Persona Revoked on this refusal was green.
    let persona_b = persona_of(&svc_b, &s.pid_b);
    assert_eq!(persona_b.status, PersonaStatus::Proposed, "B's Persona is untouched");
    assert_eq!(persona_b.org_id, None);
    assert_eq!(persona_b.member_id, None);
    let reloaded = reopen_store("selfdel-no-record", "b", "pw_b");
    assert!(reloaded.data().orgs.is_empty());
    assert!(
        reloaded.data().personas.iter().all(|p| p.status == PersonaStatus::Proposed),
        "and nothing about a Persona reaches the disk"
    );
}

// ---------------------------------------------------------------------------
// Review round 8. Two behaviours no test could falsify.
// ---------------------------------------------------------------------------

// LLR-q8emds: the pending invite is discarded only once the first admission
// has committed. Consuming it before verification was green, because every
// first admission a test sent either committed or was refused before the
// point a consumption could sit. This one comes from the administrator's own
// device key and fails verification. *Merged 2026-10-05 into worktree-person-shared-types:*
// master altered the envelope's mark after signing, which failed the
// signature; the Envelope has no signature on this branch, and a raised mark
// is still fresh to a first admission, so the Change set bytes are spoiled
// instead and verification refuses them at the decode.
// verifies: LLR-q8emds
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_that_fails_verification_leaves_the_invite_pending() {
    let mut s = setup("invite-kept").await;
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, r_addr, org_secret()),
    )
    .await
    .unwrap()
    .unwrap();
    let (_r_ep, _sender, mut msg) = r_task.await.unwrap();
    msg.envelope.delta_bytes = vec![0xffu8; 16];

    let pid_a = s.svc_a.list_personas()[0].persona_id.clone();
    let a_again = OrgEndpoint::bind(&device_kp(&s.svc_a, &pid_a)).await.unwrap();
    assert_eq!(s.svc_b.list_pending_invites().len(), 1, "B holds A's invite");
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, a_again.send(b_addr, &msg)).await.unwrap().unwrap();

    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(
        result.unwrap_err(),
        OrgNodeError::MalformedDelta,
        "an altered envelope must not verify"
    );
    assert!(svc_b.list_orgs().is_empty(), "nothing is committed");
    assert_eq!(svc_b.list_pending_invites().len(), 1, "the invite is still pending");
    assert_eq!(reopen_store("invite-kept", "b", "pw_b").data().pending_invites.len(), 1);
}

/// A chain that hands back a pure-proxy account at genesis, as
/// `SubxtChainOps` does and `MockChainOps` does not, and records the proxy
/// account every update is submitted with.
#[derive(Clone)]
struct ProxyChain {
    inner: MockChainOps,
    seen: std::sync::Arc<std::sync::Mutex<Vec<Option<ChainAccount>>>>,
}

/// The pure-proxy account `ProxyChain` hands back at genesis.
fn proxy() -> ChainAccount {
    ChainAccount::new([0x5au8; 32])
}

#[async_trait::async_trait]
impl ChainOps for ProxyChain {
    async fn submit_genesis(
        &self,
        genesis_root: RootHash,
        org_pub_key: OrgPublicKey,
    ) -> Result<(OrgId, Option<ChainAccount>), OrgNodeError> {
        let (org_id, none) = self.inner.submit_genesis(genesis_root, org_pub_key).await?;
        assert_eq!(none, None, "the mock returns no proxy of its own");
        Ok((org_id, Some(proxy())))
    }

    async fn submit_update(
        &self,
        org_id: OrgId,
        new_root: RootHash,
        org_pub_key: OrgPublicKey,
        expected_epoch: Epoch,
        proxy_account: Option<ChainAccount>,
    ) -> Result<(), OrgNodeError> {
        self.seen.lock().unwrap().push(proxy_account);
        self.inner.submit_update(org_id, new_root, org_pub_key, expected_epoch, proxy_account).await
    }

    async fn read_state(&self, org_id: OrgId) -> Result<Option<org_node::chain::OrgState>, OrgNodeError> {
        self.inner.read_state(org_id).await
    }
}

// The pure-proxy account the chain returns at genesis is kept in the record,
// on disk, and handed back on every update the administrator submits, so the
// production chain client can find the proxy after a restart. No test could
// see it: `MockChainOps` returns `None` and ignores what it is given, so not
// storing it and not passing it were both green.
// verifies: LLR-dzte8x, LLR-3v5nu9, LLR-drgdy8
#[tokio::test(flavor = "multi_thread")]
async fn the_proxy_account_from_genesis_is_kept_and_passed_on_every_update() {
    let chain = ProxyChain { inner: MockChainOps::new(), seen: Default::default() };
    let mut svc_a = OrgService::new(open_store("proxy", "a", "pw_a"), Box::new(chain.clone()));
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = svc_a.create_organisation(&mut OsRng, &pid_a).await.unwrap();
    assert_eq!(svc_a.list_orgs()[0].proxy_account, Some(proxy()), "LLR-dzte8x: kept in the record");
    assert_eq!(
        reopen_store("proxy", "a", "pw_a").data().orgs[0].proxy_account,
        Some(proxy()),
        "LLR-dzte8x: and on disk"
    );

    let jr = join_request_for_c(&mut svc_a);
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    tokio::time::timeout(NET, svc_a.admit_member(&mut OsRng, org_id, &jr, sink_addr, org_secret()))
        .await
        .unwrap()
        .unwrap();
    let _ = sink.await.unwrap();
    assert_eq!(*chain.seen.lock().unwrap(), vec![Some(proxy())], "LLR-3v5nu9: admission passes it");

    let c_id = id_by_handle(&svc_a.list_orgs()[0], "carol");
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    tokio::time::timeout(NET, svc_a.revoke_member(&mut OsRng, org_id, c_id, Some(sink_addr)))
        .await
        .unwrap()
        .unwrap();
    let _ = sink.await.unwrap();
    assert_eq!(
        *chain.seen.lock().unwrap(),
        vec![Some(proxy()), Some(proxy())],
        "LLR-drgdy8: revocation passes it"
    );
}

// ---- added 2026-10-05 by review round 1 of worktree-person-shared-types ----

/// The trie a record's member snapshots describe, rebuilt by the test as a
/// peer would, so the test can author a Change set against it.
fn trie_of(rec: &OrgRecord) -> org_node::test_fixtures::Trie {
    use org_members::MemberLeaf;
    let leaves = rec
        .trie_members
        .iter()
        .map(|s| {
            MemberLeaf::new(
                s.id,
                s.handle.clone(),
                s.member_key,
                s.name.clone(),
                s.surname.clone(),
                s.device_keys.clone(),
            )
            .unwrap()
        })
        .collect();
    org_node::test_fixtures::Trie::genesis(leaves).unwrap()
}

/// A captures the WireMessage admitting C that it sends to B: A admits C into
/// a sink. The chain is then at epoch 3 and B's record still at epoch 2.
async fn capture_c_admission(s: &mut Setup) -> WireMessage {
    let jr_c = join_request_for_c(&mut s.svc_a);
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    tokio::time::timeout(NET, s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, sink_addr, org_secret()))
        .await
        .expect("admit_member(C) timed out")
        .expect("admit_member(C) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(3));
    let (_sink_ep, _sender, msg) = sink.await.unwrap();
    msg
}

// The reviewer's jam, on the ordinary receive path. A device B accepts (the
// administrator's) rebuilds the genuine Envelope with the Sequence number
// u64::MAX. Before REQ-txvtm9 bound the number to the chain's epoch, B
// committed it, and every later genuine Envelope was StaleSeq forever. Now it
// is refused, the record and the mark stay where they were, and the genuine
// Envelope still commits with the mark at the chain's epoch.
// verifies: REQ-txvtm9, REQ-mr5abb, LLR-9f5hmr
#[tokio::test(flavor = "multi_thread")]
async fn a_sequence_number_beyond_the_chain_epoch_cannot_jam_the_receive_path() {
    let mut s = admit_b_directly(setup("jam-receive").await).await;
    let before = rec_of(&s.svc_b, s.org_id);
    let genuine = capture_c_admission(&mut s).await;
    let mut jam = genuine.clone();
    jam.envelope.parent_seq = SequenceNumber::new(u64::MAX);

    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    s.svc_a.endpoint().expect("A endpoint").send(b_addr, &jam).await.expect("send jam");
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(
        result.unwrap_err(),
        OrgNodeError::SeqNotEpoch { seq: u64::MAX, epoch: 3 },
        "a Sequence number other than the chain's epoch must be refused"
    );
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!(after.last_seq, before.last_seq, "the mark must not move");
    assert_eq!(after.epoch, Epoch::new(2));
    assert_eq!(after.root_hash, before.root_hash);

    let (b_addr, b_task) = spawn_b_receive(svc_b, &s.b_device_kp).await;
    s.svc_a.endpoint().unwrap().send(b_addr, &genuine).await.expect("send genuine");
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.expect("the genuine Envelope still commits").epoch, Epoch::new(3));
    assert_eq!(rec_of(&svc_b, s.org_id).last_seq, SequenceNumber::new(3), "the mark is the chain's epoch");
}

// The same jam on the receive path that acts on the node's own removal, on
// its update branch: the Change set (C's admission) leaves B in the record.
// verifies: REQ-txvtm9, REQ-mr5abb, LLR-9f5hmr
#[tokio::test(flavor = "multi_thread")]
async fn a_sequence_number_beyond_the_chain_epoch_cannot_jam_the_self_delete_path() {
    let mut s = admit_b_directly(setup("jam-self-delete").await).await;
    let before = rec_of(&s.svc_b, s.org_id);
    let genuine = capture_c_admission(&mut s).await;
    let mut jam = genuine.clone();
    jam.envelope.parent_seq = SequenceNumber::new(u64::MAX);

    let (b_addr, b_task) = spawn_b_self_delete(s.svc_b, &s.b_device_kp).await;
    s.svc_a.endpoint().expect("A endpoint").send(b_addr, &jam).await.expect("send jam");
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::SeqNotEpoch { seq: u64::MAX, epoch: 3 });
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!(after.last_seq, before.last_seq, "the mark must not move");
    assert_eq!(after.epoch, Epoch::new(2));
    assert_eq!(after.root_hash, before.root_hash);

    let (b_addr, b_task) = spawn_b_self_delete(svc_b, &s.b_device_kp).await;
    s.svc_a.endpoint().unwrap().send(b_addr, &genuine).await.expect("send genuine");
    let (svc_b, result) = b_task.await.unwrap();
    assert!(matches!(result, Ok(SelfDeleteOutcome::UpdatedNotRevoked { .. })), "got {result:?}");
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.last_seq), (Epoch::new(3), SequenceNumber::new(3)), "committed at the chain's epoch");
}

// REQ-ag6kqm as amended: nothing about the sender is checked, so Change set
// bytes that do not decode are refused as what they are, whoever sends them,
// and B's record does not move.
// verifies: REQ-ag6kqm, LLR-mcdh85, LLR-9sknpa
#[tokio::test(flavor = "multi_thread")]
async fn a_malformed_change_set_from_a_device_outside_the_record_is_refused_as_malformed() {
    let s = admit_b_directly(setup("known-org-pre-decode").await).await;
    let before = rec_of(&s.svc_b, s.org_id);
    let envelope = org_node::Envelope { org_id: s.org_id, parent_seq: SequenceNumber::new(3), delta_bytes: vec![0xff; 16] };
    let msg = WireMessage { envelope, org_secret: None, genesis_snapshot: None };

    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    let rogue = OrgEndpoint::bind(&DeviceSeed::from(ROGUE_SEED).signing_keypair()).await.unwrap();
    rogue.send(b_addr, &msg).await.expect("rogue send");
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::MalformedDelta);
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.last_seq, after.root_hash), (before.epoch, before.last_seq, before.root_hash));
}

// LLR-e5c9ud's administrator exclusion. The administrator's own node holds the
// administrator's Persona and C's, and both DevicePublicKeys are in its record. A
// receives an update from B's device. The Persona marked is C's, never the
// administrator's, though the administrator's comes first in the store.
// verifies: LLR-e5c9ud
#[tokio::test(flavor = "multi_thread")]
async fn the_administrators_own_persona_is_never_the_one_a_receive_marks() {
    let mut s = admit_b_directly(setup("admin-excluded").await).await;
    let _ = capture_c_admission(&mut s).await;
    let rec_a = rec_of(&s.svc_a, s.org_id);
    let c_id = id_by_handle(&rec_a, "carol");
    let admin_pid = s.svc_a.list_personas()[0].persona_id.clone();
    let c_pid = s.svc_a.list_personas()[1].persona_id.clone();
    assert_eq!(persona_of(&s.svc_a, &c_pid).status, PersonaStatus::Proposed);

    // B's device authors D's admission against A's record; the chain agrees.
    let dave = org_members::MemberLeaf::new(
        org_members::MemberId::new([0xd0u8; 32]),
        org_members::Handle::parse("dave").unwrap(),
        MemberSeed::from([0xd1u8; 32]).x25519_keypair().member_key().unwrap(),
        org_members::Name::parse("Dave").unwrap(),
        org_members::Surname::parse("Diver").unwrap(),
        vec![DeviceSeed::from([0xd2u8; 32]).signing_keypair().device_key().unwrap()],
    )
    .unwrap();
    let (new_trie, delta) = trie_of(&rec_a).add_member(dave).unwrap().recalculate().unwrap();
    let on_chain = s.chain.get(&s.org_id).unwrap();
    let epoch = Epoch::new(on_chain.epoch.get() + 1);
    s.chain.set(
        s.org_id,
        org_node::chain::OrgState { root_hash: new_trie.root_hash().unwrap(), org_pub_key: on_chain.org_pub_key, epoch },
    );
    let envelope = org_node::Envelope::build(s.org_id, SequenceNumber::new(epoch.get()), &delta).unwrap();
    let msg = WireMessage { envelope, org_secret: None, genesis_snapshot: None };

    let a_device_kp = device_kp(&s.svc_a, &admin_pid);
    let (a_addr, a_task) = spawn_b_receive(s.svc_a, &a_device_kp).await;
    let from_b = OrgEndpoint::bind(&s.b_device_kp).await.unwrap();
    from_b.send(a_addr, &msg).await.expect("send from B");
    let (svc_a, result) = a_task.await.unwrap();
    assert_eq!(result.expect("A commits D's admission").epoch, epoch);

    let admin = persona_of(&svc_a, &admin_pid);
    assert_eq!(admin.member_id, None, "the administrator's Persona must not be marked");
    let carol = persona_of(&svc_a, &c_pid);
    assert_eq!(carol.status, PersonaStatus::Active, "C's Persona is the one marked");
    assert_eq!(carol.member_id, Some(c_id));
}

/// `blob` with every occurrence of `from` replaced by `to`, re-encoded: an
/// Invite as a tampering carrier would hand it on.
fn tampered(blob: &str, from: &[u8; 32], to: &[u8; 32]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let mut bytes = STANDARD.decode(blob).unwrap();
    let at = bytes.windows(32).position(|w| w == from).expect("the key is in the blob");
    bytes[at..at + 32].copy_from_slice(to);
    STANDARD.encode(bytes)
}

// The Invite is not compared with the chain: REQ-xa6smf requires no Invite,
// so an Invite whose Organisation public key differs from the chain's does not
// stop a chain-valid first admission (owner ruling 2026-10-05).
// *Amended 2026-10-05 (review round 4, finding-2).* The record it commits
// holds the chain's Organisation public key, not the Invite's (LLR-rys5nx):
// recording the Invite's key left every test green until this assertion.
// verifies: REQ-xa6smf, LLR-j83kc8, LLR-rys5nx
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_whose_invite_names_another_organisation_key_is_committed() {
    let mut s = setup_with("invite-other-org-key", false).await;
    let genuine = s.svc_a.export_invite(s.org_id).unwrap();
    let published = *s.chain.get(&s.org_id).unwrap().org_pub_key.as_bytes();
    let other = MemberSeed::from([0x6au8; 32]).x25519_keypair().public_bytes();
    s.svc_b.import_invite(&mut OsRng, &tampered(&genuine, &published, &other)).unwrap();

    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, b_addr, org_secret()),
    )
    .await
    .expect("admit_member timed out")
    .expect("admit_member failed");
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.expect("committed: the Invite is not compared with the chain").org_id, s.org_id);
    assert_eq!(svc_b.list_orgs().len(), 1, "B committed the OrgRecord");
    assert_eq!(
        svc_b.list_orgs()[0].org_pub_key.as_bytes(),
        &published,
        "the record holds the chain's Organisation public key, not the Invite's"
    );
    assert_eq!(persona_of(&svc_b, &s.pid_b).status, PersonaStatus::Active);
}

// The abnormal case of the no-Invite first admission (review round 4,
// finding-4): with no Invite imported, a first admission whose record does not
// reach the chain's Membership root is refused, and the refusal commits
// nothing — no record, no Persona marked, nothing on disk. "On the chain anchor
// alone" (LLR-mbjfq8) cuts both ways: without the anchor there is no commit.
// The chain's root is moved after A's admission, at the same epoch, so the
// Envelope's Sequence number still equals the chain's epoch and the root is
// the check that refuses it.
// verifies: REQ-xa6smf, LLR-mbjfq8
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_with_no_imported_invite_that_misses_the_chain_root_commits_nothing() {
    let mut s = setup_with("no-invite-root-miss", false).await;
    assert!(s.svc_b.list_pending_invites().is_empty(), "B imports no invite");

    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, r_addr, org_secret()),
    )
    .await
    .unwrap()
    .unwrap();
    let (_r_ep, _sender, msg) = r_task.await.unwrap();

    let on_chain = s.chain.get(&s.org_id).unwrap();
    s.chain.set(
        s.org_id,
        org_node::chain::OrgState {
            root_hash: RootHash::new([0x5au8; 32]),
            org_pub_key: on_chain.org_pub_key,
            epoch: on_chain.epoch,
        },
    );

    let relay = OrgEndpoint::bind(&DeviceSeed::from(ROGUE_SEED).signing_keypair()).await.unwrap();
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, relay.send(b_addr, &msg)).await.unwrap().unwrap();

    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::RootMismatch);
    assert!(svc_b.list_orgs().is_empty(), "no record is committed");
    assert!(svc_b.list_pending_invites().is_empty(), "and no invite appears");
    let persona_b = persona_of(&svc_b, &s.pid_b);
    assert_eq!(persona_b.status, PersonaStatus::Proposed, "B's Persona is untouched");
    assert_eq!(persona_b.org_id, None);
    assert_eq!(persona_b.member_id, None);
    let reloaded = reopen_store("no-invite-root-miss", "b", "pw_b");
    assert!(reloaded.data().orgs.is_empty(), "nothing reaches the disk");
    assert!(reloaded.data().pending_invites.is_empty());
    assert!(
        reloaded.data().personas.iter().all(|p| p.status == PersonaStatus::Proposed
            && p.org_id.is_none()
            && p.member_id.is_none()),
        "and nothing about a Persona reaches the disk"
    );
}
