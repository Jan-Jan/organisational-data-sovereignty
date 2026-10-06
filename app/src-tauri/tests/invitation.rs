#![cfg(feature = "test-support")]
//! The invitation exchange (REQ-prjja8, REQ-tcutr6, REQ-65xqp8, REQ-ab2mfz,
//! REQ-yazum3): the Invite and the Invite reply as Blobs, parsed at this edge.
//! The chain write is a substitute (`support::FakeWriter`) over org-node's
//! `MockChainOps`, which the service reads.

use std::sync::atomic::Ordering;

use ods_poc_lib::invitation::{
    admit_reply, check_reply, issue_invite, produce_reply, Invite, InviteReply, OutstandingInvites,
};
use ods_poc_lib::submit::found_organisation;
use org_node::service::{MockChainOps, OrgService};
use org_node::store::PersonaStore;
use org_node::{DeviceSeed, Handle, Name, OrgId, Surname};
use rand::rngs::OsRng;

mod support;
use support::{reply_keys, FakeWriter};

/// A fresh, wiped directory named for `tag` and this process.
fn dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("ods-app-invitation-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A founded Organisation on A, and A's outstanding-invites file (path kept,
/// so a test can reopen what was written).
async fn founded(
    tag: &str,
    chain: &MockChainOps,
    writer: &FakeWriter,
) -> (OrgService, OrgId, OutstandingInvites, std::path::PathBuf) {
    let d = dir(&format!("{tag}-a"));
    let mut a = OrgService::new(PersonaStore::open(d.join("a.bin"), "pw").unwrap(), Box::new(chain.clone()));
    let pid = a
        .create_persona(&mut OsRng, Handle::parse("alice").unwrap(), Name::parse("Alice").unwrap(), Surname::parse("Smith").unwrap())
        .unwrap();
    let org = found_organisation(&mut a, writer, &mut OsRng, &pid).await.unwrap();
    let path = d.join("outstanding_invites.json");
    (a, org, OutstandingInvites::open(path.clone()).unwrap(), path)
}

fn joiner_service(tag: &str, chain: &MockChainOps) -> (OrgService, org_node::PersonaId) {
    let mut b = OrgService::new(
        PersonaStore::open(dir(&format!("{tag}-b")).join("b.bin"), "pw").unwrap(),
        Box::new(chain.clone()),
    );
    let pid = b
        .create_persona(&mut OsRng, Handle::parse("bob").unwrap(), Name::parse("Bob").unwrap(), Surname::parse("Builder").unwrap())
        .unwrap();
    (b, pid)
}

// verifies: LLR-9sraks
#[tokio::test]
async fn an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, org, mut outstanding, _) = founded("issue", &chain, &writer).await;
    let blob = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme Co-op", "Bob Builder").unwrap();
    let invite = Invite::parse(&blob).unwrap();
    assert_eq!((invite.org_name.as_str(), invite.org_id, invite.invitee_name.as_str()), ("Acme Co-op", org, "Bob Builder"));
    let (_, device) = a.persona_public_keys(&a.list_personas()[0].persona_id).unwrap();
    assert_eq!(invite.inviter_device_keys, vec![device]);
    assert!(outstanding.holds(org, &invite.invite_id));
    let other = Invite::parse(&issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme Co-op", "Carol").unwrap()).unwrap();
    assert_ne!(other.invite_id, invite.invite_id, "drawn at random");
}

// verifies: LLR-9sraks
#[tokio::test]
async fn no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, _org, mut outstanding, _) = founded("issue-none", &chain, &writer).await;
    assert!(issue_invite(&a, &mut outstanding, &mut OsRng, OrgId::new([9; 20]), "X", "Y").is_err());
    assert!(outstanding.is_empty());
}

// verifies: LLR-f35pda
#[tokio::test]
async fn outstanding_invite_ids_survive_a_reopen_and_settle_one_at_a_time() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, path) = founded("reopen", &chain, &writer).await;
    let carol = a
        .create_persona(&mut OsRng, Handle::parse("carol").unwrap(), Name::parse("Carol").unwrap(), Surname::parse("Jones").unwrap())
        .unwrap();
    let elsewhere = found_organisation(&mut a, &writer, &mut OsRng, &carol).await.unwrap();
    let first = Invite::parse(&issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap()).unwrap().invite_id;
    let second = Invite::parse(&issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Carol").unwrap()).unwrap().invite_id;
    let third =
        Invite::parse(&issue_invite(&a, &mut outstanding, &mut OsRng, elsewhere, "Other", "Dave").unwrap()).unwrap().invite_id;
    let mut reopened = OutstandingInvites::open(path.clone()).unwrap();
    assert!(
        reopened.holds(org, &first) && reopened.holds(org, &second) && reopened.holds(elsewhere, &third),
        "kept across a restart, each with the Organisation it was issued for"
    );
    assert!(!reopened.holds(elsewhere, &first) && !reopened.holds(org, &third), "an id is outstanding for its own Organisation only");
    // Settling an id under another Organisation settles nothing.
    reopened.settle(elsewhere, &first).unwrap();
    assert!(OutstandingInvites::open(path.clone()).unwrap().holds(org, &first));
    reopened.settle(org, &first).unwrap();
    let again = OutstandingInvites::open(path).unwrap();
    assert!(
        !again.holds(org, &first) && again.holds(org, &second) && again.holds(elsewhere, &third),
        "settle removes one and keeps the others"
    );
}

// verifies: LLR-f35pda
#[test]
fn an_outstanding_invites_file_that_is_not_a_list_of_pairs_is_refused() {
    let d = dir("bad-outstanding");
    let org = hex::encode([1u8; 20]);
    let id = hex::encode([3u8; 32]);
    let pair = |o: &str, i: &str| serde_json::json!([{ "org_id": o, "invite_id": i }]).to_string();
    for text in [
        "{}".to_string(),
        // The earlier format: bare invite ids, with no Organisation.
        serde_json::json!([id]).to_string(),
        pair(&org, "zz"),
        pair(&org, "abcd"),
        pair("zz", &id),
        pair(&hex::encode([1u8; 19]), &id),
        serde_json::json!([{ "invite_id": id }]).to_string(),
    ] {
        let path = d.join("outstanding_invites.json");
        std::fs::write(&path, &text).unwrap();
        let err = OutstandingInvites::open(path).err().expect("refused");
        assert!(err.starts_with("outstanding invite"), "{text}: {err}");
        assert!(err.contains("outstanding_invites.json"), "names the file: {err}");
    }
    // The contrast: the same pair, well formed, opens and is held.
    let path = d.join("outstanding_invites.json");
    std::fs::write(&path, pair(&org, &id)).unwrap();
    let held = OutstandingInvites::open(path).unwrap();
    assert!(held.holds(OrgId::new([1; 20]), &org_node::InviteId::new([3; 32])));
}

// verifies: LLR-b7wgpf
#[test]
fn an_invite_and_a_reply_that_parse_encode_back_to_the_same_blob() {
    let (mk, dk) = reply_keys();
    let invite = Invite::wire_for_test("Acme", &[1; 20], "Bob", &[dk], &[3; 32]);
    assert_eq!(Invite::parse(&invite).unwrap().encode().unwrap(), invite);
    let reply = InviteReply::wire_for_test(&[1; 20], &[3; 32], &mk, &dk, "bob", "Bob", "Builder");
    assert_eq!(InviteReply::parse(&reply).unwrap().encode().unwrap(), reply);
}

// verifies: LLR-b7wgpf
#[test]
fn an_invite_or_reply_that_does_not_parse_is_refused_naming_the_field() {
    let (mk, dk) = reply_keys();
    for (blob, field) in [
        (Invite::wire_for_test("O", &[1; 19], "N", &[dk], &[3; 32]), "invite.org_id"),
        (Invite::wire_for_test("O", &[1; 20], "N", &[[0xff; 32]], &[3; 32]), "invite.inviter_device_keys"),
        (Invite::wire_for_test("O", &[1; 20], "N", &[dk], &[3; 31]), "invite.invite_id"),
        ("not base64 !".to_string(), "invite"),
    ] {
        let err = Invite::parse(&blob).unwrap_err();
        assert!(err.starts_with(field), "{field}: {err}");
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
        let err = InviteReply::parse(&blob).unwrap_err();
        assert!(err.starts_with(field), "{field}: {err}");
    }
}

// verifies: LLR-b7wgpf
#[tokio::test]
async fn a_reply_that_does_not_parse_acts_on_nothing() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, _) = founded("bad-reply", &chain, &writer).await;
    issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let before = a.list_orgs()[0].clone();
    assert!(admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, "not base64 !", None, None).await.is_err());
    assert!(!outstanding.is_empty(), "nothing settled");
    assert_eq!(a.list_orgs()[0].trie_members.len(), before.trie_members.len());
    assert!(a.provisional_updates(org).is_empty(), "no service call");
}

// verifies: LLR-w4mhd4
#[tokio::test]
async fn a_confirmed_reply_carries_the_persona_and_declares_the_expected_admission() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, org, mut outstanding, _) = founded("reply", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let invite_id = Invite::parse(&invite).unwrap().invite_id;
    let (mut b, pid) = joiner_service("reply", &chain);
    let reply = InviteReply::parse(&produce_reply(&mut b, &mut OsRng, &invite, &pid, true).unwrap()).unwrap();
    let (mk, dk) = b.persona_public_keys(&pid).unwrap();
    assert_eq!((reply.member_key, reply.device_key, reply.handle.as_str()), (mk, dk, "bob"));
    assert_eq!((reply.name.to_string(), reply.surname.to_string()), ("Bob".to_string(), "Builder".to_string()));
    assert_eq!((reply.org_id, reply.invite_id), (org, invite_id));
    assert_eq!(b.expected_admissions(), &[org_node::store::ExpectedAdmission { org_id: org, invite_id }]);
}

// verifies: LLR-w4mhd4
#[tokio::test]
async fn no_reply_is_produced_and_nothing_declared_without_confirmation() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, org, mut outstanding, _) = founded("unconfirmed", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let (mut b, pid) = joiner_service("unconfirmed", &chain);
    let err = produce_reply(&mut b, &mut OsRng, &invite, &pid, false).unwrap_err();
    assert!(err.starts_with("confirm first"), "{err}");
    assert!(b.expected_admissions().is_empty());
    assert!(produce_reply(&mut b, &mut OsRng, &invite, &org_node::PersonaId::new("nobody".into()), true).is_err());
    assert!(b.expected_admissions().is_empty(), "an unknown Persona declares nothing");
}

/// org-node's refusal of `pid` as a Persona already bound (LLR-rt8gdz).
fn already_bound(pid: &org_node::PersonaId) -> String {
    org_node::OrgNodeError::PersonaAlreadyBound { persona_id: pid.clone() }.to_string()
}

// verifies: LLR-rt8gdz
#[tokio::test]
async fn no_reply_is_produced_and_nothing_declared_for_a_bound_persona() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, org, mut outstanding, _) = founded("bound-reply", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    // B's Persona founded B's own Organisation, so it is bound.
    let (mut b, pid) = joiner_service("bound-reply", &chain);
    found_organisation(&mut b, &writer, &mut OsRng, &pid).await.unwrap();
    assert_eq!(produce_reply(&mut b, &mut OsRng, &invite, &pid, true).unwrap_err(), already_bound(&pid));
    assert!(b.expected_admissions().is_empty(), "nothing declared");
    // The inviter's own Persona is bound to the very Organisation invited to.
    let mut a = a;
    let own = a.list_personas()[0].persona_id.clone();
    assert_eq!(produce_reply(&mut a, &mut OsRng, &invite, &own, true).unwrap_err(), already_bound(&own));
    assert!(a.expected_admissions().is_empty(), "nothing declared");
    // Unconfirmed is still refused as unconfirmed first.
    assert!(produce_reply(&mut b, &mut OsRng, &invite, &pid, false).unwrap_err().starts_with("confirm first"));
}

// verifies: LLR-rt8gdz
#[tokio::test]
async fn a_bound_persona_cannot_reply_and_an_unbound_one_on_the_same_device_can() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, org, mut outstanding, _) = founded("bound-contrast", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let (mut b, bound) = joiner_service("bound-contrast", &chain);
    found_organisation(&mut b, &writer, &mut OsRng, &bound).await.unwrap();
    let free = b
        .create_persona(&mut OsRng, Handle::parse("dave").unwrap(), Name::parse("Dave").unwrap(), Surname::parse("Lee").unwrap())
        .unwrap();
    assert!(produce_reply(&mut b, &mut OsRng, &invite, &bound, true).is_err());
    let reply = InviteReply::parse(&produce_reply(&mut b, &mut OsRng, &invite, &free, true).unwrap()).unwrap();
    assert_eq!(reply.handle.as_str(), "dave");
    assert_eq!(b.expected_admissions().len(), 1, "declared for the unbound Persona only");
}

// verifies: LLR-rt8gdz
#[tokio::test]
async fn a_bound_persona_founds_nothing_and_the_chain_is_not_written() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, _org, _outstanding, _) = founded("bound-found", &chain, &writer).await;
    let pid = a.list_personas()[0].persona_id.clone();
    assert_eq!(writer.geneses(), 1);
    assert_eq!(found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap_err(), already_bound(&pid));
    assert_eq!(writer.geneses(), 1, "refused before the chain writer is called");
    assert_eq!(a.list_orgs().len(), 1, "no second Organisation");
    assert!(a.genesis_provisional_updates(&pid).is_empty(), "nothing kept");
}

// verifies: LLR-gha5f6
#[tokio::test(flavor = "multi_thread")]
async fn a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, _) = founded("acted-on", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let (mut b, pid) = joiner_service("acted-on", &chain);
    let reply_blob = produce_reply(&mut b, &mut OsRng, &invite, &pid, true).unwrap();
    // A reply to an Invite this device never issued is refused, acting on nothing.
    let stranger = OutstandingInvites::open(dir("acted-on-stranger").join("o.json")).unwrap();
    assert!(check_reply(&stranger, &reply_blob).is_err());
    // Acted on: written, committed, sent; the id is settled.
    let sink = org_node::transport::endpoint::OrgEndpoint::bind(&DeviceSeed::from([0x44; 32]).signing_keypair())
        .await
        .unwrap();
    let addr = sink.inner().addr();
    // The task hands the endpoint back, so it outlives the sender's wait for
    // the receipt.
    let received = tokio::spawn(async move {
        let msg = sink.recv_one().await;
        (sink, msg)
    });
    admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, &reply_blob, Some(addr), None).await.unwrap();
    let id = InviteReply::parse(&reply_blob).unwrap().invite_id;
    let (_sink, msg) = received.await.unwrap();
    assert_eq!(msg.unwrap().1.invite_id, Some(id), "sent under the reply's invite id");
    assert!(!outstanding.holds(org, &id), "settled once acted on");
    assert_eq!(a.list_orgs()[0].trie_members.len(), 2);
    // The same reply again is refused: its invite is no longer outstanding.
    assert!(admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, &reply_blob, None, None).await.is_err());
    assert_eq!(a.list_orgs()[0].trie_members.len(), 2, "acted on once");
}

// verifies: LLR-gha5f6
#[tokio::test]
async fn a_reply_whose_admission_fails_on_chain_stays_outstanding() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, _) = founded("write-fails", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let (mut b, pid) = joiner_service("write-fails", &chain);
    let reply_blob = produce_reply(&mut b, &mut OsRng, &invite, &pid, true).unwrap();
    writer.failing.store(true, Ordering::SeqCst);
    assert!(admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, &reply_blob, None, None).await.is_err());
    assert!(outstanding.holds(org, &InviteReply::parse(&reply_blob).unwrap().invite_id));
    assert_eq!(a.list_orgs()[0].trie_members.len(), 1, "nothing committed");
}

// verifies: LLR-gha5f6
#[tokio::test]
async fn a_reply_admitted_under_another_selected_organisation_is_refused() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, _) = founded("other-org", &chain, &writer).await;
    // A second Organisation A DOES hold, founded by a second Persona of A's
    // (one Persona, one Organisation). The reply is a true reply to the
    // Invite for `org`, and the caller selects `elsewhere`: the selection is
    // not trusted, so only the outstanding pair's Organisation can be the
    // target (review round 2, finding-1).
    let carol = a
        .create_persona(&mut OsRng, Handle::parse("carol").unwrap(), Name::parse("Carol").unwrap(), Surname::parse("Jones").unwrap())
        .unwrap();
    let elsewhere = found_organisation(&mut a, &writer, &mut OsRng, &carol).await.unwrap();
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let (mut b, pid) = joiner_service("other-org", &chain);
    let reply_blob = produce_reply(&mut b, &mut OsRng, &invite, &pid, true).unwrap();
    let members = |a: &OrgService, o: OrgId| a.list_orgs().iter().find(|r| r.org_id == o).unwrap().trie_members.len();
    let err = admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, elsewhere, &reply_blob, None, None)
        .await
        .unwrap_err();
    assert_eq!(err, "this reply is for another Organisation");
    assert!(outstanding.holds(org, &InviteReply::parse(&reply_blob).unwrap().invite_id), "nothing settled");
    assert!(a.provisional_updates(elsewhere).is_empty() && a.provisional_updates(org).is_empty(), "nothing built");
    assert_eq!((members(&a, org), members(&a, elsewhere)), (1, 1), "no one admitted to either");
}

// verifies: LLR-gha5f6, LLR-f35pda, REQ-65xqp8
#[tokio::test]
async fn a_reply_naming_another_held_organisation_with_an_outstanding_invite_id_is_refused() {
    // Review round 2, finding-1: an Invite issued for `org`; a reply that
    // echoes its invite id but names `elsewhere`, a second Organisation this
    // device also holds. The shipped flow takes the target Organisation from
    // the reply (Admit.svelte preselects it), so the caller passes the
    // reply's own org id.
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, _) = founded("cross-org", &chain, &writer).await;
    let carol = a
        .create_persona(&mut OsRng, Handle::parse("carol").unwrap(), Name::parse("Carol").unwrap(), Surname::parse("Jones").unwrap())
        .unwrap();
    let elsewhere = found_organisation(&mut a, &writer, &mut OsRng, &carol).await.unwrap();
    let issued = Invite::parse(&issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap()).unwrap();
    let forged = Invite { org_id: elsewhere, ..issued.clone() }.encode().unwrap();
    let (mut b, pid) = joiner_service("cross-org", &chain);
    let reply_blob = produce_reply(&mut b, &mut OsRng, &forged, &pid, true).unwrap();
    let reply = InviteReply::parse(&reply_blob).unwrap();
    assert_eq!((reply.org_id, reply.invite_id), (elsewhere, issued.invite_id));
    let members = |a: &OrgService, o: OrgId| a.list_orgs().iter().find(|r| r.org_id == o).unwrap().trie_members.len();
    assert!(check_reply(&outstanding, &reply_blob).is_err(), "not outstanding for the Organisation it names");
    let err = admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, reply.org_id, &reply_blob, None, None)
        .await
        .unwrap_err();
    assert_eq!(err, "this reply names no Invite this device has outstanding");
    assert_eq!((members(&a, org), members(&a, elsewhere)), (1, 1), "no one admitted to either");
    assert!(a.provisional_updates(elsewhere).is_empty() && a.provisional_updates(org).is_empty(), "nothing built");
    assert!(outstanding.holds(org, &issued.invite_id), "the Invite for its own Organisation stays outstanding");
    assert!(!outstanding.holds(elsewhere, &issued.invite_id));
}

// verifies: LLR-gha5f6
#[tokio::test]
async fn a_reply_is_settled_once_committed_even_when_the_send_fails() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, _) = founded("send-fails", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let (mut b, pid) = joiner_service("send-fails", &chain);
    let reply_blob = produce_reply(&mut b, &mut OsRng, &invite, &pid, true).unwrap();
    // Loopback transport with no peer address: the write and the commit
    // succeed, and the send that follows fails.
    let err = admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, &reply_blob, None, None).await.unwrap_err();
    assert!(err.contains("the send failed"), "{err}");
    assert_eq!(a.list_orgs()[0].trie_members.len(), 2, "the admission committed");
    assert!(!outstanding.holds(org, &InviteReply::parse(&reply_blob).unwrap().invite_id), "settled although the send failed");
}
