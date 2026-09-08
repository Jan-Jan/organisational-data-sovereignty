# Dependency edges — implementation plan and dependency assessments

**Goal:** Declare the four `depends_on:` edges the code already has, each
preceded by the dependency assessment `grill-requirements` prescribes, so that
the class floor, the impact set, the export surface and the expectation gates
have something to read.
**Implements:** REQ-ysyu9g, REQ-q92yac (both `expects:` items — unmet by
construction at this change, exempt from MISSING-TEST while unmet; the
verifying test is written by the consumer when the provider delivers)
**Safety class:** C in every unit (`docs/adr/2026-09-05-units-and-per-unit-classes.md`);
no per-item override.
**Verification:** every unit's `verify_commands`, per unit over
`check-units.sh --impact master..HEAD`, plus `check-units.sh`,
`check-trace.sh` and `check-ids.sh` per unit, `check-review.sh --branch
worktree-guardrails-edges`.

Tooth 2 of `docs/plans/2026-09-05-ratchet-gap-analysis.md`. Change branch
`worktree-guardrails-edges`, opened 2026-09-06 off `master` at `e1787af`.
Baseline before any edit: all seven `verify_commands` across the four units
exit 0 (org-members 109 passed; on-chain-client 23 passed and three fuzz
targets; org-node 35 + 1 + 1 + 1 passed; app compile proof and svelte-check
0 errors).

## The interview

One question at a time, each with a recommendation. Facts were looked up, not
asked.

| Question | Answer | Written to |
|---|---|---|
| Which of org-members' twelve requirements become `exported: yes`? | **All twelve.** The SRS header already declares them the contract org-node and the app trace into; exporting permits references and obliges nothing. | `exported: yes` under each item in `org-members/docs/requirements/2026-08-31-org-membership.md` |
| Do the two behaviours the assessments found become `expects:` items now? | **Both.** | `org-node/docs/requirements/2026-09-06-dependency-expectations.md` |

Not asked: the class floor (all four units are C, `MISCLASSED-DEPENDENCY` has
nothing to convict) and segregation (nothing to segregate). Not asked either:
whether the app records expectations — it has no requirements ledger entries
at all, and an expectation is a requirement; its gaps are listed under edges 3
and 4 for the app's own requirements change.

## Dependency assessments (D10)

Each edge walked against the provider's five artefact classes. "Exports" is
what `check-units.sh --exports <provider>` printed before this change: nothing,
for every provider, because no `exported: yes` existed anywhere.

### Edge 1 — org-node → org-members

- **Export surface.** None before this change; all twelve requirements after.
  What org-node uses, by file under `org-node/src/` (`verify.rs`, `service.rs`,
  `envelope.rs` — the trie, root computation, `apply_delta`, `verify_against`
  and the change-set type, and in `service.rs` also the member, leaf, device-key
  and member-as-a-group-key types it constructs on the join, admin-bootstrap and
  self-delete paths; `chain.rs` and `chain_read.rs` — `RootHash`;
  `error.rs` — the error enum; `keys.rs` and `transport/endpoint.rs` — the
  device and member-as-a-group key types; `test_fixtures.rs` — test-only
  constructors). The behaviours it leans on are REQ-4umsuz (base matching and
  expected-root matching), REQ-avmu3j (no root for an uncomputed record),
  REQ-d3prca (immutability of a published record), REQ-shk82j (re-validation of
  wire input) and REQ-ds8ryr (no panic). All on the export surface now.
- **RMF.** `org-members/docs/risk/2026-09-02-membership-hazards.md` scopes in
  "the verify-and-commit path in `org-node`" explicitly, so org-node's use is
  an analysed situation. The register places one responsibility on its
  consumer — the independence of the expected root — which is org-node's own
  future requirement, not something org-members owes. It also records a gap
  org-node is exposed to: the rotation control holds on the direct API and is
  bypassed on the wire path (not-minted control 2). **Gap → REQ-q92yac.**
- **ADRs.** Three (`docs/adr/`): class B, class C, units. None forecloses
  anything org-node needs.
- **Open problem reports.** Two. The device-removal one names two direct-API
  operations org-node never calls; its wire-path counterpart is REQ-q92yac
  above. The chain-reader one is about org-node's own code and sits on edge 2.
- **SOUP.** `org-members/docs/architecture/soup.md` is an empty table. A
  transitive gap for every consumer (org-node inherits org-members' SOUP), and
  a documentation gap, not a behaviour anyone owes: tooth 4.

### Edge 2 — org-node → on-chain-client

- **Export surface.** Empty; on-chain-client has no requirements. What
  org-node uses (`org-node/src/chain_read.rs`, `preflight.rs`, `service.rs` —
  `from_client` inside `connect_chain_client` — `ceremony.rs` — `h160_of` —
  and `bin/preflight.rs`): `OrgRegistryClient::get_org_state`, the `OrgAdmin`
  and `OrgState` types, the H160 helper.
- **RMF.** README only. on-chain-client's decode and event lanes are analysed
  in org-members' register (its scope paragraph names them), not in the
  provider's own.
- **ADRs.** None specific to the crate.
- **Open problem reports.** None in its ledger. The finality doc-comment
  problem filed in org-members' ledger is the evidence that the one behaviour
  org-node leans on — `at = None` reads the latest finalised block — exists
  only as a doc-comment. **Gap → REQ-ysyu9g.**
- **SOUP.** Empty table; the crate carries subxt, jsonrpsee, smoldot, scale
  codec — the heaviest SOUP in the repository, undocumented. Tooth 4.

### Edge 3 — app → org-node

- **Export surface.** Empty before and after this change — org-node's two
  new items are `expects:` items, which `--exports org-node` does not list;
  nothing is exported. What the app uses
  (`app/src-tauri/src/commands.rs`, `state.rs`): `OrgService`, `ChainOps`,
  `PersonaStore`, `TransportMode`, the join-request blob, `OrgId`, `OrgState`,
  `OrgNodeError`, `SubxtChainOps`.
- **RMF, ADRs, problems, SOUP.** README-only RMF; no ADR; no problem report of
  its own (the one about its chain reader is filed in org-members' ledger
  until tooth 3); empty SOUP.
- **Gaps for the app's own requirements change** (not expectations yet —
  the app has no SRS items to carry them): identity confirmation before an act
  on a named member, and confirmation before one-step isolation, are the two
  controls org-members' register says only the administrator's surface can
  discharge. Both are app requirements, not expectations on org-node.

### Edge 4 — app → on-chain-client

- **Export surface.** Empty. The app has no direct API use of
  on-chain-client — `app/src-tauri/src` imports nothing from it. The edge
  exists as a Cargo manifest dependency only (`app/src-tauri/Cargo.toml:30`,
  `default-features = false, features = ["dev-rpc"]`): the app ships the
  crate's object code and selects its feature set, and the client itself is
  constructed inside org-node's `connect_chain_client`. Declaring the edge
  remains correct — a shipped dependency is a dependency (D12).
- **RMF, ADRs, problems, SOUP.** As edge 2. No gap found beyond SOUP.

### Class floor and segregation

All four units are class C. `MISCLASSED-DEPENDENCY` has nothing to convict on
any edge; no `segregated_from:` is written.

## Tasks

A documentation-and-configuration change. There is no code and no test to
write; the "test" of each task is the gate that reads its output, run in the
change worktree. Tasks were executed by the author in the change worktree in
the order below; the independent review at `merge-change` 6a is what
verifies them.

### T1 — Export org-members' requirements

**Files touched:** `org-members/docs/requirements/2026-08-31-org-membership.md`
**Parallel:** no (serial, first)

Add `exported: yes` on its own line inside each of the twelve item blocks,
directly after `satisfies: derived`. An amendment to existing items, edited in
the dated file that defines them; no definition moves. Expected:

    $ .guardrails/scripts/check-units.sh --exports org-members | wc -l
    12
    $ GR_CONFIG=org-members/.guardrails/config.yaml .guardrails/scripts/check-trace.sh
    ... scope: unit org-members; foreign 0, reverse 1 ... exit 0

### T2 — Declare the edges

**Files touched:** `org-node/.guardrails/config.yaml`, `app/.guardrails/config.yaml`, `.guardrails/units.yaml`
**Parallel:** no (serial, after T1)

Uncomment `depends_on:` in both consumer configs with the two providers each;
replace the "no edges yet" paragraph of the manifest header. Expected:

    $ .guardrails/scripts/check-units.sh
    units: 4, disclaimed 7; tracked paths 360
    $ .guardrails/scripts/check-units.sh --impact master..HEAD   # after commit
    org-members	touched
    on-chain-client	touched   # if touched
    org-node	touched
    app	touched

### T3 — Write the two expectations and their derived assessment

**Files touched:** `org-node/docs/requirements/2026-09-06-dependency-expectations.md`, `org-node/docs/risk/2026-09-06-dependency-expectations.md`
**Parallel:** no (serial, after T2 — `expects:` on an undeclared edge is UNDECLARED-DEPENDENCY)

IDs minted with `.guardrails/scripts/new-id.sh --unit org-node REQ 2`:
REQ-ysyu9g, REQ-q92yac. Each carries `expects: <provider>`, `opened:
2026-09-06` and `satisfies: derived` at column one in its block; the risk
draft mentions both IDs (UNANALYZED-DERIVED). Neither file may name a
provider's HAZ, RC or PR by ID — the first draft did and drew seven
`NON-EXPORTED-REF` findings, corrected by citing file and description
instead. Expected:

    $ GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-trace.sh
    UNMET-EXPECTATION on-chain-client: REQ-ysyu9g (open 0 days)
    UNMET-EXPECTATION org-members: REQ-q92yac (open 0 days)
    checked: REQ 2, HAZ 0, RC 0, SDD 0, LLR 0, PR 0
    ...
    expectations: open 2, oldest 0 days; limits age 90, open 10
    exit 0

and on each provider's run, `expectations against this unit: 1 open`.

### T4 — Glossary

**Files touched:** `docs/CONTEXT.md`
**Parallel:** yes (with T3)

The two expectations use *Organisation state* and *Finalised block*, neither
defined. Both are interface terms (they appear in `expects:` items), so they
go in the root glossary, not org-node's.

### T5 — Records

**Files touched:** `docs/plans/2026-09-05-ratchet-gap-analysis.md`, `docs/plans/2026-09-05-ratchet-setup.md`, this file
**Parallel:** yes (with T3)

Tooth 2 marked done with a pointer here; the setup checklist gains the two
expectation deadlines (2026-12-05, 90 days) as owner items.

## Consequences worth knowing

- **A change to org-members now runs org-node's and app's suites at merge**
  (`--impact` adds transitive dependents). A change to on-chain-client runs
  both too. That is the coverage the single-unit config never had.
- **The two expectations are warnings today, failures from 2026-12-06** (the
  first run at which the age exceeds 90 days; 2026-12-05 is the last day they
  pass). They
  become failures earlier the moment org-node's own risk analysis marks either
  `(implements: RC-…)`. Delivery is an exported REQ in the provider carrying
  `satisfies: REQ-ysyu9g` / `satisfies: REQ-q92yac`, with a test.
- **Tooth 3 is now unblocked.** The chain-reader problem report and the
  org-node hazards can move into org-node's ledgers once org-node's risk
  analysis exists to receive them — the edge they needed is declared.

## Not in scope

Any code change; any test annotation; moving any item between units;
app requirements; on-chain-client requirements; SOUP tables; the
`segregated_from:` key (nothing to segregate).
