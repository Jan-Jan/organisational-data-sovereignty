# Verification — app risk analysis (2026-09-15)

branch: worktree-guardrails-app-risk
reviewer: three independent subagent reviewers, dispatched blind at `merge-change` step 6a — round 1 on the 90-test tree, round 2 on the 102-test tree, round 3 on the 104-test tree. Each was given the repository, the ledger and the plan, and no account of how the work went; rounds 2 and 3 were additionally given no knowledge of their predecessors' findings, so a mis-applied fix had to be found as a defect rather than recognised as an answer. Each ran every `verify_commands` entry itself and reported the counts it saw.
verdict: round 1 — do not merge, four of eight findings blocking. Round 2 — do not merge, six of thirteen blocking. Round 3 — do not merge, two of seven findings are false claims rather than deferred work. All 28 findings accepted, none disputed. After the fixes, nine verification-gate rounds and a final confirming round reported every check passing on counts.
reproduced: yes, for every defect this change fixes, and the reproductions are the substance of the red → green table below rather than a summary of it. Six were reproduced by writing the test first and watching it fail before the code existed; five by mutation — restoring the defect in source, watching the named test fail for that reason, reverting, and verifying the revert byte-identical by SHA-256 where the file was production code. Two reproductions came back **green** and are recorded as measured negatives, not quietly dropped: removing the `StartGuard` move from the spawned task reddens nothing, and `a_non_terminal_failure_does_not_stop_the_loop` passed against the broken matcher vacuously.

Units touched: **app** (impact set: `app  touched`, one unit). Branched from `master` at `fbf17f0`.
Change: the `app` unit's first ISO 14971 hazard register, its first requirements, and its first tests — a unit that had zero.
Plan: `docs/plans/2026-09-14-app-risk-analysis.md`.

## The gate

Every figure below is from the final gate run on the tree under test, not carried forward from an earlier round.

| Gate | Result |
| --- | --- |
| `cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support` over six named targets | **78 passed, 0 failed, 0 ignored**, no warnings — `startup_policy` 12, `connection_status` 10, `org_id_parsing` 14, `receiver_events` 27, `receiver_guard` 6, `ipc` 9 |
| `npm --prefix app run check` | **194 files, 0 errors, 1 warning** — the pre-existing `@types/node` warning the unit config documents |
| `npm --prefix app run test` | **4 files, 30 passed, 0 failed** |
| `check-ids.sh` (no `--allow-draft-files`) | clean — no draft token, no draft-named file, no duplicate, no malformed ID |
| `check-trace.sh` | clean — `REQ 22, HAZ 6, RC 12, SDD 0, LLR 0, PR 6`; 5 open problems and 1 open expectation, all inside their limits |
| `check-units.sh` (repository level) | clean — `units: 4, disclaimed 7; tracked paths 405` |
| Coverage, against the class target | **SHORTFALL, accepted.** No `coverage_command` is configured. Class C requires statement **and** decision coverage; neither is measured for this unit. Accepted by the owner on 2026-09-14 as a deferral to tooth 5, so that `org-node` and `app` are floored on one measurement basis rather than this unit in isolation. Recorded in `app/.guardrails/config.yaml` and `docs/plans/2026-09-05-ratchet-setup.md`. Nothing in the register rests on a coverage figure, because there is none. Decision coverage is unmeasured in all four units. |
| Working tree | clean; no worktree registered inside the change worktree |

**Suite total: 108 tests.** The unit had none.

## Red → green

One row per ID this change implements. The subagent that ran the loop is the only party that saw the test fail, so these rows are the only durable record of that observation.

**Read the "Watched red" column with its qualifier.** T1's and T3's reds are **per target and per file**, not per test: every module was new, so the target failed to compile or the file collected zero tests, and every test in it went green on the first run afterwards. That proves the tests could not have passed before the code existed — the iron law's substance — and does not prove each individual assertion discriminates. What supplies that is the mutation column, and where a row says "by mutation" the named test was watched failing for the named reason with the defect restored in source.

| Item | Test | Watched red |
| --- | --- | --- |
| REQ-7g3k9a | `startup_policy.rs` ×5 | compile red: `E0432 could not find 'policy' in 'ods_poc_lib'` before `policy.rs` existed |
| REQ-bmk2z2 | `passphrase_refusal_names_both_variables_supplier_first`, `data_dir_refusal_names_both_variables_supplier_first` | same target red |
| REQ-rxc8sp | `startup_policy.rs` ×5 | same target red |
| REQ-e4ah9h | `configured_chain_reports_its_endpoint_and_contract`, `unconfigured_chain_reports_neither` | compile red: `ChainEndpoint`, `TransportModeName`, `connection_status_from_state` not found, 15 errors |
| REQ-bvx4nh | `endpoint_comes_from_the_built_configuration`, `absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env`, `data_dir_comes_from_the_built_configuration`, `abnormal_data_dir_paths_are_reported_verbatim` | same target red; the last two also by mutation — a constant data directory failed both with `left: "/tmp/ods-poc" right: "/var/lib/ods"`, `policy.rs` reverted byte-identical (SHA-256 `5136e084…` both sides) |
| REQ-645jq9 | `connection_status.rs` ×4, `ipc.rs` ×2 | same target red; and by mutation — passing `TransportModeName::Networked` literally failed `connection_status_reports_transport_mode_over_ipc` against a Loopback state |
| REQ-sjkp8z | `org_id_parsing.rs` ×14, `ipc.rs` ×2 | compile red: `E0432 could not find 'parsing'`; the doubled-prefix case genuinely red — `exactly one 0x prefix is permitted, not two: OrgId(0xefef…efef)`, the malformed input having been accepted |
| REQ-2k7ys4 | `receiver_events.rs` ×4 | compile red: `E0432 could not find 'events'` |
| REQ-dp95pv | `record_unreadable_names_the_organisation`, `record_unreadable_emits_exactly_one_event` | same target red |
| REQ-tw4cb5 | `receiver_events.rs` ×5 | same target red; and three added later, genuinely red against `epoch: 0` restored to the self-delete emission — `` `revoked` payload carries an epoch key: ["epoch", "org_id"] `` |
| REQ-affyf5 | `receiver_events.rs` ×4 | same target red |
| REQ-jfxah3 | `receiver_events.rs` ×5 | same target red; and four added later, genuinely red against the dead matcher — the real bind-failure message produced `[ReceiveError { … }]` and **no `Stopped` outcome at all** |
| REQ-3hfggn | `claim_succeeds_again_after_the_guard_drops`, `guard_releases_when_dropped_by_panic`, `release_then_concurrent_claims_yield_exactly_one_guard` | compile red: `E0432 no 'StartGuard' in 'events'` |
| REQ-6hgm8r | `first_claim_succeeds`, `second_claim_while_held_fails`, `concurrent_claims_yield_exactly_one_guard` | same target red |
| REQ-vgr7s2 | `revoke.validate.test.ts` ×4, `ipc.rs` ×1 | file red: `Cannot find module '../src/lib/revoke'`, 0 tests collected; and by mutation — an unconditional non-empty check failed `revoke_member_is_not_refused_for_an_empty_peer_addr` with left `"Peer addr blob (hex) is required."`, `commands.rs` reverted byte-identical (SHA-256 `20b00ae7…`) |
| REQ-he8ejb | `revoke.validate.test.ts` ×6 | same file red |
| REQ-a83vqr | `verify.row.test.ts` ×3 | file red: `Cannot find module '../src/lib/verify'`; and by mutation — the failure branch returning `verified: true` failed `renders a failed event as not verified` |
| REQ-akt4p7 | `verify.row.test.ts` ×5 | same file red |
| REQ-rq8g2v | `receiver.subscribe.test.ts` ×6 | file red: `Cannot find module '../src/lib/receiver'`; and by mutation — returning the cleanup before awaiting failed the late-registration test with `expected [ 'fast' ] to deeply equal [ 'fast', 'slow' ]` |
| REQ-kn5rtx | `receiver_events.rs` ×7 | compile red: `cannot find function 'classify_receive_error'`, `no variant named 'ReceiveError'`, 8 errors; and genuinely red for the reclassification — `OrgNotOnChain is reachable from a local condition and must not be a verdict` |
| REQ-wu6z9p | `receiver.log.test.ts` ×6 | file red: `TypeError: applyReceiverEvent is not a function`, 6 failed / 24 passed |
| PR-u34uqm | `unsubscribes exactly the listeners it registered, including those whose listen call resolved late` | by mutation, against the restored `$effect` shape: `expected [ 'fast' ] to deeply equal [ 'fast', 'slow' ]` |
| HAZ ×6, RC ×12 | — | carried transitively by the REQs that implement them; `check-trace.sh` reports no unmitigated hazard and no unimplemented control |

**Two measured negatives, recorded because a mutation that reddens nothing is evidence too.**

- Removing `let _guard = guard;` from the spawned receiver task — the line that makes REQ-3hfggn true in production — **reddens nothing.** All six targets stayed green; the only signal was `warning: unused variable: guard`. `StartGuard`'s release-on-drop is gated three ways at the unit level, panic included; its placement at the only call site is gated by nothing, because nothing in this repository can run the receiver loop (it needs a bound iroh endpoint, a real organisation, a chain and a peer). Recorded as claim 8 of the register's §3.
- `a_non_terminal_failure_does_not_stop_the_loop` **passed at red, vacuously**: with a matcher that never fired, nothing was terminal, so "does not stop" held for free. It constrains the new markers against over-matching and only means anything now that its sibling is green.

## What was wrong, and what was built

The `app` unit is a desktop shell in two languages that computes almost nothing: the handlers lock a mutex, call one `OrgService` method, and map an error to a string. What it decides is narrow and all at the edges — what to start with, what to show, what to pass on, when to stop. Every hazard found is one of those four, and none is an algorithm being wrong.

**Five defects the analysis found in the product:**

1. **Revocation could not be submitted through the only user interface.** `Revoke.svelte` refused an empty peer-address blob unconditionally, while `revoke_member`'s own doc-comment documented the field as optional and needed only for same-machine dialing. In Networked transport — the default, and the mode the code calls "the right choice for two laptops over the internet on live Paseo" — a join request carries no address, so the operator had nothing to paste. The Admit panel even displayed this to them, in amber, as "No (no p2p addr)". A safety function that exists in the backend and cannot be invoked from the only interface is not implemented (HAZ-n97v5g, S3/P3).
2. **The verification display could not report a failure.** `Membership.svelte` calls its ✓/✗ column "THE KEY PoC OUTPUT" and assigns `verified` the literal `true` at both of its only two assignment sites. `✗ MISMATCH` existed in the source and was unreachable by any input. Failures went instead to an untyped string list carrying no organisation, epoch or root, so they were not merely unmarked but unlocatable (HAZ-9fmhm4, S3/P3).
3. **Placeholder values were emitted as measured ones.** `.unwrap_or((0, String::new()))` after a successful verify, and a hard-coded `epoch: 0` on self-delete. Epoch 0 is genesis — a real, reachable value — so a consumer could not distinguish "this organisation is at genesis" from "this value is not available", and the second was rendered in the affirmative (HAZ-5ha5vv, S3/P2).
4. **Key material sat under a passphrase published in the source.** `ODS_PASSPHRASE` unset yielded the fifteen-character literal `"ods-dev-default"` — and that was the normal path, because nothing in the UI sets a passphrase, and `init` did not even log the substitution (HAZ-8ghmhn, S3/P3).
5. **A dead receiver was indistinguishable from a live one and could not be restarted.** The start guard was claimed before spawn and never released, the loop's exit was silent, and termination was decided by a substring match on another unit's formatted message (HAZ-cfp4jb, S3/P2).

**And one the change itself introduced**, found by the first independent review: routing every `Err` into the new verification-failure event made a transport fault render as `✗ MISMATCH` under "Verified Updates (chain root match)", and left `receiver-error` with no emitter at all. HAZ-9fmhm4 is *the verification display misrepresenting the verification state*; before, it could not report a failure, and after the first implementation of its controls it reported failures that were not verification verdicts. §5 of the register lists five hazards the controls introduce, and this one was found in the implementation after §5 had been written claiming four — because §5 had assessed the controls as designed rather than as built. Fixed under RC-3rddh7 by classifying on the error's type, which `OrgNodeError` had always supported and the code discarded with `e.to_string()`.

**What was built:** 6 hazards, 12 controls, 22 requirements, 6 problem reports (one resolved), a unit glossary, and 108 tests across two gated suites and a frontend test runner that did not exist. The unit's `strict_paths` widened to bring the SvelteKit frontend under trace discipline; two of its three `verify_commands` now run in CI.

**Overall residual risk for the unit: UNACCEPTABLE.** All six hazards are S3, and this project's matrix makes S3 unacceptable at every probability including improbable. Two of the six do not improve in probability at all, because their largest remaining contributor is state held by `org-node` and unreachable from here — that is REQ-x3c8n2, due 2026-12-13.

## Review

Twenty-eight findings across three blind rounds. Every one was accepted; none was disputed. The nine gate rounds' documentation findings are not reproduced here — they are recorded round by round in the plan — except where a review finding names one.

**finding-1**: (round 1) REQ-affyf5 is implemented only in the type, never in a reachable path: the sole construction site hard-codes `org_id: None`, no `OrgNodeError` variant carries an organisation, and the gating test builds a payload production code cannot construct, so it could not fail if the behaviour broke. The register nonetheless claimed "failures gain an identifier", which is true of no run.
disposition: the claimed reduction is **withdrawn** from HAZ-9fmhm4's residual rather than re-argued. The test is kept for its shape and recorded as claim 11 of §3. The "failures are unlocatable" half of that hazard is stated as not reduced by this change at all, and REQ-x3c8n2's scope was widened to cover the organisation identifier as well as the error classification.

**finding-2**: (round 1) The change silently deleted the `receiver-error` event and routed every receive-path error — including ones the previous code called transient — into `verification-failed`, which renders as ✗ MISMATCH under "Verified Updates (chain root match)"; `receiver-error` was left with no emitter, making the "Receiver Errors (non-fatal)" block dead markup and its API doc-comment false.
disposition: RC-3rddh7 minted, with REQ-kn5rtx (classify by the error's type) and REQ-wu6z9p (the consumer half, separate because it fails separately). `classify_receive_error` matches on the variant with **no wildcard arm**, so a variant added upstream is a compile error rather than a silent misfiling. Reddens without it: `a_chain_failure_is_classified_as_a_receiver_error`, and `produces no verification row for a receiver error`. §5 gained its fifth entry, describing the control and the defect as the same edit.

**finding-3**: (round 1) `revoke_member_is_not_refused_for_an_empty_peer_addr` asserted `!err.to_lowercase().contains("peer_addr")`, but the defect it guards against produced `Peer addr blob (hex) is required.` — which lowercased contains no such substring, so reintroducing the exact regression left the test green.
disposition: the assertion now derives its expected text from `OrgNodeError::OrgNotOnChain.to_string()` rather than from the absence of a spelling. Confirmed empirically before the fix: `printf 'Peer addr blob (hex) is required.' | tr A-Z a-z | grep -c peer_addr` → 0. Reddens without it: the same test, against the original wording restored.

**finding-4**: (round 1) REQ-rq8g2v's operative clause — listeners whose registration completed after teardown began — is gated by nothing: `subscribeAll` awaits every `listen` before returning a cleanup, so the clause is unreachable at the helper's own interface, and the test named for it awaits `subscribeAll` before calling cleanup.
disposition: recorded as claim 10 of §3 rather than papered over. What implements the clause is a `torndown` flag inside two components, and no test renders a component — not-minted control 1.

**finding-5**: (round 1) Two `ipc.rs` tests carried `verifies: REQ-he8ejb` while asserting the handler's check, which by definition runs after the command is invoked; REQ-he8ejb requires rejection *before any command is invoked*, so deleting the frontend check left both green.
disposition: both annotations retracted, with the reason written into the file; the tests are kept as handler-side defence in depth carrying no requirement of their own. REQ-he8ejb is now frontend-only, which the unit config records.

**finding-6**: (round 1) The connection status stopped being re-derived from the environment for the **data directory** as well as the endpoint and contract, and no requirement covered it — unmarked derived work.
disposition: REQ-bvx4nh widened to name the data directory, rather than minting a derived item for a third field of one struct. Round 2 then found the new clause gated by no test annotated to it (finding-19), and two tests were added.

**finding-7**: (round 1) REQ-sjkp8z and REQ-he8ejb read "not exactly 40 / 64 hexadecimal characters" while their tests assert a `0x`-prefixed input is accepted — and `x` is not a hexadecimal character. The permission lived only in the rationale prose.
disposition: both amended in place to permit a single optional `0x` prefix. An implementer working from the item form alone would otherwise have written a parser that rejects what these tests require it to accept, and PR-5mc4d8's entire single-versus-repeated-strip argument was anchored to that prose.

**finding-8**: (round 1) Two of the register's honesty passages were stale in the direction of overclaiming — §3 claim 3's count of verification-outcome assignment sites described the pre-change component, and claim 9 said the vacuous test "has been renamed to state what it asserts" when the new name still ends in a clause reading as coverage over environments.
disposition: claim 3 corrected. Claim 9's name left as it stands and the overclaim recorded beside it, with the observation that two successive attempts to name that test honestly both reached for a clause about the environment — because the tempting thing to say is what the test would establish if it could.

**finding-9**: (round 2) §3's headline claim — "every decision named in a requirement below was moved out … into a free function, and gated there" — is false for REQ-2k7ys4, REQ-dp95pv and REQ-jfxah3, whose decisions live in `next_outcomes` while what is gated is `emissions_for`, handed an already-chosen outcome.
disposition: the claim corrected to say most, with the three named and the distinction spelled out: the consequence is gated, the recognition is not. Added as claims 12 and 13 of §3.

**finding-10**: (round 2) REQ-he8ejb's amended wording forbade what its own test requires — whitespace is neither hexadecimal nor the permitted prefix, yet `trims surrounding whitespace before measuring` asserts a padded identifier is accepted.
disposition: amended a third time, to "ignoring surrounding whitespace and an optional single `0x` prefix". A consequence of finding-7's fix: sharpening the wording exposed a permission that lived only in the code.

**finding-11**: (round 2) The plan carried no record of the round that minted RC-3rddh7, so its latest measurement read 90 tests and nineteen requirements while the tree held 102 and 21.
disposition: both review rounds written into the plan, and every subsequent round recorded as it completed.

**finding-12**: (round 2) Three frontend behaviours have no requirement behind them — the `'(unknown organisation)'` substitution, the `detail` column, and the `receiver-stopped` consumer — while the change minted REQ-wu6z9p as the frontend twin of REQ-kn5rtx on exactly the reasoning that would require them.
disposition: recorded as **PR-j9f6kk**, open, age limit 2026-10-14, on the owner's decision of 2026-09-14 to fix the blocking four and record the rest. The asymmetry in the author's own reasoning is named in the report rather than smoothed.

**finding-13**: (round 2) The unit glossary declares Persona and Epoch as defined in the root glossary, where neither is defined — two of the register's most load-bearing terms, each glossary pointing at the other.
disposition: recorded in PR-j9f6kk.

**finding-14**: (round 2) The glossary mints **Operator** with `_Avoid_: user, admin, end user`, and the change's own ledger then uses "user" in that sense nine times.
disposition: recorded in PR-j9f6kk. A glossary the ledger written alongside it does not follow is not yet a glossary.

**finding-15**: (round 2) Six of the eight `verify.row.test.ts` tests assert fields their annotated requirement does not mention and would pass unchanged if the requirement's actual behaviour broke.
disposition: accepted as accurate and left as it stands — the annotations name the requirement the file as a whole verifies, and the outcome assertions that bear on REQ-a83vqr and REQ-akt4p7 are present and discriminating (proved by mutation in finding-2's disposition). Recorded here rather than in the register because it is a precision question about annotation granularity, not a gap in evidence.

**finding-16**: (round 2) `app/.guardrails/config.yaml` asserted measured test counts it had not re-measured, and the requirement count was wrong in four places.
disposition: corrected, and re-measured at every subsequent round. This is the defect class nine gate rounds kept finding; see Gaps.

**finding-17**: (round 2) `receiver.log.test.ts` was omitted from the register's enumeration of the frontend suite — the only file gating REQ-wu6z9p, and therefore the frontend half of the newest control.
disposition: added.

**finding-18**: (round 2) §3 claim 3, itself a correction, was still wrong: post-change there are two `VerifyEvent` construction sites, not three.
disposition: corrected to describe the sites without a count that the next edit invalidates.

**finding-19**: (round 2) REQ-bvx4nh's data-directory clause, added by finding-6's fix, is asserted by no test annotated to REQ-bvx4nh; the only `data_dir` assertion sits under `verifies: REQ-e4ah9h`.
disposition: two tests added, `data_dir_comes_from_the_built_configuration` and `abnormal_data_dir_paths_are_reported_verbatim`. The mutation that proved them also failed a pre-existing REQ-e4ah9h test — which is the finding made concrete: before this, the only thing standing between a constant data directory and a green suite was a test annotated to a requirement that says nothing about data directories.

**finding-20**: (round 2) `api.ts` still described five event types after the change took the count to eight.
disposition: corrected.

**finding-21**: (round 3) **The receiver's terminal-error detection had never worked.** The loop matched `"endpoint not bound"`, a substring occurring nowhere in `org-node` or `iroh`; the real messages are `"endpoint bind: …"`, `"endpoint bind failed unexpectedly"` and `"endpoint closed"`. So the loop had never broken, `receiver-stopped` had never been emitted in production, the guard had never been released by task exit, and three statements in the register rested on it working.
disposition: verified against the source before acting. The markers now match the messages that occur and three tests pin them. **REQ-jfxah3 had been satisfied vacuously** — the second requirement in this change with correct tests over a behaviour the system never exhibited — and HAZ-cfp4jb's residual claim that "the exit is announced and a restart works" is withdrawn and rewritten. The real behaviour was worse than the register described: `ensure_endpoint` caches the endpoint, so after `"endpoint closed"` the loop reused the dead endpoint indefinitely, emitting one error event per iteration — a hot spin, not a quiet absorbing state. Reddens without it: `every_real_terminal_message_stops_the_loop`, which against the old constant produced no `Stopped` outcome at all. The substring match itself remains unsound; REQ-x3c8n2 is unchanged.

**finding-22**: (round 3) RC-3rddh7's classification named `OrgNotOnChain` and `Trie(_)` as verification verdicts; both are also raised by purely local conditions in the receive path — the local store having no record of the organisation, and the local snapshot failing to reconstruct — so a local-state problem rendered as ✗ MISMATCH, reinstating the hazard the control was minted to remove.
disposition: verified against `org-node/src/service.rs` before acting. Both reclassified as receiver errors: the app cannot tell the origins apart from the variant, and a verdict must not be claimed where it cannot be supported — a misfiled verdict is the hazard, a misfiled receiver error is only less specific. The fix task then found `Trie(_)` has four origins, two of them local, so the exclusion was under-argued rather than over-argued. Reddens without it: `locally_reachable_variants_are_classified_as_receiver_errors`.

**finding-23**: (round 3) `subscribeAll` leaks every already-registered listener when any one subscription rejects, and the test annotated to REQ-rq8g2v pins the leaking behaviour in.
disposition: accepted as accurate and **not fixed** — the failure mode needs a decision about what a partial subscription failure should do, which is a requirement question rather than a defect with an obvious answer. Recorded here and in Gaps. The test's name and comment describe what it does; what they do not say is that no cleanup at all is the outcome they lock in.

**finding-24**: (round 3) §1's completeness correction — the paragraph whose point is that counts inside completeness claims must be measured — itself said "four of the seven components" where §6 cites three.
disposition: corrected to three.

**finding-25**: (round 3) `verify.ts` makes a second sentinel substitution with no requirement behind it: `epoch: null, root: null` on a failure row, at exactly the site HAZ-5ha5vv's reasoning says substitutions must be minted, gated under a requirement about a row's outcome.
disposition: added to PR-j9f6kk, which had named its two siblings from the same function and missed this one.

**finding-26**: (round 3) REQ-2k7ys4's and REQ-dp95pv's `RecordUnreadable` assertions run on a single organisation fixture with no abnormal-input case — the same gap a gate round already fixed for the structurally identical REQ-tw4cb5 and did not extend here.
disposition: accepted and recorded rather than fixed. The one-fixture observation had already been raised by gate round 3 and carried forward unblocking by every round since; this is the same gap seen from the other side. It is the weakest of the 21 requirements' evidence and is named as such in Gaps.

**finding-27**: (round 3) `api.ts` still read "Events — 5 event types" in a second place.
disposition: corrected.

**finding-28**: (round 3) `DeltaBaseMismatch` and `RootMismatch` compare the envelope against the *local* trie, so a corrupt local snapshot yields a verdict against a good envelope.
disposition: recorded against PR-5mc4d8 rather than acted on. Both stay classified as verdicts, defensibly — verification is a verdict on the pair — but whether the display should distinguish "your copy is wrong" from "their update is wrong" is a question no item answers.

## Gaps

What this change did **not** establish. A gap stated here is a gap; a gap left out is a claim.

**No coverage figure of any kind.** No `coverage_command` is configured, so neither statement nor decision coverage is measured for this unit. Under class C both are mandatory. Owner-accepted deferral to tooth 5; decision coverage is unmeasured in all four units.

**No test renders a Svelte component.** This is the largest hole. The frontend suite tests extracted functions, so nothing asserts that `Revoke.svelte` calls `validateRevokeInput`, that `Membership.svelte` calls `verifyResultFrom`, or that the newly reachable `✗ MISMATCH` branch is wired to the template. A defect leaving the tested function correct and the component calling something else passes every gate here. Five of the unit's requirements are realised in the frontend and nowhere else. Not-minted control 1; the residual probabilities of HAZ-n97v5g and HAZ-5ha5vv rest on it staying open.

**Thirteen claims are held by review rather than by a gate**, enumerated in the register's §3 and not repeated here. Two of the thirteen were added because a mutation came back green, and two more because a gate round measured a claim and found it false. The list grew every round, which is the direction it should move: it measures what the evidence does not reach.

**Eight of twelve command handlers are registered in the IPC suite and never invoked.** The four driven are the ones reachable without a chain, a peer or a populated store. Registration is not exercise: a renamed argument or DTO field on any of the other eight would still pass.

**No chain, no peer, no second machine, and no test drives the receiver loop.** Every claim of the form "when org-node returns X, the app does Y" is tested on the app's half and assumed on org-node's. The receiver loop needs a bound iroh endpoint, a real organisation, a chain and a peer, so the guard's placement and the emit-then-break ordering are both unreachable by any test here.

**`AppState::init` is called by no test.** The startup refusals are gated one layer below, at the policy functions where the decision lives; `init`'s own wiring — that it reads the three variables it claims to, in the right order, and propagates the refusal — is exercised by nothing.

**REQ-dp95pv's two tests share one fixture**, and REQ-bmk2z2's and REQ-jfxah3's abnormal-input cases are bounds and message assertions rather than malformed input, because the behaviour each requires exists only on a failure path. The first is the weakest evidence among the 21 requirements; all three were raised, argued and accepted by successive rounds rather than waived once.

**A partial subscription failure leaves listeners attached** (finding-23), and no requirement says what should happen instead.

**Five problem reports are open**, each with a dated checklist item: PR-w5dae4 (startup failure arrives as a panic, which is what completes two of this change's own controls), PR-h6xpnh, PR-eecx3y, PR-5mc4d8, PR-j9f6kk. One expectation stands against `org-node`, REQ-x3c8n2, due 2026-12-13, and two of the six hazards' residual probabilities depend on it being met.

**The documentation's accuracy about itself was the recurring defect.** Nine gate rounds and three reviews found exactly one class of error in the prose, repeatedly: a claim that outlived what supported it. Every instance was introduced by a correct fix to something else, and the last round found one surviving its own fix one screen below where the fix was applied. The gates that caught it are the ones that measure rather than read. Nothing in this record should be read as asserting that class is now closed — only that it was measured at every round and cleared at each.
