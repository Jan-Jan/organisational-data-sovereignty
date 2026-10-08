#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The receive paths run every chain-free check before they read the chain,
//! and read it at most once (REQ-f2k4tr, RC-mj6gjq; LLR-9ew26y, LLR-3wb7th,
//! LLR-379hnv), observed through a chain that counts its reads.

mod support;

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::revocation::RevocationNotice;
use org_node::service::SelfDeleteOutcome;
use org_node::test_fixtures::admit_member_delta;
use org_node::transport::wire::{encode_frame, WireMessage};
use org_node::{Envelope, Epoch, MemberSeed, SequenceNumber};
use support::*;

/// B admitted at epoch 2, B's chain reads counted.
async fn admitted(tag: &str) -> (Setup, CountingChain) {
    let (s, counting) = setup_counted(tag).await;
    (admit_b_directly(s).await, counting)
}

/// A Change set on a base B does not hold.
fn foreign_delta() -> org_members::delta::Delta {
    admit_member_delta(&MemberSeed::from([0x5a; 32]).x25519_keypair()).0
}

/// From a relay B's record does not list, `msg` is refused on both paths
/// with `SenderNotListed` and no chain read (owner ruling R1 of 2026-10-07,
/// LLR-2r2fha). The tests below send from A's listed Device otherwise
/// (*rewritten 2026-10-07, S3 T12a*: they sent from a relay).
async fn refused_from_a_relay(svc_b: &mut Node, counting: &CountingChain, org_id: OrgId, msg: &WireMessage) {
    let before = counting.reads();
    let not_listed = Err(OrgNodeError::SenderNotListed { org_id });
    assert_eq!(deliver_to_receive(svc_b, msg.clone()).await.map(|_| ()), not_listed);
    assert_eq!(deliver_to_self_delete(svc_b, msg.clone()).await.map(|_| ()), not_listed);
    assert_eq!(counting.reads(), before, "no chain read for an unlisted sender");
}
// Normal: an update that passes the chain-free checks reads the chain once.
// verifies: REQ-f2k4tr, LLR-9ew26y, LLR-2r2fha
#[tokio::test(flavor = "multi_thread")]
async fn an_update_that_passes_the_chain_free_checks_reads_the_chain_once() {
    let (mut s, counting) = admitted("reads-once").await;
    let msg = captured_admission_of_c(&mut s).await;
    refused_from_a_relay(&mut s.svc_b, &counting, s.org_id, &msg).await;
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, b_addr, &msg).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap().epoch, Epoch::new(3));
    assert_eq!(counting.reads() - before, 1);
    assert_eq!(rec_of(&svc_b, s.org_id).trie_members.len(), 3);
}

// Abnormal: a replayed envelope is refused as stale with no chain read, and
// the record — mark included — is untouched.
// verifies: REQ-f2k4tr, REQ-mr5abb, LLR-9ew26y, LLR-2r2fha
#[tokio::test(flavor = "multi_thread")]
async fn a_stale_envelope_is_refused_without_a_chain_read() {
    let (mut s, counting) = admitted("stale").await;
    let msg = captured_admission_of_c(&mut s).await;
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, b_addr, &msg).await;
    let (mut svc_b, first) = b_task.await.unwrap();
    first.unwrap();
    let held = rec_of(&svc_b, s.org_id);
    refused_from_a_relay(&mut svc_b, &counting, s.org_id, &msg).await;
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, b_addr, &msg).await;
    let (svc_b, again) = b_task.await.unwrap();
    assert!(matches!(again, Err(OrgNodeError::StaleSeq { .. })), "{again:?}");
    assert_eq!(counting.reads(), before, "no chain read for a stale envelope");
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.root_hash, after.last_seq), (held.epoch, held.root_hash, held.last_seq));
}

// Abnormal: a Change set built on another base is refused with no chain read.
// verifies: REQ-f2k4tr, LLR-9ew26y, LLR-2r2fha
#[tokio::test(flavor = "multi_thread")]
async fn a_change_set_on_another_base_is_refused_without_a_chain_read() {
    let (mut s, counting) = admitted("other-base").await;
    let mark = rec_of(&s.svc_b, s.org_id).last_seq;
    let envelope = Envelope::build(s.org_id, SequenceNumber::new(mark.get() + 1), &foreign_delta()).unwrap();
    refused_from_a_relay(&mut s.svc_b, &counting, s.org_id, &carrying(envelope.clone())).await;
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, b_addr, &carrying(envelope)).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::DeltaBaseMismatch);
    assert_eq!(counting.reads(), before);
}

// Abnormal: Change set bytes that do not decode are refused with no chain read.
// verifies: REQ-f2k4tr, LLR-9ew26y, LLR-2r2fha
#[tokio::test(flavor = "multi_thread")]
async fn undecodable_change_set_bytes_are_refused_without_a_chain_read() {
    let (mut s, counting) = admitted("undecodable").await;
    let mark = rec_of(&s.svc_b, s.org_id).last_seq;
    let envelope = Envelope { org_id: s.org_id, parent_seq: SequenceNumber::new(mark.get() + 1), delta_bytes: vec![0xff; 16] };
    refused_from_a_relay(&mut s.svc_b, &counting, s.org_id, &carrying(envelope.clone())).await;
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, b_addr, &carrying(envelope)).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::MalformedDelta);
    assert_eq!(counting.reads(), before);
}

// Abnormal: the self-delete path runs the same checks first.
// verifies: LLR-9ew26y, LLR-2r2fha
#[tokio::test(flavor = "multi_thread")]
async fn the_self_delete_path_refuses_a_stale_envelope_without_a_chain_read() {
    let (mut s, counting) = admitted("sd-stale").await;
    let msg = captured_admission_of_c(&mut s).await;
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, b_addr, &msg).await;
    let (mut svc_b, first) = b_task.await.unwrap();
    first.unwrap();
    refused_from_a_relay(&mut svc_b, &counting, s.org_id, &msg).await;
    let before = counting.reads();
    let (b_addr, b_task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, b_addr, &msg).await;
    let (_svc_b, again) = b_task.await.unwrap();
    assert!(matches!(again, Err(OrgNodeError::StaleSeq { .. })), "{again:?}");
    assert_eq!(counting.reads(), before);
}

// Abnormal: an Organisation the node holds no record of is refused on the
// self-delete path before any chain read, and nothing is written.
// It is also LLR-jwhzh3's abnormal case: a change naming another Organisation
// is not verified against, nor written into, the record the node holds.
// verifies: LLR-379hnv, LLR-38e2kn, LLR-jwhzh3
#[tokio::test(flavor = "multi_thread")]
async fn the_self_delete_path_refuses_an_unheld_organisation_without_a_chain_read() {
    let (s, counting) = admitted("sd-unheld").await;
    let envelope = Envelope::build(OrgId::new([0xee; 20]), SequenceNumber::new(1), &foreign_delta()).unwrap();
    let on_disk = store_bytes("sd-unheld", "b");
    let before = counting.reads();
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &carrying(envelope)).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert_eq!(counting.reads(), before);
    assert_eq!(store_bytes("sd-unheld", "b"), on_disk, "nothing written");
}

// Abnormal: for a held Organisation the chain no longer knows, the refusal
// comes only after the chain-free checks, from the one chain read.
// verifies: REQ-bvh8v6, LLR-3wb7th, LLR-2r2fha
#[tokio::test(flavor = "multi_thread")]
async fn a_held_organisation_absent_from_the_chain_is_refused_after_the_chain_free_checks() {
    let (mut s, counting) = admitted("held-absent").await;
    let msg = captured_admission_of_c(&mut s).await;
    counting.hide(s.org_id);
    let held = rec_of(&s.svc_b, s.org_id);
    // An unlisted sender is refused before the absence is met.
    refused_from_a_relay(&mut s.svc_b, &counting, s.org_id, &msg).await;
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    // A chain-free failure is reported as itself, not as the absence.
    let garbled = with_envelope(&msg, Envelope { delta_bytes: vec![0xff; 16], ..envelope_of(&msg).clone() });
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, b_addr, &garbled).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::MalformedDelta);
    assert_eq!(counting.reads(), before);
    // An envelope that passes them meets the absence on the one read.
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver_from(a_seed, b_addr, &msg).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert_eq!(counting.reads() - before, 1);
    assert_eq!(rec_of(&svc_b, s.org_id).last_seq, held.last_seq);
}

/// Bodies of Organisation information that does not decode, from `genuine`:
/// its key cut off, its key one byte short, and no snapshot and no key.
fn keyless_bodies(genuine: &WireMessage) -> [Vec<u8>; 3] {
    let body = encode_frame(genuine).unwrap()[4..].to_vec();
    // Organisation information's index, then the Envelope and nothing.
    let mut no_snapshot = vec![0u8];
    no_snapshot.extend(postcard::to_allocvec(envelope_of(genuine)).unwrap());
    [body[..body.len() - 32].to_vec(), body[..body.len() - 1].to_vec(), no_snapshot]
}

// LLR-xn5pwc, REQ-c29s93: Organisation information without its Organisation
// private key, or with a key short of 32 bytes, does not decode, and both
// receive paths refuse it with the typed error before any record is consulted
// and with no chain read; the record is untouched and nothing is written.
// verifies: LLR-xn5pwc, LLR-j5vbqj, REQ-c29s93
#[tokio::test(flavor = "multi_thread")]
async fn a_message_without_its_organisation_private_key_is_refused_before_the_chain() {
    let (mut s, counting) = admitted("keyless").await;
    let genuine = captured_admission_of_c(&mut s).await;
    let held = rec_of(&s.svc_b, s.org_id);
    let on_disk = store_bytes("keyless", "b");
    let before = counting.reads();
    let mut svc_b = s.svc_b;
    for raw in keyless_bodies(&genuine) {
        let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
        deliver_raw(addr, &raw).await;
        let (back, result) = task.await.unwrap();
        assert_eq!(result.unwrap_err(), OrgNodeError::MalformedMessage, "{} bytes", raw.len());
        let (addr, task) = spawn_self_delete(back, &s.b_device_kp).await;
        deliver_raw(addr, &raw).await;
        let (back, result) = task.await.unwrap();
        assert_eq!(result.unwrap_err(), OrgNodeError::MalformedMessage, "{} bytes", raw.len());
        svc_b = back;
    }
    assert_eq!(counting.reads(), before, "no chain read");
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.root_hash, after.last_seq), (held.epoch, held.root_hash, held.last_seq));
    assert_eq!(store_bytes("keyless", "b"), on_disk, "nothing written");
}

/// verifies: LLR-38e2kn, LLR-pt32fx, LLR-kzgjz8
///
/// Abnormal: a notice about an unheld Organisation is refused with
/// `RevocationNotHeld` on both paths, with no chain read, whoever sends it; a
/// notice for B whose proof does not verify against the chain (B still
/// listed), sent from A's listed Device, is refused with
/// `RevocationProofRefused` after one read; the same notice from a random
/// relay is refused with `SenderNotListed` and no chain read (owner ruling R1
/// of 2026-10-07; added at S3 T12a). Nothing is written.
#[tokio::test(flavor = "multi_thread")]
async fn revocations_are_refused_without_writing() {
    let (s, counting) = setup_counting("refuse-notice").await;
    let mut s = admit_b_directly(s).await;
    let b_id = id_by_handle(&rec_of(&s.svc_b, s.org_id), "bob");
    let b_device = s.b_device_kp.device_key().unwrap();
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let relay_seed: [u8; 32] = rand::random();
    let proof_from_elsewhere = trie_without(b_id, b_device).prove_absent(&b_id, &b_device).unwrap();
    let unheld = RevocationNotice { org_id: OrgId::new([9; 20]), member_id: b_id, device: b_device, proof: proof_from_elsewhere.clone() };
    let still_listed = RevocationNotice { org_id: s.org_id, member_id: b_id, device: b_device, proof: proof_from_elsewhere };
    let held = rec_of(&s.svc_b, s.org_id);
    let before = store_bytes("refuse-notice", "b");
    let cases = [
        (unheld, relay_seed, 0, "RevocationNotHeld"),
        (still_listed.clone(), a_seed, 1, "RevocationProofRefused"),
        (still_listed, relay_seed, 0, "SenderNotListed"),
    ];
    let refusal = |error: &OrgNodeError| match error {
        OrgNodeError::RevocationNotHeld { .. } => "RevocationNotHeld",
        OrgNodeError::RevocationProofRefused { .. } => "RevocationProofRefused",
        OrgNodeError::SenderNotListed { .. } => "SenderNotListed",
        _ => "another refusal",
    };
    for (notice, sender_seed, expected_reads, expected) in cases {
        let reads_before = counting.reads();
        let result = deliver_from_to_self_delete(&mut s.svc_b, sender_seed, WireMessage::Revocation(notice.clone())).await;
        assert_eq!(result.as_ref().map(|_| ()).map_err(refusal), Err(expected), "{result:?}");
        assert_eq!(counting.reads() - reads_before, expected_reads);
        let reads_before = counting.reads();
        let result = deliver_from_to_receive(&mut s.svc_b, sender_seed, WireMessage::Revocation(notice)).await;
        assert_eq!(result.as_ref().map(|_| ()).map_err(refusal), Err(expected), "{result:?}");
        assert_eq!(counting.reads() - reads_before, expected_reads);
    }
    let after = rec_of(&s.svc_b, s.org_id);
    assert_eq!(
        (after.epoch, after.root_hash, after.org_private_key_for_test()),
        (held.epoch, held.root_hash, held.org_private_key_for_test())
    );
    assert_eq!(s.svc_b.list_personas().len(), 1, "B's Persona is kept");
    assert_eq!(store_bytes("refuse-notice", "b"), before, "nothing written");
}

/// verifies: LLR-pt32fx, LLR-5azhry, LLR-3aysup
///
/// A's receive paths report B's acknowledgement, sent from B's own Device —
/// `Acknowledged` on the self-delete path, `ReceiveOutcome.acknowledged`
/// with the record's epoch and root on the ordinary one — with no chain read
/// and no write; an acknowledgement with a forged signature from B's Device
/// is refused, and B's genuine acknowledgement from a random relay is refused
/// with `AcknowledgementNotFromItsDevice` (the acknowledgement sender rule of
/// 2026-10-07; added at S3 T12a); nothing written.
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_is_checked_and_reported() {
    let (s, counting) = setup_counting("ack-report").await;
    let mut s = admit_b_directly(s).await;
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, Some(b_addr)).await.unwrap();
    let (_svc_b, outcome) = b_task.await.unwrap();
    let SelfDeleteOutcome::SelfDeleted { acknowledgements, .. } = outcome.unwrap() else { panic!("self-deleted") };
    let acknowledgement = acknowledgements[0].clone();
    let held = rec_of(&s.svc_a, s.org_id);
    let before = store_bytes("ack-report", "a");
    let reads_before = counting.reads();
    let b_seed = *s.b_device_kp.device_seed().expose_secret();

    let result =
        deliver_from_to_self_delete(&mut s.svc_a, b_seed, WireMessage::Acknowledgement(acknowledgement.clone())).await;
    let SelfDeleteOutcome::Acknowledged(verified) = result.unwrap() else { panic!("acknowledged") };
    assert_eq!(verified.into_inner(), acknowledgement);

    let outcome = deliver_from_to_receive(&mut s.svc_a, b_seed, WireMessage::Acknowledgement(acknowledgement.clone()))
        .await
        .unwrap();
    assert_eq!((outcome.org_id, outcome.epoch, outcome.root), (s.org_id, held.epoch, held.root_hash));
    assert!(outcome.acknowledgements.is_empty());
    assert_eq!(outcome.acknowledged.map(|verified| verified.into_inner()), Some(acknowledgement.clone()));

    let not_from_its_device = Err(OrgNodeError::AcknowledgementNotFromItsDevice { org_id: s.org_id });
    let relayed = deliver_to_self_delete(&mut s.svc_a, WireMessage::Acknowledgement(acknowledgement.clone())).await;
    assert_eq!(relayed.map(|_| ()), not_from_its_device);
    let relayed = deliver_to_receive(&mut s.svc_a, WireMessage::Acknowledgement(acknowledgement.clone())).await;
    assert_eq!(relayed.map(|_| ()), not_from_its_device);

    let mut forged = acknowledgement;
    forged.signature.0[0] ^= 1;
    let refused = deliver_from_to_self_delete(&mut s.svc_a, b_seed, WireMessage::Acknowledgement(forged.clone())).await;
    assert!(matches!(refused, Err(OrgNodeError::AcknowledgementSignatureInvalid { .. })), "{refused:?}");
    let refused = deliver_from_to_receive(&mut s.svc_a, b_seed, WireMessage::Acknowledgement(forged)).await;
    assert!(matches!(refused, Err(OrgNodeError::AcknowledgementSignatureInvalid { .. })), "{refused:?}");

    assert_eq!((counting.reads() - reads_before, store_bytes("ack-report", "a")), (0, before));
}

/// verifies: LLR-2r2fha, LLR-kzgjz8, LLR-3aysup
///
/// The sender is checked before the chain on both paths, for both kinds a
/// held Organisation receives; an acknowledgement is held to its own rule,
/// not to the record's list.
#[tokio::test(flavor = "multi_thread")]
async fn a_held_organisations_messages_from_an_unlisted_device_cost_no_chain_read() {
    let (s, counting) = setup_counting("unlisted-sender").await;
    let mut s = admit_b_directly(s).await;
    let admission = captured_admission_of_c(&mut s).await;
    let before = store_bytes("unlisted-sender", "b");
    let reads_before = counting.reads();
    let on_receive = deliver_to_receive(&mut s.svc_b, admission.clone()).await;
    let on_self_delete = deliver_to_self_delete(&mut s.svc_b, admission).await;
    for result in [on_receive.map(|_| ()), on_self_delete.map(|_| ())] {
        assert_eq!(result, Err(OrgNodeError::SenderNotListed { org_id: s.org_id }));
    }
    assert_eq!((counting.reads() - reads_before, store_bytes("unlisted-sender", "b")), (0, before));

    // An acknowledgement's sender is a Device the record no longer lists: B's
    // own, sent to A after B's removal, is accepted (no `SenderNotListed`).
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, Some(b_addr)).await.unwrap();
    let (_svc_b, outcome) = b_task.await.unwrap();
    let SelfDeleteOutcome::SelfDeleted { acknowledgements, .. } = outcome.unwrap() else { panic!("self-deleted") };
    let b_seed = *s.b_device_kp.device_seed().expose_secret();
    let reads_before = counting.reads();
    let accepted = deliver_from_to_receive(&mut s.svc_a, b_seed, WireMessage::Acknowledgement(acknowledgements[0].clone())).await;
    assert!(accepted.unwrap().acknowledged.is_some());
    assert_eq!(counting.reads(), reads_before);
}

/// verifies: LLR-38e2kn
///
/// Normal: a notice about an Organisation the node only expects is refused
/// on both paths with no chain read, and the expectation stays in place — so
/// the first admission it expects, arriving next, is taken.
#[tokio::test(flavor = "multi_thread")]
async fn after_an_unheld_revocation_the_expected_admission_is_still_taken() {
    let (mut s, counting) = setup_counted("unheld-keeps-expectation").await;
    let joiner = s.joiner_b.clone();
    let admission = captured_admission(&mut s, &joiner).await;
    let b_device = s.b_device_kp.device_key().unwrap();
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let proof = trie_without(b_id, b_device).prove_absent(&b_id, &b_device).unwrap();
    let notice = RevocationNotice { org_id: s.org_id, member_id: b_id, device: b_device, proof };
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let not_held = Err(OrgNodeError::RevocationNotHeld { org_id: s.org_id });
    let before = counting.reads();
    let refused = deliver_from_to_receive(&mut s.svc_b, a_seed, WireMessage::Revocation(notice.clone())).await;
    assert_eq!(refused.map(|_| ()), not_held);
    let refused = deliver_from_to_self_delete(&mut s.svc_b, a_seed, WireMessage::Revocation(notice)).await;
    assert_eq!(refused.map(|_| ()), not_held);
    assert_eq!(counting.reads(), before, "no chain read");
    assert_eq!(s.svc_b.expected_admissions().len(), 1, "the expectation stays");

    let admitted = deliver_from_to_receive(&mut s.svc_b, a_seed, admission).await.unwrap();
    assert_eq!((admitted.org_id, admitted.epoch), (s.org_id, Epoch::new(2)));
    assert_eq!(s.svc_b.list_orgs().len(), 1);
    assert!(s.svc_b.expected_admissions().is_empty());
}

/// verifies: LLR-kzgjz8
///
/// Normal: on each receive path, B's genuine revocation notice delivered by
/// A's Device, which B's record lists, passes the sender check and reaches
/// the one chain read and `accept`: one acknowledgement under the chain's
/// state, and B forgets the Organisation.
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_from_a_listed_device_reaches_accept_on_both_paths() {
    for self_delete in [false, true] {
        let tag = if self_delete { "listed-revocation-sd" } else { "listed-revocation-rv" };
        let (s, counting) = setup_counting(tag).await;
        let mut s = admit_b_directly(s).await;
        let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
        let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
        revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, Some(sink_addr)).await.unwrap();
        let (_sink, _sender, notice) = sink.await.unwrap();
        assert!(matches!(notice, WireMessage::Revocation(_)), "B's Device receives its notice");
        let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
        let reads_before = counting.reads();
        let acknowledgements = if self_delete {
            let outcome = deliver_from_to_self_delete(&mut s.svc_b, a_seed, notice).await.unwrap();
            let SelfDeleteOutcome::SelfDeleted { org_id, acknowledgements } = outcome else { panic!("self-deleted") };
            assert_eq!(org_id, s.org_id);
            acknowledgements
        } else {
            deliver_from_to_receive(&mut s.svc_b, a_seed, notice).await.unwrap().acknowledgements
        };
        assert_eq!(counting.reads() - reads_before, 1, "one chain read");
        let chain = s.chain.get(&s.org_id).unwrap();
        assert_eq!(acknowledgements.len(), 1);
        assert_eq!((acknowledgements[0].member_id, acknowledgements[0].epoch), (b_id, chain.epoch));
        assert!(s.svc_b.list_orgs().is_empty() && s.svc_b.list_personas().is_empty());
    }
}

/// verifies: LLR-3q63zv
///
/// Normal: on `receive_and_self_delete_if_revoked`, Organisation information
/// delivered by A's Device, which B's record lists, is acted on, on the
/// branch that updates the record (C's admission: committed at epoch 3) and
/// on the branch that deletes it (B's own removal: B forgets the
/// Organisation and signs its acknowledgement), each after one chain read.
#[tokio::test(flavor = "multi_thread")]
async fn the_self_delete_path_acts_on_an_update_and_a_removal_from_a_listed_device() {
    let (s, counting) = setup_counting("sd-listed").await;
    let mut s = admit_b_directly(s).await;
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);

    let admission = captured_admission_of_c(&mut s).await;
    let reads_before = counting.reads();
    let updated = deliver_from_to_self_delete(&mut s.svc_b, a_seed, admission).await.unwrap();
    assert!(matches!(updated, SelfDeleteOutcome::UpdatedNotRevoked { org_id } if org_id == s.org_id), "{updated:?}");
    assert_eq!(counting.reads() - reads_before, 1);
    assert_eq!((rec_of(&s.svc_b, s.org_id).epoch, rec_of(&s.svc_b, s.org_id).trie_members.len()), (Epoch::new(3), 3));

    let a_device = s.svc_a.persona_public_keys(&s.pid_a).unwrap().1;
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    revoke_and_tell(&mut s.svc_a, &s.chain, s.org_id, b_id, a_device, sink_addr).await.unwrap();
    let (_sink, _sender, removal) = sink.await.unwrap();
    assert!(matches!(removal, WireMessage::OrgInformation { .. }), "A's Device is listed");
    let reads_before = counting.reads();
    let removed = deliver_from_to_self_delete(&mut s.svc_b, a_seed, removal).await.unwrap();
    let SelfDeleteOutcome::SelfDeleted { org_id, acknowledgements } = removed else { panic!("self-deleted") };
    assert_eq!((org_id, acknowledgements.len()), (s.org_id, 1));
    assert_eq!(counting.reads() - reads_before, 1);
    assert!(s.svc_b.list_orgs().is_empty() && s.svc_b.list_personas().is_empty());
}

// The same refusal on a first admission: before the expectations are
// consulted — the expectation stays — and with no chain read.
// verifies: LLR-xn5pwc, REQ-c29s93
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_without_its_key_is_refused_and_keeps_the_expectation() {
    let (mut s, counting) = setup_counted("keyless-first").await;
    let joiner = s.joiner_b.clone();
    let genuine = captured_admission(&mut s, &joiner).await;
    let on_disk = store_bytes("keyless-first", "b");
    let before = counting.reads();
    let mut svc_b = s.svc_b;
    for raw in keyless_bodies(&genuine) {
        let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
        deliver_raw(addr, &raw).await;
        let (back, result) = task.await.unwrap();
        assert_eq!(result.unwrap_err(), OrgNodeError::MalformedMessage, "{} bytes", raw.len());
        svc_b = back;
    }
    assert_eq!(counting.reads(), before, "no chain read");
    assert!(svc_b.list_orgs().is_empty());
    assert_eq!(svc_b.expected_admissions().len(), 1, "the expectation is kept");
    assert_eq!(store_bytes("keyless-first", "b"), on_disk, "nothing written");
}

// REQ-uk9rw7 (supersedes REQ-ztdza4: owner ruling R1 of 2026-10-07, and the
// owner's hybrid ruling on PR-zf924s, 2026-10-08), at the value boundary: an
// update about a held Organisation from a Device the current record does not
// list is refused by the chain-free phase itself, which returns no pending
// value, so no chain state is asked for; the same update from a listed
// Device names the Organisation whose state it needs. Nothing is written
// either way.
// verifies: REQ-uk9rw7, REQ-ztdza4, LLR-2r2fha
#[tokio::test(flavor = "multi_thread")]
async fn an_update_from_an_unlisted_device_is_refused_before_any_chain_state_is_asked_for() {
    let (mut s, counting) = admitted("unlisted-value").await;
    let msg = captured_admission_of_c(&mut s).await;
    let on_disk = store_bytes("unlisted-value", "b");
    let held = rec_of(&s.svc_b, s.org_id);
    let reads_before = counting.reads();

    let stranger = org_node::test_fixtures::device_key(0x6d);
    assert!(!held.trie_members.iter().any(|member| member.device_keys.contains(&stranger)));
    let not_listed = Err(OrgNodeError::SenderNotListed { org_id: s.org_id });
    assert_eq!(s.svc_b.prepare_receive(stranger, msg.clone()).map(|_| ()), not_listed);
    assert_eq!(s.svc_b.prepare_self_delete(stranger, msg.clone()).map(|_| ()), not_listed);

    let a_device = org_node::DeviceSeed::from(device_seed_of(&s.svc_a, &s.pid_a))
        .signing_keypair()
        .device_key()
        .unwrap();
    match s.svc_b.prepare_receive(a_device, msg) {
        Ok(org_node::service::Prepared::NeedsChain(pending)) => assert_eq!(pending.org_id(), s.org_id),
        Ok(org_node::service::Prepared::Done(_)) => panic!("an update is never decided without the chain"),
        Err(refused) => panic!("an update from a listed Device passes the chain-free phase: {refused:?}"),
    }

    assert_eq!(counting.reads(), reads_before, "the chain-free phase reads no chain");
    let after = rec_of(&s.svc_b, s.org_id);
    assert_eq!((after.epoch, after.root_hash, after.last_seq), (held.epoch, held.root_hash, held.last_seq));
    assert_eq!(store_bytes("unlisted-value", "b"), on_disk, "nothing written");
}
