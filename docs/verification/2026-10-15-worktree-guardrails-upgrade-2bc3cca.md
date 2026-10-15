# Verification — guardrails upgrade to 2bc3cca (2026-10-15)

One record per change, written at `merge-change` step 5 and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line and carries the gate's figures in its own trailers; this
file is the rest of the evidence that travels with the change.

branch: worktree-guardrails-upgrade-2bc3cca
reviewer: independent subagent (one round, 2026-10-15) at the `review dispatched` commit, in its own task worktree, with the diff, the plan, the qualification record and the review checklist and no implementation narrative
verdict: round 1, verbatim: "The change can merge. Every copied file is byte-identical to upstream 2bc3cca, the upgrade covers every ratchet "Upgrading the scripts" step and every upgrade-notes item for 344faa3 → 2bc3cca, all gates exit 0, and nothing in the diff goes beyond the plan. There is one `record` finding and no `code` or `requirement` findings." The record finding is fixed.
reproduced: no defect in the software is fixed. The change moves tooling; before it, the installed skills ran the renumbered `merge-change` steps (gate at 6a, after the record at 5) while the installed scripts, the verification template and the AGENTS.md managed block named the old numbers. The old- and new-script outputs of `check-units.sh`, `check-ids.sh` and `check-trace.sh` were captured on the same ledgers for all six units and compared byte for byte.

Units touched: org-members, on-chain-client, org-node, app, person, org-io.
Impact set (`check-units.sh --impact refs/heads/master..HEAD`): all six, since
the change edits the root `.guardrails/`.

Change: moves the six units' guardrails scripts, templates and AGENTS.md
managed block from upstream `344faa3` to `2bc3cca` and records the tool
qualification. It implements, resolves and opens no item.
Plan: `docs/plans/2026-10-15-guardrails-upgrade-2bc3cca.md`. Qualification:
`docs/plans/2026-10-15-ratchet-setup-2bc3cca.md`.

## The gate

The gate's figures are not in this record. The squash commit carries them as
trailers: `Gate-tree:`, one `Gate:` line per unit and command, and `Base:`.

## Red → green

This change implements no item, so the table has no row and no test is
inherited.

| Item | Test | Watched red |
| --- | --- | --- |

## What was wrong, and what was built

The installed skills run at guardrails `2bc3cca` while `.guardrails/scripts/`
stood at `344faa3`: `merge-change` gates after the record and numbers its
steps 1, 2, 3, 4, 5, 6a–6d, 7, 8, but the installed scripts' messages, the
verification template and the AGENTS.md managed block sent a reader to the
old step numbers (the record "at step 6b", the pre-flight "at step 4"), and
the installed `TEST_GUIDELINES.md` default lacked the `## Test runs` clause
`develop-tdd` now reads.

Built: `check-ids.sh`, `check-review.sh`, `check-trace.sh`,
`finish-merge.sh`, `merge-preflight.sh` and `prune-plans.sh` (comments and
messages only), the revised `TEST_GUIDELINES.md` and `verification.md`
templates, and the AGENTS.md managed block, all byte-identical to upstream
`2bc3cca`; `guardrails_commit` moved in the six unit configs. On the same
ledgers the new check scripts print exactly what the old ones printed, for
every unit, so the upgrade adds no failure and no warning. No ledger README
changed upstream, so none is touched.

## Review

### Round 1

**finding-1**: record — `docs/plans/2026-10-15-ratchet-setup-2bc3cca.md` puts the qualifying log at `<tmp>/ratchet.log`, "beside the clone", and never says where it is kept. Ratchet step 5 says "Keep the log, outside the clone", and `references/rationale.md` says the log "is kept when the clone is removed" because it is the evidence for the recorded result. I found the log at `/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/a57e8c1e-7090-422f-a37e-ee4d8fca3a4d/scratchpad/ratchet/ratchet.log`. Its contents match the record: `1..1494`, 1494 ok, 0 not ok, and a single `run-tests:` line, `run-tests: testing commit 2bc3cca04b75e1205d1c79a8e7b40c302036bf6f`. But that is a temporary session scratchpad. The record should name a durable place for the log, or say plainly that the log is not kept beyond the session. The 344faa3 record has the same gap.
disposition: the qualification record now says plainly that the log lives in the session scratchpad and is not kept beyond the session, quotes its figures, and says how to re-qualify, `docs/plans/2026-10-15-ratchet-setup-2bc3cca.md`. The `344faa3` record is merged history and is left as written.

The reviewer ran `check-units.sh`, and `check-ids.sh` and `check-trace.sh`
for every unit, in its own worktree, all exit 0. It skipped the product
suites, since the diff changes no product source, test or build file, and
relied on the step 6a gate on the same tree. This record states that
substitution.

## Guideline reviews

None ran. The diff authors no source, test or CI file: the files under
`.guardrails/` and the managed block are upstream copies this project never
modifies, so a finding there could be answered only upstream; the rest is
six config lines and two plan documents.

## Gaps

- Coverage against the class C target: every root `Makefile` floor is met,
  but on-chain-client's line and region figures, and org-io's line, region
  and branch figures, stay below the class target, as the `Makefile`
  documents above each floor. These are the shortfalls the owner accepted
  earlier. This change touches no product source or test, so it does not
  widen them, and no floor is lowered.
- The baseline at `worktree-discipline` step 4 needed three environment
  fixes before it was green (a populated `QUINT_HOME`, a writable
  `CARGO_HOME` for the on-chain-client and app lockfiles, and `app/node_modules`
  in the new worktree), as the plan records. None is a change to the tree.
- The qualification log is not kept beyond the agent session; its figures
  are in the qualification record.
- `check-signing.sh --setup` cannot pass inside the agent sandbox (gpg
  signing times out); the owner's `finish-merge.sh` run is the proof of the
  signature and of the `Gate-tree:` trailer.
