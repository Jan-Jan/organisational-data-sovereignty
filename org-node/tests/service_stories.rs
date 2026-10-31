//! Integration test: the user stories via OrgService against chain states
//! from `ChainSlots`, each send the body built for the recipient, delivered
//! by value. Offline — no live chain, no relay.
//!
//! The gate: `cargo test -p org-node --features app,test-support --test service_stories`

#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;
use support::sweep;

// Abnormal input: a revocation notice naming this
// node's own Device, well formed and for the right Organisation, whose
// absence proof is genuine under a trie the chain never held, delivered
// by a Device the record does not list, is refused with `SenderNotListed`
// and nothing is kept (LLR-283x6f). From A's listed Device it is kept on
// receipt, and verification, whose `revocation::accept` refuses its proof
// under the chain's root that still lists B (LLR-rhsfw9), reports InStep,
// asks for no secret and leaves the record as it was.
// verifies: LLR-283x6f, LLR-rhsfw9
#[test]
fn revocation_from_an_unknown_device_leaves_the_record_in_place() {
    use org_node::error::OrgNodeError;
    use org_node::service::Checked;
    use org_node::store::PersonaStatus;
    use org_node::verify_org::Verified;
    use rand::rngs::OsRng;

    let (mut admin, _) = sweep::founded();
    let admission = sweep::admission(&mut admin, "Bob", "bob");
    let mut bob = admission.member;
    sweep::receive_and_adopt(&mut bob, admin.device, &admission.body, admission.chain);
    let notice = sweep::notice_about(support::the_org(), sweep::member_id_of(&bob), bob.device);
    let held = sweep::record_of(&bob).clone();
    let before = support::encoded_store(&bob.svc, &bob.device);

    let refused = bob.node.svc.check_received(&mut OsRng, &bob.device, support::relay_device(), &notice);
    assert_eq!(refused.unwrap_err(), OrgNodeError::SenderNotListed { org_id: support::the_org() });
    assert_eq!(support::encoded_store(&bob.svc, &bob.device), before, "nothing kept from an unlisted Device");

    let checked = sweep::receive(&mut bob, admin.device, &notice);
    assert!(matches!(checked, Checked::RevocationNoticeKept), "{checked:?}");
    let (verified, change) =
        bob.node.svc.verify(&mut OsRng, &bob.device, Some(admission.chain), support::story_time(), org_node::AheadLimit(1200), support::never_asked).unwrap();
    assert!(matches!(verified, Verified::InStep), "{verified:?}");
    assert!(change.is_none(), "InStep writes nothing");
    let after = sweep::record_of(&bob);
    assert_eq!((after.epoch, after.root_hash, after.trie_members.len()), (held.epoch, held.root_hash, held.trie_members.len()));
    assert_eq!(support::held_store(&bob.svc, &bob.device).persona.status, PersonaStatus::Active);
}
