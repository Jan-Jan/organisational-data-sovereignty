# Risk management ledger

This directory is a per-change ledger: each merged change contributes one
dated file, `YYYY-MM-DD-<slug>.md` (the finalize date, assigned by `merge-change`
from your worktree's `DRAFT-<branch>-<slug>.md`). **Edit existing items in
the file that defines them.** This README contains the project-wide
acceptability matrix; the dated files contain hazards, controls, derived
assessments, and residual-risk statements.

<!--
Item grammar (enforced by .guardrails/scripts/check-trace.sh):

  **HAZ-NNNNNN**: <hazard — potential source of harm>, <hazardous
  situation>, <harm>. Severity: <S1..S3>. Probability: <P1..P3>.

  **RC-NNNNNN**: <risk control measure>. mitigates: HAZ-NNNNNN

- Every hazard must have at least one risk control that `mitigates:` it.
- Every risk control must be implemented by at least one requirement containing
  `(implements: RC-...)` in the requirements ledger.
- Derived REQ/LLR assessments live here too. Write the assessment under a
  heading and declare which items it covers on a line of its own:
  `assesses: REQ-…, LLR-…`. "No hazard impact because <reason>" is a valid
  assessment; an ID in a table or a passing sentence is not, and check-trace
  reports the item as UNANALYZED-DERIVED.
- Mint the ID when you write the item: run
  `.guardrails/scripts/new-id.sh <PREFIX>` and paste what it prints. Never
  invent one by hand.
- Record residual risk and its acceptability after controls are in place.
- IDs in the examples above use `NNNNNN` as a placeholder, and the examples
  are indented. Both matter. A real ID here would be a reference to an item
  that does not exist, reported as `DANGLING-REF` on every run; and a
  definition form at COLUMN ONE is judged whatever its body, so an example
  written flush left is reported as `MALFORMED-ID` — inside a fenced code
  block too, because no gate in the toolkit parses fences. Indent illustrative
  forms, or keep them inline in backticks. A real ID is six characters of
  `23456789abcdefghjkmnpqrstuvwxyz` with at least one digit; `new-id.sh` draws
  it for you.
-->

## Risk acceptability matrix

Severity: S1 negligible · S2 non-serious injury · S3 serious injury or death
Probability: P1 improbable · P2 occasional · P3 frequent

| | P1 | P2 | P3 |
|---|---|---|---|
| **S3** | UNACCEPTABLE | UNACCEPTABLE | UNACCEPTABLE |
| **S2** | ACCEPTABLE | UNACCEPTABLE | UNACCEPTABLE |
| **S1** | ACCEPTABLE | ACCEPTABLE | ACCEPTABLE |

Set 2026-09-01 by the project owner. No route to serious injury is tolerated at
any probability; non-serious injury is tolerated only where improbable;
negligible harm is tolerated.

Two consequences worth stating, because they are what the matrix is for:

- **S3 is unacceptable at every probability, including improbable**, and this
  project has S3 in scope: the deployments include journalism and government
  secrets, where a member retaining access they should have lost can expose an
  identity (`docs/adr/2026-09-01-safety-class-c.md`). So every hazard on that
  pathway requires controls regardless of how unlikely it is judged to be —
  "improbable" is not an argument here, and P1 buys nothing at S3.
- An S2 hazard assessed P2 or worse requires a control, in ISO 14971 priority
  order — inherent safety by design first, protective measures second,
  information for safety last. "Documented in the manual" is the weakest
  category and does not discharge an unacceptable risk on its own.

This matrix was set on the same day the project reclassified from B to C. It
was chosen before the class changed and needs no revision because of it: it
already treated every route to serious injury as intolerable. What changed is
that such routes turned out to exist.
