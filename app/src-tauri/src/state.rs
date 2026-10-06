//! AppState: the Tauri-managed state for the ODS PoC app.
//!
//! `AppState::init` is the ONLY place in this unit that reads the process
//! environment. It reads, then hands what it read to the total functions in
//! `crate::policy`, which decide. That split is what makes the decisions
//! testable: cargo runs integration tests in threads of one process, so a test
//! that set an environment variable would race every other test in the binary.
//!
//! On startup, `init` resolves:
//!
//! - Data directory: `ODS_DATA_DIR`, else Tauri's `app_data_dir`, else — only
//!   under `ODS_ALLOW_DEV_DEFAULTS` — a temporary directory (REQ-rxc8sp).
//! - Passphrase: `ODS_PASSPHRASE`, else — only under `ODS_ALLOW_DEV_DEFAULTS` —
//!   the built-in development passphrase (REQ-7g3k9a). Absent both, startup
//!   REFUSES rather than silently protecting real key material with a published
//!   literal.
//! - Chain mode: if `ODS_CHAIN_WS` + `ODS_CONTRACT_H160` + `ODS_ADMIN_SEED`
//!   are all set, `build_chain_ops` is called EAGERLY here via `block_on`. On
//!   success it returns the read ops, the chain writer (`crate::submit`) AND
//!   the `ChainEndpoint` they were built from, which is recorded in
//!   `chain_endpoint`. On failure the service falls back to
//!   `ChainNotConfigured`, the writer to `WriterNotConfigured`, and
//!   `chain_endpoint` is `None`.
//!
//! `chain_endpoint` replaces the former `chain_ready: bool` + environment
//! re-read. The verdict and the endpoint are now ONE `Option`, so there is no
//! state in which the app reports "chain not configured" beside an endpoint it
//! is not talking to (REQ-e4ah9h, REQ-bvx4nh).
//!
//! `OrgService` is held behind a `tokio::sync::Mutex` so Tauri command
//! handlers can take an async lock without blocking the thread pool.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use org_node::service::{ChainOps, OrgService};
use org_node::store::PersonaStore;
use org_node::transport::TransportMode;
use on_chain_client::write::AccountId;
use org_node::OrgNodeError;
use tokio::sync::Mutex;

use crate::invitation::OutstandingInvites;
use crate::policy;
use crate::submit::{ChainWriter, OnChainWriter, WriterNotConfigured};

/// Re-exported from `crate::policy`, where the projection now lives, so that no
/// caller of `state::ConnectionStatus` had to change when it moved.
pub use crate::policy::{connection_status_from_state, ConnectionStatus};

/// Shared Tauri-managed state.  Tauri's `.manage()` wraps this in `State<T>`;
/// command handlers extract it via `State<'_, AppState>`.
pub struct AppState {
    pub service: Mutex<OrgService>,
    /// REQ-nfr3n2: the chain write, made by the app through on-chain-client
    /// before org-node is asked to commit (`crate::submit`).
    pub writer: Box<dyn ChainWriter>,
    /// LLR-f35pda: the Invites this device issued and has not seen acted on,
    /// kept in `outstanding_invites.json` in the data directory.
    pub outstanding: Mutex<OutstandingInvites>,
    /// The directory the store was actually opened in. Recorded here so that
    /// `connection_status` reports the directory in use rather than re-deriving
    /// one from the environment and possibly disagreeing with `init`.
    pub data_dir: PathBuf,
    /// REQ-bvx4nh: the endpoint the running configuration was built from.
    /// `None` is exactly "chain not configured" — the two are one fact, so
    /// they cannot disagree (REQ-e4ah9h).
    pub chain_endpoint: Option<policy::ChainEndpoint>,
    /// REQ-645jq9. This unit's own name for the transport mode, because it is
    /// serialised across the IPC boundary and org-node's type is not.
    pub transport_mode: policy::TransportModeName,
    /// REQ-6hgm8r / REQ-3hfggn. `Arc` so a `StartGuard` can be moved into the
    /// spawned receiver task and release the slot when that task ends.
    pub receiver_started: Arc<AtomicBool>,
}

/// A `ChainOps` implementation that rejects every read with a clear message
/// indicating that the chain env vars are not configured.  Used when the app
/// starts without `ODS_CHAIN_WS` / `ODS_CONTRACT_H160` / `ODS_ADMIN_SEED`.
/// Its write half is `crate::submit::WriterNotConfigured`.
struct ChainNotConfigured;

#[async_trait::async_trait]
impl ChainOps for ChainNotConfigured {
    async fn read_state(
        &self,
        _org_id: org_node::OrgId,
    ) -> Result<Option<org_node::OrgState>, OrgNodeError> {
        Err(OrgNodeError::Chain(
            "chain not configured: set ODS_CHAIN_WS, ODS_CONTRACT_H160, ODS_ADMIN_SEED"
                .into(),
        ))
    }
}

/// `policy::TransportModeName` — this unit's serialisable name for the mode —
/// as the mode org-node's transport layer takes.
fn transport_mode_for(name: policy::TransportModeName) -> TransportMode {
    match name {
        policy::TransportModeName::Loopback => TransportMode::Loopback,
        policy::TransportModeName::Networked => TransportMode::Networked,
    }
}

impl AppState {
    /// Initialise `AppState`.
    ///
    /// This is the only environment-reading function in the unit. Every
    /// decision it makes is delegated to `crate::policy`, which is total,
    /// parameterised and gated by `tests/startup_policy.rs`.
    ///
    /// `tauri_data_dir` is what the platform path resolver returned, or `None`
    /// if it failed — the failure is NOT swallowed into a hard-coded `/tmp`
    /// path by the caller any more, because whether that fallback is allowed is
    /// `resolve_data_dir`'s decision to make (REQ-rxc8sp).
    pub fn init(tauri_data_dir: Option<PathBuf>) -> Result<Self, String> {
        // REQ-7g3k9a / REQ-rxc8sp: the built-in development values are
        // available only behind an explicit opt-in. Absent it, a missing value
        // is a refusal with a message that names both variables (REQ-bmk2z2).
        let allow_dev = std::env::var("ODS_ALLOW_DEV_DEFAULTS").is_ok();
        let data_dir = policy::resolve_data_dir(
            std::env::var("ODS_DATA_DIR").ok().as_deref(),
            tauri_data_dir,
            allow_dev,
        )
        .map_err(|e| e.to_string())?;
        let passphrase = policy::resolve_passphrase(
            std::env::var("ODS_PASSPHRASE").ok().as_deref(),
            allow_dev,
        )
        .map_err(|e| e.to_string())?;

        // Chain read and write: try to build them from env; fall back to
        // ChainNotConfigured and WriterNotConfigured. The endpoint comes back
        // WITH them, so what is reported is what was built and cannot drift
        // from it (REQ-bvx4nh).
        let (chain, writer, chain_endpoint): (Box<dyn ChainOps>, Box<dyn ChainWriter>, _) =
            match build_chain_ops() {
                Ok((ops, writer, endpoint)) => (ops, writer, Some(endpoint)),
                Err(e) => {
                    // Surface WHY chain mode didn't come up (missing/malformed env var
                    // or a failed connect) instead of silently degrading — otherwise the
                    // UI just shows "Chain NOT configured" with no diagnosable reason.
                    eprintln!("[ods] chain config failed; running ChainNotConfigured: {e}");
                    (Box::new(ChainNotConfigured), Box::new(WriterNotConfigured), None)
                }
            };

        // Transport mode: `ODS_TRANSPORT=loopback` → Loopback (relay disabled,
        // same-machine app run or CI smoke-test); anything else (including
        // unset) → Networked (n0 relay + discovery) — the right choice for two
        // laptops over the internet on live Paseo.
        let transport_mode =
            policy::transport_mode_from(std::env::var("ODS_TRANSPORT").ok().as_deref());

        Self::assemble(data_dir, &passphrase, chain, writer, chain_endpoint, transport_mode)
    }

    /// Open the store under `data_dir` and wire the service. Shared by `init`
    /// and `for_test` so the two cannot drift in how the state is built.
    fn assemble(
        data_dir: PathBuf,
        passphrase: &str,
        chain: Box<dyn ChainOps>,
        writer: Box<dyn ChainWriter>,
        chain_endpoint: Option<policy::ChainEndpoint>,
        transport_mode: policy::TransportModeName,
    ) -> Result<Self, String> {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("create data_dir {}: {e}", data_dir.display()))?;
        let store = PersonaStore::open(data_dir.join("persona_store.bin"), passphrase)
            .map_err(|e| format!("open store: {e}"))?;

        let mut service = OrgService::new(store, chain);
        service.set_transport_mode(transport_mode_for(transport_mode));
        let outstanding = OutstandingInvites::open(data_dir.join("outstanding_invites.json"))?;

        Ok(Self {
            service: Mutex::new(service),
            writer,
            outstanding: Mutex::new(outstanding),
            data_dir,
            chain_endpoint,
            transport_mode,
            receiver_started: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Build a state directly, for tests. Never reads the environment.
    ///
    /// The store it opens is real — the point of `tests/ipc.rs` is to drive the
    /// real handlers over the real boundary — but every input the environment
    /// would have supplied is a parameter, so two tests in one binary cannot
    /// interfere with each other.
    #[cfg(feature = "test-support")]
    pub fn for_test(
        data_dir: PathBuf,
        passphrase: &str,
        chain_endpoint: Option<policy::ChainEndpoint>,
        transport_mode: policy::TransportModeName,
    ) -> Result<Self, String> {
        Self::assemble(
            data_dir,
            passphrase,
            Box::new(ChainNotConfigured),
            Box::new(WriterNotConfigured),
            chain_endpoint,
            transport_mode,
        )
    }
}

/// What a configured chain gives the app: the read, the write, and the
/// endpoint both were built from.
type ChainWiring = (Box<dyn ChainOps>, Box<dyn ChainWriter>, policy::ChainEndpoint);

/// Attempt to build the chain read (`SubxtChainOps`) and the chain write
/// (`crate::submit::OnChainWriter`) from `ODS_CHAIN_WS`, `ODS_CONTRACT_H160`,
/// and `ODS_ADMIN_SEED`.  Returns `Err` (without allocating a connection) if any
/// required var is absent or if the async connect fails.
///
/// This function is NOT async — Tauri's `setup` hook is synchronous in Tauri 2.
/// The async connection (OnlineClient::from_url + OrgRegistryClient::from_client)
/// is driven EAGERLY at startup via a `block_on` call here, inside `AppState::init`.
/// There is no lazy / deferred connection path; the ops and the writer (or the
/// `ChainNotConfigured` / `WriterNotConfigured` fallbacks) are fully determined
/// before `init` returns.
///
/// REQ-bvx4nh: the `ChainEndpoint` comes back ALONGSIDE the ops, from the same
/// values the ops were built from. Reporting it is then a read of what exists
/// rather than a second read of the environment that could disagree with it.
fn build_chain_ops() -> Result<ChainWiring, String> {
    let ws_url = std::env::var("ODS_CHAIN_WS").map_err(|_| "ODS_CHAIN_WS not set")?;
    let h160_hex =
        std::env::var("ODS_CONTRACT_H160").map_err(|_| "ODS_CONTRACT_H160 not set")?;
    let admin_seed_hex =
        std::env::var("ODS_ADMIN_SEED").map_err(|_| "ODS_ADMIN_SEED not set")?;

    // Parse contract H160 (40 hex chars).
    let h160_str = h160_hex.trim_start_matches("0x");
    if h160_str.len() != 40 {
        return Err(format!(
            "ODS_CONTRACT_H160 must be 20 bytes (40 hex chars), got {} chars",
            h160_str.len()
        ));
    }
    let h160_bytes = hex::decode(h160_str)
        .map_err(|e| format!("ODS_CONTRACT_H160 hex decode: {e}"))?;
    let mut contract_h160 = [0u8; 20];
    contract_h160.copy_from_slice(&h160_bytes);

    // Parse admin seed (64 hex chars = 32 bytes).
    let seed_str = admin_seed_hex.trim_start_matches("0x");
    if seed_str.len() != 64 {
        return Err(format!(
            "ODS_ADMIN_SEED must be 32 bytes (64 hex chars), got {} chars",
            seed_str.len()
        ));
    }
    let seed_bytes =
        hex::decode(seed_str).map_err(|e| format!("ODS_ADMIN_SEED hex decode: {e}"))?;
    let mut admin_seed = [0u8; 32];
    admin_seed.copy_from_slice(&seed_bytes);

    // Optional co-signer pubkey (64 hex chars = 32 bytes).
    let co_signatories: Vec<AccountId> = match std::env::var("ODS_COSIGNER_PUB") {
        Ok(s) => {
            let hex_str = s.trim_start_matches("0x");
            let bytes =
                hex::decode(hex_str).map_err(|e| format!("ODS_COSIGNER_PUB decode: {e}"))?;
            if bytes.len() != 32 {
                return Err("ODS_COSIGNER_PUB must be 32 bytes".into());
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            vec![AccountId(arr)]
        }
        Err(_) => vec![],
    };

    // Build the read and the write by connecting to the chain on Tauri's PERSISTENT async
    // runtime. This must NOT use a temporary `tokio::runtime::Runtime` created
    // here: the RPC client (reconnecting or not) spawns a background task for the
    // WS connection, and if that task is spawned on a throwaway runtime that is
    // dropped when this function returns, the task dies — so the first chain call
    // made later (e.g. submitting a genesis write) fails with "The client was
    // dropped" / "Error reason could not be found", even though reads issued
    // during the connect succeeded. `tauri::async_runtime` lives for the whole
    // app, so the background task (and the connection) survive until shutdown, and
    // it is the same runtime the command handlers later use.
    let endpoint = policy::ChainEndpoint {
        ws_url: ws_url.clone(),
        contract_h160: h160_hex.clone(),
    };
    let (chain, writer) = tauri::async_runtime::block_on(connect_chain(
        ws_url,
        contract_h160,
        admin_seed,
        co_signatories,
    ))?;
    Ok((Box::new(chain), Box::new(writer), endpoint))
}

/// Async: connect to the chain and build the read (`SubxtChainOps`) and the
/// write (`OnChainWriter`, on-chain-client's writer over subxt) over one client.
/// Runtime-unverified without a live chain or chopsticks fork.
async fn connect_chain(
    ws_url: String,
    contract_h160: [u8; 20],
    admin_seed: [u8; 32],
    co_signatories: Vec<AccountId>,
) -> Result<(org_node::SubxtChainOps, OnChainWriter), String> {
    use on_chain_client::write::subxt_ops::{FinalitySink, SubxtWriteOps};
    use subxt_signer::sr25519::Keypair;

    // Build the subxt client + registry reader via org-node's shared helper,
    // which uses the LegacyBackend RPC group. The default `OnlineClient::from_url`
    // backend (chainHead / new JSON-RPC) fails against the public Asset Hub
    // endpoints with "Cannot construct OnlineClientAtBlock: cannot get the block
    // header for block …", so the app must use the SAME LegacyBackend path as the
    // preflight (`connect_chain_client`) and `on-chain-client`.
    let (api, registry_client) =
        org_node::service::connect_chain_client(&ws_url, contract_h160)
            .await
            .map_err(|e| format!("connect_chain {ws_url}: {e}"))?;

    // Build the signatory's SR25519 keypair (`ODS_ADMIN_SEED`) from the raw
    // 32-byte mini-secret seed.
    // subxt_signer::sr25519::SecretKeyBytes = [u8; 32].
    let signatory = Keypair::from_secret_key(admin_seed)
        .map_err(|e| format!("admin Keypair: {e}"))?;

    let writer = OnChainWriter {
        ops: SubxtWriteOps::new(api, FinalitySink),
        signatory,
        co_signatories,
        contract: contract_h160,
    };
    Ok((org_node::SubxtChainOps::new(registry_client), writer))
}
