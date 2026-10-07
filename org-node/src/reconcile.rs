//! SDD-8cpyfa: reconciling a held record with the Organisation state the
//! caller read. A pure function over values (LLR-xgefn8).
use crate::chain::OrgState;
use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::revocation::Acknowledgement;
use crate::service::{commit_step, CommitOutcome};
use crate::store::StoreData;
use crate::types::{DeviceSeed, Epoch};

/// What `reconcile` decided (LLR-gr8x3r). Only a commit carries a successor
/// store.
#[derive(Debug)]
pub enum Reconciled {
    InStep,
    Committed { store: StoreData, outcome: CommitOutcome },
    Removed { store: StoreData, acknowledgements: Vec<Acknowledgement> },
    Behind { chain_epoch: Epoch },
}

/// Decide `org_id`'s record against `chain`, the Organisation state the
/// caller read, from the record and `chain` alone: an older state is stale;
/// the record's epoch is in step with its root and in conflict with any
/// other; a newer state is committed when this node keeps the update whose
/// root and key `chain` holds, else the record is behind. The commit runs
/// the commit step `commit_update` runs, `device_seeds` called only by a
/// commit that removes this node, to sign its acknowledgements. Only a commit returns a successor; the
/// store is borrowed and unchanged. LLR-gr8x3r, LLR-fm38ww.
pub fn reconcile(
    store: &StoreData,
    org_id: OrgId,
    chain: OrgState,
    device_seeds: impl FnOnce() -> Vec<DeviceSeed>,
) -> Result<Reconciled, OrgNodeError> {
    let record = store.orgs.iter().find(|record| record.org_id == org_id).ok_or(OrgNodeError::OrgNotHeld { org_id })?;
    if chain.epoch < record.epoch {
        return Err(OrgNodeError::StaleChainState { org_id, chain_epoch: chain.epoch, record_epoch: record.epoch });
    }
    if chain.epoch == record.epoch {
        return if chain.root_hash == record.root_hash {
            Ok(Reconciled::InStep)
        } else {
            Err(OrgNodeError::ChainStateConflict { org_id })
        };
    }
    let Some(update) = store.held_update_for(org_id, &chain) else {
        return Ok(Reconciled::Behind { chain_epoch: chain.epoch });
    };
    let committed = commit_step(store, org_id, update, &chain, device_seeds)?;
    Ok(if committed.removed {
        Reconciled::Removed { store: committed.store, acknowledgements: committed.outcome.acknowledgements }
    } else {
        Reconciled::Committed { store: committed.store, outcome: committed.outcome }
    })
}
