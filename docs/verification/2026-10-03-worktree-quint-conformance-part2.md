# Verification — quint conformance part 2, org-node (2026-10-03)

branch: worktree-quint-conformance-part2
reviewer: independent subagent reviewer, one round (fresh agent in its own nested worktree, given the diff, plan, ADR and unit configs, no implementation narrative; it re-ran every unit's verify_commands itself and ran two negative controls)
verdict: PASS. No code or requirement findings; two record findings in the moved README, fixed. That round was the last review round.
reproduced: not applicable — no defect is fixed. The import-path move was shown red before the fix (QNT013/QNT405 on `org-node/quint/protocol.qnt`), and the reviewer's negative controls showed the new gate entries fail on a broken model.

Change: `protocol.qnt`, `ods_instances.qnt` and their README move from the top-level `quint/` into `org-node/quint/`; org-node's `verify_commands` run `quint --version`, both typechecks and the five simulator invariants with explicit `--max-steps=16`; org-members drops its `protocol.qnt` typecheck (org-node is a dependent in every org-members impact set); `quint` leaves `not_a_unit`; CI paths repointed. Branched from `master` at `2405ede` (part 1's squash); `master` and `origin/master` fetched and merged each round — no newer commits.
Plan: `docs/plans/2026-10-03-quint-conformance-part2.md`.
ADR: `docs/adr/2026-10-03-quint-conformance-gate.md` (decisions 1, 3, 13; dated note added).

Units touched: org-members, org-node, app, on-chain-client (`check-units.sh --impact master..HEAD`: all four `touched`, because `.guardrails/units.yaml` changed). Impact set: org-members, org-node, app, on-chain-client.

## The gate

Final round (part 2 gate round 2), HEAD `8c758cd`, tree `5652bc12fab71992db6837ec8c8e42995ff5ac38`, quint 0.33.0. Counts read from output.

| Gate | Result |
| --- | --- |
| org-node `cargo test -p org-node --features app,test-support …` | 53 passed, 0 failed; 2 bolero fuzz targets ran clean |
| org-node `quint --version` | 0.33.0 |
| org-node `quint typecheck` protocol.qnt / ods_instances.qnt | ok |
| org-node `quint run org-node/quint/protocol.qnt --max-steps=16 --max-samples=5000` × forkSafety, revocationSafety, revokedExcludedFromOrgSecret, tauWindow, convergence | each "[ok] No violation found" |
| org-members `cargo test -p org-members` | 150 passed, 0 failed, 1 ignored (`preflight_probe`, by design) |
| org-members quint version, three typechecks, `quint test` (24 passing), mbtInv run | all ok |
| on-chain-client `cargo test …` | 73 passed, 0 failed; 3 fuzz targets ran clean |
| app `cargo test …` / `npm run check` / `npm run test` | 78 passed / 0 errors (1 pre-existing warning) / 30 passed |
| `check-ids.sh`, all four units | clean |
| `check-trace.sh`, all four units | exit 0; only informational UNRESOLVED-PR / UNMET-EXPECTATION lines within limits |
| `check-units.sh` | clean (4 units, 6 disclaimed, 436 tracked paths) |
| Coverage | no `src/` change (`git diff --stat master...HEAD -- '*/src/*'` empty); org-node has no `coverage_command` (pre-existing gap, ratchet-setup plan) |
| Working tree | clean |

Problem-ledger delta: none resolved, none opened.

## Red → green

This change implements no requirement ID, so there is no per-ID row. The evidence that the gate entries can fail:

| Item | Check | Watched red |
| --- | --- | --- |
| import path (T1) | `quint typecheck org-node/quint/protocol.qnt` | red after the move, before the import fix: QNT013 "could not load '../org-members/quint/membership_types'", QNT405; green after |
| simulator gate (review) | `quint run … --invariant=forkSafety` | red with `and dm.result == chain.root` dropped from `deviceFetchAndApply` ("Invariant violated"); typechecks stayed green; reverted |
| model gate (review) | all seven Quint model commands | red with the import broken (QNT013/QNT405); reverted |

## What was wrong, and what was built

ADR decision 1 placed each model in the unit that implements it; part 1 moved the membership model into org-members, and the protocol model stayed in a disclaimed top-level `quint/`, outside every unit's gate and traceability. Part 2 moves it into org-node and gates it there. Measured cost: the five simulator runs take about 0.1–1 s each on quint 0.33.0's rust backend (ADR decision 3's ~60 s was quint 0.32.0 with the typescript backend; noted in the ADR). The reviewer confirmed with `check-units.sh --impact` on an org-members-only master commit that org-node is listed as a dependent, so dropping org-members' `protocol.qnt` typecheck loses no coverage.

## Review

**finding-1**: record — org-node/quint/README.md:48: the documented negative control names `memberFetchAndApply`, but the action is `deviceFetchAndApply` (org-node/quint/protocol.qnt:129). Pre-existing text carried into the moved README.
disposition: README corrected to `deviceFetchAndApply`, with a note that the control was re-measured on 2026-10-03 (red on `forkSafety`, per the review).

**finding-2**: record — org-node/quint/README.md:41: "append `--backend=typescript` locally; CI uses defaults" conflicts with the gate, which runs the simulator on the default rust backend.
disposition: README now says the commands run on the default rust backend, as org-node's `verify_commands` and CI run them, and points to the config for the full list.

## Gaps

- org-node still has no quint-connect conformance test (ADR decision 13): a Rust change to org-node is not checked against a model. The node-level model is a separate change.
- `ods_instances.qnt`'s vacuity witnesses are typechecked but not run in the gate (they are expected to find counterexamples; no negated runner exists).
- org-node has no `coverage_command` (pre-existing, tracked in the ratchet-setup plan).
- Local runs needed a writable `~/.quint` substitute (the scratchpad wrapper).
