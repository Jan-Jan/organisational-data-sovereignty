# Guardrails upgrade to 6a9d6eb — 2026-10-14

**Goal.** Move the six units from the guardrails scripts at `35570e8` to
`6a9d6eb` (23 upstream commits), following `ratchet`'s "Upgrading the
scripts" path and `references/upgrade-notes.md`. No requirement, hazard or
design item changes. One production function goes: on-chain-client's
`decode_proxy_delegates`, which no in-force item described and no
production code called (Task 3). The upstream template still reads
`guardrails_version: 0.5.1`; `guardrails_commit` is what moves.

**Why now.** The installed skills already run at `6a9d6eb`: `develop-change`
dispatches `develop-tdd` and runs `review-conformance` per file that
`guidelines-file.sh` prints, `merge-change` runs `prune-plans.sh` and has no
finalize step, and both scripts were missing from `.guardrails/scripts/`.
Until this merges, those skill steps exit 127 or 2 here.

## Measured before the upgrade

On master `68ec432` with the old scripts: `check-units.sh` exit 0
(6 units, 6 disclaimed, 722 tracked paths) and `check-trace.sh` exit 0 for
every unit. No `DRAFT-` file and no `REQ-DRAFT-` token on master. The
list-item annotation grep of `upgrade-notes.md` prints nothing.

With the new scripts on the same tree, `check-ids.sh` and `check-units.sh`
exit 0, and `check-trace.sh` fails two units with five new findings:

- `OUT-OF-FORCE-VERIFIES` — on-chain-client `tests/signatory_set_decode.rs:401`
  and `:412` (LLR-a9bb7b), `tests/write_pure.rs:154` (LLR-ywhd23); org-node
  `tests/wire_frame_bound.rs:130` (LLR-js9dsu).
- `ORPHAN-ANNOTATION` — `org-node/docs/risk/2026-09-09-org-node-hazards.md:532`,
  a prose sentence wrapped so that `mitigates:` opened the line, after the
  block of RC-b6mydy had closed at the bold `**Residual risk: …**` line.

Every other finding (`UNRESOLVED-PR`, `UNMET-EXPECTATION`) is printed by the
old scripts too, so this upgrade adds **no new warning**.

## Tasks

### Task 1 — copy the machinery (done)

`scripts/*.sh` into `.guardrails/scripts/` (adds `guidelines-file.sh` and
`prune-plans.sh`); `finalize-docs.sh` deleted (`finalize-ids.sh` was already
gone). `templates/CODE_GUIDELINES.md`, `TEST_GUIDELINES.md` and
`verification.md` into `.guardrails/templates/`. The four ledger READMEs of
each unit patched three-way (base: the template at `35570e8`, theirs: at
`6a9d6eb`, ours: the unit's copy): upstream changed only the opening
paragraph and the grammar comment, the project's additions (the risk
matrices, the architecture overviews) lie outside both, and all 24 patches
applied cleanly. The AGENTS.md managed block replaced between its markers; it
was byte-identical to the `35570e8` template before.

### Task 2 — CI (done)

`.github/workflows/rust.yml` called `check-ids.sh --allow-draft-files` on
pull requests, which the new `check-ids.sh` refuses with exit 2. The two
conditional steps are one unconditional `check-ids.sh` step: a ledger file is
named the day its change creates it, so a `DRAFT-` file is a defect on a pull
request as on master.

### Task 3 — the five findings (done)

- Two tests are pointed at the item in force, keeping the old ID beside it as
  this project's dual-ID notes do, and listed as `inherited:` in this
  change's verification record. Each successor states the behavior the test
  checks at the interface the test calls: LLR-59kxsr keeps "`call` the call
  given" for `proxied_wraps_any_call_unchanged_even_another_proxied_call`
  (`proxied`), and LLR-tajh9d keeps the leftover-bytes refusal for
  `an_org_information_body_with_trailing_bytes_is_refused` (`decode_body`).
  The project's earlier rule that a green test is never re-annotated is the
  skills' rule too, with this one exception now written into `develop-tdd`
  and `verify-before-merge`: an inherited test is coverage, never red-first
  evidence. LLR-js9dsu and LLR-ywhd23 each gain a dated note saying so.

  ```
  inherited: proxied_wraps_any_call_unchanged_even_another_proxied_call — from LLR-ywhd23
  inherited: an_org_information_body_with_trailing_bytes_is_refused — from LLR-js9dsu
  ```

  Review round 1 (finding-1, low) found the first of these could not fail if
  `proxied` dropped its `call`. Its assertions were strengthened against a
  hand-built expected value (task `proxied-call`), and the new assertion was
  watched failing against a planted `proxied` that put a fixed remark in
  place of `call`, while the old assertions alone passed under that plant:

  ```
  red -> green: proxied_wraps_any_call_unchanged_even_another_proxied_call — the strengthened assertion watched failing for the right reason (outer `call` field was the planted remark), then green with the plant reverted
  ```
- The other two, `proxy_definitions_give_their_delegates_in_order` and
  `truncated_padded_or_overlong_definitions_are_malformed`, called
  `decode_proxy_delegates`, which only the superseded LLR-a9bb7b describes;
  no in-force item does, and no production code called it. By the owner's
  ruling (2026-10-14) the function, the two tests and its property in
  `fuzz_signatory_set` are deleted, and LLR-a9bb7b gains a dated note.
  LLR-n2vx5q stays covered at its own interface by
  `definitions_decode_with_type_and_delay_in_stored_order`,
  `a_malformed_definitions_value_is_refused` and `fuzz_signatory_set`.
- The orphaned line is re-wrapped so the sentence no longer opens a line
  with `mitigates:`. No word changes.

### Task 4 — the qualification record (done: `docs/plans/2026-10-14-ratchet-setup-6a9d6eb.md`)

### Task 5 — config (done)

`guardrails_commit` set to `6a9d6eb8507752fa4e52fc7519740f69c9113ffe` in all
six unit configs, and the comment above org-members' `id_prefixes` no longer
names the finalize scripts, which are gone. `prune_plans` and `review_cite`
are left absent: pruning is
on by default, and `review_cite` cannot be read in a manifest repository.

### Task 6 — guideline conformance reviews (done)

`guidelines-file.sh TEST` and `CODE` print the installed defaults,
`.guardrails/templates/TEST_GUIDELINES.md` and `CODE_GUIDELINES.md`, for
every path: no unit has a file of its own. Each was reviewed under
`review-conformance` for the paths this change authored outside the
ledgers: TEST for the three edited test files, CODE for those and
`.github/workflows/rust.yml`. The files copied from guardrails
(`.guardrails/scripts/`, `.guardrails/templates/`, the managed block, the
README grammar headers) were not reviewed: this project never modifies
them, so a finding there could be answered only upstream.

Round 1, TEST: 2 findings, both medium.

- finding-1 (Seams 1): `proxy_definitions_give_their_delegates_in_order`,
  re-pointed at LLR-n2vx5q, calls `decode_proxy_delegates`, an interface
  LLR-n2vx5q does not describe. Disposition: fixed by deleting the function
  and the test (owner ruling; Task 3), task `drop-delegates`.
- finding-2 (Seams 1): the same for
  `truncated_padded_or_overlong_definitions_are_malformed`. Disposition:
  fixed in the same task.

Round 1, CODE: 1 finding, medium.

- finding-1 (Comments 2): the new `rust.yml` comment above the `check-ids.sh`
  step narrated the upgrade ("Since guardrails 6a9d6eb", "finalize-docs.sh
  is gone"). Disposition: fixed, task `fix-comment`; the comment now states
  only the reason.

Round 2, on the fixed branch, with the deleted function's files added to
both scopes (`on-chain-client/src/write/signatory_set.rs` to CODE,
`fuzz_signatory_set/fuzz_target.rs` to both): TEST 0 findings, CODE 0
findings. The TEST reviewer noted, as no finding, that the inherited
`proxied_wraps_any_call_unchanged_even_another_proxied_call` asserts
neither `Some(Any)` nor that the inner call is carried unchanged, so it is
weak evidence for LLR-59kxsr as it was for LLR-ywhd23; its siblings
`proxied_forces_the_any_proxy_type_by_name` and
`proxied_never_leaves_the_proxy_type_unforced_or_forced_by_index` cover
what it misses.

## Follow-ups (not this change)

- **The in-flight S4 change must rename its drafts.** The
  `worktree-org-io-transport` worktree holds fifteen `DRAFT-` ledger files
  (org-node, org-io, app). If this upgrade merges first, that change runs
  `git mv <dir>/DRAFT-<branch>-<slug>.md <dir>/<YYYY-MM-DD>-<slug>.md`, dated
  the day each file was created, before it merges, and fixes each
  `DANGLING-FILE` the new names reveal; `merge-change` no longer renames them.
- **Tailor the guidelines.** No unit has a `docs/TEST_GUIDELINES.md` or
  `docs/CODE_GUIDELINES.md`, so every unit follows the installed defaults,
  which state the rules `develop-change` stated before. Run
  `tailor-guidelines` for `TEST` and `CODE` to make them the project's own.
- **Prune the merged plans** with `prune-plans.sh --all --dry-run`, then
  without `--dry-run`, as a change of its own, if the owner wants it.
- **Upstream: an unpinned qualification run.** `run-tests.sh` tests the live
  checkout, and `ratchet` step 5 records `git rev-parse HEAD` beside the
  result, so an upstream commit during the run makes the result describe
  no commit (the correction in the qualification record). To report
  upstream, with two `ratchet` papercuts: step 5's `status=$?` fails in zsh, and
  step 0's path assumes linked, not copied, skills.
