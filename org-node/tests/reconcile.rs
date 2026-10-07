#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Stage S3 (`docs/plans/2026-10-06-org-io-commit-workflow.md`).

mod support;

use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::reconcile::{reconcile, Reconciled};
use org_node::store::{ProvisionalChange, ProvisionalUpdate, StoreData};
use org_node::{Epoch, RootHash};
use rand::rngs::OsRng;
use support::*;

/// The postcard encoding of `store`: `StoreData` has no `PartialEq`.
fn encoded(store: &StoreData) -> Vec<u8> {
    postcard::to_allocvec(store).unwrap()
}

/// verifies: LLR-gr8x3r
///
/// In step and behind, and the three refusals, each with no successor.
#[tokio::test]
async fn reconcile_reports_one_outcome_from_the_record_and_the_state() {
    let s = admit_b_directly(setup("reconcile").await).await;
    let store = s.svc_a.store_data().clone();
    let record = rec_of(&s.svc_a, s.org_id);
    let at = |epoch: u64, root: RootHash| OrgState { root_hash: root, org_pub_key: record.org_pub_key, epoch: Epoch::new(epoch) };
    let epoch = record.epoch.get();
    let other_root = RootHash::new([5; 32]);
    assert!(matches!(reconcile(&store, s.org_id, at(epoch, record.root_hash), Vec::new), Ok(Reconciled::InStep)));
    assert!(matches!(
        reconcile(&store, s.org_id, at(epoch + 1, other_root), Vec::new),
        Ok(Reconciled::Behind { chain_epoch }) if chain_epoch == Epoch::new(epoch + 1)
    ));
    assert!(matches!(reconcile(&store, s.org_id, at(epoch - 1, record.root_hash), Vec::new), Err(OrgNodeError::StaleChainState { .. })));
    assert!(matches!(reconcile(&store, s.org_id, at(epoch, other_root), Vec::new), Err(OrgNodeError::ChainStateConflict { .. })));
    assert!(matches!(
        reconcile(&store, OrgId::new([9; 20]), at(epoch, record.root_hash), Vec::new),
        Err(OrgNodeError::OrgNotHeld { .. })
    ));
}

/// verifies: LLR-gr8x3r
///
/// A stored update whose root the chain holds under another Organisation
/// public key is not the chain's update: the record is behind.
#[tokio::test]
async fn reconcile_is_behind_when_no_stored_update_matches_both_root_and_key() {
    let mut s = admit_b_directly(setup("reconcile-key").await).await;
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    let record = rec_of(&s.svc_a, s.org_id);
    let chain = OrgState { root_hash: update.resulting_root, org_pub_key: record.org_pub_key, epoch: Epoch::new(record.epoch.get() + 1) };
    assert!(matches!(
        reconcile(s.svc_a.store_data(), s.org_id, chain, Vec::new),
        Ok(Reconciled::Behind { chain_epoch }) if chain_epoch == chain.epoch
    ));
}

/// verifies: LLR-fm38ww, LLR-gr8x3r
///
/// PR-vt244s, org-node half: the chain was written, the node stopped before
/// committing, and on reopening, reconcile commits the kept update the chain
/// holds; the record's epoch equals the chain's.
#[tokio::test]
async fn reconcile_commits_the_update_the_chain_holds_after_a_crash() {
    let mut s = admit_b_directly(setup("reconcile-crash").await).await;
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    drop(s.svc_a);
    let reopened = reopen_store("reconcile-crash", "a", "pw_a");
    let chain_state = s.chain.get(&s.org_id).unwrap();
    let Reconciled::Committed { store, outcome } = reconcile(reopened.data(), s.org_id, chain_state, Vec::new).unwrap() else {
        panic!("committed")
    };
    let record = store.orgs.iter().find(|r| r.org_id == s.org_id).unwrap();
    assert_eq!((record.epoch, record.root_hash, record.org_pub_key), (chain_state.epoch, chain_state.root_hash, chain_state.org_pub_key));
    let ProvisionalChange::ChangeSet { change_set, .. } = &update.change else { panic!("a Change set") };
    assert_eq!(record.kept_change_set.as_ref(), Some(change_set));
    assert!(store.provisional_updates.iter().all(|u| u.resulting_root != update.resulting_root));
    assert!(outcome.revocations.is_empty());
    assert!(outcome.acknowledgements.is_empty());
}

/// verifies: LLR-a8z7r5
///
/// Review finding-7 (2026-10-07), PR-vt244s's crash path: the chain carries
/// A's removal of B, A stopped before committing it, and on reopening the
/// reconciled commit's outcome holds exactly B's revocation notice, so B is
/// still told after recovery.
#[tokio::test]
async fn reconcile_of_a_crash_after_removing_another_device_returns_its_notice() {
    let mut s = admit_b_directly(setup("reconcile-crash-revoke").await).await;
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    drop(s.svc_a);
    let reopened = reopen_store("reconcile-crash-revoke", "a", "pw_a");
    let chain_state = s.chain.get(&s.org_id).unwrap();
    let Reconciled::Committed { store, outcome } = reconcile(reopened.data(), s.org_id, chain_state, Vec::new).unwrap() else {
        panic!("committed, A is still listed")
    };
    let record = store.orgs.iter().find(|r| r.org_id == s.org_id).unwrap();
    assert_eq!((record.epoch, record.root_hash), (chain_state.epoch, chain_state.root_hash));
    let b_device = s.b_device_kp.device_key().unwrap();
    assert_eq!(
        outcome.revocations.iter().map(|notice| (notice.org_id, notice.member_id, notice.device)).collect::<Vec<_>>(),
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
#[tokio::test]
async fn a_commit_that_does_not_remove_this_node_never_obtains_the_device_seeds() {
    let mut s = admit_b_directly(setup("reconcile-no-seeds").await).await;
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let chain_state = s.chain.get(&s.org_id).unwrap();
    let no_seeds = || -> Vec<org_node::DeviceSeed> { panic!("a non-removing commit obtained the device seeds") };
    assert!(matches!(reconcile(s.svc_a.store_data(), s.org_id, chain_state, no_seeds), Ok(Reconciled::Committed { .. })));
}

/// verifies: LLR-fm38ww
///
/// A refusal of the commit step — the kept update's Change set does not
/// verify against the chain — is returned as the error, with no successor.
#[tokio::test]
async fn reconcile_returns_a_refusal_of_the_commit_step_as_the_error() {
    let mut s = admit_b_directly(setup("reconcile-refused").await).await;
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let mut store = s.svc_a.store_data().clone();
    for kept in &mut store.provisional_updates {
        if let ProvisionalChange::ChangeSet { change_set, .. } = &mut kept.change {
            *change_set = vec![0xff; 4];
        }
    }
    assert!(matches!(
        reconcile(&store, s.org_id, s.chain.get(&s.org_id).unwrap(), Vec::new),
        Err(OrgNodeError::MalformedDelta)
    ));
}

/// verifies: LLR-fm38ww
///
/// A reconciled commit that removes this node returns `Removed`, with one
/// acknowledgement and `forget_organisation`'s successor.
#[tokio::test]
async fn reconcile_of_a_removal_signs_and_forgets() {
    let mut s = admit_b_directly(setup("reconcile-removal").await).await;
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    // B holds A's revocation as a provisional update of its own: copy it into B's store data.
    let mut b_store = s.svc_b.store_data().clone();
    b_store.provisional_updates.push(ProvisionalUpdate { persona_id: s.pid_b.clone(), ..update });
    let seeds = vec![s.b_device_kp.device_seed()];
    let Reconciled::Removed { store, acknowledgements } =
        reconcile(&b_store, s.org_id, s.chain.get(&s.org_id).unwrap(), || seeds).unwrap()
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
#[tokio::test]
async fn a_removal_that_cannot_sign_forgets_nothing() {
    let mut s = admit_b_directly(setup("reconcile-removal-unsigned").await).await;
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let mut b_store = s.svc_b.store_data().clone();
    b_store.provisional_updates.push(ProvisionalUpdate { persona_id: s.pid_b.clone(), ..update });
    let before = encoded(&b_store);
    let on_disk_before = store_bytes("reconcile-removal-unsigned", "b");
    for seeds in [vec![], vec![org_node::DeviceSeed::from([99; 32])]] {
        let refused = reconcile(&b_store, s.org_id, s.chain.get(&s.org_id).unwrap(), || seeds);
        assert!(matches!(refused, Err(OrgNodeError::DeviceSecretNotSupplied { org_id }) if org_id == s.org_id));
    }
    assert_eq!(encoded(&b_store), before);
    assert!(b_store.orgs.iter().any(|record| record.org_id == s.org_id));
    assert!(b_store.personas.iter().any(|persona| persona.persona_id == s.pid_b && persona.org_id == Some(s.org_id)));
    assert_eq!(store_bytes("reconcile-removal-unsigned", "b"), on_disk_before);
}

/// verifies: LLR-fm38ww
///
/// The interim service caller reads the chain once, reconciles, and adopts
/// and saves the successor: the reopened store holds the committed record.
#[tokio::test]
async fn the_service_reconcile_adopts_and_saves_the_committed_record() {
    let mut s = admit_b_directly(setup("reconcile-service").await).await;
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let chain_state = s.chain.get(&s.org_id).unwrap();
    assert!(matches!(s.svc_a.reconcile(&mut OsRng, s.org_id).await, Ok(Reconciled::Committed { .. })));
    assert_eq!(rec_of(&s.svc_a, s.org_id).epoch, chain_state.epoch);
    let on_disk = disk_rec_of(&reopen_store("reconcile-service", "a", "pw_a"), s.org_id);
    assert_eq!((on_disk.epoch, on_disk.root_hash), (chain_state.epoch, chain_state.root_hash));
    assert!(matches!(s.svc_a.reconcile(&mut OsRng, s.org_id).await, Ok(Reconciled::InStep)));
}

/// verifies: LLR-gr8x3r
///
/// The service caller refuses an Organisation it does not hold with
/// `OrgNotHeld` before it reads the chain: no chain read is made.
#[tokio::test]
async fn the_service_reconcile_refuses_an_unheld_organisation_without_a_chain_read() {
    let (mut s, counting) = setup_counting("reconcile-unheld").await;
    let unheld = OrgId::new([9; 20]);
    let before = counting.reads();
    assert!(matches!(
        s.svc_a.reconcile(&mut OsRng, unheld).await,
        Err(OrgNodeError::OrgNotHeld { org_id }) if org_id == unheld
    ));
    assert_eq!(counting.reads(), before, "no chain read for an unheld Organisation");
}
