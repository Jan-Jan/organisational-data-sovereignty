# Guardrails upgrade to 2bc3cca — 2026-10-15

**Goal.** Move the six units from the guardrails scripts at `344faa3` to
`2bc3cca` (3 upstream commits: `52100a8`, a ratchet qualification run names
the one commit it tested; `bb269de`, merge-change gates after the record and
step 1 stops while a squash is staged; `2bc3cca`, development runs the tests
its work touches and the whole suite runs at the merge gate), following
`ratchet`'s "Upgrading the scripts" path and `references/upgrade-notes.md`.
No requirement, hazard, design item, source file, test or CI line changes.
The upstream template still reads `guardrails_version: 0.5.1`;
`guardrails_commit` is what moves.

**Why now.** The installed skills already run at `2bc3cca`: `merge-change`
uses the renumbered steps (gate at 6a, after the record at 5), while the
installed scripts, the verification template and the AGENTS.md managed block
still name the old step numbers.

## Measured before the upgrade

On master `45e47fe` with the scripts at `344faa3` (every installed script,
the four `.guardrails/templates/` files and the AGENTS.md managed block
byte-identical to that commit): no `DRAFT-` file and no `REQ-DRAFT-` token
(`check-ids.sh` exit 0 for every unit; the only textual hits are prose in the
two earlier upgrade plans); `check-units.sh` exit 0; `check-trace.sh` exit 0
for every unit. The warnings printed are `UNRESOLVED-PR` (on-chain-client 4,
org-node 10, app 12, org-io 3) and `UNMET-EXPECTATION` (org-node 2, app 1,
org-io 1), each already recorded in its ledger and unchanged since the
`344faa3` upgrade. The list-item annotation sizing grep (upgrade-notes item 3)
prints nothing over `docs/` and every unit's `docs/`.

With the new scripts on the same ledgers, the output of `check-units.sh`,
`check-ids.sh` and `check-trace.sh` is byte-identical to the old for all six
units, exit 0 throughout. This upgrade adds **no new failure and no new
warning**.

## Baseline (worktree-discipline step 4)

Every unit's `verify_commands`, in this worktree, before the copy. The first
run was red in three units, each for an environment reason, not a defect:

- org-members: 11 `mbt_conformance` tests failed with `QUINT_HOME` set to an
  empty scratch directory (the evaluator fetch on first use, under parallel
  tests). Rerun with it populated: 271 passed, 0 failed; every quint step
  exit 0.
- on-chain-client and app (`src-tauri`): `cargo` exit 101, `failed to
  download` / `failed to open` under the read-only `~/.cargo`. Rerun with
  `CARGO_HOME=/tmp/cargo_home_fuzz`: on-chain-client 73 + 55 passed, app
  164 passed, 0 failed.
- app npm: exit 127, `app/node_modules` absent from a fresh worktree. Copied
  from the primary checkout (`app/.gitignore` ignores it): `npm run check`
  and `npm run test` exit 0.
- org-node 307 passed, person 123 passed, org-io green, every clippy and
  quint step exit 0, on the first run.

## Tasks

### Task 1 — copy the machinery (done)

From the clone of `2bc3cca` (`ratchet` step 0): `scripts/*.sh` into
`.guardrails/scripts/` (`check-ids.sh`, `check-review.sh`, `check-trace.sh`,
`finish-merge.sh`, `merge-preflight.sh` and `prune-plans.sh` change, in
comments and messages only: the merge-change step numbers; the other seven
were already identical); `templates/*_GUIDELINES.md` and `verification.md`
into `.guardrails/templates/` (`TEST_GUIDELINES.md` gains `## Test runs`;
`verification.md` renumbers its steps). The AGENTS.md managed block was
byte-identical to the `344faa3` template and is replaced between its markers;
the one changed line names `merge-preflight.sh` as merge-change steps 2 and
6c. The four ledger templates (`srs.md`, `rmf.md`, `sad.md`, `problems.md`)
and the config template did not change upstream between the two commits, so
no unit's README is touched; the risk and architecture READMEs keep the
project's own additions.

### Task 2 — config (done)

`guardrails_commit` set to `2bc3cca04b75e1205d1c79a8e7b40c302036bf6f` in all
six unit configs.

### Task 3 — the qualification record (done: `docs/plans/2026-10-15-ratchet-setup-2bc3cca.md`)

### Task 4 — upgrade-notes items with nothing to do here (checked)

- `tailor-guidelines` in upgrade mode (ratchet upgrade step 2) has nothing
  to adopt: no unit and no root has a `*_GUIDELINES.md` of its own, so every
  unit follows the installed default, which now carries `## Test runs`.
- `check-review.sh` changed in comments and messages only, so upgrade-notes
  item 6 needs no rerun.
- The installed skills already match `2bc3cca` (`diff -rq` against the
  checkout).
- Verification records and plans merged before this change use the old
  merge-change step numbers; read them through the table in
  upgrade-notes "The merge steps are renumbered".

## Follow-ups (not this change)

1. **Architecture guidelines** (carried from the `344faa3` upgrade): run
   `tailor-guidelines` for `ARCHITECTURE`, and now `TEST` for the `## Test
   runs` example, where a unit wants its own file.
