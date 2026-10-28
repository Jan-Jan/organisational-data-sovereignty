# Dependency edges onto `sparse-merkle-tree`

Assessment of two edges, org-members -> sparse-merkle-tree and
person -> sparse-merkle-tree, walked with the owner on 2026-10-24 in the
`design-architecture` step of change `worktree-generic-smt` (Section 3 of the
design, agreed that day), under `grill-requirements`, "Declaring a
dependency".

Both consumers build on the provider in the same way: the Membership record is
a `SparseMerkleTree` of depth 256 over the member leaf, a device root one of
depth 4 over the device leaf, and both proofs are the provider's proofs
composed (`docs/adr/2026-10-23-one-sparse-merkle-tree.md`). The answers below
hold for both edges.

## Export surface

`check-units.sh --exports sparse-merkle-tree` lists all eight of the unit's
requirements, each `exported: yes`, in
`sparse-merkle-tree/docs/requirements/2026-10-23-sparse-merkle-tree.md`:
depth fixed at build (REQ-vg9wq4), collisions refused (REQ-6cej5z), absence
proofs produced and verified (REQ-uy4euz, REQ-ymgg6b) and what they disclose
(REQ-s3unp5), inclusion proofs produced and verified (REQ-3szczq,
REQ-fmx9e7), and hostile input (REQ-dyjq33). Every behavior either consumer
needs is on it; neither consumer has a gap to write as `expects:`.

## Risk file

`sparse-merkle-tree/docs/risk/2026-10-23-sparse-merkle-tree.md` analyses both
uses: an unsound proof (HAZ-m5bpf6, RC-5zyczk) with the two consumers' harms
named in it — a device deleting its Organisation's data on a false
revocation, a Person's live device treated as revoked — and hostile input
(HAZ-7qhav6, RC-gy5f5r). Neither use falls outside an analysed situation.

## ADRs

`docs/adr/2026-10-23-one-sparse-merkle-tree.md` and
`docs/adr/2026-10-23-unsalted-tree-leaves.md`. Neither forecloses either use;
the first is the reason for both edges.

## Open problem reports

None (`sparse-merkle-tree/docs/problems/` holds only its README).

## SOUP, transitively

`spin` 0.9.9 and, under the `serde` feature, `serde` 1.0.229
(`sparse-merkle-tree/docs/architecture/soup.md`). Both are already in both
consumers' builds: `spin` 0.9.9 in org-members, which hands it to the
provider, and `serde` in both. The provider depends on no hash function and on
no other unit (LLR-c4eunm), so neither edge brings a further transitive unit.

## Classes

Provider and both consumers are class C: no `segregated_from:` is needed, and
the class floor has nothing to convict.

## Decision

Both edges declared on 2026-10-24. person's config, which recorded that it
depends on no unit, now depends on this one; the rule it recorded — org-members
may depend on person, never the reverse — is unchanged.
