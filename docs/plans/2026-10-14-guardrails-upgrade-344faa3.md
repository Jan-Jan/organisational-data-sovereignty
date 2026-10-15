# Guardrails upgrade to 344faa3 — 2026-10-14

**Goal.** Move the six units from the guardrails scripts at `6a9d6eb` to
`344faa3` (2 upstream commits: `bb249ca`, the squash commit certifies the
gate; `344faa3`, architecture guidelines are a project file), following
`ratchet`'s "Upgrading the scripts" path and `references/upgrade-notes.md`.
No requirement, hazard, design item, source file, test or CI line changes.
The upstream template still reads `guardrails_version: 0.5.1`;
`guardrails_commit` is what moves.

**Why now.** The installed skills already run at `344faa3`: `merge-change`
step 7 writes `Base:`, `Gate-tree:` and `Gate:` trailers, and `develop-change`
asks `guidelines-file.sh ARCHITECTURE` for a third conformance review. Until
this merges, the installed `finish-merge.sh` does not check the trailer the
skill writes, and `guidelines-file.sh ARCHITECTURE` exits 2.

## Measured before the upgrade

On master `f3a5c38` with the old scripts: no `DRAFT-` file and no
`REQ-DRAFT-` token; `check-units.sh` exit 0 (6 units, 6 disclaimed);
`check-ids.sh` and `check-trace.sh` exit 0 for every unit. The warnings
printed are `UNRESOLVED-PR` (on-chain-client 4, org-node 10, app 12,
org-io 3) and `UNMET-EXPECTATION` (org-node 2, app 1, org-io 1), each already
recorded in its ledger.

With the new scripts on the same ledgers, the output of `check-ids.sh` and
`check-trace.sh` is byte-identical to the old for all six units, and
`check-units.sh` differs only in the tracked-path count (728 → 729: the new
`.guardrails/templates/ARCHITECTURE_GUIDELINES.md`, measured after the copy
and before this plan and the qualification record were added, which bring it
to 731). This upgrade adds **no
new failure and no new warning**.

## Tasks

### Task 1 — copy the machinery (done)

From a clean `git archive` of `344faa3`: `scripts/*.sh` into
`.guardrails/scripts/` (`check-signing.sh`, `finish-merge.sh` and
`guidelines-file.sh` change; the other ten were already identical);
`templates/*_GUIDELINES.md` and `verification.md` into
`.guardrails/templates/` (adds `ARCHITECTURE_GUIDELINES.md`). The AGENTS.md
managed block was byte-identical to the `6a9d6eb` template and is replaced
between its markers; non-negotiable 2 now names the `Gate-tree:` trailer and
`finish-merge.sh`. The four ledger READMEs and the config template did not
change upstream between the two commits, so no unit's copy is touched.

### Task 2 — config (done)

`guardrails_commit` set to `344faa33a09fdc2a4d381c7ef710e1f5631e24c8` in all
six unit configs.

### Task 3 — the qualification record (done: `docs/plans/2026-10-14-ratchet-setup-344faa3.md`)

### Task 4 — upgrade-notes items with nothing to do here (checked)

- No CI line reads a verification record's `Measured on:` (grep over
  `.github/workflows/`). Records already merged keep their gate tables, as
  the upgrade notes leave them.
- `check-review.sh` is unchanged, so upgrade-notes item 6 needs no rerun.
- `tailor-guidelines` in upgrade mode (ratchet upgrade step 2) has nothing
  to adopt: no unit has a `*_GUIDELINES.md` of its own, and upstream's
  `CODE_GUIDELINES.md` and `TEST_GUIDELINES.md` did not change between
  `6a9d6eb` and `344faa3`.
- Every squash merged through `merge-change` from now on needs the trailer
  paragraph; the installed skill writes it.

## Follow-ups (not this change)

1. **Architecture guidelines.** `guidelines-file.sh ARCHITECTURE` prints the
   installed default for every unit, as `TEST` and `CODE` do. Run
   `tailor-guidelines` for `ARCHITECTURE` (per unit where a unit wants its
   own) to make it the project's own; until then every `develop-change`
   runs a third conformance review against the default.
2. The worktree `guardrails-upgrade-6a9d6eb` (merged as `f3a5c38`) is still
   registered; it is left for its owner to remove.
