# Quint conformance gate — part 1 (org-members) Implementation Plan

**Goal:** Move the membership model into `org-members/quint/`, make its checks and
a strengthened quint-connect conformance test part of org-members' merge gate, and
fix PR-vf5hdm — as decided in `docs/adr/2026-10-03-quint-conformance-gate.md`.
**Implements:** REQ-wx3wpv, LLR-n5t6bn; resolves PR-vf5hdm. New carriers, each
annotated only after the measurement in T8: LLR-ch2pkw, LLR-4n8zqx, LLR-au8het,
LLR-7tdqv9, LLR-xmpqn2, LLR-juxk9q.
**Safety class:** C (`org-members/.guardrails/config.yaml`; no per-item overrides).
**Verification:** org-members' `verify_commands` as rewritten in T2:

    cargo test -p org-members
    quint --version
    quint typecheck org-members/quint/membership_types.qnt
    quint typecheck org-members/quint/membership.qnt
    quint typecheck org-members/quint/membership_mbt.qnt
    quint test org-members/quint/membership.qnt --backend=typescript
    quint run org-members/quint/membership_mbt.qnt --invariant=mbtInv --max-steps=15 --max-samples=1000
    quint typecheck quint/protocol.qnt

plus, through `check-units.sh --impact`, org-node's and app's gates (org-node's
`quint typecheck quint/protocol.qnt` must stay green after T1's import repoint).

**Out of scope (part 2, after this merges):** moving `protocol.qnt` and
`ods_instances.qnt` into `org-node/quint/`, org-node's `verify_commands` and
`max_steps`, and the ADR's known gap (no org-node conformance test). `quint/`
stays in `not_a_unit` until part 2.

## Evidence this plan rests on

Every code block in the appendices was compiled and run on 2026-10-03 in a
scratch copy of the crate (quint 0.32.0, rustc stable). Measured there:

- The model typechecks; 21 `quint test` unit tests and 20 named scenarios pass;
  `mbtInv` holds at 200 samples × 15 steps (rust backend 0.65 s; typescript 32 s).
- Before the T7 fix, `scenario_handle_swap` and `scenario_handover_a_to_b` fail
  with "Specification and implementation states diverge";
  `scenario_handover_b_to_a` passes (the safe identifier order). The 100-sample
  random run did **not** catch PR-vf5hdm on its own — the named scenarios did.
- After the fix: org-members 6 + 108 + 23 tests pass (1 ignored, the preflight
  probe), 5 consecutive full runs green at ~9.6 s; the random run alone passed
  5 of 5 times in ~7 s, reaching every `ApplyDelta` outcome every time.
- Each model-order red in T4/T5 was produced by its scenario against the old
  model and cleared by the reorder (genesis records-first, device cap first,
  no-op before overlap).
- `quint test` traces carry no `mbt::actionTaken` (both backends) — hence the
  tracked `lastAction` (ADR decision 12).

## Sandbox note (local runs only)

`~/.cargo` and `$HOME` are read-only in the dev sandbox, and the volta `quint`
shim cannot write its temp dir. Run every command through a wrapper that sets
`HOME` to a writable dir, `CARGO_HOME=/tmp/cargo_home_fuzz`, and puts a `quint`
that execs node on quint's `cli.js` first on `PATH` (the session scratchpad's
`env.sh` and `bin/quint` do exactly this). CI uses the defaults.

---

### T1 — Move the membership model into the unit and split the exported interface

**Status: done** (merged from `worktree-quint-connect-coupling-t1` @ eda4922).
- red -> green: `membership_conformance` (`cargo test -p org-members --test mbt_conformance`) — after the git mv it failed with "file ../quint/membership_mbt.qnt does not exist"; green after the spec path became `quint/membership_mbt.qnt`.
- result: typecheck OK on membership_types, membership, membership_mbt, protocol, ods_instances; `quint test membership.qnt` 21 passing; `cargo test -p org-members` 6 + 108 + 1 passed.
- note: `.github/workflows/quint.yml` and `org-members/.guardrails/config.yaml` still name `quint/membership*.qnt` until T2. The Apalache paragraph's stale path in `quint/README.md` was also updated.

**Files touched:**
- `quint/membership.qnt` → `org-members/quint/membership.qnt` (git mv, then edit)
- `quint/membership_mbt.qnt` → `org-members/quint/membership_mbt.qnt` (git mv only)
- `org-members/quint/membership_types.qnt` (new)
- `org-members/quint/.gitignore` (new)
- `org-members/quint/README.md` (new)
- `quint/protocol.qnt`
- `quint/README.md`
- `org-members/tests/mbt_conformance.rs`

**Parallel:** no (first).
**Trace:** none new — a move with no behaviour change; its evidence is that the
existing conformance test and the Quint checks stay green.

1. `git mv quint/membership.qnt org-members/quint/membership.qnt` and
   `git mv quint/membership_mbt.qnt org-members/quint/membership_mbt.qnt`
   (one git command per call).
2. **Red.** `cargo test -p org-members --test mbt_conformance`.
   Expected: FAIL — quint cannot open `../quint/membership_mbt.qnt`.
3. In `org-members/tests/mbt_conformance.rs` change the attribute to
   `#[quint_run(spec = "quint/membership_mbt.qnt", max_samples = 50)]`
   (cargo runs integration tests with the package root as cwd).
4. Create `org-members/quint/membership_types.qnt` with **Appendix A** verbatim.
5. In `org-members/quint/membership.qnt` replace everything from
   `module membership {` up to (not including) `/// Crate constant: members cap at 4 devices.`
   with:

   ```
   module membership {
     /// The boundary types live in the exported model interface.
     import membership_types.* from "./membership_types"
     export membership_types.*

   ```
   (The `export` is what lets `membership_mbt.qnt`, which imports only
   `membership.*`, keep seeing `Key`/`Leaf`/`Snapshot`/`Result`/`Delta`.)
6. In `quint/protocol.qnt`: line 11 becomes
   `  import membership_types.* from "../org-members/quint/membership_types"`,
   and lines 4–5 of the header become:
   ```
   /// side-table, so Apalache `quint verify` is tractable at depth. The only
   /// thing taken from org-members' model is its exported interface
   /// (`membership_types`: the `Key` record); the trie semantics live in
   /// org-members/quint/membership.qnt (simulator + conformance test).
   ```
7. Create `org-members/quint/.gitignore` with the same seven lines as
   `quint/.gitignore`.
8. Move the membership-only sections of `quint/README.md` — "Modules" (the two
   membership bullets), the membership commands, "Sandbox cargo recipe",
   "Caveats (by design)" — into a new `org-members/quint/README.md` titled
   `# org-members — membership model`, with every path rewritten to
   `org-members/quint/…` and the conformance command written as
   `cargo test -p org-members --test mbt_conformance`. Leave in `quint/README.md`
   a one-line pointer: `The membership model moved to org-members/quint/ on
   2026-10-03 (docs/adr/2026-10-03-quint-conformance-gate.md).` and the protocol
   sections unchanged except the scope disclaimer's `membership.qnt` path.
9. **Green.**
   ```
   quint typecheck org-members/quint/membership_types.qnt
   quint typecheck org-members/quint/membership.qnt
   quint typecheck org-members/quint/membership_mbt.qnt
   quint typecheck quint/protocol.qnt
   quint typecheck quint/ods_instances.qnt
   quint test org-members/quint/membership.qnt --backend=typescript   # 21 passing
   cargo test -p org-members                                           # 6 + 108 + 1 passed
   ```
10. Commit: `refactor(quint): membership model into org-members/quint; exported membership_types`.

### T2 — Wire the Quint checks into the gate and CI

**Files touched:**
- `org-members/.guardrails/config.yaml`
- `.github/workflows/quint.yml`
- `.github/workflows/rust.yml`
- `.guardrails/units.yaml`

**Parallel:** no (serial, after T1).
**Trace:** none — gate configuration (ADR decisions 3, 7, 9).

**Status: done** (merged from `worktree-quint-connect-coupling-t2` @ fb1eee8).
- red -> green: n/a — no test added (no trace); evidence is the eight verify_commands passing on the new config.
- result: all eight verify_commands exit 0; `cargo test -p org-members` 6 + 108 + 1 passed; three membership typechecks OK; `quint test membership.qnt` 21 passing; `mbtInv` run (15 steps, 1000 samples, rust backend) OK; protocol.qnt typecheck OK; `check-units.sh` exit 0.
- note: local `quint --version` is 0.33.0 (not the 0.32.0 in step 5); Quint is unpinned by decision 7, so the installed version is what gets recorded. Old paths remain only in historical `docs/verification/*.md` records, which were left alone; no mutation script quotes a changed line.

1. In `org-members/.guardrails/config.yaml` replace the `verify_commands:` list
   with the eight commands in the plan header, and replace its comment with:
   ```
   # Commands merge-change and verify-before-merge must run and pass
   # (docs/adr/2026-10-03-quint-conformance-gate.md). `cargo test -p
   # org-members` includes mbt_conformance, which runs quint through
   # quint-connect and FAILS, naming the cause, when quint is absent or
   # ~/.quint is unwritable (decision 9). `quint --version` puts the Quint
   # version into every verification record (decision 7: not pinned).
   # typecheck/test use the typescript backend; the simulator run uses the
   # default rust backend (decision 9, amended). protocol.qnt imports this
   # unit's exported model interface and stays typechecked here until part 2
   # moves it into org-node.
   ```
2. In `.github/workflows/quint.yml`: every `quint/membership.qnt` /
   `quint/membership_mbt.qnt` path becomes `org-members/quint/…`; add
   `- run: quint typecheck org-members/quint/membership_types.qnt` before them;
   change the `mbtInv` run to `--max-steps=15 --max-samples=1000`; **delete the
   whole `mbt:` job** (it duplicates `rust.yml`'s `test` job).
3. In `.github/workflows/rust.yml`, replace the last paragraph of the comment
   above `test:` ("This does duplicate quint.yml's `mbt` job … does not own
   quint.yml.") with:
   `# quint.yml's duplicate \`mbt\` job was removed on 2026-10-03; this job is`
   `# the only CI run of the conformance test.`
   and correct "despite what its own header comment says" — the header is
   corrected in T6 — to "as its header comment states".
4. In `.guardrails/units.yaml`, the `quint` entry's comment becomes
   `# protocol models (org-node's), moving into org-node in part 2 of docs/adr/2026-10-03-quint-conformance-gate.md; membership model now in org-members/quint`.
5. **Verify.** Run all eight `verify_commands` (expected: all exit 0; `quint --version`
   prints `0.32.0` locally) and `.guardrails/scripts/check-units.sh` (expected:
   exit 0 for the manifest).
6. Commit: `ci(quint): membership checks in org-members' gate; drop duplicate mbt job`.

### T3 — MSRV 1.85, measured

**Files touched:**
- `Cargo.toml`
- `org-members/Cargo.toml`
- `org-node/Cargo.toml`
- `app/src-tauri/Cargo.toml`
- `.github/workflows/rust.yml`

**Parallel:** no (serial, after T2 — `rust.yml`).
**Trace:** none — ADR decision 6.

**Status: done** (merged from `worktree-quint-connect-coupling-t3` @ 989fbf1).
- red -> green: n/a — no test added (no trace). The 1.85 build was NOT measured locally (rustup has no 1.85); the CI `msrv` job run is the evidence and verify-before-merge must cite it.
- result: `cargo metadata --no-deps` shows 5 × rust_version 1.85 (org-members, org-node, three spike crates via workspace inheritance); `app/src-tauri/Cargo.toml:7` = 1.85; rust.yml jobs: test, msrv, app-frontend, clippy, no-std, coverage, guardrails; on stable 1.99 `cargo test -p org-members` 6 + 108 + 1 passed.
- note: installed quint (0.33.0) now fetches rust-evaluator v0.7.0 into `~/.quint`; with a read-only `$HOME` the conformance test fails with EPERM (the Sandbox note's known issue), green through the scratchpad env.sh wrapper.

1. Set `rust-version = "1.85"` in `[workspace.package]` of `Cargo.toml` and in
   the `[package]` of the other three manifests.
2. Add to `.github/workflows/rust.yml`, after the `test:` job:
   ```yaml
     # The declared MSRV, measured (docs/adr/2026-10-03-quint-conformance-gate.md,
     # decision 6): quint-connect is edition 2024, so the test build needs 1.85.
     msrv:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: actions/setup-node@v4
           with: { node-version: "20" }
         - run: npm i -g @informalsystems/quint
         - uses: dtolnay/rust-toolchain@1.85
         - run: cargo test -p org-members
   ```
3. **Verify.** `cargo metadata --format-version 1 --no-deps | grep -c '"rust_version":"1.85"'`
   (expected: ≥ 3 for this workspace; app is checked by
   `grep -n 'rust-version = "1.85"' app/src-tauri/Cargo.toml`). The 1.85 build
   itself cannot run in the dev sandbox (no 1.85 toolchain installed, rustup
   home read-only): **the `msrv` CI job is the evidence**, and verify-before-merge
   must cite its run.
4. Commit: `build: MSRV 1.85 (quint-connect is edition 2024), with a CI job that measures it`.

### T4 — Model: tracked actions, genesis as `init`, full device range

**Files touched:**
- `org-members/quint/membership.qnt`
- `org-members/quint/membership_mbt.qnt`
- `org-members/tests/mbt_conformance.rs`

**Parallel:** no (serial, after T3).
**Trace:** carriers for LLR-ch2pkw (measured in T8).

**Status: done** (merged from `worktree-quint-connect-coupling-t4` @ 5ef6f3b).
- red -> green: `scenario_genesis_device_error_first` — failed before the model fix, spec "DuplicateId" vs driver "EmptyDeviceList"; green after the `genesis` change.
- red -> green: `scenario_device_slots_full` — failed before the model fix, spec "DuplicateDevice" vs driver "DeviceSlotsFull" at step 4; green after the `addDevice` reorder.
- red -> green: `scenario_genesis` — n/a, happy-path replay, green on the old model too (expected by the plan).
- red -> green: `membership_conformance` — rewritten onto `run_conformance(MembershipDriver::default(), …)`; green (verifies: LLR-fv75ec, LLR-j4d38d, LLR-v3jqau).
- result: `cargo test -p org-members` 0 + 6 + 108 + 4 passed, no warnings; `quint test` 21 passing; mbtInv 1000 × 15 OK; three membership typechecks and protocol.qnt clean. quint 0.33.0.
- note: quint-connect reads `QUINT_VERBOSE` at compile time. The `Leaf` doc comment and the module doc's `TRIE_OPERATIONS` reference describe the final file and become true in T5/T6 (the deslop pass checks them).

1. Replace `org-members/quint/membership_mbt.qnt` with **Appendix C**, leaving
   out: the `ApplyDelta` variant of `Action`; `NO_OPS`, `NO_IDS`, `NO_LEAVES`,
   `applyOp`, `applyOps`, `applyAndVerify`, `applyHonest`, `applyStale`,
   `applyBuilt`, `OPS`, `ApplyDeltaStep` and its three `step` entries; and every
   `run scenario…` except `scenarioGenesis`, `scenarioGenesisDeviceErrorFirst`
   and `scenarioDeviceSlotsFull` (keep `TWO` and `hop`).
2. Replace `org-members/tests/mbt_conformance.rs` with **Appendix D**, leaving
   out: the `Op` struct, `apply_op`, `real_leaf`, `apply_delta_step`, the
   `ApplyDelta` arm, `ACTIONS`, `TRIE_OPERATIONS`, `BOUNDARY`, `Coverage`,
   `with_coverage` and the coverage block in `step`, and the tests
   `trie_operations_table_matches_source`, `quint_preflight_names_a_missing_binary`,
   `preflight_probe`; `membership_conformance` becomes
   `run_conformance(MembershipDriver::default(), "membership_conformance")`
   under the existing `/// verifies: LLR-fv75ec, LLR-j4d38d, LLR-v3jqau` block.
   Keep only the three `scenario_*` functions matching step 1.
3. **Red** (measured 2026-10-03). With `membership.qnt` still holding the old
   `genesis` and `addDevice`, run
   `cargo test -p org-members --test mbt_conformance scenario_`.
   Expected: `scenario_genesis_device_error_first` FAILED (spec `"DuplicateId"`,
   driver `"EmptyDeviceList"`: every record is built before `genesis` sees it)
   and `scenario_device_slots_full` FAILED (spec `"DuplicateDevice"`, driver
   `"DeviceSlotsFull"`: `P2pDeviceSlots::add_device` checks the cap first,
   `types.rs:387`; reachable only now that `GENS = 0.to(3)`).
4. In `membership.qnt` replace `genesis` with Appendix B's `leafDeviceError` +
   `genesis`, and `addDevice` with Appendix B's (cap before duplicate).
5. **Green.** `quint test org-members/quint/membership.qnt --backend=typescript`
   (21 passing); `quint run org-members/quint/membership_mbt.qnt --invariant=mbtInv --max-steps=15 --max-samples=1000`
   (`[ok] No violation found`); `cargo test -p org-members --test mbt_conformance`
   (4 passed).
6. Commit: `test(mbt): tracked lastAction, genesis as init, four key generations`.

### T5 — Model: the `ApplyDelta` action and one-to-one error tags

**Files touched:**
- `org-members/quint/membership.qnt`
- `org-members/quint/membership_mbt.qnt`
- `org-members/tests/mbt_conformance.rs`

**Parallel:** no (serial, after T4).
**Trace:** carriers for LLR-au8het, LLR-7tdqv9, LLR-xmpqn2, LLR-juxk9q (measured in T8).

1. Add to `membership_mbt.qnt` the definitions T4 left out, and the scenarios
   `scenarioApplyHonest`, `scenarioApplyStale`, `scenarioApplyVerifyFail`,
   `scenarioRemoveUpsertOverlap`, `scenarioNoOpBeforeOverlap`,
   `scenarioBuiltVerifyFail`. Add to `mbt_conformance.rs` the `Op` struct,
   `apply_op`, `real_leaf`, `apply_delta_step`, the `ApplyDelta` arm, and the six
   matching `scenario_*` functions (Appendix D's `err_tag` is already in place).
2. **Red** (measured). With `applyDelta` in `membership.qnt` unchanged, run
   `cargo test -p org-members --test mbt_conformance scenario_no_op_before_overlap`.
   Expected: FAILED — spec `"RemoveUpsertOverlap"`, driver `"NoOpUpsert"` (the
   crate's `validate_canonical_delta` tests no-op upserts before overlap).
3. Replace `applyDelta` with Appendix B's (no-op before overlap; doc comment
   naming LLR-n5t6bn).
4. Run the whole target. Expected: PASS, **or** a `membership_conformance`
   FAIL whose only divergence is PR-vf5hdm's signature (`ApplyDelta`, kind
   `honest`, two `handle` ops; spec `""`, driver `"ConfusableHandle"`). The
   random run does not reliably reach it (measured: it did not in the scratch
   run), and T7 owns it. Any other divergence is a new finding: stop and run
   `resolve-problem`.
5. Commit: `test(mbt): ApplyDelta action — honest, stale and built deltas, verify_against`.

**Status: done** (merged from `worktree-quint-connect-coupling-t5` @ d3c0605).
- red -> green: `scenario_no_op_before_overlap` — failed against the old `applyDelta`, spec "RemoveUpsertOverlap" vs driver "NoOpUpsert" (seed 0xd88bb2b3); green after the Appendix B reorder.
- red -> green: `scenario_apply_honest`, `scenario_apply_stale`, `scenario_apply_verify_fail`, `scenario_remove_upsert_overlap`, `scenario_built_verify_fail` — n/a, replay paths the old model already matched; green before and after.
- result: typechecks clean; `quint test` 21 passing; mbtInv 1000 × 15 OK; `cargo test -p org-members` 0 + 6 + 108 + 10 passed, no warnings.
- finding: the random `membership_conformance` now hits PR-vf5hdm in ~15% of runs (1/5, then 3/20), always ApplyDelta "honest", spec "" vs driver "ConfusableHandle". Two of three captured seeds are a **handle + add** handover (seed 0xa68bbc85: [handle c->h3, add b h1]); 0x7dd619a6 is the two-handle form. All three go green with the Appendix E fix applied temporarily — same root cause.

**Order amended (2026-10-03, dispatcher):** T7 runs before T6. Both touch the same two test files, so they stay serial; running T7 first removes the PR-vf5hdm flake that would otherwise turn T6's 20-run tally and 10-run stability step red. T7 additionally gets a handle+add handover scenario (`scenarioHandoverToNewMember`, seed 0xa68bbc85's shape).

### T6 — Enforcing the 1:1 rule, the preflight test, remaining scenarios

**Files touched:**
- `org-members/quint/membership_mbt.qnt`
- `org-members/tests/mbt_conformance.rs`

**Parallel:** no (serial, after T5).
**Trace:** none of its own (gate tests, ADR decisions 8, 9, 12).

**Status: done, ran after T7** (merged from `worktree-quint-connect-coupling-t6` @ d69499d).
- red -> green: `trie_operations_table_matches_source` — failed with the `verify_against` row deleted ("trie-yielding public operations and the TRIE_OPERATIONS/BOUNDARY table disagree", `verify_against` left only); green restored.
- red -> green: `membership_conformance` (coverage gate) — failed with the three `ApplyDeltaStep` entries commented out ("model actions never taken in the random run: [\"ApplyDelta\"]"); green restored.
- red -> green: `quint_preflight_names_a_missing_binary` — failed with the assertion changed to "PREFLIGHT: ok", child output containing "PREFLIGHT: `quint` is not runnable from PATH (Permission denied (os error 13))"; green restored.
- red -> green: 8 new `scenario_*` (add_member … isolate) — n/a, happy-path replays green on first run (as planned).
- result: `cargo test -p org-members` 0 + 6 + 108 + (24 passed, 1 ignored = `preflight_probe`, subprocess-only by design); typechecks OK; `quint test` 21 passing; mbtInv 1000 × 15 OK; clippy on the test target clean.
- step 6: 20/20 green; all 33 observed (action, outcome) pairs present in 20/20 runs; AddDevice→DeviceSlotsFull 0/20 in random runs, covered by `scenario_device_slots_full`. Table: `docs/verification/notes-quint-conformance-part1-coverage.md`.
- step 7: 10/10 green (24 passed, 1 ignored each).
- note: `cargo clippy --tests` fails pre-existing at `org-members/tests/fuzz_tests.rs:420` (untouched file); `mbt_conformance.rs` not rustfmt-clean (pre-existing) — for the deslop pass.

1. Add Appendix D's `ACTIONS`, `TRIE_OPERATIONS`, `BOUNDARY`, `Coverage`,
   `with_coverage`, the coverage block in `step`, the coverage version of
   `membership_conformance`, and the tests `trie_operations_table_matches_source`,
   `quint_preflight_names_a_missing_binary`, `preflight_probe`.
2. **Red — table.** Temporarily delete the `("verify_against", "ApplyDelta")` row
   (array length 10). Run `cargo test -p org-members --test mbt_conformance trie_operations`.
   Expected: FAIL, "trie-yielding public operations and the TRIE_OPERATIONS/BOUNDARY
   table disagree", `verify_against` only on the left. Restore.
3. **Red — coverage.** Temporarily comment out the three `ApplyDeltaStep`
   entries of the model's `step`. Run `membership_conformance`. Expected: FAIL,
   `model actions never taken in the random run: ["ApplyDelta"]`. Restore.
4. **Red — preflight.** Temporarily change the asserted text in
   `quint_preflight_names_a_missing_binary` to `"PREFLIGHT: ok"`; expected FAIL
   whose printed output contains ``PREFLIGHT: `quint` is not runnable from PATH``.
   Restore.
5. Add the remaining non-PR scenarios of Appendix C and their functions
   (`scenarioAddMember` … `scenarioIsolate`), making 17 of Appendix C's 20.
6. **Rare-branch measurement (decision 12).** Run
   `cargo test -p org-members --test mbt_conformance membership_conformance -- --nocapture`
   20 times and tally the `coverage <action> -> <outcome>` lines. Any
   (action, outcome) pair present in fewer than 10 of the 20 runs gets a named
   scenario now, in the same style. Record the 20-run table in the change's
   verification notes. (Scratch, 5 runs: every `ApplyDelta` outcome present in
   all 5; lowest counts `RemoveUpsertOverlap` 6–11 and `ConfusableHandle` 4–9 per
   run.)
7. **Stability.** Run the whole target 10 times in a row; expected 10 × green
   (scratch: `20 passed; 1 ignored` before T7). Background: one spurious failure
   of a scenario test was seen in the scratch run under parallel execution, with
   the log lost; the identified cause was the preflight's write probe, then a
   single shared file name that parallel tests deleted under each other. Appendix
   D's per-process-and-thread probe name is the fix; 5 consecutive full runs
   were green after it. A red here is a finding, not noise to rerun past.
8. Commit: `test(mbt): operation table, action coverage, fail-closed preflight, named scenarios`.

### T7 — Resolve PR-vf5hdm (resolve-problem)

**Files touched:**
- `org-members/quint/membership_mbt.qnt`
- `org-members/tests/mbt_conformance.rs`
- `org-members/src/trie.rs`
- `org-members/docs/problems/2026-09-17-review2-fixes.md`

**Parallel:** no (serial, after T6).
**Trace:** REQ-wx3wpv, LLR-n5t6bn; resolves PR-vf5hdm.

**Status: done, ran before T6** (merged from `worktree-quint-connect-coupling-t7` @ 51062f7).
- red -> green: `scenario_handover_a_to_b` — failed before the fix, "Specification and implementation states diverge", spec lastError "" vs driver "ConfusableHandle"; green after Appendix E.
- red -> green: `scenario_handle_swap` — failed before the fix, same divergence; green after.
- red -> green: `scenario_handover_to_new_member` (added by the amended order; genesis [a:h2, c:h1], honest delta [handle c->h3, add b h1]) — failed before the fix, same divergence; green after.
- red -> green: `scenario_handover_b_to_a` — n/a, the safe identifier order; green before and after, as predicted.
- result: `cargo test -p org-members` 0 + 6 + 108 + 14 passed (T6's tests not yet in); `cargo clippy -p org-members --lib -- -D warnings` clean; membership_mbt typecheck OK; `membership_conformance` 20/20 green after the fix.
- note: PR-vf5hdm `status: resolved` with a dated resolution note recording the third (handover-to-admitted-member) shape.

Run under the `resolve-problem` skill; this task is its failing-test-first cycle.

1. Add the three PR-vf5hdm `run` lines and their comment block from Appendix C,
   and the three functions from Appendix D, each annotated:
   ```rust
   /// verifies: REQ-wx3wpv, LLR-n5t6bn
   ///
   /// PR-vf5hdm: a handle moved between two members who are both still present,
   /// in one honest delta produced by `recalculate()`.
   #[test]
   fn scenario_handover_a_to_b() {
       run_scenario("scenarioHandoverAtoB")
   }
   ```
   (same annotation on `scenario_handover_b_to_a` and `scenario_handle_swap`).
2. **Red.** `cargo test -p org-members --test mbt_conformance scenario_hand`.
   Expected (measured): `scenario_handle_swap` FAILED and
   `scenario_handover_a_to_b` FAILED with "Specification and implementation
   states diverge" (driver `lastError: "ConfusableHandle"` — the crate's
   `DuplicateHandle`); `scenario_handover_b_to_a` ok.
3. **Fix.** In `OrgTrie::apply_delta` (`org-members/src/trie.rs`), apply the
   **Appendix E** hunk: a first loop over `delta.upserted` releases every
   outgoing handle from both indexes; the existing loop loses its release block.
4. **Green.** `cargo test -p org-members` — expected 6 + 108 + (23 passed,
   1 ignored). Then `cargo clippy -p org-members --lib -- -D warnings` (clean).
5. In `org-members/docs/problems/2026-09-17-review2-fixes.md`, set PR-vf5hdm's
   `status:` per `resolve-problem` (resolved by this change, naming
   REQ-wx3wpv, LLR-n5t6bn and the three scenario tests), and append a dated
   note: the owner chose not to forbid handovers (REQ-wx3wpv's draft file);
   the fix is the two-phase loop; the hazard is HAZ-y8h835 as amended.
6. Commit: `fix(org-members): apply_delta releases outgoing handles first (PR-vf5hdm)`.

### T8 — Measure the carriers, then annotate

**Files touched:**
- `org-members/tests/mbt_conformance.rs`
- `org-members/docs/risk/DRAFT-worktree-quint-connect-coupling-lawful-change-replicates.md`

**Parallel:** no (serial, after T7).
**Trace:** LLR-ch2pkw, LLR-4n8zqx, LLR-au8het, LLR-7tdqv9, LLR-xmpqn2, LLR-juxk9q.

**Status: done** (merged from `worktree-quint-connect-coupling-t8` @ 59f117b). Mutations of `src/` each reverted (empty diff after every row); "5/5" = five random-seed runs. Full table in the risk DRAFT's "Measured carriers" section.
- red -> green: LLR-ch2pkw → `membership_conformance` red 5/5 with genesis DuplicateId deleted, and with both handle-collision returns deleted. Measured negative: ConfusableHandle return alone deleted → green 5/5 (model has skeleton == handle; confusable half outside the model).
- red -> green: LLR-4n8zqx → `membership_conformance` red 5/5 under an order-dependent leaf hash (2 divergence, 3 driver round-trip panic).
- red -> green: LLR-au8het → `scenario_apply_stale` red (DeltaBaseMismatch return deleted).
- red -> green: LLR-7tdqv9 → `scenario_apply_verify_fail`, `scenario_built_verify_fail` red (verify_against always Ok).
- red -> green: LLR-xmpqn2 → overlap check deleted: `scenario_remove_upsert_overlap` + `membership_conformance` red; no-op check deleted: `scenario_no_op_before_overlap` + `membership_conformance` red; stale-removal check deleted: `membership_conformance` red via the apply loop's own guard ("Other:InvariantViolated" vs "StaleRemoval"). Ordering clauses: declared boundary (delta_canonicality_fuzz), not mutated.
- red -> green: LLR-juxk9q → `membership_conformance` red 5/5 with the handle-check block deleted and with only DuplicateHandle deleted (uniqueness half now carried).
- red -> green: LLR-n5t6bn → reverting T7's first loop reds the three handover scenarios; `scenario_handover_b_to_a` green, as predicted.
- result: mbt_conformance 24 passed, 1 ignored; org-members check-trace: no MISSING-TEST; rc=1 only from pre-existing STALE-PROBLEM PR-zz4exm (separate change).
- note: `scenario_no_op_before_overlap` annotated LLR-xmpqn2 (not in the plan's column, measured red). `membership_conformance` also reds under the LLR-au8het/LLR-7tdqv9 mutations; not annotated for them (plan column kept).

The project annotates a carrier only after a mutation of the claimed clause
reds it (risk file `2026-09-17-design-derived.md`, "Measured negatives"). For each
row: apply the mutation to `src/`, run the named target, record red/green,
revert (`git diff src/` empty before the next row).

| LLR | Mutation in `src/` | Run | Annotate on red |
|---|---|---|---|
| LLR-ch2pkw | delete `genesis`'s duplicate-identifier `return Err(DuplicateId)` | `membership_conformance` ×5 | `membership_conformance` |
| LLR-ch2pkw | delete `genesis`'s skeleton/handle collision returns | `membership_conformance` ×5 | (same) |
| LLR-4n8zqx | in `smt::insert`, mix the insertion count into the leaf hash (any order-dependent change) | `membership_conformance` ×5 | `membership_conformance` |
| LLR-au8het | delete the `base_root != current_root` check in `apply_delta` | `scenario_apply_stale` | `scenario_apply_stale` |
| LLR-7tdqv9 | make `verify_against` return `Ok` unconditionally | `scenario_apply_verify_fail`, `scenario_built_verify_fail` | both |
| LLR-xmpqn2 | delete each of the three in-model canonical checks (stale removal, no-op, overlap) in turn | `membership_conformance` ×5, `scenario_remove_upsert_overlap` | carriers that red; record that the two ordering clauses are on the declared boundary (`delta_canonicality_fuzz`) |
| LLR-juxk9q | delete the handle-check block in `apply_delta`'s second loop | `membership_conformance` ×5 | `membership_conformance` — the uniqueness half the 2026-09-17 risk file found uncarried |
| LLR-n5t6bn | revert T7's first loop | the three handover/swap scenarios | already annotated in T7; record the measurement |

1. Run every row; "×5" means five runs (random seeds) and counts as red if any
   run fails with a model/implementation divergence (not a harness `expect`
   crash — record those separately, as the 2026-09-17 file does).
2. Add `verifies:` IDs only for rows that went red, to the doc comments of the
   tests named in the last column (keep `membership_conformance`'s existing
   `LLR-fv75ec, LLR-j4d38d, LLR-v3jqau`; replace its "Not LLR-ch2pkw" paragraph
   with the measured result).
3. Write the table with outcomes as a "Measured carriers" section in the risk
   DRAFT file named above. A row that stayed green is written down as a
   measured negative, not annotated.
4. **Verify.** `GR_CONFIG=org-members/.guardrails/config.yaml .guardrails/scripts/check-trace.sh` —
   expected: no `MISSING-TEST` for REQ-wx3wpv or LLR-n5t6bn; the only remaining
   finding is the pre-existing `STALE-PROBLEM PR-zz4exm` (see below).
5. Commit: `test(mbt): measured carriers annotated`.

---

## Known blocker outside this plan

Two unit gates this change must pass are already red on `master`, for reasons
unrelated to it (measured 2026-10-03):

- org-members: `STALE-PROBLEM PR-zz4exm (open 33 days, limit 30)`;
- org-node (run through `--impact`): `STALE-PROBLEM PR-hvg2dy (open 31 days, limit 30)`.

`merge-change` cannot pass while either stands. The owner decides before
merge — resolve each in its own change first, or another disposition. This
plan does not touch them. (app's gate reports only an `UNMET-EXPECTATION`
within its age limit.)

## Self-review

1. REQ-wx3wpv and LLR-n5t6bn are verified by T7's three scenario tests
   (measured red before the fix, green after). PR-vf5hdm is resolved in T7.
   The six existing LLRs are annotated only where T8 measures red.
2. Every code step points to an appendix compiled and run on 2026-10-03, or
   shows its code inline; every command lists its expected outcome.
3. Names are consistent across tasks: `lastAction`, `orgExists`, `prev`,
   `initWith`, `applyHonest`/`applyStale`/`applyBuilt`, `run_conformance`,
   `run_scenario`, `TRIE_OPERATIONS`, `BOUNDARY`, `ACTIONS`, `quint_preflight`.
4. Every task states **Files touched:** and **Parallel:**; all tasks are serial.
   T4–T8 share the model and test files, and T2/T3 share `rust.yml`.

---

# Appendices

Copied byte-for-byte from the scratch crate that compiled and passed on 2026-10-03.

## Appendix A — `org-members/quint/membership_types.qnt` (new; T1)

````
// -*- mode: Bluespec; -*-
/// Exported model interface of the org-members membership model: the types
/// that describe membership data at the unit boundary. Another unit's model
/// (org-node's protocol model) imports this module and nothing else from
/// org-members. See docs/adr/2026-10-03-quint-conformance-gate.md, decision 2.
module membership_types {
  /// A peer-to-peer key, modeled as (who minted it, rotation generation).
  /// No bytes, no crypto: rotation bumps `gen`. This lets later layers
  /// express "revoked insider still holds an old generation".
  type Key = { owner: str, gen: int }

  /// A member leaf. `skeleton` is the UTS#39 confusable-skeleton of the
  /// handle: two handles collide iff their skeletons are equal.
  type Leaf = {
    id: str,
    handle: str,
    skeleton: str,
    name: str,
    surname: str,
    pKey: Key,
    devices: Set[Key],
  }

  /// The trie. Doubles as the RootHash: equal maps == equal root.
  type Snapshot = str -> Leaf

  /// Result of a mutation. `Err`'s payload is an error tag matching the
  /// crate's `OrgMembersError` variant names where one applies.
  type Result =
    | Ok(Snapshot)
    | Err(str)

  type Delta = {
    baseRoot: Snapshot,
    removed: Set[str],
    upserted: Set[Leaf],
  }
}
````

## Appendix B — `org-members/quint/membership.qnt` — final (T1 header; T4 `genesis`, `addDevice`; T5 `applyDelta`)

````
// -*- mode: Bluespec; -*-
/// Pure membership-trie semantics for the ODS Phase 1 Quint model.
/// Mirrors the `org-members` crate under the snapshot-as-root abstraction:
/// a RootHash is modeled as the canonical member map itself.
module membership {
  /// The boundary types live in the exported model interface.
  import membership_types.* from "./membership_types"
  export membership_types.*

  /// Crate constant: members cap at 4 devices.
  pure val MAX_DEVICES = 4

  /// True iff `skel` collides with any existing member's skeleton.
  pure def skeletonTaken(s: Snapshot, skel: str): bool =
    s.keys().exists(k => s.get(k).skeleton == skel)

  /// Insert a brand-new member. Mirrors OrgTrie::add_member.
  pure def addMember(s: Snapshot, l: Leaf): Result =
    if (s.keys().contains(l.id))
      Err("DuplicateId")
    else if (skeletonTaken(s, l.skeleton))
      Err("ConfusableHandle")
    else if (l.devices.size() == 0)
      Err("EmptyDeviceList")
    else if (l.devices.size() > MAX_DEVICES)
      Err("DeviceSlotsFull")
    else
      Ok(s.put(l.id, l))

  /// The device-list error a member record is refused with when it is built,
  /// or "" when its device list is acceptable. Mirrors `MemberLeaf::new`
  /// (empty list) and `P2pDeviceSlots::new` (over the cap).
  pure def leafDeviceError(l: Leaf): str =
    if (l.devices.size() == 0) "EmptyDeviceList"
    else if (l.devices.size() > MAX_DEVICES) "DeviceSlotsFull"
    else ""

  /// Build a trie from a list of leaves (genesis ceremony). Every record is
  /// built before `OrgTrie::genesis` sees any of them, so the first record in
  /// list order with a bad device list wins over every identifier or handle
  /// collision; only then does the fold over addMember run, and its first
  /// error short-circuits.
  pure def genesis(leaves: List[Leaf]): Result =
    val deviceErr = leaves.foldl("", (acc, l) => if (acc != "") acc else leafDeviceError(l))
    if (deviceErr != "")
      Err(deviceErr)
    else
      leaves.foldl(Ok(Map()), (acc, l) =>
        match acc {
          | Err(e) => Err(e)
          | Ok(s)  => addMember(s, l)
        })

  // ---- helpers for tests ----
  pure def leaf(idArg: str, handleArg: str, skel: str): Leaf = {
    id: idArg, handle: handleArg, skeleton: skel,
    name: "n", surname: "s",
    pKey: { owner: idArg, gen: 0 },
    devices: Set({ owner: idArg, gen: 0 }),
  }

  pure val emptyTrie: Snapshot = Map()

  run addMemberOkTest = {
    val r = addMember(emptyTrie, leaf("a", "alice", "alice"))
    assert(r == Ok(Map("a" -> leaf("a", "alice", "alice"))))
  }

  run addMemberDuplicateIdTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    assert(addMember(base, leaf("a", "alice2", "alice2")) == Err("DuplicateId"))
  }

  run addMemberConfusableTest = {
    val base = Map("a" -> leaf("a", "alice", "skel"))
    assert(addMember(base, leaf("b", "bob", "skel")) == Err("ConfusableHandle"))
  }

  pure def deleteMember(s: Snapshot, id: str): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else
      Ok(s.keys().exclude(Set(id)).mapBy(k => s.get(k)))

  /// True iff `skel` collides with any member OTHER than `selfId`.
  pure def skeletonTakenByOther(s: Snapshot, selfId: str, skel: str): bool =
    s.keys().exists(k => k != selfId and s.get(k).skeleton == skel)

  pure def updateHandle(s: Snapshot, id: str, newHandle: str, newSkel: str): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else if (skeletonTakenByOther(s, id, newSkel))
      Err("ConfusableHandle")
    else
      Ok(s.put(id, s.get(id).with("handle", newHandle).with("skeleton", newSkel)))

  pure def updateNameSurname(s: Snapshot, id: str, newName: str, newSurname: str): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else
      Ok(s.put(id, s.get(id).with("name", newName).with("surname", newSurname)))

  run deleteMemberOkTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    assert(deleteMember(base, "a") == Ok(Map()))
  }

  run deleteMemberMissingTest = {
    assert(deleteMember(emptyTrie, "ghost") == Err("IdNotFound"))
  }

  run updateHandleOkTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val exp  = base.put("a", base.get("a").with("handle", "alice2").with("skeleton", "alice2"))
    assert(updateHandle(base, "a", "alice2", "alice2") == Ok(exp))
  }

  run updateHandleConfusableTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"), "b" -> leaf("b", "bob", "bob"))
    assert(updateHandle(base, "a", "bob2", "bob") == Err("ConfusableHandle"))
  }

  run updateNameSurnameOkTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val exp  = base.put("a", base.get("a").with("name", "A").with("surname", "B"))
    assert(updateNameSurname(base, "a", "A", "B") == Ok(exp))
  }

  pure def rotateKey(s: Snapshot, id: str, newKey: Key): Result =
    if (not(s.keys().contains(id))) Err("IdNotFound")
    else Ok(s.put(id, s.get(id).with("pKey", newKey)))

  pure def addDevice(s: Snapshot, id: str, d: Key): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else if (s.get(id).devices.size() >= MAX_DEVICES)
      Err("DeviceSlotsFull")
    else if (s.get(id).devices.contains(d))
      Err("DuplicateDevice")
    else
      Ok(s.put(id, s.get(id).with("devices", s.get(id).devices.union(Set(d)))))

  pure def deleteDevice(s: Snapshot, id: str, d: Key, newKey: Key): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else if (not(s.get(id).devices.contains(d)))
      Err("DeviceNotFound")
    else
      Ok(s.put(id, s.get(id)
        .with("devices", s.get(id).devices.exclude(Set(d)))
        .with("pKey", newKey)))

  pure def isolate(s: Snapshot, id: str, newKey: Key): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else
      Ok(s.put(id, s.get(id).with("devices", Set()).with("pKey", newKey)))

  run rotateKeyOkTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val nk = { owner: "a", gen: 1 }
    assert(rotateKey(base, "a", nk) == Ok(base.put("a", base.get("a").with("pKey", nk))))
  }

  run addDeviceOkTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val d = { owner: "a-d2", gen: 0 }
    val exp = base.put("a", base.get("a").with("devices", base.get("a").devices.union(Set(d))))
    assert(addDevice(base, "a", d) == Ok(exp))
  }

  run addDeviceDuplicateTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    assert(addDevice(base, "a", { owner: "a", gen: 0 }) == Err("DuplicateDevice"))
  }

  run deleteDeviceRotatesKeyTest = {
    val d2 = { owner: "a-d2", gen: 0 }
    val base = Map("a" -> leaf("a", "alice", "alice").with("devices", Set({ owner: "a", gen: 0 }, d2)))
    val nk = { owner: "a", gen: 1 }
    val exp = base.put("a", base.get("a").with("devices", Set({ owner: "a", gen: 0 })).with("pKey", nk))
    assert(deleteDevice(base, "a", d2, nk) == Ok(exp))
  }

  run isolateRemovesAllAndRotatesTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val nk = { owner: "a", gen: 1 }
    val exp = base.put("a", base.get("a").with("devices", Set()).with("pKey", nk))
    assert(isolate(base, "a", nk) == Ok(exp))
  }

  /// Ids present in `oldS` but absent in `newS`.
  pure def removedIds(oldS: Snapshot, newS: Snapshot): Set[str] =
    oldS.keys().filter(k => not(newS.keys().contains(k)))

  /// Leaves in `newS` that are new or changed vs `oldS` (observable change).
  pure def upsertedLeaves(oldS: Snapshot, newS: Snapshot): Set[Leaf] =
    newS.keys()
      .filter(k => not(oldS.keys().contains(k)) or oldS.get(k) != newS.get(k))
      .map(k => newS.get(k))

  pure def calculateDelta(oldS: Snapshot, newS: Snapshot): Delta = {
    baseRoot: oldS,
    removed: removedIds(oldS, newS),
    upserted: upsertedLeaves(oldS, newS),
  }

  /// Canonical-form acceptance + application. Mirrors apply_delta, in the
  /// crate's check order: base must match; removed subseteq base; upserts
  /// observable; removed/upserted disjoint; post-state skeletons unique (over
  /// the whole set, independent of identifier order — LLR-n5t6bn); device caps
  /// hold. The cap branch is unreachable from the crate, whose member records
  /// cannot hold more than MAX_DEVICES keys by type.
  pure def applyDelta(s: Snapshot, d: Delta): Result =
    val upsertIds = d.upserted.map(l => l.id)
    if (d.baseRoot != s)
      Err("DeltaBaseMismatch")
    else if (not(d.removed.subseteq(s.keys())))
      Err("StaleRemoval")
    else if (d.upserted.exists(l => s.keys().contains(l.id) and s.get(l.id) == l))
      Err("NoOpUpsert")
    else if (d.removed.intersect(upsertIds) != Set())
      Err("RemoveUpsertOverlap")
    else
      val afterRemove = s.keys().exclude(d.removed).mapBy(k => s.get(k))
      val afterUpsert = d.upserted.fold(afterRemove, (acc, l) => acc.put(l.id, l))
      if (afterUpsert.keys().exists(k =>
            afterUpsert.keys().exists(k2 =>
              k != k2 and afterUpsert.get(k).skeleton == afterUpsert.get(k2).skeleton)))
        Err("ConfusableHandle")
      else if (afterUpsert.keys().exists(k => afterUpsert.get(k).devices.size() > MAX_DEVICES))
        Err("DeviceSlotsFull")
      else
        Ok(afterUpsert)

  run calcDeltaIdentityTest = {
    val s = Map("a" -> leaf("a", "alice", "alice"))
    val d = calculateDelta(s, s)
    assert(d.removed == Set() and d.upserted == Set())
  }

  run roundTripAddTest = {
    val s  = Map("a" -> leaf("a", "alice", "alice"))
    val s2 = Map("a" -> leaf("a", "alice", "alice"), "b" -> leaf("b", "bob", "bob"))
    assert(applyDelta(s, calculateDelta(s, s2)) == Ok(s2))
  }

  run roundTripRemoveTest = {
    val s  = Map("a" -> leaf("a", "alice", "alice"), "b" -> leaf("b", "bob", "bob"))
    val s2 = Map("a" -> leaf("a", "alice", "alice"))
    assert(applyDelta(s, calculateDelta(s, s2)) == Ok(s2))
  }

  run roundTripModifyTest = {
    val s  = Map("a" -> leaf("a", "alice", "alice"))
    val s2 = Map("a" -> leaf("a", "alice", "alice").with("name", "Alice2"))
    assert(applyDelta(s, calculateDelta(s, s2)) == Ok(s2))
  }

  run applyStaleBaseTest = {
    val s  = Map("a" -> leaf("a", "alice", "alice"))
    val other = Map("z" -> leaf("z", "zed", "zed"))
    val d = { baseRoot: other, removed: Set(), upserted: Set(leaf("b","bob","bob")) }
    assert(applyDelta(s, d) == Err("DeltaBaseMismatch"))
  }

  pure def skeletonsUnique(s: Snapshot): bool =
    s.keys().forall(k =>
      s.keys().forall(k2 => k == k2 or s.get(k).skeleton != s.get(k2).skeleton))

  pure def deviceCapOk(s: Snapshot): bool =
    s.keys().forall(k => s.get(k).devices.size() <= MAX_DEVICES)

  run skeletonsUniqueHoldsTest = {
    assert(skeletonsUnique(Map("a" -> leaf("a","alice","alice"), "b" -> leaf("b","bob","bob"))))
  }

  run skeletonsUniqueViolatedTest = {
    assert(not(skeletonsUnique(Map("a" -> leaf("a","alice","x"), "b" -> leaf("b","bob","x")))))
  }

  run deviceCapHoldsTest = {
    assert(deviceCapOk(Map("a" -> leaf("a","alice","alice"))))
  }
}
````

## Appendix C — `org-members/quint/membership_mbt.qnt` — final (T4–T7 add it in the slices each task names)

````
// -*- mode: Bluespec; -*-
/// Runnable state machine over `membership`, replayed against the crate by
/// org-members/tests/mbt_conformance.rs (quint-connect). Every model action
/// records itself in `lastAction` — a sum type whose tag is the action's name
/// and whose payload is its nondeterministic picks — because `quint test`
/// traces carry no `mbt::actionTaken`, and named scenarios must drive the
/// same driver as random runs. See docs/adr/2026-10-03-quint-conformance-gate.md.
module membership_mbt {
  import membership.* from "./membership"

  /// One requested member of a genesis ceremony: identifier, handle, and how
  /// many device keys its record is built with (0 and 5 are refused).
  type Seed = { id: str, h: str, devs: int }

  /// One producer-side operation, for honest deltas.
  type Op = { op: str, id: str, h: str, g: int }

  type Action =
    | Init({ seeds: List[Seed] })
    | AddMember({ id: str, h: str })
    | DeleteMember({ id: str })
    | UpdateHandle({ id: str, h: str })
    | UpdateNameSurname({ id: str, nm: str, sn: str })
    | RotateKey({ id: str, g: int })
    | AddDevice({ id: str, g: int })
    | DeleteDevice({ id: str, g: int })
    | Isolate({ id: str, g: int })
    | ApplyDelta({ kind: str, ops: List[Op], removed: Set[str], upserted: Set[Leaf], target: str })

  /// Whether an organisation exists (a failed genesis leaves none).
  var orgExists: bool
  /// The current trie under test (empty when no organisation exists).
  var trie: Snapshot
  /// The trie before the last successful change.
  var prev: Snapshot
  /// The error tag of the last attempted op, or "" if it succeeded.
  var lastError: str
  /// The action taken to reach this state, with its picks.
  var lastAction: Action

  /// Small finite universes so the simulator explores a bounded space.
  pure val IDS = Set("a", "b", "c")
  pure val HANDLES = Set("h1", "h2", "h3")
  /// Four generations, so a member can reach the device cap (MAX_DEVICES).
  pure val GENS = 0.to(3)
  pure val NAMES = Set("n", "n2")
  pure val SURNAMES = Set("s", "s2")

  /// The record a fresh member is admitted with.
  pure def mkLeaf(id: str, h: str): Leaf =
    { id: id, handle: h, skeleton: h, name: "n", surname: "s",
      pKey: { owner: id, gen: 0 }, devices: Set({ owner: id, gen: 0 }) }

  /// A genesis seed's record: `devs` device keys of generations 0..devs-1.
  pure def seedLeaf(s: Seed): Leaf =
    mkLeaf(s.id, s.h).with("devices", 0.to(s.devs - 1).map(g => { owner: s.id, gen: g }))

  /// The six orders of the three handles.
  pure val HANDLE_ORDERS: Set[List[str]] = Set(
    ["h1", "h2", "h3"], ["h1", "h3", "h2"], ["h2", "h1", "h3"],
    ["h2", "h3", "h1"], ["h3", "h1", "h2"], ["h3", "h2", "h1"])

  pure val NO_OPS: List[Op] = []
  pure val NO_IDS: Set[str] = Set()
  pure val NO_LEAVES: Set[Leaf] = Set()

  /// Apply one producer-side operation (the subset honest deltas draw from).
  pure def applyOp(s: Snapshot, o: Op): Result =
    if (o.op == "add") addMember(s, mkLeaf(o.id, o.h))
    else if (o.op == "delete") deleteMember(s, o.id)
    else if (o.op == "handle") updateHandle(s, o.id, o.h, o.h)
    else rotateKey(s, o.id, { owner: o.id, gen: o.g })

  pure def applyOps(s: Snapshot, ops: List[Op]): Result =
    ops.foldl(Ok(s), (acc, o) => match acc { | Err(e) => Err(e) | Ok(t) => applyOp(t, o) })

  /// Apply a Result: on Ok advance the trie and clear error; on Err keep the
  /// trie and record the tag.
  action commit(r: Result, a: Action): bool = all {
    match r {
      | Ok(s2) => all { trie' = s2, prev' = trie, lastError' = "" }
      | Err(e) => all { trie' = trie, prev' = prev, lastError' = e }
    },
    orgExists' = orgExists,
    lastAction' = a,
  }

  /// A mutation on a missing organisation is refused as the driver refuses
  /// it: there is no record to name a member in.
  action mutate(r: Result, a: Action): bool =
    if (orgExists) commit(r, a) else commit(Err("IdNotFound"), a)

  /// Genesis with an explicit member list — the deterministic form, used by
  /// named scenarios.
  action initWith(seeds: List[Seed]): bool =
    match genesis(seeds.foldl([], (acc, s) => acc.append(seedLeaf(s)))) {
      | Ok(s)  => all { orgExists' = true,  trie' = s,     prev' = s,     lastError' = "", lastAction' = Init({ seeds: seeds }) }
      | Err(e) => all { orgExists' = false, trie' = Map(), prev' = Map(), lastError' = e,  lastAction' = Init({ seeds: seeds }) }
    }

  action init = {
    // Member count weighted toward 2 and 3, so most traces have members.
    nondet nR = 0.to(9).oneOf()
    val n = if (nR == 0) 0 else if (nR == 1) 1 else if (nR <= 5) 2 else 3
    // Start from a valid ceremony (distinct identifiers, distinct handles,
    // one device each) and add at most one flaw: 6 in 10 ceremonies are valid.
    nondet hs = HANDLE_ORDERS.oneOf()
    nondet flaw = 0.to(9).oneOf()
    val valid = [{ id: "a", h: hs[0], devs: 1 },
                 { id: "b", h: hs[1], devs: 1 },
                 { id: "c", h: hs[2], devs: 1 }].slice(0, n)
    val seeds =
      if (n == 0) valid
      else if (flaw == 0 and n >= 2) valid.replaceAt(1, valid[1].with("id", valid[0].id))
      else if (flaw == 1 and n >= 2) valid.replaceAt(1, valid[1].with("h", valid[0].h))
      else if (flaw == 2) valid.replaceAt(n - 1, valid[n - 1].with("devs", 0))
      else if (flaw == 3) valid.replaceAt(n - 1, valid[n - 1].with("devs", 5))
      else valid
    initWith(seeds)
  }

  action doAddMember(id: str, h: str): bool =
    mutate(addMember(trie, mkLeaf(id, h)), AddMember({ id: id, h: h }))
  action doDeleteMember(id: str): bool =
    mutate(deleteMember(trie, id), DeleteMember({ id: id }))
  action doUpdateHandle(id: str, h: str): bool =
    mutate(updateHandle(trie, id, h, h), UpdateHandle({ id: id, h: h }))
  action doUpdateNameSurname(id: str, nm: str, sn: str): bool =
    mutate(updateNameSurname(trie, id, nm, sn), UpdateNameSurname({ id: id, nm: nm, sn: sn }))
  action doRotateKey(id: str, g: int): bool =
    mutate(rotateKey(trie, id, { owner: id, gen: g }), RotateKey({ id: id, g: g }))
  action doAddDevice(id: str, g: int): bool =
    mutate(addDevice(trie, id, { owner: id, gen: g }), AddDevice({ id: id, g: g }))
  action doDeleteDevice(id: str, g: int): bool =
    mutate(deleteDevice(trie, id, { owner: id, gen: 0 }, { owner: id, gen: g }), DeleteDevice({ id: id, g: g }))
  action doIsolate(id: str, g: int): bool =
    mutate(isolate(trie, id, { owner: id, gen: g }), Isolate({ id: id, g: g }))

  /// Apply a delta and verify the candidate. `target` "true": the expected
  /// root is the producer's (honest) or the candidate's own (malformed);
  /// "wrong": the expected root is the current record's, which differs from
  /// any successful candidate because every accepted delta changes the record.
  pure def applyAndVerify(s: Snapshot, d: Delta, expected: Snapshot, target: str): Result =
    match applyDelta(s, d) {
      | Err(e) => Err(e)
      | Ok(c)  => if (target == "true" and c == expected) Ok(c)
                  else if (target == "true") Err("VerificationFailed")
                  else if (c == s) Ok(c)
                  else Err("VerificationFailed")
    }

  /// An honest delta: the producer applies `ops` to the current record and
  /// computes the delta; the receiver applies it to the same record.
  action applyHonest(ops: List[Op], target: str): bool = {
    val produced = applyOps(trie, ops)
    val producerOk = match produced { | Ok(_) => true | Err(_) => false }
    val s2 = match produced { | Ok(t) => t | Err(_) => trie }
    all {
      orgExists, producerOk, s2 != trie,
      commit(applyAndVerify(trie, calculateDelta(trie, s2), s2, target),
             ApplyDelta({ kind: "honest", ops: ops, removed: NO_IDS, upserted: NO_LEAVES, target: target })),
    }
  }

  /// A stale delta: computed against the previous record, applied to the current.
  action applyStale(target: str): bool = all {
    orgExists, prev != trie,
    commit(applyAndVerify(trie, calculateDelta(prev, trie), trie, target),
           ApplyDelta({ kind: "stale", ops: NO_OPS, removed: NO_IDS, upserted: NO_LEAVES, target: target })),
  }

  /// A delta built directly from removals and upserts at the current base.
  action applyBuilt(removed: Set[str], upserted: Set[Leaf], target: str): bool = all {
    orgExists,
    // One record per identifier: the crate's canonical order cannot hold two.
    upserted.map(l => l.id).size() == upserted.size(),
    val d = { baseRoot: trie, removed: removed, upserted: upserted }
    val expected = match applyDelta(trie, d) { | Ok(c) => c | Err(_) => trie }
    commit(applyAndVerify(trie, d, expected, target),
           ApplyDelta({ kind: "built", ops: NO_OPS, removed: removed, upserted: upserted, target: target })),
  }

  pure val OPS: Set[Op] =
    tuples(Set("add", "delete", "handle", "rotate"), IDS, HANDLES, GENS)
      .map(((o, i, h, g)) => { op: o, id: i, h: h, g: g })

  action ApplyDeltaStep = {
    // Integer draws, because a Set drops duplicates: honest 2/4, stale 1/4,
    // built 1/4; target "true" 3/4.
    nondet kindR = 0.to(3).oneOf()
    nondet targetR = 0.to(3).oneOf()
    val kind = if (kindR <= 1) "honest" else if (kindR == 2) "stale" else "built"
    val target = if (targetR <= 2) "true" else "wrong"
    nondet nOps = Set(1, 2).oneOf()
    nondet o1 = OPS.oneOf()
    nondet o2 = OPS.oneOf()
    nondet removed = IDS.powerset().filter(r => r.size() <= 2).oneOf()
    nondet upIds = IDS.powerset().filter(u => u.size() <= 2).oneOf()
    nondet upH = HANDLES.oneOf()
    nondet keepCurrent = Set(true, false).oneOf()
    val upserted = upIds.map(i =>
      if (keepCurrent and trie.keys().contains(i)) trie.get(i) else mkLeaf(i, upH))
    if (kind == "honest") applyHonest([o1, o2].slice(0, nOps), target)
    else if (kind == "stale") applyStale(target)
    else applyBuilt(removed, upserted, target)
  }

  action step = any {
    nondet id = IDS.oneOf()
    nondet h = HANDLES.oneOf()
    doAddMember(id, h),
    nondet id = IDS.oneOf()
    doDeleteMember(id),
    nondet id = IDS.oneOf()
    nondet h = HANDLES.oneOf()
    doUpdateHandle(id, h),
    nondet id = IDS.oneOf()
    nondet nm = NAMES.oneOf()
    nondet sn = SURNAMES.oneOf()
    doUpdateNameSurname(id, nm, sn),
    nondet id = IDS.oneOf()
    nondet g = GENS.oneOf()
    doRotateKey(id, g),
    nondet id = IDS.oneOf()
    nondet g = GENS.oneOf()
    doAddDevice(id, g),
    nondet id = IDS.oneOf()
    nondet g = GENS.oneOf()
    doDeleteDevice(id, g),
    nondet id = IDS.oneOf()
    nondet g = GENS.oneOf()
    doIsolate(id, g),
    // Listed three times: deltas are the cross-process path and the richest
    // in error branches, so they get three of eleven draws.
    ApplyDeltaStep,
    ApplyDeltaStep,
    ApplyDeltaStep,
  }

  // ---- Named scenarios (decision 12): replayed against the crate by
  // `#[quint_test]`, one test function each in mbt_conformance.rs.

  pure val TWO: List[Seed] = [{ id: "a", h: "h1", devs: 1 }, { id: "b", h: "h2", devs: 1 }]
  pure def hop(id: str, h: str): Op = { op: "handle", id: id, h: h, g: 0 }

  // One happy path per model action.
  run scenarioGenesis = initWith(TWO)
  // Every record is built before genesis sees any: a later record's bad
  // device list wins over an earlier duplicate identifier.
  run scenarioGenesisDeviceErrorFirst =
    initWith([{ id: "a", h: "h1", devs: 1 }, { id: "a", h: "h2", devs: 0 }])
  run scenarioAddMember = initWith(TWO).then(doAddMember("c", "h3"))
  run scenarioDeleteMember = initWith(TWO).then(doDeleteMember("b"))
  run scenarioUpdateHandle = initWith(TWO).then(doUpdateHandle("a", "h3"))
  run scenarioUpdateNameSurname = initWith(TWO).then(doUpdateNameSurname("a", "n2", "s2"))
  run scenarioRotateKey = initWith(TWO).then(doRotateKey("a", 1))
  run scenarioAddDevice = initWith(TWO).then(doAddDevice("a", 1))
  run scenarioDeleteDevice = initWith(TWO).then(doAddDevice("a", 1)).then(doDeleteDevice("a", 1))
  run scenarioIsolate = initWith(TWO).then(doIsolate("a", 1))
  run scenarioApplyHonest = initWith(TWO).then(applyHonest([hop("a", "h3")], "true"))
  run scenarioApplyStale = initWith(TWO).then(doRotateKey("a", 1)).then(applyStale("true"))
  run scenarioApplyVerifyFail = initWith(TWO).then(applyHonest([hop("a", "h3")], "wrong"))

  // PR-vf5hdm: a handle moved between two present members in one delta, in
  // both identifier orders, and a two-way swap (REQ-wx3wpv, LLR-n5t6bn).
  run scenarioHandoverAtoB = initWith(TWO).then(applyHonest([hop("a", "h3"), hop("b", "h1")], "true"))
  run scenarioHandoverBtoA = initWith(TWO).then(applyHonest([hop("b", "h3"), hop("a", "h2")], "true"))
  run scenarioHandleSwap = initWith(TWO).then(applyHonest([hop("a", "h3"), hop("b", "h1"), hop("a", "h2")], "true"))

  // Error branches random runs reach rarely (measured in the plan, T6).
  run scenarioDeviceSlotsFull = initWith(TWO)
    .then(doAddDevice("a", 1)).then(doAddDevice("a", 2)).then(doAddDevice("a", 3))
    .then(doAddDevice("a", 0))
  run scenarioRemoveUpsertOverlap = initWith(TWO)
    .then(applyBuilt(Set("a"), Set(mkLeaf("a", "h3")), "true"))
  // A member both removed and re-upserted unchanged: the crate reports the
  // no-op upsert before the overlap.
  run scenarioNoOpBeforeOverlap = initWith(TWO)
    .then(applyBuilt(Set("a"), Set(mkLeaf("a", "h1")), "true"))
  run scenarioBuiltVerifyFail = initWith(TWO)
    .then(applyBuilt(Set("b"), Set(mkLeaf("c", "h3")), "wrong"))

  /// Sanity invariants the simulator should never violate.
  val mbtInv = and {
    skeletonsUnique(trie),
    deviceCapOk(trie),
    orgExists or trie == Map(),
  }
}
````

## Appendix D — `org-members/tests/mbt_conformance.rs` — final, before T7/T8 annotations (T4–T7 add it in slices)

````rust
//! Conformance test: replays traces of the membership model
//! (`org-members/quint/membership_mbt.qnt`) against the real `OrgTrie` through
//! quint-connect, requiring after every model action the same result, the same
//! error and the same membership state, plus root-hash equality classes that
//! match the model's.
//!
//! What "maps 1:1" means here is decided in
//! `docs/adr/2026-10-03-quint-conformance-gate.md` (decisions 4, 8, 10-12):
//! every model action has exactly one driver arm, every trie-yielding public
//! operation has a model action or a declared-abstraction-boundary entry
//! (`TRIE_OPERATIONS`, checked against the source), every action is taken in
//! the random run, and every error tag maps to exactly one crate result.
//!
//! Requires the `quint` CLI on PATH and a writable `$HOME` (Quint's default
//! rust backend fetches its evaluator into `~/.quint` on first use). It does
//! NOT skip when either is missing: `quint_preflight` names the cause and the
//! test fails (decision 9).

use quint_connect::runner::{self, RunConfig, TestConfig};
use quint_connect::*;
use serde::Deserialize;

use anyhow::anyhow;
use ed25519_dalek::SigningKey;
use org_members::delta::test_support;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::types::{MemberId, MemberLeaf, P2pDeviceKey, P2pMemberKey};
use org_members::OrgMembersError;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

type Trie = OrgTrie<Blake3Hasher>;

const SPEC: &str = "quint/membership_mbt.qnt";
const IDS: [&str; 3] = ["a", "b", "c"];
/// Key generations the model draws (0..=3) plus the two only a refused
/// five-device genesis record uses, so every key the driver builds inverts.
const GENS: [i64; 5] = [0, 1, 2, 3, 4];
/// Random-run bounds (decision 7): explicit, per unit.
const MAX_SAMPLES: usize = 100;
const MAX_STEPS: usize = 15;

/// Every model action, by its `lastAction` tag.
const ACTIONS: [&str; 10] = [
    "Init",
    "AddMember",
    "DeleteMember",
    "UpdateHandle",
    "UpdateNameSurname",
    "RotateKey",
    "AddDevice",
    "DeleteDevice",
    "Isolate",
    "ApplyDelta",
];

/// Decision 8(i): every public operation that yields a trie or a candidate,
/// mapped to the model action that drives it or to its declared abstraction
/// boundary. `trie_operations_table_matches_source` fails when the source
/// gains one that is not listed here.
const TRIE_OPERATIONS: [(&str, &str); 11] = [
    ("genesis", "Init"),
    ("add_member", "AddMember"),
    ("delete_member", "DeleteMember"),
    ("update_name_surname", "UpdateNameSurname"),
    ("update_handle", "UpdateHandle"),
    ("rotate_p2p_key", "RotateKey"),
    ("add_p2p_device", "AddDevice"),
    ("delete_p2p_device", "DeleteDevice"),
    ("emergency_isolate_member", "Isolate"),
    ("apply_delta", "ApplyDelta"),
    ("verify_against", "ApplyDelta"),
];

/// Declared abstraction boundary: what the model deliberately does not
/// describe, and where it is carried instead.
const BOUNDARY: [(&str, &str); 1] = [(
    "recalculate",
    "hash lifecycle: called by the driver after every change and to produce \
     honest deltas; pending/recalculated state is carried by integration_test.rs",
)];
// Also outside the model, with no public trie-yielding operation of their own:
// UTS#39 skeletons (model skeleton == handle; carried by the handle tests in
// integration_test.rs), encoding-level delta non-canonicality — ordering and
// duplicate list entries (delta_canonicality_fuzz), wire encoding
// (fuzz_tests.rs), SMT internals and hashing (integration_test.rs).

/// Mirror of the Quint `Key` record.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Debug, serde::Serialize)]
struct Key {
    owner: String,
    gen: i64,
}

/// Mirror of the Quint `Leaf` record. `Ord` because `ApplyDelta` picks a
/// `Set[Leaf]`.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Debug, serde::Serialize)]
struct Leaf {
    id: String,
    handle: String,
    skeleton: String,
    name: String,
    surname: String,
    #[serde(rename = "pKey")]
    p_key: Key,
    devices: BTreeSet<Key>,
}

/// Mirror of the Quint `Seed` record (one requested genesis member).
#[derive(Clone, Deserialize, Debug)]
struct Seed {
    id: String,
    h: String,
    devs: i64,
}

/// Mirror of the Quint `Op` record (one producer-side operation).
#[derive(Clone, Deserialize, Debug)]
struct Op {
    op: String,
    id: String,
    h: String,
    g: i64,
}

/// The verifiable model state. `prev` and `lastAction` are model bookkeeping
/// and are not compared.
#[derive(Eq, PartialEq, Deserialize, Debug)]
struct MembershipState {
    #[serde(rename = "orgExists")]
    org_exists: bool,
    trie: BTreeMap<String, Leaf>,
    #[serde(rename = "lastError")]
    last_error: String,
}

fn real_id(model_id: &str) -> MemberId {
    MemberId::new(blake3::hash(format!("id:{model_id}").as_bytes()).into())
}
fn real_member_key(k: &Key) -> P2pMemberKey {
    let seed: [u8; 32] = blake3::hash(format!("mk:{}:{}", k.owner, k.gen).as_bytes()).into();
    P2pMemberKey::new(SigningKey::from_bytes(&seed).verifying_key())
}
fn real_device_key(k: &Key) -> P2pDeviceKey {
    let seed: [u8; 32] = blake3::hash(format!("dk:{}:{}", k.owner, k.gen).as_bytes()).into();
    P2pDeviceKey::new(SigningKey::from_bytes(&seed).verifying_key())
}
fn key(owner: &str, gen: i64) -> Key {
    Key {
        owner: owner.to_string(),
        gen,
    }
}

/// The record a fresh member is admitted with (model `mkLeaf`).
fn new_leaf(id: &str, h: &str) -> core::result::Result<MemberLeaf, OrgMembersError> {
    MemberLeaf::new(
        real_id(id),
        h,
        real_member_key(&key(id, 0)),
        "n",
        "s",
        vec![real_device_key(&key(id, 0))],
    )
}

/// A genesis seed's record (model `seedLeaf`).
fn seed_leaf(s: &Seed) -> core::result::Result<MemberLeaf, OrgMembersError> {
    MemberLeaf::new(
        real_id(&s.id),
        &s.h,
        real_member_key(&key(&s.id, 0)),
        "n",
        "s",
        (0..s.devs).map(|g| real_device_key(&key(&s.id, g))).collect(),
    )
}

// --- inverse maps (domains are tiny, so brute force) ---
fn model_id_of(id: &MemberId) -> Option<String> {
    IDS.iter().find(|m| real_id(m) == *id).map(|m| m.to_string())
}
fn gen_of_member_key(owner: &str, k: &P2pMemberKey) -> Option<i64> {
    GENS.iter().copied().find(|g| real_member_key(&key(owner, *g)) == *k)
}
fn model_device_of(owner: &str, d: &P2pDeviceKey) -> Option<Key> {
    GENS.iter()
        .copied()
        .find(|g| real_device_key(&key(owner, *g)) == *d)
        .map(|g| key(owner, g))
}

fn model_leaf_of(m: &MemberLeaf) -> Result<Leaf> {
    let mid = model_id_of(m.id()).ok_or_else(|| anyhow!("unknown member id"))?;
    let gen = gen_of_member_key(&mid, m.p2p_key())
        .ok_or_else(|| anyhow!("unknown member key gen for {mid}"))?;
    let mut devices = BTreeSet::new();
    for d in m.p2p_devices() {
        devices.insert(model_device_of(&mid, d).ok_or_else(|| anyhow!("unknown device for {mid}"))?);
    }
    Ok(Leaf {
        id: mid.clone(),
        handle: m.handle().to_string(),
        skeleton: m.handle().to_string(), // model invariant: skeleton == handle
        name: m.name().to_string(),
        surname: m.surname().to_string(),
        p_key: key(&mid, gen),
        devices,
    })
}

/// Map a crate error to the model's error tag — exactly one crate result per
/// tag (decision 10). The model's handles have distinct skeletons, so a model
/// "ConfusableHandle" is the crate's `DuplicateHandle`; the crate's own
/// `ConfusableHandle` (distinct handles, one skeleton) is outside the model
/// and maps to "Other", forcing a visible mismatch if it is ever reached.
fn err_tag(e: &OrgMembersError) -> String {
    match e {
        OrgMembersError::IdNotFound => "IdNotFound",
        OrgMembersError::DuplicateId => "DuplicateId",
        OrgMembersError::DuplicateHandle => "ConfusableHandle",
        OrgMembersError::DuplicateDevice => "DuplicateDevice",
        OrgMembersError::DeviceNotFound => "DeviceNotFound",
        OrgMembersError::DeviceSlotsFull => "DeviceSlotsFull",
        OrgMembersError::EmptyDeviceList => "EmptyDeviceList",
        OrgMembersError::DeltaBaseMismatch => "DeltaBaseMismatch",
        OrgMembersError::VerificationFailed => "VerificationFailed",
        OrgMembersError::MalformedDelta("removed id not present in trie") => "StaleRemoval",
        OrgMembersError::MalformedDelta("upserted leaf identical to existing trie state") => {
            "NoOpUpsert"
        }
        OrgMembersError::MalformedDelta("id appears in both removed and upserted") => {
            "RemoveUpsertOverlap"
        }
        other => return format!("Other:{other:?}"),
    }
    .to_string()
}

/// Apply one producer-side operation (model `applyOp`).
fn apply_op(t: &Trie, o: &Op) -> core::result::Result<Trie, OrgMembersError> {
    match o.op.as_str() {
        "add" => t.add_member(new_leaf(&o.id, &o.h)?),
        "delete" => t.delete_member(&real_id(&o.id)),
        "handle" => t.update_handle(&real_id(&o.id), &o.h),
        "rotate" => t.rotate_p2p_key(&real_id(&o.id), real_member_key(&key(&o.id, o.g))),
        _ => Err(OrgMembersError::InvariantViolated),
    }
}

/// Action and outcome counts across every trace of a run (decision 8(ii)).
type Coverage = Arc<Mutex<BTreeMap<(String, String), usize>>>;

#[derive(Default)]
struct MembershipDriver {
    trie: Option<Trie>,
    prev: Option<Trie>,
    last_error: String,
    // root-hash equality classes: canonical model-state bytes -> root hex
    root_classes: std::collections::HashMap<Vec<u8>, String>,
    coverage: Option<Coverage>,
}

impl MembershipDriver {
    fn with_coverage(coverage: Coverage) -> Self {
        Self {
            coverage: Some(coverage),
            ..Self::default()
        }
    }

    fn commit(&mut self, res: core::result::Result<Trie, OrgMembersError>) {
        match res {
            Ok(t) => {
                // Mutations leave the trie with uncalculated hashes; recalculate
                // so `root_hash()` succeeds. This does not change observable
                // model state (member contents), only fills the hash cache.
                let t = match t.recalculate() {
                    Ok((t2, _delta)) => t2,
                    Err(_) => t,
                };
                // Delta-path conformance: for the real mutation old -> t, the crate's
                // calculate_delta + apply_delta + verify_against must reproduce t's
                // root (the model's round-trip law, against the real crate).
                if let Some(old) = &self.trie {
                    if let (Ok(old_root), Ok(new_root)) = (old.root_hash(), t.root_hash()) {
                        if old_root != new_root {
                            let delta = t
                                .calculate_delta(old)
                                .expect("calculate_delta failed on a real mutation");
                            let verified = old
                                .apply_delta(&delta)
                                .expect("apply_delta rejected a canonical delta from calculate_delta")
                                .verify_against(&new_root)
                                .expect("verify_against failed for the calculated delta");
                            assert_eq!(
                                verified.root_hash().expect("root_hash of verified trie"),
                                new_root,
                                "delta round-trip produced a different root than the direct mutation"
                            );
                        }
                    }
                }
                self.prev = self.trie.take();
                self.trie = Some(t);
                self.last_error = String::new();
            }
            Err(e) => {
                self.last_error = err_tag(&e);
            }
        }
    }

    fn cur(&self) -> core::result::Result<Trie, OrgMembersError> {
        self.trie.clone().ok_or(OrgMembersError::IdNotFound)
    }

    /// The real counterpart of a model leaf in an `ApplyDelta` upsert: the
    /// current record itself when the model leaf equals it, else a fresh
    /// record (model `mkLeaf`, the only other shape the model builds).
    fn real_leaf(&self, cur: &Trie, l: &Leaf) -> core::result::Result<MemberLeaf, OrgMembersError> {
        if let Some(existing) = cur.get(&real_id(&l.id)) {
            if model_leaf_of(&existing).ok().as_ref() == Some(l) {
                return Ok(existing);
            }
        }
        new_leaf(&l.id, &l.handle)
    }

    fn apply_delta_step(
        &self,
        kind: &str,
        ops: &[Op],
        removed: &BTreeSet<String>,
        upserted: &BTreeSet<Leaf>,
        target: &str,
    ) -> core::result::Result<Trie, OrgMembersError> {
        let cur = self.cur()?;
        match kind {
            // The producer performs `ops` and computes the delta itself
            // (REQ-wx3wpv: a change set the software produced).
            "honest" => {
                let mut produced = cur.clone();
                for o in ops {
                    produced = apply_op(&produced, o)?;
                }
                let (produced, delta) = produced.recalculate()?;
                let expected = if target == "true" {
                    produced.root_hash()?
                } else {
                    cur.root_hash()?
                };
                cur.apply_delta(&delta)?.verify_against(&expected)
            }
            // Computed against the previous record, applied to the current.
            "stale" => {
                let prev = self.prev.clone().ok_or(OrgMembersError::IdNotFound)?;
                let delta = cur.calculate_delta(&prev)?;
                cur.apply_delta(&delta)?.verify_against(&cur.root_hash()?)
            }
            // Built from removals and upserts at the current base, in
            // canonical (strictly increasing) order.
            "built" => {
                let mut delta = cur.calculate_delta(&cur)?;
                let mut ids: Vec<MemberId> = removed.iter().map(|i| real_id(i)).collect();
                ids.sort();
                let mut leaves = upserted
                    .iter()
                    .map(|l| self.real_leaf(&cur, l))
                    .collect::<core::result::Result<Vec<_>, _>>()?;
                leaves.sort_by(|a, b| a.id().cmp(b.id()));
                test_support::delta_set_removed(&mut delta, ids);
                test_support::delta_set_upserted(&mut delta, leaves);
                let candidate = cur.apply_delta(&delta)?;
                let expected = if target == "true" {
                    candidate.root_hash()
                } else {
                    cur.root_hash()?
                };
                candidate.verify_against(&expected)
            }
            _ => Err(OrgMembersError::InvariantViolated),
        }
    }

    /// Reconstruct the model trie from the real OrgTrie via inverse maps.
    fn model_trie(&self) -> Result<BTreeMap<String, Leaf>> {
        let mut out = BTreeMap::new();
        if let Some(t) = &self.trie {
            for m in t.members() {
                let l = model_leaf_of(&m)?;
                out.insert(l.id.clone(), l);
            }
        }
        Ok(out)
    }
}

impl State<MembershipDriver> for MembershipState {
    fn from_driver(driver: &MembershipDriver) -> Result<Self> {
        Ok(MembershipState {
            org_exists: driver.trie.is_some(),
            trie: driver.model_trie()?,
            last_error: driver.last_error.clone(),
        })
    }
}

impl Driver for MembershipDriver {
    type State = MembershipState;

    /// The model records each action in `lastAction` (decision 12): `quint
    /// test` traces carry no `mbt::actionTaken`, and named scenarios must
    /// drive this same driver.
    fn config() -> Config {
        Config {
            state: &[],
            nondet: &["lastAction"],
        }
    }

    fn step(&mut self, step: &Step) -> Result {
        let switch_result: Result = (|| {
            switch!(step {
            Init(seeds: Vec<Seed>) => {
                let res = seeds
                    .iter()
                    .map(seed_leaf)
                    .collect::<core::result::Result<Vec<_>, _>>()
                    .and_then(Trie::genesis);
                self.trie = None;
                self.prev = None;
                match res {
                    Ok(t) => {
                        self.prev = Some(t.clone());
                        self.trie = Some(t);
                        self.last_error = String::new();
                    }
                    Err(e) => self.last_error = err_tag(&e),
                }
            },
            AddMember(id: String, h: String) => {
                let res = self.cur().and_then(|t| t.add_member(new_leaf(&id, &h)?));
                self.commit(res);
            },
            DeleteMember(id: String) => {
                let res = self.cur().and_then(|t| t.delete_member(&real_id(&id)));
                self.commit(res);
            },
            UpdateHandle(id: String, h: String) => {
                let res = self.cur().and_then(|t| t.update_handle(&real_id(&id), &h));
                self.commit(res);
            },
            UpdateNameSurname(id: String, nm: String, sn: String) => {
                let res = self.cur().and_then(|t| t.update_name_surname(&real_id(&id), &nm, &sn));
                self.commit(res);
            },
            RotateKey(id: String, g: i64) => {
                let nk = real_member_key(&key(&id, g));
                let res = self.cur().and_then(|t| t.rotate_p2p_key(&real_id(&id), nk));
                self.commit(res);
            },
            AddDevice(id: String, g: i64) => {
                let d = real_device_key(&key(&id, g));
                let res = self.cur().and_then(|t| t.add_p2p_device(&real_id(&id), d));
                self.commit(res);
            },
            DeleteDevice(id: String, g: i64) => {
                let d = real_device_key(&key(&id, 0));
                let nk = real_member_key(&key(&id, g));
                let res = self.cur().and_then(|t| t.delete_p2p_device(&real_id(&id), &d, nk));
                self.commit(res);
            },
            Isolate(id: String, g: i64) => {
                let nk = real_member_key(&key(&id, g));
                let res = self.cur().and_then(|t| t.emergency_isolate_member(&real_id(&id), nk));
                self.commit(res);
            },
            ApplyDelta(kind: String, ops: Vec<Op>, removed: BTreeSet<String>, upserted: BTreeSet<Leaf>, target: String) => {
                let res = self.apply_delta_step(&kind, &ops, &removed, &upserted, &target);
                self.commit(res);
            }
        })
        })();
        switch_result?;

        if let Some(cov) = &self.coverage {
            let outcome = if self.last_error.is_empty() { "Ok".to_string() } else { self.last_error.clone() };
            *cov.lock().expect("coverage lock")
                .entry((step.action_taken.clone(), outcome))
                .or_insert(0) += 1;
        }

        // Root-hash equality-class check: equal model state <=> equal real root.
        if self.last_error.is_empty() {
            if let Some(t) = &self.trie {
                let root = t.root_hash().map_err(|e| anyhow!("root_hash: {e:?}"))?;
                let root_hex = hex::encode(root.as_bytes());
                let key = serde_json::to_vec(&self.model_trie()?).unwrap_or_default();
                match self.root_classes.get(&key) {
                    Some(prev) if *prev != root_hex => {
                        return Err(anyhow!(
                            "abstraction violated: equal model state, different roots"
                        ))
                    }
                    None => {
                        if self.root_classes.values().any(|v| v == &root_hex) {
                            return Err(anyhow!(
                                "abstraction violated: distinct model states share a root"
                            ));
                        }
                        self.root_classes.insert(key, root_hex);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

/// Decision 9: name the cause when the Quint toolchain is unusable, rather
/// than letting quint-connect panic with "Failed to execute Quint command".
fn quint_preflight() -> core::result::Result<(), String> {
    let out = std::process::Command::new("quint")
        .arg("--version")
        .output()
        .map_err(|e| format!("`quint` is not runnable from PATH ({e}); install it (npm i -g @informalsystems/quint)"))?;
    if !out.status.success() {
        return Err(format!("`quint --version` failed: {}", String::from_utf8_lossy(&out.stderr)));
    }
    let home = std::env::var_os("HOME").ok_or("HOME is not set; quint's rust backend needs ~/.quint")?;
    let dir = std::path::Path::new(&home).join(".quint");
    std::fs::create_dir_all(&dir)
        .and_then(|_| tempfile_probe(&dir))
        .map_err(|e| format!("{} is not writable ({e}); quint's rust backend fetches its evaluator there", dir.display()))
}

/// Write and remove a probe file. The name is unique per process and thread:
/// the scenario tests run in parallel, and a shared name lets one test delete
/// another's probe mid-check (measured: a spurious preflight failure).
fn tempfile_probe(dir: &std::path::Path) -> std::io::Result<()> {
    let p = dir.join(format!(
        ".org-members-preflight-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::write(&p, b"")?;
    std::fs::remove_file(&p)
}

fn run_conformance(driver: MembershipDriver, name: &str) {
    if let Err(cause) = quint_preflight() {
        panic!("conformance test cannot run: {cause}");
    }
    let config = runner::Config {
        test_name: name.to_string(),
        gen_config: RunConfig {
            spec: SPEC.to_string(),
            main: None,
            init: None,
            step: None,
            max_samples: Some(MAX_SAMPLES),
            max_steps: Some(MAX_STEPS),
            seed: runner::gen_random_seed(),
        },
    };
    if let Err(err) = runner::run_test(driver, config) {
        panic!("{err}");
    }
}

fn run_scenario(test: &str) {
    if let Err(cause) = quint_preflight() {
        panic!("conformance test cannot run: {cause}");
    }
    let config = runner::Config {
        test_name: test.to_string(),
        gen_config: TestConfig {
            spec: SPEC.to_string(),
            main: None,
            test: test.to_string(),
            max_samples: Some(1),
            seed: runner::gen_random_seed(),
        },
    };
    if let Err(err) = runner::run_test(MembershipDriver::default(), config) {
        panic!("{err}");
    }
}

/// verifies: LLR-fv75ec, LLR-j4d38d, LLR-v3jqau
///
/// The random run, with action coverage (decision 8(ii)): every model action
/// must be taken at least once across the run; per-action outcome counts are
/// printed, not gated.
#[test]
fn membership_conformance() {
    let coverage: Coverage = Arc::default();
    run_conformance(MembershipDriver::with_coverage(coverage.clone()), "membership_conformance");
    let counts = coverage.lock().expect("coverage lock");
    for ((action, outcome), n) in counts.iter() {
        println!("coverage {action} -> {outcome}: {n}");
    }
    let missing: Vec<&str> = ACTIONS
        .iter()
        .copied()
        .filter(|a| !counts.keys().any(|(taken, _)| taken == a))
        .collect();
    assert!(missing.is_empty(), "model actions never taken in the random run: {missing:?}");
}

/// Decision 8(i): the operation table matches the crate's public API.
#[test]
fn trie_operations_table_matches_source() {
    let mut found = BTreeSet::new();
    for file in ["src/trie.rs", "src/delta.rs"] {
        let src = std::fs::read_to_string(file).expect("read crate source");
        let mut rest = src.as_str();
        while let Some(i) = rest.find("pub fn ") {
            rest = &rest[i + "pub fn ".len()..];
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            let sig = &rest[..rest.find('{').unwrap_or(rest.len())];
            let ret = sig.rsplit("->").next().unwrap_or("");
            let ret: String = ret.split_whitespace().collect();
            if sig.contains("->")
                && ["Result<Self", "Result<(Self", "Result<CandidateTrie", "Result<OrgTrie"]
                    .iter()
                    .any(|p| ret.starts_with(p))
            {
                found.insert(name);
            }
        }
    }
    let declared: BTreeSet<String> = TRIE_OPERATIONS
        .iter()
        .map(|(op, _)| op.to_string())
        .chain(BOUNDARY.iter().map(|(op, _)| op.to_string()))
        .collect();
    assert_eq!(
        found, declared,
        "trie-yielding public operations and the TRIE_OPERATIONS/BOUNDARY table disagree"
    );
    for (_, action) in TRIE_OPERATIONS {
        assert!(ACTIONS.contains(&action), "{action} is not a model action");
    }
}

/// Decision 9: the preflight names the cause when quint is missing.
#[test]
fn quint_preflight_names_a_missing_binary() {
    let out = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", "preflight_probe", "--nocapture", "--include-ignored"])
        .env("PATH", "")
        .output()
        .expect("re-run test binary");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("PREFLIGHT: `quint` is not runnable from PATH"), "{text}");
}

#[test]
#[ignore = "invoked by quint_preflight_names_a_missing_binary with PATH emptied"]
fn preflight_probe() {
    match quint_preflight() {
        Ok(()) => println!("PREFLIGHT: ok"),
        Err(cause) => println!("PREFLIGHT: {cause}"),
    }
}

// ---- Named scenarios (decision 12), one per `run scenario*` in the model.

#[test]
fn scenario_genesis() {
    run_scenario("scenarioGenesis")
}
#[test]
fn scenario_genesis_device_error_first() {
    run_scenario("scenarioGenesisDeviceErrorFirst")
}
#[test]
fn scenario_add_member() {
    run_scenario("scenarioAddMember")
}
#[test]
fn scenario_delete_member() {
    run_scenario("scenarioDeleteMember")
}
#[test]
fn scenario_update_handle() {
    run_scenario("scenarioUpdateHandle")
}
#[test]
fn scenario_update_name_surname() {
    run_scenario("scenarioUpdateNameSurname")
}
#[test]
fn scenario_rotate_key() {
    run_scenario("scenarioRotateKey")
}
#[test]
fn scenario_add_device() {
    run_scenario("scenarioAddDevice")
}
#[test]
fn scenario_delete_device() {
    run_scenario("scenarioDeleteDevice")
}
#[test]
fn scenario_isolate() {
    run_scenario("scenarioIsolate")
}
#[test]
fn scenario_apply_honest() {
    run_scenario("scenarioApplyHonest")
}
#[test]
fn scenario_apply_stale() {
    run_scenario("scenarioApplyStale")
}
#[test]
fn scenario_apply_verify_fail() {
    run_scenario("scenarioApplyVerifyFail")
}
#[test]
fn scenario_handover_a_to_b() {
    run_scenario("scenarioHandoverAtoB")
}
#[test]
fn scenario_handover_b_to_a() {
    run_scenario("scenarioHandoverBtoA")
}
#[test]
fn scenario_handle_swap() {
    run_scenario("scenarioHandleSwap")
}
#[test]
fn scenario_device_slots_full() {
    run_scenario("scenarioDeviceSlotsFull")
}
#[test]
fn scenario_remove_upsert_overlap() {
    run_scenario("scenarioRemoveUpsertOverlap")
}
#[test]
fn scenario_no_op_before_overlap() {
    run_scenario("scenarioNoOpBeforeOverlap")
}
#[test]
fn scenario_built_verify_fail() {
    run_scenario("scenarioBuiltVerifyFail")
}
````

## Appendix E — `org-members/src/trie.rs` — the PR-vf5hdm fix (T7)

````diff
--- a/org-members/src/trie.rs
+++ b/org-members/src/trie.rs
@@ -542,16 +542,22 @@
             count = count.checked_sub(1).ok_or(OrgMembersError::InvariantViolated)?;
         }
 
+        // LLR-n5t6bn: release every outgoing handle before checking any
+        // incoming one, so a handle moving between two present members is
+        // judged against the record after the whole set, in any identifier
+        // order (PR-vf5hdm).
         for member in &delta.upserted {
-            let existing = smt::get_member(&root, member.id());
-
-            if let Some(ref old) = existing {
+            if let Some(old) = smt::get_member(&root, member.id()) {
                 if old.handle() != member.handle() {
                     new_skeleton_index.remove(&handle_skeleton(old.handle()));
                     new_handle_index.remove(old.handle());
                 }
             }
+        }
 
+        for member in &delta.upserted {
+            let existing = smt::get_member(&root, member.id());
+
             let needs_check = match &existing {
                 Some(old) => old.handle() != member.handle(),
                 None => true,
````

## Gate log (verify-before-merge)

- Round 1 (2026-10-03): all run commands green (org-members 138 passed/1 ignored, org-node 51, app cargo 78; quint 21 passing, mbtInv 1000 × 15 OK); check-ids clean; check-units clean; git clean. Failures: (a) STALE-PROBLEM PR-zz4exm, PR-hvg2dy — pre-existing, **owner-accepted 2026-10-03** (resolved in a separate change); (b) app npm checks not run (no node_modules) — fixed by `npm --prefix app ci --offline`; (c) robustness: no abnormal-input test for REQ-wx3wpv/LLR-n5t6bn, none for LLR-4n8zqx — dispatched as task `robust`.
- Coverage: statement 93.51% lines (floor 92), regions 92.22%; decision coverage not measured (Branches 0/0). **Owner accepted the documented decision-coverage gap on 2026-10-03** (existing gap tracked in the ratchet-setup plan). MSRV: owner ruled the `rust-version = "1.85"` declarations sufficient; no CI run required.
- Task `robust` (merged from `worktree-quint-connect-coupling-robust` @ 61d9a01). Tests passed on unmutated code first (behaviour exists); red evidence by mutation, each reverted. M1 = skip the post-release handle check in `apply_delta`'s second loop; M2 = release loop also frees each upsert's incoming handle; M3 = `insert_leaf` duplicate-id check disabled; M4 = order-dependent leaf hash in `smt::insert`.
  - red -> green: `apply_delta_rejects_two_upserts_claiming_one_handle` (verifies: REQ-wx3wpv, LLR-n5t6bn) — red under M1 (Ok returned).
  - red -> green: `apply_delta_rejects_upsert_taking_handle_of_untouched_member` (REQ-wx3wpv, LLR-n5t6bn) — red under M1 and M2.
  - red -> green: `apply_delta_rejects_confusable_of_untouched_member_after_release` (REQ-wx3wpv, LLR-n5t6bn) — red under M1 and M2 (Ok instead of ConfusableHandle).
  - red -> green: `apply_delta_never_admits_a_handle_collision` (proptest; REQ-wx3wpv, LLR-n5t6bn) — red under M1 ("admitted a handle collision", shrunk removed_mask=8) and M2.
  - red -> green: `rejected_operations_leave_no_trace_in_root` (LLR-4n8zqx) — red under M3 and M4 at the root == direct-genesis assertion.
  - result: `cargo test -p org-members` 143 passed, 1 ignored; check-trace unchanged. The stale "uniqueness carried in name only" comment on `apply_delta_rejects_confusable_in_upsert` updated by the dispatcher.
- Review round 1 (independent reviewer, 2026-10-03): PASS with six findings (2 code/requirement, 4 record); fixed by task `review1fix` (merged from `worktree-quint-connect-coupling-review1fix` @ 0926715).
  - red -> green: LLR-juxk9q annotations on `apply_delta_rejects_two_upserts_claiming_one_handle`, `apply_delta_rejects_upsert_taking_handle_of_untouched_member`, `apply_delta_never_admits_a_handle_collision` — each red with only apply_delta's `DuplicateHandle` return deleted.
  - red -> green: rewritten `rejected_operations_leave_no_trace_in_root` (LLR-4n8zqx) — red at the per-rejection Err assertion with `insert_leaf`'s DuplicateHandle return deleted, and with `update_leaf`'s deleted.
  - REQ-wx3wpv removed from the four rejection tests (they carry LLR-n5t6bn); REQ-wx3wpv covered by the handover/swap scenarios and transitively via LLR-n5t6bn.
- Gate round 4 (tree 12a2b06): PASS — 345 cargo + 30 vitest passed, 1 ignored by design; quint 21 passing, mbtInv OK; only owner-accepted STALE-PROBLEMs.
- Review round 2 (fresh reviewer): PASS with 4 findings (1 requirement: LLR-4n8zqx abnormal carrier non-discriminating; 3 record). Fix dispatched as task `review2fix`.
- Task `review2fix` (merged from `worktree-quint-connect-coupling-review2fix` @ b262e9f): `rejected_operations_leave_no_trace_in_root` (named in the robust/review1fix lines above) was **replaced** by `irregular_history_reaches_same_root_as_direct_build` (verifies: LLR-4n8zqx).
  - red -> green: `irregular_history_reaches_same_root_as_direct_build` — green on correct code first (re-carries existing behaviour); red under M1 (smt::remove leaves a tombstone), M2 (smt::insert mixes the replaced leaf into the new one), M3 (diff_recursive drops removals; red at the receiver's verify_against). Under each, `same_members_same_root_hash` and `different_insertion_order_same_root` stayed green.
  - records: forward notes in 2026-09-17-design-derived.md; third handover shape added to HAZ-y8h835 and the REQ-wx3wpv assessment (its "newcomer sorts before the releasing member" condition is derived from the pre-fix loop, not separately measured); ADR decisions 4, 9, 11, 13 amended (dated).
- Base moved: local master gained 27dab89 (stale-problem fix: PR-zz4exm, PR-hvg2dy). Merged into the change branch by task `basemerge`.
- Task `basemerge` (merged from `worktree-quint-connect-coupling-basemerge` @ 7d07b9d): local master 27dab89 merged. Only `org-members/tests/mbt_conformance.rs` conflicted; git followed the rename and carried master's `P2pKeyNotReplaced` checks and 3 `run` tests into `org-members/quint/membership.qnt`.
  - red -> green: `membership_conformance` as carrier of LLR-s97ywt — red 5/5 seeds with `delete_p2p_device`'s P2pKeyNotReplaced guard removed; green on revert.
  - red -> green: `membership_conformance` as carrier of LLR-w92psx — red 5/5 seeds with `emergency_isolate_member`'s guard removed; green on revert. (Random step reaches same-key replacement often: GENS 0..3, every member starts at gen 0 — no named scenario needed.)
  - result: `cargo test -p org-members` 8 + 118 + 24 passed, 1 ignored; `quint test` 24 passing; mbtInv OK; clippy clean; check-trace org-members and org-node exit 0 — STALE-PROBLEMs gone; check-ids, check-units clean.
  - note: the subagent used `sed` once on mbt_conformance.rs to delete a leftover conflict-marker line (rule breach; result inspected, only that line removed).
- Base moved again: local master gained f635acc (org-node loopback fix); merged into the change branch directly (no conflicts). Gate round 6 (tree c9e0457): PASS — 354 cargo + 30 vitest + 24 quint passed, 1 ignored by design; check-trace exit 0 on all four units.
- Review round 3 (fresh reviewer, tree c9e0457): PASS, no code/requirement findings; 4 record findings → **last review round** (merge-change 6a convergence rule). Fixed by task `review3fix` (merged @ e13106d): ADR 4(d)/8(i)/12 amended; quint/README Protocol-layer sentence; coverage notes re-tallied after base merge (DeleteDevice/Isolate → P2pKeyNotReplaced 20/20; all pairs 20/20; AddDevice → DeviceSlotsFull 0/20, covered by `scenario_device_slots_full`); risk table row for `scenario_handover_b_to_a`.
  - red -> green: `scenario_handover_b_to_a` (REQ-wx3wpv, LLR-n5t6bn) — red with apply_delta's release loop emptied (all four handover scenarios red; measured by reviewer round 3, reproduced by review3fix); green on revert.
