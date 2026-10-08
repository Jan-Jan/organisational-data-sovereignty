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
//! - Chain mode: if `ODS_CHAIN_WS` + `ODS_CONTRACT_H160` are both set (and
//!   `ODS_COSIGNER_PUB` optionally), they are handed to `OrgIo::connect`, which
//!   is called EAGERLY here via `block_on`. org-io reads the signing seed
//!   (`ODS_ADMIN_SEED`) itself, and only in a build with the `dev-seed`
//!   feature (`cargo tauri dev --features dev-seed`); this unit never reads
//!   it. When the connect succeeds, the `ChainEndpoint` the app handed it is
//!   recorded in `chain_endpoint`. When it fails, the handle is
//!   `OrgIo::not_configured` (every chain read and write refused) and
//!   `chain_endpoint` is `None`.
//!
//! `chain_endpoint` replaces the former `chain_ready: bool` + environment
//! re-read. The verdict and the endpoint are now ONE `Option`, so there is no
//! state in which the app reports "chain not configured" beside an endpoint it
//! is not talking to (REQ-e4ah9h, REQ-bvx4nh).
//!
//! The org-io handle is held behind a `tokio::sync::Mutex` so Tauri command
//! handlers can take an async lock without blocking the thread pool.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use org_io::node::transport::TransportMode;
use org_io::{ChainSettings, OrgIo};
use tokio::sync::Mutex;

use crate::invitation::OutstandingInvites;
use crate::policy;

/// Re-exported from `crate::policy`, where the projection now lives, so that no
/// caller of `state::ConnectionStatus` had to change when it moved.
pub use crate::policy::{connection_status_from_state, ConnectionStatus};

/// Shared Tauri-managed state.  Tauri's `.manage()` wraps this in `State<T>`;
/// command handlers extract it via `State<'_, AppState>`.
pub struct AppState {
    /// The app's one way in to org-node, the chain and the signatory key's
    /// effects (REQ-m8sgjk: the app decides when, org-io submits).
    pub org_io: Mutex<OrgIo>,
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

        // Transport mode: `ODS_TRANSPORT=loopback` → Loopback (relay disabled,
        // same-machine app run or CI smoke-test); anything else (including
        // unset) → Networked (n0 relay + discovery) — the right choice for two
        // laptops over the internet on live Paseo.
        let transport_mode =
            policy::transport_mode_from(std::env::var("ODS_TRANSPORT").ok().as_deref());

        let unconfigured = OrgIo::open(&data_dir, &passphrase, transport_mode_for(transport_mode))?;

        // The chain: try to connect org-io from the environment; fall back to
        // a handle with the chain not configured. The endpoint is recorded
        // only when the connect succeeded, from the values handed to it, so
        // what is reported is what was built and cannot drift from it
        // (REQ-bvx4nh).
        let connected = match chain_settings() {
            // Connect on Tauri's PERSISTENT async runtime. This must NOT use a
            // temporary `tokio::runtime::Runtime` created here: the RPC client
            // spawns a background task for the WS connection, and if that task
            // is spawned on a throwaway runtime that is dropped when this
            // function returns, the task dies — so the first chain call made
            // later (e.g. submitting a genesis write) fails with "The client
            // was dropped". `tauri::async_runtime` lives for the whole app, and
            // it is the same runtime the command handlers use.
            Ok((settings, endpoint)) => tauri::async_runtime::block_on(unconfigured.connect(settings))
                .map(|org_io| (org_io, endpoint))
                .map_err(|refused| {
                    // A refused connect keeps the service untouched: the
                    // unconfigured handle is rebuilt from it, and the store
                    // is never opened a second time.
                    let reason = refused.reason.to_string();
                    (refused.into_not_configured(), reason)
                }),
            Err(e) => Err((unconfigured, e)),
        };
        let (org_io, chain_endpoint) = match connected {
            Ok((org_io, endpoint)) => (org_io, Some(endpoint)),
            Err((unconfigured, reason)) => {
                // Surface WHY chain mode didn't come up (missing or malformed
                // variable, or a failed connect) instead of silently degrading
                // — otherwise the UI just shows "Chain NOT configured" with no
                // diagnosable reason.
                eprintln!("[ods] chain config failed; running not configured: {reason}");
                (unconfigured, None)
            }
        };

        Self::assemble(org_io, data_dir, chain_endpoint, transport_mode)
    }

    /// Wire the handle and the outstanding Invites beside the store. Shared
    /// by `init` and `for_test` so the two cannot drift in how the state is
    /// built.
    fn assemble(
        org_io: OrgIo,
        data_dir: PathBuf,
        chain_endpoint: Option<policy::ChainEndpoint>,
        transport_mode: policy::TransportModeName,
    ) -> Result<Self, String> {
        let outstanding = OutstandingInvites::open(data_dir.join("outstanding_invites.json"))?;
        Ok(Self {
            org_io: Mutex::new(org_io),
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
    /// interfere with each other. The chain is not configured.
    #[cfg(feature = "test-support")]
    pub fn for_test(
        data_dir: PathBuf,
        passphrase: &str,
        chain_endpoint: Option<policy::ChainEndpoint>,
        transport_mode: policy::TransportModeName,
    ) -> Result<Self, String> {
        let org_io = OrgIo::open(&data_dir, passphrase, transport_mode_for(transport_mode))?;
        Self::assemble(org_io, data_dir, chain_endpoint, transport_mode)
    }
}

/// The chain settings the app hands `OrgIo::connect`, read from
/// `ODS_CHAIN_WS`, `ODS_CONTRACT_H160` and the optional `ODS_COSIGNER_PUB`,
/// with the `ChainEndpoint` they describe (REQ-bvx4nh). Returns `Err` if
/// either of the first two is absent or the address is malformed. The
/// co-signer is parsed by org-io; the signing seed is read by org-io.
fn chain_settings() -> Result<(ChainSettings, policy::ChainEndpoint), String> {
    let ws_url = std::env::var("ODS_CHAIN_WS").map_err(|_| "ODS_CHAIN_WS not set")?;
    let h160_hex =
        std::env::var("ODS_CONTRACT_H160").map_err(|_| "ODS_CONTRACT_H160 not set")?;

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

    let endpoint = policy::ChainEndpoint {
        ws_url: ws_url.clone(),
        contract_h160: h160_hex.clone(),
    };
    let settings = ChainSettings {
        ws_url,
        contract_h160,
        co_signer: std::env::var("ODS_COSIGNER_PUB").ok(),
    };
    Ok((settings, endpoint))
}
