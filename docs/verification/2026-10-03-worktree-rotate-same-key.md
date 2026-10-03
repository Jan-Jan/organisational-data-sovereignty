# Verification — rotate refuses the current key; every key held once (2026-10-03)

branch: worktree-rotate-same-key
reviewer: four rounds of independent subagent review at `merge-change` step 6a, one fresh reviewer per round, each given the diff, the ledgers and the plan and no author narrative, each re-running the org-members and org-node suites in its own nested worktree and measuring its own mutations. Rounds 1–3 reviewed the `rotate_p2p_key` refusal and the owner's rulings; round 4 reviewed the change after the key-uniqueness extension and the merge of master's quint-connect coupling.
verdict: merge. No round found a code defect; every guard mutation each reviewer measured was caught. Round 3's requirement finding — one member could be given a key another holds — led the owner to extend the change: every key in an organisation is held once, enforced on every path. Round 4 found the extension correct (three mutations, all caught) and raised one requirement finding outside this unit, filed as org-node's PR-g7cfns. The owner then ruled (2026-10-03) to merge after the gate rather than dispatch a fifth round.
reproduced: yes — each refusal was watched fail before it existed. `rotate_p2p_key_rejects_unchanged_key` failed with `unwrap_err()` on `Ok(OrgTrie { member_count: 1, .. })`; the key-uniqueness tests first failed to build (E0599, no `DuplicateKey`), then 18 of them failed with `unwrap_err()` on `Ok`; the round-trip property caught the first draft of LLR-gjj6bx refusing a change set computed from accepted operations ("change set after DeltaUpsert(1, 1, 3) refused: Some(DuplicateKey)"); and the extended conformance generator diverged from the unchanged model (seed 0x6dd1d523, `AddDevice(b, {b,0})`).

Change: `rotate_p2p_key` refuses the member's current key (`P2pKeyNotReplaced`), and the organisation record holds every key — member keys and device keys alike — in exactly one place, refused with `DuplicateKey` on every path that builds or changes a record, `apply_delta` included. Branched from `master` at `27dab89`; master merged in at `634780b` and `32e907c` (to `5850ed7`).
Plan: `docs/plans/2026-10-03-rotate-same-key.md`.

Units touched: `org-members` (code, tests, Quint model, driver, ledgers, README) and `org-node` (test fixtures; one problem report). Impact set, `check-units.sh --impact master..HEAD`: `org-members` touched, `org-node` touched, `app` dependent.

Base: `git fetch origin` succeeded; `origin/master` is an ancestor of local `master`, which this repository integrates locally, so local `master` was merged each round.

## The gate

Final run on tree `59035450c1fb62b8337b986e12795eda6874e65c`, the tree after the last ledger edit and before this record; each unit's commands read from its config as merged.

| Gate | Result |
| --- | --- |
| `check-units.sh` | exit 0 — `units: 4, disclaimed 6; tracked paths 441` (this record makes 442) |
| org-members `cargo test -p org-members` | **194 passed, 0 failed**, 1 ignored by design (`preflight_probe`) — integration 153, mbt_conformance 32, fuzz 9 |
| org-members quint | typecheck of `membership_types`, `membership`, `membership_mbt` exit 0; `quint test membership.qnt` **63 passing**; `quint run membership_mbt --invariant=mbtInv` (1000 samples) no violation |
| org-members `make coverage-org-members` | lines **94.67%** (floor 92), regions **93.51%** (floor 91), functions 90.00%; decision coverage unmeasured |
| org-node `cargo test -p org-node --features app,test-support ...` | **53 passed, 0 failed** — lib 23, verify_against_chain 13, store_at_rest 4, admission_sender 3, service_stories 3, transport_handshake 3, wire_frame_bound 3, transport_networked 1; both bolero targets ran their budget without panic |
| org-node quint | typecheck `protocol`, `ods_instances` exit 0; `forkSafety`, `revocationSafety`, `revokedExcludedFromOrgSecret`, `tauWindow`, `convergence` no violation |
| app (dependent) cargo, 6 targets | **78 passed, 0 failed** |
| app `npm run check` / `npm run test` | 194 files, **0 errors**, 1 pre-existing warning (`@types/node`) / vitest **30 passed** |
| `check-ids.sh` (bare) | exit 0, silent |
| `check-trace.sh` | exit 0 in all four units; no STALE-PROBLEM. org-members `REQ 13, HAZ 8, RC 10, SDD 6, LLR 39, PR 5`; org-node `REQ 19, HAZ 6, RC 9, PR 7` |
| Implements map / robustness | every Implements ID (LLR-k89ahd, LLR-v6gfc7, LLR-fym7dy, LLR-gjj6bx) has `verifies:` tests with normal and abnormal cases — measured on tree `655acbf6`; the diff from there is ledger prose only |
| Working tree | clean |

Environment: every quint run used `QUINT_HOME` set to a scratch directory; `~/.quint` is deliberately read-only to agents. master's conformance preflight probed `$HOME/.quint` only, so this change makes it probe `$QUINT_HOME` when set — the directory quint itself uses; with it unset the behaviour is unchanged.

**Coverage, gap carried forward.** org-members meets its line and region floors; decision coverage is unmeasured, and org-node and app have no `coverage_command` — the class C gaps recorded in `docs/plans/2026-09-05-ratchet-setup.md`. The owner accepted the same gap for the preceding merge (`27dab89`) the same day; this record carries that acceptance forward, and the owner's signature on the squash confirms it.

## Red → green

| Item | Test | Watched red |
| --- | --- | --- |
| `LLR-k89ahd` | `rotate_p2p_key_rejects_unchanged_key` | `unwrap_err()` on `Ok(OrgTrie { member_count: 1, .. })` before the guard |
| `LLR-k89ahd` | `device_removal_never_keeps_key` (rotate arm) | "Rotate(0, Current) succeeded but kept the member's key" |
| `LLR-k89ahd` | `membership_conformance` | crate guard in, model unchanged: "Specification and implementation states diverge"; post-merge, guard removed: red 3/3 |
| `LLR-v6gfc7` | `genesis_rejects_*`, `add_member_rejects_*`, `add_p2p_device_rejects_key_held_by_another_member` | E0599 (no `DuplicateKey`), then `unwrap_err()` on `Ok` until the check existed |
| `LLR-v6gfc7` | `scenario_add_member_duplicate`, `scenario_add_device_duplicate` | each crate check disabled: red 3/3 |
| `LLR-fym7dy` | `rotate_p2p_key_rejects_key_held_elsewhere`, `delete_p2p_device_rejects_removed_devices_own_key`, `delete_p2p_device_rejects_key_held_elsewhere`, `emergency_isolate_member_rejects_key_held_elsewhere` | E0599, then `unwrap_err()` on `Ok` |
| `LLR-fym7dy` | `scenario_rotate_key_duplicate`, `scenario_delete_device_removed_key`, `scenario_isolate_duplicate` | each crate check disabled: red 3/3 |
| `LLR-gjj6bx` | `apply_delta_rejects_upsert_sharing_a_key_with_the_record`, `..._two_upserts_sharing_a_key`, `..._upsert_whose_member_key_is_its_own_device`, `apply_delta_result_refuses_keys_the_delta_introduced` | `unwrap_err()` on `Ok` before the check |
| `LLR-gjj6bx` | `apply_delta_accepts_change_set_of_delete_device_then_rotate_to_its_key`, `..._delete_member_then_add_with_its_key`, `apply_delta_accepts_member_key_freed_by_a_removed_member` | `Err(DuplicateKey)` against the first draft's base-record clause |
| `LLR-gjj6bx` | `apply_delta_accepts_change_set_of_member_key_swap`, `..._device_key_moving_between_members`, `scenario_member_key_swap` | key index made one-phase: red |
| `LLR-v6gfc7`, `LLR-fym7dy`, `LLR-gjj6bx` | `keys_stay_unique` | each check switched off: "genesis accepted a shared key", "Isolate(0, 9) succeeded with a key already held", "DeltaHandOver(2, 0) succeeded with a key already held"; round trip against the base-record clause |
| model | 20 `DuplicateKey` `run` tests; `rotateKeySameKeyRejectedTest` | QNT508 / QNT404 before the model branch |

Never red by design, and stated as such: the check-order tests (`*_reports_*_before_*`, `rotate_p2p_key_nonexistent_with_any_key_reports_id`) and the accept-normal tests, which guard against a check placed too early.

## What was wrong, and what was built

Owner rulings, 2026-10-03, after the previous change (`27dab89`) opened PR-z463w5 and PR-fzu25w: `rotate_p2p_key` refuses the exact current key; a deleted member is permanent and a deleted `MemberId` is never legitimately re-added (PR-fzu25w resolved by ruling); key history is not the software's (PR-z463w5 resolved by ruling). Review round 3 then found that one member could be given a key another member holds, and the owner decided to enforce key uniqueness in code, on every path including `apply_delta`, for member keys and device keys.

Built: `P2pKeyNotReplaced` on `rotate_p2p_key` (LLR-k89ahd amended); `OrgMembersError::DuplicateKey` and a key index carried by `OrgTrie` and `CandidateTrie`, refusing a shared key in `genesis`, `add_member`, `add_p2p_device` (LLR-v6gfc7), any key held before a key-replacing operation, the removed device's included (LLR-fym7dy), and a change set whose resulting record shares a key (LLR-gjj6bx) — two-phase, so keys may move or swap between members in one change set. LLR-gjj6bx's first draft also refused member keys held in the base record; the round-trip property showed that refuses change sets computed from accepted direct operations, and the clause was removed before merge. The Quint model mirrors every refusal and check order, and conformance now draws keys of other members so the refusals are exercised. org-node's test fixtures had used one key as the admin's member key and device key (20 tests and one fuzz target broke); they now give the admin its own device key. README "Security checks the caller MUST perform", item 11, and the doc-comments state what is refused and what — a key no longer held — remains the caller's.

Problem-ledger delta: **resolves** PR-z463w5, PR-fzu25w (org-members, by ruling); **opens** PR-g7cfns (org-node).

## Review

**finding-1**: record — (round 1) The register and SRS notes treated the rulings as closing the risk rather than moving the duty to the caller.
disposition: Reworded: the rulings transfer the checks to the caller; the residual is caller-borne and for `analyze-risks` to weigh.

**finding-2**: record — (round 1) Only re-adding a *deleted* `MemberId` is out of contract; a fresh id given an old key is in contract.
disposition: Narrowed everywhere it was stated.

**finding-3**: requirement — (round 1) The transferred duties were not where integrators read them.
disposition: README item 11 and caller-duty doc-comments on the five operations.

**finding-4**: record — (round 1) The plan claimed a coverage run without showing it.
disposition: Coverage figures recorded in the plan.

**finding-5**: record — (round 2) The SRS intro still said the rulings put the facets outside the contract.
disposition: Reworded to "moves the checks to the caller".

**finding-6**: requirement — (round 2) Caller duties named only this member's keys.
disposition: Widened to any key any removed device holds.

**finding-7**: record — (round 2) design-derived's lead-in still counted two points.
disposition: Counts three.

**finding-8**: record — (round 2) No note that the quint-connect branch's moved model must carry the new branches.
disposition: Moot — that branch merged first; this change merged master and ported its model changes into the new layout.

**finding-9**: requirement — (round 3) One member could be given a key another member holds now; no cross-member uniqueness.
disposition: Owner decision: enforce in code on every path — LLR-v6gfc7, LLR-fym7dy, LLR-gjj6bx, with the tests in the red → green table.

**finding-10**: record — (round 3) The hazard summary row omitted the caller-borne residual and the declined key-history control.
disposition: Row and not-minted item 1 corrected.

**finding-11**: record — (round 3) PR-z463w5 claimed a fix that refuses none of its sequences.
disposition: Resolution restated as by ruling, with what the later decisions do and do not refuse.

**finding-12**: record — (round 4) Two prose references still used the draft file names.
disposition: Rewritten to the dated names.

**finding-13**: requirement — (round 4) org-node derives `MemberId` from the member key, so a revoked member who rejoins with the same keypair re-adds the deleted identifier — the out-of-contract route.
disposition: Filed as PR-g7cfns in org-node's ledger, open, for the owner's decision; outside this unit's change.

**finding-14**: record — (round 4) The assessment missed `admit_member` as a second org-node path that now refuses.
disposition: Added.

## Gaps

- **A key no longer held is not refused** on any path — the software keeps no key history, by owner ruling; the caller's duty (README item 11). HAZ-s39gbh is not re-scored; the wire-path bypass (a change set removing a device and keeping the key) also stands.
- **PR-g7cfns is open**: until org-node stops deriving `MemberId` from the member key, a rejoin with the same keypair breaks the provider's contract.
- **Conformance does not reach** `genesis` with members, nor `apply_delta` beyond canonical deltas of accepted operations; the integration tests and `keys_stay_unique` carry those clauses.
- **Decision coverage unmeasured**; org-node and app coverage unmeasured.
- **No fifth review round**, by owner decision; round 4's dispositions and this record were not reviewed.
