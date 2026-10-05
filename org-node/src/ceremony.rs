//! Genesis ceremony: stand up an org's on-chain slot. Composes the chain_write
//! primitives. Block production is injected via BlockSink so this works against
//! chopsticks (mine) and live chains (wait).
#![cfg(feature = "chain")]
use subxt::{OnlineClient, config::PolkadotConfig};
use subxt_signer::sr25519::Keypair;
use org_members::RootHash;

use crate::chain_write::calldata::revive_update_runtime_call;
use crate::chain_write::multisig::{dispatch_org_call, fund, FUND_AMOUNT};
use crate::chain_write::proxy::{create_pure, map_account_call, org_id_of, proxied, BlockSink};
use crate::chain_write::WriteError;
use crate::ids::OrgId;
use crate::types::{ChainAccount, Epoch, OrgPublicKey};

/// The on-chain identity produced by genesis.
pub struct GenesisOutcome {
    /// Pure-proxy AccountId32.
    pub p: ChainAccount,
    /// org_id = h160_of(P) — the contract slot key.
    pub org_id: OrgId,
}

/// Run the full genesis ceremony for an org. The controller of the pure proxy
/// `P` is selected by `others`: empty ⇒ the admin controls `P` directly (single
/// admin, no multisig); non-empty ⇒ a threshold-1 ("any one of N") multisig.
///
/// Steps:
/// 1. create pure proxy P (signed directly by the admin, or by the multisig)
/// 2. fund P
/// 3. map_account from P (pallet-revive prerequisite), dispatched as P
/// 4. submit genesis update(root, orgPubKey, expectedEpoch=0), dispatched as P
///
/// `funder` pays for P's existential deposit / fees. `admin` is the sole signer;
/// `others` are the co-signatories of the controlling multisig (empty slice for a
/// single-admin / direct org). Every step must EXECUTE (never be left pending), so
/// each asserts `DispatchOutcome::Executed` via `into_executed()`.
#[allow(clippy::too_many_arguments)]
pub async fn genesis_ceremony(
    sink: &dyn BlockSink,
    api: &OnlineClient<PolkadotConfig>,
    contract_h160: [u8; 20],
    funder: &Keypair,
    admin: &Keypair,
    others: &[ChainAccount],
    genesis_root: RootHash,
    org_pub_key: OrgPublicKey,
) -> Result<GenesisOutcome, WriteError> {
    // Each step submits, drives the chain via `sink`, and waits for ITS extrinsic
    // to finalize successfully (ExtrinsicFailed surfaces as an error) — no longer
    // a separate fire-and-forget submit + sink.settle().
    // 1. Pure proxy.
    let p = create_pure(sink, api, admin, others).await?;
    // 2. Fund P.
    fund(sink, api, funder, p, FUND_AMOUNT).await?;
    // 3. map_account from P.
    dispatch_org_call(sink, api, admin, others, proxied(p, map_account_call()))
        .await?
        .into_executed()?;
    // 4. Genesis update (expectedEpoch = 0).
    let call = revive_update_runtime_call(contract_h160, genesis_root, org_pub_key, Epoch::new(0));
    dispatch_org_call(sink, api, admin, others, proxied(p, call))
        .await?
        .into_executed()?;

    let org_id = org_id_of(p);
    Ok(GenesisOutcome { p, org_id })
}
