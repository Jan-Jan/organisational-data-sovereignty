//! OrgService: composes store + chain + transport into the five user stories.
//! See spec §6 and plan docs/superpowers/plans/2026-06-16-ods-phase-2-4-tauri-shell.md Task 2.
//!
//! Gated on the `app` feature (which implies `chain` + `transport`).
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::{Handle, MemberId, MemberLeaf, Name, P2pMemberKey, RootHash, Surname};
use rand_core::{CryptoRng, RngCore};

use crate::chain::OrgState;
use crate::envelope::SignedDeltaEnvelope;
use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::keys::SigningKeypair;
use crate::sequence::SeqGuard;
use crate::store::{
    MemberSnapshot, OrgRecord, PendingInvite, PersonaRecord, PersonaStatus, PersonaStore, RawMemberSnapshot,
};
use crate::transport::TransportMode;
use crate::transport::endpoint::OrgEndpoint;
use crate::transport::wire::WireMessage;
use crate::types::{ChainAccount, Epoch, OrgPublicKey, OrgSecret, PersonaId, SequenceNumber};
use crate::verify::{VerifyContext, verify_envelope_against_chain};

type Trie = OrgTrie<Blake3Hasher>;

// ============================================================
// ChainOps trait — the submit/read oracle injected into OrgService.
// ============================================================

/// Abstraction over on-chain operations so the service is headless-testable.
/// Production wires a real subxt client; tests inject `MockChainOps`.
#[async_trait]
pub trait ChainOps: Send + Sync {
    /// Submit genesis (create proxy, map, update epoch 0).
    ///
    /// Returns `(org_id, proxy_account)` where `org_id = h160_of(P)` and
    /// `proxy_account` is the pure proxy's `ChainAccount` `P`
    /// (used by `submit_update` to build the `proxied(P, ...)` call).
    /// Mock implementations return `None` for `proxy_account`; the production
    /// `SubxtChainOps` returns `Some(p)`.
    async fn submit_genesis(
        &self,
        genesis_root: RootHash,
        org_pub_key: OrgPublicKey,
    ) -> Result<(OrgId, Option<ChainAccount>), OrgNodeError>;

    /// Submit a root update for an existing org at `expected_epoch`.
    ///
    /// `proxy_account` is the pure proxy's `ChainAccount` `P` that was recorded at
    /// genesis.  The production implementation (`SubxtChainOps`) uses it to
    /// construct the `proxied(P, ...)` call; mock implementations may ignore it.
    /// Passing `None` causes `SubxtChainOps` to fall back to its in-memory
    /// `proxy_map` (populated during `submit_genesis` in the same process).
    async fn submit_update(
        &self,
        org_id: OrgId,
        new_root: RootHash,
        org_pub_key: OrgPublicKey,
        expected_epoch: Epoch,
        proxy_account: Option<ChainAccount>,
    ) -> Result<(), OrgNodeError>;

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
}

impl Default for MockChainOps {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChainOps for MockChainOps {
    async fn submit_genesis(
        &self,
        genesis_root: RootHash,
        org_pub_key: OrgPublicKey,
    ) -> Result<(OrgId, Option<ChainAccount>), OrgNodeError> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        // Deterministic org_id derived from the genesis root (first 20 bytes).
        let mut id_bytes = [0u8; 20];
        id_bytes.copy_from_slice(&genesis_root.as_bytes()[..20]);
        // Use seed counter to ensure uniqueness across multiple genesis calls.
        id_bytes[0] ^= g.next_id_seed;
        g.next_id_seed = g.next_id_seed.wrapping_add(1);
        let org_id = OrgId::new(id_bytes);
        g.slots.insert(org_id, OrgState {
            root_hash: genesis_root,
            org_pub_key,
            epoch: Epoch::new(1),
        });
        // Mock has no real pure-proxy; return None so OrgRecord.proxy_account stays None.
        Ok((org_id, None))
    }

    async fn submit_update(
        &self,
        org_id: OrgId,
        new_root: RootHash,
        org_pub_key: OrgPublicKey,
        expected_epoch: Epoch,
        _proxy_account: Option<ChainAccount>,
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
        g.slots.insert(org_id, OrgState {
            root_hash: new_root,
            org_pub_key,
            epoch: Epoch::new(expected_epoch.get() + 1),
        });
        Ok(())
    }

    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        let g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        Ok(g.slots.get(&org_id).copied())
    }
}

// ============================================================
// SubxtChainOps — real impl wired to genesis_ceremony + subxt write path.
// ============================================================

#[cfg(feature = "chain")]
mod subxt_impl {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use async_trait::async_trait;
    use on_chain_client::{OrgAdmin, OrgRegistryClient};
    use org_members::RootHash;
    use subxt::OnlineClient;
    use subxt::config::PolkadotConfig;
    use subxt_signer::sr25519::Keypair;
    use tokio::time::sleep;

    use crate::chain::OrgState;
    use crate::chain_write::WriteError;
    use crate::chain_write::calldata::revive_update_runtime_call;
    use crate::chain_write::multisig::dispatch_org_call;
    use crate::chain_write::proxy::{BlockSink, proxied};
    use crate::ceremony::genesis_ceremony;
    use crate::error::OrgNodeError;
    use crate::ids::OrgId;
    use crate::types::{ChainAccount, Epoch, OrgPublicKey};

    fn write_err(e: WriteError) -> OrgNodeError {
        OrgNodeError::Chain(format!("chain write: {e}"))
    }

    /// A `BlockSink` that polls the finalized-block cursor until a block NEWER
    /// than the one that was current at construction time is finalized, then
    /// returns that block's hash.
    ///
    /// **Why this matters for live Paseo**: Paseo produces a block every ~6–12 s
    /// and finalizes with GRANDPA slightly later.  Simply calling
    /// `at_current_block()` immediately after `submit` can return the same
    /// finalized block the extrinsic was *submitted at*, before a new block
    /// including the extrinsic has been finalized.  The poll loop below waits
    /// until `at_current_block()` reports a strictly newer hash (i.e. at least
    /// one new finalized block has appeared), with a ~2 s cadence and a
    /// configurable deadline (default 90 s).
    ///
    /// **Chopsticks instant mode**: in chopsticks interval/instant mode a new
    /// block is produced in milliseconds, so the first or second poll iteration
    /// resolves immediately — the overhead is negligible.
    ///
    /// **Correctness note**: `settle()` is called AFTER the extrinsic has been
    /// submitted.  Its return value (a block hash) is consumed by
    /// `create_pure` to look up the `Proxy.PureCreated` event.
    /// We return the hash of the first *new* finalized block, which is
    /// guaranteed to be the block that included (or post-dates) our extrinsic,
    /// so the event query will find it.
    pub struct FinalitySink {
        pub api: OnlineClient<PolkadotConfig>,
        /// How long `settle` waits before giving up (default: 90 s).
        pub timeout: Duration,
    }

    #[async_trait]
    impl BlockSink for FinalitySink {
        /// Poll until a new finalized block appears (relative to the snapshot
        /// taken at the start of the call), then return its hash.
        ///
        /// Steps:
        /// 1. Snapshot the current finalized block hash (`pre_hash`).
        /// 2. Loop with ~2 s sleeps:
        ///    a. Call `at_current_block()` to get the latest finalized hash.
        ///    b. If it differs from `pre_hash`, return it — a new block landed.
        /// 3. If `self.timeout` elapses without a new block (e.g. the chain is
        ///    stalled), return the pre-submit hash so callers can degrade
        ///    gracefully rather than hanging forever.
        ///
        /// This is compile-verified; runtime verification requires a live chain
        /// or chopsticks fork.
        async fn settle(&self) -> Result<[u8; 32], WriteError> {
            // 1. Snapshot the finalized block at call time.
            let pre_hash = self
                .api
                .at_current_block()
                .await
                .map_err(|e| WriteError::Subxt(format!("settle/pre_hash at_current_block: {e}")))?
                .block_ref()
                .hash()
                .0;

            let deadline = Instant::now() + self.timeout;
            let poll_interval = Duration::from_secs(2);

            // 2. Poll until a new finalized block appears or we time out.
            loop {
                sleep(poll_interval).await;

                let current = self
                    .api
                    .at_current_block()
                    .await
                    .map_err(|e| WriteError::Subxt(format!("settle/poll at_current_block: {e}")))?;
                let current_hash = current.block_ref().hash().0;

                if current_hash != pre_hash {
                    // A new finalized block appeared — the extrinsic has landed.
                    return Ok(current_hash);
                }

                // 3. Timed out — return pre-submit hash so callers degrade
                //    gracefully rather than hanging forever.
                if Instant::now() >= deadline {
                    return Ok(pre_hash);
                }
            }
        }
    }

    /// Production `ChainOps` that drives the real on-chain ceremony and update
    /// path.  Parameterised at construction; wired from `AppState` via env vars.
    ///
    /// `proxy_map` is populated by `submit_genesis` and consumed by
    /// `submit_update` — the pure proxy AccountId32 `P` is needed to build the
    /// `proxied(P, ...)` call, but `ChainOps::submit_update` only receives the
    /// `OrgId` (which is `h160_of(P)`; we cannot reverse the keccak).
    pub struct SubxtChainOps {
        pub api: OnlineClient<PolkadotConfig>,
        pub registry_client: OrgRegistryClient,
        pub contract_h160: [u8; 20],
        /// The sole signer / admin for the 1-of-1 multisig.
        pub admin: Keypair,
        /// Co-signatories for the threshold-1 multisig (empty for a true 1-of-1).
        pub others: Vec<ChainAccount>,
        /// org_id → pure proxy AccountId32; populated by submit_genesis.
        pub proxy_map: Arc<Mutex<HashMap<OrgId, ChainAccount>>>,
        /// How long `FinalitySink::settle` waits for inclusion.
        pub settle_timeout: Duration,
    }

    impl SubxtChainOps {
        /// Construct from a connected subxt client.  `admin_seed` is the
        /// 32-byte SR25519 secret seed for the admin (read from env in AppState).
        /// `others` are the co-signer public keys (empty for a true 1-of-1).
        pub fn new(
            api: OnlineClient<PolkadotConfig>,
            registry_client: OrgRegistryClient,
            contract_h160: [u8; 20],
            admin: Keypair,
            others: Vec<ChainAccount>,
        ) -> Self {
            Self {
                api,
                registry_client,
                contract_h160,
                admin,
                others,
                proxy_map: Arc::new(Mutex::new(HashMap::new())),
                settle_timeout: Duration::from_secs(90),
            }
        }

        fn sink(&self) -> FinalitySink {
            FinalitySink {
                api: self.api.clone(),
                timeout: self.settle_timeout,
            }
        }
    }

    #[async_trait]
    impl super::ChainOps for SubxtChainOps {
        async fn submit_genesis(
            &self,
            genesis_root: RootHash,
            org_pub_key: OrgPublicKey,
        ) -> Result<(OrgId, Option<ChainAccount>), OrgNodeError> {
            let sink = self.sink();
            let outcome = genesis_ceremony(
                &sink,
                &self.api,
                self.contract_h160,
                &self.admin, // funder == admin for PoC
                &self.admin,
                &self.others,
                genesis_root,
                org_pub_key,
            )
            .await
            .map_err(write_err)?;

            // Store the pure proxy AccountId32 in the in-memory map (for same-process
            // submit_update calls) AND return it so OrgService can persist it in OrgRecord.
            let mut map = self
                .proxy_map
                .lock()
                .map_err(|_| OrgNodeError::Chain("proxy_map lock poisoned".into()))?;
            map.insert(outcome.org_id, outcome.p);

            Ok((outcome.org_id, Some(outcome.p)))
        }

        async fn submit_update(
            &self,
            org_id: OrgId,
            new_root: RootHash,
            org_pub_key: OrgPublicKey,
            expected_epoch: Epoch,
            proxy_account: Option<ChainAccount>,
        ) -> Result<(), OrgNodeError> {
            // Resolve the pure proxy AccountId32 `P`.
            // Priority: (1) the persisted value passed in by OrgService, (2) the
            // in-memory proxy_map populated by submit_genesis in this process.
            let p = if let Some(pa) = proxy_account {
                pa
            } else {
                let map = self
                    .proxy_map
                    .lock()
                    .map_err(|_| OrgNodeError::Chain("proxy_map lock poisoned".into()))?;
                *map.get(&org_id).ok_or_else(|| {
                    OrgNodeError::Chain(format!(
                        "no proxy registered for org_id {:?}; pass proxy_account or call submit_genesis first",
                        org_id
                    ))
                })?
            };

            let call = revive_update_runtime_call(self.contract_h160, new_root, org_pub_key, expected_epoch);
            // dispatch_org_call submits, drives the chain via the sink, and
            // waits for the update extrinsic to finalize successfully (surfacing
            // ExtrinsicFailed), so no separate settle() is needed.
            let sink = self.sink();
            // The update must EXECUTE; a (threshold-≥2) pending approval is not a
            // successful update, so `into_executed` rejects it.
            dispatch_org_call(&sink, &self.api, &self.admin, &self.others, proxied(p, call))
                .await
                .map_err(write_err)?
                .into_executed()
                .map_err(write_err)?;
            Ok(())
        }

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
pub use subxt_impl::{FinalitySink, SubxtChainOps};

/// Connect to a chain RPC over an explicit `LegacyBackend` (required for
/// chopsticks; mirrors tests/common/conn.rs) and build an `OrgRegistryClient`
/// for `contract`. Used by the preflight binary. Returns both so callers can
/// run raw-chain checks (the `OnlineClient`) and contract checks (the
/// `OrgRegistryClient`).
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
    // idle period (e.g. submitting a genesis write minutes after startup) fails
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

/// The record a first admission extends: decoded from the snapshot the
/// admin sent. A first admission without one is refused (REQ-d9g6nt).
// Public only for the fuzz target `fuzz_first_admission_base`; not API.
#[doc(hidden)]
pub fn first_admission_base(genesis_snapshot: Option<&[u8]>) -> Result<Trie, OrgNodeError> {
    let snap_bytes = genesis_snapshot.ok_or_else(|| {
        OrgNodeError::Chain("first admission without a record snapshot".into())
    })?;
    let raw: Vec<RawMemberSnapshot> = postcard::from_bytes(snap_bytes)
        .map_err(|e| OrgNodeError::Chain(format!("genesis_snapshot decode: {e}")))?;
    let snaps = raw.into_iter().map(MemberSnapshot::try_from).collect::<Result<Vec<_>, _>>()?;
    trie_from_snapshots(&snaps)
}

/// Encode the record a pushed envelope extends, as a `WireMessage`'s
/// `genesis_snapshot`; `first_admission_base` decodes it.
fn encode_record_snapshot(snapshots: &[MemberSnapshot]) -> Result<Vec<u8>, OrgNodeError> {
    postcard::to_allocvec(snapshots)
        .map_err(|e| OrgNodeError::Chain(format!("genesis_snapshot encode: {e}")))
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
    /// `admit_member`/`revoke_member` dial peers.
    ///
    /// Must be called BEFORE any endpoint is bound (i.e. before the first
    /// `ensure_endpoint`, `admit_member`, `receive_and_verify`, or
    /// `revoke_member` call).  In production the Tauri `AppState::init` sets
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

    /// Generate a new persona with fresh ed25519 key material, persist it in
    /// the store, and return the persona_id.
    pub fn create_persona<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        handle: Handle,
        name: Name,
        surname: Surname,
    ) -> Result<PersonaId, OrgNodeError> {
        let member_kp = SigningKeypair::generate(rng);
        let device_kp = SigningKeypair::generate(rng);
        // Derive a unique persona_id from the member public key bytes.
        let persona_id = persona_id_for(&member_kp.member_key());
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
    // Story 1: create_organisation — genesis trie + chain submit.
    // ----------------------------------------------------------

    /// Build a genesis trie for the given persona, submit it to the chain,
    /// persist the `OrgRecord`, and mark the persona as `Active`.
    /// Returns the new `org_id`.
    pub async fn create_organisation<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
    ) -> Result<OrgId, OrgNodeError> {
        let (member_kp, device_kp, handle, name, surname) = self.persona_keys(persona_id)?;

        // Build genesis trie: admin = this persona.
        let admin_id = fresh_member_id(rng);
        let admin_leaf = MemberLeaf::new(
            admin_id,
            handle.clone(),
            member_kp.member_key(),
            name.clone(),
            surname.clone(),
            vec![device_kp.device_key()],
        )
        .map_err(OrgNodeError::Trie)?;
        let trie = Trie::genesis(vec![admin_leaf]).map_err(OrgNodeError::Trie)?;

        let genesis_root = trie.root_hash().map_err(OrgNodeError::Trie)?;
        let org_pub_key = OrgPublicKey::from(&member_kp.member_key());

        // Submit genesis to chain (stub for headless test; real chain for production).
        // Returns the org_id AND the pure-proxy AccountId32 P (Some for SubxtChainOps,
        // None for MockChainOps).  P is persisted in OrgRecord so submit_update can
        // find it after a restart.
        let (org_id, proxy_account) = self.chain.submit_genesis(genesis_root, org_pub_key).await?;

        // Persist the OrgRecord.
        let admin_snap = MemberSnapshot {
            id: admin_id,
            handle,
            name,
            surname,
            member_key: member_kp.member_key(),
            device_keys: vec![device_kp.device_key()],
        };
        let org_rec = OrgRecord {
            org_id,
            root_hash: genesis_root,
            org_pub_key,
            epoch: Epoch::new(1),
            org_secret: None,
            last_seq: SequenceNumber::new(0),
            admin_member_key: member_kp.member_key(),
            trie_members: vec![admin_snap],
            // Persist the pure-proxy AccountId32 returned by submit_genesis so that
            // submit_update can find P even after a restart (fixes Gap 2).
            // None for MockChainOps; Some(p) for SubxtChainOps.
            proxy_account,
        };
        self.store.data_mut().orgs.push(org_rec);

        // Transition persona to Active.
        self.update_persona_status(persona_id, org_id, PersonaStatus::Active)?;

        self.store.save(rng)?;
        Ok(org_id)
    }

    // ----------------------------------------------------------
    // Story 2: blobs — invite / join-request exchange.
    // ----------------------------------------------------------

    /// Build and encode an `Invite` blob for the given org.
    pub fn export_invite(&self, org_id: OrgId) -> Result<String, OrgNodeError> {
        let org_rec = self.find_org(org_id)?;
        let persona = self.admin_persona_for_org(org_id)?;
        let device_kp = persona.device_seed.signing_keypair();
        // Include the real bound endpoint address so the recipient can dial us back
        // if needed (Gap 1 fix).  If the endpoint has not been bound yet, the
        // admin_node_addr field is left empty — callers should call ensure_endpoint
        // before export_invite to populate it.
        let admin_node_addr = if let Some(ep) = &self.endpoint {
            postcard::to_allocvec(&ep.node_addr_for_dial())
                .map_err(|e| OrgNodeError::Chain(format!("addr encode: {e}")))?
        } else {
            vec![]
        };
        let inv = crate::blobs::Invite {
            org_id,
            org_pub_key: org_rec.org_pub_key,
            admin_member_key: org_rec.admin_member_key,
            admin_device_key: device_kp.device_key(),
            admin_node_addr,
        };
        crate::blobs::encode(&inv)
    }

    /// Decode and persist an incoming `Invite` blob.  Returns the decoded invite.
    /// Stores a `PendingInvite` in the encrypted store so that
    /// `receive_and_verify` can cross-check the authenticated sender against
    /// the admin device key the invite asserts (Fix 1 — security hardening).
    pub fn import_invite<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        blob: &str,
    ) -> Result<crate::blobs::Invite, OrgNodeError> {
        let inv: crate::blobs::Invite = crate::blobs::decode(blob)?;
        // Upsert: replace any existing pending invite for the same org.
        let pending = PendingInvite {
            org_id: inv.org_id,
            admin_device_key: inv.admin_device_key,
            admin_member_key: inv.admin_member_key,
            org_pub_key: inv.org_pub_key,
        };
        let data = self.store.data_mut();
        if let Some(existing) = data.pending_invites.iter_mut().find(|p| p.org_id == inv.org_id) {
            *existing = pending;
        } else {
            data.pending_invites.push(pending);
        }
        self.store.save(rng)?;
        Ok(inv)
    }

    /// Build and encode a `JoinRequest` blob for the given persona.  The
    /// endpoint must be bound so we can include the node address.
    pub fn export_join_request(&self, persona_id: &PersonaId) -> Result<String, OrgNodeError> {
        let persona = self.find_persona(persona_id)?;
        let device_kp = persona.device_seed.signing_keypair();
        let member_kp = persona.member_seed.signing_keypair();
        // Provide the iroh node address from the bound endpoint (if any).
        let node_addr = if let Some(ep) = &self.endpoint {
            let addr = ep.node_addr_for_dial();
            postcard::to_allocvec(&addr)
                .map_err(|e| OrgNodeError::Chain(format!("addr encode: {e}")))?
        } else {
            vec![]
        };
        let jr = crate::blobs::JoinRequest {
            handle: persona.handle.clone(),
            name: persona.name.clone(),
            surname: persona.surname.clone(),
            member_key: member_kp.member_key(),
            device_key: device_kp.device_key(),
            node_addr,
        };
        crate::blobs::encode(&jr)
    }

    /// Decode and store a `JoinRequest` blob (stores nothing — for the PoC the
    /// join request is transient; the admin calls `admit_member` directly).
    /// Returns the decoded request; a value its type's parse refuses is
    /// refused naming the field (LLR-8bum44).
    pub fn import_join_request(
        blob: &str,
    ) -> Result<crate::blobs::JoinRequest, OrgNodeError> {
        crate::blobs::decode_join_request(blob)
    }

    // ----------------------------------------------------------
    // Story 3: admit_member — trie add + submit_update + iroh push.
    // ----------------------------------------------------------

    /// Add a new member to the org trie, bump the on-chain epoch, build a
    /// `SignedDeltaEnvelope`, and push it (+ `org_secret`) to the new member
    /// over iroh.
    ///
    /// The dial behaviour depends on the configured [`TransportMode`]:
    /// - `Loopback`: dials `peer_addr` (the `EndpointAddr` decoded from the
    ///   `JoinRequest` blob).  Used for offline tests and same-machine runs.
    /// - `Networked`: dials the peer purely by its `EndpointId` (the device
    ///   key from `join_request.device_key`), ignoring `peer_addr`.  iroh
    ///   resolves connectivity via relay/DNS discovery; `peer_addr` may be
    ///   stale or empty in this mode.
    ///
    /// `join_request` carries the new member's keys.
    /// `org_secret` is an optional symmetric secret handed to the new member.
    pub async fn admit_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        join_request: &crate::blobs::JoinRequest,
        peer_addr: iroh::EndpointAddr,
        org_secret: Option<OrgSecret>,
    ) -> Result<MemberId, OrgNodeError> {
        // Rebuild local trie from stored snapshots.
        let (trie, org_epoch, org_pub_key, admin_member_kp, last_seq, pre_add_snapshots, proxy_account) = {
            let org_rec = self.find_org(org_id)?;
            let trie = trie_from_snapshots(&org_rec.trie_members)?;
            let epoch = org_rec.epoch;
            let pub_key = org_rec.org_pub_key;
            let last_seq = org_rec.last_seq;
            // Capture pre-add snapshots so B can reconstruct the genesis trie.
            let snapshots = org_rec.trie_members.clone();
            // Carry the persisted proxy_account so submit_update can use it after a restart.
            let proxy = org_rec.proxy_account;
            // Admin is the first member whose member key = admin_member_key.
            let admin_persona = self.admin_persona_for_org(org_id)?;
            let member_kp = admin_persona.member_seed.signing_keypair();
            (trie, epoch, pub_key, member_kp, last_seq, snapshots, proxy)
        };

        // Random, so a re-admission with the same keys gets a new id.
        let new_member_id = fresh_member_id(rng);
        let new_leaf = MemberLeaf::new(
            new_member_id,
            join_request.handle.clone(),
            join_request.member_key,
            join_request.name.clone(),
            join_request.surname.clone(),
            vec![join_request.device_key],
        )
        .map_err(OrgNodeError::Trie)?;

        let (new_trie, delta) =
            trie.add_member(new_leaf.clone()).map_err(OrgNodeError::Trie)?.recalculate().map_err(OrgNodeError::Trie)?;

        let new_root = new_trie.root_hash().map_err(OrgNodeError::Trie)?;

        // Encode pre-add snapshots so B can reconstruct the genesis trie for verification.
        // Before the chain write, so an encode failure leaves chain and record unchanged.
        let genesis_snapshot = Some(encode_record_snapshot(&pre_add_snapshots)?);

        // Submit on-chain update (epoch → epoch + 1).
        // Pass proxy_account so SubxtChainOps can find P even after a restart (Gap 2).
        self.chain
            .submit_update(org_id, new_root, org_pub_key, org_epoch, proxy_account)
            .await?;
        let new_epoch = Epoch::new(org_epoch.get() + 1);

        // Build the signed envelope.
        let parent_seq = SequenceNumber::new(last_seq.get() + 1);
        let envelope =
            SignedDeltaEnvelope::build(org_id, parent_seq, &delta, &admin_member_kp)
                .map_err(|_| OrgNodeError::MalformedDelta)?;

        // Push the WireMessage to the new member over iroh.
        // Use the lazily bound endpoint; bind from this persona's device seed if not yet bound.
        let msg = WireMessage { envelope, org_secret, genesis_snapshot };
        let admin_persona_id = self.admin_persona_for_org(org_id)?.persona_id.clone();
        let mode = self.transport_mode;
        let ep = self.ensure_endpoint(&admin_persona_id).await?;
        match mode {
            TransportMode::Loopback => {
                // Loopback/same-machine: dial the full EndpointAddr from the blob.
                ep.send(peer_addr, &msg)
                    .await
                    .map_err(|e| OrgNodeError::Chain(format!("iroh send: {e}")))?;
            }
            TransportMode::Networked => {
                // Cross-network: dial purely by EndpointId so iroh relay/DNS
                // resolves the path.  The device key from the JoinRequest equals
                // the peer's iroh EndpointId (same ed25519 key).
                let peer_id = iroh::EndpointId::from_bytes(join_request.device_key.as_bytes())
                    .map_err(|_| OrgNodeError::Chain("invalid joiner device key for EndpointId".into()))?;
                ep.send_to_id(peer_id, &msg)
                    .await
                    .map_err(|e| OrgNodeError::Chain(format!("iroh send (networked): {e}")))?;
            }
        }

        // Update the persisted OrgRecord.
        let new_snap = snapshot_of(&new_leaf);
        {
            let org_rec = self.find_org_mut(org_id)?;
            org_rec.root_hash = new_root;
            org_rec.epoch = new_epoch;
            org_rec.last_seq = parent_seq;
            org_rec.trie_members.push(new_snap);
        }
        self.store.save(rng)?;

        Ok(new_member_id)
    }

    // ----------------------------------------------------------
    // Story 4: receive_and_verify — recv envelope, verify, commit.
    // ----------------------------------------------------------

    /// Accept one inbound `WireMessage`, cross-check the sender's device key
    /// against the trie, verify the envelope against the chain, and commit
    /// the new state.
    pub async fn receive_and_verify<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
    ) -> Result<ReceiveOutcome, OrgNodeError> {
        // Bind the endpoint from the first persona's device_seed if not yet bound.
        let first_persona_id = self
            .store
            .data()
            .personas
            .first()
            .ok_or_else(|| OrgNodeError::Chain("no persona found — create one first".into()))?
            .persona_id
            .clone();
        let ep = self.ensure_endpoint(&first_persona_id).await?;
        let (remote_device_key, msg) = ep
            .recv_one()
            .await
            .map_err(|e| OrgNodeError::Chain(format!("iroh recv: {e}")))?;

        let org_id = msg.envelope.org_id;

        // Look up the org by the envelope's org_id.  If we don't have it yet
        // (first-time admission), we accept the envelope and store a new OrgRecord.
        // The `admin_member_key` must be in the envelope's metadata — we take it
        // from the verified trie after the chain check.
        //
        // First look for a matching pending OrgRecord (for an already-known org).
        // For the admission case we may not have one yet; we create it.

        // Find the admin member key from on-chain state is not possible without
        // the trie.  For the PoC, we take a different approach:
        //
        //   1. Read the chain state to get the authoritative root + epoch.
        //   2. We need the admin's member key to verify the signature.
        //      On first admission, we don't have the org yet.  We use the
        //      `org_pub_key` from the on-chain state — which the admin published
        //      as part of genesis — as the author member key.  This matches
        //      `create_organisation` which sets `org_pub_key = admin_member_vk_bytes`.

        let chain_state = self
            .chain
            .read_state(org_id)
            .await?
            .ok_or(OrgNodeError::OrgNotOnChain)?;

        let author_vk = *chain_state.org_pub_key.verifying_key();

        // Check whether we already have a local OrgRecord for this org.
        let mut is_first_admission = false;
        let (local_trie, last_seq, last_epoch) = {
            if let Some(existing) = self
                .store
                .data()
                .orgs
                .iter()
                .find(|o| o.org_id == org_id)
                .cloned()
            {
                let trie = trie_from_snapshots(&existing.trie_members)?;
                (trie, existing.last_seq, existing.epoch)
            } else {
                // Fresh admission: rebuild the pre-add record from the admin's
                // `genesis_snapshot`, so `verify_envelope_against_chain` can
                // check the envelope's `base_root` against it.
                let trie = first_admission_base(msg.genesis_snapshot.as_deref())?;
                is_first_admission = true;
                (trie, SequenceNumber::new(0), Epoch::new(0))
            }
        };

        // Security hardening — Fix 1: on FIRST ADMISSION, cross-check the
        // authenticated remote device key against the admin_device_key recorded
        // in the invite B imported for this org.  This prevents a rogue peer
        // from successfully pushing an envelope even if the chain anchor and
        // signature match (defense-in-depth — the invite is the explicit trust root).
        if is_first_admission {
            let pending = self
                .store
                .data()
                .pending_invites
                .iter()
                .find(|p| p.org_id == org_id)
                .cloned();
            if let Some(inv) = pending {
                if remote_device_key != inv.admin_device_key {
                    return Err(OrgNodeError::BadSignature);
                }
            }
            // If no invite was imported for this org, fall through to the chain/sig
            // proof (existing behaviour) — log-worthy in production but not a hard fail.
        }

        let seq_guard = SeqGuard::from_last_seen(last_seq);
        let ctx = VerifyContext {
            expected_org_id: org_id,
            author_member_key: &author_vk,
            seq_guard,
            last_committed_epoch: last_epoch,
        };

        // Use the `ChainReader`-compatible oracle backed by our `ChainOps`.
        let chain_reader = ChainOpsReader { state: chain_state };

        let verified = verify_envelope_against_chain(&local_trie, &msg.envelope, &ctx, &chain_reader)?;

        // Cross-check: the sender's authenticated device key must be in the new trie.
        // Skipped on first admission because B has not yet seen any member list;
        // the invite device-key check + chain root match is already sufficient proof.
        if !is_first_admission {
            let sender_known = verified
                .trie
                .members()
                .iter()
                .any(|m| m.has_p2p_device(&remote_device_key));
            if !sender_known {
                return Err(OrgNodeError::BadSignature);
            }
        }

        // Commit: update or create the OrgRecord.
        let new_snapshots: Vec<MemberSnapshot> =
            verified.trie.members().iter().map(snapshot_of).collect();

        let new_root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;

        // The persona linked to this org — find by matching device keys in the trie.
        // Our device key should be in the new trie.
        let my_persona_id = self
            .store
            .data()
            .personas
            .iter()
            .find(|p| {
                let my_device_key = p.device_seed.signing_keypair().device_key();
                verified.trie.members().iter().any(|m| {
                    m.has_p2p_device(&my_device_key)
                        && m.p2p_key().as_bytes() != chain_state.org_pub_key.as_bytes()
                })
            })
            .map(|p| p.persona_id.clone());

        // Find the member_id for our persona.
        let my_member_id = if let Some(ref pid) = my_persona_id {
            let persona = self.find_persona(pid)?;
            let my_device_key = persona.device_seed.signing_keypair().device_key();
            verified
                .trie
                .members()
                .iter()
                .find(|m| m.has_p2p_device(&my_device_key))
                .map(|m| *m.id())
        } else {
            None
        };

        {
            let data = self.store.data_mut();
            if let Some(existing) = data.orgs.iter_mut().find(|o| o.org_id == org_id) {
                existing.root_hash = new_root;
                existing.epoch = verified.epoch;
                existing.last_seq = verified.seq_guard.last_seen();
                existing.org_secret = msg.org_secret;
                existing.trie_members = new_snapshots;
            } else {
                data.orgs.push(OrgRecord {
                    org_id,
                    root_hash: new_root,
                    org_pub_key: chain_state.org_pub_key,
                    epoch: verified.epoch,
                    org_secret: msg.org_secret,
                    last_seq: verified.seq_guard.last_seen(),
                    // The published signing key is the admin's Member key today (PR-szkat6).
                    admin_member_key: P2pMemberKey::new(*chain_state.org_pub_key.verifying_key()),
                    trie_members: new_snapshots,
                    // Member-side record: P is only known by the admin who created the org.
                    proxy_account: None,
                });
            }
            // Consume the pending invite now that first admission has committed.
            if is_first_admission {
                data.pending_invites.retain(|p| p.org_id != org_id);
            }
        }

        // Mark persona as Active + set member_id.
        if let Some(ref pid) = my_persona_id {
            let personas = &mut self.store.data_mut().personas;
            if let Some(p) = personas.iter_mut().find(|p| &p.persona_id == pid) {
                p.status = PersonaStatus::Active;
                p.org_id = Some(org_id);
                p.member_id = my_member_id;
            }
        }

        self.store.save(rng)?;

        Ok(ReceiveOutcome {
            org_id,
            epoch: verified.epoch,
            root: new_root,
        })
    }

    // ----------------------------------------------------------
    // Story 5: revoke_member — trie remove + submit_update + notify.
    // ----------------------------------------------------------

    /// Remove a member from the org trie, bump the epoch, and push a
    /// revocation `WireMessage` to the revoked member's current device address.
    ///
    /// The revoked member's `OrgRecord` is then removed from their local store
    /// when they call `receive_and_verify` and detect the root-mismatch (their
    /// device is no longer in the committed trie) — or when `self_delete_if_revoked`
    /// is called explicitly.
    ///
    /// For the PoC the admin pushes the revocation envelope to the member.
    /// `peer_addr` is the revoked member's iroh address: REQUIRED in `Loopback`
    /// mode (same-machine dial), and ignored in `Networked` mode, where the
    /// peer's `EndpointId` is derived from the revoked member's device key in the
    /// stored trie snapshot. Pass `None` when no address is available (the normal
    /// Networked case).
    pub async fn revoke_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        member_id: MemberId,
        peer_addr: Option<iroh::EndpointAddr>,
    ) -> Result<(), OrgNodeError> {
        let (trie, org_epoch, org_pub_key, admin_member_kp, last_seq, proxy_account) = {
            let org_rec = self.find_org(org_id)?;
            let trie = trie_from_snapshots(&org_rec.trie_members)?;
            let epoch = org_rec.epoch;
            let pub_key = org_rec.org_pub_key;
            let last_seq = org_rec.last_seq;
            let proxy = org_rec.proxy_account;
            let admin_persona = self.admin_persona_for_org(org_id)?;
            let member_kp = admin_persona.member_seed.signing_keypair();
            (trie, epoch, pub_key, member_kp, last_seq, proxy)
        };

        let (new_trie, delta) = trie
            .delete_member(&member_id)
            .map_err(OrgNodeError::Trie)?
            .recalculate()
            .map_err(OrgNodeError::Trie)?;

        let new_root = new_trie.root_hash().map_err(OrgNodeError::Trie)?;

        // Include the pre-revocation snapshot so B can reconstruct its local trie
        // and verify the delta (base_root must match B's current trie).
        // Before the chain write, so an encode failure leaves chain and record unchanged.
        let genesis_snapshot =
            Some(encode_record_snapshot(&self.find_org(org_id)?.trie_members)?);

        // Submit on-chain update; pass persisted proxy_account (Gap 2 fix).
        self.chain.submit_update(org_id, new_root, org_pub_key, org_epoch, proxy_account).await?;
        let new_epoch = Epoch::new(org_epoch.get() + 1);

        // Build the signed revocation envelope.
        let parent_seq = SequenceNumber::new(last_seq.get() + 1);
        let envelope =
            SignedDeltaEnvelope::build(org_id, parent_seq, &delta, &admin_member_kp)
                .map_err(|_| OrgNodeError::MalformedDelta)?;

        // Push to the revoked peer so they can self-delete.
        // Collect all data that borrows from `self` BEFORE calling ensure_endpoint
        // (which takes a &mut self borrow that overlaps with find_org / admin_persona_for_org).
        let (admin_persona_id, networked_peer_id) = {
            let org_rec = self.find_org(org_id)?;
            let admin_persona_id = self.admin_persona_for_org(org_id)?.persona_id.clone();
            // Pre-compute the EndpointId for Networked mode from the snapshot (before
            // the snapshot is modified by the org update below).
            let networked_peer_id: Option<iroh::EndpointId> = if self.transport_mode == TransportMode::Networked {
                // PoC assumption (S14): one device per member, so the first
                // device key is the member's iroh identity. Multi-device members
                // would need to notify every device key here.
                let dk = org_rec
                    .trie_members
                    .iter()
                    .find(|s| s.id == member_id)
                    .and_then(|s| s.device_keys.first().copied())
                    .ok_or_else(|| OrgNodeError::Chain(
                        "revoked member device key not found in trie snapshot".into()
                    ))?;
                Some(
                    iroh::EndpointId::from_bytes(dk.as_bytes())
                        .map_err(|_| OrgNodeError::Chain("invalid device key for EndpointId".into()))?,
                )
            } else {
                None
            };
            (admin_persona_id, networked_peer_id)
        };
        let msg = WireMessage { envelope, org_secret: None, genesis_snapshot };
        // Use the lazily bound endpoint; bind from this persona's device seed if not yet bound.
        let mode = self.transport_mode;
        let ep = self.ensure_endpoint(&admin_persona_id).await?;
        match mode {
            TransportMode::Loopback => {
                // Loopback/same-machine: dial the full EndpointAddr (required here).
                let addr = peer_addr.ok_or_else(|| {
                    OrgNodeError::Chain("Loopback revoke requires the peer's EndpointAddr".into())
                })?;
                ep.send(addr, &msg)
                    .await
                    .map_err(|e| OrgNodeError::Chain(format!("iroh send: {e}")))?;
            }
            TransportMode::Networked => {
                // Cross-network: dial purely by EndpointId so iroh relay/DNS
                // resolves the path.  The EndpointId was derived above from the
                // revoked member's device key in the pre-revocation snapshot.
                let peer_id = networked_peer_id
                    .ok_or_else(|| OrgNodeError::Chain("networked_peer_id missing in Networked mode".into()))?;
                ep.send_to_id(peer_id, &msg)
                    .await
                    .map_err(|e| OrgNodeError::Chain(format!("iroh send (networked): {e}")))?;
            }
        }

        // Update local OrgRecord.
        let new_snaps: Vec<MemberSnapshot> = new_trie.members().iter().map(snapshot_of).collect();

        {
            let org_rec = self.find_org_mut(org_id)?;
            org_rec.root_hash = new_root;
            org_rec.epoch = new_epoch;
            org_rec.last_seq = parent_seq;
            org_rec.trie_members = new_snaps;
        }
        self.store.save(rng)?;
        Ok(())
    }

    /// If the local persona has been revoked (its device key is absent from the
    /// committed trie after a `receive_and_verify`), remove the `OrgRecord` and
    /// mark the persona `Revoked`. Called on B's side after receiving a
    /// revocation envelope that removes B from the trie.
    pub async fn receive_and_self_delete_if_revoked<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
    ) -> Result<SelfDeleteOutcome, OrgNodeError> {
        // Bind the endpoint from the first persona's device_seed if not yet bound.
        let first_persona_id = self
            .store
            .data()
            .personas
            .first()
            .ok_or_else(|| OrgNodeError::Chain("no persona found — create one first".into()))?
            .persona_id
            .clone();
        let ep = self.ensure_endpoint(&first_persona_id).await?;
        let (remote_device_key, msg) = ep
            .recv_one()
            .await
            .map_err(|e| OrgNodeError::Chain(format!("iroh recv: {e}")))?;

        let org_id = msg.envelope.org_id;

        let chain_state = self
            .chain
            .read_state(org_id)
            .await?
            .ok_or(OrgNodeError::OrgNotOnChain)?;

        let author_vk = *chain_state.org_pub_key.verifying_key();

        let (local_trie, last_seq, last_epoch) = {
            let existing = self
                .store
                .data()
                .orgs
                .iter()
                .find(|o| o.org_id == org_id)
                .cloned()
                .ok_or(OrgNodeError::OrgNotOnChain)?;
            let trie = trie_from_snapshots(&existing.trie_members)?;
            (trie, existing.last_seq, existing.epoch)
        };

        let seq_guard = SeqGuard::from_last_seen(last_seq);
        let ctx = VerifyContext {
            expected_org_id: org_id,
            author_member_key: &author_vk,
            seq_guard,
            last_committed_epoch: last_epoch,
        };

        let chain_reader = ChainOpsReader { state: chain_state };
        let verified =
            verify_envelope_against_chain(&local_trie, &msg.envelope, &ctx, &chain_reader)?;

        // Check whether OUR device is still in the new trie.
        let my_still_present = self
            .store
            .data()
            .personas
            .iter()
            .filter(|p| p.org_id == Some(org_id))
            .any(|p| {
                let my_device_key = p.device_seed.signing_keypair().device_key();
                verified.trie.members().iter().any(|m| m.has_p2p_device(&my_device_key))
            });

        let _ = remote_device_key; // authenticated but not cross-checked here (revocation path)

        if my_still_present {
            // Regular admit/update — commit the update normally.
            let new_snaps: Vec<MemberSnapshot> =
                verified.trie.members().iter().map(snapshot_of).collect();
            let new_root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;
            {
                let orgs = &mut self.store.data_mut().orgs;
                if let Some(rec) = orgs.iter_mut().find(|o| o.org_id == org_id) {
                    rec.root_hash = new_root;
                    rec.epoch = verified.epoch;
                    rec.last_seq = verified.seq_guard.last_seen();
                    rec.trie_members = new_snaps;
                }
            }
            self.store.save(rng)?;
            return Ok(SelfDeleteOutcome::UpdatedNotRevoked { org_id });
        }

        // We are revoked — self-delete.
        self.store.data_mut().orgs.retain(|o| o.org_id != org_id);
        // Mark matching personas as Revoked.
        for p in self.store.data_mut().personas.iter_mut() {
            if p.org_id == Some(org_id) {
                p.status = PersonaStatus::Revoked;
            }
        }
        self.store.save(rng)?;

        Ok(SelfDeleteOutcome::SelfDeleted { org_id })
    }

    // ----------------------------------------------------------
    // Query helpers.
    // ----------------------------------------------------------

    pub fn list_personas(&self) -> &[PersonaRecord] {
        &self.store.data().personas
    }

    /// The invites imported but not yet consumed by a first admission.
    ///
    /// A read accessor alongside `list_personas` and `list_orgs`. It exists
    /// because "leaving its record unchanged" on a rejected first admission
    /// (REQ-xa6smf) includes the invite, and without a way to observe the
    /// in-memory list a test can only see what reached the disk — which a
    /// rejection never writes.
    pub fn list_pending_invites(&self) -> &[PendingInvite] {
        &self.store.data().pending_invites
    }

    pub fn list_orgs(&self) -> &[OrgRecord] {
        &self.store.data().orgs
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

    fn find_org_mut(&mut self, org_id: OrgId) -> Result<&mut OrgRecord, OrgNodeError> {
        self.store
            .data_mut()
            .orgs
            .iter_mut()
            .find(|o| o.org_id == org_id)
            .ok_or(OrgNodeError::OrgNotOnChain)
    }

    fn admin_persona_for_org(&self, org_id: OrgId) -> Result<&PersonaRecord, OrgNodeError> {
        let org_rec = self.find_org(org_id)?;
        self.store
            .data()
            .personas
            .iter()
            .find(|p| {
                p.member_seed.signing_keypair().member_key() == org_rec.admin_member_key
            })
            .ok_or_else(|| OrgNodeError::Chain("admin persona not found for org".into()))
    }

    fn persona_keys(
        &self,
        persona_id: &PersonaId,
    ) -> Result<(SigningKeypair, SigningKeypair, Handle, Name, Surname), OrgNodeError> {
        let p = self.find_persona(persona_id)?;
        Ok((
            p.member_seed.signing_keypair(),
            p.device_seed.signing_keypair(),
            p.handle.clone(),
            p.name.clone(),
            p.surname.clone(),
        ))
    }

    fn update_persona_status(
        &mut self,
        persona_id: &PersonaId,
        org_id: OrgId,
        status: PersonaStatus,
    ) -> Result<(), OrgNodeError> {
        self.store
            .data_mut()
            .personas
            .iter_mut()
            .find(|p| &p.persona_id == persona_id)
            .ok_or_else(|| OrgNodeError::Chain(format!("persona not found: {}", persona_id.as_str())))
            .map(|p| {
                p.status = status;
                p.org_id = Some(org_id);
            })
    }
}

// ============================================================
// Outcomes.
// ============================================================

/// Outcome of `receive_and_verify`.
#[derive(Debug)]
pub struct ReceiveOutcome {
    pub org_id: OrgId,
    pub epoch: Epoch,
    pub root: RootHash,
}

/// Outcome of `receive_and_self_delete_if_revoked`.
#[derive(Debug)]
pub enum SelfDeleteOutcome {
    SelfDeleted { org_id: OrgId },
    UpdatedNotRevoked { org_id: OrgId },
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
// Utility helpers.
// ============================================================

/// A Persona identifier from its Member key: the first 16 bytes, in hex.
fn persona_id_for(member_key: &P2pMemberKey) -> PersonaId {
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

// ============================================================
// Unit tests (lib tests for service.rs; integration test is service_stories.rs).
// ============================================================

