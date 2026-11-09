#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! Acting on this node's removal (SDD-3my66m and SDD-wyn7b6): the delivery of the
//! acknowledgements org-node keeps, from the removed Persona's own
//! identity over a direct bind, on a schedule, and the Persona's deletion.
//! The normal-case deliveries go to real receiving handles, whose receive
//! step decides each acknowledgement. The schedule, deletion and failure
//! tests script the recipient's verdict with a stand-in endpoint: `Accepted`
//! and `Refused` under the contract test
//! `the_scripted_recipients_verdicts_are_the_real_receive_steps`, and as
//! fault injection the verdicts a real step never writes for an
//! acknowledgement, a dropped reply and dead addresses. Every test runs one
//! at a time under `fresh_keychain`'s lock, over a fresh in-memory keychain.

use std::sync::Arc;
use std::time::Duration;

use iroh::address_lookup::MemoryLookup;

use org_io::send::SendError;
use org_io::test_support::{store_writes_to, Refusal, TestClock};
use org_io::transport::{BindPlan, OrgEndpoint, TransportMode, Verdict};
use org_io::{AcceptanceFailure, AcknowledgementOutcome, AcknowledgementReport, OrgIo, ReceiveError, Received, Verified};

mod common;
use common::handles::{
    file_of, fresh_keychain, item_held, keep_device_seed, make_read_only, make_writable, names_in, removed_file_of, store_and_removed_files,
    store_dir, NET,
};
use common::journey;

const QUIET: Duration = Duration::from_secs(2);

// The tests that count a refusing endpoint's arrivals keep only its first
// address: a round tries each kept address after a refusal (LLR-u2j6u8),
// so an endpoint bound on two sockets would be reached twice per round.

/// Wait, up to `limit`, until `io` reports no removal of `org`: its Persona
/// was deleted and its delivery is over.
async fn ack_report_gone(io: &org_io::OrgIo, org: org_node::OrgId, limit: Duration) {
    tokio::time::timeout(limit, async {
        while io.acknowledgement_report(org).await.is_some() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("the removal came to be gone");
}

// The verification that finds this node removed writes the removed store,
// starts the Persona's delivery task and returns `Removed` while the first
// acknowledgement still waits for its verdict: the Admin's real receive step
// takes it only when the test asks. Another verification starts no second
// task (the removed store is no target any more), and nor does the start at
// `start_receiving`, which resumes each kept removal: a second task would
// send its first round at once, a second acknowledgement the Admin's receive
// step would take (seen at the recipient, not through a count of the
// handle's tasks).
// verifies: LLR-zzsqj4
#[tokio::test(flavor = "multi_thread")]
async fn a_removed_outcome_starts_one_delivery_task_and_returns_without_awaiting_it() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "ack-start";
    let admin = journey::admin(tag).await;
    let bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    journey::revoked(&admin, &bob, &[]).await;

    let verified = tokio::time::timeout(Duration::from_secs(3), bob.io.verify_all()).await.expect("returns at once");
    assert!(matches!(verified.as_slice(), [(_, Verified::Removed { .. })]), "{verified:?}");
    assert!(bob.io.verify_all().await.is_empty(), "a removed store is no verification target");
    bob.io.start_receiving().await.unwrap();
    let acknowledged = journey::next(&admin.io).await;
    assert!(matches!(acknowledged, Ok(Received::Acknowledged(_))), "the first round started: {acknowledged:?}");
    assert!(journey::reports_nothing(&admin.io, QUIET).await, "none started twice: no second acknowledgement");
}

// org-io learns the acknowledgements only from org-node's list: the body
// that reaches the recipient is the one org-node keeps, and no file of
// org-io/src but acknowledge.rs (the delivery) and retention.rs (the
// deletion, which reads the list's verification time and count) asks
// org-node for, or deletes, a kept acknowledgement, while neither decodes
// one and acknowledge.rs writes no file itself.
// verifies: LLR-sn5ded
#[tokio::test(flavor = "multi_thread")]
async fn org_io_learns_and_removes_acknowledgements_only_through_org_node() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "ack-through-org-node";
    let admin = journey::admin(tag).await;
    let (admin_at, mut admin_gets, _admin_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused)]).await;
    let bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, admin_at).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let kept = journey::kept_acknowledgement_bodies(&bob);
    let (_, body) = journey::next_arrival(&mut admin_gets, NET).await.unwrap();
    assert_eq!(body, kept[0].1, "org-node's encoded acknowledgement, unchanged");

    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = vec![];
    let mut walk = vec![src.clone()];
    while let Some(dir) = walk.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk.push(path);
                continue;
            }
            let relative = path.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
            let text = std::fs::read_to_string(&path).unwrap();
            for (index, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                let lists = code.contains(".kept_acknowledgements(");
                let deletes = code.contains(".acknowledgement_accepted(");
                if (lists && relative != "acknowledge.rs" && relative != "retention.rs") || (deletes && relative != "acknowledge.rs") {
                    found.push(format!("{relative}:{}: {}", index + 1, line.trim()));
                }
                let decodes = ["decode_message", "WireMessage", "revocation::", "std::fs::", "fs::write"].iter().any(|name| code.contains(name));
                if decodes && (relative == "acknowledge.rs" || relative == "retention.rs") {
                    found.push(format!("{relative}:{}: {}", index + 1, line.trim()));
                }
            }
        }
    }
    assert!(found.is_empty(), "acknowledgements reached around org-node:\n{}", found.join("\n"));
}

// The acknowledgement comes from the removed Persona's own Device key, over
// a bind with no relay: with the handle bound for the in-process relay, the
// Persona's endpoint is closed and the Admin's receiving handle — a
// Loopback endpoint the relay cannot reach — is dialled directly; its real
// receive step accepts the acknowledgement, which it does only from the
// removed Device's own identity.
// verifies: LLR-spckm6
#[tokio::test(flavor = "multi_thread")]
async fn acknowledgements_go_from_the_removed_personas_own_identity_over_a_direct_bind() {
    let (_guard, _keychain) = fresh_keychain();
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server().await.unwrap();
    let plan = BindPlan::Relay(relay_map, iroh::address_lookup::MemoryLookup::new());
    let tag = "ack-direct";
    let admin = journey::admin(tag).await;
    // Admitted and sent its notice over Loopback; then the handle binds for
    // the relay before the verification that removes it.
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    journey::revoked(&admin, &bob, &[]).await;
    bob.io = bob.io.with_bind_plan_for_test(plan);
    bob.io.start_receiving().await.unwrap();
    let persona_id = bob.io.personas().await[0].persona_id.clone();
    assert!(bob.io.bound_endpoint_for_test(&persona_id).await.is_some(), "bound with the relay");

    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let acknowledged = journey::next(&admin.io).await;
    assert!(matches!(acknowledged, Ok(Received::Acknowledged(_))), "reached over a direct dial, from the removed Persona's own identity: {acknowledged:?}");
    assert!(bob.io.bound_endpoint_for_test(&persona_id).await.is_none(), "its relayed endpoint closed");
}

/// How many of `arrivals` take a body within five seconds each.
async fn ack_arrived(arrivals: &mut [journey::Arrivals]) -> usize {
    let mut arrived = 0;
    for recipient_arrivals in arrivals.iter_mut() {
        if let Ok(Some(_)) = tokio::time::timeout(Duration::from_secs(5), recipient_arrivals.recv()).await {
            arrived += 1;
        }
    }
    arrived
}

// Abnormal: the direct bind fails (the
// keychain locked when a reopened handle resumes the delivery). The round's
// outcome is `NotDelivered` naming the sender's endpoint for every
// recipient (the record listed three other Devices, so two are chosen),
// nothing is sent, and the next round, the keychain answering again, binds
// again and reaches both.
// verifies: LLR-spckm6
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_direct_bind_is_every_recipients_outcome_and_the_next_round_binds_again() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 17_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-bind-fails";
    let admin = journey::admin(tag).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    let carol = journey::member(&admin, tag, "c", "carol", &[&bob]).await;
    let dave = journey::member(&admin, tag, "d", "dave", &[&bob, &carol]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    let (mut arrivals, mut ends) = (vec![], vec![]);
    for (io, device_key) in [(&admin.io, admin.device_key), (&carol.io, carol.device_key), (&dave.io, dave.device_key)] {
        let (addresses, recipient_arrivals, endpoint) = journey::answering_acknowledgements(device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
        journey::keep_addresses(&bob.io, admin.org, io, device_key, addresses[..1].to_vec()).await;
        arrivals.push(recipient_arrivals);
        ends.push(endpoint);
    }
    journey::revoked(&admin, &bob, &[&carol, &dave]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    assert_eq!(ack_arrived(&mut arrivals).await, 2, "the round at T reaches both recipients");
    ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    drop(bob);

    clock.set(verified_at + 90);
    let reopened = org_io::OrgIo::open_with_clock_for_test(&store_dir(tag, "b"), TransportMode::Loopback, clock.clone()).unwrap();
    keychain.refuse_as(Some(Refusal::Locked));
    let started = reopened.start_receiving().await;
    let failed_bind = |outcome: &AcknowledgementOutcome| {
        matches!(outcome, AcknowledgementOutcome::NotDelivered { reason } if reason.to_string().contains("sender's endpoint") && reason.to_string().contains("direct bind failed"))
    };
    let report = ack_report_when(&reopened, admin.org, NET, |report| {
        report.next_round.is_some() && report.recipients.iter().all(|(_, outcome)| failed_bind(outcome))
    })
    .await;
    keychain.refuse_as(None);
    started.unwrap();
    assert_eq!(report.recipients.len(), 2, "every recipient's outcome: {report:?}");
    assert_eq!(ack_arrived(&mut arrivals).await, 0, "nothing sent without the endpoint");

    clock.set(report.next_round.unwrap());
    reopened.wake_deliveries_for_test();
    assert_eq!(ack_arrived(&mut arrivals).await, 2, "the next round binds again and reaches both");
}

/// A stand-in recipient (`Refused` under the contract test
/// `the_scripted_recipients_verdicts_are_the_real_receive_steps`; a verdict
/// a real receive step never writes for an acknowledgement, as fault
/// injection): an endpoint of `device` that takes one acknowledgement,
/// answers it `verdict` and closes: a later dial of the same identity cannot
/// reach it over a path the dialler's endpoint remembers. A body before it
/// that is no acknowledgement (the answer to the request that kept these
/// addresses) is answered `Accepted`. Its addresses and arrivals.
async fn answering_once(device_key: org_node::DevicePublicKey, verdict: Verdict) -> (Vec<String>, journey::Arrivals) {
    let endpoint = Arc::new(OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await.unwrap());
    let addresses = journey::kept_addresses(&endpoint);
    let (forward, arrivals) = tokio::sync::mpsc::unbounded_channel();
    let mut taken = journey::each_inbound(endpoint.clone());
    tokio::spawn(async move {
        while let Some((sender, body, reply)) = taken.recv().await {
            if !journey::is_acknowledgement(&body) {
                tokio::spawn(reply.answer(Verdict::Accepted));
                continue;
            }
            // The test may have stopped reading; a body no one reads is dropped.
            let _ = forward.send((sender, body));
            reply.answer(verdict).await;
            break;
        }
        endpoint.close().await;
    });
    (addresses, arrivals)
}

// In a round, the recipient's kept addresses are dialled one at a time in
// the order kept: a dead one (iroh gives up its dial after about 30 s),
// one that refuses, one that accepts — and the fourth is never dialled, nor
// the accepting one twice. Every address holds the recipient's identity,
// so the refusing endpoint closes once it has answered: the dialler's iroh
// endpoint remembers the paths of one identity across dials.
// verifies: LLR-u2j6u8
#[tokio::test(flavor = "multi_thread")]
async fn a_round_tries_each_recipients_addresses_in_order_until_accepted() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "ack-address-order";
    let admin = journey::admin(tag).await;
    let dead = journey::dead_addresses(admin.device_key).await;
    let (refusing, mut refusing_gets) = answering_once(admin.device_key, Verdict::Refused).await;
    let (accepting, mut accepting_gets, _accepting_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Accepted)]).await;
    let (never, mut never_gets, _never_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![]).await;
    let addresses = [&dead[..1], &refusing[..1], &accepting[..1], &never[..1]].concat();
    let (_admin, bob) = ack_about_to_be_removed_with(admin, tag, addresses).await;

    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    assert!(journey::next_arrival(&mut refusing_gets, NET * 2).await.is_some(), "the refusing address, after the dead one");
    assert!(journey::next_arrival(&mut accepting_gets, NET).await.is_some(), "the accepting address, third");
    assert!(journey::next_arrival(&mut never_gets, QUIET).await.is_none(), "nothing after the first Accepted");
    assert!(journey::next_arrival(&mut accepting_gets, QUIET).await.is_none(), "each address once in the round");
}

/// Member `b` of `admin`'s Organisation (party `b` of `tag`), admitted over
/// the transport, keeping `addresses` for the Admin's Device (taken by its
/// receive step) and then revoked: its next verification removes it.
async fn ack_about_to_be_removed_with(admin: journey::Admin, tag: &str, addresses: Vec<String>) -> (journey::Admin, journey::Joining) {
    let bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, addresses).await;
    journey::revoked(&admin, &bob, &[]).await;
    (admin, bob)
}

/// Poll `io`'s report for `org` until `ready` holds of it, within `limit`.
async fn ack_report_when(
    io: &org_io::OrgIo,
    org: org_node::OrgId,
    limit: Duration,
    ready: impl Fn(&AcknowledgementReport) -> bool,
) -> AcknowledgementReport {
    tokio::time::timeout(limit, async {
        loop {
            if let Some(report) = io.acknowledgement_report(org).await.filter(|report| ready(report)) {
                return report;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("the report came to hold")
}

/// A stand-in recipient (its `Accepted` under the contract test
/// `the_scripted_recipients_verdicts_are_the_real_receive_steps`): an
/// endpoint of `device` accepting every body, sending with each
/// acknowledgement's arrival, before its verdict, how many store writes had
/// landed at `watched`'s file in `dir` by then. A real receive step shows no
/// arrival before its verdict, which is the moment this order is read at.
async fn ack_counting(
    device_key: org_node::DevicePublicKey,
    dir: std::path::PathBuf,
    watched: org_node::DevicePublicKey,
) -> (Vec<String>, tokio::sync::mpsc::UnboundedReceiver<usize>, Arc<OrgEndpoint>) {
    let endpoint = Arc::new(OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await.unwrap());
    let addresses = journey::kept_addresses(&endpoint);
    let (forward, counts) = tokio::sync::mpsc::unbounded_channel();
    let mut taken = journey::each_inbound(endpoint.clone());
    tokio::spawn(async move {
        while let Some((_, body, reply)) = taken.recv().await {
            if journey::is_acknowledgement(&body) {
                // The test may have stopped reading; a body no one reads is dropped.
                let _ = forward.send(store_writes_to(&dir, &watched).len());
            }
            tokio::spawn(reply.answer(Verdict::Accepted));
        }
    });
    (addresses, counts, endpoint)
}

// Normal, over real recipients (the record listed three other Devices, so
// two are chosen): each chosen recipient's receiving handle, at its own
// addresses and holding the removing epoch, accepts its acknowledgement;
// org-node deletes each one and its sealed store is written, one write per
// acceptance; with none left the Persona is deleted at once.
// verifies: LLR-6ux6yw, LLR-n649th
#[tokio::test(flavor = "multi_thread")]
async fn accepted_acknowledgements_are_deleted_and_written_until_none_is_left() {
    let (_guard, keychain) = fresh_keychain();
    let tag = "ack-accepted-real";
    let admin = journey::admin(tag).await;
    let bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    let carol = journey::member(&admin, tag, "c", "carol", &[&bob]).await;
    let dave = journey::member(&admin, tag, "d", "dave", &[&bob, &carol]).await;
    for (io, device_key) in [(&admin.io, admin.device_key), (&carol.io, carol.device_key), (&dave.io, dave.device_key)] {
        let addresses = journey::addresses_of(io).await;
        journey::keep_addresses(&bob.io, admin.org, io, device_key, addresses).await;
    }
    journey::revoked(&admin, &bob, &[&carol, &dave]).await;
    for other in [&carol, &dave] {
        assert!(matches!(other.io.verify_all().await.as_slice(), [(_, Verified::Adopted { .. })]), "the removing epoch adopted");
    }
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let removal_written = store_writes_to(&bob.dir, &bob.device_key).len();

    let gone = ack_report_gone(&bob.io, admin.org, NET * 2);
    let ((((), at_dave), at_carol), at_admin) =
        journey::while_receiving(&admin.io, journey::while_receiving(&carol.io, journey::while_receiving(&dave.io, gone))).await;
    let reported: Vec<_> = at_admin.iter().chain(&at_carol).chain(&at_dave).collect();
    assert_eq!(reported.len(), 2, "two recipients chosen: {reported:?}");
    assert!(reported.iter().all(|received| matches!(received, Ok(Received::Acknowledged(_)))), "each accepted: {reported:?}");
    assert_eq!(store_writes_to(&bob.dir, &bob.device_key).len(), removal_written + 2, "one write per acceptance");
    assert!(!file_of(&bob.dir, &bob.device_key).exists() && !item_held(&keychain, &bob.device_key), "none left: the Persona deleted at once");
}

// Two recipients (the record listed three other Devices, so two are
// chosen), each a stand-in that reads the store writes when the
// acknowledgement arrives, before its verdict: the first recipient's
// `Accepted` has org-node delete its acknowledgement and the sealed store
// written before the second is dialled; after both, none is kept.
// verifies: LLR-6ux6yw
#[tokio::test(flavor = "multi_thread")]
async fn an_accepted_acknowledgement_is_deleted_by_org_node_and_written_before_the_next_recipient() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "ack-accepted";
    let admin = journey::admin(tag).await;
    let bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    let carol = journey::member(&admin, tag, "c", "carol", &[&bob]).await;
    let dave = journey::member(&admin, tag, "d", "dave", &[&bob, &carol]).await;
    let bob_dir = store_dir(tag, "b");
    let mut counts = vec![];
    let mut ends = vec![];
    for (io, device_key) in [(&admin.io, admin.device_key), (&carol.io, carol.device_key), (&dave.io, dave.device_key)] {
        let (addresses, arrived, endpoint) = ack_counting(device_key, bob_dir.clone(), bob.device_key).await;
        journey::keep_addresses(&bob.io, admin.org, io, device_key, addresses).await;
        counts.push(arrived);
        ends.push(endpoint);
    }
    journey::revoked(&admin, &bob, &[&carol, &dave]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let removal_written = store_writes_to(&bob_dir, &bob.device_key).len();

    let mut seen = vec![];
    for arrived in counts.iter_mut() {
        if let Ok(Some(count)) = tokio::time::timeout(Duration::from_secs(5), arrived.recv()).await {
            seen.push(count);
        }
    }
    seen.sort();
    assert_eq!(seen.len(), 2, "two recipients chosen: {seen:?}");
    assert!(seen[0] <= removal_written && seen[1] == seen[0] + 1, "the first acceptance written before the second dial: {seen:?}");
    // None left, the Persona is deleted at once (LLR-n649th): the
    // report is gone with it.
    ack_report_gone(&bob.io, admin.org, NET).await;
    assert!(bob.io.acknowledgement_report(admin.org).await.is_none(), "none kept: the removal is gone with the Persona");
    assert!(!store_and_removed_files(&bob_dir, &bob.device_key).0.exists(), "and its store file");
    assert_eq!(store_writes_to(&bob_dir, &bob.device_key).len(), seen[1] + 1, "the second acceptance written too");
}

// Abnormal: a refusal, and then no address that answers at all (the
// refusing endpoint closed; iroh gives up its dial after about 30 s), leave
// the acknowledgement kept with its recipient and addresses, and write
// nothing.
// verifies: LLR-xwz3nu
#[tokio::test(flavor = "multi_thread")]
async fn any_other_answer_or_not_delivered_keeps_the_acknowledgement() {
    let (_guard, _keychain) = fresh_keychain();
    let verified_at = 4_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-kept";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, refusing_end) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused)]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.unwrap();
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    assert_eq!(report.recipients, vec![(admin.device_key, AcknowledgementOutcome::Refused)]);
    let writes = store_writes_to(&store_dir(tag, "b"), &bob.device_key).len();
    let kept = journey::kept_acknowledgement_bodies(&bob);
    assert_eq!(kept.len(), 1, "kept after a refusal");

    refusing_end.close().await;
    clock.set(verified_at + 60);
    bob.io.wake_deliveries_for_test();
    let report = ack_report_when(&bob.io, admin.org, NET * 2, |report| {
        report.recipients.iter().any(|(_, outcome)| matches!(outcome, AcknowledgementOutcome::NotDelivered { .. }))
    })
    .await;
    assert_eq!(report.recipients.len(), 1, "{report:?}");
    assert_eq!(journey::kept_acknowledgement_bodies(&bob), kept, "kept when nothing answers");
    assert_eq!(store_writes_to(&store_dir(tag, "b"), &bob.device_key).len(), writes, "nothing written");
}

// Abnormal, fault-injected (a real receive step never writes these for an
// acknowledgement): the Admin's two kept addresses answer `AlreadyReceived`
// and `InviteUsedByAnotherDevice`. The round moves on from the first to the
// second (a verdict byte other than `Accepted`), and after both the
// acknowledgement stays kept, nothing written or deleted, the recipient
// reported `Refused`.
// verifies: LLR-xwz3nu, LLR-u2j6u8
#[tokio::test(flavor = "multi_thread")]
async fn already_received_or_invite_used_keeps_the_acknowledgement() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 20_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-other-verdicts";
    let admin = journey::admin(tag).await;
    // The first closes once it has answered: the dialler's iroh endpoint
    // remembers the paths of one identity across dials.
    let (already, mut already_gets) = answering_once(admin.device_key, Verdict::AlreadyReceived).await;
    let (used, mut used_gets, _used_endpoint) =
        journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::InviteUsedByAnotherDevice); 4]).await;
    let bob = ack_removed_at(&admin, tag, &clock, [&already[..1], &used[..1]].concat()).await;
    let writes = store_writes_to(&bob.dir, &bob.device_key).len();
    journey::next_arrival(&mut already_gets, NET).await.expect("the first address, answered AlreadyReceived");
    journey::next_arrival(&mut used_gets, NET).await.expect("the round moved on to the second");
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    assert_eq!(report.recipients, vec![(admin.device_key, AcknowledgementOutcome::Refused)]);
    assert_eq!(journey::kept_acknowledgement_bodies(&bob).len(), 1, "kept");
    assert_eq!(store_writes_to(&bob.dir, &bob.device_key).len(), writes, "nothing written");
    assert!(file_of(&bob.dir, &bob.device_key).exists() && item_held(&keychain, &bob.device_key), "nothing deleted");
}

// Abnormal (Loopback): the only address kept for the Admin holds another
// Device's identity, so no address may be dialled: the round reports the
// recipient `NotDelivered` for no address, and nothing reaches the other
// Device.
// verifies: LLR-u2j6u8
#[tokio::test(flavor = "multi_thread")]
async fn a_recipient_with_no_address_of_its_own_is_not_delivered_for_no_address() {
    let (_guard, _keychain) = fresh_keychain();
    let verified_at = 21_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-no-address";
    let admin = journey::admin(tag).await;
    let stranger = keep_device_seed([0x76; 32]);
    let (stranger_at, mut stranger_gets, _stranger_endpoint) =
        journey::answering_acknowledgements(stranger, BindPlan::Loopback, vec![]).await;
    let bob = ack_removed_at(&admin, tag, &clock, stranger_at[..1].to_vec()).await;
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    assert!(
        matches!(report.recipients.as_slice(), [(recipient, AcknowledgementOutcome::NotDelivered { reason })]
            if *recipient == admin.device_key && matches!(reason.get(), SendError::NoAddress)),
        "{report:?}"
    );
    assert!(journey::next_arrival(&mut stranger_gets, QUIET).await.is_none(), "another Device's address is not dialled");
}

// Boundary (Networked: both handles bound for the in-process relay and
// address lookup, the Admin's receiving endpoint online and in the lookup):
// no address is kept for the Admin, so the round makes one dial by the
// Admin's key alone, from an endpoint of Bob's identity bound with the relay
// and the lookup. The Admin's real receive step accepts the acknowledgement,
// and with none left Bob's Persona is deleted at once.
// verifies: LLR-spckm6, LLR-u2j6u8
#[tokio::test(flavor = "multi_thread")]
async fn networked_a_recipient_with_no_kept_address_is_dialled_by_its_key() {
    acknowledged_by_key_with_kept("ack-networked-by-key", vec![]).await;
}

// Abnormal (Networked, as above): the only address kept for the Admin does
// not parse, so it names no socket; the recipient is dialled by its key the
// same way, and accepts.
// verifies: LLR-spckm6, LLR-u2j6u8
#[tokio::test(flavor = "multi_thread")]
async fn networked_a_recipient_whose_only_kept_address_does_not_parse_is_dialled_by_its_key() {
    acknowledged_by_key_with_kept("ack-networked-unparsed", vec!["not an endpoint address".into()]).await;
}

// Abnormal (Networked, as above, with the handle's clock): the endpoint
// bound for the dial by key cannot be bound. At T the Admin is not yet in the
// lookup, so the dial by key fails for a transport reason (that the direct
// endpoint stays bound is
// `networked_after_a_failed_relay_bind_the_other_recipients_are_dialled_as_before`'s
// to show). At T+1 min the keychain is locked: the relay bind
// fails, and that is the recipient's outcome, naming the sender's endpoint.
// At T+5 min, the keychain answering and the Admin in the lookup, the round
// binds again and the Admin accepts.
// verifies: LLR-spckm6
#[tokio::test(flavor = "multi_thread")]
async fn networked_a_failed_relay_bind_is_the_recipients_outcome_and_the_next_round_binds_again() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 21_500_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-networked-relay-bind-fails";
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server().await.unwrap();
    let lookup = MemoryLookup::new();
    let plan = BindPlan::Relay(relay_map, lookup.clone());
    let mut admin = journey::admin(tag).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, vec![]).await;
    journey::revoked(&admin, &bob, &[]).await;
    admin.io = admin.io.with_bind_plan_for_test(plan.clone());
    let receiver = relayed_receiver(&admin.io).await;
    bob.io = bob.io.with_bind_plan_for_test(plan);

    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.next_round == Some(verified_at + 60)).await;
    assert!(
        matches!(report.recipients.as_slice(), [(_, AcknowledgementOutcome::NotDelivered { reason })] if matches!(reason.get(), SendError::Transport(_))),
        "not in the lookup yet: {report:?}"
    );

    keychain.refuse_as(Some(Refusal::Locked));
    clock.set(verified_at + 60);
    bob.io.wake_deliveries_for_test();
    let relay_bind_failed = |outcome: &AcknowledgementOutcome| {
        matches!(outcome, AcknowledgementOutcome::NotDelivered { reason }
            if matches!(reason.get(), SendError::SenderEndpoint(_)) && reason.to_string().contains("relay bind failed"))
    };
    let report = ack_report_when(&bob.io, admin.org, NET, |report| {
        report.next_round == Some(verified_at + 300) && report.recipients.iter().all(|(_, outcome)| relay_bind_failed(outcome))
    })
    .await;
    keychain.refuse_as(None);
    assert_eq!(report.recipients.len(), 1, "{report:?}");
    assert!(journey::reports_nothing(&admin.io, QUIET).await, "nothing sent without the endpoint");

    lookup.add_endpoint_info(receiver.inner().addr());
    clock.set(verified_at + 300);
    bob.io.wake_deliveries_for_test();
    let acknowledged = journey::next(&admin.io).await;
    assert!(matches!(acknowledged, Ok(Received::Acknowledged(_))), "the next round binds again: {acknowledged:?}");
    assert_deleted_at_once(&keychain, &admin, &bob).await;
}

// Abnormal (Networked, both handles bound for the in-process relay, the
// Admin's receiving handle online and in the lookup): the one address kept
// for the Admin is dialled and refuses, so an address was dialled and no
// dial by the Admin's key alone follows. The recipient's outcome is that
// dial's `Refused`, and the Admin's receiving handle takes nothing.
// verifies: LLR-u2j6u8
#[tokio::test(flavor = "multi_thread")]
async fn networked_a_recipient_whose_kept_addresses_all_fail_is_not_dialled_by_its_key() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "ack-networked-no-key-dial";
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server().await.unwrap();
    let lookup = MemoryLookup::new();
    let plan = BindPlan::Relay(relay_map, lookup.clone());
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets) = answering_once(admin.device_key, Verdict::Refused).await;
    let (mut admin, mut bob) = ack_about_to_be_removed_with(admin, tag, refusing[..1].to_vec()).await;
    admin.io = admin.io.with_bind_plan_for_test(plan.clone());
    let receiver = relayed_receiver(&admin.io).await;
    lookup.add_endpoint_info(receiver.inner().addr());
    bob.io = bob.io.with_bind_plan_for_test(plan);

    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    assert!(journey::next_arrival(&mut refusing_gets, NET).await.is_some(), "the kept address is dialled");
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    assert!(matches!(report.recipients.as_slice(), [(_, AcknowledgementOutcome::Refused)]), "the kept address's verdict: {report:?}");
    assert!(journey::reports_nothing(&admin.io, QUIET).await, "no dial by the Admin's key alone");
}

// Abnormal (Networked, both handles bound for the in-process relay, with
// the handle's clock; the record lists three other Devices, so two are
// chosen, the Admin, which delivered the notice, first): no address is kept
// for the Admin, and each other Device is a stand-in kept at one direct
// address, refusing. At T both rounds' dials are made. At T+1 min the
// keychain is locked: the relay bind for the dial by the Admin's key fails,
// which is the Admin's outcome, and the other recipient is still dialled
// from the direct endpoint bound at T, which is not bound again.
// verifies: LLR-spckm6
#[tokio::test(flavor = "multi_thread")]
async fn networked_after_a_failed_relay_bind_the_other_recipients_are_dialled_as_before() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 21_600_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-networked-relay-fails-others-dialled";
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server().await.unwrap();
    let plan = BindPlan::Relay(relay_map, MemoryLookup::new());
    let admin = journey::admin(tag).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    let carol = journey::member(&admin, tag, "c", "carol", &[&bob]).await;
    let dave = journey::member(&admin, tag, "d", "dave", &[&bob, &carol]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, vec![]).await;
    let (mut arrivals, mut ends) = (vec![], vec![]);
    for (io, device_key) in [(&carol.io, carol.device_key), (&dave.io, dave.device_key)] {
        let (addresses, recipient_arrivals, endpoint) = journey::answering_acknowledgements(device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
        journey::keep_addresses(&bob.io, admin.org, io, device_key, addresses[..1].to_vec()).await;
        arrivals.push(recipient_arrivals);
        ends.push(endpoint);
    }
    journey::revoked(&admin, &bob, &[&carol, &dave]).await;
    bob.io = bob.io.with_bind_plan_for_test(plan);

    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    assert_eq!(ack_arrived(&mut arrivals).await, 1, "the round at T dials the other recipient");
    ack_report_when(&bob.io, admin.org, NET, |report| report.next_round == Some(verified_at + 60)).await;

    keychain.refuse_as(Some(Refusal::Locked));
    clock.set(verified_at + 60);
    bob.io.wake_deliveries_for_test();
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.next_round == Some(verified_at + 300)).await;
    let arrived = ack_arrived(&mut arrivals).await;
    keychain.refuse_as(None);
    assert!(
        matches!(report.recipients.iter().find(|(recipient, _)| *recipient == admin.device_key), Some((_, AcknowledgementOutcome::NotDelivered { reason }))
            if matches!(reason.get(), SendError::SenderEndpoint(_)) && reason.to_string().contains("relay bind failed")),
        "the Admin's outcome is the failed relay bind: {report:?}"
    );
    assert!(
        matches!(report.recipients.iter().find(|(recipient, _)| *recipient != admin.device_key), Some((_, AcknowledgementOutcome::Refused))),
        "the other recipient's outcome is its dial's: {report:?}"
    );
    assert_eq!(arrived, 1, "the other recipient is dialled as before, from the direct endpoint");
}

/// `io`'s first Persona's endpoint, receiving and online at the relay `io`
/// is bound for.
async fn relayed_receiver(io: &OrgIo) -> Arc<OrgEndpoint> {
    io.start_receiving().await.unwrap();
    let persona_id = io.personas().await[0].persona_id.clone();
    let receiver = io.endpoint_for_test(&persona_id).await.unwrap();
    tokio::time::timeout(NET, receiver.online()).await.unwrap();
    receiver
}

/// Bob removed, keeping `addresses` for the Admin, with both handles bound
/// for an in-process relay and a shared lookup holding the Admin's
/// receiving endpoint; the Admin's receive step acknowledges Bob's
/// acknowledgement, and Bob's Persona is deleted at once.
async fn acknowledged_by_key_with_kept(tag: &str, addresses: Vec<String>) {
    let (_guard, keychain) = fresh_keychain();
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server().await.unwrap();
    let lookup = MemoryLookup::new();
    let plan = BindPlan::Relay(relay_map, lookup.clone());
    let mut admin = journey::admin(tag).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, addresses).await;
    journey::revoked(&admin, &bob, &[]).await;
    admin.io = admin.io.with_bind_plan_for_test(plan.clone());
    let receiver = relayed_receiver(&admin.io).await;
    lookup.add_endpoint_info(receiver.inner().addr());
    bob.io = bob.io.with_bind_plan_for_test(plan);

    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let acknowledged = tokio::time::timeout(NET, admin.io.next_received()).await;
    let report = bob.io.acknowledgement_report(admin.org).await;
    assert!(matches!(acknowledged, Ok(Ok(Received::Acknowledged(_)))), "dialled by key and accepted: {acknowledged:?}; Bob's report: {report:?}");
    assert_deleted_at_once(&keychain, &admin, &bob).await;
}

/// Bob (party `b` of `tag`) admitted over the transport and reading
/// `clock`, keeping `addresses` for the Admin's Device, revoked, and removed
/// by its verification at the clock's time; the round at that time runs.
async fn ack_removed_at(admin: &journey::Admin, tag: &str, clock: &Arc<TestClock>, addresses: Vec<String>) -> journey::Joining {
    let mut bob = journey::member(admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, addresses).await;
    journey::revoked(admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    bob
}

// Abnormal: the handle's clock reads before the
// Unix epoch when a round is due. That reading is never time 0: the round
// judges no deletion on it, sends nothing and deletes nothing, the condition
// is that round's failure in the report, and the round is tried again at
// the next due time (here on a wake, the clock reading again).
// verifies: LLR-bv6wkz, LLR-rav7dr
#[tokio::test(flavor = "multi_thread")]
async fn a_clock_before_the_epoch_is_the_rounds_failure_and_nothing_is_sent_or_deleted() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 5_500_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-clock-before-epoch";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let bob = ack_removed_at(&admin, tag, &clock, refusing[..1].to_vec()).await;
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T");
    ack_report_when(&bob.io, admin.org, NET, |report| report.next_round == Some(verified_at + 60)).await;
    let (persona_file, _) = store_and_removed_files(&bob.dir, &bob.device_key);

    clock.set(verified_at + 60);
    clock.set_before_epoch(true);
    bob.io.wake_deliveries_for_test();
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.last_round_failure.is_some()).await;
    let failure = report.last_round_failure.clone().unwrap().to_string();
    assert!(failure.contains("before the Unix epoch"), "the clock's condition is named: {failure}");
    assert_eq!(report.recipients, vec![(admin.device_key, AcknowledgementOutcome::Refused)], "still kept");
    assert_eq!(report.next_round, None, "with no next round");
    assert_eq!(report.last_deletion_failure, None, "no deletion judged");
    assert!(journey::next_arrival(&mut refusing_gets, QUIET).await.is_none(), "the round sent nothing");
    assert!(persona_file.exists() && item_held(&keychain, &bob.device_key), "nothing deleted");

    clock.set_before_epoch(false);
    bob.io.wake_deliveries_for_test();
    assert!(journey::next_arrival(&mut refusing_gets, NET).await.is_some(), "the round runs once the clock reads again");
    let report =
        ack_report_when(&bob.io, admin.org, NET, |report| report.next_round == Some(verified_at + 300) && report.last_round_failure.is_none()).await;
    assert_eq!(report.recipients, vec![(admin.device_key, AcknowledgementOutcome::Refused)]);
}

// Abnormal: `open` with the clock reading before the
// Unix epoch judges no deletion on that reading — a removal past its
// deadline by the true time is not deleted, with its file and item — and
// reports the condition; once receiving starts, the resumed delivery's round
// reports it too and sends nothing.
// verifies: LLR-upqj5g, LLR-5rxqr8
#[tokio::test(flavor = "multi_thread")]
async fn open_judges_no_deletion_on_a_clock_before_the_epoch_and_reports_it() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 5_600_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-open-clock-before-epoch";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let bob = ack_removed_at(&admin, tag, &clock, refusing[..1].to_vec()).await;
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T");
    let (bob_device, bob_dir) = (bob.device_key, bob.dir.clone());
    drop(bob);
    let (persona_file, removed_file) = store_and_removed_files(&bob_dir, &bob_device);

    clock.set(verified_at + 604_800);
    clock.set_before_epoch(true);
    let io = org_io::OrgIo::open_with_clock_for_test(&bob_dir, TransportMode::Loopback, clock.clone()).unwrap();
    assert!(persona_file.exists() && !removed_file.exists() && item_held(&keychain, &bob_device), "nothing deleted at open");
    let report = io.acknowledgement_report(admin.org).await.expect("the removal is held and reported");
    let failure = report.last_deletion_failure.clone().map(|failure| failure.to_string()).unwrap_or_default();
    assert!(failure.contains("before the Unix epoch"), "the clock's condition is reported: {report:?}");
    assert_eq!(report.recipients, vec![(admin.device_key, AcknowledgementOutcome::NotYetSent)], "still kept");

    io.start_receiving().await.unwrap();
    let report = ack_report_when(&io, admin.org, NET, |report| report.last_round_failure.is_some()).await;
    assert!(report.last_round_failure.unwrap().to_string().contains("before the Unix epoch"), "the resumed round reports it too");
    assert!(journey::next_arrival(&mut refusing_gets, QUIET).await.is_none(), "and sends nothing");
    assert!(persona_file.exists() && item_held(&keychain, &bob_device), "nothing deleted");
}

// The schedule: due at T, T+1 min, T+5 min, T+30 min, then hourly, until
// T+7 days; a waking past several due times runs one round and waits for
// the next due after now. The schedule is seen through the handle alone,
// its rounds and the report's next round.
// verifies: LLR-bv6wkz
#[tokio::test(flavor = "multi_thread")]
async fn rounds_are_due_at_t_one_five_thirty_minutes_then_hourly_and_missed_ones_are_not_caught_up() {
    let verified_at = 1_000_000u64;
    let (_guard, _keychain) = fresh_keychain();
    let clock = TestClock::at(verified_at);
    let tag = "ack-schedule";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 16]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    assert!(journey::next_arrival(&mut refusing_gets, NET).await.is_some(), "the round at T");

    clock.set(verified_at + 59);
    bob.io.wake_deliveries_for_test();
    assert!(journey::next_arrival(&mut refusing_gets, QUIET).await.is_none(), "not due before T+1 min");
    clock.set(verified_at + 60);
    bob.io.wake_deliveries_for_test();
    assert!(journey::next_arrival(&mut refusing_gets, NET).await.is_some(), "due at T+1 min");
    ack_report_when(&bob.io, admin.org, NET, |report| report.next_round == Some(verified_at + 300)).await;
    clock.set(verified_at + 7_200);
    bob.io.wake_deliveries_for_test();
    assert!(journey::next_arrival(&mut refusing_gets, NET).await.is_some(), "one round for the missed ones");
    assert!(journey::next_arrival(&mut refusing_gets, QUIET).await.is_none(), "and only one");
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.next_round == Some(verified_at + 9_000)).await;
    assert_eq!(report.next_round, Some(verified_at + 9_000), "the next due after now");
}

// The report: each recipient still kept with its last round's outcome, the
// next round, the deadline; `None` for an Organisation no removed Persona
// holds, and before the removal.
// verifies: LLR-rav7dr
#[tokio::test(flavor = "multi_thread")]
async fn acknowledgement_report_lists_each_kept_recipients_outcome_next_round_and_deadline() {
    let (_guard, _keychain) = fresh_keychain();
    let verified_at = 2_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-report";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused)]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(bob.io.acknowledgement_report(admin.org).await.is_none(), "not removed yet");
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.unwrap();
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    let writes = store_writes_to(&bob.dir, &bob.device_key).len();
    for _ in 0..3 {
        bob.io.acknowledgement_report(admin.org).await;
    }
    assert_eq!(store_writes_to(&bob.dir, &bob.device_key).len(), writes, "the report writes nothing");
    assert!(journey::next_arrival(&mut refusing_gets, QUIET).await.is_none(), "and sends nothing");
    assert_eq!(
        report,
        AcknowledgementReport {
            org_id: admin.org,
            recipients: vec![(admin.device_key, AcknowledgementOutcome::Refused)],
            next_round: Some(verified_at + 60),
            deadline: Some(verified_at + 604_800),
            last_deletion_failure: None,
            last_round_failure: None,
        }
    );
    assert!(bob.io.acknowledgement_report(org_node::OrgId::new([0x97; 20])).await.is_none(), "an Organisation no removed Persona holds");
    assert!(admin.io.acknowledgement_report(admin.org).await.is_none(), "the Admin's own Organisation, not removed");
}

// A restart: the removed store is reopened from its file and
// `start_receiving` resumes its delivery — one round at once, then the next
// due time anchored at the kept verification time, not at the start.
// verifies: LLR-5rxqr8
#[tokio::test(flavor = "multi_thread")]
async fn start_receiving_resumes_each_kept_removal_anchored_at_its_verification_time() {
    let (_guard, _keychain) = fresh_keychain();
    let verified_at = 3_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-resume";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.unwrap();
    drop(bob);

    clock.set(verified_at + 90);
    let reopened = org_io::OrgIo::open_with_clock_for_test(&store_dir(tag, "b"), TransportMode::Loopback, clock.clone()).unwrap();
    assert!(journey::next_arrival(&mut refusing_gets, QUIET).await.is_none(), "nothing before start_receiving");
    reopened.start_receiving().await.unwrap();
    assert!(journey::next_arrival(&mut refusing_gets, NET).await.is_some(), "one round at once");
    let report = ack_report_when(&reopened, admin.org, NET, |report| report.next_round.is_some()).await;
    assert_eq!(report.next_round, Some(verified_at + 300), "anchored at T, not at the start");
    reopened.start_receiving().await.unwrap();
    assert!(journey::next_arrival(&mut refusing_gets, QUIET).await.is_none(), "none started twice");
}

// Abnormal: the data directory cannot be listed once the handle is open (a
// transient failure of the listing). The kept removal `open` found is still
// reported, and resumed when receiving starts: the handle uses the removed
// stores' keys it remembers, not a fresh listing.
// verifies: LLR-5rxqr8, LLR-rav7dr
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_removal_open_found_is_reported_and_resumed_though_the_directory_cannot_be_listed() {
    let (_guard, _keychain) = fresh_keychain();
    let verified_at = 3_500_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-resume-unlisted";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.unwrap();
    drop(bob);

    clock.set(verified_at + 90);
    let bob_dir = store_dir(tag, "b");
    let reopened = org_io::OrgIo::open_with_clock_for_test(&bob_dir, TransportMode::Loopback, clock.clone()).unwrap();
    ack_unlistable(&bob_dir, true);
    let reported = reopened.acknowledgement_report(admin.org).await;
    let started = reopened.start_receiving().await;
    let arrived = journey::next_arrival(&mut refusing_gets, NET).await;
    ack_unlistable(&bob_dir, false);
    let reported = reported.expect("the removal open found is reported while the directory cannot be listed");
    assert_eq!(reported.recipients, vec![(admin.device_key, AcknowledgementOutcome::NotYetSent)]);
    assert_eq!(reported.deadline, Some(verified_at + 604_800));
    assert!(started.is_ok(), "{started:?}");
    assert!(arrived.is_some(), "its delivery resumed: one round at once");
}

// Abnormal: a kept removal whose deadline the clock has reached is not
// resumed at start.
// verifies: LLR-5rxqr8
#[tokio::test(flavor = "multi_thread")]
async fn start_receiving_resumes_no_removal_past_its_deadline() {
    let (_guard, _keychain) = fresh_keychain();
    let verified_at = 5_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-resume-late";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.unwrap();
    drop(bob);

    clock.set(verified_at + 604_800);
    let reopened = org_io::OrgIo::open_with_clock_for_test(&store_dir(tag, "b"), TransportMode::Loopback, clock.clone()).unwrap();
    reopened.start_receiving().await.unwrap();
    assert!(journey::next_arrival(&mut refusing_gets, QUIET).await.is_none(), "no round at the deadline");
    assert!(reopened.acknowledgement_report(admin.org).await.is_none(), "deleted at open: nothing resumed");
}

// ---- the removed Persona's deletion ----------------------------------------

/// Wait, within `limit`, until `done` holds.
async fn ack_until(limit: Duration, done: impl Fn() -> bool) {
    tokio::time::timeout(limit, async {
        while !done() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("came to hold");
}

// At T+7 days, whatever was delivered, the Persona is deleted: its file
// renamed, its keychain item deleted, the renamed file removed, its store
// dropped; the report is gone with it.
// verifies: LLR-vj2nkb
#[tokio::test(flavor = "multi_thread")]
async fn at_the_deadline_the_file_is_marked_removed_then_the_item_then_the_file_is_removed() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 4_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-deadline";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T");
    let (persona_file, removed_file) = store_and_removed_files(&store_dir(tag, "b"), &bob.device_key);
    assert!(persona_file.exists() && item_held(&keychain, &bob.device_key), "kept until the deadline");

    clock.set(verified_at + 604_800);
    bob.io.wake_deliveries_for_test();
    ack_until(NET, || !persona_file.exists() && !removed_file.exists() && !item_held(&keychain, &bob.device_key)).await;
    ack_report_gone(&bob.io, admin.org, NET).await;
    assert!(bob.io.acknowledgement_report(admin.org).await.is_none(), "dropped from the service: no removal held");
    assert!(item_held(&keychain, &admin.device_key), "no other Persona's item touched");
    assert!(journey::next_arrival(&mut refusing_gets, QUIET).await.is_none(), "no round at the deadline");
}

// The deletion's order: stopped at the
// keychain step (the keychain locked at the deadline), the store file is
// already renamed to its `removed-` name, which no Persona listing lists,
// and the renamed file is still there with the item: rename first, then the
// item, then the file. The Admin's item is untouched.
// verifies: LLR-vj2nkb
#[tokio::test(flavor = "multi_thread")]
async fn the_deletion_renames_the_file_before_the_item_and_the_item_before_the_file() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 18_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-deletion-order";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T");
    ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    let bob_dir = store_dir(tag, "b");
    let (persona_file, removed_file) = store_and_removed_files(&bob_dir, &bob.device_key);

    keychain.refuse_as(Some(Refusal::Locked));
    clock.set(verified_at + 604_800);
    bob.io.wake_deliveries_for_test();
    ack_report_when(&bob.io, admin.org, NET, |report| report.last_deletion_failure.is_some()).await;
    keychain.refuse_as(None);
    assert!(!persona_file.exists(), "the store file was renamed first");
    assert!(org_io::store_file::list_stores(&bob_dir).unwrap().is_empty(), "under a name the Persona listing does not list");
    assert!(item_held(&keychain, &bob.device_key) && removed_file.exists(), "the item step came before the file's removal");
    assert!(item_held(&keychain, &admin.device_key), "no other Persona's item touched");
}

// Boundary: the last acknowledgement, accepted by the Admin's real receive
// step, leaves none, and the Persona is deleted at once, days before its
// deadline.
// verifies: LLR-n649th, LLR-6ux6yw
#[tokio::test(flavor = "multi_thread")]
async fn no_acknowledgement_left_deletes_the_persona_at_once() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 4_500_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-none-left";
    let admin = journey::admin(tag).await;
    let admin_at = journey::addresses_of(&admin.io).await;
    let bob = ack_removed_at(&admin, tag, &clock, admin_at).await;
    let acknowledged = journey::next(&admin.io).await;
    assert!(matches!(acknowledged, Ok(Received::Acknowledged(_))), "accepted at T: {acknowledged:?}");
    // The test clock still reads T: deleted days before the deadline.
    assert_deleted_at_once(&keychain, &admin, &bob).await;
}

/// The Persona of `bob` deleted at once: its files and keychain item gone,
/// and its removal no longer held.
async fn assert_deleted_at_once(keychain: &org_io::test_support::MemoryKeychain, admin: &journey::Admin, bob: &journey::Joining) {
    let (persona_file, removed_file) = (file_of(&bob.dir, &bob.device_key), removed_file_of(&bob.dir, &bob.device_key));
    ack_until(NET, || !persona_file.exists() && !removed_file.exists() && !item_held(keychain, &bob.device_key)).await;
    ack_report_gone(&bob.io, admin.org, NET).await;
    assert!(bob.io.acknowledgement_report(admin.org).await.is_none(), "dropped from the service: no removal held");
}

/// The contract's assertions after the first round's recipient answered
/// `Refused`: the recipient reported `Refused`, the next round due at
/// T + 1 min, and the acknowledgement and the Persona kept.
async fn assert_kept_after_refusal(keychain: &org_io::test_support::MemoryKeychain, admin: &journey::Admin, bob: &journey::Joining, verified_at: u64) {
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    assert_eq!(report.recipients, vec![(admin.device_key, AcknowledgementOutcome::Refused)], "reported Refused");
    assert_eq!(report.next_round, Some(verified_at + 60), "the next round due");
    assert_eq!(journey::kept_acknowledgement_bodies(bob).len(), 1, "the acknowledgement kept");
    assert!(file_of(&bob.dir, &bob.device_key).exists() && item_held(keychain, &bob.device_key), "nothing deleted");
}

/// A second handle of the Admin's Device, opened from a copy of the Admin's
/// store files as they are now, and receiving: its record is the Admin's
/// current one, so once the Admin's next epoch removes a Member, this copy
/// still lists that Member and its real receive step refuses the Member's
/// acknowledgement.
async fn stale_copy(admin: &journey::Admin, tag: &str) -> OrgIo {
    let dir = store_dir(tag, "a-stale");
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for name in names_in(&admin.dir) {
        std::fs::copy(admin.dir.join(&name), dir.join(&name)).unwrap();
    }
    let copy = OrgIo::open(&dir, TransportMode::Loopback).unwrap();
    copy.start_receiving().await.unwrap();
    copy
}

// Contract of the scripted recipients (`journey::answering_acknowledgements`,
// `answering_once`, `ack_counting`, `ack_accepting_after`): the two verdicts
// they script that a real receive step writes for an acknowledgement act on
// the delivery as the real step's do. The same assertions run after each.
// `Refused` — scripted, and from a real receive step whose record still
// lists the removed Device (a copy of the Admin's store from before the
// removal, under the Admin's Device key) — leaves the recipient reported
// `Refused`, the next round due at T + 1 min and the acknowledgement kept.
// `Accepted` — scripted, and from the Admin's own receive step — has the
// Persona deleted at once, none left.
// verifies: LLR-xwz3nu, LLR-6ux6yw, LLR-n649th
#[tokio::test(flavor = "multi_thread")]
async fn the_scripted_recipients_verdicts_are_the_real_receive_steps() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 19_000_000u64;

    let tag = "ack-contract-refused-scripted";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) =
        journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused)]).await;
    let bob = ack_removed_at(&admin, tag, &TestClock::at(verified_at), refusing[..1].to_vec()).await;
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T");
    assert_kept_after_refusal(&keychain, &admin, &bob, verified_at).await;

    let tag = "ack-contract-refused-real";
    let admin = journey::admin(tag).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(TestClock::at(verified_at));
    let stale = stale_copy(&admin, tag).await;
    // One address: a round tries each kept address after a refusal.
    let stale_at = journey::addresses_of(&stale).await[..1].to_vec();
    journey::keep_addresses(&bob.io, admin.org, &stale, admin.device_key, stale_at).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let refused = journey::next(&stale).await;
    assert!(matches!(refused, Err(ReceiveError::Refused(_))), "the stale record still lists Bob: {refused:?}");
    assert_kept_after_refusal(&keychain, &admin, &bob, verified_at).await;

    let tag = "ack-contract-accepted-scripted";
    let admin = journey::admin(tag).await;
    let (accepting, mut accepting_gets, _accepting_endpoint) =
        journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![]).await;
    let bob = ack_removed_at(&admin, tag, &TestClock::at(verified_at), accepting[..1].to_vec()).await;
    journey::next_arrival(&mut accepting_gets, NET).await.expect("the round at T");
    assert_deleted_at_once(&keychain, &admin, &bob).await;

    let tag = "ack-contract-accepted-real";
    let admin = journey::admin(tag).await;
    let admin_at = journey::addresses_of(&admin.io).await;
    let bob = ack_removed_at(&admin, tag, &TestClock::at(verified_at), admin_at).await;
    let acknowledged = journey::next(&admin.io).await;
    assert!(matches!(acknowledged, Ok(Received::Acknowledged(_))), "{acknowledged:?}");
    assert_deleted_at_once(&keychain, &admin, &bob).await;
}

// Abnormal: the rename fails (a directory stands where the removed file
// would go). Nothing else is deleted, the Persona stays with its file and
// item, the failure is reported; an hour later, the obstacle gone, the
// deletion is made. A second Persona whose rename failed when its handle
// closed is deleted at the next open.
// verifies: LLR-3fxxpp
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_step_is_reported_and_retried_hourly_and_at_open() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 5_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-retry";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 8]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T");
    let (persona_file, removed_file) = store_and_removed_files(&store_dir(tag, "b"), &bob.device_key);
    std::fs::create_dir(&removed_file).unwrap();

    clock.set(verified_at + 604_800);
    bob.io.wake_deliveries_for_test();
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.last_deletion_failure.is_some()).await;
    assert!(report.last_deletion_failure.unwrap().to_string().contains("rename"), "the failed step is named");
    assert!(persona_file.exists() && item_held(&keychain, &bob.device_key), "nothing else deleted");
    assert!(
        bob.io.acknowledgement_report(admin.org).await.is_some_and(|report| report.recipients == vec![(admin.device_key, AcknowledgementOutcome::Refused)]),
        "the Persona stays in the service, its acknowledgement kept"
    );
    std::fs::remove_dir(&removed_file).unwrap();
    clock.set(verified_at + 604_800 + 3_599);
    bob.io.wake_deliveries_for_test();
    tokio::time::sleep(QUIET).await;
    assert!(persona_file.exists(), "not retried before the hour");
    clock.set(verified_at + 604_800 + 3_600);
    bob.io.wake_deliveries_for_test();
    ack_until(NET, || !persona_file.exists() && !removed_file.exists() && !item_held(&keychain, &bob.device_key)).await;

    let mut carol = journey::member(&admin, tag, "c", "carol", &[]).await;
    carol.io = carol.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&carol.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &carol, &[]).await;
    assert!(matches!(carol.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.expect("carol's round at her T");
    let (carol_file, carol_removed) = store_and_removed_files(&store_dir(tag, "c"), &carol.device_key);
    std::fs::create_dir(&carol_removed).unwrap();
    // Carol was removed at t + 604 800 + 3 600: her deadline is 7 days later.
    clock.set(verified_at + 2 * 604_800 + 3_600);
    carol.io.wake_deliveries_for_test();
    ack_report_when(&carol.io, admin.org, NET, |report| report.last_deletion_failure.is_some()).await;
    let carol_device = carol.device_key;
    drop(carol);
    std::fs::remove_dir(&carol_removed).unwrap();
    let reopened = org_io::OrgIo::open_with_clock_for_test(&store_dir(tag, "c"), TransportMode::Loopback, clock.clone()).unwrap();
    assert!(!carol_file.exists() && !carol_removed.exists(), "deleted at the next open");
    assert!(!item_held(&keychain, &carol_device), "its item deleted at the next open");
    assert!(reopened.acknowledgement_report(admin.org).await.is_none(), "the reopened handle holds none");
}

// Abnormal: the keychain refuses the item's deletion (locked). The file is
// already renamed, the item stays, the failure names the keychain step and
// the store is dropped; an hour later, the keychain unlocked, the retry
// deletes the item and the renamed file.
// verifies: LLR-3fxxpp
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_keychain_deletion_is_reported_and_retried_hourly() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 7_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-keychain-refused";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T");
    ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    let (persona_file, removed_file) = store_and_removed_files(&store_dir(tag, "b"), &bob.device_key);

    keychain.refuse_as(Some(Refusal::Locked));
    clock.set(verified_at + 604_800);
    bob.io.wake_deliveries_for_test();
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.last_deletion_failure.is_some()).await;
    keychain.refuse_as(None);
    assert!(report.last_deletion_failure.unwrap().to_string().contains("keychain item"), "the failed step is named");
    assert!(!persona_file.exists() && removed_file.exists(), "renamed before the item step");
    assert!(item_held(&keychain, &bob.device_key), "the item stays while the keychain refuses");
    assert!(report.recipients.is_empty(), "dropped from the service: no kept recipient is listed, only the failure");

    clock.set(verified_at + 604_800 + 3_600);
    bob.io.wake_deliveries_for_test();
    ack_until(NET, || !removed_file.exists() && !item_held(&keychain, &bob.device_key)).await;
    assert!(item_held(&keychain, &admin.device_key), "no other Persona's item touched");
    ack_report_gone(&bob.io, admin.org, NET).await;
    assert!(bob.io.acknowledgement_report(admin.org).await.is_none(), "no report once deleted");
}

// Abnormal (merge review round 1, finding-1): the keychain refuses the item's
// deletion at the deadline, so the store is dropped and only the failure is
// retried. While it is, the report's deadline is still the kept verification
// time plus 7 days, never the Unix epoch.
// verifies: LLR-rav7dr
#[tokio::test(flavor = "multi_thread")]
async fn a_deletion_retried_after_its_store_is_dropped_reports_the_kept_deadline() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 7_500_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-dropped-deadline";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T");
    ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;

    keychain.refuse_as(Some(Refusal::Locked));
    clock.set(verified_at + 604_800);
    bob.io.wake_deliveries_for_test();
    let report = ack_report_when(&bob.io, admin.org, NET, |report| report.last_deletion_failure.is_some()).await;
    keychain.refuse_as(None);
    assert!(report.recipients.is_empty(), "dropped from the service: only the failure is reported");
    assert_eq!(report.deadline, Some(verified_at + 604_800), "the kept verification time plus 7 days");
}

// Abnormal (merge review round 1, finding-1): `open` finds a removal past its
// deadline and renames its file, but the keychain refuses the item's deletion,
// so the store is dropped at open and only the failure is reported. Its
// deadline is still the kept verification time plus 7 days.
// verifies: LLR-rav7dr
#[tokio::test(flavor = "multi_thread")]
async fn a_deletion_failed_at_open_after_the_rename_reports_the_kept_deadline() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 9_500_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-open-dropped-deadline";
    let admin = journey::admin(tag).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    // Dead addresses for the Admin: Bob's acknowledgement stays kept until the deadline.
    let (dead, listening) = journey::soon_dead(admin.device_key).await;
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, dead).await;
    listening.close().await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let bob_device = bob.device_key;
    drop(bob);
    let bob_dir = store_dir(tag, "b");
    let (persona_file, removed_file) = store_and_removed_files(&bob_dir, &bob_device);

    clock.set(verified_at + 604_800);
    keychain.refuse_deletes_as(Some(Refusal::Unavailable));
    let opened = org_io::OrgIo::open_with_clock_for_test(&bob_dir, TransportMode::Loopback, clock.clone());
    keychain.refuse_deletes_as(None);
    let io = opened.unwrap();
    assert!(!persona_file.exists() && removed_file.exists() && item_held(&keychain, &bob_device), "renamed; the item refused");
    let report = io.acknowledgement_report(admin.org).await.expect("the failure is reported");
    assert!(report.recipients.is_empty(), "dropped at open: only the failure is reported");
    assert!(report.last_deletion_failure.is_some_and(|failure| failure.to_string().contains("keychain item")), "the failed step is named");
    assert_eq!(report.deadline, Some(verified_at + 604_800), "the kept verification time plus 7 days");
}

// Abnormal (merge review round 2, finding-2): `open` finds a removal whose
// deadline the clock has not reached, so it deletes nothing; the clock then
// passes the deadline before receiving starts, so `start_receiving` retries
// the deletion an hour later with no delivery round. The keychain refuses
// the item's deletion after the rename, so the store is dropped; the report
// still gives the kept verification time plus 7 days as its deadline.
// verifies: LLR-rav7dr
#[tokio::test(flavor = "multi_thread")]
async fn a_deletion_failed_on_resume_past_the_deadline_reports_the_kept_deadline() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 9_700_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-resume-dropped-deadline";
    let admin = journey::admin(tag).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    // Dead addresses for the Admin: Bob's acknowledgement stays kept until the deadline.
    let (dead, listening) = journey::soon_dead(admin.device_key).await;
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, dead).await;
    listening.close().await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let bob_device = bob.device_key;
    drop(bob);
    let bob_dir = store_dir(tag, "b");
    let (persona_file, removed_file) = store_and_removed_files(&bob_dir, &bob_device);

    clock.set(verified_at + 604_799);
    let io = org_io::OrgIo::open_with_clock_for_test(&bob_dir, TransportMode::Loopback, clock.clone()).unwrap();
    assert!(persona_file.exists(), "the deadline not reached at open: nothing deleted");

    clock.set(verified_at + 604_800);
    keychain.refuse_deletes_as(Some(Refusal::Unavailable));
    io.start_receiving().await.unwrap();
    clock.set(verified_at + 604_800 + 3_600);
    io.wake_deliveries_for_test();
    let report = ack_report_when(&io, admin.org, NET, |report| report.last_deletion_failure.is_some()).await;
    keychain.refuse_deletes_as(None);
    assert!(!persona_file.exists() && removed_file.exists() && item_held(&keychain, &bob_device), "renamed; the item refused");
    assert!(report.recipients.is_empty(), "dropped on the retry: only the failure is reported");
    assert_eq!(report.deadline, Some(verified_at + 604_800), "the kept verification time plus 7 days");
}

// Abnormal: the keychain refuses the item's deletion, and when the retry is
// due an hour later the data directory cannot be listed, so the retry cannot
// see the `removed-` file it would finish. That retry is a failure too:
// reported, the item and file kept, and tried again an hour later, which
// finishes the deletion once the listing works.
// verifies: LLR-3fxxpp
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_retry_that_cannot_list_the_data_directory_is_reported_and_retried_hourly() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 7_500_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-retry-unlisted";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T");
    ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    let bob_dir = store_dir(tag, "b");
    let (_, removed_file) = store_and_removed_files(&bob_dir, &bob.device_key);

    keychain.refuse_as(Some(Refusal::Locked));
    clock.set(verified_at + 604_800);
    bob.io.wake_deliveries_for_test();
    ack_report_when(&bob.io, admin.org, NET, |report| report.last_deletion_failure.is_some()).await;
    keychain.refuse_as(None);

    ack_unlistable(&bob_dir, true);
    clock.set(verified_at + 604_800 + 3_600);
    bob.io.wake_deliveries_for_test();
    tokio::time::sleep(QUIET).await;
    let report = bob.io.acknowledgement_report(admin.org).await;
    ack_unlistable(&bob_dir, false);
    let failure = report.and_then(|report| report.last_deletion_failure).expect("the unlisted retry is reported");
    assert!(failure.to_string().contains("store file"), "the failed step is named: {failure}");
    assert!(removed_file.exists() && item_held(&keychain, &bob.device_key), "nothing finished while unlisted");

    clock.set(verified_at + 604_800 + 7_200);
    bob.io.wake_deliveries_for_test();
    ack_until(NET, || !removed_file.exists() && !item_held(&keychain, &bob.device_key)).await;
    ack_report_gone(&bob.io, admin.org, NET).await;
}

// Abnormal: a `removed-` file whose keychain deletion `open` could not
// finish (the keychain locked) stays, and once receiving starts it is
// retried an hour later, not before.
// verifies: LLR-3fxxpp
#[tokio::test(flavor = "multi_thread")]
async fn a_leftover_removed_file_is_retried_hourly_once_receiving_starts() {
    let (_guard, keychain) = fresh_keychain();
    let opened_at = 8_000_000u64;
    let clock = TestClock::at(opened_at);
    let dir = store_dir("ack-leftover", "x");
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let leftover = common::handles::keep_device_seed([0x73; 32]);
    let (_, removed_file) = store_and_removed_files(&dir, &leftover);
    std::fs::write(&removed_file, b"sealed under a secret about to go").unwrap();

    keychain.refuse_as(Some(Refusal::Locked));
    let io = org_io::OrgIo::open_with_clock_for_test(&dir, TransportMode::Loopback, clock.clone()).unwrap();
    keychain.refuse_as(None);
    assert!(removed_file.exists() && item_held(&keychain, &leftover), "open does not finish it while the keychain refuses");
    io.start_receiving().await.unwrap();
    clock.set(opened_at + 3_599);
    io.wake_deliveries_for_test();
    tokio::time::sleep(QUIET).await;
    assert!(removed_file.exists(), "not retried before the hour");
    clock.set(opened_at + 3_600);
    io.wake_deliveries_for_test();
    ack_until(NET, || !removed_file.exists() && !item_held(&keychain, &leftover)).await;
}

/// Make `dir` unlistable (`true`: writable and searchable, not readable, so
/// a listing fails and a named file can still be renamed or removed) or
/// listable again (`false`).
#[cfg(unix)]
fn ack_unlistable(dir: &std::path::Path, unlistable: bool) {
    use std::os::unix::fs::PermissionsExt;
    let mode = if unlistable { 0o300 } else { 0o755 };
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(mode)).unwrap();
}

// Abnormal: the data directory cannot be listed when receiving starts (a
// transient failure of the listing). A leftover `removed-` file `open` could
// not finish is still retried an hour later, once the listing works again.
// verifies: LLR-3fxxpp
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_leftover_is_retried_hourly_though_the_listing_failed_when_receiving_started() {
    let (_guard, keychain) = fresh_keychain();
    let opened_at = 8_500_000u64;
    let clock = TestClock::at(opened_at);
    let dir = store_dir("ack-leftover-unlisted", "x");
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let leftover = common::handles::keep_device_seed([0x75; 32]);
    let (_, removed_file) = store_and_removed_files(&dir, &leftover);
    std::fs::write(&removed_file, b"sealed under a secret about to go").unwrap();

    keychain.refuse_as(Some(Refusal::Locked));
    let io = org_io::OrgIo::open_with_clock_for_test(&dir, TransportMode::Loopback, clock.clone()).unwrap();
    keychain.refuse_as(None);
    assert!(removed_file.exists() && item_held(&keychain, &leftover), "open does not finish it while the keychain refuses");
    ack_unlistable(&dir, true);
    let started = io.start_receiving().await;
    ack_unlistable(&dir, false);
    started.unwrap();
    clock.set(opened_at + 3_600);
    io.wake_deliveries_for_test();
    ack_until(NET, || !removed_file.exists() && !item_held(&keychain, &leftover)).await;
}

// Abnormal (PR-yng83g): `open` finds a removal past its deadline and its
// rename fails (the data directory read-only while it opens; a directory
// standing at the removed name would be taken for a leftover by open's
// first step). The Persona stays with its file and item and the failure is
// reported; once receiving starts, the deletion is tried again an hour
// later, not before, while the handle runs, not only at the next open.
// verifies: LLR-3fxxpp, LLR-upqj5g
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_deletion_at_open_is_retried_hourly_once_receiving_starts() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 9_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-open-retry";
    let admin = journey::admin(tag).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    // Dead addresses for the Admin: Bob's delivery reaches no one, so its
    // acknowledgement stays kept until the deadline.
    let (dead, listening) = journey::soon_dead(admin.device_key).await;
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, dead).await;
    listening.close().await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let bob_device = bob.device_key;
    drop(bob);
    let bob_dir = store_dir(tag, "b");
    let (persona_file, removed_file) = store_and_removed_files(&bob_dir, &bob_device);

    clock.set(verified_at + 604_800);
    make_read_only(&bob_dir);
    let opened = org_io::OrgIo::open_with_clock_for_test(&bob_dir, TransportMode::Loopback, clock.clone());
    make_writable(&bob_dir);
    let io = opened.unwrap();
    assert!(persona_file.exists() && item_held(&keychain, &bob_device), "the rename failed at open: nothing deleted");
    assert!(
        io.acknowledgement_report(admin.org).await.is_some_and(|report| report.recipients.len() == 1),
        "the Persona stays in the service, its acknowledgement kept"
    );
    let report = io.acknowledgement_report(admin.org).await.expect("the failure is reported");
    assert!(report.last_deletion_failure.is_some_and(|failure| failure.to_string().contains("rename")), "the failed step is named");

    io.start_receiving().await.unwrap();
    clock.set(verified_at + 604_800 + 3_599);
    io.wake_deliveries_for_test();
    tokio::time::sleep(QUIET).await;
    assert!(persona_file.exists(), "not retried before the hour");
    clock.set(verified_at + 604_800 + 3_600);
    io.wake_deliveries_for_test();
    ack_until(NET, || !persona_file.exists() && !removed_file.exists() && !item_held(&keychain, &bob_device)).await;
    ack_report_gone(&io, admin.org, NET).await;
    assert!(io.acknowledgement_report(admin.org).await.is_none(), "dropped from the service: no removal held");
    assert!(item_held(&keychain, &admin.device_key), "no other Persona's item touched");
}

// Abnormal (PR-wch3zu): a `removed-<key>.store` beside a live `persona-<key>.store` of the same
// key, which org-io's own deletion never leaves (it renames the store before the item goes) but
// a restored backup or a copied directory can. `open` leaves both files and the keychain item,
// opens the Persona, and reports the leftover as not finished; an hour after receiving starts,
// the retry leaves them too: a Persona's item is never deleted while its store is live
// (RC-hd5des).
// verifies: LLR-upqj5g, LLR-3fxxpp
#[tokio::test(flavor = "multi_thread")]
async fn a_removed_file_beside_a_live_store_of_its_key_is_kept_and_reported() {
    let (_guard, keychain) = fresh_keychain();
    let opened_at = 10_000_000u64;
    let clock = TestClock::at(opened_at);
    let tag = "ack-leftover-live";
    let admin = journey::admin(tag).await;
    let (org, device_key) = (admin.org, admin.device_key);
    drop(admin);
    let dir = store_dir(tag, "a");
    let (persona_file, removed_file) = store_and_removed_files(&dir, &device_key);
    std::fs::copy(&persona_file, &removed_file).unwrap();

    let io = org_io::OrgIo::open_with_clock_for_test(&dir, TransportMode::Loopback, clock.clone()).unwrap();
    assert!(item_held(&keychain, &device_key), "the live Persona's item is kept");
    assert!(persona_file.exists() && removed_file.exists(), "both files are left untouched");
    assert!(io.personas().await.iter().any(|persona| persona.device_key == device_key), "the Persona opens");
    let report = io.acknowledgement_report(org).await.expect("the kept leftover is reported");
    assert!(
        report.last_deletion_failure.as_ref().is_some_and(|failure| failure.to_string().contains("live")),
        "the report says why: {:?}",
        report.last_deletion_failure
    );

    io.start_receiving().await.unwrap();
    clock.set(opened_at + 3_600);
    io.wake_deliveries_for_test();
    tokio::time::sleep(QUIET).await;
    assert!(item_held(&keychain, &device_key), "the hourly retry keeps the item");
    assert!(persona_file.exists() && removed_file.exists(), "and both files");
}

// Boundary (merge review round 1, finding-1): a `removed-` leftover kept
// beside a live store that keeps no removal is reported with no deadline, as
// no removal is kept to have one, never one at the Unix epoch.
// verifies: LLR-rav7dr
#[tokio::test(flavor = "multi_thread")]
async fn a_kept_leftover_beside_a_store_keeping_no_removal_reports_no_deadline() {
    let (_guard, _keychain) = fresh_keychain();
    let clock = TestClock::at(10_500_000);
    let tag = "ack-leftover-no-deadline";
    let admin = journey::admin(tag).await;
    let (org, device_key) = (admin.org, admin.device_key);
    drop(admin);
    let dir = store_dir(tag, "a");
    let (persona_file, removed_file) = store_and_removed_files(&dir, &device_key);
    std::fs::copy(&persona_file, &removed_file).unwrap();

    let io = org_io::OrgIo::open_with_clock_for_test(&dir, TransportMode::Loopback, clock).unwrap();
    let report = io.acknowledgement_report(org).await.expect("the kept leftover is reported");
    assert!(report.last_deletion_failure.is_some(), "the leftover is the reported failure");
    assert_eq!(report.deadline, None, "no removal is kept: there is no deadline");
}

// Boundary (LLR-upqj5g's second clause): the last acknowledgement was
// accepted and written, so the store keeps a removal with none left, but the
// deletion at once failed (a directory stands at the removed name) and the
// handle closed. The next open, days before the deadline, deletes the
// Persona because none is kept: its files and item are gone and the handle
// holds no store of it.
// verifies: LLR-upqj5g
#[tokio::test(flavor = "multi_thread")]
async fn open_deletes_a_removal_that_keeps_no_acknowledgement_before_its_deadline() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 11_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-open-none-kept";
    let admin = journey::admin(tag).await;
    let (accepting, mut accepting_gets, _admin_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, accepting[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    let (persona_file, removed_file) = store_and_removed_files(&store_dir(tag, "b"), &bob.device_key);
    std::fs::create_dir(&removed_file).unwrap();
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut accepting_gets, NET).await.expect("accepted at T");
    ack_report_when(&bob.io, admin.org, NET, |report| report.last_deletion_failure.is_some()).await;
    let kept = common::handles::reopened_from(&bob.dir, &bob.io).kept_acknowledgements(&bob.device_key).unwrap().expect("still held, in the file");
    assert!(kept.acknowledgements.is_empty(), "the acceptance was written: none kept");
    assert!(
        bob.io.acknowledgement_report(admin.org).await.is_some_and(|report| report.recipients.is_empty()),
        "and the handle holds the removal with none kept"
    );
    let bob_device = bob.device_key;
    drop(bob);
    std::fs::remove_dir(&removed_file).unwrap();

    let reopened = org_io::OrgIo::open_with_clock_for_test(&store_dir(tag, "b"), TransportMode::Loopback, clock.clone()).unwrap();
    assert!(!persona_file.exists() && !removed_file.exists(), "deleted at open, before the deadline");
    assert!(!item_held(&keychain, &bob_device), "its item deleted");
    assert!(reopened.acknowledgement_report(admin.org).await.is_none(), "the reopened handle holds none");
    assert!(item_held(&keychain, &admin.device_key), "no other Persona's item touched");
}

// Boundary: a delivery that starts with org-node's
// list empty deletes the Persona at once, sending nothing. The last
// acknowledgement was accepted and written, and the deletions at once and at
// the next open both failed (a directory at the removed name); the obstacle
// gone, receiving starts, the delivery resumed for the removal starts with
// none kept, and the Persona is deleted days before its deadline with no
// round. (At a verification the list is never empty: the Admin whose epoch
// removed this Device is another Device of its record, so org-node keeps an
// acknowledgement for it; the start of a resumed delivery is where an empty
// list reaches the delivery.) LLR-zzsqj4 names this as its abnormal-case
// test.
// verifies: LLR-n649th, LLR-5rxqr8, LLR-zzsqj4
#[tokio::test(flavor = "multi_thread")]
async fn a_delivery_that_starts_with_none_kept_deletes_the_persona_at_once() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 11_500_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-start-none-kept";
    let admin = journey::admin(tag).await;
    let (accepting, mut accepting_gets, _admin_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![]).await;
    let bob = ack_removed_at_with_obstacle(&admin, tag, &clock, accepting[..1].to_vec()).await;
    journey::next_arrival(&mut accepting_gets, NET).await.expect("accepted at T");
    ack_report_when(&bob.io, admin.org, NET, |report| report.last_deletion_failure.is_some()).await;
    let (bob_device, bob_dir) = (bob.device_key, bob.dir.clone());
    drop(bob);
    let (persona_file, removed_file) = store_and_removed_files(&bob_dir, &bob_device);

    clock.set(verified_at + 3_600);
    let reopened = org_io::OrgIo::open_with_clock_for_test(&bob_dir, TransportMode::Loopback, clock.clone()).unwrap();
    assert!(persona_file.exists() && item_held(&keychain, &bob_device), "the deletion at open failed too");
    assert!(
        reopened.acknowledgement_report(admin.org).await.is_some_and(|report| report.recipients.is_empty()),
        "the removal is held with none kept"
    );
    std::fs::remove_dir(&removed_file).unwrap();
    reopened.start_receiving().await.unwrap();
    ack_until(NET, || !persona_file.exists() && !removed_file.exists() && !item_held(&keychain, &bob_device)).await;
    ack_report_gone(&reopened, admin.org, NET).await;
    assert!(reopened.acknowledgement_report(admin.org).await.is_none(), "no report once deleted");
    assert!(journey::next_arrival(&mut accepting_gets, QUIET).await.is_none(), "no round: nothing sent");
    assert!(item_held(&keychain, &admin.device_key), "no other Persona's item touched");
}

/// `ack_removed_at`, with a directory standing at the store's removed name
/// before the verification, so that every deletion's rename fails until the
/// test removes it.
async fn ack_removed_at_with_obstacle(admin: &journey::Admin, tag: &str, clock: &Arc<TestClock>, addresses: Vec<String>) -> journey::Joining {
    let mut bob = journey::member(admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, addresses).await;
    journey::revoked(admin, &bob, &[]).await;
    std::fs::create_dir(store_and_removed_files(&bob.dir, &bob.device_key).1).unwrap();
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    bob
}

/// A stand-in recipient (its `Accepted` under the contract test
/// `the_scripted_recipients_verdicts_are_the_real_receive_steps`): an
/// endpoint of `device` that holds its first acknowledgement's verdict until
/// `release` is notified and then answers it `Accepted`, and answers every
/// other body `Accepted` at once; its addresses and arrivals (the
/// acknowledgements only).
async fn ack_accepting_after(device_key: org_node::DevicePublicKey, release: Arc<tokio::sync::Notify>) -> (Vec<String>, journey::Arrivals, Arc<OrgEndpoint>) {
    let endpoint = Arc::new(OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await.unwrap());
    let addresses = journey::kept_addresses(&endpoint);
    let (forward, arrivals) = tokio::sync::mpsc::unbounded_channel();
    let mut taken = journey::each_inbound(endpoint.clone());
    tokio::spawn(async move {
        let mut first = true;
        while let Some((sender, body, reply)) = taken.recv().await {
            if !journey::is_acknowledgement(&body) {
                tokio::spawn(reply.answer(Verdict::Accepted));
                continue;
            }
            // The test may have stopped reading; a body no one reads is dropped.
            let _ = forward.send((sender, body));
            if std::mem::take(&mut first) {
                release.notified().await;
            }
            tokio::spawn(reply.answer(Verdict::Accepted));
        }
    });
    (addresses, arrivals, endpoint)
}

// Abnormal: the recipient accepts, but the store write of org-node's
// deletion fails (the data directory read-only). The Persona is reloaded
// from its file, so the acknowledgement stays kept, nothing is deleted and
// no deletion is tried; at the next round it is sent again, and that
// acceptance, written, leaves none and deletes the Persona.
// verifies: LLR-6ux6yw
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_write_of_an_acceptance_keeps_the_acknowledgement_for_the_next_round() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 12_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-accept-write-fails";
    let admin = journey::admin(tag).await;
    let release = Arc::new(tokio::sync::Notify::new());
    let (accepting, mut accepting_gets, _admin_endpoint) = ack_accepting_after(admin.device_key, release.clone()).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, accepting[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut accepting_gets, NET).await.expect("the round at T");
    let bob_dir = store_dir(tag, "b");
    let (persona_file, _) = store_and_removed_files(&bob_dir, &bob.device_key);
    let writes = store_writes_to(&bob_dir, &bob.device_key).len();
    let kept = journey::kept_acknowledgement_bodies(&bob);

    make_read_only(&bob_dir);
    release.notify_one();
    let report = tokio::time::timeout(NET, async {
        loop {
            if let Some(report) = bob.io.acknowledgement_report(admin.org).await.filter(|report| report.next_round.is_some()) {
                return Some(report);
            }
            // The task ended: the removal is gone with the Persona.
            bob.io.acknowledgement_report(admin.org).await.as_ref()?;
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await;
    make_writable(&bob_dir);
    let report = report.expect("the round ended").expect("the task still runs");
    assert_eq!(report.next_round, Some(verified_at + 60), "the next round is scheduled");
    assert_eq!(report.last_deletion_failure, None, "no deletion was tried");
    assert_eq!(journey::kept_acknowledgement_bodies(&bob), kept, "reloaded from its file: still kept");
    assert_eq!(store_writes_to(&bob_dir, &bob.device_key).len(), writes, "nothing written");
    assert!(persona_file.exists() && item_held(&keychain, &bob.device_key), "nothing deleted");

    clock.set(verified_at + 60);
    bob.io.wake_deliveries_for_test();
    journey::next_arrival(&mut accepting_gets, NET).await.expect("sent again at the next round");
    ack_until(NET, || !persona_file.exists() && !item_held(&keychain, &bob.device_key)).await;
}

// Abnormal (merge review round 1, finding-2): the recipient accepts, but the
// store write of org-node's deletion fails (the data directory read-only), so
// the acknowledgement stays kept. The report gives that round's outcome for
// the recipient — it accepted and the acceptance was not recorded, naming the
// failed write — not the outcome of an earlier round or `NotYetSent`.
// verifies: LLR-rav7dr, LLR-6ux6yw
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn an_acceptance_whose_write_failed_is_that_rounds_outcome_in_the_report() {
    let (_guard, _keychain) = fresh_keychain();
    let verified_at = 12_500_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-accept-write-outcome";
    let admin = journey::admin(tag).await;
    let release = Arc::new(tokio::sync::Notify::new());
    let (accepting, mut accepting_gets, _admin_endpoint) = ack_accepting_after(admin.device_key, release.clone()).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, accepting[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut accepting_gets, NET).await.expect("the round at T");
    let bob_dir = store_dir(tag, "b");

    make_read_only(&bob_dir);
    release.notify_one();
    let report = tokio::time::timeout(NET, async {
        loop {
            if let Some(report) = bob.io.acknowledgement_report(admin.org).await.filter(|report| report.next_round.is_some()) {
                return report;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await;
    make_writable(&bob_dir);
    let report = report.expect("the round ended");
    let [(recipient, outcome)] = report.recipients.as_slice() else { panic!("one kept recipient: {:?}", report.recipients) };
    assert_eq!(*recipient, admin.device_key);
    assert_ne!(*outcome, AcknowledgementOutcome::NotYetSent, "the round reached it and it accepted");
    assert!(
        matches!(outcome, AcknowledgementOutcome::AcceptedNotRecorded { reason: AcceptanceFailure::Store(_) }),
        "accepted, the failed write named: {outcome:?}"
    );
    assert_eq!(journey::kept_acknowledgement_bodies(&bob).len(), 1, "still kept for the next round");
}

// Abnormal: inside the 7-day
// window the recipient accepts, the store write of org-node's deletion fails
// (the data directory read-only) and so does the reload (the keychain
// refuses), so the removed store is dropped from memory while its
// `persona-` file and keychain item are live. The task's next round finds no
// store and ends its delivery; no deletion was begun (no `removed-` file), so
// the file and the item are kept, and the next open opens the Persona with
// its acknowledgement still kept.
// verifies: LLR-vj2nkb, LLR-3fxxpp
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_store_dropped_from_memory_keeps_its_live_file_and_item() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 16_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-dropped-live";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _admin_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused)]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T, refused");
    ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    let bob_dir = store_dir(tag, "b");
    let (persona_file, removed_file) = store_and_removed_files(&bob_dir, &bob.device_key);
    let bytes = std::fs::read(&persona_file).unwrap();

    make_read_only(&bob_dir);
    keychain.refuse_as(Some(Refusal::Unavailable));
    clock.set(verified_at + 60);
    bob.io.wake_deliveries_for_test();
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T+1 min, accepted");
    let dropped = tokio::time::timeout(NET, async {
        while bob.io.acknowledgement_report(admin.org).await.is_some() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await;
    keychain.refuse_as(None);
    make_writable(&bob_dir);
    dropped.expect("the failed write and reload dropped the store");
    assert_eq!(std::fs::read(&persona_file).unwrap(), bytes, "the file is the one written before the failure");

    clock.set(verified_at + 300);
    bob.io.wake_deliveries_for_test();
    // The round finds no store and ends the delivery; nothing of it shows but what it leaves.
    tokio::time::sleep(QUIET).await;
    assert!(item_held(&keychain, &bob.device_key), "the live file's keychain item is kept");
    assert!(persona_file.exists() && !removed_file.exists(), "the live file is kept, and no deletion begun");

    drop(bob);
    let reopened = org_io::OrgIo::open_with_clock_for_test(&bob_dir, TransportMode::Loopback, clock.clone()).expect("the next open opens it");
    let report = reopened.acknowledgement_report(admin.org).await.expect("a removed store");
    assert_eq!(report.recipients, vec![(admin.device_key, AcknowledgementOutcome::NotYetSent)], "its acknowledgement still kept for the next delivery");
}

// Abnormal (LLR-3fxxpp's last step): the file was renamed and the keychain
// refused the item; an hour later the item is deleted but the removal of the
// renamed file is refused (the data directory read-only). The failure names
// the store file step, the item stays deleted, the renamed file stays; an
// hour after that, the directory writable again, the file is removed and
// the report is gone.
// verifies: LLR-3fxxpp
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_removal_of_the_renamed_file_is_reported_and_retried_hourly() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 13_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-remove-refused";
    let admin = journey::admin(tag).await;
    let (refusing, mut refusing_gets, _refusing_endpoint) = journey::answering_acknowledgements(admin.device_key, BindPlan::Loopback, vec![Some(Verdict::Refused); 4]).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, refusing[..1].to_vec()).await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    journey::next_arrival(&mut refusing_gets, NET).await.expect("the round at T");
    ack_report_when(&bob.io, admin.org, NET, |report| report.next_round.is_some()).await;
    let bob_dir = store_dir(tag, "b");
    let (persona_file, removed_file) = store_and_removed_files(&bob_dir, &bob.device_key);

    keychain.refuse_as(Some(Refusal::Locked));
    clock.set(verified_at + 604_800);
    bob.io.wake_deliveries_for_test();
    ack_report_when(&bob.io, admin.org, NET, |report| report.last_deletion_failure.is_some()).await;
    keychain.refuse_as(None);
    assert!(!persona_file.exists() && removed_file.exists(), "renamed before the item step");

    make_read_only(&bob_dir);
    clock.set(verified_at + 604_800 + 3_600);
    bob.io.wake_deliveries_for_test();
    let report = tokio::time::timeout(NET, async {
        loop {
            let report = bob.io.acknowledgement_report(admin.org).await;
            if report.as_ref().is_none_or(|report| report.last_deletion_failure.as_ref().is_some_and(|failure| failure.to_string().contains("store file"))) {
                return report;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await;
    make_writable(&bob_dir);
    let report = report.expect("the retry ran").expect("the failure is reported");
    assert!(report.last_deletion_failure.unwrap().to_string().contains("store file"), "the failed step is named");
    assert!(report.recipients.is_empty(), "the Persona is dropped: no recipient");
    assert!(!item_held(&keychain, &bob.device_key), "the item went first");
    assert!(removed_file.exists(), "the renamed file stays");

    clock.set(verified_at + 604_800 + 7_200);
    bob.io.wake_deliveries_for_test();
    ack_until(NET, || !removed_file.exists()).await;
    ack_report_gone(&bob.io, admin.org, NET).await;
    assert!(bob.io.acknowledgement_report(admin.org).await.is_none(), "no report once deleted");
}

// Abnormal (PR-yng83g's retry, `start_receiving` called twice): a deletion
// that failed at open is retried by one task, not one per call. With the
// keychain refusing at the retry, the hour's retry asks the keychain once.
// verifies: LLR-3fxxpp
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn a_deletion_failed_at_open_is_retried_once_an_hour_however_often_receiving_starts() {
    let (_guard, keychain) = fresh_keychain();
    let verified_at = 14_000_000u64;
    let clock = TestClock::at(verified_at);
    let tag = "ack-open-retry-once";
    let admin = journey::admin(tag).await;
    let mut bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    bob.io = bob.io.with_clock_for_test(clock.clone());
    let (dead, listening) = journey::soon_dead(admin.device_key).await;
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, dead).await;
    listening.close().await;
    journey::revoked(&admin, &bob, &[]).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    let bob_device = bob.device_key;
    drop(bob);
    let bob_dir = store_dir(tag, "b");

    clock.set(verified_at + 604_800);
    make_read_only(&bob_dir);
    let opened = org_io::OrgIo::open_with_clock_for_test(&bob_dir, TransportMode::Loopback, clock.clone());
    make_writable(&bob_dir);
    let io = opened.unwrap();
    assert!(io.acknowledgement_report(admin.org).await.is_some_and(|report| report.last_deletion_failure.is_some()));
    io.start_receiving().await.unwrap();
    io.start_receiving().await.unwrap();
    tokio::time::sleep(QUIET).await;

    keychain.refuse_as(Some(Refusal::Locked));
    let calls = keychain.calls();
    clock.set(verified_at + 604_800 + 3_600);
    // Each wake rouses one waiting task: two, so a second task would run too.
    io.wake_deliveries_for_test();
    io.wake_deliveries_for_test();
    ack_report_when(&io, admin.org, NET, |report| report.last_deletion_failure.as_ref().is_some_and(|failure| failure.to_string().contains("keychain item"))).await;
    tokio::time::sleep(QUIET).await;
    let asked = keychain.calls() - calls;
    keychain.refuse_as(None);
    assert_eq!(asked, 1, "one retry this hour: one keychain deletion asked");
    assert!(item_held(&keychain, &bob_device), "the item stays while the keychain refuses");
}

// Abnormal (`start_receiving` called twice): a leftover `removed-` file
// whose deletion open could not finish is retried by one task, not one per
// call. With the keychain still refusing, the hour's retry asks it once.
// verifies: LLR-3fxxpp
#[tokio::test(flavor = "multi_thread")]
async fn a_leftover_removed_file_is_retried_once_an_hour_however_often_receiving_starts() {
    let (_guard, keychain) = fresh_keychain();
    let opened_at = 15_000_000u64;
    let clock = TestClock::at(opened_at);
    let dir = store_dir("ack-leftover-once", "x");
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let leftover = common::handles::keep_device_seed([0x74; 32]);
    let (_, removed_file) = store_and_removed_files(&dir, &leftover);
    std::fs::write(&removed_file, b"sealed under a secret about to go").unwrap();

    keychain.refuse_as(Some(Refusal::Locked));
    let io = org_io::OrgIo::open_with_clock_for_test(&dir, TransportMode::Loopback, clock.clone()).unwrap();
    io.start_receiving().await.unwrap();
    io.start_receiving().await.unwrap();
    let calls = keychain.calls();
    clock.set(opened_at + 3_600);
    // Each wake rouses one waiting task: two, so a second task would run too.
    io.wake_deliveries_for_test();
    io.wake_deliveries_for_test();
    tokio::time::sleep(QUIET).await;
    let asked = keychain.calls() - calls;
    keychain.refuse_as(None);
    assert_eq!(asked, 1, "one retry this hour: one keychain deletion asked");
    assert!(removed_file.exists() && item_held(&keychain, &leftover), "left while the keychain refuses");
}
