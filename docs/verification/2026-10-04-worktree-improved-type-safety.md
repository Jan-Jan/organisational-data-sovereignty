# Verification — parse at the system's edge (2026-10-04)

branch: worktree-improved-type-safety
reviewer: five rounds of independent subagent review at `merge-change` step 6a, one fresh reviewer per round, each given the diff, the ledgers, the ADR and the plan and no author narrative, each re-running the org-members, org-node and app `verify_commands` in its own nested worktree. The owner chose to continue after rounds 3 and 4 (each raised a `code` or `requirement` finding) and ruled to close review after round 5 once its findings were fixed.
verdict: merge. No round found a code defect in the implementation; wire bytes and roots were confirmed unchanged by every reviewer independently. Findings were test-verification gaps (circular confusable oracle, annotations naming operations a test never reached, missing abnormal cases), unrecorded observable behaviour (five error-order changes, unredacted Display/Serialize), design-ledger placement and wording, and — round 5 — a pre-existing gap: REQ-h5ret5 never stated the UTS#39 identifier-character rule `Handle::parse` has applied since 2026-05-12. The owner ruled to amend REQ-h5ret5 in this change. All 29 findings are dispositioned below.
reproduced: not applicable as a defect fix — this change is a type-level refactor plus one behaviour change (REQ-t46uad). Every new test was watched red before it passed: by the absent implementation (T2, T4), by a mutation of the guarded code reverted before commit (robustness and review-round tests), or, where named below, by a deliberately wrong expectation (weaker evidence, stated). Wire and hash stability were pinned before the refactor (T1) and held unchanged through it.

Change: every org-members signature takes and returns validated (`Handle`, `Name`, `Surname`) or tag (`MemberId`, `NodeHash`, `RootHash`, crate-internal `HeldKey`, `HandleSkeleton`) newtypes; plain types appear only where data enters the system; handle lookups take `&Handle` (REQ-t46uad); org-node parses at its edge; the rule is recorded in both units' AGENTS.md and ADR `docs/adr/2026-10-04-parse-at-the-system-edge.md`. Branched from `master` at `0f85cb9`; local master merged in at `582a0c2` (recalculate refusal).
Plan: `docs/plans/2026-10-04-parse-at-the-system-edge.md`.

Units touched: `org-members` (code, tests, ledgers, AGENTS.md, README, SOUP) and `org-node` (call sites, tests, AGENTS.md new, one problem report). Impact set, `check-units.sh --impact master..HEAD`: `org-members` touched, `org-node` touched, `app` dependent.

Base: `git fetch origin` succeeded each round; `origin/master` is an ancestor of local `master` (local 3 ahead), which this repository integrates locally, so local `master` was merged. Note for the owner: master's `582a0c2` carries no signature (`%G?` = N); it is outside this change.

Problem-ledger delta: opens PR-hqwpg9 (org-node: persona and org records print secret seeds through `Debug`), deferred to the org-node type-safety follow-up. Resolves none. Amends REQ-h5ret5 (owner ruling 2026-10-04).

## The gate

Final run (gate 10) on tree `9429d903225a59fde097b081b9bc4886ea2ccc39`, the tree after the last fix and before this record; each unit's commands read from its config as merged.

| Gate | Result |
| --- | --- |
| `check-units.sh` | exit 0 — `units: 4, disclaimed 6; tracked paths 466` (this record makes 467) |
| org-members `cargo test -p org-members` | **221 passed, 0 failed**, 1 ignored by design (`preflight_probe`) — integration 161, mbt_conformance 32, newtypes 16, fuzz 9, encoding_golden 3 |
| org-members quint | typecheck of `membership_types`, `membership`, `membership_mbt` exit 0; `quint test membership.qnt` **63 passing**; `quint run membership_mbt --invariant=mbtInv` no violation |
| org-members `make coverage-org-members` | lines **94.83%** (floor 92), regions **93.68%** (floor 91), functions 91.49%; decision coverage unmeasured (owner-accepted) |
| org-node `cargo test -p org-node --features app,test-support ...` | **57 passed, 0 failed** — lib 23, verify_against_chain 13, admission_sender 7, store_at_rest 4, service_stories 3, transport_handshake 3, wire_frame_bound 3, transport_networked 1; three bolero targets ran their budget without panic |
| org-node quint | typecheck `protocol`, `ods_instances` exit 0; `forkSafety`, `revocationSafety`, `revokedExcludedFromOrgSecret`, `tauWindow`, `convergence` no violation |
| app (dependent) cargo, 6 targets | **78 passed, 0 failed** |
| app `npm run check` / `npm run test` | 194 files, **0 errors**, 1 pre-existing warning (`@types/node`) / vitest **30 passed** |
| `check-ids.sh` (bare, per unit) | exit 0, silent, in org-members, org-node, app |
| `check-trace.sh` | exit 0 in all three; org-members `REQ 14, HAZ 8, RC 10, SDD 6, LLR 45, PR 7`; org-node `REQ 20, HAZ 6, RC 9, PR 8`; app `REQ 22, HAZ 6, RC 12, PR 6` |
| Implements map / robustness | all 15 IDs (REQ-t46uad, REQ-h5ret5, 13 LLRs) have `verifies:` tests with normal and abnormal cases; LLR-yz2gmc/LLR-v9evtu use boundary values (construction cannot fail) |
| Working tree | clean but for this record, uncommitted at the time |

Environment: `CARGO_HOME=/tmp/cargo_home_fuzz` (`~/.cargo` is read-only and lacks `finito-0.1.0`); `QUINT_HOME=<worktree>/target/quint_home` with the v0.7.0 evaluator copied read-only from `~/.quint` (GitHub fetch is blocked here, and a scratchpad `QUINT_HOME` SIGKILLs the evaluator).

**Coverage, gap accepted.** org-members meets its line and region floors; decision coverage is unmeasured and org-node and app have no `coverage_command` — the class C gaps recorded in `docs/plans/2026-09-05-ratchet-setup.md`. The owner explicitly accepted the unmeasured decision coverage for this merge on 2026-10-04. This change adds `newtypes` and `encoding_golden` to `make coverage-org-members` (owner request), widening what is measured; floors unchanged.

## Red → green

| Item | Test | Watched red |
| --- | --- | --- |
| `LLR-377ddr` | `member_record_wire_bytes_are_pinned`, `member_set_root_is_pinned` | characterization pinned on master; last hex digit flipped → failed with the real value, restored → pass |
| `LLR-377ddr`, `LLR-68tka5` | `tampered_member_record_is_refused_and_handle_is_bound_in_bytes` | `TryFrom<String>` skipping parse → tampered decode accepted; `From<Handle> for String` constant → `assert_ne!` failed; reverted |
| `LLR-377ddr` | `newtypes_encode_as_their_string` | E0432 before the types existed |
| `LLR-xzqs9r` | `handle_parse_stores_nfc`, `handle_parse_rejects_every_rule` | E0432 before `Handle` existed |
| `LLR-xzqs9r` | `handle_parse_rejects_non_identifier_characters` | `identifier_allowed` branch deleted → `accepted "a b"`; restored |
| `LLR-xzqs9r`, `LLR-w5nkbu` | `length_bounds_apply_after_nfc` | bound measured on raw input → InvalidHandle / FieldTooLong on the 192-raw/128-NFC case; restored |
| `LLR-w5nkbu` | `name_and_surname_parse_nfc_and_bound` | E0432 before `Name`/`Surname` existed |
| `LLR-w5nkbu` | `name_and_surname_try_from_match_parse` | macro `TryFrom<&str>` skipping parse → NFD stored; reverted |
| `LLR-4czn8t` | `newtype_debug_redacts` (split from T2's test) | E0432 before the types existed |
| `LLR-4czn8t` | `newtype_debug_redacts_at_the_bound_and_beyond_ascii` | newtype Debug printing the value → "Handle(josé)"; reverted |
| `LLR-4czn8t` | `decoded_member_record_debug_redacts_pii` | MemberLeaf Debug printing handle → `Debug leaked "zelda"`; reverted |
| `LLR-68tka5` | `decode_parses_newtype_fields` (now LLR-xzqs9r, LLR-w5nkbu) | E0432 before the types existed |
| `LLR-68tka5` | `decode_member_record_stores_nfc_fields` | **weaker evidence:** no src mutation can red it without breaking decode; red shown by writing the NFD expectation first (failed "josé" NFC vs NFD), then corrected |
| `REQ-t46uad`, `LLR-f3zrwd` | `handle_query_reports_invalid_absent_and_held` | E0308 "expected `&str`, found `&Handle`" before the typed lookups existed |
| `LLR-f3zrwd` | `handle_lookup_is_exact_not_confusable` | `get_by_handle` falling back to the skeleton index → `"paypa1" matched "paypal"`; reverted |
| `LLR-ub6dw9` | `handle_index_forgets_renamed_and_deleted_handles` | `update_leaf` not removing the old index entry → old handle still found; reverted |
| `LLR-5w2jx8` | `insert_rejects_confusable_handle` and four confusable carriers | `HandleSkeleton::of` made identity → failed at the ConfusableHandle assertion (independent UTS#39 oracle); reverted |
| `LLR-5w2jx8` | `genesis_multiple_members` (over-match) | `HandleSkeleton::of` constant → `Err(ConfusableHandle)`; reverted |
| `LLR-g6arcs` | `update_name_surname_stores_boundary_and_nfd_fields_only` | `with_name_surname` keeping old surname → "Smith" vs "Müller"; keeping old name → "Alice" vs 128-byte name; each reverted |
| `LLR-v3jqau` | `update_name_surname_unknown_id_leaves_trie_unchanged` | `update_name_surname` returning `DuplicateId` → "DuplicateId vs IdNotFound"; reverted |
| `LLR-c5tzyp` | `newtype_deliberate_output_shows_value`, `newtype_deliberate_output_is_nfc_form` | Display writing "[REDACTED]" → failed vs the values; reverted |
| `LLR-c5tzyp` | `newtypes_serialize_as_stored_nfc_string` | `From<Surname> for String` constant → bytes [8,"constant"] vs [5,"José"]; reverted |
| `LLR-yz2gmc`, `LLR-v9evtu` | `tag_types_construct_from_bytes` | `RootHash::from` zeroing byte 0 → failed at [0xff;32]; reverted |
| `LLR-mmst86` | `update_handle_renames_member`, `update_handle_rejects_collision`, `update_rejects_confusable_handle` | **carried, not re-reddened:** pre-existing tests rewritten to the typed API (T4); the amendment's new part (takes a `Handle`) is enforced by the compiler |
| (refactor) | full suite + `encoding_golden` | T3 (tag types, `HeldKey`) and T5 (org-node call sites) add no behaviour; guard is the unchanged suite and the pinned bytes/root; T5's red was the compile failure at four `MemberLeaf::new` sites |
| (base merge) | `member_set_root_is_pinned` | after merging master's recalculate refusal: `HashesAlreadyCalculated` on `genesis(..).recalculate()`; fixed by reading the root from genesis; `GOLDEN_ROOT` unchanged |

## What was wrong, and what was built

Invariants lived in free functions and comments: `validate_handle` returned a `String`, `MemberLeaf::with_handle` relied on "callers must validate first", the name/surname NFC-and-bound check was copied in three places, the key index was keyed by naked `[u8; 32]`, and a handle lookup answered "not found" for a string that could never be a handle. Built: `Handle`, `Name`, `Surname` (parse + TryFrom only, redacted Debug, serde via `try_from`); tag types with `new` + `From`; `HeldKey`, `HandleSkeleton`; every org-members signature typed; lookups take `&Handle` (REQ-t46uad); org-node parses at its edge; the rule in AGENTS.md (org-members, new org-node file, root index) and the ADR. Observable changes recorded in the SAD and RMF: REQ-t46uad's invalid outcome, unredacted `Display`/`as_str`/`String`/`Serialize` (LLR-c5tzyp), and five error-order changes, none affecting a control. REQ-h5ret5 now states the identifier-character rule the code already applied. Wire bytes and roots unchanged (LLR-377ddr, pinned before the refactor).

## Review

Round 1.

**finding-1**: code — `find_confusable_pair` chose its pair with `add_member` itself, so the confusable tests were circular.
disposition: rewritten to an independent UTS#39 skeleton oracle; reddens under a non-detecting `HandleSkeleton::of` (task `692e045`).

**finding-2**: code — `update_handle_rejects_invalid` and the oversized name/surname update tests never reached the operation they claimed.
disposition: renamed to `handle_for_update_rejects_invalid`, `name_for_update_rejects_oversized`, `surname_for_update_rejects_oversized` and re-annotated to the parse-level LLRs; LLR-g6arcs gained an operation-reaching abnormal test.

**finding-3**: requirement — unredacted `Display`/`From<_> for String` had no requirement; unused `Ord`; `MemberLeaf::new` error order changed.
disposition: LLR-c5tzyp minted, tested, assessed; `Ord`/`PartialOrd` removed; error order recorded in SAD and RMF.

**finding-4**: record — ADR and plan claimed `HeldKey`/`HandleSkeleton` have `new` + `From`.
disposition: ADR decision 2 and plan decision 3 corrected to the code.

**finding-5**: record — org-node AGENTS.md understated the remaining `[u8; 32]` (≈40 in three files).
disposition: corrected to 82 lines across `src/`, per file; follow-up scope widened.

**finding-6**: record — amended derived LLRs (w5nkbu, 4czn8t, ub6dw9, g6arcs) not re-assessed.
disposition: re-confirmation section with `assesses:` in the 2026-10-04 RMF.

**finding-7**: record — "validated type"/"tag type" not in CONTEXT.md; SAD headings off the defined SDD names.
disposition: glossary entries added; headings use the defined names.

Round 2.

**finding-8**: requirement — `update_handle` and `update_name_surname` error order changed, unrecorded; RMF called one change "the one".
disposition: recorded in SAD and RMF; LLR-v3jqau holds for every call the type system admits.

**finding-9**: requirement — LLR-c5tzyp said "only", but `Serialize` also emits the plain value.
disposition: LLR-c5tzyp covers serde `Serialize`; assessment extended to replication and serde logging.

**finding-10**: code — public `From<[u8; 32]>` on tag types had no requirement or test; API removals unrecorded.
disposition: LLR-yz2gmc minted and tested; removals/renames listed in the SAD.

**finding-11**: code — `handle_lookup_is_exact_not_confusable` annotated LLR-ub6dw9 but tests exact lookup.
disposition: re-annotated LLR-f3zrwd.

**finding-12**: code — `decode_parses_newtype_fields` annotated LLR-68tka5 but decodes no `MemberLeaf`.
disposition: re-annotated LLR-xzqs9r, LLR-w5nkbu; record-level tests gained LLR-68tka5; `decode_member_record_stores_nfc_fields` added (weaker red, stated above).

**finding-13**: record — ADR decision 2 named `P2pDeviceSlots` and key types as validated with `parse`, which the code does not yet have.
disposition: dated status note in the ADR; brought under the rule in the org-node follow-up (owner ruling, Option A).

**finding-14**: record — plan Implements header omitted LLR-c5tzyp.
disposition: header and self-review updated.

**finding-15**: record — SAD intro miscounted its LLRs.
disposition: intro corrected.

**finding-16**: record — "newtype" listed under _Avoid_ yet used throughout.
disposition: "Newtype" defined in CONTEXT.md as the umbrella; removed from _Avoid_.

Round 3.

**finding-17**: record — no LLR-c5tzyp test of `Serialize`; `Surname` encoding only covered indirectly.
disposition: `newtypes_serialize_as_stored_nfc_string` covers all three types.

**finding-18**: code — `Name`/`Surname` `TryFrom` never called directly.
disposition: `name_and_surname_try_from_match_parse` added.

**finding-19**: record — org-node `trie_from_snapshots` error order changed, unrecorded.
disposition: recorded as the fourth change (local store load and received first-admission snapshot).

**finding-20**: record — CONTEXT "Newtype" used "wrapper", an _Avoid_ term.
disposition: definition reworded.

Round 4.

**finding-21**: record — decode-path error order (derived `Deserialize` fails at the first invalid field) unrecorded; lists claimed completeness.
disposition: recorded as the fifth change; lists scoped to "identified by review".

**finding-22**: record — SDD-4yr9ge traced REQ-t46uad with no LLR satisfying it.
disposition: LLR-xzqs9r `satisfies: REQ-h5ret5, REQ-t46uad`.

**finding-23**: record — LLR-yz2gmc mixed member-record and hash types.
disposition: narrowed to `MemberId`; LLR-v9evtu minted for `NodeHash`/`RootHash` (placement corrected in finding-27).

**finding-24**: requirement — LLR-g6arcs's only abnormal test checked LLR-v3jqau.
disposition: re-annotated LLR-v3jqau; `update_name_surname_stores_boundary_and_nfd_fields_only` added.

Round 5.

**finding-25**: requirement — `Handle::parse` refuses non-identifier characters (since 2026-05-12), stated by no requirement.
disposition: owner ruling 2026-10-04: REQ-h5ret5, LLR-xzqs9r and RC-n2taat amended in place and assessed; `handle_parse_rejects_non_identifier_characters` added.

**finding-26**: code — no test distinguished raw from post-NFC length.
disposition: `length_bounds_apply_after_nfc` added.

**finding-27**: record — LLR-v9evtu placed under SDD-d6x85b though `types.rs` belongs to SDD-4yr9ge.
disposition: moved under SDD-4yr9ge.

**finding-28**: record — stale `validate_handle` wording in the 2026-09-17 SAD (SDD-55b2zj) and design-derived RMF.
disposition: updated to the per-field parse mechanism, dated.

**finding-29**: record — plan self-review omitted the round 3–5 tests and LLR-v9evtu.
disposition: self-review updated.

## Gaps

- Review closed by owner ruling after round 5; no reviewer read the round-5 fixes (REQ-h5ret5 amendment, two tests, three record edits). The final gate covers them mechanically.
- Decision coverage unmeasured (owner-accepted); org-node and app coverage unmeasured.
- `decode_member_record_stores_nfc_fields` was reddened by a wrong expectation, not a mutation.
- LLR-yz2gmc/LLR-v9evtu have no refused input (construction cannot fail); boundary byte patterns stand in.
- `P2pDeviceSlots` (`new`, not `parse`), key bytes constructors and org-node's 82 `[u8; 32]` lines are not yet under the rule — the org-node follow-up, with PR-hqwpg9.
- Five error-order changes are recorded as review identified them, not proven exhaustive.
- `cargo fmt --check` fails crate-wide on pre-existing code; no gate runs it.
- Two dated historical documents (`docs/plans/2026-09-15-org-members-architecture.md`, `docs/verification/2026-09-17-worktree-guardrails-org-members-arch.md`) still cite the tests renamed in finding-2; left as history.
