# ADR: IEC 62304 software safety class C

- **Date:** 2026-09-01
- **Status:** accepted
- **Supersedes:** `docs/adr/2026-08-26-safety-class-b.md`
- **Decision maker:** Jan-Jan (project owner), interviewed at the start of the
  ISO 14971 risk analysis (`analyze-risks`)
- **Recorded in:** `.guardrails/config.yaml` → `safety_class: C`

## Why this reopened

The Class B ADR of 2026-08-26 rested on an injury pathway it described as
"asserted, not yet analysed", and it named the risk analysis as the place that
would substantiate or retract it. It also said, in as many words, that revising
the class was a legitimate outcome of that analysis.

The analysis opened with exactly that question, before any hazard was
enumerated — because severity governs acceptability, and the class governs what
severity is admissible. The answer was that the intended deployment is not one
context but three:

- **Journalism and human-rights work.** The organisation's documents identify
  sources and protected individuals. Access to them is the thing being
  controlled.
- **Clinical and care organisations.** Files that staff read to make care
  decisions.
- **Government and inter-organisational secrets.** Material whose disclosure
  has consequences beyond the organisation holding it.

## Decision

**Class C.**

IEC 62304 §4.3's classification decision turns on whether the harm reachable
after risk controls external to the software could be serious injury or death. For the first and third contexts it
plainly could. A member whose access should have ended and has not — the exact
failure this system exists to prevent — can learn the identity of a source or a
protected person. The harm at the end of that chain is not inconvenience or
commercial loss; it is what happens to a person whose identity reaches someone
who should not have it. That is **S3**, and the acceptability matrix set in
`docs/risk/README.md` on the same day makes S3 unacceptable at every
probability, including improbable.

A class must cover the worst credible use, not the median one, and the first
and third contexts settle it on their own.

**Corrected 2026-09-02.** An earlier revision of this ADR added that "the
clinical context on its own would have sustained Class B" and called it the
least severe of the three. The hazard analysis written under this ADR
contradicts that: unavailability of a record at a care decision point has S3 as
its worst credible harm (HAZ-8suua9), because severity is a property of the
harm and an organisational fallback does not make an injury less severe. The
ranking is withdrawn. The Class C conclusion never depended on it — journalism
and government carry it — but the sentence as written would have licensed a
Class B reading of a clinical-only deployment, which this project's own risk
ledger rejects.

## What was wrong with the earlier reasoning

Worth recording precisely, because the error was structural rather than
careless. The Class B ADR reasoned from the *nature of the data* — access
control, confidentiality, availability — and concluded that harm reaches a
person only through a human decision and is therefore non-serious. That
inference does not hold. The intervening human decision does not bound the
severity of the harm; it only bounds the software's directness. Disclosure of an
identity to a hostile party is mediated entirely by human action and can still
end in serious injury.

The general form of the mistake: reasoning about severity from the *mechanism*
of the software rather than from the *consequences* in its deployment context.
An access-control system's severity is set by what it protects, and that was not
established until the deployment question was put directly.

## Consequences

From the guardrails verification-rigour table, Class C differs from B in four
ways, and each has real cost:

- **Coverage: statement AND decision.** Statement coverage is measured today
  (`make coverage`, line and region floors per crate). Decision coverage is
  **not**, and cannot be on the current toolchain: `cargo llvm-cov --branch` is
  unstable and fails on stable rustc. It needs a nightly toolchain with
  `llvm-tools-preview`. Until that is in place the class target is unmet, and it
  is unmet in a way the previous class did not have: this is now a gap against a
  mandatory requirement rather than a stretch goal.
- **Low-level requirements are REQUIRED per software item**, not optional per
  item: interfaces, algorithms, error behaviour and resource limits each get a
  testable LLR. This lands squarely on tooth 6 (`design-architecture`), which
  was scoped as SDD items plus optional LLRs and is now materially larger. The
  device-key bound deferred out of REQ-xdx2c2 is one of these.
- **Independent review must be thorough**, with two independent reviewers worth
  considering for critical items. Every change so far has used one fresh
  subagent reviewer, which was the Class B standard.
- **Robustness testing remains required**, as under B. **Corrected
  2026-09-02:** an earlier revision of this ADR stated that "there is no
  arbitrary-bytes fuzz target for the deserialize path". That is false —
  `org-node/tests/fuzz_envelope_decode` feeds arbitrary bytes to an envelope
  decode and then to `decode_delta`, which runs the `org-members`
  deserialisation validation. The claim was inherited from tooth 4's
  verification record and repeated here without being checked. The real gap is
  that the target is outside `test_paths`, carries no `verifies:` annotation,
  has an empty seed corpus and is run by no lane — evidence that exists and
  counts for nothing. See `docs/risk/2026-09-02-membership-hazards.md`.

Segregation is now more valuable than it was: IEC 62304 permits per-item
classification, so if any software item can be argued to be Class A or B — a
spike crate no product path depends on, say — that argument belongs in the
architecture ledger next to the SDD item, and it buys back real rigour. The
project default is C.

## Alternatives considered

- **Keep Class B and record the S3 hazards anyway.** Rejected, and it is what
  the `analyze-risks` skill explicitly forbids: an S3 finding in a Class B
  project means the classification is wrong, not that the register needs a
  footnote. Recording S3 risks under a class that assumes they cannot exist
  would make the class a decoration.
- **Split the project by deployment** — Class C for the journalism and
  government configurations, Class B for clinical. Rejected as a fiction at
  this stage: it is one codebase with one membership layer, and nothing in it
  varies by deployment. If a genuine segregation appears later — a build or a
  configuration that provably cannot serve the S3 contexts — that is a
  per-item classification argument to make then, with evidence.
- **Class B until a deployment actually exists.** Rejected. The classification
  governs how the software is built, and the software is being built now. A
  class that trails the deployment by months means the rigour arrives after the
  code it was supposed to shape.

## Open items this creates

1. **Decision coverage is unmeasured** and now mandatory. Needs
   `rustup component add llvm-tools-preview --toolchain nightly`, then a
   `--branch` coverage lane in the Makefile and CI. Until then a documented gap
   awaiting the owner's explicit acceptance at a merge — `verify-before-merge`
   step 5 requires that acceptance from the user and records it in the
   verification record, and it has not been given. Unlike the Class B coverage
   gap, this one is against a requirement rather than a target.
2. **Tooth 6 must produce LLRs per software item**, not optionally. The adoption
   order needs re-scoping before it runs.
3. **The review policy needs a decision** on what "thorough" means here, and
   whether critical items get two reviewers.
4. ~~**The hazard register itself is still empty.**~~ **Closed 2026-09-02.**
   This ADR settles the frame the register is written in, and the register was
   written under it in this same change rather than the next one:
   `docs/risk/2026-09-02-membership-hazards.md`. It reaches a conclusion this
   ADR did not anticipate — that under the matrix set here, six of seven
   hazards carry unacceptable residual risk, so the intended use must exclude
   the very three deployment contexts this ADR used to justify Class C, until
   the controls it names are in place.
