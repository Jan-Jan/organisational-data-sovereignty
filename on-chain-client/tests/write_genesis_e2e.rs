#![cfg(all(feature = "write", feature = "dev-rpc"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Chopsticks, outside every gate: the chain writer against a Paseo-AH fork.
//! Ported from the write half of org-node's `single_admin_genesis_e2e`.
//! Run: pkill -f "chopsticks.*--config"; CARGO_HOME=/tmp/cargo_home_fuzz \
//!   cargo test --manifest-path on-chain-client/Cargo.toml --features write \
//!   --test write_genesis_e2e -- --test-threads=1 --nocapture

mod common;

use std::process::Command;

use common::chopsticks_fork::{spawn_fork, ChopsticksHandle};
use common::chopsticks_reorg::mine_block;
use common::conn::legacy_client;
use on_chain_client::write::subxt_ops::{BlockSink, SubxtWriteOps};
use on_chain_client::write::{genesis, submit_update, WriteError};
use on_chain_client::{Epoch, OnChainRootHash, OrgPubKey, OrgRegistryClient};
use subxt_signer::sr25519::dev;

struct ChopsticksSink<'a> {
    handle: &'a ChopsticksHandle,
}

impl BlockSink for ChopsticksSink<'_> {
    async fn settle(&self) -> Result<(), WriteError> {
        mine_block(self.handle).await.map(|_| ()).map_err(|e| WriteError::Subxt(format!("{e:?}")))
    }
}

/// Deploy OrgRegistry onto the running chopsticks fork via the EVM path
/// (`instantiateWithCode` accepts EVM creation bytecode — same VM backend as the
/// live deploy; no resolc/PVM). Returns the deployed contract H160. Paths are
/// resolved relative to this crate's manifest dir (up to on-chain/).
fn deploy_org_registry() -> [u8; 20] {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let on_chain_dir = std::path::PathBuf::from(&manifest_dir).join("../on-chain");

    // Build the EVM artifact (out/OrgRegistry.sol/OrgRegistry.json) with forge's
    // pinned solc 0.8.27. Idempotent; skips compilation if already up to date.
    let build = Command::new("forge")
        .arg("build")
        .current_dir(&on_chain_dir)
        .output()
        .expect("spawn forge build (is Foundry installed?)");
    assert!(
        build.status.success(),
        "forge build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let output = Command::new("node")
        .arg("scripts/deploy-chopsticks-evm.mjs")
        .current_dir(&on_chain_dir)
        .env("RPC_URL", "ws://localhost:8000")
        .output()
        .expect("spawn deploy-chopsticks-evm.mjs");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("--- deploy-chopsticks-evm stdout ---\n{stdout}--- end stdout ---");
    if !stderr.is_empty() {
        eprintln!("--- deploy-chopsticks-evm stderr ---\n{stderr}--- end stderr ---");
    }
    assert!(output.status.success(), "deploy-chopsticks-evm.mjs exited non-zero");

    let marker_line = stdout
        .lines()
        .find(|l| l.starts_with("DEPLOYED_H160="))
        .expect("DEPLOYED_H160= marker not found in deploy output");
    let hex_str = marker_line
        .trim_start_matches("DEPLOYED_H160=")
        .trim_start_matches("0x");
    let bytes = hex::decode(hex_str).expect("decode H160 hex");
    let mut h160 = [0u8; 20];
    assert_eq!(bytes.len(), 20, "deployed H160 was not 20 bytes");
    h160.copy_from_slice(&bytes);
    h160
}

#[tokio::test(flavor = "multi_thread")]
async fn single_signatory_genesis_then_update_moves_the_slot() {
    let fork = spawn_fork().await.expect("spawn chopsticks fork");
    let contract = deploy_org_registry();
    let api = legacy_client(&fork.ws_url).await.expect("legacy subxt client");
    let ops = SubxtWriteOps::new(api.clone(), ChopsticksSink { handle: &fork });
    let alice = dev::alice();

    let g = genesis(&ops, &alice, &[], contract, OnChainRootHash([0x33; 32]), OrgPubKey([0x22; 32]))
        .await
        .expect("genesis");
    let reader = OrgRegistryClient::from_client(api.clone(), contract).await.expect("reader");
    let state = reader.get_org_state(g.admin, None).await.expect("read").expect("slot");
    assert_eq!((state.root_hash, state.epoch), (OnChainRootHash([0x33; 32]), Epoch(1)));

    submit_update(&ops, &alice, &[], contract, g.proxy, OnChainRootHash([0x66; 32]), OrgPubKey([0x22; 32]), Epoch(1))
        .await
        .expect("update");
    let state = reader.get_org_state(g.admin, None).await.expect("read").expect("slot");
    assert_eq!((state.root_hash, state.epoch), (OnChainRootHash([0x66; 32]), Epoch(2)));
}
