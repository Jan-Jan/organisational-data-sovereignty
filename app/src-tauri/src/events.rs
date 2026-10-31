//! What the receiver loop announces, and the guard that decides whether a
//! receiver loop may start, and what a verification the app ran announces.
//!
//! `emissions_for` is a total function from an outcome to the events that
//! outcome produces. It exists so the receiver's announcements can be tested
//! without a Tauri application, an OrgService, a chain or a peer — none of
//! which any gate in this repository can stand up.

use org_io::node::OrgNodeError;
use org_io::{ReceiveError, Received};

use crate::entry::{EntryResultDto, VerificationEntryDto};
use serde::Serialize;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// What one iteration of the receiver loop, or one verification it ran,
/// concluded.
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiverOutcome {
    /// LLR-ygr6gd: a verification found this node removed; its Persona's
    /// data is deleted. No epoch (REQ-tw4cb5).
    SelfDeleted { org_id: Option<String> },
    /// LLR-ygr6gd: a removed Device's acknowledgement was kept. Nothing is announced.
    AcknowledgementReceived { org_id: String },
    /// LLR-4bvdn7: a provisional organisation was received and kept. It is
    /// never an adoption, so it states no epoch and no root.
    ProvisionalReceived { org_id: String },
    /// LLR-dp45mm: an admission request was kept.
    AdmissionRequested(AdmissionRequested),
    /// LLR-t9rdf8: an organisation request was answered. Nothing is announced.
    OrganisationRequestAnswered { org_id: String, requester: String },
    /// LLR-ygr6gd: a revocation notice was kept; the next verification acts
    /// on it. Nothing is announced.
    RevocationNoticeKept { org_id: String },
    /// A verification adopted an epoch and its record was read back.
    Updated { org_id: String, epoch: u64, root: String },
    /// REQ-2k7ys4 / REQ-dp95pv: an epoch was adopted and the record could NOT
    /// be read back. This variant is what stops `(0, String::new())` existing.
    RecordUnreadable { org_id: Option<String> },
    /// One verification result: the entry `verify_organisation` returns for
    /// it (LLR-jf63e4), as its JSON value.
    VerificationResult { entry: serde_json::Value },
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

/// LLR-dp45mm: the `admission-requested` payload, field for field, in this
/// order: `org_id` (40 hex), `invite_id` (64), the `invitee_name` typed for
/// that Invite, the request's `name`, `surname` and `preferred_handles`,
/// and its `device_public_key` (64).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdmissionRequested {
    pub org_id: String,
    pub invite_id: String,
    pub invitee_name: String,
    pub name: String,
    pub surname: String,
    pub preferred_handles: Vec<String>,
    pub device_public_key: String,
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
///
/// The refusals of the proposal, verification, request and removal
/// operations are about this node's store or the caller, never a verdict on
/// a received update.
pub fn classify_receive_error(e: &OrgNodeError) -> ReceiverOutcome {
    match e {
        // Verdicts on an incoming update: each one means "this did not verify".
        OrgNodeError::OrgIdMismatch
        | OrgNodeError::StaleSeq { .. }
        | OrgNodeError::MalformedDelta
        | OrgNodeError::DeltaBaseMismatch
        | OrgNodeError::RootMismatch
        | OrgNodeError::StaleEpoch { .. }
        | OrgNodeError::SeqNotEpoch { .. } => ReceiverOutcome::VerifyFailed {
            // REQ-affyf5: the organisation is absent rather than invented.
            org_id: None,
            message: e.to_string(),
        },

        // Not a verdict: a chain read, a value refused, a refusal about this
        // node's own store or bookkeeping, or about what the caller supplied.
        OrgNodeError::Chain(_)
        | OrgNodeError::OrgNotOnChain
        | OrgNodeError::Trie(_)
        | OrgNodeError::InvalidOrgPublicKey
        | OrgNodeError::InvalidField { .. }
        | OrgNodeError::AdmissionNotExpected { .. }
        | OrgNodeError::AdmissionNotOurs { .. }
        | OrgNodeError::ProvisionalLimit { .. }
        | OrgNodeError::MalformedMessage
        | OrgNodeError::MessageTooLarge { .. }
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
        | OrgNodeError::InviteNotOutstanding { .. }
        | OrgNodeError::OrganisationHeldByAnotherPersona { .. }
        | OrgNodeError::NoProvisionalChange
        | OrgNodeError::OrganisationBehind { .. }
        | OrgNodeError::NoAdmissionRequestKept { .. }
        | OrgNodeError::AdmissionAlreadyRecorded { .. }
        | OrgNodeError::NoFreeHandle
        | OrgNodeError::HandleTaken { .. }
        | OrgNodeError::MemberNotHeld { .. }
        | OrgNodeError::RevocationAlreadyRecorded { .. }
        | OrgNodeError::NoProvisionalFounding
        | OrgNodeError::FoundingProxyAlreadyRecorded
        | OrgNodeError::FoundingNotWritten
        | OrgNodeError::NoProvisionalMember
        | OrgNodeError::OrganisationNotRequested { .. }
        | OrgNodeError::PersonaNotHeld { .. }
        | OrgNodeError::PersonaIdNotHeld { .. }
        | OrgNodeError::AdmissionKeyNotSender { .. }
        | OrgNodeError::OrganisationRemoved { .. }
        | OrgNodeError::AcknowledgementNotKept { .. }
        | OrgNodeError::DuplicateOrgPublicKey => ReceiverOutcome::ReceiveError {
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
        ReceiveError::Connection(_) | ReceiveError::Store(_) | ReceiveError::NoOrganisation { .. } => {
            vec![ReceiverOutcome::ReceiveError { message: error.to_string() }]
        }
    }
}

/// What org-io's `next_received` reported, as far as the announcement
/// depends on it: org-io's `Received`, ids and keys as hex, values as text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arrival {
    ProvisionalOrganisation { org_id: String },
    AdmissionRequestKept(AdmissionRequested),
    OrganisationRequestAnswered { org_id: String, requester: String },
    RevocationNoticeKept { org_id: String },
    Acknowledged { org_id: String },
}

impl From<Received> for Arrival {
    fn from(received: Received) -> Self {
        let hex_org = |org_id: &org_io::node::OrgId| hex::encode(org_id.as_bytes());
        match received {
            // LLR-4bvdn7: the epoch, root and sender are not carried; nothing
            // announces them before a verification adopts.
            Received::ProvisionalOrganisationReceived { org_id, .. } => {
                Arrival::ProvisionalOrganisation { org_id: hex_org(&org_id) }
            }
            Received::AdmissionRequestKept { org_id, invite_id, name, surname, preferred_handles, from, invitee_name } => {
                Arrival::AdmissionRequestKept(AdmissionRequested {
                    org_id: hex_org(&org_id),
                    invite_id: hex::encode(invite_id.as_bytes()),
                    invitee_name,
                    name: name.to_string(),
                    surname: surname.to_string(),
                    preferred_handles: preferred_handles.iter().map(ToString::to_string).collect(),
                    device_public_key: hex::encode(from.as_bytes()),
                })
            }
            Received::OrganisationRequestAnswered { org_id, requester, .. } => Arrival::OrganisationRequestAnswered {
                org_id: hex_org(&org_id),
                requester: hex::encode(requester.as_bytes()),
            },
            Received::RevocationNoticeKept { org_id, .. } => Arrival::RevocationNoticeKept { org_id: hex_org(&org_id) },
            Received::Acknowledged(verified) => Arrival::Acknowledged { org_id: hex_org(&verified.into_inner().org_id) },
        }
    }
}

/// LLR-4bvdn7, LLR-ygr6gd, LLR-dp45mm, LLR-t9rdf8. The outcome of one
/// arrival. No arrival is an adoption or a self-delete: those are
/// verification results (`outcomes_for_verification`).
pub fn outcome_for_arrival(arrival: Arrival) -> ReceiverOutcome {
    match arrival {
        Arrival::ProvisionalOrganisation { org_id } => ReceiverOutcome::ProvisionalReceived { org_id },
        Arrival::AdmissionRequestKept(request) => ReceiverOutcome::AdmissionRequested(request),
        Arrival::OrganisationRequestAnswered { org_id, requester } => {
            ReceiverOutcome::OrganisationRequestAnswered { org_id, requester }
        }
        Arrival::RevocationNoticeKept { org_id } => ReceiverOutcome::RevocationNoticeKept { org_id },
        Arrival::Acknowledged { org_id } => ReceiverOutcome::AcknowledgementReceived { org_id },
    }
}

/// LLR-w7m3d4: the one follow-up a received message asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FollowUp {
    /// Verify this Organisation, once, before the next message is read.
    Verify { org_id: String },
}

/// LLR-w7m3d4. "Verify this Organisation" for a received provisional
/// organisation or a kept revocation notice, nothing for any other outcome.
pub fn follow_up_for(outcome: &ReceiverOutcome) -> Option<FollowUp> {
    match outcome {
        ReceiverOutcome::ProvisionalReceived { org_id } | ReceiverOutcome::RevocationNoticeKept { org_id } => {
            Some(FollowUp::Verify { org_id: org_id.clone() })
        }
        ReceiverOutcome::SelfDeleted { .. }
        | ReceiverOutcome::AcknowledgementReceived { .. }
        | ReceiverOutcome::AdmissionRequested(_)
        | ReceiverOutcome::OrganisationRequestAnswered { .. }
        | ReceiverOutcome::Updated { .. }
        | ReceiverOutcome::RecordUnreadable { .. }
        | ReceiverOutcome::VerificationResult { .. }
        | ReceiverOutcome::VerifyFailed { .. }
        | ReceiverOutcome::ReceiveError { .. }
        | ReceiverOutcome::Stopped { .. } => None,
    }
}

/// LLR-w7m3d4. One iteration of the receiver loop: `next` once, every
/// outcome's events in order, and, right after an outcome that asks for
/// one, `verify` once for its Organisation with every outcome of that
/// verification announced, all before this returns and the loop asks
/// `next` again. Returns whether the loop stops (REQ-jfxah3).
pub async fn receive_iteration<Next, NextFuture, Verify, VerifyFuture, Emit>(
    next: Next,
    mut verify: Verify,
    mut emit: Emit,
) -> bool
where
    Next: FnOnce() -> NextFuture,
    NextFuture: Future<Output = Vec<ReceiverOutcome>>,
    Verify: FnMut(String) -> VerifyFuture,
    VerifyFuture: Future<Output = Vec<ReceiverOutcome>>,
    Emit: FnMut(Emission),
{
    let outcomes = next().await;
    let stopping = outcomes.iter().any(|outcome| matches!(outcome, ReceiverOutcome::Stopped { .. }));
    for outcome in &outcomes {
        for emission in emissions_for(outcome) {
            emit(emission);
        }
        if let Some(FollowUp::Verify { org_id }) = follow_up_for(outcome) {
            for verified in verify(org_id).await {
                for emission in emissions_for(&verified) {
                    emit(emission);
                }
            }
        }
    }
    stopping
}

/// LLR-ygr6gd, LLR-acb5g4. The outcomes of one verification result:
/// `entry` is the entry `verify_organisation` returns for it, and `record`
/// the record read back after it (epoch and root), or `None`. `adopted` is
/// `Updated` with the record read, or `RecordUnreadable` without one, then
/// the result; `removed` is a self-delete with no epoch, whatever the
/// record reads, then the result; every other result is the result alone,
/// never `Updated`. The entry is read by its type, so a result the entry
/// gains is a compile error here, not a silent default.
pub fn outcomes_for_verification(entry: &VerificationEntryDto, record: Option<(u64, String)>) -> Vec<ReceiverOutcome> {
    // LLR-acb5g4, LLR-ygr6gd: an entry with no identifier (a founding
    // whose proxy is not recorded) names none: null, never "".
    let org_id = entry.org_id.clone();
    let result = ReceiverOutcome::VerificationResult { entry: json(entry) };
    match (&entry.result, record, org_id) {
        (EntryResultDto::Adopted { .. }, Some((epoch, root)), Some(org_id)) => {
            vec![ReceiverOutcome::Updated { org_id, epoch, root }, result]
        }
        // REQ-2k7ys4 / REQ-dp95pv: no record, no epoch or root to announce.
        // An entry naming no Organisation has no record to read back either.
        (EntryResultDto::Adopted { .. }, _, org_id) => vec![ReceiverOutcome::RecordUnreadable { org_id }, result],
        (EntryResultDto::Removed, _, org_id) => vec![ReceiverOutcome::SelfDeleted { org_id }, result],
        (
            EntryResultDto::InStep { .. }
            | EntryResultDto::Behind { .. }
            | EntryResultDto::Inconsistent { .. }
            | EntryResultDto::Unverified { .. },
            _,
            _,
        ) => vec![result],
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
struct OrgOnly<'a> { org_id: Option<&'a str> }
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
            payload: json(&OrgOnly { org_id: org_id.as_deref() }),
        }],

        // LLR-2vg79y: nothing. org-io keeps a received acknowledgement; the
        // UI has no use for it.
        ReceiverOutcome::AcknowledgementReceived { .. } => Vec::new(),

        // LLR-4bvdn7: nothing of its own. The verification that follows at
        // once (LLR-w7m3d4) announces what the chain says.
        ReceiverOutcome::ProvisionalReceived { .. } => Vec::new(),

        // LLR-dp45mm: exactly one event, carrying the request's fields.
        ReceiverOutcome::AdmissionRequested(request) => vec![Emission {
            name: "admission-requested",
            payload: json(request),
        }],

        // LLR-t9rdf8 / LLR-ygr6gd: announced by no event.
        ReceiverOutcome::OrganisationRequestAnswered { .. } | ReceiverOutcome::RevocationNoticeKept { .. } => Vec::new(),

        // LLR-8grnbb: exactly one event; its payload is the entry, with its
        // `discarded`, `sends` and `requests`, whichever verification ran it.
        ReceiverOutcome::VerificationResult { entry } => vec![Emission {
            name: "verification-result",
            payload: entry.clone(),
        }],

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
            payload: json(&OrgOnly { org_id: org_id.as_deref() }),
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
