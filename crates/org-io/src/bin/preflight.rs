//! Preflight CLI: run the transport and
//! chain health checks before a real ODS session and exit non-zero if any
//! fail. Reads configuration from env vars (mirrors the app's ODS_*
//! convention).
//!
//!   ODS_DATA_DIR      the node's data directory, as the app's     (required)
//!   ODS_TRANSPORT     "networked" (default) | "loopback"
//!   ODS_CHAIN_WS      ws(s):// chain RPC URL                     (required)
//!   ODS_CONTRACT_H160 0x-prefixed 20-byte contract address       (required)
//!   ODS_ADMIN_H160    0x-prefixed 20-byte admin/org id            (required)
//!
//! The transport check (owner rulings 7 and 12) binds the
//! endpoint of every Persona whose store file is in the data directory,
//! each through its own OS keychain item, as the node does. No variable
//! names or carries a device key or seed: the data directory is a path, not
//! key material, and it has no default (owner ruling 2026-10-08).

use std::time::Duration;

use org_io::connect::connect;
use org_io::preflight::{check_chain_live, check_contract, check_transport_for_data_dir, render, CheckResult};
use org_io::transport::TransportMode;

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

    let data_dir = match std::env::var("ODS_DATA_DIR") {
        Ok(value) => std::path::PathBuf::from(value),
        Err(std::env::VarError::NotPresent) => {
            eprintln!("ODS_DATA_DIR not set");
            return std::process::ExitCode::FAILURE;
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            eprintln!("ODS_DATA_DIR is not valid Unicode");
            return std::process::ExitCode::FAILURE;
        }
    };
    let mode = match std::env::var("ODS_TRANSPORT").as_deref() {
        Ok("loopback") => TransportMode::Loopback,
        _ => TransportMode::Networked,
    };
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

    results.extend(check_transport_for_data_dir(&data_dir, mode, Duration::from_secs(15)).await);

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
