#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The commit returns every recipient's body (S4, task T5 of
//! docs/plans/2026-10-06-org-io-transport.md; LLR-gdfqn9, PR-gnh3j2): built
//! inside the commit, from the record as committed and with the key that
//! commit stored, so a later commit cannot change a body already returned.

mod support;

use org_node::reconcile::Reconciled;
use org_node::service::{CommitOutcome, OutgoingMessage, StoreChange};
use org_node::store::{OrgRecord, ProvisionalChange, ProvisionalUpdate};
use org_node::test_fixtures::{device_key, member_key};
use org_node::wire::{decode_message, encode_message, WireMessage};
use org_node::{DevicePublicKey, Envelope, Epoch, Joiner, MemberId, OrgId};
use rand::rngs::OsRng;
use support::*;

/// A joiner no node holds a Persona for, from fixture keys.
fn joiner_from(seed: u8, handle: &str) -> Joiner {
    Joiner { handle: h(handle), name: nm("Test"), surname: sn("Joiner"), member_key: member_key(seed), device_key: device_key(seed + 1) }
}

/// Every Device `record` lists, in record order (Member, then slot).
fn devices_of(record: &OrgRecord) -> Vec<DevicePublicKey> {
    record.trie_members.iter().flat_map(|member| member.device_keys.iter().copied()).collect()
}

/// The recipients of `outcome`'s bodies, in order.
fn recipients(outcome: &CommitOutcome) -> Vec<DevicePublicKey> {
    outcome.outgoing.iter().map(|message| message.recipient).collect()
}

/// The Organisation information a commit of `update` on `before` sends: the
/// Envelope built from the update, the record as it stood before and the
/// key the update holds, encoded.
fn information_for(update: &ProvisionalUpdate, before: &OrgRecord) -> Vec<u8> {
    let ProvisionalChange::ChangeSet { change_set, org_private_key } = update.change_for_test().clone() else {
        panic!("a Change set")
    };
    encode_message(&WireMessage::OrgInformation {
        envelope: Envelope { org_id: before.org_id, parent_seq: update.seq, delta_bytes: change_set },
        record_snapshot: postcard::to_allocvec(&before.trie_members).unwrap(),
        org_private_key,
    })
    .unwrap()
}

/// Build `update` with `build`, write it to the chain (mock) and commit it
/// on `node`; returns the update, the record before and the outcome.
fn build_and_commit(
    node: &mut Node,
    chain: &org_node::test_fixtures::ChainSlots,
    org_id: OrgId,
    build: impl FnOnce(&mut Node) -> ProvisionalUpdate,
) -> (ProvisionalUpdate, OrgRecord, CommitOutcome) {
    let before = rec_of(node, org_id);
    let update = build(node);
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, before.epoch).unwrap();
    let outcome = node.commit_update(&mut OsRng, org_id).unwrap();
    (update, before, outcome)
}

/// A, B (on its own node) and C (a further Persona on A's node) are Members.
fn three_members(tag: &str) -> (Setup, Joiner) {
    let mut s = admit_b_directly(setup(tag));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    commit_admission(&mut s.svc_a, &s.chain, s.org_id, &joiner_c).unwrap();
    (s, joiner_c)
}

/// The MemberId listing `device` in `record`.
fn member_listing(record: &OrgRecord, device: DevicePublicKey) -> MemberId {
    record.trie_members.iter().find(|member| member.device_keys.contains(&device)).unwrap().id
}

// Normal: an admission's outcome holds one Organisation-information body for
// each Device the committed record lists other than the sender's, in record
// order, each the Envelope, the record before the commit and the key the
// commit stored.
// verifies: LLR-gdfqn9
#[test]
fn an_admission_sends_information_to_every_listed_device_but_the_senders() {
    let (mut s, joiner_c) = three_members("bodies-admission");
    let joiner_d = joiner_from(0x41, "dave");
    let (update, before, outcome) =
        build_and_commit(&mut s.svc_a, &s.chain, s.org_id, |a| a.admit_member(&mut OsRng, s.org_id, &joiner_d).unwrap());
    let committed = rec_of(&s.svc_a, s.org_id);
    let sender_device = s.svc_a.persona_public_keys(&s.pid_a).unwrap().1;
    let expected: Vec<DevicePublicKey> = devices_of(&committed).into_iter().filter(|device| *device != sender_device).collect();
    assert_eq!(recipients(&outcome), expected, "every listed Device but the sender's, in record order");
    let mut sorted = expected.clone();
    sorted.sort_by_key(|device| *device.as_bytes());
    let mut named = vec![s.joiner_b.device_key, joiner_c.device_key, joiner_d.device_key];
    named.sort_by_key(|device| *device.as_bytes());
    assert_eq!(sorted, named, "B, C and D");
    assert_eq!(outcome.sender, s.pid_a);
    let information = information_for(&update, &before);
    for message in &outcome.outgoing {
        assert_eq!(message.body, information, "the Envelope, the record before and the commit's key");
    }
    let WireMessage::OrgInformation { org_private_key, .. } = decode_message(&information).unwrap() else { panic!() };
    assert_eq!(&org_private_key, committed.org_private_key_for_test(), "the key the commit stored");
}

// Normal: a removal's outcome holds the listed Devices' information, then
// one revocation per removed Device: the notice naming that Device and its
// MemberId, with an absence proof that verifies against the committed root.
// verifies: LLR-gdfqn9, LLR-a8z7r5
#[test]
fn a_removal_sends_revocations_to_every_removed_device_after_the_listed_ones() {
    let (mut s, joiner_c) = three_members("bodies-removal");
    let b_id = member_listing(&rec_of(&s.svc_a, s.org_id), s.joiner_b.device_key);
    let (update, before, outcome) =
        build_and_commit(&mut s.svc_a, &s.chain, s.org_id, |a| a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap());
    assert_eq!(recipients(&outcome), vec![joiner_c.device_key, s.joiner_b.device_key], "listed first, then removed");
    assert_eq!(outcome.outgoing[0].body, information_for(&update, &before));
    let WireMessage::Revocation(notice) = decode_message(&outcome.outgoing[1].body).unwrap() else {
        panic!("a revocation for the removed Device")
    };
    assert_eq!((notice.org_id, notice.member_id, notice.device), (s.org_id, b_id, s.joiner_b.device_key));
    let committed = rec_of(&s.svc_a, s.org_id);
    assert_eq!(outcome.root, committed.root_hash);
    notice
        .proof
        .verify::<org_members::hasher::Blake3Hasher>(&committed.root_hash, &b_id, &s.joiner_b.device_key)
        .expect("the absence proof verifies against the committed root");
}

// Normal: a commit that removes this node forgets the Organisation and
// returns its acknowledgements and no bodies.
// verifies: LLR-gdfqn9, LLR-b27jr6
#[test]
fn a_commit_that_removes_this_node_returns_its_acknowledgements_and_no_bodies() {
    let mut s = admit_b_directly(setup("bodies-self-removal"));
    let own = s.joiner_b.device_key;
    let b_id = member_listing(&rec_of(&s.svc_b, s.org_id), own);
    let update = s.svc_b.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_b, s.org_id).epoch).unwrap();
    let state = s.chain.get(&s.org_id);
    let keychain = s.svc_b.keychain.clone();
    let (outcome, change) = s.svc_b.svc.commit_update(&mut OsRng, s.org_id, state, |keys| keychain.secrets(keys)).unwrap();
    assert!(matches!(change, StoreChange::Forgotten { device_key } if device_key == own));
    assert!(outcome.outgoing.is_empty(), "no bodies");
    assert_eq!(outcome.acknowledgements.len(), 1);
    assert_eq!(outcome.acknowledgements[0].device, own);
}

// Normal: a founder's genesis commit returns no recipients: its outcome is a
// receive outcome, which holds no body.
// verifies: LLR-gdfqn9
#[test]
fn commit_genesis_returns_no_recipients() {
    let chain = org_node::test_fixtures::ChainSlots::new();
    let mut a = Node::new(open_store("bodies-genesis", "a"), chain.clone());
    let pid = a.create_persona(&mut OsRng, h("alice"), nm("Alice"), sn("Smith")).unwrap();
    let update = a.create_organisation(&mut OsRng, &pid).unwrap();
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    // *Rewritten 2026-10-15 (S4 review round 2, finding-8):* the outcome's
    // Debug text was searched for fields `ReceiveOutcome` does not have, so
    // the test passed by construction. The outcome is now read: a receive
    // outcome (the type annotation: no `outgoing` field, so no body), the
    // founder admitted with nothing to deliver, and the record lists the
    // founder's Device alone, so no other Device could be a recipient.
    let (outcome, _write): (org_node::ReceiveOutcome, org_node::StoreWrite) =
        a.svc.commit_genesis(&mut OsRng, &pid, org_id, test_proxy(), chain.get(&org_id)).unwrap();
    assert!(matches!(outcome.kind, org_node::service::ReceiveKind::Admitted), "{outcome:?}");
    assert_eq!(outcome.org_id, org_id);
    assert!(outcome.acknowledgements.is_empty(), "nothing to deliver: {outcome:?}");
    let founder_device = a.svc.persona_public_keys(&pid).unwrap().1;
    assert_eq!(devices_of(&rec_of(&a, org_id)), vec![founder_device], "no Device but the founder's");
}

// Normal: a commit found at startup returns the same bodies a commit_update
// of the same update returns (LLR-fm38ww's one commit step).
// verifies: LLR-gdfqn9
#[test]
fn a_reconciled_commit_returns_the_same_bodies_as_commit_update() {
    let mut s = admit_b_directly(setup("bodies-reconcile"));
    let joiner_d = joiner_from(0x43, "dave");
    let before = rec_of(&s.svc_a, s.org_id);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_d).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, before.epoch).unwrap();
    let Reconciled::Committed { outcome, .. } = s.svc_a.reconcile(&mut OsRng, s.org_id).unwrap() else {
        panic!("the held update is committed")
    };
    let committed = rec_of(&s.svc_a, s.org_id);
    let sender_device = s.svc_a.persona_public_keys(&s.pid_a).unwrap().1;
    let expected: Vec<DevicePublicKey> = devices_of(&committed).into_iter().filter(|device| *device != sender_device).collect();
    assert_eq!(recipients(&outcome), expected);
    let information = information_for(&update, &before);
    assert!(outcome.outgoing.iter().all(|message| message.body == information), "commit_update's bodies");
}

// Abnormal (PR-gnh3j2): an earlier commit's bodies keep that commit's key
// after a later commit stored a new one.
// verifies: LLR-gdfqn9, PR-gnh3j2
#[test]
fn a_later_commit_cannot_change_an_earlier_body() {
    let mut s = admit_b_directly(setup("bodies-later"));
    let (first, _, first_outcome) = build_and_commit(&mut s.svc_a, &s.chain, s.org_id, |a| {
        a.admit_member(&mut OsRng, s.org_id, &joiner_from(0x45, "dave")).unwrap()
    });
    let (second, _, _) = build_and_commit(&mut s.svc_a, &s.chain, s.org_id, |a| {
        a.admit_member(&mut OsRng, s.org_id, &joiner_from(0x47, "erin")).unwrap()
    });
    assert_ne!(private_key_of(&first), private_key_of(&second), "the second commit stored a new key");
    let to_b = first_outcome.outgoing.iter().find(|message| message.recipient == s.joiner_b.device_key).unwrap();
    let WireMessage::OrgInformation { envelope, org_private_key, .. } = decode_message(&to_b.body).unwrap() else {
        panic!("Organisation information")
    };
    assert_eq!(envelope.parent_seq, first.seq);
    assert_eq!(org_private_key, private_key_of(&first), "the first commit's key");
}

/// Every rendering of an 8-byte window of `bytes` a `Debug` could print:
/// the decimal list and lower-case hex.
fn renderings(bytes: &[u8]) -> Vec<String> {
    bytes
        .windows(8)
        .flat_map(|window| {
            let decimal = format!("{window:?}");
            let hex: String = window.iter().map(|byte| format!("{byte:02x}")).collect();
            [decimal.trim_matches(|c| c == '[' || c == ']').to_string(), hex]
        })
        .collect()
}

// Abnormal: the outcome's `Debug` prints recipients and lengths, never a
// body byte or the key a body carries.
// verifies: LLR-gdfqn9
#[test]
fn outgoing_debug_shows_no_body_byte() {
    let mut s = admit_b_directly(setup("bodies-debug"));
    let (update, _, outcome) = build_and_commit(&mut s.svc_a, &s.chain, s.org_id, |a| {
        a.admit_member(&mut OsRng, s.org_id, &joiner_from(0x49, "dave")).unwrap()
    });
    // The bodies are checked against the bodies' rendering: the outcome's
    // other fields print public values (the Organisation id) a body also
    // holds.
    let rendered = format!("{outcome:?}");
    let bodies = format!("{:?}", outcome.outgoing);
    assert!(!outcome.outgoing.is_empty());
    for message in &outcome.outgoing {
        assert!(bodies.contains(&format!("body_len: {}", message.body.len())), "the body's length: {bodies}");
        for window in renderings(&message.body) {
            assert!(!bodies.contains(&window), "a body window {window} in {bodies}");
        }
    }
    for window in renderings(private_key_of(&update).expose_secret()) {
        assert!(!rendered.contains(&window), "the key's bytes {window} in {rendered}");
    }
}

/// A trait probe for `Clone`, resolved at compile time (the probe of
/// `key_custody.rs`, the pattern of the `impls` crate).
struct CloneProbe<T: ?Sized>(std::marker::PhantomData<T>);

trait NotCloneFallback {
    const IS_CLONE: bool = false;
}

impl<T: ?Sized> NotCloneFallback for CloneProbe<T> {}

// `Clone` implies `Sized`, so this impl takes no `?Sized` (clippy, 2026-10-14,
// S4 task T18); the probe's answer for every type tested below is unchanged.
impl<T: Clone> CloneProbe<T> {
    const IS_CLONE: bool = true;
}

// Abnormal: neither the outcome nor a body can be cloned; the probe is
// checked against a type whose answer is known.
// verifies: LLR-gdfqn9
#[test]
#[allow(
    clippy::assertions_on_constants,
    reason = "the probe's answers are compile-time constants by design; asserting them at run time is the test"
)]
fn outgoing_types_are_not_clone() {
    assert!(CloneProbe::<Vec<u8>>::IS_CLONE, "the probe sees a Clone type");
    assert!(!CloneProbe::<OutgoingMessage>::IS_CLONE, "OutgoingMessage is not Clone");
    assert!(!CloneProbe::<CommitOutcome>::IS_CLONE, "CommitOutcome is not Clone");
}

// Abnormal (review round 1 of S4, finding-12): a refused commit — no chain
// state, or the state from before the chain write — returns no outcome, so
// no body, and changes nothing in memory or on the disk; the update is still
// held, and the commit against the state after the write returns its bodies.
// *Assertion removed 2026-10-15 (S4 TEST review round 1, finding-9):* it
// also compared the harness disk before and after each refusal, but the
// test calls `OrgService::commit_update` directly, past the harness that
// writes a returned change, and a refusal returns no change to write, so
// that comparison could not fail. "Nothing written" holds by type at
// org-node's seam (an `Err` carries no `StoreChange`); the in-memory
// comparison, which can fail, stays.
// verifies: LLR-gdfqn9
#[test]
fn a_refused_commit_returns_no_bodies_and_changes_nothing() {
    let tag = "bodies-refused";
    let mut s = admit_b_directly(setup(tag));
    let before_write = s.chain.get(&s.org_id);
    let joiner = joiner_from(0x4d, "erin");
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let pid_a = s.pid_a.clone();
    let in_memory = format!("{:?}", s.svc_a.store_data(&pid_a).unwrap());

    for (state, expected) in [
        (None, org_node::OrgNodeError::OrgNotOnChain),
        (before_write, org_node::OrgNodeError::NoProvisionalUpdate),
    ] {
        let refused = s.svc_a.svc.commit_update(&mut OsRng, s.org_id, state, |_| Vec::new());
        match refused {
            Err(error) => assert_eq!(error, expected),
            Ok((outcome, _)) => panic!("committed, {} bodies", outcome.outgoing.len()),
        }
        assert_eq!(format!("{:?}", s.svc_a.store_data(&pid_a).unwrap()), in_memory, "nothing changed in memory");
    }

    let (outcome, _) = s.svc_a.svc.commit_update(&mut OsRng, s.org_id, s.chain.get(&s.org_id), |_| Vec::new()).unwrap();
    assert!(recipients(&outcome).contains(&joiner.device_key), "the held update commits and returns its bodies");
}

// Normal: building the bodies writes nothing beyond the commit's one store
// change and sends nothing: no endpoint is bound.
// *Assertion removed 2026-10-15 (S4 TEST review round 1, finding-9):* it
// also compared the harness disk before and after the commit, which the
// test calls on `OrgService` directly, past the harness that writes a
// returned change, so that comparison could not fail. What the commit
// writes is what it returns: one `StoreChange`, checked below; org-node
// writes no file (`absences::org_node_is_sans_io`).
// verifies: LLR-gdfqn9
#[test]
fn building_the_bodies_writes_and_sends_nothing() {
    let mut s = admit_b_directly(setup("bodies-nothing"));
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_from(0x4b, "dave")).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let state = s.chain.get(&s.org_id);
    let (outcome, change) = s.svc_a.svc.commit_update(&mut OsRng, s.org_id, state, |_| Vec::new()).unwrap();
    assert!(!outcome.outgoing.is_empty());
    assert!(matches!(change, StoreChange::Sealed(_)), "one store change, sealed");
    // *2026-10-14 (S4, task T14):* "nothing bound, nothing sent" is an
    // absence now: org-node has no endpoint (`absences::org_node_is_sans_io`).
    assert_eq!(rec_of(&s.svc_a, s.org_id).epoch, Epoch::new(3));
}
