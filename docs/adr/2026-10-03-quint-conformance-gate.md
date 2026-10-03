# ADR: Quint models live in the unit they model, and conformance gates every change

- **Date:** 2026-10-03
- **Status:** accepted (2026-10-03)
- **Relates to:** `docs/adr/2026-09-05-units-and-per-unit-classes.md` (the
  units manifest this changes: `quint` leaves `not_a_unit`)
- **Decision maker:** Jan-Jan (project owner), interviewed by `grill-requirements`

## Context

The Quint models sat in a top-level `quint/` directory, disclaimed in
`.guardrails/units.yaml` as `not_a_unit`. `org-members` already replayed
simulator traces of `quint/membership_mbt.qnt` against the real `OrgTrie`
through `quint-connect` 0.1.2 (`org-members/tests/mbt_conformance.rs`), but
the models themselves were outside every unit: a change to a model ran no
unit's gate and entered no unit's traceability, and the round-trip law and
model invariants ran only in CI, never at the merge gate.

`protocol.qnt` (org-node's protocol model) imported `membership.*` but, since
the abstract-root remodel, used exactly one symbol from it: the `Key` record.

## Decision

1. **Placement.** `membership.qnt` and `membership_mbt.qnt` move to
   `org-members/quint/`; `protocol.qnt` and `ods_instances.qnt` move to
   `org-node/quint/`. `quint` leaves `not_a_unit`; the models fall under the
   compliance scope of the unit that implements them.
2. **Exported Quint interface.** The membership types at the boundary (`Key`,
   `Leaf`, `Snapshot`, `Delta`, `Result`) are split into
   `org-members/quint/membership_types.qnt`. `membership.qnt` imports it;
   org-node's `protocol.qnt` imports only it. The cross-unit import follows
   the declared `org-node -> org-members` edge. This is a design-level
   interface, not an `exported: yes` requirement.
3. **Gate.** Each unit's Quint checks (typecheck, `quint test`, simulator
   invariants) are unconditional entries in that unit's `verify_commands`; the
   per-unit `check-units.sh --impact` selection is the only "run when changed"
   mechanism. Apalache `quint verify` stays a best-effort CI job. The
   conformance test runs inside `cargo test -p org-members` on every change.
   Measured cost (quint 0.32.0, typescript backend): org-members' Quint
   checks ~6 s; org-node's five simulator invariants ~60 s.
4. **What "maps 1:1" means.** quint-connect's upstream practice (its examples,
   the Quint MBT docs, the quint.sh posts):
   (a) the membership model's `step` is a disjunction of named actions, each
   with exactly one driver arm calling one implementation entry point with the
   nondet picks as arguments; (b) a projection of state is compared after every
   step, abstractions made explicit in `from_driver`; (c) the implementation
   errs iff the membership model recorded an error; (d) `#[quint_run]` random
   simulation plus `#[quint_test]` named scenarios. **Stricter than
   upstream:** every public `OrgTrie` operation that yields a new trie or
   candidate has a model action; what the model deliberately omits is listed
   as a declared abstraction boundary. (Amended 2026-10-03, after independent
   review round 2: "spec" replaced by "membership model", the
   `org-members/docs/CONTEXT.md` term, here and in decision 11; in decision 13,
   where it named `protocol.qnt` and the planned node-level model, by "system
   model" and "model".) (Amended 2026-10-03, after independent review round
   3: (d) is not how the gate is built. Both the named scenarios and the
   random run call `quint_connect::runner::run_test` directly
   (`run_scenario` and `run_conformance` in
   `org-members/tests/mbt_conformance.rs`), not the `#[quint_test]` and
   `#[quint_run]` macros. Decision 8(ii) gives the reason for the random run:
   the macro consumes the driver and offers no end-of-run hook.)
5. **No new requirements for the gate itself.** These are
   verification-strategy decisions, not system behaviour; they are recorded
   here and in the change plan. New model actions carry existing LLRs through
   `verifies:`. One requirement *is* new, and it is about behaviour, not the
   gate: REQ-wx3wpv (a change set the software produced is accepted when
   applied to the record it was computed against), the hole PR-vf5hdm names.
   Handle swaps and handovers stay lawful — the owner chose not to forbid
   them, keeping the set of failures small — so the fix removes a rejection.
6. **MSRV 1.85.** quint-connect is edition 2024, so the test build needs Rust
   >= 1.85. `rust-version = "1.85"` in `[workspace.package]`, `org-members`,
   `org-node` and `app/src-tauri` (matching `on-chain-client`); editions stay
   2021. A CI job runs `cargo +1.85 test -p org-members` so the declared MSRV
   is measured — before this, every job used floating `stable` and the
   declared 1.81 was already false for the test build.
7. **Reproducibility.** Quint is **not** pinned; the Quint version in use is
   recorded in every verification record instead. Seeds stay random, so each
   run explores new traces; a failure is reproduced from the printed
   `QUINT_SEED`. `max_steps` is set explicitly, per unit — org-members'
   conformance run and org-node's simulator runs explore different state
   spaces and get different bounds. Accepted trade-off: a new Quint release
   can turn the gate red, or change what it explores, with no commit here; the
   recorded version is what makes that diagnosable.
8. **Enforcing the stricter 1:1 rule.** Two tests in the conformance test
   file, both failing closed:
   (i) a declared table maps every trie-yielding public `OrgTrie` operation to
   its model action or to its declared-abstraction-boundary entry, and a test
   matches the table against `src/trie.rs` signatures (`-> Result<Self`,
   `-> Result<CandidateTrie`) — an unmodelled new operation fails
   `cargo test`; (Amended 2026-10-03, after independent review round 3: the
   test, `trie_operations_table_matches_source`, scans every `pub fn` in two
   files, `src/trie.rs` and `src/delta.rs`, and counts a function as
   trie-yielding when its return type, whitespace removed, starts with one of
   four patterns: `Result<Self`, `Result<(Self`, `Result<CandidateTrie` or
   `Result<OrgTrie`. The set it finds must equal the `TRIE_OPERATIONS` plus
   `BOUNDARY` table exactly. That is stricter than the two patterns and one
   file stated above.)
   (ii) action coverage: the driver counts actions taken across all traces
   (via `quint_connect::runner::run_test` with a shared counter, since the
   `#[quint_run]` macro consumes the driver and offers no end-of-run hook);
   any never-taken action fails the test. Per-action Ok/Err counts are
   reported, not gated — rare error branches are pinned by `#[quint_test]`
   scenarios instead, because gating them under random seeds would be flaky.
   Being tests rather than guardrails scripts, neither needs tool
   qualification.
9. **Toolchain failure is a red gate.** `quint` absent from PATH, the rust
   evaluator unfetchable (quint-connect passes no `--backend`, so Quint's
   default rust backend downloads its evaluator into `~/.quint` on first use:
   network on a fresh machine, writable `$HOME` always), a Quint crash, or
   unparseable ITF — each fails the conformance test and the Quint
   `verify_commands`. No skip, no opt-out variable: the merge gate runs
   locally, and an opt-out there is a class-C merge without conformance. The
   test pre-checks PATH and evaluator availability and names the cause rather
   than panicking bare; the false "skips at runtime" header in
   `mbt_conformance.rs` is corrected. `quint typecheck` and `quint test` in
   `verify_commands` use `--backend=typescript`. (Amended 2026-10-03, during
   planning: `quint run --invariant=mbtInv` uses the default rust backend.
   With the `ApplyDelta` action the typescript simulator measured 6 traces/s
   — 32 s for 200 samples — against 306 traces/s on rust; and the conformance
   test in the same gate already depends on the evaluator, so this adds no
   failure mode.) (Amended 2026-10-03, after independent review round 2: only
   `quint test` takes `--backend=typescript`; the `quint typecheck` commands
   pass no backend flag (`org-members/.guardrails/config.yaml`).)
10. **The `ApplyDelta` model action.** A `prev` variable holds the previous
    trie; `ApplyDelta` nondeterministically picks an honest delta
    (`calculateDelta(trie, s')`, `s'` one **or two** mutations away — two, so
    that a handle handed between two present members is generated, the shape
    of open defect PR-vf5hdm, which part 1 fixes under `resolve-problem`
    with the handover scenario as its failing test), a stale-base delta
    (computed from `prev`), or a structurally malformed one built from a
    small universe (reaching `StaleRemoval`, `RemoveUpsertOverlap`,
    `NoOpUpsert`, `ConfusableHandle`, `DeviceSlotsFull`), then calls
    `verify_against` with the true target root or a wrong one (reaching
    `VerificationFailed`). Every model error tag maps to exactly one crate
    result — including the specific `MalformedDelta` reason, which the
    current `err_tag` collapses to `Other`. Encoding-level non-canonicality
    (ordering, duplicate list entries) is a declared abstraction boundary,
    carried by `delta_canonicality_fuzz`. (Amended 2026-10-03, after
    independent review round 1: the claim above that malformed deltas reach
    `DeviceSlotsFull` is wrong. The model's `applyDelta` keeps a device-cap
    branch, but no delta the driver builds can reach it — the crate's member
    records cannot hold more than `MAX_DEVICES` keys by type, as the model's
    own `applyDelta` doc comment in `org-members/quint/membership.qnt` records.
    The malformed deltas reach the model tags `StaleRemoval`,
    `RemoveUpsertOverlap`, `NoOpUpsert` and `ConfusableHandle` only;
    `DeviceSlotsFull` is reached through `AddDevice`.)
11. **Genesis is the membership model's `init`.** `init` nondeterministically picks a list
    of 0–3 leaves from the model universe — including duplicate identifiers,
    colliding handles and invalid device lists — and applies `genesis`, so
    every trace starts from a real ceremony (as upstream's 2PC example starts
    from an initialised state). A failed genesis leaves "no organisation"
    (`org: Option[Snapshot]`, distinct from an empty organisation, as in the
    crate) plus `lastError`; every later action then reports `IdNotFound`.
    Sample counts or weighting compensate for traces spent on failed
    genesis. Expected first finding: model `genesis` rejects empty and
    over-cap device lists itself; crate `genesis` leaves that to
    `MemberLeaf::new`. (Amended 2026-10-03, after independent review round 2:
    the model state is not `org: Option[Snapshot]`. It is `orgExists: bool`
    plus `trie: Snapshot` (`org-members/quint/membership_mbt.qnt`); a failed
    genesis sets `orgExists` false and `trie` to the empty map, and the
    distinction from an empty organisation is carried by `orgExists`.)
12. **Named scenarios (`#[quint_test]`).** A floor fixed by rule: one
    happy-path `run` per model action (genesis plus the eight mutators, and
    `ApplyDelta`'s honest, stale-base and verify-fail variants); one named
    `run` per error branch the decision-8 coverage report shows as rarely
    reached (measured over repeated runs in the plan — reached in fewer than
    half of them gets a scenario), which is what turns "not gated, would be
    flaky" into "pinned"; and every conformance failure lands with its seed's
    trace reduced to a named `run` in `membership_mbt.qnt`, in the same change
    as the fix (the scenario is `resolve-problem`'s failing annotated test).
    (Planning finding, 2026-10-03, measured with quint 0.32.0: `quint test`
    traces carry **no** `mbt::actionTaken`/`mbt::nondetPicks` on either
    backend, and `quint test` rejects `--mbt`. So the model records each
    action in its own state — a `lastAction` sum type, tag = action name,
    payload = picks — and the driver reads it through
    `Config { nondet: &["lastAction"] }`, for random runs and named scenarios
    alike. This is upstream's two-phase-commit pattern, minus Choreo.)
    (Amended 2026-10-03, after independent review round 3: the named
    scenarios are not `#[quint_test]` functions. Each scenario test calls
    `quint_connect::runner::run_test` with a `TestConfig` naming its `run`
    (`run_scenario` in `org-members/tests/mbt_conformance.rs`), as the random
    run does with a `RunConfig`; see decision 4's round-3 note and decision
    8(ii) for the macro's limitation.)
13. **org-node: Quint checks now, conformance later; two parts.**
    `protocol.qnt` models the whole distributed system (chain, devices,
    network, clock, CGKA); org-node implements one node's slice, and about
    half the system model's actions (`cgkaRotate`, `memberReceiveOrgSecret`, `tick`)
    have no implementation yet, so quint-connect against `protocol.qnt` would
    fail with "Unimplemented action" — and stub arms would make "maps 1:1" a
    false pass. So org-node gets its Quint checks (typecheck plus the
    simulator invariants with explicit `max_steps`) unconditionally, but **no
    conformance test**. A node-level model for the implemented slice
    (`org-node/quint/node_mbt.qnt`, chain and network as environment actions),
    driven by quint-connect under decisions 4, 8 and 9, is a separate
    follow-up change with its own interview. **Known gap until then: a Rust
    change to org-node is not checked for conformance.**
    Delivery is in two parts, each its own merged change: **part 1** is
    org-members only; **part 2** — every org-node change, including moving
    `protocol.qnt` and `ods_instances.qnt` into `org-node/quint/` and adding
    them to org-node's `verify_commands` — starts after part 1 is merged.
    Two boundary items belong to part 1 because part 1 cannot merge true
    without them: `quint/protocol.qnt`'s import is repointed to
    `../org-members/quint/membership_types` (forced by the move; it edits the
    disclaimed `quint/`, not `org-node/`), and the `rust-version` bumps in
    `org-node/Cargo.toml` and `app/src-tauri/Cargo.toml` land with
    org-members' — so every declared MSRV on `master` is true at every commit.

## Why

- A model outside every unit can drift from the code it describes with no gate
  noticing; inside the unit, a model change runs the same gate as a code change.
- Unconditional checks: org-members' saving would be ~6 s, and a `.qnt`-diff
  condition is new gate logic that fails open and misses Quint upgrades.
- Upstream checks only spec -> implementation completeness; at class C an
  operation the model never sees cannot be called conformant.
