#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The invitation exchange runner (SDD-rr7dtw; S4, task T13 of
//! docs/plans/2026-10-06-org-io-transport.md): org-io issues an Invite,
//! produces a reply, imports a reply and admits it, over org-node's
//! `build_invite`, `build_reply`, `check_reply` and `settle`, writing each
//! store change before it returns (REQ-prjja8, REQ-tcutr6, REQ-65xqp8,
//! REQ-a2a6rm, REQ-pnr6kh).
//!
//! *Moved 2026-10-14 (S4, task T13, ruling A):* the fourteen tests of
//! LLR-9sraks, LLR-f35pda, LLR-w4mhd4, LLR-rt8gdz and LLR-gha5f6 came from
//! the app's `tests/invitation.rs` (deleted), `verifies:` lines unchanged,
//! rewritten onto `OrgIo`'s `issue_invite`, `produce_reply`, `import_reply`
//! and `admit_reply`. The outstanding pairs live in each Persona's sealed
//! store, so "reopen" reads the store files back, and the refusals are
//! org-node's typed errors in `ExchangeError::Node` where the app compared
//! its own strings. The test of the outstanding-invites file's format is
//! rewritten for LLR-f35pda as amended: no such file is read.
//!
//! The chain is a substitute (`test_support::FakeChain`); every Persona's
//! device secret is in the in-memory keychain (`test_support::MemoryKeychain`).

use std::path::Path;
use std::sync::atomic::Ordering;

use org_node::invitation::{Invite, InviteId, InviteReply};
use org_node::store::{ExpectedAdmission, PersonaStore, SealedStore, StoreData};
use org_node::{DevicePublicKey, Epoch, OrgId, OrgNodeError, PersonaId};
use org_io::test_support::{store_writes_to, FakeChain};
use org_io::{ExchangeError, OrgIo, Received, SendError};
use rand::rngs::OsRng;

mod common;
use common::handles::{at, handle, persona_named, receiving_addr, reopened_service, store_dir, NET};

/// A founded Organisation on a fresh handle for A in test `tag`, founded by
/// Alice.
async fn founded(tag: &str, chain: &FakeChain) -> (OrgIo, OrgId) {
    let a = handle(tag, "a", chain);
    let alice = persona_named(&a, "alice", "Alice", "Smith").await;
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    (a, org)
}

/// A fresh handle for B in test `tag`, over its own view of `chain`'s
/// slots, and its Persona Bob.
async fn joiner_side(tag: &str, chain: &FakeChain) -> (OrgIo, PersonaId) {
    let b = handle(tag, "b", &FakeChain::sharing_slots_with(chain));
    let bob = persona_named(&b, "bob", "Bob", "Builder").await;
    (b, bob)
}

/// The Persona bound to `org` on `io`.
fn bound_to(io: &OrgIo, org: OrgId) -> PersonaId {
    io.node_for_test().list_personas().into_iter().find(|persona| persona.org_id == Some(org)).unwrap().persona_id.clone()
}

/// Every (Organisation, invite identifier) pair `io`'s stores hold as
/// outstanding.
fn outstanding(io: &OrgIo) -> Vec<(OrgId, InviteId)> {
    let node = io.node_for_test();
    let personas: Vec<PersonaId> = node.list_personas().into_iter().map(|persona| persona.persona_id.clone()).collect();
    personas.iter().flat_map(|persona| node.store_data(persona).unwrap().outstanding_invites.clone()).collect()
}

/// The pairs held as outstanding by what reached `party`'s data directory.
fn outstanding_on_disk(tag: &str, party: &str, io: &OrgIo) -> Vec<(OrgId, InviteId)> {
    let service = reopened_service(tag, party, io);
    let personas: Vec<PersonaId> = service.list_personas().into_iter().map(|persona| persona.persona_id.clone()).collect();
    personas.iter().flat_map(|persona| service.store_data(persona).unwrap().outstanding_invites.clone()).collect()
}

/// The Members of `org` in `io`'s record.
async fn members(io: &OrgIo, org: OrgId) -> usize {
    io.organisation(org).await.unwrap().members.len()
}

/// The org-node refusal an exchange error carries.
fn refused(error: ExchangeError) -> OrgNodeError {
    match error {
        ExchangeError::Node(refusal) => refusal,
        other => panic!("not an org-node refusal: {other:?}"),
    }
}

/// org-node's refusal of `persona_id` as a Persona already bound (LLR-rt8gdz).
fn already_bound(persona_id: &PersonaId) -> String {
    OrgNodeError::PersonaAlreadyBound { persona_id: persona_id.clone() }.to_string()
}

/// The store data of every write to `device`'s file in `dir`, from the
/// `from`th on, unsealed with its keychain item.
fn written_since(io: &OrgIo, dir: &Path, device: DevicePublicKey, from: usize) -> Vec<StoreData> {
    store_writes_to(dir, &device)
        .into_iter()
        .skip(from)
        .map(|bytes| {
            let secret = io.device_secret_for_test(&device).unwrap();
            PersonaStore::unseal(&SealedStore::from_bytes(bytes), device, secret).unwrap().data().clone()
        })
        .collect()
}

// ---- Moved from the app (LLR-9sraks, LLR-f35pda, LLR-w4mhd4, LLR-rt8gdz, LLR-gha5f6) ----

// verifies: LLR-9sraks
#[tokio::test]
async fn an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id() {
    let chain = FakeChain::new();
    let (a, org) = founded("issue", &chain).await;
    let blob = a.issue_invite(&mut OsRng, org, "Acme Co-op", "Bob Builder").await.unwrap();
    let invite = Invite::parse(&blob).unwrap();
    assert_eq!((invite.org_name.as_str(), invite.org_id, invite.invitee_name.as_str()), ("Acme Co-op", org, "Bob Builder"));
    let (_, device) = a.persona_public_keys(&bound_to(&a, org)).await.unwrap();
    assert_eq!(invite.inviter_device_keys, vec![device]);
    assert_eq!(outstanding(&a), vec![(org, invite.invite_id)]);
    let other = Invite::parse(&a.issue_invite(&mut OsRng, org, "Acme Co-op", "Carol").await.unwrap()).unwrap();
    assert_ne!(other.invite_id, invite.invite_id, "drawn at random");
}

// verifies: LLR-9sraks
#[tokio::test]
async fn no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to() {
    let chain = FakeChain::new();
    let (a, _org) = founded("issue-none", &chain).await;
    assert!(a.issue_invite(&mut OsRng, OrgId::new([9; 20]), "X", "Y").await.is_err());
    assert!(outstanding(&a).is_empty());
}

// verifies: LLR-f35pda
#[tokio::test(flavor = "multi_thread")]
async fn outstanding_invite_ids_survive_a_reopen_and_settle_one_at_a_time() {
    let tag = "reopen";
    let chain = FakeChain::new();
    let (a, org) = founded(tag, &chain).await;
    let carol = persona_named(&a, "carol", "Carol", "Jones").await;
    let elsewhere = a.found_organisation(&mut OsRng, &carol).await.unwrap();
    let first = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let first_id = Invite::parse(&first).unwrap().invite_id;
    let second = Invite::parse(&a.issue_invite(&mut OsRng, org, "Acme", "Carol").await.unwrap()).unwrap().invite_id;
    let third = Invite::parse(&a.issue_invite(&mut OsRng, elsewhere, "Other", "Dave").await.unwrap()).unwrap().invite_id;
    let reopened = outstanding_on_disk(tag, "a", &a);
    assert!(
        reopened.contains(&(org, first_id)) && reopened.contains(&(org, second)) && reopened.contains(&(elsewhere, third)),
        "kept across a restart, each with the Organisation it was issued for"
    );
    assert!(
        !reopened.contains(&(elsewhere, first_id)) && !reopened.contains(&(org, third)),
        "an id is outstanding for its own Organisation only"
    );
    let (b, bob) = joiner_side(tag, &chain).await;
    // A reply naming the first id under another Organisation settles nothing.
    let forged = Invite { org_id: elsewhere, ..Invite::parse(&first).unwrap() }.encode().unwrap();
    let forged_reply = b.produce_reply(&mut OsRng, &forged, &bob, true).await.unwrap();
    assert!(a.admit_reply(&mut OsRng, elsewhere, &forged_reply, None).await.is_err());
    assert!(outstanding_on_disk(tag, "a", &a).contains(&(org, first_id)));
    // Admitting the true reply settles the first, and only it.
    let (c, dave) = joiner_side(&format!("{tag}-c"), &chain).await;
    let reply = c.produce_reply(&mut OsRng, &first, &dave, true).await.unwrap();
    a.admit_reply(&mut OsRng, org, &reply, None).await.unwrap();
    let again = outstanding_on_disk(tag, "a", &a);
    assert!(
        !again.contains(&(org, first_id)) && again.contains(&(org, second)) && again.contains(&(elsewhere, third)),
        "settle removes one and keeps the others"
    );
}

// *Rewritten 2026-10-14 (S4, task T13):* LLR-f35pda as amended keeps the
// pairs in the sealed store (REQ-pnr6kh), so the earlier test — a malformed
// outstanding-invites file is refused, naming it — has no file to refuse.
// What stands in its place: such a file, well formed or not, is never read.
// verifies: LLR-f35pda
#[tokio::test]
async fn an_outstanding_invites_file_in_the_data_dir_is_never_read() {
    let tag = "old-file";
    let chain = FakeChain::new();
    let (a, org) = founded(tag, &chain).await;
    let dir = store_dir(tag, "a");
    let pair = serde_json::json!([{ "org_id": hex::encode(org.as_bytes()), "invite_id": hex::encode([3u8; 32]) }]);
    std::fs::write(dir.join("outstanding_invites.json"), pair.to_string()).unwrap();
    let reopened = OrgIo::open(&dir, org_io::transport::TransportMode::Loopback).unwrap();
    assert!(outstanding(&reopened).is_empty(), "the file's pair is not outstanding");
    std::fs::write(dir.join("outstanding_invites.json"), "not a list").unwrap();
    let reopened = OrgIo::open(&dir, org_io::transport::TransportMode::Loopback).expect("a malformed old file refuses nothing");
    assert!(outstanding(&reopened).is_empty());
    let blob = reopened.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    assert_eq!(std::fs::read_to_string(dir.join("outstanding_invites.json")).unwrap(), "not a list", "nor written");
    assert_eq!(outstanding(&reopened), vec![(org, Invite::parse(&blob).unwrap().invite_id)]);
    drop(a);
}

// The three Blob-parse tests moved to `org-node/tests/invitation.rs` with
// their requirement (S4, task T6; ruling A).

// verifies: LLR-w4mhd4
#[tokio::test]
async fn a_confirmed_reply_carries_the_persona_and_declares_the_expected_admission() {
    let chain = FakeChain::new();
    let (a, org) = founded("reply", &chain).await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let invite_id = Invite::parse(&invite).unwrap().invite_id;
    let (b, bob) = joiner_side("reply", &chain).await;
    let reply = InviteReply::parse(&b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap()).unwrap();
    let (member_key, device_key) = b.persona_public_keys(&bob).await.unwrap();
    assert_eq!((reply.member_key, reply.device_key, reply.handle.as_str()), (member_key, device_key, "bob"));
    assert_eq!((reply.name.to_string(), reply.surname.to_string()), ("Bob".to_string(), "Builder".to_string()));
    assert_eq!((reply.org_id, reply.invite_id), (org, invite_id));
    assert_eq!(b.node_for_test().expected_admissions(), vec![ExpectedAdmission { org_id: org }]);
}

// verifies: LLR-w4mhd4
#[tokio::test]
async fn no_reply_is_produced_and_nothing_declared_without_confirmation() {
    let chain = FakeChain::new();
    let (a, org) = founded("unconfirmed", &chain).await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let (b, bob) = joiner_side("unconfirmed", &chain).await;
    let error = b.produce_reply(&mut OsRng, &invite, &bob, false).await.unwrap_err();
    assert!(matches!(error, ExchangeError::NotConfirmed), "{error:?}");
    assert!(error.to_string().starts_with("confirm first"), "{error}");
    assert!(b.node_for_test().expected_admissions().is_empty());
    assert!(b.produce_reply(&mut OsRng, &invite, &PersonaId::new("nobody".into()), true).await.is_err());
    assert!(b.node_for_test().expected_admissions().is_empty(), "an unknown Persona declares nothing");
}

// verifies: LLR-rt8gdz
#[tokio::test]
async fn no_reply_is_produced_and_nothing_declared_for_a_bound_persona() {
    let chain = FakeChain::new();
    let (a, org) = founded("bound-reply", &chain).await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    // B's Persona founded B's own Organisation, so it is bound.
    let (b, bob) = joiner_side("bound-reply", &chain).await;
    b.found_organisation(&mut OsRng, &bob).await.unwrap();
    assert_eq!(b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap_err().to_string(), already_bound(&bob));
    assert!(b.node_for_test().expected_admissions().is_empty(), "nothing declared");
    // The inviter's own Persona is bound to the very Organisation invited to.
    let own = bound_to(&a, org);
    assert_eq!(a.produce_reply(&mut OsRng, &invite, &own, true).await.unwrap_err().to_string(), already_bound(&own));
    assert!(a.node_for_test().expected_admissions().is_empty(), "nothing declared");
    // Unconfirmed is still refused as unconfirmed first.
    assert!(matches!(b.produce_reply(&mut OsRng, &invite, &bob, false).await.unwrap_err(), ExchangeError::NotConfirmed));
}

// verifies: LLR-rt8gdz
#[tokio::test]
async fn a_bound_persona_cannot_reply_and_an_unbound_one_on_the_same_device_can() {
    let chain = FakeChain::new();
    let (a, org) = founded("bound-contrast", &chain).await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let (b, bound) = joiner_side("bound-contrast", &chain).await;
    b.found_organisation(&mut OsRng, &bound).await.unwrap();
    let free = persona_named(&b, "dave", "Dave", "Lee").await;
    assert!(b.produce_reply(&mut OsRng, &invite, &bound, true).await.is_err());
    let reply = InviteReply::parse(&b.produce_reply(&mut OsRng, &invite, &free, true).await.unwrap()).unwrap();
    assert_eq!(reply.handle.as_str(), "dave");
    assert_eq!(b.node_for_test().expected_admissions().len(), 1, "declared for the unbound Persona only");
}

// verifies: LLR-rt8gdz
#[tokio::test]
async fn a_bound_persona_founds_nothing_and_the_chain_is_not_written() {
    let chain = FakeChain::new();
    let (a, org) = founded("bound-found", &chain).await;
    let alice = bound_to(&a, org);
    assert_eq!(chain.geneses(), 1);
    assert_eq!(a.found_organisation(&mut OsRng, &alice).await.unwrap_err(), already_bound(&alice));
    assert_eq!(chain.geneses(), 1, "refused before the chain writer is called");
    assert_eq!(a.organisations().await.len(), 1, "no second Organisation");
    assert!(a.node_for_test().genesis_provisional_updates(&alice).is_empty(), "nothing kept");
}

// *2026-10-13 (S4, task T12):* the one-send-per-command item this test also
// verified is retired (superseded by org-io's fan-out; see the S4
// verification record); its ID left this line, which keeps the other.
// verifies: LLR-gha5f6
#[tokio::test(flavor = "multi_thread")]
async fn a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding() {
    let tag = "acted-on";
    let chain = FakeChain::new();
    let (a, org) = founded(tag, &chain).await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let (b, bob) = joiner_side(tag, &chain).await;
    let reply_blob = b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    // A reply to an Invite this device never issued is refused, acting on nothing.
    let (stranger, _) = founded(&format!("{tag}-stranger"), &chain).await;
    assert!(matches!(refused(stranger.import_reply(&reply_blob).await.unwrap_err()), OrgNodeError::InviteNotOutstanding));
    // Acted on: written, committed, sent; the id is settled.
    let addr_b = receiving_addr(&b).await;
    let mut rng = OsRng;
    let (admitted, received) = tokio::join!(a.admit_reply(&mut rng, org, &reply_blob, at(addr_b)), async {
        tokio::time::timeout(NET, b.next_received(&mut OsRng)).await.unwrap()
    });
    admitted.unwrap();
    assert!(
        matches!(received, Ok(Received::Committed(ref outcome)) if outcome.org_id == org),
        "the reply's Device is listed: Organisation information, committed as its first admission: {received:?}"
    );
    let id = InviteReply::parse(&reply_blob).unwrap().invite_id;
    assert!(!outstanding(&a).contains(&(org, id)), "settled once acted on");
    assert_eq!(members(&a, org).await, 2);
    // The same reply again is refused: its invite is no longer outstanding.
    assert!(a.admit_reply(&mut OsRng, org, &reply_blob, None).await.is_err());
    assert_eq!(members(&a, org).await, 2, "acted on once");
}

// verifies: LLR-gha5f6
#[tokio::test]
async fn a_reply_whose_admission_fails_on_chain_stays_outstanding() {
    let chain = FakeChain::new();
    let (a, org) = founded("write-fails", &chain).await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let (b, bob) = joiner_side("write-fails", &chain).await;
    let reply_blob = b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    chain.write_failing.store(true, Ordering::SeqCst);
    assert!(a.admit_reply(&mut OsRng, org, &reply_blob, None).await.is_err());
    assert!(outstanding(&a).contains(&(org, InviteReply::parse(&reply_blob).unwrap().invite_id)));
    assert_eq!(members(&a, org).await, 1, "nothing committed");
}

// verifies: LLR-gha5f6
#[tokio::test]
async fn a_reply_admitted_under_another_selected_organisation_is_refused() {
    let chain = FakeChain::new();
    let (a, org) = founded("other-org", &chain).await;
    // A second Organisation A DOES hold, founded by a second Persona of A's
    // (one Persona, one Organisation). The reply is a true reply to the
    // Invite for `org`, and the caller selects `elsewhere`: the selection is
    // not trusted, so only the outstanding pair's Organisation can be the
    // target (review round 2, finding-1).
    let carol = persona_named(&a, "carol", "Carol", "Jones").await;
    let elsewhere = a.found_organisation(&mut OsRng, &carol).await.unwrap();
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let (b, bob) = joiner_side("other-org", &chain).await;
    let reply_blob = b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    let error = a.admit_reply(&mut OsRng, elsewhere, &reply_blob, None).await.unwrap_err();
    assert!(matches!(refused(error), OrgNodeError::ReplyForAnotherOrganisation));
    assert!(outstanding(&a).contains(&(org, InviteReply::parse(&reply_blob).unwrap().invite_id)), "nothing settled");
    assert!(
        a.node_for_test().provisional_updates(elsewhere).is_empty() && a.node_for_test().provisional_updates(org).is_empty(),
        "nothing built"
    );
    assert_eq!((members(&a, org).await, members(&a, elsewhere).await), (1, 1), "no one admitted to either");
}

// verifies: LLR-gha5f6, LLR-f35pda, REQ-65xqp8
#[tokio::test]
async fn a_reply_naming_another_held_organisation_with_an_outstanding_invite_id_is_refused() {
    // Review round 2, finding-1: an Invite issued for `org`; a reply that
    // echoes its invite id but names `elsewhere`, a second Organisation this
    // device also holds. The shipped flow takes the target Organisation from
    // the reply (Admit.svelte preselects it), so the caller passes the
    // reply's own org id.
    let chain = FakeChain::new();
    let (a, org) = founded("cross-org", &chain).await;
    let carol = persona_named(&a, "carol", "Carol", "Jones").await;
    let elsewhere = a.found_organisation(&mut OsRng, &carol).await.unwrap();
    let issued = Invite::parse(&a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap()).unwrap();
    let forged = Invite { org_id: elsewhere, ..issued.clone() }.encode().unwrap();
    let (b, bob) = joiner_side("cross-org", &chain).await;
    let reply_blob = b.produce_reply(&mut OsRng, &forged, &bob, true).await.unwrap();
    let reply = InviteReply::parse(&reply_blob).unwrap();
    assert_eq!((reply.org_id, reply.invite_id), (elsewhere, issued.invite_id));
    assert!(a.import_reply(&reply_blob).await.is_err(), "not outstanding for the Organisation it names");
    let error = a.admit_reply(&mut OsRng, reply.org_id, &reply_blob, None).await.unwrap_err();
    assert!(matches!(refused(error), OrgNodeError::InviteNotOutstanding));
    assert_eq!((members(&a, org).await, members(&a, elsewhere).await), (1, 1), "no one admitted to either");
    assert!(
        a.node_for_test().provisional_updates(elsewhere).is_empty() && a.node_for_test().provisional_updates(org).is_empty(),
        "nothing built"
    );
    assert!(outstanding(&a).contains(&(org, issued.invite_id)), "the Invite for its own Organisation stays outstanding");
    assert!(!outstanding(&a).contains(&(elsewhere, issued.invite_id)));
}

// *Rewritten 2026-10-14 (S4, task T13):* the send's outcome is reported per
// Device in the `AdmitReport` (org-io's fan-out), not as the call's error.
// verifies: LLR-gha5f6
#[tokio::test]
async fn a_reply_is_settled_once_committed_even_when_the_send_fails() {
    let chain = FakeChain::new();
    let (a, org) = founded("send-fails", &chain).await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let (b, bob) = joiner_side("send-fails", &chain).await;
    let reply_blob = b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    // Loopback transport with no peer address: the write and the commit
    // succeed, and the send that follows fails.
    let report = a.admit_reply(&mut OsRng, org, &reply_blob, None).await.unwrap();
    assert!(
        !report.sends.per_device.is_empty() && report.sends.per_device.iter().all(|(_, sent)| sent.is_err()),
        "the send failed: {:?}",
        report.sends
    );
    assert_eq!(members(&a, org).await, 2, "the admission committed");
    assert!(!outstanding(&a).contains(&(org, InviteReply::parse(&reply_blob).unwrap().invite_id)), "settled although the send failed");
}

// ---- New in S4 (LLR-h9qf3s, LLR-9cb443, LLR-y4dg74) ----

// A failed store write returns the failure and no Blob, and the pair is not
// held: the Persona is reloaded from its file, which never held it.
// verifies: LLR-h9qf3s
#[tokio::test]
async fn the_invite_is_returned_only_after_its_store_write() {
    let tag = "invite-write";
    let chain = FakeChain::new();
    let (a, org) = founded(tag, &chain).await;
    org_io::test_support::fail_next_write_after_temporary();
    let error = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap_err();
    assert!(matches!(error, ExchangeError::Store(_)), "{error:?}");
    assert!(outstanding(&a).is_empty(), "reloaded from the file: the pair is not held");
    assert!(outstanding_on_disk(tag, "a", &a).is_empty(), "nor on disk");
    // The contrast: a write that lands, and the Blob after it.
    let blob = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    assert_eq!(outstanding_on_disk(tag, "a", &a), vec![(org, Invite::parse(&blob).unwrap().invite_id)]);
}

// verifies: LLR-9cb443
#[tokio::test]
async fn an_unconfirmed_reply_parses_and_writes_nothing() {
    let tag = "unconfirmed-nothing";
    let chain = FakeChain::new();
    let (a, org) = founded(tag, &chain).await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let (b, bob) = joiner_side(tag, &chain).await;
    let (_, device) = b.persona_public_keys(&bob).await.unwrap();
    let dir = store_dir(tag, "b");
    let before = store_writes_to(&dir, &device).len();
    // Not a Blob at all: unconfirmed is refused before any parse.
    assert!(matches!(b.produce_reply(&mut OsRng, "not a blob", &bob, false).await.unwrap_err(), ExchangeError::NotConfirmed));
    assert!(matches!(b.produce_reply(&mut OsRng, &invite, &bob, false).await.unwrap_err(), ExchangeError::NotConfirmed));
    assert_eq!(store_writes_to(&dir, &device).len(), before, "nothing written");
    assert!(b.node_for_test().expected_admissions().is_empty(), "nothing declared");
    // The contrast: confirmed, an unparsable Blob is the parse's refusal, and
    // a parsable one is written before the reply returns.
    assert!(matches!(refused(b.produce_reply(&mut OsRng, "not a blob", &bob, true).await.unwrap_err()), OrgNodeError::InvalidField { .. }));
    b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    let written = written_since(&b, &dir, device, before);
    assert_eq!(written.len(), 1);
    assert_eq!(written[0].expected_admissions, vec![ExpectedAdmission { org_id: org }]);
}

// The order, read from what reached A's store file and the chain: the
// admission's write (built, not committed, the pair outstanding), one chain
// write and one read, the commit's write (committed, the pair outstanding),
// the send to the reply's Device, and the settle's write (the pair gone).
// verifies: LLR-y4dg74
#[tokio::test(flavor = "multi_thread")]
async fn admission_runs_check_admit_write_commit_send_settle_in_order() {
    let tag = "admit-order";
    let chain = FakeChain::new();
    let (a, org) = founded(tag, &chain).await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let invite_id = Invite::parse(&invite).unwrap().invite_id;
    let (b, bob) = joiner_side(tag, &chain).await;
    let reply_blob = b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    let (_, bob_device) = b.persona_public_keys(&bob).await.unwrap();
    let addr_b = receiving_addr(&b).await;
    let (_, alice_device) = a.persona_public_keys(&bound_to(&a, org)).await.unwrap();
    let dir = store_dir(tag, "a");
    let (writes_before, updates_before, reads_before) = (store_writes_to(&dir, &alice_device).len(), chain.update_writes(), chain.reads());

    let report = a.admit_reply(&mut OsRng, org, &reply_blob, at(addr_b)).await.unwrap();

    assert_eq!((chain.update_writes() - updates_before, chain.reads() - reads_before), (1, 1), "one chain write, one read back");
    let written = written_since(&a, &dir, alice_device, writes_before);
    assert_eq!(written.len(), 3, "the admission's, the commit's and the settle's writes");
    let record = |data: &StoreData| data.orgs.iter().find(|record| record.org_id == org).cloned().unwrap();
    assert_eq!(record(&written[0]).trie_members.len(), 1, "admission built, not committed");
    assert!(written[0].provisional_updates.iter().any(|update| update.org_id == Some(org)), "the admission is kept");
    assert!(written[0].outstanding_invites.contains(&(org, invite_id)));
    assert_eq!((record(&written[1]).trie_members.len(), record(&written[1]).epoch), (2, Epoch::new(2)), "committed");
    assert!(written[1].outstanding_invites.contains(&(org, invite_id)), "not settled before the commit's write");
    assert_eq!(record(&written[2]).trie_members.len(), 2);
    assert!(!written[2].outstanding_invites.contains(&(org, invite_id)), "settled last");
    let member = record(&written[2]).trie_members.iter().find(|member| member.device_keys.contains(&bob_device)).unwrap().id;
    assert_eq!(report.member_id, member);
    assert_eq!(report.sends.per_device.len(), 1);
    assert!(matches!(report.sends.per_device[0], (device, Ok(())) if device == bob_device), "{:?}", report.sends);
    let (sender, _) = tokio::time::timeout(NET, b.next_inbound_for_test()).await.unwrap().unwrap();
    assert_eq!(sender, alice_device, "the body reached B from A's Device");
}

// verifies: LLR-y4dg74
#[tokio::test]
async fn a_failed_send_still_settles() {
    let tag = "send-fails-settles";
    let chain = FakeChain::new();
    let (a, org) = founded(tag, &chain).await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let invite_id = Invite::parse(&invite).unwrap().invite_id;
    let (b, bob) = joiner_side(tag, &chain).await;
    let reply_blob = b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    let (_, bob_device) = b.persona_public_keys(&bob).await.unwrap();
    // Loopback with no address: every send refused, nothing sent.
    let report = a.admit_reply(&mut OsRng, org, &reply_blob, None).await.unwrap();
    assert!(matches!(report.sends.per_device[..], [(device, Err(SendError::NoLoopbackAddress))] if device == bob_device));
    assert_eq!(members(&a, org).await, 2, "committed");
    assert!(!outstanding(&a).contains(&(org, invite_id)), "settled in memory");
    assert!(!outstanding_on_disk(tag, "a", &a).contains(&(org, invite_id)), "and on disk");
}

// A reply that does not parse, one not outstanding and one for another
// selected Organisation: each refused before the admission is built, so
// nothing is written to the store or the chain.
// verifies: LLR-y4dg74
#[tokio::test]
async fn a_refused_check_submits_nothing() {
    let tag = "refused-check";
    let chain = FakeChain::new();
    let (a, org) = founded(tag, &chain).await;
    let carol = persona_named(&a, "carol", "Carol", "Jones").await;
    let elsewhere = a.found_organisation(&mut OsRng, &carol).await.unwrap();
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob").await.unwrap();
    let (b, bob) = joiner_side(tag, &chain).await;
    let reply_blob = b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    let unsolicited = Invite { invite_id: InviteId::new([0x5a; 32]), ..Invite::parse(&invite).unwrap() }.encode().unwrap();
    let (c, dave) = joiner_side(&format!("{tag}-c"), &chain).await;
    let unsolicited_reply = c.produce_reply(&mut OsRng, &unsolicited, &dave, true).await.unwrap();
    let dir = store_dir(tag, "a");
    let devices: Vec<DevicePublicKey> = {
        let personas = a.personas().await;
        let mut keys = vec![];
        for persona in personas {
            keys.push(a.persona_public_keys(&persona.persona_id).await.unwrap().1);
        }
        keys
    };
    let writes = || devices.iter().map(|device| store_writes_to(&dir, device).len()).sum::<usize>();
    let (writes_before, updates_before, reads_before) = (writes(), chain.update_writes(), chain.reads());

    let not_a_blob = a.admit_reply(&mut OsRng, org, "not a blob", None).await.unwrap_err();
    assert!(matches!(refused(not_a_blob), OrgNodeError::InvalidField { .. }));
    let not_outstanding = a.admit_reply(&mut OsRng, org, &unsolicited_reply, None).await.unwrap_err();
    assert!(matches!(refused(not_outstanding), OrgNodeError::InviteNotOutstanding));
    let another = a.admit_reply(&mut OsRng, elsewhere, &reply_blob, None).await.unwrap_err();
    assert!(matches!(refused(another), OrgNodeError::ReplyForAnotherOrganisation));

    assert_eq!((writes(), chain.update_writes(), chain.reads()), (writes_before, updates_before, reads_before), "nothing written or read");
    assert!(a.node_for_test().provisional_updates(org).is_empty(), "no admission built");
    assert_eq!(members(&a, org).await, 1);
    // The contrast: the true reply under its own Organisation is admitted.
    a.admit_reply(&mut OsRng, org, &reply_blob, None).await.unwrap();
    assert_eq!(chain.update_writes() - updates_before, 1);
}
