//! org-io: all IO for an Organisation (docs/adr/2026-10-06-org-io-unit.md).
//! org-node takes values and returns values; org-io reads and writes the
//! chain, holds the user's own signatory key, and is the app's one way in.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod chain_read;
pub mod connect;
pub mod custody;
mod endpoints;
mod exchange;
mod keychain;
/// The peer-address parse, owned by the transport module (SDD-wr8m5n).
pub use crate::transport::peer_addr;
pub mod preflight;
pub mod publish;
mod receiver;
pub mod send;
pub mod signatory;
pub mod store_file;
pub mod submit;
#[cfg(feature = "test-support")]
pub mod test_support;
pub mod transport;
pub mod view;

/// org-node's value types the app names: identifiers, names, public keys,
/// org-node's refusals, the parsed Persona details, the public invitation
/// Blobs and a received message's outcome. Nothing else: org-node's
/// service, its stores and their records, the Device secret and the Wire
/// messages are not reachable through org-io (LLR-3zdw8v; plan §17.2
/// rule 3).
pub mod node {
    pub use org_node::invitation::{Invite, InviteId, InviteReply};
    pub use org_node::store::PersonaDetails;
    pub use org_node::{
        DevicePublicKey, Epoch, Handle, MemberId, Name, OrgId, OrgMembersError, OrgNodeError, PersonPublicKey, PersonaId,
        ReceiveKind, ReceiveOutcome, RootHash, Surname,
    };
}

pub use crate::exchange::{AdmitReport, ExchangeError};
pub use crate::transport::peer_addr::{parse_peer_addr, PeerAddr, PeerAddrError};
pub use crate::publish::{Reconciliation, Settled, Verification};
pub use crate::receiver::{ReceiveError, Received, TerminalCause};
pub use crate::send::{SendError, SendReport};
pub use crate::view::{BuiltUpdate, OrgSummary, PersonaSummary, Submitted};
use org_node::service::OrgService;
use org_node::store::PersonaStore;
use org_node::{DevicePublicKey, OrgId, OrgNodeError, PersonPublicKey, PersonaId, StoreChange, StoreWrite};
use rand_core::{CryptoRng, RngCore};

use crate::endpoints::Endpoints;
pub use crate::keychain::KeychainError;
use crate::keychain::{Keychain, UnsealFailure};
use crate::store_file::StoreFileError;
#[cfg(feature = "test-support")]
use crate::transport::TransportError;
use crate::transport::TransportMode;

use on_chain_client::write::AccountId;

use crate::chain_read::StateReader;
pub use crate::connect::{ConnectFailed, ConnectFailure};
use crate::signatory::{AdminStatus, OwnAdminError, SignatorySetReader};
use crate::submit::ChainWriter;

/// The chain's settings as the app read them from its environment; org-io
/// reads the signing seed itself, in development builds only (REQ-8zuka3).
pub struct ChainSettings {
    pub ws_url: String,
    pub contract_h160: [u8; 20],
    /// `ODS_COSIGNER_PUB` as the app read it; parsed here (LLR-9fy622).
    pub co_signer: Option<String>,
}

/// The app's one way in (SDD-z3ychz): org-node's service, the chain read,
/// the chain write and the signatory-set reads. The signatory key is held by
/// the writer alone and never handed out (LLR-u2pk5y); the handle keeps only
/// its public account and the configured co-signers, for the own-admin check
/// (LLR-c9fyun). The service is behind the handle's own lock and every
/// method takes `&self`, so the receive step can wait with
/// the lock released (LLR-9arvqj); each method takes the lock inside and
/// returns owned public values.
pub struct OrgIo {
    service: tokio::sync::Mutex<OrgService>,
    local: LocalStore,
    reader: Box<dyn StateReader>,
    writer: Box<dyn ChainWriter>,
    signatory_reader: Box<dyn SignatorySetReader>,
    /// The account of the signatory key this node holds; `None` when it holds none.
    own_account: Option<AccountId>,
    co_signers: Vec<AccountId>,
    /// The writes whose outcome is unknown and each Organisation's
    /// verification status (SDD-6mpmsw), behind its own lock; a method that
    /// takes both takes `service`'s first, and none waits for `service`'s
    /// while holding this one.
    publish: tokio::sync::Mutex<publish::PublishState>,
}

/// Where org-io keeps what org-node hands out: each store change in its
/// Persona's store file in the data directory (`store_file`), and each new
/// Persona's device secret in the OS keychain (`keychain`). A handle built
/// without a data directory (`not_configured`, `for_test`) keeps its stores
/// in memory only. It also holds the node's endpoints, one per Persona, each
/// bound from its keychain item (`endpoints`). Public only because `submit`'s public functions take it;
/// its fields are private and only org-io builds one.
#[derive(Default)]
pub struct LocalStore {
    data_dir: Option<std::path::PathBuf>,
    keychain: Keychain,
    endpoints: Endpoints,
}

impl LocalStore {
    /// Write `change` (a sealed store to its file, a forgotten Persona's
    /// file removed; LLR-b9bm36, LLR-amv4mb). When that fails, the Persona is
    /// reloaded into `service` from its file, which reads its keychain item
    /// once more, so the service never runs ahead of the file; when the
    /// reload fails too, its store is dropped from the service (LLR-murjr6).
    pub(crate) fn store(&self, service: &mut OrgService, change: StoreChange) -> Result<(), StoreFailure> {
        let Some(data_dir) = &self.data_dir else { return Ok(()) };
        let (device_key, stored) = match change {
            StoreChange::Sealed(write) => (write.device_key, store_file::write_store(data_dir, &write)),
            StoreChange::Forgotten { device_key } => (device_key, store_file::remove_store(data_dir, &device_key)),
        };
        let Err(failed) = stored else { return Ok(()) };
        match open_store(data_dir, &self.keychain, device_key) {
            Ok(store) => {
                service.restore_store(store);
                Err(StoreFailure::Reloaded { failed })
            }
            Err(reload) => {
                service.drop_store(&device_key);
                Err(StoreFailure::Dropped { failed, reload: Box::new(reload) })
            }
        }
    }

    /// `store` for a sealed store.
    pub(crate) fn write(&self, service: &mut OrgService, write: StoreWrite) -> Result<(), StoreFailure> {
        self.store(service, StoreChange::Sealed(write))
    }

    /// Send every body `outcome` carries from the endpoint of `sender`, the
    /// Device of the Persona the commit names (LLR-d2pzzk, LLR-znhmh2,
    /// LLR-r9sh6k).
    pub(crate) async fn send_committed(
        &self,
        sender: Result<DevicePublicKey, String>,
        outcome: &org_node::CommitOutcome,
        peer_addr: Option<PeerAddr>,
    ) -> SendReport {
        send::send_committed(&self.endpoints, sender, outcome, peer_addr).await
    }
}

/// The store of `device_key`, read from its file in `data_dir` and unsealed
/// with its keychain item (LLR-uzax55).
#[allow(clippy::result_large_err, reason = "every refusal names its Device public key, a parsed point, as the keychain's do")]
fn open_store(data_dir: &std::path::Path, keychain: &Keychain, device_key: DevicePublicKey) -> Result<PersonaStore, OpenError> {
    let path = store_file::store_path(data_dir, &device_key);
    let sealed = store_file::read_store(data_dir, &device_key).map_err(OpenError::ReadStore)?;
    keychain.unseal(&sealed, device_key).map_err(|failure| match failure {
        UnsealFailure::Keychain(KeychainError::ItemMissing { device_key }) => OpenError::KeychainItemMissing { device_key, path },
        UnsealFailure::Keychain(error) => OpenError::Keychain { path, error },
        UnsealFailure::Store(error) => OpenError::Unseal { device_key, path, error },
    })
}

/// Why `OrgIo::open` opened no handle (LLR-uzax55): public data only —
/// paths, Device public keys and the keychain's and org-node's typed
/// refusals.
#[derive(Debug)]
pub enum OpenError {
    /// The data directory could not be created; `source` is the operating
    /// system's error.
    CreateDataDir { path: std::path::PathBuf, source: std::io::Error },
    /// The data directory could not be listed.
    ListDataDir(StoreFileError),
    /// A listed store file could not be read.
    ReadStore(StoreFileError),
    /// The keychain holds no item for a listed store's Device key; the file
    /// is left untouched.
    KeychainItemMissing { device_key: DevicePublicKey, path: std::path::PathBuf },
    /// The keychain refused to give a listed store's secret (LLR-8erfbu).
    Keychain { path: std::path::PathBuf, error: KeychainError },
    /// org-node refused a listed store's sealed bytes.
    Unseal { device_key: DevicePublicKey, path: std::path::PathBuf, error: OrgNodeError },
    /// org-node refused the stores together.
    Service(OrgNodeError),
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateDataDir { path, source } => write!(formatter, "create data_dir {}: {source}", path.display()),
            Self::ListDataDir(error) => write!(formatter, "read data_dir: {error}"),
            Self::ReadStore(error) => write!(formatter, "open store: {error}"),
            Self::KeychainItemMissing { device_key, path } => {
                write!(formatter, "open store {}: no keychain item for DevicePublicKey {device_key:?}", path.display())
            }
            Self::Keychain { path, error } => write!(formatter, "open store {}: {error}", path.display()),
            Self::Unseal { path, error, .. } => write!(formatter, "open store {}: {error}", path.display()),
            Self::Service(error) => write!(formatter, "open store: {error}"),
        }
    }
}

impl std::error::Error for OpenError {
    /// The operating system's error a data-directory or store-file failure
    /// received; the other refusals keep their typed cause in their fields.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreateDataDir { source, .. } => Some(source),
            Self::ListDataDir(error) | Self::ReadStore(error) => Some(error),
            _ => None,
        }
    }
}

/// A store change that was not written (LLR-murjr6, LLR-amv4mb): the
/// operation's change was made in memory, and the Persona was reloaded from
/// its file, or, when that failed too, dropped from the service.
#[derive(Debug)]
pub enum StoreFailure {
    Reloaded { failed: StoreFileError },
    Dropped { failed: StoreFileError, reload: Box<OpenError> },
}

impl std::fmt::Display for StoreFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reloaded { failed } => write!(formatter, "storing it failed; reloaded from the file ({failed})"),
            Self::Dropped { failed, reload } => write!(
                formatter,
                "storing it failed ({failed}), and reloading from the file failed too ({reload}); \
                 the Persona's store was dropped from the service"
            ),
        }
    }
}

impl std::error::Error for StoreFailure {}

/// Why an operation of the handle failed: org-node refused it, or its store
/// change was not written.
#[derive(Debug)]
pub enum OperationError {
    Node(OrgNodeError),
    Store(StoreFailure),
}

impl From<OrgNodeError> for OperationError {
    fn from(error: OrgNodeError) -> Self {
        Self::Node(error)
    }
}

impl std::fmt::Display for OperationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(error) => write!(formatter, "{error}"),
            Self::Store(failure) => write!(formatter, "the operation was made, but {failure}"),
        }
    }
}

impl std::error::Error for OperationError {}

/// Why `OrgIo::create_persona` created no Persona (LLR-n9dnm4): org-node
/// refused, the keychain refused the secret (no file is written), or the
/// store file was not written.
#[derive(Debug)]
pub enum CreateError {
    Node(OrgNodeError),
    Keychain(KeychainError),
    Store(StoreFailure),
}

impl std::fmt::Display for CreateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(error) => write!(formatter, "{error}"),
            Self::Keychain(error) => write!(formatter, "keychain write failed; no Persona created: {error}"),
            Self::Store(failure) => write!(formatter, "the Persona's secret is in the keychain, but {failure}"),
        }
    }
}

impl std::error::Error for CreateError {}

/// org-node's queries the app makes and the operations that build an update
/// or edit the store without judging against the chain, each taking the
/// service's lock inside and returning owned public data only
/// (LLR-3zdw8v). org-node's
/// chain-judging operations (`commit_genesis`, `commit_update`,
/// `reconcile`, `check_received`/`commit_checked`) are reached only through
/// the submission and the receive step, with the state org-io read
/// (REQ-tg9zrn).
impl OrgIo {
    /// Every Persona this node holds, public fields only.
    pub async fn personas(&self) -> Vec<PersonaSummary> {
        self.service.lock().await.list_personas().into_iter().map(PersonaSummary::from).collect()
    }

    /// Every Organisation this node holds, public fields only.
    pub async fn organisations(&self) -> Vec<OrgSummary> {
        self.service.lock().await.list_orgs().into_iter().map(OrgSummary::from).collect()
    }

    /// The Organisation `org_id`, when this node holds it.
    pub async fn organisation(&self, org_id: OrgId) -> Option<OrgSummary> {
        self.service.lock().await.list_orgs().into_iter().find(|record| record.org_id == org_id).map(OrgSummary::from)
    }

    /// The Persona's Member-as-a-group key and DevicePublicKey (org-node's
    /// `persona_public_keys`).
    pub async fn persona_public_keys(&self, persona_id: &PersonaId) -> Result<(PersonPublicKey, DevicePublicKey), OrgNodeError> {
        self.service.lock().await.persona_public_keys(persona_id)
    }

    /// org-node's `admit_member`: builds the provisional update, which
    /// org-node keeps, and writes its store; returns its name for
    /// `submit_commit_send`. A failed write reloads (LLR-murjr6).
    pub async fn admit_member<R: RngCore + CryptoRng + Send>(
        &self,
        rng: &mut R,
        org_id: OrgId,
        joiner: &org_node::service::Joiner,
    ) -> Result<BuiltUpdate, OperationError> {
        let mut service = self.service.lock().await;
        let (update, write) = service.admit_member(rng, org_id, joiner)?;
        self.local.write(&mut service, write).map_err(OperationError::Store)?;
        Ok(BuiltUpdate::of(&update).ok_or(OrgNodeError::OrgNotHeld { org_id })?)
    }

    /// org-node's `revoke_member`: builds the provisional update, which
    /// org-node keeps, and writes its store; returns its name for
    /// `submit_commit_send`. A failed write reloads (LLR-murjr6).
    pub async fn revoke_member<R: RngCore + CryptoRng + Send>(
        &self,
        rng: &mut R,
        org_id: OrgId,
        member_id: org_node::MemberId,
    ) -> Result<BuiltUpdate, OperationError> {
        let mut service = self.service.lock().await;
        let (update, write) = service.revoke_member(rng, org_id, member_id)?;
        self.local.write(&mut service, write).map_err(OperationError::Store)?;
        Ok(BuiltUpdate::of(&update).ok_or(OrgNodeError::OrgNotHeld { org_id })?)
    }

    /// org-node's `expect_admission`, in `persona_id`'s store (an
    /// expectation is the replying Persona's), written to its file; a failed
    /// write reloads (LLR-murjr6). Test builds only — a caller declares an expectation through `produce_reply`,
    /// behind the user's confirmation (LLR-9cb443); tests that need a bare
    /// expectation keep this.
    #[cfg(feature = "test-support")]
    pub async fn expect_admission<R: RngCore + CryptoRng + Send>(
        &self,
        rng: &mut R,
        persona_id: &PersonaId,
        org_id: OrgId,
    ) -> Result<(), OperationError> {
        let mut service = self.service.lock().await;
        let write = service.expect_admission(rng, persona_id, org_id)?;
        self.local.write(&mut service, write).map_err(OperationError::Store)
    }
}

impl OrgIo {
    /// The chain is not configured: every read and every write refused. The
    /// handle has no data directory: its store changes are kept in memory
    /// only.
    pub fn not_configured(service: OrgService) -> Self {
        Self::unconfigured(service, LocalStore::default())
    }

    /// `not_configured`, keeping `local`'s data directory and keychain.
    pub(crate) fn unconfigured(service: OrgService, local: LocalStore) -> Self {
        Self {
            service: tokio::sync::Mutex::new(service),
            local,
            reader: Box::new(connect::StateReaderNotConfigured),
            writer: Box::new(submit::WriterNotConfigured),
            signatory_reader: Box::new(signatory::SignatorySetNotConfigured),
            own_account: None,
            co_signers: Vec::new(),
            publish: Default::default(),
        }
    }

    /// Open `data_dir` and org-node's service over its stores, with the
    /// chain not configured; `connect` configures it. The app's way to a
    /// handle: it never builds org-node's service itself. The data directory is created if missing; every store
    /// file in it (`store_file::list_stores`) is read and unsealed with its
    /// Persona's keychain item. A refusal names the directory, or the store
    /// file and its Device key, and creates nothing: a missing item
    /// (`OpenError::KeychainItemMissing`) leaves its file untouched
    /// (LLR-uzax55, REQ-fh5gsq). Each Persona's endpoint is bound in
    /// `transport_mode` when it is first needed.
    #[allow(clippy::result_large_err, reason = "a refusal names its Device public key; opening runs once, at startup")]
    pub fn open(data_dir: &std::path::Path, transport_mode: TransportMode) -> Result<Self, OpenError> {
        std::fs::create_dir_all(data_dir)
            .map_err(|source| OpenError::CreateDataDir { path: data_dir.to_path_buf(), source })?;
        let keychain = Keychain;
        let stores = store_file::list_stores(data_dir)
            .map_err(OpenError::ListDataDir)?
            .into_iter()
            .map(|device_key| open_store(data_dir, &keychain, device_key))
            .collect::<Result<Vec<_>, _>>()?;
        let service = OrgService::new(stores).map_err(OpenError::Service)?;
        let local = LocalStore {
            data_dir: Some(data_dir.to_path_buf()),
            keychain,
            endpoints: Endpoints::new(transport_mode.into()),
        };
        Ok(Self::unconfigured(service, local))
    }

    /// Create a Persona (LLR-n9dnm4): org-node draws it and its device
    /// secret; the secret is written to the OS keychain, never returned (plan
    /// §17.2 rules 1–2), and only then the Persona's store file. When the
    /// keychain refuses, the new Persona is dropped from the service and no
    /// file is written; when the file is not written, the Persona, which has
    /// no file to reload from, is dropped too (LLR-murjr6). Returns the
    /// `PersonaId` alone.
    ///
    /// When the node receives, the new Persona's endpoint is then bound and
    /// accepts (PR-y6t5kt, LLR-z35vbh). A
    /// bind that fails there does not undo the creation, which is complete:
    /// `produce_reply` binds again, and refuses, before the Persona's reply
    /// can invite a message this node would not receive.
    #[allow(clippy::result_large_err, reason = "a keychain refusal names its Device public key; not on a hot path")]
    pub async fn create_persona<R: RngCore + CryptoRng + Send>(
        &self,
        rng: &mut R,
        handle: org_node::Handle,
        name: org_node::Name,
        surname: org_node::Surname,
    ) -> Result<PersonaId, CreateError> {
        let mut service = self.service.lock().await;
        let (persona_id, secret, write) = service.create_persona(rng, handle, name, surname).map_err(CreateError::Node)?;
        if let Err(refused) = self.local.keychain.keep(secret) {
            service.drop_store(&write.device_key);
            return Err(CreateError::Keychain(refused));
        }
        let device_key = write.device_key;
        self.local.write(&mut service, write).map_err(CreateError::Store)?;
        // A failed bind is retried by `produce_reply` (above).
        let _ = self.local.endpoints.accept_if_receiving(&device_key).await;
        Ok(persona_id)
    }

    /// Connect this handle: the signing seed from the environment
    /// (development builds only, LLR-rgdx22), the co-signer from `settings`,
    /// then the chain; the read and the write are built together over one
    /// client (owner ruling of 2026-10-07: coupled in S2, split in S8).
    /// A refusal keeps the service untouched with its reason
    /// (`ConnectFailed`, LLR-rgdx22).
    #[cfg(feature = "dev-seed")]
    pub async fn connect(self, settings: ChainSettings) -> Result<Self, Box<ConnectFailed>> {
        use on_chain_client::write::subxt_ops::{FinalitySink, SubxtWriteOps};

        let (service, local) = (self.service.into_inner(), self.local);

        let signatory = match custody::signatory_from_environment() {
            Ok(Some(signatory)) => signatory,
            Ok(None) => return Err(ConnectFailed::boxed(service, local, ConnectFailure::SeedNotSet)),
            Err(error) => return Err(ConnectFailed::boxed(service, local, ConnectFailure::Config(error))),
        };
        let co_signatories: Vec<_> = match settings.co_signer.as_deref().map(custody::parse_co_signer).transpose() {
            Ok(co_signer) => co_signer.into_iter().collect(),
            Err(error) => return Err(ConnectFailed::boxed(service, local, ConnectFailure::Config(error))),
        };
        // The writer dispatches with the co-signers as the other signatories;
        // the own account among them is refused by the chain (LLR-qhyc3n).
        if co_signatories.contains(&signatory.account_id()) {
            return Err(ConnectFailed::boxed(service, local, ConnectFailure::CoSignerIsOwnAccount));
        }
        let (api, registry) = match connect::connect(&settings.ws_url, settings.contract_h160).await {
            Ok(connection) => connection,
            Err(error) => {
                let reason = ConnectFailure::Chain { ws_url: settings.ws_url, error };
                return Err(ConnectFailed::boxed(service, local, reason));
            }
        };
        let own_account = signatory.account_id();
        let writer = submit::OnChainWriter::new(
            SubxtWriteOps::new(api, FinalitySink),
            registry.clone(),
            signatory,
            co_signatories.clone(),
            settings.contract_h160,
        );
        Ok(Self {
            service: tokio::sync::Mutex::new(service),
            local,
            reader: Box::new(chain_read::OnChainStateReader::new(registry.clone())),
            writer: Box::new(writer),
            signatory_reader: Box::new(signatory::OnChainSignatorySetReader::new(registry)),
            own_account: Some(own_account),
            co_signers: co_signatories,
            publish: Default::default(),
        })
    }

    /// Built without `dev-seed`: no seed is read and nothing is connected
    /// (LLR-rgdx22); the service is kept with the refusal.
    #[cfg(not(feature = "dev-seed"))]
    pub async fn connect(self, _settings: ChainSettings) -> Result<Self, Box<ConnectFailed>> {
        Err(ConnectFailed::boxed(self.service.into_inner(), self.local, ConnectFailure::DevSeedNotBuilt))
    }

    /// Over a substitute chain: `chain` is the read, the write and the
    /// signatory-set reads. No signatory account: see `with_own_account`.
    #[cfg(feature = "test-support")]
    pub fn for_test(service: OrgService, chain: &test_support::FakeChain) -> Self {
        Self {
            service: tokio::sync::Mutex::new(service),
            local: LocalStore::default(),
            reader: Box::new(chain.clone()),
            writer: Box::new(chain.clone()),
            signatory_reader: Box::new(chain.clone()),
            own_account: None,
            co_signers: Vec::new(),
            publish: Default::default(),
        }
    }

    /// `open`, over a substitute chain: `chain` is the read, the write and
    /// the signatory-set reads (test builds only): for the app's tests,
    /// which build their handle from a data directory and cannot name
    /// org-node's service to hand `for_test`.
    #[cfg(feature = "test-support")]
    #[allow(clippy::result_large_err, reason = "as `open`'s")]
    pub fn open_for_test(
        data_dir: &std::path::Path,
        transport_mode: TransportMode,
        chain: &test_support::FakeChain,
    ) -> Result<Self, OpenError> {
        let mut handle = Self::open(data_dir, transport_mode)?;
        handle.reader = Box::new(chain.clone());
        handle.writer = Box::new(chain.clone());
        handle.signatory_reader = Box::new(chain.clone());
        Ok(handle)
    }

    /// This handle, writing its store changes to `data_dir` as `open`'s
    /// handle does (test builds only).
    #[cfg(feature = "test-support")]
    pub fn with_data_dir_for_test(mut self, data_dir: std::path::PathBuf) -> Self {
        self.local.data_dir = Some(data_dir);
        self
    }

    /// This handle, binding its endpoints for `plan` (test builds only: the
    /// in-process relay plan, `BindPlan::Relay`, dials by endpoint id as
    /// Networked does).
    #[cfg(feature = "test-support")]
    pub fn with_bind_plan_for_test(mut self, plan: transport::BindPlan) -> Self {
        self.local.endpoints = Endpoints::new(plan);
        self
    }

    /// The account and co-signers the own-admin check uses, as a node holding
    /// that account's key would have them (test builds only).
    #[cfg(feature = "test-support")]
    pub fn with_own_account(mut self, own_account: AccountId, co_signers: Vec<AccountId>) -> Self {
        self.own_account = Some(own_account);
        self.co_signers = co_signers;
        self
    }

    /// Whether this node is an admin of `org_id` (LLR-c9fyun, LLR-2hkqv2):
    /// `NotConfigured` when it holds no signatory key, `Read` when the one
    /// signatory-set read fails (never `NotAdmin`), otherwise the rule of
    /// `signatory::admin_status`. Nothing in part 1 acts on the answer.
    pub async fn is_own_admin(&self, org_id: OrgId) -> Result<AdminStatus, OwnAdminError> {
        signatory::own_admin_status(&*self.signatory_reader, self.own_account, &self.co_signers, org_id).await
    }

    /// org-node's whole service, under the handle's lock, for tests that
    /// inspect what the handle's surface does not offer or drive one of its
    /// operations directly (test builds only). A test that asks while a
    /// handle method holds the lock is a failed test.
    #[cfg(feature = "test-support")]
    #[allow(clippy::expect_used, reason = "test builds only: the lock held by a running method is a failed test")]
    pub fn node_for_test(&self) -> tokio::sync::MutexGuard<'_, OrgService> {
        self.service.try_lock().expect("node_for_test while a handle method holds the node")
    }

    /// The endpoint of `persona_id`'s Device, bound on first use (test builds
    /// only: the endpoint holds the Device's secret). An unknown Persona is
    /// refused as a failed bind.
    #[cfg(feature = "test-support")]
    pub async fn endpoint_for_test(&self, persona_id: &PersonaId) -> Result<std::sync::Arc<transport::OrgEndpoint>, TransportError> {
        let (_, device_key) =
            self.persona_public_keys(persona_id).await.map_err(|error| TransportError::Bind(error.to_string()))?;
        self.local.endpoints.endpoint(&device_key).await
    }

    /// The endpoint of `persona_id`'s Device if the node has bound one,
    /// without binding it or starting an accept task, as `endpoint_for_test`
    /// would (test builds only). `None` for an unknown Persona too.
    #[cfg(feature = "test-support")]
    pub async fn bound_endpoint_for_test(&self, persona_id: &PersonaId) -> Option<std::sync::Arc<transport::OrgEndpoint>> {
        let (_, device_key) = self.persona_public_keys(persona_id).await.ok()?;
        self.local.endpoints.bound_endpoint(&device_key).await
    }

    /// How many endpoints the node has bound (test builds only).
    #[cfg(feature = "test-support")]
    pub async fn bound_endpoints_for_test(&self) -> usize {
        self.local.endpoints.bound_count().await
    }

    /// The next body on the receive channel and its authenticated sender,
    /// before org-node sees it (test builds only).
    #[cfg(feature = "test-support")]
    pub async fn next_inbound_for_test(&self) -> Result<(DevicePublicKey, Vec<u8>), TransportError> {
        self.local.endpoints.next_inbound().await.map(|inbound| (inbound.sender, inbound.body))
    }

    /// The mode the node's endpoints are bound in (test builds only).
    #[cfg(feature = "test-support")]
    pub fn transport_mode_for_test(&self) -> TransportMode {
        self.local.endpoints.transport_mode()
    }

    /// Found an Organisation: write, read back, commit (LLR-qhjp6g,
    /// LLR-be3zv9, LLR-9r2bxd).
    pub async fn found_organisation<R: RngCore + CryptoRng + Send>(
        &self,
        rng: &mut R,
        persona_id: &PersonaId,
    ) -> Result<OrgId, String> {
        let mut service = self.service.lock().await;
        submit::found_organisation(&mut service, &self.local, &*self.writer, &*self.reader, rng, persona_id).await
    }

    /// Submit the update `update` names, as org-node holds it: settle an
    /// earlier write whose outcome is unknown (LLR-2yjuaj, LLR-nqvbb6), then
    /// write, read back, commit, then send to every recipient the commit
    /// names (LLR-qhjp6g, LLR-be3zv9, LLR-9r2bxd, LLR-znhmh2, LLR-r9sh6k,
    /// with `peer_addr` the one address a Loopback send dials); returns the
    /// committed update's public data and the per-Device send report only,
    /// never org-node's outcome, whose bodies carry the Organisation private
    /// key (LLR-3zdw8v, and org-node's requirement that the commit builds
    /// every recipient's body).
    pub async fn submit_commit_send<R: RngCore + CryptoRng + Send>(
        &self,
        rng: &mut R,
        update: &BuiltUpdate,
        peer_addr: Option<PeerAddr>,
    ) -> Result<Submitted, String> {
        let mut service = self.service.lock().await;
        let mut publish = self.publish.lock().await;
        submit::submit_commit_send(&mut service, &mut publish, &self.local, &*self.writer, &*self.reader, rng, update, peer_addr)
            .await
    }

    /// Settle `org_id`'s chain write whose outcome is unknown, by reading
    /// the chain (LLR-t98pk9). Holds the service's lock for the whole
    /// sequence, as the submission does; a late
    /// commit is sent as its submission would have sent it, to every
    /// recipient, at the address the submission was given.
    pub async fn settle_unknown<R: RngCore + CryptoRng + Send>(&self, rng: &mut R, org_id: OrgId) -> Settled {
        let mut service = self.service.lock().await;
        let mut publish = self.publish.lock().await;
        publish::settle_unknown(&mut publish, &mut service, &self.local, &*self.reader, rng, org_id).await
    }

    /// Reconcile `org_id` with the chain: one read, handed to org-node's
    /// `reconcile`; `Unverified` when the read fails or its parse refuses,
    /// with nothing committed or deleted. Sends nothing but the update of a
    /// write whose outcome was unknown, which it commits, to the recipients
    /// its submission would have sent it to (LLR-k5pzsz, LLR-kp95ty).
    pub async fn reconcile<R: RngCore + CryptoRng + Send>(&self, rng: &mut R, org_id: OrgId) -> Reconciliation {
        let mut service = self.service.lock().await;
        let mut publish = self.publish.lock().await;
        publish::reconcile(&mut publish, &mut service, &self.local, &*self.reader, rng, org_id).await
    }

    /// `reconcile` for every Organisation the store holds, in store order:
    /// the app's call at start (LLR-k5pzsz).
    pub async fn reconcile_held<R: RngCore + CryptoRng + Send>(&self, rng: &mut R) -> Vec<Reconciliation> {
        let mut service = self.service.lock().await;
        let mut publish = self.publish.lock().await;
        publish::reconcile_held(&mut publish, &mut service, &self.local, &*self.reader, rng).await
    }

    /// `org_id`'s verification status as its last reconcile left it; `None`
    /// until one ran on this handle (LLR-kp95ty).
    pub async fn verification(&self, org_id: OrgId) -> Option<Verification> {
        self.publish.lock().await.verification(org_id)
    }
}
