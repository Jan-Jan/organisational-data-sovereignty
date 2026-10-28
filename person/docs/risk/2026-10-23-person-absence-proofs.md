# Absence proofs under a Person hash, and device-root collisions

Hazards of `person/docs/requirements/2026-10-23-person-absence-proofs.md`.
Matrix and method are this unit's `README.md`. Hazard set confirmed by the
owner on 2026-10-23.

## A forged Person-level proof

**HAZ-2mwaad**: a proof that a DevicePublicKey is not held under a Person hash
is accepted although the Person definition behind that hash holds it — the
recomputed Person hash is not compared with the given one, the proof's
encoding version is not the one the hash was made under, or the device-root
step is unsound; a collaborator treats the Person's live device as revoked and
refuses what it sends; the Person cannot act with that collaborator when a
decision needs it. Severity: S3. Probability: P2.

P2: a proof is handed to a collaborator by whoever holds it, so the verifier
is reachable by anything that can hand it bytes.

**RC-ja7gx3**: a proof under a Person hash is accepted only when the Person
hash recomputed from the proof's encoded fields and device root, under the
stated encoding version, equals the given one, and the DevicePublicKey's
absence under that device root verifies. mitigates: HAZ-2mwaad

The device-root step rests on sparse-merkle-tree's control of an unsound
proof (`sparse-merkle-tree/docs/risk/2026-10-23-sparse-merkle-tree.md`).

Residual risk: with RC-ja7gx3 and that control, accepting a forged proof means
finding a second preimage of blake3 under the Person definition's domain key
or a device-root domain key. S3/P1, **not acceptable** under the matrix, as
for every S3 here: it is the residual risk of the hash function, which this
unit's `blake3` SOUP row already defers to.

## A collision

**HAZ-vjm5fg**: a DevicePublicKey whose position in a device root tree is taken by
another key of the set cannot be added; the device that generated it is not
added to the Person or Member until it holds another key; the holder is
delayed in using that device. Severity: S1. Probability: P2.

P2: at depth 4 and the bound of 8, the last addition meets a collision with
probability 7/16.

**RC-5d9qxe**: a collision is reported for a candidate key before it is added,
and an attempt to add a colliding key is refused with an error of its own, so
the device that generated the key generates another. mitigates: HAZ-vjm5fg

Residual risk: a retry of a local key generation, S1/P2, **acceptable**
(owner ruling 2026-10-23: the friction is rare and stays on the generating
device).

## Derived requirements assessment

REQ-uh2bzg realises RC-ja7gx3 against HAZ-2mwaad; REQ-9sh8qb and REQ-4e23xr
realise RC-5d9qxe against HAZ-vjm5fg. REQ-5kvqcf (production) creates nothing
a collaborator trusts without REQ-uh2bzg, and refuses to produce a proof for a
key the Person holds. What it discloses: the Name, the Surname and the
in-transit key, which the collaborator verifying it already knows (owner
ruling 2026-10-22) and which the Person hash, unsalted and published, already
lets anyone confirm (`docs/adr/2026-10-04-person-hash-unsalted.md`); no other
DevicePublicKey; and device-root hashes from which a verifier holding a
candidate DevicePublicKey of the Person can confirm it, S1/P1 and acceptable
(owner ruling 2026-10-23, `docs/adr/2026-10-23-unsalted-tree-leaves.md`). The
proofs add to the hostile-input surface of HAZ-y6ft54.
assesses: REQ-5kvqcf, REQ-uh2bzg, REQ-9sh8qb, REQ-4e23xr
