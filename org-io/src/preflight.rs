//! Preflight health checks for an ODS node before a real (two-laptop / live
//! Paseo) session, moved from org-node 2026-10-07 (SDD-z85ux9). Each check
//! returns a structured [`CheckResult`] instead of panicking, so the binary
//! can print a full checklist and exit non-zero if any fail. The functions are
//! also exercised hermetically (chopsticks) by `tests/preflight.rs`.
//!
//! The chain checks only. org-node's transport check binds an endpoint under a
//! device signing key, which org-io must not hold (LLR-3zdw8v); it stays with
//! org-node's transport until the owner rules where it goes.

use std::time::Duration;

use subxt::config::PolkadotConfig;
use subxt::OnlineClient;

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
