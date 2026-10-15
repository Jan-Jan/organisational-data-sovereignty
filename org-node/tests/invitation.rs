#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The Invite and the Invite reply (SDD-xshv5b; S4, task T6 of
//! docs/plans/2026-10-06-org-io-transport.md): the Blobs, built and parsed
//! here, and their bookkeeping in the sealed store — the outstanding
//! (Organisation, invite identifier) pairs and the expectation a reply
//! records. org-io runs the exchange; these tests play org-io, holding each
//! returned write as "the disk" (`support::Node`).
//!
//! The three LLR-b7wgpf tests moved here from the app
//! (`app/src-tauri/tests/invitation.rs`) with REQ-yazum3 and LLR-b7wgpf
//! (ruling A). *Adapted 2026-10-13 (S4, task T6):* a refusal is
//! `OrgNodeError::InvalidField` naming the field (LLR-8bum44), so the field is
//! read from the error, not from the start of a message string; and the
//! third test, which drove the app's `admit_reply`, shows the same thing on
//! org-node's side — a Blob that does not parse yields no reply to act on,
//! and nothing is settled or written.

mod support;

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::invitation::{Invite, InviteId, InviteReply};
use org_node::store::ExpectedAdmission;
use org_node::test_fixtures::{device_key, member_key, ChainSlots};
use org_node::PersonaId;
use rand::rngs::OsRng;
use support::*;

/// The `field` of an `InvalidField` refusal.
fn refused_field(error: OrgNodeError) -> &'static str {
    match error {
        OrgNodeError::InvalidField { field, .. } => field,
        other => panic!("expected InvalidField, got {other:?}"),
    }
}

/// Each Persona store's plaintext, in store order: what "byte-identical"
/// compares.
fn plaintexts(node: &Node) -> Vec<Vec<u8>> {
    node.list_personas()
        .iter()
        .map(|persona| postcard::to_allocvec(node.store_data(&persona.persona_id).unwrap()).unwrap())
        .collect()
}

/// A node over a fresh storage whose one Persona, alice, founded an
/// Organisation on `chain`.
fn founder(tag: &str, chain: &ChainSlots) -> (Node, PersonaId, OrgId) {
    let mut node = Node::new(open_store(tag, "a"), chain.clone());
    let pid = node.create_persona(&mut OsRng, h("alice"), nm("Alice"), sn("Smith")).unwrap();
    let org_id = found(&mut node, chain, &pid);
    (node, pid, org_id)
}

/// A node over a fresh storage with one unbound Persona, bob.
fn joiner(tag: &str, chain: &ChainSlots) -> (Node, PersonaId) {
    let mut node = Node::new(open_store(tag, "b"), chain.clone());
    let pid = node.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    (node, pid)
}

/// An Invite for `org_id`, as another node would have built it.
fn invite_for(org_id: OrgId) -> Invite {
    Invite {
        org_name: "Acme".into(),
        org_id,
        invitee_name: "Bob Builder".into(),
        inviter_device_keys: vec![device_key(5)],
        invite_id: InviteId::new([3; 32]),
    }
}

/// `node`'s Invite for `org_id`, its write kept on the disk.
fn build_invite(node: &mut Node, org_id: OrgId) -> Invite {
    let (invite, write) = node.svc.build_invite(&mut OsRng, org_id, "Acme Co-op", "Bob Builder").unwrap();
    node.disk.write(write);
    invite
}

/// `node`'s reply to `invite` as `persona_id`, its write kept on the disk.
fn build_reply(node: &mut Node, invite: &Invite, persona_id: &PersonaId) -> Result<InviteReply, OrgNodeError> {
    let (reply, write) = node.svc.build_reply(&mut OsRng, invite, persona_id)?;
    node.disk.write(write);
    Ok(reply)
}

// ---------------------------------------------------------------------
// The Blobs (moved from the app).
// ---------------------------------------------------------------------

// verifies: LLR-b7wgpf
#[test]
fn an_invite_and_a_reply_that_parse_encode_back_to_the_same_blob() {
    let (mk, dk) = (*member_key(8).as_bytes(), *device_key(7).as_bytes());
    let invite = Invite::wire_for_test("Acme", &[1; 20], "Bob", &[dk], &[3; 32]);
    assert_eq!(Invite::parse(&invite).unwrap().encode().unwrap(), invite);
    let reply = InviteReply::wire_for_test(&[1; 20], &[3; 32], &mk, &dk, "bob", "Bob", "Builder");
    assert_eq!(InviteReply::parse(&reply).unwrap().encode().unwrap(), reply);
}

// verifies: LLR-b7wgpf
#[test]
fn an_invite_or_reply_that_does_not_parse_is_refused_naming_the_field() {
    let (mk, dk) = (*member_key(8).as_bytes(), *device_key(7).as_bytes());
    for (blob, field) in [
        (Invite::wire_for_test("O", &[1; 19], "N", &[dk], &[3; 32]), "invite.org_id"),
        (Invite::wire_for_test("O", &[1; 20], "N", &[[0xff; 32]], &[3; 32]), "invite.inviter_device_keys"),
        (Invite::wire_for_test("O", &[1; 20], "N", &[dk], &[3; 31]), "invite.invite_id"),
        ("not base64 !".to_string(), "invite"),
    ] {
        assert_eq!(refused_field(Invite::parse(&blob).unwrap_err()), field);
    }
    let reply = |org: &[u8], id: &[u8], mk: [u8; 32], dk: [u8; 32], handle: &str, name: &str, surname: &str| {
        InviteReply::wire_for_test(org, id, &mk, &dk, handle, name, surname)
    };
    let long = "a".repeat(129);
    for (blob, field) in [
        (reply(&[1; 21], &[3; 32], mk, dk, "bob", "Bob", "Builder"), "reply.org_id"),
        (reply(&[1; 20], &[3; 31], mk, dk, "bob", "Bob", "Builder"), "reply.invite_id"),
        (reply(&[1; 20], &[3; 32], [0xff; 32], dk, "bob", "Bob", "Builder"), "reply.member_key"),
        (reply(&[1; 20], &[3; 32], mk, [0xff; 32], "bob", "Bob", "Builder"), "reply.device_key"),
        (reply(&[1; 20], &[3; 32], mk, dk, "Bob", "Bob", "Builder"), "reply.handle"),
        (reply(&[1; 20], &[3; 32], mk, dk, "bob", &long, "Builder"), "reply.name"),
        (reply(&[1; 20], &[3; 32], mk, dk, "bob", "Bob", &long), "reply.surname"),
        ("not base64 !".to_string(), "reply"),
    ] {
        assert_eq!(refused_field(InviteReply::parse(&blob).unwrap_err()), field);
    }
}

// *Deleted 2026-10-15 (S4 TEST review round 2, notes):*
// `a_reply_that_does_not_parse_acts_on_nothing`. After round 1 (finding-8)
// it asserted only that `InviteReply::parse("not base64 !")` is refused
// naming `reply`, the last case of the field-refusal test above, so it was a
// pure duplicate. "Acts on nothing" holds by type at org-node's seam:
// `build_reply`, `check_reply` and `settle` take a parsed `Invite`, a parsed
// `InviteReply` or its fields, never Blob text.

// ---------------------------------------------------------------------
// build_invite (LLR-nr8df3).
// ---------------------------------------------------------------------

// verifies: LLR-nr8df3
#[test]
fn build_invite_records_the_pair_and_carries_the_bound_personas_device_key() {
    let chain = ChainSlots::new();
    let (mut a, pid, org) = founder("issue", &chain);
    let (invite, write) = a.svc.build_invite(&mut OsRng, org, "Acme Co-op", "Bob Builder").unwrap();
    let device = a.persona_public_keys(&pid).unwrap().1;
    assert_eq!((invite.org_name.as_str(), invite.org_id, invite.invitee_name.as_str()), ("Acme Co-op", org, "Bob Builder"));
    assert_eq!(invite.inviter_device_keys, vec![device], "the bound Persona's Device");
    assert_eq!(write.device_key, device, "the write is that Persona's store");
    a.disk.write(write);
    assert_eq!(a.store_data(&pid).unwrap().outstanding_invites, vec![(org, invite.invite_id)]);
    assert_eq!(
        party("issue", "a").disk.unseal(&a.keychain)[0].data().outstanding_invites,
        vec![(org, invite.invite_id)],
        "the pair is in the sealed store"
    );
    let other = build_invite(&mut a, org);
    assert_ne!(other.invite_id, invite.invite_id, "drawn at random");
    assert_eq!(a.store_data(&pid).unwrap().outstanding_invites, vec![(org, invite.invite_id), (org, other.invite_id)]);
}

// verifies: LLR-nr8df3
#[test]
fn build_invite_refuses_an_organisation_no_persona_is_bound_to() {
    let chain = ChainSlots::new();
    let (mut a, _pid, _org) = founder("issue-none", &chain);
    a.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Jones")).unwrap();
    let (before, on_disk) = (plaintexts(&a), a.disk.bytes());
    let elsewhere = OrgId::new([9; 20]);
    assert_eq!(a.svc.build_invite(&mut OsRng, elsewhere, "X", "Y").unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert_eq!(plaintexts(&a), before, "nothing changed");
    assert_eq!(a.disk.bytes(), on_disk, "nothing written");
}

// ---------------------------------------------------------------------
// build_reply (LLR-7q9g9z).
// ---------------------------------------------------------------------

// verifies: LLR-7q9g9z
#[test]
fn build_reply_records_the_expectation_once_and_carries_the_personas_keys() {
    let chain = ChainSlots::new();
    let (mut a, _, org) = founder("reply", &chain);
    let invite = build_invite(&mut a, org);
    let (mut b, pid) = joiner("reply", &chain);
    let (reply, write) = b.svc.build_reply(&mut OsRng, &invite, &pid).unwrap();
    let (mk, dk) = b.persona_public_keys(&pid).unwrap();
    assert_eq!((reply.member_key, reply.device_key, reply.handle.as_str()), (mk, dk, "bob"));
    assert_eq!((reply.name.to_string(), reply.surname.to_string()), ("Bob".to_string(), "Builder".to_string()));
    assert_eq!((reply.org_id, reply.invite_id), (org, invite.invite_id));
    assert_eq!(write.device_key, dk, "the write is the replying Persona's store");
    b.disk.write(write);
    assert_eq!(b.expected_admissions(), vec![ExpectedAdmission { org_id: org }]);
    assert_eq!(reopen_store("reply", "b").data().expected_admissions, vec![ExpectedAdmission { org_id: org }]);
    build_reply(&mut b, &invite, &pid).unwrap();
    assert_eq!(b.expected_admissions(), vec![ExpectedAdmission { org_id: org }], "at most once");
}

// verifies: LLR-7q9g9z
#[test]
fn build_reply_refuses_a_bound_persona() {
    let chain = ChainSlots::new();
    let (mut a, own, org) = founder("bound-reply", &chain);
    let invite = build_invite(&mut a, org);
    // B's Persona founded B's own Organisation, so it is bound.
    let (mut b, pid) = joiner("bound-reply", &chain);
    found(&mut b, &chain, &pid);
    let (before, on_disk) = (plaintexts(&b), b.disk.bytes());
    assert_eq!(build_reply(&mut b, &invite, &pid).unwrap_err(), OrgNodeError::PersonaAlreadyBound { persona_id: pid });
    assert_eq!((plaintexts(&b), b.disk.bytes()), (before, on_disk), "nothing written");
    // The inviter's own Persona is bound to the very Organisation invited to.
    let (before, on_disk) = (plaintexts(&a), a.disk.bytes());
    assert_eq!(
        build_reply(&mut a, &invite, &own).unwrap_err(),
        OrgNodeError::PersonaAlreadyBound { persona_id: own.clone() }
    );
    assert_eq!((plaintexts(&a), a.disk.bytes()), (before, on_disk), "nothing written");
}

// verifies: LLR-7q9g9z
#[test]
fn build_reply_refuses_an_unknown_persona() {
    let chain = ChainSlots::new();
    let (mut b, _) = joiner("unknown-reply", &chain);
    let (before, on_disk) = (plaintexts(&b), b.disk.bytes());
    assert!(build_reply(&mut b, &invite_for(OrgId::new([0x42; 20])), &PersonaId::new("nobody".into())).is_err());
    assert!(b.expected_admissions().is_empty(), "an unknown Persona declares nothing");
    assert_eq!((plaintexts(&b), b.disk.bytes()), (before, on_disk), "nothing written");
}

// ---------------------------------------------------------------------
// check_reply and settle (LLR-d55brn).
// ---------------------------------------------------------------------

// verifies: LLR-d55brn
#[test]
fn check_reply_refuses_another_organisation_and_an_id_not_outstanding() {
    let chain = ChainSlots::new();
    let (mut a, _, org) = founder("check", &chain);
    // A second Organisation A holds, through a second Persona, with its own
    // outstanding Invite.
    let carol = a.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Jones")).unwrap();
    let elsewhere = found(&mut a, &chain, &carol);
    let invite = build_invite(&mut a, org);
    let invite_elsewhere = build_invite(&mut a, elsewhere);
    let (mut b, pid) = joiner("check", &chain);
    let reply = build_reply(&mut b, &invite, &pid).unwrap();
    assert_eq!(a.check_reply(&reply, org), Ok(()), "outstanding for the Organisation it names");
    // Selected another Organisation: refused, whatever is outstanding.
    assert_eq!(a.check_reply(&reply, elsewhere), Err(OrgNodeError::ReplyForAnotherOrganisation));
    // An id never issued.
    let stranger = InviteReply { invite_id: InviteId::new([0x77; 32]), ..reply.clone() };
    assert_eq!(a.check_reply(&stranger, org), Err(OrgNodeError::InviteNotOutstanding));
    // An id outstanding, but for another Organisation than the reply names.
    let crossed = InviteReply { org_id: elsewhere, ..reply.clone() };
    assert_eq!(a.check_reply(&crossed, elsewhere), Err(OrgNodeError::InviteNotOutstanding));
    let crossed_back = InviteReply { invite_id: invite_elsewhere.invite_id, ..reply };
    assert_eq!(a.check_reply(&crossed_back, org), Err(OrgNodeError::InviteNotOutstanding));
}

// verifies: LLR-d55brn
#[test]
fn settle_removes_exactly_the_pair_and_refuses_one_not_held() {
    let chain = ChainSlots::new();
    let (mut a, pid, org) = founder("settle", &chain);
    let carol = a.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Jones")).unwrap();
    let elsewhere = found(&mut a, &chain, &carol);
    let first = build_invite(&mut a, org).invite_id;
    let second = build_invite(&mut a, org).invite_id;
    let third = build_invite(&mut a, elsewhere).invite_id;
    // Under another Organisation the pair is not held: refused, nothing written.
    let (before, on_disk) = (plaintexts(&a), a.disk.bytes());
    assert_eq!(a.svc.settle(&mut OsRng, elsewhere, first).unwrap_err(), OrgNodeError::InviteNotOutstanding);
    assert_eq!((plaintexts(&a), a.disk.bytes()), (before, on_disk), "nothing written");
    let write = a.svc.settle(&mut OsRng, org, first).unwrap();
    assert_eq!(write.device_key, a.persona_public_keys(&pid).unwrap().1, "the store that held the pair");
    a.disk.write(write);
    assert_eq!(a.store_data(&pid).unwrap().outstanding_invites, vec![(org, second)], "removes one, keeps the others");
    assert_eq!(a.store_data(&carol).unwrap().outstanding_invites, vec![(elsewhere, third)]);
    // The disk holds the stores in Device-key order.
    let mut reopened = reopen_store("settle", "a").outstanding_invites;
    reopened.sort();
    let mut expected = vec![(org, second), (elsewhere, third)];
    expected.sort();
    assert_eq!(reopened, expected, "as written");
    assert_eq!(a.svc.settle(&mut OsRng, org, first).unwrap_err(), OrgNodeError::InviteNotOutstanding, "once only");
}

// verifies: LLR-pba7yu
#[test]
fn forgetting_an_organisation_drops_its_outstanding_invites() {
    let chain = ChainSlots::new();
    let (mut a, pid, org) = founder("forget", &chain);
    let kept = build_invite(&mut a, org).invite_id;
    let mut data = a.store_data(&pid).unwrap().clone();
    let elsewhere = OrgId::new([0x42; 20]);
    data.outstanding_invites.push((elsewhere, InviteId::new([0x77; 32])));
    let rest = data.forget_organisation(elsewhere).expect("the Persona is not bound to it");
    assert_eq!(rest.outstanding_invites, vec![(org, kept)], "only that Organisation's pairs go");
    assert!(data.forget_organisation(org).is_none(), "the bound Organisation takes the store, pairs included");
}

// ---------------------------------------------------------------------
// One membership per Organisation per node (LLR-yexf5m, REQ-uy7kfq).
// ---------------------------------------------------------------------

// verifies: LLR-yexf5m
#[test]
fn build_reply_refuses_an_organisation_another_persona_holds() {
    let chain = ChainSlots::new();
    // A founds the Organisation; B, a second Persona of the same node, is
    // unbound.
    let (mut node, _a, org) = founder("held", &chain);
    let b = node.create_persona(&mut OsRng, h("bea"), nm("Bea"), sn("Second")).unwrap();
    let invite = build_invite(&mut node, org);
    let (before, on_disk) = (plaintexts(&node), node.disk.bytes());
    assert_eq!(build_reply(&mut node, &invite, &b).unwrap_err(), OrgNodeError::OrganisationHeldByAnotherPersona { org_id: org });
    assert_eq!(plaintexts(&node), before, "both stores unchanged");
    assert_eq!(node.disk.bytes(), on_disk, "nothing written");
    // The contrast: an Invite for an Organisation no Persona here holds.
    build_reply(&mut node, &invite_for(OrgId::new([0x42; 20])), &b).unwrap();
}

// verifies: LLR-yexf5m
#[test]
fn build_reply_refuses_an_organisation_another_persona_expects() {
    let chain = ChainSlots::new();
    let (mut node, a) = joiner("expects", &chain);
    let b = node.create_persona(&mut OsRng, h("bea"), nm("Bea"), sn("Second")).unwrap();
    let org = OrgId::new([0x42; 20]);
    let invite = invite_for(org);
    build_reply(&mut node, &invite, &a).unwrap();
    let (before, on_disk) = (plaintexts(&node), node.disk.bytes());
    assert_eq!(build_reply(&mut node, &invite, &b).unwrap_err(), OrgNodeError::OrganisationHeldByAnotherPersona { org_id: org });
    assert_eq!((plaintexts(&node), node.disk.bytes()), (before, on_disk), "nothing written");
    assert_eq!(node.expected_admissions(), vec![ExpectedAdmission { org_id: org }], "A's expectation alone");
}

// verifies: LLR-yexf5m
#[test]
fn the_same_persona_replying_twice_is_not_refused_by_the_one_membership_rule() {
    let chain = ChainSlots::new();
    let (mut node, a) = joiner("twice", &chain);
    node.create_persona(&mut OsRng, h("bea"), nm("Bea"), sn("Second")).unwrap();
    let invite = invite_for(OrgId::new([0x42; 20]));
    build_reply(&mut node, &invite, &a).unwrap();
    build_reply(&mut node, &invite, &a).expect("the same Persona again is not another Persona");
    assert_eq!(node.store_data(&a).unwrap().expected_admissions.len(), 1, "recorded once");
}

// verifies: LLR-yexf5m
#[test]
fn expect_admission_refuses_the_same_two_cases() {
    let chain = ChainSlots::new();
    // Held: A founded the Organisation.
    let (mut node, a, org) = founder("expect-held", &chain);
    let b = node.create_persona(&mut OsRng, h("bea"), nm("Bea"), sn("Second")).unwrap();
    let (before, on_disk) = (plaintexts(&node), node.disk.bytes());
    assert_eq!(
        node.expect_admission_for(&mut OsRng, &b, org).unwrap_err(),
        OrgNodeError::OrganisationHeldByAnotherPersona { org_id: org }
    );
    assert_eq!((plaintexts(&node), node.disk.bytes()), (before, on_disk), "nothing written");
    // Expected: A expects another Organisation; B may not expect it too.
    let elsewhere = OrgId::new([0x42; 20]);
    let c = node.create_persona(&mut OsRng, h("cal"), nm("Cal"), sn("Third")).unwrap();
    node.expect_admission_for(&mut OsRng, &b, elsewhere).unwrap();
    let (before, on_disk) = (plaintexts(&node), node.disk.bytes());
    assert_eq!(
        node.expect_admission_for(&mut OsRng, &c, elsewhere).unwrap_err(),
        OrgNodeError::OrganisationHeldByAnotherPersona { org_id: elsewhere }
    );
    assert_eq!((plaintexts(&node), node.disk.bytes()), (before, on_disk), "nothing written");
    // The same Persona again is not refused, and A is not bound by B's rule.
    node.expect_admission_for(&mut OsRng, &b, elsewhere).unwrap();
    assert_eq!(node.store_data(&b).unwrap().expected_admissions.len(), 1);
    assert!(node.store_data(&a).unwrap().expected_admissions.is_empty());
}

// verifies: LLR-yexf5m
#[test]
fn the_refusal_names_the_organisation_and_no_persona() {
    let chain = ChainSlots::new();
    let (mut node, a, org) = founder("display", &chain);
    let b = node.create_persona(&mut OsRng, h("bea"), nm("Bea"), sn("Second")).unwrap();
    let text = build_reply(&mut node, &invite_for(org), &b).unwrap_err().to_string();
    assert!(text.contains(&format!("{org:?}")), "names the Organisation: {text}");
    for persona in [&a, &b] {
        assert!(!text.contains(persona.as_str()), "names no Persona: {text}");
    }
}
