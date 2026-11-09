# Two-level absence proofs: what a proof discloses

Hazards of `org-members/docs/requirements/2026-10-23-two-level-absence-proofs.md`.
Matrix and method are this unit's `README.md`. Hazard set confirmed by the
owner on 2026-10-23.

The Member's leaf no longer carries DevicePublicKeys: it commits to a device
root, and a DevicePublicKey's absence is shown under that root
(`docs/adr/2026-10-23-one-sparse-merkle-tree.md`). HAZ-r5tc4m replaces the
earlier disclosure hazard, whose wording listed the leaf's Device keys.

## Member data disclosed through a proof

**HAZ-r5tc4m**: an absence proof for a DevicePublicKey removed from a Member
who remains carries that Member's leaf (MemberId, handle, name, surname,
member key, device root) and the hashes of that device root; a proof that
reaches anyone other than a former device of that Member discloses that
Member's identity and member key; exposure of a protected identity in the
deployments this project is classified for. Severity: S3. Probability: P2.

P2: a proof is produced on request, and without a control
the content would be whatever the producer chose to include.

Mitigated by RC-qa2758 (only the data of the Member under the proof's
MemberId, amended to name this hazard) and by:

**RC-5nka8z**: an absence proof carries no DevicePublicKey other than the one
it shows absent; the Member's leaf commits to its device root and carries
none, and the device-root step carries only hashes.
mitigates: HAZ-r5tc4m

Who receives a proof is unchanged: org-io's and org-node's distribution to a
former device of the same Member (RC-qa2758's note, owner ruling 2026-10-06).

## Residual risk

Identity: with RC-qa2758 and RC-5nka8z, the most a proof can disclose is one
Member's leaf without its DevicePublicKeys, and a proof for a Member who is
absent discloses nothing. S3/P2, **not acceptable** until the distribution
gate exists, as recorded for the earlier disclosure hazard.

Confirmation of candidate DevicePublicKeys: the leaves are unsalted
(`docs/adr/2026-10-23-unsalted-tree-leaves.md`), so a verifier who already
has one of the Member's DevicePublicKeys can test whether a proof's hashes
were made from it. The intended verifier, a former device of the Member, knew
the Member's DevicePublicKeys when it was revoked; any other verifier has
already been shown the Member's identity by the leaf, which is the S3 above.
What confirmation adds is that already-known devices are still the Member's.
Owner ruling 2026-10-23: S1/P1, **acceptable**.

## Derived requirements assessment

REQ-sb2frw and REQ-65xrgh replace the first absence-proof requirements with
the same refusals and the same binding to one membership root, so RC-j2znx8 works as
before against HAZ-adm7gv; the new device-root step rests on
sparse-merkle-tree's control of an unsound proof
(`sparse-merkle-tree/docs/risk/2026-10-23-sparse-merkle-tree.md`). REQ-sb2frw realises RC-5nka8z and narrows
what HAZ-r5tc4m discloses. REQ-2sz7j8 (a collision on
adding a DevicePublicKey refused, the record unchanged) has the availability
effect person analyses
(`person/docs/risk/2026-10-23-person-absence-proofs.md`), S1/P2 and acceptable, and changes no
control. The proofs add to the hostile-input surface of HAZ-8suua9.
assesses: REQ-sb2frw, REQ-65xrgh, REQ-2sz7j8

### Derived design items (2026-10-24)

LLR-4rdqyn replaces the earlier four hash domains with eight: the member
and device leaf and node domains, unchanged in key, and an identity and a
content domain for each. The new domains are what lets a proof tell a
different leaf from the one it is about by hashes alone, so they serve
RC-5nka8z; separation between them is the property the four domains already
had, extended to the new keys, and it is checked the same way (the same
input under every pair of domains hashes differently). A domain key shared by
two of them would let a hash made for one be read as the other, a second
route to the false revocation HAZ-adm7gv names; the separation check is the
control.
assesses: LLR-4rdqyn

**Amended 2026-10-24 (the sparse-merkle-tree change).** LLR-25nmwc now refuses a Vacant proof whose step shows the MemberId
held, and LLR-54gp2f a Held proof for another MemberId. Both narrow what
`verify` accepts, which strengthens RC-5nka8z; neither is derived, so
neither needs an `assesses:` line.
