# Verification — on-chain-client architecture, SOUP and LLRs (2026-09-30)

branch: worktree-guardrails-on-chain-client-arch
reviewer: five rounds of independent subagent review at `merge-change` step 6a — three reviewers per round, each given the diff, the ledgers and the plan and no author narrative. Rounds 2 and 3 re-measured the dispositions of the round before rather than reading them; round 4 was the first to review the output of the two mechanical sweeps, which nothing had reviewed before; round 5 reviewed the fixes round 4 caused, including whether the corrected classification rule invalidates the two clauses the sweep cut. Two further findings, finding-69 and finding-70, were raised by no reviewer and by this record's own assembly; they are marked as such where they appear. **No round of review was given this record itself**, which is recorded as a gap rather than glossed.
verdict: merge, after five rounds and seventy dispositioned finding blocks, none disputed. Rounds 1 and 2 refused the merge on twenty findings, all in the documentation. Round 3 returned eight and found the defect that mattered most: the LLR ledger held clauses no gated test could falsify, and no amount of sampling would have finished enumerating them — so the answer was not eight edits but a mechanical sweep of every clause of all thirty-two low-level requirements. Round 4 was the first to review that sweep, and found its classification rule wrong: a mutation that breaks the library's own compilation measures nothing, and the two clauses filed UNTESTABLE-BY-MUTATION were both red. Round 5 reviewed round 4's fixes and found the corrected rule had a consequence round 4 had not applied — it invalidates one of the two clauses the sweep CUT. **LLR-xv7auy's distinctness clause had been deleted from a class C requirement on a probe that could not have observed it, and is restored as RED on a measured mutation.** That is the substantive result of this review. The standing outcome of the sweep is 99 RED (96 runtime, 3 compile-time) and 1 GREEN over one hundred independently falsifiable claims, all one hundred probed.

A sixth round is not ordered, and the reason is given rather than assumed. Rounds 1–3 found errors in what the change said; rounds 4 and 5 found errors in how it had measured. Of round 5's twenty-two findings exactly one — finding-47 — changed a classification; the rest are counts, citations and arguments. Every measurement class the reviews identified has since been swept exhaustively rather than sampled: all one hundred clauses probed, all thirty red → green citations re-resolved after the last prose edit, the anvil phantom counted after every edit, and the deviation argument stripped of two reasons it could not support. An argument for a sixth round would have to name a class of claim still unmeasured, and this record names none. "Each of five rounds found something" is true of any number of rounds and is not that argument. What stands in its place is fifteen open gaps, five of them raised by round 5, and the owner register in `docs/plans/2026-09-05-ratchet-setup.md` — where the remaining work is visible without another round.

Two conditions of this verdict are stated plainly. The gate passes with **org-node red** — 44 passed, 7 failed, all loopback timeouts, pre-existing, filed as PR-d4nye8 — which the owner accepted on 2026-09-28. And **coverage falls short of the class C target on both limbs**: statement 41.75%, decision unmeasured everywhere, also accepted as a documented gap. Neither is caused or worsened by this change, and neither is closed by it.
reproduced: not applicable as a defect reproduction — this change fixes no defect and writes no production code. What it does establish by measurement is recorded instead: the three LLR clauses the reviews found unfalsifiable were each proved so by mutating the source, running the named tests, and watching them stay GREEN (LLR-2znra8's comparison order, round 1; LLR-bhwsn6's early exit, round 2); every one of the thirty-two LLRs was discharged by watching a named test go RED under a named mutation; and PR-d4nye8's org-node timeout was reproduced directly, including with the harness sandbox disabled.

Change: `on-chain-client`'s first architecture ledger — nine software items and thirty-two low-level requirements refining the unit's twenty-one requirements, a measured SOUP inventory replacing an empty template, and one software item carrying no LLRs as a recorded class C deviation. Branched from `master` at `2cc25c0`, forty-three commits: written 2026-09-28, reviewed through 2026-09-29, recorded and re-gated 2026-09-30.
Plan: `docs/plans/2026-09-28-on-chain-client-architecture.md`.

Units touched: `on-chain-client` (architecture, risk, problems, README, SOUP) and `org-node` (one problem report). Impact set, computed with `check-units.sh --impact master..HEAD`: `on-chain-client` touched, `org-node` dependent (and subsequently touched), `app` dependent.

## The gate

Every figure from **gate run 8**, on the tree as merged. Runs 1–3 are superseded; runs 2–8 agree on every count. Run 5 followed the two mechanical sweeps, run 6 followed review round 4, run 7 followed review round 5, and run 8 followed the ten date corrections of finding-70 and the writing of this record — each because the tree had changed. Run 8 is the only run taken on the tree that is actually merged, taken after the last edit to every file in the change, which is the discipline the citation post-mortem in the plan arrives at, applied here to the gate.

One figure in the table below could not be taken that way, and it is marked. `check-units.sh` counts **419** tracked paths, where every earlier run counted 418: the difference is **this record**, which did not exist when runs 1–7 were taken and cannot cite a number measured before it was written. 418 is the figure for the tree without it, 419 for the tree with it, and both are stated rather than one being passed off as the other.

| Gate | Result |
| --- | --- |
| `check-units.sh` (repository-level) | exit 0 — `units: 4, disclaimed 7; tracked paths 419` (418 on runs 1–7, before this record existed) |
| on-chain-client `verify_commands` | **73 passed, 0 failed, 0 ignored** — lib 0, best_lane_reorg_rule 14, contract_address_filter 3, decode_org_state 9, decode_revive_event 17, h160_mapping 5, log_ownership 7, runtime_version_dispatch 5, storage_slot_layout 11, type_widths 2 |
| on-chain-client bolero targets (`harness = false`, no `test result` line) | 3/3 on budget without panic — fuzz_decode_org_state 258,273 it/s (4 corpus), fuzz_parse_revive_event 242,193 it/s (6 corpus), fuzz_event_round_trip 38,184 it/s. Rates differ run to run; what is verified is non-zero iterations and no panic. These three are **not** in the 73 — a bolero target prints no `test result` line, so the 73 is the sum of the other ten |
| org-node `verify_commands` [1/2] | **44 passed, 7 failed** — the accepted PR-d4nye8 set, all timeouts, not assertions. Both bolero targets on budget |
| org-node `verify_commands` [2/2] | `quint typecheck` exit 0, silent on success, binary resolution re-verified |
| app `verify_commands` | cargo **78 passed, 0 failed**; `svelte-check` 194 files, **0 errors**, 1 pre-existing warning; vitest **30 passed** |
| `check-review.sh` | exit 0 — `records 13, for worktree-guardrails-on-chain-client-arch 1, findings 70; provenance checked`. Its first run on this record reported MISSING-RECORD (the `branch:` line carried a trailing description) and then four MALFORMED-FINDING / ORPHAN-DISPOSITION pairs, because `**finding-45** (raised, not a defect):` opens no block. Two of the four were round 4's and had gone unnoticed for a round; all four are reshaped |
| `check-ids.sh` **bare** | exit 0, silent, in all four units — no draft token, no DRAFT-FILE, no duplicate, no malformed ID. The three impact-set units plus `org-members`, which is outside the impact set and was run anyway |
| `check-trace.sh` | exit 0 in all four units. on-chain-client: `REQ 21, HAZ 6, RC 9, SDD 9, LLR 32, PR 5`, no MISSING-TEST, UNSATISFIED-LLR, UNANALYZED-DERIVED or UNTRACED-DESIGN |
| Coverage, against the class C target | **SHORTFALL on both limbs.** Statement 41.75% (205 of 491 lines), region 42.60%, function 36.21%; decision coverage **UNMEASURED** — llvm-cov reports Branches `0/0/-` on every file. Floors of 41/42 held, which is why the command exits 0; the floors are the accepted-shortfall marker, not the class target |
| Working tree | `git status --porcelain` empty |

Two figures moved and neither is acted on. Region coverage measured 42.60% where `on-chain-client/.guardrails/config.yaml` narrates 42.45%, on identical line, line-count and covered-line figures. The gap is **exactly one region of 669** — 285 covered here against the 284 the config records, and gate runs 4 through 8 all measured 42.60%. I first wrote this off as bolero run-to-run variance. Review round 4 gave a better account and I accept it: **this change is what moved the figure.** The one new test, `root_updated_epoch_above_u64_is_refused_not_truncated`, is the first gated test to drive `parse_root_updated` down the error path of `decode_uint256_to_u64(topics[2])?`, so it takes that `?`-propagation region for the first time. That is deterministic, which is why the runs agree with each other rather than scatter, and it is the account run-to-run variance never gave. It is not separately measured against `master` and is stated as the likely cause rather than a measured one. The floors are not moved for it, and no document in this change had noticed the move. The `svelte-check` warning in `app` is a missing `@types/node`, structural and pre-existing, with zero errors.

*Found while writing this record up after round 5.* The paragraph above replaces one that had an earlier draft of itself spliced into the middle of a code span — `decode_uint256_to_u64(` … `topics[2])?` — so it read as a single sentence interrupted by a whole stale paragraph, and stated both accounts of the moved figure as if each were the conclusion. That is the same splicing defect this record catalogues twice in the ledgers, committed here in the record that catalogues it, and it survived five rounds of review because no reviewer was given this draft to read.

**Base merged from the remote, not a local ref.** `git fetch origin` succeeded (exit 0), and `git merge origin/master` reported already-up-to-date because `origin/master` is `2bb1c21` of 2026-06-17 and is an ancestor of this branch. The duplicate scan at step 4 was therefore compared against the real remote. That the fetch *succeeds* is itself evidence: the 46 unpushed commits on this repository are not an access problem, which narrows the open owner item about whether this repository pushes.

## Red → green

Thirty-two rows, one per LLR, copied from the task subagents' dispatch reports. **Every test in this change already existed and already passed** except one, so a pass count proves nothing about whether any of them could ever go red. Each was therefore discharged by RED BY MUTATION: a named mutation to the source, the named test watched failing for that reason, the mutation reverted, and the file proved byte-identical by SHA-256.

Five source files were ever mutated. Each holds one digest across every cycle in every task that touched it — `src/client.rs`'s reported independently by T4, T8 and T9, and `src/decode/v_paseo_ah.rs`'s by T6, T7, T7b and T10, which is the cross-check that no task left drift behind. Round 2 and round 3 of the independent review re-mutated `src/client.rs` themselves and returned it to the same digest.

| File | SHA-256, before every mutation and after every revert |
| --- | --- |
| `src/types.rs` | `d951091a3f461264a85c71173571ccf7a1d83c7f5ef49ea004d5ed1a46ee5832` |
| `src/h160.rs` | `e46ff22701997c1003c86752d9c0ca141a438d07bc932ad0f5e800f9fc6755bc` |
| `src/decode/dispatch.rs` | `b60cceb69528f2a2b086344059b28d75d4371cfb04c1b4dc027777acc73e2000` |
| `src/decode/v_paseo_ah.rs` | `d9557564ffd373c6d464022a7360c29495c893f8cd2522eda747dfee6bb1ca99` |
| `src/client.rs` | `9527992d6e2a26e783132be301c6a74caa30090754b9dd9ca590892323f96e68` |

| Item | Test | Watched red |
| --- | --- | --- |
| `LLR-xv7auy` | `type_widths::newtypes_have_the_widths_the_abi_gives_them` | `#[repr(align(32))]` on `OrgAdmin` → width assertion fails, left 32 right 20 |
| `LLR-z8rrkr` | `contract_address_filter::the_emitting_address_is_reported_byte_for_byte_for_both_event_shapes` | `EmittedEvent { contract: [0u8; 20], .. }` → all three tests fail on the reported address |
| `LLR-b4p32h` | `best_lane_reorg_rule::the_discarded_reference_carries_both_hash_and_number` | reorg arm → `Some(BlockRef { hash: prev.hash, number: 0 })` → fails, left 0 right 7654321 |
| `LLR-2yhra8` | `h160_mapping::reverse_path_returns_the_first_twenty_bytes` | `&account_id_32[..20]` → `[1..21]` → returned window slides, pulls a marker byte into the tail |
| `LLR-3bkhuc` | `h160_mapping::forward_path_keccaks_then_truncates` | `&hash[12..32]` → `[0..20]` → digest's high twenty bytes returned instead of its low |
| `LLR-7pjzjn` | `h160_mapping::forward_path_taken_when_only_eleven_marker_bytes_are_present` | marker window `[20..32]` → `[21..32]` → an eleven-marker account takes the reverse path |
| `LLR-vktf8w` | `storage_slot_layout::mapping_slot_matches_the_known_vector` | `buf[12..32]` → `buf[0..20]` → right-padded preimage hashes to a different key |
| `LLR-62tqnv` | `storage_slot_layout::the_mapping_slot_index_is_big_endian_in_the_low_eight_bytes_of_the_second_word` | `to_be_bytes()` → `to_le_bytes()` → the two sides become byte-identical |
| `LLR-bhwsn6` | `storage_slot_layout::a_carry_propagates_through_four_bytes` | `if carry == 0 { break; }` → unconditional `break` → carry stops after the low byte |
| `LLR-2y9qdc` | `storage_slot_layout::an_all_ones_slot_wraps_to_zero` | `sum as u8` → `sum.min(255) as u8` → the slot stays all-ones instead of wrapping |
| `LLR-v62yjq` | `storage_slot_layout::the_three_struct_field_slots_are_consecutive_and_distinct` | `carry = u16::from(offset)` → `0` → all three keys collapse onto the base |
| `LLR-b3s7st` | `runtime_version_dispatch::pinned_version_resolves_to_a_decoder_that_decodes_the_pinned_layout` | pinned arm → `Err(UnsupportedRuntime)` → the pinned version stops resolving |
| `LLR-u8ajby` | `runtime_version_dispatch::{version_one_below,version_one_above,zero_version,max_version}_is_refused` | wildcard arm → `Ok(&DECODER)` → all four refusals fail; the fall-back-to-a-guess the REQ forbids |
| `LLR-e6skvu` | `decode_revive_event::the_recognised_signature_hashes_are_the_keccak_of_the_canonical_solidity_strings` | `SIG_ROOT_UPDATED` first byte `0x24` → `0x25` → keccak of the canonical string no longer matches |
| `LLR-u2e389` | `decode_revive_event::genesis_initialized_round_trips_every_field` | `contract: [0u8; 20]` → emitting contract comes back all zeroes |
| `LLR-rjcqg3` | `decode_revive_event::an_unknown_first_topic_yields_nothing` | `_ => Ok(None)` → `Err(InvalidAddressTopic)` → an unknown topic becomes an error |
| `LLR-n6gghu` | `decode_revive_event::genesis_data_of_any_length_but_sixty_four_is_rejected` | `data.len() != 64` → `< 64` → a 192-byte payload is accepted |
| `LLR-89pdz9` | `decode_revive_event::a_non_zero_byte_at_each_padding_position_of_the_address_topic_is_rejected` | `topic[..12]` → `[..11]` → the twelfth pad byte stops being checked |
| `LLR-8242kq` | `decode_revive_event::any_trailing_bytes_after_a_well_formed_payload_are_rejected` | `if !bytes.is_empty()` guard deleted → an unread tail is ignored |
| `LLR-6tjhgk` | `decode_revive_event::root_updated_round_trips_every_field`, `fuzz_event_round_trip` | `prev_root_hash` from `&data[32..64]` → returns `org_pub_key`; fuzz target fails its inversion assertion on the 1st rng input |
| `LLR-2v5u4d` | `fuzz_parse_revive_event` | `topic[..12]` → `topic[..40]` → panics `range end index 40 out of range for slice of length 32`, 1st corpus input, 345µs |
| `LLR-mzh8df` | `decode_revive_event::root_updated_epoch_above_u64_is_refused_not_truncated` (**new in this change**) | `decode_uint256_to_u64(&topics[2])?` → direct low-eight-byte read → `Ok(.. Epoch(7) ..)` where `Err(EpochOverflow)` is required |
| `LLR-sq76u3` | `decode_org_state::ninety_seven_bytes_is_rejected` | `bytes.len() != 96` → `< 96` → a 97-byte blob decodes |
| `LLR-nq7nhg` | `decode_org_state::exactly_ninety_six_bytes_decodes_every_field` | the two slot copies swapped → fields read from each other's slot |
| `LLR-emp3g9` | `decode_org_state::a_non_zero_byte_anywhere_in_the_epoch_slots_leading_twenty_four_bytes_is_refused` | high-24 guard deleted → `Epoch(7)` returned where `EpochOverflow` is required |
| `LLR-yhw34z` | `fuzz_decode_org_state` | length guard deleted → panics `range end index 96 out of range for slice of length 95`, 1st corpus input, 303µs |
| `LLR-2znra8` | `log_ownership::the_spoof_a_valid_log_from_another_contract_is_not_ours` | contract guard deleted from `log_is_ours` → the impostor's log is admitted; HAZ-werm85's own spoof |
| `LLR-kfr75c` | `log_ownership::with_no_filter_any_admin_from_the_configured_contract_is_ours` | `None => true` → `None => false` → an absent filter reads as "match nothing" |
| `LLR-9qp3k7` | `log_ownership::with_a_filter_set_a_matching_admin_is_ours` | a const sentinel returned from `event_admin`'s `Update` arm → a genuine `RootUpdated` for the filtered admin is refused while `Genesis` still passes |
| `LLR-d3ef7s` | `best_lane_reorg_rule::repeated_hash_with_inconsistent_number_is_still_skipped` | dedup arm gains `&& prev.number == n` → the same block with an inconsistent number stops being skipped |
| `LLR-56bzcj` | `best_lane_reorg_rule::a_rewind_below_the_last_height_reports_the_discarded_head` | `n <= prev.number` disjunct dropped → a rewind no longer reports the discarded head |
| `LLR-48ygak` | `best_lane_reorg_rule::a_jump_backfills_every_skipped_height` | `prev.number + 1` → `prev.number` → the span re-reads a block already processed |

## What was wrong, and what was built

`on-chain-client` had `SDD 0, LLR 0` against twenty-one requirements, 1,271 lines of source across nine modules, and a twelve-line SOUP template with an empty table — in a unit classified IEC 62304 class C, which requires low-level requirements per software item and a SOUP inventory that reflects reality.

Nine items now divide the unit along the chain of custody of a byte: what it is typed as, how an address is derived from it, where in contract storage it lives, which decoder may read it, what that decoder refuses, who is admitted to see the result, how provisional observations are ordered, and what the transport does with all of it. Thirty-two low-level requirements refine eight of them. **No production code was written.** One test was added; thirty-one of the thirty-two LLRs are carried by tests that already existed, which is what made this tooth affordable.

**The ninth item carries no LLRs, and that is a recorded deviation rather than an omission.** SDD-3b8zef is the chain-facing subxt transport — roughly 580 of `client.rs`'s 699 lines, measured at 16.37% line coverage. Nothing this unit's gate can run reaches it; the five ungated integration targets that do exercise it need a chopsticks fork or an anvil instance. LLRs *could* have been written for it and annotated onto any of the nine ungated targets, and `check-trace.sh` would have reported a clean tree — because MISSING-TEST is satisfied by a `verifies:` reference in a file under `test_paths` and nothing more. The independent review verified that against the script and found it wider than first stated: the scan consults no cargo target, no `required-features`, no `#[ignore]` and no run record, and passes `--untracked`, so a README or an uncommitted file would satisfy it equally. Taking that route would have produced a green gate standing over evidence that was never produced — the defect this repository recorded against itself on 2026-09-18. It was put to the owner on 2026-09-28 and refused.

The SOUP inventory was measured rather than copied. `on-chain-client` declares its own `[workspace]`, so unlike its sibling unit its lockfile is the one cargo actually reads — confirmed with `cargo locate-project --workspace` before the provenance paragraph was written. Five direct dependencies over a 203-crate normal-edge closure, 407 lockfile entries. Two findings came out of it that were not visible from `Cargo.toml`: `jsonrpsee` is present at two semver-incompatible versions (0.24.11 pinned inside subxt, 0.26.0 dev-side), which is the wasm32 blockage seen from the other side; and three keccak implementations are in one binary, of which the unit calls exactly one.

## Review

**Five rounds, and seventy finding blocks below, each with its disposition.** Rounds 1 and 2 refused the merge and produced finding-1 through finding-20 between them, **every one in the documentation** — not one in the renames, the annotations, the ledger integrity, or any gate. Round 3 returned eight findings, of which only finding-21 is enumerated individually here, for the reason its own section gives: they were answered as two classes by two mechanical sweeps rather than instance by instance, and that is a stated limitation of this record. The sweeps then raised finding-22 through finding-24 against this change. Round 4 raised finding-25 through finding-46, and round 5 finding-47 through finding-68 — twenty-two each. The last two, finding-69 and finding-70, were raised by no reviewer: they were found while writing this record for the merge, and are disposed of here rather than left out because nobody else caught them.

That the findings were all documentation held until round 4. Rounds 4 and 5 were the first to review not what the change said but how it had measured, and both found defects in the measurement: a classification rule that was wrong (finding-25), a corrected rule whose consequence was not followed through, and a clause deleted from a class C requirement on a probe that could not have observed it (finding-47 for both). Those are not documentation findings and are not described as such below.

The blocks are numbered continuously across all five rounds. Where a later round superseded an earlier disposition the earlier text is left standing with the supersession attached to it, so a reader can see what was concluded, when, and on what evidence — not only the last version. Four blocks carry such a supersession — finding-4, finding-25, finding-26 and finding-28 — and finding-27 carries a footnote of the same kind.

**finding-1**: LLR-2znra8 restored, as a low-level requirement, the ordering clause REQ-9vwcwc deliberately dropped as unfalsifiable, and no annotated test gates it. Measured: `internals::log_is_ours` was rewritten to evaluate the admin filter first and the contract comparison last, and all four annotated tests stayed green.
disposition: the clause is cut, leaving only the consequence — "so that a matching Organisation admin never rescues a foreign log" — which `the_contract_check_dominates_a_matching_admin_filter` does red on. Round 2 re-ran the inversion itself and confirmed the disposition holds and the file returns to its digest.

**finding-2**: LLR-56bzcj dropped REQ-ntn4ss's hash-differs conjunct, so it demanded a reorg report for a repeated head that `scan_step` skips — contradicted by a passing test twenty lines above its own carriers.
disposition: conjunct restored; the LLR now matches REQ-ntn4ss word for word and `scan_step`'s dedup arm. Verified in round 2.

**finding-3**: LLR-kfr75c's no-filter clause is not implied by REQ-nygs7k, which is conditioned on a subscription that names an admin, and it was not marked derived.
disposition: annotation changed to `satisfies: REQ-9vwcwc, REQ-nygs7k`. Round 2 checked both requirements support their halves and did not re-file it.

**finding-4**: SDD-3b8zef's supporting evidence was overstated twice — nine integration targets claimed to exercise it where only five do, and `ClientError`/`SubscribedEventStream` listed as members of an item whose whole claim is that nothing at the gate reaches it, though `ClientError` is chain-free and reachable from the gated tests today.
disposition: counts corrected to five with the other four characterised individually; both symbols moved to SDD-5wamsz; the writable-but-unwritten LLR for `ClientError` filed as an owner item. The deviation's conclusion survives — the async transport is genuinely unreachable — at its true scope.
*Superseded in part by review round 5 (finding-54).* Moving `ClientError` was argued and is right. Moving `SubscribedEventStream` was not argued at all: it is produced by nothing but `subscribe`, no gated test can construct one, and it shares none of the properties the argument for the move rested on. The relocation therefore narrowed the deviation's **stated** scope without narrowing the unreachable surface, and "at its true scope" was not true of the second symbol.

**finding-5**: MISSING-TEST is satisfied by a `verifies:` reference in a file rather than an executed test — **verified true**, and wider than the ledger stated.
disposition: no fix; the finding confirms the load-bearing claim. Its detail (the scan passes `--untracked`, and any text file under `test_paths` serves) was written into the ledger, which now states the hazard at its true breadth.

**finding-6**: the SOUP inventory is correct on every checkable point — five direct versions, the 203-crate closure, 407 lockfile entries, every transitive version named in prose, and the workspace-root claim.
disposition: no fix required.

**finding-7**: unmarked derived work in three places — LLR-kfr75c (see finding-3), SDD-5wamsz claiming a best/finalised distinction no requirement states, and LLR-v62yjq claiming a read performed by the ungated `get_org_state`, which its test never calls.
disposition: a sentence added to SDD-5wamsz recording that the distinction is carried by the type but required by nothing; LLR-v62yjq reworded to the arithmetic its test actually pins.

**finding-8**: item grammar is clean — all 41 definitions at column one, every `traces:`/`satisfies:` resolvable and block-attributed, no definition moved between files, every named source symbol present, terms resolving to the two glossaries.
disposition: no fix required.

**finding-9**: three assertions false against the tree they ship in — "four source files" over a five-row table, the org-node report claiming an empty diff under `org-node/` while being itself a file under `org-node/`, "eighteen files" where there are nineteen — and the 18/14 robustness split attributed to "the gate" when no script measures sidedness.
disposition: all four corrected; the robustness figure reattributed to a hand count made by the verification subagent.

**finding-10**: LLR-bhwsn6 carried an unfalsifiable clause — "stopping at the first byte that does not carry". Measured: `if carry == 0 { break; }` was deleted outright and `storage_slot_layout` still reported 11 passed, because once the carry is zero every remaining byte is `byte + 0`. The plan's mutation for this LLR reds the propagation, never the stopping.
disposition: the stopping clause is cut, the arithmetic kept. Recorded in the plan's findings table as the second instance of the round-1 defect class.

**finding-11**: the risk ledger claimed "four of the five problem reports open against this unit are already in exactly that code", false twice — four are open of five that exist, and only two of the four named are in SDD-3b8zef's code.
disposition: rewritten to two of four, with the other two located (PR-qpp28h in `internals::scan_step`, SDD-m59zrg; PR-h4mb8y in `src/verify.rs`, which this change records as no item at all).

**finding-12**: round 1's nine→five correction reached the decomposition and the register but not `soup.md` or the plan, leaving two ledgers disagreeing with two others about the same set.
disposition: both corrected, with "nine" deliberately retained wherever it means "how many places an annotation could hide" — a distinction the first correction had blurred in the opposite direction.

**finding-13**: moving two symbols into SDD-5wamsz left every file-count statement at the old number, so four files disagreed about how many items `client.rs` carries.
disposition: corrected to five in the decomposition, the README's prose and its table's Source cell, the plan's table, and `soup.md`'s `futures-core` row.

**finding-14**: LLR-b4p32h was marked `satisfies: derived` while REQ-ntn4ss states its content verbatim — "carrying both the hash and the number of the head that was discarded".
disposition: changed to `satisfies: REQ-ntn4ss`. The risk ledger's assessment was rewritten rather than deleted, to record that the item was marked derived in error and that the hazard argument stands as a note; the file's counts adjusted to one derived LLR.

**finding-15**: SDD-5wamsz's item text did not describe the error type moved into it, and a class C item's text is what its LLRs are judged as refinements of.
disposition: responsibility extended to cover the typed error a caller is handed and the stream observations arrive on.

**finding-16**: the decomposition cited SDD-v2rtka as already carrying an LLR of the kind proposed for `ClientError`'s `Display`. It carries no such LLR; no LLR anywhere in the ledger mentions `Display`.
disposition: the comparison withdrawn and the owner item stated on its own terms.

**finding-17**: a restrictive clause contradicting its own list — `regenerate_corpus` named among "targets that need a chain" in the same sentence that calls it an `#[ignore]`d corpus writer constructing no client.
disposition: qualifier dropped.

**finding-18**: `.guardrails/units.yaml` cited as recording no `depends_on:` edge, when its schema cannot carry one either way; edges are declared in each consumer's config. The fix round found two further instances the finding had not named, inside a pre-existing problem report.
disposition: all three citations corrected to the configs that actually carry the declaration.

**finding-19**: LLR-u2e389's first clause — the SCALE field order of `ContractEmitted` — has no parent in REQ-5upq6n and no `derived` marking.
disposition: trimmed to what the parent carries, keeping the falsifiable substance.

**finding-20**: a missing paragraph break rendering two unrelated sentences as one paragraph.
disposition: break inserted.

### Round 3, and why it ended in two sweeps rather than eight fixes

Round 3 returned eight findings, four above the merge bar. One had the shape
rounds 1 and 2 had already produced twice: a low-level requirement asserting a
property **no gated test can falsify**. The rest above the bar were propagation
failures — a correction applied in the one file a finding named, left standing
in every other file repeating the same claim.

At that point the pattern was the finding. Three independent rounds, given no
knowledge of their predecessors, had each returned exactly those two classes,
and **reading caught neither**: every unfalsifiable clause was found by mutating
the source and watching a test not care, never by comparing the requirement to
the code. The owner was asked on 2026-09-28 whether to keep answering instances
or to attack both classes exhaustively, and chose the sweeps.

**Round 3's findings are therefore not enumerated one by one below, and that is
a deliberate limitation of this record.** They were answered as classes, not as
instances, so there is no per-finding disposition to quote for most of them.
What each sweep changed is enumerated in its own commit message — `5e8ff17` for
the low-level requirements, `10b0957` for the propagation — and the two clauses
the first sweep cut are recorded permanently in the plan's findings table and in
the decomposition's own sweep section. The one round-3 finding that traces
individually through to a disposition is recorded here.

**finding-21**: LLR-xv7auy's distinctness clause — "distinct ... so that two
fields of equal width cannot be substituted for one another" — asserts a
property no gated test can falsify.
disposition: **cut**, and confirmed by measurement before cutting. Replacing
`pub struct OrgPubKey(pub [u8; 32]);` in `src/types.rs` with
`pub use crate::types::OnChainRootHash as OrgPubKey;` performs the exact
substitution the clause forbids, and the whole of `verify_commands` stayed green
at 73 passed, 0 failed. `newtypes_have_the_widths_the_abi_gives_them` asserts
`size_of` and `.0.len()` only, which cannot tell two newtypes of equal width
apart. The four width claims in the same LLR all red under `#[repr(align(N))]`
and stand; T2's attestation is unaffected, because what it reds is the width.

### The two sweeps

Both are mechanical, both are recorded as findings against this change rather
than as achievements, and both were re-gated afterwards.

**sweep-1 — every clause of all 32 LLRs, mutation-tested.** One hundred
independently falsifiable claims; ninety-nine probed by their own source
mutation over 102 runs of the full `verify_commands`. **96 RED, 2 GREEN, 2
UNTESTABLE-BY-MUTATION.** The two GREEN are cut (LLR-xv7auy's distinctness,
finding-21 above; LLR-mzh8df's "by the same `uint256`-to-`u64` rule as the
storage path", which is a claim about code sharing rather than behaviour — an
inline byte-for-byte identical check leaves the gate at 73 passed). The two
untestable are LLR-xv7auy's "public" and LLR-z8rrkr's "as one value", each
mutable only by breaking the compilation of unrelated code; they are kept and
flagged rather than quietly counted as red. A third sampling limit is recorded:
LLR-e6skvu's "exactly two" reds against a drifted-ABI third signature but not
against an arbitrary one.

*Superseded twice, and both supersessions are below.* The paragraph above is
what sweep-1 reported. Review round 4 (finding-25) showed the two
UNTESTABLE-BY-MUTATION were both RED and rewrote the classification rule; review
round 5 (finding-47 through finding-49) showed the corrected rule invalidated
one of the two cuts and that round 4's evidence for the other reclassification
was itself wrong. The standing result is **99 RED — 96 at runtime, 3 at compile
time — and 1 GREEN**, all one hundred claims probed, with only LLR-mzh8df's
code-sharing clause cut. The figures here are left as the sweep reported them so
that the corrections can be read against something.

The result is the important part, and rounds 4 and 5 strengthened it rather than
weakening it: **the corpus was sound.** On the standing classification
**ninety-nine of a hundred** clauses are falsifiable by a mutation that reds this
unit's own gate, and three rounds of sampling had been unlucky in which of the
remaining few they reached. During sweep-1 the five source files in the digest
table were mutated and restored 102 times and every one holds its tabulated
SHA-256, re-verified at every gate run from 5 to 8. Rounds 4 and 5 mutated two
further files — `src/decode/mod.rs` and `src/state.rs` — which that table never
named, so the cordon does not cover them; that is recorded in Gaps rather than
folded in (finding-60).

**sweep-2 — every corrected claim propagated.** Seven classes of claim, each
corrected everywhere it appears rather than where a finding named it: the
nine-versus-five target count, the unreproducible "580 of 699 lines" figure
replaced by the measured one, `soup.md`'s subxt row, the README's source cells,
the robustness split's twelve-not-thirteen pairing, three reflow artifacts, and
four smaller claims swept together. Verification records under
`docs/verification/` were deliberately **not** amended: a signed record is
evidence that a review happened, and editing it afterwards falsifies it rather
than correcting it. Every live ledger those records feed now carries a dated
note pointing the other way.

**finding-22**: the phrase "a chopsticks fork or anvil" describes tooling that
does not exist in this repository. `grep -rli anvil` across `*.rs`, `*.toml`,
`*.js`, `*.ts`, `*.json`, `*.sh` and `*.yml` returns nothing; every
fork-dependent target spawns chopsticks through
`common::chopsticks_fork::spawn_fork`.
disposition: **partly fixed, and the rest filed rather than half-done.** The six
occurrences this change itself wrote, plus one config comment, are corrected.
Roughly twenty pre-existing occurrences across the SRS, the risk register, the
problem ledger and two plans are **not** touched here, because correcting a
phrase in some files and not others would recreate the propagation defect
sweep-2 exists to remove. Filed as an owner item in
`docs/plans/2026-09-05-ratchet-setup.md`: retire the phrase repository-wide in
one sweep, or define it once as a deliberate term of art.

**finding-23**: PR-d4nye8's enumeration of what still passes summed to 43 against
its own stated observable of 44 — `transport_networked`'s single test was named
in the argument below it and omitted from the arithmetic above it.
disposition: fixed at gate run 5, which reproduced the 44/7 split exactly.

**finding-24**: PR-d4nye8 described this change as "nineteen files". Review round
2 had already corrected that number once, from eighteen; the two sweeps then took
it to twenty-four and it was stale again.
disposition: fixed by removing the count rather than correcting it a third time.
The count was never the evidence — that no production source is touched is, and
the diffs the same bullet already names show it directly. The reason is stated in
place so the next reader does not helpfully restore a number.

### Round 4 — the round that was meant to be the last

Three reviewers again, given the sweeps' own output, every measured figure, and
the deviation argument respectively. **The sweeps had never been reviewed by
anyone**, and that is where most of what follows came from. Twenty-two findings,
more than any previous round, and two of them changed a class C classification
rather than a sentence.

**finding-25**: both claims the falsifiability sweep filed
UNTESTABLE-BY-MUTATION are in fact RED. The sweep tried one mutation route per
claim, found it broke the *library's* compilation, and generalised from that to
"untestable".
disposition: both re-measured and reclassified, and the classification rule
rewritten, because the rule was the defect rather than the claims. LLR-xv7auy's
"public": `pub(crate) struct OrgAdmin(pub [u8; 20]);` in `src/types.rs:19` with
a matching `pub(crate) use` at `lib.rs:32` leaves the library compiling with
warnings only and exits the gate **101** with `E0603` in **six** integration
targets — including `type_widths.rs:104`, the very target annotated
`verifies: REQ-2qa5r5, LLR-xv7auy`. LLR-z8rrkr's "as one value": re-typing
`parse_revive_event` to return `([u8; 20], Event)` with `client.rs:665`
re-assembling the struct leaves the library compiling and exits **101** with
`E0308` in four targets. The rule now distinguishes a broken library (measures
nothing) from a failing test target (an ordinary red).
*Superseded in part by review round 5.* The "public" reclassification stands.
The "as one value" one reached the right verdict on the wrong mutation — a
2-tuple is one value, so what round 4 measured was the tests' dependence on a
name (finding-49) — and the corrected rule invalidated one of the sweep's two
cuts, which round 4 did not follow through (finding-47). The outcome table round
4 wrote, **98 RED / 2 GREEN**, is not the standing one; it is **99 RED (96
runtime, 3 compile-time) / 1 GREEN**. The count of six targets above is itself a
round 5 correction (finding-57); round 4 wrote "five" over an enumeration of
six.

**finding-26**: the sweep's arithmetic did not close — 99 probed against a table
summing to 98, and "the hundredth" in the singular against two unprobed claims.
disposition: resolved by finding-25 rather than patched. All one hundred claims
are now probed. The run count is also corrected: **103 runs over 102 distinct
mutations** (`2v5u4d-panic` was run twice), where the commit message had quoted
the distinct count as the run count.
*Superseded by review round 5:* the arithmetic that closes is now **99 + 1 =
100**, not 98 + 2, because finding-47 restored a cut clause as RED. The run
count is unaffected — rounds 4 and 5 added measurements of their own, recorded
with each finding, and did not re-run the sweep.

**finding-27**: **the change was closing its own gate.** The decomposition cited
`PR-d4nye8`, an org-node ID, and on-chain-client does not declare org-node in
`depends_on` — correctly, the dependency runs the other way. `check-trace.sh`
therefore exited **1** with UNDECLARED-DEPENDENCY, while the plan asserted it
exited 0 with `SDD 9, LLR 32`.
disposition: the cross-unit reference is dropped; the sentence names the defect
without the ID. `check-trace.sh` exits 0 again, and the plan's claim is true.
*Footnote from review round 5 (finding-68):* the commit that claims this fix,
`764c386`, does not contain it — the citation had already gone in an earlier
commit. The fix is real and the gate proves it; the attribution in that one
commit message is wrong, is not rewritten, and does not survive into the squash.

**finding-28**: seven of the red→green citations — thirty of them, though round
4 counted thirty-three (finding-56) — were **four lines stale** — captured while each task ran, before this change's own annotations
lengthened two test files. Two were worse than stale: `log_ownership.rs:143`
landed on a string-literal continuation of a *self-check* the named mutation
cannot fail, and `:234` on the `Genesis` assert rather than the `RootUpdated`
one the mutation reds.
disposition: all of them re-resolved against the merged tree; the seven
corrected. The plan gains a note on the mechanical cause, because it will recur:
citations into files a change edits must be taken after the edits.
*Superseded by review round 5 (finding-55):* the commit that applied this
disposition broke five further citations by the same mechanism, in the same
commit that wrote the note. All thirty stand re-resolved against the tree after
the last prose edit of round 5.

**finding-29**: the anvil correction garbled the sentence it repaired. The
substitution left "each need a chopsticks fork **or an** chopsticks fork" in the
deviation section — the load-bearing paragraph of the whole change.
disposition: fixed.

**finding-30**: worse, this change **wrote three fresh uses of the phantom
phrase into the very register where it declares the phrase a phantom**
(`ratchet-setup.md:342`, `:934`, `:967`), one of them inside the SDD-3b8zef
owner item the deviation argument points readers to.
disposition: all three corrected; the owner item now says nine occurrences
rather than six and records that round 4 found the ones the sweep missed.

**finding-31**: the count in that owner item was measured wrongly — "roughly
twenty documents" where the measurement is **twenty-one occurrences across ten
documents**, plus two in the `Makefile`. Occurrences had been quoted as
documents.
disposition: corrected, with the breakdown that actually matters: four are
verification records left immutable by decision, two are this change's own files
where the phrase now survives only as a quotation, and **four are live documents
still asserting it**, named individually.

**finding-32**: SDD-bw7v5x's source cell read bare `` `internals` `` in the
README and the plan, against the decomposition's
`internals::solidity_mapping_slot`, `internals::increment_slot`. Bare
`internals` swallows `log_is_ours` and `scan_step`, so three items' sources
overlapped — the exact defect the note directly above it claims to have
repaired, under a sentence asserting the tables now agree on every row.
disposition: both cells narrowed; SDD-m59zrg aligned to include
`internals::ScanStep`; the note rewritten to record that round 4 completed a
sweep that had missed a row.

**finding-33**: `src/lib.rs` is named by no software item and the omission was
not stated, although `pub mod test_support` is the only route by which the gated
tests reach `internals`.
disposition: acknowledged in "What is not an item", with the argument for why it
is a test seam rather than an item — and the observation that deleting it would
strip three items of all their evidence at once.

**finding-34**: "Eight of the nine items above meet it" is true only on the
weak reading (every item has ≥1 LLR), not on the obligation the same sentence
states (every part of an item's declared interface refined). SDD-v2rtka declares
the `Decoder` interface and the typed error vocabulary alongside version
resolution and refines only the last; SDD-5wamsz owns `ClientError` whose
`Display` is conceded writable and unwritten.
disposition: qualified in place, and "already fully gated" narrowed to "whose
version-resolution interface is fully gated" both in the decomposition and where
`soup.md` repeats it.

**finding-35**: the deviation argument was posed as a dichotomy — annotate onto
ungated targets, or mint nothing — when a third option exists: mint the LLRs and
leave them **unannotated**, so `check-trace.sh` reports MISSING-TEST and exits 1
and the gap becomes machine-visible.
disposition: recorded, and conceded to be stronger on the section's own
criterion. Not adopted, for the stated reason that it reddens a gate on a tree
where nothing is broken while org-node's gate is already red — filed as an owner
item rather than settled by the author. **See the Gaps section: this concession
is the weakest point in the deviation's defence and it is not hidden.**

**finding-36**: the deviation is **enforced by nothing**, which no document
said. `check-trace.sh` never reads `safety_class`; the only gate that does is
`check-units.sh`, for cross-unit class ranking alone. No gate anywhere requires
a class C item to carry low-level requirements, so this unit's gate exits 0
today and will go on doing so for as long as SDD-3b8zef carries none.
disposition: stated plainly in the decomposition, together with the related fact
that **`verify_commands` is executed by no script in this toolkit** — it is a
configuration key the merge process runs by hand.

**finding-37**: `type_widths.rs`'s header stated three things this change made
false — that `DecodeError::EpochOverflow` is asserted "in exactly one place in
this repository", that `decode_revive_event.rs` "has no over-wide-epoch case at
all", and that a case on the event path "would be a second copy of that evidence
rather than a second piece of it" — and contradicted the decomposition head-on
about the same test. Its argument also rested on both paths sharing
`decode_uint256_to_u64`, which is the premise the sweep had just cut from
LLR-mzh8df as unfalsifiable.
disposition: rewritten. EpochOverflow is asserted in two places
(`decode_org_state.rs:237`, `decode_revive_event.rs:777`); the event path now has
a gated over-wide case, which is this change's one new test; the superseded
argument is recorded rather than deleted, and the 2026-09-11 measurement it
cited is kept as history.

**finding-38**: the propagation sweep wrote "only seven are chain-dependent"
into two test-file doc comments while the same commit said **eight** in two
other places. Eight is right — seven need a chopsticks fork and `smoldot_smoke`
needs live Paseo.
disposition: both corrected, line-for-line so the attestation citations into
those same files did not move again.

**finding-39**: three further propagation misses, all in live documents and none
of them verification records — the risk analysis still said nine targets reach
`client.rs` in two places and carried a stale 501-line denominator;
`2026-08-26-ratchet-gap-analysis.md` still said all nine integration targets need
`on-chain/scripts/node_modules`, word for word the claim the sweep corrected in
the unit config; and the hazard register still cited `units.yaml` as where
`depends_on` edges live, in a file the same sweep had edited.
disposition: all three corrected.

**finding-40**: `soup.md` said the gap between the 407 lockfile entries and the
203-crate closure "is the dev-dependency closure". Measured: dev and build edges
take the tree to 262, so they add **59**. The remaining ~145 entries are feature-
and target-gated.
disposition: corrected. The three measured figures were each right; the sentence
joining them was not.

**finding-41**: `soup.md` argued from "three keccak implementations in one
binary", and there are **two**. `keccak-hash` 0.11.0 depends on `tiny-keccak`
2.0.2 — the same crate at the same version this unit imports — so it is a
wrapper, not a rival. That inverts half the paragraph's own argument.
disposition: rewritten. The surviving point is sharper than the original:
switching to `keccak-hash` would call the identical implementation and could
never disagree, so the switch actually worth guarding against is the one to the
`sha3` → `keccak` 0.1.6 path, which the paragraph had never named.

**finding-42**: the region-coverage discrepancy (42.60% measured against the
42.45% the config narrates) was written off as bolero run-to-run variance.
disposition: **the explanation was wrong and is replaced.** Two runs agreeing
exactly is not variance. The likely cause is that this change's one new test is
the first to drive `parse_root_updated` down the error path of
`decode_uint256_to_u64(&topics[2])?`, taking that region for the first time —
deterministic, which is why the runs agree. It is stated as the likely cause,
not a measured one, and it means **this change moved a figure no document in it
had noticed moving**.

**finding-43**: `PR-d4nye8` cited `endpoint.rs:62-68` for "binds on `127.0.0.1`",
which is not in those lines — `.bind()` is called with no address there, and the
claim comes from the doc comment at `:50`.
disposition: corrected, with the halves cited separately. The relay half, which
is what the argument needs, was always right.

**finding-44**: the plan said T11 was "eighty lines below" a paragraph at line
67. T11 is at line 669.
disposition: corrected to a line reference that can be checked.

**finding-45**: (raised, not a defect) the three bolero targets print no
`test result:` line, so the headline "73 passed" is the sum of ten targets, not
thirteen, and a bolero failure appears only as a non-zero exit.
disposition: no change; the figure was already reported that way in this record's
gate table, which lists the bolero targets on their own row with their own
criterion. Recorded here so the reading is explicit rather than inferred.

**finding-46**: (raised, not acted on) replacing the unreproducible "580 of 699
lines" with llvm-cov's "336 of 491" swaps an **item-level** figure for a
**file-level** one — SDD-3b8zef's share of `client.rs` for all of `client.rs` —
and the five sites do not say so.
disposition: accepted as accurate but not equivalent. The item-level figure was
withdrawn because no reader could reproduce it, and no honest item-level
measurement exists without a coverage tool that can attribute lines to items.
Recorded in Gaps rather than papered over.

### Round 5 — the round that reviewed round 4's fixes

Round 4 was meant to be the last. It was offered to the owner as such, with the
alternative of one more round, and the owner chose one more round. That choice
is the reason the central defect below was found, and it is recorded here
because the argument for stopping had looked sound: round 4's findings were
disposed of, the gate was green, and nothing was outstanding.

Three reviewers again. One was given the corrected classification rule and asked
the question round 4 had not asked of its own fix — **does the corrected rule
change the verdict on the two clauses the sweep cut?** It does. Twenty-two
findings, and one of them reverses a deletion in a class C requirement.

**finding-47**: **round 4's corrected rule invalidates one of the two cuts, and
round 4 did not follow it through.** LLR-xv7auy's distinctness clause — "so that
two fields of equal width cannot be substituted for one another" — was cut on a
single green probe. That probe replaced `OrgPubKey`'s declaration in
`src/types.rs` with `pub use crate::types::OnChainRootHash as OrgPubKey;`, which
also deletes the *name*, so the assertions judging the clause could no longer
run against it. The probe measured the test's dependence on a name, not the
clause.
disposition: re-measured and **restored**. A probe that performs the same
forbidden substitution and leaves the tests able to observe it exists: retype
the `org_pub_key` field from `OrgPubKey` to `OnChainRootHash` at its three sites
in `src/state.rs` and its three construction sites in `src/decode/v_paseo_ah.rs`,
touching neither `src/types.rs` nor `src/lib.rs`. Measured: `cargo build --lib
--features test-support` rc 0, no warnings — publicness, newtype-ness and all
four widths stand untouched — and the gate exits **101** with **eight `E0308`**
across five targets (`contract_address_filter`, `decode_org_state`,
`decode_revive_event`, `runtime_version_dispatch`, `fuzz_event_round_trip`). The
clause is **RED** and is back in the requirement. The outcome table is now
**99 RED / 1 GREEN**.

**finding-48**: the rule round 4 wrote was missing a tie-break, which is *why*
finding-47 was possible. The sweep had already reasoned correctly about this
exact shape for LLR-e6skvu — an arbitrary third signature is green, a
drifted-ABI third signature reds, clause kept — and then did the opposite for
LLR-xv7auy in the same section, on the same evidence shape, with no stated
reason for the difference.
disposition: the tie-break is now stated. **Falsifiability is existential**: a
clause is evidence-bearing if *some* mutation that makes it false reds a named
test. One green falsifier proves nothing about the clause; it proves that one
mutation was a poor probe. **GREEN is a verdict about the search, not about the
clause**, and may be recorded only after the obvious falsifiers have been tried.

**finding-49**: **round 4's other reclassification was right in outcome and
wrong in evidence.** For LLR-z8rrkr's "as one value", round 4 re-typed
`parse_revive_event` to return `Result<Option<([u8; 20], Event)>, DecodeError>`
and called that two values. A 2-tuple **is** one value, so that mutation does
not make the clause false at all; what it falsifies is the tests' dependence on
the struct's name and field names. It reds, and a red for the wrong reason is
not evidence.
disposition: re-measured on a mutation that does falsify the clause — return the
address out of band, `fn parse_revive_event(&self, event_bytes: &[u8],
contract_out: &mut [u8; 20]) -> Result<Option<Event>, DecodeError>` at
`decode/mod.rs:56` and `decode/v_paseo_ah.rs:88`, writing `*contract_out =
contract;` before `Ok(Some(event))`, with `client.rs` re-assembling at the call
site so the library's own consumers are unaffected. Now the address really is
carried separately from the event. Measured: library clean (rc 0), gate **101**,
`E0308` in five targets. **RED**, on that mutation and not round 4's.

**finding-50**: the outcome table blended compile-time reds into runtime reds,
so a reader could not tell which rows rest on an assertion and which on the
compiler.
disposition: the table now has three rows — **RED (runtime) 96**, **RED
(compile-time) 3**, **GREEN 1** — and a passage saying what a compile-time red
does and does not establish: that the declaration is *referenced* in the shape
the clause states, by targets this unit's gate compiles; no behaviour is
exercised and no assertion runs. Weaker than a runtime red, and recorded as its
own row rather than folded in.

**finding-51**: **the deviation argument claimed something had been filed that
had not.** It said the third option was "filed as an owner item rather than
settled by the author". Nothing was filed, and declining an option *is* settling
it. This is the same defect class the record catalogues elsewhere — an assertion
that a control exists where none does — committed in the paragraph arguing that
the deviation is honestly recorded.
disposition: conceded in full. The register entry now exists in
`docs/plans/2026-09-05-ratchet-setup.md` under the SDD-3b8zef gap, and it
records the option, the objection to it, and that the author of this change
declined it, so a reader who thinks the judgement wrong can find the judgement
rather than a claim that somebody else will make it.

**finding-52**: "it reddens this unit's gate on a tree where nothing is broken",
the first reason given for declining the third option, **switches criteria at
the convenient moment**. Under the criterion the same section argues from, a
class C item missing its required low-level requirements *is* a defect and a red
gate would be accurate.
disposition: withdrawn, and recorded as withdrawn rather than quietly dropped.
So is the companion claim that a repository red for recorded reasons teaches its
readers to stop reading red — an assertion about people that this change
measured nothing about, and one that would equally forbid ever reddening a gate
for any recorded defect.

**finding-53**: the third option was **overstated, not merely unweighed**. Round
4 judged it stronger than what was done, on the ground that it makes the gap
machine-visible. It does not, and the reason was already recorded two paragraphs
away and going unused: MISSING-TEST is discharged by a `verifies:` reference in
*any* text file under `test_paths`, committed or not, in a test target or not.
disposition: the red option three raises is **one line away from a green**, and
typing that line converts a recorded class C deviation into a clean gate that no
longer reads as a deviation, leaving no argument behind. That erasability is the
real reason to decline it, it is sufficient on its own, and it now carries the
paragraph that two withdrawn reasons had been carrying.

**finding-54**: **`SubscribedEventStream` is a third partial item, and its gap
*is* of SDD-3b8zef's kind** — which this change had explicitly denied. The type
(`client.rs:100`) is produced by nothing but `subscribe` (`client.rs:287`,
`:343`, `:432`), which is SDD-3b8zef. No gated test can construct one; the only
tests that name it are three of the five ungated targets. When this change moved
`ClientError` and `SubscribedEventStream` together out of SDD-3b8zef into
SDD-5wamsz, the argument was made entirely about `ClientError` — chain-free,
re-exported under a default-on feature, reachable from the gated tests — and
`SubscribedEventStream` travelled with it on no argument at all, sharing none of
those properties.
disposition: recorded in the decomposition rather than left for the next reader.
**The relocation narrowed the deviation's stated scope without narrowing the
unreachable surface.** Of the three partial items, two are writable requirements
nobody has written and the third is a behaviour no gate can reach.

**finding-55**: **round 4 broke five more line citations with the exact
mechanism it wrote a lecture about, in the same commit that wrote the lecture.**
The commit correcting finding-28's seven stale citations also rewrote
`type_widths.rs`'s header — **+2 net lines** — and re-resolved nothing in that
file, so `type_widths.rs:143` in the plan and in the decomposition, and `:166`,
`:167`, `:175` in the decomposition, moved by two and were not updated. Two of
the five then landed on lines the named mutation cannot fail, which is the
"argues against its own evidence" failure finding-28 catalogued.
disposition: all thirty citations re-resolved against the tree **as it stands
after the last prose edit**, which is the only moment at which the check is
worth anything. The remediation note is extended to record that round 5's own
edits to that header moved the four back **by coincidence, not by fix**, so no
one mistakes one for the other. The lesson is now stated as a rule — line
citations into files a change edits are taken once, after the last edit to those
files, and re-taken whenever any later commit touches them — with the note that
three separate commits in this change broke it.

**finding-56**: "thirty-three red → green citations" was **thirty**.
disposition: corrected at both sites. The count had been carried from a draft in
which three attestations cited the same line twice.

**finding-57**: "five integration targets" in finding-25's disposition listed
**six**.
disposition: corrected to six. The enumeration was right and the number in front
of it was wrong, which is the failure mode finding-23 recorded for PR-d4nye8.

**finding-58**: the anvil count was measured a third time and had been stated
wrongly twice. Round 4's replacement — "twenty-one occurrences across ten
documents" — was measured **before its own edits landed** and named a document
that by then contained none.
disposition: restated as **eighteen occurrences across nine documents, plus two
in the `Makefile`**, measured after every edit in this change. The claim that
this change "stopped there, deliberately" is **withdrawn**: it also corrected
the single pre-existing use in `docs/plans/2026-08-26-ratchet-gap-analysis.md`
while fixing an unrelated count, so it stopped at every *document* it had no
other reason to open. Three live documents still assert the phantom and the
deferral for those stands.

**finding-59**: **round 4 wrote two fresh uses of the phantom into
`docs/plans/2026-09-10-on-chain-client-risk-analysis.md`** — a change that
declares the phrase a phantom while still adding new uses of it.
disposition: removed. This is the second time a round of this review has done
it; finding-30 recorded three written by the propagation sweep. The register
entry now says so in those terms rather than as untidiness.

**finding-60**: "five source files were ever mutated" was no longer true, and
the digest cordon that backs the red → green attestations **does not cover
rounds 4 and 5**. Round 4 mutated `src/decode/mod.rs` and round 5 also mutated
`src/state.rs`.
disposition: the claim is qualified where it appears, and the uncovered mutation
is recorded as a gap rather than folded into the cordon. All five original
digests were re-verified after every round and hold; what is *not* established
by them is the integrity of the two files they never named.

**finding-61**: coverage prose re-based to 491 lines sat under a table still
reading 501, handing a reader "336 of 491" pointing at a 336/501 table.
disposition: a basis note added rather than the table re-measured. The table is
the measurement as it stood when written; the denominator later fell to 491 when
`types.rs` shed ten fully-covered lines to the relocation. `client.rs` is 336
lines at 16.37% on either basis, which is why the conclusion drawn from it is
unaffected.

**finding-62**: `soup.md` argued from "three keccak implementations in one
binary" and there are **two** — `keccak-hash` 0.11.0 depends on the same
`tiny-keccak` 2.0.2 — and the provenance given for the third was wrong.
disposition: corrected to two, with the real provenance: `keccak` 0.1.6 arrives
via `sha3` 0.10.9 → `sp-crypto-hashing`, not via `primitive-types`. The
conclusion the paragraph drew is weakened accordingly, and "could never
disagree" is softened to what the versions actually support.

**finding-63**: `soup.md` said the gap between the 407 lockfile entries and the
203 enumerated was the dev-dependency closure. It is not — dev and build add 59
to 262, leaving roughly **145 of the composition unenumerated**.
disposition: corrected to say what is enumerated, what is not, and that the
remainder is not accounted for. An inventory that explains its own gap with the
wrong explanation is worse than one that admits the gap.

**finding-64**: the README's "every row now names the same source in all three
tables" was false for three of the nine rows.
disposition: narrowed to "six of the nine rows", and the two cells that were
wrong are corrected — SDD-bw7v5x to `internals::solidity_mapping_slot`,
`internals::increment_slot` and SDD-m59zrg to `internals::scan_step`,
`internals::ScanStep`, where both had read bare `internals`.

**finding-65**: two further sites described SDD-v2rtka as "fully gated".
disposition: qualified at both, in the SRS and in the ratchet-setup register.
Finding-34's correction had been applied where it was found and not swept for.

**finding-66**: the const citation `:73-83` in the hazard file had moved to
`:106-116`.
disposition: corrected. Same mechanism as finding-28 and finding-55, in a file
no earlier round had re-resolved.

**finding-67**: (raised, and the cut confirmed) LLR-mzh8df's code-sharing clause
was cut on a single probe, and finding-48's tie-break demands a stronger search
before a GREEN stands.
disposition: re-probed and the cut **stands**. Round 5 inlined byte-equivalent
code at *both* call sites, so that no function is shared by the two paths at
all: still **73 passed, 0 failed**. `decode_uint256_to_u64` is a private `fn`
named by no test target in code, so no compile-time carrier exists either, and
none can be manufactured without changing behaviour — which is this LLR's other,
already-red clause. This is the only clause cut in the whole sweep, and it is
now the only one to have survived two independent searches.

**finding-68**: (raised, not fixable) commit `764c386`'s message claims "the
cross-unit reference is dropped" for a fix that commit does not contain — the
`PR-d4nye8` citation had already been removed by an earlier commit.
disposition: recorded here rather than fixed, because fixing it means rewriting
the history of a branch the owner is about to sign. It matters less than it
otherwise would, because **this change lands as a single squashed commit whose
message is written fresh**, and that message does not repeat the claim. It is
named so that a reader following the branch's own history rather than the squash
finds the discrepancy already known.

### Two findings raised by writing this record, after round 5 closed

No reviewer raised these. They were found while assembling this file for the
merge, which is the first time the whole change was read in one pass with the
gate frozen. They are numbered into the same sequence and disposed of like any
other, because the alternative is a record that reports only what someone else
caught.

**finding-69**: **this record's own gate section carried the defect it
catalogues.** An earlier draft of the coverage-discrepancy paragraph had been
spliced into the middle of a code span in its own successor — `…the error path
of decode_uint256_to_u64(` + a whole stale paragraph + `topics[2])?…` — so the
section stated both the superseded account of the moved region figure and the
account that replaced it, as though each were the conclusion. It is the same
splicing failure recorded against the ledgers in round 3's sweep and again in
round 4.
disposition: rewritten as a single paragraph carrying the round 4 account, which
is the one that stands, with the superseded reading dropped rather than left
adjacent to it. The reason it survived five rounds is recorded in Gaps: **no
round of review was ever given this file.** Every reviewer had the diff, the
ledgers and the plan; none had the record.

**finding-70**: **ten notes added by rounds 4 and 5 date themselves 2026-09-28,
and both rounds landed on 2026-09-29.** The five commits carrying them —
`217bedc`, `18d9552`, `764c386`, `43a4abb`, `fc9f180` — are all dated
2026-09-29. The wrong date was inherited from the change's own start date, which
is when the files were first written, and never re-taken when the rounds ran.
disposition: all ten corrected to 2026-09-29, in `type_widths.rs`'s header,
`soup.md` (two), the decomposition (three), the risk analysis, the ratchet-setup
register (two) and PR-d4nye8. The `type_widths.rs` correction is
character-for-character the same length, so no citation into that file moves. Two
dates are **not** changed because they are right: the sweep cut LLR-xv7auy's
clause on 2026-09-28, and the owner was asked about the deviation on 2026-09-28.
This is the same failure mode as finding-28 and finding-55 in a different
dimension — a fact captured once, at the start, and carried forward past the
point where it stopped being true.

### What five rounds cost, and what they bought

Rounds 1 and 2 found prose defects. Round 3 found that the LLR ledger contained
clauses no test could falsify, which is what produced the two mechanical sweeps.
Round 4 was the first to review the sweeps and found their classification rule
wrong. Round 5 found that round 4's corrected rule had a consequence round 4 had
not applied, and that consequence restored a deleted clause in a class C
requirement.

**Every round after the second found a defect that the round before had
introduced or left standing in its own fix.** Round 4 broke five citations while
fixing seven. Round 5 corrected the evidence for one of round 4's two
reclassifications and reversed the other's premise. That pattern is the argument
for the next round, and it is also the argument against reading a green gate as
completion: the gate was green after round 3, after round 4, and is green now.

What stops it is not exhaustion but a change in kind. Rounds 1–3 found errors in
what the change said; rounds 4–5 found errors in how the change had measured.
Round 5's findings are, with the exception of finding-47, corrections of
counts, citations and arguments rather than of classifications. Finding-47 is
the exception, it is the reason this round happened, and it is now disposed of
with its own measurement recorded. A sixth round would be justified by evidence
that a class of claim remains unmeasured — not by the observation that five
rounds each found something, which will remain true of any number of rounds.

## Gaps

What this change did not establish.

- **SDD-3b8zef has no low-level requirements**, and class C requires them per software item. The deviation is recorded in the item's own text, in the architecture README's Overview, and as an owner item in `docs/plans/2026-09-05-ratchet-setup.md`. Closing it needs either further extraction of chain-free decisions or a gate that can run the chopsticks targets — both code or infrastructure changes owed their own red-first cycle.
- **An LLR for `ClientError`'s `Display` and its `From<DecodeError>` is writable and gate-verifiable today and is not written.** Found by the independent review, not by the author. It needs a new gated cargo target, which this tooth deliberately does not add. It is the cheapest of the three open items on this gap.
- **A stronger option for the deviation was identified and not taken, and that is the weakest point in this change.** Review round 4 observed that minting SDD-3b8zef's low-level requirements and leaving them *unannotated* would make `check-trace.sh` emit MISSING-TEST and exit 1 — the gap machine-visible, and visible until closed, instead of resting in prose. On the criterion the deviation section itself argues from, that is better than what was done. It was declined because it reddens a gate on a tree where nothing is broken while org-node's gate is already red, which is a judgement about this moment rather than a refutation. Filed as an owner item. A reader who thinks the judgement is wrong should know the option was seen, conceded, and passed over rather than missed.
- **Nothing enforces the deviation, or the class C obligation it deviates from.** `check-trace.sh` never reads `safety_class`; `check-units.sh` reads it only to rank classes across a dependency edge. No gate anywhere requires a class C software item to carry low-level requirements. And `verify_commands` is executed by no script in this toolkit — the commands are run by whoever follows the merge process. So every claim in this record that a gate "passed" rests on the commands having actually been run, which this record attests to and no script checks.
- **The withdrawn item-level coverage figure has no honest replacement.** "Roughly 580 of `client.rs`'s 699 lines" was SDD-3b8zef's own share and could not be reproduced; the measured figure that replaced it at all five sites — 336 of 491 lines at 16.37% — is llvm-cov's for *all* of `client.rs`, which is five items, not one. The substitution is accurate and is not equivalent, and the sites do not say so. No item-level number exists without a coverage tool that can attribute lines to software items, which this project does not have.
- **Coverage falls short of the class C target on both limbs** — statement 41.75%, decision unmeasured everywhere (llvm-cov Branches `0/0`). Accepted by the owner on 2026-09-28 as a documented gap, unchanged by this change, and tracked for tooth 5. `org-node` and `app` configure no `coverage_command` at all.
- **Fourteen of the thirty-two LLRs carry only one side of the class C robustness rule.** Twelve have their counterpart in a sibling LLR of the same item, because this ledger writes an accept path and a refuse path as two LLRs rather than one. Two do not: LLR-xv7auy is a genuine exemption, constraining a declaration with no runtime input domain, and LLR-b4p32h has no other side at all — the pairing first offered for it crossed an item boundary and pointed at another normal case, which the propagation sweep corrected. The rule is met at the software-item level and is **not** met per-LLR for fourteen, which the decomposition states plainly and lists.
- **org-node's gate is red** — 44 passed, 7 failed — and this change does not fix it. Filed as PR-d4nye8 with what is established (not this change, not new code meeting old tests, not the sandbox, and not external relay: all three failing targets bind loopback with the relay disabled) separated from what is not (machine environment versus latent nondeterminism). The single most valuable next measurement is named there: whether it reproduces on another machine.
- **No advisory-database scan** (`cargo-audit`, `cargo-deny`) backs the SOUP inventory; neither tool is installed in this toolchain. Declared in `soup.md` as a gap in the evidence rather than a clean result.
- **The red → green attestations cannot be re-derived from the repository.** They are the only record that the iron law was honoured; what is independently checkable — the five source digests, the empty diff against `on-chain-client/src`, and whether each test would red on a behaviour break — was checked by the reviews and holds.
- **The digest cordon does not cover rounds 4 and 5.** The five tabulated SHA-256 digests were the guarantee that the falsifiability sweep's 102 mutations left no residue, and they hold, re-verified at every gate run from 5 to 8. But round 4 mutated `src/decode/mod.rs` and round 5 mutated `src/state.rs`, and neither file is in that table. Their restoration rests on `git status --porcelain` being empty and on the empty diff against `on-chain-client/src`, which is real evidence and is weaker than a named digest. Anyone extending the cordon should add both files to the table rather than argue the two forms of evidence are equivalent.
- **`SubscribedEventStream` is a second instance of SDD-3b8zef's problem wearing another item's name.** It is produced only by `subscribe`, no gated test can construct one, and it now sits in SDD-5wamsz on an argument made entirely about `ClientError`. No low-level requirement refines it, and none can be gate-verified without a target this unit's gate does not run. Found by review round 5 (finding-54); recorded, not closed. It belongs in the same owner item as SDD-3b8zef's missing LLRs, and is not separately filed.
- **Three live documents still describe on-chain-client's ungated targets as needing "a chopsticks fork or anvil", and no target uses anvil.** `docs/plans/2026-09-10-on-chain-client-risk-analysis.md`, `on-chain-client/docs/requirements/2026-09-10-chain-reading.md` and `on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md`. This change corrected every use it wrote and one it did not; the rest is deferred to an owner item, because correcting some documents and not others would recreate the inconsistency the sweep existed to remove. Four verification records also carry the phrase and are deliberately not amended.
- **Roughly 145 of `soup.md`'s 407 lockfile entries are not accounted for.** 203 are enumerated, dev and build dependencies add 59 to a 262 closure, and the remainder is neither enumerated nor explained. `soup.md` had explained it as the dev closure, which review round 5 showed it is not (finding-63); the file now records the gap instead of the wrong explanation. An inventory with an admitted hole is what this change delivers, not a complete one.
- **This verification record was written by the author and reviewed by nobody.** Every round of independent review was given the diff, the ledgers and the plan; none was given this file. It is where the change's own account of its defects lives, and it is the one document in the change that no reviewer read. One splicing defect in it — an earlier draft of a paragraph spliced into the middle of a code span, in the section about the gate — survived all five rounds for exactly that reason and was found only when the record was written up for merge. If verification records are ever amended, this is a candidate; whether they may be amended at all is itself an open owner item.
