//! Preflight CLI: run the transport health check before a real ODS session
//! and exit non-zero if it fails. Reads configuration from env vars (mirrors
//! the app's ODS_* convention).
//!
//!   ODS_DEVICE_SEED   0x 32-byte hex device seed              (default: all-zero)
//!   ODS_TRANSPORT     "networked" (default) | "loopback"
//!
//! The chain checks are org-io's preflight (change `worktree-org-io-create`).
#![cfg(feature = "transport")]

use std::time::Duration;

use org_node::DeviceSeed;
use org_node::preflight::{CheckResult, check_transport, render};
use org_node::transport::TransportMode;

/// The 32 bytes of a hex seed: 64 hex characters after at most one `0x`;
/// any other length or a non-hex character is refused.
fn seed_from_hex(text: &str) -> Option<[u8; 32]> {
    let digits = text.strip_prefix("0x").unwrap_or(text);
    // Hex digits only: `from_str_radix` alone would take a leading `+`.
    if digits.len() != 64 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut seed = [0u8; 32];
    for (byte, pair) in seed.iter_mut().zip(digits.as_bytes().chunks(2)) {
        let pair = std::str::from_utf8(pair).ok()?;
        *byte = u8::from_str_radix(pair, 16).ok()?;
    }
    Some(seed)
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let mut results: Vec<CheckResult> = Vec::new();

    let mode = match std::env::var("ODS_TRANSPORT").as_deref() {
        Ok("loopback") => TransportMode::Loopback,
        _ => TransportMode::Networked,
    };
    let seed = match std::env::var("ODS_DEVICE_SEED") {
        Ok(text) => match seed_from_hex(&text) {
            Some(seed) => seed,
            None => {
                eprintln!("ODS_DEVICE_SEED must be 32-byte hex");
                return std::process::ExitCode::FAILURE;
            }
        },
        Err(_) => [0u8; 32],
    };
    let device = DeviceSeed::from(seed).signing_keypair();
    results.push(check_transport(&device, mode, Duration::from_secs(15)).await);

    println!("\n--- ODS preflight ---");
    if render(&results) {
        println!("All checks passed.");
        std::process::ExitCode::SUCCESS
    } else {
        println!("One or more checks FAILED.");
        std::process::ExitCode::FAILURE
    }
}
