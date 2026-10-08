//! The chain connection (SDD-z85ux9, moved from org-node's
//! `service::connect_chain_client` 2026-10-07).

use core::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use org_node::service::OrgService;
use org_node::{OrgId, OrgNodeError, OrgState};

use crate::chain_read::StateReader;
use crate::custody::ConfigError;

/// The report of a build without `dev-seed` (REQ-8zuka3, LLR-rgdx22).
pub(crate) const DEV_SEED_NOT_BUILT: &str =
    "chain not configured: the signing seed (ODS_ADMIN_SEED) is read only by development builds (dev-seed)";

/// Why `OrgIo::connect` refused (LLR-rgdx22).
#[derive(Debug, PartialEq, Eq)]
pub enum ConnectFailure {
    /// Built without `dev-seed`: no signing seed is read (LLR-rgdx22).
    DevSeedNotBuilt,
    /// `ODS_ADMIN_SEED` is unset.
    SeedNotSet,
    /// A configuration value was refused (the seed or the co-signer).
    Config(ConfigError),
    /// The co-signer is the account of the node's own signatory key
    /// (LLR-qhyc3n: the controller and the writer would disagree).
    CoSignerIsOwnAccount,
    /// The chain connection at `ws_url` failed.
    Chain { ws_url: String, error: OrgNodeError },
}

impl fmt::Display for ConnectFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectFailure::DevSeedNotBuilt => formatter.write_str(DEV_SEED_NOT_BUILT),
            ConnectFailure::SeedNotSet => formatter.write_str(crate::submit::NOT_CONFIGURED),
            ConnectFailure::Config(error) => write!(formatter, "{error}"),
            ConnectFailure::CoSignerIsOwnAccount => {
                formatter.write_str("ODS_COSIGNER_PUB is this node's own account; a co-signer must be another account")
            }
            ConnectFailure::Chain { ws_url, error } => write!(formatter, "connect_chain {ws_url}: {error}"),
        }
    }
}

impl std::error::Error for ConnectFailure {}

/// A refused `OrgIo::connect`: why (LLR-rgdx22), and the service the handle
/// held, kept untouched and handed back only as an unconfigured handle
/// (`into_not_configured`), so the caller does not open the store again and
/// never holds org-node's service itself (review round 2, finding 2).
/// Returned boxed: the service is large.
pub struct ConnectFailed {
    service: OrgService,
    pub reason: ConnectFailure,
}

impl ConnectFailed {
    pub(crate) fn boxed(service: OrgService, reason: ConnectFailure) -> Box<Self> {
        Box::new(Self { service, reason })
    }

    /// The handle with the chain not configured, over the service the
    /// refused connect was given.
    pub fn into_not_configured(self) -> crate::OrgIo {
        crate::OrgIo::not_configured(self.service)
    }
}

/// The read when the chain is not configured (the app's former
/// `ChainNotConfigured`): every read refused with the message that says what
/// to set. Its write half is `submit::WriterNotConfigured`.
pub struct StateReaderNotConfigured;

#[async_trait]
impl StateReader for StateReaderNotConfigured {
    async fn read_state(&self, _org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        Err(OrgNodeError::Chain(crate::submit::NOT_CONFIGURED.into()))
    }
}

/// Connect to a chain RPC over an explicit `LegacyBackend` (required for
/// chopsticks; mirrors tests/common/conn.rs) and build an `OrgRegistryClient`
/// for `contract`. Used by the preflight binary and by `OrgIo::connect`,
/// which hands the `OnlineClient` to on-chain-client's writer. Returns both
/// so callers can run raw-chain checks (the `OnlineClient`) and contract
/// checks (the `OrgRegistryClient`).
pub async fn connect(
    ws_url: &str,
    contract: [u8; 20],
) -> Result<(subxt::OnlineClient<subxt::config::PolkadotConfig>, on_chain_client::OrgRegistryClient), OrgNodeError> {
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
        .map_err(|error| OrgNodeError::Chain(format!("rpc connect: {error}")))?;
    let rpc_client = subxt::rpcs::RpcClient::new(reconnecting);
    let backend: LegacyBackend<PolkadotConfig> = LegacyBackend::builder().build(rpc_client);
    let api = subxt::OnlineClient::from_backend(Arc::new(backend))
        .await
        .map_err(|error| OrgNodeError::Chain(format!("online client: {error}")))?;
    let registry = on_chain_client::OrgRegistryClient::from_client(api.clone(), contract)
        .await
        .map_err(|error| OrgNodeError::Chain(format!("registry client: {error}")))?;
    Ok((api, registry))
}
