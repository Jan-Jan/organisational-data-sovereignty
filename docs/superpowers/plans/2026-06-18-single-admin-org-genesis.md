# Single-Admin Org Genesis — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let an org be created with a single admin (no co-signer / no multisig) — the admin directly controls the org's pure proxy `P` — while keeping the multisig path and shaping the dispatch layer so real threshold-≥2 multisig drops in later.

**Architecture:** Genesis still creates pure proxy `P` (`org_id = h160_of(P)`). A renamed dispatch helper `dispatch_org_call` branches on whether co-signers exist: empty ⇒ submit the call as a top-level extrinsic signed by the admin (direct); non-empty ⇒ the existing `as_multi_threshold_1` wrapper. It returns a `DispatchOutcome` enum (`Executed(events)` now; `ApprovalRecorded` reserved for threshold ≥2) so callers match exhaustively and the threshold-≥2 build can't bypass them.

**Tech Stack:** Rust, subxt 0.50 dynamic API (`subxt::dynamic`, `subxt::transactions`), pallet-revive/pallet-proxy/pallet-multisig on a chopsticks Paseo-AH fork.

**Spec:** `docs/superpowers/specs/2026-06-18-single-admin-org-genesis-design.md`

---

## Context for the implementer

Current write path (already on this branch, post the watch-the-extrinsic fix):
- `org-node/src/chain_write/multisig.rs`:
  - `async fn submit_and_watch<Call: subxt::transactions::Payload>(api, sink, signer, tx) -> Result<ExtrinsicEvents<PolkadotConfig>, WriteError>` — submit, drive `sink`, wait for the extrinsic's own finalized success, return its events.
  - `pub async fn dispatch_threshold_1(sink, api, signer, other_signatories, call: Value) -> Result<ExtrinsicEvents<PolkadotConfig>, WriteError>` — sorts `others`, wraps `call` in `dynamic::tx("Multisig","as_multi_threshold_1",[others, call])`, calls `submit_and_watch`.
  - `pub async fn fund(sink, api, from, dest, amount) -> Result<(), WriteError>` — unchanged by this plan.
- `org-node/src/chain_write/proxy.rs`:
  - `fn create_pure_call() -> Value`, `pub fn proxied(pure_proxy, call: Value) -> Value`, `fn account32_from_named_field(...)`.
  - `pub async fn create_pure_via_multisig(sink, api, signer, others) -> Result<[u8;32], WriteError>` — calls `dispatch_threshold_1(...)`, scans the returned events for `Proxy.PureCreated`.
  - `pub async fn rotate(sink, api, pure_proxy, signer_old, others_old, old_multi, new_multi)` — two `dispatch_threshold_1` calls.
- `org-node/src/ceremony.rs`: `genesis_ceremony(sink, api, contract_h160, funder, admin, others, genesis_root, org_pub_key)` calls `create_pure_via_multisig` then `fund` then two `dispatch_threshold_1`.
- `org-node/src/service.rs`: `SubxtChainOps` has `pub others: Vec<[u8;32]>`; `submit_update` calls `dispatch_threshold_1(&sink, &self.api, &self.admin, &self.others, proxied(p, call))`.
- `org-node/src/chain_write/mod.rs`: `WriteError` enum (`Subxt(String)`, `EventNotFound(&'static str)`, `MalformedEvent(&'static str)`).
- `org-node/tests/chain_genesis_e2e.rs`: imports `dispatch_threshold_1, fund`; the multisig e2e uses `&[bob_pub]`. Deploys the EVM artifact via `deploy-chopsticks-evm.mjs` (already on branch).

Key subxt facts: `subxt::dynamic::tx(pallet, call, fields: impl Into<Composite>)` builds a `subxt::transactions::DynamicPayload`. A composed RuntimeCall `Value` is `Variant(pallet, Unnamed([Variant(call_name, fields_composite)]))`. The direct path must turn that composed `Value` back into a top-level `DynamicPayload`.

Build/verify: `CARGO_HOME=/tmp/cargo_home_fuzz`. Chopsticks tests need `npm ci` already done in `on-chain/scripts/` and a free port 8000; run with `--test-threads=1`. The EVM e2e deploy needs `forge` on PATH (Foundry).

---

## Task 1: Add `DispatchOutcome` and a RuntimeCall→payload helper

**Files:**
- Modify: `org-node/src/chain_write/mod.rs` (add `DispatchOutcome`)
- Modify: `org-node/src/chain_write/multisig.rs` (add `runtime_call_to_tx`)

- [ ] **Step 1: Add the outcome enum to `mod.rs`**

After the `WriteError` enum in `org-node/src/chain_write/mod.rs`, add:

```rust
use subxt::config::PolkadotConfig;
use subxt::extrinsics::ExtrinsicEvents;

/// Outcome of submitting one signer's contribution to an org-controlling call.
///
/// Today's controllers (a single admin signing directly, or a threshold-1
/// multisig) always execute in one transaction, so only `Executed` is produced.
/// The variant set is defined now so callers match exhaustively: when the
/// threshold-≥2 multisig flow is added, it produces `ApprovalRecorded` for
/// non-final approvals and the compiler forces every call site to handle the
/// pending case rather than silently treating it as success.
#[derive(Debug)]
pub enum DispatchOutcome {
    /// The inner call executed in this transaction; carries the executed call's
    /// own events (e.g. `Proxy.PureCreated`, the `Revive.update` effect).
    Executed(ExtrinsicEvents<PolkadotConfig>),
    /// The signature was recorded but the multisig threshold is not yet met —
    /// the inner call has NOT executed. Reserved for the threshold-≥2 flow; its
    /// payload (call hash, timepoint, approvals) is finalised when that ships.
    #[allow(dead_code)]
    ApprovalRecorded,
}
```

- [ ] **Step 2: Run a check to confirm it compiles**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo check -p org-node --features chain`
Expected: compiles (warnings OK).

- [ ] **Step 3: Add `runtime_call_to_tx` to `multisig.rs`**

This converts a composed RuntimeCall `Value` (as built by `proxied`/`create_pure_call`) into a top-level `DynamicPayload` for the direct path. Add near the top of `org-node/src/chain_write/multisig.rs` (after imports):

```rust
use subxt::ext::scale_value::ValueDef;
use subxt::transactions::DynamicPayload;

/// Turn a composed RuntimeCall `Value` — shaped `Variant(pallet,
/// Unnamed([Variant(call_name, fields)]))` by `proxied`/`create_pure_call` — into
/// a top-level extrinsic payload. Used by the single-admin (direct) dispatch
/// path, which submits the call itself rather than nesting it in
/// `as_multi_threshold_1`.
fn runtime_call_to_tx(call: Value) -> Result<DynamicPayload, WriteError> {
    let ValueDef::Variant(pallet_var) = call.value else {
        return Err(WriteError::MalformedEvent("call is not a RuntimeCall variant"));
    };
    let pallet_name = pallet_var.name;
    let mut inner = pallet_var
        .values
        .into_values()
        .next()
        .ok_or(WriteError::MalformedEvent("RuntimeCall has no inner call"))?;
    let ValueDef::Variant(call_var) = inner.value else {
        return Err(WriteError::MalformedEvent("inner call is not a variant"));
    };
    Ok(subxt::dynamic::tx(pallet_name, call_var.name, call_var.values))
}
```

Note for implementer: confirm the scale-value accessors against the pinned crate (`Composite::into_values()` yields owned `Value`s; `Variant { name, values: Composite }`). If a method name differs, adjust — the shape (pallet variant → single unnamed inner → call variant + its composite) is the contract. `let _ = &mut inner;` is not needed; drop the `mut` if the borrow checker is happy with `inner.value` by move.

- [ ] **Step 4: Check compiles**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo check -p org-node --features chain`
Expected: compiles.

- [ ] **Step 5: Commit**

```bash
git add org-node/src/chain_write/mod.rs org-node/src/chain_write/multisig.rs
git commit --no-gpg-sign -m "feat(chain): add DispatchOutcome + RuntimeCall->payload helper"
```

---

## Task 2: `dispatch_threshold_1` → `dispatch_org_call` (branch + outcome)

**Files:**
- Modify: `org-node/src/chain_write/multisig.rs`

- [ ] **Step 1: Factor a pure `build_dispatch_tx` and rewrite the dispatcher**

Replace `pub async fn dispatch_threshold_1(...)` with a pure tx-builder plus the async dispatcher:

```rust
/// Build the extrinsic that dispatches `call` under the current controller:
/// `others` empty ⇒ the call as a top-level extrinsic (single admin, direct);
/// `others` non-empty ⇒ `Multisig.as_multi_threshold_1(others, call)`
/// (threshold-1 "any one of N"). Pure (no chain I/O) so it is unit-testable.
fn build_dispatch_tx(
    other_signatories: &[[u8; 32]],
    call: Value,
) -> Result<DynamicPayload, WriteError> {
    if other_signatories.is_empty() {
        return runtime_call_to_tx(call);
    }
    let mut sorted_others: Vec<[u8; 32]> = other_signatories.to_vec();
    sorted_others.sort();
    let others: Vec<Value> = sorted_others
        .iter()
        .map(|id| Value::from_bytes(id.as_slice()))
        .collect();
    Ok(dynamic::tx(
        "Multisig",
        "as_multi_threshold_1",
        vec![Value::unnamed_composite(others), call],
    ))
}

/// Dispatch `call` as the org's controller, contributing `signer`'s signature,
/// driving block production via `sink`, and returning the structured outcome.
/// Single admin (empty `others`) and threshold-1 multisig both execute in one
/// transaction, so this returns `DispatchOutcome::Executed`.
pub async fn dispatch_org_call(
    sink: &dyn BlockSink,
    api: &OnlineClient<PolkadotConfig>,
    signer: &Keypair,
    other_signatories: &[[u8; 32]],
    call: Value,
) -> Result<DispatchOutcome, WriteError> {
    let tx = build_dispatch_tx(other_signatories, call)?;
    let events = submit_and_watch(api, sink, signer, &tx).await?;
    Ok(DispatchOutcome::Executed(events))
}
```

Add `use crate::chain_write::DispatchOutcome;` to the imports.

- [ ] **Step 2: Check compiles (call sites will still reference the old name — expect errors there, fixed in later tasks)**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo check -p org-node --features chain 2>&1 | grep -E 'dispatch_threshold_1|error' | head`
Expected: errors only about `dispatch_threshold_1` not found at call sites (proxy.rs, ceremony.rs, service.rs). Those are fixed in Tasks 3–4.

- [ ] **Step 3: Commit**

```bash
git add org-node/src/chain_write/multisig.rs
git commit --no-gpg-sign -m "refactor(chain): dispatch_org_call branches direct vs multisig, returns DispatchOutcome"
```

---

## Task 3: `create_pure_via_multisig` → `create_pure`; `rotate` updated

**Files:**
- Modify: `org-node/src/chain_write/proxy.rs`

- [ ] **Step 1: Rewrite `create_pure_via_multisig` as `create_pure`**

Replace the function with:

```rust
/// Create the org's pure proxy `P` under the current controller and return `P`.
/// Single admin (`others` empty): the admin signs `Proxy.create_pure` directly
/// and becomes `P`'s delegate. Threshold-1 multisig: the multisig creates `P`.
/// `P` is read from the `Proxy.PureCreated` event of the executed extrinsic.
pub async fn create_pure(
    sink: &dyn BlockSink,
    api: &OnlineClient<PolkadotConfig>,
    signer: &Keypair,
    others: &[[u8; 32]],
) -> Result<[u8; 32], WriteError> {
    let events = match dispatch_org_call(sink, api, signer, others, create_pure_call()).await? {
        DispatchOutcome::Executed(events) => events,
        // Single-signer / threshold-1 always executes; a pending approval here
        // would mean a threshold-≥2 controller, which create_pure does not drive.
        DispatchOutcome::ApprovalRecorded => {
            return Err(WriteError::Subxt(
                "create_pure: multisig approval pending; threshold-≥2 not supported here".into(),
            ));
        }
    };
    for ev in events.iter() {
        let ev = ev.map_err(|e| WriteError::Subxt(format!("event iter: {e}")))?;
        if ev.pallet_name() == "Proxy" && ev.event_name() == "PureCreated" {
            let fields: Composite<()> = ev
                .decode_fields_unchecked_as()
                .map_err(|e| WriteError::Subxt(format!("decode fields: {e}")))?;
            return account32_from_named_field(&fields, "pure");
        }
    }
    Err(WriteError::EventNotFound("Proxy.PureCreated"))
}
```

Update the import line `use crate::chain_write::multisig::dispatch_threshold_1;` →
`use crate::chain_write::multisig::dispatch_org_call;` and add
`use crate::chain_write::DispatchOutcome;`.

- [ ] **Step 2: Update `rotate` to use `dispatch_org_call` (ignore the outcome)**

In `rotate`, change both `dispatch_threshold_1(sink, api, signer_old, others_old, proxied(...))` calls to `dispatch_org_call(...)` and discard the returned outcome (rotate dispatches add_proxy/remove_proxy; success is all it needs):

```rust
    dispatch_org_call(sink, api, signer_old, others_old, proxied(pure_proxy, add_proxy_call(new_multi))).await?;
    dispatch_org_call(sink, api, signer_old, others_old, proxied(pure_proxy, remove_proxy_call(old_multi))).await?;
    Ok(())
```

(`?` on a `Result<DispatchOutcome, _>` whose `Ok` is unused is fine.)

- [ ] **Step 3: Check (ceremony.rs/service.rs/e2e still broken — next tasks)**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo check -p org-node --features chain 2>&1 | grep -E 'create_pure_via_multisig|dispatch_threshold_1' | head`
Expected: remaining references only in ceremony.rs, service.rs, tests.

- [ ] **Step 4: Commit**

```bash
git add org-node/src/chain_write/proxy.rs
git commit --no-gpg-sign -m "refactor(chain): create_pure (was create_pure_via_multisig); rotate uses dispatch_org_call"
```

---

## Task 4: Update `ceremony.rs` and `service.rs` call sites

**Files:**
- Modify: `org-node/src/ceremony.rs`
- Modify: `org-node/src/service.rs`

- [ ] **Step 1: ceremony.rs — rename imports + calls**

Change imports:
```rust
use crate::chain_write::multisig::{dispatch_org_call, fund, FUND_AMOUNT};
use crate::chain_write::proxy::{create_pure, map_account_call, proxied, BlockSink};
```
Change the body:
```rust
    let p = create_pure(sink, api, admin, others).await?;
    fund(sink, api, funder, p, FUND_AMOUNT).await?;
    dispatch_org_call(sink, api, admin, others, proxied(p, map_account_call())).await?;
    let call = revive_update_runtime_call(contract_h160, genesis_root, org_pub_key, 0);
    dispatch_org_call(sink, api, admin, others, proxied(p, call)).await?;
```
(The two `dispatch_org_call` calls discard the `Ok(DispatchOutcome)` via `?` — genesis steps need only that they executed; `Executed` is the only outcome these controllers produce.)

- [ ] **Step 2: service.rs — rename the dispatch in `submit_update`**

In `org-node/src/service.rs`, change the import `use crate::chain_write::multisig::dispatch_threshold_1;` → `dispatch_org_call`, and the call:
```rust
            dispatch_org_call(&sink, &self.api, &self.admin, &self.others, proxied(p, call))
                .await
                .map_err(write_err)?;
```

- [ ] **Step 3: Build the lib**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo build -p org-node --features app`
Expected: builds cleanly (no references to the old names remain in `src/`).

- [ ] **Step 4: Commit**

```bash
git add org-node/src/ceremony.rs org-node/src/service.rs
git commit --no-gpg-sign -m "refactor(chain): ceremony + service use dispatch_org_call / create_pure"
```

---

## Task 5: Unit test for `build_dispatch_tx` (bare vs multisig)

**Files:**
- Modify: `org-node/src/chain_write/multisig.rs` (add `#[cfg(test)]` test)

- [ ] **Step 1: Write the test**

In the existing `#[cfg(test)] mod tests` of `multisig.rs` (or add one), assert the built payload's pallet/call name:

```rust
#[test]
fn build_dispatch_tx_selects_direct_vs_multisig() {
    use subxt::dynamic::Value;
    use subxt::ext::scale_value::Composite;
    // A minimal composed RuntimeCall: System.remark { remark } (any pallet/call works).
    let call = || {
        Value::variant(
            "System",
            Composite::unnamed(vec![Value::variant(
                "remark",
                Composite::named(vec![("remark".to_string(), Value::from_bytes([1u8; 4]))]),
            )]),
        )
    };

    // Empty others → direct: the call itself becomes the extrinsic.
    let direct = build_dispatch_tx(&[], call()).expect("direct payload");
    assert_eq!(direct.pallet_name(), "System");
    assert_eq!(direct.call_name(), "remark");

    // Non-empty others → wrapped in Multisig.as_multi_threshold_1.
    let multi = build_dispatch_tx(&[[9u8; 32]], call()).expect("multisig payload");
    assert_eq!(multi.pallet_name(), "Multisig");
    assert_eq!(multi.call_name(), "as_multi_threshold_1");
}
```

Note for implementer: confirm the `DynamicPayload` accessor names against subxt 0.50 (`pallet_name()`/`call_name()`; if they differ, use the available getters or validate via `Payload` encoding against test metadata). If no name getter exists, fall back to asserting on `format!("{:?}", payload)` or skip the name assert and assert `is_ok()` shape only — but prefer a real name check.

- [ ] **Step 2: Run the unit test**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features chain --lib build_dispatch_tx`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add org-node/src/chain_write/multisig.rs
git commit --no-gpg-sign -m "test(chain): build_dispatch_tx picks direct vs as_multi_threshold_1"
```

---

## Task 6: Update e2e imports + add the single-admin genesis e2e

**Files:**
- Modify: `org-node/tests/chain_genesis_e2e.rs`

- [ ] **Step 1: Rename the import + the existing multisig call site**

Change `use org_node::chain_write::multisig::{dispatch_threshold_1, fund, FUND_AMOUNT};` →
`use org_node::chain_write::multisig::{dispatch_org_call, fund, FUND_AMOUNT};`
and the admit-step call `dispatch_threshold_1(&sink, &api, &alice, &[bob_pub], proxied(p, update_call))` →
`dispatch_org_call(&sink, &api, &alice, &[bob_pub], proxied(p, update_call))` (discard the `Ok` via `.expect(...)` as before).

- [ ] **Step 2: Add a single-admin genesis test**

Add a second `#[tokio::test(flavor = "multi_thread")]` (mirroring the existing test's setup) named `single_admin_genesis_e2e`. It MUST: spawn a fresh fork, deploy the contract (reuse `deploy_org_registry()`), build the same genesis trie, then run `genesis_ceremony` with **`others = &[]`** and **`alice` as both funder and admin** (no multisig account to fund — skip the alice+bob multisig funding step entirely), then assert epoch 1 and the genesis root, then admit member B via `dispatch_org_call(&sink, &api, &alice, &[], proxied(p, update_call))` and assert epoch 2 / seq 2. Reuse the existing test's helper fns (`admin_leaf`, `member_b_leaf`, etc.).

The single-admin genesis call:
```rust
    let outcome = genesis_ceremony(
        &sink, &api, contract,
        &alice,   // funder
        &alice,   // admin (signs directly)
        &[],      // others: empty ⇒ single-admin / direct
        *genesis_root.as_bytes(),
        org_pub_key,
    )
    .await
    .expect("single-admin genesis ceremony");
```

Because the two tests share fixed port 8000, they must run serially (`--test-threads=1`). Add a module note to that effect.

- [ ] **Step 3: Compile the tests**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features chain --test chain_genesis_e2e --no-run`
Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add org-node/tests/chain_genesis_e2e.rs
git commit --no-gpg-sign -m "test(chain): single-admin genesis e2e (others=[]); rename to dispatch_org_call"
```

---

## Task 7: Full verification

- [ ] **Step 1: clippy gate (`--lib`)**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-node --features app --lib`
Expected: clean (no `unwrap_used`/`expect_used`/`panic` in lib; `ApprovalRecorded`'s `#[allow(dead_code)]` keeps it quiet).

- [ ] **Step 2: lib unit tests + non-chain e2e**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app --lib`
Then: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app --test service_stories`
Expected: all pass.

- [ ] **Step 3: both chopsticks e2es (single-admin + multisig)**

Ensure port 8000 is free (`pkill -f chopsticks`) and `forge` is on PATH. Run:
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features chain --test chain_genesis_e2e -- --test-threads=1 --nocapture`
Expected: both tests pass — multisig (epoch 1 → 2) and single-admin (epoch 1 → 2). Single-admin proves no `TooFewSignatories` and the direct path executes.

- [ ] **Step 4: Final commit (if any verification fixups were needed)**

```bash
git add -A && git commit --no-gpg-sign -m "test(chain): verification fixups for single-admin genesis"
```

---

## Self-Review (plan vs spec)

- **Spec coverage:** §4 controller model → Task 2 (`build_dispatch_tx` branch); §5.1 names/mechanics → Tasks 2–4 (`dispatch_org_call`, `create_pure`, `runtime_call_to_tx`); §5.2 outcome seam → Task 1 (`DispatchOutcome`) + exhaustive matching in Task 3/ceremony; §6 upgrade path → no build (rotate kept working, Task 3 Step 2); §7 tests → Tasks 5 (unit) + 6 (single-admin e2e) + 7 (both e2es). All covered.
- **Type consistency:** `dispatch_org_call` returns `DispatchOutcome` everywhere; `create_pure` consumes it; ceremony/service/e2e discard `Ok` via `?`/`.expect`. `build_dispatch_tx` returns `DynamicPayload`; `runtime_call_to_tx` too.
- **No placeholders:** the only deferred item is `ApprovalRecorded`'s payload, explicitly reserved per spec §5.2 (not produced now).
- **Risks flagged inline:** scale-value accessor names (Task 1 Step 3) and `DynamicPayload` name getters (Task 5 Step 1) are the two spots to verify against the pinned subxt; both have stated fallbacks.
