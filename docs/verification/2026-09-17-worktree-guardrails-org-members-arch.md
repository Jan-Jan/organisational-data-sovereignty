# Verification — org-members architecture, SOUP and LLRs (2026-09-17)

branch: worktree-guardrails-org-members-arch
reviewer: three independent review rounds, each a fresh subagent with the repository, the artifacts and no author narrative, dispatched at merge-change step 6a on 2026-09-17. Each ran the unit's verify_commands itself in its own task worktree and reported the counts it saw.
verdict: round 1 raised 13 findings and refused the merge on two; round 2 raised 7 and refused on two, one of them a product defect neither the author nor round 1 had found; round 3 raised 7, marked none blocking, and concluded "the change should merge". All 27 are dispositioned below.
reproduced: yes for every defect this record claims, each by the party that claimed it. The two review-blocking defects were reproduced by the reviewer and then independently by the author before any fix: the byte-uniqueness counterexample by byte-patching an encoded Delta from NFC to NFD and watching both strings decode, apply and verify against one root; the handle-handover refusal by building the two-rename delta and watching apply_delta return DuplicateHandle on a byte-identical base. The public panics were reproduced by execution. The LLR-tk4qxu structural claim was NOT reproduced as a failure and could not be — see Gaps.

Change: the first architecture ledger for the `org-members` unit — six software items, thirty-five low-level requirements, a measured SOUP inventory, and the unit's existing tests brought under those items. Branched from `master` at `5c0c710`.
Plan: `docs/plans/2026-09-15-org-members-architecture.md`.

Units touched: `org-members`. Impact set, computed with
`check-units.sh --impact "master..HEAD"`: `org-members` touched, `org-node`
dependent, `app` touched — `app` because this change tracks its previously
absent lockfile. `on-chain-client` is outside the set and its gates were not
run for this change.

**The base was merged from a local ref, deliberately, and this is the record
of it.** `git fetch origin` succeeded on every round, so this is not a
swallowed failure. `origin/master` is 44 commits *behind* local `master`
(`2bb1c21` is an ancestor of `5c0c710`), and local `master` is fully contained
in this branch, so step 1 had nothing to merge on any round. Step 4's duplicate
scan was therefore compared against `5c0c710`.

## The gate

Every figure below was produced by the final gate run on the merged tree, in
the change worktree, not carried forward from an earlier round.

| Gate | Result |
| --- | --- |
| `cargo test -p org-members` | 115 passed, 0 failed (0 lib + 6 fuzz + 108 integration + 1 mbt + 0 doc) |
| `quint typecheck quint/membership.qnt` | exit 0 |
| `quint typecheck quint/protocol.qnt` | exit 0 |
| `cargo test -p org-node --features app,test-support …` | 51 passed, 0 failed, + 2 libfuzzer targets clean |
| `cargo test --manifest-path app/src-tauri/Cargo.toml …` | 78 passed, 0 failed |
| `npm --prefix app run check` | 0 errors, 1 pre-existing `@types/node` warning |
| `npm --prefix app run test` | 30 passed, 0 failed |
| `check-units.sh` | units 4, disclaimed 7; tracked paths 412 |
| `check-ids.sh` (no `--allow-draft-files`, all three units) | exit 0 |
| `check-trace.sh` (all three units) | exit 0; 3 + 7 + 6 advisories, every one inside its unit's age and count limits |
| Coverage, against the class target | statement **met**, decision **UNMEASURED** — see below |
| Working tree | clean |

**Suite total: 274 passed, 0 failed, 0 skipped**, plus 2 fuzz targets and 3
quint typechecks.

**Two libfuzzer targets print no `test result` line at all.** They report
`exit reason: max duration (1s - default) exceeded` at ~190k iterations each,
with no failing input. Counted as targets passed, not as silent skips, and
named here because a reader tallying `test result` lines will otherwise come up
two short.

### Coverage, and the shortfall stated as a shortfall

```
Lines    1000 total,  65 missed   93.50%   floor 92   met
Regions  1551 total, 121 missed   92.20%   floor 91   met
Branches    0 total,   0 missed   "-"      UNMEASURED
```

Both figures are byte-identical to the 2026-08-26 baseline, which is what a
change adding five tests and no production code should do.

**The IEC 62304 class C target is statement AND decision coverage. The decision
half is not measured at all** — llvm-cov reports `Branches 0 / 0 / "-"` for
every file in the unit. That is not a pass and is not evidence of decision
coverage; it is evidence that none was taken. The shortfall pre-dates this
change, is recorded in `docs/plans/2026-09-05-ratchet-setup.md`, and is
scheduled for tooth 5, where coverage is set for `org-node` and `app` on one
measurement basis rather than piecemeal. **The owner accepted it as a
documented gap on 2026-09-17**, which is why this gate passes with it open.

## Red → green

Thirty-five LLRs, each discharged by **red by mutation**: because every
behaviour here pre-dates its test, the evidence is that a named mutation to the
source made a named test fail for the mutated reason, after which the mutation
was reverted and the file verified byte-identical by SHA-256. A test annotated
while already green proves nothing on its own, and this table is the only
durable record that none was.

The six SDD items carry no row: `check-trace.sh` has no MISSING-TEST gate for
the `SDD` prefix. Their gate is `UNTRACED-DESIGN`, clean on every run. They are
**traced, not tested**, by design.

| Item | Test | Watched red |
| --- | --- | --- |
| `LLR-xzqs9r` | `handle_uppercase_rejected` (+9 clause carriers) | uppercase check neutered in `validate_handle` |
| `LLR-5w2jx8` | `genesis_rejects_confusables`, `insert_rejects_confusable_handle`, `update_rejects_confusable_handle` | `handle_skeleton` returns its input unchanged |
| `LLR-pys2ek` | `add_p2p_device_rejects_when_full`, `member_leaf_too_many_devices`, `deserialize_rejects_too_many_devices`, `every_device_slot_reaches_the_root` | `MAX_DEVICES` 4 → 5 |
| `LLR-w5nkbu` | 7 carriers, all individually reddened | `MAX_NAME_LEN`/`MAX_SURNAME_LEN` → 129, → 256, → 127 |
| `LLR-paxj7b` | `member_leaf_empty_devices`, `member_leaf_new_rejects_empty_device_list`, `member_leaf_has_id_handle_and_key` | ≥1-device guard dropped from `MemberLeaf::new` |
| `LLR-4czn8t` | `member_leaf_debug_redacts_pii` | `Debug` prints `self.handle` |
| `LLR-xyv6p9` | 5 carriers, all individually reddened | strictly-increasing check dropped; comparison inverted; empty list rejected |
| `LLR-68tka5` | `deserialize_rejects_invalid_handle` | `validate_handle` skipped in `MemberLeaf`'s `Deserialize` |
| `LLR-72p8bz` | `hasher_domains_are_separated` | all four Blake3 contexts set to the member-leaf context |
| `LLR-kdhd2v` | `device_slot_order_does_not_change_the_root`, `every_device_slot_reaches_the_root` | `devices.sort()` removed; `MAX_DEVICES` 4 → 5 |
| `LLR-zbe553` | `member_id_bit_indexes_msb_first` | `bit()`'s `7 - (index % 8)` → `index % 8` |
| `LLR-wm5hpc` | `add_then_delete_returns_to_the_empty_root` | `smt::remove` writes a tombstone instead of the level default |
| `LLR-tk4qxu` | `insert_does_not_mutate_original`, `delete_does_not_mutate_original` | **measured negative — type-level guarantee, see Gaps** |
| `LLR-n7nya3` | `root_hash_errs_until_recalculated`, `batch_mutations_then_recalculate` | `root_hash()` falls back to the stale root; `recalculate_hashes` stops calling `set_hash` |
| `LLR-4n8zqx` | `same_members_same_root_hash`, `different_insertion_order_same_root`, `add_then_delete_returns_to_the_empty_root` | insertion counter XORed into `device_root`; `smt::remove` leaves a residue |
| `LLR-8jttpb` | `apply_delta_rejects_unsorted_removed`, `fuzz_tests::delta_canonicality_fuzz` | `diff_recursive` descends right before left |
| `LLR-ub6dw9` | `get_by_handle`, `contains_handle` | `handle_index.insert` dropped from `genesis` |
| `LLR-fv75ec` | `insert_duplicate_id_fails`, `insert_duplicate_handle_different_id_fails`, `insert_rejects_confusable_handle`, `mbt::membership_conformance` | id check dropped; skeleton check dropped |
| `LLR-ch2pkw` | 4 carriers, all individually reddened | `genesis`'s duplicate-id, skeleton and handle-index writes each dropped in turn |
| `LLR-j4d38d` | `delete_member_frees_the_handle_for_reuse`, `delete_removes_member`, `mbt::membership_conformance` | `delete_by_id` skips the index removals |
| `LLR-4phmjf` | `add_p2p_device_adds_a_device`, `_rejects_duplicate`, `_rejects_when_full` | `add_p2p_device` also replaces the key; `has_device` dropped; `MAX_DEVICES` check dropped |
| `LLR-s97ywt` | `delete_p2p_device_removes_and_rotates_key`, `_last_device_isolates`, `delete_p2p_device_unknown_device_fails` | old key kept; original slots kept; `remove_device`'s error swallowed |
| `LLR-w92psx` | 4 carriers | isolate calls `delete_by_id`; keeps slots; keeps old key |
| `LLR-k89ahd` | `rotate_p2p_key_changes_only_key` | rotate also clears the device set; rotate keeps the old key |
| `LLR-mmst86` | 4 carriers, all individually reddened | `validate_handle` skipped; rename skeleton check dropped; rename branch disabled |
| `LLR-g6arcs` | 4 carriers, all individually reddened | NFC dropped; both length bounds disabled; surname write-through removed |
| `LLR-v3jqau` | all 7 `_nonexistent_fails` carriers + `mbt::membership_conformance` | all seven presence guards return `Ok`; then all eight return `DuplicateId` |
| `LLR-au8het` | `delta_base_mismatch_fails`, `apply_delta_stale_delta_fails`, `delta_apply_and_verify` | `base_root` comparison dropped, then inverted; `recalculate` stamps the new root |
| `LLR-y38jfk` | `candidate_verify_wrong_root_fails`, `delta_apply_and_verify` | `verify_against` guard deleted; empty handle index (first clause **type-level**, see Gaps) |
| `LLR-7tdqv9` | `candidate_verify_wrong_root_fails`, `delta_apply_and_verify` | `verify_against` ignores `expected_root`, then comparison inverted ("consumed either way" is **type-level**) |
| `LLR-xmpqn2` | 8 carriers, proven per clause | increasing-removals, presence, increasing-upserts, disjointness, no-op, acceptance — six mutations |
| `LLR-juxk9q` | `apply_delta_rejects_confusable_in_upsert`, `delta_apply_and_verify` | skeleton/uniqueness block skipped; insert moved above its own check |
| `LLR-h7stq2` | 4 carriers | `is_calculated` guard dropped; diff direction reversed; `base_root := self.root_hash()` |
| `LLR-h9gs32` | `handle_validation_never_panics`, `trie_ops_never_panic_and_count_consistent`, `calculate_delta_roundtrip`, `delta_canonicality_fuzz` | `unwrap` on the duplicate path; panic in `validate_handle`'s uppercase branch |
| `LLR-sa3ugj` | `malformed_delta_error_displays_reason`, `field_too_long_error_displays_field_and_max` | `#[error]` strings drop `{0}` and `{field}`/`{max}` |

**Eleven of these rows were re-established after their evidence was lost.** The
task carrying SDD-k5wa4n's eleven LLRs stalled twice at the harness watchdog
with an entire run of mutation rounds uncommitted, and its context — the only
holder of the attestations — died with it. The annotations survived and were
verified correct by inspection (source byte-identical, so no mutation was left
live; `MISSING-TEST` 19 → 8 with exactly those eleven cleared). The
attestations were **not** assumed from the subagent's last progress message;
they were re-run from scratch in two smaller dispatches, which is what the
`LLR-ub6dw9` … `LLR-4phmjf` and `LLR-s97ywt` … `LLR-v3jqau` rows above record.

## What was wrong, and what was built

**What was wrong: the unit had 109 tests and 49 annotations, and no design
layer at all.** `SDD 0, LLR 0`. Sixty tests verified behaviour that traced to
nothing, twelve REQs had no refinement beneath them, and `soup.md` was an empty
table row — for a class C unit whose own README requires "one testable LLR
each" per item.

The SOUP gap was worse than an empty table. `org-members` declares no
`[workspace]`, so `cargo locate-project --workspace` names the repository root
and `/Cargo.lock` governs its build — and that file was gitignored, while the
tracked `org-members/Cargo.lock` was **inert**: cargo never read it, and it
recorded `blake3` 1.8.5 against the 1.8.7 that actually builds. An inventory
sourced from the tracked file would have been wrong on its first row.
`app/src-tauri/Cargo.lock` did not exist at all — the shipped binary had no
pinned dependency set of any kind.

**What was built:** six software items along the line between what a member
*is*, how the record is *hashed*, how it is *stored and changed*, and how a
change *crosses a process boundary*; thirty-five LLRs beneath them; five new
tests; the real lockfiles tracked and the inert one deleted; a SOUP inventory
whose versions are measured from the lockfile that governs the build.

**Three defects were found in the product, none of them fixed here** — this
change deliberately adds no production code, and each fix needs its own
red-first cycle:

- **`apply_delta` refuses an honest handle handover this crate itself
  produces** (PR-vf5hdm). The upsert loop walks in ascending id order and
  releases a leaf's old handle only on reaching that leaf, so a handle in
  flight between two members is still indexed when its acquirer is checked. A
  two-way swap fails in both id orders; a one-way handover fails whenever the
  taker's id sorts first. The producer makes both renames successfully,
  `calculate_delta` emits a canonical delta, and the receiver rejects it — a
  lawful membership change that cannot be replicated, with no adversary and no
  malformed input.
- **Two public methods panic on reachable input** (PR-jq43gx):
  `MemberId::bit(256)` and `DefaultHashes::at_level(300)`, both `pub` on `pub`
  types, both `u16` parameters indexing unguarded slices.
- **The `MAX_DEVICES` constant is coupled to a literal `4` by nothing but a
  test.** `to_fixed_slots` is sized by the constant, `compute_device_root` by
  the literal; they agree today by coincidence. Raise the constant and the
  fifth device key is never hashed — present in the record, returned by
  `p2p_devices()`, invisible to the root and so to every verifier.
  `every_device_slot_reaches_the_root` was written to gate exactly that.

**And one false claim was retracted across eight documents.** LLR-xmpqn2's
commentary, `delta.rs`, `trie.rs`, `soup.md`, the risk assessment, both halves
of `org-members/README.md`, and a historical spec all asserted that an accepted
change set has a *unique postcard byte string*. It does not:
`MemberLeaf`'s `Deserialize` **normalises** where `P2pDeviceSlots`' rejects, so
an NFD-encoded name and its NFC equivalent are distinct byte strings that
decode to one value and produce one root — measured, by byte-patching a real
encoded delta. The README turned the false claim into *instruction*, telling
integrators that "signatures, hashes, and replay caches all key cleanly off the
blob bytes", and the risk assessment **exported** it to `org-node`, naming
dedup, replay and signature as its consumers. Both now say what is true and
what to key on instead.

## Review

Twenty-seven findings across three rounds. Round 1 and round 2 each blocked the
merge; round 3 blocked nothing.

**finding-1**: LLR-xmpqn2 states a byte-uniqueness property the code does not have, and the risk ledger exports that property to `org-node` as the basis for dedup, replay protection and signature identity.
disposition: fixed in four sites by T11 — and found still standing in four more by round 2, then fixed in those by T12. See finding-14. The replacement states what is true (canonical form constrains the decoded `Delta`; every delta this crate produces is already canonical) and names what is false. No code changed: making the encoding injective is a behaviour change owed its own red-first cycle, recorded as such.

**finding-2**: No test in the suite ever serialises or deserialises a `Delta` — all postcard call sites handle a `MemberLeaf` or `P2pDeviceSlots` — so SDD-55b2zj's defining boundary has no end-to-end evidence, which is also why finding-1 went unnoticed.
disposition: recorded as dated Obligation D, `opened: 2026-09-17`. Verified independently by round 2 and again by round 3. Not closed here: this change adds no test beyond the five its plan names.

**finding-3**: `MemberId::bit(256)` panics through the public API, and the ledger's robustness waiver for LLR-zbe553 asserted "no caller can supply one", which is true of internal call sites and false of external ones.
disposition: PR-jq43gx filed, `opened: 2026-09-17`, code deliberately unfixed per the owner's decision of the same date. The false premise was removed from the waiver, which now stands on the deferral rather than on unreachability.

**finding-4**: `every_device_slot_reaches_the_root` detected the `MAX_DEVICES` coupling only by coincidence of a test seed — changing one helper string left the mutation green.
disposition: fixed by T11. The test now sorts a pool of `MAX_DEVICES + 1` keys and substitutes the largest, so the substitution lands in the last slot by construction. Round 3 attacked it with eight unrelated seed families including the one that defeated the original, and it reds under every one.

**finding-5**: Three uncarried `to_nfc` sites — replacing each with the identity leaves the suite green.
disposition: recorded as dated Obligation A. Round 2 found a **fourth** site the obligation had missed; see finding-17.

**finding-6**: LLR-h9gs32's rationale claims the lint posture means "no input reaches a panicking path", but the crate denies only `unwrap`/`expect`/`panic` — slice indexing and arithmetic are not denied.
disposition: recorded as dated Obligation B, with PR-jq43gx as its live instance.

**finding-7**: The SOUP advisory-database gap is disclosed in `soup.md` but tracked by no item and aged by no gate.
disposition: recorded as dated Obligation C, as a recommendation rather than a minted item — the gap is the toolchain's and is shared by all four units, so four per-unit reports may be worse than one repository-level item. That is the owner's call, not a fix round's.

**finding-8**: Unmarked derived work — `pending_changes()`, `has_pending_changes()`, `last_calculated_root` and its `InvariantViolated` guard are public, documented, carried by five tests, and named by no item.
disposition: recorded as dated Obligation E, with the note that `check-trace.sh` structurally cannot find this class: it flags LLRs without tests, never tests without LLRs.

**finding-9**: Round 1 verified claims (a), (b), (c) and (e) — the type-level arguments, the declared LLR-juxk9q gap, and LLR-s97ywt's wording against PR-zz4exm — and found no fault in any.
disposition: no action. Recorded because a review that confirms a claim is evidence, and the claims are load-bearing for rows in the table above.

**finding-10**: The plan carries LLR-tk4qxu's pre-narrowing wording under a heading calling that text the source T2 copies "verbatim".
disposition: fixed by T11 — a dated pointer at the table and a cross-reference at T2, settling that the ledger, not the plan, is authoritative for item text. History not rewritten, per this project's convention.

**finding-11**: `org-members/docs/CONTEXT.md` is still the unfilled template, while this change introduces the unit's internal vocabulary at scale and uses terms the root glossary lists under `_Avoid_`.
disposition: recorded as dated Obligation F.

**finding-12**: Two semantically wrong SOUP references — the `blake3` row cited a field bound and a capacity constant.
disposition: fixed by T11. Replaced with four hashing properties (LLR-72p8bz, LLR-kdhd2v, LLR-wm5hpc, LLR-n7nya3) rather than dropped, since the row does support hashing items and naming none would be the worse record.

**finding-13**: `fuzz_tests.proptest-regressions` is not gitignored, and the mutation protocol produces it — a committed seed harvested from deliberately broken code would pin the suite to behaviour that never existed.
disposition: fixed by T11. `*.proptest-regressions` ignored with the rationale written out.

**finding-14**: Round 1's byte-uniqueness fix was applied to four sites; the claim lives in eight — including `org-members/README.md`, which states it as the crate contract and instructs integrators to key replay caches on the blob bytes, in the file `delta.rs` points readers to.
disposition: fixed by T12 in all four remaining sites. The README correction quotes the wrong sentence, names it wrong, cites the measured counterexample, and says what to key on instead. The historical spec keeps its body and gains a dated head note. Round 3 swept the whole tree independently and found the sweep complete in every normative document.

**finding-15**: `apply_delta` refuses an honest handle handover this crate produces, and no hazard in the register covers it.
disposition: PR-vf5hdm filed, `opened: 2026-09-17`, code deliberately unfixed. The two ledger sentences it falsifies were corrected. The register gap is recorded as a finding for `analyze-risks` — HAZ-8suua9 is hostile input and HAZ-y8h835 is staleness from lag; neither is "the receiver refuses a well-formed honest change". No HAZ or RC minted in a fix round.

**finding-16**: PR-jq43gx's scope is narrower than the defect — `DefaultHashes::at_level` is the same shape and equally public.
disposition: fixed by T12. PR-jq43gx amended in place to name both sites, with the note that whichever reading of REQ-ds8ryr applies to one applies unchanged to the other.

**finding-17**: Obligation A undercounts its own site list — a fourth `to_nfc` site at `types.rs:490`, and the obligation's prose half-admits what its table omits.
disposition: fixed by T12. Verified by mutation before recording; table, heading and prose reconciled at four sites.

**finding-18**: The six SDD items' `traces:` lists disagree with their own LLRs' `satisfies:` lines — SDD-k5wa4n omitted REQ-ds8ryr while LLR-v3jqau sits under it, hiding seven carriers from impact analysis.
disposition: fixed by T12, seven trace decisions each argued in the ledger. Round 3 recomputed the union from scratch: every REQ named by a child LLR is now traced by its parent, and the five surviving entries with no refining LLR are each argued and each argument holds.

**finding-19**: Two dead cross-references — a ledger citing a draft filename `finalize-docs.sh` had renamed, and PR-jq43gx miscounting its own finding number.
disposition: fixed by T12. The recurrence of the first at the next finalize is itself recorded — see Gaps.

**finding-20**: The README Overview does not contain the decomposition picture the per-change ledger delegates to it.
disposition: fixed by T12 rather than deferred — the README is the document that survives the per-change files, so deferring would have left the ledger's delegation false. It now carries all six items with their responsibilities and source files.

**finding-21**: The ed25519 point validation on the deserialisation path is refined by no LLR, carried by no test, and not marked derived — every key in every test comes from a valid `SigningKey`, so no invalid point is ever supplied.
disposition: recorded as a dated obligation alongside E and G. A third instance of the same class: public, documented, safety-relevant behaviour outside the traceability graph, which `check-trace.sh` has no direction to find. The code is correct; the gap is in the record.

**finding-22**: SDD-55b2zj's own defining sentence still says the change set is "canonical in its **encoding**" — the last normative survival of the withdrawn claim, repeated in the README Overview table and quoted in PR-vf5hdm.
disposition: recorded as a dated obligation naming the replacement wording ("canonical in structure"). Not fixed here: every note attached to the item now states the correction at length, round 3 judged it non-blocking, and amending an item's defining text is the next change's work under `grill-requirements` rather than a merge-sequence edit. Flagged as the sentence a later change amending the item will copy.

**finding-23**: `org-members/README.md` — the integrator contract — carries no mention of PR-vf5hdm, though a defect making a lawful handover unreplicable belongs where integrators read before building replication.
disposition: recorded as a dated obligation, naturally closed by the change that fixes PR-vf5hdm.

**finding-24**: `docs/superpowers/plans/2026-05-28-org-members-hyperbridge-fixes.md` still asserts byte-uniqueness with no note, and the README links it in the same sentence as the spec that did get one.
disposition: recorded as a dated obligation — one dated head note, the same treatment its sibling spec received. Round 2 noted the spec's status and passed over the plan.

**finding-25**: The plan states the uniqueness claim twice in prescriptive task text with no inline marker.
disposition: no action. The same file records the correction exhaustively, and this project corrects by date rather than rewrite. Recorded so the sweep's boundary — normative documents — is visible rather than implied.

**finding-26**: Obligation D's justification is stale — it says LLR-xmpqn2 "is worded about the postcard encoding", which after the correction it no longer is.
disposition: recorded for correction with the obligation. Its substantive claim is true and was independently verified twice.

**finding-27**: Three measured counts are slightly off — a "only place" that is one of two inside the same test, `from_bytes` counted per-file as if across three, and "twenty-one postcard call sites" for twenty.
disposition: recorded. No conclusion changes: the first is inside the test that already holds the annotation, and the other two are tallies in prose. Noted because this change's own recurring defect was stale counts, and these are three more.

## Gaps

What this change did **not** establish.

**Decision coverage is unmeasured**, against a class C target of statement AND
decision. Accepted by the owner as a documented gap on 2026-09-17 and scheduled
for tooth 5. The statement figures above say nothing about it.

**Three rows in the table above rest on the type system rather than on a
mutation**, and are marked so because a reader who expects a red→green row and
finds none must know why rather than assume an omission:

- **LLR-tk4qxu.** The item as first written also claimed a modification "copies
  only the path from the changed leaf to the root, sharing every other node".
  That is true and is how the code works, but it is invisible at the item's
  interface: the nearest compilable surrogate reddened fifty-plus tests without
  reddening either carrier, because no assertion over the original trie can see
  structural sharing — the shared tree is never written either. The clause was
  **withdrawn from the item** before the file was ever merged and is now design
  rationale. What remains is guaranteed by `Node` having no interior mutability
  but a write-once hash cell, and every operation taking `&self`.
- **LLR-y38jfk's "exposes no member query"** and **LLR-7tdqv9's "consumed
  either way"** are enforced by `CandidateTrie`'s surface and by
  `verify_against` taking `self` by value. The mutation that would test the
  first is a signature change producing seven compile errors, which is not a
  discriminating red.

A compiler-enforced interface property is stronger evidence than a passing
test. It is simply not *test* evidence, and this record does not present it as
any.

**LLR-juxk9q's handle-uniqueness half has no carrier.** `apply_delta` checks
both uniqueness and confusability, but every `DuplicateHandle` assertion in the
suite reaches that check through `genesis`, `add_member` or `update_handle`,
never through `apply_delta`. Confirmed by mutation twice: deleting the
`DuplicateHandle` arm leaves the suite green. If that check stopped working for
deltas, no test would notice.

**LLR-ch2pkw is not reached by the model-based conformance test**, structurally
rather than by sampling: the model's `init` is the empty map and both driver
call sites are `genesis(Vec::new())`, so `genesis`'s per-member loop never
executes. Three of four candidate LLRs were annotated to it, not four.

**No advisory database stands behind the SOUP risk columns.** Neither
`cargo-audit` nor `cargo-deny` is installed and the `Makefile` has no audit
target, so every risk cell is written from the code's actual use of the crate.
Recorded as Obligation C. Two findings came out of writing them anyway:
`ed25519-dalek` is used for point validation only — there is no signing or
verification anywhere in `org-members/src` — and `unicode-security` 0.1.2 is
the weakest row, because REQ-m8aexh's homograph control *is* that crate's
confusables table, which ages with every Unicode revision.

**Eight LLR robustness waivers are arguments, not evidence.** Ten carriers were
added where the missing side already existed in the suite; for the other eight
an abnormal-input case was argued not to apply (a `Debug` impl that writes
fixed literals and never reads what it redacts; a hash over `&[u8]` with no
invalid input; address boundaries that are themselves the carrier). Round 3
reviewed those arguments and did not dispute them, but they remain arguments.

**A defect on `master` that this change did not cause and did not fix.**
`on-chain-client`'s `check-ids` gate exits 1 today with three `DRAFT-FILE`
convictions — tracked files committed by `fbf17f0`, tooth 3's on-chain-client
merge, where `finalize-docs.sh` was never run for that unit. It survived
because `verify-before-merge` runs `check-ids` **with** `--allow-draft-files`
and `merge-change` step 4 runs it **without**; the flag is the entire
difference. `on-chain-client` is outside this change's impact set, so fixing it
here would have pulled the unit in and obliged this change to own its gates.
**Tooth 4's on-chain-client change cannot merge until it is cleared.**

**A process defect this change hit twice.** A ledger cross-reference left
pointing at a draft name after `finalize-docs.sh` consumed it was review round
2's finding 19 — and recurred immediately, because a fix round cannot fix a
rename that has not happened yet: renames land at step 3, after every fix round
that writes a draft. `finalize-docs.sh` renames the file and says nothing about
references to it, and no gate reads prose links. **Grep for the consumed draft
name after every run of that script.**

**Two prose lines opened with a bold ID at column one** and were reworded here
— `**LLR-juxk9q's …**` and `**REQ-crjxk8 on SDD-4yr9ge …**`. Neither was ever a
gate failure: the definition form requires the bold span to hold the bare ID
followed by a colon. They survived on a detail of the pattern rather than on
being unambiguous, and the first instance of this shape was fixed earlier in
the risk file without sweeping for siblings — the same incompleteness that let
the uniqueness claim survive two rounds.
