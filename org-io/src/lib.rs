//! org-io: all IO for an Organisation (docs/adr/2026-10-06-org-io-unit.md).
//! org-node takes values and returns values; org-io reads and writes the
//! chain, holds the user's own signatory key, and is the app's one way in.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod chain_read;
pub mod connect;
pub mod custody;
pub mod preflight;
pub mod signatory;
pub mod submit;
#[cfg(feature = "test-support")]
pub mod test_support;
pub mod view;

/// org-node's types, re-exported for org-io's callers (transitional, accepted
/// 2026-10-06; S4 narrows it).
pub use org_node as node;

pub use crate::view::{BuiltUpdate, NodeView, OrgSummary, PersonaSummary};
use org_node::service::OrgService;
use org_node::{CommitOutcome, DevicePublicKey, OrgId, OrgNodeError, PersonaId, Prepared, ReceiveOutcome, SelfDeleteOutcome};
use rand_core::{CryptoRng, RngCore};

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
/// (LLR-c9fyun).
pub struct OrgIo {
    service: OrgService,
    reader: Box<dyn StateReader>,
    writer: Box<dyn ChainWriter>,
    signatory_reader: Box<dyn SignatorySetReader>,
    /// The account of the signatory key this node holds; `None` when it holds none.
    own_account: Option<AccountId>,
    co_signers: Vec<AccountId>,
}

/// org-node's service as `OrgIo::node_mut` hands it out: the read surface
/// (`view`) and the operations that build an update or edit the store
/// without judging against the chain, each returning public data only.
/// org-node's chain-judging operations (`commit_genesis`, `commit_update`,
/// `reconcile`, `prepare_*`/`apply_*`) are reached only through `OrgIo`'s
/// own methods, with the state org-io read (REQ-tg9zrn; review round 1,
/// finding 5); and nothing here dereferences to the service, whose
/// `endpoint()` would offer a receive and a send outside org-io's sequence
/// (review round 3, finding 1).
pub struct NodeBuilders<'handle> {
    service: &'handle mut OrgService,
}

impl NodeBuilders<'_> {
    /// The read surface (`NodeView`).
    pub fn view(&self) -> NodeView<'_> {
        NodeView::new(self.service)
    }

    /// org-node's `create_persona`.
    pub fn create_persona<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        handle: org_node::Handle,
        name: org_node::Name,
        surname: org_node::Surname,
    ) -> Result<PersonaId, OrgNodeError> {
        self.service.create_persona(rng, handle, name, surname)
    }

    /// org-node's `admit_member`: builds the provisional update, which
    /// org-node keeps; returns its name for `OrgIo::submit_commit_send`.
    pub fn admit_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        joiner: &org_node::service::Joiner,
    ) -> Result<BuiltUpdate, OrgNodeError> {
        let update = self.service.admit_member(rng, org_id, joiner)?;
        BuiltUpdate::of(&update).ok_or(OrgNodeError::OrgNotHeld { org_id })
    }

    /// org-node's `revoke_member`: builds the provisional update, which
    /// org-node keeps; returns its name for `OrgIo::submit_commit_send`.
    pub fn revoke_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        member_id: org_node::MemberId,
    ) -> Result<BuiltUpdate, OrgNodeError> {
        let update = self.service.revoke_member(rng, org_id, member_id)?;
        BuiltUpdate::of(&update).ok_or(OrgNodeError::OrgNotHeld { org_id })
    }

    /// org-node's `expect_admission`.
    pub fn expect_admission<R: RngCore + CryptoRng>(&mut self, rng: &mut R, org_id: OrgId) -> Result<(), OrgNodeError> {
        self.service.expect_admission(rng, org_id)
    }

    /// org-node's `ensure_endpoint` (test builds only).
    #[cfg(feature = "test-support")]
    pub async fn ensure_endpoint(
        &mut self,
        persona_id: &PersonaId,
    ) -> Result<&org_node::transport::endpoint::OrgEndpoint, OrgNodeError> {
        self.service.ensure_endpoint(persona_id).await
    }
}

impl OrgIo {
    /// The chain is not configured: every read and every write refused.
    pub fn not_configured(service: OrgService) -> Self {
        Self {
            service,
            reader: Box::new(connect::StateReaderNotConfigured),
            writer: Box::new(submit::WriterNotConfigured),
            signatory_reader: Box::new(signatory::SignatorySetNotConfigured),
            own_account: None,
            co_signers: Vec::new(),
        }
    }

    /// Open the Persona store `persona_store.bin` in `data_dir` (created if
    /// missing) under `passphrase`, and org-node's service over it, with the
    /// chain not configured; `connect` configures it. The app's way to a
    /// handle: it never builds org-node's service itself (review round 2,
    /// finding 2). Refusals: `create data_dir <path>: …` and
    /// `open store: …` (LLR-rgdx22, clause of 2026-10-08).
    pub fn open(
        data_dir: &std::path::Path,
        passphrase: &str,
        transport_mode: org_node::transport::TransportMode,
    ) -> Result<Self, String> {
        std::fs::create_dir_all(data_dir).map_err(|error| format!("create data_dir {}: {error}", data_dir.display()))?;
        let store = org_node::store::PersonaStore::open(data_dir.join("persona_store.bin"), passphrase)
            .map_err(|error| format!("open store: {error}"))?;
        let mut service = OrgService::new(store);
        service.set_transport_mode(transport_mode);
        Ok(Self::not_configured(service))
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

        let service = self.service;

        let signatory = match custody::signatory_from_environment() {
            Ok(Some(signatory)) => signatory,
            Ok(None) => return Err(ConnectFailed::boxed(service, ConnectFailure::SeedNotSet)),
            Err(error) => return Err(ConnectFailed::boxed(service, ConnectFailure::Config(error))),
        };
        let co_signatories: Vec<_> = match settings.co_signer.as_deref().map(custody::parse_co_signer).transpose() {
            Ok(co_signer) => co_signer.into_iter().collect(),
            Err(error) => return Err(ConnectFailed::boxed(service, ConnectFailure::Config(error))),
        };
        // The writer dispatches with the co-signers as the other signatories;
        // the own account among them is refused by the chain (LLR-qhyc3n).
        if co_signatories.contains(&signatory.account_id()) {
            return Err(ConnectFailed::boxed(service, ConnectFailure::CoSignerIsOwnAccount));
        }
        let (api, registry) = match connect::connect(&settings.ws_url, settings.contract_h160).await {
            Ok(connection) => connection,
            Err(error) => {
                let reason = ConnectFailure::Chain { ws_url: settings.ws_url, error };
                return Err(ConnectFailed::boxed(service, reason));
            }
        };
        let own_account = signatory.account_id();
        let writer = submit::OnChainWriter::new(
            SubxtWriteOps::new(api, FinalitySink),
            signatory,
            co_signatories.clone(),
            settings.contract_h160,
        );
        Ok(Self {
            service,
            reader: Box::new(chain_read::OnChainStateReader::new(registry.clone())),
            writer: Box::new(writer),
            signatory_reader: Box::new(signatory::OnChainSignatorySetReader::new(registry)),
            own_account: Some(own_account),
            co_signers: co_signatories,
        })
    }

    /// Built without `dev-seed`: no seed is read and nothing is connected
    /// (LLR-rgdx22); the service is kept with the refusal.
    #[cfg(not(feature = "dev-seed"))]
    pub async fn connect(self, _settings: ChainSettings) -> Result<Self, Box<ConnectFailed>> {
        Err(ConnectFailed::boxed(self.service, ConnectFailure::DevSeedNotBuilt))
    }

    /// Over a substitute chain: `chain` is the read, the write and the
    /// signatory-set reads. No signatory account: see `with_own_account`.
    #[cfg(feature = "test-support")]
    pub fn for_test(service: OrgService, chain: &test_support::FakeChain) -> Self {
        Self {
            service,
            reader: Box::new(chain.clone()),
            writer: Box::new(chain.clone()),
            signatory_reader: Box::new(chain.clone()),
            own_account: None,
            co_signers: Vec::new(),
        }
    }

    /// The account and co-signers the own-admin check uses, as a node holding
    /// that account's key would have them (test builds only).
    #[cfg(feature = "test-support")]
    pub fn with_own_account(mut self, own_account: AccountId, co_signers: Vec<AccountId>) -> Self {
        self.own_account = Some(own_account);
        self.co_signers = co_signers;
        self
    }

    /// Whether this node is an admin of `org_id` (LLR-c9fyun, REQ-f3eu9n):
    /// `NotConfigured` when it holds no signatory key, `Read` when a
    /// signatory-set read fails (never `NotAdmin`), otherwise the rule of
    /// `signatory::admin_status`. Nothing in S2 acts on the answer.
    pub async fn is_own_admin(&self, org_id: OrgId) -> Result<AdminStatus, OwnAdminError> {
        signatory::own_admin_status(&*self.signatory_reader, self.own_account, &self.co_signers, org_id).await
    }

    /// The read surface: an allowlist of org-node's queries, each returning
    /// public data only (`NodeView`; review round 3, finding 1).
    pub fn view(&self) -> NodeView<'_> {
        NodeView::new(&self.service)
    }

    /// The builders that judge nothing against the chain (`NodeBuilders`).
    pub fn node_mut(&mut self) -> NodeBuilders<'_> {
        NodeBuilders { service: &mut self.service }
    }

    /// org-node's whole service, read-only, for tests that inspect what the
    /// handle's surface does not offer (test builds only).
    #[cfg(feature = "test-support")]
    pub fn node_for_test(&self) -> &OrgService {
        &self.service
    }

    /// org-node's whole service, for tests that drive one of its
    /// chain-judging operations directly (test builds only).
    #[cfg(feature = "test-support")]
    pub fn node_mut_for_test(&mut self) -> &mut OrgService {
        &mut self.service
    }

    /// Found an Organisation: write, read back, commit (LLR-qhjp6g,
    /// LLR-be3zv9, LLR-9r2bxd).
    pub async fn found_organisation<R: RngCore + CryptoRng + Send>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
    ) -> Result<OrgId, String> {
        submit::found_organisation(&mut self.service, &*self.writer, &*self.reader, rng, persona_id).await
    }

    /// Submit the update `update` names, as org-node holds it: write, read
    /// back, commit, send to `recipient` (LLR-qhjp6g, LLR-be3zv9,
    /// LLR-9r2bxd).
    pub async fn submit_commit_send<R: RngCore + CryptoRng + Send>(
        &mut self,
        rng: &mut R,
        update: &BuiltUpdate,
        recipient: DevicePublicKey,
        peer_addr: Option<iroh::EndpointAddr>,
    ) -> Result<CommitOutcome, String> {
        submit::submit_commit_send(&mut self.service, &*self.writer, &*self.reader, rng, update, recipient, peer_addr)
            .await
    }

    /// Receive one message (LLR-7pj5af): its sender and the message go to
    /// org-node's chain-free phase; a refusal or a finished outcome (an
    /// acknowledgement) is returned with no read; a pending value is read
    /// for once, and only then applied.
    pub async fn receive_and_verify<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> Result<ReceiveOutcome, OrgNodeError> {
        // The sender is a value from here on (S3a's member-sender rule runs
        // in prepare).
        let (sender, message) = self.service.receive_message().await?;
        match self.service.prepare_receive(sender, message)? {
            // An acknowledgement: decided against the store alone, no read.
            Prepared::Done(outcome) => Ok(outcome),
            Prepared::NeedsChain(pending) => {
                let chain_state = self.reader.read_state(pending.org_id()).await?;
                self.service.apply_receive(rng, pending, chain_state)
            }
        }
    }

    /// `receive_and_verify`, on the self-delete path (LLR-7pj5af).
    pub async fn receive_and_self_delete_if_revoked<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
    ) -> Result<SelfDeleteOutcome, OrgNodeError> {
        let (sender, message) = self.service.receive_message().await?;
        match self.service.prepare_self_delete(sender, message)? {
            Prepared::Done(outcome) => Ok(outcome),
            Prepared::NeedsChain(pending) => {
                let chain_state = self.reader.read_state(pending.org_id()).await?;
                self.service.apply_self_delete(rng, pending, chain_state)
            }
        }
    }
}
