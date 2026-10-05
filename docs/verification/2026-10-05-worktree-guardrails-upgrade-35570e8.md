# Verification — guardrails upgrade to 35570e8 (2026-10-05)

branch: worktree-guardrails-upgrade-35570e8
reviewer: two independent fresh subagents, one per round, each in its own review worktree (`.worktrees/worktree-guardrails-upgrade-35570e8-review`, then `-review2`) with the diff, the plan, the ratchet upgrade procedure and the review checklist, and no implementation narrative
verdict: ACCEPTED for merge 2026-10-05 — round 1 raised one requirement finding (medium) and four record findings; round 2 raised two requirement findings, both low, and two record findings, which made it the last review round; every finding is fixed
reproduced: yes — the defect this change answers is the gap between the installed scripts and upstream: the 35570e8 `check-trace.sh`, run read-only against master `05f6f04` before any edit, reported 123 UNANALYZED-DERIVED, 5 ORPHAN-ANNOTATION and 1 DANGLING-FILE where the installed 0.5.1 scripts exit 0 on the same tree

Change: upgrade the five units' guardrails machinery from upstream `e2eac86`
(0.5.1) to `35570e8`, and bring the ledgers up to what the new checks read.
Branched from `master` at `05f6f04`; local `master` at `1feb608` (the org-node
type-safety change) merged in at `c6a41da`, without conflicts.
Plan: `docs/plans/2026-10-05-guardrails-upgrade-35570e8.md`.
Qualification: `docs/plans/2026-10-05-ratchet-setup-35570e8.md`.

Resolves: none. Accepts: none. Opens: none.

## The gate

Measured on: `c6a41da406111061233698a8cd2d2a522dc100e4` — tree
`d3d2a6d14723d676f0b5dd4e65ab300820897069`, clean worktree, by a fresh gate
subagent after local `master` (`1feb608`) was merged in. Every command was
re-read from the current configs (org-node's list gained `persona_records`,
`secret_redaction` and `calldata_typed` with that merge). `merge-change`
step 3 renamed nothing (this change has no draft ledger file). Impact set:
all five units, `touched`. `check-units.sh`: exit 0. All 35 commands passed
on their first run.

| Gate | Result |
| --- | --- |
| org-members `cargo test -p org-members` | 225 passed, 0 failed, 1 ignored |
| org-members quint: typecheck ×3, `quint test`, `quint run … mbtInv` | clean; 63 passing; no violation |
| on-chain-client `cargo test` (lib + 12 targets) | 73 passed, 0 failed |
| org-node `cargo test` (features `app,test-support`, lib + 22 targets) | 167 passed, 0 failed |
| org-node quint: typecheck ×2, five invariants | clean; no violation in any |
| app `cargo test` (6 targets) | 82 passed, 0 failed |
| app `npm run check` / `npm run test` | 0 errors, 1 warning (missing `@types/node`) / 30 passed |
| person `cargo test -p person` / clippy `-D warnings` | 73 passed, 0 failed / clean |
| `check-ids.sh --allow-draft-files`, each unit | exit 0, no findings |
| `check-trace.sh`, each unit | exit 0; this change's own effect: 123 UNANALYZED-DERIVED and 5 ORPHAN-ANNOTATION and 1 DANGLING-FILE gone; no ID entered or left the roll-call; `ADR` now counted |
| `merge-preflight.sh --local-base`, with the review check | CLEAN-TREE, BASE-MERGED (local base), UNITS, REVIEW, NESTED-WORKTREE ok; IDS/TRACE run per unit |
| Coverage, against the class target | **SHORTFALL, accepted by the owner 2026-10-05 for this merge.** Measured on this tree with each configured `coverage_command`, all exit 0: person 100.00% lines, 100.00% regions; org-members 94.91% lines, 93.86% regions; on-chain-client 41.75% lines, 42.60% regions (`client.rs`, the live-chain client, 16.37%). Class C needs statement **and** decision coverage: decision coverage is unmeasured in every unit (branches reported `-`), and org-node and app configure no `coverage_command`. This change adds no code; the org-members movement since the acceptance (94.83% → 94.91% lines) came with `1feb608`. |
| Working tree | clean before and after |

Earlier gates on this change, none of whose figures are reported above:
round 1 at tree `b2430e66…` (org-members' `mbt_conformance` failed 11 of 32 on
the first run, every failure "Quint returned non-zero code" while quint
fetched its evaluator into an empty `QUINT_HOME`; the unchanged re-run passed —
an environment fault, inferred, not proven); round 2 at tree `94bf20c9…` and
round 3 at tree `254e6ef0…`, before `master` moved, both green first run. The
owner's coverage acceptance was given on round 3's figures.

## Red → green

This change implements no REQ, LLR, SDD or RC, and adds and changes no test.
No row applies.

| Item | Test | Watched red |
| --- | --- | --- |
| none | none | not applicable: documentation and tooling only |

## What was wrong, and what was built

**Measured before.** Upstream `35570e8` adds `merge-preflight.sh`,
`task-worktree.sh` and `find-items.sh`, which `merge-change` and
`develop-change` now call; the previous merge
(`worktree-guardrails-org-node-arch`) ran their checks by hand. Run read-only
against master `05f6f04`, the new `check-trace.sh` reported, per unit:
org-members 20 UNANALYZED-DERIVED, 1 DANGLING-FILE; org-node 59; app 22;
on-chain-client 22; each unit 1 ORPHAN-ANNOTATION (line 44 of its
`docs/problems/README.md`, the 0.5.1 template's own example line). Person was
clean apart from that line.

**Built.**
- The twelve scripts copied from upstream; their blob hashes and modes match
  upstream `35570e8` (both review rounds confirmed).
- Twenty ledger READMEs and the verification template three-way merged
  (base `e2eac86`, theirs `35570e8`, ours the project's copy), all clean; the
  project's own additions are kept (the filled acceptability matrix in every
  risk README; the decomposition sections in three architecture READMEs).
  This cleared the five ORPHAN-ANNOTATIONs.
- The AGENTS.md managed block replaced between its markers; the project had
  not edited inside it, and nothing outside it changed.
- `guardrails_commit: 35570e8…` and `ADR` in `id_prefixes` in all five unit
  configs. Declaring ADR changes no exit code; each `checked:` line gains
  `, ADR 0`.
- 71 `assesses:` lines (122 distinct IDs, 123 mentions) declaring assessments
  that already existed; no assessment text changed.
- LLR-v3jqau, the 123rd report, was a misread, not a missing assessment: the
  item `satisfies: REQ-ds8ryr`, and a later paragraph inside its block quoted
  `satisfies: derived` about LLR-k89ahd. The quotation now reads "It stays
  derived", and a heading now closes the item's block before that paragraph.
- The org-members DANGLING-FILE was rename narration beside a correct dated
  reference; the narration no longer spells the draft name.
- Tool qualification re-recorded: the upstream suite at `35570e8`, 1053/1053
  ok, exit 0 (second run; the first run's exit code was lost to zsh's
  read-only `status`). The new `check-review.sh` run once over each of the 24
  existing records: 24 of 24 exit 0.

## Review

### Round 1

**finding-1**: requirement, medium — Task 7 of the plan and steps 5–6 of ratchet's "Upgrading the scripts" are not in the tree, and that procedure says to do them before `merge-change`. Missing: the tool-qualification record `docs/plans/2026-10-05-ratchet-setup-35570e8.md` (the `run-tests.sh` version, commit and result), and any evidence of `check-review.sh --branch <name>` run once over each existing verification record.
disposition: fixed. `docs/plans/2026-10-05-ratchet-setup-35570e8.md` records the basis (`35570e8`, all five configs), the suite result (1053/1053, exit 0) and the 24-of-24 `check-review.sh` pass with every branch named; `docs/plans/2026-09-05-ratchet-setup.md` points to it. Round 2 re-ran the 24 checks itself: 24 of 24 exit 0.

**finding-2**: record — The plan says Tasks 3–6 touch `<unit>/docs/risk/*.md` only, but commit f6574f5 edits `org-members/docs/architecture/2026-09-17-decomposition.md`, and the plan never says that the 123rd UNANALYZED-DERIVED (LLR-v3jqau) was cleared by removing a false derived declaration rather than by an `assesses:` line.
disposition: fixed. The plan's Tasks 3–6 outcome states the LLR-v3jqau misread, the fix and its commit, made by the dispatcher outside the task's file set.

**finding-3**: record — Task 8 says declaring ADR "changes no report". Measured, the `checked:` line of every unit gains `, ADR 0`. No exit code or gate result changes.
disposition: fixed. Task 8 now says no gate result changes and the `checked:` line gains `, ADR 0`.

**finding-4**: record — The plan does not mark Tasks 3–6 or Task 8 "(done)", though the tree shows both done.
disposition: fixed. Tasks 3–6, 7 and 8 are marked done.

**finding-5**: record — Two units place their `assesses:` lines in a different style: in app and on-chain-client the line comes right after its passage with no blank line, so it renders as part of that text.
disposition: the plan states the two placements; the rendering is fixed under round 2's finding-2.

### Round 2

**finding-1**: requirement, low — The LLR-v3jqau fix removes the misread, but the paragraph that caused it is still inside LLR-v3jqau's block (`org-members/docs/architecture/2026-09-17-decomposition.md:316-345`); an annotation-shaped quotation written there again would attach to LLR-v3jqau again. A heading above the paragraph would close the block. The same shape exists at `org-node/docs/architecture/2026-10-03-decomposition.md:816`, harmless today because the item's own `satisfies: derived` wins.
disposition: fixed for org-members: the heading "Amendments to LLR-s97ywt, LLR-w92psx and LLR-k89ahd" now closes LLR-v3jqau's block; every unit's `checked:` counts and exit codes are unchanged. The org-node instance is left as it is, harmless as the reviewer states, and noted under Gaps.

**finding-2**: requirement, low — In app and on-chain-client the `assesses:` line comes straight after the passage with no blank line, so in rendered Markdown it joins the preceding paragraph or list item.
disposition: fixed. Blank lines inserted around every such line (46 blank lines across three files; the diff adds only empty lines); check-trace unchanged.

**finding-3**: record — The plan's "122 IDs declared on 71 `assesses:` lines" counts distinct IDs; the lines hold 123 mentions, because REQ-bvx4nh is declared twice in app. Both are genuine.
disposition: fixed. The plan states 122 distinct IDs, 123 mentions, and why.

**finding-4**: record — The setup record says the script blob hashes were "checked by the change's independent review" before that review existed.
disposition: fixed. It now says both review rounds confirmed it after the line was first written.

## Gaps

- Neither review round re-ran the 1053-test upstream suite; its result rests on
  the dispatcher's two runs, recorded in the setup file.
- `check-signing.sh --setup` cannot pass inside the agent sandbox (gpg cannot
  open `trustdb.gpg`); the owner's `finish-merge.sh` is the signing proof.
- The trace check's item block runs to the next definition or heading, so a
  quotation of an annotation in trailing prose is read as the item's own. One
  instance remains, harmless, at `org-node/docs/architecture/2026-10-03-decomposition.md:816`
  (LLR-e5c9ud). The behaviour is upstream's; not reported upstream by this change.
- Coverage: the class C shortfall in the gate table, accepted by the owner for
  this merge only. Decision coverage is unmeasured everywhere, and org-node and
  app measure nothing; the ratchet's coverage tooth (5) is still to come.
- The coverage commands were run by the dispatcher after the gate subagent
  reported, on the same clean tree; the gate prompts had omitted check 5.
