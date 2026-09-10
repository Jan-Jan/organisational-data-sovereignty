# Derived requirements assessment — dependency expectations

First file in org-node's risk ledger. It is **not** org-node's hazard
analysis; that is tooth 3 of `docs/plans/2026-09-05-ratchet-gap-analysis.md`
and is written with `analyze-risks`. This file does the one thing
`check-trace.sh` requires of a derived requirement before it may exist:
assesses it in the risk management file (UNANALYZED-DERIVED). Read each entry
as "hazard impact against the hazards org-members' register already
enumerates", because org-node has enumerated none of its own yet.

(Corrected 2026-09-09: org-node's hazard analysis now exists —
`org-node/docs/risk/2026-09-09-org-node-hazards.md` — so org-node has six
hazards of its own, and the two entries below are read against org-members'
register only because that is the frame in which they were assessed.)

The two items assessed are the expectations in
`org-node/docs/requirements/2026-09-06-dependency-expectations.md`.
Both are derived: they exist because the system is partitioned into units and
org-node's verify-and-commit path leans on behaviour its providers perform
without stating. Neither has a parent in a system-needs document, because
there is none.

org-members' hazards and controls are referred to below by description and
file, never by ID: only requirements cross a unit boundary, and a consumer's
reference to a provider's HAZ, RC or PR is `NON-EXPORTED-REF` on every run.
The register in question is
`org-members/docs/risk/2026-09-02-membership-hazards.md`.

## Assessment

REQ-ysyu9g (on-chain-client returns the finalised Organisation state when no
block is named). Hazard impact: **mitigates**, and directly on the decisive
control of org-members' register. Its stale-or-divergent-record hazard and its
root-reversion hazard both rest on the base-matching control that REQ-4umsuz
realises, and the register states in as many words that the root the control
compares against must come from a path the attacker does not control. A
best-block read is such a path only until the next reorganisation; a
finalised read is not. Failure behaviour if the expectation is violated: a
revocation is committed against a root the chain later discards, and a
removed Member is believed removed on evidence that no longer exists — the
retained-access pathway, S3. No new hazard is introduced by asking for the
finalised read; its cost is latency, which that register's staleness
discussion already carries.

REQ-q92yac (org-members rejects a Change set that removes a Device key without
replacing the Member-as-a-group key). Hazard impact: **mitigates** the
register's device-removed-without-effective-rotation hazard, on the half of it
that the direct-API control behind REQ-ewdg2q does not cover. That register
evaluates the residual risk of that hazard as unacceptable on two counts, the
second being exactly this wire-path bypass. Failure behaviour if the
expectation is violated: a Change set authored elsewhere removes a device and
keeps the key, org-node applies it because every check it performs passes,
and the removed device continues to decrypt — the same S3 pathway as the open
problem report in `org-members/docs/problems/2026-08-31-device-removal-key-check.md`,
reached without calling either operation that report names. Hazard introduced
by the control: a legitimate Change set that removes a device without rotating
is refused, which is an unavailability pathway of the kind the register
already accepts for the direct-API control (rotation is unconditional by
decision).

## What this file does not do

It does not evaluate either expectation against the acceptability matrix,
assign severity or probability to org-node's own hazards, or decide residual
risk. org-node's register does that when it is written. Until then the two
items above are candidates for `(implements: RC-…)` in that register, and
their `expects:` clock — 90 days from 2026-09-06, per
`expectation_age_days` — is what keeps them from being forgotten in the
meantime.

(Corrected 2026-09-09: org-node's register is written —
`org-node/docs/risk/2026-09-09-org-node-hazards.md` — and it does evaluate
against the matrix. It also names the hazard each expectation was written
toward: REQ-ysyu9g stands against HAZ-tawvm2, a Change set accepted on its
sender's word, and REQ-q92yac against HAZ-vxabf9, a device removed from the
record that keeps acting as a member. Neither carries `(implements: RC-…)`,
and the annotation is still deliberately withheld: an unmet expectation
carrying it fails this unit's gate on every run, and the providers' ninety
days run to 2026-12-05. The clock in this paragraph is therefore still what
keeps the two items from being forgotten.)
