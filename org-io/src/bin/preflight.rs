//! Preflight CLI (moved from org-node 2026-10-07): run the chain health checks
//! before a real ODS session and exit non-zero if any fail. Reads
//! configuration from env vars (mirrors the app's ODS_* convention).
//!
//!   ODS_CHAIN_WS      ws(s):// chain RPC URL                 (required)
//!   ODS_CONTRACT_H160 0x-prefixed 20-byte contract address   (required)
//!   ODS_ADMIN_H160    0x-prefixed 20-byte admin/org id        (required)
//!
//! The transport check (ODS_DEVICE_SEED, ODS_TRANSPORT) is not here: it needs
//! a device signing key, which org-io does not hold (LLR-3zdw8v).

use std::time::Duration;

use org_io::connect::connect;
use org_io::preflight::{check_chain_live, check_contract, render, CheckResult};

fn h160_from_env(var: &str) -> Result<[u8; 20], String> {
    let text = std::env::var(var).map_err(|_| format!("{var} not set"))?;
    let digits = text.strip_prefix("0x").unwrap_or(&text);
    let bytes = hex::decode(digits).map_err(|error| format!("{var}: bad hex: {error}"))?;
    let address: [u8; 20] = bytes.try_into().map_err(|_| format!("{var}: expected 20 bytes"))?;
    Ok(address)
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let mut results: Vec<CheckResult> = Vec::new();

    let ws = match std::env::var("ODS_CHAIN_WS") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("ODS_CHAIN_WS not set");
            return std::process::ExitCode::FAILURE;
        }
    };
    let contract = match h160_from_env("ODS_CONTRACT_H160") {
        Ok(contract) => contract,
        Err(error) => {
            eprintln!("{error}");
            return std::process::ExitCode::FAILURE;
        }
    };
    match connect(&ws, contract).await {
        Ok((api, registry)) => {
            results.push(check_chain_live(&api, Duration::from_secs(15)).await);
            match h160_from_env("ODS_ADMIN_H160") {
                Ok(admin) => {
                    results.push(check_contract(&registry, on_chain_client::OrgAdmin(admin)).await);
                }
                Err(error) => results.push(CheckResult { name: "contract.config".into(), ok: false, detail: error }),
            }
        }
        Err(error) => results.push(CheckResult {
            name: "chain.connect".into(),
            ok: false,
            detail: format!("connect failed: {error}"),
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
