#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Chopsticks, outside every gate: the founding
//! contract `tests/chain_write.rs` runs over `test_support::FakeChain`
//! (`common::chain_writer_contract`), run here over the production writer, `OnChainWriter`, as
//! `OrgIo::connect` builds it: on-chain-client's `create_proxy` and `found_at` over subxt
//! against a Paseo Asset Hub fork with OrgRegistry deployed, signed by the development seed
//! (Alice's, which the fork's config funds), with a miner finalising blocks. This is the run
//! that reaches `write.rs`'s `h160_of(proxy)` and its `WriteError::Step` mapping. Beside it,
//! the reader and writer contract (`common::chain_contract`) runs over the
//! production halves at org-io's seams (`test_support::production_chain`).
//!
//! Where no fork can be had (chopsticks not installed, port 8000 taken, `forge` missing or the
//! deploy failing) each test says so and skips, rather than failing.
//!
//! Needs `on-chain/scripts/node_modules` (`npm ci` there), Foundry's `forge`, and port 8000
//! free. Run:
//!   pkill -f "chopsticks.*--config"; cargo test -p org-io --features test-support,dev-seed \
//!     --test chain_writer_contract -- --test-threads=1 --nocapture

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use org_io::test_support::{production_chain, MemoryKeychain};
use org_io::transport::TransportMode;
use org_io::{ChainSettings, OrgIo};

mod common;
use common::chain_contract::{reader_and_writer_contract, signatory_set_contract, ChainUnderTest};
use common::chopsticks_fork::{spawn_fork, ChopsticksHandle};
use common::chopsticks_reorg::mine_block;

/// Alice's sr25519 seed, as `tests/dev_seed_env.rs` uses it: the fork's config funds her.
const ALICE_SEED_HEX: &str = "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a";

/// Deploy OrgRegistry onto the running fork by the EVM path, as on-chain-client's
/// `write_genesis_e2e.rs` does; the deployed contract's H160, or why it could not be.
fn deploy_org_registry(ws_url: &str) -> Result<[u8; 20], String> {
    let on_chain_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../on-chain");
    let build =
        Command::new("forge").arg("build").current_dir(&on_chain_dir).output().map_err(|error| format!("spawn forge build: {error}"))?;
    if !build.status.success() {
        return Err(format!("forge build failed:\n{}", String::from_utf8_lossy(&build.stderr)));
    }
    let output = Command::new("node")
        .arg("scripts/deploy-chopsticks-evm.mjs")
        .current_dir(&on_chain_dir)
        .env("RPC_URL", ws_url)
        .output()
        .map_err(|error| format!("spawn deploy-chopsticks-evm.mjs: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        return Err(format!("deploy-chopsticks-evm.mjs failed:\n{stdout}\n{}", String::from_utf8_lossy(&output.stderr)));
    }
    let line = stdout.lines().find(|line| line.starts_with("DEPLOYED_H160=")).ok_or("no DEPLOYED_H160= in the deploy output")?;
    let bytes = hex::decode(line.trim_start_matches("DEPLOYED_H160=").trim_start_matches("0x")).map_err(|error| error.to_string())?;
    bytes.try_into().map_err(|_| "not a 20-byte H160".to_string())
}

/// A fork with OrgRegistry deployed and a miner finalising blocks, the development seed set; or
/// `None`, said on stderr, when none can be had (the test then skips).
struct Fork {
    fork: Arc<ChopsticksHandle>,
    settings: ChainSettings,
    stop: Arc<AtomicBool>,
    miner: tokio::task::JoinHandle<()>,
}

impl Fork {
    async fn start(test: &str) -> Option<Self> {
        let fork = match spawn_fork().await {
            Ok(fork) => Arc::new(fork),
            Err(error) => {
                eprintln!("{test}: skipped, no chopsticks fork: {error:?}");
                return None;
            }
        };
        let contract = match deploy_org_registry(&fork.ws_url) {
            Ok(contract) => contract,
            Err(error) => {
                eprintln!("{test}: skipped, OrgRegistry not deployed: {error}");
                return None;
            }
        };
        // Blocks are made only on request on a fork: a miner keeps finalising them while the
        // writer waits for its extrinsics' finality.
        let stop = Arc::new(AtomicBool::new(false));
        let (miner_stop, miner_fork) = (stop.clone(), fork.clone());
        let miner = tokio::spawn(async move {
            while !miner_stop.load(Ordering::Relaxed) {
                // A block mined only moves the fork on; a failed one fails the next read.
                let _ = mine_block(&miner_fork).await;
            }
        });
        std::env::set_var("ODS_ADMIN_SEED", ALICE_SEED_HEX);
        let settings = ChainSettings {
            ws_url: fork.ws_url.clone(),
            contract_h160: contract,
            co_signer: None,
            ahead_limit: std::time::Duration::from_secs(1_200),
        };
        Some(Self { fork, settings, stop, miner })
    }

    async fn stop(self) {
        std::env::remove_var("ODS_ADMIN_SEED");
        self.stop.store(true, Ordering::Relaxed);
        // The miner is stopped with the fork; how its task ended is not the test's.
        let _ = self.miner.await;
    }
}

// The contract test of `FakeChain`'s `ChainWriter`, against the real dependency.
// verifies: LLR-5tpdxw, LLR-sq5pbq
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_production_writer_meets_the_founding_contract() {
    MemoryKeychain::shared().install();
    let Some(fork) = Fork::start("the_production_writer_meets_the_founding_contract").await else { return };
    let dir = std::env::temp_dir().join(format!("org-io-chain-writer-contract-{}", std::process::id()));
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    let settings = ChainSettings {
        ws_url: fork.settings.ws_url.clone(),
        contract_h160: fork.settings.contract_h160,
        co_signer: None,
        ahead_limit: fork.settings.ahead_limit,
    };
    let io = OrgIo::open(&dir, TransportMode::Loopback)
        .unwrap()
        .connect(settings)
        .await
        .unwrap_or_else(|failed| panic!("connect refused: {}", failed.reason));

    common::chain_writer_contract::founding_contract(&io).await;
    fork.stop().await;
}

// The contract test of `FakeChain`'s reads, write window, update and signatory-set read, against
// the real dependency.
// verifies: LLR-js7wgk, LLR-j4nkx5, LLR-bwerd2
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_production_chain_meets_the_reader_and_writer_contract() {
    let Some(fork) = Fork::start("the_production_chain_meets_the_reader_and_writer_contract").await else { return };
    let chain = production_chain(&fork.settings).await.unwrap_or_else(|error| panic!("the production chain: {error}"));
    let under = ChainUnderTest { reader: &chain.reader, writer: &chain.writer, signatory_set: &chain.signatory_set };
    let fork_client = fork.fork.clone();
    let (org, proxy) = reader_and_writer_contract(&under, || {
        let fork_client = fork_client.clone();
        async move {
            mine_block(&fork_client).await.expect("a block mined");
        }
    })
    .await;
    signatory_set_contract(&under, org, proxy, chain.controller).await;
    fork.stop().await;
}
