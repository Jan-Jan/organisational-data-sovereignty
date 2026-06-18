//! Preflight health checks for an ODS node before a real (two-laptop / live
//! Paseo) session. Each check returns a structured [`CheckResult`] instead of
//! panicking, so the binary can print a full checklist and exit non-zero if
//! any fail. The functions are also exercised hermetically (local relay +
//! chopsticks) by the test suite.
#![cfg(feature = "app")]

use std::time::Duration;

use subxt::OnlineClient;
use subxt::config::PolkadotConfig;

use crate::keys::SigningKeypair;
use crate::transport::{TransportMode, endpoint::OrgEndpoint};

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

/// Bind an endpoint in `mode` and confirm it acquires a reachable address.
/// For `Networked`, waits up to `online_timeout` for a relay home;
/// `online_timeout` is unused for `Loopback` (which only checks the bound socket).
pub async fn check_transport(
    device: &SigningKeypair,
    mode: TransportMode,
    online_timeout: Duration,
) -> CheckResult {
    let ep = match OrgEndpoint::bind_with_mode(device, mode).await {
        Ok(ep) => ep,
        Err(e) => return CheckResult::fail("transport.bind", format!("bind failed: {e}")),
    };
    match mode {
        TransportMode::Loopback => {
            // bound_sockets() is exactly what node_addr_for_dial() builds from.
            if !ep.inner().bound_sockets().is_empty() {
                CheckResult::ok("transport.bind", "loopback endpoint bound with a direct socket")
            } else {
                CheckResult::fail("transport.bind", "loopback endpoint has no bound socket")
            }
        }
        TransportMode::Networked => {
            match tokio::time::timeout(online_timeout, ep.inner().online()).await {
                Ok(()) => {
                    let addr = ep.inner().addr();
                    if addr.relay_urls().next().is_some() {
                        CheckResult::ok("transport.online", "acquired a relay home")
                    } else {
                        CheckResult::fail(
                            "transport.online",
                            "came online but no relay home was assigned",
                        )
                    }
                }
                Err(_) => CheckResult::fail(
                    "transport.online",
                    format!("did not come online within {online_timeout:?}"),
                ),
            }
        }
    }
}

/// Confirm the chain RPC is reachable AND making progress: sample the current
/// finalized hash twice (with a delay) and require it to advance.
pub async fn check_chain_live(
    api: &OnlineClient<PolkadotConfig>,
    sample_gap: Duration,
) -> CheckResult {
    let first = match api.at_current_block().await {
        Ok(b) => b.block_ref().hash().0,
        Err(e) => return CheckResult::fail("chain.rpc", format!("at_current_block failed: {e}")),
    };
    tokio::time::sleep(sample_gap).await;
    let second = match api.at_current_block().await {
        Ok(b) => b.block_ref().hash().0,
        Err(e) => return CheckResult::fail("chain.rpc", format!("second sample failed: {e}")),
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
        Ok(Some(state)) => CheckResult::ok(
            "contract.query",
            format!("get_org_state ok (epoch {})", state.epoch.0),
        ),
        Ok(None) => CheckResult::ok(
            "contract.query",
            "get_org_state ok (slot uninitialised — contract reachable)",
        ),
        Err(e) => CheckResult::fail("contract.query", format!("get_org_state failed: {e}")),
    }
}

/// Render a checklist and return whether all checks passed.
pub fn render(results: &[CheckResult]) -> bool {
    let mut all_ok = true;
    for r in results {
        let mark = if r.ok { "PASS" } else { "FAIL" };
        if !r.ok {
            all_ok = false;
        }
        println!("[{mark}] {:<22} {}", r.name, r.detail);
    }
    all_ok
}
