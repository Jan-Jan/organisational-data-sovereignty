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
use org_node::service::{MockChainOps, OrgService};
use org_node::store::{PersonaStatus, PersonaStore};
use org_node::transport::endpoint::OrgEndpoint;
use org_node::transport::wire::WireMessage;
use rand::rngs::OsRng;

const NET: Duration = Duration::from_secs(30);
const ORG_SECRET: Option<[u8; 32]> = Some([0xffu8; 32]);
/// The rogue relay's device key — a third device, neither A's nor B's.
const ROGUE_SEED: [u8; 32] = [0x33u8; 32];

/// Fresh encrypted store under `temp_dir()`, unique per test and party.
fn open_store(tag: &str, party: &str, password: &str) -> PersonaStore {
    let dir = std::env::temp_dir().join(format!(
        "ods-admission-sender-{tag}-{party}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    PersonaStore::open(dir.join("store.bin"), password).unwrap()
}

/// The device keypair of a persona, from its persisted `device_seed`.
fn device_kp(svc: &OrgService, persona_id: &str) -> SigningKeypair {
    let seed = svc
        .list_personas()
        .iter()
        .find(|p| p.persona_id == persona_id)
        .map(|p| p.device_seed)
        .expect("persona not found");
    SigningKeypair::from_seed(seed)
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
    let jr_blob = svc_b.export_join_request(&pid_b).unwrap();
    let join_request_b = OrgService::import_join_request(&jr_blob).unwrap();
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
    let jr_blob = svc_a.export_join_request(&pid_c).unwrap();
    let jr = OrgService::import_join_request(&jr_blob).unwrap();
    assert_eq!(jr.handle, "carol");
    jr
}

// Abnormal case of the first-admission invite cross-check: the envelope is
// chain-valid and correctly signed by the admin, and the org_secret and genesis
// snapshot are genuine, but it arrives from a device that is not the invite's
// `admin_device_key`. B must reject it and commit nothing.
// verifies: REQ-xa6smf
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
    let persona_b = svc_b.list_personas().iter().find(|p| p.persona_id == s.pid_b).unwrap();
    assert_ne!(persona_b.status, PersonaStatus::Active, "B's persona must not be Active");
    assert_eq!(persona_b.org_id, None);
}

// Normal case of the post-admission sender check: after B has committed epoch 2,
// a further update (C's admission) pushed by the admin's own device — a member
// device in the new trie — is verified and committed.
// verifies: REQ-ztdza4
#[tokio::test(flavor = "multi_thread")]
async fn update_from_the_admin_after_admission_is_committed() {
    let mut s = admit_b_directly(setup("update-admin").await).await;

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
// verifies: REQ-ztdza4, REQ-mr5abb
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

// Normal case of REQ-d9g6nt: the founding admin's id and an admitted
// member's id are not their keys, and differ from each other.
// verifies: REQ-d9g6nt
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
// verifies: REQ-d9g6nt
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
// verifies: REQ-d9g6nt
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
// verifies: REQ-d9g6nt
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
    let persona_b = svc_b.list_personas().iter().find(|p| p.persona_id == s.pid_b).unwrap();
    assert_ne!(persona_b.status, PersonaStatus::Active, "B's persona must not be Active");
    assert_eq!(persona_b.org_id, None);
}
