# Architecture ledger

This directory is a per-change ledger: each merged change contributes one
dated file, `YYYY-MM-DD-<slug>.md` (merge date, assigned by `merge-change`
from your worktree's `DRAFT-<branch>-<slug>.md`). **Edit existing items in
the file that defines them.** Use this README's Overview section for the
system-wide decomposition picture; `soup.md` (single file) holds the SOUP
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
  assessed in the risk ledger.
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

*First written 2026-10-03 by the architecture tooth
(`docs/plans/2026-10-03-org-node-architecture.md`). This section is the
standing picture; each change amends it rather than restating it in a dated
file.*

### What this unit is

org-node is the node. It holds a member's keys, decides whether a received
change to an Organisation's membership is one it will commit, and performs the
chain and peer-to-peer I/O through which access is granted or withdrawn. It is
IEC 62304 **class C** throughout, with no per-item override.

It consumes two units and is consumed by one: `org-members` supplies the
membership trie and the delta algebra, `on-chain-client` supplies the chain
reading, and `app` drives it. Both providers are class C, so no
`segregated_from:` entry is needed and the class floor has nothing to convict.
What this unit relies on beyond what its providers state as requirements is
written as `expects:` items in its own SRS — REQ-ysyu9g against
`on-chain-client` and REQ-q92yac against `org-members`.

### The decomposition

**Nineteen software items.** An item is a responsibility with an interface,
not a file: seven of them live wholly or partly in `service.rs`, and one,
SDD-z85ux9, spans seven whole files and parts of two more. They group into
five layers. *Corrected 2026-10-05 by review round 8: this said "eight" and
"parts of six modules", neither of which was counted.*

| Layer | Items | What it decides |
|---|---|---|
| **Values and custody** | SDD-swtd3w, SDD-sxp8hb | What an identifier is, what a refusal is called, and which key plays which role |
| **The commit rule** | SDD-kk2y3e, SDD-d8ktxa, SDD-na9nc3, SDD-pa6p7w | What a change must prove before it is believed |
| **Carriage and rest** | SDD-kwncn7, SDD-8uyg4s, SDD-af5vnt, SDD-vee2fq | How bytes reach the node, and where secrets sit when they are not moving |
| **The five stories** | SDD-ueh4tm, SDD-89es4z, SDD-rx2yvy, SDD-8cpyfa, SDD-72ddm6, SDD-b8tuv3 | How the parts compose into what a Persona does |
| **The supplier edge** | SDD-msb6xh, SDD-rq6nv4, SDD-z85ux9 | What is said to the chain |

**The decisive property is one function.** `verify_envelope_against_chain`
performs eight checks in a security-critical order, and ten low-level
requirements refine that one ordering (SDD-na9nc3). A received change is
committed only if applying it to the local record reproduces a root that
independently matches the root the chain reports at an epoch newer than the
last committed one. Everything else in this unit either feeds that function or
acts on its verdict.

**Two seams exist so that rule can be tested.** `ChainOps` (SDD-ueh4tm) makes
the chain substitutable, and `ChainReader` (SDD-pa6p7w) makes the trusted-root
oracle substitutable. Without them the five user stories would be reachable
only with a live chain, and four items would lose all their evidence.

**A seam is only as good as the substitutes written for it.** Until 2026-10-04
every `ChainReader` this unit's gate could reach returned `Ok` — `MockChain`
always succeeds and `ChainOpsReader` wraps a state already read — so *absence*
from the chain was tested and *failure to read* it was not, although the two
are different rejections and the code distinguishes them. Review round 2 found
that; `FailingChain` in `tests/verify_against_chain.rs` closes it. The general
lesson belongs in this standing section rather than in a dated one: a seam that
only ever carries a substitute that succeeds is tested on one side.

### Where the gate can and cannot see

This unit's gate reaches about seventy per cent of it. **SDD-z85ux9 — the
chain-facing I/O shell, 1125 of 3768 source lines measured — carries no
low-level requirements**, which is a deviation from what class C asks and is
argued in full in the dated decomposition file rather than here. The short
form: that code *is* exercised, by `chain_genesis_e2e`, `finality_polling` and
`preflight`, but those three targets are excluded from `verify_commands`
because they spawn a chopsticks fork, so no mutation to the code they cover
would redden this unit's gate. Writing low-level requirements there would
assert properties no gate can observe. The remedy — bringing those targets
into the gate — is booked in `docs/plans/2026-09-05-ratchet-setup.md`.

Further things the gate cannot observe are named where they live, in the
dated decomposition file's sections on behaviours deliberately not refined and
in the verification record's Gaps:

- `RelayMode::Disabled` on the Loopback arm. A relay home is acquired only
  after `online()`, so an assertion at bind time is vacuous.
- REQ-2wzfzv's bind failures, which no gated test can produce.
- The Networked arm of `admit_member` and `revoke_member`, and
  `ensure_endpoint`'s use of the transport mode. Both need an injected-relay
  constructor this unit does not have.
- Which Persona a receive operation binds its endpoint from. Every receive
  test injects its endpoint.

### Evidence

Every low-level requirement outside SDD-z85ux9 is carried by a test that runs
at this unit's gate, and was **discharged by red by mutation** rather than by
being observed already green, **with named exceptions**. The dated
decomposition file lists them, each with its reason:

- four requirements that are compile-enforced, not test-enforced;
- LLR-6adc99, which no gated test can reach;
- the first-admission clause of LLR-u6rq4s, where the skip it describes is
  unnecessary rather than unverified;
- the administrator-key exclusion in LLR-e5c9ud, which sits inside PR-mdv38y's
  defect.

The attestations are in the verification record for the change that wrote
them.

*Corrected 2026-10-04 by review round 5, which found this section claiming
every requirement discharged by red over a decomposition that names seven
exceptions, and the list above it naming two of the five unobservable
behaviours. It now names the exceptions and where they live, rather than a
count each round changes.*

Seven known defects are pinned rather than fixed, each by a test that asserts
today's behaviour and reddens when it is corrected: PR-vt244s, PR-b9wab3,
PR-mdv38y, PR-8qsnhx, PR-xwek5e, PR-322qst and PR-u4c2vp, in
`org-node/docs/problems/`. *PR-u4c2vp added 2026-10-05 by review round 8; its
pin was written in round 7.*
PR-mdv38y and PR-8qsnhx share one cause: the code assumes one Persona, one
Organisation and one endpoint per store, and nothing enforces that. PR-322qst,
PR-xwek5e and PR-u4c2vp do not. Each reproduces on a store holding one Persona and one
Organisation, and each has a cause of its own. *Corrected 2026-10-04 by review
round 7, which measured that this sentence said "the last four".*

`test_paths` reads `verifies:` annotations only from `org-node/tests`, so a
test in a `#[cfg(test)]` module under `src` cannot carry one. Twenty-two such
tests were relocated into `tests/` in 2026-10-03 for exactly that reason,
following the move this unit first made on 2026-09-09; `org_node::test_support`
is the seam by which a crate-private item that carries a low-level requirement
reaches an integration test.
