# Absence proofs under a Person hash

A revoked DevicePublicKey's absence must be provable under a Person hash, as
it is under a membership root for a Member (owner ruling 2026-10-22), and the
proof hides the Person's other DevicePublicKeys. The device root becomes the
root of the device root tree, a sparse Merkle tree of depth 4, built on the `sparse-merkle-tree` unit (owner
rulings 2026-10-23); the V1 encoding keeps its name and is defined over that
device root, since no Person hash has been published or used outside this
repository (owner ruling 2026-10-23).

A proof is in two steps: the Person hash is recomputed from the Person
definition's other encoded fields and its device root, and the
DevicePublicKey is absent under that device root. The proof discloses the
Name, the Surname and the in-transit key, which the verifier is assumed to
know already (owner ruling 2026-10-22), and no other DevicePublicKey.

**REQ-5kvqcf**: The software shall produce, from a Person definition and a
DevicePublicKey it does not hold, a proof that the DevicePublicKey is not held
under that Person definition's hash, including no DevicePublicKey other than
the given one, and shall refuse, with an error that says so, when the Person
definition holds that DevicePublicKey.
satisfies: derived
exported: yes

**REQ-uh2bzg**: The software shall accept a proof that a DevicePublicKey is
not held under a given Person hash only when the Person hash recomputed from
the proof's encoded fields and device root under the stated encoding version
equals the given one and the DevicePublicKey is absent under that device root;
it shall reject every other proof with an error that says which of these
failed.
implements: RC-ja7gx3
satisfies: derived
exported: yes

A proof shows absence under one Person hash only, not that the key was never
held.

## Collisions in the device root tree

At depth 4 two DevicePublicKeys may share a position. The holder's device
generates its own key pairs, a different one for every record it is in, so it
checks a new key against the one set it will join and generates another on a
collision; the check is part of what the device root tree offers (owner ruling
2026-10-22). At the bound of 8 the last addition meets a collision with
probability 7/16.

**REQ-9sh8qb**: The software shall report, for a set of DevicePublicKeys and a
candidate DevicePublicKey, whether the candidate's position in that set's
device root tree is taken by a key of the set, without changing the set.
implements: RC-5d9qxe
satisfies: derived
exported: yes

**REQ-4e23xr**: The software shall reject adding a DevicePublicKey whose
position in the set's device root tree is taken by another key of the set, with an
error distinct from a duplicate key and from a full set.
implements: RC-5d9qxe
satisfies: derived
exported: yes
