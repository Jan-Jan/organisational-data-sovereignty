# ODS Phase 2 — Hermetic Coverage for the Networked + Live-Chain Paths

**Status:** design / spec
**Date:** 2026-06-18
**Author:** brainstorming session (Jan-Jan + Claude)
**Supersedes/extends:** `2026-06-15-ods-phase-2-poc-design.md` (testing section)

## 1. Problem

Phase 2 ships two code paths that are **compile-verified only** — not one line
has executed at runtime:

1. **`TransportMode::Networked`** (`org-node/src/transport/endpoint.rs`):
   `presets::N0` (n0 relays + DNS/Pkarr discovery) and `send_to_id` (dial
   purely by `EndpointId`). This is *the* two-laptop code path.
2. **`FinalitySink::settle`** (`org-node/src/service.rs`, `mod subxt_impl`):
   the poll-until-newer-finalized-block + timeout loop, including its
   `at_current_block()` call. The live-Paseo finality path.

What *is* runtime-verified: loopback iroh (relay disabled, direct `127.0.0.1`
dial via `inner().addr()`), the headless 5-story e2e, and a chopsticks
genesis→admit→verify e2e — but that chain test substitutes its own
`ChopsticksSink` (mines a block synchronously, `chain_genesis_e2e.rs:46`), so
it *bypasses* the production `FinalitySink`.

Consequently the two-laptop + live-Paseo manual run can fail for at least
three independent, untested reasons: our Networked transport code, our
finality-polling code, and real cross-NAT hole-punching.

## 2. Goal & non-goals

**Goal.** Exercise *all of our code* on the distributed paths with **hermetic,
CI-reproducible, secret-free** tests, shrinking the manual two-laptop run to
the single irreducible unknown (cross-NAT hole-punch between two distinct
networks), and gate even that behind fail-fast diagnostics.

**Hard floor (non-goal).** Two processes on one machine share a network, so
real cross-NAT traversal cannot be reproduced locally. We do not attempt it.
After this work the two-laptop run remains as *optional* confirmation of only
that last bit.

**Out of scope (explicitly chosen "hermetic only"):** no `#[ignore]` tests
against real n0 relays or live Paseo; no funded accounts; no secrets; nothing
that reaches external infrastructure.

## 3. Architecture

Three deliverables, all behind existing feature gates (`transport`, `chain`,
`app`) and a new `test-support` feature:

| # | Deliverable | Closes gap | Hermetic mechanism |
|---|-------------|-----------|--------------------|
| 1 | Networked-transport integration test | #1 (Networked transport) | in-process `iroh-relay` server |
| 2 | Finality-polling integration test | #2 (`FinalitySink`) | chopsticks + background timer miner |
| 3 | Preflight health-check harness | debuggability of the manual run | shared assertion fns, reused by #1/#2 |

### 3.1 Component 1 — Networked transport over a local relay

**Production stays unchanged.** `TransportMode` keeps its two variants;
`Networked` keeps `presets::N0`. We do **not** add a `Custom` variant to the
public enum (it is threaded through the service and would leak test config
into the production API).

**New test-support seam.** Add, behind a new `test-support` cargo feature, a
public constructor on `OrgEndpoint`:

```rust
// org-node/src/transport/endpoint.rs, behind #[cfg(feature = "test-support")]
/// Bind a Networked-style endpoint whose ONLY relay is `relay_map`, with
/// direct UDP paths disabled so traffic is forced through the relay. For
/// hermetic tests against an in-process iroh-relay; never used in production.
pub async fn bind_with_relay(
    device: &SigningKeypair,
    relay_map: iroh::RelayMap,
) -> Result<Self, TransportError>;
```

It mirrors the `Networked` builder (same ALPN, same secret-key-from-seed) but
swaps `presets::N0`'s relay/discovery for `RelayMode::Custom(relay_map)` and
no DNS discovery. Discovery-by-id is satisfied in-test by seeding the peer's
`EndpointAddr` (relay-only, no direct sockets) via
`Endpoint::add_node_addr`, so `send_to_id(EndpointId)` resolves through the
local relay rather than real DNS/Pkarr.

**The test** (`org-node/tests/transport_networked.rs`,
`#[cfg(all(feature = "transport", feature = "test-support"))]`):
1. Spawn an in-process `iroh-relay` server; obtain its `RelayUrl` → `RelayMap`.
2. `bind_with_relay` two endpoints (admin device, member device) against it.
3. Seed each endpoint with the other's relay-only `EndpointAddr` via
   `add_node_addr` (no direct socket addresses, forcing the relay datapath).
4. Run the genesis→admit→verify handshake from `transport_handshake.rs`, but
   dial with **`send_to_id(peer_device_id)`** (the Networked method) instead
   of `send(addr)`.
5. Assert: receiver gets the message, `recv_one` returns the
   cryptographically-authenticated remote device key, and the admit delta
   verifies — identical assertions to the loopback test, over the relay path.

**Covers:** the Networked builder config, relay datapath, `send_to_id`
dial-by-`EndpointId`, frame codec, admit/verify over the wire.
**Leaves uncovered (by design):** real cross-NAT traversal.

### 3.2 Component 2 — Finality polling via delayed chopsticks

**Required re-export.** Add to `org-node/src/service.rs`:
```rust
pub use subxt_impl::FinalitySink; // currently only SubxtChainOps is re-exported
```
so the production sink is constructible from an integration test.

**The test** (`org-node/tests/finality_polling.rs`, `#[cfg(feature = "chain")]`)
reuses `common::chopsticks_fork::spawn_fork` and
`common::chopsticks_reorg::mine_block`, with two cases:

*Positive — delayed finality.* Spawn a background task that calls
`mine_block` every ~3 s. Construct the production
`FinalitySink { api, timeout: 30 s }` against the fork and call `settle()`
directly (no extrinsic needed — we are testing the poll loop, not a write).
Assert it returns a **strictly newer** finalized hash than the snapshot taken
at call time, and that it returns *after* at least one poll interval (i.e. it
actually waited, rather than the bypassed synchronous path).

*Negative — stall → graceful timeout.* No background miner; construct
`FinalitySink { api, timeout: 4 s }`; assert `settle()` returns `Ok(pre_hash)`
(the snapshot hash) after roughly the timeout, proving the degrade-gracefully
branch (`service.rs:281`) rather than hanging.

**Covers:** the real `settle` poll/timeout logic and its `at_current_block()`
call, both first-executed here.
**Implementation verification point:** confirm `dev_newBlock` advances
whatever cursor `at_current_block()` reads (finalized vs best). If
`dev_newBlock` advances only the best block and not the finalized cursor, the
miner task must additionally drive finalization (chopsticks finalizes built
blocks in dev mode by default, but this must be asserted, not assumed).

### 3.3 Component 3 — Preflight health-check harness

A binary `org-node/src/bin/preflight.rs` (compiled under the `app` feature)
plus a reusable assertions module `org-node/src/preflight.rs`. Given an iroh
endpoint, a chain RPC URL, a contract H160, and the admin + co-signer
accounts, it runs and prints a pass/fail checklist:

- **transport:** endpoint binds; for `Networked`, a relay home is acquired and
  `addr()` reports a reachable address within a timeout.
- **chain RPC:** reachable; finalized head advances across two samples (chain
  is live, not stalled).
- **contract:** `Revive.PristineCode`/code present at the configured H160.
- **funds:** admin and co-signer free balances ≥ the threshold the write path
  needs (fees + storage deposit).

Each check returns a structured `CheckResult { name, ok, detail }`; the binary
prints them and exits non-zero if any fail. The assertion functions are pure
(take a connected client / endpoint) so they are **reused as setup guards in
Components 1 and 2** (pointed at the local relay + chopsticks), and by the
operator before a manual two-laptop session (pointed at real infra).

## 4. File-level change map

- **Modify** `org-node/Cargo.toml`: add `test-support` feature; add
  `iroh-relay` as a `dev-dependency` pinned compatible with `iroh 0.98`.
- **Modify** `org-node/src/transport/endpoint.rs`: add
  `#[cfg(feature = "test-support")] pub async fn bind_with_relay`.
- **Modify** `org-node/src/service.rs`: add `pub use subxt_impl::FinalitySink;`.
- **Create** `org-node/src/preflight.rs`: `CheckResult` + check fns (feature `app`).
- **Create** `org-node/src/bin/preflight.rs`: CLI wrapper (feature `app`).
- **Create** `org-node/tests/transport_networked.rs`: Component 1 test.
- **Create** `org-node/tests/finality_polling.rs`: Component 2 test.
- **Modify** `org-node/src/lib.rs`: export `preflight` module under `app`.
- **Update** `docs/superpowers/demo/two-laptop-paseo.md`: add a "run preflight
  first" step referencing the new binary.

## 5. Testing & verification

- Components 1 & 2 are themselves the tests; both must pass with **no network
  and no secrets** (`cargo test -p org-node --features "transport,chain,app,test-support"`).
- Existing loopback (`transport_handshake.rs`), headless 5-story
  (`service_stories.rs`), and chopsticks (`chain_genesis_e2e.rs`) tests stay
  green; fuzz targets unaffected.
- Clippy gate unchanged: `--lib` only, `-D unwrap_used/expect_used/panic`
  (tests may unwrap).
- Component 3's checks are unit-tested against the local relay + chopsticks
  (happy path) and against a deliberately-wrong H160 / unfunded account
  (failure path).

## 6. Risks

- **iroh-relay 0.98 spawn API.** The exact in-process relay-server constructor
  shifts between iroh versions and must be pinned at implementation time.
  *Fallback if impractical:* a weaker Component 1 that still exercises
  `send_to_id` + the `Networked` builder but routes direct (no relay hop) —
  this downgrade must be flagged explicitly in the test's module doc, not
  accepted silently.
- **`at_current_block()` cursor semantics** under chopsticks `dev_newBlock`
  (see §3.2 verification point).

## 7. What remains manual after this work

Only cross-NAT hole-punch between two distinct real networks. The two-laptop
runbook stays, but is optional and preflight-gated.
