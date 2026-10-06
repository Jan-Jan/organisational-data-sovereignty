#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Expected admissions (REQ-8amu2a, RC-2ferct): a first admission is read
//! against the chain only when its Organisation and invite identifier match an
//! expectation the app declared, and committed only when the verified record
//! lists one of this node's Personas (REQ-kt877x).

mod support;

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::service::{MockChainOps, OrgService};
use org_node::store::ExpectedAdmission;
use org_node::test_fixtures::{admin_device, genesis_trie};
use org_node::transport::wire::WireMessage;
use org_node::{InviteId, Joiner, MemberSeed};
use rand::rngs::OsRng;
use support::*;

/// A second joining node that expects nothing yet, reading through `counting`.
fn second_joiner(tag: &str, counting: &CountingChain) -> (OrgService, org_node::keys::SigningKeypair, Joiner) {
    let mut svc = OrgService::new(open_store(tag, "b2", "pw_b2"), Box::new(counting.clone()));
    let pid = svc.create_persona(&mut OsRng, h("bea"), nm("Bea"), sn("Second")).unwrap();
    let joiner = joiner_of(&svc, &pid);
    let kp = device_kp(&svc, &pid);
    (svc, kp, joiner)
}

fn expectation(org_id: OrgId, invite_id: InviteId) -> ExpectedAdmission {
    ExpectedAdmission { org_id, invite_id }
}

// Normal: an expected first admission commits and its expectation is
// cleared, in memory and on disk.
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-q8emds, LLR-mbjfq8
#[tokio::test(flavor = "multi_thread")]
async fn an_expected_first_admission_is_committed_and_clears_the_expectation() {
    let s = setup("expected").await;
    let id = invite_for(&s.joiner_b.device_key);
    assert_eq!(s.svc_b.expected_admissions(), &[expectation(s.org_id, id)]);
    let s = admit_b_directly(s).await;
    assert!(s.svc_b.expected_admissions().is_empty());
    assert!(reopen_store("expected", "b", "pw_b").data().expected_admissions.is_empty());
}

// Abnormal: an unexpected first admission — no expectation at all — is refused
// before its snapshot is decoded and without a chain read, and nothing is
// written.
// verifies: REQ-8amu2a, LLR-s8xp7m
#[tokio::test(flavor = "multi_thread")]
async fn an_unexpected_first_admission_is_refused_before_the_snapshot_or_the_chain() {
    let (mut s, counting) = setup_counted("unexpected").await;
    let (mut svc_b2, kp, joiner) = second_joiner("unexpected", &counting);
    let msg = captured_admission(&mut s, &joiner).await;
    let garbled = WireMessage { genesis_snapshot: Some(vec![0xff; 4]), ..msg.clone() };
    let on_disk = store_bytes("unexpected", "b2");
    let before = counting.reads();
    for m in [&msg, &garbled] {
        let (addr, task) = spawn_receive(svc_b2, &kp).await;
        deliver(addr, m).await;
        let (back, result) = task.await.unwrap();
        svc_b2 = back;
        assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotExpected { org_id: s.org_id });
    }
    assert_eq!(counting.reads(), before, "no chain read");
    assert!(svc_b2.list_orgs().is_empty());
    assert_eq!(store_bytes("unexpected", "b2"), on_disk, "nothing written");
}

// Abnormal: an expectation names one Organisation and admits no other.
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-y2v8v2
#[tokio::test(flavor = "multi_thread")]
async fn an_expectation_for_one_organisation_admits_no_other() {
    let (mut s, counting) = setup_counted("other-org").await;
    let (mut svc_b2, kp, joiner) = second_joiner("other-org", &counting);
    let elsewhere = OrgId::new([0x42; 20]);
    let id = invite_for(&joiner.device_key);
    svc_b2.expect_admission(&mut OsRng, elsewhere, id).unwrap();
    let msg = captured_admission(&mut s, &joiner).await;
    let (addr, task) = spawn_receive(svc_b2, &kp).await;
    deliver(addr, &msg).await;
    let (svc_b2, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotExpected { org_id: s.org_id });
    assert_eq!(svc_b2.expected_admissions(), &[expectation(elsewhere, id)], "the other expectation is untouched");
}

// Abnormal (pre-emption): for the right Organisation, a first admission under
// another invite identifier, or under none, is refused without a chain read
// and leaves the expectation; the genuine one then commits.
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-ms8njy
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_under_another_invite_id_is_refused_and_keeps_the_expectation() {
    let (mut s, counting) = setup_counted("other-id").await;
    let joiner = s.joiner_b.clone();
    let msg = captured_admission(&mut s, &joiner).await;
    let before = counting.reads();
    let mut svc_b = s.svc_b;
    for invite_id in [Some(InviteId::new([0x13; 32])), None] {
        let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
        deliver(addr, &WireMessage { invite_id, ..msg.clone() }).await;
        let (back, result) = task.await.unwrap();
        svc_b = back;
        assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotExpected { org_id: s.org_id });
    }
    assert_eq!(counting.reads(), before, "no chain read");
    assert_eq!(svc_b.expected_admissions(), &[expectation(s.org_id, invite_for(&joiner.device_key))]);
    let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(addr, &msg).await;
    let (svc_b, result) = task.await.unwrap();
    result.expect("the genuine admission still commits");
    assert!(svc_b.expected_admissions().is_empty());
}

// Abnormal (pre-emption, REQ-kt877x): a genuine admission of someone else,
// relayed under this node's invite identifier, verifies against the chain
// but lists none of this node's Personas: refused, nothing created, nothing
// written, the expectation kept; this node's own admission then commits.
// verifies: REQ-kt877x, LLR-3f5h7b, LLR-mxskg9
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_that_lists_none_of_our_personas_is_refused() {
    let (mut s, counting) = setup_counted("not-ours").await;
    let (mut svc_b2, kp, joiner_b2) = second_joiner("not-ours", &counting);
    let id_b2 = invite_for(&joiner_b2.device_key);
    svc_b2.expect_admission(&mut OsRng, s.org_id, id_b2).unwrap();
    // A admits B (epoch 2); the relay re-labels B's admission with B2's id.
    let joiner_b = s.joiner_b.clone();
    let of_b = captured_admission(&mut s, &joiner_b).await;
    let on_disk = store_bytes("not-ours", "b2");
    let (addr, task) = spawn_receive(svc_b2, &kp).await;
    deliver(addr, &WireMessage { invite_id: Some(id_b2), ..of_b }).await;
    let (back, result) = task.await.unwrap();
    svc_b2 = back;
    assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotOurs { org_id: s.org_id });
    assert!(svc_b2.list_orgs().is_empty(), "no record of an Organisation it is not in");
    assert_eq!(svc_b2.list_personas()[0].status, org_node::store::PersonaStatus::Proposed);
    assert_eq!(svc_b2.expected_admissions(), &[expectation(s.org_id, id_b2)]);
    assert_eq!(store_bytes("not-ours", "b2"), on_disk, "nothing written");
    // Normal: B2's own admission (epoch 3) commits.
    let (addr, task) = spawn_receive(svc_b2, &kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &joiner_b2, addr, org_secret()).await.unwrap();
    let (svc_b2, result) = task.await.unwrap();
    assert_eq!(result.unwrap().epoch, org_node::Epoch::new(3));
    assert!(svc_b2.expected_admissions().is_empty());
}

// Normal and abnormal: a declaration is recorded once however often it is
// made, a pair differing in either part is a second entry, and each reaches
// the disk before `expect_admission` returns.
// verifies: REQ-8amu2a, LLR-9zfnmb, LLR-95753m
#[test]
fn expect_admission_records_a_pair_once_and_reaches_the_disk() {
    let mut svc = OrgService::new(open_store("once", "b", "pw_b"), Box::new(MockChainOps::new()));
    let (org, other) = (OrgId::new([0x42; 20]), OrgId::new([0x43; 20]));
    let (x, y) = (InviteId::new([1; 32]), InviteId::new([2; 32]));
    svc.expect_admission(&mut OsRng, org, x).unwrap();
    svc.expect_admission(&mut OsRng, org, x).unwrap();
    assert_eq!(svc.expected_admissions(), &[expectation(org, x)]);
    assert_eq!(reopen_store("once", "b", "pw_b").data().expected_admissions, vec![expectation(org, x)]);
    svc.expect_admission(&mut OsRng, org, y).unwrap();
    svc.expect_admission(&mut OsRng, other, x).unwrap();
    assert_eq!(
        reopen_store("once", "b", "pw_b").data().expected_admissions,
        vec![expectation(org, x), expectation(org, y), expectation(other, x)]
    );
}

// Abnormal: a first admission refused after the expectation check leaves the
// expectation in place.
// verifies: REQ-8amu2a, LLR-q8emds
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_first_admission_leaves_the_expectation() {
    let mut s = setup("refused-keeps").await;
    let joiner = s.joiner_b.clone();
    let msg = captured_admission(&mut s, &joiner).await;
    let stranger = MemberSeed::from([0x5a; 32]).x25519_keypair();
    let wrong_base = snapshot_bytes(&genesis_trie(&stranger, &admin_device()));
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(addr, &WireMessage { genesis_snapshot: Some(wrong_base), ..msg }).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::DeltaBaseMismatch);
    assert_eq!(svc_b.expected_admissions(), &[expectation(s.org_id, invite_for(&joiner.device_key))]);
    assert!(svc_b.list_orgs().is_empty());
}

// Normal: the commit clears only the expectation it matched; another for the
// same Organisation (a second Invite) stays.
// verifies: REQ-8amu2a, LLR-q8emds
#[tokio::test(flavor = "multi_thread")]
async fn a_second_expectation_for_the_same_organisation_survives_the_commit() {
    let mut s = setup("two-ids").await;
    let spare = InviteId::new([0x77; 32]);
    s.svc_b.expect_admission(&mut OsRng, s.org_id, spare).unwrap();
    let s = admit_b_directly(s).await;
    assert_eq!(s.svc_b.expected_admissions(), &[expectation(s.org_id, spare)]);
}

// Abnormal: an expected first admission decodes its snapshot — and refuses
// one it lacks or cannot decode — before it reads the chain.
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-j6j95z
#[tokio::test(flavor = "multi_thread")]
async fn an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain() {
    let (mut s, counting) = setup_counted("decode-first").await;
    let joiner = s.joiner_b.clone();
    let msg = captured_admission(&mut s, &joiner).await;
    let before = counting.reads();
    let mut svc_b = s.svc_b;
    for snapshot in [Some(vec![0xff; 4]), None] {
        let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
        deliver(addr, &WireMessage { genesis_snapshot: snapshot, ..msg.clone() }).await;
        let (back, result) = task.await.unwrap();
        svc_b = back;
        assert!(matches!(result, Err(OrgNodeError::Chain(_))), "{result:?}");
    }
    assert_eq!(counting.reads(), before, "no chain read before the snapshot decodes");
    assert_eq!(svc_b.expected_admissions(), &[expectation(s.org_id, invite_for(&joiner.device_key))]);
}
