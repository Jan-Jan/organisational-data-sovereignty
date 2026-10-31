#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The receive path takes no chain state: what it refuses or decides it
//! does without one. `check_received`'s signature takes no chain. An
//! acknowledgement is decided in the check, an update from an unlisted
//! Device is refused there, and a revocation is refused there in a store
//! that holds no record of its Organisation (LLR-95av3e).

mod support;

use org_node::error::OrgNodeError;
use org_node::revocation::sign_acknowledgement;
use org_node::service::Checked;
use org_node::wire::{encode_message, WireMessage};

use support::*;

/// verifies: LLR-5wg8cu, LLR-5azhry
///
/// A's receive path reports B's acknowledgement, sent from B's own Device —
/// `Acknowledged`, decided in the check — with no write; an acknowledgement
/// with a forged signature from B's Device is refused, and B's genuine
/// acknowledgement from a random relay is refused with
/// `AcknowledgementNotFromItsDevice`; nothing written. B's acknowledgement
/// is signed with `revocation::sign_acknowledgement` over A's removal of B,
/// and B's admission and removal run through the operations
/// (`support::admit_b`, `remove_b`).
#[test]
fn an_acknowledgement_is_checked_and_reported() {
    let story = admit_b(setup("ack-report"));
    let (mut story, _) = remove_b(story);
    let a_persona_id = story.a_persona_id.clone();
    let b_device = story.b_device_keypair.device_key().unwrap();
    let state = story.chain.get(&story.org_id).unwrap();
    let secret = story.node_b.keychain.secret(&b_device).unwrap();
    let acknowledgement =
        sign_acknowledgement(&story.node_b.store_of(&story.b_persona_id).persona, story.org_id, &state, &secret).unwrap();
    let before = store_bytes("ack-report", "a");
    let b_seed = story.node_b.keychain.seed(&b_device);

    let checked =
        receive_message_from(&mut story.node_a, &a_persona_id, b_seed, &WireMessage::Acknowledgement(acknowledgement.clone())).unwrap();
    let Checked::Acknowledged(verified) = checked else { panic!("not decided as an acknowledgement: {checked:?}") };
    assert_eq!(verified.into_inner(), acknowledgement);

    let not_from_its_device = Err(OrgNodeError::AcknowledgementNotFromItsDevice { org_id: story.org_id });
    let relayed =
        receive_message_from(&mut story.node_a, &a_persona_id, rand::random(), &WireMessage::Acknowledgement(acknowledgement.clone()));
    assert_eq!(relayed.map(|_| ()), not_from_its_device);

    let mut forged = acknowledgement;
    forged.signature.0[0] ^= 1;
    let refused = receive_message_from(&mut story.node_a, &a_persona_id, b_seed, &WireMessage::Acknowledgement(forged));
    assert!(matches!(refused, Err(OrgNodeError::AcknowledgementSignatureInvalid { .. })), "{refused:?}");

    assert_eq!(store_bytes("ack-report", "a"), before, "nothing written");
}

// REQ-uk9rw7, at the value boundary: an update about a held Organisation
// from a Device the current record does not list is refused by the check
// itself, which takes no chain state; nothing is written. From A's listed
// Device the same update is kept as a provisional organisation, the record
// unchanged. B's admission runs through the operations.
// verifies: REQ-uk9rw7, LLR-283x6f, REQ-mr5abb
#[test]
fn an_update_from_an_unlisted_device_is_refused_before_any_chain_state_is_asked_for() {
    let mut story = admit_b(setup("unlisted-value"));
    let b_device = persona_of(&story.node_b, &story.b_persona_id).device_key;
    let (update, _) = information_after(&record_of(&story.node_b, story.org_id), 0x75);
    let body = encode_message(&update).unwrap();
    let on_disk = store_bytes("unlisted-value", "b");
    let held = record_of(&story.node_b, story.org_id);

    let stranger = org_node::test_fixtures::device_key(0x6d);
    assert!(!held.trie_members.iter().any(|member| member.device_keys.contains(&stranger)));
    let not_listed = Err(OrgNodeError::SenderNotListed { org_id: story.org_id });
    assert_eq!(story.node_b.check_received(&mut rand::rngs::OsRng, &b_device, stranger, &body).map(|_| ()), not_listed);
    assert_eq!(store_bytes("unlisted-value", "b"), on_disk, "nothing written");

    let a_device = persona_of(&story.node_a, &story.a_persona_id).device_key;
    let (checked, change) = story.node_b.check_received(&mut rand::rngs::OsRng, &b_device, a_device, &body).unwrap();
    assert!(matches!(checked, Checked::ProvisionalOrganisationKept { .. }), "from a listed Device it is kept: {checked:?}");
    assert!(change.is_some());

    let after = record_of(&story.node_b, story.org_id);
    assert_eq!((after.epoch, after.root_hash, after.last_seq), (held.epoch, held.root_hash, held.last_seq));
}

// LLR-95av3e: in a store holding no record, a founding store or a
// provisional Member's, a revocation is refused with `OrgNotHeld` before any
// field past its kind and Organisation identifier is decoded: a body whose
// tail is garbage is refused the same way. Nothing is kept, no record is
// created, the expectation stays, and the call takes no chain state.
// verifies: LLR-95av3e
#[test]
fn a_revocation_in_a_store_holding_no_record_is_refused_as_org_not_held() {
    use org_node::MemberId;
    use rand::rngs::OsRng;

    // A founding store, its proxy recorded, not yet adopted.
    let mut founder = sweep::found();
    let unrelated = support::relay_device();
    let notice = sweep::notice_about(support::the_org(), MemberId::new([0x31; 32]), founder.device);
    let before = support::encoded_store(&founder.svc, &founder.device);
    let refused = founder.node.svc.check_received(&mut OsRng, &founder.device, unrelated, &notice);
    assert_eq!(refused.unwrap_err(), OrgNodeError::OrgNotHeld { org_id: support::the_org() });
    assert_eq!(support::encoded_store(&founder.svc, &founder.device), before, "nothing kept in the founding store");
    assert!(support::held_store(&founder.svc, &founder.device).record.is_none(), "no record created");

    // A provisional Member's store; the notice comes from the issuer.
    let (mut admin, _) = sweep::founded();
    let mut pending = sweep::admission(&mut admin, "Bob", "bob");
    let bob_device = pending.member.device;
    let notice = sweep::notice_about(support::the_org(), MemberId::new([0x32; 32]), bob_device);
    let mut garbled = notice[..21].to_vec(); // the kind and the twenty-byte Organisation identifier
    garbled.extend_from_slice(&[0xff; 12]);
    let before = support::encoded_store(&pending.member.svc, &bob_device);
    for body in [notice, garbled] {
        let refused = pending.member.svc.check_received(&mut OsRng, &bob_device, admin.device, &body);
        assert_eq!(refused.unwrap_err(), OrgNodeError::OrgNotHeld { org_id: support::the_org() }, "{} bytes", body.len());
    }
    assert_eq!(support::encoded_store(&pending.member.svc, &bob_device), before, "nothing kept in the provisional Member's store");
    let store_data = support::held_store(&pending.member.svc, &bob_device);
    assert!(store_data.record.is_none() && store_data.kept_revocation_notices.is_empty());
    assert_eq!(store_data.expected_admissions.len(), 1, "the expectation stays");
}

// LLR-95av3e: in a store holding a record, a revocation about another
// Organisation, even from a Device the record lists, is refused by
// `check_notice` with `RevocationNotHeld` naming that Organisation, keeping
// nothing.
// verifies: LLR-95av3e
#[test]
fn a_revocation_about_another_organisation_is_refused_as_revocation_not_held_and_writes_nothing() {
    use org_node::OrgId;
    use rand::rngs::OsRng;

    let (mut admin, _) = sweep::founded();
    let admission = sweep::admission(&mut admin, "Bob", "bob");
    let mut bob = admission.member;
    sweep::receive_and_adopt(&mut bob, admin.device, &admission.body, admission.chain);
    let b_id = sweep::member_id_of(&bob);
    let elsewhere = OrgId::new([9; 20]);
    let notice = sweep::notice_about(elsewhere, b_id, bob.device);
    let held = sweep::record_of(&bob).clone();
    let before = support::encoded_store(&bob.svc, &bob.device);
    let refused = bob.node.svc.check_received(&mut OsRng, &bob.device, admin.device, &notice);
    assert_eq!(refused.unwrap_err(), OrgNodeError::RevocationNotHeld { org_id: elsewhere });
    assert_eq!(support::encoded_store(&bob.svc, &bob.device), before, "nothing written");
    assert!(support::held_store(&bob.svc, &bob.device).kept_revocation_notices.is_empty(), "nothing kept");
    assert_eq!((sweep::record_of(&bob).epoch, sweep::record_of(&bob).root_hash), (held.epoch, held.root_hash));
}

// LLR-95av3e, LLR-q8emds: a revocation refused by a provisional Member's
// store leaves its expected admission in place, so the first admission
// arriving next is kept and adopted; the adoption's one sealed change
// clears the expectation it matched (LLR-q8emds).
// verifies: LLR-95av3e, LLR-q8emds
#[test]
fn after_a_refused_revocation_the_expected_admission_is_still_in_place() {
    use org_node::verify_org::Verified;
    use org_node::MemberId;
    use rand::rngs::OsRng;

    let (mut admin, _) = sweep::founded();
    let mut pending = sweep::admission(&mut admin, "Bob", "bob");
    let bob_device = pending.member.device;
    let notice = sweep::notice_about(support::the_org(), MemberId::new([0x33; 32]), bob_device);
    let refused = pending.member.svc.check_received(&mut OsRng, &bob_device, admin.device, &notice);
    assert_eq!(refused.unwrap_err(), OrgNodeError::OrgNotHeld { org_id: support::the_org() });
    assert_eq!(support::held_store(&pending.member.svc, &bob_device).expected_admissions.len(), 1, "the expectation stays");

    let verified = sweep::receive_and_adopt(&mut pending.member, admin.device, &pending.body, pending.chain);
    assert!(matches!(verified, Verified::Adopted { epoch, .. } if epoch == pending.chain.epoch), "{verified:?}");
    let store_data = support::held_store(&pending.member.svc, &bob_device);
    assert_eq!(store_data.record.as_ref().map(|record| record.org_id), Some(support::the_org()));
    assert!(store_data.expected_admissions.is_empty(), "settled by the adoption");
}
