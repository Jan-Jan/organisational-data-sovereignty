# Recalculating a calculated trie — assessment of the derived requirement

assesses: LLR-j35sxz

LLR-j35sxz is derived behaviour by owner ruling (2026-10-03), assessed against
the existing register (`2026-09-02-membership-hazards.md`). "Trie" is the
`OrgTrie` that holds a Membership record, as in the LLR.

**What it reduces.** Nothing in the register. Before it, recalculating a
calculated trie returned the same root and an empty change set, so no
Membership record was ever wrong because of the call. The refusal makes a
caller's confusion about whether a trie is calculated visible, which is a
diagnostic gain, not a hazard control.

**What it introduces.** An unavailability pathway: a caller that recalculated a
calculated trie, and so used to succeed, now receives an error. No hazard in
the register has this cause. HAZ-8suua9 is the register's unavailability
hazard, but its cause is malformed, hostile or oversized input that makes the
library panic or exhaust memory. Here the cause is a caller error, with no
adversary and no malformed input. By the same reasoning as the PR-vf5hdm
analysis (`2026-09-17-design-derived.md`, item 6), that is not HAZ-8suua9. This
pathway shares only that hazard's harm. The refusal does meet RC-c4truv: it is
a typed error, not a panic.

That analysis found a pathway no HAZ covers, and it concluded that
`analyze-risks` has to rate that pathway. This one owes no rating, because it
differs in two ways:
- **Reach.** The PR-vf5hdm pathway is reached by correct code with a correct
  input. This one is reached only by a caller that is itself in error, and no
  such caller remains in the workspace (below).
- **Detection.** A future caller error would be found in testing, not in the
  field. The refusal is deterministic, and the fuzz properties now assert it
  instead of discarding it.

With no reachable cause there is nothing to rate. This is the author's
assessment, and the owner confirms it by signing the merge.

Every such caller in the workspace is gone. The callers that remained after
master's `0f85cb9` were removed by this change. The full set:
- the sites PR-zqvs7t lists, except the `receive_and_verify` fallback, which
  `0f85cb9` removed;
- the two it missed, fuzz `keys_stay_unique` and org-node `wire_frame_bound`
  (through `test_fixtures.rs`);
- the two `chain_genesis_e2e.rs` sites. These were fixed by inspection and
  compiled, not run, because that target needs chopsticks.

The three fuzz properties `delta_roundtrip`, `calculate_delta_roundtrip` and
`delta_canonicality_fuzz` were changed without going red, so that they no
longer treat the refusal as a no-op. Of the callers this change removed, one
was on an org-node production path: `create_organisation`, which built the
first administrator's trie by `genesis()` and then recalculated it.

If a missed caller remains, it fails closed. The operation returns an error and
changes no Membership record, because `recalculate()` takes `&self` and the
refusal is checked before anything is computed. It never publishes a wrong
root, and never accepts a change set that should have been refused. A fuzz
property that discarded the error with `if let Ok` would hide that failure in
testing; this change removes the ones that did.

**Not a hazard of the check itself.** The check reads only whether a root is
cached (`has_pending_changes()`), the same state `root_hash()` already reads for
REQ-avmu3j. A trie mutated back to its original members still recalculates (see
LLR-j35sxz), so an administrator's add-then-remove sequence is not refused.
HAZ items are not re-scored here.
