# Verification — guardrails-units (2026-09-05)

One record per change, written at `merge-change` step 6b and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line, so this file is the evidence that travels with the change.

branch: worktree-guardrails-units
reviewer: fresh general-purpose subagent, no chat history, own nested review worktree (`.worktrees/worktree-guardrails-units-review`), three rounds; the cargo and npm suites were run by a separate verification-gate subagent in the change worktree and the reviewer rested on that gate's summary for them, running every guardrails gate and both quint typechecks itself
verdict: PASS — round 1 PASS WITH FINDINGS (eight), round 2 PASS WITH FINDINGS (three, all wording), round 3 PASS with no new findings; every finding below carries its disposition
reproduced: not a defect fix — a toolkit upgrade and a layout conversion. What was reproduced is the false-green shape the change exists to remove: with the manifest present, a bare `check-trace.sh` at the root is exit 2 rather than a scan over nothing, and `new-id.sh` outside every unit is exit 2 rather than a guess (both measured on this tree). Also reproduced, unintentionally: the round-1 gate showed org-node's first verify command failing closed (exit 101, no tests ran) — the defect finding-1 names

Change: convert the repository to a multi-unit guardrails project — root
`.guardrails/units.yaml`, four unit configs (org-members, on-chain-client,
org-node, app), ledgers relocated into `org-members/docs/`, scripts upgraded to
upstream `e2eac86`, CI reshaped. Branched from `master` at `630fa0d`; base
merged in three times (`origin/master` = `2bb1c21`, strictly behind local
`master`, nothing to merge).
Plan: `docs/plans/2026-09-05-ratchet-gap-analysis.md`.
Decision: `docs/adr/2026-09-05-units-and-per-unit-classes.md`.

units touched: org-members, on-chain-client, org-node, app (a change under the
root `.guardrails/` maps to every unit).
impact set (`check-units.sh --impact master..HEAD`, exit 0):
org-members touched; on-chain-client touched; org-node touched; app touched.
Units run: all four, gates and `verify_commands`, three times (rounds 1–3).

## The gate

Every figure derived from the tree under test, `fdfc284`, working tree clean,
by the round-3 gate dispatch (logs kept outside the tree under the session
scratchpad, `verify-logs-3/`). Cargo ran with a scratch `CARGO_HOME`; nothing
was installed.

| Gate | Result |
| --- | --- |
| `check-units.sh` (repository-level) | exit 0 — units 4, disclaimed 7, tracked paths 357 |
| `check-units.sh --impact master..HEAD` | exit 0 — four units, all `touched` |
| org-members `check-trace.sh` | exit 0 — REQ 12, HAZ 8, RC 10, SDD 0, LLR 0, PR 2; UNRESOLVED-PR PR-zz4exm (5 days), PR-hvg2dy (3 days), both inside limits age 30 / open 10; scope: foreign 0, reverse 0 |
| org-members `check-ids.sh` | exit 0 |
| org-members `cargo test -p org-members` | PASS — 109 passed, 0 failed (fuzz_tests 6, integration_test 102, mbt_conformance 1) |
| org-members `quint typecheck` membership.qnt, protocol.qnt | PASS, PASS (exit 0; quint prints nothing on success) |
| on-chain-client `check-trace.sh` / `check-ids.sh` | exit 0 / exit 0 — all counts zero (README-only ledgers) |
| on-chain-client cargo `--lib` + three fuzz targets | PASS — 23 passed, 0 failed; fuzz targets ran to their 1 s default with no crash (278,488 / 42,166 / 257,371 inputs) |
| org-node `check-trace.sh` / `check-ids.sh` | exit 0 / exit 0 — all counts zero |
| org-node cargo `--features app,test-support --lib` + five named targets | PASS — 38 passed, 0 failed (lib 35, service_stories 1, transport_handshake 1, transport_networked 1); both fuzz targets to their 1 s default, no crash |
| org-node `quint typecheck quint/protocol.qnt` | PASS |
| app `check-trace.sh` / `check-ids.sh` | exit 0 / exit 0 — all counts zero |
| app `cargo test --manifest-path app/src-tauri/Cargo.toml` | PASS — 0 tests: a compile proof, the crate has no tests |
| app `npm --prefix app run check` | PASS — svelte-check: 166 files, 0 errors, 1 warning (tsconfig: no type definitions for `node`) |
| `finalize-docs.sh` ×4 (step 3) | exit 0 each; no `DRAFT-` file existed, nothing renamed, `git status` clean afterwards — so the step-2 and step-6 gate runs are over byte-identical trees and one run stands for both |
| `check-review.sh --branch worktree-guardrails-units` (6c) | exit 0 — records 6, for this branch 1, findings 11, none undisposed (run once this file held the eleven findings; the two rows below were then filled in and nothing else changed) |
| Coverage, against the class target | `make coverage-org-members` exit 0: 93.50% lines / 92.20% regions (floors 92 / 91); `make coverage-on-chain-client` exit 0: 53.57% lines / 59.19% regions (floors 52 / 58). Identical to the 2026-08-26 measurement, as expected for a change touching no source file. Statement half of the class C target only; decision coverage unmeasured |
| Working tree | clean at `fdfc284` (`git status --porcelain` empty before and after every run) |
| Tool qualification | upstream bats suite at `e2eac86`: 602 ok, 0 not ok, exit 0 (second run; the first run's exit was lost to a zsh `status` clash, its TAP output complete and all ok) |

Coverage: the two configured `coverage_command`s were run on this tree after
the gate, with the scratch `CARGO_HOME`; both floors hold (table above). Class
C asks for statement AND decision coverage; decision coverage is unmeasured for
every unit and org-node and app have no coverage command — carried as open
items in the setup checklist, not established here.

## Red → green

No item is implemented by this change: no REQ, RC, SDD or LLR was written or
modified, and no test was added or annotated. The table is therefore empty by
construction, not by omission.

| Item | Test | Watched red |
| --- | --- | --- |
| — | — | — |

## What was wrong, and what was built

Measured before the change: one root `.guardrails/config.yaml` governed a
repository whose four crates version, release and take risk separately; the
installed eight scripts were at upstream `bb7eee5` and every one differed from
upstream `e2eac86`, which introduces the unit machinery; `check-units.sh` did
not exist here; `guardrails_commit` was unset, so the tool-qualification basis
named a version string only.

Built: the manifest (four units, seven disclaimers each with its reason), a
config per unit (all class C, `guardrails_commit` set), README ledgers for the
three units that had none and per-unit `docs/CONTEXT.md` skeletons, the
existing ledgers relocated with history into `org-members/docs/` (32 items,
IDs and text unchanged — verified by the reviewer as an empty diff after path
normalisation), org-node and app under `strict_paths`/`test_paths` for the
first time, the CI `guardrails` job reshaped for a manifest repository and the
`test` job given org-node's suite, an ADR, a re-cut gap analysis and setup
checklist, and `.gitignore` entries for two local tool-output directories.

Measured on the result: a bare `check-trace.sh` or `check-ids.sh` at the root
exits 2 with a message naming the remedy; `new-id.sh` outside every unit exits
2, inside a unit or with `--unit` it mints. One thing was tried and reverted
inside the change: PR-hvg2dy (an org-node defect) moved to org-node's ledger
drew `UNDECLARED-DEPENDENCY` in both units' runs, because it cites org-members
items and the RMF cites it back; it stays in org-members' ledger until an edge
and org-node's own risk analysis exist. The boundary is real from the first
tooth.

## Review

Round 1 (on `993db46`), eight findings; round 2 (on `2c21815`), three; round 3
(on `fdfc284`), none. Findings are copied in the reviewer's terms.

**finding-1**: org-node's `verify_commands` names test targets whose `required-features` are not enabled (`service_stories` → `app`, `transport_handshake` → `transport`, `transport_networked` → `transport`,`test-support`); cargo refuses the command before compiling anything (exit 101). It fails closed, but the gap analysis' statement that the lists "are set to what the crates' own layout says should pass" is false, and the setup checklist misdiagnoses the risk.
disposition: the line gained `--features app,test-support`; the round-2 and round-3 gates measured 38 passed, 0 failed on it. Config comment, gap analysis and setup checklist rewritten to the measured facts. The command that reddens without the fix is the command itself: cargo exit 101, no tests run (round-1 gate log).

**finding-2**: This change's own impact set is all four units, so org-node's and app's `verify_commands` are due at THIS merge, not a later one; the gap analysis framed them as deferred.
disposition: both were run at this change's gate in rounds 2 and 3 (org-node 38/0; app cargo compile proof; `npm run check` 0 errors after `app/node_modules` was copied from the primary checkout — it is gitignored and a fresh worktree lacks it). Deferral wording removed; the setup checklist ticks both measurements and keeps the node_modules provisioning open.

**finding-3**: CI no longer mirrors `verify_commands`, and its header says it does — `rust.yml` cited a root config that no longer exists, and org-node's and app's commands ran nowhere in CI.
disposition: header rewritten for the per-unit configs; org-node's command added verbatim to the `test` job; app's two entries recorded as running nowhere in CI, with the two reasons (Tauri Linux system packages; `npm --prefix app ci`), in the workflow, the gap analysis and the setup checklist. Unexercised until the first push — an open checklist item.

**finding-4**: The per-unit CI loops `for u in $(check-units.sh --list)` can go green on a failed `--list`: `set -e` does not catch a failed command substitution in a for-list, so the body runs zero times and the step passes.
disposition: all three loops now `units=$(… --list) || exit 2` before iterating, with the reason as a comment. Verified by the round-2 reviewer at the three sites; YAML parses.

**finding-5**: Recorded measurements do not match the committed tree — `tracked paths 354` (tree: 357), "30 ID'd items" (tree: 32).
disposition: corrected to 357 and 32 (12 REQ, 8 HAZ, 10 RC, 2 PR); reviewer re-counted both.

**finding-6**: A minted, undefined ID token (`HAZ-` plus six characters) is recorded in prose in the gap analysis; the moment `docs` is claimed or the text quoted into a ledger it is a DANGLING-REF.
disposition: replaced by a description of the token; a tree-wide grep for it finds nothing.

**finding-7**: Gate scope narrowed by the manifest — `check-ids.sh`'s DRAFT/MALFORMED scans and DANGLING-REF's definition set are now unit-scoped, and "Nothing was loosened" did not say so.
disposition: recorded in the gap analysis under that paragraph, with the measured effect (none today: no definition-form line outside `org-members/docs/`, every unit `foreign 0, reverse 0`) and the design reference (D4/D7). No mechanical change: this is the toolkit's design, adopted knowingly.

**finding-8**: Stale root-config wording inherited from upstream templates — the four problems READMEs and the AGENTS.md managed block say the limits live in `.guardrails/config.yaml`.
disposition: no change here; the files are byte-identical to upstream's templates at `e2eac86` and the managed block is not edited by hand. Worth an upstream note; the round-2 reviewer confirmed AGENTS.md is untouched by the whole change.

**finding-9**: The org-node config comment claims `--lib` compiles the library "with the `app` feature set, which is the configuration org-node ships in", but the command also enables `test-support`, which Cargo.toml says is never enabled in production builds.
disposition: comment rewritten — `app` plus `test-support`, the latter test-only, so the lib under test differs from the shipped lib by that feature alone. Round-3 reviewer confirmed against Cargo.toml.

**finding-10**: The reason app's `npm run check` does not run in CI is stated inconsistently across the workflow header, the gap analysis and the setup checklist; the checklist wrongly said a node setup step was missing.
disposition: all four locations (the gap analysis has two) now give the same two reasons; the false "node setup" claim removed. Round-3 reviewer grepped every location.

**finding-11**: The setup item "Confirm the CI reshaping on the first push" names only the `guardrails` job and not the new org-node step in the `test` job.
disposition: item extended to name both, including that it is the first CI run of `transport_networked` on ubuntu-latest.

Also disclosed: two wording corrections the round-1 fix subagent flagged but
was not asked to make (a sentence in the setup checklist claiming two units
could not build in the sandbox; the feature flags missing from the gap
analysis' configuration table) were made by the author directly on the change
branch (`2c21815`), not through a dispatched fix. The round-2 reviewer saw and
passed them.

## Gaps

- **Decision coverage** is unmeasured for every unit; org-node and app have no
  `coverage_command` at all. Class C requires statement and decision coverage.
  Open in the setup checklist.
- **app's `verify_commands` run nowhere in CI**; the merge gate is app's only
  gate until Tauri's Linux packages and `npm --prefix app ci` are added.
- **The reshaped CI has not run**: no push happened in this change. The first
  push confirms the `guardrails` job's new shape and the org-node `test` step.
- **No dependency edge is declared.** The four edges the code has are the next
  tooth, each with its assessment. Until then a change to org-members does not
  run org-node's suite — the same coverage the root config gave, now visible in
  the manifest.
- **Hazards about org-node and on-chain-client live in org-members' RMF**, and
  PR-hvg2dy in org-members' problem ledger, until those units' own risk
  analyses exist (tooth 3).
- **The three README-only units pass vacuously**: their "clean" gates checked
  zero items. That is the honest state of a first tooth, not evidence.
- **Historical records cite the old `docs/…` ledger paths.** Left unedited by
  the project's convention; this record and the ADR name the move.
- **`quint typecheck` prints nothing on success**, so its PASS is an exit code
  only.
- **The svelte-check warning** (no type definitions for `node`) is a tsconfig
  matter, not code; it did not affect the exit code and was not fixed here.
