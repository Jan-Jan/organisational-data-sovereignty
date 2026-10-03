# Problem reports — the conformance driver drops two errors

**PR-499dzp**: `MembershipDriver::commit` in `org-members/tests/mbt_conformance.rs`
swallows a `recalculate()` failure after an accepted mutation, and skips the
delta round-trip check without a signal when either `root_hash()` fails.
affects: REQ-wx3wpv, LLR-h7stq2, LLR-n7nya3.
status: resolved
Root cause: `commit` treated both errors as "nothing to check". It now
recalculates only a trie with pending changes, and fails the step with a panic
naming the call and the error. Measured under mutations M1–M3 below, using
`membership_conformance`, `scenario_add_member` and `scenario_apply_honest`.
No reproducing test exists, because the defect hides a failure rather than
causing one. The table below is the substitute.

Found by reading the driver after the quint conformance change (part 1) merged,
not by a failing test. The defect silences a failure instead of producing one,
so no test could catch it. The two lines:

- `match t.recalculate() { Ok((t2, _)) => t2, Err(_) => t }` kept the trie
  as it was when `recalculate()` failed;
- `if let (Ok(old_root), Ok(new_root)) = (old.root_hash(), t.root_hash())`
  ran `calculate_delta -> apply_delta -> verify_against` only when both roots
  existed, and did nothing otherwise.

The driver also recalculated every accepted result, including an `ApplyDelta`
result, which `from_candidate` returns already calculated. The owner has ruled
that recalculating a calculated trie should be refused (PR-zqvs7t). The driver
therefore now recalculates only when `has_pending_changes()`. On a pending trie
`recalculate` fails only with `InvariantViolated`, so an `Err` there is a crate
defect. Every trie the driver keeps carries a root, so a `root_hash` `Err` is
one too.

Measured 2026-10-03 with three temporary mutations of `org-members/src/trie.rs`.
Each was reverted after its run and none was committed. The tests run were
`scenario_add_member`, `scenario_apply_honest` and `membership_conformance`:

| mutation | before the fix | after the fix |
|---|---|---|
| M1: `recalculate` returns `InvariantViolated` on a trie with no pending changes (the behaviour PR-zqvs7t asks for) | all three green: the error was swallowed | all three green: the driver no longer asks |
| M2: `recalculate` always returns `InvariantViolated` | all three red. `scenario_add_member` and `membership_conformance` failed one stage late, in the root-class check, with "root_hash: HashesNotCalculated", naming neither the step nor the call. `scenario_apply_honest` failed with a model divergence: the honest `ApplyDelta` arm's own `recalculate()?` fails first | `scenario_add_member` and `membership_conformance` red in `commit`: "recalculate after an accepted mutation: InvariantViolated"; `scenario_apply_honest` unchanged (divergence) |
| M3: `root_hash` always returns `HashesNotCalculated` | all three red, in the root-class check | all three red, the same way: that check runs on the genesis step, before any `commit` |

In the M2 row, the `membership_conformance` cells depend on the random seed.
When an honest `ApplyDelta` is the first accepted change, the run ends in a
model divergence instead. The independent review saw 8 root-class failures and
3 divergences in 11 runs on master's driver, and 5 `commit` panics and 6
divergences in 11 on the fixed driver. Every other cell reproduces exactly.

The mutations, as inserted at the top of `OrgTrie::recalculate` (M1, M2) or
replacing the body of `OrgTrie::root_hash` (M3), in `org-members/src/trie.rs`:

- M1: `if !self.has_pending_changes() { return Err(OrgMembersError::InvariantViolated); }`
- M2: `if true { return Err(OrgMembersError::InvariantViolated); }`
- M3: `Err(OrgMembersError::HashesNotCalculated)`

Before the fix, no measured case hid a defect. The only silent case, M1, is the
behaviour the owner wants. M2 and M3 were caught one stage later, and only
because of the order the checks happen to run in. So the fix is hardening and
diagnosis: a driver error now stops the step where it happens and names the
call. The second drop cannot be shown green with any mutation that keeps
`root_hash` deterministic. Every trie that `commit` reads as `old` has already
passed the root-class check, and a trie does not change after it is built.

The fix is in the test oracle, not the crate. The driver's panics fire only
under a broken crate, so they carry no `verifies:` of their own, and the
evidence is the mutation table above.
