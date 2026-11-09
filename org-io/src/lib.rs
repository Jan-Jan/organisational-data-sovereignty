//! org-io: all IO for an Organisation (docs/adr/2026-10-06-org-io-unit.md).
//! org-node takes values and returns values; org-io reads and writes the
//! chain, holds the user's own signatory key, and is the app's one way in.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod acknowledge;
pub mod chain_read;
pub mod clock;
pub mod connect;
pub mod custody;
mod endpoints;
mod exchange;
mod keychain;
mod person;
mod persona;
pub mod preflight;
mod propose;
mod receiver;
mod removed;
mod retention;
mod rng;
pub mod send;
pub mod store_file;
#[cfg(feature = "test-support")]
pub mod test_support;
pub mod transport;
pub mod verify;
pub mod view;
pub mod write;

/// org-node's value types the app names: identifiers, names, public keys,
/// org-node's refusals and the public Invite. Nothing else: org-node's
/// service, its stores and their records, the Device secret and the Wire
/// messages are not reachable through org-io (LLR-3zdw8v).
pub mod node {
    pub use org_node::invitation::{Invite, InviteId};
    pub use org_node::{
        DevicePublicKey, Epoch, Handle, MemberId, Name, OrgId, OrgMembersError, OrgNodeError, PersonPublicKey, PersonaId,
        RootHash, Surname,
    };
}

pub use crate::connect::{ConnectFailed, ConnectFailure};
pub use crate::exchange::ExchangeError;
pub use crate::keychain::KeychainError;
pub use crate::persona::{ProposedFounding, ProposedMember};
pub use crate::propose::{ProposedOrganisation, ProvisionalChangeSummary};
pub use crate::receiver::{ReceiveError, Received, TerminalCause};
pub use crate::removed::{RemovedDevice, RemovedDevicesRefused};
pub use crate::send::{Delivery, RequestError, SendError, SendRefused, SendReport};
pub use crate::verify::{Verified, WriteOutcome};
pub use crate::view::{
    AcceptanceFailure, AcknowledgementOutcome, AcknowledgementReport, DeletionFailure, InviteSummary, RoundFailure, KeptAdmissionRequestSummary, OrgSummary, PersonSummary,
    PersonaSummary,
};
pub use crate::write::{HeldTarget, WriteFailure, WriteRefused, WriteReport};

use org_node::service::OrgService;
use org_node::store::PersonaStore;
use org_node::{DevicePublicKey, OrgId, OrgNodeError, StoreChange};
#[cfg(feature = "test-support")]
use org_node::{PersonPublicKey, PersonaId};

use crate::chain_read::StateReader;
use crate::endpoints::Endpoints;
use crate::keychain::{Keychain, UnsealFailure};
use crate::rng::HandleRng;
use crate::store_file::StoreFileError;
#[cfg(feature = "test-support")]
use crate::transport::TransportError;
use crate::transport::TransportMode;
use crate::write::ChainWriter;

/// The chain's settings as the app read them from its environment; org-io
/// reads the signing seed itself, in development builds only (REQ-8zuka3).
pub struct ChainSettings {
    pub ws_url: String,
    pub contract_h160: [u8; 20],
    /// `ODS_COSIGNER_PUB` as the app read it; parsed here (LLR-9fy622).
    pub co_signer: Option<String>,
    /// How long verification keeps received Organisation information one epoch ahead of the
    /// chain unadopted, in whole seconds (a fraction is dropped): the app gives twice its
    /// verification interval (LLR-22se2q).
    pub ahead_limit: std::time::Duration,
}

/// The ahead limit a handle holds for `limit`: its whole seconds, a fraction dropped. The one
/// conversion `connect` and the test-support constructors give a limit through (LLR-22se2q).
#[cfg(any(feature = "dev-seed", feature = "test-support"))]
pub(crate) fn ahead_limit_from(limit: std::time::Duration) -> org_node::AheadLimit {
    org_node::AheadLimit(limit.as_secs())
}

/// The app's one way in (SDD-z3ychz): org-node's service, the chain read
/// and the chain write. The signatory key is held by
/// the writer alone and never handed out (LLR-u2pk5y). The service is behind
/// the handle's own lock and every method takes `&self`; each takes the lock
/// inside and returns owned public values.
pub struct OrgIo {
    /// Shared with the removal's delivery tasks (SDD-3my66m).
    service: std::sync::Arc<tokio::sync::Mutex<OrgService>>,
    local: LocalStore,
    reader: Box<dyn StateReader>,
    writer: Box<dyn ChainWriter>,
    /// The unknown writes, one per target, kept in memory (SDD-mk39e5, LLR-7y46be).
    publish: tokio::sync::Mutex<write::PublishState>,
    /// The removal acknowledgements' delivery tasks, one at most per Persona (LLR-zzsqj4);
    /// dropping the handle stops them.
    deliveries: acknowledge::Tasks,
    /// The ahead limit every verification gives org-node (LLR-22se2q): `connect`'s
    /// `ChainSettings::ahead_limit`; `None` on a handle with no chain, which verifies nothing.
    ahead_limit: Option<org_node::AheadLimit>,
}

/// Where org-io keeps what org-node hands out: each store change in its
/// Persona's store file (`store_file`), each new Persona's Device seed in
/// the OS keychain (`keychain`), the node's endpoints (`endpoints`) and the
/// handle's random source (`rng`, LLR-4qd5fx), and the Person, held apart
/// from org-node's service (`persons`, at most one: LLR-3cysjk, LLR-z7kbf9).
/// A handle built without a data directory keeps its stores in memory only.
/// The handle's clock (`clock`, LLR-fzf2n3) is the system's outside test builds.
pub(crate) struct LocalStore {
    data_dir: Option<std::path::PathBuf>,
    keychain: Keychain,
    endpoints: Endpoints,
    /// Shared with the removal's delivery tasks (SDD-3my66m).
    rng: std::sync::Arc<HandleRng>,
    persons: std::sync::Mutex<Vec<org_node::PersonStore>>,
    clock: std::sync::Arc<dyn clock::Clock>,
}

impl Default for LocalStore {
    fn default() -> Self {
        Self {
            data_dir: None,
            keychain: Keychain,
            endpoints: Endpoints::default(),
            rng: std::sync::Arc::new(HandleRng::os()),
            persons: std::sync::Mutex::new(Vec::new()),
            clock: std::sync::Arc::new(clock::SystemClock),
        }
    }
}

impl LocalStore {
    /// Write `change`, the sealed store org-node returned, to its file
    /// (LLR-b9bm36), drawing the temporary suffix from the handle's source.
    /// When that fails, the Persona is reloaded into `service` from its file,
    /// which reads its keychain item once more, so the service never runs
    /// ahead of the file; when the reload fails too, its store is dropped
    /// from the service (LLR-murjr6).
    pub(crate) fn store(&self, service: &mut OrgService, change: StoreChange) -> Result<(), StoreFailure> {
        self.writer().store(service, change)
    }

    /// The handle's store write, for a task that holds no `LocalStore` (a
    /// removal's delivery, SDD-3my66m): its data directory and its source.
    pub(crate) fn writer(&self) -> StoreWriter {
        StoreWriter { data_dir: self.data_dir.clone(), rng: self.rng.clone() }
    }
}

/// The handle's store write (`LocalStore::writer`): its data directory (none:
/// kept in memory only) and its random source, shared.
#[derive(Clone)]
pub(crate) struct StoreWriter {
    data_dir: Option<std::path::PathBuf>,
    pub(crate) rng: std::sync::Arc<HandleRng>,
}

impl StoreWriter {
    /// Write `change` to its Persona's store file, drawing the temporary
    /// suffix from the handle's source; on a failure the Persona is reloaded
    /// from its file, or dropped when that fails too (LLR-b9bm36,
    /// LLR-murjr6).
    pub(crate) fn store(&self, service: &mut OrgService, change: StoreChange) -> Result<(), StoreFailure> {
        let StoreChange::Sealed(write) = change;
        let Some(data_dir) = &self.data_dir else { return Ok(()) };
        let device_key = write.device_key;
        let Err(failed) = store_file::write_store(data_dir, &write, &self.rng) else { return Ok(()) };
        match open_store(data_dir, &Keychain, device_key) {
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

/// The Person file of `device_key`, unsealed with its keychain item
/// (LLR-z7kbf9): refused as a Persona's is, naming the file.
#[allow(clippy::result_large_err, reason = "as open_store's")]
fn open_person(
    data_dir: &std::path::Path,
    keychain: &Keychain,
    device_key: DevicePublicKey,
) -> Result<org_node::PersonStore, OpenError> {
    let path = store_file::person_path(data_dir, &device_key);
    let sealed = store_file::read_person_store(data_dir, &device_key).map_err(OpenError::ReadStore)?;
    keychain.unseal_person(&sealed, device_key).map_err(|failure| match failure {
        UnsealFailure::Keychain(KeychainError::ItemMissing { device_key }) => OpenError::KeychainItemMissing { device_key, path },
        UnsealFailure::Keychain(error) => OpenError::Keychain { path, error },
        UnsealFailure::Store(error) => OpenError::Unseal { device_key, path, error },
    })
}

/// The DevicePublicKey of the Persona holding `org_id` as its current
/// record; a founding or a provisional Member bound to `org_id` is not one
/// (`send::bound_holder_of` matches those too). Stores are named by
/// DevicePublicKey, so no Persona-id lookup is needed. The stores are
/// enumerated by the DevicePublicKeys of org-node's key-free record views
/// (`persona_records`); everything else is read through the Persona view.
pub(crate) fn record_holder_of(service: &OrgService, org_id: OrgId) -> Option<DevicePublicKey> {
    service.persona_records().into_iter().map(|persona| persona.device_key).find(|device_key| {
        // `persona_view` refuses only a key no store holds and a removed
        // store, and `persona_records` lists neither: no refusal is skipped.
        service.persona_view(device_key).is_ok_and(|view| view.org_id == Some(org_id) && view.record.is_some())
    })
}

/// The current record of `org_id`, read through the key-free Persona view
/// of the Persona `record_holder_of` finds (LLR-jqb5z7), not through
/// org-node's `OrgRecord`, which holds the Organisation private key.
pub(crate) fn held_record(service: &OrgService, org_id: OrgId) -> Option<org_node::store::RecordView> {
    let device_key = record_holder_of(service, org_id)?;
    service.persona_view(&device_key).ok()?.record
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
    /// The data directory holds more than one Person file (a corruption);
    /// none was unsealed (LLR-z7kbf9). The Device keys of every Person file,
    /// in byte order.
    MoreThanOnePerson { device_keys: Vec<DevicePublicKey> },
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
            Self::MoreThanOnePerson { device_keys } => write!(
                formatter,
                "the data directory holds {} Person files ({}); one Person per device",
                device_keys.len(),
                device_keys.iter().map(|key| hex::encode(key.as_bytes())).collect::<Vec<_>>().join(", ")
            ),
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

/// A store change that was not written (LLR-murjr6): the
/// operation's change was made in memory, and the Persona was reloaded from
/// its file, or, when that failed too, dropped from the service.
#[derive(Debug)]
pub enum StoreFailure {
    Reloaded { failed: StoreFileError },
    Dropped { failed: StoreFileError, reload: Box<OpenError> },
    /// A new store's file was not written; the store is not held (a Person: LLR-3cysjk).
    NotKept { failed: StoreFileError },
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
            Self::NotKept { failed } => write!(formatter, "storing it failed ({failed}); it is not held"),
        }
    }
}

impl std::error::Error for StoreFailure {}

/// The error an outcome kept as its cause, typed (ARCHITECTURE Boundaries 2:
/// the cause is kept, not turned into text). It is shared, so that the
/// outcome holding it stays `Clone`, and two are equal when they read the
/// same, since the cause need not be comparable itself. Its `Display` is the
/// cause's own; `get` gives the cause.
pub struct Cause<Failure>(std::sync::Arc<Failure>);

impl<Failure> Cause<Failure> {
    pub fn new(error: Failure) -> Self {
        Self(std::sync::Arc::new(error))
    }

    /// The cause.
    pub fn get(&self) -> &Failure {
        &self.0
    }
}

impl<Failure> Clone for Cause<Failure> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Failure: std::fmt::Display> PartialEq for Cause<Failure> {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0) || self.0.to_string() == other.0.to_string()
    }
}

impl<Failure: std::fmt::Display> Eq for Cause<Failure> {}

impl<Failure: std::fmt::Debug> std::fmt::Debug for Cause<Failure> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<Failure: std::fmt::Display> std::fmt::Display for Cause<Failure> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Why an operation of the handle failed: org-node refused it, its store
/// change was not written, or org-io's own lookup refused it.
#[derive(Debug)]
pub enum OperationError {
    Node(OrgNodeError),
    Store(StoreFailure),
    /// No Persona here holds `org_id`'s current record (org-io's lookup;
    /// nothing asked of org-node, nothing written).
    NotHeld { org_id: OrgId },
    /// org-node recorded the admission of `invite_id` and its store was
    /// written, but its provisional changes list none for that invite: the
    /// Handle given cannot be reported.
    AdmissionNotListed { invite_id: org_node::invitation::InviteId },
    /// org-node proposed `org_id`'s updated Organisation and its store was
    /// written, but its provisional organisations list no current proposal
    /// of this node's: the proposal cannot be reported.
    ProposalNotListed { org_id: OrgId },
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
            Self::NotHeld { org_id } => crate::write_not_held(formatter, org_id),
            Self::AdmissionNotListed { invite_id } => write!(
                formatter,
                "the admission of invite {} was recorded and written, but no provisional change lists it",
                hex::encode(invite_id.as_bytes())
            ),
            Self::ProposalNotListed { org_id } => write!(
                formatter,
                "Organisation {}'s proposal was made and written, but no current proposal is listed",
                hex::encode(org_id.as_bytes())
            ),
        }
    }
}

impl std::error::Error for OperationError {}

/// The text of a refusal because no Persona here holds `org_id`.
pub(crate) fn write_not_held(formatter: &mut std::fmt::Formatter<'_>, org_id: &OrgId) -> std::fmt::Result {
    write!(formatter, "no Persona here holds Organisation {}", hex::encode(org_id.as_bytes()))
}

/// Why a Persona or a Person was not created (LLR-kna4zj): org-node
/// refused, the keychain refused the secret (no file is written), or the
/// store file was not written.
#[derive(Debug)]
pub enum CreateError {
    Node(OrgNodeError),
    Keychain(KeychainError),
    Store(StoreFailure),
    /// A Person is held already: one Person per device (owner ruling
    /// 2026-10-22); nothing written.
    PersonAlreadyHeld { device_key: DevicePublicKey },
    /// org-node proposed the founding and its Persona was written, but
    /// org-node's view of that Persona holds no founding: the founding
    /// cannot be reported.
    FoundingNotListed { founder: DevicePublicKey },
}

impl std::fmt::Display for CreateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(error) => write!(formatter, "{error}"),
            Self::Keychain(error) => write!(formatter, "keychain write failed; nothing created: {error}"),
            Self::Store(failure) => write!(formatter, "the Device seed is in the keychain, but {failure}"),
            Self::PersonAlreadyHeld { device_key } => {
                write!(formatter, "a Person is held already ({}); nothing created", hex::encode(device_key.as_bytes()))
            }
            Self::FoundingNotListed { founder } => write!(
                formatter,
                "the founding of Persona {} was written, but its view holds no founding",
                hex::encode(founder.as_bytes())
            ),
        }
    }
}

impl std::error::Error for CreateError {}

/// The handle's production surface (LLR-zhdc9f): the domain's methods,
/// these queries, `open`, `connect` and `not_configured`. Each returns public data only, of the types
/// `tests/absences.rs`'s `HANDLE_RETURN_ALLOWLIST` lists (LLR-gh4rw3).
impl OrgIo {
    /// Every Persona this node holds, public fields only.
    pub async fn personas(&self) -> Vec<PersonaSummary> {
        self.service.lock().await.persona_records().into_iter().map(PersonaSummary::from).collect()
    }

    /// Every Organisation this node holds, public fields only, each with the
    /// name its store keeps (LLR-pn3hgc), read through org-node's key-free
    /// Persona views.
    pub async fn organisations(&self) -> Vec<OrgSummary> {
        let service = self.service.lock().await;
        service
            .persona_records()
            .into_iter()
            // A listed Persona's view is never refused (see `record_holder_of`).
            .filter_map(|persona| service.persona_view(&persona.device_key).ok())
            .filter_map(|view| view.record.as_ref().map(|record| OrgSummary::of(record, &view.org_name)))
            .collect()
    }

    /// The Organisation `org_id`, when this node holds it.
    pub async fn organisation(&self, org_id: OrgId) -> Option<OrgSummary> {
        self.organisations().await.into_iter().find(|summary| summary.org_id == org_id)
    }

    /// The Persona's Member-as-a-group key and DevicePublicKey (org-node's
    /// `persona_public_keys`; test builds only, LLR-zhdc9f).
    #[cfg(feature = "test-support")]
    pub async fn persona_public_keys(&self, persona_id: &PersonaId) -> Result<(PersonPublicKey, DevicePublicKey), OrgNodeError> {
        self.service.lock().await.persona_public_keys(persona_id)
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
        Self::unconfigured_shared(std::sync::Arc::new(tokio::sync::Mutex::new(service)), local)
    }

    /// `unconfigured`, over a service already shared (a refused connect's).
    pub(crate) fn unconfigured_shared(service: std::sync::Arc<tokio::sync::Mutex<OrgService>>, local: LocalStore) -> Self {
        Self {
            service,
            local,
            reader: Box::new(connect::StateReaderNotConfigured),
            writer: Box::new(write::WriterNotConfigured),
            publish: Default::default(),
            deliveries: Default::default(),
            ahead_limit: None,
        }
    }

    /// Open `data_dir` and org-node's service over its stores, with the
    /// chain not configured; `connect` configures it. The app's way to a
    /// handle: it never builds org-node's service itself. The data directory is created if missing; every store
    /// file in it (`store_file::list_stores`) is read and unsealed with its
    /// Persona's keychain item. A refusal names the directory, or the store
    /// file and its Device key, and creates nothing: a missing item
    /// (`OpenError::KeychainItemMissing`) leaves its file untouched
    /// (LLR-uzax55, REQ-fh5gsq). The Person file (`list_person_stores`) is
    /// listed apart and opened the same way; two or more are refused as
    /// `OpenError::MoreThanOnePerson` before any file is read (LLR-z7kbf9).
    /// Each Persona's endpoint is bound in
    /// `transport_mode` when it is first needed.
    #[allow(clippy::result_large_err, reason = "a refusal names its Device public key; opening runs once, at startup")]
    pub fn open(data_dir: &std::path::Path, transport_mode: TransportMode) -> Result<Self, OpenError> {
        Self::open_with(data_dir, transport_mode, std::sync::Arc::new(clock::SystemClock))
    }

    /// `open`, its handle reading `clock` (the system's, but under test-support: LLR-6vtbn6).
    #[allow(clippy::result_large_err, reason = "as open's")]
    fn open_with(
        data_dir: &std::path::Path,
        transport_mode: TransportMode,
        clock: std::sync::Arc<dyn clock::Clock>,
    ) -> Result<Self, OpenError> {
        std::fs::create_dir_all(data_dir)
            .map_err(|source| OpenError::CreateDataDir { path: data_dir.to_path_buf(), source })?;
        // Every deletion a `removed-` file records is finished before the
        // Persona files are listed, unsealing nothing (LLR-upqj5g).
        // A leftover beside a live store of its key is kept, and reported
        // once the stores are open (LLR-upqj5g).
        let kept_leftovers = retention::finish_leftovers(data_dir);
        let keychain = Keychain;
        // Two or more Person files are refused before any file is read
        // (LLR-z7kbf9).
        let person_keys = store_file::list_person_stores(data_dir).map_err(OpenError::ListDataDir)?;
        if person_keys.len() > 1 {
            return Err(OpenError::MoreThanOnePerson { device_keys: person_keys });
        }
        let store_keys = store_file::list_stores(data_dir).map_err(OpenError::ListDataDir)?;
        let stores =
            store_keys.iter().map(|device_key| open_store(data_dir, &keychain, *device_key)).collect::<Result<Vec<_>, _>>()?;
        let persons = person_keys
            .into_iter()
            .map(|device_key| open_person(data_dir, &keychain, device_key))
            .collect::<Result<Vec<_>, _>>()?;
        let mut service = OrgService::new(stores).map_err(OpenError::Service)?;
        // Each removal past its deadline, or keeping no acknowledgement, is
        // deleted before the handle is returned; a failure does not refuse
        // the open, it is reported (LLR-upqj5g, LLR-3fxxpp).
        let mut failures = retention::kept_leftover_failures(&service, &kept_leftovers);
        failures.extend(retention::delete_expired_at_open(data_dir, &mut service, &store_keys, clock.now()));
        let local = LocalStore {
            data_dir: Some(data_dir.to_path_buf()),
            keychain,
            endpoints: Endpoints::new(transport_mode.into()),
            rng: std::sync::Arc::new(HandleRng::os()),
            persons: std::sync::Mutex::new(persons),
            clock,
        };
        // The removed stores left after those deletions are remembered, so
        // that the report and the resume at start list no directory again
        // (LLR-5rxqr8, LLR-rav7dr).
        let deliveries = acknowledge::Tasks::default();
        deliveries.0.remember_removals_held(&service, &store_keys);
        deliveries.0.record_failures(failures);
        let mut handle = Self::unconfigured(service, local);
        handle.deliveries = deliveries;
        Ok(handle)
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

        let (service, local) = (self.service, self.local);

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
        // the own account among them is refused by the chain (LLR-rgdx22).
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
        let writer = write::OnChainWriter::new(
            SubxtWriteOps::new(api, FinalitySink),
            registry.clone(),
            signatory,
            co_signatories,
            settings.contract_h160,
        );
        Ok(Self {
            service,
            local,
            reader: Box::new(chain_read::OnChainStateReader::new(registry)),
            writer: Box::new(writer),
            publish: Default::default(),
            // The handle's delivery tasks go on in the connected handle.
            deliveries: self.deliveries,
            ahead_limit: Some(ahead_limit_from(settings.ahead_limit)),
        })
    }

    /// Built without `dev-seed`: no seed is read and nothing is connected
    /// (LLR-rgdx22); the service is kept with the refusal.
    #[cfg(not(feature = "dev-seed"))]
    pub async fn connect(self, _settings: ChainSettings) -> Result<Self, Box<ConnectFailed>> {
        Err(ConnectFailed::boxed(self.service, self.local, ConnectFailure::DevSeedNotBuilt))
    }

    /// Over a substitute chain: `chain` is the read and the write.
    #[cfg(feature = "test-support")]
    pub fn for_test(service: OrgService, chain: &test_support::FakeChain) -> Self {
        Self {
            service: std::sync::Arc::new(tokio::sync::Mutex::new(service)),
            local: LocalStore::default(),
            reader: Box::new(chain.clone()),
            writer: Box::new(chain.clone()),
            publish: Default::default(),
            deliveries: Default::default(),
            ahead_limit: Some(ahead_limit_from(test_support::TEST_AHEAD_LIMIT)),
        }
    }

    /// `open`, over a substitute chain: `chain` is the read and the write
    /// (test builds only): for the app's tests,
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
        handle.ahead_limit = Some(ahead_limit_from(test_support::TEST_AHEAD_LIMIT));
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

    /// What a removal's delivery task shares with this handle.
    pub(crate) fn delivery_context(&self) -> acknowledge::Context {
        acknowledge::Context {
            service: self.service.clone(),
            mode: self.local.endpoints.transport_mode(),
            bind_plan: self.local.endpoints.bind_plan(),
            deliveries: self.deliveries.0.clone(),
            store: self.local.writer(),
            clock: self.local.clock.clone(),
        }
    }

    /// The removal of `org_id` as its delivery stands (LLR-rav7dr): each
    /// recipient whose acknowledgement org-node still keeps, with its last
    /// round's outcome, the next round and the deadline; `None` for an
    /// Organisation no removed Persona holds, except while a failed deletion
    /// is retried (its failure reported) and for a `removed-` file kept
    /// beside a live store of its Persona (`DeletionFailure::LeftoverKept`,
    /// no recipient and no deadline). Writes and sends nothing.
    pub async fn acknowledgement_report(&self, org_id: OrgId) -> Option<AcknowledgementReport> {
        let service = self.service.lock().await;
        acknowledge::report(&service, &self.deliveries.0, org_id)
    }

    /// Wake the delivery tasks to read the handle's clock again (test
    /// builds only: a settable clock moves without a sleep ending).
    #[cfg(feature = "test-support")]
    pub fn wake_deliveries_for_test(&self) {
        self.deliveries.0.wake.notify_one();
    }

    /// The mode the node's endpoints are bound in (test builds only).
    #[cfg(feature = "test-support")]
    pub fn transport_mode_for_test(&self) -> TransportMode {
        self.local.endpoints.transport_mode()
    }
}
