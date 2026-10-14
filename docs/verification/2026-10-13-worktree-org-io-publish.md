# Verification — S3b-io part 1, the publish path (2026-10-13)

One record per change, written at `merge-change` step 6b and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line, so this file is the evidence that travels with the change.

branch: worktree-org-io-publish
reviewer: independent subagent (two rounds, 2026-10-13) — round 1 at `fb19b14` (tree `9f55468837bfd982237a984ee1ba6024c9f92c4e`), round 2 at `5b5a67f` (after the merge `3350f41` of the round-1 fixes); round 2 was the last round (all its code and requirement findings low), so the rerun ends at step 6b with no further reviewer
verdict: round 2, verbatim: "approve — every round-1 finding is resolved or booked, and all five new findings are low or record. By the checklist rule, a round with only low code/requirement findings is the last review round." Round 1 was "changes requested — two medium findings (one code, one requirement); every other is low or record": an executed unknown write the reconcile never sent or cleared (finding-1, fixed) and REQ-dqbg9p's "never" contradicted by the founding clause (finding-2, narrowed by owner ruling with a new-proxy test); of its six low or record findings, five were fixed and one booked (PR-6nhrr7). Of round 2's five, four were fixed and one booked by owner ruling (PR-jgvf3b).
reproduced: PR-924ftr — reproduced by `org-io/tests/publish_path.rs` `a_write_that_lands_after_its_timeout_is_committed_and_sent_to_its_own_recipient` (T14): compile-red before the settle existed, then watched failing with the settle step disabled, i.e. S2's behaviour (D's write went out, "epoch mismatch: expected 1, found 2", and C's update was never committed); green after. PR-wk7zwq — reproduced by `org-io/tests/signatory_rule.rs` `a_restricted_or_delayed_delegate_cannot_dispatch` (T16a): compile-red, then runtime-red against the superseded any-delegate rule (`Admin` for `(controller, NonTransfer, 0)` and `(controller, Any, 10)`). PR-ksm2ua — reproduced by `on-chain-client/tests/signatory_set_decode.rs` `the_signatory_set_is_read_at_one_block` (T16a): runtime-red with no `signatory_set` in client.rs, then red against a planted two-block read. Round 2's finding-3 (a reconcile that fails to save forgets the late write's send) was reproduced by a probe test (store file read-only during `reconcile`) and booked as PR-jgvf3b, with the probe as its reproduction recipe. Every review finding fixed in code was watched red before its fix; each red is in that finding's disposition below.

Units touched: org-io, org-node, app, on-chain-client. Impact set
(`check-units.sh --impact e41cb09..HEAD`): on-chain-client, org-node, app,
org-io.

Change: S3b-io part 1 of the org-io roadmap. org-io settles every chain write
it submits whose outcome is unknown (it reads the chain until the chain
decides, writes nothing else for that Organisation meanwhile, and commits a
late write and sends it to its own submission's recipient); it reconciles
every held Organisation with the chain at start and on request, marking a
device that cannot read the chain unverified and committing or deleting
nothing then; it discards provisional updates in the background; and it
deletes nothing except on org-node's verified paths. on-chain-client makes
every update write mortal (64 blocks) and reports its window. With it, T16a:
"admin" is a `(controller, Any, 0)` proxy definition, read at one finalised
block, and the writer forces `Any`. The app reconciles at start and on
request and shows the unverified state; it offers no discard. Branched from
`master` at `e41cb09` (merge base).
Plan: `docs/plans/2026-10-08-org-io-publish.md`.

## The gate

Every figure below is measured on one tree, and the table names it, so a later
reader can re-measure the same tree instead of guessing which round produced
these numbers. **No figure here is copied forward from an earlier round.**

Measured on: `3f2fd20` — `git rev-parse HEAD` — tree
`2903519929dd9eb0c9cec73e3682f4041eb0d5eb`, from `git rev-parse HEAD^{tree}`
on a clean worktree (final gate, run 6, 2026-10-13; verdict PASS). Agent
sandbox, `CARGO_HOME=/tmp/cargo_home_fuzz`,
`QUINT_HOME=/private/tmp/claude-501/quint_home`,
`CARGO_TARGET_DIR=/private/tmp/claude-501/target-gate2`; `app/node_modules`
symlinked for npm and removed before the working-tree check.

| Gate | Result |
| --- | --- |
| org-io `cargo test -p org-io --features test-support …` (the config's first line, with `publish_path` and `startup_reconcile`) | pass: 104 passed, 0 failed, 0 ignored (absences 14, app_boundary 3, chain_read_rule 3, key_custody_config 5, open 4, publish_path 26, receive_order 12, signatory_key 6, signatory_rule 10, startup_reconcile 6, submit_flow 15); `fuzz_seed_parse` 1 s ok |
| org-io `cargo test -p org-io --features test-support,dev-seed …` | pass: 11 passed, 0 failed |
| org-io `cargo build -p org-io`; clippy `-D warnings` | pass: build ok; clippy clean |
| org-node `cargo test` and Quint | pass: 307 passed, 0 failed, 0 ignored (admission_sender 52, commit_paths 39, verify_against_chain 24, revocation 20, …); 4 fuzz targets ok; quint 0.33.0, two typechecks ok; forkSafety, revocationSafety, revokedExcludedFromOrgSecret, tauWindow, convergence each "[ok] No violation found" |
| on-chain-client, both test lines | pass: line 1 73 passed + 3 fuzz targets; write line 57 passed (write_pure 22, write_events 12, signatory_set_decode 12, write_compose 9, write_manifest 2) + `fuzz_signatory_set` |
| app `cargo test`; `npm run check`; `npm run test`; release-shape `cargo check` | pass: 164 passed (ipc 36, receiver_events 35, csp_policy 24, invitation 17, org_id_parsing 14, startup_policy 13, connection_status 10, receiver_guard 6, state_assembly 4, reconcile 3, submit_flow 2); svelte-check 201 files, 0 errors, 0 warnings; vitest 8 files, 51 passed; release-shape check ok |
| `check-ids.sh` (without `--allow-draft-files`), all six units | pass: exit 0 |
| `check-trace.sh`, all six units | pass: exit 0, advisories only. This change's IDs entering the roll-call: org-io REQ-by9k5q, REQ-77rh5a, REQ-dqbg9p, REQ-cz753a, REQ-ntkjk6, SDD-6mpmsw, LLR-2yjuaj, LLR-t98pk9, LLR-nqvbb6, LLR-fptp57, LLR-36zp69, LLR-k5pzsz, LLR-kp95ty, LLR-cc59ys, LLR-2hkqv2; on-chain-client REQ-4aq8zh, LLR-zfm3bf, LLR-zj4jju, REQ-b8x2py, LLR-n2vx5q, LLR-59kxsr, LLR-sc79gq; org-node LLR-pk6mhv, REQ-and2gz (an `expects: person` item, UNMET-EXPECTATION advisory); app REQ-363366, LLR-ekz4zn, LLR-8axx7h, LLR-hwcj9n. Leaving the open-problem roll-call: PR-924ftr, PR-wk7zwq, PR-ksm2ua. Entering it: PR-6nhrr7 (org-io), PR-jgvf3b (org-node). Every claimed LLR has `verifies:` tests (2yjuaj 3, t98pk9 14, nqvbb6 6, fptp57 5, 36zp69 1, k5pzsz 6, kp95ty 1, cc59ys 3, 2hkqv2 5, a2acvh 2, ekz4zn 5, 8axx7h 4, hwcj9n 7, zfm3bf 3, zj4jju 3, 59kxsr 4, n2vx5q 3, sc79gq 6, pk6mhv 3); REQs through their LLRs; REQ-6jefu2 12 direct; REQ-and2gz an expectation, no test by design |
| Coverage, against the class target | org-io: lines 77.38% (floor 70), regions 76.39% (floor 68), branches 49/62, 79.03% (floor 64) — every floor met. on-chain-client: lines 50.36% (floor 46), regions 51.31% (floor 47), branches 58/58 (floor 99) — every floor met. Both units' line and region figures stay below the class C target, as the root `Makefile` documents above each floor (org-io: the chain-facing shell, SDD-z85ux9, and unreachable defensive branches; on-chain-client: client.rs's chain-facing async paths) — previously owner-accepted shortfalls (org-io at S2's merge, 2026-10-08; on-chain-client's predates S2), not widened here. Floors not lowered; T17b made the coverage set match the verify line (`--test publish_path --test startup_reconcile` in `ORG_IO_COVERAGE_ARGS`) |
| Robustness (normal and abnormal per LLR) | pass: one test asserting both for LLR-36zp69, LLR-kp95ty, LLR-cc59ys. Observation: merged LLR-a2acvh has two normal tests in this tree; its failure path is tested under sibling LLR-z9vugt (`a_failed_genesis_step_is_named_and_stops_the_ceremony`) and REQ-6jefu2 has both halves — accepted, no gap (a merged item; this change only added the normal-case test of review round 1, finding-2) |
| Working tree | clean (`git status` empty), tree unchanged |

Totals on this tree: 716 Rust tests passed (104 + 11 + 307 + 73 + 57 + 164),
0 failed; 51 vitest; 9 fuzz targets; 5 Quint invariant runs, no violation.
Gate log: `/private/tmp/claude-501/worktree-org-io-publish-gate6.log`.

### Problem-ledger delta (this change's IDs only)

- Resolves: PR-924ftr (moved from the app's ledger to
  `org-io/docs/problems/2026-10-13-publish.md` under ruling A, arriving
  resolved, T17a; resolution corrected after review round 1, finding-1);
  PR-wk7zwq (org-io, T16a); PR-ksm2ua (on-chain-client, T16a).
- Opens: PR-6nhrr7 (org-io, `org-io/docs/problems/2026-10-13-publish.md`;
  review round 1, finding-4); PR-jgvf3b (org-node,
  `org-node/docs/problems/2026-10-13-publish.md`; review round 2, finding-3,
  owner ruling). Each `status: open`, `opened: 2026-10-13`. Expectation
  opened: REQ-and2gz (org-node, `expects: person`; owner ruling, §6a second
  set, 4).
- Accepted (residuals, owner rulings): genesis updates of a Persona whose
  foundings all failed, until roadmap stage S4b (§6a second set, 3); a late
  founding founds a separate Organisation under its own new pure proxy
  (REQ-dqbg9p narrowed, review round 1, finding-2).
- Stays open by ruling: PR-vt244s (org-node) by owner ruling D1 — part 1
  landed the unknown-write settlement and the startup reconcile; delivery of
  a reconciled commit and durable resend are part 2's; dated progress note
  added at T17a.
- Widened, stays open: PR-jav8zn (org-node) to `discard_provisional`'s save
  order (merged LLR-7cmp38), owner ruling §6a second set, 2.
- Age limits raised by owner ruling: org-node `problem_age_days` 45 → 60
  (D1, T13c; confirmed 2026-10-08); app 30 → 60 (§6a, 3).

### Supersessions and edits of merged items

- Superseded (hybrid rule; new red-first tests carry both IDs, green tests
  not re-annotated): REQ-ntkjk6 > REQ-f3eu9n and LLR-2hkqv2 > LLR-vyd5d3
  (org-io); REQ-b8x2py > REQ-v8jczx, LLR-n2vx5q > LLR-a9bb7b and
  LLR-59kxsr > LLR-ywhd23 (on-chain-client). Owner rulings D4 + D5.
- Narrowed in place: merged REQ-8p2veg — its own read was removed with
  `OrgRegistryClient::original_account`; the mapped account is provided only
  through `signatory_set`'s pinned read (dated notes, review round 1,
  finding-8, and round 2, finding-4).
- Owner-accepted edits of merged green tests:
  `a_build_without_dev_seed_says_why_the_own_admin_check_is_not_configured`
  (LLR-rgdx22) mechanically ported to the one `signatory_set` read,
  assertions and annotation unchanged (§6a, 2);
  `commit_genesis_refuses_a_persona_bound_to_another_organisation`
  (LLR-eyc4ud) one assertion "the update stays" → none stays, forced by the
  rewritten LLR-pk6mhv (§6a second set, 1). Also inherited:
  `product_registers_the_same_commands_as_the_harness` (LLR-4wcyqy), command
  count 12 → 15 → 16 → 13, dated notes.
- Deleted with a superseded meaning (T16a; dated notes under LLR-ywhd23 and
  LLR-vyd5d3): `proxied_and_map_account_build_the_named_calls`,
  `admin_when_the_mapped_proxy_lists_the_controller`,
  `not_admin_for_an_unmapped_h160_no_delegates_or_other_delegates`.
- Minted and deleted within this change, with their tests (owner ruling 4 of
  2026-10-13; IDs in commit `94140e9`'s body and the plan, not in the
  ledgers): the app's discard requirement and its command LLRs, org-io's
  held-update listing LLR, org-node's by-reference listing LLR; an org-io
  guard LLR minted and removed unused before any commit.

## Red → green

Copied from the plan's `red -> green:` lines (Progress: T14, T15, T15b, T16a,
T15c, T15d, gate1fix, review round 1 fixes, review round 2 fixes). gate2trace
(`103b2df`) was a ledger-only fix (LLR-zj4jju's robustness note names
org-io's test, not its item: UNDECLARED-DEPENDENCY) and has no red. A REQ or
SDD is verified through the LLRs it names; its own red is theirs. Rows for
tests later deleted are kept: the observation was made.

| Item | Test | Watched red |
| --- | --- | --- |
| LLR-zfm3bf | `the_last_valid_block_is_the_birth_plus_the_mortality`, `the_last_valid_block_saturates`, `the_mortality_is_a_power_of_two` (on-chain-client) | T14: compile-red (no `WriteWindow`/`WRITE_MORTALITY_BLOCKS`) before the implementation, green after |
| LLR-zj4jju | `submit_update_dispatches_with_the_window_it_was_given`, `genesis_dispatches_with_no_window` | T14: compile-red (`dispatch` and `submit_update` arity), green after |
| LLR-zj4jju | `a_refused_mortal_dispatch_is_reported_and_not_retried_without_its_window` | gate1fix: red against a planted retry without the window, reverted, 8/8 green |
| LLR-pk6mhv (old text) | `discarding_a_personas_genesis_updates_removes_each_and_keeps_the_rest`, `discarding_the_genesis_updates_of_a_persona_with_none_is_refused_and_writes_nothing` (org-node) | T14 fallback: compile-red (no `discard_genesis_updates`), green after; deleted in T15d |
| LLR-2yjuaj, LLR-t98pk9, LLR-nqvbb6, PR-924ftr | `a_write_that_lands_after_its_timeout_is_committed_and_sent_to_its_own_recipient` | T14: compile-red (no `Settled`, no `settle_unknown`), then runtime-red with the settle step disabled (S2's behaviour: D's write went out, "epoch mismatch: expected 1, found 2"); `PR-924ftr` added to its `verifies:` at T17a |
| LLR-2yjuaj | `a_failed_window_read_writes_nothing`, `a_failed_write_records_no_unknown_write` | T14: compile-red, green after |
| LLR-nqvbb6 | `an_unknown_write_blocks_a_second_write_for_that_organisation_only` | T14: compile-red, then runtime-red with the settle step disabled |
| LLR-t98pk9 | `an_unknown_write_whose_chain_moved_elsewhere_is_reconciled`, `a_failed_read_leaves_the_outcome_unknown_and_changes_nothing`, `within_the_window_the_outcome_stays_unknown`, `past_the_window_at_the_records_epoch_the_write_is_not_executed`, `settle_with_no_unknown_write_reads_nothing`, `a_committed_late_write_whose_send_fails_keeps_the_commit` | T14: compile-red (no `Settled`/`Reconciliation`), green after |
| LLR-fptp57, LLR-36zp69 (old texts) | five `discard_*` tests: `discard_removes_a_named_update_and_its_key`, `discard_of_an_update_not_held_is_refused_and_changes_nothing`, `discard_of_an_unknown_write_is_refused_until_it_is_known_not_executed`, `discard_genesis_removes_every_genesis_update_the_persona_built`, `discard_genesis_of_a_persona_with_none_is_refused` | T14: compile-red (no `DiscardRefused`, no `discard_*` on `OrgIo`), green after; deleted in T15d |
| LLR-cc59ys | `org_io_removes_nothing_itself_and_discards_only_in_publish` | T14: runtime-red ("lib.rs offers fn discard_update once, left: 0"; then naming a planted `remove_file` and `discard_provisional` in lib.rs), green after; amended in T15d |
| LLR-k5pzsz | `a_write_that_executed_before_a_crash_is_committed_at_the_next_start`, `in_step_behind_and_refused_are_reported_and_change_nothing`, `a_reconciled_removal_of_this_node_is_reported_removed_and_returns_no_acknowledgement`, `reconcile_held_reports_every_held_organisation_in_store_order` | T15: compile-red (no `reconcile`/`reconcile_held`/`verification` on `OrgIo`, no `Verification`), green after |
| LLR-kp95ty | `an_unreachable_chain_marks_unverified_and_commits_and_deletes_nothing` | T15: compile-red, green after |
| LLR-cc59ys | `no_deletion_on_any_trigger_but_a_verified_one` | T15: compile-red, green after; T15d (amended: last trigger the settle's own discard): runtime-red before the settle's discard, green after |
| LLR-t98pk9 | `a_settle_that_finds_no_state_or_an_older_state_stays_unknown`, `a_settle_whose_record_is_no_longer_held_is_refused` | T15: code from T14, so each runtime-red against a planted change of its arm (the `Ok(None)` arm returning `NotExecuted`; the older-epoch arm disabled; a planted `StillUnknown` for the unheld record), reverted, green |
| LLR-ekz4zn, LLR-8axx7h (app) | `reconcile_all_reports_in_step_and_lists_the_organisation_verified`, `reconcile_all_over_an_unreadable_chain_reports_unverified_with_the_reason`, `each_reconciliation_crosses_ipc_tagged_with_its_fields` (reconcile.rs) | T15b: compile-red (no `commands::reconcile_all`, `ReconciliationDto`, `OrgDto::new`), green after |
| LLR-ekz4zn, LLR-8axx7h (app) | `reconcile_organisations_with_the_chain_not_configured_reports_every_organisation_unverified`, `reconcile_organisations_on_an_empty_store_reports_nothing`, `list_orgs_reports_not_checked_before_any_reconcile`; three discard IPC tests (deleted in T15d) | T15b: compile-red (`generate_handler!` found no such commands), green after |
| LLR-hwcj9n | reconcile.view.test.ts's seven tests of `reconciliationLine`, `verificationLabel` and the three wrappers | T15b: suite red (module `src/lib/reconcile` missing), green after |
| LLR-hwcj9n | "asks for a reconcile once when it mounts" | T15b: suite red, then runtime-red against a planted `onMount` without the call ("expected undefined to be 1"), reverted, green |
| LLR-59kxsr, LLR-ywhd23 | `proxied_forces_the_any_proxy_type_by_name`, `proxied_never_leaves_the_proxy_type_unforced_or_forced_by_index` | T16a: runtime-red against the `None` shape before `proxied` forced `Some(Any)`, green after |
| LLR-59kxsr | `proxied_encodes_any_at_the_index_the_runtime_gives_it`, `a_runtime_without_any_refuses_the_proxied_call` | gate1fix: red against a planted `force_proxy_type: None`, reverted, 22/22 green |
| LLR-n2vx5q, LLR-a9bb7b | `definitions_decode_with_type_and_delay_in_stored_order`, `a_malformed_definitions_value_is_refused` | T16a: compile-red (no `ProxyDefinition`/`decode_proxy_definitions`), green after |
| LLR-n2vx5q | `fuzz_signatory_set` extension | T16a: red against a planted decoder accepting trailing bytes, reverted, green |
| LLR-sc79gq, PR-ksm2ua | `the_signatory_set_is_read_at_one_block` | T16a: runtime-red (no `signatory_set` in client.rs), then red against a planted two-block read, reverted, green; amended in gate1fix (compile-red E0432 before the move, green after) |
| LLR-sc79gq | `the_any_index_is_read_from_the_runtimes_metadata_by_name`, `a_runtime_without_an_any_proxy_type_is_refused`, `the_set_is_the_account_its_definitions_and_the_any_index`, `a_failed_read_or_an_undecodable_value_is_an_error`, `each_entry_names_its_storage_and_key` | gate1fix: compile-red (E0432) before the move from client.rs to `write/signatory_set.rs`, 12/12 green after |
| LLR-2hkqv2, LLR-vyd5d3, PR-wk7zwq | `a_restricted_or_delayed_delegate_cannot_dispatch` | T16a: compile-red (no `DelegateCannotDispatch`/`SignatorySetView`), then runtime-red against the superseded any-delegate rule, reverted, green |
| LLR-2hkqv2, LLR-vyd5d3 | `only_controller_any_zero_is_admin`, `an_unmapped_account_is_not_admin_whatever_the_definitions`, `the_any_index_is_the_reads_not_a_constant` | T16a: compile-red, then each runtime-red against a planted rule (first-definition-only; no unmapped guard; a constant index 0), reverted, green |
| LLR-2hkqv2 | `a_failed_signatory_set_read_is_an_error_never_not_admin` | T16a: compile-red only (the error path is the `?` on the one read) |
| (deleted listing LLRs, T15c) | `provisional_updates_names_each_held_update_and_nothing_else`, `provisional_updates_of_an_unheld_organisation_is_empty`, `listing_held_updates_copies_no_held_update`; org-node `held_provisional_updates_yields_the_organisations_updates_by_reference`; two `list_provisional_updates_*` IPC tests; two view tests | T15c: each compile- or runtime-red (the scan against a planted body reading org-node's cloning `provisional_updates(`; the org-node test against a planted filter also yielding genesis updates), reverted, green; all deleted in T15d with their items |
| LLR-pk6mhv | `a_founding_drops_the_personas_other_genesis_updates` | T15d: runtime-red on the old `commit_genesis`, green after |
| LLR-pk6mhv | `a_refused_founding_drops_no_genesis_update`, `a_founding_keeps_another_personas_genesis_update` | T15d: runtime-red against planted drops (before the refusal checks; of every Persona's updates), reverted, green |
| LLR-t98pk9 | `a_write_known_not_executed_is_discarded_with_its_key`, `a_settle_that_cannot_save_its_discard_stays_unknown` | T15d: runtime-red (the settle kept the update / returned `NotExecuted` with the store file read-only) before the settle's discard, green after |
| LLR-fptp57 | `a_submitted_commit_leaves_no_other_update_for_the_organisation`, `a_settled_late_write_leaves_no_other_update_for_the_organisation`, `a_reconciled_commit_leaves_no_other_update_for_the_organisation`, `a_received_update_leaves_no_other_update_for_the_organisation` | T15d: pass on the code before (org-node's `discard_orphans`); each watched runtime-red against a planted `discard_orphans` keeping the orphans, reverted, green |
| LLR-fptp57 | `a_path_that_commits_nothing_discards_nothing` | gate1fix: red against a planted discard on a behind reconcile, reverted, 20/20 green |
| LLR-36zp69 | `a_founding_after_a_failed_one_leaves_no_genesis_update_held` | T15d: runtime-red against a planted `commit_genesis` keeping the other genesis updates, reverted, green |
| LLR-cc59ys | `no_public_method_discards_a_provisional_update` | T15d: runtime-red against a planted `pub fn discard_planted`, reverted, green |
| LLR-k5pzsz, LLR-t98pk9 | `a_reconcile_that_commits_an_unknown_write_sends_it_to_its_recipient_and_clears_it` | review round 1 (finding-1): runtime-red on the old code (settle after the reconcile returned `Reconciled(InStep)`, not `NoUnknownWrite`), green after |
| LLR-k5pzsz, LLR-nqvbb6; LLR-t98pk9, LLR-nqvbb6 | `after_a_reconcile_commits_an_unknown_write_the_next_submission_is_written`; `a_settle_that_finds_its_write_already_committed_lets_the_submission_write` | review round 1 (finding-1): runtime-red on the old code ("was settled by reconciling with the chain (InStep …); this update was not written"), green after |
| LLR-a2acvh, REQ-6jefu2 (merged) | `each_genesis_creates_a_new_pure_proxy_and_writes_through_it` (on-chain-client `write_compose.rs`) | review round 1 (finding-2): runtime-red against a planted `genesis` reusing the prior founding's proxy (second founding returned `PROXY` again), reverted, 9/9 green |
| LLR-t98pk9 | `an_unknown_write_whose_epoch_the_record_has_passed_is_settled_at_once` | review round 1 (finding-3): runtime-red on the old code (`StillUnknown { "… may yet execute until block 64 (finalised: 0)" }`), green after |
| LLR-nqvbb6 | `resubmitting_an_update_whose_write_did_not_execute_says_it_must_be_rebuilt` | review round 1 (finding-5): runtime-red on the old code (`NOT_HELD_ON_CURRENT_ROOT`), green after |
| LLR-hwcj9n | "calls reconcile_organisations when its reconcile button is pressed" (app `tests/reconcile.view.test.ts`) | review round 1 (finding-6): runtime-red against a planted handler without the call (`requested = []`), reverted, green |
| LLR-nqvbb6 | `resubmitting_an_update_another_commit_overtook_says_it_must_be_rebuilt` | review round 2 (finding-2): runtime-red on the old code (the report said "did not execute (its window has passed)"), green after |
| REQ-by9k5q, REQ-77rh5a, REQ-dqbg9p, REQ-cz753a, SDD-6mpmsw | through LLR-2yjuaj, LLR-t98pk9, LLR-nqvbb6, LLR-fptp57, LLR-36zp69, LLR-k5pzsz, LLR-kp95ty, LLR-cc59ys | see those rows |
| REQ-ntkjk6 | through LLR-2hkqv2 (and LLR-c9fyun, LLR-qhyc3n by dated note) | see those rows |
| REQ-4aq8zh; REQ-b8x2py | through LLR-zfm3bf, LLR-zj4jju; LLR-n2vx5q, LLR-59kxsr, LLR-sc79gq | see those rows |
| REQ-363366 (app; SDD-rmbr3t, SDD-huk773 amended) | through LLR-ekz4zn, LLR-8axx7h, LLR-hwcj9n | see those rows |
| REQ-and2gz | none — an `expects: person` item | not applicable: an open expectation (Gaps) |

## What was wrong, and what was built

What was wrong: a chain write that timed out was reported "outcome unknown"
and then never settled (PR-924ftr). If it landed late, the next submission's
write went out against the moved chain and was refused by the contract's
epoch check ("epoch mismatch: expected 1, found 2"), while the earlier update
was never committed or sent to its own recipient; nothing discarded kept
updates, so every failed founding left a genesis update holding an
Organisation private key. The own-admin check said `Admin` for any delegate
equal to the controller whatever its proxy type and delay (PR-wk7zwq), and
its two signatory-set reads could come from different blocks (PR-ksm2ua).
Writes were signed with subxt's defaults, so no window bounded when an
unknown write could still land. Nothing called org-node's reconcile: a device
never learned at start that the chain had moved, and had no unverified state.

What was built: on-chain-client signs every update write mortal (64 blocks)
and returns its window; reads the signatory set (mapped account, proxy
definitions with type and delay, and the runtime's `Any` index) at one
finalised block; and the writer forces `Any`. org-io's new `publish.rs`
(`PublishState`, one `OrgIo` field) records an unknown write with its
recipient and expected epoch, settles it before any second write for that
Organisation (committing and sending a late write to its own recipient;
`NotExecuted` only past the window at the record's epoch, or at once when
the record has passed the expected epoch; otherwise still unknown), and
discards in the background only what is known not to execute. `reconcile`
and `reconcile_held` reconcile every held Organisation at start and on
request, marking a failed read `Unverified` and changing nothing, and settle
an executed unknown write they find. Admin is a `(controller, Any, 0)`
definition, with `DelegateCannotDispatch` for a restricted or delayed one.
org-node's `commit_genesis` drops the Persona's other genesis updates. The app
reconciles at start (`onMount`) and from its reconcile button and shows each
Organisation's verification; it offers no discard.

## Review

One block per finding the reviewers raised, verbatim, grouped by round. Each
round's own header lines are quoted above its findings.

### Round 1

> Round 1 review of worktree-org-io-publish at the 'review dispatched' commit (tree 9f55468837bfd982237a984ee1ba6024c9f92c4e), independent subagent, 2026-10-13.
> verdict: changes requested — two medium findings (one code, one requirement); every other is low or record. Suite green; check-trace, check-ids and check-units pass for all four units.
>
> Checked and sound: D3 mortal-write rule for update writes (at_current_block = latest finalised; finalised block read before state; Era::mortal(64,b) last valid b+63, last_valid_block b+64 one block conservative); background discard (no public discard; discard_provisional only in publish.rs; no app discard command); key custody (new OrgIo return types allowlisted, public data only, no seed/private key exposed or cloned, keychain unchanged); merged-test edits only the two owner-accepted ones; PR-wk7zwq, PR-ksm2ua have reproducing tests; age-limit raises match rulings.
>
> Suite counts (fresh CARGO_TARGET_DIR=/private/tmp/claude-501/target-review, all exit 0): org-io line 1 98 passed (publish_path 20, startup_reconcile 6, …); line 2 11; build OK; clippy clean. on-chain-client 73 + 56. org-node 307 (commit_paths 39); Quint 0.33.0 both typechecks, five invariants "[ok] No violation found". app cargo 164 (ipc 36, reconcile 3); svelte-check 201 files 0/0; vitest 8 files 50 passed.

Round 1 (at `fb19b14`):

**finding-1**: code, medium — `OrgIo::reconcile` / `reconcile_held` ignore a recorded unknown write, so REQ-by9k5q's "whenever the app asks … commit that update and send it to the Device the submission named" is not met. The app's only way to ask is `reconcile_organisations` → `reconcile_held` (app/src-tauri/src/commands.rs:376-384), which never calls `settle_unknown`; `publish::reconcile` (org-io/src/publish.rs:212-230) never reads or clears `state.unknown_writes`. Scenario: update C times out and later lands; the user presses "Reconcile with chain"; org-node's reconcile commits C and returns `Reconciliation::Committed`, but nothing sends C to the Device C's submission named, and the unknown write stays recorded. The next submission D runs `settle_unknown`; C is now committed, so `executed == true` and org-node's reconcile returns `InStep`, giving `Settled::Reconciled(InStep)`, and D is refused with "was settled by reconciling with the chain (InStep …); this update was not written" (publish.rs:196-205, submit.rs:245-248). C is never delivered to the Device the submission named. LLR-t98pk9 leaves the "executed but already committed" case unspecified. PR-924ftr's resolution text (org-io/docs/problems/2026-10-13-publish.md, last paragraph: "the retry is org-io's settle, run before the next write and by the reconcile") claims behaviour that does not exist. No test reconciles while an unknown write is recorded. Remedy: in `reconcile`, when the Organisation has an unknown write, route it through `settle_unknown`, or on `Committed` of that write's update send it to the recorded recipient and clear it; treat "executed and already InStep" as settled, not as a refusal of the new submission. Red-first test, e.g. `a_reconcile_that_commits_an_unknown_write_sends_it_to_its_recipient_and_clears_it`.
disposition: fixed (`e345643`) — `reconcile`/`reconcile_held` clear an unknown write whose update the state read holds and, when org-node's reconcile commits it, send it to the recorded recipient (a send failure does not unwind the commit; the Device is among `recipients`); `settle_unknown`'s executed-and-already-`InStep` result lets the submission write. Tests (`org-io/tests/publish_path.rs`): `a_reconcile_that_commits_an_unknown_write_sends_it_to_its_recipient_and_clears_it`, red on the old code (settle after the reconcile returned `Reconciled(InStep)`, not `NoUnknownWrite`); `after_a_reconcile_commits_an_unknown_write_the_next_submission_is_written` and `a_settle_that_finds_its_write_already_committed_lets_the_submission_write`, red on the old code ("was settled by reconciling with the chain (InStep …); this update was not written"). LLR-k5pzsz (now also `satisfies: REQ-by9k5q`), LLR-t98pk9, LLR-nqvbb6 amended in place; PR-924ftr's resolution corrected with a dated note.

**finding-2**: requirement, medium — REQ-dqbg9p's "It shall never discard an update whose write may still execute" (org-io/docs/requirements/2026-10-13-publish.md:108-109) is contradicted by its own founding clause and the code. `commit_genesis` drops every other genesis update of the Persona (org-node/src/service.rs:294-297, LLR-pk6mhv), including one whose founding timed out. Genesis writes are not tracked as unknown and are signed with subxt's defaults: mortal 32 blocks when block info is available, else immortal (LLR-zj4jju, REQ-4aq8zh note). So a timed-out founding's RecordGenesis can still execute after its update and key are gone. LLR-36zp69 and the risk file (org-io/docs/risk/2026-10-13-publish.md, REQ-dqbg9p passage, "a founding that timed out and executed later…") record this as an availability-only residual, but the requirement text still says "never". It also breaks the stated project rule that an unknown-outcome write is never discarded before the 64-block window has passed and a finalised block shows it did not land. Remedy: narrow REQ-dqbg9p by a dated in-place note with an owner ruling recorded, or have `commit_genesis` keep genesis updates whose founding timed out.
disposition: owner ruling 2026-10-13 — REQ-dqbg9p narrowed in place by a dated note quoting the ruling: its "never … may still execute" clause covers update writes; a late founding founds a separate Organisation under its own new pure proxy. Risk passage re-assessed citing the ruling; LLR-36zp69 dated note. No existing non-chopsticks test asserted that two foundings yield two distinct proxies, so a new test on the merged items: `each_genesis_creates_a_new_pure_proxy_and_writes_through_it` (`on-chain-client/tests/write_compose.rs`; LLR-a2acvh, REQ-6jefu2; no existing test re-annotated), red against a planted `genesis` reusing the prior founding's proxy (the second founding returned `PROXY` again).

**finding-3**: code, low — `settle_unknown` judges "may still execute" against the record's current epoch, not the epoch the write expected (org-io/src/publish.rs:165-176). If a received update has meanwhile moved the record (and chain) to E+1, the write expected E can never pass the contract's epoch check, yet the settle returns `StillUnknown { "… may yet execute until block …" }` and blocks every submission for that Organisation for up to 64 blocks; afterwards `discard_provisional` returns `NoProvisionalUpdate` and the result is `NotExecuted`. Remedy: keep the write's expected epoch in `UnknownWrite`; when the record's epoch is past it, settle at once.
disposition: fixed (`e345643`) — `UnknownWrite` keeps its expected epoch; a record already past it settles `NotExecuted` at once with no read (the discard tolerating `NoProvisionalUpdate`). Test `an_unknown_write_whose_epoch_the_record_has_passed_is_settled_at_once` (`org-io/tests/publish_path.rs`, LLR-t98pk9), red on the old code (`StillUnknown { "… may yet execute until block 64 (finalised: 0)" }`). LLR-2yjuaj and LLR-t98pk9 amended in place; the requirement wording of this case is round 2's finding-1.

**finding-4**: code, low — a write that fails after broadcast (subxt watch or RPC error inside `submit_and_watch`, on-chain-client/src/write/subxt_ops.rs:63-75) is `WriteNotDone::Failed`, reported as "chain write failed; nothing committed or sent", and no unknown write is recorded (org-io/src/submit.rs:276-281). The mortal transaction may still land within 64 blocks, and the next submission is then refused by the contract's epoch check until the user reconciles. Predates the change (S2), but REQ-by9k5q only covers the timeout. Remedy: book a problem report, or classify post-submit watch errors as unknown outcomes.
disposition: booked as open problem report PR-6nhrr7 (`org-io/docs/problems/2026-10-13-publish.md`); not fixed in this change.

**finding-5**: code, low — a user who resubmits the same timed-out update after its window loses it without being told why. `settle_unknown` returns `NotExecuted` and discards it, `submission_report` is empty for `NotExecuted` (publish.rs:135), and the submission is then refused with `NOT_HELD_ON_CURRENT_ROOT` (submit.rs:219-221, 245-266), whose message lists only "discarded" among possible causes and does not say this write did not execute and must be rebuilt. Remedy: when the settle discards the very `BuiltUpdate` being submitted, return a specific report.
disposition: fixed (`e345643`) — resubmitting the very update whose write the settle finds not executed returns `NOT_EXECUTED_RESUBMITTED` ("… did not execute … rebuild the update …"). Test `resubmitting_an_update_whose_write_did_not_execute_says_it_must_be_rebuilt` (`org-io/tests/publish_path.rs`, LLR-nqvbb6), red on the old code (`NOT_HELD_ON_CURRENT_ROOT`). LLR-nqvbb6 amended in place.

**finding-6**: code, low — LLR-hwcj9n's clause "the Organisation list calls it again when the user presses its reconcile button" (app/docs/architecture/2026-10-13-publish.md) has no test; app/tests/reconcile.view.test.ts covers only the `onMount` call, by a regex over the page source. The button is in app/src/lib/components/PersonaList.svelte:25-37,138. Remedy: add a test of that button, or move the clause to SDD-6g3wnh's untested shell with a note.
disposition: fixed (`61ca846`) — test "calls reconcile_organisations when its reconcile button is pressed" (`app/tests/reconcile.view.test.ts`, LLR-hwcj9n; the source-scan style of the `onMount` case), red against a planted handler without the call (`requested = []`). LLR-hwcj9n dated note.

**finding-7**: record — the verification record (docs/verification/2026-10-13-worktree-org-io-publish.md) is still the T17a DRAFT and is stale: gate figures say org-io 97 passed (publish_path 19) and on-chain-client 73 + 48 (measured: 98, publish_path 20; 73 + 56); it omits gate1fix's red -> green tests (LLR-sc79gq ×5, LLR-59kxsr ×2, LLR-zj4jju, LLR-fptp57 `a_path_that_commits_nothing_discards_nothing`) and the gate2trace commit; its T16a deviation ("no metadata fixture … shell") was superseded by gate1fix; it has no `## Guideline reviews` section (only a note that guidelines-file.sh is absent) and does not list `docs/plans/2026-10-08-org-io-publish.md` as a plan left whole (part 2 pending). The plan still names `DRAFT-worktree-org-io-publish-publish.md` paths (lines 150, 155, 176, 230, 245, 255, 285, 311, 321). The app config's T15b measurement comment (app/.guardrails/config.yaml:136-143, "ipc 39 … 167 tests; vitest 52") has no T15d re-measurement (now 164 / vitest 50). Remedy: regenerate the record at 6b with final figures and missing sections; update those references.
disposition: fixed — the plan's references name `2026-10-13-publish.md` paths and `app/.guardrails/config.yaml` gains the dated T15d re-measurement (`61ca846`); this record was regenerated at step 6b from the template on the final gate's tree, with gate1fix's red -> green rows, the gate2trace commit, the gate1fix supersession of the T16a "no metadata fixture" note (Deviations), a Guideline reviews section, and the plan listed as left whole (Gaps).

**finding-8**: record — the superseded-ID sweep is incomplete. LLR-c9fyun and LLR-qhyc3n still name only `satisfies: REQ-f3eu9n` (org-io/docs/architecture/2026-10-08-org-io.md:748, 809); both now realise REQ-ntkjk6, which traces only through LLR-2hkqv2. LLR-c9fyun's body still states the old `admin_status(…, original_account, delegates)` result; its 2026-10-13 note calls this "claims unchanged", but the result set gained `DelegateCannotDispatch`. SDD-f4khqn's text still says "that account's proxy delegates". on-chain-client keeps public `OrgRegistryClient::proxy_delegates` / `original_account` (client.rs:251-253, doc citing REQ-v8jczx/LLR-a9bb7b), each resolving its own block (the PR-ksm2ua shape), and nothing uses them any more. Remedy: dated notes adding REQ-ntkjk6 where the item now serves it; retire or mark the unused superseded reads.
disposition: fixed (`61ca846`) — dated notes add `satisfies: REQ-ntkjk6` to LLR-c9fyun and LLR-qhyc3n; LLR-c9fyun's note corrected (`DelegateCannotDispatch` joins its result set); SDD-f4khqn note; on-chain-client's unused `OrgRegistryClient::original_account` and `proxy_delegates` (and the private `fetch_storage_raw` only they used) removed, with dated notes on SDD-rxfu6h, LLR-a9bb7b, REQ-v8jczx, and REQ-8p2veg (narrowed: provided through `signatory_set`'s pinned read).

### Round 2

> Round 2 review of worktree-org-io-publish at 5b5a67f ('review dispatched', tree 54bc186f, after merge 3350f41 of r1fix), independent subagent, 2026-10-13.
> verdict: approve — every round-1 finding is resolved or booked, and all five new findings are low or record. By the checklist rule, a round with only low code/requirement findings is the last review round.
>
> Round-1 resolution: r1-f1 resolved (residual f3); r1-f2 resolved by owner ruling (REQ-dqbg9p narrowed, test each_genesis_creates_a_new_pure_proxy_and_writes_through_it, LLR-a2acvh/REQ-6jefu2); r1-f3 resolved (doc gap f1); r1-f4 booked PR-6nhrr7; r1-f5 resolved (message nit f2); r1-f6 resolved; r1-f7 resolved in part (record regenerated at 6b); r1-f8 resolved (residual f4).
>
> Checked sound: no app discard command; REQ-dqbg9p narrowing quotes the ruling and the new-proxy test exists; no new public return type; no merged test re-annotated; age limits as ruled; new tests would fail if behaviour broke. find-items.sh show ran too slowly in the sandbox; items not reopened this round beyond those listed in the report.
>
> Suite (fresh CARGO_TARGET_DIR=/private/tmp/claude-501/target-review2, all exit 0): org-io 103 + 11, build OK, clippy clean; on-chain-client 73 + 57; org-node 307, quint both typechecks, five invariants "[ok] No violation found"; app cargo 164, svelte-check 201 files 0/0, vitest 8 files 51. check-ids and check-trace exit 0 for org-io, on-chain-client, org-node, app; check-units not finished.

Round 2 (at `5b5a67f`):

**finding-1**: requirement, low — The new settle case (b) is not covered by REQ-by9k5q, and its wording conflicts with ruling D3. Code: org-io/src/publish.rs:174-182 — when the record's epoch is past the write's `expected_epoch`, the code returns `NotExecuted` at once and calls `discard_provisional`, inside the 64-block window, with no read. REQ-by9k5q (org-io/docs/requirements/2026-10-13-publish.md:37-49) allows `NotExecuted` only when the chain is still at the record's epoch and a finalised block past the window has been read; otherwise "report the outcome as still unknown". Also contradicted: the risk passage for REQ-by9k5q (org-io/docs/risk/2026-10-13-publish.md:31-34) "a discard is refused until a finalised block past the write's mortality shows the record's epoch", and LLR-t98pk9's earlier sentence "It never discards before `NotExecuted` (owner ruling D3)". The code itself is right: the contract's epoch check means the write can never execute, and in every reachable state org-node's commit had already dropped the update (`discard_orphans`, org-node/src/service.rs:304-308), so the discard returns `NoProvisionalUpdate` and does nothing. Remedy: dated in-place note on REQ-by9k5q stating case (b); update the risk passage; record that D3 still holds because the moving commit already dropped the orphan, or get the owner to confirm.
disposition: fixed, doc only (`d77e8f9`) — dated notes on REQ-by9k5q (case (b): a record already past the write's expected epoch settles it not executed at once, since the contract's epoch check means it can never execute), on its risk passage (D3's control unchanged in effect, no new residual), and on LLR-t98pk9 (D3 still holds: it forbids discarding a write that may still land, this one cannot, and the commit that moved the record already dropped the update via `discard_orphans`, so the discard is a no-op). No code change, so no red.

**finding-2**: code, low — The resubmission refusal message is wrong in case (b). `NOT_EXECUTED_RESUBMITTED` (org-io/src/submit.rs:226-228) says "did not execute (its window has passed)"; the case-(b) path also returns it when another commit moved the record and the window has not passed. The test `resubmitting_an_update_whose_write_did_not_execute_says_it_must_be_rebuilt` checks only "did not execute" and "rebuild". Remedy: drop the parenthetical, or give case (b) its own report ("another update committed first").
disposition: fixed (`d77e8f9`) — `NOT_EXECUTED_RESUBMITTED` drops "(its window has passed)", so it is true in case (b) too. Test `resubmitting_an_update_another_commit_overtook_says_it_must_be_rebuilt` (`org-io/tests/publish_path.rs`, LLR-nqvbb6; case (b) through `submit_commit_send`, asserting no "window" in the report), red on the old code (the report said "did not execute (its window has passed)").

**finding-3**: code, low — `reconcile` forgets an executed unknown write before org-node has committed it (org-io/src/publish.rs:273-284): the write is removed from `unknown_writes` before `svc.reconcile` runs; if org-node's reconcile returns `Err` (e.g. the store save fails), the update is not committed and the write is forgotten; a later reconcile then commits the update but sends it to no one, so REQ-by9k5q's "send it to the Device the submission named" is lost. `settle_unknown` has the same shape at publish.rs:212-219, from before this change. LLR-k5pzsz's amendment allows this ("clears the unknown write whatever org-node returns"), and the state is rare. No abnormal-input test for org-node refusing on this new path. Remedy: clear the write only on `Ok(Committed | InStep)`, or book a problem report and add a dependency-failure test.
disposition: booked by owner ruling (2026-10-13) as open problem report PR-jgvf3b (`org-node/docs/problems/2026-10-13-publish.md`; affects LLR-fm38ww, SDD-8cpyfa), for S4 to fix with PR-jav8zn's memory-ahead-of-disk class; a dated note on LLR-k5pzsz points to it. The requested change was probed with a dependency-failure test (store file read-only during `reconcile`) and does not do what the finding expects: org-node's `reconcile` (`org-node/src/service.rs:1019-1021`) commits in memory before its save fails, so the next reconcile is `InStep` (nothing to send) and the next settle reports the executed write `NotExecuted` — the send is lost either way and the fix would add a false report. The probe is PR-jgvf3b's reproduction recipe. Code unchanged (`2f9fd52`).

**finding-4**: record — on-chain-client/docs/requirements/2026-10-13-publish.md:45 still says "REQ-8p2veg's own read is unchanged and stays", contradicting the 2026-10-13 narrowing of REQ-8p2veg and the removal of `OrgRegistryClient::original_account`. Remedy: a dated note at that line.
disposition: fixed (`d77e8f9`) — a dated correction under REQ-b8x2py (`on-chain-client/docs/requirements/2026-10-13-publish.md`): REQ-8p2veg's own read was removed and the mapped account is provided only through `signatory_set`.

**finding-5**: record — The plan header still describes the app discard that owner ruling 4 reversed: docs/plans/2026-10-08-org-io-publish.md:3 (Goal) says "lets the app discard a provisional update"; the file table at lines 80 and 86 lists `discard_update`/`discard_genesis` as OrgIo methods and app commands. The plan is left whole (part 2 pending), so its prose should match the tree. Remedy: a dated strike or note pointing to T15d.
disposition: fixed (`d77e8f9`) — the plan's Goal and its §4 file table (`org-io/src/lib.rs` and app rows) strike the app discard with dated notes pointing to T15d (owner ruling 4).

## Deviations

Recorded in the plan's Progress, each accepted or noted there:

- T14: `on-chain-client/tests/write_compose.rs` was edited with a Python
  script instead of the editor, against AGENTS.md's no-sed rule for Rust
  files; the diff was reviewed and the target is green. gate1fix's edits were
  Python-free.
- T14: org-node fallback taken — `OrgService::discard_genesis_updates`
  (LLR-pk6mhv, old text), an S4 hotspot in `org-node/src/service.rs`; removed
  in T15d, where `commit_genesis` drops the genesis updates instead. T15c
  took a second org-node fallback (a by-reference held-update listing), also
  removed in T15d.
- T14: `Reconciliation` defined in T14 because `Settled::Reconciled` carries
  it; the window-read failure keeps S2's "chain write failed; nothing
  committed or sent:" prefix (LLR-2yjuaj amended in place); settle design
  gaps (no state or an older state → `StillUnknown`; record no longer held →
  `Refused`) amended into LLR-t98pk9, tests moved to T15.
- T15c was added (orchestrator's call on T15b's finding: the user could not
  learn an update's name to discard it) and then reversed the same day by
  owner ruling 4 (background discard); T15d removed it with its items and
  tests.
- T15: the crash test models the restart as `OrgIo::for_test` over the
  reopened store and `FakeChain::sharing_slots_with`; `verification` returns
  `Option`, and LLR-kp95ty was clarified in place.
- T15b: the app's reconcile at start runs from the webview's `onMount`, not
  `AppState::init`.
- T16a: the port of the merged LLR-rgdx22 test (owner-accepted). Its note
  that the `Any` index was shell with no metadata fixture is superseded by
  gate1fix, which moved the lookup into `write/signatory_set.rs` and tests it
  against `scale-info` metadata (test-only dev-dependency, already in subxt's
  tree; SOUP noted). Untouched: on-chain-client's `clippy --all-targets` is
  red on four pre-existing targets not in its verify lines, and
  `tests/common/proxy.rs` (chopsticks helper) still builds
  `force_proxy_type: None`.
- T15d: ledger deletion notes name deleted items by description
  (check-trace rejects a deleted ID named in the ledgers); the store-save
  failure is injected by a read-only store file.
- T17a: the app risk file's two mentions of PR-924ftr describe it by name
  after the move (`NON-EXPORTED-REF`).
- Robustness observation on merged LLR-a2acvh: two normal tests here; its
  failure path is tested under sibling LLR-z9vugt
  (`a_failed_genesis_step_is_named_and_stops_the_ceremony`) and REQ-6jefu2
  has both halves. Accepted, no gap.

## Guideline reviews

None run. `.guardrails/scripts/guidelines-file.sh` is missing from this
guardrails 0.5.1 install, so the develop-change guidelines step could not run
for any task of this change; it was not reinstalled (noted at T15).

## Gaps

- The plan `docs/plans/2026-10-08-org-io-publish.md` is left whole: part 2
  (T13b, T16b–T16g, T17b) is pending, blocked on S4's merge.
- Coverage: org-io and on-chain-client meet their floors, but their line and
  region figures stay below the class C target (the chain-facing shell,
  SDD-z85ux9, and on-chain-client's client.rs paths), previously
  owner-accepted shortfalls (org-io at S2's merge; on-chain-client's
  predates S2).
- Open problem reports from this change: PR-6nhrr7 (a write that fails after
  broadcast records no unknown write), PR-jgvf3b (org-node's reconcile
  commits in memory before a failed save, so org-io loses a late write's
  send). PR-vt244s stays open for part 2; PR-jav8zn now also covers
  `discard_provisional`'s save order.
- REQ-and2gz is an open expectation (`expects: person`), met by no test.
- Genesis updates of a Persona whose foundings all failed stay held until one
  of its foundings commits; accepted until roadmap stage S4b.
- Delivery of a reconciled commit to the Devices it is for (beyond the
  unknown write's own recipient) and durable resend are part 2's (T16d).
