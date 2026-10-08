//! Test-only substitutes (the `test-support` feature; never in production
//! builds). `FakeChain` stands in for the chain on both sides of org-io's
//! seams: its writes (`ChainWriter`) move a set of `ChainSlots`, and its
//! reads (`StateReader`) answer from them, counted; its signatory-set reads
//! (`SignatorySetReader`) answer from a mapped account per Organisation and
//! a delegate list per account, counted apart. It generalises the app's
//! former `FakeWriter`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use on_chain_client::write::AccountId;
use org_node::test_fixtures::ChainSlots;
use org_node::{ChainAccount, Epoch, OrgId, OrgNodeError, OrgPublicKey, OrgState, RootHash};

use crate::chain_read::StateReader;
use crate::signatory::SignatorySetReader;
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
/// - `original_account_failing`, `delegates_failing`: the one signatory-set
///   read or the other fails; those reads are counted together
///   (`signatory_reads`).
#[derive(Clone, Default)]
pub struct FakeChain {
    slots: ChainSlots,
    proxies: Arc<Mutex<HashMap<[u8; 32], OrgId>>>,
    signatory_set: Arc<Mutex<SignatorySet>>,
    pub write_failing: Arc<AtomicBool>,
    pub write_hanging: Arc<AtomicBool>,
    pub read_failing: Arc<AtomicBool>,
    pub parse_refusing: Arc<AtomicBool>,
    pub state_hidden: Arc<AtomicBool>,
    pub epoch_ahead: Arc<AtomicBool>,
    pub original_account_failing: Arc<AtomicBool>,
    pub delegates_failing: Arc<AtomicBool>,
    reads: Arc<AtomicUsize>,
    signatory_reads: Arc<AtomicUsize>,
}

/// What the signatory-set reads answer from: `Revive.OriginalAccount` per
/// Organisation and `Proxy.Proxies` per account.
#[derive(Default)]
struct SignatorySet {
    original_accounts: HashMap<OrgId, AccountId>,
    delegates: HashMap<AccountId, Vec<AccountId>>,
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
            ..Self::default()
        }
    }

    /// Map `org_id`'s H160 to `account` (`Revive.OriginalAccount`).
    pub fn map_original_account(&self, org_id: OrgId, account: AccountId) {
        self.signatory_set().original_accounts.insert(org_id, account);
    }

    /// Set `account`'s proxy delegates (`Proxy.Proxies`).
    pub fn set_delegates(&self, account: AccountId, delegates: Vec<AccountId>) {
        self.signatory_set().delegates.insert(account, delegates);
    }

    /// How many signatory-set reads this view has answered, both kinds.
    pub fn signatory_reads(&self) -> usize {
        self.signatory_reads.load(Ordering::SeqCst)
    }

    fn signatory_set(&self) -> MutexGuard<'_, SignatorySet> {
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
}

#[async_trait]
impl SignatorySetReader for FakeChain {
    async fn original_account(&self, org_id: OrgId) -> Result<Option<AccountId>, String> {
        self.signatory_reads.fetch_add(1, Ordering::SeqCst);
        if self.original_account_failing.load(Ordering::SeqCst) {
            return Err("node unreachable".into());
        }
        Ok(self.signatory_set().original_accounts.get(&org_id).copied())
    }

    async fn proxy_delegates(&self, account: AccountId) -> Result<Vec<AccountId>, String> {
        self.signatory_reads.fetch_add(1, Ordering::SeqCst);
        if self.delegates_failing.load(Ordering::SeqCst) {
            return Err("node unreachable".into());
        }
        Ok(self.signatory_set().delegates.get(&account).cloned().unwrap_or_default())
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

    async fn update(&self, proxy: ChainAccount, root: RootHash, key: OrgPublicKey, expected_epoch: Epoch) -> Result<(), String> {
        self.gate().await?;
        let org_id = *self.proxies().get(proxy.as_bytes()).ok_or("unknown proxy")?;
        self.slots.apply_update(org_id, root, key, expected_epoch).map_err(|error| error.to_string())
    }
}
