# Tauri app: smoldot light client as the default chain transport

**Date:** 2026-06-18
**Status:** Design — approved for planning
**Scope:** Make the ODS PoC Tauri app connect to Asset Hub via an embedded smoldot light client by default, replacing the single-WS-RPC dependency that is the source of the app's connection problems. Keep a WS path as an explicit opt-in. Validate reads, subscriptions, and transaction submission over the light client, with an ultralight zombienet harness for deterministic testing.

---

## 1. Problem

The Tauri app's chain connection is unreliable. Today (`app/src-tauri/src/state.rs`, `org-node/src/service.rs::connect_chain_client`) it connects to a **single WS RPC endpoint** (`ODS_CHAIN_WS`) over an explicit `LegacyBackend`. The code comments already document the pain: public Asset Hub endpoints break subxt's default chainHead probe (`"cannot get the block header for block …"`), forcing the pin to `LegacyBackend`, and a single public endpoint is rate-limited and flaky. The connect is also driven **eagerly via `block_on` inside Tauri's synchronous `setup` hook**, so a slow or failing endpoint stalls startup.

A light client removes the single-endpoint dependency: instead of trusting one RPC server, the app runs an in-process **smoldot** light client that joins the chain's libp2p peer network, follows GRANDPA-finalised headers, and reads state via Merkle proofs verified against a finalised header. This aligns with the project's data-sovereignty goal (trust-minimised reads, no privileged RPC operator) and eliminates the flaky-endpoint failure mode.

> Terminology note: this is **not** gossipsub. smoldot connects over libp2p to full nodes (discovered from the chainspec's bootnodes), warp-syncs to a recent finalised header, and verifies finality via GRANDPA justifications. The win is *no single RPC dependency + cryptographically verified reads*, not "gossip".

## 2. Pre-existing groundwork

The smoldot path is largely built but unwired:

- `on-chain-client` has a `smoldot` feature (`subxt/light-client`) and committed light-client-friendly `.smol` chainspecs: `chainspecs/paseo.raw.json` (relay) + `chainspecs/asset-hub-paseo.raw.json` (para_id 1000).
- `on-chain-client/tests/smoldot_smoke.rs` already spins up the embedded light client (`LightClient::relay_chain` → `.parachain` → `OnlineClient::from_rpc_client`), connects to live Paseo AH, checks the runtime `spec_version` against the pinned decoder, and reads one finalised block.
- `OrgRegistryClient::from_client(api, contract)` and `SubxtChainOps` are **backend-agnostic** — they accept any `OnlineClient<PolkadotConfig>`. So wiring smoldot is fundamentally a *connection-construction* change, not a rewrite of client/event/write logic.

What the smoke test does **not** cover, and this work must: the `ReviveApi::get_storage` runtime call (contract reads), the best/finalised event subscriptions, and transaction submission (genesis/update writes) over the light-client backend.

## 3. Decisions (locked)

| Decision | Choice |
|---|---|
| Target runtime | **Native Tauri desktop now**, PWA-aware design. The wasm32-browser subxt 0.50.1 blocker (jsonrpsee feature unification) does **not** affect the native Tauri backend; it remains a future PWA concern gated on a subxt bump. |
| App chain mode | **smoldot is the unconditional default.** `ODS_CHAIN_WS` no longer selects the mode. |
| RPC opt-in | A CLI flag **`--force-rpc`** (plus env mirror **`ODS_FORCE_RPC=1`** for headless/CI) selects the WS+`LegacyBackend` path. |
| Operation scope | **All three** — reads, subscriptions, and tx submission — must work over smoldot. |
| Cold-sync UX | **Async connect + live status.** Window opens immediately; smoldot syncs in the background; status flips to `Ready` when synced. |
| Tx feedback | **Surface the full transaction lifecycle** to the user — `Submitted → InBlock → Finalised` (plus failure) — by consuming subxt's `TxProgress` stream, not the current drop-and-poll approach. |
| Test harness | **Ultralight zombienet + `chain-spec-builder`** chain (fresh minimal genesis from the AH runtime WASM), not a zombie-bite full-state fork. |

## 4. Architecture

### 4.1 Connection construction

Add a smoldot constructor alongside the existing WS one. Reuse the exact ctor chain the smoke test proves:

```
connect_chain_smoldot(relay_spec, para_spec, contract):
    relay    = LightClient::relay_chain(relay_spec)   // handle MUST be kept alive
    ah_rpc   = relay.parachain(para_spec)
    api      = OnlineClient::from_rpc_client(ah_rpc)    // CombinedBackend → chainHead wins (smoldot exposes chainHead_v1)
    registry = OrgRegistryClient::from_client(api, contract)
    return (relay /* owned */, api, registry)
```

Two consequences pinned by design:

- **The relay `LightClient` handle must be owned for the app's lifetime.** Dropping it tears down the Asset Hub connection. It lives in `AppState` (or inside the `SubxtChainOps`/chain-ops slot).
- **Backend flips from `LegacyBackend` → `CombinedBackend`/chainHead.** The existing `client.rs` subscription code uses backend-neutral subxt APIs (`stream_best_blocks`, `stream_blocks`), so it rides along — but this is a validation point (§6), not a free lunch.

### 4.2 Connection lifecycle (the real restructure)

Today `build_chain_ops()` is synchronous and freezes `chain_ready` once, inside `setup`. smoldot cold warp-sync can take tens of seconds to minutes (the smoke test budgets 300 s), so blocking `setup` is not acceptable. New model:

- `setup` returns immediately. `AppState` holds chain ops behind a **swappable slot** — `Arc<Mutex<Box<dyn ChainOps>>>` (or `arc-swap`), initialised to `ChainNotConfigured`.
- A **background tokio task** runs the selected connector. On success it swaps the real `SubxtChainOps` into the slot and advances a shared status.
- Status is an enum: **`Connecting → Syncing → Ready → Failed { reason }`**, held in shared state (replacing the single `chain_ready: bool`). The existing `connection_status` command is extended to report it so the Svelte UI can show a "syncing…" state and a failure reason.
- Chain commands invoked before `Ready` return a clear error (`"chain still syncing"`) rather than blocking.

This requires `OrgService` to allow its `ChainOps` to be **replaced after construction** (today it is set once in `OrgService::new`). That is the one non-trivial change to `org-node`'s service surface. Options: expose a `set_chain_ops(&self, Box<dyn ChainOps>)` that swaps an internal `Arc<Mutex<…>>`, or have the app own the slot and pass a thin `ChainOps` shim into the service that forwards to the slot. The shim approach keeps `OrgService` unchanged and is preferred unless it complicates the command handlers.

### 4.3 Mode selection

```
if --force-rpc (or ODS_FORCE_RPC=1):
    require ODS_CHAIN_WS  (else hard startup error: "--force-rpc requires ODS_CHAIN_WS")
    connect via WS + LegacyBackend (reconnecting-rpc-client)   // existing connect_chain_client
else:
    connect via smoldot (default)                              // connect_chain_smoldot
```

- `ODS_CHAIN_WS` is **ignored for mode selection** in the default path; it only supplies the endpoint when `--force-rpc` is set.
- `ODS_CONTRACT_H160` + `ODS_ADMIN_SEED` remain required in both modes.
- The flag is parsed from `std::env::args` in the Tauri backend; `ODS_FORCE_RPC=1` is an accepted env mirror.
- The WS path is hardened with subxt's `reconnecting-rpc-client` feature (already added to `org-node`'s `subxt` dependency).

### 4.4 Chainspecs

- Default: the committed `paseo.raw.json` + `asset-hub-paseo.raw.json`, `include_str!`'d into the app (as the smoke test does).
- Optional path overrides `ODS_RELAY_CHAINSPEC` / `ODS_PARA_CHAINSPEC` so the test harness (and future networks) can point smoldot at a different chain without a rebuild.

## 5. The three operation paths over smoldot

The client/event/write logic is unchanged; each operation rides the new `OnlineClient`. Each has a smoldot-specific risk and a matching acceptance check.

**1. Contract reads — `ReviveApi::get_storage` (lowest risk).** Over chainHead this is a `chainHead_v1_call`: smoldot executes the runtime wasm against a Merkle state proof fetched from peers. Reads at `at = None` resolve to the latest *finalised* block (available post-sync). The first call post-sync may be slow (fetches runtime code + proofs). ⚠️ Reading at an explicit *old* block hash (before the sync origin) fails — the light client has no ancient state.

**2. Event subscriptions — best + finalised lanes (medium risk).** The finalised lane (`stream_blocks`) is already exercised by the smoke test. The best lane (`stream_best_blocks` + `at_block(number)` gap-fill) backfills `seed.number+1 ..= head`; over a light client, blocks before the sync origin are unretrievable, so the "unbounded span" caveat already noted in `client.rs` turns from *slow* into a *hard failure* if the seed is ever stale. **Design change:** cap the best-lane backfill span and surface a resync rather than walking into unretrievable blocks.

**3. Transaction submission — genesis/update writes.** Maps to smoldot's `transactionWatch_v1_submitAndWatch` via the CombinedBackend; subxt's `sign_and_submit_then_watch` rides it. Nonce, genesis hash, mortal era, and runtime version for signing all come from light-client state reads (fine post-sync). **smoldot's watch API already emits the full lifecycle** — `validated → broadcasted → bestChainBlockIncluded → finalized`, plus `invalid`/`dropped`/`error` — and subxt's `TxProgress` stream surfaces each. So the staged feedback in §5.1 is not new machinery: the events are already there; the current code simply discards the stream. The only nuance is the universal one — `bestChainBlockIncluded` ("in block") is provisional and can be retracted by a reorg, while `finalized` is authoritative — not a light-client-specific weakness.

### 5.1 Transaction progress feedback

The user must see the transaction lifecycle, not just a final ok/err. Today `chain_write/submit.rs` calls `sign_and_submit_then_watch_default`, captures the extrinsic hash, then **drops the `TxProgress` stream immediately** and tracks finality out-of-band by polling `FinalitySink::settle()` (a coarse "a new finalised block appeared" loop). That yields no submitted/in-block signal. This design replaces that, for the genesis/update submit calls, with **driving the `TxProgress` stream to completion** and emitting a stage at each transition. Because smoldot (and any chainHead node) already produces these transitions, the work is wiring them through, not generating them.

**`TxStage` (app-facing enum, defined in `org-node`):**
- `Submitted { ext_hash }` — accepted/broadcast by the network (subxt `Validated`/`Broadcasted`; smoldot `validated`/`broadcasted`).
- `InBlock { ext_hash, block_hash, block_number }` — included in a best block (smoldot `bestChainBlockIncluded`); **provisional** — may be retracted on reorg.
- `Finalised { ext_hash, block_hash, block_number }` — included in a finalised block (smoldot `finalized`); **authoritative** completion.
- `Failed { ext_hash: Option, reason }` — `Invalid` / `Dropped` / `FinalityTimeout` / `error` / transport error.

**Mechanism.** `ChainOps::submit_genesis` / `submit_update` gain a **progress sink** parameter — an `mpsc::UnboundedSender<TxStage>` (or a small `TxProgressSink` trait). `SubxtChainOps` maps each subxt `TxStatus` to a `TxStage` and sends it; the method still returns its final result once `Finalised` (or a failure) is reached. `ChainNotConfigured` ignores the sink (it errors before any stage). The app-layer Tauri command supplies a sink that forwards each `TxStage` to the frontend as a **`tx-progress` event** via `AppHandle::emit`; the Svelte UI renders submitted → in block → finalised (and surfaces failures). This reuses the push-status approach already chosen for connection state (§4.2).

**Interaction with the proxy-event lookup.** `submit_genesis` currently needs the finalised block to find the `Proxy.PureCreated` event (via `FinalitySink`). Driving the `TxProgress` stream gives the `Finalised { block_hash }` directly, so the lookup block comes from the stream rather than a separate poll — a strict improvement. `FinalitySink` may remain for any path that still needs the standalone "next finalised block" probe; consolidating versus keeping both is a planning detail.

## 6. Testing strategy

### 6.1 Hermetic WS suite — unchanged

The existing chopsticks-based integration suite stays exactly as-is on the WS path. **smoldot cannot sync from chopsticks** (confirmed by research): chopsticks exposes only a WebSocket JSON-RPC endpoint (no libp2p port) and produces no GRANDPA warp-sync proofs, while smoldot connects over libp2p to bootnodes and bootstraps via GRANDPA. This is a justified constraint, not an oversight.

### 6.2 Ultralight zombienet harness — new, hermetic-ish, deterministic

Build a deterministic smoldot integration harness on a **fresh minimal chain**, not a fork:

1. **Build genesis with `chain-spec-builder`** from the **Asset Hub runtime WASM** whose `spec_version` matches the pinned `decode/dispatch` decoders (Paseo AH `2_002_002`). Keeping the real runtime preserves `pallet-revive`, event formats, and `ReviveApi::get_storage`; only the *state* is fresh and tiny. Seed funded admin/co-signer accounts via a genesis patch.
2. **Spawn with zombienet** (relay + AH collator). Real nodes → real GRANDPA finality, libp2p, and bootnode multiaddrs. ⚠️ Local Asset Hub block production has known gotchas (polkadot-sdk #11247 "AssetHub local does not produce blocks", #5932 manual-seal+AH); budget for getting the collator/omni-node setup right.
3. **Deploy `OrgRegistry` fresh** onto the spawned chain via the existing `on-chain/scripts` deploy path.
4. **Derive smoldot chainspecs** (relay + AH) from the spawned nodes' `cfg/` chainspecs, injecting the localhost bootnode multiaddrs. **No `lightSyncState` checkpoint needed** — for a short-lived chain smoldot header-syncs from genesis via the all-forks protocol (it can source starting chain info from genesis, a checkpoint, or GRANDPA runtime calls). Point smoldot at these via `ODS_RELAY_CHAINSPEC` / `ODS_PARA_CHAINSPEC`.
5. **Run the three acceptance flows over the light client** against the spawned chain:
   - read: `get_org_state` for a deployed org returns the expected decoded `OrgState`;
   - subscribe: both lanes stay live across N blocks and a `ContractEmitted` decodes;
   - write: a submitted genesis (or update) emits the staged sequence `Submitted → InBlock → Finalised` (assert the stages are observed in order, not just final success) and lands in a finalised block.

The harness is heavyweight (real node binaries) — it runs as an explicitly-invoked test (dedicated cargo feature / `#[ignore]`), never on a default `cargo test`.

### 6.3 Live-Paseo smoke tests — extended

Extend `smoldot_smoke.rs` (and add siblings) as `#[ignore]` live-Paseo checks covering the same three flows against the real deployed contract — a sanity layer over the real network, complementing the deterministic ultralight harness.

### 6.4 App-level smoke

A native build check: the app window opens immediately (does not block on connect), status reports `Connecting`/`Syncing`, and chain commands return `"chain still syncing"` until `Ready`.

## 7. Components touched

- `org-node/Cargo.toml` — already gained `reconnecting-rpc-client`; add a way for the `chain` feature to pull `on-chain-client/smoldot`.
- `org-node/src/service.rs` — `connect_chain_smoldot` constructor (owns the relay `LightClient`); chain-ops swappability (or a forwarding shim); `ChainOps::submit_genesis`/`submit_update` gain a `TxStage` progress-sink parameter (`ChainNotConfigured` ignores it); `TxStage` enum defined here. Best-lane backfill cap in `on-chain-client/src/client.rs`.
- `org-node/src/chain_write/submit.rs` (and `multisig.rs`) — stop dropping the `TxProgress` stream; drive it to completion, mapping each `TxStatus` to a `TxStage` sent on the sink; derive the proxy-event lookup block from the `Finalised` status.
- `app/src-tauri/Cargo.toml` — `on-chain-client` with `smoldot` + `dev-rpc`; `org-node` `app` feature.
- `app/src-tauri/src/state.rs` — async background connect; swappable chain-ops slot; `Connecting→Syncing→Ready→Failed` status; `--force-rpc`/`ODS_FORCE_RPC` selection; chainspec `include_str!` + path overrides; remove the eager `block_on` in `setup`.
- `app/src-tauri/src/commands.rs` — `connection_status` reports the status enum; chain commands gate on `Ready`; submit commands pass a sink that forwards each `TxStage` to the frontend as a `tx-progress` event via `AppHandle::emit`.
- Svelte UI — render the syncing/failed states (minimal: a status line) and the per-transaction `Submitted → InBlock → Finalised`/failure progress.
- Tests — extended `smoldot_smoke.rs`; new ultralight zombienet harness (feature-gated).

## 8. Risks & open questions

- **Local Asset Hub block production** under zombienet is a known rough edge (#11247, #5932). First harness milestone: get a fresh AH chain finalising blocks locally before any smoldot wiring.
- **chainHead backend subscription parity.** Confirm `stream_best_blocks`/`stream_blocks` + the reorg/gap-fill logic behave correctly over the CombinedBackend/chainHead path (vs the LegacyBackend the suite was written against).
- **tx submission over light client** — smoldot's `transactionWatch_v1` provides the full status stream natively, so the main work is consuming it (not generating feedback). Residual risks: correctly handling `bestChainBlockIncluded` retraction on reorg (don't report a provisional in-block as done), and the `--force-rpc` escape hatch covers writes operationally if any mapping issue surfaces.
- **Runtime WASM provenance** for `chain-spec-builder` must match `spec_version 2_002_002` so existing decoders apply; if unavailable, add a decoder for whatever runtime the harness uses.
- **PWA (future):** native smoldot does not unblock the browser target; the subxt 0.50.1 wasm lane remains blocked upstream and is out of scope here.

## 9. Out of scope

- Browser/PWA wasm light-client target (blocked upstream; future work).
- Removing the WS path (kept as `--force-rpc` opt-in; chopsticks suite depends on it).
- zombie-bite full-state fork harness (superseded by the ultralight zombienet harness).
- Changes to the contract, decoders, or transport (iroh) layers beyond what wiring smoldot requires.
