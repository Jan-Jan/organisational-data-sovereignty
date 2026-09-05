# Derived-requirement assessments — organisation membership

Every requirement in the membership set is `satisfies: derived`: the rules came
from design and threat reasoning, not from a written system-needs document.
ISO 14971 and IEC 62304 both require a derived requirement to be assessed for
hazard impact rather than merely noted, and `check-trace.sh` enforces the
mention (`UNANALYZED-DERIVED`).

**What this file is not.** These are impact assessments of derived
requirements, not a hazard analysis. No hazard is enumerated here, and no
severity or probability is assigned in the assessments below.

**Amended 2026-09-02, and the superseded text is quoted rather than deleted.**
As written on 2026-08-31 this note said:

> No hazard is enumerated here, no severity or probability is assigned, and the
> acceptability matrix in this ledger's README is still `TBD` — filling it is a
> quality-manual decision and is tooth 5 (`analyze-risks`). Read every "Hazard
> impact: mitigates" below as shorthand for "mitigates a hazard that has not
> been enumerated yet": each names a mechanism and a pathway, and none of them
> is anchored to a HAZ item, because none exists. The safety-class ADR
> (`docs/adr/2026-08-26-safety-class-b.md`) rests on an injury pathway that
> tooth 5 must substantiate, and these are the requirements that pathway runs
> through.

Every part of that has happened, in
`docs/risk/2026-09-02-membership-hazards.md`: the matrix is set, the hazards are
enumerated, and every requirement assessed below now carries a risk control it
implements. So the shorthand is no longer needed — each "Hazard impact:
mitigates" below names a mechanism and a pathway that a HAZ item now exists
for. Three of the assessments called their own shots and were right:
REQ-m8aexh's is now HAZ-jkc6tj, REQ-4umsuz's is now HAZ-y8h835 and HAZ-h58jn6,
and the warning under REQ-ewdg2q that its control's effectiveness was unverified
is now the residual-risk statement for HAZ-s39gbh.

The class citation above is also superseded: the injury pathway the class B ADR
asserted was substantiated further than that ADR allowed, and the project is
Class C as of 2026-09-01 (`docs/adr/2026-09-01-safety-class-c.md`). The
assessments themselves are unchanged and stand as written — the class governs
verification rigour, not whether a derived requirement affects a hazard.

The two harms in view throughout: a member wrongly denied access cannot reach
organisational data when a decision needs it, and a member wrongly granted or
retaining access acts on data they should not hold. The hazard analysis keeps
both and states their severity — disclosure S3, unavailability S3 at its worst
credible outcome.

## Identity and naming

REQ-crjxk8 (stable 32-byte member identifier, distinct from handle and keys).
Hazard impact: mitigates. An organisation that identified members by a mutable
handle would silently re-target every grant made to a member when that member
was renamed, and would confuse two members across a rename-and-reuse. Both are
wrong-grant paths. This requirement is what makes a grant survive a rename
without following the name to whoever holds it next.

REQ-kmvc96 (handle uniqueness within the organisation). Hazard impact:
mitigates. Two members holding one handle makes any human instruction naming
that handle ambiguous — "grant Alice access" resolves to two people, and an
administrator cannot tell which. Wrong-grant path, mediated by a person.

REQ-h5ret5 (handle validity: non-empty, ≤128 bytes, NFC, lowercase, no `.`,
single script). Hazard impact: mitigates, and partly enabling. NFC and
lowercase make handle comparison total, so uniqueness (REQ-kmvc96) is decidable
rather than approximate; single-script and the `.` exclusion narrow the space
of handles that can be made to read as another. The length bound is resource
protection, with no direct hazard path.

REQ-m8aexh (confusable-skeleton rejection). Hazard impact: mitigates, and this
is the sharpest of the naming set. Its threat is an attacker enrolling a handle
that renders identically to an existing member's, so an administrator grants to
the wrong principal while reading the right name. The harm is a wrong grant
that no amount of care by the administrator would catch, because the two
strings are visually identical. This is a candidate hazard for tooth 5 in its
own right.

## Keys and devices

REQ-xdx2c2 (bounded device keys per member, no duplicates). Hazard impact:
mitigates, weakly. The bound limits how many endpoints a single member's
compromise exposes and bounds the work of isolating that member; it does not
prevent any harm by itself. Duplicate rejection is an integrity property of the
device set rather than a safety one.

REQ-ewdg2q (member-as-a-group key replaced when a device key is removed).
Hazard impact: mitigates, and directly on the retained-access path. A removed
device holds the member-as-a-group key it was enrolled under; without
replacement, removal changes who is listed but not who can decrypt, which is the
exact failure of a revocation that appears to succeed. Assessed on the decision
recorded during the requirements interview that rotation is unconditional: the
software cannot verify a caller's claim that a retirement was benign, and a
stolen device reported as cleanly retired would otherwise keep access.

**The mitigation is incomplete as implemented.** The replacement key is not
compared against the key it replaces, so a caller passing the current key back
performs a removal that mitigates nothing (PR-zz4exm). Until that is fixed, this
requirement's contribution to the retained-access path is conditional on the
caller, which is precisely the property a risk control must not have. Tooth 5
should treat this as a control whose effectiveness is currently unverified
rather than as one in place.

REQ-r784fu (isolate a member in one step, retaining membership, reversible by
adding a device). Hazard impact: mitigates, on both harms at once. It is the
emergency response to a member whose devices are compromised, so its absence
leaves only piecewise removal — during which the member retains access. Keeping
the member in the record and making the state reversible is what stops the
emergency response from becoming a wrongful expulsion that must be undone by
re-admission. A candidate risk control for tooth 5.

## Integrity of the record

REQ-avmu3j (no membership root reported for an un-recomputed record). Hazard
impact: mitigates. A stale root published as current would attest to a
membership that no longer holds — including one still containing a member just
removed. Erroring instead is what prevents a revocation from being contradicted
by the very value used to prove membership.

REQ-d3prca (modification leaves the existing record unchanged). Hazard impact:
mitigates. A published membership record that could be altered in place would
invalidate whatever was decided against it and would break the ability to
compare two membership states — which is what a distributed organisation needs
in order to converge. No direct injury path; it protects the evidence rather
than the decision.

REQ-4umsuz (reject a change set whose base does not match; reject a result
whose root does not match the expected root). Hazard impact: mitigates. Both
halves are integrity checks on membership changes arriving from another
administrator. Applying a change set to the wrong base silently produces a
membership neither party intended, which is a wrong-grant and wrong-deny path
simultaneously. This is the requirement closest to the fork-safety and
convergence invariants already mechanised in the quint models, and tooth 5
should connect them.

REQ-shk82j (re-validate handles and device sets received from outside the
process). Hazard impact: mitigates. Without it, every rule above — validity,
uniqueness, confusables, the device bound — is enforced only on the path
through the constructor, and a serialized record becomes a way to introduce a
member the software would have refused. It is what makes the other
requirements properties of the membership record rather than of one code path.

## Robustness

REQ-ds8ryr (report errors, never panic, for any input). Hazard impact:
mitigates availability harm. A panic in a library embedded in the node or the
application takes down the caller, and the harm in view is a member unable to
reach data when a decision needs it. It is also the requirement that makes the
others meaningful under hostile input: a validation rule that aborts the
process on a malformed handle has not rejected the handle, it has denied the
service.
