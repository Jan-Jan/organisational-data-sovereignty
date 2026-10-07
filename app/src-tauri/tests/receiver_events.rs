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

/// Every emission the self-delete path produces for `org_id`, with the offending
/// payload returned when any of them carries an `epoch` key at any depth.
fn self_delete_epoch_offender(org_id: &str) -> Option<serde_json::Value> {
    events::emissions_for(&ReceiverOutcome::SelfDeleted {
        org_id: org_id.to_string(),
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
    let out = events::emissions_for(&ReceiverOutcome::RecordUnreadable { org_id: org() });
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
    let out = events::emissions_for(&ReceiverOutcome::RecordUnreadable { org_id: org() });
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
    let out = events::emissions_for(&ReceiverOutcome::RecordUnreadable { org_id: org() });
    assert_eq!(names(&out), vec!["record-unreadable"]);
    assert_eq!(out[0].payload["org_id"], serde_json::json!(org()));
}

// verifies: LLR-hgsdm8
#[test]
fn record_unreadable_emits_exactly_one_event() {
    let out = events::emissions_for(&ReceiverOutcome::RecordUnreadable { org_id: org() });
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
            org_id: org_id.to_string(),
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
    let out = events::emissions_for(&ReceiverOutcome::SelfDeleted { org_id: org() });
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
/// A received acknowledgement announces nothing to the UI (S3: org-io keeps
/// it, from S3b-io).
#[test]
fn a_received_acknowledgement_emits_nothing() {
    assert!(events::emissions_for(&ReceiverOutcome::AcknowledgementReceived { org_id: org() }).is_empty());
}

// verifies: LLR-2vg79y
#[test]
fn self_delete_emits_revoked_naming_the_organisation() {
    let out = events::emissions_for(&ReceiverOutcome::SelfDeleted { org_id: org() });
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
    let out = events::emissions_for(&ReceiverOutcome::SelfDeleted { org_id: org() });
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
            org_id: org_id.to_string(),
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
// The receiver loop used to route EVERY `Err` from the receive path into
// `VerifyFailed`, which the frontend renders as a "✗ MISMATCH" row under
// "Verified Updates (chain root match)". A transport hiccup or a chain read
// failure was therefore displayed to the operator as a root mismatch — a fresh
// instance of HAZ-9fmhm4, the verification display misrepresenting the
// verification state.
//
// The classification is asserted VARIANT BY VARIANT, against concrete event
// names. It deliberately does not re-derive the partition from a predicate: a
// test that recomputed the match would agree with any match.

use org_members::OrgMembersError;
use org_node::OrgNodeError;

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
        // Added 2026-10-05: a Sequence number that is not the chain's epoch
        // (an org-node commit rule) refuses the received update itself.
        OrgNodeError::SeqNotEpoch { seq: 9, epoch: 3 },
    ]
}

// verifies: LLR-7bk6qh
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

// verifies: LLR-7bk6qh
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

// verifies: LLR-7bk6qh
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

// verifies: LLR-7bk6qh, LLR-usxk57
#[test]
fn a_chain_failure_emits_no_verification_event_for_any_message() {
    // Abnormal inputs: an empty message, a terminal one the loop's substring
    // check looks for, and an ordinary transient one. None of them is a verdict,
    // so none of them may be announced as one — including the terminal case,
    // where the loop also stops.
    for msg in [
        "",
        "iroh recv: iroh accept error: endpoint closed",
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

// verifies: LLR-7bk6qh
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
// The terminal check was a substring match on "endpoint not bound", a string
// neither org-node nor iroh ever formats. The loop therefore never broke, the
// `receiver-stopped` event was never emitted, and the guard was never released
// by task exit: the trigger for REQ-jfxah3 was unreachable.
//
// The cases below are the messages the receive path really produces, written
// exactly as they reach the app — `OrgNodeError::Chain`'s Display prefixes
// "chain read failed: ", and the transport's own Display prefixes the rest. The
// expected rendering is spelled out beside each one so that a reworded message
// upstream fails HERE rather than silently resurrecting the infinite retry.

/// Every terminal case: the error as the app receives it, and the message the
/// operator is shown.
fn terminal_cases() -> Vec<(OrgNodeError, &'static str)> {
    vec![
        // `ensure_endpoint` in org-node/src/service.rs — OrgEndpoint::bind_with_mode failed.
        (
            OrgNodeError::Chain(
                "endpoint bind: iroh bind error: address already in use".to_string(),
            ),
            "chain read failed: endpoint bind: iroh bind error: address already in use",
        ),
        // `ensure_endpoint` again — the endpoint is still None afterwards.
        (
            OrgNodeError::Chain("endpoint bind failed unexpectedly".to_string()),
            "chain read failed: endpoint bind failed unexpectedly",
        ),
        // `recv_one` in org-node/src/transport/endpoint.rs — accept() returned None.
        (
            OrgNodeError::Chain("iroh recv: iroh accept error: endpoint closed".to_string()),
            "chain read failed: iroh recv: iroh accept error: endpoint closed",
        ),
    ]
}

// verifies: LLR-a9rjtf
#[test]
fn every_real_terminal_message_stops_the_loop() {
    for (e, rendered) in terminal_cases() {
        assert_eq!(
            e.to_string(),
            rendered,
            "the message the operator sees has changed; the match is keyed to it"
        );
        let outcomes = events::outcomes_for_receive_error(&e);
        assert!(
            outcomes.contains(&ReceiverOutcome::Stopped {
                reason: rendered.to_string()
            }),
            "{rendered:?} is terminal and must stop the loop, got {outcomes:?}"
        );
    }
}

// verifies: LLR-a9rjtf
#[test]
fn a_non_terminal_failure_does_not_stop_the_loop() {
    // Abnormal inputs: a transport hiccup, a chain timeout, a local startup
    // condition and an empty message. None of them is terminal — stopping on one
    // would end the receiver for a failure it could have retried past.
    for inner in [
        "iroh recv: stream error: connection reset by peer",
        "rpc timeout",
        "no persona found — create one first",
        "",
    ] {
        let e = OrgNodeError::Chain(inner.to_string());
        let outcomes = events::outcomes_for_receive_error(&e);
        assert_eq!(
            outcomes,
            vec![ReceiverOutcome::ReceiveError {
                message: e.to_string()
            }],
            "Chain({inner:?}) is not terminal and must not stop the loop"
        );
    }
    // …and neither is a verdict on an update: a root mismatch is a reason to
    // reject that envelope, not a reason to stop receiving.
    let verdict = OrgNodeError::RootMismatch;
    assert_eq!(
        events::outcomes_for_receive_error(&verdict),
        vec![ReceiverOutcome::VerifyFailed {
            org_id: None,
            message: verdict.to_string()
        }]
    );
}

// verifies: LLR-a9rjtf
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

// verifies: LLR-7bk6qh
#[test]
fn the_commit_workflow_refusals_are_classified_as_receiver_errors() {
    // The ten refusals org-node gained with the commit workflow (S3). None is
    // a verdict of `verify_envelope_against_chain` on an update: the
    // reconcile refusals are about a chain state read or the node's own
    // record, a revocation's proof and an acknowledgement are not updates
    // shown under "Verified Updates (chain root match)", and the device
    // secret and the recipient refusals are about the caller. A misfiled
    // verdict is the hazard (HAZ-9fmhm4), a misfiled receiver error merely
    // less specific, so each is a receiver error.
    let org_id = org_node::OrgId::new([1; 20]);
    for e in [
        OrgNodeError::OrgNotHeld { org_id },
        OrgNodeError::StaleChainState {
            org_id,
            chain_epoch: org_node::Epoch::new(1),
            record_epoch: org_node::Epoch::new(2),
        },
        OrgNodeError::ChainStateConflict { org_id },
        OrgNodeError::RevocationProofRefused { org_id, cause: OrgMembersError::IdNotFound },
        OrgNodeError::AcknowledgementNotHeld { org_id },
        OrgNodeError::AcknowledgementFromFuture { org_id },
        OrgNodeError::AcknowledgementForListedDevice { org_id },
        OrgNodeError::AcknowledgementSignatureInvalid { org_id },
        OrgNodeError::DeviceSecretNotSupplied { org_id },
        OrgNodeError::NoRevocationForRecipient { org_id },
    ] {
        assert_eq!(
            events::classify_receive_error(&e),
            ReceiverOutcome::ReceiveError { message: e.to_string() },
            "{e:?} must be a receiver error"
        );
    }
}

// verifies: LLR-7bk6qh
#[test]
fn the_sender_refusals_are_classified_as_receiver_errors() {
    // Owner rulings of 2026-10-07: a message from a Device the record does
    // not list, or an acknowledgement not sent by its own Device, is refused
    // before anything is verified, so neither is a verdict on an update.
    let org_id = org_node::OrgId::new([1; 20]);
    for e in [OrgNodeError::SenderNotListed { org_id }, OrgNodeError::AcknowledgementNotFromItsDevice { org_id }] {
        assert_eq!(
            events::classify_receive_error(&e),
            ReceiverOutcome::ReceiveError { message: e.to_string() },
            "{e:?}"
        );
    }
}

// verifies: LLR-7bk6qh
#[test]
fn locally_reachable_variants_are_classified_as_receiver_errors() {
    // `OrgNotOnChain` and `Trie(_)` sound like verdicts and are not always:
    // `receive_and_self_delete_if_revoked` in org-node/src/service.rs returns
    // `OrgNotOnChain` when the LOCAL store has no record of the organisation,
    // and the same function returns `Trie(_)` when the LOCAL members snapshot
    // fails to reconstruct. Neither says anything about
    // whether an update verified, so neither may be rendered as a ✗ MISMATCH row
    // under "Verified Updates (chain root match)" — the defect RC-3rddh7 exists
    // to remove. Of the four refusals org-node gained with provisional
    // operations, an unexpected first admission is refused before anything is
    // verified, one that lists none of this node's Personas on a rule about
    // this node, and the other two never arise on receive. A Persona already
    // bound to an Organisation is a refusal about this node's own store,
    // raised before anything is verified.
    // The four refusals of a received Wire message are not verdicts of
    // `verify_envelope_against_chain` either: a message that does not decode
    // and a revocation about an Organisation not held are refused before
    // anything is verified, and a key that is not the chain's and a
    // revocation that leaves this node listed after it, on a rule about the
    // key or this node.
    for e in [
        OrgNodeError::OrgNotOnChain,
        OrgNodeError::Trie(OrgMembersError::IdNotFound),
        OrgNodeError::AdmissionNotExpected { org_id: org_node::OrgId::new([1; 20]) },
        OrgNodeError::AdmissionNotOurs { org_id: org_node::OrgId::new([1; 20]) },
        OrgNodeError::ProvisionalLimit { limit: 1 },
        OrgNodeError::NoProvisionalUpdate,
        OrgNodeError::PersonaAlreadyBound { persona_id: org_node::PersonaId::new("p".into()) },
        OrgNodeError::MalformedMessage,
        OrgNodeError::OrgKeyMismatch { org_id: org_node::OrgId::new([1; 20]) },
        OrgNodeError::RevocationNotHeld { org_id: org_node::OrgId::new([1; 20]) },
        OrgNodeError::RevocationNotForThisDevice { org_id: org_node::OrgId::new([1; 20]) },
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

// verifies: LLR-7bk6qh
#[test]
fn a_refused_key_or_field_is_classified_as_a_receiver_error() {
    // Neither is a verdict on an update: an Organisation public key the chain
    // holds that is not a valid key, or a received or stored record holding
    // a value its type refuses. Before org-node parsed these they surfaced as
    // `Chain(..)` or `Trie(..)`, both receiver errors; the class is unchanged.
    // Merged 2026-10-05 into worktree-person-shared-types: master's
    // `InvalidKey` (an Organisation public key off the Edwards curve) is
    // `InvalidOrgPublicKey` here (an invalid X25519 key).
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
// Review round 3a of worktree-person-shared-types (2026-10-05): whether each
// refusal this change added stops the loop
// ---------------------------------------------------------------------------

// Finding-12. REQ-jfxah3's loop stops only when it cannot continue: the
// transport endpoint is gone (`TERMINAL_ERRORS`). None of these refusals is
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
    let receiver_errors = [OrgNodeError::InvalidOrgPublicKey];
    for e in receiver_errors {
        assert_eq!(
            events::outcomes_for_receive_error(&e),
            vec![ReceiverOutcome::ReceiveError { message: e.to_string() }],
            "{e:?} is one receiver error and must not stop the loop"
        );
    }
    let verdict = OrgNodeError::SeqNotEpoch { seq: 9, epoch: 3 };
    assert_eq!(
        events::outcomes_for_receive_error(&verdict),
        vec![ReceiverOutcome::VerifyFailed { org_id: None, message: verdict.to_string() }],
        "a Sequence number other than the chain's epoch refuses one update and must not stop the loop"
    );
}
