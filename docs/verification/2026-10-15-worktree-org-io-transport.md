# Verification — S4, transport, invitation exchange and storage IO leave org-node (2026-10-15)

One record per change, written at `merge-change` step 5 and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line and carries the gate's figures in its own trailers; this
file is the rest of the evidence that travels with the change.

branch: worktree-org-io-transport
reviewer: independent subagent (fresh context), round 1; independent subagent (fresh context), round 2; independent subagent (fresh context), round 3 — each at its round's `review dispatched` commit, with the diff, the plan and the review checklist and no implementation narrative
verdict: round 3, verbatim: "Ready to merge. All three suites are green, every round-2 disposition holds in the tree, and the only new findings are one low and one record." Round 3 was the last review round (its code and requirement findings are all low); each of its findings is fixed. The earlier rounds' verdicts are quoted under their own headings below.
reproduced: yes — every defect this change fixes was reproduced by a test that failed before its fix: PR-gnh3j2 (`a_later_commit_cannot_change_an_earlier_body`, org-node, red against the send-time rule), PR-jav8zn (`a_commit_whose_store_write_fails_says_the_commit_was_made_and_reloads`, org-io, compile-red before the store files existed), PR-n696wv (`the_file_holds_exactly_the_sealed_bytes`, `a_write_interrupted_after_the_temporary_leaves_the_old_store_unsealing`, compile-red), PR-8qsnhx (`a_second_organisations_message_goes_out_under_its_own_personas_key`, compile-red; its pin of the old behaviour was inverted), PR-7v8anh (`a_command_completes_while_the_receiver_waits`, compile-red, and red with the node lock taken before the wait), PR-kwwap5 (`a_declared_first_admission_is_committed_by_the_one_path`, compile-red), PR-2dmjzj and PR-3ue4va (the `send_fanout` tests, compile-red, and red against a dropped no-recipient check and a fan-out stopping at the first failure), PR-tcd2ta (`after_a_stop_start_receiver_starts_a_loop_that_runs`, watched failing in T19 while `#[ignore]`d, and the three T20 `endpoints` tests, each red before the fix), PR-47aycj (`a_send_that_rebinds_a_closed_endpoint_while_the_receiver_runs_accepts_on_it`, red before the fix), PR-erskh4 (the three `stale_check` tests and org-io's `a_check_that_went_stale_is_checked_again_once`, red before the fix), PR-y6t5kt (`a_persona_created_while_the_receiver_runs_is_received_on`, `no_reply_is_produced_while_receiving_for_a_persona_whose_endpoint_cannot_be_bound`, both red first), PR-9q5ems, PR-aa7prn and PR-d2p5y2 (the reproducing tests their resolutions name: `a_settle_whose_record_holds_its_executed_update_does_not_report_it_not_executed` and `a_settle_after_a_later_epoch_was_committed_does_not_report_its_executed_write_not_executed`; `lend_to_keychain_returns_no_value_the_writer_could_carry_the_bytes_in`; `a_stalled_connection_does_not_stop_another_peers_message`), PR-jgvf3b (`a_reconcile_whose_commit_is_not_stored_keeps_the_unknown_write_and_sends_next_time`, run against mutated code with the write not kept). PR-7zz76t closed obsolete (no defect left to reproduce) and PR-6f3qku was resolved by the org-node all-targets clippy line going red → clean (T18). Pins of existing behaviour, each with a mutation check instead of a red-first run, are marked "NOT red first" or "pin" in the table below (T9, T11a, T15, T17, T19, T21, T22, T23, the review-round-1 tests and the conformance tests).

Units whose product code or ledgers this change edits: org-io, org-node, app.
The two guardrails upgrades merged into the branch (6a9d6eb, 344faa3 on
master; 2bc3cca) touch every unit's `.guardrails/config.yaml`.

Change: stage S4 of the org-io roadmap — org-node becomes free of IO (no file,
endpoint, keychain or async), org-io takes the transport (one iroh endpoint per
Persona), one sealed store file and one OS keychain item per Persona, the
lock-free receive loop and the invitation exchange, and the app depends on
org-io alone and starts with no passphrase, every committed update going to
every listed Device in Networked mode.
Plan: `docs/plans/2026-10-06-org-io-transport.md` (task list "## Tasks",
reports under "## Progress").

## The gate

The gate's figures are not in this record. The squash commit carries them as
trailers (`merge-change` step 7): `Gate-tree:` names the tree the step 6a gate
measured, a `Gate:` line gives each command's totals, and `Base:` names the
base and the commit step 1 merged. The base is local `master` (the S3b-io
session's merge notes, item 1: local master is ahead of `origin/master`).

## Red → green

Copied from the `red -> green:` lines of the plan's task reports (T1–T23), the
"Reconciled with S3b-io part 1" note, and the review and conformance fix rows
of its Progress table. A REQ, RC or SDD is verified through the LLRs it names;
its own red is theirs. Rows for tests later deleted or renamed are kept under
the name they had: the observation was made.

| Item | Test | Watched red |
| --- | --- | --- |
| LLR-37k2s3, LLR-yp69jg (org-node) | `a_device_secret_debug_renders_only_the_marker`, `a_device_secret_declares_zeroize_on_drop`, `the_device_key_is_the_ed25519_verifying_key_of_the_seed`, `a_generated_secret_differs_from_the_next_one`, `from_keychain_bytes_refuses_any_length_but_32_and_echoes_no_byte`, `lend_to_keychain_calls_the_writer_exactly_once_with_the_seed`, `device_secret_offers_only_its_four_public_operations`, `a_device_secret_cannot_be_cloned_copied_displayed_or_serialised` (key_custody.rs) | T1: all eight compile-red before `DeviceSecret` existed (`error[E0432]: unresolved import org_node::keys::DeviceSecret`), green after |
| LLR-er2x8n, LLR-8kh3zf, LLR-pkruy8 (org-node); LLR-7bk6qh (app) | `an_encoded_message_is_its_postcard_body_with_no_length_prefix`, `a_body_over_the_bound_is_refused_before_decoding`, `a_body_that_is_not_a_wire_message_is_malformed` (wire_frame_bound.rs); `a_wire_message_over_the_bound_is_classified_as_a_receiver_error` (receiver_events.rs) | T2: compile-red (E0432 unresolved `org_node::wire`; E0599 no variant `MessageTooLarge`; app E0004 `MessageTooLarge` not covered), green after |
| LLR-bg538c, LLR-2qqqx6, LLR-bvt5up, LLR-rdda86, LLR-e5kqhg, LLR-hby4jr, LLR-uw7nmv, LLR-pba7yu (org-node) | the 14 `sealed_store` tests (`a_sealed_store_unseals_with_its_secret_and_device_key` … `create_persona_returns_the_secret_and_keeps_only_the_public_key`) and `fuzz_unseal` | T3: compile-red (E0432 unresolved `SealedStore`, `StoreRefusal`, `StoreWrite`, `StoreChange`; E0599 no `PersonaStore::unseal`; missing variants), green after: 14 passed, `fuzz_unseal` no panic |
| LLR-3zdw8v (org-io) | `device_secret_is_named_only_in_its_sanctioned_places` | T3: runtime-red ("the transitional holder holds the secrets"), green after |
| LLR-677fqq, LLR-8erfbu, LLR-n9dnm4, LLR-36jer8 (org-io) | the nine `keychain.rs` tests; `keyring_is_named_only_in_the_keychain_module_and_its_mock_only_under_test_support`; `keychain_bytes_reach_no_clone_debug_or_serialise` | T8: compile-red (E0432 unresolved `org_io::test_support::keychain`, `MemoryKeychain`, `KeychainError`); the two scans runtime-red (no `keychain.rs`); green after |
| LLR-677fqq | `no_lower_unit_depends_on_a_keychain_crate` | T8: NOT red first (an absence check). Mutation-checked: with org-io added to the scanned units it failed ("org-io depends on the keychain crate `keyring`") |
| LLR-zetn7a, LLR-6qjmhf (org-node); S3a's LLR-2r2fha, LLR-kzgjz8, LLR-3aysup, LLR-r7zm39, LLR-tx8ruv, LLR-5azhry, LLR-38e2kn, LLR-s8xp7m, LLR-b27jr6, LLR-pt32fx, LLR-ba2ejp restated | the 18 `receive_path` tests (`a_body_that_does_not_decode_is_malformed_and_changes_nothing` … `no_receive_branch_is_unreachable`, renamed 2026-10-15 `an_unheld_organisations_notice_is_refused_as_not_held`) | T4: all 18 compile-red (E0432 unresolved `Checked`, `CheckedMessage`, `ReceiveKind`; E0599 no `check_received`/`commit_checked`); `no_receive_branch_is_unreachable` also runtime-red after the first implementation until the store choice moved to `revocation::notice_store`; 18/18 green |
| LLR-gdfqn9, PR-gnh3j2 (org-node) | the nine `outgoing_bodies` tests | T5: compile-red (E0432 unresolved `OutgoingMessage`; E0609 no field `sender`), green after; `a_later_commit_cannot_change_an_earlier_body` also runtime-red against the send-time rule ("the first commit's key") |
| LLR-3zdw8v, LLR-gdfqn9 (org-io) | `no_handle_method_returns_a_type_holding_an_outgoing_body` | T5: runtime-red ("org-io/src/lib.rs:396: returns CommitOutcome: pub async fn submit_commit_send"), green after; LLR-gdfqn9 later dropped from its annotation (T18, cross-unit) |
| LLR-nr8df3, LLR-7q9g9z, LLR-d55brn, LLR-yexf5m, LLR-b7wgpf, LLR-pba7yu (org-node) | the 16 `invitation` tests; `fuzz_invitation_parse` | T6: compile-red (E0432 unresolved `org_node::invitation`; E0599 no `build_invite`/`build_reply`/`check_reply`/`settle`; missing variants; E0609 no field `outstanding_invites`), green after |
| LLR-ayrdr8 (re-pin) | `persona_store_plaintext_is_pinned` | T6: runtime-red (round-trip `left == right` at encoding_golden.rs:158), green once `outstanding_invites` landed; derived by the format rule |
| LLR-nt8dja, LLR-b9bm36, LLR-murjr6, LLR-uzax55, LLR-85zque, LLR-n9dnm4, LLR-amv4mb, REQ-6d9xv6, PR-n696wv, PR-jav8zn (org-io) | the first 15 `store_files` tests (`each_persona_has_its_own_file_named_for_its_device_key` … `a_failed_removal_is_reported_and_the_persona_reloaded`) | T9: compile-red (E0432 unresolved `org_io::store_file`, `CreateError`, `OpenError`, `OperationError`, `StoreFailure`; E0425 no `fail_next_write_after_temporary`; E0599 no `OrgIo::create_persona`); mutation-checked too (without `restore_store` in the reload the three reload tests fail) |
| LLR-rdda86 (org-node) | `restore_store_replaces_or_adds_one_store_and_drop_store_removes_one` | T9: compile-red (E0599 no `restore_store`/`drop_store`), green after |
| LLR-jd82t6 (org-node) | `only_the_removal_step_adopt_and_drop_store_remove_records_or_stores` | T9: runtime-red against a temporary `self.stores.retain(\|_\| true)` in `restore_store` ("service.rs calls `stores.retain(` (LLR-jd82t6)") |
| LLR-murjr6 | `a_dropped_persona_keeps_its_file_and_item_and_opens_again` | T9: NOT red first (written after `drop_store` existed). Mutation-checked: with the `drop_store` call removed from the reload-failure branch it fails ("dropped from the running service") |
| LLR-t4v69b | `no_code_names_the_old_files` | T9: NOT red first (an absence check). Mutation-checked: with `persona_store.bin` in a `store_file.rs` comment it fails |
| LLR-d5y7ks, LLR-d2pzzk, LLR-z35vbh, LLR-h9tcat, REQ-6d9xv6, PR-8qsnhx (org-io) | the seven `endpoints` tests and four `preflight_transport` tests of T10 | T10: compile-red (E0432 unresolved `org_io::transport`, `check_transport`, `check_transport_for_data_dir`; E0599 no `endpoint_for_test`, `start_receiving`, …), green after |
| LLR-d5y7ks; LLR-h9tcat | `iroh_secret_key_is_named_in_one_function_only`; `the_preflight_reads_no_key_or_seed_from_the_environment` | T10: runtime-red ("the scan found bind_from_keychain's use of SecretKey"; "preflight.rs runs the transport check"), green after |
| LLR-9arvqj, LLR-cvyyr8, LLR-szhe7j, PR-7v8anh, PR-kwwap5 (org-io) | the twelve `receive_step` tests of T11 | T11: all twelve compile-red (E0432 unresolved `ReceiveError`, `TerminalCause`; E0599 no `next_received`, …), green after; `a_command_completes_while_the_receiver_waits` also mutation-checked (node lock taken before the wait: fails at the 2 s timeout) |
| LLR-wvgec4, LLR-unqfj5, REQ-ur7z4a, RC-b4m5sj (org-io) | the first six `acknowledgement_inbound` tests | T11a: compile-red (E0432 unresolved `dial_direct_as`, `transport::Verdict`); mutation-checked after green (always Accepted reddens the three refusal tests; dropping the reply reddens the four network tests; a stray `write_all` or `is_listed` check reddens both scans) |
| LLR-unqfj5 | `a_dropped_responder_writes_nothing` | T11a: NOT red first. Mutation-checked: a dropped responder answering Refused reddens it ("no byte written, read [1]") |
| LLR-znhmh2, LLR-r9sh6k, LLR-pw369n, LLR-jn5jeh, PR-3ue4va, PR-2dmjzj (org-io) | the five `send_fanout` tests | T12: compile-red (E0432 unresolved `org_io::send`; E0061 `submit_commit_send` arity; E0609 no field `sends`); mutation-checked too (dropping the no-recipient check, stopping at the first failed send) |
| LLR-2b2ghe | the four new `peer_addr` tests; `fuzz_peer_addr` | T12: compile-red (E0432 unresolved `org_io::peer_addr`), green after |
| LLR-h9qf3s, LLR-9cb443, LLR-y4dg74 (org-io) | `the_invite_is_returned_only_after_its_store_write`, `an_unconfirmed_reply_parses_and_writes_nothing`, `admission_runs_check_admit_write_commit_send_settle_in_order`, `a_failed_send_still_settles`, `a_refused_check_submits_nothing` | T13: compile-red (E0599 no `issue_invite`/`produce_reply`/`admit_reply`, no `FakeChain::updates`; E0432 `ExchangeError`, `store_writes_to`), green after |
| REQ-a2a6rm, LLR-cvyyr8, REQ-2ugh2f | `node_stories::five_stories_full_e2e` | T13: compile-red (E0599 no `issue_invite`/`produce_reply`/`import_reply`/`admit_reply`), green after |
| LLR-4tcxsu, REQ-x3jrh8; LLR-qezw3n (org-node) | `org_node_is_sans_io`; `org_node_holds_no_dialling_address` | T14: runtime-red before the deletions ("src/keys.rs contains `iroh`: LLR-4tcxsu: the transport is org-io's"; "src/transport/endpoint.rs contains `node_addr_for_dial`: LLR-qezw3n"), green after |
| LLR-vzf8j2 (app) | `the_app_manifest_names_no_iroh_base64_or_postcard` | T15: runtime-red ("the app's [dependencies] name ["base64", "postcard", "iroh"]"), green after |
| LLR-3zdw8v (org-io) | `org_io_reexports_no_service_store_secret_or_wire_type_of_org_node` | T15: red with the implementation temporarily reverted (`pub use org_node as node;` restored: "org-io re-exports org-node's internals: the whole of org-node") |
| REQ-m8sgjk; LLR-7bk6qh; LLR-pms47e (app) | `admit_member_hands_the_reply_to_org_io_and_reports_a_committed_admission_whose_send_failed`; `the_store_and_invitation_refusals_are_classified_as_receiver_errors`; "offers a Persona whose Organisation is the empty string as bound" | T15: each NOT red first. Mutation-checked: without `report_sends(&admitted.sends)?` it fails; `OrganisationHeldByAnotherPersona` in the VerifyFailed arm fails it; filter `!p.org_id` fails it |
| LLR-t98pk9; LLR-k5pzsz (org-io, S3b-io items on S4's structure) | `a_settle_whose_commit_is_not_stored_stays_unknown_and_commits_next_time`; `a_reconcile_whose_commit_is_not_stored_keeps_the_unknown_write_and_sends_next_time` (the PR-jgvf3b reproduction) | S3b-io merge (`tm`): verified on mutated code (the write not kept) |
| LLR-zw6jqy, LLR-a9rjtf; LLR-bym4md, LLR-2vg79y; LLR-5wb4vd (app) | `terminal_is_decided_by_the_error_variant_not_its_text`; `the_outcome_kind_comes_from_receive_kind`, `a_revocation_accepted_by_its_notice_is_announced_as_self_deleted`; `the_receiver_holds_no_app_lock_while_waiting` | T16: compile-red (E0308 expected `&OrgNodeError`, found `&ReceiveError`; E0432 unresolved `events::Arrival`; E0425 no `outcome_for_arrival`); the lock test runtime-red against the app's `Mutex<OrgIo>` ("list_personas completes while the receiver waits for a message: Timeout"); green after. LLR-a9rjtf was later removed from the first test's annotation (review round 1, finding-13) |
| LLR-tpkmh3, REQ-7g3k9a (app) | `startup_reads_no_passphrase`; `a_keychain_refusal_fails_startup_naming_the_condition` | T17: the first runtime-red ("the app still names ODS_PASSPHRASE in [.../src/policy.rs, .../src/state.rs]"); the second compile-red only (E0061), NOT red at runtime first, mutation-checked (a generic `map_err` fails it: "the refusal names the Persona's store") |
| REQ-x6kdse, REQ-qza96g, REQ-v6r2gd, REQ-tzr3dk, REQ-jxyh9c, REQ-txkm94 (org-io, the six met expectations) | the twelve `expectations_met` tests | T19: the behaviour existed (owner ruling 1, 2026-10-14), so each carries a mutation check, reverted after (e.g. `issue_invite` without its store write; `open` skipping failed stores; the seed appended to each store file; no `KeyMismatch` check; revocations skipped); `a_body_the_check_refuses_reads_no_chain_and_writes_nothing`'s mutation is weak (no org-io break shows its "no chain read" half) |
| LLR-yrb6th (app) | `a_closed_endpoint_stops_the_loop_announces_it_once_and_releases_the_guard`; `after_a_stop_start_receiver_starts_a_loop_that_runs` | T19: the first mutation-checked (`forget(guard)`, `break`→`continue`, a doubled emit each fail it); the second watched fail ("the restarted loop runs: it announces no stop", PR-tcd2ta), `#[ignore]` removed in T20 and green |
| LLR-z35vbh, LLR-d2pzzk, PR-tcd2ta (org-io) | `after_its_endpoint_closes_start_receiving_binds_it_again_and_receives`, `after_its_endpoint_closes_a_send_binds_it_again`, `a_refused_rebind_is_a_failed_bind_and_the_next_start_binds` | T20: each red before the fix ("start_receiving binds the closed endpoint again"; `Err(Transport(Connect("Endpoint is closed")))`; "the refused rebind was not a failed bind: Ok(())"), green after |
| LLR-z35vbh, LLR-d2pzzk, PR-47aycj | `a_send_that_rebinds_a_closed_endpoint_while_the_receiver_runs_accepts_on_it`; `a_refused_rebind_while_the_receiver_runs_binds_and_accepts_on_the_next_send` | T21: the first red before the fix ("the rebound endpoint has no accept task"); the second NOT red before the fix existed (written after it), red with the send-path fix disabled (`if false &&`, restored) |
| LLR-6qjmhf, PR-erskh4 (org-node); LLR-cvyyr8 (org-io) | `an_update_checked_before_a_local_commit_is_refused_as_moved`, `the_nodes_own_update_relayed_back_before_its_commit_is_refused_as_moved`, `a_revocation_checked_before_a_local_commit_is_refused_as_moved`; `a_check_that_went_stale_is_checked_again_once` | T22: each red before the fix (`RootMismatch`; "committed: ReceiveOutcome{.. kind: Updated ..}"; `RevocationProofRefused{AbsenceProofRootMismatch}`; `RecordMovedSinceCheck` in place of `StaleSeq{3,3}`) |
| LLR-6qjmhf; LLR-7bk6qh (app) | `a_first_admission_checked_before_its_persona_is_bound_is_not_ours`; `a_record_moved_since_the_check_is_classified_as_a_receiver_error` | T22: pins of existing behaviour. Mutation-checked: without the `persona.org_id.is_some()` filter it fails; the variant in the `VerifyFailed` arm fails it |
| LLR-9arvqj; LLR-e5kqhg; LLR-5wb4vd | `a_command_completes_while_the_steps_chain_read_stalls`; `create_persona_refuses_a_seed_whose_store_is_already_held_and_keeps_nothing`; `a_refused_and_a_mutating_command_complete_while_the_receiver_waits` | T23 (both cases per new item): mutation-checked (node lock before `read_state` → "Elapsed"; duplicate check disabled → `Ok(PersonaId…)`; node lock before the inbound wait → "Timeout") |
| LLR-d5y7ks | `the_endpoint_and_its_constructor_are_reachable_only_in_test_builds` | review round 1, finding-1: red on the unchanged tree |
| LLR-z35vbh, PR-y6t5kt | `a_persona_created_while_the_receiver_runs_is_received_on`, `no_reply_is_produced_while_receiving_for_a_persona_whose_endpoint_cannot_be_bound` | review round 1, finding-2: both red first |
| LLR-37k2s3; LLR-4tcxsu, REQ-x3jrh8; LLR-6qjmhf, PR-erskh4; LLR-rdda86; LLR-2qqqx6; LLR-gdfqn9; LLR-h9tcat | `a_device_secrets_seed_is_held_in_zeroizing`; `the_sans_io_scan_catches_grouped_file_imports_and_environment_reads` (folded into `org_node_is_sans_io` at TEST round 2); `a_record_that_appeared_since_the_check_is_refused_as_moved`, `a_record_that_went_since_the_check_is_refused_as_moved`; `a_first_admission_commits_into_no_store_that_only_declared_or_is_only_listed`, `a_first_admission_commits_into_the_store_that_declared_and_is_listed`; `a_store_sealed_under_any_other_key_derivation_is_refused`; `a_refused_commit_returns_no_bodies_and_changes_nothing`; `the_binary_without_a_data_dir_exits_non_zero_before_any_check` | review round 1, findings 8, 9, 11, 12: pins of existing behaviour, each mutation-checked |
| LLR-h9tcat | `the_binary_with_a_non_unicode_data_dir_says_so_and_exits_before_any_check` | CODE round 2, finding-8: red before the fix |
| LLR-ty85xv, LLR-n6twt7, LLR-ctrfz4 (app, moved back) | the seven `ipc.rs` peer-address tests (`revoke_member_treats_a_blank_or_whitespace_only_peer_addr_as_absent` … `admit_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr`) | TEST round 1, finding-4 (task `c4`): pins, each mutation-checked in commands.rs (blank refused, `0x` mis-stripped, the error swallowed) |
| LLR-36jer8 (org-io) | `the_in_memory_keychain_keeps_the_credential_contract`, `the_os_keychain_keeps_the_credential_contract`; `the_in_memory_keychain_refuses_as_a_locked_store_does`, `a_locked_os_keychain_refuses_as_no_storage_access` | TEST rounds 1–3: contract tests of the fake; the locked half mutation-checked (the fake answering PlatformFailure failed it). The real-keychain run, by the owner in a normal terminal 2026-10-15: `ODS_REAL_KEYCHAIN_TESTS=1 cargo test -p org-io --features test-support --test keychain_contract` → 2 passed, 0 failed (the sandbox blocks keychain writes) |
| LLR-nqdr97 (org-io) | `a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing` | TEST round 2 owner ruling (task `ct3`, `ct4`): pin, mutation-checked (another error → fails) |
| LLR-3vb4k9 (org-io) | `iroh_is_named_only_in_the_transport_module_and_test_support` | ARCHITECTURE round 1 (task `af`): red on the old tree (send.rs:124, peer_addr.rs:11/28/35/75), green after the move into `org-io/src/transport/` |
| LLR-8uf9a5, REQ-mnd9rj, RC-96w7nf, PR-d2p5y2 (org-io) | `a_stalled_connection_does_not_stop_another_peers_message`, `a_stalled_connection_is_dropped_after_the_limit`, `a_seventeenth_connection_waits_for_a_slot_while_sixteen_stall` | review round 2, finding-1 (task `ra`, commit `104cba2b`): `a_stalled_connection_does_not_stop_another_peers_message` watched fail before the fix (the second peer's body never arrived within 5 s), green after; `a_stalled_connection_is_dropped_after_the_limit` mutation-checked (the timeout removed → fails); `a_seventeenth_connection_waits_for_a_slot_while_sixteen_stall` mutation-checked (the cap set to 17 → fails) |
| LLR-t98pk9, LLR-nqvbb6, PR-9q5ems | `a_settle_whose_record_holds_its_executed_update_does_not_report_it_not_executed`, `a_settle_after_a_later_epoch_was_committed_does_not_report_its_executed_write_not_executed` | review round 2, A3 (task `rb`, commit `a75e25c3`): the first watched fail before the fix (settle returned NotExecuted where Reconciled(InStep{epoch 2}) was expected), the second watched fail before the fix (the resubmission was told "did not execute … rebuild the update"); both green after; mutation-checked (dropping the `epoch == expected_epoch` guard fails the second, dropping `!holds_update(..)` fails the first) |
| LLR-yp69jg, LLR-37k2s3, PR-aa7prn (org-node) | `lend_to_keychain_returns_no_value_the_writer_could_carry_the_bytes_in` | review round 2, finding-3 (task `rc`, commit `5f6daebf`): watched fail before the fix (the signature read `…<R>(&self, write: impl FnOnce(&[u8;32]) -> R) -> R`), green after the change to `Result<(), E: Error>`; a signature check (review round 3, finding-1: a caller-defined error type wrapping the bytes rests on review) |
| LLR-cc59ys; LLR-d2pzzk; LLR-szhe7j; LLR-zw6jqy | `no_public_function_writes_or_removes_a_file`; `a_closing_endpoint_is_not_handed_out_again`; `a_connection_failing_before_a_body_is_not_terminal`; `store_and_keychain_failures_are_one_receiver_error_and_do_not_stop_the_loop` | review round 2, findings 2, A7, 7 (tasks `rb`, `ra`, `rc`): `no_public_function_writes_or_removes_a_file` watched fail on the old tree (public `write_store` and `remove_store`), green after; `a_closing_endpoint_is_not_handed_out_again` watched fail before the fix (the same sockets handed back), green after; `a_connection_failing_before_a_body_is_not_terminal` mutation-checked (`Connection` mapped to `Terminal` → fails); the app test mutation-checked (Store or Keychain moved to the Terminal arm → fails; commit `83754a48`). The A6 flag order has no deterministic test (a hook between listing and bind would be needed; LLR-z35vbh's dated amendment says so) |
| the org-io REQs and RCs of the plan header (REQ-bupq5y … REQ-fh5gsq; RC-pt49zn … RC-d6f25u), SDD-xwft92, SDD-aw3nka, SDD-egx8rv, SDD-4fvypt, SDD-rr7dtw, SDD-v2bq89, SDD-wr8m5n | through the LLRs above | see those rows |
| the org-node REQs of the plan header (REQ-x3jrh8, REQ-26mjqa, REQ-qthum6, REQ-2ugh2f, REQ-6ehufy, REQ-58tzex), REQ-uy7kfq, SDD-asxwp8, SDD-fysk96, SDD-esg2kc, SDD-bm27xt, SDD-xshv5b | through the LLRs above | see those rows |
| app REQ-q89tjr | none — an `expects: org-io` item opened by review round 1, finding-3 | not applicable: an open expectation (Gaps) |

inherited: the three `invitation.rs` LLR-b7wgpf tests (org-node) — from LLR-b7wgpf in app/src-tauri/tests/invitation.rs (moved, same ID, ruling A)
inherited: `the_store_is_opened_inside_the_data_dir_it_creates`, `a_data_dir_that_cannot_be_created_is_refused_naming_it` — from LLR-85zque in the app's state_assembly.rs (moved, same ID)
inherited: org-node `absences::only_forget_organisation_removes_records_and_personas` — from LLR-23sfdh (retired), deleted and replaced by the LLR-jd82t6 scan
inherited: the eleven transport tests T10 moved from org-node (`transport_handshake` ×7, `transport_networked` ×1, `endpoints` ×3) — from LLR-v873fx, LLR-wwunf4, LLR-wx77j5, LLR-6adc99, LLR-ygn78w, LLR-k2y6nn, LLR-2smrvx, LLR-fa7jt8, LLR-6zjzn2, LLR-cns6q6, LLR-ecz9a6 (moved, same IDs)
inherited: the `invitation_exchange` tests T13 moved from the app — from LLR-9sraks, LLR-f35pda, LLR-w4mhd4, LLR-rt8gdz, LLR-gha5f6 (moved, same IDs)
inherited: `every_real_terminal_message_stops_the_loop`, `a_non_terminal_failure_does_not_stop_the_loop`, `the_stop_is_announced_after_the_failure_for_the_same_error` — from LLR-a9rjtf (superseded by LLR-zw6jqy; both IDs carried, review round 1, finding-13)
inherited: the three earlier invite.personas tests — from LLR-rt8gdz (its webview half, now LLR-pms47e; review round 1, finding-14)
inherited: `forget_organisation_is_defined_once_and_called_only_from_accept_and_the_removal_step`, `a_commit_that_removes_another_member_forgets_nothing` — from LLR-23sfdh (→ LLR-jd82t6); each cites only the successor, because the old definition is deleted
inherited: `each_command_sends_once_to_one_device_and_the_kind_follows_the_record` — from LLR-q225ws (→ LLR-r9sh6k); cites only the successor (the old definition is deleted)
inherited: `produce_invite_reply_refuses_without_confirmation_over_ipc` — from LLR-w4mhd4 (→ app REQ-ab2mfz); cites only the successor (the old item moved to org-io)

## What was wrong, and what was built

What was wrong. org-node did IO: it held the iroh transport and its async
runtime, wrote its own store file under an Argon2 passphrase key, read its
device seed, and the app kept outstanding invites in a second, unencrypted
file. Its store save mutated memory before the write, so a failed save left
memory ahead of the disk (PR-jav8zn), and the write was not atomic
(PR-n696wv). The receive loop held the node lock while it waited, so every
command stalled behind it (PR-7v8anh), and a first admission went through a
second, self-deleting receive path (PR-kwwap5). A node with two Personas sent
every Organisation's messages under the first Persona's key (PR-8qsnhx). A
commit chose an Organisation-information body at send time, so a later commit
could change an earlier commit's key (PR-gnh3j2). An update went to one Device
only (PR-3ue4va), and the Loopback address was not checked against the
recipients (PR-2dmjzj). Found while building S4 and fixed in it: a closed
endpoint was kept and never rebound (PR-tcd2ta), a rebound endpoint took no
connections while the receiver ran (PR-47aycj), a Persona created while
receiving was never received on (PR-y6t5kt), a check that went stale before
its commit could accept a node's own relayed update as new (PR-erskh4), one
stalled peer could stop a Persona's endpoint receiving (PR-d2p5y2), the settle
could report an executed write as not executed (PR-9q5ems), and the
`DeviceSecret` lend could hand the seed back out (PR-aa7prn). S3b-io's
reconcile lost a late write's send when its save failed (PR-jgvf3b).

What was built. org-node takes values and returns values: `DeviceSecret` is a
move-only seed that leaves only through a scoped lend; each Persona's store is
sealed under an HKDF key of its own seed, and every mutating operation returns
one `StoreWrite` for org-io to persist while memory is adopted only once
sealed; the receive path is one value-driven `check_received` then
`commit_checked`, which refuses with `RecordMovedSinceCheck` when the record
moved in between; the commit returns every recipient's body; the Invite, the
reply and their bookkeeping live in the sealed store. org-io holds one OS
keychain item and one atomically written store file per Persona, one iroh
endpoint per Persona bound lazily from the keychain and rebound when closed,
a receive loop that holds no lock while it waits and reads each connection in
its own time-limited task, the acknowledgement-only acceptance of a revoked
Device with a verdict byte, the fan-out to every recipient with the Loopback
address check, the peer-address parse and the invitation exchange. iroh is
named only in `org-io/src/transport/`. The app depends on org-io alone, starts
with no passphrase, refuses when the keychain refuses, and stops its receive
loop on org-io's typed terminal condition. S3b-io part 1's settle and
reconcile run on S4's structure with the publish lock taken after the node
lock; no public discard method exists.

## Review

One block per finding the reviewers raised, verbatim, grouped by round. Each
round's own verdict line is quoted above its findings.

### Round 1

> Review round 1 (reviewer: independent subagent, fresh context; reviewed commit 4cf1f0f "review dispatched")
> verdict: Not ready to merge. Three medium findings (1, 2, 3) and the unruled T22 owner point (finding-4) need a fix or a ruling and another review round. The suite is green except the coverage commands, which fail here only because the review worktree copied the parent's target dir; org-io's own figures meet every floor.

**finding-1**: requirement, medium — LLR-d5y7ks's last sentence is not met: "no type reachable from the public API holds an `iroh::Endpoint` or an `OrgEndpoint`" (org-io/docs/architecture/2026-10-07-transport.md:100-101). In production builds `pub mod transport` (org-io/src/lib.rs:22) re-exports `OrgEndpoint` (transport/mod.rs:14), which holds `inner: iroh::Endpoint` (transport/endpoint.rs:68-71). Its public constructor `pub async fn bind_from_keychain` (endpoint.rs:99) lets any dependent crate bind an endpoint that authenticates as any Device with a keychain item, around `Endpoints`' one-open-endpoint rule (LLR-d2pzzk) and RC-pt49zn. `iroh_secret_key_is_named_in_one_function_only` (org-io/tests/absences.rs:872-929) checks only `pub` fields.
disposition: fixed (task `fa`) — the endpoint module is private, `OrgEndpoint`, `BindPlan` and `InboundReply` public only under `test-support` (`org-io/src/transport/mod.rs`, `org-io/src/transport/endpoint.rs`); new scan `the_endpoint_and_its_constructor_are_reachable_only_in_test_builds` (`org-io/tests/absences.rs`, LLR-d5y7ks), red on the unchanged tree.

**finding-2**: code, medium — A Persona created while the receiver loop runs never receives anything: `OrgIo::create_persona` (lib.rs:420-435) binds no endpoint and does not tell `Endpoints` to accept for the new key; the app's `start_receiver` returns early while the guard is held (commands.rs:459-461); `produce_reply` (exchange.rs:111-126) binds nothing; T21's rebind covers only endpoints a send binds. Admission sent to B's Device is never accepted. No test covers it.
disposition: fixed (task `fa`) — PR-y6t5kt filed and resolved: `Endpoints::accept_if_receiving` is called from `create_persona` (a bind failure is swallowed and the creation stands, accepted by the dispatcher) and from `produce_reply` (refuses with `ExchangeError::Bind`), `org-io/src/endpoints.rs`; tests `a_persona_created_while_the_receiver_runs_is_received_on` and `no_reply_is_produced_while_receiving_for_a_persona_whose_endpoint_cannot_be_bound` (`org-io/tests/endpoints.rs`, LLR-z35vbh, PR-y6t5kt), both red first.

**finding-3**: requirement, medium — REQ-x3c8n2 is declared "Met 2026-10-14" (app/docs/requirements/2026-09-14-tauri-shell.md:457-464), but only its terminal-versus-transient half is met; the 2026-09-14 widening (about lines 500-517) also requires the provider to say which Organisation a failed verification belongs to (REQ-affyf5, HAZ-9fmhm4's "unlocatable" half). events.rs:88 hard-codes `org_id: None`. Marked met, the open half is silently closed.
disposition: fixed (task `fc`) — REQ-x3c8n2's "Met" note narrowed to its terminal half by a dated clarification; new app expectation REQ-q89tjr (`expects: org-io`) carries the Organisation-of-a-failure half and is UNMET-EXPECTATION again; dated notes at HAZ-9fmhm4's residual and SDD-mwqf6x (`app/docs/requirements/2026-09-14-tauri-shell.md`, `app/docs/risk/2026-09-14-app-hazards.md`).

**finding-4**: record — The T22 report raises an owner point neither ruled nor filed (plan:5181-5184): a node behind the chain refuses (DeltaBaseMismatch) a valid update built on its own pending epoch; nothing recovers it.
disposition: filed as open problem report PR-m3tfnu (`org-io/docs/problems/2026-10-14-review-open.md`), per the owner's explanation that the decline itself is by design and what is missing is the catch-up; this record lists it under Gaps and opens, `docs/verification/2026-10-15-worktree-org-io-transport.md`.

**finding-5**: code, low — An invite stays outstanding after a timed-out admission that later commits: `admit_reply` returns at the `submit_commit_send` error before `settle` (exchange.rs:164); the late commit lands through settle_unknown/reconcile (publish.rs:294-298, 378-384), none of which settle the invite. Re-admitting is refused (`DuplicateKey`), contrary to REQ-prjja8. Needs an unknown write; nothing lost or admitted twice.
disposition: filed as open problem report PR-7hu3v5 (`org-io/docs/problems/2026-10-14-review-open.md`); not fixed in this change.

**finding-6**: requirement, low — REQ-n82q47 (org-io/docs/requirements/2026-10-06-transport.md:511) says "accept exactly the hexadecimal encoding … refuse any other input"; code (peer_addr.rs:68-72) and LLR-2b2ghe also accept blank as Ok(None) and one `0x`. REQ text not amended.
disposition: fixed — dated clarification of REQ-n82q47 (blank is no address; one leading `0x` is stripped), `org-io/docs/requirements/2026-10-06-transport.md`.

**finding-7**: requirement, low — REQ-x3jrh8 says ignoring the sealed bytes "is refused at compile time"; `unused_must_use = "deny"` catches only a discarded statement; `let _ = …` still compiles.
disposition: fixed — dated clarification of REQ-x3jrh8: a discarded statement is refused at compile time, `let _ =` is a review matter, `org-node/docs/requirements/2026-10-06-sans-io.md`.

**finding-8**: code, low — `a_device_secret_declares_zeroize_on_drop` (org-node/tests/key_custody.rs:210-214) checks only the hand-written marker trait (keys.rs:145); nothing checks the bytes are wiped. The pub-method scan (key_custody.rs:258-272) reads only the first `impl DeviceSecret` block (misses keys.rs:110); the trait-absence test (key_custody.rs:306-323) omits `Deserialize`.
disposition: fixed (task `fb`, `org-node/tests/key_custody.rs`) — the pub-method scan reads every `impl DeviceSecret` block; the trait probe adds `Deserialize`; new structural test `a_device_secrets_seed_is_held_in_zeroizing` (LLR-37k2s3), mutation-checked.

**finding-9**: code, low — Source-scan absence tests weaker than their LLRs: `org_node_is_sans_io` (org-node/tests/absences.rs:215-243) misses `use std::{fs,…}` and `std::env::var("X")`; `keychain_bytes_reach_no_clone_debug_or_serialise` (org-io/tests/absences.rs:820-836) scans only lines with `get_secret` or `seed`; `device_secret_is_named_only_in_its_sanctioned_places` (org-io/tests/absences.rs:121) still allows the deleted `device_secrets.rs`.
disposition: fixed (tasks `fa`, `fb`) — `org_node_is_sans_io` catches grouped `std::{fs, env}` imports and `env::var` reads, its self-check now inside it (`org-node/tests/absences.rs`, folded at TEST round 2); the keychain scan follows every copy of the keychain bytes and the `DeviceSecret` allowlist is narrowed (`org-io/tests/absences.rs`); each mutation-checked.

**finding-10**: code, low — `PersonaStore::data_mut` is `pub` (org-node/src/store.rs:917), changing a store without a `StoreWrite`; `SigningKeypair` derives `Clone` and is re-exported (keys.rs:21, lib.rs:46). Both predate S4.
disposition: filed as open problem reports PR-6gf4t2 and PR-gq83cs (`org-node/docs/problems/2026-10-14-review-open.md`); not fixed in this change.

**finding-11**: code, low — `stale_check.rs:49-59` never asserts the refusal calls no secret source (LLR-6qjmhf T22 amendment); no test covers a record that appeared or went between check and commit.
disposition: fixed (task `fb`, `org-node/tests/stale_check.rs`) — the refusals assert no secret source is called; new tests `a_record_that_appeared_since_the_check_is_refused_as_moved` and `a_record_that_went_since_the_check_is_refused_as_moved` (LLR-6qjmhf, PR-erskh4), mutation-checked.

**finding-12**: code, low — Missing abnormal cases: LLR-rdda86's two-store rule (service.rs:745-754) untested; LLR-2qqqx6 and LLR-gdfqn9 normal-case only; `a_target_without_a_native_backend_refuses_as_unavailable` (org-io/tests/keychain.rs:177-217) never checks `reason: "NoNativeBackend"`; LLR-h9tcat's unset `ODS_DATA_DIR` exit (bin/preflight.rs:36-41) untested.
disposition: fixed (tasks `fa`, `fb`), each test mutation-checked — `a_first_admission_commits_into_no_store_that_only_declared_or_is_only_listed` and `a_first_admission_commits_into_the_store_that_declared_and_is_listed` (`org-node/tests/expected_admission.rs`, LLR-rdda86); `a_store_sealed_under_any_other_key_derivation_is_refused` (`org-node/tests/sealed_store.rs`, LLR-2qqqx6); `a_refused_commit_returns_no_bodies_and_changes_nothing` (`org-node/tests/outgoing_bodies.rs`, LLR-gdfqn9); the `NoNativeBackend` reason asserted in `org-io/tests/keychain.rs`; `the_binary_without_a_data_dir_exits_non_zero_before_any_check` (`org-io/tests/preflight_transport.rs`, LLR-h9tcat). The over-`MAX_MESSAGE` body case is not built (it needs thousands of Members); review round 2, finding-9 filed it as PR-qpam4k.

**finding-13**: record — LLR-zw6jqy's lists cite `every_real_terminal_message_stops_the_loop`, `a_non_terminal_failure_does_not_stop_the_loop`, `the_stop_is_announced_after_the_failure_for_the_same_error`, rewritten in T16 for the typed rule yet still `verifies: LLR-a9rjtf` only (receiver_events.rs:646, 660, 696), T16 "inherited: none"; `terminal_is_decided_by_the_error_variant_not_its_text` (:956) carries LLR-a9rjtf although it asserts the opposite of its rule.
disposition: fixed (task `fc`) — the three tests verify LLR-zw6jqy beside LLR-a9rjtf and are listed as inherited in this record; LLR-a9rjtf removed from `terminal_is_decided_by_the_error_variant_not_its_text` (`app/src-tauri/tests/receiver_events.rs`); `docs/verification/2026-10-15-worktree-org-io-transport.md` lists them.

**finding-14**: record — LLR-pms47e lists four tests; three have no `verifies:` (invite.personas.test.ts); its CreateOrg clause has no gated test and is not marked untested.
disposition: fixed (task `fc`) — the three invite.personas tests cite LLR-pms47e (inherited from LLR-rt8gdz) and LLR-pms47e's founding clause is marked verified by review only (`app/tests/invite.personas.test.ts`, `app/docs/architecture/2026-10-06-invitation.md`); listed as inherited in `docs/verification/2026-10-15-worktree-org-io-transport.md`.

**finding-15**: record — Stale text: LLR-5wb4vd omits T23's test; LLR-yrb6th still says the restart test is ignored; PR-tcd2ta names a non-existent test; LLR-677fqq says keyring is named in keychain.rs only (test_support.rs too, sanctioned).
disposition: fixed — LLR-5wb4vd lists T23's `receiver_guard` test and LLR-yrb6th notes T20 un-ignored the restart test (`app/docs/architecture/2026-10-05-decomposition.md`); PR-tcd2ta names its tests (`org-io/docs/problems/2026-10-14-receiver-restart.md`); LLR-677fqq clarified (`org-io/docs/architecture/2026-10-07-transport.md`).

**finding-16**: record — Reproducing tests of resolved PR-n696wv and PR-jav8zn (store_files.rs:164, 181, 234) cite only LLRs.
disposition: fixed — the two PR-n696wv tests now carry `verifies: LLR-b9bm36, PR-n696wv` (`org-io/tests/store_files.rs`); PR-jav8zn is org-node's and cannot be cited from org-io, so its ledger gains a note naming the org-io test (`org-node/docs/problems/2026-10-08-save-after-mutate.md`).

**finding-17**: record — Plan T18 step 10 lists LLR-a9rjtf as retired; it is superseded (by LLR-zw6jqy). The record must say superseded.
disposition: fixed — corrected in the plan (`docs/plans/2026-10-06-org-io-transport.md`, T18 report, dated 2026-10-14); this record lists LLR-a9rjtf as superseded, not retired, under Gaps, `docs/verification/2026-10-15-worktree-org-io-transport.md`.

**finding-18**: record — No S4 verification record yet; it must cover findings 17 and 4, the accepted coverage gap, merge-note items 3 and 6.
disposition: fixed — this record, `docs/verification/2026-10-15-worktree-org-io-transport.md`, written at step 5: finding-17 and finding-4 above and under Gaps; the coverage gap under "Owner decisions and rulings" and Gaps; merge-note item 3 (REQ-by9k5q) and item 6 (S3b-io part 2's stand-ins) under "Merge notes from the S3b-io session".

**finding-19**: record — receive_step.rs:343 "B keeps its update" and :384 "B's own commit stands" are A's.
disposition: fixed — the assertion messages name A, `org-io/tests/receive_step.rs`.

**finding-20**: requirement, low — New ledger text uses "Device key" ~65 times; docs/CONTEXT.md:73 lists "device key" under Avoid. Predates this change.
disposition: no change — the term predates this change and the ledger sweep is outside it; recorded under Gaps. User-facing text and error messages were moved to the glossary terms by the ARCHITECTURE reviews (round 1 finding-3, round 2 findings 1–3).

### Round 2

> Review round 2 (reviewer: independent subagent, fresh context; reviewed commit 12e1147 "review dispatched")
> verdict: Not ready to merge. Three medium code findings need a fix or an owner ruling, then another round: finding-1 (stalled peer blocks receive, no hazard), finding-2 (public `remove_store`), and finding-3 (`lend_to_keychain` can return the seed). The suite is green apart from the expected coverage misreport in the copied worktree, and org-io's own figures meet every floor.

Owner ruling 2026-10-15 on this round's scope: fix the mediums and the
mechanical lows, file the larger lows as one umbrella org-io report
(PR-k5tdxj), and raise org-io's `problem_open_max` from 10 to 15. The
reviewer's supplementary report (its own read-only sub-reviewer) is recorded
after the numbered findings, as finding-15 onward with its labels; its A1
repeats finding-1, A2 finding-6, A4 finding-2 (plus `read_store` public), A5
the open PR-7hu3v5, and A9 findings 5 and 13.

**finding-1**: code, medium — One peer can stop a Persona's endpoint from receiving anything, and no hazard records it. Each endpoint has one accept task handling one connection at a time with no timeout (org-io/src/endpoints.rs:220-236); recv_answerable (transport/endpoint.rs:306-312) awaits accept, handshake, accept_bi, read_to_end. A peer that completes the handshake and never opens or finishes a stream blocks the endpoint indefinitely. LLR-wvgec4 accepts every handshake. HAZ-4n7pss and RC-5wz559 (org-io/docs/risk/2026-10-06-transport.md:97-123) cover memory and floods only. Same shape was in org-node's transport on master; needs a problem report, a hazard entry, then a per-connection task or timeout.
disposition: fixed (task `ra`) — PR-d2p5y2 filed and resolved: each inbound connection is read in its own task, bounded by `CONNECTION_TIME_LIMIT` (30 s) and `CONNECTIONS_AT_ONCE` (16) per endpoint (`org-io/src/endpoints.rs`, `org-io/src/transport/endpoint.rs`); new HAZ-4hj5m9, RC-96w7nf, REQ-mnd9rj, LLR-8uf9a5 (`org-io/docs/risk/2026-10-06-transport.md`); tests `a_stalled_connection_does_not_stop_another_peers_message`, `a_stalled_connection_is_dropped_after_the_limit`, `a_seventeenth_connection_waits_for_a_slot_while_sixteen_stall` (`org-io/tests/endpoints.rs`). HAZ-4hj5m9's S3/P1 residual accepted by the owner 2026-10-15.

**finding-2**: code, medium — `pub mod store_file` (org-io/src/lib.rs:19) exports `pub fn remove_store(data_dir, device_key)` (store_file.rs:161) and `write_store`; the app can delete a store directly, against REQ-cz753a and LLR-cc59ys. Scans miss it (absences.rs:1159, :1177-1188). Make both pub(crate).
disposition: fixed (task `rb`) — `read_store`, `write_store` and `remove_store` are `pub(crate)` (`org-io/src/store_file.rs`); new scan `no_public_function_writes_or_removes_a_file` (`org-io/tests/absences.rs`, LLR-cc59ys) fails on a public file-writing function.

**finding-3**: code, medium — `DeviceSecret::lend_to_keychain<R>(&self, write: impl FnOnce(&[u8; 32]) -> R) -> R` is pub (org-node/src/keys.rs:105); with R generic, `|s| *s` returns the seed unzeroised. key_custody.rs:291-298 does exactly that. Breaks LLR-37k2s3 and REQ-26mjqa's scoped lend; LLR-yp69jg's signature permits it. The pub-method scan (key_custody.rs:304-322) cannot see it. Bound R to a non-secret type (e.g. a Result of () and an error).
disposition: fixed (task `rc`) — PR-aa7prn filed and resolved: `lend_to_keychain<E: std::error::Error>(&self, write: impl FnOnce(&[u8; 32]) -> Result<(), E>) -> Result<(), E>` (`org-node/src/keys.rs`); LLR-yp69jg amended, LLR-37k2s3 clarified; test `lend_to_keychain_returns_no_value_the_writer_could_carry_the_bytes_in` (`org-node/tests/key_custody.rs`); helpers and org-io's keychain test no longer copy the seed out through the lend's return. Round 3, finding-1 narrowed what the amendment claims.

**finding-4**: code, low — keyring calls run synchronously from async code on tokio worker threads, no spawn_blocking (create_persona's keep, commit/reconcile secret closures, receive step, bind_from_keychain), against the keyring SOUP row (soup.md:37); nothing serialises item access (Windows Credential Manager).
disposition: booked in the umbrella open report PR-k5tdxj, item 1 (`org-io/docs/problems/2026-10-15-review-round-2.md`), by the owner's scope ruling; not fixed in this change.

**finding-5**: code, low — transport/mod.rs:7-8 and the LLR-d5y7ks scan comment say "nothing outside Endpoints binds one", but `pub async fn preflight::check_transport` (preflight.rs:88-90) calls bind_from_keychain in production; any dependent can bring a second endpoint for a Device online.
disposition: fixed (task `ra`) — the transport docs and the LLR-d5y7ks scan comment name the preflight's bind (LLR-h9tcat), `org-io/src/transport/mod.rs`, `org-io/tests/absences.rs`, `org-io/docs/architecture/2026-10-07-transport.md`.

**finding-6**: code, low — reconcile sends only when an in-memory unknown write is recorded (publish.rs:364-383): bodies of an executed write never delivered after a restart; after a read-back/commit/store failure in submit_commit_send (submit.rs:315-325); when org-node's reconcile refuses an executed unknown write (Err(Node), cleared :299, :383); when a late commit removes this node. PR-vt244s / S3b-io part 2 (T16d) appear to cover it; REQ-by9k5q's note says the startup reconcile "settles it".
disposition: booked in the umbrella open report PR-k5tdxj, item 2 (`org-io/docs/problems/2026-10-15-review-round-2.md`); REQ-by9k5q's "settles it" note clarified the same day (`org-io/docs/requirements/2026-10-13-publish.md`); PR-vt244s stays open for S3b-io part 2.

**finding-7**: code, low — test gaps: LLR-szhe7j (Connection never driven through next_received; Chain vs Refused merged at receive_order.rs:95); LLR-zw6jqy (no test feeds ReceiveError::Store/Keychain to the app's non-terminal arm, events.rs:202); SDD-v2bq89 (`plan_send`, send.rs:126, untested at its interface).
disposition: fixed in part (tasks `ra`, `rc`) — `a_connection_failing_before_a_body_is_not_terminal` (`org-io/tests/receive_step.rs`, LLR-szhe7j) drives `ReceiveError::Connection` through `next_received`, and `org-io/tests/receive_order.rs` asserts Refused and Chain apart; `store_and_keychain_failures_are_one_receiver_error_and_do_not_stop_the_loop` (`app/src-tauri/tests/receiver_events.rs`, LLR-zw6jqy), mutation-checked. `plan_send` at its interface is booked in PR-k5tdxj, item 5.

**finding-8**: code, low — `the_os_keychain_keeps_the_credential_contract` (keychain_contract.rs:120-126) returns early "ok" unless ODS_REAL_KEYCHAIN_TESTS=1 — should be #[ignore]; `commit_genesis_returns_no_recipients` (org-node outgoing_bodies.rs:151-161) checks Debug text for fields ReceiveOutcome lacks — passes by construction.
disposition: fixed (tasks `rb`, `rc`) — the real-keychain contract test is `#[ignore]`d with its reason (`org-io/tests/keychain_contract.rs`); `commit_genesis_returns_no_recipients` reads the real outcome (`org-node/tests/outgoing_bodies.rs`).

**finding-9**: code, low — the outgoing-body bound (encode_message, MAX_MESSAGE) refuses inside commit_update/reconcile (org-node service.rs:1453-1468), after org-io has written the chain; unmarked derived behaviour, no abnormal test (REQ-6ehufy, LLR-gdfqn9).
disposition: filed as open problem report PR-qpam4k (`org-node/docs/problems/2026-10-15-review-round2.md`); not fixed in this change.

**finding-10**: requirement, low — LLR-yp69jg says `field: "device secret"`; code and test use "Device seed" (keys.rs:89-90; key_custody.rs:281).
disposition: fixed (task `rc`) — LLR-yp69jg's field name clarified to "Device seed", `org-node/docs/architecture/2026-10-07-sans-io.md`.

**finding-11**: requirement, low — LLR-b7wgpf (a Blob that parses encodes back to the same text) false for some inputs: unarmour trims whitespace, postcard::from_bytes ignores trailing bytes (invitation.rs:88-94; wire.rs uses take_from_bytes and refuses leftovers); an Invite with no inviter Device keys and unbounded name lengths accepted without an LLR.
disposition: fixed in the ledger (task `rc`) — LLR-b7wgpf clarified; the trailing bytes and unbounded Invite fields filed as open problem report PR-bjaq7j (`org-node/docs/problems/2026-10-15-review-round2.md`).

**finding-12**: record — the keyring SOUP row says the issue tracker "is to be checked before merge" (soup.md:37; plan T7 step 3); nothing records that review.
disposition: fixed (task `rb`) — the keyring tracker was reviewed 2026-10-15 and the SOUP row records it: only issue #163 (Windows concurrency) bears on org-io, carried in PR-k5tdxj, `org-io/docs/architecture/soup.md`.

**finding-13**: record — stale text: app events.rs:54-58 (retired receive_and_self_delete_if_revoked); app policy.rs:12 (org-node transport::TransportMode); org-io tests/open.rs:2-7 (persona_store.bin, passphrase); transport.md:1338 (handle takes no peer address).
disposition: fixed (tasks `rb`, `rc`) — `app/src-tauri/src/events.rs` and `app/src-tauri/src/policy.rs` name the current receive path and org-io's `TransportMode`; `org-io/tests/open.rs` and `org-io/docs/architecture/2026-10-07-transport.md` corrected.

**finding-14**: record — plan "conformance r2" row calls four tests "inherited" though they cite only the successor (old definitions deleted); invite.personas.test.ts says inherited "from the one-Persona-one-Organisation item" without naming LLR-rt8gdz.
disposition: fixed — the plan's row says each now cites only the successor (`docs/plans/2026-10-06-org-io-transport.md`); `app/tests/invite.personas.test.ts` names org-io's moved item in prose (task `rc`); this record lists the four under `inherited:` with that note, `docs/verification/2026-10-15-worktree-org-io-transport.md`.

**finding-15**: code, medium — supplementary report, A3: settle case (b) (publish.rs:242-249) can report an executed write as not executed when the record jumps past e+1 via a received later epoch → NOT_EXECUTED_RESUBMITTED for an update already on chain.
disposition: fixed (task `rb`) — PR-9q5ems filed and resolved: case (b) settles at once only when the record is at the epoch after the expected one with another root and Organisation public key, and otherwise reads the chain (`org-io/src/publish.rs`; LLR-t98pk9 and REQ-by9k5q amended); tests `a_settle_whose_record_holds_its_executed_update_does_not_report_it_not_executed`, `a_settle_after_a_later_epoch_was_committed_does_not_report_its_executed_write_not_executed` (`org-io/tests/publish_path.rs`). It restores LLR-t98pk9's and REQ-by9k5q's own clause; offered to the owner for confirmation (see "Merge notes from the S3b-io session").

**finding-16**: code, low — supplementary report, A6: race start_receiving vs create_persona (receiver.rs:91 lists then endpoints.rs:135 sets receiving — set the flag first).
disposition: fixed (task `ra`) — `start_receiving` marks the node receiving before it lists the Personas (`Endpoints::mark_receiving`, `org-io/src/endpoints.rs`, `org-io/src/receiver.rs`).

**finding-17**: code, low — supplementary report, A7: a closing (not closed) endpoint reused.
disposition: fixed (task `ra`) — `OrgEndpoint::is_closed` counts a closing endpoint as closed (`org-io/src/transport/endpoint.rs`); test `a_closing_endpoint_is_not_handed_out_again` (`org-io/tests/endpoints.rs`, LLR-d2pzzk).

**finding-18**: code, low — supplementary report, A8, B(e) and C: unknown recorded only on TimedOut (predates S4); allowlist scan shallow (fields of allowlisted types unchecked); test gaps (LLR-znhmh2 random failing position, plan_send, LLR-murjr6 call sites, LLR-b9bm36 fsync/create_new mutations, peer-address parse with transport addresses, LLR-szhe7j Connection, LLR-z35vbh hidden by test hooks, LLR-t98pk9 read order, LLR-nqvbb6 Reconciled(_) branch, and more lows).
disposition: booked in the umbrella open report PR-k5tdxj, items 3–5 (`org-io/docs/problems/2026-10-15-review-round-2.md`), by the owner's scope ruling; the LLR-szhe7j Connection gap was fixed under finding-7.

### Round 3

> Review round 3 (reviewer: independent subagent, fresh context; reviewed commit e334e82 "review dispatched")
> verdict: Ready to merge. All three suites are green, every round-2 disposition holds in the tree, and the only new findings are one low and one record.

**finding-1**: requirement, low — DeviceSecret::lend_to_keychain (org-node/src/keys.rs:103-112) and LLR-yp69jg's 2026-10-15 amendment (org-node/docs/architecture/2026-10-07-sans-io.md:213-220) claim an error bound to std::error::Error carries no value out; a caller-defined `struct Carrier([u8; 32]); impl Error for Carrier` can return Err(Carrier(*seed)). The guarding test (org-node/tests/key_custody.rs:321) compares signature text only. The only production caller (org-io keychain.rs:135) returns keyring's error, so nothing leaks today; the guarantee rests on review, as LLR-37k2s3's clarification already says for closure capture. Reword the amendment, or bound E to a non-generic error type.
disposition: fixed (task `fz`) — LLR-yp69jg's amendment reworded: the bound stops a byte array, not a caller-defined error type wrapping one; the remaining guarantee rests on review, with the one production caller named (`org-node/docs/architecture/2026-10-07-sans-io.md`, `org-io/src/keychain.rs`). No code change.

**finding-2**: record — the A6 amendment (LLR-z35vbh) and the A7 amendment (LLR-d2pzzk) sit after LLR-8uf9a5's block (org-io/docs/architecture/2026-10-07-transport.md:213, :223), so find-items.sh show attributes them to LLR-8uf9a5.
disposition: fixed (task `fz`) — both amendments moved under their own items in `org-io/docs/architecture/2026-10-07-transport.md`; recorded in `docs/verification/2026-10-15-worktree-org-io-transport.md`.

## Guideline reviews

One review per guidelines file `guidelines-file.sh` printed for the change diff,
each with its findings verbatim and a disposition for every one. Every path of
the diff resolves to the installed defaults under `.guardrails/templates/`:
no unit or root guidelines file exists. Before these reviews, a deslop pass
(plan, "Deslop pass, 2026-10-14", under guardrails 0.5.1, whose
`guidelines-file.sh` was absent) reviewed the scripted blocks of
`org-node/src/service.rs` and the test support with no behaviour change; its
one behaviour finding became T22 (PR-erskh4).

### .guardrails/templates/TEST_GUIDELINES.md

Governs: the test files the change adds or modifies in org-io, org-node and
the app (`org-io/tests/`, `org-node/tests/`, `app/src-tauri/tests/`,
`app/tests/`), as `master...worktree-org-io-transport` lists them at each
round.

### Round 1

> TEST conformance review round 1 (review-conformance, .guardrails/templates/TEST_GUIDELINES.md), over master...worktree-org-io-transport at 20ae01c; 71 test files.
> verdict: 12 findings (0 high, 5 medium, 7 low); 5 are above low (findings 1–5).

**finding-1**: guideline, medium — Doubles 3 — MemoryKeychain (org-io/src/test_support.rs:405-500) has no contract test running the same assertions against the real OS keychain (org-io/tests/keychain.rs:2-7).
disposition: fixed (task `ct`) — `org-io/tests/keychain_contract.rs`: `the_in_memory_keychain_keeps_the_credential_contract` and `the_os_keychain_keeps_the_credential_contract` run one contract against both. The owner's real-keychain run in a normal terminal, 2026-10-15: 2 passed, 0 failed.

**finding-2**: guideline, medium — Owned services 1 — new REQ-level normal-case org-io tests use FakeChain in place of the contained on-chain contract (expectations_met.rs:110, :342, :409; node_stories.rs:53; invitation_exchange, send_fanout, receive_step).
disposition: no change — owner ruling 2026-10-14: the chain is a boundary for org-io's tests (an external network reached through on-chain-client); FakeChain stays, and on-chain-client's end-to-end tests cover the real contract.

**finding-3**: guideline, medium — Seams 4 — node_stories::five_stories_full_e2e walks the Invite exchange but does not cite REQ-prjja8, REQ-tcutr6, REQ-65xqp8.
disposition: fixed (task `ct`) — `five_stories_full_e2e` cites REQ-prjja8, REQ-tcutr6 and REQ-65xqp8, `org-io/tests/node_stories.rs`.

**finding-4**: guideline, medium — Seams 1 / Existing suites 1 — org-io/tests/peer_addr.rs:70-139: seven tests moved from app ipc.rs verify LLR-ty85xv, LLR-n6twt7, LLR-ctrfz4 (revoke_member/admit_member parsing) but call only parse_peer_addr.
disposition: fixed (task `c4`) — LLR-ty85xv, LLR-n6twt7 and LLR-ctrfz4 moved back to the app under SDD-2pa6h6, IDs kept; seven IPC tests in `app/src-tauri/tests/ipc.rs` (`revoke_member_treats_a_blank_or_whitespace_only_peer_addr_as_absent`, `revoke_member_does_not_refuse_an_encoded_endpoint_addr`, `revoke_member_refuses_a_peer_addr_that_is_not_hex`, `revoke_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr`, `admit_member_does_not_refuse_a_blank_or_decodable_peer_addr`, `admit_member_refuses_a_peer_addr_that_is_not_hex`, `admit_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr`), each mutation-checked; the org-io copies removed.

**finding-5**: guideline, medium — Existing suites 1 / Seams 1 — org-node store_at_rest.rs:40-41, :172-177; provisional_store.rs:75-81 repaired in place onto test hooks store_for_test / seal_for_test instead of moving to the StoreWrite seam.
disposition: fixed (task `ct`) — both tests read an operation's `StoreWrite`, `org-node/tests/store_at_rest.rs`, `org-node/tests/provisional_store.rs`.

**finding-6**: guideline, low — Seams 2 — endpoints.rs:358-372 drives the send-path bind through endpoint_for_test, not submit_commit_send.
disposition: fixed in part (task `ct`) — the refused send-path rebind is driven through `submit_commit_send`, `org-io/tests/endpoints.rs`; the rebind half completed at round 2, finding-3.

**finding-7**: guideline, low — floor — org-node absences.rs:267 helper-on-strings test cites REQ-x3jrh8 but cannot fail if org-node does IO.
disposition: fixed (task `ct`, then round 2) — the helper's self-check runs inside `org_node_is_sans_io` (`org-node/tests/absences.rs`).

**finding-8**: guideline, low — floor — org-node invitation.rs:134 a_reply_that_does_not_parse_acts_on_nothing: unchanged-store assertions cannot fail.
disposition: fixed (task `ct`) — the assertions that could not fail removed, `org-node/tests/invitation.rs`.

**finding-9**: guideline, low — floor — org-node outgoing_bodies.rs:303, :322 "nothing written" checks bypass the harness disk.
disposition: fixed (task `ct`) — the "nothing written" assertions that could not fail removed, `org-node/tests/outgoing_bodies.rs`.

**finding-10**: guideline, low — Seams 1 — org-node receive_path.rs:419 no_receive_branch_is_unreachable pins a constructor's file.
disposition: fixed in part (task `ct`) — the test observes the refusal through `check_received` (`org-node/tests/receive_path.rs`); its remaining source scan dropped at round 2, finding-5, and renamed `an_unheld_organisations_notice_is_refused_as_not_held` at round 3.

**finding-11**: guideline, low — Seams 1 — org-node absences.rs:486 asserts exact statement text inside adopt / drop_store.
disposition: fixed (task `ct`) — the store-removal scan checks the rule, not the statements of `adopt` and `drop_store`, `org-node/tests/absences.rs`.

**finding-12**: guideline, low — floor — app receiver_events.rs:649, :664, :701 keep LLR-a9rjtf (superseded) beside LLR-zw6jqy.
disposition: no change — the upgraded guardrails rule directs a re-pointed test to keep the old ID beside its successor and to be listed as inherited; the live LLR-zw6jqy is what these tests verify, and this record lists them as inherited.

### Round 2

> TEST conformance review round 2 (review-conformance, .guardrails/templates/TEST_GUIDELINES.md), at a941d1c, 75 test files.
> verdict: 5 findings (0 high, 2 medium, 3 low); 2 are above low (finding-1 is new, from the finding-7 fix; finding-2 was missed by round 1).

**finding-1**: guideline, medium — the floor ("every test carries `verifies:`") — org-node/tests/absences.rs:271 — the finding-7 fix left the_sans_io_scan_catches_grouped_file_imports_and_environment_reads with no verifies: line.
disposition: fixed (task `ct2`) — the grouped-import self-check runs inside `org_node_is_sans_io` (verifies LLR-4tcxsu, REQ-x3jrh8) and the separate unannotated test is gone, `org-node/tests/absences.rs`; mutation: a helper ignoring grouped fs/env items failed `org_node_is_sans_io`.

**finding-2**: guideline, medium — the floor, and the upgraded rule (a re-pointed test keeps the old ID beside its successor) — app/src-tauri/tests/ipc.rs:915, org-node/tests/absences.rs:568, org-node/tests/service_stories.rs:601 — the change removed the only ID from three running tests (produce_invite_reply_refuses_without_confirmation_over_ipc had LLR-w4mhd4; forget_organisation_is_defined_once_and_called_only_from_accept_and_the_removal_step and a_commit_that_removes_another_member_forgets_nothing had LLR-23sfdh).
disposition: fixed (task `ct2`, extended to every test of the change left with no `verifies:`) — the tests point at the item they check, with dated comments: the two org-node tests at LLR-jd82t6, `each_command_sends_once_to_one_device_and_the_kind_follows_the_record` at LLR-r9sh6k, `produce_invite_reply_refuses_without_confirmation_over_ipc` at REQ-ab2mfz (`org-node/tests/absences.rs`, `org-node/tests/service_stories.rs`, `org-io/tests/submit_flow.rs`, `app/src-tauri/tests/ipc.rs`); each cites only its successor because the old definition is deleted or moved, listed as inherited in this record. Owner rulings 2026-10-15 for the two no item stated: the app's `init_opens_the_store_once_even_when_the_connect_is_refused` deleted with `state_assembly.rs`; LLR-nqdr97 minted for `a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing` (task `ct3`).

**finding-3**: guideline, low — Seams 2 — org-io/tests/endpoints.rs:392 — the rebind half still goes through endpoint_for_test, not a send.
disposition: fixed (task `ct2`) — the rebind after a refused bind is driven through the next `submit_commit_send`, `org-io/tests/endpoints.rs`; mutation: the send remembering a refused bind failed it.

**finding-4**: guideline, low — Doubles 5 / Doubles 3 — org-io/src/test_support.rs:388-389, org-io/tests/keychain_contract.rs:327-368 — Refusal::Locked/Unavailable imitate keyring's NoStorageAccess/PlatformFailure; the contract has no (even skipped) assertion that a locked native store answers NoStorageAccess.
disposition: fixed (task `ct2`) — `the_in_memory_keychain_refuses_as_a_locked_store_does` and `a_locked_os_keychain_refuses_as_no_storage_access` (`org-io/tests/keychain_contract.rs`); the native run is `#[ignore]`d with its reason (a locked keychain is not reachable); mutation: the fake answering PlatformFailure failed it.

**finding-5**: guideline, low — Seams 1 — org-node/tests/receive_path.rs:737-738 — the first assertion still scans service.rs for `RevocationNotHeld {`; its mutation check adds dead code.
disposition: fixed (task `ct2`) — the source scan dropped and the comment fixed, `org-node/tests/receive_path.rs`; mutation: `notice_store` starting from `RevocationNotForThisDevice` failed it.

### Round 3

> TEST conformance review round 3 (review-conformance, .guardrails/templates/TEST_GUIDELINES.md), at b15ca6c, 66 test files added/modified + 5 deleted.
> verdict: 1 finding (0 high, 0 medium, 1 low); none is above low.

**finding-1**: guideline, low — floor ("every test fails when the behavior it `verifies:` breaks") — org-io/tests/keychain_contract.rs:161, :180 — the two locked-store contract tests cite LLR-8erfbu (org-io's NoStorageAccess → Locked mapping) but run no org-io code; they check the fake's fidelity (LLR-36jer8). The mapping keeps its own tests (keychain.rs:114-175).
disposition: fixed (task `ct4`) — the two tests cite LLR-36jer8 and the `locked_contract` doc is corrected, `org-io/tests/keychain_contract.rs`; the round's notes fixed with it (LLR-nqdr97's abnormal test asserts org-node's `IdNotFound` refusal, mutation-checked; `an_unheld_organisations_notice_is_refused_as_not_held` renamed; LLR-h9tcat's two refusals clarified).

### .guardrails/templates/CODE_GUIDELINES.md

Governs: the production source files the change adds or modifies in org-io,
org-node and the app (`org-io/src/`, `org-node/src/`, `app/src-tauri/src/`,
`app/src/`).

### Round 1

> CODE conformance review round 1 (review-conformance, .guardrails/templates/CODE_GUIDELINES.md), over master...worktree-org-io-transport at 20ae01c; 45 source files +4742/-2536.
> verdict: 12 findings — 0 high, 2 medium (finding-1 change narration, finding-2 discarded late-commit send failures), 10 low; findings above low exist, so the change does not yet conform.

**finding-1**: guideline, medium — Comments 2 — ~128 added comment lines carry change history (S4 task/round/date narration), incl. tombstones app/src-tauri/src/policy.rs:61, state.rs:56.
disposition: fixed (task `cf`, no behaviour change) — comments describe the code, not S4's task, round or reconcile history; the tombstones in `app/src-tauri/src/policy.rs` and `app/src-tauri/src/state.rs` deleted. Residual requirement-history phrases were round 2, finding-3.

**finding-2**: guideline, medium — Errors 2 — org-io/src/publish.rs:383 `let _ = send_late_commit(...)` discards per-Device send failures; `Reconciliation::Committed { recipients }` reads as delivered.
disposition: fixed in part (task `cf`) — the failures are held under a named binding whose comment points to S3b-io part 2's `SendReport`, and `Reconciliation::Committed`'s doc says failures are not reported (`org-io/src/publish.rs`); reporting them changes S3b-io's public type, so it is tracked as open PR-b8kmk2 until S3b-io part 2.

**finding-3**: guideline, low — Errors 2 / Errors 1 — org-io/src/keychain.rs:196 `Keychain::for_signing` drops each KeychainError (submit and reconcile removal paths).
disposition: filed as open problem report PR-43t2zn (`org-io/docs/problems/2026-10-14-conformance-open.md`): the fix changes behaviour, outside a no-behaviour-change refactor.

**finding-4**: guideline, low — Errors 1 — org-node store key derivation / store encode / Blob encode report OrgNodeError::Chain; build_invite on an unbound Organisation returns OrgNotOnChain (store.rs:813, :931; invitation.rs:100; service.rs:895).
disposition: filed as open problem report PR-u784zs (`org-node/docs/problems/2026-10-14-conformance-open.md`): a behaviour change.

**finding-5**: guideline, low — Errors 1 — unparseable endpoint identity reported as TransportError::Malformed (endpoint.rs:291; send.rs:107).
disposition: filed as open problem report PR-vy83sd (`org-io/docs/problems/2026-10-14-conformance-open.md`): a behaviour change.

**finding-6**: guideline, low — Errors 3 / Errors 2 — PeerAddr::to_hex unwrap_or_default() → "" parses back as no address (peer_addr.rs:25).
disposition: filed as open problem report PR-wayqj3 (`org-io/docs/problems/2026-10-14-conformance-open.md`): a behaviour change.

**finding-7**: guideline, low — Errors 2 — app commands.rs:476 `let _ = emit(...)` without a reason.
disposition: fixed (task `cf`) — the start-up emit says why a failed emit is not reported, `app/src-tauri/src/commands.rs`.

**finding-8**: guideline, low — Errors 3 — preflight ODS_TRANSPORT: any value other than loopback silently becomes Networked (bin/preflight.rs:45).
disposition: filed as open problem report PR-a63sk3 (`org-io/docs/problems/2026-10-14-conformance-open.md`): a behaviour change.

**finding-9**: guideline, low — Names 1 — single-letter names on new lines (commands.rs, invitation.rs, store.rs, service.rs).
disposition: fixed in part (task `cf`) — closure and loop names spelled out in `app/src-tauri/src/commands.rs`, `org-node/src/service.rs`, `org-node/src/store.rs`, `org-node/src/invitation.rs`; the rest at round 2, finding-1.

**finding-10**: guideline, low — Names 4 — org-io's service named node/svc/service; keychain named keychain/device_secrets.
disposition: fixed (task `cf`) — org-io names `OrgService` `service` and the `Keychain` `keychain` throughout its source, `org-io/src/lib.rs`, `org-io/src/publish.rs`, `org-io/src/submit.rs`.

**finding-11**: guideline, low — Simplicity 4 — report_sends (app) and send_late_commit (org-io) both walk SendReport.per_device, rendering keys differently.
disposition: fixed (task `cf`) — both use `SendReport::failure_summary`, `org-io/src/send.rs`, `app/src-tauri/src/commands.rs`.

**finding-12**: guideline, low — Names 2 — org-node/src/revocation.rs:222 iter::once(...).filter as an if.
disposition: fixed (task `cf`) — a plain `if`, `org-node/src/revocation.rs`.

### Round 2

> CODE conformance review round 2 (review-conformance, .guardrails/templates/CODE_GUIDELINES.md), master...worktree-org-io-transport at a941d1c; production source only (45 files, +4758/-2576).
> verdict: 9 findings — 0 high, 0 medium, 9 low; none above low.

**finding-1**: guideline, low — Names 1, "Every name the change writes, a loop index and a callback parameter included, says what it holds" — org-node/src/service.rs:296, 348, 365, 1133, 1146, 1165 (`|u|`, `|o|`); org-node/src/keys.rs:138 (`f: &mut fmt::Formatter`, where the change elsewhere writes `formatter`); app/src-tauri/src/state.rs:115, 193, 208 (`|e|`) — lines this change rewrote still carry one-letter names; round-1 finding-9's fix did not reach them.
disposition: fixed (task `cf2`) — the names spelled out in `org-node/src/service.rs`, `org-node/src/keys.rs`, `app/src-tauri/src/state.rs`.

**finding-2**: guideline, low — Comments 2, "A comment describes the code as it is" — org-io/src/transport/endpoint.rs:188 — "Production callers get the three things they read from it, below" is now followed by four production methods (addr, bound_sockets, is_closed, online); `is_closed` (:205) was added by this change and the count was carried over unchanged.
disposition: fixed (task `cf2`) — the comment counts its four production readers, `org-io/src/transport/endpoint.rs`.

**finding-3**: guideline, low — Comments 2, "never the change that wrote it: no ... 'previously', 'fixed' or ticket history" — org-node/src/invitation.rs:20 "(LLR-ms8njy as narrowed)"; org-node/src/service.rs:757 "(LLR-d9778a as amended 2026-10-07)"; org-node/src/service.rs:987 "note of 2026-10-13"; org-node/src/store.rs:62 and :600 "closed 2026-10-08" — record the history of requirements and rulings, not the code's reason.
disposition: fixed (task `cf2`) — requirement history dropped from the comments, IDs kept, `org-node/src/invitation.rs`, `org-node/src/service.rs`, `org-node/src/store.rs`.

**finding-4**: guideline, low — Names 4, "A concept has one name in every file, test and task of the change" — org-io/src/peer_addr.rs:54 (and :40, :55) — the doc says each refusal "names the field, `peer_addr`", the Decode refusal says `peer_addr`, the Hex refusal says `peer_addr_blob`: one field, two names in one enum.
disposition: fixed (task `cf2`) — `PeerAddrError`'s doc names what each refusal names; the file now lives at `org-io/src/transport/peer_addr.rs` (ARCHITECTURE round 1, finding-1).

**finding-5**: guideline, low — Names 4, "A concept has one name in every file ... and where the codebase already names it, that name" — app/src-tauri/src/commands.rs:159, 175, 191, 229, 283, 332, 341, 348, 368, 414, 456 — the shared OrgIo handle is `org_io` in AppState and state.rs but `io` in every command and in `reconcile_all`/`next_outcomes`.
disposition: fixed (task `cf2`) — the shared handle is `org_io` in every app command, `app/src-tauri/src/commands.rs`.

**finding-6**: guideline, low — Errors 2, "Where an error is expected and handled, the handling says why that is safe" — org-io/src/store_file.rs:138 — `let _ = std::fs::remove_file(&temporary)` drops the cleanup failure; the doc (:122) says only "where it can", not why a leftover `.tmp-` file is harmless (list_stores ignores it).
disposition: fixed (task `cf2`) — the comment says why a leftover `.tmp-` file is harmless, `org-io/src/store_file.rs`.

**finding-7**: guideline, low — Errors 2, "no error output sent away where the caller needs it" — org-io/src/exchange.rs:181 — when the committed admission's Member is not found in the record, `admit_reply` returns `ExchangeError::Submit` and drops `submitted.sends` (:176), so the caller loses the per-Device send failures of an update that did commit (rare state).
disposition: fixed (task `cf2`) — the refusal appends `SendReport::failure_summary()` to its message, `org-io/src/exchange.rs`; no test, the state is not reachable through the public API without faking project code, and the reason is recorded at the code.

**finding-8**: guideline, low — Errors 1, "An error is reported where it is detected, naming what failed" — org-io/src/bin/preflight.rs:39 — `Err(_)` reports a set but non-Unicode ODS_DATA_DIR as "ODS_DATA_DIR not set".
disposition: fixed (task `cf2`) — the binary reports "ODS_DATA_DIR is not valid Unicode", `org-io/src/bin/preflight.rs`; Unix test `the_binary_with_a_non_unicode_data_dir_says_so_and_exits_before_any_check` (`org-io/tests/preflight_transport.rs`, LLR-h9tcat), red before the fix.

**finding-9**: guideline, low — Simplicity 2, "No parameter, flag, setting or hook without a caller that uses it now" — org-io/src/transport/mod.rs:33, :36 — `TransportMode` derives `Default` with `#[default] Loopback`; nothing in org-io, its tests or the app calls it (moved over from org-node unchanged).
disposition: fixed (task `cf2`) — `TransportMode` no longer derives an unused `Default`, `org-io/src/transport/mod.rs`.

### .guardrails/templates/ARCHITECTURE_GUIDELINES.md

Governs: the change's paths in org-io, org-node and the app, reviewed after
master's guardrails `344faa3` added this kind (2026-10-15).

### Round 1

> ARCHITECTURE conformance review round 1 (review-conformance, .guardrails/templates/ARCHITECTURE_GUIDELINES.md), over master...worktree-org-io-transport after the 344faa3 merge.
> verdict: 8 findings (0 high, 5 medium, 3 low); findings 1-5 are above low.

Owner ruling 2026-10-15: fix every finding above low now. The reviewer's
note — org-io passes `OrgNodeError` to the app through `org_io::node`
(Boundaries 2) where the plan's §17.2 rule 3, an owner ruling, keeps it — is
recorded here for the owner; the tree follows the ruling.

**finding-1**: guideline, medium — Core and edges 4 — iroh called outside the transport adapter: org-io/src/send.rs:124 (EndpointId::from_bytes), peer_addr.rs:11, :75 (EndpointAddr wrap + postcard decode), preflight.rs:104 (relay_urls() on OrgEndpoint::addr()'s iroh type); no LLR or scan binds iroh to one module (Constraints 1).
disposition: fixed (task `af`) — `peer_addr.rs` moved to `org-io/src/transport/peer_addr.rs`; `OrgEndpoint::{send_body_to_device, send_body_at, has_relay_home}` replace iroh use in `org-io/src/send.rs` and `org-io/src/preflight.rs`, `addr()` removed; new LLR-3vb4k9 and scan `iroh_is_named_only_in_the_transport_module_and_test_support` (`org-io/tests/absences.rs`), red on the old tree.

**finding-2**: guideline, medium — Core and edges 1 — org-io/src/send.rs:92-130 send_committed mixes dials with the Loopback recipient rules (AddressNotARecipient, NoLoopbackAddress, NotAttemptedInLoopback; LLR-r9sh6k); no pure plan.
disposition: fixed (task `af`) — a pure `send::plan_send` holds the Loopback recipient rules, no behaviour change, `org-io/src/send.rs`.

**finding-3**: guideline, medium — Domain 1 — new user-facing text says "Device key" (keychain.rs:51-60, lib.rs:198, transport/mod.rs:101-104, send.rs:67, preflight.rs:92-117); glossaries list it under Avoid.
disposition: fixed (task `af`) — new error text says DevicePublicKey and Device seed, `org-io/src/keychain.rs`, `org-io/src/transport/mod.rs`, `org-io/src/preflight.rs`.

**finding-4**: guideline, medium — Domain 2 — new cross-unit public terms undefined in a glossary: DeviceSecret (org-node keys.rs:64), SealedStore (store.rs:760), KeychainError / "keychain item" (keychain.rs:34), Verdict (transport/mod.rs:50), "store file" (store_file.rs:56); "Device secret" vs "Device seed" both used.
disposition: fixed (task `af`) — `docs/CONTEXT.md` defines Device seed, DeviceSecret, Keychain item, SealedStore, Store file and Verdict, with "Device secret" under Avoid.

**finding-5**: guideline, medium — Boundaries 3 — SDD-v2bq89 (org-io/docs/architecture/2026-10-07-transport.md:1134) = "sending, and the peer address": two responsibilities.
disposition: fixed (task `af`) — SDD-v2bq89 narrowed to the fan-out; new SDD-wr8m5n is the peer-address parse and LLR-2b2ghe is re-traced to it, `org-io/docs/architecture/2026-10-07-transport.md`.

**finding-6**: guideline, low — Boundaries 1 — app/src-tauri/tests/startup_policy.rs:87-90 copies org-io's store-file naming.
disposition: fixed (task `af`) — the test names the store file through org-io, `app/src-tauri/tests/startup_policy.rs`.

**finding-7**: guideline, low — Boundaries 2 — TransportError, StoreFileError.reason, OpenError::CreateDataDir stringify the received error; none keeps it as source().
disposition: fixed in part (task `af`) — `OpenError::CreateDataDir` keeps its `io::Error` as `source()` (`org-io/src/lib.rs`); `StoreFileError` unchanged (an error field breaks clippy's large-error limit and LLR-b9bm36's stated shape); iroh errors stay strings on purpose, since keeping them would expose iroh types through org-io's public API against LLR-d5y7ks and finding-1.

**finding-8**: guideline, low — Core and edges 4 — the hex crate is called from six new modules.
disposition: no change — hex is a stateless encoding with no failure mode or upgrade risk the clause guards against; accepted.

### Round 2

> ARCHITECTURE conformance review round 2 (review-conformance, .guardrails/templates/ARCHITECTURE_GUIDELINES.md), at 73d5b55.
> verdict: 5 findings (0 high, 3 medium, 2 low). Findings 1–3 are above low, all Domain 1 glossary wording in user-facing text.

**finding-1**: guideline, medium — Domain 1 — org-node/src/error.rs:167, :171; keys.rs:90, :129 — new error text uses "device secret" / "device key" (avoided terms).
disposition: fixed (task `af2`) — the error text says Device seed, Persona store and DevicePublicKey, `org-node/src/error.rs`, `org-node/src/keys.rs`.

**finding-2**: guideline, medium — Domain 1 — app/src/lib/components/Revoke.svelte:138-139 — new UI text "reached by its Device key".
disposition: fixed (task `af2`) — the hint says DevicePublicKey, `app/src/lib/components/Revoke.svelte`.

**finding-3**: guideline, medium — Domain 1 — org-io/src/preflight.rs:132 — "no Persona store file"; "persona store file" is under Store file's Avoid.
disposition: fixed (task `af2`) — the preflight says store file, and the round's notes (doc comments in send.rs and preflight.rs) use the glossary terms, `org-io/src/preflight.rs`, `org-io/src/send.rs`.

**finding-4**: guideline, low — Domain 1 — new public identifiers named `device_key` typed DevicePublicKey (org-io keychain.rs, lib.rs; org-node keys.rs, store.rs, invitation.rs); no identifier exemption for DevicePublicKey as there is for PersonPublicKey.
disposition: fixed (task `af2`) — a dated identifier exemption for `device_key` added to the DevicePublicKey glossary entry, matching PersonPublicKey's, no renames, `docs/CONTEXT.md`.

**finding-5**: guideline, low — Domain 3 — org-node store.rs:812, :819, :930 report non-chain failures as OrgNodeError::Chain.
disposition: no change in this round — already recorded as open problem report PR-u784zs (CODE round 1, finding-4).

### Round 3

> ARCHITECTURE conformance review round 3 (review-conformance, .guardrails/templates/ARCHITECTURE_GUIDELINES.md), at 261c73d6.
> verdict: 1 finding (0 high, 0 medium, 1 low); none is above low.

**finding-1**: guideline, low — Core and edges 4 — org-node/src/invitation.rs:92, :97 — the Blob codec moved into org-node calls postcard directly, a sixth org-node module doing so (with store.rs, service.rs, envelope.rs, revocation.rs, wire.rs).
disposition: no change — postcard is org-node's one wire and storage encoding, called from five modules on master already; the moved Blob codec follows that existing pattern, and gathering postcard behind one org-node adapter is a refactor across code S4 did not write, left for a later change. The round's note (about 35 rustdoc comments still saying "Device key" or "device secret") is recorded under Gaps.

## Owner decisions and rulings recorded here

- **REQ-by9k5q clarification approved** (2026-10-14). The S3b-io session asked
  for it in this record (merge notes, item 3): on S4's structure "the Device
  the submission named" reads "the recipients the submission would have sent
  to", a dated clarification with the ID kept
  (`org-io/docs/requirements/2026-10-13-publish.md`). No merged S3b-io test
  is re-annotated.
- **org-io class C coverage gap accepted** (2026-10-14). The floors in the
  root `Makefile` are met (org-io lines 80, regions 78, branches 82; the
  gate's figures are in the squash trailers). The shortfall against the class
  C target, per file as T18 measured it: the chain-facing shell
  (`org-io/src/bin/preflight.rs` 0.00%, `preflight.rs` 40.00%, `connect.rs`
  30.00%, `chain_read.rs` 54.17%, `signatory.rs` 72.92%), and two shells S4
  adds: `transport/mod.rs` 54.17% (the iroh relay and Networked-mode binding
  no gated test reaches without a relay) and the platform keychain and file
  system refusal arms (`keychain.rs` 91.89%, `store_file.rs` 90.35%).
- **The chain is a boundary for org-io's tests** (2026-10-14; TEST round 1,
  finding-2): FakeChain stays; on-chain-client's end-to-end tests cover the
  real contract.
- **Six met expectations owe org-io tests; HAZ-cfp4jb lowered only after a
  stop-and-restart test** (2026-10-14; T19, T20). HAZ-cfp4jb now S3/P1,
  acceptability unchanged.
- **Gate check 6** (2026-10-14): the robustness gaps of S4's new items were
  closed in T23; the moved items' gaps are recorded as known (Gaps).
- **HAZ-4hj5m9 residual accepted** (2026-10-15): S3/P1, on the benefit-risk
  reasoning of HAZ-4n7pss.
- **org-io `problem_open_max` raised from 10 to 15** (2026-10-15;
  `org-io/.guardrails/config.yaml`), with the round-2 scope ruling.
- **Round-2 scope** (2026-10-15): fix the mediums and the mechanical lows;
  file the larger lows as one umbrella org-io report (PR-k5tdxj).
- **Fix all architecture findings above low** (2026-10-15; ARCHITECTURE
  round 1).
- **Two test rulings** (2026-10-15; TEST round 2): delete
  `state_assembly::init_opens_the_store_once_even_when_the_connect_is_refused`
  (covered by org-io's store-opening tests and the app's startup-refusal
  tests); mint an org-io LLR for
  `a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing`
  (LLR-nqdr97, minted in task `ct3`).
- **Retirement ruling** (2026-10-13): a retired item's definition and its
  `assesses:` entries are deleted and every ledger mention rewritten as prose
  that does not name it; this record lists the retirements by ID. Retired:
  LLR-379hnv (T4, done in T6), LLR-g9vmbx (T6), LLR-23sfdh (superseded by
  LLR-jd82t6 under the owner's T9 ruling, option A, then retired; no
  `supersedes:` line, since the old definition is gone), LLR-q225ws (T12).
- **Superseded, kept with `superseded-by:`**: LLR-a9rjtf → LLR-zw6jqy (T16).
  Not retired (review round 1, finding-17).
- **Earlier rulings this change implements**: rulings 1–12 of the plan header
  (2026-10-06 to 2026-10-08), and the 2026-10-13 ruling "An individual can
  only be member of an organisation once" (REQ-uy7kfq, LLR-yexf5m).

## Problem reports

- Resolves (each `status: resolved` in its ledger): PR-gnh3j2 (org-node),
  PR-jav8zn (org-node), PR-n696wv, PR-8qsnhx, PR-7v8anh, PR-kwwap5,
  PR-2dmjzj, PR-3ue4va, PR-tcd2ta, PR-47aycj, PR-y6t5kt, PR-9q5ems,
  PR-d2p5y2 (org-io), PR-erskh4, PR-aa7prn, PR-jgvf3b, PR-6f3qku (org-node),
  and PR-7zz76t (app, closed obsolete).
- Opens (each `status: open` in its ledger): PR-m3tfnu, PR-7hu3v5, PR-b8kmk2,
  PR-43t2zn, PR-vy83sd, PR-wayqj3, PR-a63sk3, PR-k5tdxj (org-io); PR-6gf4t2,
  PR-gq83cs, PR-u784zs, PR-qpam4k, PR-bjaq7j (org-node).
- Left open by the S3b-io session's notes: PR-vt244s (org-node, S3b-io part
  2, ruling D1) and PR-6nhrr7 (org-io).
- Accepts: the owner decisions above.

## Merge notes from the S3b-io session

From the plan's "Merge notes from the S3b-io session (received 2026-10-14)":

- PR-jgvf3b is under Resolves (above). PR-vt244s and PR-6nhrr7 stay open; both
  age limits stay at 60 days.
- For S3b-io part 2, S4's stand-ins: `dial_direct_as` (the test-support
  direct dial a revoked Device uses, `org-io/src/test_support.rs`), the kept
  keychain item of a forgotten Persona (S4 removes the store file and leaves
  the item; its deletion is S3b-io's, owner ruling 11), and acknowledgements
  the app drops (the app receives `Received::Acknowledged` and announces
  nothing). Deferred: a `SendReport` in `Settled::Committed.sent` (kept as
  `Result<(), String>`, S3b-io's public type) and in
  `Reconciliation::Committed`, whose send failures are not reported until
  part 2 (PR-b8kmk2).
- The settle (A3) fix to S3b-io's rule (review round 2, finding-15,
  PR-9q5ems) restores LLR-t98pk9's and REQ-by9k5q's own clause; it is offered
  to the owner for confirmation.
- Review scope (item 7): that S3b-io's behaviour survived the restructure —
  an unknown write is never forgotten, the publish lock is taken after the
  node lock, LLR-pk6mhv's founding rule holds, and no public discard method
  exists (LLR-cc59ys) — was checked in the deslop pass (plan, all hold) and
  left untouched by T20 and T22 (their reports).

## Gaps

- Moved items' one-sided tests. The gate's robustness heuristic found these
  moved items with a normal-case or abnormal-case test only, each moved with
  its tests unchanged (ruling A): LLR-2smrvx, LLR-6adc99, LLR-cns6q6,
  LLR-jn5jeh, LLR-k2y6nn, LLR-pw369n, LLR-rt8gdz, LLR-v873fx (abnormal only);
  LLR-6zjzn2, LLR-ecz9a6, LLR-fa7jt8, LLR-wx77j5, LLR-ygn78w (normal only).
  Known gaps outside S4 (owner, 2026-10-14). LLR-qezw3n's and LLR-4tcxsu's
  planted violation was refused by the session's permission classifier in
  T23; their abnormal case is T14's red run against the real violations.
- `two_different_keypairs_do_not_share_a_device_key`
  (`org-node/tests/key_custody.rs`) has had no `verifies:` since before S4
  (dropped at REQ-ztdza4's amendment); left as it is.
- About 35 rustdoc comments on new public items still say "Device key" or
  "device secret", and the ledgers use "Device key" widely (review round 1,
  finding-20; ARCHITECTURE round 3 note). User-facing text and error messages
  use the glossary terms.
- LLR-nqdr97's normal case is covered by a test that verifies LLR-r9sh6k
  (its Normal pointer says so); its own test is the abnormal case.
- LLR-3vb4k9's red evidence is its scan run red on the old tree (send.rs:124,
  peer_addr.rs:11/28/35/75), not a test written before an implementation.
- The A6 flag order (`Endpoints::mark_receiving` before the listing) has no
  deterministic test; forcing the interleaving would need a hook between the
  listing and the bind (LLR-z35vbh's dated amendment).
- org-node and the app have no coverage target or floor (T18 measured them for
  information only).
- org-io's coverage shortfall against the class C target is accepted (above).
- Open expectation opened by this change: app REQ-q89tjr (`expects:
  org-io`; the Organisation a failed verification belongs to).
- Open problem reports from this change, listed above; the larger review lows
  sit in PR-k5tdxj (keychain calls on worker threads, undelivered bodies of an
  executed write, the shallow allowlist scan, tests missing at an item's
  interface).
- The submission, settle, reconcile and reconcile_held hold the node lock for
  their whole sequence, chain write included (T11 note; LLR-9arvqj covers the
  receive step only); a narrowing was offered to the owner.
- `admit_member` and `revoke_member` return Err when a committed update's send
  failed; the alternative (Ok plus a warning, a DTO change) was offered to the
  owner in T15. No ruling is recorded in the plan; the current behaviour
  stands.
- `OrgNodeError::Chain` remains org-node's catch-all for encode failures
  (PR-u784zs); the postcard adapter is a later refactor (ARCHITECTURE round
  3).
- Platforms: the Windows and iOS keychain backends were assessed from source
  only; Android has no keychain backend in keyring 3.6.3 and is a later stage
  (roadmap row). Linux CI runs org-io's networked and keychain targets over
  the memory keychain and loopback; a relay flake there is possible.
- Left-whole plans: none — `prune-plans.sh` pruned only this plan.
