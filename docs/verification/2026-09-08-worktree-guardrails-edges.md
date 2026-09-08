# Verification — guardrails-edges (2026-09-08)

One record per change, written at `merge-change` step 6b and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line, so this file is the evidence that travels with the change.

branch: worktree-guardrails-edges
reviewer: fresh general-purpose subagent per round, no chat history, own nested review worktree (`.worktrees/worktree-guardrails-edges-review`, created and removed each round), four rounds; every round ran every guardrails gate and both quint typechecks itself and rested on the same-tree gate dispatch for the cargo and npm suites, the diff containing no source or test file
verdict: PASS — round 1 PASS WITH FINDINGS (six, all prose inaccuracies in the plan, the manifest header and one SRS paragraph), round 2 PASS WITH FINDINGS (two: an incomplete file list, and a disposition that only this record could close), round 3 PASS WITH FINDINGS (one residual in the same file list), round 4 PASS with no findings; every finding below carries its disposition
reproduced: not a defect fix — a dependency declaration. What was reproduced is the boundary the edges exist to make legible: on the first draft of org-node's two ledger files, which cited org-members' hazards, controls and problem reports by ID, org-node's `check-trace.sh` reported seven NON-EXPORTED-REF findings (HAZ-h58jn6, HAZ-s39gbh, HAZ-y8h835, PR-hvg2dy, PR-zz4exm, RC-9z65hw, RC-mqtks7 — only requirements cross a unit boundary), and before the edges were declared the same references would have read UNDECLARED-DEPENDENCY, as measured on 2026-09-05. Also reproduced: before `exported: yes` existed, `check-units.sh --exports org-members` printed nothing

Change: declare the four `depends_on:` edges the code has (org-node →
org-members, on-chain-client; app → org-node, on-chain-client), each preceded
by the dependency assessment `grill-requirements` prescribes; export all twelve
org-members requirements; record org-node's first two requirements as
`expects:` items on its providers. Branched from `master` at `e1787af`
(`origin/master` = `2bb1c21`, strictly behind local `master`, fetched
2026-09-06 with nothing to merge).
Plan: `docs/plans/2026-09-06-dependency-edges.md` (tooth 2 of
`docs/plans/2026-09-05-ratchet-gap-analysis.md`).

units touched: org-members (SRS amended), org-node (config, two new ledger
files), app (config); on-chain-client is touched only through the root
`.guardrails/units.yaml` header, which maps to every unit.
impact set (`check-units.sh --impact master..HEAD`, exit 0):
org-members touched; on-chain-client touched; org-node touched; app touched.
Units run: all four, gates and `verify_commands`, at both gate rounds.

## The gate

Every figure derived from the tree under test by a fresh gate dispatch, five
times: `a80455a` (round 1, before finalization), `bf1e852` (round 2, after
it), `dc3cbcf`, `fc288c4` and `30cc12e` (rounds 3–5, after each review
round's corrections). Logs kept outside the tree under the session
scratchpad as `edges-gate-round<N>.log`. Every round reported the same counts
— the corrections were prose in documents no suite reads — and the figures
below are round 5's, on the tree the squash carries. Cargo ran with a
scratch `CARGO_HOME`; nothing was installed.

| Gate | Result |
| --- | --- |
| `check-units.sh` (repository-level) | exit 0 — units 4, disclaimed 7, tracked paths 361 |
| `check-units.sh --impact master..HEAD` | exit 0 — four units, all `touched` |
| `check-units.sh --exports org-members` | exit 0 — twelve REQ, all in `2026-08-31-org-membership.md` (was empty before this change) |
| org-members `check-trace.sh` | exit 0 — REQ 12, HAZ 8, RC 10, SDD 0, LLR 0, PR 2; UNRESOLVED-PR PR-zz4exm (8 days), PR-hvg2dy (6 days), inside limits age 30 / open 10; scope: foreign 0, reverse 1; expectations against this unit: 1 open |
| org-members `check-ids.sh` (no draft flag) | exit 0 |
| org-members `cargo test -p org-members` | PASS — 109 passed, 0 failed (fuzz_tests 6, integration_test 102, mbt_conformance 1) |
| org-members `quint typecheck` membership.qnt, protocol.qnt | PASS, PASS (exit 0; quint prints nothing on success) |
| on-chain-client `check-trace.sh` / `check-ids.sh` | exit 0 / exit 0 — all counts zero; scope: foreign 0, reverse 1; expectations against this unit: 1 open |
| on-chain-client cargo `--lib` + three fuzz targets | PASS — 23 passed, 0 failed; three fuzz targets to their 1 s default, no crash |
| org-node `check-trace.sh` | exit 0 — REQ 2, everything else 0; UNMET-EXPECTATION on-chain-client: REQ-ysyu9g (2 days), UNMET-EXPECTATION org-members: REQ-q92yac (2 days); scope: foreign 12 (the exported org-members REQs), reverse 0; expectations: open 2, limits age 90 / open 10 |
| org-node `check-ids.sh` (no draft flag) | exit 0 |
| org-node cargo `--features app,test-support --lib` + five named targets | PASS — 38 passed, 0 failed (lib 35, service_stories 1, transport_handshake 1, transport_networked 1); both fuzz targets to their 1 s default, no crash |
| org-node `quint typecheck quint/protocol.qnt` | PASS |
| app `check-trace.sh` / `check-ids.sh` | exit 0 / exit 0 — all counts zero |
| app `cargo test --manifest-path app/src-tauri/Cargo.toml` | PASS — 0 tests: a compile proof, the crate has no tests |
| app `npm --prefix app run check` | PASS — svelte-check: 166 files, 0 errors, 1 warning (tsconfig: no type definitions for `node`, pre-existing) |
| `finalize-docs.sh` ×4 (step 3) | exit 0 each; org-node renamed two files `DRAFT-worktree-guardrails-edges-dependency-expectations.md` → `2026-09-06-dependency-expectations.md` (requirements, risk); the other three units had no drafts. Three prose cross-references updated to the new names in the same commit (`bf1e852`). Round 2 (`check-ids.sh` without `--allow-draft-files`, `git ls-files org-node/docs`) confirms no DRAFT- file remains |
| `check-review.sh --branch worktree-guardrails-edges` (6c) | exit 0 — records 7, for this branch 1, findings 9, none undisposed (run once this file held all nine findings; this row was then filled in and nothing else changed) |
| Coverage, against the class target | `make coverage-org-members` exit 0: 93.50% lines / 92.20% regions (floors 92 / 91); `make coverage-on-chain-client` exit 0: 53.57% lines / 59.19% regions (floors 52 / 58, cleared by 1.6 / 1.2 points). Identical to the 2026-09-05 measurement, as expected for a change touching no source file. Statement half of the class C target only; decision coverage unmeasured; org-node and app have no coverage command |
| Working tree | clean at `a80455a` (round 1), `bf1e852` (round 2), `dc3cbcf` (round 3), `fc288c4` (round 4) and `30cc12e` (round 5), `git status --porcelain` empty before and after every run |


## Red → green

No row. Both IDs this change implements are unmet `expects:` items: by
guardrails D10 an unmet expectation cannot have a verifying test (the provider
has not committed to the behaviour) and is exempt from MISSING-TEST while
unmet — `check-trace.sh` reports each once, accurately, as
UNMET-EXPECTATION instead. No test was written, so nothing was watched red.
The verifying test is the consumer's integration test against the provider's
real behaviour, written when the provider delivers an exported REQ carrying
`satisfies:` naming the expectation; that test is what turns this table from
empty to full.

| Item | Test | Watched red |
| --- | --- | --- |
| `REQ-ysyu9g` | none — unmet expectation on on-chain-client | n/a |
| `REQ-q92yac` | none — unmet expectation on org-members | n/a |

## What was wrong, and what was built

Before this change every unit was a freestanding guardrails project sharing a
repository: `depends_on:` was commented out in both consumer configs, no
requirement anywhere carried `exported: yes`, and the four dependency edges the
Cargo manifests declare were invisible to every gate — so a change to
org-members ran org-members' suite only, the class floor had nothing to read,
and moving even one problem report across the boundary drew
UNDECLARED-DEPENDENCY both ways (measured 2026-09-05).

Built: the four edges, declared after walking each provider's export surface,
RMF, ADRs, open problem reports and SOUP (the assessments are in the plan).
org-members exports all twelve of its requirements (owner's decision: the SRS
header already declared them the contract its consumers trace into). Two gaps
the assessments found became org-node's first requirements, each an
expectation on the provider that performs the behaviour: REQ-ysyu9g (the
Organisation state read with no block named is the Finalised-block state — the
behaviour that exists today only as a doc-comment and is the subject of the
open chain-reader problem report) and REQ-q92yac (a Change set that removes a
Device key without rotating the Member-as-a-group key is rejected — the
wire-path half of org-members' not-minted control 2). Both are derived and
assessed in org-node's risk ledger; both carry `opened: 2026-09-06` and a
90-day clock. Two interface terms the items use, Organisation state and
Finalised block, entered the root glossary.

Consequences now in force: a change to org-members or on-chain-client runs
org-node's and app's gates and suites at merge; org-node's every run prints the
two open expectations, and each provider's run counts the one standing against
it; tooth 3 (org-node's own risk analysis, and the redistribution of its
hazards and problem report) has the edge it needed.

## Review

Round 1 (on `bf1e852`): PASS WITH FINDINGS, six. All six are inaccuracies in
prose — the plan's assessments, the manifest header, one paragraph of
org-node's SRS, one date — and none changes what was declared; every gate the
reviewer ran exited 0 with exactly the counts the plan predicts. The reviewer
rested on the round-1 gate summary for the cargo and npm suites (the diff
touches no source or test file) and ran every guardrails gate and both quint
typechecks itself. The corrections were dispatched into
`.worktrees/worktree-guardrails-edges-fix1` and merged onto the change branch;
round 2 below re-verified the corrected tree.

**finding-1**: Edge 4's assessment misstates what the app uses. `docs/plans/2026-09-06-dependency-edges.md:219-221` says the app calls `OrgRegistryClient::from_client` in `state.rs`. The only occurrence in `app/src-tauri/src/state.rs` is a doc-comment (line 176); the actual call is `org-node/src/service.rs:468` inside `connect_chain_client`, which `state.rs:265` calls. `app/src-tauri/src` contains no `on_chain_client` import at all; the edge exists as a Cargo dependency only (`app/src-tauri/Cargo.toml:30`, `default-features = false, features = ["dev-rpc"]`). Declaring the edge is still right (the app ships the crate's object code; D12), but the assessment records a call that does not exist and omits the real content of the edge (a feature-flagged manifest dependency with no direct API use).
disposition: confirmed by `grep on_chain_client app/src-tauri/src` (no import; one doc-comment). The edge-4 bullet now states that the app has no direct API use of on-chain-client, that the edge is the manifest dependency at `app/src-tauri/Cargo.toml:30` selecting the crate's feature set, and that the client is constructed inside org-node's `connect_chain_client`. The edge stays declared. No test — a plan correction.

**finding-2**: Edge 2's file list is inaccurate. `docs/plans/2026-09-06-dependency-edges.md:187` names `chain_read.rs`, `preflight.rs`, `chain_write/` as the users of on-chain-client. None of the five files in `org-node/src/chain_write/` references `on_chain_client`; the actual users are `chain_read.rs`, `preflight.rs`, `service.rs` (`from_client`, `connect_chain_client`), `ceremony.rs:64` (`h160_of`) and `bin/preflight.rs`.
disposition: confirmed by `grep -rl on_chain_client org-node/src`. The list now names the five files the reviewer found. Plan correction.

**finding-3**: Two statements about requirement counts / export surfaces contradict the gates. `.guardrails/units.yaml:19-20` says "the other three units have no requirements yet, so their export surfaces are empty" — org-node now has two (`checked: REQ 2`); its surface is empty because expectations are not `exported: yes`, not because it has none. `docs/plans/2026-09-06-dependency-edges.md:203-204` says edge 3's export surface is "after it, two unmet expectations and nothing else" — `--exports org-node` prints nothing; `expects:` items are not on the export surface.
disposition: both statements corrected: the manifest header now says org-node has two requirements, both expectations, and that an expectation is not an export; the edge-3 bullet now says the surface is empty before and after, because `--exports org-node` lists nothing. Confirmed by `check-units.sh --exports org-node` printing nothing on the corrected tree.

**finding-4**: Off-by-one on the failure date. `docs/plans/2026-09-06-dependency-edges.md:312` says the expectations become "failures on 2026-12-05"; `check-trace.sh:324` convicts only when age `-gt 90`, so the first failing run is 2026-12-06. The setup checklist's wording (`docs/plans/2026-09-05-ratchet-setup.md:96-100`, "before 2026-12-05 … after that date it fails") is consistent with the gate; the plan is not.
disposition: confirmed against `check-trace.sh` (`-gt "$exp_age_limit"`). The plan now reads "failures from 2026-12-06; 2026-12-05 is the last day they pass". The setup checklist was left as written.

**finding-5**: The "no item at all" claim for the wire-path rotation rule is contestable. `org-members/docs/requirements/2026-08-31-org-membership.md:79-82` (REQ-ewdg2q) requires key replacement "in the same operation that removes a device key" with no restriction to the direct API; the direct-API scoping lives only in RC-mqtks7 and the register's prose, and REQ-ewdg2q's tests (`org-members/tests/integration_test.rs:362, :384`) exercise `delete_p2p_device` only. `org-node/docs/requirements/2026-09-06-dependency-expectations.md:530` ("REQ-ewdg2q states the rotation rule for the direct API") and `:536-537` ("the wire-path half has no item at all") therefore overstate. REQ-q92yac is not redundant given the register's own scoping, but org-members may legitimately meet it by amending REQ-ewdg2q with `satisfies: REQ-q92yac` plus a wire-path test rather than writing "an exported requirement" as `ratchet-setup.md:101-103` prescribes; the org-node text should say the ambiguity exists.
disposition: accepted as stated. The REQ-q92yac prose now says REQ-ewdg2q is worded broadly enough to cover the wire path but is tested, and scoped by its control, on the direct API only; that this ambiguity is what the expectation resolves from the consumer's side; and that org-members may meet it either by amending REQ-ewdg2q in place with `satisfies: REQ-q92yac` and a wire-path test or by a new exported requirement. The setup checklist's delivery item names both routes. REQ-q92yac's text is unchanged — the reviewer agrees it is not redundant.

**finding-6**: Ledger naming convention not followed as written. Both `org-node/docs/requirements/README.md` and `org-node/docs/risk/README.md` say the dated name is the merge date assigned by `merge-change`; the two files were renamed to `2026-09-06-…` by the author's commit bf1e852 (2026-09-06) before merge, and the gap analysis marks tooth 2 "DONE 2026-09-06" pre-merge. If the merge lands on a later date the file names misstate the ledger's own rule.
disposition: accepted; the file names are left as they are and the discrepancy is recorded under Gaps below. `finalize-docs.sh` ran at `merge-change` step 3 on 2026-09-06, the day the merge was expected; the session was then interrupted for two days. Renaming committed ledger files a second time was judged worse than a two-day skew whose explanation travels with the record. The gap analysis marker no longer carries a literal date — it names the change and points at the squash commit for the merge date.

Round 2 (on `dc3cbcf`, the corrected tree): PASS WITH FINDINGS, two. The
reviewer verified each of the six round-1 corrections against the code and the
gates and found all six in place and accurate; every gate exited 0 with the
expected counts (12 exports, 2 UNMET-EXPECTATION, 1 open expectation against
each provider, nothing NON-EXPORTED). The one correction was dispatched into
`.worktrees/worktree-guardrails-edges-fix2` and merged; round 3 below
re-verified.

**finding-7**: `docs/plans/2026-09-06-dependency-edges.md` edge 1 names org-node's org-members users as "`verify.rs`, `service.rs`, `envelope.rs`, `chain.rs`", but `grep -rln org_members org-node/src` gives nine files: also `error.rs` (wraps `OrgMembersError`), `keys.rs` and `transport/endpoint.rs` (`P2pDeviceKey`/`P2pMemberKey`), `chain_read.rs` (`RootHash`), `test_fixtures.rs`. Nothing false is named and the behaviours listed are unaffected; the list is incomplete in the way edge 2's was corrected to be complete. Minor.
disposition: confirmed by the same grep. The edge-1 bullet now names all nine files, grouped by what each takes from org-members. Plan correction; the five behaviours and REQ IDs the bullet leans on are unchanged.

**finding-8**: The round-1 disposition of finding 6 relies on the verification record, which does not yet exist in the tree (`check-review.sh --branch worktree-guardrails-edges` → MISSING-RECORD). Not a defect of this diff, but the item is open until merge-change writes `docs/verification/<date>-worktree-guardrails-edges.md` with that statement; the merger should check it.
disposition: closed by this record. The ledger-date discrepancy finding 6 names is stated under Gaps below, and `check-review.sh --branch worktree-guardrails-edges` exits 0 on the tree that carries this file (gate table).

Round 3 (on `fc288c4`): PASS WITH FINDINGS, one. The reviewer verified the
finding-7 correction file by file against `grep -rln org_members org-node/src`
and re-checked edges 2, 3 and 4 against the code; every gate exited 0 with the
same counts. The one residual was dispatched into
`.worktrees/worktree-guardrails-edges-fix3` and merged; round 4 below is the
final verification.

**finding-9**: The 956746a rewrite dropped "the member and leaf types" and the new per-file grouping no longer accounts for them: `MemberId`/`MemberLeaf` are used in live code in `org-node/src/service.rs` (lines 11, 479, 493-494, 601-602, 790, 796, 958-960, 1161 — leaf construction on the join, admin-bootstrap and self-delete paths), and service.rs also constructs `P2pDeviceKey`/`P2pMemberKey` (lines 484-1333), yet the bullet credits service.rs only with "the trie, root computation, `apply_delta`, `verify_against` and the change-set type" and places the member/leaf types under `test_fixtures.rs — test-only constructors`. The export-surface inventory in `docs/plans/2026-09-06-dependency-edges.md:49-54` is therefore still incomplete for service.rs. Low severity: it is a plan-file description, no gate reads it, the exported REQ set (all twelve) is unaffected, and the behaviours listed (REQ-4umsuz, avmu3j, d3prca, shk82j, ds8ryr) remain correct. A one-clause fix ("`service.rs` additionally the member, leaf and key types") suffices.
disposition: confirmed by `grep -n "MemberLeaf::new\|MemberId::new\|P2pDeviceKey\|P2pMemberKey" org-node/src/service.rs`. The edge-1 bullet now credits `service.rs` with the member, leaf, device-key and member-as-a-group-key types it constructs on the join, admin-bootstrap and self-delete paths. Plan correction; nothing else in the file changed.

Round 4 (on `30cc12e`, the tree the squash carries): PASS, no findings. The
reviewer verified the finding-9 correction against `org-node/src/service.rs`
function by function (`admit_member` is the join path, `create_organisation`
the admin-bootstrap path, `receive_and_self_delete_if_revoked` the self-delete
path, all constructing the four types the bullet now names), confirmed that
commit `2040a56` touches nothing but that bullet, and ran every guardrails
gate and both quint typechecks itself with the counts in the gate table.
Cargo and npm suites rested on the round-4 gate summary, the diff containing
no source or test file.


## Gaps

- **The ledger files are dated 2026-09-06 and the squash landed 2026-09-08.**
  `finalize-docs.sh` ran on 2026-09-06 when the merge was expected the same
  day; the session was then interrupted for two days. The date in the file
  names is the finalization date, not the merge date; nothing else depends on
  it, and `opened: 2026-09-06` on both expectations is the date they were
  actually written, so the 90-day clock is honest. Left as is rather than
  renaming committed files a second time.
- **Both expectations are unmet, by construction.** Delivery is the provider's
  work: an exported REQ carrying `satisfies: REQ-ysyu9g` in on-chain-client,
  `satisfies: REQ-q92yac` in org-members, each with a test. Deadline
  2026-12-05 (`expectation_age_days: 90`); recorded as owner items in the
  setup checklist.
- **on-chain-client, org-node and app export nothing.** Their export surfaces
  are empty because they have no requirements; every cross-unit reference to
  them remains UNDECLARED- or NON-EXPORTED-REF until they write some.
- **The app records no expectations**, though the assessment found two
  behaviours only the app can provide (identity confirmation before acting on
  a named member; confirmation before one-step isolation). They are app
  requirements, not expectations on org-node, and the app has no SRS items yet
  to carry them. Listed in the plan under edges 3 and 4.
- **SOUP tables are empty in all four units.** Every consumer inherits its
  providers' SOUP; on-chain-client's (subxt, jsonrpsee, smoldot, SCALE codec)
  is the heaviest and is undocumented. Tooth 4.
- **Coverage** is statement/region only, in two of four units, as before.
  Decision coverage (class C) is unmeasured everywhere; org-node and app have
  no coverage command. Unchanged by this change; open in the setup checklist.
- **CI has still not run the multi-unit job shape.** Nothing has been pushed
  since the 2026-09-05 conversion. The first push exercises it, with this
  change's edges included; the impact loop in CI runs the union over `--list`,
  so the edges change nothing there.
