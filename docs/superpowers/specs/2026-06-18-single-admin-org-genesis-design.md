# ODS — Single-Admin Org Genesis (with stable-identity upgrade path)

**Status:** design / spec
**Date:** 2026-06-18
**Author:** brainstorming session (Jan-Jan + Claude)
**Affects:** `org-node` write path (`chain_write/`, `ceremony.rs`, `service.rs`) and the chopsticks e2e

## 1. Problem

`genesis_ceremony` runs every on-chain step through `dispatch_threshold_1`,
which wraps the call in `Multisig.as_multi_threshold_1(others, call)`, and
`create_pure_via_multisig` has the **multisig pseudo-account** create the org's
pure proxy `P`. pallet-multisig requires **≥2 signatories**, so when no
co-signer is supplied (`others` empty) the genesis write fails on-chain with
`Multisig::TooFewSignatories`.

We want to **create an organisation with a single admin and no co-signer**, and
to be able to **add a signatory and adjust the multisig threshold later without
changing the org's on-chain identity**.

## 2. Goal & non-goals

**Goal (build now).** A single-admin genesis path: the admin account directly
controls the org's pure proxy `P` (signs every genesis extrinsic itself, no
multisig, no co-signer), producing `org_id = h160_of(P)` exactly as today.

**Design-for (do not build now).** The transitions — add a signatory / move to
a multisig, and adjust the threshold — are specified here so the genesis design
does not preclude them, but no code for them ships in this iteration. The
dispatch layer's **names and return type are shaped now** so the real
threshold-≥2 multisig flow drops in later without reshaping (§5.2, §6).

**Non-goals.** No app/UI affordances for managing admins or threshold. No real
threshold-≥2 multisig flow (`as_multi`/`approve_as_multi`) built now. No change
to the EVM deploy or the watch-the-extrinsic write fix (already on this branch).
The existing threshold-1 multisig path keeps its on-chain **behaviour**, but is
renamed and now returns the `DispatchOutcome` seam (§5.2) — always `Executed`
for threshold-1, so no behavioural change.

## 3. Key invariant: the org identity is the pure proxy

`P` is created once at genesis and never changes; only `P`'s proxy **delegate**
(its controller) changes over the org's life. Because `org_id = h160_of(P)` is
derived from `P` — not from whoever controls it — the org's on-chain identity is
**stable** across single-admin → multisig → different-threshold-multisig
transitions. This is the reason genesis still creates a pure proxy even for a
single admin (rather than anchoring identity on the admin's own account).

## 4. Controller model

`P`'s controller (its proxy delegate) is one of three kinds. The write path
selects among them; this iteration builds the first two and *shapes the
interface* for the third.

| Controller | pallet-multisig call | Transactions | Built now? |
|---|---|---|---|
| **Admin (direct)** — single admin | none (top-level extrinsic) | 1, executes immediately | ✅ |
| **Multisig, threshold 1** — "any one of N acts alone" | `as_multi_threshold_1` | 1, executes immediately | ✅ (existing path, kept) |
| **Multisig, threshold ≥2** — "M of N must approve" | `as_multi` + `approve_as_multi` | M, from M different signers/sessions; the M-th executes the inner call | ⛔ shaped-for, not built |

The "1-of-2 with a phantom co-signer" used at present is the threshold-1 case
abused for a single admin — it only avoids `TooFewSignatories` by inventing a
co-signer that never signs. The **Admin (direct)** controller removes that
fakery; the threshold-1 *multisig* remains for genuine "any one of N" orgs.

**Crucial difference for threshold ≥2:** it is **not** one transaction from one
place. Each signer submits their **own** transaction from their **own** account
(hence different machines/sessions). The first creates a pending multisig
(`Multisig.NewMultisig`), middle approvals record (`Multisig.MultisigApproval`),
and only the **threshold-meeting** transaction *executes the inner call*
(`Multisig.MultisigExecuted` + the inner call's own events). A submitter learns
**"did I just approve, or did I complete it?"** *only from the on-chain
feedback* — which is why the dispatch layer must return a structured outcome
(§5.2), not a flat event list.

In the Phase 2 app, the controller is selected from `ODS_COSIGNER_PUB`
(`app/src-tauri/src/state.rs`): unset ⇒ Admin (direct); set ⇒ multisig. So unset
`ODS_COSIGNER_PUB` flips from *error* to *single-admin*; threshold and
multi-signer config arrive with the threshold-≥2 build.

## 5. Single-admin genesis flow

Identical structure to the multisig ceremony, but each step is signed **directly
by the admin** and run through the existing `submit_and_watch` helper
(submit → drive chain via sink → wait for the extrinsic's own finalized events,
surfacing `ExtrinsicFailed`):

1. **`Proxy.create_pure`** signed by the admin → the **admin** becomes `P`'s
   delegate (vs. the multisig pseudo-account today). `P` is read from the
   `Proxy.PureCreated` event; `org_id = h160_of(P)`.
2. **Fund `P`** — `Balances.transfer_keep_alive(P, FUND_AMOUNT)` signed by the
   funder (the admin). Unchanged.
3. **`Proxy.proxy(P, Revive.map_account)`** signed by the admin (the map_account
   runs as `P`; the outer signer is the admin, not a multisig).
4. **`Proxy.proxy(P, Revive.update(genesis_root, org_pub_key, expectedEpoch=0))`**
   signed by the admin.

### 5.1 Dispatch-layer mechanics

Where the multisig path wraps a call `Value` as
`dynamic::tx("Multisig", "as_multi_threshold_1", [others, call])`, the
single-admin path submits the call as a **top-level extrinsic**:
`dynamic::tx("Proxy", "create_pure", […])` and
`dynamic::tx("Proxy", "proxy", [real=P, force_proxy_type=None, inner_call])`.
The inner proxied call (`map_account`, `update`) stays a nested `Value`
argument to `Proxy.proxy` — only the outer wrapper differs (bare vs.
`as_multi_threshold_1`).

The branch lives in the dispatch helpers. Today's names mislead (they assume a
multisig), so rename to neutral names that take the controller and branch
internally:

- `dispatch_threshold_1(sink, api, signer, others, call)` →
  **`dispatch_org_call(sink, api, signer, others, call) -> DispatchOutcome`**:
  `others` empty ⇒ `submit_and_watch` the bare call (Admin/direct); else ⇒
  `as_multi_threshold_1` wrapper (threshold-1). Returns the structured outcome
  in §5.2.
- `create_pure_via_multisig(sink, api, signer, others)` → **`create_pure(…)`**:
  `others` empty ⇒ admin signs `create_pure` directly; else ⇒ threshold-1
  multisig creates it. Reads `PureCreated` from the **`Executed`** outcome's
  events in both cases.

`proxied(P, call)` (the `Proxy.proxy` wrapper) and the call builders
(`create_pure_call`, `map_account_call`, `revive_update_runtime_call`) are
reused unchanged; only how the outer extrinsic is formed changes.

`SubxtChainOps::{submit_genesis, submit_update}` (`service.rs`) pass their
existing `others` through to the renamed helpers, so both inherit the branch
with no behavioural change for the multisig case.

### 5.2 Dispatch outcome (the seam that scales to threshold ≥2)

`submit_and_watch` currently returns a flat `ExtrinsicEvents`. For a real
multisig that conflates "the inner call executed" with "an approval was
recorded." The dispatch layer instead returns:

```rust
enum DispatchOutcome {
    /// The inner call executed in this transaction; `events` are the executed
    /// call's own events (Proxy.PureCreated, the Revive.update effect, …).
    Executed(ExtrinsicEvents<PolkadotConfig>),
    /// This signature was recorded but the threshold is NOT yet met — the inner
    /// call has not executed. Carries what the next signer needs to continue:
    /// call hash, the multisig timepoint, and approvals-so-far. (Produced only
    /// by the threshold-≥2 path; see §6.)
    ApprovalRecorded { /* call_hash, timepoint, approvals, threshold */ },
}
```

**Built now:** Admin (direct) and threshold-1 multisig submissions always
execute, so `dispatch_org_call` only ever returns `Executed`. Callers
(`create_pure`, the ceremony, `submit_update`) **match exhaustively** and treat
`ApprovalRecorded` as an error in genesis context ("genesis cannot be left
pending"). Defining the enum now — even though only `Executed` is produced —
means the threshold-≥2 build adds the `ApprovalRecorded` arm and the compiler
forces every call site to decide what pending means. That is the "scale"
guarantee: new behaviour can't silently bypass existing callers.

(`ApprovalRecorded`'s payload is sketched, not finalised; the threshold-≥2
iteration fills it in. It may carry `#[allow(dead_code)]` until then.)

## 6. Upgrade path (design-for, not built)

All transitions are pure-proxy delegate edits on the fixed `P`; `org_id` is
invariant. Two distinct mechanisms are involved and must not be conflated:

**(a) Editing `P`'s delegates** (who/what controls the org). These are
`Proxy.add_proxy` / `Proxy.remove_proxy`, each dispatched by a *current*
delegate of `P`:

- **Add a signatory / go multisig.** A current delegate signs `Proxy.add_proxy(P,
  delegate = M, Any, 0)` where `M = multi_account_id([admin, admin₂, …], T)`,
  then optionally `Proxy.remove_proxy(P, delegate = admin)` so only `M` controls
  `P`. The existing `rotate()` performs add+remove.
- **Adjust threshold.** A multisig account is deterministic from
  `(sorted_signers, threshold)`, so changing `T` means rotating `P`'s delegate
  from `M(T_old)` to `M(T_new)` — again `add_proxy` then `remove_proxy`.

**(b) Dispatching a call once `M` is the delegate** (threshold ≥2). This is the
real multi-party flow that `DispatchOutcome` (§5.2) exists for:

- Signer 1 submits `Multisig.as_multi(T, others, None, proxied(P, call),
  max_weight)` → `dispatch_org_call` returns **`ApprovalRecorded`** (pending;
  `NewMultisig`), carrying the call hash + timepoint.
- Signers 2…T-1 submit `Multisig.approve_as_multi(T, others, Some(timepoint),
  call_hash, max_weight)` → each returns **`ApprovalRecorded`** (`MultisigApproval`).
- Signer T submits the final `as_multi` with the full call → returns
  **`Executed`** (`MultisigExecuted` + the inner call's events). Only this
  signer learns, from the feedback, that the call ran.

Each submission is a separate transaction from a separate signer/session;
coordination (sharing the call hash + timepoint between signers) is the
threshold-≥2 iteration's concern. This spec only guarantees that (1) genesis
leaves `P` controllable so `add_proxy`/`remove_proxy` are possible, and (2) the
dispatch interface already returns `DispatchOutcome`, so the threshold-≥2 flow
is a fill-in rather than a reshape. A future iteration adds the `ChainOps`
methods, the multi-session coordination, and (later) app UI.

## 7. Testing & verification

- **Single-admin e2e** (new): a chopsticks `chain_genesis_e2e` variant with
  `others = []` — genesis → epoch 1, admit → epoch 2 — proving the direct path
  end-to-end on a forked Paseo-AH runtime. Asserts every step's outcome is
  `Executed` (genesis is never left pending).
- **Multisig (threshold-1) e2e** (existing): the unchanged 1-of-2 "any one"
  path stays green and now also returns `Executed`.
- **Unit:** assert `dispatch_org_call` builds a bare extrinsic when `others` is
  empty and an `as_multi_threshold_1` extrinsic when non-empty (inspect the
  `DynamicPayload`'s pallet/call name), and that both yield `Executed`.
- **Deferred (threshold ≥2):** the `ApprovalRecorded` → … → `Executed` sequence
  and the per-signer feedback are tested when that path is built (it needs ≥2
  funded signers driving separate transactions); out of scope here.
- Clippy gate unchanged: `--lib`, `-D unwrap_used/expect_used/panic` (tests may
  unwrap).

## 8. Risks

- **`P` derivation depends on the creator.** A single-admin `P` (created by the
  admin) differs from a multisig-created `P`; this is expected — the org's
  identity is whatever `P` genesis produces. Switching control later does not
  re-derive `P`.
- **Direct `Proxy.create_pure` delegate semantics.** The single signer of
  `create_pure` becomes `P`'s delegate. Verified conceptually against
  pallet-proxy; the new single-admin e2e is the empirical check.
