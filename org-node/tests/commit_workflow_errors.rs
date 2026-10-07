#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Stage S3 (`docs/plans/2026-10-06-org-io-commit-workflow.md`).

mod support;

use org_members::OrgMembersError;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::Epoch;

fn all_new(org_id: OrgId) -> Vec<OrgNodeError> {
    vec![
        OrgNodeError::OrgNotHeld { org_id },
        OrgNodeError::StaleChainState { org_id, chain_epoch: Epoch::new(3), record_epoch: Epoch::new(4) },
        OrgNodeError::ChainStateConflict { org_id },
        OrgNodeError::RevocationProofRefused { org_id, cause: OrgMembersError::DeviceStillHeld },
        OrgNodeError::AcknowledgementNotHeld { org_id },
        OrgNodeError::AcknowledgementFromFuture { org_id },
        OrgNodeError::AcknowledgementForListedDevice { org_id },
        OrgNodeError::AcknowledgementSignatureInvalid { org_id },
        OrgNodeError::DeviceSecretNotSupplied { org_id },
        OrgNodeError::NoRevocationForRecipient { org_id },
    ]
}

/// verifies: LLR-n67aw8
///
/// Normal: each variant is distinct from every other and its `Display`
/// names the Organisation (in `OrgId`'s `Debug` form, as the existing
/// variants do; `OrgId` has no `Display`).
#[test]
fn the_ten_new_variants_are_distinct_and_name_the_organisation() {
    let org_id = OrgId::new([7; 20]);
    let organisation_name = format!("{org_id:?}");
    let errors = all_new(org_id);
    for (index, first) in errors.iter().enumerate() {
        for second in errors.iter().skip(index + 1) {
            assert_ne!(first.to_string(), second.to_string());
        }
        assert!(first.to_string().contains(&organisation_name), "{first} names the Organisation");
    }
}

/// verifies: LLR-n67aw8
///
/// Abnormal: the stale-state refusal names both epochs; the proof refusal
/// names its cause.
#[test]
fn stale_state_names_both_epochs_and_the_proof_refusal_names_its_cause() {
    let org_id = OrgId::new([7; 20]);
    let stale = OrgNodeError::StaleChainState { org_id, chain_epoch: Epoch::new(3), record_epoch: Epoch::new(4) };
    let text = stale.to_string();
    assert!(text.contains("epoch 3") && text.contains("epoch 4"), "{text}");
    let proof = OrgNodeError::RevocationProofRefused { org_id, cause: OrgMembersError::DeviceStillHeld };
    assert!(proof.to_string().contains(&OrgMembersError::DeviceStillHeld.to_string()));
}

/// verifies: LLR-2r2fha, LLR-kzgjz8, LLR-5azhry
///
/// The two sender refusals are distinct from every S3 variant and name the
/// Organisation.
#[test]
fn the_sender_refusals_are_distinct_and_name_the_organisation() {
    let org_id = OrgId::new([7; 20]);
    let sender = OrgNodeError::SenderNotListed { org_id };
    let ack = OrgNodeError::AcknowledgementNotFromItsDevice { org_id };
    assert_ne!(sender, ack);
    for error in [&sender, &ack] {
        assert!(error.to_string().contains(&format!("{org_id:?}")), "{error}");
        for earlier in all_new(org_id) {
            assert_ne!(*error, earlier);
            assert_ne!(error.to_string(), earlier.to_string());
        }
        assert_ne!(*error, OrgNodeError::RevocationNotForThisDevice { org_id });
    }
}
