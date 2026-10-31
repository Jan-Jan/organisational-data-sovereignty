#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The Admin's provisional changes (SDD-7mhqaz) and the proposed Organisation (SDD-4drrpr): each
//! operation has org-node record or calculate with the handle's source, writes the store it
//! returns before reporting, and touches no chain or transport (LLR-un47j8, LLR-6kb33y, LLR-5fchwt).

use org_node::invitation::{Invite, InviteId};
use org_node::store::ProvisionalChange;
use org_node::{MemberId, OrgNodeError};
use org_io::test_support::FakeChain;
use org_io::{OperationError, OrgIo, ProvisionalChangeSummary};

mod common;
use common::handles::{founded, fresh_keychain, handle, reopened_service, requested, store_bytes};

fn changes_on_disk(tag: &str, io: &OrgIo, founder: &org_node::DevicePublicKey) -> Vec<ProvisionalChange> {
    reopened_service(tag, "a", io).store_data(founder).unwrap().provisional_changes.clone()
}

/// The chain calls `chain` has counted: reads, finalised-block reads, update writes.
fn chain_calls(chain: &FakeChain) -> (usize, usize, usize) {
    (chain.reads(), chain.finalised_reads(), chain.update_writes())
}

fn refused(proposed: Result<ProvisionalChangeSummary, OperationError>) -> OrgNodeError {
    match proposed {
        Err(OperationError::Node(error)) => error,
        other => panic!("not org-node's refusal: {other:?}"),
    }
}

// verifies: LLR-un47j8
#[tokio::test]
async fn propose_admission_records_the_change_with_the_handle_given_and_writes_first() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "propose-admission";
    let alice = handle(tag, "a", &chain);
    let (org, founder) = founded(&alice, "alice").await;
    let first = Invite::parse(&alice.issue_invite(org, "Bob").await.unwrap()).unwrap();
    requested(&alice, &handle(tag, "b", &chain), &first.encode().unwrap(), &["bob", "bobby"]).await;
    let recorded = alice.propose_admission(org, first.invite_id, None).await.unwrap();
    assert_eq!(
        recorded,
        ProvisionalChangeSummary::Admission { org_id: org, invite_id: first.invite_id, handle: org_node::Handle::parse("bob").unwrap() }
    );
    assert_eq!(changes_on_disk(tag, &alice, &founder).len(), 1, "written before it returned");

    let second = Invite::parse(&alice.issue_invite(org, "Carol").await.unwrap()).unwrap();
    requested(&alice, &handle(tag, "c", &chain), &second.encode().unwrap(), &["bob"]).await;
    assert_eq!(refused(alice.propose_admission(org, second.invite_id, None).await), OrgNodeError::NoFreeHandle, "bob is taken by the first change");
    let chosen = alice.propose_admission(org, second.invite_id, Some("carla")).await.unwrap();
    assert_eq!(
        chosen,
        ProvisionalChangeSummary::Admission { org_id: org, invite_id: second.invite_id, handle: org_node::Handle::parse("carla").unwrap() }
    );
    assert_eq!(chain.update_writes(), 0, "no chain write");
}

// verifies: LLR-un47j8
#[tokio::test]
async fn an_org_node_refusal_of_an_admission_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "propose-admission-refused";
    let alice = handle(tag, "a", &chain);
    let (org, _) = founded(&alice, "alice").await;
    let before = store_bytes(tag, "a");
    assert_eq!(refused(alice.propose_admission(org, InviteId::new([0x44; 32]), None).await),
        OrgNodeError::NoAdmissionRequestKept { invite_id: InviteId::new([0x44; 32]) }
    );
    assert_eq!(store_bytes(tag, "a"), before, "nothing written");
    let invite = Invite::parse(&alice.issue_invite(org, "Bob").await.unwrap()).unwrap();
    requested(&alice, &handle(tag, "b", &chain), &invite.encode().unwrap(), &["bob"]).await;
    alice.propose_admission(org, invite.invite_id, None).await.unwrap();
    let before = store_bytes(tag, "a");
    assert_eq!(
        refused(alice.propose_admission(org, invite.invite_id, None).await),
        OrgNodeError::AdmissionAlreadyRecorded { invite_id: invite.invite_id }
    );
    assert_eq!(store_bytes(tag, "a"), before, "nothing written");
}

// verifies: LLR-6kb33y
#[tokio::test]
async fn propose_revocation_records_the_change_and_sends_and_writes_nothing_to_the_chain() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "propose-revocation";
    let alice = handle(tag, "a", &chain);
    let (org, founder) = founded(&alice, "alice").await;
    let calls = chain_calls(&chain);
    let member = alice.organisation(org).await.unwrap().members[0].id;
    let recorded = alice.propose_revocation(org, member).await.unwrap();
    assert_eq!(recorded, ProvisionalChangeSummary::Revocation { org_id: org, member_id: member });
    assert_eq!(changes_on_disk(tag, &alice, &founder), vec![ProvisionalChange::Revoke { member_id: member }], "written before it returned");
    assert_eq!(chain_calls(&chain), calls, "no chain call");
    assert_eq!(alice.bound_endpoints_for_test().await, 0, "nothing sent: no endpoint bound");
    assert_eq!(alice.organisation(org).await.unwrap().members.len(), 1, "the current record is unchanged");
}

// verifies: LLR-5fchwt
#[tokio::test]
async fn propose_updated_organisation_returns_the_epoch_root_and_key_and_makes_no_chain_call() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "propose-updated";
    let alice = handle(tag, "a", &chain);
    let (org, founder) = founded(&alice, "alice").await;
    // Abnormal: org-node's refusal is returned with
    // nothing written.
    let before = store_bytes(tag, "a");
    assert!(matches!(alice.propose_updated_organisation(org).await, Err(OperationError::Node(OrgNodeError::NoProvisionalChange))));
    assert_eq!(store_bytes(tag, "a"), before, "nothing written for org-node's refusal");
    let blob = alice.issue_invite(org, "Bob").await.unwrap();
    let invite_id = Invite::parse(&blob).unwrap().invite_id;
    requested(&alice, &handle(tag, "b", &chain), &blob, &["bob"]).await;
    alice.propose_admission(org, invite_id, None).await.unwrap();
    let calls = chain_calls(&chain);
    let next = alice.propose_updated_organisation(org).await.unwrap();
    assert_eq!((next.org_id, next.epoch.get()), (org, 2));
    let current = alice.organisation(org).await.unwrap();
    assert_ne!(next.root, current.root_hash, "the next record's root");
    assert_ne!(next.org_public_key, current.org_pub_key, "a fresh Organisation key pair");
    assert_eq!(current.epoch.get(), 1, "the current record is unchanged");
    let on_disk = reopened_service(tag, "a", &alice);
    let views = on_disk.provisional_organisations(&founder).unwrap();
    let last = views.last().unwrap();
    assert_eq!(
        (last.epoch, last.root, last.org_pub_key, last.current_proposal),
        (next.epoch, next.root, next.org_public_key, true),
        "written before it returned"
    );
    assert!(on_disk.store_data(&founder).is_ok());
    assert_eq!(chain_calls(&chain), calls, "no chain read or write");
}

// Abnormal: an Organisation no Persona here holds is
// org-io's own refusal, `OperationError::NotHeld` naming it, for each of the
// three operations, never an org-node error org-io made up; nothing is
// written, read from or written to the chain.
// verifies: LLR-un47j8, LLR-6kb33y, LLR-5fchwt
#[tokio::test]
async fn an_organisation_no_persona_holds_is_org_ios_own_refusal_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "propose-not-held";
    let alice = handle(tag, "a", &chain);
    founded(&alice, "alice").await;
    let (before, calls) = (store_bytes(tag, "a"), chain_calls(&chain));
    let stranger = org_node::OrgId::new([0x9a; 20]);
    let not_held = |proposed: Result<(), OperationError>| matches!(proposed, Err(OperationError::NotHeld { org_id }) if org_id == stranger);
    assert!(not_held(alice.propose_admission(stranger, InviteId::new([0x44; 32]), None).await.map(drop)), "an admission");
    assert!(not_held(alice.propose_revocation(stranger, MemberId::new([0x99; 32])).await.map(drop)), "a revocation");
    assert!(not_held(alice.propose_updated_organisation(stranger).await.map(drop)), "a proposal");
    assert_eq!(store_bytes(tag, "a"), before, "nothing written");
    assert_eq!(chain_calls(&chain), calls, "no chain call");
}

// verifies: LLR-6kb33y
#[tokio::test]
async fn a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let tag = "propose-revocation-unheld";
    let alice = handle(tag, "a", &chain);
    let (org, _founder) = founded(&alice, "alice").await;
    let before = store_bytes(tag, "a");
    let stranger = MemberId::new([0x99; 32]);
    assert!(matches!(refused(alice.propose_revocation(org, stranger).await), OrgNodeError::MemberNotHeld { .. }));
    assert_eq!(store_bytes(tag, "a"), before, "nothing written");
    assert_eq!(chain.update_writes(), 0, "nothing written to the chain");
    assert_eq!(alice.bound_endpoints_for_test().await, 0, "nothing sent");
}
