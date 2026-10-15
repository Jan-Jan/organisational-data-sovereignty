#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The receive path runs every chain-free check before it reads the chain,
//! and reads it at most once (REQ-f2k4tr, RC-mj6gjq; LLR-9ew26y, LLR-3wb7th),
//! observed through a chain that counts its reads.
//!
//! *S4 task T4, 2026-10-13:* there is one receive path, `check_received`
//! then `commit_checked`, driven by values (support's `Node::receive_from`);
//! the tests that ran a message through both the ordinary and the
//! self-delete path run it through the one path, and the self-delete path's
//! `OrgNotOnChain` item for an unheld Organisation is retired (S4 T4; see the
//! S4 verification record).

mod support;

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::revocation::RevocationNotice;
use org_node::service::{Checked, ReceiveKind};
use org_node::test_fixtures::admit_member_delta;
use org_node::wire::{encode_message, WireMessage};
use org_node::{Envelope, Epoch, MemberSeed, SequenceNumber};
use support::*;

/// B admitted at epoch 2, B's chain reads counted.
fn admitted(tag: &str) -> (Setup, CountingChain) {
    let (s, counting) = setup_counted(tag);
    (admit_b_directly(s), counting)
}

/// A Change set on a base B does not hold.
fn foreign_delta() -> org_members::delta::Delta {
    admit_member_delta(&MemberSeed::from([0x5a; 32]).x25519_keypair()).0
}

/// From a relay B's record does not list, `msg` is refused with
/// `SenderNotListed` and no chain read (owner ruling R1 of 2026-10-07,
/// LLR-2r2fha). The tests below send from A's listed Device otherwise
/// (*rewritten 2026-10-07, S3 T12a*: they sent from a relay).
fn refused_from_a_relay(svc_b: &mut Node, counting: &CountingChain, org_id: OrgId, msg: &WireMessage) {
    let before = counting.reads();
    let not_listed = Err(OrgNodeError::SenderNotListed { org_id });
    assert_eq!(deliver_to_receive(svc_b, msg.clone()).map(|_| ()), not_listed);
    assert_eq!(counting.reads(), before, "no chain read for an unlisted sender");
}
// Normal: an update that passes the chain-free checks reads the chain once.
// verifies: REQ-f2k4tr, LLR-9ew26y, LLR-2r2fha
#[test]
fn an_update_that_passes_the_chain_free_checks_reads_the_chain_once() {
    let (mut s, counting) = admitted("reads-once");
    let msg = captured_admission_of_c(&mut s);
    refused_from_a_relay(&mut s.svc_b, &counting, s.org_id, &msg);
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b);
    deliver_from(a_seed, b_addr, &msg);
    let (svc_b, result) = b_task.run();
    assert_eq!(result.unwrap().epoch, Epoch::new(3));
    assert_eq!(counting.reads() - before, 1);
    assert_eq!(rec_of(&svc_b, s.org_id).trie_members.len(), 3);
}

// Abnormal: a replayed envelope is refused as stale with no chain read, and
// the record — mark included — is untouched.
// verifies: REQ-f2k4tr, REQ-mr5abb, LLR-9ew26y, LLR-2r2fha
#[test]
fn a_stale_envelope_is_refused_without_a_chain_read() {
    let (mut s, counting) = admitted("stale");
    let msg = captured_admission_of_c(&mut s);
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let (b_addr, b_task) = spawn_receive(s.svc_b);
    deliver_from(a_seed, b_addr, &msg);
    let (mut svc_b, first) = b_task.run();
    first.unwrap();
    let held = rec_of(&svc_b, s.org_id);
    refused_from_a_relay(&mut svc_b, &counting, s.org_id, &msg);
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(svc_b);
    deliver_from(a_seed, b_addr, &msg);
    let (svc_b, again) = b_task.run();
    assert!(matches!(again, Err(OrgNodeError::StaleSeq { .. })), "{again:?}");
    assert_eq!(counting.reads(), before, "no chain read for a stale envelope");
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.root_hash, after.last_seq), (held.epoch, held.root_hash, held.last_seq));
}

// Abnormal: a Change set built on another base is refused with no chain read.
// verifies: REQ-f2k4tr, LLR-9ew26y, LLR-2r2fha
#[test]
fn a_change_set_on_another_base_is_refused_without_a_chain_read() {
    let (mut s, counting) = admitted("other-base");
    let mark = rec_of(&s.svc_b, s.org_id).last_seq;
    let envelope = Envelope::build(s.org_id, SequenceNumber::new(mark.get() + 1), &foreign_delta()).unwrap();
    refused_from_a_relay(&mut s.svc_b, &counting, s.org_id, &carrying(envelope.clone()));
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b);
    deliver_from(a_seed, b_addr, &carrying(envelope));
    let (_svc_b, result) = b_task.run();
    assert_eq!(result.unwrap_err(), OrgNodeError::DeltaBaseMismatch);
    assert_eq!(counting.reads(), before);
}

// Abnormal: Change set bytes that do not decode are refused with no chain read.
// verifies: REQ-f2k4tr, LLR-9ew26y, LLR-2r2fha
#[test]
fn undecodable_change_set_bytes_are_refused_without_a_chain_read() {
    let (mut s, counting) = admitted("undecodable");
    let mark = rec_of(&s.svc_b, s.org_id).last_seq;
    let envelope = Envelope { org_id: s.org_id, parent_seq: SequenceNumber::new(mark.get() + 1), delta_bytes: vec![0xff; 16] };
    refused_from_a_relay(&mut s.svc_b, &counting, s.org_id, &carrying(envelope.clone()));
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b);
    deliver_from(a_seed, b_addr, &carrying(envelope));
    let (_svc_b, result) = b_task.run();
    assert_eq!(result.unwrap_err(), OrgNodeError::MalformedDelta);
    assert_eq!(counting.reads(), before);
}

// Abnormal: the self-delete path runs the same checks first. *S4 T4,
// 2026-10-13:* the self-delete path folded onto the one receive path; this
// is that path's stale replay, after a first commit on it.
// verifies: LLR-9ew26y, LLR-2r2fha
#[test]
fn the_self_delete_path_refuses_a_stale_envelope_without_a_chain_read() {
    let (mut s, counting) = admitted("sd-stale");
    let msg = captured_admission_of_c(&mut s);
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let (b_addr, b_task) = spawn_receive(s.svc_b);
    deliver_from(a_seed, b_addr, &msg);
    let (mut svc_b, first) = b_task.run();
    first.unwrap();
    refused_from_a_relay(&mut svc_b, &counting, s.org_id, &msg);
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(svc_b);
    deliver_from(a_seed, b_addr, &msg);
    let (_svc_b, again) = b_task.run();
    assert!(matches!(again, Err(OrgNodeError::StaleSeq { .. })), "{again:?}");
    assert_eq!(counting.reads(), before);
}

// Abnormal: an Organisation the node holds no record of is refused before
// any chain read, and nothing is written.
// It is also LLR-jwhzh3's abnormal case: a change naming another Organisation
// is not verified against, nor written into, the record the node holds.
// *S4 T4, 2026-10-13:* on the one receive path Organisation information about
// an unheld Organisation is a first admission, refused by the expectation
// gate (`AdmissionNotExpected`, LLR-38e2kn's order); the self-delete path's
// `OrgNotOnChain` (the retired self-delete-path item; see the S4 verification
// record) is gone with that path.
// verifies: LLR-38e2kn, LLR-jwhzh3
#[test]
fn the_self_delete_path_refuses_an_unheld_organisation_without_a_chain_read() {
    let (s, counting) = admitted("sd-unheld");
    let unheld = OrgId::new([0xee; 20]);
    let envelope = Envelope::build(unheld, SequenceNumber::new(1), &foreign_delta()).unwrap();
    let on_disk = store_bytes("sd-unheld", "b");
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b);
    deliver(b_addr, &carrying(envelope));
    let (_svc_b, result) = b_task.run();
    assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotExpected { org_id: unheld });
    assert_eq!(counting.reads(), before);
    assert_eq!(store_bytes("sd-unheld", "b"), on_disk, "nothing written");
}

// Abnormal: for a held Organisation the chain no longer knows, the refusal
// comes only after the chain-free checks, from the one chain read.
// verifies: REQ-bvh8v6, LLR-3wb7th, LLR-2r2fha
#[test]
fn a_held_organisation_absent_from_the_chain_is_refused_after_the_chain_free_checks() {
    let (mut s, counting) = admitted("held-absent");
    let msg = captured_admission_of_c(&mut s);
    counting.hide(s.org_id);
    let held = rec_of(&s.svc_b, s.org_id);
    // An unlisted sender is refused before the absence is met.
    refused_from_a_relay(&mut s.svc_b, &counting, s.org_id, &msg);
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    // A chain-free failure is reported as itself, not as the absence.
    let garbled = with_envelope(&msg, Envelope { delta_bytes: vec![0xff; 16], ..envelope_of(&msg).clone() });
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b);
    deliver_from(a_seed, b_addr, &garbled);
    let (svc_b, result) = b_task.run();
    assert_eq!(result.unwrap_err(), OrgNodeError::MalformedDelta);
    assert_eq!(counting.reads(), before);
    // An envelope that passes them meets the absence on the one read.
    let (b_addr, b_task) = spawn_receive(svc_b);
    deliver_from(a_seed, b_addr, &msg);
    let (svc_b, result) = b_task.run();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert_eq!(counting.reads() - before, 1);
    assert_eq!(rec_of(&svc_b, s.org_id).last_seq, held.last_seq);
}

/// Bodies of Organisation information that does not decode, from `genuine`:
/// its key cut off, its key one byte short, and no snapshot and no key.
fn keyless_bodies(genuine: &WireMessage) -> [Vec<u8>; 3] {
    let body = encode_message(genuine).unwrap();
    // Organisation information's index, then the Envelope and nothing.
    let mut no_snapshot = vec![0u8];
    no_snapshot.extend(postcard::to_allocvec(envelope_of(genuine)).unwrap());
    [body[..body.len() - 32].to_vec(), body[..body.len() - 1].to_vec(), no_snapshot]
}

// LLR-xn5pwc, REQ-c29s93: Organisation information without its Organisation
// private key, or with a key short of 32 bytes, does not decode, and the
// receive path refuses it with the typed error before any record is
// consulted and with no chain read; the record is untouched and nothing is
// written. (*S4 T4:* one path; it ran on both.)
// verifies: LLR-xn5pwc, LLR-j5vbqj, REQ-c29s93
#[test]
fn a_message_without_its_organisation_private_key_is_refused_before_the_chain() {
    let (mut s, counting) = admitted("keyless");
    let genuine = captured_admission_of_c(&mut s);
    let held = rec_of(&s.svc_b, s.org_id);
    let on_disk = store_bytes("keyless", "b");
    let before = counting.reads();
    let mut svc_b = s.svc_b;
    for raw in keyless_bodies(&genuine) {
        let (addr, task) = spawn_receive(svc_b);
        deliver_raw(addr, &raw);
        let (back, result) = task.run();
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
/// `RevocationNotHeld`, with no chain read, whoever sends it; a notice for B
/// whose proof does not verify against the chain (B still listed), sent from
/// A's listed Device, is refused with `RevocationProofRefused` after one
/// read; the same notice from a random relay is refused with
/// `SenderNotListed` and no chain read (owner ruling R1 of 2026-10-07; added
/// at S3 T12a). Nothing is written. (*S4 T4:* one path; it ran on both.)
#[test]
fn revocations_are_refused_without_writing() {
    let (s, counting) = setup_counting("refuse-notice");
    let mut s = admit_b_directly(s);
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
        let result = deliver_from_to_receive(&mut s.svc_b, sender_seed, WireMessage::Revocation(notice));
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

/// `svc_b`, admitted, receives `svc_a`'s revocation of it: returns B's node
/// and the acknowledgement B signed.
fn revoked_acknowledgement(
    svc_b: Node,
    svc_a: &mut Node,
    chain: &org_node::test_fixtures::ChainSlots,
    org_id: OrgId,
) -> (Node, org_node::revocation::Acknowledgement) {
    let b_id = id_by_handle(&rec_of(svc_a, org_id), "bob");
    let (b_addr, b_task) = spawn_receive(svc_b);
    revoke(svc_a, chain, org_id, b_id, b_addr).unwrap();
    let (svc_b, outcome) = b_task.run();
    let outcome = outcome.unwrap();
    assert_eq!(outcome.kind, ReceiveKind::Revoked);
    (svc_b, outcome.acknowledgements[0].clone())
}

/// verifies: LLR-pt32fx, LLR-5azhry, LLR-3aysup
///
/// A's receive path reports B's acknowledgement, sent from B's own Device —
/// `Acknowledged`, decided in the check — with no chain read and no write;
/// an acknowledgement with a forged signature from B's Device is refused,
/// and B's genuine acknowledgement from a random relay is refused with
/// `AcknowledgementNotFromItsDevice` (the acknowledgement sender rule of
/// 2026-10-07; added at S3 T12a); nothing written. (*S4 T4:* one path; the
/// ordinary path's `ReceiveOutcome.acknowledged` with the record's epoch and
/// root went with the self-delete path's `Acknowledged`, into
/// `Checked::Acknowledged`.)
#[test]
fn an_acknowledgement_is_checked_and_reported() {
    let (s, counting) = setup_counting("ack-report");
    let mut s = admit_b_directly(s);
    let (svc_b, acknowledgement) = revoked_acknowledgement(s.svc_b, &mut s.svc_a, &s.chain, s.org_id);
    let before = store_bytes("ack-report", "a");
    let reads_before = counting.reads();
    let b_seed = svc_b.keychain.seed(&s.b_device_kp.device_key().unwrap());

    let verified = receive_message_from(&mut s.svc_a, b_seed, &WireMessage::Acknowledgement(acknowledgement.clone()))
        .unwrap()
        .acknowledged();
    assert_eq!(verified.into_inner(), acknowledgement);

    let not_from_its_device = Err(OrgNodeError::AcknowledgementNotFromItsDevice { org_id: s.org_id });
    let relayed =
        receive_message_from(&mut s.svc_a, rand::random(), &WireMessage::Acknowledgement(acknowledgement.clone()));
    assert_eq!(relayed.map(|_| ()), not_from_its_device);

    let mut forged = acknowledgement;
    forged.signature.0[0] ^= 1;
    let refused = receive_message_from(&mut s.svc_a, b_seed, &WireMessage::Acknowledgement(forged));
    assert!(matches!(refused, Err(OrgNodeError::AcknowledgementSignatureInvalid { .. })), "{refused:?}");

    assert_eq!((counting.reads() - reads_before, store_bytes("ack-report", "a")), (0, before));
}

/// verifies: LLR-2r2fha, LLR-kzgjz8, LLR-3aysup
///
/// The sender is checked before the chain, for both kinds a held
/// Organisation receives; an acknowledgement is held to its own rule, not to
/// the record's list. (*S4 T4:* one path; it ran on both.)
#[test]
fn a_held_organisations_messages_from_an_unlisted_device_cost_no_chain_read() {
    let (s, counting) = setup_counting("unlisted-sender");
    let mut s = admit_b_directly(s);
    let admission = captured_admission_of_c(&mut s);
    let before = store_bytes("unlisted-sender", "b");
    let reads_before = counting.reads();
    let on_receive = deliver_to_receive(&mut s.svc_b, admission);
    assert_eq!(on_receive.map(|_| ()), Err(OrgNodeError::SenderNotListed { org_id: s.org_id }));
    assert_eq!((counting.reads() - reads_before, store_bytes("unlisted-sender", "b")), (0, before));

    // An acknowledgement's sender is a Device the record no longer lists: B's
    // own, sent to A after B's removal, is accepted (no `SenderNotListed`).
    let (svc_b, acknowledgement) = revoked_acknowledgement(s.svc_b, &mut s.svc_a, &s.chain, s.org_id);
    let b_seed = svc_b.keychain.seed(&s.b_device_kp.device_key().unwrap());
    let reads_before = counting.reads();
    let accepted = receive_message_from(&mut s.svc_a, b_seed, &WireMessage::Acknowledgement(acknowledgement));
    assert!(matches!(accepted, Ok(Received::Acknowledged(_))), "{accepted:?}");
    assert_eq!(counting.reads(), reads_before);
}

/// verifies: LLR-38e2kn
///
/// Normal: a notice about an Organisation the node only expects is refused
/// with no chain read, and the expectation stays in place — so the first
/// admission it expects, arriving next, is taken. (*S4 T4:* one path; it ran
/// on both.)
#[test]
fn after_an_unheld_revocation_the_expected_admission_is_still_taken() {
    let (mut s, counting) = setup_counted("unheld-keeps-expectation");
    let joiner = s.joiner_b.clone();
    let admission = captured_admission(&mut s, &joiner);
    let b_device = s.b_device_kp.device_key().unwrap();
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let proof = trie_without(b_id, b_device).prove_absent(&b_id, &b_device).unwrap();
    let notice = RevocationNotice { org_id: s.org_id, member_id: b_id, device: b_device, proof };
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let not_held = Err(OrgNodeError::RevocationNotHeld { org_id: s.org_id });
    let before = counting.reads();
    let refused = deliver_from_to_receive(&mut s.svc_b, a_seed, WireMessage::Revocation(notice));
    assert_eq!(refused.map(|_| ()), not_held);
    assert_eq!(counting.reads(), before, "no chain read");
    assert_eq!(s.svc_b.expected_admissions().len(), 1, "the expectation stays");

    let admitted = deliver_from_to_receive(&mut s.svc_b, a_seed, admission).unwrap();
    assert_eq!((admitted.org_id, admitted.epoch), (s.org_id, Epoch::new(2)));
    assert_eq!(s.svc_b.list_orgs().len(), 1);
    assert!(s.svc_b.expected_admissions().is_empty());
}

/// verifies: LLR-kzgjz8
///
/// Normal: B's genuine revocation notice delivered by A's Device, which B's
/// record lists, passes the sender check and reaches the one chain read and
/// `accept`: one acknowledgement under the chain's state, and B forgets the
/// Organisation. (*S4 T4:* one path; it ran on each of two.)
#[test]
fn a_revocation_from_a_listed_device_reaches_accept_on_both_paths() {
    let (s, counting) = setup_counting("listed-revocation");
    let mut s = admit_b_directly(s);
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let (sink_addr, sink) = spawn_recv_one(rand::random());
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, sink_addr).unwrap();
    let (_sink, _sender, notice) = sink.run();
    assert!(matches!(notice, WireMessage::Revocation(_)), "B's Device receives its notice");
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);
    let reads_before = counting.reads();
    let outcome = deliver_from_to_receive(&mut s.svc_b, a_seed, notice).unwrap();
    assert_eq!((outcome.org_id, outcome.kind), (s.org_id, ReceiveKind::Revoked));
    let acknowledgements = outcome.acknowledgements;
    assert_eq!(counting.reads() - reads_before, 1, "one chain read");
    let chain = s.chain.get(&s.org_id).unwrap();
    assert_eq!(acknowledgements.len(), 1);
    assert_eq!((acknowledgements[0].member_id, acknowledgements[0].epoch), (b_id, chain.epoch));
    assert!(s.svc_b.list_orgs().is_empty() && s.svc_b.list_personas().is_empty());
}

/// verifies: LLR-3q63zv
///
/// Normal: Organisation information delivered by A's Device, which B's
/// record lists, is acted on, on the branch that updates the record (C's
/// admission: committed at epoch 3) and on the branch that deletes it (B's
/// own removal: B forgets the Organisation and signs its acknowledgement),
/// each after one chain read. (*S4 T4:* on the one receive path; it was
/// `receive_and_self_delete_if_revoked`.)
#[test]
fn the_self_delete_path_acts_on_an_update_and_a_removal_from_a_listed_device() {
    let (s, counting) = setup_counting("sd-listed");
    let mut s = admit_b_directly(s);
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let a_seed = device_seed_of(&s.svc_a, &s.pid_a);

    let admission = captured_admission_of_c(&mut s);
    let reads_before = counting.reads();
    let updated = deliver_from_to_receive(&mut s.svc_b, a_seed, admission).unwrap();
    assert_eq!((updated.org_id, updated.kind), (s.org_id, ReceiveKind::Updated));
    assert_eq!(counting.reads() - reads_before, 1);
    assert_eq!((rec_of(&s.svc_b, s.org_id).epoch, rec_of(&s.svc_b, s.org_id).trie_members.len()), (Epoch::new(3), 3));

    // *Amended 2026-10-13 (S4, task T5):* the commit builds no body for its
    // sender's own Device, so the removal is captured as told to C's Device
    // (a further Persona on A's node), which the committed record lists.
    let c_device = device_by_handle(&rec_of(&s.svc_a, s.org_id), "carol");
    let (sink_addr, sink) = spawn_recv_one(rand::random());
    revoke_and_tell(&mut s.svc_a, &s.chain, s.org_id, b_id, c_device, sink_addr).unwrap();
    let (_sink, _sender, removal) = sink.run();
    assert!(matches!(removal, WireMessage::OrgInformation { .. }), "C's Device is listed");
    let reads_before = counting.reads();
    let removed = deliver_from_to_receive(&mut s.svc_b, a_seed, removal).unwrap();
    assert_eq!((removed.org_id, removed.kind, removed.acknowledgements.len()), (s.org_id, ReceiveKind::Removed, 1));
    assert_eq!(counting.reads() - reads_before, 1);
    assert!(s.svc_b.list_orgs().is_empty() && s.svc_b.list_personas().is_empty());
}

// The same refusal on a first admission: before the expectations are
// consulted — the expectation stays — and with no chain read.
// verifies: LLR-xn5pwc, REQ-c29s93
#[test]
fn a_first_admission_without_its_key_is_refused_and_keeps_the_expectation() {
    let (mut s, counting) = setup_counted("keyless-first");
    let joiner = s.joiner_b.clone();
    let genuine = captured_admission(&mut s, &joiner);
    let on_disk = store_bytes("keyless-first", "b");
    let before = counting.reads();
    let mut svc_b = s.svc_b;
    for raw in keyless_bodies(&genuine) {
        let (addr, task) = spawn_receive(svc_b);
        deliver_raw(addr, &raw);
        let (back, result) = task.run();
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
// list is refused by the check itself, which returns no checked message, so
// no chain state is asked for; the same update from a listed Device names
// the Organisation whose state it needs. Nothing is written either way.
// (*S4 T4:* `check_received` in place of `prepare_receive` and
// `prepare_self_delete`.)
// verifies: REQ-uk9rw7, REQ-ztdza4, LLR-2r2fha
#[test]
fn an_update_from_an_unlisted_device_is_refused_before_any_chain_state_is_asked_for() {
    let (mut s, counting) = admitted("unlisted-value");
    let msg = captured_admission_of_c(&mut s);
    let body = encode_message(&msg).unwrap();
    let on_disk = store_bytes("unlisted-value", "b");
    let held = rec_of(&s.svc_b, s.org_id);
    let reads_before = counting.reads();

    let stranger = org_node::test_fixtures::device_key(0x6d);
    assert!(!held.trie_members.iter().any(|member| member.device_keys.contains(&stranger)));
    let not_listed = Err(OrgNodeError::SenderNotListed { org_id: s.org_id });
    assert_eq!(s.svc_b.check_received(stranger, &body).map(|_| ()), not_listed);

    let a_device = persona_of(&s.svc_a, &s.pid_a).device_key;
    match s.svc_b.check_received(a_device, &body) {
        Ok(Checked::ReadChain(checked)) => assert_eq!(checked.org_id(), s.org_id),
        Ok(Checked::Acknowledged(_)) => panic!("an update is never decided without the chain"),
        Err(refused) => panic!("an update from a listed Device passes the check: {refused:?}"),
    }

    assert_eq!(counting.reads(), reads_before, "the check reads no chain");
    let after = rec_of(&s.svc_b, s.org_id);
    assert_eq!((after.epoch, after.root_hash, after.last_seq), (held.epoch, held.root_hash, held.last_seq));
    assert_eq!(store_bytes("unlisted-value", "b"), on_disk, "nothing written");
}
