#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Value types and the rejection vocabulary (SDD-swtd3w).
//!
//! Relocated from the `#[cfg(test)]` modules in `org-node/src/ids.rs` so the
//! `verifies:` annotations sit under `test_paths`, which this unit's config
//! reads only from `org-node/tests`. The variant-distinctness and
//! error-carrying cases are new.

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::sequence::SeqGuard;
use org_node::SequenceNumber;

// verifies: REQ-gju89b, LLR-7gnrnz
#[test]
fn org_id_is_twenty_bytes_and_round_trips_through_postcard() {
    let id = OrgId::new([7u8; 20]);
    assert_eq!(id.as_bytes().len(), 20);
    let bytes = postcard::to_allocvec(&id).unwrap();
    let back: OrgId = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(id, back);
}

// verifies: REQ-gju89b, LLR-7gnrnz
#[test]
fn org_ids_differing_in_one_byte_are_not_equal() {
    let mut raw = [7u8; 20];
    let a = OrgId::new(raw);
    raw[19] = 8;
    assert_ne!(a, OrgId::new(raw));
}

// verifies: LLR-gu6u53
#[test]
fn org_id_debug_is_the_twenty_bytes_in_order_as_forty_lowercase_hex_digits() {
    // The fixture bytes are all DIFFERENT. Review round 1 found the previous
    // `[0xab; 20]` could not see byte order at all: reversing the iteration in
    // `OrgId`'s Debug left the assertion satisfied.
    let mut raw = [0u8; 20];
    for (i, b) in raw.iter_mut().enumerate() {
        *b = (i as u8) + 0xa0;
    }
    let rendered = format!("{:?}", OrgId::new(raw));

    let expected: String = raw.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(rendered, format!("OrgId(0x{expected})"));

    let inner = rendered.trim_start_matches("OrgId(0x").trim_end_matches(')');
    assert_eq!(inner.len(), 40, "twenty bytes is forty digits");
    assert!(inner.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    // First and last byte land at the ends, so a reversal is visible.
    assert!(inner.starts_with("a0"), "first byte must come first: {inner}");
    assert!(inner.ends_with("b3"), "last byte must come last: {inner}");
}

// verifies: REQ-9g6as6, REQ-bcxz96, LLR-z8fubr, LLR-mxskg9
#[test]
fn every_rejection_variant_is_distinct_from_every_other() {
    let all = [
        OrgNodeError::OrgIdMismatch,
        // `InvalidOrgPublicKey` is the chain-state refusal REQ-8jb4ny added.
        OrgNodeError::InvalidOrgPublicKey,
        OrgNodeError::StaleSeq { got: 1, last_seen: 2 },
        OrgNodeError::MalformedDelta,
        OrgNodeError::DeltaBaseMismatch,
        OrgNodeError::OrgNotOnChain,
        OrgNodeError::RootMismatch,
        OrgNodeError::StaleEpoch { got: 1, last: 2 },
        OrgNodeError::Chain("read failed".into()),
        OrgNodeError::Trie(org_members::OrgMembersError::DuplicateHandle),
        // The Sequence number that is not the chain's epoch (REQ-txvtm9).
        OrgNodeError::SeqNotEpoch { seq: 1, epoch: 2 },
        // The two first-admission refusals (REQ-8amu2a, REQ-kt877x).
        OrgNodeError::AdmissionNotExpected { org_id: OrgId::new([1; 20]) },
        OrgNodeError::AdmissionNotOurs { org_id: OrgId::new([1; 20]) },
        // The bound on one Organisation's provisional updates (REQ-fwfku9).
        OrgNodeError::ProvisionalLimit { limit: 1 },
        // No provisional update produces the chain's state (LLR-mxskg9).
        OrgNodeError::NoProvisionalUpdate,
    ];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "variants {i} and {j} compare equal");
            }
        }
    }
}

// Normal: each first-admission refusal names its Organisation; abnormal: two
// Organisations' refusals, and the two refusals of one, are not the same error.
// verifies: LLR-mxskg9, REQ-8amu2a, REQ-kt877x
#[test]
fn the_first_admission_refusals_name_their_organisation() {
    let org = OrgId::new([0xa0; 20]);
    let unexpected = OrgNodeError::AdmissionNotExpected { org_id: org };
    let not_ours = OrgNodeError::AdmissionNotOurs { org_id: org };
    for err in [&unexpected, &not_ours] {
        assert!(format!("{err}").contains(&format!("{org:?}")), "{err}");
    }
    assert_ne!(unexpected, OrgNodeError::AdmissionNotExpected { org_id: OrgId::new([0xa1; 20]) });
    assert_ne!(unexpected, not_ours);
}

// Normal: the refusal names the limit in bytes; abnormal: two limits differ.
// verifies: LLR-mxskg9, REQ-fwfku9
#[test]
fn the_provisional_limit_refusal_names_the_limit() {
    let err = OrgNodeError::ProvisionalLimit { limit: 1_048_576 };
    assert_eq!(err.to_string(), "provisional updates for one Organisation would exceed 1048576 bytes");
    assert_ne!(err, OrgNodeError::ProvisionalLimit { limit: 1 });
}

// verifies: REQ-9g6as6, REQ-bcxz96, LLR-z8fubr
#[test]
fn a_stale_sequence_rejection_carries_the_offered_number_and_the_mark() {
    // Rewritten 2026-10-04 after review round 1. The previous version
    // destructured literals it had just constructed and then asserted they
    // equalled themselves, and checked the rendering with `contains('4') &&
    // contains('9')` — symmetric, so exchanging the two fields was invisible.
    // This one takes the error from a real producer and pins the ORDER of the
    // two numbers in the message, which is the only part a reader relies on.
    let guard = SeqGuard::from_last_seen(SequenceNumber::new(9));
    let err = guard.check(SequenceNumber::new(4)).unwrap_err();
    assert_eq!(err, OrgNodeError::StaleSeq { got: 4, last_seen: 9 });

    let rendered = format!("{err}");
    let at_offered = rendered.find('4').expect("the offered number must be rendered");
    let at_mark = rendered.find('9').expect("the mark must be rendered");
    assert!(
        at_offered < at_mark,
        "the message must name the offered number before the mark it was judged against: {rendered}"
    );
}

// verifies: REQ-9g6as6, LLR-7cgg8a
#[test]
fn a_provider_error_is_carried_into_the_trie_variant_without_loss() {
    let provider = org_members::OrgMembersError::InvalidHandle("bad handle".into());
    let carried: OrgNodeError = provider.clone().into();
    match carried {
        OrgNodeError::Trie(inner) => assert_eq!(inner, provider),
        other => panic!("wrong variant: {other:?}"),
    }
}
