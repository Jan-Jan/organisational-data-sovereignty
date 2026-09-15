//! Startup policy and the connection-status projection.
//!
//! Every function here is total and takes its inputs as parameters: none reads
//! the process environment. The caller (`AppState::init`) does the reading.
//! That is deliberate — cargo runs integration tests in threads of one
//! process, so a test that set an environment variable would race every other
//! test in the binary.

use std::path::PathBuf;

/// Which of the two ways this installation reaches other nodes. Mirrors
/// org_node::transport::TransportMode, but is this unit's own type because it
/// is serialised across the IPC boundary and org-node's is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportModeName {
    Networked,
    Loopback,
}

/// The chain endpoint the running configuration was actually built from.
/// Present only when `build_chain_ops` succeeded (REQ-bvx4nh).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainEndpoint {
    pub ws_url: String,
    /// The contract address as the operator supplied it, 0x-prefixed hex.
    pub contract_h160: String,
}

/// A startup value that is missing and is not permitted to be defaulted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupError {
    MissingPassphrase,
    UnresolvableDataDir,
}

impl core::fmt::Display for StartupError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // REQ-bmk2z2: the message names BOTH the variable that would supply the
        // value AND the variable that would waive the requirement, and names the
        // supplying variable FIRST — a refusal an operator reads as "set the
        // waiver" is a refusal that recreates the hazard it was added for.
        match self {
            StartupError::MissingPassphrase => write!(
                f,
                "ODS_PASSPHRASE is not set. Set it to the passphrase protecting \
                 this installation's persona store. To run with the built-in \
                 development passphrase instead, set ODS_ALLOW_DEV_DEFAULTS=1 — \
                 do not do this on an installation holding real key material."
            ),
            StartupError::UnresolvableDataDir => write!(
                f,
                "The application data directory could not be resolved and \
                 ODS_DATA_DIR is not set. Set ODS_DATA_DIR to the directory \
                 this installation should keep its persona store in. To fall \
                 back to a temporary directory instead, set \
                 ODS_ALLOW_DEV_DEFAULTS=1 — that directory is world-readable \
                 and is cleared by the operating system."
            ),
        }
    }
}

/// The built-in passphrase. Public so the test can assert on the exact value
/// rather than restating the literal, and so a future reader greps one place.
pub const DEV_PASSPHRASE: &str = "ods-dev-default";

/// The built-in data directory fallback.
pub const DEV_DATA_DIR: &str = "/tmp/ods-poc";

/// REQ-7g3k9a. `configured` is `ODS_PASSPHRASE`; `allow_dev_defaults` is
/// whether `ODS_ALLOW_DEV_DEFAULTS` was set.
pub fn resolve_passphrase(
    configured: Option<&str>,
    allow_dev_defaults: bool,
) -> Result<String, StartupError> {
    match configured {
        // An empty ODS_PASSPHRASE is treated as absent, not as an empty
        // passphrase: a variable set to "" is overwhelmingly a scripting
        // accident, and honouring it would be the original hazard with an
        // extra step.
        Some(p) if !p.is_empty() => Ok(p.to_string()),
        _ if allow_dev_defaults => Ok(DEV_PASSPHRASE.to_string()),
        _ => Err(StartupError::MissingPassphrase),
    }
}

/// REQ-rxc8sp. `override_dir` is `ODS_DATA_DIR`; `resolved` is what the
/// platform path resolver returned (`None` on error).
pub fn resolve_data_dir(
    override_dir: Option<&str>,
    resolved: Option<PathBuf>,
    allow_dev_defaults: bool,
) -> Result<PathBuf, StartupError> {
    if let Some(d) = override_dir.filter(|d| !d.is_empty()) {
        return Ok(PathBuf::from(d));
    }
    if let Some(p) = resolved {
        return Ok(p);
    }
    if allow_dev_defaults {
        return Ok(PathBuf::from(DEV_DATA_DIR));
    }
    Err(StartupError::UnresolvableDataDir)
}

/// The transport mode, from `ODS_TRANSPORT`. Anything other than the literal
/// "loopback" is Networked, preserving the existing behaviour exactly.
pub fn transport_mode_from(raw: Option<&str>) -> TransportModeName {
    match raw {
        Some("loopback") => TransportModeName::Loopback,
        _ => TransportModeName::Networked,
    }
}

/// Connection status reported by the `connection_status` command.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConnectionStatus {
    /// True only when a real chain client was constructed at startup.
    /// This is a startup fact, not a liveness fact — see PR-eecx3y.
    pub chain_configured: bool,
    /// REQ-e4ah9h / REQ-bvx4nh: the endpoint the running configuration was
    /// built from, or absent. Never re-read from the environment.
    pub chain_ws: Option<String>,
    pub contract_h160: Option<String>,
    /// REQ-645jq9.
    pub transport_mode: TransportModeName,
    pub data_dir: String,
}

/// REQ-e4ah9h, REQ-bvx4nh, REQ-645jq9.
pub fn connection_status_from_state(
    data_dir: &std::path::Path,
    chain: Option<&ChainEndpoint>,
    transport_mode: TransportModeName,
) -> ConnectionStatus {
    ConnectionStatus {
        chain_configured: chain.is_some(),
        chain_ws: chain.map(|c| c.ws_url.clone()),
        contract_h160: chain.map(|c| c.contract_h160.clone()),
        transport_mode,
        data_dir: data_dir.display().to_string(),
    }
}
