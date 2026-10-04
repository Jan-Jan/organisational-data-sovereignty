# Verification — recalculating a calculated trie is refused (2026-10-04)

branch: worktree-recalculate-refusal
reviewer: four rounds of independent subagent review at `merge-change` step 6a. Each round had a fresh reviewer, who was given the diff, the ledgers and the plan, but no author narrative. Each reviewer re-ran the org-members, org-node and app suites in its own nested worktree and measured its own mutations.
verdict: merge. Rounds 1–3 each raised a code or requirement finding, and each finding was fixed:
- round 1: the requirement wording, its trace, and a boundary mutant M2;
- round 2: the empty-organisation mutant M5;
- round 3: the one-member mutant M8.
Round 4 raised record findings only, so under the convergence rule it was the last round. Its findings were fixed and the gate re-run, but no reviewer has read those corrections. Round 4 killed every guard and variant mutant it tried, including one that skips the refusal for three-member tries.
reproduced: yes. Every test annotated `verifies: LLR-j35sxz` was watched failing before it held. `recalculate_refuses_a_calculated_trie` and `recalculate_succeeds_until_no_changes_are_pending` first failed to compile (E0599, no variant `HashesAlreadyCalculated`). With only the variant added, they failed with `unwrap_err()` on an `Ok` value, and they passed once the guard existed. `recalculate_accepts_a_mutated_trie_with_an_empty_change_set` failed under mutant M2 (guard `self.pending_changes()?.is_empty()`) with `Err(HashesAlreadyCalculated)`. The refusal test's empty-organisation case failed under M5, and its one-member case failed under M8, each with `unwrap_err()` on an `Ok` value.

Change: `OrgTrie::recalculate()` on a calculated trie returns the new `OrgMembersError::HashesAlreadyCalculated`. A trie is calculated when `genesis()`, `recalculate()` or `CandidateTrie::verify_against()` has returned it and it has not been mutated since. The check is `!has_pending_changes()` and runs before anything is computed. A trie mutated back to its original members still recalculates. Every caller that recalculated a calculated trie stops doing so. In production that is org-node `create_organisation`, which now uses the `genesis()` trie directly; the root it publishes is unchanged. The rest are test support and tests in org-members and org-node, and the org-members fuzz properties no longer discard the refusal with `if let Ok`.
Plan: `docs/plans/2026-10-03-recalculate-refusal.md`.

Units touched: `org-members` (error variant, guard, tests, the derived LLR-j35sxz and its assessment, the problem report) and `org-node` (`create_organisation`, test support, tests, a fuzz target). Impact set, from `check-units.sh --impact master..HEAD`: org-members touched, org-node touched, app dependent.

Base: `git fetch origin` succeeded on every round. `origin/master` was already an ancestor of the branch, and local `master` was merged at step 1. Master moved twice while this change was open, and both moves were merged in before any gate counted here: `4c66400`, then the re-squash of the same change as `0f85cb9`. The conflict in `service.rs` took master's side. Master's `first_admission_base` replaced the `receive_and_verify` fallback that PR-zqvs7t listed, and it does not recalculate the trie it builds. Every gate run counted here and all four reviews ran with `0f85cb9` merged.

## The gate

Final run (r6) on tree `35ac18a84a9d55ec2170063333290ce7ec25db3c`. That tree has every fix from the four review rounds, and only this record is added on top of it. Earlier runs gave the same counts in every unit:
- r2 on `a9dc241`, before the ledger files were finalized;
- r3 on `b4500d9`, after finalization;
- r4 on `36cc2eb`, after round 2's fixes;
- r5 on `05627fb`, after round 3's fixes.

The empty-organisation and one-member cases were added inside an existing test, so no count moved.

| Gate | Result |
| --- | --- |
| `check-units.sh` | exit 0. Output: `units: 4, disclaimed 6; tracked paths 455`. `--impact master..HEAD`: org-members touched, org-node touched, app dependent |
| org-members `cargo test -p org-members` | **197 passed, 0 failed**, 1 ignored by design (`preflight_probe`): integration 156, mbt_conformance 32, fuzz 9 |
| org-members quint 0.33.0 | typecheck of `membership_types`, `membership`, `membership_mbt`: exit 0. `quint test membership.qnt`: **63 passing**. `quint run membership_mbt --invariant=mbtInv`: no violation |
| org-members `make coverage-org-members` | lines **94.69%** (floor 92), regions **93.53%** (floor 91), functions 90.00%; `trie.rs` lines 98.48%. Decision coverage unmeasured |
| org-node `cargo test -p org-node --features app,test-support ...` | **57 passed, 0 failed**: lib 23, admission_sender 7, service_stories 3, store_at_rest 4, transport_handshake 3, transport_networked 1, verify_against_chain 13, wire_frame_bound 3. Three bolero targets each ran their 1 s budget without a panic |
| org-node quint 0.33.0 | typecheck of `protocol`, `ods_instances`: exit 0. `forkSafety`, `revocationSafety`, `revokedExcludedFromOrgSecret`, `tauWindow`, `convergence`: no violation |
| app cargo, 6 targets | **78 passed, 0 failed** |
| app `npm run check` / `npm run test` | `npm run check`: 194 files, **0 errors**, 1 pre-existing warning (`@types/node`). `npm run test` (vitest): **30 passed** |
| `check-ids.sh` | no `--allow-draft-files`: exit 0, no output, in all three units |
| `check-trace.sh` | exit 0 in all three units. Counts: org-members `REQ 13, HAZ 8, RC 10, SDD 6, LLR 40, PR 7`; org-node `REQ 20, HAZ 6, RC 9, PR 7`; app `REQ 22, HAZ 6, RC 12, PR 6`. The only notices are open problems and expectations that predate this change and are within their limits |
| Implements map / robustness | LLR-j35sxz. Normal case: `recalculate_succeeds_until_no_changes_are_pending` and `recalculate_accepts_a_mutated_trie_with_an_empty_change_set`. Abnormal input: `recalculate_refuses_a_calculated_trie`, covering `genesis()` with 0, 1 and 2 members, `recalculate()` and `verify_against()` |
| `check-review.sh` | exit 0 — `checked: records 22, for worktree-recalculate-refusal 1, findings 16; provenance checked` |
| Working tree | clean |

Environment: every quint run used a scratch `QUINT_HOME`, because `~/.quint` is deliberately read-only to agents. `CARGO_HOME=/tmp/cargo_home_fuzz`.

**Not run, by owner instruction:** `org-node/tests/chain_genesis_e2e.rs`. It is outside `verify_commands` because it needs chopsticks, and the owner said to ignore it for this change. Its two genesis sites were fixed by inspection, and the target compiles (`--no-run`).

**Coverage gap accepted by the owner for this merge (2026-10-03).** org-members meets its line and region floors, but its decision coverage is unmeasured. org-node and app have no `coverage_command`. Class C requires statement and decision coverage. The owner accepted the gap explicitly for this change, after review round 1, and it carries forward the gap recorded in `docs/plans/2026-09-05-ratchet-setup.md`.

## Red → green

| Item | Test | Watched red |
| --- | --- | --- |
| `LLR-j35sxz` | `recalculate_refuses_a_calculated_trie` (abnormal) | E0599, no variant; then `unwrap_err()` on `Ok` before the guard (T1, annotated LLR-n7nya3 then; re-annotated in T3) |
| `LLR-j35sxz` | `recalculate_succeeds_until_no_changes_are_pending` (normal) | E0599; then `unwrap_err()` on the recalculated trie got `Ok` (T1, re-annotated in T3) |
| `LLR-j35sxz` | `recalculate_accepts_a_mutated_trie_with_an_empty_change_set` (normal, boundary) | under M2: `Err(HashesAlreadyCalculated)` at `reverted.recalculate().unwrap()` (T3) |
| `LLR-j35sxz` | `recalculate_refuses_a_calculated_trie`, empty-organisation case | under M5 (refuse only when `member_count() > 0`): `unwrap_err()` on an `Ok` value at the `genesis(vec![])` line (T4) |
| `LLR-j35sxz` | `recalculate_refuses_a_calculated_trie`, one-member case | under M8 (refusal skipped when `member_count() == 1`): `unwrap_err()` on an `Ok` value at the one-member line (T5) |

T2 added no tests. The existing org-node tests were its red: with T1 merged and no org-node edit, the following failed with `HashesAlreadyCalculated`:
- `envelope::tests` (6) and `create_organisation_advances_mock_chain`;
- `admission_sender` 3 of 3 and `service_stories` 3 of 3;
- `verify_against_chain` 10 of 13 and `wire_frame_bound` 2 of 3;
- one `transport_handshake` test and the `transport_networked` test;
- the `fuzz_verify_against_chain` harness.

All of them passed once the genesis recalculates were removed.

## What was wrong, and what was built

PR-zqvs7t: recalculating a trie that was already calculated succeeded, returning the same root and an empty change set. The owner's intent is that a node's hash is a write-once cell, so asking to fill the cells of a record whose cells are already filled is a caller error and is reported.

Owner rulings, 2026-10-03 and 2026-10-04:
- the call returns an error;
- the error is a new variant, `HashesAlreadyCalculated`, distinct from `HashesNotCalculated` and `InvariantViolated`;
- this is a change of its own;
- after review round 1, REQ-avmu3j does not ask for the refusal, so it is a separate derived LLR, LLR-j35sxz, with a hazard-impact assessment. LLR-n7nya3 stays as on master. The decision-coverage gap is accepted.
- after review round 3 (2026-10-04), the surviving one-member mutant is closed with a one-member example, not a property test.

Built:
- the variant and the guard;
- LLR-j35sxz (`org-members/docs/architecture/2026-10-04-recalculate-refusal.md`, `satisfies: derived`) and its assessment (`org-members/docs/risk/2026-10-04-recalculate-refusal.md`, `assesses: LLR-j35sxz`), which records an unavailability pathway that fails closed and no reduced hazard;
- three tests, the refusal test covering calculated tries of 0, 1 and 2 members from `genesis()` as well as `recalculate()` and `verify_against()` results;
- the caller fixes in both units.

REQ-avmu3j's wording was checked and is unchanged.

Problem-ledger delta: **resolves** PR-zqvs7t (org-members). Opens nothing.

## Review

**finding-1**: requirement — (round 1) The new sentence in LLR-n7nya3 refused `recalculate()` on "a trie with no pending changes", but the code tests `!has_pending_changes()` (`cached_root_hash.is_none()`): whether the trie was mutated since it was calculated, not whether its change set is empty. The two disagree for a trie mutated and then mutated back.
disposition: The refusal is now LLR-j35sxz, which defines "calculated" (returned by `genesis()`, `recalculate()` or `verify_against()` and not mutated since) and states that a mutated trie recalculates even when its change set is empty. LLR-n7nya3 is reverted to master.

**finding-2**: requirement — (round 1) The refusal traced to REQ-avmu3j through LLR-n7nya3's `satisfies:`, while the amendment itself said REQ-avmu3j does not cover it. That kept derived behaviour out of the derived-item risk assessment.
disposition: The owner ruled for a separate derived item. LLR-j35sxz is `satisfies: derived` and is assessed by `org-members/docs/risk/2026-10-04-recalculate-refusal.md` (`assesses: LLR-j35sxz`), which states the availability effect on the org-node production path.

**finding-3**: code — (round 1) The tests annotated to the refusal did not pin where it starts: under M2 (guard `self.pending_changes()?.is_empty()`) both stayed green.
disposition: Added `recalculate_accepts_a_mutated_trie_with_an_empty_change_set` (verifies: LLR-j35sxz). It fails under M2 and passes on the real guard.

**finding-4**: record — (round 2) LLR-j35sxz said it refines SDD-d9svdj, but the refusal is in `OrgTrie::recalculate` in `src/trie.rs` (SDD-k5wa4n) and the variant is in `src/error.rs` (SDD-m9gs5g).
disposition: The LLR's prose now names SDD-k5wa4n, and SDD-m9gs5g for the variant.

**finding-5**: code — (round 2) None of the `verifies: LLR-j35sxz` tests covered an empty organisation. Mutation M5 (refuse only when `member_count > 0`) survived them, and only the fuzz property `trie_ops_never_panic_and_count_consistent`, which is not traced to LLR-j35sxz, killed it.
disposition: `recalculate_refuses_a_calculated_trie` now also refuses `genesis(vec![])`. It fails under M5 and passes on the real guard (T4).

**finding-6**: record — (round 2) The risk assessment called the refusal an unavailability pathway without tying it to HAZ-8suua9 or noting that it meets RC-c4truv.
disposition: It now names HAZ-8suua9 and RC-c4truv, and states why the probability is unchanged: no production caller remains, and a missed one fails closed with a typed error.

**finding-7**: record — (round 2) The risk assessment said PR-zqvs7t lists every caller, while PR-zqvs7t's own resolution says its lists missed two.
disposition: It now cites the T1 and T2 red -> green records in the plan for the full set.

**finding-8**: record — (round 2) The problem report said "Resolved 2026-10-03", before the resolving item was written.
disposition: Now 2026-10-04, the date of the finalized ledger files and of the squash.

**finding-9**: record — (round 2) The plan header read as though LLR-n7nya3 had been amended.
disposition: It now says the amendment was planned and then withdrawn, and that LLR-n7nya3 is unchanged from master.

**finding-10**: record — (round 3) The risk assessment called the refusal "the pathway of HAZ-8suua9", but that hazard's cause is malformed, hostile or oversized input. The register already rules that a lawful caller being refused is not HAZ-8suua9 (PR-vf5hdm analysis).
disposition: The assessment now says no register hazard has this cause, cites the PR-vf5hdm ruling, says that the refusal shares only HAZ-8suua9's harm, and keeps the RC-c4truv sentence.

**finding-11**: record — (round 3) The risk file said the full caller set was the T1/T2 red -> green record, but the `chain_genesis_e2e` sites were fixed by inspection and three fuzz properties changed without going red.
disposition: The caller set is now listed explicitly: the PR-zqvs7t sites, the two it missed, and the e2e sites, which were compiled but not run. The fuzz changes are named as changed without going red.

**finding-12**: record — (round 3) The LLR file and the risk file mixed "record" and "trie" for the same thing.
disposition: Both files now say "trie" throughout and define it once as the `OrgTrie` that holds a Membership record. "Membership record" is used only where the record itself is meant.

**finding-13**: code — (round 3) Mutant M8 (refusal skipped for one-member tries) survived the LLR-j35sxz tests. Only the fuzz property traced to REQ-ds8ryr / LLR-h9gs32 killed it.
disposition: By owner ruling 5, closed with an example rather than a property test. `recalculate_refuses_a_calculated_trie` now also refuses a one-member `genesis`. The new case fails under M8 and passes on the real guard (T5).

**finding-14**: record — (round 4) The risk file said every caller "was removed in this change", including the PR-zqvs7t sites, but the `receive_and_verify` fallback was removed by master's `0f85cb9`, as the file itself said later on.
disposition: The file now says the change removed the callers that remained after `0f85cb9`, and names the fallback as `0f85cb9`'s. Only `create_organisation` is now counted as a production caller removed here.

**finding-15**: record — (round 4) The plan's T2 org-node result (53 passed, two bolero targets) predated the master merge, which added tests and `fuzz_first_admission_base`.
disposition: The plan now marks those as T2-time figures and points to this record for the merged tree's figures: 57 passed, three bolero targets.

**finding-16**: record — (round 4) The risk file cited the PR-vf5hdm ruling as if it applied directly. It gave no reason why the new unavailability pathway owes no rating, although that analysis says a pathway no HAZ covers is what `analyze-risks` has to rate. It also said "lawful caller" where the LLR says "caller error".
disposition: The file now cites the ruling by analogy and says "caller error". It states why no rating is owed: no correct input reaches the pathway, no caller remains, and a future caller error is found deterministically in testing. It marks this as the author's assessment, which the owner confirms by signing.

## Gaps

- **`chain_genesis_e2e` not run**, by owner instruction. Fixed by inspection; compiles.
- **Decision coverage unmeasured**; org-node and app coverage unmeasured. Accepted by the owner for this merge.
- **A missed caller would fail closed.** It would get an error and change no Membership record. The author and all four reviewers searched the workspace, and none remains.
- **Round 4's corrections were not reviewed**, under the convergence rule.
- **Size-specific mutants are closed by examples, not a property.** Owner ruling 5 chose examples at 0, 1 and 2 members, and 3 through the recalculated trie. A mutant that skips another size is killed by fuzz `trie_ops_never_panic_and_count_consistent`, which is traced to REQ-ds8ryr / LLR-h9gs32, not to LLR-j35sxz.
- **Two survivors with no test to reach them.** Round 3's M7 moves the guard after the hash computation. It is equivalent from outside: the computation fills nothing on a calculated trie, and `recalculate()` takes `&self`. M9 changes the variant's Display text. No requirement specifies that text.
- **Toolkit gap (observed by the round 4 reviewer, not this change's).** `check-trace.sh` stayed green with the `assesses: LLR-j35sxz` line removed. `UNANALYZED-DERIVED` accepts a bare mention of the ID in an RMF file as an assessment, which the check-traceability skill says it must not. That belongs in the guardrails toolkit's own ledger.
- **Round 3's reviewer reported org-node at 51 tests** (admission_sender 3, transport_handshake 1). Every gate run and the round 2 and round 4 reviewers counted 57 from the cargo output. 51 matches the stale count in the config comment, so it reads as a misreport.
