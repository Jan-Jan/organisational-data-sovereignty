#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! A check that went stale before its commit (S4, task T22 of
//! docs/plans/2026-10-06-org-io-transport.md; PR-erskh4). org-io's receive
//! step releases the service between `check_received` and `commit_checked`
//! for the one chain read, so a local commit to the same Organisation can
//! land in between. Each test checks a received body at B, makes B's local
//! change, then commits the earlier check against the chain as it then
//! stands, and compares the result with a fresh check of the same body.
//!
//! *Strengthened 2026-10-14 (review round 1 of S4, finding-11):* every
//! refusal as moved is also shown to call no secret source
//! (`refused_as_moved`), and a record that appeared or went between the
//! check and the commit has its own case.
//!
//! An acknowledgement has no case here: `check_received` decides it against
//! the store alone (`Checked::Acknowledged`) and nothing commits it later.

mod support;
use support::*;

use org_members::DevicePublicKey;
use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::keys::DeviceSecret;
use org_node::service::{Checked, CheckedMessage, ReceiveOutcome, StoreChange};
use org_node::store::{OrgRecord, ProvisionalChange, ProvisionalUpdate};
use org_node::wire::{encode_message, WireMessage};
use org_node::{Envelope, Epoch, OrgId};
use rand::rngs::OsRng;

/// The Organisation information `update` travels in, as a commit of it
/// builds it: its Envelope, the snapshot of `before` and its key.
fn information_body(org_id: OrgId, update: &ProvisionalUpdate, before: &OrgRecord) -> Vec<u8> {
    let ProvisionalChange::ChangeSet { change_set, org_private_key } = update.change_for_test().clone() else {
        panic!("a Change set")
    };
    encode_message(&WireMessage::OrgInformation {
        envelope: Envelope { org_id, parent_seq: update.seq, delta_bytes: change_set },
        record_snapshot: postcard::to_allocvec(&before.trie_members).unwrap(),
        org_private_key,
    })
    .unwrap()
}

/// `check_received` must return a message to read.
fn checked(node: &Node, sender: DevicePublicKey, body: &[u8]) -> CheckedMessage {
    match node.svc.check_received(sender, body) {
        Ok(Checked::ReadChain(checked)) => checked,
        other => panic!("not a message to read: {other:?}"),
    }
}

/// `commit_checked` against `state`, the secrets from `node`'s keychain.
fn commit(
    node: &mut Node,
    checked: CheckedMessage,
    state: Option<OrgState>,
) -> Result<(ReceiveOutcome, StoreChange), OrgNodeError> {
    let keychain = node.keychain.clone();
    node.svc.commit_checked(&mut OsRng, checked, state, |keys: &[DevicePublicKey]| -> Vec<DeviceSecret> {
        keychain.secrets(keys)
    })
}

/// The plaintext of every store `node` holds.
fn plaintext(node: &Node) -> Vec<Vec<u8>> {
    node.svc
        .list_personas()
        .iter()
        .map(|persona| postcard::to_allocvec(node.svc.store_data(&persona.persona_id).unwrap()).unwrap())
        .collect()
}

/// B's own admission of a further Persona of its node, kept: the update.
fn local_admission(s: &mut Setup) -> ProvisionalUpdate {
    let joiner = joiner_for_c(&mut s.svc_b);
    s.svc_b.admit_member(&mut OsRng, s.org_id, &joiner).unwrap()
}

/// `commit_checked` of `checked` against `state` is the stale-check refusal
/// for `org_id`, the stores are as the local commit left them, and the
/// secret source was never called (LLR-6qjmhf as amended by T22; review
/// round 1 of S4, finding-11): it records every call and yields nothing.
fn refused_as_moved(node: &mut Node, checked: CheckedMessage, state: Option<OrgState>, org_id: OrgId, after_local: &[Vec<u8>]) {
    let calls = std::cell::Cell::new(0);
    let refused = node.svc.commit_checked(&mut OsRng, checked, state, |_: &[DevicePublicKey]| -> Vec<DeviceSecret> {
        calls.set(calls.get() + 1);
        Vec::new()
    });
    match refused {
        Err(error) => assert_eq!(error, OrgNodeError::RecordMovedSinceCheck { org_id }),
        Ok((outcome, change)) => panic!("committed: {outcome:?}, {change:?}"),
    }
    assert_eq!(calls.get(), 0, "the refusal called no secret source");
    assert_eq!(plaintext(node), after_local, "the local commit stands, nothing else written");
}

// A's update (epoch 3) is checked by B; B's own update then takes epoch 3 on
// the chain and B commits it, so A's never reaches the chain. Before T22 the
// stale commit was refused by the chain root (`RootMismatch`); a fresh check
// refuses the body as a replay.
// verifies: LLR-6qjmhf, PR-erskh4
#[test]
fn an_update_checked_before_a_local_commit_is_refused_as_moved() {
    let mut s = admit_b_directly(setup("sc-update"));
    let before = rec_of(&s.svc_a.svc, s.org_id);
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let a_update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    let body = information_body(s.org_id, &a_update, &before);
    let sender = s.svc_a.sender_for(s.org_id);
    let check = checked(&s.svc_b, sender, &body);

    let b_update = local_admission(&mut s);
    s.chain.apply_update(s.org_id, b_update.resulting_root, b_update.org_pub_key, before.epoch).unwrap();
    s.svc_b.commit_update(&mut OsRng, s.org_id).unwrap();
    let after_local = plaintext(&s.svc_b);

    let state = s.chain.get(&s.org_id);
    refused_as_moved(&mut s.svc_b, check, state, s.org_id, &after_local);
    let fresh = s.svc_b.svc.check_received(sender, &body).map(|_| ());
    assert_eq!(fresh, Err(OrgNodeError::StaleSeq { got: 3, last_seen: 3 }));
}

// B's own update, relayed back to B by A, checked before B commits that
// update. Before T22 the stale commit was accepted (`Updated`, the record
// rewritten with the values it already held), though a fresh check refuses
// the body as a replay.
// verifies: LLR-6qjmhf, PR-erskh4
#[test]
fn the_nodes_own_update_relayed_back_before_its_commit_is_refused_as_moved() {
    let mut s = admit_b_directly(setup("sc-echo"));
    let before = rec_of(&s.svc_b.svc, s.org_id);
    let update = local_admission(&mut s);
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, before.epoch).unwrap();
    let body = information_body(s.org_id, &update, &before);
    let sender = s.svc_a.sender_for(s.org_id);
    let check = checked(&s.svc_b, sender, &body);

    s.svc_b.commit_update(&mut OsRng, s.org_id).unwrap();
    let after_local = plaintext(&s.svc_b);

    let state = s.chain.get(&s.org_id);
    refused_as_moved(&mut s.svc_b, check, state, s.org_id, &after_local);
    let fresh = s.svc_b.svc.check_received(sender, &body).map(|_| ());
    assert_eq!(fresh, Err(OrgNodeError::StaleSeq { got: 3, last_seen: 3 }));
}

// A's removal of B (epoch 3) is checked by B; B then commits its own update
// against a state, given as a value, that holds it at epoch 3. Before T22
// the stale commit was decided on the moved record (`RevocationProofRefused`),
// as a fresh check and commit decide it.
// verifies: LLR-6qjmhf, PR-erskh4
#[test]
fn a_revocation_checked_before_a_local_commit_is_refused_as_moved() {
    let mut s = admit_b_directly(setup("sc-revocation"));
    let b_before = rec_of(&s.svc_b.svc, s.org_id);
    let b_update = local_admission(&mut s);
    let b_id = id_by_handle(&rec_of(&s.svc_a.svc, s.org_id), "bob");
    let outcome = commit_removal(&mut s.svc_a, &s.chain, s.org_id, b_id).unwrap();
    let body = body_for(&outcome, s.joiner_b.device_key).unwrap();
    let sender = s.svc_a.sender_for(s.org_id);
    let check = checked(&s.svc_b, sender, &body);

    let b_state = OrgState {
        root_hash: b_update.resulting_root,
        org_pub_key: b_update.org_pub_key,
        epoch: Epoch::new(b_before.epoch.get() + 1),
    };
    let keychain = s.svc_b.keychain.clone();
    let (_, change) = s.svc_b.svc.commit_update(&mut OsRng, s.org_id, Some(b_state), |keys| keychain.secrets(keys)).unwrap();
    s.svc_b.disk.apply(change);
    let after_local = plaintext(&s.svc_b);

    refused_as_moved(&mut s.svc_b, check, Some(b_state), s.org_id, &after_local);
    let fresh = checked(&s.svc_b, sender, &body);
    let decided = commit(&mut s.svc_b, fresh, Some(b_state)).map(|_| ());
    assert!(matches!(decided, Err(OrgNodeError::RevocationProofRefused { org_id, .. }) if org_id == s.org_id), "{decided:?}");
}

// A's admission of B is checked by B; B's Persona then founds an
// Organisation of its own. No store held A's Organisation at the check and
// none holds it now, so the record did not move: the commit refuses the
// admission because no unbound Persona expects it, as a fresh check and
// commit do.
// verifies: LLR-6qjmhf
#[test]
fn a_first_admission_checked_before_its_persona_is_bound_is_not_ours() {
    let mut s = setup("sc-first");
    let joiner = s.joiner_b.clone();
    let (_, outcome) = commit_admission(&mut s.svc_a, &s.chain, s.org_id, &joiner).unwrap();
    let body = body_for(&outcome, joiner.device_key).unwrap();
    let sender = s.svc_a.sender_for(s.org_id);
    let check = checked(&s.svc_b, sender, &body);

    let pid_b = s.pid_b.clone();
    found(&mut s.svc_b, &s.chain, &pid_b);
    let after_local = plaintext(&s.svc_b);

    let refused = commit(&mut s.svc_b, check, s.chain.get(&s.org_id)).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::AdmissionNotOurs { org_id: s.org_id }));
    assert_eq!(plaintext(&s.svc_b), after_local);
    let fresh = checked(&s.svc_b, sender, &body);
    let decided = commit(&mut s.svc_b, fresh, s.chain.get(&s.org_id)).map(|_| ());
    assert_eq!(decided, Err(OrgNodeError::AdmissionNotOurs { org_id: s.org_id }));
}

// A record that appeared between the check and the commit (review round 1
// of S4, finding-11): B checks A's admission of B twice while no store holds
// the Organisation; the second check commits, so the record appears; the
// first is then refused as moved — not judged against the record it never
// read — and nothing is written and no secret asked for.
// verifies: LLR-6qjmhf, PR-erskh4
#[test]
fn a_record_that_appeared_since_the_check_is_refused_as_moved() {
    let mut s = setup("sc-appeared");
    let joiner = s.joiner_b.clone();
    let (_, outcome) = commit_admission(&mut s.svc_a, &s.chain, s.org_id, &joiner).unwrap();
    let body = body_for(&outcome, joiner.device_key).unwrap();
    let sender = s.svc_a.sender_for(s.org_id);
    let earlier = checked(&s.svc_b, sender, &body);
    let later = checked(&s.svc_b, sender, &body);

    let state = s.chain.get(&s.org_id);
    let (admitted, change) = commit(&mut s.svc_b, later, state).unwrap();
    s.svc_b.disk.apply(change);
    assert_eq!(admitted.org_id, s.org_id, "the record appeared");
    let after_local = plaintext(&s.svc_b);

    refused_as_moved(&mut s.svc_b, earlier, state, s.org_id, &after_local);
}

// A record that went between the check and the commit (review round 1 of
// S4, finding-11): B checks A's removal of B twice; the second commits and
// B forgets the Organisation; the first is then refused as moved, with no
// secret asked for and nothing written.
// verifies: LLR-6qjmhf, PR-erskh4
#[test]
fn a_record_that_went_since_the_check_is_refused_as_moved() {
    let mut s = admit_b_directly(setup("sc-went"));
    let b_id = id_by_handle(&rec_of(&s.svc_a.svc, s.org_id), "bob");
    let outcome = commit_removal(&mut s.svc_a, &s.chain, s.org_id, b_id).unwrap();
    let body = body_for(&outcome, s.joiner_b.device_key).unwrap();
    let sender = s.svc_a.sender_for(s.org_id);
    let earlier = checked(&s.svc_b, sender, &body);
    let later = checked(&s.svc_b, sender, &body);

    let state = s.chain.get(&s.org_id);
    let (removed, change) = commit(&mut s.svc_b, later, state).unwrap();
    s.svc_b.disk.apply(change);
    assert_eq!(removed.org_id, s.org_id);
    assert!(s.svc_b.svc.list_orgs().is_empty(), "the record went");
    let after_local = plaintext(&s.svc_b);

    refused_as_moved(&mut s.svc_b, earlier, state, s.org_id, &after_local);
}
