# Problem reports — `apply_delta` refuses a lawful handle handover

**PR-vf5hdm**: `apply_delta` rejects a well-formed, honestly produced change set
with `DuplicateHandle` when that change set moves a handle from one member to
another — always for a two-way swap, and for a one-way handover whenever the
member acquiring the handle sorts before the member releasing it — so a lawful
membership change cannot be replicated and producer and receiver diverge.
affects: SDD-55b2zj, LLR-juxk9q, LLR-xmpqn2, LLR-8jttpb, LLR-h7stq2.
opened: 2026-09-17
status: resolved

Resolved 2026-10-03 by the change on `worktree-quint-connect-coupling`
(REQ-wx3wpv, LLR-n5t6bn): root cause the single-pass upsert loop above; fix a
first loop in `OrgTrie::apply_delta` that releases every outgoing handle from
both indexes before the second loop checks any incoming one. Reproducing tests
in `org-members/tests/mbt_conformance.rs`: `scenario_handover_a_to_b`,
`scenario_handle_swap`, `scenario_handover_to_new_member` (red before the fix,
green after) and `scenario_handover_b_to_a` (the safe order, green throughout).

Found by the second independent review of the architecture change
(`merge-change` step 6a, finding-2 of that round), not by a failing test. No
adversary, no malformed input, no hostile encoding: every input on the path is
what an honest producer of this crate emits.

## The mechanism

`OrgTrie::apply_delta` in `org-members/src/trie.rs` processes `delta.removed`
first and `delta.upserted` afterwards, in a **single pass in ascending
identifier order**. Within the upsert loop it releases a leaf's *old* handle
only when it reaches that leaf:

    for member in &delta.upserted {
        let existing = smt::get_member(&root, member.id());
        if let Some(ref old) = existing {
            if old.handle() != member.handle() {
                new_skeleton_index.remove(&handle_skeleton(old.handle()));
                new_handle_index.remove(old.handle());
            }
        }
        let needs_check = …;
        if needs_check {
            let skeleton = handle_skeleton(member.handle());
            if let Some(existing_handle) = new_skeleton_index.get(&skeleton) { … }
            …
        }
        …
    }

So a handle **in flight between two members** is still in
`new_skeleton_index` — held by the member who has not been reached yet — at the
moment the acquiring member is checked. The lookup finds it, the found handle
equals the one being acquired, and the branch returns `DuplicateHandle`.

Removals do not have this problem: they are drained completely before the first
upsert, so a handle freed by a departing member is available to every upsert.
The defect is specific to a handle passing between two members that are **both**
still present, which is exactly the upsert–upsert case.

## The two shapes

**A two-way swap fails in both identifier orders.** Members `A` and `B` hold
`alpha` and `bravo`; the change gives `A` `bravo` and `B` `alpha`. Whichever of
the two has the lower identifier is processed first, releases its own old handle,
and then collides with the other member's still-indexed handle. The order is
symmetric, so there is no ordering of the two upserts that succeeds — and
canonical form (LLR-xmpqn2) does not permit reordering them anyway.

**A one-way handover fails in one order of the two.** The giver renames
`bravo` → `charlie`; the taker renames `alpha` → `bravo`. If the giver's
identifier sorts first, the giver releases `bravo` before the taker is checked
and the delta applies. If the **taker's** identifier sorts first — which is a
property of two 32-byte identifiers and nothing anyone chooses — the taker is
checked while the giver still holds `bravo`, and the delta is refused.

## Why it is a divergence and not merely a refusal

Every step before the refusal is lawful and every step before the refusal
succeeds:

1. The producer performs both renames **sequentially** through `update_handle`.
   Each one is individually legal — the first frees a handle, the second takes
   it — so the producing node's trie reaches the new state with no error.
2. `calculate_delta` emits a canonical two-leaf change set for that transition:
   both leaves genuinely changed, identifiers strictly increasing, removals
   empty, the two sets disjoint (LLR-8jttpb, LLR-h7stq2).
3. The receiver, holding a trie **byte-identical** to the producer's base and
   the correct `base_root`, refuses it with `DuplicateHandle`.

The producer has advanced; the receiver cannot. There is no retry that helps and
no recovery inside this crate: the same delta fails every time, and the
membership records stay apart for as long as the handle assignment stands.

## What it contradicts

- **SDD-55b2zj** is "the change set that crosses a process boundary — anchored
  to the record it was computed against, canonical in its encoding, and usable
  only after its result is verified against a root supplied independently of
  it." A change set this crate produced, applied to the record it was computed
  against, is not usable. The item's whole purpose is that crossing, and there
  is a class of lawful transition it cannot carry.
- **LLR-juxk9q** — "an upserted member record is checked for handle uniqueness
  and confusability when the change set is applied, not when it is decoded" — is
  the item whose check does the refusing. The item states *that* the check runs;
  it does not state that the check is correct for a multi-leaf change set, and
  this is where that silence bites. The check is sound leaf-by-leaf and unsound
  over the set.
- **LLR-xmpqn2** and **LLR-8jttpb** are implicated through the design claim
  built on them rather than through their own text. Both are true as written.
  What was inferred from them — that an honest producer never trips an honest
  receiver's acceptance checks — is false, and both the decomposition ledger and
  the LLR-8jttpb risk assessment have been corrected to say so and to cite this
  report.
- **LLR-h7stq2** — `calculate_delta(old)` produces the change set transforming
  `old` into the receiver — holds. The change set it produces is the right one.
  It is named here because it is the producer half of the divergence and a
  reader tracing the path will arrive at it.

**No requirement is contradicted, and that is itself part of the finding.**
There is no REQ in `org-members/docs/requirements/` saying that a change set the
software produces shall be accepted by a receiver in the state it was computed
against. REQ-4umsuz governs base and root *mismatch*; REQ-kmvc96 governs
admitting or renaming to a handle **already held by another member**, which after
the handover is no longer the case; REQ-ds8ryr governs reporting rejections as
errors, which this does correctly. The requirement set has a hole where
"a lawful change replicates" would be, and every gate is green over it.

## No hazard in the register covers this

Recorded here and in `../risk/2026-09-17-design-derived.md` as a finding for
`analyze-risks`. Neither existing hazard reaches it:

- **HAZ-8suua9** is hostile or oversized input reaching the membership record.
  There is no hostile input here. The input is what this crate's own
  `calculate_delta` produced from two of its own `update_handle` calls.
- **HAZ-y8h835** is the closest, and it still does not reach it. Its hazardous
  situation — a device's view diverging from the committed record — is adjacent,
  but every cause it names is lag-shaped: concurrent administrators, a delta
  applied to the **wrong** base, a root published before it was recomputed, a
  device that has not read the chain recently. This delta is applied to the
  **right** base and is still refused. More decisively, that hazard's
  probability and acceptability argument rests on recoverability — "the chain
  anchor is what makes it recoverable" — and recoverability is exactly what is
  absent here: the receiver has already received the change, refuses it, and will
  refuse the identical bytes on every retry, forever. Re-reading the anchor
  changes nothing. A hazard whose acceptance rests on an argument that does not
  hold for this pathway does not cover this pathway.

The harm is the register's own — a member wrongly denied access, or granted
access under a handle the rest of the organisation no longer associates with
them, on a replica that believes it is current. It is reached with **no
adversary and no fault in any component**, which is the pathway the register
does not currently contain. **No HAZ and no RC is minted here**: minting either
owes an acceptability evaluation against the matrix, and that is `analyze-risks`
work, not a documentation fix round's.

## Not fixed in this change, deliberately

The owner's disposition on 2026-09-17 was to file this **without touching the
code**. The round that found it is a review-fix round: it corrects claims and
writes no production code, adds no test, and changes no behaviour. A fix here is
a real behaviour change — the plausible shape is a two-phase upsert loop that
releases every outgoing handle from the indexes before checking any incoming one
— and under `resolve-problem` it starts with a test that reproduces the refusal,
which this round is not permitted to add. That is the same disposition PR-zz4exm
carried from 2026-08-31 until its resolution on 2026-10-03, and PR-jq43gx has
carried since 2026-09-17.

When it is fixed, the reproducing test is cheap and needs no new machinery: build
a two-member trie, swap the handles through two `update_handle` calls, take
`calculate_delta` against the original, and `apply_delta` it to the original.
That is the red. The one-way variant needs the same with identifiers chosen so
the taker sorts first.

## Resolution note (2026-10-03)

The owner chose not to forbid handle swaps or handovers between members
(REQ-wx3wpv's draft file): a handle may move between two present members in one
honest delta, or to a member the same delta admits. The fix is the two-phase
upsert loop this report anticipated — release every outgoing handle, then check
every incoming one — so the check runs against the record after the whole set,
in any identifier order. The hazard is HAZ-y8h835 as amended by this change. The
handover to an admitted member was not in this report's original two shapes; the
random conformance run found it (seed 0xa68bbc85: handle c->h3, add b h1), and
`scenario_handover_to_new_member` now pins it.
