//! What the receiver loop announces, and the guard that decides whether a
//! receiver loop may start.
//!
//! `emissions_for` is a total function from an outcome to the events that
//! outcome produces. It exists so the receiver's announcements can be tested
//! without a Tauri application, an OrgService, a chain or a peer — none of
//! which any gate in this repository can stand up.

use org_io::node::{OrgNodeError, ReceiveKind};
use org_io::ReceiveError;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// What one iteration of the receiver loop concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiverOutcome {
    /// This persona was revoked and deleted itself.
    SelfDeleted { org_id: String },
    /// A revoked Device's acknowledgement of its removal from this
    /// organisation verified (S3, LLR-2vg79y). Nothing is announced.
    AcknowledgementReceived { org_id: String },
    /// An update verified and the organisation record was read back.
    Updated { org_id: String, epoch: u64, root: String },
    /// REQ-2k7ys4 / REQ-dp95pv: an update verified and the record could NOT be
    /// read back. This variant is what stops `(0, String::new())` existing.
    RecordUnreadable { org_id: String },
    /// REQ-affyf5: verification failed. `org_id` is present when the failing
    /// update named an organisation and absent when the failure happened
    /// before one was identified.
    VerifyFailed { org_id: Option<String>, message: String },
    /// REQ-kn5rtx / RC-3rddh7: the receive path failed for a reason that is NOT
    /// a verdict on any update — a chain read or a transport failure. Nothing
    /// verified and nothing failed to verify, so this carries no organisation,
    /// no epoch, no root and no verdict.
    ReceiveError { message: String },
    /// REQ-jfxah3: the loop is about to exit.
    Stopped { reason: String },
}

/// RC-3rddh7 / REQ-kn5rtx. Which kind of failure a receive-path error is, from
/// the error's TYPE rather than from its rendered message.
///
/// The previous code called `e.to_string()` first and routed every `Err` into
/// `VerifyFailed`, which the frontend renders as a "✗ MISMATCH" row under
/// "Verified Updates (chain root match)". A transport hiccup was therefore
/// displayed to the operator as a root mismatch — a fresh instance of
/// HAZ-9fmhm4, the verification display misrepresenting the verification state.
///
/// `OrgNotOnChain` and `Trie(_)` are deliberately NOT verdicts, despite sounding
/// like two of the plainest ones. Each is also reachable from a purely LOCAL
/// condition on the receive path, with no bearing on whether anything verified:
///
/// - org-node's receive path (`OrgService::check_received` and
///   `OrgService::commit_checked`, `org-node/src/service.rs`) returns
///   `OrgNotOnChain` when the local store holds no record of the organisation,
///   and also when org-io read no chain state for it;
/// - the same path returns `Trie(_)` when the local members snapshot
///   fails to reconstruct (and `verify.rs` returns it again for the
///   local trie's own `root_hash()`).
///
/// The app cannot tell the two origins apart from the variant alone, and the
/// safety principle is that a verification verdict must not be claimed when it
/// cannot be supported. The asymmetry is the whole argument: a misfiled VERDICT
/// is the hazard — a local-store problem rendered as "✗ MISMATCH" under
/// "Verified Updates (chain root match)" is HAZ-9fmhm4 again, exactly what
/// RC-3rddh7 was minted to remove — while a misfiled RECEIVER ERROR is merely
/// less specific: the operator is told the receive path failed and shown the
/// message, which is true of both origins.
///
/// There is no wildcard arm, deliberately: a new `OrgNodeError` variant is a
/// compile error here, not a silent default into one of the two classes.
pub fn classify_receive_error(e: &OrgNodeError) -> ReceiverOutcome {
    match e {
        // Verdicts on an incoming update: each one means "this did not verify",
        // and each is produced only by `verify_envelope_against_chain`.
        // `StaleEpoch` refuses the update because the chain holds no state
        // newer than the node's record for it to be the change to.
        OrgNodeError::OrgIdMismatch
        | OrgNodeError::StaleSeq { .. }
        | OrgNodeError::MalformedDelta
        | OrgNodeError::DeltaBaseMismatch
        | OrgNodeError::RootMismatch
        | OrgNodeError::StaleEpoch { .. }
        // A Sequence number other than the chain's epoch (an org-node commit
        // rule) refuses the received update itself.
        | OrgNodeError::SeqNotEpoch { .. } => ReceiverOutcome::VerifyFailed {
            // REQ-affyf5: the organisation is absent rather than invented. The
            // service reports the failure without naming one.
            org_id: None,
            message: e.to_string(),
        },

        // Not a verdict: the chain could not be read or answered with an
        // Organisation state the node refuses (`InvalidOrgPublicKey`, before
        // anything is verified against it), the transport failed, the local
        // store could not supply what the verification needed, or a value
        // read from the store or a received snapshot is not one its type
        // admits (`InvalidField` — before org-node parsed these they surfaced
        // as Chain or Trie, so the class is unchanged). An unexpected first
        // admission is refused before anything is verified
        // (`AdmissionNotExpected`), one that lists none of this node's
        // Personas on a rule about this node (`AdmissionNotOurs`), and the
        // provisional-update refusals never arise on receive. A Persona
        // already bound to an Organisation (`PersonaAlreadyBound`) is a
        // refusal about this node's own store, before anything is verified.
        // The four refusals of a received Wire message are not verdicts of
        // `verify_envelope_against_chain` either (LLR-7bk6qh): a message that
        // does not decode (`MalformedMessage`) and a revocation about an
        // Organisation not held (`RevocationNotHeld`) are refused before
        // anything is verified; a key that is not the chain's
        // (`OrgKeyMismatch`) and a revocation that leaves this node listed
        // (`RevocationNotForThisDevice`) after it, on a rule about the key or
        // this node. The nine refusals of the commit workflow (S3) are not
        // verdicts on an update either: reconcile's refusals concern a chain
        // state read or the node's own record (`OrgNotHeld`,
        // `StaleChainState`, `ChainStateConflict`); a revocation's proof and
        // an acknowledgement are not updates (`RevocationProofRefused`, the
        // four `Acknowledgement*`); and the last concerns what the caller
        // supplied (`DeviceSecretNotSupplied`).
        // The two sender refusals of the owner rulings of 2026-10-07 are
        // made before anything is verified, before the chain is read: a
        // message from a Device the record does not list (`SenderNotListed`)
        // and an acknowledgement not sent by its own Device
        // (`AcknowledgementNotFromItsDevice`). A Wire message over the bound
        // (`MessageTooLarge`) is refused before it is decoded.
        // The store and invitation refusals are about this node's own store
        // or its invitation bookkeeping, never verdicts on an update: a sealed
        // store refused (`StoreRefused`), a Device secret that is not the
        // store's (`DeviceSecretMismatch`), two stores for one Device
        // (`DuplicatePersonaStore`), a reply for another Organisation or not
        // outstanding (`ReplyForAnotherOrganisation`, `InviteNotOutstanding`),
        // and an Organisation another Persona holds or expects
        // (`OrganisationHeldByAnotherPersona`).
        // A commit refused because this node's own record
        // moved after the message was checked (`RecordMovedSinceCheck`) is
        // about this node's store, not a verdict on the update; org-io
        // checks the body once more before it reports one.
        OrgNodeError::Chain(_)
        | OrgNodeError::OrgNotOnChain
        | OrgNodeError::Trie(_)
        | OrgNodeError::InvalidOrgPublicKey
        | OrgNodeError::InvalidField { .. }
        | OrgNodeError::AdmissionNotExpected { .. }
        | OrgNodeError::AdmissionNotOurs { .. }
        | OrgNodeError::ProvisionalLimit { .. }
        | OrgNodeError::NoProvisionalUpdate
        | OrgNodeError::PersonaAlreadyBound { .. }
        | OrgNodeError::MalformedMessage
        | OrgNodeError::MessageTooLarge { .. }
        | OrgNodeError::OrgKeyMismatch { .. }
        | OrgNodeError::RevocationNotHeld { .. }
        | OrgNodeError::RevocationNotForThisDevice { .. }
        | OrgNodeError::OrgNotHeld { .. }
        | OrgNodeError::StaleChainState { .. }
        | OrgNodeError::ChainStateConflict { .. }
        | OrgNodeError::RevocationProofRefused { .. }
        | OrgNodeError::AcknowledgementNotHeld { .. }
        | OrgNodeError::AcknowledgementFromFuture { .. }
        | OrgNodeError::AcknowledgementForListedDevice { .. }
        | OrgNodeError::AcknowledgementSignatureInvalid { .. }
        | OrgNodeError::DeviceSecretNotSupplied { .. }
        | OrgNodeError::SenderNotListed { .. }
        | OrgNodeError::AcknowledgementNotFromItsDevice { .. }
        | OrgNodeError::StoreRefused(_)
        | OrgNodeError::DeviceSecretMismatch
        | OrgNodeError::DuplicatePersonaStore
        | OrgNodeError::ReplyForAnotherOrganisation
        | OrgNodeError::InviteNotOutstanding
        | OrgNodeError::OrganisationHeldByAnotherPersona { .. }
        | OrgNodeError::RecordMovedSinceCheck { .. } => ReceiverOutcome::ReceiveError {
            message: e.to_string(),
        },
    }
}

/// LLR-zw6jqy (REQ-jfxah3, REQ-kn5rtx, REQ-x3c8n2). The outcomes one FAILED
/// receive iteration produces, in emission order, decided by the variant of
/// org-io's `ReceiveError` and never by its text.
///
/// A refusal from org-node is classified by `classify_receive_error`. Every
/// other failure is not a verdict on any update and is one receiver error.
/// Only `ReceiveError::Terminal` (the endpoint closed, or a bind failed:
/// org-io's REQ-8e57wm) stops the loop: it is followed by a `Stopped` outcome
/// with the same message, because REQ-jfxah3 announces the stop "before that
/// loop exits" and emission order is what a consumer sees.
///
/// Classified by variant, not by matching the error's text: under a text
/// match a reworded message would turn a terminal failure into an endless
/// retry, and a transient failure whose text happened to contain a fragment
/// would stop the loop. No wildcard arm: a new `ReceiveError` variant is a
/// compile error here.
pub fn outcomes_for_receive_error(error: &ReceiveError) -> Vec<ReceiverOutcome> {
    match error {
        ReceiveError::Refused(refused) => vec![classify_receive_error(refused)],
        ReceiveError::Terminal(_) => {
            let message = error.to_string();
            vec![
                ReceiverOutcome::ReceiveError { message: message.clone() },
                // REQ-jfxah3: say that the loop is stopping, and why.
                ReceiverOutcome::Stopped { reason: message },
            ]
        }
        ReceiveError::Connection(_) | ReceiveError::Chain(_) | ReceiveError::Store(_) | ReceiveError::Keychain(_) => {
            vec![ReceiverOutcome::ReceiveError { message: error.to_string() }]
        }
    }
}

/// What org-io's `next_received` returned, as far as the announcement
/// depends on it: a committed message of a `ReceiveKind`, or a revoked
/// Device's acknowledgement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrival {
    Committed(ReceiveKind),
    Acknowledged,
}

/// LLR-bym4md. The outcome of one received message for `org_id`, given the
/// record as read back after it (`(epoch, root)`, or `None` when it could not
/// be read). `Removed` and `Revoked` are a self-delete, whatever the record
/// read; `Updated` and `Admitted` are `Updated` with the record read, or
/// `RecordUnreadable` without one (REQ-2k7ys4, REQ-dp95pv); an
/// acknowledgement is `AcknowledgementReceived`.
pub fn outcome_for_arrival(arrival: Arrival, org_id: String, record: Option<(u64, String)>) -> ReceiverOutcome {
    match (arrival, record) {
        (Arrival::Acknowledged, _) => ReceiverOutcome::AcknowledgementReceived { org_id },
        (Arrival::Committed(ReceiveKind::Removed | ReceiveKind::Revoked), _) => ReceiverOutcome::SelfDeleted { org_id },
        (Arrival::Committed(ReceiveKind::Updated | ReceiveKind::Admitted), Some((epoch, root))) => {
            ReceiverOutcome::Updated { org_id, epoch, root }
        }
        // REQ-2k7ys4 / REQ-dp95pv. No record, no epoch or root to announce: a
        // default such as `(0, String::new())` would announce a verified
        // update at genesis with an empty root — indistinguishable from a
        // real one.
        (Arrival::Committed(ReceiveKind::Updated | ReceiveKind::Admitted), None) => {
            ReceiverOutcome::RecordUnreadable { org_id }
        }
    }
}

/// One event to emit: a name and a payload.
#[derive(Debug, Clone, PartialEq)]
pub struct Emission {
    pub name: &'static str,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
struct MembershipUpdated<'a> { org_id: &'a str, epoch: u64, root: &'a str }
#[derive(Debug, Clone, Serialize)]
struct OrgOnly<'a> { org_id: &'a str }
#[derive(Debug, Clone, Serialize)]
struct EpochChanged<'a> { org_id: &'a str, epoch: u64 }
#[derive(Debug, Clone, Serialize)]
struct VerifyFailed<'a> { org_id: Option<&'a str>, message: &'a str }
#[derive(Debug, Clone, Serialize)]
struct MessageOnly<'a> { message: &'a str }
#[derive(Debug, Clone, Serialize)]
struct Stopped<'a> { reason: &'a str }

fn json<T: Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).expect("event payloads are plain structs of owned scalars")
}

/// The events one outcome produces, in emission order.
pub fn emissions_for(outcome: &ReceiverOutcome) -> Vec<Emission> {
    match outcome {
        // REQ-tw4cb5: NO epoch-changed event here. The record is gone; there is
        // no epoch to report; the previous code emitted a literal 0, which is
        // genesis and therefore a reachable, meaningful value (HAZ-5ha5vv).
        ReceiverOutcome::SelfDeleted { org_id } => vec![Emission {
            name: "revoked",
            payload: json(&OrgOnly { org_id }),
        }],

        // LLR-2vg79y (amended in S3): nothing. org-io keeps a received
        // acknowledgement from S3b-io; the UI has no use for it today.
        ReceiverOutcome::AcknowledgementReceived { .. } => Vec::new(),

        ReceiverOutcome::Updated { org_id, epoch, root } => vec![
            Emission {
                name: "membership-updated",
                payload: json(&MembershipUpdated { org_id, epoch: *epoch, root }),
            },
            Emission {
                name: "incoming-verified",
                payload: json(&MembershipUpdated { org_id, epoch: *epoch, root }),
            },
            Emission {
                name: "epoch-changed",
                payload: json(&EpochChanged { org_id, epoch: *epoch }),
            },
        ],

        // REQ-2k7ys4: no membership-updated, no incoming-verified, no
        // epoch-changed. REQ-dp95pv: an event that says so, naming the org.
        ReceiverOutcome::RecordUnreadable { org_id } => vec![Emission {
            name: "record-unreadable",
            payload: json(&OrgOnly { org_id }),
        }],

        ReceiverOutcome::VerifyFailed { org_id, message } => vec![Emission {
            name: "verification-failed",
            payload: json(&VerifyFailed { org_id: org_id.as_deref(), message }),
        }],

        // REQ-kn5rtx / REQ-wu6z9p: its own event name, outside the verification
        // vocabulary, carrying a message and nothing a verification row could
        // be built from.
        ReceiverOutcome::ReceiveError { message } => vec![Emission {
            name: "receiver-error",
            payload: json(&MessageOnly { message }),
        }],

        ReceiverOutcome::Stopped { reason } => vec![Emission {
            name: "receiver-stopped",
            payload: json(&Stopped { reason }),
        }],
    }
}

/// REQ-6hgm8r / REQ-3hfggn. Claims the right to run a receiver loop, and
/// releases it on drop — however the loop ends, including by panic.
///
/// The previous code set the flag before spawning and never cleared it, so an
/// exited loop kept the slot forever and no restart was possible (HAZ-cfp4jb).
pub struct StartGuard(Arc<AtomicBool>);

impl StartGuard {
    /// `Some` if no loop is running and this caller may start one; `None` if
    /// one is already running.
    pub fn try_claim(flag: &Arc<AtomicBool>) -> Option<StartGuard> {
        flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| StartGuard(Arc::clone(flag)))
    }
}

impl Drop for StartGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
