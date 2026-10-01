# Problem reports — on-chain-client's ledger opens

Five reports, all found by the code survey behind this unit's first hazard
analysis
(`on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md`)
rather than by a failing test. Each was recorded before anyone investigated a
fix, as `resolve-problem` requires.

**One of the five is fixed in this change** and is marked resolved below. That
is a departure from the call org-node's half of this tooth made, which left all
of its reports open on the grounds that changing `service.rs` under a register
written the same day mixes two jobs. The owner decided on 2026-09-10 that the
event-filter defect is different in two respects: the control the register mints
for it, RC-5e3bdk, is **unimplementable** without the fix — there was no
behaviour to word a requirement from — and the fix is small, local to the
decoder and its one caller, and directly reachable by a gated test. Leaving it
open would have meant minting a control against behaviour that did not exist, or
minting no control at all for the register's most probable hazard.

The other four stay open, for the reason the org-node half gave: they are in
`client.rs`'s async paths and in `verify.rs`, and fixing them under a hazard
analysis would mix a register with a change to the code it assesses. Each has a
dated item on the setup checklist (`docs/plans/2026-09-05-ratchet-setup.md`)
with an age limit of 2026-10-10 under `problem_age_days: 30`. Four open against
`problem_open_max: 10`.

## Events were never filtered by the contract that emitted them

**PR-p5ngya**: `OrgRegistryClient` delivered every `Revive::ContractEmitted` log
whose first topic matched an `OrgRegistry` event signature to its subscribers,
whatever contract emitted it, while the doc-comment on the field holding the
configured address stated that "Events from other contract addresses are
filtered out before reaching subscribers" — so any account could deploy a
contract, emit a log carrying the `GenesisInitialized` or `RootUpdated`
signature hash and a victim Organisation's indexed admin, both of which are
public on-chain, and every subscriber of that Organisation would receive it as a
genuine event with an attacker-chosen Membership root and epoch.
affects: RC-5e3bdk
status: resolved

Root cause: the decoder discarded the one field that could have answered the
question and the caller's check was a stub. `parse_revive_event` decoded the
`ContractEmitted` payload's `contract` field into `let _contract` and dropped
it, and `event_matches_contract` was `let _ = (ev, contract); true` — it
returned `true` for every log from every contract. Fixed in commit `62e1607`
(merged as `8bf3445`): the address is carried out of the decoder as
`on_chain_client::state::EmittedEvent`
(`on-chain-client/src/state.rs:45-54`), bound at
`on-chain-client/src/decode/v_paseo_ah.rs:98` and returned at `:119`; the stub
is deleted; and the comparison is made in the caller. Reproducing tests:
`on-chain-client/tests/contract_address_filter.rs` (three cases, including a
log from a different contract carrying a *valid* signature and a valid indexed
admin — the spoofing attempt) and
`on-chain-client/tests/log_ownership.rs` (seven cases over the extracted
decision). Both run at the gate.

**The fix needed a second commit, and the reason is the part of this report
worth keeping.** With `62e1607` in place the prescribed red-by-mutation was run
— disable the `emitted.contract != contract` comparison, watch the spoofing
case fail — and it **did not fail**: `--test contract_address_filter` still
reported 3 passed with the comparison switched off. The three tests verify the
decoder's half, which is real and necessary; the caller's half sat inside
`async fn decode_contract_events`, which needs a chain, and every one of the
five ungated targets that constructs an `OrgRegistryClient` deploys exactly one
`OrgRegistry` and hands its address to `from_client`, so no gated test could
reach the comparison and none of the ungated ones had a second contract to
spoof from either. (**Corrected 2026-09-28 by the review sweep of the change
`worktree-guardrails-on-chain-client-arch`:** this said "the nine chopsticks
targets". Nine is the count of ungated targets; five —
`off_chain_genesis_ceremony`, `p_address_is_orgid`, `reorg_cancels_proposed`,
`scenario_a_full`, `two_orgs_one_watcher` — is the count that constructs a
client, and five is the number this argument needs.) The fix would have
closed the hole and left its decisive line unguarded: anyone could have deleted
it and every gate would have stayed green.

So the decision was extracted into `internals::log_is_ours`
(`on-chain-client/src/client.rs:508-520`) in commit `0801aa1` (merged as
`db3bbb0`), where a test reaches it without a chain, and
`decode_contract_events` now makes one call
(`on-chain-client/src/client.rs:680-682`) where it previously made two checks;
behaviour is identical, the contract check having already preceded the admin
check. The mutation is now observable three ways, all measured: the comparison
inverted reds 6 of 7 cases; the check **deleted outright** — the "anyone could
delete the line" case the first commit could not detect — reds exactly the
three rejection cases and leaves every acceptance case green; and the admin
filter ignored reds one.

What is **not** closed, and is recorded in the register as HAZ-werm85's residual
rather than here: no test in this repository exercises the real
`decode_contract_events` path. `log_is_ours` is pinned; the wiring from
`from_client`'s address to that call site is established by reading. Closing it
needs a chopsticks fixture with two deployed contracts, which is a not-minted
control.

The doc-comment quoted above (`on-chain-client/src/client.rs:108-109`) is
unchanged and is now true. Two other doc-comments that the defect left false
were corrected in the same commit: `parse_revive_event`'s description of what
`Ok(None)` means, in `on-chain-client/src/decode/mod.rs:23-29` and
`on-chain-client/src/decode/v_paseo_ah.rs:12-20`, which had attributed
contract filtering to a signature-hash mismatch. And one inline test was
renamed — `parse_event_from_other_contract_returns_none` to
`parse_event_with_unknown_signature_returns_none` — because the old name
asserted precisely the false belief the defect rested on, and leaving it would
have re-stated the defect inside the suite.

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
status: open

Where: `on-chain-client/src/client.rs:129-147` resolves the decoder at `:138`
from the version read at `:137` and stores it at `:144`; the doc-comment that
states the obligation nothing enforces is at `:111-113`. There is no
reconstruction trigger, no periodic re-check and no error a caller could notice.

**The information needed for the check is already in hand**, which is what makes
this a defect rather than a design limit. `get_org_state` resolves a block on
every call (`:175-186`) and the resulting block object carries `spec_version()`
— the same accessor `from_client` used at `:137` — and it is never compared
against the pinned version. The comparison is one `if`.

Observable symptom, as a test would show it: on a chopsticks fork, construct a
client, upgrade the runtime under it, and `get_org_state` returns `Ok` with a
state decoded by the old decoder rather than `Err(UnsupportedRuntime)`.
Reproduced in this change by reading, not by running: the reproducing test is
the first step of the fix, per `resolve-problem`, and it needs the chopsticks
lane that no gate runs.

Why it matters more than a stale comment. pallet-revive is pre-stable both as a
pallet and as a storage and event shape, which is the reason
`on-chain-client/src/decode/mod.rs:8-10` gives for the decoders being
version-gated at all; the version gate is this unit's whole answer to a layout
that changes underneath it. RC-d7r82e governs the *resolution* of a decoder and
is implemented and gated
(`on-chain-client/tests/runtime_version_dispatch.rs`). This defect is that the
resolution is never repeated, so the control holds for exactly as long as the
session — and the failure it exists to prevent is precisely the one that happens
*during* a session. It is HAZ-8s5chy's residual in the register, S3/P2.

The fix, in outline (not-minted control 4 in the register): compare the resolved
block's `spec_version()` against the pinned one on each read and each
subscription, and return `UnsupportedRuntime` rather than decode. Widening
`for_runtime` to a known-good range (not-minted control 5) is a separate
question and reduces how often the loud failure fires; it does not remove the
need for the check.

## The module named for verification contains nothing

**PR-h4mb8y**: `on-chain-client/src/verify.rs` is seven lines of doc-comment and
no code, while its own header states in the present tense that it is the
verifier that "closes the loop with `org_members::CandidateTrie::verify_against`"
and compares a candidate trie's root against on-chain state — so the unit named
for verification performs none, and a consumer written from the module's own
description calls a function that does not exist.
affects: RC-5e3bdk
opened: 2026-09-10
status: open

Where: `on-chain-client/src/verify.rs:1-7` is the whole file; the module is
declared at `on-chain-client/src/lib.rs:28`. The header's own last paragraph
gives the history — "Lands in Stage 2 Task 5 (after the client surface in
`client.rs` is in place). At that point `org-members` is added as a dependency;
for Task 1 this module is a placeholder so the module tree matches the plan."
Stage 2 landed. `org-members` is not a dependency of this crate. The placeholder
is still a placeholder, and `on-chain-client/.guardrails/config.yaml` declares no
`depends_on:` edge from on-chain-client to anything — the key is present only as
a commented-out example. (An edge lives in the depending unit's own config, not
in `.guardrails/units.yaml`, whose schema is `units:` and `not_a_unit:` alone.)

Observable symptom: the module exports nothing, so there is nothing to call. It
is not a wrong answer, it is an absent one, and it is visible only to a reader
who opens the file rather than trusting the design or the module tree.

Why it is filed rather than noted. The consumer does perform the root comparison
— `org-node/src/verify.rs` does it, in the consumer's own arrangement, and the
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
`on-chain-client/.guardrails/config.yaml` after the dependency assessment
`docs/plans/2026-09-06-dependency-edges.md` prescribes, and writing the
comparison with a gated test — or delete the module and move its promise into
the design, where a reader will not mistake it for shipped code. Either closes
the report. What is not acceptable is a module whose header describes in the
present tense a verification it does not perform.

## The best lane's first backfill span is unbounded

**PR-uq5r97**: the best-block lane seeds its scan from the latest finalised
block, which on a live chain lags the best tip, so the **first** best-head
notification after a subscription backfills the entire lag — every height from
the finalised head to the new best head, at one `at_block` round-trip per height
— before any event of interest is delivered.
affects: RC-kemv75
opened: 2026-09-10
status: open

Where: `on-chain-client/src/client.rs:348-356` captures the seed from
`at_current_block()`, which subxt documents as the latest finalised block
(`subxt-0.50.1/src/client/online_client.rs:269`); the seeding comment explains
why a seed is needed at all — `chain_subscribe_new_heads` pushes only future
heads and does not replay the current one (`:330-339`); and the loop that pays
for it is `for number in from..=to` at `:399`, whose span comes from
`scan_step`'s `from` (`:557-560`). **The code admits it**, in a `NOTE` at
`:391-398`: "this span is unbounded … a long-lived watcher on a live chain may
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
best-effort and consumers are told to de-duplicate (`:279-283`) — and is not
what this report is about.

The fix, in outline (not-minted control 7): cap the span, and resync by an
explicit `get_org_state` read rather than by replay when the cap is exceeded.
The seed's hash must stay the finalised hash for the dedup and reorg comparisons
to behave (`:337-339`), so the cap belongs on the span and not on the seed.

## A reorganisation deeper than the notification gap is undetectable

**PR-qpp28h**: the best lane derives reorganisations from the parent hash of
each best-head notification, so it detects only a head at or below the last one
processed and a head at the next height with a foreign parent; for a jump of two
or more heights it reports no reorganisation at all, and the by-number backfill
that fills the gap cannot supply one either, so a consumer that acted on a block
the chain then discarded across such a gap is never told.
affects: RC-kemv75
opened: 2026-09-10
status: open

Where: the reorg predicate is `on-chain-client/src/client.rs:551-556`, inside
`internals::scan_step`. **The code admits it**, in the `subscribe` doc-comment
at `:322-328`: "For jumps `n > last.number + 1` the intermediates come from
canonical backfill and no reorg signal is derivable — acceptable for the
manual-mining scenarios." And the backfill genuinely cannot help: looking a
height up by number "ALWAYS resolves the canonical best block at that height —
so it can never observe a discarded sibling" (`:306-309`).

Observable symptom, as a test would show it: with the lane at height 10, deliver
a head at height 13 whose ancestry does not include the block previously at 11
or 12; the lane reports `reorged: None` and backfills 11, 12, 13 from the new
canonical chain, and the consumer is never told that its belief about 11 came
from a block that no longer exists. `on-chain-client/tests/best_lane_reorg_rule.rs`
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
exercises delivery is `on-chain-client/tests/reorg_cancels_proposed.rs`, which
needs a chopsticks fork and runs in no gate — and chopsticks finalises every
`dev_newBlock` immediately, as that file's own header records, so it cannot be
evidence about real finality even when it is run. The only test that could be is
`on-chain-client/tests/smoldot_smoke.rs`, which is `#[ignore]`d and needs live
Paseo.

The fix, in outline (not-minted control 8): detect a discarded ancestor rather
than a discarded head — walk back from the new head's parent until a hash
already processed is reached, and report every processed head not on that path
as discarded. That makes the detectable depth a property of what the lane
remembers rather than of how often it was notified. It costs one header fetch
per step of the walk, which is the same currency the backfill already spends.
