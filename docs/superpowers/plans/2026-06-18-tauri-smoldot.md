# Tauri smoldot light-client transport — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the ODS PoC Tauri app connect to Asset Hub via an embedded smoldot light client by default (RPC only via an explicit `--force-rpc` opt-in), surface the full transaction lifecycle to the UI, and validate reads/subscriptions/writes over the light client with an ultralight zombienet harness.

**Architecture:** smoldot plugs in as an alternative `OnlineClient<PolkadotConfig>` constructor; the client/event/write logic is backend-agnostic and unchanged. The app's eager synchronous connect is replaced by an async background connect with a live `Connecting→Syncing→Ready→Failed` status and a swappable chain-ops slot. The write path stops discarding subxt's `TxProgress` stream and instead emits `Submitted→InBlock→Finalised` stages forwarded to the frontend as Tauri events.

**Tech Stack:** Rust, subxt 0.50 (`light-client`, `native`, `jsonrpsee`, `reconnecting-rpc-client`), smoldot (via subxt), Tauri 2, Svelte, zombienet + `chain-spec-builder` + `polkadot-omni-node` + pallet-revive `eth-rpc`, Foundry (EVM deploy).

## Global Constraints

- **Spec:** `docs/superpowers/specs/2026-06-18-tauri-smoldot-design.md` — source of truth.
- **smoldot is the unconditional default.** `ODS_CHAIN_WS` does NOT select the mode; only `--force-rpc` (CLI flag) or `ODS_FORCE_RPC=1` (env mirror) selects the WS+`LegacyBackend` path. `--force-rpc` without `ODS_CHAIN_WS` is a hard startup error.
- **All tests use the EVM (REVM) backend.** The `OrgRegistry` contract is the stock `forge build` artifact, deployed with `forge create` over the chain's pallet-revive `eth-rpc` endpoint. NEVER the PVM/`resolc`/`deploy-live.mjs` path. App writes remain subxt `Revive.call` extrinsics (VM-transparent at the call level).
- **Target is the native Tauri desktop backend.** The wasm32-browser subxt lane is out of scope (blocked upstream).
- **Runtime pin:** the harness chain runs the Asset Hub runtime WASM at `spec_version 2_002_002` (the version `on-chain-client::decode::dispatch` has a decoder for). If unavailable, add a decoder rather than changing the pin silently.
- **`InBlock` is provisional** (reorg-revertible); `Finalised` is authoritative. Never report `InBlock` as completion.
- **Versions:** subxt `0.50`, iroh `0.98`, edition pins per each crate's existing `Cargo.toml`. Do not bump.
- **Lints:** `on-chain-client` denies `unwrap_used`/`expect_used`/`panic` in non-test code. `org-node` uses workspace lints. Honor both.
- **Commits:** no `Co-Authored-By` trailer.

---

## File structure

**`org-node` (chain feature):**
- `src/chain_write/progress.rs` — NEW. `TxStage` enum + `TxProgressSink` (newtype over `mpsc::UnboundedSender<TxStage>`) + a no-op sink for callers that don't care.
- `src/service.rs` — MODIFY. `ChainOps` trait methods gain a `&TxProgressSink` param; `connect_chain_smoldot` constructor; `connect_chain_client` unchanged.
- `src/chain_write/proxy.rs`, `src/chain_write/submit.rs` — MODIFY. Drive `TxProgress` to completion, emit stages, return the finalised block from the stream.
- `Cargo.toml` — MODIFY. Pull-through feature so the app can enable `on-chain-client/smoldot`.

**`on-chain-client`:**
- `src/client.rs` — MODIFY. Cap the best-lane backfill span.
- `tests/smoldot_smoke.rs` — MODIFY. Add a `get_org_state` read assertion.

**`app/src-tauri`:**
- `src/chain_conn.rs` — NEW. Mode selection + the two connectors + chainspec loading.
- `src/state.rs` — MODIFY. `ChainStatus` enum, swappable chain-ops slot, async background connect, remove eager `block_on`.
- `src/commands.rs` — MODIFY. `connection_status` reports the enum; submit commands forward `TxStage` → `tx-progress` events.
- `src/lib.rs` — MODIFY. Spawn the background connect after `manage`.
- `Cargo.toml` — MODIFY. `on-chain-client` features `["dev-rpc","smoldot"]`.
- `chainspecs/` — NEW. `include_str!` source for the bundled Paseo specs (copied from `on-chain-client/chainspecs`).

**`app/src/`** (Svelte): a connection-status line + per-tx progress display.

**Harness (new top-level `harness/smoldot-ultralight/`):**
- `chainspec/` — `chain-spec-builder` inputs + generated specs.
- `zombienet.toml` — relay + AH collator + `eth-rpc` definition.
- `spawn.sh`, `derive-smoldot-chainspec.mjs`, `deploy-evm.sh` — orchestration.
- `org-node/tests/smoldot_ultralight.rs` — the gated integration test (or a standalone xtask).

---

## Phase 1 — `org-node`: transaction-progress plumbing (no smoldot yet)

### Task 1: Define `TxStage` and `TxProgressSink`

**Files:**
- Create: `org-node/src/chain_write/progress.rs`
- Modify: `org-node/src/chain_write/mod.rs` (add `pub mod progress;`)
- Test: in-file `#[cfg(test)]` module

**Interfaces:**
- Produces: `pub enum TxStage { Submitted{ext_hash:[u8;32]}, InBlock{ext_hash:[u8;32],block_hash:[u8;32],block_number:u64}, Finalised{ext_hash:[u8;32],block_hash:[u8;32],block_number:u64}, Failed{ext_hash:Option<[u8;32]>,reason:String} }`; `pub struct TxProgressSink(Option<tokio::sync::mpsc::UnboundedSender<TxStage>>)` with `pub fn none()->Self`, `pub fn channel()->(Self, UnboundedReceiver<TxStage>)`, `pub fn send(&self, stage: TxStage)`.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sink_forwards_stages_in_order() {
        let (sink, mut rx) = TxProgressSink::channel();
        sink.send(TxStage::Submitted { ext_hash: [1u8; 32] });
        sink.send(TxStage::Finalised { ext_hash: [1u8; 32], block_hash: [2u8; 32], block_number: 7 });
        assert!(matches!(rx.recv().await, Some(TxStage::Submitted { .. })));
        assert!(matches!(rx.recv().await, Some(TxStage::Finalised { block_number: 7, .. })));
    }

    #[test]
    fn none_sink_is_a_noop() {
        let sink = TxProgressSink::none();
        sink.send(TxStage::Submitted { ext_hash: [0u8; 32] }); // must not panic
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p org-node --features chain chain_write::progress -- --nocapture`
Expected: FAIL — `TxStage` / `TxProgressSink` not found.

- [ ] **Step 3: Write minimal implementation**

```rust
//! Transaction-progress reporting for the on-chain write path.
//! `TxStage` is the app-facing lifecycle; `TxProgressSink` is an optional
//! channel the write helpers push stages into. A `none()` sink discards.
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxStage {
    Submitted { ext_hash: [u8; 32] },
    InBlock { ext_hash: [u8; 32], block_hash: [u8; 32], block_number: u64 },
    Finalised { ext_hash: [u8; 32], block_hash: [u8; 32], block_number: u64 },
    Failed { ext_hash: Option<[u8; 32]>, reason: String },
}

#[derive(Clone)]
pub struct TxProgressSink(Option<UnboundedSender<TxStage>>);

impl TxProgressSink {
    pub fn none() -> Self { Self(None) }
    pub fn channel() -> (Self, UnboundedReceiver<TxStage>) {
        let (tx, rx) = unbounded_channel();
        (Self(Some(tx)), rx)
    }
    /// Best-effort: a closed receiver is ignored (the UI may have navigated away).
    pub fn send(&self, stage: TxStage) {
        if let Some(tx) = &self.0 { let _ = tx.send(stage); }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p org-node --features chain chain_write::progress -- --nocapture`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add org-node/src/chain_write/progress.rs org-node/src/chain_write/mod.rs
git commit -m "feat(org-node): TxStage + TxProgressSink for tx-lifecycle reporting"
```

---

### Task 2: Thread the sink through `ChainOps`

**Files:**
- Modify: `org-node/src/service.rs:43-67` (trait), `:111-` (MockChainOps impl), the `ChainNotConfigured` impl in `app/src-tauri/src/state.rs:43-79`, `SubxtChainOps` impl `org-node/src/service.rs:339-413`, and `OrgService` call sites.
- Test: existing `org-node/tests/service_stories.rs` must still pass (callers pass `TxProgressSink::none()`).

**Interfaces:**
- Consumes: `TxProgressSink` (Task 1).
- Produces: updated trait:
  ```rust
  async fn submit_genesis(&self, genesis_root:[u8;32], org_pub_key:[u8;32], progress:&TxProgressSink) -> Result<(OrgId, Option<[u8;32]>), OrgNodeError>;
  async fn submit_update(&self, org_id:OrgId, new_root:[u8;32], org_pub_key:[u8;32], expected_epoch:u64, proxy_account:Option<[u8;32]>, progress:&TxProgressSink) -> Result<(), OrgNodeError>;
  ```
  `read_state` unchanged.

- [ ] **Step 1: Update the trait + all impls to accept `progress: &TxProgressSink`**

Add `use crate::chain_write::progress::TxProgressSink;` to `service.rs`. Add the `progress: &TxProgressSink` param to both methods in the trait and in `MockChainOps` (ignored), `SubxtChainOps` (threaded into the write helpers in Task 3 — for now pass it down but you may temporarily ignore it to keep this task compiling), and `ChainNotConfigured` (ignored — it errors first).

Update `OrgService` internal callers to pass `&TxProgressSink::none()` (find them: `grep -n "submit_genesis\|submit_update" org-node/src/service.rs`).

- [ ] **Step 2: Run the workspace build + story tests to verify they fail then pass**

Run: `cargo build -p org-node --features app && cargo test -p org-node --features app service_stories`
Expected: compiles; `service_stories` PASS (Mock ignores the sink). If a call site was missed, the compiler names it — fix and re-run.

- [ ] **Step 3: Update `ChainNotConfigured` in the app**

In `app/src-tauri/src/state.rs`, add `progress: &org_node::service::TxProgressSink` (re-export it from `org_node`) to both methods; body unchanged (still returns the "chain not configured" error).

Run: `cargo build -p ods-poc`
Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add org-node/src/service.rs app/src-tauri/src/state.rs
git commit -m "feat(org-node): thread TxProgressSink through ChainOps submit methods"
```

---

### Task 3: Drive the `TxProgress` stream and emit stages

**Files:**
- Modify: `org-node/src/chain_write/submit.rs:64-77`, `org-node/src/chain_write/proxy.rs` (the `genesis_ceremony` + `dispatch_threshold_1` that currently submit-and-drop), `org-node/src/service.rs:339-413` (pass the real sink; derive the finalised block from the stream).
- Test: `org-node/tests/chain_genesis_e2e.rs` (chopsticks) — assert the stage sequence.

**Interfaces:**
- Consumes: `TxProgressSink::send` (Task 1), subxt `TxProgress` / `TxStatus`.
- Produces: write helpers take `progress: &TxProgressSink`, return the finalised `BlockRef` `{hash,number}` instead of relying on `FinalitySink::settle()` for inclusion.

- [ ] **Step 1: Write a helper that drives a `TxProgress` to finalisation, emitting stages**

Add to `submit.rs`:

```rust
use subxt::tx::TxStatus;
use crate::chain_write::progress::{TxProgressSink, TxStage};

/// Drive a subxt TxProgress stream to a finalised block, emitting a TxStage at
/// each transition. Returns the finalised (block_hash, block_number).
/// `InBlock` is provisional (reorg-revertible); only `Finalized` completes.
pub async fn watch_to_finalised(
    mut progress: subxt::tx::TxProgress<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ext_hash: [u8; 32],
    sink: &TxProgressSink,
) -> Result<([u8; 32], u64), WriteError> {
    sink.send(TxStage::Submitted { ext_hash });
    while let Some(status) = progress.next().await {
        match status.map_err(|e| WriteError::Subxt(format!("tx status: {e}")))? {
            TxStatus::InBestBlock(b) => {
                sink.send(TxStage::InBlock {
                    ext_hash,
                    block_hash: b.block_hash().0,
                    block_number: 0, // best-block number not exposed here; 0 = unknown
                });
            }
            TxStatus::InFinalizedBlock(b) => {
                let bh = b.block_hash().0;
                // resolve number for the UI; non-fatal if it fails.
                let number = b.block().await.map(|blk| blk.number().into()).unwrap_or(0);
                sink.send(TxStage::Finalised { ext_hash, block_hash: bh, block_number: number });
                return Ok((bh, number));
            }
            TxStatus::Error { message }
            | TxStatus::Invalid { message }
            | TxStatus::Dropped { message } => {
                sink.send(TxStage::Failed { ext_hash: Some(ext_hash), reason: message.clone() });
                return Err(WriteError::Subxt(format!("tx failed: {message}")));
            }
            _ => {} // Validated / Broadcasted → already covered by Submitted
        }
    }
    sink.send(TxStage::Failed { ext_hash: Some(ext_hash), reason: "stream ended before finalisation".into() });
    Err(WriteError::Subxt("tx stream ended before finalisation".into()))
}
```

> Note: confirm the exact `TxStatus` variant names against subxt 0.50 (`api.tx()` docs) on first compile — they are `Validated`, `Broadcasted`, `InBestBlock`, `NoLongerInBestBlock`, `InFinalizedBlock`, `Error`, `Invalid`, `Dropped`. Adjust the match arms to the version's spelling; the mapping intent is fixed.

- [ ] **Step 2: Rewrite `submit.rs` to use it instead of dropping the stream**

Replace lines 68-76 (capture `progress`, drop it) with: capture `ext_hash = progress.extrinsic_hash().0`, then `watch_to_finalised(progress, ext_hash, sink).await` — threading a new `sink: &TxProgressSink` parameter into this function and its callers. Do the same in `proxy.rs::genesis_ceremony` / `dispatch_threshold_1` (use the `sink` to drive the proxy-create + the contract call; the `Finalised` block hash replaces `FinalitySink::settle()` for the `Proxy.PureCreated` event lookup).

- [ ] **Step 3: Wire the real sink in `SubxtChainOps`**

In `service.rs:341-413`, replace the `let sink = self.sink();` (`FinalitySink`) usage with the passed-in `progress: &TxProgressSink`, and use the finalised block returned by `watch_to_finalised` for the event lookup. Remove the now-dead `FinalitySink::settle()` call in `submit_update` (the stream's `Finalised` is the completion signal).

- [ ] **Step 4: Update the chopsticks e2e test to assert the stage sequence**

In `chain_genesis_e2e.rs`, build a sink via `TxProgressSink::channel()`, pass it to `submit_genesis`, drain the receiver after the call, and assert:

```rust
let (sink, mut rx) = TxProgressSink::channel();
let (_org_id, _p) = chain_ops.submit_genesis(root, pubkey, &sink).await.expect("genesis");
let mut stages = Vec::new();
while let Ok(s) = rx.try_recv() { stages.push(s); }
assert!(matches!(stages.first(), Some(TxStage::Submitted { .. })), "first stage is Submitted");
assert!(stages.iter().any(|s| matches!(s, TxStage::InBlock { .. })), "saw InBlock");
assert!(matches!(stages.last(), Some(TxStage::Finalised { .. })), "last stage is Finalised");
```

- [ ] **Step 5: Run the chopsticks e2e test**

Run: `cargo test -p org-node --features chain --test chain_genesis_e2e -- --nocapture`
Expected: PASS, with the three stage assertions holding. (Requires chopsticks per the existing harness; if the suite is environment-gated, run it the same way the repo already does.)

- [ ] **Step 6: Commit**

```bash
git add org-node/src/chain_write/ org-node/src/service.rs org-node/tests/chain_genesis_e2e.rs
git commit -m "feat(org-node): emit Submitted/InBlock/Finalised by driving the TxProgress stream"
```

---

## Phase 2 — `org-node` + `on-chain-client`: smoldot connector

### Task 4: `connect_chain_smoldot` constructor

**Files:**
- Modify: `org-node/Cargo.toml` (add a `chain-smoldot` feature = `["chain", "on-chain-client/smoldot"]`), `org-node/src/service.rs` (new fn next to `connect_chain_client:436`).
- Test: `org-node` compiles under `--features chain-smoldot`; a unit test asserting the fn signature/owned-handle type (no live connect in unit tests).

**Interfaces:**
- Produces:
  ```rust
  pub struct SmoldotConn { pub _relay: subxt::lightclient::LightClient, pub api: OnlineClient<PolkadotConfig>, pub registry: on_chain_client::OrgRegistryClient }
  pub async fn connect_chain_smoldot(relay_spec:&str, para_spec:&str, contract:[u8;20]) -> Result<SmoldotConn, OrgNodeError>;
  ```
  The `_relay` field keeps the relay light client alive for the connection's lifetime.

- [ ] **Step 1: Add the feature**

In `org-node/Cargo.toml` `[features]`:
```toml
# smoldot light-client transport for the chain feature (native only).
chain-smoldot = ["chain", "on-chain-client/smoldot"]
```

- [ ] **Step 2: Write the constructor**

In `service.rs`, gated `#[cfg(feature = "chain-smoldot")]`:
```rust
/// Connect to Asset Hub via an embedded smoldot light client. The returned
/// `SmoldotConn` OWNS the relay `LightClient`; dropping it tears down the AH
/// connection. Mirrors on-chain-client/tests/smoldot_smoke.rs.
pub async fn connect_chain_smoldot(
    relay_spec: &str,
    para_spec: &str,
    contract: [u8; 20],
) -> Result<SmoldotConn, OrgNodeError> {
    use subxt::lightclient::LightClient;
    use subxt::config::PolkadotConfig;

    let (relay, _relay_rpc) = LightClient::relay_chain(relay_spec)
        .map_err(|e| OrgNodeError::Chain(format!("smoldot relay init: {e}")))?;
    let ah_rpc = relay.parachain(para_spec)
        .map_err(|e| OrgNodeError::Chain(format!("smoldot parachain init: {e}")))?;
    let api = subxt::OnlineClient::<PolkadotConfig>::from_rpc_client(ah_rpc)
        .await
        .map_err(|e| OrgNodeError::Chain(format!("online client from light client: {e}")))?;
    let registry = on_chain_client::OrgRegistryClient::from_client(api.clone(), contract)
        .await
        .map_err(|e| OrgNodeError::Chain(format!("registry client: {e}")))?;
    Ok(SmoldotConn { _relay: relay, api, registry })
}

pub struct SmoldotConn {
    pub _relay: subxt::lightclient::LightClient,
    pub api: subxt::OnlineClient<subxt::config::PolkadotConfig>,
    pub registry: on_chain_client::OrgRegistryClient,
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p org-node --features chain-smoldot`
Expected: compiles. (Resolves the `subxt/light-client` feature via `on-chain-client/smoldot`.)

- [ ] **Step 4: Commit**

```bash
git add org-node/Cargo.toml org-node/src/service.rs
git commit -m "feat(org-node): connect_chain_smoldot (owns relay LightClient)"
```

---

### Task 5: Cap the best-lane backfill span

**Files:**
- Modify: `on-chain-client/src/client.rs` (the best-lane `from..=to` backfill in `best_lane`, ~the `ScanStep::Block` handling).
- Test: `on-chain-client/src/client.rs` `#[cfg(test)]` — a unit test on the span-clamping helper.

**Interfaces:**
- Produces: `fn clamp_backfill(from:u64, to:u64, max:u64) -> (u64 /*from*/, bool /*truncated*/)` and a `const MAX_BACKFILL_SPAN: u64 = 256;`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn backfill_span_is_capped() {
    assert_eq!(clamp_backfill(10, 20, 256), (10, false));      // within cap
    assert_eq!(clamp_backfill(0, 1000, 256), (745, true));     // 1000-256+1
}
```

- [ ] **Step 2: Run it (fails: fn missing)**

Run: `cargo test -p on-chain-client --features dev-rpc client::tests::backfill_span_is_capped`
Expected: FAIL.

- [ ] **Step 3: Implement + apply in `best_lane`**

```rust
const MAX_BACKFILL_SPAN: u64 = 256;

/// Clamp an inclusive `from..=to` backfill range to at most `max` blocks,
/// keeping the most recent. Over a light client, blocks before the sync
/// origin are unretrievable, so an unbounded backfill would hard-fail; we
/// cap it and signal truncation so the caller can log a resync gap.
fn clamp_backfill(from: u64, to: u64, max: u64) -> (u64, bool) {
    if to.saturating_sub(from) + 1 > max { (to - max + 1, true) } else { (from, false) }
}
```
In the best-lane backfill loop, replace `for number in from..=to` with the clamped start and `log`/emit when `truncated` (the existing code already notes the unbounded-span hazard in comments).

- [ ] **Step 4: Run it (passes)**

Run: `cargo test -p on-chain-client --features dev-rpc client::tests::backfill_span_is_capped`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add on-chain-client/src/client.rs
git commit -m "fix(on-chain-client): cap best-lane backfill span (light-client safe)"
```

---

### Task 6: Extend `smoldot_smoke.rs` to exercise `get_org_state`

**Files:**
- Modify: `on-chain-client/tests/smoldot_smoke.rs`.

**Interfaces:**
- Consumes: `OrgRegistryClient::get_org_state` over the smoldot-backed client.

- [ ] **Step 1: Add a read assertion to the live-Paseo smoke test**

After the existing finalised-block assertion, add (using an org admin H160 known to exist on live Paseo AH — read from env `ODS_SMOKE_ORG_ADMIN` so the test is not hard-coded):

```rust
if let Ok(admin_hex) = std::env::var("ODS_SMOKE_ORG_ADMIN") {
    let admin_bytes = hex::decode(admin_hex.trim_start_matches("0x")).expect("admin hex");
    let mut admin = [0u8; 20];
    admin.copy_from_slice(&admin_bytes);
    let registry = on_chain_client::OrgRegistryClient::from_client(api.clone(), CONTRACT_H160)
        .await
        .expect("registry from light client");
    let state = registry.get_org_state(on_chain_client::types::OrgAdmin(admin), None).await;
    eprintln!("get_org_state over smoldot = {state:?}");
    assert!(state.is_ok(), "ReviveApi::get_storage over smoldot failed: {state:?}");
}
```
Add a `const CONTRACT_H160: [u8;20]` (the deployed Paseo AH OrgRegistry address) at the top, mirroring how the address is configured elsewhere.

- [ ] **Step 2: Run the ignored smoke test (live connectivity)**

Run: `ODS_SMOKE_ORG_ADMIN=0x… cargo test -p on-chain-client --no-default-features --features smoldot --test smoldot_smoke -- --ignored --nocapture`
Expected: prints a `spec_version`, a finalised block, and `Ok(...)` from `get_org_state`. (Network-dependent; may take up to 300 s to warp-sync.)

- [ ] **Step 3: Commit**

```bash
git add on-chain-client/tests/smoldot_smoke.rs
git commit -m "test(on-chain-client): smoldot smoke also reads get_org_state (ReviveApi over light client)"
```

---

## Phase 3 — app: async connect, status, mode selection, tx events

### Task 7: `ChainStatus` enum + swappable chain-ops slot

**Files:**
- Modify: `app/src-tauri/src/state.rs` (replace `chain_ready: bool` with a shared status; hold chain ops behind a swappable slot).
- Test: `app/src-tauri` unit test on the status transitions of the slot wrapper.

**Interfaces:**
- Produces:
  ```rust
  #[derive(Clone, serde::Serialize)] pub enum ChainStatus { Connecting, Syncing, Ready, Failed(String) }
  pub struct ChainSlot { /* Arc<Mutex<Arc<dyn ChainOps>>> + Arc<Mutex<ChainStatus>> */ }
  impl ChainSlot { pub fn new()->Self; pub fn status(&self)->ChainStatus; pub fn set_status(&self,s:ChainStatus); pub fn swap_ops(&self, ops:Arc<dyn ChainOps>); pub fn ops(&self)->Arc<dyn ChainOps>; }
  ```
  The slot stores `Arc<dyn ChainOps>` (NOT `Box`) so the forwarding shim (Task 9) can clone the current ops and drop the mutex guard before awaiting.
- `OrgService` is NOT modified: the app passes a forwarding `ChainOps` shim (Task 9) that delegates to the slot, per spec §4.2 (preferred shim approach).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn chain_slot_starts_connecting_and_swaps() {
    let slot = ChainSlot::new();
    assert!(matches!(slot.status(), ChainStatus::Connecting));
    slot.set_status(ChainStatus::Syncing);
    assert!(matches!(slot.status(), ChainStatus::Syncing));
    slot.set_status(ChainStatus::Ready);
    assert!(matches!(slot.status(), ChainStatus::Ready));
}
```

- [ ] **Step 2: Run it (fails)**

Run: `cargo test -p ods-poc chain_slot_starts_connecting_and_swaps`
Expected: FAIL.

- [ ] **Step 3: Implement `ChainStatus` + `ChainSlot`**

```rust
use std::sync::{Arc, Mutex};
use org_node::service::ChainOps;

#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "state", content = "reason")]
pub enum ChainStatus { Connecting, Syncing, Ready, Failed(String) }

#[derive(Clone)]
pub struct ChainSlot {
    ops: Arc<Mutex<Arc<dyn ChainOps>>>,
    status: Arc<Mutex<ChainStatus>>,
}
impl ChainSlot {
    pub fn new() -> Self {
        Self {
            ops: Arc::new(Mutex::new(Arc::new(super::state::ChainNotConfigured))),
            status: Arc::new(Mutex::new(ChainStatus::Connecting)),
        }
    }
    pub fn status(&self) -> ChainStatus { self.status.lock().unwrap_or_else(|p| p.into_inner()).clone() }
    pub fn set_status(&self, s: ChainStatus) { *self.status.lock().unwrap_or_else(|p| p.into_inner()) = s; }
    pub fn swap_ops(&self, ops: Arc<dyn ChainOps>) { *self.ops.lock().unwrap_or_else(|p| p.into_inner()) = ops; }
    /// Clone the current ops Arc so callers can drop the guard before awaiting.
    pub fn ops(&self) -> Arc<dyn ChainOps> { self.ops.lock().unwrap_or_else(|p| p.into_inner()).clone() }
}
```
Make `ChainNotConfigured` `pub`.

- [ ] **Step 4: Run it (passes)**

Run: `cargo test -p ods-poc chain_slot_starts_connecting_and_swaps`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/state.rs
git commit -m "feat(app): ChainStatus + swappable ChainSlot (connecting/syncing/ready/failed)"
```

---

### Task 8: Mode selection + connectors + chainspec loading

**Files:**
- Create: `app/src-tauri/src/chain_conn.rs`, `app/src-tauri/chainspecs/paseo.raw.json`, `app/src-tauri/chainspecs/asset-hub-paseo.raw.json` (copies of the on-chain-client specs).
- Modify: `app/src-tauri/Cargo.toml` (`on-chain-client` features `["dev-rpc","smoldot"]`; ensure `org-node` enables `chain-smoldot`), `app/src-tauri/src/lib.rs` (`mod chain_conn;`).
- Test: `chain_conn.rs` unit test for `select_mode`.

**Interfaces:**
- Produces:
  ```rust
  pub enum ChainMode { Smoldot, ForceRpc { ws_url: String } }
  pub fn select_mode(args: &[String], env: &dyn Fn(&str)->Option<String>) -> Result<ChainMode, String>;
  pub async fn connect(mode: ChainMode, contract:[u8;20], admin_seed:[u8;32], others:Vec<[u8;32]>) -> Result<Box<dyn ChainOps>, String>;
  ```

- [ ] **Step 1: Write the failing test for `select_mode`**

```rust
#[test]
fn smoldot_is_default_even_with_ws_set() {
    let env = |k: &str| if k == "ODS_CHAIN_WS" { Some("wss://x".into()) } else { None };
    assert!(matches!(select_mode(&[], &env).unwrap(), ChainMode::Smoldot));
}
#[test]
fn force_rpc_flag_requires_ws() {
    let env = |_: &str| None;
    let err = select_mode(&["--force-rpc".into()], &env).unwrap_err();
    assert!(err.contains("requires ODS_CHAIN_WS"));
}
#[test]
fn force_rpc_with_ws_selects_rpc() {
    let env = |k: &str| if k == "ODS_CHAIN_WS" { Some("wss://x".into()) } else { None };
    let m = select_mode(&["--force-rpc".into()], &env).unwrap();
    assert!(matches!(m, ChainMode::ForceRpc { .. }));
}
#[test]
fn env_mirror_also_forces_rpc() {
    let env = |k: &str| match k { "ODS_FORCE_RPC" => Some("1".into()), "ODS_CHAIN_WS" => Some("wss://x".into()), _ => None };
    assert!(matches!(select_mode(&[], &env).unwrap(), ChainMode::ForceRpc { .. }));
}
```

- [ ] **Step 2: Run them (fail)**

Run: `cargo test -p ods-poc chain_conn::tests`
Expected: FAIL.

- [ ] **Step 3: Implement `select_mode` + `connect`**

```rust
use org_node::service::ChainOps;

pub const RELAY_SPEC: &str = include_str!("../chainspecs/paseo.raw.json");
pub const PARA_SPEC: &str = include_str!("../chainspecs/asset-hub-paseo.raw.json");

pub enum ChainMode { Smoldot, ForceRpc { ws_url: String } }

pub fn select_mode(args: &[String], env: &dyn Fn(&str) -> Option<String>) -> Result<ChainMode, String> {
    let forced = args.iter().any(|a| a == "--force-rpc") || env("ODS_FORCE_RPC").as_deref() == Some("1");
    if forced {
        let ws_url = env("ODS_CHAIN_WS").ok_or("--force-rpc requires ODS_CHAIN_WS")?;
        Ok(ChainMode::ForceRpc { ws_url })
    } else {
        Ok(ChainMode::Smoldot)
    }
}

pub async fn connect(
    mode: ChainMode, contract: [u8; 20], admin_seed: [u8; 32], others: Vec<[u8; 32]>,
) -> Result<Box<dyn ChainOps>, String> {
    use subxt_signer::sr25519::Keypair;
    let admin = Keypair::from_secret_key(admin_seed).map_err(|e| format!("admin keypair: {e}"))?;
    match mode {
        ChainMode::Smoldot => {
            let relay = std::env::var("ODS_RELAY_CHAINSPEC").ok()
                .map(std::fs::read_to_string).transpose().map_err(|e| format!("relay spec: {e}"))?
                .unwrap_or_else(|| RELAY_SPEC.to_string());
            let para = std::env::var("ODS_PARA_CHAINSPEC").ok()
                .map(std::fs::read_to_string).transpose().map_err(|e| format!("para spec: {e}"))?
                .unwrap_or_else(|| PARA_SPEC.to_string());
            let conn = org_node::service::connect_chain_smoldot(&relay, &para, contract)
                .await.map_err(|e| format!("smoldot connect: {e}"))?;
            Ok(Box::new(org_node::SubxtChainOps::from_smoldot(conn, contract, admin, others)))
        }
        ChainMode::ForceRpc { ws_url } => {
            let (api, registry) = org_node::service::connect_chain_client(&ws_url, contract)
                .await.map_err(|e| format!("rpc connect {ws_url}: {e}"))?;
            Ok(Box::new(org_node::SubxtChainOps::new(api, registry, contract, admin, others)))
        }
    }
}
```
Add a `SubxtChainOps::from_smoldot(conn: SmoldotConn, ...)` constructor in `org-node` that stores `conn.api`/`conn.registry` and keeps `conn._relay` alive (store it in the struct as `_relay`). This is a small additive change to `SubxtChainOps`.

- [ ] **Step 4: Run the tests (pass) + build**

Run: `cargo test -p ods-poc chain_conn::tests && cargo build -p ods-poc`
Expected: 4 tests PASS; app builds with smoldot + dev-rpc.

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/chain_conn.rs app/src-tauri/chainspecs/ app/src-tauri/Cargo.toml app/src-tauri/src/lib.rs org-node/src/service.rs
git commit -m "feat(app): smoldot-default chain mode with --force-rpc opt-in"
```

---

### Task 9: Async background connect (remove eager `block_on`)

**Files:**
- Modify: `app/src-tauri/src/state.rs` (`AppState` holds a `ChainSlot`; `init` no longer connects), `app/src-tauri/src/lib.rs` (spawn the connect task after `manage`).

**Interfaces:**
- Consumes: `ChainSlot` (Task 7), `chain_conn::{select_mode, connect}` (Task 8).
- Produces: `AppState { service, chain: ChainSlot, receiver_started }`; the service is constructed with a forwarding shim that delegates to `chain.ops()`.

- [ ] **Step 1: Replace eager connect in `AppState::init` with the slot + shim**

`init` builds the `ChainSlot`, constructs a `SlotChainOps` shim (holds `Arc<Mutex<Box<dyn ChainOps>>>`, forwards every call), and passes that into `OrgService::new`. No `build_chain_ops`/`block_on` remains. Delete `build_chain_ops`/`connect_chain` from `state.rs` (superseded by `chain_conn`).

```rust
struct SlotChainOps(ChainSlot);
#[async_trait::async_trait]
impl ChainOps for SlotChainOps {
    async fn submit_genesis(&self, r:[u8;32], k:[u8;32], p:&TxProgressSink) -> Result<(OrgId,Option<[u8;32]>),OrgNodeError> {
        let ops = self.0.ops();      // clones the Arc; no guard held
        ops.submit_genesis(r, k, p).await
    }
    // submit_update / read_state: same `let ops = self.0.ops();` then await pattern
}
```
> `ChainSlot::ops()` returns a cloned `Arc<dyn ChainOps>`, so no `Mutex` guard is held across `.await` — the reason the slot stores `Arc`, not `Box` (Task 7).

- [ ] **Step 2: Spawn the connect task in `lib.rs` setup**

```rust
let app_state = AppState::init(tauri_data_dir).unwrap_or_else(|e| panic!("AppState init failed: {e}"));
let slot = app_state.chain.clone();
app.manage(app_state);
// Background connect — window opens immediately; status drives the UI.
tauri::async_runtime::spawn(async move {
    let args: Vec<String> = std::env::args().collect();
    let mode = match crate::chain_conn::select_mode(&args, &|k| std::env::var(k).ok()) {
        Ok(m) => m, Err(e) => { slot.set_status(state::ChainStatus::Failed(e)); return; }
    };
    slot.set_status(state::ChainStatus::Syncing);
    let (contract, seed, others) = match crate::chain_conn::read_env_keys() { Ok(v)=>v, Err(e)=>{ slot.set_status(state::ChainStatus::Failed(e)); return; } };
    match crate::chain_conn::connect(mode, contract, seed, others).await {
        Ok(ops) => { slot.swap_ops(ops.into()); slot.set_status(state::ChainStatus::Ready); }
        Err(e) => slot.set_status(state::ChainStatus::Failed(e)),
    }
});
```
Add `chain_conn::read_env_keys()` that parses `ODS_CONTRACT_H160`/`ODS_ADMIN_SEED`/`ODS_COSIGNER_PUB` (lift the existing parsing from the old `build_chain_ops`).

- [ ] **Step 3: Build + run the app non-blocking smoke**

Run: `cargo build -p ods-poc` then launch via the project's run path (e.g. `cd app && npm run tauri dev`) with NO chain env set.
Expected: window opens immediately; status shows `Connecting`→`Syncing` (then `Failed` if keys absent) — it does NOT block on connect.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/state.rs app/src-tauri/src/lib.rs app/src-tauri/src/chain_conn.rs
git commit -m "feat(app): async background chain connect, no eager block_on in setup"
```

---

### Task 10: `connection_status` reports the enum; commands gate on `Ready`

**Files:**
- Modify: `app/src-tauri/src/commands.rs` (`connection_status`), `app/src-tauri/src/state.rs` (`ConnectionStatus` carries `ChainStatus`).

**Interfaces:**
- Produces: `ConnectionStatus { chain_status: ChainStatus, contract_h160: Option<String>, data_dir: String, mode: String }`.

- [ ] **Step 1: Update `ConnectionStatus` + the command**

Replace `chain_configured: bool` with `chain_status: ChainStatus` (from `AppState.chain.status()`), add `mode: String` (`"smoldot"`/`"rpc"`). Keep `contract_h160`/`data_dir`.

- [ ] **Step 2: Build**

Run: `cargo build -p ods-poc`
Expected: compiles. (Frontend consumes the new shape in Task 11.)

- [ ] **Step 3: Commit**

```bash
git add app/src-tauri/src/commands.rs app/src-tauri/src/state.rs
git commit -m "feat(app): connection_status reports ChainStatus + mode"
```

---

### Task 11: Submit commands forward `TxStage` → `tx-progress` events; Svelte UI

**Files:**
- Modify: `app/src-tauri/src/commands.rs` (the genesis/update commands — find via `grep -n "create_organisation\|submit" commands.rs`), `app/src/` Svelte (status line + tx progress).

**Interfaces:**
- Produces: a `tx-progress` Tauri event with payload `{ stage: "submitted"|"in_block"|"finalised"|"failed", ext_hash: String, block_hash?: String, block_number?: u64, reason?: String }`.

- [ ] **Step 1: Forward stages in the submit command**

In the command that triggers genesis/update, create a sink, spawn a forwarder that emits events, then call the service method:

```rust
let (sink, mut rx) = org_node::service::TxProgressSink::channel();
let app2 = app.clone();
tauri::async_runtime::spawn(async move {
    while let Some(stage) = rx.recv().await {
        let _ = app2.emit("tx-progress", tx_stage_dto(&stage));
    }
});
// ... call service method, passing &sink down to ChainOps::submit_*
```
Add `fn tx_stage_dto(s:&TxStage)->serde_json::Value` mapping each variant to the JSON above. The service/command chain must thread `&sink` to `ChainOps::submit_genesis`/`submit_update` (the `OrgService` method gains a `progress: &TxProgressSink` arg, defaulting to `none()` for non-app callers).

- [ ] **Step 2: Svelte — render status + tx progress**

In the relevant Svelte component: poll/`invoke('connection_status')` to show `Connecting/Syncing/Ready/Failed(reason)`; `listen('tx-progress', …)` to show a 3-step indicator (Submitted → In block → Finalised) and surface `failed.reason`.

- [ ] **Step 3: Build frontend + app**

Run: `cd app && npm run build && cargo build -p ods-poc`
Expected: builds.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/commands.rs app/src/
git commit -m "feat(app): emit tx-progress events and render sync + tx lifecycle in the UI"
```

---

## Phase 4 — Ultralight zombienet harness (EVM backend)

> Heavyweight, explicitly invoked. Goal: a fresh minimal AH chain (real GRANDPA + libp2p + `eth-rpc`), `OrgRegistry` deployed via `forge create`, smoldot pointed at it, running the three acceptance flows. The chopsticks suite is untouched.

### Task 12: Spawn the ultralight chain with `eth-rpc`

**Files:**
- Create: `harness/smoldot-ultralight/zombienet.toml`, `harness/smoldot-ultralight/spawn.sh`, `harness/smoldot-ultralight/README.md`.

- [ ] **Step 1: Build a fresh AH genesis chainspec from the runtime WASM**

`harness/smoldot-ultralight/spawn.sh` (documented, idempotent):
```bash
#!/usr/bin/env bash
set -euo pipefail
WASM=${ODS_AH_RUNTIME_WASM:?set to the asset-hub runtime wasm at spec_version 2_002_002}
OUT=harness/smoldot-ultralight/chainspec
mkdir -p "$OUT"
chain-spec-builder -c "$OUT/ah.plain.json" create \
  --relay-chain rococo-local --para-id 1000 \
  --runtime "$WASM" named-preset development
# Fund the EVM deployer + admin accounts via a genesis patch (jq merge).
jq -s '.[0] * .[1]' "$OUT/ah.plain.json" harness/smoldot-ultralight/genesis-patch.json > "$OUT/ah.patched.json"
chain-spec-builder -c "$OUT/ah.raw.json" convert-to-raw "$OUT/ah.patched.json"
```
Create `genesis-patch.json` seeding `balances` for the deployer H160-mapped account and the sr25519 admin.

- [ ] **Step 2: Define the network (relay + AH collator + eth-rpc)**

`zombienet.toml`:
```toml
[relaychain]
chain = "rococo-local"
default_command = "polkadot"
  [[relaychain.nodes]]
  name = "alice"
  [[relaychain.nodes]]
  name = "bob"

[[parachains]]
id = 1000
chain_spec_path = "harness/smoldot-ultralight/chainspec/ah.raw.json"
  [parachains.collator]
  name = "collator01"
  command = "polkadot-omni-node"
  args = ["--rpc-cors=all", "--rpc-methods=unsafe"]
```
Document that the pallet-revive `eth-rpc` proxy is started separately against the collator's WS RPC (next step), since zombienet doesn't manage it.

- [ ] **Step 3: Start eth-rpc against the collator**

Append to `spawn.sh` (after zombienet is up; read the collator ws port from zombienet output):
```bash
eth-rpc --node-rpc-url "ws://127.0.0.1:${COLLATOR_WS_PORT}" --rpc-port 8545 &
echo $! > harness/smoldot-ultralight/eth-rpc.pid
```

- [ ] **Step 4: Manual verification**

Run: `zombienet spawn harness/smoldot-ultralight/zombienet.toml` (or the project's preferred provider) and confirm: relay finalises blocks (GRANDPA), the AH collator produces blocks (watch logs — guards against polkadot-sdk #11247), and `curl -s -X POST http://127.0.0.1:8545 -d '{"jsonrpc":"2.0","method":"eth_chainId","id":1}' -H 'content-type: application/json'` returns a chain id.
Expected: finalised relay blocks, AH block production, eth-rpc responds.

- [ ] **Step 5: Commit**

```bash
git add harness/smoldot-ultralight/
git commit -m "test(harness): ultralight zombienet AH chain with eth-rpc (no fork)"
```

---

### Task 13: Deploy `OrgRegistry` via EVM (`forge create`)

**Files:**
- Create: `harness/smoldot-ultralight/deploy-evm.sh`.

- [ ] **Step 1: Deploy the stock EVM artifact over eth-rpc**

```bash
#!/usr/bin/env bash
set -euo pipefail
cd on-chain && forge build
ADDR=$(forge create src/OrgRegistry.sol:OrgRegistry \
  --rpc-url http://127.0.0.1:8545 \
  --private-key "${ODS_EVM_DEPLOYER_KEY:?secp256k1 deployer key}" \
  --broadcast --json | jq -r '.deployedTo')
echo "$ADDR" > ../harness/smoldot-ultralight/contract.h160
echo "Deployed OrgRegistry (EVM) to $ADDR"
```
> EVM `forge create` over eth-rpc per `on-chain/README.md` §"EVM path". NOT `deploy-live.mjs`.

- [ ] **Step 2: Verify deployment**

Run: `harness/smoldot-ultralight/deploy-evm.sh` then `cast code $(cat harness/smoldot-ultralight/contract.h160) --rpc-url http://127.0.0.1:8545`
Expected: non-empty bytecode.

- [ ] **Step 3: Commit**

```bash
git add harness/smoldot-ultralight/deploy-evm.sh
git commit -m "test(harness): deploy OrgRegistry via forge create (EVM/REVM, not PVM)"
```

---

### Task 14: Derive smoldot chainspecs for the ephemeral network

**Files:**
- Create: `harness/smoldot-ultralight/derive-smoldot-chainspec.mjs`.

- [ ] **Step 1: Inject the spawned bootnodes into smoldot-ready specs**

The script reads the zombienet-generated relay + AH raw chainspecs (from each node's `cfg/`) and the bootnode multiaddrs from zombienet's spawn output, writes `bootNodes` into both specs, and emits `relay.smol.json` + `ah.smol.json`. No `lightSyncState` is added — smoldot header-syncs the short chain from genesis.
```javascript
import { readFileSync, writeFileSync } from "node:fs";
const [relayIn, ahIn, bootnodesCsv] = process.argv.slice(2);
const boot = bootnodesCsv.split(",");
for (const [inp, outp] of [[relayIn, "relay.smol.json"], [ahIn, "ah.smol.json"]]) {
  const spec = JSON.parse(readFileSync(inp, "utf8"));
  spec.bootNodes = boot;                 // localhost /ip4/.../tcp/<port>/ws/p2p/<peerid>
  writeFileSync(`harness/smoldot-ultralight/chainspec/${outp}`, JSON.stringify(spec));
}
```

- [ ] **Step 2: Verify smoldot accepts the specs**

Run the existing app/test path with `ODS_RELAY_CHAINSPEC`/`ODS_PARA_CHAINSPEC` pointed at the generated files against the running network; confirm a finalised block arrives (reuse the smoke-test connect).
Expected: smoldot connects to localhost bootnodes and reports a finalised block.

- [ ] **Step 3: Commit**

```bash
git add harness/smoldot-ultralight/derive-smoldot-chainspec.mjs
git commit -m "test(harness): derive smoldot chainspecs (localhost bootnodes, genesis sync)"
```

---

### Task 15: The three acceptance flows over smoldot (gated test)

**Files:**
- Create: `org-node/tests/smoldot_ultralight.rs` (gated behind a `smoldot-ultralight` cargo feature so it never runs on default `cargo test`).
- Modify: `org-node/Cargo.toml` (`[[test]]` entry + feature).

- [ ] **Step 1: Write the integration test**

Reads `ODS_RELAY_CHAINSPEC`/`ODS_PARA_CHAINSPEC`/`contract.h160`/deployer+admin keys from env (set by the harness scripts), connects via `connect_chain_smoldot`, then asserts:
```rust
#![cfg(feature = "smoldot-ultralight")]
// 1. read: get_org_state for a freshly-deployed-and-genesis'd org returns Some.
// 2. subscribe: spawn subscribe(), mine N blocks, assert both lanes stay live
//    and a ContractEmitted decodes.
// 3. write: submit_genesis with a TxProgressSink::channel(); drain rx; assert
//    stages == [Submitted, .., InBlock, .., Finalised] and read-back matches.
```
Use the same stage-sequence assertion shape as Task 3, Step 4.

- [ ] **Step 2: Run the full harness end-to-end**

Run (documented in `harness/smoldot-ultralight/README.md`):
```bash
harness/smoldot-ultralight/spawn.sh
harness/smoldot-ultralight/deploy-evm.sh
node harness/smoldot-ultralight/derive-smoldot-chainspec.mjs <relay-cfg> <ah-cfg> "<bootnodes-csv>"
ODS_RELAY_CHAINSPEC=harness/smoldot-ultralight/chainspec/relay.smol.json \
ODS_PARA_CHAINSPEC=harness/smoldot-ultralight/chainspec/ah.smol.json \
ODS_CONTRACT_H160=$(cat harness/smoldot-ultralight/contract.h160) \
cargo test -p org-node --features "chain-smoldot smoldot-ultralight" --test smoldot_ultralight -- --nocapture
```
Expected: all three flows PASS; the write flow shows `Submitted → InBlock → Finalised`.

- [ ] **Step 3: Commit**

```bash
git add org-node/tests/smoldot_ultralight.rs org-node/Cargo.toml harness/smoldot-ultralight/README.md
git commit -m "test(org-node): smoldot ultralight harness — reads, subscribe, write (EVM contract)"
```

---

## Phase 5 — Live-Paseo extended smoke tests (ignored)

### Task 16: Live-Paseo subscribe + write smoke tests

**Files:**
- Create: `on-chain-client/tests/smoldot_subscribe_smoke.rs`, and a write smoke in `org-node/tests/` (gated `chain-smoldot`, `#[ignore]`).

- [ ] **Step 1: Subscribe smoke (ignored)**

Connect via smoldot to live Paseo AH, `subscribe(None)`, assert the stream yields at least one item (or stays live) within a timeout across several finalised blocks.

- [ ] **Step 2: Write smoke (ignored)**

`#[ignore]` live-Paseo `submit_genesis`/`submit_update` against the real deployed EVM contract with a `TxProgressSink::channel()`, asserting the stage sequence reaches `Finalised`. Requires a funded admin; document the env.

- [ ] **Step 3: Run them explicitly**

Run: `cargo test -p on-chain-client --no-default-features --features smoldot --test smoldot_subscribe_smoke -- --ignored --nocapture`
Expected: PASS against live Paseo (network-dependent).

- [ ] **Step 4: Commit**

```bash
git add on-chain-client/tests/smoldot_subscribe_smoke.rs org-node/tests/
git commit -m "test: live-Paseo smoldot subscribe + write smoke (ignored)"
```

---

## Self-review

**Spec coverage:**
- §4.1 connection construction → Task 4. Owned relay handle → Task 4 (`SmoldotConn._relay`) + Task 8 (`from_smoldot`).
- §4.2 async lifecycle + swappable slot + status enum → Tasks 7, 9, 10.
- §4.3 mode selection (`--force-rpc`/`ODS_FORCE_RPC`, smoldot default, hard error) → Task 8.
- §4.4 chainspecs + overrides → Task 8.
- §5.1 + tx feedback → Tasks 1, 2, 3, 11.
- §5 op-2 best-lane backfill cap → Task 5.
- §6.1 chopsticks unchanged → respected (only `chain_genesis_e2e` extended for stage assertions, same WS path).
- §6.2 ultralight harness (EVM, eth-rpc, genesis-from-WASM, derived specs, genesis sync) → Tasks 12-15.
- §6.3 live-Paseo extended smoke → Tasks 6 (read), 16 (subscribe + write).
- §6.4 app non-blocking smoke → Task 9, Step 3.
- §7 components → all mapped above.
- Global constraint "EVM not PVM" → Tasks 13 (forge create) + 15/16 (exercise the EVM contract); explicitly excludes `deploy-live.mjs`.

**Placeholder scan:** Code steps carry real code; the Svelte step (Task 11) and harness shell/JS steps give concrete file contents/commands. The one flagged uncertainty (subxt `TxStatus` variant spelling, Task 3) is called out with the fix instruction, not left vague.

**Type consistency:** `TxStage`/`TxProgressSink` defined in Task 1 used identically in Tasks 2/3/11/15/16. `ChainStatus`/`ChainSlot` defined in Task 7 used in 9/10. `ChainMode`/`select_mode`/`connect` defined in Task 8 used in 9. `SmoldotConn`/`connect_chain_smoldot` defined in Task 4 used in 8. The slot stores `Arc<dyn ChainOps>` consistently (Task 7 `ChainSlot`, `swap_ops`, `ops()` and the Task 9 shim all agree) so no mutex guard is held across `.await`; `connect()` returns `Box<dyn ChainOps>` and converts via `Arc::from`/`.into()` at the `swap_ops` call.
