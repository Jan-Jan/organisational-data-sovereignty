# Architecture ledger

This directory is a per-change ledger: each change contributes one dated
file, `YYYY-MM-DD-<slug>.md`, dated the day the change creates it and never
renamed. **Edit existing items in the file that defines them.** A reworded
item keeps its ID; a changed behavior gets a new ID and the old item is
deleted; a dropped item is deleted. The squash message records each under
`Items:`. Use this README's Overview section for the system-wide decomposition
picture; `soup.md` (single file) contains the SOUP inventory.

<!--
Item grammar (enforced by .guardrails/scripts/check-trace.sh):

  **SDD-NNNNNN**: <software item and its responsibility>. traces: REQ-NNNNNN[, REQ-...]

Low-level requirements (design data — the directly codeable refinement of
the high-level REQs), written under the software item they belong to:

  **LLR-NNNNNN**: <directly codeable behavior>. satisfies: REQ-NNNNNN[, REQ-...]
  **LLR-NNNNNN**: <behavior with no parent requirement>. satisfies: derived

- An item is terse: no rationale, history or implementation detail.
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

*First written 2026-10-05 by the architecture tooth. This section is the standing
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
| **Rust: startup** | SDD-k95rmp, SDD-aq7m6b | Which passphrase, data directory, transport mode and verification interval the node runs with, when startup is refused, and how the managed state is assembled from those values |
| **Rust: the command side** | SDD-2pa6h6, SDD-6pjvhc, SDD-6zqqm8, SDD-q4dpym | What an operator-supplied value must be before it reaches org-io, what each command of org-io's domain surface returns, what a verification or a write asks for next (a send, a request, a verification), and what is reported about the running configuration |
| **Rust: periodic verification** | SDD-tgu7dp | That every held Organisation is verified once per interval while the app runs, through the same body as every other verification |
| **Rust: the receiver side** | SDD-5fchuy, SDD-mwqf6x | Whether a receiver loop may start, and how each received message and verification result becomes a named outcome and event with a payload |
| **Webview** | SDD-8jw9mn, SDD-swt9fs, SDD-ua2d9b, SDD-xg4nj7, SDD-p7sdy3, SDD-czjf2c, SDD-7aq69w, SDD-g5wmxt | How events become the verification view; what a verification shows; when the webview asks for one; the notices of a membership change in flight; the Admin's commands that take a change to the chain (propose the next epoch, write it, cancel an Invite); whether a revocation may be proposed; the invitee's confirmation before the first admission request; whether a refused Person stops the founding |
| **Configuration** | SDD-32hath | What any script in the webview may load, run or connect to |
| **Untested shell** | SDD-6g3wnh | Everything above, wired together at its call sites, that no gated test reaches |

*Amended 2026-10-20 (change `worktree-org-io-domain-api`, consistency
pass):* the table is brought up to the in-force items. SDD-6pjvhc replaces
the earlier command surface (the commands carry org-io's domain-surface
names), the warning before the first admission request replaces the Loopback
address check, and the verification view replaces the reconcile view;
SDD-tgu7dp (periodic verification) and SDD-xg4nj7 (the admission and notices
view) are new, all in `2026-10-16-domain-api.md`. Since that
change the Rust half drives org-io, not org-node: the app names no org-node
item, and org-io holds the node, the chain connection and the transport.

*Amended 2026-10-28 (change `worktree-org-io-domain-api`, round 3 review
AR3-5 and AR3-6):* the verification view's item is split into SDD-swt9fs
(the display) and SDD-ua2d9b (when the webview asks), and the pre-command
input decisions' item into SDD-czjf2c (the revocation input) and SDD-7aq69w
(the admission-request confirmation), each for holding two responsibilities; SDD-g5wmxt (the creation form's refused-Person
decision) is new.

*Amended 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 1, MR1-A1):* SDD-p7sdy3 (the Admin's epoch commands in the webview:
"Propose next epoch", "Write to chain" for a proposed update, cancelling an
Invite) is new, in `2026-10-16-domain-api.md`.

The policy decisions are pure: SDD-k95rmp and SDD-q4dpym take every input as
a parameter and read no environment, and SDD-mwqf6x is a total mapping from
outcome to events. That is what lets the gate reach them without a live chain
or a running application.

### The IPC boundary

The two halves meet only over Tauri IPC, in two directions.

- **Webview to Rust: commands.** The frontend invokes the commands of
  SDD-6pjvhc's surface by their snake_case names (those whose success needs a
  chain are SDD-6g3wnh's), through the wrappers in
  `app/src/lib/api.ts` (SDD-6g3wnh). This is the trust edge: SDD-2pa6h6 parses
  the identifiers, addresses and secrets those commands take again on the
  Rust side, whatever the webview checked first, so SDD-czjf2c's and SDD-7aq69w's checks are
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
