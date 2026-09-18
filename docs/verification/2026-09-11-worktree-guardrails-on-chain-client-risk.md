# Verification — worktree-guardrails-on-chain-client-risk (2026-09-11)

branch: worktree-guardrails-on-chain-client-risk
reviewer: four independent review rounds, each a fresh dispatched subagent in its own nested worktree at `.worktrees/worktree-guardrails-on-chain-client-risk-review{,2,3,4}`, given the diff, the artefacts and the whole repository but no implementation narrative and no chat history. Rounds 2, 3 and 4 were additionally given no knowledge of the preceding rounds' findings, so a mis-applied fix had to be found as a defect rather than recognised as a correction. Each ran every unit's `verify_commands` itself and reported the counts it saw.
verdict: merge. Round 4 raised seven findings, two above the stated merge bar, both wording rather than code, and neither touching a severity, a probability, or the conclusion; both are fixed. Across four rounds 41 findings were raised, every one accepted and dispositioned, none disputed. Round 4's clearance is the substantive one: all 21 requirements satisfied by code checked against the implementing function rather than its own prose; no test reimplementing what it tests; all thirteen probability estimates re-derived against the current scale and every one reproducing; nothing leaning on recoverability; no residual claim stronger than its evidence; all 19 not-minted controls either queued with an owner and a date or explicitly exempted with a reason — so no ISO 14971 gap.
reproduced: yes, for every defect this change fixes and for several of its own claims. The spoofing defect (PR-p5ngya) was reproduced as a decoder that drops the emitting address and a call site that compares nothing, and the fix was then proved by deleting the comparison and watching three named cases red. The change's own prescribed red-by-mutation for that control was run, **watched staying green**, and the false green removed by extracting the decision rather than by rewording the record. Review round 3 reproduced a claim's unfalsifiability by inverting the evaluation order inside a total boolean predicate and observing no test result change; review round 4 reproduced a requirement's ambiguity by narrowing the code to one reading of it and watching a named case red. The measurements this record carries were re-run and reproduced by later rounds: round 3 re-ran fourteen of them, round 4 four more.

Change: the `on-chain-client` unit's first ISO 14971 risk analysis — six hazards, nine risk controls, twenty-one derived requirements with gated tests, five problem reports — plus the one code fix its worst hazard needed, and the relocation of every test in the crate to where a gate can read its annotations. Branched from `master` at `50a5254`; 82 commits.
Plan: `docs/plans/2026-09-10-on-chain-client-risk-analysis.md`.
Units touched: `on-chain-client`. Impact set: `on-chain-client` (touched), `org-node` (dependent), `app` (dependent).

## The gate

Sixth and final run, 2026-09-11, in the change worktree at `0450a6d`. Every figure below was measured on the tree under test by the dispatched gate; none is carried forward from an earlier run.

| Gate | Result |
| --- | --- |
| `check-units.sh` (repository) | exit 0 — `units: 4, disclaimed 7; tracked paths 384` |
| on-chain-client `verify_commands` | **72 passed, 0 failed** over ten counted targets, plus three bolero targets exiting on their one-second budget without panicking (261 750 / 38 244 / 239 986 iterations per second) |
| org-node `verify_commands` | **51 passed, 0 failed** over eight counted targets, plus two bolero targets; `quint typecheck quint/protocol.qnt` exit 0 |
| app `verify_commands` | `cargo test` **0 passed, 0 failed** across three targets — a compile proof, as that unit's config states; `npm --prefix app run check` **166 files, 0 errors**, one pre-existing `@types/node` warning |
| Suite total, impact set | **123 passed, 0 failed** |
| `check-ids.sh --allow-draft-files` × 3 units | exit 0, silent on all three |
| `check-trace.sh` × 3 units | exit 0 on all three. on-chain-client: `REQ 21, HAZ 6, RC 9, SDD 0, LLR 0, PR 5`; problems open 4, oldest 1 day, against limits 30 and 10 |
| Coverage, against the class C target | on-chain-client **41.75% lines** (205/491) against a floor of 41, **42.45% regions** (284/669) against 42; org-members **93.50%** (935/1000) against 92 and **92.20%** (1430/1551) against 91. Floors enforced in-command by `--fail-under-*`. **Both remain shortfalls against the class C target, not targets met — see Gaps.** |
| Working tree | clean; `git status --porcelain` empty |

**Exit 0 was not read as a pass.** Three commands report no counts and each was chased rather than assumed. `check-ids.sh` is silent by design, so the gate re-ran its own duplicate harvest over the same pathspec — 109 definition sites, 109 distinct IDs, zero duplicates — proving the scan reads real content. `quint typecheck` is silent by design, so the gate injected a deliberate type error into a scratchpad copy and watched it convict with `Error [QNT000]`, exit 1. The five bolero targets report iteration totals and an exit reason rather than a pass count, which their own headers document; green means the process exhausted its second without panicking.

## Red → green

One row per requirement this change implements, copied from the `red -> green:` lines of the dispatch reports. The subagent that ran each loop is the only party that saw the test fail; this table is where that observation becomes durable. **Thirty-nine mutations were applied across the development tasks and a further eleven during the fix rounds.** Every implementation here pre-dated its test, so red came by mutation throughout — the honest substitute, and stated as such rather than dressed as a TDD cycle.

The nine controls and six hazards carry no `verifies:` test and correctly so: a control is realised by requirements carrying `implements:`, a hazard is mitigated by controls carrying `mitigates:`, and `check-trace.sh` reports neither `UNIMPLEMENTED-CONTROL` nor `UNMITIGATED-HAZARD`.

| Item | Test | Watched red |
| --- | --- | --- |
| REQ-9vwcwc | `log_ownership.rs` (5 cases) | T2: all cases watched failing pre-implementation on `error[E0432]` — the decoder had nowhere to surrender the address. T2b: seven cases on the same error, then three measured mutations, the decisive one **deleting the contract check outright** — 3 failed, 4 passed, exactly the three rejection cases |
| REQ-5upq6n | `contract_address_filter.rs` (3 cases) | T2: watched failing pre-implementation on `E0432`, then under a substituted decoder mutation reporting `left: [0…] right: [153; 20]`. fix2a re-verified after re-pointing the file at the real predicate: deleting the contract comparison now reds **2 of 3**, where before the fix it reddened none |
| REQ-nygs7k | `log_ownership.rs` (2 cases) | T2b: `E0432` pre-implementation; then `admin_filter` ignored (`_ => true`) — 1 failed |
| REQ-52uc8f | `decode_revive_event.rs` | T3 M7: `SIG_GENESIS_INITIALIZED[0]` `0x8e`→`0x8f` — 8 tests red, this among them, every genesis-shaped case collapsing to `Ok(None)` |
| REQ-wnjz9j | `decode_revive_event.rs` (3 cases) | T3 M6: unknown-signature arm made to decode as genesis — `an_unknown_first_topic_yields_nothing` returned `Err(InvalidTopicCount)`; M13: empty-topics guard removed |
| REQ-88fp2h | `decode_revive_event.rs` (8 cases) | T3 M4/M5 (`!= 2`→`< 2`, `!= 3`→`< 3`) and **M10/M11 (`> 2`, `> 3`), which panicked inside `src`** — `index out of bounds: the len is 1 but the index is 1` at `v_paseo_ah.rs:141`, and `len is 2 but the index is 2` at `:169` |
| REQ-twdu84 | `decode_revive_event.rs` (4 cases) | T3 M1: `topic[..12]`→`[..11]` — the padding sweep red at **position 11**, `Ok(Some(Genesis{…}))` where `Err(InvalidAddressTopic)` was required |
| REQ-axcxf7 | `decode_revive_event.rs` (5 cases) | T3 M8: trailing-byte check removed; M9: topics decode `unwrap_or_default()` — red at cut 86. **The fifth case, added by fix6b for the requirement's untested second clause, passed first time and is recorded as such: no red→green, none manufactured** — the SCALE codec already refused that input; what was missing was evidence, not a fix |
| REQ-sx5b6g | `fuzz_decode_org_state`, `fuzz_parse_revive_event` | **No cycle applies, by design.** The never-panic-on-arbitrary-bytes property pre-dates this change; T9 added annotations to targets already exercising it. Attested as *measured*, twice, with figures, rather than dressed as a red→green |
| REQ-n6v896 | `fuzz_event_round_trip` | fix4: `prev_root_hash.copy_from_slice(&data[64..96])` commented out, leaving the field defaulted. bolero shrank to a single distinguishing byte — `left: […0,0,…]` against `right: [175,0,…]`, every other field round-tripping |
| REQ-hd6m9d | `runtime_version_dispatch.rs` (5 cases) | T5: the plan's mutation (`PASEO_AH_SPEC_VERSION \| _ if true`) reddened all four refusals; **it left the positive case green, so a second mutation was written for it** (`if false`, dropping the pinned arm) — that case proves *which* decoder came back |
| REQ-tkhe3u | `h160_mapping.rs` (2 cases) | T6: reverse path's early return disabled — returned the keccak answer instead of `[0,1,…,19]`, and the keccak of `[0xEE; 32]` instead of `[0xEE; 20]` |
| REQ-rz7fja | `h160_mapping.rs` (3 cases) | T6: `account_id_32[20..32]`→`[21..32]` — red at the eleven-of-twelve near-miss **and** at the sweep's position 20 |
| REQ-9m2rnd | `storage_slot_layout.rs` (6 cases) | T7 M2 (`buf[12..32]`→`[0..20]`, 4 failed), M3 (`map_slot` dropped, 2 failed — the measured justification for insisting on a non-zero index), M4 (`to_be_bytes`→`to_le_bytes`), M6 (hash one word — the only mutation reddening the zero-admin case) |
| REQ-xudf25 | `storage_slot_layout.rs` (6 cases) | T7 M1 (`carry = sum >> 8`→`0`, 4 failed), M5, M7. **The plan's own suggested mutation was run first and measured inert** — 11 passed, reddens nothing — and recorded as a measured negative before the substitute was used |
| REQ-2qa5r5 | `type_widths.rs` | fix4: `#[repr(align(N))]` per newtype, four separate reds — `32/20`, `64/32`, `64/32`, `16/8`. **The obvious mutation (`u64`→`u32`) was rejected as evidence**: it fails to compile the library, so the named test never ran, and a test that did not execute observed nothing |
| REQ-4astjb | `decode_org_state.rs` (5 cases) | T4 M1 (`!= 96`→`< 96`) and **M2 (`> 96`), which panicked inside `src`** at three unguarded `copy_from_slice` calls — reached by the **empty blob**, the most realistic abnormal input of the set |
| REQ-9wwenn | `decode_org_state.rs` (5 cases) | T4 M3 (`bytes[..24]`→`[..23]`, red at position 23), M4 (`[..25]`, the mirror), M5 (`[24..32]`→`[23..31]`), M6 — which is only detectable because the relocated test replaced uniform fills with ascending runs |
| REQ-ntn4ss | `best_lane_reorg_rule.rs` (6 cases) | T8 M1 (`n <= prev.number`→`<`), M5 (reorg detection disabled), M6 (the `*last` advance removed) |
| REQ-gr2ver | `best_lane_reorg_rule.rs` (3 cases) | T8 M3 (dedup guard narrowed — a spurious re-emission **and** a false reorg report where `Skip` was required) and **M4 (dedup keyed on number, not hash) — which makes the rule discard a reorg's replacement block entirely** |
| REQ-5zux82 | `best_lane_reorg_rule.rs` (5 cases) | T8 M2 (`from` = `prev.number`) and **M7 (`from` overshoots) — producing `from 10..=9`, a descending span that iterates zero times and silently reads no blocks** |

**Two mutations are worth carrying beyond their rows.** T8's M4 and M7 each lose membership events *silently* — one by discarding a reorg's replacement block, the other by producing a span that reads nothing — and neither surfaces as an error anywhere. They are the argument for extracting that rule out of a closure where no test could reach it. And T3's M10/M11 with T4's M2 show that this crate's length and count guards are the only thing between untrusted chain bytes and a panic in a `no_std` library whose crate root denies `panic`: in each case the plan's one-sided mutation proved only half the bound, and the unproven half was the one where failure is a crash rather than a wrong answer.

## What was wrong, and what was built

**The unit had no risk analysis, and its evidence counted for nothing.** Every test in the crate — twenty-three of them, all passing — lived in `#[cfg(test)]` modules inside `on-chain-client/src`. This unit's `test_paths` is `on-chain-client/tests`, so an annotation written in `src` is read by no gate. The tests were real; the traceability was absent by construction, and no gate could have said so.

**One defect in that code was live, and it was the unit's worst.** `parse_revive_event` decoded the `ContractEmitted` payload's `contract` field into `let _contract` and dropped it; `event_matches_contract` was `let _ = (ev, contract); true`. The client's own doc-comment claimed "events from other contract addresses are filtered out before reaching subscribers", which was false. Any contract on Asset Hub could emit a log carrying the OrgRegistry signature hash and a victim organisation's indexed admin, and every subscriber would receive it as genuine, with an attacker-chosen Membership root. That is PR-p5ngya, and this change fixes it: the emitting address is carried out of the decoder as `EmittedEvent` and compared against the address the reader was constructed for.

**The fix's first attempt was verified in the wrong half, and the measurement said so.** The prescribed red-by-mutation — disable the comparison, watch the spoofing test fail — did not fail: three tests still passed with the check switched off. The decoder's half was gated; the caller's half sat inside `async fn decode_contract_events`, which no gated test reaches, and no chopsticks target could reach it either, since each deploys exactly one OrgRegistry and so has no second contract to spoof from. The remedy was structural, not editorial: extract the decision into a pure `log_is_ours` and gate the decision. Deleting the comparison now reds three named cases. **That measurement — a prescribed step run, producing the answer nobody wants — is recorded in the register's conclusion rather than summarised away**, because a report of "red → green, control verified" for that task would have been literally true and substantively false.

**What was built.** Six hazards with identifiers and nine risk controls, five further distinct hazards in prose, nineteen controls identified but not minted, and a residual-risk table in which no assessed risk is acceptable. Twenty-one derived requirements realise the controls, each verified in `on-chain-client/tests`: the twenty-three relocated tests, plus the class C abnormal-input cases they lacked — twelve padding positions swept, twelve marker positions, twenty-four epoch-slot positions, the empty blob, 32/64/128/192-byte slot miscounts, `u64::MAX` as a mapping index, the 256-bit wrap, and the reorg rule's first tests of any kind. Two source refactors made the untestable testable: the best lane's per-notification decision extracted out of a closure inside `best_lane`, and the ownership decision out of an async function. Nine gated test targets declared with `required-features`, so a target named in the gate cannot silently not run, and named in the same order in `verify_commands`, in CI, and in the coverage recipe.

**The conclusion.** Every assessed risk is S3 and none is acceptable under this project's matrix; the unit's overall residual risk is **UNACCEPTABLE** against the intended use — the same conclusion `org-members`' register reached for the membership capability and `org-node`'s for the node, and it does not change either. Nine claims are recorded as held by review rather than by a gate: the contract's `ZeroValue()` guard and its storage layout, the `h160_of` ground truth, the two event-signature literals, the width test's ABI derivation, the pinned `spec_version`'s value, the construction-time refusal, the best/finalised distinction, and the wiring between the ownership decision and its call site. Each is true today, established in this change by reading, and re-established by nothing.

## Review

Four rounds, 41 findings, every one accepted and dispositioned, none disputed. Rounds 2, 3 and 4 were given no knowledge of the preceding rounds, so several findings below are defects the *fix* rounds introduced — which is what the rounds were for. The full dispositions, with the measurements behind them, are in the plan's four review sections; each block below carries the finding in the reviewer's terms and what changed.

### Round 1 — twelve findings

**finding-1**: Five documents claim all twenty-three `#[cfg(test)]` tests were relocated out of `src`; twenty-one were. `on-chain-client/src/types.rs:36-55` still holds two, unannotated and invisible to `check-trace.sh` — and `types.rs` is explicitly in the analysis scope. The gate's own `lib 2` line was the proof, unexamined.
disposition: fixed (fix2a, fix2b). Both relocated to a new gated target `tests/type_widths.rs`; the width test strengthened to derive each width from the ABI region the decoder reads and annotated `verifies: REQ-2qa5r5`; the `Display` test relocated without an annotation, which is legal and stated. Reddened four ways by `#[repr(align(N))]`, one per width.

**finding-2**: All three tests in `contract_address_filter.rs` carry `verifies: REQ-9vwcwc` and cannot go red if the ownership decision breaks: the file reimplements the predicate locally as `is_from_configured_contract`, commented "The caller's decision, verbatim". Measured: with the real comparison deleted, this file reported 3 passed while `log_ownership` reported 3 failed.
disposition: fixed (fix2a). The local copy deleted and `on_chain_client::test_support::log_is_ours` imported. Re-measured: the same mutation now reds 2 of 3 in that file. A copy of a predicate tests the copy.

**finding-3**: The coverage floors were lowered from 52/58 to 41/42 citing, in the past tense, a verification record that does not exist — and the plan's own section still said "No floor was touched", contradicting the Makefile and a green run.
disposition: fixed (fix2a for the Makefile and config wording, fix2b for the plan). Reworded to name where the decision is recorded without asserting the file already exists; the record is written at step 6b, after review, so the citation was true-in-advance, which is not true.

**finding-4**: HAZ-xfg9cz names three routes to its hazardous situation; its P1 argument covers two and silently drops the third — a runtime-shape change, which HAZ-8s5chy assesses at P2 for the same event. The file gives one event two probabilities.
disposition: fixed (fix2b). Raised to P2, argued as the maximum over its three routes. The re-argument exposed a second gap, now recorded: that third route has **no control at all**, and a slot reading absent after a shape change takes the `Ok(None)` shortcut rather than the width guard — so its residual went from one count to two.

**finding-5**: RC-8w9wtp and REQ-sx5b6g claim no decoder "allocates from an unchecked length taken out of the input". Nothing in `src` checks that; every such allocation is inside `parity-scale-codec`, a SOUP item, and `soup.md` is still the empty template. The fuzz targets evidence panic-freedom, not allocation size.
disposition: fixed (fix2b). The allocation clause dropped from both, leaving the property the evidence supports; the `parity-scale-codec` dependency recorded in HAZ-95sc43's residual with the same treatment the register gives `OrgRegistry.sol` — a load-bearing behaviour implemented outside this unit.

**finding-6**: Not-minted controls 5, 11 and 14 have no owner, date or checklist item — and 11 and 14 are the sole named control for prose hazards assessed S3 and not acceptable. ISO 14971 requires options for an unacceptable risk to be carried to a decision.
disposition: fixed (fix2b). All three added to the setup checklist with owner and decide-by date, and the two independently numbered not-minted series in that file disambiguated.

**finding-7**: Two behaviours have no requirement: which two event signatures are recognised — the ABI binding to the contract — is pinned by a test annotated to a requirement stating only the negative; and `fuzz_event_round_trip` asserts an inversion property while annotated to REQ-sx5b6g, whose arbitrary-bytes property that target never exercises.
disposition: fixed (fix2a, fix2b, fix3). RC-675a3h minted with REQ-52uc8f for the ABI binding; REQ-n6v896 minted for the inversion property and the target re-annotated. The half I dropped — annotating the test — was caught only by the merged state and repaired in fix3.

**finding-8**: The probability scale is defined once and applied two ways: HAZ-werm85 is P2 on a silent narrowing of "deliberate act" to "the victim's deliberate act", while two other entries count any act at all.
disposition: fixed (fix2b). The scale qualified to "a deliberate act by a party whose cooperation the situation requires", and all twelve estimates re-checked against it.

**finding-9**: The two round-trip tests carry REQ-wnjz9j, which has no accepting side, so a decoder returning `Err` for unknown signatures would leave them green.
disposition: fixed (fix2a). Dropped from that pair; the requirement has three dedicated tests already.

**finding-10**: "The crate has twenty-two integration targets. Eleven run at the gate … Nine run nowhere" — 11 + 9 = 20. `common/` and `fuzz_support/` are shared modules, not cargo targets; the figure came from a directory listing.
disposition: fixed (fix2b), written with the post-fix numbers: twenty-one targets, twelve gated, nine ungated.

**finding-11**: The plan says "seven new test targets"; there are eight. And the register argues HAZ-v2cmtx is "P1 rather than P0", where P0 is not on this project's scale.
disposition: fixed (fix2b). Both corrected; the P0 sentence re-argued against the real scale (P1 improbable, P2 occasional, P3 frequent).

**finding-12**: REQ-9vwcwc bundles a delivery filter and a reporting obligation, verified by different tests in different files, against the ledger's single-behaviour form.
disposition: fixed (fix2a, fix2b). Split: REQ-9vwcwc kept delivery, REQ-5upq6n minted for reporting, each verified in the file that actually gates it.

### Round 2 — fourteen findings

**finding-13**: REQ-hd6m9d states a property the software does not have. It requires decoding "only through the decoder compiled for the Runtime spec version the chain reported", unqualified — but `from_client` resolves once at construction and `get_org_state` decodes through the stored decoder thereafter, which is open report PR-w5sk5k. The ledger asserts the opposite. Its only test exercises `dispatch::for_runtime`, never the client, so nothing gated can detect the gap.
disposition: fixed (fix6a). Reworded to the resolution decision the code makes, leaving the per-decode obligation with PR-w5sk5k. fix6a found RC-d7r82e carried the identical overclaim and narrowed it too, adding an explicit "what this control does not claim" paragraph. Class C does not permit a requirement asserting what an open defect says is absent.

**finding-14**: The pinned `SPEC_VERSION` — the constant RC-d7r82e turns on — is gated by nothing. Measured: every case in `runtime_version_dispatch.rs` derives its expectation from the constant itself, so changing `2_002_002` to `3_003_003` leaves the whole gate green while the crate is pinned to a runtime that does not exist. The register asserted four claims held by review; this is a fifth.
disposition: fixed (fix6a). Recorded as the fifth entry with a residual sentence under HAZ-8s5chy. No code change: the failure is fail-closed, so it lands on the total-outage pathway already assessed S3/P2.

**finding-15**: The sweep of hazards introduced by the controls is incomplete and self-contradictory — it names six controls as introducing nothing while an entry above assesses two of them, and **RC-kemv75 appears in neither list**, the one control of nine never checked for what it breaks, in the section whose whole purpose is that check.
disposition: fixed (fix6a). Sweep completed: five of nine with an entry, four introducing nothing, each with a reason. RC-kemv75's entry records two introduced situations — the first backfill span as long as the finality lag, and a deduplicated head reported once and never again — both S3/P2, both further triggers rather than new hazards, so the total stays at ten.

**finding-16**: HAZ-8s5chy's P2 does not reproduce from the scale as round 1 amended it: its argument still reads "no deliberate act by any attacker", the vocabulary the amended Method section explicitly disowns, while the amended rule makes a runtime upgrade an act by whoever ships it. The file claims every estimate re-derives under the new reading; for this one it does not.
disposition: fixed (fix6a), by amending the scale rather than the estimate. "A party whose cooperation the situation requires" now means the one who has to *decide to change something*, excluding an adversary and a system merely continuing to run — so a release is P2 and a decision to change a convention carried in one is P1. Moving HAZ-8s5chy to P1 instead would have called a total outage on every routine Paseo upgrade "improbable", using P1 as a wastebasket the file itself warns against. All twelve estimates re-derived; the register's own false claim that round 1 had already re-checked them corrected.

**finding-17**: REQ-2qa5r5 has no abnormal-input case and no recorded exemption — the only one of twenty-one in that position. The cause is visible: it was minted *after* the gate run whose check-6 clearance the plan records, and that clearance was never re-run over the five new items.
disposition: fixed (fix6a). A sentence, not a test: it constrains a declaration and has no input domain. The same recorded for REQ-n6v896, whose generator produces only structurally valid shapes by construction. Both exemptions were re-judged sound by rounds 3 and 4 and by two gate runs.

**finding-18**: Three test-header cross-references point at the wrong artefact, two because round 1's split was not propagated: a layout-drift residual attributed to the wrong hazard, and two pointers sending a reader to `contract_address_filter.rs` for REQ-9vwcwc, which that file's own header disclaims.
disposition: fixed (fix6b).

**finding-19**: Four source citations stale or off by a line, all load-bearing — including `types.rs:11-27`, which a fix round had itself invalidated by adding seven lines of doc-comment to that file in the same round that wrote the citation.
disposition: fixed (fix6a for the register's three, fix6b for the test file). fix6a then verified **all 115 citations in the register** rather than the three it was asked about, and found two more wrong: the too-many-topics mutation panics cited at two lines that cannot panic. Where it could not reproduce the second site's message it gave a line range rather than inventing one.

**finding-20**: The plan records `tracked paths 383`; it is now 384. And the coverage acceptance table still carries figures superseded four hundred lines later, with no marker at the table and the parenthesised counts corrected nowhere.
disposition: fixed (fix6c). The superseded row kept and marked, a current row added beside it — history readable, no stale figure mistakable for a current one.

**finding-21**: The setup checklist calls the not-minted list "sixteen"; a fix round made it seventeen. And its panic-freedom item says "eight new integration targets"; there are nine.
disposition: fixed (fix6c).

**finding-22**: The Makefile contradicts itself on the figure its recalibration turns on — `client.rs` at 16.37% in one note and 12.75% in another (a pre-relocation figure) — and both give a denominator of 501 where it is now 491, uncaveated, in the file the recalibration names as holding its full reasoning. Plus a broken sentence fragment in the unit config.
disposition: fixed (fix6c), after re-measuring: 16.37%, 205/491 lines, 284/669 regions.

**finding-23**: REQ-nygs7k carries two behaviours — a delivery rule and an ordering rule — the defect round 1 split REQ-9vwcwc for. Its ordering clause's test is annotated to the other requirement, and a test carrying REQ-nygs7k exercises a no-filter case the requirement does not cover.
disposition: fixed (fix6a, fix7). The ordering clause folded into REQ-9vwcwc rather than split out, since that requirement already owned the contract check and its test already verified the ordering — no new ID needed. One annotation re-pointed. Round 3 then measured the ordering clause unfalsifiable and it was dropped entirely.

**finding-24**: REQ-axcxf7's second clause — "ends inside a field's own length prefix" — is exercised by neither test annotated to it. Both cuts land outside any prefix; the literal case is a cut at byte 21, inside `data`'s two-byte compact prefix, and it is untested.
disposition: fixed (fix6b). The byte arithmetic confirmed independently before the test was written — 64 bytes encodes as a two-byte compact prefix at offsets 20-21, since single-byte mode stops at 63 — and the new case asserts the prefix bytes and mode bits rather than assuming them, so an encoding change reds it in place. **It passed first time, recorded as such.**

**finding-25**: REQ-2qa5r5's wording contradicts the unit glossary (the Organisation admin is the key the slot is keyed on, not a value read from it) and uses "Organisation public key", which neither glossary defines — both call it the signing key.
disposition: fixed (fix6a).

**finding-26**: The two round-trip tests carry REQ-88fp2h but cannot red on its stated property — measured: one-sided weakening of all four shape guards reds the six dedicated refusals and leaves both round-trips green.
disposition: recorded, not re-annotated (fix6c). A nit, because those tests do execute the guards and serve as the requirement's normal-input case; the requirement's red comes from its six dedicated refusals. Recorded so the record does not read nine annotations as nine independent reds. **fix6c could not reproduce the mutation — the sandbox blocked that target — and attributed the figure to the reviewer rather than claiming it, while verifying by reading that all four guard sites are where cited.**

### Round 3 — eight findings, four above the stated merge bar

**finding-27**: Two minted controls carry a leading clause no requirement realises, while the register asserts each control "is realised by requirements carrying `(implements: RC-…)`". RC-kemv75 opens with the best-block/finalised distinction — none of its three requirements states it, and the tell is that *Finalised observation* is listed among the terms the requirements use and appears in no requirement. RC-d7r82e ends "the refusal is made at construction, before any read is possible" — stated by no requirement and reached by no gated test. `check-trace.sh` passes because each control has *some* implementing requirement; a clause-level check would not.
disposition: fixed (fix8a) by narrowing both controls and recording the removed clauses as not-minted controls 18 and 19. The alternative — minting requirements — was checked and is closed: both clauses live in async code needing a live chain, so a new requirement would draw `MISSING-TEST` on the next run, and a requirement with no possible gated test is the defect this change removes, not a fix for one. fix8a also judged both clauses into the review-held collection, since the register still leans on each.

**finding-28**: A sixth claim held by review, where the list asserts completeness at five: the `OrgRegistry` storage layout and event shapes that RC-5ejucb and RC-6gfh8d hard-code were established by reading a disclaimed file, and no gate re-establishes them — the register says so itself and its own not-minted control 9 names the layout, but the list collects only the `ZeroValue()` revert.
disposition: fixed (fix8a). Added and "five" re-derived at every site.

**finding-29**: A clause three documents say is gated, measured unobservable. REQ-9vwcwc required the contract comparison be made "before, and independently of, any other filter". The reviewer rewrote `log_is_ours` to evaluate the admin filter first and return the contract comparison last — the exact inversion — and `log_ownership` reported 7 passed, 0 failed. Ordering inside a total boolean predicate is unobservable; what the cited test gates is the consequence, which does red.
disposition: fixed (fix8a, fix8b), wording only. The ordering phrase dropped, the observable consequence kept, all three sites corrected. **fix8a found a fourth site the review had not: RC-5e3bdk carried the same clause**, and dropping it from the requirement alone would have orphaned it in the control — creating finding-27's defect while fixing this one.

**finding-30**: Two requirements state properties the software does not have. REQ-ntn4ss says a best head "at or below" the last processed is reported as a reorg — a re-notification of the same head takes the dedup arm and yields nothing, so it contradicts REQ-gr2ver directly below it. REQ-5zux82 says an observation is reported "for every height" in the span; a height carrying no event produces no notification.
disposition: fixed (fix8a). Both reworded against the code and re-checked against every annotated test.

**finding-31**: Two register figures stale by one, both pre-dating a test added in a later round: `decode_revive_event.rs` called "(fifteen cases)" where it holds 16, and the `SPEC_VERSION` mutation recorded as "71 passed" where it is 72.
disposition: fixed (fix8a), both re-measured.

**finding-32**: A count that does not add up in the paragraph introducing the problem ledger: four hazards filed, one fixed, "the other four stay open".
disposition: fixed (fix8a) — the register was counting hazards in one sentence and reports in the next.

**finding-33**: Citation drift, again on the same item: the ABI-drift test cited at a line that is mid-comment.
disposition: fixed (fix8a).

**finding-34**: REQ-5upq6n says the software "shall report" the emitting contract with every decoded event. The decoder does; the address is dropped before a subscriber sees it, since `SubscribedEvent` carries no contract field.
disposition: fixed (fix8a) — the surface named.

### Round 4 — seven findings, two above the bar

**finding-35**: REQ-5zux82 and REQ-ntn4ss both turn on the word "extend", which neither glossary defines and which they use in two incompatible senses — so under whichever single sense you adopt, one of the two states a property the software does not have. Measured: narrowing `from` in `scan_step` to the sense REQ-ntn4ss's first clause forces reds `a_jump_backfills_every_skipped_height`, 13 passed 1 failed. This is the residue of finding-30: that round fixed both quantifiers and left the term.
disposition: fixed (fix9a), the word eliminated from both. **fix9a corrected my disposition rather than following it**: I passed on the reviewer's phrase, which leaves the `last == None` branch unstated, and two gated tests exercise that branch. It checked `scan_step`'s actual `from` computation, found the `_` arm covers both cases, named both, and re-read all fourteen tests against the reworded pair — every one matches.

**finding-36**: A ninth claim held by review, where the section asserts eight. Under HAZ-werm85 the register says the wiring between `log_is_ours` and its call site — that the address compared is the one `from_client` was given, that `continue` drops the log rather than surfacing it — is "established by reading", with not-minted control 13 named as what would close it. RC-5e3bdk's entire "reduced" residual depends on it.
disposition: fixed (fix9a). Added as the ninth, placed so every existing ordinal cross-reference stays correct; "eight" re-derived across 16 lines and 21 numerals, more sites than the review had counted. fix9a declined a tenth — RC-d7r82e's client→decoder binding — because the list excludes defects and that gap is carried as PR-w5sk5k, and recorded the judgement so a future round disagrees knowingly.

**finding-37**: `type_widths.rs` names a target as gating a refusal it does not contain: the over-wide-epoch refusal is gated in `decode_org_state.rs` only, and the event path's epoch has no gated case of its own, though REQ-9wwenn is worded over both.
disposition: fixed (fix9b). **The justification was verified before it was written**: switching the shared guard off reds `decode_org_state` at 8 passed, 1 failed, on the named case — so it is a citation error, not a coverage hole. fix9b added the distinction that an event-path case would gate that the *refusal* is reachable there, not merely that the narrowing happens.

**finding-38**: `fuzz_parse_revive_event`'s header sends a reader to the file that disclaims the thing — verbatim the defect round 2's finding-18 fixed in a sibling file; this copy was not swept.
disposition: fixed (fix9b).

**finding-39**: The setup checklist says the controls introduce "three entries"; the register says four, the fourth added by finding-15.
disposition: fixed (fix9b).

**finding-40**: Two minted controls still carry a sub-clause no requirement realises — RC-d7r82e's "the decoder a client reads through", which lives in async code, and RC-675a3h's "detected by an independently recomputed comparison", which is a property of the evidence, not of the software, and which no requirement could state.
disposition: fixed (fix9a) by **softening the claim rather than narrowing further**. A control clause realised by a gated test rather than by a requirement is legitimate; the register now says so, names the two exceptions and where each is recorded, and keeps both clauses verbatim — they are true and they matter.

**finding-41**: Two nits: REQ-5zux82's "for each event it finds" reads against REQ-nygs7k's withholding, and two counts of the introduced entries use different units — per control in one place, per entry in another, neither saying which.
disposition: fixed (fix9a). The specific-over-general resolution stated; both counts now say what they count.

## Gaps

What this change did **not** establish. A gap stated here is a gap; a gap left out is a claim.

**The two coverage shortfalls the owner accepted, 2026-09-10.** `client.rs` sits at **16.37%** statement coverage — 281 of its 336 lines unexercised — and it is two thirds of the crate. It is the async subxt code the nine chopsticks/anvil targets exercise, and no gate runs those. And **decision coverage is unmeasured in every unit of this repository**: llvm-cov's Branches column reads `0 / 0 / -` throughout, so the class C target's second half is not measured anywhere. Both are shortfalls against a mandatory class C requirement, not targets met, and both are pre-existing rather than caused by this change. `org-node` and `app` have no `coverage_command` at all, so two of the three impacted units have no structural coverage measurement whatsoever.

**The coverage floors were recalibrated, and the basis changed.** 52/58 became 41/42 on the owner's explicit decision. This is a re-baselining, not a lowering: the relocation moved 214 executed `#[cfg(test)]` lines out of the measurement, and llvm-cov had been counting them as 100%-covered *library* lines, inflating both sides of the old ratio. Netting them out, real covered library lines rose from ~169 to 215 across this change. The reasoning is recorded in the Makefile beside the floors, not only here. **The region margin is 0.45 points** — the tightest number in this change, below the one-point slack the Makefile's own rule prefers, and CI enforces it on a different platform than the floors were calibrated on. Raising it is a ratchet step for a settled basis and is deliberately not taken here.

**Nine claims are held by review rather than by a gate**, listed in the register's own collection: the contract's `ZeroValue()` guard and its storage layout and event shapes; the `h160_of` ground truth against pallet-revive; the two event-signature literals against the deployed contract; the width test's ABI derivation; the pinned `spec_version`'s value; the construction-time refusal; the best-block/finalised distinction; and the wiring between the ownership decision and its call site. Each is true today, established in this change by reading, and re-established by no gate. Two of them rest on `on-chain/src/OrgRegistry.sol`, in a directory `.guardrails/units.yaml` disclaims, whose forge tests run in no CI lane in this repository.

**Twelve test targets run nowhere.** Nine chopsticks/anvil targets in this unit and three in `org-node` need a live chain fork and `on-chain/scripts/node_modules`. They are excluded from every `verify_commands` and from the coverage measurement by design, and that exclusion is the cause of the `client.rs` figure above. Consequently **finality and reorg delivery have no gated evidence**: chopsticks finalises every `dev_newBlock` immediately, so no chopsticks test can be evidence about real finality, and the only test that could be — the `#[ignore]`d live-Paseo `smoldot_smoke` — runs nowhere.

**Four problem reports stay open**, each with a dated checklist item and an age limit of 2026-10-11: PR-w5sk5k (the decoder pinned at construction and never re-checked), PR-h4mb8y (`src/verify.rs` is seven lines of doc-comment and no code, so the module the design names as closing the loop contains nothing), PR-uq5r97 (the best lane's first backfill span is unbounded), PR-qpp28h (a reorg deeper than the notification gap is undetectable, by the code's own admission). PR-p5ngya is resolved by this change.

**Nineteen controls are identified but not minted.** Eighteen carry a checklist item with an owner and a decide-by date; control 17 is explicitly exempted because it reduces no residual risk, which was confirmed by measurement. **`fuzz_event_round_trip` has an empty seed corpus** — visible in the run line, which carries no `corpus inputs:` field where the other two report 4 and 6 — and has been visible in measured output since 2026-08-26 without being written down as a gap until now.

**The expectation org-node holds against this unit, REQ-ysyu9g, is deliberately not met by this change.** The fact it asks for is confirmed — subxt 0.50.1 documents `at_current_block` as the current finalized block, and `get_org_state(_, None)` calls it — but meeting it would flip org-node's item from exempt to ordinary, requiring a verifying test there whose only honest form is a chopsticks target no gate runs. The obligation is carried as a not-minted control with the providers' deadline of 2026-12-05.

**Clippy runs `--lib` only**, so the nine new integration targets sit outside the panic-freedom denial the crate root declares, and the crate is not `rustfmt`-clean at baseline with no `cargo fmt` step in the workflow. Both are recorded as decisions rather than fixed here. **`app`'s `npm run check` emits one pre-existing warning** — `@types/node` is named in the generated tsconfig and absent from `package.json` — and `app/node_modules` is gitignored, so it must be installed on the merging machine for that command to run at all.

**The date on this record is 2026-09-11; the ledger files carry 2026-09-10 names.** The change was finalized on the 10th and merged on the 11th. The names are kept, as the same judgement was made for the `org-node` half of this tooth.

> **Correction, 2026-09-17 — the paragraph immediately above was untrue when
> this record was written, and the original text is left standing because this
> project dates its corrections rather than rewriting the record.**
>
> What was untrue: the ledger files did **not** carry 2026-09-10 names. They
> carried no dated names at all. `merge-change` step 3 (`finalize-docs.sh`) was
> never run for this change, so `fbf17f0` squashed all three onto `master` still
> wearing their draft names —
> `on-chain-client/docs/requirements/DRAFT-worktree-guardrails-on-chain-client-risk-chain-reading.md`,
> `on-chain-client/docs/risk/DRAFT-worktree-guardrails-on-chain-client-risk-on-chain-client-hazards.md`
> and
> `on-chain-client/docs/problems/DRAFT-worktree-guardrails-on-chain-client-risk-on-chain-client-problems.md`.
> They stayed that way on `master` from 2026-09-11 until the repair named below
> merged. No terminal date is given: this note first carried 2026-09-17 and the
> repair did not merge that day, which is the same forward-dating the final
> paragraph of this note describes.
>
> What is true now: the three files are
> `on-chain-client/docs/requirements/2026-09-10-chain-reading.md`,
> `on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md` and
> `on-chain-client/docs/problems/2026-09-10-on-chain-client-problems.md`. The
> 2026-09-10 names were the owner's decision of 2026-09-17, chosen so that the
> paragraph above becomes true and so that this unit matches what both sibling
> units of the same tooth did — `org-node` merged 2026-09-10 with 2026-09-09
> names, `app` merged 2026-09-15 with 2026-09-14 names.
>
> Two steps were skipped, not one, and this record is the evidence of both. The
> gate table above has a row for `check-ids.sh --allow-draft-files` across three
> units and **no row for `finalize-docs.sh`** and **no row for a bare
> `check-ids.sh`** — that is `merge-change` step 3 and step 4, neither run. The
> two omissions are one omission: step 4 is the only local run without
> `--allow-draft-files`, so it is the only local gate that would have convicted
> the skipped step 3. A draft-named ledger file is `DRAFT-FILE` under
> `check-ids.sh`, so this unit's id gate was exit 1 on `master` for that whole
> span — at least six days.
>
> CI would have convicted it, had anything been pushed. `.github/workflows/rust.yml`
> runs `check-ids.sh` bare per unit whenever `github.event_name != 'pull_request'`.
> But `origin/master` is still at `2bb1c21` of 2026-06-17 and local `master` is 45
> commits ahead, so no workflow has run on any of them. Stronger still, and
> stated so this record does not imply a gate with a history: `rust.yml` was
> itself added by the unpushed `4bb5509` (2026-08-27) and is absent from
> `origin/master` entirely, so it has never executed on any commit. "CI would
> have convicted it" describes a workflow that has never once run, not one that
> lapsed. The full account, per workflow and per quint invariant, is in
> `docs/plans/2026-09-05-ratchet-setup.md` under "Added 2026-09-17".
>
> The repair is the change `worktree-guardrails-on-chain-client-finalize`, whose
> verification record is written at `merge-change` step 6b — after the
> independent review — and lands in `docs/verification/` declaring
> `branch: worktree-guardrails-on-chain-client-finalize`. That record and this
> note land in the same signed squash, so the two are consistent in the merged
> commit.
>
> That record is named here by its branch and not by its path, deliberately.
> A verification record is named for the date it merges, which is not knowable
> while it is being written — this note first named a 2026-09-17 path and the
> merge did not happen that day, which would have left a certified record
> pointing at a file that never existed. The branch is the durable handle:
> `check-review.sh` selects a record by its `branch:` field, matched whole, and
> never by its filename. Guessing a merge date ahead of the merge is the precise
> mistake this whole repair exists to correct, and it was about to be repeated
> one paragraph away from the account of it. The corrected account, with the full
> reasoning for the dates, is in
> `docs/plans/2026-09-10-on-chain-client-risk-analysis.md` under
> "Filenames, corrected on 2026-09-17". The systemic open items are filed in
> `docs/plans/2026-09-05-ratchet-setup.md` under "Added 2026-09-17".

**This unit still has no architecture or SOUP.** `on-chain-client/docs/architecture/` holds a README and an empty `soup.md` template, so the `parity-scale-codec` and `subxt` dependencies this analysis leans on are recorded in the register's prose and nowhere structural. That is tooth 4's work, and this change does not begin it.
