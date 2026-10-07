//! OrgService: composes store + chain + transport into the five user stories.
//! See spec §6 and plan docs/superpowers/plans/2026-06-16-ods-phase-2-4-tauri-shell.md Task 2.
//!
//! Gated on the `app` feature (which implies `chain` + `transport`).
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use org_members::delta::Delta;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::{DevicePublicKey, Handle, MemberId, MemberLeaf, Name, PersonPublicKey, RootHash, Surname};
use rand_core::{CryptoRng, RngCore};

use crate::chain::OrgState;
use crate::envelope::Envelope;
use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::keys::{SigningKeypair, X25519Keypair};
use crate::revocation::{Acknowledgement, RevocationNotice, VerifiedAcknowledgement};
use crate::sequence::SeqGuard;
use crate::store::{
    ExpectedAdmission, MemberSnapshot, OrgRecord, PersonaRecord, PersonaStatus, PersonaStore,
    ProvisionalChange, ProvisionalUpdate, RawMemberSnapshot, StoreData,
};
pub use crate::store::ProvisionalTarget;
use crate::transport::{TransportError, TransportMode};
use crate::transport::endpoint::OrgEndpoint;
use crate::transport::wire::WireMessage;
use crate::types::{ChainAccount, DeviceSeed, Epoch, OrgPrivateKey, OrgPublicKey, PersonaId, SequenceNumber};
use crate::verify::{VerifiedUpdate, VerifyContext, verify_envelope_against_chain};

type Trie = OrgTrie<Blake3Hasher>;

// ============================================================
// ChainOps trait — the read oracle injected into OrgService.
// ============================================================

/// Read-only access to the chain: org-node writes nothing to it (LLR-65py3d).
/// Production wires a real subxt client; tests inject `MockChainOps`.
#[async_trait]
pub trait ChainOps: Send + Sync {
    /// Read current on-chain state for `org_id`.
    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError>;
}

// ============================================================
// MockChainOps — in-memory stub for headless tests.
// ============================================================

/// Shared, thread-safe mock chain state.
#[derive(Default, Clone)]
pub struct MockChainInner {
    slots: std::collections::HashMap<OrgId, OrgState>,
    next_id_seed: u8,
}

/// Mock `ChainOps` backed by an `Arc<Mutex<MockChainInner>>`.  Clone the `Arc`
/// to share the same mock chain between the two `OrgService` instances in tests.
#[derive(Clone)]
pub struct MockChainOps {
    inner: Arc<Mutex<MockChainInner>>,
}

impl MockChainOps {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(MockChainInner::default())) }
    }

    /// Directly seed the chain (for test setup only).
    pub fn set(&self, org_id: OrgId, state: OrgState) {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.slots.insert(org_id, state);
    }

    /// Read a slot directly (for test assertions).
    pub fn get(&self, org_id: &OrgId) -> Option<OrgState> {
        let g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.slots.get(org_id).copied()
    }

    /// Stand-in for the genesis chain write the app makes through
    /// on-chain-client: a slot at epoch one holding `root` and `key`, under an
    /// id derived from the root and a counter (LLR-ryzr8m).
    pub fn apply_genesis(&self, root: RootHash, key: OrgPublicKey) -> OrgId {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let mut id_bytes = [0u8; 20];
        id_bytes.copy_from_slice(&root.as_bytes()[..20]);
        id_bytes[0] ^= g.next_id_seed;
        g.next_id_seed = g.next_id_seed.wrapping_add(1);
        let org_id = OrgId::new(id_bytes);
        g.slots.insert(org_id, OrgState { root_hash: root, org_pub_key: key, epoch: Epoch::new(1) });
        org_id
    }

    /// Stand-in for the update chain write the app makes through
    /// on-chain-client: `root` and `key` at `expected_epoch + 1`, refused
    /// unless the slot is at `expected_epoch` (LLR-ryzr8m).
    pub fn apply_update(
        &self,
        org_id: OrgId,
        root: RootHash,
        key: OrgPublicKey,
        expected_epoch: Epoch,
    ) -> Result<(), OrgNodeError> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let state = g.slots.get(&org_id).copied().ok_or(OrgNodeError::OrgNotOnChain)?;
        if state.epoch != expected_epoch {
            return Err(OrgNodeError::Chain(format!(
                "epoch mismatch: expected {}, found {}",
                expected_epoch.get(),
                state.epoch.get()
            )));
        }
        g.slots.insert(org_id, OrgState { root_hash: root, org_pub_key: key, epoch: Epoch::new(expected_epoch.get() + 1) });
        Ok(())
    }
}

impl Default for MockChainOps {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChainOps for MockChainOps {
    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        let g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        Ok(g.slots.get(&org_id).copied())
    }
}

// ============================================================
// SubxtChainOps — the chain read through the registry contract.
// ============================================================

#[cfg(feature = "chain")]
mod subxt_impl {
    use async_trait::async_trait;
    use on_chain_client::{OrgAdmin, OrgRegistryClient};

    use crate::chain::OrgState;
    use crate::error::OrgNodeError;
    use crate::ids::OrgId;

    /// Production `ChainOps`: reads an Organisation's state through the
    /// registry contract. It writes nothing (LLR-65py3d).
    pub struct SubxtChainOps {
        pub registry_client: OrgRegistryClient,
    }

    impl SubxtChainOps {
        pub fn new(registry_client: OrgRegistryClient) -> Self {
            Self { registry_client }
        }
    }

    #[async_trait]
    impl super::ChainOps for SubxtChainOps {
        async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
            let admin = OrgAdmin(*org_id.as_bytes());
            let state = self
                .registry_client
                .get_org_state(admin, None)
                .await
                .map_err(|e| OrgNodeError::Chain(format!("get_org_state: {e}")))?
                .map(crate::chain_read::org_state_from_chain)
                .transpose()?;
            Ok(state)
        }
    }
}

#[cfg(feature = "chain")]
pub use subxt_impl::SubxtChainOps;

/// Connect to a chain RPC over an explicit `LegacyBackend` (required for
/// chopsticks; mirrors tests/common/conn.rs) and build an `OrgRegistryClient`
/// for `contract`. Used by the preflight binary and by the app, which also
/// hands the `OnlineClient` to on-chain-client's writer; org-node itself only
/// reads. Returns both so callers can run raw-chain checks (the `OnlineClient`)
/// and contract checks (the `OrgRegistryClient`).
pub async fn connect_chain_client(
    ws_url: &str,
    contract: [u8; 20],
) -> Result<
    (
        subxt::OnlineClient<subxt::config::PolkadotConfig>,
        on_chain_client::OrgRegistryClient,
    ),
    crate::error::OrgNodeError,
> {
    use subxt::backend::LegacyBackend;
    use subxt::config::PolkadotConfig;
    use subxt::rpcs::client::ReconnectingRpcClient;

    // Use the reconnecting RPC client rather than `from_insecure_url`. Public RPC
    // nodes close idle WS connections; with a plain client the next call after an
    // idle period (e.g. the app's genesis write minutes after startup) fails
    // deep in subxt with "cannot get the current block: ... Error reason could not
    // be found. This is a bug." (a jsonrpsee dead-connection error). The
    // reconnecting client keeps the link alive with WS pings AND transparently
    // reconnects, so reads at startup and writes much later both succeed.
    let reconnecting = ReconnectingRpcClient::builder()
        .build(ws_url)
        .await
        .map_err(|e| crate::error::OrgNodeError::Chain(format!("rpc connect: {e}")))?;
    let rpc_client = subxt::rpcs::RpcClient::new(reconnecting);
    let backend: LegacyBackend<PolkadotConfig> = LegacyBackend::builder().build(rpc_client);
    let api = subxt::OnlineClient::from_backend(Arc::new(backend))
        .await
        .map_err(|e| crate::error::OrgNodeError::Chain(format!("online client: {e}")))?;
    let registry = on_chain_client::OrgRegistryClient::from_client(api.clone(), contract)
        .await
        .map_err(|e| crate::error::OrgNodeError::Chain(format!("registry client: {e}")))?;
    Ok((api, registry))
}

// ============================================================
// Helper: rebuild a trie mirror from persisted MemberSnapshots.
// ============================================================

fn trie_from_snapshots(snapshots: &[MemberSnapshot]) -> Result<Trie, OrgNodeError> {
    let leaves = snapshots
        .iter()
        .map(|s| {
            MemberLeaf::new(s.id, s.handle.clone(), s.member_key, s.name.clone(), s.surname.clone(), s.device_keys.clone())
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(OrgNodeError::Trie)?;
    Trie::genesis(leaves).map_err(OrgNodeError::Trie)
}

/// The persisted snapshot of a trie member.
fn snapshot_of(m: &MemberLeaf) -> MemberSnapshot {
    MemberSnapshot {
        id: *m.id(),
        handle: m.handle().clone(),
        name: m.name().clone(),
        surname: m.surname().clone(),
        member_key: *m.p2p_key(),
        device_keys: m.p2p_devices().to_vec(),
    }
}

/// The record a first admission extends, decoded from the snapshot its
/// Organisation-information message carries (REQ-d9g6nt, LLR-j6j95z).
// Public only for the fuzz target `fuzz_first_admission_base`; not API.
#[doc(hidden)]
pub fn first_admission_base(record_snapshot: &[u8]) -> Result<Trie, OrgNodeError> {
    let raw: Vec<RawMemberSnapshot> = postcard::from_bytes(record_snapshot)
        .map_err(|e| OrgNodeError::Chain(format!("record snapshot decode: {e}")))?;
    let snaps = raw.into_iter().map(MemberSnapshot::try_from).collect::<Result<Vec<_>, _>>()?;
    trie_from_snapshots(&snaps)
}

/// Encode the record a committed update extends, as an
/// Organisation-information message's `record_snapshot`;
/// `first_admission_base` decodes it.
fn encode_record_snapshot(snapshots: &[MemberSnapshot]) -> Result<Vec<u8>, OrgNodeError> {
    postcard::to_allocvec(snapshots)
        .map_err(|e| OrgNodeError::Chain(format!("record snapshot encode: {e}")))
}

/// The person an admission adds, as parsed values the app built from the
/// Invite reply it parsed (LLR-kkj64b).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Joiner {
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
    pub member_key: PersonPublicKey,
    pub device_key: DevicePublicKey,
}

// ============================================================
// OrgService — the composition root.
// ============================================================

/// Composes `PersonaStore` + `ChainOps` + lazy `OrgEndpoint` into the
/// five org-node user stories. Headless-testable: inject `MockChainOps` +
/// a real loopback `OrgEndpoint` to exercise the full composition without a
/// live chain.
pub struct OrgService {
    store: PersonaStore,
    chain: Box<dyn ChainOps>,
    /// Lazily bound device endpoint; `None` until `bind_endpoint` is called.
    endpoint: Option<OrgEndpoint>,
    /// How to bind the iroh endpoint and dial peers.
    /// Defaults to `Loopback` so all existing offline tests are unaffected.
    transport_mode: TransportMode,
}

impl OrgService {
    pub fn new(store: PersonaStore, chain: Box<dyn ChainOps>) -> Self {
        Self { store, chain, endpoint: None, transport_mode: TransportMode::Loopback }
    }

    /// Override the transport mode used when `ensure_endpoint` binds and when
    /// `send_update` dials peers.
    ///
    /// Must be called BEFORE any endpoint is bound (i.e. before the first
    /// `ensure_endpoint`, `receive_and_verify` or `send_update` call).  In production the Tauri `AppState::init` sets
    /// this to `TransportMode::Networked` so both laptops use relay + discovery.
    pub fn set_transport_mode(&mut self, mode: TransportMode) {
        self.transport_mode = mode;
    }

    /// Inject a pre-bound endpoint (used in tests to supply loopback endpoints).
    pub fn with_endpoint(mut self, ep: OrgEndpoint) -> Self {
        self.endpoint = Some(ep);
        self
    }

    /// Borrow the endpoint if one is set.
    pub fn endpoint(&self) -> Option<&OrgEndpoint> {
        self.endpoint.as_ref()
    }

    // ----------------------------------------------------------
    // Story precursor: create a persona (no chain, no network).
    // ----------------------------------------------------------

    /// Generate a new persona with a fresh X25519 member seed and ed25519
    /// device seed, persist it in the store, and return the persona_id.
    pub fn create_persona<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        handle: Handle,
        name: Name,
        surname: Surname,
    ) -> Result<PersonaId, OrgNodeError> {
        let member_kp = X25519Keypair::generate(rng);
        let device_kp = SigningKeypair::generate(rng);
        // Derive a unique persona_id from the Member-as-a-group key.
        let persona_id = persona_id_for(&member_kp.member_key()?);
        let rec = PersonaRecord {
            persona_id: persona_id.clone(),
            org_id: None,
            handle,
            name,
            surname,
            member_seed: member_kp.member_seed(),
            device_seed: device_kp.device_seed(),
            member_id: None,
            status: PersonaStatus::Proposed,
        };
        self.store.data_mut().personas.push(rec);
        self.store.save(rng)?;
        Ok(persona_id)
    }

    // ----------------------------------------------------------
    // Story 1: create_organisation builds the genesis update; commit_genesis
    // records it once the chain carries it.
    // ----------------------------------------------------------

    /// Build the genesis provisional update for `persona_id` and keep it: a
    /// fresh Organisation key pair whose private key the update holds, no
    /// chain operation, no record, no binding (LLR-s6qnht, LLR-qjz3q4,
    /// LLR-68yd3j, LLR-sj7cd5). A Persona already bound to an Organisation
    /// is refused (LLR-6z5xya).
    pub fn create_organisation<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        self.ensure_unbound(persona_id)?;
        let (member_kp, device_kp, handle, name, surname) = self.persona_keys(persona_id)?;
        let founder = MemberLeaf::new(
            fresh_member_id(rng),
            handle,
            member_kp.member_key()?,
            name,
            surname,
            vec![device_kp.device_key()?],
        )
        .map_err(OrgNodeError::Trie)?;
        let trie = Trie::genesis(vec![founder.clone()]).map_err(OrgNodeError::Trie)?;
        // REQ-ech45n: drawn for this Organisation alone, its public key equal
        // to no key of the genesis record (LLR-sj7cd5).
        let org_kp = X25519Keypair::generate(rng);
        let org_pub_key = org_kp.org_public_key()?;
        org_pub_key.ensure_distinct_from(&trie.members())?;
        let update = ProvisionalUpdate {
            org_id: None,
            persona_id: persona_id.clone(),
            base_root: None,
            resulting_root: trie.root_hash().map_err(OrgNodeError::Trie)?,
            // Genesis produces epoch 1 on the contract (Decision 16).
            seq: SequenceNumber::new(1),
            org_pub_key,
            change: ProvisionalChange::Genesis {
                members: vec![snapshot_of(&founder)],
                org_private_key: org_kp.org_private_key(),
            },
        };
        self.store.data_mut().insert_provisional(update.clone())?;
        self.store.save(rng)?;
        Ok(update)
    }

    /// Commit the genesis update `persona_id` built, once the chain carries
    /// its root and key at epoch 1: read the state once, select the update,
    /// require the epoch to be the update's Sequence number and the members
    /// to rebuild the root, then create the record with the chain's values,
    /// mark 1, the update's private key and `proxy_account`, consume the
    /// update and bind the Persona, which must be unbound, with the MemberId
    /// of the genesis leaf listing its Device (LLR-hby4jr) (LLR-wzqqg9,
    /// LLR-qjz3q4, LLR-w3fhhg, LLR-dzte8x, LLR-eyc4ud). Any refusal changes and writes nothing (LLR-ewkg85); no
    /// endpoint is bound (LLR-4tcxsu).
    pub async fn commit_genesis<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
        org_id: OrgId,
        proxy_account: ChainAccount,
    ) -> Result<ReceiveOutcome, OrgNodeError> {
        // One Persona, one Organisation (REQ-yp75u9, LLR-eyc4ud).
        self.ensure_unbound(persona_id)?;
        let state = self.chain.read_state(org_id).await?.ok_or(OrgNodeError::OrgNotOnChain)?;
        let update = self
            .genesis_updates(persona_id)
            .find(|u| u.resulting_root == state.root_hash && u.org_pub_key == state.org_pub_key)
            .cloned()
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        // A chain already past epoch 1 — another signatory updated the
        // Organisation first — is refused too: the record then comes from an
        // update another Member sends (owner ruling 2026-10-06).
        if state.epoch.get() != update.seq.get() {
            return Err(OrgNodeError::SeqNotEpoch { seq: update.seq.get(), epoch: state.epoch.get() });
        }
        let ProvisionalChange::Genesis { members, org_private_key } = update.change.clone() else {
            return Err(OrgNodeError::NoProvisionalUpdate);
        };
        if trie_from_snapshots(&members)?.root_hash().map_err(OrgNodeError::Trie)? != state.root_hash {
            return Err(OrgNodeError::RootMismatch);
        }
        // The founder's MemberId is the genesis leaf listing this Persona's
        // Device; the Persona is bound with it (LLR-hby4jr, PR-eqs4fs), and
        // an update with no such leaf is refused (LLR-wzqqg9).
        // `create_organisation` builds that leaf from this Persona, so a
        // genesis update without it is, like a non-genesis change above, no
        // update this Persona can commit.
        let (_, device) = self.persona_public_keys(persona_id)?;
        let founder_id = members
            .iter()
            .find(|member| member.device_keys.contains(&device))
            .map(|member| member.id)
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        let data = self.store.data_mut();
        data.orgs.push(OrgRecord {
            org_id,
            root_hash: state.root_hash,
            org_pub_key: state.org_pub_key,
            epoch: state.epoch,
            last_seq: update.seq,
            trie_members: members,
            proxy_account: Some(proxy_account),
            org_private_key,
            kept_change_set: None,
        });
        data.provisional_updates.retain(|u| *u != update);
        Self::discard_orphans(data, org_id, state.root_hash);
        self.bind_persona(persona_id, org_id, founder_id)?;
        self.store.save(rng)?;
        Ok(ReceiveOutcome { org_id, epoch: state.epoch, root: state.root_hash, acknowledgements: Vec::new(), acknowledged: None })
    }

    /// After a commit to `org_id` at `root`, drop every provisional update for
    /// that Organisation built on another base (LLR-mkj4bz).
    fn discard_orphans(data: &mut StoreData, org_id: OrgId, root: RootHash) {
        data.provisional_updates.retain(|u| u.org_id != Some(org_id) || u.base_root == Some(root));
    }

    /// Remove the one provisional update for `target` whose resulting root
    /// is `resulting_root` and whose Organisation public key is
    /// `org_pub_key`, with the Organisation private key it holds, and save;
    /// refuse one not stored with `NoProvisionalUpdate`, changing and writing
    /// nothing (LLR-7cmp38).
    pub fn discard_provisional<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        target: ProvisionalTarget,
        resulting_root: RootHash,
        org_pub_key: OrgPublicKey,
    ) -> Result<(), OrgNodeError> {
        let updates = &mut self.store.data_mut().provisional_updates;
        let position = updates
            .iter()
            .position(|u| u.resulting_root == resulting_root && u.org_pub_key == org_pub_key && target.names(u))
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        updates.remove(position);
        self.store.save(rng)
    }

    /// The stored provisional updates for `org_id` (LLR-nvn3wk).
    pub fn provisional_updates(&self, org_id: OrgId) -> Vec<ProvisionalUpdate> {
        self.store.data().provisional_updates.iter().filter(|u| u.org_id == Some(org_id)).cloned().collect()
    }

    /// The genesis updates `persona_id` built (LLR-nvn3wk).
    pub fn genesis_provisional_updates(&self, persona_id: &PersonaId) -> Vec<ProvisionalUpdate> {
        self.genesis_updates(persona_id).cloned().collect()
    }

    fn genesis_updates<'a>(&'a self, persona_id: &PersonaId) -> impl Iterator<Item = &'a ProvisionalUpdate> + 'a {
        let target = ProvisionalTarget::Genesis(persona_id.clone());
        self.store.data().provisional_updates.iter().filter(move |u| target.names(u))
    }

    /// The proxy account the record holds, unchanged and uninterpreted
    /// (LLR-3v5nu9).
    pub fn proxy_account(&self, org_id: OrgId) -> Result<Option<ChainAccount>, OrgNodeError> {
        Ok(self.find_org(org_id)?.proxy_account)
    }

    /// The Persona's Member-as-a-group key and DevicePublicKey, each from its
    /// own seed, and no dialling address (LLR-437fvx).
    pub fn persona_public_keys(&self, persona_id: &PersonaId) -> Result<(PersonPublicKey, DevicePublicKey), OrgNodeError> {
        let p = self.find_persona(persona_id)?;
        Ok((p.member_seed.x25519_keypair().member_key()?, p.device_seed.signing_keypair().device_key()?))
    }

    // ----------------------------------------------------------
    // Story 3: admit_member builds the admission; commit_update records it
    // once the chain carries it; send_update sends the committed update.
    // ----------------------------------------------------------

    /// Build the admission of `joiner` into `org_id` and keep it: no chain
    /// operation, no endpoint, no send, the record unchanged (LLR-rb8r65,
    /// LLR-vdyu65). The new Member's id is drawn, so a re-admission with the
    /// same keys gets a new one (REQ-d9g6nt).
    pub fn admit_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        joiner: &Joiner,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let rec = self.find_org(org_id)?.clone();
        let persona_id = self.first_persona_bound_to(org_id)?.persona_id.clone();
        let leaf = MemberLeaf::new(
            fresh_member_id(rng),
            joiner.handle.clone(),
            joiner.member_key,
            joiner.name.clone(),
            joiner.surname.clone(),
            vec![joiner.device_key],
        )
        .map_err(OrgNodeError::Trie)?;
        let (new_trie, delta) = trie_from_snapshots(&rec.trie_members)?
            .add_member(leaf)
            .map_err(OrgNodeError::Trie)?
            .recalculate()
            .map_err(OrgNodeError::Trie)?;
        self.keep_change_set(rng, &rec, persona_id, &new_trie, &delta)
    }

    /// Keep a Change set built on `rec` as a provisional update: its Sequence
    /// number is the epoch it produces, the record's plus one (LLR-ghja3x,
    /// REQ-txvtm9, Decision 16). Its root calculated, the batch ends and a
    /// fresh Organisation key pair is drawn for it (REQ-stx9v3, LLR-e2b7gv):
    /// distinct from the record's current key and from every key of the
    /// resulting record, else `DuplicateKey`, nothing kept or written. The
    /// update holds the private key; the record's keys are not changed. A
    /// refusal by the bound writes nothing (LLR-jq7qh7).
    fn keep_change_set<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        rec: &OrgRecord,
        persona_id: PersonaId,
        new_trie: &Trie,
        delta: &Delta,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let seq = SequenceNumber::new(rec.epoch.get() + 1);
        let resulting_root = new_trie.root_hash().map_err(OrgNodeError::Trie)?;
        let org_kp = X25519Keypair::generate(rng);
        let org_pub_key = org_kp.org_public_key()?;
        if org_pub_key == rec.org_pub_key {
            return Err(OrgNodeError::Trie(org_members::OrgMembersError::DuplicateKey));
        }
        org_pub_key.ensure_distinct_from(&new_trie.members())?;
        let update = ProvisionalUpdate {
            org_id: Some(rec.org_id),
            persona_id,
            base_root: Some(rec.root_hash),
            resulting_root,
            seq,
            org_pub_key,
            change: ProvisionalChange::ChangeSet {
                change_set: Envelope::build(rec.org_id, seq, delta)?.delta_bytes,
                org_private_key: org_kp.org_private_key(),
            },
        };
        self.store.data_mut().insert_provisional(update.clone())?;
        self.store.save(rng)?;
        Ok(update)
    }

    /// Commit the node's own provisional update for `org_id` once the chain
    /// carries its root and key, by the checks a received update passes,
    /// taking the update's key pair into the record (LLR-cmdrp9, LLR-6s785x).
    /// A refusal changes and writes nothing (LLR-ewkg85); nothing is bound or
    /// sent (LLR-4tcxsu). A commit that removes every Persona bound to the
    /// Organisation forgets it instead, through the one removal step, and
    /// returns the acknowledgements it signed (LLR-b27jr6, LLR-23sfdh).
    pub async fn commit_update<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
    ) -> Result<CommitOutcome, OrgNodeError> {
        self.find_org(org_id)?;
        let state = self.chain.read_state(org_id).await?.ok_or(OrgNodeError::OrgNotOnChain)?;
        let update = self
            .store
            .data()
            .held_update_for(org_id, &state)
            .cloned()
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        let data = self.store.data();
        let committed = commit_step(data, org_id, &update, &state, || device_seeds_bound_to(data, org_id))?;
        *self.store.data_mut() = committed.store;
        self.store.save(rng)?;
        Ok(committed.outcome)
    }

    /// Write a verified update to a held record — root, epoch, mark and
    /// members together (LLR-cja9zv), and the Organisation key pair of the
    /// update, replacing the one it held, no earlier one kept (LLR-6s785x,
    /// LLR-ckk5nz, LLR-4kh9w9) — keep the verified Envelope's `change_set`,
    /// replacing any earlier one (LLR-d9778a), and drop the provisional
    /// updates it orphans (LLR-mkj4bz).
    fn commit_held(
        data: &mut StoreData,
        org_id: OrgId,
        verified: &VerifiedUpdate,
        change_set: &[u8],
        org_private_key: OrgPrivateKey,
        org_pub_key: OrgPublicKey,
    ) -> Result<(), OrgNodeError> {
        let root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;
        let snapshots: Vec<MemberSnapshot> = verified.trie.members().iter().map(snapshot_of).collect();
        let rec = data.orgs.iter_mut().find(|o| o.org_id == org_id).ok_or(OrgNodeError::OrgNotOnChain)?;
        rec.root_hash = root;
        rec.epoch = verified.epoch;
        rec.last_seq = verified.seq_guard.last_seen();
        rec.trie_members = snapshots;
        rec.org_private_key = org_private_key;
        rec.org_pub_key = org_pub_key;
        rec.kept_change_set = Some(change_set.to_vec());
        Self::discard_orphans(data, org_id, root);
        Ok(())
    }

    /// Commit received Organisation information that leaves this node listed
    /// in its held record (LLR-jsx922); a revocation never reaches this, it
    /// is decided by `revocation::accept` (LLR-pt32fx). The record takes
    /// the key the message carried and the chain's public key, a pair
    /// (REQ-ju6vn2, LLR-ckk5nz, LLR-4kh9w9), and keeps the received
    /// Envelope's Change set (LLR-d9778a).
    fn commit_received(
        &mut self,
        org_id: OrgId,
        verified: &VerifiedUpdate,
        envelope: &Envelope,
        carried_key: OrgPrivateKey,
        chain_state: &OrgState,
    ) -> Result<(), OrgNodeError> {
        Self::commit_held(self.store.data_mut(), org_id, verified, &envelope.delta_bytes, carried_key, chain_state.org_pub_key)
    }

    /// Refuse an Organisation private key whose X25519 public half is not the
    /// Organisation public key the chain state read carries (LLR-ba2ejp,
    /// RC-9cefcn).
    fn check_carried_key(org_id: OrgId, key: &OrgPrivateKey, state: &OrgState) -> Result<(), OrgNodeError> {
        if key.x25519_keypair().org_public_key()? == state.org_pub_key {
            Ok(())
        } else {
            Err(OrgNodeError::OrgKeyMismatch { org_id })
        }
    }

    /// A receive path's removal: the one removal step against `chain`, its
    /// successor adopted (LLR-b27jr6, LLR-23sfdh). A signing refusal
    /// changes nothing. The caller saves.
    fn remove_self(&mut self, org_id: OrgId, chain: &OrgState) -> Result<Vec<Acknowledgement>, OrgNodeError> {
        let data = self.store.data();
        let (successor, acknowledgements) = removal_step(data, org_id, chain, device_seeds_bound_to(data, org_id))?;
        *self.store.data_mut() = successor;
        Ok(acknowledgements)
    }

    /// Decide a received revocation notice (LLR-pt32fx): `check_notice`
    /// first, with no chain read (LLR-38e2kn, LLR-r7zm39); then `sender`
    /// must be listed in the record (LLR-kzgjz8), still with no chain read;
    /// then the one chain read and `revocation::accept` (LLR-tx8ruv). A
    /// refusal writes nothing. On acceptance the successor store is adopted
    /// and saved; returns the state the notice was accepted against and the
    /// acknowledgements.
    async fn receive_revocation<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        sender: &DevicePublicKey,
        notice: &RevocationNotice,
    ) -> Result<(OrgState, Vec<Acknowledgement>), OrgNodeError> {
        crate::revocation::check_notice(self.store.data(), notice)?;
        sender_listed(self.find_org(notice.org_id)?, notice.org_id, sender)?;
        let chain = self.chain.read_state(notice.org_id).await?;
        let data = self.store.data();
        let accepted = crate::revocation::accept(data, notice, chain, || device_seeds_bound_to(data, notice.org_id))?;
        // `accept` succeeds only with a chain state.
        let state = chain.ok_or(OrgNodeError::OrgNotOnChain)?;
        *self.store.data_mut() = accepted.store;
        self.store.save(rng)?;
        Ok((state, accepted.acknowledgements))
    }

    /// Send a committed update to `recipient`'s device, from the device of
    /// the first Persona bound to its Organisation (LLR-2xzys9). The kind
    /// follows the recipient (LLR-6ymd6d): a Device the node's record of the
    /// Organisation lists receives Organisation information — the Envelope,
    /// the record as it stood before the commit, and the Organisation
    /// private key that record holds, none taken from the caller
    /// (REQ-szq3ud); a Device the outcome holds a notice for receives that
    /// notice alone (REQ-3dsweu, LLR-8hdu9x); any other Device is refused
    /// with `NoRevocationForRecipient`, nothing sent. No invite identifier (LLR-48jakr). No
    /// record: `OrgNotOnChain`, nothing sent. Loopback dials `peer_addr` and
    /// refuses without one, before binding (LLR-jn5jeh, LLR-pw369n);
    /// Networked dials by `recipient`. Writes nothing (LLR-t4znbk).
    pub async fn send_update(
        &mut self,
        outcome: &CommitOutcome,
        recipient: DevicePublicKey,
        peer_addr: Option<iroh::EndpointAddr>,
    ) -> Result<(), OrgNodeError> {
        let outgoing = &outcome.outgoing;
        let mode = self.transport_mode;
        let rec = self.find_org(outgoing.envelope.org_id)?;
        let msg = if rec.trie_members.iter().any(|m| m.device_keys.contains(&recipient)) {
            WireMessage::OrgInformation {
                envelope: outgoing.envelope.clone(),
                record_snapshot: outgoing.record_snapshot.clone(),
                org_private_key: rec.org_private_key.clone(),
            }
        } else if let Some(notice) = outcome.revocations.iter().find(|notice| notice.device == recipient) {
            WireMessage::Revocation(notice.clone())
        } else {
            return Err(OrgNodeError::NoRevocationForRecipient { org_id: outgoing.envelope.org_id });
        };
        let persona_id = self.first_persona_bound_to(outgoing.envelope.org_id)?.persona_id.clone();
        let loopback_addr = match (mode, peer_addr) {
            (TransportMode::Loopback, None) => {
                return Err(OrgNodeError::Chain("Loopback send requires the peer's EndpointAddr".into()));
            }
            (TransportMode::Loopback, Some(addr)) => Some(addr),
            (TransportMode::Networked, _) => None,
        };
        let ep = self.ensure_endpoint(&persona_id).await?;
        match loopback_addr {
            Some(addr) => ep.send(addr, &msg).await.map_err(|e| OrgNodeError::Chain(format!("iroh send: {e}"))),
            None => {
                // Cross-network: dial purely by EndpointId so iroh relay/DNS
                // resolves the path; the device key is the EndpointId.
                let peer = iroh::EndpointId::from_bytes(recipient.as_bytes())
                    .map_err(|_| OrgNodeError::Chain("invalid recipient device key for EndpointId".into()))?;
                ep.send_to_id(peer, &msg).await.map_err(|e| OrgNodeError::Chain(format!("iroh send (networked): {e}")))
            }
        }
    }

    // ----------------------------------------------------------
    // Story 4: receive_and_verify — recv envelope, verify, commit.
    // ----------------------------------------------------------

    /// Accept one inbound `WireMessage`, verify its envelope against the
    /// chain, and commit the new state. An update for a held Organisation,
    /// and a revocation, are acted on only from a Device the record lists
    /// (LLR-2r2fha, LLR-kzgjz8); an acknowledgement only from the Device it
    /// names (LLR-3aysup); a first admission from any sender (REQ-xa6smf).
    pub async fn receive_and_verify<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
    ) -> Result<ReceiveOutcome, OrgNodeError> {
        // What the message is (LLR-js9dsu). A revocation is decided by its
        // proof — an unheld Organisation refused before the expectations are
        // consulted or the chain is read (LLR-38e2kn) — and an
        // acknowledgement by the store alone (LLR-pt32fx).
        let (sender, message) = self.receive_one().await?;
        let (envelope, record_snapshot, carried_key) = match message {
            WireMessage::OrgInformation { envelope, record_snapshot, org_private_key } => {
                (envelope, record_snapshot, org_private_key)
            }
            WireMessage::Revocation(notice) => {
                let (state, acknowledgements) = self.receive_revocation(rng, &sender, &notice).await?;
                return Ok(ReceiveOutcome {
                    org_id: notice.org_id,
                    epoch: state.epoch,
                    root: state.root_hash,
                    acknowledgements,
                    acknowledged: None,
                });
            }
            WireMessage::Acknowledgement(ack) => {
                let org_id = ack.org_id;
                let verified = crate::revocation::check_acknowledgement(self.store.data(), sender, ack)?;
                let rec = self.find_org(org_id)?;
                return Ok(ReceiveOutcome {
                    org_id,
                    epoch: rec.epoch,
                    root: rec.root_hash,
                    acknowledgements: Vec::new(),
                    acknowledged: Some(verified),
                });
            }
        };
        let org_id = envelope.org_id;

        // The record this message extends. No Invite is required (REQ-xa6smf,
        // owner ruling 2026-10-05). For a held Organisation the sender must
        // be listed in the record, before anything is decoded or the chain
        // is read (LLR-2r2fha); a first admission is accepted from any
        // sender (owner amendment of 2026-10-07).
        let existing = self.store.data().orgs.iter().find(|o| o.org_id == org_id).cloned();
        if let Some(record) = &existing {
            sender_listed(record, org_id, &sender)?;
        }
        let is_first_admission = existing.is_none();
        // A first admission is read only when the app expects one to this
        // Organisation — before its snapshot is decoded or the chain is read
        // (LLR-s8xp7m, RC-2ferct).
        let expectation = ExpectedAdmission { org_id };
        if is_first_admission && !self.store.data().expected_admissions.contains(&expectation) {
            return Err(OrgNodeError::AdmissionNotExpected { org_id });
        }
        let (local_trie, last_seq, last_epoch) = match &existing {
            Some(rec) => (trie_from_snapshots(&rec.trie_members)?, rec.last_seq, rec.epoch),
            // The record a first admission extends: from the snapshot its
            // Organisation-information message carries (REQ-d9g6nt,
            // LLR-j6j95z).
            None => (first_admission_base(&record_snapshot)?, SequenceNumber::new(0), Epoch::new(0)),
        };
        let ctx = VerifyContext {
            expected_org_id: org_id,
            seq_guard: SeqGuard::from_last_seen(last_seq),
            last_committed_epoch: last_epoch,
        };
        // What the Change set must reach (RC-6a2dke, RC-e5atck).
        let (verified, chain_state) = self.verify_received(&local_trie, &envelope, &ctx).await?;
        // The key Organisation information carries must be the private half
        // of the chain's key (LLR-ba2ejp, RC-9cefcn) — checked before the
        // own-Persona rule and before anything is written.
        Self::check_carried_key(org_id, &carried_key, &chain_state)?;
        let members = verified.trie.members();

        let new_root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;

        // This node's persona: the first whose DevicePublicKey is in the new
        // record, whatever its member key (LLR-e5c9ud), among those this
        // commit may bind — on a first admission the unbound ones, otherwise
        // those bound to this Organisation (LLR-eyc4ud, REQ-yp75u9).
        let eligible = if is_first_admission { None } else { Some(org_id) };
        let my_member = self.store.data().personas.iter().filter(|p| p.org_id == eligible).find_map(|p| {
            let dk = crate::revocation::persona_device(p)?;
            members
                .iter()
                .find(|m| m.has_p2p_device(&dk))
                .map(|m| (p.persona_id.clone(), *m.id()))
        });

        // A first admission commits only if it lists one of our Personas
        // (LLR-3f5h7b, REQ-kt877x); the refusal writes nothing.
        if is_first_admission && my_member.is_none() {
            return Err(OrgNodeError::AdmissionNotOurs { org_id });
        }

        if is_first_admission {
            let data = self.store.data_mut();
            data.orgs.push(OrgRecord {
                org_id,
                root_hash: new_root,
                // The chain's key, never a value from the Wire message
                // (LLR-xq9nrq).
                org_pub_key: chain_state.org_pub_key,
                epoch: verified.epoch,
                last_seq: verified.seq_guard.last_seen(),
                trie_members: members.iter().map(snapshot_of).collect(),
                // A first-admission record: only `commit_genesis` keeps a
                // proxy account (LLR-3v5nu9).
                proxy_account: None,
                org_private_key: carried_key,
                // The Change set of the update that admitted this node
                // (REQ-uv3v5w, LLR-d9778a as amended 2026-10-07).
                kept_change_set: Some(envelope.delta_bytes.clone()),
            });
            // Clear the expectation the committed first admission matched,
            // and only that one (LLR-q8emds).
            data.expected_admissions.retain(|e| *e != expectation);
            Self::discard_orphans(data, org_id, new_root);
        } else if still_member(self.store.data(), org_id, &verified.trie) {
            self.commit_received(org_id, &verified, &envelope, carried_key, &chain_state)?;
        } else {
            // The node's own removal, on this path as on every other, through
            // the one removal step (LLR-b27jr6, LLR-23sfdh): nothing is
            // committed, nothing bound.
            let acknowledgements = self.remove_self(org_id, &chain_state)?;
            self.store.save(rng)?;
            return Ok(ReceiveOutcome { org_id, epoch: verified.epoch, root: new_root, acknowledgements, acknowledged: None });
        }

        if let Some((persona_id, member_id)) = my_member {
            self.bind_persona(&persona_id, org_id, member_id)?;
        }

        self.store.save(rng)?;

        Ok(ReceiveOutcome { org_id, epoch: verified.epoch, root: new_root, acknowledgements: Vec::new(), acknowledged: None })
    }

    // ----------------------------------------------------------
    // Story 5: revoke_member builds the removal; commit_update records it
    // once the chain carries it; send_update sends the committed update.
    // ----------------------------------------------------------

    /// Build the removal of `member_id` from `org_id` and keep it: no chain
    /// operation, no endpoint, no send, the record unchanged (LLR-6dc598).
    pub fn revoke_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        member_id: MemberId,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let rec = self.find_org(org_id)?.clone();
        let persona_id = self.first_persona_bound_to(org_id)?.persona_id.clone();
        let (new_trie, delta) = trie_from_snapshots(&rec.trie_members)?
            .delete_member(&member_id)
            .map_err(OrgNodeError::Trie)?
            .recalculate()
            .map_err(OrgNodeError::Trie)?;
        self.keep_change_set(rng, &rec, persona_id, &new_trie, &delta)
    }

    /// Receive one message and act on it (LLR-pt32fx): a revocation notice
    /// accepted against the chain makes this node sign its acknowledgements
    /// and forget the Organisation; a received acknowledgement is checked
    /// against the store; Organisation information is verified against the
    /// chain and committed, or — when the committed record no longer lists
    /// this node — removes it through the one removal step (LLR-b27jr6).
    pub async fn receive_and_self_delete_if_revoked<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
    ) -> Result<SelfDeleteOutcome, OrgNodeError> {
        let (sender, message) = self.receive_one().await?;
        let (envelope, carried_key) = match message {
            WireMessage::OrgInformation { envelope, org_private_key, .. } => (envelope, org_private_key),
            WireMessage::Revocation(notice) => {
                let (_state, acknowledgements) = self.receive_revocation(rng, &sender, &notice).await?;
                return Ok(SelfDeleteOutcome::SelfDeleted { org_id: notice.org_id, acknowledgements });
            }
            WireMessage::Acknowledgement(ack) => {
                let verified = crate::revocation::check_acknowledgement(self.store.data(), sender, ack)?;
                return Ok(SelfDeleteOutcome::Acknowledged(verified));
            }
        };
        let org_id = envelope.org_id;
        // An Organisation not held is refused before any chain read (LLR-379hnv).
        let existing = self
            .store
            .data()
            .orgs
            .iter()
            .find(|o| o.org_id == org_id)
            .cloned()
            .ok_or(OrgNodeError::OrgNotOnChain)?;
        // Then the sender must be listed in the record, before anything is
        // decoded or the chain is read (LLR-2r2fha).
        sender_listed(&existing, org_id, &sender)?;
        let local_trie = trie_from_snapshots(&existing.trie_members)?;
        let ctx = VerifyContext {
            expected_org_id: org_id,
            seq_guard: SeqGuard::from_last_seen(existing.last_seq),
            last_committed_epoch: existing.epoch,
        };
        let (verified, chain_state) = self.verify_received(&local_trie, &envelope, &ctx).await?;
        // The receipt check, before any commit or record deletion (LLR-ba2ejp).
        Self::check_carried_key(org_id, &carried_key, &chain_state)?;
        if still_member(self.store.data(), org_id, &verified.trie) {
            // An ordinary update.
            self.commit_received(org_id, &verified, &envelope, carried_key, &chain_state)?;
            self.store.save(rng)?;
            return Ok(SelfDeleteOutcome::UpdatedNotRevoked { org_id });
        }

        // We are removed: the one removal step (LLR-b27jr6, LLR-23sfdh).
        let acknowledgements = self.remove_self(org_id, &chain_state)?;
        self.store.save(rng)?;

        Ok(SelfDeleteOutcome::SelfDeleted { org_id, acknowledgements })
    }

    /// Receive one message on the endpoint of the first Persona, binding it
    /// if not yet bound. Returns the sending Device the transport
    /// authenticated with the message; the receive paths check it against
    /// the record (LLR-2r2fha, LLR-kzgjz8) or, for an acknowledgement,
    /// against the Device it names (LLR-3aysup).
    async fn receive_one(&mut self) -> Result<(DevicePublicKey, WireMessage), OrgNodeError> {
        let first_persona_id = self
            .store
            .data()
            .personas
            .first()
            .ok_or_else(|| OrgNodeError::Chain("no persona found — create one first".into()))?
            .persona_id
            .clone();
        let ep = self.ensure_endpoint(&first_persona_id).await?;
        // A body that does not decode — Organisation information without its
        // snapshot or key among them — is refused as such, before any record,
        // expectation or chain is consulted (LLR-xn5pwc, REQ-c29s93).
        ep.recv_one().await.map_err(|e| match e {
            TransportError::Malformed => OrgNodeError::MalformedMessage,
            other => OrgNodeError::Chain(format!("iroh recv: {other}")),
        })
    }

    /// Verify a received Envelope against `local_trie`: every check that
    /// needs no chain, then the one chain read (LLR-9ew26y). Returns the
    /// verified update and the state read.
    async fn verify_received(
        &self,
        local_trie: &Trie,
        envelope: &Envelope,
        ctx: &VerifyContext,
    ) -> Result<(VerifiedUpdate, OrgState), OrgNodeError> {
        crate::verify::check_chain_free(local_trie, envelope, ctx)?;
        let state = self.chain.read_state(envelope.org_id).await?.ok_or(OrgNodeError::OrgNotOnChain)?;
        let verified = verify_envelope_against_chain(local_trie, envelope, ctx, &ChainOpsReader { state })?;
        Ok((verified, state))
    }

    // ----------------------------------------------------------
    // Query helpers.
    // ----------------------------------------------------------

    pub fn list_personas(&self) -> &[PersonaRecord] {
        &self.store.data().personas
    }

    pub fn list_orgs(&self) -> &[OrgRecord] {
        &self.store.data().orgs
    }

    /// The store data, read only, for tests that call the pure functions.
    #[cfg(feature = "test-support")]
    pub fn store_data(&self) -> &StoreData {
        self.store.data()
    }

    /// Reconcile `org_id`'s record with the chain: refuse an unheld
    /// Organisation with `OrgNotHeld` before any chain read, then read the
    /// state once, call
    /// `reconcile::reconcile`, and adopt and save a successor (LLR-gr8x3r,
    /// LLR-fm38ww). The interim caller until org-io's startup reconcile.
    pub async fn reconcile<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
    ) -> Result<crate::reconcile::Reconciled, OrgNodeError> {
        use crate::reconcile::Reconciled;
        // Held check first: an unheld Organisation costs no chain read.
        if !self.store.data().orgs.iter().any(|record| record.org_id == org_id) {
            return Err(OrgNodeError::OrgNotHeld { org_id });
        }
        let state = self.chain.read_state(org_id).await?.ok_or(OrgNodeError::OrgNotOnChain)?;
        let data = self.store.data();
        let reconciled = crate::reconcile::reconcile(data, org_id, state, || device_seeds_bound_to(data, org_id))?;
        if let Reconciled::Committed { store, .. } | Reconciled::Removed { store, .. } = &reconciled {
            *self.store.data_mut() = store.clone();
            self.store.save(rng)?;
        }
        Ok(reconciled)
    }

    /// Record that the app expects a first admission to `org_id`, at most
    /// once, and save before returning (LLR-9zfnmb).
    pub fn expect_admission<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
    ) -> Result<(), OrgNodeError> {
        let expectation = ExpectedAdmission { org_id };
        let expected = &mut self.store.data_mut().expected_admissions;
        if !expected.contains(&expectation) {
            expected.push(expectation);
        }
        self.store.save(rng)
    }

    /// The first admissions the app declared it expects (read only).
    pub fn expected_admissions(&self) -> &[ExpectedAdmission] {
        &self.store.data().expected_admissions
    }

    // ----------------------------------------------------------
    // Endpoint management — Gap 1 fix.
    // ----------------------------------------------------------

    /// Ensure a device endpoint is bound for `persona_id`.
    ///
    /// If an endpoint is already stored on this `OrgService`, it is reused as-is
    /// (one active device per instance for the PoC).  Otherwise, a new endpoint
    /// is bound from the persona's `device_seed` via `DeviceSeed::signing_keypair`
    /// using the stored [`TransportMode`] (default `Loopback`, overridable via
    /// [`set_transport_mode`]).
    ///
    /// Returns a shared reference to the bound endpoint.
    ///
    /// [`set_transport_mode`]: OrgService::set_transport_mode
    pub async fn ensure_endpoint(
        &mut self,
        persona_id: &PersonaId,
    ) -> Result<&OrgEndpoint, OrgNodeError> {
        if self.endpoint.is_none() {
            let device_kp = self.find_persona(persona_id)?.device_seed.signing_keypair();
            let ep = OrgEndpoint::bind_with_mode(&device_kp, self.transport_mode)
                .await
                .map_err(|e| OrgNodeError::Chain(format!("endpoint bind: {e}")))?;
            self.endpoint = Some(ep);
        }
        // SAFETY: we just set it above if it was None.
        self.endpoint
            .as_ref()
            .ok_or_else(|| OrgNodeError::Chain("endpoint bind failed unexpectedly".into()))
    }

    // ----------------------------------------------------------
    // Private helpers.
    // ----------------------------------------------------------

    fn find_persona(&self, persona_id: &PersonaId) -> Result<&PersonaRecord, OrgNodeError> {
        self.store
            .data()
            .personas
            .iter()
            .find(|p| &p.persona_id == persona_id)
            .ok_or_else(|| OrgNodeError::Chain(format!("persona not found: {}", persona_id.as_str())))
    }

    fn find_org(&self, org_id: OrgId) -> Result<&OrgRecord, OrgNodeError> {
        self.store
            .data()
            .orgs
            .iter()
            .find(|o| o.org_id == org_id)
            .ok_or(OrgNodeError::OrgNotOnChain)
    }

    /// The first Persona in store order bound to `org_id`, whatever its
    /// status (LLR-2xzys9, Decision 6).
    fn first_persona_bound_to(&self, org_id: OrgId) -> Result<&PersonaRecord, OrgNodeError> {
        self.store
            .data()
            .personas
            .iter()
            .find(|p| p.org_id == Some(org_id))
            .ok_or_else(|| OrgNodeError::Chain(format!("no persona bound to organisation {org_id:?}")))
    }

    /// Refuse a Persona already bound to an Organisation (LLR-6z5xya,
    /// LLR-eyc4ud, REQ-yp75u9).
    fn ensure_unbound(&self, persona_id: &PersonaId) -> Result<(), OrgNodeError> {
        match self.find_persona(persona_id)?.org_id {
            Some(_) => Err(OrgNodeError::PersonaAlreadyBound { persona_id: persona_id.clone() }),
            None => Ok(()),
        }
    }

    fn persona_keys(
        &self,
        persona_id: &PersonaId,
    ) -> Result<(X25519Keypair, SigningKeypair, Handle, Name, Surname), OrgNodeError> {
        let p = self.find_persona(persona_id)?;
        Ok((
            p.member_seed.x25519_keypair(),
            p.device_seed.signing_keypair(),
            p.handle.clone(),
            p.name.clone(),
            p.surname.clone(),
        ))
    }

    /// Bind `persona_id` to `org_id` as `member_id`, Active.
    fn bind_persona(&mut self, persona_id: &PersonaId, org_id: OrgId, member_id: MemberId) -> Result<(), OrgNodeError> {
        self.store
            .data_mut()
            .personas
            .iter_mut()
            .find(|p| &p.persona_id == persona_id)
            .ok_or_else(|| OrgNodeError::Chain(format!("persona not found: {}", persona_id.as_str())))
            .map(|p| {
                p.status = PersonaStatus::Active;
                p.org_id = Some(org_id);
                p.member_id = Some(member_id);
            })
    }
}

// ============================================================
// Outcomes.
// ============================================================

/// Outcome of `receive_and_verify` (LLR-pt32fx): the Organisation, the epoch
/// and root of the chain state it was decided against (of the held record
/// for an acknowledgement), the acknowledgements this node signed when the
/// message removed it (else empty), and a received acknowledgement that
/// verified (else `None`).
#[derive(Debug)]
pub struct ReceiveOutcome {
    pub org_id: OrgId,
    pub epoch: Epoch,
    pub root: RootHash,
    pub acknowledgements: Vec<Acknowledgement>,
    pub acknowledged: Option<VerifiedAcknowledgement>,
}

/// The committed update a node sends: the Envelope and the encoded record as
/// it stood before the commit (LLR-cmdrp9, LLR-bg3vsw).
#[derive(Clone, Debug)]
pub struct OutgoingUpdate {
    pub envelope: Envelope,
    pub record_snapshot: Vec<u8>,
}

/// What `commit_update` returns (LLR-cmdrp9): the committed update, the
/// notice of each Device it removed (LLR-a8z7r5), and the acknowledgements
/// this node signed when the commit removed it (LLR-b27jr6; else empty).
#[derive(Clone, Debug)]
pub struct CommitOutcome {
    pub org_id: OrgId,
    pub epoch: Epoch,
    pub root: RootHash,
    pub outgoing: OutgoingUpdate,
    pub revocations: Vec<RevocationNotice>,
    pub acknowledgements: Vec<Acknowledgement>,
}

/// Outcome of `receive_and_self_delete_if_revoked` (LLR-pt32fx).
// One outcome per received message, moved once to the caller: the size of
// `Acknowledged` costs nothing worth a box, and the design names the variant
// as holding the verified acknowledgement itself.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum SelfDeleteOutcome {
    /// This node was removed: it signed these acknowledgements, then forgot
    /// the Organisation.
    SelfDeleted { org_id: OrgId, acknowledgements: Vec<Acknowledgement> },
    UpdatedNotRevoked { org_id: OrgId },
    /// A received acknowledgement verified; nothing was written.
    Acknowledged(VerifiedAcknowledgement),
}

// ============================================================
// ChainOpsReader — adapts a single OrgState into ChainReader for verify.rs.
// ============================================================

/// Adapts a cached `OrgState` value into the synchronous `ChainReader` trait
/// expected by `verify_envelope_against_chain`.  The state was read from the
/// async `ChainOps::read_state` before calling verify.
struct ChainOpsReader {
    state: OrgState,
}

impl crate::chain::ChainReader for ChainOpsReader {
    fn get_org_state(&self, _org_id: &OrgId) -> Result<Option<OrgState>, String> {
        Ok(Some(self.state))
    }
}

// ============================================================
// The commit step and the removal step, over store values.
// ============================================================

/// What `commit_step` produced: the successor store, the outcome, and
/// whether the commit removed this node (the successor then forgot the
/// Organisation and the outcome holds its acknowledgements).
pub(crate) struct Committed {
    pub(crate) store: StoreData,
    pub(crate) outcome: CommitOutcome,
    pub(crate) removed: bool,
}

/// Commit `update`, this node's own provisional update for `org_id`,
/// against `chain`, the state read for it: the Envelope built from the
/// update, verified against `store`'s record and `chain` (LLR-cmdrp9,
/// LLR-6s785x); the notices of exactly this update (LLR-a8z7r5); then the
/// record takes the update's key pair (REQ-jy6ybw), or, when no Persona
/// bound to `org_id` is still listed, the one removal step with the seeds
/// `device_seeds` yields, called on that path alone so a commit that keeps
/// this node listed never obtains a device seed (LLR-b27jr6, LLR-23sfdh). `commit_update` and `reconcile`
/// both run it (LLR-fm38ww). A refusal returns the error alone; `store` is
/// borrowed and unchanged.
pub(crate) fn commit_step(
    store: &StoreData,
    org_id: OrgId,
    update: &ProvisionalUpdate,
    chain: &OrgState,
    device_seeds: impl FnOnce() -> Vec<DeviceSeed>,
) -> Result<Committed, OrgNodeError> {
    let rec = store.orgs.iter().find(|record| record.org_id == org_id).ok_or(OrgNodeError::OrgNotOnChain)?;
    let ProvisionalChange::ChangeSet { change_set, org_private_key } = update.change.clone() else {
        return Err(OrgNodeError::NoProvisionalUpdate);
    };
    let envelope = Envelope { org_id, parent_seq: update.seq, delta_bytes: change_set };
    let ctx = VerifyContext {
        expected_org_id: org_id,
        seq_guard: SeqGuard::from_last_seen(rec.last_seq),
        last_committed_epoch: rec.epoch,
    };
    let local = trie_from_snapshots(&rec.trie_members)?;
    let verified = verify_envelope_against_chain(&local, &envelope, &ctx, &ChainOpsReader { state: *chain })?;
    let record_snapshot = encode_record_snapshot(&rec.trie_members)?;
    let root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;
    // The notices of exactly this update: each Device the record listed
    // before it and the committed record no longer lists (LLR-a8z7r5).
    let committed = OrgRecord { trie_members: verified.trie.members().iter().map(snapshot_of).collect(), ..rec.clone() };
    let revocations = crate::revocation::notices_for(rec, &committed, &verified.trie)?;
    let removed = !still_member(store, org_id, &verified.trie);
    let (successor, acknowledgements) = if removed {
        removal_step(store, org_id, chain, device_seeds())?
    } else {
        let mut successor = store.clone();
        OrgService::commit_held(&mut successor, org_id, &verified, &envelope.delta_bytes, org_private_key, update.org_pub_key)?;
        (successor, Vec::new())
    };
    let outcome = CommitOutcome {
        org_id,
        epoch: verified.epoch,
        root,
        outgoing: OutgoingUpdate { envelope, record_snapshot },
        revocations,
        acknowledgements,
    };
    Ok(Committed { store: successor, outcome, removed })
}

/// The one removal step of every commit path (LLR-23sfdh): a commit removed
/// this node from `org_id`, so it signs one acknowledgement per Persona
/// bound to it against `chain`, the state the commit was decided against,
/// then returns `forget_organisation(org_id)` with the acknowledgements
/// (LLR-b27jr6, LLR-6p4pj2). A signing refusal returns the error alone.
fn removal_step(
    store: &StoreData,
    org_id: OrgId,
    chain: &OrgState,
    device_seeds: Vec<DeviceSeed>,
) -> Result<(StoreData, Vec<Acknowledgement>), OrgNodeError> {
    let acknowledgements = crate::revocation::sign_acknowledgements(store, org_id, chain, device_seeds)?;
    Ok((store.forget_organisation(org_id), acknowledgements))
}

/// The member-sender rule (owner ruling R1 of 2026-10-07): `sender` must be
/// in some member snapshot's `device_keys` of `record`, else
/// `SenderNotListed`. Called for Organisation information about a held
/// Organisation (LLR-2r2fha) and for a revocation (LLR-kzgjz8), never for a
/// first admission or an acknowledgement.
fn sender_listed(record: &OrgRecord, org_id: OrgId, sender: &DevicePublicKey) -> Result<(), OrgNodeError> {
    if record.trie_members.iter().any(|member| member.device_keys.contains(sender)) {
        Ok(())
    } else {
        Err(OrgNodeError::SenderNotListed { org_id })
    }
}

/// Whether any Persona bound to `org_id` has its device in `trie`.
fn still_member(store: &StoreData, org_id: OrgId, trie: &Trie) -> bool {
    store.personas.iter().filter(|persona| persona.org_id == Some(org_id)).any(|persona| {
        crate::revocation::persona_device(persona)
            .is_some_and(|device| trie.members().iter().any(|member| member.has_p2p_device(&device)))
    })
}

/// The device seeds of the Personas bound to `org_id`, cloned for one
/// signing call.
// S4: org-io moves the seed in from the OS keychain (REQ-y99c9w)
fn device_seeds_bound_to(store: &StoreData, org_id: OrgId) -> Vec<DeviceSeed> {
    store
        .personas
        .iter()
        .filter(|persona| persona.org_id == Some(org_id))
        .map(|persona| persona.device_seed.clone())
        .collect()
}

// ============================================================
// Utility helpers.
// ============================================================

/// A Persona identifier from its Member key: the first 16 bytes, in hex.
fn persona_id_for(member_key: &PersonPublicKey) -> PersonaId {
    use std::fmt::Write as _;
    PersonaId::new(member_key.as_bytes().iter().take(16).fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    }))
}

/// A fresh `MemberId`: 32 bytes from the caller's cryptographic random
/// source, never derived from a key (REQ-d9g6nt).
fn fresh_member_id<R: RngCore + CryptoRng>(rng: &mut R) -> MemberId {
    let mut id = [0u8; 32];
    rng.fill_bytes(&mut id);
    MemberId::new(id)
}
