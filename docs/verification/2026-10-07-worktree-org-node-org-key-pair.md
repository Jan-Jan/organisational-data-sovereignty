# Verification — Organisation key pair (2026-10-07)

One record per change, written at `merge-change` step 6b and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line.

branch: worktree-org-node-org-key-pair
reviewer: one independent review round, a fresh subagent with its own worktree (.worktrees/worktree-org-node-org-key-pair-review), the diff, the plan and the ledger items, no implementation narrative, suites re-run itself; round 1 on 2026-10-07
verdict: approve — round 1 raised six low code/requirement findings and one record finding, so by the convergence rule it is the last round; findings 1, 2, 3, 6 and 7 fixed in place before the squash merge, finding-4 recorded as PR-gnh3j2 and finding-5 as PR-q8r32t
reproduced: yes — each defect this change resolves was reproduced as a failing test before its fix: PR-xwek5e and PR-szkat6 by T2's tests (B's record held no key), PR-ve9zw8 by T9's tests (B committed a substituted key), PR-g9u3xq by T8's test (a removed Device held a key still in use); see the plan's red → green attestations

Units touched: org-node, app. on-chain-client is untouched (no file under
`on-chain-client/` changes); its lines were run because the merge gate runs
every unit.

Change: change 3 of the owner's chain-authority sequence — the Organisation
secret is replaced by one X25519 Organisation private key per epoch, carried
in every Organisation-information Wire message, checked against the chain's
`org_pub_key` before it is stored (RC-9cefcn), drawn fresh for every
provisional update and taken into the record on commit; a revocation carries
no key, goes only to a Device the committed record no longer lists, and is
accepted only as the receiver's own removal; the invite identifier no longer
travels between peers. Branched from `master` at `fdf4e77`; master was merged
in at `c036fab`.

Base: step 1 fetched `origin` successfully; `origin/master` (`fdf4e77`) is an
ancestor of the local `master` (`c036fab`), which is ahead and unpushed, so the
local base was merged (owner standing instruction) and the pre-flight ran with
`--local-base`.

Plan: `docs/plans/2026-10-06-org-key-pair.md`.

Resolves: PR-szkat6, PR-ve9zw8, PR-xwek5e, PR-g9u3xq (org-node).
Opens: PR-gnh3j2, PR-q8r32t (org-node); PR-3ue4va (app).
Noted, stays open: PR-qmvj83 (org-node; a dated note records that the
revocation no longer carries the snapshot); PR-2dmjzj (org-node; the Loopback
recipient check, LLR-jn5jeh's abnormal case).
Owner-accepted for this merge: the class-C coverage shortfall (2026-10-07,
below). The owner's rulings of 2026-10-06 that shape this change are recorded
in `org-node/docs/requirements/2026-10-07-org-key-pair.md`.

## The gate

Measured on: `02a4084ec990417c9f78e8c0575bd88ab23e3453` — `git rev-parse HEAD`
— tree `e5d83d9ed32bfe1d13d557c18cf5b81a51fa7b8e`, from
`git rev-parse HEAD^{tree}` on a clean worktree, unchanged from the start of
the run to its end; no `Cargo.lock` changed. This is gate 4, run after the
review-round-1 fixes and the ledger finalization; `merge-change` step 3 renamed
nothing on this pass. Each count is the sum of the `test result:` lines of that
command's own output, keyed to the target above it.

| Gate | Result |
| --- | --- |
| org-node `cargo test -p org-node --features app,test-support --lib --test …` (26 targets) | 247 passed, 0 failed, 0 ignored over 23 counted targets: admission_sender 52, commit_paths 32, verify_against_chain 23, persona_records 16, node_value_types 12, organisation_key 11, key_custody 10, receive_chain_reads 9, secret_redaction 9, service_lifecycle 9, value_types 9, expected_admission 8, store_at_rest 7, wire_frame_bound 7, absences 6, transport_handshake 6, provisional_store 5, encoding_golden 4, envelope_binding 4, chain_read_state 3, service_stories 3, lib 1, transport_networked 1; the four bolero fuzz targets (fuzz_envelope_decode, fuzz_first_admission_base, fuzz_verify_against_chain, fuzz_wire_decode) ran their 1 s budget (no count printed) |
| org-node `quint --version`; `quint typecheck` ×2; `quint run` ×5 (forkSafety, revocationSafety, revokedExcludedFromOrgSecret, tauWindow, convergence) | 0.33.0; typechecks exit 0 (silent on success); each run `[ok] No violation found` |
| app `cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support --test …` (10 targets) | 159 passed, 0 failed: ipc 33, receiver_events 32, csp_policy 24, invitation 17, org_id_parsing 14, startup_policy 13, connection_status 10, submit_flow 7, receiver_guard 6, state_assembly 3 |
| app `npm --prefix app run check` | 198 files, 0 errors, 1 warning (environmental: tsconfig "Cannot find type definition file for 'node'") |
| app `npm --prefix app run test` | 7 files, 44 passed |
| on-chain-client `cargo test … --features test-support --lib --test …` (12 targets) | 73 passed, 0 failed: decode_revive_event 17, best_lane_reorg_rule 14, storage_slot_layout 11, decode_org_state 9, log_ownership 7, h160_mapping 5, runtime_version_dispatch 5, contract_address_filter 3, type_widths 2, lib 0; the three bolero fuzz targets ran their 1 s budget (no count printed) |
| on-chain-client `cargo test … --features test-support,write --test write_manifest --test write_pure --test write_compose --test write_events` | 34 passed, 0 failed: write_pure 16, write_events 11, write_compose 5, write_manifest 2 |
| org-members `cargo test -p org-members`; quint typecheck ×3, `quint test`, `quint run --invariant=mbtInv` (not touched) | 271 passed, 0 failed, 1 ignored (`preflight_probe`, by design); typechecks ok; quint test 63 passing; mbtInv no violation |
| person `cargo test -p person`; `cargo clippy -p person --all-targets -- -D warnings` (not touched) | 123 passed, 0 failed (one fuzz target, 1 s); clippy clean |
| `cargo clippy -p org-node --lib --bins --features app,test-support -- -D warnings`; app `cargo clippy … --all-targets -- -D warnings` | both exit 0, clean |
| `check-ids.sh --allow-draft-files`, each of the five units | exit 0, no output; no `DRAFT-` file remains |
| `check-trace.sh`, each of the five units | exit 0 in all five; only UNRESOLVED-PR and UNMET-EXPECTATION lines. This change's IDs: PR-szkat6, PR-ve9zw8 and PR-xwek5e left the open list (PR-g9u3xq was opened and resolved here and never entered it); PR-gnh3j2, PR-q8r32t (org-node) and PR-3ue4va (app) entered it |
| `check-units.sh` | exit 0 |
| Implements map | every ID of the plan's Implements header is verified by an annotated test or carried by a tested LLR (REQ-y7tsft, REQ-tcutr6 and the seven SDDs, through their LLRs) |
| Coverage, against class C | on-chain-client 41.75% lines, 42.60% regions, 36.21% functions, decisions unmeasured — short of class C, **owner-accepted 2026-10-07**; org-members 96.55% lines, 94.57% regions; person 100% lines and regions; org-node and the app have no coverage command |
| Working tree | clean; tree unchanged across the run |

Earlier gate runs on this branch — gate 1 on tree `f3e7584` (which found
thirteen robustness gaps and dispatched task `robust`), gate 2 on `44e46df`
(after the master merge `c036fab`), gate 3 on `f66a5614` (after the first
finalization) — are history in the plan's gate sections; none of their figures
is used above.

## Red → green

One row per ID of the plan's Implements header, copied from the red → green
attestations of tasks T1–T11, task `robust` and the review-fix task `r1fix`.
Test paths are relative to the repository root. Where a test was green before
its implementation and its red was shown another way — under a stub, under a
production mutation, by running the old test text against the new code, or
only at compile time — the row says so. A test listed as "re-run" carries an
amended ID whose amendment changes none of its assertions: it was run green
by the task named and was not watched red in this change. SDDs, the RC and two
REQs are covered through the tested LLRs named in their rows.

| Item | Test | Watched red |
| --- | --- | --- |
| REQ-szq3ud | org-node/tests/admission_sender.rs::pr_xwek5e_another_members_removal_leaves_the_receiver_holding_the_organisations_key; org-node/tests/admission_sender.rs::pr_szkat6_an_admitted_member_holds_the_organisation_private_key; org-node/tests/commit_paths.rs::send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other; also through LLR-6ymd6d, LLR-qsjde3, LLR-8hdu9x | (1) written against the old API, runtime-red on the unchanged source in `private_key_held` (B's record held no key) (T2); (2) the same runtime red (T2); (3) compile-red as its T2 predecessor (E0560 no `org_private_key` on `WireMessage`, E0061 `send_update` arity), and compile-red again when rewritten in T4 (16 errors, E0223, no OrgInformation/Revocation kinds) |
| REQ-stx9v3 | org-node/tests/organisation_key.rs::an_update_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused; org-node/tests/commit_paths.rs::admit_member_keeps_a_provisional_update_and_changes_nothing_else; org-node/tests/commit_paths.rs::revoke_member_keeps_a_provisional_update_and_changes_nothing_else; org-node/tests/admission_sender.rs::pr_g9u3xq_a_removed_device_holds_no_key_used_after_its_removal; org-node/tests/provisional_store.rs::every_updates_private_key_lives_only_in_its_provisional_update | under T8's structural stub (the `ChangeSet` had its field, `keep_change_set` still copied the record's key): (1) `unwrap_err` on Ok, no draw and no refusal; (2) assert_ne, update key == record key; (3) assert_ne, no fresh key pair; (4) "the removal published a fresh key" (keys equal), and after cycle A "A took it"; (5) compile-red only (E0559, no field `org_private_key`) — the test builds the update itself, so no runtime red is possible once the field exists (T8) |
| REQ-jy6ybw | org-node/tests/commit_paths.rs::commit_update_commits_a_provisional_update_the_chain_carries; org-node/tests/commit_paths.rs::commit_update_selects_the_update_by_its_root_and_its_key; org-node/tests/admission_sender.rs::pr_g9u3xq_a_removed_device_holds_no_key_used_after_its_removal | (1) passed under T8's stub (update and record shared one key); runtime-red after cycle A: "the record takes the update's key pair"; (2) red under the stub and after cycle A: `unwrap_err` on Ok (selection by root alone); (3) red under the stub and after cycle A, as in REQ-stx9v3 (T8) |
| REQ-c29s93 | org-node/tests/receive_chain_reads.rs::a_message_without_its_organisation_private_key_is_refused_before_the_chain; org-node/tests/receive_chain_reads.rs::a_first_admission_without_its_key_is_refused_and_keeps_the_expectation; org-node/tests/wire_frame_bound.rs::an_organisation_information_body_without_its_snapshot_or_key_does_not_decode; org-node/tests/fuzz_wire_decode | (1) runtime-red: left `Chain("iroh recv: malformed wire message")`, right `MalformedMessage` (403-byte body with its key cut off) (T6); (2) the same runtime red on a 287-byte body (T6); (3) compile-red (E0223 `WireMessage::OrgInformation` on a struct, E0599 no `envelope`) (T4); (4) compile-red (E0223) before the enum; exits 0 after (T4) |
| REQ-bwx7eg | org-node/tests/admission_sender.rs::pr_ve9zw8_a_first_admission_carrying_a_substituted_key_is_refused; org-node/tests/admission_sender.rs::pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths | (1) runtime-red: `unwrap_err` on Ok, `ReceiveOutcome { … epoch: Epoch(2) … }` — B committed the substituted key (T9); (2) runtime-red: `unwrap_err` on Ok, `Epoch(3)` (T9) |
| REQ-ju6vn2 | org-node/tests/admission_sender.rs::pr_xwek5e_another_members_removal_leaves_the_receiver_holding_the_organisations_key; org-node/tests/admission_sender.rs::pr_szkat6_an_admitted_member_holds_the_organisation_private_key; org-node/tests/admission_sender.rs::a_committed_update_gives_the_record_the_new_key_pair_on_both_paths | (1), (2) runtime-red on the unchanged source (B's record held no key) (T2); (3) runtime-red: left `OrgPublicKey(81edea9e..)`, right `OrgPublicKey(d4e3cf61..)` — the record kept the genesis public key (T9) |
| REQ-3dsweu | org-node/tests/admission_sender.rs::a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths; org-node/tests/commit_paths.rs::send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other; org-node/tests/wire_frame_bound.rs::the_two_kinds_round_trip_and_a_body_of_neither_kind_is_refused | (1) runtime-red: `unwrap_err` on Ok, `ReceiveOutcome { … epoch: Epoch(3) … }` — the relabelled message was committed (T5); (2) compile-red (16 errors, E0223) (T4); (3) compile-red (E0223, E0599) before the enum (T4) |
| REQ-vxqc5g | org-node/tests/admission_sender.rs::a_revocation_about_an_organisation_not_held_is_refused_before_the_chain; also through LLR-38e2kn, LLR-j5vbqj | compile-red only (admission_sender: 27 errors, no Revocation kind, no `envelope()`) when rewritten from `first_admission_without_a_record_snapshot_is_refused` (T4); no runtime red recorded |
| REQ-8amu2a | org-node/tests/expected_admission.rs::expect_admission_records_an_organisation_once_and_reaches_the_disk; org-node/tests/expected_admission.rs::an_expectation_for_another_organisation_survives_the_commit; org-node/tests/expected_admission.rs::an_expected_first_admission_is_committed_and_clears_the_expectation +6 more | (1) compile-red (E0061 `expect_admission` takes 3 arguments, E0063 missing `invite_id`) (T3); (2) compile-red (E0061) (T3); (3) and the other re-asserted expected_admission tests compile-red (E0061/E0063) (T3), `…_decodes_its_snapshot_before_reading_the_chain` compile-red again in T4 (11 errors); `admission_sender::a_first_admission_to_an_expected_organisation_rests_on_the_chain_alone` and `value_types::the_first_admission_refusals_name_their_organisation` re-run, not red; also through LLR-ms8njy (runtime red, T3) |
| REQ-hzm4kt | org-node/tests/store_at_rest.rs::organisation_private_key_does_not_appear_in_the_file +7 more (store_at_rest, service_lifecycle); also through LLR-s78sh7 | (1) compile-red (`OrgRecord` still had `org_secret`) as the rewritten `organisation_secret_does_not_appear_in_the_file` (T2); the other seven re-run (T2), not red |
| REQ-y7tsft | through LLR-bwb9pu, LLR-ecxc76, LLR-2dvhz8 | transitive (see those rows) |
| REQ-tcutr6 | through LLR-w4mhd4 | transitive (see that row) |
| RC-9cefcn | org-node/tests/admission_sender.rs::pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths; implemented by REQ-c29s93 and REQ-bwx7eg | runtime-red: `unwrap_err` on Ok, `Epoch(3)` (T9); otherwise transitive (see those rows) |
| SDD-swtd3w | through LLR-j5vbqj, LLR-qsjde3, LLR-ms8njy, LLR-sz4xhc | transitive (see those rows) |
| SDD-kwncn7 | through LLR-js9dsu, LLR-ecxc76 | transitive (see those rows) |
| SDD-af5vnt | through LLR-s78sh7, LLR-95753m, LLR-byjvd9, LLR-qjz3q4, LLR-7cmp38 | transitive (see those rows) |
| SDD-89es4z | through LLR-sj7cd5, LLR-s6qnht | transitive (see those rows) |
| SDD-rx2yvy | through LLR-6ymd6d, LLR-e2b7gv, LLR-ghja3x, LLR-8hdu9x | transitive (see those rows) |
| SDD-8cpyfa | through LLR-38e2kn, LLR-pt32fx, LLR-xn5pwc, LLR-6s785x, LLR-ba2ejp, LLR-4kh9w9 | transitive (see those rows) |
| SDD-72ddm6 | through LLR-pt32fx, LLR-tax3pm, LLR-ba2ejp, LLR-jwhzh3 | transitive (see those rows) |
| LLR-js9dsu | org-node/tests/wire_frame_bound.rs::the_two_kinds_round_trip_and_a_body_of_neither_kind_is_refused; org-node/tests/wire_frame_bound.rs::an_organisation_information_body_without_its_snapshot_or_key_does_not_decode; org-node/tests/encoding_golden.rs::organisation_information_wire_message_is_pinned +2 more (revocation_wire_message_is_pinned, fuzz_wire_decode) | each compile-red before the two-kind enum existed (E0223 on a struct, E0599 no `envelope`; encoding_golden: no `WireMessage::OrgInformation`); derived literals match the plan's byte for byte (316 and 169 bytes); the fuzz target exits 0 after (T4) |
| LLR-ecxc76 | org-node/tests/secret_redaction.rs::a_wire_message_of_either_kind_never_renders_the_key | compile-red only (E0223/E0599, no two kinds, no `envelope()`) (T4) — a never-renders property with no runtime red |
| LLR-j5vbqj | org-node/tests/value_types.rs::every_rejection_variant_is_distinct_from_every_other; org-node/tests/value_types.rs::the_received_message_refusals_say_what_they_refuse; org-node/tests/receive_chain_reads.rs::a_message_without_its_organisation_private_key_is_refused_before_the_chain | (1), (2) compile-red only (E0599, no variant MalformedMessage, OrgKeyMismatch, RevocationNotHeld, RevocationNotForThisDevice) (T1); (3) runtime-red, `Chain(…)` ≠ `MalformedMessage` (T6) |
| LLR-qsjde3 | org-node/tests/absences.rs::org_node_holds_no_organisation_secret | runtime-red on the unchanged source: "types.rs contains `OrgSecret`: LLR-qsjde3" (T2) |
| LLR-byjvd9 | org-node/tests/persona_records.rs::a_record_carries_its_organisation_private_key_and_a_store_without_one_is_refused | compile-red (E0308 expected `Option<OrgPrivateKey>`, E0599 no `expose_secret` on `Option`) (T7) |
| LLR-6ymd6d | org-node/tests/commit_paths.rs::send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other; org-node/tests/commit_paths.rs::send_update_refuses_an_organisation_it_holds_no_record_of; org-node/tests/admission_sender.rs::a_revocation_reaches_the_administrators_disk | (1) compile-red (T4); (2) red under a production mutation (`send_update` looking up `orgs.first()` instead of the named Organisation): left `Chain("no persona bound to organisation …")`, right `OrgNotOnChain`; green after revert (robust); (3) compile-red (admission_sender, no Revocation kind to match) (T4) |
| LLR-e2b7gv | org-node/tests/organisation_key.rs::an_update_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused; org-node/tests/commit_paths.rs::admit_member_keeps_a_provisional_update_and_changes_nothing_else; org-node/tests/commit_paths.rs::revoke_member_keeps_a_provisional_update_and_changes_nothing_else +1 more (pr_g9u3xq_…) | each red under T8's structural stub, as in REQ-stx9v3 (T8) |
| LLR-xn5pwc | org-node/tests/receive_chain_reads.rs::a_message_without_its_organisation_private_key_is_refused_before_the_chain; org-node/tests/receive_chain_reads.rs::a_first_admission_without_its_key_is_refused_and_keeps_the_expectation; org-node/tests/admission_sender.rs::a_committed_update_gives_the_record_the_new_key_pair_on_both_paths | (1), (2) runtime-red, `Chain("iroh recv: malformed wire message")` ≠ `MalformedMessage` (T6); (3) runtime-red, the record kept the genesis public key (T9; its LLR-xn5pwc annotation added in robust) |
| LLR-ba2ejp | org-node/tests/admission_sender.rs::pr_ve9zw8_a_first_admission_carrying_a_substituted_key_is_refused; org-node/tests/admission_sender.rs::pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths; org-node/tests/admission_sender.rs::a_persona_whose_member_key_the_chain_publishes_is_still_bound +1 more (a_first_admission_records_the_chains_key_the_private_key_and_the_member) | (1), (2) runtime-red, B committed the substituted key (`Epoch(2)`, `Epoch(3)`) (T9); (3) passed before the implementation (nothing checked the key); red shown two ways: with the comparison in `check_carried_key` inverted it fails with `OrgKeyMismatch`, and the old body against the real implementation fails the same way (T9); (4) its new line already held since T8, so red shown under a stub in which `set_record_keys` does not write the private key: "the private half of the chain's key (LLR-ba2ejp)" (T9) |
| LLR-38e2kn | org-node/tests/admission_sender.rs::a_revocation_about_an_organisation_not_held_is_refused_before_the_chain; org-node/tests/receive_chain_reads.rs::the_self_delete_path_refuses_an_unheld_organisation_without_a_chain_read | (1) compile-red only (27 errors) (T4); (2) existing test, annotation added in T4; not red in this change |
| LLR-6s785x | org-node/tests/commit_paths.rs::commit_update_commits_a_provisional_update_the_chain_carries; org-node/tests/commit_paths.rs::commit_update_refusals_change_nothing_and_write_nothing; org-node/tests/commit_paths.rs::commit_update_selects_the_update_by_its_root_and_its_key +1 more (pr_g9u3xq_…) | (1) passed under T8's stub; runtime-red after cycle A ("the record takes the update's key pair"); (2) red under the stub and after cycle A: `unwrap_err` on Ok (a chain root under an unheld key was committed); (3) red under the stub and after cycle A (selection by root alone) (T8) |
| LLR-4kh9w9 | org-node/tests/admission_sender.rs::a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths; org-node/tests/admission_sender.rs::a_committed_update_gives_the_record_the_new_key_pair_on_both_paths | (1) runtime-red, the relabelled message was committed at `Epoch(3)` (T5; LLR-4kh9w9 annotation added in robust); (2) runtime-red, the record kept the genesis public key (T9) |
| LLR-pt32fx | org-node/tests/admission_sender.rs::a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths; org-node/tests/admission_sender.rs::a_self_delete_reaches_the_revoked_nodes_disk | (1) runtime-red, `unwrap_err` on Ok, `Epoch(3)` (T5); (2) existing test of the accepting branch, LLR-pt32fx annotation added after gate 3 (r1fix); not red in this change |
| LLR-37cj3n | org-node/tests/admission_sender.rs::pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths; org-node/tests/admission_sender.rs::update_relayed_by_a_non_member_after_admission_is_committed | (1) runtime-red, `Epoch(3)` (T9); (2) re-run (T2, T9), not red. The item's text was amended again in place for review finding-3 |
| LLR-8hdu9x | org-node/tests/admission_sender.rs::a_revocation_reaches_the_administrators_disk; org-node/tests/commit_paths.rs::send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other; org-node/tests/admission_sender.rs::a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths | (1) compile-red (no Revocation kind) (T4); (2) compile-red (T2, T4); (3) runtime-red (T5; LLR-8hdu9x annotation added in robust) |
| LLR-9zfnmb | org-node/tests/expected_admission.rs::expect_admission_records_an_organisation_once_and_reaches_the_disk; org-node/tests/admission_sender.rs::a_receiver_holding_two_organisations_commits_into_the_one_the_change_names | (1) compile-red (E0061, E0063) (T3); (2) compile-red with T2's two-Organisation tests (E0061 `send_update` arity, E0609 no field `org_private_key`) (T2) |
| LLR-bg3vsw | org-node/tests/commit_paths.rs::a_joiner_that_missed_its_admission_rebuilds_from_the_next_updates_snapshot; org-node/tests/commit_paths.rs::commit_update_commits_a_provisional_update_the_chain_carries; org-node/tests/commit_paths.rs::send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other +1 more (service_stories::five_stories_full_e2e) | (1) red under a production mutation (`commit_update` encoding the after-snapshot): panicked "B rebuilds from the snapshot before the update: DeltaBaseMismatch"; green after revert (robust); (2) runtime-red after T8's cycle A, as above (T8); (3) compile-red (T4); five_stories_full_e2e re-run (T2), not red |
| LLR-ckk5nz | org-node/tests/admission_sender.rs::a_first_admission_records_the_chains_key_the_private_key_and_the_member; org-node/tests/admission_sender.rs::pr_xwek5e_another_members_removal_leaves_the_receiver_holding_the_organisations_key; org-node/tests/admission_sender.rs::a_committed_update_gives_the_record_the_new_key_pair_on_both_paths | (1) runtime-red on the unchanged source (B's record held no key) (T2), and red under T9's `set_record_keys` stub; (2) runtime-red (T2); (3) runtime-red, the record kept the genesis public key (T9). The item's text was amended again in place for review finding-2 |
| LLR-ghja3x | org-node/tests/organisation_key.rs::an_update_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused; org-node/tests/commit_paths.rs::admit_member_keeps_a_provisional_update_and_changes_nothing_else; org-node/tests/admission_sender.rs::the_admission_envelope_carries_the_epoch_its_update_produced | (1) red under T8's stub, `unwrap_err` on Ok (T8; LLR-ghja3x annotation added in robust); (2) red under the stub, update key == record key (T8); (3) re-run (T4, T8), not red |
| LLR-j6j95z | org-node/tests/expected_admission.rs::an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain; org-node/tests/admission_sender.rs::a_revocation_about_an_organisation_not_held_is_refused_before_the_chain | (1) compile-red (expected_admission: 11 errors, `with_snapshot` matched kinds that did not exist) (T4); (2) compile-red only (T4) |
| LLR-jn5jeh | org-node/tests/admission_sender.rs::in_loopback_mode_the_joiner_is_dialled_at_the_full_address; org-node/tests/commit_paths.rs::send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other | (1) re-run after T2/T4 edits, not red; (2) compile-red (T4). Abnormal case not verified (PR-2dmjzj, Gaps) |
| LLR-jsx922 | org-node/tests/admission_sender.rs::a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths; org-node/tests/service_stories.rs::revocation_of_another_member_is_committed_not_self_deleted; org-node/tests/admission_sender.rs::an_update_that_does_not_revoke_us_reaches_the_disk +2 more | (1) runtime-red (T5); (2) its behaviour changes rather than a feature being missing, so red shown by running the pre-rewrite text against the new `service.rs`: "B must verify and commit C's revocation: RevocationNotForThisDevice { … }" (T5); (3) and the other two re-run (T2, T5), not red |
| LLR-jwhzh3 | org-node/tests/admission_sender.rs::a_self_delete_removes_only_the_organisation_the_revocation_came_from; org-node/tests/admission_sender.rs::membership_of_one_organisation_is_judged_by_that_organisations_personas_alone; org-node/tests/receive_chain_reads.rs::the_self_delete_path_refuses_an_unheld_organisation_without_a_chain_read | (1) compile-red with T2's two-Organisation tests (E0061, E0609) (T2); (2) re-run (T2, T5), not red; (3) existing test, LLR-jwhzh3 (abnormal) annotation added in robust; not red in this change |
| LLR-mbjfq8 | org-node/tests/admission_sender.rs::pr_ve9zw8_a_first_admission_carrying_a_substituted_key_is_refused; org-node/tests/expected_admission.rs::an_expected_first_admission_is_committed_and_clears_the_expectation; org-node/tests/admission_sender.rs::a_first_admission_to_an_expected_organisation_rests_on_the_chain_alone +1 more | (1) runtime-red, `Epoch(2)` (T9); (2) compile-red (E0061/E0063) (T3); (3) and `a_first_admission_that_misses_the_chain_root_commits_nothing` re-run (T3), not red |
| LLR-q8emds | org-node/tests/expected_admission.rs::an_expected_first_admission_is_committed_and_clears_the_expectation; org-node/tests/expected_admission.rs::a_refused_first_admission_leaves_the_expectation; org-node/tests/expected_admission.rs::an_expectation_for_another_organisation_survives_the_commit +4 more | (1), (2), (3) compile-red (E0061/E0063) (T3); the four others (admission_sender ×3, service_stories) re-run (T2, T3, T4), not red |
| LLR-s78sh7 | org-node/tests/store_at_rest.rs::organisation_private_key_does_not_appear_in_the_file; org-node/tests/store_at_rest.rs::seeds_do_not_appear_in_the_file | (1) compile-red (`OrgRecord` still had `org_secret`) (T2); (2) re-run, not red. An absence property: normal case only (Gaps) |
| LLR-tax3pm | org-node/tests/organisation_key.rs::a_revocation_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused; org-node/tests/commit_paths.rs::revoke_member_keeps_a_provisional_update_and_changes_nothing_else | (1) red under a production mutation (`keep_change_set`'s equality check made never-true): "unwrap_err() on an Ok value: ProvisionalUpdate { … }"; green after revert (robust); (2) red under T8's stub, no fresh key pair (T8) |
| LLR-u6rq4s | org-node/tests/admission_sender.rs::a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths; org-node/tests/admission_sender.rs::a_removal_relayed_by_the_member_it_removes_is_committed; org-node/tests/admission_sender.rs::update_from_the_admin_after_admission_is_committed +1 more | (1) runtime-red (T5); (2) red shown by running the pre-rewrite text against the new `service.rs`: "a chain-valid removal is committed whoever relays it: RevocationNotForThisDevice { … }" (T5); (3) and `update_relayed_by_a_non_member_…` re-run (T2, T4, T9), not red |
| LLR-y2v8v2 | org-node/tests/admission_sender.rs::a_receiver_holding_two_organisations_commits_into_the_one_the_change_names; org-node/tests/expected_admission.rs::an_expectation_for_one_organisation_admits_no_other | (1) compile-red with T2's two-Organisation tests (T2); (2) compile-red (E0061/E0063) (T3) |
| LLR-ayrdr8 | org-node/tests/encoding_golden.rs::persona_store_plaintext_is_pinned; org-node/tests/encoding_golden.rs::organisation_information_wire_message_is_pinned; org-node/tests/encoding_golden.rs::revocation_wire_message_is_pinned +1 more (truncated_pinned_values_are_refused) | (1) compile-red with the suite (T2, T3); runtime-red against the old code, `DeserializeBadOption` at offset 493 (T7); runtime-red before any source change, `DeserializeBadVarint` on the new GOLDEN_STORE (T8); (2), (3) and (4) compile-red (no `WireMessage::OrgInformation`) (T4); each literal matches its derivation |
| LLR-bwb9pu | org-node/tests/secret_redaction.rs::records_and_wire_messages_never_render_secret_bytes; org-node/tests/secret_redaction.rs::secrets_at_the_high_byte_bound_and_many_records_stay_unrendered; org-node/tests/secret_redaction.rs::a_wire_message_of_either_kind_never_renders_the_key +2 more | (1), (2) compile-red only (E0560 on `WireMessage.org_private_key`, `OrgRecord` missing `org_secret`) (T2); (3) compile-red only (T4); the other two re-run, not red — a never-renders property with no runtime red |
| LLR-g76zqd | org-node/tests/absences.rs::org_node_has_no_administrator_key; org-node/tests/persona_records.rs::create_persona_holds_the_parsed_details_across_a_reopen; org-node/tests/persona_records.rs::a_persona_record_decoded_directly_refuses_an_invalid_handle +1 more (provisional_store::provisional_updates_round_trip_through_the_encrypted_file) | re-run (T2, T3, T7, T8); not red in this change |
| LLR-sz4xhc | org-node/tests/node_value_types.rs::secret_types_redact_debug_and_give_bytes_only_through_the_accessor; org-node/tests/node_value_types.rs::secret_debug_is_the_same_whatever_the_bytes; org-node/tests/node_value_types.rs::secret_types_serialise_as_the_plain_bytes | re-run in T2 (`OrgSecret` dropped from the secret types); no red recorded in the plan |
| LLR-2dvhz8 | org-node/tests/secret_redaction.rs::a_record_debug_renders_the_organisation_private_key_redacted; org-node/tests/secret_redaction.rs::records_and_wire_messages_never_render_secret_bytes; org-node/tests/organisation_key.rs::the_organisation_private_key_is_not_in_the_record_debug_output | (1) compile-red (E0308 at secret_redaction.rs:92, expected `Option<OrgPrivateKey>`) (T7); (2) compile-red (T2); (3) re-run (T7), not red |
| LLR-322xfu | org-node/tests/node_value_types.rs::the_organisation_private_key_is_a_secret_type_and_the_only_way_to_its_key_pair; org-node/tests/secret_redaction.rs::the_organisation_private_key_and_its_key_pair_never_render_its_bytes | already verified, test unchanged (the amendment only drops `OrgSecret` from the list it is compared with); not red in this change |
| LLR-3fwykc | org-node/tests/admission_sender.rs::a_first_admission_records_the_chains_key_the_private_key_and_the_member; org-node/tests/admission_sender.rs::pr_szkat6_an_admitted_member_holds_the_organisation_private_key; org-node/tests/admission_sender.rs::pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths +3 more | (1), (2) runtime-red on the unchanged source (B's record held no key) (T2); (3) runtime-red, `Epoch(3)` (T9; LLR-3fwykc (abnormal) annotation added in robust); the two organisation_key tests and `commit_genesis_creates_the_record_…` re-run (T2, T7), not red |
| LLR-sj7cd5 | org-node/tests/organisation_key.rs::an_update_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused; org-node/tests/organisation_key.rs::an_organisation_key_equal_to_a_genesis_key_is_refused; org-node/tests/organisation_key.rs::a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals +1 more | (1) red under T8's stub, `unwrap_err` on Ok (T8); (2), (3) and `…_any_genesis_member_or_device_key_is_refused` re-run (T7, T8), not red |
| LLR-2xzys9 | org-node/tests/commit_paths.rs::send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other; org-node/tests/commit_paths.rs::send_update_refuses_without_a_bound_persona_or_a_loopback_address | (1) compile-red (T2, T3, T4); (2) re-run (T2, T3), not red |
| LLR-48jakr | org-node/tests/absences.rs::org_node_holds_no_invite_identifier; org-node/tests/commit_paths.rs::send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other | (1) runtime-red: "types.rs contains `InviteId`: LLR-ms8njy" (absences 5 passed, 1 failed) (T3); (2) compile-red (E0061 `send_update` takes 4 arguments) (T3), and again in T4 |
| LLR-7cmp38 | org-node/tests/commit_paths.rs::discarding_an_unknown_root_is_refused_and_writes_nothing; org-node/tests/commit_paths.rs::two_updates_for_the_same_change_are_two_and_one_is_discarded_alone; org-node/tests/commit_paths.rs::discarding_a_genesis_update_removes_it_and_its_private_key_and_saves +2 more | (1) red under T8's stub: `unwrap_err` on Ok (another update's key was discarded) (T8); (2) red under the stub: `assert_ne(first.org_pub_key, second.org_pub_key)` (T8); (3) and the other two re-run (T8), not red |
| LLR-95753m | org-node/tests/expected_admission.rs::expect_admission_records_an_organisation_once_and_reaches_the_disk; org-node/tests/provisional_store.rs::an_update_with_the_same_identity_replaces_and_others_are_kept; org-node/tests/commit_paths.rs::two_updates_for_the_same_change_are_two_and_one_is_discarded_alone +1 more | (1) compile-red (E0061, E0063) (T3); (2) red under T8's stub: left 4, right 5 (the same root under another key replaced the update) (T8); (3) red under the stub (T8); `provisional_updates_round_trip_…` re-run, not red |
| LLR-cmdrp9 | org-node/tests/commit_paths.rs::commit_update_commits_a_provisional_update_the_chain_carries; org-node/tests/commit_paths.rs::commit_update_refusals_change_nothing_and_write_nothing; org-node/tests/commit_paths.rs::commit_update_selects_the_update_by_its_root_and_its_key +1 more | as LLR-6s785x (T8); `admission_sender::an_admission_reaches_the_administrators_disk` re-run (T8), not red |
| LLR-ms8njy | org-node/tests/absences.rs::org_node_holds_no_invite_identifier; org-node/tests/encoding_golden.rs::organisation_information_wire_message_is_pinned; org-node/tests/encoding_golden.rs::revocation_wire_message_is_pinned | (1) runtime-red: "types.rs contains `InviteId`: LLR-ms8njy" (T3); (2) compile-red as `admission_wire_message_is_pinned` (T3), then compile-red when renamed (T4); (3) new in T4, compile-red before the enum |
| LLR-mxskg9 | org-node/tests/value_types.rs::every_rejection_variant_is_distinct_from_every_other; org-node/tests/expected_admission.rs::a_first_admission_that_lists_none_of_our_personas_is_refused; org-node/tests/value_types.rs::the_first_admission_refusals_name_their_organisation +3 more | (1) compile-red only, extended in T1 (E0599) — with derived `PartialEq` no stub can make distinct variants compare equal; (2) compile-red (E0061/E0063) (T3); the others already verified (plan header), re-run, not red |
| LLR-qjz3q4 | org-node/tests/provisional_store.rs::every_updates_private_key_lives_only_in_its_provisional_update; org-node/tests/commit_paths.rs::admit_member_keeps_a_provisional_update_and_changes_nothing_else; org-node/tests/organisation_key.rs::the_organisation_private_key_is_kept_only_in_the_encrypted_store +2 more | (1) compile-red only (E0559) (T8); (2) red under T8's stub (T8); (3) and the two commit_paths genesis tests re-run (T7, T8), not red |
| LLR-s6qnht | org-node/tests/commit_paths.rs::create_organisation_keeps_a_genesis_update_and_nothing_else; org-node/tests/organisation_key.rs::an_organisation_key_equal_to_a_genesis_key_is_refused | already verified (plan header), tests unchanged; LLR-s6qnht (abnormal) annotation added to (2) in robust; not red in this change |
| LLR-s8xp7m | org-node/tests/expected_admission.rs::an_expected_first_admission_is_committed_and_clears_the_expectation; org-node/tests/expected_admission.rs::an_expectation_for_one_organisation_admits_no_other; org-node/tests/expected_admission.rs::an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain +1 more | (1), (2), (3) compile-red (E0061/E0063) (T3); (3) compile-red again (11 errors) (T4); `an_unexpected_first_admission_is_refused_before_the_snapshot_or_the_chain` re-run, not red |
| LLR-wzqqg9 | org-node/tests/commit_paths.rs::commit_genesis_creates_the_record_once_the_chain_carries_the_root; org-node/tests/commit_paths.rs::commit_genesis_refusals_change_nothing_and_write_nothing; org-node/tests/commit_paths.rs::a_chain_past_epoch_one_refuses_commit_genesis_and_keeps_the_update +1 more | re-run (T2, T7; the commit_genesis test re-asserted on the key); no red recorded in the plan |
| LLR-q225ws | app/src-tauri/tests/submit_flow.rs::each_command_sends_once_to_one_device_and_the_kind_follows_the_record; app/src-tauri/tests/submit_flow.rs::a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing; app/src-tauri/tests/invitation.rs::a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding | (1) compile-red (E0432 unresolved `commands::revoke_and_send`); then runtime-red under stubs: a `todo!()` body panicked, and a stub sending to a Member other than the removed one failed at submit_flow.rs:198 "the removed Device is not listed" (T10); (2) added for review finding-6, green on the code as written; red under a production mutation making `revoke_and_send` fall back to `members.last()`: it built, wrote and committed a revocation against Bob (r1fix); (3) compile-red (lib E0432/E0061) (T10) |
| LLR-8krgzj | app/src-tauri/tests/ipc.rs::admit_member_takes_no_organisation_secret; app/tests/api.admit.test.ts "invokes admit_member with the org id, the reply and the peer address alone" | (1) compile-red (lib E0432 `org_node::OrgSecret`, E0061, E0004) (T10); (2) runtime-red: AssertionError, extra `"orgSecretHex": null` in the invoke args (T11) |
| LLR-ctrfz4 | app/src-tauri/tests/ipc.rs::admit_member_takes_no_organisation_secret +3 more (the peer-address tests) | (1) compile-red (T10); the three peer-address tests re-argued (T10), not red. Amendment is a note only |
| LLR-gha5f6 | app/src-tauri/tests/invitation.rs::a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding +4 more | (1) compile-red (lib E0432/E0061; the old assertion read `WireMessage.invite_id`) (T10); the other four re-run (T10), not red |
| LLR-w4mhd4 | app/src-tauri/tests/invitation.rs::a_confirmed_reply_carries_the_persona_and_declares_the_expected_admission; app/src-tauri/tests/ipc.rs::produce_invite_reply_refuses_without_confirmation_over_ipc; app/src-tauri/tests/invitation.rs::no_reply_is_produced_and_nothing_declared_without_confirmation | (1) compile-red (E0061 `expect_admission` arity, E0432 `org_node::InviteId`) (T10); (2), (3) re-run (T10), not red |
| LLR-f35pda | app/src-tauri/tests/invitation.rs::outstanding_invite_ids_survive_a_reopen_and_settle_one_at_a_time +2 more | amendment is a note only; its tests recompile against the app's own `InviteId` with unchanged assertions (T10); not red in this change |

Two items outside the Implements header were amended in place when the plan's
open questions were closed (2026-10-06): LLR-7bk6qh (the four new
`OrgNodeError` variants are receiver errors), verified by
`app/src-tauri/tests/receiver_events.rs::locally_reachable_variants_are_classified_as_receiver_errors`
— compile-red (E0004 in events.rs), then runtime-red under a stub classifying
the four as verdicts ("MalformedMessage is reachable from a local condition and
must not be a verdict", receiver_events.rs:765) (T10); and LLR-rys5nx ("no
other public key"), whose tests were re-run.

Tests named in the attestations under an earlier name survive renamed:
`send_update_sends_the_committed_update_with_the_records_key_and_writes_nothing`
(T2) is now `send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other`
(T4); `admission_wire_message_is_pinned` (T2, T3) is now
`organisation_information_wire_message_is_pinned` (T4);
`organisation_secret_does_not_appear_in_the_file` is now
`organisation_private_key_does_not_appear_in_the_file` (T2). Deleted tests and
their replacing evidence are listed in the plan's "Tests deleted" table.

## What was wrong, and what was built

Before this change the Organisation secret was an opaque value that nothing
authenticated: no receiver could check it against the chain (PR-ve9zw8, a
relay could substitute it); it was never given to an admitted Member
(PR-szkat6, B's record held no key — the Organisation key was conflated with
the founding Member's); and an update that carried none cleared the stored one
(PR-xwek5e, another Member's removal left the receiver without the key). Nor
was the key ever rotated, so a removed Device held a key still in use after its
removal (PR-g9u3xq).

Under the owner's rulings of 2026-10-06
(`org-node/docs/requirements/2026-10-07-org-key-pair.md`) this change builds:

- org-node: the Organisation secret is gone (LLR-qsjde3). There is one X25519
  Organisation private key per epoch, held in every record (LLR-byjvd9). The
  Wire message has two kinds (LLR-js9dsu): Organisation information, which
  always carries the key the sending node's record holds (REQ-szq3ud) — its
  absence is a parse error (REQ-c29s93, LLR-xn5pwc) — and a revocation, which
  carries none. A receiver stores the carried key only when its public half is
  the chain's `org_pub_key` (RC-9cefcn, REQ-bwx7eg, LLR-ba2ejp), and stores it
  on commit (REQ-ju6vn2). Every provisional root draws a fresh key pair
  (REQ-stx9v3, LLR-e2b7gv) that the commit takes into the record (REQ-jy6ybw,
  LLR-6s785x). A revocation goes only to a Device the committed record no
  longer lists, and is accepted only as the receiver's own removal (REQ-3dsweu,
  LLR-pt32fx); one about an Organisation not held is refused before the chain
  (REQ-vxqc5g). The invite identifier never travels between peers; an
  expected admission names the Organisation alone (REQ-8amu2a, LLR-ms8njy).
- app: `admitMember` and `admit_member` take no Organisation secret
  (LLR-8krgzj); each command sends once to one Device, the kind following the
  record (LLR-q225ws); the four new refusals are receiver errors (LLR-7bk6qh).
  The fan-out to every Device is not done here (PR-3ue4va).

## Review

One round, by a fresh subagent in its own worktree
(`.worktrees/worktree-org-node-org-key-pair-review`), given the diff, the plan
and the ledger items and no account of the work. It re-ran the suites itself:
org-node 247 passed, 0 failed (4 bolero targets exit 0); app 158 passed, 0
failed; vitest 44 passed; check-trace for org-node and the app reported no
errors. Every code and requirement finding was `low`, so by the convergence
rule of `merge-change` step 6a this was the last round and no further reviewer
was dispatched; the fixes went through the sequence again from step 1 (gate 4
above).

Verdict, verbatim: "approve — all code and requirement findings are low, so
this is the last review round; fix findings 1–6 in place before the squash
merge."

**finding-1**: requirement, low — `org-node/docs/CONTEXT.md:36-46` (Wire message) still says the message carries "the invite identifier of the admission it carries, if any" and that "the snapshot is sent with every update", and that the invite identifier is what a waiting node matches. This change edited this entry and left those lines in. In the tree no message carries an invite identifier (LLR-ms8njy) and a `Revocation` carries no snapshot (LLR-bg3vsw). `org-node/docs/CONTEXT.md:103-107` (Expected admission) still defines it as "An Organisation identifier and invite identifier", but `ExpectedAdmission { org_id }` holds only the Organisation (`org-node/src/store.rs`, LLR-95753m). The glossary contradicts the amended items.
disposition: the glossary entries *Wire message* and *Expected admission* in `org-node/docs/CONTEXT.md` reworded to the two kinds (no invite identifier; a revocation carries no snapshot) and to an expectation that names the Organisation alone (task r1fix). Text only; no test reddens.

**finding-2**: requirement, low — LLR-ckk5nz (`org-node/docs/architecture/2026-10-03-decomposition.md:1953`) says "`commit_held` takes no key or secret argument". The code at `org-node/src/service.rs:636` is `commit_held(org_id, verified, org_private_key: OrgPrivateKey, org_pub_key: OrgPublicKey)`, and `commit_update` and `commit_received` both pass the key through it. The behaviour is right; the item misstates the internal interface.
disposition: LLR-ckk5nz amended in place to state the interface the code has (task r1fix). The behaviour was already right and its tests are unchanged.

**finding-3**: requirement, low — LLR-37cj3n (`org-node/docs/architecture/2026-10-03-decomposition.md:1772-1781`) says the record `receive_and_verify` produces is decided "never by a value carried in the Wire message other than its Envelope (and, on a first admission, its snapshot)". Since this change, the record's `org_private_key` is the value the message carries (LLR-ckk5nz, REQ-ju6vn2). The chain's key constrains it, but the message decides it. The item should name the carried key as the third input.
disposition: LLR-37cj3n amended in place to name the carried Organisation private key as the third input (task r1fix). Its tests are unchanged.

**finding-4**: requirement, low — REQ-szq3ud (`org-node/docs/requirements/2026-10-07-org-key-pair.md:68`) promises "the Organisation private key of the epoch the message's update reaches". `send_update` (`org-node/src/service.rs:725`) sends the key the record holds when it runs. If a later commit came first, that is a later epoch's key next to an earlier Envelope. LLR-6ymd6d's note admits this, and the receiver then fails verification anyway, so nothing is lost. The REQ wording is still stronger than the tree, in a rare state.
disposition: recorded as open problem PR-gnh3j2 (org-node).

**finding-5**: code, low — `org-node/src/service.rs:800` (`(None, None) => RevocationNotHeld`) and `:841` (`let Some(org_private_key) = carried_key else RevocationNotHeld`) cannot be reached: the arm at `:783` already returns for every revocation about an Organisation not held. No test can exercise them. Either restructure, e.g. carry the first-admission data in the match, or make them `unreachable!`, so the code does not hold untestable refusal paths that look like extra REQ-vxqc5g sites.
disposition: recorded as open problem PR-q8r32t (org-node).

**finding-6**: code, low — LLR-q225ws (`app/docs/architecture/2026-10-07-org-key-pair.md:21`) is verified only by the normal-case tests `each_command_sends_once_to_one_device_and_the_kind_follows_the_record` (`app/src-tauri/tests/submit_flow.rs`) and `a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding`. No test carrying its annotation covers an abnormal input. For example, `revoke_and_send` with a member id the record does not hold should send nothing and keep no update. Also, the revoked Member in the test has one Device, so choosing `device_keys.first()` cannot be told apart from any other Device. Existing ipc tests cover the command's argument refusals under other IDs.
disposition: new test `a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing` (`app/src-tauri/tests/submit_flow.rs`, verifies LLR-q225ws). It reddens under a mutation making `revoke_and_send` fall back to `members.last()`: it built, wrote and committed a revocation against Bob (task r1fix).

**finding-7**: record — `org-node/docs/risk/2026-10-07-org-key-pair.md:24` has a stray "refused ;". Also, `org-node/.guardrails/config.yaml` (the new comment over `verify_commands`) points to "the verification record" for per-target counts. No verification record is in this diff yet; that is expected only if merge-change writes it.
disposition: the typo fixed; the config comment now resolves to this record, which carries the per-target counts (The gate, above).

After gate 3 (tree `f66a5614`) one further gap was closed in the same task:
`a_self_delete_reaches_the_revoked_nodes_disk` now carries LLR-pt32fx, for the
accepting branch of the own-removal rule.

## Gaps

- **Coverage.** org-node and the app have no coverage command; decision
  coverage is unmeasured everywhere; on-chain-client is short of class C
  (41.75% lines, 42.60% regions). Owner-accepted for this merge (2026-10-07).
- **LLR-jn5jeh's abnormal case** is not verified: in Loopback mode
  `send_update` dials `peer_addr` without checking that it is the recipient's
  Device (PR-2dmjzj, open).
- **No input side.** LLR-s78sh7 (the key never appears in the store file) and
  LLR-ecxc76 (a Wire message never renders its key) are verified only by
  normal-case absence and rendering tests; the properties have no abnormal
  input to drive.
- **Compile-red only, or never red here.** Several rows above were watched
  failing only at compile time, or are re-runs of unchanged tests that were not
  red in this change; each says so.
- **Fan-out.** The app sends each committed update to one Device only; every
  other current Member stays on the earlier epoch's key until it receives
  Organisation information (PR-3ue4va, the `org-io` session's work).
- **Open review follow-ups.** REQ-szq3ud's wording is stronger than the tree
  in a rare ordering (PR-gnh3j2); two unreachable refusal arms remain in
  `service.rs` (PR-q8r32t).
- **Clippy debt.** org-node `clippy --all-targets` reports 32 pre-existing
  findings on test-file lines this change did not touch; `--lib --bins` is
  clean, as is the app's `--all-targets`.
- **Toolchain.** The bolero fuzz targets print no counts; quint typecheck and
  check-ids print nothing on success; `npm run check` reports one
  environmental warning.
- **No live chain.** Every chain read is evidenced against substitutes.
- **Not reviewed again:** the round-1 fixes and this record, which were gated
  but, per the convergence rule, not reviewed again.
