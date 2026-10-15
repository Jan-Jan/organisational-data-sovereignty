#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The receive step (SDD-aw3nka; S4, task T11 of
//! docs/plans/2026-10-06-org-io-transport.md): `next_received` waits for the
//! next body with no lock on the node held (LLR-9arvqj), hands org-node the
//! body and the sender the handshake authenticated, reads the chain only
//! after org-node's check accepts and then once, and writes the store change
//! before it returns (LLR-cvyyr8); the end of the receive loop is a typed
//! terminal condition, and nothing else is (LLR-szhe7j).
//!
//! Every test runs over fresh directories and a fresh in-memory keychain,
//! one test at a time under `fresh_keychain`'s lock: keyring's default
//! credential builder is process-global, and two tests lock the keychain.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use org_node::wire::WireMessage;
use org_node::{MemberId, OrgId, OrgNodeError, PersonaId, ReceiveKind};
use org_io::store_file::store_path;
use org_io::test_support::{FakeChain, Refusal};
use org_io::{KeychainError, OrgIo, ReceiveError, Received, TerminalCause};
use rand::rngs::OsRng;

mod common;
use common::handles::{
    at, bind_seed, committed, create, deliver, fresh_keychain, handle, joiner, joiner_of, receiving_addr, refusal,
    reopened_service, spawn_recv_one, store_dir, NET,
};

/// A has founded an Organisation over `chain`; B has a Persona, Bob, whose
/// endpoint accepts, and its own read counter (`chain_b`) over the same
/// slots. Both write their stores to directories of their own.
struct Pair {
    a: OrgIo,
    chain: FakeChain,
    b: OrgIo,
    chain_b: FakeChain,
    org: OrgId,
    alice: PersonaId,
    bob: PersonaId,
    addr_b: iroh::EndpointAddr,
    b_dir: PathBuf,
}

async fn pair(tag: &str, b_expects_the_admission: bool) -> Pair {
    let chain = FakeChain::new();
    let a = handle(tag, "a", &chain);
    let alice = create(&a, "alice").await;
    let org = a.found_organisation(&mut OsRng, &alice).await.unwrap();
    let chain_b = FakeChain::sharing_slots_with(&chain);
    let b = handle(tag, "b", &chain_b);
    let bob = create(&b, "bob").await;
    if b_expects_the_admission {
        b.expect_admission(&mut OsRng, &bob, org).await.unwrap();
    }
    let addr_b = receiving_addr(&b).await;
    Pair { a, chain, b, chain_b, org, alice, bob, addr_b, b_dir: store_dir(tag, "b") }
}

/// A submits `update`, built for B's Bob, and sends it to B, whose
/// `next_received` runs meanwhile; returns B's result. *S4 task T12,
/// 2026-10-13:* `submit_commit_send` names no recipient and takes org-io's
/// `PeerAddr`; B's address is Bob's Device's.
async fn deliver_to_b(pair: &Pair, update: org_io::BuiltUpdate) -> Result<Received, ReceiveError> {
    let (received, sent) = tokio::join!(
        async { tokio::time::timeout(NET, pair.b.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            pair.a.submit_commit_send(&mut OsRng, &update, at(pair.addr_b.clone())).await
        }
    );
    sent.unwrap();
    received
}

/// A admits Bob and sends the admission to B; returns B's result.
async fn admit_bob(pair: &Pair) -> Result<Received, ReceiveError> {
    let bob = joiner_of(&pair.b, &pair.bob);
    let update = pair.a.admit_member(&mut OsRng, pair.org, &bob).await.unwrap();
    deliver_to_b(pair, update).await
}

/// Bob's MemberId in A's record.
fn member_id_of_bob(pair: &Pair) -> MemberId {
    let device_bob = joiner_of(&pair.b, &pair.bob).device_key;
    let record = pair.a.node_for_test().list_orgs().into_iter().find(|record| record.org_id == pair.org).cloned().unwrap();
    record.trie_members.iter().find(|member| member.device_keys.contains(&device_bob)).unwrap().id
}

/// A revokes Bob and sends the notice to B; returns B's result.
async fn revoke_bob(pair: &Pair, before_the_send: impl FnOnce()) -> Result<Received, ReceiveError> {
    let update = pair.a.revoke_member(&mut OsRng, pair.org, member_id_of_bob(pair)).await.unwrap();
    before_the_send();
    deliver_to_b(pair, update).await
}

/// The bytes `body` arrives as at `addr`, sent from `endpoint`.
async fn send_raw(endpoint: &org_io::transport::OrgEndpoint, addr: iroh::EndpointAddr, body: &[u8]) {
    tokio::time::timeout(NET, endpoint.send_body(addr, body)).await.unwrap().unwrap();
}

/// B's next result while `send` runs.
async fn received_by_b_while<F: std::future::Future<Output = ()>>(pair: &Pair, send: F) -> Result<Received, ReceiveError> {
    let (received, ()) = tokio::join!(async { tokio::time::timeout(NET, pair.b.next_received(&mut OsRng)).await.unwrap() }, async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        send.await;
    });
    received
}

/// A's genuine admission of a third Member, captured by a sink: a body B
/// may receive from A's Device, or have relayed by another.
async fn captured_admission_of_carol(pair: &Pair) -> WireMessage {
    let carol = joiner(0x71, "carol");
    let update = pair.a.admit_member(&mut OsRng, pair.org, &carol).await.unwrap();
    // *S4 task T12, 2026-10-13:* the sink is Carol's Device (`joiner(0x71,
    // ..)`'s is the seed `[0x72; 32]`'s): a Loopback send goes only to a
    // recipient's address.
    let (sink_addr, sink) = spawn_recv_one([0x72; 32]).await;
    pair.a.submit_commit_send(&mut OsRng, &update, at(sink_addr)).await.unwrap();
    let (_, _, captured) = sink.await.unwrap();
    captured
}

// ---- LLR-9arvqj: no lock held while the receive step waits -------------------

// A command that needs the node, a query and a mutating call, completes
// while `next_received` waits with no peer sending (the reproduction of
// the app's receiver holding the lock, PR-7v8anh).
// verifies: LLR-9arvqj, PR-7v8anh
#[tokio::test(flavor = "multi_thread")]
async fn a_command_completes_while_the_receiver_waits() {
    let (_guard, _keychain) = fresh_keychain();
    let io = Arc::new(handle("step-waits", "a", &FakeChain::new()));
    create(&io, "alice").await;
    io.start_receiving().await.unwrap();
    let receiver = {
        let io = io.clone();
        tokio::spawn(async move { io.next_received(&mut OsRng).await })
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

// Abnormal: the chain read the step makes for an accepted body stalls (a
// dependency that does not answer). The step holds no lock across it: a
// query and a mutating command complete meanwhile, and the step commits once
// the read answers (S4, task T23).
// verifies: LLR-9arvqj
#[tokio::test(flavor = "multi_thread")]
async fn a_command_completes_while_the_steps_chain_read_stalls() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-read-stalls", true).await;
    let update = pair.a.admit_member(&mut OsRng, pair.org, &joiner_of(&pair.b, &pair.bob)).await.unwrap();
    let hold = pair.chain_b.hold_next_read();
    let (received, sent, counts) = tokio::join!(
        async { tokio::time::timeout(NET, pair.b.next_received(&mut OsRng)).await.unwrap() },
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            pair.a.submit_commit_send(&mut OsRng, &update, at(pair.addr_b.clone())).await
        },
        async {
            tokio::time::timeout(NET, hold.arrived()).await.expect("the step reads the chain for the accepted body");
            let commands = async {
                let before = pair.b.personas().await.len();
                create(&pair.b, "carol").await;
                (before, pair.b.personas().await.len())
            };
            let counts = tokio::time::timeout(Duration::from_secs(2), commands).await;
            hold.release();
            counts
        }
    );
    sent.unwrap();
    assert_eq!(counts.expect("the commands waited on the stalled chain read"), (1, 2));
    assert_eq!(committed(received).kind, ReceiveKind::Admitted, "the step commits once the read answers");
}

// ---- LLR-cvyyr8: check, then one read, then commit and write ----------------

// The first admission the user declared is committed by the one receive
// path (the reproduction of the app's loop calling only the self-delete
// entry point, PR-kwwap5).
// verifies: LLR-cvyyr8, PR-kwwap5
#[tokio::test(flavor = "multi_thread")]
async fn a_declared_first_admission_is_committed_by_the_one_path() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-first-admission", true).await;
    let outcome = committed(admit_bob(&pair).await);
    assert_eq!((outcome.org_id, outcome.kind), (pair.org, ReceiveKind::Admitted));
    let held = pair.b.organisations().await;
    assert_eq!(held.len(), 1, "B holds the Organisation");
    assert_eq!(held[0].org_id, pair.org);
}

// Abnormal: an undeclared first admission, and an update from a Device the
// record does not list, are refused by org-node's check, and no chain read
// follows.
// verifies: LLR-cvyyr8
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_check_reads_no_chain() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-refused-check", false).await;
    let refused = refusal(admit_bob(&pair).await);
    assert!(matches!(refused, OrgNodeError::AdmissionNotExpected { org_id } if org_id == pair.org), "{refused:?}");
    assert_eq!(pair.chain_b.reads(), 0, "no read after a refused check");

    let pair = self::pair("step-refused-check-sender", true).await;
    committed(admit_bob(&pair).await);
    let reads_before = pair.chain_b.reads();
    let captured = captured_admission_of_carol(&pair).await;
    let refused = refusal(received_by_b_while(&pair, deliver(pair.addr_b.clone(), &captured)).await);
    assert!(matches!(refused, OrgNodeError::SenderNotListed { org_id } if org_id == pair.org), "{refused:?}");
    assert_eq!(pair.chain_b.reads(), reads_before, "no read after a refused check");
}

// verifies: LLR-cvyyr8
#[tokio::test(flavor = "multi_thread")]
async fn an_accepted_body_reads_the_chain_once() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-read-once", true).await;
    committed(admit_bob(&pair).await);
    assert_eq!(pair.chain_b.reads(), 1, "one read, for the accepted body");
}

// The same body from a Device the record lists commits; from one it does
// not list, it is refused: the sender org-node judges is the connection's.
// verifies: LLR-cvyyr8
#[tokio::test(flavor = "multi_thread")]
async fn the_sender_org_node_sees_is_the_handshakes() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-sender", true).await;
    committed(admit_bob(&pair).await);
    let captured = captured_admission_of_carol(&pair).await;
    let refused = refusal(received_by_b_while(&pair, deliver(pair.addr_b.clone(), &captured)).await);
    assert!(matches!(refused, OrgNodeError::SenderNotListed { org_id } if org_id == pair.org), "{refused:?}");
    let endpoint_a = pair.a.endpoint_for_test(&pair.alice).await.unwrap();
    let outcome = committed(
        received_by_b_while(&pair, async {
            tokio::time::timeout(NET, endpoint_a.send(pair.addr_b.clone(), &captured)).await.unwrap().unwrap();
        })
        .await,
    );
    assert_eq!((outcome.kind, outcome.epoch.get()), (ReceiveKind::Updated, 3), "from A's listed Device: committed");
}

// verifies: LLR-cvyyr8
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_is_settled_with_no_chain_read_and_no_store_write() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-acknowledgement", true).await;
    committed(admit_bob(&pair).await);
    let endpoint_bob = pair.b.endpoint_for_test(&pair.bob).await.unwrap();
    let revoked = committed(revoke_bob(&pair, || {}).await);
    let acknowledgement = revoked.acknowledgements.into_iter().next().expect("B signed its acknowledgement");
    let addr_a = receiving_addr(&pair.a).await;
    let device_alice = pair.a.persona_public_keys(&pair.alice).await.unwrap().1;
    let file_a = store_path(&store_dir("step-acknowledgement", "a"), &device_alice);
    let (reads_before, bytes_before) = (pair.chain.reads(), std::fs::read(&file_a).unwrap());
    let message = WireMessage::Acknowledgement(acknowledgement.clone());
    let (received, ()) = tokio::join!(async { tokio::time::timeout(NET, pair.a.next_received(&mut OsRng)).await.unwrap() }, async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        tokio::time::timeout(NET, endpoint_bob.send(addr_a, &message)).await.unwrap().unwrap();
    });
    match received {
        Ok(Received::Acknowledged(verified)) => assert_eq!(verified.into_inner(), acknowledgement),
        other => panic!("not settled as an acknowledgement: {other:?}"),
    }
    assert_eq!(pair.chain.reads(), reads_before, "no chain read");
    assert_eq!(std::fs::read(&file_a).unwrap(), bytes_before, "no store write");
}

// verifies: LLR-cvyyr8
#[tokio::test(flavor = "multi_thread")]
async fn an_accepted_revocation_signs_with_the_keychain_secret_and_removes_the_store() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-revocation", true).await;
    committed(admit_bob(&pair).await);
    let device_bob = joiner_of(&pair.b, &pair.bob).device_key;
    let file_bob = store_path(&pair.b_dir, &device_bob);
    assert!(file_bob.is_file());
    let outcome = committed(revoke_bob(&pair, || {}).await);
    assert_eq!(outcome.kind, ReceiveKind::Revoked);
    assert_eq!(outcome.acknowledgements.len(), 1, "one acknowledgement, Bob's");
    assert_eq!(outcome.acknowledgements[0].device, device_bob, "signed for Bob's Device");
    assert!(!file_bob.exists(), "Bob's store file is removed");
    assert!(pair.b.organisations().await.is_empty(), "the Organisation is forgotten");
}

// Abnormal: the keychain is locked when the revocation needs Bob's secret
// to sign his acknowledgement (REQ-6d9xv6: a signing operation reads the
// keychain).
// verifies: LLR-cvyyr8
#[tokio::test(flavor = "multi_thread")]
async fn a_locked_keychain_refuses_the_revocation_and_changes_nothing() {
    let (_guard, keychain) = fresh_keychain();
    let pair = pair("step-revocation-locked", true).await;
    committed(admit_bob(&pair).await);
    let device_bob = joiner_of(&pair.b, &pair.bob).device_key;
    let file_bob = store_path(&pair.b_dir, &device_bob);
    let bytes_before = std::fs::read(&file_bob).unwrap();
    let refused = revoke_bob(&pair, || keychain.refuse_as(Some(Refusal::Locked))).await;
    keychain.refuse_as(None);
    assert!(
        matches!(&refused, Err(ReceiveError::Keychain(error)) if matches!(**error, KeychainError::Locked { .. })),
        "{refused:?}"
    );
    assert_eq!(std::fs::read(&file_bob).unwrap(), bytes_before, "the store file is unchanged");
    let held = pair.b.organisations().await;
    assert_eq!((held.len(), held[0].epoch.get()), (1, 2), "the record is unchanged");
}

// verifies: LLR-cvyyr8
#[tokio::test(flavor = "multi_thread")]
async fn the_store_write_lands_before_the_outcome_returns() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-write-first", true).await;
    let outcome = committed(admit_bob(&pair).await);
    let persisted = reopened_service("step-write-first", "b", &pair.b);
    let orgs = persisted.list_orgs();
    assert_eq!(orgs.len(), 1, "the file holds the admission when the outcome returns");
    assert_eq!((orgs[0].org_id, orgs[0].epoch), (outcome.org_id, outcome.epoch));
}

/// The Organisation information of A's own kept update for the Organisation,
/// as A's commit of it would build it, relayed back to A.
fn own_update_relayed(pair: &Pair) -> WireMessage {
    let node = pair.a.node_for_test();
    let update = node.provisional_updates(pair.org).into_iter().next().expect("A keeps its update");
    let record = node.list_orgs().into_iter().find(|record| record.org_id == pair.org).cloned().unwrap();
    let org_node::store::ProvisionalChange::ChangeSet { change_set, org_private_key } = update.change_for_test().clone() else {
        panic!("a Change set")
    };
    WireMessage::OrgInformation {
        envelope: org_node::Envelope { org_id: pair.org, parent_seq: update.seq, delta_bytes: change_set },
        record_snapshot: postcard::to_allocvec(&record.trie_members).unwrap(),
        org_private_key,
    }
}

// A's own update reaches A from B's Device while A's receive step reads the
// chain for it, and A's submission of that update commits in between, so
// org-node refuses the stale check (`RecordMovedSinceCheck`). The step
// checks the body again, once, against the record as it now stands, and
// returns that check's refusal; the local commit stands.
// verifies: LLR-cvyyr8
#[tokio::test(flavor = "multi_thread")]
async fn a_check_that_went_stale_is_checked_again_once() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-stale-check", true).await;
    committed(admit_bob(&pair).await);
    let update = pair.a.admit_member(&mut OsRng, pair.org, &joiner(0x71, "carol")).await.unwrap();
    let relayed = own_update_relayed(&pair);
    let endpoint_bob = pair.b.endpoint_for_test(&pair.bob).await.unwrap();
    let addr_a = receiving_addr(&pair.a).await;
    let hold = pair.chain.hold_next_read();

    let (received, ()) = tokio::join!(async { tokio::time::timeout(NET, pair.a.next_received(&mut OsRng)).await.unwrap() }, async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        tokio::time::timeout(NET, endpoint_bob.send(addr_a, &relayed)).await.unwrap().unwrap();
        tokio::time::timeout(NET, hold.arrived()).await.expect("the step reads the chain for the checked body");
        pair.a.submit_commit_send(&mut OsRng, &update, None).await.unwrap();
        hold.release();
    });

    let refused = refusal(received);
    assert_eq!(refused, OrgNodeError::StaleSeq { got: 3, last_seen: 3 }, "the second check's refusal");
    let held = pair.a.organisations().await;
    assert_eq!((held[0].epoch.get(), held[0].root_hash), (3, update.resulting_root), "A's own commit stands");
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
    let received = tokio::time::timeout(NET, io.next_received(&mut OsRng)).await.unwrap();
    assert!(matches!(received, Err(ReceiveError::Terminal(TerminalCause::EndpointClosed))), "{received:?}");
    // Boundary: a node that accepts on no endpoint has nothing to wait for.
    let idle = handle("step-closed", "b", &FakeChain::new());
    let received = tokio::time::timeout(NET, idle.next_received(&mut OsRng)).await.unwrap();
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

// Abnormal: bytes that do not decode are org-node's refusal, and the
// receive step goes on to the next body.
// verifies: LLR-szhe7j
#[tokio::test(flavor = "multi_thread")]
async fn a_malformed_body_is_not_terminal() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-malformed", true).await;
    let stranger = bind_seed([0x5c; 32]).await;
    let refused = received_by_b_while(&pair, send_raw(&stranger, pair.addr_b.clone(), b"not a wire message")).await;
    assert!(matches!(refused, Err(ReceiveError::Refused(OrgNodeError::MalformedMessage))), "{refused:?}");
    let outcome = committed(admit_bob(&pair).await);
    assert_eq!(outcome.kind, ReceiveKind::Admitted, "the next body is received");
}

// Abnormal: a connection that fails before it carries a body (the peer
// completes the handshake, then closes without opening a stream) is
// `ReceiveError::Connection`, not terminal, and the receive step goes on to
// the next body. *Added 2026-10-15 (S4 review round 2, finding 7).*
// verifies: LLR-szhe7j
#[tokio::test(flavor = "multi_thread")]
async fn a_connection_failing_before_a_body_is_not_terminal() {
    let (_guard, _keychain) = fresh_keychain();
    let pair = pair("step-connection-failed", true).await;
    let stranger = bind_seed([0x5d; 32]).await;
    let addr_b = pair.addr_b.clone();
    let failed = received_by_b_while(&pair, async {
        let connection = tokio::time::timeout(NET, stranger.inner().connect(addr_b, org_io::transport::ALPN)).await.unwrap().unwrap();
        connection.close(1u32.into(), b"no body");
    })
    .await;
    assert!(matches!(failed, Err(ReceiveError::Connection(_))), "{failed:?}");
    let outcome = committed(admit_bob(&pair).await);
    assert_eq!(outcome.kind, ReceiveKind::Admitted, "the next body is received");
}
