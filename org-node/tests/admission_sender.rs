#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests: the sender cross-checks of `OrgService::receive_and_verify`
//! over the service API, with real iroh endpoints on loopback and a shared
//! `MockChainOps`. Offline — no live chain, no relay.
//!
//! The admin (A) always produces a chain-valid, correctly signed admission
//! envelope. What varies is *who delivers it* to the member (B):
//!
//!   * first admission — B has imported A's invite, so the authenticated QUIC
//!     sender must equal the invite's `admin_device_key`; a rogue endpoint R
//!     relaying the identical message is rejected (REQ-xa6smf);
//!   * after admission — the authenticated sender must be a device of a member
//!     in the newly committed trie; A's own device passes, R's does not
//!     (REQ-ztdza4).
//!
//! The same setup carries the MemberId tests: ids are random, not keys, and a
//! re-admission with the same keys gets a fresh id (REQ-d9g6nt).
//!
//! The gate:
//! `cargo test -p org-node --features app,test-support --test admission_sender`

use std::time::Duration;

use org_node::blobs::JoinRequest;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::service::{ChainOps, MockChainOps, OrgService, SelfDeleteOutcome};
use org_node::store::{OrgRecord, PersonaRecord, PersonaStatus, PersonaStore};
use org_node::transport::endpoint::OrgEndpoint;
use org_node::transport::wire::WireMessage;
use rand::rngs::OsRng;

const NET: Duration = Duration::from_secs(30);
const ORG_SECRET: Option<[u8; 32]> = Some([0xffu8; 32]);
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

/// Reopen a store that `open_store` already created, WITHOUT wiping it — the
/// only way to see what actually reached the disk.
fn reopen_store(tag: &str, party: &str, password: &str) -> PersonaStore {
    PersonaStore::open(store_dir(tag, party).join("store.bin"), password).unwrap()
}

/// A persona's record, cloned.
fn persona_of(svc: &OrgService, persona_id: &str) -> PersonaRecord {
    svc.list_personas()
        .iter()
        .find(|p| p.persona_id == persona_id)
        .expect("persona not found")
        .clone()
}

/// The device keypair of a persona, from its persisted `device_seed`.
fn device_kp(svc: &OrgService, persona_id: &str) -> SigningKeypair {
    SigningKeypair::from_seed(persona_of(svc, persona_id).device_seed)
}

/// The member and device keypairs of `svc`'s first persona — the
/// administrator in every fixture that calls this — from its persisted seeds.
fn admin_keys(svc: &OrgService) -> (SigningKeypair, SigningKeypair) {
    let admin = &svc.list_personas()[0];
    (SigningKeypair::from_seed(admin.member_seed), SigningKeypair::from_seed(admin.device_seed))
}

/// A persona's join request, exported and imported back as an administrator
/// receives it.
fn join_request_of(svc: &OrgService, persona_id: &str) -> JoinRequest {
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
fn id_by_handle(rec: &OrgRecord, handle: &str) -> [u8; 32] {
    rec.trie_members.iter().find(|m| m.handle == handle).expect("member in the record").id
}

/// A well-formed peer identity, from `seed`, carrying NO transport addresses.
/// The call sites say why a dial to it fails at once.
fn dead_addr(seed: [u8; 32]) -> iroh::EndpointAddr {
    iroh::EndpointId::from_bytes(SigningKeypair::from_seed(seed).device_key().as_bytes())
        .map(|id| iroh::EndpointAddr::from_parts(id, std::iter::empty()))
        .expect("a well-formed endpoint identity")
}

/// Stories 1–2 over the service API: A creates a persona and the organisation
/// (epoch 1); B creates a persona, imports A's invite (persisting
/// `admin_device_key` for the cross-check) and exports a join request that A
/// imports. A's endpoint is bound from A's persona `device_seed`, so the
/// authenticated sender on B's side equals the invite's `admin_device_key`.
struct Setup {
    chain: MockChainOps,
    svc_a: OrgService,
    svc_b: OrgService,
    org_id: OrgId,
    pid_b: String,
    b_device_kp: SigningKeypair,
    join_request_b: JoinRequest,
}

async fn setup(tag: &str) -> Setup {
    let chain = MockChainOps::new();

    let mut svc_a = OrgService::new(open_store(tag, "a", "pw_a"), Box::new(chain.clone()));
    let mut svc_b = OrgService::new(open_store(tag, "b", "pw_b"), Box::new(chain.clone()));

    // Story 1: A creates persona + org.
    let pid_a = svc_a.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();
    let org_id = svc_a.create_organisation(&mut OsRng, &pid_a).await.unwrap();
    assert_eq!(chain.get(&org_id).unwrap().epoch, 1);

    // A's outbound endpoint, bound from A's persona device seed.
    let ep_a = OrgEndpoint::bind(&device_kp(&svc_a, &pid_a)).await.unwrap();
    let svc_a = svc_a.with_endpoint(ep_a);

    // Story 2: B creates persona; A exports the invite; B imports it.
    let pid_b = svc_b.create_persona(&mut OsRng, "bob", "Bob", "Builder").unwrap();
    let invite_blob = svc_a.export_invite(org_id).unwrap();
    let invite = svc_b.import_invite(&mut OsRng, &invite_blob).unwrap();
    assert_eq!(invite.org_id, org_id);

    // B exports a join request; A imports it.
    let join_request_b = join_request_of(&svc_b, &pid_b);
    assert_eq!(join_request_b.handle, "bob");

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
    tokio::task::JoinHandle<(OrgEndpoint, org_members::P2pDeviceKey, WireMessage)>,
) {
    let ep = OrgEndpoint::bind(&SigningKeypair::from_seed(seed)).await.unwrap();
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
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, b_addr, ORG_SECRET),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, 2, "admitting B must bump to epoch 2");

    let (svc_b, outcome) = b_task.await.unwrap();
    let outcome = outcome.expect("B's direct admission from A must verify");
    assert_eq!(outcome.epoch, 2);
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(svc_b.list_orgs()[0].epoch, 2);
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 2, "admin + B");
    s.svc_b = svc_b;
    s
}

/// A creates a further persona C and imports its join request. A's admin
/// persona is still found by member key, so A stays the org's admin.
fn join_request_for_c(svc_a: &mut OrgService) -> JoinRequest {
    let pid_c = svc_a.create_persona(&mut OsRng, "carol", "Carol", "Coder").unwrap();
    let jr = join_request_of(svc_a, &pid_c);
    assert_eq!(jr.handle, "carol");
    jr
}

// Abnormal case of the first-admission invite cross-check: the envelope is
// chain-valid and correctly signed by the admin, and the org_secret and genesis
// snapshot are genuine, but it arrives from a device that is not the invite's
// `admin_device_key`. B must reject it and commit nothing.
// verifies: REQ-xa6smf, LLR-j83kc8
#[tokio::test(flavor = "multi_thread")]
async fn first_admission_from_a_device_other_than_the_invites_admin_is_rejected() {
    let mut s = setup("first-rogue").await;

    // R waits for A's push; A "admits B" but is handed R's address.
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, r_addr, ORG_SECRET),
    )
    .await
    .expect("admit_member(B via R) timed out")
    .expect("admit_member(B via R) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, 2);

    let (ep_r, sender_seen_by_r, msg) = r_task.await.unwrap();
    assert_eq!(
        sender_seen_by_r,
        device_kp(&s.svc_a, &s.svc_a.list_personas()[0].persona_id).device_key(),
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
    assert!(
        matches!(result, Err(OrgNodeError::BadSignature)),
        "B must reject a first admission whose sender is not the invite's admin device, got {result:?}"
    );
    assert!(svc_b.list_orgs().is_empty(), "B must not have committed an OrgRecord");
    let persona_b = persona_of(&svc_b, &s.pid_b);
    assert_ne!(persona_b.status, PersonaStatus::Active, "B's persona must not be Active");
    assert_eq!(persona_b.org_id, None);
    // "Leaving its record unchanged" includes the invite. Added 2026-10-03:
    // the falsifiability sweep clearing `pending_invites` on this rejection
    // path left the suite green, so the clause was unevidenced. A consumed
    // invite here would mean a rogue relay could make B unable to accept the
    // genuine admission that follows.
    assert!(
        !svc_b.list_pending_invites().is_empty(),
        "a rejected first admission must leave B's imported invite in place"
    );
}

// Normal case of the post-admission sender check: after B has committed epoch 2,
// a further update (C's admission) pushed by the admin's own device — a member
// device in the new trie — is verified and committed.
// verifies: REQ-ztdza4, LLR-u6rq4s, LLR-cja9zv
#[tokio::test(flavor = "multi_thread")]
async fn update_from_the_admin_after_admission_is_committed() {
    let mut s = admit_b_directly(setup("update-admin").await).await;
    let seq_at_2 = s.svc_b.list_orgs()[0].last_seq;

    let jr_c = join_request_for_c(&mut s.svc_a);

    // B waits for the next update; A admits C, pushing the envelope to B.
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, ORG_SECRET))
        .await
        .expect("admit_member(C) timed out")
        .expect("admit_member(C) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, 3, "admitting C must bump to epoch 3");

    let (svc_b, result) = b_task.await.unwrap();
    let outcome = result.expect("B must accept an update sent by the admin's own device");
    assert_eq!(outcome.org_id, s.org_id);
    assert_eq!(outcome.epoch, 3, "B must commit epoch 3");
    assert_eq!(
        outcome.root,
        *s.chain.get(&s.org_id).unwrap().root_hash.as_bytes(),
        "B's committed root must match the on-chain root"
    );
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(svc_b.list_orgs()[0].epoch, 3);
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 3, "admin + B + C");
    // The sequence mark advances with the commit. Added 2026-10-03: this is
    // the UPDATE branch of the commit, distinct from the first-admission
    // branch that `a_committed_admission_reaches_the_disk_and_consumes_the_invite`
    // covers, and the sweep found zeroing it here reddened nothing. Compared
    // against the mark held BEFORE this update, so the assertion cannot be
    // satisfied by the value it is reading.
    assert!(
        svc_b.list_orgs()[0].last_seq > seq_at_2,
        "the mark must advance with the commit: was {seq_at_2}, now {}",
        svc_b.list_orgs()[0].last_seq
    );
}

// Abnormal case of the post-admission sender check: the same chain-valid,
// admin-signed update relayed by a device that is not in the committed trie
// must be rejected, and B's record must stay at epoch 2.
//
// It also covers REQ-mr5abb's "never on a rejection at any step" over the
// whole receive operation rather than over `SeqGuard` alone. This is the
// deepest rejection the service has: the envelope's org binding, signature,
// sequence number, base root, epoch and chain root all pass, and the message
// is refused only afterwards, on the sender cross-check. The persisted
// high-water mark (`OrgRecord.last_seq`) must be exactly where it was.
// verifies: REQ-ztdza4, REQ-mr5abb, LLR-u6rq4s
#[tokio::test(flavor = "multi_thread")]
async fn update_relayed_by_a_non_member_after_admission_is_rejected() {
    let mut s = admit_b_directly(setup("update-rogue").await).await;
    let root_at_2 = s.svc_b.list_orgs()[0].root_hash;

    let jr_c = join_request_for_c(&mut s.svc_a);

    // R captures A's C-admission push.
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(NET, s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, r_addr, ORG_SECRET))
        .await
        .expect("admit_member(C via R) timed out")
        .expect("admit_member(C via R) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, 3);
    let (ep_r, _sender, msg) = r_task.await.unwrap();

    // The mark B holds before the rejected relay, to compare after it.
    let last_seq_at_2 = s.svc_b.list_orgs()[0].last_seq;

    // R relays it to B.
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, ep_r.send(b_addr, &msg))
        .await
        .expect("R relay send timed out")
        .expect("R relay send failed");

    let (svc_b, result) = b_task.await.unwrap();
    assert!(
        matches!(result, Err(OrgNodeError::BadSignature)),
        "B must reject an update whose sender is not a member device, got {result:?}"
    );
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(
        svc_b.list_orgs()[0].last_seq, last_seq_at_2,
        "the Sequence-number high-water mark must not advance on a rejection"
    );
    assert_eq!(svc_b.list_orgs()[0].epoch, 2, "B's record must still be at epoch 2");
    assert_eq!(svc_b.list_orgs()[0].root_hash, root_at_2, "B's root must be unchanged");
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 2, "admin + B only");
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

// The counterpart to the two cross-check tests above: on the removal path the
// sender is authenticated and deliberately NOT checked against the record.
// A revocation relayed by a rogue device — the same device whose relay is
// refused by `receive_and_verify` in the test above — is still acted upon,
// because a node being removed cannot be required to find the remover in a
// record it is no longer part of. This test exists to make that deliberate
// absence observable; if a sender check were added to the removal path, this
// test reddens and the decision gets re-made rather than drifting.
// verifies: LLR-3q63zv, LLR-6p4pj2
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
    // What R captured really did come from A, so the only thing that differs
    // when R relays it is who B authenticates on the connection.
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

    // A well-formed, correctly signed message for an org that was never put on
    // the mock chain. B holds no record of it either, so if the chain read did
    // NOT come first this would be taken for a first admission.
    let stranger = SigningKeypair::from_seed([0x5au8; 32]);
    let (delta, _) = org_node::test_fixtures::admit_member_delta(&stranger);
    let absent_org = OrgId::new([0xeeu8; 20]);
    let envelope =
        org_node::SignedDeltaEnvelope::build(absent_org, 1, &delta, &stranger).unwrap();
    let msg = WireMessage { envelope, org_secret: None, genesis_snapshot: None };

    let relay = OrgEndpoint::bind(&SigningKeypair::from_seed([0x5bu8; 32])).await.unwrap();
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
// verifies: REQ-nhe2zu, REQ-xa6smf, LLR-cja9zv, LLR-q8emds
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
    assert_eq!(in_memory.epoch, 2);
    assert_ne!(in_memory.last_seq, 0, "the sequence mark must have been written");

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
    assert_eq!(in_memory.epoch, 2, "A must hold the epoch it submitted");
    assert_eq!(in_memory.trie_members.len(), 2, "admin + B");
    assert_ne!(in_memory.last_seq, 0, "A must hold the sequence it signed over");

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
        orgs[0].trie_members.iter().any(|m| m.handle == "bob"),
        "B must be among A's persisted members"
    );

    // The root A records is the one it published. Added 2026-10-04 by review
    // round 6, which measured that leaving `org_rec.root_hash` unwritten was
    // green: nothing in the library reads it, so only a test can.
    let published = *s.chain.get(&s.org_id).unwrap().root_hash.as_bytes();
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
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, dead_addr([0x6fu8; 32]), ORG_SECRET),
    )
    .await
    .expect("admit_member to a dead peer timed out");

    assert!(result.is_err(), "a push to a peer that is not listening must fail");

    // The chain moved — `submit_update` runs before the send, and that is the
    // documented order. What must NOT have moved is A's own record.
    assert_eq!(
        s.chain.get(&s.org_id).unwrap().epoch,
        epoch_before + 1,
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
    assert_ne!(admin_snap.id, admin_snap.member_key, "the admin's id is its member key");
    assert_ne!(admin_snap.id, admin_snap.device_keys[0], "the admin's id is its device key");

    // A admits B, delivered to B's own endpoint as in `admit_b_directly`.
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    let id_b = tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, b_addr, ORG_SECRET),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");
    let (_svc_b, outcome) = b_task.await.unwrap();
    outcome.expect("B's admission from A must verify");

    assert_ne!(id_b, s.join_request_b.member_key, "B's id is its member key");
    assert_ne!(id_b, s.join_request_b.device_key, "B's id is its device key");
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

    let mut ids: Vec<[u8; 32]> = Vec::new();
    for round in 0..3 {
        // Admit the same join request.
        let (addr, sink) = spawn_recv_one(rand::random()).await;
        let id = tokio::time::timeout(
            NET,
            s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, addr, ORG_SECRET),
        )
        .await
        .expect("admit_member(B) timed out")
        .unwrap_or_else(|e| panic!("admission round {round} with the same keys failed: {e:?}"));
        sink.await.unwrap();

        assert_ne!(id, s.join_request_b.member_key, "round {round}: id is B's member key");
        assert_ne!(id, s.join_request_b.device_key, "round {round}: id is B's device key");
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
    let pid = svc.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();

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
// stripped. B must refuse it with the explicit error — not reconstruct a
// record — and commit nothing. The refusal precedes the sender cross-check,
// so the relaying device does not matter here.
// verifies: REQ-d9g6nt, LLR-j6j95z
#[tokio::test(flavor = "multi_thread")]
async fn first_admission_without_a_record_snapshot_is_refused() {
    let mut s = setup("no-snapshot").await;

    // Capture A's genuine admission of B.
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, r_addr, ORG_SECRET),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");
    let (ep_r, _sender, mut msg) = r_task.await.unwrap();
    assert!(msg.genesis_snapshot.is_some(), "A sent a snapshot to strip");
    msg.genesis_snapshot = None;

    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, ep_r.send(b_addr, &msg))
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

    // LLR-tax3pm, read off the envelope A transmitted rather than off A's own
    // record: the mark is one past what A held, and the signature is the
    // administrator's MEMBER key, not the device key the connection
    // authenticated.
    assert_eq!(
        msg.envelope.parent_seq,
        before.last_seq + 1,
        "the revocation envelope must carry the mark one past the record's last"
    );
    let (member_kp, admin_device_kp) = admin_keys(&s.svc_a);
    assert_ne!(
        member_kp.verifying_key().as_bytes(),
        admin_device_kp.verifying_key().as_bytes(),
        "the fixture must hold distinct member and device keys"
    );
    assert!(
        msg.envelope.verify_signature(&member_kp.verifying_key()),
        "the revocation must verify under the administrator's member key"
    );
    assert!(
        !msg.envelope.verify_signature(&admin_device_kp.verifying_key()),
        "the revocation must not verify under the administrator's device key"
    );

    // LLR-6dc598: the chain moved before the message was sent, so the anchor
    // the receiver will check against existed when the message arrived.
    assert_eq!(
        s.chain.get(&s.org_id).unwrap().epoch,
        before.epoch + 1,
        "the on-chain root must advance for a revocation"
    );

    let in_memory = s.svc_a.list_orgs()[0].clone();
    assert_eq!(in_memory.trie_members.len(), 1, "A holds itself only after revoking B");
    assert_eq!(in_memory.epoch, before.epoch + 1, "the revocation advanced the epoch");
    assert_eq!(
        in_memory.last_seq,
        before.last_seq + 1,
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
    let published = *s.chain.get(&s.org_id).unwrap().root_hash.as_bytes();
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
        s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, ORG_SECRET),
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
    let published = *s.chain.get(&s.org_id).unwrap().root_hash.as_bytes();
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

// `import_invite` is the joiner's trust root: the `admin_device_key` it stores
// is what LLR-j83kc8 cross-checks the first admission's sender against. An
// invite that reaches no disk is a cross-check that does not survive a
// restart, and deleting this `save` left the gate green.
// verifies: REQ-xa6smf, LLR-9zfnmb
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
        "the admin device key the cross-check needs must be persisted"
    );

    // Re-importing an invite for the same Organisation replaces the stored one
    // rather than appending a second, so the cross-check has one answer.
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
// verifies: LLR-ghja3x
#[tokio::test(flavor = "multi_thread")]
async fn the_admission_envelope_carries_the_mark_one_past_the_records_last() {
    let mut s = setup("envelope-mark").await;
    let before = s.svc_a.list_orgs()[0].clone();

    // A "admits B" into a sink, which hands back the message A transmitted.
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, sink_addr, ORG_SECRET),
    )
    .await
    .expect("admit_member timed out")
    .expect("admit_member failed");
    let (_sink_ep, _sender, msg) = sink.await.unwrap();

    assert_eq!(
        msg.envelope.parent_seq,
        before.last_seq + 1,
        "the envelope must carry the sequence number one greater than the record's last"
    );
    assert_eq!(msg.envelope.org_id, s.org_id);

    // ...and it is signed by the administrator's MEMBER key, not its device key.
    // Both keys are rebuilt from the persona's own persisted seeds, so this
    // compares the envelope against the record rather than against itself.
    let (member_kp, admin_device_kp) = admin_keys(&s.svc_a);
    assert_eq!(
        member_kp.verifying_key().as_bytes(),
        &before.admin_member_key,
        "the fixture's member key must be the one the record names"
    );
    assert_ne!(
        member_kp.verifying_key().as_bytes(),
        admin_device_kp.verifying_key().as_bytes(),
        "the fixture must hold distinct member and device keys for this to mean anything"
    );
    assert!(
        msg.envelope.verify_signature(&member_kp.verifying_key()),
        "the envelope must verify under the administrator's member key"
    );
    assert!(
        !msg.envelope.verify_signature(&admin_device_kp.verifying_key()),
        "the envelope must not verify under the administrator's device key"
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
// verifies: REQ-ztdza4, LLR-jn5jeh
#[tokio::test(flavor = "multi_thread")]
async fn in_loopback_mode_the_joiner_is_dialled_at_the_full_address() {
    let mut s = setup("loopback-dial").await;

    // A peer whose endpoint is listening, named by the full address it reports.
    let (full_addr, sink) = spawn_recv_one(ROGUE_SEED).await;
    let dialled_id = full_addr.id;

    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, full_addr, ORG_SECRET),
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
        epoch_before + 1,
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
    // of the Organisation it names — the key LLR-j83kc8 pins the first
    // admission's sender against.
    let (admin_member, admin_device) = admin_keys(&s.svc_a);
    let invite: org_node::blobs::Invite =
        org_node::blobs::decode(&s.svc_a.export_invite(s.org_id).unwrap()).unwrap();
    assert_eq!(invite.org_id, s.org_id);
    assert_eq!(
        &invite.admin_device_key,
        admin_device.verifying_key().as_bytes(),
        "the invite must name the administrator's device key"
    );
    assert_ne!(
        invite.admin_device_key, invite.admin_member_key,
        "the device key and the member key are two different keys"
    );
    assert_eq!(&invite.admin_member_key, admin_member.verifying_key().as_bytes());

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
    let pid_u = unbound.create_persona(&mut OsRng, "unbound", "Un", "Bound").unwrap();
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
        &jr.member_key,
        SigningKeypair::from_seed(persona_b.member_seed).verifying_key().as_bytes(),
        "the join request must carry the persona's member key"
    );
    assert_eq!(
        &jr.device_key,
        SigningKeypair::from_seed(persona_b.device_seed).verifying_key().as_bytes(),
        "the join request must carry the persona's device key"
    );
    assert_ne!(jr.member_key, jr.device_key, "the two keys must be distinct");
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
    let pid_1 = svc_a.create_persona(&mut OsRng, "first", "First", "Admin").unwrap();
    let pid_2 = svc_a.create_persona(&mut OsRng, "second", "Second", "Admin").unwrap();
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
    let pid_b = svc_b.create_persona(&mut OsRng, "bob", "Bob", "Builder").unwrap();
    let invite_blob = svc_a.export_invite(org_2).unwrap();
    let invite = svc_b.import_invite(&mut OsRng, &invite_blob).unwrap();
    assert_eq!(invite.org_id, org_2, "the invite must name the Organisation it was exported for");
    assert_eq!(
        &invite.admin_member_key, &two_before.admin_member_key,
        "the invite must carry org 2's administrator, not org 1's"
    );
    let jr_b = join_request_of(&svc_b, &pid_b);
    let b_device_kp = device_kp(&svc_b, &pid_b);

    // Admit B into org_2. B verifies the envelope against org_2's on-chain
    // state, so an envelope signed by org_1's administrator is refused.
    let (b_addr, b_task) = spawn_b_receive(svc_b, &b_device_kp).await;
    let new_member = tokio::time::timeout(
        NET,
        svc_a.admit_member(&mut OsRng, org_2, &jr_b, b_addr, ORG_SECRET),
    )
    .await
    .expect("admit_member(org 2) timed out")
    .expect("admit_member(org 2) failed");
    let (svc_b, outcome) = b_task.await.unwrap();
    let outcome = outcome.expect("B must verify an admission signed by org 2's administrator");
    assert_eq!(svc_b.list_orgs()[0].org_id, org_2, "B committed the wrong Organisation");

    // org_2 moved: on chain, in A's record, and in B's.
    let two_after = rec_of(&svc_a, org_2);
    assert_eq!(two_after.epoch, two_before.epoch + 1, "org 2's epoch must advance");
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
        epoch_before + 1,
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
// the node holds; and the first-admission sender cross-check — RC-b6mydy's
// explicit trust root — comparing the authenticated sender against whichever
// pending invite happens to come first.
//
// This fixture is the receiving half of the class: a node that holds two
// Organisations through two Personas, with both invites imported before
// either admission so the cross-check has something to get wrong.
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
    pid_b1: String,
    pid_b2: String,
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
    let pid_a1 = svc_a1.create_persona(&mut OsRng, "admin1", "Admin", "One").unwrap();
    let pid_a2 = svc_a2.create_persona(&mut OsRng, "admin2", "Admin", "Two").unwrap();
    let org_1 = svc_a1.create_organisation(&mut OsRng, &pid_a1).await.unwrap();
    let org_2 = svc_a2.create_organisation(&mut OsRng, &pid_a2).await.unwrap();
    assert_ne!(org_1, org_2);
    let a1_dev = device_kp(&svc_a1, &pid_a1);
    let a2_dev = device_kp(&svc_a2, &pid_a2);
    let mut svc_a1 = svc_a1.with_endpoint(OrgEndpoint::bind(&a1_dev).await.unwrap());
    let mut svc_a2 = svc_a2.with_endpoint(OrgEndpoint::bind(&a2_dev).await.unwrap());

    // B: two Personas, one per Organisation.
    let mut svc_b = OrgService::new(open_store(tag, "b", "pw_b"), Box::new(chain.clone()));
    let pid_b1 = svc_b.create_persona(&mut OsRng, "bob-one", "Bob", "One").unwrap();
    let pid_b2 = svc_b.create_persona(&mut OsRng, "bob-two", "Bob", "Two").unwrap();

    // **Order matters.** org 2's invite is imported FIRST, so a cross-check
    // that takes "whichever invite comes first" rather than the one for this
    // Organisation picks the wrong administrator's device key on the admission
    // that follows.
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
        svc_a1.admit_member(&mut OsRng, org_1, &jr_b1, addr, Some([0x11u8; 32])),
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
        svc_a2.admit_member(&mut OsRng, org_2, &jr_b2, addr, Some([0x22u8; 32])),
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
// verifies: LLR-cja9zv, LLR-q8emds, LLR-j83kc8, LLR-y2v8v2, LLR-9zfnmb, LLR-e5c9ud
#[tokio::test(flavor = "multi_thread")]
async fn a_receiver_holding_two_organisations_commits_into_the_one_the_change_names() {
    let mut s = two_org_receiver("two-org-recv").await;
    let one_before = rec_of(&s.svc_b, s.org_1);
    let two_before = rec_of(&s.svc_b, s.org_2);
    assert_ne!(one_before.org_secret, two_before.org_secret, "distinct secrets");

    // A2 admits C into org 2, and B receives it.
    let pid_c = s.svc_a2.create_persona(&mut OsRng, "carol", "Carol", "Coder").unwrap();
    let jr_c = join_request_of(&s.svc_a2, &pid_c);
    let (addr, task) = spawn_b_receive(s.svc_b, &s.b2_device_kp).await;
    let c_id = tokio::time::timeout(
        NET,
        s.svc_a2.admit_member(&mut OsRng, s.org_2, &jr_c, addr, Some([0x22u8; 32])),
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
    let org2_root = *s.chain.get(&s.org_2).unwrap().root_hash.as_bytes();
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
        s.svc_a2.admit_member(&mut OsRng, s.org_2, &jr_b1, addr, Some([0x22u8; 32])),
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
    let b1_device = s.b1_device_kp.verifying_key().to_bytes();
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

// LLR-u6rq4s's "present in the trie the message verified INTO". The record used
// to call this clause undischargeable, on the ground that the old and new tries
// differ only for a member the same change removes and that removals arrive on
// the other receive path. Review round 5 showed the second half false — a
// removal can arrive through `receive_and_verify` — and wrote this probe: C is
// admitted (B commits it), A revokes C, and **C relays its own removal to B**.
// C is in the trie B holds and not in the trie the change verifies into, so
// checking the sender against the old trie accepts what this test refuses.
// verifies: LLR-u6rq4s
#[tokio::test(flavor = "multi_thread")]
async fn a_removal_relayed_by_the_member_it_removes_is_refused() {
    let mut s = admit_b_directly(setup("relay-by-removed").await).await;

    // C: a persona on A's device whose seed the test holds. The admission is
    // pushed to B, so B's record holds C.
    let pid_c = s.svc_a.create_persona(&mut OsRng, "carol", "Carol", "Coder").unwrap();
    let c_seed = persona_of(&s.svc_a, &pid_c).device_seed;
    let jr_c = join_request_of(&s.svc_a, &pid_c);
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    let c_id = tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, ORG_SECRET),
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
    assert_eq!(
        result.unwrap_err(),
        OrgNodeError::BadSignature,
        "the member this change removes is not in the trie it verifies into"
    );
    let after = svc_b.list_orgs()[0].clone();
    assert_eq!(after.epoch, before.epoch, "a refused relay commits nothing");
    assert_eq!(after.last_seq, before.last_seq);
}

// Three things a first admission writes that no low-level requirement stated
// until review round 5: the administrator key on the member's record, the
// Organisation secret the message carried, and the admitted Persona's member
// id and status. The last two were already evidenced by mutation; the first
// was not — zeroing it left the gate green.
// verifies: LLR-xq9nrq, LLR-ckk5nz, LLR-e5c9ud
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_records_the_signing_key_the_secret_and_the_member() {
    let s = admit_b_directly(setup("first-admission-fields").await).await;
    let published = s.chain.get(&s.org_id).unwrap().org_pub_key;
    let rec = s.svc_b.list_orgs()[0].clone();
    assert_eq!(rec.admin_member_key, published, "the administrator key is the Published signing key");
    assert_eq!(rec.org_pub_key, published);
    assert_eq!(rec.org_secret, ORG_SECRET, "the secret the message carried is stored");

    let b_device = s.b_device_kp.verifying_key().to_bytes();
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
    assert_eq!(reloaded.data().orgs[0].admin_member_key, published);
    assert_eq!(reloaded.data().orgs[0].org_secret, ORG_SECRET);
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
    assert_eq!(s.svc_b.list_orgs()[0].org_secret, ORG_SECRET);

    let jr_c = join_request_for_c(&mut s.svc_a);
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    let c_id = tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, ORG_SECRET),
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
        s.svc_a2.admit_member(&mut OsRng, s.org_2, &jr_b1, addr, Some([0x22u8; 32])),
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
    let pid_c = s.svc_a1.create_persona(&mut OsRng, "carol", "Carol", "Coder").unwrap();
    let jr_c = join_request_of(&s.svc_a1, &pid_c);
    let (addr, task) = spawn_b_self_delete(svc_b, &s.b1_device_kp).await;
    tokio::time::timeout(
        NET,
        s.svc_a1.admit_member(&mut OsRng, s.org_1, &jr_c, addr, Some([0x11u8; 32])),
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
// second Organisation's admissions under the FIRST Persona's device key. The
// joiner's invite names the second Persona's device, and it refuses — after
// the administrator has already moved the chain and its own record. Found by
// review round 5; `a_second_organisation_is_admitted_into_without_touching_the_first`
// hides it by injecting the second Persona's endpoint by hand. No endpoint is
// injected here. Correcting the defect reddens the assertions marked below.
#[tokio::test(flavor = "multi_thread")]
async fn pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key() {
    let chain = MockChainOps::new();
    let mut svc_a = OrgService::new(open_store("pr-8qsnhx", "a", "pw_a"), Box::new(chain.clone()));
    let pid_1 = svc_a.create_persona(&mut OsRng, "first", "First", "Admin").unwrap();
    let pid_2 = svc_a.create_persona(&mut OsRng, "second", "Second", "Admin").unwrap();
    let org_1 = svc_a.create_organisation(&mut OsRng, &pid_1).await.unwrap();
    let org_2 = svc_a.create_organisation(&mut OsRng, &pid_2).await.unwrap();

    let mut svc_b1 = OrgService::new(open_store("pr-8qsnhx", "b1", "pw_b1"), Box::new(chain.clone()));
    let mut svc_b2 = OrgService::new(open_store("pr-8qsnhx", "b2", "pw_b2"), Box::new(chain.clone()));
    let pid_b1 = svc_b1.create_persona(&mut OsRng, "bob", "Bob", "One").unwrap();
    let pid_b2 = svc_b2.create_persona(&mut OsRng, "bea", "Bea", "Two").unwrap();
    svc_b1.import_invite(&mut OsRng, &svc_a.export_invite(org_1).unwrap()).unwrap();
    svc_b2.import_invite(&mut OsRng, &svc_a.export_invite(org_2).unwrap()).unwrap();
    let jr_b1 = join_request_of(&svc_b1, &pid_b1);
    let jr_b2 = join_request_of(&svc_b2, &pid_b2);
    let b1_dev = device_kp(&svc_b1, &pid_b1);
    let b2_dev = device_kp(&svc_b2, &pid_b2);

    // The first Organisation's admission binds A's endpoint from Persona 1.
    let (addr, task) = spawn_b_receive(svc_b1, &b1_dev).await;
    tokio::time::timeout(NET, svc_a.admit_member(&mut OsRng, org_1, &jr_b1, addr, ORG_SECRET))
        .await
        .unwrap()
        .unwrap();
    let (_svc_b1, out1) = task.await.unwrap();
    out1.expect("the first Organisation's admission is accepted");

    // The second goes out under Persona 1's device key.
    let epoch_before = chain.get(&org_2).unwrap().epoch;
    let (addr, task) = spawn_b_receive(svc_b2, &b2_dev).await;
    tokio::time::timeout(NET, svc_a.admit_member(&mut OsRng, org_2, &jr_b2, addr, ORG_SECRET))
        .await
        .unwrap()
        .expect("PR-8qsnhx: A reports success");
    let (svc_b2, out2) = task.await.unwrap();
    assert_eq!(
        out2.unwrap_err(),
        OrgNodeError::BadSignature,
        "PR-8qsnhx: the joiner refuses a sender its invite does not name"
    );
    assert_eq!(chain.get(&org_2).unwrap().epoch, epoch_before + 1, "PR-8qsnhx: the chain moved anyway");
    assert!(svc_b2.list_orgs().is_empty(), "the joiner holds nothing");
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
    assert_eq!(result.expect("PR-322qst: committed as an update").epoch, 3);
    let b_dev = s.b_device_kp.verifying_key().to_bytes();
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
    tokio::time::timeout(NET, s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, ORG_SECRET))
        .await
        .unwrap()
        .unwrap();
    let (svc_b, outcome) = b_task.await.unwrap();
    let b_dev = s.b_device_kp.verifying_key().to_bytes();
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
    let p1 = svc.create_persona(&mut OsRng, "one", "Persona", "One").unwrap();
    let p2 = svc.create_persona(&mut OsRng, "two", "Persona", "Two").unwrap();
    let p1_dev = device_kp(&svc, &p1);
    let p2_dev = device_kp(&svc, &p2);
    let svc = svc.with_endpoint(OrgEndpoint::bind(&p1_dev).await.unwrap());

    let jr = join_request_of(&svc, &p2);
    assert_eq!(jr.device_key, p2_dev.verifying_key().to_bytes(), "the blob names p2's device");
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
    let p1 = svc.create_persona(&mut OsRng, "one", "Persona", "One").unwrap();
    let p2 = svc.create_persona(&mut OsRng, "two", "Persona", "Two").unwrap();
    let org = svc.create_organisation(&mut OsRng, &p2).await.unwrap();
    let p1_dev = device_kp(&svc, &p1);
    let p2_dev = device_kp(&svc, &p2);
    let svc = svc.with_endpoint(OrgEndpoint::bind(&p1_dev).await.unwrap());

    let invite: org_node::blobs::Invite = org_node::blobs::decode(&svc.export_invite(org).unwrap()).unwrap();
    assert_eq!(invite.admin_device_key, p2_dev.verifying_key().to_bytes(), "the Invite names p2, the administrator");
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

// PR-u4c2vp — PINS A DEFECT, open since 2026-09-09. On the self-delete path's
// `UpdatedNotRevoked` branch the authenticated sender is not cross-checked,
// so a change relayed by a device the record does not name is committed.
// LLR-3q63zv states that this path does not cross-check; its reason holds for
// the branch where the node is removed and not for this one. A rogue relay R
// forwards the administrator's genuine admission of C to B. Correcting the
// defect reddens the assertion below. Found by review round 7.
// verifies: LLR-3q63zv
#[tokio::test(flavor = "multi_thread")]
async fn pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path() {
    let mut s = admit_b_directly(setup("pr-u4c2vp").await).await;
    let epoch_before = s.svc_b.list_orgs()[0].epoch;

    // A admits C, but the push goes to R.
    let jr_c = join_request_for_c(&mut s.svc_a);
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(NET, s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, r_addr, ORG_SECRET))
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
            outcome.expect("PR-u4c2vp: a stranger's relay is committed"),
            SelfDeleteOutcome::UpdatedNotRevoked { .. }
        ),
        "PR-u4c2vp: committed as an ordinary update"
    );
    assert!(svc_b.list_orgs()[0].epoch > epoch_before, "PR-u4c2vp: and B's record moved");
}

// A first admission with NO imported invite is accepted on the chain anchor
// and the signature alone: the invite cross-check (LLR-j83kc8) applies only
// when an invite exists. Stated by no requirement until review round 7, which
// measured that refusing it left the gate green. The hazard analysis
// discusses the trade-off (`org-node/docs/risk/2026-09-09-org-node-hazards.md`,
// RC-b6mydy); this test pins what the code does.
// verifies: LLR-mbjfq8
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_with_no_imported_invite_rests_on_the_chain_alone() {
    let chain = MockChainOps::new();
    let mut svc_a = OrgService::new(open_store("no-invite", "a", "pw_a"), Box::new(chain.clone()));
    let mut svc_b0 = OrgService::new(open_store("no-invite", "b", "pw_b"), Box::new(chain.clone()));
    let pid_a = svc_a.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();
    let org_id = svc_a.create_organisation(&mut OsRng, &pid_a).await.unwrap();
    let a_dev = device_kp(&svc_a, &pid_a);
    let mut svc_a = svc_a.with_endpoint(OrgEndpoint::bind(&a_dev).await.unwrap());
    let pid_b = svc_b0.create_persona(&mut OsRng, "bob", "Bob", "Builder").unwrap();
    assert!(svc_b0.list_pending_invites().is_empty(), "B imports no invite");
    let jr = join_request_of(&svc_b0, &pid_b);
    let b_dev = device_kp(&svc_b0, &pid_b);

    let (addr, task) = spawn_b_receive(svc_b0, &b_dev).await;
    tokio::time::timeout(NET, svc_a.admit_member(&mut OsRng, org_id, &jr, addr, ORG_SECRET))
        .await
        .unwrap()
        .unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert_eq!(outcome.expect("accepted with no invite to check against").org_id, org_id);
    assert_eq!(svc_b.list_orgs().len(), 1);
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
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, addr, ORG_SECRET),
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
// point a consumption could sit. This one passes the invite cross-check —
// it comes from the administrator's own device key — and then fails
// verification, because the envelope's mark was altered after signing.
// verifies: LLR-q8emds
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_that_fails_verification_leaves_the_invite_pending() {
    let mut s = setup("invite-kept").await;
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    tokio::time::timeout(
        NET,
        s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, r_addr, ORG_SECRET),
    )
    .await
    .unwrap()
    .unwrap();
    let (_r_ep, _sender, mut msg) = r_task.await.unwrap();
    msg.envelope.parent_seq += 1;

    let pid_a = s.svc_a.list_personas()[0].persona_id.clone();
    let a_again = OrgEndpoint::bind(&device_kp(&s.svc_a, &pid_a)).await.unwrap();
    assert_eq!(s.svc_b.list_pending_invites().len(), 1, "B holds A's invite");
    let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, a_again.send(b_addr, &msg)).await.unwrap().unwrap();

    let (svc_b, result) = b_task.await.unwrap();
    assert!(result.is_err(), "an altered envelope must not verify, got {result:?}");
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
    seen: std::sync::Arc<std::sync::Mutex<Vec<Option<[u8; 32]>>>>,
}

const PROXY: [u8; 32] = [0x5au8; 32];

#[async_trait::async_trait]
impl ChainOps for ProxyChain {
    async fn submit_genesis(
        &self,
        genesis_root: [u8; 32],
        org_pub_key: [u8; 32],
    ) -> Result<(OrgId, Option<[u8; 32]>), OrgNodeError> {
        let (org_id, none) = self.inner.submit_genesis(genesis_root, org_pub_key).await?;
        assert_eq!(none, None, "the mock returns no proxy of its own");
        Ok((org_id, Some(PROXY)))
    }

    async fn submit_update(
        &self,
        org_id: OrgId,
        new_root: [u8; 32],
        org_pub_key: [u8; 32],
        expected_epoch: u64,
        proxy_account: Option<[u8; 32]>,
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
    let pid_a = svc_a.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();
    let org_id = svc_a.create_organisation(&mut OsRng, &pid_a).await.unwrap();
    assert_eq!(svc_a.list_orgs()[0].proxy_account, Some(PROXY), "LLR-dzte8x: kept in the record");
    assert_eq!(
        reopen_store("proxy", "a", "pw_a").data().orgs[0].proxy_account,
        Some(PROXY),
        "LLR-dzte8x: and on disk"
    );

    let jr = join_request_for_c(&mut svc_a);
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    tokio::time::timeout(NET, svc_a.admit_member(&mut OsRng, org_id, &jr, sink_addr, ORG_SECRET))
        .await
        .unwrap()
        .unwrap();
    let _ = sink.await.unwrap();
    assert_eq!(*chain.seen.lock().unwrap(), vec![Some(PROXY)], "LLR-3v5nu9: admission passes it");

    let c_id = id_by_handle(&svc_a.list_orgs()[0], "carol");
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    tokio::time::timeout(NET, svc_a.revoke_member(&mut OsRng, org_id, c_id, Some(sink_addr)))
        .await
        .unwrap()
        .unwrap();
    let _ = sink.await.unwrap();
    assert_eq!(
        *chain.seen.lock().unwrap(),
        vec![Some(PROXY), Some(PROXY)],
        "LLR-drgdy8: revocation passes it"
    );
}
