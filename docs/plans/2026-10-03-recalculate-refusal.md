# Recalculating a calculated trie is refused — Implementation Plan

**Goal:** resolve PR-zqvs7t
(`org-members/docs/problems/2026-10-03-recalculate-calculated.md`).
`OrgTrie::recalculate` on a trie with no pending changes returns the new
`OrgMembersError::HashesAlreadyCalculated`, and every caller that recalculated
a calculated trie stops doing so.
**Implements:** LLR-j35sxz (`satisfies: derived`, in
`org-members/docs/architecture/2026-10-04-recalculate-refusal.md`,
assessed in the risk file of the same name). Owner ruling 4, after the first
independent review, withdrew the amendment first planned here.
**Planned, then withdrawn:** an amendment to LLR-n7nya3 in
`org-members/docs/architecture/2026-09-17-decomposition.md`. LLR-n7nya3 is
unchanged from master, and so is its parent REQ-avmu3j, whose wording was
checked. Resolves PR-zqvs7t.
**Safety class:** C (org-members, org-node).
**Verification:** each unit's `verify_commands` over the impact set from
`check-units.sh --impact master..HEAD`; `check-trace.sh`, `check-ids.sh`,
`check-units.sh`. Run through the sandbox wrapper
`/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/0ab6b8e8-6122-45d2-aff4-9077ce217f4e/scratchpad/env.sh`
(writable HOME for the quint evaluator, `CARGO_HOME=/tmp/cargo_home_fuzz`).

## Owner rulings (2026-10-03)

1. `recalculate()` on a trie that already has a root hash returns an error:
   node hashes are write-once cells (`spin::Once`), so filling them when none
   is unset is a caller error.
2. The error is a new variant, `HashesAlreadyCalculated`, mirroring
   `HashesNotCalculated` and distinct from `InvariantViolated`.
3. It is its own change, separate from the conformance-driver fix (PR-499dzp).
4. (After review round 1.) REQ-avmu3j does not ask for the refusal, so it is a
   separate derived LLR, LLR-j35sxz, with a hazard-impact assessment.
   LLR-n7nya3 stays as on master. The decision-coverage gap is accepted for
   this merge.
5. (After review round 3, 2026-10-04.) The surviving one-member mutant is
   closed with a one-member example in the refusal test, not a property test.

### T5 — review round 3 fix: the refusal on a one-member trie

**Files touched:** `org-members/tests/integration_test.rs`
**Parallel:** no (single task)

Owner ruling 5 (2026-10-04): close finding-4 with a one-member example, not a
property test.

1. RED: in `recalculate_refuses_a_calculated_trie` (verifies: LLR-j35sxz), add
   a calculated one-member trie: `genesis` with one member, then `recalculate()`
   returns `Err(HashesAlreadyCalculated)`. Watch it fail under the reviewer's
   mutation M8 (refusal skipped when `member_count() == 1`), then revert the
   mutation and watch it pass. Do not commit the mutation.
2. Run the org-members `verify_commands`.

**Done** (merged into the change branch, task commit `b4b70bf`).

red -> green: `recalculate_refuses_a_calculated_trie` (verifies: LLR-j35sxz),
the new one-member case. Under M8 (guard
`!self.has_pending_changes() && self.member_count() != 1`) it failed at the
one-member line with `unwrap_err()` on an `Ok` value. It went green once the
mutation was reverted; the mutation was never committed. Result:
`cargo test -p org-members` 0 failed (integration 156, fuzz 9, conformance 32
passed / 1 ignored). quint clean, 63 passing, no violation. check-trace reports
no MISSING-TEST.

### T4 — review round 2 fix: the refusal on an empty organisation

**Files touched:** `org-members/tests/integration_test.rs`
**Parallel:** no (single task)

1. RED: in `recalculate_refuses_a_calculated_trie` (verifies: LLR-j35sxz), add
   the case of a calculated trie with no members: `genesis(vec![])`, then
   `recalculate()` returns `Err(HashesAlreadyCalculated)`. Watch it fail under
   the reviewer's mutation M5 (refuse only when `member_count() > 0`), then
   revert the mutation and watch it pass. Do not commit the mutation.
2. Run the org-members `verify_commands`.

**Done** (merged into the change branch, task commit `bf705f3`).

red -> green: `recalculate_refuses_a_calculated_trie` (verifies: LLR-j35sxz),
the new empty-organisation case. Under M5 (guard
`!self.has_pending_changes() && self.member_count() > 0`) it failed at the
`genesis(vec![])` line with `unwrap_err()` on an `Ok` value. It was green once
the mutation was reverted, and the mutation was never committed. Result:
`cargo test -p org-members` 0 failed (integration 156, fuzz 9, conformance 32
passed / 1 ignored); quint clean, 63 passing, no violation; check-trace no
MISSING-TEST.

### T3 — review round 1 fixes

**Files touched:** `org-members/tests/integration_test.rs`,
`org-members/src/error.rs`, `org-members/src/trie.rs`, `org-members/AGENTS.md`
**Parallel:** no (single task)

1. Re-annotate the two refusal tests from LLR-n7nya3 to LLR-j35sxz, and point
   the doc comments in `error.rs` and `trie.rs` and the `AGENTS.md` line at
   LLR-j35sxz.
2. RED: add a normal-case test annotated `verifies: LLR-j35sxz`. A trie mutated
   and then reverted to its original members, so its change set is empty,
   still recalculates, with an empty delta and the original root. Watch it fail
   under the reviewer's mutation M2 (guard `self.pending_changes()?.is_empty()`),
   then revert the mutation and watch it pass.

**Done** (merged into the change branch, task commit `9b7a5cf`).

red -> green: `recalculate_accepts_a_mutated_trie_with_an_empty_change_set`
(verifies: LLR-j35sxz) — under mutation M2 it failed at
`reverted.recalculate().unwrap()` with `Err(HashesAlreadyCalculated)`; green
once the mutation was reverted (never committed).

Re-annotated, not newly written: `recalculate_refuses_a_calculated_trie` and
`recalculate_succeeds_until_no_changes_are_pending` now verify LLR-j35sxz. Their
red was watched in T1, when they were annotated LLR-n7nya3. Result:
`cargo test -p org-members` 0 failed (integration 156, fuzz 9, conformance 32
passed / 1 ignored); quint typecheck clean, `quint test` 63 passing, `quint run`
no violation; check-trace no longer reports MISSING-TEST LLR-j35sxz.

Design: the check is `!self.has_pending_changes()` at the top of
`recalculate()`, before `pending_changes()`. Error message
`"hashes already calculated"`. A trie is calculated when `genesis()`,
`recalculate()` or `CandidateTrie::verify_against()` returns it.

### T1 — the refusal, in org-members

**Files touched:** `org-members/src/error.rs`, `org-members/src/trie.rs`,
`org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`
(and any other `org-members/tests/*.rs` that the refusal turns red)
**Parallel:** no — T2 depends on it

1. RED: in `integration_test.rs`, next to `root_hash_errs_until_recalculated`,
   add two tests annotated `/// verifies: LLR-n7nya3`:
   - abnormal: `recalculate()` on a `genesis()` trie, on a trie that
     `recalculate()` returned, and on a trie from
     `apply_delta(..)?.verify_against(..)?` each returns
     `Err(HashesAlreadyCalculated)`;
   - normal: a mutated trie recalculates, and the trie that returns is then
     refused (the refusal starts exactly when the pending changes end).
   Watch both fail for the right reason (the variant missing, then `Ok` where
   `Err` is expected).
2. GREEN: add the variant to `OrgMembersError` with a doc comment naming
   LLR-n7nya3, and the guard in `recalculate()`. Update `recalculate()`'s and
   `has_pending_changes()`'s doc comments to state the refusal.
3. Fix the tests the refusal turns red. Expected: four integration tests
   (`root_hash_errs_until_recalculated`,
   `device_slot_order_does_not_change_the_root`,
   `every_device_slot_reaches_the_root`,
   `add_then_delete_returns_to_the_empty_root`) — drop the redundant
   `recalculate()` on the calculated trie, keep what each test asserts.
4. `fuzz_tests.rs` `trie_ops_never_panic_and_count_consistent`: `Op::Recalculate`
   must no longer discard the `Err` with `if let Ok`. On a calculated trie
   assert `Err(HashesAlreadyCalculated)`; on a pending trie it must succeed.
   The `member_count` check runs on every step. Check `delta_roundtrip`, the
   `calculate_delta` round trip and `delta_canonicality_fuzz` for the same
   shape: an early return on `Err` is acceptable only where it can be shown to
   be the no-op case, otherwise branch on `has_pending_changes()`.
5. `mbt_conformance.rs` needs no change (it recalculates only a pending trie);
   confirm it stays green.
6. Run `cargo test -p org-members` and the unit's quint commands.

**Done** (merged into the change branch, task commit `6240d81`).

(Both tests below were re-annotated to LLR-j35sxz in T3.)
red -> green: `recalculate_refuses_a_calculated_trie` (verifies: LLR-n7nya3) —
failed to compile (E0599, no variant `HashesAlreadyCalculated`); with only the
variant added, `unwrap_err()` on the genesis `recalculate()` panicked on an
`Ok` value; green after the `!has_pending_changes()` guard.
red -> green: `recalculate_succeeds_until_no_changes_are_pending` (verifies:
LLR-n7nya3) — E0599 first; then `unwrap_err()` on the recalculated trie got
`Ok`; green after the guard.

Also turned red by the refusal and fixed: the four integration tests named in
step 3, and fuzz `keys_stay_unique`, which recalculated a trie that may already
be calculated (genesis, a failed op, a `verify_against` result) and now goes
through a `calculated()` helper. `delta_roundtrip`, `calculate_delta_roundtrip`
and `delta_canonicality_fuzz` return early only when `!has_pending_changes()`;
otherwise `recalculate()` must succeed. Result: `cargo test -p org-members`
196 passed, 0 failed, 1 ignored; quint typecheck clean, `quint test` 63
passing, `quint run` no violation.

### T2 — callers stop recalculating calculated tries, in org-node

**Files touched:** `org-node/src/service.rs`, `org-node/src/test_fixtures.rs`,
`org-node/tests/chain_genesis_e2e.rs`, `org-node/tests/transport_handshake.rs`,
`org-node/tests/transport_networked.rs`,
`org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`
**Parallel:** no — runs after T1 is merged into the change branch

1. RED: with T1 merged, run org-node's `verify_commands` unchanged and record
   which tests fail with `HashesAlreadyCalculated` (expected, from PR-zqvs7t:
   `service_stories`, `admission_sender`, `verify_against_chain`,
   `chain_genesis_e2e`, `transport_handshake`, `transport_networked`,
   `service::tests::create_organisation_advances_mock_chain`, six
   `envelope::tests`, and the `fuzz_verify_against_chain` harness).
2. GREEN: at each site that calls `recalculate()` on a `genesis()` result,
   use the genesis trie directly. Production sites: `service.rs`
   `create_organisation` and the `receive_and_verify` fallback (since removed by master's `0f85cb9`); the root and
   every observable behaviour are unchanged. Sites that recalculate a mutated
   trie (`add_member`, `delete_member`) stay as they are.
3. Run org-node's `verify_commands` and the dependent `app` unit's.

**Done** (merged into the change branch, task commit `139ed0f`). No tests
added: the existing tests are the red. The red run was org-node's cargo
`verify_command` with T1 merged and no org-node edit, with `--no-fail-fast`.

red -> green: lib `envelope::tests` (6) — panic at `test_fixtures.rs:48`,
`HashesAlreadyCalculated`.
red -> green: lib `service::tests::create_organisation_advances_mock_chain` —
`Trie(HashesAlreadyCalculated)` from `create_organisation`.
red -> green: `admission_sender` (3 of 3) — the same, from `create_organisation`.
red -> green: `service_stories` (3 of 3) — the same, from `create_organisation`.
red -> green: `verify_against_chain` (10 of 13) — at `test_fixtures.rs:48`.
red -> green: `wire_frame_bound` (2 of 3) — at `test_fixtures.rs:48`. PR-zqvs7t's
list left this target out.
red -> green: `transport_handshake::delivers_and_verifies_admit_over_iroh` — at
`transport_handshake.rs:42`.
red -> green: `transport_networked::delivers_and_verifies_admit_over_relay_by_id`
— at `transport_networked.rs:50`.
red -> green: `fuzz_verify_against_chain` harness — panic at `fuzz_target.rs:39`.

Not run: `chain_genesis_e2e` is outside `verify_commands` (it needs
chopsticks). Its two genesis sites were fixed by inspection and the target
compiles (`--no-run`). The app unit was green before and after; nothing in it
calls `recalculate()`. Result at T2: org-node cargo 53 passed, 0 failed, both
bolero targets ran their budget; org-node quint clean; app cargo 78 passed,
`npm run check` 0 errors, vitest 30 passed. Those figures predate the merge of
master (`4c66400`, then `0f85cb9`), which added org-node tests and the
`fuzz_first_admission_base` target. On the merged tree the gate reruns give
org-node 57 passed with three bolero targets, and app is unchanged. The
verification record has the final figures.
