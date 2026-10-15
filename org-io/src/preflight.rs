//! Preflight health checks for an ODS node before a real (two-laptop / live
//! Paseo) session, moved from org-node 2026-10-07 (SDD-z85ux9). Each check
//! returns a structured [`CheckResult`] instead of panicking, so the binary
//! can print a full checklist and exit non-zero if any fail. The functions are
//! also exercised hermetically (chopsticks) by `tests/preflight.rs`.
//!
//! The transport check (owner rulings 7 and 12) sits beside the chain
//! checks and takes the keychain path (LLR-h9tcat): it binds each Device's
//! endpoint through its keychain item (`OrgEndpoint::bind_from_keychain`),
//! and the Device keys come from the data directory's store files, as the
//! node finds them. No key or seed is read from the environment.

use std::path::Path;
use std::time::Duration;

use org_node::DevicePublicKey;
use subxt::config::PolkadotConfig;
use subxt::OnlineClient;

use crate::store_file::list_stores;
use crate::transport::{OrgEndpoint, TransportMode};

/// Outcome of a single preflight check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

impl CheckResult {
    fn ok(name: &str, detail: impl Into<String>) -> Self {
        Self { name: name.into(), ok: true, detail: detail.into() }
    }
    fn fail(name: &str, detail: impl Into<String>) -> Self {
        Self { name: name.into(), ok: false, detail: detail.into() }
    }
}

/// Confirm the chain RPC is reachable AND making progress: sample the current
/// finalized hash twice (with a delay) and require it to advance.
pub async fn check_chain_live(api: &OnlineClient<PolkadotConfig>, sample_gap: Duration) -> CheckResult {
    let first = match api.at_current_block().await {
        Ok(block) => block.block_ref().hash().0,
        Err(error) => return CheckResult::fail("chain.rpc", format!("at_current_block failed: {error}")),
    };
    tokio::time::sleep(sample_gap).await;
    let second = match api.at_current_block().await {
        Ok(block) => block.block_ref().hash().0,
        Err(error) => return CheckResult::fail("chain.rpc", format!("second sample failed: {error}")),
    };
    if first != second {
        CheckResult::ok("chain.live", "finalized head advanced between samples")
    } else {
        CheckResult::fail(
            "chain.live",
            format!("finalized head did not advance within {sample_gap:?} (chain stalled?)"),
        )
    }
}

/// Confirm the OrgRegistry contract is queryable at the configured admin: the
/// client constructs (metadata exposes the revive pallet) and `get_org_state`
/// returns without error. NOTE: this proves the RPC + pallet + contract
/// address are reachable; it does NOT re-verify the deployed bytecode hash
/// (the deploy script already does that at deploy time).
///
/// Reads at the latest finalised block (`at = None`). A successful query for a
/// never-written slot (`Ok(None)`) still proves reachability and so PASSes —
/// the connectivity, not the org's existence, is what preflight verifies.
pub async fn check_contract(
    client: &on_chain_client::OrgRegistryClient,
    admin: on_chain_client::OrgAdmin,
) -> CheckResult {
    match client.get_org_state(admin, None).await {
        Ok(Some(state)) => CheckResult::ok("contract.query", format!("get_org_state ok (epoch {})", state.epoch.0)),
        Ok(None) => CheckResult::ok("contract.query", "get_org_state ok (slot uninitialised — contract reachable)"),
        Err(error) => CheckResult::fail("contract.query", format!("get_org_state failed: {error}")),
    }
}

/// Bind `device_key`'s endpoint in `mode` through its keychain item and
/// confirm it acquires a reachable address (LLR-h9tcat): for `Loopback`, a
/// bound socket; for `Networked`, a relay home within `online_timeout`
/// (unused for `Loopback`). A keychain refusal or a missing item is one
/// FAIL row naming the DevicePublicKey, and nothing is bound. Every row names
/// the DevicePublicKey.
pub async fn check_transport(device_key: &DevicePublicKey, mode: TransportMode, online_timeout: Duration) -> CheckResult {
    let device = hex::encode(device_key.as_bytes());
    let endpoint = match OrgEndpoint::bind_from_keychain(device_key, mode.into()).await {
        Ok(endpoint) => endpoint,
        Err(error) => return CheckResult::fail("transport.bind", format!("DevicePublicKey {device}: bind failed: {error}")),
    };
    match mode {
        TransportMode::Loopback => {
            // bound_sockets() is exactly what node_addr_for_dial() builds from.
            if endpoint.bound_sockets().is_empty() {
                CheckResult::fail("transport.bind", format!("DevicePublicKey {device}: loopback endpoint has no bound socket"))
            } else {
                CheckResult::ok("transport.bind", format!("DevicePublicKey {device}: loopback endpoint bound with a direct socket"))
            }
        }
        TransportMode::Networked => match tokio::time::timeout(online_timeout, endpoint.online()).await {
            Ok(()) if endpoint.has_relay_home() => {
                CheckResult::ok("transport.online", format!("DevicePublicKey {device}: acquired a relay home"))
            }
            Ok(()) => CheckResult::fail(
                "transport.online",
                format!("DevicePublicKey {device}: came online but no relay home was assigned"),
            ),
            Err(_) => CheckResult::fail(
                "transport.online",
                format!("DevicePublicKey {device}: did not come online within {online_timeout:?}"),
            ),
        },
    }
}

/// `check_transport` for every DevicePublicKey whose store file is in
/// `data_dir`, as the node lists them (LLR-nt8dja): one row per DevicePublicKey,
/// each bound through its own keychain item. A directory with no store file,
/// or one that cannot be read, is one FAIL row naming the directory, and
/// nothing is bound.
pub async fn check_transport_for_data_dir(data_dir: &Path, mode: TransportMode, online_timeout: Duration) -> Vec<CheckResult> {
    let device_keys = match list_stores(data_dir) {
        Ok(device_keys) => device_keys,
        Err(error) => {
            return vec![CheckResult::fail("transport.data_dir", format!("{}: {error}", data_dir.display()))];
        }
    };
    if device_keys.is_empty() {
        return vec![CheckResult::fail("transport.data_dir", format!("{}: no store file", data_dir.display()))];
    }
    let mut results = Vec::with_capacity(device_keys.len());
    for device_key in &device_keys {
        results.push(check_transport(device_key, mode, online_timeout).await);
    }
    results
}

/// Render a checklist and return whether all checks passed.
pub fn render(results: &[CheckResult]) -> bool {
    let mut all_ok = true;
    for result in results {
        let mark = if result.ok { "PASS" } else { "FAIL" };
        if !result.ok {
            all_ok = false;
        }
        println!("[{mark}] {:<22} {}", result.name, result.detail);
    }
    all_ok
}
