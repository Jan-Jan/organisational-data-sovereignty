# Recalculating a calculated trie is refused

Low-level requirement added 2026-10-03 by owner ruling, in the change that
resolves PR-zqvs7t. "Trie" here is the `OrgTrie` that holds a Membership
record; this item is about its API, so it uses the implementation term. A
node's hash is a write-once cell (LLR-n7nya3), so a request to fill the cells
of a trie whose cells are already filled is a caller error, and the owner ruled
that it is reported. It refines SDD-k5wa4n (the organisation trie,
`src/trie.rs`), where `recalculate()` checks the trie's cached root, and the
variant it returns belongs to SDD-m9gs5g (the typed error, `src/error.rs`). It
is `satisfies: derived`: REQ-avmu3j governs reporting a root, and the refusal
reports none. It is assessed in `../risk/2026-10-04-recalculate-refusal.md`.

A trie is *calculated* when `genesis()`, `recalculate()` or
`CandidateTrie::verify_against()` has returned it and it has not been mutated
since. Mutating it, even back to the same members, makes it pending again.

**LLR-j35sxz**: `recalculate()` on a calculated trie is refused with
`HashesAlreadyCalculated`, a variant distinct from `HashesNotCalculated` and
`InvariantViolated`. A trie mutated since it was calculated recalculates, even
when its change set is empty. satisfies: derived

The test is whether the trie was mutated, not whether its change set is empty.
A trie whose members were added and then removed again has an empty change
set, and recalculating it is not a caller error: the caller did mutate it, and
the result is the original root with an empty change set.
