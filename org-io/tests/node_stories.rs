#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The five stories end to end, over org-io's handle, two Loopback
//! endpoints and the substitute chain (`test_support::FakeChain`): S4 task
//! T13 duplicates org-node's `tests/service_stories.rs::five_stories_full_e2e`,
//! whose org-node copy is value-driven since S4 task T4 (plan §5).
//!
//!   1. A creates a Persona and an Organisation → epoch 1 in the slots.
//!   2. B creates a Persona and replies to A's Invite, declaring the
//!      admission it expects.
//!   3. A admits B's reply (trie add → epoch 2; the body sent to B).
//!   4. B receives and commits it → B's Persona Active, its record stored.
//!   5. A revokes B (trie remove → epoch 3); B acknowledges and forgets the
//!      Organisation with its Persona.
//!
//! *Annotation, 2026-10-14 (S4, task T13):* the original's `verifies:` line
//! names org-node's items (the envelope commit at the chain's epoch and
//! root, the first admission, the self-delete on the device's own removal,
//! and four of their low-level items); a test of this unit cites only its
//! own items and those exported to it, so this copy names org-io's
//! exchange order and receive step and org-node's exported one-receive-path
//! requirement instead, and names the originals here in prose.

use std::time::Duration;

use org_node::invitation::Invite;
use org_node::store::PersonaStatus;
use org_node::{Epoch, OrgId, ReceiveKind, ReceiveOutcome};
use org_io::test_support::FakeChain;
use org_io::{OrgIo, Received};
use rand::rngs::OsRng;

mod common;
use common::handles::{at, handle, persona_named, receiving_addr, NET};

/// B's next result while `send` runs on A.
async fn received_while<T>(b: &OrgIo, send: impl std::future::Future<Output = T>) -> (ReceiveOutcome, T) {
    let (received, sent) = tokio::join!(async { tokio::time::timeout(NET, b.next_received(&mut OsRng)).await.unwrap() }, async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        send.await
    });
    match received {
        Ok(Received::Committed(outcome)) => (outcome, sent),
        other => panic!("B did not commit: {other:?}"),
    }
}

fn epoch_on_chain(chain: &FakeChain, org: OrgId) -> Epoch {
    chain.slots().get(&org).unwrap().epoch
}

// *Corrected 2026-10-15 (S4 TEST review round 1, finding-3; Seams 4):* the
// journey walks the invitation exchange, so it verifies the three exchange
// requirements that came to org-io with it (SDD-rr7dtw): the Invite
// (REQ-prjja8), the reply and its declared expectation (REQ-tcutr6), and
// acting on an outstanding reply for the Organisation it was issued for
// (REQ-65xqp8). An S4-new test, so its annotation is corrected, not amended.
// The two `outstanding` assertions joined it then; mutation check, that day:
// `admit_reply` with its `settle` removed failed the second.
// verifies: REQ-a2a6rm, LLR-cvyyr8, REQ-2ugh2f, REQ-prjja8, REQ-tcutr6, REQ-65xqp8
#[tokio::test(flavor = "multi_thread")]
async fn five_stories_full_e2e() {
    let chain = FakeChain::new();
    let a = handle("stories", "a", &chain);
    let b = handle("stories", "b", &FakeChain::sharing_slots_with(&chain));

    // ---- Story 1: A creates a Persona and an Organisation ----
    let admin = persona_named(&a, "admin", "Admin", "User").await;
    let org = a.found_organisation(&mut OsRng, &admin).await.unwrap();
    assert_eq!(epoch_on_chain(&chain, org), Epoch::new(1));
    assert_eq!(a.personas().await[0].status, PersonaStatus::Active);

    // ---- Story 2: B creates a Persona and replies to A's Invite ----
    let bob = persona_named(&b, "bob", "Bob", "Builder").await;
    let invite = a.issue_invite(&mut OsRng, org, "Acme", "Bob Builder").await.unwrap();
    assert_eq!(Invite::parse(&invite).unwrap().org_id, org);
    let invite_id = Invite::parse(&invite).unwrap().invite_id;
    let outstanding = |io: &OrgIo| io.node_for_test().store_data(&admin).unwrap().outstanding_invites.clone();
    assert_eq!(outstanding(&a), vec![(org, invite_id)], "A holds the Invite's identifier as outstanding");
    let reply = b.produce_reply(&mut OsRng, &invite, &bob, true).await.unwrap();
    assert_eq!(b.node_for_test().expected_admissions()[0].org_id, org, "B declared the admission it expects");
    let imported = a.import_reply(&reply).await.unwrap();
    assert_eq!(imported.handle.as_str(), "bob");

    // ---- Story 3+4: B waits, A admits ----
    let addr_b = receiving_addr(&b).await;
    let (admitted, report) = received_while(&b, a.admit_reply(&mut OsRng, org, &reply, at(addr_b.clone()))).await;
    let report = report.unwrap();
    assert_eq!(epoch_on_chain(&chain, org), Epoch::new(2), "admit must bump to epoch 2");
    assert!(outstanding(&a).is_empty(), "the identifier is no longer outstanding once the reply was acted on");
    assert_eq!((admitted.org_id, admitted.epoch), (org, Epoch::new(2)));
    assert_eq!(admitted.root, chain.slots().get(&org).unwrap().root_hash, "B's committed root is the chain's");
    let b_persona = b.personas().await.into_iter().find(|persona| persona.persona_id == bob).unwrap();
    assert_eq!(b_persona.status, PersonaStatus::Active, "B's Persona is Active after the commit");
    let b_orgs = b.organisations().await;
    assert_eq!((b_orgs.len(), b_orgs[0].epoch), (1, Epoch::new(2)), "B holds exactly the one record");
    assert_eq!(
        b.node_for_test().list_orgs()[0].org_private_key_for_test(),
        a.node_for_test().list_orgs()[0].org_private_key_for_test(),
        "B holds the Organisation private key A's record holds"
    );
    assert!(b.node_for_test().expected_admissions().is_empty(), "the expectation is cleared on commit");

    // ---- Story 5: A revokes B; B acknowledges and forgets ----
    let update = a.revoke_member(&mut OsRng, org, report.member_id).await.unwrap();
    let (revoked, submitted) = received_while(&b, a.submit_commit_send(&mut OsRng, &update, at(addr_b))).await;
    submitted.unwrap();
    assert_eq!(epoch_on_chain(&chain, org), Epoch::new(3), "revoke must bump to epoch 3");
    let ReceiveOutcome { org_id, kind: ReceiveKind::Revoked, acknowledgements, .. } = revoked else {
        panic!("expected a revocation accepted, got {revoked:?}");
    };
    assert_eq!(org_id, org);
    assert_eq!(acknowledgements.len(), 1, "one acknowledgement, B's");
    assert_eq!(acknowledgements[0].member_id, report.member_id);
    assert!(b.organisations().await.is_empty(), "B holds no record");
    assert!(b.personas().await.iter().all(|persona| persona.persona_id != bob), "B's Persona is gone");
    let a_orgs = a.organisations().await;
    assert_eq!((a_orgs.len(), a_orgs[0].epoch, a_orgs[0].members.len()), (1, Epoch::new(3), 1), "only A remains");
}
