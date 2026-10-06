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

`on-chain-client` is a reader. It decides what the chain says an organisation's
membership is — decoding `OrgRegistry` state and pallet-revive events — and a
wrong or stale reading feeds every consumer's access decision, which is why the
unit is class C. It writes nothing to any chain and holds no membership policy.

Nine items divide it along the chain of custody of a byte: what it is typed as,
how an address is derived from it, where in contract storage it lives, which
decoder is allowed to read it, what that decoder refuses, who is admitted to
see the result, how provisional observations are ordered, and what the
transport does with all of it. The items are defined in
`2026-09-28-decomposition.md`, contributed by the change
`worktree-guardrails-on-chain-client-arch`, which stays the authority for their
text.

| Item | Responsibility | Source |
|---|---|---|
| SDD-5wamsz | the public value types whose widths mirror the contract ABI, the vocabulary an observation is reported in, the typed error a caller of the client surface is handed, and the stream those observations arrive on | `src/types.rs`, `src/state.rs`, `src/client.rs` (`ClientError`, `SubscribedEventStream`) |
| SDD-5b8wxs | the mapping from a 32-byte account identifier to the 20-byte Organisation admin pallet-revive gives it | `src/h160.rs` |
| SDD-bw7v5x | the arithmetic turning an Organisation admin into the storage keys its state is held at | `src/client.rs` (`internals::solidity_mapping_slot`, `internals::increment_slot`) |
| SDD-v2rtka | the `Decoder` interface, the typed error vocabulary, and the single place a Runtime spec version resolves to a decoder | `src/decode/mod.rs`, `src/decode/dispatch.rs` |
| SDD-d5jh6t | the decoder turning a `ContractEmitted` payload into a typed event paired with its Emitting contract, and refusing everything else | `src/decode/v_paseo_ah.rs` |
| SDD-f2s7bx | the decoder turning three concatenated storage slots into an Organisation state | `src/decode/v_paseo_ah.rs` |
| SDD-4z3k2u | whether a decoded log belongs to the organisation this reader watches | `src/client.rs` (`internals::log_is_ours`, `event_admin`) |
| SDD-m59zrg | the best lane's per-notification decision: seen before, reorganised, and which heights to read | `src/client.rs` (`internals::scan_step`, `internals::ScanStep`) |
| SDD-3b8zef | the subxt-backed transport shell — **no LLRs; recorded deviation** | `src/client.rs` (remainder) |
| SDD-yg7n55 | the chain writer, behind the `write` feature: the genesis ceremony and update submission through the Organisation's proxy, composed over the `WriteOps` seam (its subxt implementation carries no LLRs, as SDD-3b8zef) | `src/write/` |

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This unit is no longer only a reader. The owner moved the chain write out of
org-node into this unit, and the app calls it: SDD-yg7n55, defined in
`2026-10-06-chain-write.md`, refines REQ-6jefu2
and REQ-aat4yt. The first paragraph of this Overview ("It writes nothing to any
chain") and the count "Nine items" describe the unit without the `write`
feature, which is how it still compiles by default; with it there are ten. The
writer holds no key and stores nothing, takes this unit's own value types, and
depends on neither org-node nor org-members. It is class C like the rest and
has no segregation boundary of its own.

**Source cells corrected 2026-09-28 by this change's review sweep.**
SDD-4z3k2u read `` `src/client.rs` (`internals`) ``, but the item is
`internals::log_is_ours` **and** `event_admin`, and `event_admin` is not inside
`mod internals`: the module spans `client.rs:488-598` and itself does
`use super::event_admin;` at `:493`, while `event_admin` is defined at `:694`.
SDD-m59zrg is narrowed to `internals::scan_step`, `internals::ScanStep` in the
same pass. **Review round 4 completed the sweep, which had missed one row.**
SDD-bw7v5x was still `` (`internals`) ``, and bare `internals` swallows
`log_is_ours` and `scan_step` — so the three items' sources overlapped, which is
the defect this note describes repairing. It now reads
`internals::solidity_mapping_slot`, `internals::increment_slot`.

**Six of the nine rows now name exactly the same source in all three tables.**
Review round 5 checked and the remaining three differ in *granularity*, not in
substance: SDD-d5jh6t and SDD-f2s7bx are `src/decode/v_paseo_ah.rs` here, "event
half" and "storage half" in the plan, and named by symbol in the decomposition;
SDD-3b8zef is `src/client.rs` (remainder) here and in the plan, and named by
symbol in the decomposition. That is deliberate — this table and the plan index
the items, the decomposition defines them — but the earlier claim that all three
tables "name the same source for every row" was wrong, and is withdrawn. **The
decomposition is the authority wherever they differ.**

An item is a responsibility with an interface, not a file: `v_paseo_ah.rs`
carries two items and `client.rs` carries five. `client.rs`'s `internals` block
exists precisely because three of its decisions were extracted so they could be
tested without a chain, and that extraction is what makes SDD-bw7v5x,
SDD-4z3k2u and SDD-m59zrg verifiable items rather than parts of the shell.

**Segregation.** There is no cross-class boundary inside this unit and none at
its edge: every item is class C with no per-item override, and
`on-chain-client/.guardrails/config.yaml` declares no `depends_on:` edge — the
key is present only as a commented-out example — so there is no supplier of a
different class to segregate from and no `segregated_from:` entry to cite. (A
dependency edge is declared in the *consumer's* own config, never in
`.guardrails/units.yaml`, whose schema is `units:` and `not_a_unit:` and which
cannot carry an edge in either direction; the edges onto this unit are in
`org-node/.guardrails/config.yaml` and `app/.guardrails/config.yaml`, both of
which list `on-chain-client`.) The unit is depended *upon* — by `org-node`
and `app` — and the obligations that creates run in the other direction, as
expectations addressed to this unit.

**SDD-3b8zef is a recorded deviation from class C's per-item LLR obligation**,
not a per-item class override. The majority of `client.rs` — the async subxt
transport — has no low-level requirement because nothing at this unit's gate can
verify one; llvm-cov analyses **336 of that file's lines out of the 491 it
analyses across this crate**, at **16.37%** line coverage. (The hand-computed
"roughly 580 of 699" that stood here was dropped 2026-09-28 by this change's
review sweep as unreproducible; see the decomposition file for why.) The full
statement — what was already extracted, why annotating the ungated
chopsticks targets was refused, and what would close it — is in that same
decomposition file under "The item with no low-level requirements", and it is
carried as an owner item in
`docs/plans/2026-09-05-ratchet-setup.md`.
