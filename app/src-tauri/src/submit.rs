//! Submitting org-node's provisional updates (REQ-nfr3n2): the app writes the
//! chain through on-chain-client, then asks org-node to commit and send;
//! when the write fails, or does not finish within `WRITE_TIMEOUT`, it asks
//! org-node for neither and reports the failure (LLR-qhjp6g, LLR-be3zv9).

use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;
use on_chain_client::write::subxt_ops::{FinalitySink, SubxtWriteOps};
use on_chain_client::write::{genesis, submit_update, AccountId};
use on_chain_client::{OnChainRootHash, OrgPubKey};
use org_node::service::OrgService;
use org_node::store::ProvisionalUpdate;
use org_node::{
    ChainAccount, CommitOutcome, DevicePublicKey, Epoch, OrgId, OrgNodeError, OrgPublicKey, PersonaId, RootHash,
};
use rand::{CryptoRng, RngCore};

/// How long one chain write may take before the app gives up on it
/// (LLR-be3zv9): the settle timeout org-node used before the write moved.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(90);

const NOT_CONFIGURED: &str = "chain not configured: set ODS_CHAIN_WS, ODS_CONTRACT_H160, ODS_ADMIN_SEED";

/// The chain write as the app performs it.
#[async_trait]
pub trait ChainWriter: Send + Sync {
    /// Stand up the Organisation's slot: its id and its proxy account.
    async fn genesis(&self, root: RootHash, key: OrgPublicKey) -> Result<(OrgId, ChainAccount), String>;
    /// Move the slot, through `proxy`, from `expected_epoch`.
    async fn update(&self, proxy: ChainAccount, root: RootHash, key: OrgPublicKey, expected_epoch: Epoch)
        -> Result<(), String>;
}

/// One writer call, bounded by `WRITE_TIMEOUT`; a timeout is a failed write.
async fn bounded<T>(write: impl Future<Output = Result<T, String>>) -> Result<T, String> {
    tokio::time::timeout(WRITE_TIMEOUT, write)
        .await
        .map_err(|_| format!("the chain write timed out after {} s", WRITE_TIMEOUT.as_secs()))?
}

/// The production writer: on-chain-client's, over subxt.
pub struct OnChainWriter {
    pub ops: SubxtWriteOps<FinalitySink>,
    pub signatory: subxt_signer::sr25519::Keypair,
    pub co_signatories: Vec<AccountId>,
    pub contract: [u8; 20],
}

#[async_trait]
impl ChainWriter for OnChainWriter {
    async fn genesis(&self, root: RootHash, key: OrgPublicKey) -> Result<(OrgId, ChainAccount), String> {
        let g = genesis(
            &self.ops,
            &self.signatory,
            &self.co_signatories,
            self.contract,
            OnChainRootHash(*root.as_bytes()),
            OrgPubKey(*key.as_bytes()),
        )
        .await
        .map_err(|e| e.to_string())?;
        Ok((OrgId::new(g.admin.0), ChainAccount::new(g.proxy.0)))
    }

    async fn update(&self, proxy: ChainAccount, root: RootHash, key: OrgPublicKey, expected_epoch: Epoch) -> Result<(), String> {
        submit_update(
            &self.ops,
            &self.signatory,
            &self.co_signatories,
            self.contract,
            AccountId(*proxy.as_bytes()),
            OnChainRootHash(*root.as_bytes()),
            OrgPubKey(*key.as_bytes()),
            on_chain_client::Epoch(expected_epoch.get()),
        )
        .await
        .map_err(|e| e.to_string())
    }
}

/// The writer when the chain is not configured: every write refused.
pub struct WriterNotConfigured;

#[async_trait]
impl ChainWriter for WriterNotConfigured {
    async fn genesis(&self, _: RootHash, _: OrgPublicKey) -> Result<(OrgId, ChainAccount), String> {
        Err(NOT_CONFIGURED.into())
    }

    async fn update(&self, _: ChainAccount, _: RootHash, _: OrgPublicKey, _: Epoch) -> Result<(), String> {
        Err(NOT_CONFIGURED.into())
    }
}

/// Found an Organisation: build the genesis update, write it, commit it.
pub async fn found_organisation<R: RngCore + CryptoRng + Send>(
    svc: &mut OrgService,
    writer: &dyn ChainWriter,
    rng: &mut R,
    persona_id: &PersonaId,
) -> Result<OrgId, String> {
    let update = svc.create_organisation(rng, persona_id).map_err(|e| e.to_string())?;
    let (org_id, proxy) = bounded(writer.genesis(update.resulting_root, update.org_pub_key))
        .await
        .map_err(|e| format!("chain write failed; nothing committed: {e}"))?;
    svc.commit_genesis(rng, persona_id, org_id, proxy).await.map_err(|e| e.to_string())?;
    Ok(org_id)
}

/// Write `update` to the chain; only once that executed, commit it and send
/// it to `recipient` (REQ-nfr3n2). org-node chooses the kind of message from
/// its committed record; the app passes it no key, secret or invite
/// identifier (LLR-q225ws).
pub async fn submit_commit_send<R: RngCore + CryptoRng + Send>(
    svc: &mut OrgService,
    writer: &dyn ChainWriter,
    rng: &mut R,
    update: &ProvisionalUpdate,
    recipient: DevicePublicKey,
    peer_addr: Option<iroh::EndpointAddr>,
) -> Result<CommitOutcome, String> {
    let org_id = update.org_id.ok_or("a genesis update is founded, not submitted")?;
    let rec = svc
        .list_orgs()
        .iter()
        .find(|o| o.org_id == org_id)
        .cloned()
        .ok_or_else(|| OrgNodeError::OrgNotOnChain.to_string())?;
    let proxy = rec
        .proxy_account
        .ok_or("this device holds no proxy account for the Organisation; only its founding device can submit")?;
    bounded(writer.update(proxy, update.resulting_root, update.org_pub_key, rec.epoch))
        .await
        .map_err(|e| format!("chain write failed; nothing committed or sent: {e}"))?;
    let outcome = svc.commit_update(rng, org_id).await.map_err(|e| e.to_string())?;
    svc.send_update(&outcome.outgoing, recipient, peer_addr)
        .await
        .map_err(|e| format!("committed at epoch {}, but the send failed: {e}", outcome.epoch.get()))?;
    Ok(outcome)
}
