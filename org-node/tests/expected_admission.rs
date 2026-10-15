#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Expected admissions (REQ-8amu2a, RC-2ferct): a first admission is read
//! against the chain only when its Organisation matches an expectation the
//! app declared, and committed only when the verified record
//! lists one of this node's Personas (REQ-kt877x).

mod support;

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::store::ExpectedAdmission;
use org_node::test_fixtures::{admin_device, genesis_trie};
use org_node::{Joiner, MemberSeed};
use rand::rngs::OsRng;
use support::*;

/// A second joining node that expects nothing yet, reading through `counting`.
fn second_joiner(tag: &str, counting: &CountingChain) -> (Node, Joiner) {
    let mut svc = Node::new(open_store(tag, "b2"), counting.clone());
    let pid = svc.create_persona(&mut OsRng, h("bea"), nm("Bea"), sn("Second")).unwrap();
    let joiner = joiner_of(&svc, &pid);
    (svc, joiner)
}

fn expectation(org_id: OrgId) -> ExpectedAdmission {
    ExpectedAdmission { org_id }
}

// Normal: an expected first admission commits and its expectation is
// cleared, in memory and on disk.
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-q8emds, LLR-mbjfq8
#[test]
fn an_expected_first_admission_is_committed_and_clears_the_expectation() {
    let s = setup("expected");
    assert_eq!(s.svc_b.expected_admissions(), &[expectation(s.org_id)]);
    let s = admit_b_directly(s);
    assert!(s.svc_b.expected_admissions().is_empty());
    assert!(reopen_store("expected", "b").data().expected_admissions.is_empty());
}

// Abnormal: an unexpected first admission — no expectation at all — is refused
// before its snapshot is decoded and without a chain read, and nothing is
// written.
// verifies: REQ-8amu2a, LLR-s8xp7m
#[test]
fn an_unexpected_first_admission_is_refused_before_the_snapshot_or_the_chain() {
    let (mut s, counting) = setup_counted("unexpected");
    let (mut svc_b2, joiner) = second_joiner("unexpected", &counting);
    let msg = captured_admission(&mut s, &joiner);
    let garbled = with_snapshot(&msg, vec![0xff; 4]);
    let on_disk = store_bytes("unexpected", "b2");
    let before = counting.reads();
    for m in [&msg, &garbled] {
        let (addr, task) = spawn_receive(svc_b2);
        deliver(addr, m);
        let (back, result) = task.run();
        svc_b2 = back;
        assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotExpected { org_id: s.org_id });
    }
    assert_eq!(counting.reads(), before, "no chain read");
    assert!(svc_b2.list_orgs().is_empty());
    assert_eq!(store_bytes("unexpected", "b2"), on_disk, "nothing written");
}

// Abnormal: an expectation names one Organisation and admits no other.
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-y2v8v2
#[test]
fn an_expectation_for_one_organisation_admits_no_other() {
    let (mut s, counting) = setup_counted("other-org");
    let (mut svc_b2, joiner) = second_joiner("other-org", &counting);
    let elsewhere = OrgId::new([0x42; 20]);
    svc_b2.expect_admission(&mut OsRng, elsewhere).unwrap();
    let msg = captured_admission(&mut s, &joiner);
    let (addr, task) = spawn_receive(svc_b2);
    deliver(addr, &msg);
    let (svc_b2, result) = task.run();
    assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotExpected { org_id: s.org_id });
    assert_eq!(svc_b2.expected_admissions(), &[expectation(elsewhere)], "the other expectation is untouched");
}

// Abnormal (pre-emption, REQ-kt877x): a genuine admission of someone else,
// delivered to a node that expects an admission to that Organisation,
// verifies against the chain but lists none of this node's Personas:
// refused, nothing created, nothing written, the expectation kept; this
// node's own admission then commits.
// verifies: REQ-kt877x, LLR-3f5h7b, LLR-mxskg9
#[test]
fn a_first_admission_that_lists_none_of_our_personas_is_refused() {
    let (mut s, counting) = setup_counted("not-ours");
    let (mut svc_b2, joiner_b2) = second_joiner("not-ours", &counting);
    svc_b2.expect_admission(&mut OsRng, s.org_id).unwrap();
    // A admits B (epoch 2); a relay hands B's admission to B2.
    let joiner_b = s.joiner_b.clone();
    let of_b = captured_admission(&mut s, &joiner_b);
    let on_disk = store_bytes("not-ours", "b2");
    let (addr, task) = spawn_receive(svc_b2);
    deliver(addr, &of_b);
    let (back, result) = task.run();
    svc_b2 = back;
    assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotOurs { org_id: s.org_id });
    assert!(svc_b2.list_orgs().is_empty(), "no record of an Organisation it is not in");
    assert_eq!(svc_b2.list_personas()[0].status, org_node::store::PersonaStatus::Proposed);
    assert_eq!(svc_b2.expected_admissions(), &[expectation(s.org_id)]);
    assert_eq!(store_bytes("not-ours", "b2"), on_disk, "nothing written");
    // Normal: B2's own admission (epoch 3) commits.
    let (addr, task) = spawn_receive(svc_b2);
    admit(&mut s.svc_a, &s.chain, s.org_id, &joiner_b2, addr).unwrap();
    let (svc_b2, result) = task.run();
    assert_eq!(result.unwrap().epoch, org_node::Epoch::new(3));
    assert!(svc_b2.expected_admissions().is_empty());
}

// Normal and abnormal: a declaration is recorded once however often it is
// made, another Organisation is a second entry, and each reaches the disk
// before `expect_admission` returns.
// *Renamed 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `expect_admission_records_a_pair_once_and_reaches_the_disk`: an expectation
// named an invite identifier too.
// verifies: REQ-8amu2a, LLR-9zfnmb, LLR-95753m
#[test]
fn expect_admission_records_an_organisation_once_and_reaches_the_disk() {
    let mut svc = Node::new(open_store("once", "b"), org_node::test_fixtures::ChainSlots::new());
    let _persona = svc.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("B")).unwrap();
    let (org, other) = (OrgId::new([0x42; 20]), OrgId::new([0x43; 20]));
    svc.expect_admission(&mut OsRng, org).unwrap();
    svc.expect_admission(&mut OsRng, org).unwrap();
    assert_eq!(svc.expected_admissions(), &[expectation(org)]);
    assert_eq!(reopen_store("once", "b").data().expected_admissions, vec![expectation(org)]);
    svc.expect_admission(&mut OsRng, other).unwrap();
    assert_eq!(
        reopen_store("once", "b").data().expected_admissions,
        vec![expectation(org), expectation(other)]
    );
}

// Abnormal: a first admission refused after the expectation check leaves the
// expectation in place.
// verifies: REQ-8amu2a, LLR-q8emds
#[test]
fn a_refused_first_admission_leaves_the_expectation() {
    let mut s = setup("refused-keeps");
    let joiner = s.joiner_b.clone();
    let msg = captured_admission(&mut s, &joiner);
    let stranger = MemberSeed::from([0x5a; 32]).x25519_keypair();
    let wrong_base = snapshot_bytes(&genesis_trie(&stranger, &admin_device()));
    let (addr, task) = spawn_receive(s.svc_b);
    deliver(addr, &with_snapshot(&msg, wrong_base));
    let (svc_b, result) = task.run();
    assert_eq!(result.unwrap_err(), OrgNodeError::DeltaBaseMismatch);
    assert_eq!(svc_b.expected_admissions(), &[expectation(s.org_id)]);
    assert!(svc_b.list_orgs().is_empty());
}

// Normal: the commit clears only the expectation for the Organisation it
// admitted to; one for another Organisation stays. (At most one is held per
// Organisation, LLR-9zfnmb.)
// *Rewritten 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `a_second_expectation_for_the_same_organisation_survives_the_commit`, of a
// second invite identifier for the same Organisation.
// verifies: REQ-8amu2a, LLR-q8emds
#[test]
fn an_expectation_for_another_organisation_survives_the_commit() {
    let mut s = setup("two-orgs-expected");
    let elsewhere = OrgId::new([0x77; 20]);
    s.svc_b.expect_admission(&mut OsRng, elsewhere).unwrap();
    let s = admit_b_directly(s);
    assert_eq!(s.svc_b.expected_admissions(), &[expectation(elsewhere)]);
}

// Abnormal: an expected first admission decodes its snapshot — and refuses
// one it cannot decode — before it reads the chain. (One without a snapshot
// is a revocation, refused before the expectations:
// `admission_sender::a_revocation_about_an_organisation_not_held_is_refused_before_the_chain`.)
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-j6j95z
#[test]
fn an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain() {
    let (mut s, counting) = setup_counted("decode-first");
    let joiner = s.joiner_b.clone();
    let msg = captured_admission(&mut s, &joiner);
    let before = counting.reads();
    let (addr, task) = spawn_receive(s.svc_b);
    deliver(addr, &with_snapshot(&msg, vec![0xff; 4]));
    let (svc_b, result) = task.run();
    assert!(matches!(result, Err(OrgNodeError::Chain(_))), "{result:?}");
    assert_eq!(counting.reads(), before, "no chain read before the snapshot decodes");
    assert_eq!(svc_b.expected_admissions(), &[expectation(s.org_id)]);
}

// --- LLR-rdda86: the one store a first admission commits into (review round
// 1 of S4, finding-12) ---

/// A's admission of `joiner`, committed at A: the body for the joiner's
/// Device and the Device A sends it from.
fn admission_body(s: &mut Setup, joiner: &Joiner) -> (org_node::DevicePublicKey, Vec<u8>) {
    let (_, outcome) = commit_admission(&mut s.svc_a, &s.chain, s.org_id, joiner).unwrap();
    (s.svc_a.sender_for(s.org_id), body_for(&outcome, joiner.device_key).unwrap())
}

// Abnormal: two stores on one node, one that declared the expectation but
// whose Device the record does not list (carol), and one whose Device the
// record lists but that declared nothing (bob). Neither is both, so the
// admission is refused as not ours: nothing written, no Persona bound, the
// expectation kept.
// verifies: LLR-rdda86
#[test]
fn a_first_admission_commits_into_no_store_that_only_declared_or_is_only_listed() {
    let tag = "rdda86-split";
    let mut s = setup_with(tag, false);
    let pid_c = s.svc_b.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Declared")).unwrap();
    s.svc_b.expect_admission_for(&mut OsRng, &pid_c, s.org_id).unwrap();
    let joiner_b = s.joiner_b.clone();
    let (sender, body) = admission_body(&mut s, &joiner_b);
    let on_disk = store_bytes(tag, "b");

    let refused = s.svc_b.receive_from(&mut OsRng, sender, &body).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::AdmissionNotOurs { org_id: s.org_id }));
    assert!(s.svc_b.list_orgs().is_empty(), "no record committed");
    assert!(s.svc_b.list_personas().iter().all(|persona| persona.org_id.is_none()), "no Persona bound");
    assert_eq!(s.svc_b.store_of(&pid_c).expected_admissions, vec![expectation(s.org_id)]);
    assert!(s.svc_b.store_of(&s.pid_b).expected_admissions.is_empty());
    assert_eq!(store_bytes(tag, "b"), on_disk, "nothing written");
}

// Normal beside abnormal: the store that declared the expectation and is
// listed (dave) is chosen over an earlier unbound store (carol), which is
// left untouched. (Carol cannot declare the same expectation as well: one
// Persona per Organisation, `OrganisationHeldByAnotherPersona`; the case of
// a store that only declared is the test above.)
// verifies: LLR-rdda86
#[test]
fn a_first_admission_commits_into_the_store_that_declared_and_is_listed() {
    let tag = "rdda86-chosen";
    let (mut s, counting) = setup_counted(tag);
    let mut node = Node::new(open_store(tag, "b2"), counting.clone());
    let carol = node.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Earlier")).unwrap();
    let dave = node.create_persona(&mut OsRng, h("dave"), nm("Dave"), sn("Listed")).unwrap();
    node.expect_admission_for(&mut OsRng, &dave, s.org_id).unwrap();
    let joiner = joiner_of(&node, &dave);
    let (sender, body) = admission_body(&mut s, &joiner);
    let carol_before = format!("{:?}", node.store_of(&carol));

    let committed = node.receive_from(&mut OsRng, sender, &body).unwrap().committed();
    assert_eq!(committed.org_id, s.org_id);
    assert_eq!(persona_of(&node, &dave).org_id, Some(s.org_id), "dave's Persona is bound");
    assert!(node.store_of(&dave).expected_admissions.is_empty(), "dave's expectation is cleared");
    assert_eq!(format!("{:?}", node.store_of(&carol)), carol_before, "carol's store is untouched");
    assert_eq!(persona_of(&node, &carol).org_id, None);
}
