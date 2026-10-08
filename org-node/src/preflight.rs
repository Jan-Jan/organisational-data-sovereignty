//! Preflight health check of an ODS node's transport before a real
//! (two-laptop) session. Each check returns a structured [`CheckResult`]
//! instead of panicking, so the binary can print a full checklist and exit
//! non-zero if any fail. The function is also exercised hermetically (local
//! relay) by the test suite.
//!
//! The chain checks moved to org-io with the chain connection (change
//! `worktree-org-io-create`); this transport check stays here until S4 moves
//! the iroh binding, with the device seed, into org-io.
#![cfg(feature = "transport")]

use std::time::Duration;

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
            if !ep.bound_sockets().is_empty() {
                CheckResult::ok("transport.bind", "loopback endpoint bound with a direct socket")
            } else {
                CheckResult::fail("transport.bind", "loopback endpoint has no bound socket")
            }
        }
        TransportMode::Networked => {
            match tokio::time::timeout(online_timeout, ep.online()).await {
                Ok(()) => {
                    let addr = ep.addr();
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
