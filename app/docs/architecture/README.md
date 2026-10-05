# Architecture ledger

This directory is a per-change ledger: each merged change contributes one
dated file, `YYYY-MM-DD-<slug>.md` (the finalize date, assigned by `merge-change`
from your worktree's `DRAFT-<branch>-<slug>.md`). **Edit existing items in
the file that defines them.** Use this README's Overview section for the
system-wide decomposition picture; `soup.md` (single file) contains the SOUP
inventory.

<!--
Item grammar (enforced by .guardrails/scripts/check-trace.sh):

  **SDD-NNNNNN**: <software item and its responsibility>. traces: REQ-NNNNNN[, REQ-...]

Low-level requirements (design data — the directly codeable refinement of
the high-level REQs), written under the software item they belong to:

  **LLR-NNNNNN**: <directly codeable behavior>. satisfies: REQ-NNNNNN[, REQ-...]
  **LLR-NNNNNN**: <behavior with no parent requirement>. satisfies: derived

- Every design item must trace to at least one requirement.
- Every LLR satisfies a REQ or is marked derived; derived LLRs must be
  assessed in the risk ledger, where an `assesses:` line names them.
- Tests verify LLRs where they exist; the parent REQ is covered
  transitively.
- Class C items require LLRs (interfaces, algorithms, error behavior,
  resource limits — one testable LLR each); class B optional per item.
- If an item's safety class differs from the project default, state it in the
  item text (IEC 62304 allows per-item classification).
- Mint the ID when you write the item: run
  `.guardrails/scripts/new-id.sh <PREFIX>` and paste what it prints. Never
  invent one by hand.
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

## Overview

<!-- System decomposition, key interfaces, and the segregation rationale
     between items of different safety classes (if any). -->

*First written 2026-10-05 by the architecture tooth
(`docs/plans/2026-10-05-app-architecture.md`). This section is the standing
picture; each change amends it rather than restating it in a dated file. The
items, their low-level requirements and their tests are in
`app/docs/architecture/2026-10-05-decomposition.md`.*

### What this unit is

app is the operator's desktop application: a Tauri process whose Rust half
drives `org-node` and whose Svelte half, running in the webview, is what the
operator sees and types into. It decides nothing about membership itself. It
starts the node from its configuration, turns the operator's input into
service calls, and turns what the node's receiver reports into what the
operator is shown. It is IEC 62304 **class C** throughout, with no per-item
override. Both units it depends on, `org-node` and `on-chain-client`, are
class C, so no segregation argument is needed.

### The decomposition

An item is a responsibility with an interface, not a file. The items group by
the half of the process they live in.

| Half | Items | What it decides |
|---|---|---|
| **Rust: startup** | SDD-k95rmp, SDD-aq7m6b | Which passphrase, data directory and transport mode the node runs with, when startup is refused, and how the managed state is assembled from those values |
| **Rust: the command side** | SDD-2pa6h6, SDD-rmbr3t, SDD-q4dpym | What an operator-supplied value must be before it reaches the service, what each command returns, and what is reported about the running configuration |
| **Rust: the receiver side** | SDD-5fchuy, SDD-mwqf6x | Whether a receiver loop may start, and how each receive result becomes a named event with a payload |
| **Webview** | SDD-8jw9mn, SDD-jx363y | How events become the verification view, and whether a revocation may be submitted |
| **Configuration** | SDD-32hath | What any script in the webview may load, run or connect to |
| **Untested shell** | SDD-6g3wnh | Everything above, wired together at its call sites, that no gated test reaches |

The policy decisions are pure: SDD-k95rmp and SDD-q4dpym take every input as
a parameter and read no environment, and SDD-mwqf6x is a total mapping from
outcome to events. That is what lets the gate reach them without a live chain
or a running application.

### The IPC boundary

The two halves meet only over Tauri IPC, in two directions.

- **Webview to Rust: commands.** The frontend invokes the commands of
  SDD-rmbr3t's surface by their snake_case names (those whose success needs a
  chain are SDD-6g3wnh's), through the wrappers in
  `app/src/lib/api.ts` (SDD-6g3wnh). This is the trust edge: SDD-2pa6h6 parses
  the identifiers, addresses and secrets those commands take again on the
  Rust side, whatever the webview checked first, so SDD-jx363y's checks are
  the operator's convenience and not the unit's protection. What crosses back is a DTO that carries no secret key
  material.
- **Rust to webview: events.** The receiver loop (SDD-6g3wnh) classifies each
  receive result and emits it under SDD-mwqf6x's vocabulary, the only place
  the event names and payload shapes are written down. SDD-8jw9mn registers the
  view's listeners as a set and files each event into the view.

SDD-32hath bounds this boundary from the webview's side: the shipped policy
lets a script connect to the IPC sources and nowhere else.

### The deviation item

SDD-6g3wnh carries **no low-level requirements**, which is a deviation from
what class C asks. It holds the process entry and startup wiring, the chain
connection, the receiver loop, the commands whose success needs a chain,
`api.ts`, and every Svelte component and route. No gated test reaches that
code, so a requirement written there could not redden when its behaviour
broke. The decomposition file's section "The item with no low-level
requirements" argues it in full and books each part for the change that gives
it a harness.

### Evidence

This section names items, not figures. The unit's counts of requirements,
tests and attestations, and how each requirement was shown to redden, live in
the verification record for the change that wrote them, under
`docs/verification/`. Known defects are booked in `app/docs/problems/`.
