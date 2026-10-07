# Branch Coverage (decision coverage) Plan

**Goal:** Measure and enforce the decision half of the class C coverage target (`docs/adr/2026-09-01-safety-class-c.md`, open item 1) in every unit that already has a statement-coverage target: org-members, on-chain-client and person.
**Implements:** no ledger items. This is a guardrails (tooling) change. It closes the "Decision coverage — mandatory under class C" item of `docs/plans/2026-09-05-ratchet-setup.md` for these three units.
**Owner approval:** 2026-10-06, during the absence-proofs (S1) merge gate: "add the branch testing as per your suggestion" (a pinned nightly with a branch floor, while stable keeps the line and region floors).
**Scope out:** org-node and app. They have no coverage target of any kind; their statement and decision coverage arrive together at tooth 5 (ratchet-setup plan, "Coverage for org-node and app").

## Design

- **Why nightly:** rustc's branch instrumentation (`-Z coverage-options=branch`) is unstable, so `cargo llvm-cov --branch` runs only on nightly. Building, testing and the stable line and region floors stay as they are.
- **The pin:** `BRANCH_TOOLCHAIN := nightly-2026-10-03`, named by the channel-manifest date. That build is rustc 1.101.0-nightly (0abfedbc7 2026-10-02), the one the S1 figure was measured with. CI's `coverage-branch` job pins the same toolchain. Moving the pin is a reviewed edit to both.
- **One suite, two measurements:** the stable and nightly recipes read the same `<UNIT>_COVERAGE_ARGS` variable, so they cannot drift apart.
- **The floor:** cargo-llvm-cov 0.9.0 has no `--fail-under-branches`. The recipe writes the JSON summary and `jq -e` compares `totals.branches.percent` with the floor. A summary that has no branches, or no branch field at all, fails.
- **Separate build directory:** the branch recipes build in `target/branch-coverage`, so nightly binaries never reach `target/llvm-cov-target`. Mixing the two used to fail the stable floors (the person note in the Makefile).
- **Floors:** one point below the measurement, rounded down, raised but never lowered (the same rule as the line and region floors).

## Measurement (2026-10-06, nightly-2026-10-03, cargo-llvm-cov 0.9.0, aarch64-darwin)

| Unit | Branches | Floor | Lines (stable recipe) |
|---|---|---|---|
| org-members | 123 of 124, 99.19% | 98 | 96.55% |
| on-chain-client | 34 of 34, 100% | 99 | 41.75% |
| person | 36 of 36, 100% | 99 | 100% |

The uncovered org-members branch is the `!old.is_calculated()` operand of `calculate_delta`'s guard in `org-members/src/trie.rs`: no test has a calculated `self` with an uncalculated `old`. It predates S1.

**Caveat, found while measuring:** a branch figure counts only functions that ran, so on-chain-client's 100% does not close its statement shortfall. The full note is in the Makefile, beside the branch floors.

## Tasks

### T1 — Makefile targets, configs, CI, docs

**Files touched:** Makefile, .github/workflows/rust.yml, org-members/.guardrails/config.yaml, on-chain-client/.guardrails/config.yaml, person/.guardrails/config.yaml, docs/adr/2026-09-01-safety-class-c.md, docs/plans/2026-09-05-ratchet-setup.md, app/docs/risk/2026-09-14-app-hazards.md (a one-line amendment to its "decision coverage is unmeasured in every unit"), this plan.

1. Move each stable recipe's arguments into `ORG_MEMBERS_COVERAGE_ARGS`, `ON_CHAIN_CLIENT_COVERAGE_ARGS` and `PERSON_COVERAGE_ARGS`. Check that `make coverage` still passes.
2. Add `BRANCH_TOOLCHAIN`, `BRANCH_TARGET_DIR`, `BRANCH_REPORT_DIR`, the three branch floors, `judge_branches`, `coverage-branch-<unit>` × 3 and `coverage-branch`.
3. Add `make coverage-branch-<unit>` to each unit's `coverage_command`.
4. Add a `coverage-branch` CI job on the pinned nightly.
5. Amend the ADR's open item 1 and the ratchet-setup item.

**Verification:**
- `make coverage-branch` exits 0 and prints the three figures above.
- A floor above the measurement fails: `make coverage-branch-person PERSON_BRANCHES=100.5` exits non-zero with "below its floor".
- `make coverage` exits 0.
- `.guardrails/scripts/check-units.sh`, `check-ids.sh` and `check-trace.sh` pass for every unit.

## Progress

- [x] T1
  - Results: `make coverage-branch` gave org-members 123/124 (floor 98), on-chain-client 34/34 (floor 99) and person 36/36 (floor 99), exit 0. With `PERSON_BRANCHES=100.5` it printed "person: branch coverage is below its floor of 100.5%" and exited 2. `make coverage` passed for all three units: org-members 96.55% lines and 94.57% regions; on-chain-client 41.75% lines and 42.60% regions; person 100% and 100%. `check-units.sh` exited 0, and `check-trace.sh` and `check-ids.sh` exited 0 for all five units.
  - Sandbox notes: the first runs used the floating `nightly`, which is the same build as the pin (manifest date 2026-10-03), because `~/.rustup` is read-only in the sandbox. Once the owner had installed `nightly-2026-10-03`, a plain `make coverage-branch` (the pin, rustc 1.101.0-nightly 0abfedbc7) gave the same three figures and exited 0. on-chain-client fetched its crates through `CARGO_HOME=/tmp/cargo_home_fuzz` because `~/.cargo` is read-only in the sandbox. With that override, a stable `make coverage-person` also counted an unexcluded proptest source file and fell below the floor; without it, the run reads 100% (a sandbox artefact, not a defect in the recipe).
- [x] Deslop pass: 15 findings (3 medium, 12 low, no high), all fixed. They were stale "unmeasured" statements in the ADR, the app hazard register and the on-chain-client config; the Makefile's Requires block (jq, the pinned nightly); the CI job (cargo-llvm-cov pinned at 0.9.0, the toolchain passed to make from one env value); one stated line figure (the stable 41.75%, not the nightly 42.68%); line-number pointers replaced by names; the denominator caveat kept once, in the Makefile; and this Progress entry split.
- [x] Review round 1 (last round, all low): findings 1–5 fixed — the uncovered org-members branch re-attributed to `calculate_delta`'s `!old.is_calculated()` operand (here, the Makefile, and a dated correction in S1's record); a CI step that fails if rust.yml's `BRANCH_TOOLCHAIN` differs from the Makefile pin; the ratchet-setup "unmeasured in all four units" line amended; PR-b795an (the `write` feature outside on-chain-client's coverage run) named in the ADR and the on-chain-client config.
