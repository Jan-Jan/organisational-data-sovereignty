#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The receive sequence (LLR-7pj5af): org-io takes one message and its
//! sender from org-node's transport, hands both to org-node's chain-free
//! phase, reads the chain only when that phase returns a pending value, and
//! then exactly once; a refusal or an acknowledgement (decided against the
//! store alone) reads nothing; a failed read or a refused parse applies
//! nothing. Each receiving handle reads through its own `FakeChain` counter
//! over the slots every handle shares, so its reads are its own.
//!
//! *S4 task T4, 2026-10-13:* there is one receive path (`check_received`,
//! then `commit_checked`); the tests that drove the self-delete path
//! (`receive_and_self_delete_if_revoked`, deleted) drive it.
//!
//! *S4 task T11, 2026-10-13:* the one receive is `next_received` (which
//! replaced `receive_and_verify`), on `&self`; its error separates
//! org-node's refusal from a failed chain read (`ReceiveError`).

use std::sync::atomic::Ordering;
use std::time::Duration;

use org_node::revocation::Acknowledgement;
use org_node::wire::WireMessage;
use org_node::{Epoch, MemberId, OrgId, OrgNodeError, PersonaId, ReceiveKind, ReceiveOutcome};
use org_io::test_support::FakeChain;
use org_io::{OrgIo, ReceiveError, Received};
use rand::rngs::OsRng;

mod common;
use common::handles::{
    at, deliver, handle, joiner, joiner_of, persona, receiving_addr, reopened_service, spawn_recv_one, spawn_recv_one_as,
    NET,
};

// *S4 task T12, 2026-10-13:* `submit_commit_send` names no recipient (the
// commit names every one) and takes org-io's `PeerAddr` (`at`); a Loopback
// send goes only to an address whose identity is a recipient's Device key,
// so each sink below is bound as the recipient's Device.

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
    /// B's endpoint, held here because a removed Persona is no longer the
    /// node's (S4, task T10: the endpoints are org-io's, one per Persona).
    endpoint_b: std::sync::Arc<org_io::transport::OrgEndpoint>,
}

async fn pair(tag: &str, b_expects_the_admission: bool) -> Pair {
    let chain = FakeChain::new();
    let a = handle(tag, "a", &chain);
    let pid_a = persona(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &pid_a).await.unwrap();
    // *S4 task T10, 2026-10-13:* A's endpoint is bound from its Persona's
    // keychain item on its first send (org-io's endpoints).
    let chain_b = FakeChain::sharing_slots_with(&chain);
    let b = handle(tag, "b", &chain_b);
    let pid_b = persona(&b, "bob").await;
    if b_expects_the_admission {
        b.expect_admission(&mut OsRng, &pid_b, org).await.unwrap();
    }
    let addr_b = receiving_addr(&b).await;
    let endpoint_b = b.endpoint_for_test(&pid_b).await.unwrap();
    Pair { a, chain, b, chain_b, org, pid_b, addr_b, endpoint_b }
}

/// A admits B and sends the admission to B's `next_received`; returns
/// B's result.
async fn admit_b(pair: &mut Pair) -> Result<ReceiveOutcome, ReceiveError> {
    let bob = joiner_of(&pair.b, &pair.pid_b);
    let update = pair.a.admit_member(&mut OsRng, pair.org, &bob).await.unwrap();
    let Pair { a, b, addr_b, .. } = pair;
    let (received, _sent) = tokio::join!(
        async { tokio::time::timeout(NET, b.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            a.submit_commit_send(&mut OsRng, &update, at(addr_b.clone())).await
        }
    );
    received.map(committed)
}

/// org-node's refusal inside the handle's error. *S4 task T9, 2026-10-13:*
/// a store change the receive could not write is reported apart; none of
/// these tests writes to a failing store. *Review round 2, finding 7
/// (2026-10-15):* a refusal and a failed read are told apart, each test
/// asserting the one it expects (`ReceiveError::Refused` here,
/// `ReceiveError::Chain` in `failed_read`).
fn node_refusal(error: ReceiveError) -> OrgNodeError {
    match error {
        ReceiveError::Refused(refused) => refused,
        other => panic!("not org-node's refusal: {other}"),
    }
}

/// The failed chain read, or the refused parse of the state read, inside
/// the handle's error: nothing was committed.
fn failed_read(error: ReceiveError) -> OrgNodeError {
    match error {
        ReceiveError::Chain(failed) => failed,
        other => panic!("not a failed chain read: {other}"),
    }
}

/// The outcome of a receive that commits; an acknowledgement there is a
/// failed test.
fn committed(received: Received) -> ReceiveOutcome {
    match received {
        Received::Committed(outcome) => outcome,
        Received::Acknowledged(verified) => panic!("decided as an acknowledgement: {verified:?}"),
    }
}

/// B's MemberId in A's record.
fn member_id_of_b(pair: &Pair) -> MemberId {
    let device_b = joiner_of(&pair.b, &pair.pid_b).device_key;
    let record = pair.a.node_for_test().list_orgs().into_iter().find(|o| o.org_id == pair.org).cloned().unwrap();
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
    let update = pair.a.admit_member(&mut OsRng, pair.org, &carol).await.unwrap();
    let Pair { a, b, addr_b, .. } = &mut pair;
    let (received, sent) = tokio::join!(
        async { tokio::time::timeout(NET, b.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            a.submit_commit_send(&mut OsRng, &update, at(addr_b.clone())).await
        }
    );
    sent.unwrap();
    assert_eq!(committed(received.unwrap()).epoch, Epoch::new(3));
    assert_eq!(pair.chain_b.reads(), 2, "the held Organisation's update: one more read");
    assert_eq!(pair.b.node_for_test().list_orgs()[0].epoch, Epoch::new(3), "applied");
}

// The first admission is accepted from any sender: guarded by the
// expectation before the read and by the chain after it.
// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_relayed_by_any_device_is_read_once_then_applied() {
    let pair = pair("order-any-sender", true).await;
    let bob = joiner_of(&pair.b, &pair.pid_b);
    let update = pair.a.admit_member(&mut OsRng, pair.org, &bob).await.unwrap();
    let (sink_addr, sink) = spawn_recv_one_as(&bob.device_key).await;
    pair.a.submit_commit_send(&mut OsRng, &update, at(sink_addr)).await.unwrap();
    let (_, _, captured) = sink.await.unwrap();
    let addr_b = pair.addr_b.clone();
    let (received, ()) = tokio::join!(
        async { tokio::time::timeout(NET, pair.b.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            deliver(addr_b, &captured).await;
        }
    );
    assert_eq!(committed(received.unwrap()).epoch, Epoch::new(2));
    assert_eq!(pair.chain_b.reads(), 1, "one read, after the expectation gate passed");
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_message_the_chain_free_phase_refuses_causes_no_read() {
    // B does not expect the admission: AdmissionNotExpected, before any read.
    let mut pair = pair("order-not-expected", false).await;
    let refused = node_refusal(admit_b(&mut pair).await.unwrap_err());
    assert!(matches!(refused, OrgNodeError::AdmissionNotExpected { org_id } if org_id == pair.org), "{refused:?}");
    assert_eq!(pair.chain_b.reads(), 0, "no read");
    assert!(pair.b.node_for_test().list_orgs().is_empty());
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_read_applies_nothing() {
    let mut pair = pair("order-read-fails", true).await;
    pair.chain_b.read_failing.store(true, Ordering::SeqCst);
    let refused = failed_read(admit_b(&mut pair).await.unwrap_err());
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
    let refused = failed_read(admit_b(&mut pair).await.unwrap_err());
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
    let update = pair.a.admit_member(&mut OsRng, pair.org, &carol).await.unwrap();
    // Carol's Device, `joiner(0x71, ..)`'s, is the seed `[0x72; 32]`'s.
    let (sink_addr, sink) = spawn_recv_one([0x72; 32]).await;
    pair.a.submit_commit_send(&mut OsRng, &update, at(sink_addr)).await.unwrap();
    let (_, _, captured) = sink.await.unwrap();
    let addr_b = pair.addr_b.clone();
    let (received, ()) = tokio::join!(
        async { tokio::time::timeout(NET, pair.b.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            deliver(addr_b, &captured).await;
        }
    );
    let refused = node_refusal(received.unwrap_err());
    assert!(matches!(refused, OrgNodeError::SenderNotListed { org_id } if org_id == pair.org), "{refused:?}");
    assert_eq!(pair.chain_b.reads(), reads_before, "no read");
    assert_eq!(pair.b.node_for_test().list_orgs()[0].epoch, Epoch::new(2), "nothing applied");
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_unheld_revocation_causes_no_read() {
    let mut pair = pair("order-unheld-revocation", true).await;
    admit_b(&mut pair).await.unwrap();
    // A revokes B; the notice A sends is captured. *S4 task T12, 2026-10-13:*
    // captured at B, from its receive channel before org-node sees it
    // (`next_inbound_for_test`), in place of a sink: a sink would have to
    // share B's Device key, and A's endpoint reaches that identity at the
    // address it already learnt, B's.
    let member_b = member_id_of_b(&pair);
    let update = pair.a.revoke_member(&mut OsRng, pair.org, member_b).await.unwrap();
    let mut rng = OsRng;
    let (sent, inbound) = tokio::join!(pair.a.submit_commit_send(&mut rng, &update, at(pair.addr_b.clone())), async {
        tokio::time::timeout(NET, pair.b.next_inbound_for_test()).await.unwrap().unwrap()
    });
    sent.unwrap();
    let captured = org_node::wire::decode_message(&inbound.1).unwrap();
    assert!(matches!(captured, WireMessage::Revocation(_)), "{captured:?}");
    // C holds no Organisation.
    let chain_c = FakeChain::sharing_slots_with(&pair.chain);
    let c = handle("order-unheld-revocation", "c", &chain_c);
    persona(&c, "carol").await;
    let addr_c = receiving_addr(&c).await;
    let (received, ()) = tokio::join!(
        async { tokio::time::timeout(NET, c.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            deliver(addr_c, &captured).await;
        }
    );
    let refused = node_refusal(received.unwrap_err());
    assert!(matches!(refused, OrgNodeError::RevocationNotHeld { org_id } if org_id == pair.org), "{refused:?}");
    assert_eq!(chain_c.reads(), 0, "no read");
}

/// A revokes B, which it admitted, and sends the notice to B's receive;
/// returns B's result (A's submission must succeed).
async fn revoke_b(pair: &mut Pair) -> Result<ReceiveOutcome, ReceiveError> {
    let member_b = member_id_of_b(pair);
    let update = pair.a.revoke_member(&mut OsRng, pair.org, member_b).await.unwrap();
    let Pair { a, b, addr_b, .. } = pair;
    let (received, sent) = tokio::join!(
        async { tokio::time::timeout(NET, b.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            a.submit_commit_send(&mut OsRng, &update, at(addr_b.clone())).await
        }
    );
    sent.unwrap();
    received.map(committed)
}

/// A admits B, then revokes B and tells B's receive, which reads once and
/// signs its acknowledgement. Returns the pair and the acknowledgement.
async fn revoked_acknowledgement(tag: &str) -> (Pair, Acknowledgement) {
    let mut pair = pair(tag, true).await;
    admit_b(&mut pair).await.unwrap();
    let received = revoke_b(&mut pair).await.unwrap();
    assert_eq!(received.kind, ReceiveKind::Revoked, "B was not removed: {received:?}");
    let acknowledgements = received.acknowledgements;
    assert_eq!(pair.chain_b.reads(), 2, "the revocation: one read");
    let acknowledgement = acknowledgements.into_iter().next().unwrap();
    (pair, acknowledgement)
}

/// Deliver `acknowledgement` to A's `next_received`, from B's own
/// endpoint when `from_b`, else from a relay Device.
async fn deliver_acknowledgement_to_a(
    pair: &mut Pair,
    acknowledgement: Acknowledgement,
    from_b: bool,
) -> Result<Received, OrgNodeError> {
    let addr_a = receiving_addr(&pair.a).await;
    let message = WireMessage::Acknowledgement(acknowledgement);
    let Pair { a, endpoint_b, .. } = pair;
    let (received, ()) = tokio::join!(
        async { tokio::time::timeout(NET, a.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            if from_b {
                tokio::time::timeout(NET, endpoint_b.send(addr_a, &message)).await.unwrap().unwrap();
            } else {
                deliver(addr_a, &message).await;
            }
        }
    );
    received.map_err(node_refusal)
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_is_decided_with_no_read() {
    let (mut pair, acknowledgement) = revoked_acknowledgement("order-ack").await;
    let reads_before = pair.chain.reads();
    let received = deliver_acknowledgement_to_a(&mut pair, acknowledgement.clone(), true).await.unwrap();
    let Received::Acknowledged(verified) = received else { panic!("not decided as an acknowledgement: {received:?}") };
    assert_eq!(verified.into_inner(), acknowledgement);
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

/// B, admitted, receives A's revocation of it while `break_read` makes B's
/// one read fail; returns the pair and B's result.
async fn revocation_with_a_broken_read(
    tag: &str,
    break_read: impl FnOnce(&FakeChain),
) -> (Pair, Result<ReceiveOutcome, ReceiveError>) {
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
    let node_b = pair.b.node_for_test();
    let held = node_b.list_orgs();
    assert_eq!(held.len(), 1, "the record is not deleted");
    assert_eq!((held[0].org_id, held[0].epoch), (pair.org, Epoch::new(2)), "and not changed");
    let persisted = reopened_service(tag, "b", &pair.b);
    assert_eq!(persisted.list_orgs().len(), 1, "the persisted record is not deleted");
    assert_eq!(persisted.list_orgs()[0].epoch, Epoch::new(2), "and not changed");
}

// The self-delete path's chain read (LLR-7pj5af; *S4 T4:* the one receive
// path's): a revocation notice whose
// read fails is applied to nothing, so B deletes nothing and signs no
// acknowledgement. A false deletion is the harm the owner's ruling on the
// revocation hazard asks to make extremely unlikely.
// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_whose_read_fails_deletes_nothing() {
    let tag = "order-revocation-read-fails";
    let (pair, received) =
        revocation_with_a_broken_read(tag, |chain| chain.read_failing.store(true, Ordering::SeqCst)).await;
    let refused = failed_read(received.unwrap_err());
    assert!(matches!(&refused, OrgNodeError::Chain(reason) if reason == "node unreachable"), "{refused:?}");
    b_still_holds_the_organisation(&pair, tag);
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_whose_read_is_refused_at_parse_deletes_nothing() {
    let tag = "order-revocation-parse-refused";
    let (pair, received) =
        revocation_with_a_broken_read(tag, |chain| chain.parse_refusing.store(true, Ordering::SeqCst)).await;
    let refused = failed_read(received.unwrap_err());
    assert!(matches!(refused, OrgNodeError::InvalidOrgPublicKey), "{refused:?}");
    b_still_holds_the_organisation(&pair, tag);
}

// An acknowledgement received on the self-delete path is decided against the
// store alone (`Prepared::Done`): its outcome is returned with no read.
// *S4 T4, 2026-10-13:* the self-delete path folded onto the one receive
// path; `check_received` decides the acknowledgement (`Checked::Acknowledged`).
// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_on_the_self_delete_path_is_decided_with_no_read() {
    let (mut pair, acknowledgement) = revoked_acknowledgement("order-ack-self-delete").await;
    let reads_before = pair.chain.reads();
    let addr_a = receiving_addr(&pair.a).await;
    let message = WireMessage::Acknowledgement(acknowledgement.clone());
    let Pair { a, endpoint_b, .. } = &mut pair;
    let (received, ()) = tokio::join!(
        async { tokio::time::timeout(NET, a.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            tokio::time::timeout(NET, endpoint_b.send(addr_a, &message)).await.unwrap().unwrap();
        }
    );
    match received.unwrap() {
        Received::Acknowledged(verified) => assert_eq!(verified.into_inner(), acknowledgement),
        other => panic!("not decided as an acknowledgement: {other:?}"),
    }
    assert_eq!(pair.chain.reads(), reads_before, "decided against the store alone: no read");
}
