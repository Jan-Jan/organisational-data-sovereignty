# Verification — on-chain-client ledger finalize (2026-09-18)

branch: worktree-guardrails-on-chain-client-finalize
reviewer: four independent rounds, each a fresh dispatched subagent in its own nested worktree at `.worktrees/worktree-guardrails-on-chain-client-finalize-review{,2,3,4}`, given the diff, the artefacts and the whole repository but no implementation narrative and no chat history. Each round was told which findings its predecessors had raised only to the extent of "do not re-derive them"; none was given the reasoning behind any fix, so a mis-applied correction had to be found as a defect rather than recognised as a repair.
verdict: merge. Rounds 1, 2 and 3 each refused it, raising 3, 11 and 6 findings; round 4 raised 3, of which two were must-fix and one noted. All 23 are dispositioned below. Every finding across all four rounds was in the PROSE — not one touched the renames, the cross-references, the ledger integrity or any gate. Round 4's clearance is the substantive one: every factual claim the change makes was re-checked against its source, gate by gate and commit by commit, and the mechanical work was confirmed correct for the fourth time.
reproduced: yes — the defect directly, and the mechanism that hid it. On `master` at `ec66743`, `GR_CONFIG=on-chain-client/.guardrails/config.yaml .guardrails/scripts/check-ids.sh` exited 1 with three `DRAFT-FILE` convictions naming the three files; bare on this branch it exits 0, silent, on seven independent gate runs. The masking half was reproduced too: the same command WITH `--allow-draft-files` — the form `verify-before-merge` check 3 runs — exits 0 on the unrepaired tree, which is why the change that caused the defect could not have detected it.

Change: rename the three draft-named `on-chain-client` ledger files that `fbf17f0` merged onto the base branch as drafts, repair the cross-references, and record what was skipped and why it went undetected. Branched from `master` at `ec66743`.
Plan: none. This is a one-commit repair of a merge step that did not run; `plan-change` governs changes with tasks and a test cycle, and this has neither. The change it repairs was planned in `docs/plans/2026-09-10-on-chain-client-risk-analysis.md`, which this change amends.
Units touched: `on-chain-client` and `app`. Impact set, computed with `check-units.sh --impact master..HEAD` and not judged: `on-chain-client` touched, `app` touched, `org-node` dependent — all three gated. `app` entered the touched set at fix round 2, when a stale date citation in its hazard register was corrected; the set was recomputed rather than assumed, and `finalize-docs.sh` was then run for both touched units.

## The gate

Seventh and final run, on the tree under test at `3f926ae`. Every figure was measured on that tree; none is carried forward. The gate was dispatched seven times across the four review rounds, and an eighth attempt died mid-run on an authentication failure and was discarded rather than partially credited.

| Gate | Result |
| --- | --- |
| on-chain-client `verify_commands` | exit 0 — **72 passed, 0 failed, 0 ignored** over ten counted targets, plus 3 bolero targets exiting on their one-second budget without panicking |
| app `verify_commands` (3 entries) | exit 0 — `cargo test` **78 passed, 0 failed**; `npm run check` 194 files, **0 errors**, 1 pre-existing `@types/node` warning; `npm run test` **30 passed** over 4 files |
| org-node `verify_commands` (2 entries) | exit 0 — **51 passed, 0 failed**, plus 2 bolero targets; `quint typecheck quint/protocol.qnt` exit 0 |
| Suite total, impact set | **231 passed, 0 failed, 0 ignored, 0 skipped** |
| `check-ids.sh --allow-draft-files` × 3 units | exit 0, silent on all three |
| `check-ids.sh` **bare** × 3 units | exit 0, silent on all three — the change's whole objective, and the run `merge-change` step 4 makes. The flag now changes nothing on this tree |
| `check-trace.sh` × 3 units | exit 0 on all three. on-chain-client `REQ 21, HAZ 6, RC 9, SDD 0, LLR 0, PR 5`. Standing warnings within limits: 4 open PRs on on-chain-client (oldest 8 days, limit 30), 5 on org-node (oldest 16), 5 on app (oldest 4); 3 unmet expectations, oldest 12 days against a 90-day limit |
| `check-units.sh` (repository) | exit 0 — `units: 4, disclaimed 7; tracked paths 413` |
| `check-review.sh` on the two records this change edits | exit 0 for `worktree-guardrails-on-chain-client-risk` (**41 findings**) and `worktree-guardrails-org-members-arch` (**27 findings**) — both unchanged from before the edits, so neither dated correction note disturbed the record it was added to |
| Coverage, against the class C target | on-chain-client **41.75% lines** (205/491), **42.45% regions** (284/669). **A shortfall, not a target met.** Decision coverage **unmeasured** — Branches 0/0 on every file. app and org-node configure no `coverage_command` at all. See Gaps |
| Working tree | clean; `git status --porcelain` empty on every run |

**Exit 0 was not read as a pass.** Six commands report no counts and each was chased rather than assumed. The five bolero targets are `harness = false` and print iterations and an exit reason instead of a pass count, a panic being their failure signal; `quint typecheck` and `check-ids.sh` are silent by design, and the silence was verified to be discrimination rather than absence — an injected type error in a scratchpad copy of `protocol.qnt` printed `Error [QNT000]` and exited 1.

## Red → green

**Empty, and that is the correct content for this change.** It implements no requirement IDs and adds no tests. `git diff --stat master..HEAD` is nine files, every one Markdown, 621 insertions and 8 deletions; the diff adds zero `Implements:` lines and zero `verifies:` annotations, and touches nothing in any unit's `strict_paths` or `test_paths`. With no Implements IDs there is no red → green obligation to discharge, and no row is invented to suggest otherwise. The gate's Implements map is correspondingly empty and its class B/C robustness check is vacuous — there is no new behaviour to probe and no new test whose discrimination a mutation could measure.

This is the one shape of change for which an empty table is honest rather than a gap. Rows here would describe work that did not happen.

## What was wrong, and what was built

`merge-change` step 3 runs `finalize-docs.sh`, which renames a change's `DRAFT-<branch>-<slug>.md` ledger files to dated names. For the change merged as `fbf17f0` — the `on-chain-client` half of tooth 3 — it never ran, and neither did step 4. Three files went onto the base branch still carrying draft names, in `on-chain-client/docs/requirements`, `docs/risk` and `docs/problems`. `check-ids.sh` convicts a draft-named ledger file as `DRAFT-FILE`, so this unit's id gate has been exit 1 on `master` from 2026-09-11 until this change merges, and no `on-chain-client` change could have passed `merge-change` step 4 in that time. Tooth 4's architecture work for this unit was blocked behind it.

**The names are dated 2026-09-10**, not the 2026-09-11 merge date, and the choice is the owner's. `fbf17f0`'s own verification record states that the change was finalized on the 10th and merged on the 11th, and asserts that the ledger files carry 2026-09-10 names — which they never did, because step 3 never ran. Naming them 2026-09-10 makes that existing assertion true rather than leaving it false in a new way, and it matches what both sibling units of this same tooth did: org-node's ledger is dated 2026-09-09 against a 2026-09-10 merge, app's 2026-09-14 against a 2026-09-15 merge. This diverges from the ledger READMEs' written wording of "merge date"; that divergence is pre-existing, is the observed practice of three of the four units, and is already recorded as finding-69 in `docs/verification/2026-09-10-worktree-guardrails-org-node-risk.md`.

**`finalize-docs.sh` was deliberately bypassed for the rename itself.** It takes no date argument — it stamps `date +%Y-%m-%d` and strips only the *current* branch's prefix — so run from this repair's branch it would have produced `2026-09-17-worktree-guardrails-on-chain-client-risk-chain-reading.md`, wrong in the date and still carrying the dead branch name. The `git mv` commands reproduce the output the script owed on the day it was due. `finalize-docs.sh --dry-run` was still run at step 3 for both touched units and correctly found nothing left to rename.

**Seven cross-references were repaired in the same commit.** That timing is deliberate: `finalize-docs.sh` renames a file and says nothing about references to it, and no gate in the toolkit reads prose links, so nothing mechanical will ever catch a reference left pointing at a consumed draft name. This project has now hit that defect three times; the rule it settled on — after every rename, sweep the tree for the name just consumed — was followed here over all files rather than markdown alone.

**Why it went undetected is the finding worth keeping, and the first two answers written here were both wrong.** Locally, the sequence cannot self-detect it: `verify-before-merge` check 3 runs `check-ids.sh` **with** `--allow-draft-files` — correct there, since a change's own draft is legitimate for its whole life — and the only local bare run is `merge-change` step 4, which the same operator omission skips. But this change first claimed step 4 was the *only* bare runner anywhere, which is false: `.github/workflows/rust.yml` runs `check-ids.sh` bare per unit on every non-pull-request event. It then claimed CI had been dormant since 2026-06-17, which is also false, and understated the problem. `rust.yml` was added by `4bb5509` on 2026-08-27 — one of the 45 unpushed commits — and `origin/master` is still at `2bb1c21` of 2026-06-17, so **that workflow has never executed once.** The bare `check-ids.sh`, the clippy denial, the `no_std`/wasm32 checks and the cross-platform coverage re-run have no execution history at all rather than a lapsed one. Within `quint.yml` the picture is mixed and is now recorded per invariant: three randomised runs and three Apalache verifies existed at `2bb1c21` and ran until that date; `tauWindow` and `convergence` were added by `620b459`, also unpushed, and have never run.

**How it was found was also stated wrongly at first.** It did not surface from an impact-set run: `on-chain-client` was outside the org-members change's impact set and its gates were not run as part of it. The red came from a deliberate out-of-scope probe. The structural point matters more than the correction — the dependency graph alone could not have pulled this unit into an org-members change's impact set, since org-members is a leaf and the two units neither depend on each other, so crediting the impact-set mechanism would make the process look sounder than it is.

## Review

**finding-1**: The 2026-09-11 date was defensible against the ledger READMEs' written convention, but the stated basis for it was contradicted by `fbf17f0`'s own verification record, which says the change was finalized on the 10th and asserts the files carry 2026-09-10 names; observed practice in org-node and app agrees with the record, not the README.
disposition: fixed. The owner chose 2026-09-10; the three files were re-dated with `git mv` and all ten references rewritten. The justification now cites the record and both sibling units, and discloses the divergence from the README wording along with its pre-existing precedent (finding-69 of the org-node record).

**finding-2**: `docs/verification/2026-09-11-worktree-guardrails-on-chain-client-risk.md` asserts "the ledger files carry 2026-09-10 names" — untrue before this change and untrue after it while the files were dated 09-11 — and is itself the documentary evidence of the skip, its gate table carrying a row for `check-ids.sh --allow-draft-files` and none for `finalize-docs.sh` or a bare `check-ids.sh`.
disposition: fixed. The re-date makes the assertion true, and a dated 2026-09-17 correction note was added to that record recording what was untrue, what is true now, that steps 3 and 4 were both skipped, and the missing gate-table rows as the evidence. Original text left standing per the project's convention. `check-review.sh` for that branch still reports 41 findings.

**finding-3**: The claim "only `merge-change` step 4 runs it bare" is false — `.github/workflows/rust.yml` runs `check-ids.sh` bare per unit on every non-pull-request event, so CI would have convicted `fbf17f0` on the first push. The real reason nothing convicted was unstated: nothing has been pushed.
disposition: fixed. Both bare runners are now named, and the actual cause recorded — `origin/master` at `2bb1c21`, 45 commits behind — with the dormant CI-only gates enumerated.

**finding-4**: The claim that the defect was found by the org-members change's impact-set run is false; that unit was outside the set and the red came from a deliberate out-of-scope probe. Crediting the impact-set mechanism overstates the process in the one paragraph written to record its gap.
disposition: fixed. The account now says it was an out-of-scope probe, and adds the structural point that on-chain-client could not have entered that change's impact set via the dependency graph.

**finding-5**: The systemic lesson was filed only in the plan of a change that merged six days earlier, where no future author would read it; the standing open-items register was untouched and no problem report was filed.
disposition: fixed. A dated section was added to `docs/plans/2026-09-05-ratchet-setup.md`, the repository's running register, carrying the open items as unchecked owner decisions. No `PR` item was minted — see Gaps.

**finding-6**: `check-signing.sh` was listed as a CI-only, therefore dormant gate, with the rider that non-negotiable 2 is "enforced by the merging operator alone". Both halves are false: `finish-merge.sh:130` runs it `--strict` and unconditionally as guard 1, which is *stricter* than CI's non-strict run.
disposition: fixed in both copies of the list. The entry now states the local `--strict` run and describes CI as an absent independent backstop rather than the absent enforcement. The false rider was deleted.

**finding-7**: `make coverage` was listed as dormant. `Makefile:139` makes it exactly the two per-unit `coverage_command` targets, consumed locally by `verify-before-merge` check 5 — this change's own gate measured them.
disposition: fixed in both copies. Only the cross-platform re-run is dormant (floors calibrated on aarch64-darwin, enforced in CI on x86_64-linux against a floating toolchain), which is what the entry now says.

**finding-8**: The bare `check-ids.sh` was listed as CI-only two paragraphs after the same section correctly said the only local bare run is `merge-change` step 4 — a self-contradiction that told a reader no local bare run exists.
disposition: fixed in both copies. The entry now says a local bare run exists at step 4 but is downstream of the omission that skips it, so CI's is the only bare run independent of the operator reaching that step.

**finding-9**: "The whole of `quint.yml`" is both overstated and understated: two of its typechecks are verbatim local `verify_commands` entries and its `mbt` job duplicates a local target, while four genuinely CI-only invocations went unnamed.
disposition: fixed in both copies. The enumeration now names what is CI-only and explicitly excludes what runs locally.

**finding-10**: "Three of the four risk analyses" undercounts the unpushed commits; all four are unpushed.
disposition: fixed, with the four SHAs named.

**finding-11**: The re-date left `app/docs/risk/2026-09-14-app-hazards.md` citing on-chain-client's register as "(2026-09-11)", inconsistent in exactly the dimension the date reasoning relies on.
disposition: fixed in place. This is what moved `app` into the touched impact set, and the set was recomputed rather than assumed.

**finding-12**: Reason 3 for the date ("keeps the files ordered before the `app` register that cites them") discriminates nothing — 2026-09-11 also sorts before 2026-09-14 — and the app register cites no file path at all.
disposition: fixed. Reason 3 deleted; the two reasons that do the work stand on their own.

**finding-13**: The correction note asserted the repair is the change "and its own verification record", a record that did not exist at commit time, inside a record whose subject is a skipped merge step.
disposition: fixed, then fixed again — see finding-21. It was first reworded to name the record's exact dated path; the clock later made that path wrong too.

**finding-14**: Two documents still asserted a blocker this change removes — `docs/plans/2026-09-15-org-members-architecture.md` and `docs/verification/2026-09-17-worktree-guardrails-org-members-arch.md` both said on-chain-client's gate "is red on `master` today" and that tooth 4's change for that unit cannot merge.
disposition: fixed. Dated correction notes added to both, blockquoted so nothing at column one can be read by `check-review.sh` as a field or a finding header. That record still reports 27 findings.

**finding-15**: Nothing in the committed prose disclosed that the three names were assigned by hand rather than by the tool; only an intermediate commit message said so, and the squash discards it.
disposition: fixed. The plan now records that `finalize-docs.sh` was deliberately bypassed, why a late run would have produced the wrong name in both halves, and that `--dry-run` was still run at step 3.

**finding-16**: The "delete or disable the workflows" option would moot three still-open items in the same register with no cross-reference to any of them.
disposition: fixed. All three are named with their exact wording under that option.

**finding-17**: The dormancy was mis-dated and the error understated the problem: "dormant since 2026-06-17" reads as a gate that lapsed, but `rust.yml` was added by an unpushed commit and has never run at all, and two of the five quint invariants likewise.
disposition: fixed. The section now separates "ran until 2026-06-17" from "never ran", per workflow and per invariant, and the owner-decision text calls pushing a first bring-up rather than a catch-up.

**finding-18**: A class C hazard register cites CI verification evidence that was never produced — `org-members/docs/risk/2026-09-02-membership-hazards.md` argues of `tauWindow` that "CI runs that invariant under the simulator (5000 samples, 16 steps) and under Apalache to depth 5, so it is checked rather than merely written down". That CI step has never executed. The change stated the general rule about records implying gates ran, but named no affected document.
disposition: recorded, not fixed, by the owner's decision of 2026-09-17. Both locations are now named exactly — that register and `org-node/docs/risk/2026-09-09-org-node-hazards.md`, whose weaker form is a minting condition rather than a false assertion — and filed as an unchecked owner item. Correcting them alters what a hazard's residual-risk argument rests on, which is a risk decision belonging to `analyze-risks`, and would pull two further units' risk ledgers into a deliberately documentation-only change. See Gaps: this is the most consequential thing the change uncovered and it leaves the repository unrepaired.

**finding-19**: A false locator — an item said to be "in the 2026-09-14 section" is in a different section entirely.
disposition: fixed, and the two companion locators checked and confirmed correct.

**finding-20**: The absolute "on-chain-client can never appear in an org-members change's impact set" is not true: `check-units.sh` maps a change under the root `.guardrails/` to every unit.
disposition: fixed. The claim is now scoped to the dependency graph, with the root-config rule stated explicitly.

**finding-21**: The 2026-09-17 date the correction note had been given for this change's own verification record went stale when the merge did not happen that day, which would have shipped a certified record pointing at a file that never existed.
disposition: fixed by removing the guess rather than re-dating it. The record is named by its branch, which is how `check-review.sh` selects one — `branch:`, matched whole, never the filename, as the verification template states. The note records why, since guessing a merge date ahead of a merge is the precise mistake this change exists to repair.

**finding-22**: Five passages across three files dated the repair as already landed on `master` on 2026-09-17. It had not landed; the 2026-09-11 record contradicted itself inside one blockquote, one paragraph naming that terminal date and another, four paragraphs down, saying the merge did not happen that day.
disposition: fixed. Every terminal date and day count is now written against the merge rather than a guessed date, and each site says why no date is given. The one surviving 2026-09-17 is the sentence describing the mistake, which needs it.

**finding-23**: The register section opened "Two open items … Neither item below is the repair. Both are about why nothing convicted" and had grown to four; its heading still said "the dormant CI", which the section's own body repudiates.
disposition: fixed. The intro counts four and splits them — (a) and (b) structural, the other two documentary collateral — and the heading is retitled. All four inbound cross-references cite the section as "Added 2026-09-17" and none quotes the subtitle, so nothing else moved.

## Gaps

**What this change did not establish, stated so that each is a gap rather than a claim.**

- **A class C hazard register still argues from verification evidence that was never produced.** `org-members/docs/risk/2026-09-02-membership-hazards.md` uses a CI run of `tauWindow` to argue the invariant "is checked rather than merely written down"; that CI has never executed. This change names the location and defers the repair to its own change under `analyze-risks`, by the owner's decision. Until then the register's argument stands on evidence that does not exist. This is the most consequential thing the change uncovered.
- **No test covers the defect, and none can at this layer.** The failure is a merge-sequence step not being run; its evidence is a gate's exit code, not something a unit test could assert. The reproduction is recorded in `reproduced:` above and nowhere else.
- **Nothing here makes the local sequence self-detecting.** The next skipped step 3 will reach the base branch the same way. The toolkit is upstream (guardrails 0.5.1 at `e2eac86`) and was not modified.
- **CI has never run for the principal workflow and this change does not change that.** Whether to push is the owner's decision, recorded in `docs/plans/2026-09-05-ratchet-setup.md` and acted on nowhere.
- **Two stale verification-record references written by `fbf17f0` are named but not fixed** — `Makefile:47` and `on-chain-client/.guardrails/config.yaml:106` both cite a `2026-09-10` record path that does not exist. Fixing them would have cost this change its documentation-only property by pulling gate configuration into a prose repair. Recorded as an open item instead.
- **No problem report was minted.** A `PR` item belongs to a unit's ledger and describes an anomaly of that unit's product; these are process and tooling defects, and filing them under one of the four units would misplace them.
- **`on-chain-client` statement coverage is 41.75% against a class C target requiring statement coverage — a shortfall, not a pass.** The Makefile floor of 41 is a regression guard set one point below the measured figure. `client.rs` alone accounts for it: 281 of its 336 lines are unexercised, because the chopsticks/anvil integration targets that exercise them sit outside `verify_commands`. Unchanged by this change, which touched no source.
- **Decision coverage is unmeasured in every unit** — llvm-cov reports Branches 0/0 repository-wide — against a class C target that requires it. Accepted by the owner as a documented gap and scheduled for tooth 5. Recorded as unmeasured, never as passing.
- **`org-node` and `app` configure no `coverage_command` at all**, so neither has ever been measured. The owner's 2026-09-14 decision was to set both floors together at a later tooth. Their absence is not a pass either.
- **`on-chain-client` has `SDD 0, LLR 0` against 21 requirements**, and its `soup.md` is still the empty template. That is tooth 4's work for this unit, which this change unblocks and does not perform.
