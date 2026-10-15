//! Submission (SDD-erzj3m), moved from the app's `submit.rs`: org-io writes
//! org-node's provisional update to the chain through on-chain-client,
//! reads back the state the write left (LLR-9r2bxd), and only then asks
//! org-node to commit, and to send; when the write fails it asks org-node for
//! neither and reports the failure (LLR-qhjp6g); when it does not finish
//! within `WRITE_TIMEOUT` it asks for neither and reports the outcome
//! unknown (LLR-be3zv9); and when the read-back fails, or finds
//! no state, or org-node's commit refuses for any other reason, it says the
//! write executed and that nothing was committed.

use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;
use on_chain_client::write::subxt_ops::{FinalitySink, SubxtWriteOps};
use on_chain_client::write::{genesis, submit_update, AccountId, WriteWindow};
use on_chain_client::{OnChainRootHash, OrgPubKey, OrgRegistryClient};
use org_node::service::OrgService;
use org_node::{ChainAccount, Epoch, OrgId, OrgNodeError, OrgPublicKey, PersonaId, RootHash};
use rand_core::{CryptoRng, RngCore};

use crate::chain_read::StateReader;
use crate::custody::SignatoryKey;
use crate::peer_addr::PeerAddr;
use crate::publish::{self, PublishState, Reconciliation, Settled};
use crate::view::{BuiltUpdate, Submitted};
use crate::{LocalStore, StoreFailure};

/// How long one chain write may take before org-io gives up on it
/// (LLR-be3zv9): the settle timeout org-node used before the write moved.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(90);

/// Why the chain is not configured, as every refused write and read says it
/// (REQ-8zuka3): a development build names the variables to set; a build
/// without `dev-seed` never reads the seed, so it says so instead.
#[cfg(feature = "dev-seed")]
pub(crate) const NOT_CONFIGURED: &str = "chain not configured: set ODS_CHAIN_WS, ODS_CONTRACT_H160, ODS_ADMIN_SEED";
#[cfg(not(feature = "dev-seed"))]
pub(crate) const NOT_CONFIGURED: &str = crate::connect::DEV_SEED_NOT_BUILT;

/// The chain write as org-io performs it (org-io's own seam for the
/// submission tests).
#[async_trait]
pub trait ChainWriter: Send + Sync {
    /// Stand up the Organisation's slot: its id and its proxy account.
    async fn genesis(&self, root: RootHash, key: OrgPublicKey) -> Result<(OrgId, ChainAccount), String>;
    /// The block an update write is signed mortal from: the latest
    /// finalised block (LLR-2yjuaj).
    async fn write_window(&self) -> Result<WriteWindow, String>;
    /// Move the slot, through `proxy`, from `expected_epoch`, mortal from
    /// `window`'s birth block.
    async fn update(
        &self,
        proxy: ChainAccount,
        root: RootHash,
        key: OrgPublicKey,
        expected_epoch: Epoch,
        window: WriteWindow,
    ) -> Result<(), String>;
}

/// Why a bounded write did not succeed.
enum WriteNotDone {
    /// The writer reported a failure.
    Failed(String),
    /// The write did not finish within `WRITE_TIMEOUT`: it may yet execute.
    TimedOut,
}

impl WriteNotDone {
    /// The report, ending with what was not done. A timeout is treated as
    /// not committed (LLR-be3zv9) but reported as an unknown outcome, since
    /// the write may have executed.
    fn report(self, nothing_done: &str) -> String {
        match self {
            WriteNotDone::Failed(error) => format!("chain write failed; {nothing_done}: {error}"),
            WriteNotDone::TimedOut => format!(
                "chain write outcome unknown: it timed out after {} s and may yet execute on chain; {nothing_done}",
                WRITE_TIMEOUT.as_secs()
            ),
        }
    }
}

/// One writer call, bounded by `WRITE_TIMEOUT`.
async fn bounded<T>(write: impl Future<Output = Result<T, String>>) -> Result<T, WriteNotDone> {
    match tokio::time::timeout(WRITE_TIMEOUT, write).await {
        Ok(written) => written.map_err(WriteNotDone::Failed),
        Err(_) => Err(WriteNotDone::TimedOut),
    }
}

/// The production writer: on-chain-client's, over subxt, signing with the
/// user's own signatory key, which only this value holds (LLR-u2pk5y).
pub struct OnChainWriter {
    ops: SubxtWriteOps<FinalitySink>,
    /// The finalised-block read the write window comes from.
    registry: OrgRegistryClient,
    signatory: SignatoryKey,
    co_signatories: Vec<AccountId>,
    contract: [u8; 20],
}

impl OnChainWriter {
    /// Only `OrgIo::connect` builds one, and only under `dev-seed`, where the
    /// key comes from (REQ-8zuka3).
    #[cfg(feature = "dev-seed")]
    pub(crate) fn new(
        ops: SubxtWriteOps<FinalitySink>,
        registry: OrgRegistryClient,
        signatory: SignatoryKey,
        co_signatories: Vec<AccountId>,
        contract: [u8; 20],
    ) -> Self {
        Self { ops, registry, signatory, co_signatories, contract }
    }
}

#[async_trait]
impl ChainWriter for OnChainWriter {
    async fn genesis(&self, root: RootHash, key: OrgPublicKey) -> Result<(OrgId, ChainAccount), String> {
        let founded = genesis(
            &self.ops,
            self.signatory.keypair(),
            &self.co_signatories,
            self.contract,
            OnChainRootHash(*root.as_bytes()),
            OrgPubKey(*key.as_bytes()),
        )
        .await
        .map_err(|error| error.to_string())?;
        Ok((OrgId::new(founded.admin.0), ChainAccount::new(founded.proxy.0)))
    }

    async fn write_window(&self) -> Result<WriteWindow, String> {
        self.registry.write_window().await.map_err(|error| error.to_string())
    }

    async fn update(
        &self,
        proxy: ChainAccount,
        root: RootHash,
        key: OrgPublicKey,
        expected_epoch: Epoch,
        window: WriteWindow,
    ) -> Result<(), String> {
        submit_update(
            &self.ops,
            self.signatory.keypair(),
            &self.co_signatories,
            self.contract,
            AccountId(*proxy.as_bytes()),
            OnChainRootHash(*root.as_bytes()),
            OrgPubKey(*key.as_bytes()),
            on_chain_client::Epoch(expected_epoch.get()),
            window,
        )
        .await
        .map_err(|error| error.to_string())
    }
}

/// The writer when the chain is not configured: every write refused.
pub struct WriterNotConfigured;

#[async_trait]
impl ChainWriter for WriterNotConfigured {
    async fn genesis(&self, _: RootHash, _: OrgPublicKey) -> Result<(OrgId, ChainAccount), String> {
        Err(NOT_CONFIGURED.into())
    }

    async fn write_window(&self) -> Result<WriteWindow, String> {
        Err(NOT_CONFIGURED.into())
    }

    async fn update(&self, _: ChainAccount, _: RootHash, _: OrgPublicKey, _: Epoch, _: WriteWindow) -> Result<(), String> {
        Err(NOT_CONFIGURED.into())
    }
}

/// org-node's commit refused after a write that executed (LLR-9r2bxd). The
/// write is on-chain whatever the refusal, so every refusal is reported as a
/// failed read-back is: the write executed, nothing was committed, and the
/// refusal is the cause. A read-back that found no state (`OrgNotOnChain`)
/// says so.
fn refused_after_write(error: OrgNodeError, nothing_done: &str) -> String {
    match error {
        OrgNodeError::OrgNotOnChain => {
            format!("the chain write executed, but reading it back found no state; {nothing_done}: {error}")
        }
        other => format!("the chain write executed, but the commit was refused; {nothing_done}: {other}"),
    }
}

/// A commit whose store change was not written (LLR-murjr6, org-node's
/// save-after-mutate report): the chain write executed and the
/// commit was made, so it is not reported as "nothing committed"; the
/// failure says whether the Persona was reloaded from its file.
fn committed_but_not_stored(failure: &StoreFailure, after: &str) -> String {
    format!("the chain write executed and the commit was made, but {failure}{after}")
}

/// Found an Organisation: build the genesis update, write it, read back the
/// state it left, commit against that state.
pub async fn found_organisation<R: RngCore + CryptoRng + Send>(
    service: &mut OrgService,
    local: &LocalStore,
    writer: &dyn ChainWriter,
    reader: &dyn StateReader,
    rng: &mut R,
    persona_id: &PersonaId,
) -> Result<OrgId, String> {
    let (update, kept) = service.create_organisation(rng, persona_id).map_err(|error| error.to_string())?;
    local
        .write(service, kept)
        .map_err(|failure| format!("the genesis update was built, but {failure}; nothing written or committed"))?;
    let (org_id, proxy) = bounded(writer.genesis(update.resulting_root, update.org_pub_key))
        .await
        .map_err(|not_done| not_done.report("nothing committed"))?;
    let chain_state = reader
        .read_state(org_id)
        .await
        .map_err(|error| format!("the chain write executed, but reading it back failed; nothing committed: {error}"))?;
    let (_, committed) = service
        .commit_genesis(rng, persona_id, org_id, proxy, chain_state)
        .map_err(|error| refused_after_write(error, "nothing committed"))?;
    // A failed write is a storage failure, not a refused commit: the chain
    // write executed and org-node committed; the Persona is reloaded from
    // its file (LLR-murjr6, org-node's save-after-mutate report).
    local.write(service, committed).map_err(|failure| committed_but_not_stored(&failure, ""))?;
    Ok(org_id)
}

/// The refusal of an update org-node does not hold on the record's current
/// root (review round 3, finding 2).
pub(crate) const NOT_HELD_ON_CURRENT_ROOT: &str = "refused before any write: org-node holds no such provisional update \
     built on the Organisation's current root (altered, discarded, or built before another update committed); \
     nothing written, committed or sent";

/// The refusal of an update whose own earlier write, of unknown outcome, the
/// settle has just found not executed and discarded (review round 1,
/// finding-5): its window passed, or another update committed first (review
/// round 2, finding-2), so the report names neither.
pub(crate) const NOT_EXECUTED_RESUBMITTED: &str = "refused before any write: this update's earlier chain write, whose \
     outcome was unknown, did not execute and the update was discarded; rebuild the update and submit it again; \
     nothing written, committed or sent";

/// Write the update `update` names, as org-node holds it, to the chain;
/// only once that executed, read back the state it left, commit against it
/// and send the committed update to every recipient the commit names
/// (LLR-qhjp6g, LLR-9r2bxd; `send::send_committed`).
/// Before any write it refuses an Organisation the store does not hold
/// (`OrgNotHeld`), a record with no proxy account, and an update org-node
/// does not hold built on the record's current root. org-node builds the
/// body for each recipient inside the commit; this returns the committed
/// update's public data only. An earlier write for the
/// Organisation whose outcome is unknown is settled first, and unless it
/// was not executed nothing is written (LLR-nqvbb6); a write that times out
/// is recorded as unknown, with the address the submission was given
/// (LLR-2yjuaj): it names no recipient, so the late commit goes to every
/// recipient as this one would have.
#[allow(clippy::too_many_arguments)]
pub async fn submit_commit_send<R: RngCore + CryptoRng + Send>(
    service: &mut OrgService,
    publish: &mut PublishState,
    local: &LocalStore,
    writer: &dyn ChainWriter,
    reader: &dyn StateReader,
    rng: &mut R,
    update: &BuiltUpdate,
    peer_addr: Option<PeerAddr>,
) -> Result<Submitted, String> {
    let org_id = update.org_id;
    let resubmitted = publish.is_unknown_write(update);
    match publish::settle_unknown(publish, service, local, reader, rng, org_id).await {
        Settled::NotExecuted { .. } if resubmitted => return Err(NOT_EXECUTED_RESUBMITTED.into()),
        // An executed write whose update was already committed is settled
        // (review round 1, finding-1): the record is in step with the chain.
        Settled::NoUnknownWrite | Settled::NotExecuted { .. } | Settled::Reconciled(Reconciliation::InStep { .. }) => {}
        settled => return Err(settled.submission_report()),
    }
    // Read by reference and copy only public fields: no record and no held
    // update, with the Organisation private key each holds, is cloned here
    // (review round 4, finding 2; org-node's `holds_provisional`).
    let record = service
        .list_orgs()
        .into_iter()
        .find(|record| record.org_id == org_id)
        .ok_or_else(|| OrgNodeError::OrgNotHeld { org_id }.to_string())?;
    let (proxy, epoch, current_root) = (record.proxy_account, record.epoch, record.root_hash);
    let proxy =
        proxy.ok_or("this device holds no proxy account for the Organisation; only its founding device can submit")?;
    let held = service.holds_provisional(org_id, update.base_root, update.resulting_root, update.org_pub_key);
    // The root comparison is defensive: every path that moves a record's root
    // also discards the updates built on the old one, so today `held` alone
    // refuses a stale update (LLR-qhjp6g, note of 2026-10-08).
    if !held || update.base_root != current_root {
        return Err(NOT_HELD_ON_CURRENT_ROOT.into());
    }
    // A refusal of the write step, reported as a failed write is (a handle
    // with no chain refuses here, LLR-qhjp6g), and saying no write was made
    // (LLR-2yjuaj).
    let window = writer.write_window().await.map_err(|error| {
        format!(
            "chain write failed; nothing committed or sent: reading the chain's write window failed, so no write \
             was made: {error}"
        )
    })?;
    if let Err(not_done) = bounded(writer.update(proxy, update.resulting_root, update.org_pub_key, epoch, window)).await {
        if matches!(not_done, WriteNotDone::TimedOut) {
            publish.record_unknown(*update, peer_addr, window.last_valid_block(), epoch);
        }
        return Err(not_done.report("nothing committed or sent"));
    }
    let chain_state = reader.read_state(org_id).await.map_err(|error| {
        format!("the chain write executed, but reading it back failed; nothing committed or sent: {error}")
    })?;
    let (outcome, change) = service
        .commit_update(rng, org_id, chain_state, |keys| local.device_secrets_for_signing(keys))
        .map_err(|error| refused_after_write(error, "nothing committed or sent"))?;
    // A failed write is a storage failure, as in `found_organisation`
    // (LLR-murjr6).
    local
        .store(service, change)
        .map_err(|failure| committed_but_not_stored(&failure, &format!(" (epoch {}); nothing sent", outcome.epoch.get())))?;
    // Every body the commit built (org-node builds them), sent unchanged to
    // its recipient, from the endpoint of the Persona the commit names as
    // its sender, never another Persona's (LLR-d2pzzk). A failed send is
    // reported per Device and leaves the commit in place (LLR-znhmh2,
    // LLR-r9sh6k).
    let sender = service.persona_public_keys(&outcome.sender).map(|(_, device)| device).map_err(|error| error.to_string());
    let sends = local.send_committed(sender, &outcome, peer_addr).await;
    Ok(Submitted { org_id: outcome.org_id, epoch: outcome.epoch, root: outcome.root, sends })
}
