//! Startup policy and the connection-status projection.
//!
//! Every function here is total and takes its inputs as parameters: none reads
//! the process environment. The caller (`AppState::init`) does the reading.
//! That is deliberate — cargo runs integration tests in threads of one
//! process, so a test that set an environment variable would race every other
//! test in the binary.

use std::path::PathBuf;
use std::time::Duration;

/// Which of the two ways this installation reaches other nodes. Mirrors
/// org-io's `transport::TransportMode`, but is this unit's own type because it
/// is serialised across the IPC boundary and org-io's is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportModeName {
    Networked,
    Loopback,
}

/// The chain endpoint the running configuration was actually built from.
/// Present only when `OrgIo::connect` succeeded with it (REQ-bvx4nh).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainEndpoint {
    pub ws_url: String,
    /// The contract address as the operator supplied it, 0x-prefixed hex.
    pub contract_h160: String,
}

/// A startup value that is missing and is not permitted to be defaulted,
/// or one that is not of its form. No passphrase is among them: org-io
/// opens each store with its keychain item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupError {
    UnresolvableDataDir,
    /// LLR-rvrc25: `ODS_VERIFICATION_INTERVAL_SECS` as given, refused.
    InvalidVerificationInterval { given: String },
}

impl core::fmt::Display for StartupError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // REQ-bmk2z2: the message names BOTH the variable that would supply the
        // value AND the variable that would waive the requirement, and names the
        // supplying variable FIRST — a refusal an operator reads as "set the
        // waiver" is a refusal that recreates the hazard it exists to prevent.
        match self {
            StartupError::UnresolvableDataDir => write!(
                f,
                "The application data directory could not be resolved and \
                 ODS_DATA_DIR is not set. Set ODS_DATA_DIR to the directory \
                 this installation should keep its persona store in. To fall \
                 back to a temporary directory instead, set \
                 ODS_ALLOW_DEV_DEFAULTS=1 — that directory is world-readable \
                 and is cleared by the operating system."
            ),
            // LLR-rvrc25: the variable, the form, and the value quoted.
            StartupError::InvalidVerificationInterval { given } => write!(
                f,
                "ODS_VERIFICATION_INTERVAL_SECS must be a whole number of seconds of at least 1 \
                 (at most 4294967295), got {given:?}"
            ),
        }
    }
}

/// LLR-2dpn52: the verification interval when none is configured.
pub const DEFAULT_VERIFICATION_INTERVAL: Duration = Duration::from_secs(600);

/// REQ-ergn9z. `raw` is `ODS_VERIFICATION_INTERVAL_SECS`. Absent or empty
/// is the 10-minute default (LLR-2dpn52); a decimal integer from 1 to
/// 4294967295, with no sign, space or other character, is that many
/// seconds; anything else is refused quoting the value (LLR-rvrc25). What
/// leaves here is never zero.
pub fn resolve_verification_interval(raw: Option<&str>) -> Result<Duration, StartupError> {
    let given = match raw {
        None | Some("") => return Ok(DEFAULT_VERIFICATION_INTERVAL),
        Some(given) => given,
    };
    let refused = || StartupError::InvalidVerificationInterval { given: given.to_string() };
    if !given.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(refused());
    }
    let seconds: u32 = given.parse().map_err(|_| refused())?;
    if seconds == 0 {
        return Err(refused());
    }
    Ok(Duration::from_secs(u64::from(seconds)))
}

/// LLR-hgrez9: the ahead limit the app gives org-io for every verification
/// (`ChainSettings::ahead_limit`): how long received Organisation
/// information one epoch ahead of the chain is kept unadopted. Twice the
/// verification interval, so at least two chain reads fall inside it
/// whatever the interval; the default interval gives 1200 seconds. An
/// interval `resolve_verification_interval` accepts (at most 4294967295
/// seconds) never overflows when doubled; a greater one saturates.
pub fn verification_ahead_limit(verification_interval: Duration) -> Duration {
    verification_interval.saturating_mul(2)
}

/// REQ-ergn9z, LLR-rvrc25. `raw` is `ODS_VERIFICATION_INTERVAL_SECS` as the
/// environment holds it. A value that is not UTF-8 does not parse, so it is
/// refused, carrying its lossy form, and never read as absent; a UTF-8 value
/// is decided by `resolve_verification_interval`.
pub fn verification_interval_from(raw: Option<&std::ffi::OsStr>) -> Result<Duration, StartupError> {
    match raw.map(|value| value.to_str().ok_or(value)) {
        None => resolve_verification_interval(None),
        Some(Ok(text)) => resolve_verification_interval(Some(text)),
        Some(Err(value)) => {
            Err(StartupError::InvalidVerificationInterval { given: value.to_string_lossy().into_owned() })
        }
    }
}

/// The built-in data directory fallback.
pub const DEV_DATA_DIR: &str = "/tmp/ods-poc";

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
