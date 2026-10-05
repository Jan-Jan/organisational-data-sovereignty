//! Integration test: the five user stories via OrgService against MockChainOps
//! + loopback iroh endpoints.  Offline — no live chain, no relay.
//!
//! Stories exercised:
//!   1. A creates a persona and an organisation → epoch 1 in MockChain.
//!   2. B creates a persona; exports a JoinRequest; A imports it.
//!      A exports an Invite; B imports it (persists admin_device_key for cross-check).
//!   3. A admits B (trie add → epoch 2 in MockChain; envelope pushed to B over iroh).
//!   4. B receives and verifies the envelope → B's persona Active, OrgRecord stored.
//!      Cross-check: B asserts sender's QUIC device key == invite's admin_device_key.
//!   5. A revokes B (trie remove → epoch 3); B self-deletes its OrgRecord.
//!
//! The gate: `cargo test -p org-node --features app --test service_stories`

#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use org_node::keys::SigningKeypair;
use org_node::service::{MockChainOps, OrgService, SelfDeleteOutcome};
use org_node::store::{PersonaStatus, PersonaStore};
use org_node::transport::endpoint::OrgEndpoint;

// The proper end-to-end test that runs all 5 stories in one function.
// `five_stories_headless` (stories 1-4 only, no invite cross-check) was
// deleted in the Fix 3 cleanup — `five_stories_full_e2e` is the canonical gate.
//
// Normal cases of three org-node requirements, over the service API: story 4
// commits the applied Change set with the chain's epoch and root (REQ-nhe2zu)
// and passes the first-admission invite cross-check with the genuine admin as
// sender (REQ-xa6smf); story 5 is the self-delete on the device's own removal
// (REQ-uxv2x2). Further abnormal cases are in verify_against_chain.rs and
// admission_sender.rs; REQ-uxv2x2's own abnormal-input case — a revocation
// whose envelope fails verification — is the last test in this file.
// verifies: REQ-nhe2zu, REQ-xa6smf, REQ-uxv2x2, LLR-rb8r65, LLR-ghja3x, LLR-bg3vsw, LLR-t4znbk, LLR-37cj3n, LLR-q8emds, LLR-68yd3j, LLR-6zjzn2, LLR-cns6q6, LLR-6p4pj2
#[tokio::test(flavor = "multi_thread")]
async fn five_stories_full_e2e() {
    use rand::rngs::OsRng;

    // ---- Shared chain ----
    let chain = MockChainOps::new();
    let chain_a = chain.clone();
    let chain_b_admit = chain.clone();   // for story 3/4
    let chain_b_revoke = chain.clone();  // for story 5

    // B's device keypair (fixed seed so we can reconstruct it for story 5).
    let b_device_kp = SigningKeypair::from_seed([0x22u8; 32]);

    // ---- Bind B's admission endpoint ----
    let ep_b_admit = OrgEndpoint::bind(&b_device_kp).await.unwrap();
    let b_addr_admit = ep_b_admit.inner().addr();

    // ---- Stores ----
    let store_path = |party: &str| {
        let dir = std::env::temp_dir().join(format!("ods-e2e-{party}-{}", std::process::id()));
        // Cleared first, as the other helpers in this file do: a reused pid
        // would open a store an earlier run left (review round 6).
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("store.bin")
    };
    let store_a_path = store_path("a");
    let store_b_path = store_path("b");

    let store_a = PersonaStore::open(store_a_path.clone(), "pw_a").unwrap();
    let store_b = PersonaStore::open(store_b_path.clone(), "pw_b").unwrap();

    // svc_a starts without an endpoint; we bind it below once we know A's device seed.
    let mut svc_a = OrgService::new(store_a, Box::new(chain_a));
    let mut svc_b = OrgService::new(store_b, Box::new(chain_b_admit));

    // ---- Story 1: A creates persona + org ----
    let pid_a = svc_a.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();
    let org_id = svc_a.create_organisation(&mut OsRng, &pid_a).await.unwrap();

    assert_eq!(chain.get(&org_id).unwrap().epoch, 1);
    assert_eq!(svc_a.list_personas()[0].status, PersonaStatus::Active);

    // Bind A's endpoint from the SAME device seed that `create_persona` generated.
    // This ensures the QUIC-authenticated sender identity on B's side equals the
    // `admin_device_key` that `export_invite` will encode — the cross-check gate.
    let a_device_seed = svc_a.list_personas()
        .iter()
        .find(|p| p.persona_id == pid_a)
        .map(|p| p.device_seed)
        .expect("persona not found after create");
    let a_device_kp = SigningKeypair::from_seed(a_device_seed);
    let ep_a_admit = OrgEndpoint::bind(&a_device_kp).await.unwrap();
    let svc_a = svc_a.with_endpoint(ep_a_admit);
    let mut svc_a = svc_a;

    // ---- Story 2: B creates persona; A exports Invite; B imports it ----
    let pid_b = svc_b.create_persona(&mut OsRng, "bob", "Bob", "Builder").unwrap();

    // A exports the invite (admin_device_key = A's persona device key).
    let invite_blob = svc_a.export_invite(org_id).unwrap();
    // B imports and persists the invite — stores admin_device_key for the cross-check.
    let invite = svc_b.import_invite(&mut OsRng, &invite_blob).unwrap();
    assert_eq!(invite.org_id, org_id);

    // B exports a JoinRequest (includes B's iroh addr so A can dial back if needed).
    let ep_b_for_jr = OrgEndpoint::bind(&b_device_kp).await.unwrap();
    let b_addr_jr = ep_b_for_jr.inner().addr();
    let svc_b = svc_b.with_endpoint(ep_b_for_jr);

    let jr_blob = svc_b.export_join_request(&pid_b).unwrap();
    let join_request = OrgService::import_join_request(&jr_blob).unwrap();
    assert_eq!(join_request.handle, "bob");

    // Rebuild svc_b with ep_b_admit so B can receive A's push.
    let store_b2 = PersonaStore::open(store_b_path.clone(), "pw_b").unwrap();
    let mut svc_b2 = OrgService::new(store_b2, Box::new(chain_b_revoke.clone()))
        .with_endpoint(ep_b_admit);

    // ---- Story 3+4: B spawns recv, A admits ----
    // B receives first — spawn BEFORE A dials.
    let recv_handle = tokio::spawn(async move {
        let outcome = tokio::time::timeout(
            Duration::from_secs(30),
            svc_b2.receive_and_verify(&mut OsRng),
        )
        .await
        .expect("B receive_and_verify timed out")
        .expect("B receive_and_verify failed");
        (svc_b2, outcome)
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let b_member_id = tokio::time::timeout(
        Duration::from_secs(30),
        svc_a.admit_member(&mut OsRng, org_id, &join_request, b_addr_admit, Some([0xffu8; 32])),
    )
    .await
    .expect("admit_member timed out")
    .expect("admit_member failed");

    // MockChain: epoch 2.
    assert_eq!(chain.get(&org_id).unwrap().epoch, 2, "admit must bump to epoch 2");

    // Collect B's result + svc_b (moved out of the task).
    let (svc_b3, b_outcome) = recv_handle.await.unwrap();

    assert_eq!(b_outcome.org_id, org_id, "B's outcome org_id must match");
    assert_eq!(b_outcome.epoch, 2, "B must commit epoch 2");
    assert_eq!(
        b_outcome.root,
        *chain.get(&org_id).unwrap().root_hash.as_bytes(),
        "B's committed root must match on-chain root"
    );

    // B's persona must be Active.
    assert_eq!(
        svc_b3.list_personas().iter().find(|p| p.persona_id == pid_b).unwrap().status,
        PersonaStatus::Active,
        "B's persona must be Active after successful verification"
    );
    // B must have the OrgRecord.
    assert_eq!(svc_b3.list_orgs().len(), 1, "B must have exactly 1 OrgRecord");
    assert_eq!(svc_b3.list_orgs()[0].epoch, 2);
    assert_eq!(svc_b3.list_orgs()[0].org_secret, Some([0xffu8; 32]));

    // ---- Story 5: A revokes B; B self-deletes ----

    // Bind a fresh B endpoint for receiving the revocation.
    let b_device_kp2 = SigningKeypair::from_seed([0x22u8; 32]);
    let ep_b_revoke = OrgEndpoint::bind(&b_device_kp2).await.unwrap();
    let b_addr_revoke = ep_b_revoke.inner().addr();

    // Replace B's endpoint.
    let mut svc_b3 = svc_b3.with_endpoint(ep_b_revoke);

    // B waits for the revocation message.
    let revoke_recv_handle = tokio::spawn(async move {
        let outcome = tokio::time::timeout(
            Duration::from_secs(30),
            svc_b3.receive_and_self_delete_if_revoked(&mut OsRng),
        )
        .await
        .expect("B revoke recv timed out")
        .expect("B receive_and_self_delete_if_revoked failed");
        (svc_b3, outcome)
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Rebind A's outbound endpoint for the revocation send.
    let ep_a_revoke = OrgEndpoint::bind(&a_device_kp).await.unwrap();
    let mut svc_a = svc_a.with_endpoint(ep_a_revoke);

    tokio::time::timeout(
        Duration::from_secs(30),
        svc_a.revoke_member(&mut OsRng, org_id, b_member_id, Some(b_addr_revoke)),
    )
    .await
    .expect("revoke_member timed out")
    .expect("revoke_member failed");

    // MockChain: epoch 3.
    assert_eq!(chain.get(&org_id).unwrap().epoch, 3, "revoke must bump to epoch 3");

    // Collect B's self-delete result.
    let (svc_b_final, delete_outcome) = revoke_recv_handle.await.unwrap();

    match delete_outcome {
        SelfDeleteOutcome::SelfDeleted { org_id: oid } => {
            assert_eq!(oid, org_id, "self-deleted org_id must match");
        }
        SelfDeleteOutcome::UpdatedNotRevoked { .. } => {
            panic!("expected SelfDeleted but got UpdatedNotRevoked");
        }
    }

    // B must no longer have the OrgRecord.
    assert_eq!(
        svc_b_final.list_orgs().len(),
        0,
        "B must have no OrgRecords after self-delete"
    );

    // B's persona must be Revoked.
    assert_eq!(
        svc_b_final.list_personas().iter().find(|p| p.persona_id == pid_b).unwrap().status,
        PersonaStatus::Revoked,
        "B's persona must be Revoked after self-delete"
    );

    // A still has the org at epoch 3 with only the admin in the trie.
    assert_eq!(svc_a.list_orgs().len(), 1);
    assert_eq!(svc_a.list_orgs()[0].epoch, 3);
    assert_eq!(svc_a.list_orgs()[0].trie_members.len(), 1, "only admin should remain");

    let _ = (pid_b, b_addr_jr, chain_b_revoke); // suppress unused warnings
}

// Abnormal case of the self-delete rule: a revocation Change set that removes a
// *different* member must be verified, committed as an ordinary update, and must
// NOT trigger a self-delete. Story 5 above is the normal case (the device's own
// removal); this is the other side of the same requirement.
// verifies: REQ-uxv2x2, LLR-jsx922
#[tokio::test(flavor = "multi_thread")]
async fn revocation_of_another_member_is_committed_not_self_deleted() {
    use rand::rngs::OsRng;

    const NET: Duration = Duration::from_secs(30);
    const ORG_SECRET: Option<[u8; 32]> = Some([0xffu8; 32]);

    /// Fresh encrypted store under `temp_dir()`, unique per party and process.
    fn store_path(party: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("ods-revoke-other-{party}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("store.bin")
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

    // ---- Shared chain and two services ----
    let chain = MockChainOps::new();
    let mut svc_a = OrgService::new(
        PersonaStore::open(store_path("a"), "pw_a").unwrap(),
        Box::new(chain.clone()),
    );
    let mut svc_b = OrgService::new(
        PersonaStore::open(store_path("b"), "pw_b").unwrap(),
        Box::new(chain.clone()),
    );

    // ---- Stories 1-2: A creates persona + org; B imports A's invite ----
    let pid_a = svc_a.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();
    let org_id = svc_a.create_organisation(&mut OsRng, &pid_a).await.unwrap();
    assert_eq!(chain.get(&org_id).unwrap().epoch, 1);

    // A's endpoint is bound from A's persona device seed, so the authenticated
    // QUIC sender on B's side equals the invite's `admin_device_key`.
    let a_device_kp = device_kp(&svc_a, &pid_a);
    let mut svc_a = svc_a.with_endpoint(OrgEndpoint::bind(&a_device_kp).await.unwrap());

    let pid_b = svc_b.create_persona(&mut OsRng, "bob", "Bob", "Builder").unwrap();
    let invite_blob = svc_a.export_invite(org_id).unwrap();
    let invite = svc_b.import_invite(&mut OsRng, &invite_blob).unwrap();
    assert_eq!(invite.org_id, org_id);

    let jr_b_blob = svc_b.export_join_request(&pid_b).unwrap();
    let jr_b = OrgService::import_join_request(&jr_b_blob).unwrap();
    let b_device_kp = device_kp(&svc_b, &pid_b);

    // ---- Stories 3-4: A admits B; B receives and commits epoch 2 ----
    let ep_b = OrgEndpoint::bind(&b_device_kp).await.unwrap();
    let b_addr = ep_b.inner().addr();
    let mut svc_b_admit = svc_b.with_endpoint(ep_b);
    let b_task = tokio::spawn(async move {
        let r = tokio::time::timeout(NET, svc_b_admit.receive_and_verify(&mut OsRng))
            .await
            .expect("B receive_and_verify(B admission) timed out");
        (svc_b_admit, r)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    tokio::time::timeout(
        NET,
        svc_a.admit_member(&mut OsRng, org_id, &jr_b, b_addr, ORG_SECRET),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, 2, "admitting B must bump to epoch 2");

    let (svc_b, r) = b_task.await.unwrap();
    let outcome = r.expect("B's direct admission from A must verify");
    assert_eq!(outcome.epoch, 2);
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 2, "admin + B");

    // ---- A creates persona C and admits it, pushing the update to B ----
    // C lives in A's store; `admin_persona_for_org` matches by member key, so A
    // stays the org's admin.
    let pid_c = svc_a.create_persona(&mut OsRng, "carol", "Carol", "Coder").unwrap();
    let jr_c_blob = svc_a.export_join_request(&pid_c).unwrap();
    let jr_c = OrgService::import_join_request(&jr_c_blob).unwrap();
    assert_eq!(jr_c.handle, "carol");

    let ep_b = OrgEndpoint::bind(&b_device_kp).await.unwrap();
    let b_addr = ep_b.inner().addr();
    let mut svc_b_update = svc_b.with_endpoint(ep_b);
    let b_task = tokio::spawn(async move {
        let r = tokio::time::timeout(NET, svc_b_update.receive_and_verify(&mut OsRng))
            .await
            .expect("B receive_and_verify(C admission) timed out");
        (svc_b_update, r)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let c_member_id = tokio::time::timeout(
        NET,
        svc_a.admit_member(&mut OsRng, org_id, &jr_c, b_addr, ORG_SECRET),
    )
    .await
    .expect("admit_member(C) timed out")
    .expect("admit_member(C) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, 3, "admitting C must bump to epoch 3");

    let (svc_b, r) = b_task.await.unwrap();
    let outcome = r.expect("B must accept C's admission pushed by the admin's own device");
    assert_eq!(outcome.epoch, 3, "B must commit epoch 3");
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 3, "admin + B + C");

    // ---- A revokes C; B receives the revocation ----
    let ep_b = OrgEndpoint::bind(&b_device_kp).await.unwrap();
    let b_addr = ep_b.inner().addr();
    let mut svc_b_revoke = svc_b.with_endpoint(ep_b);
    let b_task = tokio::spawn(async move {
        let r = tokio::time::timeout(
            NET,
            svc_b_revoke.receive_and_self_delete_if_revoked(&mut OsRng),
        )
        .await
        .expect("B receive_and_self_delete_if_revoked timed out");
        (svc_b_revoke, r)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    tokio::time::timeout(
        NET,
        svc_a.revoke_member(&mut OsRng, org_id, c_member_id, Some(b_addr)),
    )
    .await
    .expect("revoke_member(C) timed out")
    .expect("revoke_member(C) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, 4, "revoking C must bump to epoch 4");

    let (svc_b_final, r) = b_task.await.unwrap();
    let delete_outcome = r.expect("B must verify and commit C's revocation");

    // The requirement: someone else's removal is an ordinary update, never a
    // self-delete.
    match delete_outcome {
        SelfDeleteOutcome::UpdatedNotRevoked { org_id: oid } => {
            assert_eq!(oid, org_id, "the updated org_id must match");
        }
        SelfDeleteOutcome::SelfDeleted { .. } => {
            panic!("B must not self-delete when a different member is revoked");
        }
    }

    // B's OrgRecord must still be there, advanced to epoch 4 without C.
    assert_eq!(
        svc_b_final.list_orgs().len(),
        1,
        "B's OrgRecord must still be present after another member's revocation"
    );
    let rec_b = &svc_b_final.list_orgs()[0];
    assert_eq!(rec_b.org_id, org_id);
    assert_eq!(rec_b.epoch, 4, "B must commit epoch 4");
    assert_eq!(
        rec_b.root_hash,
        *chain.get(&org_id).unwrap().root_hash.as_bytes(),
        "B's committed root must match the on-chain root"
    );
    assert_eq!(rec_b.trie_members.len(), 2, "admin + B only");
    assert!(
        !rec_b.trie_members.iter().any(|m| m.id == c_member_id),
        "C must be gone from B's trie snapshot"
    );

    // And B's own persona must still be Active.
    assert_eq!(
        svc_b_final
            .list_personas()
            .iter()
            .find(|p| p.persona_id == pid_b)
            .unwrap()
            .status,
        PersonaStatus::Active,
        "B's persona must stay Active when someone else is revoked"
    );
}

// Abnormal-input case of the self-delete rule: a revocation Change set that
// removes this node's own Device key, but whose Envelope is signed by a key
// that is NOT the Organisation's published signing key, must be rejected — and
// the rejection must leave the OrgRecord exactly as it was. The node must never
// delete its record of the Organisation on a message it refused to verify.
// verifies: REQ-uxv2x2, LLR-6qmq2g, LLR-vw2jn6
#[tokio::test(flavor = "multi_thread")]
async fn unverified_revocation_leaves_the_record_in_place() {
    use rand::rngs::OsRng;

    use org_members::hasher::Blake3Hasher;
    use org_members::trie::OrgTrie;
    use org_members::{Handle, MemberId, MemberLeaf, Name, Surname};
    use org_node::envelope::SignedDeltaEnvelope;
    use org_node::error::OrgNodeError;
    use org_node::store::MemberSnapshot;
    use org_node::transport::wire::WireMessage;

    type Trie = OrgTrie<Blake3Hasher>;

    const NET: Duration = Duration::from_secs(30);
    const ORG_SECRET: Option<[u8; 32]> = Some([0xffu8; 32]);

    /// Fresh encrypted store under `temp_dir()`, unique per party and process.
    fn store_path(party: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ods-bad-sig-{party}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("store.bin")
    }

    /// The member and device keypairs of a persona, from its persisted seeds.
    fn keys_of(svc: &OrgService, persona_id: &str) -> (SigningKeypair, SigningKeypair) {
        let p = svc
            .list_personas()
            .iter()
            .find(|p| p.persona_id == persona_id)
            .expect("persona not found");
        (
            SigningKeypair::from_seed(p.member_seed),
            SigningKeypair::from_seed(p.device_seed),
        )
    }

    // ---- Two services over one shared mock chain ----
    let chain = MockChainOps::new();
    let mut svc_a = OrgService::new(
        PersonaStore::open(store_path("a"), "pw_a").unwrap(),
        Box::new(chain.clone()),
    );
    let mut svc_b = OrgService::new(
        PersonaStore::open(store_path("b"), "pw_b").unwrap(),
        Box::new(chain.clone()),
    );

    // ---- A creates persona + org; B imports A's invite ----
    let pid_a = svc_a.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();
    let org_id = svc_a.create_organisation(&mut OsRng, &pid_a).await.unwrap();
    assert_eq!(chain.get(&org_id).unwrap().epoch, 1);

    let (a_member_kp, a_device_kp) = keys_of(&svc_a, &pid_a);
    let mut svc_a = svc_a.with_endpoint(OrgEndpoint::bind(&a_device_kp).await.unwrap());

    let pid_b = svc_b.create_persona(&mut OsRng, "bob", "Bob", "Builder").unwrap();
    let invite_blob = svc_a.export_invite(org_id).unwrap();
    svc_b.import_invite(&mut OsRng, &invite_blob).unwrap();

    let jr_b_blob = svc_b.export_join_request(&pid_b).unwrap();
    let jr_b = OrgService::import_join_request(&jr_b_blob).unwrap();
    let (b_member_kp, b_device_kp) = keys_of(&svc_b, &pid_b);

    // ---- A admits B; B receives and commits epoch 2 ----
    let ep_b = OrgEndpoint::bind(&b_device_kp).await.unwrap();
    let b_addr = ep_b.inner().addr();
    let mut svc_b_admit = svc_b.with_endpoint(ep_b);
    let b_task = tokio::spawn(async move {
        let r = tokio::time::timeout(NET, svc_b_admit.receive_and_verify(&mut OsRng))
            .await
            .expect("B receive_and_verify(B admission) timed out");
        (svc_b_admit, r)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let b_member_id = tokio::time::timeout(
        NET,
        svc_a.admit_member(&mut OsRng, org_id, &jr_b, b_addr, ORG_SECRET),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, 2, "admitting B must bump to epoch 2");

    let (svc_b, r) = b_task.await.unwrap();
    r.expect("B's direct admission from A must verify");

    // ---- Forge a revocation of B, signed by a key the chain does not name ----
    // Rebuild B's committed trie from B's own snapshot. An integration test
    // cannot call the crate's private snapshot-to-trie helper, so the leaves
    // are rebuilt from the two personas' seeds and matched to the snapshot by
    // member key; the root assertion below proves the reconstruction faithful.
    let (epoch_before, members_before, root_before, seq_before, org_pub_key) = {
        let rec = &svc_b.list_orgs()[0];
        (rec.epoch, rec.trie_members.len(), rec.root_hash, rec.last_seq, rec.org_pub_key)
    };

    let leaf_of = |s: &MemberSnapshot| -> MemberLeaf {
        let (member, device) = if s.member_key == *a_member_kp.member_key().as_bytes() {
            (&a_member_kp, &a_device_kp)
        } else if s.member_key == *b_member_kp.member_key().as_bytes() {
            (&b_member_kp, &b_device_kp)
        } else {
            panic!("snapshot member key belongs to neither A nor B");
        };
        MemberLeaf::new(
            MemberId::new(s.id),
            Handle::parse(&s.handle).unwrap(),
            member.member_key(),
            Name::parse(&s.name).unwrap(),
            Surname::parse(&s.surname).unwrap(),
            vec![device.device_key()],
        )
        .unwrap()
    };
    let leaves: Vec<MemberLeaf> =
        svc_b.list_orgs()[0].trie_members.iter().map(leaf_of).collect();
    let b_trie = Trie::genesis(leaves).unwrap();
    assert_eq!(
        b_trie.root_hash().unwrap().as_bytes(),
        &root_before,
        "the rebuilt trie must reproduce B's committed root"
    );

    // The Change set really does remove B's own Device key: were verification
    // skipped or its error ignored, the self-delete branch would fire on it.
    let (_after, delta) = b_trie
        .delete_member(&MemberId::new(b_member_id))
        .unwrap()
        .recalculate()
        .unwrap();

    // Signed by a keypair that is not the Organisation's published signing key.
    let forger = SigningKeypair::from_seed([0x99u8; 32]);
    assert_ne!(
        *forger.member_key().as_bytes(),
        org_pub_key,
        "the forging key must not be the Organisation's published signing key"
    );
    let envelope = SignedDeltaEnvelope::build(org_id, seq_before + 1, &delta, &forger).unwrap();
    let msg = WireMessage { envelope, org_secret: None, genesis_snapshot: None };

    // ---- B receives the forged revocation ----
    let ep_b = OrgEndpoint::bind(&b_device_kp).await.unwrap();
    let b_addr = ep_b.inner().addr();
    let mut svc_b_recv = svc_b.with_endpoint(ep_b);
    let b_task = tokio::spawn(async move {
        let r = tokio::time::timeout(
            NET,
            svc_b_recv.receive_and_self_delete_if_revoked(&mut OsRng),
        )
        .await
        .expect("B receive_and_self_delete_if_revoked timed out");
        (svc_b_recv, r)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let forger_device = SigningKeypair::from_seed([0x77u8; 32]);
    let ep_forger = OrgEndpoint::bind(&forger_device).await.unwrap();
    tokio::time::timeout(NET, ep_forger.send(b_addr, &msg))
        .await
        .expect("forged send timed out")
        .expect("forged send failed");

    let (svc_b_final, r) = b_task.await.unwrap();

    // The requirement: the message is rejected, and nothing is deleted.
    let err = r.expect_err("a revocation whose envelope fails verification must be rejected");
    assert!(
        matches!(err, OrgNodeError::BadSignature),
        "expected BadSignature, got {err:?}"
    );

    assert_eq!(
        svc_b_final.list_orgs().len(),
        1,
        "B must still hold its OrgRecord after a rejected revocation"
    );
    let rec = &svc_b_final.list_orgs()[0];
    assert_eq!(rec.org_id, org_id);
    assert_eq!(rec.epoch, epoch_before, "the record's epoch must be unchanged");
    assert_eq!(rec.root_hash, root_before, "the record's root must be unchanged");
    assert_eq!(rec.last_seq, seq_before, "the high-water mark must not advance");
    assert_eq!(
        rec.trie_members.len(),
        members_before,
        "the record's member count must be unchanged"
    );
    assert_eq!(
        svc_b_final
            .list_personas()
            .iter()
            .find(|p| p.persona_id == pid_b)
            .unwrap()
            .status,
        PersonaStatus::Active,
        "B's persona must stay Active after a rejected revocation"
    );
}
