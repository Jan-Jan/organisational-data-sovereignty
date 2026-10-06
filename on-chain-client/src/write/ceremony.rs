//! Genesis and update submission over the `WriteOps` seam
//! (LLR-a2acvh, LLR-z9vugt, LLR-kv27gp, LLR-hun4wf).

use super::calldata::revive_update_runtime_call;
use super::proxy::{map_account_call, proxied};
use super::{AccountId, DispatchOutcome, Genesis, GenesisStep, WriteError, WriteOps};
use crate::h160::h160_of;
use crate::types::{Epoch, OnChainRootHash, OrgAdmin, OrgPubKey};

/// 100 PAS at 10 decimals: existential deposit, fees and revive deposits for
/// a fresh pure proxy.
pub const FUND_AMOUNT: u128 = 1_000_000_000_000;

fn failed(step: GenesisStep, e: WriteError) -> WriteError {
    WriteError::Step { step, reason: e.to_string() }
}

fn executed(step: GenesisStep, outcome: Result<DispatchOutcome, WriteError>) -> Result<(), WriteError> {
    match outcome {
        Ok(DispatchOutcome::Executed) => Ok(()),
        Ok(DispatchOutcome::ApprovalRecorded) => {
            Err(WriteError::Step { step, reason: "multisig approval recorded; the call did not execute".into() })
        }
        Err(e) => Err(failed(step, e)),
    }
}

/// Stand up an Organisation's slot: create the pure proxy under the
/// controller, fund it from the signatory, map it in pallet-revive, record
/// the genesis root and key at epoch zero — each step only after the previous
/// one succeeded (LLR-a2acvh); a failure names its step (LLR-z9vugt).
pub async fn genesis<O: WriteOps>(
    ops: &O,
    signatory: &O::Signer,
    co_signatories: &[AccountId],
    contract: [u8; 20],
    root: OnChainRootHash,
    key: OrgPubKey,
) -> Result<Genesis, WriteError> {
    let proxy = ops.create_pure(signatory, co_signatories).await.map_err(|e| failed(GenesisStep::CreatePure, e))?;
    ops.fund(signatory, proxy, FUND_AMOUNT).await.map_err(|e| failed(GenesisStep::Fund, e))?;
    executed(
        GenesisStep::MapAccount,
        ops.dispatch(signatory, co_signatories, proxied(proxy, map_account_call())).await,
    )?;
    let record = revive_update_runtime_call(contract, root, key, Epoch(0));
    executed(GenesisStep::RecordGenesis, ops.dispatch(signatory, co_signatories, proxied(proxy, record)).await)?;
    Ok(Genesis { proxy, admin: OrgAdmin(h160_of(proxy.0)) })
}

/// Submit one update to the Organisation's slot through its proxy; `Ok` only
/// when the dispatch executed (LLR-kv27gp, LLR-hun4wf).
#[allow(clippy::too_many_arguments)]
pub async fn submit_update<O: WriteOps>(
    ops: &O,
    signatory: &O::Signer,
    co_signatories: &[AccountId],
    contract: [u8; 20],
    proxy: AccountId,
    root: OnChainRootHash,
    key: OrgPubKey,
    expected_epoch: Epoch,
) -> Result<(), WriteError> {
    let call = proxied(proxy, revive_update_runtime_call(contract, root, key, expected_epoch));
    match ops.dispatch(signatory, co_signatories, call).await? {
        DispatchOutcome::Executed => Ok(()),
        DispatchOutcome::ApprovalRecorded => Err(WriteError::PendingApproval),
    }
}
