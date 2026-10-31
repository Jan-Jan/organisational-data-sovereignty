#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The send path to the addresses kept per Device key (SDD-2bzgtp):
//! `send_organisation` sends org-node's messages for the current epoch, in
//! its order, to every recipient, and reports each Device's delivery; a
//! failed send writes no store and leaves the record (LLR-pxqdy4). Each
//! recipient is dialled at the addresses kept for its key: in Loopback mode
//! only those whose endpoint identity is its key, refused per recipient when
//! none is kept or none is its own; in Networked mode the key, with its kept
//! addresses as a hint (LLR-34tfm8). Every send is from the sending
//! Persona's endpoint (LLR-hx5c2y), an Organisation no Persona holds is not
//! held (LLR-zrgj8z), and the verdict byte the recipient writes is mapped to
//! the delivery (LLR-fwz3tb).
//!
//! The chain is a substitute (`test_support::FakeChain`); each normal-case
//! recipient is a real receiving handle, whose receive step keeps what the
//! send carries and writes the verdict. A stand-in recipient
//! (`journey::answering`) stands only where a test injects what a real one
//! does not do: a verdict byte it never writes for an Organisation's
//! information, a dropped reply, or the address of another Device. Every
//! test runs one at a time under `fresh_keychain`'s lock, over a fresh
//! in-memory keychain.

use std::time::Duration;

use iroh::address_lookup::MemoryLookup;
use org_io::send::{Delivery, SendError, SendRefused};
use org_io::test_support::{FakeChain, Refusal};
use org_io::transport::{BindPlan, OrgEndpoint, Verdict};
use org_io::{HeldTarget, ReceiveError, Received, Verified, WriteReport};
use org_node::{Epoch, OrgId};

mod common;
use common::handles::{fresh_keychain, handle, keep_device_seed, store_bytes, store_dir, NET};
use common::journey;

/// How long a receiver waits before it concludes nothing was sent to it.
const QUIET: Duration = Duration::from_secs(2);

// Normal, over real recipients: Bob and Carol are Members holding the epoch
// that admitted Carol, and the adopted epoch, Dave's admission, is held by
// none of the three; each receives on its own handle. One call sends
// org-node's messages for the adopted epoch, in org-node's order, each to
// its recipient from the Admin's Device: each recipient's receive step keeps
// that epoch (the Admin's epoch and root, from the Admin's Device), and every
// recipient is reported Delivered. A second call sends the same messages:
// each recipient already holds that epoch, accepts it and reports nothing.
// No store byte of the Admin's changes.
// verifies: LLR-pxqdy4, REQ-u894ty
#[tokio::test(flavor = "multi_thread")]
async fn send_organisation_sends_org_nodes_messages_in_order_to_every_recipient_and_writes_no_store() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "send-in-order";
    let admin = journey::admin(tag).await;
    let bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    let carol = journey::member(&admin, tag, "c", "carol", &[&bob]).await;
    let dave = journey::invited(&admin, tag, "d", "dave").await;
    journey::admit(&admin, &dave).await;
    let expected: Vec<_> = journey::current_messages(&admin).into_iter().map(|(recipient, _)| recipient).collect();
    let record = admin.io.organisation(admin.org).await.unwrap();
    let files = store_bytes(tag, "a");

    let (received, report) = journey::epoch_to(&admin, &[&bob.io, &carol.io, &dave.io]).await;
    let order: Vec<_> = report.per_device.iter().map(|(recipient, _)| *recipient).collect();
    assert_eq!(order, expected, "org-node's order");
    assert!(report.per_device.iter().all(|(_, sent)| matches!(sent, Delivery::Delivered)), "{report:?}");
    for kept in &received {
        assert!(
            matches!(kept, Ok(Received::ProvisionalOrganisationReceived { org_id, epoch, root, from })
                if (*org_id, *epoch, *root, *from) == (admin.org, record.epoch, record.root_hash, admin.device_key)),
            "the adopted epoch's message, from the Admin's Device: {kept:?}"
        );
    }

    let send = admin.io.send_organisation(admin.org);
    let (((report, at_dave), at_carol), at_bob) =
        journey::while_receiving(&bob.io, journey::while_receiving(&carol.io, journey::while_receiving(&dave.io, send))).await;
    let order: Vec<_> =
        report.unwrap().per_device.iter().map(|(recipient, sent)| (*recipient, matches!(sent, Delivery::Delivered))).collect();
    assert_eq!(order, expected.iter().map(|recipient| (*recipient, true)).collect::<Vec<_>>(), "sent again, in order, and accepted");
    assert!(at_bob.is_empty() && at_carol.is_empty() && at_dave.is_empty(), "already held: {at_bob:?} {at_carol:?} {at_dave:?}");
    assert_eq!(store_bytes(tag, "a"), files, "the send writes no store");
}

// Abnormal, per recipient, in one Loopback send: no address kept (Bob), only
// an address whose identity is another Device (Carol's), and a proper one
// (Dave's, a real receiving handle). The first two are refused with nothing
// sent to them; Dave is still sent to, and his receive step keeps the epoch.
// verifies: LLR-34tfm8
#[tokio::test(flavor = "multi_thread")]
async fn loopback_refuses_no_address_and_no_address_of_the_recipient_per_recipient() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "send-loopback-refusals";
    let admin = journey::admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_keeping(&admin, &bob, vec![]).await;
    let rogue = keep_device_seed([0x5c; 32]);
    let (rogue_at, mut rogue_gets, _rogue) = journey::answering(rogue, BindPlan::Loopback, vec![]).await;
    let carol = journey::invited(&admin, tag, "c", "carol").await;
    journey::admit_keeping(&admin, &carol, rogue_at).await;
    let dave = journey::invited(&admin, tag, "d", "dave").await;
    journey::admit(&admin, &dave).await;

    let (at_dave, report) = journey::deliver_epoch(&admin.io, admin.org, &dave.io).await;
    let outcome = |device_key| &report.per_device.iter().find(|(recipient, _)| *recipient == device_key).unwrap().1;
    assert!(matches!(outcome(bob.device_key), Delivery::NotDelivered { reason: SendError::NoLoopbackAddress }), "{report:?}");
    assert!(matches!(outcome(carol.device_key), Delivery::NotDelivered { reason: SendError::AddressNotARecipient }), "{report:?}");
    assert!(matches!(outcome(dave.device_key), Delivery::Delivered), "{report:?}");
    assert!(matches!(at_dave, Ok(Received::ProvisionalOrganisationReceived { from, .. }) if from == admin.device_key), "{at_dave:?}");
    assert!(journey::next_arrival(&mut rogue_gets, QUIET).await.is_none(), "nothing reaches another Device's address");
}

// Abnormal, Loopback: the only addresses kept for Bob are not in the kept
// text form — his own endpoint's address with its identity in upper-case
// hex, and one whose identity is too short. Neither names an identity, so
// neither is dialled: Bob is refused `AddressNotARecipient` and his
// receiving handle is sent nothing.
// verifies: LLR-34tfm8
#[tokio::test(flavor = "multi_thread")]
async fn loopback_dials_no_kept_address_outside_the_lower_case_hex_form() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "send-address-form";
    let admin = journey::admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    bob.io.start_receiving().await.unwrap();
    let bob_at = journey::addresses_of(&bob.io).await;
    let (endpoint_id, socket) = bob_at[0].split_once('@').unwrap();
    let malformed = vec![format!("{}@{socket}", endpoint_id.to_uppercase()), format!("{}@{socket}", &endpoint_id[..62])];
    journey::admit_keeping(&admin, &bob, malformed).await;

    let report = admin.io.send_organisation(admin.org).await.unwrap();
    let outcome = &report.per_device.iter().find(|(recipient, _)| *recipient == bob.device_key).unwrap().1;
    assert!(matches!(outcome, Delivery::NotDelivered { reason: SendError::AddressNotARecipient }), "{report:?}");
    assert!(journey::reports_nothing(&bob.io, QUIET).await, "no malformed address was dialled");
}

/// `journey::admit_keeping` for an Admin bound for the in-process relay, whose
/// sockets are the wildcard ones: the joiner's request is sent from `from`
/// (the joiner Device's own endpoint, so the Admin learns no other path to
/// that Device) to their loopback form, then kept by the Admin's receive
/// step, proposed, written and adopted (no state written by hand).
async fn admit_keeping_networked(admin: &journey::Admin, joining: &journey::Joining, from: &OrgEndpoint, addresses: Vec<String>) {
    admin.io.start_receiving().await.unwrap();
    let request = common::handles::reopened_from(&joining.dir, &joining.io).build_admission_request(&joining.device_key, addresses).unwrap();
    let persona_id = admin.io.personas().await[0].persona_id.clone();
    let endpoint = admin.io.endpoint_for_test(&persona_id).await.unwrap();
    let loopback: Vec<std::net::SocketAddr> = endpoint
        .bound_sockets()
        .into_iter()
        .map(|socket| {
            let ip = if socket.is_ipv4() { std::net::Ipv4Addr::LOCALHOST.into() } else { std::net::Ipv6Addr::LOCALHOST.into() };
            std::net::SocketAddr::new(ip, socket.port())
        })
        .collect();
    let (read_back, kept) = tokio::join!(from.send_for_verdict(&admin.device_key, &loopback, &request.body), journey::next(&admin.io));
    assert_eq!(read_back.unwrap(), vec![Verdict::Accepted.as_byte()], "the request kept");
    assert!(matches!(kept, Ok(org_io::Received::AdmissionRequestKept { .. })), "{kept:?}");
    journey::propose_admission(admin, joining).await;
    journey::write_and_adopt(admin).await;
}

// Networked (the in-process relay and address lookup), over real receiving
// handles bound for the relay: a recipient whose kept addresses are none is
// dialled by its Device key and reached; one whose kept addresses are its
// own is reached with them as the hint. Each recipient's receive step keeps
// the epoch from the Admin's Device.
// verifies: LLR-34tfm8
#[tokio::test(flavor = "multi_thread")]
async fn networked_dials_the_recipients_key_with_its_kept_addresses_as_a_hint() {
    let (_guard, _keychain) = fresh_keychain();
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server().await.unwrap();
    let lookup = MemoryLookup::new();
    let plan = BindPlan::Relay(relay_map, lookup.clone());
    let tag = "send-networked";
    let chain = FakeChain::new();
    let io = handle(tag, "a", &chain).with_bind_plan_for_test(plan.clone());
    let device_key = journey::propose_founding(&io, "alice").await;
    assert!(matches!(io.write_organisation_to_chain(HeldTarget::Founding(device_key)).await.unwrap(), WriteReport::Included { .. }));
    assert!(matches!(io.verify_organisation(HeldTarget::Founding(device_key)).await, Verified::Adopted { .. }));
    let org = io.organisations().await[0].org_id;
    let admin = journey::Admin { io, chain, org, device_key, dir: store_dir(tag, "a") };
    let sender = endpoint_of(&admin.io).await;
    tokio::time::timeout(NET, sender.online()).await.unwrap();
    lookup.add_endpoint_info(sender.inner().addr());

    let bob = relayed(journey::invited(&admin, tag, "b", "bob").await, &plan, &lookup).await;
    let bob_endpoint = endpoint_of(&bob.io).await;
    admit_keeping_networked(&admin, &bob, &bob_endpoint, vec![]).await;
    let carol = relayed(journey::invited(&admin, tag, "c", "carol").await, &plan, &lookup).await;
    let carol_endpoint = endpoint_of(&carol.io).await;
    admit_keeping_networked(&admin, &carol, &carol_endpoint, journey::kept_addresses(&carol_endpoint)).await;

    let send = tokio::time::timeout(Duration::from_secs(120), admin.io.send_organisation(admin.org));
    let ((report, at_carol), at_bob) = journey::while_receiving(&bob.io, journey::while_receiving(&carol.io, send)).await;
    let report = report.unwrap().unwrap();
    assert!(report.per_device.iter().all(|(_, sent)| matches!(sent, Delivery::Delivered)), "{report:?}");
    let from_admin = |reported: &[Result<Received, ReceiveError>]| {
        matches!(reported, [Ok(Received::ProvisionalOrganisationReceived { from, .. })] if *from == admin.device_key)
    };
    assert!(from_admin(&at_bob), "no address kept: dialled by key: {at_bob:?}");
    assert!(from_admin(&at_carol), "kept addresses as the hint: {at_carol:?}");
}

/// `joining`'s handle bound for `plan` and receiving, its endpoint online
/// and in `lookup`.
async fn relayed(mut joining: journey::Joining, plan: &BindPlan, lookup: &MemoryLookup) -> journey::Joining {
    joining.io = joining.io.with_bind_plan_for_test(plan.clone());
    joining.io.start_receiving().await.unwrap();
    let endpoint = endpoint_of(&joining.io).await;
    tokio::time::timeout(NET, endpoint.online()).await.unwrap();
    lookup.add_endpoint_info(endpoint.inner().addr());
    joining
}

/// The endpoint of `io`'s first Persona.
async fn endpoint_of(io: &org_io::OrgIo) -> std::sync::Arc<OrgEndpoint> {
    let persona_id = io.personas().await[0].persona_id.clone();
    io.endpoint_for_test(&persona_id).await.unwrap()
}

// Every body comes from the endpoint of the Persona holding the
// Organisation, as Bob's real receive step reports it; when that endpoint
// cannot be had (closed, and its keychain item locked, so it cannot be bound
// again) every recipient reports `SenderEndpoint` and nothing is sent.
// verifies: LLR-hx5c2y
#[tokio::test(flavor = "multi_thread")]
async fn every_send_is_from_the_sending_personas_endpoint() {
    let (_guard, keychain) = fresh_keychain();
    let tag = "send-from-persona";
    let admin = journey::admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit(&admin, &bob).await;
    let (at_bob, _) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert!(
        matches!(at_bob, Ok(Received::ProvisionalOrganisationReceived { from, .. }) if from == admin.device_key),
        "the Admin Persona's Device: {at_bob:?}"
    );

    endpoint_of(&admin.io).await.inner().close().await;
    keychain.refuse_as(Some(Refusal::Locked));
    let report = admin.io.send_organisation(admin.org).await.unwrap();
    keychain.refuse_as(None);
    assert!(
        report.per_device.iter().all(|(_, sent)| matches!(sent, Delivery::NotDelivered { reason: SendError::SenderEndpoint(_) })),
        "{report:?}"
    );
    assert!(!report.per_device.is_empty(), "{report:?}");
    assert!(journey::reports_nothing(&bob.io, QUIET).await, "nothing sent");
}

// Abnormal: an Organisation no Persona here holds is `NotHeld`; boundary: a
// founded Organisation with no other Device has no recipient, and the report
// is empty.
// verifies: LLR-zrgj8z
#[tokio::test(flavor = "multi_thread")]
async fn an_organisation_no_persona_holds_is_not_held_and_no_recipient_is_an_empty_report() {
    let (_guard, _keychain) = fresh_keychain();
    let admin = journey::admin("send-not-held").await;
    let elsewhere = OrgId::new([0x99; 20]);
    let refused = admin.io.send_organisation(elsewhere).await;
    assert!(matches!(refused, Err(SendRefused::NotHeld { org_id }) if org_id == elsewhere), "{refused:?}");
    let report = admin.io.send_organisation(admin.org).await.unwrap();
    assert!(report.per_device.is_empty(), "{report:?}");
}

// Each verdict byte the recipient writes is reported as itself. `Accepted`
// and `Refused` come from Bob's real receive step: it refuses the adopted
// epoch while its store write fails, then keeps it. The three
// outcomes a real receive step never gives an Organisation's information —
// `AlreadyReceived`, `InviteUsedByAnotherDevice`, and silence (the reply
// dropped, `NotDelivered`) — are injected at Carol's address by a stand-in.
// A byte outside the four is no verdict.
// verifies: LLR-fwz3tb
#[tokio::test(flavor = "multi_thread")]
async fn the_verdict_byte_maps_to_delivered_refused_already_received_invite_used_or_not_delivered() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "send-verdicts";
    let admin = journey::admin(tag).await;
    let carol = journey::invited(&admin, tag, "c", "carol").await;
    let script = vec![Some(Verdict::AlreadyReceived), Some(Verdict::InviteUsedByAnotherDevice), None];
    let (carol_at, _carol_gets, _carol) = journey::answering(carol.device_key, BindPlan::Loopback, script).await;
    journey::admit_keeping(&admin, &carol, carol_at).await;
    let mut seen = vec![];
    for _ in 0..3 {
        let report = admin.io.send_organisation(admin.org).await.unwrap();
        seen.push(format!("{:?}", report.per_device[0].1));
    }
    assert!(seen[0].starts_with("AlreadyReceived"), "{seen:?}");
    assert!(seen[1].starts_with("InviteUsedByAnotherDevice"), "{seen:?}");
    assert!(seen[2].starts_with("NotDelivered"), "{seen:?}");

    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit(&admin, &bob).await;
    let outcome_for_bob = |report: &org_io::send::SendReport| {
        format!("{:?}", report.per_device.iter().find(|(recipient, _)| *recipient == bob.device_key).unwrap().1)
    };
    org_io::test_support::fail_next_write_after_temporary();
    let (refused, report) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert!(matches!(refused, Err(ReceiveError::Store(_))), "its store write failed: {refused:?}");
    assert!(outcome_for_bob(&report).starts_with("Refused"), "{report:?}");
    let (kept, report) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert!(matches!(kept, Ok(Received::ProvisionalOrganisationReceived { .. })), "{kept:?}");
    assert!(outcome_for_bob(&report).starts_with("Delivered"), "{report:?}");
    // "Any other byte is no verdict" is seen at the report (`another_byte_or_none_within_the_limit_is_reported_not_delivered`),
    // not on the transport's `Verdict` type.
}

// Abnormal: what the recipient
// writes back is judged at `send_organisation`'s report, not on the verdict
// type. A byte that is none of the four verdicts is `NotDelivered {
// UnknownVerdict(byte) }`; a recipient that takes the body and writes
// nothing, holding the stream open, is `NotDelivered { NoVerdict }` once the
// 10-second limit has passed, not before.
// verifies: LLR-fwz3tb
#[tokio::test(flavor = "multi_thread")]
async fn another_byte_or_none_within_the_limit_is_reported_not_delivered() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "send-verdict-bad-or-late";
    let admin = journey::admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    // A raw receiver at Bob's identity, below org-io's transport (which only
    // ever writes a verdict): it writes 0x07 back on the first stream, and
    // holds the second open, writing nothing, until the test ends.
    let endpoint = std::sync::Arc::new(OrgEndpoint::bind_from_keychain(&bob.device_key, BindPlan::Loopback).await.unwrap());
    let bob_at = journey::kept_addresses(&endpoint);
    let raw = endpoint.clone();
    let held = tokio::spawn(async move {
        let mut open = vec![];
        for byte in [Some(0x07u8), None] {
            let connection = raw.inner().accept().await.expect("a connection").await.expect("its handshake");
            let (mut send, mut recv) = connection.accept_bi().await.expect("its stream");
            recv.read_to_end(2 * 1024 * 1024).await.expect("its body");
            if let Some(byte) = byte {
                send.write_all(&[byte]).await.expect("the byte written");
                send.finish().expect("the stream finished");
            }
            open.push((connection, send));
        }
        open
    });
    journey::admit_keeping(&admin, &bob, bob_at).await;

    let report = admin.io.send_organisation(admin.org).await.unwrap();
    assert!(
        matches!(report.per_device.as_slice(), [(recipient, Delivery::NotDelivered { reason: SendError::UnknownVerdict(0x07) })] if *recipient == bob.device_key),
        "{report:?}"
    );
    let started = std::time::Instant::now();
    let report = admin.io.send_organisation(admin.org).await.unwrap();
    let waited = started.elapsed();
    assert!(
        matches!(report.per_device.as_slice(), [(_, Delivery::NotDelivered { reason: SendError::NoVerdict })]),
        "{report:?}"
    );
    assert!(waited >= org_io::transport::VERDICT_TIME_LIMIT, "not before the limit: {waited:?}");
    drop(held);
}

// Abnormal: a send to addresses where nothing listens is reported, and the adopted record and its store file
// stay as they were.
// verifies: LLR-pxqdy4
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_send_writes_no_store_and_leaves_the_record() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "send-failed";
    let admin = journey::admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    let nowhere = journey::dead_addresses(bob.device_key).await;
    journey::admit_keeping(&admin, &bob, nowhere).await;
    let files = store_bytes(tag, "a");
    let report = admin.io.send_organisation(admin.org).await.unwrap();
    assert!(matches!(report.per_device[0].1, Delivery::NotDelivered { .. }), "{report:?}");
    assert_eq!(admin.io.organisation(admin.org).await.unwrap().epoch, Epoch::new(2), "the record stands");
    assert_eq!(store_bytes(tag, "a"), files, "nothing written");
}

// Each recipient is dialled at the addresses its own key has kept, and only
// it receives there: each real receiving handle keeps the one message sent
// to its Device and is sent nothing else.
// verifies: LLR-34tfm8
#[tokio::test(flavor = "multi_thread")]
async fn each_recipient_is_dialled_at_the_addresses_kept_for_its_key() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "send-kept-addresses";
    let admin = journey::admin(tag).await;
    let bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    let carol = journey::invited(&admin, tag, "c", "carol").await;
    journey::admit(&admin, &carol).await;
    let (received, report) = journey::epoch_to(&admin, &[&bob.io, &carol.io]).await;
    assert_eq!(report.per_device.len(), 2, "{report:?}");
    for (kept, io) in received.iter().zip([&bob.io, &carol.io]) {
        assert!(matches!(kept, Ok(Received::ProvisionalOrganisationReceived { from, .. }) if *from == admin.device_key), "{kept:?}");
        assert!(journey::reports_nothing(io, QUIET).await, "and nothing else");
    }
}
