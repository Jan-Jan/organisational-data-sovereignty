#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The receive step (SDD-2fc2cq): `next_received` waits for the next body
//! with no lock on the node held (LLR-eeh4pq); it hands each body and its
//! authenticated sender to org-node's `check_received`, writes the store
//! change before the one verdict byte, reports what was kept (public data
//! only), reads no chain and adopts nothing (LLR-kz9hj4); it answers an
//! organisation request through the send path (LLR-898z24); and the end of
//! the receive loop is a typed terminal condition, and nothing else is
//! (LLR-szhe7j).
//!
//! Every test runs over fresh directories and a fresh in-memory keychain,
//! one test at a time under `fresh_keychain`'s lock: keyring's default
//! credential builder is process-global, and one test locks the keychain.

use std::sync::Arc;
use std::time::Duration;

use org_io::send::Delivery;
use org_io::test_support::{dial_direct_as, store_writes_to, FakeChain, Refusal};
use org_io::{OrgIo, ReceiveError, Received, TerminalCause};
use org_node::wire::{encode_message, WireMessage};
use org_node::OrgNodeError;

mod common;
use common::handles::{bind_seed, create, fresh_keychain, handle, keep_device_seed, receiving_addr, store_bytes, store_dir, NET};
use common::journey;

/// The bytes `body` arrives as at `addr`, sent from `endpoint`.
async fn send_raw(endpoint: &org_io::transport::OrgEndpoint, addr: iroh::EndpointAddr, body: &[u8]) {
    tokio::time::timeout(NET, endpoint.send_body(addr, body)).await.unwrap().unwrap();
}

/// One handle receiving on one Persona's endpoint: the address a stranger sends to.
async fn receiving(tag: &str) -> (OrgIo, iroh::EndpointAddr) {
    let io = handle(tag, "b", &FakeChain::new());
    create(&io, "bob").await;
    let addr = receiving_addr(&io).await;
    (io, addr)
}

/// What `io` receives while `send` runs.
async fn received_while<F: std::future::Future<Output = ()>>(io: &OrgIo, send: F) -> Result<Received, ReceiveError> {
    let (received, ()) = tokio::join!(async { tokio::time::timeout(NET, io.next_received()).await.unwrap() }, send);
    received
}

// ---- LLR-eeh4pq: no lock held while the receive step waits

// A command that needs the node, a query and a mutating call, completes
// while `next_received` waits with no peer sending (the reproduction of
// the app's receiver holding the lock, PR-7v8anh).
// verifies: LLR-eeh4pq, PR-7v8anh
#[tokio::test(flavor = "multi_thread")]
async fn a_command_completes_while_the_receiver_waits() {
    let (_guard, _keychain) = fresh_keychain();
    let io = Arc::new(handle("step-waits", "a", &FakeChain::new()));
    create(&io, "alice").await;
    io.start_receiving().await.unwrap();
    let receiver = {
        let io = io.clone();
        tokio::spawn(async move { io.next_received().await })
    };
    tokio::time::sleep(Duration::from_millis(100)).await;
    let commands = async {
        assert!(io.organisations().await.is_empty());
        create(&io, "bob").await;
        io.personas().await.len()
    };
    let personas = tokio::time::timeout(Duration::from_secs(2), commands).await.expect("the commands waited on the receiver");
    assert_eq!(personas, 2);
    assert!(!receiver.is_finished(), "the receiver is still waiting");
    receiver.abort();
}

// ---- LLR-szhe7j: the terminal condition is typed -----------------------------

// verifies: LLR-szhe7j
#[tokio::test(flavor = "multi_thread")]
async fn a_closed_endpoint_is_terminal_endpoint_closed() {
    let (_guard, _keychain) = fresh_keychain();
    let io = handle("step-closed", "a", &FakeChain::new());
    let alice = create(&io, "alice").await;
    io.start_receiving().await.unwrap();
    io.endpoint_for_test(&alice).await.unwrap().inner().close().await;
    let received = tokio::time::timeout(NET, io.next_received()).await.unwrap();
    assert!(matches!(received, Err(ReceiveError::Terminal(TerminalCause::EndpointClosed))), "{received:?}");
    // Boundary: a node that accepts on no endpoint has nothing to wait for.
    let idle = handle("step-closed", "b", &FakeChain::new());
    let received = tokio::time::timeout(NET, idle.next_received()).await.unwrap();
    assert!(matches!(received, Err(ReceiveError::Terminal(TerminalCause::EndpointClosed))), "{received:?}");
}

// Abnormal: the keychain refuses the bind.
// verifies: LLR-szhe7j
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_bind_is_terminal_bind_failed() {
    let (_guard, keychain) = fresh_keychain();
    let io = handle("step-bind-failed", "a", &FakeChain::new());
    create(&io, "alice").await;
    keychain.refuse_as(Some(Refusal::Locked));
    let started = io.start_receiving().await;
    keychain.refuse_as(None);
    assert!(matches!(started, Err(ReceiveError::Terminal(TerminalCause::BindFailed(_)))), "{started:?}");
}

// Abnormal: bytes that do not decode are org-node's refusal, and the receive step goes on to
// the next body, refused the same way.
// verifies: LLR-szhe7j
#[tokio::test(flavor = "multi_thread")]
async fn a_malformed_body_is_not_terminal() {
    let (_guard, _keychain) = fresh_keychain();
    let (io, addr) = receiving("step-malformed").await;
    let stranger = bind_seed([0x5c; 32]).await;
    let refused = received_while(&io, send_raw(&stranger, addr.clone(), b"not a wire message")).await;
    assert!(matches!(refused, Err(ReceiveError::Refused(OrgNodeError::MalformedMessage))), "{refused:?}");
    let again = received_while(&io, send_raw(&stranger, addr, b"still not one")).await;
    assert!(matches!(again, Err(ReceiveError::Refused(OrgNodeError::MalformedMessage))), "the next body is received: {again:?}");
}

// Abnormal: a connection that fails before it carries a body is `ReceiveError::Connection`,
// not terminal, and the receive step goes on to the next body.
// verifies: LLR-szhe7j
#[tokio::test(flavor = "multi_thread")]
async fn a_connection_failing_before_a_body_is_not_terminal() {
    let (_guard, _keychain) = fresh_keychain();
    let (io, addr) = receiving("step-connection-failed").await;
    let stranger = bind_seed([0x5d; 32]).await;
    let dial = addr.clone();
    let failed = received_while(&io, async {
        let connection = tokio::time::timeout(NET, stranger.inner().connect(dial, org_io::transport::ALPN)).await.unwrap().unwrap();
        connection.close(1u32.into(), b"no body");
    })
    .await;
    assert!(matches!(failed, Err(ReceiveError::Connection(_))), "{failed:?}");
    let next = received_while(&io, send_raw(&stranger, addr, b"not a wire message")).await;
    assert!(matches!(next, Err(ReceiveError::Refused(OrgNodeError::MalformedMessage))), "the next body is received: {next:?}");
}

// ---- the receive step over org-node's check ----------------------------------

/// A founded Admin of `tag`, receiving. The caller holds `fresh_keychain`'s lock.
async fn receiving_admin(tag: &str) -> journey::Admin {
    let admin = journey::admin(tag).await;
    admin.io.start_receiving().await.unwrap();
    admin
}

/// The admission request `joining`'s Device would send, carrying no
/// addresses: built by org-node from the store `joining`'s handle wrote, read
/// apart from the handle, and dialled by the test as that Device.
fn request_body_of(joining: &journey::Joining) -> org_node::service::OutgoingMessage {
    common::handles::reopened_from(&joining.dir, &joining.io).build_admission_request(&joining.device_key, vec![]).unwrap()
}

/// `member` (admitted, the Admin's only other Device) removed by `admin`;
/// its kept acknowledgement for the Admin. Before the removal it keeps dead
/// addresses for the Admin, so its own delivery task cannot reach the
/// Admin and the test's hand dial is the only sender (a first adoption
/// keeps the Invite issuer's live addresses).
async fn removed_with_ack_for_admin(admin: &journey::Admin, member: &journey::Joining) -> Vec<u8> {
    let (dead, soon_closed) = journey::soon_dead(admin.device_key).await;
    journey::keep_addresses(&member.io, admin.org, &admin.io, admin.device_key, dead).await;
    soon_closed.close().await;
    let acknowledgements = journey::removed_over_transport(admin, member).await;
    acknowledgements.into_iter().find(|(recipient, _)| *recipient == admin.device_key).unwrap().1
}

// Normal: the provisional Member keeps the admitting epoch the Admin sent:
// the store change is in its file when the report returns, and no chain
// read of any kind was made on the way.
// verifies: LLR-kz9hj4
#[tokio::test(flavor = "multi_thread")]
async fn next_received_writes_the_store_change_before_it_reports_and_reads_no_chain() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-writes-first";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    bob.io.start_receiving().await.unwrap();
    let (kept, _) = journey::request_over_transport(&admin, &bob).await;
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
    journey::propose_admission(&admin, &bob).await;
    journey::write_and_adopt(&admin).await;
    let writes = store_writes_to(&store_dir(tag, "b"), &bob.device_key).len();
    let (received, _) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert!(matches!(received, Ok(Received::ProvisionalOrganisationReceived { from, .. }) if from == admin.device_key), "{received:?}");
    assert_eq!(store_writes_to(&store_dir(tag, "b"), &bob.device_key).len(), writes + 1, "written before the report");
    assert_eq!((bob.chain.reads(), bob.chain.finalised_reads()), (0, 0), "no chain read on the receive path");
}

// The admission request's report is filled from the kept request (the same
// values `admission_requests` lists); a provisional organisation already
// held is accepted (`0x00`) and reported to no one.
// verifies: LLR-x36szv
#[tokio::test(flavor = "multi_thread")]
async fn received_reports_public_data_only_and_an_already_held_one_reports_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-reports";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    bob.io.start_receiving().await.unwrap();
    let (kept, _) = journey::request_over_transport(&admin, &bob).await;
    let listed = admin.io.admission_requests().await;
    assert_eq!(listed.len(), 1);
    match kept {
        Ok(Received::AdmissionRequestKept { org_id, invite_id, name, surname, preferred_handles, from, invitee_name }) => {
            assert_eq!((org_id, invite_id, from), (admin.org, bob.invite_id, bob.device_key));
            assert_eq!(
                (name, surname, preferred_handles, invitee_name),
                (listed[0].name.clone(), listed[0].surname.clone(), listed[0].preferred_handles.clone(), listed[0].invitee_name.clone())
            );
        }
        other => panic!("not kept: {other:?}"),
    }
    journey::propose_admission(&admin, &bob).await;
    journey::write_and_adopt(&admin).await;
    let (first, _) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert!(matches!(first, Ok(Received::ProvisionalOrganisationReceived { .. })), "{first:?}");
    let (silent, sent) =
        tokio::join!(tokio::time::timeout(Duration::from_secs(2), bob.io.next_received()), admin.io.send_organisation(admin.org));
    assert!(silent.is_err(), "an already held one is reported to no one: {silent:?}");
    assert!(matches!(sent.unwrap().per_device[0].1, Delivery::Delivered), "and answered 0x00");
}

// Two first admission requests for one Organisation, from two Invites: each
// report names its own invite and Device, read from the request kept for
// that invite and not from another one the store already keeps.
// verifies: LLR-x36szv, LLR-zkybw7
#[tokio::test(flavor = "multi_thread")]
async fn each_admission_report_is_filled_from_the_request_kept_for_its_own_invite() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-reports-own-invite";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    let carol = journey::invited(&admin, tag, "c", "carol").await;
    for joining in [&bob, &carol] {
        let (kept, _) = journey::request_over_transport(&admin, joining).await;
        match kept {
            Ok(Received::AdmissionRequestKept { org_id, invite_id, from, .. }) => {
                assert_eq!((org_id, invite_id, from), (admin.org, joining.invite_id, joining.device_key), "its own request");
            }
            other => panic!("not kept: {other:?}"),
        }
    }
    assert_eq!(admin.io.admission_requests().await.len(), 2, "both kept");
}

// Abnormal: the store write of a first admission request fails. The step
// returns `ReceiveError::Store`, writes `0x01`, and the Persona, reloaded
// from its file, keeps no request; the same request sent again is kept and
// answered `0x00`.
// verifies: LLR-kz9hj4
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_store_write_is_receive_error_store_answered_0x01_and_the_persona_reloaded() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-store-fails";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    let request = request_body_of(&bob);
    let files = store_bytes(tag, "a");
    org_io::test_support::fail_next_write_after_temporary();
    let (received, read_back) = journey::dialled(&admin.io, &bob.device_key, &request.body).await;
    assert!(matches!(received, Err(ReceiveError::Store(org_io::StoreFailure::Reloaded { .. }))), "{received:?}");
    assert_eq!(read_back, vec![0x01], "refused: nothing kept");
    assert!(admin.io.admission_requests().await.is_empty(), "reloaded from its file: no request held");
    assert_eq!(store_bytes(tag, "a"), files, "the file unchanged");
    let (again, read_back) = journey::dialled(&admin.io, &bob.device_key, &request.body).await;
    assert!(matches!(again, Ok(Received::AdmissionRequestKept { .. })), "{again:?}");
    assert_eq!(read_back, vec![0x00]);
}

// A Device no record lists, over a direct connection, is accepted at the
// transport and judged by org-node's check: its body comes back refused
// with one byte, not dropped at accept.
// verifies: LLR-j467e9
#[tokio::test(flavor = "multi_thread")]
async fn every_completed_handshake_is_accepted_with_no_allow_list() {
    let (_guard, _keychain) = fresh_keychain();
    let admin = receiving_admin("receive-no-allow-list").await;
    let stranger = keep_device_seed([0x5e; 32]);
    let (received, read_back) = journey::dialled(&admin.io, &stranger, b"not a wire message").await;
    assert!(matches!(received, Err(ReceiveError::Refused(OrgNodeError::MalformedMessage))), "{received:?}");
    assert_eq!(read_back, vec![0x01], "the check refused it; the accept path took it");
}

// One byte per message, whatever the outcome: `0x01` for a refusal, `0x00`
// for a first admission request, `0x02` for the same request again.
// verifies: LLR-hhgdk7
#[tokio::test(flavor = "multi_thread")]
async fn exactly_one_verdict_byte_is_written_per_message() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-one-byte";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    let request = request_body_of(&bob);
    let (_, refused) = journey::dialled(&admin.io, &bob.device_key, b"\xff").await;
    let (_, first) = journey::dialled(&admin.io, &bob.device_key, &request.body).await;
    let (_, again) = tokio::join!(tokio::time::timeout(Duration::from_secs(2), admin.io.next_received()), async {
        dial_direct_as(&bob.device_key, journey::address_of(&admin.io).await, &request.body).await.unwrap()
    });
    assert_eq!((refused, first, again), (vec![0x01], vec![0x00], vec![0x02]));
}

// The first request for an invite is written, then answered `0x00`; the same
// Device asking again gets `0x02` and another Device with the same Invite
// `0x03`, neither written nor reported.
// verifies: LLR-zkybw7
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_request_is_written_then_answered_0x00_and_repeats_0x02_or_0x03() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-admission-verdicts";
    let admin = receiving_admin(tag).await;
    let blob = admin.io.issue_invite(admin.org, "Bob").await.unwrap();
    let bob = handle(tag, "b", &FakeChain::sharing_slots_with(&admin.chain));
    bob.propose_member(&blob, "Bob", "Builder", &["bob"]).await.unwrap();
    let carol = handle(tag, "c", &FakeChain::sharing_slots_with(&admin.chain));
    carol.propose_member(&blob, "Carol", "Cook", &["carol"]).await.unwrap();
    let writes = store_writes_to(&store_dir(tag, "a"), &admin.device_key).len();

    let (kept, first) = tokio::join!(journey::next(&admin.io), bob.request_admission(admin.org));
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
    assert!(matches!(first.unwrap(), Delivery::Delivered));
    assert_eq!(store_writes_to(&store_dir(tag, "a"), &admin.device_key).len(), writes + 1, "written, then 0x00");
    let files = store_bytes(tag, "a");
    let (silent, again) =
        tokio::join!(tokio::time::timeout(Duration::from_secs(2), admin.io.next_received()), bob.request_admission(admin.org));
    assert!(silent.is_err(), "not reported");
    assert!(matches!(again.unwrap(), Delivery::AlreadyReceived));
    let (silent, other) =
        tokio::join!(tokio::time::timeout(Duration::from_secs(2), admin.io.next_received()), carol.request_admission(admin.org));
    assert!(silent.is_err(), "not reported");
    assert!(matches!(other.unwrap(), Delivery::InviteUsedByAnotherDevice));
    assert_eq!(store_bytes(tag, "a"), files, "neither written");
}

// While `next_received` waits with nothing arriving, a command that takes
// the node's lock completes.
// verifies: LLR-eeh4pq
#[tokio::test(flavor = "multi_thread")]
async fn a_command_completes_while_next_received_waits() {
    let (_guard, _keychain) = fresh_keychain();
    let admin = receiving_admin("receive-command-completes").await;
    tokio::select! {
        received = admin.io.next_received() => panic!("nothing was sent: {received:?}"),
        listed = tokio::time::timeout(Duration::from_secs(5), async {
            (admin.io.organisations().await, admin.io.issue_invite(admin.org, "Dora").await)
        }) => {
            let (organisations, invite) = listed.expect("the commands complete while next_received waits");
            assert_eq!(organisations.len(), 1);
            assert!(invite.is_ok(), "{invite:?}");
        }
    }
}

// Abnormal: a body org-node's check refuses leaves the node's lock released. Once
// `next_received` returned the refusal, a command that takes the lock completes while it waits
// again.
// verifies: LLR-eeh4pq
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_body_leaves_no_lock_held_for_the_next_wait() {
    let (_guard, _keychain) = fresh_keychain();
    let admin = receiving_admin("receive-refused-then-waits").await;
    let (received, _) = journey::dialled(&admin.io, &keep_device_seed([0x6a; 32]), b"not a wire message").await;
    assert!(matches!(received, Err(ReceiveError::Refused(OrgNodeError::MalformedMessage))), "{received:?}");
    tokio::select! {
        received = admin.io.next_received() => panic!("nothing more was sent: {received:?}"),
        listed = tokio::time::timeout(Duration::from_secs(5), admin.io.organisations()) => {
            assert_eq!(listed.expect("the command completes after the refusal").len(), 1);
        }
    }
}

// Abnormal: a body that is no wire message is refused by org-node's check,
// with no chain read and nothing written.
// verifies: LLR-kz9hj4
#[tokio::test(flavor = "multi_thread")]
async fn a_message_the_check_refuses_reads_no_chain_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-check-refuses";
    let admin = receiving_admin(tag).await;
    let files = store_bytes(tag, "a");
    let reads = admin.chain.reads();
    let (received, _) = journey::dialled(&admin.io, &keep_device_seed([0x5f; 32]), b"not a wire message").await;
    assert!(matches!(received, Err(ReceiveError::Refused(OrgNodeError::MalformedMessage))), "{received:?}");
    assert_eq!((admin.chain.reads(), store_bytes(tag, "a")), (reads, files), "no read, nothing written");
}

// Abnormal: the Admin's own epoch relayed to a Member by a Device the
// Member's record does not list is refused, with no chain read.
// verifies: LLR-kz9hj4
#[tokio::test(flavor = "multi_thread")]
async fn organisation_information_from_an_unlisted_device_is_refused_with_no_read() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-unlisted-sender";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    let (_, body) = journey::current_messages(&admin).into_iter().find(|(recipient, _)| *recipient == bob.device_key).unwrap();
    let reads = bob.chain.reads();
    let (received, read_back) = journey::dialled(&bob.io, &keep_device_seed([0x60; 32]), &body).await;
    assert!(matches!(received, Err(ReceiveError::Refused(OrgNodeError::SenderNotListed { .. }))), "{received:?}");
    assert_eq!((read_back, bob.chain.reads()), (vec![0x01], reads), "refused, and no read");
}

// A removed Member's acknowledgement is accepted with no chain read.
// verifies: LLR-kz9hj4
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_is_decided_with_no_read() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-ack-no-read";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    let body = removed_with_ack_for_admin(&admin, &bob).await;
    let reads = admin.chain.reads();
    let (received, read_back) = journey::dialled(&admin.io, &bob.device_key, &body).await;
    assert!(matches!(received, Ok(Received::Acknowledged(_))), "{received:?}");
    assert_eq!((read_back, admin.chain.reads()), (vec![0x00], reads));
}

// Abnormal: that acknowledgement dialled by another Device is refused, with
// no chain read.
// verifies: LLR-kz9hj4
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_from_another_device_is_refused_with_no_read() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-ack-relayed";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    let body = removed_with_ack_for_admin(&admin, &bob).await;
    let reads = admin.chain.reads();
    let (received, read_back) = journey::dialled(&admin.io, &keep_device_seed([0x61; 32]), &body).await;
    assert!(matches!(received, Err(ReceiveError::Refused(_))), "{received:?}");
    assert_eq!((read_back, admin.chain.reads()), (vec![0x01], reads));
}

// Abnormal: a refused check reads no chain.
// verifies: LLR-kz9hj4
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_check_reads_no_chain() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-refused-no-read";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    let (_, body) = journey::current_messages(&admin).into_iter().find(|(recipient, _)| *recipient == bob.device_key).unwrap();
    let other_chain = FakeChain::sharing_slots_with(&admin.chain);
    let other = handle(tag, "d", &other_chain);
    journey::propose_founding(&other, "dora").await;
    other.start_receiving().await.unwrap();
    let (received, _) = journey::dialled(&other, &admin.device_key, &body).await;
    assert!(matches!(received, Err(ReceiveError::Refused(_))), "a founding store keeps no epoch it never expected: {received:?}");
    assert_eq!(other_chain.reads(), 0, "no read on the receive path");
}

// Abnormal: org-node judges the sender the handshake
// authenticated — a request naming Bob's Device, dialled from another
// Device, is refused for that.
// verifies: LLR-kz9hj4
#[tokio::test(flavor = "multi_thread")]
async fn the_sender_org_node_sees_is_the_handshakes() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-handshake-sender";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    let request = request_body_of(&bob);
    let (received, _) = journey::dialled(&admin.io, &keep_device_seed([0x62; 32]), &request.body).await;
    assert!(matches!(received, Err(ReceiveError::Refused(OrgNodeError::AdmissionKeyNotSender { .. }))), "{received:?}");
    let (received, _) = journey::dialled(&admin.io, &bob.device_key, &request.body).await;
    assert!(matches!(received, Ok(Received::AdmissionRequestKept { from, .. }) if from == bob.device_key), "{received:?}");
}

// An acknowledgement is settled with no chain read and no store write.
// verifies: LLR-kz9hj4
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_is_settled_with_no_chain_read_and_no_store_write() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-ack-no-write";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    let body = removed_with_ack_for_admin(&admin, &bob).await;
    let (reads, writes) = (admin.chain.reads(), store_writes_to(&store_dir(tag, "a"), &admin.device_key).len());
    let (received, _) = journey::dialled(&admin.io, &bob.device_key, &body).await;
    assert!(matches!(received, Ok(Received::Acknowledged(_))), "{received:?}");
    assert_eq!((admin.chain.reads(), store_writes_to(&store_dir(tag, "a"), &admin.device_key).len()), (reads, writes));
}

// When the report returns, the store write has
// landed in the file, byte for byte.
// verifies: LLR-kz9hj4
#[tokio::test(flavor = "multi_thread")]
async fn the_store_write_lands_before_the_outcome_returns() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "receive-write-lands";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    let (kept, _) = journey::request_over_transport(&admin, &bob).await;
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
    let landed = store_writes_to(&store_dir(tag, "a"), &admin.device_key);
    let path = org_io::store_file::store_path(&store_dir(tag, "a"), &admin.device_key);
    assert_eq!(std::fs::read(path).unwrap(), *landed.last().unwrap(), "the file holds the kept request");
}

// ---- answering an organisation request ---------------------------------------

// Normal: a Member behind (the Admin adopted an epoch it was never sent)
// asks for the Organisation. The Admin's receive step writes the request's
// addresses, answers `0x00`, and sends the full record on a new connection
// to the addresses the request carried (the Admin kept dead ones for Bob
// before it); the Member keeps it and adopts it by verification. The Admin
// reads no chain on the way.
// verifies: LLR-898z24
#[tokio::test(flavor = "multi_thread")]
async fn an_answer_is_written_0x00_then_sent_on_a_new_connection_to_the_kept_addresses() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "answer-record";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    // Bob reaches the Admin at the Invite's addresses, which his first adoption keeps; the
    // Admin's addresses for Bob are made dead.
    let (dead, soon_closed) = journey::soon_dead(bob.device_key).await;
    journey::keep_addresses(&admin.io, admin.org, &bob.io, bob.device_key, dead).await;
    soon_closed.close().await;
    let carol = journey::invited(&admin, tag, "c", "carol").await;
    journey::admit(&admin, &carol).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, org_io::Verified::Behind { .. })]), "Bob is behind");
    let writes = store_writes_to(&store_dir(tag, "a"), &admin.device_key).len();
    let reads = (admin.chain.reads(), admin.chain.finalised_reads());

    let (answered, requested, kept) =
        tokio::join!(journey::next(&admin.io), bob.io.request_organisation(admin.org), journey::next(&bob.io));
    match answered {
        Ok(Received::OrganisationRequestAnswered { org_id, requester, delivery }) => {
            assert_eq!((org_id, requester), (admin.org, bob.device_key));
            assert!(matches!(delivery, Delivery::Delivered), "{delivery:?}");
        }
        other => panic!("not answered: {other:?}"),
    }
    let requested = requested.unwrap();
    assert!(
        matches!(requested.per_device.as_slice(), [(recipient, Delivery::Delivered)] if *recipient == admin.device_key),
        "the inbound connection carried the verdict 0x00: {requested:?}"
    );
    assert!(
        matches!(kept, Ok(Received::ProvisionalOrganisationReceived { epoch, from, .. }) if epoch == org_node::Epoch::new(3) && from == admin.device_key),
        "the answer arrived on a connection of its own: {kept:?}"
    );
    assert_eq!(store_writes_to(&store_dir(tag, "a"), &admin.device_key).len(), writes + 1, "the request's addresses written");
    assert_eq!((admin.chain.reads(), admin.chain.finalised_reads()), reads, "no chain read on the answer path");
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, org_io::Verified::Adopted { .. })]));
}

// Abnormal: the store write of an organisation
// request's addresses fails after its temporary file. The step is
// `ReceiveError::Store`, the verdict written is `0x01`, and no answer is
// sent: nothing reaches the addresses the request carried, nor the
// requester's own receiving handle.
// verifies: LLR-898z24
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_store_write_of_a_request_is_receive_error_store_0x01_and_no_answer() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "answer-store-fails";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    let (answering_addresses, mut arrivals, _listening) = journey::answering(bob.device_key, org_io::transport::BindPlan::Loopback, vec![]).await;
    let body = encode_message(&WireMessage::OrganisationRequest { org_id: admin.org, addresses: answering_addresses }).unwrap();
    org_io::test_support::fail_next_write_after_temporary();
    let (received, read_back) = journey::dialled(&admin.io, &bob.device_key, &body).await;
    assert!(matches!(received, Err(ReceiveError::Store(org_io::StoreFailure::Reloaded { .. }))), "{received:?}");
    assert_eq!(read_back, vec![0x01], "refused: the addresses not kept");
    let (answered, to_bob) = tokio::join!(
        journey::next_arrival(&mut arrivals, Duration::from_secs(3)),
        tokio::time::timeout(Duration::from_secs(3), bob.io.next_received())
    );
    assert!(answered.is_none(), "no answer reaches the addresses the request carried");
    assert!(to_bob.is_err(), "nor the requester's own endpoint");
}

// A Device held as revoked asks: it is answered through the send path with
// its revocation notice alone, which it keeps, and its next verification
// acts on it.
// verifies: LLR-898z24
#[tokio::test(flavor = "multi_thread")]
async fn a_revoked_requester_gets_its_notice_through_the_send_path() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "answer-notice";
    let admin = receiving_admin(tag).await;
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit_over_transport(&admin, &bob).await;
    // The Admin revokes Bob, who has not verified it yet: he is behind.
    journey::revoke(&admin, bob.device_key).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, org_io::Verified::Behind { .. })]));
    let reads = (admin.chain.reads(), admin.chain.finalised_reads());
    let (answered, _, kept) =
        tokio::join!(journey::next(&admin.io), bob.io.request_organisation(admin.org), journey::next(&bob.io));
    assert!(
        matches!(answered, Ok(Received::OrganisationRequestAnswered { requester, delivery: Delivery::Delivered, .. }) if requester == bob.device_key),
        "{answered:?}"
    );
    assert!(matches!(kept, Ok(Received::RevocationNoticeKept { from, .. }) if from == admin.device_key), "{kept:?}");
    assert_eq!((admin.chain.reads(), admin.chain.finalised_reads()), reads, "no chain read on the answer path");
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, org_io::Verified::Removed { .. })]));
}

// Abnormal: a Device no record here has listed (a provisional Member asking
// the issuer before its admission) is refused `0x01`; no store write, no
// answer, no report.
// verifies: LLR-236rf9
#[tokio::test(flavor = "multi_thread")]
async fn a_never_listed_requester_gets_0x01_and_no_answer() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "answer-never-listed";
    let admin = receiving_admin(tag).await;
    let carol = journey::invited(&admin, tag, "c", "carol").await;
    carol.io.start_receiving().await.unwrap();
    let files = store_bytes(tag, "a");
    let reads = admin.chain.reads();
    let (refused, requested, answer) = tokio::join!(
        journey::next(&admin.io),
        carol.io.request_organisation(admin.org),
        tokio::time::timeout(Duration::from_secs(3), carol.io.next_received())
    );
    assert!(matches!(refused, Err(ReceiveError::Refused(OrgNodeError::SenderNotListed { .. }))), "{refused:?}");
    assert!(matches!(requested.unwrap().per_device.as_slice(), [(_, Delivery::Refused)]));
    assert!(answer.is_err(), "no answer is sent");
    assert_eq!(store_bytes(tag, "a"), files, "nothing written");
    assert_eq!(admin.chain.reads(), reads, "no chain read");
}
