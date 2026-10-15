# Verification — guardrails upgrade to 344faa3 (2026-10-15)

One record per change, written at `merge-change` step 6b and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line and carries the gate's figures in its own trailers; this
file is the rest of the evidence that travels with the change.

branch: worktree-guardrails-upgrade-344faa3
reviewer: independent subagent (one round, 2026-10-15) at the `review dispatched` commit, in its own task worktree, with the diff, the plan and the review checklist and no implementation narrative
verdict: round 1, verbatim: "pass. The only findings are the two records, so per the review checklist this is the last review round." It raised no code or requirement finding; both record findings are fixed.
reproduced: no defect in the software is fixed. The change moves tooling; before it, the installed skills (`merge-change` writing `Gate-tree:`, `develop-change` asking `guidelines-file.sh ARCHITECTURE`) ran ahead of the installed scripts, and the old `guidelines-file.sh ARCHITECTURE` exits 2. The old- and new-script outputs of `check-ids.sh` and `check-trace.sh` were captured on the same ledgers for all six units and compared byte for byte.

Units touched: org-members, on-chain-client, org-node, app, person, org-io.
Impact set (`check-units.sh --impact refs/heads/master..HEAD`): all six, since
the change edits the root `.guardrails/`.

Change: moves the six units' guardrails scripts, templates and AGENTS.md
managed block from upstream `6a9d6eb` to `344faa3` and records the tool
qualification. It implements, resolves and opens no item.
Plan: `docs/plans/2026-10-14-guardrails-upgrade-344faa3.md`. Qualification:
`docs/plans/2026-10-14-ratchet-setup-344faa3.md`.

## The gate

The gate's figures are not in this record. The squash commit carries them as
trailers: `Gate-tree:`, one `Gate:` line per unit and command, and `Base:`.

## Red → green

This change implements no item, so the table has no row and no test is
inherited.

| Item | Test | Watched red |
| --- | --- | --- |

## What was wrong, and what was built

The installed skills run at guardrails `344faa3` while
`.guardrails/scripts/` stood at `6a9d6eb`: `merge-change` step 7 writes a
`Gate-tree:` trailer that the installed `finish-merge.sh` did not check, and
`develop-change` runs a review per file `guidelines-file.sh ARCHITECTURE`
prints, which the installed script refused with exit 2.

Built: `check-signing.sh`, `finish-merge.sh` and `guidelines-file.sh`, the
new `ARCHITECTURE_GUIDELINES.md` and the revised `verification.md` template,
and the AGENTS.md managed block, all byte-identical to a clean `344faa3`
export; `guardrails_commit` moved in the six unit configs. On the same
ledgers the new `check-ids.sh` and `check-trace.sh` print exactly what the
old ones printed, for every unit, so the upgrade adds no failure and no
warning. No ledger README changed upstream, so none is touched.

## Review

### Round 1

**finding-1**: record — The plan's "Measured before the upgrade" paragraph says `check-units.sh` reports "728 → 729" tracked paths. That is true for the tree after the files were copied but before the two plan docs were added; at HEAD it reports 731, because the two new plan files count too. The plan should say the 729 was measured before the plan docs went in, or give 731 for HEAD.
disposition: the plan now says when 729 was measured and that the two documents bring it to 731, `docs/plans/2026-10-14-guardrails-upgrade-344faa3.md`.

**finding-2**: record — The plan does not record ratchet upgrade step 2 (`tailor-guidelines` in upgrade mode before copying). In this tree it had nothing to do: no unit has a `*_GUIDELINES.md` of its own, and `CODE_GUIDELINES.md` and `TEST_GUIDELINES.md` did not change upstream between 6a9d6eb and 344faa3. One line in Task 4 would close it.
disposition: Task 4 of `docs/plans/2026-10-14-guardrails-upgrade-344faa3.md` gains that line.

The reviewer ran `check-trace.sh` and `check-ids.sh` for every unit,
`check-units.sh`, and person's two `verify_commands` in its own worktree. For
the other units' product suites it relied on the step 2 gate on the same
tree, since the diff changes no product source, test or build file. This
record states that substitution.

## Guideline reviews

None ran. The diff authors no source, test or CI file: the files under
`.guardrails/` and the managed block are upstream copies this project never
modifies, so a finding there could be answered only upstream; the rest is
six config lines and two plan documents.

## Gaps

- Coverage against the class C target: every root `Makefile` floor is met,
  but on-chain-client's line and region figures, and org-io's line, region
  and branch figures, stay below the class target, as the `Makefile`
  documents above each floor (on-chain-client: client.rs's chain-facing
  paths; org-io: the chain-facing shell, SDD-z85ux9). These are the
  shortfalls the owner accepted earlier (org-io at S2's merge, on-chain-client
  before it). This change touches no product source or test, so it does not
  widen them, and no floor is lowered.
- The ARCHITECTURE guidelines are the installed default for every unit;
  tailoring them is a follow-up in the plan.
- `check-signing.sh --setup` cannot pass inside the agent sandbox; the
  owner's `finish-merge.sh` run is the proof of the signature and of the
  `Gate-tree:` trailer.
