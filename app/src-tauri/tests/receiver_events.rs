//! What one iteration of the receiver loop announces.
//!
//! Requirements carried here: REQ-2k7ys4 (an update that could not be read back
//! announces no membership state), REQ-dp95pv (it announces that fact instead),
//! REQ-tw4cb5 (a self-delete announces no epoch), REQ-affyf5 (a verification
//! failure is announced, with the organisation when one is known), REQ-jfxah3
//! (the loop announces that it is stopping, and why).
//!
//! The negative assertions are the substance of this file, and they are written
//! against the event NAMES: "no membership event" has to mean no emission named
//! `membership-updated`, `incoming-verified` or `epoch-changed`.
//!
//! A name-level assertion is not enough on its own, though: an emission named
//! `revoked` carrying `{"org_id": …, "epoch": 0}` passes every name check in
//! this file and violates REQ-tw4cb5 outright. So REQ-tw4cb5 is also asserted
//! at the PAYLOAD level, recursively, across differing organisation ids, with a
//! contrast case against `Updated` — which legitimately does carry an epoch —
//! so the absence is shown to be discriminating rather than incidental.

use ods_poc_lib::entry::{EntryResultDto, VerificationEntryDto, WriteLabelDto};
use ods_poc_lib::events::{self, Emission, ReceiverOutcome};

const MEMBERSHIP_EVENTS: [&str; 3] = ["membership-updated", "incoming-verified", "epoch-changed"];

fn names(out: &[Emission]) -> Vec<&str> {
    out.iter().map(|e| e.name).collect()
}

fn org() -> String {
    "aa".repeat(20)
}

/// True if `key` appears as an object key anywhere in `v`, at any depth —
/// including inside arrays and nested objects. A shallow `payload.get("epoch")`
/// would miss an epoch tucked inside a nested object, which is exactly the
/// shape a well-meaning refactor reaches for.
fn has_key_anywhere(v: &serde_json::Value, key: &str) -> bool {
    match v {
        serde_json::Value::Object(map) => {
            map.contains_key(key) || map.values().any(|inner| has_key_anywhere(inner, key))
        }
        serde_json::Value::Array(items) => items.iter().any(|item| has_key_anywhere(item, key)),
        _ => false,
    }
}

/// A verification entry for the Organisation `org()`, with `result`.
fn entry_of(result: EntryResultDto) -> VerificationEntryDto {
    VerificationEntryDto { org_id: Some(org()), founder: None, result, write: None }
}

/// `entry`, as the `verification-result` payload carries it.
fn wire(entry: &VerificationEntryDto) -> serde_json::Value {
    serde_json::to_value(entry).expect("serialises")
}

/// An `adopted` result at epoch 4, `own` as given, nothing discarded.
fn adopted_at_4(own: bool) -> EntryResultDto {
    EntryResultDto::Adopted { epoch: 4, own, discarded: Vec::new(), sends: None, sends_refused: None }
}

/// Every emission the self-delete path produces for `org_id`, with the offending
/// payload returned when any of them carries an `epoch` key at any depth.
fn self_delete_epoch_offender(org_id: &str) -> Option<serde_json::Value> {
    events::emissions_for(&ReceiverOutcome::SelfDeleted {
        org_id: Some(org_id.to_string()),
    })
    .into_iter()
    .find(|e| has_key_anywhere(&e.payload, "epoch"))
    .map(|e| e.payload)
}

// ---------------------------------------------------------------------------
// REQ-2k7ys4 — an update announces membership state only when it has some
// ---------------------------------------------------------------------------

// verifies: LLR-p2nm5a, PR-bu6mau
// PR-bu6mau: pins today's behaviour. `membership-updated` and
// `incoming-verified` both carry the same verified update, and the frontend
// files each as a verification row.
#[test]
fn updated_emits_membership_incoming_and_epoch() {
    let out = events::emissions_for(&ReceiverOutcome::Updated {
        org_id: org(),
        epoch: 7,
        root: "ff".repeat(32),
    });
    assert_eq!(
        names(&out),
        vec!["membership-updated", "incoming-verified", "epoch-changed"]
    );
}

// verifies: LLR-p2nm5a
#[test]
fn updated_payload_carries_the_measured_epoch_and_root() {
    let root = "ff".repeat(32);
    let out = events::emissions_for(&ReceiverOutcome::Updated {
        org_id: org(),
        epoch: 7,
        root: root.clone(),
    });
    for e in &out {
        assert_eq!(e.payload["org_id"], serde_json::json!(org()), "{}", e.name);
        assert_eq!(e.payload["epoch"], serde_json::json!(7), "{}", e.name);
    }
    let membership = &out[0];
    assert_eq!(membership.payload["root"], serde_json::json!(root));
}

// verifies: LLR-p2nm5a
#[test]
fn updated_carries_boundary_epochs_and_roots_verbatim() {
    // Abnormal inputs: genesis (0, a reachable and meaningful epoch), the
    // largest epoch, and an empty root. Each is carried exactly as measured,
    // in all three events, rather than dropped, clamped or replaced.
    for (epoch, root) in [(0u64, String::new()), (u64::MAX, "00".repeat(32))] {
        let out = events::emissions_for(&ReceiverOutcome::Updated {
            org_id: org(),
            epoch,
            root: root.clone(),
        });
        assert_eq!(names(&out), MEMBERSHIP_EVENTS, "epoch {epoch}");
        for e in &out {
            assert_eq!(e.payload["epoch"], serde_json::json!(epoch), "{} at {epoch}", e.name);
        }
        assert_eq!(out[0].payload["root"], serde_json::json!(root), "epoch {epoch}");
        assert_eq!(out[1].payload["root"], serde_json::json!(root), "epoch {epoch}");
    }
}

// verifies: LLR-hgsdm8
#[test]
fn record_unreadable_emits_no_membership_event() {
    let out = events::emissions_for(&ReceiverOutcome::RecordUnreadable { org_id: Some(org()) });
    let names = names(&out);
    for forbidden in MEMBERSHIP_EVENTS {
        assert!(!names.contains(&forbidden), "emitted {forbidden}: {names:?}");
    }
}

// verifies: LLR-hgsdm8
#[test]
fn record_unreadable_emits_no_epoch_event() {
    // The previous code substituted `(0, String::new())` for the unreadable
    // record, so the frontend was told the organisation was at genesis with an
    // empty root — a reachable, meaningful value that was simply untrue
    // (HAZ-5ha5vv). No epoch event may be emitted at all.
    let out = events::emissions_for(&ReceiverOutcome::RecordUnreadable { org_id: Some(org()) });
    assert!(!names(&out).contains(&"epoch-changed"), "{:?}", names(&out));
    for e in &out {
        assert!(
            e.payload.get("epoch").is_none(),
            "no payload may carry an epoch: {:?}",
            e.payload
        );
        assert!(
            e.payload.get("root").is_none(),
            "no payload may carry a root: {:?}",
            e.payload
        );
    }
}

// ---------------------------------------------------------------------------
// REQ-dp95pv — and it says so, naming the organisation
// ---------------------------------------------------------------------------

// verifies: LLR-hgsdm8
#[test]
fn record_unreadable_names_the_organisation() {
    let out = events::emissions_for(&ReceiverOutcome::RecordUnreadable { org_id: Some(org()) });
    assert_eq!(names(&out), vec!["record-unreadable"]);
    assert_eq!(out[0].payload["org_id"], serde_json::json!(org()));
}

// verifies: LLR-hgsdm8
#[test]
fn record_unreadable_emits_exactly_one_event() {
    let out = events::emissions_for(&ReceiverOutcome::RecordUnreadable { org_id: Some(org()) });
    assert_eq!(out.len(), 1, "{:?}", names(&out));
}

// verifies: LLR-hgsdm8
#[test]
fn record_unreadable_holds_for_any_organisation_id() {
    // Abnormal inputs: an all-zero id, an all-ff id, an empty string, a
    // non-hex string and an over-long one. Each produces exactly one
    // `record-unreadable` naming that id verbatim, with no epoch and no root.
    let all_zero = "00".repeat(20);
    let all_ff = "ff".repeat(20);
    let over_long = "ab".repeat(64);
    for org_id in [
        all_zero.as_str(),
        all_ff.as_str(),
        "",
        "not-an-org-id",
        over_long.as_str(),
    ] {
        let out = events::emissions_for(&ReceiverOutcome::RecordUnreadable {
            org_id: Some(org_id.to_string()),
        });
        assert_eq!(names(&out), vec!["record-unreadable"], "{org_id:?}");
        assert_eq!(out[0].payload, serde_json::json!({ "org_id": org_id }), "{org_id:?}");
    }
}

// ---------------------------------------------------------------------------
// REQ-tw4cb5 — a self-delete has no epoch to report
// ---------------------------------------------------------------------------

// verifies: LLR-2vg79y
#[test]
fn self_delete_emits_no_epoch_event() {
    let out = events::emissions_for(&ReceiverOutcome::SelfDeleted { org_id: Some(org()) });
    let names = names(&out);
    for forbidden in MEMBERSHIP_EVENTS {
        assert!(!names.contains(&forbidden), "emitted {forbidden}: {names:?}");
    }
    for e in &out {
        assert!(
            e.payload.get("epoch").is_none(),
            "the record is gone; there is no epoch: {:?}",
            e.payload
        );
    }
}

/// verifies: LLR-2vg79y
///
/// A received acknowledgement announces nothing to the UI (org-io keeps it).
#[test]
fn a_received_acknowledgement_emits_nothing() {
    assert!(events::emissions_for(&ReceiverOutcome::AcknowledgementReceived { org_id: org() }).is_empty());
}

// verifies: LLR-2vg79y
#[test]
fn self_delete_emits_revoked_naming_the_organisation() {
    let out = events::emissions_for(&ReceiverOutcome::SelfDeleted { org_id: Some(org()) });
    assert_eq!(names(&out), vec!["revoked"]);
    assert_eq!(out[0].payload["org_id"], serde_json::json!(org()));
}

// verifies: LLR-2vg79y
#[test]
fn self_delete_payload_carries_no_epoch_key_at_any_depth() {
    // The two tests above assert on event NAMES only. Renaming `epoch-changed`
    // away while keeping the field — emitting `revoked` with an `epoch` inside
    // its payload — satisfies both of them and violates REQ-tw4cb5. This asserts
    // on the payload object's KEYS instead, recursively, so a nested
    // `{"state": {"epoch": 0}}` is caught as well as a top-level one.
    let out = events::emissions_for(&ReceiverOutcome::SelfDeleted { org_id: Some(org()) });
    assert!(!out.is_empty(), "the self-delete path emits something");
    for e in &out {
        let keys: Vec<&str> = e
            .payload
            .as_object()
            .expect("every payload is a JSON object")
            .keys()
            .map(String::as_str)
            .collect();
        assert!(
            !keys.contains(&"epoch"),
            "`{}` payload carries an epoch key: {:?}",
            e.name,
            keys
        );
        assert!(
            !has_key_anywhere(&e.payload, "epoch"),
            "`{}` payload carries a nested epoch key: {}",
            e.name,
            e.payload
        );
    }
}

// verifies: LLR-2vg79y
#[test]
fn self_delete_carries_no_epoch_for_any_organisation_id() {
    // Abnormal inputs. The requirement is a property of the self-delete path,
    // not of one fixture organisation, so drive it with materially different
    // ids: an all-zero one (the boundary value that reads as "unset" and is the
    // shape a hard-coded default takes), an all-ff maximal one, an empty string,
    // and one that is not hex at all. The absence must hold for every one.
    let all_zero = "00".repeat(20);
    let all_ff = "ff".repeat(20);
    let over_long = "ab".repeat(64);
    for org_id in [
        all_zero.as_str(),
        all_ff.as_str(),
        "",
        "not-an-org-id",
        over_long.as_str(),
    ] {
        let out = events::emissions_for(&ReceiverOutcome::SelfDeleted {
            org_id: Some(org_id.to_string()),
        });
        assert!(
            !out.is_empty(),
            "org_id {org_id:?} still produces an announcement"
        );
        assert_eq!(
            self_delete_epoch_offender(org_id),
            None,
            "org_id {org_id:?}: an emission carried an epoch"
        );
        // and the organisation is still named, so the absence is not the
        // trivial absence of any payload at all.
        assert_eq!(out[0].payload["org_id"], serde_json::json!(org_id));
    }
}

// verifies: LLR-2vg79y
#[test]
fn the_epoch_probe_finds_an_epoch_on_updated_and_none_on_self_delete() {
    // Contrast. `Updated` legitimately carries an epoch, so if the probe used by
    // the two tests above cannot find one THERE, its silence on the self-delete
    // path proves nothing. Same probe, same organisation, two outcomes.
    let updated = events::emissions_for(&ReceiverOutcome::Updated {
        org_id: org(),
        epoch: 7,
        root: "ff".repeat(32),
    });
    assert!(
        updated.iter().all(|e| has_key_anywhere(&e.payload, "epoch")),
        "the probe must find the epoch Updated does carry: {:?}",
        updated
    );
    // …and its "anywhere" is not a claim the emissions above can exercise: no
    // real payload nests. Shown here instead, so the recursion the self-delete
    // assertions rely on is not dead code.
    assert!(has_key_anywhere(
        &serde_json::json!({"state": [{"epoch": 0}]}),
        "epoch"
    ));
    assert!(!has_key_anywhere(
        &serde_json::json!({"state": [{"org_id": "aa"}]}),
        "epoch"
    ));
    assert_eq!(
        self_delete_epoch_offender(&org()),
        None,
        "and find none on the self-delete path"
    );
}

// ---------------------------------------------------------------------------
// REQ-affyf5 — a verification failure is announced as a failure
// ---------------------------------------------------------------------------

// verifies: LLR-p38be7
#[test]
fn verify_failure_carries_the_organisation_when_known() {
    let out = events::emissions_for(&ReceiverOutcome::VerifyFailed {
        org_id: Some(org()),
        message: "root mismatch".to_string(),
    });
    assert_eq!(names(&out), vec!["verification-failed"]);
    assert_eq!(out[0].payload["org_id"], serde_json::json!(org()));
}

// verifies: LLR-p38be7
#[test]
fn verify_failure_carries_null_organisation_when_unknown() {
    // Abnormal input: the failure happened before an organisation was
    // identified. The field must be present and null — not absent, and not an
    // empty string standing in for one.
    let out = events::emissions_for(&ReceiverOutcome::VerifyFailed {
        org_id: None,
        message: "malformed update".to_string(),
    });
    assert_eq!(out[0].payload["org_id"], serde_json::Value::Null);
    assert!(
        out[0].payload.get("org_id").is_some(),
        "the field is present and null: {:?}",
        out[0].payload
    );
}

// verifies: LLR-p38be7
#[test]
fn verify_failure_carries_the_message() {
    let out = events::emissions_for(&ReceiverOutcome::VerifyFailed {
        org_id: None,
        message: "recomputed root does not match the on-chain root".to_string(),
    });
    assert_eq!(
        out[0].payload["message"],
        serde_json::json!("recomputed root does not match the on-chain root")
    );
}

// verifies: LLR-p38be7
#[test]
fn verify_failure_emits_no_membership_event() {
    // Nothing verified, so nothing may be announced as verified.
    let out = events::emissions_for(&ReceiverOutcome::VerifyFailed {
        org_id: Some(org()),
        message: "root mismatch".to_string(),
    });
    let names = names(&out);
    for forbidden in MEMBERSHIP_EVENTS {
        assert!(!names.contains(&forbidden), "emitted {forbidden}: {names:?}");
    }
}

// ---------------------------------------------------------------------------
// REQ-jfxah3 — the loop announces that it is stopping
// ---------------------------------------------------------------------------

// verifies: LLR-2zmhvs
#[test]
fn stopped_names_the_reason() {
    let out = events::emissions_for(&ReceiverOutcome::Stopped {
        reason: "transport closed".to_string(),
    });
    assert_eq!(names(&out), vec!["receiver-stopped"]);
    assert_eq!(out[0].payload["reason"], serde_json::json!("transport closed"));
}

// verifies: LLR-2zmhvs
#[test]
fn stopped_emits_exactly_one_event() {
    // Abnormal input: an empty reason is still a stop, and still exactly one
    // announcement — a silent exit is the defect REQ-jfxah3 is about.
    let out = events::emissions_for(&ReceiverOutcome::Stopped {
        reason: String::new(),
    });
    assert_eq!(out.len(), 1, "{:?}", names(&out));
    assert_eq!(out[0].name, "receiver-stopped");
}

// ---------------------------------------------------------------------------
// REQ-kn5rtx — a receive-path failure is reported as a verification failure
// only when the underlying error IS a verification verdict; every other
// failure is reported as a receiver error (RC-3rddh7)
// ---------------------------------------------------------------------------
//
// A receive-path `Err` routed into `VerifyFailed` is rendered by the frontend
// as a "✗ MISMATCH" row under "Verified Updates (chain root match)". A
// transport hiccup or a chain read failure routed there would be displayed to
// the operator as a root mismatch — an instance of HAZ-9fmhm4, the
// verification display misrepresenting the verification state.
//
// The classification is asserted VARIANT BY VARIANT, against concrete event
// names. It deliberately does not re-derive the partition from a predicate: a
// test that recomputed the match would agree with any match.

use org_io::node::{OrgMembersError, OrgNodeError};

/// The `OrgNodeError` variants that ARE verdicts of the verification the
/// receiver performs on an incoming update. Each one means "this update does
/// not verify", each is produced only by `verify_envelope_against_chain`, and
/// each is a legitimate ✗ row.
///
/// `OrgNotOnChain` and `Trie(_)` are absent on purpose: both are ALSO reachable
/// from a purely local condition on the receive path, so neither can be claimed
/// as a verdict. They are asserted as receiver errors below.
fn verification_verdicts() -> Vec<OrgNodeError> {
    vec![
        OrgNodeError::OrgIdMismatch,
        OrgNodeError::StaleSeq { got: 3, last_seen: 7 },
        OrgNodeError::MalformedDelta,
        OrgNodeError::DeltaBaseMismatch,
        OrgNodeError::RootMismatch,
        OrgNodeError::StaleEpoch { got: 2, last: 9 },
        // A Sequence number that is not the chain's epoch
        // (an org-node commit rule) refuses the received update itself.
        OrgNodeError::SeqNotEpoch { seq: 9, epoch: 3 },
    ]
}

// verifies: LLR-wgr46u
#[test]
fn every_verification_verdict_is_classified_as_a_verification_failure() {
    for e in verification_verdicts() {
        let outcome = events::classify_receive_error(&e);
        assert!(
            matches!(outcome, ReceiverOutcome::VerifyFailed { .. }),
            "{e:?} is a verification verdict and must classify as VerifyFailed, got {outcome:?}"
        );
        assert_eq!(
            names(&events::emissions_for(&outcome)),
            vec!["verification-failed"],
            "{e:?} must be announced as a verification failure"
        );
    }
}

// verifies: LLR-wgr46u
#[test]
fn a_verification_verdict_carries_its_own_message_and_no_invented_organisation() {
    for e in verification_verdicts() {
        let outcome = events::classify_receive_error(&e);
        let out = events::emissions_for(&outcome);
        assert_eq!(out.len(), 1, "{e:?}: {:?}", names(&out));
        assert_eq!(
            out[0].payload["message"],
            serde_json::json!(e.to_string()),
            "{e:?}: the verdict's own message must reach the operator"
        );
        // REQ-affyf5: the service reports the failure without naming an
        // organisation, and a placeholder would be a worse answer than none.
        assert_eq!(out[0].payload["org_id"], serde_json::Value::Null, "{e:?}");
    }
}

// verifies: LLR-wgr46u
#[test]
fn a_chain_failure_is_classified_as_a_receiver_error() {
    // `Chain` is a chain read or transport failure. It is not a verdict on any
    // update: nothing was verified and nothing failed to verify.
    let e = OrgNodeError::Chain("websocket closed".to_string());
    let outcome = events::classify_receive_error(&e);
    assert_eq!(
        outcome,
        ReceiverOutcome::ReceiveError {
            message: e.to_string()
        },
        "a chain failure is a receiver error, not a verification verdict"
    );
    let out = events::emissions_for(&outcome);
    assert_eq!(names(&out), vec!["receiver-error"]);
    assert_eq!(out[0].payload["message"], serde_json::json!(e.to_string()));
}

// verifies: LLR-wgr46u, LLR-usxk57
#[test]
fn a_chain_failure_emits_no_verification_event_for_any_message() {
    // Abnormal inputs: an empty message, one that reads like a closed
    // endpoint, and an ordinary transient one. None of them is a verdict, so
    // none of them may be announced as one. Terminality is org-io's typed
    // condition, not this text.
    for msg in [
        "",
        "accept error: endpoint closed",
        "connection reset by peer",
    ] {
        let outcome = events::classify_receive_error(&OrgNodeError::Chain(msg.to_string()));
        let out = events::emissions_for(&outcome);
        let names = names(&out);
        assert!(
            !names.contains(&"verification-failed"),
            "Chain({msg:?}) must not be announced as a verification failure: {names:?}"
        );
        for forbidden in MEMBERSHIP_EVENTS {
            assert!(
                !names.contains(&forbidden),
                "Chain({msg:?}) emitted {forbidden}: {names:?}"
            );
        }
        assert_eq!(names, vec!["receiver-error"], "Chain({msg:?})");
    }
}

// verifies: LLR-usxk57
#[test]
fn an_invalid_organisation_public_key_is_classified_as_a_receiver_error() {
    // org-node refuses an Organisation state whose key is not a valid X25519
    // key when it reads the chain, before verifying anything against it:
    // nothing failed to verify, so no verdict may be claimed.
    let e = OrgNodeError::InvalidOrgPublicKey;
    let outcome = events::classify_receive_error(&e);
    assert_eq!(
        outcome,
        ReceiverOutcome::ReceiveError {
            message: e.to_string()
        }
    );
    let out = events::emissions_for(&outcome);
    assert_eq!(names(&out), vec!["receiver-error"]);
    assert!(
        !names(&out).contains(&"verification-failed"),
        "an unreadable Organisation state is not a verification failure"
    );
}

// verifies: REQ-kn5rtx
#[test]
fn the_receiver_error_payload_carries_a_message_and_no_verification_state() {
    // A receiver error has no organisation, no epoch, no root and no verdict.
    // A payload carrying any of those would let the frontend render it as a
    // verification row after all (HAZ-9fmhm4).
    let out = events::emissions_for(&ReceiverOutcome::ReceiveError {
        message: "chain read failed: rpc timeout".to_string(),
    });
    assert_eq!(out.len(), 1, "{:?}", names(&out));
    assert_eq!(out[0].name, "receiver-error");
    assert_eq!(
        out[0].payload["message"],
        serde_json::json!("chain read failed: rpc timeout")
    );
    for forbidden in ["epoch", "root", "verified", "org_id"] {
        assert!(
            !has_key_anywhere(&out[0].payload, forbidden),
            "a receiver error carries no `{forbidden}`: {}",
            out[0].payload
        );
    }
}

// verifies: LLR-wgr46u
#[test]
fn the_two_classes_are_actually_distinguished() {
    // Contrast. If every error classified the same way, the assertions above
    // would still hold for one of the two classes and would prove nothing about
    // the partition. Same classifier, one member of each class, two different
    // event names.
    let verdict = events::classify_receive_error(&OrgNodeError::RootMismatch);
    let transport = events::classify_receive_error(&OrgNodeError::Chain("io error".to_string()));
    assert_eq!(
        names(&events::emissions_for(&verdict)),
        vec!["verification-failed"]
    );
    assert_eq!(
        names(&events::emissions_for(&transport)),
        vec!["receiver-error"]
    );
    assert_ne!(
        events::emissions_for(&verdict)[0].name,
        events::emissions_for(&transport)[0].name
    );
}

// ---------------------------------------------------------------------------
// REQ-jfxah3 — the terminal messages that actually occur
// ---------------------------------------------------------------------------
//
// The loop stops on org-io's typed `ReceiveError::Terminal` (LLR-zw6jqy,
// which supersedes LLR-a9rjtf), so the cases below are org-io's two terminal
// causes, and the message each carries is whatever org-io renders: nothing
// is keyed to its text. The text-independence itself is
// `terminal_is_decided_by_the_error_variant_not_its_text`'s.

/// Every terminal case: the error as the app receives it, and the message the
/// operator is shown.
fn terminal_cases() -> Vec<(ReceiveError, String)> {
    [
        // org-io's `start_receiving`: an endpoint could not be bound.
        ReceiveError::Terminal(TerminalCause::BindFailed(TransportError::Bind("address already in use".to_string()))),
        // org-io's `next_received`: every accept task has ended.
        ReceiveError::Terminal(TerminalCause::EndpointClosed),
    ]
    .into_iter()
    .map(|error| {
        let rendered = error.to_string();
        (error, rendered)
    })
    .collect()
}

// inherited from LLR-a9rjtf (superseded by LLR-zw6jqy)
// verifies: LLR-zw6jqy
#[test]
fn every_real_terminal_message_stops_the_loop() {
    for (e, rendered) in terminal_cases() {
        let outcomes = events::outcomes_for_receive_error(&e);
        assert!(
            outcomes.contains(&ReceiverOutcome::Stopped {
                reason: rendered.to_string()
            }),
            "{rendered:?} is terminal and must stop the loop, got {outcomes:?}"
        );
    }
}

// inherited from LLR-a9rjtf (superseded by LLR-zw6jqy)
// verifies: LLR-zw6jqy
#[test]
fn a_non_terminal_failure_does_not_stop_the_loop() {
    // Abnormal inputs: a transport hiccup, a chain timeout, a local startup
    // condition and an empty message. None of them is terminal — stopping on one
    // would end the receiver for a failure it could have retried past. Each
    // arrives as org-io's `ReceiveError::Connection`.
    for inner in [
        "stream error: connection reset by peer",
        "rpc timeout",
        "no persona found — create one first",
        "",
    ] {
        let failure = ReceiveError::Connection(TransportError::Stream(inner.to_string()));
        let outcomes = events::outcomes_for_receive_error(&failure);
        assert_eq!(
            outcomes,
            vec![ReceiverOutcome::ReceiveError {
                message: failure.to_string()
            }],
            "Connection({inner:?}) is not terminal and must not stop the loop"
        );
    }
    // …and neither is a verdict on an update: a root mismatch is a reason to
    // reject that envelope, not a reason to stop receiving.
    let verdict = OrgNodeError::RootMismatch;
    assert_eq!(
        events::outcomes_for_receive_error(&ReceiveError::Refused(OrgNodeError::RootMismatch)),
        vec![ReceiverOutcome::VerifyFailed {
            org_id: None,
            message: verdict.to_string()
        }]
    );
}

// inherited from LLR-a9rjtf (superseded by LLR-zw6jqy)
// verifies: LLR-zw6jqy
#[test]
fn the_stop_is_announced_after_the_failure_for_the_same_error() {
    // REQ-jfxah3 says the stop is announced "before that loop exits", and what a
    // consumer sees is the emission ORDER: the failure first, then the stop that
    // ends the loop, both carrying the same message.
    for (e, rendered) in terminal_cases() {
        let emitted: Vec<Emission> = events::outcomes_for_receive_error(&e)
            .iter()
            .flat_map(events::emissions_for)
            .collect();
        assert_eq!(
            names(&emitted),
            vec!["receiver-error", "receiver-stopped"],
            "{rendered:?}"
        );
        assert_eq!(
            emitted[0].payload["message"],
            serde_json::json!(rendered),
            "{rendered:?}"
        );
        assert_eq!(
            emitted[1].payload["reason"],
            serde_json::json!(rendered),
            "{rendered:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// REQ-kn5rtx — a variant reachable from a local condition is not a verdict
// ---------------------------------------------------------------------------

// verifies: LLR-wgr46u
#[test]
fn the_commit_workflow_refusals_are_classified_as_receiver_errors() {
    // The ten refusals of org-node's commit workflow. None is
    // a verdict of `verify_envelope_against_chain` on an update: the
    // reconcile refusals are about a chain state read or the node's own
    // record, a revocation's proof and an acknowledgement are not updates
    // shown under "Verified Updates (chain root match)", and the device
    // secret and the recipient refusals are about the caller. A misfiled
    // verdict is the hazard (HAZ-9fmhm4), a misfiled receiver error merely
    // less specific, so each is a receiver error.
    let org_id = org_io::node::OrgId::new([1; 20]);
    for e in [
        OrgNodeError::OrgNotHeld { org_id },
        OrgNodeError::StaleChainState {
            org_id,
            chain_epoch: org_io::node::Epoch::new(1),
            record_epoch: org_io::node::Epoch::new(2),
        },
        OrgNodeError::ChainStateConflict { org_id },
        OrgNodeError::RevocationProofRefused { org_id, cause: OrgMembersError::IdNotFound },
        OrgNodeError::AcknowledgementNotHeld { org_id },
        OrgNodeError::AcknowledgementFromFuture { org_id },
        OrgNodeError::AcknowledgementForListedDevice { org_id },
        OrgNodeError::AcknowledgementSignatureInvalid { org_id },
        OrgNodeError::DeviceSecretNotSupplied { org_id },
    ] {
        assert_eq!(
            events::classify_receive_error(&e),
            ReceiverOutcome::ReceiveError { message: e.to_string() },
            "{e:?} must be a receiver error"
        );
    }
}

// verifies: LLR-wgr46u
#[test]
fn the_sender_refusals_are_classified_as_receiver_errors() {
    // A message from a Device the record does
    // not list, or an acknowledgement not sent by its own Device, is refused
    // before anything is verified, so neither is a verdict on an update.
    let org_id = org_io::node::OrgId::new([1; 20]);
    for e in [OrgNodeError::SenderNotListed { org_id }, OrgNodeError::AcknowledgementNotFromItsDevice { org_id }] {
        assert_eq!(
            events::classify_receive_error(&e),
            ReceiverOutcome::ReceiveError { message: e.to_string() },
            "{e:?}"
        );
    }
}

// verifies: LLR-wgr46u
#[test]
fn the_store_and_invitation_refusals_are_classified_as_receiver_errors() {
    // The refusals of the sealed per-Persona store and the invitation
    // bookkeeping concern this node's own store or its outstanding Invites,
    // never whether an update verified, so each is a receiver error —
    // `OrganisationHeldByAnotherPersona` among them.
    let org_id = org_io::node::OrgId::new([1; 20]);
    for e in [
        OrgNodeError::DeviceSecretMismatch,
        OrgNodeError::DuplicatePersonaStore,
        OrgNodeError::InviteNotOutstanding { org_id, invite_id: org_io::node::InviteId::new([3; 32]) },
        OrgNodeError::OrganisationHeldByAnotherPersona { org_id },
    ] {
        assert_eq!(
            events::classify_receive_error(&e),
            ReceiverOutcome::ReceiveError { message: e.to_string() },
            "{e:?} must be a receiver error"
        );
    }
}

// verifies: LLR-wgr46u
#[test]
fn locally_reachable_variants_are_classified_as_receiver_errors() {
    // `OrgNotOnChain` and `Trie(_)` sound like verdicts and are not always:
    // each is also reachable from a LOCAL condition (no record held, a local
    // members snapshot that fails to reconstruct), so neither may be
    // rendered as a ✗ MISMATCH row — the defect RC-3rddh7 exists to remove.
    // An unexpected first admission, one that lists none of this node's
    // Personas, and the provisional limit are refusals about this node.
    // A message that does not decode and a revocation about an Organisation
    // not held are refused before anything is verified, and a revocation
    // that leaves this node listed after it, on a rule about this node.
    for e in [
        OrgNodeError::OrgNotOnChain,
        OrgNodeError::Trie(OrgMembersError::IdNotFound),
        OrgNodeError::AdmissionNotExpected { org_id: org_io::node::OrgId::new([1; 20]) },
        OrgNodeError::AdmissionNotOurs { org_id: org_io::node::OrgId::new([1; 20]) },
        OrgNodeError::ProvisionalLimit { limit: 1 },
        OrgNodeError::MalformedMessage,
        OrgNodeError::RevocationNotHeld { org_id: org_io::node::OrgId::new([1; 20]) },
        OrgNodeError::RevocationNotForThisDevice { org_id: org_io::node::OrgId::new([1; 20]) },
    ] {
        let outcome = events::classify_receive_error(&e);
        assert_eq!(
            outcome,
            ReceiverOutcome::ReceiveError {
                message: e.to_string()
            },
            "{e:?} is reachable from a local condition and must not be a verdict"
        );
        let out = events::emissions_for(&outcome);
        assert_eq!(names(&out), vec!["receiver-error"], "{e:?}");
        assert!(
            !has_key_anywhere(&out[0].payload, "org_id"),
            "{e:?}: a receiver error carries no organisation: {}",
            out[0].payload
        );
    }
}

// A Wire message over the bound (the codec's own refusal) is refused before anything is decoded, so it is no
// verdict on an update.
// verifies: LLR-wgr46u
#[test]
fn a_wire_message_over_the_bound_is_classified_as_a_receiver_error() {
    let e = OrgNodeError::MessageTooLarge { length: 1 << 21 };
    assert_eq!(events::classify_receive_error(&e), ReceiverOutcome::ReceiveError { message: e.to_string() });
}

// verifies: LLR-wgr46u
#[test]
fn a_refused_key_or_field_is_classified_as_a_receiver_error() {
    // Neither is a verdict on an update: an Organisation public key the chain
    // holds that is not a valid key, or a received or stored record holding
    // a value its type refuses. `InvalidOrgPublicKey` is an Organisation
    // public key that is not a valid X25519 key.
    let cases = [
        OrgNodeError::InvalidOrgPublicKey,
        OrgNodeError::InvalidField {
            field: "member.handle",
            reason: "invalid handle: handle must be lowercase".to_string(),
        },
    ];
    for e in cases {
        assert_eq!(
            events::classify_receive_error(&e),
            ReceiverOutcome::ReceiveError { message: e.to_string() },
            "{e:?} must be a receiver error"
        );
    }
}

// ---------------------------------------------------------------------------
// Whether a refusal about one Organisation or one update stops the loop
// ---------------------------------------------------------------------------

// REQ-jfxah3's loop stops only when it cannot continue: the transport
// endpoint is gone (org-io's `ReceiveError::Terminal`). None of these refusals is
// about the endpoint. An Organisation public key on chain that is not a valid
// X25519 key is refused on every read of that Organisation for as long as the
// chain holds it, but it concerns that one Organisation: the endpoint still
// receives for every other, so the loop goes on and reports each refusal as
// one receiver error. The same holds for a Sequence number that is not the
// chain's epoch, a verdict on one update.
// Each produces exactly one outcome, of its class, and no stop.
// verifies: REQ-jfxah3, REQ-kn5rtx
#[test]
fn a_refusal_this_change_added_does_not_stop_the_loop() {
    // Each refusal arrives as org-io's `ReceiveError::Refused`.
    let receiver_errors = [OrgNodeError::InvalidOrgPublicKey];
    for e in receiver_errors {
        let (message, shown) = (e.to_string(), format!("{e:?}"));
        assert_eq!(
            events::outcomes_for_receive_error(&ReceiveError::Refused(e)),
            vec![ReceiverOutcome::ReceiveError { message }],
            "{shown} is one receiver error and must not stop the loop"
        );
    }
    let verdict = OrgNodeError::SeqNotEpoch { seq: 9, epoch: 3 };
    assert_eq!(
        events::outcomes_for_receive_error(&ReceiveError::Refused(OrgNodeError::SeqNotEpoch { seq: 9, epoch: 3 })),
        vec![ReceiverOutcome::VerifyFailed { org_id: None, message: verdict.to_string() }],
        "a Sequence number other than the chain's epoch refuses one update and must not stop the loop"
    );
}

// ---------------------------------------------------------------------------
// org-io's typed receive error and its receive kinds
// ---------------------------------------------------------------------------

use org_io::transport::TransportError;
use org_io::{ReceiveError, TerminalCause};

// The loop stops on org-io's `ReceiveError::Terminal`, whatever its text, and
// on nothing else, whatever its text (REQ-x3c8n2, answered by org-io's
// REQ-8e57wm). Each case is checked twice: the outcomes, and the emissions in
// order.
// verifies: LLR-zw6jqy
#[test]
fn terminal_is_decided_by_the_error_variant_not_its_text() {
    for terminal in [
        ReceiveError::Terminal(TerminalCause::EndpointClosed),
        ReceiveError::Terminal(TerminalCause::BindFailed(TransportError::Bind("no interface to bind".to_string()))),
        // Boundary: a terminal cause whose own text is empty.
        ReceiveError::Terminal(TerminalCause::BindFailed(TransportError::Bind(String::new()))),
    ] {
        let message = terminal.to_string();
        assert_eq!(
            events::outcomes_for_receive_error(&terminal),
            vec![
                ReceiverOutcome::ReceiveError { message: message.clone() },
                ReceiverOutcome::Stopped { reason: message.clone() },
            ],
            "{terminal:?} is terminal: the failure, then the stop, both with its message"
        );
        let emitted: Vec<Emission> =
            events::outcomes_for_receive_error(&terminal).iter().flat_map(events::emissions_for).collect();
        assert_eq!(names(&emitted), vec!["receiver-error", "receiver-stopped"], "{terminal:?}");
    }

    // Abnormal input: failures after which the loop goes on, each with a
    // text that reads like a closed or unbound endpoint.
    for not_terminal in [
        ReceiveError::Connection(TransportError::Stream("endpoint closed".to_string())),
        ReceiveError::Connection(TransportError::Accept("endpoint bind: refused".to_string())),
        ReceiveError::Connection(TransportError::Stream("endpoint bind failed unexpectedly".to_string())),
        ReceiveError::Refused(OrgNodeError::Chain("endpoint closed".to_string())),
        ReceiveError::Connection(TransportError::Stream(String::new())),
        // org-io's own refusal of a body kept by a Persona bound to no
        // Organisation yet.
        ReceiveError::NoOrganisation { receiver: Box::new(org_io::node::DevicePublicKey::parse(&ARRIVAL_DEVICE_KEY).expect("a point")) },
    ] {
        let outcomes = events::outcomes_for_receive_error(&not_terminal);
        assert_eq!(
            outcomes,
            vec![ReceiverOutcome::ReceiveError { message: not_terminal.to_string() }],
            "{not_terminal:?} is one receiver error and must not stop the loop, whatever its text"
        );
    }

    // A refusal keeps org-node's classification (LLR-wgr46u) and never stops.
    let verdict = ReceiveError::Refused(OrgNodeError::RootMismatch);
    assert_eq!(
        events::outcomes_for_receive_error(&verdict),
        vec![ReceiverOutcome::VerifyFailed { org_id: None, message: OrgNodeError::RootMismatch.to_string() }]
    );
}

// Abnormal: a store write that failed after a
// check is one receiver error and never stops the loop: it is not org-io's
// `Terminal`. Both store failures are fed in, with the emissions checked as
// well.
// verifies: LLR-zw6jqy
#[test]
fn store_failures_are_one_receiver_error_and_do_not_stop_the_loop() {
    use org_io::store_file::{StoreFileError, StoreFileStep};
    use org_io::{OpenError, StoreFailure};

    let failed = || StoreFileError { step: StoreFileStep::Write, path: "/data/store".into(), reason: "disk full".into() };
    let failures = [
        ReceiveError::Store(StoreFailure::Reloaded { failed: failed() }),
        ReceiveError::Store(StoreFailure::Dropped {
            failed: failed(),
            reload: Box::new(OpenError::ReadStore(failed())),
        }),
    ];
    for failure in failures {
        assert_eq!(
            events::outcomes_for_receive_error(&failure),
            vec![ReceiverOutcome::ReceiveError { message: failure.to_string() }],
            "{failure:?} is one receiver error and must not stop the loop"
        );
        let emitted: Vec<Emission> =
            events::outcomes_for_receive_error(&failure).iter().flat_map(events::emissions_for).collect();
        assert_eq!(names(&emitted), vec!["receiver-error"], "{failure:?}");
    }
}

// ---------------------------------------------------------------------------
// What the receiver maps an arrival to
// ---------------------------------------------------------------------------

/// The ed25519 base point's canonical encoding: a valid Device public key.
const ARRIVAL_DEVICE_KEY: [u8; 32] = [
    0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
];

fn device_hex() -> String {
    hex::encode(ARRIVAL_DEVICE_KEY)
}

fn admission_requested() -> events::AdmissionRequested {
    events::AdmissionRequested {
        org_id: org(),
        invite_id: "1c".repeat(32),
        invitee_name: "Bob Builder".to_string(),
        name: "Bob".to_string(),
        surname: "Builder".to_string(),
        preferred_handles: vec!["bob".to_string(), "bobb".to_string(), "builder".to_string()],
        device_public_key: device_hex(),
    }
}

// verifies: LLR-4bvdn7
#[test]
fn a_received_provisional_organisation_is_provisional_received_never_updated() {
    let outcome = events::outcome_for_arrival(events::Arrival::ProvisionalOrganisation { org_id: org() });
    assert_eq!(outcome, ReceiverOutcome::ProvisionalReceived { org_id: org() });
    assert!(
        !matches!(outcome, ReceiverOutcome::Updated { .. } | ReceiverOutcome::RecordUnreadable { .. }),
        "a received provisional organisation is never an adoption: {outcome:?}"
    );
    let emitted = events::emissions_for(&outcome);
    for forbidden in MEMBERSHIP_EVENTS.iter().chain(["record-unreadable"].iter()) {
        assert!(
            !names(&emitted).contains(forbidden),
            "{forbidden} emitted for a provisional organisation: {:?}",
            names(&emitted)
        );
    }
    for emission in &emitted {
        for key in ["epoch", "root"] {
            assert!(!has_key_anywhere(&emission.payload, key), "{} carries `{key}`: {}", emission.name, emission.payload);
        }
    }
}

// verifies: LLR-ygr6gd
#[test]
fn removed_is_self_deleted_then_the_result_and_a_kept_notice_announces_nothing() {
    let entry = entry_of(EntryResultDto::Removed);
    // Whatever record is read back, `removed` is a self-delete, then the result.
    for record in [Some((7, "cd".repeat(32))), Some((0, String::new())), None] {
        assert_eq!(
            events::outcomes_for_verification(&entry, record.clone()),
            vec![
                ReceiverOutcome::SelfDeleted { org_id: Some(org()) },
                ReceiverOutcome::VerificationResult { entry: wire(&entry) },
            ],
            "record {record:?}"
        );
    }
    // The self-delete states no epoch (REQ-tw4cb5).
    assert_eq!(self_delete_epoch_offender(&org()), None);

    // A kept revocation notice names the Organisation and announces nothing.
    let kept = events::outcome_for_arrival(events::Arrival::RevocationNoticeKept { org_id: org() });
    assert_eq!(kept, ReceiverOutcome::RevocationNoticeKept { org_id: org() });
    assert_eq!(events::emissions_for(&kept), Vec::<Emission>::new());

    // An acknowledgement is AcknowledgementReceived, announced by nothing.
    let acknowledged = events::outcome_for_arrival(events::Arrival::Acknowledged { org_id: org() });
    assert_eq!(acknowledged, ReceiverOutcome::AcknowledgementReceived { org_id: org() });
    assert_eq!(events::emissions_for(&acknowledged), Vec::<Emission>::new());
}

// verifies: LLR-dp45mm
#[test]
fn an_admission_request_kept_is_one_admission_requested_event_with_its_fields() {
    let outcome = events::outcome_for_arrival(events::Arrival::AdmissionRequestKept(admission_requested()));
    assert_eq!(outcome, ReceiverOutcome::AdmissionRequested(admission_requested()));
    let emitted = events::emissions_for(&outcome);
    assert_eq!(names(&emitted), vec!["admission-requested"]);
    assert_eq!(
        emitted[0].payload,
        serde_json::json!({
            "org_id": org(),
            "invite_id": "1c".repeat(32),
            "invitee_name": "Bob Builder",
            "name": "Bob",
            "surname": "Builder",
            "preferred_handles": ["bob", "bobb", "builder"],
            "device_public_key": device_hex(),
        }),
        "exactly the request's fields, verbatim and in order"
    );
    assert_eq!(emitted[0].payload["org_id"].as_str().map(str::len), Some(40));
    assert_eq!(emitted[0].payload["invite_id"].as_str().map(str::len), Some(64));
    assert_eq!(emitted[0].payload["device_public_key"].as_str().map(str::len), Some(64));
}

// verifies: LLR-t9rdf8
#[test]
fn an_answered_organisation_request_is_announced_by_no_event() {
    let outcome = events::outcome_for_arrival(events::Arrival::OrganisationRequestAnswered {
        org_id: org(),
        requester: device_hex(),
    });
    assert_eq!(outcome, ReceiverOutcome::OrganisationRequestAnswered { org_id: org(), requester: device_hex() });
    assert_eq!(events::emissions_for(&outcome), Vec::<Emission>::new());
}

/// Every `OrgNodeError` variant the app can build through org-io's
/// re-exports, with whether it is a verdict on a delivered update
/// (LLR-wgr46u). `StoreRefused(_)` is the one variant left out: org-io
/// does not re-export `StoreRefusal`, so the app cannot build it. The
/// no-wildcard scan below covers it, because its arm must name it.
fn every_org_node_error_variant() -> Vec<(org_io::node::OrgNodeError, bool)> {
    use org_io::node::{DevicePublicKey, Epoch, MemberId, OrgId, OrgMembersError, OrgNodeError};
    let org_id = OrgId::new([1; 20]);
    let member_id = MemberId::new([2; 32]);
    let recipient = DevicePublicKey::parse(&ARRIVAL_DEVICE_KEY).unwrap();
    let invite_id = org_io::node::InviteId::new([3; 32]);
    vec![
        // The five refusals of a delivered update's own content, and the two chain verdicts.
        (OrgNodeError::OrgIdMismatch, true),
        (OrgNodeError::StaleSeq { got: 3, last_seen: 7 }, true),
        (OrgNodeError::MalformedDelta, true),
        (OrgNodeError::DeltaBaseMismatch, true),
        (OrgNodeError::SeqNotEpoch { seq: 9, epoch: 3 }, true),
        (OrgNodeError::RootMismatch, true),
        (OrgNodeError::StaleEpoch { got: 2, last: 9 }, true),
        // Every other variant: a receiver error.
        (OrgNodeError::Chain("io".to_string()), false),
        (OrgNodeError::OrgNotOnChain, false),
        (OrgNodeError::Trie(OrgMembersError::IdNotFound), false),
        (OrgNodeError::InvalidOrgPublicKey, false),
        (OrgNodeError::InvalidField { field: "member.handle", reason: "bad".to_string() }, false),
        (OrgNodeError::AdmissionNotExpected { org_id }, false),
        (OrgNodeError::AdmissionNotOurs { org_id }, false),
        (OrgNodeError::ProvisionalLimit { limit: 1 }, false),
        (OrgNodeError::MalformedMessage, false),
        (OrgNodeError::MessageTooLarge { length: 1 << 21 }, false),
        (OrgNodeError::RevocationNotHeld { org_id }, false),
        (OrgNodeError::RevocationNotForThisDevice { org_id }, false),
        (OrgNodeError::OrgNotHeld { org_id }, false),
        (OrgNodeError::StaleChainState { org_id, chain_epoch: Epoch::new(1), record_epoch: Epoch::new(2) }, false),
        (OrgNodeError::ChainStateConflict { org_id }, false),
        (OrgNodeError::RevocationProofRefused { org_id, cause: OrgMembersError::IdNotFound }, false),
        (OrgNodeError::AcknowledgementNotHeld { org_id }, false),
        (OrgNodeError::AcknowledgementFromFuture { org_id }, false),
        (OrgNodeError::AcknowledgementForListedDevice { org_id }, false),
        (OrgNodeError::AcknowledgementSignatureInvalid { org_id }, false),
        (OrgNodeError::DeviceSecretNotSupplied { org_id }, false),
        (OrgNodeError::SenderNotListed { org_id }, false),
        (OrgNodeError::AcknowledgementNotFromItsDevice { org_id }, false),
        (OrgNodeError::DeviceSecretMismatch, false),
        (OrgNodeError::DuplicatePersonaStore, false),
        (OrgNodeError::InviteNotOutstanding { org_id, invite_id }, false),
        (OrgNodeError::OrganisationHeldByAnotherPersona { org_id }, false),
        (OrgNodeError::NoProvisionalChange, false),
        (OrgNodeError::OrganisationBehind { org_id, chain_epoch: Epoch::new(5) }, false),
        (OrgNodeError::NoAdmissionRequestKept { invite_id }, false),
        (OrgNodeError::AdmissionAlreadyRecorded { invite_id }, false),
        (OrgNodeError::NoFreeHandle, false),
        (OrgNodeError::HandleTaken { handle: org_io::node::Handle::parse("zed").unwrap() }, false),
        (OrgNodeError::MemberNotHeld { member_id }, false),
        (OrgNodeError::RevocationAlreadyRecorded { member_id }, false),
        (OrgNodeError::NoProvisionalFounding, false),
        (OrgNodeError::FoundingProxyAlreadyRecorded, false),
        (OrgNodeError::FoundingNotWritten, false),
        (OrgNodeError::NoProvisionalMember, false),
        (OrgNodeError::OrganisationNotRequested { org_id }, false),
        (OrgNodeError::PersonaNotHeld { device_key: Box::new(recipient) }, false),
        (OrgNodeError::PersonaIdNotHeld { persona_id: org_io::node::PersonaId::new("nobody".into()) }, false),
        (OrgNodeError::AdmissionKeyNotSender { org_id }, false),
        (OrgNodeError::OrganisationRemoved { org_id }, false),
        (OrgNodeError::AcknowledgementNotKept { recipient: Box::new(recipient) }, false),
        (OrgNodeError::DuplicateOrgPublicKey, false),
    ]
}

/// `classify_receive_error`'s body, as written in `src/events.rs`.
fn classify_source() -> &'static str {
    let source = include_str!("../src/events.rs");
    let start = source.find("pub fn classify_receive_error").expect("classify_receive_error is defined");
    let end = start + source[start..].find("\n}\n").expect("its body ends at a closing brace in column one");
    &source[start..end]
}

// verifies: LLR-wgr46u
#[test]
fn every_org_node_error_variant_is_classified_with_no_wildcard() {
    for (refusal, verdict) in every_org_node_error_variant() {
        let expected = if verdict {
            ReceiverOutcome::VerifyFailed { org_id: None, message: refusal.to_string() }
        } else {
            ReceiverOutcome::ReceiveError { message: refusal.to_string() }
        };
        assert_eq!(events::classify_receive_error(&refusal), expected, "{refusal:?}");
    }
    let body = classify_source();
    assert!(!body.contains("_ =>"), "classify_receive_error has a wildcard arm:\n{body}");
    let all_variants = [
        "OrgIdMismatch",
        "StaleSeq",
        "MalformedDelta",
        "DeltaBaseMismatch",
        "SeqNotEpoch",
        "RootMismatch",
        "StaleEpoch",
        "Chain",
        "OrgNotOnChain",
        "Trie",
        "InvalidOrgPublicKey",
        "InvalidField",
        "AdmissionNotExpected",
        "AdmissionNotOurs",
        "ProvisionalLimit",
        "MalformedMessage",
        "MessageTooLarge",
        "RevocationNotHeld",
        "RevocationNotForThisDevice",
        "OrgNotHeld",
        "StaleChainState",
        "ChainStateConflict",
        "RevocationProofRefused",
        "AcknowledgementNotHeld",
        "AcknowledgementFromFuture",
        "AcknowledgementForListedDevice",
        "AcknowledgementSignatureInvalid",
        "DeviceSecretNotSupplied",
        "SenderNotListed",
        "AcknowledgementNotFromItsDevice",
        "StoreRefused",
        "DeviceSecretMismatch",
        "DuplicatePersonaStore",
        "InviteNotOutstanding",
        "OrganisationHeldByAnotherPersona",
        "NoProvisionalChange",
        "OrganisationBehind",
        "NoAdmissionRequestKept",
        "AdmissionAlreadyRecorded",
        "NoFreeHandle",
        "HandleTaken",
        "MemberNotHeld",
        "RevocationAlreadyRecorded",
        "NoProvisionalFounding",
        "FoundingProxyAlreadyRecorded",
        "FoundingNotWritten",
        "NoProvisionalMember",
        "OrganisationNotRequested",
        "PersonaNotHeld",
        "PersonaIdNotHeld",
        "AdmissionKeyNotSender",
        "OrganisationRemoved",
        "AcknowledgementNotKept",
        "DuplicateOrgPublicKey",
    ];
    for variant in all_variants {
        assert!(body.contains(&format!("OrgNodeError::{variant}")), "classify_receive_error names no arm for {variant}");
    }
    // The removed variants are named nowhere (org-node's final error set).
    for removed in [
        "NoProvisionalUpdate",
        "ReplyForAnotherOrganisation",
        "PersonaAlreadyBound",
        "RecordMovedSinceCheck",
        "OrgKeyMismatch",
    ] {
        assert!(!body.contains(removed), "classify_receive_error still names {removed}");
    }
}

// verifies: LLR-4bvdn7, LLR-ygr6gd, LLR-dp45mm, LLR-t9rdf8
#[test]
fn the_outcome_kind_comes_from_the_received_kind() {
    let cases = [
        (events::Arrival::ProvisionalOrganisation { org_id: org() }, ReceiverOutcome::ProvisionalReceived { org_id: org() }),
        (events::Arrival::AdmissionRequestKept(admission_requested()), ReceiverOutcome::AdmissionRequested(admission_requested())),
        (
            events::Arrival::OrganisationRequestAnswered { org_id: org(), requester: device_hex() },
            ReceiverOutcome::OrganisationRequestAnswered { org_id: org(), requester: device_hex() },
        ),
        (events::Arrival::RevocationNoticeKept { org_id: org() }, ReceiverOutcome::RevocationNoticeKept { org_id: org() }),
        (events::Arrival::Acknowledged { org_id: org() }, ReceiverOutcome::AcknowledgementReceived { org_id: org() }),
    ];
    for (arrival, expected) in cases {
        let outcome = events::outcome_for_arrival(arrival.clone());
        assert_eq!(outcome, expected, "{arrival:?}");
        // No arrival is an adoption, and none is a self-delete: those come from verification only.
        assert!(
            !matches!(
                outcome,
                ReceiverOutcome::Updated { .. } | ReceiverOutcome::RecordUnreadable { .. } | ReceiverOutcome::SelfDeleted { .. }
            ),
            "{arrival:?} gave {outcome:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// The follow-up verification and its outcomes
// ---------------------------------------------------------------------------

// verifies: LLR-j9m88u
// `receive_iteration` has its own item, beside `follow_up_for`'s
// (LLR-w7m3d4), since the order it states cannot be observed through the
// spawned receiver loop without faking org-io.
#[tokio::test]
async fn one_iteration_verifies_once_before_the_next_message_and_announces_what_it_verified() {
    use std::cell::RefCell;
    let second_org = "bb".repeat(20);
    let verified_entry = |org_id: &str| serde_json::json!({ "result": "in_step", "org_id": org_id, "epoch": 2 });
    // Popped from the end: the first message, then the second, then the third.
    let script = RefCell::new(vec![
        vec![ReceiverOutcome::AdmissionRequested(admission_requested())],
        vec![ReceiverOutcome::RevocationNoticeKept { org_id: second_org.clone() }],
        vec![ReceiverOutcome::ProvisionalReceived { org_id: org() }],
    ]);
    let log = RefCell::new(Vec::<String>::new());
    for _ in 0..3 {
        let stopping = events::receive_iteration(
            || {
                log.borrow_mut().push("next".to_string());
                std::future::ready(script.borrow_mut().pop().unwrap())
            },
            |org_id: String| {
                log.borrow_mut().push(format!("verify {org_id}"));
                std::future::ready(vec![ReceiverOutcome::VerificationResult { entry: verified_entry(&org_id) }])
            },
            |emission: Emission| log.borrow_mut().push(format!("emit {}", emission.name)),
        )
        .await;
        assert!(!stopping, "no outcome stopped the loop");
    }
    assert_eq!(
        *log.borrow(),
        vec![
            "next".to_string(),
            format!("verify {}", org()),
            "emit verification-result".to_string(),
            "next".to_string(),
            format!("verify {second_org}"),
            "emit verification-result".to_string(),
            "next".to_string(),
            "emit admission-requested".to_string(),
        ],
        "one verification per provisional organisation or kept notice, announced before the next message, and none otherwise"
    );

    // A stopped receive ends the loop, after its event is emitted.
    let emitted = RefCell::new(Vec::<&'static str>::new());
    let stopping = events::receive_iteration(
        || std::future::ready(vec![ReceiverOutcome::Stopped { reason: "endpoint closed".to_string() }]),
        |_org_id: String| std::future::ready(Vec::new()),
        |emission: Emission| emitted.borrow_mut().push(emission.name),
    )
    .await;
    assert!(stopping, "a Stopped outcome stops the loop");
    assert_eq!(*emitted.borrow(), vec!["receiver-stopped"]);
}

// verifies: LLR-w7m3d4
#[test]
fn follow_up_for_asks_a_verification_for_a_provisional_or_a_kept_notice_only() {
    // follow_up_for: exactly those two outcomes ask, every other asks nothing.
    assert_eq!(
        events::follow_up_for(&ReceiverOutcome::ProvisionalReceived { org_id: org() }),
        Some(events::FollowUp::Verify { org_id: org() })
    );
    assert_eq!(
        events::follow_up_for(&ReceiverOutcome::RevocationNoticeKept { org_id: org() }),
        Some(events::FollowUp::Verify { org_id: org() })
    );
    for other in [
        ReceiverOutcome::SelfDeleted { org_id: Some(org()) },
        ReceiverOutcome::AcknowledgementReceived { org_id: org() },
        ReceiverOutcome::AdmissionRequested(admission_requested()),
        ReceiverOutcome::OrganisationRequestAnswered { org_id: org(), requester: device_hex() },
        ReceiverOutcome::Updated { org_id: org(), epoch: 2, root: "ff".repeat(32) },
        ReceiverOutcome::RecordUnreadable { org_id: Some(org()) },
        ReceiverOutcome::VerificationResult { entry: serde_json::json!({ "result": "in_step", "org_id": org(), "epoch": 2 }) },
        ReceiverOutcome::VerifyFailed { org_id: None, message: "m".to_string() },
        ReceiverOutcome::ReceiveError { message: "m".to_string() },
        ReceiverOutcome::Stopped { reason: "r".to_string() },
    ] {
        assert_eq!(events::follow_up_for(&other), None, "{other:?}");
    }
}

// verifies: LLR-acb5g4
#[test]
fn adopted_maps_to_updated_or_record_unreadable_then_the_result() {
    let root = "cd".repeat(32);
    let adopted = entry_of(adopted_at_4(true));
    assert_eq!(
        events::outcomes_for_verification(&adopted, Some((4, root.clone()))),
        vec![
            ReceiverOutcome::Updated { org_id: org(), epoch: 4, root: root.clone() },
            ReceiverOutcome::VerificationResult { entry: wire(&adopted) },
        ]
    );
    assert_eq!(
        events::outcomes_for_verification(&adopted, None),
        vec![
            ReceiverOutcome::RecordUnreadable { org_id: Some(org()) },
            ReceiverOutcome::VerificationResult { entry: wire(&adopted) },
        ]
    );
    // A verification that also settled an unknown write is entered as its
    // inner outcome with a `write` label (LLR-jf63e4): an adoption still.
    let settled = VerificationEntryDto { write: Some(WriteLabelDto::Adopted), ..entry_of(adopted_at_4(true)) };
    assert_eq!(
        events::outcomes_for_verification(&settled, Some((4, root.clone()))),
        vec![
            ReceiverOutcome::Updated { org_id: org(), epoch: 4, root: root.clone() },
            ReceiverOutcome::VerificationResult { entry: wire(&settled) },
        ]
    );
    // Every other result is the result alone, whatever record reads back.
    // The entry is typed, so a result the app does not know cannot be
    // built; a result the entry gains is a compile error in
    // `outcomes_for_verification`.
    for entry in [
        entry_of(EntryResultDto::InStep { epoch: 4 }),
        entry_of(EntryResultDto::Behind { chain_epoch: 9, requests: Some(Vec::new()), requests_refused: None }),
        entry_of(EntryResultDto::Inconsistent { reason: "root differs".to_string() }),
        entry_of(EntryResultDto::Unverified { reason: "chain not configured".to_string() }),
        VerificationEntryDto { write: Some(WriteLabelDto::NotExecuted), ..entry_of(EntryResultDto::InStep { epoch: 4 }) },
        VerificationEntryDto {
            org_id: None,
            founder: Some(device_hex()),
            result: EntryResultDto::Unverified { reason: "the founding is not written".to_string() },
            write: None,
        },
    ] {
        for record in [Some((4, root.clone())), None] {
            assert_eq!(
                events::outcomes_for_verification(&entry, record.clone()),
                vec![ReceiverOutcome::VerificationResult { entry: wire(&entry) }],
                "{entry:?} with {record:?}"
            );
        }
    }
}

// verifies: LLR-acb5g4, LLR-ygr6gd
// An entry with no Organisation
// identifier (a provisional founding whose proxy is not recorded) names
// none in the `record-unreadable` or `revoked` event: `org_id` null, never
// an empty string, as the entry itself carries it (LLR-jf63e4).
#[test]
fn an_entry_with_no_org_id_names_none_in_record_unreadable_or_revoked() {
    let founding = |result| VerificationEntryDto { org_id: None, founder: Some(device_hex()), result, write: None };
    for (entry, outcome, event) in [
        (founding(adopted_at_4(true)), ReceiverOutcome::RecordUnreadable { org_id: None }, "record-unreadable"),
        (founding(EntryResultDto::Removed), ReceiverOutcome::SelfDeleted { org_id: None }, "revoked"),
    ] {
        let outcomes = events::outcomes_for_verification(&entry, None);
        assert_eq!(outcomes, vec![outcome, ReceiverOutcome::VerificationResult { entry: wire(&entry) }], "{entry:?}");
        let emitted: Vec<Emission> = outcomes.iter().flat_map(events::emissions_for).collect();
        assert_eq!(names(&emitted), vec![event, "verification-result"]);
        assert_eq!(emitted[0].payload, serde_json::json!({ "org_id": null }), "{event} names no Organisation");
        assert_eq!(emitted[1].payload["founder"], serde_json::json!(device_hex()), "the result names the founding");
    }
}

// verifies: LLR-8grnbb
#[test]
fn a_verification_result_is_one_event_carrying_the_entry() {
    let device = device_hex();
    for entry in [
        serde_json::json!({
            "result": "adopted", "org_id": org(), "epoch": 4, "own": true,
            "discarded": [
                { "kind": "admission", "invite_id": "1c".repeat(32), "name": "Bob", "surname": "Builder", "handle": "bob" },
                { "kind": "revocation", "member_id": "2d".repeat(32), "handle": "carol" }
            ],
            "sends": [ { "device": device, "result": "sent" }, { "device": "3e".repeat(32), "result": "failed", "reason": "no address" } ]
        }),
        serde_json::json!({
            "result": "behind", "org_id": org(), "chain_epoch": 9,
            "requests": [ { "device": device, "result": "failed", "reason": "timed out" } ]
        }),
        serde_json::json!({ "result": "in_step", "org_id": org(), "epoch": 4 }),
        serde_json::json!({ "result": "in_step", "org_id": org(), "epoch": 4, "write": "deprecated" }),
        serde_json::json!({ "result": "unverified", "org_id": null, "founder": device.clone(), "reason": "refused by org-node" }),
        serde_json::json!({ "result": "removed", "org_id": org() }),
    ] {
        let emitted = events::emissions_for(&ReceiverOutcome::VerificationResult { entry: entry.clone() });
        assert_eq!(names(&emitted), vec!["verification-result"], "{entry}");
        assert_eq!(emitted[0].payload, entry, "the payload is the entry, unchanged");
    }
    // Through the mapping: an adoption announces the record's events, then exactly one result.
    let adopted = entry_of(adopted_at_4(false));
    let emitted: Vec<Emission> = events::outcomes_for_verification(&adopted, Some((4, "cd".repeat(32))))
        .iter()
        .flat_map(events::emissions_for)
        .collect();
    assert_eq!(
        names(&emitted),
        vec!["membership-updated", "incoming-verified", "epoch-changed", "verification-result"]
    );
    assert_eq!(emitted[3].payload, wire(&adopted));
}
