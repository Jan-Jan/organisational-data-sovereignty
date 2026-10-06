# Problem reports — Envelope signing key conflated with the Organisation public key

**PR-szkat6**: The Organisation private key is held only by the node that
created the Organisation (LLR-3fwykc) and no admission gives it to a Member,
whereas the design intends `org_pub_key` to be the public half of an
Organisation key pair whose secret every Member holds
(`Organisational Data Sovereignty p1.md`, design point 5).
affects: REQ-ech45n, LLR-3fwykc
opened: 2026-10-04
status: resolved
resolution: every Organisation-information message carries the Organisation private key the sending node's record holds (REQ-szq3ud), and its receiver stores it once its public half is the chain's key (REQ-ju6vn2, REQ-bwx7eg); `OrgPublicKey` is parsed by person's X25519 rule (LLR-3jjgtw); reproduced by `pr_szkat6_an_admitted_member_holds_the_organisation_private_key` (org-node/tests/admission_sender.rs), red before (a member's record held none), green after.

*Restated 2026-10-05 by review round 4 (finding-6).* The report first read:
"The node verifies every Envelope under the key stored in the Organisation
state's `org_pub_key` field (REQ-ag6kqm) and sets that field to the
administrator's Member-as-a-group key at genesis (`service.rs`
`create_organisation`), whereas the design intends `org_pub_key` to be the
public half of the Organisation key pair whose secret every Member holds, so
once that key pair is implemented as designed any Member could sign a Change
set the node accepts", with `affects: REQ-ag6kqm, RC-pm9kmx`. Neither half
holds on change `worktree-person-shared-types`: the Envelope carries no
signature and nothing is verified under `org_pub_key` (REQ-ag6kqm and
RC-pm9kmx as amended), and `org_pub_key` is the public half of a fresh X25519
key pair (REQ-ech45n). What stays open is the part the reopening note below
names, so the statement is that part and `affects:` names the items whose
behaviour resolving it changes.

Found 2026-10-04 in the `grill-requirements` interview for the org-node
type-safety change, when the owner stated that `org_pub_key` is not an
administrator key but the public half of a key pair shared with all Members,
agreed by the administrators so that CGKA need not run at Organisation level
(`Organisational Data Sovereignty p1.md`, design point 5). The glossary entry
*Published signing key* recorded the PoC's choice ("today it is the
administrator's Member-as-a-group key"), and the *Organisation secret* the node
hands out at admission is opaque user input with no relation to `org_pub_key`.
Nothing is exploitable today: the secret half of `org_pub_key` is the
administrator's member seed, which no other Member holds. Which key Envelopes
must verify under — and how that relates to the on-chain admin multisig — is a
protocol decision for its own change. The type-safety change types the field as
`OrgPublicKey`, after its intended role, and changes no behaviour.

(Amended 2026-10-05 after that change's independent review round 5:
`OrgPublicKey` checks the field as an ed25519 Edwards point, which is right for
the key it holds today and wrong for the intended one. The owner ruled
(2026-10-04, recorded in `person/docs/requirements/2026-10-04-identity-types.md`)
that the Organisation public key is an X25519 key belonging to org-node and
checked by `person`'s X25519 rule. Resolving this report must therefore also
move `OrgPublicKey` to that rule, or the published X25519 key would be refused
for about half of its values.)

**Reopened 2026-10-05** (docs/plans/2026-10-05-switch-trim.md). Change
`worktree-person-shared-types` makes `org_pub_key` the public half of a fresh
X25519 Organisation key pair, distinct from every genesis key (REQ-ech45n,
LLR-sj7cd5, LLR-3fwykc, LLR-322xfu), parses it by `person`'s X25519 rule
(LLR-3jjgtw, LLR-mmdu38), and verifies nothing under it (REQ-ag6kqm as
amended). What the design intends is not done yet: the Organisation private
key is not given to the Members. Chain-authority's change 2 (admission sends
the Organisation private key, which a receiver checks against the chain's
`org_pub_key`) resolves this report.
