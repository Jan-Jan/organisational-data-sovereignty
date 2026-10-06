#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The receive paths run every chain-free check before they read the chain,
//! and read it at most once (REQ-f2k4tr, RC-mj6gjq; LLR-9ew26y, LLR-3wb7th,
//! LLR-379hnv), observed through a chain that counts its reads.

mod support;

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
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

// Normal: an update that passes the chain-free checks reads the chain once.
// verifies: REQ-f2k4tr, LLR-9ew26y
#[tokio::test(flavor = "multi_thread")]
async fn an_update_that_passes_the_chain_free_checks_reads_the_chain_once() {
    let (mut s, counting) = admitted("reads-once").await;
    let msg = captured_admission_of_c(&mut s).await;
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap().epoch, Epoch::new(3));
    assert_eq!(counting.reads() - before, 1);
    assert_eq!(rec_of(&svc_b, s.org_id).trie_members.len(), 3);
}

// Abnormal: a replayed envelope is refused as stale with no chain read, and
// the record — mark included — is untouched.
// verifies: REQ-f2k4tr, REQ-mr5abb, LLR-9ew26y
#[tokio::test(flavor = "multi_thread")]
async fn a_stale_envelope_is_refused_without_a_chain_read() {
    let (mut s, counting) = admitted("stale").await;
    let msg = captured_admission_of_c(&mut s).await;
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (svc_b, first) = b_task.await.unwrap();
    first.unwrap();
    let held = rec_of(&svc_b, s.org_id);
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (svc_b, again) = b_task.await.unwrap();
    assert!(matches!(again, Err(OrgNodeError::StaleSeq { .. })), "{again:?}");
    assert_eq!(counting.reads(), before, "no chain read for a stale envelope");
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.root_hash, after.last_seq), (held.epoch, held.root_hash, held.last_seq));
}

// Abnormal: a Change set built on another base is refused with no chain read.
// verifies: REQ-f2k4tr, LLR-9ew26y
#[tokio::test(flavor = "multi_thread")]
async fn a_change_set_on_another_base_is_refused_without_a_chain_read() {
    let (s, counting) = admitted("other-base").await;
    let mark = rec_of(&s.svc_b, s.org_id).last_seq;
    let envelope = Envelope::build(s.org_id, SequenceNumber::new(mark.get() + 1), &foreign_delta()).unwrap();
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &WireMessage::Revocation { envelope }).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::DeltaBaseMismatch);
    assert_eq!(counting.reads(), before);
}

// Abnormal: Change set bytes that do not decode are refused with no chain read.
// verifies: REQ-f2k4tr, LLR-9ew26y
#[tokio::test(flavor = "multi_thread")]
async fn undecodable_change_set_bytes_are_refused_without_a_chain_read() {
    let (s, counting) = admitted("undecodable").await;
    let mark = rec_of(&s.svc_b, s.org_id).last_seq;
    let envelope = Envelope { org_id: s.org_id, parent_seq: SequenceNumber::new(mark.get() + 1), delta_bytes: vec![0xff; 16] };
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &WireMessage::Revocation { envelope }).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::MalformedDelta);
    assert_eq!(counting.reads(), before);
}

// Abnormal: the self-delete path runs the same checks first.
// verifies: LLR-9ew26y
#[tokio::test(flavor = "multi_thread")]
async fn the_self_delete_path_refuses_a_stale_envelope_without_a_chain_read() {
    let (mut s, counting) = admitted("sd-stale").await;
    let msg = captured_admission_of_c(&mut s).await;
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (svc_b, first) = b_task.await.unwrap();
    first.unwrap();
    let before = counting.reads();
    let (b_addr, b_task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
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
    deliver(b_addr, &WireMessage::Revocation { envelope }).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert_eq!(counting.reads(), before);
    assert_eq!(store_bytes("sd-unheld", "b"), on_disk, "nothing written");
}

// Abnormal: for a held Organisation the chain no longer knows, the refusal
// comes only after the chain-free checks, from the one chain read.
// verifies: REQ-bvh8v6, LLR-3wb7th
#[tokio::test(flavor = "multi_thread")]
async fn a_held_organisation_absent_from_the_chain_is_refused_after_the_chain_free_checks() {
    let (mut s, counting) = admitted("held-absent").await;
    let msg = captured_admission_of_c(&mut s).await;
    counting.hide(s.org_id);
    let held = rec_of(&s.svc_b, s.org_id);
    // A chain-free failure is reported as itself, not as the absence.
    let garbled = with_envelope(&msg, Envelope { delta_bytes: vec![0xff; 16], ..msg.envelope().clone() });
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &garbled).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::MalformedDelta);
    assert_eq!(counting.reads(), before);
    // An envelope that passes them meets the absence on the one read.
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert_eq!(counting.reads() - before, 1);
    assert_eq!(rec_of(&svc_b, s.org_id).last_seq, held.last_seq);
}

/// Bodies of Organisation information that does not decode, from `genuine`:
/// its key cut off, its key one byte short, and no snapshot and no key.
fn keyless_bodies(genuine: &WireMessage) -> [Vec<u8>; 3] {
    let body = encode_frame(genuine).unwrap()[4..].to_vec();
    let mut no_snapshot = encode_frame(&WireMessage::Revocation { envelope: genuine.envelope().clone() }).unwrap()[4..].to_vec();
    no_snapshot[0] = 0;
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
