# Verification — guardrails toolkit upgrade 0.4.0 → 0.5.1 (2026-09-01)

> **Corrected 2026-09-02, body left unedited.** This record states that
> `check-signing --strict` "has never passed here" and presents it as measured.
> The measurement was real but its scope was not: every such run had been taken
> inside an agent sandbox that cannot open the GPG trustdb, where a valid
> signature reads as unverifiable, and an independent reviewer's confirmation
> ran in the same sandbox — one blind spot counted twice. On the owner's own
> terminal `--strict` passes and `--setup` exits 0. The correction is recorded
> in `docs/plans/2026-08-26-ratchet-setup.md`. The body below is left as
> written, because a verification record is the evidence for a completed
> change; this pointer exists so a reader meets the correction before the
> claim, the same treatment the superseded Class B ADR was given.


branch: worktree-guardrails-051
reviewer: fresh general-purpose subagent, dispatched at merge-change step 6a with the diff, the upstream source tree for comparison, and the SKILL.md upgrade procedure as acceptance criteria — no implementation narrative and no session history
verdict: fit to merge with conditions, 11 findings. The mechanical upgrade was found clean on every check that mattered — scripts byte-identical to upstream, correct modes, managed block replaced within its markers and nothing else. What it found was prose the upgrade itself falsified: four documents describing a toolkit that no longer behaves that way. Conditions met.
reproduced: yes, for every finding acted on. `diff docs/problems/README.md` against the upstream template exits 1 while the other three ledger READMEs exit 0, so the drift was isolated rather than assumed. The stale config sentence was read at its line. The `check-signing --setup` result and its worktree explanation were reproduced by reading `git config --show-origin --get commit.gpgsign` inside the worktree. The one thing NOT reproduced by me is the reviewer's `--strict` run over the three existing merges (three `UNVERIFIED`, exit 1) — my sandbox cannot reach the GPG trustdb, so that figure is the reviewer's measurement, and it is marked as such wherever it is quoted.

Change: re-ratchet. Vendored toolkit upgraded from 0.4.0 to 0.5.1, which brings
`finish-merge.sh`, the macOS awk/sed/date fixes, five new fatal config-schema
rules, and the removal of `owner:` from the problem-report grammar. Branched
from `master` at `067e56c`.
Plan: `docs/plans/2026-08-26-ratchet-gap-analysis.md`, and the ratchet skill's
own "Upgrading the scripts in an existing project" procedure.

## The gate

| Gate | Result |
| --- | --- |
| `cargo test -p org-members` | `integration_test` 102 passed, `fuzz_tests` 6 passed, 0 failed. `mbt_conformance` not run here (sandbox cannot give quint a writable `~/.quint`); unchanged by this commit, which touches no Rust. |
| `cargo test … on-chain-client --lib + 3 fuzz targets` | Untouched by this change. |
| `quint typecheck` ×2 | exit 0, exit 0 |
| `make coverage` | exit 0, figures unchanged from baseline |
| `check-ids.sh` | exit 0 |
| `check-trace.sh` | exit 0 — `REQ 12, PR 1`; `UNRESOLVED-PR PR-zz4exm (open 1 days)`, note the 0.5.1 warning no longer prints an owner |
| `check-review.sh --branch worktree-guardrails-051` | **Ran locally, for the first time in this project.** See below. |
| Tool qualification suite | 435 tests, 435 ok, 0 not ok, exit 0, at upstream `bb7eee5`. Independently re-run by the reviewer with the same result. |
| Working tree | clean |

**Step 6c executed on this machine.** Under 0.4.0 it exited 2 on every
invocation and the first three records were hand-checked instead. That is the
substantive difference this upgrade makes to the process, and it showed
immediately: the reviewer's run against this branch returned `MISSING-RECORD`
before this file existed — the gate correctly refusing to pass over a change
with no record, which is precisely what it could never do here before.

## What was wrong, and what was built

The vendored toolkit was three months of upstream development behind, and one
of its scripts could not run at all on this platform. Specifically:
`check-review.sh` handed `awk -v` a value carrying literal newlines, which the
BWK awk shipped with macOS rejects outright — so `merge-change` step 6c exited
2 for the first three changes of this project, and exit 2 reads as a *setup*
error, which sent me looking at our own configuration rather than at the tool.
The defect was reported upstream from here, fixed in `31e2303`, and the suite
now carries tests for it.

What this change installs is the fix, plus `finish-merge.sh`, plus five
config-schema rules that each catch a key not taking effect as written. It also
records a tool-qualification result for the first time: the 0.4.0 install
recorded none, which is how a broken verification tool reached this repository
and stayed for three changes.

The upgrade's semantic change — `owner:` removed from problem reports — is
where the findings clustered. Removing a field from a grammar falsifies every
document that taught it, and this change initially updated one of the four.

## Review

**finding-1**: `check-review.sh --branch worktree-guardrails-051` exits 1 with `MISSING-RECORD`, because step 6b had not yet run. Recorded as fact rather than as a defect in the diff.
disposition: This file is that record. Re-run after writing it; result in the gate table above.

**finding-2**: The setup document claimed the suite "now carries five regression tests" for the awk defect. No reading of `tests/portability.bats` yields five: it holds nine tests, six on the awk-newline rule and three on `date` and `sed`.
disposition: Corrected to the measured counts. A number stated as measured must be measured; I had estimated it.

**finding-3**: "Four signed merges on `master`" listed three that exist and one that had not happened, under a heading reading "What is already in place".
disposition: Corrected to three, with this change named as the fourth-to-be.

**finding-4**: The CI item referred to "the `check-signing --strict` step in CI". There is no such step — the workflow runs `check-signing.sh` unflagged on push to master.
disposition: Reworded to the conditional ("a `--strict` step in CI *would be* a backstop"). The item already said two lines later that it currently runs non-strict; now the first sentence agrees with the second.

**finding-5**: `docs/problems/README.md` still taught the 0.4.0 grammar — `owner:` in the item block, "every OPEN item an `owner:` and an `opened:` date", and warnings "carrying their age and owner" — while `AGENTS.md` in the same commit says problem reports carry no owner, and the actual warning prints `(open 1 days)`. The only ledger README that had drifted; the other three diff clean against upstream. Upstream ships a test for exactly this case.
disposition: Replaced with the upstream template, now byte-identical. This was the most consequential finding: the item grammar is the document a future author reads before writing a problem report, and it would have taught them a field the toolkit no longer defines. That upstream test — "problem grammar prose: no shipped file still carries owner:" — passes upstream, and this repository was the case it was written to prevent.

**finding-6**: The gap analysis's tooth 8 instructed a future backfill to write PR items with `owner:`/`opened:`/`status:`.
disposition: Corrected, with a parenthesis recording that the field existed until 2026-09-01 and why it went. A forward-looking instruction against a removed field is the same defect as finding 5, one step further out in time.

**finding-7**: The gap analysis still listed Signing as tooth 2, which SKILL.md Step 3.4 now explicitly forbids — signing is a prerequisite the first merge enforces, not a scheduled tooth. The setup checklist had been rewritten to lead with that; the adoption order it cross-references still contradicted it.
disposition: Reclassified in place rather than deleted, so the history of the decision survives: the row now says it was a tooth, that 0.5.1 moved it out of the adoption order, and why a merge-time gate is not something you schedule.

**finding-8**: `.github/workflows/rust.yml` asserted that `check-review.sh` "cannot run on macOS at all" and named "the upstream awk fix" as one of the things that would close the gap. That fix is the substance of this very commit.
disposition: Rewritten as history — why the step was added, that 0.5.1 fixed it, and what actually remains: step 6c covers the gate on the merging machine now, and CI provides no *independent* backstop only because this project merges without pull requests.

**finding-9**: `.guardrails/config.yaml`'s header comment still described the 0.4.0 schema, including "a duplicate key … is accepted and the later one ignored", which 0.5.1 rejects, and predated four other new fatal rules.
disposition: The whole file regenerated from the 0.5.1 template with this project's settings re-applied and its own coverage-gate prose preserved, rather than patching the one sentence. Patching would have left a 0.4.0 document with one 0.5.1 sentence in it. A script performed the regeneration and asserted afterwards that all fourteen settings survived.

**finding-10**: SKILL.md Step 5's precondition is unmet for this change itself — `check-signing.sh --setup` exits 1, so `finish-merge.sh` guard 1 will refuse cleanup after the merge lands.
disposition: Accepted as a disclosed deviation, not fixed here, because it cannot be fixed here: `--setup` needs `gpg.format` set and must run from the primary checkout, and both are the user's machine rather than this worktree. The setup document leads with it and does not claim adoption is complete. See Gaps — this is the one open condition on the merge.

**finding-11**: CI runs `check-signing.sh` non-strict and on HEAD alone, where Step 5 asks for `--strict <base>..HEAD`. Pre-existing from `4bb5509`, reasoned in the workflow comments, flagged in the setup document.
disposition: Not changed here. `--strict` in CI cannot pass until a verification root exists (the same blocker as finding 10), so tightening it now would redden `master` on every push. Recorded in Gaps as conditional on the signing items.

## Gaps

- **The signing chain is unproved, and it now gates cleanup.**
  `check-signing.sh --setup` exits 1: `gpg.format` is not set explicitly, and
  `--setup` must be run from the primary checkout rather than a worktree, where
  this project deliberately sets `commit.gpgsign=false`. Until it exits 0,
  `finish-merge.sh` will land the squash and then refuse to remove the worktree
  and delete the branch. That is a disclosed, expected outcome of merging this
  change, not a surprise to diagnose afterwards.
- **`--strict` has never passed here.** Measured by the reviewer over the three
  existing merges: three `UNVERIFIED`, exit 1. The signatures are real; nothing
  on this machine can verify them, because there is no allowed_signers file and
  the GPG keyring cannot verify the committer's own key.
- **`check-review.sh` provides no CI backstop**, because this project merges
  locally without pull requests. Step 6c now covers it on the merging machine,
  which is new, but a second pair of eyes in CI would need PRs.
- **The upstream basis moved during the change.** The recorded qualification
  basis is `bb7eee5`; upstream's working copy is one commit further on at
  `f64be47`. The reviewer confirmed `bb7eee5..f64be47` touches documentation
  only — `scripts/`, `templates/` and `tests/` are identical — so the recorded
  basis and the vendored bytes agree. Worth knowing that the basis is a moving
  target between ratchets.
- **`PR-zz4exm` keeps a vestigial `owner:` line.** Ignored rather than rejected
  by the 0.5.1 gates. Left in place: editing a merged ledger item for tidiness
  is worse than a line that no longer does anything, and it will go when the
  problem is resolved.
- **Nothing about the project's own requirements, risk or architecture changed
  here.** This is toolchain work. Teeth 5 and 6 remain where they were.
