# Absence proof requirements

Stage S1 of the org-io roadmap (owner rulings 2026-10-06, recorded in the
`grill-requirements` interview on branch
`worktree-org-members-absence-proofs`). A device whose Member or Device key
has been revoked must be able to confirm the revocation against the on-chain
membership root without receiving the membership record. Any device holding
the current record produces the proof; the revoked device verifies it.

The owner chose proofs over sending the revoked device the whole record, and
chose to keep the root's definition unchanged. Because a Member's leaf hash
covers its whole record including its device root, a proof that a Device key
is absent from a Member who remains reveals that Member's leaf. It is given
only to a former device of that same Member, which held that data before its
revocation. A proof shows absence at one root only; it does not show that a
key was never held, which keeps the no-key-history ruling of REQ-ewdg2q.

## Production

A Device key held by a different Member does not stop the proof (owner ruling
2026-10-06). The case it raises — a key revoked from one Member and later held
by another, whose device would then verify a true proof and delete its data —
is prevented where the revoked-device list is kept: a Device key must be
unique across the membership record and that list together, so a revoked key
is never admitted again. That rule belongs to the unit that stores the list,
not to org-members.

**REQ-yyuxh8**: The software shall include in an absence proof no Member's
data when the proof's MemberId holds no Member, and no data of any Member
other than the one under the proof's MemberId otherwise.
implements: RC-qa2758
satisfies: derived
exported: yes
