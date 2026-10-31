#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The invitation operations (SDD-zqv3v6, SDD-9rbm8t, SDD-cxh9d8): `issue_invite` builds the Invite in the
//! holding Persona's store with that endpoint's addresses and writes before
//! it returns (LLR-95e3jx, LLR-v9dkrv); `read_invite` reads its public
//! fields (LLR-npw5zg); `cancel_invite` removes a kept admission request and
//! its invite (LLR-8ze6cc); `admission_requests` lists the kept requests
//! (LLR-dm2h8u).
//!
//! Each test starts from `holding_acme`: a founding proposed, written and
//! adopted through the handle. The outstanding pairs live in each Persona's
//! sealed store, so "reopen" reads the store files back, and the refusals
//! are org-node's typed errors in `ExchangeError::Node`; no
//! outstanding-invites file is read (LLR-f35pda).
//!
//! The chain is a substitute (`test_support::FakeChain`); every Persona's
//! device secret is in the in-memory keychain (`test_support::MemoryKeychain`).

use std::path::Path;

use org_node::invitation::{Invite, InviteId};
use org_node::{DevicePublicKey, OrgId, OrgNodeError};
use org_io::test_support::{FakeChain, Refusal};
use org_io::transport::TransportMode;
use org_io::{ExchangeError, OrgIo};

mod common;
use common::handles::{fresh_keychain, handle, persona_of, reopened_service, store_bytes, store_dir};

/// A's handle over `chain`, holding "Acme" as a current record: a founding proposed, written and
/// adopted through the handle (`common::handles::founded`).
async fn holding_acme(tag: &str, chain: &FakeChain) -> (OrgIo, OrgId, DevicePublicKey) {
    let alice = handle(tag, "a", chain);
    let (org_id, founder) = common::handles::founded(&alice, "alice").await;
    (alice, org_id, founder)
}

/// The (Organisation, invite id) pairs `device`'s store keeps, as A's handle wrote them in test
/// `tag` (read from its store file, apart from the handle).
fn outstanding(tag: &str, io: &OrgIo, device_key: &DevicePublicKey) -> Vec<(OrgId, InviteId)> {
    reopened_service(tag, "a", io)
        .store_data(device_key)
        .unwrap()
        .outstanding_invites
        .iter()
        .map(|invite| (invite.org_id, invite.invite_id))
        .collect()
}

/// B proposes itself as a Member from `blob` and A's receive step keeps its admission request,
/// over the transport (`common::handles::requested`).
async fn requested(alice: &OrgIo, tag: &str, chain: &FakeChain, blob: &str) -> DevicePublicKey {
    let bob = handle(tag, "b", chain);
    common::handles::requested(alice, &bob, blob, &["bob", "bobby"]).await
}

// verifies: LLR-95e3jx
#[tokio::test]
async fn issue_invite_builds_in_the_holding_persona_and_writes_before_it_returns() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, founder) = holding_acme("issue-holding", &chain).await;
    let blob = alice.issue_invite(org, "Bob Builder").await.unwrap();
    let invite = Invite::parse(&blob).unwrap();
    assert_eq!(
        (invite.org_name.as_str(), invite.org_id, invite.invitee_name.as_str(), invite.issuer),
        ("Acme", org, "Bob Builder", founder)
    );
    assert_eq!(outstanding("issue-holding", &alice, &founder), vec![(org, invite.invite_id)]);
    let on_disk = reopened_service("issue-holding", "a", &alice);
    assert_eq!(on_disk.store_data(&founder).unwrap().outstanding_invites.len(), 1, "written before the Blob was returned");
    // Abnormal: an Organisation no Persona here holds is org-io's own
    // refusal, naming it.
    let stranger = OrgId::new([9; 20]);
    assert!(matches!(alice.issue_invite(stranger, "Y").await, Err(ExchangeError::NotHeld { org_id }) if org_id == stranger));
}

// verifies: LLR-v9dkrv
#[tokio::test(flavor = "multi_thread")]
async fn the_invite_carries_the_issuing_endpoints_addresses_in_the_current_mode() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, founder) = holding_acme("issue-addresses", &chain).await;
    assert_eq!(alice.transport_mode_for_test(), TransportMode::Loopback);
    let invite = Invite::parse(&alice.issue_invite(org, "Bob").await.unwrap()).unwrap();
    let endpoint = alice.bound_endpoint_for_test(&persona_of(&alice, &founder).await).await.expect("bound by the issue");
    let id = hex::encode(founder.as_bytes());
    let bound: Vec<String> = endpoint.bound_sockets().iter().map(|socket| format!("{id}@{socket}")).collect();
    assert_eq!(invite.issuer_addresses.as_slice(), bound.as_slice(), "the bound loopback sockets, as kept address text");
    assert!(!bound.is_empty(), "at least one bound socket");
    for address in &bound {
        let (endpoint_id, socket) = address.split_once('@').expect("<endpoint id>@<socket>");
        assert_eq!(endpoint_id, id, "the endpoint id is the issuing Device key, 64 lower-case hex");
        assert!(socket.starts_with("127.0.0.1:") || socket.starts_with("[::1]:"), "{address}");
    }
}

// Abnormal: the issuing Persona's endpoint cannot be bound (the keychain refuses its item).
// verifies: LLR-v9dkrv
#[tokio::test]
async fn a_failed_bind_refuses_the_invite_and_writes_nothing() {
    let (_guard, keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, founder) = holding_acme("issue-bind-fails", &chain).await;
    let before = store_bytes("issue-bind-fails", "a");
    keychain.refuse_as(Some(Refusal::Locked));
    let refused = alice.issue_invite(org, "Bob").await;
    keychain.refuse_as(None);
    assert!(matches!(refused, Err(ExchangeError::Bind(_))), "{refused:?}");
    assert!(outstanding("issue-bind-fails", &alice, &founder).is_empty(), "nothing built");
    assert_eq!(store_bytes("issue-bind-fails", "a"), before, "nothing written");
}

// verifies: LLR-npw5zg
#[tokio::test]
async fn read_invite_returns_the_public_fields_and_writes_nothing() {
    let (_guard, keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, founder) = holding_acme("read-invite", &chain).await;
    let blob = alice.issue_invite(org, "Bob").await.unwrap();
    let bob = handle("read-invite", "b", &chain);
    let (bytes, items) = (store_bytes("read-invite", "b"), keychain.items());
    let summary = bob.read_invite(&blob).await.unwrap();
    assert_eq!(
        (summary.org_name.as_str(), summary.org_id, summary.invitee_name.as_str(), summary.issuer),
        ("Acme", org, "Bob", founder)
    );
    assert_eq!(summary.issuer_addresses, Invite::parse(&blob).unwrap().issuer_addresses.as_slice().to_vec());
    assert_eq!((store_bytes("read-invite", "b"), keychain.items()), (bytes, items), "nothing written, nothing created");
    assert!(bob.personas().await.is_empty());
    match bob.read_invite("not an invite").await {
        Err(ExchangeError::Node(OrgNodeError::InvalidField { field, .. })) => assert!(field.starts_with("invite"), "{field}"),
        other => panic!("not refused naming the field: {other:?}"),
    }
}

// verifies: LLR-8ze6cc
#[tokio::test]
async fn cancel_invite_writes_the_store_and_refuses_an_invite_with_no_kept_request() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, founder) = holding_acme("cancel", &chain).await;
    let kept = Invite::parse(&alice.issue_invite(org, "Bob").await.unwrap()).unwrap();
    let bare = Invite::parse(&alice.issue_invite(org, "Carol").await.unwrap()).unwrap();
    requested(&alice, "cancel", &chain, &kept.encode().unwrap()).await;
    let before = store_bytes("cancel", "a");
    assert!(matches!(alice.cancel_invite(org, bare.invite_id).await, Err(ExchangeError::Node(OrgNodeError::NoAdmissionRequestKept { invite_id })) if invite_id == bare.invite_id
    ));
    assert_eq!(store_bytes("cancel", "a"), before, "nothing written for a refusal");
    // Abnormal: an Organisation no Persona here holds is org-io's own
    // refusal, naming it, with nothing written.
    let stranger = OrgId::new([9; 20]);
    assert!(matches!(alice.cancel_invite(stranger, kept.invite_id).await, Err(ExchangeError::NotHeld { org_id }) if org_id == stranger));
    assert_eq!(store_bytes("cancel", "a"), before, "nothing written for org-io's refusal");
    alice.cancel_invite(org, kept.invite_id).await.unwrap();
    assert_eq!(outstanding("cancel", &alice, &founder), vec![(org, bare.invite_id)], "the kept request's invite is gone");
    assert!(alice.admission_requests().await.is_empty(), "and its kept request");
    let on_disk = reopened_service("cancel", "a", &alice);
    assert!(on_disk.store_data(&founder).unwrap().kept_admission_requests.is_empty(), "written before it returned");
}

// Abnormal: the cancel's store write fails after its
// temporary file. The cancel is `ExchangeError::Store`, the Persona reloaded
// from its file: the kept request and its outstanding invite are both still
// there, in the handle and in the file.
// verifies: LLR-8ze6cc
#[tokio::test]
async fn a_failed_write_of_a_cancel_is_exchange_error_store_and_reloads_the_persona() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "cancel-write-fails";
    let (alice, org, founder) = holding_acme(tag, &chain).await;
    let kept = Invite::parse(&alice.issue_invite(org, "Bob").await.unwrap()).unwrap();
    requested(&alice, tag, &chain, &kept.encode().unwrap()).await;
    org_io::test_support::fail_next_write_after_temporary();
    let cancelled = alice.cancel_invite(org, kept.invite_id).await;
    assert!(matches!(cancelled, Err(ExchangeError::Store(org_io::StoreFailure::Reloaded { .. }))), "{cancelled:?}");
    let requests = alice.admission_requests().await;
    assert_eq!(requests.len(), 1, "reloaded from its file: the request is still kept");
    assert_eq!(requests[0].invite_id, kept.invite_id);
    assert_eq!(outstanding(tag, &alice, &founder), vec![(org, kept.invite_id)], "the file keeps the invite");
}

// verifies: LLR-dm2h8u
#[tokio::test]
async fn admission_requests_lists_every_kept_request_with_its_invitee_name_and_changes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, _) = holding_acme("requests-listed", &chain).await;
    assert!(alice.admission_requests().await.is_empty(), "none kept yet");
    let invite = Invite::parse(&alice.issue_invite(org, "Bob the guess").await.unwrap()).unwrap();
    let bob = requested(&alice, "requests-listed", &chain, &invite.encode().unwrap()).await;
    let (before, reads) = (store_bytes("requests-listed", "a"), chain.reads());
    let listed = alice.admission_requests().await;
    assert_eq!(listed, alice.admission_requests().await, "two calls, equal lists");
    assert_eq!(listed.len(), 1);
    let request = &listed[0];
    assert_eq!((request.org_id, request.invite_id, request.device_key), (org, invite.invite_id, bob));
    assert_eq!(
        (request.name.as_str(), request.surname.as_str(), request.invitee_name.as_str()),
        ("Bob", "Builder", "Bob the guess")
    );
    assert_eq!(request.preferred_handles.iter().map(|handle| handle.as_str()).collect::<Vec<_>>(), ["bob", "bobby"]);
    assert_eq!(store_bytes("requests-listed", "a"), before, "nothing written");
    assert_eq!(chain.reads(), reads, "no chain read");
}

// verifies: LLR-95e3jx
#[tokio::test]
async fn an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, founder) = holding_acme("issue", &chain).await;
    let invite = Invite::parse(&alice.issue_invite(org, "Bob Builder").await.unwrap()).unwrap();
    assert_eq!((invite.org_name.as_str(), invite.invitee_name.as_str()), ("Acme", "Bob Builder"));
    let other = Invite::parse(&alice.issue_invite(org, "Carol").await.unwrap()).unwrap();
    assert_ne!(invite.invite_id, other.invite_id, "a fresh identifier each time");
    assert_eq!(outstanding("issue", &alice, &founder), vec![(org, invite.invite_id), (org, other.invite_id)]);
}

// verifies: LLR-95e3jx
#[tokio::test]
async fn no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, _org, founder) = holding_acme("issue-none", &chain).await;
    let before = store_bytes("issue-none", "a");
    assert!(alice.issue_invite(OrgId::new([9; 20]), "Y").await.is_err());
    assert!(outstanding("issue-none", &alice, &founder).is_empty());
    assert_eq!(store_bytes("issue-none", "a"), before);
}

// verifies: LLR-95e3jx
#[tokio::test]
async fn outstanding_invite_ids_survive_a_reopen_and_leave_only_by_adoption_or_cancel() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "outstanding-reopen";
    let (alice, org, founder) = holding_acme(tag, &chain).await;
    let first = Invite::parse(&alice.issue_invite(org, "Bob").await.unwrap()).unwrap();
    let second = Invite::parse(&alice.issue_invite(org, "Carol").await.unwrap()).unwrap();
    let reopened = || -> Vec<InviteId> {
        reopened_service(tag, "a", &alice)
            .store_data(&founder)
            .unwrap()
            .outstanding_invites
            .iter()
            .map(|invite| invite.invite_id)
            .collect()
    };
    assert_eq!(reopened(), vec![first.invite_id, second.invite_id], "both survive a reopen");
    requested(&alice, tag, &chain, &first.encode().unwrap()).await;
    alice.cancel_invite(org, first.invite_id).await.unwrap();
    assert_eq!(reopened(), vec![second.invite_id], "a cancel removes one");
    assert!(alice.cancel_invite(org, second.invite_id).await.is_err(), "no other operation removes the other");
    assert_eq!(reopened(), vec![second.invite_id]);
}

// verifies: LLR-95e3jx
#[tokio::test]
async fn an_outstanding_invites_file_in_the_data_dir_is_never_read() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let (alice, org, founder) = holding_acme("legacy-file", &chain).await;
    let dir = store_dir("legacy-file", "a");
    let legacy = format!("{{\"{}\":[\"{}\"]}}", hex::encode(org.as_bytes()), hex::encode([7u8; 32]));
    std::fs::write(Path::new(&dir).join("outstanding_invites.json"), legacy).unwrap();
    let invite = Invite::parse(&alice.issue_invite(org, "Bob").await.unwrap()).unwrap();
    assert_eq!(outstanding("legacy-file", &alice, &founder), vec![(org, invite.invite_id)], "only the store's own invite");
}

// verifies: LLR-95e3jx
#[tokio::test]
async fn the_invite_is_returned_only_after_its_store_write() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "returned-after-write";
    let (alice, org, founder) = holding_acme(tag, &chain).await;
    org_io::test_support::fail_next_write_after_temporary();
    let refused = alice.issue_invite(org, "Bob").await;
    assert!(matches!(refused, Err(ExchangeError::Store(_))), "no Blob after a failed write: {refused:?}");
    assert!(outstanding(tag, &alice, &founder).is_empty(), "nothing written");
    // The next issue writes the whole store the handle holds: only its own invite is in it, so
    // the refused one was not kept in memory — the Persona was reloaded from its file.
    let blob = alice.issue_invite(org, "Bob").await.unwrap();
    assert_eq!(outstanding(tag, &alice, &founder), vec![(org, Invite::parse(&blob).unwrap().invite_id)], "the Persona reloaded from its file");
}
