//! The follow-up decision (SDD-6zqqm8): what the app asks org-io for next
//! after a verification or a write it ran. Pure: it reads org-io's outcome
//! and returns the decision; the command surface (SDD-6pjvhc) makes the
//! call it names.

use org_io::node::OrgId;
use org_io::{Verified, WriteReport};

/// What a verification's outcome asks org-io for next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationFollowUp {
    /// Send the Organisation's adopted epoch to its Devices (REQ-qv4d2z).
    Send(OrgId),
    /// Request the Organisation from its Members (REQ-vgn96n).
    Request(OrgId),
}

/// LLR-292r26. An adoption this node built asks a send, a `behind` result a
/// request, each for the Organisation the entry names; a settled
/// verification follows its outcome; every other outcome, and any outcome
/// of an entry naming no Organisation (a founding with no proxy), asks
/// nothing.
pub fn after_verification(verified: &Verified, org_id: Option<OrgId>) -> Option<VerificationFollowUp> {
    let org_id = org_id?;
    match verified {
        Verified::Adopted { own: true, .. } => Some(VerificationFollowUp::Send(org_id)),
        Verified::Behind { .. } => Some(VerificationFollowUp::Request(org_id)),
        Verified::Settled { outcome, .. } => after_verification(outcome, Some(org_id)),
        Verified::Adopted { own: false, .. }
        | Verified::InStep { .. }
        | Verified::Removed { .. }
        | Verified::Inconsistent { .. }
        | Verified::Unverified { .. }
        | Verified::Refused { .. }
        | Verified::Store { .. }
        | Verified::Keychain { .. } => None,
    }
}

/// LLR-bbfgk7. Only an included write asks a verification of its target.
pub fn verifies_after_write(report: &WriteReport) -> bool {
    match report {
        WriteReport::Included { .. } => true,
        WriteReport::Refused { .. } | WriteReport::Unknown { .. } => false,
    }
}
