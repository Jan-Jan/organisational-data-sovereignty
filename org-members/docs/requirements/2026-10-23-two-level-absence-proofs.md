# Two-level absence proofs

A revocation proof hides the holder's other DevicePublicKeys (owner ruling
2026-10-22). The absence proof of
`2026-10-06-absence-proofs.md` ends in the Member's whole leaf, its
DevicePublicKeys included; its successor shows a DevicePublicKey's absence in
two steps, both built on the sparse Merkle tree of the `sparse-merkle-tree`
unit (owner ruling 2026-10-23): the Member's leaf, which commits to its device
root and holds no DevicePublicKey, is included under the membership root, and
the DevicePublicKey is absent under that device root. A MemberId that holds
no Member is shown, as before, by its absence under the membership root.

REQ-yyuxh8 (no other Member's data) and the refusals of the old items carry
over unchanged.

## Production

**REQ-sb2frw**: The software shall produce, from a calculated membership
record, an absence proof for a given MemberId and DevicePublicKey whenever
that DevicePublicKey is not held by the Member under that MemberId, consisting
either of the MemberId's absence under the membership root or of the Member's
leaf included under the membership root and the DevicePublicKey's absence
under that leaf's device root; it shall include no DevicePublicKey other than
the given one, and shall refuse, with an error that says which, when the record
is not calculated or when that Member holds that DevicePublicKey.
supersedes: REQ-535jcd
(implements: RC-5nka8z)
satisfies: derived
exported: yes

## Verification

**REQ-65xrgh**: The software shall accept an absence proof for a given MemberId
and DevicePublicKey only when it shows, under a given membership root, either
that the MemberId holds no Member, or that the Member's leaf under that MemberId
is included and that the DevicePublicKey is absent under that leaf's device
root; it shall reject every other proof with an error that says which of these
failed.
supersedes: REQ-tk2qqj
(implements: RC-j2znx8)
satisfies: derived
exported: yes

RC-j2znx8's wording is unchanged by the second step: the proof still binds to
exactly one membership root, through the included leaf's device root.

## Collisions in a Member's device root tree

A Member's device root tree is a sparse Merkle tree of depth 4, in which two
DevicePublicKeys may share a position; the device that generated the key
generates another (owner ruling 2026-10-22). In the membership record itself
a MemberId is its own position, so a collision there is a duplicate MemberId,
already rejected (REQ-crjxk8).

**REQ-2sz7j8**: The software shall reject adding a DevicePublicKey to a Member
when its position in that Member's device root tree is taken by another of the
Member's DevicePublicKeys, with an error distinct from a duplicate key and
from a Member at the bound, and shall leave the membership record unchanged.
satisfies: derived
exported: yes
