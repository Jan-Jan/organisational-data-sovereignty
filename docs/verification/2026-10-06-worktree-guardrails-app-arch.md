# Verification — app architecture, SOUP, low-level requirements and CSP (2026-10-06)

branch: worktree-guardrails-app-arch
reviewer: two independent fresh subagents, one per round, each in its own review worktree (`.worktrees/worktree-guardrails-app-arch-review`, then `-review2`). Each had the diff, the plan, the ledgers and the review checklist, and no implementation narrative. Each ran the suite itself.
verdict: ACCEPTED for merge 2026-10-06. Round 1 raised three medium and three low requirement findings and three record findings. Round 2 found all nine resolved and raised three low requirement findings and two record findings, which made it the last review round. Every finding is fixed.
reproduced: yes, for the one production defect this change fixes. At `06c357b`, `app/src-tauri/tauri.conf.json` sets `app.security.csp: null`. The new test `shipped_csp_meets_every_clause` was watched fail on it ("policy must be a string, found null") before the policy was written. The architecture, SOUP and LLR work is documentation and test work with no defect to reproduce. Its tests were proved able to fail by mutation instead (the appendix).

Change: the `app` unit's first architecture ledger (11 SDD, 46 LLR), a measured SOUP inventory, five new problem reports, and a Content-Security-Policy for the webview (HAZ-e2zupz, RC-mq2365, REQ-cj5jmx). Branched from local `master` at `06c357b`; `master` did not move while the change was open.
Plan: `docs/plans/2026-10-05-app-architecture.md`.

Resolves: none. Accepts: none. Opens: PR-8vh53d, PR-cu2h2g, PR-uxj7bg, PR-bu6mau, PR-7zz76t.

## The gate

Measured on: `fe92b7c95018d4569a0a6feab3f851df9756960c` — tree
`fed80016d7a75532adb0d30b114396870aadf3a2`, a clean worktree, by a fresh gate
subagent after review round 2's fixes. `merge-change` step 3 had already run
earlier and renamed this change's five draft ledger files. Nothing was left to
rename at this tree. Impact set (`check-units.sh --impact master..HEAD`):
`app` touched, `org-node` touched (one dated correction in its `soup.md`).
`check-units.sh`: exit 0.

**Environment fault, recorded.** On the first run, both cargo commands exited
101 before compiling ("failed to find ark-poly v0.5.0 in path source"). About
550 of the unpacked crates in the shared `CARGO_HOME=/tmp/cargo_home_fuzz` had
lost their files during the run, from something outside the gate. A plain
retry failed the same way. Both commands were then re-run unchanged against a
fresh, isolated `CARGO_HOME`, fetching online, and the figures below are from
those runs. Every other command passed on its first run.

| Gate | Result |
| --- | --- |
| app `cargo test … --features test-support` (8 targets) | 133 passed, 0 failed, 0 ignored |
| app `npm --prefix app run check` | 0 errors, 1 warning (the existing missing `@types/node`) |
| app `npm --prefix app run test` | 36 passed, 0 failed (4 files) |
| org-node `cargo test -p org-node --features app,test-support` (lib + 22 targets) | 167 passed, 0 failed; the three bolero targets ran their 1 s budget |
| org-node quint: typecheck ×2, five invariants | clean; no violation in any |
| `check-ids.sh`, app and org-node | exit 0, no findings; no draft file remains |
| `check-trace.sh`, app and org-node | exit 0. This change's own IDs entered the app roll-call: HAZ-e2zupz, RC-mq2365, REQ-cj5jmx, 11 SDD, 46 LLR, and PR-8vh53d, PR-cu2h2g, PR-uxj7bg, PR-bu6mau and PR-7zz76t. None left it. org-node's roll-call is unchanged. |
| Implements map | all 46 LLRs have a direct `verifies:` test. REQ-cj5jmx and RC-mq2365 are covered through their eight CSP LLRs. Every LLR's Normal and Abnormal lists name tests that exist and carry it. |
| `merge-preflight.sh --before-review --local-base` | CLEAN-TREE, BASE-MERGED (local base), UNITS, NESTED-WORKTREE ok; IDS/TRACE run per unit; REVIEW skipped |
| Coverage, against the class target | **SHORTFALL, accepted by the owner 2026-10-05 for this merge.** Neither `app` nor `org-node` configures a `coverage_command`, so class C statement and decision coverage are unmeasured in both. |
| Working tree | clean before and after |

Earlier gates on this change, none of whose figures are reported above:

- step 2, at tree `663023d9…`;
- after the step-3 renames, at tree `4f5e4253…`;
- after review round 1's fixes, at tree `119db9d0…`.

All three were green on the first run. Every review round also ran the suite
in its own worktree: round 1 counted 122/0 cargo and 36 vitest; round 2
counted 133/0 and 36.

## Red → green

**The CSP is this change's one production fix.** These rows come from the T2
and R1 dispatch reports, copied from the plan:

| Item | Test | Watched red |
| --- | --- | --- |
| REQ-cj5jmx, RC-mq2365 (via LLR-7ymxtn and the other CSP LLRs) | `shipped_csp_meets_every_clause` | failed on `csp: null` ("policy must be a string, found null") before the policy was set |
| LLR-tc5aax | `dev_csp_adds_only_the_dev_server_origins` | failed with `devCsp` absent, before it was set |
| LLR-uus6ar | `script_src_elem_is_rejected`, `script_src_attr_is_rejected`, `worker_src_is_rejected` | all three failed before the checker's directive allow-list existed (R1); `shipped_csp_meets_every_clause` reddened under round 1's `script-src-elem 'unsafe-inline'` probe |
| LLR-tc5aax | `dev_policy_that_differs_beyond_the_dev_origins_is_rejected`, `dev_csp_differs_from_csp_only_by_the_dev_server_origins` | failed against a stub comparator; reddened by each of round 1's devCsp probes (R1) |

**Every other LLR describes code that existed before this change.** Its tests
are characterization tests, and so are the tests re-pointed from REQ to LLR. A
characterization test can only be watched red by mutating the code it pins.
The "Watched red" column therefore names the mutation runs that reddened the
test, listed in the appendix. Each run restored its file, and SHA-256 proved
the restore byte-identical.

- **New tests.** T3 and T4 added new tests, and R1 and R2 strengthened others.
  Each has its own red → green line in the plan: the mutation that reddened
  it and the hash check.
- **LLR-uus6ar** has no sweep run. It was written after the sweep, and its
  red is the R1 row above.

| Item | Test | Watched red |
| --- | --- | --- |
| LLR-tpkmh3 | `configured_passphrase_is_used`, `absent_passphrase_is_refused`, `empty_passphrase_is_refused_not_honoured`, `absent_passphrase_with_opt_in_uses_dev_default`, `configured_passphrase_wins_over_opt_in` | sweep runs T5-A01, T5-A02, T5-A03, T5-A04, T5-A06, T5-A07 (appendix) |
| LLR-kfmng5 | `override_data_dir_is_used`, `resolved_data_dir_is_used_when_no_override`, `unresolvable_data_dir_is_refused`, `unresolvable_data_dir_with_opt_in_uses_temp`, `empty_override_is_ignored_not_used_as_path` | sweep runs T5-A12, T5-A13 (appendix) |
| LLR-97rww8 | `passphrase_refusal_names_both_variables_supplier_first`, `data_dir_refusal_names_both_variables_supplier_first`, `refusals_for_empty_values_name_both_variables_supplier_first` | sweep runs T5-A16, T5-A17, T5-A18, T5-A19, T5-A20, T5-A21, T3-M1 (appendix) |
| LLR-j5pacp | `networked_transport_is_reported`, `loopback_transport_is_reported`, `unknown_transport_value_is_networked` | sweep runs T5-A23, T5-A26, T5-A27 (appendix) |
| LLR-85zque | `connection_status_reports_a_data_dir_it_had_to_create_verbatim`, `the_store_is_opened_inside_the_data_dir_it_creates`, `a_data_dir_that_cannot_be_created_is_refused_naming_it`, `a_store_under_another_passphrase_is_refused` | sweep runs T5-A29, T5-A30, T5-A31, T5-A32, T3-M2 (appendix) |
| LLR-vqkr5t | `connection_status_reports_the_directory_the_store_was_opened_in`, `connection_status_reports_a_data_dir_it_had_to_create_verbatim` | sweep runs T5-A34, T3-M5 (appendix) |
| LLR-9x6qrg | `configured_chain_reports_its_endpoint_and_contract`, `unconfigured_chain_reports_neither`, `absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env` | sweep runs T5-A35, T5-A36 (appendix) |
| LLR-kze6ak | `configured_chain_reports_its_endpoint_and_contract`, `endpoint_comes_from_the_built_configuration`, `data_dir_comes_from_the_built_configuration`, `abnormal_data_dir_paths_are_reported_verbatim` | sweep runs T5-A42, T5-A44, T5-A45 (appendix) |
| LLR-8tzbzn | `networked_transport_is_reported`, `loopback_transport_is_reported`, `transport_mode_serialises_lowercase`, `connection_status_reports_transport_mode_over_ipc`, `connection_status_is_registered_under_its_name`, `unknown_command_is_rejected` | sweep runs T5-A48, T5-A49, T5-A50, T5-A51, T5-A52, T5-A53, T5-A54, T5-A55 (appendix) |
| LLR-ecaw34 | `forty_hex_characters_are_accepted`, `a_zero_x_prefix_is_accepted_and_parses_identically`, `uppercase_hex_is_accepted`, `a_zero_x_prefix_with_forty_characters_after_it_is_accepted`, `an_empty_string_is_refused`, `thirty_nine_characters_are_refused`, `forty_one_characters_are_refused`, `an_odd_length_is_refused_on_width_not_on_hex_decoding`, `a_non_hex_character_at_the_first_position_is_refused`, `a_non_hex_character_at_the_last_position_is_refused`, `forty_non_hex_characters_are_refused`, `a_doubled_zero_x_prefix_is_refused`, `a_zero_x_in_the_middle_is_refused`, `the_bare_prefix_alone_is_refused` | sweep runs T5-A73, T5-A74, T5-A75, T5-A78, T5-A79, T5-A80, T5-A82 (appendix) |
| LLR-vzf8j2 | `export_invite_rejects_a_short_org_id`, `export_invite_rejects_a_non_hex_org_id`, `admit_member_refuses_a_malformed_org_id_with_the_parsers_message`, `admit_member_does_not_refuse_a_well_formed_org_id`, `revoke_member_refuses_a_malformed_org_id_with_the_parsers_message`, `revoke_member_does_not_refuse_a_well_formed_org_id`, `export_invite_does_not_refuse_a_well_formed_org_id` | sweep runs T5-A83, T3-M6 (appendix) |
| LLR-6pmrma | `revoke_member_rejects_a_short_member_id`, `revoke_member_rejects_a_doubled_zero_x_prefix_on_the_member_id`, `revoke_member_is_not_refused_for_an_empty_peer_addr`, `revoke_member_accepts_a_zero_x_prefixed_member_id` | sweep runs T5-A87, T5-A88, T5-A89, T5-A90, T3-M7 (appendix) |
| LLR-ty85xv | `revoke_member_is_not_refused_for_an_empty_peer_addr`, `revoke_member_treats_a_whitespace_only_peer_addr_as_absent` | sweep runs T5-A91, T3-M8 (appendix) |
| LLR-n6twt7 | `revoke_member_does_not_refuse_an_encoded_endpoint_addr`, `revoke_member_refuses_a_peer_addr_that_is_not_hex`, `revoke_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr` | sweep runs T5-A93, T5-A94, T3-M9 (appendix) |
| LLR-8krgzj | `admit_member_refuses_an_org_secret_that_is_not_32_bytes`, `admit_member_does_not_refuse_a_32_byte_or_absent_org_secret`, `admit_member_refuses_an_org_secret_that_is_not_hex` | sweep runs T5-A97, T5-A98, T5-A99, T3-M12 (appendix) |
| LLR-ctrfz4 | `admit_member_does_not_refuse_a_32_byte_or_absent_org_secret`, `admit_member_does_not_refuse_a_decodable_node_addr`, `admit_member_refuses_a_node_addr_that_is_not_an_endpoint_addr` | sweep runs T5-A101, T5-A103, T3-M14, T3-M15 (appendix) |
| LLR-p7dfxb | `create_persona_stores_the_parsed_details`, `create_persona_refuses_an_invalid_field_naming_it_and_creates_nothing` | sweep runs T5-A105, T5-A106, T5-A107, T5-A108 (appendix) |
| LLR-4wcyqy | `connection_status_is_registered_under_its_name`, `unknown_command_is_rejected`, `product_registers_the_same_commands_as_the_harness`, `list_personas_returns_an_empty_list_on_a_fresh_store` | sweep runs T5-A56, T5-A57 (appendix) |
| LLR-pguhw5 | `list_personas_reports_exactly_the_persona_fields`, `a_persona_in_no_organisation_reports_a_null_org_id` | sweep runs T5-A58, T5-A59, T5-A60, T5-A61, T5-A62, T5-A63, T5-A64, T3-M16 (appendix) |
| LLR-pmus9f | `import_join_request_reports_the_request_it_decodes`, `import_join_request_refuses_a_malformed_blob` | sweep runs T5-A65, T5-A66, T5-A67, T5-A68, T5-A69, T5-A70, T5-A71, T5-A72, T3-M18 (appendix) |
| LLR-5zd6j8 | `first_claim_succeeds`, `second_claim_while_held_fails`, `concurrent_claims_yield_exactly_one_guard` | sweep runs T5-B01, T5-B02 (appendix) |
| LLR-z66f27 | `claim_succeeds_again_after_the_guard_drops`, `guard_releases_when_dropped_by_panic`, `release_then_concurrent_claims_yield_exactly_one_guard` | sweep runs T5-B04, T5-B05 (appendix) |
| LLR-7bk6qh | `every_verification_verdict_is_classified_as_a_verification_failure`, `a_verification_verdict_carries_its_own_message_and_no_invented_organisation`, `a_chain_failure_is_classified_as_a_receiver_error`, `a_chain_failure_emits_no_verification_event_for_any_message`, `the_two_classes_are_actually_distinguished`, `locally_reachable_variants_are_classified_as_receiver_errors`, `a_refused_key_or_field_is_classified_as_a_receiver_error` | sweep runs T5-B06, T5-B07, T5-B08, T5-B09, T5-B10, T5-B11, T5-B12, T5-B13, T5-B14, T5-B15, T5-B16, T5-B17, T5-B18, T5-B19, T5-B20 (appendix) |
| LLR-usxk57 | `a_chain_failure_emits_no_verification_event_for_any_message`, `the_receiver_error_payload_carries_a_message_and_no_verification_state` | sweep runs T5-B21, T5-B22, T5-B23, T5-B24, T5-B25, T5-B26, T5-B27, T5-B28 (appendix) |
| LLR-a9rjtf | `every_real_terminal_message_stops_the_loop`, `a_non_terminal_failure_does_not_stop_the_loop`, `the_stop_is_announced_after_the_failure_for_the_same_error` | sweep runs T5-B29, T5-B30, T5-B31, T5-B32, T5-B33, T5-B34, T5-B35, T5-B36 (appendix) |
| LLR-2zmhvs | `stopped_names_the_reason`, `stopped_emits_exactly_one_event` | sweep runs T5-B37, T5-B38, T5-B39, T5-B40 (appendix) |
| LLR-p2nm5a | `updated_emits_membership_incoming_and_epoch`, `updated_payload_carries_the_measured_epoch_and_root`, `updated_carries_boundary_epochs_and_roots_verbatim` | sweep runs T5-B41, T5-B42, T5-B43, T5-B44, T5-B45, T5-B46, T5-B47, T5-B48, T5-B49, T5-B50, T5-B51, T5-B52, T5-B53, T5-B73, T3-M20 (appendix) |
| LLR-hgsdm8 | `record_unreadable_emits_no_membership_event`, `record_unreadable_emits_no_epoch_event`, `record_unreadable_names_the_organisation`, `record_unreadable_emits_exactly_one_event`, `record_unreadable_holds_for_any_organisation_id` | sweep runs T5-B54, T5-B55, T5-B56, T5-B57, T5-B58, T5-B59, T5-B60, T3-M21 (appendix) |
| LLR-2vg79y | `self_delete_emits_no_epoch_event`, `self_delete_emits_revoked_naming_the_organisation`, `self_delete_payload_carries_no_epoch_key_at_any_depth`, `self_delete_carries_no_epoch_for_any_organisation_id`, `the_epoch_probe_finds_an_epoch_on_updated_and_none_on_self_delete` | sweep runs T5-B61, T5-B62, T5-B63, T5-B64, T5-B65, T5-B66 (appendix) |
| LLR-p38be7 | `verify_failure_carries_the_organisation_when_known`, `verify_failure_carries_null_organisation_when_unknown`, `verify_failure_carries_the_message`, `verify_failure_emits_no_membership_event` | sweep runs T5-B67, T5-B68, T5-B69, T5-B70, T5-B71, T5-B72 (appendix) |
| LLR-53hayh | `renders a verified event as verified`, `gives a verified event no detail`, `preserves epoch 0 on a verified event rather than coercing it to null`, `passes the timestamp through unmodified` | sweep runs T5-C01, T5-C02, T5-C04, T5-C05, T5-C06 (appendix) |
| LLR-a4xwvj | `renders a failed event as not verified`, `gives a failed event no epoch and no root`, `carries the failure message as the row detail`, `passes the timestamp through unmodified` | sweep runs T5-C07, T5-C08, T5-C09, T5-C10, T5-C11 (appendix) |
| LLR-mzae5q | `renders a placeholder rather than `, `keeps the organisation a failed event names` | sweep runs T5-C12, T3-M23 (appendix) |
| LLR-fb7jp5 | `produces no verification row for a receiver error`, `renders a receiver error outside the verification log`, `leaves an existing verification log untouched`, `assigns no verification outcome however many receiver errors arrive`, `does put a verification failure in the log and not in the receiver errors`, `does not mutate the view it was given`, `files the newest announcement first in each list`, `keeps at most 20 receiver errors, dropping the oldest` | sweep runs T5-C13, T5-C14, T5-C15, T5-C16, T5-C17, T5-C18, T5-C19 (appendix) |
| LLR-rzx6ks | `files the newest announcement first in each list`, `keeps at most 50 verification rows, dropping the oldest`, `keeps at most 20 receiver errors, dropping the oldest` | sweep runs T5-C20, T5-C21, T3-M24, T3-M25, T3-M26 (appendix) |
| LLR-z4ky6f | `registers every subscription it is given`, `unsubscribes exactly the listeners it registered, including those whose listen call resolved late`, `resolves only after every subscription has registered`, `is idempotent — calling the cleanup twice unsubscribes once`, `propagates a subscription failure rather than resolving a partial cleanup`, `registers nothing and cleans up cleanly when given no subscriptions` | sweep runs T5-C22, T5-C23, T5-C24, T5-C25, T5-C26, T5-C27 (appendix) |
| LLR-csbs5v | `rejects a 63-character member id`, `rejects a 65-character member id`, `rejects an empty member id`, `rejects a 64-character non-hexadecimal member id`, `accepts a 0x-prefixed member id`, `rejects a doubled 0x prefix`, `trims surrounding whitespace before measuring` | sweep runs T5-C28, T5-C29, T5-C30, T5-C31, T5-C32, T3-M27 (appendix) |
| LLR-c9r5uf | `accepts an empty peer address in networked transport`, `accepts a supplied peer address in networked transport`, `rejects an empty peer address in loopback transport`, `accepts a supplied peer address in loopback transport`, `trims surrounding whitespace before measuring` | sweep runs T5-C33, T5-C34, T5-C35, T5-C36, T5-C37, T5-C38 (appendix) |
| LLR-7ymxtn | `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`, `null_policy_is_rejected`, `empty_policy_is_rejected`, `default_src_other_than_self_is_rejected` | sweep runs T5-D01, T5-D02, T5-D03, T5-D04, T5-D06, T5-D48 (appendix) |
| LLR-hdvy6x | `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`, `unsafe_inline_in_script_src_is_rejected`, `unsafe_eval_in_script_src_is_rejected`, `remote_origin_in_script_src_is_rejected` | sweep runs T5-D09, T5-D10, T5-D11, T5-D12 (appendix) |
| LLR-c88jhh | `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`, `missing_object_src_is_rejected`, `each_none_directive_is_required_to_be_none` | sweep runs T5-D17, T5-D18, T5-D19, T5-D20, T5-D21, T5-D22, T5-D23, T5-D24, T5-D25 (appendix) |
| LLR-df7prq | `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`, `connect_src_beyond_ipc_is_rejected`, `localhost_dev_server_is_rejected_in_shipped_policy` | sweep runs T5-D27, T5-D28, T5-D29, T5-D30, T5-D31 (appendix) |
| LLR-p3xwx4 | `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`, `wildcard_source_is_rejected_in_any_directive`, `https_scheme_source_is_rejected_in_any_directive` | sweep runs T5-D32, T5-D33, T5-D34, T5-D35 (appendix) |
| LLR-ausr5q | `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`, `directive_names_are_matched_case_insensitively`, `repeated_directive_is_rejected` | sweep runs T5-D36, T5-D37, T5-D38, T5-D39 (appendix) |
| LLR-uus6ar | `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`, `script_src_elem_is_rejected`, `script_src_attr_is_rejected`, `worker_src_is_rejected` | no sweep run |
| LLR-tc5aax | `dev_csp_adds_only_the_dev_server_origins`, `dev_csp_differs_from_csp_only_by_the_dev_server_origins`, `dev_policy_rejects_an_origin_beyond_the_dev_server`, `dev_fixture_is_a_pure_addition_to_the_fixture`, `dev_policy_that_differs_beyond_the_dev_origins_is_rejected` | sweep runs T5-D40, T5-D41, T5-D42, T5-D43, T5-D44, T5-D45, T5-D46 (appendix) |

## What was wrong, and what was built

**Measured before**, at `06c357b`:

- `app` had 22 REQ, 6 HAZ and 12 RC. It had **SDD 0 and LLR 0** under class C.
  Its `soup.md` was an empty template.
- `tauri.conf.json` had `csp: null`. Any script that reached the webview could
  call all twelve ungated Tauri commands, `revoke_member` and `admit_member`
  among them, through `core:default` IPC.
- The gate ran 82 cargo tests over 6 targets and 30 vitest tests.

**Built:**

- **SOUP** (`app/docs/architecture/soup.md`).
  - `app/src-tauri` is its own workspace, with a tracked lock.
  - 495 third-party crates are shipped on the host target (521 with build and
    dev edges), counted as `name version` pairs.
  - The npm closure, with the caveat that its `dev` flag does not mean
    unshipped. npm audit results.
  - The inherited rows, and the platform webview that enforces the CSP.
  - A dated correction in `org-node/docs/architecture/soup.md`, whose claim
    that the root lock covers app's Rust side was false.
- **CSP** (`app/docs/risk/2026-10-05-csp.md`,
  `app/docs/requirements/2026-10-05-csp.md`, `tauri.conf.json`). It adds
  HAZ-e2zupz (S3/P1) and RC-mq2365, a protective measure. REQ-cj5jmx is
  `satisfies: derived` and assessed. The policy:
  - `default-src 'self'`;
  - no inline, evaluated or remote script;
  - `object-src`, `base-uri`, `frame-ancestors` and `form-action` are
    `'none'`;
  - `connect-src` allows Tauri IPC only;
  - the directive names are allow-listed;
  - `devCsp` equals `csp` except for the dev-server origins.
  - The residual is S3/P1, **UNACCEPTABLE and not accepted**, as for every S3
    entry in this register. The owner confirmed the ratings on 2026-10-05.
- **Decomposition** (`app/docs/architecture/2026-10-05-decomposition.md`).
  - Ten items carry LLRs: SDD-k95rmp, SDD-aq7m6b, SDD-q4dpym, SDD-2pa6h6,
    SDD-rmbr3t, SDD-5fchuy, SDD-mwqf6x, SDD-8jw9mn, SDD-jx363y and SDD-32hath.
  - SDD-6g3wnh is **the declared no-LLR deviation item** (owner rulings 1
    and 3). It holds everything no gated test reaches: the process entry and
    `init` wiring, chain-connection setup, `start_receiver`/`next_outcomes`,
    `api.ts`, every Svelte component, and the onboarding commands and panels.
  - There are 46 LLRs, 13 of them derived, each with an `assesses:` section in
    `app/docs/risk/2026-10-05-derived.md`.
  - The architecture README has the Overview.
- **Tests.** One new target, `state_assembly`, and a new `csp_policy` target.
  New and strengthened tests in the existing targets and vitest files. Every
  gated test now verifies an LLR, or a PR for the PR-cu2h2g pin.
  - The gate ran 133 cargo tests over 8 targets and 36 vitest tests.
  - No production source changed; `tauri.conf.json` is the only product file
    touched.
- **Problem reports** (`app/docs/problems/2026-10-05-problems.md`): PR-8vh53d,
  PR-cu2h2g, PR-uxj7bg, PR-bu6mau and PR-7zz76t. They are booked and not fixed
  (owner ruling 2), and pinned where the harness reaches.
  - A dated addendum to PR-h6xpnh covers the receiver start race and the event
    loss on a tab switch.
  - `problem_open_max` is raised from 10 to 15 (owner ruling 2).
- **Falsifiability sweep** (T5, the appendix).
  - 277 clauses: 261 RED and 16 GREEN.
  - Every GREEN clause was cut or narrowed. Eight of them share one cause: on
    `AppState::for_test`, admit and revoke fail at `find_org` before anything
    reaches the service.
- **The cleanup pass** renamed four tests whose names claimed something they
  could not observe, and LLR-vzf8j2 was narrowed to match.

## Review

### Round 1

Verdict (verbatim): not ready to merge. Three medium requirement findings (1, 3, 4) mean another round is needed. The shipped CSP itself is correct, and every problem-report fact and SOUP figure I re-checked holds.

Reviewer's suite: cargo 122 passed, 0 failed (8 targets); `npm run check` 0 errors, 1 warning; `npm run test` 36/36; check-trace rc=0; check-ids rc=0.

**finding-1**: requirement, medium — REQ-cj5jmx (`app/docs/requirements/2026-10-05-csp.md`) and LLR-hdvy6x constrain only `script-src`. Under CSP Level 3, `script-src-elem` and `script-src-attr` override it. The checker only rejects network sources in other directives (`app/src-tauri/tests/csp_policy.rs:79-86`, `:100-106`). Probe: adding `script-src-elem 'self' 'unsafe-inline'; script-src-attr 'unsafe-inline'` to the shipped `csp` kept csp_policy at 18/18 green. So the gate passes a policy that admits inline script, which RC-mq2365 ("no inline or evaluated script") forbids. That is a plausible edit for someone fixing a blank window. Fix: extend the REQ, LLR and checker to every directive that governs script (`script-src-elem`, `script-src-attr`, `worker-src`), or allow-list the permitted directive names.
disposition: fixed (R1). REQ-cj5jmx gained a directive allow-list clause, under the new LLR-uus6ar, and the checker rejects any directive outside it. `script_src_elem_is_rejected`, `script_src_attr_is_rejected` and `worker_src_is_rejected` were red before the allow-list existed. `shipped_csp_meets_every_clause` reddens under this finding's probe. Round 2 re-ran the probe: red.

**finding-2**: requirement, low — REQ-cj5jmx's clause that the development policy "shall differ from it only by adding the local development server's origins" has no LLR and no test. LLR-tc5aax (decomposition :710-717) narrows it to "meets the same clauses", and `dev_csp_adds_only_the_dev_server_origins` (`csp_policy.rs:146-153`) never compares `devCsp` with `csp`. Probe: a `devCsp` with `style-src` removed, `script-src-elem 'unsafe-inline' 'unsafe-eval'`, `worker-src blob:` and `img-src … blob:` stayed green.
disposition: fixed (R1). LLR-tc5aax now states the dev clause. `dev_csp_differs_from_csp_only_by_the_dev_server_origins` compares the two policies directive by directive, and reddens under each of this finding's probes. Round 2 re-ran them: red.

**finding-3**: requirement, medium — REQ-sjkp8z's org-id refusal in `admit_member` and `revoke_member` (`commands.rs:206`, `:265`) is neither gated nor booked anywhere. The harness reaches it: on `for_test`, a malformed `orgId` is refused before `find_org`. Owner ruling 1 therefore required a new test, not a narrowing. Instead the decomposition narrows LLR-vzf8j2 to `export_invite` (`2026-10-05-decomposition.md:234-237`). The refusal is not in the SDD-6g3wnh candidate list (:798-826) and not in any problem report. Probe: replacing revoke's `parse_org_id(&org_id)?` with `unwrap_or(OrgId::new([0xaa;20]))` left the whole cargo gate green. The architecture README Overview calls SDD-2pa6h6's Rust-side parsing "the unit's protection" at the trust edge, so the revocation path's piece of it is unprotected.
disposition: fixed (R1). LLR-vzf8j2 is widened back to all three commands. `admit_member_refuses_a_malformed_org_id_with_the_parsers_message` and `revoke_member_refuses_a_malformed_org_id_with_the_parsers_message` each redden under this finding's `unwrap_or` probe on their command.

**finding-4**: requirement, medium — LLR-4wcyqy ("the commands are registered under their snake_case names", decomposition :345-351) is verified only against the test's own copy of the handler list (`tests/ipc.rs:70-85`). The sweep's T5-A56 and T5-A57 mutated `tests/ipc.rs`, not product code. Probe: deleting `commands::revoke_member` from `app/src-tauri/src/lib.rs:40` left all 122 cargo tests green. `2026-10-05-derived.md:108-115` nonetheless credits it with product behaviour ("a frontend wrapper whose name drifts … fails visibly"). Fix: restate it as a property of the harness and move the product registration into SDD-6g3wnh's text, or gate `lib.rs`'s list (for example, one shared list both use).
disposition: fixed (R1). `product_registers_the_same_commands_as_the_harness` reads the `generate_handler!` list in `lib.rs` and compares it with the harness list. It reddens when `commands::revoke_member` is deleted from `lib.rs`. LLR-4wcyqy and the derived assessment are restated to match.

**finding-5**: requirement, low — LLR-pmus9f's clause "refused with org-node's message" cannot be falsified by any test. Probe: `map_err(|_| "invalid blob".to_string())` in `decode_join_request` (`commands.rs:173`) stayed green, because the test only checks `contains("blob")` (`ipc.rs:716`). The sweep's T3-M19 also dropped the word "blob", so it tested only the "names the blob" half.
disposition: fixed (R1). `import_join_request_refuses_a_malformed_blob` asserts equality with org-node's own message, and reddens under the `"invalid blob"` probe.

**finding-6**: requirement, low — LLR-6pmrma names the exact message `member_id must be 32 bytes (64 hex chars)`, but `revoke_member_rejects_a_short_member_id` accepts either `"32 bytes"` or `"64 hex"` (`ipc.rs:267`). Probe: rewording the message to `"member_id must be 64 hex chars"` stayed green. The sweep's T5-A90 removed both substrings, so it missed this.
disposition: fixed (R1). `revoke_member_rejects_a_short_member_id` asserts the exact message, and reddens under the rewording probe.

**finding-7**: record — two sweep RED verdicts came only from tests not annotated to the LLR under test. T5-C14 (LLR-fb7jp5's `[<timestamp>] <message>` format): only LLR-rzx6ks tests failed. T5-A42 (LLR-kze6ak, contract address reported verbatim): only `configured_chain_reports_its_endpoint_and_contract` (LLR-9x6qrg) failed. Both clauses hold, but the tests that verify them should carry those LLR IDs.
disposition: fixed (R1). The receiver.log tests that check the row format carry LLR-fb7jp5, and `configured_chain_reports_its_endpoint_and_contract` carries LLR-kze6ak. The decomposition's Normal lists name them.

**finding-8**: record — `app/docs/architecture/soup.md` does not record that RC-mq2365 is carried out by SOUP. Tauri injects the policy and appends the script hashes; the platform webview (WKWebView, WebView2, WebKitGTK) enforces it, and the webview is not inventoried at all. The tauri row (:87) does not list REQ-cj5jmx, and the Tauri configuration section (:140-154) does not mention the CSP.
disposition: fixed (R1). The tauri row cites REQ-cj5jmx and RC-mq2365, and says Tauri injects the policy and appends the script hashes. A new platform-webview row (WKWebView, WebView2, WebKitGTK) records that the OS supplies it, unpinned, and that it enforces the CSP. The Tauri configuration section names the CSP.

**finding-9**: record — two stale details: `soup.md:100` says `tempfile` is used in `tests/ipc.rs`; it is also used by `tests/state_assembly.rs`. The `test_paths` comment in `app/.guardrails/config.yaml` still says "six cargo integration targets"; there are now eight.
disposition: fixed (R1). The tempfile row names `state_assembly.rs`, and the `test_paths` comment says eight targets.

### Round 2

Verdict (verbatim): ready to merge on code and requirements. Every round-1 finding is resolved and I raise only low and record findings, so by the checklist this is the last review round. The owner's CSP smoke test (`npm --prefix app run tauri build`) is still required before the squash.

Reviewer's suite: cargo 133 passed, 0 failed (8 targets); `npm run check` 0 errors, 1 warning; `npm run test` 36/36; check-trace rc=0 (LLR 46); check-ids rc=0. All nine round-1 findings were re-probed and found resolved; round-1 regression probes R1–R6 still red.

**finding-1**: requirement, low — LLR-vzf8j2 says each command "returns the parser's message" on a refusal. For `export_invite`, no gated test can show that clause false. Its tests check only `contains("40 hex chars")` and `contains("hex")` (`app/src-tauri/tests/ipc.rs:224`, `:240`). The R1 fix asserts the exact message only for admit and revoke. Probe: replace `export_invite`'s error with a fixed `"org_id must be 40 hex chars"` / `"bad hex"` (`app/src-tauri/src/commands.rs:130`). All 133 tests stayed green. Fix: assert `parser_refusal(&bad)` there as well, the same way the admit and revoke tests do.
disposition: fixed (R2). Both `export_invite` refusal tests assert equality with `parser_refusal(&bad)`, and both redden under this finding's fixed-message probe.

**finding-2**: requirement, low — `product_registers_the_same_commands_as_the_harness` (`app/src-tauri/tests/ipc.rs:441-465`) reads only the first `generate_handler!` list in `lib.rs`. Tauri's `Builder::invoke_handler` replaces any earlier handler, so the last call wins. Probe: add `.invoke_handler(tauri::generate_handler![commands::list_personas])` before `.run(...)` in `app/src-tauri/src/lib.rs:46`. All 133 tests stayed green while the product registered one command. This needs a contrived edit. Fix: either assert that `lib.rs` contains exactly one `generate_handler!`/`invoke_handler`, or record this limit beside the one derived.md:124-126 already states ("compares names, not registration order or attributes").
disposition: fixed (R2). The test also requires exactly one `generate_handler!` and one `.invoke_handler(` outside `//` comments, and reddens under this finding's second-`invoke_handler` probe. Its limits are stated under LLR-4wcyqy and in Gaps.

**finding-3**: requirement, low — LLR-pmus9f still claims org-node's message "names the blob" (`app/docs/architecture/2026-10-05-decomposition.md`, LLR-pmus9f). The strengthened test (`ipc.rs:811-826`) now checks only that the message equals org-node's own. If org-node changed its message so it no longer names the blob, the test would stay green and the clause would be false. That clause is org-node's behaviour, which the app passes through. Fix: drop the clause, or keep a `contains("blob")` assertion next to the equality.
disposition: fixed (R2). `contains("blob")` is kept beside the equality. With org-node's blob errors reworded so they no longer name the blob, the equality stayed green and the `contains` failed.

**finding-4**: record — `app/docs/risk/2026-10-05-derived.md:5-13` still says "forty-five low-level requirements. Thirty-two refine…" and "Counted 2026-10-05 … `grep -c '^\*\*LLR-'` is 45". R1 added LLR-uus6ar, so the counts are now 46 and 33. The derived count of 13 is still correct.
disposition: fixed (R2). The counts read forty-six, thirty-three and thirteen, measured by grep.

**finding-5**: record — In `app/docs/risk/2026-10-05-csp.md` §3 (:95-99), the verification text still says only that the checker "checks every clause of REQ-cj5jmx". It does not mention the directive allow-list (LLR-uus6ar) or the devCsp-versus-csp comparison that R1 added. The text is not wrong, but it does not record the two strengthenings, while the REQ and decomposition do.
disposition: fixed (R2). §3 names the directive allow-list (LLR-uus6ar) and the comparison of devCsp with csp (LLR-tc5aax).

## Gaps

- **The CSP is not proved to load at runtime.** No gate can show that the
  bundled app still renders under the policy. The policy relies on Tauri
  appending hashes of SvelteKit's inline bootstrap script at build time.
  - The round-1 reviewer read Tauri 2.11.5's `set_csp` and confirmed that it
    does so.
  - The owner ran `npm --prefix app run tauri build` on 2026-10-05. The
    compile and the `.app` bundle succeeded; the DMG step then failed in
    `bundle_dmg.sh`, a bundling fault unrelated to the CSP.
  - The `.app` was not launched. **The owner directed the change to proceed
    without the smoke test (2026-10-05).** A blank window or a CSP console
    error at the first launch is the signal.
- **Coverage: a class C shortfall, accepted by the owner (2026-10-05) for this
  merge only.** Neither `app` nor `org-node` configures a `coverage_command`,
  so statement and decision coverage are unmeasured in both. Ratchet tooth 5
  still owes it.
- **SDD-6g3wnh, the deviation item, carries no LLRs.** Its code is not
  measured here, and no gated test reaches it. Two seams are what is
  missing:
  - a test seam for `start_receiver`;
  - a `MockChainOps`-style service seam in the app harness, without which
    nothing a command hands to the service can be observed.
  - The onboarding REQs are owed by the Gap-20 first-admission change (owner
    ruling 3).
- **No Rust advisory scan.** cargo-audit and cargo-deny are not installed.
  - npm audit found devalue 5.8.1 (high), booked as PR-8vh53d. It is not in
    today's `build/`.
- **Two defects reported here belong to org-node, and this change books only
  their app side.**
  - PR-7zz76t: REQ-vgr7s2's premise is false. org-node's join request carries
    an address whenever an endpoint is bound. This also makes HAZ-n97v5g's
    wording false; the hazard text was not edited.
  - In Loopback with no peer address, org-node's `revoke_member` submits the
    chain update before it refuses the missing address
    (`org-node/src/service.rs:1129`, `:1173-1176`). It was found by reading
    the code, not by running it, and is recorded as context in PR-uxj7bg. It
    has no org-node report; that is the owner's call.
- **The inherent-safety control for HAZ-e2zupz was not minted.** It would give
  each command a Tauri capability in place of `core:default`.
- **The test that reads `lib.rs` has known limits** (LLR-4wcyqy). It compares
  command names, not registration order or attributes, and it does not parse
  block comments or string literals.

## Appendix: the falsifiability sweep (task T5)

Copied from the sweep report. Test names in the raw logs predate the cleanup pass's renames, which the report notes.

Every one of the 45 low-level requirements in
`app/docs/architecture/DRAFT-worktree-guardrails-app-arch-decomposition.md` was
split into clauses that can each be falsified on their own. Each clause got
one source mutation, and the gated suite was run against it.

**Method.** Driver `scratchpad/appt5-mut.py`, specs `appt5-spec-{a,b,c,d}.py`.
Raw records are in `scratchpad/appt5-results.jsonl`, with one full log per run
in `appt5-log-<id>.txt`. For each run, the driver:

1. copied the target file to a pristine copy and hashed it;
2. applied one textual substitution, whose anchor had to occur exactly once;
3. ran the gate;
4. restored the file from the pristine copy;
5. proved the restore byte-identical by SHA-256.

All 266 runs restored identically. `git status` on the task worktree was
clean after the sweep.

The gate for each kind of run:

- a Rust source mutation ran the full gated cargo line (all eight targets,
  `--no-fail-fast`);
- a `tauri.conf.json` or checker mutation ran `csp_policy`;
- a TypeScript mutation ran `npm run test` (vitest, all four files).

**Verdicts.** RED means at least one named test failed. GREEN means none did.
GREEN is a verdict about this search, not about the clause. Each GREEN clause
was cut from its requirement or narrowed, and a note under that requirement in
the draft names the run.

**Reused runs.** The `T3-M*` rows are T3's characterization mutations from
`appt3-mut-results.jsonl`, reused where a clause matches. T3 ran only the
named test target, so their failing-test lists can be partial. Each file's
SHA-256 at T3 equals its SHA-256 here, except `csp_policy.rs`, which changed
after T3. T3's M22 on that file was therefore rerun as T5-D47. T4's mutation
(the `incoming-verified` rename, run with a test filter) was rerun on the full
suite as T5-B73.

**Renamed tests.** After the sweep, four `ipc` tests were renamed to what they
observe. The table uses the new names; the raw logs carry the old ones.
`export_invite_hands_a_well_formed_org_id_to_the_service` is now
`export_invite_does_not_refuse_a_well_formed_org_id`;
`revoke_member_hands_an_encoded_endpoint_addr_to_the_service` is now
`revoke_member_does_not_refuse_an_encoded_endpoint_addr`;
`admit_member_hands_a_32_byte_or_absent_org_secret_to_the_service` is now
`admit_member_does_not_refuse_a_32_byte_or_absent_org_secret`;
`admit_member_hands_a_decoded_node_addr_to_the_service` is now
`admit_member_does_not_refuse_a_decodable_node_addr`.

**Checker probes.** Fifteen runs mutate the CSP checker in
`tests/csp_policy.rs` rather than a clause of the policy: D05–D08, D14–D16,
D25, D26, D30, D31, D35, D38, D39 and D47. They test whether each abnormal
case isolates its check. Two came back GREEN:

- D07: the exactness check on `default-src` (`default-src *` is also refused
  by the network-origin check);
- D16: the remote-origin check in `script-src`, for the same reason.

Neither one is a clause GREEN. The clause still holds, shown by T5-D48 and
T5-D11.

### Counts

| | |
|---|---|
| Runs, T5 | 266 (A 107, B 73, C 38, D 48) |
| Runs, reused from T3 | 26 |
| Clauses | 277 |
| RED | 261 |
| GREEN, so cut or narrowed | 16 |
| Checker probes | 15 (13 RED, 2 GREEN) |
| LLRs touched by a cut | 10 |

How the clause count is formed:

- runs (266 + 26) less the 15 checker probes;
- plus one, because B03 also probes LLR-z66f27's "claimable by exactly one
  caller after a release";
- less one, because D48 re-probes D03's clause without D03's network
  addition.

### The 16 GREEN clauses and what was done

| Run | LLR | Clause | Action |
|---|---|---|---|
| T5-A05 | LLR-tpkmh3 | empty + opt-in gives `DEV_PASSPHRASE` | cut |
| T5-A09 | LLR-kfmng5 | override beats `DEV_DATA_DIR` under the opt-in | narrowed to "development defaults not enabled" |
| T5-A11 | LLR-kfmng5 | resolved path beats `DEV_DATA_DIR` under the opt-in | narrowed (same) |
| T5-A43 | LLR-kze6ak | the data dir is not normalised | cut ("not made absolute, no fallback" kept: A44, A45 RED) |
| T5-A46 | LLR-kze6ak | an empty endpoint gets no fallback | cut |
| T5-A47 | LLR-kze6ak | an empty contract address gets no fallback | cut |
| T5-A77 | LLR-ecaw34 | byte order of the twenty bytes | narrowed to "twenty bytes of the values they encode" (A78 RED) |
| T5-A85 | LLR-vzf8j2 | `admit_member` parses the org id first | narrowed to `export_invite` |
| T5-A86 | LLR-vzf8j2 | `revoke_member` parses the org id first | narrowed (same) |
| T5-A92 | LLR-ty85xv | a blank address hands the service none | cut |
| T5-A95 | LLR-n6twt7 | the decoded address is handed to the service | cut |
| T5-A96 | LLR-8krgzj | no secret handed when none supplied | narrowed to "does not refuse" |
| T5-A100 | LLR-8krgzj | the decoded secret is handed to the service | cut |
| T5-A102 | LLR-ctrfz4 | the id-only address is built from the device key | cut |
| T5-A104 | LLR-ctrfz4 | the decoded node address is handed to the service | cut |
| T5-D13 | LLR-hdvy6x | hash and nonce sources are admitted | cut |

Eight of the sixteen share one cause: A92, A95, A96, A100, A102, A104, A85
and A86. On `AppState::for_test`, every `admit_member` and `revoke_member`
call fails at `find_org` (`OrgNotOnChain`) before the service uses the
address, the secret or the identifier. That is what the harness can reach
until a stored Organisation exists. It is the same limit that SDD-6g3wnh
records.

The derived-risk draft had three `assesses:` texts that relied on a cut
clause: LLR-n6twt7, LLR-8krgzj and LLR-ctrfz4. Each now says which step is
read from the code rather than shown by a test.

### Per-run attestations

| Run | LLR | Clause | Mutation | Verdict | Failing tests | SHA-256 before = after |
|---|---|---|---|---|---|---|
| T5-A01 | LLR-tpkmh3 | a non-empty configured passphrase is returned (no opt-in) | src/policy.rs: `Some(p) if !p.is_empty() => Ok(p.to_string()),` → `Some(p) if !p.is_empty() && allow_dev_defaults => Ok(p.to_string()),` | RED | configured_passphrase_is_used | 5136e084b83f ✓ |
| T5-A02 | LLR-tpkmh3 | a non-empty configured passphrase is returned with the opt-in too | src/policy.rs: `Some(p) if !p.is_empty() => Ok(p.to_string()),` → `Some(p) if !p.is_empty() && !allow_dev_defaults => Ok(p.to_string()),` | RED | configured_passphrase_wins_over_opt_in | 5136e084b83f ✓ |
| T5-A03 | LLR-tpkmh3 | the configured passphrase is returned unaltered | src/policy.rs: `Some(p) if !p.is_empty() => Ok(p.to_string()),` → `Some(p) if !p.is_empty() => Ok(p.trim().to_uppercase()),` | RED | configured_passphrase_is_used, configured_passphrase_wins_over_opt_in | 5136e084b83f ✓ |
| T5-A04 | LLR-tpkmh3 | absent + opt-in returns DEV_PASSPHRASE | src/policy.rs: `_ if allow_dev_defaults => Ok(DEV_PASSPHRASE.to_string()),` → `_ if allow_dev_defaults => Ok(String::from("dev")),` | RED | absent_passphrase_with_opt_in_uses_dev_default | 5136e084b83f ✓ |
| T5-A05 | LLR-tpkmh3 | empty + opt-in returns DEV_PASSPHRASE | src/policy.rs: `Some(p) if !p.is_empty() => Ok(p.to_string()),` → `Some(p) if !p.is_empty() \|\| allow_dev_defaults => Ok(p.to_string()),` | GREEN | — | 5136e084b83f ✓ |
| T5-A06 | LLR-tpkmh3 | absent, no opt-in, is MissingPassphrase | src/policy.rs: `        _ => Err(StartupError::MissingPassphrase),` → `        None => Ok(String::new()),⏎        _ => Err(StartupError::MissingPassphrase),` | RED | passphrase_refusal_names_both_variables_supplier_first, absent_passphrase_is_refused | 5136e084b83f ✓ |
| T5-A07 | LLR-tpkmh3 | empty, no opt-in, is MissingPassphrase | src/policy.rs: `        _ => Err(StartupError::MissingPassphrase),` → `        Some(_) => Ok(String::new()),⏎        _ => Err(StartupError::MissingPassphrase),` | RED | empty_passphrase_is_refused_not_honoured, refusals_for_empty_values_name_both_variables_supplier_first | 5136e084b83f ✓ |
| T5-A08 | LLR-kfmng5 | a non-empty override wins over the resolved path | src/policy.rs: `if let Some(d) = override_dir.filter(\|d\| !d.is_empty()) {` → `if let Some(d) = override_dir.filter(\|d\| !d.is_empty() && resolved.is_none()) {` | RED | override_data_dir_is_used | 5136e084b83f ✓ |
| T5-A09 | LLR-kfmng5 | a non-empty override wins over the dev default | src/policy.rs: `if let Some(d) = override_dir.filter(\|d\| !d.is_empty()) {` → `if let Some(d) = override_dir.filter(\|d\| !d.is_empty() && !allow_dev_defaults) {` | GREEN | — | 5136e084b83f ✓ |
| T5-A10 | LLR-kfmng5 | the resolved path is used when there is no override | src/policy.rs: `if let Some(p) = resolved {` → `if let Some(p) = resolved.filter(\|_\| false) {` | RED | empty_override_is_ignored_not_used_as_path, resolved_data_dir_is_used_when_no_override | 5136e084b83f ✓ |
| T5-A11 | LLR-kfmng5 | the resolved path wins over the dev default | src/policy.rs: `if let Some(p) = resolved {` → `if let Some(p) = resolved.filter(\|_\| !allow_dev_defaults) {` | GREEN | — | 5136e084b83f ✓ |
| T5-A12 | LLR-kfmng5 | neither + opt-in returns DEV_DATA_DIR | src/policy.rs: `return Ok(PathBuf::from(DEV_DATA_DIR));` → `return Ok(std::env::temp_dir());` | RED | unresolvable_data_dir_with_opt_in_uses_temp | 5136e084b83f ✓ |
| T5-A13 | LLR-kfmng5 | neither, no opt-in, is UnresolvableDataDir | src/policy.rs: `    Err(StartupError::UnresolvableDataDir)⏎}` → `    Ok(PathBuf::from(DEV_DATA_DIR))⏎}` | RED | empty_override_is_ignored_not_used_as_path, refusals_for_empty_values_name_both_variables_supplier_first, data_dir_refusal_names_both_variables_supplier_first, unresolvable_data_dir_is_refused | 5136e084b83f ✓ |
| T5-A14 | LLR-kfmng5 | an empty override counts as absent | src/policy.rs: `override_dir.filter(\|d\| !d.is_empty())` → `override_dir.filter(\|_\| true)` | RED | empty_override_is_ignored_not_used_as_path, refusals_for_empty_values_name_both_variables_supplier_first | 5136e084b83f ✓ |
| T5-A16 | LLR-97rww8 | the passphrase refusal names ODS_ALLOW_DEV_DEFAULTS | src/policy.rs: `development passphrase instead, set ODS_ALLOW_DEV_DEFAULTS=1` → `development passphrase instead, set the dev flag` | RED | passphrase_refusal_names_both_variables_supplier_first, refusals_for_empty_values_name_both_variables_supplier_first | 5136e084b83f ✓ |
| T5-A17 | LLR-97rww8 | the passphrase refusal names the supplier first | src/policy.rs: `"ODS_PASSPHRASE is not set. Set it` → `"ODS_ALLOW_DEV_DEFAULTS=1 waives this. ODS_PASSPHRASE is not set. Set it` | RED | passphrase_refusal_names_both_variables_supplier_first, refusals_for_empty_values_name_both_variables_supplier_first | 5136e084b83f ✓ |
| T5-A18 | LLR-97rww8 | the data-dir refusal names ODS_DATA_DIR | src/policy.rs: `ODS_DATA_DIR is not set. Set ODS_DATA_DIR to` → `the override is not set. Set it to` | RED | data_dir_refusal_names_both_variables_supplier_first, refusals_for_empty_values_name_both_variables_supplier_first | 5136e084b83f ✓ |
| T5-A19 | LLR-97rww8 | the data-dir refusal names ODS_ALLOW_DEV_DEFAULTS | src/policy.rs: `ODS_ALLOW_DEV_DEFAULTS=1 — that directory` → `the dev flag — that directory` | RED | data_dir_refusal_names_both_variables_supplier_first, refusals_for_empty_values_name_both_variables_supplier_first | 5136e084b83f ✓ |
| T5-A20 | LLR-97rww8 | the data-dir refusal names the supplier first | src/policy.rs: `"The application data directory could not be resolved and \` → `"ODS_ALLOW_DEV_DEFAULTS=1 waives this. The application data directory could not be resolved and \` | RED | data_dir_refusal_names_both_variables_supplier_first, refusals_for_empty_values_name_both_variables_supplier_first | 5136e084b83f ✓ |
| T5-A21 | LLR-97rww8 | an empty passphrase gets the same naming refusal | src/policy.rs: `Some(p) if !p.is_empty() => Ok(p.to_string()),` → `Some("") if !allow_dev_defaults => Err(StartupError::UnresolvableDataDir),⏎        Some(p) if !p.is_empty()…` | RED | empty_passphrase_is_refused_not_honoured, refusals_for_empty_values_name_both_variables_supplier_first | 5136e084b83f ✓ |
| T5-A22 | LLR-97rww8 | an empty data-dir override gets the same naming refusal | src/policy.rs: `    if let Some(d) = override_dir.filter(\|d\| !d.is_empty()) {` → `    if override_dir == Some("") && resolved.is_none() && !allow_dev_defaults {⏎        return Err(StartupEr…` | RED | empty_override_is_ignored_not_used_as_path, refusals_for_empty_values_name_both_variables_supplier_first | 5136e084b83f ✓ |
| T5-A23 | LLR-j5pacp | exactly 'loopback' is Loopback | src/policy.rs: `Some("loopback") => TransportModeName::Loopback,` → `Some("loop") => TransportModeName::Loopback,` | RED | loopback_transport_is_reported | 5136e084b83f ✓ |
| T5-A24 | LLR-j5pacp | an absent value is Networked | src/policy.rs: `Some("loopback") => TransportModeName::Loopback,` → `Some("loopback") \| None => TransportModeName::Loopback,` | RED | networked_transport_is_reported | 5136e084b83f ✓ |
| T5-A25 | LLR-j5pacp | an empty value is Networked | src/policy.rs: `Some("loopback") => TransportModeName::Loopback,` → `Some("loopback") \| Some("") => TransportModeName::Loopback,` | RED | unknown_transport_value_is_networked | 5136e084b83f ✓ |
| T5-A26 | LLR-j5pacp | 'LOOPBACK' is Networked (case-sensitive) | src/policy.rs: `Some("loopback") => TransportModeName::Loopback,` → `Some(s) if s.eq_ignore_ascii_case("loopback") => TransportModeName::Loopback,` | RED | unknown_transport_value_is_networked | 5136e084b83f ✓ |
| T5-A27 | LLR-j5pacp | every other value is Networked | src/policy.rs: `Some("loopback") => TransportModeName::Loopback,` → `Some(s) if s.starts_with("loop") => TransportModeName::Loopback,` | RED | unknown_transport_value_is_networked | 5136e084b83f ✓ |
| T5-A28 | LLR-85zque | assemble creates the data directory | src/state.rs: `std::fs::create_dir_all(&data_dir)⏎` → `std::fs::metadata(&data_dir).map(\|_\| ())⏎` | RED | connection_status_reports_a_data_dir_it_had_to_create_verbatim, the_store_is_opened_inside_the_data_dir_it_creates | 601198b3c8ce ✓ |
| T5-A29 | LLR-85zque | with any missing parents | src/state.rs: `std::fs::create_dir_all(&data_dir)⏎` → `(if data_dir.is_dir() { Ok(()) } else { std::fs::create_dir(&data_dir) })⏎` | RED | connection_status_reports_a_data_dir_it_had_to_create_verbatim, the_store_is_opened_inside_the_data_dir_it_creates | 601198b3c8ce ✓ |
| T5-A30 | LLR-85zque | the store is opened under the given passphrase | src/state.rs: `PersonaStore::open(data_dir.join("persona_store.bin"), passphrase)` → `PersonaStore::open(data_dir.join("persona_store.bin"), "fixed")` | RED | a_store_under_another_passphrase_is_refused | 601198b3c8ce ✓ |
| T5-A31 | LLR-85zque | the create refusal begins 'create data_dir' | src/state.rs: `format!("create data_dir {}: {e}", data_dir.display())` → `format!("mkdir {}: {e}", data_dir.display())` | RED | a_data_dir_that_cannot_be_created_is_refused_naming_it | 601198b3c8ce ✓ |
| T5-A32 | LLR-85zque | the create refusal names the path | src/state.rs: `format!("create data_dir {}: {e}", data_dir.display())` → `format!("create data_dir {}: {e}", "the data directory")` | RED | a_data_dir_that_cannot_be_created_is_refused_naming_it | 601198b3c8ce ✓ |
| T5-A33 | LLR-85zque | the open refusal begins 'open store: ' | src/state.rs: `.map_err(\|e\| format!("open store: {e}"))?;` → `.map_err(\|e\| format!("store: {e}"))?;` | RED | a_store_under_another_passphrase_is_refused | 601198b3c8ce ✓ |
| T5-A34 | LLR-vqkr5t | connection_status reports exactly AppState.data_dir | src/commands.rs: `        &state.data_dir,⏎` → `        &state.data_dir.join("x"),⏎` | RED | connection_status_reports_a_data_dir_it_had_to_create_verbatim, connection_status_reports_the_directory_the_store_was_opened_in | a7314e9b5605 ✓ |
| T5-A35 | LLR-9x6qrg | chain_configured is true when an endpoint is handed | src/policy.rs: `chain_configured: chain.is_some(),` → `chain_configured: false,` | RED | configured_chain_reports_its_endpoint_and_contract | 5136e084b83f ✓ |
| T5-A36 | LLR-9x6qrg | chain_configured is false when none is | src/policy.rs: `chain_configured: chain.is_some(),` → `chain_configured: true,` | RED | absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env, unconfigured_chain_reports_neither | 5136e084b83f ✓ |
| T5-A37 | LLR-9x6qrg | chain_ws is reported when handed | src/policy.rs: `chain_ws: chain.map(\|c\| c.ws_url.clone()),` → `chain_ws: None,` | RED | endpoint_comes_from_the_built_configuration, configured_chain_reports_its_endpoint_and_contract | 5136e084b83f ✓ |
| T5-A38 | LLR-9x6qrg | contract_h160 is reported when handed | src/policy.rs: `contract_h160: chain.map(\|c\| c.contract_h160.clone()),` → `contract_h160: None,` | RED | endpoint_comes_from_the_built_configuration, configured_chain_reports_its_endpoint_and_contract | 5136e084b83f ✓ |
| T5-A39 | LLR-9x6qrg | chain_ws is absent when no endpoint | src/policy.rs: `chain_ws: chain.map(\|c\| c.ws_url.clone()),` → `chain_ws: Some(chain.map(\|c\| c.ws_url.clone()).unwrap_or_default()),` | RED | absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env, unconfigured_chain_reports_neither | 5136e084b83f ✓ |
| T5-A40 | LLR-9x6qrg | contract_h160 is absent when no endpoint | src/policy.rs: `contract_h160: chain.map(\|c\| c.contract_h160.clone()),` → `contract_h160: Some(chain.map(\|c\| c.contract_h160.clone()).unwrap_or_default()),` | RED | absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env, unconfigured_chain_reports_neither | 5136e084b83f ✓ |
| T5-A41 | LLR-kze6ak | the endpoint is reported verbatim (not normalised) | src/policy.rs: `chain.map(\|c\| c.ws_url.clone())` → `chain.map(\|c\| format!("{}/", c.ws_url.trim_end_matches('/')))` | RED | configured_chain_reports_its_endpoint_and_contract, endpoint_comes_from_the_built_configuration | 5136e084b83f ✓ |
| T5-A42 | LLR-kze6ak | the contract address is reported verbatim | src/policy.rs: `c.contract_h160.clone()` → `c.contract_h160.trim_start_matches("0x").to_string()` | RED | configured_chain_reports_its_endpoint_and_contract | 5136e084b83f ✓ |
| T5-A43 | LLR-kze6ak | the data dir is not normalised | src/policy.rs: `data_dir: data_dir.display().to_string(),` → `data_dir: data_dir.components().collect::<std::path::PathBuf>().display().to_string(),` | GREEN | — | 5136e084b83f ✓ |
| T5-A44 | LLR-kze6ak | the data dir is not made absolute | src/policy.rs: `data_dir: data_dir.display().to_string(),` → `data_dir: std::path::absolute(data_dir).unwrap_or_default().display().to_string(),` | RED | abnormal_data_dir_paths_are_reported_verbatim | 5136e084b83f ✓ |
| T5-A45 | LLR-kze6ak | an empty data dir is not replaced by a fallback | src/policy.rs: `data_dir: data_dir.display().to_string(),` → `data_dir: if data_dir.as_os_str().is_empty() { DEV_DATA_DIR.to_string() } else { data_dir.display().to_stri…` | RED | abnormal_data_dir_paths_are_reported_verbatim | 5136e084b83f ✓ |
| T5-A46 | LLR-kze6ak | an empty endpoint is not replaced by a fallback | src/policy.rs: `chain.map(\|c\| c.ws_url.clone())` → `chain.map(\|c\| if c.ws_url.is_empty() { "ws://127.0.0.1:9944".to_string() } else { c.ws_url.clone() })` | GREEN | — | 5136e084b83f ✓ |
| T5-A47 | LLR-kze6ak | an empty contract address is not replaced by a fallback | src/policy.rs: `c.contract_h160.clone()` → `if c.contract_h160.is_empty() { "0x".to_string() } else { c.contract_h160.clone() }` | GREEN | — | 5136e084b83f ✓ |
| T5-A48 | LLR-8tzbzn | the command is registered under 'connection_status' | tests/ipc.rs: `            commands::connection_status,⏎` → `` | RED | connection_status_reports_a_data_dir_it_had_to_create_verbatim, connection_status_reports_the_directory_the_store_was_opened_in, connection_status_is_registered_under_its_name, connection_status_reports_transport_mode_over_ipc | 411e24256336 ✓ |
| T5-A49 | LLR-8tzbzn | it returns 'chain_configured' | src/policy.rs: `    pub chain_configured: bool,` → `    #[serde(rename = "chainConfigured")]⏎    pub chain_configured: bool,` | RED | connection_status_is_registered_under_its_name | 5136e084b83f ✓ |
| T5-A50 | LLR-8tzbzn | it returns 'chain_ws' | src/policy.rs: `    pub chain_ws: Option<String>,` → `    #[serde(rename = "chainWs")]⏎    pub chain_ws: Option<String>,` | RED | connection_status_is_registered_under_its_name | 5136e084b83f ✓ |
| T5-A51 | LLR-8tzbzn | it returns 'contract_h160' | src/policy.rs: `    pub contract_h160: Option<String>,` → `    #[serde(rename = "contractH160")]⏎    pub contract_h160: Option<String>,` | RED | connection_status_is_registered_under_its_name | 5136e084b83f ✓ |
| T5-A52 | LLR-8tzbzn | it returns 'transport_mode' | src/policy.rs: `    pub transport_mode: TransportModeName,` → `    #[serde(rename = "transportMode")]⏎    pub transport_mode: TransportModeName,` | RED | transport_mode_serialises_lowercase, connection_status_reports_transport_mode_over_ipc, connection_status_is_registered_under_its_name | 5136e084b83f ✓ |
| T5-A53 | LLR-8tzbzn | it returns 'data_dir' | src/policy.rs: `    pub data_dir: String,` → `    #[serde(rename = "dataDir")]⏎    pub data_dir: String,` | RED | connection_status_is_registered_under_its_name, connection_status_reports_the_directory_the_store_was_opened_in, connection_status_reports_a_data_dir_it_had_to_create_verbatim | 5136e084b83f ✓ |
| T5-A54 | LLR-8tzbzn | transport_mode is the configured mode | src/commands.rs: `        state.transport_mode,⏎    ))` → `        policy::TransportModeName::Networked,⏎    ))` | RED | connection_status_reports_transport_mode_over_ipc | a7314e9b5605 ✓ |
| T5-A55 | LLR-8tzbzn | the mode serialises as 'networked'/'loopback' | src/policy.rs: `#[serde(rename_all = "lowercase")]` → `#[serde(rename_all = "UPPERCASE")]` | RED | transport_mode_serialises_lowercase, connection_status_reports_transport_mode_over_ipc | 5136e084b83f ✓ |
| T5-A56 | LLR-4wcyqy | the commands are registered under their snake_case names | tests/ipc.rs: `            commands::list_personas,⏎` → `` | RED | create_persona_stores_the_parsed_details, a_persona_in_no_organisation_reports_a_null_org_id, create_persona_refuses_an_invalid_field_naming_it_and_creates_nothing, list_personas_returns_an_empty_list_on_a_fresh_store, import_join_request_reports_the_request_it_decodes, list_personas_reports_exactly_the_persona_fields, import_join_request_refuses_a_malformed_blob | 411e24256336 ✓ |
| T5-A57 | LLR-4wcyqy | an invocation of any other name is refused | tests/ipc.rs: `    let app = mock_builder()` → `    #[tauri::command]⏎    #[allow(non_snake_case)]⏎    fn connectionStatus() {}⏎    let app = mock_builder()` ; `            commands::start_receiver,⏎` → `            commands::start_receiver,⏎            connectionStatus,⏎` | RED | unknown_command_is_rejected | 411e24256336 ✓ |
| T5-A58 | LLR-pguhw5 | no other field (no seed) crosses IPC | src/commands.rs: `    pub status: String,⏎}` → `    pub status: String,⏎    pub seed: String,⏎}` ; `            status: format!("{:?}", p.status),⏎` → `            status: format!("{:?}", p.status),⏎            seed: hex::encode([0u8; 32]),⏎` | RED | list_personas_reports_exactly_the_persona_fields | a7314e9b5605 ✓ |
| T5-A59 | LLR-pguhw5 | field 'org_id' | src/commands.rs: `    pub org_id: Option<String>,⏎    pub handle: String,` → `    #[serde(rename = "orgId")]⏎    pub org_id: Option<String>,⏎    pub handle: String,` | RED | a_persona_in_no_organisation_reports_a_null_org_id, list_personas_reports_exactly_the_persona_fields | a7314e9b5605 ✓ |
| T5-A60 | LLR-pguhw5 | field 'handle' | src/commands.rs: `    pub org_id: Option<String>,⏎    pub handle: String,` → `    pub org_id: Option<String>,⏎    #[serde(rename = "handle_")]⏎    pub handle: String,` | RED | create_persona_stores_the_parsed_details, list_personas_reports_exactly_the_persona_fields | a7314e9b5605 ✓ |
| T5-A61 | LLR-pguhw5 | field 'name' | src/commands.rs: `    pub handle: String,⏎    pub name: String,⏎    pub surname: String,⏎    pub status: String,` → `    pub handle: String,⏎    #[serde(rename = "firstName")]⏎    pub name: String,⏎    pub surname: String,⏎ …` | RED | create_persona_stores_the_parsed_details, list_personas_reports_exactly_the_persona_fields | a7314e9b5605 ✓ |
| T5-A62 | LLR-pguhw5 | field 'surname' | src/commands.rs: `    pub surname: String,⏎    pub status: String,` → `    #[serde(rename = "lastName")]⏎    pub surname: String,⏎    pub status: String,` | RED | list_personas_reports_exactly_the_persona_fields | a7314e9b5605 ✓ |
| T5-A63 | LLR-pguhw5 | field 'status' | src/commands.rs: `    pub status: String,⏎}` → `    #[serde(rename = "state")]⏎    pub status: String,⏎}` | RED | list_personas_reports_exactly_the_persona_fields | a7314e9b5605 ✓ |
| T5-A64 | LLR-pguhw5 | 'org_id' is present (as null) for a persona in no Organisation | src/commands.rs: `    pub org_id: Option<String>,⏎    pub handle: String,` → `    #[serde(skip_serializing_if = "Option::is_none")]⏎    pub org_id: Option<String>,⏎    pub handle: String,` | RED | a_persona_in_no_organisation_reports_a_null_org_id, list_personas_reports_exactly_the_persona_fields | a7314e9b5605 ✓ |
| T5-A65 | LLR-pmus9f | it stores nothing | src/commands.rs: `pub async fn import_join_request(blob: String) -> Result<JoinRequestDto, String> {⏎    let jr = decode_join…` → `pub async fn import_join_request(state: State<'_, AppState>, blob: String) -> Result<JoinRequestDto, String…` | RED | import_join_request_reports_the_request_it_decodes | a7314e9b5605 ✓ |
| T5-A66 | LLR-pmus9f | it returns the handle | src/commands.rs: `handle: jr.handle.to_string(),` → `handle: jr.name.to_string(),` | RED | import_join_request_reports_the_request_it_decodes | a7314e9b5605 ✓ |
| T5-A67 | LLR-pmus9f | it returns the name | src/commands.rs: `name: jr.name.to_string(),` → `name: jr.surname.to_string(),` | RED | import_join_request_reports_the_request_it_decodes | a7314e9b5605 ✓ |
| T5-A68 | LLR-pmus9f | it returns the surname | src/commands.rs: `surname: jr.surname.to_string(),` → `surname: jr.handle.to_string(),` | RED | import_join_request_reports_the_request_it_decodes | a7314e9b5605 ✓ |
| T5-A69 | LLR-pmus9f | it returns the member key as hex | src/commands.rs: `member_key: hex::encode(jr.member_key.as_bytes()),` → `member_key: hex::encode(jr.device_key.as_bytes()),` | RED | import_join_request_reports_the_request_it_decodes | a7314e9b5605 ✓ |
| T5-A70 | LLR-pmus9f | it returns the device key as hex | src/commands.rs: `device_key: hex::encode(jr.device_key.as_bytes()),` → `device_key: hex::encode(jr.member_key.as_bytes()),` | RED | import_join_request_reports_the_request_it_decodes | a7314e9b5605 ✓ |
| T5-A71 | LLR-pmus9f | has_node_addr is false when the request carries none | src/commands.rs: `has_node_addr: !jr.node_addr.is_empty(),` → `has_node_addr: true,` | RED | import_join_request_reports_the_request_it_decodes | a7314e9b5605 ✓ |
| T5-A72 | LLR-pmus9f | it returns the node address as hex | src/commands.rs: `node_addr_blob: hex::encode(&jr.node_addr),` → `node_addr_blob: String::new(),` | RED | import_join_request_reports_the_request_it_decodes | a7314e9b5605 ✓ |
| T5-A73 | LLR-ecaw34 | a leading '0x' is stripped | src/parsing.rs: `let s = s.strip_prefix("0x").unwrap_or(s);` → `let s = s;` | RED | export_invite_does_not_refuse_a_well_formed_org_id, a_doubled_zero_x_prefix_is_refused, a_zero_x_prefix_is_accepted_and_parses_identically, a_zero_x_prefix_with_forty_characters_after_it_is_accepted, the_bare_prefix_alone_is_refused | 11d7ce87853d ✓ |
| T5-A74 | LLR-ecaw34 | at most one '0x' is stripped | src/parsing.rs: `let s = s.strip_prefix("0x").unwrap_or(s);` → `let s = s.trim_start_matches("0x");` | RED | a_doubled_zero_x_prefix_is_refused | 11d7ce87853d ✓ |
| T5-A75 | LLR-ecaw34 | exactly forty characters are accepted (longer refused) | src/parsing.rs: `if s.len() != 40 {` → `if s.len() < 40 {` | RED | a_doubled_zero_x_prefix_is_refused, forty_one_characters_are_refused | 11d7ce87853d ✓ |
| T5-A76 | LLR-ecaw34 | uppercase hex is accepted | src/parsing.rs: `    let bytes = hex::decode(s)` → `    if s.chars().any(\|c\| c.is_ascii_uppercase()) {⏎        return Err("org_id hex: uppercase".into());⏎  …` | RED | uppercase_hex_is_accepted | 11d7ce87853d ✓ |
| T5-A77 | LLR-ecaw34 | the twenty bytes they encode (order) | src/parsing.rs: `    arr.copy_from_slice(&bytes);⏎` → `    arr.copy_from_slice(&bytes);⏎    arr.reverse();⏎` | GREEN | — | 11d7ce87853d ✓ |
| T5-A78 | LLR-ecaw34 | the twenty bytes they encode (values) | src/parsing.rs: `    arr.copy_from_slice(&bytes);⏎` → `    arr.copy_from_slice(&bytes);⏎    arr[0] ^= 1;⏎` | RED | uppercase_hex_is_accepted, a_zero_x_prefix_is_accepted_and_parses_identically, a_zero_x_prefix_with_forty_characters_after_it_is_accepted, forty_hex_characters_are_accepted | 11d7ce87853d ✓ |
| T5-A79 | LLR-ecaw34 | the width refusal reads 'org_id must be 40 hex chars, got N' | src/parsing.rs: `"org_id must be 40 hex chars, got {}"` → `"org_id must be 40 characters, got {}"` | RED | export_invite_rejects_a_short_org_id, an_odd_length_is_refused_on_width_not_on_hex_decoding, a_doubled_zero_x_prefix_is_refused, the_bare_prefix_alone_is_refused, an_empty_string_is_refused | 11d7ce87853d ✓ |
| T5-A80 | LLR-ecaw34 | N is the length after the strip | src/parsing.rs: `    let s = s.strip_prefix("0x").unwrap_or(s);` → `    let raw = s;⏎    let s = s.strip_prefix("0x").unwrap_or(s);` ; `got {}", s.len()` → `got {}", raw.len()` | RED | a_doubled_zero_x_prefix_is_refused, the_bare_prefix_alone_is_refused | 11d7ce87853d ✓ |
| T5-A81 | LLR-ecaw34 | the width is checked before the alphabet | src/parsing.rs: `    if s.len() != 40 {` → `    hex::decode(s).map_err(\|e\| format!("org_id hex: {e}"))?;⏎    if s.len() != 40 {` | RED | export_invite_rejects_a_short_org_id, an_odd_length_is_refused_on_width_not_on_hex_decoding, forty_one_characters_are_refused, a_doubled_zero_x_prefix_is_refused, thirty_nine_characters_are_refused | 11d7ce87853d ✓ |
| T5-A82 | LLR-ecaw34 | a non-hex character is refused with 'org_id hex' | src/parsing.rs: `format!("org_id hex: {e}")` → `format!("invalid: {e}")` | RED | export_invite_rejects_a_non_hex_org_id, a_non_hex_character_at_the_first_position_is_refused, a_non_hex_character_at_the_last_position_is_refused, forty_non_hex_characters_are_refused, a_zero_x_in_the_middle_is_refused | 11d7ce87853d ✓ |
| T5-A83 | LLR-vzf8j2 | export_invite refuses a malformed id before the service | src/commands.rs: `let oid = parse_org_id(&org_id)?;⏎    let svc = state.service.lock().await;` → `let oid = parse_org_id(&org_id).unwrap_or(org_node::OrgId::new([0xaa; 20]));⏎    let svc = state.service.lo…` | RED | export_invite_rejects_a_short_org_id, export_invite_rejects_a_non_hex_org_id | a7314e9b5605 ✓ |
| T5-A84 | LLR-vzf8j2 | the refusal is the parser's message | src/commands.rs: `let oid = parse_org_id(&org_id)?;⏎    let svc = state.service.lock().await;` → `let oid = parse_org_id(&org_id).map_err(\|_\| String::from("bad organisation"))?;⏎    let svc = state.servi…` | RED | export_invite_rejects_a_non_hex_org_id, export_invite_rejects_a_short_org_id | a7314e9b5605 ✓ |
| T5-A85 | LLR-vzf8j2 | admit_member parses the org id before the service | src/commands.rs: `let oid = parse_org_id(&org_id)?;⏎    let jr = decode_join_request` → `let oid = parse_org_id(&org_id).unwrap_or(org_node::OrgId::new([0xaa; 20]));⏎    let jr = decode_join_request` | GREEN | — | a7314e9b5605 ✓ |
| T5-A86 | LLR-vzf8j2 | revoke_member parses the org id before the service | src/commands.rs: `let oid = parse_org_id(&org_id)?;⏎    let member_bytes` → `let oid = parse_org_id(&org_id).unwrap_or(org_node::OrgId::new([0xaa; 20]));⏎    let member_bytes` | GREEN | — | a7314e9b5605 ✓ |
| T5-A87 | LLR-6pmrma | at most one '0x' is stripped from the member id | src/commands.rs: `hex::decode(member_id_hex.strip_prefix("0x").unwrap_or(&member_id_hex))` → `hex::decode(member_id_hex.trim_start_matches("0x"))` | RED | revoke_member_rejects_a_doubled_zero_x_prefix_on_the_member_id | a7314e9b5605 ✓ |
| T5-A88 | LLR-6pmrma | non-hex is refused with 'member_id hex' | src/commands.rs: `format!("member_id hex: {e}")` → `format!("member_id: {e}")` | RED | revoke_member_rejects_a_doubled_zero_x_prefix_on_the_member_id | a7314e9b5605 ✓ |
| T5-A89 | LLR-6pmrma | a length other than 32 bytes is refused | src/commands.rs: `    if member_bytes.len() != 32 {` → `    let member_bytes = {⏎        let mut b = member_bytes;⏎        b.resize(32, 0);⏎        b⏎    };⏎    if…` | RED | revoke_member_rejects_a_short_member_id | a7314e9b5605 ✓ |
| T5-A90 | LLR-6pmrma | the width refusal reads 'member_id must be 32 bytes (64 hex chars)' | src/commands.rs: `"member_id must be 32 bytes (64 hex chars)"` → `"member_id has the wrong length"` | RED | revoke_member_rejects_a_short_member_id | a7314e9b5605 ✓ |
| T5-A91 | LLR-ty85xv | an empty peer address is not refused | src/commands.rs: `if trimmed.is_empty() {` → `if trimmed.is_empty() && false {` | RED | revoke_member_is_not_refused_for_an_empty_peer_addr, revoke_member_accepts_a_zero_x_prefixed_member_id, revoke_member_treats_a_whitespace_only_peer_addr_as_absent | a7314e9b5605 ✓ |
| T5-A92 | LLR-ty85xv | a blank peer address hands the service no address | src/commands.rs: `    let peer_addr: Option<EndpointAddr> = if trimmed.is_empty() {⏎        None⏎` → `    let peer_addr: Option<EndpointAddr> = if trimmed.is_empty() {⏎        Some(iroh::SecretKey::from_bytes(…` | GREEN | — | a7314e9b5605 ✓ |
| T5-A93 | LLR-n6twt7 | non-hex is refused with 'peer_addr_blob hex:' | src/commands.rs: `format!("peer_addr_blob hex: {e}")` → `format!("peer_addr hex: {e}")` | RED | revoke_member_refuses_a_peer_addr_that_is_not_hex | a7314e9b5605 ✓ |
| T5-A94 | LLR-n6twt7 | a non-EndpointAddr is refused with 'peer_addr decode:' | src/commands.rs: `format!("peer_addr decode: {e}")` → `format!("peer_addr postcard: {e}")` | RED | revoke_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr | a7314e9b5605 ✓ |
| T5-A95 | LLR-n6twt7 | the decoded address is what is handed to the service | src/commands.rs: `Some(postcard::from_bytes(&addr_bytes).map_err(\|e\| format!("peer_addr decode: {e}"))?)` → `{⏎            let _a: EndpointAddr = postcard::from_bytes(&addr_bytes).map_err(\|e\| format!("peer_addr dec…` | GREEN | — | a7314e9b5605 ✓ |
| T5-A96 | LLR-8krgzj | no secret is handed when none is supplied | src/commands.rs: `        None => None,` → `        None => Some(OrgSecret::from([0u8; 32])),` | GREEN | — | a7314e9b5605 ✓ |
| T5-A97 | LLR-8krgzj | non-hex is refused with 'org_secret hex:' | src/commands.rs: `format!("org_secret hex: {e}")` → `format!("org_secret: {e}")` | RED | admit_member_refuses_an_org_secret_that_is_not_hex | a7314e9b5605 ✓ |
| T5-A98 | LLR-8krgzj | a length other than 32 bytes is refused | src/commands.rs: `            if bytes.len() != 32 {` → `            let bytes = {⏎                let mut b = bytes;⏎                b.resize(32, 0);⏎             …` | RED | admit_member_refuses_an_org_secret_that_is_not_32_bytes | a7314e9b5605 ✓ |
| T5-A99 | LLR-8krgzj | the width refusal reads 'org_secret must be 32 bytes' | src/commands.rs: `"org_secret must be 32 bytes".into()` → `"org_secret has the wrong length".into()` | RED | admit_member_refuses_an_org_secret_that_is_not_32_bytes | a7314e9b5605 ✓ |
| T5-A100 | LLR-8krgzj | the decoded secret is what is handed to the service | src/commands.rs: `Some(OrgSecret::from(arr))` → `Some(OrgSecret::from([0u8; 32]))` | GREEN | — | a7314e9b5605 ✓ |
| T5-A101 | LLR-ctrfz4 | no node address takes the id-only branch | src/commands.rs: `let peer_addr: EndpointAddr = if jr.node_addr.is_empty() {` → `let peer_addr: EndpointAddr = if false {` | RED | admit_member_does_not_refuse_a_32_byte_or_absent_org_secret, admit_member_refuses_an_org_secret_that_is_not_32_bytes, admit_member_refuses_an_org_secret_that_is_not_hex | a7314e9b5605 ✓ |
| T5-A102 | LLR-ctrfz4 | the id-only address is built from the device key | src/commands.rs: `iroh::EndpointId::from_bytes(jr.device_key.as_bytes())` → `iroh::EndpointId::from_bytes(jr.member_key.as_bytes())` | GREEN | — | a7314e9b5605 ✓ |
| T5-A103 | LLR-ctrfz4 | the refusal begins 'node_addr decode:' | src/commands.rs: `format!("node_addr decode: {e}")` → `format!("node_addr: {e}")` | RED | admit_member_refuses_a_node_addr_that_is_not_an_endpoint_addr | a7314e9b5605 ✓ |
| T5-A104 | LLR-ctrfz4 | the decoded address is what is handed to the service | src/commands.rs: `        postcard::from_bytes(&jr.node_addr)⏎            .map_err(\|e\| format!("node_addr decode: {e}"))?⏎` → `        {⏎            let _a: EndpointAddr = postcard::from_bytes(&jr.node_addr)⏎                .map_err(\…` | GREEN | — | a7314e9b5605 ✓ |
| T5-A105 | LLR-p7dfxb | a refusal names the field | src/commands.rs: `format!("{}: {reason}", field.trim_start_matches("persona."))` → `format!("{reason}")` | RED | create_persona_refuses_an_invalid_field_naming_it_and_creates_nothing | a7314e9b5605 ✓ |
| T5-A106 | LLR-p7dfxb | without org-node's 'persona.' prefix | src/commands.rs: `field.trim_start_matches("persona.")` → `field` | RED | create_persona_refuses_an_invalid_field_naming_it_and_creates_nothing | a7314e9b5605 ✓ |
| T5-A107 | LLR-p7dfxb | a refusal creates nothing | src/commands.rs: `    let PersonaDetails { handle, name, surname } =⏎` → `    if PersonaDetails::parse(&handle, &name, &surname).is_err() {⏎        if let Ok(d) = PersonaDetails::pa…` | RED | create_persona_refuses_an_invalid_field_naming_it_and_creates_nothing | a7314e9b5605 ✓ |
| T5-A108 | LLR-p7dfxb | the parsed details are stored, each in its field | src/commands.rs: `PersonaDetails::parse(&handle, &name, &surname).map_err` → `PersonaDetails::parse(&handle, &surname, &name).map_err` | RED | create_persona_stores_the_parsed_details, create_persona_refuses_an_invalid_field_naming_it_and_creates_nothing, import_join_request_reports_the_request_it_decodes | a7314e9b5605 ✓ |
| T5-B01 | LLR-5zd6j8 | a guard is returned while none is held | src/events.rs: `flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)⏎            .ok()` → `flag.compare_exchange(true, true, Ordering::AcqRel, Ordering::Acquire)⏎            .ok()` | RED | first_claim_succeeds, release_then_concurrent_claims_yield_exactly_one_guard, guard_releases_when_dropped_by_panic, claim_succeeds_again_after_the_guard_drops, second_claim_while_held_fails, concurrent_claims_yield_exactly_one_guard | 921a7bf45f8b ✓ |
| T5-B02 | LLR-5zd6j8 | none is returned while one is held | src/events.rs: `flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)⏎            .ok()` → `Some(flag.swap(true, Ordering::AcqRel))` | RED | second_claim_while_held_fails, release_then_concurrent_claims_yield_exactly_one_guard, concurrent_claims_yield_exactly_one_guard | 921a7bf45f8b ✓ |
| T5-B03 | LLR-5zd6j8 | exactly one guard under concurrent calls (also LLR-z66f27: after a release) | src/events.rs: `flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)⏎            .ok()` → `(!flag.load(Ordering::Acquire)).then(\|\| {⏎                std::thread::yield_now();⏎                flag.…` | RED | concurrent_claims_yield_exactly_one_guard, release_then_concurrent_claims_yield_exactly_one_guard | 921a7bf45f8b ✓ |
| T5-B04 | LLR-z66f27 | dropping the guard releases the slot | src/events.rs: `self.0.store(false, Ordering::Release);` → `let _ = &self.0;` | RED | claim_succeeds_again_after_the_guard_drops, guard_releases_when_dropped_by_panic, release_then_concurrent_claims_yield_exactly_one_guard | 921a7bf45f8b ✓ |
| T5-B05 | LLR-z66f27 | including when dropped by a panic | src/events.rs: `self.0.store(false, Ordering::Release);` → `if !std::thread::panicking() {⏎            self.0.store(false, Ordering::Release);⏎        }` | RED | guard_releases_when_dropped_by_panic | 921a7bf45f8b ✓ |
| T5-B06 | LLR-7bk6qh | OrgIdMismatch is VerifyFailed | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::OrgIdMismatch => ReceiverOutcome::ReceiveError { message: e.to_string()…` | RED | every_verification_verdict_is_classified_as_a_verification_failure | 921a7bf45f8b ✓ |
| T5-B07 | LLR-7bk6qh | BadSignature is VerifyFailed | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::BadSignature => ReceiverOutcome::ReceiveError { message: e.to_string() },⏎` | RED | a_non_terminal_failure_does_not_stop_the_loop, every_verification_verdict_is_classified_as_a_verification_failure | 921a7bf45f8b ✓ |
| T5-B08 | LLR-7bk6qh | StaleSeq is VerifyFailed | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::StaleSeq { .. } => ReceiverOutcome::ReceiveError { message: e.to_string…` | RED | every_verification_verdict_is_classified_as_a_verification_failure | 921a7bf45f8b ✓ |
| T5-B09 | LLR-7bk6qh | MalformedDelta is VerifyFailed | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::MalformedDelta => ReceiverOutcome::ReceiveError { message: e.to_string(…` | RED | every_verification_verdict_is_classified_as_a_verification_failure | 921a7bf45f8b ✓ |
| T5-B10 | LLR-7bk6qh | DeltaBaseMismatch is VerifyFailed | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::DeltaBaseMismatch => ReceiverOutcome::ReceiveError { message: e.to_stri…` | RED | every_verification_verdict_is_classified_as_a_verification_failure | 921a7bf45f8b ✓ |
| T5-B11 | LLR-7bk6qh | RootMismatch is VerifyFailed | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::RootMismatch => ReceiverOutcome::ReceiveError { message: e.to_string() },⏎` | RED | every_verification_verdict_is_classified_as_a_verification_failure, the_two_classes_are_actually_distinguished | 921a7bf45f8b ✓ |
| T5-B12 | LLR-7bk6qh | StaleEpoch is VerifyFailed | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::StaleEpoch { .. } => ReceiverOutcome::ReceiveError { message: e.to_stri…` | RED | every_verification_verdict_is_classified_as_a_verification_failure | 921a7bf45f8b ✓ |
| T5-B13 | LLR-7bk6qh | VerifyFailed carries the error's message | src/events.rs: `            org_id: None,⏎            message: e.to_string(),` → `            org_id: None,⏎            message: String::new(),` | RED | a_non_terminal_failure_does_not_stop_the_loop, a_verification_verdict_carries_its_own_message_and_no_invented_organisation | 921a7bf45f8b ✓ |
| T5-B14 | LLR-7bk6qh | VerifyFailed carries no organisation | src/events.rs: `            org_id: None,⏎            message: e.to_string(),` → `            org_id: Some(String::from("unknown")),⏎            message: e.to_string(),` | RED | a_verification_verdict_carries_its_own_message_and_no_invented_organisation, a_non_terminal_failure_does_not_stop_the_loop | 921a7bf45f8b ✓ |
| T5-B15 | LLR-7bk6qh | Chain is ReceiveError | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::Chain(_) => ReceiverOutcome::VerifyFailed { org_id: None, message: e.to…` | RED | a_non_terminal_failure_does_not_stop_the_loop, a_chain_failure_is_classified_as_a_receiver_error, a_chain_failure_emits_no_verification_event_for_any_message, the_stop_is_announced_after_the_failure_for_the_same_error, the_two_classes_are_actually_distinguished | 921a7bf45f8b ✓ |
| T5-B16 | LLR-7bk6qh | OrgNotOnChain is ReceiveError | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::OrgNotOnChain => ReceiverOutcome::VerifyFailed { org_id: None, message:…` | RED | locally_reachable_variants_are_classified_as_receiver_errors | 921a7bf45f8b ✓ |
| T5-B17 | LLR-7bk6qh | Trie is ReceiveError | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::Trie(_) => ReceiverOutcome::VerifyFailed { org_id: None, message: e.to_…` | RED | locally_reachable_variants_are_classified_as_receiver_errors | 921a7bf45f8b ✓ |
| T5-B18 | LLR-7bk6qh | InvalidKey is ReceiveError | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::InvalidKey => ReceiverOutcome::VerifyFailed { org_id: None, message: e.…` | RED | a_refused_key_or_field_is_classified_as_a_receiver_error | 921a7bf45f8b ✓ |
| T5-B19 | LLR-7bk6qh | InvalidField is ReceiveError | src/events.rs: `    match e {⏎` → `    match e {⏎        OrgNodeError::InvalidField { .. } => ReceiverOutcome::VerifyFailed { org_id: None, me…` | RED | a_refused_key_or_field_is_classified_as_a_receiver_error | 921a7bf45f8b ✓ |
| T5-B20 | LLR-7bk6qh | ReceiveError carries the error's message | src/events.rs: `ReceiverOutcome::ReceiveError {⏎            message: e.to_string(),⏎        },` → `ReceiverOutcome::ReceiveError {⏎            message: String::new(),⏎        },` | RED | a_non_terminal_failure_does_not_stop_the_loop, locally_reachable_variants_are_classified_as_receiver_errors, a_chain_failure_is_classified_as_a_receiver_error, a_refused_key_or_field_is_classified_as_a_receiver_error, the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B21 | LLR-usxk57 | exactly one event | src/events.rs: `        ReceiverOutcome::ReceiveError { message } => vec![Emission {⏎            name: "receiver-error",⏎  …` → `        ReceiverOutcome::ReceiveError { message } => vec![Emission {⏎            name: "receiver-error",⏎  …` | RED | a_chain_failure_is_classified_as_a_receiver_error, locally_reachable_variants_are_classified_as_receiver_errors, a_chain_failure_emits_no_verification_event_for_any_message, the_stop_is_announced_after_the_failure_for_the_same_error, the_receiver_error_payload_carries_a_message_and_no_verification_state, the_two_classes_are_actually_distinguished | 921a7bf45f8b ✓ |
| T5-B22 | LLR-usxk57 | named 'receiver-error' | src/events.rs: `name: "receiver-error",` → `name: "receiver-failed",` | RED | a_chain_failure_emits_no_verification_event_for_any_message, a_chain_failure_is_classified_as_a_receiver_error, locally_reachable_variants_are_classified_as_receiver_errors, the_receiver_error_payload_carries_a_message_and_no_verification_state, the_stop_is_announced_after_the_failure_for_the_same_error, the_two_classes_are_actually_distinguished | 921a7bf45f8b ✓ |
| T5-B23 | LLR-usxk57 | the payload carries the message | src/events.rs: `payload: json(&MessageOnly { message }),` → `payload: json(&MessageOnly { message: "" }),` | RED | a_chain_failure_is_classified_as_a_receiver_error, the_stop_is_announced_after_the_failure_for_the_same_error, the_receiver_error_payload_carries_a_message_and_no_verification_state | 921a7bf45f8b ✓ |
| T5-B24 | LLR-usxk57 | no organisation | src/events.rs: `payload: json(&MessageOnly { message }),` → `payload: json(&VerifyFailed { org_id: None, message }),` | RED | locally_reachable_variants_are_classified_as_receiver_errors, the_receiver_error_payload_carries_a_message_and_no_verification_state | 921a7bf45f8b ✓ |
| T5-B25 | LLR-usxk57 | no epoch | src/events.rs: `payload: json(&MessageOnly { message }),` → `payload: serde_json::json!({ "message": message, "epoch": 0 }),` | RED | the_receiver_error_payload_carries_a_message_and_no_verification_state | 921a7bf45f8b ✓ |
| T5-B26 | LLR-usxk57 | no root | src/events.rs: `payload: json(&MessageOnly { message }),` → `payload: serde_json::json!({ "message": message, "root": "" }),` | RED | the_receiver_error_payload_carries_a_message_and_no_verification_state | 921a7bf45f8b ✓ |
| T5-B27 | LLR-usxk57 | no verdict | src/events.rs: `payload: json(&MessageOnly { message }),` → `payload: serde_json::json!({ "message": message, "verified": false }),` | RED | the_receiver_error_payload_carries_a_message_and_no_verification_state | 921a7bf45f8b ✓ |
| T5-B28 | LLR-usxk57 | at any depth | src/events.rs: `payload: json(&MessageOnly { message }),` → `payload: serde_json::json!({ "message": message, "detail": { "epoch": 0 } }),` | RED | the_receiver_error_payload_carries_a_message_and_no_verification_state | 921a7bf45f8b ✓ |
| T5-B29 | LLR-a9rjtf | the class outcome is returned | src/events.rs: `let mut out = vec![classify_receive_error(e)];` → `let mut out: Vec<ReceiverOutcome> = vec![];` | RED | a_non_terminal_failure_does_not_stop_the_loop, the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B30 | LLR-a9rjtf | 'endpoint bind:' stops | src/events.rs: `    "endpoint bind:",⏎` → `    "endpoint bind: x",⏎` | RED | every_real_terminal_message_stops_the_loop, the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B31 | LLR-a9rjtf | 'endpoint bind failed unexpectedly' stops | src/events.rs: `    "endpoint bind failed unexpectedly",⏎` → `    "endpoint bind failed unexpectedly!",⏎` | RED | every_real_terminal_message_stops_the_loop, the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B32 | LLR-a9rjtf | 'endpoint closed' stops | src/events.rs: `    "endpoint closed",⏎` → `    "endpoint closed!",⏎` | RED | every_real_terminal_message_stops_the_loop, the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B33 | LLR-a9rjtf | the fragment may appear anywhere in the message | src/events.rs: `message.contains(marker)` → `message.starts_with(marker)` | RED | every_real_terminal_message_stops_the_loop, the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B34 | LLR-a9rjtf | Stopped follows the class outcome | src/events.rs: `out.push(ReceiverOutcome::Stopped { reason: message });` → `out.insert(0, ReceiverOutcome::Stopped { reason: message });` | RED | the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B35 | LLR-a9rjtf | the reason is that message | src/events.rs: `out.push(ReceiverOutcome::Stopped { reason: message });` → `out.push(ReceiverOutcome::Stopped { reason: String::new() });` | RED | every_real_terminal_message_stops_the_loop, the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B36 | LLR-a9rjtf | otherwise nothing else | src/events.rs: `if is_terminal_error(&message) {` → `if true {` | RED | a_non_terminal_failure_does_not_stop_the_loop | 921a7bf45f8b ✓ |
| T5-B37 | LLR-2zmhvs | exactly one event | src/events.rs: `        ReceiverOutcome::Stopped { reason } => vec![Emission {⏎            name: "receiver-stopped",⏎      …` → `        ReceiverOutcome::Stopped { reason } => vec![Emission {⏎            name: "receiver-stopped",⏎      …` | RED | stopped_emits_exactly_one_event, stopped_names_the_reason, the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B38 | LLR-2zmhvs | named 'receiver-stopped' | src/events.rs: `name: "receiver-stopped",` → `name: "receiver-stop",` | RED | stopped_emits_exactly_one_event, stopped_names_the_reason, the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B39 | LLR-2zmhvs | the payload carries the reason | src/events.rs: `payload: json(&Stopped { reason }),` → `payload: json(&Stopped { reason: "stopped" }),` | RED | stopped_names_the_reason, the_stop_is_announced_after_the_failure_for_the_same_error | 921a7bf45f8b ✓ |
| T5-B40 | LLR-2zmhvs | even when the reason is empty | src/events.rs: `        ReceiverOutcome::Stopped { reason } => vec![Emission {` → `        ReceiverOutcome::Stopped { reason } if reason.is_empty() => vec![],⏎        ReceiverOutcome::Stoppe…` | RED | stopped_emits_exactly_one_event | 921a7bf45f8b ✓ |
| T5-B41 | LLR-p2nm5a | membership-updated before incoming-verified | src/events.rs: `name: "membership-updated",` → `name: "@@TMP@@",` ; `name: "incoming-verified",` → `name: "membership-updated",` ; `name: "@@TMP@@",` → `name: "incoming-verified",` | RED | updated_carries_boundary_epochs_and_roots_verbatim, updated_emits_membership_incoming_and_epoch | 921a7bf45f8b ✓ |
| T5-B42 | LLR-p2nm5a | epoch-changed last | src/events.rs: `name: "incoming-verified",` → `name: "@@TMP@@",` ; `name: "epoch-changed",⏎                payload: json(&EpochChanged { org_id, epoch: *epoch }),` → `name: "incoming-verified",⏎                payload: json(&MembershipUpdated { org_id, epoch: *epoch, root }),` ; `name: "@@TMP@@",⏎                payload: json(&MembershipUpdated { org_id, epoch: *epoch, root }),` → `name: "epoch-changed",⏎                payload: json(&EpochChanged { org_id, epoch: *epoch }),` | RED | updated_emits_membership_incoming_and_epoch, updated_carries_boundary_epochs_and_roots_verbatim | 921a7bf45f8b ✓ |
| T5-B43 | LLR-p2nm5a | a 'membership-updated' event | src/events.rs: `name: "membership-updated",` → `name: "membership-changed",` | RED | updated_carries_boundary_epochs_and_roots_verbatim, updated_emits_membership_incoming_and_epoch | 921a7bf45f8b ✓ |
| T5-B44 | LLR-p2nm5a | an 'epoch-changed' event | src/events.rs: `name: "epoch-changed",` → `name: "epoch-updated",` | RED | updated_emits_membership_incoming_and_epoch, updated_carries_boundary_epochs_and_roots_verbatim | 921a7bf45f8b ✓ |
| T5-B45 | LLR-p2nm5a | membership-updated carries the organisation | src/events.rs: `                name: "membership-updated",⏎                payload: json(&MembershipUpdated { org_id, epoc…` → `                name: "membership-updated",⏎                payload: json(&MembershipUpdated { org_id: "", …` | RED | updated_payload_carries_the_measured_epoch_and_root | 921a7bf45f8b ✓ |
| T5-B46 | LLR-p2nm5a | membership-updated carries the epoch | src/events.rs: `                name: "membership-updated",⏎                payload: json(&MembershipUpdated { org_id, epoc…` → `                name: "membership-updated",⏎                payload: json(&MembershipUpdated { org_id, epoc…` | RED | updated_payload_carries_the_measured_epoch_and_root, updated_carries_boundary_epochs_and_roots_verbatim | 921a7bf45f8b ✓ |
| T5-B47 | LLR-p2nm5a | membership-updated carries the root | src/events.rs: `                name: "membership-updated",⏎                payload: json(&MembershipUpdated { org_id, epoc…` → `                name: "membership-updated",⏎                payload: json(&MembershipUpdated { org_id, epoc…` | RED | updated_carries_boundary_epochs_and_roots_verbatim, updated_payload_carries_the_measured_epoch_and_root | 921a7bf45f8b ✓ |
| T5-B48 | LLR-p2nm5a | incoming-verified carries the organisation | src/events.rs: `                name: "incoming-verified",⏎                payload: json(&MembershipUpdated { org_id, epoch…` → `                name: "incoming-verified",⏎                payload: json(&MembershipUpdated { org_id: "", e…` | RED | updated_payload_carries_the_measured_epoch_and_root | 921a7bf45f8b ✓ |
| T5-B49 | LLR-p2nm5a | incoming-verified carries the epoch | src/events.rs: `                name: "incoming-verified",⏎                payload: json(&MembershipUpdated { org_id, epoch…` → `                name: "incoming-verified",⏎                payload: json(&MembershipUpdated { org_id, epoch…` | RED | updated_carries_boundary_epochs_and_roots_verbatim, updated_payload_carries_the_measured_epoch_and_root | 921a7bf45f8b ✓ |
| T5-B50 | LLR-p2nm5a | incoming-verified carries the root | src/events.rs: `                name: "incoming-verified",⏎                payload: json(&MembershipUpdated { org_id, epoch…` → `                name: "incoming-verified",⏎                payload: json(&MembershipUpdated { org_id, epoch…` | RED | updated_carries_boundary_epochs_and_roots_verbatim | 921a7bf45f8b ✓ |
| T5-B51 | LLR-p2nm5a | epoch-changed carries the organisation | src/events.rs: `payload: json(&EpochChanged { org_id, epoch: *epoch }),` → `payload: json(&EpochChanged { org_id: "", epoch: *epoch }),` | RED | updated_payload_carries_the_measured_epoch_and_root | 921a7bf45f8b ✓ |
| T5-B52 | LLR-p2nm5a | epoch 0 carried verbatim (membership-updated) | src/events.rs: `                name: "membership-updated",⏎                payload: json(&MembershipUpdated { org_id, epoc…` → `                name: "membership-updated",⏎                payload: json(&MembershipUpdated { org_id, epoc…` | RED | updated_carries_boundary_epochs_and_roots_verbatim | 921a7bf45f8b ✓ |
| T5-B53 | LLR-p2nm5a | an empty root carried verbatim | src/events.rs: `                name: "membership-updated",⏎                payload: json(&MembershipUpdated { org_id, epoc…` → `                name: "membership-updated",⏎                payload: json(&MembershipUpdated { org_id, epoc…` | RED | updated_carries_boundary_epochs_and_roots_verbatim | 921a7bf45f8b ✓ |
| T5-B54 | LLR-hgsdm8 | exactly one event | src/events.rs: `        ReceiverOutcome::RecordUnreadable { org_id } => vec![Emission {⏎            name: "record-unreadabl…` → `        ReceiverOutcome::RecordUnreadable { org_id } => vec![Emission {⏎            name: "record-unreadabl…` | RED | record_unreadable_holds_for_any_organisation_id, record_unreadable_names_the_organisation, record_unreadable_emits_exactly_one_event | 921a7bf45f8b ✓ |
| T5-B55 | LLR-hgsdm8 | named 'record-unreadable' | src/events.rs: `name: "record-unreadable",` → `name: "record-missing",` | RED | record_unreadable_holds_for_any_organisation_id, record_unreadable_names_the_organisation | 921a7bf45f8b ✓ |
| T5-B56 | LLR-hgsdm8 | the payload names the organisation | src/events.rs: `            name: "record-unreadable",⏎            payload: json(&OrgOnly { org_id }),` → `            name: "record-unreadable",⏎            payload: json(&OrgOnly { org_id: "" }),` | RED | record_unreadable_holds_for_any_organisation_id, record_unreadable_names_the_organisation | 921a7bf45f8b ✓ |
| T5-B57 | LLR-hgsdm8 | the organisation alone | src/events.rs: `            name: "record-unreadable",⏎            payload: json(&OrgOnly { org_id }),` → `            name: "record-unreadable",⏎            payload: serde_json::json!({ "org_id": org_id, "note": "…` | RED | record_unreadable_holds_for_any_organisation_id | 921a7bf45f8b ✓ |
| T5-B58 | LLR-hgsdm8 | no membership event | src/events.rs: `        ReceiverOutcome::RecordUnreadable { org_id } => vec![Emission {⏎            name: "record-unreadabl…` → `        ReceiverOutcome::RecordUnreadable { org_id } => vec![Emission {⏎            name: "record-unreadabl…` | RED | record_unreadable_emits_no_membership_event, record_unreadable_names_the_organisation, record_unreadable_holds_for_any_organisation_id, record_unreadable_emits_exactly_one_event, record_unreadable_emits_no_epoch_event | 921a7bf45f8b ✓ |
| T5-B59 | LLR-hgsdm8 | no verification event | src/events.rs: `        ReceiverOutcome::RecordUnreadable { org_id } => vec![Emission {⏎            name: "record-unreadabl…` → `        ReceiverOutcome::RecordUnreadable { org_id } => vec![Emission {⏎            name: "record-unreadabl…` | RED | record_unreadable_emits_exactly_one_event, record_unreadable_holds_for_any_organisation_id, record_unreadable_names_the_organisation | 921a7bf45f8b ✓ |
| T5-B60 | LLR-hgsdm8 | no epoch event | src/events.rs: `        ReceiverOutcome::RecordUnreadable { org_id } => vec![Emission {⏎            name: "record-unreadabl…` → `        ReceiverOutcome::RecordUnreadable { org_id } => vec![Emission {⏎            name: "record-unreadabl…` | RED | record_unreadable_holds_for_any_organisation_id, record_unreadable_emits_no_epoch_event, record_unreadable_emits_exactly_one_event, record_unreadable_emits_no_membership_event, record_unreadable_names_the_organisation | 921a7bf45f8b ✓ |
| T5-B61 | LLR-2vg79y | exactly one event | src/events.rs: `        ReceiverOutcome::SelfDeleted { org_id } => vec![Emission {⏎            name: "revoked",⏎           …` → `        ReceiverOutcome::SelfDeleted { org_id } => vec![Emission {⏎            name: "revoked",⏎           …` | RED | self_delete_emits_revoked_naming_the_organisation | 921a7bf45f8b ✓ |
| T5-B62 | LLR-2vg79y | named 'revoked' | src/events.rs: `name: "revoked",` → `name: "self-deleted",` | RED | self_delete_emits_revoked_naming_the_organisation | 921a7bf45f8b ✓ |
| T5-B63 | LLR-2vg79y | naming the organisation | src/events.rs: `            name: "revoked",⏎            payload: json(&OrgOnly { org_id }),` → `            name: "revoked",⏎            payload: json(&OrgOnly { org_id: "" }),` | RED | self_delete_emits_revoked_naming_the_organisation, self_delete_carries_no_epoch_for_any_organisation_id | 921a7bf45f8b ✓ |
| T5-B64 | LLR-2vg79y | no epoch key | src/events.rs: `            name: "revoked",⏎            payload: json(&OrgOnly { org_id }),` → `            name: "revoked",⏎            payload: json(&EpochChanged { org_id, epoch: 0 }),` | RED | self_delete_payload_carries_no_epoch_key_at_any_depth, self_delete_emits_no_epoch_event, self_delete_carries_no_epoch_for_any_organisation_id, the_epoch_probe_finds_an_epoch_on_updated_and_none_on_self_delete | 921a7bf45f8b ✓ |
| T5-B65 | LLR-2vg79y | no epoch key at any depth | src/events.rs: `            name: "revoked",⏎            payload: json(&OrgOnly { org_id }),` → `            name: "revoked",⏎            payload: serde_json::json!({ "org_id": org_id, "state": { "epoch":…` | RED | self_delete_payload_carries_no_epoch_key_at_any_depth, self_delete_carries_no_epoch_for_any_organisation_id, the_epoch_probe_finds_an_epoch_on_updated_and_none_on_self_delete | 921a7bf45f8b ✓ |
| T5-B66 | LLR-2vg79y | whatever the organisation identifier | src/events.rs: `            name: "revoked",⏎            payload: json(&OrgOnly { org_id }),` → `            name: "revoked",⏎            payload: if org_id.is_empty() { serde_json::json!({ "org_id": org_…` | RED | self_delete_carries_no_epoch_for_any_organisation_id | 921a7bf45f8b ✓ |
| T5-B67 | LLR-p38be7 | exactly one event | src/events.rs: `        ReceiverOutcome::VerifyFailed { org_id, message } => vec![Emission {⏎            name: "verificatio…` → `        ReceiverOutcome::VerifyFailed { org_id, message } => vec![Emission {⏎            name: "verificatio…` | RED | a_verification_verdict_carries_its_own_message_and_no_invented_organisation, every_verification_verdict_is_classified_as_a_verification_failure, the_two_classes_are_actually_distinguished, verify_failure_carries_the_organisation_when_known | 921a7bf45f8b ✓ |
| T5-B68 | LLR-p38be7 | named 'verification-failed' | src/events.rs: `name: "verification-failed",⏎            payload` → `name: "verify-failed",⏎            payload` | RED | every_verification_verdict_is_classified_as_a_verification_failure, the_two_classes_are_actually_distinguished, verify_failure_carries_the_organisation_when_known | 921a7bf45f8b ✓ |
| T5-B69 | LLR-p38be7 | carrying the message | src/events.rs: `payload: json(&VerifyFailed { org_id: org_id.as_deref(), message }),` → `payload: json(&VerifyFailed { org_id: org_id.as_deref(), message: "" }),` | RED | a_verification_verdict_carries_its_own_message_and_no_invented_organisation, verify_failure_carries_the_message | 921a7bf45f8b ✓ |
| T5-B70 | LLR-p38be7 | carrying the organisation | src/events.rs: `payload: json(&VerifyFailed { org_id: org_id.as_deref(), message }),` → `payload: json(&VerifyFailed { org_id: None, message }),` | RED | verify_failure_carries_the_organisation_when_known | 921a7bf45f8b ✓ |
| T5-B71 | LLR-p38be7 | present and null when the outcome names none | src/events.rs: `struct VerifyFailed<'a> { org_id: Option<&'a str>, message: &'a str }` → `struct VerifyFailed<'a> {⏎    #[serde(skip_serializing_if = "Option::is_none")]⏎    org_id: Option<&'a str>…` | RED | verify_failure_carries_null_organisation_when_unknown | 921a7bf45f8b ✓ |
| T5-B72 | LLR-p38be7 | no membership event with it | src/events.rs: `        ReceiverOutcome::VerifyFailed { org_id, message } => vec![Emission {⏎            name: "verificatio…` → `        ReceiverOutcome::VerifyFailed { org_id, message } => vec![Emission {⏎            name: "verificatio…` | RED | a_verification_verdict_carries_its_own_message_and_no_invented_organisation, every_verification_verdict_is_classified_as_a_verification_failure, the_two_classes_are_actually_distinguished, verify_failure_carries_the_organisation_when_known, verify_failure_emits_no_membership_event | 921a7bf45f8b ✓ |
| T5-B73 | LLR-p2nm5a | an 'incoming-verified' event (T4's mutation, rerun on the full suite) | src/events.rs: `name: "incoming-verified",` → `name: "incoming-verified-MUTATED",` | RED | updated_emits_membership_incoming_and_epoch, updated_carries_boundary_epochs_and_roots_verbatim | 921a7bf45f8b ✓ |
| T5-C01 | LLR-53hayh | a verified event's row has verified true | verify.ts: `verified: true,` → `verified: false,` | RED | renders a verified event as verified, leaves an existing verification log untouched | 0961c0a6a01e ✓ |
| T5-C02 | LLR-53hayh | the row has the event's epoch | verify.ts: `epoch: event.epoch,` → `epoch: event.epoch + 1,` | RED | renders a verified event as verified, preserves epoch 0 on a verified event rather than coercing it to null, files the newest announcement first in each list, keeps at most 50 verification rows, dropping the oldest | 0961c0a6a01e ✓ |
| T5-C03 | LLR-53hayh | including epoch 0 | verify.ts: `epoch: event.epoch,` → `epoch: event.epoch \|\| null,` | RED | preserves epoch 0 on a verified event rather than coercing it to null | 0961c0a6a01e ✓ |
| T5-C04 | LLR-53hayh | the row has the event's root | verify.ts: `root: event.root,` → `root: event.root.slice(0, 8),` | RED | renders a verified event as verified | 0961c0a6a01e ✓ |
| T5-C05 | LLR-53hayh | a null detail | verify.ts: `⇥⇥⇥detail: null,⏎⇥⇥⇥ts⏎` → `⇥⇥⇥detail: '',⏎⇥⇥⇥ts⏎` | RED | renders a verified event as verified, gives a verified event no detail | 0961c0a6a01e ✓ |
| T5-C06 | LLR-53hayh | the timestamp it was given, unmodified | verify.ts: `⇥⇥⇥detail: null,⏎⇥⇥⇥ts⏎` → `⇥⇥⇥detail: null,⏎⇥⇥⇥ts: ts.trim()⏎` | RED | passes the timestamp through unmodified | 0961c0a6a01e ✓ |
| T5-C07 | LLR-a4xwvj | a failed event's row has verified false | verify.ts: `verified: false,` → `verified: true,` | RED | renders a failed event as not verified, does put a verification failure in the log and not in the receiver errors | 0961c0a6a01e ✓ |
| T5-C08 | LLR-a4xwvj | null epoch | verify.ts: `⇥⇥epoch: null,⏎⇥⇥root: null,` → `⇥⇥epoch: 0,⏎⇥⇥root: null,` | RED | gives a failed event no epoch and no root | 0961c0a6a01e ✓ |
| T5-C09 | LLR-a4xwvj | null root | verify.ts: `⇥⇥epoch: null,⏎⇥⇥root: null,` → `⇥⇥epoch: null,⏎⇥⇥root: '',` | RED | gives a failed event no epoch and no root | 0961c0a6a01e ✓ |
| T5-C10 | LLR-a4xwvj | the event's message as detail | verify.ts: `detail: event.message,` → `detail: null,` | RED | carries the failure message as the row detail | 0961c0a6a01e ✓ |
| T5-C11 | LLR-a4xwvj | the timestamp it was given, unmodified | verify.ts: `⇥⇥detail: event.message,⏎⇥⇥ts⏎` → `⇥⇥detail: event.message,⏎⇥⇥ts: ts.trim()⏎` | RED | passes the timestamp through unmodified | 0961c0a6a01e ✓ |
| T5-C12 | LLR-mzae5q | '(unknown organisation)' when the event's is null | verify.ts: `org_id: event.org_id ?? '(unknown organisation)',` → `org_id: event.org_id ?? String(event.org_id),` | RED | renders a placeholder rather than "null" when the failure names no organisation | 0961c0a6a01e ✓ |
| T5-C13 | LLR-fb7jp5 | a receiver error is filed only in receiverErrors | verify.ts: `⇥⇥⇥verifyLog: view.verifyLog,⏎` → `⇥⇥⇥verifyLog: [verifyResultFrom({ kind: 'failed', org_id: null, message: event.message }, ts), ...view.veri…` | RED | produces no verification row for a receiver error, leaves an existing verification log untouched, assigns no verification outcome however many receiver errors arrive, files the newest announcement first in each list | 0961c0a6a01e ✓ |
| T5-C14 | LLR-fb7jp5 | as '[<timestamp>] <message>' | verify.ts: `'[${ts}] ${event.message}'` → `'${ts}: ${event.message}'` | RED | files the newest announcement first in each list, keeps at most 20 receiver errors, dropping the oldest | 0961c0a6a01e ✓ |
| T5-C15 | LLR-fb7jp5 | leaving the verification log unchanged | verify.ts: `⇥⇥⇥verifyLog: view.verifyLog,⏎` → `⇥⇥⇥verifyLog: view.verifyLog.slice(1),⏎` | RED | leaves an existing verification log untouched, files the newest announcement first in each list | 0961c0a6a01e ✓ |
| T5-C16 | LLR-fb7jp5 | a verified or failed event is filed in the log | verify.ts: `if (event.kind === 'receiver-error') {` → `if (event.kind === 'failed') {⏎⇥⇥return { verifyLog: view.verifyLog, receiverErrors: [event.message, ...vie…` | RED | does put a verification failure in the log and not in the receiver errors | 0961c0a6a01e ✓ |
| T5-C17 | LLR-fb7jp5 | leaving receiverErrors unchanged | verify.ts: `receiverErrors: view.receiverErrors⏎` → `receiverErrors: [...view.receiverErrors, ts]⏎` | RED | does put a verification failure in the log and not in the receiver errors, files the newest announcement first in each list | 0961c0a6a01e ✓ |
| T5-C18 | LLR-fb7jp5 | it does not modify the view (receiver-error path) | verify.ts: `if (event.kind === 'receiver-error') {` → `view.receiverErrors.unshift('');⏎⇥if (event.kind === 'receiver-error') {` | RED | renders a receiver error outside the verification log, assigns no verification outcome however many receiver errors arrive, does put a verification failure in the log and not in the receiver errors, does not mutate the view it was given, files the newest announcement first in each list, keeps at most 20 receiver errors, dropping the oldest | 0961c0a6a01e ✓ |
| T5-C19 | LLR-fb7jp5 | it does not modify the view (verification path) | verify.ts: `if (event.kind === 'receiver-error') {` → `if (event.kind !== 'receiver-error') view.verifyLog.unshift(verifyResultFrom(event, ts));⏎⇥if (event.kind =…` | RED | assigns no verification outcome however many receiver errors arrive, does put a verification failure in the log and not in the receiver errors, files the newest announcement first in each list, keeps at most 50 verification rows, dropping the oldest | 0961c0a6a01e ✓ |
| T5-C20 | LLR-rzx6ks | the verification log drops the oldest | verify.ts: `...view.verifyLog.slice(0, MAX_VERIFY_ROWS - 1)` → `...view.verifyLog.slice(-(MAX_VERIFY_ROWS - 1))` | RED | keeps at most 50 verification rows, dropping the oldest | 0961c0a6a01e ✓ |
| T5-C21 | LLR-rzx6ks | the receiver-error list drops the oldest | verify.ts: `...view.receiverErrors.slice(0, MAX_RECEIVER_ERRORS - 1)` → `...view.receiverErrors.slice(-(MAX_RECEIVER_ERRORS - 1))` | RED | keeps at most 20 receiver errors, dropping the oldest | 0961c0a6a01e ✓ |
| T5-C22 | LLR-z4ky6f | it registers every subscription it is given | receiver.ts: `subscriptions.map(([subscribe, handler])` → `subscriptions.slice(1).map(([subscribe, handler])` | RED | registers every subscription it is given, unsubscribes exactly the listeners it registered, including those whose listen call resolved late, resolves only after every subscription has registered, is idempotent — calling the cleanup twice unsubscribes once | 041be48d126e ✓ |
| T5-C23 | LLR-z4ky6f | it resolves only after all have registered | receiver.ts: `const unlisteners = await Promise.all(⏎⇥⇥subscriptions.map(([subscribe, handler]) => subscribe(handler as (…` → `const pending = Promise.all(⏎⇥⇥subscriptions.map(([subscribe, handler]) => subscribe(handler as (p: unknown…` ; `for (const u of unlisteners) u();` → `void pending.then((us) => us.forEach((u) => u()));` | RED | unsubscribes exactly the listeners it registered, including those whose listen call resolved late, resolves only after every subscription has registered, is idempotent — calling the cleanup twice unsubscribes once, propagates a subscription failure rather than resolving a partial cleanup, leaves a registered listener attached when another registration fails (PR-cu2h2g) | 041be48d126e ✓ |
| T5-C24 | LLR-z4ky6f | the cleanup cancels each registered listener | receiver.ts: `for (const u of unlisteners) u();` → `for (const u of unlisteners.slice(1)) u();` | RED | unsubscribes exactly the listeners it registered, including those whose listen call resolved late, is idempotent — calling the cleanup twice unsubscribes once | 041be48d126e ✓ |
| T5-C25 | LLR-z4ky6f | exactly once however many times it is called | receiver.ts: `if (done) return;` → `if (done && false) return;` | RED | is idempotent — calling the cleanup twice unsubscribes once | 041be48d126e ✓ |
| T5-C26 | LLR-z4ky6f | including listeners whose registration resolved late | receiver.ts: `subscribe(handler as (p: unknown) => void))` → `Promise.race([subscribe(handler as (p: unknown) => void), new Promise<Unlisten>((r) => setTimeout(() => r((…` | RED | unsubscribes exactly the listeners it registered, including those whose listen call resolved late, resolves only after every subscription has registered | 041be48d126e ✓ |
| T5-C27 | LLR-z4ky6f | when a registration fails it rejects with that failure | receiver.ts: `const unlisteners = await Promise.all(⏎⇥⇥subscriptions.map(([subscribe, handler]) => subscribe(handler as (…` → `const unlisteners = (await Promise.allSettled(⏎⇥⇥subscriptions.map(([subscribe, handler]) => subscribe(hand…` | RED | propagates a subscription failure rather than resolving a partial cleanup, leaves a registered listener attached when another registration fails (PR-cu2h2g) | 041be48d126e ✓ |
| T5-C28 | LLR-csbs5v | the member id is trimmed | revoke.ts: `const memberId = input.memberIdHex.trim().replace(/^0x/, '');` → `const memberId = input.memberIdHex.replace(/^0x/, '');` | RED | trims surrounding whitespace before measuring | 696ce3fdcb3e ✓ |
| T5-C29 | LLR-csbs5v | a leading '0x' is stripped | revoke.ts: `const memberId = input.memberIdHex.trim().replace(/^0x/, '');` → `const memberId = input.memberIdHex.trim();` | RED | accepts a 0x-prefixed member id, rejects a doubled 0x prefix | 696ce3fdcb3e ✓ |
| T5-C30 | LLR-csbs5v | a remainder that is not exactly 64 is refused | revoke.ts: `if (memberId.length !== 64) {` → `if (memberId.length < 64) {` | RED | rejects a 65-character member id, rejects a doubled 0x prefix | 696ce3fdcb3e ✓ |
| T5-C31 | LLR-csbs5v | the refusal names its length | revoke.ts: `got ${memberId.length}.` → `got the wrong length.` | RED | rejects a 63-character member id, rejects a 65-character member id, rejects an empty member id, rejects a doubled 0x prefix | 696ce3fdcb3e ✓ |
| T5-C32 | LLR-csbs5v | a non-hexadecimal remainder is refused | revoke.ts: `if (!/^[0-9a-fA-F]+$/.test(memberId)) {` → `if (false) {` | RED | rejects a 64-character non-hexadecimal member id | 696ce3fdcb3e ✓ |
| T5-C33 | LLR-c9r5uf | Loopback requires a peer address | revoke.ts: `if (input.transportMode === 'loopback' && input.peerAddrBlob.trim() === '') {` → `if (false) {` | RED | rejects an empty peer address in loopback transport, trims surrounding whitespace before measuring | 696ce3fdcb3e ✓ |
| T5-C34 | LLR-c9r5uf | a blank (whitespace) address counts as missing | revoke.ts: `if (input.transportMode === 'loopback' && input.peerAddrBlob.trim() === '') {` → `if (input.transportMode === 'loopback' && input.peerAddrBlob === '') {` | RED | trims surrounding whitespace before measuring | 696ce3fdcb3e ✓ |
| T5-C35 | LLR-c9r5uf | the refusal names Loopback | revoke.ts: `'Peer address is required in Loopback transport (same-machine dialling).'` → `'Peer address is required.'` | RED | rejects an empty peer address in loopback transport | 696ce3fdcb3e ✓ |
| T5-C36 | LLR-c9r5uf | Networked accepts no address | revoke.ts: `if (input.transportMode === 'loopback' && input.peerAddrBlob.trim() === '') {` → `if (input.peerAddrBlob.trim() === '') {` | RED | accepts an empty peer address in networked transport, accepts a 0x-prefixed member id, trims surrounding whitespace before measuring | 696ce3fdcb3e ✓ |
| T5-C37 | LLR-c9r5uf | Networked accepts a supplied address | revoke.ts: `if (input.transportMode === 'loopback' && input.peerAddrBlob.trim() === '') {` → `if (input.transportMode === 'networked' && input.peerAddrBlob !== '') {⏎⇥⇥return { ok: false, message: 'x' …` | RED | accepts a supplied peer address in networked transport | 696ce3fdcb3e ✓ |
| T5-C38 | LLR-c9r5uf | Loopback accepts a supplied address | revoke.ts: `if (input.transportMode === 'loopback' && input.peerAddrBlob.trim() === '') {` → `if (input.transportMode === 'loopback') {` | RED | accepts a supplied peer address in loopback transport | 696ce3fdcb3e ✓ |
| T5-D01 | LLR-7ymxtn | the shipped csp is not null | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": null,` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D02 | LLR-7ymxtn | the shipped csp is not empty | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "",` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D03 | LLR-7ymxtn | default-src is exactly 'self' | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self' https:; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D04 | LLR-7ymxtn | default-src is present | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src ipc: http://…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D05 | LLR-7ymxtn | checker: null refused | tests/csp_policy.rs: `.ok_or_else(\|\| format!("policy must be a string, found {value}"))?;` → `.unwrap_or("default-src 'self'; script-src 'self'; connect-src ipc:; object-src 'none'; base-uri 'none'; fr…` | RED | null_policy_is_rejected | 3e7abc0e4acd ✓ |
| T5-D06 | LLR-7ymxtn | checker: empty refused | tests/csp_policy.rs: `    let d = parse(policy)?;⏎` → `    if policy.is_empty() {⏎        return Ok(());⏎    }⏎    let d = parse(policy)?;⏎` | RED | empty_policy_is_rejected | 3e7abc0e4acd ✓ |
| T5-D07 | LLR-7ymxtn | checker: default-src other than 'self' refused | tests/csp_policy.rs: `if sources("default-src")? != &["'self'"] {` → `if sources("default-src")?.is_empty() {` | GREEN | — | 3e7abc0e4acd ✓ |
| T5-D08 | LLR-7ymxtn | checker: missing default-src refused | tests/csp_policy.rs: `if sources("default-src")? != &["'self'"] {` → `if sources("default-src").map_or(false, \|s\| s != &["'self'"]) {` | RED | default_src_other_than_self_is_rejected | 3e7abc0e4acd ✓ |
| T5-D09 | LLR-hdvy6x | script-src has no 'unsafe-inline' | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 's…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D10 | LLR-hdvy6x | script-src has no 'unsafe-eval' | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'sel…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D11 | LLR-hdvy6x | script-src has no remote origin | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self' https://cdn.example.org; style-src 'self' 'unsafe-inline'; im…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D12 | LLR-hdvy6x | script-src admits no other keyword | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D13 | LLR-hdvy6x | hash and nonce sources are admitted (checker) | tests/csp_policy.rs: `            \|\| is_hash_or_nonce(src)⏎` → `` | GREEN | — | 3e7abc0e4acd ✓ |
| T5-D14 | LLR-hdvy6x | checker: 'unsafe-inline' refused | tests/csp_policy.rs: `let allowed = src == "'self'"` → `let allowed = src == "'self'" \|\| src == "'unsafe-inline'"` | RED | unsafe_inline_in_script_src_is_rejected | 3e7abc0e4acd ✓ |
| T5-D15 | LLR-hdvy6x | checker: 'unsafe-eval' refused | tests/csp_policy.rs: `let allowed = src == "'self'"` → `let allowed = src == "'self'" \|\| src == "'unsafe-eval'"` | RED | unsafe_eval_in_script_src_is_rejected | 3e7abc0e4acd ✓ |
| T5-D16 | LLR-hdvy6x | checker: a remote origin in script-src refused (script-src check alone) | tests/csp_policy.rs: `let allowed = src == "'self'"` → `let allowed = src == "'self'" \|\| src.starts_with("https://")` | GREEN | — | 3e7abc0e4acd ✓ |
| T5-D17 | LLR-c88jhh | object-src is 'none' | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D18 | LLR-c88jhh | base-uri is 'none' | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D19 | LLR-c88jhh | frame-ancestors is 'none' | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D20 | LLR-c88jhh | form-action is 'none' | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D21 | LLR-c88jhh | object-src is present | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D22 | LLR-c88jhh | base-uri is present | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D23 | LLR-c88jhh | frame-ancestors is present | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D24 | LLR-c88jhh | form-action is present | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D25 | LLR-c88jhh | checker: each 'none' directive enforced | tests/csp_policy.rs: `        if sources(name)? != &["'none'"] {` → `        if sources(name)?.is_empty() {` | RED | each_none_directive_is_required_to_be_none | 3e7abc0e4acd ✓ |
| T5-D26 | LLR-c88jhh | checker: a missing 'none' directive refused | tests/csp_policy.rs: `        if sources(name)? != &["'none'"] {` → `        if sources(name).map_or(false, \|s\| s != &["'none'"]) {` | RED | missing_object_src_is_rejected | 3e7abc0e4acd ✓ |
| T5-D27 | LLR-df7prq | connect-src is present | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; obje…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D28 | LLR-df7prq | connect-src admits no keyword source beyond IPC | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D29 | LLR-df7prq | connect-src does not admit the dev server | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D30 | LLR-df7prq | checker: connect-src beyond IPC refused | tests/csp_policy.rs: `if !IPC_SOURCES.contains(&src.as_str()) && !extra_origins.contains(&src.as_str()) {` → `if false {` | RED | connect_src_beyond_ipc_is_rejected | 3e7abc0e4acd ✓ |
| T5-D31 | LLR-df7prq | checker: a missing connect-src refused | tests/csp_policy.rs: `    for src in sources("connect-src")? {` → `    for src in d.get("connect-src").into_iter().flatten() {` | RED | connect_src_beyond_ipc_is_rejected | 3e7abc0e4acd ✓ |
| T5-D32 | LLR-p3xwx4 | no wildcard in any directive | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: *; co…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D33 | LLR-p3xwx4 | no network scheme in any directive | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D34 | LLR-p3xwx4 | no network host in any directive | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.example.org; …` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D35 | LLR-p3xwx4 | checker: the network-origin sweep | tests/csp_policy.rs: `if is_network_source(src)⏎` → `if false && is_network_source(src)⏎` | RED | https_scheme_source_is_rejected_in_any_directive, wildcard_source_is_rejected_in_any_directive | 3e7abc0e4acd ✓ |
| T5-D36 | LLR-ausr5q | no directive appears twice | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D37 | LLR-ausr5q | whatever the case of its name | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self'; SCRIPT-SRC 'unsafe-inline'; script-src 'self'; style-src 'self' 'unsafe-inline'…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T5-D38 | LLR-ausr5q | checker: a repeated directive refused | tests/csp_policy.rs: `if out.contains_key(&name) {` → `if false {` | RED | directive_names_are_matched_case_insensitively | 3e7abc0e4acd ✓ |
| T5-D39 | LLR-ausr5q | checker: names compared case-insensitively | tests/csp_policy.rs: `.map(str::to_ascii_lowercase)` → `.map(str::to_string)` | RED | directive_names_are_matched_case_insensitively | 3e7abc0e4acd ✓ |
| T5-D40 | LLR-tc5aax | devCsp admits no origin beyond the dev server | tauri.conf.json: `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self…` → `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self…` | RED | dev_csp_adds_only_the_dev_server_origins | 9922fff00b9c ✓ |
| T5-D41 | LLR-tc5aax | devCsp names http://localhost:5173 | tauri.conf.json: `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self…` → `"devCsp": "default-src 'self'; script-src 'self' ws://localhost:5173; style-src 'self' 'unsafe-inline'; img…` | RED | dev_csp_adds_only_the_dev_server_origins | 9922fff00b9c ✓ |
| T5-D42 | LLR-tc5aax | devCsp names ws://localhost:5173 | tauri.conf.json: `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self…` → `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173; style-src 'self' 'unsafe-inline'; i…` | RED | dev_csp_adds_only_the_dev_server_origins | 9922fff00b9c ✓ |
| T5-D43 | LLR-tc5aax | devCsp meets the script-src clause | tauri.conf.json: `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self…` → `"devCsp": "default-src 'self'; script-src 'self' 'unsafe-eval' http://localhost:5173 ws://localhost:5173; s…` | RED | dev_csp_adds_only_the_dev_server_origins | 9922fff00b9c ✓ |
| T5-D44 | LLR-tc5aax | devCsp meets the 'none' clause | tauri.conf.json: `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self…` → `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self…` | RED | dev_csp_adds_only_the_dev_server_origins | 9922fff00b9c ✓ |
| T5-D45 | LLR-tc5aax | devCsp meets the default-src clause | tauri.conf.json: `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self…` → `"devCsp": "default-src *; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self' 'un…` | RED | dev_csp_adds_only_the_dev_server_origins | 9922fff00b9c ✓ |
| T5-D46 | LLR-tc5aax | devCsp meets the no-repeat clause | tauri.conf.json: `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self…` → `"devCsp": "default-src 'self'; script-src 'self' http://localhost:5173 ws://localhost:5173; style-src 'self…` | RED | dev_csp_adds_only_the_dev_server_origins | 9922fff00b9c ✓ |
| T5-D47 | LLR-tc5aax | checker: an origin beyond the extra ones refused (T3 M22, rerun: csp_policy.rs changed since) | tests/csp_policy.rs: `\|\| extra_origins.contains(&src.as_str());` → `\|\| !extra_origins.is_empty();` ; `if !IPC_SOURCES.contains(&src.as_str()) && !extra_origins.contains(&src.as_str()) {` → `if !IPC_SOURCES.contains(&src.as_str()) && extra_origins.is_empty() {` ; `                && !extra_origins.contains(&src.as_str())⏎` → `                && extra_origins.is_empty()⏎` | RED | dev_policy_rejects_an_origin_beyond_the_dev_server | 3e7abc0e4acd ✓ |
| T5-D48 | LLR-7ymxtn | default-src is exactly 'self' (a non-network addition) | tauri.conf.json: `"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; conn…` → `"csp": "default-src 'self' data:; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:…` | RED | shipped_csp_meets_every_clause | 9922fff00b9c ✓ |
| T3-M1 | LLR-97rww8 | the passphrase refusal names ODS_PASSPHRASE | src/policy.rs: `"ODS_PASSPHRASE is not set. Set it` → `"The passphrase is not set. Set it` | RED | refusals_for_empty_values_name_both_variables_supplier_first, passphrase_refusal_names_both_variables_supplier_first | 5136e084b83f ✓ |
| T3-M2 | LLR-85zque | the store is persona_store.bin inside the directory | src/state.rs: `data_dir.join("persona_store.bin")` → `data_dir.join("store.bin")` | RED | the_store_is_opened_inside_the_data_dir_it_creates, a_store_under_another_passphrase_is_refused | 601198b3c8ce ✓ |
| T3-M3 | LLR-85zque | a directory that cannot be created is refused | src/state.rs: `std::fs::create_dir_all(&data_dir)⏎            .map_err(\|e\| format!("create data_dir {}: {e}", data_dir.d…` → `let _ = std::fs::create_dir_all(&data_dir);` | RED | a_data_dir_that_cannot_be_created_is_refused_naming_it | 601198b3c8ce ✓ |
| T3-M4 | LLR-85zque | no other store is opened in its place | src/state.rs: `PersonaStore::open(data_dir.join("persona_store.bin"), passphrase)⏎` → `PersonaStore::open(data_dir.join("persona_store.bin"), passphrase)⏎            .or_else(\|_\| PersonaStore:…` | RED | a_store_under_another_passphrase_is_refused | 601198b3c8ce ✓ |
| T3-M5 | LLR-vqkr5t | AppState.data_dir is the directory the store was opened in | src/state.rs: `            data_dir,⏎            chain_endpoint,⏎            transport_mode,⏎            receiver_started` → `            data_dir: data_dir.join("."),⏎            chain_endpoint,⏎            transport_mode,⏎         …` | RED | connection_status_reports_a_data_dir_it_had_to_create_verbatim, connection_status_reports_the_directory_the_store_was_opened_in | 601198b3c8ce ✓ |
| T3-M6 | LLR-vzf8j2 | export_invite hands the parsed identifier on | src/commands.rs: `let oid = parse_org_id(&org_id)?;⏎    let svc = state.service.lock().await;⏎    svc.export_invite(oid)` → `let oid = parse_org_id(&org_id[1..])?;⏎    let svc = state.service.lock().await;⏎    svc.export_invite(oid)` | RED | export_invite_does_not_refuse_a_well_formed_org_id | a7314e9b5605 ✓ |
| T3-M7 | LLR-6pmrma | a leading 0x is stripped from the member id | src/commands.rs: `hex::decode(member_id_hex.strip_prefix("0x").unwrap_or(&member_id_hex))` → `hex::decode(&member_id_hex)` | RED | revoke_member_accepts_a_zero_x_prefixed_member_id | a7314e9b5605 ✓ |
| T3-M8 | LLR-ty85xv | a whitespace-only peer address is absent | src/commands.rs: `let trimmed = peer_addr_blob.trim().trim_start_matches("0x");` → `let trimmed = peer_addr_blob.trim_start_matches("0x");` | RED | revoke_member_treats_a_whitespace_only_peer_addr_as_absent | a7314e9b5605 ✓ |
| T3-M9 | LLR-n6twt7 | a non-blank address is decoded as an EndpointAddr (a valid one passes) | src/commands.rs: `Some(postcard::from_bytes(&addr_bytes).map_err` → `Some(postcard::from_bytes(&addr_bytes[1..]).map_err` | RED | revoke_member_does_not_refuse_an_encoded_endpoint_addr | a7314e9b5605 ✓ |
| T3-M10 | LLR-n6twt7 | non-hex text is refused | src/commands.rs: `hex::decode(trimmed).map_err(\|e\| format!("peer_addr_blob hex: {e}"))?;` → `hex::decode(trimmed).unwrap_or_default();` | RED | revoke_member_refuses_a_peer_addr_that_is_not_hex | a7314e9b5605 ✓ |
| T3-M11 | LLR-n6twt7 | bytes that are not an EndpointAddr are refused, not dropped | src/commands.rs: `Some(postcard::from_bytes(&addr_bytes).map_err(\|e\| format!("peer_addr decode: {e}"))?)` → `postcard::from_bytes(&addr_bytes).ok()` | RED | revoke_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr | a7314e9b5605 ✓ |
| T3-M12 | LLR-8krgzj | a 32-byte secret passes | src/commands.rs: `if bytes.len() != 32 {` → `if bytes.len() != 31 {` | RED | admit_member_does_not_refuse_a_32_byte_or_absent_org_secret, admit_member_refuses_an_org_secret_that_is_not_32_bytes | a7314e9b5605 ✓ |
| T3-M13 | LLR-8krgzj | non-hex text is refused | src/commands.rs: `let bytes = hex::decode(hex_str.trim_start_matches("0x"))⏎                .map_err(\|e\| format!("org_secre…` → `let bytes = hex::decode(hex_str.trim_start_matches("0x")).unwrap_or_else(\|_\| vec![0u8; 32]);` | RED | admit_member_refuses_an_org_secret_that_is_not_hex | a7314e9b5605 ✓ |
| T3-M14 | LLR-ctrfz4 | otherwise the node address is decoded | src/commands.rs: `postcard::from_bytes(&jr.node_addr)` → `postcard::from_bytes(&jr.node_addr[1..])` | RED | admit_member_does_not_refuse_a_decodable_node_addr | a7314e9b5605 ✓ |
| T3-M15 | LLR-ctrfz4 | bytes that are not an EndpointAddr are refused | src/commands.rs: `let peer_addr: EndpointAddr = if jr.node_addr.is_empty() {` → `let peer_addr: EndpointAddr = if jr.node_addr.len() < 8 {` | RED | admit_member_refuses_a_node_addr_that_is_not_an_endpoint_addr | a7314e9b5605 ✓ |
| T3-M16 | LLR-pguhw5 | field persona_id | src/commands.rs: `pub struct PersonaDto {⏎    pub persona_id: String,` → `pub struct PersonaDto {⏎    #[serde(rename = "id")]⏎    pub persona_id: String,` | RED | list_personas_reports_exactly_the_persona_fields | a7314e9b5605 ✓ |
| T3-M17 | LLR-pguhw5 | org_id is null, not "", for a persona in no Organisation | src/commands.rs: `org_id: p.org_id.map(\|id\| hex::encode(id.as_bytes())),` → `org_id: Some(p.org_id.map(\|id\| hex::encode(id.as_bytes())).unwrap_or_default()),` | RED | a_persona_in_no_organisation_reports_a_null_org_id | a7314e9b5605 ✓ |
| T3-M18 | LLR-pmus9f | has_node_addr is true when the request carries an address | src/commands.rs: `has_node_addr: !jr.node_addr.is_empty(),` → `has_node_addr: jr.node_addr.len() > 64,` | RED | import_join_request_reports_the_request_it_decodes | a7314e9b5605 ✓ |
| T3-M19 | LLR-pmus9f | a malformed blob is refused with org-node's message, naming the blob | src/commands.rs: `org_node::service::OrgService::import_join_request(blob).map_err(\|e\| e.to_string())` → `org_node::service::OrgService::import_join_request(blob).map_err(\|_\| String::from("refused"))` | RED | import_join_request_refuses_a_malformed_blob | a7314e9b5605 ✓ |
| T3-M20 | LLR-p2nm5a | epoch-changed carries the epoch, including 0 | src/events.rs: `payload: json(&EpochChanged { org_id, epoch: *epoch }),` → `payload: json(&EpochChanged { org_id, epoch: (*epoch).max(1) }),` | RED | updated_carries_boundary_epochs_and_roots_verbatim | 921a7bf45f8b ✓ |
| T3-M21 | LLR-hgsdm8 | the organisation identifier verbatim | src/events.rs: `name: "record-unreadable",⏎            payload: json(&OrgOnly { org_id }),` → `name: "record-unreadable",⏎            payload: json(&OrgOnly { org_id: org_id.trim_start_matches('0') }),` | RED | record_unreadable_holds_for_any_organisation_id | 921a7bf45f8b ✓ |
| T3-M23 | LLR-mzae5q | the event's organisation is kept | verify.ts: `org_id: event.org_id ?? '(unknown organisation)',` → `org_id: '(unknown organisation)',` | RED | keeps the organisation a failed event names | 0961c0a6a01e ✓ |
| T3-M24 | LLR-rzx6ks | each list keeps the newest first | verify.ts: `verifyResultFrom(event, ts),⏎⇥⇥⇥...view.verifyLog.slice(0, MAX_VERIFY_ROWS - 1)` → `...view.verifyLog.slice(0, MAX_VERIFY_ROWS - 1),⏎⇥⇥⇥verifyResultFrom(event, ts)` | RED | files the newest announcement first in each list, keeps at most 50 verification rows, dropping the oldest | 0961c0a6a01e ✓ |
| T3-M25 | LLR-rzx6ks | the log keeps at most 50 rows | verify.ts: `const MAX_VERIFY_ROWS = 50;` → `const MAX_VERIFY_ROWS = 51;` | RED | keeps at most 50 verification rows, dropping the oldest | 0961c0a6a01e ✓ |
| T3-M26 | LLR-rzx6ks | the error list keeps at most 20 entries | verify.ts: `const MAX_RECEIVER_ERRORS = 20;` → `const MAX_RECEIVER_ERRORS = 21;` | RED | keeps at most 20 receiver errors, dropping the oldest | 0961c0a6a01e ✓ |
| T3-M27 | LLR-csbs5v | at most one 0x is stripped | revoke.ts: `.replace(/^0x/, '')` → `.replace(/^(0x)+/, '')` | RED | rejects a doubled 0x prefix | 696ce3fdcb3e ✓ |
