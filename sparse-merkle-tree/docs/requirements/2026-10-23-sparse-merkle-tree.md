# The sparse Merkle tree

The one sparse Merkle tree the Membership record (org-members) and the device
root (person) are built on, so that there is one implementation to test and
audit (owner ruling 2026-10-22). Each consumer chooses the leaf it holds, the
hash domains and the depth; the domain rules — who may hold what, and the
errors that name them — stay with the consumer.

The boundary between the tree and its leaf (owner ruling 2026-10-23): the
leaf decides what its key is, how it is hashed, how a held key is told apart
from a different one, and what of it a proof discloses; the tree decides the
positions, the hashing from leaf to root and the shape of a proof. A Member's
leaf commits to its device root, so a proof about a DevicePublicKey under a
membership root is a proof in two steps, the second under that device root.

Every item here is derived: the unit exists because the two consumers' trees
were factored into one, and its behavior is what both of them need from it.
The system need behind it is the provable revocation of a DevicePublicKey or
a MemberId, which the consumers' own requirements state.

## Depth

**REQ-vg9wq4**: The software shall hold leaves in a sparse Merkle tree of the
depth its consumer fixes when it builds the software, so that two consumers
may use trees of different depths.
satisfies: derived
exported: yes

The depths in use: 256 for the Membership record, 4 for a device root tree (owner
rulings 2026-10-22/23). A depth is fixed in the consumer's code, not chosen at
run time, and a change of depth changes every root at that depth, so the
consumer that changes one versions it.

## Collisions

**REQ-6cej5z**: The software shall refuse to add a leaf whose position holds a
leaf of a different identity, with an error that names a collision and no
other cause, and shall leave the tree unchanged.
(implements: RC-5zyczk)
satisfies: derived
exported: yes

A leaf is never displaced. What a leaf's identity and position are is its
consumer's leaf implementation's (owner ruling 2026-10-23); the holder of the
leaf being added avoids the collision by choosing another identity — a
DevicePublicKey's holder generates a new
key pair, which a device does for itself (owner ruling 2026-10-23); at depth 4
a holder of three DevicePublicKeys meets a collision on about one addition in
eight.

## Absence proofs

**REQ-uy4euz**: The software shall produce, for a leaf identity and a tree
whose hashes are calculated, a proof that no leaf of that identity is held
under the tree's root, and shall refuse, with an error that says which, when
such a leaf is held or the hashes are not calculated.
satisfies: derived
exported: yes

**REQ-ymgg6b**: The software shall accept an absence proof for a leaf identity
only when the proof resolves to a given root at the verifier's own depth and
shows that the identity's position is empty or holds a leaf of a different
identity; it shall reject every other proof with an error that says which of
these failed.
(implements: RC-5zyczk)
satisfies: derived
exported: yes

The depth is the verifier's, never read from the proof: a proof built at one
depth is rejected by a verifier of another. Depths that vary by Organisation,
published on-chain, are a possible future need and not this unit's today
(owner ruling 2026-10-23); should they come, the verifier takes the depth from
a trusted source, never from the proof. A position holding a leaf of a
different identity is told apart from one holding the identity itself by the
whole identity, so no leaf that is held can be shown absent. A proof shows
absence at one root only, not that a leaf was never held, as for the
Membership record's absence proofs
(org-members/docs/requirements/2026-10-06-absence-proofs.md).

**REQ-s3unp5**: The software shall include in an absence proof no leaf
identity other than the one the proof is for, and no content of any leaf;
every other part of the proof shall be a hash.
satisfies: derived
exported: yes

What a revocation proof must not disclose is the holder's other
DevicePublicKeys (owner ruling 2026-10-22). The leaves are not salted (owner
ruling 2026-10-23): a verifier who already has a candidate key can hash it and
test whether a proof's hashes were made from it, so the proof hides the
holder's other keys from a verifier who has none of them, not from one who
has met the holder's devices. That residual disclosure is the owner's
accepted risk, recorded in this unit's risk file.

## Hostile input

**REQ-dyjq33**: The software shall report every rejected operation, every
rejected proof included, as an error, shall not panic on any input, and shall
reject a proof larger than its depth allows before using any of it.
(implements: RC-gy5f5r)
satisfies: derived
exported: yes

## Inclusion proofs

A DevicePublicKey's absence under a Membership root is shown in two steps
(owner ruling 2026-10-23): the Member's leaf, which commits to its device
root, is included under the Membership root, and the DevicePublicKey is absent
under that device root. The first step needs an inclusion proof.

**REQ-3szczq**: The software shall produce, for a leaf identity and a tree
whose hashes are calculated, a proof that the leaf of that identity is held
under the tree's root, and shall refuse, with an error that says which, when
no leaf of that identity is held or the hashes are not calculated.
satisfies: derived
exported: yes

**REQ-fmx9e7**: The software shall accept an inclusion proof for a leaf only
when the proof resolves to a given root at the verifier's own depth with that
leaf at its identity's position; it shall reject every other proof with an
error that says which of these failed.
(implements: RC-5zyczk)
satisfies: derived
exported: yes

An inclusion proof discloses the leaf it is for, which its consumer chooses;
the Membership record's leaf commits to its device root and holds no
DevicePublicKey.
