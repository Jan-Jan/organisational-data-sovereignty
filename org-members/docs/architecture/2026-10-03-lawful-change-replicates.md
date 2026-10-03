# Architecture — the set-level handle check on applying a change set

One low-level requirement under an existing software item, SDD-55b2zj (delta
exchange and the trust boundary, defined in `2026-09-17-decomposition.md`).
No new software item, no new SOUP. Decided 2026-10-03 (`design-architecture`),
for the change recorded by `docs/adr/2026-10-03-quint-conformance-gate.md`.

LLR-juxk9q stays as written: it states *when* the check runs (at apply, not at
decode), which this change does not alter. What PR-vf5hdm found missing is
*what the check is against* — the check was sound leaf by leaf and unsound over
the set — and that is this item.

**LLR-n5t6bn**: applying a change set checks handle uniqueness and
confusability against the record as it stands once every removed member's
handle and every upserted member's outgoing handle has been released, so the
outcome does not depend on the order of the members' identifiers.
satisfies: REQ-wx3wpv

Carriers (written by the implementing change): named scenarios for a two-way
swap and for a one-way handover in both identifier orders (acceptance), and
`apply_delta` rejection tests for collisions that survive the release
(abnormal input: two upserts claiming one handle, an upsert taking an untouched
member's handle or a confusable of it). The conformance test's random run over
honest deltas of one or two mutations is supporting evidence only, not a
carrier: with the fix reverted it went red in 2 of 5 runs (measured
2026-10-03), so it carries no `verifies:` for this item.
