#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Stage S3 (`docs/plans/2026-10-06-org-io-commit-workflow.md`).

mod support;

use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::reconcile::{reconcile, Reconciled};
use org_node::store::{ProvisionalChange, ProvisionalUpdate, ProvisionalUpdateParts};
use org_node::{DeviceSecret, DevicePublicKey, Epoch, RootHash};
use rand::rngs::OsRng;
use support::*;

/// The postcard encoding of `store`: `StoreData` has no `PartialEq`.
fn encoded<T: serde::Serialize>(store: &T) -> Vec<u8> {
    postcard::to_allocvec(store).unwrap()
}

/// A device-secret source that supplies none (S4, task T3: the source takes
/// the Device keys it is asked for).
fn none(_: &[DevicePublicKey]) -> Vec<DeviceSecret> {
    Vec::new()
}

/// verifies: LLR-gr8x3r
///
/// In step and behind, and the three refusals, each with no successor.
#[test]
fn reconcile_reports_one_outcome_from_the_record_and_the_state() {
    let s = admit_b_directly(setup("reconcile"));
    let store = s.svc_a.store_of(&s.pid_a).clone();
    let record = rec_of(&s.svc_a, s.org_id);
    let at = |epoch: u64, root: RootHash| OrgState { root_hash: root, org_pub_key: record.org_pub_key, epoch: Epoch::new(epoch) };
    let epoch = record.epoch.get();
    let other_root = RootHash::new([5; 32]);
    assert!(matches!(reconcile(&store, s.org_id, at(epoch, record.root_hash), none), Ok(Reconciled::InStep)));
    assert!(matches!(
        reconcile(&store, s.org_id, at(epoch + 1, other_root), none),
        Ok(Reconciled::Behind { chain_epoch }) if chain_epoch == Epoch::new(epoch + 1)
    ));
    assert!(matches!(reconcile(&store, s.org_id, at(epoch - 1, record.root_hash), none), Err(OrgNodeError::StaleChainState { .. })));
    assert!(matches!(reconcile(&store, s.org_id, at(epoch, other_root), none), Err(OrgNodeError::ChainStateConflict { .. })));
    assert!(matches!(
        reconcile(&store, OrgId::new([9; 20]), at(epoch, record.root_hash), none),
        Err(OrgNodeError::OrgNotHeld { .. })
    ));
}

/// verifies: LLR-gr8x3r
///
/// A stored update whose root the chain holds under another Organisation
/// public key is not the chain's update: the record is behind.
#[test]
fn reconcile_is_behind_when_no_stored_update_matches_both_root_and_key() {
    let mut s = admit_b_directly(setup("reconcile-key"));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    let record = rec_of(&s.svc_a, s.org_id);
    let chain = OrgState { root_hash: update.resulting_root, org_pub_key: record.org_pub_key, epoch: Epoch::new(record.epoch.get() + 1) };
    assert!(matches!(
        reconcile(s.svc_a.store_of(&s.pid_a), s.org_id, chain, none),
        Ok(Reconciled::Behind { chain_epoch }) if chain_epoch == chain.epoch
    ));
}

/// verifies: LLR-fm38ww, LLR-gr8x3r
///
/// PR-vt244s, org-node half: the chain was written, the node stopped before
/// committing, and on reopening, reconcile commits the kept update the chain
/// holds; the record's epoch equals the chain's.
#[test]
fn reconcile_commits_the_update_the_chain_holds_after_a_crash() {
    let mut s = admit_b_directly(setup("reconcile-crash"));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    drop(s.svc_a);
    let reopened = reopened_service("reconcile-crash", "a");
    let chain_state = s.chain.get(&s.org_id).unwrap();
    let Reconciled::Committed { store, outcome } =
        reconcile(reopened.store_data(&s.pid_a).unwrap(), s.org_id, chain_state, none).unwrap()
    else {
        panic!("committed")
    };
    let record = store.orgs.iter().find(|r| r.org_id == s.org_id).unwrap();
    assert_eq!((record.epoch, record.root_hash, record.org_pub_key), (chain_state.epoch, chain_state.root_hash, chain_state.org_pub_key));
    let ProvisionalChange::ChangeSet { change_set, .. } = update.change_for_test() else { panic!("a Change set") };
    assert_eq!(record.kept_change_set.as_ref(), Some(change_set));
    assert!(store.provisional_updates.iter().all(|u| u.resulting_root != update.resulting_root));
    // *Amended 2026-10-13 (S4, task T5):* the notices are among the bodies.
    assert!(notices_in(&outcome).is_empty());
    assert!(outcome.acknowledgements.is_empty());
}

/// verifies: LLR-a8z7r5
///
/// Review finding-7 (2026-10-07), PR-vt244s's crash path: the chain carries
/// A's removal of B, A stopped before committing it, and on reopening the
/// reconciled commit's outcome holds exactly B's revocation notice, so B is
/// still told after recovery.
#[test]
fn reconcile_of_a_crash_after_removing_another_device_returns_its_notice() {
    let mut s = admit_b_directly(setup("reconcile-crash-revoke"));
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    drop(s.svc_a);
    let reopened = reopened_service("reconcile-crash-revoke", "a");
    let chain_state = s.chain.get(&s.org_id).unwrap();
    let Reconciled::Committed { store, outcome } =
        reconcile(reopened.store_data(&s.pid_a).unwrap(), s.org_id, chain_state, none).unwrap()
    else {
        panic!("committed, A is still listed")
    };
    let record = store.orgs.iter().find(|r| r.org_id == s.org_id).unwrap();
    assert_eq!((record.epoch, record.root_hash), (chain_state.epoch, chain_state.root_hash));
    let b_device = s.b_device_kp.device_key().unwrap();
    assert_eq!(
        // *Amended 2026-10-13 (S4, task T5):* the notices are among the bodies.
        notices_in(&outcome).iter().map(|notice| (notice.org_id, notice.member_id, notice.device)).collect::<Vec<_>>(),
        vec![(s.org_id, b_id, b_device)],
        "the removed Device's notice"
    );
    assert!(outcome.acknowledgements.is_empty());
}

/// verifies: LLR-fm38ww, LLR-gr8x3r
///
/// A commit that leaves this node listed never asks for the device seeds:
/// the seed source panics if called, and the commit still succeeds. The
/// seeds are obtained only on the removal path.
#[test]
fn a_commit_that_does_not_remove_this_node_never_obtains_the_device_seeds() {
    let mut s = admit_b_directly(setup("reconcile-no-seeds"));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let chain_state = s.chain.get(&s.org_id).unwrap();
    let no_seeds = |_: &[DevicePublicKey]| -> Vec<DeviceSecret> { panic!("a non-removing commit obtained the device seeds") };
    assert!(matches!(reconcile(s.svc_a.store_of(&s.pid_a), s.org_id, chain_state, no_seeds), Ok(Reconciled::Committed { .. })));
}

/// verifies: LLR-fm38ww
///
/// A refusal of the commit step — the kept update's Change set does not
/// verify against the chain — is returned as the error, with no successor.
#[test]
fn reconcile_returns_a_refusal_of_the_commit_step_as_the_error() {
    let mut s = admit_b_directly(setup("reconcile-refused"));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let mut store = s.svc_a.store_of(&s.pid_a).clone();
    for kept in &mut store.provisional_updates {
        if let ProvisionalChange::ChangeSet { change_set, .. } = kept.change_mut_for_test() {
            *change_set = vec![0xff; 4];
        }
    }
    assert!(matches!(
        reconcile(&store, s.org_id, s.chain.get(&s.org_id).unwrap(), none),
        Err(OrgNodeError::MalformedDelta)
    ));
}

/// verifies: LLR-fm38ww
///
/// A reconciled commit that removes this node returns `Removed`, with one
/// acknowledgement and `forget_organisation`'s successor.
#[test]
fn reconcile_of_a_removal_signs_and_forgets() {
    let mut s = admit_b_directly(setup("reconcile-removal"));
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    // B holds A's revocation as a provisional update of its own: copy it into B's store data.
    let mut b_store = s.svc_b.store_of(&s.pid_b).clone();
    b_store.provisional_updates.push(ProvisionalUpdate::from(ProvisionalUpdateParts { persona_id: s.pid_b.clone(), ..update.into() }));
    let keychain = s.svc_b.keychain.clone();
    let Reconciled::Removed { store, acknowledgements } =
        reconcile(&b_store, s.org_id, s.chain.get(&s.org_id).unwrap(), |keys| keychain.secrets(keys)).unwrap()
    else {
        panic!("removed")
    };
    assert_eq!(encoded(&store), encoded(&b_store.forget_organisation(s.org_id)));
    assert_eq!(acknowledgements.len(), 1);
}

/// verifies: LLR-b27jr6
///
/// Abnormal: a reconciled commit that removes this node — the commit step
/// `commit_update` runs too — with no seed for B's Device (none, or only
/// another Device's) cannot sign. The refusal is the error, with no
/// successor: B's store still holds the record and its Persona, and B's
/// store file is unchanged.
#[test]
fn a_removal_that_cannot_sign_forgets_nothing() {
    let mut s = admit_b_directly(setup("reconcile-removal-unsigned"));
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let mut b_store = s.svc_b.store_of(&s.pid_b).clone();
    b_store.provisional_updates.push(ProvisionalUpdate::from(ProvisionalUpdateParts { persona_id: s.pid_b.clone(), ..update.into() }));
    let before = encoded(&b_store);
    let on_disk_before = store_bytes("reconcile-removal-unsigned", "b");
    for seeds in [vec![], vec![[99u8; 32]]] {
        let source = |_: &[DevicePublicKey]| seeds.iter().map(|seed| org_node::test_fixtures::device_secret(*seed)).collect();
        let refused = reconcile(&b_store, s.org_id, s.chain.get(&s.org_id).unwrap(), source);
        assert!(matches!(refused, Err(OrgNodeError::DeviceSecretNotSupplied { org_id }) if org_id == s.org_id));
    }
    assert_eq!(encoded(&b_store), before);
    assert!(b_store.orgs.iter().any(|record| record.org_id == s.org_id));
    assert!(b_store.persona.persona_id == s.pid_b && b_store.persona.org_id == Some(s.org_id));
    assert_eq!(store_bytes("reconcile-removal-unsigned", "b"), on_disk_before);
}

/// verifies: LLR-fm38ww
///
/// The interim service caller, given the state org-io read, reconciles, and
/// adopts and saves the successor: the reopened store holds the committed
/// record. (2026-10-08, ruling B: the state is an argument.) *Adapted
/// 2026-10-08 (S4, task T3):* "saves" is the store change the service
/// returns, which the test writes as org-io does.
#[test]
fn the_service_reconcile_adopts_and_saves_the_committed_record() {
    let mut s = admit_b_directly(setup("reconcile-service"));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let chain_state = s.chain.get(&s.org_id).unwrap();
    let (reconciled, change) = s.svc_a.svc.reconcile(&mut OsRng, s.org_id, s.chain.get(&s.org_id), none).unwrap();
    assert!(matches!(reconciled, Reconciled::Committed { .. }));
    s.svc_a.disk.apply(change.expect("a commit changes the store"));
    assert_eq!(rec_of(&s.svc_a, s.org_id).epoch, chain_state.epoch);
    let on_disk = disk_rec_of(&reopen_store("reconcile-service", "a"), s.org_id);
    assert_eq!((on_disk.epoch, on_disk.root_hash), (chain_state.epoch, chain_state.root_hash));
    assert!(matches!(
        s.svc_a.svc.reconcile(&mut OsRng, s.org_id, s.chain.get(&s.org_id), none),
        Ok((Reconciled::InStep, None))
    ));
}

/// verifies: LLR-gr8x3r
///
/// The service caller refuses an Organisation it does not hold with
/// `OrgNotHeld` whatever state it is given — a state, or none — before it
/// looks at the state. *Renamed 2026-10-08 (ruling B):* was
/// `the_service_reconcile_refuses_an_unheld_organisation_without_a_chain_read`;
/// org-node reads no chain, and the order of org-io's read is org-io's.
#[test]
fn the_service_reconcile_refuses_an_unheld_organisation_whatever_state_it_is_given() {
    let mut s = setup("reconcile-unheld");
    let unheld = OrgId::new([9; 20]);
    let held_state = s.chain.get(&s.org_id);
    assert!(held_state.is_some(), "fixture: a real state to offer");
    for given in [held_state, None] {
        assert!(matches!(
            s.svc_a.svc.reconcile(&mut OsRng, unheld, given, none),
            Err(OrgNodeError::OrgNotHeld { org_id }) if org_id == unheld
        ));
    }
}
