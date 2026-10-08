#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The receive sequence (LLR-7pj5af): org-io takes one message and its
//! sender from org-node's transport, hands both to org-node's chain-free
//! phase, reads the chain only when that phase returns a pending value, and
//! then exactly once; a refusal or an acknowledgement (decided against the
//! store alone) reads nothing; a failed read or a refused parse applies
//! nothing. Each receiving handle reads through its own `FakeChain` counter
//! over the slots every handle shares, so its reads are its own.

use std::sync::atomic::Ordering;
use std::time::Duration;

use org_io::node::revocation::Acknowledgement;
use org_io::node::transport::wire::WireMessage;
use org_io::node::{Epoch, MemberId, OrgId, OrgNodeError, PersonaId, ReceiveOutcome, SelfDeleteOutcome};
use org_io::test_support::FakeChain;
use org_io::OrgIo;
use rand::rngs::OsRng;

mod common;
use common::handles::{
    deliver, handle, joiner, joiner_of, persona, receiving_addr, reopened_service, spawn_recv_one, NET,
};

/// A has founded an Organisation over `chain`; B has a Persona, an endpoint
/// and its own counter (`chain_b`) over the same slots.
struct Pair {
    a: OrgIo,
    chain: FakeChain,
    b: OrgIo,
    chain_b: FakeChain,
    org: OrgId,
    pid_b: PersonaId,
    addr_b: iroh::EndpointAddr,
}

async fn pair(tag: &str, b_expects_the_admission: bool) -> Pair {
    let chain = FakeChain::new();
    let mut a = handle(tag, "a", &chain);
    let pid_a = persona(&mut a, "alice");
    let org = a.found_organisation(&mut OsRng, &pid_a).await.unwrap();
    let chain_b = FakeChain::sharing_slots_with(&chain);
    let mut b = handle(tag, "b", &chain_b);
    let pid_b = persona(&mut b, "bob");
    if b_expects_the_admission {
        b.node_mut().expect_admission(&mut OsRng, org).unwrap();
    }
    let addr_b = receiving_addr(&mut b).await;
    Pair { a, chain, b, chain_b, org, pid_b, addr_b }
}

/// A admits B and sends the admission to B's `receive_and_verify`; returns
/// B's result.
async fn admit_b(pair: &mut Pair) -> Result<ReceiveOutcome, OrgNodeError> {
    let bob = joiner_of(&pair.b, &pair.pid_b);
    let update = pair.a.node_mut().admit_member(&mut OsRng, pair.org, &bob).unwrap();
    let Pair { a, b, addr_b, .. } = pair;
    let (received, _sent) = tokio::join!(
        async { tokio::time::timeout(NET, b.receive_and_verify(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            a.submit_commit_send(&mut OsRng, &update, bob.device_key, Some(addr_b.clone())).await
        }
    );
    received
}

/// B's MemberId in A's record.
fn member_id_of_b(pair: &Pair) -> MemberId {
    let device_b = joiner_of(&pair.b, &pair.pid_b).device_key;
    let record = pair.a.node_for_test().list_orgs().iter().find(|o| o.org_id == pair.org).cloned().unwrap();
    record.trie_members.iter().find(|m| m.device_keys.contains(&device_b)).unwrap().id
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_held_organisations_message_is_read_once_then_applied() {
    let mut pair = pair("order-read-once", true).await;
    let admitted = admit_b(&mut pair).await.unwrap();
    assert_eq!(admitted.epoch, Epoch::new(2));
    assert_eq!(pair.chain_b.reads(), 1, "the first admission: one read");
    // B now holds the Organisation: A's next update, from A's listed Device.
    let carol = joiner(0x71, "carol");
    let update = pair.a.node_mut().admit_member(&mut OsRng, pair.org, &carol).unwrap();
    let device_b = joiner_of(&pair.b, &pair.pid_b).device_key;
    let Pair { a, b, addr_b, .. } = &mut pair;
    let (received, sent) = tokio::join!(
        async { tokio::time::timeout(NET, b.receive_and_verify(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            a.submit_commit_send(&mut OsRng, &update, device_b, Some(addr_b.clone())).await
        }
    );
    sent.unwrap();
    assert_eq!(received.unwrap().epoch, Epoch::new(3));
    assert_eq!(pair.chain_b.reads(), 2, "the held Organisation's update: one more read");
    assert_eq!(pair.b.node_for_test().list_orgs()[0].epoch, Epoch::new(3), "applied");
}

// The first admission is accepted from any sender: guarded by the
// expectation before the read and by the chain after it.
// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_relayed_by_any_device_is_read_once_then_applied() {
    let mut pair = pair("order-any-sender", true).await;
    let bob = joiner_of(&pair.b, &pair.pid_b);
    let update = pair.a.node_mut().admit_member(&mut OsRng, pair.org, &bob).unwrap();
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    pair.a.submit_commit_send(&mut OsRng, &update, bob.device_key, Some(sink_addr)).await.unwrap();
    let (_, _, captured) = sink.await.unwrap();
    let addr_b = pair.addr_b.clone();
    let (received, ()) = tokio::join!(
        async { tokio::time::timeout(NET, pair.b.receive_and_verify(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            deliver(addr_b, &captured).await;
        }
    );
    assert_eq!(received.unwrap().epoch, Epoch::new(2));
    assert_eq!(pair.chain_b.reads(), 1, "one read, after the expectation gate passed");
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_message_the_chain_free_phase_refuses_causes_no_read() {
    // B does not expect the admission: AdmissionNotExpected, before any read.
    let mut pair = pair("order-not-expected", false).await;
    let refused = admit_b(&mut pair).await.unwrap_err();
    assert!(matches!(refused, OrgNodeError::AdmissionNotExpected { org_id } if org_id == pair.org), "{refused:?}");
    assert_eq!(pair.chain_b.reads(), 0, "no read");
    assert!(pair.b.node_for_test().list_orgs().is_empty());
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_read_applies_nothing() {
    let mut pair = pair("order-read-fails", true).await;
    pair.chain_b.read_failing.store(true, Ordering::SeqCst);
    let refused = admit_b(&mut pair).await.unwrap_err();
    assert!(matches!(&refused, OrgNodeError::Chain(reason) if reason == "node unreachable"), "{refused:?}");
    assert_eq!(pair.chain_b.reads(), 1, "the one read was made, and failed");
    assert!(pair.b.node_for_test().list_orgs().is_empty(), "no record");
    assert_eq!(pair.b.node_for_test().expected_admissions().len(), 1, "the expectation stays");
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_parse_applies_nothing() {
    let mut pair = pair("order-parse-refused", true).await;
    pair.chain_b.parse_refusing.store(true, Ordering::SeqCst);
    let refused = admit_b(&mut pair).await.unwrap_err();
    assert!(matches!(refused, OrgNodeError::InvalidOrgPublicKey), "{refused:?}");
    assert_eq!(pair.chain_b.reads(), 1, "the one read was made, and its parse refused");
    assert!(pair.b.node_for_test().list_orgs().is_empty(), "no record");
    assert_eq!(pair.b.node_for_test().expected_admissions().len(), 1, "the expectation stays");
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_update_from_a_device_the_record_does_not_list_causes_no_read() {
    let mut pair = pair("order-sender-not-listed", true).await;
    admit_b(&mut pair).await.unwrap();
    let reads_before = pair.chain_b.reads();
    // A's genuine admission of C, captured by a sink, relayed to B by a
    // Device no record lists.
    let carol = joiner(0x71, "carol");
    let update = pair.a.node_mut().admit_member(&mut OsRng, pair.org, &carol).unwrap();
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    pair.a.submit_commit_send(&mut OsRng, &update, carol.device_key, Some(sink_addr)).await.unwrap();
    let (_, _, captured) = sink.await.unwrap();
    let addr_b = pair.addr_b.clone();
    let (received, ()) = tokio::join!(
        async { tokio::time::timeout(NET, pair.b.receive_and_verify(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            deliver(addr_b, &captured).await;
        }
    );
    let refused = received.unwrap_err();
    assert!(matches!(refused, OrgNodeError::SenderNotListed { org_id } if org_id == pair.org), "{refused:?}");
    assert_eq!(pair.chain_b.reads(), reads_before, "no read");
    assert_eq!(pair.b.node_for_test().list_orgs()[0].epoch, Epoch::new(2), "nothing applied");
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_unheld_revocation_causes_no_read() {
    let mut pair = pair("order-unheld-revocation", true).await;
    admit_b(&mut pair).await.unwrap();
    // A revokes B; the notice A sends is captured by a sink.
    let member_b = member_id_of_b(&pair);
    let device_b = joiner_of(&pair.b, &pair.pid_b).device_key;
    let update = pair.a.node_mut().revoke_member(&mut OsRng, pair.org, member_b).unwrap();
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    pair.a.submit_commit_send(&mut OsRng, &update, device_b, Some(sink_addr)).await.unwrap();
    let (_, _, captured) = sink.await.unwrap();
    assert!(matches!(captured, WireMessage::Revocation(_)), "{captured:?}");
    // C holds no Organisation.
    let chain_c = FakeChain::sharing_slots_with(&pair.chain);
    let mut c = handle("order-unheld-revocation", "c", &chain_c);
    persona(&mut c, "carol");
    let addr_c = receiving_addr(&mut c).await;
    let (received, ()) = tokio::join!(
        async { tokio::time::timeout(NET, c.receive_and_self_delete_if_revoked(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            deliver(addr_c, &captured).await;
        }
    );
    let refused = received.unwrap_err();
    assert!(matches!(refused, OrgNodeError::RevocationNotHeld { org_id } if org_id == pair.org), "{refused:?}");
    assert_eq!(chain_c.reads(), 0, "no read");
}

/// A revokes B, which it admitted, and sends the notice to B's self-delete
/// receive; returns B's result (A's submission must succeed).
async fn revoke_b(pair: &mut Pair) -> Result<SelfDeleteOutcome, OrgNodeError> {
    let member_b = member_id_of_b(pair);
    let device_b = joiner_of(&pair.b, &pair.pid_b).device_key;
    let update = pair.a.node_mut().revoke_member(&mut OsRng, pair.org, member_b).unwrap();
    let Pair { a, b, addr_b, .. } = pair;
    let (received, sent) = tokio::join!(
        async { tokio::time::timeout(NET, b.receive_and_self_delete_if_revoked(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            a.submit_commit_send(&mut OsRng, &update, device_b, Some(addr_b.clone())).await
        }
    );
    sent.unwrap();
    received
}

/// A admits B, then revokes B and tells B's self-delete receive, which
/// reads once and signs its acknowledgement. Returns the pair and the
/// acknowledgement.
async fn revoked_acknowledgement(tag: &str) -> (Pair, Acknowledgement) {
    let mut pair = pair(tag, true).await;
    admit_b(&mut pair).await.unwrap();
    let received = revoke_b(&mut pair).await;
    let acknowledgements = match received.unwrap() {
        SelfDeleteOutcome::SelfDeleted { acknowledgements, .. } => acknowledgements,
        other => panic!("B was not removed: {other:?}"),
    };
    assert_eq!(pair.chain_b.reads(), 2, "the revocation, on the self-delete path: one read");
    let acknowledgement = acknowledgements.into_iter().next().unwrap();
    (pair, acknowledgement)
}

/// Deliver `acknowledgement` to A's `receive_and_verify`, from B's own
/// endpoint when `from_b`, else from a relay Device.
async fn deliver_acknowledgement_to_a(
    pair: &mut Pair,
    acknowledgement: Acknowledgement,
    from_b: bool,
) -> Result<ReceiveOutcome, OrgNodeError> {
    let addr_a = receiving_addr(&mut pair.a).await;
    let message = WireMessage::Acknowledgement(acknowledgement);
    let Pair { a, b, .. } = pair;
    let (received, ()) = tokio::join!(
        async { tokio::time::timeout(NET, a.receive_and_verify(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            if from_b {
                tokio::time::timeout(NET, b.node_for_test().endpoint().unwrap().send(addr_a, &message)).await.unwrap().unwrap();
            } else {
                deliver(addr_a, &message).await;
            }
        }
    );
    received
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_is_decided_with_no_read() {
    let (mut pair, acknowledgement) = revoked_acknowledgement("order-ack").await;
    let reads_before = pair.chain.reads();
    let outcome = deliver_acknowledgement_to_a(&mut pair, acknowledgement.clone(), true).await.unwrap();
    assert_eq!(outcome.acknowledged.map(|verified| verified.into_inner()), Some(acknowledgement));
    assert_eq!(pair.chain.reads(), reads_before, "decided against the store alone: no read");
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_from_another_device_is_refused_with_no_read() {
    let (mut pair, acknowledgement) = revoked_acknowledgement("order-ack-relayed").await;
    let reads_before = pair.chain.reads();
    let refused = deliver_acknowledgement_to_a(&mut pair, acknowledgement, false).await.unwrap_err();
    assert!(matches!(refused, OrgNodeError::AcknowledgementNotFromItsDevice { org_id } if org_id == pair.org), "{refused:?}");
    assert_eq!(pair.chain.reads(), reads_before, "no read");
}

/// B, admitted, receives A's revocation of it on the self-delete path while
/// `break_read` makes B's one read fail; returns the pair and B's result.
async fn revocation_with_a_broken_read(
    tag: &str,
    break_read: impl FnOnce(&FakeChain),
) -> (Pair, Result<SelfDeleteOutcome, OrgNodeError>) {
    let mut pair = pair(tag, true).await;
    admit_b(&mut pair).await.unwrap();
    let reads_before = pair.chain_b.reads();
    break_read(&pair.chain_b);
    let received = revoke_b(&mut pair).await;
    assert_eq!(pair.chain_b.reads(), reads_before + 1, "the revocation needed the chain: one read, made");
    (pair, received)
}

/// Nothing of B's was deleted: the record is held, in memory and on disk,
/// at the epoch B's admission left it.
fn b_still_holds_the_organisation(pair: &Pair, tag: &str) {
    let held = pair.b.node_for_test().list_orgs();
    assert_eq!(held.len(), 1, "the record is not deleted");
    assert_eq!((held[0].org_id, held[0].epoch), (pair.org, Epoch::new(2)), "and not changed");
    let persisted = reopened_service(tag, "b");
    assert_eq!(persisted.list_orgs().len(), 1, "the persisted record is not deleted");
    assert_eq!(persisted.list_orgs()[0].epoch, Epoch::new(2), "and not changed");
}

// The self-delete path's chain read (LLR-7pj5af): a revocation notice whose
// read fails is applied to nothing, so B deletes nothing and signs no
// acknowledgement. A false deletion is the harm the owner's ruling on the
// revocation hazard asks to make extremely unlikely.
// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_whose_read_fails_deletes_nothing() {
    let tag = "order-revocation-read-fails";
    let (pair, received) =
        revocation_with_a_broken_read(tag, |chain| chain.read_failing.store(true, Ordering::SeqCst)).await;
    let refused = received.unwrap_err();
    assert!(matches!(&refused, OrgNodeError::Chain(reason) if reason == "node unreachable"), "{refused:?}");
    b_still_holds_the_organisation(&pair, tag);
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_whose_read_is_refused_at_parse_deletes_nothing() {
    let tag = "order-revocation-parse-refused";
    let (pair, received) =
        revocation_with_a_broken_read(tag, |chain| chain.parse_refusing.store(true, Ordering::SeqCst)).await;
    let refused = received.unwrap_err();
    assert!(matches!(refused, OrgNodeError::InvalidOrgPublicKey), "{refused:?}");
    b_still_holds_the_organisation(&pair, tag);
}

// An acknowledgement received on the self-delete path is decided against the
// store alone (`Prepared::Done`): its outcome is returned with no read.
// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_on_the_self_delete_path_is_decided_with_no_read() {
    let (mut pair, acknowledgement) = revoked_acknowledgement("order-ack-self-delete").await;
    let reads_before = pair.chain.reads();
    let addr_a = receiving_addr(&mut pair.a).await;
    let message = WireMessage::Acknowledgement(acknowledgement.clone());
    let Pair { a, b, .. } = &mut pair;
    let (received, ()) = tokio::join!(
        async { tokio::time::timeout(NET, a.receive_and_self_delete_if_revoked(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            tokio::time::timeout(NET, b.node_for_test().endpoint().unwrap().send(addr_a, &message)).await.unwrap().unwrap();
        }
    );
    match received.unwrap() {
        SelfDeleteOutcome::Acknowledged(verified) => assert_eq!(verified.into_inner(), acknowledgement),
        other => panic!("not decided as an acknowledgement: {other:?}"),
    }
    assert_eq!(pair.chain.reads(), reads_before, "decided against the store alone: no read");
}
