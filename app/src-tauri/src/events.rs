//! What the receiver loop announces, and the guard that decides whether a
//! receiver loop may start.
//!
//! `emissions_for` is a total function from an outcome to the events that
//! outcome produces. It exists so the receiver's announcements can be tested
//! without a Tauri application, an OrgService, a chain or a peer — none of
//! which any gate in this repository can stand up.

use org_node::OrgNodeError;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// What one iteration of the receiver loop concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiverOutcome {
    /// This persona was revoked and deleted itself.
    SelfDeleted { org_id: String },
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
/// - `receive_and_self_delete_if_revoked` in `org-node/src/service.rs` returns
///   `OrgNotOnChain` when the local store holds no record of the organisation;
/// - the same function returns `Trie(_)` when the local members snapshot
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
        OrgNodeError::OrgIdMismatch
        | OrgNodeError::BadSignature
        | OrgNodeError::StaleSeq { .. }
        | OrgNodeError::MalformedDelta
        | OrgNodeError::DeltaBaseMismatch
        | OrgNodeError::RootMismatch
        | OrgNodeError::StaleEpoch { .. } => ReceiverOutcome::VerifyFailed {
            // REQ-affyf5: the organisation is absent rather than invented. The
            // service reports the failure without naming one.
            org_id: None,
            message: e.to_string(),
        },

        // Not a verdict: the chain could not be read, the transport failed, or
        // the local store could not supply what the verification needed.
        OrgNodeError::Chain(_) | OrgNodeError::OrgNotOnChain | OrgNodeError::Trie(_) => {
            ReceiverOutcome::ReceiveError {
                message: e.to_string(),
            }
        }
    }
}

/// The message fragments that mean the loop cannot continue and must exit.
///
/// Each one is a fragment org-node or the transport actually formats:
///
/// - `endpoint bind:` — `ensure_endpoint` in `org-node/src/service.rs`, the
///   bind itself failed;
/// - `endpoint bind failed unexpectedly` — `ensure_endpoint` again, the
///   endpoint is still absent after a bind that reported success;
/// - `endpoint closed` — `recv_one` in `org-node/src/transport/endpoint.rs`,
///   `accept()` returned `None`; it reaches the app as
///   `chain read failed: iroh recv: iroh accept error: endpoint closed`.
///
/// The previous value was the single string `"endpoint not bound"`, which
/// appears nowhere in org-node or iroh, so the match never fired: the loop never
/// broke, `receiver-stopped` was never emitted, and the guard was never released
/// by task exit.
///
/// This is still a SUBSTRING MATCH on messages org-node formats, and that is
/// still a real weakness: a reworded message would once again turn a terminal
/// failure into an infinite retry. REQ-x3c8n2 is the expectation on `org-node`
/// that would make this a typed condition instead, and it stands — correcting
/// the fragments makes a dead mechanism live, it does not make it sound. What
/// this change buys is that the consequence is now visible (`receiver-stopped`
/// carries the reason, REQ-jfxah3) and recoverable (the guard is released,
/// REQ-3hfggn) at all.
const TERMINAL_ERRORS: [&str; 3] = [
    "endpoint bind:",
    "endpoint bind failed unexpectedly",
    "endpoint closed",
];

/// REQ-jfxah3. Whether a rendered receive-path error message means the loop
/// cannot continue.
pub fn is_terminal_error(message: &str) -> bool {
    TERMINAL_ERRORS.iter().any(|marker| message.contains(marker))
}

/// REQ-jfxah3 / REQ-kn5rtx. The outcomes one FAILED receive iteration produces,
/// in emission order.
///
/// A `Vec` rather than a single outcome because a terminal error produces two:
/// the failure itself (REQ-affyf5 or REQ-kn5rtx) and then the loop's exit
/// (REQ-jfxah3), in that order — the requirement says the stop is announced
/// "before that loop exits", and emission order is what a consumer sees. Every
/// other failed iteration produces exactly one.
///
/// Terminality is orthogonal to the class: a terminal endpoint failure is a
/// `ReceiveError` that ALSO stops the loop, so the `Stopped` push applies to
/// whichever class `classify_receive_error` returned.
pub fn outcomes_for_receive_error(e: &OrgNodeError) -> Vec<ReceiverOutcome> {
    let message = e.to_string();
    let mut out = vec![classify_receive_error(e)];
    if is_terminal_error(&message) {
        // REQ-jfxah3: say that the loop is stopping, and why.
        out.push(ReceiverOutcome::Stopped { reason: message });
    }
    out
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
