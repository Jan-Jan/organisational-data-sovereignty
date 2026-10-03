# Membership change replication requirement

Fills the hole PR-vf5hdm (`../problems/2026-09-17-review2-fixes.md`) names:
no requirement said that a change set the software produces is accepted by a
receiver holding the record it was computed against. Decided 2026-10-03 in
the `grill-requirements` interview recorded by
`docs/adr/2026-10-03-quint-conformance-gate.md`.

The owner chose not to forbid handle swaps or handovers between members: a
change set that moves a handle between two present members is lawful, and the
fix removes a rejection rather than adding one, keeping the set of failures
small.

## Replication

**REQ-wx3wpv**: The software shall accept a set of membership changes that it
produced from a membership record when that set is applied to that same
record, and the result shall hold the same membership as the record the
producer reached.
satisfies: derived
exported: yes
