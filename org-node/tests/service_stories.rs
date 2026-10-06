//! Integration test: the five user stories via OrgService against MockChainOps
//! + loopback iroh endpoints.  Offline — no live chain, no relay.
//!
//! Stories exercised:
//!   1. A creates a persona and an organisation → epoch 1 in MockChain.
//!   2. B creates a persona and declares it expects the admission; A reads
//!      the joiner it admits B as (B's details and two public keys).
//!   3. A admits B (trie add → epoch 2 in MockChain; envelope pushed to B over iroh).
//!   4. B receives and verifies the envelope → B's persona Active, OrgRecord stored.
//!   5. A revokes B (trie remove → epoch 3); B self-deletes its OrgRecord.
//!
//! The gate: `cargo test -p org-node --features app --test service_stories`

#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;
use support::{
    admit, device_kp, found, h, joiner_of, nm, open_store, prepare_to_join, reopen_store, revoke, revoke_and_tell, sn,
    spawn_receive, spawn_self_delete,
};

use std::time::Duration;

use org_node::service::{MockChainOps, OrgService, SelfDeleteOutcome};
use org_node::store::PersonaStatus;
use org_node::transport::endpoint::OrgEndpoint;
use org_node::{DeviceSeed, Epoch, SequenceNumber};

// The proper end-to-end test that runs all 5 stories in one function.
// `five_stories_headless` (stories 1-4 only, no invite) was
// deleted in the Fix 3 cleanup — `five_stories_full_e2e` is the canonical gate.
//
// Normal cases of three org-node requirements, over the service API: story 4
// commits the applied Change set with the chain's epoch and root (REQ-nhe2zu,
// and REQ-txvtm9's epoch rule with it) as a first admission (REQ-xa6smf);
// story 5 is the self-delete on the device's own removal
// (REQ-uxv2x2). Further abnormal cases are in verify_against_chain.rs and
// admission_sender.rs; REQ-uxv2x2's own abnormal-input case — a revocation
// from a device outside its record — is the last test in this file.
// (Merged with master 05f6f04: LLR-ghja3x and LLR-37cj3n, which state that
// the envelope is signed and verified under a signing key, are not carried —
// the Envelope has no signature on this branch. *Amended 2026-10-05:*
// LLR-rb8r65 is amended in place. *T10 of the chain-authority change:*
// LLR-rb8r65 and LLR-t4znbk are verified in commit_paths.rs.)
// verifies: REQ-nhe2zu, REQ-txvtm9, REQ-xa6smf, REQ-uxv2x2, LLR-bg3vsw, LLR-q8emds, LLR-6zjzn2, LLR-cns6q6, LLR-6p4pj2
#[tokio::test(flavor = "multi_thread")]
async fn five_stories_full_e2e() {
    use rand::rngs::OsRng;

    // ---- Shared chain ----
    let chain = MockChainOps::new();
    let chain_a = chain.clone();
    let chain_b_admit = chain.clone();   // for story 3/4
    let chain_b_revoke = chain.clone();  // for story 5

    // B's device keypair (fixed seed so we can reconstruct it for story 5).
    let b_device_kp = DeviceSeed::from([0x22u8; 32]).signing_keypair();

    // ---- Bind B's admission endpoint ----
    let ep_b_admit = OrgEndpoint::bind(&b_device_kp).await.unwrap();
    let b_addr_admit = ep_b_admit.inner().addr();

    // ---- Stores (cleared first: a reused pid would open a store an earlier
    // run left, review round 6) ----
    // svc_a starts without an endpoint; we bind it below once we know A's device seed.
    let mut svc_a = OrgService::new(open_store("e2e", "a", "pw_a"), Box::new(chain_a));
    let mut svc_b = OrgService::new(open_store("e2e", "b", "pw_b"), Box::new(chain_b_admit));

    // ---- Story 1: A creates persona + org ----
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = found(&mut svc_a, &chain, &pid_a).await;

    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(1));
    assert_eq!(svc_a.list_personas()[0].status, PersonaStatus::Active);

    // Bind A's endpoint from the SAME device seed that `create_persona` generated.
    // The QUIC-authenticated sender identity on B's side then equals A's
    // persona device key; nothing checks it.
    let a_device_kp = device_kp(&svc_a, &pid_a);
    let ep_a_admit = OrgEndpoint::bind(&a_device_kp).await.unwrap();
    let mut svc_a = svc_a.with_endpoint(ep_a_admit);

    // ---- Story 2: B creates persona; B declares it expects the admission ----
    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();

    // B declares it expects the admission, and persists the expectation.
    prepare_to_join(&mut svc_b, org_id);
    assert_eq!(svc_b.expected_admissions()[0].org_id, org_id);

    // A reads the joiner it admits B as.
    let ep_b_for_jr = OrgEndpoint::bind(&b_device_kp).await.unwrap();
    let b_addr_jr = ep_b_for_jr.inner().addr();
    let svc_b = svc_b.with_endpoint(ep_b_for_jr);

    let joiner_b = joiner_of(&svc_b, &pid_b);
    assert_eq!(joiner_b.handle.as_str(), "bob");

    // Rebuild svc_b with ep_b_admit so B can receive A's push.
    let store_b2 = reopen_store("e2e", "b", "pw_b");
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

    let b_member_id = admit(&mut svc_a, &chain, org_id, &joiner_b, b_addr_admit)
        .await
        .expect("admit_member failed");

    // MockChain: epoch 2.
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(2), "admit must bump to epoch 2");

    // Collect B's result + svc_b (moved out of the task).
    let (svc_b3, b_outcome) = recv_handle.await.unwrap();

    assert_eq!(b_outcome.org_id, org_id, "B's outcome org_id must match");
    assert_eq!(b_outcome.epoch, Epoch::new(2), "B must commit epoch 2");
    assert_eq!(
        b_outcome.root,
        chain.get(&org_id).unwrap().root_hash,
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
    assert_eq!(svc_b3.list_orgs()[0].epoch, Epoch::new(2));
    assert_eq!(svc_b3.list_orgs()[0].org_private_key, svc_a.list_orgs()[0].org_private_key);
    assert!(svc_b3.expected_admissions().is_empty(), "the expectation is cleared on commit");

    // ---- Story 5: A revokes B; B self-deletes ----

    // Bind a fresh B endpoint for receiving the revocation.
    let b_device_kp2 = DeviceSeed::from([0x22u8; 32]).signing_keypair();
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

    revoke(&mut svc_a, &chain, org_id, b_member_id, Some(b_addr_revoke))
        .await
        .expect("revoke_member failed");

    // MockChain: epoch 3.
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(3), "revoke must bump to epoch 3");

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

    // A still has the org at epoch 3 with only its founding member in the trie.
    assert_eq!(svc_a.list_orgs().len(), 1);
    assert_eq!(svc_a.list_orgs()[0].epoch, Epoch::new(3));
    assert_eq!(svc_a.list_orgs()[0].trie_members.len(), 1, "only A should remain");

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

    // ---- Shared chain and two services ----
    let chain = MockChainOps::new();
    let mut svc_a = OrgService::new(open_store("revoke-other", "a", "pw_a"), Box::new(chain.clone()));
    let mut svc_b = OrgService::new(open_store("revoke-other", "b", "pw_b"), Box::new(chain.clone()));

    // ---- Stories 1-2: A creates persona + org; B declares it expects the admission ----
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = found(&mut svc_a, &chain, &pid_a).await;
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(1));

    // A's endpoint is bound from A's persona device seed, so the authenticated
    // QUIC sender on B's side equals A's persona device key.
    let a_device_kp = device_kp(&svc_a, &pid_a);
    let mut svc_a = svc_a.with_endpoint(OrgEndpoint::bind(&a_device_kp).await.unwrap());

    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    prepare_to_join(&mut svc_b, org_id);
    assert_eq!(svc_b.expected_admissions()[0].org_id, org_id);

    let jr_b = joiner_of(&svc_b, &pid_b);
    let b_device_kp = device_kp(&svc_b, &pid_b);

    // ---- Stories 3-4: A admits B; B receives and commits epoch 2 ----
    let (b_addr, b_task) = spawn_receive(svc_b, &b_device_kp).await;

    admit(&mut svc_a, &chain, org_id, &jr_b, b_addr)
        .await
        .expect("admit_member(B) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(2), "admitting B must bump to epoch 2");

    let (svc_b, r) = b_task.await.unwrap();
    let outcome = r.expect("B's direct admission from A must verify");
    assert_eq!(outcome.epoch, Epoch::new(2));
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 2, "A + B");

    // ---- A creates persona C and admits it, pushing the update to B ----
    // C lives in A's store; A's endpoint stays bound from A's Persona, so B
    // sees C's admission arrive from A's device.
    let pid_c = svc_a.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let jr_c = joiner_of(&svc_a, &pid_c);
    assert_eq!(jr_c.handle.as_str(), "carol");

    let (b_addr, b_task) = spawn_receive(svc_b, &b_device_kp).await;

    let c_member_id = admit(&mut svc_a, &chain, org_id, &jr_c, b_addr)
        .await
        .expect("admit_member(C) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(3), "admitting C must bump to epoch 3");

    let (svc_b, r) = b_task.await.unwrap();
    let outcome = r.expect("B must accept C's admission pushed by A's own device");
    assert_eq!(outcome.epoch, Epoch::new(3), "B must commit epoch 3");
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 3, "A + B + C");

    // ---- A revokes C; B receives the revocation ----
    let (b_addr, b_task) = spawn_self_delete(svc_b, &b_device_kp).await;

    // Told to B's Device, which the committed record lists: Organisation
    // information (REQ-3dsweu).
    revoke_and_tell(&mut svc_a, &chain, org_id, c_member_id, b_device_kp.device_key().unwrap(), b_addr)
        .await
        .expect("revoke_member(C) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(4), "revoking C must bump to epoch 4");

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
    assert_eq!(rec_b.epoch, Epoch::new(4), "B must commit epoch 4");
    assert_eq!(
        rec_b.root_hash,
        chain.get(&org_id).unwrap().root_hash,
        "B's committed root must match the on-chain root"
    );
    assert_eq!(rec_b.trie_members.len(), 2, "A + B only");
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
// removes this node's own DevicePublicKey, well formed and naming the right
// Organisation, but never published on chain, delivered by a device in no
// member's slots of the node's record. Nothing about the sender is checked
// (LLR-3q63zv, amended 2026-10-05); the chain refuses it, and the refusal
// leaves the OrgRecord exactly as it was. The node must never delete its
// record of the Organisation on a message it refused.
// verifies: REQ-uxv2x2, LLR-6qmq2g, LLR-vw2jn6, LLR-3q63zv
#[tokio::test(flavor = "multi_thread")]
async fn revocation_from_an_unknown_device_leaves_the_record_in_place() {
    use rand::rngs::OsRng;

    use org_members::hasher::Blake3Hasher;
    use org_members::trie::OrgTrie;
    use org_members::MemberLeaf;
    use org_node::envelope::Envelope;
    use org_node::error::OrgNodeError;
    use org_node::store::MemberSnapshot;
    use org_node::keys::{SigningKeypair, X25519Keypair};
    use org_node::transport::wire::WireMessage;
    use org_node::PersonaId;

    type Trie = OrgTrie<Blake3Hasher>;

    /// The member and device keypairs of a persona, from its persisted seeds.
    fn keys_of(svc: &OrgService, persona_id: &PersonaId) -> (X25519Keypair, SigningKeypair) {
        let p = svc
            .list_personas()
            .iter()
            .find(|p| &p.persona_id == persona_id)
            .expect("persona not found");
        (
            p.member_seed.x25519_keypair(),
            p.device_seed.signing_keypair(),
        )
    }

    // ---- Two services over one shared mock chain ----
    let chain = MockChainOps::new();
    let mut svc_a = OrgService::new(open_store("unknown-sender", "a", "pw_a"), Box::new(chain.clone()));
    let mut svc_b = OrgService::new(open_store("unknown-sender", "b", "pw_b"), Box::new(chain.clone()));

    // ---- A creates persona + org; B declares it expects the admission ----
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = found(&mut svc_a, &chain, &pid_a).await;
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(1));

    let (a_member_kp, a_device_kp) = keys_of(&svc_a, &pid_a);
    let mut svc_a = svc_a.with_endpoint(OrgEndpoint::bind(&a_device_kp).await.unwrap());

    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    prepare_to_join(&mut svc_b, org_id);

    let jr_b = joiner_of(&svc_b, &pid_b);
    let (b_member_kp, b_device_kp) = keys_of(&svc_b, &pid_b);

    // ---- A admits B; B receives and commits epoch 2 ----
    let (b_addr, b_task) = spawn_receive(svc_b, &b_device_kp).await;

    let b_member_id = admit(&mut svc_a, &chain, org_id, &jr_b, b_addr)
        .await
        .expect("admit_member(B) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(2), "admitting B must bump to epoch 2");

    let (svc_b, r) = b_task.await.unwrap();
    r.expect("B's direct admission from A must verify");

    // ---- Forge a revocation of B, sent from a device B's record does not hold ----
    // Rebuild B's committed trie from B's own snapshot. An integration test
    // cannot call the crate's private snapshot-to-trie helper, so the leaves
    // are rebuilt from the two personas' seeds and matched to the snapshot by
    // member key; the root assertion below proves the reconstruction faithful.
    let (epoch_before, members_before, root_before, seq_before) = {
        let rec = &svc_b.list_orgs()[0];
        (rec.epoch, rec.trie_members.len(), rec.root_hash, rec.last_seq)
    };

    let leaf_of = |s: &MemberSnapshot| -> MemberLeaf {
        let (member, device) = if s.member_key == a_member_kp.member_key().expect("valid key") {
            (&a_member_kp, &a_device_kp)
        } else if s.member_key == b_member_kp.member_key().expect("valid key") {
            (&b_member_kp, &b_device_kp)
        } else {
            panic!("snapshot member key belongs to neither A nor B");
        };
        MemberLeaf::new(
            s.id,
            s.handle.clone(),
            member.member_key().expect("valid key"),
            s.name.clone(),
            s.surname.clone(),
            vec![device.device_key().unwrap()],
        )
        .unwrap()
    };
    let leaves: Vec<MemberLeaf> =
        svc_b.list_orgs()[0].trie_members.iter().map(leaf_of).collect();
    let b_trie = Trie::genesis(leaves).unwrap();
    assert_eq!(
        b_trie.root_hash().unwrap(),
        root_before,
        "the rebuilt trie must reproduce B's committed root"
    );

    // The Change set really does remove B's own Device key: were verification
    // skipped or its error ignored, the self-delete branch would fire on it.
    let (_after, delta) = b_trie
        .delete_member(&b_member_id)
        .unwrap()
        .recalculate()
        .unwrap();

    // No signature to forge any more, and the sender is not checked: only the
    // chain gives it away. The forging device is in no member's slots of B's
    // record.
    let forger_device = DeviceSeed::from([0x77u8; 32]).signing_keypair();
    assert!(
        !svc_b.list_orgs()[0]
            .trie_members
            .iter()
            .any(|m| m.device_keys.contains(&forger_device.device_key().unwrap())),
        "the forging device must not be in B's record"
    );
    let envelope = Envelope::build(org_id, SequenceNumber::new(seq_before.get() + 1), &delta).unwrap();
    let msg = WireMessage::Revocation { envelope };

    // ---- B receives the forged revocation ----
    let (b_addr, b_task) = spawn_self_delete(svc_b, &b_device_kp).await;

    let ep_forger = OrgEndpoint::bind(&forger_device).await.unwrap();
    tokio::time::timeout(support::NET, ep_forger.send(b_addr, &msg))
        .await
        .expect("forged send timed out")
        .expect("forged send failed");

    let (svc_b_final, r) = b_task.await.unwrap();

    // The requirement: the message is rejected, and nothing is deleted.
    let err = r.expect_err("a revocation the chain never published must be rejected");
    assert!(
        matches!(err, OrgNodeError::StaleEpoch { .. }),
        "expected StaleEpoch (the chain holds no newer state), got {err:?}"
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
