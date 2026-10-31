//! Genesis and update submission over the `WriteOps` seam: the genesis in
//! two steps, `create_proxy` and `found_at` (LLR-tge9q8, LLR-q6buam,
//! LLR-chy957), `genesis` their composition (LLR-hea6y2), and one update
//! (LLR-kv27gp, LLR-hun4wf).

use super::calldata::revive_update_runtime_call;
use super::proxy::{map_account_call, proxied};
use super::{AccountId, DispatchOutcome, Genesis, GenesisStep, WriteError, WriteOps, WriteWindow};
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

/// The genesis's first step: create the pure proxy an Organisation is
/// founded at, under the controller (LLR-tge9q8) — one `create_pure` call,
/// whose failure is the `CreatePure` step (LLR-chy957).
pub async fn create_proxy<O: WriteOps>(
    ops: &O,
    signatory: &O::Signer,
    co_signatories: &[AccountId],
) -> Result<AccountId, WriteError> {
    ops.create_pure(signatory, co_signatories).await.map_err(|error| failed(GenesisStep::CreatePure, error))
}

/// The genesis's second step, at a proxy `create_proxy` made (LLR-q6buam):
/// read the account the proxy's H160 maps to, read the proxy's free balance,
/// fund it from the signatory, map it in pallet-revive and record the
/// genesis root and key at an expected epoch of zero — each only after the
/// previous one succeeded; a failure names its step (LLR-chy957). It never
/// creates a proxy.
///
/// What is done is decided from the two reads alone (REQ-6vh3ht): a proxy
/// mapped to itself is neither funded, nor its balance read (LLR-jk76wf),
/// nor mapped again (LLR-95puh9); an unmapped proxy holding at least
/// `FUND_AMOUNT` is not funded again (LLR-jk76wf). The record is never
/// skipped and no Organisation slot is read: on a slot that already holds a
/// genesis the contract refuses it, and that is `RecordGenesis` (LLR-g8qevz).
pub async fn found_at<O: WriteOps>(
    ops: &O,
    signatory: &O::Signer,
    co_signatories: &[AccountId],
    contract: [u8; 20],
    proxy: AccountId,
    root: OnChainRootHash,
    key: OrgPubKey,
) -> Result<Genesis, WriteError> {
    let admin = OrgAdmin(h160_of(proxy.0));
    let mapped = ops.mapped_account(admin).await.map_err(|error| failed(GenesisStep::MapAccount, error))?;
    if mapped.is_some_and(|account| account != proxy) {
        return Err(WriteError::Step {
            step: GenesisStep::MapAccount,
            reason: "the proxy's address is mapped to another account".into(),
        });
    }
    if mapped.is_none() {
        let balance = ops.free_balance(proxy).await.map_err(|error| failed(GenesisStep::Fund, error))?;
        if balance < FUND_AMOUNT {
            ops.fund(signatory, proxy, FUND_AMOUNT).await.map_err(|error| failed(GenesisStep::Fund, error))?;
        }
        executed(
            GenesisStep::MapAccount,
            ops.dispatch(signatory, co_signatories, proxied(proxy, map_account_call()), None).await,
        )?;
    }
    let record = revive_update_runtime_call(contract, root, key, Epoch(0));
    executed(GenesisStep::RecordGenesis, ops.dispatch(signatory, co_signatories, proxied(proxy, record), None).await)?;
    Ok(Genesis { proxy, admin })
}

/// Stand up an Organisation's slot in one call (REQ-6jefu2): `create_proxy`,
/// then, only when that succeeded, `found_at` at the proxy it returned, whose
/// result this is (LLR-hea6y2).
pub async fn genesis<O: WriteOps>(
    ops: &O,
    signatory: &O::Signer,
    co_signatories: &[AccountId],
    contract: [u8; 20],
    root: OnChainRootHash,
    key: OrgPubKey,
) -> Result<Genesis, WriteError> {
    let proxy = create_proxy(ops, signatory, co_signatories).await?;
    found_at(ops, signatory, co_signatories, contract, proxy, root, key).await
}

/// Submit one update to the Organisation's slot through its proxy, mortal
/// from `window`'s birth block (LLR-zj4jju); `Ok` only when the dispatch
/// executed (LLR-kv27gp, LLR-hun4wf).
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
    window: WriteWindow,
) -> Result<(), WriteError> {
    let call = proxied(proxy, revive_update_runtime_call(contract, root, key, expected_epoch));
    match ops.dispatch(signatory, co_signatories, call, Some(window)).await? {
        DispatchOutcome::Executed => Ok(()),
        DispatchOutcome::ApprovalRecorded => Err(WriteError::PendingApproval),
    }
}
