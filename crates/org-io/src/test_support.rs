//! Test-only substitutes (the `test-support` feature; never in production
//! builds). `FakeChain` stands in for the chain on both sides of org-io's
//! seams: its writes (`ChainWriter`) move a set of `ChainSlots`, and its
//! reads (`StateReader`) answer from them, counted.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use on_chain_client::write::{GenesisStep, WriteError, WriteWindow};
use on_chain_client::ClientError;
use org_node::test_fixtures::ChainSlots;
use org_node::{ChainAccount, Epoch, OrgId, OrgNodeError, OrgPublicKey, OrgState, RootHash};

use crate::chain_read::StateReader;
use crate::write::{ChainWriteError, ChainWriter};

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
///   the write window cannot be read; `write_window_hanging`: its read never
///   returns (`write_hanging` leaves the window read alone). Every update write is counted
///   (`update_writes`), whatever it answers.
/// - A founding's two steps: `create_proxy` makes proxy `[n; 20] ++ [0xEE; 12]`
///   for its n-th call (from `0xc0`) with Organisation `h160_of(proxy)`, which
///   is `[n; 20]`; `found_at` sets that
///   slot at epoch 1, refusing a slot already set as the contract refuses a
///   second genesis; `found_at_failing` fails `found_at` at its `Fund` step.
///   Every step is logged (`founding_calls`), whatever it answers.
/// - The finalised block (`advance_finalised`, shared with
///   `sharing_slots_with`): the write window is born at it and
///   `finalised_block` answers it, counted apart (`finalised_reads`);
///   `read_failing` fails that read too.
/// - `hold_next_read`: the next state read waits until the test releases it.
#[derive(Clone, Default)]
pub struct FakeChain {
    slots: ChainSlots,
    proxies: Arc<Mutex<HashMap<[u8; 32], OrgId>>>,
    pub write_failing: Arc<AtomicBool>,
    pub write_hanging: Arc<AtomicBool>,
    pub read_failing: Arc<AtomicBool>,
    pub parse_refusing: Arc<AtomicBool>,
    pub state_hidden: Arc<AtomicBool>,
    pub epoch_ahead: Arc<AtomicBool>,
    pub write_lands_then_hangs: Arc<AtomicBool>,
    pub write_window_failing: Arc<AtomicBool>,
    pub write_window_hanging: Arc<AtomicBool>,
    pub found_at_failing: Arc<AtomicBool>,
    founding_log: Arc<Mutex<FoundingLog>>,
    next_proxy: Arc<AtomicU8>,
    reads: Arc<AtomicUsize>,
    finalised: Arc<AtomicU64>,
    finalised_reads: Arc<AtomicUsize>,
    update_writes: Arc<AtomicUsize>,
    write_windows: Arc<AtomicUsize>,
    last_window: Arc<Mutex<Option<WriteWindow>>>,
    read_hold: Arc<Mutex<Option<ReadHold>>>,
}

/// The founding steps a `FakeChain` was asked for, in order: the step's name and the proxy
/// `found_at` was given.
pub type FoundingLog = Vec<(&'static str, Option<ChainAccount>)>;

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

impl FakeChain {
    pub fn new() -> Self {
        Self::default()
    }

    /// The same slots and proxies as `other` (one chain),
    /// with its own read counters and its own switches: another node's view
    /// of that chain.
    pub fn sharing_slots_with(other: &FakeChain) -> Self {
        Self {
            slots: other.slots.clone(),
            proxies: other.proxies.clone(),
            finalised: other.finalised.clone(),
            ..Self::default()
        }
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

    /// How many write windows this view has been asked for.
    pub fn write_windows(&self) -> usize {
        self.write_windows.load(Ordering::SeqCst)
    }

    /// The window the last update write was given.
    pub fn last_update_window(&self) -> Option<WriteWindow> {
        *self.last_window.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// How many Organisations have had a genesis written on this chain.
    pub fn geneses(&self) -> usize {
        self.proxies().len()
    }

    fn founding_log(&self) -> MutexGuard<'_, FoundingLog> {
        self.founding_log.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Every founding step this view was asked for, in order, with the proxy `found_at` was given.
    pub fn founding_calls(&self) -> FoundingLog {
        self.founding_log().clone()
    }

    fn proxies(&self) -> MutexGuard<'_, HashMap<[u8; 32], OrgId>> {
        self.proxies.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    async fn gate(&self) -> Result<(), ChainWriteError> {
        if self.write_hanging.load(Ordering::SeqCst) {
            std::future::pending::<()>().await;
        }
        if self.write_failing.load(Ordering::SeqCst) {
            return Err(unreachable_node());
        }
        Ok(())
    }

    /// The Organisation of `proxy`, as this chain's writer knows it.
    fn org_of(&self, proxy: &ChainAccount) -> Result<OrgId, ChainWriteError> {
        self.proxies().get(proxy.as_bytes()).copied().ok_or(ChainWriteError::Write(WriteError::Subxt("unknown proxy".into())))
    }
}

/// A write failing as on-chain-client's does when the node cannot be reached.
fn unreachable_node() -> ChainWriteError {
    ChainWriteError::Write(WriteError::Subxt("node unreachable".into()))
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
impl ChainWriter for FakeChain {
    async fn create_proxy(&self) -> Result<(OrgId, ChainAccount), ChainWriteError> {
        self.founding_log().push(("create_proxy", None));
        self.gate().await?;
        let seed = 0xc0u8.wrapping_add(self.next_proxy.fetch_add(1, Ordering::SeqCst));
        // A real-shaped proxy account: its Organisation is the H160 on-chain-client's `h160_of`
        // gives for it, as the production writer's is (`tests/common/chain_writer_contract.rs`).
        let mut account = [0xEEu8; 32];
        account[..20].copy_from_slice(&[seed; 20]);
        let (org_id, proxy) = (OrgId::new(on_chain_client::h160_of(account)), ChainAccount::new(account));
        self.proxies().insert(*proxy.as_bytes(), org_id);
        Ok((org_id, proxy))
    }

    async fn found_at(&self, proxy: ChainAccount, root: RootHash, key: OrgPublicKey) -> Result<(), ChainWriteError> {
        self.founding_log().push(("found_at", Some(proxy)));
        if self.found_at_failing.load(Ordering::SeqCst) {
            return Err(ChainWriteError::Write(WriteError::Step { step: GenesisStep::Fund, reason: "node unreachable".into() }));
        }
        self.gate().await?;
        let org_id = self.org_of(&proxy)?;
        if self.slots.get(&org_id).is_some() {
            let reason = "the slot already holds a genesis".into();
            return Err(ChainWriteError::Write(WriteError::Step { step: GenesisStep::RecordGenesis, reason }));
        }
        self.slots.set(org_id, OrgState { root_hash: root, org_pub_key: key, epoch: Epoch::new(1) });
        Ok(())
    }

    async fn write_window(&self) -> Result<WriteWindow, ChainWriteError> {
        self.write_windows.fetch_add(1, Ordering::SeqCst);
        if self.write_window_hanging.load(Ordering::SeqCst) {
            std::future::pending::<()>().await;
        }
        if self.write_window_failing.load(Ordering::SeqCst) {
            return Err(ChainWriteError::Window(ClientError::Subxt("node unreachable".into())));
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
        window: WriteWindow,
    ) -> Result<(), ChainWriteError> {
        self.update_writes.fetch_add(1, Ordering::SeqCst);
        *self.last_window.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(window);
        self.gate().await?;
        let org_id = self.org_of(&proxy)?;
        // The contract's refusal of a stale expected epoch, as the real chain returns it: the call
        // made through the proxy failed (`tests/common/chain_writer_contract.rs`).
        self.slots
            .apply_update(org_id, root, key, expected_epoch)
            .map_err(|error| ChainWriteError::Write(WriteError::InnerCallFailed(error.to_string())))?;
        if self.write_lands_then_hangs.load(Ordering::SeqCst) {
            std::future::pending::<()>().await;
        }
        Ok(())
    }
}

/// The production halves of the chain `FakeChain` stands in for, built as `OrgIo::connect`
/// builds them, with no co-signer: the real side of `FakeChain`'s contract tests
/// (`tests/common/chain_contract.rs`; test builds with `dev-seed` only, under chopsticks).
#[cfg(feature = "dev-seed")]
pub struct ProductionChain {
    pub reader: crate::chain_read::OnChainStateReader,
    pub writer: crate::write::OnChainWriter,
}

/// `ProductionChain` over `settings`, signing with the seed `ODS_ADMIN_SEED` holds.
#[cfg(feature = "dev-seed")]
pub async fn production_chain(settings: &crate::ChainSettings) -> Result<ProductionChain, String> {
    use on_chain_client::write::subxt_ops::{FinalitySink, SubxtWriteOps};
    let signatory = crate::custody::signatory_from_environment()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "ODS_ADMIN_SEED is not set".to_string())?;
    let (api, registry) =
        crate::connect::connect(&settings.ws_url, settings.contract_h160).await.map_err(|error| error.to_string())?;
    let ops = SubxtWriteOps::new(api, FinalitySink);
    Ok(ProductionChain {
        reader: crate::chain_read::OnChainStateReader::new(registry.clone()),
        writer: crate::write::OnChainWriter::new(ops, registry, signatory, Vec::new(), settings.contract_h160),
    })
}

/// A settable clock (LLR-6vtbn6): every reading counted; `set_before_epoch` makes it read before
/// the Unix epoch.
#[derive(Default)]
pub struct TestClock {
    seconds: AtomicU64,
    before_epoch: AtomicBool,
    readings: AtomicUsize,
}

impl TestClock {
    /// A clock reading `seconds`.
    pub fn at(seconds: u64) -> Arc<Self> {
        Arc::new(Self { seconds: AtomicU64::new(seconds), ..Self::default() })
    }

    pub fn set(&self, seconds: u64) {
        self.seconds.store(seconds, Ordering::SeqCst);
    }

    pub fn set_before_epoch(&self, before: bool) {
        self.before_epoch.store(before, Ordering::SeqCst);
    }

    /// How many times the clock was read.
    pub fn readings(&self) -> usize {
        self.readings.load(Ordering::SeqCst)
    }
}

impl crate::clock::Clock for TestClock {
    fn now(&self) -> Option<u64> {
        self.readings.fetch_add(1, Ordering::SeqCst);
        (!self.before_epoch.load(Ordering::SeqCst)).then(|| self.seconds.load(Ordering::SeqCst))
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
/// files' own tests (`tests/store_files.rs`; test builds only), over an
/// operating-system source of its own.
pub fn write_store(
    data_dir: &std::path::Path,
    write: &org_node::StoreWrite,
) -> Result<(), crate::store_file::StoreFileError> {
    crate::store_file::write_store(data_dir, write, &crate::rng::HandleRng::os())
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
    /// Reads alone refused as this says, writes and deletes still answering: a store locked
    /// after a write (pinned in tests/keychain_contract.rs).
    refuse_reads_as: Mutex<Option<Refusal>>,
    /// Deletes alone refused as this says, reads and writes still answering: a keychain that
    /// unseals a store and then will not give up its item. A fault injector (TEST_GUIDELINES
    /// Doubles 5) only with `Refusal::Unavailable`, an uninterpreted OS failure; with
    /// `Refusal::Locked` it would imitate a store locked for deletes alone, which no contract
    /// test pins (LLR-36jer8), so its users pass `Unavailable`.
    refuse_deletes_as: Mutex<Option<Refusal>>,
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
        Self::refused(refuse_as)
    }

    /// A read's refusal: `refusal`'s, or else the read switch's.
    fn read_refusal(&self) -> keyring::Result<()> {
        self.refusal()?;
        Self::refused(*self.refuse_reads_as.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }

    /// A delete's refusal: `refusal`'s, or else the delete switch's.
    fn delete_refusal(&self) -> keyring::Result<()> {
        self.refusal()?;
        Self::refused(*self.refuse_deletes_as.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }

    fn refused(refuse_as: Option<Refusal>) -> keyring::Result<()> {
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

    /// Refuse every read as `refusal` says, writes and deletes still answering, or, with
    /// `None`, no read: a keychain that takes a new item and then will not give it back.
    pub fn refuse_reads_as(&self, refusal: Option<Refusal>) {
        *self.shared.refuse_reads_as.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = refusal;
    }

    /// Refuse every delete as `refusal` says, reads and writes still answering, or, with
    /// `None`, no delete. A fault injector with `Refusal::Unavailable` (an OS failure the code
    /// does not interpret); `Refusal::Locked` here has no contract test, so pass `Unavailable`.
    pub fn refuse_deletes_as(&self, refusal: Option<Refusal>) {
        *self.shared.refuse_deletes_as.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = refusal;
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
        self.shared.read_refusal()?;
        self.shared.items().get(&self.name()).cloned().ok_or(keyring::Error::NoEntry)
    }

    fn delete_credential(&self) -> keyring::Result<()> {
        self.shared.delete_refusal()?;
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

/// The handle's random source and store, as tests reach them (LLR-5acgfg): test builds only, and
/// the one file the absence scans of LLR-ctm86b except. The gate repeats the module's, so the
/// handle-surface scan of LLR-3zdw8v reads these methods as test-only.
#[cfg(feature = "test-support")]
impl crate::OrgIo {
    /// This handle, drawing from `source` in place of the operating system's (a seeded generator,
    /// to reproduce an invite identifier or a key pair; a failing one, LLR-94nn6q).
    pub fn with_rng_for_test(mut self, source: Box<dyn rand_core::CryptoRngCore + Send>) -> Self {
        self.local.rng = Arc::new(crate::rng::HandleRng::replaced_by(source));
        self
    }

    /// This handle, reading `clock` in place of the system's (LLR-6vtbn6).
    pub fn with_clock_for_test(mut self, clock: Arc<dyn crate::clock::Clock>) -> Self {
        self.local.clock = clock;
        self
    }

    /// `open`, with `clock` as the handle's from the start (LLR-6vtbn6).
    #[allow(clippy::result_large_err, reason = "as open's")]
    pub fn open_with_clock_for_test(
        data_dir: &std::path::Path,
        transport_mode: crate::transport::TransportMode,
        clock: Arc<dyn crate::clock::Clock>,
    ) -> Result<Self, crate::OpenError> {
        Self::open_with(data_dir, transport_mode, clock)
    }

    /// One reading of the handle's clock.
    pub fn clock_reading_for_test(&self) -> Option<u64> {
        self.local.clock.now()
    }

    /// This handle, giving every verification `limit` as its ahead limit in place of the one it
    /// holds (LLR-22se2q), through `connect`'s own conversion: whole seconds, a fraction dropped.
    pub fn with_ahead_limit_for_test(mut self, limit: std::time::Duration) -> Self {
        self.ahead_limit = Some(crate::ahead_limit_from(limit));
        self
    }

    /// The ahead limit this handle gives every verification, `None` on a handle with no chain
    /// (LLR-22se2q).
    pub fn ahead_limit_for_test(&self) -> Option<std::time::Duration> {
        self.ahead_limit.map(|limit| std::time::Duration::from_secs(limit.0))
    }
}

/// The ahead limit of a handle over a substitute chain (`OrgIo::for_test`, `open_for_test`):
/// 1200 s, twice the app's default verification interval. Test builds only; a connected handle's
/// is the app's (`ChainSettings::ahead_limit`, LLR-22se2q).
pub const TEST_AHEAD_LIMIT: std::time::Duration = std::time::Duration::from_secs(1_200);

