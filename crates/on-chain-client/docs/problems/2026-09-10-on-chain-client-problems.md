# Problem reports — on-chain-client's ledger opens

Reports found by the code survey behind this unit's first hazard
analysis
(`crates/on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md`)
rather than by a failing test. Each was recorded before anyone investigated a
fix, as `resolve-problem` requires.

The first four below were left open by that analysis, for the reason the
org-node half gave: they are in `client.rs`'s async paths and in `verify.rs`,
and fixing them under a hazard analysis would mix a register with a change to
the code it assesses. Each of those four now carries the owner's ruling in its
`disposition:`. The last, PR-hj4wwd, was found later, while a merge was being recorded.

## The decoder is pinned at construction and never re-checked

**PR-w5sk5k**: `OrgRegistryClient::from_client` resolves a decoder from the
runtime `spec_version` once, at construction, and stores it for the life of the
client; the field's own doc-comment says the client "should be reconstructed" if
the runtime upgrades mid-session and nothing enforces it, so a client that
outlives a runtime upgrade keeps decoding post-upgrade bytes with a decoder
compiled for the layout the previous runtime used, returning a well-typed but
wrong Organisation state or event with no error anywhere.
affects: RC-d7r82e
opened: 2026-09-10
status: accepted
disposition: ruled on 2026-10-28 — owner ruling 2026-10-28, re-confirmed 2026-10-29 on the corrected premise: the defect is real and stays as written; its fix belongs to a separate on-chain-client change, not worktree-org-io-domain-api, whose comment sweep edited doc comments in src/client.rs but not the defect

Where: `crates/on-chain-client/src/client.rs:128-146` resolves the decoder at `:137`
from the version read at `:136` and stores it at `:143`; the doc-comment that
states the obligation nothing enforces is at `:110-112`. There is no
reconstruction trigger, no periodic re-check and no error a caller could notice.

**The information needed for the check is already in hand**, which is what makes
this a defect rather than a design limit. `get_org_state` resolves a block on
every call (`:174-185`) and the resulting block object carries `spec_version()`
— the same accessor `from_client` used at `:136` — and it is never compared
against the pinned version. The comparison is one `if`.

Observable symptom, as a test would show it: on a chopsticks fork, construct a
client, upgrade the runtime under it, and `get_org_state` returns `Ok` with a
state decoded by the old decoder rather than `Err(UnsupportedRuntime)`.
Reproduced in this change by reading, not by running: the reproducing test is
the first step of the fix, per `resolve-problem`, and it needs the chopsticks
lane that no gate runs.

Why it matters more than a stale comment. pallet-revive is pre-stable both as a
pallet and as a storage and event shape, which is the reason
`crates/on-chain-client/src/decode/mod.rs:8-10` gives for the decoders being
version-gated at all; the version gate is this unit's whole answer to a layout
that changes underneath it. RC-d7r82e governs the *resolution* of a decoder and
is implemented and gated
(`crates/on-chain-client/tests/runtime_version_dispatch.rs`). This defect is that the
resolution is never repeated, so the control holds for exactly as long as the
session — and the failure it exists to prevent is precisely the one that happens
*during* a session. It is HAZ-8s5chy's residual in the register, S3/P2.

The fix, in outline (not-minted control 4 in the register): compare the resolved
block's `spec_version()` against the pinned one on each read and each
subscription, and return `UnsupportedRuntime` rather than decode. Widening
`for_runtime` to a known-good range (not-minted control 5) is a separate
question and reduces how often the loud failure fires; it does not remove the
need for the check.

*Note, 2026-10-30 (merge review round 1 of `worktree-org-io-domain-api`):*
that change's comment sweep removed one doc-comment line near the top of
`src/client.rs`, so every `client.rs` line cited in this report moved up by
one; the citations above are corrected to the current file. The code they
cite is not changed by that sweep.

## The module named for verification contains nothing

**PR-h4mb8y**: `crates/on-chain-client/src/verify.rs` is seven lines of doc-comment and
no code, while its own header states in the present tense that it is the
verifier that "closes the loop with `org_members::CandidateTrie::verify_against`"
and compares a candidate trie's root against on-chain state — so the unit named
for verification performs none, and a consumer written from the module's own
description calls a function that does not exist.
affects: RC-5e3bdk
opened: 2026-09-10
status: accepted
disposition: ruled on 2026-10-28 — owner ruling 2026-10-28, re-confirmed 2026-10-29 on the corrected premise: the defect is real and stays as written; its fix belongs to a separate on-chain-client change, not worktree-org-io-domain-api, whose comment sweep replaced src/verify.rs's header but not the defect: the module still holds no code

Where: `crates/on-chain-client/src/verify.rs:1-7` is the whole file; the module is
declared at `crates/on-chain-client/src/lib.rs:28`. The header's own last paragraph
gives the history — "Lands in Stage 2 Task 5 (after the client surface in
`client.rs` is in place). At that point `org-members` is added as a dependency;
for Task 1 this module is a placeholder so the module tree matches the plan."
Stage 2 landed. `org-members` is not a dependency of this crate. The placeholder
is still a placeholder, and `crates/on-chain-client/.guardrails/config.yaml` declares no
`depends_on:` edge from on-chain-client to anything — the key is present only as
a commented-out example. (An edge lives in the depending unit's own config, not
in `.guardrails/units.yaml`, whose schema is `units:` and `not_a_unit:` alone.)

Observable symptom: the module exports nothing, so there is nothing to call. It
is not a wrong answer, it is an absent one, and it is visible only to a reader
who opens the file rather than trusting the design or the module tree.

Why it is filed rather than noted. The consumer does perform the root comparison
— `crates/org-node/src/verify.rs` does it, in the consumer's own arrangement, and the
consumer's register assesses that control — so nothing shipped is broken today.
What is wrong is that the design assigns the comparison to a provider and the
provider does not perform it, which under IEC 62304 Class C is not a lesser
category of defect: the next consumer either duplicates the work or trusts a
module that is empty, and a second implementation of a security-relevant
comparison is exactly the kind of divergence a shared one prevents. It is a
hazard in prose in the register, S3/P1.

`affects: RC-5e3bdk` because ownership of a log is the closest control this unit
has to the comparison the module promises: both are the question "is this the
Organisation's own value", answered on the event path by RC-5e3bdk and on the
root path by nothing here.

The fix, in outline (not-minted control 6): either implement the module — which
means taking `org-members` as a dependency, declaring the edge in
`crates/on-chain-client/.guardrails/config.yaml` after assessing the new edge (a
dependency edge is declared only once its assessment is done), and writing the
comparison with a gated test — or delete the module and move its promise into
the design, where a reader will not mistake it for shipped code. Either closes
the report. What is not acceptable is a module whose header describes in the
present tense a verification it does not perform.

*Note, 2026-10-30 (merge review round 1 of `worktree-org-io-domain-api`):*
the header quoted above ("Lands in Stage 2 Task 5 … placeholder") no longer
exists. That change's comment sweep replaced `src/verify.rs`'s header; the
module is now "reserved for a verifier" and states that it holds no code and
that the crate does not depend on `org-members`; the file is now five lines
of doc-comment (`src/verify.rs:1-5`), not seven. The module still holds no
code, so the defect stands: the design still assigns the comparison to this
unit and the unit does not perform it. The remaining fix, implementing the
module or deleting it and moving its promise into the design, stays for the
separate on-chain-client change the disposition names.

## The best lane's first backfill span is unbounded

**PR-uq5r97**: the best-block lane seeds its scan from the latest finalised
block, which on a live chain lags the best tip, so the **first** best-head
notification after a subscription backfills the entire lag — every height from
the finalised head to the new best head, at one `at_block` round-trip per height
— before any event of interest is delivered.
affects: RC-kemv75
opened: 2026-09-10
status: accepted
disposition: ruled on 2026-10-28 — owner ruling 2026-10-28, re-confirmed 2026-10-29 on the corrected premise: the defect is real and stays as written; its fix belongs to a separate on-chain-client change, not worktree-org-io-domain-api, whose comment sweep edited doc comments in src/client.rs but not the defect

Where: `crates/on-chain-client/src/client.rs:392-400` captures the seed from
`at_current_block()`, which subxt documents as the latest finalised block
(`subxt-0.50.1/src/client/online_client.rs:269`); the seeding comment explains
why a seed is needed at all — `chain_subscribe_new_heads` pushes only future
heads and does not replay the current one (`:374-383`); and the loop that pays
for it is `for number in from..=to` at `:443`, whose span comes from
`scan_step`'s `from` (`:601-604`). **The code admits it**, in a `NOTE` at
`:435-442`: "this span is unbounded … a long-lived watcher on a live chain may
want a cap + resync strategy before relying on this lane."

Observable symptom: on a chain with a finality lag of L blocks, the first
notification issues L round-trips and delivers L blocks' events, some of them
for blocks imported before `subscribe()` was called. On the manual-mining forks
the seven fork-dependent ungated targets use, L is 0 or 1 and the behaviour is
invisible, which is why no test has ever shown it. (**Corrected 2026-09-28 by
the review sweep of the change `worktree-guardrails-on-chain-client-arch`:**
this said "the nine ungated targets". Nine run at no gate, but `smoldot_smoke`
needs live Paseo rather than a manual-mining fork and `regenerate_corpus` needs
no chain at all.) Reproduced in this change by reading, not by
running.

Why it is a hazard and not only a performance note: it is unavailability on the
consumer, S3/P2 in the register under HAZ-xd4urb. A subscription that spends its
first seconds or minutes replaying history is a subscription that has not yet
delivered the membership change the consumer is waiting for, and the cost scales
with the chain's lag rather than with anything the consumer controls. The
duplicate delivery is separately harmless — the lane is documented as
best-effort and consumers are told to de-duplicate (`:323-327`) — and is not
what this report is about.

The fix, in outline (not-minted control 7): cap the span, and resync by an
explicit `get_org_state` read rather than by replay when the cap is exceeded.
The seed's hash must stay the finalised hash for the dedup and reorg comparisons
to behave (`:382-383`), so the cap belongs on the span and not on the seed.

*Note, 2026-10-30 (merge review round 1 of `worktree-org-io-domain-api`):*
the `client.rs` line citations above are corrected to the current file. That
change's comment sweep removed one doc-comment line near the top of
`src/client.rs`, and the citations, written against the file of 2026-09-10,
had already drifted by more than that line. Each passage cited reads as
quoted.

## A reorganisation deeper than the notification gap is undetectable

**PR-qpp28h**: the best lane derives reorganisations from the parent hash of
each best-head notification, so it detects only a head at or below the last one
processed and a head at the next height with a foreign parent; for a jump of two
or more heights it reports no reorganisation at all, and the by-number backfill
that fills the gap cannot supply one either, so a consumer that acted on a block
the chain then discarded across such a gap is never told.
affects: RC-kemv75
opened: 2026-09-10
status: accepted
disposition: ruled on 2026-10-28 — owner ruling 2026-10-28, re-confirmed 2026-10-29 on the corrected premise: the defect is real and stays as written; its fix belongs to a separate on-chain-client change, not worktree-org-io-domain-api, whose comment sweep edited doc comments in src/client.rs but not the defect

Where: the reorg predicate is `crates/on-chain-client/src/client.rs:595-600`, inside
`internals::scan_step`. **The code admits it**, in the `best_lane` doc-comment
at `:366-372`: "For jumps `n > last.number + 1` the intermediates come from
canonical backfill and no reorg signal is derivable — acceptable for the
manual-mining scenarios." And the backfill genuinely cannot help: looking a
height up by number "ALWAYS resolves the canonical best block at that height —
so it can never observe a discarded sibling" (`:351-353`).

Observable symptom, as a test would show it: with the lane at height 10, deliver
a head at height 13 whose ancestry does not include the block previously at 11
or 12; the lane reports `reorged: None` and backfills 11, 12, 13 from the new
canonical chain, and the consumer is never told that its belief about 11 came
from a block that no longer exists. `crates/on-chain-client/tests/best_lane_reorg_rule.rs`
pins that behaviour today, deliberately and with a label: the jump case asserts
`reorged: None` as *today's behaviour under this report*, not as a claim that
nothing was discarded. Reproduced in this change by reading, not by running.

Why it is a hazard: HAZ-xd4urb in the register, S3/P2 — reorganisations are
ordinary chain behaviour, and the depth this rule can see is bounded by how
often the notification stream happens to deliver, which is a property of the
transport and the block time rather than of the risk. Note that the harm needs
the consumer to have acted on the provisional observation; a consumer that waits
for a Finalised observation is not exposed, which is what makes
`SubscribedEvent`'s best/finalised distinction the more important half of
RC-kemv75.

Also recorded here rather than filed separately, because it is the same gap seen
from the other side: **there is no gated evidence that the notifications this
rule is fed are the notifications the chain produced.** The only test that
exercises delivery is `crates/on-chain-client/tests/reorg_cancels_proposed.rs`, which
needs a chopsticks fork and runs in no gate — and chopsticks finalises every
`dev_newBlock` immediately, as that file's own header records, so it cannot be
evidence about real finality even when it is run. The only test that could be is
`crates/on-chain-client/tests/smoldot_smoke.rs`, which is `#[ignore]`d and needs live
Paseo.

The fix, in outline (not-minted control 8): detect a discarded ancestor rather
than a discarded head — walk back from the new head's parent until a hash
already processed is reached, and report every processed head not on that path
as discarded. That makes the detectable depth a property of what the lane
remembers rather than of how often it was notified. It costs one header fetch
per step of the walk, which is the same currency the backfill already spends.

*Note, 2026-10-30 (merge review round 1 of `worktree-org-io-domain-api`):*
the `client.rs` line citations in "Where" are corrected to the current file.
That change's comment sweep removed one doc-comment line near the top of
`src/client.rs`, and the citations, written against the file of 2026-09-10,
had already drifted by more than that line. The sentence quoted there sits
in `best_lane`'s doc-comment, not `subscribe`'s; the text said `subscribe`.
Each passage cited reads as quoted.

## CI does not run the writer set of the verify commands

**PR-hj4wwd**: CI does not run on-chain-client's second `verify_commands` line, the writer set (`cargo test --manifest-path crates/on-chain-client/Cargo.toml --features test-support,write --test write_manifest --test write_pure --test write_compose --test write_events --test signatory_set_decode --test fuzz_signatory_set`), so a change that breaks the chain writer's pure half or the signatory-set decode passes CI and is caught only by the local gate.
affects: REQ-aat4yt, REQ-6jefu2, LLR-24mwew
opened: 2026-10-31
status: open

Where: `.github/workflows/rust.yml`. Its step under the comment
"on-chain-client's verify_commands entry" runs only the first of the two
lines in `crates/on-chain-client/.guardrails/config.yaml` `verify_commands`. No
step names `--features test-support,write` or any of the six writer
targets, and the `clippy` job stays on `--lib`. The second line predates
change `worktree-org-io-domain-api`: it came with the chain writer, and that
change neither added nor altered it. Found while writing that change's
verification record (merge-change step 5, 2026-10-31). Expected: CI runs
every `verify_commands` line of the unit, the writer set included, as it
does for org-node's and org-io's lines.
