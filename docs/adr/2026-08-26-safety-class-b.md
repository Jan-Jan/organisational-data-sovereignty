# ADR: IEC 62304 software safety class B

- **Date:** 2026-08-26
- **Status:** **SUPERSEDED 2026-09-01 by
  `docs/adr/2026-09-01-safety-class-c.md`.** The injury pathway this ADR
  asserted and left unproven was examined at the start of the risk analysis.
  It substantiated further than this document allowed for: the intended
  deployments include journalism and government secrets, where wrongful
  disclosure can lead to serious injury. That is S3, and this ADR's own
  reasoning — that serious injury is not reachable — is what fails. Left in
  place unedited below, because it records the decision that was actually
  taken on 2026-08-26 and the open item that caught it.
- **Decision maker:** Jan-Jan (project owner), interviewed by `/ratchet`
- **Recorded in:** `.guardrails/config.yaml` → `safety_class: B` (changed to
  `C` on 2026-09-01)

## Context

IEC 62304 §4.3 requires every software system to be assigned a safety class
before the lifecycle activities are scaled to it. The class governs how much
verification rigour guardrails demands: coverage target, robustness testing,
whether low-level requirements are mandatory, and how independent the review
must be.

The system under classification is the two-tier access-control project
("Organisational Data Sovereignty"): an on-chain anchored membership trie
(`org-members`), a chain client (`on-chain-client`), a local-first node
(`org-node`) and a Tauri application, which together decide who is a member of
an organisation and therefore who can decrypt and read that organisation's
documents.

## The class boundary, stated correctly

The three classes are separated by **harm**, not by whether a hazardous
situation can arise:

- **Class A** — no injury or damage to health is possible. A failure that
  produces a hazardous situation is still class A if no injury can result from
  it.
- **Class B** — injury is possible, but not serious injury.
- **Class C** — death or serious injury is possible.

So the question that decides A versus B is not "can this software contribute to
a hazardous situation" — it is "can a person be injured as a result". This ADR
originally argued the boundary the other way round, and the correction matters,
because the confidentiality and availability harms this system's failures
produce are not in themselves injuries.

## Decision

**Class B.**

The interview ran the standard two steps:

1. *Can a failure of this software contribute to a hazardous situation at
   all?* — **Yes.** The software's whole purpose is to grant and revoke access
   to organisational data. A wrong decision (a revoked member who retains
   access, a legitimate member locked out, a fork that leaves two divergent
   views of membership) puts the organisation in a situation it did not choose.
2. *Could the resulting harm, after risk controls external to the software, be
   serious injury or death?* — **No, but non-serious injury is possible.** The
   pathway asserted by the project owner: this membership layer is intended to
   gate access to organisational data that can feed a care or safety decision —
   an organisation using it to control who may read records that a person then
   acts on. A legitimate member wrongly locked out cannot reach data at the
   moment a decision needs it; a revoked member retaining access can act on
   data they should no longer hold. Injury reachable that way is non-serious:
   the software's output reaches a person only through a human decision, and it
   controls neither dose, nor energy delivery, nor any direct physical actuator.

Class B follows: injury is possible, serious injury or death is not the
foreseeable outcome.

**This pathway is asserted, not yet analysed.** It is the owner's answer to the
interview, and it is written here so the classification is not a bare
assertion — but no HAZ item, severity or probability exists for it yet. Against
the severity scale this change ships in `docs/risk/README.md`
(`S1 negligible · S2 non-serious injury · S3 serious injury or death`), the
claim is that at least one hazardous situation reaches S2. Tooth 5
(`analyze-risks`) is where that either survives contact with a real hazard
analysis or is revised — and revising it downward to class A is a legitimate
outcome of that analysis, provided the analysis, not convenience, is what
drives it.

## Consequences

From the guardrails verification-rigour table for class B:

- **Statement coverage** is the target. It is not yet measured —
  `cargo-llvm-cov` is not installed and `coverage_command` is consequently
  unset. Tooth 3 of `docs/plans/2026-08-26-ratchet-gap-analysis.md` closes
  this; until it does, the class target is stated and unmet, which is why it is
  written down here rather than left implicit.
- **Robustness tests are required**, normal and abnormal per requirement or
  low-level requirement. The existing proptest suite, the three libfuzzer
  targets in `on-chain-client`, and the quint MBT conformance harness are the
  foundation to build these on, not a substitute for them: none is currently
  annotated with what it verifies.
- **Low-level requirements are optional per item**, required only where an
  item is complex enough to need them. Class C's blanket LLR-per-SDD-item rule
  does not apply.
- **One independent reviewer** per change, exercised at `merge-change` step 6a
  and recorded in the change's verification record.
- Segregation: if any item is later argued to be class A (a spike crate, say,
  that no product path depends on), that per-item classification is stated in
  the architecture ledger next to the SDD item, per IEC 62304's allowance for
  per-item classes. The project default stays B.

## Alternatives considered

- **Class A** — rejected, but on narrower grounds than the class boundary
  above might suggest. A is the right class if no injury is possible at all,
  and that is a defensible reading of an access-control layer whose harms are
  confidentiality and availability. It is rejected here because the owner
  identified a use in which the data gated by this layer feeds a decision a
  person acts on, which makes non-serious injury reachable. If tooth 5's hazard
  analysis cannot substantiate that pathway, A becomes the correct class and
  this ADR should be superseded rather than defended.
- **Class C** — rejected as unjustified today. It would be the right class if a
  wrong access decision could contribute to serious injury or death; nothing in
  scope shows that. The `/ratchet` skill's own rule — that when in doubt
  between two classes the higher governs until justified otherwise — is what
  would have forced C in the presence of real doubt. There is no such doubt at
  the B/C boundary; the doubt in this decision sits at the A/B boundary and is
  resolved by the owner's answer to interview step 2, recorded above.

## Open item

The hazardous situations behind this classification are asserted here in prose
and have not yet been enumerated as HAZ items with severity and probability.
That is tooth 5 (`analyze-risks`), and it is also where the risk acceptability
matrix in `docs/risk/README.md` — shipped as `TBD` — must be filled from the
quality manual. Until then this ADR is the only written statement of why the
class is what it is, and its weakest link is the injury pathway in step 2.
