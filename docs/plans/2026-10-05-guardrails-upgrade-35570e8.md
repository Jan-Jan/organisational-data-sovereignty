# Guardrails upgrade to 35570e8 — 2026-10-05

**Goal.** Move the five units from the guardrails scripts at `e2eac86`
(0.5.1) to `35570e8`, following `ratchet`'s "Upgrading the scripts" path and
`references/upgrade-notes.md`. No requirement, hazard, design item or
production code changes. The upstream template still reads
`guardrails_version: 0.5.1`; `guardrails_commit` is what moves.

**Why now.** The new scripts add `merge-preflight.sh`, `task-worktree.sh` and
`find-items.sh`, which `merge-change` and `develop-change` now call; the
previous merge ran their component checks by hand. Doing this before the next
feature change means that change is gated by the checks it will merge under.

## Measured before the upgrade

The new `check-trace.sh`, run read-only against master `05f6f04`, reported as
new failures: 123 `UNANALYZED-DERIVED` (org-members 20, org-node 59, app 22,
on-chain-client 22), one `ORPHAN-ANNOTATION` per unit (line 44 of each
`docs/problems/README.md`, the 0.5.1 template's own example line) and one
`DANGLING-FILE` in org-members. The old scripts exit 0 on the same tree.
`UNRESOLVED-PR` and the plain `UNMET-EXPECTATION` lines are informational under
both versions.

## Tasks

### Task 1 — copy the machinery (done, `chore(guardrails): scripts, …`)

`scripts/*.sh` into `.guardrails/scripts/`. The four ledger READMEs per unit
and `.guardrails/templates/verification.md` three-way merged (base: the
template at `e2eac86`, theirs: at `35570e8`, ours: the project's copy); all 21
merged clean, keeping the project's additions to the risk and architecture
READMEs. The AGENTS.md managed block replaced between its markers; the
project had not edited inside it, and the old block held no prose rules.
Clears the five `ORPHAN-ANNOTATION`s.

### Task 2 — config and the dangling link (done)

`guardrails_commit: 35570e8…` in every unit config. The org-members
`DANGLING-FILE` was narration of a rename beside a correct dated reference;
the narration no longer spells the draft name.

### Tasks 3–6 — declare the derived assessments, one unit each (done)

Files touched: `<unit>/docs/risk/*.md` only. Units: 3 org-members, 4 org-node,
5 app, 6 on-chain-client.

**Outcome.** 122 distinct IDs declared on 71 `assesses:` lines (org-members
19, org-node 26, app 4, on-chain-client 22): 123 mentions, because app
declares REQ-bvx4nh twice — in the §9 group and on the paragraph that
assesses it specifically, both genuine assessments. Every one had an existing
assessment, and no assessment text was rewritten. The 123rd report,
LLR-v3jqau, was a misread: the item `satisfies: REQ-ds8ryr`, but its block
runs to the next heading and took in a later paragraph that quoted the
annotation `satisfies: derived` about LLR-k89ahd. That quotation now reads
"It stays derived", in `org-members/docs/architecture/2026-09-17-decomposition.md`
(commit `f6574f5`, made by the dispatcher outside the task's file set,
because the fix is in the architecture ledger, not the RMF). Declaring an
assessment for an item that is not derived would have been false.

Two placements are in use, both on a line of their own as the risk README
requires: before the passage (org-members, org-node) and directly after it
(app, on-chain-client).

For every `UNANALYZED-DERIVED` ID the unit's `check-trace.sh` reports, find the
passage in the unit's RMF files that already assesses that item, and add a
column-one `assesses: <IDs>` line to that passage naming exactly the items it
assesses. Never on a table row or a passing mention. An ID with no assessment
anywhere is not annotated: it is reported, and becomes an `analyze-risks`
item before merge. No assessment text is rewritten.

Done when the unit's `check-trace.sh` reports no `UNANALYZED-DERIVED`.

### Task 7 — the qualification record (done: `docs/plans/2026-10-05-ratchet-setup-35570e8.md`)

Run `<guardrails>/tests/run-tests.sh` once, record version, commit and result
in `docs/plans/2026-10-05-ratchet-setup-35570e8.md`. Run
`check-review.sh --branch <name>` once over each existing verification record
(upgrade-notes: older `check-review.sh` never executed on macOS).

### Task 8 — `ADR` as a declared prefix (done)

The shipped config declares `ADR`. No ADR in this repository has an item
line, and no document cites an ADR by ID, so declaring it changes no gate
result: every exit code is unchanged, and the only output difference is that
each unit's `checked:` line gains `, ADR 0`. It lets `new-id.sh ADR` mint one.
