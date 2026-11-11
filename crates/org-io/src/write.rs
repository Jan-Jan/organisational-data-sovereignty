//! The chain write (SDD-mk39e5): org-io's seam to on-chain-client's writer, the production writer
//! over subxt, the writer of a handle with no chain, the write's bound, and
//! `write_organisation_to_chain` (an update, or a founding in two steps) with the unknown
//! writes it records. Chain IO only: no state read, no adoption, no store change.

use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;
use on_chain_client::write::subxt_ops::{FinalitySink, SubxtWriteOps};
use on_chain_client::write::{submit_update, AccountId, WriteError, WriteWindow};
use on_chain_client::{ClientError, OnChainRootHash, OrgPubKey, OrgRegistryClient};
use org_node::store::Origin;
use org_node::{ChainAccount, DevicePublicKey, Epoch, OrgId, OrgNodeError, OrgPublicKey, RootHash};

use crate::custody::SignatoryKey;
use crate::{Cause, OrgIo, StoreFailure};

/// How long one chain write may take before org-io gives up on it
/// (LLR-d3farr).
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(90);

/// Why the chain is not configured, as every refused write and read says it
/// (REQ-8zuka3): a development build names the variables to set; a build
/// without `dev-seed` never reads the seed, so it says so instead.
#[cfg(feature = "dev-seed")]
pub(crate) const NOT_CONFIGURED: &str = "chain not configured: set ODS_CHAIN_WS, ODS_CONTRACT_H160, ODS_ADMIN_SEED";
#[cfg(not(feature = "dev-seed"))]
pub(crate) const NOT_CONFIGURED: &str = crate::connect::DEV_SEED_NOT_BUILT;

/// Why one `ChainWriter` call failed (LLR-a66ufr, LLR-sq5pbq): the error it
/// received, kept, typed.
#[derive(Debug)]
pub enum ChainWriteError {
    /// The handle has no chain: every write is refused (REQ-8zuka3).
    NotConfigured,
    /// on-chain-client's write failed; a genesis step's failure names the step.
    Write(WriteError),
    /// on-chain-client's read of the write window failed.
    Window(ClientError),
}

impl std::fmt::Display for ChainWriteError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfigured => formatter.write_str(NOT_CONFIGURED),
            Self::Write(WriteError::Step { step, reason }) => write!(formatter, "genesis step {step:?} did not execute: {reason}"),
            Self::Write(error) => write!(formatter, "{error}"),
            Self::Window(error) => write!(formatter, "{error}"),
        }
    }
}

/// The error on-chain-client returned, typed; none when the chain is not configured.
impl std::error::Error for ChainWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotConfigured => None,
            Self::Write(error) => Some(error),
            Self::Window(error) => Some(error),
        }
    }
}

/// The chain write as org-io performs it (org-io's own seam for the
/// submission tests).
#[async_trait]
pub trait ChainWriter: Send + Sync {
    /// Create the founding's pure proxy under the controller (on-chain-client's `create_proxy`):
    /// the Organisation identifier `h160_of` gives for it and the proxy account.
    async fn create_proxy(&self) -> Result<(OrgId, ChainAccount), ChainWriteError>;
    /// Fund, map and record the genesis `root` and `key` at `proxy` (on-chain-client's `found_at`,
    /// which skips what the chain shows done); a step's failure names the step.
    async fn found_at(&self, proxy: ChainAccount, root: RootHash, key: OrgPublicKey) -> Result<(), ChainWriteError>;
    /// The block an update write is signed mortal from: the latest
    /// finalised block (LLR-a66ufr).
    async fn write_window(&self) -> Result<WriteWindow, ChainWriteError>;
    /// Move the slot, through `proxy`, from `expected_epoch`, mortal from
    /// `window`'s birth block.
    async fn update(
        &self,
        proxy: ChainAccount,
        root: RootHash,
        key: OrgPublicKey,
        expected_epoch: Epoch,
        window: WriteWindow,
    ) -> Result<(), ChainWriteError>;
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
    async fn create_proxy(&self) -> Result<(OrgId, ChainAccount), ChainWriteError> {
        let proxy = on_chain_client::write::create_proxy(&self.ops, self.signatory.keypair(), &self.co_signatories)
            .await
            .map_err(ChainWriteError::Write)?;
        Ok((OrgId::new(on_chain_client::h160_of(proxy.0)), ChainAccount::new(proxy.0)))
    }

    async fn found_at(&self, proxy: ChainAccount, root: RootHash, key: OrgPublicKey) -> Result<(), ChainWriteError> {
        on_chain_client::write::found_at(
            &self.ops,
            self.signatory.keypair(),
            &self.co_signatories,
            self.contract,
            AccountId(*proxy.as_bytes()),
            OnChainRootHash(*root.as_bytes()),
            OrgPubKey(*key.as_bytes()),
        )
        .await
        .map(|_| ())
        .map_err(ChainWriteError::Write)
    }

    async fn write_window(&self) -> Result<WriteWindow, ChainWriteError> {
        self.registry.write_window().await.map_err(ChainWriteError::Window)
    }

    async fn update(
        &self,
        proxy: ChainAccount,
        root: RootHash,
        key: OrgPublicKey,
        expected_epoch: Epoch,
        window: WriteWindow,
    ) -> Result<(), ChainWriteError> {
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
        .map_err(ChainWriteError::Write)
    }
}

/// The writer when the chain is not configured: every write refused.
pub struct WriterNotConfigured;

#[async_trait]
impl ChainWriter for WriterNotConfigured {
    async fn create_proxy(&self) -> Result<(OrgId, ChainAccount), ChainWriteError> {
        Err(ChainWriteError::NotConfigured)
    }

    async fn found_at(&self, _: ChainAccount, _: RootHash, _: OrgPublicKey) -> Result<(), ChainWriteError> {
        Err(ChainWriteError::NotConfigured)
    }

    async fn write_window(&self) -> Result<WriteWindow, ChainWriteError> {
        Err(ChainWriteError::NotConfigured)
    }

    async fn update(&self, _: ChainAccount, _: RootHash, _: OrgPublicKey, _: Epoch, _: WriteWindow) -> Result<(), ChainWriteError> {
        Err(ChainWriteError::NotConfigured)
    }
}

/// What a write or a verification is for (LLR-s7rq5x): a held Organisation's next epoch, or a
/// provisional founding, by its founder's DevicePublicKey.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeldTarget {
    Organisation(OrgId),
    Founding(DevicePublicKey),
}

/// A write's outcome (LLR-s7rq5x): public data only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteReport {
    /// The write executed; `epoch` is the epoch it wrote (LLR-s7rq5x; the app's
    /// write-to-chain command reports it).
    Included { epoch: Epoch },
    /// It did not; the provisional organisation stays held.
    Refused { failure: WriteFailure },
    /// The bound elapsed: it may yet execute (LLR-d3farr).
    Unknown { reason: String },
}

/// Why a write that made a chain call did not execute (LLR-a66ufr,
/// LLR-sq5pbq): the step that failed and, where the writer answered, its
/// error, kept as the `source()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteFailure {
    /// A founding's `create_proxy` failed; nothing is stored.
    CreateProxy(Cause<ChainWriteError>),
    /// An Unanswered proxy creation: a founding's `create_proxy` passed `WRITE_TIMEOUT`; nothing
    /// is stored, and the next write creates the proxy again (LLR-sq5pbq).
    CreateProxyUnanswered,
    /// A founding's `found_at` at `proxy` failed.
    Founding { proxy: ChainAccount, cause: Cause<ChainWriteError> },
    /// An update's write-window read failed; no write was made.
    WindowRead(Cause<ChainWriteError>),
    /// An update's write-window read passed `WRITE_TIMEOUT`; no write was made.
    WindowReadTimedOut,
    /// An update's write did not execute.
    Update(Cause<ChainWriteError>),
}

impl std::fmt::Display for WriteFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateProxy(error) => write!(formatter, "creating the pure proxy failed; nothing stored: {error}"),
            Self::CreateProxyUnanswered => write!(
                formatter,
                "the pure proxy creation went unanswered for {} s; nothing stored, and the next write creates the proxy again",
                WRITE_TIMEOUT.as_secs()
            ),
            Self::Founding { proxy, cause } => {
                write!(formatter, "the founding at proxy {} did not complete: {cause}", hex::encode(proxy.as_bytes()))
            }
            Self::WindowRead(error) => write!(formatter, "reading the chain's write window failed, so no write was made: {error}"),
            Self::WindowReadTimedOut => formatter.write_str("reading the chain's write window timed out, so no write was made"),
            Self::Update(error) => write!(formatter, "the chain write did not execute: {error}"),
        }
    }
}

/// The writer's error, typed; a timeout has none.
impl std::error::Error for WriteFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreateProxy(error) | Self::Founding { cause: error, .. } | Self::WindowRead(error) | Self::Update(error) => {
                Some(error.get())
            }
            Self::CreateProxyUnanswered | Self::WindowReadTimedOut => None,
        }
    }
}

/// Why nothing was written (LLR-48me7m, LLR-mhn4n5): public data only.
#[derive(Debug)]
pub enum WriteRefused {
    /// No own provisional organisation or founding is held for the target.
    NotHeld,
    /// The Organisation's record holds no proxy account to write with.
    NoProxyAccount,
    /// The provisional organisation is no longer the proposal of the current record and its
    /// provisional changes.
    Stale,
    /// An earlier write's outcome is still unknown: the Organisation is verified first.
    OutcomeUnknown,
    /// org-node refused a query or a record the write needs.
    Node(OrgNodeError),
    /// A founding's proxy exists but its store write failed (LLR-5tpdxw).
    Store { proxy: ChainAccount, failure: StoreFailure },
}

impl std::fmt::Display for WriteRefused {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotHeld => formatter.write_str("no own provisional organisation or founding is held for that target"),
            Self::NoProxyAccount => formatter.write_str("the Organisation's record holds no proxy account to write with"),
            Self::Stale => formatter.write_str(
                "the provisional organisation is no longer the proposal of the current record and provisional changes; propose it again",
            ),
            Self::OutcomeUnknown => {
                formatter.write_str("an earlier write's outcome is still unknown; verify the Organisation before writing again")
            }
            Self::Node(error) => write!(formatter, "org-node refused the write: {error}"),
            Self::Store { proxy, failure } => write!(
                formatter,
                "the proxy account {} was created, but recording it failed: {failure}",
                hex::encode(proxy.as_bytes())
            ),
        }
    }
}

/// The cause, typed, of a refusal that has one: org-node's refusal or the
/// store failure.
impl std::error::Error for WriteRefused {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Node(error) => Some(error),
            Self::Store { failure, .. } => Some(failure),
            Self::NotHeld | Self::NoProxyAccount | Self::Stale | Self::OutcomeUnknown => None,
        }
    }
}

/// The write whose outcome is unknown (LLR-7y46be): what it wrote, the epoch it expected, and
/// for an update the window's last valid block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UnknownWrite {
    pub(crate) root: RootHash,
    pub(crate) key: OrgPublicKey,
    pub(crate) expected_epoch: Epoch,
    pub(crate) last_valid_block: Option<u64>,
}

/// What org-io keeps between calls, in the handle's memory (SDD-mk39e5, SDD-t4z8sz): one unknown
/// write per target, cleared only by verification (LLR-zey4mg), and the last verification
/// outcome per target (LLR-ce42ce).
#[derive(Default)]
pub(crate) struct PublishState {
    pub(crate) unknown_writes: HashMap<HeldTarget, UnknownWrite>,
    pub(crate) verifications: HashMap<HeldTarget, crate::verify::Verified>,
}

impl PublishState {
    /// The unknown write recorded for `target`, copied (LLR-7y46be).
    pub(crate) fn unknown_write(&self, target: &HeldTarget) -> Option<UnknownWrite> {
        self.unknown_writes.get(target).copied()
    }

    /// Clear `target`'s unknown write: only verification's settlement calls this (LLR-zey4mg).
    pub(crate) fn clear_unknown_write(&mut self, target: &HeldTarget) {
        self.unknown_writes.remove(target);
    }
}

/// Why a bounded call did not succeed.
enum WriteNotDone {
    Failed(Cause<ChainWriteError>),
    TimedOut,
}

/// One writer call, bounded by `WRITE_TIMEOUT` (LLR-d3farr).
async fn bounded<Written>(write: impl Future<Output = Result<Written, ChainWriteError>>) -> Result<Written, WriteNotDone> {
    match tokio::time::timeout(WRITE_TIMEOUT, write).await {
        Ok(written) => written.map_err(|error| WriteNotDone::Failed(Cause::new(error))),
        Err(_) => Err(WriteNotDone::TimedOut),
    }
}

fn timed_out() -> WriteReport {
    WriteReport::Unknown {
        reason: format!("the write timed out after {} s and may yet execute on chain", WRITE_TIMEOUT.as_secs()),
    }
}

impl OrgIo {
    /// Write `target`'s latest own provisional organisation to the chain (LLR-s7rq5x): no state
    /// read, no adoption, no store change but a founding's proxy (LLR-5tpdxw).
    pub async fn write_organisation_to_chain(&self, target: HeldTarget) -> Result<WriteReport, WriteRefused> {
        match target {
            HeldTarget::Organisation(org_id) => self.write_update(org_id).await,
            HeldTarget::Founding(founder) => self.write_founding(founder).await,
        }
    }

    /// A founding in two steps (LLR-5tpdxw, LLR-sq5pbq): the proxy created unless one is stored,
    /// recorded by org-node and written before `found_at`; a retry founds at the stored proxy.
    /// A `create_proxy` that fails or times out stores and records nothing (LLR-sq5pbq).
    /// An unknown founding write expects epoch zero, the genesis's, and has no window.
    async fn write_founding(&self, founder: DevicePublicKey) -> Result<WriteReport, WriteRefused> {
        let target = HeldTarget::Founding(founder);
        let mut publish = self.publish.lock().await;
        if publish.unknown_writes.contains_key(&target) {
            return Err(WriteRefused::OutcomeUnknown);
        }
        let (stored, root, key, written_epoch) = {
            let service = self.service.lock().await;
            // Read through org-node's key-free Persona view (`persona_view`, which clones no private key).
            // It refuses only a key no store holds and a removed store: neither holds a founding,
            // which is what `NotHeld` says.
            let founding = service.persona_view(&founder).ok().and_then(|view| view.founding).ok_or(WriteRefused::NotHeld)?;
            (founding.proxy, founding.root, founding.org_pub_key, founding.epoch)
        };
        let unknown = UnknownWrite { root, key, expected_epoch: Epoch::new(0), last_valid_block: None };
        let proxy = match stored {
            Some(proxy) => proxy,
            None => {
                let (org_id, proxy) = match bounded(self.writer.create_proxy()).await {
                    Ok(created) => created,
                    Err(WriteNotDone::Failed(error)) => {
                        return Ok(WriteReport::Refused { failure: WriteFailure::CreateProxy(error) })
                    }
                    // No proxy is known, so nothing is recorded and the next write creates one
                    // again (LLR-sq5pbq); a proxy the unanswered call made holds only its deposit.
                    Err(WriteNotDone::TimedOut) => {
                        return Ok(WriteReport::Refused { failure: WriteFailure::CreateProxyUnanswered })
                    }
                };
                let mut service = self.service.lock().await;
                let recorded = {
                    let mut rng = self.local.rng.lock();
                    service.record_founding_proxy(&mut *rng, &founder, org_id, proxy)
                };
                let change = recorded.map_err(WriteRefused::Node)?;
                self.local.store(&mut service, change).map_err(|failure| WriteRefused::Store { proxy, failure })?;
                proxy
            }
        };
        match bounded(self.writer.found_at(proxy, root, key)).await {
            Ok(()) => Ok(WriteReport::Included { epoch: written_epoch }),
            Err(WriteNotDone::Failed(cause)) => Ok(WriteReport::Refused { failure: WriteFailure::Founding { proxy, cause } }),
            Err(WriteNotDone::TimedOut) => {
                publish.unknown_writes.insert(target, unknown);
                Ok(timed_out())
            }
        }
    }

    /// The next epoch of `org_id`: the refusals first (LLR-48me7m, LLR-mhn4n5), then one window
    /// read and one bounded update (LLR-a66ufr, LLR-d3farr), an elapsed bound recorded (LLR-7y46be).
    /// The service lock is released before any chain call; the publish lock, held across the
    /// write, keeps a second write from racing the record of an unknown one.
    async fn write_update(&self, org_id: OrgId) -> Result<WriteReport, WriteRefused> {
        let target = HeldTarget::Organisation(org_id);
        let (proxy, root, key, expected_epoch, written_epoch) = {
            let service = self.service.lock().await;
            let device_key = crate::record_holder_of(&service, org_id).ok_or(WriteRefused::NotHeld)?;
            let record_epoch = crate::held_record(&service, org_id).map(|record| record.epoch).ok_or(WriteRefused::NotHeld)?;
            let own = service
                .provisional_organisations(&device_key)
                .map_err(WriteRefused::Node)?
                .into_iter()
                .rev()
                .find(|view| matches!(view.origin, Origin::Own))
                .ok_or(WriteRefused::NotHeld)?;
            let proxy = service.proxy_account(org_id).map_err(WriteRefused::Node)?.ok_or(WriteRefused::NoProxyAccount)?;
            if !own.current_proposal {
                return Err(WriteRefused::Stale);
            }
            (proxy, own.root, own.org_pub_key, record_epoch, own.epoch)
        };
        let mut publish = self.publish.lock().await;
        if publish.unknown_writes.contains_key(&target) {
            return Err(WriteRefused::OutcomeUnknown);
        }
        let window = match bounded(self.writer.write_window()).await {
            Ok(window) => window,
            Err(WriteNotDone::Failed(error)) => return Ok(WriteReport::Refused { failure: WriteFailure::WindowRead(error) }),
            Err(WriteNotDone::TimedOut) => return Ok(WriteReport::Refused { failure: WriteFailure::WindowReadTimedOut }),
        };
        match bounded(self.writer.update(proxy, root, key, expected_epoch, window)).await {
            Ok(()) => Ok(WriteReport::Included { epoch: written_epoch }),
            Err(WriteNotDone::Failed(error)) => Ok(WriteReport::Refused { failure: WriteFailure::Update(error) }),
            Err(WriteNotDone::TimedOut) => {
                let unknown = UnknownWrite { root, key, expected_epoch, last_valid_block: Some(window.last_valid_block()) };
                publish.unknown_writes.insert(target, unknown);
                Ok(timed_out())
            }
        }
    }
}
