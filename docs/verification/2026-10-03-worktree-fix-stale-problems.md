# Verification — the two stale problem reports (2026-10-03)

branch: worktree-fix-stale-problems
reviewer: five rounds of independent subagent review at `merge-change` step 6a, one fresh reviewer per round, each given the diff, the ledgers and the plan and no author narrative, each re-running the org-members and org-node suites in its own nested worktree and measuring its own mutations.
verdict: merge. Every round found the code correct: both guards and the check order are caught by mutation in every round (unit tests, the property test, and Quint conformance), and the refusal is atomic by construction. The twenty findings were all about the record or about adjacent gaps; none disputed the fix. Rounds 3 to 5 each found a further adjacent route by which a removed device keeps or regains the member key, every one now recorded as an open problem report. After round 5 the owner ruled (2026-10-03) to stop reviewing and merge rather than dispatch a sixth round, because round 5's one finding was another adjacent case, already recorded in PR-z463w5, not a defect in this change.
reproduced: yes for PR-zz4exm — `delete_p2p_device_rejects_unchanged_key`, `delete_p2p_device_last_device_rejects_unchanged_key` and `emergency_isolate_member_rejects_unchanged_key` were each watched fail with `Ok(OrgTrie { .. })` where `P2pKeyNotReplaced` was expected, before the guard existed; Quint conformance diverged ("State invariant failed") once the crate refused and the model did not. Not applicable for PR-hvg2dy, a doc-comment with no behaviour to reproduce; its facts were checked against `on-chain-client/src/client.rs`, subxt 0.50's `at_current_block` and `org-node/src/service.rs`.

Change: `delete_p2p_device` and `emergency_isolate_member` refuse a replacement member-as-a-group key equal to the current one, atomically, and the chain reader's doc-comment states the finalised read it performs. Branched from `master` at `343fd64`.
Plan: `docs/plans/2026-10-03-fix-stale-problems.md`.

Units touched: `org-members` (code, tests, Quint model, ledgers) and `org-node` (doc-comment, README, ledgers). Impact set, computed with `check-units.sh --impact master..HEAD`: `org-members` touched, `org-node` touched, `app` dependent.

Base: `git fetch origin` succeeded every round; `origin/master` is an ancestor of local `master`, which is 47 commits ahead (this repository integrates locally), so both were merged and both were already up to date each round.

## The gate

Final run (gate run 6) on tree `fba8eff91415faef058fff0395bd6bf6937881ad`, the tree after the last ledger edit and before this record. Runs 1 to 5 on the earlier trees agree on every count that did not change by design (integration 113 → 114 when round 1's finding-5 test was added).

| Gate | Result |
| --- | --- |
| `check-units.sh` (repository-level) | exit 0 — `units: 4, disclaimed 7; tracked paths 422` (this record makes 423) |
| org-members `cargo test -p org-members` | **122 passed, 0 failed** — lib 0, fuzz_tests 7, integration_test 114, mbt_conformance 1, doc 0 |
| org-members `quint typecheck quint/membership.qnt`, `quint typecheck quint/protocol.qnt` | exit 0, silent on success |
| org-members `quint test quint/membership.qnt` (extra) | **24 passing, 0 failing** |
| org-node `cargo test -p org-node --features app,test-support ...` | **51 passed, 0 failed** — lib 23, admission_sender 3, service_stories 3, store_at_rest 4, transport_handshake 1, transport_networked 1, verify_against_chain 13, wire_frame_bound 3; both bolero targets ran their 1 s budget without panic |
| org-node `quint typecheck quint/protocol.qnt` | exit 0 |
| app (dependent) `cargo test ... --features test-support` (6 targets) | **78 passed, 0 failed** |
| app `npm --prefix app run check` | 194 files, **0 errors**, 1 pre-existing warning (no `@types/node`) |
| app `npm --prefix app run test` | vitest **30 passed**, 4 files |
| `check-ids.sh` (bare) | exit 0, silent |
| `check-trace.sh` | exit 0 in all four units; no STALE-PROBLEM anywhere. org-members `REQ 12, HAZ 8, RC 10, SDD 6, LLR 35, PR 5`; org-node `REQ 17, HAZ 6, RC 9, PR 6` |
| Implements map / robustness | every Implements ID (LLR-s97ywt, LLR-w92psx, REQ-ewdg2q, REQ-r784fu) has `verifies:` tests and both normal and abnormal cases — measured at gate run 5 (tree `8dad8e82`); the diff from there to run 6 is ledger prose only, no test touched |
| Coverage, against the class C target | org-members lines **93.54%** (floor 92), regions **92.23%** (floor 91), functions 88.71%; decision coverage **unmeasured**; org-node unmeasured — accepted gap, below |
| Working tree | clean |

Environment: quint 0.33.0 needs evaluator v0.7.0 and `~/.quint` is not writable from the agent sandbox, so every quint-backed run used `QUINT_HOME` set to a scratch directory; without it `mbt_conformance` fails with "Quint returned non-zero code" at baseline too. The app's npm checks ran after `npm ci` from `app/package-lock.json` in the change worktree (the primary checkout's install lacks `vitest` and `@types/node`). The one svelte-check warning (no `@types/node`) is pre-existing: the lockfile does not include that package.

**Coverage, accepted gap.** org-members meets its line and region floors, but decision coverage is unmeasured (llvm-cov reports Branches `0/0`), and org-node has no `coverage_command`. Both fall short of the class C target and are already recorded in `docs/plans/2026-09-05-ratchet-setup.md`. The owner accepted them as a documented gap for this merge on 2026-10-03.

## Red → green

| Item | Test | Watched red |
| --- | --- | --- |
| `LLR-s97ywt` | `delete_p2p_device_rejects_unchanged_key` | E0599 (no variant), then `Ok(OrgTrie { member_count: 1, .. })` where `P2pKeyNotReplaced` was expected; green with the guard |
| `LLR-s97ywt` | `delete_p2p_device_last_device_rejects_unchanged_key` | E0599, then `Ok(OrgTrie { .. })`; green with the guard |
| `LLR-s97ywt` | `delete_p2p_device_unknown_device_with_unchanged_key_reports_device` | never red by design: a check-order guard. Red under the check-order mutation (key check before `remove_device`) in review rounds 1, 2, 3, 4 and 5 |
| `LLR-s97ywt`, `LLR-w92psx` | `device_removal_never_keeps_key` | each guard removed alone: minimised `[DeleteDevice(0, 0, Current)]` / `[Isolate(0, Current)]`, "succeeded but kept the member's key"; after the round-1 tightening, the check-order mutation: "DeleteDevice(0, 3, Current) reported P2pKeyNotReplaced for an absent device" |
| `LLR-s97ywt`, `LLR-w92psx` | `membership_conformance` | with the crate guard and no model change, "Specification and implementation states diverge"; each crate guard removed alone after the model change, red 3/3 runs |
| `LLR-w92psx` | `emergency_isolate_member_rejects_unchanged_key` | E0599, then `Ok(OrgTrie { .. })`; green with the guard |
| `LLR-w92psx` | `emergency_isolate_member_already_isolated_rejects_unchanged_key` | isolate guard removed: `unwrap_err()` on `Ok` |
| `LLR-w92psx` | `emergency_isolate_member_nonexistent_with_any_key_reports_id` | never red by design: a check-order guard (IdNotFound precedes the key check) |
| model | `deleteDeviceSameKeyRejectedTest`, `isolateSameKeyRejectedTest` | QNT508 Assertion failed before the model change |

## What was wrong, and what was built

PR-zz4exm, open since 2026-08-31 and stale at 33 days: `delete_p2p_device` and `emergency_isolate_member` stored any caller-supplied replacement key, including the current one, so a removal could leave the removed device's access intact. The owner decided on 2026-10-03 that an unchanged replacement key is an error and the operation is atomic. Built: `OrgMembersError::P2pKeyNotReplaced`, checked after the member lookup (and the device lookup, for `delete_p2p_device`); LLR-s97ywt and LLR-w92psx amended in place to state it; the Quint model `quint/membership.qnt` and the conformance driver gained the same error and order, with named `run` scenarios, because the step generator picks the current key as the replacement in about a third of removal steps; a property test; dated corrections at every living ledger statement that described the gap as open. `rotate_p2p_key` (LLR-k89ahd) was deliberately left alone: it removes no device.

PR-hvg2dy, open since 2026-09-02 and stale at 31 days: `OnChainReader::refresh()` was documented as reading the "current best" state while it reads the latest finalised block. Built: the doc-comment now states the finalised read (REQ-ysyu9g), the caching, and that production does not use this reader; README and ledgers corrected.

Problem-ledger delta: **resolves** PR-zz4exm (org-members), PR-hvg2dy (org-node); **opens** PR-z463w5 and PR-fzu25w (org-members).

## Review

**finding-1**: requirement — (round 1) The guard compares only against the current key; an earlier key the removed device held, or a later `rotate_p2p_key` back to it, still hands the device a key it held. Unrecorded anywhere.
disposition: Out of scope for this fix (the code meets REQ-ewdg2q's letter); filed as PR-z463w5 for an owner decision.

**finding-2**: record — (round 1) PR-zz4exm's resolution note cites a verification record for this branch that did not yet exist.
disposition: This record is that file; the reference resolves at merge.

**finding-3**: code — (round 1) `device_removal_never_keeps_key`'s catch-all `Err(_)` arm did not check which error, so it missed the check-order mutation.
disposition: fix-r1 asserts `DeviceNotFound` (and fails on any isolate error other than `P2pKeyNotReplaced`); the check-order mutation now reddens it.

**finding-4**: code — (round 1) The "changed nothing" re-reads of the original persistent trie cannot fail; atomicity holds by construction, not by those assertions.
disposition: Accepted in round 1 with no code change; repeated as round 3's finding-15 and fixed there.

**finding-5**: requirement — (round 1) LLR-w92psx refuses the current key for an already-isolated member, where no device is removed and REQ-ewdg2q does not reach; untested.
disposition: Test `emergency_isolate_member_already_isolated_rejects_unchanged_key` added (reddens with the isolate guard removed); the LLR rationale grounds it in REQ-r784fu's "replaces".

**finding-6**: record — (round 2) The hazard register's control summary still called RC-mqtks7's second clause unenforced.
disposition: Dated correction added.

**finding-7**: record — (round 2) PR-z463w5 named only REQ-ewdg2q; `emergency_isolate_member` has the same earlier-key gap.
disposition: `affects:` extended to REQ-r784fu, third sequence added.

**finding-8**: code — (round 2) The guard compares encoded key bytes, not curve points; a non-canonical encoding of the current key passes. Not asked to be fixed.
disposition: Recorded in PR-z463w5 as the encoding facet of "what counts as a key the device held", for the owner's decision.

**finding-9**: record — (round 3) "Met as of 2026-10-03" on REQ-ewdg2q over-claimed: the wire path and PR-z463w5 remain.
disposition: Qualified to the direct-API operations, with the open gaps named.

**finding-10**: record — (round 3) `org-node/README.md` still said "current best".
disposition: Corrected to the finalised read.

**finding-11**: record — (round 3) org-node's hazard register and dependency-expectations SRS still described the chain-reader report as open.
disposition: Dated corrections added.

**finding-12**: record — (round 3) Two org-node ledgers still called the device-removal report open.
disposition: Dated corrections added, citing the org-members file by path (cross-unit rule).

**finding-13**: record — (round 3) Four org-members statements (SRS intro, HAZ-s39gbh's P2 rationale, design-derived, decomposition) still treated PR-zz4exm as open.
disposition: Dated corrections added; P2 explicitly not re-scored.

**finding-14**: record — (round 3) The resolution note's citation of this record, repeated from finding-2.
disposition: As finding-2.

**finding-15**: code — (round 3) The tautological re-reads of finding-4, now with comments presenting them as atomicity evidence.
disposition: fix-r3 removed them and reworded the comments: atomicity holds by construction (`&self` → `Result<Self, _>`), and the evidence is the `Err` itself. Spot-check: delete guard disabled → three tests red.

**finding-16**: requirement — (round 4) `delete_member` then `add_member` with the same key and one device fewer removes a device with no key change, contradicting the register's "must go through `delete_p2p_device`". Probe test passed.
disposition: Filed as PR-fzu25w; the "met" note and the register claim corrected.

**finding-17**: record — (round 4) PR-z463w5 said "two" sequences and listed three.
disposition: Corrected.

**finding-18**: record — (round 4) Two review-fix ledgers still said PR-zz4exm "has carried" its deferral.
disposition: Corrected to the past tense with the resolution date.

**finding-19**: record — (round 4) design-derived still said PR-zz4exm "closes an unacceptable residual risk outright", contradicting the register's withdrawal of that claim.
disposition: Withdrawn in a dated correction.

**finding-20**: requirement — (round 5) A removal may install the removed device's own key as the new member key; the device controls it outright. Probe test passed.
disposition: Recorded in PR-z463w5 as its third facet. The owner then ruled to end review and merge (see `verdict:`).

## Gaps

- **REQ-ewdg2q's purpose is not met end to end.** Open and recorded: the wire path (`apply_delta` accepts a device removed with the key unchanged; the register's not-minted control 2; org-node's REQ-q92yac), earlier-key reuse, key identity by encoding, and the removed device's own key (PR-z463w5), and delete-and-re-add (PR-fzu25w). HAZ-s39gbh stays not acceptable and was not re-scored; that is `analyze-risks` work after the owner settles the key-identity requirement.
- **`rotate_p2p_key` accepts any key**, including the current one or one a device held. Deliberately untouched; covered by PR-z463w5's owner question.
- **Atomicity is structural, not tested.** No test could observe a partial update, because the API cannot produce one; the evidence is the type signature.
- **Decision coverage unmeasured; org-node coverage unmeasured.** Accepted by the owner for this merge.
- **No sixth review round**, by owner decision. Round 5's one finding was recorded, not reviewed again. No reviewer read this record.
- **The quint-connect coupling branch** (`worktree-quint-connect-coupling`) moves the model to `org-members/quint/membership.qnt`; when it merges master in, the moved model must carry `P2pKeyNotReplaced` and its three `run` scenarios.
