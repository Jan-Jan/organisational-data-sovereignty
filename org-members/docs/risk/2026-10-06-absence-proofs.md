# Absence proofs — hazards and controls

Risk analysis for S1 of `docs/plans/2026-10-06-org-io-roadmap.md`: absence
proofs (REQ-535jcd, REQ-tk2qqj) and PR-jq43gx. Matrix and method are the ones
in `README.md` and `2026-09-02-membership-hazards.md` ("Method, and what the
harms are"). Losing access to organisational records at the moment a decision
needs them is that register's unavailability pathway, which it assesses as S3;
the same reasoning applies here.

## A current member's device deletes its data on a false proof

**HAZ-adm7gv**: an absence proof that does not reflect the current membership
— forged, malformed, or checked against a root other than the current
on-chain one — is accepted by a device whose Member still holds its Device key;
the device takes the proof as its revocation and deletes everything it holds
for the Organisation; the member cannot reach organisational records when a
decision needs them. Severity: S3. Probability: P2.

P2: proofs arrive from peers, and the verifier is reachable by anything that
can hand the node bytes.

**RC-j2znx8**: an absence proof is accepted only when it resolves to the
membership root it is checked against and shows absence for the exact MemberId
and Device key it is checked for; every other proof is rejected with a typed
error. mitigates: HAZ-adm7gv

Which root the proof is checked against is not decided here. Owner ruling,
2026-10-06: org-io reads the latest on-chain state and has the proof checked
against that root, and org-io must offer no way to be invoked against any
other root. That control is org-io's, analysed and tested there (S3 and S5 of
the roadmap); org-members cannot know which root is current. org-members'
part is that a proof binds to exactly one root, so a proof that was genuine
under an older root fails against the current one.

Residual risk, org-members' part: with RC-j2znx8, accepting a proof that
contradicts the root means finding a second preimage of a blake3 node hash.
S3/P1, **not acceptable** under the matrix, as for every S3 in this register
(`2026-09-02-membership-hazards.md`, "Residual risk"). The stale-root pathway
stays open until org-io's control exists.

## Member data disclosed through a proof

**HAZ-gwbn5n**: an absence proof for a Device key removed from a Member who
remains carries that Member's leaf (MemberId, handle, name, surname, member
key, Device keys); a proof that reaches anyone other than a former device of
that Member discloses that Member's identity and keys; exposure of a protected
identity in the deployments this project is classified for. Severity: S3.
Probability: P2.

P2: a proof is produced on request, and without a control the content would
be whatever the producer chose to include.

**RC-qa2758**: an absence proof carries no Member's data when its MemberId
holds no Member, and only the data of the Member under its MemberId
otherwise; it never carries another Member's data. mitigates: HAZ-gwbn5n

To whom a proof is given is not decided here. Owner ruling, 2026-10-06:
exclusion is by design, part of the path that produces the proof. A node
produces one only for a requester whose authenticated Device key and supplied
MemberId match a (Device key, MemberId) pair on the revoked-device list, and
only over the connection that authenticated that key. The list and that path
are org-io's and org-node's (S5 of the roadmap). org-members cannot see the
list.

Residual risk, org-members' part: with RC-qa2758, the most a proof can
disclose is one Member's leaf, and a proof for a Member who is absent
discloses nothing. S3/P2, **not acceptable**, until the S5 gate exists.

## Hostile input — HAZ-8suua9, extended to proofs

Proofs are a new hostile-input surface. They belong to the existing
HAZ-8suua9 and its control RC-c4truv: every rejection is a typed error, and no
input causes a panic. No new hazard is minted. The proof type is parsed under a
fixed bound before use, so an oversized or truncated proof is a typed error.
The sibling list is bounded at 256 while decoding; a Leaf ending's handle,
name and surname are decoded before validation and bounded only by the
input's length, as for any received MemberLeaf. PR-jq43gx is a defect of
RC-c4truv, in the two primitives the verifier walks (`MemberId::bit`,
`DefaultHashes::at_level`), and is resolved in this change. HAZ-8suua9's
probability is unchanged: the surface grows, but so does the control's
coverage, and the one known breach of the control on this path is closed.

## Derived requirements assessment

REQ-535jcd and REQ-tk2qqj exist because the owner chose to tell a revoked
device about its revocation with a proof, not with the membership record.
REQ-tk2qqj is RC-j2znx8's realisation. REQ-535jcd produces what HAZ-gwbn5n
discloses, and REQ-yyuxh8, RC-qa2758's realisation, bounds that. None of them
changes how any existing control works; production and verification add a
surface to HAZ-8suua9, assessed above. Owner confirmed S3/P2 for both new
hazards on 2026-10-06.
assesses: REQ-535jcd, REQ-tk2qqj, REQ-yyuxh8
