#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration tests: who delivers a message to a node's ordinary receive
//! (`prepare_receive` then `apply_receive`, in the sequence org-io runs:
//! `support::Node::receive_and_verify`) and to the self-delete path, over the service API, with real iroh
//! endpoints on loopback and a shared `ChainSlots`. Offline — no live chain,
//! no relay.
//!
//! A (the founding node) always produces a chain-valid admission envelope. The
//! Envelope carries no signature; what varies is *who delivers it* to the
//! member (B). A chain-valid first admission is committed whichever device
//! relays it (REQ-xa6smf, owner amendment of 2026-10-07). Since the owner
//! rulings of 2026-10-07 (R1; LLR-2r2fha, LLR-kzgjz8) an update for an
//! Organisation B holds, and a revocation, are acted on only when the sending
//! Device is listed in B's record; the 2026-10-05 ruling that nothing about
//! the sender is checked is reversed for them.
//!
//! The same setup carries the MemberId tests: ids are random, not keys, and a
//! re-admission with the same keys gets a fresh id (REQ-d9g6nt).
//!
//! The gate:
//! `cargo test -p org-node --features app,test-support --test admission_sender`

mod support;
use support::*;

use org_members::MemberId;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::service::SelfDeleteOutcome;
use org_node::test_fixtures::ChainSlots;
use org_node::store::{OrgRecord, PersonaRecord, PersonaStatus};
use org_node::transport::endpoint::OrgEndpoint;
use org_node::transport::wire::WireMessage;
use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, OrgPublicKey, PersonaId, RootHash, SequenceNumber};
use rand::rngs::OsRng;

// REQ-xa6smf as amended 2026-10-05: a first admission that verifies against
// the chain is committed whichever device delivers it. A rogue relay R
// forwards A's genuine admission of B; B commits it and clears its expectation.
// The member-sender rule of 2026-10-07 (LLR-2r2fha) exempts a first
// admission, by the owner's amendment the same day: "to avoid scenarios where
// something happens to the admin's device during this window"; "the new
// joiner has no org information to disclose, and they verify the org
// information they receive on-chain so the risk here is only a new joiner
// being DoS'ed which is acceptable". This is that exemption clause.
// verifies: REQ-xa6smf, LLR-2r2fha, LLR-j83kc8, LLR-9fvb3y
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_relayed_by_another_device_is_committed() {
    let mut s = setup("first-rogue").await;

    // R waits for A's push; A "admits B" but is handed R's address.
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, r_addr)
        .await
        .expect("admit_member(B via R) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(2));

    let (ep_r, sender_seen_by_r, msg) = r_task.await.unwrap();
    assert_eq!(
        sender_seen_by_r,
        device_kp(&s.svc_a, &s.svc_a.list_personas()[0].persona_id).device_key().unwrap(),
        "R received the admission from A's device"
    );
    assert!(matches!(msg, WireMessage::OrgInformation { .. }), "Organisation information");

    // R relays the identical message to B.
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
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
    assert!(svc_b.expected_admissions().is_empty(), "the expectation is cleared on commit");
}

// Normal case of an update: after B has committed epoch 2, a further update
// (C's admission) pushed by A's own device is verified and committed.
// It is also LLR-9f5hmr's normal case (the Sequence number equals the chain's
// epoch, and the mark becomes that epoch).
// verifies: REQ-ztdza4, LLR-u6rq4s, LLR-cja9zv, LLR-9f5hmr
#[tokio::test(flavor = "multi_thread")]
async fn update_from_the_admin_after_admission_is_committed() {
    let mut s = admit_b_directly(setup("update-admin").await).await;
    let seq_at_2 = s.svc_b.list_orgs()[0].last_seq;

    let jr_c = joiner_for_c(&mut s.svc_a);

    // B waits for the next update; A admits C, pushing the envelope to B.
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, b_addr)
        .await
        .expect("admit_member(C) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(3), "admitting C must bump to epoch 3");

    let (svc_b, result) = b_task.await.unwrap();
    let outcome = result.expect("B must accept an update sent by A's own device");
    assert_eq!(outcome.org_id, s.org_id);
    assert_eq!(outcome.epoch, Epoch::new(3), "B must commit epoch 3");
    assert_eq!(
        outcome.root,
        s.chain.get(&s.org_id).unwrap().root_hash,
        "B's committed root must match the on-chain root"
    );
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(svc_b.list_orgs()[0].epoch, Epoch::new(3));
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 3, "A + B + C");
    // The sequence mark advances with the commit. Added 2026-10-03: this is
    // the UPDATE branch of the commit, distinct from the first-admission
    // branch that `a_committed_admission_reaches_the_disk_and_clears_the_expectation`
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

// The member-sender rule (owner ruling R1 of 2026-10-07, LLR-2r2fha): an
// update for an Organisation B holds is acted on only when the sending Device
// is listed in B's record. A rogue device R relays A's genuine admission of
// C: B refuses it with `SenderNotListed`, its record and its store file
// unchanged. The same message from A's own Device is committed (epoch 3).
// *Rewritten 2026-10-07 (S3 T12a)*: was
// `update_relayed_by_a_non_member_after_admission_is_committed` (REQ-ztdza4
// and REQ-ag6kqm as amended 2026-10-05: committed whoever relays it).
// verifies: LLR-2r2fha, LLR-u6rq4s
#[tokio::test(flavor = "multi_thread")]
async fn update_relayed_by_a_non_member_after_admission_is_refused() {
    let mut s = admit_b_directly(setup("update-rogue").await).await;
    let root_at_2 = s.svc_b.list_orgs()[0].root_hash;
    let record_at_2 = s.svc_b.list_orgs()[0].clone();

    let jr_c = joiner_for_c(&mut s.svc_a);

    // R captures A's C-admission push.
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, r_addr)
        .await
        .expect("admit_member(C via R) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(3));
    let (ep_r, _sender, msg) = r_task.await.unwrap();

    // R relays it to B.
    let before = store_bytes("update-rogue", "b");
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, ep_r.send(b_addr, &msg))
        .await
        .expect("R relay send timed out")
        .expect("R relay send failed");

    let (mut svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.map(|_| ()), Err(OrgNodeError::SenderNotListed { org_id: s.org_id }));
    let unchanged = svc_b.list_orgs()[0].clone();
    assert_eq!(
        (unchanged.epoch, unchanged.root_hash, unchanged.last_seq, &unchanged.trie_members),
        (record_at_2.epoch, record_at_2.root_hash, record_at_2.last_seq, &record_at_2.trie_members),
        "the refusal leaves B's record as it was"
    );
    assert_eq!(store_bytes("update-rogue", "b"), before, "the refusal writes nothing");

    // The same message from A's own Device is committed.
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let outcome = deliver_from_to_receive(&mut svc_b, a_seed, msg)
        .await
        .expect("an update from a listed Device is committed");
    assert_eq!(outcome.epoch, Epoch::new(3));
    assert_eq!(svc_b.list_orgs()[0].epoch, Epoch::new(3));
    assert_eq!(svc_b.list_orgs()[0].last_seq, SequenceNumber::new(3), "the mark is the epoch committed");
    assert_ne!(svc_b.list_orgs()[0].root_hash, root_at_2, "B's root moved to the chain's");
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 3, "A + B + C");
}

// ---- added 2026-10-03 by the architecture tooth ----------------------------

// The member-sender rule for revocations (owner ruling R1 of 2026-10-07,
// LLR-kzgjz8): a revocation is acted on only when the sending Device is
// listed in the receiver's record. A rogue device R relays A's genuine notice
// for B to B's self-delete path: B refuses it with `SenderNotListed`, keeps
// its record and its Persona, and no chain read happens — so the seed source,
// called only after the read and every check (LLR-tx8ruv), is not reached
// either. The same notice from A's own Device makes B self-delete with one
// acknowledgement.
// *Rewritten 2026-10-07 (S3 T12a)*: was
// `a_revocation_relayed_by_a_non_member_is_still_acted_on` (REQ-ztdza4 and
// LLR-3q63zv as amended 2026-10-05: acted on whoever relays it).
// verifies: LLR-kzgjz8
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_relayed_by_a_non_member_is_refused() {
    let (s, counting) = setup_counting("revoke-relay").await;
    let mut s = admit_b_directly(s).await;
    let b_member_id = s.svc_b.list_personas()[0].member_id.expect("B has a member id");

    // R stands between A and B: A revokes B and sends the message to R's
    // address; R captures it and relays it on to B.
    let (rogue_addr, rogue_task) = spawn_recv_one(ROGUE_SEED).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_member_id, Some(rogue_addr))
        .await
        .expect("revoke_member failed");

    let (rogue_ep, sender, captured) = rogue_task.await.unwrap();
    assert_eq!(
        sender.as_bytes(),
        s.svc_a.endpoint().expect("A endpoint").device_key().as_bytes(),
        "R must have received the revocation from A itself"
    );
    assert!(matches!(captured, WireMessage::Revocation(_)), "B's Device is no longer listed");

    let reads_before = counting.reads();
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    rogue_ep.send(b_addr, &captured).await.expect("rogue relay to B failed");
    let (mut svc_b, refused) = b_task.await.unwrap();
    assert_eq!(refused.map(|_| ()), Err(OrgNodeError::SenderNotListed { org_id: s.org_id }));
    assert_eq!(counting.reads(), reads_before, "the refusal reads no chain");
    assert_eq!(svc_b.list_orgs().len(), 1, "B keeps its record");
    assert_eq!(svc_b.list_personas().len(), 1, "B keeps its Persona");

    // The same notice from A's own Device.
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let outcome = deliver_from_to_self_delete(&mut svc_b, a_seed, captured)
        .await
        .expect("B acts on a chain-valid revocation from a listed Device");
    assert!(
        matches!(&outcome, SelfDeleteOutcome::SelfDeleted { acknowledgements, .. } if acknowledgements.len() == 1),
        "expected SelfDeleted with B's acknowledgement, got {outcome:?}"
    );
    assert!(svc_b.list_orgs().is_empty(), "B's record of the org must be gone");
    // S3 T9: the Persona is deleted with its keys (was: marked Revoked).
    assert!(svc_b.list_personas().is_empty(), "B's Persona must be gone");
}

// A first admission whose snapshot and Change set are consistent, for an
// Organisation the chain does not know: refused with `OrgNotOnChain` after the
// chain-free checks (LLR-3wb7th), not with a "no such record" error.
// verifies: REQ-bvh8v6, LLR-3wb7th
#[tokio::test(flavor = "multi_thread")]
async fn a_message_for_an_organisation_absent_from_the_chain_is_refused() {
    let mut s = setup("absent-org").await;
    s.svc_b.expect_admission(&mut OsRng, OrgId::new([0xeeu8; 20])).unwrap();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;

    // A well-formed first admission for an org that was never put on the mock
    // chain, carrying the snapshot its Change set extends.
    let stranger = MemberSeed::from([0x5au8; 32]).x25519_keypair();
    let base = org_node::test_fixtures::genesis_trie(&stranger, &org_node::test_fixtures::admin_device());
    let (delta, _) = org_node::test_fixtures::admit_member_delta(&stranger);
    let absent_org = OrgId::new([0xeeu8; 20]);
    let envelope = org_node::Envelope::build(absent_org, SequenceNumber::new(1), &delta).unwrap();
    let msg = WireMessage::OrgInformation {
        envelope,
        record_snapshot: snapshot_bytes(&base),
        org_private_key: OrgPrivateKey::from([0x42u8; 32]),
    };

    let relay = OrgEndpoint::bind(&DeviceSeed::from([0x5bu8; 32]).signing_keypair()).await.unwrap();
    relay.send(b_addr, &msg).await.expect("send to B failed");

    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert!(svc_b.list_orgs().is_empty(), "nothing may be committed");
}

// The committing half of `receive_and_verify`, which the falsifiability sweep
// of 2026-10-03 found almost unevidenced: deleting `self.store.save(rng)?`,
// writing a zero sequence mark, and never clearing what the first admission
// consumes all left the suite green. Everything asserted here is read back
// from DISK, so a commit that only ever existed in memory fails it.
// verifies: REQ-nhe2zu, REQ-txvtm9, LLR-cja9zv, LLR-q8emds
#[tokio::test(flavor = "multi_thread")]
async fn a_committed_admission_reaches_the_disk_and_clears_the_expectation() {
    let s = setup("commit-persisted").await;
    // Before admission B holds the expectation it declared.
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
        "A + B must be persisted as snapshots"
    );

    // The expectation is cleared, and that too reached the disk.
    assert!(reloaded.data().expected_admissions.is_empty(), "the expectation is cleared on commit");
}

// The admitting node's own side of an admission: A updates and persists its
// Organisation record. Added 2026-10-04 after review round 1, which found that
// deleting `self.store.save(rng)?` from `admit_member` left the whole gate
// green — the twin, on the write path, of the gap the sweep had already found
// and closed on the receive path.
// *Amended by T10 of the chain-authority change:* the record is now written
// by `commit_update`, which the story helper calls; LLR-t4znbk, LLR-rb8r65
// and LLR-ghja3x are verified in commit_paths.rs.
// verifies: LLR-cmdrp9, LLR-cja9zv
#[tokio::test(flavor = "multi_thread")]
async fn an_admission_reaches_the_administrators_disk() {
    let s = admit_b_directly(setup("admin-persisted").await).await;

    let in_memory = s.svc_a.list_orgs()[0].clone();
    assert_eq!(in_memory.epoch, Epoch::new(2), "A must hold the epoch it committed");
    assert_eq!(in_memory.trie_members.len(), 2, "A + B");
    assert_ne!(in_memory.last_seq, SequenceNumber::new(0), "A must hold the sequence of its update");

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

// LLR-t4znbk as amended by the chain-authority change: the record is the
// commit's, never the send's. The chain carries the update and the node
// commits it before anything is sent, so a send that fails leaves the
// committed record in place — epoch, member and mark advanced with the
// chain, in memory and on disk. (Until T10 this test pinned the opposite:
// the record stayed behind when the push failed.)
//
// The send is made to fail by dialling a well-formed peer identity carrying
// NO transport addresses: the sending endpoint is Loopback mode, so the relay
// is disabled and there is no address discovery; iroh has no path to try and
// refuses immediately. (Dialling a loopback port nothing listens on does NOT
// work here — QUIC retries it for thirty seconds.)
// verifies: LLR-t4znbk
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_send_leaves_the_committed_record_in_place() {
    let mut s = setup("admin-send-fails").await;
    let epoch_before = s.chain.get(&s.org_id).unwrap().epoch;
    let before = s.svc_a.list_orgs()[0].clone();

    let result = admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, dead_addr([0x6fu8; 32])).await;
    assert!(result.is_err(), "a send to a peer that is not listening must fail");

    let chain_after = s.chain.get(&s.org_id).unwrap();
    assert_eq!(chain_after.epoch, Epoch::new(epoch_before.get() + 1), "the chain carries the update");
    for (after, place) in [
        (s.svc_a.list_orgs()[0].clone(), "in memory"),
        (disk_rec_of(&reopen_store("admin-send-fails", "a", "pw_a"), s.org_id), "on disk"),
    ] {
        assert_eq!(after.epoch, chain_after.epoch, "the record's epoch advanced with the chain ({place})");
        assert_eq!(after.root_hash, chain_after.root_hash, "the record holds the chain's root ({place})");
        assert_eq!(after.last_seq, SequenceNumber::new(before.last_seq.get() + 1), "the mark advanced ({place})");
        assert_eq!(after.trie_members.len(), before.trie_members.len() + 1, "the member is recorded ({place})");
        assert!(after.trie_members.iter().any(|m| m.handle.as_str() == "bob"), "B is the member added ({place})");
    }
}

// Normal case of REQ-d9g6nt: the founding admin's id and an admitted
// member's id are not their keys, and differ from each other.
// verifies: REQ-d9g6nt, LLR-ag9mgm, LLR-rjg3m2
#[tokio::test(flavor = "multi_thread")]
async fn member_ids_are_not_derived_from_keys() {
    let mut s = setup("ids-not-keys").await;

    let admin_snap = s.svc_a.list_orgs()[0].trie_members[0].clone();
    assert_eq!(admin_snap.member_key, s.svc_a.persona_public_keys(&s.pid_a).unwrap().0);
    assert_ne!(admin_snap.id.as_bytes(), admin_snap.member_key.as_bytes(), "the admin's id is its member key");
    assert_ne!(admin_snap.id.as_bytes(), admin_snap.device_keys[0].as_bytes(), "the admin's id is its device key");

    // A admits B, delivered to B's own endpoint as in `admit_b_directly`.
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    let id_b = admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, b_addr)
        .await
        .expect("admit_member(B) failed");
    let (_svc_b, outcome) = b_task.await.unwrap();
    outcome.expect("B's admission from A must verify");

    assert_ne!(id_b.as_bytes(), s.joiner_b.member_key.as_bytes(), "B's id is its member key");
    assert_ne!(id_b.as_bytes(), s.joiner_b.device_key.as_bytes(), "B's id is its device key");
    assert_ne!(id_b, admin_snap.id, "B's id equals the admin's id");
    assert!(
        s.svc_a.list_orgs()[0].trie_members.iter().any(|m| m.id == id_b),
        "A's record holds B under the returned id"
    );
}

// Abnormal case of REQ-d9g6nt: admit B, revoke B by MemberId, admit the
// SAME joiner (same member key and device key) again. Re-admission
// succeeds (owner ruling: same keys allowed), and each new id differs from
// every deleted id and from every key. Three rounds.
// verifies: REQ-d9g6nt, LLR-ag9mgm
#[tokio::test(flavor = "multi_thread")]
async fn readmission_with_same_keys_gets_a_fresh_member_id() {
    let mut s = setup("readmit-same-keys").await;
    let admin_id = s.svc_a.list_orgs()[0].trie_members[0].id;

    let mut ids: Vec<MemberId> = Vec::new();
    for round in 0..3 {
        // Admit the same joiner.
        let (addr, sink) = spawn_recv_one(rand::random()).await;
        let id = admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, addr)
            .await
            .unwrap_or_else(|e| panic!("admission round {round} with the same keys failed: {e:?}"));
        sink.await.unwrap();

        assert_ne!(id.as_bytes(), s.joiner_b.member_key.as_bytes(), "round {round}: id is B's member key");
        assert_ne!(id.as_bytes(), s.joiner_b.device_key.as_bytes(), "round {round}: id is B's device key");
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
        revoke(&mut s.svc_a, &s.chain, s.org_id, id, Some(addr))
            .await
            .expect("revoke_member(B) failed");
        sink.await.unwrap();
        assert!(
            !s.svc_a.list_orgs()[0].trie_members.iter().any(|m| m.id == id),
            "round {round}: revoked id still in A's record"
        );
    }
}

// LLR-38e2kn and LLR-j6j95z: a first admission is attempted only from
// Organisation information, whose snapshot is required. A's genuine admission
// of B, relabelled by a relay as a revocation (its Envelope alone), is
// refused before the expectations are consulted or the chain is read: no
// record, the expectation kept, B's Persona untouched, nothing written. The
// genuine message then commits.
// *Rewritten 2026-10-06 (change worktree-org-node-org-key-pair, T4).* Was
// `first_admission_without_a_record_snapshot_is_refused`, of a message whose
// optional snapshot was stripped.
// *Rewritten 2026-10-07 (S3 T9).* A revocation holds a notice, not an
// Envelope, so the relay's message is a notice about the Organisation B
// expects but holds no record of; the refusal is the same.
// verifies: REQ-d9g6nt, REQ-vxqc5g, LLR-j6j95z, LLR-38e2kn
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_about_an_organisation_not_held_is_refused_before_the_chain() {
    let (mut s, counting) = setup_counted("no-snapshot").await;
    let joiner = s.joiner_b.clone();
    let genuine = captured_admission(&mut s, &joiner).await;
    let some_member = MemberId::new([0x17; 32]);
    let b_device = s.b_device_kp.device_key().unwrap();
    let relabelled = WireMessage::Revocation(org_node::revocation::RevocationNotice {
        org_id: s.org_id,
        member_id: some_member,
        device: b_device,
        proof: trie_without(some_member, b_device).prove_absent(&some_member, &b_device).unwrap(),
    });
    let on_disk = store_bytes("no-snapshot", "b");
    let reads = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &relabelled).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::RevocationNotHeld { org_id: s.org_id });
    assert_eq!(counting.reads(), reads, "no chain read");
    assert!(svc_b.list_orgs().is_empty(), "no record");
    assert_eq!(svc_b.expected_admissions().len(), 1, "the expectation is kept");
    let persona_b = persona_of(&svc_b, &s.pid_b);
    assert_eq!((persona_b.status, persona_b.org_id), (PersonaStatus::Proposed, None));
    assert_eq!(store_bytes("no-snapshot", "b"), on_disk, "nothing written");
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(b_addr, &genuine).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.expect("the genuine admission commits").org_id, s.org_id);
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

// The revoking node's own side of a revocation. Deleting `self.store.save`
// from `revoke_member` left the entire gate green: A's record of the removal
// never had to reach the disk, so a node that restarted recovered
// the member it had just removed on chain. Since T11 of the chain-authority
// change the removal reaches the record through `commit_update`; LLR-6dc598,
// LLR-tax3pm and LLR-pw369n are verified in commit_paths and in
// `a_loopback_revocation_without_an_address_burns_no_epoch`.
// verifies: LLR-qg9utu, LLR-8hdu9x, LLR-6ymd6d
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_reaches_the_administrators_disk() {
    let mut s = admit_b_directly(setup("revoke-persisted").await).await;
    let before = s.svc_a.list_orgs()[0].clone();
    assert_eq!(before.trie_members.len(), 2, "A + B before the revocation");
    let b_id = id_by_handle(&before, "bob");

    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, Some(sink_addr))
        .await
        .expect("revoke_member failed");
    let (_sink_ep, _sender, msg) = sink.await.unwrap();

    // LLR-8hdu9x: the Device the removal removed is sent its notice and
    // nothing else — no Envelope, no snapshot, no key (amended in S3).
    let b_device = s.b_device_kp.device_key().unwrap();
    let WireMessage::Revocation(notice) = msg else { panic!("a revocation, got {msg:?}") };
    assert_eq!((notice.org_id, notice.member_id, notice.device), (s.org_id, b_id, b_device));

    // LLR-tax3pm's sequence clause: the Sequence number is the epoch the
    // revocation's chain update produced, past the record's last. Since S3
    // the removed Device receives no Envelope, so it is read off A's record.
    let committed_seq = s.svc_a.list_orgs()[0].last_seq;
    assert_eq!(
        committed_seq,
        SequenceNumber::new(s.chain.get(&s.org_id).unwrap().epoch.get()),
        "the revocation's mark is the epoch its chain update produced"
    );
    assert!(committed_seq > before.last_seq, "past the record's last mark");

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
// not happened. It is also LLR-pt32fx's acceptance branch: the message B
// receives is a Revocation (its Device is no longer listed, LLR-8hdu9x), and
// one that removes this node's own Device is accepted: record deleted,
// Persona Revoked.
// verifies: REQ-uxv2x2, LLR-6p4pj2, LLR-pt32fx
#[tokio::test(flavor = "multi_thread")]
async fn a_self_delete_reaches_the_revoked_nodes_disk() {
    let mut s = admit_b_directly(setup("selfdel-persisted").await).await;
    let b_member_id = s.svc_b.list_personas()[0].member_id.expect("B has a member id");
    assert_eq!(s.svc_b.list_orgs().len(), 1, "B holds the record before the revocation");

    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_member_id, Some(b_addr))
        .await
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
    // S3 T9: the Persona is deleted with its keys (was: marked Revoked).
    assert!(svc_b.list_personas().is_empty(), "B's Persona is gone in memory");

    let reloaded = reopen_store("selfdel-persisted", "b", "pw_b");
    assert!(
        reloaded.data().orgs.is_empty(),
        "the deletion must reach the disk — a restart must not recover the record"
    );
    assert!(
        reloaded.data().personas.is_empty(),
        "the Persona's deletion must reach the disk too"
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
    let jr_c = joiner_for_c(&mut s.svc_a);
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    let c_id = admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, b_addr)
        .await
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
    assert_eq!(in_memory.trie_members.len(), 3, "A + B + C");

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

// LLR-ghja3x's second clause. Review round 2 found that changing
// `let parent_seq = last_seq + 1` to `+ 2` in `admit_member` left the whole
// gate green: both sides of the exchange take the mark from the same envelope,
// and `SeqGuard` only requires strict increase, so nothing compared the
// envelope's number against the administrator's record. This captures the
// envelope A actually sends and compares it against the mark A held before.
//
// *Amended 2026-10-05 (review round 1, finding-1):* the Sequence number is
// the epoch the admission's chain update produced (LLR-ghja3x, REQ-txvtm9),
// no longer one past the record's last. Master's genesis record was at
// epoch 1 with mark 0, where the two rules gave different numbers (2 against
// 1); since T9 of the chain-authority change the mark is 1 at epoch 1
// (Decision 16), and the Sequence number is 2 either way.
// verifies: LLR-ghja3x, REQ-txvtm9
#[tokio::test(flavor = "multi_thread")]
async fn the_admission_envelope_carries_the_epoch_its_update_produced() {
    let mut s = setup("envelope-mark").await;
    let before = s.svc_a.list_orgs()[0].clone();

    // A "admits B" into a sink, which hands back the message A transmitted.
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, sink_addr)
        .await
        .expect("admit_member failed");
    let (_sink_ep, _sender, msg) = sink.await.unwrap();

    assert_eq!(
        envelope_of(&msg).parent_seq,
        SequenceNumber::new(s.chain.get(&s.org_id).unwrap().epoch.get()),
        "the envelope must carry the epoch its chain update produced"
    );
    assert_eq!(envelope_of(&msg).parent_seq, SequenceNumber::new(before.epoch.get() + 1));
    assert!(envelope_of(&msg).parent_seq > before.last_seq, "past the record's last mark");
    assert_eq!(envelope_of(&msg).org_id, s.org_id);

    // Master asserted here that the envelope is signed by the administrator's
    // MEMBER key and not its DevicePublicKey (LLR-ghja3x's first clause). The
    // Envelope carries no signature on this branch (REQ-ag6kqm), and the record
    // names no administrator key (T8 of the chain-authority change), so neither
    // is asserted.

    // A's record then carries that same mark.
    assert_eq!(s.svc_a.list_orgs()[0].last_seq, envelope_of(&msg).parent_seq);
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

    admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, full_addr)
        .await
        .expect("admit_member to the full address failed");

    // It arrived at the endpoint that address named, and it is the admission.
    let (ep, _sender, msg) = sink.await.unwrap();
    assert_eq!(
        ep.inner().id(),
        dialled_id,
        "the admission reached an endpoint other than the one the address named"
    );
    assert!(matches!(msg, WireMessage::OrgInformation { .. }), "Organisation information, not a revocation");
    assert_eq!(envelope_of(&msg).org_id, s.org_id);
}

// The revocation twin of `a_failed_send_leaves_the_committed_record_in_place`.
// Round 1 found that clause unevidenced on the admission path and closed it
// there; round 2 found that `revoke_member` — the same shape, the same
// ordering, the same ~120 lines — was refined by no low-level requirement at
// all. Writing the twin is the point: a lesson learned on one side of a seam
// is not evidence about the other.
//
// Rewritten in T11 of the chain-authority change: the record follows the
// chain, so a removal the chain carries is committed before the send, and a
// send that fails afterwards undoes nothing.
// verifies: LLR-qg9utu, LLR-t4znbk
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_revocation_send_leaves_the_committed_removal_in_place() {
    let mut s = admit_b_directly(setup("revoke-send-fails").await).await;
    let before = s.svc_a.list_orgs()[0].clone();
    let epoch_before = s.chain.get(&s.org_id).unwrap().epoch;
    let b_id = id_by_handle(&before, "bob");

    // A well-formed identity with no transports: the dial fails at once rather
    // than waiting on the network, which a loopback port with no listener does
    // not do (QUIC retries it for thirty seconds).
    let result = revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, Some(dead_addr([0x7fu8; 32]))).await;
    assert!(result.is_err(), "a revocation send to a peer that is not listening must fail");

    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(epoch_before.get() + 1), "the chain carries the removal");
    let after = s.svc_a.list_orgs()[0].clone();
    assert_eq!(after.epoch, Epoch::new(before.epoch.get() + 1), "A's record follows the chain");
    assert!(!after.trie_members.iter().any(|m| m.id == b_id), "the removal is committed in A's record");

    let reloaded = reopen_store("revoke-send-fails", "a", "pw_a");
    assert!(
        !reloaded.data().orgs[0].trie_members.iter().any(|m| m.id == b_id),
        "and on A's disk"
    );
    assert_eq!(reloaded.data().orgs[0].epoch, after.epoch);
}

// LLR-437fvx: org-node hands the app a Persona's two public keys, each from
// its own seed, and no dialling address; an unknown Persona is refused and
// nothing is written. The absence half (no Invite, no Join request, no
// address) is `org_node_holds_no_invitation_exchange` in tests/absences.rs.
// verifies: LLR-437fvx, LLR-ctzkv7
#[tokio::test(flavor = "multi_thread")]
async fn persona_public_keys_returns_the_personas_two_keys() {
    let s = setup("persona-keys").await;
    let persona_b = persona_of(&s.svc_b, &s.pid_b);

    let (member_key, device_key) = s.svc_b.persona_public_keys(&s.pid_b).expect("B's own Persona");
    assert_eq!(
        (member_key, device_key),
        (
            persona_b.member_seed_for_test().x25519_keypair().member_key().unwrap(),
            persona_b.device_seed_for_test().signing_keypair().device_key().unwrap()
        ),
        "each key from the Persona's own seed"
    );
    assert_ne!(member_key.as_bytes(), device_key.as_bytes(), "the two keys must be distinct");

    // Abnormal: a Persona this node does not hold is refused, and nothing is
    // written.
    let before = store_bytes("persona-keys", "b");
    assert!(s.svc_b.persona_public_keys(&PersonaId::new("nobody".into())).is_err());
    assert_eq!(store_bytes("persona-keys", "b"), before, "the refusal writes nothing");
}

// Normal: the Member admitted carries exactly the Joiner's five values.
// verifies: LLR-kkj64b, REQ-qn2erx
#[tokio::test(flavor = "multi_thread")]
async fn an_admitted_member_carries_exactly_the_joiners_values() {
    let s = admit_b_directly(setup("joiner-values").await).await;
    let rec = rec_of(&s.svc_a, s.org_id);
    let bob = rec.trie_members.iter().find(|m| m.handle == s.joiner_b.handle).expect("bob");
    assert_eq!(
        (&bob.name, &bob.surname, bob.member_key, bob.device_keys.as_slice()),
        (&s.joiner_b.name, &s.joiner_b.surname, s.joiner_b.member_key, &[s.joiner_b.device_key][..])
    );
}

// Abnormal: a Joiner whose member key the Organisation already holds is
// refused by the trie and nothing is admitted. (A duplicate key is
// org-members' `DuplicateKey` refusal — every key held once, the owner's
// ruling of 2026-10-04 — which holds whatever the handle.)
// verifies: LLR-kkj64b
#[tokio::test(flavor = "multi_thread")]
async fn a_joiner_with_a_member_key_already_held_is_refused() {
    let mut s = setup("joiner-dup").await;
    let (a_member_key, _) = s.svc_a.persona_public_keys(&s.pid_a).unwrap();
    let dup = org_node::Joiner { member_key: a_member_key, ..s.joiner_b.clone() };
    let before = rec_of(&s.svc_a, s.org_id);
    let (sink_addr, _sink) = spawn_recv_one(rand::random()).await;
    let err = admit(&mut s.svc_a, &s.chain, s.org_id, &dup, sink_addr).await.unwrap_err();
    assert!(matches!(err, OrgNodeError::Trie(_)), "{err:?}");
    assert_eq!(rec_of(&s.svc_a, s.org_id).trie_members.len(), before.trie_members.len());
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
// The failure that makes it matter: a node holding two Organisations calls
// `admit_member(org_2, …)`, and the joiner is minted into org_1's record and
// the update built on org_1's root, sent from org_1's Persona. Nothing in
// this unit said otherwise.
//
// This test is the whole class in one fixture: two personas, two
// Organisations, one admission into the second. Each of the four lookups is
// reddened by it.
// ---------------------------------------------------------------------------

// verifies: LLR-vdyu65, LLR-w3fhhg
#[tokio::test(flavor = "multi_thread")]
async fn a_second_organisation_is_admitted_into_without_touching_the_first() {
    let chain = ChainSlots::new();
    let mut svc_a = Node::new(open_store("two-orgs-admit", "a", "pw_a"), chain.clone());
    let mut svc_b = Node::new(open_store("two-orgs-admit", "b", "pw_b"), chain.clone());

    // TWO personas, so org_2's founder is not the first persona in the
    // store. The first Persona bound to org_2 is the second.
    let pid_1 = svc_a.create_persona(&mut OsRng, h("first"), nm("First"), sn("Admin")).unwrap();
    let pid_2 = svc_a.create_persona(&mut OsRng, h("second"), nm("Second"), sn("Admin")).unwrap();
    let org_1 = found(&mut svc_a, &chain, &pid_1).await;
    let org_2 = found(&mut svc_a, &chain, &pid_2).await;
    assert_ne!(org_1, org_2);

    // `commit_genesis` binds each persona to the Organisation it founded,
    // which is `update_persona_status`'s job.
    assert_eq!(persona_of(&svc_a, &pid_1).org_id, Some(org_1), "persona 1 belongs to org 1");
    assert_eq!(persona_of(&svc_a, &pid_2).org_id, Some(org_2), "persona 2 belongs to org 2");

    let one_before = rec_of(&svc_a, org_1);
    let two_before = rec_of(&svc_a, org_2);
    assert_ne!(
        one_before.trie_members[0].member_key, two_before.trie_members[0].member_key,
        "two personas must give the two Organisations different founding Members"
    );

    // A's endpoint, bound from persona 2's device seed — the founder of the
    // Organisation being admitted into.
    let ep_a = OrgEndpoint::bind(&device_kp(&svc_a, &pid_2)).await.unwrap();
    let mut svc_a = svc_a.with_endpoint(ep_a);

    // B expects org_2's admission; A admits B as B's joiner.
    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    let jr_b = joiner_of(&svc_b, &pid_b);
    let b_device_kp = device_kp(&svc_b, &pid_b);
    svc_b.expect_admission(&mut OsRng, org_2).unwrap();

    // Building an admission into org_2 keeps it under org_2 alone: org_1's
    // provisional updates and record are untouched (LLR-vdyu65).
    let other = org_node::Joiner {
        handle: h("dora"),
        member_key: org_node::test_fixtures::member_key(0x71),
        device_key: org_node::test_fixtures::device_key(0x72),
        ..jr_b.clone()
    };
    let built = svc_a.admit_member(&mut OsRng, org_2, &other).unwrap();
    assert_eq!(built.org_id, Some(org_2));
    assert_eq!(svc_a.provisional_updates(org_2), vec![built]);
    assert!(svc_a.provisional_updates(org_1).is_empty(), "nothing kept under org 1");
    let one_now = rec_of(&svc_a, org_1);
    assert_eq!(
        (one_now.root_hash, one_now.epoch, one_now.last_seq, one_now.trie_members.len()),
        (one_before.root_hash, one_before.epoch, one_before.last_seq, one_before.trie_members.len()),
        "org 1's record is untouched by a build into org 2"
    );

    // Admit B into org_2. B verifies the envelope against org_2's on-chain
    // state, so an envelope built on org_1's record is refused.
    let (b_addr, b_task) = spawn_receive(svc_b, &b_device_kp).await;
    let new_member = admit(&mut svc_a, &chain, org_2, &jr_b, b_addr)
        .await
        .expect("admit_member(org 2) failed");
    let (svc_b, outcome) = b_task.await.unwrap();
    let outcome = outcome.expect("B must verify an admission into org 2");
    assert_eq!(svc_b.list_orgs()[0].org_id, org_2, "B committed the wrong Organisation");

    // org_2 moved: on chain, in A's record, and in B's.
    let two_after = rec_of(&svc_a, org_2);
    assert_eq!(two_after.epoch, Epoch::new(two_before.epoch.get() + 1), "org 2's epoch must advance");
    assert_eq!(two_after.trie_members.len(), 2, "org 2 holds its founder and B");
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
        "org 1 must still hold its founder alone"
    );
    assert!(
        !one_after.trie_members.iter().any(|m| m.id == new_member),
        "the joiner was minted into the Organisation the caller did not name"
    );
    assert_eq!(
        chain.get(&org_1).unwrap().epoch,
        one_before.epoch,
        "org 1's root must not have moved on chain"
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

// LLR-pw369n's refusal half and PR-b9wab3's reproduction. In Loopback mode
// there is no discovery to fall back on, so a send offered no peer address is
// refused with a typed error rather than attempted and lost.
//
// PR-b9wab3: `revoke_member` used to write the chain BEFORE checking for the
// address, so a missing address cost an on-chain epoch while A's record kept
// the member. The pin was inverted in T11 of the chain-authority change (red
// against that code: the chain at epoch 3, not 2). Now `revoke_member` writes
// nothing; the caller's one chain write is followed by `commit_update`, and
// the address is checked by `send_update`, after a commit the chain already
// agrees with — so no epoch can be burned.
// verifies: LLR-pw369n, LLR-6dc598, PR-b9wab3
#[tokio::test(flavor = "multi_thread")]
async fn a_loopback_revocation_without_an_address_burns_no_epoch() {
    let mut s = admit_b_directly(setup("revoke-no-addr").await).await;
    let before = s.svc_a.list_orgs()[0].clone();
    let epoch_before = s.chain.get(&s.org_id).unwrap().epoch;
    let b_id = id_by_handle(&before, "bob");

    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    assert_eq!(
        s.chain.get(&s.org_id).unwrap().epoch,
        epoch_before,
        "PR-b9wab3: building a revocation must not have moved the chain"
    );

    // The app's chain write and the commit: each moves by exactly one.
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, before.epoch).unwrap();
    let out = s.svc_a.commit_update(&mut OsRng, s.org_id).await.unwrap();
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(epoch_before.get() + 1));
    let committed = s.svc_a.list_orgs()[0].clone();
    assert_eq!(committed.epoch, Epoch::new(before.epoch.get() + 1), "A's record follows the chain");
    assert!(!committed.trie_members.iter().any(|m| m.id == b_id), "the removal is committed");

    // The recipient's own identity listens; the send without an address is
    // refused, nothing reaches it, and nothing else moves.
    let sink = OrgEndpoint::bind(&s.b_device_kp).await.unwrap();
    let result = s.svc_a.send_update(&out, s.joiner_b.device_key, None).await;
    match &result {
        Err(OrgNodeError::Chain(m)) if m.contains("requires the peer's EndpointAddr") => {}
        other => panic!("expected a refusal naming the missing address, got {other:?}"),
    }
    assert_eq!(
        s.chain.get(&s.org_id).unwrap().epoch,
        Epoch::new(epoch_before.get() + 1),
        "the caller's one write, nothing more"
    );
    let after = s.svc_a.list_orgs()[0].clone();
    assert_eq!((after.epoch, after.root_hash), (committed.epoch, committed.root_hash), "the commit stands");
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(2), sink.recv_one()).await.is_err(),
        "nothing is sent"
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
// Organisations through two Personas, with both expectations declared before
// either admission so the expectation lookup has something to get wrong.
// ---------------------------------------------------------------------------

/// A receiver holding two Organisations. Two independent founding nodes
/// share one chain; B joins both. Returns everything the tests below need.
struct TwoOrgReceiver {
    chain: ChainSlots,
    svc_a1: Node,
    svc_a2: Node,
    svc_b: Node,
    org_1: OrgId,
    org_2: OrgId,
    pid_b1: PersonaId,
    pid_b2: PersonaId,
    b1_device_kp: SigningKeypair,
    b2_device_kp: SigningKeypair,
}

async fn two_org_receiver(tag: &str) -> TwoOrgReceiver {
    let chain = ChainSlots::new();

    // Two founding nodes, each with its own store and its own Organisation.
    // They are separate services so that each can hold its own endpoint; what
    // is under test here is the RECEIVER's lookups, not the sender's.
    let mut svc_a1 = Node::new(open_store(tag, "a1", "pw_a1"), chain.clone());
    let mut svc_a2 = Node::new(open_store(tag, "a2", "pw_a2"), chain.clone());
    let pid_a1 = svc_a1.create_persona(&mut OsRng, h("admin1"), nm("Admin"), sn("One")).unwrap();
    let pid_a2 = svc_a2.create_persona(&mut OsRng, h("admin2"), nm("Admin"), sn("Two")).unwrap();
    let org_1 = found(&mut svc_a1, &chain, &pid_a1).await;
    let org_2 = found(&mut svc_a2, &chain, &pid_a2).await;
    assert_ne!(org_1, org_2);
    let a1_dev = device_kp(&svc_a1, &pid_a1);
    let a2_dev = device_kp(&svc_a2, &pid_a2);
    let mut svc_a1 = svc_a1.with_endpoint(OrgEndpoint::bind(&a1_dev).await.unwrap());
    let mut svc_a2 = svc_a2.with_endpoint(OrgEndpoint::bind(&a2_dev).await.unwrap());

    // B: two Personas, one per Organisation.
    let mut svc_b = Node::new(open_store(tag, "b", "pw_b"), chain.clone());
    let pid_b1 = svc_b.create_persona(&mut OsRng, h("bob-one"), nm("Bob"), sn("One")).unwrap();
    let pid_b2 = svc_b.create_persona(&mut OsRng, h("bob-two"), nm("Bob"), sn("Two")).unwrap();

    // **Order matters.** org 2's expectation is declared FIRST, so a lookup
    // that takes "whichever expectation comes first" rather than the one for
    // this Organisation gets the admission that follows wrong.
    let b1_device_kp = device_kp(&svc_b, &pid_b1);
    let b2_device_kp = device_kp(&svc_b, &pid_b2);
    svc_b.expect_admission(&mut OsRng, org_2).unwrap();
    svc_b.expect_admission(&mut OsRng, org_1).unwrap();
    assert_eq!(svc_b.expected_admissions().len(), 2, "two expectations, one each");
    assert_eq!(svc_b.expected_admissions()[0].org_id, org_2);

    // First admission into org 1, with org 2's expectation sitting ahead of it.
    let jr_b1 = joiner_of(&svc_b, &pid_b1);
    let (addr, task) = spawn_receive(svc_b, &b1_device_kp).await;
    admit(&mut svc_a1, &chain, org_1, &jr_b1, addr)
        .await
        .expect("admit b1 failed");
    let (svc_b, outcome) = task.await.unwrap();
    outcome.expect("B must accept its first admission to org 1");
    assert_eq!(svc_b.expected_admissions().len(), 1, "org 1's expectation and only that one is cleared");
    assert_eq!(svc_b.expected_admissions()[0].org_id, org_2, "org 2's expectation survives");

    // First admission into org 2.
    let jr_b2 = joiner_of(&svc_b, &pid_b2);
    let (addr, task) = spawn_receive(svc_b, &b2_device_kp).await;
    admit(&mut svc_a2, &chain, org_2, &jr_b2, addr)
        .await
        .expect("admit b2 failed");
    let (svc_b, outcome) = task.await.unwrap();
    outcome.expect("B must accept its first admission to org 2");
    assert_eq!(svc_b.list_orgs().len(), 2, "B now holds two Organisations");
    assert!(svc_b.expected_admissions().is_empty(), "both expectations are cleared");

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
// two expectations are held, one each) and LLR-e5c9ud's binding of the
// admitted Persona to the change's Organisation (b1 to org 1, b2 to org 2):
// the author's trace check after review round 7 found both falsified only by
// this test, which did not name them.
// verifies: LLR-cja9zv, LLR-q8emds, LLR-y2v8v2, LLR-9zfnmb, LLR-e5c9ud
#[tokio::test(flavor = "multi_thread")]
async fn a_receiver_holding_two_organisations_commits_into_the_one_the_change_names() {
    let mut s = two_org_receiver("two-org-recv").await;
    let one_before = rec_of(&s.svc_b, s.org_1);
    let two_before = rec_of(&s.svc_b, s.org_2);
    assert_ne!(one_before.org_private_key_for_test().clone(), two_before.org_private_key_for_test().clone(), "two Organisations, two keys");

    // A2 admits C into org 2, and B receives it.
    let pid_c = s.svc_a2.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let jr_c = joiner_of(&s.svc_a2, &pid_c);
    let (addr, task) = spawn_receive(s.svc_b, &s.b2_device_kp).await;
    let c_id = admit(&mut s.svc_a2, &s.chain, s.org_2, &jr_c, addr)
        .await
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
        one_after.org_private_key_for_test().clone(), one_before.org_private_key_for_test().clone(),
        "org 1's Organisation private key must not be overwritten by another Organisation's"
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
    assert_eq!(on_disk(s.org_1).org_private_key_for_test().clone(), one_before.org_private_key_for_test().clone());

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

    let (addr, task) = spawn_self_delete(s.svc_b, &s.b2_device_kp).await;
    revoke(&mut s.svc_a2, &s.chain, s.org_2, b2_member_id, Some(addr))
        .await
        .expect("revoke b2 failed");

    let (svc_b, outcome) = task.await.unwrap();
    assert!(
        matches!(
            outcome.expect("B must act on its own revocation from org 2"),
            SelfDeleteOutcome::SelfDeleted { org_id, .. } if org_id == s.org_2
        ),
        "B must self-delete from org 2 and say so"
    );

    // org 2 is gone; **org 1 is not**.
    assert_eq!(svc_b.list_orgs().len(), 1, "exactly one Organisation must remain");
    let one_after = rec_of(&svc_b, s.org_1);
    assert_eq!(one_after.epoch, one_before.epoch, "org 1's record must be untouched");
    assert_eq!(one_after.org_private_key_for_test().clone(), one_before.org_private_key_for_test().clone(), "org 1's key must survive");
    assert_eq!(one_after.trie_members.len(), one_before.trie_members.len());

    // The Persona bound to org 2 is deleted; the one bound to org 1 is not
    // (S3 T9: was "revoked").
    assert!(svc_b.list_personas().iter().all(|p| p.persona_id != s.pid_b2), "b2 is deleted");
    assert_eq!(
        persona_of(&svc_b, &s.pid_b1).status,
        PersonaStatus::Active,
        "b1 is in another Organisation and must not be deleted by org 2's removal"
    );

    let reloaded = reopen_store("two-org-selfdel", "b", "pw_b");
    assert_eq!(reloaded.data().personas.len(), 1, "only b1 may remain on disk");
    assert_eq!(reloaded.data().personas[0].persona_id, s.pid_b1);
    assert_eq!(reloaded.data().orgs.len(), 1, "only org 1 may remain on disk");
    assert_eq!(reloaded.data().orgs[0].org_id, s.org_1);
    assert_eq!(reloaded.data().orgs[0].org_private_key_for_test().clone(), one_before.org_private_key_for_test().clone());
}

// The two lookups on the self-delete path that the test above leaves green,
// found by probing the other nine of review round 4's thirteen rather than the
// four the round showed: the record the "still a member" branch commits into,
// and which Personas decide whether the node is still a member at all.
//
// Both need org 2's record to hold a device key belonging to B's **org 1**
// Persona. So org 2's founder enrols b1 as well — B
// receives that change through the self-delete path, which is the first
// half — and then revokes b2. A "still a member" test that consults every
// Persona rather than org 2's finds b1's device in org 2's trie and keeps B in
// an Organisation it was just removed from: any Member who can admit could
// pin another in place by enrolling a key that member uses elsewhere.
// verifies: LLR-jsx922, LLR-6p4pj2, LLR-jwhzh3
#[tokio::test(flavor = "multi_thread")]
async fn membership_of_one_organisation_is_judged_by_that_organisations_personas_alone() {
    let mut s = two_org_receiver("two-org-mine").await;
    let one_before = rec_of(&s.svc_b, s.org_1);
    let two_before = rec_of(&s.svc_b, s.org_2);

    // Half 1 — an ordinary update arriving on the self-delete path. org 2's
    // founder admits b1's device key into org 2.
    let jr_b1 = joiner_of(&s.svc_b, &s.pid_b1);
    let (addr, task) = spawn_self_delete(s.svc_b, &s.b2_device_kp).await;
    admit(&mut s.svc_a2, &s.chain, s.org_2, &jr_b1, addr)
        .await
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
    let (addr, task) = spawn_self_delete(svc_b, &s.b2_device_kp).await;
    revoke(&mut s.svc_a2, &s.chain, s.org_2, b2_member_id, Some(addr))
        .await
        .expect("revoke b2 failed");
    let (svc_b, outcome) = task.await.unwrap();
    assert!(
        matches!(
            outcome.expect("B must act on its own revocation from org 2"),
            SelfDeleteOutcome::SelfDeleted { org_id, .. } if org_id == s.org_2
        ),
        "b2 was B's membership of org 2; another Organisation's Persona must not keep B in it"
    );
    assert_eq!(svc_b.list_orgs().len(), 1, "only org 1 may remain");
    assert_eq!(svc_b.list_orgs()[0].org_id, s.org_1);
}

// ---------------------------------------------------------------------------
// Review round 5.
// ---------------------------------------------------------------------------

// LLR-u6rq4s as amended 2026-10-06: the delivering device is compared with no
// Membership record. A removes C and sends the Organisation information about
// it to C's endpoint — addressed to B's Device, which the committed record
// lists — and C relays it to B on the ordinary receive path; C is in B's
// record before the change and not after it, and B commits the change because
// it matches the chain. (A revocation C relayed would be refused by B, whose
// Device is still listed: `a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths`.)
// Since the owner rulings of 2026-10-07 (R1, LLR-2r2fha) this is also the
// member-sender rule's "listed before, absent after" normal case: C is
// listed in B's record when it relays, so the update passes the check.
// verifies: REQ-ztdza4, LLR-u6rq4s, LLR-2r2fha
#[tokio::test(flavor = "multi_thread")]
async fn a_removal_relayed_by_the_member_it_removes_is_committed() {
    let mut s = admit_b_directly(setup("relay-by-removed").await).await;

    // C: a persona on A's device whose seed the test holds. The admission is
    // pushed to B, so B's record holds C.
    let pid_c = s.svc_a.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let c_seed = *persona_of(&s.svc_a, &pid_c).device_seed_for_test().expose_secret();
    let jr_c = joiner_of(&s.svc_a, &pid_c);
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    let c_id = admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, b_addr)
        .await
        .expect("admit C failed");
    let (svc_b, out) = b_task.await.unwrap();
    out.expect("B commits C's admission");
    assert!(svc_b.list_orgs()[0].trie_members.iter().any(|m| m.id == c_id));
    let before = svc_b.list_orgs()[0].clone();

    // A revokes C and pushes the change, addressed to B's Device, to C's own
    // endpoint.
    let (c_addr, c_task) = spawn_recv_one(c_seed).await;
    revoke_and_tell(&mut s.svc_a, &s.chain, s.org_id, c_id, s.b_device_kp.device_key().unwrap(), c_addr)
        .await
        .expect("revoke C failed");
    let (c_ep, _sender, msg) = c_task.await.unwrap();
    assert!(matches!(msg, WireMessage::OrgInformation { .. }), "B's Device is listed");

    // C relays its own removal to B, on B's ordinary receive path.
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    c_ep.send(b_addr, &msg).await.expect("relay to B");
    let (svc_b, result) = b_task.await.unwrap();
    let committed = result.expect("a chain-valid removal is committed whoever relays it");
    assert!(committed.epoch > before.epoch);
    let after = svc_b.list_orgs()[0].clone();
    assert!(!after.trie_members.iter().any(|m| m.id == c_id), "C is gone from B's record");
}

// The owner's ruling on relabelling (REQ-3dsweu, LLR-pt32fx): a revocation is
// accepted only as the receiving Device's own removal. A relay takes the
// Organisation information A sends about C's admission and relabels it a
// revocation; B, whose Device the verified record still lists, refuses it on
// either receive path — its record and both keys, its Personas and its
// provisional updates unchanged, nothing written — and then commits the
// genuine message.
// It is also the abnormal case of LLR-8hdu9x's receiver clause and of
// LLR-4kh9w9's "a revocation never sets either key".
// *Rewritten 2026-10-07 (S3 T9).* A revocation holds a notice, so a relay
// can no longer relabel Organisation information; it sends B a notice
// instead. Two are refused: one naming another Member's Device
// (`RevocationNotForThisDevice`, LLR-r7zm39) and one naming B whose proof
// the chain's root, which still lists B, refuses (`RevocationProofRefused`,
// LLR-tx8ruv, LLR-jsx922 as amended).
// verifies: LLR-pt32fx, LLR-jsx922, LLR-u6rq4s, REQ-3dsweu, LLR-8hdu9x, LLR-4kh9w9, LLR-kzgjz8
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths() {
    let mut s = admit_b_directly(setup("relabelled").await).await;
    let dora = org_node::Joiner {
        handle: h("dora"),
        name: nm("Dora"),
        surname: sn("Diver"),
        member_key: org_node::test_fixtures::member_key(0x71),
        device_key: org_node::test_fixtures::device_key(0x72),
    };
    s.svc_b.admit_member(&mut OsRng, s.org_id, &dora).expect("B keeps a provisional update");
    let genuine = captured_admission_of_c(&mut s).await;
    let before = rec_of(&s.svc_b, s.org_id);
    let a_member = before.trie_members.iter().find(|m| m.handle.as_str() == "admin").expect("A in B's record").clone();
    let a_device = a_member.device_keys[0];
    let b_id = persona_of(&s.svc_b, &s.pid_b).member_id.expect("B has a member id");
    let b_device = s.b_device_kp.device_key().unwrap();
    let notice = |member_id, device| {
        WireMessage::Revocation(org_node::revocation::RevocationNotice {
            org_id: s.org_id,
            member_id,
            device,
            proof: trie_without(member_id, device).prove_absent(&member_id, &device).unwrap(),
        })
    };
    // *Rewritten 2026-10-07 (S3 T12a):* the notice naming B now reaches the
    // chain only from a Device B's record lists (A's); from the relay it is
    // refused first with `SenderNotListed` (LLR-kzgjz8). The notice naming
    // another Device is refused by `check_notice`, before the sender check,
    // whoever sends it.
    let relay_seed = [0x5bu8; 32];
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let refusals = [
        (notice(a_member.id, a_device), relay_seed, "RevocationNotForThisDevice"),
        (notice(b_id, b_device), a_seed, "RevocationProofRefused"),
        (notice(b_id, b_device), relay_seed, "SenderNotListed"),
    ];
    let pending = s.svc_b.provisional_updates(s.org_id);
    let b_binding = binding(&persona_of(&s.svc_b, &s.pid_b));
    let on_disk = store_bytes("relabelled", "b");

    let mut svc_b = s.svc_b;
    for (relabelled, sender_seed, expected) in refusals {
        let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
        deliver_from(sender_seed, addr, &relabelled).await;
        let (back, result) = task.await.unwrap();
        let refused = |result: &Result<_, OrgNodeError>| match result {
            Err(OrgNodeError::RevocationNotForThisDevice { org_id }) if *org_id == s.org_id => "RevocationNotForThisDevice",
            Err(OrgNodeError::RevocationProofRefused { org_id, .. }) if *org_id == s.org_id => "RevocationProofRefused",
            Err(OrgNodeError::SenderNotListed { org_id }) if *org_id == s.org_id => "SenderNotListed",
            _ => "something else",
        };
        assert_eq!(refused(&result.map(|_| ())), expected, "receive_and_verify");
        let (addr, task) = spawn_self_delete(back, &s.b_device_kp).await;
        deliver_from(sender_seed, addr, &relabelled).await;
        let (back, result) = task.await.unwrap();
        assert_eq!(refused(&result.map(|_| ())), expected, "receive_and_self_delete_if_revoked");
        svc_b = back;
    }

    let after = rec_of(&svc_b, s.org_id);
    assert_eq!(
        (after.epoch, after.last_seq, after.root_hash, after.org_pub_key),
        (before.epoch, before.last_seq, before.root_hash, before.org_pub_key)
    );
    assert_eq!(after.org_private_key_for_test().clone(), before.org_private_key_for_test().clone(), "the key is kept");
    assert_eq!(svc_b.provisional_updates(s.org_id), pending, "provisional updates kept");
    assert_eq!(binding(&persona_of(&svc_b, &s.pid_b)), b_binding, "the Persona as it was");
    assert_eq!(store_bytes("relabelled", "b"), on_disk, "nothing written");

    let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, addr, &genuine).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.expect("the genuine message commits").epoch, Epoch::new(3));
    assert_eq!(rec_of(&svc_b, s.org_id).trie_members.len(), 3, "A + B + C");
}

// Three things a first admission writes: the chain's Organisation public key,
// the Organisation private key the message carried — the one the admitting
// node's record holds (REQ-szq3ud) — and the admitted Persona's member id
// and status. A member's record holds the key since the change
// worktree-org-node-org-key-pair (LLR-3fwykc as amended); before it, only the
// creating node's did.
// *Renamed 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `a_first_admission_records_the_chains_key_the_secret_and_the_member`, after
// the Organisation secret this change removes.
// verifies: LLR-ckk5nz, LLR-e5c9ud, LLR-rys5nx, LLR-3fwykc, LLR-ba2ejp
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_records_the_chains_key_the_private_key_and_the_member() {
    let s = admit_b_directly(setup("first-admission-fields").await).await;
    let published = s.chain.get(&s.org_id).unwrap().org_pub_key;
    let held_by_a = rec_of(&s.svc_a, s.org_id).org_private_key_for_test().clone();
    let rec = s.svc_b.list_orgs()[0].clone();
    assert_eq!(rec.org_pub_key, published);
    assert_eq!(rec.org_private_key_for_test().clone(), held_by_a, "the key the message carried, the one A's record holds");
    assert_eq!(held_by_a.x25519_keypair().org_public_key().unwrap(), published, "the private half of the chain's key (LLR-ba2ejp)");

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
    assert_eq!(reloaded.data().orgs[0].org_pub_key, published);
    assert_eq!(reloaded.data().orgs[0].org_private_key_for_test().clone(), held_by_a);
}

// PR-xwek5e — reproduction. `revoke_member` sent no Organisation secret and
// `receive_and_verify` wrote whatever a message carried into the record, so a
// Member that received another Member's removal lost the secret while
// remaining a Member. The secret is gone: every message a node sends carries
// the Organisation private key its record holds (REQ-szq3ud), and the
// receiver stores it (REQ-ju6vn2). A removes C and tells B, still a Member.
// Red before the change (B's record held no key), green after.
// verifies: LLR-ckk5nz, REQ-szq3ud, REQ-ju6vn2, PR-xwek5e
#[tokio::test(flavor = "multi_thread")]
async fn pr_xwek5e_another_members_removal_leaves_the_receiver_holding_the_organisations_key() {
    let mut s = admit_b_directly(setup("pr-xwek5e").await).await;
    let jr_c = joiner_for_c(&mut s.svc_a);
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    let c_id = admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, b_addr).await.unwrap();
    let (svc_b, out) = b_task.await.unwrap();
    out.unwrap();

    let b_device = s.b_device_kp.device_key().unwrap();
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    revoke_and_tell(&mut s.svc_a, &s.chain, s.org_id, c_id, b_device, b_addr).await.unwrap();
    let (svc_b, out) = b_task.await.unwrap();
    out.expect("B commits C's removal");
    let held = rec_of(&svc_b, s.org_id).org_private_key_for_test().clone();
    assert_eq!(held, rec_of(&s.svc_a, s.org_id).org_private_key_for_test().clone(), "PR-xwek5e: B holds the key A's record holds");
    assert_eq!(
        held.x25519_keypair().org_public_key().unwrap(),
        s.chain.get(&s.org_id).unwrap().org_pub_key,
        "the private half of the chain's key"
    );
    let reloaded = reopen_store("pr-xwek5e", "b", "pw_b");
    assert_eq!(reloaded.data().orgs[0].org_private_key_for_test().clone(), held, "and on disk");
}

// PR-szkat6 — reproduction. The Organisation private key was held only by the
// node that created the Organisation, and no admission gave it to a Member.
// Every message a node sends now carries the key its record holds
// (REQ-szq3ud), and the admitted node stores it (REQ-ju6vn2). Red before the
// change (B's record held none), green after.
// verifies: LLR-3fwykc, REQ-szq3ud, REQ-ju6vn2, PR-szkat6
#[tokio::test(flavor = "multi_thread")]
async fn pr_szkat6_an_admitted_member_holds_the_organisation_private_key() {
    let s = admit_b_directly(setup("pr-szkat6").await).await;
    let held = rec_of(&s.svc_b, s.org_id).org_private_key_for_test().clone();
    assert_eq!(held, rec_of(&s.svc_a, s.org_id).org_private_key_for_test().clone(), "the key the creating node holds");
    assert_eq!(held.x25519_keypair().org_public_key().unwrap(), s.chain.get(&s.org_id).unwrap().org_pub_key);
}

// PR-g9u3xq — reproduction. Removing a Member did not replace the Organisation
// key pair, so a removed Device kept the key every remaining Member still
// used. Every provisional update now draws a fresh key pair once its root is
// calculated (REQ-stx9v3), the committing node takes it (REQ-jy6ybw), and the
// next Organisation information carries it. B, removed, held the key of the
// epoch before its removal and holds nothing after it; the chain's key after
// the removal is another, and C's admission brings yet another. Red before
// (the key never changed), green after.
// verifies: LLR-e2b7gv, LLR-6s785x, REQ-stx9v3, REQ-jy6ybw, PR-g9u3xq
#[tokio::test(flavor = "multi_thread")]
async fn pr_g9u3xq_a_removed_device_holds_no_key_used_after_its_removal() {
    let mut s = admit_b_directly(setup("pr-g9u3xq").await).await;
    let b_key = rec_of(&s.svc_b, s.org_id).org_private_key_for_test().clone();
    let b_id = s.svc_b.list_personas()[0].member_id.expect("B has a member id");
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, Some(b_addr)).await.unwrap();
    let (svc_b, outcome) = b_task.await.unwrap();
    s.svc_b = svc_b;
    assert!(matches!(outcome.unwrap(), SelfDeleteOutcome::SelfDeleted { .. }));
    assert!(s.svc_b.list_orgs().is_empty(), "B holds no record, so no key");
    let after_removal = s.chain.get(&s.org_id).unwrap().org_pub_key;
    assert_ne!(b_key.x25519_keypair().org_public_key().unwrap(), after_removal, "the removal published a fresh key");
    assert_eq!(
        rec_of(&s.svc_a, s.org_id).org_private_key_for_test().x25519_keypair().org_public_key().unwrap(),
        after_removal,
        "A took it"
    );
    let next = captured_admission_of_c(&mut s).await;
    let WireMessage::OrgInformation { org_private_key, .. } = &next else { panic!("Organisation information") };
    assert_ne!(org_private_key, &b_key);
    let now = s.chain.get(&s.org_id).unwrap().org_pub_key;
    assert_eq!(org_private_key.x25519_keypair().org_public_key().unwrap(), now);
    assert_ne!(now, after_removal, "and C's admission another");
}

// PR-ve9zw8 — reproduction. The Organisation secret a node stored was
// authenticated by nothing: any relay could hand a Member a secret of its
// choosing. The receipt check (RC-9cefcn) refuses an Organisation private key
// whose X25519 public half is not the chain's key. A relay substitutes the
// key in A's genuine admission of B: B refuses it after verification and
// before any commit — no record, the expectation kept, B's Persona untouched,
// nothing written — and then commits the genuine message. Red before (B
// committed the substituted key), green after.
// verifies: LLR-ba2ejp, LLR-mbjfq8, REQ-bwx7eg, PR-ve9zw8
#[tokio::test(flavor = "multi_thread")]
async fn pr_ve9zw8_a_first_admission_carrying_a_substituted_key_is_refused() {
    let mut s = setup("pr-ve9zw8-first").await;
    let joiner = s.joiner_b.clone();
    let genuine = captured_admission(&mut s, &joiner).await;
    let substituted = with_key(&genuine, OrgPrivateKey::from([0x42u8; 32]));
    let on_disk = store_bytes("pr-ve9zw8-first", "b");
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(addr, &substituted).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgKeyMismatch { org_id: s.org_id });
    assert!(svc_b.list_orgs().is_empty(), "no record");
    assert_eq!(svc_b.expected_admissions().len(), 1, "the expectation is kept");
    assert_eq!(persona_of(&svc_b, &s.pid_b).status, PersonaStatus::Proposed, "the Persona untouched");
    assert_eq!(store_bytes("pr-ve9zw8-first", "b"), on_disk, "nothing written");
    let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(addr, &genuine).await;
    let (svc_b, result) = task.await.unwrap();
    result.expect("the genuine admission commits");
    assert_eq!(
        rec_of(&svc_b, s.org_id).org_private_key_for_test().x25519_keypair().org_public_key().unwrap(),
        s.chain.get(&s.org_id).unwrap().org_pub_key
    );
}

// PR-ve9zw8 on an Organisation already held, on both receive paths: the
// substituted key is refused and B's record — root, epoch, mark and both keys —
// is unchanged, nothing written; the genuine message then commits. The
// chain's key decides only whether the message commits (LLR-37cj3n).
// It is also LLR-3fwykc's abnormal case: only a committed message replaces the key.
// *Rewritten 2026-10-07 (S3 T12a):* the messages come from A's listed Device
// (LLR-2r2fha); the substituted key from a relay B's record does not list is
// refused first, with `SenderNotListed`.
// verifies: LLR-ba2ejp, LLR-37cj3n, REQ-bwx7eg, RC-9cefcn, PR-ve9zw8, LLR-3fwykc, LLR-2r2fha
#[tokio::test(flavor = "multi_thread")]
async fn pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths() {
    let mut s = admit_b_directly(setup("pr-ve9zw8-held").await).await;
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let genuine = captured_admission_of_c(&mut s).await;
    let substituted = with_key(&genuine, OrgPrivateKey::from([0x42u8; 32]));
    let held = rec_of(&s.svc_b, s.org_id);
    let on_disk = store_bytes("pr-ve9zw8-held", "b");
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(addr, &substituted).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::SenderNotListed { org_id: s.org_id });
    let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, addr, &substituted).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgKeyMismatch { org_id: s.org_id });
    let (addr, task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, addr, &substituted).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgKeyMismatch { org_id: s.org_id });
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!(
        (after.epoch, after.last_seq, after.root_hash, after.org_pub_key),
        (held.epoch, held.last_seq, held.root_hash, held.org_pub_key)
    );
    assert_eq!(after.org_private_key_for_test().clone(), held.org_private_key_for_test().clone());
    assert_eq!(store_bytes("pr-ve9zw8-held", "b"), on_disk, "nothing written");
    let (addr, task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, addr, &genuine).await;
    let (_svc_b, result) = task.await.unwrap();
    assert!(matches!(result, Ok(SelfDeleteOutcome::UpdatedNotRevoked { .. })), "{result:?}");
}

// LLR-ckk5nz, LLR-4kh9w9: a committed Organisation-information update gives
// the record the key it carried and the chain's public key — on either receive
// path, in memory and on disk — so the record's two keys are always a pair,
// and the same pair A's record holds.
// It is also LLR-xn5pwc's normal case: a well-formed message decodes and commits.
// verifies: LLR-ckk5nz, LLR-4kh9w9, REQ-ju6vn2, LLR-xn5pwc
#[tokio::test(flavor = "multi_thread")]
async fn a_committed_update_gives_the_record_the_new_key_pair_on_both_paths() {
    let mut s = admit_b_directly(setup("new-pair").await).await;
    // B's record, in memory and on disk, holds the chain's key and A's pair.
    let (chain, org_id) = (s.chain.clone(), s.org_id);
    let check = |svc_a: &Node, svc_b: &Node, path: &str| {
        for (rec, place) in [
            (rec_of(svc_b, org_id), path),
            (disk_rec_of(&reopen_store("new-pair", "b", "pw_b"), org_id), "on disk"),
        ] {
            assert_eq!(rec.org_pub_key, chain.get(&org_id).unwrap().org_pub_key, "{place}");
            assert_eq!(rec.org_private_key_for_test().clone(), rec_of(svc_a, org_id).org_private_key_for_test().clone(), "{place}");
            assert_eq!(rec.org_private_key_for_test().x25519_keypair().org_public_key().unwrap(), rec.org_pub_key, "{place}: a pair");
        }
    };
    let jr_c = joiner_for_c(&mut s.svc_a);
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, addr).await.unwrap();
    let (svc_b, result) = task.await.unwrap();
    result.unwrap();
    check(&s.svc_a, &svc_b, "after receive_and_verify");

    let pid_d = s.svc_a.create_persona(&mut OsRng, h("dave"), nm("Dave"), sn("Diver")).unwrap();
    let jr_d = joiner_of(&s.svc_a, &pid_d);
    let (addr, task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &jr_d, addr).await.unwrap();
    let (svc_b, result) = task.await.unwrap();
    assert!(matches!(result, Ok(SelfDeleteOutcome::UpdatedNotRevoked { .. })), "{result:?}");
    check(&s.svc_a, &svc_b, "after the self-delete path");
}

// PR-mdv38y — reproduction. `receive_and_verify` picked the Persona to mark
// Active from every Persona whose device key was in the verified trie, with
// no regard to which Organisation that Persona was bound to, and rebound it.
// Org 2's founder enrols b1 (B's org 1 Persona) and B receives that on the
// ordinary path. Before the fix b1 was rebound to org 2; an ordinary org 1
// update then made B delete its org 1 record, and org 2 revoking b2 left B in
// org 2. Found by review round 5 of the architecture tooth.
// *Rewritten 2026-10-06 (R1a, REQ-yp75u9).* This pinned the defect. An update
// to a held record marks only a Persona bound to that Organisation: b1 stays
// bound to org 1 with its member id, org 1's update is committed, and org 2
// revoking b2 removes B from org 2 although b1's device is still in its trie.
// verifies: LLR-eyc4ud, REQ-yp75u9, PR-mdv38y
#[tokio::test(flavor = "multi_thread")]
async fn pr_mdv38y_an_update_enrolling_another_organisations_persona_does_not_rebind_it() {
    let mut s = two_org_receiver("pr-mdv38y").await;
    let b1_before = binding(&persona_of(&s.svc_b, &s.pid_b1));
    assert_eq!(b1_before.0, Some(s.org_1));

    // Org 2 enrols b1's device; B receives it on receive_and_verify.
    let jr_b1 = joiner_of(&s.svc_b, &s.pid_b1);
    let (addr, task) = spawn_receive(s.svc_b, &s.b2_device_kp).await;
    admit(&mut s.svc_a2, &s.chain, s.org_2, &jr_b1, addr)
        .await
        .unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert_eq!(outcome.unwrap().org_id, s.org_2);
    assert_eq!(binding(&persona_of(&svc_b, &s.pid_b1)), b1_before, "org 1's Persona is not rebound");

    // An ordinary org 1 update: B is still in org 1's trie, and keeps org 1.
    let pid_c = s.svc_a1.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let jr_c = joiner_of(&s.svc_a1, &pid_c);
    let (addr, task) = spawn_self_delete(svc_b, &s.b1_device_kp).await;
    admit(&mut s.svc_a1, &s.chain, s.org_1, &jr_c, addr)
        .await
        .unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert!(
        matches!(
            outcome.unwrap(),
            SelfDeleteOutcome::UpdatedNotRevoked { org_id } if org_id == s.org_1
        ),
        "B, still in org 1, commits org 1's update"
    );

    // Org 2 revokes b2, B's own membership: B leaves org 2.
    let b2_member_id = persona_of(&svc_b, &s.pid_b2).member_id.unwrap();
    let (addr, task) = spawn_self_delete(svc_b, &s.b2_device_kp).await;
    revoke(&mut s.svc_a2, &s.chain, s.org_2, b2_member_id, Some(addr))
        .await
        .unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert!(
        matches!(
            outcome.unwrap(),
            SelfDeleteOutcome::SelfDeleted { org_id, .. } if org_id == s.org_2
        ),
        "B, removed from org 2, deletes it"
    );
    assert!(svc_b.list_orgs().iter().all(|o| o.org_id != s.org_2));
    assert!(svc_b.list_orgs().iter().any(|o| o.org_id == s.org_1), "and keeps org 1");
}

// A first admission binds only a Persona with no binding. B's Persona b1 is
// bound to org 1; B adds b2 and expects an admission to org 2. Org 2 enrols
// b1 (the push is lost) and then b2, and B receives b2's admission: the
// record lists both devices. b1 comes first in B's store; before the fix it
// was the one bound to org 2, and b1's org 1 binding was lost. Now b2, the
// unbound one, is bound.
// verifies: LLR-eyc4ud, REQ-yp75u9
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_binds_the_unbound_persona_not_one_bound_elsewhere() {
    let mut s = admit_b_directly(setup("bind-unbound").await).await;
    let (mut a2, org_2) = second_founder(&s.chain, "bind-unbound").await;
    let b1_before = binding(&persona_of(&s.svc_b, &s.pid_b));
    let pid_b2 = s.svc_b.create_persona(&mut OsRng, h("bob-two"), nm("Bob"), sn("Two")).unwrap();
    let b2_dev = device_kp(&s.svc_b, &pid_b2);
    s.svc_b.expect_admission(&mut OsRng, org_2).unwrap();

    let (sink, _lost) = spawn_recv_one(rand::random()).await;
    admit(&mut a2, &s.chain, org_2, &s.joiner_b, sink).await.unwrap();
    let jr_b2 = joiner_of(&s.svc_b, &pid_b2);
    let (addr, task) = spawn_receive(s.svc_b, &b2_dev).await;
    let b2_id = admit(&mut a2, &s.chain, org_2, &jr_b2, addr).await.unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert_eq!(outcome.expect("b2's first admission commits").org_id, org_2);

    assert_eq!(binding(&persona_of(&svc_b, &s.pid_b)), b1_before, "b1 keeps its org 1 binding");
    let b2 = persona_of(&svc_b, &pid_b2);
    assert_eq!((b2.org_id, b2.member_id, b2.status), (Some(org_2), Some(b2_id), PersonaStatus::Active));
}

// Abnormal: a first admission whose only Persona the record lists is bound to
// another Organisation. B's one Persona is bound to org 1; B expects an
// admission to org 2 and org 2 enrols that Persona's keys. Before the fix B
// committed org 2 and rebound its Persona, losing org 1. Now B refuses it as
// not its own (LLR-3f5h7b): no record, the expectation kept, the Persona as
// it was, nothing written; and org 1 stays B's.
// verifies: LLR-eyc4ud, LLR-3f5h7b, REQ-yp75u9, PR-mdv38y
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_listing_only_a_persona_bound_elsewhere_is_refused() {
    let mut s = admit_b_directly(setup("bound-elsewhere").await).await;
    let (mut a2, org_2) = second_founder(&s.chain, "bound-elsewhere").await;
    prepare_to_join(&mut s.svc_b, org_2);
    let b_before = binding(&persona_of(&s.svc_b, &s.pid_b));
    let expectations = s.svc_b.expected_admissions().to_vec();
    let before = store_bytes("bound-elsewhere", "b");

    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    admit(&mut a2, &s.chain, org_2, &s.joiner_b, addr).await.unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert_eq!(outcome.unwrap_err(), OrgNodeError::AdmissionNotOurs { org_id: org_2 });
    assert!(svc_b.list_orgs().iter().all(|o| o.org_id != org_2), "no org 2 record");
    assert_eq!(svc_b.expected_admissions(), expectations.as_slice(), "the expectation is kept");
    assert_eq!(binding(&persona_of(&svc_b, &s.pid_b)), b_before, "the Persona is as it was");
    assert_eq!(store_bytes("bound-elsewhere", "b"), before, "nothing written");

    // Org 1 is still B's: its next update is committed as an update.
    let jr_c = joiner_for_c(&mut s.svc_a);
    let (addr, task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, addr).await.unwrap();
    let (_svc_b, outcome) = task.await.unwrap();
    assert!(matches!(
        outcome.unwrap(),
        SelfDeleteOutcome::UpdatedNotRevoked { org_id } if org_id == s.org_id
    ));
}

/// A second founding node on `chain`, its endpoint bound, and its
/// Organisation.
async fn second_founder(chain: &ChainSlots, tag: &str) -> (Node, OrgId) {
    let mut a2 = Node::new(open_store(tag, "a2", "pw_a2"), chain.clone());
    let pid = a2.create_persona(&mut OsRng, h("admin2"), nm("Admin"), sn("Two")).unwrap();
    let org_2 = found(&mut a2, chain, &pid).await;
    let dev = device_kp(&a2, &pid);
    (a2.with_endpoint(OrgEndpoint::bind(&dev).await.unwrap()), org_2)
}

/// A Persona's binding: its Organisation, member id and status.
fn binding(p: &PersonaRecord) -> (Option<OrgId>, Option<MemberId>, PersonaStatus) {
    (p.org_id, p.member_id, p.status)
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
    let chain = ChainSlots::new();
    let mut svc_a = Node::new(open_store("pr-8qsnhx", "a", "pw_a"), chain.clone());
    let pid_1 = svc_a.create_persona(&mut OsRng, h("first"), nm("First"), sn("Admin")).unwrap();
    let pid_2 = svc_a.create_persona(&mut OsRng, h("second"), nm("Second"), sn("Admin")).unwrap();
    let org_1 = found(&mut svc_a, &chain, &pid_1).await;
    let org_2 = found(&mut svc_a, &chain, &pid_2).await;

    let mut svc_b1 = Node::new(open_store("pr-8qsnhx", "b1", "pw_b1"), chain.clone());
    let mut svc_b2 = Node::new(open_store("pr-8qsnhx", "b2", "pw_b2"), chain.clone());
    let pid_b1 = svc_b1.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("One")).unwrap();
    let pid_b2 = svc_b2.create_persona(&mut OsRng, h("bea"), nm("Bea"), sn("Two")).unwrap();
    prepare_to_join(&mut svc_b1, org_1);
    prepare_to_join(&mut svc_b2, org_2);
    let jr_b1 = joiner_of(&svc_b1, &pid_b1);
    let jr_b2 = joiner_of(&svc_b2, &pid_b2);
    let b1_dev = device_kp(&svc_b1, &pid_b1);
    let b2_dev = device_kp(&svc_b2, &pid_b2);

    // The first Organisation's admission binds A's endpoint from Persona 1.
    let (addr, task) = spawn_receive(svc_b1, &b1_dev).await;
    admit(&mut svc_a, &chain, org_1, &jr_b1, addr)
        .await
        .unwrap();
    let (_svc_b1, out1) = task.await.unwrap();
    out1.expect("the first Organisation's admission is accepted");

    // The second goes out under Persona 1's device key: a relay R captures it.
    let epoch_before = chain.get(&org_2).unwrap().epoch;
    let (r_addr, r_task) = spawn_recv_one(rand::random()).await;
    admit(&mut svc_a, &chain, org_2, &jr_b2, r_addr)
        .await
        .expect("PR-8qsnhx: A reports success");
    let (r_ep, sender, msg) = r_task.await.unwrap();
    assert_eq!(
        sender,
        device_kp(&svc_a, &pid_1).device_key().unwrap(),
        "PR-8qsnhx: the second Organisation's admission goes out under Persona 1's device key"
    );
    assert_eq!(chain.get(&org_2).unwrap().epoch, Epoch::new(epoch_before.get() + 1), "the chain moved");

    // The joiner checks nothing about the sender, so it commits the admission.
    let (addr, task) = spawn_receive(svc_b2, &b2_dev).await;
    r_ep.send(addr, &msg).await.expect("relay to the joiner");
    let (svc_b2, out2) = task.await.unwrap();
    assert_eq!(out2.expect("a chain-valid first admission is committed whoever delivers it").org_id, org_2);
    assert_eq!(svc_b2.list_orgs().len(), 1, "the joiner holds the record");
}

// ---------------------------------------------------------------------------
// Review round 6. Three more pins, of the class round 5 named: state one path
// writes and another reads.
// ---------------------------------------------------------------------------

// PR-322qst reproduction. The node's OWN revocation, arriving on
// `receive_and_verify` rather than on the self-delete path, is its removal
// (REQ-uxv2x2 is not scoped to a path): B deletes its record of the
// Organisation and every provisional update for it, and its Persona is
// deleted (S3 T9; was: Revoked) — in memory and on disk. Before the fix B
// committed the removal as an ordinary update at epoch 3 and stayed Active.
// verifies: REQ-uxv2x2, LLR-b27jr6, LLR-pt32fx, PR-322qst
#[tokio::test(flavor = "multi_thread")]
async fn an_own_revocation_on_the_ordinary_path_deletes_the_record() {
    let mut s = admit_b_directly(setup("pr-322qst").await).await;
    let b_member_id = s.svc_b.list_personas()[0].member_id.expect("B has a member id");
    let carol = org_node::Joiner {
        handle: h("carol"),
        name: nm("Carol"),
        surname: sn("Coder"),
        member_key: org_node::test_fixtures::member_key(0x61),
        device_key: org_node::test_fixtures::device_key(0x62),
    };
    s.svc_b.admit_member(&mut OsRng, s.org_id, &carol).expect("B keeps a provisional update");
    assert_eq!(s.svc_b.provisional_updates(s.org_id).len(), 1);
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_member_id, Some(b_addr))
        .await
        .unwrap();
    let (svc_b, result) = b_task.await.unwrap();
    // S3 T9: B's Device receives its notice; the ordinary path accepts it,
    // returns B's acknowledgement and deletes the Persona (was: Revoked).
    let outcome = result.expect("PR-322qst: the removal verifies");
    assert_eq!(outcome.acknowledgements.len(), 1, "B's acknowledgement");
    assert_eq!(outcome.acknowledgements[0].member_id, b_member_id);
    assert!(svc_b.list_orgs().iter().all(|o| o.org_id != s.org_id), "PR-322qst: B forgets the record");
    assert!(svc_b.list_personas().iter().all(|p| p.persona_id != s.pid_b), "PR-322qst: and its Persona");
    assert!(svc_b.provisional_updates(s.org_id).is_empty(), "and its provisional updates");
    let reloaded = reopen_store("pr-322qst", "b", "pw_b");
    assert!(reloaded.data().orgs.iter().all(|o| o.org_id != s.org_id), "PR-322qst: on disk too");
    assert!(reloaded.data().provisional_updates.iter().all(|u| u.org_id != Some(s.org_id)));
    assert!(reloaded.data().personas.iter().all(|p| p.persona_id != s.pid_b), "the Persona is gone on disk");
}

// PR-mdv38y, second writer — reproduction. `commit_genesis` bound the
// founding Persona to the Organisation it founded, overwriting the binding of
// a Persona that was a member elsewhere; the self-delete path reads that
// binding, so an ordinary update from the first Organisation made B delete it
// while B was still in its trie. Found by review round 6 of the architecture
// tooth, and again by review round 1 of this change (finding-1).
// *Rewritten 2026-10-06 (R1a, REQ-yp75u9).* This pinned the defect, carrying
// LLR-w3fhhg's overwrite clause and LLR-q3aj8z's member-id clause. A Persona
// bound to one Organisation now cannot found another: the founding is refused
// with the Persona named, nothing is kept or written, and the next ordinary
// org 1 update is committed as an update.
// verifies: LLR-6z5xya, REQ-yp75u9, PR-mdv38y
#[tokio::test(flavor = "multi_thread")]
async fn pr_mdv38y_a_member_persona_cannot_found_another_organisation() {
    let mut s = admit_b_directly(setup("pr-mdv38y-found").await).await;
    let b_before = binding(&persona_of(&s.svc_b, &s.pid_b));
    assert_eq!(b_before.0, Some(s.org_id));
    assert!(b_before.1.is_some(), "B holds its org 1 member id");
    let before = store_bytes("pr-mdv38y-found", "b");

    let err = s.svc_b.create_organisation(&mut OsRng, &s.pid_b).unwrap_err();
    assert_eq!(err, OrgNodeError::PersonaAlreadyBound { persona_id: s.pid_b.clone() });
    assert!(s.svc_b.genesis_provisional_updates(&s.pid_b).is_empty(), "no genesis update kept");
    assert_eq!(binding(&persona_of(&s.svc_b, &s.pid_b)), b_before, "the Persona is as it was");
    assert_eq!(store_bytes("pr-mdv38y-found", "b"), before, "nothing written");

    let jr_c = joiner_for_c(&mut s.svc_a);
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, b_addr)
        .await
        .unwrap();
    let (svc_b, outcome) = b_task.await.unwrap();
    assert!(
        matches!(
            outcome.unwrap(),
            SelfDeleteOutcome::UpdatedNotRevoked { org_id } if org_id == s.org_id
        ),
        "B, still in org 1, commits org 1's update"
    );
    assert!(svc_b.list_orgs().iter().any(|o| o.org_id == s.org_id), "B keeps org 1");
}

// ---------------------------------------------------------------------------
// Review round 7.
// ---------------------------------------------------------------------------

// PR-u4c2vp, with the member-sender rule of the owner rulings of 2026-10-07
// (R1, LLR-2r2fha; LLR-3q63zv as amended): a rogue relay R forwards A's
// genuine admission of C to B's self-delete path; B refuses it with
// `SenderNotListed`, its epoch unchanged and no chain read. The same message
// from A's own Device is committed as an ordinary update.
// *Rewritten 2026-10-07 (S3 T12a)*: was
// `pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path`
// (owner ruling 2026-10-05: nothing about the sender is checked).
// verifies: LLR-2r2fha, LLR-3q63zv, PR-u4c2vp
#[tokio::test(flavor = "multi_thread")]
async fn pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused_on_the_self_delete_path() {
    let (s, counting) = setup_counting("pr-u4c2vp").await;
    let mut s = admit_b_directly(s).await;
    let epoch_before = s.svc_b.list_orgs()[0].epoch;

    // A admits C, but the push goes to R.
    let jr_c = joiner_for_c(&mut s.svc_a);
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, r_addr)
        .await
        .unwrap();
    let (r_ep, _sender, msg) = r_task.await.unwrap();

    // R relays it to B's self-delete path.
    let reads_before = counting.reads();
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    r_ep.send(b_addr, &msg).await.expect("relay to B");
    let (mut svc_b, outcome) = b_task.await.unwrap();
    assert_eq!(outcome.map(|_| ()), Err(OrgNodeError::SenderNotListed { org_id: s.org_id }));
    assert_eq!(svc_b.list_orgs()[0].epoch, epoch_before, "B's record did not move");
    assert_eq!(counting.reads(), reads_before, "the refusal reads no chain");

    // The same message from A's own Device.
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let outcome = deliver_from_to_self_delete(&mut svc_b, a_seed, msg).await;
    assert!(
        matches!(outcome, Ok(SelfDeleteOutcome::UpdatedNotRevoked { .. })),
        "committed as an ordinary update from a listed Device, got {outcome:?}"
    );
    assert!(svc_b.list_orgs()[0].epoch > epoch_before, "and B's record moved");
}

// A first admission to an Organisation the app expects, with NO imported
// Invite, is committed on the chain anchor alone (REQ-xa6smf, LLR-mbjfq8,
// amended 2026-10-05; REQ-8amu2a): no Invite is required, only the
// expectation. Renamed 2026-10-06 (T6) from
// `a_first_admission_with_no_imported_invite_rests_on_the_chain_alone`.
// verifies: REQ-xa6smf, REQ-8amu2a, LLR-mbjfq8
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_to_an_expected_organisation_rests_on_the_chain_alone() {
    let mut s = setup_with("no-invite-chain-alone", false).await;
    s.svc_b.expect_admission(&mut OsRng, s.org_id).unwrap();

    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, b_addr)
        .await
        .expect("admit_member(B) failed");

    let (svc_b, outcome) = b_task.await.unwrap();
    assert_eq!(outcome.expect("committed with no Invite").org_id, s.org_id);
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(persona_of(&svc_b, &s.pid_b).status, PersonaStatus::Active);
}

// LLR-xq9nrq, amended 2026-10-05 (owner answer Q1): on a first admission the
// record keeps the Organisation public key read from the chain in the same
// operation, and no administrator key (T8 removed the field). Since the
// change worktree-org-node-org-key-pair it holds the Organisation private
// key the message carried as well (REQ-ju6vn2).
// *Renamed 2026-10-06 (T8).* Was
// `a_first_admission_records_the_chains_key_as_admin_member_key`.
// verifies: LLR-xq9nrq, LLR-rys5nx
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_records_the_chains_organisation_public_key() {
    let mut s = setup_with("no-invite-admin-key", false).await;
    s.svc_b.expect_admission(&mut OsRng, s.org_id).unwrap();

    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, b_addr)
        .await
        .expect("admit_member(B) failed");

    let (svc_b, outcome) = b_task.await.unwrap();
    outcome.expect("committed with no Invite");
    let published = s.chain.get(&s.org_id).unwrap().org_pub_key;
    let rec = rec_of(&svc_b, s.org_id);
    assert_eq!(rec.org_pub_key, published, "the chain's key, with no Invite");
    assert_eq!(rec.org_private_key_for_test().clone(), rec_of(&s.svc_a, s.org_id).org_private_key_for_test().clone(), "and the key A's record holds");
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
    let (addr, task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, addr)
        .await
        .unwrap();
    let (svc_b, outcome) = task.await.unwrap();
    assert_eq!(outcome.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert!(svc_b.list_orgs().is_empty(), "nothing is written");
    assert_eq!(svc_b.expected_admissions().len(), 1, "the expectation is not cleared");
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

// LLR-q8emds: the expectation is cleared only once the first admission has
// committed (until T7 the pending invite, consumed the same way). Consuming
// it before verification was green, because every first admission a test sent either committed or was refused before the
// point a consumption could sit. This one comes from A's own
// device key and fails verification. *Merged 2026-10-05 into worktree-person-shared-types:*
// master altered the envelope's mark after signing, which failed the
// signature; the Envelope has no signature on this branch, and a raised mark
// is still fresh to a first admission, so the Change set bytes are spoiled
// instead and verification refuses them at the decode.
// verifies: LLR-q8emds
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_that_fails_verification_leaves_the_expectation() {
    let mut s = setup("invite-kept").await;
    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, r_addr)
        .await
        .unwrap();
    let (_r_ep, _sender, genuine) = r_task.await.unwrap();
    let msg = with_envelope(&genuine, org_node::Envelope { delta_bytes: vec![0xffu8; 16], ..envelope_of(&genuine).clone() });

    let pid_a = s.svc_a.list_personas()[0].persona_id.clone();
    let a_again = OrgEndpoint::bind(&device_kp(&s.svc_a, &pid_a)).await.unwrap();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, a_again.send(b_addr, &msg)).await.unwrap().unwrap();

    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(
        result.unwrap_err(),
        OrgNodeError::MalformedDelta,
        "an altered envelope must not verify"
    );
    assert!(svc_b.list_orgs().is_empty(), "nothing is committed");
    assert_eq!(svc_b.expected_admissions().len(), 1);
}

// The pure-proxy account the app hands `commit_genesis` is kept in the
// record and on disk, and handed back to the app (which makes every chain
// write) unchanged by admissions and revocations; org-node passes it to no
// chain operation, since its chain seam only reads.
// verifies: LLR-dzte8x, LLR-3v5nu9, LLR-drgdy8
#[tokio::test(flavor = "multi_thread")]
async fn the_proxy_account_from_genesis_is_kept_and_handed_back() {
    let chain = ChainSlots::new();
    let mut svc_a = Node::new(open_store("proxy", "a", "pw_a"), chain.clone());
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let update = svc_a.create_organisation(&mut OsRng, &pid_a).unwrap();
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    svc_a.commit_genesis(&mut OsRng, &pid_a, org_id, test_proxy()).await.unwrap();
    assert_eq!(svc_a.list_orgs()[0].proxy_account, Some(test_proxy()), "LLR-dzte8x: kept in the record");
    assert_eq!(svc_a.proxy_account(org_id).unwrap(), Some(test_proxy()));
    assert_eq!(
        reopen_store("proxy", "a", "pw_a").data().orgs[0].proxy_account,
        Some(test_proxy()),
        "LLR-dzte8x: and on disk"
    );

    let jr = joiner_for_c(&mut svc_a);
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    admit(&mut svc_a, &chain, org_id, &jr, sink_addr).await.unwrap();
    let _ = sink.await.unwrap();

    let c_id = id_by_handle(&svc_a.list_orgs()[0], "carol");
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    revoke(&mut svc_a, &chain, org_id, c_id, Some(sink_addr)).await.unwrap();
    let _ = sink.await.unwrap();
    assert_eq!(svc_a.proxy_account(org_id).unwrap(), Some(test_proxy()), "LLR-drgdy8: handed back unchanged");
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

// The reviewer's jam, on the ordinary receive path. A device B accepts
// (A's) rebuilds the genuine Envelope with the Sequence number
// u64::MAX. Before REQ-txvtm9 bound the number to the chain's epoch, B
// committed it, and every later genuine Envelope was StaleSeq forever. Now it
// is refused, the record and the mark stay where they were, and the genuine
// Envelope still commits with the mark at the chain's epoch.
// verifies: REQ-txvtm9, REQ-mr5abb, LLR-9f5hmr
#[tokio::test(flavor = "multi_thread")]
async fn a_sequence_number_beyond_the_chain_epoch_cannot_jam_the_receive_path() {
    let mut s = admit_b_directly(setup("jam-receive").await).await;
    let before = rec_of(&s.svc_b, s.org_id);
    // A admits C into a sink: the chain is at epoch 3, B's record still at 2.
    let genuine = captured_admission_of_c(&mut s).await;
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(3));
    let jam = with_envelope(&genuine, org_node::Envelope { parent_seq: SequenceNumber::new(u64::MAX), ..envelope_of(&genuine).clone() });

    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
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

    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
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
    // A admits C into a sink: the chain is at epoch 3, B's record still at 2.
    let genuine = captured_admission_of_c(&mut s).await;
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(3));
    let jam = with_envelope(&genuine, org_node::Envelope { parent_seq: SequenceNumber::new(u64::MAX), ..envelope_of(&genuine).clone() });

    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    s.svc_a.endpoint().expect("A endpoint").send(b_addr, &jam).await.expect("send jam");
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::SeqNotEpoch { seq: u64::MAX, epoch: 3 });
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!(after.last_seq, before.last_seq, "the mark must not move");
    assert_eq!(after.epoch, Epoch::new(2));
    assert_eq!(after.root_hash, before.root_hash);

    let (b_addr, b_task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    s.svc_a.endpoint().unwrap().send(b_addr, &genuine).await.expect("send genuine");
    let (svc_b, result) = b_task.await.unwrap();
    assert!(matches!(result, Ok(SelfDeleteOutcome::UpdatedNotRevoked { .. })), "got {result:?}");
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.last_seq), (Epoch::new(3), SequenceNumber::new(3)), "committed at the chain's epoch");
}

// The member-sender rule (owner ruling R1 of 2026-10-07, LLR-2r2fha): Change
// set bytes that do not decode, for an Organisation B holds, are refused
// from a Device B's record does not list with `SenderNotListed`, before they
// are decoded; the same bytes from A's listed Device are refused as what they
// are (`MalformedDelta`). Neither reads the chain, and B's record does not
// move.
// *Rewritten 2026-10-07 (S3 T12a)*: was
// `a_malformed_change_set_from_a_device_outside_the_record_is_refused_as_malformed`
// (REQ-ag6kqm as amended: nothing about the sender is checked).
// verifies: LLR-2r2fha, LLR-mcdh85, LLR-9sknpa
#[tokio::test(flavor = "multi_thread")]
async fn a_malformed_change_set_from_a_device_outside_the_record_is_refused_before_it_is_decoded() {
    let (s, counting) = setup_counting("known-org-pre-decode").await;
    let s = admit_b_directly(s).await;
    let before = rec_of(&s.svc_b, s.org_id);
    let envelope = org_node::Envelope { org_id: s.org_id, parent_seq: SequenceNumber::new(3), delta_bytes: vec![0xff; 16] };
    let msg = carrying(envelope);
    let reads_before = counting.reads();

    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    let rogue = OrgEndpoint::bind(&DeviceSeed::from(ROGUE_SEED).signing_keypair()).await.unwrap();
    rogue.send(b_addr, &msg).await.expect("rogue send");
    let (mut svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::SenderNotListed { org_id: s.org_id });

    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let from_a = deliver_from_to_receive(&mut svc_b, a_seed, msg).await;
    assert_eq!(from_a.unwrap_err(), OrgNodeError::MalformedDelta);

    assert_eq!(counting.reads(), reads_before, "neither refusal reads the chain");
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.last_seq, after.root_hash), (before.epoch, before.last_seq, before.root_hash));
}

// LLR-e5c9ud as amended 2026-10-05: the Persona a receive marks is the first in
// the store whose DevicePublicKey is in the verified record, whatever its
// member key. The founding node holds the founder's Persona and C's, and both
// DevicePublicKeys are in its record. A receives an update from B's device.
// The Persona marked is the founder's, which comes first in the store — the
// administrator-key exclusion that used to skip it is gone.
// *Rewritten 2026-10-06 (T8).* Was
// `the_administrators_own_persona_is_never_the_one_a_receive_marks`, which
// pinned that exclusion.
// verifies: LLR-e5c9ud
#[tokio::test(flavor = "multi_thread")]
async fn the_persona_marked_active_is_the_first_whose_device_is_in_the_record() {
    let mut s = admit_b_directly(setup("admin-excluded").await).await;
    let _ = captured_admission_of_c(&mut s).await;
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(3));
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
    let msg = WireMessage::OrgInformation {
        envelope,
        record_snapshot: snapshot_bytes(&trie_of(&rec_a)),
        org_private_key: rec_a.org_private_key_for_test().clone(),
    };

    let a_device_kp = device_kp(&s.svc_a, &admin_pid);
    let (a_addr, a_task) = spawn_receive(s.svc_a, &a_device_kp).await;
    let from_b = OrgEndpoint::bind(&s.b_device_kp).await.unwrap();
    from_b.send(a_addr, &msg).await.expect("send from B");
    let (svc_a, result) = a_task.await.unwrap();
    assert_eq!(result.expect("A commits D's admission").epoch, epoch);

    // By handle: a committed record lists its members in trie order, not in
    // the order they were admitted.
    let founder_id = id_by_handle(&rec_a, "admin");
    let founder = persona_of(&svc_a, &admin_pid);
    assert_eq!(
        (founder.status, founder.org_id, founder.member_id),
        (PersonaStatus::Active, Some(s.org_id), Some(founder_id)),
        "the first Persona whose device is in the record is the one marked"
    );
    let carol = persona_of(&svc_a, &c_pid);
    assert_eq!((carol.status, carol.member_id), (PersonaStatus::Proposed, None), "only the first is marked");
    assert_ne!(c_id, founder_id);
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
async fn a_first_admission_that_misses_the_chain_root_commits_nothing() {
    let mut s = setup_with("no-invite-root-miss", false).await;
    s.svc_b.expect_admission(&mut OsRng, s.org_id).unwrap();

    let (r_addr, r_task) = spawn_recv_one(ROGUE_SEED).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, r_addr)
        .await
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
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    tokio::time::timeout(NET, relay.send(b_addr, &msg)).await.unwrap().unwrap();

    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::RootMismatch);
    assert!(svc_b.list_orgs().is_empty(), "no record is committed");
    assert_eq!(svc_b.expected_admissions().len(), 1, "a refusal keeps the expectation");
    let persona_b = persona_of(&svc_b, &s.pid_b);
    assert_eq!(persona_b.status, PersonaStatus::Proposed, "B's Persona is untouched");
    assert_eq!(persona_b.org_id, None);
    assert_eq!(persona_b.member_id, None);
    let reloaded = reopen_store("no-invite-root-miss", "b", "pw_b");
    assert!(reloaded.data().orgs.is_empty(), "nothing reaches the disk");
    assert!(
        reloaded.data().personas.iter().all(|p| p.status == PersonaStatus::Proposed
            && p.org_id.is_none()
            && p.member_id.is_none()),
        "and nothing about a Persona reaches the disk"
    );
}

// Normal side of LLR-e5c9ud as amended: the Persona whose device the trie
// holds is bound even when its member key is the key the chain publishes —
// the administrator-key exclusion is gone. The record keeps the chain's key,
// not one from the message (LLR-xq9nrq). Since the receipt check (LLR-ba2ejp)
// the message must carry that key's private half: here B's member seed.
// verifies: LLR-e5c9ud, LLR-xq9nrq, LLR-ba2ejp
#[tokio::test(flavor = "multi_thread")]
async fn a_persona_whose_member_key_the_chain_publishes_is_still_bound() {
    let mut s = setup("published-key").await;
    let joiner = s.joiner_b.clone();
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &joiner, sink_addr).await.unwrap();
    let msg = sink.await.unwrap().2;
    let state = s.chain.get(&s.org_id).unwrap();
    let published = OrgPublicKey::parse(joiner.member_key.as_bytes()).unwrap();
    s.chain.set(s.org_id, org_node::chain::OrgState { org_pub_key: published, ..state });
    let b_seed = *persona_of(&s.svc_b, &s.pid_b).member_seed_for_test().expose_secret();
    let msg = with_key(&msg, OrgPrivateKey::from(b_seed));
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(addr, &msg).await;
    let (svc_b, result) = task.await.unwrap();
    result.unwrap();
    let p = persona_of(&svc_b, &s.pid_b);
    assert_eq!((p.status, p.org_id), (PersonaStatus::Active, Some(s.org_id)));
    assert!(p.member_id.is_some());
    assert_eq!(rec_of(&svc_b, s.org_id).org_pub_key, published);
}
