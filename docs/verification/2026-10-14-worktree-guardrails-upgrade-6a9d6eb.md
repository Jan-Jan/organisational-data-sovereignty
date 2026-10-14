# Verification — guardrails upgrade to 6a9d6eb (2026-10-14)

One record per change, written at `merge-change` step 6b and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line, so this file is the evidence that travels with the change.

branch: worktree-guardrails-upgrade-6a9d6eb
reviewer: independent subagent (one round, 2026-10-14) at the `review dispatched` commit, tree `413cfbc1ba18ec6d1045742a7b96bf72dd73b18b`; its only code or requirement finding was low, so it was the last review round and the rerun ends at step 6b with no further reviewer
verdict: round 1, verbatim: "approve. The only `code`/`requirement` finding is low (finding-1), so this is the last review round. finding-2 should be fixed in the plan and record at 6b before 6c." All three findings are fixed.
reproduced: no defect in the software is fixed. The five findings the new scripts raised on master were reproduced by running the new `check-trace.sh` on the unchanged tree (four `OUT-OF-FORCE-VERIFIES`, one `ORPHAN-ANNOTATION`; the old scripts exit 0 there) and are gone on this tree. Review finding-1 (an inherited test that could not fail if `proxied` dropped its `call`) was reproduced by a planted `proxied` that put a fixed remark in place of `call`: the old assertions passed under it, the strengthened one failed.

Units touched: org-members, on-chain-client, org-node, app, person, org-io.
Impact set (`check-units.sh --impact refs/heads/master..HEAD`): all six, since
the change edits the root `.guardrails/`.

Change: moves the six units' guardrails scripts, templates, ledger READMEs and
AGENTS.md managed block from upstream `35570e8` to `6a9d6eb`, fixes the five
findings the stricter `check-trace.sh` raised, makes CI's `check-ids.sh` step
unconditional (the new script refuses `--allow-draft-files`), deletes
on-chain-client's unused `decode_proxy_delegates` by owner ruling, and records
the tool qualification. Branched from local `master` at `68ec432`, which
contains `origin/master` (`e41cb09`); local `master` was merged at every
step 1, after a successful fetch, because it is ahead of the unpushed remote.
Plan: `docs/plans/2026-10-14-guardrails-upgrade-6a9d6eb.md`. Qualification:
`docs/plans/2026-10-14-ratchet-setup-6a9d6eb.md`.

## The gate

Measured on: `fc2eb30` — tree `8c61aca5b354d4795dc7f44f1ac1ab7f311d23ae`, from
`git rev-parse HEAD^{tree}` on a clean worktree; step 3 found it unchanged, so
the rows below are the step 2 gate summary. Every command ran with
`CARGO_HOME=/tmp/cargo_home_fuzz`; org-members with a scratch `QUINT_HOME`
holding a copy of the working quint evaluator.

| Gate | Result |
| --- | --- |
| org-members `verify_commands` | pass: `cargo test -p org-members` 271 passed, 0 failed, 1 ignored (`preflight_probe`, invoked by another test by design); `quint --version` 0.33.0; three `quint typecheck` clean; `quint test membership.qnt` 63 passing; `quint run membership_mbt.qnt --invariant=mbtInv` no violation |
| on-chain-client `verify_commands` | pass: read lane 73 passed, 0 failed; write lane 55 passed, 0 failed; `fuzz_signatory_set` ran its budget with no panic |
| org-node `verify_commands` | pass: `cargo test` 307 passed, 0 failed; `quint --version` 0.33.0; two typechecks clean; five `quint run` invariants (forkSafety, revocationSafety, revokedExcludedFromOrgSecret, tauWindow, convergence) no violation |
| app `verify_commands` | pass: `cargo test` 164 passed, 0 failed; `npm run check` 201 files, 0 errors, 0 warnings; `npm run test` 51 of 51 |
| person `verify_commands` | pass: `cargo test` 123 passed, 0 failed; `clippy -D warnings` clean |
| org-io `verify_commands` | pass: test-support 104 passed, dev-seed 11 passed, 0 failed; `cargo build` and `clippy -D warnings` clean |
| `check-ids.sh` | exit 0 for all six units, no output |
| `check-trace.sh` | exit 0 for all six units. This change's own effect on the roll-call: the four `OUT-OF-FORCE-VERIFIES` and the `ORPHAN-ANNOTATION` the new script raised on master are gone; LLR-a9bb7b has no test, as an out-of-force item owes none |
| `check-units.sh` | exit 0 |
| Coverage, against the class target | every Makefile floor met, after `cargo llvm-cov clean --workspace` under one `CARGO_HOME`. org-members lines 96.55% (floor 92), regions 94.57% (91), branches 123/124 (98); person lines and regions 100% (99), branches 36/36 (99); on-chain-client lines 50.05% (46), regions 51.05% (47), branches 58/58 (99); org-io lines 77.38% (70), regions 76.39% (68), branches 49/62 (64). on-chain-client's and org-io's line and region figures stay below the class C target, as the root `Makefile` documents above each floor: owner-accepted shortfalls, not widened here. org-node and app have no `coverage_command` |
| Working tree | clean; the tree unchanged at the end of the run |

## Red → green

This change implements no item, so the table has no row.

| Item | Test | Watched red |
| --- | --- | --- |

inherited: proxied_wraps_any_call_unchanged_even_another_proxied_call — from LLR-ywhd23
inherited: an_org_information_body_with_trailing_bytes_is_refused — from LLR-js9dsu

The first inherited test was also strengthened at review (finding-1), and its
new assertion was watched red:
red -> green: proxied_wraps_any_call_unchanged_even_another_proxied_call — the strengthened assertion watched failing for the right reason (the outer `call` field was the planted remark), then green with the plant reverted

## What was wrong, and what was built

The installed skills ran at guardrails `6a9d6eb` while the project's
`.guardrails/scripts/` stood at `35570e8`: `develop-change` called
`guidelines-file.sh` and `merge-change` called `prune-plans.sh`, neither
present, and `merge-change` no longer renamed `DRAFT-` ledger files.

Measured before the change, on master `68ec432`: the old scripts exit 0 for
every unit; the new `check-trace.sh` fails on-chain-client and org-node with
four `OUT-OF-FORCE-VERIFIES` (tests whose `verifies:` named only superseded
LLRs: `signatory_set_decode.rs:401`, `:412`, `write_pure.rs:154`,
`wire_frame_bound.rs:130`) and one `ORPHAN-ANNOTATION`
(`org-node/docs/risk/2026-09-09-org-node-hazards.md:532`, prose wrapped so
`mitigates:` opened a line). It adds no new warning. CI's `check-ids.sh
--allow-draft-files` would exit 2 under the new script.

Built: the thirteen scripts, the three templates and the managed block,
byte-identical to a clean `6a9d6eb` export (`cmp`); `finalize-docs.sh`
deleted; the 24 ledger READMEs patched with exactly the template's own
old-to-new change; one unconditional CI `check-ids.sh` step;
`guardrails_commit` recorded in all six configs. Of the four tests, two are
pointed at their successor, LLR-59kxsr and LLR-tajh9d, which state the
behaviour they check at the interface they call (inherited above), and the
other two tested `decode_proxy_delegates`, which no in-force item described
and no production code called: by owner ruling the function, both tests and
its `fuzz_signatory_set` property are deleted, and LLR-n2vx5q stays covered
by its own tests. LLR-a9bb7b, LLR-ywhd23 and LLR-js9dsu gain dated notes.
The orphaned sentence is re-wrapped with no word changed.

## Review

### Round 1

**finding-1**: code, low — The inherited test `proxied_wraps_any_call_unchanged_even_another_proxied_call` (`on-chain-client/tests/write_pure.rs:154-161`) would not fail if `proxied` dropped or replaced its `call` argument with any fixed value: its `assert_eq!` only shows `proxied` is deterministic and its `assert_ne!` compares calls that already differ in `real`, so it does not verify LLR-59kxsr's "`call` the call given". Low because `proxied_forces_the_any_proxy_type_by_name` pins the full call value.
disposition: fixed in `on-chain-client/tests/write_pure.rs`: the test asserts the inner and the outer call against a hand-built `expected_proxied` value, so the outer `call` field must be the inner proxied call and that one's the remark; watched red against a planted `proxied` that put a fixed remark in place of `call`, then green with the plant reverted (`proxied_wraps_any_call_unchanged_even_another_proxied_call`).

**finding-2**: record — The plan says the two re-pointed tests are listed as `inherited:` in this change's verification record but has no literal lines for step 6b to copy, so LLR-js9dsu's note points at a record entry that does not exist.
disposition: the plan's Task 3 now carries both `inherited:` lines, copied above into `docs/verification/2026-10-14-worktree-guardrails-upgrade-6a9d6eb.md`, which is the record LLR-js9dsu's note names.

**finding-3**: record — `org-members/.guardrails/config.yaml:30` changes the comment "managed by the check/finalize scripts" to "the check scripts", which the plan's Task 5 does not mention.
disposition: the plan's Task 5 (`docs/plans/2026-10-14-guardrails-upgrade-6a9d6eb.md`) now names the comment edit; recorded here in `docs/verification/2026-10-14-worktree-guardrails-upgrade-6a9d6eb.md`.

## Guideline reviews

The files copied from guardrails (`.guardrails/scripts/`,
`.guardrails/templates/`, the managed block, the README grammar headers) were
not reviewed against the guidelines: this project never modifies them, so a
finding there could be answered only upstream. Each review covered the paths
this change authored outside the ledgers.

### .guardrails/templates/TEST_GUIDELINES.md

Governs: on-chain-client/tests/signatory_set_decode.rs, on-chain-client/tests/write_pure.rs, on-chain-client/tests/fuzz_signatory_set/fuzz_target.rs (round 2), org-node/tests/wire_frame_bound.rs

### Round 1

**finding-1**: guideline, medium — Seams 1, "A test calls only an interface that a REQ or LLR describes" — on-chain-client/tests/signatory_set_decode.rs:401
disposition: fixed by deleting the interface and the test, by owner ruling: `decode_proxy_delegates` is gone from `on-chain-client/src/write/signatory_set.rs`, and LLR-a9bb7b's note records it.

**finding-2**: guideline, medium — Seams 1, "A test calls only an interface that a REQ or LLR describes" — on-chain-client/tests/signatory_set_decode.rs:412
disposition: fixed with finding-1: the test is deleted from `on-chain-client/tests/signatory_set_decode.rs`; LLR-n2vx5q's refusals stay asserted by `a_malformed_definitions_value_is_refused`.

### Round 2

No findings.

### .guardrails/templates/CODE_GUIDELINES.md

Governs: .github/workflows/rust.yml, on-chain-client/src/write/signatory_set.rs (round 2), on-chain-client/tests/signatory_set_decode.rs, on-chain-client/tests/write_pure.rs, on-chain-client/tests/fuzz_signatory_set/fuzz_target.rs (round 2), org-node/tests/wire_frame_bound.rs

### Round 1

**finding-1**: guideline, medium — Comments 2, "No change narration: A comment describes the code as it is, never the change that wrote it" — .github/workflows/rust.yml:257-261
disposition: fixed in `.github/workflows/rust.yml`: the comment states only the reason (a ledger file is never renamed, so a DRAFT-named file is a defect on a pull request and on a push alike; `check-ids.sh` refuses `--allow-draft-files`).

### Round 2

No findings.

## Gaps

- Coverage: on-chain-client and org-io meet their floors, but their line and
  region figures stay below the class C target, as the root `Makefile`
  documents above each floor: owner-accepted shortfalls, not widened here.
  org-node and app have no `coverage_command`; this change does not add one.
- The guidelines conformance reviews did not cover the vendored guardrails
  files (above).
- The inherited `an_org_information_body_with_trailing_bytes_is_refused` has
  no red-first evidence against LLR-tajh9d; it is coverage, and LLR-tajh9d's
  own red-first test is
  `exactly_three_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused`.
- The suite qualification ran the whole suite on a clean archive and
  `mutations.bats` on a clean clone, not the whole suite on one clean clone.
- The in-flight S4 change (`worktree-org-io-transport`) holds fifteen
  `DRAFT-` ledger files it must rename before it merges; this change does not
  touch that worktree.
