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
/// - `hold_next_read`: the next state read waits until the test releases it.
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
    read_hold: Arc<Mutex<Option<ReadHold>>>,
}

/// A state read held open: `hold_next_read` arms it; the next
/// state read signals `arrived` and then waits for `release` before it
/// answers, so a test can act while a caller is between its check and its
/// commit.
#[derive(Clone, Default)]
pub struct ReadHold {
    arrived: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Notify>,
}

impl ReadHold {
    /// Wait until the held read has started.
    pub async fn arrived(&self) {
        self.arrived.notified().await;
    }

    /// Let the held read answer.
    pub fn release(&self) {
        self.release.notify_one();
    }
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

    /// Hold this view's next state read open until the returned hold is
    /// released.
    pub fn hold_next_read(&self) -> ReadHold {
        let hold = ReadHold::default();
        *self.read_hold.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(hold.clone());
        hold
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
        let hold = self.read_hold.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).take();
        if let Some(hold) = hold {
            hold.arrived.notify_one();
            hold.release.notified().await;
        }
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

/// The most a dialler reads back after its body: the verdict is one byte,
/// and a few more show a receiver that wrote anything else.
const READ_BACK_BOUND: usize = 64;

/// A relay-free test dialler holding a Device's identity:
/// bind a Loopback endpoint (relay disabled) from `device_key`'s keychain
/// item, dial `addr`'s direct addresses only, send `body` framed as a send
/// does, finish, and return every byte the receiver wrote back, up to its
/// end of stream (at most `READ_BACK_BOUND`). The sending side of a revoked
/// Device's acknowledgement is S3b-io's; this stands in for it.
pub async fn dial_direct_as(
    device_key: &org_node::DevicePublicKey,
    addr: iroh::EndpointAddr,
    body: &[u8],
) -> Result<Vec<u8>, crate::transport::TransportError> {
    use crate::transport::{BindPlan, OrgEndpoint, TransportError, ALPN};
    let stream_error = |error: &dyn std::fmt::Display| TransportError::Stream(error.to_string());
    let endpoint = OrgEndpoint::bind_from_keychain(device_key, BindPlan::Loopback).await?;
    let direct = iroh::EndpointAddr::from_parts(addr.id, addr.addrs.into_iter().filter(|address| matches!(address, iroh::TransportAddr::Ip(_))));
    let conn = endpoint.inner().connect(direct, ALPN).await.map_err(|error| TransportError::Connect(error.to_string()))?;
    let (mut send, mut recv) = conn.open_bi().await.map_err(|error| stream_error(&error))?;
    send.write_all(&crate::transport::frame::frame_body(body)?).await.map_err(|error| stream_error(&error))?;
    send.finish().map_err(|error| stream_error(&error))?;
    let read_back = recv.read_to_end(READ_BACK_BOUND).await.map_err(|error| stream_error(&error))?;
    conn.close(0u32.into(), b"done");
    Ok(read_back)
}

/// The keychain module's functions, for the keychain's own tests
/// (`tests/keychain.rs`).
pub use crate::keychain::for_test as keychain;

/// Make this thread's next store file write fail after its temporary file is
/// written, before it replaces the store file (LLR-b9bm36's interruption).
pub fn fail_next_write_after_temporary() {
    crate::store_file::test_hook::fail_next_write_after_temporary();
}

/// `store_file::write_store`, crate-visible in every build, for the store
/// files' own tests (`tests/store_files.rs`; test builds only).
pub fn write_store(
    data_dir: &std::path::Path,
    write: &org_node::StoreWrite,
) -> Result<(), crate::store_file::StoreFileError> {
    crate::store_file::write_store(data_dir, write)
}

/// The sealed bytes of every store file write that landed for
/// `device_key` in `data_dir`, in the order they landed, in this process
/// (the order of an admission's store writes).
pub fn store_writes_to(data_dir: &std::path::Path, device_key: &org_node::DevicePublicKey) -> Vec<Vec<u8>> {
    crate::store_file::test_hook::writes_to(&crate::store_file::store_path(data_dir, device_key))
}

/// Whether a `MemoryKeychain` was installed in this process: until one is,
/// the keychain module refuses every call in a test build, so no test reaches
/// the OS keychain.
static IN_MEMORY_KEYCHAIN_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Whether a `MemoryKeychain` was installed in this process.
pub(crate) fn in_memory_keychain_installed() -> bool {
    IN_MEMORY_KEYCHAIN_INSTALLED.load(Ordering::SeqCst)
}

/// How a `MemoryKeychain` told to refuse refuses every call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// As a locked keychain does: `keyring::Error::NoStorageAccess`.
    Locked,
    /// As any other keychain failure: `keyring::Error::PlatformFailure`.
    Unavailable,
}

/// The items and the refusal switch every credential of one
/// `MemoryKeychain` shares.
#[derive(Default)]
struct MemoryItems {
    items: Mutex<HashMap<(String, String), Vec<u8>>>,
    refuse_as: Mutex<Option<Refusal>>,
    /// Every read, write and delete asked of an item, refused ones included.
    calls: AtomicUsize,
}

impl MemoryItems {
    fn items(&self) -> MutexGuard<'_, HashMap<(String, String), Vec<u8>>> {
        self.items.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Count the call, then refuse it as the switch says.
    fn refusal(&self) -> keyring::Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let refuse_as = *self.refuse_as.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        match refuse_as {
            None => Ok(()),
            Some(Refusal::Locked) => Err(keyring::Error::NoStorageAccess("refused by the test".into())),
            Some(Refusal::Unavailable) => Err(keyring::Error::PlatformFailure("refused by the test".into())),
        }
    }
}

/// An in-memory credential store for keyring (LLR-36jer8): every credential
/// it builds is a view onto one shared map, by (service, user), so an item
/// written through one `keyring::Entry` reads back through another, as in
/// the OS keychain (keyring's own in-memory store keeps each item in its
/// entry). Clones share the map and the switch.
#[derive(Clone, Default)]
pub struct MemoryKeychain {
    shared: Arc<MemoryItems>,
}

impl MemoryKeychain {
    /// One keychain for the whole test process, for test binaries whose
    /// tests run in parallel and only need the keychain to hold what the
    /// handle writes: each test's items are its own Device keys'.
    pub fn shared() -> &'static MemoryKeychain {
        static SHARED: std::sync::OnceLock<MemoryKeychain> = std::sync::OnceLock::new();
        SHARED.get_or_init(MemoryKeychain::default)
    }

    /// Install this store as keyring's default credential builder, for the
    /// whole process.
    pub fn install(&self) {
        keyring::set_default_credential_builder(Box::new(self.clone()));
        IN_MEMORY_KEYCHAIN_INSTALLED.store(true, Ordering::SeqCst);
    }

    /// Refuse every call as `refusal` says, or, with `None`, none.
    pub fn refuse_as(&self, refusal: Option<Refusal>) {
        *self.shared.refuse_as.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = refusal;
    }

    /// Put `bytes` in the item of (`service`, `user`), bypassing keyring: an
    /// item another program or an older version left.
    pub fn insert(&self, service: &str, user: &str, bytes: Vec<u8>) {
        self.shared.items().insert((service.to_string(), user.to_string()), bytes);
    }

    /// How many item reads, writes and deletes were asked of this keychain,
    /// refused ones included.
    pub fn calls(&self) -> usize {
        self.shared.calls.load(Ordering::SeqCst)
    }

    /// The (service, user) names of the items held, sorted.
    pub fn items(&self) -> Vec<(String, String)> {
        let mut names: Vec<_> = self.shared.items().keys().cloned().collect();
        names.sort();
        names
    }
}

impl keyring::credential::CredentialBuilderApi for MemoryKeychain {
    fn build(&self, _target: Option<&str>, service: &str, user: &str) -> keyring::Result<Box<keyring::credential::Credential>> {
        Ok(Box::new(MemoryCredential {
            shared: self.shared.clone(),
            service: service.to_string(),
            user: user.to_string(),
        }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Debug for MemoryKeychain {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("MemoryKeychain")
    }
}

/// One item's view onto a `MemoryKeychain`'s map.
struct MemoryCredential {
    shared: Arc<MemoryItems>,
    service: String,
    user: String,
}

impl MemoryCredential {
    fn name(&self) -> (String, String) {
        (self.service.clone(), self.user.clone())
    }
}

impl keyring::credential::CredentialApi for MemoryCredential {
    fn set_secret(&self, secret: &[u8]) -> keyring::Result<()> {
        self.shared.refusal()?;
        self.shared.items().insert(self.name(), secret.to_vec());
        Ok(())
    }

    fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        self.shared.refusal()?;
        self.shared.items().get(&self.name()).cloned().ok_or(keyring::Error::NoEntry)
    }

    fn delete_credential(&self) -> keyring::Result<()> {
        self.shared.refusal()?;
        self.shared.items().remove(&self.name()).map(|_| ()).ok_or(keyring::Error::NoEntry)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// The item's name only, never its bytes (keyring logs each credential it
/// builds with `Debug`).
impl std::fmt::Debug for MemoryCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "MemoryCredential {{ service: {:?}, user: {:?} }}", self.service, self.user)
    }
}
