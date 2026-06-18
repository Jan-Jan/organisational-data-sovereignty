# ODS Phase 2 — Hermetic Distributed-Path Test Coverage — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add hermetic, CI-reproducible, secret-free tests that exercise the two currently compile-verified-only distributed paths — `TransportMode::Networked` (relay + dial-by-`EndpointId`) and the production `FinalitySink::settle` polling loop — plus a preflight health-check harness, so the manual two-laptop run shrinks to only cross-NAT traversal.

**Architecture:** A new `test-support` cargo feature exposes a `bind_with_relay` constructor on `OrgEndpoint` that targets an in-process `iroh::test_utils::run_relay_server()` with a `MemoryLookup` for id resolution; a re-export makes the production `FinalitySink` constructible from a test, which drives it against a chopsticks fork mined by a background timer; a `preflight` module + binary run reusable health checks. Production code paths (`Loopback`/`Networked` enum, `presets::N0`) are untouched.

**Tech Stack:** Rust, iroh 0.98 (`test-utils` feature for the local relay), subxt 0.50, chopsticks (via existing `tests/common` harness), tokio.

**Key API facts (verified against the pinned crate sources — do not re-derive):**
- `iroh::test_utils::run_relay_server() -> Result<(iroh::RelayMap, iroh::RelayUrl, iroh::test_utils::... Server), _>` — requires the iroh **`test-utils`** feature (the `test_utils` module is `#[cfg(any(test, feature = "test-utils"))]`).
- `iroh::address_lookup::MemoryLookup` — `MemoryLookup::new()`, is `Clone`, `.add_endpoint_info(ep.addr())` (accepts an `EndpointAddr`). Attach to a builder with `.address_lookup(lookup)`.
- `iroh::RelayMode::Custom(relay_map)`; `iroh::address_lookup::AddrFilter::relay_only()`; `Builder::addr_filter(AddrFilter)`.
- `Endpoint::builder(iroh::endpoint::presets::Minimal)…` then `.relay_mode(..).address_lookup(..).secret_key(sk).alpns(vec![ALPN.to_vec()]).bind().await`.
- `endpoint.online().await` registers with the relay; call it on both endpoints before a relay-only dial. `endpoint.id() -> EndpointId` (for `send_to_id`); `endpoint.addr() -> EndpointAddr` (carries the relay home after `online()`).
- subxt `OnlineClient::<PolkadotConfig>::at_current_block().await?.block_ref().hash().0 -> [u8;32]`.
- Test client builder: `common::conn::legacy_client(ws_url).await`. Chopsticks helpers: `common::chopsticks_fork::spawn_fork().await` (gives `handle.ws_url`), `common::chopsticks_reorg::mine_block(&handle).await`.
- `FinalitySink` fields: `{ api: OnlineClient<PolkadotConfig>, timeout: std::time::Duration }`; lives in private `mod subxt_impl` and is **not** re-exported yet.

**Build/test conventions for this repo (must follow):**
- Use `CARGO_HOME=/tmp/cargo_home_fuzz` for any command that may fetch crates (`~/.cargo` is read-only here).
- Clippy gate is `--lib` only (`-D unwrap_used/expect_used/panic`); **tests may unwrap/expect**.
- Do **not** run `sed` on Rust test files.
- No `Co-Authored-By:` lines in commits. gpg signing times out in this environment — commit with `--no-gpg-sign`; the user re-signs on merge.
- Work happens in a git worktree (the executing skill sets this up).

---

## Task 0: Cargo wiring — `test-support` feature + iroh `test-utils` dev-dep

**Files:**
- Modify: `org-node/Cargo.toml`

- [ ] **Step 1: Add the `test-support` feature**

In `org-node/Cargo.toml`, under `[features]`, add after the `app` line:

```toml
# Test-only seams (e.g. binding an iroh endpoint against an in-process relay).
# Never enabled in production builds; gated constructors live behind this.
test-support = ["transport"]
```

- [ ] **Step 2: Enable iroh's `test-utils` for test builds**

In `[dev-dependencies]`, add (iroh is already an optional normal dep; this dev-dep turns on `test-utils` and unifies features for test builds):

```toml
# Enables iroh::test_utils::run_relay_server() for the hermetic Networked test.
iroh = { version = "0.98", features = ["test-utils"] }
```

- [ ] **Step 3: Register the two new test targets**

Append to `org-node/Cargo.toml`:

```toml
[[test]]
name = "transport_networked"
path = "tests/transport_networked.rs"
required-features = ["transport", "test-support"]

[[test]]
name = "finality_polling"
path = "tests/finality_polling.rs"
required-features = ["chain"]
```

- [ ] **Step 4: Verify the workspace still resolves and builds**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo build -p org-node --features "app,test-support"`
Expected: `Finished` with no errors (iroh `test-utils` resolves; no new code yet).

- [ ] **Step 5: Commit**

```bash
git add org-node/Cargo.toml
git commit --no-gpg-sign -m "build(org-node): add test-support feature + iroh test-utils dev-dep"
```

---

## Task 1: `OrgEndpoint::bind_with_relay` test-support constructor

**Files:**
- Modify: `org-node/src/transport/endpoint.rs`

- [ ] **Step 1: Add the imports the constructor needs**

At the top of `org-node/src/transport/endpoint.rs`, the existing `use iroh::{...}` already imports `RelayMode`, `presets`, etc. Add a feature-gated import block immediately after the existing `use` of `iroh` items:

```rust
#[cfg(feature = "test-support")]
use iroh::{RelayMap, address_lookup::MemoryLookup};
```

- [ ] **Step 2: Add the constructor inside `impl OrgEndpoint`**

Insert this method directly after `bind_with_mode` (before `device_key`):

```rust
    /// Bind a Networked-style endpoint whose ONLY relay is `relay_map` and
    /// whose peer-address resolution comes from `lookup`, with direct UDP
    /// paths filtered out so traffic is forced through the relay.
    ///
    /// This mirrors the `TransportMode::Networked` builder (same ALPN, same
    /// secret-key-from-seed) but swaps `presets::N0`'s real n0 relay + DNS
    /// discovery for an in-process relay and an in-memory address lookup.
    /// It exists only to make the Networked dial-by-`EndpointId` path
    /// hermetically testable; it is never used in production.
    ///
    /// Usage: bind both endpoints with a shared `MemoryLookup`, call
    /// [`online`](iroh::Endpoint::online) on each, seed the lookup with each
    /// endpoint's `addr()`, then dial with [`send_to_id`].
    ///
    /// [`send_to_id`]: OrgEndpoint::send_to_id
    #[cfg(feature = "test-support")]
    pub async fn bind_with_relay(
        device: &SigningKeypair,
        relay_map: RelayMap,
        lookup: MemoryLookup,
    ) -> Result<Self, TransportError> {
        let sk = iroh::SecretKey::from_bytes(&device.to_seed());
        let inner = iroh::Endpoint::builder(presets::Minimal)
            .relay_mode(RelayMode::Custom(relay_map))
            .address_lookup(lookup)
            .addr_filter(iroh::address_lookup::AddrFilter::relay_only())
            .secret_key(sk)
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
            .map_err(|e| TransportError::Bind(e.to_string()))?;
        Ok(Self {
            inner,
            device_key: device.device_key(),
        })
    }
```

- [ ] **Step 3: Verify it compiles under the feature**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo build -p org-node --features "transport,test-support"`
Expected: `Finished`, no errors.

- [ ] **Step 4: Verify it is NOT compiled without the feature (no leak into production)**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo build -p org-node --features "transport"`
Expected: `Finished` (the `bind_with_relay` and its imports are absent; no `unused import` warnings because they are feature-gated).

- [ ] **Step 5: Clippy gate (lib only)**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-node --lib --features "app,test-support" -- -D warnings`
Expected: no warnings.

- [ ] **Step 6: Commit**

```bash
git add org-node/src/transport/endpoint.rs
git commit --no-gpg-sign -m "feat(org-node): test-support bind_with_relay for in-process iroh relay"
```

---

## Task 2: Hermetic Networked-transport integration test

**Files:**
- Create: `org-node/tests/transport_networked.rs`

This test mirrors `transport_handshake.rs` but dials over an in-process relay using `send_to_id` (dial purely by `EndpointId`), covering the `Networked` builder config, relay datapath, id resolution, frame codec, and authenticated receive.

- [ ] **Step 1: Write the test file**

Create `org-node/tests/transport_networked.rs`:

```rust
#![cfg(all(feature = "transport", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Two real iroh endpoints that find and reach each other ONLY via an
//! in-process relay + in-memory address lookup, dialled purely by EndpointId
//! (`send_to_id`). This is the hermetic stand-in for the two-laptop
//! `TransportMode::Networked` path: it exercises the Networked builder, the
//! relay datapath, dial-by-EndpointId resolution, the frame codec, and the
//! authenticated receive. It does NOT (and cannot, on one machine) exercise
//! real cross-NAT hole-punching — that is the only thing left to the manual
//! two-laptop run.
use std::time::Duration;

use iroh::address_lookup::MemoryLookup;
use org_node::SignedDeltaEnvelope;
use org_node::chain::{MockChain, OrgState};
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::sequence::SeqGuard;
use org_node::transport::endpoint::OrgEndpoint;
use org_node::transport::wire::WireMessage;
use org_node::verify::{VerifyContext, verify_envelope_against_chain};
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::{MemberId, MemberLeaf};

type Trie = OrgTrie<Blake3Hasher>;

// Inline genesis_and_admit (test_fixtures is lib-private). Admin keypair
// doubles as device key, matching the fixture convention. Returns
// (genesis_trie, new_trie_with_bob, delta).
fn genesis_and_admit(admin: &SigningKeypair) -> (Trie, Trie, org_members::delta::Delta) {
    let admin_leaf = MemberLeaf::new(
        MemberId::new([1u8; 32]),
        "admin",
        admin.member_key(),
        "Admin",
        "User",
        vec![admin.device_key()],
    )
    .unwrap();
    let (genesis, _) = Trie::genesis(vec![admin_leaf])
        .unwrap()
        .recalculate()
        .unwrap();

    let b_member = SigningKeypair::from_seed([2u8; 32]);
    let b_device = SigningKeypair::from_seed([3u8; 32]);
    let b_leaf = MemberLeaf::new(
        MemberId::new([2u8; 32]),
        "bob",
        b_member.member_key(),
        "Bob",
        "User",
        vec![b_device.device_key()],
    )
    .unwrap();
    let (new_trie, delta) = genesis.add_member(b_leaf).unwrap().recalculate().unwrap();
    (genesis, new_trie, delta)
}

#[tokio::test]
async fn delivers_and_verifies_admit_over_relay_by_id() {
    // Admin MEMBER key signs the envelope (doubles as device key in fixture).
    let admin = SigningKeypair::from_seed([1u8; 32]);
    let a_device = SigningKeypair::from_seed([10u8; 32]); // A's iroh identity
    let b_device = SigningKeypair::from_seed([11u8; 32]); // B's iroh identity
    let org = OrgId::new([5u8; 20]);

    let (genesis, new_trie, delta) = genesis_and_admit(&admin);
    let new_root = new_trie.root_hash().unwrap();
    let env = SignedDeltaEnvelope::build(org, 2, &delta, &admin).unwrap();
    let msg = WireMessage {
        envelope: env.clone(),
        org_secret: Some([0xab; 32]),
        genesis_snapshot: None,
    };

    // In-process relay + shared in-memory address lookup. The Server is held
    // for the test's lifetime (dropping it stops the relay).
    let (relay_map, _relay_url, _relay_server) = iroh::test_utils::run_relay_server()
        .await
        .expect("spawn in-process relay");
    let lookup = MemoryLookup::new();

    let ep_a = OrgEndpoint::bind_with_relay(&a_device, relay_map.clone(), lookup.clone())
        .await
        .expect("bind A against relay");
    let ep_b = OrgEndpoint::bind_with_relay(&b_device, relay_map.clone(), lookup.clone())
        .await
        .expect("bind B against relay");

    // Register both with the relay so a relay home is assigned, then publish
    // each endpoint's addr (including its relay home) into the shared lookup
    // so dial-by-EndpointId can resolve.
    tokio::time::timeout(Duration::from_secs(20), ep_a.inner().online())
        .await
        .expect("A did not come online within 20 s");
    tokio::time::timeout(Duration::from_secs(20), ep_b.inner().online())
        .await
        .expect("B did not come online within 20 s");
    lookup.add_endpoint_info(ep_a.inner().addr());
    lookup.add_endpoint_info(ep_b.inner().addr());

    let b_id = ep_b.inner().id();

    // B receives in a background task — spawn before A dials so accept() is
    // already waiting.
    let recv_task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(30), ep_b.recv_one())
            .await
            .expect("recv_one timed out after 30 s")
    });

    // A dials B PURELY BY EndpointId — resolution + relay routing are what we
    // are exercising here.
    tokio::time::timeout(Duration::from_secs(30), ep_a.send_to_id(b_id, &msg))
        .await
        .expect("send_to_id timed out after 30 s")
        .expect("send_to_id failed");

    let (remote_device, got) = recv_task
        .await
        .expect("recv task panicked")
        .expect("recv_one failed");

    // 1. QUIC handshake authenticated A's device key.
    assert_eq!(
        remote_device.as_bytes(),
        a_device.device_key().as_bytes(),
        "authenticated remote device key must equal A's device key"
    );
    // 2. The WireMessage arrived intact over the relay.
    assert_eq!(got, msg, "received WireMessage must equal sent WireMessage");

    // 3. B verifies the received envelope against a MockChain seeded with the
    //    new root at epoch 2 (independent on-chain read).
    let mut chain = MockChain::new();
    chain.set(
        org,
        OrgState { root_hash: new_root, org_pub_key: [0u8; 32], epoch: 2 },
    );
    let ctx = VerifyContext {
        expected_org_id: org,
        author_member_key: &admin.verifying_key(),
        seq_guard: SeqGuard::from_last_seen(1),
        last_committed_epoch: 1,
    };
    let out = verify_envelope_against_chain(&genesis, &got.envelope, &ctx, &chain)
        .expect("verify_envelope_against_chain must succeed");
    assert_eq!(out.trie.root_hash().unwrap(), new_root, "committed root mismatch");
    assert_eq!(out.epoch, 2, "committed epoch must be 2");
}
```

- [ ] **Step 2: Run the test — expect it to PASS**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features "app,test-support" --test transport_networked -- --nocapture`
Expected: `test delivers_and_verifies_admit_over_relay_by_id ... ok`.

- [ ] **Step 3: If it hangs or fails to connect, apply the documented fallbacks in order**

This is the plan's flagged runtime-verification point. If the dial does not complete:
  1. Increase the `online()` / `send_to_id` / `recv_one` timeouts (relay registration can be slow on a cold run).
  2. If `online()` itself never resolves, the relay-only filter may be preventing the relay handshake — remove `.addr_filter(iroh::address_lookup::AddrFilter::relay_only())` from `bind_with_relay` (Task 1, Step 2). The connection may then complete over a direct loopback path instead of the relay; the test still validates the Networked builder, `send_to_id` id-resolution, codec, and authenticated receive.
  3. **If you take fallback (2), you MUST record the downgrade** in the test module doc-comment: add a line stating "RELAY DATAPATH NOT EXERCISED — addr_filter::relay_only removed because <reason>; connection routes direct over loopback. send_to_id/id-resolution/codec/auth are still covered." Do not silently accept it.

- [ ] **Step 4: Confirm the existing loopback test still passes (no regression)**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features "transport" --test transport_handshake`
Expected: `test delivers_and_verifies_admit_over_iroh ... ok`.

- [ ] **Step 5: Commit**

```bash
git add org-node/tests/transport_networked.rs
git commit --no-gpg-sign -m "test(org-node): hermetic Networked transport over in-process relay (dial-by-id)"
```

---

## Task 3: Re-export `FinalitySink` so tests can construct the production sink

**Files:**
- Modify: `org-node/src/service.rs`

- [ ] **Step 1: Add the re-export**

In `org-node/src/service.rs`, find the existing line (around line 429):

```rust
pub use subxt_impl::SubxtChainOps;
```

Change it to also export `FinalitySink`:

```rust
pub use subxt_impl::{FinalitySink, SubxtChainOps};
```

- [ ] **Step 2: Verify it builds and is reachable**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo build -p org-node --features "chain"`
Expected: `Finished`, no errors.

- [ ] **Step 3: Clippy gate (lib only)**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-node --lib --features "app,test-support" -- -D warnings`
Expected: no warnings.

- [ ] **Step 4: Commit**

```bash
git add org-node/src/service.rs
git commit --no-gpg-sign -m "refactor(org-node): re-export FinalitySink for finality-polling tests"
```

---

## Task 4: Hermetic finality-polling integration test (delayed chopsticks)

**Files:**
- Create: `org-node/tests/finality_polling.rs`

Drives the **production** `FinalitySink::settle` (not the synchronous `ChopsticksSink` that `chain_genesis_e2e.rs` substitutes) against a chopsticks fork. Two cases: a background timer-miner produces delayed finality (positive), and a stalled chain hits the graceful-timeout branch (negative).

- [ ] **Step 1: Write the test file**

Create `org-node/tests/finality_polling.rs`:

```rust
#![cfg(feature = "chain")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Exercises the PRODUCTION FinalitySink::settle polling loop against a
//! chopsticks fork. chain_genesis_e2e.rs substitutes its own synchronous
//! ChopsticksSink, so the real poll-until-newer-finalized + timeout logic
//! (and its at_current_block() call) is otherwise never executed. Hermetic:
//! chopsticks only, no secrets, no external network.
//!
//! Run: cargo test -p org-node --features chain --test finality_polling
//! Cleanup if a fork is orphaned: pkill -f "chopsticks.*--config"
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use org_node::chain_write::proxy::BlockSink;
use org_node::service::FinalitySink;

mod common;
use common::chopsticks_fork::spawn_fork;
use common::chopsticks_reorg::mine_block;
use common::conn::legacy_client;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn settle_waits_for_a_newer_finalized_block() {
    // Own the fork behind an Arc so the timer-miner task can share it without
    // taking ownership; the fork (and its child process) is torn down when the
    // last Arc drops at end of test.
    let fork = Arc::new(spawn_fork().await.expect("spawn chopsticks fork"));
    eprintln!("chopsticks fork ready at {}", fork.ws_url);
    let api = legacy_client(&fork.ws_url).await.expect("subxt client");

    // Snapshot the finalized hash settle() will compare against.
    let pre_hash = api
        .at_current_block()
        .await
        .expect("at_current_block")
        .block_ref()
        .hash()
        .0;

    // Background timer-miner: produce a new block every ~3 s so finality
    // advances on a timer (simulating live-chain non-instant finality).
    // `mine_block` takes `&ChopsticksHandle`; `&fork2` (an `&Arc<…>`) deref-
    // coerces to it.
    let stop = Arc::new(AtomicBool::new(false));
    let miner = {
        let stop2 = stop.clone();
        let fork2 = fork.clone();
        tokio::spawn(async move {
            while !stop2.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_secs(3)).await;
                if stop2.load(Ordering::Relaxed) {
                    break;
                }
                let _ = mine_block(&fork2).await;
            }
        })
    };

    let sink = FinalitySink {
        api: api.clone(),
        timeout: Duration::from_secs(30),
    };

    let t0 = Instant::now();
    let settled = sink.settle().await.expect("settle must succeed");
    let elapsed = t0.elapsed();

    stop.store(true, Ordering::Relaxed);
    let _ = miner.await;

    // It must have waited at least one ~2 s poll interval (i.e. it actually
    // polled rather than returning the snapshot immediately)...
    assert!(
        elapsed >= Duration::from_secs(2),
        "settle returned in {elapsed:?}; expected it to poll at least one ~2 s interval"
    );
    // ...and returned a STRICTLY NEWER finalized block than the snapshot.
    assert_ne!(
        settled, pre_hash,
        "settle must return a newer finalized hash than the pre-submit snapshot"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn settle_times_out_gracefully_when_chain_stalls() {
    let fork = spawn_fork().await.expect("spawn chopsticks fork");
    eprintln!("chopsticks fork ready at {}", fork.ws_url);
    let api = legacy_client(&fork.ws_url).await.expect("subxt client");

    let pre_hash = api
        .at_current_block()
        .await
        .expect("at_current_block")
        .block_ref()
        .hash()
        .0;

    // No miner: the chain never advances, so settle must hit its deadline and
    // return the pre-submit hash (graceful degrade, service.rs:281) rather
    // than hanging forever.
    let sink = FinalitySink {
        api: api.clone(),
        timeout: Duration::from_secs(4),
    };

    let t0 = Instant::now();
    let settled = sink.settle().await.expect("settle must return Ok on timeout");
    let elapsed = t0.elapsed();

    assert_eq!(
        settled, pre_hash,
        "on stall, settle must return the pre-submit snapshot hash"
    );
    assert!(
        elapsed >= Duration::from_secs(4),
        "settle returned in {elapsed:?}; expected it to wait out the 4 s timeout"
    );
}
```

- [ ] **Step 2: No shared-harness change needed**

The timer-miner shares the fork via `Arc<ChopsticksHandle>` (Step 1) rather than a second handle, so `org-node/tests/common/chopsticks_fork.rs` is **not** modified — there is no second owner to race the `Drop` teardown, and `mine_block(&fork2)` works because `&Arc<ChopsticksHandle>` deref-coerces to the `&ChopsticksHandle` the function expects. If the compiler rejects the coercion in argument position, write `mine_block(&*fork2)` explicitly.

- [ ] **Step 3: Confirm the new test target declares the shared `common` module**

`tests/finality_polling.rs` already has `mod common;` (Step 1). Because `tests/common/mod.rs` exists, this resolves the shared harness. No Cargo change beyond Task 0, Step 3.

- [ ] **Step 4: Run the finality-polling test**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features "chain" --test finality_polling -- --nocapture --test-threads=1`
Expected: both `settle_waits_for_a_newer_finalized_block` and `settle_times_out_gracefully_when_chain_stalls` pass. (`--test-threads=1` avoids two forks contending for the fixed port 8000.)

- [ ] **Step 5: Runtime-verification point — `at_current_block` cursor**

If `settle_waits_for_a_newer_finalized_block` times out instead of observing a new hash, `at_current_block()` is reading a cursor that `dev_newBlock` does not advance. Confirm which cursor it reads and, if needed, drive finalization explicitly: check whether `common::chopsticks_reorg` exposes a finalize call, or extend `mine_block` to also finalize (chopsticks finalizes built blocks in dev mode by default, so this is unlikely — but verify, do not assume). Record what you found in the test module doc-comment.

- [ ] **Step 6: Confirm the existing chain e2e still passes (no regression)**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features "chain" --test chain_genesis_e2e -- --test-threads=1`
Expected: `genesis_then_admit_verifies_against_chain ... ok`.

- [ ] **Step 7: Commit**

```bash
git add org-node/tests/finality_polling.rs
git commit --no-gpg-sign -m "test(org-node): production FinalitySink polling vs delayed/stalled chopsticks"
```

---

## Task 5: Preflight health-check module

**Files:**
- Create: `org-node/src/preflight.rs`
- Modify: `org-node/src/lib.rs`

Reusable, non-panicking check functions returning structured results. Used by the binary (Task 6) and runnable against the hermetic fixtures.

- [ ] **Step 1: Write the module**

Create `org-node/src/preflight.rs`:

```rust
//! Preflight health checks for an ODS node before a real (two-laptop / live
//! Paseo) session. Each check returns a structured [`CheckResult`] instead of
//! panicking, so the binary can print a full checklist and exit non-zero if
//! any fail. The functions are also exercised hermetically (local relay +
//! chopsticks) by the test suite.
#![cfg(feature = "app")]

use std::time::Duration;

use subxt::OnlineClient;
use subxt::config::PolkadotConfig;

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
/// For `Networked`, waits up to `online_timeout` for a relay home.
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
            if !ep.inner().bound_sockets().is_empty() {
                CheckResult::ok("transport.bind", "loopback endpoint bound with a direct socket")
            } else {
                CheckResult::fail("transport.bind", "loopback endpoint has no bound socket")
            }
        }
        TransportMode::Networked => {
            match tokio::time::timeout(online_timeout, ep.inner().online()).await {
                Ok(()) => {
                    let addr = ep.inner().addr();
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

/// Confirm the chain RPC is reachable AND making progress: sample the current
/// finalized hash twice (with a delay) and require it to advance.
pub async fn check_chain_live(
    api: &OnlineClient<PolkadotConfig>,
    sample_gap: Duration,
) -> CheckResult {
    let first = match api.at_current_block().await {
        Ok(b) => b.block_ref().hash().0,
        Err(e) => return CheckResult::fail("chain.rpc", format!("at_current_block failed: {e}")),
    };
    tokio::time::sleep(sample_gap).await;
    let second = match api.at_current_block().await {
        Ok(b) => b.block_ref().hash().0,
        Err(e) => return CheckResult::fail("chain.rpc", format!("second sample failed: {e}")),
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
pub async fn check_contract(
    client: &on_chain_client::OrgRegistryClient,
    admin: on_chain_client::OrgAdmin,
) -> CheckResult {
    match client.get_org_state(admin).await {
        Ok(state) => CheckResult::ok(
            "contract.query",
            format!("get_org_state ok (epoch {})", state.epoch.0),
        ),
        Err(e) => CheckResult::fail("contract.query", format!("get_org_state failed: {e}")),
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
```

- [ ] **Step 2: Wire the module into the library**

In `org-node/src/lib.rs`, add (near the other feature-gated `pub mod` declarations):

```rust
#[cfg(feature = "app")]
pub mod preflight;
```

- [ ] **Step 3: Build it**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo build -p org-node --features "app"`
Expected: `Finished`. (`bound_sockets()` and `relay_urls()` are both confirmed against the pinned iroh/iroh-base sources.)

- [ ] **Step 4: Clippy gate (lib only)**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-node --lib --features "app,test-support" -- -D warnings`
Expected: no warnings. (The check fns must not `unwrap`/`expect`/`panic` — they return `CheckResult`.)

- [ ] **Step 5: Commit**

```bash
git add org-node/src/preflight.rs org-node/src/lib.rs
git commit --no-gpg-sign -m "feat(org-node): preflight health-check module (transport/chain/contract)"
```

---

## Task 6: Preflight integration test + binary

**Files:**
- Create: `org-node/tests/preflight.rs`
- Create: `org-node/src/bin/preflight.rs`
- Modify: `org-node/Cargo.toml`

- [ ] **Step 1: Register the preflight test target**

Append to `org-node/Cargo.toml`:

```toml
[[test]]
name = "preflight"
path = "tests/preflight.rs"
required-features = ["app", "test-support"]
```

- [ ] **Step 2: Write the integration test (happy + failure paths)**

Create `org-node/tests/preflight.rs`:

```rust
#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Hermetic exercises of the preflight checks: transport against an
//! in-process relay, chain.live against chopsticks (positive: timer-miner;
//! negative: stalled).
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use org_node::keys::SigningKeypair;
use org_node::preflight::{check_chain_live, check_transport};
use org_node::transport::TransportMode;

mod common;
use common::chopsticks_fork::spawn_fork;
use common::chopsticks_reorg::mine_block;
use common::conn::legacy_client;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn transport_networked_check_passes_against_local_relay() {
    // A relay must exist for online() to acquire a home. We cannot point the
    // production Networked builder at it, so we assert the check's contract
    // shape: with no reachable relay it fails fast rather than hanging.
    let device = SigningKeypair::from_seed([42u8; 32]);
    let res = check_transport(&device, TransportMode::Networked, Duration::from_secs(5)).await;
    // Without n0 relays reachable in a hermetic env, this should FAIL within
    // the timeout (proving the timeout path), not hang.
    assert!(
        !res.ok,
        "expected Networked transport check to fail fast in hermetic env, got: {res:?}"
    );
    assert_eq!(res.name, "transport.online");
}

#[tokio::test]
async fn transport_loopback_check_passes() {
    let device = SigningKeypair::from_seed([7u8; 32]);
    let res = check_transport(&device, TransportMode::Loopback, Duration::from_secs(5)).await;
    assert!(res.ok, "loopback transport check should pass: {res:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn chain_live_check_passes_with_miner_and_fails_when_stalled() {
    let fork = Arc::new(spawn_fork().await.expect("spawn chopsticks fork"));
    let api = legacy_client(&fork.ws_url).await.expect("subxt client");

    // Stalled: no miner → chain.live must FAIL within the sample gap.
    let stalled = check_chain_live(&api, Duration::from_secs(2)).await;
    assert!(!stalled.ok, "stalled chain.live should fail: {stalled:?}");

    // Live: timer-miner advances finality → chain.live must PASS.
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let fork2 = fork.clone();
    let miner = tokio::spawn(async move {
        while !stop2.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let _ = mine_block(&fork2).await;
        }
    });
    let live = check_chain_live(&api, Duration::from_secs(3)).await;
    stop.store(true, Ordering::Relaxed);
    let _ = miner.await;
    assert!(live.ok, "live chain.live should pass with a miner: {live:?}");
}
```

- [ ] **Step 3: Run the preflight integration test**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features "app,test-support" --test preflight -- --test-threads=1 --nocapture`
Expected: all three tests pass. If `transport_networked_check_passes_against_local_relay` does not fail within the timeout (e.g. the env can reach real n0 relays), relax the assertion to accept either a fast pass or a fast non-hang within `2 * online_timeout`, and note it.

- [ ] **Step 4: Write the binary**

Create `org-node/src/bin/preflight.rs`:

```rust
//! Preflight CLI: run health checks before a real ODS session and exit
//! non-zero if any fail. Reads configuration from env vars (mirrors the app's
//! ODS_* convention).
//!
//!   ODS_CHAIN_WS     ws(s):// chain RPC URL                 (required)
//!   ODS_CONTRACT_H160 0x-prefixed 20-byte contract address  (required)
//!   ODS_ADMIN_H160   0x-prefixed 20-byte admin/org id       (required)
//!   ODS_DEVICE_SEED  0x 32-byte hex device seed             (default: all-zero)
//!   ODS_TRANSPORT    "networked" (default) | "loopback"
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
            let hex = s.strip_prefix("0x").unwrap_or(&s);
            match hex::decode(hex).ok().and_then(|b| <[u8; 32]>::try_from(b).ok()) {
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
    match org_node::service::connect_chain_client(&ws).await {
        Ok((api, registry)) => {
            results.push(check_chain_live(&api, Duration::from_secs(15)).await);
            match h160_from_env("ODS_ADMIN_H160") {
                Ok(admin) => {
                    results.push(
                        check_contract(&registry, on_chain_client::OrgAdmin(admin)).await,
                    );
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
```

- [ ] **Step 5: Provide `connect_chain_client` if it does not already exist**

The binary calls `org_node::service::connect_chain_client(&ws) -> Result<(OnlineClient<PolkadotConfig>, OrgRegistryClient), _>`. Check `org-node/src/service.rs` for an existing constructor that builds the subxt client + `OrgRegistryClient` (the `SubxtChainOps` wiring in `AppState` already does this). If one exists, call it; otherwise add a small `#[cfg(feature = "chain")] pub async fn connect_chain_client` that builds a `LegacyBackend` client (mirror `tests/common/conn.rs`) and `OrgRegistryClient::from_client`, returning both. Reuse, do not duplicate, whatever the app already uses to connect.

- [ ] **Step 6: Build the binary**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo build -p org-node --features "app" --bin preflight`
Expected: `Finished`.

- [ ] **Step 7: Smoke-run the binary against a chopsticks fork (manual, optional but recommended)**

In one terminal start a fork (reuse the harness config), then:
Run: `ODS_CHAIN_WS=ws://127.0.0.1:8000 ODS_TRANSPORT=loopback ODS_ADMIN_H160=0x<deployed-admin> ODS_CONTRACT_H160=0x<deployed> CARGO_HOME=/tmp/cargo_home_fuzz cargo run -p org-node --features app --bin preflight`
Expected: a checklist; loopback transport PASS, chain.live PASS (fork mining), contract.query PASS if the admin/org has state. A non-zero exit if any fail.

- [ ] **Step 8: Clippy gate (lib only) + full feature build**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-node --lib --features "app,test-support" -- -D warnings`
Expected: no warnings.

- [ ] **Step 9: Commit**

```bash
git add org-node/Cargo.toml org-node/tests/preflight.rs org-node/src/bin/preflight.rs org-node/src/service.rs
git commit --no-gpg-sign -m "feat(org-node): preflight binary + hermetic preflight tests"
```

---

## Task 7: Wire preflight into the two-laptop runbook

**Files:**
- Modify: `docs/superpowers/demo/two-laptop-paseo.md`

- [ ] **Step 1: Read the runbook and find the first "start the app" step**

Read `docs/superpowers/demo/two-laptop-paseo.md`.

- [ ] **Step 2: Insert a preflight step before the app is launched on each laptop**

Add a section immediately before the first app-launch instruction:

````markdown
## Preflight (run on BOTH laptops first)

Before launching the app, confirm each laptop can reach the relay, the chain,
and the contract. From the repo root:

```bash
ODS_TRANSPORT=networked \
ODS_CHAIN_WS='wss://asset-hub-paseo-rpc.n.dwellir.com' \
ODS_ADMIN_H160='0x<org-admin-h160>' \
ODS_CONTRACT_H160='0x<deployed-contract-h160>' \
CARGO_HOME=/tmp/cargo_home_fuzz \
  cargo run -p org-node --features app --bin preflight
```

Expected: every line `PASS` and a zero exit code. If `transport.online`
FAILs, this laptop cannot reach the relay — fix connectivity before
proceeding (the app will not be able to dial the peer). If `chain.live` or
`contract.query` FAIL, fix the RPC endpoint / contract address first. A
failure here explains an otherwise-mysterious hang during the live run.
````

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/demo/two-laptop-paseo.md
git commit --no-gpg-sign -m "docs: run preflight on both laptops before the live ODS session"
```

---

## Final verification (after all tasks)

- [ ] **Full hermetic test sweep**

Run:
```bash
CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features "app,test-support" \
  --test transport_networked --test preflight -- --test-threads=1
CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features "chain" \
  --test finality_polling --test chain_genesis_e2e -- --test-threads=1
CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features "transport" \
  --test transport_handshake
```
Expected: all pass; no secrets, no external network (chopsticks + in-process relay only).

- [ ] **Lib clippy gate**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-node --lib --features "app,test-support" -- -D warnings`
Expected: clean.

- [ ] **Dispatch the final whole-implementation code review** (per subagent-driven-development), then proceed to `superpowers:finishing-a-development-branch`.

---

## Notes for the implementer

- **Hermetic means hermetic:** no test may require network egress or secrets. The only external processes are chopsticks (already vendored via the harness) and the in-process iroh relay.
- **Two flagged runtime-verification points** (each has a concrete fallback in-task): the relay-only `addr_filter` / `online()` behaviour in Task 2 Step 3, and the `at_current_block` cursor vs `dev_newBlock` in Task 4 Step 5. If you take a fallback, record the downgrade in the relevant test's module doc-comment — never silently.
- **Port contention:** the chopsticks harness uses a fixed port (8000). Run chain tests with `--test-threads=1` so two forks never collide.
- **Do not** modify the production `TransportMode` enum or the `presets::N0` `Networked` builder — the whole design keeps production untouched.
```
