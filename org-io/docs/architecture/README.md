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

org-io does all IO for an Organisation: the chain (read and write, through
on-chain-client; the own-admin check is retired, owner ruling 6, 2026-11-03),
and from stage S4 the transport
and storage file IO. org-node takes values and returns values: org-io reads
the chain, hands org-node the parsed state, and submits the values org-node
returns. The app depends on org-io alone. The decision and its options are in
`docs/adr/2026-10-06-org-io-unit.md`.

No software item is defined yet: the crate lands in S2's develop phase. Every
item is class C, the unit's class, and no segregation is argued.

*Amended 2026-10-20 (change `worktree-org-io-domain-api`, consistency
pass):* the paragraph above is history; the items are defined. The in-force
software items, by what they hold (the dated files and
`2026-10-16-domain-api.md` define them):

| Area | Items | What it decides or does |
|---|---|---|
| **The handle** | SDD-z3ychz, SDD-3qwjs2 | The app's one way in, its domain surface and queries; the one random source every operation draws from |
| **Persons and Personas** | SDD-j7k844, SDD-bzu2jg | The user's Person, kept apart; a new Persona, with its keychain item, store file and endpoint, per founding and per provisional Member |
| **Invitation** | SDD-zqv3v6, SDD-9rbm8t, SDD-cxh9d8 | The Organisation's outstanding invites, issued or cancelled; an imported Invite read; the kept admission requests listed |
| **Proposals** | SDD-7mhqaz, SDD-4drrpr | Recording provisional changes and proposing the updated Organisation; no command discards |
| **Chain** | SDD-mk39e5, SDD-4bbt47, SDD-z85ux9, SDD-789u6d | Writing one provisional organisation (the founding in two steps); the read at a finalised block; the connection to a chain; the signatory key |
| **Verification** | SDD-t4z8sz | Judging a held Organisation against the chain through org-node, writing what org-node returns, settling an unknown write |
| **Transport** | SDD-2bzgtp, SDD-dhh3pw, SDD-2fc2cq, SDD-8uyg4s, SDD-xwft92, SDD-b8tuv3 | The send path to a Device key at its kept addresses; admission and organisation requests; the receive step and its one verdict; the endpoint whose identity is the Device key, one per Persona, and when it exists |
| **Storage** | SDD-egx8rv, SDD-4fvypt | The sealed store files and the OS keychain |
| **This node's removal** | SDD-3my66m, SDD-wyn7b6 | Sending the kept acknowledgements on a backoff, having org-node delete each one accepted; keeping the removed Persona's data and deleting its keychain item and then its store file when none is left or after 7 days |
