# Dependency expectations — what org-node relies on its providers for

First items in org-node's requirements ledger. Both are **expectations**
(`expects:`, guardrails D10): behaviour org-node relies on that neither
provider states as a requirement of its own. Each was found by the dependency
assessment recorded in `docs/plans/2026-09-06-dependency-edges.md`, made when
the edges org-node → org-members and org-node → on-chain-client were declared
in `org-node/.guardrails/config.yaml`.

An expectation is **met** when the provider defines an exported requirement
carrying `satisfies:` naming the item below. Until then org-node's own gate
reports it as `UNMET-EXPECTATION` on every run and exempts it from
`MISSING-TEST`, because no test can verify a behaviour the provider has not
committed to; the provider's run counts it among the open expectations
standing against it. Once met, the item is an ordinary requirement again and
the test that verifies it is org-node's integration test against the
provider's real behaviour, not a mock.

Both are `satisfies: derived`: they arise from how the system is partitioned
into units and what org-node's verify-and-commit path was built to lean on,
not from a written system-needs document. The assessment is
`org-node/docs/risk/2026-09-06-dependency-expectations.md`.

Neither implements a risk control of org-node's: org-node has no risk analysis
yet (tooth 3 of `docs/plans/2026-09-05-ratchet-gap-analysis.md`). When that
analysis is written, both items are candidates for `(implements: RC-…)`, and
an unmet expectation that implements a control fails the run outright — which
is the intended escalation.

(Corrected 2026-09-09: that register now exists —
`org-node/docs/risk/2026-09-09-org-node-hazards.md`, written as tooth 3 — and
it names the hazard each item above was written toward: the finalised-block
expectation (now held by org-io, see below) stands against HAZ-tawvm2, a Change set accepted on its sender's word, and REQ-q92yac
against HAZ-vxabf9, a device removed from the record that keeps acting as a
member. Neither item was annotated `(implements: RC-…)`, and the withholding
is deliberate for the reason this paragraph gives: an unmet expectation that
implements a control fails this unit's gate on every run, and the providers'
ninety days run to 2026-12-05. The register records the two hazards and the
due date in place of the annotation.)

A note on citations. This file names org-members' requirements by ID where it
needs to, because those are exported; it does not name org-members' hazards,
risk controls or problem reports by ID, because only requirements can be
exported and a consumer's reference to anything else is `NON-EXPORTED-REF` on
every run. Where the text below leans on one of those it says which file to
read.

## The independent trusted root

*Moved 2026-10-07 to org-io's requirements ledger
(`org-io/docs/requirements/2026-10-08-org-io.md`, finalised
at the merge of change `worktree-org-io-create`) by ruling A: the expectation
that on-chain-client reads the latest Finalised block when no block is named
is now held by org-io, which performs the chain read, with its opening date
(2026-09-06) unchanged. org-node reads no chain: it takes the Organisation
state as a value. Its assessment moved with it.*

## Device removal on the wire path

**REQ-q92yac**: The software shall rely on org-members to reject a Change set
that removes a Device key from a Member without replacing that Member's
Member-as-a-group key, so that a Change set received from outside the process
cannot leave a removed device able to derive access from the key it held
while enrolled.
expects: org-members
opened: 2026-09-06
satisfies: derived

Why this is an expectation. org-node never calls org-members' direct
device-removal operations; every membership change it applies arrives as a
Change set and goes through `apply_delta`. REQ-ewdg2q is worded broadly
enough to cover the wire path — "in the same operation that removes a device
key", with no restriction to the direct API — but it is tested, and is scoped
by its risk control, on the direct API only; org-members' own hazard analysis
records that the control behind it holds only there and is bypassed on the
wire path — its "controls identified but not minted" list carries this as
item 2, *relate a leaf's device set to its member-as-a-group key in delta
validation*. Whether the wire path is inside REQ-ewdg2q or outside it is
exactly the ambiguity this expectation resolves from the consumer's side. The
direct-API half is tracked as an open problem report in
`org-members/docs/problems/2026-08-31-device-removal-key-check.md` (corrected
2026-10-03: resolved — the direct API now refuses an unchanged replacement
key; the wire path this item is about is unchanged). org-node
is the unit that takes the wire path, so the gap is org-node's exposure, and
this item is how a consumer states a need on its provider without filing it
as the provider's anomaly. org-members may meet REQ-q92yac either by amending
REQ-ewdg2q in place with `satisfies: REQ-q92yac` plus a wire-path test, or by
writing a new exported requirement — both are the provider's choice.
