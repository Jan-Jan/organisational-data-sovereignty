# Sparse Merkle tree hazards

The hazards of the one sparse Merkle tree the Membership record and the device
root are built on (`docs/adr/2026-10-23-one-sparse-merkle-tree.md`). Matrix
and method are this unit's `README.md`. Hazard set confirmed by the owner on
2026-10-23.

## An unsound proof

**HAZ-m5bpf6**: the tree accepts a proof that shows a held leaf as absent,
or a leaf that is not held as included — because the depth is taken from the
proof, a held leaf is told apart from a different one by its position or by
less than what its leaf implementation names as its identity, or a colliding
leaf displaced the one it collided with; a consumer acts on the proof, and a device whose holder still holds its
DevicePublicKey takes it as its revocation and deletes everything it holds for
the Organisation (org-members' false-revocation hazard,
`org-members/docs/risk/2026-10-06-absence-proofs.md`), or a Person's live device is
treated as revoked by a collaborator; the holder cannot reach the records when
a decision needs them. Severity: S3. Probability: P2.

P2: proofs arrive from peers, and the verifier is reachable by anything that
can hand a node bytes, as for the false-revocation hazard it feeds.

**RC-5zyczk**: an absence or inclusion proof is accepted only when it resolves
to the given root at the verifier's own depth, never one read from the proof,
and a held leaf is told apart from a different one by the whole of what its
leaf implementation names as its identity; a leaf is never displaced from its
position, a colliding one being refused. Which leaf a DevicePublicKey or a
MemberId names is the leaf implementation's, not the tree's.
mitigates: HAZ-m5bpf6

What the consumer does with an accepted proof, and against which root it has
it checked, are the consumer's controls (org-members' binding of a proof to one
membership root, org-io's choice of the current on-chain root). This unit's part is that a proof binds
to exactly one root and one depth.

Residual risk: with RC-5zyczk, accepting a proof that contradicts the root
means finding a second preimage of a blake3 node or leaf hash under the
consumer's domain keys. S3/P1, **not acceptable** under the matrix, as for
every S3 in this project's registers (org-members'
`2026-09-02-membership-hazards.md`, "Residual risk"): it is the residual risk
of the hash function itself, which no software control here can reduce.

## Hostile input

**HAZ-7qhav6**: the tree is given a malformed, hostile or oversized proof,
key or leaf — a proof with more siblings than the depth, a bit index past the
depth, a truncated encoding; the tree panics inside the node or the
application and takes the caller down with it, or exhausts memory; the holder
cannot reach the Organisation's or the Person's records when a decision needs
them. Severity: S3. Probability: P2.

Severity and probability as for the consumers' hostile-input hazards
(org-members' `2026-09-02-membership-hazards.md`, person's
`2026-10-06-person-hazards.md`): the input is attacker-chosen, it arrives from
peers, and a panic inside this crate happens before any consumer can turn it
into an error. The path has panicked before: org-members' problem report on
`MemberId::bit` and `DefaultHashes::at_level`, the primitives a verifier
walks, resolved 2026-10-06.

**RC-gy5f5r**: every rejection by the tree is a typed error and no input
causes a panic; a proof's size is bounded by the verifier's depth before any
of it is used. mitigates: HAZ-7qhav6

Residual risk: with RC-gy5f5r, no input reaches a panic or an unbounded
allocation, as the no-panic property tests and the crate's own rule against
slice indexing are to show. S3/P1, **not acceptable** under the matrix, as
for every S3 in this project's registers.

## Derived requirements assessment

Every requirement of this unit is derived: the unit exists because the
Membership record's tree and the device root tree were factored into one.

REQ-ymgg6b and REQ-fmx9e7 (verification at the verifier's own depth, keys
told apart by the whole key) and REQ-6cej5z (a colliding key refused, never
displacing a leaf) realise RC-5zyczk against HAZ-m5bpf6. REQ-vg9wq4 (depth
fixed at build) is what lets a verifier have a depth of its own; a depth
mismatch between producer and verifier fails closed, every proof rejected,
which is no hazard beyond HAZ-m5bpf6's availability side. REQ-uy4euz and
REQ-3szczq (production) create nothing a verifier trusts without REQ-ymgg6b
or REQ-fmx9e7, and they refuse to produce a proof that is false. REQ-s3unp5
bounds what a proof discloses to hashes and the key it is for; its residual,
the confirmation of candidate keys through unsalted leaves, is S1/P1 and
acceptable (owner ruling 2026-10-23, org-members'
`2026-10-23-two-level-absence-proofs.md` and
`docs/adr/2026-10-23-unsalted-tree-leaves.md`). An inclusion proof discloses
its leaf, which the consumer chooses and analyses (org-members'
`2026-10-23-two-level-absence-proofs.md`). REQ-dyjq33 realises
RC-gy5f5r against HAZ-7qhav6.
assesses: REQ-vg9wq4, REQ-6cej5z, REQ-uy4euz, REQ-ymgg6b, REQ-s3unp5, REQ-3szczq, REQ-fmx9e7, REQ-dyjq33

### Derived design items (2026-10-24)

LLR-kep2ju (a single-leaf subtree stored as one node, hashed through the
empty-subtree siblings) is the one with safety weight. A wrong hash there
makes the stored tree's root differ from the classic root that LLR-53ezgd and
LLR-c7bdee fold, which verify along the classic tree: proofs the tree
produces then fail against its own root, and so does every comparison of
roots with another implementation. That failure is closed, so it adds an
availability case to HAZ-m5bpf6's harm and no case of a held leaf shown
absent; it is caught by checking the root of every generated tree against a
classic reference that stores every node, and by the golden roots.
LLR-yuyn8f (the empty-subtree hashes, with a typed error above `DEPTH`),
LLR-3zffc2 (the root depends on the leaf set alone) and LLR-u2r4vj (the tree-changes
walk's order, which org-members' canonical change sets rely on) are carried
over from org-members' store unchanged in meaning; LLR-yuyn8f's error is a
case of RC-gy5f5r. LLR-jd4phg (one `NodeHash` for the workspace) and
LLR-c4eunm (no dependency beyond `spin` and an optional `serde`) change no
hazard: the first removes a parallel type, the second keeps the SOUP surface
to the two rows of this unit's inventory.
assesses: LLR-kep2ju, LLR-yuyn8f, LLR-3zffc2, LLR-u2r4vj, LLR-jd4phg, LLR-c4eunm

**Amended 2026-10-24 (plan `docs/plans/2026-10-24-sparse-merkle-tree.md`,
T1).** LLR-yuyn8f now computes the empty-subtree hashes once per empty tree
and once per verification rather than once per type, and LLR-u2r4vj walks
uncalculated trees too, skipping a pair only when it is one node or both
hashes are set and equal. No hazard impact: neither changes a value the tree
or a proof produces; the first changes when a value is computed, the second
skips no pair the calculated-only rule would have walked, and a pair it
skips is unchanged by construction (one node) or by hash.
