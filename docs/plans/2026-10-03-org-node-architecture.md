# org-node — architecture, SOUP and low-level requirements

Tooth 4 of the ratchet (`docs/plans/2026-09-05-ratchet-gap-analysis.md`, which
numbers it 6) for the **org-node** unit: the first architecture ledger, a
measured SOUP inventory, and the low-level requirements class C asks for.

Branch: `worktree-guardrails-org-node-arch`. Unit config:
`org-node/.guardrails/config.yaml` (class C, no per-item override).

## Why org-node before app

The register's rule is that a consumer inherits its providers' SOUP, and
`app depends_on org-node`. app's inventory cannot be written until org-node's
exists. org-members (`ec66743`) and on-chain-client (`343fd64`) are done.

## The baseline, measured on this branch

`master` was merged in **six** times while this change was open. The branch
carries six merge commits, and `git log --merges master..HEAD` names them:

| Merge commit | Brought in | What it changed here |
|---|---|---|
| `4f4ec19` | `5850ed7` | Part 2 of the quint-conformance ADR: the protocol models moved into `org-node/quint/`, and this unit's single `quint typecheck` became two typechecks and five simulator invariants. Conflicted on `verify_commands`. |
| `cb6a357` | `f5e16d2` | Carried during the falsifiability sweep; no conflict. |
| `4ebe1c0` | `0f85cb9` | The random-`MemberId` change: REQ-d9g6nt, a third bolero target `fuzz_first_admission_base`, and the resolution of PR-g7cfns. Conflicted on `verify_commands` again, and on `admission_sender.rs`. |
| `23749b8` | `582a0c2` | The recalculate-refusal change: `OrgTrie::genesis` no longer recalculates, so `fuzz_verify_against_chain`'s fixture changed under this branch and was re-probed after the merge. No conflict. **It also removed `org-members` from this change's impact set** — a branch behind `master` on a path reads as having touched it, and this branch stopped being behind. |
| `f1262e2` | `24317e3` | Parse at the system's edge: org-members' constructors take `Handle`, `Name`, `Surname` and `RootHash::new`. No textual conflict, but two branch-added call sites stopped compiling (the fuzz target's joiner leaf, `service_lifecycle`'s `RootHash::from_bytes`), fixed in `a760577`. Brought PR-hqwpg9. No `service.rs` line moved. Every review-round-4 probe was re-run on the merged tree. |
| `ed5b66d` | `f688a3c` | The `person` unit and its identity types. Touches none of this change's paths; `docs/CONTEXT.md` gains three glossary entries, whose `_Avoid_` terms were scanned against this change's documents (no hits). No `service.rs` line moved, so every appendix citation stands. |

*Corrected 2026-10-04 by review round 2. This paragraph said "twice", and named
`2405ede` and `5850ed7`. `2405ede` is not a merge parent of this branch at all
— it is an ancestor of `5850ed7`, the quint-connect coupling change that moved
the membership model into `org-members/quint/` and raised org-node's
`rust-version` to 1.85, and it arrived here inside the `5850ed7` merge rather
than as a merge of its own. The verification record gave a third list, also
wrong, which is now corrected against the same `git log` output. Three
documents, three different accounts of a fact git answers exactly: the lesson
is that a merge history belongs in a table generated from the repository, not
in a sentence written from memory a week later.*

**`5850ed7` conflicted with this change on `verify_commands`**, because both
rewrote that key. Resolved to carry both: the cargo line is this change's (it
names the six relocated-test targets the other change could not have known
about), the quint lines are `5850ed7`'s unchanged. The first resolution put a
comment between two list items and the config validator rejected it, which is
recorded in the config beside the fix.

All figures below are the **baseline at the branch point `f635acc`**, not the
tree as it stands. Review round 2 re-measured them there and they hold. The
current figures are in the verification record's gate table; the unit is 3768
lines over 24 files now, and the two numbers are different because this change
deleted eight `#[cfg(test)]` modules from `src` — which is also how the README
came to be carrying the baseline figure as though it were current.

| Measurement | Value |
|---|---|
| Source | 4004 lines, 24 files |
| Largest file | `service.rs`, 1619 lines (40%) |
| Gated tests | 53 passed, 0 failed |
| quint (two typechecks, five invariants) | exit 0 — quint 0.33.0 |
| `check-trace.sh` | exit 0 — REQ 19, HAZ 6, RC 9, **SDD 0, LLR 0**, PR 6 |
| One mutation cycle | 15 seconds (measured, `sequence.rs` probe) |

Four problem reports are open at 23–24 days against `problem_age_days: 30`:
PR-vt244s, PR-2dmjzj and PR-u4c2vp fire **2026-10-10**, PR-w88sr9 **2026-10-11**.
This change must reach its squash before then or the gate reddens underneath it.

## The decomposition — nineteen items

An item is a responsibility with an interface, not a file. Several items share
`service.rs`; one item spans seven whole files and parts of two more.

| Item | Responsibility | Source |
|---|---|---|
| SDD-swtd3w | Value types and the rejection vocabulary | `ids.rs`, `error.rs` |
| SDD-sxp8hb | Device key and Member-as-a-group key custody | `keys.rs` |
| SDD-kk2y3e | The signed Envelope | `envelope.rs` |
| SDD-d8ktxa | The replay watermark | `sequence.rs` |
| SDD-na9nc3 | Verify-against-chain | `verify.rs` |
| SDD-pa6p7w | The chain as a read oracle | `chain.rs` |
| SDD-kwncn7 | The wire frame and its bound | `transport/wire.rs`, `transport/mod.rs` |
| SDD-8uyg4s | The authenticated endpoint | `transport/endpoint.rs` |
| SDD-af5vnt | The encrypted persona store | `store.rs` |
| SDD-vee2fq | Out-of-band exchange blobs | `blobs.rs` |
| SDD-msb6xh | Update calldata | `chain_write/calldata.rs` |
| SDD-rq6nv4 | Threshold-1 multisig account derivation | `chain_write/multisig.rs` |
| SDD-ueh4tm | The chain-operations seam | `service.rs` (`ChainOps`, `MockChainOps`) |
| SDD-89es4z | Persona and organisation genesis | `service.rs` |
| SDD-rx2yvy | Admission | `service.rs`, with `blobs.rs` |
| SDD-8cpyfa | The receive-and-commit path | `service.rs` (`receive_and_verify`) |
| SDD-72ddm6 | Revocation and self-delete | `service.rs` |
| SDD-b8tuv3 | Endpoint lifecycle in the service | `service.rs` (`ensure_endpoint`) |
| SDD-z85ux9 | The chain-facing I/O shell — **no LLRs, a deviation** | seven files and parts of `service.rs` and `multisig.rs` |

## The deviation, stated before it is written

SDD-z85ux9 carries **no low-level requirements**, under a class that requires
them. It is the async chain-I/O: `chain_read.rs`, `chain_write/proxy.rs`,
`chain_write/submit.rs`, `chain_write/mod.rs`, `ceremony.rs`, `preflight.rs`,
`bin/preflight.rs` and `service::connect_chain_client` — about **815 lines, a
fifth of the unit**. *Re-measured 2026-10-04 by review round 6: **1125 lines,
about 30%**, once `SubxtChainOps`, `submit_and_watch`, `fund` and
`dispatch_org_call` are counted. The measurement is in the decomposition.*

This is **not** on-chain-client's SDD-3b8zef situation, and the record must not
borrow its argument. That item was unreachable: nothing exercised it. These
functions *are* exercised — by `chain_genesis_e2e`, `finality_polling` and
`preflight`, which exist and pass. They are excluded from `verify_commands` by
a deliberate decision recorded in the unit config, because they spawn a
chopsticks fork and need `on-chain/scripts/node_modules`.

So the honest statement is narrower and worse: an LLR here could be written and
could be true, but **no mutation to it would redden this unit's gate**, so it
would be an assertion with no evidence behind it — the precise failure this
repository has now recorded three times. The remedy is to bring the chopsticks
targets into the gate, which is an infrastructure change of its own, and it is
booked rather than attempted here.

## Tasks

1. **SOUP inventory** — measured from the repository-root `Cargo.lock` (org-node
   is a root-workspace member; it has no lockfile of its own, unlike
   on-chain-client). Direct dependencies with exact versions, the role of each,
   the requirements it supports, known anomalies. Record the closure size and
   how it was measured.
2. **Write the nineteen items** into
   `org-node/docs/architecture/DRAFT-worktree-guardrails-org-node-arch-decomposition.md`,
   each tracing to ≥1 REQ, with LLRs under each (SDD-z85ux9 excepted).
3. **Annotate the tests** — `verifies:` on each gated test naming the LLRs it
   carries. The parent REQs are then covered transitively.
4. **The falsifiability sweep** — decompose every LLR into independently
   falsifiable claims; one mutation per claim; RED if a named test fails, GREEN
   means the clause is unfalsifiable at this gate and is cut. Existential rule:
   GREEN is a verdict about the search, not the clause. Restore from pristine
   copies and prove byte-identical by SHA-256.
5. **Overview section** in `org-node/docs/architecture/README.md` — the standing
   decomposition picture every later change amends.
6. **Gates, independent review, merge** — `merge-change` over the impact set
   (`org-node` touched, `app` dependent).

## Red → green attestations

**They are in the verification record**, as the appendix "The per-mutation
attestations" — one row per run, each naming the clause mutated, the verdict,
and the tests that failed, in one block for the author's sweep and one for
each review round. *Corrected 2026-10-04 by review round 5, which found this
saying "three blocks" over an appendix holding five. It now names the rule
rather than a count that every round changes.*

*This table used to sit here reading "(pending)" while two other documents
pointed at it as the source. Review round 1 found the attestations existed in
no document at all, and wrote the appendix. Review round 2 then found the
appendix had no row for six low-level requirements and one row filed under the
wrong item's ID — so the first version of this fix was itself incomplete, which
is the pattern of this change. The appendix now names the three low-level
requirements that are **not** discharged by a red, and why, rather than leaving
a reader to infer that every one of them is.*

*A row count is deliberately not given here. This section carried "129 rows"
for a day and the number was out of date the moment the next round ran; the
record is the one place that counts them.*
