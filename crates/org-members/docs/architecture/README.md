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

`org-members` is the membership authority: an immutable, hash-addressed record
of who belongs to an organisation and which keys speak for them. It computes no
policy and performs no I/O. Six items divide it, along the line between *what a
member is*, *how the record is hashed*, *how the record is stored and changed*,
and *how a change crosses a process boundary*.

The six, with the source they own (added 2026-09-17: this section delegated the
decomposition picture to itself and did not contain it, so a reader sent here
had to go back to the dated file; the items are defined in
`2026-09-17-decomposition.md`, which stays the authority for their text):

| Item | Responsibility | Source |
|---|---|---|
| SDD-4yr9ge | the validated, canonically serialisable member record — identifier, handle, member-as-a-group key, device key set, personal fields — and the validation and normalisation every construction path applies, the path from deserialised bytes included | `src/types.rs`, `src/normalize.rs` |
| SDD-d6x85b | the hash interface that provides the Membership record's eight separated hash domains, four member, four device, per LLR-4rdqyn | `src/hasher.rs` |
| SDD-d9svdj | the Membership record's leaf on the `sparse-merkle-tree` unit's tree at depth 256 — a member record, held with its device root, that the tree places by its MemberId | `src/member_tree.rs` |
| SDD-k5wa4n | the organisation trie and its eight membership operations, with the handle and skeleton indexes that make uniqueness and confusability decidable without walking the store | `src/trie.rs` |
| SDD-55b2zj | the change set that crosses a process boundary — anchored to the record it was computed against, canonical in its encoding, usable only after its result is verified against an independently supplied root | `src/delta.rs`, and `apply_delta` / `calculate_delta` / the canonical-form check in `src/trie.rs` |
| SDD-m9gs5g | the single typed error by which every rejection leaves the crate, and the lint posture that keeps a rejection from becoming a panic | `src/error.rs`, `src/lib.rs` |

(Amended 2026-10-05: `src/device_trie.rs` no longer exists. The device
sub-trie is `crates/person/src/device_trie.rs`, and SDD-d6x85b is `src/hasher.rs`,
which implements `person`'s `DeviceTrieHasher` with org-members' own device
domain keys and sentinel; see the note under SDD-d6x85b in
`2026-09-17-decomposition.md`.)

The four conceptual lines map onto them as: *what a member is* (SDD-4yr9ge),
*how the record is hashed* (SDD-d6x85b), *how it is stored and changed*
(SDD-d9svdj, SDD-k5wa4n), *how a change crosses a process boundary*
(SDD-55b2zj) — with SDD-m9gs5g cutting across all four, since every item reports
its rejections through the one error type.

There is no segregation boundary inside the unit: every item is class C, so
IEC 62304 §5.3.5 has nothing to argue here. The boundary that matters is the
one at the crate's edge, and it is stated as a non-responsibility —
authentication, organisation binding, replay protection across time, authority,
and supply of an independent trusted root all lie **above** this crate
(`crates/org-members/src/delta.rs`, "What this crate does NOT do"). SDD-55b2zj carries
that boundary.
