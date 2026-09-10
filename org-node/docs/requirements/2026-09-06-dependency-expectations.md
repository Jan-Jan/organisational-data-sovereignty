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
it names the hazard each item above was written toward: REQ-ysyu9g stands
against HAZ-tawvm2, a Change set accepted on its sender's word, and REQ-q92yac
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

**REQ-ysyu9g**: The software shall rely on on-chain-client to return, when
asked for an Organisation's state without naming a block, the Organisation
state recorded at the latest Finalised block, so that the Membership root a
Change set is verified against is never one that a chain reorganisation can
later discard.
expects: on-chain-client
opened: 2026-09-06
satisfies: derived

Why this is an expectation and not org-node's own requirement. org-node's
verify-and-commit path applies a Change set under REQ-4umsuz's rules and then
compares the recomputed Membership root against a root obtained from the
chain, and that comparison is only worth anything if the chain read is one the
chain will not revoke. The read is performed by on-chain-client, which
documents the behaviour in a doc-comment on `get_org_state` ("`at = None`
reads at the latest finalised block") and nowhere else: on-chain-client has no
requirements ledger entry stating it, and `org-node/src/chain_read.rs`
documents the same call as "current best" — the contradiction filed as an open
problem report in `org-members/docs/problems/2026-09-02-chain-reader-finality-doc.md`.
(Corrected 2026-09-10, as part of the 2026-09-09 correction above: that report
moved with this change into org-node's own ledger, under D14, and now lives in
`org-node/docs/problems/2026-09-09-org-node-problems.md`. The org-members file
named above is emptied and defines no item; it is kept only so its ledger's
history reads chronologically.)
A doc-comment is not a commitment. This item is what makes it one, on the
provider that performs the read.

What org-node owes in return, and does not state here: that the root it
compares against came from this read and not from the envelope or its author.
That is the "independent trusted root" responsibility org-members' hazard
analysis (`org-members/docs/risk/2026-09-02-membership-hazards.md`, the
stale-or-divergent-record hazard) places on its consumer. It is org-node's own
requirement, written from org-node's own risk analysis, not an expectation on
anyone.

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
`org-members/docs/problems/2026-08-31-device-removal-key-check.md`. org-node
is the unit that takes the wire path, so the gap is org-node's exposure, and
this item is how a consumer states a need on its provider without filing it
as the provider's anomaly. org-members may meet REQ-q92yac either by amending
REQ-ewdg2q in place with `satisfies: REQ-q92yac` plus a wire-path test, or by
writing a new exported requirement — both are the provider's choice.
