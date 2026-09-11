# Requirements — reading the chain: ownership, decoding, derivation and finality

on-chain-client's first requirements of its own behaviour. Each realises a risk
control from
`on-chain-client/docs/risk/DRAFT-worktree-guardrails-on-chain-client-risk-on-chain-client-hazards.md`
and is worded from behaviour the crate has; the exceptions are REQ-9vwcwc and
REQ-5upq6n, whose behaviour did not exist before this change — the emitting
address was decoded and dropped, and nothing compared it — and which were
written here as the fix for the problem report in
`on-chain-client/docs/problems/DRAFT-worktree-guardrails-on-chain-client-risk-on-chain-client-problems.md`
that records why. Every requirement is `satisfies: derived` — it exists because
of how this reader was built and what it was partitioned to do, not because a
system-needs document asked for it — and each is assessed in that risk file.

Terms: *Organisation slot*, *Slot key*, *Emitting contract*, *Event signature*,
*Best-block observation*, *Reorg notification*, *Runtime spec version*,
*Epoch* and *Organisation admin* are defined in
`on-chain-client/docs/CONTEXT.md`; *Organisation*, *Organisation state*,
*Membership root* and *Finalised block* in the root `docs/CONTEXT.md`. An
Organisation's *signing key* has no entry of its own in either: it is the field
both glossaries name inside their definitions — the root's *Organisation state*
and this unit's *Organisation slot* — and REQ-2qa5r5 uses it in that sense and
no other. ("Organisation public key", which that requirement said until review
round 2 (finding-13), is defined nowhere and is not used here. *Finalised
observation* was listed here until review round 3 (finding-1) and is used by no
requirement below — the best/finalised distinction is stated by no requirement
of this ledger, for the reason given under "What these requirements do not say".)

Every requirement below is verified by at least one test in
`on-chain-client/tests` carrying `verifies:` with its identifier, and every one
of those targets runs at this unit's gate — the nine declared with
`required-features = ["test-support"]` in `on-chain-client/Cargo.toml` and named
in `verify_commands`, plus the three bolero targets. Twenty-three of those tests
were relocated into `on-chain-client/tests` from `#[cfg(test)]` modules inside
`on-chain-client/src` by this change, because `test_paths` for this unit is
`on-chain-client/tests` and an annotation written in `src` is read by no gate;
each kept its assertion and its independent recomputation on the way. The
abnormal-input cases beside them are new.

Two of the twenty-one carry no abnormal-input case, and the exemption is
recorded here rather than left for a reader to notice as an omission, because
class C asks for one or the other per requirement. **REQ-2qa5r5** constrains a
declaration and not a behaviour: it has no runtime input domain at all, so there
is no input to malform — a mutation of it is a mutation of the type, not of an
argument, and that is how its evidence reds. **REQ-n6v896** is the inversion
property, and its evidence is a generator that produces only structurally valid
events by construction; abnormal input to that same decoder is exactly what
REQ-88fp2h, REQ-twdu84, REQ-axcxf7 and REQ-sx5b6g state and evidence beside it.
Neither gap is closed by inventing a case, and both are argued at the
corresponding entries of the risk file's derived-requirements assessment (review
round 2, finding-5).

## Whose log it is

**REQ-9vwcwc**: The software shall deliver a decoded `OrgRegistry` event to a
subscriber only when the Emitting contract of the log it was decoded from is the
contract the reader was constructed for, so that no other filter the
subscription carries can admit a log from another contract.
(implements: RC-5e3bdk)
satisfies: derived

**REQ-5upq6n**: The software's event decoder shall report, with every
`OrgRegistry` event it decodes, the Emitting contract of the log that event was
decoded from. (implements: RC-5e3bdk)
satisfies: derived

**REQ-nygs7k**: The software shall, when a subscription names an Organisation
admin to filter on, deliver a decoded `OrgRegistry` event to that subscriber
only when the event's Organisation admin is the one named.
(implements: RC-5e3bdk)
satisfies: derived

## Which signatures are ours

**REQ-52uc8f**: The software shall recognise as an `OrgRegistry` Event
signature exactly two values, each the `keccak256` of the canonical Solidity
signature string of one of the two events the deployed contract declares —
`GenesisInitialized(address,bytes32,bytes32)` and
`RootUpdated(address,uint256,bytes32,bytes32,bytes32)` — and no other value.
(implements: RC-675a3h)
satisfies: derived

## The event decoder's shape checks

**REQ-wnjz9j**: The software shall yield no event and no error for a log whose
first topic matches no Event signature it knows, and for a log carrying no
topics at all. (implements: RC-6gfh8d)
satisfies: derived

**REQ-n6v896**: The software shall, for any structurally valid `OrgRegistry`
event, decode that event's canonical encoding back to every field the event
carried, the Emitting contract included, without loss, reordering or defaulting.
(implements: RC-6gfh8d)
satisfies: derived

**REQ-88fp2h**: The software shall refuse, with a typed error naming both the
expected and the actual value, a log whose topic count or whose data length
does not exactly match the Event signature its first topic claims.
(implements: RC-6gfh8d)
satisfies: derived

**REQ-twdu84**: The software shall refuse, with a typed error, a log whose
indexed Organisation admin topic carries a non-zero byte at any of the twelve
positions Solidity zero-pads, and shall accept the topic when all twelve are
zero. (implements: RC-6gfh8d)
satisfies: derived

## Arbitrary bytes

**REQ-axcxf7**: The software shall refuse, with a typed error, an event payload
that carries bytes beyond the fields it declares and one that ends inside a
field's own length prefix. (implements: RC-8w9wtp)
satisfies: derived

**REQ-sx5b6g**: The software shall, for any sequence of bytes offered to any of
its decoders, either decode it or return a typed error, and shall not panic or
abort. (implements: RC-8w9wtp)
satisfies: derived

## Which runtime the bytes came from

**REQ-hd6m9d**: The software shall resolve, for a Runtime spec version, only the
decoder compiled for that exact version, and shall refuse a Runtime spec version
for which no decoder is compiled in with a typed error naming the version it was
asked about, rather than resolving to another decoder or to a default.
(implements: RC-d7r82e)
satisfies: derived

## Deriving the Organisation's identifier

**REQ-tkhe3u**: The software shall derive an Organisation admin from a 32-byte
account identifier by returning that identifier's first twenty bytes when, and
only when, all twelve of its marker positions hold pallet-revive's EVM-fallback
marker byte. (implements: RC-mugta4)
satisfies: derived

**REQ-rz7fja**: The software shall derive an Organisation admin from a 32-byte
account identifier by hashing the whole identifier and taking the last twenty
bytes of the hash whenever any one of its twelve marker positions does not hold
the EVM-fallback marker byte. (implements: RC-mugta4)
satisfies: derived

## Deriving the Organisation slot

**REQ-9m2rnd**: The software shall derive the Slot key of an Organisation slot
as the hash of the Organisation admin, left-padded into the first of two
32-byte words, concatenated with the map's declared slot index written
big-endian into the second. (implements: RC-5ejucb)
satisfies: derived

**REQ-xudf25**: The software shall derive the Slot key of the field at a given
offset within an Organisation slot by adding that offset to the base Slot key as
a 32-byte big-endian value, propagating carry through every byte it reaches.
(implements: RC-5ejucb)
satisfies: derived

## The state decoder's bounds

**REQ-2qa5r5**: The software shall represent each field of an Organisation state,
and the Organisation admin whose Organisation slot holds it, as a public type
whose width is the width the contract's ABI gives that field: thirty-two bytes
for a Membership root, thirty-two for the Organisation's signing key, eight for
an Epoch, and twenty for an Organisation admin. (implements: RC-sxjnx9)
satisfies: derived

**REQ-4astjb**: The software shall decode an Organisation state only from
exactly the expected width of storage bytes, and shall refuse any other width —
shorter or longer, including none at all — with a typed error naming both the
expected and the actual length. (implements: RC-sxjnx9)
satisfies: derived

**REQ-9wwenn**: The software shall refuse, with a typed error, an Epoch whose
on-chain value does not fit the range this reader represents, rather than
truncating it to that range, and shall accept the largest value that does fit
whole. (implements: RC-sxjnx9)
satisfies: derived

## Provisional and committed observations

**REQ-ntn4ss**: The software shall report a Reorg notification, carrying both
the hash and the number of the head that was discarded, when and only when it
has already processed a head, the new best head's hash differs from that head's,
and the new best head is either at or below that head's height or at the next
height with a parent other than that head. (implements: RC-kemv75)
satisfies: derived

**REQ-gr2ver**: The software shall report nothing, and shall not advance the
head it has processed, for a best-head notification whose hash equals that of
the last head it processed. (implements: RC-kemv75)
satisfies: derived

**REQ-5zux82**: The software shall read, in ascending order, every height from
the one after the last head it processed up to and including the new head's —
and the new head's height alone when that height is at or below the last
processed head's, or when it has processed no head at all — and shall report a
Best-block observation for each `OrgRegistry` event it finds in that span that
REQ-9vwcwc and REQ-nygs7k admit to the subscriber, those two being the specific
rules this general one gives way to. (implements: RC-kemv75)
satisfies: derived

## What these requirements do not say

They do not state which block a state read taken without naming one is at. That
is org-node's expectation REQ-ysyu9g, addressed to this unit and due 2026-12-05,
and this change deliberately does not answer it: the risk file's section "The
obligation this change does not meet" records the confirmed fact — subxt 0.50.1
documents `at_current_block` as the current finalised block at
`subxt-0.50.1/src/client/online_client.rs:269`, and `get_org_state(_, None)`
calls it — and carries the obligation as a not-minted control rather than as a
requirement. Answering it means exporting a requirement here, which turns the
consumer's exempt expectation into an ordinary one needing a `verifies:` test on
the consumer's side at the same merge, and the only honest test of finality is a
chopsticks target — on a tool that finalises every block it mines immediately —
that no gate runs.

They do not state anything about how large an allocation an input can provoke.
REQ-sx5b6g said, until review round 1 (finding-4), that no decoder shall
"allocate from an unchecked length taken out of the input"; nothing in
`on-chain-client/src` implements or checks that. Every allocation made from a
length in the input is made inside `parity-scale-codec`, decoding the payload's
`data` and `topics` vectors, and the three bolero targets evidence
panic-freedom, not allocation size. The clause is gone, the dependency is
recorded in HAZ-95sc43's residual in the risk file, and a requirement about
allocation bounds would be written against a check this unit does not yet have.

They do not state that the contract at the configured address **is** the
`OrgRegistry` this decoder was written against. REQ-9vwcwc states the address
comparison, which is all the code does; a code-hash check or a construction-time
probe is a not-minted control.

They do not state anything about re-checking the Runtime spec version after
construction, about the depth of reorganisation the best lane can detect, or
about the length of its first backfill span. All three are defects under open
problem reports in
`on-chain-client/docs/problems/DRAFT-worktree-guardrails-on-chain-client-risk-on-chain-client-problems.md`,
and a requirement would be written against the fix, not against the defect.
REQ-hd6m9d and REQ-ntn4ss are worded to the decision each function actually
makes for exactly that reason.

REQ-hd6m9d was **not** so worded until review round 2 (finding-1), and the
correction is worth stating because it is the shape class C forbids. It used to
require that the software "decode Organisation state and events only through the
decoder compiled for the Runtime spec version the chain reported" — a property
this software does not have, and which PR-w5sk5k, open in this unit's own ledger,
says in terms it does not have: `from_client` resolves a decoder once
(`on-chain-client/src/client.rs:137-139`) and `get_org_state` decodes through
that stored decoder thereafter without ever comparing the block's
`spec_version()` against the pinned one. A requirement may not assert what an
open defect records as absent. What the software does decide is the
**resolution** — `dispatch::for_runtime`, which is what the requirement's tests
exercise — and that is what REQ-hd6m9d now states. The per-decode obligation
stays where it belongs: PR-w5sk5k, and not-minted control 4 in the risk file.

Neither REQ-ntn4ss nor REQ-5zux82 uses the word **extend** any longer, and the
removal is a correction rather than a polish. Both turned on it until review
round 4 (finding-1), which measured that they were using it in two incompatible
senses. Under the sense REQ-ntn4ss's own condition forces — an immediate child
carrying the right parent hash — a multi-height jump does not extend, so
REQ-5zux82's carve-out demanded the new head's height alone: narrowing `from` in
`internals::scan_step` to exactly that reading reds
`a_jump_backfills_every_skipped_height`, **13 passed, 1 failed**. Under the
other sense (`n > prev.number`), REQ-ntn4ss's second clause forbade a reorg
report the code makes and
`next_height_with_a_foreign_parent_reports_the_discarded_head` asserts. The term
is defined by neither glossary and is used in this sense nowhere else in the
repository, so it is eliminated rather than defined. REQ-5zux82 now names the
condition `scan_step` branches on (`on-chain-client/src/client.rs:557-560`:
`prev.number + 1` when the new height is above the last processed head's, and
the new head's height itself both when it is not and when no head has been
processed yet), and REQ-ntn4ss states its condition once and exhaustively,
having carried a second clause that settled nothing its first did not.

They no longer state, anywhere, **when** the admin comparison is made relative
to the contract comparison. That clause has moved twice and is now gone. Review
round 2 (finding-11) folded it out of REQ-nygs7k — which had been carrying a
delivery rule and an ordering rule together, the defect round 1's finding-12
split REQ-9vwcwc for — into REQ-9vwcwc, on the ground that the ordering is a
property of the contract check and that the test named for it,
`the_contract_check_dominates_a_matching_admin_filter` in
`on-chain-client/tests/log_ownership.rs`, was already annotated there. Review
round 3 (finding-3) **measured** that no test can gate the ordering at all:
`internals::log_is_ours` was rewritten to evaluate the admin filter first and
return the contract comparison last — the exact inversion — and
`--test log_ownership` reported **7 passed, 0 failed**. Ordering inside a total
boolean predicate is not observable from outside it, and this ledger may not
state what nothing can red. What that test does gate is the **consequence**, and
it reds honestly on it: a log from a foreign contract carrying a matching admin
is refused, so no other filter can admit a log from another contract. That
consequence is the clause REQ-9vwcwc keeps. The dominance is still true of the
code and is still what the test is named for; it is now recorded as a fact about
`on-chain-client/src/client.rs:508-520` in the risk file, not as a requirement.

They do not state that an observation taken from a best block is delivered as a
different kind of notification from one taken from a finalised block, and they
do not state that an unrecognised Runtime spec version is refused **at
construction, before any read is possible**. Both were clauses of minted
controls — RC-kemv75's first and RC-d7r82e's last — until review round 3
(finding-1) found that no requirement here realised either. Both are true of the
code: the best lane, through `events_in_best_block`, and `finalised_lane` pass
different `wrap` closures into `decode_contract_events`
(`on-chain-client/src/client.rs:616-618`, `:459-465`), which is the only
difference between the two lanes, and `from_client` resolves the decoder and
fails before returning a client (`:129-147`). Neither could acquire
a requirement here, because both live in `async` code that needs a live chain:
the only targets that reach them are the nine chopsticks/anvil ones and the
live-Paseo smoke test, none of which any gate runs, so a requirement written for
either would draw `MISSING-TEST` on the next run. A requirement with no possible
gated test is the shape this change exists to remove. The controls are narrowed
to what their requirements carry and each removed clause is recorded as a
not-minted control in the risk file instead.

And they do not state anything about verification. `on-chain-client/src/verify.rs`
is seven lines of doc-comment and no code; the module the design names as the
one that closes the loop with `org-members` contains nothing, which is a hazard
in prose and an open report, not a requirement.

They also do not cover every module of the unit. The hazard analysis was run
over the whole of `on-chain-client/src`, while the requirements above state only
the behaviour the nine controls that analysis chose call for — and all nine are
chain-free decisions. The async subxt code around them — `get_org_state`'s slot
loop and runtime-API call, the two subscription lanes' stream plumbing,
`from_client`'s construction — has no requirement here, because the only tests
that reach it are the nine chopsticks and anvil targets and the live-Paseo smoke
test, none of which any gate runs. Stating what that code shall do, and the
low-level requirements of the whole unit, is tooth 4 of
`docs/plans/2026-09-05-ratchet-gap-analysis.md`.
