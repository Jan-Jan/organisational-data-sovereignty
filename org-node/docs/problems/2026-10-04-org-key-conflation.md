# Problem reports — Envelope signing key conflated with the Organisation public key

**PR-szkat6**: The node verifies every Envelope under the key stored in the
Organisation state's `org_pub_key` field (REQ-ag6kqm) and sets that field to
the administrator's Member-as-a-group key at genesis (`service.rs`
`create_organisation`), whereas the design intends `org_pub_key` to be the
public half of the Organisation key pair whose secret every Member holds, so
once that key pair is implemented as designed any Member could sign a Change
set the node accepts.
affects: REQ-ag6kqm, RC-pm9kmx
opened: 2026-10-04
status: open

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
