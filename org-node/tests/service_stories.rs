//! Integration test: the five user stories via OrgService against chain
//! states from `ChainSlots` (passed as values, since 2026-10-08), each send
//! the body the sender's commit yields, delivered by value (since S4 task T4,
//! 2026-10-13; it was loopback iroh endpoints). Offline — no live chain, no
//! relay.
//!
//! Stories exercised:
//!   1. A creates a persona and an organisation → epoch 1 in the slots.
//!   2. B creates a persona and declares it expects the admission; A reads
//!      the joiner it admits B as (B's details and two public keys).
//!   3. A admits B (trie add → epoch 2 in the slots; envelope sent to B).
//!   4. B receives and verifies the envelope → B's persona Active, OrgRecord stored.
//!   5. A revokes B (trie remove → epoch 3); B self-deletes its OrgRecord.
//!
//! The gate: `cargo test -p org-node --features app,test-support --test service_stories`

#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;
use support::{
    admit, admit_b_directly, captured_admission_of_c, deliver_from_to_receive, device_kp, found, h,
    id_by_handle, joiner_for_c, joiner_of, nm, open_store, prepare_to_join, rec_of, reopen_store, revoke,
    revoke_and_tell, setup, sn, spawn_receive, spawn_recv_one, Relay,
};
use org_node::wire::WireMessage;

use org_node::service::{ReceiveKind, ReceiveOutcome};
use org_node::test_fixtures::ChainSlots;
use support::Node;
use org_node::store::PersonaStatus;
use org_node::test_fixtures::device_keypair;
use org_node::Epoch;
use support::device_seed_of;

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
// *2026-10-13 (S4, task T10):* the two endpoint-lifecycle items this line
// also named (bound once; bound from the named Persona's Device) moved to
// org-io with the endpoint, so org-node cannot cite them; this test binds no
// endpoint since T4, and org-io's `tests/endpoints.rs` verifies them.
// verifies: REQ-nhe2zu, REQ-txvtm9, REQ-xa6smf, REQ-uxv2x2, LLR-bg3vsw, LLR-q8emds, LLR-6p4pj2
#[test]
fn five_stories_full_e2e() {
    use rand::rngs::OsRng;

    // ---- Shared chain ----
    // *Rewritten 2026-10-13 (S4 task T4):* value-driven, no endpoint: each
    // send is the body A's commit yields, delivered from A's Device to B's
    // `receive_from` (support's `Inbox`); org-io keeps the copy over two
    // loopback endpoints.
    let chain = ChainSlots::new();

    // ---- Stores ----
    let mut svc_a = Node::new(open_store("e2e", "a"), chain.clone());
    let mut svc_b = Node::new(open_store("e2e", "b"), chain.clone());

    // ---- Story 1: A creates persona + org ----
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = found(&mut svc_a, &chain, &pid_a);

    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(1));
    assert_eq!(svc_a.list_personas()[0].status, PersonaStatus::Active);

    // ---- Story 2: B creates persona; B declares it expects the admission ----
    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();

    // B declares it expects the admission, and persists the expectation.
    prepare_to_join(&mut svc_b, org_id);
    assert_eq!(svc_b.expected_admissions()[0].org_id, org_id);

    // A reads the joiner it admits B as.
    let joiner_b = joiner_of(&svc_b, &pid_b);
    assert_eq!(joiner_b.handle.as_str(), "bob");

    // ---- Story 3+4: B waits, A admits ----
    let (b_addr_admit, recv_handle) = spawn_receive(svc_b);

    let b_member_id = admit(&mut svc_a, &chain, org_id, &joiner_b, b_addr_admit).expect("admit_member failed");

    // The chain slots: epoch 2.
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(2), "admit must bump to epoch 2");

    // Collect B's result + svc_b (moved out of the task).
    let (svc_b3, b_outcome) = recv_handle.run();
    let b_outcome = b_outcome.expect("B receive failed");

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
    assert_eq!(svc_b3.list_orgs()[0].org_private_key_for_test(), svc_a.list_orgs()[0].org_private_key_for_test());
    assert!(svc_b3.expected_admissions().is_empty(), "the expectation is cleared on commit");

    // ---- Story 5: A revokes B; B self-deletes ----

    // B waits for the revocation message.
    let (b_addr_revoke, revoke_recv_handle) = spawn_receive(svc_b3);

    revoke(&mut svc_a, &chain, org_id, b_member_id, b_addr_revoke).expect("revoke_member failed");

    // The chain slots: epoch 3.
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(3), "revoke must bump to epoch 3");

    // Collect B's self-delete result.
    let (svc_b_final, delete_outcome) = revoke_recv_handle.run();
    let delete_outcome = delete_outcome.expect("B's receive of its revocation failed");

    // S3: B acts on its notice, signs one acknowledgement, and forgets the
    // Organisation with its Persona (REQ-uxv2x2 as amended). *S4 T4:* on the
    // one receive path, as `ReceiveKind::Revoked`.
    let ReceiveOutcome { org_id: oid, kind: ReceiveKind::Revoked, acknowledgements, .. } = delete_outcome else {
        panic!("expected a revocation accepted, got {delete_outcome:?}");
    };
    assert_eq!(oid, org_id, "self-deleted org_id must match");
    assert_eq!(acknowledgements.len(), 1, "one acknowledgement, B's");
    assert_eq!(acknowledgements[0].member_id, b_member_id);

    // B must no longer have the OrgRecord.
    assert_eq!(
        svc_b_final.list_orgs().len(),
        0,
        "B must have no OrgRecords after self-delete"
    );

    // B's persona is deleted with its keys (was: marked Revoked).
    assert!(
        svc_b_final.list_personas().iter().all(|p| p.persona_id != pid_b),
        "B's persona must be gone after self-delete"
    );

    // A still has the org at epoch 3 with only its founding member in the trie.
    assert_eq!(svc_a.list_orgs().len(), 1);
    assert_eq!(svc_a.list_orgs()[0].epoch, Epoch::new(3));
    assert_eq!(svc_a.list_orgs()[0].trie_members.len(), 1, "only A should remain");
}

// Abnormal case of the self-delete rule: a revocation Change set that removes a
// *different* member must be verified, committed as an ordinary update, and must
// NOT trigger a self-delete. Story 5 above is the normal case (the device's own
// removal); this is the other side of the same requirement. On both receive
// paths B keeps the received Envelope's Change set (LLR-d9778a). *S4 T4,
// 2026-10-13:* there is one receive path; both updates arrive on it.
// verifies: REQ-uxv2x2, LLR-jsx922, LLR-d9778a
#[test]
fn revocation_of_another_member_is_committed_not_self_deleted() {
    use rand::rngs::OsRng;

    // ---- Shared chain and two services ----
    let chain = ChainSlots::new();
    let mut svc_a = Node::new(open_store("revoke-other", "a"), chain.clone());
    let mut svc_b = Node::new(open_store("revoke-other", "b"), chain.clone());

    // ---- Stories 1-2: A creates persona + org; B declares it expects the admission ----
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = found(&mut svc_a, &chain, &pid_a);
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(1));

    // A sends from A's persona's Device, so the sender B sees is A's persona
    // device key.
    let a_device_kp = device_kp(&svc_a, &pid_a);
    let mut svc_a = svc_a.sending_as(&a_device_kp);

    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    prepare_to_join(&mut svc_b, org_id);
    assert_eq!(svc_b.expected_admissions()[0].org_id, org_id);

    let jr_b = joiner_of(&svc_b, &pid_b);
    let b_device_kp = device_kp(&svc_b, &pid_b);

    // ---- Stories 3-4: A admits B; B receives and commits epoch 2 ----
    let (b_addr, b_task) = spawn_receive(svc_b);

    admit(&mut svc_a, &chain, org_id, &jr_b, b_addr)
        .expect("admit_member(B) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(2), "admitting B must bump to epoch 2");

    let (svc_b, r) = b_task.run();
    let outcome = r.expect("B's direct admission from A must verify");
    assert_eq!(outcome.epoch, Epoch::new(2));
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 2, "A + B");

    // ---- A creates persona C and admits it, pushing the update to B ----
    // C lives in A's store; A still sends as A's Persona, so B sees C's
    // admission arrive from A's device.
    let pid_c = svc_a.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let jr_c = joiner_of(&svc_a, &pid_c);
    assert_eq!(jr_c.handle.as_str(), "carol");

    let (b_addr, b_task) = spawn_receive(svc_b);

    let c_member_id = admit(&mut svc_a, &chain, org_id, &jr_c, b_addr)
        .expect("admit_member(C) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(3), "admitting C must bump to epoch 3");

    let (svc_b, r) = b_task.run();
    let outcome = r.expect("B must accept C's admission pushed by A's own device");
    assert_eq!(outcome.epoch, Epoch::new(3), "B must commit epoch 3");
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 3, "A + B + C");
    // B keeps the Change set of the Envelope it received — the one A
    // committed and sent (LLR-d9778a).
    let a_kept = svc_a.list_orgs()[0].kept_change_set.clone();
    assert!(a_kept.is_some(), "A keeps the Change set it committed");
    assert_eq!(svc_b.list_orgs()[0].kept_change_set, a_kept, "B keeps the received Envelope's Change set");

    // ---- A revokes C; B receives the revocation ----
    let (b_addr, b_task) = spawn_receive(svc_b);

    // Told to B's Device, which the committed record lists: Organisation
    // information (REQ-3dsweu).
    revoke_and_tell(&mut svc_a, &chain, org_id, c_member_id, b_device_kp.device_key().unwrap(), b_addr)
        .expect("revoke_member(C) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(4), "revoking C must bump to epoch 4");

    let (svc_b_final, r) = b_task.run();
    let delete_outcome = r.expect("B must verify and commit C's revocation");

    // The requirement: someone else's removal is an ordinary update, never a
    // self-delete. (*S4 T4:* the self-delete path folded onto the one path;
    // its `UpdatedNotRevoked` is `ReceiveKind::Updated`.)
    match delete_outcome {
        ReceiveOutcome { org_id: oid, kind: ReceiveKind::Updated, .. } => {
            assert_eq!(oid, org_id, "the updated org_id must match");
        }
        other => panic!("B must not self-delete when a different member is revoked, got {other:?}"),
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
    // The removal's Change set replaces the admission's (LLR-d9778a).
    let a_kept = svc_a.list_orgs()[0].kept_change_set.clone();
    assert!(a_kept.is_some(), "A keeps the revocation's Change set");
    assert_eq!(rec_b.kept_change_set, a_kept, "B keeps the received revocation's Change set");
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

// Abnormal-input case of the self-delete rule: a revocation of this node's
// own DevicePublicKey, well formed and naming the right Organisation, whose
// absence proof is genuine under a removal never published on chain,
// delivered by a device in no member's slots of the node's record. The
// member-sender rule refuses it with `SenderNotListed` (owner ruling R1 of
// 2026-10-07, LLR-kzgjz8); the same notice from A's listed device reaches the
// chain, which refuses it — the proof does not verify under its root. Each
// refusal leaves the OrgRecord exactly as it was. The node must never delete
// its record of the Organisation on a message it refused.
// *Rewritten 2026-10-07 (S3 T9):* the forged message was a revocation
// Envelope refused as `StaleEpoch`; a revocation now holds a notice, refused
// as `RevocationProofRefused` (LLR-pt32fx, LLR-tx8ruv).
// *Rewritten 2026-10-07 (S3 T12a):* split into the refusal from the forger
// and the chain's refusal from a listed device (was: the forger reached the
// chain, nothing about the sender being checked).
// verifies: REQ-uxv2x2, LLR-6qmq2g, LLR-vw2jn6, LLR-3q63zv, LLR-pt32fx, LLR-kzgjz8, LLR-tx8ruv
#[test]
fn revocation_from_an_unknown_device_leaves_the_record_in_place() {
    use rand::rngs::OsRng;

    use org_members::hasher::Blake3Hasher;
    use org_members::trie::OrgTrie;
    use org_members::MemberLeaf;
    use org_node::error::OrgNodeError;
    use org_node::revocation::RevocationNotice;
    use org_node::store::MemberSnapshot;
    use org_node::keys::{SigningKeypair, X25519Keypair};
    use org_node::wire::WireMessage;
    use org_node::PersonaId;

    type Trie = OrgTrie<Blake3Hasher>;

    /// The member and device keypairs of a persona, from its persisted seeds.
    fn keys_of(svc: &Node, persona_id: &PersonaId) -> (X25519Keypair, SigningKeypair) {
        let p = support::persona_of(svc, persona_id);
        (
            p.member_seed_for_test().x25519_keypair(),
            device_keypair(svc.keychain.seed(&p.device_key)),
        )
    }

    // ---- Two services over one shared mock chain ----
    let chain = ChainSlots::new();
    let mut svc_a = Node::new(open_store("unknown-sender", "a"), chain.clone());
    let mut svc_b = Node::new(open_store("unknown-sender", "b"), chain.clone());

    // ---- A creates persona + org; B declares it expects the admission ----
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = found(&mut svc_a, &chain, &pid_a);
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(1));

    let (a_member_kp, a_device_kp) = keys_of(&svc_a, &pid_a);
    let mut svc_a = svc_a.sending_as(&a_device_kp);

    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    prepare_to_join(&mut svc_b, org_id);

    let jr_b = joiner_of(&svc_b, &pid_b);
    let (b_member_kp, b_device_kp) = keys_of(&svc_b, &pid_b);

    // ---- A admits B; B receives and commits epoch 2 ----
    let (b_addr, b_task) = spawn_receive(svc_b);

    let b_member_id = admit(&mut svc_a, &chain, org_id, &jr_b, b_addr)
        .expect("admit_member(B) failed");
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(2), "admitting B must bump to epoch 2");

    let (svc_b, r) = b_task.run();
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

    // The removal really does remove B's own Device key, and its absence
    // proof is genuine — under a root the chain never published: were the
    // proof not checked against the chain, the self-delete would fire on it.
    let (after, _delta) = b_trie
        .delete_member(&b_member_id)
        .unwrap()
        .recalculate()
        .unwrap();
    let b_device = b_device_kp.device_key().unwrap();
    let proof = after.prove_absent(&b_member_id, &b_device).unwrap();
    proof.verify::<Blake3Hasher>(&after.root_hash().unwrap(), &b_member_id, &b_device).unwrap();

    // No signature to forge any more. The forging device is in no member's
    // slots of B's record, which the sender check catches; from a listed
    // device, only the chain gives it away.
    let forger_device = device_keypair([0x77u8; 32]);
    assert!(
        !svc_b.list_orgs()[0]
            .trie_members
            .iter()
            .any(|m| m.device_keys.contains(&forger_device.device_key().unwrap())),
        "the forging device must not be in B's record"
    );
    // S3: a revocation holds a notice — B's identity and the absence proof —
    // not an Envelope (rewritten at T9 from a forged revocation Envelope).
    let msg = WireMessage::Revocation(RevocationNotice { org_id, member_id: b_member_id, device: b_device, proof });

    // Nothing is deleted after either refusal.
    let assert_nothing_deleted = |svc: &Node| {
        assert_eq!(svc.list_orgs().len(), 1, "B must still hold its OrgRecord after a rejected revocation");
        let rec = &svc.list_orgs()[0];
        assert_eq!(rec.org_id, org_id);
        assert_eq!(rec.epoch, epoch_before, "the record's epoch must be unchanged");
        assert_eq!(rec.root_hash, root_before, "the record's root must be unchanged");
        assert_eq!(rec.last_seq, seq_before, "the high-water mark must not advance");
        assert_eq!(rec.trie_members.len(), members_before, "the record's member count must be unchanged");
        assert_eq!(
            svc.list_personas().iter().find(|p| p.persona_id == pid_b).unwrap().status,
            PersonaStatus::Active,
            "B's persona must stay Active after a rejected revocation"
        );
    };

    // ---- B receives the forged revocation from the forging device ----
    // *Rewritten 2026-10-07 (S3 T12a):* the member-sender rule (owner ruling
    // R1, LLR-kzgjz8) refuses it before the chain is read.
    let (b_addr, b_task) = spawn_receive(svc_b);

    let ep_forger = Relay::new(&forger_device);
    ep_forger.send(b_addr, &msg).expect("forged send failed");

    let (mut svc_b, r) = b_task.run();
    assert_eq!(r.map(|_| ()), Err(OrgNodeError::SenderNotListed { org_id }));
    assert_nothing_deleted(&svc_b);

    // ---- The same forged notice from A's listed device ----
    // The chain refuses it: the proof does not verify under its root.
    let a_seed = svc_a.keychain.seed(&a_device_kp.device_key().unwrap());
    let err = deliver_from_to_receive(&mut svc_b, a_seed, msg)
        .expect_err("a revocation the chain never published must be rejected");
    assert!(
        matches!(err, OrgNodeError::RevocationProofRefused { org_id: refused, .. } if refused == org_id),
        "expected RevocationProofRefused (the proof is not under the chain's root), got {err:?}"
    );
    assert_nothing_deleted(&svc_b);
}

/// verifies: LLR-8hdu9x, LLR-6ymd6d, LLR-a8z7r5
///
/// PR-qmvj83: revoking B in a two-Member Organisation sends B's Device a
/// revocation holding only B's identity and an absence proof that verifies
/// against the chain's root — no Envelope, Change set, snapshot, key, or any
/// leaf of A.
#[test]
fn a_revoked_device_receives_only_its_notice() {
    use org_node::wire::WireMessage;

    let mut s = admit_b_directly(setup("pr-qmvj83"));
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let (sink_addr, sink) = spawn_recv_one(s.svc_b.keychain.seed(&s.b_device_kp.device_key().unwrap()));
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, sink_addr).unwrap();
    let (_endpoint, _sender, message) = sink.run();
    let WireMessage::Revocation(notice) = message else { panic!("a revocation") };
    let b_device = s.b_device_kp.device_key().unwrap();
    assert_eq!((notice.org_id, notice.member_id, notice.device), (s.org_id, b_id, b_device));
    let root = s.chain.get(&s.org_id).unwrap().root_hash;
    notice.proof.verify::<org_members::hasher::Blake3Hasher>(&root, &b_id, &b_device).unwrap();
    let body = postcard::to_allocvec(&notice).unwrap();
    let a_leaf = &rec_of(&s.svc_a, s.org_id).trie_members[0];
    assert!(!body.windows(32).any(|window| window == a_leaf.member_key.as_bytes()), "no leaf of A");
}

/// verifies: LLR-6ymd6d, LLR-a8z7r5
///
/// An admission's outcome holds no notice; a revocation's holds exactly the
/// removed pair; a Device neither record lists is sent nothing.
/// *Rewritten 2026-10-13 (S4, task T5):* over the bodies the commit returns,
/// `send_update` and its refusal gone: a Device neither record lists has no
/// body.
#[test]
fn send_update_chooses_by_record_and_outcome() {
    use org_node::test_fixtures::device_key;
    use rand::rngs::OsRng;

    let notices = |outcome: &org_node::CommitOutcome| {
        support::notices_in(outcome).iter().map(|notice| (notice.member_id, notice.device)).collect::<Vec<_>>()
    };
    let mut s = admit_b_directly(setup("send-choice"));
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let outcome = s.svc_a.commit_update(&mut OsRng, s.org_id).unwrap();
    let b_device = s.b_device_kp.device_key().unwrap();
    assert_eq!(notices(&outcome), vec![(b_id, b_device)]);
    let stranger = device_key(42);
    assert!(outcome.outgoing.iter().all(|message| message.recipient != stranger), "no body for a stranger");
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let admission = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, admission.resulting_root, admission.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    assert!(notices(&s.svc_a.commit_update(&mut OsRng, s.org_id).unwrap()).is_empty());
}

/// verifies: LLR-pt32fx, LLR-6p4pj2
///
/// *2026-10-13 (S4, task T9):* the retired removal item's ID is dropped from
/// this annotation (owner's retirement ruling).
///
/// Story 5: B receives its notice, accepts it against the chain, signs one
/// acknowledgement, and forgets the Organisation and its Persona, on disk.
#[test]
fn a_revoked_node_acknowledges_then_forgets_everything() {
    let mut s = admit_b_directly(setup("story5-s3"));
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let (b_addr, b_task) = spawn_receive(s.svc_b);
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, b_addr).unwrap();
    let (svc_b, outcome) = b_task.run();
    let ReceiveOutcome { org_id, kind: ReceiveKind::Revoked, acknowledgements, .. } = outcome.unwrap() else {
        panic!("revoked")
    };
    assert_eq!((org_id, acknowledgements.len()), (s.org_id, 1));
    assert_eq!(acknowledgements[0].member_id, b_id);
    let chain = s.chain.get(&s.org_id).unwrap();
    assert_eq!((acknowledgements[0].epoch, acknowledgements[0].root), (chain.epoch, chain.root_hash));
    assert!(svc_b.list_orgs().is_empty() && svc_b.list_personas().is_empty());
    let disk = reopen_store("story5-s3", "b");
    assert!(disk.data().orgs.is_empty() && disk.data().personas.is_empty());
}

/// verifies: LLR-b27jr6, LLR-jsx922
///
/// *2026-10-13 (S4, task T9):* the retired removal item's ID is dropped from
/// this annotation (owner's retirement ruling).
///
/// Organisation information about a commit that keeps B listed (C's
/// admission) commits on the receive path as an update with no
/// acknowledgements. Organisation information about B's own removal —
/// sent to A's listed Device and relayed to B — removes B through the one
/// step: one acknowledgement under the chain's state, no record and no
/// Persona, on disk.
#[test]
fn an_org_information_commit_that_removes_the_node_forgets_through_the_one_step() {
    let mut s = admit_b_directly(setup("one-step"));
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");

    // Both messages come from A's Device, which B's record lists (owner
    // ruling R1 of 2026-10-07, LLR-2r2fha; S3 T12a).
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let admission = captured_admission_of_c(&mut s);
    let kept = deliver_from_to_receive(&mut s.svc_b, a_seed, admission).unwrap();
    assert_eq!(kept.epoch, Epoch::new(3));
    assert!(kept.acknowledgements.is_empty() && kept.kind == ReceiveKind::Updated);
    assert_eq!(rec_of(&s.svc_b, s.org_id).trie_members.len(), 3, "A + B + C");

    // *Amended 2026-10-13 (S4, task T5):* the commit builds no body for its
    // sender's own Device, so the removal is captured as told to C's Device
    // (a further Persona on A's node), which the committed record lists.
    let c_device = support::device_by_handle(&rec_of(&s.svc_a, s.org_id), "carol");
    let (sink_addr, sink) = spawn_recv_one(rand::random());
    revoke_and_tell(&mut s.svc_a, &s.chain, s.org_id, b_id, c_device, sink_addr).unwrap();
    let (_sink, _sender, removal) = sink.run();
    assert!(matches!(removal, WireMessage::OrgInformation { .. }), "C's Device is listed");
    let removed = deliver_from_to_receive(&mut s.svc_b, a_seed, removal).unwrap();
    let chain = s.chain.get(&s.org_id).unwrap();
    assert_eq!((removed.org_id, removed.epoch, removed.root), (s.org_id, chain.epoch, chain.root_hash));
    assert_eq!(removed.acknowledgements.len(), 1);
    assert_eq!(removed.acknowledgements[0].member_id, b_id);
    assert!(s.svc_b.list_orgs().is_empty() && s.svc_b.list_personas().is_empty());
    let disk = reopen_store("one-step", "b");
    assert!(disk.data().orgs.is_empty() && disk.data().personas.is_empty());
}

/// *Pointed at the successor 2026-10-15 (S4 TEST review round 2, finding-2;
/// the upgraded guardrails rule on superseded IDs):* this verified the
/// removal item LLR-jd82t6 superseded on 2026-10-13 (S4, task T9). That
/// item's definition is deleted (owner ruling of 2026-10-13), so only the
/// successor is cited; its content is kept by LLR-jd82t6.
///
/// verifies: LLR-jd82t6
///
/// Abnormal (boundary of the one removal step): a commit that removes a
/// Device, but not one of this node's, is no reason to forget. A commits C's
/// removal and B receives it from A's listed Device: each still holds the
/// Organisation and its Personas, in memory and on disk, and signs no
/// acknowledgement.
#[test]
fn a_commit_that_removes_another_member_forgets_nothing() {
    let mut s = admit_b_directly(setup("other-removed"));
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let admission = captured_admission_of_c(&mut s);
    deliver_from_to_receive(&mut s.svc_b, a_seed, admission).unwrap();
    let c_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "carol");
    let a_personas = s.svc_a.list_personas().len();

    let b_device = s.b_device_kp.device_key().unwrap();
    let (sink_addr, sink) = spawn_recv_one(rand::random());
    revoke_and_tell(&mut s.svc_a, &s.chain, s.org_id, c_id, b_device, sink_addr).unwrap();
    let (_sink, _sender, removal) = sink.run();
    assert_eq!(rec_of(&s.svc_a, s.org_id).trie_members.len(), 2, "A holds the record without C");
    assert_eq!(s.svc_a.list_personas().len(), a_personas);

    let received = deliver_from_to_receive(&mut s.svc_b, a_seed, removal).unwrap();
    assert!(received.acknowledgements.is_empty());
    assert_eq!((rec_of(&s.svc_b, s.org_id).epoch, rec_of(&s.svc_b, s.org_id).trie_members.len()), (Epoch::new(4), 2));
    assert_eq!(s.svc_b.list_personas().len(), 1);
    for party in ["a", "b"] {
        let disk = reopen_store("other-removed", party);
        assert_eq!(disk.data().orgs.len(), 1, "{party} keeps the Organisation on disk");
        assert!(!disk.data().personas.is_empty());
    }
}

