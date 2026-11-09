#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The bound on one Organisation's epoch state (REQ-g2537c, LLR-m76r4k):
//! every keep measures the postcard encoding of `(provisional_changes,
//! provisional_organisations, kept_revocation_notices)` the store would hold
//! after it, and refuses past `MAX_PROVISIONAL_BYTES`, keeping nothing.
//!
//! Here: the bound over a received provisional organisation's keep. The
//! keeps of `record_admission`, `record_revocation` and
//! `propose_updated_organisation` are tested beside those operations, in
//! proposals.rs.

mod support;
use support::*;

use org_node::service::Checked;
use org_node::store::{Origin, ProvisionalChange, ProvisionalOrganisation, ProvisionalOrganisationParts};
use org_node::wire::encode_message;
use org_node::{Epoch, MemberId, OrgNodeError, OrgPrivateKey, PersonaId, RootHash, MAX_PROVISIONAL_BYTES};

/// A provisional organisation for a later epoch whose Change set is `bytes`
/// long: epoch state that only fills the bound.
fn filler(bytes: usize) -> ProvisionalOrganisation {
    ProvisionalOrganisation::from(ProvisionalOrganisationParts {
        epoch: Epoch::new(9),
        base_root: RootHash::new([0x91; 32]),
        members: vec![],
        root: RootHash::new([0x92; 32]),
        org_pub_key: org_node::test_fixtures::org_public_key(),
        org_private_key: OrgPrivateKey::from([0x93; 32]),
        change_set: Some(vec![0xab; bytes]),
        base_snapshot: None,
        origin: Origin::Own,
        applied: vec![],
        first_ahead_at: None,
    })
}

/// B admitted, holding a filler of `bytes`; the Organisation information A's
/// Device sends B for epoch 3, encoded; B's Persona and A's Device.
fn b_with_filler(tag: &str, bytes: usize) -> (Setup, PersonaId, org_node::DevicePublicKey, Vec<u8>) {
    let mut story = admit_b(setup(tag));
    let b_persona_id = story.b_persona_id.clone();
    let mut store_data = story.node_b.store_of(&b_persona_id).clone();
    store_data.provisional_organisations.push(filler(bytes));
    story.node_b.put(store_data);
    let a_device = persona_of(&story.node_a, &story.a_persona_id).device_key;
    let (message, _) = information_after(&record_of(&story.node_b, story.org_id), 0x70);
    (story, b_persona_id, a_device, encode_message(&message).unwrap())
}

/// The filler size that makes the store B keeps after `body` measure
/// exactly `target` bytes: measured once with a small filler, then shifted
/// (the Change set's length prefix is three bytes either way).
fn filler_for(tag: &str, target: usize) -> usize {
    let first = 100_000;
    let (mut story, b_persona_id, a_device, body) = b_with_filler(tag, first);
    let (checked, _sealed) = check(&mut story.node_b, &b_persona_id, a_device, &body).unwrap();
    assert!(matches!(checked, Checked::ProvisionalOrganisationKept { .. }), "fixture: the small filler is kept: {checked:?}");
    first + (target - fixtures::bound_len(story.node_b.store_of(&b_persona_id)))
}

// Abnormal: a keep that would bring the epoch state past 1 MiB is refused
// with `ProvisionalLimit`, keeping nothing and returning no change; the
// store is exactly as it was.
// verifies: LLR-m76r4k
#[test]
fn a_keep_past_one_mebibyte_is_refused_with_provisional_limit_and_changes_nothing() {
    let (mut story, b_persona_id, a_device, body) = b_with_filler("ps-past", MAX_PROVISIONAL_BYTES);
    let before = postcard::to_allocvec(story.node_b.store_of(&b_persona_id)).unwrap();
    assert_eq!(
        check(&mut story.node_b, &b_persona_id, a_device, &body).err(),
        Some(OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES })
    );
    assert_eq!(postcard::to_allocvec(story.node_b.store_of(&b_persona_id)).unwrap(), before, "StoreData exactly as it was");
}

// Normal (boundary):
// a keep that brings the epoch state to exactly 1 MiB is kept; the bound is
// a constant of its own, equal to the Wire message's.
// verifies: REQ-g2537c, LLR-m76r4k
#[test]
fn epoch_state_of_exactly_the_bound_is_kept() {
    assert_eq!(MAX_PROVISIONAL_BYTES, org_node::wire::MAX_MESSAGE);
    let bytes = filler_for("ps-exact-measure", MAX_PROVISIONAL_BYTES);
    let (mut story, b_persona_id, a_device, body) = b_with_filler("ps-exact", bytes);
    let (checked, change) = check(&mut story.node_b, &b_persona_id, a_device, &body).unwrap();
    assert!(matches!(checked, Checked::ProvisionalOrganisationKept { .. }), "{checked:?}");
    assert!(change.is_some());
    assert_eq!(fixtures::bound_len(story.node_b.store_of(&b_persona_id)), MAX_PROVISIONAL_BYTES, "exactly the bound");
}

// Abnormal: one byte over is refused naming the limit, nothing changed; the bound
// measures the provisional changes and kept notices too — with a recorded
// change taking the room, the keep that fitted alone is refused.
// verifies: REQ-g2537c, LLR-m76r4k, LLR-3sddey
#[test]
fn a_keep_that_would_exceed_the_bound_is_refused_and_nothing_changes() {
    let bytes = filler_for("ps-over-measure", MAX_PROVISIONAL_BYTES + 1);
    let (mut story, b_persona_id, a_device, body) = b_with_filler("ps-over", bytes);
    let before = postcard::to_allocvec(story.node_b.store_of(&b_persona_id)).unwrap();
    let refused = check(&mut story.node_b, &b_persona_id, a_device, &body).err().unwrap();
    assert_eq!(refused, OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES });
    assert!(refused.to_string().contains("1048576"), "{refused}");
    assert_eq!(postcard::to_allocvec(story.node_b.store_of(&b_persona_id)).unwrap(), before);

    let bytes = filler_for("ps-changes-measure", MAX_PROVISIONAL_BYTES);
    let (mut story, b_persona_id, a_device, body) = b_with_filler("ps-changes", bytes);
    let mut store_data = story.node_b.store_of(&b_persona_id).clone();
    store_data.provisional_changes.push(ProvisionalChange::Revoke { member_id: MemberId::new([0x55; 32]) });
    story.node_b.put(store_data);
    assert_eq!(
        check(&mut story.node_b, &b_persona_id, a_device, &body).err(),
        Some(OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES }),
        "the recorded change counts"
    );
    let mut store_data = story.node_b.store_of(&b_persona_id).clone();
    store_data.provisional_changes.clear();
    store_data.kept_revocation_notices.push(org_node::store::KeptNotice {
        notice: org_node::test_fixtures::bob_absence_notice(story.org_id),
        from: org_node::test_fixtures::device_key(0x21),
    });
    story.node_b.put(store_data);
    assert_eq!(
        check(&mut story.node_b, &b_persona_id, a_device, &body).err(),
        Some(OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES }),
        "the kept notice counts"
    );
}
