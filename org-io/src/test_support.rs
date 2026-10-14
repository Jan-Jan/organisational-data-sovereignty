//! Test-only substitutes (the `test-support` feature; never in production
//! builds). `FakeChain` stands in for the chain on both sides of org-io's
//! seams: its writes (`ChainWriter`) move a set of `ChainSlots`, and its
//! reads (`StateReader`) answer from them, counted; its signatory-set read
//! (`SignatorySetReader`) answers from a mapped account per Organisation, a
//! definition list per account and an `Any` index, counted apart. It
//! generalises the app's former `FakeWriter`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use on_chain_client::write::{AccountId, WriteWindow};
use org_node::test_fixtures::ChainSlots;
use org_node::{ChainAccount, Epoch, OrgId, OrgNodeError, OrgPublicKey, OrgState, RootHash};

use crate::chain_read::StateReader;
use crate::signatory::{ProxyDefinition, SignatorySetReader, SignatorySetView};
use crate::submit::ChainWriter;

/// A substitute chain. Clones share everything. The switches:
/// - `write_failing`: every write fails; `write_hanging`: every write never
///   finishes.
/// - `read_failing`: every read fails as an unreachable node does;
///   `parse_refusing`: every read fails as a state org-node's parse refuses
///   does; `state_hidden`: every read finds no state; `epoch_ahead`: every
///   read finds the slot one epoch past where the writes left it (a state
///   org-node's commit refuses for a reason other than `OrgNotOnChain`).
///   Every read is counted (`reads`), whatever it answers.
/// - `write_lands_then_hangs`: an update write moves the slot, then never
///   returns (a write that executes after its bound); `write_window_failing`:
///   the write window cannot be read. Every update write is counted
///   (`update_writes`), whatever it answers.
/// - The finalised block (`advance_finalised`, shared with
///   `sharing_slots_with`): the write window is born at it and
///   `finalised_block` answers it, counted apart (`finalised_reads`);
///   `read_failing` fails that read too.
/// - `original_account_failing`, `delegates_failing`: the signatory-set read
///   fails at the mapped account, or (for a mapped H160) at its definitions;
///   the storage values it reads are counted (`signatory_reads`).
#[derive(Clone, Default)]
pub struct FakeChain {
    slots: ChainSlots,
    proxies: Arc<Mutex<HashMap<[u8; 32], OrgId>>>,
    signatory_set: Arc<Mutex<SignatorySlots>>,
    pub write_failing: Arc<AtomicBool>,
    pub write_hanging: Arc<AtomicBool>,
    pub read_failing: Arc<AtomicBool>,
    pub parse_refusing: Arc<AtomicBool>,
    pub state_hidden: Arc<AtomicBool>,
    pub epoch_ahead: Arc<AtomicBool>,
    pub original_account_failing: Arc<AtomicBool>,
    pub delegates_failing: Arc<AtomicBool>,
    pub write_lands_then_hangs: Arc<AtomicBool>,
    pub write_window_failing: Arc<AtomicBool>,
    reads: Arc<AtomicUsize>,
    signatory_reads: Arc<AtomicUsize>,
    finalised: Arc<AtomicU64>,
    finalised_reads: Arc<AtomicUsize>,
    update_writes: Arc<AtomicUsize>,
}

/// What the signatory-set read answers from: `Revive.OriginalAccount` per
/// Organisation, `Proxy.Proxies` per account, and the runtime's `Any` index
/// (0 unless set).
#[derive(Default)]
struct SignatorySlots {
    original_accounts: HashMap<OrgId, AccountId>,
    definitions: HashMap<AccountId, Vec<ProxyDefinition>>,
    any_proxy_type: u8,
}

impl FakeChain {
    pub fn new() -> Self {
        Self::default()
    }

    /// The same slots, proxies and signatory set as `other` (one chain),
    /// with its own read counters and its own switches: another node's view
    /// of that chain.
    pub fn sharing_slots_with(other: &FakeChain) -> Self {
        Self {
            slots: other.slots.clone(),
            proxies: other.proxies.clone(),
            signatory_set: other.signatory_set.clone(),
            finalised: other.finalised.clone(),
            ..Self::default()
        }
    }

    /// Map `org_id`'s H160 to `account` (`Revive.OriginalAccount`).
    pub fn map_original_account(&self, org_id: OrgId, account: AccountId) {
        self.signatory_set().original_accounts.insert(org_id, account);
    }

    /// Set `account`'s proxy definitions to each delegate's founding
    /// definition, `(delegate, Any, 0)`.
    pub fn set_delegates(&self, account: AccountId, delegates: Vec<AccountId>) {
        let any = self.signatory_set().any_proxy_type;
        let definitions =
            delegates.into_iter().map(|delegate| ProxyDefinition { delegate, proxy_type: any, delay: 0 }).collect();
        self.set_definitions(account, definitions);
    }

    /// Set `account`'s proxy definitions (`Proxy.Proxies`), in stored order.
    pub fn set_definitions(&self, account: AccountId, definitions: Vec<ProxyDefinition>) {
        self.signatory_set().definitions.insert(account, definitions);
    }

    /// Set the runtime's `Any` proxy-type index the read reports.
    pub fn set_any_proxy_type(&self, index: u8) {
        self.signatory_set().any_proxy_type = index;
    }

    /// How many signatory-set storage values this view has read:
    /// `Revive.OriginalAccount` on every read, `Proxy.Proxies` only for a
    /// mapped H160, as on-chain-client's pinned read fetches them.
    pub fn signatory_reads(&self) -> usize {
        self.signatory_reads.load(Ordering::SeqCst)
    }

    fn signatory_set(&self) -> MutexGuard<'_, SignatorySlots> {
        self.signatory_set.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The slots the writes move and the reads answer from.
    pub fn slots(&self) -> &ChainSlots {
        &self.slots
    }

    /// How many reads this view has answered.
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }

    /// Move the chain's finalised block on by `blocks`.
    pub fn advance_finalised(&self, blocks: u64) {
        self.finalised.fetch_add(blocks, Ordering::SeqCst);
    }

    /// How many finalised-block reads this view has answered.
    pub fn finalised_reads(&self) -> usize {
        self.finalised_reads.load(Ordering::SeqCst)
    }

    /// How many update writes this view has been asked for.
    pub fn update_writes(&self) -> usize {
        self.update_writes.load(Ordering::SeqCst)
    }

    /// How many Organisations have had a genesis written on this chain.
    pub fn geneses(&self) -> usize {
        self.proxies().len()
    }

    fn proxies(&self) -> MutexGuard<'_, HashMap<[u8; 32], OrgId>> {
        self.proxies.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    async fn gate(&self) -> Result<(), String> {
        if self.write_hanging.load(Ordering::SeqCst) {
            std::future::pending::<()>().await;
        }
        if self.write_failing.load(Ordering::SeqCst) {
            return Err("node unreachable".into());
        }
        Ok(())
    }
}

#[async_trait]
impl StateReader for FakeChain {
    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        if self.read_failing.load(Ordering::SeqCst) {
            return Err(OrgNodeError::Chain("node unreachable".into()));
        }
        if self.parse_refusing.load(Ordering::SeqCst) {
            // What `read_org_state` returns when org-node's parse refuses
            // the state's Organisation public key.
            return Err(OrgNodeError::InvalidOrgPublicKey);
        }
        if self.state_hidden.load(Ordering::SeqCst) {
            return Ok(None);
        }
        let state = self.slots.get(&org_id);
        if self.epoch_ahead.load(Ordering::SeqCst) {
            return Ok(state.map(|slot| OrgState { epoch: Epoch::new(slot.epoch.get() + 1), ..slot }));
        }
        Ok(state)
    }

    async fn finalised_block(&self) -> Result<u64, OrgNodeError> {
        self.finalised_reads.fetch_add(1, Ordering::SeqCst);
        if self.read_failing.load(Ordering::SeqCst) {
            return Err(OrgNodeError::Chain("node unreachable".into()));
        }
        Ok(self.finalised.load(Ordering::SeqCst))
    }
}

#[async_trait]
impl SignatorySetReader for FakeChain {
    async fn signatory_set(&self, org_id: OrgId) -> Result<SignatorySetView, String> {
        self.signatory_reads.fetch_add(1, Ordering::SeqCst);
        if self.original_account_failing.load(Ordering::SeqCst) {
            return Err("node unreachable".into());
        }
        let slots = self.signatory_set();
        let original_account = slots.original_accounts.get(&org_id).copied();
        let definitions = match original_account {
            None => Vec::new(),
            Some(account) => {
                self.signatory_reads.fetch_add(1, Ordering::SeqCst);
                if self.delegates_failing.load(Ordering::SeqCst) {
                    return Err("node unreachable".into());
                }
                slots.definitions.get(&account).cloned().unwrap_or_default()
            }
        };
        Ok(SignatorySetView { original_account, definitions, any_proxy_type: slots.any_proxy_type })
    }
}

#[async_trait]
impl ChainWriter for FakeChain {
    async fn genesis(&self, root: RootHash, key: OrgPublicKey) -> Result<(OrgId, ChainAccount), String> {
        self.gate().await?;
        let org_id = self.slots.apply_genesis(root, key);
        let mut proxy = [0u8; 32];
        proxy[..20].copy_from_slice(org_id.as_bytes());
        self.proxies().insert(proxy, org_id);
        Ok((org_id, ChainAccount::new(proxy)))
    }

    async fn write_window(&self) -> Result<WriteWindow, String> {
        if self.write_window_failing.load(Ordering::SeqCst) {
            return Err("node unreachable".into());
        }
        let birth_number = self.finalised.load(Ordering::SeqCst);
        let mut birth_hash = [0u8; 32];
        birth_hash[..8].copy_from_slice(&birth_number.to_le_bytes());
        Ok(WriteWindow { birth_number, birth_hash })
    }

    async fn update(
        &self,
        proxy: ChainAccount,
        root: RootHash,
        key: OrgPublicKey,
        expected_epoch: Epoch,
        _window: WriteWindow,
    ) -> Result<(), String> {
        self.update_writes.fetch_add(1, Ordering::SeqCst);
        self.gate().await?;
        let org_id = *self.proxies().get(proxy.as_bytes()).ok_or("unknown proxy")?;
        self.slots.apply_update(org_id, root, key, expected_epoch).map_err(|error| error.to_string())?;
        if self.write_lands_then_hangs.load(Ordering::SeqCst) {
            std::future::pending::<()>().await;
        }
        Ok(())
    }
}
