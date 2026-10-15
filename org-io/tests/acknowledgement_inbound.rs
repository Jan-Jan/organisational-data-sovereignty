#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! A revoked Device may connect, for its acknowledgement only (REQ-ur7z4a;
//! S4, task T11a of docs/plans/2026-10-06-org-io-transport.md). The accept
//! path takes every inbound connection whose handshake completes and hands
//! org-node the body with its authenticated sender, so only a revoked
//! Device's acknowledgement is acted on (LLR-wvgec4); every inbound stream
//! is answered with one verdict byte, accepted or refused, and nothing else
//! (LLR-unqfj5).
//!
//! The revoked Device is driven by `dial_direct_as`, a relay-free test
//! dialler bound from that Device's keychain item: the sending side is
//! S3b-io's, not merged yet. Every test runs over fresh directories and a
//! fresh in-memory keychain, one test at a time under `fresh_keychain`'s
//! lock.

use std::path::{Path, PathBuf};
use std::time::Duration;

use org_node::revocation::{Acknowledgement, RevocationNotice};
use org_node::wire::{decode_message, encode_message, WireMessage};
use org_node::{DevicePublicKey, MemberId, OrgId, OrgNodeError, PersonaId};
use org_io::store_file::store_path;
use org_io::test_support::{dial_direct_as, FakeChain};
use org_io::transport::Verdict;
use org_io::{OrgIo, ReceiveError, Received};
use rand::rngs::OsRng;

mod common;
use common::handles::{at, create, fresh_keychain, handle, joiner_of, receiving_addr, refusal, store_dir, NET};

/// A founded an Organisation and admitted Bob (B) and then Carol (C); A
/// then removed Bob, and B accepted the notice and signed its
/// acknowledgement. A holds Bob's Device as revoked; C, which never saw the
/// removal, still lists it. Bob's Device secret stays in the keychain.
struct Trio {
    a: OrgIo,
    chain: FakeChain,
    c: OrgIo,
    org: OrgId,
    device_bob: DevicePublicKey,
    device_carol: DevicePublicKey,
    /// A's direct (bound-socket) loopback address.
    direct_a: iroh::EndpointAddr,
    /// C's direct (bound-socket) loopback address.
    direct_c: iroh::EndpointAddr,
    a_dir: PathBuf,
    acknowledgement: Acknowledgement,
    /// A's admission of Carol, as A's Device sent it.
    admission_of_carol: WireMessage,
    /// A revocation notice naming Alice's Member and Device, which B built
    /// from the notice A sent it.
    forged_notice_to_alice: WireMessage,
}

/// What A's `submit_commit_send` of `update` sends to `recipient`, captured
/// before org-node sees it. *S4 task T12, 2026-10-13:* a Loopback send goes
/// only to a recipient's address and `submit_commit_send` names no
/// recipient, so the body is sent to the recipient's own node at `addr` and
/// taken from its receive channel there, unanswered (`next_inbound_for_test`),
/// in place of a sink bound with another key: a second endpoint of the
/// recipient's key would share its identity, and A's endpoint reaches an
/// identity at any address it learnt for it.
async fn captured(a: &OrgIo, update: &org_io::BuiltUpdate, recipient: &OrgIo, addr: &iroh::EndpointAddr) -> WireMessage {
    let mut rng = OsRng;
    let (sent, inbound) = tokio::join!(a.submit_commit_send(&mut rng, update, at(addr.clone())), async {
        tokio::time::timeout(NET, recipient.next_inbound_for_test()).await.unwrap().unwrap()
    });
    assert!(sent.unwrap().sends.per_device.iter().any(|(_, outcome)| outcome.is_ok()), "sent");
    decode_message(&inbound.1).unwrap()
}

/// `receiver`'s next result while A's Device sends it `message` at `addr`.
async fn forwarded_from_a(a: &OrgIo, alice: &PersonaId, receiver: &OrgIo, addr: iroh::EndpointAddr, message: &WireMessage) -> Received {
    let endpoint_a = a.endpoint_for_test(alice).await.unwrap();
    let (received, ()) = tokio::join!(async { tokio::time::timeout(NET, receiver.next_received(&mut OsRng)).await.unwrap() }, async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        tokio::time::timeout(NET, endpoint_a.send(addr, message)).await.unwrap().unwrap();
    });
    received.unwrap()
}

async fn trio(tag: &str) -> Trio {
    let chain = FakeChain::new();
    let a = handle(tag, "a", &chain);
    let alice = create(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    let b = handle(tag, "b", &FakeChain::sharing_slots_with(&chain));
    let bob = create(&b, "bob").await;
    b.expect_admission(&mut OsRng, &bob, org).await.unwrap();
    let c = handle(tag, "c", &FakeChain::sharing_slots_with(&chain));
    let carol = create(&c, "carol").await;
    c.expect_admission(&mut OsRng, &carol, org).await.unwrap();
    let addr_b = receiving_addr(&b).await;
    let addr_c = receiving_addr(&c).await;
    let (joiner_bob, joiner_carol) = (joiner_of(&b, &bob), joiner_of(&c, &carol));
    let (device_bob, device_carol) = (joiner_bob.device_key, joiner_carol.device_key);

    let update = a.admit_member(&mut OsRng, org, &joiner_bob).await.unwrap();
    let admission_of_bob = captured(&a, &update, &b, &addr_b).await;
    forwarded_from_a(&a, &alice, &b, addr_b.clone(), &admission_of_bob).await;
    let update = a.admit_member(&mut OsRng, org, &joiner_carol).await.unwrap();
    let admission_of_carol = captured(&a, &update, &c, &addr_c).await;
    forwarded_from_a(&a, &alice, &c, addr_c, &admission_of_carol).await;

    let device_alice = a.persona_public_keys(&alice).await.unwrap().1;
    let member_id_of = |device: &DevicePublicKey| -> MemberId {
        let node = a.node_for_test();
        let record = node.list_orgs().into_iter().find(|record| record.org_id == org).unwrap();
        record.trie_members.iter().find(|member| member.device_keys.contains(device)).unwrap().id
    };
    let (member_id_of_alice, member_id_of_bob) = (member_id_of(&device_alice), member_id_of(&device_bob));
    let update = a.revoke_member(&mut OsRng, org, member_id_of_bob).await.unwrap();
    let notice_to_bob = captured(&a, &update, &b, &addr_b).await;
    let acknowledgement = match forwarded_from_a(&a, &alice, &b, addr_b, &notice_to_bob).await {
        Received::Committed(outcome) => outcome.acknowledgements.into_iter().next().expect("B signed its acknowledgement"),
        other => panic!("B did not commit its removal: {other:?}"),
    };
    // A notice carries no signature: B, revoked, can name Alice in one, with
    // the proof of its own notice.
    let WireMessage::Revocation(notice) = notice_to_bob else { panic!("not a notice: {notice_to_bob:?}") };
    let forged_notice_to_alice =
        WireMessage::Revocation(RevocationNotice { member_id: member_id_of_alice, device: device_alice, ..notice });

    a.start_receiving().await.unwrap();
    let direct_a = a.endpoint_for_test(&alice).await.unwrap().node_addr_for_dial();
    let direct_c = c.endpoint_for_test(&carol).await.unwrap().node_addr_for_dial();
    Trio {
        a,
        chain,
        c,
        org,
        device_bob,
        device_carol,
        direct_a,
        direct_c,
        a_dir: store_dir(tag, "a"),
        acknowledgement,
        admission_of_carol,
        forged_notice_to_alice,
    }
}

/// `receiver`'s next result, and every byte the dialler read back, while
/// `dialler`'s Device dials `addr` directly and sends `body`.
async fn dialled(receiver: &OrgIo, dialler: &DevicePublicKey, addr: iroh::EndpointAddr, body: &[u8]) -> (Result<Received, ReceiveError>, Vec<u8>) {
    let (received, read_back) = tokio::join!(async { tokio::time::timeout(NET, receiver.next_received(&mut OsRng)).await.unwrap() }, async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        tokio::time::timeout(NET, dial_direct_as(dialler, addr, body)).await.unwrap().unwrap()
    });
    (received, read_back)
}

fn store_bytes(dir: &Path, device_key: &DevicePublicKey) -> Vec<u8> {
    std::fs::read(store_path(dir, device_key)).unwrap()
}

fn body_of(message: &WireMessage) -> Vec<u8> {
    encode_message(message).unwrap()
}

// ---- LLR-wvgec4, LLR-unqfj5: the revoked Device's acknowledgement --------------

// B, revoked, dials A's direct loopback address with no relay and sends its
// acknowledgement: A settles it (no store write), and B reads exactly one
// byte, the accepted verdict, then the end of the stream.
// verifies: LLR-wvgec4, LLR-unqfj5
#[tokio::test(flavor = "multi_thread")]
async fn a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted() {
    let (_guard, _keychain) = fresh_keychain();
    let trio = trio("ack-inbound-accepted").await;
    let device_alice = trio.a.persona_public_keys(&trio.a.personas().await[0].persona_id).await.unwrap().1;
    let (reads_before, bytes_before) = (trio.chain.reads(), store_bytes(&trio.a_dir, &device_alice));
    let body = body_of(&WireMessage::Acknowledgement(trio.acknowledgement.clone()));
    let (received, read_back) = dialled(&trio.a, &trio.device_bob, trio.direct_a.clone(), &body).await;
    match received {
        Ok(Received::Acknowledged(verified)) => {
            let acknowledgement = verified.into_inner();
            assert_eq!((acknowledgement.device, acknowledgement.org_id), (trio.device_bob, trio.org), "B's Device, for the Organisation");
        }
        other => panic!("not settled as an acknowledgement: {other:?}"),
    }
    assert_eq!(read_back, vec![Verdict::Accepted.as_byte()], "one byte, accepted, then end of stream");
    assert_eq!(Verdict::Accepted.as_byte(), 0x00);
    assert_eq!(trio.chain.reads(), reads_before, "no chain read");
    assert_eq!(store_bytes(&trio.a_dir, &device_alice), bytes_before, "nothing written to A's store file");
}

// From B's identity, an update for the Organisation, a revocation notice and
// bytes that do not decode are each refused before any chain read or store
// write, and B reads the refused verdict and nothing else.
// verifies: LLR-wvgec4, LLR-unqfj5
#[tokio::test(flavor = "multi_thread")]
async fn a_revoked_device_sending_anything_but_an_acknowledgement_is_refused_and_served_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let trio = trio("ack-inbound-other-kinds").await;
    let device_alice = trio.a.persona_public_keys(&trio.a.personas().await[0].persona_id).await.unwrap().1;
    let (reads_before, bytes_before) = (trio.chain.reads(), store_bytes(&trio.a_dir, &device_alice));
    let cases: [(&str, Vec<u8>); 3] = [
        ("an update (OrgInformation)", body_of(&trio.admission_of_carol)),
        ("a revocation notice naming A's Device", body_of(&trio.forged_notice_to_alice)),
        ("bytes that do not decode", vec![0xff; 7]),
    ];
    for (case, body) in cases {
        let (received, read_back) = dialled(&trio.a, &trio.device_bob, trio.direct_a.clone(), &body).await;
        let refused = refusal(received);
        match case {
            "bytes that do not decode" => assert!(matches!(refused, OrgNodeError::MalformedMessage), "{case}: {refused:?}"),
            _ => assert!(matches!(refused, OrgNodeError::SenderNotListed { org_id } if org_id == trio.org), "{case}: {refused:?}"),
        }
        assert_eq!(read_back, vec![Verdict::Refused.as_byte()], "{case}: one byte, refused, and nothing else");
        assert_eq!(trio.chain.reads(), reads_before, "{case}: no chain read");
        assert_eq!(store_bytes(&trio.a_dir, &device_alice), bytes_before, "{case}: A's store file unchanged");
    }
    assert_eq!(Verdict::Refused.as_byte(), 0x01);
}

// C never committed Bob's removal, so it still lists Bob's Device: it
// refuses the acknowledgement, and the refused verdict tells B's sender to
// try its next kept address. org-node judges the epoch before the listing,
// so a Member behind the removal refuses the acknowledgement as naming an
// epoch after its record's (the plan named the listing refusal; S4, task
// T11a, 2026-10-13).
// verifies: LLR-wvgec4, LLR-unqfj5
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_a_member_still_listing_the_device_refuses_is_answered_refused() {
    let (_guard, _keychain) = fresh_keychain();
    let trio = trio("ack-inbound-still-listed").await;
    let still_listed = {
        let node = trio.c.node_for_test();
        let record = node.list_orgs().into_iter().find(|record| record.org_id == trio.org).unwrap();
        record.trie_members.iter().any(|member| member.device_keys.contains(&trio.device_bob))
    };
    assert!(still_listed, "C still lists Bob's Device");
    let body = body_of(&WireMessage::Acknowledgement(trio.acknowledgement.clone()));
    let (received, read_back) = dialled(&trio.c, &trio.device_bob, trio.direct_c.clone(), &body).await;
    let refused = refusal(received);
    assert!(matches!(refused, OrgNodeError::AcknowledgementFromFuture { org_id } if org_id == trio.org), "{refused:?}");
    assert_eq!(read_back, vec![Verdict::Refused.as_byte()]);
}

// C's Device relays Bob's acknowledgement to A: refused, as an
// acknowledgement not from its own Device.
// verifies: LLR-wvgec4
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_relayed_by_another_device_is_refused() {
    let (_guard, _keychain) = fresh_keychain();
    let trio = trio("ack-inbound-relayed").await;
    let body = body_of(&WireMessage::Acknowledgement(trio.acknowledgement.clone()));
    let (received, read_back) = dialled(&trio.a, &trio.device_carol, trio.direct_a.clone(), &body).await;
    let refused = refusal(received);
    assert!(matches!(refused, OrgNodeError::AcknowledgementNotFromItsDevice { org_id } if org_id == trio.org), "{refused:?}");
    assert_eq!(read_back, vec![Verdict::Refused.as_byte()]);
}

// A body whose responder is dropped unanswered (here: taken off the receive
// channel before the receive step) is answered with nothing: the dialler
// reads no byte, which its sender treats as a refusal.
// verifies: LLR-unqfj5
#[tokio::test(flavor = "multi_thread")]
async fn a_dropped_responder_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let trio = trio("ack-inbound-dropped").await;
    let body = body_of(&WireMessage::Acknowledgement(trio.acknowledgement.clone()));
    let (taken, read_back) = tokio::join!(async { tokio::time::timeout(NET, trio.a.next_inbound_for_test()).await.unwrap() }, async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        tokio::time::timeout(NET, dial_direct_as(&trio.device_bob, trio.direct_a.clone(), &body)).await.unwrap()
    });
    assert_eq!(taken.unwrap(), (trio.device_bob, body), "the body reached the receive channel");
    // An error is the stream or the connection ending with nothing written.
    if let Ok(bytes) = read_back {
        assert!(bytes.is_empty(), "no byte written, read {bytes:?}");
    }
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
// verifies: LLR-wvgec4
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
// verdict, followed by `finish`; the sending path (`send_framed`) is the
// only other write in these files.
// verifies: LLR-unqfj5
#[test]
fn nothing_but_the_verdict_is_written_on_an_inbound_stream() {
    let writes = [".write(", ".write_all(", ".write_chunk", ".write_all_chunks(", "open_uni", "open_bi"];
    let mut verdict_writes = vec![];
    let mut other_writes = vec![];
    for (path, text) in receive_side_sources() {
        let text = without_fn(&code(&text), "send_framed");
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
