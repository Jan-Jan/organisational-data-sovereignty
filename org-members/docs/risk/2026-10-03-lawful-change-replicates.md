# Derived requirement assessment — a lawful change set replicates

Assesses the one derived requirement written by the change recorded in
`docs/adr/2026-10-03-quint-conformance-gate.md`. **No hazard and no risk
control is minted here**: the owner decided on 2026-10-03 (`analyze-risks`
interview) that the pathway PR-vf5hdm describes is a cause of an existing
hazard, HAZ-y8h835, and amended that hazard's text in its defining file
(`2026-09-02-membership-hazards.md`) rather than founding a new one.

## Derived requirements assessment

### REQ-wx3wpv — a change set the software produced is accepted at the record it was computed against

assesses: REQ-wx3wpv

Affects an existing hazardous situation; introduces no new hazard; changes no
control's stated effectiveness.

**HAZ-y8h835 (views diverge) — reduced.** PR-vf5hdm is a cause of that hazard
with no adversary and no fault: `apply_delta` refuses, with
`DuplicateHandle`, a delta this crate's own `recalculate()` or
`calculate_delta` produced, when it moves a handle between two members who are
both still present (always for a swap; for a one-way handover whenever the
taker's identifier sorts first), or hands a handle to a member the same delta
admits (`scenario_handover_to_new_member`; whenever the newcomer's identifier
sorts before the releasing member's). The record is immutable (REQ-d3prca), so the
refusal commits nothing wrong: every party keeps the old committed root, and a
candidate is discarded unless it applies and verifies against the chain. The
cost is that the lawful change cannot replicate by delta — blocked at
co-administrator review, or, if a root is published without that review,
every member lags identically until it obtains a complete trie from an
administrator. REQ-wx3wpv removes the cause by design (ISO 14971's first
priority): the delta is accepted in every identifier order. It is carried,
through LLR-n5t6bn, by named scenarios for the swap and both orders of the
one-way handover, and on the abnormal-input side by `apply_delta` rejection
tests for collisions that survive the release. The conformance test's random
run over honest deltas of one or two mutations is supporting evidence only:
with the fix reverted it went red in 2 of 5 runs.

**HAZ-jkc6tj (handles a person cannot tell apart) — not worsened.** The owner
noted that a member ever expecting a swap is not foreseen, and that a swap
would confuse people about whom they address — HAZ-jkc6tj's own pathway, "re-use
of a departed member's name", extended to a present one. REQ-wx3wpv does not
create that state: the producer already reaches it through sequential
`update_handle` calls, each individually lawful, and RC-n2taat's uniqueness
and confusability rules hold over the resulting record whether it arrives by
delta or not. What REQ-wx3wpv changes is only that receivers agree with the
producer. The owner chose not to forbid swaps or handovers, to keep the set of
failures small; HAZ-jkc6tj's residual-risk statement already places the
remaining protection — the administrator's surface showing the stable
identifier, not only the handle — outside this codebase, and it covers this
case unchanged.

**RC-9z65hw (base and root must match) — unaffected.** REQ-wx3wpv concerns a
delta at the **correct** base whose result verifies; RC-9z65hw's refusals of a
wrong base or a wrong result root are untouched, and the membership model's
stale-base and verify-fail deltas keep checking them.

**Residual for this cause.** Recovery from any *other* refusal still rests on
a full-trie fallback that is org-node's to implement and is not implemented
(recorded in HAZ-y8h835's amendment). That is a pre-existing gap in the
consumer, not one this requirement opens or closes; it is noted here so the
assessment does not read as closing it.

## Measured carriers

Measured 2026-10-03, quint 0.33.0, against the conformance test as this change
leaves it (`org-members/tests/mbt_conformance.rs`: 24 tests plus the ignored
`preflight_probe`). Method as in `2026-09-17-design-derived.md` ("Measured
negatives"): mutate the claimed clause in `src/`, run the named target, record,
revert (`git diff` empty before the next row). "×5" is five runs of the random
`membership_conformance` (100 traces × 15 steps, fresh seed each), red if any
run fails on a model/implementation divergence. A `verifies:` was added only
where a row went red.

| LLR | Mutation in `src/` | Run | Outcome | Annotated |
|---|---|---|---|---|
| LLR-ch2pkw | `genesis`: delete the `DuplicateId` return | `membership_conformance` ×5 | red 5/5, divergence | `membership_conformance` |
| LLR-ch2pkw | `genesis`: delete both handle-collision returns | `membership_conformance` ×5 | red 5/5, divergence | (same) |
| LLR-ch2pkw | `genesis`: delete only the `ConfusableHandle` return | `membership_conformance` ×5 | **green 5/5** — measured negative | — |
| LLR-4n8zqx | `smt::insert`: XOR the trie's leaf count at insertion into the leaf's device root (order-dependent) | `membership_conformance` ×5 | red 5/5: 2 divergence (receiver `VerificationFailed` on an honest delta), 3 the round-trip `expect` in the driver's `commit` | `membership_conformance` |
| LLR-au8het | `apply_delta`: delete the `DeltaBaseMismatch` return | `scenario_apply_stale`; `membership_conformance` ×5 | red; red 5/5 | `scenario_apply_stale` |
| LLR-7tdqv9 | `verify_against`: delete the `VerificationFailed` return (always `Ok`) | `scenario_apply_verify_fail`, `scenario_built_verify_fail`; `membership_conformance` ×5 | red, red; red 5/5 | both scenarios |
| LLR-xmpqn2 | delete the stale-removal check | `membership_conformance` ×5; `scenario_remove_upsert_overlap` | red 5/5 (driver `Other:InvariantViolated` vs model `StaleRemoval` — the apply loop's own guard, not the canonical check); green | `membership_conformance` |
| LLR-xmpqn2 | delete the no-op upsert check | `membership_conformance` ×5; `scenario_remove_upsert_overlap`; `scenario_no_op_before_overlap` | red 5/5; green; red | `membership_conformance`, `scenario_no_op_before_overlap` |
| LLR-xmpqn2 | delete the remove/upsert overlap check | `membership_conformance` ×5; `scenario_remove_upsert_overlap`; `scenario_no_op_before_overlap` | red 5/5; red; green | `membership_conformance`, `scenario_remove_upsert_overlap` |
| LLR-juxk9q | `apply_delta` second loop: delete the handle-check block | `membership_conformance` ×5 | red 5/5, divergence | `membership_conformance` |
| LLR-juxk9q | same loop: delete only the `DuplicateHandle` return | `membership_conformance` ×5 | red 5/5, divergence | (same) |
| LLR-n5t6bn | revert T7's first loop (pre-fix single loop) | `scenario_hand*` (4 tests) | `scenario_handover_a_to_b`, `scenario_handle_swap`, `scenario_handover_to_new_member` red; `scenario_handover_b_to_a` green (the safe order, as predicted) | already annotated in T7 |
| LLR-n5t6bn | same reversion | `membership_conformance` ×5 | red 2/5 (independent review, round 1) — supporting evidence only | — |
| LLR-n5t6bn | `apply_delta`: empty the release loop (the first `delta.upserted` loop's body removed, no outgoing handle released) | `scenario_hand*` (4 tests) | all four red, `scenario_handover_b_to_a` included (measured, independent review round 3; reproduced 2026-10-03 after the base merge, 4 failed of 4, divergence panic at `mbt_conformance.rs:616`). So `scenario_handover_b_to_a`'s `verifies:` is earned by the release clause: green under the reverted single loop (safe identifier order), red with the release loop emptied | `scenario_handover_b_to_a` (already annotated in T7, now measured) |

What the table settles, and what it leaves:

- **LLR-juxk9q's uniqueness half is now carried.** The 2026-09-17 file recorded
  it uncarried (every `DuplicateHandle` assertion reached the check through
  `genesis`, `add_member` or `update_handle`); the model's `ApplyDelta` reaches
  it through `apply_delta`, and deleting that one return reds every run. The
  same deletion also reds `apply_delta_rejects_two_upserts_claiming_one_handle`,
  `apply_delta_rejects_upsert_taking_handle_of_untouched_member` and the
  proptest `apply_delta_never_admits_a_handle_collision`, which now carry
  `verifies: LLR-juxk9q` as well (independent review, round 1).
- **LLR-ch2pkw is now reached**, reversing the 2026-09-17 finding that it was
  structurally out of the model's reach: genesis is the model's `init` with
  generated seeds, so `genesis`'s per-member loop runs.
- **Measured negatives — the confusable (skeleton) clauses** of LLR-ch2pkw and
  LLR-juxk9q. The model's skeleton is its handle, so it never produces two
  distinct handles with one skeleton, and the driver maps the crate's
  `ConfusableHandle` to `Other` (decision 10). They stay carried by the handle
  tests in `integration_test.rs`, not by this test.
- **LLR-xmpqn2's two ordering clauses** (removals and upserts strictly
  increasing) are on the conformance test's declared boundary: the driver's
  `built` deltas are sorted and honest deltas are canonical by construction
  (LLR-8jttpb). Not mutated here; carried by `delta_canonicality_fuzz`.
- **Reds for the wrong reason, recorded as in 2026-09-17**: the stale-removal
  deletion surfaces as `InvariantViolated` from the apply loop rather than as
  `MalformedDelta`; three of the five LLR-4n8zqx reds are the driver's own
  round-trip `expect`, a harness panic before the model comparison.
- **Extra reds not annotated** (beyond the plan's carrier column):
  `membership_conformance` also reds under the LLR-au8het and LLR-7tdqv9
  mutations (5/5 each).
