#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! A revoked Device may connect, for its acknowledgement only (REQ-ur7z4a),
//! over the receive step (SDD-2fc2cq): every completed handshake is
//! accepted and the body judged by org-node's check with its authenticated
//! sender, so only a revoked Device's acknowledgement is acted on
//! (LLR-j467e9); every inbound stream is answered with one verdict byte,
//! accepted or refused, and nothing else (LLR-hhgdk7). The tests run one at a
//! time under `fresh_keychain`'s lock: keyring's default credential builder
//! is process-global.

use std::path::Path;
use std::time::Duration;

use org_io::test_support::dial_direct_as;
use org_io::{ReceiveError, Received, Verified};
use org_node::wire::{decode_message, WireMessage};
use org_node::OrgNodeError;

mod common;
use common::handles::{fresh_keychain, keep_device_seed, store_bytes, NET};
use common::journey;

// ---- a revoked Device's connection --------------------------------------------

/// Admin `a` receiving, Member `b` admitted over the transport and then
/// removed; `b`'s one kept acknowledgement's body. Before its removal `b`
/// keeps dead addresses for the Admin (an organisation request from the
/// Admin's Device carrying them, taken by `b`'s receive step), so its own
/// delivery task cannot reach the Admin's receiving endpoint and the hand
/// dial is the only sender (a first adoption keeps the Invite issuer's live
/// addresses). The caller holds `fresh_keychain`'s lock.
async fn ack_removed(tag: &str) -> (journey::Admin, journey::Joining, Vec<u8>) {
    let admin = journey::admin(tag).await;
    admin.io.start_receiving().await.unwrap();
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    let (dead, listening) = journey::soon_dead(admin.device_key).await;
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, dead).await;
    listening.close().await;
    let mut kept = journey::removed_over_transport(&admin, &bob).await;
    assert_eq!(kept.len(), 1, "one recipient for one other Device");
    let (recipient, body) = kept.remove(0);
    assert_eq!(recipient, admin.device_key);
    (admin, bob, body)
}

/// Wait, within `NET`, until `member`'s store keeps no acknowledgement.
async fn none_kept(member: &journey::Joining) {
    tokio::time::timeout(NET, async {
        while !journey::kept_acknowledgement_bodies(member).is_empty() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("org-node deleted the acknowledgement in time");
}

// The removed Member's own delivery task, which holds the Admin's addresses
// from the Invite, connects directly from the revoked Device's identity; the
// Admin's receive step accepts the acknowledgement, and the task reads the
// verdict `0x00` back: only `Accepted` has org-node delete the
// acknowledgement (LLR-6ux6yw), so its deletion shows that byte was read.
// The delivery's normal path to a real recipient: started by the
// verification that removes the Device (LLR-zzsqj4), and the
// acknowledgement deleted once accepted (LLR-6ux6yw).
// verifies: LLR-j467e9, LLR-hhgdk7, LLR-zzsqj4, LLR-6ux6yw
#[tokio::test(flavor = "multi_thread")]
async fn a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "ack-in-accepted";
    let admin = journey::admin(tag).await;
    admin.io.start_receiving().await.unwrap();
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    journey::revoke(&admin, bob.device_key).await;
    let (notice, _) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert!(matches!(notice, Ok(Received::RevocationNoticeKept { .. })), "{notice:?}");
    let (received, verified) = tokio::join!(journey::next(&admin.io), bob.io.verify_all());
    assert!(matches!(verified.as_slice(), [(_, Verified::Removed { .. })]), "{verified:?}");
    assert!(matches!(received, Ok(Received::Acknowledged(_))), "{received:?}");
    none_kept(&bob).await;
}

// Abnormal: from a Device no record lists any more, Organisation
// information is refused by the sender gate (`SenderNotListed`), and an
// admission request carrying another Device's key (Carol's, for her own
// outstanding invite) because its key is not the sender's
// (`AdmissionKeyNotSender`); each is answered with one `0x01`, and nothing is
// kept.
// verifies: LLR-j467e9, LLR-hhgdk7
#[tokio::test(flavor = "multi_thread")]
async fn a_revoked_device_sending_anything_but_an_acknowledgement_or_an_organisation_request_is_refused() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "ack-in-anything-else";
    let (admin, bob, _) = ack_removed(tag).await;
    let information = journey::current_messages(&admin).into_iter().next().map(|(_, body)| body);
    let carol = journey::invited(&admin, tag, "c", "carol").await;
    let stale_request = common::handles::reopened_from(&carol.dir, &carol.io).build_admission_request(&carol.device_key, vec![]).unwrap().body;
    let files = store_bytes(tag, "a");
    let (received, read_back) = journey::dialled(&admin.io, &bob.device_key, &information.expect("the epoch's information")).await;
    assert!(matches!(received, Err(ReceiveError::Refused(OrgNodeError::SenderNotListed { org_id })) if org_id == admin.org), "{received:?}");
    assert_eq!(read_back, vec![0x01]);
    let (received, read_back) = journey::dialled(&admin.io, &bob.device_key, &stale_request).await;
    assert!(matches!(received, Err(ReceiveError::Refused(OrgNodeError::AdmissionKeyNotSender { .. }))), "{received:?}");
    assert_eq!(read_back, vec![0x01]);
    assert_eq!(store_bytes(tag, "a"), files, "nothing kept");
}

// Abnormal: a Member that has not adopted the
// removing epoch still lists the Device, and refuses its acknowledgement.
// verifies: LLR-j467e9, LLR-hhgdk7
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_a_member_still_listing_the_device_refuses_is_answered_refused() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "ack-in-still-listed";
    let admin = journey::admin(tag).await;
    admin.io.start_receiving().await.unwrap();
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    // Dead addresses for the Admin: Bob's own delivery task reaches no one (the first adoption keeps the Invite issuer's live addresses).
    let (dead, listening) = journey::soon_dead(admin.device_key).await;
    journey::keep_addresses(&bob.io, admin.org, &admin.io, admin.device_key, dead).await;
    listening.close().await;
    let carol = journey::invited(&admin, tag, "c", "carol").await;
    carol.io.start_receiving().await.unwrap();
    let (kept, _) = journey::request_over_transport(&admin, &carol).await;
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
    journey::propose_admission(&admin, &carol).await;
    journey::write_and_adopt(&admin).await;
    let (to_bob, to_carol, _) = tokio::join!(journey::next(&bob.io), journey::next(&carol.io), admin.io.send_organisation(admin.org));
    assert!(
        matches!(
            (&to_bob, &to_carol),
            (Ok(Received::ProvisionalOrganisationReceived { .. }), Ok(Received::ProvisionalOrganisationReceived { .. }))
        ),
        "{to_bob:?} {to_carol:?}"
    );
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Adopted { .. })]));
    assert!(matches!(carol.io.verify_all().await.as_slice(), [(_, Verified::Adopted { .. })]));
    journey::revoke(&admin, bob.device_key).await;
    let (notice, information, _) = tokio::join!(journey::next(&bob.io), journey::next(&carol.io), admin.io.send_organisation(admin.org));
    assert!(matches!(notice, Ok(Received::RevocationNoticeKept { .. })), "{notice:?}");
    assert!(matches!(information, Ok(Received::ProvisionalOrganisationReceived { .. })), "kept by Carol, not adopted: {information:?}");
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Removed { .. })]));
    // One acknowledgement, signed once, goes to each recipient org-node chose
    // (here one of the two, by key order): its body names no recipient.
    let (_, body) = journey::kept_acknowledgement_bodies(&bob).into_iter().next().unwrap();
    let (received, read_back) = journey::dialled(&carol.io, &bob.device_key, &body).await;
    assert!(matches!(received, Err(ReceiveError::Refused(_))), "Carol's record still lists Bob: {received:?}");
    assert_eq!(read_back, vec![0x01]);
}

// Abnormal: an acknowledgement relayed by another Device is refused.
// verifies: LLR-j467e9
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_relayed_by_another_device_is_refused() {
    let (_guard, _keychain) = fresh_keychain();
    let (admin, _bob, body) = ack_removed("ack-in-relayed").await;
    let (received, read_back) = journey::dialled(&admin.io, &keep_device_seed([0x63; 32]), &body).await;
    assert!(matches!(received, Err(ReceiveError::Refused(_))), "{received:?}");
    assert_eq!(read_back, vec![0x01]);
}

// Abnormal: a body taken off the channel and never
// answered (its responder dropped) gets no byte back.
// verifies: LLR-hhgdk7
#[tokio::test(flavor = "multi_thread")]
async fn a_dropped_responder_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let (admin, bob, body) = ack_removed("ack-in-dropped").await;
    let addr = journey::address_of(&admin.io).await;
    let (taken, read_back) = tokio::join!(
        tokio::time::timeout(NET, admin.io.next_inbound_for_test()),
        tokio::time::timeout(NET, dial_direct_as(&bob.device_key, addr, &body))
    );
    let read_back = read_back.expect("the dial ended in time");
    assert_eq!(taken.expect("a body taken in time").unwrap().0, bob.device_key);
    // The dropped responder drops the connection: the dialler reads no byte,
    // seeing either the stream's end or the connection lost.
    assert!(read_back.as_ref().map_or(true, Vec::is_empty), "nothing written: {read_back:?}");
    assert!(matches!(decode_message(&body).unwrap(), WireMessage::Acknowledgement(_)));
}

// ---- source scans ----------------------------------------------------------------

/// The receive side's sources: the transport module, the endpoints and their
/// accept tasks, and the receive step.
fn receive_side_sources() -> Vec<(String, String)> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut paths = vec![src.join("receiver.rs"), src.join("endpoints.rs")];
    for entry in std::fs::read_dir(src.join("transport")).unwrap() {
        paths.push(entry.unwrap().path());
    }
    paths
        .into_iter()
        .map(|path| (path.display().to_string(), std::fs::read_to_string(&path).unwrap()))
        .collect()
}

/// A source's code with `//` comments (doc comments included) removed.
fn code(text: &str) -> String {
    text.lines().map(|line| line.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n")
}

/// `text` with the lines of `fn name` (from its signature to the closing
/// brace at the signature's indentation) blanked, so line numbers hold.
fn without_fn(text: &str, name: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let Some(start) = lines.iter().position(|line| line.contains(&format!("fn {name}("))) else { return text.to_string() };
    let indent = lines[start].len() - lines[start].trim_start().len();
    let end = (start + 1..lines.len())
        .find(|&index| lines[index].len() - lines[index].trim_start().len() == indent && lines[index].trim() == "}")
        .unwrap_or(lines.len() - 1);
    lines.iter().enumerate().map(|(index, line)| if (start..=end).contains(&index) { "" } else { line }).collect::<Vec<_>>().join("\n")
}

// No accept-time check of the remote id against a record: the gate is
// org-node's `check_received`; and the accept is not filtered by whether
// the path is direct or relayed.
// verifies: LLR-j467e9
#[test]
fn the_accept_path_has_no_allow_list() {
    let forbidden = [
        // a record or its members, consulted on the receive side
        "list_orgs",
        "trie_members",
        "device_keys.contains",
        "list_personas",
        "is_listed",
        "allow_list",
        "allowlist",
        "denylist",
        "blocklist",
        // the path a connection took
        "remote_info",
        "conn_type",
        "ConnectionType",
        "paths()",
        "is_relay",
        "is_direct",
        "Source::Relay",
        "addr_filter",
    ];
    let mut found = vec![];
    for (path, text) in receive_side_sources() {
        // `bind_from_keychain`'s test-only relay plan filters what it
        // dials, never what it accepts; `start_receiving` lists the node's
        // own Personas to bind their endpoints.
        let text = without_fn(&without_fn(&code(&text), "bind_from_keychain"), "start_receiving");
        for (index, line) in text.lines().enumerate() {
            for token in forbidden {
                if line.contains(token) {
                    found.push(format!("{path}:{}: `{token}`: {line}", index + 1));
                }
            }
        }
    }
    assert!(found.is_empty(), "the accept path filters connections:\n{}", found.join("\n"));
    let endpoint = code(&std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/transport/endpoint.rs")).unwrap());
    assert!(endpoint.contains("self.inner.accept()"), "the scan found the accept call");
}

// The receive side's only write to an inbound stream is the one-byte
// verdict, followed by `finish`; the sending paths (`send_framed`, and
// `send_for_verdict`, the dial that reads the verdict back) are the only
// other writes in these files.
// verifies: LLR-hhgdk7
#[test]
fn nothing_but_the_verdict_is_written_on_an_inbound_stream() {
    let writes = [".write(", ".write_all(", ".write_chunk", ".write_all_chunks(", "open_uni", "open_bi"];
    let mut verdict_writes = vec![];
    let mut other_writes = vec![];
    for (path, text) in receive_side_sources() {
        let text = without_fn(&without_fn(&code(&text), "send_framed"), "send_for_verdict");
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let verdict_write = "write_all(&[verdict.as_byte()])";
            if let Some(at) = line.find(verdict_write) {
                let rest = format!("{}\n{}", &line[at + verdict_write.len()..], lines[index + 1..].join("\n"));
                let finished = rest.find(".finish()").is_some_and(|at| !rest[..at].contains(".write"));
                assert!(finished, "{path}:{}: the verdict write is not followed by `finish`", index + 1);
                verdict_writes.push(format!("{path}:{}", index + 1));
            } else if writes.iter().any(|write| line.contains(write)) {
                other_writes.push(format!("{path}:{}: {line}", index + 1));
            }
        }
    }
    assert!(other_writes.is_empty(), "another write on the receive side:\n{}", other_writes.join("\n"));
    assert_eq!(verdict_writes.len(), 1, "exactly one verdict write: {verdict_writes:?}");
}

// Normal: the Admin lists Bob as removed once its epoch removing him is
// adopted, with Bob's MemberId and no acknowledgement, and with the epoch
// Bob's acknowledgement names once Bob's own delivery has been accepted by
// the Admin's receive step. Each query reads no chain and writes nothing
// to the Admin's store. Abnormal: an Organisation no Persona holds is
// `NotHeld`.
// verifies: LLR-3ydu9e
#[tokio::test(flavor = "multi_thread")]
async fn the_admin_lists_a_removed_device_and_then_its_acknowledgement() {
    use org_io::test_support::store_writes_to;
    use org_io::{RemovedDevice, RemovedDevicesRefused};
    let (_guard, _keychain) = fresh_keychain();
    let tag = "ack-in-removed-list";
    let admin = journey::admin(tag).await;
    admin.io.start_receiving().await.unwrap();
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    let (admin, touched) = (&admin, &|| (admin.chain.reads(), store_writes_to(&admin.dir, &admin.device_key).len()));
    let removed_devices = move |org_id| async move {
        let before = touched();
        let listed = admin.io.removed_devices(org_id).await;
        assert_eq!(touched(), before, "the query read no chain and wrote no store");
        listed
    };
    assert_eq!(removed_devices(admin.org).await.unwrap(), vec![], "none removed yet");
    let bob_member = journey::member_id_of(admin, bob.device_key).await;
    journey::revoke(admin, bob.device_key).await;
    let unacknowledged = RemovedDevice { member_id: bob_member, device_key: bob.device_key, acknowledged_epoch: None };
    assert_eq!(removed_devices(admin.org).await.unwrap(), vec![unacknowledged.clone()]);
    let (notice, _) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert!(matches!(notice, Ok(Received::RevocationNoticeKept { .. })), "{notice:?}");
    let (received, verified) = tokio::join!(journey::next(&admin.io), bob.io.verify_all());
    assert!(matches!(verified.as_slice(), [(_, Verified::Removed { .. })]), "{verified:?}");
    let Ok(Received::Acknowledged(acknowledgement)) = received else { panic!("{received:?}") };
    let acknowledgement = acknowledgement.into_inner();
    assert_eq!((acknowledgement.member_id, acknowledgement.device), (bob_member, bob.device_key));
    assert_eq!(
        removed_devices(admin.org).await.unwrap(),
        vec![RemovedDevice { acknowledged_epoch: Some(acknowledgement.epoch), ..unacknowledged }]
    );
    assert!(matches!(
        removed_devices(org_node::OrgId::new([0xee; 20])).await,
        Err(RemovedDevicesRefused::NotHeld { .. })
    ));
}
