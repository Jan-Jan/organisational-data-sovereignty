# Verification — the conformance driver's dropped errors (2026-10-03)

branch: worktree-dropped-errors
reviewer: five rounds of independent subagent review at `merge-change` step 6a, a fresh reviewer each round. Each was given the diff, the ledgers and the related items, and no author narrative. Each re-ran org-members' verify_commands in its own nested worktree and re-measured the mutations itself.
verdict: merge. Every round found the driver fix correct. Round 5 approved with two record findings only, so it was the last round. Rounds 1–3 converged on a requirements question that the owner settled (finding-3, finding-6, finding-8). Round 4 found stale driver texts and inaccurate call-site lists. All findings are fixed below.
reproduced: no failing test can reproduce PR-499dzp, because the defect hid failures rather than causing them. It was measured with three temporary crate mutations (M1–M3, recipes in `org-members/docs/problems/2026-10-03-conformance-drops.md`) on master's driver and on the fixed driver. Rounds 1, 2, 4 and 5 re-measured them independently.

Change: `MembershipDriver::commit` (`org-members/tests/mbt_conformance.rs`) now recalculates only a trie with pending changes. A `recalculate` or `root_hash` error fails the step, naming the call, where before it was swallowed or skipped the round-trip check. Branched from `master` at `2405ede`.
Plan: none — a single-task change from a handoff. Its scope is PR-499dzp.

Units touched: `org-members` (test driver, problem ledger). Impact set, computed with `check-units.sh --impact master..HEAD`: `org-members` touched, `org-node` dependent, `app` dependent.

Base: `git fetch origin` succeeded every round. `origin/master` is an ancestor of local `master`, because this repository integrates locally, so the base merged each round was local `master`. Before round 4, master moved by the quint conformance part 2 merge (`5850ed7`), and that merge was clean.

A first squash attempt went wrong. Master moved to `a547ff3` (every key held once) after the final round's merge reported "Already up to date" and before the squash. The squash was therefore staged on top of a tree that was never tested, and an earlier draft of this record wrongly said `a547ff3` had been merged. `finish-merge.sh` caught it ("HEAD and worktree-dropped-errors differ") before removing anything. The unpushed squash commit was reset off master with the owner's approval. `a547ff3` was then merged into the branch, cleanly. PR-zqvs7t's org-node line numbers were updated for the lines `a547ff3` moved. M1 and M2 were re-measured on the combined tree (same outcomes as the table), and the whole gate below was re-run on it.

Problem-ledger delta: resolves PR-499dzp; opens PR-zqvs7t.

## The gate

Final run on tree `cb41c5c94854843e710e5dfeb2d49d2bf3311db3`, after `a547ff3` was merged in and before this record was corrected. Rounds 1–5 ran on trees without `a547ff3`, with 150 org-members tests (118 integration, 24 conformance) and 24 quint tests. The higher counts below come from `a547ff3`'s own tests.

| Gate | Result |
| --- | --- |
| `check-units.sh` (repository-level) | exit 0 — `units: 4, disclaimed 6; tracked paths 445` |
| org-members `cargo test -p org-members` | **194 passed, 0 failed, 1 ignored** — fuzz_tests 9, integration_test 153, mbt_conformance 32 (+1 ignored by design, `preflight_probe`) |
| org-members `quint --version` | 0.33.0 |
| org-members `quint typecheck` on `membership_types.qnt`, `membership.qnt` and `membership_mbt.qnt` | clean |
| org-members `quint test membership.qnt --backend=typescript` | **63 passing** |
| org-members `quint run membership_mbt.qnt --invariant=mbtInv --max-steps=15 --max-samples=1000` | `[ok] No violation found` |
| org-node `cargo test -p org-node --features app,test-support ...` (10 targets) | **53 passed, 0 failed**; both fuzz targets ran their 1 s budget without failure |
| org-node `quint typecheck` on `protocol.qnt` and `ods_instances.qnt`, and five `quint run` invariants | clean, and all five `[ok] No violation found` |
| app `cargo test ... --features test-support` (6 targets) | **78 passed, 0 failed** |
| app `npm --prefix app run check` | 194 files, **0 errors**, 1 pre-existing warning (no `@types/node`) |
| app `npm --prefix app run test` | vitest **30 passed**, 4 files |
| `check-ids.sh` (bare, per unit) | exit 0, silent, in all three units |
| `check-trace.sh` (per unit) | exit 0, warnings only. org-members `REQ 13, HAZ 8, RC 10, SDD 6, LLR 39, PR 7`; org-node `REQ 19, HAZ 6, RC 9, PR 7`; app `REQ 22, HAZ 6, RC 12, PR 6` |
| Coverage, against the class C target | org-members lines **94.67%** (floor 92), regions **93.51%** (floor 91), functions 90.00%; decision coverage **unmeasured**; org-node and app unmeasured — gap accepted, below |
| Working tree | clean |

Environment: in the agent sandbox `$HOME` and `~/.cargo` are read-only. Every command therefore ran through a wrapper that sets `HOME` to a scratch directory (quint's evaluator cache), sets `CARGO_HOME=/tmp/cargo_home_fuzz`, and puts a `quint` that execs node on quint's `cli.js` first on `PATH`. The app's npm checks ran after `npm ci` from `app/package-lock.json` in the change worktree, because the primary checkout's install lacks `vitest` and `@types/node`.

**Coverage, accepted gap.** org-members meets its line and region floors, but decision coverage is unmeasured (llvm-cov reports Branches `0/0`), and org-node and app have no `coverage_command`. These fall short of the class C target and were already recorded in `docs/plans/2026-09-05-ratchet-setup.md`. The owner accepted them as a documented gap for this merge on 2026-10-03. This change does not touch `org-members/src`; the figures moved only because `a547ff3` did.

## Red → green

This change implements no REQ or LLR. It fixes the test oracle, so there is no `verifies:` test to watch go red. The row below records the mutation evidence instead.

| Item | Test | Watched red |
| --- | --- | --- |
| PR-499dzp | `membership_conformance`, `scenario_add_member` under M2 (`recalculate` always errs) | Before the fix, red one stage late in the root-class check, with "root_hash: HashesNotCalculated", naming neither step nor call. After the fix, red in `commit`: "recalculate after an accepted mutation: InvariantViolated". Under M1, the refusal the owner wants, both drivers are green: master's because it swallowed the error, the fixed one because it no longer recalculates a calculated trie. |

## What was wrong, and what was built

`commit` swallowed a `recalculate` error (`Err(_) => t`), and ran the delta round-trip check only when both `root_hash` calls succeeded. Neither error is lawful. Measured, no case hid a crate defect. The only silent case, M1 (refusing to recalculate a calculated trie), is the behaviour the owner wants. M2 and M3 were caught one stage later, but only because of the order the checks run in, and with a message that names nothing. The fix fails the step where the error happens and names the call. The driver also stopped recalculating an `ApplyDelta` result, which is already calculated, so it works both before and after the crate starts refusing that call.

That refusal is owner-ruled and deferred to its own change (PR-zqvs7t). It spans org-members and org-node: `org-node/src/service.rs` calls `genesis(..).recalculate()` in production.

## Review

**finding-1**: record — (round 1) the report said a failed `recalculate` dropped the round-trip check while staying green; under M1 the check still ran, and only the error itself was lost.
disposition: rewritten. The report now says no measured case lost the round-trip check and stayed green.

**finding-2**: record — (round 1) the report called the `commit` round trip "the only direct check of REQ-wx3wpv"; the honest `ApplyDelta` arm and the four handover scenarios also check it.
disposition: claim removed, and those checks are named.

**finding-3**: requirement — (round 1) the driver's panic assumed `recalculate` succeeds on an already-calculated trie, and no item stated that.
disposition: first answered with a no-op clause on LLR-n7nya3 and a test. Superseded by the owner's ruling at finding-8: the clause and test were reverted, the driver no longer relies on the assumption, and PR-zqvs7t records the intended refusal.

**finding-4**: record — (round 1) `affects:` omitted LLR-n7nya3, the item that governs `recalculate()` and the `root_hash` refusal.
disposition: added to PR-499dzp's `affects:`.

**finding-5**: record — (round 1) no permanent regression evidence, and the resolution cited a branch that will not survive the squash.
disposition: the branch reference was removed. The report states why no reproducing test exists, and it records the M1–M3 recipes so the measurement can be re-run (see also finding-15).

**finding-6**: requirement — (round 2) the new LLR-n7nya3 clause's link to REQ-avmu3j was unstated.
disposition: an amendment note was tried, then superseded by the owner's ruling (finding-8). The clause no longer exists.

**finding-7**: record — (round 2) the LLR-n7nya3 amendment carried no amendment note.
disposition: moot. The amendment was reverted, and the decomposition file is unchanged from master.

**finding-8**: requirement — (round 3) the amendment's rationale was false: `has_pending_changes()` is public, and `root_hash()` does not refuse a trie with nothing pending.
disposition: put to the owner under the derailment rule, after rounds 2 and 3 both raised the same item. Ruling, 2026-10-03: recalculating a calculated trie must be refused with a new `OrgMembersError::HashesAlreadyCalculated`, in a separate change. The clause and its test were reverted, PR-zqvs7t was opened, and the driver recalculates only a pending trie.

**finding-9**: record — (round 3) under M2 the honest `ApplyDelta` arm fails first, with a model divergence, not in the root-class check.
disposition: the M2 row now states both mechanisms.

**finding-10**: code — (round 3) the no-op test relied on `genesis` being calculated without asserting it.
disposition: the assert was added, then the test was removed with the clause (finding-8).

**finding-11**: record — (round 4) PR-zqvs7t listed `service_stories.rs` as a direct call site; it fails only through `service.rs`, and other indirectly affected tests went unnamed.
disposition: PR-zqvs7t now separates direct sites, with line numbers, from tests that fail through `service.rs` or `test_fixtures.rs`.

**finding-12**: record — (round 4) PR-zqvs7t missed `fuzz_tests.rs`, whose `Op::Recalculate` discards an `Err` and will silently skip its `member_count` check once the refusal lands.
disposition: added, together with the scope item "make the fuzz op stop discarding the refusal".

**finding-13**: record — (round 4) the M2 `membership_conformance` cells depend on the seed.
disposition: marked as such, with the reviewers' run counts.

**finding-14**: code — (round 4) the `BOUNDARY` text said "called by the driver after every change", and one panic said "root_hash after recalculate"; both had become inaccurate.
disposition: the BOUNDARY text now says "after every change that leaves hashes pending", and the panic now reads "root_hash of the accepted trie". Round 5 confirmed both.

**finding-15**: record — (round 4) the mutation evidence could not be re-run.
disposition: the exact M1–M3 lines are recorded in PR-499dzp.

**finding-16**: record — (round 5) the resolution said `scenario_*`, but only `scenario_add_member` and `scenario_apply_honest` were measured.
disposition: both tests are named, with a sentence saying that the mutation table substitutes for a reproducing test.

**finding-17**: record — (round 5) `service.rs:969-971` is not a first-admin path; it is `receive_and_verify`'s fallback that rebuilds a single-admin trie when there is no snapshot.
disposition: reworded in PR-zqvs7t.

## Gaps

- Decision coverage is unmeasured, and org-node and app have no coverage command (accepted above).
- PR-499dzp's evidence is a mutation measurement, not a committed test. The recipes are in the report, but no gate runs them.
- The refusal itself (PR-zqvs7t) is open. Until it lands, `recalculate` on a calculated trie still succeeds silently, and `fuzz_tests`' `Op::Recalculate` still discards errors.
