# Verification — quint-connect-coupling part 1 (2026-10-03)

branch: worktree-quint-connect-coupling
reviewer: independent subagent reviewers, three rounds (each a fresh agent in its own nested worktree, given the diff, plan, ADR and ledger excerpts, no implementation narrative; each re-ran every unit's verify_commands itself)
verdict: PASS. Round 1 and round 2 raised code/requirement findings (fixed); round 3 raised record findings only and was the last review round. No open finding.
reproduced: yes — PR-vf5hdm reproduced before the fix as failing named scenarios (`scenario_handover_a_to_b`, `scenario_handle_swap`, `scenario_handover_to_new_member`: "Specification and implementation states diverge", membership model accepts, crate returns DuplicateHandle mapped to "ConfusableHandle"), and as random-run divergences on seeds 0xa68bbc85 and 0x7dd619a6 (task T5).

Change: the org-members Quint membership model moves into `org-members/quint/`, exports `membership_types.qnt` to `quint/protocol.qnt`, and becomes part of org-members' gate through quint-connect conformance tests (one model action per public trie operation, enforced); PR-vf5hdm is fixed (`apply_delta` releases outgoing handles before checking incoming ones); MSRV 1.85 declared. Branched from `master` at `343fd64`; `master` merged in at `27dab89` and `f635acc` (local master; `origin/master` is an ancestor of it and was fetched each round — no newer commits).
Plan: `docs/plans/2026-10-03-quint-conformance-part1.md`.
ADR: `docs/adr/2026-10-03-quint-conformance-gate.md`.

Units touched: org-members, org-node, app, on-chain-client (`check-units.sh --impact master..HEAD`: all four `touched`; the root `.guardrails/units.yaml` changed). Impact set: org-members, org-node, app, on-chain-client.

## The gate

Final round (gate round 7), HEAD `d2e5588`, tree `9164f6287c7015eedbb958542482649cca089a0e`, quint 0.33.0. Every figure read from command output, not exit codes.

| Gate | Result |
| --- | --- |
| org-members `cargo test -p org-members` | 150 passed, 0 failed, 1 ignored (`preflight_probe`, run as a subprocess by `quint_preflight_names_a_missing_binary`): fuzz_tests 8, integration_test 118, mbt_conformance 24 |
| org-members `quint --version` | 0.33.0 (recorded, not pinned — ADR decision 7) |
| org-members `quint typecheck` membership_types / membership / membership_mbt | ok |
| org-members `quint test membership.qnt --backend=typescript` | 24 passing |
| org-members `quint run membership_mbt.qnt --invariant=mbtInv --max-steps=15 --max-samples=1000` | no violation (seed 0xa30b90aceccdd6c9) |
| org-members / org-node `quint typecheck quint/protocol.qnt` | ok |
| org-node `cargo test -p org-node --features app,test-support …` | 53 passed, 0 failed; 2 bolero fuzz targets ran clean |
| on-chain-client `cargo test …` | 73 passed, 0 failed; 3 fuzz targets ran clean |
| app `cargo test --manifest-path app/src-tauri/Cargo.toml …` | 78 passed, 0 failed |
| app `npm --prefix app run check` | 0 errors, 1 pre-existing warning (tsconfig `@types/node`) |
| app `npm --prefix app run test` | 30 passed |
| `check-ids.sh` (no draft allowance), all four units | clean |
| `check-trace.sh`, all four units | exit 0; only informational UNRESOLVED-PR / UNMET-EXPECTATION lines within limits |
| `check-units.sh` | clean (4 units, 7 disclaimed, 434 tracked paths) |
| Coverage (`make coverage-org-members`, gate round 2) | lines 94.11% (floor 92), regions 92.93% (floor 91); decision coverage **not measured** — owner accepted the documented gap 2026-10-03 (class C target; tracked in the ratchet-setup plan) |
| MSRV 1.85 | `rust-version = "1.85"` declared in the workspace and `app/src-tauri/Cargo.toml`; no 1.85 build run — owner ruled the declarations sufficient (2026-10-03) |
| Working tree | clean |

Problem-ledger delta: resolves PR-vf5hdm. Opens none. (PR-zz4exm and PR-hvg2dy were resolved by a separate change already on master.)

## Red → green

From the dispatch reports, as recorded beside each task in the plan.

| Item | Test | Watched red |
| --- | --- | --- |
| REQ-wx3wpv, LLR-n5t6bn | `scenario_handover_a_to_b`, `scenario_handle_swap`, `scenario_handover_to_new_member` | red before the fix (divergence: model accepts, crate DuplicateHandle); green after the two-phase loop |
| REQ-wx3wpv, LLR-n5t6bn | `scenario_handover_b_to_a` | green under the pre-fix loop (safe identifier order); red with the release loop emptied (review round 3, reproduced) |
| LLR-n5t6bn, LLR-juxk9q | `apply_delta_rejects_two_upserts_claiming_one_handle`, `apply_delta_rejects_upsert_taking_handle_of_untouched_member`, `apply_delta_never_admits_a_handle_collision` | red with the post-release check skipped, and with only apply_delta's DuplicateHandle return deleted |
| LLR-n5t6bn | `apply_delta_rejects_confusable_of_untouched_member_after_release` | red with the post-release check skipped and with the release loop freeing incoming handles |
| LLR-4n8zqx | `irregular_history_reaches_same_root_as_direct_build` | red under three src mutations (remove leaves a tombstone; insert mixes the replaced leaf; diff drops removals) while the normal-path determinism tests stayed green |
| LLR-4n8zqx | `membership_conformance` | red 5/5 under an order-dependent leaf hash |
| LLR-ch2pkw | `membership_conformance` | red 5/5 with genesis DuplicateId deleted, and with both handle-collision returns deleted; measured negative for ConfusableHandle alone (see Gaps) |
| LLR-au8het | `scenario_apply_stale` | red with DeltaBaseMismatch deleted |
| LLR-7tdqv9 | `scenario_apply_verify_fail`, `scenario_built_verify_fail` | red with `verify_against` always Ok |
| LLR-xmpqn2 | `scenario_remove_upsert_overlap`, `scenario_no_op_before_overlap`, `membership_conformance` | red with the overlap check, resp. the no-op check, deleted |
| LLR-juxk9q | `membership_conformance` | red 5/5 with the handle-check block deleted and with DuplicateHandle alone deleted |
| LLR-s97ywt, LLR-w92psx | `membership_conformance` | red 5/5 seeds each with the P2pKeyNotReplaced guard removed from `delete_p2p_device`, resp. `emergency_isolate_member` (after the base merge) |
| model ordering (no ID) | `scenario_genesis_device_error_first`, `scenario_device_slots_full`, `scenario_no_op_before_overlap` | red against the pre-change model (DuplicateId vs EmptyDeviceList; DuplicateDevice vs DeviceSlotsFull; RemoveUpsertOverlap vs NoOpUpsert) |
| gate tests (no ID) | `trie_operations_table_matches_source`, coverage gate in `membership_conformance`, `quint_preflight_names_a_missing_binary` | each red under its seeded fault (row deleted; ApplyDelta steps removed; assertion inverted) |

## What was wrong, and what was built

PR-vf5hdm: a change set produced by `recalculate()` that moved a handle between two members (or to a member admitted by the same delta) was refused by a receiver holding the exact base it was computed against, whenever the receiving member's identifier sorted before the releasing member's — `apply_delta` checked uniqueness per member while iterating, before the releasing member's old handle had been freed. Producer and receiver then diverge with no adversary. Fix: a first loop releases every outgoing handle of every upserted member from both indexes, then the second loop checks and inserts. Owner decision: handovers stay lawful (REQ-wx3wpv, derived; HAZ-y8h835 amended).

The membership model and its conformance test were restructured so the model tracks `lastAction`, starts from genesis, covers the full device range and the `ApplyDelta` action with one-to-one error tags; three model orderings were corrected to match the crate. The gate now runs the Quint typecheck, unit tests, simulator and conformance test for every org-members change, fails closed when `quint` is unavailable, and enforces that every public trie-yielding operation has a model action. Random-run outcome coverage: 20/20 runs reach every observed (action, outcome) pair (`docs/verification/notes-quint-conformance-part1-coverage.md`).

## Review

Round 1 (code/requirement findings → fixed, re-reviewed):

**finding-1**: requirement — four rejection tests carried `verifies: REQ-wx3wpv`, an acceptance-only requirement; with the fix reverted three of them stayed green.
disposition: REQ-wx3wpv removed from those four; they verify LLR-n5t6bn (post-release check) and LLR-juxk9q. REQ-wx3wpv is carried by the handover scenarios and transitively through LLR-n5t6bn.

**finding-2**: record — integration_test.rs comment claimed the DuplicateHandle half of LLR-juxk9q was carried by a test not annotated for it.
disposition: `verifies: LLR-juxk9q` added to the three tests after measuring each red with only DuplicateHandle deleted; comment updated.

**finding-3**: record — architecture and risk files named the random conformance run as a carrier of LLR-n5t6bn/REQ-wx3wpv though it is unannotated and probabilistic (red 2 of 5 with the fix reverted).
disposition: reworded; the named scenarios and rejection tests are the carriers, the random run is supporting evidence only.

**finding-4**: record — glossary drift (avoided terms in the README and risk file; the "one action, one operation" definition was untrue for ApplyDelta).
disposition: terms replaced; CONTEXT.md model-action definition amended.

**finding-5**: record — ADR decision 10 claimed built deltas reach DeviceSlotsFull.
disposition: dated amendment note in the ADR.

**finding-6**: code — `rejected_operations_leave_no_trace_in_root` asserted only one rejection's effect directly.
disposition: rewritten to assert every rejection (red under insert_leaf and update_leaf DuplicateHandle deletion); later replaced, see finding-7.

Round 2 (one requirement finding → fixed, re-reviewed):

**finding-7**: requirement — LLR-4n8zqx's abnormal-input carrier could not fail: a rejected `&self` operation cannot change the cached root.
disposition: replaced by `irregular_history_reaches_same_root_as_direct_build`, red under three mutations that leave the normal-path determinism tests green.

**finding-8**: record — 2026-09-17 risk file still stated two gaps this change closes, with no forward note.
disposition: dated forward notes added; original text kept.

**finding-9**: record — HAZ-y8h835 amendment and REQ-wx3wpv assessment omitted the handover-to-admitted-member shape.
disposition: third shape added in place (its identifier-order condition is reasoned from the pre-fix loop, not separately measured).

**finding-10**: record — ADR decisions 9, 11 and the term "spec" disagreed with the code and glossary.
disposition: dated amendment notes; "spec" replaced by the glossary term.

Round 3 (record findings only → last review round):

**finding-11**: record — ADR 8(i) understated what the operation-table test scans; 4(d) and 12 named macros the code does not use.
disposition: dated amendment notes stating the four return patterns over src/trie.rs and src/delta.rs, and `runner::run_test` for scenarios and the random run.

**finding-12**: record — quint/README.md described protocol.qnt as built over the membership model and used an avoided term.
disposition: sentence corrected (it imports only `membership_types.qnt`); term replaced.

**finding-13**: record — coverage notes predated the base merge (no P2pKeyNotReplaced rows) and named no change.
disposition: 20-run re-tally after the base merge appended (both new pairs 20/20); header sentence names this branch and this record.

**finding-14**: record — risk table recorded `scenario_handover_b_to_a` only as green, though it carries a `verifies:`.
disposition: row added — red with the release loop emptied (measured by the reviewer, reproduced).

## Gaps

- Decision coverage is not measured for org-members (class C asks for it); owner-accepted 2026-10-03.
- MSRV 1.85 is declared, not built: no Rust 1.85 run happened; the CI `msrv` job exists but was not run for this record. Owner-accepted.
- LLR-juxk9q's confusability half and LLR-ch2pkw's ConfusableHandle clause are outside the membership model (skeleton == handle); deleting only those returns stays green in the conformance test. They are carried by the integration tests only.
- LLR-xmpqn2's ordering clauses are a declared abstraction boundary (`delta_canonicality_fuzz`), not modelled.
- `AddDevice → DeviceSlotsFull` is reached in 0 of 20 random runs; carried only by `scenario_device_slots_full`.
- The coverage figures above are from gate round 2; later rounds changed tests and docs but not `src/` beyond the base merge, and coverage was not re-run.
- The conformance driver's `commit` silently drops `recalculate`/`root_hash` errors; deferred by the owner to a separate change.
- Local runs needed a writable `~/.quint` substitute (quint 0.33.0 fetches rust-evaluator v0.7.0); in the sandbox the tests ran through a wrapper.
