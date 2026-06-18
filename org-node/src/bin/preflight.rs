//! Preflight CLI: run health checks before a real ODS session and exit
//! non-zero if any fail. Reads configuration from env vars (mirrors the app's
//! ODS_* convention).
//!
//!   ODS_CHAIN_WS      ws(s):// chain RPC URL                 (required)
//!   ODS_CONTRACT_H160 0x-prefixed 20-byte contract address   (required)
//!   ODS_ADMIN_H160    0x-prefixed 20-byte admin/org id        (required)
//!   ODS_DEVICE_SEED   0x 32-byte hex device seed              (default: all-zero)
//!   ODS_TRANSPORT     "networked" (default) | "loopback"
#![cfg(feature = "app")]

use std::time::Duration;

use org_node::keys::SigningKeypair;
use org_node::preflight::{CheckResult, check_chain_live, check_contract, check_transport, render};
use org_node::transport::TransportMode;

fn h160_from_env(var: &str) -> Result<[u8; 20], String> {
    let s = std::env::var(var).map_err(|_| format!("{var} not set"))?;
    let hex = s.strip_prefix("0x").unwrap_or(&s);
    let bytes = hex::decode(hex).map_err(|e| format!("{var}: bad hex: {e}"))?;
    let arr: [u8; 20] = bytes
        .try_into()
        .map_err(|_| format!("{var}: expected 20 bytes"))?;
    Ok(arr)
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let mut results: Vec<CheckResult> = Vec::new();

    // Transport.
    let mode = match std::env::var("ODS_TRANSPORT").as_deref() {
        Ok("loopback") => TransportMode::Loopback,
        _ => TransportMode::Networked,
    };
    let seed = match std::env::var("ODS_DEVICE_SEED") {
        Ok(s) => {
            let hex = s.strip_prefix("0x").unwrap_or(&s).to_string();
            match hex::decode(&hex).ok().and_then(|b| <[u8; 32]>::try_from(b).ok()) {
                Some(arr) => arr,
                None => {
                    eprintln!("ODS_DEVICE_SEED must be 32-byte hex");
                    return std::process::ExitCode::FAILURE;
                }
            }
        }
        Err(_) => [0u8; 32],
    };
    let device = SigningKeypair::from_seed(seed);
    results.push(check_transport(&device, mode, Duration::from_secs(15)).await);

    // Chain.
    let ws = match std::env::var("ODS_CHAIN_WS") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("ODS_CHAIN_WS not set");
            return std::process::ExitCode::FAILURE;
        }
    };
    let contract = match h160_from_env("ODS_CONTRACT_H160") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    match org_node::service::connect_chain_client(&ws, contract).await {
        Ok((api, registry)) => {
            results.push(check_chain_live(&api, Duration::from_secs(15)).await);
            match h160_from_env("ODS_ADMIN_H160") {
                Ok(admin) => {
                    results.push(check_contract(&registry, on_chain_client::OrgAdmin(admin)).await);
                }
                Err(e) => results.push(CheckResult {
                    name: "contract.config".into(),
                    ok: false,
                    detail: e,
                }),
            }
        }
        Err(e) => results.push(CheckResult {
            name: "chain.connect".into(),
            ok: false,
            detail: format!("connect failed: {e}"),
        }),
    }

    println!("\n--- ODS preflight ---");
    if render(&results) {
        println!("All checks passed.");
        std::process::ExitCode::SUCCESS
    } else {
        println!("One or more checks FAILED.");
        std::process::ExitCode::FAILURE
    }
}
