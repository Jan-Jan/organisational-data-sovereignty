# Hazard analysis — the on-chain-client unit

on-chain-client's first hazard enumeration, run under IEC 62304 **Class C**
(`docs/adr/2026-09-05-units-and-per-unit-classes.md`) and evaluated against the
acceptability matrix in this ledger's README. It is the second half of tooth 3
of `docs/plans/2026-09-05-ratchet-gap-analysis.md`; the plan for the change is
`docs/plans/2026-09-10-on-chain-client-risk-analysis.md`. The first half, which
merged on 2026-09-09, is org-node's register
(`org-node/docs/risk/2026-09-09-org-node-hazards.md`), and this file follows its
form deliberately: the two units are the two halves of one question, and a
reader who has read one should not have to learn a second vocabulary to read the
other.

## Scope

The capability analysed is **the reader that decides what the chain says an
Organisation's membership is**: deriving an Organisation's identifier and its
storage key, reading the three slots that hold its Organisation state, decoding
those bytes, decoding the events the registry contract emits, deciding which of
those events belong to the Organisation being watched, and telling a consumer
which observations are provisional and which are committed. Concretely that is
every module of `on-chain-client/src`: `client.rs`, `decode/mod.rs`,
`decode/dispatch.rs`, `decode/v_paseo_ah.rs`, `h160.rs`, `state.rs`, `types.rs`
and `verify.rs`. `lib.rs` is module declarations, re-exports and the
feature-gated `test_support` re-export (`on-chain-client/src/lib.rs:37-46`), not
a module of the capability. `verify.rs` is in scope and is analysed **for what
it does not contain**: it is seven lines of doc-comment and no code
(`on-chain-client/src/verify.rs:1-7`), which is a hazard in prose below and an
open problem report.

The whole unit was surveyed before any item was written. So were the two things
outside it that this unit's correctness rests on and that are cited by file
throughout: `on-chain/src/OrgRegistry.sol`, the contract whose storage and
events this reader decodes, and `subxt-0.50.1/src/client/online_client.rs`, the
library call that decides which block a state read is taken at.

The hazard analysis and the requirements it mints do not have the same reach,
and the difference is deliberate. The analysis covers the unit. The twenty-one
requirements in
`on-chain-client/docs/requirements/2026-09-10-chain-reading.md`
realise only the nine controls this analysis chose, all of which are
**chain-free decisions**: the derivations, the decoders, the ownership test and
the best lane's step rule. The async subxt code that surrounds them —
`get_org_state`'s slot loop and runtime-API call, the two subscription lanes'
stream plumbing, `from_client`'s construction — acquires no requirement of its
own here, because no gated test reaches it (see "Where the evidence is, and
where it is not"). Stating what that code shall do, and the unit's low-level
requirements throughout, is tooth 4 of
`docs/plans/2026-09-05-ratchet-gap-analysis.md`.

Deliberately **outside** this analysis, and not to be read as assessed:

- **What a consumer does with a reading.** Whether a Membership root read from
  the chain is the right thing to verify a Change set against, and what happens
  when it is not, is the consumer's analysis. org-node's is
  `org-node/docs/risk/2026-09-09-org-node-hazards.md`; the app's follows this
  change.
- **The contract's own behaviour.** `on-chain/src/OrgRegistry.sol` is listed
  under `not_a_unit:` in `.guardrails/units.yaml` and is outside compliance.
  It is cited here as an external fact — twice as the reason a control of this
  unit is sound, which is exactly the thing ISO 14971 requires be recorded
  rather than assumed.
- **The membership rules.** What a Membership root commits to, and what makes a
  Change set applicable, is org-members' behaviour, analysed in
  `org-members/docs/risk/2026-09-02-membership-hazards.md`.
- **Writing to the chain.** This crate is read-only. Publishing a root is
  org-node's `chain_write/`.
- **What membership grants.** The CGKA/ACL layer of Phase 3, as in both
  sibling registers.

## Method, and what the harms are

The chain walked is ISO 14971's: hazard, hazardous situation, harm, severity,
probability, evaluation, controls, residual risk. Candidates were sought by
walking every input this crate has and asking, at each, what arrives, what is
checked before it is believed, and what a consumer is told when the check fails.
There are exactly three inputs, and every hazard below is on one of them: the
**event stream** (attacker-chosen bytes from any contract on the chain), the
**storage read** (bytes from a slot this unit computed the address of itself),
and the **runtime** (a version number, and a set of layout conventions that
change without asking). The survey that fed this is recorded in the plan.

The two harms are the two the class C ADR names and both sibling registers argue
(`docs/adr/2026-09-01-safety-class-c.md`): **disclosure** — someone reads
organisational material they should not, and in the journalism and government
deployments that material identifies a person — and **unavailability** — a
member cannot reach material at the moment a decision needs it. Both are **S3**
at their worst credible outcome.

This unit reaches both through one pathway, and it is worth stating once so that
six severity judgements below do not each have to re-argue it. **This crate
decides what the chain says the membership is.** It is the only thing in the
system that turns bytes in a contract's storage into an `OrgState`, and the only
thing that turns a log into an `Event`. Every consumer's access decision is
taken against a value this crate produced. So a wrong reading — a root that is
not the Organisation's root, an epoch that is not its epoch, an event that is
not its event — reaches the disclosure pathway directly: a consumer verifies a
Change set against a root an attacker chose, commits it, and a principal the
Organisation never admitted holds membership. And a reading that fails, or is
absent, or is stale reaches the unavailability pathway just as directly: a
consumer that cannot read the Organisation state cannot verify anything at all,
so no membership change can be committed on that device. That is the pathway
`docs/adr/2026-09-05-units-and-per-unit-classes.md` names when it puts this unit
at class C, and it is why **every hazard below is S3**.

As in both sibling registers, every probability below is the probability of the
**hazardous situation arising**; the situation-to-harm step is not estimated and
no number carries it. P2 is an ordinary event needing no deliberate act **by a
party whose cooperation the situation requires**; P1 needs such a deliberate
act, or a precondition that does not hold today. The party whose cooperation a
situation requires is the one who has to **decide to change something** before
the situation can arise at all — ship a revision, redeploy, reconfigure. Two
kinds of actor are therefore *not* such a party, and both exclusions are
load-bearing here:

- **An adversary is not cooperating.** This qualification was added at review
  round 1 (finding-8) because the scale as first written was being read two ways
  in one file. An attacker who deploys a contract and emits a log has certainly
  performed a deliberate act — but not one the *situation* depends on anybody
  cooperating with: nobody on the victim's side, and nobody who operates or
  ships this software, has to do anything for HAZ-werm85's situation to arise,
  and nothing anywhere had to be revised first. That is what makes it ordinary,
  and it is P2.
- **Neither is a system merely continuing to run.** Block production,
  reorganisation and the **release of a new runtime** are a chain doing what it
  ordinarily does, on its own cadence, whether or not anybody here acts and
  without anybody having decided to change anything this software depends on.
  This clause was added at review round 2 (finding-4), which found HAZ-8s5chy
  still arguing its P2 in the pre-amendment vocabulary — "no deliberate act by
  any attacker" — that the bullet above disowns, while the scale as round 1 left
  it did not say which way a runtime upgrade went.

What is P1 is a change **somebody has to decide to make** to a thing this reader
depends on: a redeployment, a misconfiguration, a contract revision, or an
upstream revision to a convention such as pallet-revive's `AccountId32` → H160
mapping. The situation cannot arise unless the party who ships or configures
that thing acts, and those entries are P1.

The line between the second exclusion and that last case is the one this scale
now turns on, so it is drawn explicitly. A runtime **release** is P2; a decision
to move pallet-revive's marker window, reorder the registry struct or rename one
of its events is P1 — even though a release is what would carry it. What makes
the two different is what each hazard's *situation* is. HAZ-8s5chy's situation
is a session decoding post-upgrade bytes through the decoder it pinned before
the upgrade, and that arises on **any** release at all; whether the release
actually moved a layout is the situation-to-harm step, which this file does not
estimate. HAZ-v2cmtx's situation is a derivation returning a well-formed key the
chain no longer uses, and that cannot arise until somebody has decided to change
the convention.

Every estimate in this file reproduces under the scale as it now stands, and
every one was re-checked against it entry by entry in round 2 — hazards, prose
hazards and the entries the controls introduce alike. Round 1 made the same
claim for the scale as it amended it and had missed one, which is finding-4
above; that is why the re-check is recorded as done rather than asserted as
obvious. One estimate has moved since the scale was written, for a different
reason, and it is argued where it stands: HAZ-xfg9cz, raised from P1 to P2 by
finding-2.

Severity is uniform and uncomfortable: every hazard below is S3, and the matrix
makes S3 unacceptable at every probability. That is the correct result for the
component that decides what the chain said, not a defect in the estimates; what
it means for acceptability is settled in the residual-risk section. In
particular, **no severity below is argued down from recoverability**. The class
C ADR withdrew that argument on 2026-09-02, and org-node's register had to have
two such arguments removed at review; neither is repeated here.

## Where the evidence is, and where it is not

This is the fact that shapes every residual assessment in the file, so it is
stated once, at the front, with its measurements.

`test_paths` for this unit is `on-chain-client/tests`. Before this change every
unit test in the crate lived in a `#[cfg(test)]` module inside
`on-chain-client/src` — twenty-three of them, all passing, and **not one visible
to `check-trace.sh`**. An annotation written there is read by no gate. This
change relocates all twenty-three into `on-chain-client/tests`, deleting each
`src` copy in the same commit that adds the relocated one, gives each a
`verifies:` annotation, and joins them with the abnormal-input cases class C
requires. Nothing was deleted and no behaviour of the crate was changed to fit a
requirement, with the single exception recorded under HAZ-werm85, which is a
defect fixed rather than a behaviour bent.

The crate has **twenty-one** integration targets. Twelve run at the gate: the
three bolero fuzz targets and the nine this change declares
(`on-chain-client/Cargo.toml`, each with `required-features = ["test-support"]`
so that naming one in the gate with the feature off is a cargo error rather than
a silent skip — the lesson tooth 1 learned in org-node). **Nine run nowhere**:
`00_chopsticks_sanity`, `01_multisig_sanity`, `off_chain_genesis_ceremony`,
`p_address_is_orgid`, `reorg_cancels_proposed`, `scenario_a_full` and
`two_orgs_one_watcher` need a chopsticks or anvil fork this repository's CI does
not start; `smoldot_smoke` is `#[ignore]`d and needs live Paseo; and
`regenerate_corpus` is an `#[ignore]`d corpus writer that needs no chain at all
and is excluded for that reason rather than for want of a fork. (**Corrected
2026-09-28 by the review sweep of the change
`worktree-guardrails-on-chain-client-arch`:** `regenerate_corpus` was listed
among the targets needing a fork. Nine run nowhere either way.)

(Twelve plus nine is the whole of it: `on-chain-client/tests/common`
and `on-chain-client/tests/fuzz_support` are shared modules the targets pull in
with `mod`, not cargo targets of their own. An earlier draft
of this paragraph counted the directory listing and said twenty-two, eleven and
nine — three figures that do not add up, which is what review round 1's
finding-10 caught.)

The eight of those nine that touch a chain at all are where every
chain-dependent property of this crate is tested (`regenerate_corpus` touches
none; corrected 2026-09-28 by the review sweep of the change
`worktree-guardrails-on-chain-client-arch`, which said "those nine"). So
the shape of this register is: **the chain-free decisions have gated evidence
and get controls; the chain-dependent properties get prose, residual statements
and not-minted controls that name the ungated target by file.** That is the
honest partition, and it is preferred to the alternative — minting a control
whose evidence no gate runs — which org-members' register calls "evidence that
exists and counts for nothing".

Two limits of the gated evidence itself, both measured during this change and
both carried into the residuals below rather than into a control:

- **clippy runs `--lib` only** (the workflow's `clippy` job lints
  `-p org-members --lib` and `--manifest-path on-chain-client/Cargo.toml
  --lib`). The crate root denies `unwrap_used`, `expect_used` and `panic`
  (`on-chain-client/Cargo.toml:96-99`), and `--lib` is where that denial is
  checked — which is why the `test-support` feature was deliberately kept out of
  the clippy job. The cost is that the nine new integration targets are outside
  the panic-freedom gate, and they do use `panic!` and `expect` freely, as the
  pre-existing targets beside them always have.
- **the crate is not `rustfmt`-clean at baseline**, independently of this
  change, and there is no `cargo fmt` step in the workflow. Recorded as a
  decision, not fixed here: making it clean would touch pre-existing lines
  across the crate under a change that is a hazard analysis.

### Nine claims held by review, not by a gate

A second partition cuts across the first, and it is collected here rather than
left implicit in nine places because by the end of this change it had nine
instances. A reader who meets them one at a time reads nine coincidences; they
are one shape. In each, this register relies on something that is **true
today**, that was established in this change **by reading** the code or the
contract, and that **no gate in this repository re-establishes**. What keeps
each of them true is review, and review is not a control. None is a defect; all
nine are recorded rather than fixed; each names the not-minted control that
would close it.

The list held five until review round 3, which added three: the sixth
(finding-2) was already argued at its own site and named by not-minted control 9
but had never been collected here, while the list asserted completeness at five;
the seventh and eighth (finding-1) are the two clauses narrowed out of RC-d7r82e
and RC-kemv75, each a property this register still leans on and no requirement
can state. Review round 4 (finding-2) added the ninth, on the same ground as the
sixth: the ownership decision's call site is argued under HAZ-werm85 and named
by not-minted control 13, and was collected nowhere while this list asserted
completeness at eight.

- **The contract's `ZeroValue()` revert and its atomic three-field write**
  (HAZ-xfg9cz). What makes this reader's `Ok(None)` mean "no such Organisation"
  is three lines of `on-chain/src/OrgRegistry.sol`, in a directory
  `.guardrails/units.yaml` disclaims, whose own tests for exactly those two
  reverts run in no lane here. Not-minted control 9.
- **The H160 mapping's ground truth** (HAZ-v2cmtx, RC-mugta4).
  `on-chain-client/tests/h160_mapping.rs` recomputes the mapping independently
  and does catch any change to `h160.rs`; what it cannot catch is the runtime's
  answer changing, and the one target that consults the runtime,
  `on-chain-client/tests/p_address_is_orgid.rs`, runs in no gate. Not-minted
  control 1.
- **The two event-signature literals** (HAZ-v2cmtx, RC-675a3h). The ABI-drift
  test recomputes both hashes from the canonical strings **as this repository
  writes them**, so it catches a constant that drifts from its own documented
  string and cannot catch a contract redeployed with a different declaration.
  Not-minted control 9.
- **The derivation `type_widths.rs` states its widths as** (HAZ-xfg9cz,
  RC-sxjnx9). `ABI_WORD`, `ABI_ADDRESS_OFFSET` and `ABI_EPOCH_OFFSET` are
  `const`s of the test file itself, fed by nothing in `src`, so the test's
  header claim that each width is "the ABI word minus the offset the decoder
  reads from" is mirrored from `v_paseo_ah.rs` by hand. Not-minted control 17.
- **The pinned `spec_version`'s value** (HAZ-8s5chy, RC-d7r82e). `2_002_002`
  (`on-chain-client/src/decode/v_paseo_ah.rs:45`) is the number the whole
  dispatch turns on, and its ground truth — captured from a chopsticks fork
  whose `state_getRuntimeVersion` reported `specVersion: 2002002` — lives only
  in the comment above it (`:40-44`). Every case in
  `on-chain-client/tests/runtime_version_dispatch.rs` derives its expectation
  from `PASEO_AH_SPEC_VERSION`, imported from `src` (`:22`), so the suite proves
  the dispatch **self-consistent** and cannot see the pin drift from the chain.
  Measured at review round 2 and re-measured at round 3 (finding-5): changing
  the constant to `3_003_003` — pinning the crate to a runtime that does not
  exist — left the full `verify_commands` line at **72 passed, 0 failed**. The consequence is fail-closed, so it lands on
  HAZ-8s5chy's total-outage side, not its wrong-reading side: a wrong pin makes
  every `from_client` refuse. Not-minted control 10, which is the only thing in
  this repository that would ask a real chain.
- **The `OrgRegistry` storage layout and the two event shapes** (HAZ-v2cmtx,
  HAZ-xfg9cz, RC-5ejucb, RC-6gfh8d). That `orgs` is the contract's only state
  variable and therefore sits at slot 0, that its `OrgState` struct puts
  `rootHash`, `orgPubKey` and `epoch` at `S`, `S+1`, `S+2`, and that
  `GenesisInitialized` gives two topics and 64 bytes of data while `RootUpdated`
  gives three and 96 — all of it was established in this change by reading
  `on-chain/src/OrgRegistry.sol` (`:8-12`, `:14`, `:20`, `:22-28`). Those are
  the figures RC-5ejucb's derivation and RC-6gfh8d's shape checks hard-code, and
  they are hard-coded, not read from the chain. `on-chain` is disclaimed under
  `not_a_unit:` in `.guardrails/units.yaml` and its forge tests run in no lane
  here, so a contract revision that inserted a state variable ahead of `orgs`,
  reordered the struct, or changed which parameters an event indexes would break
  every read and every decode this crate performs and **no gate here would go
  red**. Not-minted control 9, the same one the `ZeroValue()` revert names.
  Added at review round 3 (finding-2): the register argues this at the passage
  that establishes the layout, and control 9 already named the layout alongside
  the revert, but this list collected only the revert while asserting
  completeness at five.
- **That the refusal of an unrecognised runtime happens at construction**
  (HAZ-8s5chy, RC-d7r82e). `from_client` resolves the version and returns `Err`
  before a client exists (`on-chain-client/src/client.rs:129-147`, the refusal
  at `:138-139`), which is what makes the outage this control introduces a total
  one rather than a degraded mode — the argument under "Hazards introduced by
  these controls" turns on it. `from_client` is `async` and needs a live chain,
  so no gated test reaches the construction path; the fact is read, not checked.
  It was a clause of RC-d7r82e until review round 3 (finding-1) narrowed the
  control to what its requirement carries. Not-minted control 19.
- **That a best-block observation is delivered as a different kind of
  notification from a finalised one** (HAZ-xd4urb, RC-kemv75). The variants are
  distinct in the type (`on-chain-client/src/state.rs:62-70`) and the two lanes
  differ in exactly one thing, the `wrap` closure each passes into
  `decode_contract_events` (`on-chain-client/src/client.rs:616-618` for the best
  lane, through `events_in_best_block`, and `:459-465` for `finalised_lane`) —
  but which lane wraps which observation is settled only inside `async` code, and
  the one target that could observe a lane's output,
  `on-chain-client/tests/reorg_cancels_proposed.rs`, runs in no gate and could
  not be evidence about finality anyway, since chopsticks finalises every
  `dev_newBlock` immediately. It was RC-kemv75's opening clause until review
  round 3 (finding-1). Not-minted control 18, with control 10 behind it.
- **The wiring between the ownership decision and its call site**
  (HAZ-werm85, RC-5e3bdk). `internals::log_is_ours` is pinned by
  `on-chain-client/tests/log_ownership.rs` and
  `on-chain-client/tests/contract_address_filter.rs`, with the three mutation
  counts recorded under that hazard. What no gate reaches is the call site,
  `decode_contract_events` (`on-chain-client/src/client.rs:680-682`): that the
  address the predicate is handed is the one `from_client` was given, and that
  `continue` really drops the log rather than surfacing it to the subscriber as
  an error. That function is `async` and needs a live chain, and every
  chopsticks target deploys exactly one `OrgRegistry`, so no target in the crate
  — gated or not — has a second contract to spoof from. This register leans on
  this entry harder than on any other: RC-5e3bdk's whole **reduced** residual is
  the claim that a spoofed log is dropped, and were `decode_contract_events` not
  to reach `log_is_ours`, this unit's most probable hazard would be mitigated by
  nothing. Not-minted control 13. Collected here at review round 4 (finding-2);
  the fact itself has been argued under HAZ-werm85 since this change was
  written.

**The fourth of the nine costs something different from the other eight, and
saying so is the point of collecting them.** In the other eight — the contract's
guards, the mapping's ground truth, the signature literals, the pin's value, the
storage layout and event shapes, the construction-time refusal, the
best/finalised distinction and the ownership decision's call site — a
**property** this register relies on is ungated,
and the residual assessments
below carry that. In the fourth entry the property is gated: the decoder's
offsets are pinned by `on-chain-client/tests/decode_org_state.rs`, measured
rather than assumed under RC-sxjnx9, and what is ungated is only the *derivation
a comment asserts*. It is a documentation-accuracy point, not a coverage hole,
and it is in this list because the failure mode is the same one even where the
consequence is not: a reader takes a written claim for a checked one, and stops
looking.

**What the nine have in common is also the reason each was found late.** Every
one of them was invisible to `check-trace.sh` by construction — a gate can see
that a requirement has a test and that the test runs; it cannot see that the
test's *reason for being right* lives in a file no gate reads, and it cannot see
a **clause** of a control that no requirement realises, because it compares
items and not clauses. Eight of the nine were found by review rounds of this
change — three of them by round 3 and the ninth by round 4 — and one by a
negative control run
when a red could not otherwise be established. That is
the method this register recommends for the shape generally: where a claim is
load-bearing and its check is arithmetic over constants, mutate the thing the
claim is about and watch whether anything goes red — which is exactly how the
fifth was found. Round 3 adds a second method for the same shape, and it is what
found the seventh and eighth: read each control against its implementing
requirements **one clause at a time**. Round 4 adds a third, which found the
ninth and which the sixth would also have yielded: read every passage in this
register that says a fact is *established by reading* against this list's
criterion, and check that the list collects it.

A tenth relative is argued at its own site and is deliberately not counted here,
because it is a dependency rather than a claim: the panic-freedom
`parity-scale-codec` supplies to `parse_revive_event`, relied on and not
re-established, under HAZ-95sc43.

## Where the controls come from, and what they are not

Nine controls are minted below. Each is worded from behaviour the crate already
has — with one exception, RC-5e3bdk, whose behaviour did not exist and was
written in this change as the fix for PR-p5ngya — and **every clause of every
one of them is realised by a requirement in**
`on-chain-client/docs/requirements/2026-09-10-chain-reading.md`
carrying `(implements: RC-…)`, **with two exceptions, named here rather than
left for a reader to find, and neither of them a gap**.

The two are worth stating precisely, and they are not the same case. A clause
realised by a **gated test** rather than by a requirement is legitimate — the
first below is that, and nothing about it is missing. The second is realised by
neither, and is carried as an open defect and a not-minted control instead.
What is not legitimate is a register claiming a uniformity it does not have, and
that claim is what review round 4 (finding-6) removed.

- **RC-675a3h's closing clause** — *a drift between the two is detected by an
  independently recomputed comparison rather than silently changing which logs
  this reader treats as the Organisation's*. No requirement realises it and none
  could: it is a property of the **evidence**, not of the software. What
  realises it is the gated ABI-drift test
  `the_recognised_signature_hashes_are_the_keccak_of_the_canonical_solidity_strings`
  (`on-chain-client/tests/decode_revive_event.rs:174-175`), whose shape — it
  recomputes both hashes itself rather than comparing against the crate's
  constants — is what the clause asserts. Recorded at RC-675a3h below, in the
  paragraph beginning "The evidence is the ABI-drift test"; REQ-52uc8f states
  the binding the test checks.
- **RC-d7r82e's opening words** — *the decoder **a client reads through*** is
  resolved from the runtime `spec_version`. REQ-hd6m9d realises the resolution,
  which is what `dispatch::for_runtime` decides and what
  `on-chain-client/tests/runtime_version_dispatch.rs` gates. What no requirement
  states is the binding between that resolved decoder and the client that
  decodes through it, which lives in `from_client` and the `decoder` field
  (`on-chain-client/src/client.rs:114`, `:137-145`) and is read by
  `get_org_state` on every call — `async` code, reached by no gated test, the
  same place review round 3 narrowed this control's construction-time clause out
  of. It is recorded three times below and is not left to this paragraph: at the
  control itself, in HAZ-8s5chy's residual "the decoder is pinned once and never
  re-checked", and as **PR-w5sk5k** with not-minted control 4 against it.

Neither clause is deleted, because both are true and both matter: the first is
the reason RC-675a3h was minted at all, and the second is what makes RC-d7r82e a
control over reads rather than a note about a lookup table.

The clause-level claim is a stronger statement than this section made until
review round 3
(finding-1), and it is stated at clause level deliberately, because the weaker
one was hiding two gaps. `check-trace.sh` passes as soon as a control has *some*
implementing requirement; it has no view of clauses, so RC-kemv75 could open
with the best/finalised distinction and RC-d7r82e could end with the
construction-time refusal, and neither clause be stated by any requirement,
without a gate noticing. Round 3 read the controls against their requirements
one clause at a time and found exactly those two, plus an ordering clause in
RC-5e3bdk that round 3's finding-3 measured unobservable. All three are narrowed
out at the controls above, each removed clause is recorded as a not-minted
control (18, 19 and — for the ordering, which is a fact about the code rather
than work to be done — the paragraph at RC-5e3bdk itself). Review round 4
(finding-6) read the same pass again and found the two clauses above, which
narrowing is the wrong remedy for: neither overclaims, and removing either would
cost the register something true. The claim is softened instead, and what it now
asserts — every clause realised by a requirement, bar two realised otherwise and
named — is true clause by clause.

Four hazards below have the shape of a **defect** rather than only a missing
control — HAZ-werm85, HAZ-8s5chy, HAZ-xd4urb and the empty-verifier hazard
recorded in prose — and between them they carry **five** problem reports, all
filed in this unit's ledger
(`on-chain-client/docs/problems/2026-09-10-on-chain-client-problems.md`)
— HAZ-xd4urb carries two of the five, which is why the hazard count and the
report count differ and why review round 3 (finding-6) found this paragraph
subtracting one from the other. One report — events never filtered by their
emitting contract, while the code's own doc-comment claimed they were — is
**fixed by this change**, because the control was otherwise unimplementable and
the fix was small and directly gatable. The other **four reports** stay open:
fixing them would mix a hazard analysis with a change to `client.rs`'s async
paths, which is the same call org-node's half made and org-members' before it.

This unit is a **provider**. org-node and app depend on it
(declared in each consumer's own config — `org-node/.guardrails/config.yaml` and
`app/.guardrails/config.yaml`, not in `.guardrails/units.yaml`, whose schema
cannot carry an edge); it depends on nothing. One requirement of org-node's
is addressed to this unit as an expectation — REQ-ysyu9g, that a state read with
no block named returns the state at the latest Finalised block — and it has a
section of its own below, because this change deliberately does not meet it.
Everything else in a consumer's ledger is cited here **by file path and never by
identifier**, which is not a stylistic choice: `check-trace.sh` reports
`UNDECLARED-DEPENDENCY` for a provider that names a consumer's item, and the
org-node half of this tooth failed that gate three times learning it.

## Hazards, and the controls chosen for them

### A log another contract emitted, accepted as this Organisation's own

**HAZ-werm85**: the reader could accept a `ContractEmitted` log as an
`OrgRegistry` event of the Organisation it watches on the strength of the log's
signature hash and indexed admin alone, both of which any account may put in a
log of its own; an attacker deploys a contract on Asset Hub, emits a log whose
first topic is the `GenesisInitialized` or `RootUpdated` signature hash and
whose indexed admin is a victim Organisation's — both values are public
on-chain — and every subscriber of that Organisation receives it as a genuine
event carrying an attacker-chosen Membership root and epoch; a consumer forms
its belief about the Organisation's membership from a root the Organisation
never published, admits a principal it never admitted or believes a removal
that never happened, and material identifying a source or a protected person
reaches whoever the attacker's root names. Severity: S3. Probability: P2.

P2, and the argument is short because nothing has to go wrong for it. Deploying
a contract on Asset Hub is permissionless. The signature hashes are
`keccak256` of the contract's own public ABI strings. The indexed admin is the
H160 of a pure proxy, visible in every genuine event the Organisation has ever
emitted. There is no precondition beyond "an attacker who wants to", and no
deliberate act by anyone on the victim's side.

**RC-5e3bdk**: a decoded log is delivered to a subscriber only if the contract
that emitted it is the contract the reader was constructed for, and — where the
subscription named an Organisation to filter on — only if the event's admin is
that Organisation's; the emitting address is carried out of the decoder with
every decoded event rather than dropped, and the contract comparison is
decisive, so a matching admin can never rescue a log from a foreign contract.
mitigates: HAZ-werm85

**What this control no longer says**, and the reason is a measurement. It said,
until review round 3 (finding-3), that the contract comparison "is made
**before** the admin comparison". It is — `internals::log_is_ours` returns on the contract
mismatch before it looks at the filter (`on-chain-client/src/client.rs:513-519`)
— but that ordering is **not observable from outside the predicate**, and no
test can therefore gate it. The reviewer rewrote `log_is_ours` to evaluate the
admin filter first and return the contract comparison last, the exact inversion,
and `--test log_ownership` reported **7 passed, 0 failed**. What the suite does
gate is the *consequence*, and it reds honestly on that: deleting the contract
comparison outright reds exactly the three rejection cases —
`the_contract_check_dominates_a_matching_admin_filter` among them — which is the
measurement recorded below. So the control keeps the consequence — a matching
admin can never rescue a foreign log — and
drops the ordering, as REQ-9vwcwc does. Ordering inside a total boolean
predicate is a fact about the code, recorded in the paragraph below; it is not a
claim any evidence in this repository supports.

How it sits in the code, and what it replaced. `parse_revive_event` now binds
the payload's `contract` field (`on-chain-client/src/decode/v_paseo_ah.rs:98`)
and returns it paired with the decoded event as an `EmittedEvent`
(`on-chain-client/src/state.rs:45-54`, constructed at
`on-chain-client/src/decode/v_paseo_ah.rs:119`). The caller's decision is
`internals::log_is_ours` (`on-chain-client/src/client.rs:508-520`): the contract
comparison first and returning `false` on mismatch (`:513-515`), the admin
filter second and only for a log that already belongs to the right contract
(`:516-519`). `decode_contract_events` calls it once, for both subscription
lanes (`on-chain-client/src/client.rs:680-682`).

What was there before is the reason this hazard is first in the file. The
payload's contract field was decoded into `let _contract` and discarded, and the
function that was supposed to make this decision, `event_matches_contract`, was
`let _ = (ev, contract); true` — it returned `true` unconditionally, for every
log, from every contract. The `contract` field's own doc-comment said, and still
says, "Events from other contract addresses are filtered out before reaching
subscribers" (`on-chain-client/src/client.rs:108-109`); before this change that
sentence was false. Any contract could emit a log with the OrgRegistry signature
hash and a victim's indexed admin and every subscriber received it as genuine.
That is **PR-p5ngya**, and it is resolved by this change: the stub is deleted,
the address is carried, and the doc-comment is true.

Residual risk: reduced, **not acceptable**, and the reason is measured rather
than estimated.

The fix was landed first (task commit `62e1607`) with three tests over the
decoder's half — that the emitting address is surrendered rather than dropped,
and reported byte-for-byte. Then the prescribed red-by-mutation was run against
the caller's half: disable the `emitted.contract != contract` comparison and
watch the spoofing case fail. **It did not fail.** With the comparison switched
off, `--test contract_address_filter` still reported 3 passed. The reason is
structural: the comparison sat inside `async fn decode_contract_events`, which
needs a chain, so no gated test could reach it — and every one of the five
ungated targets that constructs an `OrgRegistryClient` at all deploys exactly
one OrgRegistry and hands its address to `from_client`, so not one of them has a
second contract to spoof from either. (**Corrected 2026-09-28 by the review
sweep of the change `worktree-guardrails-on-chain-client-arch`:** this said
"the nine chopsticks targets". Nine is the count of ungated targets; five is the
count that construct a client — `off_chain_genesis_ceremony`,
`p_address_is_orgid`, `reorg_cancels_proposed`, `scenario_a_full`,
`two_orgs_one_watcher` — and five is the number this argument needs.)
The change would have fixed the hole and left the fix's decisive line unguarded:
anyone could have deleted it and every gate would have stayed green.

That is the false-green shape this whole tooth exists to remove, so it got the
same remedy: the decision was extracted into `log_is_ours` (task commit
`0801aa1`) where a test can reach it without a chain, and the mutation is now
observable three ways — the comparison inverted reds 6 of 7 cases, the check
**deleted outright** reds exactly the three rejection cases and leaves every
acceptance case green, and the admin filter ignored reds 1.

What remains open is that **no test in this repository exercises the real
`decode_contract_events` path**. `log_is_ours` is pinned; the call site at
`on-chain-client/src/client.rs:680` is not, and the wiring between them —
that the address compared against is the one `from_client` was given, that
`continue` really means the log is dropped rather than surfaced as an error — is
established by reading. Closing that needs a chopsticks fixture with two
deployed contracts, which is not-minted control 13. It is also the **ninth entry
of "Nine claims held by review, not by a gate"** at the front of this file,
collected there at review round 4 (finding-2) — argued here since this change
was written, and until that round collected nowhere while the list asserted
completeness at eight. It is the entry this register leans on hardest: the
"reduced" verdict below is the claim that a spoofed log is dropped, and that
claim passes through this wiring.

Two smaller residuals of the same control. The check is on the address and
nothing else: a contract **at** the configured address is trusted entirely, with
no code-hash check and no interface probe, which is the trust-one-address hazard
in prose below. And a filtered log is `continue`, not an error — the consumer is
told nothing, ever, about a log that matched a signature and failed the address
check, which is exactly the observation that would reveal an attack in progress
and is not-minted control 12.

### A runtime whose layout this reader cannot decode, read anyway

**HAZ-8s5chy**: the shapes this reader decodes — pallet-revive's
`ContractEmitted` payload and the contract storage the `ReviveApi::get_storage`
runtime API returns — are set by a pallet that is pre-stable by this crate's own
account, and they can change under a runtime upgrade without the reader being
told; Paseo Asset Hub upgrades, as it does routinely, and a client constructed
before the upgrade keeps decoding post-upgrade bytes with the decoder it pinned
at construction; the reader returns a well-typed but wrong Organisation state —
a root that is not the root, an epoch that is not the epoch — or a well-typed
but wrong event, and the consumer's access decision follows it as if it were
what the chain said. Severity: S3. Probability: P2.

P2, and the argument is re-made here in the amended vocabulary because until
review round 2 (finding-4) it was made in the old one — "no deliberate act by
any attacker" — which the Method section explicitly disowns, and which the
amended scale would have answered the other way, since somebody certainly ships
each runtime.

The situation this estimate is of is the one the statement above names: **a
client keeps decoding post-upgrade bytes with the decoder it pinned at
construction.** That arises on any Paseo Asset Hub release at all, whether or
not the release moved a layout, because exactly one `spec_version` is compiled
in and nothing re-checks it (PR-w5sk5k, below). A release is the chain
continuing to run on its own cadence — the second of the Method section's two
exclusions — so no party whose cooperation the situation requires has to decide
to change anything for it to arise. That makes it ordinary, and ordinary is P2.
Whether a given release actually moves the `ContractEmitted` payload or the
`ReviveApi::get_storage` answer is the situation-to-harm step, which this file
does not estimate; that it *can*, without the reader being told, is
pallet-revive's own pre-stability, which is the stated reason
`on-chain-client/src/decode/mod.rs:8-10` gives for the decoders being
version-gated at all.

Note what this does **not** collapse into. HAZ-v2cmtx stays P1 on the same
scale, because its situation needs somebody to have decided to revise the
mapping convention itself, and a release alone does not produce it.

**RC-d7r82e**: the decoder a client reads through is resolved from the runtime
`spec_version` the chain reported, by exact match against the versions compiled
in; and a `spec_version` for which no decoder is compiled in is refused with a
typed error naming the version asked for, rather than served by the nearest
decoder or by a default. mitigates: HAZ-8s5chy

**What this control does not claim**, stated at the control rather than left to
the residual, because review round 2 (finding-1) found the claim being made and
this register may not make it. It does **not** say that state and events are
decoded only through a decoder compiled for the version the chain reported *at
the time of the decode*. The resolution happens once, and repeating it per read
is PR-w5sk5k's fix and not-minted control 4 — not part of what is implemented
here. REQ-hd6m9d is worded to the resolution for the same reason.

**And it no longer says *when* the refusal is made.** It ended, until review
round 3 (finding-1), with a third clause: *and the refusal is made at
construction, before any read is possible*. That is true of the code —
`from_client` resolves the version from the block it is constructed against and
returns `Err` before a `Self` exists (`on-chain-client/src/client.rs:129-147`,
the refusal at `:138-139`) — and it is load-bearing for this register's own
argument that the resulting outage is total rather than a degraded mode, under
"Hazards introduced by these controls". But **no requirement states it**, and
none can: `from_client` is `async` and needs a live chain, so no gated test
reaches the construction path at all, and a requirement written for it would be
`MISSING-TEST` on the next run. The clause is now **not-minted control 19**, and
the fact it asserts is the seventh entry of the review-held list at the front of
this file — true, read, and re-established by no gate.

How it sits in the code. `dispatch::for_runtime`
(`on-chain-client/src/decode/dispatch.rs:26-31`) is the single place a version
becomes a decoder: one match arm for `PASEO_AH_SPEC_VERSION`, and
`Err(DecodeError::UnsupportedRuntime { spec_version })` for everything else,
carrying the version that was asked about.  `from_client`
(`on-chain-client/src/client.rs:129-147`) resolves the version from the block
the client is constructed against (`:137`) and fails fast, before any read is
possible, if the dispatch refuses it (`:138-139`). The pinned version is
`2_002_002`, captured from a chopsticks fork
(`on-chain-client/src/decode/v_paseo_ah.rs:40-45`).

The evidence is `on-chain-client/tests/runtime_version_dispatch.rs`, and one
thing about it is worth stating because it is what makes the test worth having:
the positive case does not assert that *some* `&dyn Decoder` came back, it
asserts that the decoder which came back decodes a known-good 96-byte blob to a
known-good `OrgState`. That is the difference between pinning the dispatch and
pinning the dispatch's *answer*, and it needed its own mutation to red —
widening the match to accept everything reds the four refusal cases and leaves
the positive case green, so dropping the pinned arm was applied separately and
watched failing on `UnsupportedRuntime { spec_version: 2002002 }`.

What that suite cannot see is the pinned **number**. Every case in it, positive
and negative alike, derives its expectation from `PASEO_AH_SPEC_VERSION`
imported from `src` (`on-chain-client/tests/runtime_version_dispatch.rs:22`), so
it establishes that the dispatch is self-consistent and says nothing about
whether `2_002_002` is what Paseo Asset Hub reports. That is the fifth entry of
"Nine claims held by review, not by a gate", measured at review round 2, and it
is carried in this control's residual below.

Residual risk: reduced, **not acceptable**, on three counts, the first of them a
defect.

**The decoder is pinned once and never re-checked.** `from_client` resolves it
at construction and stores it (`on-chain-client/src/client.rs:114`, `:138-145`);
the field's own doc-comment says that "if the runtime upgrades mid-session the
client should be reconstructed" (`:111-113`), and **nothing enforces it**. There
is no reconstruction trigger, no periodic re-check, and no error a caller could
notice. Worse, the information needed is already in hand: `get_org_state`
resolves a block on every call (`:175-186`) and that block object carries a
`spec_version()` — the same accessor `from_client` used at `:137` — which is
never compared against the pinned one. So the control holds for exactly as long
as the session, and the failure it is supposed to prevent is precisely the one
that happens *during* a session. That is **PR-w5sk5k**, open, and its fix is
not-minted control 4.

**Refusal is total, and that is a cost this register does not hide.** One
`spec_version` is compiled in. On the day Paseo AH upgrades, every
`from_client` fails and the crate stops reading anything at all until a decoder
module and a match arm are added and shipped. Fail-closed is the correct
direction for a decoder — guessing a layout is how the wrong-reading half of
this hazard happens — but the unavailability is real, ordinary and total, and it
is assessed as a hazard the control introduces, below.

**And the pinned number's *value* is held by review.** Everything above turns on
`2_002_002` being the `spec_version` Paseo Asset Hub actually reports, and
nothing in this repository re-establishes that: the dispatch's whole test suite
derives its expectations from the constant itself, so the pin can drift from the
chain without a single case going red. Measured at review round 2, and collected
as the fifth entry of "Nine claims held by review, not by a gate" at the front
of this file. It lands on this hazard's **total-outage** side rather than its
wrong-reading side, because the failure is fail-closed: a pin that does not
match the chain makes every `from_client` refuse, which is the outage entry
already assessed S3/P2 under "Hazards introduced by these controls", not a
silently wrong decode. Closing it needs a gated target that asks a real chain —
not-minted control 10.

### Hostile or malformed chain bytes crash or mislead the reader

**HAZ-95sc43**: every byte this crate decodes is chosen by someone else — an
event payload by whichever contract emitted the log, a storage blob by whatever
the runtime API returned for a slot address this crate computed — and a decoder
that trusts a length, a count or a padding it did not check is reading past the
end of the input or into the wrong field; a log arrives whose topic count, data
length, indexed-address padding or SCALE framing does not match the signature it
claims, or a storage read returns a blob that is not 96 bytes; the reader either
panics inside the decoder, taking the consumer's whole chain view down at the
moment a membership decision needs it, or decodes the wrong bytes into a
right-looking `Event` or `OrgState` that the consumer then believes. Severity:
S3. Probability: P2.

P2: the input is attacker-chosen by construction on the event path, since any
contract can emit a log onto the stream this crate follows, and the surface is a
decoder reachable by anything that can send one transaction.

**RC-6gfh8d**: a log whose topic count, data length or indexed-address padding
does not match the signature its first topic claims is refused with a typed
error naming both the expected and the actual value; a first topic matching
no signature this decoder knows yields no event rather than an error, so that a
malformed log is distinguishable from a log that is simply not ours; and a log
that *is* well formed decodes to every field it carried — the emitting address
included — with nothing lost, reordered or silently defaulted, so that accepting
a log and reading it correctly are one property and not two.
mitigates: HAZ-95sc43

The last clause is the **inversion** property, and it was added at review round 1
(finding-7b). The shape checks above say what is refused; without the inversion
they say nothing about what an accepted log becomes, and a decoder reading the
right shape out of the wrong offsets would satisfy every one of them. It is
REQ-n6v896, evidenced by `fuzz_event_round_trip`, which generates an event,
encodes it canonically and compares the decode against the original. That target
had been annotated to REQ-sx5b6g — the arbitrary-bytes property it never
exercises, since it only ever feeds the decoder encodings it built itself — in
contradiction of its own header.

That is also why REQ-n6v896 carries **no abnormal-input case, by exemption**,
recorded at review round 2 (finding-5). Its generator produces only structurally
valid events by construction, which is the whole of what makes it an inversion
test: feed it a malformed event and it is testing something else. Abnormal input
to the same decoder is what REQ-88fp2h, REQ-twdu84, REQ-axcxf7 and REQ-sx5b6g
state and evidence beside it, so nothing is left uncovered — the case belongs to
its siblings, and none was invented here to fill a column.

**RC-8w9wtp**: any sequence of bytes offered to a decoder of this crate yields
a typed `Ok` or a typed `Err`, and never a panic or an abort.
mitigates: HAZ-95sc43

That wording is narrower than the one this register carried until review round 1
(finding-4), and deliberately so. It used to add "or an allocation sized from an
unchecked length in the input", which **this unit implements no check for**:
every allocation made from a length taken out of an input is made inside
`parity-scale-codec`, and the three bolero targets evidence panic-freedom, not
allocation size. The clause is dropped here and from REQ-sx5b6g, and the
dependency it was silently standing on is recorded in the residual below.

How the two sit in the code. `parse_genesis` checks topic count and data length
before touching either (`on-chain-client/src/decode/v_paseo_ah.rs:137-150`), and
`parse_root_updated` the same (`:164-177`); each error variant carries both the
expected and the actual figure (`on-chain-client/src/decode/mod.rs:78-91`).
`unpack_address_topic` requires all twelve padding bytes of an indexed address
topic to be zero (`on-chain-client/src/decode/v_paseo_ah.rs:195-202`). The SCALE
framing is closed at both ends: a payload with bytes left over after `contract`,
`data` and `topics` is refused (`:104-109`), and a payload that ends inside one
of them fails in `parity_scale_codec` and is mapped to `DecodeError::Scale`
(`:98-103`). A log whose first topic matches neither signature returns
`Ok(None)` (`:117`), and so does a log with no topics at all (`:111-113`).
Panic-freedom is denied at the crate root — `unwrap_used`, `expect_used` and
`panic` (`on-chain-client/Cargo.toml:96-99`) — and that denial is checked by the
workflow's `clippy` job over `--lib`.

The behavioural evidence is `on-chain-client/tests/decode_revive_event.rs`
(sixteen cases) and `on-chain-client/tests/decode_org_state.rs` (nine), plus the
three bolero targets `fuzz_parse_revive_event`, `fuzz_decode_org_state` and
`fuzz_event_round_trip`, which have run at every merge since 2026-08-26 through
this unit's `verify_commands` and which this change annotates so that
`check-trace.sh` credits them at all.

**One measurement belongs in this hazard's block rather than in a verification
record, because it is the argument for the control.** The plan named one
mutation per guard: narrow `topics.len() != 2` to `< 2`, narrow
`bytes.len() != 96` to `< 96`. Both were run, both redded a case — and both are
the *too-many* side. The mirrors were applied as well, and they behave
differently in kind. `topics.len() != 2` narrowed to `> 2` did not produce a
wrong answer: it **panicked inside `src`**, `index out of bounds: the len is 1
but the index is 1`, at the first topic index the widened guard lets through —
`unpack_address_topic(&topics[1])`,
`on-chain-client/src/decode/v_paseo_ah.rs:151` — and the `!= 3` mirror the same
way at `:178-179`. `bytes.len() != 96` widened to `> 96` panicked at three
separate unguarded fixed-width reads (`:77`, `:79`, `:80`), and the input that
reached them was the most ordinary abnormal input of the whole set — the
**empty blob** a storage read returns when it returns nothing.

So the guards these two controls describe are not a politeness about error
messages. They are the only thing standing between untrusted chain bytes and a
panic, in a `no_std` library whose crate root denies `panic`, whose panic would
propagate into whatever process the consumer is — and the too-few side, which no
plan-named mutation reaches, is the side where the failure is a crash rather
than a wrong answer.

Residual risk: reduced, **not acceptable**, on five counts.

**Part of what RC-8w9wtp promises is implemented outside this unit, and this
register gives that the same treatment it gives `OrgRegistry.sol` under
HAZ-xfg9cz.** The SCALE framing at the front of every event payload is not
parsed here. `parse_revive_event` hands the bytes to `parity-scale-codec`
(`parity-scale-codec = "3"`, `on-chain-client/Cargo.toml:42`) three times —
`contract`, then `data: Vec<u8>`, then `topics: Vec<[u8; 32]>`
(`on-chain-client/src/decode/v_paseo_ah.rs:98-103`) — and the last two are
vectors whose lengths come from compact prefixes an attacker wrote into the log.
Nothing in `src` inspects those prefixes before the decode; what `src` checks is
what comes back. So on that path the "never a panic or an abort" this control
claims is `parity-scale-codec`'s panic-freedom, relied on and not
re-established, and the three bolero targets evidence exactly what they exercise
— that no input reached in one second of generated bytes panicked — and nothing
at all about how large an allocation a hostile length prefix can provoke.

It is a SOUP item, and it is recorded **nowhere else**:
`on-chain-client/docs/architecture/soup.md` is still the empty template, so this
unit's SOUP inventory names neither `parity-scale-codec`, nor its version, nor
what this register relies on it for. Filling that template is tooth 4 of
`docs/plans/2026-09-05-ratchet-gap-analysis.md` and an open item on
`docs/plans/2026-09-05-ratchet-setup.md`; this change does not touch it, and the
gap is pre-existing rather than introduced here. Until it is filled, the
paragraph above is the only place in this unit's documents where the dependency
is written down. ISO 14971 permits a control whose implementation lies outside
the software under analysis; what it does not permit is leaving it unrecorded,
which is the standard this register sets for itself under HAZ-xfg9cz and now
meets on its own decoder.

**The fuzzing is a smoke depth, and the figures say so.** All three targets are
`harness = false` bolero targets run at their default engine, which is a
**wall-clock budget, not a case count**: every run in this change's measurements
ended `exit reason: max duration (1s - default) exceeded`, and two runs of the
same target differed by 40% (`fuzz_decode_org_state` 182,624 then 261,916 rng
inputs; `fuzz_parse_revive_event` 207,437 then 241,990; `fuzz_event_round_trip`
23,404 then 39,337). A green run reports run time, iterations per second, corpus
inputs, rng inputs and an exit reason, and **never a pass count**; green means
the process exhausted its second without panicking or aborting. libFuzzer depth
is a separate explicit invocation that no lane in this repository runs.

**One of the three targets has an empty seed corpus.**
`fuzz_decode_org_state` has four seeds and `fuzz_parse_revive_event` six;
`fuzz_event_round_trip`'s corpus directory holds only a `.gitkeep`. That is
visible in the run line itself, which carries no `corpus inputs:` field where
the other two report 4 and 6 — and it has been visible in measured output since
the 2026-08-26 verification record **without ever being written down as a gap**,
which is the same shape org-node's register found on its own side. It is not
neglect of the other two's kind: `tests/regenerate_corpus.rs` seeds "the two
raw-byte targets" only, and a file dropped into the third would be consumed as
`TypeGenerator` driver bytes rather than as a payload. Seeding it is a different
job, and it is not-minted control 2.

**The panic-freedom denial covers `--lib` only.** The nine integration targets
this change adds are outside it, and use `panic!` and `expect` as the targets
beside them always have. A regression inside `src` would still be caught; a
regression in what the tests themselves assert would not.

**Every `crashes/` directory holds only a `.gitkeep`.** No reproducer has ever
been found and committed. That is consistent with the code being panic-free at
the depths reached; it is not evidence that it is panic-free.

### The Organisation's slot computed differently from the one the chain keeps

**HAZ-v2cmtx**: what this reader reads, and which logs it reads at all, is not
told to it by anyone — it is derived offline, in this crate, from three
conventions this crate does not own: pallet-revive's `AccountId32` → H160
mapping, which decides what an Organisation's on-chain identifier even is;
Solidity's mapping-slot formula with consecutive struct slots, which decides
where that identifier's state lives; and the `OrgRegistry` ABI's two event
signature strings, whose `keccak256` decides which logs on the whole chain this
reader will even look at; any one of the three changes upstream — a
pallet-revive revision moving or re-meaning the twelve-byte EVM-fallback marker,
a contract revision adding a state variable ahead of `orgs` or reordering the
struct, a contract revision renaming an event or changing a parameter type — and
the derivation keeps returning a perfectly well-formed 32-byte key, or a
perfectly well-formed 32-byte signature hash, that is no longer the one the
chain uses; the reader then reads a slot that belongs to a different
Organisation and returns its state as this one's, or reads a slot that belongs
to nobody and reports that the Organisation does not exist, or matches no log
the registry now emits and reports an Organisation whose membership never
changes. Severity: S3. Probability: P1.

The third convention was added to this statement at review round 1 (finding-7):
the ABI binding was pinned by a test but stated by no item, and it is the same
shape as the other two — a value computed offline from something a contract
revision can move. It gets its own control, RC-675a3h, below.

P1: it needs an upstream change — a pallet-revive mapping revision, a contract
storage-layout change, or a revision to the two event signatures the contract
declares — and each is a deliberate act by the party whose cooperation the
situation requires, namely whoever ships the next version of the pallet or of
the contract, rather than an ordinary event on the timescale of a session. That
is the P1 side of the line the Method section draws, and it is worth saying
which side of it this hazard sits on and why, since HAZ-8s5chy is P2 on the same
scale: what this situation needs is not a *release* but a **decision to change a
convention** carried in one. A release that moved nothing here would not produce
it. P1 is
the **lowest** rung this project's scale offers — P1 improbable, P2 occasional,
P3 frequent (`on-chain-client/docs/risk/README.md:42`) — so the argument here is
not that the estimate could have been lower and was not; it is that P1 is not
being used as a synonym for "will not happen". Pallet-revive is pre-stable and
the byte position of the marker **has already changed across versions**
(`on-chain-client/src/h160.rs:16-17`). This is a thing that has happened, not a
thing that might, and at S3 the rung buys nothing either way — the README says
so in terms. (An earlier draft argued "P1 rather than P0"; there is no P0 on
this scale, which review round 1's finding-11 caught.)

**RC-mugta4**: an Organisation's identifier is derived from an `AccountId32` by
exactly one of pallet-revive's two documented cases — the reverse case when all
twelve marker bytes are the EVM-fallback marker, the forward case otherwise —
with the choice made on the full twelve-byte marker window and never on a
prefix of it. mitigates: HAZ-v2cmtx

**RC-5ejucb**: the storage key for an Organisation's slot is derived by the
Solidity mapping formula over the left-padded admin address and the map's
declared slot index, and the struct's successive fields are read at successive
slot keys derived by big-endian increment of that key, with carry.
mitigates: HAZ-v2cmtx

**RC-675a3h**: the decoder recognises exactly the two Event signatures the
deployed `OrgRegistry` declares and no others, each being the `keccak256` of
that event's canonical Solidity signature string, so that a first topic is
matched against a value derived from the contract's ABI rather than against a
constant of unstated provenance — and a drift between the two is detected by an
independently recomputed comparison rather than silently changing which logs
this reader treats as the Organisation's. mitigates: HAZ-v2cmtx

How it sits in the code. The two constants are byte arrays in `src`, each
documented with the canonical string it is the hash of —
`keccak256("GenesisInitialized(address,bytes32,bytes32)")` at
`on-chain-client/src/decode/v_paseo_ah.rs:47-54` and
`keccak256("RootUpdated(address,uint256,bytes32,bytes32,bytes32)")` at `:56-60`
— and they are the *only* two values a first topic is matched against
(`:115-116`), everything else falling to the `Ok(None)` arm at `:117`. The
strings themselves are the contract's, at `on-chain/src/OrgRegistry.sol:20` and
`:22-28`.

The evidence is the ABI-drift test
`the_recognised_signature_hashes_are_the_keccak_of_the_canonical_solidity_strings`
in `on-chain-client/tests/decode_revive_event.rs:174-175`, and the shape of it is
what makes it a check on the binding rather than a restatement of it. The two
constants are private to the crate, so the test cannot compare against them and
does not try. It recomputes both hashes itself, keccaking the canonical strings
written out in the test file, cross-checks that derivation against
`fuzz_support`'s independent one
(`on-chain-client/tests/fuzz_support/mod.rs:33`, `:37`), and then asks the
*decoder*: a log whose first topic is the keccak of each canonical string must
decode, and a log whose first topic is the keccak of a plausibly **drifted**
string — one parameter dropped from each event — must yield no event. So a
constant edited in `src`, or a Solidity declaration that moves without the
constant moving with it, reds this test before any decoder logic gets a chance
to mismatch real events silently. Its requirement is REQ-52uc8f.

This control was minted at review round 1 (finding-7a). The behaviour existed
and was tested; what it lacked was an item. The test carried
`verifies: REQ-wnjz9j`, which states only the negative — an unknown first topic
yields nothing — and never says which signatures are known, so the register
asserted no positive ABI binding anywhere and a change to either constant would
have contradicted no written requirement.

Residual, specific to this control and not repeated below: the recomputation is
from the strings **as this repository holds them**, so it catches a constant
that drifts from its own documented string and cannot catch a contract that is
redeployed with a different event declaration. That is the same ungated-contract
gap HAZ-xfg9cz records and is covered by not-minted control 9.

How the other two sit in the code. `h160_of` (`on-chain-client/src/h160.rs:36-48`)
tests all twelve bytes of `account_id_32[20..32]` against `0xEE` (`:37`) and, on
a match, returns the first twenty bytes unchanged (`:38-40`); otherwise it
keccaks the full thirty-two and takes the last twenty (`:42-47`).
`solidity_mapping_slot` (`on-chain-client/src/client.rs:570-581`) left-pads the
admin into bytes `[12..32]` of a 64-byte buffer (`:573`), writes the map index
big-endian into the second word's low eight bytes (`:575`), and keccaks the
whole 64 (`:576-579`). `increment_slot` (`:587-597`) adds an offset to a
32-byte big-endian value from the least-significant end with carry.
`get_org_state` uses both, deriving the base key once and stepping it three
times (`:188-193`).

**The layout is confirmed against the contract, not assumed.** `orgs` is
`OrgRegistry`'s only state variable and is `private`
(`on-chain/src/OrgRegistry.sol:14`), so it sits at slot 0 — which is why
`solidity_mapping_slot(admin, 0)` is right — and has no getter, which is why the
slot has to be read directly at all rather than through a call. It is a
`mapping(address => OrgState)` over a three-field struct
(`on-chain/src/OrgRegistry.sol:8-12`), so `rootHash`, `orgPubKey` and `epoch`
occupy `S`, `S+1`, `S+2`, which is what the three `increment_slot` steps read.
The contract keys each Organisation on `msg.sender`
(`on-chain/src/OrgRegistry.sol:32`), so no caller can write another
Organisation's slot. The event shapes match too: `GenesisInitialized` indexes
only `admin` (`:20`), giving two topics and 64 bytes of data, and `RootUpdated`
indexes `admin` and `epoch` (`:22-28`), giving three topics and 96 bytes —
exactly what `parse_genesis` and `parse_root_updated` require.

The evidence is `on-chain-client/tests/h160_mapping.rs` (five cases) and
`on-chain-client/tests/storage_slot_layout.rs` (eleven), each recomputing its
expectation independently in the test rather than importing a constant from
`src`.

Residual risk: reduced, **not acceptable**, on three counts.

**The mapping is pinned against our reading of pallet-revive, not against
pallet-revive.** `h160_mapping.rs` recomputes `keccak256(account)[12..32]` and
the twenty-byte reverse path itself and compares — an independent derivation, so
it catches a change to `h160.rs`, and it caught four distinct mutations of the
marker window. What it cannot catch is the case this hazard is about: the
runtime's answer changing. The only test that consults the runtime is
`on-chain-client/tests/p_address_is_orgid.rs`, which takes the admin out of a
real `GenesisInitialized` event on a chopsticks fork and compares it with
`h160_of`, and it **runs in no gate**. That is not-minted control 1, and it is
the single most valuable ungated test in the crate.

**The contract half is confirmed by reading a disclaimed file.** Everything in
the paragraph above about `OrgRegistry.sol` was established by reading it in
this change. `on-chain` is listed under `not_a_unit:` in
`.guardrails/units.yaml` and is outside compliance; its forge tests run in no
lane of this repository, whose workflow directory holds a Rust workflow and a
Quint workflow and mentions `forge` in neither. So a contract revision that
inserted a state variable ahead of `orgs`, or reordered the struct, would break
every read this crate performs and **no gate here would go red**. Not-minted
control 9, and the sixth entry of "Nine claims held by review, not by a gate"
at the front of this file — added there at review round 3 (finding-2), which
found this passage making the argument while the list collected only the
contract's `ZeroValue()` revert.

**One narrowing, recorded rather than fixed.** `solidity_mapping_slot` takes the
map index as a `u64` (`on-chain-client/src/client.rs:570`) and places it in the
second word's low eight bytes (`:575`), where Solidity's mapping index is a
`uint256`. Not a live limit — `orgs` is at slot 0 — but indices above `u64::MAX`
are unrepresentable, and the shape of the mistake it invites (writing the index
at `[32..40]` instead of `[56..64]`) is what the `u64::MAX` case in
`storage_slot_layout.rs` exists to pin.

### A partial or wrong-width storage read decoded as an Organisation state

**HAZ-xfg9cz**: EVM storage cannot distinguish a slot that was never written
from a slot holding zero, and this reader treats any one of the three slots
reading absent as proof that the Organisation does not exist — returning
`Ok(None)` and stopping, without reading the other two; a slot of an initialised
Organisation reads absent, or a slot returns bytes of a width the decoder did
not expect, whether because a contract revision permitted a zero field, because
a write was not atomic, or because the runtime API's answer changed shape; the
reader tells the consumer there is no such Organisation — so every verification
that consumer performs fails and no membership change can be committed on that
device — or decodes a wrong-width blob into an `OrgState` whose fields are
offset from the values the chain holds, which the consumer then acts on.
Severity: S3. Probability: P2.

P2, and the estimate is the **maximum over the three routes** the situation
above names, because the probability of a disjunction is the probability of its
likeliest disjunct and not the average of them.

Two of the three routes are P1, and they are the two the first draft of this
paragraph argued. A contract revision that permitted a legitimately-zero field,
and a write that was not atomic, are both forbidden by
`on-chain/src/OrgRegistry.sol` today, and each would need a deliberate revision
by the party whose cooperation the situation requires — whoever ships the next
contract. Improbable, on this project's scale, is the right word for those.

The third route is not, and dropping it silently is what review round 1's
finding-2 caught. **The runtime API's answer changing shape** reaches this
hazard through the very situation HAZ-8s5chy names: a client reading
post-upgrade bytes through the decoder it pinned before the upgrade. That
situation arises on any Paseo Asset Hub release at all, and a release is — under
the second of the Method section's two exclusions — a chain continuing to run,
not a party deciding to change something whose cooperation the situation
requires. So this route is ordinary, and that is not a new judgement to make
here: HAZ-8s5chy assesses the same situation, in this same file, at **P2**. One
situation cannot carry two probabilities in one register, so this hazard takes
the higher: **P2**. (Round 1 argued that identity in the pre-amendment
vocabulary — "needing no deliberate act by anybody at all". It is re-argued here
in the scale's own terms at review round 2, finding-4, and comes out the same
way.)

Raising it changes nothing downstream and is recorded because it is the
conservative direction, not because it moves a verdict. The matrix makes S3
unacceptable at P1 and at P2 alike, so the residual below, the conclusion, and
the wording of RC-sxjnx9 are all unaffected. What changes is that the file now
says the same thing about the same event in both of the places it says it.

**RC-sxjnx9**: a storage blob is decoded into an Organisation state only at
exactly the expected width, refused with a typed error naming both the expected
and the actual length otherwise, an epoch whose value does not fit the
representable range is refused rather than truncated to it, and the public
newtypes those bytes are decoded into hold exactly the widths the contract's
ABI gives them. mitigates: HAZ-xfg9cz

How it sits in the code, at three levels. The widths first, because the other
two are written against them: `OrgAdmin` wraps `[u8; 20]`, `OnChainRootHash` and
`OrgPubKey` `[u8; 32]`, and `Epoch` a `u64`
(`on-chain-client/src/types.rs:19`, `:23`, `:37`, `:42`) — 20, 32, 32 and 8,
which is why a three-slot blob is 96 bytes and why every fixed-width
`copy_from_slice` in the
state decoder is the length it is. (Line numbers re-resolved 2026-10-05: the
`OrgPubKey` and `Epoch` lines were `:29` and `:34` before the doc comment on
`OrgPubKey` grew on 2026-10-05.) That is REQ-2qa5r5, and its evidence is
`on-chain-client/tests/type_widths.rs`, relocated in this change's fix round
out of a `#[cfg(test)]` module in `types.rs` where no gate could read its
annotation.
*(Note 2026-10-05, review round 3, finding-13, change
`worktree-person-shared-types`: REQ-2qa5r5 is superseded by REQ-54txzh
(`2026-10-05-organisation-public-key.md`), which states the same widths and
calls the thirty-two-byte key field the Organisation public key. What is
said here of REQ-2qa5r5 holds for REQ-54txzh, which carries this exemption
and this evidence; REQ-2qa5r5 is named as the record of the earlier text. The same holds for the
next paragraph's exemption.)*

**What that test gates, and what it does not — both measured.** It gates the
four widths, and it reds honestly on each of them. The obvious mutation
(`Epoch(pub u64)` → `u32`) fails to compile the *library*, so the named test
never runs and observes nothing, which is not evidence; the mutation that keeps
the crate compiling is `#[repr(align(N))]` on the newtype, widening `size_of`
past the ABI field without changing the inner array's type. Applied once per
width it produced `left: 32 / right: 20` on `OrgAdmin`, `64 / 32` on
`OnChainRootHash`, `64 / 32` on `OrgPubKey` and `16 / 8` on `Epoch`, with
`epoch_display_is_the_inner_value` green throughout. There is no abnormal-input
case beside those four and there is not meant to be: REQ-2qa5r5 constrains a
declaration, so it has no runtime input domain — the only thing there is to
malform is the type itself, which is what the four mutations above do. That
exemption is recorded at review round 2 (finding-5), which found this the one
requirement of twenty-one with neither a case nor a reason. What the test does
**not**
gate is the correspondence its own header asserts between those widths and the
offsets the decoder actually reads from: `ABI_WORD`, `ABI_ADDRESS_OFFSET` and
`ABI_EPOCH_OFFSET` are `const`s declared inside the test file
(`on-chain-client/tests/type_widths.rs:106-116`; the citation read `:73-83`
until review round 5, and had been stale since the file gained its header), and
nothing in `src` feeds them, so drift in the decoder's offsets cannot red this
target. That was established by negative control rather than inferred from
reading: shifting both regions in
`on-chain-client/src/decode/v_paseo_ah.rs` — `unpack_address_topic` from
`topic[..12]`/`topic[12..32]` to `topic[..8]`/`topic[8..28]`, and
`decode_uint256_to_u64` from `bytes[..24]`/`bytes[24..32]` to
`bytes[..16]`/`bytes[16..24]`, each keeping its width so the crate still
compiles — left `type_widths` **green, 2 passed**.

*(Note 2026-10-05, review round 3, finding-13: the REQ-2qa5r5 of this
paragraph is superseded by REQ-54txzh, which the same test verifies; see the
note above.)*

**That is a documentation-accuracy point and not a coverage hole**, and the same
negative control is what establishes the difference rather than asserting it:
the identical mutation immediately reds
`on-chain-client/tests/decode_org_state.rs` at **4 failed of 9**. The decoder's
offsets are gated — elsewhere, and by a target that decodes real blobs rather
than by an arithmetic identity over two constants of the test's own. So the
accurate statement, recorded here rather than left in a comment that overclaims:
`type_widths` gates the **newtypes' widths**, and its derivation of them from
the decoder's regions is a record of `v_paseo_ah.rs` mirrored **by hand**, kept
in step by review and not by the compiler. The test file's header is corrected
in this change to say exactly that. Making it a real coupling would mean
exporting the offsets from `src`, which is a change to production code that a
hazard analysis should not smuggle in, so it is proposed as not-minted control
17 instead. This is the fourth instance of the shape collected at the front of
this file under "Nine claims held by review, not by a gate", and the one whose
consequence is smallest.

Then, at two levels of the read itself. Per slot, `get_org_state` accepts a slot
value only at exactly 32 bytes (`on-chain-client/src/client.rs:195`) and
converts any other width into `StorageLengthMismatch { expected: 32, actual }`
(`:200-205`). Over the assembled blob, `decode_org_state` requires exactly 96
(`on-chain-client/src/decode/v_paseo_ah.rs:70-75`) before it copies anything,
and `decode_uint256_to_u64` requires the high 24 bytes of the epoch slot to be
zero, returning `EpochOverflow` rather than reading the low eight regardless
(`:123-134`).

**And now the part of this hazard that is its real shape.** `get_org_state`
returns `Ok(None)` the moment any one slot reads absent
(`on-chain-client/src/client.rs:196-199`), and the comment beside it gives the
reason: "The contract writes all three slots atomically on genesis so a
partial-read state shouldn't appear". Both halves of that sentence are **true
today**, and they are true because of `on-chain/src/OrgRegistry.sol`:

- `update` reverts with `ZeroValue()` if either `rootHash` or `orgPubKey` is
  zero (`on-chain/src/OrgRegistry.sol:31`), so neither of those two fields can
  legitimately hold zero;
- the epoch is set to `1` at genesis (`:38`) and incremented by one thereafter
  (`:45`), so it is never zero after initialisation;
- all three fields are written in the one external call, in both branches
  (`:36-38` and `:43-45`), so there is no window in which one is set and another
  is not.

Every slot of an initialised Organisation is therefore non-zero, and the three
are always written together. That is what makes "absent means no such
Organisation" sound.

Residual risk: reduced, **not acceptable**, on two counts, the first stated
plainly because ISO 14971 requires exactly this to be recorded rather than
assumed and the second because the probability estimate now turns on it.

**The control that holds down two of this hazard's three routes is implemented
outside this unit, outside this crate, and outside compliance.** It is three lines of
Solidity in a directory `.guardrails/units.yaml` disclaims. The contract's own
tests for exactly the two reverts the argument rests on —
`test_RevertsZeroValue_WhenRootHashIsZero` at
`on-chain/test/OrgRegistry.t.sol:67` and
`test_RevertsZeroValue_WhenOrgPubKeyIsZero` at `:73` — run in **no CI lane in
this repository**: the workflow directory holds a Rust workflow and a Quint
workflow, and neither mentions `forge`. So a later contract revision that
relaxed the zero guard, or that wrote the fields in two calls, would make this
reader's `Ok(None)` wrong for an Organisation that exists, and nothing on this
side would notice.

ISO 14971 permits a control whose implementation lies outside the software under
analysis. What it does not permit is leaving it unrecorded, which is what this
register would be doing if it stated the `Ok(None)` shortcut as safe without
naming what makes it so. The control is named, its implementation is located,
and the gap — that no gate here re-establishes it — is not-minted control 9.

**And the third route has no such control at all.** RC-sxjnx9 refuses a
wrong-width blob, which is the right answer for the storage path and is what
turns a shape change into a typed error rather than an offset `OrgState`. It
does not make the shape change less likely, and nothing in this unit does: the
runtime API's answer is the runtime's to define, the decoder for it is pinned at
construction and never re-checked (PR-w5sk5k, under HAZ-8s5chy). A slot value
that came back at an unexpected *width* would indeed be refused, with both
figures named (`on-chain-client/src/client.rs:195-205`). A slot value that came
back **absent** because the runtime API now answers a differently-shaped
question would not be refused at all: it takes the `Ok(None)` shortcut
(`:196-199`) and the consumer is told there is no such Organisation, which is
the unavailability half of this hazard arriving through the one path the control
does not cover. This is the route
that carries the P2, and its residual is HAZ-8s5chy's: one `spec_version`
compiled in, resolved once per session.

One thing this hazard is **not**, and an earlier draft of the survey had it
wrong: it is not about a legitimately-zero field, which the contract forbids. It
is about a load-bearing guarantee held somewhere a gate cannot see — and, on the
third route, about a shape this unit does not own at all.

Three of the nine entries in "Nine claims held by review, not by a gate" are
this hazard's — the contract's guards, above; the width test's derivation, under
RC-sxjnx9; and the `OrgRegistry` storage layout, added to that list at review
round 3 (finding-2) — and none of the three adds a further count to the
residual. The first is already counted here, as the first of the two. The second
changes no residual at all, because what it leaves ungated is a sentence rather
than a property; that distinction is drawn at the list, not repeated here. The
third is the same ungated-contract gap as the first, reaching this hazard by the
same route and closed by the same not-minted control 9 — it is collected
separately because it is a different property of the same disclaimed file, and
it is counted in the residual once, here.

### An uncommitted observation acted on as committed

**HAZ-xd4urb**: the best lane reports events from blocks the chain has not
committed to and can still discard, best-head notifications arrive one per
best head rather than one per block so heights are silently skipped, and a
reorganisation replaces blocks that have already been reported; a reorg
discards a block whose events a consumer has already acted on, or the
notification gap swallows a height, or a repeated head re-delivers events the
consumer has already seen; the consumer holds a membership belief taken from a
block the chain then discarded — a removal believed that never happened, which
locks a member out, or a re-admission believed that was reversed, which leaves
access with someone the Organisation removed. Severity: S3. Probability: P2.

P2: a reorganisation is the chain continuing to run — the Method section's
second exclusion — so no party whose cooperation the situation requires has to
decide to change anything for it to happen; and the notification gap is not an
anomaly at all — it is how `chain_subscribe_new_heads` works, as
`on-chain-client/src/client.rs:297-312` records.

**RC-kemv75**: a best head that replaces a previously reported one is reported
to the consumer as a reorganisation carrying the identity — hash and number
both — of the head that was discarded; every height between the last head
processed and the new one is read, in ascending order, so that no block's events
are silently omitted; and a head already processed is not reported a second
time. mitigates: HAZ-xd4urb

**What this control no longer says**, narrowed at review round 3 (finding-1) and
recorded at the control rather than left to the residual. It opened, until that
round, with a fourth clause: *an observation from a best block is delivered as a
distinct kind of notification from one taken from a finalised block*. That is
true of the code. `SubscribedEvent`'s three variants are distinct
(`on-chain-client/src/state.rs:62-70`), and the two lanes differ in exactly one
thing: the `wrap` closure each passes into `decode_contract_events`. The best
lane's, through `events_in_best_block`, builds a `BestBlockEvent`
(`on-chain-client/src/client.rs:616-618`); `finalised_lane`'s builds a
`FinalisedEvent` (`:459-465`). But **no requirement of this unit states it**,
and none can: both lanes are `async` and need a live chain, so the only targets
that reach the distinction are the nine that run in no gate. A requirement written for it would
be `MISSING-TEST` on the next run, which is the shape this change exists to
remove rather than a fix for one. The clause is now **not-minted control 18**,
and it is the eighth entry of the review-held list at the front of this file.

How it sits in the code. The three notification kinds are distinct variants of
`SubscribedEvent` (`on-chain-client/src/state.rs:62-70`), a fact the control no
longer claims, for the reason given above, and one of the nine held by review.
`Reorged` carries a whole `BlockRef` — hash and number — rather than a hash alone
(`:66-67`, the type at `:11-16`), because a consumer keyed on either needs both.
The decision itself is `internals::scan_step`
(`on-chain-client/src/client.rs:540-565`): the dedup on hash equality at `:549`,
the reorg predicate at `:551-556` — a head at or below the last height, or at
the next height with a parent that is not the last hash — and the ascending
backfill span at `:557-562`. `best_lane` turns that into notifications: the
`Reorged` is emitted **before** the block's own events (`:387-390`), and the
span is iterated in ascending order (`:399`), by number, which
`at_block(number)` always resolves to the canonical block at that height
(`:306-309`).

That decision is not where it started. Until this change it lived inside
`best_lane`'s `.scan()` closure, where nothing without a chain could reach it,
and it had **no tests at all** — the first tests this logic has ever had are
`on-chain-client/tests/best_lane_reorg_rule.rs`, fourteen of them, added in this
change against seven mutations.

**Two of those mutations belong in this hazard's block**, because each is a
plausible simplification of the rule that loses events *silently*, with nothing
anywhere reporting an error. Keying the dedup on block number instead of block
hash makes a depth-1 reorg's **replacement** block read as a repeat, so it is
skipped and its events are never delivered at all. And making the backfill's
`from` overshoot produces a span like `10..=9` — descending, so `for number in
from..=to` iterates zero times and the lane reads no blocks while reporting
success. Neither would surface as an error anywhere. They are the argument for
extracting this rule out of a closure and putting a gate on it.

Residual risk: reduced, **not acceptable**, on three counts, two of them
defects.

**A reorg deeper than the notification gap is undetectable.** For a new head at
`n > last.number + 1` the rule reports no reorganisation, and the code's own
comment says why: "no reorg signal is derivable"
(`on-chain-client/src/client.rs:325-328`). The by-number backfill cannot supply
one either, because looking a height up by number always resolves the canonical
block and so can never observe the discarded sibling (`:306-309`). A consumer
that acted on a block two or more heights back, when the chain reorganised
across that gap, is never told. That is **PR-qpp28h**, open. The fourteen tests
pin today's behaviour here, including a jump case that asserts `reorged: None`
— asserted and labelled as *today's behaviour under an open report*, not as a
claim that nothing was discarded.

**The first backfill span is unbounded.** The scan is seeded from
`at_current_block()`, which is the latest **finalised** block, and on a live
chain that lags the best tip by many blocks; the first head notification then
backfills that entire span at one `at_block` round-trip per height. The code
admits it in a `NOTE` (`on-chain-client/src/client.rs:391-398`) and the seeding
comment explains the reason (`:330-339`). Fine for a manual-mining fork;
unbounded work on a live chain, which is an availability cost on the consumer.
That is **PR-uq5r97**, open.

**And RC-kemv75 covers the decision, not the delivery.** Everything gated here
is `scan_step`: given a notification, what should be emitted. Whether subxt
delivers the notifications that decision is made on — whether a reorg produces a
head notification at all, whether the finalised lane is what it claims — is
established by exactly one test, `on-chain-client/tests/reorg_cancels_proposed.rs`,
which needs a chopsticks fork and runs in no gate. And that test cannot be
evidence about finality anyway: **chopsticks finalises every `dev_newBlock`
immediately**, as that file's own header records, which is a divergence from
live GRANDPA where a finalised block can never be reorged. The only test in the
crate that could establish anything about real finality is
`on-chain-client/tests/smoldot_smoke.rs`, which is `#[ignore]`d, needs live
Paseo, and runs nowhere. So the distinction between provisional and committed —
which this control opened with until review round 3 (finding-1) narrowed it out,
and which the code does draw — has **no gated evidence at all** on the side that
matters. That is not-minted control 10, with control 18 naming the clause
itself, and it is the eighth entry of "Nine claims held by review, not by a
gate": the distinction is real, established by reading, and re-established by
nothing.

## Hazards introduced by these controls

ISO 14971 requires each control to be checked for what it breaks, and this
section is the one place in the file where a bookkeeping error is not a
bookkeeping error — an omission here is a control never checked. Review round 2
(finding-3) found two: the sweep both named RC-6gfh8d and RC-sxjnx9 as
introducing nothing *and* gave them an entry below, and it left RC-kemv75 out of
both lists entirely — the one control of the nine never checked for what it
breaks. Both are corrected here, and the counting is re-derived over the
corrected sweep.

**Five of the nine controls have something to record** and are assessed below:
RC-d7r82e,
RC-5e3bdk, RC-6gfh8d, RC-sxjnx9 and RC-kemv75. They occupy **four entries**, not
five, because RC-6gfh8d and RC-sxjnx9 introduce one consequence between them —
the fail-closed cost of the typed refusals — and are assessed in a single entry.
Both figures are used in this file and each says which it counts: five is a
count of **controls**, four a count of **entries**. **The other four introduce
nothing today** — RC-8w9wtp, RC-mugta4, RC-5ejucb and RC-675a3h — and are named
so that they are revisited rather than quietly dropped. RC-8w9wtp adds no
refusal of its own beyond the ones RC-6gfh8d's entry assesses; RC-mugta4 and
RC-5ejucb state derivations that reject nothing and emit nothing, their one
recorded consequence being the `u64` narrowing under HAZ-v2cmtx, which is a
limit of the derivation and not a situation the control creates; and RC-675a3h
adds no behaviour at all — it states the binding the decoder already had and
puts an independently recomputed test on it.

**How the entries are counted**, using the same test org-node's register
applies: an entry is a *distinct hazard* when its hazardous situation and its
harm are ones no hazard already counted covers, and a *further trigger* when it
is another route to a hazardous situation and harm already counted. By that
test, counted **per entry**, one of the four entries below is a distinct hazard
and three are further triggers; counted **per control**, one of the five is a
distinct hazard and four are further triggers. Either way these controls add
**one** to the register's count — the correction to the
sweep adds an entry, not a hazard.

- **RC-d7r82e (refuse an unrecognised runtime).** Exactly one `spec_version` is
  compiled in (`on-chain-client/src/decode/v_paseo_ah.rs:45`,
  `on-chain-client/src/decode/dispatch.rs:26-31`), and the refusal is at
  construction (`on-chain-client/src/client.rs:138-139`), so it is not a
  degraded mode — a fact this assessment leans on, established by reading and
  re-established by no gate, which is the seventh entry of "Nine claims held by
  review, not by a gate" and not-minted control 19. On the day Paseo Asset Hub
  upgrades, `from_client` fails, no
  client can be built, no state can be read and no subscription can be opened,
  for every consumer, until a decoder module and a match arm are written,
  reviewed and shipped. Assessed **S3 / P2**, **not acceptable**. The severity
  is the unavailability pathway's at its worst credible outcome, as the Method
  section argues, and it is not argued down from the fact that a fix exists —
  recoverability is not severity, which is the correction the class C ADR made
  to itself on 2026-09-02. The probability is the probability of a runtime
  upgrade, which for a pre-stable pallet on a testnet parachain is occasional
  rather than remote; that is the same P2 HAZ-8s5chy carries, and this is a
  further trigger for HAZ-8s5chy's unavailability half rather than a hazard of
  its own — the hazardous situation is the runtime moving out from under the
  pinned decoder, which is that hazard. Two controls, neither minted: widen
  `for_runtime` to a known-good range once several upstream versions have been
  round-tripped, which the code's own comment already contemplates
  (`on-chain-client/src/decode/dispatch.rs:14-18`) — not-minted control 5 — and
  re-check the version per read rather than once per session, which is also
  PR-w5sk5k's fix and not-minted control 4. Note the second does not remove the
  outage; it converts a session that silently decodes post-upgrade bytes into
  one that fails loudly, which is a different and better failure.
- **RC-5e3bdk (the contract-address filter).** A log that fails the address
  comparison is dropped with `continue`
  (`on-chain-client/src/client.rs:680-682`) — not surfaced, not counted, not
  reported. Two consequences, and they are opposite in sign. The intended one
  is that the stream is not polluted by every other contract's logs, which is
  right and is why `Ok(None)` on an unknown signature is not an error either
  (`on-chain-client/src/decode/v_paseo_ah.rs:117`). The unintended one is that
  **an OrgRegistry redeployed at a new address, or a reader constructed with a
  stale address, hears nothing and says nothing**: every genuine event of the
  Organisation is silently discarded, and a consumer whose subscription has gone
  quiet cannot distinguish "no membership changes" from "watching the wrong
  contract". Its record then diverges from the chain's for as long as the
  mistake lasts. Assessed **S3 / P1**, **not acceptable**. The harm is the
  unavailability pathway — a member's device acting on a membership that has
  moved on — and by the counting test this **is** a distinct hazard: no hazard
  above covers a reader that is correct, working, and pointed at the wrong
  place. P1 because it needs a redeployment or a misconfiguration, neither of
  which is an ordinary event, though neither requires an attacker. Two controls,
  neither minted: report a log whose first topic matched a known signature but
  whose emitting contract did not match the configured one — at least once, as a
  distinguishable notification rather than silence, since that observation is
  simultaneously the evidence of a spoofing attempt (HAZ-werm85) and of a
  misconfiguration (not-minted control 12); and give a consumer a way to check
  the configured address against the chain at construction, which is
  not-minted control 14's larger half.
- **RC-sxjnx9 and RC-6gfh8d (typed refusals).** Every guard these two describe
  converts a malformed input into an `Err` rather than a guess, and every such
  `Err` propagates out of `get_org_state` or onto the subscription stream, where
  the consumer's verification fails with it. Fail-closed is the intended
  direction and is what stands between the wrong-reading half of HAZ-95sc43 and
  HAZ-xfg9cz and a consumer that acts on it — a refusal in place of a guess,
  which reduces what the situation leads to and not how often it arises.
  The cost is that a single malformed slot value, or a
  single malformed log from any contract on the chain, is enough to fail a
  consumer's read — and on the event path the input is attacker-chosen, so an
  attacker who cannot forge an acceptable event can still emit an
  **unacceptable** one and make the stream carry an error. By the counting test
  this is a further trigger for HAZ-95sc43's own unavailability half rather than
  a hazard of its own, and it is assessed **S3 / P2** there. It is recorded here
  because the trade is real and was chosen: a decoder that recovered instead of
  refusing would be the wrong-reading hazard, and that is worse.
- **RC-kemv75 (the best lane's step rule).** This entry was missing entirely
  until review round 2 (finding-3); its two situations were already argued under
  REQ-gr2ver and REQ-5zux82 in the derived assessment, and what was absent was
  the sweep that claims to have checked every control. Two consequences, both of
  clauses the control asks for rather than of the code's incidental shape.
  **The ascending-backfill clause makes the first span as long as the finality
  lag.** Requiring that every height between the last processed head and the new
  one be reported means the *first* notification backfills the whole distance
  from the seed, which is the finalised head
  (`on-chain-client/src/client.rs:330-339`, `:391-398`); on a live chain that is
  unbounded work at one `at_block` round-trip per height before a consumer is
  told anything. **And the dedup clause means a head is reported once and never
  again.** Keying it on hash rather than number is what makes it right
  (`:549`) — a number-keyed dedup would discard a depth-1 reorg's replacement,
  which is the mutation the fourteen tests were written against — but the
  consequence of any dedup is that a consumer which lost or failed to process
  the first report of a head is never given a second, and nothing reports the
  fact. Assessed **S3 / P2**, **not acceptable**. The severity is the
  unavailability pathway's, as the Method section argues. P2 because neither
  situation needs anybody to decide to change anything: a finality lag and a
  repeated head notification are the chain and subxt doing what they ordinarily
  do. By the counting test both are **further triggers** and neither is a
  distinct hazard: the unbounded span leaves a consumer's view not advancing,
  which is the stale-view hazard in prose below, and the never-repeated head
  leaves its event record with a hole, which is HAZ-xd4urb's own situation. Both
  are already carried as residuals of that hazard, the first as PR-uq5r97 with
  not-minted control 7 against it. What is added here is only that they are
  consequences the control *asks for*, not defects beside it.

## Hazards recorded in prose

These carry severity, probability and their controls, and no identifier, for the
reason both sibling registers state: the gate refuses a hazard whose control is
not implemented and tested, and an identifier would assert a traced control
where there is none. Nothing is lost to a reader; what is lost is the mechanical
trace, and that is reported here rather than worked around.

**The module named for verification contains nothing.** `on-chain-client/src/verify.rs`
is seven lines of doc-comment and no code (`:1-7`). The design names it as the
module that "closes the loop with `org_members::CandidateTrie::verify_against`"
and says it "lands in Stage 2 Task 5"; Stage 2 landed and it did not. So the
unit named for verification performs none, and the whole burden of comparing a
recomputed Membership root against the chain's sits on the consumer — which the
consumer does do, in `org-node/src/verify.rs`, but by its own arrangement rather
than by this unit's. Hazardous situation: a second consumer, written from the
design, calls a verifier that is not there, or writes its own and gets a detail
wrong that a shared implementation would have got right. Harm: as HAZ-werm85's —
a membership belief formed on the wrong root. Assessed **S3 / P1**: it needs a
consumer to be written, and today there is one. Filed as **PR-h4mb8y**. The
control — implement the module, or delete it and move its promise into the
design where a reader will not mistake it for shipped code — is not minted
(not-minted control 6).

**One address, trusted totally.** RC-5e3bdk's check is that the emitting address
equals the configured one, and that is the whole of it: there is no code-hash
check, no interface probe, and nothing that establishes that the contract at the
configured address is the `OrgRegistry` this decoder was written against. A
consumer handed the wrong address, or an address at which a different contract
now lives, gets a reader that filters diligently for the wrong thing. Nothing in
this crate can tell the difference, because the only thing it reads from that
address is raw storage slots and raw log topics, both of which any contract can
produce. Assessed **S3 / P1** — it needs a wrong address to be supplied, which
is a deployment act. The control is not minted (not-minted control 14): read
something at construction that only the intended contract could produce, or
accept a code hash alongside the address and check it.

**Nothing re-reads on a schedule, and nothing reports staleness.**
`get_org_state` reads when it is asked; `subscribe` streams while its stream is
held. A consumer that does neither holds whatever it last read, for as long as
it holds it, and this crate offers it no signal at all about age: an `OrgState`
carries a root, a key and an epoch (`on-chain-client/src/state.rs:18-25`) and
**no block reference**, so a value returned by `get_org_state` cannot afterwards
be asked which block it came from. Hazardous situation: a consumer verifies
against an Organisation state older than the chain's. Harm: the unavailability
and disclosure pathways both, depending on which way the state moved. Assessed
**S3 / P2** — holding a value is the ordinary case. The consumer-side half of
this is the stale-view hazard org-node's register assesses at the same level;
what belongs on this side is the missing signal. Not minted (not-minted control
11).

## The obligation this change does not meet

org-node holds one requirement addressed to this unit: **REQ-ysyu9g**, an
`expects:` item requiring that a state read taken without naming a block return
the Organisation state recorded at the latest Finalised block, so that the
Membership root a Change set is verified against is never one a reorganisation
can later discard. It falls due 2026-12-05 under `expectation_age_days: 90`.

**This change does not meet it**, deliberately, and the owner's reasoning is
recorded in the plan's interview. What it does instead is establish the fact.

The fact is confirmed, not inferred. `get_org_state(admin, None)` takes the
`None` branch and calls `self.api.at_current_block()`
(`on-chain-client/src/client.rs:181-186`), and subxt 0.50.1 documents
`at_current_block` as instantiating a client "to work at the current finalized
block _at the time of instantiation_"
(`subxt-0.50.1/src/client/online_client.rs:269`); the implementation immediately
below it resolves `latest_finalized_block_ref()`. So this crate's own
doc-comment — "`at = None` reads at the latest finalised block"
(`on-chain-client/src/client.rs:4-7`) — is right, and the contradicting
doc-comment in `org-node/src/chain_read.rs` is wrong. That contradiction is an
open report in the consumer's own ledger
(`org-node/docs/problems/2026-09-09-org-node-problems.md`), cited by file
because a provider may not name a consumer's items; this change does not touch
it.

Why the obligation is recorded rather than discharged. Meeting it means writing
an **exported** requirement here carrying a `satisfies:` line naming the
consumer's expectation, and the moment that lands, the consumer's item stops
being an exempt expectation and becomes an ordinary requirement, which
`check-trace.sh` then requires a `verifies:` test for **on the consumer's side,
at the same merge**. The only honest test of "this read is at the finalised
block" is a chopsticks target, and chopsticks finalises every `dev_newBlock`
immediately — so such a test would establish nothing about finality even if a
gate ran it, and no gate runs one. Delivering the requirement without the
evidence would put a green tick on the one property this register has just spent
a section explaining has no gated evidence.

So it is recorded here as **not-minted control 3**, with its deadline, and the
gap it leaves is named where it bites: the decisive input to the consumer's
root-matching control rests on a doc-comment of this crate plus a doc-comment of
subxt, both of which this change has now read and confirmed, and neither of
which is a commitment either library owes the other. Nothing in this register
minted a control that turns on it.

## Residual risk, and the conclusion this analysis reaches

Per hazard, after controls:

| Hazard | S/P | Residual | Why |
|---|---|---|---|
| HAZ-werm85 | S3/P2 | not acceptable | the ownership decision is gated, its call site is not; one address trusted totally; a rejected log is silent |
| HAZ-8s5chy | S3/P2 | not acceptable | the decoder is pinned once and never re-checked (PR-w5sk5k); one version compiled in; that version's *value* is held by review, not by a gate |
| HAZ-95sc43 | S3/P2 | not acceptable | one second of generative fuzzing at the gate; one target's corpus empty; clippy `--lib` only |
| HAZ-v2cmtx | S3/P1 | not acceptable | pinned against our reading of pallet-revive, not the runtime's answer; the contract half is disclaimed and ungated |
| HAZ-xfg9cz | S3/P2 | not acceptable | two of its three routes rest on three lines of Solidity outside compliance, tested in no lane here; the third is a runtime-shape change, ordinary and P2 |
| HAZ-xd4urb | S3/P2 | not acceptable | deep reorgs undetectable (PR-qpp28h); first span unbounded (PR-uq5r97); no gated finality evidence |

And the three prose hazards: the empty verifier module **not acceptable**
(S3/P1, PR-h4mb8y); one address trusted totally **not acceptable** (S3/P1); no
re-read and no staleness signal **not acceptable** (S3/P2). The controls
introduced **four entries** of their own — a count of entries, not of controls:
the five controls with something to record share four entries, because
RC-6gfh8d and RC-sxjnx9 are assessed together — and **none of the four is
acceptable**:
the total refusal on an unrecognised runtime **not acceptable** (S3/P2, a
further trigger for HAZ-8s5chy), the silently deaf reader pointed at a stale
address **not acceptable** (S3/P1, a distinct hazard), the fail-closed cost
of the typed refusals **not acceptable** (S3/P2, a further trigger for
HAZ-95sc43), and the best lane's unbounded first span and never-repeated head
**not acceptable** (S3/P2, further triggers for the stale-view prose hazard and
for HAZ-xd4urb). The fourth was added at review round 2 (finding-3), which found
RC-kemv75 missing from the sweep altogether.

The count is six identified hazards, three further distinct hazards in prose,
and one further from the controls themselves — **ten**. The counting test is the
one stated under "Hazards introduced by these controls", and it excludes three
of the four introduced entries as further triggers for hazards already counted;
completing the sweep in round 2 added an entry and no hazard, so the total is
unchanged. **Of all ten, and of the three further triggers with them, not one
carries a residual risk this project's matrix calls acceptable**, and no probability
estimate could have changed that: the matrix makes S3 unacceptable at P1, and
this unit's harm pathway is S3 throughout for the reason the Method section
gives. There is no S2 anywhere in this file, and no severity in it is argued
down from the existence of a workaround.

### Overall residual risk

**Against the intended use that makes this software Class C, the overall
residual risk of the on-chain-client unit is UNACCEPTABLE.**

That restates, for this unit, the conclusion org-members' register reached for
the membership capability on 2026-09-02 and org-node's for the node on
2026-09-09, and it does not change either. The specific reasons here are:
four open defects, one of them the mechanism by which a wrong runtime's bytes
are decoded silently and two of them limits of the reorg rule the code itself
admits to; two controls whose soundness rests on a file this repository
disclaims and whose tests no lane here runs; one control — the mapping — pinned
against our reading of an upstream convention rather than against the upstream
itself, with the test that would fix that sitting in the crate, written, and
ungated; an expectation from a consumer that this change deliberately does not
meet, with its deadline recorded; and all four of the entries the controls
themselves introduced unacceptable in their own right.

What follows is what followed there. The software is a proof of concept and has
not reached its intended use; the restriction on use still has to be written
where a user meets it (setup checklist, owner's item); the class stays C because
it is set by the planned use, and lifting the restriction reopens this
evaluation, not the class. And, as there, the verdict is a reasoned conclusion
rather than the output of a stated criterion, because the project has no risk
management plan defining overall acceptability — the same open item on the
checklist.

One thing this register can say that neither sibling could, and it belongs in
the conclusion rather than being lost in a task note. **This change measured its
own false green and removed it.** The plan's red-by-mutation instruction for
RC-5e3bdk was followed exactly, the mutation was applied, and the tests stayed
green — which is the answer nobody wants and the only one worth having. The
control's decisive line was unreachable by every gate in the repository, the
fix's own tests could not have told anyone if it were deleted, and that was
found by running the prescribed step rather than by reasoning about it. The
remedy was structural (extract the decision, gate the decision) and the result
is recorded above with the three mutation counts. A register that reported "red
→ green, control verified" for that task would have been literally true and
substantively false.

## Controls identified but not minted

Each would reduce a residual risk above, with the single exception of 17, which
would reduce none and says so at its entry: it makes a claim accurate rather
than a property covered, and it is listed here because this is where a reader
looks for the work that would close a gap. None has a requirement, and none is
given an RC identifier until it does — an identifier here would assert a traced
control where there is none.

1. **Pin the H160 mapping against the runtime's own answer at a gate**
   (HAZ-v2cmtx). The test exists:
   `on-chain-client/tests/p_address_is_orgid.rs` takes the admin out of a real
   `GenesisInitialized` event on a chopsticks fork and compares it with
   `h160_of`. It runs in no gate because no lane starts a fork. Until it does,
   RC-mugta4 pins our reading of pallet-revive rather than pallet-revive.
2. **Seed `fuzz_event_round_trip`'s corpus** (HAZ-95sc43). It is the one of the
   three targets with no seeds, and the seeding job is different from the other
   two's: `tests/regenerate_corpus.rs` seeds the raw-byte targets, while this
   one consumes driver bytes for a `TypeGenerator`. Visible in measured output
   since 2026-08-26 and never written down as a gap until now.
3. **Meet REQ-ysyu9g** — an exported requirement of this unit stating that a
   state read with no block named returns the state at the latest Finalised
   block, carrying the `satisfies:` line the expectation needs, together with
   whatever evidence a gate can actually run. Due **2026-12-05**. See "The
   obligation this change does not meet" for why it is here and not in the
   requirements ledger.
4. **Re-check the runtime version per read** (HAZ-8s5chy, PR-w5sk5k). The
   information is already in hand — `get_org_state` resolves a block on every
   call and that block carries `spec_version()` — and the comparison against the
   pinned version is not made. This converts a session that silently decodes
   post-upgrade bytes into one that refuses loudly.
5. **Widen `for_runtime` to a known-good range** (HAZ-8s5chy, and the outage the
   refusal introduces). The code contemplates it already
   (`on-chain-client/src/decode/dispatch.rs:14-18`); what it needs is several
   upstream versions round-tripped, which needs control 10.
6. **Implement `verify.rs`, or delete it** (the empty-verifier hazard,
   PR-h4mb8y). Either closes it. What is not acceptable is a module that
   promises in the present tense a verification it does not perform.
7. **Cap the first backfill span, with a resync strategy** (HAZ-xd4urb,
   PR-uq5r97). The seed is the finalised head and the first notification
   backfills the whole lag at one round-trip per height.
8. **Detect a reorganisation deeper than the notification gap** (HAZ-xd4urb,
   PR-qpp28h). Today none is derivable from the head notifications alone and the
   by-number backfill cannot supply one. It needs the discarded chain to be
   walked, or a different subscription.
9. **Bring the contract's guards inside a gate** (HAZ-xfg9cz, HAZ-v2cmtx). Two
   controls of this register are implemented in `on-chain/src/OrgRegistry.sol`
   — the `ZeroValue()` revert and the atomic three-field write that make
   "absent means no such Organisation" sound, and the storage layout the slot
   derivation hard-codes. Its own tests cover the first
   (`on-chain/test/OrgRegistry.t.sol:67`, `:73`) and run in no lane here. Either
   run `forge test` in CI, or add a chopsticks fixture on this side that
   re-establishes both properties against a deployed contract.
10. **Run the chain-dependent lane in a gate** (HAZ-xd4urb, HAZ-werm85,
    HAZ-v2cmtx). Nine targets in `on-chain-client/tests` run at no gate: seven
    need a chopsticks or anvil fork, `smoldot_smoke` needs live Paseo, and
    `regenerate_corpus` needs no chain at all. (**Corrected 2026-09-28 by the
    review sweep of the change `worktree-guardrails-on-chain-client-arch`:**
    this read "nine … and one needs live Paseo … none of the ten", counting ten
    ungated targets where there are nine. `smoldot_smoke` **is** one of the
    nine.)
    The eight of them that touch a chain are where every chain-dependent
    property of this crate is tested.
    Note the limit that comes with it: chopsticks finalises every `dev_newBlock`
    immediately, so gating those targets buys reorg and delivery evidence but
    **not** finality evidence, which needs the live-Paseo path.
11. **A staleness signal** (the no-re-read hazard). Return the block a state was
    read at alongside the state, so a consumer can tell how old its belief is;
    optionally a re-read on a schedule. Today `OrgState` carries no block
    reference at all.
12. **Report a signature-matching log from an unconfigured contract**
    (HAZ-werm85, and the deaf-reader hazard RC-5e3bdk introduces). One
    observation serves both: it is the evidence of a spoofing attempt and the
    evidence of a misconfiguration, and today it is `continue`.
13. **A two-contract chopsticks fixture** (HAZ-werm85). Every existing
    chopsticks target deploys one OrgRegistry, which is why the real
    `decode_contract_events` path has never been exercised against an impostor.
    Deploy a second contract that emits a well-formed OrgRegistry log with the
    victim's indexed admin, and assert the subscriber receives nothing.
14. **Establish that the configured address holds the expected contract** (the
    trust-one-address hazard). A code hash checked at construction, or a read at
    construction that only the intended contract could answer.
15. **Extend the panic-freedom gate past `--lib`, and make the crate
    `rustfmt`-clean** (HAZ-95sc43). Clippy lints `--lib` only, so the nine new
    integration targets are outside the denial the crate root declares; and
    there is no `cargo fmt` step in the workflow, against a crate that is not
    clean at baseline. Both are decisions to take deliberately rather than
    defects to fix in a hazard analysis.
16. **A deep-fuzz lane** (HAZ-95sc43). The three targets exist and run one
    second each at the gate under the generative engine. A libFuzzer lane, with
    control 2's corpus, is what would make RC-8w9wtp's claim carry weight
    beyond that second.
17. **Couple the width test to the decoder's regions by exporting them from
    `src`** (HAZ-xfg9cz). `on-chain-client/tests/type_widths.rs` derives each
    newtype's width as the ABI word minus an offset, from three `const`s of its
    own; `unpack_address_topic` and `decode_uint256_to_u64`
    (`on-chain-client/src/decode/v_paseo_ah.rs`) hold the offsets the decoder
    actually reads, and the two sets are kept in step by review. Making them one
    value — the offsets as constants in `v_paseo_ah.rs`, re-exported under the
    `test-support` feature and imported by the test — would turn a mirrored
    comment into a compiler-checked coupling. It is a change to production code
    and is proposed here rather than made in a hazard analysis. Its value should
    be weighed before it is done, and this register's own measurement is the
    argument against urgency: the offsets are already gated by
    `on-chain-client/tests/decode_org_state.rs`, which reds at 4 of 9 on exactly
    the drift this coupling would catch, so what the coupling buys is not
    coverage but that the test's stated *reason* stops being hand-maintained.
    Unlike controls 1 and 9 it reduces no residual risk above; it is listed here
    because the same list is where a reader looks for "the thing that would close
    the shape".
18. **Gate the best/finalised distinction** (HAZ-xd4urb, RC-kemv75). *The
    behaviour*: an observation taken from a best block is delivered as a
    `SubscribedEvent::BestBlockEvent` and one taken from a finalised block as a
    `FinalisedEvent`, so a consumer can act optimistically on the first and
    commit only on the second. *It is real in the code*: the three variants are
    declared distinct at `on-chain-client/src/state.rs:62-70`, and the two lanes
    differ in exactly one thing — the `wrap` closure each passes into
    `decode_contract_events`, the best lane's through `events_in_best_block`
    (`on-chain-client/src/client.rs:616-618`) and `finalised_lane`'s inline
    (`:459-465`). *No gate reaches it*: both
    lanes are `async` and need a live chain, so the distinction is settled
    nowhere a gated test can look, and the whole of what a gated test can see is
    that three variants exist. *The ungated target that would*:
    `on-chain-client/tests/reorg_cancels_proposed.rs`, which subscribes against
    a chopsticks fork — and it buys only half, because chopsticks finalises
    every `dev_newBlock` immediately, so the committed side of the distinction
    needs `on-chain-client/tests/smoldot_smoke.rs` against live Paseo. Both are
    among the ten control 10 would run. This was RC-kemv75's opening clause
    until review round 3 (finding-1) found that none of that control's three
    requirements states it. It is recorded here rather than minted as a
    requirement because a requirement for it could carry no gated test at all —
    `MISSING-TEST` on the next run — which is the defect this change removes,
    not a fix for one.
19. **Gate the construction-time refusal of an unrecognised runtime**
    (HAZ-8s5chy, RC-d7r82e). *The behaviour*: a `spec_version` for which no
    decoder is compiled in is refused **before any read is possible**, so there
    is no client that exists and reads through a decoder it could not resolve.
    *It is real in the code*: `from_client` reads the version off the block it
    is constructed against and returns `Err(ClientError::UnsupportedRuntime)`
    before a `Self` is built (`on-chain-client/src/client.rs:129-147`, the
    refusal at `:138-139`). *No gate reaches it*: `from_client` is `async` and
    needs a live chain; `on-chain-client/tests/runtime_version_dispatch.rs`
    gates `dispatch::for_runtime`, the decision `from_client` delegates to, and
    can say nothing about when that decision is taken or what happens to the
    client if it refuses. *The ungated target that would*: any of the
    chopsticks targets — `on-chain-client/tests/00_chopsticks_sanity.rs` is the
    smallest — constructing against a fork whose runtime version has been
    overridden to one no decoder is compiled for, and asserting that
    `from_client` returns `UnsupportedRuntime` rather than a client. This was
    RC-d7r82e's last clause until review round 3 (finding-1), and it is recorded
    here rather than minted for the same reason as 18.

## Derived requirements assessment

Every requirement minted by this change is `satisfies: derived`: each exists
because of how this reader was built and what it was partitioned to do — a
decoder pinned to a runtime version, a slot key derived offline, a stream of
best and finalised observations — and not because a system-needs document asked
for it. Finding the hazard a rule addresses gives it a justification, not a
parent. All twenty-one are assessed here, and each has a hazard to point at.

- **REQ-9vwcwc** (deliver a log only if the contract that emitted it is the
  configured one, so that no other filter can admit a log from another
  contract): realises
  RC-5e3bdk against HAZ-werm85. Introduces the
  silently-deaf reader assessed under "Hazards introduced by these controls"
  (S3/P1, not acceptable: a stale or wrong configured address discards every
  genuine event without saying so). It states the **delivery** half only; the
  reporting half it used to carry is REQ-5upq6n, split out at review round 1
  (finding-12). Its evidence is
  `on-chain-client/tests/log_ownership.rs`, which
  reaches `internals::log_is_ours` itself rather than reimplementing it.
  **What that evidence gates, measured.** An **ordering** clause was folded into
  this requirement at review round 2 (finding-11), out of REQ-nygs7k, on the
  ground that ordering is a property of the contract check and that
  `the_contract_check_dominates_a_matching_admin_filter` was already annotated
  here. Review round 3 (finding-3) measured that no test gates ordering at all:
  `internals::log_is_ours` rewritten to evaluate the admin filter first and
  return the contract comparison last — the exact inversion — left
  `--test log_ownership` at **7 passed, 0 failed**, because ordering inside a
  total boolean predicate is not observable from outside it. What the suite does
  gate is the **consequence** the requirement now states, and it reds on it:
  inverting the comparison reds 6 of 7 cases, deleting it outright reds exactly
  the three rejection cases — `the_contract_check_dominates_a_matching_admin_filter`
  among them, which is the case that establishes a matching admin cannot rescue
  a foreign log. So the clause is dropped from the requirement and from
  RC-5e3bdk, and what remains is what the evidence supports. The dominance is
  still true of `on-chain-client/src/client.rs:513-519`; it is recorded there as
  a fact about the code, not asserted as a verified property.

assesses: REQ-9vwcwc

- **REQ-5upq6n** (the *decoder* reports the Emitting contract's address with
  every `OrgRegistry` event it decodes): realises RC-5e3bdk against HAZ-werm85.
  The surface is named in the requirement's own text from review round 3
  (finding-8) — it said "the software shall report" and did not say *to whom*,
  which overstated the reach. The decoder does report it, in `EmittedEvent`
  (`on-chain-client/src/state.rs:51-53`); the address is then consumed by
  `log_is_ours` and **dropped before a subscriber sees it**, because
  `SubscribedEvent` carries no contract field (`:62-70`) and
  `decode_contract_events` lifts only `emitted.event` through `wrap`
  (`on-chain-client/src/client.rs:683`). That is the intended design — the
  address is what the filter is *for*, not something a consumer has to
  re-check — and both the register and this file's evidence always meant the
  decoder's surface; only the requirement's wording did not say so. No hazard
  impact of its own —
  it reports a value, it discards nothing — but it is the precondition for
  REQ-9vwcwc being implementable at all, and its absence *was* the defect:
  before this change the address was decoded into `let _contract` and dropped,
  so the delivery filter had nothing to compare. Split out of REQ-9vwcwc at
  review round 1 (finding-12), which found that one requirement was carrying a
  delivery obligation and a reporting obligation verified by different tests in
  different files, against this ledger's single-behaviour form. Its evidence is
  `on-chain-client/tests/contract_address_filter.rs`, which after finding-3's
  fix is exactly what that file is for.

assesses: REQ-5upq6n

- **REQ-nygs7k** (with an admin filter set, deliver a log only if the event's
  admin is the filtered one): realises RC-5e3bdk against HAZ-werm85. No hazard
  impact of its own — the contract check dominates it, so a matching admin can
  never rescue a foreign log — but it is what stops a consumer that subscribed
  for one Organisation forming a membership belief from another's events, which
  is behaviour that had no item before this change. It now states that one
  behaviour and no other: the clause fixing *when* the admin comparison is made
  relative to the contract comparison moved to REQ-9vwcwc at review round 2
  (finding-11), where the contract check and its test already are, and review
  round 3 (finding-3) then dropped it from there as well, having measured that
  no test can gate an ordering internal to a total boolean predicate. What
  survives of it is the consequence, stated by REQ-9vwcwc and gated: a matching
  admin can never rescue a log from a foreign contract.

assesses: REQ-nygs7k

- **REQ-wnjz9j** (a first topic matching no known signature, or no topics at
  all, yields no event and no error): realises RC-6gfh8d against HAZ-95sc43.
  Introduces one situation worth naming: an event this decoder does not know is
  indistinguishable from a log that is not ours, so a future OrgRegistry event
  type would be silently ignored by an old reader rather than refused. That is
  the intended direction for a stream carrying every contract's logs, and it is
  the same silence not-minted control 12 addresses.

assesses: REQ-wnjz9j

- **REQ-52uc8f** (recognise exactly the two Event signatures the deployed
  contract declares, each the `keccak256` of its canonical Solidity signature
  string): realises RC-675a3h against HAZ-v2cmtx. Introduces one situation, and
  it is the mirror image of REQ-wnjz9j's: a binding this tight means a contract
  that legitimately adds a third event, or renames a parameter of one of these
  two, is not partially understood by an old reader — it is not understood at
  all, and the reader goes quiet rather than wrong. That is the intended
  direction and the same silence not-minted control 12 addresses. Minted at
  review round 1 (finding-7a), because the behaviour was pinned by a test
  annotated to REQ-wnjz9j, which states only the negative — an unknown first
  topic yields nothing — and never says which signatures are known.

assesses: REQ-52uc8f

- **REQ-88fp2h** (refuse a log whose topic count or data length does not match
  the signature it claims, with both figures named): realises RC-6gfh8d against
  HAZ-95sc43. Its hazard impact is the fail-closed cost assessed above: a
  malformed log from any contract puts an error on the stream. Measured
  mutations of its guards panic inside `src` on the too-few side, which is why
  the requirement is worded as an exact match rather than a minimum.

assesses: REQ-88fp2h

- **REQ-twdu84** (refuse an indexed address topic with any non-zero byte in its
  twelve-byte padding): realises RC-6gfh8d against HAZ-95sc43. No hazard impact:
  a refusal yields an error and no event. It is the boundary at which a
  non-Solidity emitter is told apart from a Solidity one.

assesses: REQ-twdu84

- **REQ-axcxf7** (refuse a payload with trailing bytes, or one that ends inside
  its own framing): realises RC-8w9wtp against HAZ-95sc43. No hazard impact; it
  is what stops a well-formed prefix of a hostile payload being accepted as the
  whole of it.

assesses: REQ-axcxf7

- **REQ-sx5b6g** (any byte sequence yields `Ok` or a typed `Err`, never a
  panic): realises RC-8w9wtp against HAZ-95sc43. No hazard impact. Its evidence
  is the three bolero targets, and the honest statement of what they establish
  is in HAZ-95sc43's residual: one second of generated inputs per target, an
  exit reason rather than a pass count, one of the three unseeded, and the
  SCALE framing decoded by `parity-scale-codec` rather than by anything here.
  Review round 1 (finding-4) removed a clause from this requirement that claimed
  more than that evidence supports — that no decoder allocates from an unchecked
  length taken out of the input — which this unit implements no check for.

assesses: REQ-sx5b6g

- **REQ-n6v896** (for a structurally valid event, decoding its canonical
  encoding reproduces every field, the emitting address included): realises
  RC-6gfh8d against HAZ-95sc43. No hazard impact. This is the **inversion**
  property, and it is a different claim from REQ-sx5b6g's: not that arbitrary
  bytes are survived, but that a well-formed event survives the round trip with
  nothing lost, reordered or silently defaulted — which is what would catch a
  decoder reading the right shape out of the wrong offsets. Its evidence is the
  bolero target `fuzz_event_round_trip`, which generates the event, encodes it
  canonically and compares the decode against the original. Minted at review
  round 1 (finding-7b): that target was annotated `verifies: REQ-sx5b6g`, whose
  arbitrary-bytes property it never exercises — it only ever feeds the decoder
  encodings it built itself — and the annotation contradicted the target's own
  header. The same round noted the corpus limit that applies to it, unchanged:
  it is the unseeded one of the three. **Abnormal-input exemption**, recorded at
  review round 2 (finding-5) rather than answered with an invented case: this
  requirement's evidence is a generator that produces only structurally valid
  events by construction, which is what makes it the inversion property at all —
  feed it a malformed event and it is no longer testing inversion. Abnormal
  input to the same decoder is exactly what REQ-88fp2h, REQ-twdu84, REQ-axcxf7
  and REQ-sx5b6g state and evidence beside it, so the case is not missing from
  the unit; it is stated where it belongs.

assesses: REQ-n6v896

- **REQ-hd6m9d** (resolve, for a `spec_version`, only the decoder compiled for
  that exact version, and refuse an unrecognised one naming the version asked
  for): realises RC-d7r82e
  against HAZ-8s5chy. Introduces the total outage on a runtime upgrade assessed
  above (S3/P2, not acceptable), and does **not** address the pinning defect —
  it governs the resolution, and PR-w5sk5k is about the resolution never being
  repeated. Until review round 2 (finding-1) it said more than that: it required
  that state and events be decoded "only through the decoder compiled for the
  Runtime spec version the chain reported", full stop — a property PR-w5sk5k, in
  this unit's own ledger, records as absent, since `from_client` resolves once
  (`on-chain-client/src/client.rs:137-139`) and `get_org_state` decodes through
  the stored decoder without ever comparing the block's `spec_version()` against
  the pinned one. A requirement may not assert what an open report says the
  software does not do, and the only test the requirement had exercised
  `dispatch::for_runtime` and could not have seen the gap. It is now worded to
  the resolution decision the code makes, as REQ-ntn4ss is worded to the rule
  `scan_step` makes, and the per-decode obligation stays with PR-w5sk5k and
  not-minted control 4.

assesses: REQ-hd6m9d

- **REQ-tkhe3u** (take the reverse mapping case only when all twelve marker
  bytes are the marker): realises RC-mugta4 against HAZ-v2cmtx. No hazard
  impact. Its hardest case is the all-`0xEE` `AccountId32`, where the two paths
  are least distinguishable.

assesses: REQ-tkhe3u

- **REQ-rz7fja** (take the forward mapping case when any of the twelve marker
  bytes is not the marker): realises RC-mugta4 against HAZ-v2cmtx. No hazard
  impact. The pair with REQ-tkhe3u is what pins the window from both edges,
  which one of them alone does not: a mutation narrowing the window reds the
  near-miss, and only the per-position sweep catches which position moved.

assesses: REQ-rz7fja

- **REQ-9m2rnd** (derive the slot key by the Solidity mapping formula over the
  left-padded admin and the map index): realises RC-5ejucb against HAZ-v2cmtx.
  Introduces the `u64` narrowing of a `uint256` mapping index recorded under
  that hazard — not a live limit at slot 0, and recorded rather than fixed.

assesses: REQ-9m2rnd

- **REQ-xudf25** (read successive struct fields at successive slot keys, by
  big-endian increment with carry): realises RC-5ejucb against HAZ-v2cmtx. No
  hazard impact.

assesses: REQ-xudf25

- **REQ-4astjb** (decode an Organisation state only from exactly 96 bytes, and
  refuse any other length naming both figures): realises RC-sxjnx9 against
  HAZ-xfg9cz. Its hazard impact is the fail-closed cost: a wrong-width slot
  value fails the read rather than being decoded. The mutation that widens its
  guard panics on the empty blob, which is the most ordinary abnormal input this
  path sees.

assesses: REQ-4astjb

- **REQ-2qa5r5** (the public newtypes representing an Organisation state's
  fields, and the Organisation admin its slot is keyed on, have exactly the
  widths the contract's ABI gives them: Membership root 32 bytes, the
  Organisation's signing key 32, Epoch 8, Organisation admin 20 —
  `OnChainRootHash`, `OrgPubKey`, `Epoch`, `OrgAdmin`): realises RC-sxjnx9
  against HAZ-xfg9cz. No
  hazard impact — it constrains a declaration, not a behaviour — but it is the
  invariant every fixed-width copy in the decoder is written against: 20 + 32 +
  32 + 8 is why a slot blob is 96 bytes at all, and a width that drifted from
  the ABI would make `decode_org_state`'s exact-96 guard right about the wrong
  number. Minted at review round 1 (finding-1), which found that
  `on-chain-client/src/types.rs` still held two `#[cfg(test)]` tests after the
  relocation this change claimed was complete — invisible to `check-trace.sh`,
  in a file explicitly in scope, with the gate's own `lib 2` line as the
  unexamined proof. The width test states a real ABI invariant and now has an
  item; the `Display` test relocated beside it without one, which is legal —
  requirements need tests, tests do not need requirements. What its gate
  actually reaches was then measured rather than assumed: all four widths, each
  individually red under a `#[repr(align(N))]` mutation, and **not** the
  correspondence between those widths and the offsets the decoder reads, which
  the test states from `const`s of its own. Argued in full under RC-sxjnx9,
  collected as the fourth of the nine claims held by review, and proposed as
  not-minted control 17. Two further corrections at review round 2. **Its
  wording contradicted this unit's glossary** (finding-13): it said the software
  wraps "each value it reads from an Organisation slot" and then listed the
  Organisation admin, which is the key an Organisation slot is *keyed on* and
  not a value read from it — `get_org_state` takes it as an argument — and it
  called the third field an "Organisation public key", a term neither
  `on-chain-client/docs/CONTEXT.md` nor the root `docs/CONTEXT.md` defines;
  both call that field the Organisation's **signing key**. It is reworded to
  both glossaries. **And it carries an abnormal-input exemption** (finding-5),
  recorded rather than answered with an invented case: it constrains a
  declaration and has no runtime input domain, so there is no input to malform.
  Its evidence reds by mutating the type, which is the only thing there is to
  mutate, and that is measured above.

  **Note, 2026-10-05:** the sentence above saying both glossaries call the
  field the Organisation's signing key was true when it was written and is
  not true now. The owner ruled (2026-10-04) that the field is the
  Organisation's X25519 key-agreement key and never a signing key. The root
  `docs/CONTEXT.md` now defines *Organisation public key* for it, and both
  glossaries use that term. REQ-54txzh supersedes REQ-2qa5r5 with that term
  and is assessed in `on-chain-client/docs/risk/2026-10-05-organisation-public-key.md`.
  This assessment stays as the record of REQ-2qa5r5.

assesses: REQ-2qa5r5

- **REQ-9wwenn** (refuse an epoch whose value does not fit the representable
  range rather than truncating it): realises RC-sxjnx9 against HAZ-xfg9cz. No
  hazard impact. It is unreachable through the contract as written — the epoch
  is a `+1` counter from 1 — which is exactly why it is stated: observing it
  means something upstream is not what this register believes.

assesses: REQ-9wwenn

- **REQ-ntn4ss** (report a best head **whose hash differs from the last
  processed head's** and which replaces it as a reorganisation carrying the
  discarded head's hash and number): realises
  RC-kemv75 against HAZ-xd4urb. Its hazard impact is the limit named in that
  residual: it reports what it can derive, and a reorganisation deeper than the
  notification gap is not derivable (PR-qpp28h). The requirement is worded to
  the decision the rule makes, not to a guarantee the notifications do not
  support. **The hash qualifier was added at review round 3 (finding-4), and it
  is a correction rather than a refinement.** Without it the requirement said a
  best head "at or below" the last processed one is reported as a
  reorganisation, which the software does not do and which contradicted
  REQ-gr2ver immediately below it — two requirements giving different answers
  for one input. `scan_step` tests hash equality first
  (`on-chain-client/src/client.rs:547-549`): a re-notification of the *same*
  head takes the dedup arm and yields `ScanStep::Skip`, whatever its number,
  which is exactly what
  `repeated_hash_with_inconsistent_number_is_still_skipped` in
  `on-chain-client/tests/best_lane_reorg_rule.rs` asserts. Only a head whose
  hash differs reaches the reorg predicate at `:551-556`. No case annotated to
  this requirement supplies a head whose hash equals the last processed head's —
  the two that do belong to REQ-gr2ver — and one supplies no last head at all,
  so the reworded requirement matches all six of them and now agrees with
  REQ-gr2ver on the input the two used to answer differently. **Reworded again
  at review round 4 (finding-1)**, which found the residue round 3 left: a
  closing clause reading "no Reorg notification for a best head that **extends**
  the last head it processed", on a word neither glossary defines and which
  REQ-5zux82 was using in the opposite sense. The clause settled nothing the
  first clause did not, so it is dropped, and the first clause now says "when
  and only when" so that the negative cases it always governed are explicit.
  All six cases still match, the two that assert `reorged: None` included —
  `child_of_the_last_head_reports_no_reorg`, which the condition excludes, and
  `the_first_notification_reports_no_reorg`, which the reworded clause excludes
  explicitly by requiring a head already processed.

assesses: REQ-ntn4ss

- **REQ-gr2ver** (do not report a head already processed a second time):
  realises RC-kemv75 against HAZ-xd4urb. Introduces one situation, and it is the
  reason the dedup is on hash and not on number: a genuine replacement block at
  a height already seen must **not** read as a repeat, and a dedup keyed on
  number would discard a depth-1 reorg's replacement entirely. That mutation was
  applied and watched, and the requirement is worded on the hash for that
  reason.

assesses: REQ-gr2ver

- **REQ-5zux82** (read every height between the last processed head and the
  new one, in ascending order, and report the observations found there):
  realises RC-kemv75 against HAZ-xd4urb.
  Introduces the unbounded first span assessed in that residual (PR-uq5r97): the
  seed is the finalised head, so the first notification's span is as long as the
  chain's finality lag. The ascending order is not decoration — a span computed
  the other way round iterates zero times and reads nothing while reporting
  success. **It said "report a Best-block observation for every height" until
  review round 3 (finding-4), and that was a property the software does not
  have.** A height carrying no `OrgRegistry` event produces no notification at
  all — `events_in_best_block` returns whatever the block held, which is
  routinely nothing (`on-chain-client/src/client.rs:605-620`) — and the gated
  evidence never asserted otherwise: every case in
  `on-chain-client/tests/best_lane_reorg_rule.rs` annotated to this requirement
  asserts the **span**, the `from` and `to` of a `ScanStep::Block`, because
  `scan_step` is the only part of the rule a test without a chain can reach.
  Reworded to the span the rule computes and the reading the lane does with it,
  which is what `for number in from..=to` performs (`:399-412`), all five cases
  still match. **Reworded once more at review round 4 (finding-1 and
  finding-7)**, on wording and nothing else. The carve-out said "when the new
  head does not **extend** the last one", an undefined term REQ-ntn4ss was using
  in the opposite sense; it now names the two conditions `scan_step` actually
  branches `from` on (`:557-560`) — the new height at or below the last
  processed head's, and no head processed yet — and all five cases still match,
  the two `None`-seeded ones in `the_backfill_span_is_always_ascending` now
  covered by text rather than by silence. The closing clause also now says that
  REQ-9vwcwc and REQ-nygs7k govern where they meet it: this requirement says
  which heights are read and reported from, those two say which events reach a
  subscriber, and the specific rule wins.

assesses: REQ-5zux82

The expectation addressed to this unit, REQ-ysyu9g, is not a requirement of this
unit and acquires no assessment here. What this change adds to it is the
confirmed fact recorded under "The obligation this change does not meet", the
hazard it stands against on the consumer's side, and its deadline of 2026-12-05.
