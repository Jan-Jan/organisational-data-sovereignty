# Verification — org-io domain API: batched epochs, verification-only adoption, pull delivery (2026-10-31)

One record per change, written at `merge-change` step 5 and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line and carries the gate's figures in its own trailers; this
file is the rest of the evidence that travels with the change.

branch: worktree-org-io-domain-api
reviewer: independent fresh subagent reviewers at `merge-change` step 4, with no implementation narrative given to them — in round 1 one per unit (org-node, org-io, app, on-chain-client), then one per affected scope in each later round through round 7, each at its round's `review dispatched` commit with the diff, the plan, the claimed items and the review checklist
verdict: round 7, the last review round, raised no code or requirement finding above low (its reviewer: "Every sentence the changed risk text, PR-hh4xvx and the LLRs state about the software matches the code in org-node/src, org-io/src and app/src-tauri/src"; "This is the last review round"); every finding of rounds 1–7 is dispositioned below, and each round's own verdict is quoted under its heading.
reproduced: in part. This is a feature change (the domain API) that also resolves defects. PR-8sgsb5 was reproduced by `a_first_adoption_keeps_the_issuers_addresses_in_the_address_book` (org-node, red before the fix: the address book was empty); PR-8ghk9s by `a_create_proxy_timeout_records_no_unknown_write_and_the_next_write_creates_again` (org-io, red before: "not refused: Unknown { … the write timed out after 90 s … }"); PR-yng83g by `a_failed_deletion_at_open_is_retried_hourly_once_receiving_starts` (org-io, red before: the wait elapsed with nothing retried); PR-wch3zu by `a_removed_file_beside_a_live_store_of_its_key_is_kept_and_reported` (org-io, red before: the open failed `KeychainItemMissing`). PR-vt244s and PR-k5tdxj item 2 were not reproduced by a failing test: their defective paths (the chain written before the record in org-node's commit path; delivery hanging on the in-memory unknown write that `reconcile` cleared in `submit.rs`/`publish.rs`) were removed whole, the root cause was established by reading the removed code, and their resolutions name the tests that verify the replacement (org-node `verify_adopts_the_provisional_organisation_the_chain_holds_after_a_restart`, org-io `a_write_that_executed_before_a_restart_is_adopted_by_the_next_verification`, app `an_own_adoption_is_sent_once_and_nothing_else_is` and `a_behind_organisation_is_requested_at_each_run_until_in_step`, among others).

Units touched: on-chain-client, org-node, org-io, app. Impact set
(`.guardrails/scripts/check-units.sh --impact refs/heads/master..HEAD`):
on-chain-client, org-node, org-io and app, each `touched`.

Change: org-io gains the demo journey's domain surface — propose a Person, a
founding, a Member, admissions and revocations; propose, write, verify and send
an Organisation's next epoch; request it when behind; act on this node's own
removal — over a sans-IO org-node that keeps batched epoch state per Persona and
adopts only by verification, an on-chain-client whose genesis runs in two
resumable steps, and an app and webview that drive it.
Plan: `docs/plans/2026-10-21-org-io-domain-api.md` (tasks under "## 2. Task
outline", reports under "## Progress").

Implements (the plan header's **Implements:** lines):
- **on-chain-client** — REQ: REQ-v86pyy, REQ-6vh3ht; RC: none new; SDD: none new. In-force dated items this change's LLRs satisfy or its REQs implement: REQ-6jefu2. Dated SDDs amended by this change: SDD-5wamsz, SDD-rxfu6h, SDD-yg7n55.
- **org-node** — REQ: REQ-4qh2h8, REQ-dhr58q, REQ-3pj7rd, REQ-x2dnee, REQ-9ktgwx, REQ-dwb9bq, REQ-zxgd46, REQ-g2537c, REQ-kd34f5, REQ-2nnsmv, REQ-hx4r8y, REQ-6qxesy, REQ-f8p36u, REQ-nbx4nn, REQ-p4hwuv, REQ-2yduet, REQ-akfwf3, REQ-ncv4sc, REQ-eqt23j, REQ-ttv8k7, REQ-ky5tyd, REQ-9xf99q, REQ-48zc3e, REQ-y9p27b, REQ-h29c8e, REQ-6tepxq, REQ-ayyt5f, REQ-35s4eb; RC: RC-sp46zh, RC-amgwm9, RC-d4fc4s, RC-dv6mka, RC-5jkcxz, RC-45k3zg, RC-5gbgwb; SDD: SDD-undd7q, SDD-hwax8v, SDD-kegag2, SDD-z4m57w, SDD-gatf4h, SDD-8zzmav, SDD-busuj3, SDD-qzv5mn, SDD-n2x3gb, SDD-42cjqk, SDD-pkqc2j, SDD-27p679. In-force dated items this change's LLRs satisfy or its REQs implement: REQ-3dsweu, REQ-8amu2a, REQ-b462sh, REQ-ea4qs5, REQ-ech45n, REQ-f2k4tr, REQ-ju6vn2, REQ-jy6ybw, REQ-kt877x, REQ-m2xh8q, REQ-ps2gy2, REQ-qrtsc9, REQ-qthum6, REQ-szq3ud, REQ-tqap3r, REQ-uk9rw7, REQ-uy7kfq, REQ-vxqc5g, REQ-x3jrh8, REQ-y7tsft, REQ-yp75u9, RC-b6mydy, RC-ub82my, RC-zutc67. Dated SDDs amended by this change: SDD-89es4z, SDD-af5vnt, SDD-rx2yvy, SDD-swtd3w, SDD-vee2fq.
- **org-io** — REQ: REQ-ggjuu6, REQ-3pjmub, REQ-36gxmg, REQ-nactq6, REQ-bm4ppg, REQ-sjqxg2, REQ-wb93ch, REQ-z73ctw, REQ-wpq4fp, REQ-7jh4rh, REQ-kh2uth, REQ-k6kcgd, REQ-sd7ayv, REQ-98a3ty, REQ-jqmpz5, REQ-qfnx94, REQ-wd683g, REQ-4bvmwc, REQ-ek652a, REQ-xd5a2d, REQ-cj9u3y, REQ-2yw5y3, REQ-u894ty, REQ-2xgsbn, REQ-bg5whd; RC: RC-6m89tr; SDD: SDD-3qwjs2, SDD-j7k844, SDD-bzu2jg, SDD-zqv3v6, SDD-9rbm8t, SDD-cxh9d8, SDD-7mhqaz, SDD-4drrpr, SDD-mk39e5, SDD-4bbt47, SDD-t4z8sz, SDD-2bzgtp, SDD-dhh3pw, SDD-2fc2cq, SDD-3my66m, SDD-wyn7b6. In-force dated items this change's LLRs satisfy or its REQs implement: REQ-3pxa8f, REQ-6fk4qj, REQ-8trkps, REQ-bupq5y, REQ-fh5gsq, REQ-g9cu2c, REQ-ntkjk6, REQ-pnr6kh, REQ-qkrj55, REQ-rz93uz, REQ-tg9zrn, RC-wbs9ch. Dated SDDs amended by this change: SDD-f4khqn, SDD-z85ux9.
- **app** — REQ: REQ-akeah6, REQ-5q7nyg, REQ-vgn96n, REQ-ergn9z, REQ-88npkt, REQ-s3vnmh, REQ-pe2c7u, REQ-qv4d2z, REQ-y53dqn, REQ-2yh3c8, REQ-vb5pag, REQ-k7ay5b; RC: RC-rv4xhs, RC-dfzb5q, RC-8h344c, RC-c5tpre; SDD: SDD-6pjvhc, SDD-tgu7dp, SDD-swt9fs, SDD-ua2d9b, SDD-xg4nj7, SDD-czjf2c, SDD-7aq69w, SDD-g5wmxt, SDD-23vjd8. In-force dated items this change's LLRs satisfy or its REQs implement: REQ-2k7ys4, REQ-dp95pv, REQ-kn5rtx, REQ-sjkp8z, REQ-tw4cb5. Dated SDDs amended by this change: SDD-2pa6h6, SDD-6g3wnh, SDD-k95rmp, SDD-mwqf6x.
- Every in-force LLR of the four DRAFT architecture files (listed in the coverage table at the end of the plan's §2), and LLR-gh4rw3 once restored (T1, design gap G1). Dated LLRs amended by this change are trace-only (LLR-v873fx, LLR-qhyc3n) or noted (LLR-d2pzzk, LLR-z35vbh, carried by T30); none needs new code. LLR-y2v8v2 (trace-only before), LLR-3wb7th and LLR-9fvb3y are superseded by this change (dispatcher decision O4: by LLR-f9x3vf, LLR-rr558z and LLR-283x6f).
- Items minted or amended during the merge review and the conformance reviews are named in their dispositions below (for example LLR-4tsgfh, LLR-22se2q, LLR-hgrez9, LLR-c759yd, LLR-6sfub6, LLR-jqb5z7, LLR-n2srhc).

## The gate

The gate's figures are not in this record. The squash commit carries them as
trailers (`merge-change` step 7): `Gate-tree:` names the tree the step 6a gate
measured, a `Gate:` line gives each command's totals, and `Base:` names the
base and the commit step 1 merged. `finish-merge.sh` refuses the cleanup
unless `Gate-tree:` is the commit's own tree.

## Red → green

Every row of a per-ID table for this change would restate the plan's task
reports, so, as the S4 record did, the evidence is given grouped by task and
round: every `red -> green:`, `inherited:`, `removed:`, `not red-first` and
`changed …` line of the plan, copied verbatim from its task reports under
"## Progress" (each task report names the IDs its tests verify), and, first,
the `inherited:` lines the task text itself planned. A REQ, RC or SDD is
verified through the LLRs it names; its own red is theirs. Lines naming tests
later renamed or deleted are kept as written: the observation was made. The
subagent running each task loop is the only party that saw these tests fail;
these lines are where that observation becomes durable.

#### Planned in the task text — T3 — on-chain-client: `create_proxy` and `found_at`, `genesis` their composition

```
  inherited: an_approval_without_execution_fails_the_genesis_step — from LLR-z9vugt
  inherited: a_failed_genesis_step_is_named_and_stops_the_ceremony — from LLR-z9vugt
  inherited: genesis_creates_funds_maps_and_records_in_that_order — from LLR-a2acvh
  inherited: each_genesis_creates_a_new_pure_proxy_and_writes_through_it — from LLR-a2acvh
```

#### Planned in the task text — T5 — org-node: the single-Organisation store schema and the rejection vocabulary; the commit path removed

```
  inherited: the_ten_new_variants_are_distinct_and_name_the_organisation — from LLR-n67aw8
  inherited: stale_state_names_both_epochs_and_the_proof_refusal_names_its_cause — from LLR-n67aw8
  inherited: a_persona_record_decoded_directly_refuses_an_invalid_handle — from LLR-g76zqd
  inherited: a_store_of_valid_records_opens_with_every_field_parsed — from LLR-8bum44
  inherited: a_store_with_any_one_field_invalid_is_refused_naming_exactly_that_field — from LLR-8bum44
  inherited: a_store_with_two_invalid_fields_in_one_record_reports_the_first_in_record_order — from LLR-8bum44
  inherited: a_record_snapshot_holding_an_invalid_value_is_refused_naming_the_field — from LLR-8bum44
  inherited: a_record_snapshot_with_an_invalid_handle_and_member_key_reports_the_handle — from LLR-8bum44
  inherited: a_record_carries_its_organisation_private_key_and_a_store_without_one_is_refused — from LLR-byjvd9
  inherited: two_stores_with_one_device_key_are_refused — from LLR-rdda86
  inherited: an_operation_changes_only_its_personas_store — from LLR-rdda86
  inherited: list_personas_returns_one_record_per_store — from LLR-rdda86
  inherited: restore_store_replaces_or_adds_one_store_and_drop_store_removes_one — from LLR-rdda86
```

#### Planned in the task text — T7 — org-node: the Wire message of six kinds

```
  inherited: an_org_information_body_with_trailing_bytes_is_refused — from LLR-tajh9d, LLR-js9dsu
  inherited: an_organisation_information_body_without_its_snapshot_or_key_does_not_decode — from LLR-js9dsu, REQ-c29s93
```

#### Planned in the task text — T11 — org-node: the receive path's sender rule and the admission request

```
  inherited: a_revocation_relayed_by_a_non_member_is_refused — from LLR-kzgjz8
  inherited: a_body_that_does_not_decode_is_malformed_and_changes_nothing — from LLR-zetn7a
  inherited: an_update_for_a_held_organisation_from_an_unlisted_sender_is_refused_in_the_check — from LLR-2r2fha
```

#### Planned in the task text — T17 — org-node: adopting a matched provisional organisation

```
  inherited: kept_change_set_round_trips_through_the_store — from LLR-d9778a
```

#### Planned in the task text — T24 — org-node: deciding this node's removal and signing its acknowledgements

```
  inherited: acknowledgement_signed_bytes_are_the_domain_then_the_postcard_tuple — from LLR-gbe9bt
  inherited: a_signature_without_the_acknowledgement_domain_is_refused — from LLR-gbe9bt
  inherited: accept_refuses_in_order_and_changes_nothing — from LLR-tx8ruv, LLR-uw7nmv
```

#### Planned in the task text — T26 — org-node: the pinned store encodings

```
  inherited: organisation_information_wire_message_is_pinned — from LLR-ayrdr8, LLR-ms8njy, LLR-js9dsu
  inherited: truncated_pinned_values_are_refused — from LLR-ayrdr8
```

#### Planned in the task text — T27 — org-node: the cross-cutting properties — one store change, one deleting path, no IO, key custody

```
  inherited: no_sealed_store_holds_the_device_seed — from LLR-bvt5up
  inherited: saving_needs_no_device_secret — from LLR-bvt5up
```

#### Planned in the task text — T28 — org-io: the handle's random source; the removed methods and sequences gone

```
  inherited: iroh_is_named_only_in_the_transport_module_and_test_support — from LLR-3vb4k9
```

#### Planned in the task text — T30 — org-io: a new Persona per founding and per Member

```
  inherited: the_write_lends_the_bytes_only_through_lend_to_keychain — from LLR-n9dnm4
```

#### Planned in the task text — T31 — org-io: issuing, reading and cancelling an Invite; the kept admission requests

```
  inherited: an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id — from LLR-9sraks
  inherited: no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to — from LLR-9sraks
  inherited: an_outstanding_invites_file_in_the_data_dir_is_never_read — from LLR-f35pda
  inherited: the_invite_is_returned_only_after_its_store_write — from LLR-h9qf3s
```

#### Planned in the task text — T35 — org-io: verification — the read, the outcome and the handle's clock

```
  inherited: org_io_hands_out_no_bare_service — from LLR-7pj5af, LLR-9r2bxd
```

#### Planned in the task text — T40 — org-io: the receive step

```
  inherited: a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted — from LLR-wvgec4, LLR-unqfj5
  inherited: an_acknowledgement_a_member_still_listing_the_device_refuses_is_answered_refused — from LLR-wvgec4, LLR-unqfj5
  inherited: an_acknowledgement_relayed_by_another_device_is_refused — from LLR-wvgec4
  inherited: a_dropped_responder_writes_nothing — from LLR-unqfj5
  inherited: the_accept_path_has_no_allow_list — from LLR-wvgec4
  inherited: nothing_but_the_verdict_is_written_on_an_inbound_stream — from LLR-unqfj5
  inherited: a_body_the_check_refuses_reads_no_chain_and_writes_nothing — from REQ-tzr3dk
  inherited: a_refused_check_reads_no_chain — from LLR-cvyyr8
  inherited: the_sender_org_node_sees_is_the_handshakes — from LLR-cvyyr8
  inherited: an_acknowledgement_is_settled_with_no_chain_read_and_no_store_write — from LLR-cvyyr8
  inherited: the_store_write_lands_before_the_outcome_returns — from LLR-cvyyr8
```

#### Planned in the task text — T45 — org-io: the handle surface

```
  inherited: a_node_holding_no_signatory_key_is_refused_with_not_configured — from LLR-c9fyun
  inherited: a_failed_signatory_read_is_an_error_never_not_admin — from LLR-c9fyun
  inherited: a_configured_account_listed_as_delegate_is_admin_and_delegates_are_read_only_for_a_mapped_h160 — from LLR-c9fyun
```

#### Planned in the task text — T51 — app: `verify_organisation` and `verify_and_send` — send once, request when behind

```
  inherited: list_orgs_reports_not_checked_before_any_reconcile — from LLR-8axx7h
```

#### Planned in the task text — T52 — app: the kept admission requests listed; the handler set and the `org_id` parse

```
  inherited: product_registers_the_same_commands_as_the_harness — from LLR-4wcyqy
  inherited: list_personas_returns_an_empty_list_on_a_fresh_store — from LLR-4wcyqy
```

#### Planned in the task text — T53 — app: what the receiver maps an arrival to

```
  inherited: every_verification_verdict_is_classified_as_a_verification_failure — from LLR-7bk6qh
  inherited: a_verification_verdict_carries_its_own_message_and_no_invented_organisation — from LLR-7bk6qh
  inherited: a_chain_failure_is_classified_as_a_receiver_error — from LLR-7bk6qh
  inherited: the_two_classes_are_actually_distinguished — from LLR-7bk6qh
  inherited: the_commit_workflow_refusals_are_classified_as_receiver_errors — from LLR-7bk6qh
  inherited: the_sender_refusals_are_classified_as_receiver_errors — from LLR-7bk6qh
  inherited: the_store_and_invitation_refusals_are_classified_as_receiver_errors — from LLR-7bk6qh
  inherited: locally_reachable_variants_are_classified_as_receiver_errors — from LLR-7bk6qh
  inherited: a_wire_message_over_the_bound_is_classified_as_a_receiver_error — from LLR-7bk6qh
  inherited: a_refused_key_or_field_is_classified_as_a_receiver_error — from LLR-7bk6qh
```

#### Planned in the task text — T56 — app webview: the verification view

```
  inherited: names the Organisation and the result, with the epoch where there is one — from LLR-hwcj9n
  inherited: shows the reason for a refused or unverified result and never calls it in step — from LLR-hwcj9n
  inherited: renders a result it does not know as unverified rather than in step — from LLR-hwcj9n
  inherited: renders each status, an unverified one with its reason — from LLR-hwcj9n
```

#### Planned in the task text — T59 — app webview: the warning before the first admission request; the peer-address decisions gone

```
  inherited: refuses until the user has confirmed — from LLR-n2u4uf
```

#### Planned in the task text — T60 — Sweep: annotations left, MISSING-TEST, and the tests of in-force items over removed operations

```
  inherited: the_service_presents_no_chain_write — from LLR-65py3d, LLR-3v5nu9
  inherited: org_node_is_sans_io — from LLR-4tcxsu
  inherited: member_ids_are_not_derived_from_keys — from LLR-ag9mgm
  inherited: readmission_with_same_keys_gets_a_fresh_member_id — from LLR-ag9mgm
  inherited: org_node_holds_no_invitation_exchange — from LLR-836z24, LLR-qezw3n
  inherited: a_revocation_about_an_organisation_not_held_is_refused_before_the_chain — from LLR-38e2kn
  inherited: a_revocation_reaches_the_administrators_disk — from LLR-qg9utu, LLR-8hdu9x
  inherited: a_failed_revocation_send_leaves_the_committed_removal_in_place — from LLR-qg9utu, LLR-t4znbk
  inherited: a_first_admission_records_the_chains_key_the_private_key_and_the_member — from LLR-3fwykc, LLR-e5c9ud
  inherited: pr_szkat6_an_admitted_member_holds_the_organisation_private_key — from LLR-3fwykc
  inherited: pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths — from LLR-37cj3n, LLR-3fwykc
  inherited: a_committed_update_gives_the_record_the_new_key_pair_on_both_paths — from LLR-xn5pwc
  inherited: pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused_on_the_self_delete_path — from LLR-3q63zv
  inherited: revocation_from_an_unknown_device_leaves_the_record_in_place — from LLR-3q63zv, LLR-tx8ruv
  inherited: a_new_persona_is_proposed_with_an_id_derived_from_its_member_key — from LLR-tev8h8, LLR-v82xds
  inherited: a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals — from LLR-3fwykc, LLR-sj7cd5
  inherited: the_organisation_private_key_is_kept_only_in_the_encrypted_store — from LLR-3fwykc, LLR-qjz3q4
  inherited: a_founding_is_adopted_once_the_chain_carries_its_root — from LLR-3fwykc, LLR-w3fhhg, LLR-q3aj8z, LLR-dzte8x, LLR-3v5nu9, LLR-4tcxsu (commit_genesis_creates_the_record_once_the_chain_carries_the_root)
  inherited: an_own_provisional_organisation_the_chain_carries_is_adopted — from LLR-4tcxsu (commit_update_commits_a_provisional_update_the_chain_carries)
  inherited: the_record_takes_a_recorded_revocation_only_through_adoption — from LLR-qg9utu, LLR-drgdy8, LLR-3v5nu9, LLR-t4znbk (the_revoking_record_takes_the_removal_only_through_commit_update)
```

#### T1 done

```
  red -> green: none (setup, no tests).
  inherited: none.
```

#### T2 done

```
  red -> green: on-chain-client/tests/write_pure.rs::free_balance_of_none_is_zero (verifies: LLR-htp5tk) — E0432 unresolved import free_balance_of; E0599 no variant MalformedStorage → pass
  red -> green: on-chain-client/tests/write_pure.rs::free_balance_of_reads_free_at_bytes_sixteen_to_thirty_one (verifies: LLR-htp5tk) — same compile errors → pass
  red -> green: on-chain-client/tests/write_pure.rs::free_balance_of_refuses_any_length_but_eighty_as_malformed_storage (verifies: LLR-htp5tk) — same compile errors → pass
  red -> green: on-chain-client/tests/fuzz_signatory_set/fuzz_target.rs::free_balance_of_never_panics (verifies: LLR-htp5tk) — same two errors in the fuzz target → pass
  inherited: none
```

#### T3 done

```
  red -> green: on-chain-client/tests/write_compose.rs::create_proxy_makes_one_create_pure_call_and_returns_its_account (verifies: LLR-tge9q8) — E0432 unresolved import create_proxy → pass
  red -> green: on-chain-client/tests/write_compose.rs::a_failed_create_pure_is_the_create_pure_step (verifies: LLR-tge9q8, LLR-chy957) — E0432 unresolved import create_proxy → pass
  red -> green: on-chain-client/tests/write_compose.rs::found_at_on_a_fresh_proxy_reads_then_funds_maps_and_records_in_order (verifies: LLR-q6buam) — E0432 unresolved import found_at → pass
  red -> green: on-chain-client/tests/write_compose.rs::found_at_never_calls_create_pure (verifies: LLR-q6buam) — E0432 unresolved import found_at → pass
  red -> green: on-chain-client/tests/write_compose.rs::each_failing_step_is_named_and_no_later_call_is_made (verifies: LLR-chy957) — E0432 unresolved import found_at → pass
  red -> green: on-chain-client/tests/write_compose.rs::a_mapping_read_holding_another_account_is_the_map_account_step (verifies: LLR-chy957) — E0432 unresolved import found_at → pass
  red -> green: on-chain-client/tests/write_compose.rs::genesis_is_create_proxy_then_found_at_at_its_proxy (verifies: LLR-hea6y2) — E0432 unresolved imports create_proxy, found_at → pass
  inherited: an_approval_without_execution_fails_the_genesis_step — from LLR-z9vugt
  inherited: a_failed_genesis_step_is_named_and_stops_the_ceremony — from LLR-z9vugt
  inherited: genesis_creates_funds_maps_and_records_in_that_order — from LLR-a2acvh
  inherited: each_genesis_creates_a_new_pure_proxy_and_writes_through_it — from LLR-a2acvh
```

#### T5 done

```
  red -> green: org-node/tests/epoch_store.rs::store_data_holds_its_fields_in_encoding_order_and_round_trips_through_the_sealed_store (verifies: LLR-d925uu) — compile red, missing StoreData fields → pass
  red -> green: org-node/tests/epoch_store.rs::store_data_holds_no_provisional_updates_field (verifies: LLR-d925uu) — compile red → pass
  red -> green: org-node/tests/epoch_store.rs::an_outstanding_invite_keeps_its_invitee_name (verifies: LLR-d925uu) — compile red, no OutstandingInvite → pass
  red -> green: org-node/tests/epoch_store.rs::a_persona_record_is_built_from_parsed_types_and_a_provisional_member_holds_no_handle (verifies: LLR-pv2rdy) — compile red → pass
  red -> green: org-node/tests/epoch_store.rs::endpoint_addresses_accept_zero_to_sixteen_entries_of_at_most_256_bytes (verifies: LLR-h4gwns) — compile red, no EndpointAddresses → pass
  red -> green: org-node/tests/epoch_store.rs::endpoint_addresses_refuse_seventeen_entries_or_one_over_256_bytes (verifies: LLR-h4gwns) — compile red → pass
  red -> green: org-node/tests/persona_records.rs::unseal_refuses_an_invalid_field_of_each_epoch_state_kind_naming_it (verifies: LLR-7yu2xy) — compile red → pass
  red -> green: org-node/tests/persona_records.rs::unseal_refuses_a_provisional_member_with_zero_four_or_repeated_handles (verifies: LLR-7yu2xy) — compile red → pass
  red -> green: org-node/tests/persona_records.rs::a_proposed_founding_persona_holds_the_parsed_details_across_a_reopen (verifies: LLR-pv2rdy; rewrite) — compile red → pass
  red -> green: org-node/tests/persona_records.rs::a_store_with_every_epoch_state_kind_opens_with_every_field_parsed (verifies: LLR-7yu2xy; rewrite) — compile red → pass
  red -> green: org-node/tests/persona_records.rs::a_store_with_an_invalid_provisional_member_is_refused_naming_the_field (verifies: LLR-7yu2xy; rewrite) — compile red → pass
  red -> green: org-node/tests/sealed_store.rs::an_unknown_device_key_is_refused_as_persona_not_held (verifies: LLR-f9x3vf) — compile red → pass
  red -> green: org-node/tests/value_types.rs::the_new_variants_are_distinct_and_their_display_names_the_value (verifies: LLR-3sddey) — compile red, missing OrgNodeError variants → pass
  red -> green: org-node/tests/value_types.rs::the_provisional_limit_display_names_changes_and_organisations (verifies: LLR-3sddey) — compile red → pass
  red -> green: org-node/tests/value_types.rs::the_provisional_limit_refusal_names_the_limit_and_the_epoch_state (verifies: LLR-3sddey, REQ-g2537c; rewrite) — compile red → pass
  red -> green: org-node/tests/value_types.rs::the_received_message_refusals_say_what_they_refuse_without_org_key_mismatch (verifies: LLR-3sddey; rewrite) — compile red → pass
  inherited: the_ten_new_variants_are_distinct_and_name_the_organisation — from LLR-n67aw8
  inherited: stale_state_names_both_epochs_and_the_proof_refusal_names_its_cause — from LLR-n67aw8
  inherited: a_persona_record_decoded_directly_refuses_an_invalid_handle — from LLR-g76zqd
  inherited: a_store_of_valid_records_opens_with_every_field_parsed — from LLR-8bum44
  inherited: a_store_with_any_one_field_invalid_is_refused_naming_exactly_that_field — from LLR-8bum44
  inherited: a_store_with_two_invalid_fields_in_one_record_reports_the_first_in_record_order — from LLR-8bum44
  inherited: a_record_snapshot_holding_an_invalid_value_is_refused_naming_the_field — from LLR-8bum44
  inherited: a_record_snapshot_with_an_invalid_handle_and_member_key_reports_the_handle — from LLR-8bum44
  inherited: a_record_carries_its_organisation_private_key_and_a_store_without_one_is_refused — from LLR-byjvd9
  inherited: two_stores_with_one_device_key_are_refused — from LLR-rdda86
  inherited: an_operation_changes_only_its_personas_store — from LLR-rdda86
  inherited: list_personas_returns_one_record_per_store — from LLR-rdda86
  inherited: restore_store_replaces_or_adds_one_store_and_drop_store_removes_one — from LLR-rdda86
  inherited: org_node_holds_no_dialling_address — from LLR-qezw3n
  inherited: org_node_holds_no_invitation_exchange — from LLR-qezw3n
  inherited: a_chain_state_at_the_records_epoch_with_another_root_is_a_conflict — from LLR-tx8ruv
```

#### T4 done

```
  red -> green: on-chain-client/tests/write_compose.rs::a_mapped_proxy_is_not_funded_and_its_balance_is_not_read (verifies: LLR-jk76wf) — failed on T3's code: log showed FreeBalance read for a mapped proxy → pass
  red -> green: on-chain-client/tests/write_compose.rs::an_unmapped_proxy_holding_the_fund_amount_is_not_funded_again (verifies: LLR-jk76wf) — failed: T3 funded anyway (1000000000000) → pass
  red -> green: on-chain-client/tests/write_compose.rs::a_mapped_proxy_is_not_mapped_again (verifies: LLR-95puh9) — failed: log had five steps → pass
  red -> green: on-chain-client/tests/write_compose.rs::a_record_refused_on_a_slot_holding_a_genesis_is_the_record_genesis_step (verifies: LLR-g8qevz) — failed: Step{Fund} from T3's balance read → pass
  red -> green: on-chain-client/tests/write_compose.rs::dust_below_the_fund_amount_is_not_taken_as_funded (verifies: LLR-jk76wf) — green on arrival; mutation check: `balance < FUND_AMOUNT` → `balance == 0` made it red (no Fund step), reverted → green
  red -> green: on-chain-client/tests/write_compose.rs::the_record_step_is_always_dispatched_at_expected_epoch_zero (verifies: LLR-g8qevz) — green on arrival; mutation check: record dispatch guarded by `if mapped.is_none()` made it red at (Some(PROXY), 0), reverted → green
  inherited: none
```

#### T6 done

```
  red -> green: org-node/tests/person_store.rs::create_parses_the_name_and_surname_and_holds_no_handle (verifies: LLR-wqu26s) — compile red, unresolved imports org_node::Person / PersonData / PersonStore → pass
  red -> green: org-node/tests/person_store.rs::create_refuses_an_invalid_name_or_surname_naming_the_field_and_creates_nothing (verifies: LLR-wqu26s) — same unresolved imports → pass
  red -> green: org-node/tests/person_store.rs::a_sealed_person_store_unseals_to_the_same_person (verifies: LLR-73sgz2) — same unresolved imports → pass
  red -> green: org-node/tests/person_store.rs::a_persona_store_never_opens_as_a_person_store_or_the_reverse (verifies: LLR-73sgz2) — same unresolved imports → pass
  red -> green: org-node/tests/person_store.rs::the_person_and_member_types_are_named_under_org_node_paths (verifies: LLR-5uf4u2) — unresolved imports, cannot find type Person in crate org_node → pass
  inherited: none
```

#### T7 done

```
  red -> green: org-node/tests/wire_frame_bound.rs::the_six_kinds_round_trip_with_their_indices (verifies: LLR-xr88a8) — compile red, no variant AdmissionRequest / OrganisationRequest / Organisation → pass
  red -> green: org-node/tests/wire_frame_bound.rs::a_key_field_that_is_not_32_bytes_does_not_decode (verifies: LLR-xr88a8, REQ-zxgd46) — same compile red → pass
  red -> green: org-node/tests/wire_frame_bound.rs::org_id_returns_every_kinds_organisation (verifies: LLR-xr88a8) — same compile red → pass
  red -> green: org-node/tests/wire_frame_bound.rs::peek_admission_request_reads_only_the_index_org_id_and_invite_id (verifies: LLR-g9n9w9) — unresolved import peek_admission_request → pass
  red -> green: org-node/tests/wire_frame_bound.rs::peek_admission_request_is_none_for_another_index_or_a_short_body (verifies: LLR-g9n9w9) — unresolved import → pass
  red -> green: org-node/tests/wire_frame_bound.rs::exactly_six_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused (verifies: LLR-xr88a8; rewrite) — compile red → pass
  red -> green: org-node/tests/absences.rs::only_the_admission_request_carries_an_invite_identifier (verifies: LLR-g3wmen; rewrite) — runtime red: variants read were three, not six → pass
  red -> green: org-node/tests/secret_redaction.rs::no_kind_debug_shows_an_organisation_private_key_byte (verifies: LLR-4ces44) — compile red, missing variants → pass
  inherited: an_org_information_body_with_trailing_bytes_is_refused — from LLR-tajh9d, LLR-js9dsu
  inherited: an_organisation_information_body_without_its_snapshot_or_key_does_not_decode — from LLR-js9dsu, REQ-c29s93
```

#### T8 done

```
  red -> green: org-node/tests/invitation.rs::invite_parse_refuses_the_first_failing_field_naming_it (verifies: LLR-9jxwq4) — E0061 wire_for_test arity → pass
  red -> green: org-node/tests/invitation.rs::an_invite_with_zero_issuer_addresses_parses (verifies: LLR-9jxwq4) — E0061; E0609 no field issuer_addresses → pass
  red -> green: org-node/tests/invitation.rs::text_that_is_not_a_blob_is_refused_as_invite (verifies: LLR-9jxwq4) — target did not compile against the old Invite → pass
  red -> green: org-node/tests/invitation.rs::build_invite_draws_a_fresh_id_and_carries_the_stored_name_and_the_issuer_device (verifies: LLR-h9f5c9) — E0308 build_invite arguments; E0609 no field issuer → pass
  red -> green: org-node/tests/invitation.rs::an_invite_that_parses_encodes_back_to_the_same_blob (verifies: LLR-9jxwq4) — E0560 no field issuer; E0061 → pass
  red -> green: org-node/tests/invitation.rs::an_invite_that_does_not_parse_is_refused_naming_the_field (verifies: LLR-9jxwq4) — E0061 → pass
  red -> green: org-node/tests/invitation.rs::build_invite_records_the_outstanding_invite_with_its_invitee_name_and_carries_the_issuer (verifies: LLR-h9f5c9) — E0308 arguments / StoreChange expected → pass
  red -> green: org-node/tests/invitation.rs::build_invite_refuses_a_store_with_no_current_record (verifies: LLR-h9f5c9) — E0308 arguments → pass
  inherited: fuzz_invitation_parse names_a_field — from LLR-b7wgpf (rewritten over the six Invite fields, verifies: LLR-9jxwq4)
  inherited: an_invite_that_parses_encodes_back_to_the_same_blob — from LLR-b7wgpf
  inherited: an_invite_that_does_not_parse_is_refused_naming_the_field — from LLR-b7wgpf
  inherited: build_invite_records_the_outstanding_invite_with_its_invitee_name_and_carries_the_issuer — from LLR-nr8df3
  inherited: build_invite_refuses_a_store_with_no_current_record — from LLR-nr8df3
```

#### T9 done

```
  red -> green: org-node/tests/proposals.rs::propose_founding_adds_one_store_with_a_provisional_founding_at_epoch_one_and_no_record (verifies: LLR-d4gpxm) — E0599 propose_founding → pass
  red -> green: org-node/tests/proposals.rs::propose_founding_calls_no_chain_operation_and_returns_the_secret_by_value (verifies: LLR-d4gpxm) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::a_founding_key_equal_to_a_founder_key_is_refused_adding_no_store (verifies: LLR-9qavm7) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::details_are_parsed_in_order_before_any_draw (verifies: LLR-rjphn4) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::record_founding_proxy_sets_the_proxy_and_the_org_id (verifies: LLR-nx87k5) — E0599 record_founding_proxy → pass
  red -> green: org-node/tests/proposals.rs::record_founding_proxy_refuses_no_founding_or_another_proxy_and_accepts_the_same_again (verifies: LLR-nx87k5) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::a_founding_key_equal_to_a_founder_key_is_refused (verifies: LLR-9qavm7) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::founding_details_parse_into_their_types_and_add_a_store (verifies: LLR-rjphn4) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::founding_and_member_details_refuse_each_invalid_field_naming_it (verifies: LLR-rjphn4) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::non_nfc_details_are_stored_in_nfc (verifies: LLR-rjphn4) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::details_with_two_invalid_fields_report_the_first (verifies: LLR-rjphn4) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::propose_founding_returns_the_secret_and_keeps_only_the_public_key (verifies: LLR-d4gpxm) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::propose_founding_keeps_a_provisional_founding_and_no_record (verifies: LLR-d4gpxm, REQ-dhr58q, REQ-4qh2h8) — E0599 → pass
  inherited: a_founding_key_equal_to_a_founder_key_is_refused — from LLR-tax3pm
  inherited: founding_details_parse_into_their_types_and_add_a_store — from LLR-q6n25z
  inherited: founding_and_member_details_refuse_each_invalid_field_naming_it — from LLR-q6n25z
  inherited: non_nfc_details_are_stored_in_nfc — from LLR-q6n25z
  inherited: details_with_two_invalid_fields_report_the_first — from LLR-q6n25z
  inherited: propose_founding_returns_the_secret_and_keeps_only_the_public_key — from LLR-e5kqhg
  inherited: propose_founding_keeps_a_provisional_founding_and_no_record — from LLR-68yd3j, REQ-xs4ab8
```

#### T10 done

```
  red -> green: org-node/tests/proposals.rs::propose_member_adds_one_store_with_a_provisional_member_and_an_expected_admission (verifies: LLR-6acxu9) — E0599 no method propose_member → pass
  red -> green: org-node/tests/proposals.rs::propose_member_refuses_no_handle_four_handles_or_a_repeated_one (verifies: LLR-6acxu9, LLR-rjphn4) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::propose_member_refuses_an_organisation_a_held_store_is_bound_to_or_expects (verifies: LLR-w2uyyh) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::a_personas_organisation_is_written_only_by_propose_member_and_record_founding_proxy (verifies: LLR-qqqg4b) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::propose_member_refuses_an_organisation_another_persona_holds (verifies: LLR-w2uyyh) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::propose_member_refuses_an_organisation_another_persona_expects (verifies: LLR-w2uyyh) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::the_refusal_names_the_organisation_and_no_persona (verifies: LLR-w2uyyh) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::founding_and_member_details_refuse_each_invalid_field_naming_it (verifies: LLR-rjphn4; Member half added) — E0599 → pass
  inherited: none (three rewrites replace tests T5 cut: LLR-yexf5m → LLR-w2uyyh, each with its own red -> green line)
```

#### T11 done

```
  red -> green: org-node/tests/receive_path.rs::check_received_takes_no_chain_state_and_refuses_an_oversized_or_undecodable_body (verifies: LLR-94rzrk) — compile red → pass
  red -> green: org-node/tests/receive_path.rs::a_body_from_a_device_the_record_does_not_list_is_refused_before_use (verifies: LLR-283x6f) — compile red → pass
  red -> green: org-node/tests/receive_path.rs::from_an_unlisted_device_only_three_kinds_are_acted_on (verifies: LLR-w7b9kd) — compile red → pass
  red -> green: org-node/tests/receive_path.rs::an_admission_request_for_an_invite_not_outstanding_is_refused_before_its_fields_decode (verifies: LLR-6gkxpg) — compile red → pass
  red -> green: org-node/tests/receive_path.rs::an_admission_request_whose_key_is_not_the_sender_is_refused (verifies: LLR-6gkxpg) — compile red → pass
  red -> green: org-node/tests/receive_path.rs::the_first_admission_request_is_kept_and_a_repeat_or_another_device_is_not (verifies: LLR-6gkxpg, LLR-v3av9t) — compile red → pass
  red -> green: org-node/tests/receive_path.rs::a_kept_admission_request_holds_its_senders_addresses (verifies: LLR-v3av9t) — compile red → pass
  red -> green: org-node/tests/receive_chain_reads.rs::a_held_organisations_messages_from_an_unlisted_device_are_refused_before_use (verifies: LLR-283x6f, LLR-5wg8cu; rewrite) — compile red → pass
  inherited: a_revocation_relayed_by_a_non_member_is_refused — from LLR-kzgjz8
  inherited: a_body_that_does_not_decode_is_malformed_and_changes_nothing — from LLR-zetn7a
  inherited: an_update_for_a_held_organisation_from_an_unlisted_sender_is_refused_in_the_check — from LLR-2r2fha
```

#### T12 done

```
  red -> green: org-node/tests/receive_path.rs::organisation_information_from_a_listed_device_is_kept_as_received_with_its_derived_key (verifies: LLR-76bhw9, LLR-s3vq6d) — unwrap on Err MalformedMessage → pass
  red -> green: org-node/tests/receive_path.rs::the_chain_free_checks_refuse_with_no_chain_state (verifies: LLR-76bhw9) — MalformedMessage, expected OrgIdMismatch → pass
  red -> green: org-node/tests/receive_path.rs::a_sequence_number_other_than_epoch_plus_one_is_seq_not_epoch (verifies: LLR-76bhw9) — MalformedMessage, expected SeqNotEpoch{4,3} → pass
  red -> green: org-node/tests/receive_path.rs::an_identity_already_held_is_already_held_and_keeps_no_copy (verifies: LLR-s3vq6d) — unwrap on Err MalformedMessage → pass
  red -> green: org-node/tests/receive_path.rs::in_a_store_with_no_record_only_expected_organisation_information_is_decoded (verifies: LLR-e6w3nr) — kind 1 MalformedMessage, expected OrgNotHeld → pass
  red -> green: org-node/tests/receive_path.rs::unexpected_organisation_information_in_a_store_with_no_record_is_refused (verifies: LLR-e6w3nr; rewrite) — OrgNotHeld, expected AdmissionNotExpected → pass
  red -> green: org-node/tests/receive_path.rs::expected_organisation_information_is_kept_from_any_sender (verifies: LLR-e6w3nr; rewrite) — unwrap on Err OrgNotHeld → pass
  red -> green: org-node/tests/provisional_store.rs::a_keep_past_one_mebibyte_is_refused_with_provisional_limit_and_changes_nothing (verifies: LLR-m76r4k) — MalformedMessage, expected ProvisionalLimit → pass
  red -> green: org-node/tests/provisional_store.rs::epoch_state_of_exactly_the_bound_is_kept (verifies: REQ-g2537c, LLR-m76r4k; rewrite) — unwrap on Err MalformedMessage → pass
  red -> green: org-node/tests/provisional_store.rs::a_keep_that_would_exceed_the_bound_is_refused_and_nothing_changes (verifies: REQ-g2537c, LLR-m76r4k, LLR-3sddey; rewrite) — unwrap on Err MalformedMessage → pass
  red -> green: org-node/tests/receive_chain_reads.rs::an_update_from_an_unlisted_device_is_refused_before_any_chain_state_is_asked_for (verifies: REQ-uk9rw7, REQ-ztdza4, LLR-2r2fha; rewrite) — listed-Device half: unwrap on Err MalformedMessage → pass
  inherited: none
```

#### T13 done

```
  red -> green: org-node/tests/receive_path.rs::a_notice_check_notice_passes_is_kept_and_nothing_is_deleted (verifies: LLR-r4qtqe) — Err(MalformedMessage) from T11's placeholder Revocation arm → pass
  red -> green: org-node/tests/receive_path.rs::a_kept_notice_counts_against_the_bound (verifies: LLR-r4qtqe, LLR-m76r4k) — same MalformedMessage → pass
  red -> green: org-node/tests/receive_path.rs::an_acknowledgement_is_decided_by_check_acknowledgement_with_the_sender_and_no_store_change (verifies: LLR-5wg8cu) — green on arrival (T11 left the arm working); mutation check: passing ack.device for sender made it red (relayed acknowledgement from Eve accepted, receive_path.rs:912), reverted → green
  inherited: none
```

#### T14 done

```
  red -> green: org-node/tests/verification.rs::no_chain_state_is_org_not_on_chain_and_a_founding_without_proxy_is_founding_not_written (verifies: LLR-rr558z) — E0599 / E0425 → pass
  red -> green: org-node/tests/verification.rs::a_chain_behind_the_record_or_at_its_epoch_with_another_root_is_inconsistent (verifies: LLR-rr558z) — E0432 / E0599 → pass
  red -> green: org-node/tests/verification.rs::the_records_epoch_and_root_are_in_step (verifies: LLR-rr558z) — E0432 / E0599 → pass
  red -> green: org-node/tests/verification.rs::a_greater_epoch_with_no_match_is_behind (verifies: LLR-rr558z) — E0599 / E0425 → pass
  red -> green: org-node/tests/verification.rs::in_step_behind_inconsistent_and_refusals_change_no_sealed_byte_and_ask_no_secret (verifies: LLR-qvp7mq) — E0599 → pass
  red -> green: org-node/tests/verification.rs::behind_sets_the_marker_in_step_clears_it_and_unseal_leaves_it_none (verifies: LLR-zk7fx2) — E0599 → pass; extra mutation check: removing `mark(...)` made it red (verification.rs:334), reverted → green
  red -> green: org-node/tests/verification.rs::a_refused_verification_keeps_the_earlier_change_set (verifies: LLR-qvp7mq) — E0599 → pass
  red -> green: org-node/tests/verification.rs::verify_reports_one_outcome_from_the_record_and_the_state (verifies: LLR-rr558z) — E0432 / E0599 → pass
  red -> green: org-node/tests/verification.rs::verify_is_behind_when_no_provisional_organisation_matches_root_and_key (verifies: LLR-rr558z) — E0599 → pass
  red -> green: org-node/tests/sealed_store.rs::saving_needs_no_device_secret (verifies: LLR-bvt5up; test-only fix, beyond the plan's file list) — failed once `verify` existed (["unseal","verify"] vs ["unseal"]); `verify` added to the list as the test's own comment foresaw → pass
  inherited: none
```

#### T15 done

```
  red -> green: org-node/tests/proposals.rs::recording_a_change_appends_it_and_leaves_the_record_byte_for_byte (verifies: LLR-b2hgd4) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::record_admission_gives_the_first_free_preferred_handle_in_ranked_order (verifies: LLR-bwtx3h) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::with_no_free_handle_the_chosen_one_is_parsed_and_given (verifies: LLR-bwtx3h) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::record_admission_refuses_each_case_recording_nothing (verifies: LLR-mq7n6u) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::an_admission_refused_by_the_bound_records_nothing (verifies: REQ-g2537c, LLR-m76r4k, LLR-mq7n6u) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::record_admission_and_record_revocation_refuse_while_behind (verifies: LLR-mq7n6u, LLR-f3hru6, LLR-zk7fx2) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::record_revocation_refuses_an_unheld_or_already_named_member (verifies: LLR-f3hru6) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::revoking_a_member_the_record_does_not_hold_records_nothing (verifies: LLR-f3hru6) — E0599 → pass
  inherited: none
```

#### T16 done

```
  red -> green: org-node/tests/proposals.rs::propose_updated_organisation_applies_every_change_in_order_and_keeps_one_own_provisional_organisation (verifies: LLR-va2yqz) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::the_proposal_keeps_its_applied_changes_and_the_changes_stay_held (verifies: LLR-va2yqz) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::a_key_equal_to_the_records_or_to_a_key_of_the_result_is_refused_keeping_nothing (verifies: LLR-qghs95) — E0599 → pass; mutation check: disabling the record-key equality check made it red, reverted → green
  red -> green: org-node/tests/proposals.rs::propose_updated_organisation_refuses_no_change_no_record_behind_and_the_bound (verifies: LLR-n6ssue) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::the_views_hold_no_private_key_and_mark_the_current_proposal (verifies: LLR-8ru6mv) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::current_proposal_turns_false_after_a_later_change_or_a_new_record (verifies: LLR-8ru6mv) — E0599 → pass; mutation check: dropping the applied == provisional_changes test made it red, reverted → green
  red -> green: org-node/tests/proposals.rs::admission_requests_pairs_each_kept_request_with_its_invitee_name (verifies: LLR-8ru6mv) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::an_admitted_member_carries_exactly_the_kept_requests_values (verifies: LLR-va2yqz, LLR-bwtx3h; rewrite) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::reading_provisional_changes_and_organisations_changes_nothing (verifies: LLR-8ru6mv; rewrite) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::persona_view_copies_each_stores_public_values_and_no_key (verifies: LLR-8ru6mv; decision O1) — E0599 → pass
  red -> green: org-node/tests/proposals.rs::an_updated_organisation_key_equal_to_the_records_or_to_a_key_of_the_result_is_refused (verifies: LLR-qghs95; rewrite) — E0599 → pass; caught by the same mutation as above
  inherited: none
```

#### T17 done

```
  red -> green: org-node/tests/verification.rs::the_match_is_the_one_provisional_organisation_with_the_chains_epoch_root_and_key (verifies: LLR-8528ud) — E0599 no Verified::Adopted; E0432 no DiscardedChange → pass
  red -> green: org-node/tests/verification.rs::own_received_and_answer_provisional_organisations_match_alike (verifies: LLR-8528ud) — E0599 → pass
  red -> green: org-node/tests/verification.rs::adoption_replaces_the_record_and_reports_own_in_one_sealed_change (verifies: LLR-34c5y3) — E0599 → pass
  red -> green: org-node/tests/verification.rs::changes_the_adopted_record_left_out_are_discarded_and_reported_in_order (verifies: LLR-d9x6xf) — E0599 / E0432 → pass
  red -> green: org-node/tests/verification.rs::adoption_drops_provisional_organisations_up_to_its_epoch_and_every_kept_notice (verifies: LLR-x8drb8) — compile red → pass
  red -> green: org-node/tests/verification.rs::an_adoption_discards_the_provisional_organisations_at_or_below_its_epoch (verifies: LLR-x8drb8) — compile red → pass
  red -> green: org-node/tests/verification.rs::every_adoption_keeps_the_change_set_and_base_snapshot_that_produced_the_record (verifies: LLR-p8vpgz, LLR-34c5y3) — compile red → pass
  red -> green: org-node/tests/verification.rs::a_provisional_organisation_whose_key_is_not_the_chains_is_no_match (verifies: LLR-8528ud) — compile red only; mutation check: dropping the org_pub_key equality from `matching` made it red (Adopted vs Behind), reverted → green
  red -> green: org-node/tests/verification.rs::verify_adopts_the_provisional_organisation_the_chain_holds_after_a_restart (verifies: LLR-34c5y3) — E0599 → pass
  red -> green: org-node/tests/verification.rs::an_adoption_that_keeps_this_device_never_asks_for_the_device_secrets (verifies: LLR-34c5y3, LLR-5tvn7g) — E0599 → pass
  red -> green: org-node/tests/store_at_rest.rs::the_kept_change_set_and_base_snapshot_are_the_adopted_ones (verifies: LLR-p8vpgz) — runtime red: verify returned no store change → pass
  inherited: kept_change_set_round_trips_through_the_store — from LLR-d9778a
```

#### T18 done

```
  red -> green: org-node/tests/verification.rs::an_admitted_requests_invite_is_settled_and_its_addresses_move_to_the_address_book (verifies: LLR-rzqs2u) — both requests still kept → pass
  red -> green: org-node/tests/verification.rs::a_settled_invite_refuses_a_later_request_as_not_outstanding (verifies: LLR-rzqs2u, LLR-6gkxpg) — invite still outstanding → pass
  red -> green: org-node/tests/verification.rs::an_adoption_holds_each_dropped_device_as_revoked_once (verifies: LLR-bgj2ff) — revoked_devices lacked slots 0 and 2 → pass
  red -> green: org-node/tests/verification.rs::addresses_for_prefers_the_address_book_then_a_kept_request_then_the_invite (verifies: LLR-pgy2rb) — E0599 no addresses_for → pass
  red -> green: org-node/tests/verification.rs::org_node_keeps_endpoint_addresses_as_opaque_text_and_dials_none (verifies: LLR-ds228z) — E0599 → pass; mutation check on the source-scan half: planting `// SocketAddr` in src/ids.rs made it red, reverted → green
  inherited: none
```

#### T19 done

```
  red -> green: org-node/tests/verification.rs::a_provisional_member_adopts_a_match_listing_its_keys_and_takes_its_member_id_and_handle (verifies: LLR-q4puca) — T17's judge said Behind for a no-record store → pass
  red -> green: org-node/tests/verification.rs::a_match_not_listing_the_provisional_member_is_admission_not_ours (verifies: LLR-q4puca) — unwrap_err on Ok(Behind) → pass
  red -> green: org-node/tests/verification.rs::the_first_records_key_is_the_chains (verifies: LLR-q4puca) — record None, nothing adopted → pass
  red -> green: org-node/tests/verification.rs::a_founding_store_adopts_its_founding_at_epoch_one_with_its_proxy (verifies: LLR-gcm7bq) — got Behind, expected Adopted → pass
  red -> green: org-node/tests/verification.rs::a_first_adoption_holds_no_device_as_revoked (verifies: LLR-bgj2ff; moved from T18) — green on arrival; mutation checks: pushing the founder's Device (adopt_founding) and the joiner's Device (adopt_first) into revoked_devices each made it red, reverted → green
  red -> green: org-node/tests/verification.rs::a_first_admission_whose_record_misses_the_chain_root_adopts_nothing (verifies: REQ-eqt23j, LLR-q4puca; rewrite) — green on arrival; mutation check: matching the no-record arm on epoch only made it red, reverted → green
  red -> green: org-node/tests/verification.rs::a_founding_whose_chain_is_past_epoch_one_is_behind_and_keeps_the_founding (verifies: LLR-gcm7bq, LLR-qvp7mq; rewrite) — green on arrival; mutation check: removing the epoch-1 check made it red, reverted → green
  red -> green: org-node/tests/verification.rs::a_founding_whose_members_do_not_rebuild_the_chain_root_is_not_adopted (verifies: LLR-gcm7bq; rewrite) — green on arrival; mutation check: removing rebuilds(..) made it red, reverted → green
  inherited: none
```

#### T20 done

```
  red -> green: org-node/tests/outgoing_bodies.rs::messages_for_the_current_epoch_inform_every_listed_device_but_the_own_then_notify_every_removed_one (verifies: LLR-degg8t) — E0599 no messages_for_current_epoch → pass
  red -> green: org-node/tests/outgoing_bodies.rs::a_store_with_no_record_is_refused_and_no_body_carries_an_invite_id (verifies: LLR-degg8t) — E0599 → pass
  red -> green: org-node/tests/outgoing_bodies.rs::an_adopted_admission_sends_information_to_every_listed_device_but_the_own (verifies: LLR-degg8t; rewrite) — E0599 → pass
  red -> green: org-node/tests/outgoing_bodies.rs::an_adopted_removal_sends_revocations_to_every_removed_device_after_the_listed_ones (verifies: LLR-degg8t; rewrite) — E0599 → pass
  red -> green: org-node/tests/outgoing_bodies.rs::a_record_with_no_kept_change_set_returns_no_message (verifies: LLR-degg8t; rewrite) — E0599 → pass
  red -> green: org-node/tests/outgoing_bodies.rs::outgoing_message_debug_shows_no_body_byte (verifies: LLR-rsjs8e; rewrite) — E0599 → pass; mutation check: adding the body to Debug made it red, reverted → green
  red -> green: org-node/tests/outgoing_bodies.rs::messages_for_the_current_epoch_change_nothing_and_repeat (verifies: LLR-degg8t; rewrite) — E0599 → pass
  red -> green: org-node/tests/outgoing_bodies.rs::outgoing_message_debug_renders_the_recipient_and_the_length_only (verifies: LLR-rsjs8e) — green on arrival; mutation check: adding the body to Debug made it red, reverted → green
  red -> green: org-node/tests/outgoing_bodies.rs::outgoing_message_is_not_clone (verifies: LLR-rsjs8e; rewrite) — green on arrival; mutation check: #[derive(Clone)] made it red, reverted → green
  inherited: none
```

#### T21 done

```
  red -> green: org-node/tests/invitation.rs::admission_request_and_cancel::build_admission_request_encodes_the_provisional_member_for_the_issuer_with_the_given_addresses (verifies: LLR-g32hjn) — E0599 → pass
  red -> green: org-node/tests/invitation.rs::admission_request_and_cancel::build_admission_request_refuses_no_provisional_member_and_bad_addresses (verifies: LLR-g32hjn, LLR-h4gwns) — E0599 → pass
  red -> green: org-node/tests/invitation.rs::admission_request_and_cancel::cancel_invite_removes_the_request_its_pair_and_its_admit_change (verifies: LLR-ag8hjd) — E0599 → pass
  red -> green: org-node/tests/invitation.rs::admission_request_and_cancel::cancel_invite_refuses_an_invite_with_no_kept_request (verifies: LLR-ag8hjd) — E0599 → pass
  inherited: none
```

#### T22 done

```
  red -> green: org-node/tests/organisation_request.rs::build_organisation_request_addresses_every_listed_device_or_the_invites_issuer (verifies: LLR-vyank2) — E0599 no build_organisation_request → pass
  red -> green: org-node/tests/organisation_request.rs::a_listed_requester_is_answered_with_the_full_record_and_its_key (verifies: LLR-7kdhst) — Err(MalformedMessage) from the transitional arm → pass
  red -> green: org-node/tests/organisation_request.rs::a_revoked_requester_is_answered_with_its_notice_and_nothing_else (verifies: LLR-j338wq) — MalformedMessage → pass
  red -> green: org-node/tests/organisation_request.rs::a_never_listed_requester_is_refused_and_answered_nothing (verifies: LLR-283x6f, LLR-j338wq) — MalformedMessage, not InvalidField request.addresses → pass
  red -> green: org-node/tests/organisation_request.rs::an_answer_over_the_message_bound_is_refused (verifies: LLR-xc9c6w) — MalformedMessage, not MessageTooLarge → pass
  red -> green: org-node/tests/organisation_request.rs::an_answer_is_kept_only_from_a_listed_device_while_behind_and_for_a_later_epoch (verifies: LLR-kd7u2r) — MalformedMessage, not OrganisationNotRequested → pass
  red -> green: org-node/tests/organisation_request.rs::a_request_sets_its_senders_address_book_entry_and_seals_only_on_change (verifies: LLR-h9d65p) — unwrap on Err(MalformedMessage) → pass
  red -> green: org-node/tests/organisation_request.rs::a_provisional_member_keeps_an_expected_answer_only_while_behind (verifies: LLR-e6w3nr) — OrgNotHeld, not OrganisationNotRequested → pass
  red -> green: org-node/tests/receive_path.rs::in_a_store_with_no_record_only_expected_organisation_information_is_decoded (verifies: LLR-e6w3nr; test-only rewrite, plan delta) — old version failed on T22's code; mutation check: `&& false` in the expected-answer branch made the rewrite red, reverted → green
  inherited: none
```

#### T23 done

```
  red -> green: org-node/tests/own_removal.rs::store_content_encodes_its_variant_index_then_its_fields (verifies: LLR-6zaeb6) — no `removal` module → pass
  red -> green: org-node/tests/own_removal.rs::unseal_reparses_a_removed_forms_keys_and_addresses_and_refuses_an_unknown_variant (verifies: LLR-6zaeb6) — no removal module / store_content → pass
  red -> green: org-node/tests/own_removal.rs::into_removed_keeps_only_the_org_id_the_time_and_the_acknowledgements (verifies: LLR-2d5a7m) — no into_removed → pass
  red -> green: org-node/tests/own_removal.rs::a_removal_seals_a_removed_store_for_that_personas_device_key (verifies: LLR-2d5a7m) — no seal_removal_for_test / content → pass
  red -> green: org-node/tests/own_removal.rs::every_other_operation_on_a_removed_store_is_organisation_removed (verifies: LLR-myfk8w) — no removal module → pass
  red -> green: org-node/tests/fuzz_unseal/fuzz_target.rs shape 3 (verifies: LLR-bg538c, LLR-6zaeb6) — green on arrival; mutation check: panicking in unseal's Removed-form arm made it fail on its first input, reverted → green
  inherited: none
```

#### T24 done

```
  red -> green: org-node/tests/own_removal.rs::a_match_not_listing_this_device_is_removed_and_seals_the_removed_store (verifies: LLR-jhy6t6) — compile red, then stubbed: Adopted, expected Removed → pass
  red -> green: org-node/tests/own_removal.rs::a_kept_notice_accepted_against_the_chain_is_removed_after_the_inconsistent_checks (verifies: LLR-jhy6t6) — compile red, then Behind, expected Removed → pass
  red -> green: org-node/tests/own_removal.rs::a_kept_notice_accept_refuses_stays_kept_and_the_judgement_continues (verifies: LLR-jhy6t6) — green on arrival under stubs; mutation check: treating every kept notice as accepted made it red, reverted → green
  red -> green: org-node/tests/own_removal.rs::the_acknowledgement_is_signed_once_with_the_personas_secret_and_kept_per_recipient (verifies: LLR-5tvn7g) — compile red, then no removal change → pass
  red -> green: org-node/tests/own_removal.rs::no_secret_returned_is_device_secret_not_supplied_and_nothing_changes (verifies: LLR-5tvn7g) — compile red, then adopted instead → pass
  red -> green: org-node/tests/own_removal.rs::the_recipient_count_is_three_two_one_or_zero_and_their_order_is_fixed (verifies: LLR-vq5d5n) — compile red, then stub count 0 at n = 1 → pass
  red -> green: org-node/tests/own_removal.rs::a_removal_that_cannot_sign_deletes_nothing (verifies: LLR-5tvn7g) — compile red, then missing DeviceSecretNotSupplied → pass
  red -> green: org-node/tests/own_removal.rs::a_signing_refusal_is_returned_unchanged_with_nothing_deleted (verifies: LLR-5tvn7g) — compile red, then Ok(Behind) → pass
  red -> green: org-node/tests/own_removal.rs::a_missing_device_secret_is_refused_before_anything_is_deleted (verifies: LLR-5tvn7g, LLR-f9mcsq) — compile red, then missing DeviceSecretNotSupplied → pass
  red -> green: org-node/tests/own_removal.rs::a_removal_signs_with_the_supplied_secret_and_an_adoption_reads_none (verifies: LLR-5tvn7g, LLR-f9mcsq) — compile red, then no removal at epoch 5 → pass
  red -> green: org-node/tests/revocation.rs::acknowledgement_domain::acknowledgement_signed_bytes_are_the_domain_then_the_tuple (verifies: LLR-bs6yxf) — green on arrival; mutation check: literal bytes for ACK_DOMAIN made it red, reverted → green
  red -> green: org-node/tests/revocation.rs::accept_decides_only::accept_returns_ok_for_a_later_epoch_without_the_device_and_takes_no_secret_source (verifies: LLR-rhsfw9) — compile red (4-argument accept), then always-refusing stub → pass
  red -> green: org-node/tests/revocation.rs::a_refused_notice_is_refused_in_order_with_no_secret_asked_for (verifies: LLR-rhsfw9; rewrite) — compile red, then wrong refusal order → pass
  inherited: acknowledgement_signed_bytes_are_the_domain_then_the_postcard_tuple — from LLR-gbe9bt
  inherited: a_signature_without_the_acknowledgement_domain_is_refused — from LLR-gbe9bt
  inherited: accept_refuses_in_order_and_changes_nothing — from LLR-tx8ruv, LLR-uw7nmv (closure argument dropped; now LLR-rhsfw9, LLR-f9mcsq)
```

#### T25 done

```
  red -> green: org-node/tests/own_removal.rs::acknowledgement_accepted_removes_that_recipients_acknowledgement_and_returns_the_count_left (verifies: LLR-85kves) — no method acknowledgement_accepted → pass
  red -> green: org-node/tests/own_removal.rs::acknowledgement_accepted_refuses_an_unnamed_recipient_and_an_active_store (verifies: LLR-85kves) — same compile error → pass
  red -> green: org-node/tests/own_removal.rs::kept_acknowledgements_is_none_for_an_active_store_and_lists_a_removed_ones_in_order (verifies: LLR-75sn9g) — unresolved KeptAcknowledgements, no kept_acknowledgements → pass
  inherited: none
```

#### T26 done

```
  red -> green: org-node/tests/encoding_golden.rs::persona_store_plaintext_is_pinned_in_its_active_form (verifies: LLR-mkvx2k, LLR-d925uu, LLR-6zaeb6, LLR-p8vpgz) — pin matches; mutation check: moving `handle` before `surname` in PersonaRecordEncoding made the byte round trip fail, reverted → green
  red -> green: org-node/tests/encoding_golden.rs::a_removed_store_is_pinned_in_its_removed_form (verifies: LLR-mkvx2k, LLR-6zaeb6) — pin matches; mutation check: moving `acknowledgements` before `verified_at` in RemovedStore made it fail, reverted → green
  inherited: organisation_information_wire_message_is_pinned — from LLR-ayrdr8, LLR-ms8njy, LLR-js9dsu (now LLR-mkvx2k, LLR-g3wmen, LLR-xr88a8; body unchanged)
  inherited: truncated_pinned_values_are_refused — from LLR-ayrdr8 (now LLR-mkvx2k; store half rewritten over the new pins, wire half unchanged)
```

#### T27 done

```
  red -> green: org-node/tests/sealed_store.rs::store_change_properties::every_store_changing_operation_returns_one_sealed_change_with_a_fresh_nonce (verifies: LLR-ay8nvu) — green on arrival; mutation check: dropping the nonce fill in seal made it red, reverted → green
  red -> green: org-node/tests/sealed_store.rs::store_change_properties::store_change_has_no_forgotten_variant_and_a_refusal_returns_none (verifies: LLR-ay8nvu) — green on arrival; mutation checks: removing #[must_use], and sealing on verify's Done branch, each made it red, reverted → green
  red -> green: org-node/tests/sealed_store.rs::store_change_properties::no_sealed_store_holds_the_device_seed (verifies: LLR-ay8nvu) — green on arrival; mutation check: storing the Device seed as member_seed made it red, reverted → green
  red -> green: org-node/tests/sealed_store.rs::store_change_properties::saving_needs_no_device_secret (verifies: LLR-ay8nvu) — green on arrival; mutation check: making add_store pub made it red, reverted → green
  red -> green: org-node/tests/absences.rs::into_removed_is_called_only_from_verify (verifies: LLR-9jn3z8) — green on arrival; mutation check: an into_removed call in cancel_invite made it red, reverted → green
  red -> green: org-node/tests/absences.rs::into_removed_is_called_only_from_verify_and_an_acknowledgement_removed_only_on_acceptance (verifies: LLR-9jn3z8) — green on arrival; mutation check: acknowledgements.clear() in kept_acknowledgements made it red, reverted → green
  red -> green: org-node/tests/absences.rs::only_the_removal_and_the_accepted_acknowledgement_delete (verifies: LLR-9jn3z8) — green on arrival; mutation checks: stores.clear() in restore_store, and record = None in record_revocation, each made it red, reverted → green
  red -> green: org-node/tests/absences.rs::verification_revocation_and_removal_do_no_io_and_read_no_clock (verifies: LLR-g2jeqd) — green on arrival; mutation checks: SystemTime::now() in judge, and a pub(crate) &mut StoreData helper, each made it red, reverted → green
  red -> green: org-node/tests/own_removal.rs::on_an_active_store_nothing_but_the_removal_deletes_and_each_operation_removes_only_what_it_names (verifies: LLR-f9mcsq) — green on arrival; mutation checks: address_book.clear() in cancel_invite, and clearing provisional changes on Inconsistent, each made it red, reverted → green
  red -> green: org-node/tests/secret_redaction.rs::organisation_key_custody::a_provisional_private_key_leaves_the_store_only_inside_a_message_to_a_listed_device (verifies: LLR-qw8tyu) — green on arrival; mutation checks: the answer carrying a provisional key, and un-gating the key accessor, each made it red, reverted → green
  inherited: no_sealed_store_holds_the_device_seed — from LLR-bvt5up
  inherited: saving_needs_no_device_secret — from LLR-bvt5up
```

#### T28 done

```
  red -> green: org-io/tests/rng_source.rs::every_org_node_draw_and_the_temporary_suffix_come_from_the_handles_source (verifies: LLR-4qd5fx) — compile red (lib broken against P2); mutation check: HandleRng::os() for &self.rng made it red, reverted → green
  red -> green: org-io/tests/absences.rs::no_file_outside_rng_and_test_support_names_a_generator (verifies: LLR-4qd5fx) — compile red; mutation check: OsRng drawn in store_file.rs made it red, reverted → green
  red -> green: org-io/tests/absences.rs::no_public_method_takes_or_returns_a_random_source (verifies: LLR-ctm86b) — compile red; mutation check: a planted pub fn taking &mut dyn RngCore made it red, reverted → green
  red -> green: org-io/tests/rng_source.rs::a_seeded_source_reproduces_a_draw_under_test_support (verifies: LLR-5acgfg) — compile red (no with_rng_for_test); mutation check: ignoring the seed made it red, reverted → green
  red -> green: org-io/tests/rng_source.rs::a_failing_entropy_source_refuses_the_store_write_and_writes_nothing (verifies: LLR-94nn6q) — compile red; mutation check: falling back to OsRng on a failed draw made it red, reverted → green
  red -> green: org-io/tests/receive_step.rs::a_malformed_body_is_not_terminal (verifies: LLR-szhe7j; rewrite) — compile red; mutation check: mapping the refusal to Terminal made it red, reverted → green
  red -> green: org-io/tests/receive_step.rs::a_connection_failing_before_a_body_is_not_terminal (verifies: LLR-szhe7j; rewrite) — compile red; mutation check: mapping a connection error to Terminal made it red, reverted → green
  inherited: iroh_is_named_only_in_the_transport_module_and_test_support — from LLR-3vb4k9 (re-pointed to LLR-4k43dm)
```

#### T29 done

```
  red -> green: org-io/tests/person.rs::propose_person_writes_the_keychain_item_then_the_person_file (verifies: LLR-3cysjk) — no propose_person / person on OrgIo → pass
  red -> green: org-io/tests/person.rs::a_refused_name_writes_nothing_and_a_failed_file_write_leaves_the_item (verifies: LLR-3cysjk) — no propose_person, no StoreFailure::NotKept → pass
  red -> green: org-io/tests/person.rs::a_second_person_is_refused_naming_the_one_held_and_writes_nothing (verifies: LLR-3cysjk) — no CreateError::PersonAlreadyHeld → pass
  red -> green: org-io/tests/person.rs::open_refuses_two_person_files_naming_both_and_touches_nothing (verifies: LLR-z7kbf9) — no OpenError::MoreThanOnePerson → pass
  red -> green: org-io/tests/person.rs::every_person_summary_is_provisional (verifies: LLR-mas8vc) — unresolved PersonStatus → pass
  red -> green: org-io/tests/person.rs::open_lists_person_files_apart_and_refuses_one_whose_item_is_missing (verifies: LLR-z7kbf9) — no propose_person → pass
  red -> green: org-io/tests/person.rs::no_endpoint_is_bound_for_the_persons_device_key (verifies: LLR-g6kaa8) — no propose_person → pass; mutation check: binding an endpoint for the Person's key made it red, reverted → green
  red -> green: org-io/tests/absences.rs::org_io_names_no_person_crate (verifies: LLR-b97usg) — green on arrival; mutation checks: naming person::Person in view.rs, and a person entry in Cargo.toml, each made it red, reverted → green
  inherited: none
```

#### T30 done

```
  red -> green: org-io/tests/persona_creation.rs::each_proposal_creates_a_new_persona_item_first_then_file (verifies: LLR-kna4zj) — compile red → pass
  red -> green: org-io/tests/persona_creation.rs::a_refusal_writes_nothing_and_a_keychain_or_file_failure_is_reported (verifies: LLR-kna4zj) — compile red → pass
  red -> green: org-io/tests/persona_creation.rs::propose_founding_organisation_returns_the_founder_epoch_root_and_key_and_touches_no_chain (verifies: LLR-4yte89) — compile red → pass
  red -> green: org-io/tests/persona_creation.rs::propose_member_parses_the_invite_and_sends_nothing (verifies: LLR-6d6wfw) — compile red → pass
  red -> green: org-io/tests/persona_creation.rs::a_persona_created_while_receiving_is_bound_and_accepted_on (verifies: LLR-jvx6j9) — compile red → pass
  red -> green: org-io/tests/persona_creation.rs::organisations_report_their_stored_name_and_no_send_carries_it (verifies: LLR-pn3hgc) — compile red (no method; no field name) → pass
  red -> green: org-io/tests/persona_creation.rs::propose_founding_writes_the_item_before_the_file_and_nothing_when_the_keychain_refuses (verifies: LLR-kna4zj) — compile red → pass
  red -> green: org-io/tests/expectations_met.rs::outstanding_invites_are_kept_in_the_store_files_and_settled_by_adoption (verifies: REQ-2xgsbn) — compile red → pass
  red -> green: org-io/tests/expectations_met.rs::an_invite_that_does_not_parse_changes_no_file (verifies: REQ-2xgsbn) — compile red → pass
  inherited: the_write_lends_the_bytes_only_through_lend_to_keychain — from LLR-n9dnm4 (re-pointed to LLR-kna4zj)
```

#### T31 done

```
  red -> green: org-io/tests/invitation_exchange.rs::issue_invite_builds_in_the_holding_persona_and_writes_before_it_returns (verifies: LLR-95e3jx) — E0599 → pass
  red -> green: org-io/tests/invitation_exchange.rs::the_invite_carries_the_issuing_endpoints_addresses_in_the_current_mode (verifies: LLR-v9dkrv) — E0599 → pass
  red -> green: org-io/tests/invitation_exchange.rs::a_failed_bind_refuses_the_invite_and_writes_nothing (verifies: LLR-v9dkrv) — E0599 → pass
  red -> green: org-io/tests/invitation_exchange.rs::read_invite_returns_the_public_fields_and_writes_nothing (verifies: LLR-npw5zg) — E0599 → pass
  red -> green: org-io/tests/invitation_exchange.rs::cancel_invite_writes_the_store_and_refuses_an_invite_with_no_kept_request (verifies: LLR-8ze6cc) — E0599 → pass
  red -> green: org-io/tests/invitation_exchange.rs::admission_requests_lists_every_kept_request_with_its_invitee_name_and_changes_nothing (verifies: LLR-dm2h8u) — E0599 → pass
  red -> green: org-io/tests/invitation_exchange.rs::outstanding_invite_ids_survive_a_reopen_and_leave_only_by_adoption_or_cancel (verifies: LLR-95e3jx; rewrite) — E0599 → pass
  red -> green: org-io/tests/invitation_exchange.rs::an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id (verifies: LLR-95e3jx) — E0599 → pass
  red -> green: org-io/tests/invitation_exchange.rs::no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to (verifies: LLR-95e3jx) — E0599 → pass
  red -> green: org-io/tests/invitation_exchange.rs::an_outstanding_invites_file_in_the_data_dir_is_never_read (verifies: LLR-95e3jx) — E0599 → pass
  red -> green: org-io/tests/invitation_exchange.rs::the_invite_is_returned_only_after_its_store_write (verifies: LLR-95e3jx) — E0599 → pass
  inherited: an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id — from LLR-9sraks
  inherited: no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to — from LLR-9sraks
  inherited: an_outstanding_invites_file_in_the_data_dir_is_never_read — from LLR-f35pda
  inherited: the_invite_is_returned_only_after_its_store_write — from LLR-h9qf3s
```

#### T32 done

```
  red -> green: org-io/tests/propose.rs::propose_admission_records_the_change_with_the_handle_given_and_writes_first (verifies: LLR-un47j8) — no propose_admission, no ProvisionalChangeSummary → pass
  red -> green: org-io/tests/propose.rs::an_org_node_refusal_of_an_admission_writes_nothing (verifies: LLR-un47j8) — no propose_admission → pass
  red -> green: org-io/tests/propose.rs::propose_revocation_records_the_change_and_sends_and_writes_nothing_to_the_chain (verifies: LLR-6kb33y) — no propose_revocation → pass
  red -> green: org-io/tests/propose.rs::propose_updated_organisation_returns_the_epoch_root_and_key_and_makes_no_chain_call (verifies: LLR-5fchwt) — no propose_updated_organisation, no ProposedOrganisation → pass
  red -> green: org-io/tests/propose.rs::a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_writes_nothing (verifies: LLR-6kb33y; re-created from LLR-nqdr97's cut test) — no propose_revocation → pass
  inherited: none
```

#### T33 done

```
  red -> green: org-io/tests/chain_write.rs::an_update_is_written_and_reported_included_with_no_state_read_and_no_adoption (verifies: LLR-s7rq5x) — compile red (no write_organisation_to_chain, no WriteReport) → pass
  red -> green: org-io/tests/chain_write.rs::not_held_no_proxy_and_stale_are_refused_before_any_chain_call (verifies: LLR-48me7m) — compile red → pass; mutation check: disabling the Stale refusal made it red, reverted → green
  red -> green: org-io/tests/chain_write.rs::an_unknown_outcome_refuses_a_second_write_for_that_target_only (verifies: LLR-mhn4n5) — compile red → pass; mutation check: not recording the unknown write made it red, reverted → green
  red -> green: org-io/tests/chain_write.rs::a_write_past_ninety_seconds_is_unknown_and_recorded (verifies: LLR-d3farr, LLR-7y46be) — compile red → pass; same mutation → red, reverted
  red -> green: org-io/tests/chain_write.rs::the_window_is_read_once_and_passed_and_its_failure_writes_nothing (verifies: LLR-a66ufr) — compile red → pass
  red -> green: org-io/tests/chain_write.rs::the_unknown_write_holds_root_key_expected_epoch_and_last_valid_block (verifies: LLR-7y46be) — compile red → pass; same mutation → red, reverted
  red -> green: org-io/tests/chain_write.rs::a_failed_window_read_writes_nothing (verifies: LLR-a66ufr) — compile red → pass
  red -> green: org-io/tests/chain_write.rs::a_refused_write_records_no_unknown_write (verifies: LLR-a66ufr) — compile red → pass
  red -> green: org-io/tests/chain_write.rs::an_unknown_write_refuses_a_second_write_for_that_target_only (verifies: LLR-mhn4n5) — compile red → pass; same mutation → red, reverted
  red -> green: org-io/tests/chain_write.rs::a_refused_update_write_keeps_the_provisional_organisation_and_sends_nothing (verifies: LLR-s7rq5x) — compile red → pass
  red -> green: org-io/tests/chain_write.rs::a_write_that_never_finishes_times_out_as_unknown (verifies: LLR-d3farr) — compile red → pass
  red -> green: org-io/tests/chain_write.rs::a_handle_with_no_chain_refuses_every_write (verifies: LLR-s7rq5x) — compile red → pass
  red -> green: org-io/tests/chain_write.rs::a_stale_provisional_organisation_is_refused_before_any_write (verifies: LLR-48me7m) — compile red → pass; Stale mutation → red, reverted
  red -> green: org-io/tests/chain_write.rs::a_write_for_an_organisation_no_persona_holds_is_refused_as_not_held (verifies: LLR-s7rq5x) — compile red → pass
  inherited: none
```

#### T34 done

```
  red -> green: org-io/tests/chain_write.rs::a_founding_creates_the_proxy_records_it_and_only_then_founds_at_it (verifies: LLR-5tpdxw) — Err(NotHeld) from T33's founding arm → pass
  red -> green: org-io/tests/chain_write.rs::a_failed_store_write_after_create_proxy_stops_and_names_the_proxy (verifies: LLR-5tpdxw) — Err(NotHeld) → pass
  red -> green: org-io/tests/chain_write.rs::a_founding_with_a_stored_proxy_skips_create_proxy (verifies: LLR-5tpdxw) — Err(NotHeld) → pass
  red -> green: org-io/tests/chain_write.rs::a_refused_found_at_step_is_refused_naming_it_and_keeps_the_proxy (verifies: LLR-sq5pbq) — Err(NotHeld) → pass
  red -> green: org-io/tests/chain_write.rs::a_failed_create_proxy_stores_no_proxy (verifies: LLR-sq5pbq) — Err(NotHeld) → pass
  red -> green: org-io/tests/chain_write.rs::a_founding_write_creates_the_proxy_records_it_then_founds_at_it (verifies: LLR-5tpdxw; rewrite) — Err(NotHeld) → pass
  red -> green: org-io/tests/chain_write.rs::a_refused_genesis_step_is_reported_refused_naming_the_step (verifies: LLR-sq5pbq; rewrite) — Err(NotHeld) → pass
  red -> green: org-io/tests/signatory_key.rs::a_build_without_dev_seed_never_asks_for_the_seed_on_a_write_or_a_read (verifies: LLR-rgdx22; rewrite, plan delta) — Err(NotHeld) → pass
  inherited: none
```

#### T35 done

```
  red -> green: org-io/tests/verification.rs::verify_organisation_reads_the_block_then_the_state_once_and_maps_each_outcome (verifies: LLR-t2pve7) — compile red (no org_io::verify) → pass
  red -> green: org-io/tests/verification.rs::a_failed_read_or_parse_is_unverified_calls_no_org_node_and_writes_nothing (verifies: LLR-ce42ce) — compile red → pass
  red -> green: org-io/tests/verification.rs::verification_returns_the_last_outcome_and_unverified_before_any (verifies: LLR-ce42ce) — compile red → pass
  red -> green: org-io/tests/verification.rs::the_time_given_is_one_clock_reading_after_the_read (verifies: LLR-fzf2n3) — compile red (no TestClock) → pass
  red -> green: org-io/tests/verification.rs::a_clock_before_the_epoch_is_unverified (verifies: LLR-fzf2n3) — compile red → pass
  red -> green: org-io/tests/verification.rs::a_test_clock_replaces_the_handles_clock_under_test_support_only (verifies: LLR-6vtbn6) — compile red → pass
  red -> green: org-io/tests/verification.rs::a_verification_that_adopts_an_unknown_write_reports_it_adopted_and_clears_it (verifies: LLR-t2pve7, LLR-zey4mg) — compile red → pass
  red -> green: org-io/tests/verification.rs::a_write_that_executed_before_a_restart_is_adopted_by_the_next_verification (verifies: LLR-t2pve7) — compile red → pass
  red -> green: org-io/tests/verification.rs::an_unreachable_chain_reports_every_target_unverified_and_writes_nothing (verifies: LLR-ce42ce) — compile red → pass
  red -> green: org-io/tests/absences.rs::org_nodes_verification_is_called_only_from_verify_rs (verifies: LLR-3ame3g) — runtime red: no verify.rs → pass
  inherited: org_io_hands_out_no_bare_service — from LLR-7pj5af, LLR-9r2bxd (re-pointed to LLR-4usfu2, G5)
```

#### T36 done

```
  red -> green: org-io/tests/verification.rs::a_founding_is_read_at_its_stored_proxy_and_none_yet_is_no_proxy_yet (verifies: LLR-4qyeut) — Unverified{NotHeld}, expected Unverified{NoProxyYet} → pass
  red -> green: org-io/tests/verification.rs::verify_all_verifies_every_target_in_store_file_order_whatever_fails (verifies: LLR-kq7z58) — compile red, then an empty stub returned [] → pass
  red -> green: org-io/tests/verification.rs::verify_all_reports_every_target_in_store_file_order (verifies: LLR-kq7z58; re-created from startup_reconcile.rs:163) — compile red, then stub len 0 → pass
  red -> green: org-io/tests/verification.rs::a_provisional_member_reads_the_invites_organisation_and_adopts_nothing_itself (verifies: LLR-fp2cat) — Unverified{NotHeld}, expected Behind → pass
  red -> green: org-io/tests/verification.rs::discarded_lists_each_left_out_change_with_its_kind_names_and_handle (verifies: LLR-gaanm2) — discarded was [] → pass
  red -> green: org-io/tests/verification.rs::adopted_and_removed_write_the_returned_store_and_the_others_write_nothing (verifies: LLR-zcw3xv) — green on arrival (T35's writes); mutation check: filtering out the store change made it red, reverted → green
  red -> green: org-io/tests/verification.rs::a_failed_write_is_verified_store_and_reloads_the_persona (verifies: LLR-zcw3xv) — green on arrival; same mutation → red, reverted → green
  inherited: none
```

#### T37 done

```
  red -> green: org-io/tests/verification.rs::an_unknown_write_is_settled_adopted_not_executed_deprecated_or_still_unknown (verifies: LLR-zey4mg) — no Verified::Settled, no WriteOutcome → pass
  red -> green: org-io/tests/verification.rs::a_foundings_unknown_write_is_settled_only_adopted_or_still_unknown (verifies: LLR-zey4mg) — same → pass
  red -> green: org-io/tests/verification.rs::an_unknown_write_whose_chain_moved_elsewhere_is_deprecated (verifies: LLR-zey4mg) — same → pass
  red -> green: org-io/tests/verification.rs::a_failed_read_leaves_an_unknown_write_still_unknown (verifies: LLR-zey4mg) — same → pass
  red -> green: org-io/tests/verification.rs::within_the_window_an_unknown_write_stays_still_unknown (verifies: LLR-zey4mg) — same → pass
  red -> green: org-io/tests/verification.rs::past_the_window_at_the_base_epoch_the_write_is_not_executed (verifies: LLR-zey4mg) — same → pass
  red -> green: org-io/tests/verification.rs::nothing_is_discarded_on_behind_inconsistent_unverified_or_a_refused_write (verifies: LLR-pbng7d) — compile red only; mutation check: a planted discard on non-adopting outcomes made it red, reverted → green
  red -> green: org-io/tests/verification.rs::a_verification_that_adopts_nothing_discards_nothing (verifies: LLR-pbng7d) — same mutation → red, reverted → green
  red -> green: org-io/tests/verification.rs::the_secrets_closure_reads_the_keychain_only_to_sign_and_a_refusal_is_verified_keychain (verifies: LLR-8w2z64) — green on arrival; mutation check: dropping the noted refusal made it red, reverted → green
  red -> green: org-io/tests/chain_read_rule.rs::the_state_reader_is_named_only_in_verify_and_preflight (verifies: LLR-k3zmqb) — green on arrival; mutation check: a planted state read in write.rs made it red, reverted → green
  inherited: none
```

#### T38 done

```
  red -> green: org-io/tests/send_fanout.rs::send_organisation_sends_org_nodes_messages_in_order_to_every_recipient_and_writes_no_store (verifies: LLR-pxqdy4) — no send_organisation, no Delivery / SendRefused → pass
  red -> green: org-io/tests/send_fanout.rs::loopback_refuses_no_address_and_no_address_of_the_recipient_per_recipient (verifies: LLR-34tfm8) — no send_organisation → pass
  red -> green: org-io/tests/send_fanout.rs::networked_dials_the_recipients_key_with_its_kept_addresses_as_a_hint (verifies: LLR-34tfm8) — no send_organisation → pass
  red -> green: org-io/tests/send_fanout.rs::every_send_is_from_the_sending_personas_endpoint (verifies: LLR-hx5c2y) — no send_organisation → pass
  red -> green: org-io/tests/send_fanout.rs::an_organisation_no_persona_holds_is_not_held_and_no_recipient_is_an_empty_report (verifies: LLR-zrgj8z) — no send_organisation / SendRefused → pass
  red -> green: org-io/tests/send_fanout.rs::the_verdict_byte_maps_to_delivered_refused_already_received_invite_used_or_not_delivered (verifies: LLR-fwz3tb) — no Verdict::AlreadyReceived / InviteUsedByAnotherDevice / from_byte → pass
  red -> green: org-io/tests/send_fanout.rs::a_failed_send_writes_no_store_and_leaves_the_record (verifies: LLR-pxqdy4) — no send_organisation → pass
  red -> green: org-io/tests/send_fanout.rs::each_recipient_is_dialled_at_the_addresses_kept_for_its_key (verifies: LLR-34tfm8) — no send_organisation → pass
  red -> green: org-io/tests/expectations_met.rs::each_recipient_receives_the_body_org_node_built_for_the_adopted_epoch (verifies: REQ-u894ty) — no Delivery / send_organisation → pass
  red -> green: org-io/tests/expectations_met.rs::the_last_other_member_removed_is_still_sent_its_revocation_by_send_organisation (verifies: REQ-u894ty) — no send_organisation → pass
  inherited: none
```

#### PR-8ghk9s fixed

```
  red -> green: org-io/tests/chain_write.rs::a_create_proxy_timeout_records_no_unknown_write_and_the_next_write_creates_again (verifies: LLR-sq5pbq, LLR-d3farr; PR-8ghk9s) — red: "not refused: Unknown { reason: the write timed out after 90 s and may yet execute on chain }" → pass
```

#### T39 done

```
  red -> green: org-io/tests/requests.rs::request_admission_binds_builds_with_its_own_addresses_and_sends_to_the_issuer (verifies: LLR-adgk9z) — no request_admission, no RequestError → pass
  red -> green: org-io/tests/requests.rs::request_admission_reports_each_outcome_and_writes_nothing (verifies: LLR-adgk9z) — same → pass
  red -> green: org-io/tests/requests.rs::loopback_dials_only_an_invite_address_of_the_issuer (verifies: LLR-qa4nrt) — no request_admission → pass
  red -> green: org-io/tests/requests.rs::request_organisation_asks_every_listed_device_or_the_invites_issuer (verifies: LLR-uf5cyy) — no request_organisation → pass
  inherited: none
```

#### T40 done

```
  red -> green: org-io/tests/receive_step.rs::next_received_writes_the_store_change_before_it_reports_and_reads_no_chain (verifies: LLR-kz9hj4) — compile red (no ProvisionalOrganisationReceived) → pass; M2 → red
  red -> green: org-io/tests/receive_step.rs::received_reports_public_data_only_and_an_already_held_one_reports_nothing (verifies: LLR-x36szv) — compile red → pass
  red -> green: org-io/tests/receive_step.rs::every_completed_handshake_is_accepted_with_no_allow_list (verifies: LLR-j467e9) — green on arrival; M1 → red, reverted → green
  red -> green: org-io/tests/receive_step.rs::exactly_one_verdict_byte_is_written_per_message (verifies: LLR-hhgdk7) — green on arrival; M1 and M3 → red, reverted → green
  red -> green: org-io/tests/receive_step.rs::a_first_admission_request_is_written_then_answered_0x00_and_repeats_0x02_or_0x03 (verifies: LLR-zkybw7) — compile red → pass; M3 → red
  red -> green: org-io/tests/receive_step.rs::a_command_completes_while_next_received_waits (verifies: LLR-eeh4pq) — green on arrival; M4 → red (Elapsed), reverted → green
  red -> green: org-io/tests/receive_step.rs::a_message_the_check_refuses_reads_no_chain_and_writes_nothing (verifies: LLR-kz9hj4) — green on arrival; M2 → red, reverted → green
  red -> green: org-io/tests/receive_step.rs::organisation_information_from_an_unlisted_device_is_refused_with_no_read (verifies: LLR-kz9hj4) — green on arrival; M1 and M2 → red, reverted → green
  red -> green: org-io/tests/receive_step.rs::an_acknowledgement_is_decided_with_no_read (verifies: LLR-kz9hj4) — compile red (no Received::Acknowledged) → pass; M2 → red
  red -> green: org-io/tests/receive_step.rs::an_acknowledgement_from_another_device_is_refused_with_no_read (verifies: LLR-kz9hj4) — green on arrival; M1 and M2 → red, reverted → green
  red -> green: org-io/tests/receive_step.rs::a_refused_check_reads_no_chain (verifies: LLR-kz9hj4) — green on arrival; M2 → red, reverted → green
  red -> green: org-io/tests/receive_step.rs::the_sender_org_node_sees_is_the_handshakes (verifies: LLR-kz9hj4) — compile red → pass
  red -> green: org-io/tests/receive_step.rs::an_acknowledgement_is_settled_with_no_chain_read_and_no_store_write (verifies: LLR-kz9hj4) — compile red → pass; M2 → red
  red -> green: org-io/tests/receive_step.rs::the_store_write_lands_before_the_outcome_returns (verifies: LLR-kz9hj4) — compile red → pass
  red -> green: org-io/tests/acknowledgement_inbound.rs::a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted (verifies: LLR-j467e9, LLR-hhgdk7) — compile red → pass
  red -> green: org-io/tests/acknowledgement_inbound.rs::a_revoked_device_sending_anything_but_an_acknowledgement_or_an_organisation_request_is_refused (verifies: LLR-j467e9, LLR-hhgdk7) — green on arrival; M1 → red, reverted → green
  red -> green: org-io/tests/acknowledgement_inbound.rs::an_acknowledgement_a_member_still_listing_the_device_refuses_is_answered_refused (verifies: LLR-j467e9, LLR-hhgdk7) — compile red → pass; M1 → red
  red -> green: org-io/tests/acknowledgement_inbound.rs::an_acknowledgement_relayed_by_another_device_is_refused (verifies: LLR-j467e9) — green on arrival; M1 → red, reverted → green
  red -> green: org-io/tests/acknowledgement_inbound.rs::a_dropped_responder_writes_nothing (verifies: LLR-hhgdk7) — green on arrival; M5 → red (Ok([1])), reverted → green
  red -> green: org-io/tests/app_boundary.rs::the_app_calls_no_adopting_operation_and_builds_no_service (verifies: LLR-kz9hj4) — green on arrival; a planted node.verify in app commands.rs → red, reverted → green
  red -> green: org-io/tests/expectations_met.rs::one_receive_path_keeps_organisation_information_an_admission_request_and_a_notice (verifies: REQ-2yw5y3) — compile red → pass
  red -> green: org-io/tests/expectations_met.rs::a_body_the_check_refuses_reads_no_chain_and_writes_nothing (verifies: REQ-2yw5y3) — green on arrival; M2 → red, reverted → green
  inherited: a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted — from LLR-wvgec4, LLR-unqfj5
  inherited: an_acknowledgement_a_member_still_listing_the_device_refuses_is_answered_refused — from LLR-wvgec4, LLR-unqfj5
  inherited: an_acknowledgement_relayed_by_another_device_is_refused — from LLR-wvgec4
  inherited: a_dropped_responder_writes_nothing — from LLR-unqfj5
  inherited: the_accept_path_has_no_allow_list — from LLR-wvgec4 (body kept, re-pointed to LLR-j467e9)
  inherited: nothing_but_the_verdict_is_written_on_an_inbound_stream — from LLR-unqfj5 (body kept, re-pointed to LLR-hhgdk7)
  inherited: a_body_the_check_refuses_reads_no_chain_and_writes_nothing — from REQ-tzr3dk
  inherited: a_refused_check_reads_no_chain — from LLR-cvyyr8
  inherited: the_sender_org_node_sees_is_the_handshakes — from LLR-cvyyr8
  inherited: an_acknowledgement_is_settled_with_no_chain_read_and_no_store_write — from LLR-cvyyr8
  inherited: the_store_write_lands_before_the_outcome_returns — from LLR-cvyyr8
```

#### T42 done

```
  red -> green: org-io/tests/acknowledgements.rs::a_removed_outcome_starts_one_delivery_task_and_returns_without_awaiting_it (verifies: LLR-zzsqj4) — compile red, then a stub returning 0: nothing sent → pass
  red -> green: org-io/tests/acknowledgements.rs::org_io_learns_and_removes_acknowledgements_only_through_org_node (verifies: LLR-sn5ded) — no arrival at the recipient → pass
  red -> green: org-io/tests/acknowledgements.rs::acknowledgements_go_from_the_removed_personas_own_identity_over_a_direct_bind (verifies: LLR-cqr26z) — "reached over a direct dial" failed, nothing sent → pass
  red -> green: org-io/tests/acknowledgements.rs::a_round_tries_each_recipients_addresses_in_order_until_accepted (verifies: LLR-3w22wc; rewritten, plan delta) — nothing sent → pass; mutation check: removing the `break` on Delivered made it red, restored → green
  inherited: none
```

#### T41 done

```
  red -> green: org-io/tests/receive_step.rs::an_answer_is_written_0x00_then_sent_on_a_new_connection_to_the_kept_addresses (verifies: LLR-898z24) — pre-T41 receiver wrote 0x01 and sent no answer (requester timed out) → pass
  red -> green: org-io/tests/receive_step.rs::a_revoked_requester_gets_its_notice_through_the_send_path (verifies: LLR-898z24) — no notice arrived → pass
  red -> green: org-io/tests/receive_step.rs::a_never_listed_requester_gets_0x01_and_no_answer (verifies: LLR-236rf9) — green on arrival; mutation check: the check-refusal verdict as Accepted made it red, reverted → green
  inherited: none
```

#### T43 done

```
  red -> green: org-io/tests/acknowledgements.rs::an_accepted_acknowledgement_is_deleted_by_org_node_and_written_before_the_next_recipient (verifies: LLR-6ux6yw) — compile red, then stubs: no acceptance written → pass
  red -> green: org-io/tests/acknowledgements.rs::any_other_answer_or_not_delivered_keeps_the_acknowledgement (verifies: LLR-xwz3nu) — stubs: no report (Elapsed) → pass
  red -> green: org-io/tests/acknowledgements.rs::rounds_are_due_at_t_one_five_thirty_minutes_then_hourly_and_missed_ones_are_not_caught_up (verifies: LLR-bv6wkz) — stubs: next(t) None, expected t+60 → pass
  red -> green: org-io/tests/acknowledgements.rs::acknowledgement_report_lists_each_kept_recipients_outcome_next_round_and_deadline (verifies: LLR-rav7dr) — stubs: no report → pass
  red -> green: org-io/tests/acknowledgements.rs::start_receiving_resumes_each_kept_removal_anchored_at_its_verification_time (verifies: LLR-5rxqr8) — stubs: nothing resumed → pass
  red -> green: org-io/tests/acknowledgements.rs::start_receiving_resumes_no_removal_past_its_deadline (verifies: LLR-5rxqr8; added abnormal case) — green on arrival; mutation check: `|| true` on resume's deadline made it red, reverted (confirmed by grep, pre-commit) → green
  inherited: none
```

#### PR-8sgsb5 fixed

```
  red -> green: org-node/tests/verification.rs::a_first_adoption_keeps_the_issuers_addresses_in_the_address_book (verifies: LLR-q4puca, LLR-h9d65p; PR-8sgsb5) — address_book was [] → pass
  red -> green: org-io/tests/acknowledgement_inbound.rs::a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted (verifies: LLR-j467e9, LLR-hhgdk7; rewritten to drive the real delivery) — hung after the fix; mutation check: reverting the org-node fix made it fail at journey::next's 30 s limit → restored → green
  red -> green: org-io/tests/acknowledgement_inbound.rs::a_dropped_responder_writes_nothing / a_revoked_device_sending_anything_but_an_acknowledgement_or_an_organisation_request_is_refused / an_acknowledgement_relayed_by_another_device_is_refused (verifies: LLR-j467e9, LLR-hhgdk7; rewritten) — hung after the fix; mutation check: removing the dead-address steps made each fail within its limit, restored → green
  red -> green: org-io/tests/receive_step.rs::an_acknowledgement_from_another_device_is_refused_with_no_read / an_acknowledgement_is_settled_with_no_chain_read_and_no_store_write (verifies: LLR-kz9hj4; rewritten) — hung after the merge; same mutation → fail within limit, restored → green
```

#### T44 done

```
  red -> green: org-io/tests/acknowledgements.rs::at_the_deadline_the_file_is_marked_removed_then_the_item_then_the_file_is_removed (verifies: LLR-vj2nkb) — Elapsed, nothing deleted → pass
  red -> green: org-io/tests/acknowledgements.rs::no_acknowledgement_left_deletes_the_persona_at_once (verifies: LLR-n649th) — Elapsed → pass
  red -> green: org-io/tests/acknowledgements.rs::a_failed_step_is_reported_and_retried_hourly_and_at_open (verifies: LLR-3fxxpp) — no failure reported → pass
  red -> green: org-io/tests/acknowledgements.rs::a_refused_keychain_deletion_is_reported_and_retried_hourly (verifies: LLR-3fxxpp) — no failure reported → pass
  red -> green: org-io/tests/acknowledgements.rs::a_leftover_removed_file_is_retried_hourly_once_receiving_starts (verifies: LLR-3fxxpp; added) — Elapsed, no retry existed → pass
  red -> green: org-io/tests/open.rs::open_finishes_removed_files_and_deletes_expired_or_empty_removals (verifies: LLR-upqj5g) — "the leftover file removed" failed → pass
  red -> green: org-io/tests/open.rs::a_persona_file_without_its_item_is_still_refused_and_untouched (verifies: LLR-upqj5g) — green on arrival; mutation check: a planted mark_removed for a store with no item made it red, reverted → green
  red -> green: org-io/tests/acknowledgements.rs::an_accepted_acknowledgement_is_deleted_by_org_node_and_written_before_the_next_recipient (verifies: LLR-6ux6yw; tail rewritten) — green before and after; mutation check: parking deliver forever when none is left made it red, reverted → green
  inherited: none
```

#### T45 done

```
  red -> green: org-io/tests/absences.rs::the_handles_production_methods_are_exactly_the_domain_surface (verifies: LLR-4usfu2) — listed persona_public_keys → pass after gating it
  red -> green: org-io/tests/absences.rs::every_returned_type_is_in_the_handle_return_allowlist_and_holds_no_secret (verifies: LLR-gh4rw3) — "WriteOutcome: not allowlisted" → pass; mutation check: a HandleRng field planted in AcknowledgementReport made it red, reverted → green
  red -> green: org-io/tests/absences.rs::org_io_removes_nothing_itself_but_in_the_persona_deletion (verifies: LLR-vvd8dn; rewrite) — green on arrival; mutation checks: a planted public remove_file in lib.rs, and a remove_store call in verify.rs, each made it red, reverted → green
  red -> green: org-io/tests/absences.rs::no_public_function_writes_renames_or_removes_a_file (verifies: LLR-vvd8dn; rewrite) — green on arrival; same lib.rs plant → red, reverted → green
  red -> green: org-io/tests/absences.rs::no_public_method_discards_a_provisional_change_or_organisation (verifies: LLR-vvd8dn; rewrite) — green on arrival; same plant → red, reverted → green
  red -> green: org-io/tests/signatory_rule.rs::is_own_admin_makes_one_pinned_signatory_set_read (verifies: LLR-7qdjqn) — green on arrival; mutation check: a second signatory_set read made it red (2 vs 1), reverted → green
  inherited: a_node_holding_no_signatory_key_is_refused_with_not_configured — from LLR-c9fyun
  inherited: a_failed_signatory_read_is_an_error_never_not_admin — from LLR-c9fyun
  inherited: a_configured_account_listed_as_delegate_is_admin_and_delegates_are_read_only_for_a_mapped_h160 — from LLR-c9fyun
```

#### T46 done

```
  red -> green: app/src-tauri/tests/ipc.rs::the_app_manifest_names_no_rand_rand_core_or_getrandom (verifies: LLR-vj5s28) — compile red; mutation check: rand re-added to Cargo.toml → red, restored → green
  red -> green: app/src-tauri/tests/ipc.rs::no_app_source_names_a_generator_or_passes_an_rng (verifies: LLR-vj5s28) — compile red; mutation: an OsRng literal in parsing.rs → red, restored
  red -> green: app/src-tauri/tests/ipc.rs::propose_person_returns_the_device_key_names_and_provisional (verifies: LLR-k57uqv) — compile red; mutation: status "Provisional" → red, restored
  red -> green: app/src-tauri/tests/ipc.rs::propose_person_stores_the_parsed_details (verifies: LLR-k57uqv) — compile red; mutation: name taken from the surname → red, restored
  red -> green: app/src-tauri/tests/ipc.rs::propose_person_refuses_an_invalid_field_naming_it_and_creates_nothing (verifies: LLR-k57uqv) — compile red; mutation: prefix not stripped → red, restored
  red -> green: app/src-tauri/tests/ipc.rs::propose_founding_organisation_returns_the_new_personas_id_and_leaves_none_on_refusal (verifies: LLR-6ff735) — real red on first run (member.handle prefix), fixed; mutation: founder = root hex → red, restored
  red -> green: app/src-tauri/tests/ipc.rs::issue_invite_returns_org_ios_blob_unchanged_and_takes_no_name_or_address (verifies: LLR-y3zykm) — compile red; mutation: invitee name replaced → red, restored
  red -> green: app/src-tauri/tests/ipc.rs::read_invite_returns_the_public_fields_and_stores_nothing (verifies: LLR-b7ywy4) — compile red; mutation: issuer_device_key = org_id hex → red, restored
  red -> green: app/src-tauri/tests/ipc.rs::list_personas_reports_exactly_the_persona_fields (verifies: LLR-pguhw5; body rewritten) — compile red; mutation: persona_id renamed → red, restored
  red -> green: app/src-tauri/tests/ipc.rs::a_persona_in_no_organisation_reports_a_null_org_id (verifies: LLR-pguhw5; body rewritten) — compile red; mutation: org_id Some("") → red, restored
  red -> green: app/src-tauri/tests/receiver_guard.rs::the_receiver_holds_no_app_lock_while_waiting (verifies: LLR-5wb4vd; new body) — compile red; mutation: list_personas waits on the receiver → Timeout, restored
  red -> green: app/src-tauri/tests/receiver_guard.rs::a_refused_and_a_mutating_command_complete_while_the_receiver_waits (verifies: LLR-5wb4vd; new body) — compile red; mutation: propose_founding_organisation waits → Timeout, restored
  red -> green: app/src-tauri/tests/startup_policy.rs::a_keychain_refusal_fails_startup_naming_the_condition (verifies: LLR-tpkmh3, REQ-7g3k9a; new body) — compile red; mutation: open error replaced → red, restored
  red -> green: app/src-tauri/tests/receiver_events.rs::the_store_and_invitation_refusals_are_classified_as_receiver_errors (verifies: LLR-7bk6qh; narrowed) — compile red; mutation: InviteNotOutstanding to the verdict arm → red, restored
  red -> green: app/src-tauri/tests/receiver_events.rs::locally_reachable_variants_are_classified_as_receiver_errors (verifies: LLR-7bk6qh; narrowed) — compile red; mutation: MalformedMessage to the verdict arm → red, restored
  red -> green: app/src-tauri/tests/receiver_events.rs::a_non_terminal_failure_does_not_stop_the_loop (verifies: LLR-zw6jqy, LLR-a9rjtf) — compile red; mutation: Connection terminal → red, restored
  red -> green: app/src-tauri/tests/receiver_events.rs::terminal_is_decided_by_the_error_variant_not_its_text (verifies: LLR-zw6jqy) — compile red; same mutation → red, restored
  red -> green: app/src-tauri/tests/receiver_events.rs::store_failures_are_one_receiver_error_and_do_not_stop_the_loop (verifies: LLR-zw6jqy; renamed) — compile red; mutation: Store terminal → red, restored
  inherited: none. Deleted as planned (out-of-force or cut for later tasks); additionally `receiver_events.rs::a_revocation_accepted_by_its_notice_is_announced_as_self_deleted` (retired behaviour; LLR-2vg79y stays verified by the self_delete_* tests).
```

#### T47 done

```
  red -> green: app/src-tauri/tests/startup_policy.rs::an_absent_or_empty_interval_is_ten_minutes (verifies: LLR-2dpn52) — E0425 no resolve_verification_interval / DEFAULT_VERIFICATION_INTERVAL → pass
  red -> green: app/src-tauri/tests/startup_policy.rs::a_decimal_from_one_to_u32_max_is_that_many_seconds (verifies: LLR-2dpn52) — E0425 → pass
  red -> green: app/src-tauri/tests/startup_policy.rs::zero_a_sign_a_space_a_suffix_a_fraction_or_over_u32_max_is_refused_naming_the_variable (verifies: LLR-rvrc25) — E0425; E0599 no StartupError::InvalidVerificationInterval → pass
  inherited: none
```

#### io-followup done

```
  red -> green: org-io/tests/acknowledgements.rs::a_failed_deletion_at_open_is_retried_hourly_once_receiving_starts (verifies: LLR-3fxxpp, LLR-upqj5g; PR-yng83g) — Elapsed, nothing retried during the run → pass
```

#### T48 done

```
  red -> green: app/src-tauri/tests/ipc.rs::propose_member_hands_the_handles_in_order_and_sends_nothing (verifies: LLR-rg9vc7) — compile red (no propose_member / request_admission commands, no AdmissionDto) → pass
  red -> green: app/src-tauri/tests/ipc.rs::request_admission_unconfirmed_is_refused_and_asks_org_io_nothing (verifies: LLR-q2k9z8) — compile red → pass
  red -> green: app/src-tauri/tests/ipc.rs::request_admission_for_an_unknown_persona_is_refused (verifies: LLR-q2k9z8) — compile red → pass
  red -> green: app/src-tauri/tests/ipc.rs::request_admission_refuses_without_confirmation_over_ipc (verifies: LLR-q2k9z8; rewrite of the cut REQ-ab2mfz test) — compile red → pass
  red -> green: app/src-tauri/tests/ipc.rs::request_admission_tags_each_outcome_and_never_reports_an_used_invite_as_refused (verifies: LLR-8cz828) — compile red → pass
  inherited: none
```

#### T49 done

```
  red -> green: app/src-tauri/tests/ipc.rs::propose_admission_unconfirmed_is_refused_and_records_nothing (verifies: LLR-cr9nmw) — compile red (no propose_admission) → pass
  red -> green: app/src-tauri/tests/ipc.rs::a_blank_handle_is_handed_on_as_none_and_another_unchanged (verifies: LLR-gmb4p8) — compile red (no chosen_handle) → pass; mutation check after the scenario fix: the chosen Handle replaced by None made it red, reverted → green
  red -> green: app/src-tauri/tests/ipc.rs::propose_admission_takes_no_organisation_secret_reply_or_peer_address (verifies: LLR-gmb4p8; rewrite) — compile red → pass
  red -> green: app/tests/api.admit.test.ts::invokes propose_admission with the org id, the invite id, the handle and the confirmation alone (verifies: LLR-gmb4p8; rewrite) — proposeAdmission is not a function → pass
  red -> green: app/src-tauri/tests/ipc.rs::propose_revocation_parses_the_member_id_and_takes_no_peer_address (verifies: LLR-7n4hgr; rewrite) — compile red → pass
  red -> green: app/src-tauri/tests/ipc.rs::propose_revocation_rejects_a_short_member_id (verifies: LLR-7n4hgr; rewrite) — compile red → pass
  red -> green: app/src-tauri/tests/ipc.rs::propose_revocation_rejects_a_doubled_zero_x_prefix_on_the_member_id (verifies: LLR-7n4hgr; rewrite) — compile red → pass
  red -> green: app/src-tauri/tests/ipc.rs::propose_revocation_accepts_a_zero_x_prefixed_member_id (verifies: LLR-7n4hgr) — compile red → pass
  red -> green: app/src-tauri/tests/ipc.rs::cancel_invite_calls_org_io_once_and_returns_its_refusal_as_text (verifies: LLR-vqz4s6) — compile red (no cancel_invite, no parse_invite_id) → pass
  red -> green: app/src-tauri/tests/ipc.rs::an_invite_id_that_is_not_hex_or_not_32_bytes_is_refused_before_org_io (verifies: LLR-r8jeu8) — compile red → pass
  inherited: none
```

#### T56 done

```
  red -> green: app/tests/verification.view.test.ts::verificationLine renders every result and never a removed or behind one as in step (verifies: LLR-mfzr6z) — verificationLine missing → pass
  red -> green: app/tests/verification.view.test.ts::follows a settled result with its write and names a founding with no proxy by its founder (verifies: LLR-mfzr6z) — verificationLine missing → pass
  red -> green: app/tests/verification.view.test.ts::verificationLabel renders each status (verifies: LLR-njb6mr) — 'not checked' vs 'in step' → pass
  red -> green: app/tests/verification.view.test.ts::verifyOrganisations invokes verify_organisation with no argument (verifies: LLR-j2x9jb) — not a function → pass
  red -> green: app/tests/verification.view.test.ts::asks for a verification once when it mounts (verifies: LLR-j2x9jb) — no call in onMount → pass
  red -> green: app/tests/verification.view.test.ts::calls verify_organisation when its Verify with chain button is pressed (verifies: LLR-j2x9jb) — no button → pass
  red -> green: app/tests/verification.view.test.ts::no source defines or invokes a request action (verifies: LLR-zkpsw5) — verificationLine missing → pass
  red -> green: app/tests/verification.view.test.ts::a verification-result event is filed newest first through verificationLine (verifies: LLR-84cxpt) — fileVerificationResults missing → pass
  red -> green: app/tests/verification.view.test.ts::offers a founding for writing by the founder key its proposal or its verification entry carries (verifies: LLR-rhc555) — foundingWrites missing → pass
  red -> green: app/tests/verification.view.test.ts::writes a founding with writeOrganisationToChain from its Write to chain button (verifies: LLR-rhc555) — writeOrganisationToChain missing → pass
  red -> green: app/tests/founding.propose.test.ts::proposes the Person and then the founding, and returns the founder (verifies: LLR-6ff735, LLR-rhc555) — module missing → pass
  red -> green: app/tests/founding.propose.test.ts::a Person held already is not proposed again and the founding is still proposed (verifies: LLR-6ff735, LLR-rhc555) — module missing → pass; mutation check: dropping the PERSON_ALREADY_HELD pass-over made it red, reverted → green
  red -> green: app/tests/founding.propose.test.ts::any other refusal of the Person stops before the founding, naming the field (verifies: LLR-6ff735) — module missing → pass
  red -> green: app/tests/founding.propose.test.ts::a refused founding is reported as org-io words it (verifies: LLR-6ff735) — module missing → pass
  red -> green: app/tests/founding.propose.test.ts::the creation form shows the founder and hands it to the founding-writes list (verifies: LLR-rhc555) — module missing → pass; mutation check: removing the onProposed call in CreateOrg.svelte made it red, reverted → green
  inherited: names the Organisation and the result, with the epoch where there is one — from LLR-hwcj9n (now LLR-mfzr6z)
  inherited: shows the reason for a refused or unverified result and never calls it in step — from LLR-hwcj9n (now LLR-mfzr6z)
  inherited: renders a result it does not know as unverified rather than in step — from LLR-hwcj9n (now LLR-mfzr6z)
  inherited: renders each status, an unverified one with its reason — from LLR-hwcj9n (now LLR-njb6mr; mutation check: 'unchecked' for not_checked made it red, reverted → green)
```

#### T50 done

```
  red -> green: app/src-tauri/tests/ipc.rs::propose_updated_organisation_returns_epoch_root_and_key_as_hex_and_writes_no_chain (verifies: LLR-2u3cga) — compile red (no command) → pass
  red -> green: app/src-tauri/tests/ipc.rs::write_organisation_to_chain_takes_exactly_one_of_org_id_or_founder (verifies: LLR-rhc555) — compile red → pass
  red -> green: app/src-tauri/tests/verification.rs::write_organisation_to_chain_tags_included_refused_and_unknown (verifies: LLR-rhc555) — compile red (no write_then_verify) → pass
  red -> green: app/src-tauri/tests/verification.rs::an_included_write_is_verified_once_and_a_refused_or_unknown_one_is_not (verifies: LLR-k4rhbs) — compile red → pass
  red -> green: app/src-tauri/tests/verification.rs::an_included_founding_write_is_verified_by_its_founder_and_an_update_by_its_org_id (verifies: LLR-k4rhbs) — compile red → pass
  inherited: none
```

#### T59 done

```
  red -> green: app/tests/invite.confirm.test.ts::states each thing the user must be told before the first admission request (verifies: LLR-qcg6bz) — ADMISSION_REQUEST_WARNING missing → pass
  red -> green: app/tests/invite.confirm.test.ts::refuses until the user has confirmed (verifies: LLR-qcg6bz) — admissionRequestGate missing → pass
  red -> green: app/tests/invite.confirm.test.ts::allows the request once the user confirmed (verifies: LLR-qcg6bz) — admissionRequestGate missing → pass
  red -> green: app/tests/invite.confirm.test.ts::a retry of a confirmed request asks nothing again (verifies: LLR-prh433) — admissionRequestGate missing → pass
  red -> green: app/tests/revoke.validate.test.ts::trims surrounding whitespace before measuring (verifies: LLR-csbs5v; LLR-c9r5uf part dropped) — green on arrival; mutation check: removing .trim() made it red, reverted → green
  red -> green: app/tests/revoke.validate.test.ts::takes the member identifier alone (verifies: LLR-csbs5v) — green on arrival; mutation check: a second parameter made it red, reverted → green
  inherited: refuses until the user has confirmed — from LLR-n2u4uf
```

#### T57 done

```
  red -> green: app/tests/notices.admission.test.ts::admissionConfirmation shows the invitee name beside the request and marks a mismatch (verifies: LLR-47zvjn) — stub threw → pass
  red -> green: app/tests/notices.admission.test.ts::admissionGate refuses until confirmed and proposeAdmission is invoked with confirmed true (verifies: LLR-e63yrs) — stub threw; then old Admit.svelte had no doAdmit → pass
  red -> green: app/tests/notices.admission.test.ts::the page lists kept requests once after it listens (verifies: LLR-5mw2dj) — no listAdmissionRequests call in onMount → pass
  red -> green: app/tests/notices.admission.test.ts::a request listed and announced is shown once per org id and invite id (verifies: LLR-dhtec7) — stub threw → pass
  red -> green: app/tests/notices.admission.test.ts::an invite used by another device is an error message above the form (verifies: LLR-d8998k) — stub threw; then Invite.svelte had no requestMessage block → pass
  inherited: none
```

#### T51 done

```
  red -> green: app/src-tauri/tests/verification.rs::an_own_adoption_is_sent_once_and_nothing_else_is (verifies: LLR-es49wp) — compile red (no send_and_request) → pass
  red -> green: app/src-tauri/tests/verification.rs::a_behind_result_is_requested_once_with_no_user_step (verifies: LLR-nkg8sb) — no requests on Behind → pass
  red -> green: app/src-tauri/tests/verification.rs::sends_reports_each_device_sent_or_failed_beside_the_others (verifies: LLR-rywe8f) — compile red (no device_reports, no sends field) → pass
  red -> green: app/src-tauri/tests/verification.rs::a_founding_with_no_proxy_is_entered_with_a_null_org_id_and_its_founder (verifies: LLR-jf63e4) — compile red (no verify_all_and_send) → pass
  red -> green: app/src-tauri/tests/verification.rs::a_founding_with_its_proxy_recorded_is_entered_with_its_org_id_and_still_its_founder (verifies: LLR-jf63e4) — compile red → pass
  red -> green: app/src-tauri/tests/verification.rs::verify_all_reports_in_step_and_lists_the_organisation_verified (verifies: LLR-jf63e4, LLR-g8wfy9) — compile red → pass
  red -> green: app/src-tauri/tests/verification.rs::verify_all_over_an_unreadable_chain_reports_unverified_with_the_reason (verifies: LLR-jf63e4, LLR-g8wfy9) — compile red → pass
  red -> green: app/src-tauri/tests/verification.rs::list_orgs_reports_not_checked_removed_settled_and_refused_statuses (verifies: LLR-g8wfy9) — compile red (OrgDto::new arity) → pass
  red -> green: app/src-tauri/tests/ipc.rs::list_orgs_carries_each_verification_status_or_not_checked (verifies: LLR-g8wfy9) — compile red (no verify_organisation) → pass
  red -> green: app/src-tauri/tests/ipc.rs::verify_organisation_with_the_chain_not_configured_reports_every_organisation_unverified (verifies: LLR-jf63e4, LLR-g8wfy9) — compile red → pass
  red -> green: app/src-tauri/tests/ipc.rs::verify_organisation_on_an_empty_store_reports_nothing (verifies: LLR-jf63e4) — compile red → pass
  red -> green: app/src-tauri/tests/verification.rs::a_settled_verification_is_its_outcomes_entry_with_the_write_label (verifies: LLR-jf63e4) — mapping from T50; mutation check: dropping the write label made it red, reverted → green
  red -> green: app/src-tauri/tests/verification.rs::discarded_lists_each_change_tagged_by_kind_and_is_empty_not_absent (verifies: LLR-n64wvr) — mutation check: admission name forced to None → red, reverted → green
  red -> green: app/src-tauri/tests/verification.rs::each_verification_crosses_ipc_tagged_with_its_fields (verifies: LLR-jf63e4) — mutation check: Removed mapped to Unverified → red, reverted → green
  red -> green: app/src-tauri/tests/verification.rs::verify_organisation_returns_one_tagged_entry_per_result_store_keychain_and_refused_as_unverified (verifies: LLR-jf63e4) — mutation check: Keychain reason via Debug → red, reverted → green
  red -> green: app/src-tauri/tests/ipc.rs::list_orgs_reports_not_checked_before_any_reconcile (verifies: LLR-g8wfy9; re-created) — mutation check: disabling NotYetVerified → NotChecked → red, reverted → green
  inherited: list_orgs_reports_not_checked_before_any_reconcile — from LLR-8axx7h
```

#### T58 done

```
  red -> green: app/tests/notices.discarded.test.ts::applyDiscarded adds one notice per change and keeps it until dismissed (verifies: LLR-k5rf3r) — applyDiscarded missing → pass
  red -> green: app/tests/notices.discarded.test.ts::reproposal names propose_admission or propose_revocation and the notice stays (verifies: LLR-md6hhz) — applyDiscarded missing → pass
  red -> green: app/tests/notices.discarded.test.ts::reproposalStep asks for the confirmation beside the kept request before an admission is proposed again (verifies: LLR-md6hhz, LLR-e63yrs) — applyDiscarded missing → pass
  red -> green: app/tests/notices.discarded.test.ts::the panel proposes an admission again only on the confirmation, and a cancel proposes nothing (verifies: LLR-md6hhz, LLR-e63yrs) — no proposeAgain in Membership → pass
  red -> green: app/tests/notices.discarded.test.ts::sendOutcomes returns every device's outcome and offers no action (verifies: LLR-fsk4g2) — sendOutcomes missing → pass
  red -> green: app/tests/notices.discarded.test.ts::a failed listing of the kept admission requests shows under its own label, not the verification one (verifies: LLR-5mw2dj; added) — the catch assigned startupError → pass
  inherited: none
```

#### T52 done

```
  red -> green: app/src-tauri/tests/ipc.rs::list_admission_requests_returns_each_kept_request_with_the_events_fields_and_changes_nothing (verifies: LLR-y7nc6x) — "Command list_admission_requests not found" → pass
  red -> green: app/src-tauri/tests/ipc.rs::the_harness_registers_exactly_the_seventeen_commands_and_refuses_the_old_names (verifies: LLR-kk8gc7) — harness list lacked list_admission_requests → pass
  red -> green: app/src-tauri/tests/ipc.rs::every_org_id_taking_command_parses_it_first_with_the_parsers_message (verifies: LLR-97hujy) — green on arrival; mutation checks: parsing a corrupted org_id, and parsing invite_id before org_id, each made it red, reverted → green
  red -> green: app/src-tauri/tests/ipc.rs::propose_revocation_refuses_a_malformed_org_id_with_the_parsers_message (verifies: LLR-97hujy) — green on arrival; corrupted-parse mutation → red, reverted
  red -> green: app/src-tauri/tests/ipc.rs::propose_revocation_does_not_refuse_a_well_formed_org_id (verifies: LLR-97hujy) — green on arrival; same mutation → red, reverted
  red -> green: app/src-tauri/tests/ipc.rs::issue_invite_rejects_a_short_org_id (verifies: LLR-97hujy) — green on arrival; same mutation → red, reverted
  red -> green: app/src-tauri/tests/ipc.rs::issue_invite_rejects_a_non_hex_org_id (verifies: LLR-97hujy) — green on arrival; same mutation → red, reverted
  red -> green: app/src-tauri/tests/ipc.rs::issue_invite_does_not_refuse_a_well_formed_org_id (verifies: LLR-97hujy) — green on arrival; same mutation → red, reverted
  red -> green: app/src-tauri/tests/ipc.rs::propose_admission_refuses_a_malformed_org_id_with_the_parsers_message (verifies: LLR-97hujy) — green on arrival; same mutation → red, reverted
  red -> green: app/src-tauri/tests/ipc.rs::propose_admission_does_not_refuse_a_well_formed_org_id (verifies: LLR-97hujy) — green on arrival; same mutation → red, reverted
  inherited: product_registers_the_same_commands_as_the_harness — from LLR-4wcyqy (re-pointed to LLR-kk8gc7, count 16 → 17; it also went red first, 16 != 17)
  inherited: list_personas_returns_an_empty_list_on_a_fresh_store — from LLR-4wcyqy (re-pointed to LLR-kk8gc7)
  inherited: connection_status_is_registered_under_its_name — from LLR-4wcyqy (now LLR-8tzbzn, LLR-kk8gc7)
  inherited: unknown_command_is_rejected — from LLR-4wcyqy (now LLR-8tzbzn, LLR-kk8gc7)
```

#### T53 done

```
  red -> green: app/src-tauri/tests/receiver_events.rs::a_received_provisional_organisation_is_provisional_received_never_updated (verifies: LLR-4bvdn7) — compile red (no Arrival::ProvisionalOrganisation, no ReceiverOutcome::ProvisionalReceived) → pass
  red -> green: app/src-tauri/tests/receiver_events.rs::removed_is_self_deleted_then_the_result_and_a_kept_notice_announces_nothing (verifies: LLR-ygr6gd) — compile red (no outcomes_for_verification, no RevocationNoticeKept) → pass
  red -> green: app/src-tauri/tests/receiver_events.rs::an_admission_request_kept_is_one_admission_requested_event_with_its_fields (verifies: LLR-dp45mm) — compile red (no events::AdmissionRequested) → pass
  red -> green: app/src-tauri/tests/receiver_events.rs::an_answered_organisation_request_is_announced_by_no_event (verifies: LLR-t9rdf8) — compile red → pass
  red -> green: app/src-tauri/tests/receiver_events.rs::the_outcome_kind_comes_from_the_received_kind (verifies: LLR-4bvdn7, LLR-ygr6gd, LLR-dp45mm, LLR-t9rdf8) — compile red → pass
  red -> green: app/src-tauri/tests/receiver_events.rs::every_org_node_error_variant_is_classified_with_no_wildcard (verifies: LLR-wgr46u) — green on arrival; mutation checks: a wildcard arm, and OrganisationRemoved moved to the verdict group, each made it red, reverted → green
  inherited: every_verification_verdict_is_classified_as_a_verification_failure — from LLR-7bk6qh
  inherited: a_verification_verdict_carries_its_own_message_and_no_invented_organisation — from LLR-7bk6qh
  inherited: a_chain_failure_is_classified_as_a_receiver_error — from LLR-7bk6qh
  inherited: the_two_classes_are_actually_distinguished — from LLR-7bk6qh
  inherited: the_commit_workflow_refusals_are_classified_as_receiver_errors — from LLR-7bk6qh
  inherited: the_sender_refusals_are_classified_as_receiver_errors — from LLR-7bk6qh
  inherited: the_store_and_invitation_refusals_are_classified_as_receiver_errors — from LLR-7bk6qh
  inherited: locally_reachable_variants_are_classified_as_receiver_errors — from LLR-7bk6qh
  inherited: a_wire_message_over_the_bound_is_classified_as_a_receiver_error — from LLR-7bk6qh
  inherited: a_refused_key_or_field_is_classified_as_a_receiver_error — from LLR-7bk6qh
```

#### T54 done

```
  red -> green: app/src-tauri/tests/receiver_events.rs::a_provisional_received_or_kept_notice_asks_one_verification_before_the_next_message (verifies: LLR-w7m3d4) — compile red (no receive_iteration / follow_up_for / FollowUp) → pass
  red -> green: app/src-tauri/tests/receiver_events.rs::adopted_maps_to_updated_or_record_unreadable_then_the_result (verifies: LLR-acb5g4) — runtime red: only VerificationResult, no Updated → pass
  red -> green: app/src-tauri/tests/receiver_events.rs::a_verification_result_is_one_event_carrying_the_entry (verifies: LLR-8grnbb) — runtime red: no event emitted → pass
  inherited: none
```

#### T55 done

```
  red -> green: app/src-tauri/tests/periodic.rs::no_run_before_one_interval_then_one_per_interval_never_two_at_once (verifies: LLR-3ampqm) — compile red (no periodic module) → pass; mutation check: first tick at start → red, reverted → green
  red -> green: app/src-tauri/tests/periodic.rs::a_late_run_delays_the_next_rather_than_bursting (verifies: LLR-3ampqm) — compile red → pass; mutation check: MissedTickBehavior::Burst → red, reverted → green
  red -> green: app/src-tauri/tests/periodic.rs::each_result_is_announced_as_the_receiver_loop_announces_it (verifies: LLR-2267hj) — compile red (round 1); the plan-form rewrite passed on arrival; mutation checks: skipping VerificationResult outcomes, and verification_outcomes with Some(target), each made it red, reverted → green
  red -> green: app/src-tauri/tests/periodic.rs::a_behind_organisation_is_requested_at_each_run_until_in_step_or_adopted (verifies: LLR-fe7umz) — compile red (round 1), then compile red against the old periodic_run type (round 2) → pass; mutation check: stripping `requests` → red, reverted → green
  inherited: none
```

#### T60 done

```
  red -> green: org-node/tests/admission_sender.rs::the_first_adoption_marks_its_persona_active_and_changes_no_other_store (verifies: LLR-exaa9u, LLR-f9x3vf) — compile red E0583 (support/sweep.rs), then Proposed where Active was expected → pass after the verify_org.rs fix
  red -> green: org-node/tests/admission_sender.rs::a_first_admission_records_the_chains_key_the_private_key_and_the_member (verifies: LLR-exaa9u, LLR-rys5nx, LLR-qw8tyu) — Proposed vs Active → pass after the fix; mutation: the first admission takes a constant private key → red, reverted
  red -> green: org-node/tests/verification.rs::a_founding_is_adopted_once_the_chain_carries_its_root (verifies: REQ-tqap3r, LLR-gcm7bq, LLR-exaa9u, LLR-qw8tyu, LLR-x5asbx, LLR-g2jeqd) — Proposed vs Active → pass after the fix; mutation: adopt_founding sets proxy_account None → red, reverted
  red -> green: org-node/tests/service_stories.rs::revocation_from_an_unknown_device_leaves_the_record_in_place (verifies: LLR-283x6f, LLR-rhsfw9) — Proposed vs Active → pass after the fix; mutation: `listed || true` → red, reverted
  red -> green: org-node/tests/admission_sender.rs::the_proxy_account_recorded_for_a_founding_is_kept_and_handed_back (verifies: LLR-x5asbx) — green on arrival; mutation: adopt_founding sets proxy_account None → red, reverted
  red -> green: org-node/tests/admission_sender.rs::member_ids_are_not_derived_from_keys (verifies: REQ-d9g6nt, LLR-4rv4kr, LLR-rjg3m2) — green on arrival; mutation: MemberId derived from the member key → red, reverted
  red -> green: org-node/tests/admission_sender.rs::readmission_with_same_keys_gets_a_fresh_member_id (verifies: REQ-d9g6nt, LLR-4rv4kr) — green on arrival; same mutation → red, reverted
  red -> green: org-node/tests/admission_sender.rs::a_revocation_about_an_organisation_not_held_is_refused_before_the_chain (verifies: REQ-d9g6nt, REQ-vxqc5g, LLR-j6j95z, LLR-95av3e) — green on arrival; mutation: check_without_record decodes the whole body before refusing → red, reverted
  red -> green: org-node/tests/admission_sender.rs::a_revocation_reaches_the_administrators_disk (verifies: LLR-34c5y3, LLR-degg8t) — green on arrival; mutation: messages_for_current_epoch drops the revocations → red, reverted
  red -> green: org-node/tests/admission_sender.rs::a_failed_revocation_send_leaves_the_committed_removal_in_place (verifies: LLR-34c5y3, LLR-g2jeqd) — green on arrival; same mutation → red, reverted
  red -> green: org-node/tests/admission_sender.rs::pr_szkat6_an_admitted_member_holds_the_organisation_private_key (verifies: LLR-qw8tyu, REQ-szq3ud, REQ-ju6vn2, PR-szkat6) — green on arrival; mutation: the first admission takes a constant private key → red, reverted
  red -> green: org-node/tests/admission_sender.rs::pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths (verifies: LLR-8528ud, LLR-qw8tyu, PR-ve9zw8) — green on arrival; mutations: a later adoption keeps the previous private key, and `listed || true`, each → red, reverted
  red -> green: org-node/tests/admission_sender.rs::a_committed_update_gives_the_record_the_new_key_pair_on_both_paths (verifies: LLR-94rzrk, REQ-ju6vn2) — green on arrival; mutation: a later adoption keeps the previous private key → red, reverted
  red -> green: org-node/tests/admission_sender.rs::pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused_on_the_self_delete_path (verifies: LLR-283x6f, PR-u4c2vp) — green on arrival; mutation: `listed || true` → red, reverted
  red -> green: org-node/tests/absences.rs::org_node_names_no_chain_library_and_only_verify_judges_against_the_chain (verifies: LLR-7en38k) — green on arrival; mutation: `fn commit_update fn build_reply` planted in src/lib.rs → red, reverted
  red -> green: org-node/tests/absences.rs::org_node_holds_no_invitation_exchange (verifies: LLR-zj88e6, LLR-ds228z, LLR-96z2k5, LLR-8qxwst, REQ-qn2erx) — ported; green on arrival; same plant → red, reverted
  red -> green: org-node/tests/receive_chain_reads.rs::a_revocation_in_a_store_holding_no_record_is_refused_as_org_not_held (verifies: LLR-95av3e) — green on arrival; mutation: check_without_record decodes the whole body first → red, reverted
  red -> green: org-node/tests/receive_chain_reads.rs::a_revocation_about_another_organisation_is_refused_as_revocation_not_held_and_writes_nothing (verifies: LLR-95av3e) — green on arrival; mutation: check_notice's other-organisation refusal becomes RevocationNotForThisDevice → red, reverted
  red -> green: org-node/tests/receive_chain_reads.rs::after_a_refused_revocation_the_expected_admission_is_still_in_place (verifies: LLR-95av3e, LLR-q8emds) — green on arrival; mutation: the first-admission adoption no longer clears the expectation (`retain ==`) → red, reverted
  red -> green: org-node/tests/organisation_key.rs::a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals (verifies: REQ-ech45n, LLR-9qavm7, LLR-qw8tyu) — green on arrival; mutation: adopt_founding takes a constant private key → red, reverted
  red -> green: org-node/tests/organisation_key.rs::the_organisation_private_key_is_kept_only_in_the_encrypted_store (verifies: REQ-ech45n, LLR-qw8tyu) — green on arrival; mutation: seal_bytes appends the plaintext → red, reverted
  red -> green: org-node/tests/service_lifecycle.rs::a_new_persona_is_proposed_with_an_id_derived_from_its_member_key (verifies: REQ-hzm4kt, LLR-d4gpxm, LLR-6acxu9, LLR-s7yu4k, LLR-t2adzg) — green on arrival; mutation: propose_member records org_id None → red, reverted
  red -> green: org-node/tests/service_lifecycle.rs::only_the_first_adoption_sets_a_personas_member_id (verifies: LLR-t2adzg) — green on arrival; same mutation → red, reverted
  red -> green: org-node/tests/verification.rs::an_own_provisional_organisation_the_chain_carries_is_adopted (verifies: REQ-tqap3r, REQ-jy6ybw, LLR-cja9zv, LLR-g2jeqd) — green on arrival; mutation: a later adoption keeps the previous private key → red, reverted
  red -> green: org-node/tests/verification.rs::the_record_takes_a_recorded_revocation_only_through_adoption (verifies: REQ-tqap3r, LLR-34c5y3, LLR-x5asbx, LLR-g2jeqd) — green on arrival; mutation: messages_for_current_epoch drops the revocations → red, reverted
  red -> green: org-node/tests/secret_redaction.rs::organisation_key_custody::a_provisional_private_key_leaves_the_store_only_inside_a_message_to_a_listed_device (verifies: LLR-qw8tyu) — E1 rewrite; green on arrival; mutation: Organisation information also sent to the removed Device → red (the old fixture would have missed it), reverted
  red -> green: org-io/tests/persona_creation.rs::organisations_report_their_stored_name_and_no_send_carries_it (verifies: LLR-pn3hgc) — E7 rewrite; the multi-line signature made the old scan red (`takers: []`) → pass with public_signatures
  red -> green: org-io/tests/chain_write.rs::each_write_refusal_displays_human_readable_text (verifies: LLR-48me7m, LLR-mhn4n5, LLR-5tpdxw) — E5; E0599 WriteRefused does not implement Display → pass
  red -> green: app/src-tauri/tests/verification.rs::a_refusal_before_the_chain_is_org_ios_display_text (verifies: LLR-rhc555) — E5; `left: "NotHeld"` against the Display text → pass after error.to_string()
  red -> green: org-io/tests/acknowledgements.rs::a_removed_file_beside_a_live_store_of_its_key_is_kept_and_reported (verifies: LLR-upqj5g, LLR-3fxxpp) — PR-wch3zu; open failed KeychainItemMissing (the live item deleted) → pass
  red -> green: org-io/tests/verification.rs::the_time_given_is_one_clock_reading_after_the_read (verifies: LLR-fzf2n3) — flake fix; mutation: a second clock.now() in verify.rs gave `left: 2` → red, reverted
  inherited: the_service_presents_no_chain_write — from LLR-65py3d, LLR-3v5nu9 (REQ-xs4ab8 dropped)
  inherited: org_node_is_sans_io — from LLR-4tcxsu
  inherited: member_ids_are_not_derived_from_keys — from LLR-ag9mgm
  inherited: readmission_with_same_keys_gets_a_fresh_member_id — from LLR-ag9mgm
  inherited: org_node_holds_no_invitation_exchange — from LLR-836z24, LLR-qezw3n
  inherited: a_revocation_about_an_organisation_not_held_is_refused_before_the_chain — from LLR-38e2kn
  inherited: a_revocation_reaches_the_administrators_disk — from LLR-qg9utu, LLR-8hdu9x
  inherited: a_failed_revocation_send_leaves_the_committed_removal_in_place — from LLR-qg9utu, LLR-t4znbk
  inherited: a_first_admission_records_the_chains_key_the_private_key_and_the_member — from LLR-3fwykc, LLR-e5c9ud
  inherited: pr_szkat6_an_admitted_member_holds_the_organisation_private_key — from LLR-3fwykc
  inherited: pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths — from LLR-37cj3n, LLR-3fwykc
  inherited: a_committed_update_gives_the_record_the_new_key_pair_on_both_paths — from LLR-xn5pwc
  inherited: pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused_on_the_self_delete_path — from LLR-3q63zv
  inherited: revocation_from_an_unknown_device_leaves_the_record_in_place — from LLR-3q63zv, LLR-tx8ruv
  inherited: a_new_persona_is_proposed_with_an_id_derived_from_its_member_key — from LLR-tev8h8, LLR-v82xds
  inherited: a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals — from LLR-3fwykc, LLR-sj7cd5
  inherited: the_organisation_private_key_is_kept_only_in_the_encrypted_store — from LLR-3fwykc, LLR-qjz3q4
  inherited: a_founding_is_adopted_once_the_chain_carries_its_root — from LLR-3fwykc, LLR-w3fhhg, LLR-q3aj8z, LLR-dzte8x, LLR-3v5nu9, LLR-4tcxsu
  inherited: an_own_provisional_organisation_the_chain_carries_is_adopted — from LLR-4tcxsu
  inherited: the_record_takes_a_recorded_revocation_only_through_adoption — from LLR-qg9utu, LLR-drgdy8, LLR-3v5nu9, LLR-t4znbk
```

#### T61 done

```
  red -> green: none (T61 writes no tests)
```

#### Guideline reviews — round 1 (2026-10-25) — Round 1 fix — CODE app + on-chain-client + Makefile (task `cfa`, merged 2026-10-25)

```
  red -> green: app/tests/notices.admission.test.ts::a refused subscription at start is shown on the page, not left unhandled (verifies: LLR-5mw2dj) — no `.catch` ended the subscription chain → pass after the +page.svelte fix
```

#### Guideline reviews — round 1 (2026-10-25) — Coverage task `cov` (merged 2026-10-25)

```
  red -> green: org-io/tests/acknowledgements.rs::open_deletes_a_removal_that_keeps_no_acknowledgement_before_its_deadline (verifies: LLR-upqj5g) — green on arrival; mutation: dropped the `is_empty() ||` operand in delete_expired_at_open → red, reverted
  red -> green: org-io/tests/acknowledgements.rs::a_failed_write_of_an_acceptance_keeps_the_acknowledgement_for_the_next_round (verifies: LLR-6ux6yw) — green on arrival; mutation: `accepted(..).await.unwrap_or(0)` in round → red, reverted
  red -> green: org-io/tests/acknowledgements.rs::a_refused_removal_of_the_renamed_file_is_reported_and_retried_hourly (verifies: LLR-3fxxpp) — green on arrival; mutation: remove_store treats every error but Other as gone → red, reverted
  red -> green: org-io/tests/acknowledgements.rs::a_deletion_failed_at_open_is_retried_once_an_hour_however_often_receiving_starts (verifies: LLR-3fxxpp) — green on arrival; mutation: retry_deletion's running-task check removed → red (2 deletions vs 1), reverted
  red -> green: org-io/tests/acknowledgements.rs::a_leftover_removed_file_is_retried_once_an_hour_however_often_receiving_starts (verifies: LLR-3fxxpp) — green on arrival; mutation: start_leftover_retry's running-task check removed → red, reverted
  red -> green: org-io/tests/verification.rs::an_unknown_write_on_chain_whose_adoption_failed_to_write_stays_still_unknown (verifies: LLR-zey4mg) — green on arrival; mutation: dropped `&& !its_write` in settle → red, reverted
  red -> green: org-io/tests/verification.rs::past_the_window_below_the_base_epoch_the_write_stays_still_unknown (verifies: LLR-zey4mg) — green on arrival; mutation: dropped the expected-epoch operand → red, reverted
  red -> green: org-io/tests/receive_step.rs::a_failed_store_write_is_receive_error_store_answered_0x01_and_the_persona_reloaded (verifies: LLR-kz9hj4) — green on arrival; mutation: store failure in check_and_keep ignored → red, reverted
  red -> green: org-io/tests/receive_step.rs::each_admission_report_is_filled_from_the_request_kept_for_its_own_invite (verifies: LLR-x36szv, LLR-zkybw7) — green on arrival; mutation: dropped `&& kept.invite_id == invite_id` → red, reverted
  red -> green: org-io/tests/chain_write.rs::a_foundings_unknown_outcome_refuses_a_second_write_of_that_founding_only (verifies: LLR-mhn4n5) — green on arrival; mutation: `if false &&` on write_founding's unknown-write check → red, reverted
  red -> green: org-io/tests/send_fanout.rs::loopback_dials_no_kept_address_outside_the_lower_case_hex_form (verifies: LLR-34tfm8) — green on arrival; mutation: lower-case hex check removed in parse_kept_address → red, reverted
  red -> green: org-io/tests/persona_creation.rs::propose_member_keeps_nothing_when_the_keychain_refuses_its_item (verifies: LLR-kna4zj, LLR-6d6wfw) — green on arrival; mutation: drop_store removed from propose_member's keychain branch → red, reverted
```

#### Guideline reviews — round 1 (2026-10-25) — Round 1 fix — TEST org-node (task `tfn`, merged 2026-10-25)

```
  red -> green: org-node/tests/receive_path.rs::from_an_unlisted_device_only_three_kinds_are_acted_on (verifies: LLR-w7b9kd) — M1: in check_received, `OrganisationRequest => listed || revoked` changed to `listed`; went red.
  red -> green: org-node/tests/receive_path.rs::a_revocation_from_an_unlisted_sender_is_refused_in_the_check (verifies: LLR-283x6f, LLR-r7zm39) — M2: a refused Revocation pushed onto kept_revocation_notices before SenderNotListed is returned; red at the "nothing kept" assert.
  red -> green: finding-2 fixtures, covering every admit_b/remove_b user in receive_path.rs, admission_sender.rs, receive_chain_reads.rs and provisional_store.rs — M3: messages_for_current_epoch returns no messages. Red in 11 receive_path, 5 receive_chain_reads, 3 provisional_store and 12 admission_sender tests, which shows the change-set message path now runs in these fixtures.
  red -> green: org-node/tests/own_removal.rs::a_removal_seals_a_removed_store_for_that_personas_device_key (verifies: LLR-2d5a7m) — M4: seal_removal seals but does not replace the held content; went red.
  red -> green: org-node/tests/own_removal.rs::into_removed_keeps_only_the_org_id_the_time_and_the_acknowledgements (verifies: LLR-2d5a7m) — M5: into_removed returns verified_at 0 and no acknowledgements; went red.
  red -> green: org-node/tests/absences.rs::into_removed_is_called_only_from_verify (verifies: LLR-9jn3z8) — M6: a second into_removed call added in cancel_invite; went red.
  red -> green: org-node/tests/absences.rs::into_removed_is_called_only_from_verify_and_an_acknowledgement_removed_only_on_acceptance (verifies: LLR-9jn3z8) — M6, same mutation; went red.
  red -> green: org-node/tests/absences.rs::only_the_removal_and_the_accepted_acknowledgement_delete (verifies: LLR-9jn3z8) — M7: drop_store removes nothing, red. M8: a second `StoreContent::Removed(..)` construction in record_founding_proxy, red, naming record_founding_proxy. A drop_store rewritten with position plus swap_remove stayed green, as finding-6 asks.
  red -> green: org-node/tests/sealed_store.rs::an_unknown_device_key_is_refused_as_persona_not_held (verifies: LLR-f9x3vf) — M9: verify resolves an unknown key to store 0; red with "verify: left Ok(()) right Err(PersonaNotHeld)".
  red -> green: org-node/tests/sealed_store.rs::a_sealed_store_unseals_with_its_secret_and_device_key, a_created_store_opens_under_the_hkdf_key_of_its_secret, the_store_key_is_hkdf_sha256_of_the_seed_under_the_versioned_label, restore_store_replaces_or_adds_one_store_and_drop_store_removes_one, list_personas_returns_one_record_per_store, unseal_refuses_each_malformed_input_with_its_own_error (verifies: LLR-bg538c, LLR-2qqqx6, LLR-f9x3vf) — M10: add_store seals a removed form instead of the new store; all six went red. The old seal_for_test fixture could not have caught this.
  red -> green: org-node/tests/sealed_store.rs::a_store_sealed_under_any_other_key_derivation_is_refused (LLR-2qqqx6), two_stores_with_one_device_key_are_refused (LLR-f9x3vf) and an_operation_changes_only_its_personas_store (LLR-f9x3vf) — M11 to M13 were applied together, one per test: STORE_KEY_INFO set to v2, OrgService::new's duplicate check removed, and build_invite also writing to another store. All three went red.
  red -> green: org-node/tests/sealed_store.rs::store_change_properties::every_store_changing_operation_returns_one_sealed_change_with_a_fresh_nonce (verifies: LLR-ay8nvu; T27 note (c)) — M14: seal_removal and acknowledgement_accepted seal with a zero-nonce RNG; red at "every write draws a fresh 24-byte nonce".
  red -> green: org-node/tests/organisation_request.rs::an_answer_whose_snapshot_does_not_decode_or_parse_is_refused_and_keeps_nothing (verifies: LLR-kd7u2r) — M15: keep_organisation_answer treats an undecodable snapshot as an empty member list; went red.
  red -> green: org-node/tests/organisation_key.rs::propose_founding_refuses_an_organisation_key_equal_to_the_founders_member_key (verifies: LLR-9qavm7) — M16: ensure_distinct_from call removed from propose_founding; went red.
  red -> green: org-node/tests/persona_records.rs::a_proposed_founding_persona_holds_the_parsed_details_across_a_reopen (verifies: LLR-pv2rdy) — M18: propose_founding stores handle None; went red.
  red -> green: org-node/tests/persona_records.rs::a_persona_record_decoded_directly_refuses_an_invalid_handle (verifies: LLR-pv2rdy) — M19: persona.handle parsed leniently (lowercased, refusal dropped); went red.
  inherited: org-node/tests/receive_path.rs::a_revocation_from_an_unlisted_sender_is_refused_in_the_check — from LLR-kzgjz8 (now LLR-283x6f)
  inherited: org-node/tests/receive_path.rs::an_acknowledgement_is_decided_in_the_check_from_its_named_device_only — from LLR-3aysup (now LLR-5wg8cu)
  inherited: org-node/tests/receive_chain_reads.rs::an_acknowledgement_is_checked_and_reported — from LLR-pt32fx and LLR-3aysup (now LLR-5wg8cu, the in-force acknowledgement rule it checks; pt32fx's direct successor LLR-jhy6t6 is not what it checks)
  inherited: org-node/tests/receive_chain_reads.rs::an_update_from_an_unlisted_device_is_refused_before_any_chain_state_is_asked_for — from REQ-ztdza4 and LLR-2r2fha (now REQ-uk9rw7, already annotated, and LLR-283x6f)
  inherited: org-node/tests/value_types.rs::every_rejection_variant_is_distinct_from_every_other — from LLR-mxskg9 and LLR-j5vbqj (now LLR-3sddey)
  inherited: org-node/tests/fuzz_wire_decode/fuzz_target.rs (fuzz_wire_decode) — from LLR-js9dsu (now LLR-xr88a8, through LLR-tajh9d)
  inherited: org-node/tests/organisation_key.rs::an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused — from LLR-sj7cd5 (now LLR-9qavm7). Its device-key half stays at ensure_distinct_from under Seams 2.1, because a 2^-128 collision cannot be forced through propose_founding.
  red -> green: app/src-tauri/tests/ipc.rs::a_second_person_is_refused_with_the_text_the_creation_form_passes_over (verifies: LLR-k57uqv, LLR-6ff735) — passed on arrival; mutation: PERSON_ALREADY_HELD in founding.ts changed to 'a Person is already held' → red (left/right differ); reverted
  red -> green: app/src-tauri/tests/periodic.rs::no_run_before_one_interval_then_every_held_target_once_per_interval (LLR-3ampqm) — mutation: interval_at(Instant::now(), …) → red; reverted
  red -> green: app/src-tauri/tests/periodic.rs::a_late_run_delays_the_next_rather_than_overlapping_or_bursting (LLR-3ampqm) — mutation 1: MissedTickBehavior::Burst → red ([260,300] vs [260,320]); mutation 2: runs spawned detached instead of awaited → red (reads 5 vs 2); both reverted
  red -> green: app/src-tauri/tests/periodic.rs::an_adopted_result_is_announced_as_the_receiver_loop_announces_it (LLR-2267hj) — mutation: emissions reversed and duplicated in periodic_runs → red; reverted
  red -> green: app/src-tauri/tests/periodic.rs::an_in_step_result_is_announced_by_its_verification_result_alone (LLR-2267hj) — same mutation → red; reverted
  red -> green: app/src-tauri/tests/periodic.rs::a_behind_organisation_is_requested_at_each_run_until_in_step (LLR-fe7umz) — mutation: send_and_request's behind arm drops the report → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::write_organisation_to_chain_tags_included_refused_and_unknown (LLR-rhc555) — mutation: write_then_verify maps Refused to Unknown → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::a_refusal_before_the_chain_is_org_ios_display_text (LLR-rhc555) — mutation: map_err uses Debug → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::an_included_write_is_verified_once_and_a_refused_or_unknown_one_is_not (LLR-k4rhbs) — mutation: Included verifies twice → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::an_included_founding_write_is_verified_by_its_founder_and_an_update_by_its_org_id (LLR-k4rhbs) — mutation: org_id_for drops the founding's org id → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::verify_organisation_over_an_unreadable_chain_is_unverified_with_org_ios_reason (LLR-jf63e4) — mutation: verify_organisation's Some arm returns [] → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::a_founding_with_no_proxy_is_entered_with_a_null_org_id_and_its_founder (LLR-jf63e4) — mutation: entry_for drops founder → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::a_founding_with_its_proxy_recorded_is_entered_with_its_org_id_and_still_its_founder (LLR-jf63e4) — mutation: org_id_for drops org id → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::an_own_adoption_is_sent_once_and_nothing_else_is (LLR-es49wp) — mutation: send guard `own: true` removed → red (the not-own entry carried sends); reverted
  red -> green: app/src-tauri/tests/verification.rs::a_behind_result_is_requested_once_with_no_user_step (LLR-nkg8sb) — mutation: behind arm drops the report → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::verify_all_reports_in_step_and_lists_the_organisation_verified (LLR-jf63e4, LLR-g8wfy9) — mutation: VerificationDto InStep → NotChecked → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::verify_all_over_an_unreadable_chain_reports_unverified_with_the_reason (LLR-jf63e4, LLR-g8wfy9) — mutation: list's Unverified reason prefixed → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::entry_for_tags_each_result_and_maps_store_keychain_and_refused_to_unverified (LLR-73by2q) — mutation: Refused → Inconsistent → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::a_settled_verification_is_its_outcomes_entry_with_the_write_label (LLR-73by2q) — mutation: settled.write = None → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::entry_for_names_a_founding_by_its_founder_beside_its_org_id_or_null (LLR-73by2q) — mutation: entry_for drops founder → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::sends_reports_each_device_sent_or_failed_beside_the_others (LLR-rywe8f, LLR-cugcu6) — mutation: AlreadyReceived text changed → red; reverted
  red -> green: app/src-tauri/tests/verification.rs::org_dto_reports_not_checked_removed_settled_and_refused_statuses (LLR-q96vdr) — mutation: Keychain → Inconsistent in VerificationDto::of → red; reverted
  red -> green: app/src-tauri/tests/receiver_events.rs::one_iteration_verifies_once_before_the_next_message_and_announces_what_it_verified (LLR-j9m88u) — mutation: the verify outcomes' emissions dropped in receive_iteration → red; reverted
  red -> green: app/src-tauri/tests/receiver_events.rs::follow_up_for_asks_a_verification_for_a_provisional_or_a_kept_notice_only (LLR-w7m3d4) — mutation: follow_up_for returns a different org id → red; reverted
  red -> green: on-chain-client/tests/write_compose.rs::the_substitute_reads_an_untouched_proxy_as_the_chain_does / ..._funded_unmapped_... / ..._mapped_to_itself_... (LLR-jk76wf, LLR-95puh9, LLR-htp5tk) — mutation: FakeOps free_balance +1 and mapped_account always None → all three red; reverted
  red -> green: on-chain-client/tests/write_genesis_e2e.rs::subxt_reads_answer_the_write_reads_contract — not run (chopsticks absent); no red evidence
  inherited: verification.rs::discarded_lists_each_change_tagged_by_kind_and_is_empty_not_absent — from LLR-n64wvr (body unchanged), plus LLR-73by2q, since it calls entry_for
  inherited: verification.rs::sends_reports_… — from LLR-rywe8f, plus LLR-cugcu6 (mutation-checked above)
  inherited: notices.discarded.test.ts "a failed listing …" and notices.admission.test.ts "a refused subscription …" — LLR-5mw2dj unchanged as an ID, now verifying the amended text (mutation-checked above)
```

#### Guideline reviews — round 1 (2026-10-25) — Round 1 fix — TEST org-io (task `tfo`, merged 2026-10-28)

```
  red -> green: `a_leftover_is_retried_hourly_though_the_listing_failed_when_receiving_started` (org-io/tests/acknowledgements.rs; verifies: LLR-3fxxpp). Red: panicked "came to hold" (acknowledgements.rs:489), because the leftover was never retried. Green after `may_hold_leftovers`.
  red -> green: `a_retry_that_cannot_list_the_data_directory_is_reported_and_retried_hourly` (org-io/tests/acknowledgements.rs; verifies: LLR-3fxxpp). Red: panicked "the unlisted retry is reported", because the report was `None` and the retry had silently ended. Green after the listing error became a reported failure.
  red -> green: each line names the mutation that turned it red; every mutation was reverted exactly, checked with git status/diff (src clean).
  red -> green: app/tests/notices.admission.test.ts::the Admit panel proposes nothing until the Admin confirms, then proposes once with confirmed true (verifies: LLR-e63yrs) — dropped `return` from the `!gate.ok` branch of doAdmit (the finding's false-green case) → red (propose_admission called unconfirmed); also `confirmed` true→false → red; reverted
  red -> green: app/tests/notices.admission.test.ts::the page lists kept requests once after it listens (verifies: LLR-5mw2dj) — moved listAdmissionRequests() before subscribeAll → red (the request announced during the start was missed); reverted
  red -> green: app/tests/notices.admission.test.ts::a refused subscription at start is shown on the page, not left unhandled (verifies: LLR-5mw2dj) — listenError label swapped to "verification at start failed:" → red (the same listing-first mutation also turned it red); reverted
  red -> green: app/tests/notices.admission.test.ts::an invite used by another device is an error message above the form (verifies: LLR-d8998k) — moved the alert below the Paste Invite field → red; reverted
  red -> green: app/tests/notices.admission.test.ts::no other outcome of the request shows that error (verifies: LLR-d8998k) — Invite always shows the invite_used message → red; reverted
  red -> green: app/tests/invite.confirm.test.ts::the invitee is shown the warning and nothing is requested until the user confirms (verifies: LLR-qcg6bz) — dropped `return` from the gate branch of doRequestAdmission → red; reverted
  red -> green: app/tests/invite.confirm.test.ts::a retry of a confirmed request asks nothing again (verifies: LLR-prh433) — removed the confirmedPersonas append → red (the warning is shown again); reverted
  red -> green: app/tests/founding.propose.test.ts::the creation form shows the founder and hands it to the founding-writes list (verifies: LLR-rhc555) — removed onProposed?.(founder) in CreateOrg → red; reverted
  red -> green: app/tests/notices.discarded.test.ts::reproposal names propose_admission or propose_revocation and the notice stays (verifies: LLR-md6hhz) — proposeAgain dismisses a revocation notice → red; reverted
  red -> green: app/tests/notices.discarded.test.ts::the panel proposes an admission again only on the confirmation, and a cancel proposes nothing (verifies: LLR-md6hhz, LLR-e63yrs) — dropped `return` from the gateAgain branch of confirmProposeAgain → red; reverted
  red -> green: app/tests/notices.discarded.test.ts::the confirmation shows a mismatch again, and no kept request proposes nothing (verifies: LLR-md6hhz, LLR-e63yrs) — removed the mismatch alert in Membership → red; reverted
  red -> green: app/tests/notices.discarded.test.ts::a failed listing of the kept admission requests shows under its own label, not the verification one (verifies: LLR-5mw2dj) — admissionsError label swapped to "verification at start failed:" → red; reverted
  red -> green: app/tests/verification.view.test.ts::asks for a verification once when it mounts (verifies: LLR-j2x9jb) — added setInterval(verifyOrganisations, 5 min) in onMount → red (13 calls in an hour, fake timers); reverted
  red -> green: app/tests/verification.view.test.ts::calls verify_organisation when its Verify with chain button is pressed (verifies: LLR-j2x9jb) — dropped fileResults?.(entries) in PersonaList.verify → red; reverted
  red -> green: app/tests/verification.view.test.ts::a verification-result event is filed newest first through verificationLine (verifies: LLR-84cxpt) — the page's verification-result handler files [] → red; reverted
  red -> green: app/tests/verification.view.test.ts::writes a founding with writeOrganisationToChain from its Write to chain button (verifies: LLR-rhc555) — writeFounding sends { orgId: founder } → red; reverted
  red -> green: org-io/tests/endpoints.rs::a_second_organisations_message_goes_out_under_its_own_personas_key (verifies: LLR-d2pzzk, PR-8qsnhx) — mutation: Endpoints::endpoint returned the first bound endpoint for any key → red (the receiving joiner timed out)
  red -> green: org-io/tests/endpoints.rs::after_its_endpoint_closes_a_send_binds_it_again (verifies: LLR-d2pzzk, PR-tcd2ta) — mutation: bound_or_bind kept closed endpoints → red
  red -> green: org-io/tests/endpoints.rs::a_send_that_rebinds_a_closed_endpoint_while_the_receiver_runs_accepts_on_it (verifies: LLR-z35vbh, LLR-d2pzzk, PR-47aycj) — mutation: endpoint() skipped keep_accepting while receiving → red "no accept task"
  red -> green: org-io/tests/endpoints.rs::a_refused_rebind_while_the_receiver_runs_binds_and_accepts_on_the_next_send (verifies: LLR-z35vbh, LLR-d2pzzk, PR-47aycj) — same mutation → red
  red -> green: org-io/tests/expectations_met.rs::every_store_change_reaches_the_file_whole_and_reopens_as_held (verifies: REQ-x6kdse) — mutation: issue_invite skipped its store write → red "after issue_invite"
  red -> green: org-io/tests/expectations_met.rs::a_store_file_opens_under_its_own_item_and_keeps_the_device_key_not_the_seed (verifies: REQ-qza96g) — mutation: StoreWriter::store appended the keychain seed to the file → red "the sealed file holds no seed"
  red -> green: org-io/tests/expectations_met.rs::the_device_seed_is_in_the_keychain_alone_and_renders_no_bytes (verifies: REQ-v6r2gd) — same mutation → red
  red -> green: org-io/tests/expectations_met.rs::outstanding_invites_are_kept_in_the_store_files_and_settled_by_adoption (verifies: REQ-2xgsbn; rewritten) — mutation: verify did not write the store on a later adoption → red
  red -> green: org-io/tests/store_files.rs::a_failed_write_reports_failure_and_reloads_the_persona_from_its_file (verifies: LLR-murjr6; rewritten) — mutation: no restore_store on reload → red "the change is gone"
  red -> green: org-io/tests/store_files.rs::a_dropped_persona_keeps_its_file_and_item_and_opens_again (verifies: LLR-murjr6; rewritten) — mutation: no drop_store → red
  red -> green: org-io/tests/store_files.rs::an_ordinary_save_succeeds_while_the_keychain_is_locked (verifies: REQ-6d9xv6; rewritten) — mutation: the store write read the keychain → red
  red -> green: org-io/tests/rng_source.rs::every_org_node_draw_and_the_temporary_suffix_come_from_the_handles_source (verifies: LLR-4qd5fx; rewritten) — mutation: propose_founding drew from OsRng → red
  red -> green: org-io/tests/rng_source.rs::a_failing_entropy_source_refuses_the_store_write_and_writes_nothing (verifies: LLR-94nn6q; rewritten) — mutation: the suffix draw fell back to OsRng → red
  red -> green: org-io/tests/chain_write.rs::fake_chain_meets_the_founding_contract (verifies: LLR-5tpdxw, LLR-sq5pbq) — red-first against the old fake (OrgId != h160_of(proxy)), green after the fake fix; also red when the fake's step text drifted
  red -> green: org-io/tests/chain_writer_contract.rs::the_production_writer_meets_the_founding_contract (verifies: LLR-5tpdxw, LLR-sq5pbq) — chopsticks only, not run (no node_modules); no red recorded
  red -> green: org-io/tests/acknowledgements.rs::a_store_dropped_from_memory_keeps_its_live_file_and_item (verifies: LLR-vj2nkb, LLR-3fxxpp) — mutation: T44's guard removed (finish_removed called unconditionally) → red "the live file's keychain item is kept"
  red -> green: org-io/tests/acknowledgements.rs::a_failed_direct_bind_is_every_recipients_outcome_and_the_next_round_binds_again (verifies: LLR-cqr26z) — mutations: (a) the bind failure was not recorded → red; (b) the task stopped after a failed bind → red "the next round binds again"
  red -> green: org-io/tests/acknowledgements.rs::the_deletion_renames_the_file_before_the_item_and_the_item_before_the_file (verifies: LLR-vj2nkb) — mutations: (a) file removed before the item → red; (b) item deleted before the rename → red
  red -> green: org-io/tests/verification.rs::the_finalised_block_is_read_before_the_state (verifies: LLR-t2pve7) — mutation: the two reads swapped → red
  red -> green: org-io/tests/verification.rs::a_failed_write_of_a_removal_is_verified_store_and_starts_no_delivery (verifies: LLR-zcw3xv) — mutation: a failed store write ignored on Removed → red
  red -> green: org-io/tests/verification.rs::a_provisional_member_reads_the_invites_organisation_and_adopts_nothing_itself (verifies: LLR-fp2cat; tightened to Refused{AdmissionNotOurs}) — mutation: org-node's refusal mapped to Behind → red
  red -> green: org-io/tests/requests.rs::a_request_whose_endpoint_cannot_be_bound_is_refused_as_a_failed_bind (verifies: LLR-adgk9z, LLR-uf5cyy) — mutations: (a) the admission request's bind error mapped to NotDelivered → red; (b) the organisation request's bind error mapped to an empty report → red
  red -> green: org-io/tests/chain_write.rs::a_window_read_that_never_returns_is_bounded_and_writes_nothing (verifies: LLR-d3farr) — mutation: bounded() removed from the window read → red (outer timeout)
  red -> green: org-io/tests/chain_write.rs::an_unknown_outcome_on_one_handle_never_refuses_its_other_target (verifies: LLR-mhn4n5) — mutation: a per-handle flag (any unknown write refuses all) → red; the two old LLR-mhn4n5 tests stayed green under it, which confirms finding-9
  red -> green: org-io/tests/persona_creation.rs::a_bind_that_fails_while_receiving_is_reported_and_undoes_no_creation (verifies: LLR-jvx6j9) — mutations: (a) the founding's bind error dropped → red; (b) propose_member undone on a failed bind → red
  inherited: org-io/tests/receive_step.rs::a_command_completes_while_the_receiver_waits — from LLR-9arvqj (now LLR-eeh4pq, PR-7v8anh)
```

#### Guideline reviews — round 1 (2026-10-25) — Round 1 fix — ARCHITECTURE (task `arch`, merged 2026-10-28)

```
  red -> green: org-io/tests/acknowledgements.rs::a_refused_listing_is_the_rounds_failure_and_the_next_due_round_sends_again (verifies: LLR-rav7dr, LLR-bv6wkz) — red: panicked "the report came to hold" (the refusal ended the task and deleted the Persona) → green after round returns Result
  red -> green: org-io/tests/acknowledgements.rs::a_removal_open_found_is_reported_and_resumed_though_the_directory_cannot_be_listed (verifies: LLR-5rxqr8, LLR-rav7dr) — red: panicked "the removal open found is reported while the directory cannot be listed" (report None, no round) → green after the remembered keys
  red -> green: org-io/tests/chain_write.rs::a_write_refusal_keeps_its_cause_as_its_source (verifies: LLR-48me7m, LLR-5tpdxw) — red: source() None for Node → green after the source() impl
  red -> green: org-io/tests/absences.rs::org_io_reads_org_node_stores_only_through_key_free_views (verifies: LLR-jqb5z7) — green on arrival; mutation: planted `service.list_orgs()` in verify.rs → red, reverted
  red -> green: org-node/tests/proposals.rs::persona_records_and_the_person_view_copy_the_public_fields_and_no_seed (verifies: LLR-8ru6mv) — green on arrival; mutations: PersonView.person_key from the device key → red; PersonaRecordView.status forced Revoked → red; both reverted
  red -> green: org-io/tests/absences.rs::org_io_removes_nothing_itself_but_in_the_persona_deletion (verifies: LLR-vvd8dn; rewritten for retention.rs) — mutation: planted mark_removed( call in acknowledge.rs → red "deletes outside retention.rs", reverted
  red -> green: org-io/tests/acknowledgements.rs::org_io_learns_and_removes_acknowledgements_only_through_org_node (verifies: LLR-sn5ded; rewritten) — mutation: planted ".acknowledgement_accepted(" in retention.rs → red, reverted
  red -> green: org-io/tests/keychain.rs::the_write_lends_the_bytes_only_through_lend_to_keychain (verifies: LLR-6sfub6) — mutation: second set_secret call outside lend_to_keychain → red, reverted
  red -> green: app/tests/notices.admission.test.ts::the Admit panel proposes nothing until the Admin confirms, then proposes once with confirmed true (verifies: LLR-e63yrs; new label assertion) — mutation: Admit label back to "Device key" → red, reverted
  inherited: org-io/tests/keychain.rs::the_write_lends_the_bytes_only_through_lend_to_keychain — from LLR-kna4zj (now LLR-6sfub6)
  inherited: org-io/tests/chain_read_rule.rs::the_state_reader_is_named_only_in_verify_and_preflight — LLR-k3zmqb unchanged as an ID; retention.rs added to its list
  inherited: org-node/tests/person_store.rs::the_person_and_member_types_are_named_under_org_node_paths — LLR-5uf4u2; PersonView added
```

#### Guideline reviews — round 2 (2026-10-28, head 29f9d82e) — Round 2 fix — org-node CODE/TEST and ARCHITECTURE AR2-7/8/9/10/11 (task `n2`, merged 2026-10-28)

```
  red -> green: value_types::the_new_variants_are_distinct_and_their_display_names_the_value (extended) — DuplicateOrgPublicKey: failed to compile (no variant) before the variant existed; field-carrying refusals: mutation check (Display without invite_id / device_key) failed the test, reverted
  red -> green: organisation_key::an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused (expectation changed) — failed to compile (no DuplicateOrgPublicKey) before implementation
  red -> green: organisation_key::propose_founding_refuses_an_organisation_key_equal_to_the_founders_member_key (expectation changed) — same red
  red -> green: proposals::a_founding_key_equal_to_a_founder_key_is_refused_adding_no_store / a_founding_key_equal_to_a_founder_key_is_refused / a_key_equal_to_the_records_or_to_a_key_of_the_result_is_refused_keeping_nothing / an_updated_organisation_key_equal_to_the_records_or_to_a_key_of_the_result_is_refused (expectations changed) — same red
  red -> green: absences::a_kept_acknowledgement_is_removed_only_on_acceptance — green on arrival; mutation: `acknowledgements.clear()` added to kept_acknowledgements failed it; renaming remove_acknowledgement kept it green (the refactor the finding asked to allow); reverted
  red -> green: absences::verification_revocation_and_removal_do_no_io_and_read_no_clock (scan narrowed) — mutation: `pub fn mutation_probe(data: &mut StoreData)` in verify_org.rs failed it; reverted
  red -> green: proposals::a_personas_organisation_is_set_once_and_kept_by_every_store_change — green on arrival; mutation: cancel_invite writing data.persona.org_id failed it ("cancel_invite"); reverted
  red -> green: own_removal::a_removal_by_a_match_keeps_three_two_one_or_no_recipients — green on arrival; mutation `3 | 4 => 3` in recipient_count failed it ("n = 3: k = 2"); reverted
  red -> green: own_removal::the_still_listed_come_first_on_a_match_and_all_ascending_on_a_notice — green on arrival; mutation sorting all recipients ascending in recipients() failed it; reverted
```

#### Guideline reviews — round 2 (2026-10-28, head 29f9d82e) — Round 2 fix — org-io CODE/TEST and ARCHITECTURE AR2-5/6 (task `o2`, merged 2026-10-28)

```
  red -> green: org-io/tests/propose.rs::an_organisation_no_persona_holds_is_org_ios_own_refusal_and_writes_nothing (verifies: LLR-un47j8, LLR-6kb33y, LLR-5fchwt) — red: no variant `OperationError::NotHeld` (compile) → green after AR2-5
  red -> green: org-io/tests/invitation_exchange.rs::issue_invite_builds_in_the_holding_persona_and_writes_before_it_returns (changed; verifies: LLR-95e3jx) — red: no variant `ExchangeError::NotHeld` → green
  red -> green: org-io/tests/invitation_exchange.rs::cancel_invite_writes_the_store_and_refuses_an_invite_with_no_kept_request (changed; verifies: LLR-8ze6cc) — red: no variant `ExchangeError::NotHeld` → green
  red -> green: app/src-tauri/tests/receiver_events.rs::terminal_is_decided_by_the_error_variant_not_its_text (changed, NoOrganisation case; verifies: LLR-zw6jqy) — red: no variant `ReceiveError::NoOrganisation` → green
  red -> green: org-io/tests/acknowledgements.rs::a_clock_before_the_epoch_is_the_rounds_failure_and_nothing_is_sent_or_deleted (verifies: LLR-bv6wkz, LLR-rav7dr) — red: "the report came to hold: Elapsed" (round ran at time 0, no failure) → green after C2-O1
  red -> green: org-io/tests/acknowledgements.rs::open_judges_no_deletion_on_a_clock_before_the_epoch_and_reports_it (verifies: LLR-upqj5g, LLR-5rxqr8) — red: report had last_deletion_failure None → green after C2-O1
  red -> green: org-io/tests/acknowledgements.rs::a_failed_read_of_the_list_at_the_deletion_is_reported_and_retried_hourly (verifies: LLR-3fxxpp) — red: no deletion failure reported (refusal read as not held) → green after C2-O2
  red -> green: org-io/tests/acknowledgements.rs::a_refused_listing_is_the_rounds_failure_and_the_next_due_round_sends_again (changed; verifies: LLR-rav7dr, LLR-bv6wkz) — red: no method fail_acknowledgement_listing_for_test → green after T2-O2
  red -> green: org-io/tests/invitation_exchange.rs::a_failed_write_of_a_cancel_is_exchange_error_store_and_reloads_the_persona (verifies: LLR-8ze6cc) — green on arrival; mutation M1 (cancel ignores its store failure) made it fail, reverted
  red -> green: org-io/tests/receive_step.rs::a_failed_store_write_of_a_request_is_receive_error_store_0x01_and_no_answer (verifies: LLR-898z24) — green on arrival; mutation M2 (receive step ignores store failure) made it fail, reverted
  red -> green: org-io/tests/send_fanout.rs::another_byte_or_none_within_the_limit_is_reported_not_delivered (verifies: LLR-fwz3tb) — green on arrival; mutations M3a (unknown byte → Refused) and M3b (timeout → 0x00) each made it fail, reverted
  red -> green: org-io/tests/person.rs::a_message_addressed_to_the_persons_key_is_never_accepted (verifies: LLR-g6kaa8) — green on arrival; mutation M4 (bind the Person's endpoint when proposed while receiving) first SURVIVED, test strengthened with the bound-endpoint count, then failed under M4, reverted
  red -> green: org-io/tests/acknowledgements.rs::a_delivery_that_starts_with_none_kept_deletes_the_persona_at_once (verifies: LLR-n649th, LLR-5rxqr8) — green on arrival; mutation M5 (resume skips a NoneKept removal) made it fail, reverted
  red -> green: org-io/tests/propose.rs::propose_updated_organisation_returns_the_epoch_root_and_key_and_makes_no_chain_call (changed, "nothing written" added; verifies: LLR-5fchwt) — green on arrival; mutation M6 (the refusal writes the store file) made it fail, reverted
```

#### Guideline reviews — round 2 (2026-10-28, head 29f9d82e) — Round 2 fix — app CODE/TEST, ARCHITECTURE AR2-1/2/3/4, TEST guideline tailoring (task `a2`, merged 2026-10-28)

```
  red -> green: app/src-tauri/tests/receiver_events.rs::an_entry_with_no_org_id_names_none_in_record_unreadable_or_revoked (verifies: LLR-acb5g4, LLR-ygr6gd) — failed to compile: RecordUnreadable/SelfDeleted had no "none" org id
  red -> green: app/src-tauri/tests/verification.rs::an_own_adoption_asks_a_send_and_behind_a_request_and_nothing_else_asks_anything (verifies: LLR-292r26) — follow_up module missing
  red -> green: app/src-tauri/tests/verification.rs::only_an_included_write_asks_a_verification (verifies: LLR-bbfgk7) — follow_up module missing
  red -> green: app/src-tauri/tests/ipc.rs::a_second_person_is_refused_by_its_kind_beside_org_ios_text (verifies: LLR-k57uqv) — refusal was a bare string
  red -> green: app/src-tauri/tests/ipc.rs::propose_person_refuses_an_invalid_field_naming_it_and_creates_nothing (changed) — kind absent
  red -> green: app/tests/founding.propose.test.ts::a Person held already is passed over by its kind… — the form branched on the text
  red -> green: app/tests/founding.propose.test.ts::any other refusal of the Person is shown as its text… — the form showed [object Object]
  red -> green: app/src-tauri/tests/verification.rs::a_send_or_request_refused_as_a_whole_is_an_empty_report_and_its_text (verifies: LLR-k4rxyt, LLR-es49wp, LLR-nkg8sb) — attach_report missing
  red -> green: fake-contract drift (invite.confirm x2, notices.admission x2, notices.discarded x3, verification.view x2, changed) — fake values refused by the new contract, then fixed
  red -> green: app/src-tauri/tests/ipc.rs::{a_fresh_stores_answers_and_refusals_have_the_fakes_shapes, the_writes_and_verifications_have_the_fakes_shapes, the_exchanges_answers_and_events_have_the_fakes_shapes, an_adoption_with_discarded_changes_is_announced_in_the_fakes_shape, every_backend_entry_has_a_real_producer_here} — green on arrival; all failed with backend.json mutated (status, transport_mode, verification tag, member_id width, an extra entry), reverted
  red -> green: app/tests/verification.view.test.ts::offers each held founding once… — green on arrival; failed with the foundingWrites dedupe removed, and again with the order reversed; reverted
  red -> green: app/tests/verification.view.test.ts::a verification-result event is filed newest first… (changed) — green on arrival; failed with the fileVerificationResults order reversed; reverted
  red -> green: app/tests/verification.view.test.ts::an event that arrives before the start verification returns… — green on arrival; failed when a later filing overwrote the list; reverted
  red -> green: app/tests/notices.discarded.test.ts::the confirmation is the kept request of that Organisation and invite… — green on arrival; failed with reproposalStep's org check dropped; reverted
  red -> green: app/src-tauri/tests/ipc.rs::a_blank_handle_is_handed_on_as_none_and_another_unchanged (changed) — green on arrival; failed with no blank filter, and again with trimming; reverted
  red -> green: app/src-tauri/tests/ipc.rs::request_admission_tags_each_outcome_and_never_reports_an_used_invite_as_refused (changed) — green on arrival; no own mutation check; its `refused` tag is also checked by the mutation-checked contract test
  red -> green: app/src-tauri/tests/verification.rs::a_behind_request_a_device_refuses_is_reported_failed_for_that_device (verifies: LLR-nkg8sb) — green on arrival; failed with the requests report emptied; reverted
  red -> green: app/src-tauri/tests/ipc.rs::issue_invite_for_an_organisation_not_held_is_org_ios_refusal_as_its_text (verifies: LLR-y3zykm) — green on arrival; failed with the text reworded; reverted
  red -> green: app/src-tauri/tests/periodic.rs::a_run_whose_chain_read_fails_is_announced_unverified_and_the_next_run_recovers (verifies: LLR-2267hj) — green on arrival; failed when unverified entries were dropped; reverted
  red -> green: on-chain-client/tests/write_compose.rs::the_substitute_reads_* x3 (changed; verifies: LLR-jk76wf, LLR-95puh9) — green on arrival; the funded test failed with found_at's `<` changed to `<=`; reverted
```

#### Guideline reviews — round 3 (2026-10-28, head 663647b4) — Round 3 fix — guideline tailoring, app and org-node docs, founding decision (task `a3`, merged 2026-10-28)

```
  red -> green: app/tests/founding.decision.test.ts::proceeds past a Person held already, whatever its wording (verifies: LLR-d8u93q) — watched fail (personRefusalStep is not a function)
  red -> green: app/tests/founding.decision.test.ts::stops on a refusal of any other kind, with its message, though worded like a held Person (verifies: LLR-d8u93q) — watched fail (not a function)
  red -> green: app/tests/founding.decision.test.ts::stops on a rejection that is a plain string, with that string (verifies: LLR-d8u93q) — watched fail (not a function)
  red -> green: app/tests/founding.decision.test.ts::stops on any other rejection with its text, never undefined (verifies: LLR-d8u93q) — watched fail (not a function)
  red -> green: app/tests/founding.decision.test.ts::imports types alone, so it names no IPC adapter function (verifies: LLR-5sxa6c) — watched fail (founding.ts imported proposePerson from ./api); after the ?raw import, re-reddened by a value import, reverted
  red -> green: app/tests/verification.view.test.ts::a status it does not know is labelled unverified, never not checked (verifies: LLR-njb6mr) — watched fail (Received "not checked")
  red -> green: app/tests/api.admit.test.ts::invokes propose_admission with the org id, the invite id, the handle and the confirmation alone (changed; verifies: LLR-gmb4p8) — green on arrival; reddened by an extra peerAddr argument, reverted
  red -> green: app/tests/verification.view.test.ts::writes a founding with writeOrganisationToChain from its Write to chain button (changed; verifies: LLR-rhc555) — green on arrival; reddened by changing writeOrganisationToChain's arguments, reverted
```

#### Guideline reviews — round 3 (2026-10-28, head 663647b4) — Round 3 fix — org-io TEST and ARCHITECTURE (task `o3`, merged 2026-10-28)

```
  red -> green: org-io/tests/chain_write.rs::a_write_past_ninety_seconds_is_unknown_and_recorded (changed; verifies: LLR-d3farr, LLR-7y46be) — green on arrival; failed with the timeout's record removed; reverted
  red -> green: org-io/tests/chain_write.rs::a_window_read_that_never_returns_is_bounded_and_writes_nothing (changed; verifies: LLR-d3farr) — green on arrival; failed with an unknown write recorded on the window timeout; reverted
  red -> green: org-io/tests/chain_write.rs::the_unknown_write_holds_root_key_expected_epoch_and_last_valid_block (changed; verifies: LLR-7y46be) — green on arrival; failed with last_valid_block+1, and again with the root not recorded; reverted
  red -> green: org-io/tests/chain_write.rs::a_failed_window_read_writes_nothing (changed; verifies: LLR-a66ufr) — green on arrival; failed with an unknown write recorded on the window failure; reverted
  red -> green: org-io/tests/chain_write.rs::a_refused_write_records_no_unknown_write (changed, re-annotated; verifies: LLR-d3farr, LLR-7y46be) — green on arrival; failed with an unknown write recorded on a refused update; reverted
  red -> green: org-io/tests/chain_write.rs::a_refused_found_at_step_is_refused_naming_it_and_keeps_the_proxy (changed; verifies: LLR-sq5pbq) — green on arrival; failed with an unknown write recorded on a found_at failure; reverted
  red -> green: org-io/tests/chain_write.rs::a_create_proxy_timeout_records_no_unknown_write_and_the_next_write_creates_again (changed: accessor line dropped; verifies: LLR-sq5pbq, LLR-d3farr) — its existing next-write assertion carries the check; no own mutation run
  red -> green: org-io/tests/chain_write.rs::each_write_refusal_displays_human_readable_text (re-annotated; verifies: LLR-5tpdxw) — green on arrival; failed with the Store refusal's proxy not named; reverted
  red -> green: org-io/tests/chain_write.rs::fake_chain_meets_the_reader_and_writer_contract (new; verifies: LLR-t2pve7, LLR-ce42ce, LLR-a66ufr, LLR-s7rq5x, LLR-7qdjqn) — green on arrival; failed with the fake's window born 5 blocks late, its stale refusal as Subxt, and its parse refusal as Chain; each reverted
  red -> green: org-io/tests/chain_writer_contract.rs::the_production_chain_meets_the_reader_and_writer_contract (new, outside the gate; same verifies) — not run against a fork here (skips: no chopsticks); its assertions are the fake's, mutation-checked above
  red -> green: org-io/tests/chain_writer_contract.rs::the_production_writer_meets_the_founding_contract (changed: skips with no fork; verifies: LLR-5tpdxw, LLR-sq5pbq) — skip confirmed; not run against a fork
  red -> green: org-io/tests/rng_source.rs::a_seeded_source_reproduces_a_draw_under_test_support (changed; verifies: LLR-5acgfg) — green on arrival; failed with with_rng_for_test ignoring its source; reverted
  red -> green: org-io/tests/verification.rs::a_verification_that_adopts_an_unknown_write_reports_it_adopted_and_clears_it (changed: accessor line dropped; verifies: LLR-t2pve7, LLR-zey4mg) — its existing not-refused assertion carries the check
  red -> green: org-io/tests/verification.rs::an_unknown_write_on_chain_whose_adoption_failed_to_write_stays_still_unknown (changed; verifies: LLR-zey4mg) — green on arrival; failed with settlement always clearing; reverted
  red -> green: org-io/tests/verification.rs::past_the_window_below_the_base_epoch_the_write_stays_still_unknown (changed; verifies: LLR-zey4mg) — green on arrival; failed with settlement always clearing; reverted
  red -> green: org-io/tests/verification.rs::a_failed_write_of_a_removal_is_verified_store_and_starts_no_delivery (changed; verifies: LLR-zcw3xv) — green on arrival; failed with acknowledge::start removed from Removed; reverted
  red -> green: org-io/tests/verification.rs::an_adoption_that_leaves_nothing_out_lists_no_discarded_change (new; verifies: LLR-gaanm2) — green on arrival; failed with a fabricated discarded entry per member; reverted
  red -> green: org-io/tests/persona_creation.rs::a_founding_is_proposed_with_the_chain_failing_and_makes_no_chain_call (new; verifies: LLR-4yte89) — green on arrival; failed with a finalised_block read added to the proposal; reverted
  red -> green: org-io/tests/persona_creation.rs::an_empty_and_a_spaced_non_ascii_organisation_name_are_reported_unchanged (new; verifies: LLR-pn3hgc) — green on arrival; failed with OrgSummary trimming the name; reverted
  red -> green: org-io/tests/acknowledgements.rs::a_removed_outcome_starts_one_delivery_task_and_returns_without_awaiting_it (changed; verifies: LLR-zzsqj4) — green on arrival; failed with start's running-task check disabled (a second acknowledgement arrived); reverted
  red -> green: org-io/tests/acknowledgements.rs::a_delivery_that_starts_with_none_kept_deletes_the_persona_at_once (re-annotated; verifies: LLR-n649th, LLR-5rxqr8, LLR-zzsqj4) — green on arrival; failed with an empty list not taken as due; reverted
  red -> green: org-io/tests/app_boundary.rs::the_app_calls_no_adopting_operation_and_builds_no_service (re-annotated; verifies: LLR-ntag3g) — green on arrival; failed with OrgService::new( planted in app/src-tauri/src/follow_up.rs; reverted
  removed: org-io/tests/acknowledgements.rs::a_refused_listing_is_the_rounds_failure_and_the_next_due_round_sends_again and ::a_failed_read_of_the_list_at_the_deletion_is_reported_and_retried_hourly (owner ruling T3-O2)
  inherited: none
```

#### Guideline reviews — round 3 (2026-10-28, head 663647b4) — Round 3 follow-up — app leftovers (task `a4`, merged 2026-10-28)

```
  red -> green: app/src-tauri/tests/ipc.rs::every_backend_entry_has_a_real_producer_here (changed) — failed: the producer list named list_personas.provisional_member, which backend.json did not hold
  red -> green: app/src-tauri/tests/ipc.rs::the_exchanges_answers_and_events_have_the_fakes_shapes (changed; verifies gains LLR-pguhw5) — failed: "backend.json holds no answers list_personas.provisional_member"
  red -> green: app/src-tauri/tests/ipc.rs::a_fresh_stores_answers_and_refusals_have_the_fakes_shapes (changed; verifies gains LLR-pguhw5) — the founder entry and its producer went in together; not watched fail separately
  red -> green: app/tests/personas.list.test.ts::handleLabel > shows a Handle as it is (verifies: LLR-q735zx) — failed: src/lib/personas.ts missing
  red -> green: app/tests/personas.list.test.ts::handleLabel > shows a missing Handle as the stated label, never as null (verifies: LLR-q735zx) — failed: src/lib/personas.ts missing
  red -> green: app/tests/personas.list.test.ts::the Persona list > shows a provisional Member with no Handle by the stated label and not as "null" (verifies: LLR-q735zx) — failed: expected '' to be '(no Handle yet)'
  red -> green: app/tests/personas.list.test.ts::the Persona list > shows a Persona that holds a Handle by that Handle (verifies: LLR-q735zx) — failed only on the fake refusing a shape not in backend.json; guards the normal case, not red-first on the component
  inherited: none
```

#### Guideline reviews — round 4 (2026-10-28, head 8eb3f15f) — Round 4 fix — CODE app + on-chain-client, ARCH low 5, TEST lows (task `a5`, merged 2026-10-28)

```
  red -> green: app/tests/revoke.validate.test.ts::accepts a 64-character hexadecimal member id (changed; verifies: LLR-csbs5v) — green on arrival; failed with validateRevokeInput refusing an all-'b' id; reverted
  inherited: none
```

#### Guideline reviews — round 4 (2026-10-28, head 8eb3f15f) — Round 4 fix — CODE and TEST org-node, AR4-3, AR4-7 part (task `n4`, merged 2026-10-28)

```
  red -> green: org-node/tests/verification.rs::an_admit_whose_request_matches_a_listed_member_on_one_key_only_is_discarded (new; verifies: LLR-d9x6xf) — green on arrival; failed with verify_org's `lists` matching on the member key only, and again on the Device key only; reverted
  red -> green: org-node/tests/verification.rs::a_revoke_whose_member_id_the_adopted_record_still_holds_is_discarded (new; verifies: LLR-d9x6xf) — green on arrival; failed with a Revoke rule that ignores the MemberId; reverted
  red -> green: org-node/tests/receive_path.rs::check_received_takes_no_chain_state_and_refuses_an_oversized_or_undecodable_body (changed; verifies: LLR-94rzrk) — exact-text assertion removed; no new behaviour claimed
  red -> green: org-node/tests/organisation_request.rs::build_organisation_request_refuses_a_recorded_founding_not_yet_adopted_as_org_not_held (new; verifies: LLR-vyank2) — failed with the (None, None) arm returning FoundingNotWritten; reverted
  red -> green: org-node/tests/organisation_request.rs::a_listed_requesters_invalid_addresses_are_refused_and_answered_nothing (new; verifies: LLR-7kdhst) — failed with the address parse falling back to empty; reverted
  red -> green: org-node/tests/organisation_request.rs::a_listed_requester_is_answered_with_the_full_record_and_its_key (changed; verifies: LLR-7kdhst) — failed with the address-book change also rewriting org_name; reverted
  red -> green: org-node/tests/proposals.rs::every_per_store_view_refuses_an_unknown_device_key_as_persona_not_held (new; verifies: LLR-8ru6mv) — failed with provisional_changes using store_index(..).unwrap_or(0); reverted
  red -> green: org-node/tests/own_removal.rs::every_other_operation_on_a_removed_store_is_organisation_removed (changed, persona_view added; verifies: LLR-myfk8w) — failed with persona_view mapping the error to FoundingNotWritten; reverted
  red -> green: org-node/tests/proposals.rs::propose_founding_calls_no_chain_operation_and_returns_the_secret_by_value (changed; verifies: LLR-d4gpxm) — text check removed; the compiler pins the return type; no new assertion
  red -> green: org-node/tests/absences.rs::the_service_presents_no_chain_write (changed; verifies: LLR-7en38k, LLR-x5asbx) — green on arrival; failed with a proxy_account read in record_admission, with `pub fn submit_mutation(&self)`, and with `pub fn hand_on(&self, _: ChainAccount)`; each reverted
  red -> green: org-node/tests/outgoing_bodies.rs::messages_for_the_current_epoch_inform_every_listed_device_but_the_own_then_notify_every_removed_one (changed; verifies: LLR-degg8t) — failed with notices_for naming another MemberId (which the old assert_notice passed); reverted
  red -> green: org-node/tests/outgoing_bodies.rs::an_adopted_removal_sends_revocations_to_every_removed_device_after_the_listed_ones (changed; verifies: LLR-degg8t) — same mutation, same failure; reverted
  red -> green: org-node/tests/revocation.rs::acknowledgement_domain::acknowledgement_signed_bytes_are_the_domain_then_the_tuple (changed; verifies: LLR-bs6yxf) — failed with a second sign over [ACK_DOMAIN, b"other"], and with a second const holding the same bytes (which the old test passed); reverted
  red -> green: org-node/tests/organisation_key.rs::an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused (re-annotated; verifies: REQ-ech45n) — failed with the Device clause of ensure_distinct_from disabled; reverted
  changed, assertions or labels removed only: revocation.rs::accept_refuses_in_order, revocation.rs::accept_returns_ok_for_a_later_epoch_without_the_device_and_takes_no_secret_source, wire_frame_bound.rs::a_key_field_that_is_not_32_bytes_does_not_decode
  inherited: none
```

#### Guideline reviews — round 4 (2026-10-28, head 8eb3f15f) — Round 4 fix — org-io CODE, TEST and ARCHITECTURE (task `o4`, merged 2026-10-28)

```
  red -> green: org-io/tests/verification.rs::a_failed_read_keeps_the_readers_error_as_its_source (new; verifies: LLR-ce42ce) — failed to compile (Cause<OrgNodeError> has no Error source) before ChainReadError existed
  red -> green: org-io/tests/chain_write.rs::a_create_proxy_timeout_records_no_unknown_write_and_the_next_write_creates_again (changed) — failed to compile (no CreateProxyUnanswered) before the rename
  red -> green: org-io/tests/keychain_contract.rs::the_in_memory_keychain_refusing_reads_answers_as_a_store_locked_after_a_write (new; verifies: LLR-36jer8) — failed at locked_read with the read refusals changed to PlatformFailure; reverted
  red -> green: org-io/tests/chain_write.rs::fake_chain_meets_the_reader_and_writer_contract (re-annotated; verifies: LLR-js7wgk, LLR-j4nkx5, LLR-bwerd2; epoch_ahead clause added) — failed clause 1 with finalised_block returning Ok(0), and the epoch_ahead clause with epoch_ahead set to +2; reverted
  red -> green: org-io/tests/chain_writer_contract.rs::the_production_chain_meets_the_reader_and_writer_contract (re-annotated only) — runs only on chopsticks, outside the gates; not run here
  red -> green: org-io/tests/receive_step.rs::a_refused_body_leaves_no_lock_held_for_the_next_wait (new; verifies: LLR-eeh4pq) — timed out (Elapsed) with the refusal path changed to leak the node's lock; reverted
  red -> green: org-io/tests/person.rs::a_refused_proposal_and_a_reopened_person_stay_provisional_with_their_fields (new; verifies: LLR-mas8vc) — failed with PersonSummary's name lowercased; reverted
  red -> green: org-io/tests/chain_write.rs::a_failed_store_write_after_create_proxy_stops_and_names_the_proxy (changed; verifies: LLR-5tpdxw) — failed "names the proxy create_proxy returned" with WriteRefused::Store's proxy set to zeros; reverted
  red -> green: org-io/tests/persona_creation.rs::a_persona_created_while_receiving_is_bound_and_accepted_on (changed; verifies: LLR-jvx6j9) — timed out with accept_if_receiving changed to bind without accepting; reverted
  inherited: none
```

#### Guideline reviews — round 5 (2026-10-28, head 59701b70) — Round 5 fix — CODE and TEST org-node (task `n5`, merged 2026-10-28, after the master merge of 641bec1)

```
  red -> green: org-node/tests/key_custody.rs::two_device_seeds_give_two_device_keys (re-annotated; verifies: LLR-ctzkv7) — green on arrival; failed with from_keychain_bytes ignoring its bytes; reverted
  red -> green: org-node/tests/organisation_key.rs::an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused (re-annotated; verifies: LLR-c759yd) — failed with the DevicePublicKey comparison dropped from ensure_distinct_from; reverted
  red -> green: org-node/tests/person_store.rs::an_authentic_person_store_holding_a_refused_value_or_another_device_key_is_refused (new; verifies: LLR-73sgz2) — failed to compile without seal_person_plaintext_for_test; the behaviour already existed, and the test failed with the device-key check dropped, with Undecodable mapped to NotAuthentic, and with the person.device_key field renamed; each reverted
  red -> green: org-node/tests/proposals.rs::a_revocation_refused_by_the_bound_records_nothing (new; verifies: LLR-f3hru6, LLR-m76r4k) — green on arrival; failed with ensure_within_bound removed from record_revocation; reverted
  red -> green: org-node/tests/verification.rs::adoption_replaces_the_record_and_reports_own_in_one_sealed_change (changed; verifies: LLR-34c5y3) — failed with `behind = None` removed from prune_after_adoption; reverted
  red -> green: org-node/tests/organisation_request.rs::a_listed_requester_is_answered_with_the_full_record_and_its_key (changed) — failed with a Debug that renders a secret's bytes; reverted
  red -> green: org-node/tests/proposals.rs::founding_and_member_details_refuse_each_invalid_field_naming_it, ::propose_member_refuses_no_handle_four_handles_or_a_repeated_one (changed; the second verifies: LLR-6acxu9, LLR-rjphn4) — each failed with a draw at the top of propose_member; reverted
  red -> green: org-node/tests/verification.rs::a_founding_whose_chain_is_past_epoch_one_is_behind_and_keeps_the_founding, ::a_founding_whose_members_do_not_rebuild_the_chain_root_is_not_adopted (re-annotated; verifies: LLR-8528ud) — one failed with `epoch >= 1`, the other with the rebuild check dropped; reverted
  red -> green: org-node/tests/verification.rs::a_founding_whose_root_the_chain_holds_under_another_key_is_not_adopted (new; verifies: LLR-8528ud) — green on arrival; failed with the org_pub_key comparison dropped from founding_matches; reverted
  red -> green: org-node/tests/receive_path.rs::an_unheld_organisations_notice_is_refused_as_not_held (changed, re-annotated; verifies: LLR-95av3e) — failed with check_without_record returning MalformedMessage; reverted
  red -> green: org-node/tests/receive_path.rs::an_admission_request_with_no_four_or_a_repeated_handle_is_refused (new; verifies: LLR-6gkxpg) — green on arrival; failed with check_preferred_handles removed for the request; reverted
  red -> green: org-node/tests/receive_path.rs::expected_organisation_information_failing_its_snapshot_or_chain_free_checks_is_refused (new; verifies: LLR-e6w3nr) — green on arrival; failed with check_chain_free removed from keep_information; reverted
  red -> green: org-node/tests/admission_sender.rs::persona_public_keys_returns_the_personas_two_keys (changed) — failed with persona_index refusing with another variant; reverted
  red -> green: org-node/tests/sealed_store.rs::every_store_changing_operation_returns_one_sealed_change_with_a_fresh_nonce (changed; verifies: LLR-ay8nvu) — failed with a byte of each sealed write flipped in OrgService::change; reverted
  red -> green: org-node/tests/secret_redaction.rs::no_kind_debug_shows_an_organisation_private_key_byte (changed; verifies: LLR-4ces44) — failed with a Debug that renders a secret's bytes; reverted
  red -> green: org-node/tests/absences.rs::a_kept_acknowledgement_is_removed_only_on_acceptance (changed; one shared list, with dedup) — failed with a public probe calling acknowledgements.dedup_by; reverted
  red -> green: org-node/tests/fuzz_unseal/fuzz_target.rs (changed assertions) — not mutation-checked: a 1 s fuzz budget seldom reaches an authentic plaintext
  inherited: none
```

#### Guideline reviews — round 5 (2026-10-28, head 59701b70) — Round 5 fix — TEST and CODE org-io (task `o5`, merged 2026-10-29, after the master merge and n5)

```
  red -> green: org-io/tests/send_fanout.rs::send_organisation_sends_org_nodes_messages_in_order_to_every_recipient_and_writes_no_store (changed; verifies: LLR-pxqdy4, REQ-u894ty) — failed with send_organisation sending in reverse order; reverted
  red -> green: org-io/tests/send_fanout.rs::loopback_refuses_no_address_and_no_address_of_the_recipient_per_recipient (changed) — failed with only the first recipient sent; reverted
  red -> green: org-io/tests/send_fanout.rs::loopback_dials_no_kept_address_outside_the_lower_case_hex_form (changed) — failed with parse_kept_address accepting upper-case hex; reverted
  red -> green: org-io/tests/send_fanout.rs::networked_dials_the_recipients_key_with_its_kept_addresses_as_a_hint (changed) — failed with only the first recipient sent; reverted
  red -> green: org-io/tests/send_fanout.rs::every_send_is_from_the_sending_personas_endpoint (changed) — failed with send_organisation reporting Delivered and sending nothing; reverted
  red -> green: org-io/tests/send_fanout.rs::the_verdict_byte_maps_to_delivered_refused_already_received_invite_used_or_not_delivered (changed) — failed with send_organisation reporting Delivered and sending nothing; reverted
  red -> green: org-io/tests/send_fanout.rs::each_recipient_is_dialled_at_the_addresses_kept_for_its_key (changed) — failed with only the first recipient sent; reverted
  red -> green: org-io/tests/expectations_met.rs::each_recipient_receives_the_body_org_node_built_for_the_adopted_epoch (changed) — failed with Revocation messages reported Delivered and not sent; reverted
  red -> green: org-io/tests/expectations_met.rs::the_last_other_member_removed_is_still_sent_its_revocation_by_send_organisation (changed) — same mutation, same failure; reverted
  red -> green: org-io/tests/requests.rs::request_admission_binds_builds_with_its_own_addresses_and_sends_to_the_issuer (changed) — failed with the admission request carrying no own addresses; reverted
  red -> green: org-io/tests/requests.rs::request_admission_reports_each_outcome_and_writes_nothing (changed) — failed with AlreadyReceived mapped to Delivered; reverted
  red -> green: org-io/tests/requests.rs::request_organisation_asks_every_listed_device_or_the_invites_issuer (changed) — failed with the organisation request carrying no own addresses; reverted
  red -> green: org-io/tests/requests.rs::a_request_whose_endpoint_cannot_be_bound_is_refused_as_a_failed_bind (changed) — failed with request_admission reporting Delivered without sending; reverted
  red -> green: org-io/tests/acknowledgements.rs::a_removed_outcome_starts_one_delivery_task_and_returns_without_awaiting_it (changed; verifies: LLR-zzsqj4) — failed with start()'s one-task guard removed; reverted
  red -> green: org-io/tests/acknowledgements.rs::acknowledgements_go_from_the_removed_personas_own_identity_over_a_direct_bind (changed) — failed with the round reporting Delivered without dialling; reverted
  red -> green: org-io/tests/acknowledgements.rs::accepted_acknowledgements_are_deleted_and_written_until_none_is_left (new; verifies: LLR-6ux6yw, LLR-n649th) — failed under the same mutation; reverted
  red -> green: org-io/tests/acknowledgements.rs::no_acknowledgement_left_deletes_the_persona_at_once (changed; verifies: LLR-n649th, LLR-6ux6yw) — failed under the same mutation; reverted
  red -> green: org-io/tests/acknowledgements.rs::the_scripted_recipients_verdicts_are_the_real_receive_steps (new; verifies: LLR-xwz3nu, LLR-6ux6yw, LLR-n649th) — failed under the same mutation; reverted
  red -> green: org-io/tests/acknowledgements.rs::already_received_or_invite_used_keeps_the_acknowledgement (new; verifies: LLR-xwz3nu, LLR-3w22wc) — failed with AlreadyReceived mapped to Delivered (the round did not move on); reverted
  red -> green: org-io/tests/acknowledgements.rs::a_recipient_with_no_address_of_its_own_is_not_delivered_for_no_address (new; verifies: LLR-3w22wc) — failed with Loopback ignoring the address identity; reverted
  red -> green: org-io/tests/acknowledgements.rs::a_clock_before_the_epoch_is_the_rounds_failure_and_nothing_is_sent_or_deleted (changed; verifies: LLR-bv6wkz, LLR-rav7dr) — failed with next_round not cleared (Some(T+60)); reverted
  red -> green: org-io/tests/acknowledgements.rs::acknowledgement_report_lists_each_kept_recipients_outcome_next_round_and_deadline (changed; verifies: LLR-rav7dr) — failed with the report resuming the deliveries, guard removed; reverted
  red -> green: org-io/tests/acknowledgement_inbound.rs::a_revoked_device_sending_anything_but_an_acknowledgement_or_an_organisation_request_is_refused (changed; verifies: LLR-j467e9, LLR-hhgdk7) — failed with every refusal reported as MalformedMessage; reverted
  red -> green: org-io/tests/verification.rs::a_test_clock_replaces_the_handles_clock_under_test_support_only (changed; verifies: LLR-6vtbn6) — failed with the cfg removed from the test_support impl OrgIo; reverted
  red -> green: org-io/tests/chain_read_rule.rs::the_state_reader_is_named_only_in_verify_and_preflight (changed; verifies: LLR-k3zmqb) — failed with a .read_state( call planted in connect.rs; reverted
  red -> green: org-io/tests/app_boundary.rs::the_app_calls_no_adopting_operation_and_builds_no_service (changed; verifies: LLR-ntag3g) — failed with `use org_node::{service::OrgService}` planted in an app sub-module; reverted
  red -> green: org-io/tests/absences.rs::org_io_hands_out_no_bare_service (re-annotated; verifies: LLR-n2srhc) — failed with a public OrgService field planted; reverted
  red -> green: org-io/tests/absences.rs::every_returned_type_is_in_the_handle_return_allowlist_and_holds_no_secret (changed; verifies: LLR-gh4rw3) — failed with a tuple ChainReadError(OrgPrivateKey) planted; reverted
  red -> green: org-io/tests/absences.rs::org_io_names_no_person_crate (changed; verifies: LLR-b97usg) — failed with `package = "person"` planted in Cargo.toml; reverted
  red -> green: org-io/tests/persona_creation.rs::organisations_report_their_stored_name_and_no_send_carries_it (changed; verifies: LLR-pn3hgc) — failed with a `planted(&self, organisation: &str)` method; reverted
  red -> green: org-io/tests/person.rs::a_second_person_is_refused_naming_the_one_held_and_writes_nothing (changed; verifies: LLR-3cysjk) — failed with propose_person drawing before its check; reverted
  red -> green: org-io/tests/chain_write.rs::each_write_refusal_displays_human_readable_text (changed; verifies: LLR-5tpdxw) — failed with Stale showing NotHeld's text; reverted
  red -> green: org-io/tests/rng_source.rs::a_failing_draw_inside_org_node_panics_before_any_store_change (new; verifies: LLR-94nn6q) — failed with issue_invite catching the panic; reverted
  red -> green: org-io/tests/open.rs::open_creates_the_data_dir_and_opens_the_store_inside_it (changed) — failed with open loading no store; reverted
  changed, assertions added, not watched fail: chain_write.rs::not_held_no_proxy_and_stale_are_refused_before_any_chain_call and ::a_stale_provisional_organisation_is_refused_before_any_write (store bytes unchanged; no plausible mutation writes before these refusals); verification.rs::the_secrets_closure_reads_the_keychain_only_to_sign_and_a_refusal_is_verified_keychain (no delivery starts)
  inherited: org-io/tests/absences.rs::the_handle_returns_only_allowlisted_public_types (re-pointed from superseded LLR-3zdw8v/7pj5af/9r2bxd to LLR-gh4rw3, LLR-4usfu2); org-io/tests/acknowledgement_inbound.rs::a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted (annotation extended with LLR-zzsqj4, LLR-6ux6yw)
```

#### Guideline reviews — round 6 (2026-10-29, head 8b6d1bc6, after the master merge of 641bec1) — Round 6 fix — CODE org-node, plus TEST org-node lows (task `n6`, merged 2026-10-29)

```
  red -> green: org-node/tests/own_removal.rs::into_removed_keeps_only_the_org_id_the_time_and_the_acknowledgements (changed; verifies: LLR-2d5a7m) — failed with RECORD_KEY, and then [1;32], appended to the removed bytes; reverted
  red -> green: org-node/tests/proposals.rs::a_revocation_refused_by_the_bound_records_nothing (changed) — failed with an extra rng draw in record_revocation, before the store lookup (the refusal drew) and after the seal (28 != 24); reverted
  red -> green: org-node/tests/proposals.rs::record_admission_and_record_revocation_refuse_while_behind (changed) — failed with the same extra draw; reverted
  red -> green: org-node/tests/proposals.rs::record_revocation_refuses_an_unheld_or_already_named_member (changed) — failed with the same extra draw; reverted
  red -> green: org-node/tests/proposals.rs::revoking_a_member_the_record_does_not_hold_records_nothing (changed) — failed with the same extra draw; reverted
  inherited: absences.rs::org_node_holds_no_invitation_exchange (REQ-qn2erx dropped; its LLRs stay); absences.rs::org_node_writes_nothing_to_the_chain (REQ-xs4ab8 → REQ-dhr58q, REQ-4qh2h8); absences.rs::org_node_has_no_administrator_key (LLR-xq9nrq, LLR-g76zqd → LLR-q4puca); commit_workflow_errors.rs, the sender-refusals test at :60 (LLR-2r2fha, LLR-kzgjz8 → LLR-283x6f); encoding_golden.rs::revocation_wire_message_is_pinned (LLR-ayrdr8, ms8njy, js9dsu → LLR-mkvx2k, g3wmen, xr88a8); fuzz_wire_decode (REQ-c29s93 → REQ-zxgd46); persona_records.rs::a_valid_record_snapshot_decodes_with_every_field_parsed (LLR-8bum44 → LLR-7yu2xy); wire_frame_bound.rs::the_three_kinds_round_trip_with_their_indices and ::malformed_revocations_and_acknowledgements_are_refused (LLR-js9dsu → LLR-xr88a8)
```

#### Guideline reviews — round 7 (2026-10-29, head 1ab33e5) — Round 7 fix — C7-N1, by owner ruling: one targeted fix (task `n7`, merged 2026-10-29)

```
  red -> green: org-node/tests/verification.rs::a_founding_store_adopts_its_founding_at_epoch_one_with_its_proxy (changed: the expected proxy is now support::the_proxy(), with stores::founding recording it; verifies: LLR-gcm7bq) — failed with stores::founding recording ChainAccount::new([7;32]) again; reverted
  inherited: none
```

#### Guideline reviews — round 8 (2026-10-29, head bf0f2dc) — Round 8 fix — CODE org-node (task `n8`, merged 2026-10-29)

```
  red -> green: org-node/tests/organisation_request.rs::a_listed_requester_is_answered_with_the_full_record_and_its_key (changed: RECORD_KEY is checked by support::assert_not_rendered in every rendering form, a superset of the old two) — failed with OrgPrivateKey's Debug rendering its bytes; reverted
  inherited: none
```

#### T62 — The webview proposes, writes and cancels; notices outlive the tab (merge review MR1-A1, MR1-A2) — T62 — merge review round 1, app (merged 2026-10-30)

```
  red -> green: app/src-tauri/tests/startup_policy.rs::an_interval_that_is_not_utf8_is_refused_not_defaulted (new; verifies: LLR-rvrc25) — failed to compile before policy.rs had verification_interval_from
  red -> green: app/tests/epoch.commands.test.ts, all 13 tests (new; verifies: LLR-dt8ak9, LLR-5vgpkd, LLR-tbtrb3, LLR-e7bbm5, LLR-b59vcx) — 13 of 13 failed before the implementation
  red -> green: app/tests/notices.page.test.ts, all 8 tests (new; verifies: LLR-cgtav9, LLR-9r6y7p) — 8 of 8 failed before the implementation
  red -> green: app/src-tauri/tests/verification.rs::an_own_adoption_is_sent_once_and_nothing_else_is (changed; verifies: LLR-es49wp) — failed when the report marked each Device already received; reverted
  red -> green: app/src-tauri/tests/verification.rs::a_behind_result_is_requested_once_with_no_user_step (changed; verifies: LLR-nkg8sb) — failed with request_organisation called twice (2 != 1); reverted
  red -> green: app/src-tauri/tests/periodic.rs::a_behind_organisation_is_requested_at_each_run_until_in_step (changed; verifies: LLR-fe7umz) — failed with the request reported refused; reverted
  not red-first: app/src-tauri/tests/ipc.rs::the_epoch_commands_answers_have_the_fakes_shapes and ::the_send_and_request_reports_have_the_fakes_shapes (new) — fixture contract checks over backend behaviour that already existed; they passed once their backend.json entries were added
  inherited: none
```

#### T62 — The webview proposes, writes and cancels; notices outlive the tab (merge review MR1-A1, MR1-A2) — m2 — merge review round 1, org-node (merged 2026-10-30; owner ruling: retire and delete)

```
  red -> green: org-node/tests/receive_path.rs::the_organisation_and_the_sequence_number_are_judged_before_the_change_set_is_decoded (new; verifies: REQ-gju89b, REQ-6yu72z, REQ-f2k4tr, LLR-4fbuy8, LLR-ybn5pr, LLR-xpbkp5, LLR-wx3php, LLR-fuq379) — failed with decoding before the org check, with decoding before the seq check, with `>=`, with StaleSeq's fields swapped, and with from_last_seen ignoring its argument; each reverted
  red -> green: org-node/tests/receive_path.rs::organisation_information_from_a_listed_device_is_kept_as_received_with_its_derived_key (re-pointed; adds REQ-f2k4tr, LLR-fuq379, LLR-wx3php) — failed with check accepting only seq > mark+1; reverted
  red -> green: org-node/tests/receive_path.rs::the_chain_free_checks_refuse_with_no_chain_state (re-pointed; adds REQ-f2k4tr, LLR-fuq379) — failed with the base-root check removed; reverted
  red -> green: org-node/tests/value_types.rs::rejects_equal_and_lower_seq (moved; verifies: REQ-6yu72z, LLR-wx3php) — failed with `>=`, and with the fields swapped; reverted
  red -> green: org-node/tests/value_types.rs::from_last_seen_starts_the_guard_at_the_given_mark (moved; verifies: REQ-mr5abb, LLR-duwz79) — failed with from_last_seen ignoring its argument; reverted
  red -> green: org-node/tests/value_types.rs::check_does_not_advance_the_mark (moved; verifies: REQ-mr5abb, LLR-f5kq88) — failed with from_last_seen ignoring its argument; a check that moves the mark does not build; reverted
  red -> green: org-node/tests/absences.rs::verification_revocation_and_removal_do_no_io_and_read_no_clock (scan widened; verifies: LLR-g2jeqd) — red with the old scan once accepted_notice became pub(crate); after widening, failed with a planted pub(crate) fn taking `&mut StoreData`; reverted
  red -> green: org-node/tests/secret_redaction.rs::organisation_key_custody::a_provisional_private_key_leaves_the_store_only_inside_a_message_to_a_listed_device (scan widened) — red with the old scan; after widening, failed with a planted pub(crate) fn returning an OrgPrivateKey; reverted
  inherited: org-io/tests/transport_handshake.rs::delivers_and_verifies_admit_over_iroh and org-io/tests/transport_networked.rs::delivers_and_verifies_admit_over_relay_by_id — step 3 rewritten because its callee was deleted; verifies: LLR-v873fx unchanged; both pass
```

#### T62 — The webview proposes, writes and cancels; notices outlive the tab (merge review MR1-A1, MR1-A2) — m3 — merge review round 1: org-io lows, org-io and on-chain-client record findings (merged 2026-10-30)

```
  red -> green: org-io/tests/acknowledgements.rs::a_deletion_retried_after_its_store_is_dropped_reports_the_kept_deadline (new; verifies: LLR-rav7dr) — failed with deadline 0 (expected 8104800)
  red -> green: org-io/tests/acknowledgements.rs::a_deletion_failed_at_open_after_the_rename_reports_the_kept_deadline (new; verifies: LLR-rav7dr) — failed with deadline 0 (expected 10104800)
  red -> green: org-io/tests/acknowledgements.rs::a_kept_leftover_beside_a_store_keeping_no_removal_reports_no_deadline (new; verifies: LLR-rav7dr) — failed to compile (E0277: u64 compared with Option) before the deadline became Option<u64>
  red -> green: org-io/tests/acknowledgements.rs::an_acceptance_whose_write_failed_is_that_rounds_outcome_in_the_report (new; verifies: LLR-rav7dr, LLR-6ux6yw) — failed with the outcome NotYetSent; then strengthened to match AcceptedNotRecorded { reason: AcceptanceFailure::Store(_) }
  changed for the type only (Some(...)), verifies: unchanged: acknowledgements.rs::acknowledgement_report_lists_each_kept_recipients_outcome_next_round_and_deadline, ::a_removal_open_found_is_reported_and_resumed_though_the_directory_cannot_be_listed; verification.rs (line 215)
  inherited: none
```

#### Merge review — round 2 (merge-change step 4, 2026-10-30, commit 29370c0) — a6 — merge review round 2, app lows (merged 2026-10-30)

```
  red -> green: app/tests/epoch.commands.test.ts::a second proposal at a later epoch replaces the first: the offer shows the second's epoch, not the first's (new; verifies: LLR-tbtrb3) — failed with fileProposal keeping the held entry; reverted
  red -> green: app/src-tauri/tests/ipc.rs::an_unknown_write_has_the_fakes_shape (new; verifies: LLR-tbtrb3, LLR-k4rhbs) — failed "backend.json holds no answers write_organisation_to_chain.unknown"; green once the entry was added
  red -> green: app/tests/epoch.commands.test.ts::a write of unknown outcome says so, never that the epoch is written, and keeps the offer (new; verifies: LLR-tbtrb3) — failed with the message changed to "Epoch written.", and with afterWrite withdrawing the offer on unknown; reverted
  red -> green: app/tests/epoch.commands.test.ts::a write still in flight across a tab switch stays guarded, and its outcome is shown on return (new; verifies: LLR-tbtrb3) — failed (the button was re-enabled after remount) before the state was lifted
  red -> green: app/tests/epoch.commands.test.ts::a proposal still in flight across a tab switch is shown on return (new; verifies: LLR-5vgpkd) — failed before the state was lifted
  red -> green: app/tests/epoch.commands.test.ts::a cancel still in flight across a tab switch stays guarded, and its outcome is shown on return (new; verifies: LLR-e7bbm5) — failed (Cancel invite was re-enabled) before the state was lifted
  inherited: none
```

#### Merge review — round 2 (merge-change step 4, 2026-10-30, commit 29370c0) — n9 — merge review round 2, org-node (merged 2026-10-30)

```
  red -> green: org-node/tests/fuzz_check_received (new; verifies: LLR-7u3dg7, REQ-rv4dwb) — failed with a panic planted just after base.apply_delta in keep_information: unconditional, then only for the store with a record, then only for the store with none; each failed on the first input and was reverted. The "no store change on refusal" assertion was not shown red; no plant made a refusal change the store.
  red -> green: org-node/tests/value_types.rs::the_seq_not_epoch_refusal_names_the_next_epoch_after_the_held_record_and_no_chain (new; verifies: LLR-76bhw9) — failed with "parent_seq 4 is not the on-chain epoch 3" before the error.rs change
  inherited: value_types.rs::every_rejection_variant_is_distinct_from_every_other — from REQ-bcxz96 (now REQ-rv4dwb)
  inherited: value_types.rs::a_stale_sequence_rejection_carries_the_offered_number_and_the_mark — from REQ-bcxz96 (now REQ-rv4dwb)
  inherited: receive_chain_reads.rs::an_update_from_an_unlisted_device_is_refused_before_any_chain_state_is_asked_for; admission_sender.rs::a_revocation_reaches_the_administrators_disk; admission_sender.rs::pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_from_a_relay_and_not_adopted_from_a_listed_device; verification.rs::adoption_replaces_the_record_and_reports_own_in_one_sealed_change — REQ-mr5abb added (finding-3)
```

#### Merge review — round 2 (merge-change step 4, 2026-10-30, commit 29370c0) — o6 — merge review round 2, org-io lows (merged 2026-10-30)

```
  red -> green: org-io/tests/acknowledgements.rs::a_deletion_failed_on_resume_past_the_deadline_reports_the_kept_deadline (new; verifies: LLR-rav7dr) — failed with deadline None (expected Some(10304800)) before the resume fix
  inherited: none
```

#### Merge review — round 3 (merge-change step 4, 2026-10-30, commit a10d5e6) — a7 — merge review round 3, app (merged 2026-10-30)

```
  red -> green: app/tests/epoch.commands.test.ts::a second press while the first proposal is in flight proposes nothing more, the button disabled (new; verifies: LLR-5vgpkd) — failed (disabled was false) with `epochBusy = orgId;` removed; reverted
  red -> green: app/tests/epoch.commands.test.ts::a proposal still in flight across a tab switch stays guarded, and its outcome is shown on return (changed; verifies: LLR-5vgpkd) — the new disabled-on-return assertion failed with `epochBusy = orgId;` removed; reverted
  inherited: none
```

#### Merge review — round 3 (merge-change step 4, 2026-10-30, commit a10d5e6) — n10 — merge review round 3, org-node lows and record findings (merged 2026-10-30)

```
  red -> green: org-node/tests/verification.rs::an_inconsistent_verification_leaves_the_behind_marker_as_it_was (new; verifies: LLR-zk7fx2, LLR-qvp7mq) — failed with the Inconsistent arm set to clear the marker, and with it set to set the marker; reverted
  red -> green: org-node/tests/own_removal.rs::acknowledgement_accepted_for_a_later_recipient_leaves_the_earlier_ones (new; verifies: LLR-85kves) — failed with remove_acknowledgement always removing index 0; reverted
  red -> green: org-node/tests/own_removal.rs::into_removed_refuses_a_store_with_no_record (new; verifies: LLR-2d5a7m) — failed with each arm mutated separately; reverted
  red -> green: org-node/tests/fuzz_check_received (changed; verifies: LLR-7u3dg7, REQ-rv4dwb) — mutant A survived the old target (52 inputs) and fails the new one on input 1
  inherited: none
```

#### Merge review — round 3 (merge-change step 4, 2026-10-30, commit a10d5e6) — n11 — MR3-N1, by the owner's final ruling (merged 2026-10-30)

```
  red -> green: org-node/tests/encoding_golden.rs::persona_store_plaintext_is_pinned_in_its_active_form (changed; verifies: LLR-mkvx2k, LLR-d925uu, LLR-6zaeb6, LLR-p8vpgz, LLR-s3vq6d) — the new pinned value failed to decode (DeserializeBadUtf8) before the field existed
  red -> green: org-node/tests/verification.rs::a_verification_discards_received_organisations_below_the_chain_beyond_the_next_epoch_or_unmatched_at_it (new; verifies: LLR-4tsgfh, RC-d4fc4s) — failed with all seven kept and unstamped; also failed with the rule-3 match check changed to keep
  red -> green: org-node/tests/verification.rs::an_adopting_verification_applies_the_rules_to_what_the_adoption_leaves (new; verifies: LLR-4tsgfh, LLR-x8drb8) — failed with epoch 6 kept and epoch 5 unstamped
  red -> green: org-node/tests/verification.rs::the_next_epochs_information_is_kept_up_to_the_ahead_limit_after_its_stamp_and_discarded_after (new; verifies: LLR-4tsgfh, RC-d4fc4s) — failed with no stamp sealed; later failed with `<=` changed to `<`, and with re-stamping
  red -> green: org-node/tests/verification.rs::two_ahead_limits_give_two_outcomes_for_one_stamp (new; verifies: LLR-4tsgfh) — failed with the body kept under the 600 s limit; later failed with re-stamping
  red -> green: org-node/tests/verification.rs::a_stamped_one_the_chain_reaches_is_judged_by_the_match (new; verifies: LLR-4tsgfh, LLR-8528ud) — failed with both kept at the chain's epoch, unmatched
  red -> green: org-node/tests/verification.rs::a_flood_of_unadoptable_information_delays_an_admission_only_until_the_ahead_limit (new; verifies: LLR-4tsgfh, RC-d4fc4s, REQ-g2537c) — failed with nothing discarded or stamped; later failed with `<=` changed to `<`
  red -> green: org-node/tests/verification.rs::own_proposals_and_provisional_changes_survive_every_rule (new; verifies: LLR-4tsgfh) — passed before the implementation; failed afterwards with the Own exemption removed
  not red-first: org-node/tests/verification.rs::no_chain_state_discards_nothing (new; verifies: LLR-4tsgfh, LLR-qvp7mq) — pins the order of the chain check, which returns before any rule runs
  red -> green: org-node/tests/verification.rs::verify_is_behind_when_no_provisional_organisation_matches_root_and_key, ::a_provisional_organisation_whose_key_is_not_the_chains_is_no_match, ::a_first_admission_whose_record_misses_the_chain_root_adopts_nothing; org-node/tests/admission_sender.rs::pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_from_a_relay_and_not_adopted_from_a_listed_device (changed; each gains LLR-4tsgfh) — their "kept, no change" assertions failed against the ruling; each now asserts the discard, and each also fails with the rule-3 mutant
  inherited: none
```

#### Merge review — round 3 (merge-change step 4, 2026-10-30, commit a10d5e6) — x1 — MR3-N1 rule 4 across org-io and the app (merged 2026-10-30)

```
  red -> green: org-io/tests/verification.rs::the_ahead_limit_the_handle_was_given_is_the_one_verification_uses (new; verifies: LLR-22se2q) — failed to compile first; then, with verify still given the 1200 s constant, failed "verify-ahead-600: discarded a second past the limit"; green once verify took the handle's limit
  red -> green: org-io/tests/verification.rs::a_handle_with_no_chain_holds_no_ahead_limit_and_verifies_nothing (new; verifies: LLR-22se2q) — written after the feature; failed with the unconfigured handle's limit set to Some(1200); reverted
  red -> green: app/src-tauri/tests/startup_policy.rs::the_ahead_limit_is_twice_the_verification_interval (new; verifies: LLR-hgrez9) — failed with verification_ahead_limit stubbed to return the interval (600s against 1200s)
  red -> green: app/src-tauri/tests/verification.rs::the_states_handle_holds_twice_the_verification_interval_as_its_ahead_limit (new; verifies: LLR-hgrez9) — failed against the same stub
  inherited: none
```

#### Merge review — round 4 (merge-change step 4, 2026-10-30, commit c458e96) — a8 — merge review round 4, app (merged 2026-10-30)

```
  red -> green: app/src-tauri/tests/verification.rs::the_chain_settings_init_hands_connect_hold_twice_the_verification_interval_as_their_ahead_limit (new; verifies: LLR-hgrez9) — failed to compile before chain_settings_for_test existed; with the shared function's limit changed to the undoubled interval, this test (1s, expected 2s) and the_states_handle_holds_twice_the_verification_interval_as_its_ahead_limit (600s, expected 1200s) both failed; reverted
  inherited: none
```

#### Merge review — round 4 (merge-change step 4, 2026-10-30, commit c458e96) — n12 — merge review round 4, org-node (merged 2026-10-30)

```
  red -> green: org-node/tests/verification.rs::a_chain_read_behind_the_record_runs_no_rule_and_one_at_the_records_epoch_does (new; verifies: LLR-4tsgfh) — failed ("nothing to seal") before the C < R guard existed
  red -> green: org-node/tests/verification.rs::a_verification_earlier_than_the_stamp_keeps_the_next_epochs_information (new; verifies: LLR-4tsgfh) — the behaviour already existed; failed with plain subtraction in place of saturating_sub (overflow panic); reverted
  red -> green: org-node/tests/verification.rs::a_chain_at_the_last_epoch_keeps_nothing_ahead_and_does_not_overflow (new; verifies: LLR-4tsgfh) — the behaviour already existed; failed with an unchecked `epoch + 1` (overflow panic); reverted
  red -> green: org-node/tests/verification.rs::an_ahead_limit_of_zero_keeps_the_next_epochs_information_only_within_its_stamps_second (new; verifies: LLR-4tsgfh) — the behaviour already existed; failed with `<` in place of `<=`; reverted
  inherited: none
```

#### Merge review — round 4 (merge-change step 4, 2026-10-30, commit c458e96) — o7 — merge review round 4, org-io (merged 2026-10-30)

```
  red -> green: org-io/tests/verification.rs::the_ahead_limit_is_the_durations_whole_seconds_the_fraction_dropped (new; verifies: LLR-22se2q) — failed to compile before the conversion existed; then failed against a rounding mutant (601s, expected 600s); reverted
  red -> green: org-io/tests/verification.rs::a_failed_write_of_a_discard_on_a_behind_verification_is_verified_store_and_writes_nothing (new; verifies: LLR-zcw3xv) — the behaviour already existed; failed against a mutant that skips the write on Behind; reverted
  red -> green: org-io/tests/verification.rs::adopted_and_removed_write_the_returned_store_and_the_others_write_nothing (changed: a discard case added; verifies: LLR-zcw3xv) — failed against the same mutant; reverted
  inherited: none
```

#### Merge review — round 5 (merge-change step 4, 2026-10-30, commit 6ff8483) — r6 — merge review round 5 (merged 2026-10-31)

```
  red -> green: org-node/tests/proposals.rs::a_store_put_back_from_its_file_keeps_the_behind_marker_the_replaced_store_held (new; verifies: LLR-zk7fx2) — failed before the fix (marker None, expected Some(Epoch(4)))
  red -> green: org-io/tests/verification.rs::a_failed_write_of_a_discard_on_a_behind_verification_keeps_the_organisation_behind (new; verifies: LLR-zcw3xv) — failed with the org-node fix removed (MemberNotHeld in place of OrganisationBehind)
  red -> green: app/src-tauri/tests/verification.rs::startup_chain_builds_the_settings_with_twice_the_interval_it_decides (new, under the name init_builds_the_settings_it_connects_with_from_its_environment_with_twice_the_interval_decided; renamed by task r7, 2026-10-31; verifies: LLR-hgrez9) — failed to compile before startup_chain_for_test existed; then failed against a mutant that hands chain_settings the doubled limit (2400s, expected 1200s)
  inherited: none
```

#### Merge review — round 6 (merge-change step 4, 2026-10-31, commit 483800a) — r7 — merge review round 6 (merged 2026-10-31)

```
  red -> green: org-node/tests/own_removal.rs::a_store_put_back_over_a_removed_store_replaces_it_with_no_behind_marker (new; verifies: LLR-zk7fx2) — pins existing behaviour, so it passed first; failed with restore_store changed to keep a held store in the removed form; reverted
  inherited: none
```

## What was wrong, and what was built

What was wrong. The app had no domain surface for the demo journey (found an
Organisation, invite and admit a second user, later revoke them): its commands
drove org-node's old commit path, in which the step that wrote the chain also
replaced the record. Any failure between the chain write and the record's
update, or between the update and the send, left the chain one epoch ahead of
the record and a removal untold to the Device it removed (PR-vt244s). In org-io,
delivery of an executed write hung on the in-memory unknown write that
`reconcile` cleared, so a restart, a failed read-back or store write, or a
refused reconcile left the write's bodies unsent (PR-k5tdxj item 2). Found and
fixed while building the change: a provisional Member's first adoption dropped
the Invite issuer's addresses, so a new Member could not reach the Admin in
Loopback mode (PR-8sgsb5); a timed-out `create_proxy` recorded an unknown write
that verification could never settle, blocking the founding's later writes
(PR-8ghk9s); a removed Persona whose deletion failed at open was never retried
while the handle ran (PR-yng83g); and a leftover `removed-` file beside a live
store of the same key deleted the live Persona's keychain item (PR-wch3zu).

What was built, as the change stands at merge.

- **on-chain-client.** The genesis runs in two resumable steps:
  `create_proxy`, then `found_at`, with `genesis` their composition; `found_at`
  resumes from the chain's own reads (the proxy's free balance and its account
  mapping, read on the writer's seam), so a founding interrupted after the proxy
  exists finishes without creating a second one.
- **org-node (sans-IO).** Each Persona's sealed store holds one Organisation,
  plus a separate sealed Person store. An Organisation advances in batched
  epochs: the Admin's admissions and revocations are kept as provisional
  changes and proposed together as the next epoch. Nothing becomes current on
  receipt: Organisation information and organisation answers are kept as
  provisional organisations under the sender rule and the 1 MiB bound, and
  `verify` judges them against a chain state the caller read — it adopts a
  provisional organisation only when the chain holds its root and key, reports
  `Behind` (setting the behind marker, which refuses new proposals until in
  step) or `Inconsistent`, and never adopts on receipt. Every verification
  applies the discard rules of MR3-N1 to received provisional organisations at
  epoch E against the chain's C: E < C, E > C + 1, and E = C not matching are
  discarded; E = C + 1 is stamped by the first verification that sees it and
  discarded once the caller's ahead limit has passed since the stamp; this
  node's own proposals, the provisional changes and the founding are exempt; a
  chain read behind the record (C < R) is Inconsistent and discards nothing.
  The six Wire kinds carry Organisation information, revocation notices,
  acknowledgements, admission requests, organisation requests and their
  answers; a listed Device asking is answered with the full record, a revoked
  one with its revocation notice alone. org-node decides its own removal,
  seals a removed store form and signs the acknowledgements it keeps until
  each is accepted.
- **org-io.** Writing, verifying and sending are separate operations. The
  handle owns the random source; each founding and each Member is a new
  Persona with its own keychain item, sealed store file and endpoint, and the
  Person has its own. Chain writes (an update, or a founding over
  on-chain-client's two steps) report Included, Refused or Unknown, and an
  unknown write is settled only by verification. Verification reads the
  finalised block before the state, passes the app's ahead limit to org-node,
  writes the store and starts a removal's acknowledgement delivery. The send
  path sends org-node's bodies for the adopted epoch to every recipient at the
  addresses kept per DevicePublicKey. Delivery is pull-driven: a Device behind
  requests the Organisation from its Members, and an admission request goes to
  the Invite's issuer. The receive step keeps and reports with a verdict byte;
  the removal acknowledgements are delivered on a backoff schedule until
  accepted, after which the removed Persona is deleted under the retention
  rules. The handle returns only allowlisted public types that hold no secret.
- **app and webview.** The old commands and `rand` are gone. The commands
  propose a Person, a founding and a Member; issue, read and cancel an Invite;
  request admission; propose admissions and revocations, the next epoch, and
  write it; `verify_organisation` and `verify_and_send` send once for this
  node's own adoption and request when behind; a periodic verification runs at
  the configured interval with an ahead limit of twice that interval. The
  webview shows the verification view, confirms an admission against the kept
  requests, warns before the first admission request, proposes, writes and
  cancels (T62), and keeps discarded-change notices and send outcomes until
  dismissed, across tab switches.

## Review

The merge-change step 4 rounds, each finding verbatim from its reviewer's
report, renumbered within its round (the reviewer's own number is the one the
quoted text and the round's blockquote use), and prefixed with its unit where a
round had several reviewers. The plan's IDs for the findings above low (MR1-A1,
MR1-N1, MR3-N1, …) are named in the dispositions. Owner rulings that decided a
disposition are stated in it.

### Round 1

> Reviewers, one fresh reviewer per unit: org-node, org-io, app and on-chain-client, on commit 82f6a3c. Verdicts:
> - org-node: "no high finding. A medium requirement finding (finding-1: in-force verify-and-commit items traced only to the now-dead `verify_envelope_against_chain`) should be dispositioned before merge. Everything else is low."
> - org-io: "the org-io code meets its claimed requirements and LLRs in the sample; findings 1 and 2 are `code, low` and the rest are `record`, so nothing blocks the merge for org-io."
> - app: "not ready to merge. Medium requirement findings remain (finding-1: no UI path to propose or write an Organisation update, so revocations never reach the chain while the UI says "submitted"; finding-2: discarded-change notices and send outcomes are lost on tab switch and never shown for verifications the user asks for), plus low findings."
> - on-chain-client: "On-chain-client: approve. Every claimed REQ and LLR (REQ-v86pyy, REQ-6vh3ht, REQ-6jefu2; LLR-tge9q8, q6buam, chy957, jk76wf, 95puh9, g8qevz, hea6y2, htp5tk) is met. Each has normal and abnormal tests that fail when the behaviour breaks (every mutation caught). Tests: all passed, plus the bolero targets green. Trace and IDs both exit 0. There are `record` findings and no code or requirement finding, so this unit is ready to merge once the record fixes are applied."
> The finding numbers inside these quotes are each reviewer's own; below, findings are renumbered within the round.

**finding-1**: requirement, medium — org-node: In-force verify-and-commit requirements are now verified only through a function the product no longer calls.
- **The function.** `verify_envelope_against_chain` (org-node/src/verify.rs:73, re-exported at org-node/src/lib.rs:63) had two callers on master, both in service.rs (the old receive and commit paths, master service.rs:708 and :1410). After this change no file under org-node/src, org-io/src or app/src-tauri/src calls it.
- **What still points at it.** org-node/tests/verify_against_chain.rs (24 tests) and tests/fuzz_verify_against_chain still carry `verifies:` for in-force items:
  - REQs: REQ-wp2nyc, REQ-8gz8bu, REQ-mr5abb, REQ-nhe2zu, REQ-txvtm9, REQ-bcxz96, REQ-gju89b, REQ-6yu72z, REQ-ag6kqm, REQ-bvh8v6, REQ-f2k4tr.
  - LLRs: LLR-8n95rf, LLR-vf5mjx, LLR-d6kvbx, LLR-hs7g6j and others, all in org-node/docs/architecture/2026-10-03-decomposition.md and org-node/docs/requirements/2026-09-09-verify-and-commit.md.
- **Why they no longer hold.** Several no longer describe the shipped behaviour:
  - REQ-8gz8bu and LLR-vf5mjx: "reject a received Change set … `StaleEpoch`".
  - REQ-wp2nyc and LLR-8n95rf: "refused with `RootMismatch`".
  - The product now keeps the Change set on receipt and `verify` reports `Behind` or `Inconsistent` (REQ-dwb9bq, LLR-rr558z).
  - For the other items, only the chain-free checks are still shared through `check_chain_free`.
- **Effect.** check-trace stays green because tests of dead code exist. The plan's G2 sweep caught only tests that called *removed* operations, and this function was kept.
- **Fix.** Supersede or retire the items whose behaviour the domain API replaced (pointing the successors at LLR-rr558z or LLR-8528ud). Then either delete `verify_envelope_against_chain` with its target and fuzz target, or re-point the still-valid chain-free items (REQ-gju89b, REQ-6yu72z, REQ-mr5abb) at tests of the receive path.
- **Severity.** Arguably high under "makes a gate pass that should fail". I rate it medium because the safety substance (no adoption of a root or key the chain does not hold) is enforced and tested by the new match rule.
disposition: MR1-N1: owner ruling 2026-10-29: retire and delete; fixed by task `m2`. `verify_envelope_against_chain`, `VerifiedUpdate`, `SeqGuard::advance`, the lib.rs re-export, tests/verify_against_chain.rs and the fuzz_verify_against_chain target are deleted; REQ-wp2nyc, REQ-8gz8bu, REQ-nhe2zu and REQ-txvtm9 are superseded by REQ-dwb9bq, REQ-bvh8v6 and LLR-vf5mjx by LLR-rr558z, REQ-ag6kqm and LLR-8n95rf by LLR-8528ud, and the other replaced LLRs by their receive-path successors. The chain-free items that still hold (REQ-gju89b, REQ-6yu72z, REQ-f2k4tr, REQ-mr5abb and their LLRs) are re-pointed at receive-path and value-type tests such as `the_organisation_and_the_sequence_number_are_judged_before_the_change_set_is_decoded` in `org-node/tests/receive_path.rs`. REQ-bcxz96 and LLR-hs7g6j were first retired here; round 2 (MR2-N1) found that wrong and task `n9` superseded them by REQ-rv4dwb and LLR-7u3dg7.

**finding-2**: requirement, low — org-node: Any listed Device can fill the shared bound with notices, and the store then stays stuck.
- **The rules.** Kept revocation notices are never de-duplicated and are counted in the shared 1 MiB bound. `check_received` appends every notice `check_notice` passes (org-node/src/service.rs:400-406, LLR-r4qtqe). `check_notice` checks only identity (Organisation, MemberId, Device), not the proof, because receipt reads no chain. A refused notice stays kept (LLR-f9mcsq), and notices are cleared only by an adoption (LLR-x8drb8).
- **The attack.** One listed Device can send this Persona notices naming it with bogus proofs until `ProvisionalLimit`.
- **The effect.** From then on, every keep of Organisation information or an answer is refused, and so are `record_admission`, `record_revocation` and `propose_updated_organisation`. With no provisional organisation to match, the store can never adopt again, so it stays behind or stuck permanently.
- **Severity.** Rated low because it needs a malicious or faulty Member. It is a gap in the requirements (LLR-r4qtqe, LLR-f9mcsq and LLR-m76r4k together allow it), not a code defect. Suggest risk analysis consider at least refusing an identical notice already kept, or bounding notices apart from provisional organisations.
disposition: not fixed — opened by task `m2` as PR-ug8q68 (status open, 2026-10-30) in `org-node/docs/problems/2026-10-25-domain-api.md`, for risk analysis to decide whether to refuse identical kept notices or bound notices apart from provisional organisations.

**finding-3**: requirement, low — org-node: Organisation information received in a store with a record keeps the carried `record_snapshot` as `base_snapshot` without checking that it is the record it extends (org-node/src/service.rs:666, as LLR-76bhw9 states).
- **What is checked.** With a record, the base is rebuilt from `record.trie_members` and the carried snapshot is not compared to it.
- **What follows.** After adoption it becomes `kept_base_snapshot`. `messages_for_current_epoch` re-sends it and builds revocation notices from it (service.rs:1105-1110). A wrong snapshot gives wrong or missing notices, or a refusal of the whole send.
- **Why it is low.** It is latent today: org-io and the app send only an own adoption, whose snapshot org-node built itself.
- **Cheap fix.** Check that the carried snapshot rebuilds the record's root, or keep the record's own members instead.
disposition: not fixed — opened by task `m2` as PR-d4r6te (status open, 2026-10-30) in `org-node/docs/problems/2026-10-25-domain-api.md`; the gap is latent while org-io and the app send only an own adoption.

**finding-4**: requirement, low — org-node: REQ-dwb9bq (org-node/docs/requirements/2026-10-14-batched-epoch.md:92) says the software "shall refuse with a typed error, leaving the store unchanged, a state whose epoch is lower than its record's or equal to it with a different Membership root". LLR-rr558z and the code return `Ok(Verified::Inconsistent { reason })` (verify_org.rs:76-87), an outcome rather than an error. The tree is right and the REQ's wording is stale.
disposition: fixed in the ledger by task `m2`: REQ-dwb9bq now says the state is reported inconsistent, naming the reason, matching LLR-rr558z and `org-node/src/verify_org.rs`.

**finding-5**: code, low — org-node: LLR-myfk8w says every operation on a removed store other than the four it lists refuses with `OrganisationRemoved`. Two paths answer otherwise:
- `check_received` makes its size check before `data()`. An oversized body to a removed store is `MessageTooLarge` (service.rs:370-375).
- `proxy_account(org_id)` for a removed Organisation is `OrgNotHeld` (service.rs:733), since `record_index` skips removed stores.

Both change nothing. `persona_public_keys` documents its own exception, but these two do not.
disposition: fixed in the ledger by task `m2`: LLR-myfk8w now documents both exceptions for a removed store, `MessageTooLarge` from the size check in `check_received` and `proxy_account`'s `OrgNotHeld` (`org-node/src/service.rs`); the code is unchanged.

**finding-6**: code, low — org-node: `verify_org::accepted_notice` is `pub` in a `pub mod` (org-node/src/verify_org.rs:104), but only `verify` uses it, and no LLR names it as interface. Make it `pub(crate)` like its siblings `judge` and `matching`.
disposition: fixed by task `m2`: `accepted_notice` in `org-node/src/verify_org.rs` is now `pub(crate)`. The absences and secret-redaction scans were widened to read verify_org.rs's `pub(crate)` functions, `verification_revocation_and_removal_do_no_io_and_read_no_clock` was shown red with a planted function, and LLR-g2jeqd has a dated note.

**finding-7**: code, low — org-io: `org-io/src/acknowledge.rs:236-244`. When the Persona is gone and only a deletion failure is being retried, `report` returns `deadline: 0`. The field's doc (`view.rs:229`, "The kept verification time plus 7 days") and LLR-rav7dr define the deadline as that time. A consumer would read a deadline at the Unix epoch, already passed. The app does not read the field today, and LLR-rav7dr does not specify the value for this case. Fix: either make the field `Option<u64>`, or state the sentinel in LLR-rav7dr and in the doc comment.
disposition: fixed by task `m3`: `AcknowledgementReport::deadline` is now `Option<u64>` and carries the real kept deadline while a deletion failure is retried (`org-io/src/acknowledge.rs`); LLR-rav7dr is amended. Tests `a_deletion_retried_after_its_store_is_dropped_reports_the_kept_deadline` and `a_deletion_failed_at_open_after_the_rename_reports_the_kept_deadline` failed red with deadline 0.

**finding-8**: code, low — org-io: `org-io/src/acknowledge.rs:387-393`. A recipient that answers `Delivered` (accepted) goes through `accepted()` and then `continue`. If org-node refuses that acceptance or its store write fails, `accepted()` returns `None` and no outcome is recorded. `acknowledgement_report` then keeps showing that recipient's previous outcome (for example `NotYetSent`) although the last round reached it and it accepted. LLR-rav7dr says the report gives each kept recipient "its last round's outcome". Nothing is lost: the acknowledgement stays kept and is sent again, as LLR-6ux6yw requires. Only the report is stale, and only after a rare store-write failure.
disposition: fixed by task `m3`: an acceptance refused by org-node or not written is now that round's outcome, `AcknowledgementOutcome::AcceptedNotRecorded { reason: AcceptanceFailure }`, and the acknowledgement stays kept; LLR-rav7dr and LLR-6ux6yw are amended and `an_acceptance_whose_write_failed_is_that_rounds_outcome_in_the_report` verifies it.

**finding-9**: record — org-io: `org-io/docs/problems/2026-10-24-domain-api.md`, PR-wch3zu has three problems:
- it is `status: resolved` but has no `resolution:` field, unlike PR-8ghk9s and PR-yng83g in the same file; its resolution is a free paragraph headed "Resolution (2026-10-25, plan T60)";
- that paragraph ends "LLR-upqj5g's text … does not yet state the skip", which is stale, because LLR-upqj5g was amended in place on 2026-10-25 to state it;
- it is a third item this change resolves in org-io that the merge brief does not list (the brief names PR-8ghk9s, PR-yng83g and PR-k5tdxj item 2).

Its fix and test hold: `retention.rs:81-100` and `a_removed_file_beside_a_live_store_of_its_key_is_kept_and_reported`, which I ran and which passes.
disposition: fixed by task `m3`: PR-wch3zu in `org-io/docs/problems/2026-10-24-domain-api.md` gains its `resolution:` line (fixed in 4c25e49) and a dated note on its stale prose; this record lists it among the items the change resolves (`docs/verification/2026-10-31-worktree-org-io-domain-api.md`).

**finding-10**: record — org-io: `docs/plans/2026-10-21-org-io-domain-api.md:20`. The plan header's org-io verification line still lists `--test node_stories`. T60/C4-O11 deleted that test file and target (plan line ~41214). `org-io/.guardrails/config.yaml:59` and `Makefile:608` are correct, so only the header disagrees with the diff.
disposition: fixed by the dispatcher: the org-io verification line in the header of `docs/plans/2026-10-21-org-io-domain-api.md` no longer lists `--test node_stories`, matching `org-io/.guardrails/config.yaml`.

**finding-11**: record — org-io: `check-trace.sh` for org-io exits 0 but still prints three advisories: `UNMET-EXPECTATION org-node: REQ-tzr3dk / REQ-jxyh9c / REQ-txkm94`. All three items carry `superseded-by:` lines naming this change's successors (REQ-2yw5y3, REQ-u894ty, REQ-2xgsbn). None of the successors is reported unmet, so the advisories are for out-of-force expectations. The checker counts them anyway, and they reach the "open 23 days" age display. This does not block the merge. The record should say these three are superseded rather than open.
disposition: fixed in the ledger by task `m3`: no org-io `expects:` names those REQs; the advisories come from the superseded items' own `expects:` lines, and dated notes on REQ-tzr3dk, REQ-jxyh9c and REQ-txkm94 say they are superseded, not open. The check-trace tooling gap is PR-bbu24f (status open, opened by task `o6`) in `org-io/docs/problems/2026-10-24-domain-api.md`; this record states it (`docs/verification/2026-10-31-worktree-org-io-domain-api.md`).

**finding-12**: requirement, medium — app: The webview offers no way to propose the updated Organisation, to write an Organisation update, or to cancel an invite. As a result, an Admin's admission or revocation can never reach the chain from the app, and the screen claims it is on its way.
- **Where:**
  - `app/src/lib/api.ts` has no wrapper for `propose_updated_organisation` or `cancel_invite`.
  - `writeOrganisationToChain` is called only as `{ founder }` (`app/src/lib/components/PersonaList.svelte` `writeFounding`). Nothing under `app/src` or `app/tests` names `propose_updated_organisation`, `cancel_invite` or a write by `orgId`.
- **Why it matters:**
  - REQ-vb5pag requires the app to decide "from the user's commands … when to propose the updated Organisation and when to write it to the chain". SDD-6g3wnh's amendment lists "the Svelte panels for … the proposal and the write" among the realised call sites.
  - The backend commands exist and are tested, but no user command reaches them. The demo journey's admit and revoke steps therefore stop at the provisional change.
- **What the user is told:**
  - Admit says "Admission proposed: it goes with the next epoch you write." (`app/src/lib/components/Admit.svelte:102`), but there is nothing to write it with.
  - Revoke says "Revocation submitted. Waiting for the remote peer to receive + self-delete…" (`app/src/lib/components/Revoke.svelte:111`), but no epoch carrying the revocation can be proposed or written from the UI. A revoked Device is not removed, while the Admin is told it is in progress.
- **Severity:** I rate this medium (a stated requirement not met, and a misleading message in normal use). The revocation half touches a safety control, so it should not merge as is.
- **Root cause:** the plan has no webview task for these wrappers or panels (grep of the plan finds none), so the fix also needs a plan and record entry.
disposition: MR1-A1: owner ruling 2026-10-29: add the screens now; fixed by task `T62`. New SDD-p7sdy3 with LLR-dt8ak9 (the wrappers `proposeUpdatedOrganisation` and `cancelInvite` in `app/src/lib/api.ts`), LLR-5vgpkd ("Propose next epoch"), LLR-tbtrb3 ("Write to chain" for a proposed update, held in `app/src/routes/+page.svelte`), LLR-e7bbm5 ("Cancel invite") and LLR-b59vcx (Admit and Revoke say the change is proposed and written only with the next epoch; "Revocation submitted" is gone), verified red-first by app/tests/epoch.commands.test.ts. The round 2 app reviewer found it fixed.

**finding-13**: requirement, medium — app: The notices for discarded changes, and each Device's send outcome, are held only by the S4 component. They are fed only from `verification-result` events received while S4 is mounted.
- **Where:**
  - `app/src/lib/components/Membership.svelte:80-95` holds `notices` and `lastSends` as component state.
  - Its only input is `[onVerificationResult, recordVerification]` (line 234).
  - The page mounts Membership only while tab S4 is active (`app/src/routes/+page.svelte:136`).
- **Consequences:**
  1. Switching to any other tab destroys every undismissed notice. This contradicts REQ-pe2c7u and RC-8h344c ("until the Admin dismisses it") and LLR-k5rf3r ("keeps every notice until `dismissNotice` removes that one").
  2. An `adopted` entry returned by the at-mount or "Verify with chain" verification (`verifyOrganisations`), or by a write (`WriteDto.verification`), goes only to the page's `results` list (`fileVerificationResults`). It never reaches `applyDiscarded` or `sendOutcomes`. Its discarded changes therefore get no notice, and its per-Device send outcomes are not shown. This contradicts REQ-pe2c7u and REQ-qv4d2z/RC-c5tpre ("shows the Admin each Device's outcome").
  3. Events emitted while another tab is open are not received by Membership at all.
- **Tests:** they test `applyDiscarded` and `dismissNotice` as pure functions (`app/tests/notices.discarded.test.ts:85`), so none fails.
- **Suggested fix:** lift the notices and send outcomes to the page, as was done for `results` and `admissions`, and file every verification entry through them.
disposition: MR1-A2: fixed by task `T62`: notices and per-Device send reports now live in `app/src/routes/+page.svelte` (new LLR-cgtav9 and LLR-9r6y7p under SDD-xg4nj7), and one `fileResults` files every source: the verification at mount, the button, an included write and the periodic event; Membership no longer subscribes. Verified red-first by app/tests/notices.page.test.ts; the round 2 app reviewer found it fixed.

**finding-14**: code, low — app: The tests of the "exactly once" clauses would still pass if the send or the request were made twice.
- **LLR-es49wp:** `an_own_adoption_is_sent_once_and_nothing_else_is` asserts only `sends.len() == 1` and the device (`app/src-tauri/tests/verification.rs:512`).
  - `attach_report` replaces `sends` on each call, so a second send leaves one entry either way.
  - The test never asserts `sends[0].result == "sent"`.
- **LLR-nkg8sb:** `a_behind_result_is_requested_once_with_no_user_step` (line 629) has the same gap.
  - Its fixture gives the same outcome as the "abnormal" test at line 828 (the member refuses: `failed`, "refused by the recipient").
  - So no test shows a request that ends `sent`.
- **LLR-fe7umz:** the periodic test asserts only that `requests` is an array at each run (`app/src-tauri/tests/periodic.rs`, last test).
- **Severity:** low, because the code makes a single call structurally (`make_follow_up`, `app/src-tauri/src/commands.rs:639-645`) and the gap is in test strength.
- **Suggested fix:** assert the delivery result, or count the recipient's received messages.
disposition: partly fixed by task `T62`: `a_behind_result_is_requested_once_with_no_user_step` and the periodic test now pin one request ending `sent` (LLR-nkg8sb, LLR-fe7umz), and `an_own_adoption_is_sent_once_and_nothing_else_is` asserts the result. A doubled send of an own adoption still cannot be seen at the app's interface, because org-io reports a repeated send as delivered; it is open as PR-2t6ssm in `app/docs/problems/2026-10-30-domain-api.md` (LLR-es49wp).

**finding-15**: code, low — app: A whole refusal of the send or the request is never shown to the user.
- **Where:**
  - `sends_refused` and `requests_refused` appear only in the type declarations in `app/src/lib/api.ts:101,104`.
  - Membership shows "Sent to each Device (…)" with an empty list when the send was refused as a whole (`recordVerification`, `Membership.svelte:91-95`). The Admin sees no outcome and no reason.
- **Severity:** low, because the state is rare (Organisation not held, or endpoint not bound right after a successful verification).
disposition: fixed by task `T62`: a send or request refused as a whole is now shown with its reason (`app/src/lib/api.ts` types, rendered at page level under LLR-9r6y7p).

**finding-16**: code, low — app: A non-UTF-8 `ODS_VERIFICATION_INTERVAL_SECS` is silently replaced by the 10-minute default instead of refusing to start.
- **Where:** `app/src-tauri/src/state.rs:123` uses `std::env::var(..).ok()`, which maps `VarError::NotUnicode` to `None`.
- **Why it matters:** REQ-ergn9z says the app "shall refuse to start … when the configured value does not parse".
- **Severity:** low (a contrived environment). The same pattern already exists for `ODS_DATA_DIR`.
disposition: fixed by task `T62`: a non-UTF-8 `ODS_VERIFICATION_INTERVAL_SECS` is refused at start through `verification_interval_from` in `app/src-tauri/src/policy.rs`; LLR-rvrc25 is amended and `an_interval_that_is_not_utf8_is_refused_not_defaulted` verifies it red-first.

**finding-17**: requirement, low — app: Two webview behaviours are carried by tests but stated by no LLR (derived work, not marked).
1. `foundingWrites` (`app/src/lib/verification.ts:99`) stops offering "Write to chain" for a founder whose newest entry is `adopted` or `in_step`.
   - LLR-rhc555 says where the founder key comes from, not which foundings are offered.
   - The tests verifying LLR-rhc555 assert the filter.
2. `reproposalStep` (`app/src/lib/notices.ts:138-158`) returns `unavailable`, which means no "Propose again", when no kept request matches the discarded admission.
   - LLR-md6hhz (as amended) does not state this case.
   - If org-node drops the kept request once its admission is recorded, this path is the normal one for a discarded admission, and REQ-pe2c7u's "offer to propose the change again" is not met for admissions.
- **Suggested fix:** state both in LLR-rhc555 and LLR-md6hhz, or mark them derived. Confirm against org-node whether a discarded admission's request is still kept.
disposition: fixed by task `T62`: LLR-rhc555 now states the founding-offer stop rule and LLR-md6hhz states the `unavailable` step. Checked against org-node: a kept request survives a recorded admission, so "Propose again" for a discarded admission takes the normal path.

**finding-18**: record — on-chain-client: `on-chain-client/docs/architecture/2026-09-28-decomposition.md:51-58`. This change removed REQ-2qa5r5 from SDD-5wamsz's `traces:` and added a dated amendment. The next paragraph still says "(Note, 2026-10-05: REQ-2qa5r5 … It stays in the trace above as history, beside its replacement.)", which is now false. The tree is right: REQ-54txzh is traced. Remedy: append a few words to the 2026-10-05 note, or to the 2026-10-20 amendment, saying the 2026-10-20 amendment removed it.
disposition: fixed by task `m3`: SDD-5wamsz's 2026-10-05 note in `on-chain-client/docs/architecture/2026-09-28-decomposition.md` now says REQ-2qa5r5 left the trace on 2026-10-20.

**finding-19**: record — on-chain-client: `on-chain-client/docs/problems/2026-09-10-on-chain-client-problems.md:118, 166, 217, 265`. The four dispositions written for the owner ruling of 2026-10-28 give as their premise that this change "touches no on-chain-client file it concerns". That was true when the ruling was recorded. Plan line 40806 says "This branch changes no on-chain-client file", but the later C4-A1 sweep (plan line 41146) then edited two of the files these reports cite:
- `src/verify.rs`: its whole header was rewritten. PR-h4mb8y's "Where" paragraph quotes the old header verbatim ("Lands in Stage 2 Task 5 … placeholder"), and that text no longer exists. The new header no longer claims, in the present tense, a verification it does not perform, which is the report's own "what is not acceptable" condition. So the sweep did part of PR-h4mb8y's fix outside `resolve-problem`.
- `src/client.rs`: one doc line was removed near the top. Every line citation in PR-w5sk5k, PR-uq5r97 and PR-qpp28h past line ~20 is now off by one; for example, `:111-113` is now 110-112.

The defects themselves stand; the code was not changed. check-trace already prints "Check that the ruling in disposition: still stands". Remedy:
- Ask the owner to re-confirm the ruling on its corrected premise.
- Add a dated note to PR-h4mb8y recording that its quoted header was replaced in this change and that the module still holds no code.
- Fix the disposition text "touches no on-chain-client file it concerns".

No `code` or `requirement` findings.
disposition: owner ruling 2026-10-29: the ruling on the stale on-chain-client PRs is re-confirmed on the corrected premise; fixed by task `m3` in `on-chain-client/docs/problems/2026-09-10-on-chain-client-problems.md`. The dispositions of PR-w5sk5k, PR-h4mb8y, PR-uq5r97 and PR-qpp28h now say "re-confirmed 2026-10-29 on the corrected premise", PR-h4mb8y has a dated note that its quoted header was replaced and the module still holds no code, and the line citations of PR-w5sk5k, PR-uq5r97 and PR-qpp28h are corrected with dated notes.

### Round 2

> Reviewers, one fresh reviewer per unit: app, org-node and org-io, on commit 29370c0 (on-chain-client had record fixes only and got no reviewer). Verdicts:
> - app: "fit to merge. Every round 1 medium finding is fixed. This round has `code` or `requirement` findings, all low, and a `record` finding, so by the checklist this is the last round." The app reviewer re-checked MR1-A1 and MR1-A2 and found them fixed.
> - org-node: "not yet fit to merge. Medium findings, finding-1 (requirement: hostile-apply no-panic retired with its evidence) and finding-2 (code: misleading SeqNotEpoch message), need dispositions; the rest are low or record."
> - org-io: "the round-1 org-io lows are fixed and hold. All findings this round are low (`code` and `requirement`), so nothing blocks the merge for org-io and this can be the last round for the unit."
> The finding numbers inside these quotes are each reviewer's own; below, findings are renumbered within the round.

**finding-1**: code, low — app: Nothing tests that a second proposal of an Organisation *replaces* the first.
- **Test:** `app/tests/epoch.commands.test.ts:170-176` ("a second proposal of the same Organisation replaces the first offer").
- **Why it misses:** both proposals return the same `PROPOSED` (epoch 2), and the test counts only the "Write to chain" buttons.
- **Mutant that survives:** `fileProposal` returning `held` unchanged when the Organisation is already held (`app/src/lib/epoch.ts:24-25`). It survived. That mutant keeps the stale epoch shown beside "Write to chain".
- **Clause not verified:** LLR-tbtrb3's "the latest proposal of an Organisation replacing an earlier one".
- **Fix:** answer a second epoch (for example 3) and assert "epoch 3 proposed for …".
disposition: fixed by task `a6`: a new test in app/tests/epoch.commands.test.ts answers a second proposal at a later epoch and asserts the offer shows it (LLR-tbtrb3); the reviewer's surviving `fileProposal` mutant in `app/src/lib/epoch.ts` now fails it.

**finding-2**: code, low — app: LLR-tbtrb3's `unknown` outcome has no test and cannot be produced by the fake.
- **What is untested:** the `unknown` message ("Write outcome unknown: <reason>") and the rule that the offer is kept on `unknown`.
- **Why the fake cannot produce it:** `app/tests/contract/backend.json` has no `write_organisation_to_chain` variant with `"result": "unknown"`. `result` is a tag, so the fake refuses that shape.
- **Mutants that survive:**
  - Rewriting the message (`epoch.ts:51-52`) to "Epoch written." survived.
  - Withdrawing the offer on `unknown` in `afterWrite` would survive too.
- **Why it matters:** this is the one outcome where the Admin must not be told the epoch is on the chain.
- **Fix:** add an `unknown` entry with its ipc.rs producer and a rendered test.
disposition: fixed by task `a6`: backend.json gains `write_organisation_to_chain.unknown`, produced over org-io's FakeChain by `an_unknown_write_has_the_fakes_shape` (LLR-tbtrb3, LLR-k4rhbs), and a rendered test shows "Write outcome unknown" and keeps the offer; both reviewer mutants now fail it.

**finding-3**: code, low — app: The outcome of a proposal, write or cancel still in flight when the Admin switches tab is never shown, and the busy guard is lost with it.
- **Where:** the message and the busy flag are component state: `epochMessage` and `epochBusy` (`Membership.svelte:107-141`), and `cancelMessage` and `cancelBusy` (`Admit.svelte:24-41`).
- **What happens:** a chain write takes block time. If the Admin leaves S4 meanwhile:
  - a `refused` or `unknown` outcome is written into a destroyed component and never shown;
  - a re-mounted panel shows the offer enabled again.
- **What is not lost:**
  - An included write's verification and the offer withdrawal go through page-level callbacks.
  - A repeated write is refused by org-io (`OutcomeUnknown` or `Stale`).
  - So no state is lost or corrupted; only LLR-tbtrb3's "shows the outcome" is missed.
- **Fix:** lift `epochMessage` and the in-flight flag to the page beside `proposedUpdates`.
disposition: fixed by task `a6`: the epoch and cancel messages and busy guards moved to `app/src/routes/+page.svelte`, so an outcome landing after a tab switch is shown and its guard kept (LLR-tbtrb3, LLR-5vgpkd, LLR-e7bbm5); fake-tauri gains `defer`. This structural change was included in round 3's review scope.

**finding-4**: code, low — app: The comment added to the send test states a mechanism that does not exist.
- **Where:** `app/src-tauri/tests/verification.rs:523-524`.
- **The comment says:** "a second send of the same epoch would be answered 'already received'".
- **What org-io does:** it answers a repeated Organisation information `Checked::AlreadyHeld` with `Verdict::Accepted` (`org-io/src/receiver.rs:217`), which the app reports as `sent`.
- **Consequence:** the new `result == "sent"` assertion cannot detect a doubled send. That matches T62's own record ("a doubled *send* … cannot be seen at the app's interface"). The comment is wrong, not the record.
- **Fix:** correct the comment so a later reader does not take finding-3's send half as closed.
disposition: fixed by task `a6`: the comment in `app/src-tauri/tests/verification.rs` now says org-io answers a repeated send `Accepted`; the undetectable doubled send is open as PR-2t6ssm (LLR-es49wp).

**finding-5**: requirement, low — app: "Cancel invite" is traced to a requirement that does not state it.
- **The items:** LLR-e7bbm5 and SDD-p7sdy3's "cancelling an outstanding Invite" clause both trace to REQ-vb5pag.
- **What REQ-vb5pag covers:** only when to propose (a Person, a founding, a Member, an admission, a revocation, the updated Organisation) and when to write. No app requirement mentions cancelling.
- **How the backend half is handled:** LLR-vqz4s6 is `satisfies: derived` and risk-assessed (`app/docs/risk/2026-10-15-domain-api.md:230-233`, `assesses: … LLR-vqz4s6`).
- **Result:** the webview half is unmarked derived work with no assessment.
- **Fix:** mark LLR-e7bbm5 `satisfies: derived` and add it to that assessment, or amend REQ-vb5pag to name cancelling.
- **Severity:** low, because the tree's behaviour is right and the assessment's conclusion carries over.
disposition: fixed by task `a6`: LLR-e7bbm5 is now `satisfies: derived` and is assessed in `app/docs/risk/2026-10-15-domain-api.md` beside LLR-vqz4s6; no new hazard was found.

**finding-6**: record — app: Two documentation gaps:
1. `withdrawAdmissionRequest` (`app/src/lib/notices.ts:206-212`) serves LLR-e7bbm5, but no SDD file list names it.
   - SDD-p7sdy3 lists `epoch.ts`, the two wrappers and the page.
   - SDD-xg4nj7's amendment adds only `applySendReports` and `dismissSendReport`.
2. `ipc.rs::the_epoch_commands_answers_have_the_fakes_shapes` (line 1825) annotates `verifies: LLR-dt8ak9`.
   - That LLR is the TypeScript wrappers' argument passing, which a Rust shape check cannot exercise.
   - epoch.commands.test.ts verifies it properly.
   - The record already lists this test as "not red-first"; the annotation should drop LLR-dt8ak9.
disposition: fixed by task `a6`: SDD-p7sdy3's file list in `app/docs/architecture/2026-10-16-domain-api.md` names `withdrawAdmissionRequest`, and `app/src-tauri/tests/ipc.rs` no longer cites LLR-dt8ak9.

**finding-7**: requirement, medium — org-node: The retirement of REQ-bcxz96 and LLR-hs7g6j removes the only requirement and the only evidence that a hostile Change set cannot panic the node once it is *applied*. The code path is still live.
- The deleted fuzz_verify_against_chain shape 3 perturbed an honest delta, so decodable deltas with the right base root reached `apply_delta` and the root computation.
- That same work now runs on receipt, at org-node/src/service.rs:627-638 (`keep_information`: `check_chain_free`, then `base.apply_delta(&delta)` and `candidate.verify_against(&root)`).
- It is reachable by any Device the record lists. In a store with no record, `check_without_record` (service.rs:534-566) accepts it "from whichever Device sent it" for an expected Organisation.
- RC-e2uvje is in force: "no sequence of envelope bytes … causes a panic in decoding or in verification". HAZ-5f9jcm names a hostile Change set panicking the node. Only REQ-9g6as6 is left, and it bounds *decoding* alone. The remaining fuzz targets (fuzz_envelope_decode, fuzz_wire_decode, fuzz_first_admission_base) never reach `apply_delta`.
- The retirement reason, "no operation now verifies an Envelope against a record and a chain state", is literally true. But it misses that the hazardous step, applying attacker bytes to the record, moved into receipt rather than going away.
- org-members' proptests (org-members/tests/fuzz_tests.rs) apply only structurally generated deltas, not byte-perturbed ones.
- Remedy: supersede rather than retire. A successor REQ/LLR would state that `check_received` neither panics nor keeps anything on any Organisation-information body. Re-target the deleted shapes 2 and 3 at `check_received` from a listed Device and from an unlisted one into a provisional Member's store.
- I did not find a panic in `apply_delta` by reading it. The gap is the lost requirement and its evidence for a class C hazard control.
disposition: MR2-N1: fixed by task `n9`: REQ-bcxz96 and LLR-hs7g6j are now superseded by new REQ-rv4dwb (`implements: RC-e2uvje`, derived, assessed with HAZ-5f9jcm's rating unchanged) and LLR-7u3dg7 under SDD-busuj3, not retired. New bolero target `org-node/tests/fuzz_check_received/fuzz_target.rs` drives arbitrary bodies and the deleted shapes into `check_received` for a store with a record and one without, asserting no panic and an unchanged store on refusal; it is on the org-node verify line and in rust.yml.

**finding-8**: code, medium — org-node: `OrgNodeError::SeqNotEpoch`'s doc and Display still describe the deleted chain check, but the variant is now produced on receipt with no chain state.
- org-node/src/error.rs:23-27 reads: doc "not the epoch of the Organisation state it was verified against (REQ-txvtm9)", Display `"parent_seq {seq} is not the on-chain epoch {epoch}"`.
- `keep_information` (service.rs:629-631) raises it with `epoch: record.epoch + 1`, the expected next epoch derived from the local record. No chain was read.
- The app shows `to_string()` as a `VerifyFailed` message (app/src-tauri/src/events.rs:116-118).
- In normal use, a Member who missed one epoch and then receives the next epoch's Organisation information is told "parent_seq 4 is not the on-chain epoch 3". That misstates both the cause (it is behind, not facing a chain contradiction) and where the 3 came from. Nothing is lost; verification reports Behind and the request path recovers.
- The doc also cites REQ-txvtm9, which m2 superseded.
- The module header at error.rs:1-2 ("Every rejection path in verify-against-chain") is stale too.
disposition: MR2-N2: fixed test-first by task `n9`: `SeqNotEpoch`'s Display in `org-node/src/error.rs` now reads "the Change set's parent sequence {seq} is not {epoch}, the next epoch after the held record", its doc cites LLR-76bhw9, and the module header is corrected; `the_seq_not_epoch_refusal_names_the_next_epoch_after_the_held_record_and_no_chain` verifies it.

**finding-9**: requirement, low — org-node: REQ-mr5abb is kept, and its 2026-10-30 note re-reads "the mark" as the current record's Sequence number, set only by an adoption. Its annotated tests do not exercise that.
- The annotated tests are org-node/tests/value_types.rs:296 `from_last_seen_starts_the_guard_at_the_given_mark` and :307 `check_does_not_advance_the_mark`. They exercise `SeqGuard` alone.
- If `keep_information` or a refused `verify` moved `record.last_seq`, neither would fail.
- The behaviour is guarded by tests annotated for other IDs: receive_chain_reads.rs:86, admission_sender.rs:322 and :467, and verification.rs:458 assert `last_seq` unchanged on a keep or Behind, and set to the epoch on adoption.
- Remedy: add `REQ-mr5abb` to those tests' `verifies:`, or to one new test asserting the mark is unchanged across a keep and a refused verify and equals the epoch after adoption.
disposition: fixed by task `n9`: REQ-mr5abb is added to the `verifies:` of the four tests that assert `last_seq`, among them `adoption_replaces_the_record_and_reports_own_in_one_sealed_change` in org-node/tests/verification.rs.

**finding-10**: requirement, low — org-node: m2 superseded parent REQs without re-parenting their in-force children. Several in-force LLRs now `satisfies:` only out-of-force REQs; this is the LLR-level twin of the checklist's RC rule, and no gate reports it:
- LLR-pzde8b and LLR-e7s4ye satisfy only REQ-ag6kqm (decomposition.md:531, :442).
- LLR-cja9zv satisfies only REQ-nhe2zu and REQ-txvtm9 (decomposition.md:2346). It is still verified by verification.rs:1069.
- LLR-7en38k satisfies only REQ-bvh8v6 (domain-api.md:943). It was minted in this change, and absences.rs tests it.

Superseding REQ-bvh8v6 and REQ-ag6kqm by *LLRs* (LLR-rr558z, LLR-8528ud) also leaves no REQ stating two behaviours:
- "no chain state → refuse, nothing changed".
- "the decision uses no signature and no delivering Device".

REQ-dwb9bq's text has no `None` clause, although its 2026-10-30 note claims RC-6a2dke's "unknown Organisation is rejected" half through LLR-rr558z. The behaviour is stated at LLR level and tested (verification.rs:212), so the tree is right.

Remedy: re-point each LLR above at an in-force REQ (REQ-dwb9bq, REQ-9g6as6 or REQ-f2k4tr as fits). Either give REQ-dwb9bq the no-state clause, or supersede REQ-bvh8v6 and REQ-ag6kqm with a REQ.
disposition: fixed by task `n9`: REQ-dwb9bq is amended to refuse with a typed error when there is no Organisation state, and to check no signature and never consult the delivering Device; LLR-pzde8b, LLR-e7s4ye, LLR-cja9zv and LLR-7en38k are re-pointed to in-force REQs.

**finding-11**: code, low — org-node: `encode_record_snapshot` maps a postcard encode failure to `OrgNodeError::Chain(...)` (org-node/src/service.rs:1428), whose Display reads "chain read failed: …". No chain is involved. It needs an allocation failure, so it is rare.
disposition: not fixed — opened by task `n9` as PR-xkp5bv (status open) in `org-node/docs/problems/2026-10-25-domain-api.md`: an encode failure in `encode_record_snapshot` is reported as `Chain`.

**finding-12**: record — org-node: A test still claims a retired item: org-node/tests/value_types.rs:53 and :250 carry `verifies: REQ-9g6as6, REQ-bcxz96, …`. Drop REQ-bcxz96 from both.
disposition: fixed by task `n9`: the two tests in org-node/tests/value_types.rs cite REQ-rv4dwb in place of the superseded REQ-bcxz96.

**finding-13**: record — org-node: Two items still describe the deleted function or method:
- SDD-d8ktxa (decomposition.md:561) still says "the separation of asking from committing that lets the decisive root check run between the two", with no dated note. `SeqGuard::advance` is deleted, and the note sits only on SDD-na9nc3.
- LLR-fuq379's narrowing note says its tests are "now receive-path tests (`receive_path.rs`)". That is right, but its item text keeps the struck clause naming `verify_envelope_against_chain` unmarked inline, so a reader must reach the note.
disposition: fixed by task `n9`: SDD-d8ktxa has a dated note, and LLR-fuq379's struck clause naming `verify_envelope_against_chain` is marked inline.

**finding-14**: record — org-node: The `traces:` lines of SDD-msb6xh, SDD-rq6nv4 and SDD-ueh4tm now name only superseded REQs (nhe2zu, txvtm9, bvh8v6). The checklist allows a `traces:` line to name history. These three SDDs have no in-force REQ left in their trace, though, so check whether that was intended.
disposition: fixed by task `n9`: SDD-msb6xh, SDD-rq6nv4 and SDD-ueh4tm are re-traced to in-force REQs (recorded in `docs/plans/2026-10-21-org-io-domain-api.md` and in `docs/verification/2026-10-31-worktree-org-io-domain-api.md`).

**finding-15**: requirement, low — org-io: `org-io/docs/architecture/2026-10-16-domain-api.md:2104-2147` against `org-io/src/view.rs:150-154`.

- The Boundaries 2 exception lists every public type that carries org-node's `OrgNodeError`, and round 4 amended it to add three: `Verified::Refused`, `DeletionFailure::Listing` and `RoundFailure::Listing`.
- m3 adds a fourth public carrier, `AcceptanceFailure::Refused(Cause<OrgNodeError>)`, reachable as `AcknowledgementOutcome::AcceptedNotRecorded { reason }` in the public report. The exception does not name it.
- The tree is right: it is a refusal org-node returned, which is the exception's own criterion. But the enumerated list, which reviewers and the A2 tripwire rely on, is now incomplete.
- Fix: a dated amendment adding `view::AcceptanceFailure::Refused` to the exception.
disposition: fixed by task `o6`: the Boundaries 2 exception in `org-io/docs/architecture/2026-10-16-domain-api.md` names `view::AcceptanceFailure::Refused`, with a dated amendment; `AcceptanceFailure::Store` holds org-io's own error and is outside it, and the doc of `Refused` is corrected.

**finding-16**: code, low — org-io: `org-io/src/acknowledge.rs:209-227` with `retention.rs:271-281`.

- `AcknowledgementReport::deadline`'s doc (`view.rs:263-268`), LLR-rav7dr's 2026-10-30 amendment and the m3 record all say the deadline is `None` *only* for a leftover beside a live store keeping no removal.
- There is one more path. `open` holds a removal whose deadline has not passed, so `delete_expired_at_open` records nothing. The clock then passes the deadline before `start_receiving`. Next:
  - `resume` takes the `Expired` arm and calls `retry_deletion`. No `deliver` runs, so `progress.deadline` is never set.
  - If that deletion then fails after the rename, the store is dropped.
  - The report then gives `deadline: None` for a removal that had one.
- The state is contrived: the deadline has to fall between open and start, and then a deletion step has to fail. Nothing is lost.
- Fix: record the deadline in `resume`'s `Expired` arm, which has `kept.verified_at`, or in `retry_deletion`. Add a test with the clock set past the deadline between `open_with_clock_for_test` and `start_receiving`, using `refuse_deletes_as`.
disposition: fixed test-first by task `o6`: resume's `Expired` arm records the deadline before `retry_deletion`; `a_deletion_failed_on_resume_past_the_deadline_reports_the_kept_deadline` failed red with deadline None (LLR-rav7dr).

**finding-17**: code, low — org-io: `org-io/src/test_support.rs:528-531, 554-558, 606-610` (`refuse_deletes_as`), against TEST_GUIDELINES Doubles 3 and 5.

- Its comment calls it a fault injector (Doubles 5), but with `Refusal::Locked` it returns `keyring::Error::NoStorageAccess`. That is service-specific content which `keychain.rs::refusal` interprets and maps to `KeychainError::Locked`. Doubles 5 puts such a double on the fake side, which needs a contract test.
- Its sibling `refuse_reads_as` was treated that way: it is pinned against a store that is locked after a write by `keychain_contract.rs::the_in_memory_keychain_refusing_reads_answers_as_a_store_locked_after_a_write` (LLR-36jer8).
- `refuse_deletes_as` imitates a state with no such test: reads answer while deletes are refused as locked. Nothing shows a real keychain enters that state; a locked macOS keychain refuses reads too.
- Low, because its one user (`a_deletion_failed_at_open_after_the_rename_reports_the_kept_deadline`) asserts only that the keychain step failed, not which error.
- Fix, either:
  - use `Refusal::Unavailable` (`PlatformFailure`, an uninterpreted OS failure) and keep the fault-injector label; or
  - add a contract clause under LLR-36jer8 the way `refuse_reads_as` has one.
disposition: fixed by task `o6`: the one user of `refuse_deletes_as` (`a_deletion_failed_at_open_after_the_rename_reports_the_kept_deadline`) now passes `Refusal::Unavailable`, so it is a fault injector, and its doc in `org-io/src/test_support.rs` says `Locked` has no contract test.

**finding-18**: requirement, low — org-io: LLR-rav7dr (`org-io/docs/architecture/2026-10-16-domain-api.md:1700-1716`) and the `OrgIo::acknowledgement_report` doc (`org-io/src/lib.rs:767-770`).

- Both still say the report is "`None` for an Organisation no removed Persona holds".
- A `removed-` leftover beside a live, never-removed Persona's store returns `Some(report)` with `LeftoverKept` and no deadline. `a_kept_leftover_beside_a_store_keeping_no_removal_reports_no_deadline` asserts this (the admin is not removed), and LLR-upqj5g and the 2026-10-30 amendment describe it.
- The main clause was not amended to carve the case out, so the item now contradicts its own amendment. The tree is right.
- Fix: add the `LeftoverKept` case to the `None` clause's exceptions, in LLR-rav7dr and in the method's doc.
disposition: fixed by task `o6`: LLR-rav7dr's main clause carves out a kept `removed-` leftover beside a live store (dated note), and the doc of `acknowledgement_report` in `org-io/src/lib.rs` states both exceptions to `None`.

### Round 3

> Reviewer: a fresh reviewer per affected scope, covering org-node in full and the app changes made since round 2 (task `a6`); report mr3-org-node-app.md.
> Commit reviewed: a10d5e6.
> Verdict, verbatim: "the app's a6 changes are fit to merge, and MR2-N1 and MR2-N2 hold. org-node is not yet on its last round: finding-1 (requirement, medium) needs a fix or an owner ruling."

**finding-1**: requirement, medium — RC-d4fc4s's clause "a provisional organisation that can never be adopted is discarded after verification" is not met. As a result, any Device can block a provisional Member's admission for good.
- **Where:**
  - In a store with no record, `check_without_record` (service.rs:571-611) keeps Organisation information for the expected Organisation "from whichever Device" (LLR-e6w3nr). It passes `SequenceNumber::new(0)` and has no epoch check.
  - `verify` → `judge` (verify_org.rs:72-85) returns `Behind` and discards nothing.
  - LLR-x8drb8 discards provisional organisations only at an adoption, and a Member that is never admitted never reaches one.
- **The state:** a hostile Device that knows the Organisation identifier sends distinct bodies until REQ-g2537c's 1 MiB bound is reached. Every later keep is then refused with `ProvisionalLimit`, including the genuine Organisation information and the organisation answer (`keep_organisation_answer` keeps through the same bound). The Member is never admitted.
- The same holds in a store with a record for bodies from a listed Device: a `Behind` verification discards nothing that can no longer be adopted.
- **The risk file understates it.** RC-d4fc4s's residual risk (risk/2026-10-15-domain-api.md:62-79) assesses only "a listed Member's ability to make another Device store up to the bound". It does not cover an unlisted Device filling a provisional Member's store, or the blocked admission.
- **Why medium:** a stated risk control is unmet in a state an adversary reaches on purpose; it does not need a rare or contrived state.
- Raised by the receive-slice helper as low. I confirmed it in the code and raised the severity. It is the same class as PR-ug8q68.
- **Fix:** at a verification that adopts nothing, discard provisional organisations at or below the chain's epoch; or re-assess HAZ-cuj9gn under risk analysis and amend RC-d4fc4s.
disposition: MR3-N1, fixed by owner ruling. Owner ruling 2026-10-30 first said to accept only the issuer; before task `n10` made any edit the owner revised it: Organisation information is accepted from any Device, and every verification, adopting or not, discards a received provisional organisation at epoch E against the chain's epoch C when E < C, E > C+1, E = C unmatched, or (rule 4) E = C+1 more than a time limit after the first verification that saw it stamped it; own proposals and the provisional changes are exempt. On rule 4 the owner first clarified that the limit is decided by a fresh chain read, then decided (owner ruling 2026-10-30) that the limit is a parameter of `verify` and that the app passes twice its verification interval, so no scheduled re-check is needed. Task `n11` implemented the rules in org-node (`verify_org::discard_unadoptable`, `AheadLimit`, the `first_ahead_at` stamp; LLR-4tsgfh minted, RC-d4fc4s, HAZ-cuj9gn and the residual-risk text in `org-node/docs/risk/2026-10-15-domain-api.md` amended), pinned by `a_flood_of_unadoptable_information_delays_an_admission_only_until_the_ahead_limit`; task `x1` carried the limit through org-io (LLR-22se2q) and the app (LLR-hgrez9), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-2**: code, low — `fuzz_check_received`'s evidence for REQ-rv4dwb is thin.
- **Few inputs:**
  - About 50 inputs per gated run, each giving 8 offers.
  - The target has no corpus directory. The deleted `fuzz_verify_against_chain` had a `corpus/`.
  - Shape 3 has about 152 × 256 single-byte variants of the honest Change set. Each run samples about 50 per store, around 0.03%, with a new seed each time and nothing kept.
- **Reach of the perturbation:** the offset is `usize::from(bytes[0])`, so it never exceeds 255 (fuzz_target.rs:99-107). Shape 4 therefore never reaches the last 181 bytes of the 437-byte body, or the last 67 bytes of the 323-byte body. That includes the carried Organisation private key and most of the record snapshot. The header's claim that shape 4 reaches "the snapshot decode and the key decode too" (fuzz_target.rs:21-22) holds only in part.
- **Behind marker:** the store-unchanged assertion does not see it (mutant A above).
- **Answer path:** neither store has the behind marker set, so the organisation answer's snapshot decode is never reached through `check_received`. `fuzz_first_admission_base` covers that function directly.
- **What does hold:** n9 showed the no-panic half red three ways, and I showed the store-unchanged half red with mutant B.
- **Fix:** take the offset from two bytes; add a deterministic sweep or a seeded corpus over shape 3; compare the behind marker too; or raise `BOLERO_RANDOM_ITERATIONS` for this target.
disposition: Fixed in task `n10`: the perturbation offset in `org-node/tests/fuzz_check_received/fuzz_target.rs` is now a u16 taken modulo the body length, so it reaches every byte; the store comparison includes the behind marker, and the header is corrected. Mutant A (a refusal that sets the behind marker) survived the old target and fails the new one on its first input; the iteration rate is unchanged and stated in the header (verifies REQ-rv4dwb, LLR-7u3dg7).

**finding-3**: code, low — Two tests inherited from REQ-bcxz96 claim REQ-rv4dwb but cannot fail if receipt panics or a refusal changes the store.
- The tests are `every_rejection_variant_is_distinct_from_every_other` (org-node/tests/value_types.rs:53) and `a_stale_sequence_rejection_carries_the_offered_number_and_the_mark` (value_types.rs:264). Neither calls `check_received`.
- **Fix:** drop REQ-rv4dwb from their `verifies:`, or record that they cover only the "typed error" clause.
disposition: Fixed in task `n10`: `every_rejection_variant_is_distinct_from_every_other` and `a_stale_sequence_rejection_carries_the_offered_number_and_the_mark` in `org-node/tests/value_types.rs` no longer claim REQ-rv4dwb in their `verifies:`.

**finding-4**: code, low — The LLR-85kves test cannot tell when the wrong recipient's acknowledgement is removed.
- Mutant M1 made `RemovedStore::remove_acknowledgement` (removal.rs, about line 140) remove index 0 whenever any recipient matches. It survived.
- `own_removal.rs:633` accepts 0x21 and then 0x22, and each is at index 0 when accepted.
- **Fix:** have the test accept the second kept recipient first.
disposition: Fixed in task `n10`: the new test `acknowledgement_accepted_for_a_later_recipient_leaves_the_earlier_ones` (`org-node/tests/own_removal.rs`, verifies LLR-85kves) accepts the recipient at index 1 and keeps index 0's acknowledgement; it failed with `remove_acknowledgement` always removing index 0.

**finding-5**: code, low — No test shows that an `Inconsistent` verification leaves the behind marker as it was (LLR-zk7fx2, LLR-qvp7mq; `update_behind_marker`, service.rs:1400).
- verification.rs:292 compares only the encoded bytes, which leave out the marker.
- Its `Inconsistent` cases run after a `Behind` has already set the marker.
- This mutant was not run.
disposition: Fixed in task `n10`: the new test `an_inconsistent_verification_leaves_the_behind_marker_as_it_was` (`org-node/tests/verification.rs`, verifies LLR-zk7fx2, LLR-qvp7mq) failed with the Inconsistent arm set to clear the marker and with it set to set the marker.

**finding-6**: code, low — `Acknowledgement::signed_bytes` (revocation.rs) falls back to the domain bytes alone if `postcard::to_allocvec` fails. A signature made then would cover no Organisation, Member or epoch.
- postcard cannot fail on these fixed-shape types, so the state is contrived.
- Returning a `Result` would remove the fallback.
disposition: not fixed — removing the fallback changes a `pub` signature of `org-node/src/revocation.rs`, so task `n10` opened it as PR-y6ay44 rather than change the interface in the merge review.

**finding-7**: record — LLR-2d5a7m (architecture/2026-10-16-domain-api.md:1001) gives `into_removed(...) -> RemovedStore`.
- The `pub` function returns `Result` and refuses with `OrgNotHeld` or `OrgNotOnChain`. No item states these refusals and no test covers them.
- `verify`, its only caller (pinned by absences.rs:636), cannot reach them.
- **Fix:** state the refusals, or make the function `pub(crate)`.
disposition: Fixed in task `n10`: LLR-2d5a7m now states `into_removed`'s `Result` and its two refusals, `OrgNotHeld` and `OrgNotOnChain`, both unreachable through `verify`, and `into_removed_refuses_a_store_with_no_record` (`org-node/tests/own_removal.rs`) covers them, failing with each arm mutated separately.

**finding-8**: record — Doc comments in org-node/src still cite IDs this change superseded. The trace gate does not read these sites.
- error.rs:
  - LLR-n67aw8 nine times (:95-131; successor LLR-3sddey, confirmed).
  - LLR-mxskg9, LLR-s8xp7m and LLR-3f5h7b (:58, :63).
  - LLR-j5vbqj (:85, :90).
  - LLR-hby4jr (:135).
  - LLR-2r2fha and LLR-kzgjz8 (:140).
  - LLR-3aysup (:145).
- LLR-8bum44 at error.rs:53, and at removal.rs:62, a line this change added.
- envelope.rs:2 cites REQ-ag6kqm; revocation.rs:58 cites REQ-y99c9w; types.rs:56 cites LLR-3fwykc.
- `OrgNotHeld`'s doc (error.rs:94-95) still says "A reconcile for an Organisation…", but reconcile is deleted.
disposition: Fixed in task `n10`: a sweep of every superseded ID re-pointed each one cited in org-node/src (including `org-node/src/error.rs` and `org-node/src/removal.rs`) to its successor, such as LLR-3sddey for LLR-n67aw8, and also caught LLR-38e2kn and LLR-pt32fx, which the reviewer had not listed; "reconcile" is gone from the docs.

**finding-9**: record — org-node/docs/CONTEXT.md is stale against this change:
- The intro says org-node commits "by reading the chain".
- **Wire message** says it is "of one of two kinds"; there are now six.
- **Organisation private key** says it is kept at receipt only if it matches the chain's key.
- **Expected admission** says a non-matching admission is "refused before the chain is read".
- **Sequence number** cites the superseded REQ-txvtm9.
disposition: Fixed in task `n10`: `org-node/docs/CONTEXT.md` is refreshed in its intro and its Wire message, Sequence number, Organisation private key and Expected admission entries.

**finding-10**: record — Three amended items keep text that their dated notes say no longer holds:
- SDD-d8ktxa (decomposition.md:573): the separation of asking from committing.
- SDD-ueh4tm (:1389): the boundary is "expressed as a trait".
- LLR-cja9zv (:2402): "the store is saved before the outcome is returned".
disposition: Fixed in task `n10`: the statements of SDD-d8ktxa, SDD-ueh4tm and LLR-cja9zv in `org-node/docs/architecture/2026-10-16-domain-api.md` are reconciled with their dated notes.

**finding-11**: code, low — app: nothing tests the in-flight guard of "Propose next epoch".
- The mutant removed `epochBusy = orgId;` from `proposeNextEpoch` (app/src/routes/+page.svelte, the page-level function a6 added). All 110 vitest tests still pass.
- LLR-5vgpkd says the command is invoked "once".
- The write's and the cancel's guards are pinned across a tab switch; the proposal's is not.
disposition: Fixed in task `a7`: a rendered test in `app/tests/epoch.commands.test.ts` ("a second press while the first proposal is in flight proposes nothing more, the button disabled", verifies LLR-5vgpkd) failed with `epochBusy = orgId;` removed; because jsdom delivers Svelte 5's delegated click to a disabled button, `proposeNextEpoch` in `app/src/routes/+page.svelte` also refuses at the function, and the tab-switch test asserts the remounted button is disabled while the proposal is in flight.

**finding-12**: record — app: a6 adds two behaviours whose tests cite LLR-tbtrb3, LLR-5vgpkd and LLR-e7bbm5, but no LLR text states them. SDD-p7sdy3's dated note calls them "unchanged in substance".
- The in-flight guard outlives the panel.
- An outcome that lands after a tab switch is shown on return.
- **Fix:** add a sentence to each LLR.
disposition: Fixed in task `a7`: LLR-5vgpkd, LLR-tbtrb3 and LLR-e7bbm5 each state, in a dated note in `app/docs/architecture/2026-10-16-domain-api.md`, the in-flight guard that outlives the panel and the outcome shown on return after a tab switch, and SDD-p7sdy3's round 2 note is corrected.

### Round 4

> Reviewer: a fresh reviewer over the round's fix tasks (`n10`, `n11`, `x1`, `a7`), covering org-node in full and the org-io and app changes since a10d5e6; report mr4.md.
> Commit reviewed: c458e96.
> Verdict, verbatim: "not yet the last round. org-node's code implements the owner's final MR3-N1 ruling exactly and the golden bytes are right, but finding-1 (requirement, medium: org-io's REQ-36gxmg and LLR-zcw3xv still say InStep, Behind and Inconsistent write nothing) needs an in-place amendment and one failed-write test; everything else is low or record."

**finding-1**: requirement, medium — org-io's exported REQ-36gxmg still says that at the current epoch, at a later unmatched epoch and when inconsistent the software shall "change nothing" (org-io/docs/requirements/2026-10-14-domain-api.md:93, :104, :106). LLR-zcw3xv still says "on `Behind`, `InStep` and `Inconsistent` it writes nothing, so the store file's bytes are unchanged" (org-io/docs/architecture/2026-10-16-domain-api.md:974).
- The tree now does otherwise. `judge` writes any store change org-node returns (org-io/src/verify.rs:301-305), and LLR-4tsgfh's discard or stamp returns one on those three outcomes.
- x1's own test reads the discard back from disk after an `InStep` (org-io/tests/verification.rs, `the_ahead_limit_the_handle_was_given_is_the_one_verification_uses`).
- org-node's REQ-dwb9bq was amended for this exception; the consumer-side items were not. LLR-22se2q `satisfies: REQ-36gxmg`, a requirement its behaviour contradicts.
- `adopted_and_removed_write_the_returned_store_and_the_others_write_nothing` (verification.rs:435) still passes only because its store holds no received provisional organisation.
- One behavioural consequence is stated nowhere and untested. When the write of a Behind/InStep discard fails, the outcome is `Verified::Store`, not `Behind`, so the app makes no organisation request that round.
- Fix: amend REQ-36gxmg and LLR-zcw3xv in place to carry LLR-4tsgfh's exception, and add one abnormal test of a failed discard write.
disposition: MR4-O1, fixed in task `o7`: REQ-36gxmg (`org-io/docs/requirements/2026-10-14-domain-api.md`) and LLR-zcw3xv are amended in place so that in step, behind and inconsistent change nothing but what verification's discard rules discarded or stamped, and a failed write of that change reports `Verified::Store` with the Persona reloaded from its unchanged file. The new test `a_failed_write_of_a_discard_on_a_behind_verification_is_verified_store_and_writes_nothing` and a discard case added to `adopted_and_removed_write_the_returned_store_and_the_others_write_nothing` (`org-io/tests/verification.rs`) failed against a mutant that skips the write on Behind.

**finding-2**: code, low — On `Inconsistent::ChainBehindRecord` (chain C < record R, for example from a lagging RPC finalised read), the rules run against a chain state the verification has itself rejected (service.rs:973-976).
- Rule 2 then discards the genuine next-epoch information at R+1 (> C+1) that a listed Admin sent.
- It is recoverable: once the read recovers, `Behind` leads to a request. The state is not normal use.
- `a_verification_discards_…` pins Inconsistent only for `OtherRoot` (C = R).
- Ask the owner to confirm "every verification" covers a chain read behind the record, or skip the rules on `ChainBehindRecord`.
disposition: Fixed in task `n12` by owner ruling 2026-10-30 and the owner's clarification: a current record is never ahead of the on-chain epoch, so a chain read with C < R is stale or faulty; it is reported Inconsistent and the discard rules do not run, so nothing is discarded or stamped, while an Inconsistent read at the record's own epoch with a different root still runs them. LLR-4tsgfh, LLR-rr558z and the RC-d4fc4s note say so, and `a_chain_read_behind_the_record_runs_no_rule_and_one_at_the_records_epoch_does` (`org-node/tests/verification.rs`) failed before the C < R guard existed.

**finding-3**: code, low — LLR-hgrez9's two tests do not reach `init`'s wiring.
- `AppState::for_test_over_every` (app/src-tauri/src/state.rs, test-support) itself calls `with_ahead_limit_for_test(policy::verification_ahead_limit(interval))`.
- So `the_states_handle_holds_twice_the_verification_interval_as_its_ahead_limit` passes if `init` (state.rs:135) hands `connect` the undoubled interval or a constant. Only the pure `verification_ahead_limit` is pinned.
- In the same way, `connect`'s `as_secs()` (org-io/src/lib.rs:653) is untested. The fraction-drop assertion goes through `with_ahead_limit_for_test`'s own copy of the conversion.
- The x1 record admits that neither is observable. One shared conversion/builder used by both `init`/`connect` and the test helpers would make the tests bite.
disposition: Fixed in tasks `a8` (app) and `o7` (org-io): `chain_settings` in `app/src-tauri/src/state.rs` is the one place the interval is doubled, used by `init` and the test constructors and pinned by `the_chain_settings_init_hands_connect_hold_twice_the_verification_interval_as_their_ahead_limit`; org-io's `ahead_limit_from` in `org-io/src/lib.rs` is the one conversion used by `connect` and every test-support constructor, stated in LLR-22se2q and pinned by `the_ahead_limit_is_the_durations_whole_seconds_the_fraction_dropped`.

**finding-4**: requirement, low — LLR-4tsgfh has no abnormal-input cases for:
- a `verified_at` earlier than the stamp (the wall clock stepped back). The code keeps the body via `saturating_sub` (verify_org.rs:164); the LLR is silent, and a plain-subtraction mutant would panic in debug and go unseen;
- a chain epoch at `u64::MAX` (the `checked_add` None arm);
- `AheadLimit(0)`.
disposition: Fixed in task `n12`: LLR-4tsgfh states the abnormal cases and new tests in `org-node/tests/verification.rs` pin them: `a_verification_earlier_than_the_stamp_keeps_the_next_epochs_information` (failed with plain subtraction), `a_chain_at_the_last_epoch_keeps_nothing_ahead_and_does_not_overflow` (failed with an unchecked `epoch + 1`) and `an_ahead_limit_of_zero_keeps_the_next_epochs_information_only_within_its_stamps_second` (failed with `<` for `<=`).

**finding-5**: requirement, low — The residual-risk amendment (org-node/docs/risk/2026-10-15-domain-api.md, after RC-d4fc4s) is mostly honest, but overstates the bound and the rating.
- It does say a flooder delays "for as long as it keeps sending". But "cannot block it for good" holds only if the flooder stops.
- Any Device that knows the Organisation identifier can refill the 1 MiB bound with C+1 bodies once per ahead limit. That blocks a new Member's admission indefinitely, at trivial cost.
- It restates S1/P2 as "unchanged", though the initiating population widened from listed Members to any Device. n11's record says the rating was not re-analysed and is for the owner to confirm. The risk file carries no such caveat, and no owner confirmation is recorded.
- "Live at most the ahead limit plus one verification interval" is measured from the stamp, not from arrival.
- Fix: record the owner's confirmation of the rating, or re-analyse it, and state the sustained-flood case plainly.
disposition: Fixed in task `n12` by owner ruling 2026-10-30: the residual-risk text under RC-d4fc4s in `org-node/docs/risk/2026-10-15-domain-api.md` now states that any Device that knows the Organisation identifier can refill the bound once per ahead limit and so, by sustaining the flood, hold off an admission indefinitely, that each verification clears a flood, and that a next-epoch body lives at most the ahead limit from its stamp plus one interval. The owner kept S1/P2 (a delayed admission, with no wrong record, data loss or key exposure) and accepted the residual, recorded as "Residual accepted by the owner, 2026-10-30"; the logging part of that acceptance was withdrawn in round 5 (MR5-N1).

**finding-6**: record — HAZ-cuj9gn's own statement still reads "from any Device the current record lists … a malicious Member". The "not limited to a listed Device" amendment sits below RC-d4fc4s, so `find-items.sh show HAZ-cuj9gn` shows the stale hazard. Move or copy the amendment under the HAZ.
disposition: Fixed in task `n12`: HAZ-cuj9gn's own statement in `org-node/docs/risk/2026-10-15-domain-api.md` now names any Device, listed or not; its rating stays S3/P2.

**finding-7**: record — The new domain terms "ahead limit" and the `first_ahead_at` stamp are used in LLRs of three units. They are not in org-node/docs/CONTEXT.md, or in org-io's or the app's.
disposition: Fixed in task `n12`: the root `docs/CONTEXT.md` defines Ahead limit and amends Verification time, and `org-node/docs/CONTEXT.md` defines First-ahead stamp (used by LLR-4tsgfh).

**finding-8**: record — LLR-hgrez9 `satisfies: REQ-ergn9z, REQ-5q7nyg`, neither of which mentions a limit. Its own note says no app requirement names it. Consider `satisfies: derived` with an assessment in app/docs/risk, as was done for LLR-e7bbm5.
disposition: Fixed in task `a8`: LLR-hgrez9 is now `satisfies: derived` with a dated note, and is assessed in `app/docs/risk/2026-10-15-domain-api.md` (a limit shorter than two intervals could discard genuine next-epoch information before a second chain read; twice the interval prevents that; no new hazard).

### Round 5

> Reviewer: a fresh reviewer over the round's fix tasks (`n12`, `o7`, `a8`), the changes since c458e96; report mr5.md.
> Commit reviewed: 6ff8483.
> Verdict, verbatim: "Not the last round: finding-1 (medium; the residual-risk claim that the logs show the flooder's key) needs a fix or the owner's re-acceptance. The code implements the owner's rulings (1) and (2) exactly, and every claimed test fails when the behaviour it verifies breaks."

**finding-1**: requirement, medium — The residual risk under RC-d4fc4s (org-node/docs/risk/2026-10-15-domain-api.md:115-122) and owner ruling (3) say a sustained flood "comes from a Device whose key the logs show". No log in this software records the sender of Organisation information or of an organisation answer:
- org-node, org-io and the app have no logging dependency (no `tracing`, `log` or subscriber). The app's one `eprintln!` (state.rs:167) is the chain-config fallback.
- The app's `Arrival::ProvisionalOrganisation` drops the sender on purpose (app/src-tauri/src/events.rs:226-230, LLR-4bvdn7).
- org-io's `Received::ProvisionalOrganisationReceived { from }` (org-io/src/receiver.rs:213) carries the sender only as far as the app, which discards it. An organisation answer reports no sender at all.

The owner accepted the residual partly on that traceability, so the acceptance rests on a property the tree lacks. Either remove the claim and have the owner re-confirm the acceptance without it, or add a requirement that the software log or show the sender, and implement it.
disposition: MR5-N1, fixed in task `r6` by owner ruling 2026-10-31, which re-accepted the residual risk without the logging claim: any Device that knows the Organisation identifier, and keeps re-flooding once per ahead limit, can hold off an admission indefinitely and is not identified; each verification clears the flood, the harm is a delayed admission, S1/P2 is kept, and the residual under RC-d4fc4s in `org-node/docs/risk/2026-10-15-domain-api.md` records "Residual accepted by the owner, 2026-10-31" with a dated note that the earlier logging claim was the dispatcher's wording and false. Sender logging is opened as PR-hh4xvx (org-io) for a later improvement; the "records" clause r6 added to the residual was corrected in round 6 (MR6-N1).

**finding-2**: code, low — A failed write of a discard on a `Behind` verification clears the in-memory behind marker. In org-node, `verify` sets `behind` (service.rs:1003) before org-io writes. When the write fails, org-io's `StoreWriter::store` reloads the Persona from its file (org-io/src/lib.rs:181-196). The marker is not encoded, so it reopens `None` (org-node/src/store.rs:984, 1299). Until the next verification:
- `ensure_not_behind` lets the Admin record a revocation or admission, or propose, against a record the chain has moved past;
- an organisation answer is refused with `OrganisationNotRequested`.

LLR-zcw3xv's new note says the marker "is still not persisted", but it does not say that this failure clears a marker the verification has just set. Before this change a `Behind` verification never wrote, so it could not reach this state. It is low because it needs a failed store write and lasts one interval. The same reload after a failed adoption write predates this change. Fix: state it in LLR-zcw3xv, or keep the marker across the reload.
disposition: Fixed test-first in task `r6`: `OrgService::restore_store` in `org-node/src/service.rs` keeps the replaced store's behind marker, so any failed write keeps the Organisation behind (LLR-zk7fx2, LLR-zcw3xv); `a_store_put_back_from_its_file_keeps_the_behind_marker_the_replaced_store_held` and `a_failed_write_of_a_discard_on_a_behind_verification_keeps_the_organisation_behind` failed before the fix.

**finding-3**: requirement, low — LLR-4tsgfh (org-node/docs/architecture/2026-10-16-domain-api.md:818-819) still opens "before judging it (LLR-rr558z) and whatever the outcome, discards …". Later in the same item: "when C is below the epoch R … nothing is discarded or stamped". The amended sentence is right and the code follows it, but the opening clause contradicts it. Fix: change "whatever the outcome" to "unless C is below the record's epoch".
disposition: Fixed in task `r6`: LLR-4tsgfh in `org-node/docs/architecture/2026-10-16-domain-api.md` now opens "unless C is below the epoch R of the store's own record".

**finding-4**: requirement, low — LLR-hgrez9 states that "`init` … gives it the interval undoubled", and the risk assessment (app/docs/risk/2026-10-15-domain-api.md:276-288) says "the tested limit is the one `init` gives". No test reaches `init`'s call at app/src-tauri/src/state.rs:135-140: it reads the environment and connects to a live chain. A regression that passed `policy::verification_ahead_limit(interval)` there would quadruple the limit, and every test would stay green. The shared function narrows the gap but does not close it. Fix: soften the assessment's wording, or extract `init`'s settings step so that it can be tested without connecting.
disposition: Fixed test-first in task `r6`: `init` in `app/src-tauri/src/state.rs` now reads its variables and hands them to `startup_chain`, which reads no environment and is pinned by `startup_chain_builds_the_settings_with_twice_the_interval_it_decides` (failed against a mutant that hands `chain_settings` the doubled limit); LLR-hgrez9 and its assessment in `app/docs/risk/2026-10-15-domain-api.md` state what remains untested, `init` reading the environment and its connect call.

**finding-5**: requirement, low — HAZ-cuj9gn (org-node/docs/risk/2026-10-15-domain-api.md:53-60) and the first residual-risk bullet say Organisation information and organisation answers are kept "from any Device, listed or not". That holds only for a store with no record. With a record, both are refused from an unlisted sender (`SenderNotListed`, org-node/src/service.rs:388-394; LLR-kd7u2r). The amendment note words it right ("a store with no record keeps …"). The statement overstates the hazard, which errs on the safe side, but it is inaccurate: qualify it by store state.
disposition: Fixed in task `r6`: HAZ-cuj9gn, the residual under RC-d4fc4s and LLR-4tsgfh are qualified by store state; a store with no record keeps information from any Device, and a store with a record refuses an unlisted sender with `SenderNotListed` (`org-node/docs/risk/2026-10-15-domain-api.md`).

### Round 6

> Reviewer: a fresh reviewer over the round's fix task (`r6`, 6ff8483..483800a); report mr6.md.
> Commit reviewed: 483800a.
> Verdict, verbatim: "finding-1 (medium; a text-only correction to RC-d4fc4s's residual and PR-hh4xvx), so this is not the last round. Everything else is low or record, and the code changes are correct."

**finding-1**: requirement, medium — The risk text again says something about the software that is not so, this time an understatement.

- Where it appears:
  - RC-d4fc4s's residual, third bullet (org-node/docs/risk/2026-10-15-domain-api.md:137-142): "is not identified: nothing in this software records or shows the sender".
  - PR-hh4xvx's statement (org-io/docs/problems/2026-10-24-domain-api.md:149): "no part of the software records or shows the sender of received Organisation information, of an organisation answer, …".
- What the software does: org-node records the sender of every body it keeps.
  - LLR-s3vq6d requires `origin: Origin` with `Received { from: DevicePublicKey }` / `Answer { from: DevicePublicKey }`.
  - The code sets it to the sender (org-node/src/service.rs:550 and 653).
  - The origin is serialised into the sealed store file (`RawOrigin`, org-node/src/store.rs:483-498) and kept until a verification discards the body.
  - org-io also reports the sender of each kept body to its caller (`Received::ProvisionalOrganisationReceived { from }`, org-io/src/receiver.rs:213). PR-hh4xvx's own "Where" paragraph says so.
- So the bodies that fill the bound during a flood are exactly the ones whose sender is held in the store. What is true:
  - No unit logs the sender.
  - Nothing shows the sender to a user (the app drops it).
  - A refused body's sender is not kept or reported.
- Why it matters: the owner's ruling says only "is not identified", which stays true because nothing shows the sender to a user. The clause "nothing … records" is the dispatcher's addition, the same pattern as MR5-N1. In a class C residual that a reviewer was asked to make "state only what is true", it is a wrong statement that misleads, though it loses nothing and errs on the conservative side.
- Fix: text only.
  - Replace "nothing in this software records or shows the sender" with something like: "nothing in this software logs the sender or shows it to a user; org-node keeps the sender of each kept body in its origin until a verification discards it, and the sender of a refused body is not kept".
  - Correct PR-hh4xvx's statement and its "Expected" paragraph to match: the gap is showing or logging the sender, and keeping the sender of a refused body.
disposition: MR6-N1, fixed in text in task `r7`: RC-d4fc4s's residual and PR-hh4xvx (`org-io/docs/problems/2026-10-24-domain-api.md`) now say no unit logs the sender and nothing shows it to a user, the sender of each kept body is held in its origin (LLR-s3vq6d) and org-io reports it per kept body, and a refused body's sender is not kept; PR-hh4xvx asks for the sender of each body, kept or refused, to be logged or shown. No new owner ruling was needed: the "records" clause came from the dispatcher's r6 instructions, the fix returns the text to what the owner accepted on 2026-10-31, and "is not identified" and that acceptance are unchanged; a dated note records the false clause.

**finding-2**: code, low — When a failed write reloads a store whose held copy is `Removed`, no marker is carried over (org-node/src/service.rs:932, `if let (Ok(held), Ok(restored))`).

- The failing path:
  1. Verification returns `Removed`, so the held content becomes the removed form.
  2. The write of that change fails.
  3. org-io reloads the Active store from its file. `data()` on the held store errs, so the restored store has no marker.
  4. `record_admission`, `record_revocation` and `propose_updated_organisation` are then not refused against a record the chain has moved past, until the next verification, which reports `Removed` again.
- This agrees with LLR-zk7fx2's note: a removed store holds no marker, so there is nothing to keep. It is the same class of gap as MR5's finding-2, but it needs a failed write of a removal. That is rare, and the chain write would have to accept a proposal from a Device the chain no longer lists.
- No test covers a held store that is `Removed`. Either state this case in LLR-zk7fx2's note, or leave it as is knowingly.
disposition: Closed by test in task `r7`: a held store in the removed form has no behind marker, and the Active store's marker is gone once `seal_removal` replaces the content, which LLR-zk7fx2 states; `a_store_put_back_over_a_removed_store_replaces_it_with_no_behind_marker` (`org-node/tests/own_removal.rs`) pins it. The user-facing effect of a failed removal write was re-raised in round 7 and opened as PR-7ud5fv.

**finding-3**: requirement, low — Two items now require the restore behaviour only in dated amendment notes, not in their statements.

- LLR-zk7fx2's statement (org-node/docs/architecture/2026-10-16-domain-api.md:201-210) still lists only "`verify` sets it … clears it" and "`None` after `unseal`". The new rule, that `restore_store` keeps the replaced store's marker, appears only in the 2026-10-31 note.
- The same holds for LLR-zcw3xv (org-io/docs/architecture/2026-10-16-domain-api.md:974-985 vs 1008-1016).
- The statements do not contradict the new behaviour; they are silent on it. Contrast LLR-4tsgfh, whose statement this round edited directly.
- The tests' `verifies:` therefore point at behaviour that lives in notes. This is a documentation gap where the tree is right, and it follows existing project practice.
disposition: Fixed in task `r7`: the statements of LLR-zk7fx2 (`org-node/docs/architecture/2026-10-16-domain-api.md`) and LLR-zcw3xv (`org-io/docs/architecture/2026-10-16-domain-api.md`) now carry the restore rule, including the removed-form case.

**finding-4**: requirement, low — LLR-4tsgfh's last sentence (org-node/docs/architecture/2026-10-16-domain-api.md:867-870) reads "Nothing restricts the sender: … kept from any Device by a store with no record …, and from a Device the record lists by a store with one". The sentence then states a restriction, the listing, so "Nothing restricts the sender" now contradicts its own continuation. The intended meaning is "this item adds no sender restriction". The fix is a rewording, with no behaviour change.
disposition: Fixed in task `r7`: LLR-4tsgfh's last sentence now opens "These rules add no restriction on the sender:" (`org-node/docs/architecture/2026-10-16-domain-api.md`); no behaviour changes.

**finding-5**: code, low — The app test's name, `init_builds_the_settings_it_connects_with_from_its_environment_with_twice_the_interval_decided` (app/src-tauri/tests/verification.rs:242), says `init` and "its environment". The test actually calls `startup_chain` through `startup_chain_for_test`, with values that only look like the environment's; it reaches neither `init` nor the environment. Finding-4 of round 5 was exactly an overclaim that `init` was tested. The item note and the risk note now state the gap correctly, but the name still suggests `init` is covered. Rename it, for example `startup_chain_builds_the_settings_init_connects_with_with_twice_the_interval_decided`.
disposition: Fixed in task `r7`: the test is renamed `startup_chain_builds_the_settings_with_twice_the_interval_it_decides` (`app/src-tauri/tests/verification.rs`), and LLR-hgrez9's note follows.

**finding-6**: record — Two small points in the plan's r6 disposition (docs/plans/2026-10-21-org-io-domain-api.md, r6 entry):
- "the residual now says … nothing … records" is not stated, but the disposition calls MR5-N1 "fixed by owner ruling" without noting that the added "records" clause is not part of the ruling (see finding-1).
- The test count for org-io in the gates ("line 1 passes 329") has no breakdown. This round's verification count is 36.
disposition: Fixed in task `r7`'s entry in `docs/plans/2026-10-21-org-io-domain-api.md`: it records that the r6 disposition called MR5-N1 "fixed by owner ruling" while its "records" clause was the dispatcher's wording, not the owner's, and the RC-d4fc4s note records the same; the corrected disposition is carried into `docs/verification/2026-10-31-worktree-org-io-domain-api.md`.

### Round 7

> Reviewer: a fresh reviewer over the round's fix task (`r7`, 483800a..32fde8c); report mr7.md.
> Commit reviewed: 32fde8c.
> Verdict, verbatim: "no medium or high findings. This is the last review round; fix the record line for the renamed test before verify-before-merge."

**finding-1**: code, low — A failed write of a removal leaves the Persona Active with no behind marker, even when the Active store held one before the removal (`org-node/src/service.rs:1350-1355` and `929-939`; `org-io/src/lib.rs:186-189`).
- How it happens: a Device that a Behind verification marked, and that a later verification then removes, is a common case. If the write of the removed store fails, the store reloaded from its file is Active, and `restore_store` finds no marker on the held removed store.
- The effect until the next verification is the same one round 5 finding-2 fixed for `Behind`: `record_admission`, `record_revocation` and `propose_updated_organisation` are not refused, and an organisation answer is refused as not requested.
- The r7 disposition's "a failed write of a removal loses nothing" is true of the restore step only, not of the state the user is left in.
- Why it is low: it needs a failed disk write, and the next verification sees the same chain state and repeats the removal. LLR-zk7fx2 and LLR-zcw3xv now state this behaviour accurately, so text and code agree; it is a design choice the owner may accept as stated.
disposition: not fixed — opened by the dispatcher as PR-7ud5fv (org-node, status open, affects LLR-zk7fx2) with the cause and a suggested fix, since it needs a failed disk write and the next verification repeats the removal; LLR-zk7fx2 has a note pointing to it, recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-2**: requirement, low — Two edge cases of the changed text are not stated.
- (a) The residual's "until a verification discards or adopts the body" (`org-node/docs/risk/2026-10-15-domain-api.md:137-142`) leaves out two other ways a kept body goes: `drop_store` (`org-io/src/retention.rs:65,70`, `persona.rs:53,92`, and the dropped Persona after a failed reload at `org-io/src/lib.rs:192`), and a failed write, whose reload discards a body kept in memory but never written.
- (b) LLR-zk7fx2's restore sentence does not state what happens when the replacement itself is in the removed form and replaces an Active store. The code leaves it with no marker (`data_mut()` fails). That state is not reached in normal use.
- Neither affects the residual's conclusion that the flooder is not identified.
disposition: Fixed in text by the dispatcher: RC-d4fc4s's residual and PR-hh4xvx's note now say the sender is held until a verification discards or adopts the body, or the store itself is dropped, and that a body counts as kept only once its store is written; LLR-zk7fx2's statement names a replacement in the removed form, with a dated note. No behaviour changes.

**finding-3**: record — The round-5 disposition's `red -> green:` line (`docs/plans/2026-10-21-org-io-domain-api.md:42172`) still names `app/src-tauri/tests/verification.rs::init_builds_the_settings_it_connects_with_from_its_environment_with_twice_the_interval_decided`, which no longer exists after r7's rename.
- The r7 disposition records the rename in prose ("Finding-5 fixed") but gives no `red -> green:` or other line tying the attestation to `startup_chain_builds_the_settings_with_twice_the_interval_it_decides`.
- verify-before-merge check 4 reads those lines and will look for a test that is gone. Add a line to the r7 block that maps the old name to the new one, or correct the round-5 line with a dated note.
- Related: LLR-hgrez9's dated round-5 note was edited in place to carry the new name (`app/docs/architecture/2026-10-16-domain-api.md:761`). The round-6 note below it records the old name, so the history is still recoverable. This is noted only, not a separate finding.
disposition: Fixed by the dispatcher: the round-5 `red -> green:` line in `docs/plans/2026-10-21-org-io-domain-api.md` now names the renamed test `startup_chain_builds_the_settings_with_twice_the_interval_it_decides` and states its original name and the rename by task `r7`, as recorded in `docs/verification/2026-10-31-worktree-org-io-domain-api.md`.

## Guideline reviews

The `develop-change` conformance reviews, rounds 1–9, one review per
guidelines file that `guidelines-file.sh` printed for the change diff, split by
unit for depth (TEST and CODE: app with on-chain-client, org-io, org-node).
Each review ran again only while its last round had a finding above low. Every
path resolved to the installed defaults under `.guardrails/templates/`, except
TEST, which resolved to the root tailoring `docs/TEST_GUIDELINES.md` from round
3 on, once the owner had ruled it into existence (T2-A6). Findings are verbatim
from the round reports, renumbered within each round, with the unit prefixed;
round 1's reports were not kept, so its findings are the plan's summaries of
them. The owner ruled on every recurrence: A2 (an exception to Boundaries 2),
T2-A2 and T2-A6 (by tailoring `docs/TEST_GUIDELINES.md`), T3-A1, T3-O2, the
LLR-zzsqj4 ruling, T5-N2 (by minting LLR-c759yd), T5-O5 (accepted under
SDD-z85ux9's deviation) and C7-N1 (one targeted fix); each ruling is in its
finding's disposition.

### .guardrails/templates/TEST_GUIDELINES.md

Governs: the test files under org-node/tests, org-io/tests, app/src-tauri/tests, app/tests and on-chain-client/tests that master...worktree-org-io-domain-api adds or modifies, with their test support (app/tests/fake-tauri.ts and the backend.json fixture, org-io/src/test_support.rs, org-node/tests/support) and the LLRs those tests verify, as the review reports state their scope.

### Round 1

Round 1's report text was not kept; it survives only as the summaries in the plan's "Guideline reviews — round 1" section (docs/plans/2026-10-21-org-io-domain-api.md), which the blocks below quote.

**finding-1**: guideline, high — app+on-chain-client: T-A1 — the founding test fakes org-io's "a Person is held already" text, with no contract test pinning it (Doubles 3).
disposition: fixed in task `tfa` (merged 2026-10-25): an ipc test drives the second `propose_person` over IPC and asserts the refusal equals `PERSON_ALREADY_HELD` read from founding.ts, mutation-checked; the test is now `a_second_person_is_refused_by_its_kind_beside_org_ios_text` in app/src-tauri/tests/ipc.rs, verifying LLR-k57uqv after round 2's T2-A4 and AR2-4 fixes.

**finding-2**: guideline, medium — app+on-chain-client: T-A2 — `FakeOps` `free_balance` / `mapped_account` have no contract against `SubxtWriteOps` (Doubles 3).
disposition: fixed for the fake in task `tfa`: the shared contract on-chain-client/tests/support/write_reads_contract.rs runs against `FakeOps`, mutation-checked; its real side, `subxt_reads_answer_the_write_reads_contract` in on-chain-client/tests/write_genesis_e2e.rs, is chopsticks-only, outside the gates, and was not run (no chopsticks in the worktree).

**finding-3**: guideline, medium — app+on-chain-client: T-A3 — webview tests regex-match component source instead of rendering (UI 1/3); a false green is possible on the admission gate.
disposition: not fixed in task `tfa` (rendered-DOM tests needed a DOM test dependency, the owner's not-minted control 1 in app/docs/risk/2026-09-14-app-hazards.md §8); owner ruling 2026-10-27: add the DOM dependencies now. Fixed in task `dom` (merged 2026-10-28): @testing-library/svelte and jsdom added, every listed source-regex test rendered against a fake of the Tauri boundary only (app/tests/fake-tauri.ts, app/tests/page.ts), and the admission gate proven both ways (LLR-e63yrs), with SOUP rows and a dated note on control 1.

**finding-4**: guideline, medium — app+on-chain-client: T-A4 — T58's error-label test is annotated LLR-5mw2dj, which states no such label (Seams 1).
disposition: fixed in task `tfa`: LLR-5mw2dj amended in place to state both start-up error labels, so the existing annotation in app/tests/notices.admission.test.ts now names what the test checks.

**finding-5**: guideline, medium — app+on-chain-client: T-A5 — periodic, verification and receiver tests call helpers no LLR names (Seams 2).
disposition: fixed in task `tfa`: tests moved up to the commands and `run_periodic_verification`; the helper tests kept are under new LLRs, each citing its Seams 2 criterion: LLR-73by2q (`entry_for`), LLR-cugcu6 (`device_reports`), LLR-q96vdr (`OrgDto::new`) and LLR-j9m88u (`receive_iteration`, tested by `one_iteration_verifies_once_before_the_next_message_and_announces_what_it_verified`).

**finding-6**: guideline, medium — app+on-chain-client: T-A6 — periodic.rs asserts source text of owned calls (Doubles 1).
disposition: fixed in task `tfa`: app/src-tauri/tests/periodic.rs asserts the emitted events, not source text (for example `an_adopted_result_is_announced_as_the_receiver_loop_announces_it`, mutation-checked).

**finding-7**: guideline, low — app+on-chain-client: app low 7 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfa` records "Low 7 not fixed" with no reason given (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST app fix).

**finding-8**: guideline, medium — org-node: T-N1 — `seal_removal_for_test` below the seam (Seams 1).
disposition: fixed in task `tfn` (merged 2026-10-25): verify's removal is driven and the `seal_removal_for_test` hook is deleted; `a_removal_seals_a_removed_store_for_that_personas_device_key` in org-node/tests/own_removal.rs went red under a mutation (LLR-2d5a7m).

**finding-9**: guideline, medium — org-node: T-N2 — support fixtures hand-build states the operations no longer produce (no change set, no revoked entry) (Seams 1 / Owned services 2).
disposition: fixed in task `tfn`: every support fixture in org-node/tests/support/mod.rs runs the real operations, so no hand-built state is left; a mutation of `messages_for_current_epoch` turned the receive_path, receive_chain_reads, provisional_store and admission_sender fixtures' tests red, showing the change-set path now runs.

**finding-10**: guideline, medium — org-node: T-N3 — an LLR-f9x3vf test checks only through `store_data`.
disposition: fixed in task `tfn`: every per-store operation is driven with an unknown key; `an_unknown_device_key_is_refused_as_persona_not_held` in org-node/tests/sealed_store.rs went red when verify resolved an unknown key to store 0 (LLR-f9x3vf).

**finding-11**: guideline, medium — org-node: T-N4 — sealed_store tests seal through `seal_for_test`.
disposition: fixed in task `tfn`: org-node/tests/sealed_store.rs seals through the operations; `a_sealed_store_unseals_with_its_secret_and_device_key` and its siblings went red when `add_store` sealed a removed form (LLR-bg538c, LLR-2qqqx6, LLR-f9x3vf).

**finding-12**: guideline, medium — org-node: T-N5 — absences scans pin private call chains and `stores.retain(`.
disposition: fixed in task `tfn`: org-node/tests/absences.rs follows callers up to the public entry points (`into_removed_is_called_only_from_verify`, mutation-checked, LLR-9jn3z8); round 2's T2-N1 later found the same pin on `remove_acknowledgement`, a different function.

**finding-13**: guideline, medium — org-node: T-N6 — absences scans pin private call chains and `stores.retain(`.
disposition: fixed in task `tfn`: any removing method now passes the scan; `only_the_removal_and_the_accepted_acknowledgement_delete` in org-node/tests/absences.rs stayed green with `drop_store` rewritten with position plus swap_remove, and went red when `drop_store` removed nothing (LLR-9jn3z8).

**finding-14**: guideline, medium — org-node: T-N7 — no abnormal test for an undecodable Organisation answer snapshot (LLR-kd7u2r).
disposition: fixed in task `tfn`: `an_answer_whose_snapshot_does_not_decode_or_parse_is_refused_and_keeps_nothing` in org-node/tests/organisation_request.rs (LLR-kd7u2r), red under a mutation that read an undecodable snapshot as an empty member list.

**finding-15**: guideline, low — org-node: low 8 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfn`: the vacuous chain-read counts were removed (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-node fix).

**finding-16**: guideline, low — org-node: low 9 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-17**: guideline, low — org-node: low 10 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-18**: guideline, low — org-node: low 11 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-19**: guideline, low — org-node: low 12 — the report's text was not kept; the plan names it only by number
disposition: fixed in part in task `tfn` ("12 in part"); the plan records no detail of the part left (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-20**: guideline, low — org-node: low 13 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-21**: guideline, low — org-node: low 14 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-22**: guideline, low — org-node: low 15 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-23**: guideline, low — org-node: low 16 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-24**: guideline, low — org-node: low 17 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-25**: guideline, low — org-node: low 18 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-26**: guideline, low — org-node: low 19 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfn` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-node fix, "Lows fixed").

**finding-27**: guideline, low — org-node: low 20 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfn` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-node fix, "Lows fixed").

**finding-28**: guideline, low — org-node: low 21 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfn` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-node fix, "Lows fixed").

**finding-29**: guideline, low — org-node: low 22 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-30**: guideline, low — org-node: low 23 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfn` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-node fix, "Lows fixed").

**finding-31**: guideline, low — org-node: low 24 — the report's text was not kept; the plan names it only by number
disposition: not fixed — task `tfn` lists it among "Lows 9–11, 13–18, 22 and 24 are not fixed", with no reason given (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-32**: guideline, low — org-node: low 25 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfn` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-node fix, "Lows fixed").

**finding-33**: guideline, low — org-node: low 26 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfn` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-node fix, "Lows fixed").

**finding-34**: guideline, medium — org-io: T-O1 — six endpoints tests cut "for T60" were never re-created (PR-8qsnhx, PR-47aycj, PR-tcd2ta regressions; LLR-d2pzzk, LLR-z35vbh) (Existing suites 1).
disposition: fixed in task `tfo` (merged 2026-10-28): endpoint tests re-created at the public API in org-io/tests/endpoints.rs (`a_second_organisations_message_goes_out_under_its_own_personas_key`, `after_its_endpoint_closes_a_send_binds_it_again`, `a_send_that_rebinds_a_closed_endpoint_while_the_receiver_runs_accepts_on_it`, `a_refused_rebind_while_the_receiver_runs_binds_and_accepts_on_the_next_send`), each mutation-checked, verifying LLR-d2pzzk, LLR-z35vbh, PR-8qsnhx, PR-tcd2ta and PR-47aycj; the creation and reply tests are succeeded by LLR-jvx6j9's tests.

**finding-35**: guideline, medium — org-io: T-O2 — three normal-case tests cut from expectations_met.rs (REQ-x6kdse, qza96g, v6r2gd; RC-d8dbxs, RC-jgyd6m).
disposition: fixed in task `tfo`: the normal-case tests are re-created over the domain operations in org-io/tests/expectations_met.rs (`every_store_change_reaches_the_file_whole_and_reopens_as_held`, REQ-x6kdse; `a_store_file_opens_under_its_own_item_and_keeps_the_device_key_not_the_seed`, REQ-qza96g; `the_device_seed_is_in_the_keychain_alone_and_renders_no_bytes`, REQ-v6r2gd), each mutation-checked.

**finding-36**: guideline, medium — org-io: T-O3 — an REQ-2xgsbn test drives org-node directly.
disposition: fixed in task `tfo`: `outstanding_invites_are_kept_in_the_store_files_and_settled_by_adoption` in org-io/tests/expectations_met.rs is rewritten to use only the handle's operations (REQ-2xgsbn), red under a mutation that skipped the store write on a later adoption.

**finding-37**: guideline, medium — org-io: T-O4 — store_files and rng_source tests call `store_for_test`.
disposition: fixed in task `tfo`: org-io/tests/store_files.rs and org-io/tests/rng_source.rs use the handle's founding write instead of `store_for_test` (for example `a_failed_write_reports_failure_and_reloads_the_persona_from_its_file`, LLR-murjr6; `a_failing_entropy_source_refuses_the_store_write_and_writes_nothing`, LLR-94nn6q); round 2's T2-O1 then deleted `store_for_test` altogether.

**finding-38**: guideline, medium — org-io: T-O5 — a direct test of `next_round_after` with no LLR.
disposition: fixed in task `tfo`: the direct `next_round_after` asserts and its `pub` re-export from org-io/src/lib.rs are removed; the handle-driven half is kept.

**finding-39**: guideline, medium — org-io: T-O6 — `FakeChain` `create_proxy` / `found_at` have no contract test; write.rs's step mapping and `h160_of` never run.
disposition: partly fixed in task `tfo`: one founding contract (org-io/tests/common/chain_writer_contract.rs) runs over `FakeChain` in the gate as `fake_chain_meets_the_founding_contract` (red first; the fake now gives OrgId = h160_of(proxy)) and over the production `OnChainWriter` as `the_production_writer_meets_the_founding_contract`, which is chopsticks-only and was not run; write.rs's `h160_of` and `WriteError::Step` mapping stay reachable only through the chopsticks test, because `OnChainWriter` is concrete over `SubxtWriteOps`.

**finding-40**: guideline, medium — org-io: T-O7 — keychain.rs:235 was re-annotated to LLR-kna4zj, which does not state its rule.
disposition: not fixed in task `tfo` (no in-force LLR stated the lend-to-keychain rule; it was only in superseded LLR-n9dnm4) and passed on; fixed in task `arch` (merged 2026-10-28): new LLR-6sfub6 satisfies REQ-rz93uz under RC-jgyd6m, and `the_write_lends_the_bytes_only_through_lend_to_keychain` in org-io/tests/keychain.rs verifies it, mutation-checked.

**finding-41**: guideline, low — org-io: low 8 — the report's text was not kept; the plan names it only by number
disposition: not fixed — the `tfo` task report lists lows 8, 10, 11, 15, 16, 18, 20, 24 and 25 as not fixed, "not cheap, or already covered" (docs/plans/2026-10-21-org-io-domain-api.md records only the fixed lows).

**finding-42**: guideline, low — org-io: low 9 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfo`; the task report ties it to `an_unknown_outcome_on_one_handle_never_refuses_its_other_target` in org-io/tests/chain_write.rs (LLR-mhn4n5), whose per-handle mutation left the older LLR-mhn4n5 tests green, "which confirms finding-9".

**finding-43**: guideline, low — org-io: low 10 — the report's text was not kept; the plan names it only by number
disposition: not fixed — the `tfo` task report lists it as not fixed, "not cheap, or already covered" (docs/plans/2026-10-21-org-io-domain-api.md records only the fixed lows).

**finding-44**: guideline, low — org-io: low 11 — the report's text was not kept; the plan names it only by number
disposition: not fixed — the `tfo` task report lists it as not fixed, "not cheap, or already covered" (docs/plans/2026-10-21-org-io-domain-api.md records only the fixed lows).

**finding-45**: guideline, low — org-io: low 12 — the report's text was not kept; the plan names it only by number
disposition: fixed in part in task `tfo`, "12 (order)": the ordering is tested, but its "closes the Persona's endpoint" clause is not, because the delivery endpoint's closure cannot be seen from any test interface (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-46**: guideline, low — org-io: low 13 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfo` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-io fix, "Lows fixed").

**finding-47**: guideline, low — org-io: low 14 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfo` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-io fix, "Lows fixed").

**finding-48**: guideline, low — org-io: low 15 — the report's text was not kept; the plan names it only by number
disposition: not fixed — the `tfo` task report lists it as not fixed, "not cheap, or already covered" (docs/plans/2026-10-21-org-io-domain-api.md records only the fixed lows).

**finding-49**: guideline, low — org-io: low 16 — the report's text was not kept; the plan names it only by number
disposition: not fixed — the `tfo` task report lists it as not fixed, "not cheap, or already covered" (docs/plans/2026-10-21-org-io-domain-api.md records only the fixed lows).

**finding-50**: guideline, low — org-io: low 17 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfo` (docs/plans/2026-10-21-org-io-domain-api.md, "Lows fixed"); its test uses three other Devices because org-node picks fewer acknowledgement recipients than candidates.

**finding-51**: guideline, low — org-io: low 18 — the report's text was not kept; the plan names it only by number
disposition: not fixed in task `tfo` — its failed-acceptance part was already covered by the coverage task `cov` (`a_failed_write_of_an_acceptance_keeps_the_acknowledgement_for_the_next_round`, LLR-6ux6yw) (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-52**: guideline, low — org-io: low 19 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfo` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-io fix, "Lows fixed").

**finding-53**: guideline, low — org-io: low 20 — the report's text was not kept; the plan names it only by number
disposition: not fixed in task `tfo` — already covered earlier, per the task report (docs/plans/2026-10-21-org-io-domain-api.md records only the fixed lows).

**finding-54**: guideline, low — org-io: low 21 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfo` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-io fix, "Lows fixed").

**finding-55**: guideline, low — org-io: low 22 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfo` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-io fix, "Lows fixed").

**finding-56**: guideline, low — org-io: low 23 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfo` (docs/plans/2026-10-21-org-io-domain-api.md, round 1 TEST org-io fix, "Lows fixed").

**finding-57**: guideline, low — org-io: low 24 — the report's text was not kept; the plan names it only by number
disposition: not fixed — the `tfo` task report lists it as not fixed, "not cheap, or already covered" (docs/plans/2026-10-21-org-io-domain-api.md records only the fixed lows).

**finding-58**: guideline, low — org-io: low 25 — the report's text was not kept; the plan names it only by number
disposition: not fixed in task `tfo` — already covered earlier, per the task report (docs/plans/2026-10-21-org-io-domain-api.md records only the fixed lows).

**finding-59**: guideline, low — org-io: low 26 — the report's text was not kept; the plan names it only by number
disposition: fixed in task `tfo`: low T-O26 is T44's guard (untested false side; reachable through `drop_store`), now tested by `a_store_dropped_from_memory_keeps_its_live_file_and_item` in org-io/tests/acknowledgements.rs (LLR-vj2nkb, LLR-3fxxpp), red with the guard removed.

### Round 2

**finding-1**: guideline, medium — app+on-chain-client: Doubles 3, "Every fake has a contract test. The same assertions run against the fake and against the real dependency ... A fake with no contract test drifts: it accepts what the real dependency rejects" — app/tests/fake-tauri.ts:1-88; app/tests/invite.confirm.test.ts:31; app/tests/notices.admission.test.ts:185, 207
The faked Tauri boundary answers each command with service-specific content that the webview interprets: DTO shapes, `status` values and refusal texts. Only one answer has a contract: the `propose_person` refusal text, pinned by ipc.rs:424-431. Everything else is unchecked, and it has already drifted. The fake answers `propose_member` with `status: 'provisional'`, but the real command returns `"provisional_member"` (commands.rs:309, LLR-rg9vc7, asserted in ipc.rs `propose_member_hands_the_handles_in_order_and_sends_nothing`). The same applies to `START` (`connection_status`, `list_*`, `verify_organisation`), `read_invite`, `request_admission`, `write_organisation_to_chain` and the `verification-result` and `admission-requested` payloads that `tauri.emit` sends.
disposition: fixed as T2-A1 in task `a2` (merged 2026-10-28), with the owner ruling 2026-10-28 on T2-A2: one shared fixture, app/tests/contract/backend.json, now holds every answer and event payload; app/tests/fake-tauri.ts refuses any shape not in it, and app/src-tauri/tests/ipc.rs drives the real producer for every entry (`every_backend_entry_has_a_real_producer_here`, failed with backend.json mutated). Four drifts were found and fixed, `provisional_member` among them.

**finding-2**: guideline, medium — app+on-chain-client: Owned services 1, "A service the project owns runs for real on the normal path ... A backend ... that the codebase contains ... runs for real in a REQ-level test of the normal case" (repeated in UI 3, "A service the project owns runs for real on the normal path (Owned services 1)") — app/tests/fake-tauri.ts:20-29; founding.propose.test.ts:76-108; invite.confirm.test.ts:29-46; notices.admission.test.ts:78-104, 110-129; notices.discarded.test.ts:89-98, 151-178; verification.view.test.ts:180-190, 211-222, 262-282
The Rust backend in app/src-tauri is part of this codebase, yet every rendered feature test fakes it on the normal path. Examples are a successful `propose_admission`, `propose_founding_organisation`, `write_organisation_to_chain` `included`, and `verify_organisation` and `list_admission_requests` answers. The clause permits a fake only for a failure case, and then only under a contract test (see finding-1).
disposition: owner ruling 2026-10-28 (T2-A2): keep the rendered tests on the Tauri-boundary fake, contract-test every answer and event payload it gives in app/src-tauri/tests/ipc.rs, and record the boundary in a dated note on control 1 of app/docs/risk/2026-09-14-app-hazards.md §8; done in task `a2`. The ruling was written into docs/TEST_GUIDELINES.md as a tailoring of Owned services 1 in round 3 (T3-A1, task `a3`).

**finding-3**: guideline, medium — app+on-chain-client: Seams 1, "A test calls only an interface that a REQ or LLR describes. A private helper is covered through the interface above it"; Seams 2, "A deeper interface gets a direct test only once it has an LLR" — app/tests/founding.propose.test.ts:44, 63 (and the four `proposeFounding` tests); app/tests/verification.view.test.ts:213, 241-247; app/tests/notices.discarded.test.ts:160-188; app/src-tauri/tests/ipc.rs:954; app/src-tauri/tests/ipc.rs:839
These tests call helpers that no LLR, REQ or SDD file list in app/docs names: `proposeFounding` (founding.ts, which no item lists), `foundingWrites`, `fileVerificationResults`, `reproposalStep`, `commands::chosen_handle` and `AdmissionDto::from`. Each covers behaviour stated at an interface above it: LLR-6ff735 and LLR-rhc555 (the creation form and write button), LLR-84cxpt (the view's list), LLR-md6hhz and LLR-e63yrs (the panel), LLR-gmb4p8 (`propose_admission`) and LLR-8cz828 (`request_admission`). Several of those tests already drive the same case through the DOM or IPC next to the direct call. The other LLR-gmb4p8 blanks (`None`, `""`, `"\t\n"`) and LLR-8cz828's `Refused` mapping are reached only through the helpers.
disposition: fixed as T2-A3 in task `a2`: the tests now go through the rendered page or the IPC exchange, and `chosen_handle` is private; for example `a_blank_handle_is_handed_on_as_none_and_another_unchanged` in app/src-tauri/tests/ipc.rs drives the blanks of LLR-gmb4p8 over IPC, failed with no blank filter and with trimming.

**finding-4**: guideline, medium — app+on-chain-client: Test guidelines preamble, "The compliance floor the skills state wins any conflict"; the floor: "every test fails when the behavior it `verifies:` breaks" — on-chain-client/tests/write_compose.rs:578, 584, 590; on-chain-client/tests/write_genesis_e2e.rs:90; app/src-tauri/tests/ipc.rs:424
The contract tests of the substitute's two reads cite LLR-jk76wf and LLR-95puh9, which are `found_at`'s skip rules, yet none of them calls `found_at`. They also cite LLR-htp5tk (`free_balance_of`), which `FakeOps` never calls. Breaking any of those three behaviours leaves the write_compose contract tests green. The chopsticks test reaches only LLR-htp5tk, through `SubxtWriteOps::free_balance`. The same pattern appears in ipc.rs:424: `a_second_person_is_refused_with_the_text_the_creation_form_passes_over` cites LLR-6ff735 (`propose_founding_organisation`) but proposes no founding. A contract test should cite only what it can redden, or carry no LLR beyond the fake's contract.
disposition: fixed as T2-A4 in task `a2`: the write_compose contract tests in on-chain-client/tests/write_compose.rs now run `found_at` (LLR-jk76wf, LLR-95puh9; the funded test failed with `found_at`'s `<` changed to `<=`), LLR-htp5tk stays only on the chopsticks test, and the ipc.rs test is now `a_second_person_is_refused_by_its_kind_beside_org_ios_text`, citing LLR-k57uqv.

**finding-5**: guideline, medium — app+on-chain-client: Doubles 5, "The robustness rule is that every requirement gets abnormal-input tests (bad input, boundary values, resource exhaustion, dependency failure) beside its normal-case tests" (the floor: "class B and C: abnormal-input tests for every REQ and LLR") — app/src-tauri/tests/verification.rs:494, 611; app/src-tauri/tests/ipc.rs:636; app/src-tauri/tests/periodic.rs:166, 203; app/tests/verification.view.test.ts:211
These LLRs have only normal-case tests:
- LLR-es49wp: no test of a send refused as a whole (`sends` empty, `sends_refused` carrying org-io's text). This is reachable at `verify_and_send`, for example with the endpoint not bound.
- LLR-nkg8sb: no test of `requests_refused`, and no test of a `failed` Device entry.
- LLR-y3zykm: no refusal case. The org-io refusal in ipc.rs `issue_invite_does_not_refuse_a_well_formed_org_id` is annotated LLR-97hujy only.
- LLR-2267hj: no periodic run whose verification fails, such as `read_failing` giving `unverified`.
- LLR-84cxpt: no abnormal event, such as an unknown result or an event arriving before the start's listing.
disposition: fixed as T2-A5 in task `a2`: abnormal tests added, each mutation-checked: `a_send_or_request_refused_as_a_whole_is_an_empty_report_and_its_text` (LLR-k4rxyt, LLR-es49wp, LLR-nkg8sb), `a_behind_request_a_device_refuses_is_reported_failed_for_that_device` (LLR-nkg8sb), `issue_invite_for_an_organisation_not_held_is_org_ios_refusal_as_its_text` (LLR-y3zykm), `a_run_whose_chain_read_fails_is_announced_unverified_and_the_next_run_recovers` (LLR-2267hj), and an event arriving before the start's listing in app/tests/verification.view.test.ts (LLR-84cxpt).

**finding-6**: guideline, medium — app+on-chain-client: Seams 2, "A deeper interface gets a direct test only once it has an LLR, and it earns one only where at least one of these holds" (cited at .guardrails/templates/TEST_GUIDELINES.md:16) — a clause that contradicts the floor
The floor says "class C: tests at every touched SDD item's interface". When a class C SDD item's interface sits below the REQ interface, Seams 2 forbids that test unless criterion 1, 2 or 3 holds. SDD-mwqf6x is an example: its interface is `outcome_for_arrival`, `outcomes_for_verification`, `follow_up_for` and `classify_receive_error`. Its touched items have tests at that interface (receiver_events.rs: LLR-4bvdn7, LLR-ygr6gd, LLR-dp45mm, LLR-t9rdf8, LLR-w7m3d4, LLR-acb5g4, LLR-8grnbb), and none states a criterion. Several cases are reachable through the real receiver loop with the exchange harness ipc.rs already has. Read literally, Seams 2 makes those required tests findings. It has also pushed the draft into justifying each SDD-level test by criterion (the "Added 2026-10-25" notes under LLR-73by2q, LLR-cugcu6, LLR-q96vdr and LLR-j9m88u). The floor wins, so the clause needs an exception for an SDD item's own interface in class B and C units.
disposition: owner ruling 2026-10-28 (T2-A6): tailor the TEST guidelines for class B/C units so that an SDD item's named interface is a seam that may be tested directly when an LLR states its contract. Fixed in task `a2`: the new root docs/TEST_GUIDELINES.md is the template verbatim plus Seams 2 criterion 4, with a dated tailoring note; rounds 3 to 6 review against it.

**finding-7**: guideline, low — app+on-chain-client: the floor, "every test fails when the behavior it `verifies:` breaks", for the "exactly once" in LLR-fe7umz and LLR-nkg8sb — app/src-tauri/tests/periodic.rs:236; app/src-tauri/tests/verification.rs:611-912
LLR-fe7umz states that a behind Organisation is asked for "exactly once at each run". The test checks only that `requests` is an array. LLR-nkg8sb states that `request_organisation` is asked "exactly once". The test checks that one Device entry is reported, which a second call would not change. A run that requested twice would pass both tests.
disposition: not fixed — task `a2` lists T2 findings 7–9 as not fixed (docs/plans/2026-10-21-org-io-domain-api.md, round 2 app fix); carried open, not blocking. LLR-fe7umz and LLR-nkg8sb keep their tests in app/src-tauri/tests/periodic.rs and app/src-tauri/tests/verification.rs.

**finding-8**: guideline, low — app+on-chain-client: UI 3, "Find elements the way a user does: by accessible role, label and visible text, not by test identifier or class name" — app/tests/verification.view.test.ts:31-32, 276; app/tests/notices.admission.test.ts:92-94; app/tests/notices.discarded.test.ts:75, 200-203, 235
These tests reach elements through DOM structure: `heading.nextElementSibling`, `querySelectorAll('li')`, `getByText(...).nextElementSibling` and `.parentElement`. They should use roles and accessible names, such as `getAllByRole('listitem')` inside the list's region, a definition term's accessible value, or `within(getByRole(...))`. A layout refactor that keeps every visible behaviour breaks them.
disposition: not fixed — task `a2` lists T2 findings 7–9 as not fixed (docs/plans/2026-10-21-org-io-domain-api.md, round 2 app fix); the structural lookups recurred as lows in round 3 and in round 4 and stay carried open, not blocking (app/tests/verification.view.test.ts).

**finding-9**: guideline, low — app+on-chain-client: UI 4, "List the states each screen can be in and test each: loading, empty, error, partial, stale, offline, permission denied" — app/tests/notices.admission.test.ts; app/tests/notices.discarded.test.ts; app/tests/verification.view.test.ts
The changed screens have no state list. Error states are tested: the start's refused listing, subscription and verification. The Admit panel with no kept request, the Membership panel with no notice, the empty "Last verifications" list, and the loading state of any of these are not tested.
disposition: not fixed — task `a2` lists T2 findings 7–9 as not fixed (docs/plans/2026-10-21-org-io-domain-api.md, round 2 app fix); the pending and empty states recurred as a low in round 4 and stay carried open, not blocking (app/tests/notices.admission.test.ts).

**finding-10**: guideline, medium — org-io: Seams 1, "A test calls only an interface that a REQ or LLR describes", and Seams 2, "Convenience is not on the list. A case reachable from the REQ interface with a boundary fake is tested there, even where a deeper seam would be easier".

Several new setup helpers and tests reach their state in two ways that skip the handle:
- They call org-node's operations through `node_for_test()` (`record_founding_proxy`, `verify`, `check_received`) with the test's own `OsRng`.
- They write the result with the test-only `store_for_test`, which no REQ or LLR describes.

None of these handle operations is out of reach. `write_organisation_to_chain`, `verify_organisation`, `request_admission`/`request_organisation` with `next_received`, and `send_organisation` with `next_received` all run here over `FakeChain` and Loopback. `tests/endpoints.rs:120-159` (`found`, `admitted`) and `common/journey.rs:202-248` already do exactly that.

Two helpers mutate org-node's in-memory store and write nothing, so the tests built on them start from a state the shipped receive step never produces:
- `common/journey.rs:269-277` (`deliver_by_node`)
- `common/journey.rs:292-299` (`keep_addresses_of`)

Nearly every test in `acknowledgements.rs` and `open.rs:122-125` uses them.

Locations:
- `org-io/tests/common/handles.rs:89-96` (`create_named`), `:234-248` (`founded`), `:252-258` (`requested`)
- `org-io/tests/common/journey.rs:73-79` (`keep_request`), `:269-277`, `:292-299`, `:303-306`
- `org-io/tests/chain_write.rs:87-93`
- `org-io/tests/verification.rs:47-49`, `:64-65`, `:508-510`, `:597-599`
- `org-io/tests/persona_creation.rs:50-58`
- `org-io/tests/invitation_exchange.rs:49-63`, `:77-84`
- `org-io/tests/receive_step.rs:504-511`
disposition: fixed as T2-O1 in task `o2` (merged 2026-10-28): `node_for_test()` and `store_for_test` are deleted, and every test reaches its state through the handle's operations, over the transport where needed, through the helpers in org-io/tests/common/journey.rs and org-io/tests/common/handles.rs; the task report lists the few observations that are now indirect.

**finding-11**: guideline, medium — org-io: Doubles 1, "No mock of code the project owns". The double is not a Doubles 5 fault injector either: it "imitates service-specific content the code interprets", namely org-node's `MalformedMessage`.

`refuse_acknowledgement_listing_for_test` stands in for org-node, which the project owns. It makes `kept_acknowledgements` return `OrgNodeError::MalformedMessage` from inside org-io's own `acknowledge::listing`. Its doc comment says "no real store makes org-node refuse it". The test then asserts the report names that error ("malformed").

Locations:
- `org-io/tests/acknowledgements.rs:402-444` (the call at :418)
- `org-io/src/acknowledge.rs:373-376`
- `org-io/src/lib.rs:713-720`
disposition: fixed as T2-O2 in task `o2` by a content-free `fail_acknowledgement_listing_for_test` fault at org-io's own read. That hook recurred as T3-O2, and by owner ruling 2026-10-28 task `o3` removed it and its two tests; LLR-rav7dr, LLR-bv6wkz and LLR-3fxxpp now name the listing refusal as an untested clause and say why.

**finding-12**: guideline, low — org-io: Seams 2, "A deeper interface gets a direct test only once it has an LLR, and it earns one only where at least one of these holds: Unreachable … Combinatorial … Intrinsic".

Several tests read the handle's private `PublishState` through `unknown_write_for_test`. The same facts are visible at the handle:
- the record exists: the next write is refused with `WriteRefused::OutcomeUnknown`;
- it is cleared: the next write is not refused;
- its root, key and window: the `Verified::Settled` outcomes `Adopted`, `Deprecated`, `NotExecuted` and `StillUnknown`, which the same files already drive (for example `verification.rs:700-724`, where the window boundary is seen through settlement).

LLR-7y46be exists, but none of the three criteria holds.

Locations:
- `org-io/tests/chain_write.rs:163`, `:204`, `:218`, `:230`, `:243`, `:409`, `:444`
- `org-io/tests/verification.rs:286`, `:832`, `:852`
disposition: not fixed in task `o2` (the plan lists T2-O3 as not fixed); the accessor was deleted in round 3 under T3-O1 (task `o3`), and LLR-7y46be's record is now observed at the handle through verification's settlement, for example `the_unknown_write_holds_root_key_expected_epoch_and_last_valid_block` in org-io/tests/chain_write.rs.

**finding-13**: guideline, medium — org-io: Doubles 5, "The robustness rule is that every requirement gets abnormal-input tests (bad input, boundary values, resource exhaustion, dependency failure) beside its normal-case tests". This is also the floor rule for class C.

These LLRs state a failure or boundary outcome that no test drives:
- **LLR-8ze6cc**: "a failed write is `ExchangeError::Store`, the Persona reloaded". Untested. `invitation_exchange.rs:163-180` covers the normal case and org-node's refusal only.
- **LLR-898z24**: "A failed store write is `ReceiveError::Store` with `0x01` written and no answer sent". Untested. `receive_step.rs:519-585` holds two normal-case tests.
- **LLR-fwz3tb**: "no byte in time" (`VERDICT_TIME_LIMIT`, 10 s) is never driven. "any other byte" is checked only on `Verdict::from_byte`, not as a `NotDelivered` report (`send_fanout.rs:209-239`).
- **LLR-zzsqj4 / LLR-n649th**: the boundary "when the list is empty it deletes the Persona at once" at the start of a delivery is untested. `acknowledgements.rs:700-718` covers only the "none left after an acceptance" branch.
- **LLR-g6kaa8**: "a message addressed to the Person's key is never accepted by this node" is not driven. `person.rs:206-217` counts bound endpoints only.
- **LLR-5fchwt**: org-node's `NoProvisionalChange` refusal is asserted, but "returned with nothing written" is not. The store bytes are not compared (`propose.rs:102`), so the test would pass if that refusal wrote.
disposition: fixed as T2-O4 in task `o2`, each test mutation-checked: `a_failed_write_of_a_cancel_is_exchange_error_store_and_reloads_the_persona` (LLR-8ze6cc), `a_failed_store_write_of_a_request_is_receive_error_store_0x01_and_no_answer` (LLR-898z24), `another_byte_or_none_within_the_limit_is_reported_not_delivered` (LLR-fwz3tb), `a_message_addressed_to_the_persons_key_is_never_accepted` (LLR-g6kaa8), `a_delivery_that_starts_with_none_kept_deletes_the_persona_at_once` (LLR-n649th) and the nothing-written check on LLR-5fchwt. LLR-zzsqj4's verification-time empty list was unreachable; it recurred in round 3 and owner ruling 2026-10-28 (zzsqj4) amended the LLR to say so.

**finding-14**: guideline, low — org-io: Doubles 5, "every requirement gets abnormal-input tests … beside its normal-case tests".

These LLRs have only normal-case tests and no recorded robustness N/A:
- LLR-mas8vc (`person.rs:153-172`)
- LLR-4yte89 (`persona_creation.rs:120-133`)
- LLR-pn3hgc (`persona_creation.rs:241-270`)
- LLR-gaanm2 (`verification.rs:523-552`: no case of an adoption that leaves nothing out)
- the source-scan LLRs LLR-ctm86b, LLR-b97usg, LLR-k3zmqb, LLR-3ame3g, LLR-4k43dm, LLR-gh4rw3, LLR-5acgfg and LLR-6vtbn6 (`absences.rs:1336`, `:1366`, `:1387`, `:688`, `:1286`; `chain_read_rule.rs:86`; `rng_source.rs:154`; `verification.rs:236`)

The same draft records "robustness, N/A" for LLR-6sfub6 and LLR-jqb5z7. Its LLR-mas8vc, LLR-ctm86b and LLR-gaanm2 entries (lines 176-181, 102-108 and 852-870) have no such note.
disposition: not fixed in task `o2` (the plan lists T2-O5 as not fixed). Later rounds closed part of it: LLR-gaanm2's empty case in round 3 (`an_adoption_that_leaves_nothing_out_lists_no_discarded_change`, task `o3`) and LLR-mas8vc's abnormal case in round 4 (`a_refused_proposal_and_a_reopened_person_stay_provisional_with_their_fields`, task `o4`); the source-scan LLRs' missing N/A notes recurred as a low in round 6, carried open (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-15**: guideline, low — org-io: Seams 2, "A case reachable from the REQ interface with a boundary fake is tested there, even where a deeper seam would be easier".

`send_fanout.rs:235-238` checks "any other byte is no verdict" by calling `Verdict::from_byte` / `Verdict::as_byte` directly. That transport type is not the interface LLR-fwz3tb describes, which is `send_organisation`'s report. A recipient that writes the raw byte (a `dial_direct_as`-style test-support writer on the receiving side) reaches the case at that report.
disposition: fixed in task `o2` (the plan lists T2-O6 among the lows fixed): `another_byte_or_none_within_the_limit_is_reported_not_delivered` in org-io/tests/send_fanout.rs drives another verdict byte to `send_organisation`'s report (LLR-fwz3tb), failed with an unknown byte mapped to Refused.

**finding-16**: guideline, low — org-io: Owned services 1, "A service the project owns runs for real on the normal path", and Doubles 3, "Every fake has a contract test".

The project owns OrgRegistry (`on-chain/`). This change gave `FakeChain`'s `ChainWriter` new behaviour: `create_proxy`/`found_at`, plus `update` now recording its window. Its contract (`common/chain_writer_contract.rs`, run for real by `chain_writer_contract.rs` on chopsticks) covers only the founding's two steps. The update write's normal path runs only against `FakeChain`, with no contract assertion and no run against the real contract: `write_window`, then `update`, then the `WrongEpoch`-style refusal whose text `chain_write.rs:274` matches.

Locations:
- `org-io/src/test_support.rs:333-366`
- `org-io/tests/common/chain_writer_contract.rs:1-46`
- `org-io/tests/chain_write.rs:48-66`, `:166-185`
disposition: not fixed in task `o2` (the plan lists T2-O7 as not fixed); the update, window and read halves of `FakeChain` gained a contract in round 3 under T3-O3 (task `o3`): org-io/tests/common/chain_contract.rs, run over the fake by `fake_chain_meets_the_reader_and_writer_contract` and over the production chain outside the gate.

**finding-17**: guideline, low — org-io: UI 6, "A markup snapshot carries no `verifies:` annotation". The clause contradicts the floor, "every test carries `verifies:`".

The floor wins. The clause should be rewritten to say a markup snapshot is not a test the suite counts, or else to keep the annotation. No changed line obeys it, so the finding is against the clause.

Location: `.guardrails/templates/TEST_GUIDELINES.md:205`
disposition: not fixed — task `o2` lists T2-O8 as not fixed (docs/plans/2026-10-21-org-io-domain-api.md); UI 6 is unchanged in docs/TEST_GUIDELINES.md and applies to no changed file (no markup snapshot exists), carried open, not blocking.

**finding-18**: guideline, medium — org-node: Seams 1, "A test calls only an interface that a REQ or LLR describes. A private helper is covered through the interface above it." — `org-node/tests/absences.rs:612-623`, `org-node/tests/absences.rs:771`

`into_removed_is_called_only_from_verify_and_an_acknowledgement_removed_only_on_acceptance` names the private helper `RemovedStore::remove_acknowledgement` (`pub(crate)`, `src/removal.rs:136`):
- `callers_of("remove_acknowledgement")` expects its definition in `removal.rs`;
- `public_entry_points_reaching("remove_acknowledgement")` expects one public entry point;
- the sweep for `acknowledgements.<method>` exempts that function by name.

`verification_revocation_and_removal_do_no_io_and_read_no_clock` exempts `pub(crate)fnremove_acknowledgement(` by name too. That test also extends LLR-g2jeqd's "every public function" to `pub(crate)` signatures, which the file's own `is_public` (line 558) calls not public.

LLR-9jn3z8 says only that a kept acknowledgement "is removed in exactly one [place], `acknowledgement_accepted`". No LLR or SDD names `remove_acknowledgement`. Inlining the removal into `acknowledgement_accepted`, or renaming the helper, keeps every stated behavior and fails both tests.

Round 1 fixed the same pinning for `into_removed` (finding-5) by following private callers up to the public entry points. The acknowledgement half needs the same fix: anchor on `acknowledgement_accepted` and the public entry points, not on the helper's name.
disposition: fixed as T2-N1 in task `n2` (merged 2026-10-28): absences asserts acceptance only at `acknowledgement_accepted`, and the LLR-g2jeqd scan reads `pub fn` only; `a_kept_acknowledgement_is_removed_only_on_acceptance` in org-node/tests/absences.rs failed under a mutation and stays green when `remove_acknowledgement` is renamed.

**finding-19**: guideline, medium — org-node: Seams 1, "a test pinned to a function breaks on a refactor that moves it and keeps every behavior" — `org-node/tests/proposals.rs:498-528`

`a_personas_organisation_is_written_only_by_propose_member_and_record_founding_proxy` pins the implementation's text rather than an interface LLR-qqqg4b describes:
- It finds every `persona.org_id = ` in `service.rs` and credits it to the nearest `pub fn ` above it in the text. A write moved into a private helper defined after some other `pub fn` is credited to that function, so the test fails with the behavior unchanged.
- It requires the literal `org_id: Some(org_id)` inside `propose_member`, which pins a local variable's name.
- It lists `store.rs`, `revocation.rs` and `invitation.rs` by file.

Its "behaviour" half calls `build_invite` on a store with no record. That call is refused with `OrgNotHeld` before it could touch anything, so it cannot fail.

LLR-qqqg4b does not prescribe a source scan, as LLR-9jn3z8 does for `absences`. The rule should be checked at the operations: every store-changing `OrgService` operation, run on a bound store and on a founding store, leaves `persona_view().org_id` as it was. Alternatively, the scan should follow writers up to their public entry points, as `absences.rs::public_entry_points_reaching` does.
disposition: fixed as T2-N2 in task `n2`: the source scan is replaced by a behaviour test over every store-changing operation, `a_personas_organisation_is_set_once_and_kept_by_every_store_change` in org-node/tests/proposals.rs (LLR-qqqg4b), which failed with `cancel_invite` writing the Persona's org id.

**finding-20**: guideline, medium — org-node: Seams 2, "A deeper interface gets a direct test only once it has an LLR … Convenience is not on the list." — `org-node/tests/own_removal.rs:535-567`

`the_recipient_count_is_three_two_one_or_zero_and_their_order_is_fixed` calls `org_node::removal::recipient_count(n)` and `org_node::removal::recipients(&previous, &own, Option<&still>)` directly.

LLR-vq5d5n states the rule for choosing recipients but names neither function nor its signature. The test therefore freezes a `pub` interface no LLR records, which the clause's reason forbids ("an LLR freezes an interface, which should be a recorded design decision, not a side effect of a test").

Every case is reachable through `verify` with a store and a chain state:
- n = 0 to 6 candidates;
- removal by a match, with still-listed recipients first;
- removal by a kept notice.

The same test already does this for k = 0, and `own_removal.rs:487-517` does it for k = 3. Seven count cases and three order cases is short of the "roughly a dozen" the Combinatorial criterion gives as guidance. Either:
- move the cases to `verify`, or
- give the two functions an LLR that names them, citing which of criteria 2.1 to 2.3 holds.
disposition: fixed as T2-N3 in task `n2`: the recipient cases run through `verify`, and the helpers are `pub(crate)`; `a_removal_by_a_match_keeps_three_two_one_or_no_recipients` and `the_still_listed_come_first_on_a_match_and_all_ascending_on_a_notice` in org-node/tests/own_removal.rs (LLR-vq5d5n) each failed under a mutation.

**finding-21**: guideline, low — org-node: Seams 2, "A deeper interface gets a direct test only once it has an LLR" — `org-node/tests/organisation_key.rs:110-153` (annotation at :122)

`an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused` calls `OrgPublicKey::ensure_distinct_from` directly. This round re-pointed its annotation to `verifies: REQ-ech45n, LLR-9qavm7`.

LLR-9qavm7 states `propose_founding`'s refusal, not a contract of `ensure_distinct_from`. Only LLR-qghs95 mentions the function, as the means for `propose_updated_organisation`. The LLR that stated it before (LLR-sj7cd5) is superseded.

Criterion 2.1 (unreachable) does hold for the device half. What is missing is an in-force LLR stating that interface. The test should name the LLR that states `ensure_distinct_from`, which would mean adding one. Otherwise, keep only the device half and say why it is unreachable through `propose_founding`.
disposition: task `n2` fixed only the `ensure_distinct_from` doc comment. The missing LLR recurred in rounds 3 and 4 and, as T5-N2, was closed by owner ruling (round 5): task `n5` minted LLR-c759yd under SDD-swtd3w, and `an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused` in org-node/tests/organisation_key.rs verifies it under Seams 2 criterion 4.

**finding-22**: guideline, low — org-node: Doubles 3, "Every fake has a contract test. … A fake with no contract test drifts" (with Doubles 2: "Code calls the adapter, never the third-party interface directly, and the fake implements the adapter's interface. … The clock and randomness are fakeable in the same way.") — `org-node/tests/organisation_key.rs:161-179` (`OneByte`), `org-node/tests/proposals.rs:57-75` (`ConstRng`), `org-node/tests/proposals.rs:804-839` (`rc::TwoRng`)

Three new doubles fake randomness's content: one repeated byte, or a scripted two-byte sequence. `organisation_key.rs:158` calls `OneByte` "the boundary fake of randomness (Doubles 2)".
- Each implements the third-party `rand_core::RngCore`/`CryptoRng` traits directly, not an adapter the project owns.
- None has a contract test run against the real source.

The drift risk is small, but the stated rule is unmet. Two options:
- one shared fake with a contract test (the same assertions against it and `OsRng`: `fill_bytes` fills `dest`, `try_fill_bytes` is `Ok`);
- a recorded reason why a deterministic `RngCore` needs none.

`CountingRng` (`proposals.rs:78-99`, `person_store.rs:15-38`) wraps the real `OsRng` and only counts. It is not a fake, and its count is the requirement ("before any draw", LLR-rjphn4 and LLR-wqu26s; Doubles 4).
disposition: not fixed — task `n2` names no fix for it (docs/plans/2026-10-21-org-io-domain-api.md); task `n4` (C4-N2) later merged the copies into one `ConstRng` and one `CountingRng` in org-node/tests/support/mod.rs, but no contract test against `OsRng` was added. Carried open, not blocking.

**finding-23**: guideline, low — org-node: Doubles 5, "The robustness rule is that every requirement gets abnormal-input tests (bad input, boundary values, resource exhaustion, dependency failure) beside its normal-case tests." — `org-node/tests/verification.rs:630-650`

LLR-d9x6xf has one test, `changes_the_adopted_record_left_out_are_discarded_and_reported_in_order`, and it is a normal case. LLR-d9x6xf's rule is "an `Admit` is included exactly when the adopted record lists a member with the kept request's Member-as-a-group key and DevicePublicKey". No test has a member matching one key and not the other, so the "exactly when … and" boundary is untested. The robustness rule is also part of the compliance floor (class C).
disposition: not fixed in task `n2`; closed in round 4 as T4-N3 (task `n4`): `an_admit_whose_request_matches_a_listed_member_on_one_key_only_is_discarded` in org-node/tests/verification.rs (LLR-d9x6xf) failed with the rule matching on either key alone.

### docs/TEST_GUIDELINES.md

Governs: the test files under org-node/tests, org-io/tests, app/src-tauri/tests, app/tests and on-chain-client/tests that master...worktree-org-io-domain-api adds or modifies, with their test support (app/tests/fake-tauri.ts and the backend.json fixture, org-io/src/test_support.rs, org-node/tests/support) and the LLRs those tests verify, as the review reports state their scope. This root tailoring replaced the template as the governing file from round 3 on: task `a2` created it in round 2's fixes by owner ruling 2026-10-28 (T2-A6), as the template verbatim plus Seams 2 criterion 4, and task `a3` tailored it again in round 3 (T3-A1).

### Round 3

**finding-1**: guideline, medium — app+on-chain-client: Owned services 1, "A service the project owns runs for real on the normal path … A backend … that the codebase contains … runs for real in a REQ-level test of the normal case"; UI 3, "Fake only what is outside the codebase … A service the project owns runs for real on the normal path (Owned services 1)" — app/tests/fake-tauri.ts:39-51 (used by founding.propose.test.ts:17-18, notices.admission.test.ts:11-12, notices.discarded.test.ts:11-12, verification.view.test.ts:13-14, invite.confirm.test.ts:15-16)

The rendered feature tests replace `@tauri-apps/api/core` `invoke` and `@tauri-apps/api/event` `listen` with a fake. Every answer and event on the normal path then comes from a fake of the app's own Rust backend (`app/src-tauri`), not from the backend running for real. Examples: the admission proposed in notices.admission.test.ts:81-109, the founding in founding.propose.test.ts:47-57, and the verification at start in verification.view.test.ts:178-187. The fixture contract (contract.ts, backend.json, ipc.rs:1508-1829) holds the fake to the backend's shapes. That is the remedy Doubles 3 gives for a boundary fake. It does not stand in for the backend running for real, which is what Owned services 1 asks for an owned service on the normal path. ipc.rs:1508-1509 cites an "owner ruling of 2026-10-28" for this arrangement. The guidelines file does not record that ruling: its tailoring note says "Everything else in this file is the installed default, verbatim". So the rule as written is unmet. The fix is either for the normal-path feature tests to reach the real backend, or for the owner to tailor Owned services 1 / UI 3 to the ruling, with its reason.
disposition: T3-A1, a recurrence of T2-A2. Owner ruling 2026-10-28: tailor the clause. Fixed in task `a3` (merged 2026-10-28): docs/TEST_GUIDELINES.md now tailors Owned services 1 so that a webview test may use a double of the IPC process boundary when the shared fixture (app/tests/contract/backend.json, checked by app/src-tauri/tests/ipc.rs) pins every answer and event the double gives; UI 3 points to that exception, and a dated tailoring note records the ruling.

**finding-2**: guideline, low — app+on-chain-client: Doubles 3, "Every fake has a contract test. The same assertions run against the fake and against the real dependency" — app/tests/api.admit.test.ts:10-12

This test still builds its own `invoke` double (`vi.fn(async () => null)`) and a bare `listen: vi.fn()`. Neither goes through fake-tauri.ts, so backend.json holds neither, and no contract test checks them. The value it answers (`null` for `propose_admission`) happens to match the fixture's `recorded` entry. What the test asserts is the outgoing call, which LLR-gmb4p8 states, so Doubles 4 covers the assertion. The gap is only that this double sits outside the contract every other webview double sits under.
disposition: fixed in task `a3` (the plan lists T3 finding 2 among the lows fixed): app/tests/api.admit.test.ts's test of the outgoing call (LLR-gmb4p8) was reworked and failed with an extra argument added.

**finding-3**: guideline, low — app+on-chain-client: UI 3, "Find elements the way a user does: by accessible role, label and visible text, not by test identifier or class name" — app/tests/notices.admission.test.ts:92-94; app/tests/notices.discarded.test.ts:75-77, 165-166, 191-194, 226; app/tests/verification.view.test.ts:29-33, 36-40, 321

The confirmation's values are read as `getByText('Invited as').nextElementSibling` (likewise `'Request'`, `'DevicePublicKey'`, `'Handle'`). A notice is scoped through `getByText(text).parentElement`. "Last verifications" is read as `heading.nextElementSibling.querySelectorAll('li')`. The founding-writes list is read through `button.parentElement.textContent`. These walk the DOM's sibling and parent structure rather than finding elements by role or label (for example `getByRole('list')` / `listitem`, a labelled `term`/`definition`, or `within(getByRole(...))`). A markup refactor that changes nothing the user perceives, such as wrapping a value or reordering siblings, breaks them.
disposition: not fixed — task `a3` names no fix for it (docs/plans/2026-10-21-org-io-domain-api.md); it recurred as round 4's low against UI 3, also not fixed. Carried open, not blocking (app/tests/notices.discarded.test.ts).

**finding-4**: guideline, low — app+on-chain-client: Seams 1, "A test calls only an interface that a REQ or LLR describes" — app/tests/verification.view.test.ts:301-307; app/src-tauri/tests/ipc.rs:1767

- verification.view.test.ts:301-307 calls the api.ts wrapper `proposeFoundingOrganisation` directly and asserts the arguments it invokes with. No LLR names that wrapper: LLR-6ff735 names the IPC command, and LLR-rhc555, which this test cites, names only `writeOrganisationToChain({ founder })`. The two lines also assert nothing LLR-rhc555 states.
- ipc.rs:1767 drives `commands::verification_outcomes` to produce the `verification-result` payload for the fixture entry `adopted_own_update`. No LLR in either design file names `verification_outcomes`. Two LLR-named interfaces would give the same evidence: `verify_and_send` with `outcomes_for_verification`, or `run_periodic_verification`'s emitter, as periodic.rs does.
disposition: partly fixed in task `a3` (the plan lists T3 finding 4, webview half): the test in app/tests/verification.view.test.ts now drives the Write to chain button and checks `writeOrganisationToChain`'s arguments (LLR-rhc555), failed with those arguments changed. The ipc.rs half (`commands::verification_outcomes` behind the fixture entry) was not changed — carried open, not blocking.

**finding-5**: guideline, low — app+on-chain-client: Doubles 5, "The robustness rule is that every requirement gets abnormal-input tests (bad input, boundary values, resource exhaustion, dependency failure) beside its normal-case tests" — on-chain-client/tests/write_compose.rs:326 (and 585/595/613); app/src-tauri/tests/periodic.rs:248; app/src-tauri/tests/receiver_events.rs:1143, 1316; app/tests/verification.view.test.ts:129, 135

These LLRs have only normal-case tests citing them:
- LLR-95puh9 (on-chain-client): every test citing it is a normal resume or contract case. The abnormal mapping cases (a mapping read holding another account, a failed or approval-only mapping dispatch) are tested, but only under LLR-chy957 (write_compose.rs:606-645, 855-871).
- LLR-fe7umz (app): one normal test. It has no case where the request fails or is refused as a whole across runs, and no case of a run after `adopted` (which the LLR names beside `in_step`).
- LLR-t9rdf8 (app): no abnormal case.
- LLR-njb6mr (app): only the seven known statuses. Contrast LLR-mfzr6z, which has the "a result it does not know" case at verification.view.test.ts:78-81.

The behaviour is mostly right and partly covered under other IDs, so this is low. Each needs an abnormal case annotated with its own ID.
disposition: not fixed as a whole — task `a3` names no fix for it (docs/plans/2026-10-21-org-io-domain-api.md). Its LLR-njb6mr part has an abnormal case since `a3` (C3 finding-7: a status the view does not know is labelled unverified, in app/tests/verification.view.test.ts); the LLR-95puh9, LLR-fe7umz and LLR-t9rdf8 parts are carried open, not blocking.

**finding-6**: guideline, low — app+on-chain-client: Doubles 5, "… abnormal-input tests … beside its normal-case tests" — app/src-tauri/tests/verification.rs:609-627 and 818-841

`a_behind_result_is_requested_once_with_no_user_step` (the normal case of LLR-nkg8sb) and `a_behind_request_a_device_refuses_is_reported_failed_for_that_device` (its abnormal case) build the same scenario: `admitted`, then `write_org`, then `epoch_ahead`. The second asserts that the one request is `failed`, "refused by the recipient". So the normal-case test also passes only with the request failing. The `result: sent` branch of LLR-nkg8sb is never reached at `verify_and_send`. That mapping is covered one seam down, at `attach_report` (verification.rs:784-796, LLR-k4rxyt), so the code looks right. But the requirement's normal case has no test at its own interface, and the normal-case test's name says more than its assertions check.
disposition: not fixed — task `a3` names no fix for it (docs/plans/2026-10-21-org-io-domain-api.md); it recurred as low 4 of round 4 (LLR-nkg8sb), also not fixed. Carried open, not blocking (app/src-tauri/tests/verification.rs).

**finding-7**: guideline, medium — org-io: Seams 1, "A test calls only an interface that a REQ or LLR describes", and Seams 2, "A case reachable from the REQ interface with a boundary fake is tested there, even where a deeper seam would be easier" — `org-io/src/test_support.rs:662` (`unknown_write_for_test`), `org-io/src/lib.rs:746` (`delivery_tasks_for_test`), `org-io/src/test_support.rs:654` (`draw_for_test`). The change adds three test-only accessors that read the handle's private state. No LLR describes any of them; LLR-5acgfg names only `with_rng_for_test`, and LLR-6vtbn6 names only the clock hooks. The tests use them as evidence:
  - `chain_write.rs:217`: `the_unknown_write_holds_root_key_expected_epoch_and_last_valid_block` is LLR-7y46be's only test of the record's contents. It reads the `PublishState` map directly. Those contents can be seen at `verify_organisation`, with the fake chain: the last valid block shows as NotExecuted after 65 blocks, and the root and key show as Adopted against Deprecated.
  - `chain_write.rs:163, 204, 230, 243, 418, 453` and `verification.rs:290, 829, 849` also read the map. Most of these assertions already have an observable twin, a `WriteRefused::OutcomeUnknown` refusal or its absence.
  - `acknowledgements.rs:74,76` is the only evidence in LLR-zzsqj4's test for "at most one delivery task runs per Persona". A second task would show at the boundary as a second send per round. `verification.rs:403,406` likewise stands in for "starts no acknowledgement delivery".
  - `rng_source.rs:162-165` tests LLR-5acgfg through raw draws. The LLR's stated purpose is a reproduced invite identifier or key pair, which the handle's own operations show.
disposition: fixed as T3-O1 in task `o3` (merged 2026-10-28): `unknown_write_for_test`, `delivery_tasks_for_test` and `draw_for_test` are deleted, and each fact is observed at the handle: LLR-7y46be's record through settlement (`the_unknown_write_holds_root_key_expected_epoch_and_last_valid_block`), LLR-zzsqj4's single task by no second acknowledgement (`a_removed_outcome_starts_one_delivery_task_and_returns_without_awaiting_it`), and the rng by reproduced key pairs (`a_seeded_source_reproduces_a_draw_under_test_support`, LLR-5acgfg), each mutation-checked.

**finding-8**: guideline, medium — org-io: Doubles 1, "No mock of code the project owns". Doubles 2 adds: "Fakes at the codebase boundary only". — `org-io/src/lib.rs:740` (`fail_acknowledgement_listing_for_test`), used at `acknowledgements.rs:430` and `acknowledgements.rs:558`.
  - The hook makes org-io's own read of org-node's `kept_acknowledgements` fail. org-node is a unit of this codebase.
  - Doubles 5 allows a fault injector only "at any boundary". The boundary Doubles 2 means is the codebase's, a service or resource outside the codebase behind one owned adapter. A switch inside org-io's call into another owned unit is not that boundary.
  - LLR-bv6wkz's note of 2026-10-28 records the hook. Recording it in an LLR does not make it a codebase-boundary double.
  - The failing tests are `a_refused_listing_is_the_rounds_failure_and_the_next_due_round_sends_again` and `a_failed_read_of_the_list_at_the_deletion_is_reported_and_retried_hourly`.
  - Seams 2.1 names the route the file gives for such a case: a direct test at a deeper interface that has an LLR, where the case is "Unreachable ... without faking code the project owns".
disposition: T3-O2, a recurrence of T2-O2. Owner ruling 2026-10-28: drop the test and the hook, and amend LLR-rav7dr and LLR-bv6wkz to say the listing refusal cannot be produced, so it is not tested; the report-and-retry code stays. Done in task `o3`: both listing-fault tests, `fail_acknowledgement_listing_for_test` and the `fail_listing` field are removed, and LLR-rav7dr, LLR-bv6wkz and LLR-3fxxpp name the untested clause and why.

**finding-9**: guideline, medium — org-io: Doubles 3, "Every fake has a contract test. The same assertions run against the fake and against the real dependency" — `org-io/src/test_support.rs:247-303` and `306-366`, with `tests/common/chain_writer_contract.rs:7-20`.
  - The contract this change adds covers only the founding's `create_proxy` and `found_at`.
  - `FakeChain`'s other halves interpret service-specific content and have no contract test:
    - `StateReader::read_state` / `finalised_block`: `parse_refusing` returns `OrgNodeError::InvalidOrgPublicKey`, `state_hidden` returns `Ok(None)`, and `epoch_ahead` returns a shifted epoch.
    - `ChainWriter::write_window`: a window born at the finalised block.
    - `ChainWriter::update`: `ChainSlots::apply_update`'s stale-epoch refusal, returned as text.
    - `SignatorySetReader`.
  - The change edits these halves (`update` now records its window, `write_window` counts and hangs). It also writes most of `verification.rs`, the update half of `chain_write.rs` and `signatory_rule.rs:213` on them.
disposition: fixed as T3-O3 in task `o3`: org-io/tests/common/chain_contract.rs is one reader-and-writer contract (finalised block and window, missing state, signatory sets, an update moving the epoch, a stale update refused with state unchanged), run over `FakeChain` in the gate by `fake_chain_meets_the_reader_and_writer_contract` (mutation-checked) and over the production chain by `the_production_chain_meets_the_reader_and_writer_contract` outside it; the real half was not run here (no chopsticks).

**finding-10**: guideline, low — org-io: Doubles 3, "wherever the real dependency is reachable in the test environment, and are skipped where it is not" — `org-io/tests/chain_writer_contract.rs:50-52`. The production half of the founding contract does not skip when the fork is unreachable. It panics at `spawn_fork().await.expect(...)` and at `forge build`. It stays out of the gate only because `verify_commands` does not name the target, so an unfiltered `cargo test -p org-io --features test-support,dev-seed` fails instead of skipping.
disposition: fixed as T3-O4 in task `o3`: both chopsticks tests in org-io/tests/chain_writer_contract.rs skip with a stderr reason when no fork is available; the skip was confirmed here (`ChopsticksBinaryMissing`).

**finding-11**: guideline, medium — org-io: Test guidelines (preamble), "The compliance floor the skills state wins any conflict". The floor item is "every test fails when the behavior it `verifies:` breaks". — `org-io/tests/chain_write.rs:234-236`.
  - `a_refused_write_records_no_unknown_write` is annotated `verifies: LLR-a66ufr`. It sets `write_failing`, so the update itself fails and the window read is untouched. It then checks that no unknown write is recorded and that the next write is not refused as unknown.
  - LLR-a66ufr states the single `write_window()` read, its pass to `update`, and its failure. A break there leaves this test green.
  - The property it checks is LLR-d3farr's and LLR-7y46be's: only a timeout records an unknown write, and the record is cleared only by LLR-zey4mg.
disposition: fixed as T3-O5 in task `o3`: `a_refused_write_records_no_unknown_write` in org-io/tests/chain_write.rs now verifies LLR-d3farr and LLR-7y46be, and failed with an unknown write recorded on a refused update.

**finding-12**: guideline, medium — org-io: Test guidelines (preamble), "The compliance floor the skills state wins any conflict". The floor item is "every test fails when the behavior it `verifies:` breaks". — `org-io/tests/app_boundary.rs:229-231`.
  - This change rewrote `the_app_calls_no_adopting_operation_and_builds_no_service` and moved its annotation from LLR-7pj5af to LLR-kz9hj4.
  - The test scans the app's sources (`app/src-tauri/src`) for `check_received`, org-node `verify`, `OrgService::new` and the like.
  - LLR-kz9hj4 states `OrgIo::next_received`'s behaviour: the check, the write before the report, no chain read, and the refusal and store-failure variants. It says nothing about the app's code. A break of LLR-kz9hj4 leaves this scan green.
disposition: fixed as T3-O6 in task `o3`: a new LLR-ntag3g under SDD-z3ychz states the app-source scan's rule, and `the_app_calls_no_adopting_operation_and_builds_no_service` in org-io/tests/app_boundary.rs verifies it, failed with `OrgService::new(` planted in the app.

**finding-13**: guideline, low — org-io: Test guidelines (preamble), floor item "every test fails when the behavior it `verifies:` breaks", and Seams 1, "A test calls only an interface that a REQ or LLR describes" — `org-io/tests/chain_write.rs:542-544`.
  - `each_write_refusal_displays_human_readable_text` builds each `WriteRefused` value by hand and checks its `Display` text. It is annotated `verifies: LLR-48me7m, LLR-mhn4n5, LLR-5tpdxw`.
  - LLR-48me7m and LLR-mhn4n5 state when each refusal is returned, not its text. A break in when `NotHeld`, `NoProxyAccount`, `Stale` or `OutcomeUnknown` is returned cannot redden a test that never calls `write_organisation_to_chain`.
  - Only LLR-5tpdxw's "saying the proxy exists and naming it" is a text the test checks.
disposition: fixed as T3-O7 in task `o3`: `each_write_refusal_displays_human_readable_text` in org-io/tests/chain_write.rs verifies LLR-5tpdxw only, and failed with the Store refusal's proxy not named.

**finding-14**: guideline, medium — org-io: Doubles 5, "The robustness rule is that every requirement gets abnormal-input tests (bad input, boundary values, resource exhaustion, dependency failure) beside its normal-case tests". The floor item is "class B and C: abnormal-input tests for every REQ and LLR". — These behavioural LLRs have no abnormal or boundary case among the tests that cite them:
  - LLR-zzsqj4 (`acknowledgements.rs:58`): normal case only. The failed write of a `Removed` outcome, which starts no delivery, is tested at `verification.rs:393` but cites only LLR-zcw3xv.
  - LLR-4yte89 (`persona_creation.rs:117`): normal case only. `CreateError::FoundingNotListed`, which the LLR states, is reached by no test. org-node's refusal of a field is tested at `persona_creation.rs:79` under LLR-kna4zj only.
  - LLR-gaanm2 (`verification.rs:531`): normal case only. No test checks the boundary of an adoption that leaves nothing out (`discarded` empty).
  - LLR-pn3hgc (`persona_creation.rs:242`): normal case plus source scans. No test gives an Organisation name org-node refuses.

note: These are things the file does not state as rules. None blocks merge.
- Two more LLR clauses are reached by no test: `ReceiveError::NoOrganisation` (LLR-kz9hj4) and org-node's refusal in LLR-6ux6yw ("no such acknowledgement" writes nothing). LLR-xwz3nu's `AlreadyReceived`, `InviteUsedByAnotherDevice` and "another byte" answers are untested; only `Refused` and `NotDelivered` are. The Mutants section names no command, so these are not a Mutants 1 finding.
- The source-scan LLRs (LLR-3ame3g, LLR-k3zmqb, LLR-b97usg, LLR-ctm86b, LLR-gh4rw3, LLR-4k43dm, LLR-4usfu2, LLR-sn5ded's scan half) carry no "robustness, N/A" note. The draft gives that note for LLR-6sfub6 and LLR-jqb5z7.
- `org-io/tests/node_stories.rs` now holds no test but is still a `verify_commands` target. No journey test walks the demo journey: Seams 4 has nothing to apply to. The journey is covered step by step by `common/journey.rs`'s helpers inside other LLRs' tests.
- `store_writes_to` (a recording hook in `store_file.rs`) and `keychain.calls()` / `chain.founding_calls()` are used as ordering and count evidence (LLR-6ux6yw, LLR-3fxxpp, LLR-5tpdxw, LLR-8w2z64). Read as observations of file-system and boundary effects that the LLRs state, they fit Doubles 4.
- `acknowledgements.rs:792-793` asserts the same `acknowledgement_report(..).is_none()` twice under two messages.
- Stale headers name superseded items as current: `receive_step.rs:6-10` (LLR-9arvqj), `acknowledgement_inbound.rs:6-12` (LLR-wvgec4, LLR-unqfj5) and `send_fanout.rs:6-11` (LLR-znhmh2, LLR-r9sh6k).
- Makefile: the TEST guidelines say nothing about the coverage floors or recipes. `ORG_IO_COVERAGE_ARGS` matches the verify line's targets, plus `dev_seed_env` under `dev-seed`.
- The UI section has nothing to apply to (org-io has no UI). Test runs 1 binds development practice, not the diff.

disposition: T3-O8, fixed in task `o3` except as ruled: LLR-gaanm2 by `an_adoption_that_leaves_nothing_out_lists_no_discarded_change`; LLR-pn3hgc by `an_empty_and_a_spaced_non_ascii_organisation_name_are_reported_unchanged`; LLR-4yte89 partly, by `a_founding_is_proposed_with_the_chain_failing_and_makes_no_chain_call`, with `FoundingNotListed` recorded as unreachable without faking org-node. The LLR-zzsqj4 part recurs T2-O4; owner ruling 2026-10-28 (zzsqj4): LLR-zzsqj4 is amended to say the empty list at verification time is impossible, and the branch is verified on a resume by `a_delivery_that_starts_with_none_kept_deletes_the_persona_at_once`. Of the reviewer's notes, the stale headers and the duplicate `is_none()` assert were fixed; the unreached `NoOrganisation`, LLR-6ux6yw and LLR-xwz3nu clauses were not.

**finding-15**: guideline, low — org-node: Seams 1, "A test calls only an interface that a REQ or LLR describes … a test pinned to a function breaks on a refactor that moves it and keeps every behavior" — `org-node/tests/revocation.rs:636`
`acknowledgement_signed_bytes_are_the_domain_then_the_tuple` is new in `mod acknowledgement_domain`. It counts the textual occurrences of `ACK_DOMAIN` in `src/` and asserts `[("revocation.rs", 2)]`, meaning exactly two mentions: the definition and `signed_bytes`.
- **What LLR-bs6yxf states:** "No other message org-node signs or verifies begins with `ACK_DOMAIN`." It says nothing about how many times the constant is named, or in which file.
- **Why that matters:** a refactor that keeps every behavior would fail this test. Examples are a helper that calls `signed_bytes`'s prefix logic, a `const` re-export, or a doc test in `revocation.rs`.
- **Duplication:** the same file already pins the signed bytes at `revocation.rs:169` (`acknowledgement_signed_bytes_are_the_domain_then_the_postcard_tuple`). The new test repeats that check and adds the count.
disposition: not fixed in round 3 (the TEST org-node review had nothing above low, so no task took its lows); it recurred as low 10 of round 4 and was fixed in task `n4`: `acknowledgement_signed_bytes_are_the_domain_then_the_tuple` in org-node/tests/revocation.rs (LLR-bs6yxf) now fails on a second const holding the same bytes, which the counting version passed.

**finding-16**: guideline, low — org-node: Seams 2, "A deeper interface gets a direct test only once it has an LLR" — `org-node/tests/organisation_key.rs:123` (calls at :143, :149, :154)
`an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused` calls `OrgPublicKey::ensure_distinct_from` directly. The diff edits it: a re-pointed comment and the new `DuplicateOrgPublicKey` error.
- **The mismatch:** it is annotated `verifies: REQ-ech45n, LLR-9qavm7`. LLR-9qavm7 states `propose_founding`'s contract and names no `ensure_distinct_from` interface.
- **The only mention:** the one LLR that names the function, LLR-qghs95, does so in a parenthesis, as the mechanism of `propose_updated_organisation`.
- **What is right:** criterion 2.1 ("Unreachable") holds for the device half, as the comment argues.
- **What is missing:** the direct seam needs an LLR that states `ensure_distinct_from`'s own contract, and the test should cite it. As it stands, the test is credited to an LLR whose interface it never calls.
disposition: not fixed in round 3; it recurred as low 5 of round 4 (re-annotated to REQ-ech45n in task `n4`) and then as T5-N2, closed by owner ruling: task `n5` minted LLR-c759yd under SDD-swtd3w, and `an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused` verifies it under Seams 2 criterion 4.

**finding-17**: guideline, low — org-node: Doubles 3, "Every fake has a contract test. The same assertions run against the fake and against the real dependency" — `org-node/tests/organisation_key.rs:160`, `org-node/tests/proposals.rs:59`, `org-node/tests/proposals.rs:80`, `org-node/tests/proposals.rs:839`, `org-node/tests/person_store.rs:15`
The diff adds five randomness doubles, and none has a contract test against `OsRng`:
- `OneByte` (organisation_key.rs)
- `ConstRng`, `CountingRng` and `rc::TwoRng` (proposals.rs)
- `CountingRng` (person_store.rs)

Doubles 2 allows them ("The clock and randomness are fakeable in the same way"), and they are fakes rather than fault injectors, since nothing in them fails. Doubles 3 asks that each run the same assertions as the real source, for example that `fill_bytes` fills the whole of `dest` and that `try_fill_bytes` returns `Ok`. Each double is correct as written, so this is a gap rather than a defect. It is the same pattern as the `ConstRng` and `TwoRng` already on master.
disposition: not fixed — no task took round 3's org-node TEST lows (docs/plans/2026-10-21-org-io-domain-api.md); task `n4` (C4-N2) reduced the doubles to one `ConstRng` and one `CountingRng` in org-node/tests/support/mod.rs, with no contract test against `OsRng`. Carried open, not blocking.

**finding-18**: guideline, low — org-node: Doubles 5, "The robustness rule is that every requirement gets abnormal-input tests (bad input, boundary values, resource exhaustion, dependency failure) beside its normal-case tests" — `org-node/tests/organisation_request.rs:174`, `org-node/tests/own_removal.rs:286`, `org-node/tests/own_removal.rs:307`
Two class C LLRs have only normal-case tests annotated to them:
- **LLR-7kdhst** (a listed Device's organisation request is answered):
  - Its one test is `a_listed_requester_is_answered_with_the_full_record_and_its_key`.
  - Abnormal cases for a listed requester do exist: refused addresses from CAROL at `organisation_request.rs:216`, and the oversized answer at `:225`.
  - Those tests are annotated LLR-283x6f / LLR-j338wq and LLR-xc9c6w instead.
- **LLR-2d5a7m** (`into_removed` keeps nothing else):
  - Both of its tests drive the normal rich-store removal.
  - Its boundary case, a removal with no recipient and so an empty `acknowledgements`, is exercised at `own_removal.rs:569` but credited to LLR-vq5d5n only.

The abnormal behavior is tested, but not under these LLRs, so the class C trace for them shows normal cases only. A milder case of the same thing: LLR-b2hgd4's only annotated test is normal. Its refusals ("recording nothing") are annotated LLR-mq7n6u and LLR-f3hru6.
disposition: partly fixed later: the LLR-7kdhst half recurred as low 7 of round 4 and task `n4` added `a_listed_requesters_invalid_addresses_are_refused_and_answered_nothing` (LLR-7kdhst) in org-node/tests/organisation_request.rs. The LLR-2d5a7m and LLR-b2hgd4 halves are not fixed — carried open, not blocking (docs/plans/2026-10-21-org-io-domain-api.md).

### Round 4

**finding-1**: guideline, low — app+on-chain-client: UI 1, "The REQ interface of a UI is what the user perceives and does ... a test that names them breaks on a refactor that keeps every behavior". At app/tests/verification.view.test.ts:196-220 (LLR-zkpsw5) and app/tests/notices.discarded.test.ts:240-255 (LLR-fsk4g2). Both LLRs open with a clause the user can perceive: "the view shows no action to request an Organisation" and "offers no action with any of them". The tests verify that clause only by grepping `app/src` source text for identifiers (`offersRequest|requestOrganisation|'request_organisation'`, `sendOrganisation|'send_organisation'`). The test still passes if a "Request" or "Retry" button is wired under any other name. It fails on a rename that keeps every behaviour. No test renders a `behind` result or an own adoption with sends and checks the page for an action. The source clause each LLR also states is met. The gap is the perceivable clause.
disposition: not fixed — carried open, not blocking (the TEST app + on-chain-client loop ended at round 4 with nothing above low, and task `a5` took only its low 5; docs/plans/2026-10-21-org-io-domain-api.md); the source clause of LLR-zkpsw5 and LLR-fsk4g2 stays met by app/tests/verification.view.test.ts.

**finding-2**: guideline, low — app+on-chain-client: UI 3, "Find elements the way a user does: by accessible role, label and visible text, not by test identifier or class name". Locations:
- app/tests/verification.view.test.ts:29-40 (`heading.nextElementSibling`, `querySelectorAll('li')`, `button.parentElement`) and :319
- app/tests/notices.admission.test.ts:92-94
- app/tests/notices.discarded.test.ts:76, 165-166, 191-194, 226 (`getByText(label).nextElementSibling`, `parentElement`)
- app/tests/personas.list.test.ts:54, 63 (`closest('tr')`)

These locate elements through the markup's sibling, parent and tag structure, not through role or label. The roles exist for them: `list`/`listitem`, `row`/`cell`, a definition term and its value, a `group`/`region` per notice. Moving a label into a wrapper or swapping a `<ul>` for another list element breaks the tests while the user sees the same thing. This is not a test id or a class name, so it is low.
disposition: not fixed — carried open, not blocking (the TEST app + on-chain-client loop ended at round 4 with nothing above low, and task `a5` took only its low 5; docs/plans/2026-10-21-org-io-domain-api.md); it repeats low 8 of round 2 and low 3 of round 3 (app/tests/notices.discarded.test.ts).

**finding-3**: guideline, low — app+on-chain-client: UI 4, "List the states each screen can be in and test each: loading, empty, error, partial, stale ...". The screens this diff changes have pending states that no test enters:
- app/src/lib/components/CreateOrg.svelte:26-54, 84-104: `busy`, the fields disabled and the button reading "Proposing…"
- Invite.svelte:185-232: `issueBusy`, `readBusy`, `proposeBusy`, `requestBusy` ("Issuing…", "Reading…", "Proposing…")
- Admit.svelte:91-97: `admitBusy`
- PersonaList.svelte:68-71: `loading`

The rendered tests (founding.propose, invite.confirm, notices.admission, notices.discarded, personas.list) answer each command at once, so no test holds a command pending to check the disabled form or its label, or that a second press sends nothing. The code may well be right; this is a gap in the tests.
disposition: not fixed — carried open, not blocking (the TEST app + on-chain-client loop ended at round 4 with nothing above low, and task `a5` took only its low 5; docs/plans/2026-10-21-org-io-domain-api.md); the pending states of app/src/lib/components/CreateOrg.svelte and its siblings have no test.

**finding-4**: guideline, low — app+on-chain-client: Doubles 5, the robustness rule, "every requirement gets abnormal-input tests ... beside its normal-case tests". Also the floor: every test fails when the behavior it `verifies:` breaks.
- At app/src-tauri/tests/verification.rs:620-638 and :827-850 (LLR-nkg8sb), the normal-case test `a_behind_result_is_requested_once_with_no_user_step` and the "Abnormal" test `a_behind_request_a_device_refuses_is_reported_failed_for_that_device` run the same set-up. Both use `admitted()`, `write_org`, `epoch_ahead` and `verify_and_send`. The second asserts that the request comes back `failed`, "refused by the recipient". So the refusal labelled abnormal is just what this set-up always gives, and the normal-case test asserts only `device`. It passes whether the request is `sent` or `failed`.
- Likewise :503-516 (LLR-es49wp) asserts the send's `device` and not its `result`.
- LLR-cugcu6's note says "`verify_and_send`'s own tests (LLR-es49wp, LLR-nkg8sb) cover a delivered send". No test at `verify_and_send` requires a `sent` entry, for a request or a send. The `sent` mapping is covered only below, at `device_reports` and `attach_report`.
- The abnormal test injects nothing, so it would also pass if the request were refused for the wrong reason.
disposition: not fixed — carried open, not blocking (the TEST app + on-chain-client loop ended at round 4 with nothing above low, and task `a5` took only its low 5; docs/plans/2026-10-21-org-io-domain-api.md); the `sent` mapping stays covered below `verify_and_send`, at `attach_report` (LLR-k4rxyt) in app/src-tauri/tests/verification.rs.

**finding-5**: guideline, low — app+on-chain-client: Seams 1, "A test calls only an interface that a REQ or LLR describes". Also the floor: every test fails when the behavior it `verifies:` breaks.
- At app/tests/revoke.validate.test.ts:79-85, `it('takes the member identifier alone')` cites LLR-csbs5v. LLR-csbs5v (app/docs/architecture/2026-10-05-decomposition.md:1299) states only the trim, the `0x` strip, the width and the alphabet. "Takes no peer address, allowed in every mode" was LLR-zqy3tq's clause, retired 2026-10-20 (DRAFT:1268-1274). So the test verifies behaviour no in-force item states.
- Its `expect(validateRevokeInput.length).toBe(1)` cannot fail for the claim it makes. The function takes one object, so its arity is 1 whatever fields the object carries. Adding a peer-address field back to the input leaves the assertion green.
disposition: fixed in task `a5` (merged 2026-10-28; the plan lists TEST app low 5): the arity test is replaced by the acceptance LLR-csbs5v states, a test in app/tests/revoke.validate.test.ts that accepts a 64-character hexadecimal member id and failed with an all-'b' id refused.

**finding-6**: guideline, low — app+on-chain-client: Doubles 3, "The same assertions run against the fake and against the real dependency ... A fake with no contract test drifts: it accepts what the real dependency rejects". At on-chain-client/tests/write_compose.rs:257-271 (`a_mapping_read_holding_another_account_is_the_map_account_step`, LLR-chy957), `FakeOps::on_chain(Some(other), ...)` answers `mapped_account` with an account other than the proxy. The contract in on-chain-client/tests/support/write_reads_contract.rs:19-40 checks only three states against `SubxtWriteOps`: untouched, funded and unmapped, mapped to itself. No assertion of the contract covers the "mapped to another account" answer. The design says pallet-revive maps an account only from that account, so the real chain may never give this answer. If the branch is defensive, the fake state it rests on is still unchecked.

note: these are recorded for the reader, not flagged:
- events::receive_iteration (receiver_events.rs:1347-1404, LLR-j9m88u) is driven with scripted `next` and `verify` closures, and its test asserts the order they are called in. These stand in for org-io, which the project owns. I read this as Doubles 4 (the order is what LLR-j9m88u states) at a Seams 2 criterion 1/4 interface, so I raise no finding. A stricter reading of Doubles 1 ("no assertion on how owned code was called") would.
- backend.json's `verification-result.adopted_own_update` carries `sends: []`. The TS shape rule refuses a non-empty array against an empty expected one, so the fake can never emit an own adoption with sends. No rendered test can then show the page's per-Device outcomes. The Rust check passes against the real non-empty `sends` only because of the direction its subset check runs in.
- The app's normal-path tests run over org-io's `test_support::FakeChain`. Its contract test is org-io's and outside these paths. The OrgRegistry contract the codebase contains never runs on the app's normal path. Under Owned services 1 that is a test-environment matter; on-chain-client's chopsticks targets are where the real chain is met.
- Some tests assert function arity: `proposeAdmission.length` (api.admit.test.ts:30) and `verifyOrganisations.length` (verification.view.test.ts:160). In both, the `toHaveBeenCalledWith` beside the arity check is what verifies the LLR. The arity adds nothing and would break on an optional parameter.
- ipc.rs:1197-1201 asserts only that the refusal is not EXACTLY_ONE_TARGET ("whatever it then reports"). The call is not shown to reach org-io.
- Several Rust tests wait on wall-clock sleeps and polling: ipc.rs:780 and :827, `kept_requests`, verification.rs `admitted()`. No clause covers this; it is a flakiness risk.

disposition: not fixed — carried open, not blocking (the TEST app + on-chain-client loop ended at round 4 with nothing above low, and task `a5` took only its low 5; docs/plans/2026-10-21-org-io-domain-api.md); the contract in on-chain-client/tests/support/write_reads_contract.rs still pins three mapping states. The reviewer's notes raised no finding.

**finding-7**: guideline, medium — org-io: Floor, "every test fails when the behavior it `verifies:` breaks" (stated under the file's preamble: "The compliance floor the skills state wins any conflict").
- Where: org-io/tests/chain_write.rs:574-611 (`fake_chain_meets_the_reader_and_writer_contract`, `verifies:` at :581) and org-io/tests/chain_writer_contract.rs:125-143 (`the_production_chain_meets_the_reader_and_writer_contract`, `verifies:` at :127).
- What: both tests cite `LLR-t2pve7, LLR-ce42ce, LLR-a66ufr, LLR-s7rq5x, LLR-7qdjqn`. Those LLRs state the behaviour of `verify_organisation` (block then state read, outcome mapping), its `Unverified` on a failed read, `write_organisation_to_chain`'s single window read passed to `update`, the write's report, and `is_own_admin`. Neither test calls any of those methods. They call `StateReader`, `ChainWriter` and `SignatorySetReader` directly, to show that the fake answers as the production halves do (Doubles 3).
- Why it matters: if `verify_organisation` read the state twice, `write_organisation_to_chain` read two windows, or `is_own_admin` mis-ruled, both tests would still pass. The annotations credit these LLRs with tests that cannot fail when their behaviour breaks. They should cite what the contract does verify: the seam's own contract, under an LLR that states it.
disposition: fixed as T4-O1 in task `o4` (merged 2026-10-28): three new LLRs state the seams' own contract, LLR-js7wgk (StateReader, SDD-4bbt47), LLR-j4nkx5 (ChainWriter, SDD-mk39e5) and LLR-bwerd2 (SignatorySetReader, SDD-f4khqn), and both `fake_chain_meets_the_reader_and_writer_contract` and `the_production_chain_meets_the_reader_and_writer_contract` now cite them.

**finding-8**: guideline, medium — org-io: Doubles 5, "A fake, with a contract test: a double that imitates service-specific content the code interprets", and Doubles 3, "Every fake has a contract test".
- Where: org-io/src/test_support.rs:536, :556, :601-602 (`MemoryKeychain::refuse_reads_as`, commented as "a fault injector"). Used at org-io/tests/persona_creation.rs:214-217 (`a_bind_that_fails_while_receiving_is_reported_and_undoes_no_creation`, LLR-jvx6j9).
- What: the switch makes reads alone fail with keyring's `NoStorageAccess` or `PlatformFailure` while writes succeed. Those are typed library errors that org-io maps (LLR-8erfbu: `NoStorageAccess` becomes `KeychainError::Locked`), and the test asserts that mapped `KeychainError::Locked { device_key }`. By Doubles 5 this is service-specific content the code interprets, so it is a fake, not a fault injector.
- Why it matters: keychain_contract.rs pins only a locked store that refuses every read, write and delete (`locked_contract`). The state "takes the write, refuses the read as locked" is pinned against no real keychain, so this abnormal test runs against a behaviour the real dependency is not shown to have.
disposition: fixed as T4-O2 in task `o4`: `refuse_reads_as` is pinned by a contract clause, a store locked after a write, in `the_in_memory_keychain_refusing_reads_answers_as_a_store_locked_after_a_write` (LLR-36jer8) in org-io/tests/keychain_contract.rs, which the native locked run also checks.

**finding-9**: guideline, medium — org-io: Floor, "class B and C: abnormal-input tests for every REQ and LLR", and Doubles 5's statement of the robustness rule, "every requirement gets abnormal-input tests ... beside its normal-case tests".
- LLR-mas8vc: org-io/tests/person.rs:153-172 (`every_person_summary_is_provisional`) is its only test. It covers a normal proposal plus a source scan; no abnormal case, such as a Person reopened from its file or after a refused proposal.
- LLR-sn5ded: org-io/tests/acknowledgements.rs:99-146 (`org_io_learns_and_removes_acknowledgements_only_through_org_node`) is its only test. It covers a normal delivery plus a scan.
- LLR-eeh4pq: org-io/tests/receive_step.rs:63-87 and :360-377 are both normal-case. A command completes while `next_received` waits with nothing arriving. No test covers the lock not being held while an answer is sent, or a check and store write running while another command waits.
- Contrast: the static-scan LLRs that carry no input got a dated "robustness, N/A" note in the DRAFT LLR file (lines 259-261, 2105-2106, 2124-2125). These three behavioural LLRs have neither a test nor a recorded reason.
disposition: fixed as T4-O3 in task `o4`: abnormal tests `a_refused_body_leaves_no_lock_held_for_the_next_wait` (LLR-eeh4pq) and `a_refused_proposal_and_a_reopened_person_stay_provisional_with_their_fields` (LLR-mas8vc), each mutation-checked, and a dated "robustness, N/A" note on LLR-sn5ded.

**finding-10**: guideline, low — org-io: Owned services 1, "A backend, database or sync service that the codebase contains ... runs for real in a REQ-level test of the normal case".
- Where: org-io/tests/chain_write.rs:52-70, org-io/tests/verification.rs:71-100 and every other normal-case write and verify test of an update. All run only over `test_support::FakeChain`.
- What: the OrgRegistry contract the writes and reads go to is in the codebase (on-chain/). The only REQ-level normal case over the real chain is the founding: org-io/tests/chain_writer_contract.rs:105-123, through the handle, outside the gates. No handle-level normal case writes and verifies a next epoch against the real chain. The seam-level contract (clauses 5-6) pins the update's mechanics, not the REQ behaviour.
- Severity: low, because the contract test bounds the drift and whether a deployed contract counts as an "owned service" is a reading of the clause. If the owner reads it otherwise, the clause is the place to say so.
disposition: not fixed — task `o4` lists TEST low 4 as not done (docs/plans/2026-10-21-org-io-domain-api.md); the update's normal case over the real chain stays outside the gates (org-io/tests/chain_writer_contract.rs). Carried open, not blocking.

**finding-11**: guideline, low — org-io: Doubles 3, "Every fake has a contract test. The same assertions run against the fake and against the real dependency".
- `epoch_ahead` (org-io/src/test_support.rs:61, :278) returns the slot one epoch past its writes, with the same root and key. It drives the `Behind` outcome (org-io/tests/verification.rs:84-86, and the `adopted_and_removed_write...` test). It is a chain state the code interprets, and org-io/tests/common/chain_contract.rs pins it in no clause. The module doc lists `state_hidden` and `parse_refusing` as pinned and `read_failing` as a fault injector, and leaves `epoch_ahead` out.
- `found_at_failing` (org-io/src/test_support.rs:333) returns on-chain-client's typed `WriteError::Step { step: Fund, .. }`. org-io/tests/chain_write.rs:435-453 asserts that the report names it ("Fund"). The founding contract pins only the `RecordGenesis` step.
- Severity: low, because the `Step` shape itself is pinned.
disposition: partly fixed in task `o4`: `epoch_ahead` is pinned by a new clause of `fake_chain_meets_the_reader_and_writer_contract` (failed with `epoch_ahead` set to +2). `found_at_failing`'s Fund step is not, since it needs an unfunded controller on the fork; it recurred as low 8 of round 5 and low 6 of round 6, carried open (org-io/src/test_support.rs).

**finding-12**: guideline, low — org-io: Floor, "every test fails when the behavior it `verifies:` breaks" (LLR-5tpdxw: "returns `WriteRefused::Store` saying the proxy exists and naming it").
- Where: org-io/tests/chain_write.rs:399-414, :613-655 and :657-679.
- What:
  - At the handle, `a_failed_store_write_after_create_proxy_stops_and_names_the_proxy` binds the refusal's `proxy` and only prints it (:410). It never compares it with the proxy `create_proxy` returned.
  - `each_write_refusal_displays_human_readable_text` (`verifies: LLR-5tpdxw`) and `a_write_refusal_keeps_its_cause_as_its_source` (`verifies: LLR-48me7m, LLR-5tpdxw`) build `WriteRefused::Store { proxy, .. }` and `WriteRefused::Node(..)` by hand.
- Why it matters: if the handle named the wrong proxy, or dropped the cause it was given, all three tests would still pass. The handle-level test should assert the proxy it names.
disposition: fixed in task `o4` (TEST lows 6–9): `a_failed_store_write_after_create_proxy_stops_and_names_the_proxy` in org-io/tests/chain_write.rs (LLR-5tpdxw) now compares the named proxy with the one `create_proxy` returned, and failed with the proxy set to zeros.

**finding-13**: guideline, low — org-io: Floor, "every test fails when the behavior it `verifies:` breaks", in its assertion.
- Where: org-io/tests/chain_write.rs:298, :306 (`a_refused_update_write_keeps_the_provisional_organisation_and_sends_nothing`).
- What: "nothing sent: no endpoint bound by the write" compares `bound_endpoints_for_test()` before and after. But `ready()` already started receiving: `proposed`, then `requested`, then `start_receiving`, so the Persona's endpoint is bound before the write. A send from that endpoint would leave the count unchanged.
- Why it matters: the assertion and its message claim a check that cannot fail. LLR-s7rq5x states no "sends nothing", so the line misleads rather than verifies.
disposition: fixed in task `o4` (the plan lists TEST lows 6–9 as fixed; docs/plans/2026-10-21-org-io-domain-api.md): the nothing-sent check that could not fail is gone, and the test is now `a_refused_update_write_keeps_the_provisional_organisation` in org-io/tests/chain_write.rs, verifying LLR-s7rq5x by the refusal and the provisional Organisation still held.

**finding-14**: guideline, low — org-io: Seams 2, "Convenience is not on the list. A case reachable from the REQ interface with a boundary fake is tested there, even where a deeper seam would be easier."
- Where: org-io/tests/persona_creation.rs:185-200 (`a_persona_created_while_receiving_is_bound_and_accepted_on`, LLR-jvx6j9).
- What: it shows "accepted on" through `next_inbound_for_test`, the receive channel before org-node. LLR-jvx6j9 is an `OrgIo` item (SDD-bzu2jg), not the endpoints item's interface, and the same acceptance is observable through `next_received`. A body sent there comes back `Err(ReceiveError::Refused(MalformedMessage))`, as receive_step.rs:299-307 does for an existing endpoint.
disposition: fixed in task `o4` (TEST lows 6–9): `a_persona_created_while_receiving_is_bound_and_accepted_on` (LLR-jvx6j9) in org-io/tests/persona_creation.rs now shows the acceptance at the handle, and timed out with the endpoint bound without accepting.

**finding-15**: guideline, low — org-io: Seams 2 criterion 1, "Unreachable. A case cannot be triggered through the REQ interface without faking code the project owns", read with Mutants 1, "a survivor means one of: a test is missing at the seam ... the mutated code is dead: delete it."
- Where, in org-io/docs/architecture/DRAFT-worktree-org-io-domain-api-domain-api.md:
  - :273-280 (LLR-4yte89, `CreateError::FoundingNotListed`)
  - :1643-1651 (LLR-rav7dr)
  - :1716-1724 (LLR-bv6wkz)
  - :1910-1916 (LLR-3fxxpp)
- What: each note records a clause that is reachable only with a double of org-node, and keeps both the clause and its code untested ("The code stays as it is"). The first note even cites Seams 2 criterion 1 as the reason for no test. In the file, criterion 1 is exactly the case that earns a direct test at a deeper interface with an LLR. Mutants 1 gives the other way out: the code is dead, so delete it. Neither clause allows keeping stated, untested behaviour.
- Severity: low, because each branch needs a state the notes argue cannot occur.
disposition: fixed in task `o4` (the plan lists TEST lows 6–9 as fixed): LLR-4yte89's untested-clause note was amended in place. The listing clauses of LLR-rav7dr, LLR-bv6wkz and LLR-3fxxpp stay untested under the owner ruling of 2026-10-28 on T3-O2.

**finding-16**: guideline, medium — org-node: floor via preamble ("The compliance floor the skills state wins any conflict"): "every test fails when the behavior it `verifies:` breaks" — org-node/tests/revocation.rs:274-299
- `accept_refuses_in_order_and_changes_nothing` is annotated `verifies: LLR-rhsfw9, LLR-f9mcsq`.
- `revocation::accept` takes `&StoreData` (src/revocation.rs:127), so `assert_eq!(encoded(&store), before)` cannot fail.
- LLR-f9mcsq governs `OrgService` operations: a refused kept notice stays kept, and nothing is removed. No `OrgService` call is made here.
- So a `verify` that deleted a refused notice would still pass this test, and the LLR-f9mcsq annotation claims evidence the test cannot give. The real evidence is in own_removal.rs.
disposition: fixed as T4-N1 in task `n4` (merged 2026-10-28): `accept_refuses_in_order` in org-node/tests/revocation.rs verifies LLR-rhsfw9 only, and the assertion that could not fail is dropped; LLR-f9mcsq keeps its `OrgService` tests in org-node/tests/own_removal.rs.

**finding-17**: guideline, medium — org-node: floor: "every test fails when the behavior it `verifies:` breaks" — org-node/tests/absences.rs:172-179
- `the_service_presents_no_chain_write` is annotated `verifies: LLR-7en38k, LLR-x5asbx` (annotation added in this diff).
- Its only assertion is that "async fn submit_" is absent from service.rs.
- LLR-x5asbx says the proxy account is passed to no operation and is read or changed by no operation but those named.
- A synchronous `fn submit_x(proxy)`, or any operation that read or rewrote `proxy_account`, passes this test.
disposition: fixed as T4-N2 in task `n4`: `the_service_presents_no_chain_write` (LLR-7en38k, LLR-x5asbx) now refuses any `fn submit_`, any public ChainAccount taker other than `record_founding_proxy`, and any proxy use outside the named operations; it failed under each of three mutations.

**finding-18**: guideline, medium — org-node: floor: "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/verification.rs:626-648
- LLR-d9x6xf's only test, `changes_the_adopted_record_left_out_are_discarded_and_reported_in_order`, is the normal case.
- The LLR's decision rule is untested at its boundary: "an `Admit` is included exactly when the adopted record lists a member with the kept request's Member-as-a-group key and DevicePublicKey". Two cases are missing:
  - a record listing the member key with another Device key, or the reverse;
  - a `Revoke` whose MemberId is still held.
- A rule matching on one key alone passes.
disposition: fixed as T4-N3 in task `n4`: `an_admit_whose_request_matches_a_listed_member_on_one_key_only_is_discarded` and `a_revoke_whose_member_id_the_adopted_record_still_holds_is_discarded` in org-node/tests/verification.rs (LLR-d9x6xf), each mutation-checked.

**finding-19**: guideline, low — org-node: floor: "every test fails when the behavior it `verifies:` breaks" — org-node/tests/outgoing_bodies.rs:211-216 (used at :239 and :294)
- `assert_notice` checks `device`, `org_id` and that the proof verifies for `notice.member_id`, but never that `member_id` is the removed Member's.
- An absence proof verifies for any MemberId the record lacks, so a wrong-Member notice passes.
- LLR-degg8t requires that Device's notice.
disposition: fixed in task `n4` (TEST lows 4 to 12): `messages_for_the_current_epoch_inform_every_listed_device_but_the_own_then_notify_every_removed_one` in org-node/tests/outgoing_bodies.rs (LLR-degg8t) now fails with a notice naming another MemberId, which the old `assert_notice` passed.

**finding-20**: guideline, low — org-node: floor: "every test fails when the behavior it `verifies:` breaks" — org-node/tests/organisation_key.rs:115-153
- `an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused` is annotated `verifies: REQ-ech45n, LLR-9qavm7`, but calls only `OrgPublicKey::ensure_distinct_from`, never `propose_founding`.
- If `propose_founding` stopped checking the DevicePublicKey, it still passes.
- Seams 2: "A deeper interface gets a direct test only once it has an LLR". `ensure_distinct_from` has no LLR of its own; it is named only in an amendment note to LLR-9qavm7.
- The case is unreachable through `propose_founding` (Seams 2.1), so the code path is right. But the trace credits `propose_founding`'s device clause, and the boundary-fake tests (proposals.rs:183-210, organisation_key.rs:403-411) cover only the Member-key half.
disposition: fixed in task `n4` as far as the trace (TEST lows 4 to 12): `an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused` was re-annotated to REQ-ech45n, with an LLR for `ensure_distinct_from` left open. That recurred as T5-N2 and was closed by owner ruling: task `n5` minted LLR-c759yd, which the test now verifies.

**finding-21**: guideline, low — org-node: floor: "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/organisation_request.rs:122-170
- LLR-vyank2 states the refusal `OrgNotHeld` for a store with neither a record nor a provisional Member. A founding store whose proxy is recorded and not yet adopted is one such store.
- The test covers `FoundingNotWritten`, `PersonaNotHeld` and bad addresses, but not `OrgNotHeld`.
disposition: fixed in task `n4`: `build_organisation_request_refuses_a_recorded_founding_not_yet_adopted_as_org_not_held` in org-node/tests/organisation_request.rs (LLR-vyank2), failed with that arm returning `FoundingNotWritten`.

**finding-22**: guideline, low — org-node: floor: "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/organisation_request.rs:172-188, 206-221
- LLR-7kdhst's one annotated test is the normal answer only.
- Its abnormal case, a listed requester's invalid addresses refused with nothing kept (:216-220), sits in a test annotated `LLR-283x6f, LLR-j338wq`. So LLR-7kdhst has no abnormal test by trace.
- The `:187` assertion is only `change.is_some()`.
disposition: fixed in task `n4`: `a_listed_requesters_invalid_addresses_are_refused_and_answered_nothing` (LLR-7kdhst) is new, and `a_listed_requester_is_answered_with_the_full_record_and_its_key` now checks the answer's content; both failed under a mutation.

**finding-23**: guideline, low — org-node: floor: "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/proposals.rs:1325-1365 (and :1215-1300)
- LLR-8ru6mv: "Every per-store view refuses an unknown Device key with `PersonaNotHeld`".
- Only `persona_view` is tested for it. `provisional_changes`, `provisional_organisations` and `admission_requests` never are.
disposition: fixed in task `n4`: `every_per_store_view_refuses_an_unknown_device_key_as_persona_not_held` in org-node/tests/proposals.rs (LLR-8ru6mv) covers every per-store view, and failed with `provisional_changes` defaulting to store 0.

**finding-24**: guideline, low — org-node: floor: "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/own_removal.rs:332-366
- LLR-myfk8w: every operation other than the four named refuses a removed store with `OrganisationRemoved`.
- `every_other_operation_on_a_removed_store_is_organisation_removed` enumerates fifteen operations, but not `persona_view` (src/service.rs:1184), the view org-io reads stores through.
- The code likely refuses (via `data()?`); this is a gap where the code is right.
disposition: fixed in task `n4`: `every_other_operation_on_a_removed_store_is_organisation_removed` in org-node/tests/own_removal.rs (LLR-myfk8w) now includes `persona_view`, and failed with it mapping the error to `FoundingNotWritten`.

**finding-25**: guideline, low — org-node: Seams 1: "a test pinned to a function breaks on a refactor that moves it and keeps every behavior" — org-node/tests/revocation.rs:614-636
- The test asserts that `ACK_DOMAIN` is named exactly twice, in revocation.rs alone.
- LLR-bs6yxf states only that no other signed message begins with `ACK_DOMAIN`.
- A third mention, such as an alias or helper, fails the test with no behaviour change. A second literal of the same bytes passes it.
disposition: fixed in task `n4`: `acknowledgement_signed_bytes_are_the_domain_then_the_tuple` in org-node/tests/revocation.rs (LLR-bs6yxf) failed with a second sign over another message under `ACK_DOMAIN` and with a second const holding the same bytes, which the counting version passed.

**finding-26**: guideline, low — org-node: Seams 1 (same words) — org-node/tests/receive_path.rs:101-105; org-node/tests/revocation.rs:656-662; org-node/tests/proposals.rs:144-153
- Each test slices a function's signature out of `include_str!` source and compares exact text, for example `-> Result<(Checked, Option<StoreChange>), OrgNodeError>`.
- A rustfmt wrap or a type alias breaks it with no behaviour change.
- The same tests already call the function with the LLR's arguments, so the compiler pins the stated signature.
disposition: fixed in task `n4`: the exact-signature text checks were removed, leaving the compiler to pin the signatures, in `check_received_takes_no_chain_state_and_refuses_an_oversized_or_undecodable_body` (LLR-94rzrk), `propose_founding_calls_no_chain_operation_and_returns_the_secret_by_value` (LLR-d4gpxm) and the revocation.rs test.

**finding-27**: guideline, low — org-node: floor: "every test fails when the behavior it `verifies:` breaks" (on the case it names) — org-node/tests/wire_frame_bound.rs:283-289
- The "33 key bytes" case appends a byte after a fixed `[u8; 32]` key. That reaches the leftover-byte refusal, not a key-length case.
- The leftover-byte refusal is already covered elsewhere in the file, so the label claims evidence the input never reaches.
disposition: fixed in task `n4`: the mislabelled case was removed from `a_key_field_that_is_not_32_bytes_does_not_decode` in org-node/tests/wire_frame_bound.rs (labels and assertions removed only).

**finding-28**: guideline, low — org-node: Mutants 1, "A survivor never justifies a test below the seam unless one of the three criteria above holds" — docs/TEST_GUIDELINES.md:192-193
- Seams 2 now has four criteria: criterion 4 was tailored 2026-10-28.
- This clause still says three, so it excludes the SDD-item-interface seam the floor requires. The rule changed and this reference was kept.
disposition: fixed in task `a5` (the plan lists TEST org-node low 13): Mutants 1 in docs/TEST_GUIDELINES.md now counts Seams 2's four criteria, and the tailoring note is updated.

### Round 5

**finding-1**: guideline, medium — org-io: Owned services 1, "A service the project owns runs for real on the normal path … runs for real in a REQ-level test of the normal case" — org-io/tests/send_fanout.rs:45, :154, :337; org-io/tests/requests.rs:50, :139; org-io/tests/expectations_met.rs:363-388, :394-410; org-io/tests/acknowledgements.rs:79, :155, :353, :772.
- What: in these normal-case tests the recipient Device is a stand-in, not a real receiving org-io handle. The stand-ins are `journey::answering` / `answering_acknowledgements` (journey.rs:158-177, :422-447) and `requests.rs::answer_one` (requests.rs:36-42). Each binds a real `OrgEndpoint`, but the test picks the verdict, so the recipient's receive step and org-node's check never run.
- LLRs/REQs affected: LLR-pxqdy4, LLR-34tfm8, LLR-adgk9z, LLR-uf5cyy, REQ-u894ty, LLR-zzsqj4, LLR-cqr26z, LLR-6ux6yw and LLR-n649th.
- Why: results such as "Delivered" for every recipient, or "accepted, then deleted", are checked against an answer the real recipient was never shown to give in that test.
- A real receiving handle is practical here: endpoints.rs:152, receive_step.rs:196 and :320 use one already.
- The one real-recipient run of the acknowledgement normal path is acknowledgement_inbound.rs:79-81. Its comment says it shows LLR-6ux6yw's deletion, but it is annotated only `LLR-j467e9, LLR-hhgdk7`.
disposition: fixed as T5-O1 in task `o5` (merged 2026-10-29): the normal-case send, request, expectation and acknowledgement tests now send to real receiving OrgIo handles and assert on their receive-step reports, for example `send_organisation_sends_org_nodes_messages_in_order_to_every_recipient_and_writes_no_store` (LLR-pxqdy4, REQ-u894ty), each mutation-checked; `ack_counting` stays as the one stand-in for LLR-6ux6yw's ordering half, and its definition says why.

**finding-2**: guideline, medium — org-io: Doubles 3, "Every fake has a contract test. The same assertions run against the fake and against the real dependency"; Owned services 1, "a fake of the service, under the same contract test as any fake … A double that returns the service's own error … is this kind, not a fault injector" — org-io/tests/common/journey.rs:158-177, :422-447; org-io/tests/send_fanout.rs:237-263; org-io/tests/requests.rs:79-102; org-io/tests/acknowledgements.rs:111, :210, :282, :402, :452 (and the other `answering_acknowledgements` uses).
- What: the stand-ins script the receiving node's own answers: `Refused` (0x01), `AlreadyReceived` (0x02), `InviteUsedByAnotherDevice` (0x03) and `Accepted`. The sender interprets each one (next address, keep, or delete). These are fakes of an owned service, not fault injectors.
- Why: no contract test runs the same assertions over the stand-ins and over the real receive step.
- The real bytes are pinned only by separate tests: receive_step.rs:301-313, :320-345, :428-429 and :625; acknowledgement_inbound.rs:113, :159 and :170.
- These are fine as they are: a dropped reply (`None`), `dead_addresses` and `soon_dead` are fault injectors under Doubles 5.
disposition: fixed as T5-O2 in task `o5`: every stand-in definition states why it is kept, and the new contract test `the_scripted_recipients_verdicts_are_the_real_receive_steps` in org-io/tests/acknowledgements.rs (LLR-xwz3nu, LLR-6ux6yw, LLR-n649th) runs the same assertions over the stand-in's verdicts and the real receive step's; the send verdict test takes Delivered and Refused from a real step.

**finding-3**: guideline, medium — org-io: floor (preamble: "The compliance floor the skills state wins any conflict"), "every test fails when the behavior it `verifies:` breaks" — org-io/tests/acknowledgements.rs:394-425.
- What: LLR-xwz3nu covers any answer but `Accepted`: `Refused`, `AlreadyReceived`, `InviteUsedByAnotherDevice`, or another byte. Its one test answers only `Refused`, then no answer.
- Why: src/acknowledge.rs:389 maps `AlreadyReceived` and `InviteUsedByAnotherDevice` to refused. A plausible mutant that treats `AlreadyReceived` as delivered ("they have it") survives every test. The stand-ins can already answer those bytes (requests.rs:89).
disposition: fixed as T5-O3 in task `o5`: `already_received_or_invite_used_keeps_the_acknowledgement` (LLR-xwz3nu, LLR-3w22wc) failed with `AlreadyReceived` mapped to Delivered.

**finding-4**: guideline, medium — org-io: floor, "every test fails when the behavior it `verifies:` breaks", read with Existing suites 1, "These rules bind the tests a change writes or edits" — org-io/tests/absences.rs:530.
- What: the edited test `the_handle_returns_only_allowlisted_public_types` still cites `verifies: LLR-3zdw8v, LLR-7pj5af, LLR-9r2bxd`. LLR-7pj5af is superseded by LLR-kz9hj4 (DRAFT:1378) and LLR-9r2bxd by LLR-s7rq5x (DRAFT:655).
- Why: in the same change, `org_io_hands_out_no_bare_service` was moved off the same two IDs. This test credits retired items and does not name the in-force item it checks (LLR-gh4rw3 / LLR-4usfu2).
disposition: fixed as T5-O4 in task `o5`: `the_handle_returns_only_allowlisted_public_types` in org-io/tests/absences.rs now cites LLR-gh4rw3 and LLR-4usfu2 in place of the superseded items.

**finding-5**: guideline, medium — org-io: floor, "every test carries `verifies:`", read with Existing suites 1 — org-io/tests/preflight.rs:18-19 (`chain_live_check_passes_with_miner_and_fails_when_stalled`).
- What: this change edits the test (its module doc, and a comment added inside its body at :46), and the test carries no `verifies:`.
disposition: T5-O5, accepted: owner ruling 2026-10-29 (accept and record). The chain.live test in org-io/tests/preflight.rs verifies no ID by design: no REQ states the check, and SDD-z85ux9 carries no LLRs by its recorded deviation. A dated note in that deviation (org-io/docs/architecture/2026-10-08-org-io.md) and the test file's header say so (task `o5`).

**finding-6**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/verification.rs:260-270 (verifies LLR-6vtbn6).
- What: the scan for `fn with_clock_for_test`, `fn open_with_clock_for_test` and `TestClock` skips `test_support.rs`, the only file that defines them.
- Why: what keeps them out of a production build is `#[cfg(feature = "test-support")] pub mod test_support;` (src/lib.rs:24-25) and the gate on its `impl OrgIo` block. The test checks neither, so it passes with the gates removed.
disposition: fixed in task `o5` (TEST lows 6, 7, 9–16, 20–24): `a_test_clock_replaces_the_handles_clock_under_test_support_only` (LLR-6vtbn6) now checks the gate and failed with the cfg removed from the test_support `impl OrgIo`.

**finding-7**: guideline, low — org-io: Seams 1, "a test at a requirement's interface breaks only when a stated behavior changes" — org-io/tests/chain_write.rs:647-671 (`each_write_refusal_displays_human_readable_text`, verifies LLR-5tpdxw).
- What: it asserts the exact Display text of `NotHeld`, `NoProxyAccount`, `Stale`, `OutcomeUnknown` and `Node`. No LLR states that text; the test's own comment at :635 says LLR-5tpdxw's `Store` text is the only one an LLR states.
disposition: fixed in task `o5`: `each_write_refusal_displays_human_readable_text` in org-io/tests/chain_write.rs (LLR-5tpdxw) was changed, and failed with `Stale` showing `NotHeld`'s text.

**finding-8**: guideline, low — org-io: Doubles 5, "A fake, with a contract test: a double that imitates service-specific content the code interprets" — org-io/src/test_support.rs:376-378; relied on at org-io/tests/chain_write.rs:424, :450, :519 and org-io/tests/verification.rs:687.
- What: `found_at_failing` returns on-chain-client's typed `WriteError::Step { step: GenesisStep::Fund, .. }`, which LLR-sq5pbq reports by step name.
- Why: the founding contract (common/chain_writer_contract.rs clause 3) pins only the `RecordGenesis` refusal against the real writer. The Fund-step answer has no contract.
disposition: not fixed — pinning the Fund step needs the real writer on a fork (task `o5`, docs/plans/2026-10-21-org-io-domain-api.md); recurred as low 6 of round 6, carried open, not blocking (org-io/src/test_support.rs).

**finding-9**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/verification.rs:777-781 (verifies LLR-8w2z64).
- What: LLR-8w2z64 says that on a keychain refusal "no acknowledgement delivery starts". The test asserts nothing about the delivery; compare :414, which checks `acknowledgement_report`.
disposition: fixed in task `o5`: `the_secrets_closure_reads_the_keychain_only_to_sign_and_a_refusal_is_verified_keychain` in org-io/tests/verification.rs (LLR-8w2z64) now asserts that no delivery starts (added, not watched fail).

**finding-10**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/chain_write.rs:74-110, :339-347 (verify LLR-48me7m).
- What: LLR-48me7m says each refusal happens "writing nothing to the chain or the store". Both tests check only that no chain call was made, never that the store bytes are unchanged (`store_bytes` is already imported).
disposition: fixed in task `o5`: `not_held_no_proxy_and_stale_are_refused_before_any_chain_call` and `a_stale_provisional_organisation_is_refused_before_any_write` in org-io/tests/chain_write.rs (LLR-48me7m) now assert the store bytes unchanged (added, not watched fail).

**finding-11**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/absences.rs:264-267.
- What: `org_io_hands_out_no_bare_service` is re-annotated `verifies: LLR-4usfu2`. It scans only for a public `OrgService` field or by-value return, which LLR-4usfu2 does not state (it states the exact method list).
- Why: adding or removing a method breaks LLR-4usfu2 and does not fail this test.
disposition: fixed in task `o5`: `org_io_hands_out_no_bare_service` is re-annotated to the newly minted LLR-n2srhc, which states what it checks, and failed with a public `OrgService` field planted.

**finding-12**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/absences.rs:594-615, :698 (verifies LLR-gh4rw3).
- What: `type_body` matches only `pub struct X ` / `pub struct X{` / `pub enum X ` / `pub type X `, and line 698 silently skips anything else (`else { continue }`).
- Why: a tuple or generic struct reached at depth goes unread, for example `ChainReadError(Cause<OrgNodeError>)` (src/verify.rs:36). LLR-gh4rw3 forbids a returned type holding a secret "at any depth".
disposition: fixed in task `o5`: `every_returned_type_is_in_the_handle_return_allowlist_and_holds_no_secret` in org-io/tests/absences.rs (LLR-gh4rw3) failed with a tuple `ChainReadError(OrgPrivateKey)` planted.

**finding-13**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/absences.rs:1331-1333 (verifies LLR-b97usg).
- What: the manifest check flags only lines starting `person ` or `person=`.
- Why: it misses `[dependencies.person]` / `[dev-dependencies.person]` tables and a renamed `package = "person"` dependency, all of which the LLR forbids.
disposition: fixed in task `o5`: `org_io_names_no_person_crate` in org-io/tests/absences.rs (LLR-b97usg) failed with `package = "person"` planted in Cargo.toml.

**finding-14**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/chain_read_rule.rs:85-99 (verifies LLR-k3zmqb).
- What: every `.read_state(` and `.finalised_block(` call in `chain_read.rs` and `connect.rs` is exempted as a "defining" file.
- Why: those files define the functions with `fn read_state(`, which the call pattern never matches, so the exemption only hides a real call that LLR-k3zmqb forbids.
disposition: fixed in task `o5`: `the_state_reader_is_named_only_in_verify_and_preflight` in org-io/tests/chain_read_rule.rs (LLR-k3zmqb) failed with a `.read_state(` call planted in connect.rs.

**finding-15**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/app_boundary.rs:228-231 (verifies LLR-ntag3g).
- What: one `read_dir` of `app/src-tauri/src`, with no walk into subfolders; the pattern `org_node::service` also misses `use org_node::{service::…}`. LLR-ntag3g covers "no file of `app/src-tauri/src`".
disposition: fixed in task `o5`: `the_app_calls_no_adopting_operation_and_builds_no_service` in org-io/tests/app_boundary.rs (LLR-ntag3g) failed with `use org_node::{service::OrgService}` planted in an app sub-module.

**finding-16**: guideline, low — org-io: Seams 1, "A test calls only an interface that a REQ or LLR describes" — org-io/tests/open.rs:47, :54.
- What: the edited test reads its result through the test-only `persona_public_keys` (src/lib.rs:487-490), which no LLR names.
- `personas()` already gives the Device key, as `common::handles::persona_of` uses it.
disposition: fixed in task `o5`: `open_creates_the_data_dir_and_opens_the_store_inside_it` in org-io/tests/open.rs was changed and failed with open loading no store.

**finding-17**: guideline, low — org-io: Seams 1, "A test calls only an interface that a REQ or LLR describes" — org-io/tests/acknowledgements.rs:168, :171, :176, :417 and each `wake_deliveries_for_test` call; org-io/tests/acknowledgement_inbound.rs:182.
- What: these tests call `with_bind_plan_for_test`, `bound_endpoint_for_test`, `wake_deliveries_for_test` (src/lib.rs:702, :738, :779) and `next_inbound_for_test`, which no LLR names.
- acknowledgements.rs:176 observes LLR-cqr26z's endpoint close through a test accessor. acknowledgement_inbound.rs:182 takes the body off the channel before the receive step.
disposition: not fixed — the `*_for_test` hooks stand in for the hourly timer and the relay bind plan (task `o5`, docs/plans/2026-10-21-org-io-domain-api.md); carried open, not blocking (org-io/tests/acknowledgements.rs).

**finding-18**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/acknowledgements.rs:274-292 (verifies LLR-3w22wc).
- What: these delivery clauses are untested:
  - moving on after "any other verdict byte";
  - "no byte within 10 s";
  - `NotDelivered { reason: NoAddress }` (src/acknowledge.rs:454);
  - Loopback dialling only addresses whose identity is the recipient's key.
disposition: partly fixed in task `o5`: `NotDelivered { reason: NoAddress }` and the Loopback identity are driven by `a_recipient_with_no_address_of_its_own_is_not_delivered_for_no_address` (LLR-3w22wc), with other verdict bytes; the rest (no byte within 10 s) needs a raw stream endpoint and is not fixed — carried open, not blocking (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-19**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/acknowledgements.rs:439-475 (verifies LLR-bv6wkz).
- What: for a clock before the epoch, the test never asserts "with no next round" (`next_round == None`). It retries only through `wake_deliveries_for_test`, never through the hourly timer, so a stale `next_round` or a dropped timer survives.
disposition: partly fixed in task `o5`: `a_clock_before_the_epoch_is_the_rounds_failure_and_nothing_is_sent_or_deleted` (LLR-bv6wkz, LLR-rav7dr) now asserts `next_round == None`, failed with it not cleared; the retry through the hourly timer is not driven — carried open, not blocking (docs/plans/2026-10-21-org-io-domain-api.md).

**finding-20**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/acknowledgements.rs:551-581 (verifies LLR-rav7dr).
- What: "it writes nothing and sends nothing" is not asserted. `store_writes_to` and the arrivals are both at hand in the test.
disposition: fixed in task `o5` (the plan lists TEST lows 20–24 as fixed; docs/plans/2026-10-21-org-io-domain-api.md): the LLR-rav7dr test in org-io/tests/acknowledgements.rs now asserts that the round writes and sends nothing.

**finding-21**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/acknowledgement_inbound.rs:110-114 (verifies LLR-j467e9).
- What: the rewrite weakened master's specific `OrgNodeError::SenderNotListed` / `AcknowledgementNotFromItsDevice` / `AcknowledgementFromFuture` to `Err(ReceiveError::Refused(_))`.
- Why: the bodies here (the Admin's own epoch message, and a request for a settled invite) may be refused for reasons other than the sender gate. The relayed case at :165 still discriminates.
disposition: fixed in task `o5`: `a_revoked_device_sending_anything_but_an_acknowledgement_or_an_organisation_request_is_refused` (LLR-j467e9, LLR-hhgdk7) pins the specific refusals again, and failed with every refusal reported as `MalformedMessage`.

**finding-22**: guideline, low — org-io: floor, "class B and C: abnormal-input tests for every REQ and LLR" (clause coverage) — org-io/tests/rng_source.rs:188-196, :207-232 (verifies LLR-94nn6q).
- What: the `fill_bytes` half (org-node panics before any store change, so nothing is written) is not exercised; the test's own comment says so. It is reachable with the random-source double the file already uses.
disposition: fixed in task `o5`: `a_failing_draw_inside_org_node_panics_before_any_store_change` in org-io/tests/rng_source.rs (LLR-94nn6q), failed with `issue_invite` catching the panic.

**finding-23**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/persona_creation.rs:249-271 (verifies LLR-pn3hgc).
- What: the "no method but `propose_founding_organisation` takes an Organisation name" scan matches only a parameter spelled `org_name`, and reads only the top level of `src/` (it skips `src/transport/`). A method taking the name under another parameter name passes.
disposition: fixed in task `o5`: `organisations_report_their_stored_name_and_no_send_carries_it` in org-io/tests/persona_creation.rs (LLR-pn3hgc) failed with a method taking the name under another parameter name.

**finding-24**: guideline, low — org-io: floor, "every test fails when the behavior it `verifies:` breaks" — org-io/tests/person.rs:109-127 (verifies LLR-3cysjk).
- What: LLR-3cysjk refuses a second Person "before org-node is called or anything is drawn". The test checks no item and no file, but not that nothing was drawn from the handle's random source; a seeded source through `with_rng_for_test` would show it.
disposition: fixed in task `o5`: `a_second_person_is_refused_naming_the_one_held_and_writes_nothing` in org-io/tests/person.rs (LLR-3cysjk) failed with `propose_person` drawing before its check.

**finding-25**: guideline, medium — org-node: floor (Preamble: "The compliance floor the skills state wins"), "every test carries `verifies:`" — org-node/tests/key_custody.rs:50-55
- The test `two_different_keypairs_do_not_share_a_device_key` has no `verifies:` annotation. Its comment says "No requirement states it; it is a regression check".
- This change edited the comment, so the test is one the change touches.
- The floor requires `verifies:` on every test, so either name the item the test checks or delete the test.
disposition: fixed as T5-N1 in task `n5` (merged 2026-10-28): the key_custody test is now `two_device_seeds_give_two_device_keys`, verifying LLR-ctzkv7, and failed with `from_keychain_bytes` ignoring its bytes.

**finding-26**: guideline, medium — org-node: Seams 2, "A deeper interface gets a direct test only once it has an LLR" — org-node/tests/organisation_key.rs:107-150 (`an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused`)
- The test calls `OrgPublicKey::ensure_distinct_from` directly. Its own comment says "The check has no LLR of its own".
- LLR-sj7cd5, the check's old LLR, is superseded by LLR-9qavm7, which is stated of `propose_founding`. The amendments to LLR-9qavm7 and LLR-qghs95 mention `ensure_distinct_from` only in passing.
- The test cites Seams 2.1 (unreachable) as its reason, but that criterion needs an LLR to hold first. This change edited the test: the expected error changed from `Trie(DuplicateKey)` to `DuplicateOrgPublicKey`.
- Related: the DevicePublicKey half of LLR-9qavm7 and LLR-qghs95 is checked only by this test below the seam. The proposals.rs tests at 142-158, 1010-1029 and 1211-1230 collide only with Member-as-a-group keys and the record's key.
- Fix: either have an LLR state `ensure_distinct_from`'s contract and annotate the test with it, or drop the direct test.
disposition: T5-N2, a recurrence of T4-N5. Owner ruling (round 5): mint an LLR. Task `n5` minted LLR-c759yd under SDD-swtd3w (`ensure_distinct_from` refuses a key equal to any genesis Member key or DevicePublicKey; it satisfies REQ-ech45n and REQ-3pj7rd); `an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused` verifies it under Seams 2 criterion 4, failed with the DevicePublicKey comparison dropped, and LLR-9qavm7 has a dated note.

**finding-27**: guideline, medium — org-node: floor, "class B and C: abnormal-input tests for every REQ and LLR" and "every test fails when the behavior it `verifies:` breaks" — org-node/tests/person_store.rs:73-133, against org-node/src/person_store.rs:153-162
- LLR-73sgz2 says `PersonStore::unseal` "re-parses every field (`person.*`)". The code also refuses bytes that do not decode (`Undecodable`) and a decoded `device_key` that is not the one given (`DeviceSecretMismatch`).
- No test gives `unseal` any of these:
  - a sealed store that authenticates but holds a name, surname or key the parse refuses;
  - an authenticated plaintext that does not decode;
  - a store whose `device_key` differs from the argument.
- The tests cover only a wrong secret, a short input, a flipped byte and crossing between Persona and Person stores. The only `person.*` refusals tested are `create`'s.
- Deleting the `TryFrom` re-parse, the `Undecodable` mapping or the device-key check fails no test.
disposition: fixed as T5-N3 in task `n5`: `an_authentic_person_store_holding_a_refused_value_or_another_device_key_is_refused` in org-node/tests/person_store.rs (LLR-73sgz2), through a new hook `seal_person_plaintext_for_test`, failed with the device-key check dropped, with `Undecodable` mapped to NotAuthentic, and with the field renamed.

**finding-28**: guideline, medium — org-node: floor, "class B and C: abnormal-input tests for every REQ and LLR" (resource exhaustion, per Doubles 5's robustness rule) — org-node/tests/proposals.rs:928-990, and the claim at org-node/tests/provisional_store.rs:8-10
- LLR-f3hru6 says `record_revocation` refuses "the behind marker set and the bound". LLR-m76r4k names `record_revocation` as one of the four operations whose keeps are bounded.
- `ProvisionalLimit` is tested for the other three operations:
  - `record_admission`: proposals.rs:894 and :912;
  - `propose_updated_organisation`: proposals.rs:1119;
  - `check_received`: provisional_store.rs, and receive_path.rs:879.
- No test covers `record_revocation`. Removing `ensure_within_bound` from it fails no test.
- The module doc of provisional_store.rs says these keeps "are tested beside those operations", which is not true for this one.
disposition: fixed as T5-N4 in task `n5`: `a_revocation_refused_by_the_bound_records_nothing` in org-node/tests/proposals.rs (LLR-f3hru6, LLR-m76r4k) failed with `ensure_within_bound` removed from `record_revocation`.

**finding-29**: guideline, medium — org-node: floor, "every test fails when the behavior it `verifies:` breaks" — org-node/tests/verification.rs:594-617 (the assert is at :616)
- `adoption_replaces_the_record_and_reports_own_in_one_sealed_change` verifies LLR-34c5y3, which says the adoption "clears the behind marker". The test asserts `behind == None` after adopting.
- Its fixture never sets the marker, so the assert cannot fail.
- LLR-zk7fx2 says `verify` clears the marker "on `InStep` and `Adopted`". The only test that sets it before `verify` (verification.rs:473-494, line 489) covers `InStep` only.
- If adoption stopped clearing the marker, no test would fail.
disposition: fixed as T5-N5 in task `n5`: `adoption_replaces_the_record_and_reports_own_in_one_sealed_change` (LLR-34c5y3) sets the behind marker first, and failed with the clearing removed from `prune_after_adoption`.

**finding-30**: guideline, low — org-node: floor, "every test fails when the behavior it `verifies:` breaks" — org-node/tests/organisation_request.rs:200
- The check `!format!("{:?}", ...).contains("6060")` cannot fail. The key is `[0x60; 32]`, and a derived `Debug` would print it in decimal (`96, 96, …`), never as "6060".
- So LLR-7kdhst's "`Debug` redacted" clause is not checked here. It is low because LLR-4ces44 is tested in secret_redaction.rs.
disposition: fixed in task `n5` (TEST lows 6, 7, 9, 12–19): `a_listed_requester_is_answered_with_the_full_record_and_its_key` in org-node/tests/organisation_request.rs now fails with a Debug that renders a secret's bytes.

**finding-31**: guideline, low — org-node: floor, "every test fails when the behavior it `verifies:` breaks" — org-node/tests/proposals.rs:260-282 and 403-420
- LLR-rjphn4 says `propose_founding` and `propose_member` refuse "before any draw from `rng`".
- The draw is counted (`CountingRng`, line 174) only for `propose_founding`. The refusals of `propose_member` use `OsRng`.
- Moving `propose_member`'s seed draws ahead of the parse fails no test.
disposition: fixed in task `n5`: `propose_member_refuses_no_handle_four_handles_or_a_repeated_one` (LLR-6acxu9, LLR-rjphn4) counts the draws, and failed with a draw at the top of `propose_member`.

**finding-32**: guideline, low — org-node: floor, "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/organisation_request.rs:214-239
- LLR-j338wq's `prove_absent` refusal ("returned as `Trie(cause)` with no answer and no `StoreChange`") is untested.
- It can be reached by seeding a `RevokedDevice` whose `member_id` the record still holds.
- The LLR's other abnormal case is tested, hence low.
disposition: not fixed — `prove_absent`'s `Trie` refusal cannot be reached through `check_received` (task `n5`, docs/plans/2026-10-21-org-io-domain-api.md); LLR-j338wq's other abnormal case stays tested. Carried open, not blocking.

**finding-33**: guideline, low — org-node: floor, "every test fails when the behavior it `verifies:` breaks" — org-node/tests/verification.rs:1100-1130
- Both tests carry `verifies: LLR-gcm7bq` (the second also LLR-qvp7mq). What they actually exercise is LLR-8528ud's founding-match rule: epoch 1, and the members giving the root when calculated again.
- Breaking LLR-gcm7bq, which states what adopting the founding writes, would fail neither test.
- No test carries LLR-8528ud's ID for its founding clause.
disposition: fixed in task `n5`: `a_founding_whose_chain_is_past_epoch_one_is_behind_and_keeps_the_founding` and `a_founding_whose_members_do_not_rebuild_the_chain_root_is_not_adopted` in org-node/tests/verification.rs are re-annotated to LLR-8528ud, each mutation-checked.

**finding-34**: guideline, low — org-node: floor, "every test fails when the behavior it `verifies:` breaks" — org-node/tests/verification.rs:804-814
- `an_adoption_that_keeps_this_device_never_asks_for_the_device_secrets` carries LLR-5tvn7g, which is about signing the acknowledgement before a removal.
- The test removes nobody, so no clause of LLR-5tvn7g runs.
- The LLR's "count is zero" clause is tested at own_removal.rs:556-562.
disposition: not fixed — it needs an LLR change (task `n5`, docs/plans/2026-10-21-org-io-domain-api.md); LLR-5tvn7g's count-zero clause stays tested in org-node/tests/own_removal.rs. Carried open, not blocking.

**finding-35**: guideline, low — org-node: floor, "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/own_removal.rs:320-355
- LLR-myfk8w says every `OrgService` operation outside its list refuses a removed store with `OrganisationRemoved`.
- The test does not call these on a removed store: `proxy_account`, `persona_public_keys`, `list_personas`, `persona_records`, `list_orgs`, `expected_admissions`, `restore_store`.
- In src/service.rs:723-766 those skip the store or answer `OrgNotHeld` or `PersonaIdNotHeld`. No test pins either behaviour, and the LLR and the code disagree: that disagreement is for the CODE/ARCHITECTURE reviews.
disposition: not fixed — LLR-myfk8w and the code disagree, which is a design question (task `n5`, docs/plans/2026-10-21-org-io-domain-api.md); carried open, not blocking.

**finding-36**: guideline, low — org-node: floor, "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/verification.rs:421-446
- LLR-8528ud says a founding matches when "its root and key equal the chain's".
- No founding-store case gives the founding's own root under another `org_pub_key`. That case is tested only for a received provisional organisation (verification.rs:770-785).
disposition: fixed in task `n5`: `a_founding_whose_root_the_chain_holds_under_another_key_is_not_adopted` (LLR-8528ud), failed with the `org_pub_key` comparison dropped from `founding_matches`.

**finding-37**: guideline, low — org-node: floor, "every test fails when the behavior it `verifies:` breaks" — org-node/tests/receive_path.rs:430-445
- `an_unheld_organisations_notice_is_refused_as_not_held` is tagged LLR-94rzrk. It asserts `RevocationNotHeld`, which is LLR-95av3e's and LLR-r7zm39's rule, not LLR-94rzrk's.
- Its comment also claims an `OrgNotHeld` case that the test never checks.
disposition: fixed in task `n5`: `an_unheld_organisations_notice_is_refused_as_not_held` in org-node/tests/receive_path.rs is re-annotated to LLR-95av3e, and failed with the check returning `MalformedMessage`.

**finding-38**: guideline, low — org-node: floor, "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/receive_path.rs:223-330, against src/service.rs:689
- LLR-6gkxpg (amended 2026-10-22) says a received admission request with fewer than one, more than three, or repeated preferred Handles is refused with `InvalidField { field: "request.preferred_handles" }`.
- No test sends such a request through `check_received`. The string `request.preferred_handles` appears nowhere under org-node/tests.
disposition: fixed in task `n5`: `an_admission_request_with_no_four_or_a_repeated_handle_is_refused` (LLR-6gkxpg), failed with `check_preferred_handles` removed for the request.

**finding-39**: guideline, low — org-node: floor, "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/receive_path.rs:555-639
- LLR-e6w3nr says expected Organisation information in a store with no record has its snapshot decoded and passes `check_chain_free`.
- No receive-path test gives it a snapshot that does not decode, or a Change set that fails `check_chain_free` against that base.
- Those refusals are tested only at their own interfaces (persona_records.rs, verify_against_chain.rs), not as the receive path passes them on.
disposition: fixed in task `n5`: `expected_organisation_information_failing_its_snapshot_or_chain_free_checks_is_refused` (LLR-e6w3nr), failed with `check_chain_free` removed from `keep_information`.

**finding-40**: guideline, low — org-node: floor, "every test fails when the behavior it `verifies:` breaks" — org-node/tests/admission_sender.rs:78-82
- The unknown-Persona half of `persona_public_keys_returns_the_personas_two_keys` asserts only `.is_err()`. The LLR (C2-N2 amendment, DRAFT…:1414-1417) names the refusal `PersonaIdNotHeld { persona_id }`.
- sealed_store.rs:411 pins the exact variant, hence low.
disposition: fixed in task `n5`: `persona_public_keys_returns_the_personas_two_keys` in org-node/tests/admission_sender.rs pins the refusal's variant, and failed with another variant returned.

**finding-41**: guideline, low — org-node: floor, "every test fails when the behavior it `verifies:` breaks" — org-node/tests/fuzz_unseal/fuzz_target.rs:46-48 (and :32-39)
- Shape 3, which this change added, asserts `store.device_key() == device_key`. That cannot fail: `PersonaStore::unseal` builds the store from its own argument (src/store.rs:1291).
- So the doc's LLR-bg538c claim, that whatever unseal accepts has the Device key it was given, is not checked by the target. The example test sealed_store.rs:144-154 does check it.
disposition: fixed in task `n5` (the plan lists TEST lows 12–19): the assertions of org-node/tests/fuzz_unseal/fuzz_target.rs were changed; not mutation-checked, since a 1 s fuzz budget seldom reaches an authentic plaintext.

**finding-42**: guideline, low — org-node: floor, "every test fails when the behavior it `verifies:` breaks" — org-node/tests/sealed_store.rs:507-595
- `every_store_changing_operation_returns_one_sealed_change_with_a_fresh_nonce` verifies LLR-ay8nvu ("sealed … under the open store's key"). It checks each write's device_key and that the nonces differ, but never opens a write.
- A write of, for example, `record_admission`, `record_revocation`, `cancel_invite` or `acknowledgement_accepted` sealed under another key would still pass.
disposition: fixed in task `n5`: `every_store_changing_operation_returns_one_sealed_change_with_a_fresh_nonce` (LLR-ay8nvu) now opens each write, and failed with a byte of each sealed write flipped.

**finding-43**: guideline, low — org-node: floor, "class B and C: abnormal-input tests for every REQ and LLR" — org-node/tests/secret_redaction.rs:308-352
- LLR-4ces44 has one test, marked Normal, with one key value (`sentinel(0x80)`).
- Unlike its sibling tests, it has no case at the byte bounds for the two kinds that carry a key.
disposition: fixed in task `n5`: `no_kind_debug_shows_an_organisation_private_key_byte` in org-node/tests/secret_redaction.rs (LLR-4ces44) was changed, and failed with a Debug that renders a secret's bytes.

**finding-44**: guideline, low — org-node: floor, "every test fails when the behavior it `verifies:` breaks" — org-node/tests/absences.rs:55-94
- `org_node_holds_no_invitation_exchange` is the only test of LLR-96z2k5. It scans the source for eleven fixed names.
- A new operation that took joiner bytes under any other name would pass. The LLR's clause that an admitted Member is built "from that kept request alone" is never exercised.
disposition: not fixed — it needs a new behavioural test (task `n5`, docs/plans/2026-10-21-org-io-domain-api.md); LLR-96z2k5 keeps its source scan in org-node/tests/absences.rs. Carried open, not blocking.

### Round 6

**finding-1**: guideline, low — org-io: Clause: the floor, adopted in the file's preamble ("The compliance floor the skills state wins any conflict"), "class B and C: abnormal-input tests for every REQ and LLR". Where:
- DRAFT LLR-4qd5fx (96-103), LLR-ctm86b (105-111), LLR-5acgfg (113-120), LLR-b97usg (208-212), LLR-vvd8dn (561-573), LLR-k3zmqb (869-874), LLR-6vtbn6 (1032-1041), LLR-3ame3g (1160-1162), LLR-4k43dm (2043-2056), LLR-4usfu2 (2123-2137), LLR-gh4rw3 (2197-2204).
- Their tests: absences.rs:531, 654, 677, 1159, 1194, 1232, 1254, 1278, 1304, 1335, 1361; chain_read_rule.rs:85; rng_source.rs:84, 147; verification.rs:251.

What and why: each of these items is tested only by a source scan or by a test-build check. None has an abnormal-input test. Unlike its siblings, none has the "robustness, N/A" note this project uses to record that a no-input item has no abnormal case and that its scan was watched red. The siblings with the note are LLR-6sfub6:259, LLR-sn5ded:1664, LLR-n2srhc:2156, LLR-jqb5z7:2238 and LLR-ntag3g:2262, plus 2026-10-08-org-io.md:827-833, 1093 and 1104. On its face the floor is unmet for these eleven items. The fix is a note per item (or one shared note), not code.
disposition: not fixed — carried open, not blocking (the TEST org-io loop ended at round 6 with nothing above low, and no fix task took its lows; docs/plans/2026-10-21-org-io-domain-api.md); the scan LLRs, LLR-k3zmqb among them, keep their scans without an N/A note.

**finding-2**: guideline, low — org-io: Clause: the floor, "every test fails when the behavior it `verifies:` breaks". Where: DRAFT LLR-kz9hj4 1384-1396 (`ReceiveError::NoOrganisation { receiver }`, with `0x01` written); src/receiver.rs:199-204; the LLR-kz9hj4 tests at receive_step.rs:176-507.

What and why: no test reaches `NoOrganisation`; no test file names it. If the branch broke (wrong verdict, or a report under no Organisation), none of the tests verifying LLR-kz9hj4 would fail. The design records no ruling for it, though it does for the other own refusals that only an org-node double could produce (FoundingNotListed, AdmissionNotListed, ProposalNotListed, and the rav7dr/bv6wkz/3fxxpp listing failures). Either:
- add a test (a body org-node keeps in a founding store with no proxy, if one exists), or
- record the clause as untested, under the owner ruling and Doubles 1 as the others are.
disposition: not fixed — carried open, not blocking (the TEST org-io loop ended at round 6 with nothing above low, and no fix task took its lows; docs/plans/2026-10-21-org-io-domain-api.md); task `o3` had already left the unreached `ReceiveError::NoOrganisation` clause of LLR-kz9hj4 undone.

**finding-3**: guideline, low — org-io: Clause: the floor, "every test fails when the behavior it `verifies:` breaks". Where:
- DRAFT LLR-qa4nrt 1325-1332, Networked clause ("in Networked mode it dials the issuing Device's key with the Invite's direct addresses as a hint") and "one of any other form names no identity and is not dialled".
- LLR-v9dkrv 389-396, LLR-adgk9z 1288-1302 and LLR-uf5cyy 1338-1348, the addresses carried in Networked mode ("its direct addresses ... possibly none").
- Tests: requests.rs:137-161 (the only LLR-qa4nrt test, Loopback only), requests.rs:52-77 and 171-222, invitation_exchange.rs:80-98.

What and why: every test of these items runs in Loopback. None would fail if Networked `request_admission` dialled the Invite's socket instead of the issuer's key. None would fail if a malformed Invite address were dialled. None would fail if an Invite or request carried the wrong addresses in Networked mode. The Networked send test (send_fanout.rs:165-205) shows the in-process relay can reach these clauses. It builds its request through org-node (send_fanout.rs:145-163), so it bypasses `request_admission` and `issue_invite`. LLR-34tfm8 has the matching malformed-form test (send_fanout.rs:116-138); LLR-qa4nrt has none.
disposition: not fixed — carried open, not blocking (the TEST org-io loop ended at round 6 with nothing above low, and no fix task took its lows; docs/plans/2026-10-21-org-io-domain-api.md); LLR-qa4nrt's Networked clause is tested only in Loopback in org-io/tests/requests.rs.

**finding-4**: guideline, low — org-io: Clause: the floor, "every test fails when the behavior it `verifies:` breaks". Where:
- DRAFT LLR-pxqdy4 1194-1203 ("takes, under the node's lock ..., releases the lock, and sends each").
- LLR-3w22wc 1696-1697 ("The node's lock is not held while a dial or a verdict read is in progress").
- LLR-898z24 1497-1499 and LLR-eeh4pq 1530-1534 ("holds none while it sends an answer").
- Tests: send_fanout.rs:53-86 and 365-378; acknowledgements.rs:256-273; receive_step.rs:51-77, 339-374, 518-557.

What and why: the LLR-eeh4pq tests show a command completing only while `next_received` waits for a body. No test shows a command completing while any of these is in progress:
- a `send_organisation` dial or verdict read;
- an acknowledgement round's dial;
- an answer's send.

A regression that held the lock across the network (the PR-7v8anh class) would pass every test that verifies these clauses. A recipient that holds its verdict, as send_fanout.rs:318-360 already builds, would reach them.
disposition: not fixed — carried open, not blocking (the TEST org-io loop ended at round 6 with nothing above low, and no fix task took its lows; docs/plans/2026-10-21-org-io-domain-api.md); the lock-release clauses of LLR-pxqdy4, LLR-3w22wc, LLR-898z24 and LLR-eeh4pq have no test with a network step in progress.

**finding-5**: guideline, low — org-io: Clause: Owned services 1, "A double that returns the service's own error, such as a conflict response with its body, is this kind [a fake, under the same contract test as any fake], not a fault injector"; also Doubles 5, "A fake, with a contract test: a double that imitates service-specific content the code interprets". Where:
- acknowledgements.rs:447-475 (`already_received_or_invite_used_keeps_the_acknowledgement`, LLR-xwz3nu, LLR-3w22wc);
- send_fanout.rs:269-293 (`the_verdict_byte_maps_...`, LLR-fwz3tb);
- the stand-ins journey.rs:154-185 (`answering`), journey.rs:464-502 (`answering_acknowledgements`) and acknowledgements.rs:220-247 (`answering_once`).

What and why: the stand-in recipients script the owned receive step's own verdicts. `AlreadyReceived` (0x02) and `InviteUsedByAnotherDevice` (0x03) are bytes the real step writes for admission requests, and the code interprets them. The tests label them "fault injection". By Owned services 1 a double returning the owned service's own answer is a fake that needs a contract test. Here no contract test can hold, because the real step never writes these bytes for an acknowledgement or for an Organisation's information. These tests therefore verify against answers the shipped service never gives. The contract test `the_scripted_recipients_verdicts_are_the_real_receive_steps` (acknowledgements.rs:894-947) covers only `Accepted` and `Refused`.

If the intent is that a peer Device's answer is outside what the owned service guarantees (a peer may run other software), that reading needs recording as an owner ruling or a tailoring of Owned services 1, as the IPC paragraph was. The clause as written does not allow it.
disposition: not fixed — carried open, not blocking (the TEST org-io loop ended at round 6 with nothing above low, and no fix task took its lows; docs/plans/2026-10-21-org-io-domain-api.md); no owner ruling or tailoring records the peer-answer reading, and the contract `the_scripted_recipients_verdicts_are_the_real_receive_steps` covers Accepted and Refused only.

**finding-6**: guideline, low — org-io: Clause: Doubles 3, "Every fake has a contract test", and Doubles 5's line between a fake and a fault injector ("service-specific content the code interprets"). Where:
- src/test_support.rs:37-41 (`found_at_failing` fails `found_at` "at its `Fund` step");
- tests/common/chain_contract.rs:31 (classes "the writes' failures" as fault injectors);
- tests/common/chain_writer_contract.rs:10-13 (pins only `RecordGenesis`);
- uses at chain_write.rs:427-464 and 518-547, verification.rs:684-708, persona_creation.rs:134.

What and why: `found_at_failing` answers with on-chain-client's typed `WriteError::Step { step: Fund, .. }`. write.rs interprets that content: LLR-sq5pbq's report names the step, and chain_write.rs:460 asserts "Fund". That makes the switch a fake by Doubles 5, not the content-free fault injector chain_contract.rs:31 calls it. The production contract run pins only the `RecordGenesis` step's shape. A Fund-step refusal from the real writer is never compared with the fake's. The risk is small, since both build the same on-chain-client type. Still, either a Fund-step clause in the founding contract or a recorded reclassification would close it.
disposition: not fixed — carried open, not blocking (the TEST org-io loop ended at round 6 with nothing above low, and no fix task took its lows; docs/plans/2026-10-21-org-io-domain-api.md); it repeats low 5 of round 4 and low 8 of round 5, where pinning the Fund step was found to need the real writer on a fork (org-io/src/test_support.rs).

**finding-7**: guideline, low — org-node: floor (file header: "The compliance floor the skills state wins any conflict"): "every test fails when the behavior it `verifies:` breaks", with Existing suites 1, "These rules bind the tests a change writes or edits" — five locations:
- org-node/tests/absences.rs:74 cites REQ-qn2erx.
- org-node/tests/absences.rs:239 cites REQ-xs4ab8. The body of this test was edited (`l` renamed to `line`).
- org-node/tests/commit_workflow_errors.rs:60 cites LLR-2r2fha and LLR-kzgjz8. The helper this test calls was renamed in the diff.
- org-node/tests/encoding_golden.rs:201 (`revocation_wire_message_is_pinned`) cites LLR-ayrdr8, LLR-ms8njy and LLR-js9dsu. Its comment and body were edited.
- org-node/tests/fuzz_wire_decode/fuzz_target.rs:21 cites REQ-c29s93. This annotation line was rewritten in the diff.

What and why:
- The diff writes or edits each of these tests, but its `verifies:` still cites an item that this change retires:
  - REQ-qn2erx, REQ-xs4ab8 and REQ-c29s93 are superseded in DRAFT-worktree-org-io-domain-api-batched-epoch.md.
  - LLR-2r2fha and LLR-kzgjz8 are superseded by LLR-283x6f (DRAFT:1232).
  - LLR-ayrdr8 is superseded by LLR-mkvx2k (DRAFT:301).
  - LLR-ms8njy is superseded at DRAFT:1790.
  - LLR-js9dsu was already superseded on master by 2026-10-08-wire-kinds.md.
- The live items these tests exercise are not cited: LLR-283x6f, LLR-mkvx2k and LLR-xr88a8.
- The neighbouring tests in the same files were moved to the live items, for example encoding_golden.rs:233, :340, :366 and :417, and wire_frame_bound.rs:185 and :201. These five were missed.
- The annotations therefore name behaviour that is no longer required and leave the live LLRs uncredited here.
- **Why low:** the assertions are right; only the trace is wrong.
disposition: fixed in task `n6` (merged 2026-10-29; TEST low 1): the edited tests and four unedited ones now cite in-force items, for example `revocation_wire_message_is_pinned` in org-node/tests/encoding_golden.rs to LLR-mkvx2k and LLR-xr88a8, and the commit_workflow_errors.rs sender-refusals test to LLR-283x6f.

**finding-8**: guideline, low — org-node: floor: "every test fails when the behavior it `verifies:` breaks" — org-node/tests/own_removal.rs:250-252 (`into_removed_keeps_only_the_org_id_the_time_and_the_acknowledgements`, verifies LLR-2d5a7m).

What and why:
- The loop `for gone in [[0x60u8; 32], [0x62; 32], [0x11; 32]]` asserts "no Organisation key or member seed is kept". Two of the three patterns can never occur in `rich_data()`:
  - `[0x60; 32]` is no key in the fixture. The current record's Organisation private key is `RECORD_KEY = [0x3d; 32]` (tests/support/fixtures.rs:31, through `support::fixtures::record`), and nothing checks it.
  - `[0x11; 32]` is the member-key seed of Bob's snapshot, of which only the derived public key is stored.
  - The Persona's own member seed (`[1; 32]`, from `persona_record("p-alice", 1, OWN)`) is not checked either.
- Only the provisional key `[0x62; 32]` is really tested.
- LLR-2d5a7m says the record with its Organisation private key and the member seed are dropped. A removed form that kept them inside an acknowledgement's bytes would still pass this loop.
- **Why low:** the field-name assertion at :248 catches any added field, so the code is right and the gap is in this assertion only.
disposition: fixed in task `n6` (TEST low 2): `into_removed_keeps_only_the_org_id_the_time_and_the_acknowledgements` in org-node/tests/own_removal.rs (LLR-2d5a7m) checks `RECORD_KEY`, the provisional key and the member seed's byte count, and failed with `RECORD_KEY` appended to the removed bytes.

**finding-9**: guideline, low — org-node: floor: "every test fails when the behavior it `verifies:` breaks", read with LLR-f9mcsq "`check_received` removes nothing on any outcome … The test drives each" — org-node/tests/own_removal.rs:781-787 (`on_an_active_store_nothing_but_the_removal_deletes_...`, verifies LLR-f9mcsq).

What and why:
- On the rich store, the test drives `check_received` to only one non-refusal outcome, an answered organisation request, plus the two refusals at :749-751.
- These outcomes are never driven against the rich store: a provisional organisation kept, an admission request kept, a revocation notice kept, `AlreadyHeld`, and an acknowledgement decided.
- If one of those outcomes deleted a field, this test would still pass, so "on any outcome" is unmet.
- **Why low:** the code is right.
disposition: not fixed — each remaining `check_received` outcome needs its own staging (task `n6`, docs/plans/2026-10-21-org-io-domain-api.md); LLR-f9mcsq keeps its tests in org-node/tests/own_removal.rs. Carried open, not blocking.

**finding-10**: guideline, low — org-node: floor: "every test fails when the behavior it `verifies:` breaks", read with LLR-f3hru6 "it builds no trie and draws nothing" — org-node/tests/proposals.rs:771-875. These are the four tests that verify LLR-f3hru6, at :773, :799, :827 and :855.

What and why:
- Every test that verifies LLR-f3hru6 passes `OsRng`, and none checks that nothing is drawn.
- `support::CountingRng` exists for exactly this and is used for `propose_founding` and `propose_member` (proposals.rs:138, 238, 375, 408).
- If `record_revocation` began drawing from its random source, no test would fail.
- **Why low:** it is a gap, and the code is right.
disposition: fixed in task `n6` (TEST low 4): the LLR-f3hru6 tests use `CountingRng`, for example `a_revocation_refused_by_the_bound_records_nothing`, which failed with an extra draw. Noted for the owner: LLR-f3hru6 says "draws nothing" while a successful revocation draws the seal's 24-byte nonce; the tests read it as nothing beyond that nonce, and the plan carries it as an open low.

### .guardrails/templates/CODE_GUIDELINES.md

Governs: every code path of the diff master...worktree-org-io-domain-api in on-chain-client, org-node, org-io and app (src, tests, Cargo.toml and config comments, the Makefile), as the reports state their scope; `guidelines-file.sh CODE` printed this template for each path.

### Round 1

Round 1's reviewer reports were not kept; their text survives only as the plan's summaries in `docs/plans/2026-10-21-org-io-domain-api.md` ("Guideline reviews — round 1" and the fix-task records of `cfn`, `cfa`, `cfo` and `arch`), and each finding below is quoted from there.

**finding-1**: guideline, medium — app+on-chain-client: C-A1 — single-letter names (Names 1).
disposition: fixed by task `cfa` (merged 2026-10-25), recorded in `docs/plans/2026-10-21-org-io-domain-api.md` under "Round 1 fix — CODE app + on-chain-client + Makefile".

**finding-2**: guideline, medium — app+on-chain-client: C-A2 — TypeScript DTO names differ from the Rust DTOs; `verifyOrganisations` vs `verify_organisation` (Names 4).
disposition: fixed by task `cfa`: the TypeScript DTOs in `app/src/lib/api.ts` now carry the Rust names, and the wrapper is `proposeMember(inviteBlob, …)`. `verifyOrganisations` is kept, because LLR-j2x9jb names it.

**finding-3**: guideline, medium — app+on-chain-client: C-A3 — `parse_founder` repeats `parse_32_bytes` (Simplicity 4).
disposition: fixed by task `cfa`: `parse_founder` lives in `app/src-tauri/src/parsing.rs` and is built on `parse_32_bytes`.

**finding-4**: guideline, medium — app+on-chain-client: C-A4 — the JSON round trip in `verification_outcomes` adds untested `Unknown` and unparsable-id arms (Simplicity 1).
disposition: fixed by task `cfa` in `app/src-tauri/src/commands.rs`: there is no JSON round trip any more, and the read-back is matched exhaustively. `VerifiedKind::Unknown` stayed then, because receiver_events.rs exercised it through JSON input; task `a2` removed it in round 2 with the typed hand-off in `app/src-tauri/src/entry.rs` (AR2-3).

**finding-5**: guideline, medium — app+on-chain-client: C-A9 — comments that narrate the change, with task numbers and dates, in code, Cargo.toml and the Makefile (Comments 2).
disposition: fixed by task `cfa` (merged 2026-10-25), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`. Later narration in the `Makefile` and `app/src-tauri/Cargo.toml` was raised again in rounds 2 and 3 (see those rounds).

**finding-6**: guideline, low — app+on-chain-client: low 5 — the report's text was not kept; the plan names it only by number
disposition: fixed by task `cfa` together with C-A4 (the `verification_outcomes` read-back in `app/src-tauri/src/commands.rs`), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-7**: guideline, low — app+on-chain-client: low 6 — an adopted or removed founding with no proxy announcing `org_id: ""`
disposition: not fixed in round 1 (task `cfa`: it needed a design decision). Round 2 raised it again as C2-A1, and task `a2` fixed it test-first: `SelfDeleted` and `RecordUnreadable` carry an optional org id and the payload is `{org_id: null}`, never "" (LLR-acb5g4, LLR-ygr6gd amended; test `an_entry_with_no_org_id_names_none_in_record_unreadable_or_revoked`).

**finding-8**: guideline, low — app+on-chain-client: low 7 — the report's text was not kept; the plan names it only by number (its disposition names `WriteError::MalformedStorage(&'static str)`)
disposition: not fixed — task `cfa` kept `WriteError::MalformedStorage(&'static str)`, because it is public and named by LLR-htp5tk; recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-9**: guideline, low — app+on-chain-client: low 8 — T57's listing-label note: not present (own label at +page.svelte:119); a rejected subscription leaves it unshown (low C-A8).
disposition: fixed test-first by task `cfa`: `app/tests/notices.admission.test.ts` "a refused subscription at start is shown on the page, not left unhandled" (verifies LLR-5mw2dj) was red because no `.catch` ended the subscription chain, and passed after the +page.svelte fix.

**finding-10**: guideline, medium — org-node: C-N1 — change narration and task numbers in service.rs comments (Comments 2).
disposition: fixed by task `cfn` (merged 2026-10-25) in `org-node/src/service.rs`, recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-11**: guideline, medium — org-node: C-N2 — change narration and task numbers in service.rs comments (Comments 2).
disposition: fixed by task `cfn` (merged 2026-10-25) in `org-node/src/service.rs`, recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-12**: guideline, medium — org-node: C-N4 — single-letter names (Names 1).
disposition: fixed by task `cfn` (merged 2026-10-25), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-13**: guideline, medium — org-node: C-N5 — `filter_map(.. data().ok())` drops `OrganisationRemoved` with no reason given (Errors 2).
disposition: fixed as a comment by task `cfn`: a removed store keeps no PersonaRecord, so skipping it is the only possible behaviour, and its acknowledgements are read by DevicePublicKey (LLR-2d5a7m, LLR-6zaeb6, LLR-75sn9g).

**finding-14**: guideline, medium — org-node: C-N6 — peer-input decode failures reported as `OrgNodeError::Chain` (Errors 1).
disposition: partly fixed by task `cfn`: the peer-input site (`first_admission_base`) now refuses `InvalidField record_snapshot`. The local encode and derivation sites were left, because a new variant would force a new arm in the app's exhaustive match (the A2 ruling); they recur as lows in later rounds and as AR4-6, which task `n4` also left. Recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-15**: guideline, medium — org-node: C-N7 — person_store repeats the store's AEAD constants and unseal sequence (Simplicity 4).
disposition: fixed by task `cfn`: `labelled_cipher` and `open_sealed` in `org-node/src/store.rs` are shared with `org-node/src/person_store.rs`.

**finding-16**: guideline, medium — org-node: C-N8 — verify_org repeats `set_address` with retain+push, and the copies have drifted (Simplicity 4).
disposition: fixed by task `cfn`: `org-node/src/verify_org.rs` calls `set_address`.

**finding-17**: guideline, medium — org-node: C-N9 — wire kind indices and the `kind > 5` literal repeated in service.rs (Simplicity 4).
disposition: fixed by task `cfn`: the kind indices and `KIND_COUNT` live once in `org-node/src/wire.rs`.

**finding-18**: guideline, medium — org-node: C-N14 — "device key", "Device secret" and "person key" in prose and user text (Names 4).
disposition: fixed by task `cfn` (merged 2026-10-25), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-19**: guideline, low — org-node: low 3 — the report's text was not kept; the plan names it only by number
disposition: fixed by task `cfn`, recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-20**: guideline, low — org-node: low 10 — the report's text was not kept; the plan names it only by number (its disposition names `prune_after_adoption` and `adopt_founding`)
disposition: fixed by task `cfn`: `prune_after_adoption` is shared and `adopt_founding` is justified in `org-node/src/service.rs`.

**finding-21**: guideline, low — org-node: low 11 — whether a founding whose members build no trie should be an error rather than Behind
disposition: not fixed — carried open, not blocking; task `cfn` left it as an open design question, and no later round of `docs/plans/2026-10-21-org-io-domain-api.md` decides it.

**finding-22**: guideline, low — org-node: low 12 — the report's text was not kept; the plan names it only by number
disposition: fixed by task `cfn`, recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-23**: guideline, low — org-node: low 13 — the `""` invitee name when no invite matches
disposition: not fixed — task `cfn` left it as an open design question. It was raised again in rounds 2, 3 and 4 (the `admission_requests` fallback in `org-node/src/service.rs`), and task `n4` left it because each fallback would change behaviour without an LLR.

**finding-24**: guideline, low — org-node: low 15 — the report's text was not kept; the plan names it only by number
disposition: fixed by task `cfn`, recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-25**: guideline, low — org-node: low 16 — the report's text was not kept; the plan names it only by number (its kept part is the public `data()`)
disposition: partly fixed by task `cfn`; the public `data()` is design-named, so it was kept. Recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-26**: guideline, low — org-node: low 17 — the report's text was not kept; the plan names it only by number
disposition: fixed by task `cfn`, recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-27**: guideline, low — org-node: low 18 — the report's text was not kept; the plan names it only by number (its kept part is the one-variant `StoreChange`)
disposition: partly fixed by task `cfn`; the one-variant `StoreChange` in `org-node/src/service.rs` is design-named, so it was kept. Recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-28**: guideline, medium — org-io: C-O1 — change narration and task numbers in comments and Cargo.toml (Comments 2).
disposition: fixed by task `cfo` (merged 2026-10-27): the plan-task and change narration was removed from exchange.rs, persona.rs, verify.rs, write.rs, transport/mod.rs, view.rs and `org-io/Cargo.toml`.

**finding-29**: guideline, medium — org-io: C-O2 — dead `send_body_to_device` and test-only `send_body` behind `allow(dead_code)` (Simplicity 3).
disposition: fixed by task `cfo`: `send_body_to_device` and `SendReport::failure_summary` were deleted (nothing called them; LLR-3vb4k9, which named the first, is superseded), and `send_body` / `send_framed` became test-support with no `allow(dead_code)`.

**finding-30**: guideline, medium — org-io: C-O3 — a `kept_acknowledgements` error ends the delivery task silently (Errors 2).
disposition: owner ruling 2026-10-27: report and retry. Fixed test-first by task `arch` (merged 2026-10-28): only `PersonaNotHeld` ends the task, and any other refusal is that round's failure in `AcknowledgementReport::last_round_failure`, retried at the next due time (LLR-rav7dr and LLR-bv6wkz amended). Its red-first test was later removed by the owner ruling on T3-O2, which amended LLR-rav7dr to name the clause as untested.

**finding-31**: guideline, medium — org-io: C-O4 — `accepted()` drops org-node's refusal and the store-write failure (Errors 1/2).
disposition: fixed as a comment by task `cfo`: LLR-6ux6yw states the behaviour (a refusal writes nothing, a failed write reloads the Persona, the acknowledgement is kept and sent again), and the comment in `org-io/src/acknowledge.rs` says why no acknowledgement is lost.

**finding-32**: guideline, medium — org-io: C-O5 — a bind failure is reduced to "the direct bind failed" (Errors 1).
disposition: fixed by task `cfo`: the reason now carries the bind's error, `SenderEndpoint("the direct bind failed: <TransportError>")`; LLR-cqr26z's outcome is unchanged.

**finding-33**: guideline, medium — org-io: C-O6 — `list_removed` / `list_stores` errors become empty lists (Errors 2).
disposition: partly fixed by task `cfo`, with two defects fixed test-first (LLR-3fxxpp): a failed listing no longer stops the hourly leftover retry (`a_leftover_is_retried_hourly_though_the_listing_failed_when_receiving_started`), and the deletion retry's listing failure is reported and retried (`a_retry_that_cannot_list_the_data_directory_is_reported_and_retried_hourly`). The remainder followed owner ruling 2026-10-27 (the handle remembers the removed-store keys) and was fixed test-first by task `arch` (`a_removal_open_found_is_reported_and_resumed_though_the_directory_cannot_be_listed`; SDD-3my66m, LLR-5rxqr8 and LLR-rav7dr amended).

**finding-34**: guideline, medium — org-io: C-O7 — org-node query errors dropped across verify, send, lib, exchange, write and persona (Errors 2).
disposition: fixed as comments by task `cfo` at every site (`org-io/src/verify.rs`, send.rs, lib.rs, exchange.rs, write.rs, persona.rs): each fallback either iterates the listed Personas or runs right after a successful org-node call under the same lock, so no refusal can reach it.

**finding-35**: guideline, medium — org-io: C-O11 — two `holder_of` functions with different rules (Names 4).
disposition: fixed by task `cfo`: `record_holder_of` (`org-io/src/lib.rs`) matches a current record only, and `bound_holder_of` (`org-io/src/send.rs`) matches a record, a founding or a provisional Member; each doc names the other.

**finding-36**: guideline, medium — org-io: C-O12 — `remove_store` removes only `removed-` files (Names 3).
disposition: left open by task `cfo` (the design items name the function). Fixed by task `arch` (merged 2026-10-28): `remove_store` is renamed `remove_removed_file` in `org-io/src/store_file.rs`, with LLR-vj2nkb, LLR-3fxxpp, LLR-vvd8dn and LLR-zcw3xv updated.

**finding-37**: guideline, low — org-io: low 8 — a clock before 1970
disposition: not fixed in round 1 (task `cfo`: it needed a design decision). Round 2 raised it as C2-O1, and task `o2` fixed it test-first: a clock before 1970 is never time 0 (`a_clock_before_the_epoch_is_the_rounds_failure_and_nothing_is_sent_or_deleted`, `open_judges_no_deletion_on_a_clock_before_the_epoch_and_reports_it`; LLR-bv6wkz, LLR-rav7dr).

**finding-38**: guideline, low — org-io: low 9 — the report's text was not kept; the plan names it only by number
disposition: fixed as a comment by task `cfo` (r1-fix-cfo: org-node's `verify_org::judge` returns `InStep` only against a held record, so epoch 0 is never reported). The fallback itself was raised again in rounds 3 and 4, and task `o4` removed it in `org-io/src/verify.rs`.

**finding-39**: guideline, low — org-io: low 10 — the report's text was not kept; the plan names it only by number
disposition: fixed as comments by task `cfo` in `org-io/src/receiver.rs` (r1-fix-cfo: a lookup miss there means the request was removed in between, so there is nothing to report).

**finding-40**: guideline, low — org-io: low 13 — the report's text was not kept; the plan names it only by number
disposition: fixed by task `cfo` (r1-fix-cfo: DevicePublicKey and Device seed prose in the lines the review listed), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-41**: guideline, low — org-io: low 15 — the report's text was not kept; the plan names it only by number
disposition: fixed by task `cfo` (r1-fix-cfo: `PersonSummary::of(person_data)`), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-42**: guideline, low — org-io: low 16 — the report's text was not kept; the plan names it only by number
disposition: fixed by task `cfo` (r1-fix-cfo: `CreateError::Store` reads "the Device seed is in the keychain, but …", true for a Person and a Persona), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-43**: guideline, low — org-io: low (unnumbered) — LLR-rav7dr has no "accepted but not written" outcome
disposition: not fixed — carried open, not blocking; task `cfo` listed it as open (a new outcome value needs a design decision), and no later round of `docs/plans/2026-10-21-org-io-domain-api.md` adds the outcome to LLR-rav7dr.

### Round 2

**finding-1**: guideline, medium — app+on-chain-client: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — `app/src-tauri/src/events.rs:360`
`outcomes_for_verification` reads `org_id` with `.unwrap_or_default()`. For a provisional founding with no proxy recorded the entry's `org_id` is JSON `null`, and `commands.rs` (`verification_outcomes`, the `(Adopted, None)` arm) expects that case. The missing id becomes `""`, so the code emits `RecordUnreadable { org_id: "" }` (adopted) or `SelfDeleted { org_id: "" }` (removed). No requirement names the empty string as a default. LLR-jf63e4 says "null, never an empty string", and LLR-acb5g4 and LLR-ygr6gd say "naming the Organisation".
disposition: fixed test-first by task `a2` (merged 2026-10-28) as C2-A1: `SelfDeleted` and `RecordUnreadable` carry an optional org id and the payload is `{org_id: null}`, never ""; the webview shows "a provisional founding". LLR-acb5g4 and LLR-ygr6gd amended; red-first test `an_entry_with_no_org_id_names_none_in_record_unreadable_or_revoked`.

**finding-2**: guideline, low — app+on-chain-client: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — `app/src/lib/notices.ts:93`
`|| 'the invitee'` fills in a name when org-io reports neither the name nor the surname. Neither LLR-k5rf3r nor LLR-n64wvr names this default text; LLR-n64wvr only says such a field is `null`.
disposition: not fixed — task `a2` (merged 2026-10-28) lists it among the lows not fixed; the fallback is still in `app/src/lib/notices.ts`, and the round 3 reviewer read it as a requirement-named display default. Carried open, not blocking.

**finding-3**: guideline, low — app+on-chain-client: Errors 2, "no discarded return code or error value … Where an error is expected and handled, the handling says why that is safe" — `app/src-tauri/src/commands.rs:535`
The new emit closure in the receiver loop discards `emit`'s error with `let _ =` and gives no reason there. The other two discards say why: the bind-failure branch at :521 and `lib.rs:46`.
disposition: fixed by task `a2` (merged 2026-10-28) (C2 low 3): the discard in the receiver loop of `app/src-tauri/src/commands.rs` says why it is safe.

**finding-4**: guideline, low — app+on-chain-client: Names 1, "Every name the change writes … says what it holds" — `app/src-tauri/src/periodic.rs:54,56`; `on-chain-client/src/write/ceremony.rs:33,54`
The type parameter `E` in `run_periodic_verification<E>` is one letter, while its sibling `periodic_runs` names the same role `Emit`. The new `create_proxy<O: WriteOps>` and `found_at<O: WriteOps>` also add the one-letter type parameter `O`.
disposition: not fixed — task `a2` (merged 2026-10-28) lists it among the lows not fixed; the type parameters remain in `app/src-tauri/src/periodic.rs` and `on-chain-client/src/write/ceremony.rs`. Carried open, not blocking.

**finding-5**: guideline, low — app+on-chain-client: Names 1, "a loop index and a callback parameter included, says what it holds" — `app/src/lib/components/Membership.svelte:118,137`; `app/src/lib/components/PersonaList.svelte:74`
The new `catch (e)` blocks in `proposeAgain`, `confirmProposeAgain` and `writeFounding` should name what they catch, for example `refusal`, as `founding.ts` and `+page.svelte` do.
disposition: fixed by task `a2` (merged 2026-10-28) (C2 low 5): the `catch` bindings in `app/src/lib/components/Membership.svelte` and PersonaList.svelte name what they catch.

**finding-6**: guideline, low — app+on-chain-client: Names 3, "no `data`, `info`, `item`, `value`, `result` … alone" — `app/src-tauri/src/commands.rs:676,745`; `app/src/lib/verification.ts:17,48,79`
- `VerificationEntryDto.result` and `DeviceReportDto.result` are struct fields named `result`.
- `resultLine(result)` and `verificationLine(result)` take a parameter named `result`; it holds a verification entry.
- `fileVerificationResults(list, …)` takes a parameter named `list`, a type word.
disposition: not fixed by task `a2` (merged 2026-10-28). The `app/src/lib/verification.ts` parameter names were raised again in round 3 and fixed by task `a3`; the DTO fields named `result` are serde tags the round 4 reviewer treated as tool-fixed names. Carried open, not blocking.

**finding-7**: guideline, low — app+on-chain-client: Names 3, "A name says what the thing is in the project's domain" — `app/src-tauri/src/commands.rs:725`
`org_id_for` returns an `EntryOrg`, which is either an id or a founder plus an optional id. It does not return an org id, so the name misstates what comes back.
disposition: fixed by task `a2` (merged 2026-10-28) (C2 low 7): the function in `app/src-tauri/src/commands.rs` is now `entry_org_for`, named for the `EntryOrg` it returns.

**finding-8**: guideline, low — app+on-chain-client: Names 4, "where the codebase already names it, that name" — `app/src-tauri/src/commands.rs:231` (user-facing text), plus prose throughout
`docs/CONTEXT.md` names the concept **DevicePublicKey** and lists "device key" under _Avoid_ for "prose and user-facing text". `device_key` is the in-code identifier. The change uses:
- **Prose:** "Device key" in the refusal text `org-io lists no Persona with Device key …` (commands.rs:231). It also appears in doc comments in `commands.rs` (PersonDto, ProposedFoundingDto, `persona_id_for`, EntryOrg), `parsing.rs` (`parse_founder`), `api.ts` (PersonDto, ProposedFoundingDto, EntryOrg), `founding.ts`, `notices.ts` (`admissionConfirmation`), `CreateOrg.svelte` and `PersonaList.svelte`.
- **DTO fields:** one value has four field names: `device_public_key` (PersonDto, AdmissionRequestDto), `issuer_device_key` (InviteDto), `device` (DeviceReportDto) and the codebase's `device_key`.
disposition: not fixed by task `a2` (merged 2026-10-28). The user-facing refusal text was fixed in round 3 by task `a4` (`persona_id_for` in `app/src-tauri/src/commands.rs` now says "DevicePublicKey"); the four DTO field names remain. Carried open, not blocking.

**finding-9**: guideline, low — app+on-chain-client: Names 4, "A concept has one name in every file, test and task of the change" — `app/src/lib/api.ts:253`
The wrapper for the `verify_organisation` command is named `verifyOrganisations`. Every other wrapper takes its command's name, and the file header (api.ts:4) says signatures "mirror commands.rs exactly".
disposition: not fixed — `verifyOrganisations` in `app/src/lib/api.ts` is named by LLR-j2x9jb (round 1 C-A2) and by the app DRAFT design (round 4, task `a5`, low 4), so it is kept.

**finding-10**: guideline, low — app+on-chain-client: Names 2, "no chained conditional expression standing in for an `if`" — `app/src/lib/components/Invite.svelte:233`
`requestBusy ? 'Requesting…' : confirmedBefore ? 'Request admission again' : 'Request admission'` is a chained ternary.
disposition: not fixed — task `a2` (merged 2026-10-28) lists it among the lows not fixed (`app/src/lib/components/Invite.svelte`). Carried open, not blocking.

**finding-11**: guideline, low — app+on-chain-client: Simplicity 2, "no interface, base class, factory or wrapper without a second caller" — `app/src-tauri/src/periodic.rs:47`; `app/src-tauri/src/events.rs:265`
- `periodic_run` is a one-line wrapper over `commands::verification_outcomes(org_io, None)` with one caller (`run_periodic_verification`). Together with `periodic_runs` there are three layers for one call path, and the tests drive only `run_periodic_verification`.
- `FollowUp` is a one-variant enum that is always wrapped in `Option`. `Option<String>` (the org id to verify) carries the same information.
disposition: not fixed by task `a2` (merged 2026-10-28). `periodic_run` was deleted in round 4 by task `a5` (low 5); the one-variant `FollowUp` in `app/src-tauri/src/events.rs` remains, carried open, not blocking.

**finding-12**: guideline, low — app+on-chain-client: Comments 2, "A comment describes the code as it is, never the change that wrote it" — `app/src-tauri/src/commands.rs:38-39`; `Makefile:407-408`; `app/src-tauri/Cargo.toml:23`
- "(LLR-pguhw5's six fields, unchanged)" is change narration: "unchanged" means unchanged by this change.
- "BELOW the floor of 82. The floor is not lowered: the shortfall goes to the owner (merge verification record)" narrates a decision taken at this change.
- "# No rand: …" exists only to mark the dependency this change removed. The reason belongs in the commit message.
disposition: fixed by task `a2` (merged 2026-10-28) (C2 low 12), as recorded in `docs/plans/2026-10-21-org-io-domain-api.md`. The `Makefile` block and the `app/src-tauri/Cargo.toml` "No rand" line were raised again in round 3 and remain, carried open, not blocking.

**finding-13**: guideline, medium — org-io: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-io/src/lib.rs:493; org-io/src/acknowledge.rs:205, 273, 299, 310; org-io/src/retention.rs:151, 161, 204
`clock.now().unwrap_or(0)` turns a clock that reads before the Unix epoch (`Clock::now` returns `None`, clock.rs:4) into time 0, and nothing reports it. This is the same value that verify.rs:238 refuses as `UnverifiedReason::ClockBeforeEpoch`. With 0, `open` deletes nothing that is due. `deliver` never finds a deletion due. `wait_until` sleeps `due - 0` seconds, about 1.7e9, so the task hangs with no report. The retry tasks are anchored at 0 + 3600. The change states nowhere that 0 is the default.
disposition: fixed test-first by task `o2` (merged 2026-10-28) as C2-O1 (dispatcher ruling, consistent with the owner's report-and-retry ruling): at open no deletion is judged and the condition is reported; in the delivery task it is that round's failure, retried an hour later. Tests `a_clock_before_the_epoch_is_the_rounds_failure_and_nothing_is_sent_or_deleted` and `open_judges_no_deletion_on_a_clock_before_the_epoch_and_reports_it` (LLR-bv6wkz, LLR-rav7dr, LLR-upqj5g).

**finding-14**: guideline, medium — org-io: Errors 2, "No swallowed error ... no discarded return code or error value ... Where an error is expected and handled, the handling says why that is safe" — org-io/src/retention.rs:180
`matches!(service.kept_acknowledgements(device_key), Ok(Some(_)))` treats an org-node refusal as "not held". With a data directory and no `removed-` file, the `(false, Some(data_dir))` arm then returns `Ok(())`. The task forgets the removal's progress and its removed key, and ends. The Persona is still held and is not deleted, and no failure is reported. No comment says why discarding the `Err` is safe here.
disposition: fixed test-first by task `o2` (merged 2026-10-28) as C2-O2: in `org-io/src/retention.rs` only `PersonaNotHeld` or `Ok(None)` means "not held", and any other failure is reported and retried hourly. Its red-first test was removed in round 3 by the owner ruling on T3-O2, and LLR-3fxxpp was amended to name that clause as untested.

**finding-15**: guideline, low — org-io: Errors 2, "no discarded return code or error value ... the handling says why that is safe" — org-io/src/acknowledge.rs:115, 184, 208, 233; org-io/src/retention.rs:122
These also drop an `Err` from `kept_acknowledgements` and treat it as "no kept removal": `matches!(.., Ok(Some(_)))`, `let Ok(Some(kept)) = kept else { return | continue }` and `.ok().flatten()`. None of these sites says why that is safe. Similar sites elsewhere in the change carry such a comment (lib.rs `record_holder_of`, verify.rs:167, exchange.rs:89). At acknowledge.rs:184, the doc comment "A store holding no kept removal starts none" covers `Ok(None)`, not a refusal.
disposition: fixed by task `o2` (merged 2026-10-28) (low C2-O3): each `kept_acknowledgements` discard in `org-io/src/acknowledge.rs` and retention.rs says why it is safe.

**finding-16**: guideline, medium — org-io: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it" — org-io/src/acknowledge.rs:402 (`dialable`) and org-io/src/send.rs:132 (`sockets_for`)
Both functions state one rule: parse each kept address with `parse_kept_address`, then in Loopback mode keep only the addresses whose identity is the recipient's key, and in Networked mode keep every parsed one. `sockets_for` adds the typed refusals. `dialable` reimplements the filter instead of calling it, so a change to the Loopback rule has to be made twice.
disposition: fixed by task `o2` (merged 2026-10-28) as C2-O4: `dialable` is deleted and the delivery uses `send::sockets_for` in `org-io/src/send.rs`.

**finding-17**: guideline, medium — org-io: Names 4, "A concept has one name in every file, test and task of the change" — org-io/src/send.rs:34
`pub type AdmissionOutcome = Delivery;` gives one concept, one recipient's delivery outcome, two public names. Both are re-exported (lib.rs:49), and the tests use both. `request_admission` returns `AdmissionOutcome` while every other send path returns `Delivery`.
disposition: fixed by task `o2` (merged 2026-10-28) as C2-O5: the `AdmissionOutcome` alias is deleted and `request_admission` returns `Delivery` (`org-io/src/send.rs`).

**finding-18**: guideline, low — org-io: Names 4, "where the codebase already names it, that name" — org-io/src/lib.rs:103 (`acks`), org-io/src/lib.rs:687 (`ack_context`)
docs/CONTEXT.md names the concept **Removal acknowledgement** and lists "_Avoid_: ack". The change itself uses `acknowledge`, `AcknowledgementReport` and `acknowledgement_report` elsewhere.
disposition: fixed by task `o2` (merged 2026-10-28) (low C2-O6): the field is now `deliveries` in `org-io/src/lib.rs`.

**finding-19**: guideline, low — org-io: Names 3, "A name says what the thing is in the project's domain" — org-io/src/lib.rs:96 (`publish: tokio::sync::Mutex<write::PublishState>`), org-io/src/write.rs:240 (`PublishState`)
The change deletes `publish.rs`. This state now holds the Unknown writes and the last Verification outcome of each target (write.rs:236-243), and neither is a "publish". Its own doc calls it "What org-io keeps between calls". A name such as `unknown_writes_and_verifications` would say what it holds.
disposition: not fixed — task `o2` (merged 2026-10-28) lists C2-O7 among the lows not fixed; `PublishState` remains in `org-io/src/write.rs`. Carried open, not blocking.

**finding-20**: guideline, low — org-io: Names 4, "One name per concept across the change" — org-io/src/write.rs:32-33
"The chain write as org-io performs it (org-io's own seam for the submission tests)." This change removes the submission (`submit.rs`, `tests/submit_flow.rs`) and calls the concept the chain write (`write_organisation_to_chain`, tests/chain_write.rs). The trait doc keeps the old name.
disposition: not fixed — task `o2` (merged 2026-10-28) lists C2-O8 among the lows not fixed (`org-io/src/write.rs`). Carried open, not blocking.

**finding-21**: guideline, low — org-io: Names 4, "One name per concept across the change" — org-io/src/store_file.rs:140, 178, 185 (`list_person_stores`, `read_person_store`, `write_person_store`)
The same change calls this file the "Person file" in the docs, errors and comments (lib.rs `open_person`, `OpenError::MoreThanOnePerson` "Person files", store_file.rs:91 "The Person file of ..."). The functions call it a "person store". CONTEXT.md's **Store file** is "one Persona's SealedStore ... One per Persona", so the Person's file is a distinct thing with one name in prose and another in code.
disposition: not fixed — task `o2` (merged 2026-10-28) lists C2-O9 among the lows not fixed (`org-io/src/store_file.rs`). Carried open, not blocking.

**finding-22**: guideline, low — org-io: Names 1, "Every name the change writes ... says what it holds" — org-io/src/write.rs:264
`async fn bounded<T>(...)`: the type parameter is a single character the change chose, not one the language fixes. A name such as `Written` would say what it holds.
disposition: not fixed by task `o2` (merged 2026-10-28) (C2-O10). Round 4 raised it again and task `o4` fixed it: the type parameter of `bounded` in `org-io/src/write.rs` is now `Written`.

**finding-23**: guideline, low — org-io: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-io/src/acknowledge.rs:239
When no kept removal is found but a deletion failure is recorded, the report is built with `deadline: 0`. view.rs documents `deadline` as "The kept verification time plus 7 days", so 0 is a placeholder that reads as a real time (the Unix epoch). The field should be optional or the case reported some other way.
disposition: not fixed — task `o2` (merged 2026-10-28) lists C2-O11 among the lows not fixed; round 3 raised the `deadline: 0` placeholder again (`org-io/src/acknowledge.rs`). Carried open, not blocking.

**finding-24**: guideline, low — org-io: Errors 1, "An error is reported where it is detected, naming what failed and on what input" — org-io/src/transport/mod.rs:541 (`parse_kept_address` returns `Option`), consumed at org-io/src/send.rs:138 and org-io/src/acknowledge.rs:405
A malformed kept address is dropped by `filter_map` and nothing names it. In Loopback mode, a recipient whose kept addresses are all malformed is then reported as `AddressNotARecipient` or `NoAddress`, which misnames the cause. This needs corrupted disk or wire text, hence low.
disposition: not fixed by task `o2` (merged 2026-10-28) (C2-O12). The `filter_map` over `parse_kept_address` was raised again in rounds 3 and 4, and task `o4` fixed it in round 4 (`org-io/src/send.rs`).

**finding-25**: guideline, low — org-io: Errors 1, "An error is reported where it is detected ... and the code stops or returns an error its caller must handle" — org-io/src/retention.rs:93
`let _ = finish_removed(data_dir, &device_key);` The comment explains why nothing is lost. However, the failure (keychain item or file) is reported nowhere, not at open and not on each hourly retry, while the module doc (retention.rs:8-9) states "a failed step is reported and retried".
disposition: not fixed by task `o2` (merged 2026-10-28) (C2-O13). The unreported `finish_removed` failure was raised again in rounds 3 and 4, and task `o4` fixed it in round 4 (`org-io/src/retention.rs`).

**finding-26**: guideline, low — org-io: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-io/src/receiver.rs:163
`.pop().map_or(Delivery::NotDelivered { reason: SendError::NoAddress }, ..)`: an empty send report silently becomes "no address", and no comment says why that cannot happen. `send_all` given one `Outgoing` always returns one entry, so the code is right, but the fallback misnames the case if it is ever reached.
disposition: fixed by task `o2` (merged 2026-10-28) (low C2-O14) in `org-io/src/receiver.rs`.

**finding-27**: guideline, low — org-io: Simplicity 3, "Code no path reaches, an unused function, import or variable ... are deleted" — org-io/src/rng.rs:38-42
`impl Default for HandleRng` has no caller. `LocalStore::default`, `open_with` and test_support all call `HandleRng::os()` or `replaced_by` directly, and no `#[derive(Default)]` holds a `HandleRng`.
disposition: not fixed by task `o2` (merged 2026-10-28) (C2-O15). Round 4 raised it as C4-O2, and task `o4` deleted `impl Default for HandleRng` from `org-io/src/rng.rs`.

**finding-28**: guideline, low — org-io: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays" — org-io/src/persona.rs:51-56 and org-io/src/persona.rs:90-95
`propose_founding_organisation` and `propose_member` repeat the same steps: keep the Device seed in the keychain, drop the store on a refusal, write the store, then `accept_if_receiving` after the lock is released. This is the rule of SDD-bzu2jg, "keychain item, then store file, then its endpoint", stated twice.
disposition: not fixed — task `o2` (merged 2026-10-28) lists C2-O16 among the lows not fixed (`org-io/src/persona.rs`). Carried open, not blocking.

**finding-29**: guideline, low — org-io: Comments 2, "No change narration ... no ... ticket history" — org-io/Cargo.toml:82, org-io/src/test_support.rs:312
"(TEST review round 1, finding-6)" and "(the contract test of finding-6)" record which review round produced the code. This is history that the commit and the review records carry, and it describes nothing about the code as it is.
disposition: fixed by task `o2` (merged 2026-10-28) (low C2-O17). The `org-io/Cargo.toml` review-history comment was raised again in rounds 3 and 4 and removed by task `o4` in round 4.

**finding-30**: guideline, low — org-io: Names 2, "Write code to be read, not to be short: one action per statement" — org-io/src/verify.rs:272, org-io/src/receiver.rs:198
At verify.rs:272, `let record_epoch = ...;        match outcome {` puts two statements on one line, so the `match` that carries the function's result hides at the end of a `let`. At receiver.rs:198, `let root =service...` is unformatted. Both read as unfinished edits. rustfmt would fix both.
disposition: fixed by task `o2` (merged 2026-10-28) (low C2-O18): `org-io/src/verify.rs` and receiver.rs are formatted, one statement per line.

**finding-31**: guideline, medium — org-node: Names 4, "One name per concept across the change ... where the codebase already names it, that name" — org-node/src/service.rs:1584 (`pub enum AdmissionVerdict`, re-exported at lib.rs:39)
`docs/CONTEXT.md` already defines **Verdict** as the one byte a receive side writes back (accepted or refused), and this change's own glossary entry for Removal acknowledgement says "Not a **Verdict**". `AdmissionVerdict` uses the same word for a different concept: a three-way outcome (`Accepted`, `AlreadyReceived`, `InviteUsedByAnotherDevice`) of keeping an admission request. It is public, so readers of org-io will take it for the transport Verdict. Rename it, e.g. `AdmissionRequestOutcome`.
disposition: fixed by task `n2` (merged 2026-10-28) as C2-N1: `AdmissionVerdict` is now `AdmissionRequestOutcome` and its field is `outcome`; LLR-94rzrk, LLR-6gkxpg and LLR-zkybw7 amended.

**finding-32**: guideline, medium — org-node: Errors 1, "An error is reported where it is detected, naming what failed and on what input" — org-node/src/error.rs:163, 184, 188, 198, 233
Several new or reworded refusals carry no input, though the code has the input to hand when it raises them:
- `InviteNotOutstanding` (163) leaves out the `(org_id, invite_id)` that was not outstanding.
- `NoAdmissionRequestKept` (184) and `AdmissionAlreadyRecorded` (188) leave out the `invite_id`.
- `HandleTaken` (198) leaves out the Handle.
- `PersonaNotHeld` (233) leaves out the DevicePublicKey.

The other variants in the enum name their `org_id`/`member_id`, as `MemberNotHeld` and `RevocationAlreadyRecorded` do in this same change. These ones say only "no request is kept for this invite" and similar.
disposition: fixed by task `n2` (merged 2026-10-28) as C2-N2: `InviteNotOutstanding { org_id, invite_id }`, `NoAdmissionRequestKept { invite_id }`, `AdmissionAlreadyRecorded { invite_id }`, `HandleTaken { handle }` and `PersonaNotHeld { device_key }` in `org-node/src/error.rs`, plus a new `PersonaIdNotHeld`; LLR-3sddey and LLR-f9x3vf amended; red-first test, now named `the_stated_variants_are_distinct_and_their_display_names_the_value`.

**finding-33**: guideline, low — org-node: Errors 1, "naming what failed" — org-node/src/store.rs:420, 1352, 1365; org-node/src/person_store.rs:141; org-node/src/service.rs:1434
New encoding failures are reported as `OrgNodeError::Chain(..)`, whose message is `"chain read failed: {0}"` (error.rs:44). So a failure to encode the epoch state, a removed store, the Person store or a record snapshot reads as "chain read failed: person store encode: …", which names the wrong thing that failed. This needs an encoder failure, which is rare.
disposition: not fixed — task `n2` (merged 2026-10-28) does not name it. It was raised again in round 3 and as AR4-6 in round 4, which task `n4` left because renaming or adding `OrgNodeError` variants (`org-node/src/error.rs`) changes a public enum the app matches exhaustively. Carried open, not blocking.

**finding-34**: guideline, low — org-node: Errors 1, "naming what failed" — org-node/src/error.rs:68
The `ProvisionalLimit` doc (65–67) and `ensure_within_bound` (store.rs:410–424) count provisional changes, provisional organisations **and kept revocation notices**. The message says only "provisional changes and organisations for one Organisation would exceed {limit} bytes". A refusal caused by kept notices therefore names the wrong contents.
disposition: not fixed — task `n2` (merged 2026-10-28) left C2-N4, because the fix changes a pinned Display string (`org-node/src/error.rs`). Carried open, not blocking.

**finding-35**: guideline, low — org-node: Errors 1, "naming what failed" — org-node/src/service.rs:1353; org-node/src/removal.rs:127
When there is no current record, `remove_own` refuses with `OrgNotOnChain` (`data.record.as_ref().ok_or(OrgNodeError::OrgNotOnChain)`), and `into_removed` does the same for a store bound to no Organisation. The thing missing is the local record, not a chain state. `current_record` (service.rs:1416), the helper for this exact state, names it `OrgNotHeld`/`FoundingNotWritten`. Both paths are unreachable in normal use (`judge` and `accepted_notice` need a record), hence low.
disposition: not fixed — task `n2` (merged 2026-10-28) does not name it. It was raised again in rounds 3 and 4, and task `n4` left it (CODE low 6: a new or renamed `OrgNodeError` variant changes a public enum the app matches). Carried open, not blocking; `org-node/src/service.rs`.

**finding-36**: guideline, low — org-node: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-node/src/service.rs:539, 621, 1172
- 539: `base_root: base_root.unwrap_or(root)` stores the answer's own root as a stand-in base when the store has no record. The comment says nothing reads it, but no requirement is cited for the default.
- 621: `last_committed_epoch: record_epoch.unwrap_or_default()` silently uses epoch 0 when there is no record. There is no comment and no requirement.
- 1172: `.map_or("", ..)` returns an empty invitee name when no outstanding invite matches. The doc says this "is never returned in practice", which is exactly the case where an error should surface.
disposition: not fixed — task `n2` (merged 2026-10-28) does not name it. It was raised again in rounds 3 and 4, and task `n4` left it (CODE low 5: each fallback in `org-node/src/service.rs` would change behaviour without an LLR). Carried open, not blocking.

**finding-37**: guideline, low — org-node: Errors 2, "No swallowed error ... no discarded return code or error value" — org-node/src/removal.rs:34
In the test-support `PartialEq for StoreContent`, `postcard::to_allocvec(self).ok() == postcard::to_allocvec(other).ok()` makes two values whose encoding fails compare equal, so an encode error turns into "equal". This happens only in test builds and only on an encoder failure.
disposition: not fixed by task `n2` (merged 2026-10-28). It was raised again in rounds 3 and 4, and task `n4` fixed it in round 4 (CODE low 7, `org-node/src/removal.rs`).

**finding-38**: guideline, low — org-node: Names 4, "One name per concept across the change ... where the codebase already names it, that name" — org-node/src/removal.rs:52, 67, 94; org-node/src/service.rs:397, 1364
`docs/CONTEXT.md` (Removal acknowledgement) lists `_Avoid_: ack`. The change adds the field `KeptAcknowledgement::ack`, the raw field `ack`, the parse label `"removed.ack.device"`, and the locals `ack` in `check_received` and `remove_own`. Elsewhere in the same change the concept is spelled `acknowledgement` (`acknowledgements_kept`, `AcknowledgementNotKept`, `acknowledgement_accepted`).
disposition: not fixed — task `n2` (merged 2026-10-28) left the `ack` names, because renaming them changes encoding labels (`removed.ack.device` in `org-node/src/removal.rs`). Carried open, not blocking.

**finding-39**: guideline, low — org-node: Names 4, "One name per concept across the change" — org-node/src/verify_org.rs:49 vs org-node/src/service.rs:981, 1100, 1355 and org-node/src/removal.rs:165
`own` means two things. In `Verified::Adopted { own: bool }` it is "the adopted provisional organisation was this node's own proposal" (`Origin::Own`). In `verify`, `messages_for_current_epoch`, `remove_own` and `removal::recipients` it is this Persona's DevicePublicKey.
disposition: not fixed — task `n2` (merged 2026-10-28) does not name it (`org-node/src/verify_org.rs`). Carried open, not blocking; recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-40**: guideline, low — org-node: Names 4, "One name per concept across the change" — org-node/src/store.rs:951 (`StoreData::member`), used e.g. at verify_org.rs:83, 194 and service.rs:839, 879
The field holding the **Provisional Member** is called `member`. Throughout the same code `member` also names a `MemberSnapshot` of the record (`|member| member.device_keys…`, including verify_org.rs:199, next to line 194). The glossary term is Provisional Member, so name the field after it (e.g. `provisional_member`).
disposition: not fixed — task `n2` (merged 2026-10-28) does not name it; the field is still `member` in `org-node/src/store.rs`. Carried open, not blocking.

**finding-41**: guideline, low — org-node: Names 4, "One name per concept across the change" — org-node/src/service.rs:979–980
`if let Judgement::Adopt(matched) = judgement { let matched = &data.provisional_organisations[matched]; …` uses the name `matched` first for an index and then for the provisional organisation. Line 992 uses `matched` as the index again.
disposition: not fixed by task `n2` (merged 2026-10-28). It was raised again in rounds 3 and 4, and task `n4` fixed the `matched` naming in round 4 (CODE low 8, `org-node/src/service.rs`).

**finding-42**: guideline, low — org-node: Names 3, "A name says what the thing is in the project's domain ... no ... `process` alone" — org-node/src/service.rs:1405
`fn mark(data, verified)` is a bare verb. Its doc calls it the "behind marker", which suggests a name like `update_behind_marker`. A reader at the call site (service.rs:989) has to open the body to see what gets marked.
disposition: not fixed by task `n2` (merged 2026-10-28), which does not name it; the function in the reviewed tree is `update_behind_marker` (`org-node/src/service.rs`).

**finding-43**: guideline, low — org-node: Names 2, "no chained conditional expression standing in for an `if`, no trick that saves a line" — org-node/src/verify_org.rs:316, 319–320
`(!included).then(|| DiscardedChange::Admit {..})` uses `bool::then` in place of an `if`. In the `Revoke` arm, `let included = !adopted…any(..); (!included).then_some(..)` is a double negation: the arm returns `Some` when the member is still present. A plain `if` reads directly.
disposition: not fixed by task `n2` (merged 2026-10-28). It was raised again in rounds 3 and 4, and task `n4` fixed it in round 4 (CODE low 10, `org-node/src/verify_org.rs`).

**finding-44**: guideline, low — org-node: Simplicity 2, "no interface, base class, factory or wrapper without a second caller" — org-node/src/store.rs:38; org-node/src/person_store.rs:101
- `parse_each` has one caller (store.rs:395).
- `person_store::cipher` is a one-line wrapper over `labelled_cipher` with one caller (person_store.rs:126).
disposition: not fixed — task `n2` (merged 2026-10-28) does not name it (`org-node/src/store.rs`, `org-node/src/person_store.rs`). Carried open, not blocking.

**finding-45**: guideline, low — org-node: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it" — org-node/src/verify_org.rs:199 vs 143–148; org-node/src/service.rs:531 vs 635; org-node/src/service.rs:475 vs store.rs:1157–1169
- `adopt_first` writes out the same check as `lists`: `listed.member_key == … && listed.device_keys.contains(…)`.
- The provisional-organisation identity check `held.root == root && held.org_pub_key == org_pub_key` is added twice, in `keep_organisation_answer` and in `keep_information`.
- `answer_organisation_request` repeats `set_address`'s "entry unchanged" test before calling it, although `set_address` already returns whether the entry changed.
disposition: not fixed — task `n2` (merged 2026-10-28) does not name it (`org-node/src/verify_org.rs`, `org-node/src/service.rs`). Carried open, not blocking.

### Round 3

**finding-1**: guideline, low — app+on-chain-client: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it" — app/src-tauri/src/commands.rs:111 (`VerificationDto::of`) and app/src-tauri/src/entry.rs:122 (`entry_for`). Both encode one rule: which `Verified` variant becomes which status, with `Refused`/`Store`/`Keychain` becoming `unverified` carrying org-io's text and `Settled` recursing into its outcome. They are two copies that can drift: a new `Verified` variant, or a change to how a store failure is worded, has to be made in both. A single mapping (e.g. `VerificationDto` derived from `EntryResultDto`, with `NotYetVerified` as the one extra case) would serve both.
disposition: not fixed — round 3's CODE review of this unit had nothing above low, so its loop closed (`docs/plans/2026-10-21-org-io-domain-api.md`). Round 4 raised the same pair again, and task `a5` left it: merging duplicated product code (`app/src-tauri/src/entry.rs`, commands.rs) is structural. Carried open, not blocking.

**finding-2**: guideline, low — app+on-chain-client: Simplicity 4, "one copy stays, where both can reach it" — app/src-tauri/src/events.rs:63 (`AdmissionRequested`) and app/src-tauri/src/commands.rs:774 (`AdmissionRequestDto`). These are two structs with the same seven fields. Each has its own hex/`to_string` mapping (events.rs `From<Received> for Arrival`, commands.rs `From<&KeptAdmissionRequestSummary>`). The `AdmissionRequestDto` doc says it is "the `admission-requested` event's fields and encodings (LLR-dp45mm)", and api.ts uses one TS type for both. That makes it one rule held in two copies.
disposition: not fixed — round 3's CODE review of this unit had nothing above low, so its loop closed. Round 4 raised the same pair again, and task `a5` left it: merging duplicated product code (`app/src-tauri/src/events.rs`, commands.rs) is structural. Carried open, not blocking.

**finding-3**: guideline, low — app+on-chain-client: Simplicity 4, "one copy stays" — app/src/lib/notices.ts:32 (`AdmissionCheck`) and app/src/lib/invite.ts:12 (`RequestCheck`). Both are new in this change and both are `{ ok: true } | { ok: false; message: string }`, the same shape as revoke.ts's existing `RevokeCheck`. That is three copies of one gate-result type.
disposition: not fixed — carried open, not blocking (`app/src/lib/notices.ts`, `app/src/lib/invite.ts`); round 3's CODE review of this unit had nothing above low, so its loop closed and no task names it.

**finding-4**: guideline, low — app+on-chain-client: Simplicity 4, "one copy stays" — app/src-tauri/src/commands.rs:229 (`persona_id_for`) and app/src-tauri/src/commands.rs:623 (`entry_org_for`, Founding arm). Both scan `org_io.personas()` for the Persona whose `device_key` equals a given key, and each projects a different field. A single "Persona by Device key" lookup would serve both. As finding-5 shows, the two copies already differ on what a miss means.
disposition: not fixed — round 3's CODE review of this unit had nothing above low, so its loop closed. Round 4 raised the same pair again, and task `a5` left it: merging duplicated product code (`app/src-tauri/src/commands.rs`) is structural. Carried open, not blocking.

**finding-5**: guideline, low — app+on-chain-client: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — app/src-tauri/src/commands.rs:633. In `entry_org_for`, if no Persona has the founder's key, `.and_then(|persona| persona.org_id)` silently yields `org_id: None`. That is the same value as "founding whose proxy is not recorded", so the entry claims a state it never checked. `persona_id_for` (line 229) treats the same miss as an error naming the key. The doc's "none before" names only the not-yet-recorded case, not a missing Persona.
disposition: not fixed — carried open, not blocking (`app/src-tauri/src/commands.rs`); round 3's CODE review of this unit had nothing above low, so its loop closed and no task names it.

**finding-6**: guideline, low — app+on-chain-client: Errors 1, "An error is reported where it is detected, naming what failed and on what input" — app/src/lib/founding.ts:38-39. `refusal as PersonRefusalDto` is an unchecked cast. Any rejection of `proposePerson` that is not the command's DTO has no `kind` or `message`: for example a Tauri IPC error, which rejects with a plain string. Such a rejection passes the `kind !== 'person_already_held'` test and runs `throw message`, which throws `undefined`. The form then shows "undefined", and the text of the real error is lost.
disposition: fixed by task `a3` (merged 2026-10-28): the founding decision is the pure `personRefusalStep` in `app/src/lib/founding.ts`, and a plain-string rejection is shown as its text, never undefined (LLR-d8u93q; red-first tests in `app/tests/founding.decision.test.ts`).

**finding-7**: guideline, low — app+on-chain-client: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — app/src/lib/verification.ts:68-69. In `verificationLabel`, the `default:` arm covers `not_checked` and also any status this view does not know, and it labels both "not checked". The sibling `resultLine` in the same file (line 31 onward) shows an unknown result as unverified, and its doc cites HAZ-9fmhm4. Here an unknown status is silently given a non-alarming label. An explicit `case 'not_checked'` plus a default that reads as unverified would match `resultLine`.
disposition: fixed test-first by task `a3`: an unknown status reads unverified (LLR-njb6mr amended; `app/src/lib/verification.ts`).

**finding-8**: guideline, low — app+on-chain-client: Simplicity 3, "an unused function, import or variable ... are deleted" — app/src-tauri/src/commands.rs:29-32. `pub use crate::entry::{...}` re-exports `DeviceReportDto`, `DeviceResultDto`, `DiscardedDto` and `WriteLabelDto`. commands.rs does not use them, and nothing in the crate or its tests imports them through `commands::`: tests/receiver_events.rs takes `WriteLabelDto` from `entry::`. The result is two import paths for the same types, and one of them is dead.
disposition: not fixed in round 3. Round 4 raised it again, and task `a5` fixed it (low 5): the `entry` re-exports are gone from `app/src-tauri/src/commands.rs`.

**finding-9**: guideline, low — app+on-chain-client: Simplicity 2, "no interface, base class, factory or wrapper without a second caller" — app/src-tauri/src/periodic.rs:47. `periodic_run` is a one-line wrapper over `commands::verification_outcomes(org_io, None)` and has one caller (line 62). The same holds for `verify_all_and_send` (commands.rs, under line 688), which only drops the `Option<OrgId>` from `verified_entries` for its single caller, `verify_organisation`.
disposition: not fixed in round 3. Round 4 raised `periodic_run` again, and task `a5` deleted it (low 5) from `app/src-tauri/src/periodic.rs`; the `verify_all_and_send` part is carried open, not blocking.

**finding-10**: guideline, low — app+on-chain-client: Simplicity 2, "No parameter, flag, setting or hook without a caller that uses it now" — app/src-tauri/src/events.rs:269 and app/src-tauri/src/events.rs:214.
- `FollowUp` is an enum with one variant (`Verify { org_id }`), where `Option<String>` says the same thing.
- `Arrival` is an intermediate type that maps one to one from `Received` and one to one onto `ReceiverOutcome` (`outcome_for_arrival` only renames variants), so every new arrival is written three times.
disposition: not fixed — carried open, not blocking (`app/src-tauri/src/events.rs`); round 3's CODE review of this unit had nothing above low, so its loop closed, and the round 4 reviewer judged `FollowUp` read by its own path.

**finding-11**: guideline, low — app+on-chain-client: Simplicity 2, "No parameter, flag, setting or hook without a caller that uses it now" — app/src-tauri/src/commands.rs:726 and app/src/lib/notices.ts:66.
- `WriteDto::Included.verification` is `Option<_>` with `skip_serializing_if`. Its own doc says it is present for every included write, and `follow_up::verifies_after_write` returns `true` for every `Included`, so no path produces `None`. api.ts declares the field required.
- `RequestMessage.severity` has the single value `'error'`, and nothing reads it: Invite.svelte renders only `.text`.
disposition: not fixed — carried open, not blocking (`app/src-tauri/src/commands.rs`, `app/src/lib/notices.ts`); round 3's CODE review of this unit had nothing above low, so its loop closed, and the round 4 reviewer noted both without a finding.

**finding-12**: guideline, low — app+on-chain-client: Names 1, "No single-character names. Every name the change writes, a loop index and a callback parameter included, says what it holds" — app/src-tauri/src/periodic.rs:54 and app/src/lib/components/PersonaList.svelte:74.
- `run_periodic_verification<E>` uses the type parameter `E`. The function just above it names its parameters `Run`, `RunFuture` and `Emit`.
- `writeFounding` uses `catch (e)`. The same change uses `catch (refusal)` in Membership.svelte and +page.svelte.
disposition: fixed by task `a3` (C3 finding 12), as recorded in `docs/plans/2026-10-21-org-io-domain-api.md`; the type parameter `E` of `run_periodic_verification` is still in `app/src-tauri/src/periodic.rs`.

**finding-13**: guideline, low — app+on-chain-client: Names 3, "A name says what the thing is in the project's domain, not its type or a role in general: no `data`, `info`, `item`, `value`, `result` ... alone" — app/src/lib/verification.ts:17, :48 and :79, and app/src/lib/components/Invite.svelte:68.
- `resultLine(result)` and `verificationLine(result)` name the `VerificationEntryDto` `result`, which leads to `result.result`.
- `fileVerificationResults(list, ...)` names the filed results by their type, `list`.
- Invite.svelte's `let read = $state<InviteDto | null>` names the Invite after an action.
disposition: fixed by task `a3` (C3 finding 13) in `app/src/lib/verification.ts` and `app/src/lib/components/Invite.svelte`, recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-14**: guideline, low — app+on-chain-client: Names 4, "A concept has one name in every file, test and task of the change" — app/src-tauri/src/commands.rs:176 and app/src-tauri/src/entry.rs:161.
- The glossary's DevicePublicKey crosses IPC as `device_public_key` (PersonDto, AdmissionRequestDto, `admission-requested`), as `issuer_device_key` (InviteDto) and as `device` (DeviceReportDto).
- Separately, the TS wrapper for the `verify_organisation` command is `verifyOrganisations` (app/src/lib/api.ts:255). Every other wrapper in api.ts is the camelCase of its command.
disposition: not fixed — `verifyOrganisations` is kept because LLR-j2x9jb and the app DRAFT design name it (round 4, task `a5`, low 4); the DTO field names in `app/src-tauri/src/commands.rs` are carried open, not blocking.

**finding-15**: guideline, low — app+on-chain-client: Names 4, "one name per concept across the change" — app/src/lib/components/Membership.svelte:85 and app/src/lib/notices.ts:149. The same `AdmissionReproposal` value is called `reproposal` in `ReproposalStep` and `again` in `reproposalStep`'s local and in Membership's `confirming.again`. The step's field and the component's field hold the same value under two names.
disposition: not fixed — carried open, not blocking (`app/src/lib/components/Membership.svelte`); round 3's CODE review of this unit had nothing above low, so its loop closed and no task names it.

**finding-16**: guideline, low — app+on-chain-client: Comments 1, "the code already says what it does, and a comment that repeats it drifts from it at the next edit" — app/src-tauri/src/commands.rs:79, :686-687 and :488, and app/src-tauri/src/periodic.rs:4. These comments have already drifted from the code:
- `OrgDto::new`'s doc still speaks of "`verification` as `OrgIo::verification` returned it", but the parameter is now `last`.
- `verify_and_send`'s doc says it is the body "the receiver loop's ... and the periodic run's" verification runs, but the receiver loop and the periodic run reach `verified_entry`/`verified_entries` through `verification_outcomes` and never call `verify_and_send`.
- periodic.rs's module doc ("over `verify_and_send`") says the same wrong thing.
disposition: not fixed — carried open, not blocking (`app/src-tauri/src/commands.rs`, `app/src-tauri/src/periodic.rs`); round 3's CODE review of this unit had nothing above low, so its loop closed and no task names it.

**finding-17**: guideline, low — app+on-chain-client: Comments 1, "a comment that repeats it drifts" — app/src-tauri/src/state.rs:70-73. The `verification_interval` doc says the same thing twice: "which `run`'s setup hands the periodic verification (SDD-tgu7dp). Read once by `run`'s setup, which hands it to `periodic::run_periodic_verification` (SDD-tgu7dp)."
disposition: not fixed — carried open, not blocking (`app/src-tauri/src/state.rs`); round 3's CODE review of this unit had nothing above low, so its loop closed, and the round 4 reviewer judged it a redundancy, not a clause breach.

**finding-18**: guideline, low — app+on-chain-client: Comments 2, "A comment describes the code as it is, never the change that wrote it: no 'added', 'now uses', 'previously', 'fixed' or ticket history" — Makefile:407-418 and app/src-tauri/Cargo.toml:23.
- The ORG_IO_BRANCHES block keeps a 2026-10-25 reading "180 of 224 branches, 80.36% — BELOW the floor of 82 ... the shortfall goes to the owner". It then adds "Re-measured 2026-10-28 (worktree-org-io-domain-api, round 2 review fixes, task o2) ... 86.32%". The first is no longer true of the code, and the second is task history.
- Cargo.toml's "# No rand: ..." describes a dependency that was removed. It means something only next to the previous manifest.

The new measurement blocks for ORG_MEMBERS, ON_CHAIN_CLIENT and ORG_IO lines/regions follow the Makefile's established dated-measurement convention. I counted them under this finding only where they now contradict a later reading.
disposition: not fixed — carried open, not blocking; round 3's CODE review of this unit had nothing above low, so its loop closed. Both the `Makefile` block and the `app/src-tauri/Cargo.toml` line are still in the tree; the round 4 reviewer did not flag the Cargo.toml line, since it states a standing constraint.

**finding-19**: guideline, low — org-io: Simplicity 3, "an unused function, import or variable ... are deleted" — org-io/src/rng.rs:38
`impl Default for HandleRng` has no caller. Every construction calls `HandleRng::os()` directly (lib.rs:129, lib.rs:529, test_support.rs:453), and no `#[derive(Default)]` type holds a `HandleRng`.
disposition: not fixed in round 3 (task `o3` (merged 2026-10-28) did the other CODE lows only where named). Round 4 raised it as C4-O2, and task `o4` deleted `impl Default for HandleRng` from `org-io/src/rng.rs`.

**finding-20**: guideline, low — org-io: Simplicity 2, "No parameter, flag, setting or hook without a caller that uses it now" — org-io/src/acknowledge.rs:245
`report` takes a `stores: impl FnOnce() -> Vec<DevicePublicKey>` hook. Its one caller (lib.rs:724) passes `|| self.deliveries.0.removed_keys()`, which `report` could read itself from the `deliveries` argument it already takes. The hook gives that caller nothing.
disposition: fixed by task `o3` (merged 2026-10-28) (CODE finding-2): `report` in `org-io/src/acknowledge.rs` reads the removed keys from `deliveries` itself, and the `stores` hook is gone.

**finding-21**: guideline, low — org-io: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays" — org-io/src/lib.rs:355, org-io/src/exchange.rs:40, org-io/src/send.rs:92
The change adds the same refusal three times. `OperationError::NotHeld`, `ExchangeError::NotHeld` and `SendRefused::NotHeld` each carry `{ org_id }` and print the same text, `"no Persona here holds Organisation {}"`. All three are fed by the same `record_holder_of`/`bound_holder_of` lookup.
disposition: not fixed in round 3 (task `o3` (merged 2026-10-28): other CODE lows not done). Round 4 raised the repeated `NotHeld` text again (low 5), and task `o4` fixed it (`org-io/src/send.rs`).

**finding-22**: guideline, low — org-io: Simplicity 4, "one copy stays, where both can reach it" — org-io/src/send.rs:211-217 and org-io/src/send.rs:296-302
`send_organisation` and `request_organisation` each map org-node's messages to `Outgoing` with an identical closure (`kept_for(&service, &<sender>, message.recipient)` and then `Outgoing { recipient, body, addresses }`). This is one helper in two copies.
disposition: not fixed in round 3 (task `o3` (merged 2026-10-28): other CODE lows not done). Round 4 raised the repeated `Outgoing` closure again (low 5), and task `o4` fixed it in `org-io/src/send.rs`.

**finding-23**: guideline, low — org-io: Errors 2, "No ... discarded return code or error value ... Where an error is expected and handled, the handling says why that is safe" — org-io/src/send.rs:135
`addresses.iter().filter_map(|text| parse_kept_address(text))` drops every kept address that does not parse, and no comment says why. In Loopback mode a store whose addresses all fail to parse is then reported as `AddressNotARecipient` ("no kept address is the recipient's"). That hides a malformed kept address behind a refusal that means something else.
disposition: not fixed in round 3 (task `o3` (merged 2026-10-28): other CODE lows not done). Round 4 raised it again (low 7), and task `o4` fixed it in `org-io/src/send.rs`.

**finding-24**: guideline, low — org-io: Errors 1, "An error is reported where it is detected, naming what failed and on what input" — org-io/src/send.rs:165
A read-back of two bytes becomes `UnknownVerdict(first byte)`, which prints "verdict byte 0x{byte} is none of the four". When the first byte is a valid verdict (for example `0x00` followed by a stray byte), that message is false. What actually failed is the read-back's length.
disposition: not fixed — carried open, not blocking (`org-io/src/send.rs`); task `o3` (merged 2026-10-28) records the other CODE lows as not done.

**finding-25**: guideline, low — org-io: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-io/src/acknowledge.rs:263
When no kept removal is found but a deletion failure is recorded, the report sets `deadline: 0`. `AcknowledgementReport::deadline` is documented as "The kept verification time plus 7 days" (view.rs). A missing deadline is therefore reported as the Unix epoch, and no cited requirement names that default. It should be a value that says "no deadline", such as `Option`.
disposition: not fixed — carried open, not blocking (`org-io/src/acknowledge.rs`); task `o3` (merged 2026-10-28) records the other CODE lows as not done.

**finding-26**: guideline, low — org-io: Errors 3, "A missing ... value is an error unless a requirement names the default" — org-io/src/verify.rs:277
`Verified::InStep { epoch: record_epoch.unwrap_or(Epoch::new(0)) }` falls back to epoch 0 when the held record is missing. The comment argues the fallback is never reached. Under the clause it should still be an error (for example `Verified::Refused`), not a made-up epoch reported as in step.
disposition: not fixed in round 3 (task `o3` (merged 2026-10-28): other CODE lows not done). Round 4 raised it again (low 6), and task `o4` fixed it in `org-io/src/verify.rs`.

**finding-27**: guideline, low — org-io: Errors 2, "Where an error is expected and handled, the handling says why that is safe" — org-io/src/verify.rs:211
`service.persona_view(&founder).map_err(|_| UnverifiedReason::NotHeld)?` discards org-node's refusal with no stated reason. The same mapping in the same file (verify.rs:167-168) and in write.rs:300-301 states why, but this one does not.
disposition: fixed by task `o3` (merged 2026-10-28) (CODE finding-9): the `persona_view` mapping in `org-io/src/verify.rs` says why the refusal is safe to drop.

**finding-28**: guideline, low — org-io: Errors 1, "naming what failed and on what input" — org-io/src/write.rs:183-186
`WriteRefused::NotHeld`, `NoProxyAccount`, `Stale` and `OutcomeUnknown` carry no target. Their messages say "that target" or "the Organisation" without naming which one, while the sibling refusals added by this change (`OperationError::NotHeld { org_id }` and others) do name it. `NotHeld` is also used for two different failures in `write_update` (lines 355-356 and 363): no Persona holds the record, and no own provisional organisation exists.
disposition: not fixed — carried open, not blocking (`org-io/src/write.rs`); task `o3` (merged 2026-10-28) records the other CODE lows as not done.

**finding-29**: guideline, low — org-io: Errors 2, "no error output sent away where the caller needs it" — org-io/src/retention.rs:93 (against retention.rs:8-9)
`finish_leftovers` discards each `finish_removed` failure with `let _ =`. The comment explains the retry, but the failure is never recorded for the report. The module doc says "a failed step is reported and retried hourly". For a leftover `removed-` file's keychain or file step, nothing is reported, so the doc's claim does not hold.
disposition: not fixed in round 3 (task `o3` (merged 2026-10-28): other CODE lows not done). Round 4 raised it again (low 8), and task `o4` fixed it in `org-io/src/retention.rs`.

**finding-30**: guideline, low — org-io: Names 4, "A concept has one name in every file ... and where the codebase already names it, that name" — org-io/src/propose.rs:25, org-io/src/persona.rs:18, org-io/src/write.rs:231
The Organisation public key has three names in this change:
- `org_public_key` in `ProposedOrganisation` and `ProposedFounding`
- `org_pub_key` in `OrgSummary` (view.rs:51), which is also the name org-node's views use and the code copies from (propose.rs:80, persona.rs:64)
- `key` in `UnknownWrite`
disposition: not fixed — carried open, not blocking (`org-io/src/propose.rs`, persona.rs, write.rs); task `o3` (merged 2026-10-28) records the other CODE lows as not done.

**finding-31**: guideline, low — org-io: Names 4, "not `account_id` in one file and `customer_id` in another for the same value" — org-io/src/receiver.rs:33, org-io/src/receiver.rs:251
`Received::AdmissionRequestKept` names the requester's DevicePublicKey `from`. `KeptAdmissionRequestSummary` (view.rs) names it `device_key`, and the code copies it across (`from: kept.device_key`), even though the doc at receiver.rs:19-20 says the field types mirror that summary.
disposition: not fixed — carried open, not blocking (`org-io/src/receiver.rs`); task `o3` (merged 2026-10-28) records the other CODE lows as not done.

**finding-32**: guideline, low — org-io: Names 4, "One name per concept across the change" — org-io/src/send.rs:256, org-io/src/send.rs:285
`request_admission` and `request_organisation` name the sending Persona's DevicePublicKey `device`. Every other new site, including `send_organisation` a few lines above, names it `device_key` or `sender`.
disposition: not fixed — carried open, not blocking (`org-io/src/send.rs`); task `o3` (merged 2026-10-28) records the other CODE lows as not done.

**finding-33**: guideline, low — org-io: Names 3, "no `data`, `info`, `item`, `value`, `result`, `manager`, `handle` or `process` alone" — org-io/src/lib.rs:539
`let mut handle = Self::unconfigured(service, local);` names an `OrgIo` with the bare word `handle`, which the clause lists. In this crate `handle` also names the glossary's **Handle** (docs/CONTEXT.md:37, for example `PersonaSummary.handle` and `ProvisionalChangeSummary::Admission { handle }`), so the same word means two concepts.
disposition: not fixed — carried open, not blocking (`org-io/src/lib.rs`); task `o3` (merged 2026-10-28) records the other CODE lows as not done.

**finding-34**: guideline, low — org-io: Comments 2, "no 'added', 'now uses', 'previously', 'fixed' or ticket history" — org-io/Cargo.toml:82
"Chopsticks-only, as `preflight` (TEST review round 1, finding-6)" records the review history that produced the comment. That history belongs in the commit and the change records.
disposition: not fixed in round 3 (task `o3` (merged 2026-10-28): other CODE lows not done). Round 4 raised it again (low 4), and task `o4` removed the comment from `org-io/Cargo.toml`.

**finding-35**: guideline, low — org-node: Errors 1, "An error is reported where it is detected, naming what failed" — org-node/src/person_store.rs:141, org-node/src/store.rs:420, org-node/src/store.rs:1352, org-node/src/store.rs:1365, org-node/src/service.rs:1437
These new lines report a postcard *encode* failure as `OrgNodeError::Chain(..)`. That variant's message is `"chain read failed: {0}"` (error.rs:44), so a person-store, epoch-state, removed-store or record-snapshot encode failure shows up as "chain read failed: …". No chain is read on these paths. This is low because an in-memory encode practically never fails.
disposition: not fixed — carried open, not blocking; round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task. Round 4 raised it as AR4-6, and task `n4` left it, because a new `OrgNodeError` variant (`org-node/src/error.rs`) changes a public enum the app matches exhaustively.

**finding-36**: guideline, low — org-node: Errors 1, "naming what failed and on what input" — org-node/src/service.rs:1356
`remove_own` refuses a store with no current record as `OrgNotOnChain` ("no on-chain state found for org"). What is missing is the store's record, not chain state. `OrgNotHeld` or `current_record` (service.rs:1419) names that case. The path is unreachable today, because both callers reach it only with a record.
disposition: not fixed — carried open, not blocking; round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task. Round 4 raised it again, and task `n4` left it (CODE low 6, a public enum the app matches; `org-node/src/service.rs`).

**finding-37**: guideline, low — org-node: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-node/src/service.rs:539, org-node/src/service.rs:1172, org-node/src/service.rs:621
- service.rs:539 stores `base_root.unwrap_or(root)`: in a store with no record, an organisation answer gets its own root as a stand-in base root. The comment says why no reader looks at it, but no requirement names that default. A persisted field now holds a value that is not what its name says.
- service.rs:1172 `admission_requests` falls back to `""` for the invitee name when no outstanding invite matches. The doc comment says it is "never returned in practice", but no requirement names an empty name.
- service.rs:621 `record_epoch.unwrap_or_default()` sets `last_committed_epoch` to epoch 0 when there is no record. No requirement is cited for that default.
disposition: not fixed — carried open, not blocking; round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task. Round 4 raised it again, and task `n4` left it (CODE low 5: each fallback in `org-node/src/service.rs` would change behaviour without an LLR).

**finding-38**: guideline, low — org-node: Errors 2, "no discarded return code or error value … Where an error is expected and handled, the handling says why that is safe" — org-node/src/removal.rs:34
The test-only `PartialEq for StoreContent` compares `postcard::to_allocvec(self).ok() == postcard::to_allocvec(other).ok()`. Two values that both fail to encode compare equal (`None == None`), and the encode error is thrown away with no stated reason. It is test-support code, and encode failure is contrived.
disposition: not fixed in round 3 (round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task). Round 4 raised it again, and task `n4` fixed it (CODE low 7, `org-node/src/removal.rs`).

**finding-39**: guideline, low — org-node: Names 4, "where the codebase already names it, that name" — org-node/src/removal.rs:52 (also removal.rs:67, removal.rs:94 field label `removed.ack.device`, service.rs:1367, service.rs:1372)
The new public field `KeptAcknowledgement::ack`, its raw mirror and the local `let ack` name the Removal acknowledgement `ack`. `docs/CONTEXT.md` lists `ack` under _Avoid_ for **Removal acknowledgement**, and the same change's wire.rs and service.rs spell it `acknowledgement`.
disposition: not fixed — carried open, not blocking; round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task, and round 2's task `n2` left the `ack` names because renaming them changes encoding labels (`org-node/src/removal.rs`).

**finding-40**: guideline, low — org-node: Names 4, "One name per concept across the change" — org-node/src/revocation.rs:158
`sign_acknowledgement` refuses a secret whose DevicePublicKey is not the Persona's with `DeviceSecretNotSupplied` ("device secret … was not supplied"). The codebase already names exactly that case `DeviceSecretMismatch` (error.rs:151, "A device secret whose Device public key is not the one named"), and the same change uses that name in `open_sealed` (store.rs) and `PersonStore::unseal`. One concept now has two names.
disposition: not fixed — carried open, not blocking; round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task (`org-node/src/revocation.rs`).

**finding-41**: guideline, low — org-node: Names 4, "two names read as two concepts" (here, one name for two things) — org-node/src/service.rs:979–992
In `verify`, `matched` is first the `usize` index from `Judgement::Adopt(matched)`. Line 980 shadows it with the `&ProvisionalOrganisation`. Lines 992–993 bind `matched` again as an index. Within one function the same name holds an index and an organisation.
disposition: not fixed in round 3 (round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task). Round 4 raised it again, and task `n4` fixed the `matched` naming (CODE low 8, `org-node/src/service.rs`).

**finding-42**: guideline, low — org-node: Names 3, "A name says what the thing is in the project's domain, not … a role in general" — org-node/src/service.rs:1408
`fn mark(data, verified)` does not say what it marks. The body sets or clears the behind marker, so a name such as `mark_behind` or `update_behind_marker` would say it.
disposition: not fixed in round 3 (round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task); the function in the reviewed tree is `update_behind_marker` (`org-node/src/service.rs`).

**finding-43**: guideline, low — org-node: Names 2, "no chained conditional expression standing in for an `if`, no trick that saves a line" — org-node/src/verify_org.rs:319–320 (also verify_org.rs:316)
`let included = !adopted…any(|member| member.id == *member_id); (!included).then_some(..)` is a double negation feeding `bool::then_some` where an `if` is meant. `included` is true when the member is *gone*, so the reader has to undo two negations to see that a `Revoke` is discarded when its member is still listed.
disposition: not fixed in round 3 (round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task). Round 4 raised it again, and task `n4` fixed it (CODE low 10, `org-node/src/verify_org.rs`).

**finding-44**: guideline, low — org-node: Simplicity 2, "no interface, base class, factory or wrapper without a second caller" — org-node/src/service.rs:115–117
This change removed `StoreChange::Forgotten`, so `StoreChange` is now an enum with one variant, `Sealed(StoreWrite)`, that only wraps `StoreWrite`. Every return site has to write `StoreChange::Sealed(write)`, and no second variant exists to justify the enum.
disposition: not fixed — carried open, not blocking; round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task, and round 1's task `cfn` kept the one-variant `StoreChange` (`org-node/src/service.rs`) as design-named.

**finding-45**: guideline, low — org-node: Simplicity 3, "an unused function, import or variable … are deleted" — org-node/src/invitation.rs:17
`pub use crate::ids::{…, MAX_ENDPOINT_ADDRESSES, MAX_ENDPOINT_ADDRESS_BYTES}` re-exports two constants that nothing reaches through `invitation`. No file in org-node, org-io or app uses them outside `ids.rs`, and `ids` is itself a public module.
disposition: not fixed — carried open, not blocking; round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task (`org-node/src/invitation.rs`).

**finding-46**: guideline, low — org-node: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays" — org-node/src/removal.rs:18–28 against org-node/src/store.rs:1270, 1279, 1350, 1362
The store-form tag rule (0 = Active, 1 = Removed) is written twice. Once it is implied by `StoreContent`'s derived serde (the variant index, which removal.rs:15 documents and the golden tests decode). Once it is hand-coded as the literals `0u8` and `1u8` in `seal_data` and `seal_content`, and as `Some(0)` and `Some(1)` in `unseal`. Reordering the enum would change one copy and not the other.
disposition: not fixed — carried open, not blocking; round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task (`org-node/src/removal.rs`, `org-node/src/store.rs`).

**finding-47**: guideline, low — org-node: Names 3/4, "A name says what the thing is in the project's domain"; "where the codebase already names it, that name" — org-node/src/store.rs:951 (also `PersonaView::member_of`, store.rs)
`StoreData.member` holds the store's **Provisional Member** (`docs/CONTEXT.md`, _Avoid_: Persona, applicant). In the same functions, `member` is also the closure name for `MemberSnapshot`s of the record, for example verify_org.rs:411 `store.member` beside `listed`/`member` snapshots, and service.rs:875/879. Dropping "provisional" lets one name stand for two concepts.
disposition: not fixed — carried open, not blocking; round 3's CODE review of this unit had nothing above low, so its loop closed with no fix task (`org-node/src/store.rs`).

### Round 4

**finding-1**: guideline, medium — app+on-chain-client: Comments 2, "No change narration. A comment describes the code as it is, never the change that wrote it: no "added", "now uses", "previously", "fixed" or ticket history." Locations:
- `app/src-tauri/tests/ipc.rs:7-8, 25-30, 175, 321-329, 416, 603, 654, 719, 857, 908, 971, 1011, 1145, 1212, 1304, 1508-1509`
- `app/src-tauri/tests/periodic.rs:8-12, 189, 224`
- `app/src-tauri/tests/verification.rs:15-19, 219, 570, 708, 760`
- `app/src-tauri/tests/receiver_events.rs:688, 813-814, 842-843, 990, 1012, 1040, 1344, 1348-1351, 1464-1467, 1492`
- `app/src-tauri/tests/receiver_guard.rs:215-216`
- `app/src-tauri/tests/startup_policy.rs:200`
- `app/tests/api.admit.test.ts:10`, `app/tests/founding.decision.test.ts:4-6`, `app/tests/founding.propose.test.ts:3-4, 9-12`, `app/tests/notices.discarded.test.ts:156-157`, `app/tests/verification.view.test.ts:5-7, 139, 225-226, 242, 262-263`, `app/tests/revoke.validate.test.ts:4, 7-8`
- `app/vitest.config.ts:9` ("so the pure tests run as they did")
- `on-chain-client/tests/write_compose.rs:573-583`, `on-chain-client/tests/write_genesis_e2e.rs:85-92`

What: these comments record review rounds and finding numbers ("*Round 2 (TEST finding-3), 2026-10-28:*"), plan tasks ("T46", "T48"–"T54", "T60", "Amended (T49): thirteen…"), what was removed or renamed ("left with their tests", "Renamed from reconcile.view.test.ts", "no longer cites LLR-htp5tk", "the direct calls … are gone"), and the change itself ("change worktree-org-io-domain-api", "nothing in this change renders").
Why medium: some of these are already false, so they mislead in normal use. `ipc.rs:175` says the IPC surface does not reach "the write and the verification until T50/T51", but `write_organisation_to_chain` and `verify_organisation` are now registered and tested over IPC in the same file. `revoke.validate.test.ts:7-8` says "nothing in this change renders" a component, but the change adds rendered component tests (`vitest.config.ts`, `page.ts`). The rule's own reason applies: narration is true only at the commit that wrote it.
disposition: fixed by task `a5` (merged 2026-10-28) as C4-A1, by a sweep of all app and on-chain-client source and test files: comments naming rounds, findings, plan tasks, dates or removed code now describe the code as it is, and the false ones are corrected (`app/src-tauri/tests/ipc.rs` "until T50/T51", "nothing in this change renders", Cargo.toml's removed targets). The test name `a_refusal_this_change_added_does_not_stop_the_loop` is kept, because the decomposition and hazards docs name it.

**finding-2**: guideline, low — app+on-chain-client: Names 1, "No single-character names. Every name the change writes, a loop index and a callback parameter included, says what it holds." Locations:
- `app/src-tauri/tests/ipc.rs`: `h: &Harness` at 176, 187, 198, 209, 937 and the closures at 864, 1251, 1743; `let h = …` at 634, 660, 1013, 1062, 1091, 1125, 1192, 1249, 1380, 1409, 1429, 1439, 1452, 1463, 1474, 1487, 1497, 1676, 1701; `|e|` at 1310, 1400; `|((g, n, v), _)|` at 1537; `|c|` at 515
- `app/src-tauri/tests/periodic.rs:219, 237` (`|(_, e)|`)
- `app/src-tauri/tests/receiver_events.rs:696` (`let e`), `1160` (`OrgNodeError as E`), `1235` (`for (e, verdict)`)
- `app/src/lib/components/Invite.svelte:41, 59, 103, 120, 139` (`catch (e)` in the rewritten functions; `CreateOrg.svelte` and `PersonaList.svelte` were renamed to `refusal` in the same change)
- `app/tests/notices.admission.test.ts:153, 156, 157` (`a`, `b`, `(c)`), `app/tests/notices.discarded.test.ts:94, 113` (`(n)`)
- `on-chain-client/tests/write_compose.rs:239, 247, 464` (`i`), `244, 252` (`step: s`), `283, 284, 338, 356` (`|s|`)
- `on-chain-client/tests/fuzz_signatory_set/fuzz_target.rs:43` (`Err(e)`)

What and why: the change writes each of these names, and none says what it holds. The same change renamed `|e|` to `|error|` in `ceremony.rs` and `catch (e)` to `catch (refusal)` in `CreateOrg.svelte`, so the files under review follow the rule unevenly.
disposition: fixed by task `a5` (merged 2026-10-28) (low 2); the `|c|` closure in `app/src-tauri/tests/ipc.rs` may be left.

**finding-3**: guideline, low — app+on-chain-client: Names 3, "Name in concrete terms … no `data`, `info`, `item`, `value`, `result` … alone." Locations:
- `on-chain-client/src/write/proxy.rs:18` (parameter `value`) and `:22` (`let info`)
- `on-chain-client/src/write/subxt_ops.rs:127` (`let value`)
- `app/src-tauri/src/policy.rs:38, 58` (field `InvalidVerificationInterval { value }`) and `:76-80` (`let value`)
- `app/src-tauri/tests/verification.rs:823`: `adopted_field` is also called on `behind` entries (`:804`), so the name does not say what it reads.

What and why: `value` and `info` are names the clause forbids; each holds something concrete, such as the `System.Account` bytes, the `AccountInfo` record or the raw `ODS_VERIFICATION_INTERVAL_SECS` text. `adopted_field` names one result kind, but it reads a field of any entry.
disposition: fixed by task `a5` (merged 2026-10-28) (low 3), in `on-chain-client/src/write/proxy.rs` and the other cited files.

**finding-4**: guideline, low — app+on-chain-client: Names 4, "A concept has one name in every file, test and task of the change, and where the codebase already names it, that name." Location: `app/src/lib/api.ts:256` (`verifyOrganisations`, which invokes `verify_organisation`), against `api.ts:4` ("Command signatures mirror commands.rs exactly").
What and why: every other wrapper is the camelCase of its command (`proposeAdmission`, `issueInvite`, `writeOrganisationToChain`). This one adds a plural, so the TypeScript name and the command name read as two concepts, and the file header's "mirror … exactly" is not true.
disposition: not fixed — `verifyOrganisations` in `app/src/lib/api.ts` is named by the app DRAFT design and by LLR-j2x9jb (task `a5` (merged 2026-10-28), low 4).

**finding-5**: guideline, low — app+on-chain-client: Simplicity 2, "no interface, base class, factory or wrapper without a second caller." Locations:
- `app/src-tauri/src/periodic.rs:47-49`: `periodic_run` is a one-line wrapper over `commands::verification_outcomes(org_io, None)`. Its only caller is `run_periodic_verification` (`:62`), and `tests/periodic.rs:9-11` records that the tests stopped calling it.
- `app/src-tauri/src/commands.rs:29-32`: `pub use crate::entry::{attach_report, device_reports, entry_for, DeviceReportDto, DeviceResultDto, DiscardedDto, EntryOrg, EntryResultDto, VerificationEntryDto, WriteLabelDto}` re-exports `entry`'s items under a second path. The tests reach the same items both ways (`tests/verification.rs:26` through `commands::`, `:773` and `tests/receiver_events.rs:20` through `entry::`), and `DeviceResultDto`, `DiscardedDto`, `WriteLabelDto` and `attach_report` are used through `commands::` by nothing.

What and why: both add something to read and keep working with no caller that needs it. The re-export also gives one item two names (Names 4).
disposition: fixed by task `a5` (merged 2026-10-28) (low 5): `periodic_run` is gone from `app/src-tauri/src/periodic.rs` and the `entry` re-exports from commands.rs.

**finding-6**: guideline, low — app+on-chain-client: Simplicity 3, "Code no path reaches, an unused function, import or variable … are deleted." Location: `app/src/lib/components/Membership.svelte:85, 111`.
What and why: the `confirming` state stores `notice: DiscardedNotice`, set at `:111`, but nothing reads `confirming.notice` (the reads are `.confirmation`, at `:89` and `:335-340`, and `.again`, at `:130` and `:338`). The field is unused state.
disposition: fixed by task `a5` (merged 2026-10-28) (low 6): the unused `confirming.notice` field is gone from `app/src/lib/components/Membership.svelte`.

**finding-7**: guideline, low — app+on-chain-client: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it." This finding covers product code. Locations:
- `app/src-tauri/src/commands.rs:775-797` (`AdmissionRequestDto` and its `From<&KeptAdmissionRequestSummary>`) against `app/src-tauri/src/events.rs:63-72, 231-241` (`AdmissionRequested` and its build from `Received::AdmissionRequestKept`). These are the same seven fields, in the same order, with the same hex encodings. `commands.rs:772-774` states the rule itself: "the `admission-requested` event's fields and encodings (LLR-dp45mm)".
- `app/src-tauri/src/commands.rs:111-125` (`VerificationDto::of`) against `app/src-tauri/src/entry.rs:145-149` (`entry_for`). Both state the rule "`Refused`, `Store` and `Keychain` are `unverified` with org-io's text", and the settled case recurses in both.
- `app/src-tauri/src/commands.rs:230-238` (`persona_id_for`) against `:624-636` (`entry_org_for`). Each scans `org_io.personas()` for the Persona with a given `device_key`.
- `app/src/lib/components/Admit.svelte:78-88` against `app/src/lib/components/Membership.svelte:333-342`. These are the same confirmation markup, the same mismatch warning text and the same checkbox label, and the code states it as one rule ("the confirmation of REQ-88npkt, as for the first admission").

What and why: each pair is one rule written twice, and the next change to the admission-request payload, the unverified mapping or the confirmation text has to find both copies.
disposition: not fixed — task `a5` (merged 2026-10-28) left low 7, because merging duplicated product code (`app/src-tauri/src/commands.rs`, events.rs, entry.rs, Admit.svelte) is structural. Carried open, not blocking.

**finding-8**: guideline, low — app+on-chain-client: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it." This finding covers test code. Locations:
- `on-chain-client/tests/write_compose.rs:137-139` (`admin_of`) duplicates `on-chain-client/tests/support/write_reads_contract.rs:15-17` (`contract::admin_of`), which the same file includes at `:24-25`.
- `on-chain-client/tests/fuzz_signatory_set/fuzz_target.rs:34` re-states `ACCOUNT_INFO_BYTES` from `on-chain-client/src/write/proxy.rs:12`.
- `app/src-tauri/tests/verification.rs:286-289` (`BASE_POINT`) and `app/src-tauri/tests/receiver_events.rs:1044-1047` (`ARRIVAL_DEVICE_KEY`) are the same 32 bytes with the same doc line. `wire` is defined at `verification.rs:117` and again at `receiver_events.rs:53`.
- The founding-then-org-id helpers appear three times: `app/src-tauri/tests/ipc.rs:198-222` (`device_key_of`, `founded_org`), `verification.rs:88-100` (`founding`, `org_of`) and `periodic.rs:39-51` (`propose_founding`, `adopted_founding`). The kept-request poll appears at `ipc.rs:758-767` (`kept_requests`) and again inline at `verification.rs:168-175`.
- The creation-form fill at `app/tests/verification.view.test.ts:286-293` repeats `proposeFromTheForm` (`app/tests/founding.propose.test.ts:32-41`). The read-and-propose sequence at `app/tests/notices.admission.test.ts:184-196` and `:206-218` repeats `proposeMemberFromInvite` (`app/tests/invite.confirm.test.ts:29-45`). `app/tests/page.ts:18` inlines `settle()` (`:28-30`). The `import.meta.glob` source scan is written twice (`notices.discarded.test.ts:26-29`, `verification.view.test.ts:44-47`).

What and why: the change already has shared places for these (`support/`, `app/tests/page.ts`), and the copies come from separate tasks. That is the drift the clause names.
disposition: partly fixed by task `a5` (merged 2026-10-28) (low 8): `on-chain-client/tests/write_compose.rs` uses `contract::admin_of`, and openApp calls `settle()`. The rest is not fixed: there is no shared support module across test binaries, and `ACCOUNT_INFO_BYTES` is private.

**finding-9**: guideline, low — app+on-chain-client: Errors 1, "An error is reported where it is detected, naming what failed and on what input." Location: `on-chain-client/src/write/ceremony.rs:66-69`.
What and why: `found_at` refuses with "the proxy's address is mapped to another account". The message names neither the proxy, its H160 nor the account it is mapped to, although `proxy`, `admin` and `account` are all in scope. Someone reading the `MapAccount` failure cannot tell which proxy or mapping caused it.
disposition: not fixed — task `a5` (merged 2026-10-28) left low 9, because rewording the MapAccount refusal in `on-chain-client/src/write/ceremony.rs` changes behaviour and suits a TDD task. Carried open, not blocking.

**finding-10**: guideline, low — app+on-chain-client: Errors 2, "No swallowed error … no discarded return code or error value … Where an error is expected and handled, the handling says why that is safe." Locations:
- `app/src-tauri/tests/ipc.rs:1733`: `let _ = sender.send(…)` discards the send error with no reason given. The `start_receiver` sites in `commands.rs` and `lib.rs` each state theirs.
- `app/src-tauri/tests/verification.rs:139-145`: the background drain loop drops every `next_received` error except `Terminal` and does not say why that is safe.
- `on-chain-client/tests/write_compose.rs:214`: `let _ = found_at(…)` discards the result. The test comment (`:203-204`) says why the steps are checked, but not why the outcome is ignored.

What and why: a failure at any of these sites would vanish without a trace, which is what the clause guards against. Each needs either a check or a stated reason.

---

note: `app/src-tauri/Cargo.toml:23` ("# No rand: org-io holds the one random source …") explains a dependency that is absent, which reads close to change narration. It does state a standing constraint, which `ipc.rs:537-547` tests, so I did not flag it.
note: `app/src/lib/components/Admit.svelte:46` (`if (selected === null) return;`) and `Membership.svelte:125` (`if (confirming === null) return;`) cannot be reached after the gate passes. They are kept for type narrowing, and the guidelines do not say whether a narrowing guard counts as dead code.
note: `app/src-tauri/src/state.rs:69-73`: the doc of `verification_interval` says twice that `run`'s setup hands the interval to the periodic verification (SDD-tgu7dp). This is a redundancy, not a clause breach.
note: `app/src/lib/notices.ts:64-66`: `RequestMessage.severity` takes only `'error'`, and only a test reads it. `events::FollowUp` (`events.rs:269`) has a single variant. Both are close to Simplicity 2 ("no … flag … without a caller that uses it"), but each is read by its own code path, so I did not flag either.
note: `app/src/lib/api.ts:308` types `WriteDto.included.verification` as always present, while `commands.rs` declares it `Option` with `skip_serializing_if`. This is a contract accuracy question, not a CODE clause.
note: the DTO fields named `result` (`entry.rs` `VerificationEntryDto.result`, `DeviceReportDto.result`) are serde-flattened tags fixed by the wire contract, so I treated them as names a tool fixes (Names 1's exemption, applied by analogy to Names 3).
note: the compliance-floor items (`verifies:` annotations, red runs, abnormal-input coverage, SDD tracing) belong to the TEST and ARCHITECTURE reviews. No clause of this file contradicts the floor, and no clause refers to another guidelines file or a missing document.

disposition: fixed by task `a5` (merged 2026-10-28) (low 10), in `app/src-tauri/tests/ipc.rs`, verification.rs and `on-chain-client/tests/write_compose.rs`.

**finding-11**: guideline, medium — org-io: Simplicity 3, "an unused function ... [is] deleted" — org-io/src/test_support.rs:204
- What: `FakeChain::register_proxy` has no caller anywhere in org-io or app.
disposition: fixed by task `o4` (merged 2026-10-28) as C4-O1: `FakeChain::register_proxy` is deleted from `org-io/src/test_support.rs`.

**finding-12**: guideline, medium — org-io: Simplicity 3, "an unused function ... [is] deleted" / Simplicity 2, "no ... hook without a caller that uses it now"
- org-io/src/rng.rs:38-42: `impl Default for HandleRng` is never used. Every construction calls `HandleRng::os()` or `replaced_by`.
- org-io/src/lib.rs:343-347: `impl<E> From<E> for Cause<E>` has no caller. There is no `Cause::from`, no `.into()` into a Cause, and no `?` into a `Result<_, Cause<_>>`. Every site uses `Cause::new`.
disposition: fixed by task `o4` (merged 2026-10-28) as C4-O2: `impl Default for HandleRng` (`org-io/src/rng.rs`) and `From<E> for Cause<E>` are deleted.

**finding-13**: guideline, medium — org-io: Comments 2, "A comment describes the code as it is, never the change that wrote it: no ... ticket history" — org-io/.guardrails/config.yaml:97-98
- What: "Since T1 of docs/plans/... (2026-10-21), eight empty targets ... join it" and "Since T28 ... six targets leave ..." narrate the plan's history.
- Why it misleads: "eight empty targets" is now false, because those targets hold the tests.
disposition: fixed by task `o4` (merged 2026-10-28) as C4-O3: `org-io/.guardrails/config.yaml` and Cargo.toml describe the code as it is.

**finding-14**: guideline, low — org-io: Comments 2 (quoted above) — org-io/Cargo.toml:82
- What: "(TEST review round 1, finding-6)" is review-ticket history in a manifest comment.
disposition: fixed by task `o4` (merged 2026-10-28) (CODE low 4) in `org-io/Cargo.toml`.

**finding-15**: guideline, low — org-io: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays"
- view.rs:217-218 `CLOCK_BEFORE_EPOCH` and verify.rs:49 hold the same text twice. verify.rs:49 writes the literal "the clock reads before the Unix epoch" by hand.
- The const's doc, "`UnverifiedReason::ClockBeforeEpoch`'s text", is false because that Display never uses it.
- The `NotHeld` message "no Persona here holds Organisation {}" is added three times: exchange.rs:40, lib.rs:405 and send.rs:92.
- The closure that turns messages into `Outgoing` with `kept_for` is added twice: send.rs:211-216 and send.rs:297-302.
disposition: fixed by task `o4` (merged 2026-10-28) (CODE low 5): one copy each of the clock text, the `NotHeld` text and the `Outgoing` closure (`org-io/src/send.rs`, verify.rs, view.rs).

**finding-16**: guideline, low — org-io: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-io/src/verify.rs:280 and 319
- verify.rs:280: `record_epoch.unwrap_or(Epoch::new(0))` reports `InStep { epoch: 0 }` when the record is missing. The comment says this is unreachable, but the code still invents a value instead of failing.
- verify.rs:319: `held_record(...).map(|record| record.members).unwrap_or_default()` silently gives every revoked Member a `None` Handle. No comment gives a reason.
disposition: fixed by task `o4` (merged 2026-10-28) (CODE low 6) in `org-io/src/verify.rs`.

**finding-17**: guideline, low — org-io: Errors 2, "no discarded ... error value ... the handling says why that is safe" — org-io/src/send.rs:135
- What: `filter_map(|text| parse_kept_address(text))` silently drops every kept address that does not parse.
- Why: neither a Networked nor a Loopback send reports the dropped addresses, and no comment says why dropping them is safe.
disposition: fixed by task `o4` (merged 2026-10-28) (CODE low 7) in `org-io/src/send.rs`.

**finding-18**: guideline, low — org-io: Errors 1, "An error is reported where it is detected" — org-io/src/retention.rs:94 and 176
- What: `finish_leftovers` discards each `finish_removed` failure with `let _`. The hourly retry also ignores what `finish_leftovers` returns.
- Why: a leftover `removed-` file that cannot be finished is never reported, even though the module doc (retention.rs:8) says "a failed step is reported". The comment explains why it is retried, but not why it goes unreported.
disposition: fixed by task `o4` (merged 2026-10-28) (CODE low 8) in `org-io/src/retention.rs`.

**finding-19**: guideline, low — org-io: Names 1, "No single-character names. Every name the change writes ... says what it holds"
- Type parameter `E` on `Cause<E>` and its impls: lib.rs:330-369.
- Type parameter `T` on `bounded<T>`: write.rs:346.
- These are chosen names. See the note on type parameters.
disposition: fixed by task `o4` (merged 2026-10-28) (CODE low 9): the type parameter of `bounded` in `org-io/src/write.rs` is now `Written`.

**finding-20**: guideline, medium — org-io: Simplicity 3, "Code no path reaches, an unused function ... [is] deleted ... delete the implementation the change replaced" — tests/common/handles.rs
- Helpers left with no caller once the change deleted their callers:
  - `forwarding` at :150-160
  - `dead_addr` at :174-178
  - `spawn_recv_one` at :199-203
  - `spawn_recv_one_as` at :209-221
  - `deliver_from` at :223-226 (called only by `deliver`)
  - `deliver` at :229-231
- I confirmed that nothing calls them.
disposition: fixed by task `o4` (merged 2026-10-28) as C4-O10: the six dead helpers are deleted from `org-io/tests/common/handles.rs`.

**finding-21**: guideline, medium — org-io: Simplicity 3 (quoted above) — tests/node_stories.rs, the whole file, with its `[[test]]` entry in Cargo.toml
- What: the file holds no test, only a module doc and a lint allow, but it is still built and run as a target.
- Why it misleads: its header (lines 2-27) describes five stories that no longer exist.
disposition: fixed by task `o4` (merged 2026-10-28) as C4-O11: tests/node_stories.rs is deleted, with its target in `org-io/Cargo.toml`, its verify-line entry and its `Makefile` coverage entry.

**finding-22**: guideline, medium — org-io: Simplicity 2, "No parameter ... without a caller that uses it now"; Simplicity 3, "an unused ... variable"
- Parameters taken and then thrown away with `let _ =`, while every caller still passes them:
  - common/handles.rs:236-237: `founded`'s `chain`
  - common/handles.rs:256-257: `requested`'s `founder`
  - verification.rs:63-64: `remove_b`'s `founder`
- Statements that do nothing:
  - verification.rs:206: `let _ = joiner;`, after `joiner` was already used.
  - verification.rs:527: `let _ = joiner;`, where `joiner` is otherwise unused.
- store_files.rs:76-78: `create_async` returns a `Result` whose error case can never happen.
disposition: fixed by task `o4` (merged 2026-10-28) as C4-O12: the unused parameters, the `let _ = joiner` lines and `create_async` are removed (`org-io/tests/common/handles.rs` and the cited tests).

**finding-23**: guideline, medium — org-io: Simplicity 4, "one copy stays, where both can reach it" — helpers added more than once in this change
- "Propose a founding, write it in two steps, adopt it" is written five times:
  - common/handles.rs:236-251 `founded`
  - common/journey.rs:70-80 `founded`
  - endpoints.rs:125-133 `found`
  - expectations_met.rs:123-129 `founded_on`
  - invitation_exchange.rs:49-53, a third `founded`
- receive_step.rs:521-538 `keep_addresses` and its `take_everything` copy common/journey.rs:382-392. The same file uses both copies (:188 and :556), and they have already drifted apart (`vec![0x00]` against `Verdict::Accepted.as_byte()`).
- verification.rs:37-45 `deliver_epoch` reimplements `journey::deliver_epoch` (journey.rs:238) under the same name. verification.rs:619-626 `settle_deliver` calls the journey copy.
disposition: fixed by task `o4` (merged 2026-10-28) as C4-O13: one `handles::founded`, one `journey::keep_addresses` and one `journey::deliver_epoch` remain (`org-io/tests/common/journey.rs`).

**finding-24**: guideline, low — org-io: Simplicity 4 (quoted above) — smaller repeated helpers and checks
- `names_in`: person.rs:33-38, persona_creation.rs:40-45, rng_source.rs:77-82, and store_files.rs:107.
- `make_read_only`/`make_writable`: person.rs:44-56 against store_files.rs:114-126, plus inline copies at persona_creation.rs:101-109 and 353-361.
- The persona-id to endpoint lookup: `issuer_endpoint` at requests.rs:30-33, written out inline at send_fanout.rs:135-136, 170-171 and 208-209.
- open.rs:151 `dir_entry` against `handles::file_of`.
- `stranger_key` at journey.rs:148-150 is the same as `key` at chain_contract.rs:49-51.
- The verdict dispatch in `answering` (journey.rs:174-192) and `answering_acknowledgements` (journey.rs:429-455).
- The bind/forward/answer loop, written four times in acknowledgements.rs: :35-55, :247-265, :325-343, :1220-1240.
- `ack_files` at acknowledgements.rs:687-690 against `handles::file_of`.
- `persona_with` at expectations_met.rs:117-119 against `handles::persona_of`.
- `admitted` at endpoints.rs:139-154 against the journey steps.
- The source walks at acknowledgements.rs:120-145 and chain_read_rule.rs:59-77 against absences.rs `sources`/`code_lines`.
- The inline `ack_read_only` at acknowledgements.rs:1040-1047 and the inline `ack_report_gone` at acknowledgements.rs:1330-1335.
- `none_kept` at acknowledgement_inbound.rs:76-84 against `ack_until` at acknowledgements.rs:693-701.
- The `ack_removed_at` setup (acknowledgements.rs:428-435), written out inline at :526, 560, 595, 628, 662, 715, 747, 779, 806, 867, 909, 1133, 1258, 1314 and 1370.
- Tests that make the same check twice:
  - chain_write.rs:113-126 and :276-290
  - chain_write.rs:378-397 and :534-545
  - chain_write.rs:181-189 and :246-259
  - invitation_exchange.rs:89-92 and :238-247
  - receive_step.rs:70 and :366 (LLR-eeh4pq)
disposition: partly fixed by task `o4` (merged 2026-10-28) (CODE low 14): `names_in`, `make_read_only` and `make_writable` are shared. The rest would need a restructure and is not fixed, carried open, not blocking (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-25**: guideline, medium — org-io: Comments 2, "A comment describes the code as it is, never the change that wrote it"
- These comments are false or stale:
  - invitation_exchange.rs:29-30 says `founded` is "adopted by org-node directly", but it now goes through the handle.
  - common/handles.rs:8-11 says `founded` is "gone", but it is defined at :236.
  - expectations_met.rs:342-343 still describes `node_for_test`.
  - common/handles.rs:134-136 says `held_store` shows provisional changes, but its body (:137-145) does not.
  - receive_step.rs:625 copies the address comment from :554 onto code that uses no addresses.
- These file headers describe the old subject and give plan or review history:
  - node_stories.rs:2-27
  - send_fanout.rs:5-19
  - receive_step.rs:6-18
disposition: fixed by task `o4` (merged 2026-10-28) as C4-O15: every org-io/tests file was swept for stale or narrating comments (`org-io/tests/common/handles.rs` and the cited files). The sweep also dropped the dated "Mutation check" paragraphs, including keychain_contract's note of the owner's passing run against a real OS keychain, which merge-change step 5 must carry into the verification record.

**finding-26**: guideline, low — org-io: Comments 2, "no 'added', 'now uses', 'previously', 'fixed' or ticket history"
- Comments narrating plan tasks (T28, T37, T60, ...), review rounds, "Rewritten from <deleted file>:<line>", "Retargeted", "Tightened", "Moved", "Adapted/Restored".
- absences.rs: 272, 383, 461-462, 490, 493-497, 984-985, 1183-1186, 1223, 1262, 1285.
- acknowledgement_inbound.rs: 12-24, 49-51, 86-87, 111, 133, 177, 188, 243, 290.
- acknowledgements.rs: 6-7, 75-76, 190, 380, 437, 475, 513-516, 677, 733, 1159, 1168-1170, 1296.
- app_boundary.rs: 217-218, 225, 229-230.
- chain_write.rs: 6, 43, 86-87, 128, 163, 192, 215-216, 262-263, 326-327, 370, 562, 575, 613, 616-618.
- chain_writer_contract.rs: 2, 9, 13, 126.
- common/chain_contract.rs: 1-2.
- common/chain_writer_contract.rs: 1-2, 37-39.
- common/handles.rs: 13-17, 235.
- common/journey.rs: 1-3, 8-13, 161, 224, 330.
- common/mod.rs: 10, 12, 15-16.
- endpoints.rs: 110-119.
- expectations_met.rs: 30-45, 131-132, 147-148, 220-221, 280-281, 341, 356, 389-390, 447, 475.
- invitation_exchange.rs: 6-7, 25-28, 48, 56, 89, 167, 179.
- keychain.rs: 212-213.
- store_files.rs: 21-27, 226-229, 249-251, 264-265, 397-398.
- rng_source.rs: 6, 89-92, 154, 233-234, 255.
- signatory_key.rs: 85-88, 149-151.
- preflight_transport.rs: 81-82.
- persona_creation.rs: 6, 47, 126, 135, 166, 203, 260-261, 293.
- person.rs: 6, 219.
- propose.rs: 7, 114, 142.
- requests.rs: 6-7, 12, 195.
- receive_step.rs: 66-67, 174, 184, 361, 381, 395, 412, 428, 517, 554, 587.
- send_fanout.rs: 130, 264-266, 269, 320, 339.
- verification.rs: 6, 36, 103, 164, 199, 203, 231, 286, 367, 389, 402, 520, 528-530, 571, 611, 618, 690, 783, 798, 813, 825, 855, 881.
disposition: fixed by task `o4` (merged 2026-10-28) as low C4-O16, by the same sweep of every org-io/tests file (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-27**: guideline, low — org-io: Names 1, "No single-character names"
- acknowledgements.rs:
  - `t` at 200, 396, 446, 484, 520, 555, 590, 623, 657, 710, 742, 774, 801, 862, 904, 948, 989, 1023, 1080, 1128, 1175, 1252, 1309, 1365, 1422, 1468.
  - `_a` at 111, 161, 778, 1132, 1179, 1257, 1313.
  - `_c` at 281 and `_n` at 282.
  - `_r` at 450, 488, 525, 559, 594, 627, 661, 714, 746, 805, 866, 908, 1369.
- chain_write.rs:
  - `a` at 27, 35, 44, 57, 78, 117, 137, 157, 176, 200, 226, 251, 269, 281, 297, 314, 329, 346, 358, 364, 371, 383, 406, 421, 440, 460, 479, 511, 539, 552.
  - `b` at 79, 181, 238.
  - `c` at 288.
- expectations_met.rs: `a` at 353, `b` at 361 and 381.
- invitation_exchange.rs: `a` at 69, 79, 100, 121, 136, 159, 189, 206, 230, 243, 256, 281, 295; `b` at 70 and 138.
- common/chain_writer_contract.rs:45: `|c: char|`.
- `F` at common/chain_contract.rs:60 and common/journey.rs:273.
- open.rs:118: `t`.
- person.rs:134-135: `a`, `b`.
- propose.rs: `a` at 42, 70, 94, 112, 152, 170.
- verification.rs:
  - Parameters `a`, `b` at 37 and 63.
  - Locals `a`/`b` at 51, 78, 110, 129, 155, 185, 188, 191, 210, 213, 229, 277, 302, 323, 343, 376, 397, 422, 445, 485, 501, 504, 545, 580.
  - `c` at 516.
disposition: not fixed — task `o4` (merged 2026-10-28) left CODE low 17: the one-letter test locals each need their own edit, because "a"/"b" also appear in strings. Carried open, not blocking (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-28**: guideline, low — org-io: Names 3, "no `data`, `info`, `item`, `value`, `result`, `manager`, `handle` or `process` alone"
- `data`: chain_write.rs:372, persona_creation.rs:128 and 167.
- `item`: absences.rs:643, which holds an impl line.
- `value`: common/journey.rs:281.
- `result`: propose.rs:156.
- `handle`, holding the chopsticks fork rather than an OrgIo: chain_writer_contract.rs:133 and 135.
disposition: fixed by task `o4` (merged 2026-10-28) (CODE low 18), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-29**: guideline, low — org-io: Names 4, "One name per concept across the change ... where the codebase already names it, that name"
- `founded` names three different helpers (handles.rs:236, journey.rs:70, invitation_exchange.rs:49), and the same step is also called `found` and `founded_on`.
- Stores reopened from disk have three names: `on_disk`, which only wraps `reopened_from` (journey.rs:59-61), `reopened_from` and `reopened_service`.
- Persona ids are called `pid`/`bob_pid` at requests.rs:31, 64 and 152 and at send_fanout.rs:135, 170 and 208, but `persona_id` everywhere else, including the API.
disposition: fixed by task `o4` (merged 2026-10-28) (CODE low 19), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-30**: guideline, low — org-io: Names 2, "one action per statement ... no trick that saves a line"
- acknowledgements.rs:1277: `.as_ref()?` used as a hidden early return.
- invitation_exchange.rs:284: `["outstanding", "_invites.json"].concat()`, with no reason given.
- Tuple assignments that each do two unrelated things: chain_write.rs:58, acknowledgements.rs:208, expectations_met.rs:382, invitation_exchange.rs:139 and 210.
disposition: partly fixed by task `o4` (merged 2026-10-28) (CODE low 20), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`; the remainder is carried open, not blocking.

**finding-31**: guideline, low — org-io: Errors 2, "no discarded return code or error value ... Where an error is expected and handled, the handling says why that is safe"
- `let _ = forward.send(...)`: acknowledgements.rs:46, 258, 337 and 1232; common/journey.rs:182, 417 and 445.
- `if let Ok` silently dropping a failed receive: journey.rs:416.
- `let _ = std::fs::remove_dir_all(&dir)`: open.rs:109 and 168, person.rs:29, verification.rs:246, acknowledgements.rs:951, 992 and 1471, chain_writer_contract.rs:113.
- `let _ = mine_block(...)`: chain_writer_contract.rs:91.
- `let _ = self.miner.await`, which loses a JoinError: chain_writer_contract.rs:102.
- receive_step.rs:525-531: the local `take_everything` throws away every result other than Terminal. This goes away with finding-13.
- person.rs:169: `let Ok(text) = read_to_string(..) else { continue }`. If a scanned file is renamed, the scan passes without reading anything (also Errors 3).
disposition: fixed by task `o4` (merged 2026-10-28) (CODE low 21), recorded in `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-32**: guideline, medium — org-node: Comments 2, "A comment describes the code as it is, never the change that wrote it: no "added", "now uses", "previously", "fixed" or ticket history" — org-node/tests/** (about 107 added comment lines) and org-node/.guardrails/config.yaml:276-279.
The added test comments record history, not the code as it is. Examples:
- tests/absences.rs:64, 126, 173, 354, 396, 595-603, 616-621, 662-667, 675, 722, 744-748, 780
- tests/admission_sender.rs:8-10, 36-42, 405, 419, 514
- tests/encoding_golden.rs:125-128, 142, 276, 318
- tests/invitation.rs:3, 9, 77, 98, 190, 216
- tests/organisation_key.rs:11, 115, 180 ("Abnormal (added 2026-10-25, TEST review round 1 ..."), 184 ("was `Trie(DuplicateKey)` until 2026-10-28, round 2")
- tests/organisation_request.rs:305
- tests/own_removal.rs:282, 303, 563, 791-792
- tests/persona_records.rs:12-15, 123, 148, 442
- tests/proposals.rs:176, 187, 284, 300-302, 332, 350, 367, 384, 515, 565, 585, 605, 1128, 1378
- tests/provisional_store.rs:8-9, 91, 106
- tests/receive_chain_reads.rs:4-9, 33-36, 75-79
- tests/receive_path.rs:9-11, 171, 340-343, 357-359, 378-380, 424, 449-453, 629, 647
- tests/revocation.rs:5
- tests/sealed_store.rs:9-11, 53, 295-296, 337, 348, 375-377, 427, 579, 652-653, 684
- tests/secret_redaction.rs:9, 307, 467, 508
- tests/service_lifecycle.rs:18-19, 84, 199-200
- tests/service_stories.rs:5
- tests/store_at_rest.rs:15-17, 47, 200, 236
- tests/support/mod.rs:8-12, 214
- tests/value_types.rs:56, 82, 189
- tests/wire_frame_bound.rs:10-11, 215
- tests/fuzz_invitation_parse/fuzz_target.rs:2-4, 14
- tests/fuzz_wire_decode/fuzz_target.rs:4-5, 20

These comments say "*Rewritten 2026-10-21 (T8):* was ...", "*Stripped ...*", "*Re-pointed ...*", "Mutation check, 2026-10-25: ... failed this test, and was removed", and review-round and finding numbers. The config.yaml lines are the same kind ("2026-10-21, T1 of docs/plans/...: six empty targets registered ..."). That one is already stale, because the targets are no longer empty.
Why it matters: the clause says this history goes in the commit message and the change's records. In the tree it is true only at the commit that wrote it.
disposition: fixed by task `n4` (merged 2026-10-28) as C4-N1: all of org-node/tests and `org-node/.guardrails/config.yaml` were swept for change narration, and the config's keys and values are byte-identical.

**finding-33**: guideline, medium — org-node: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it" — test helpers.
- `trie_of(members: &[MemberSnapshot]) -> Trie` has the same body in organisation_request.rs:47, own_removal.rs:63, proposals.rs:681, receive_path.rs:718, sealed_store.rs:466, store_at_rest.rs:312 and verification.rs:89. The change also adds `support::trie_of_members` (tests/support/mod.rs:718-725), which does the same thing and which every one of these files can reach.
- `h(text: &str) -> Handle { Handle::parse(text).unwrap() }` is defined five times: invitation.rs:272, organisation_request.rs:26, own_removal.rs:37, proposals.rs:658, sealed_store.rs:453.
- Two random-source doubles are each added twice. `CountingRng` (counts drawn bytes, delegates to OsRng) is at proposals.rs:80 and person_store.rs:15. The constant-byte source is `ConstRng` at proposals.rs:59 and `OneByte` at organisation_key.rs:160.
Why it matters: these copies state one rule. The next change will fix only one of them.
disposition: fixed by task `n4` (merged 2026-10-28) as C4-N2: the test helpers have one copy each, support's `trie_of_members`, `handle`, `ConstRng` and `CountingRng` (`org-node/tests/support/mod.rs`).

**finding-34**: guideline, medium — org-node: Names 1, "Every name the change writes, a loop index and a callback parameter included, says what it holds" — single-character names in added test lines (none in src/).
- Bindings and parameters:
  - `s`: receive_path.rs:56, 58, 66, 106, 125, 144, 238, 263, 284, 317, 347, 387, 460, 472, 589, 635, 654; provisional_store.rs:50; receive_chain_reads.rs:43, 86; admission_sender.rs:49; support/mod.rs:733, 764; verification.rs:84
  - `a`: admission_sender.rs:115, 169; organisation_key.rs:203, 233; verification.rs:1102; service_lifecycle.rs:103
  - `b`: admission_sender.rs:292, 439, 492, 527; receive_chain_reads.rs:134, 162, 190; service_stories.rs:42
  - `m`, `p`, `d`: encoding_golden.rs:205, 259; persona_records.rs:134, 400, 405; proposals.rs:345; support/mod.rs:537
  - `n`, `k`: own_removal.rs:535, 576
  - loop variables `m`, `d`: organisation_key.rs:212, 214
- Function name `h`: invitation.rs:272, organisation_request.rs:26, own_removal.rs:37, proposals.rs:658, sealed_store.rs:453.
- Closure parameters `|m|`, `|s|`, `|i|`, `|k|`, `|a|`, `|a, b|`, `|c|`, `|w|`, `|l|`, `|e|`, `|h|`, `|r|`, `|p|`, `|b|`: absences.rs:139, 412, 507; admission_sender.rs:219, 265, 267, 327, 336, 359, 361, 365, 369, 393; epoch_store.rs:264; invitation.rs:309; organisation_request.rs:50; outgoing_bodies.rs:73; own_removal.rs:41, 66, 153, 488, 514-515, 536, 542, 577, 591; persona_records.rs:469-477, 515-524, 542, 564-569; proposals.rs:125, 684, 805; receive_chain_reads.rs:199; receive_path.rs:721, 819; revocation.rs:627; sealed_store.rs:469, 676, 679; secret_redaction.rs:389, 449-450; service_lifecycle.rs:49, 69, 139; store_at_rest.rs:315, 326; support/mod.rs:381, 721, 767; support/sweep.rs:183, 187, 240; verification.rs:97, 885, 1131, 1185-1193.
disposition: fixed by task `n4` (merged 2026-10-28) as C4-N3: no single-character names are left in org-node's tests (`org-node/tests/support/mod.rs` and the cited files).

**finding-35**: guideline, low — org-node: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-node/tests/absences.rs:551, 581, 634, 727.
`enclosing_function(...).unwrap_or_default()` and `caller.split("::").nth(1).unwrap_or_default()` turn "no enclosing function found" into `""`. In `a_kept_acknowledgement_is_removed_only_on_acceptance` and the removed-form builder sweep, an occurrence the scan cannot place then goes to `public_entry_points_reaching("")`. That call finds no entry point, so the removing call drops out of the asserted list without a failure. This is low because it needs a removing call outside any `fn` body.
disposition: fixed by task `n4` (merged 2026-10-28) (CODE low 4) in `org-node/tests/absences.rs`.

**finding-36**: guideline, low — org-node: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-node/src/service.rs:270, 539, 1092-1094, 1172.
- :270: a fourth or later preferred Handle that fails to parse falls back to the field name `member.preferred_handles`. The doc at :243 promises `member.preferred_handles[i]`.
- :539: `base_root.unwrap_or(root)` stores the answer's own root as a stand-in base. The comment gives a reason, but no requirement names that default.
- :1092-1094: if exactly one of `kept_change_set` / `kept_base_snapshot` is `Some`, the result is a silent empty list. The doc names only "no Change set".
- :1172: `.map_or("", ...)` returns an empty invitee name. The doc says this is "never returned in practice" rather than making it an error.
Each needs a state that adoption and keeping do not produce, so the severity is low.
disposition: not fixed — task `n4` (merged 2026-10-28) left CODE low 5, because each fallback in `org-node/src/service.rs` would change behaviour without an LLR. Carried open, not blocking.

**finding-37**: guideline, low — org-node: Errors 1, "An error is reported where it is detected, naming what failed and on what input" — some errors name the wrong failure.
- org-node/src/revocation.rs:156: a Persona with no MemberId is reported as `OrgNotHeld { org_id }`.
- org-node/src/service.rs:1356: `remove_own` on a store with no record is reported as `OrgNotOnChain`.
- org-node/src/store.rs:1284: an unknown store-form byte is refused as `InvalidField { field: "store", reason: "unknown store form" }` without the byte read.
- org-node/src/error.rs:193 (`NoFreeHandle`), 211 (`NoProvisionalFounding`), 216 (`FoundingProxyAlreadyRecorded`) and 225 (`NoProvisionalMember`) name no input. Their siblings do: `NoAdmissionRequestKept { invite_id }` and `AdmissionAlreadyRecorded { invite_id }` name the invite.
The first three need contrived states, so the severity is low.
disposition: not fixed — task `n4` (merged 2026-10-28) left CODE low 6 (with AR4-6), because renaming or adding `OrgNodeError` variants (`org-node/src/error.rs`) changes a public enum the app matches exhaustively. Carried open, not blocking.

**finding-38**: guideline, low — org-node: Errors 2, "No empty catch block, no discarded return code or error value" — org-node/src/removal.rs:34.
The test-build `PartialEq for StoreContent` compares `postcard::to_allocvec(self).ok() == postcard::to_allocvec(other).ok()`. Two contents that both fail to encode compare equal (`None == None`), and the encode error is discarded with no stated reason.
disposition: fixed by task `n4` (merged 2026-10-28) (CODE low 7) in `org-node/src/removal.rs`.

**finding-39**: guideline, low — org-node: Names 4, "A concept has one name in every file, test and task of the change" — org-node/src/service.rs:979-980; tests/support/mod.rs:217 vs tests/support/sweep.rs:75; tests/proposals.rs:59 vs tests/organisation_key.rs:160.
- In `verify`, `matched` first names the index from `Judgement::Adopt(matched)`. It is then rebound at :980 to the `ProvisionalOrganisation`. At :992 the same name is the index again.
- The test node that keeps org-io's writes is `Node` in support/mod.rs and `Side` in support/sweep.rs. Each has its own `keep`, `adopt`, `reopened` and `propose_member`.
- The constant-byte random source is `ConstRng` in one file and `OneByte` in another.
disposition: partly fixed by task `n4` (merged 2026-10-28) (CODE low 8): `matched` and `ConstRng`/`OneByte` are fixed. Merging `Node` and `Side` (`org-node/tests/support/sweep.rs`) would restructure every sweep test, so that part is not fixed, carried open, not blocking.

**finding-40**: guideline, low — org-node: Names 3, "A name says what the thing is in the project's domain, not its type or a role in general: no `data`, `info`, `item`, `value`, `result` ..." — org-node/src/store.rs:43; tests/provisional_store.rs:61; tests/support/mod.rs:277-287.
- store.rs:43 `parse_each` names its element `value`.
- provisional_store.rs:61 `fn keep(...)` runs `check_received` and returns its outcome. It keeps nothing itself.
- support/mod.rs:277 `create_persona` now calls `propose_founding`, the codebase's name for that operation. Its name no longer says what it does.
disposition: fixed by task `n4` (merged 2026-10-28) (CODE low 9) in `org-node/src/store.rs`, provisional_store.rs and `org-node/tests/support/mod.rs`.

**finding-41**: guideline, low — org-node: Names 2, "Write code to be read ... no trick that saves a line" — org-node/src/verify_org.rs:319-320 (and :316).
`let included = !adopted...any(...)` followed by `(!included).then_some(...)` uses a double negation to say "the adopted record still holds the member". A plain `if` on the positive condition reads directly.
disposition: fixed by task `n4` (merged 2026-10-28) (CODE low 10) in `org-node/src/verify_org.rs`.

**finding-42**: guideline, low — org-node: Simplicity 2, "no interface, base class, factory or wrapper without a second caller" — org-node/tests/support/sweep.rs:105-108.
`Side::reopened_store` only calls `clone_store`. It is a wrapper with no behaviour of its own.
disposition: fixed by task `n4` (merged 2026-10-28) (CODE low 11) in `org-node/tests/support/sweep.rs`.

### Round 5

**finding-1**: guideline, low — app+on-chain-client: Names 1, "Every name the change writes, a loop index and a callback parameter included, says what it holds". These are new single-character names:
- app/src-tauri/tests/ipc.rs:738: `fn kept_requests(inviter: &Harness, n: usize)`. `n` is the count of kept requests to wait for.
- Type parameters the change chose. A language or tool does not fix these names:
  - app/src-tauri/src/periodic.rs:46 `run_periodic_verification<E>`. Its neighbour `periodic_runs` in the same file names the same role `Emit`.
  - app/src-tauri/tests/ipc.rs:173 `block_on<F: Future>`.
  - on-chain-client/src/write/ceremony.rs:33 and :54 `create_proxy<O: WriteOps>` and `found_at<O: WriteOps>`.
  - on-chain-client/tests/support/write_reads_contract.rs:21, :28 and :36 `<O: WriteOps>`.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `app/src-tauri/tests/ipc.rs`, with the type parameters in `app/src-tauri/src/periodic.rs` and `on-chain-client/src/write/ceremony.rs`.

**finding-2**: guideline, low — app+on-chain-client: Names 3, "A name says what the thing is in the project's domain, not its type or a role in general". app/src-tauri/tests/ipc.rs:1302 `const SEVENTEEN: [&str; 17]` names a count, not what it holds (the registered command names). It becomes false when the surface gains or loses a command.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `app/src-tauri/tests/ipc.rs`, where `SEVENTEEN` is still the name of the registered command list.

**finding-3**: guideline, low — app+on-chain-client: Names 3, "A name says what the thing is in the project's domain". app/src/lib/components/Invite.svelte:68 `let read = $state<InviteDto | null>` is a verb that holds the Invite the user read. Its siblings in the same block (`readBusy`, `readErr`) make `read` read as a flag or an action.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `app/src/lib/components/Invite.svelte`, which still binds `let read`.

**finding-4**: guideline, low — app+on-chain-client: Names 4, "A concept has one name in every file, test and task of the change". app/src/lib/api.ts:256 `verifyOrganisations()` is the only wrapper not named for its command (`verify_organisation`). Every other wrapper mirrors its command, and the module header at api.ts:4 says "Command signatures mirror commands.rs exactly". The wrapper also drops the command's `orgId` argument.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `app/src/lib/api.ts`, whose wrapper is still `verifyOrganisations()`.

**finding-5**: guideline, low — app+on-chain-client: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it". Names 4 also applies ("not `account_id` in one file and `customer_id` in another for the same value"). Two parts of the change add the same Rust test helpers:
- The ed25519 base point appears as two constants under two names, each with the same doc line ("The ed25519 base point's canonical encoding: a valid Device public key."): app/src-tauri/tests/receiver_events.rs:1014 `ARRIVAL_DEVICE_KEY` and app/src-tauri/tests/verification.rs:284 `BASE_POINT`.
- `wire` is defined twice: receiver_events.rs:53 and verification.rs:112.
- One shared test module, as on-chain-client's `tests/support/` does with `#[path]`, could hold one copy of each.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `app/src-tauri/tests/receiver_events.rs` and `app/src-tauri/tests/verification.rs`.

**finding-6**: guideline, low — app+on-chain-client: Simplicity 4, "one copy stays, where both can reach it", and Names 4, "A concept has one name". The webview tests repeat the same helpers. app/tests/page.ts is the shared home that both files of each pair already import.
- The same `AdmissionRequestDto` builder is written twice under two names: app/tests/notices.admission.test.ts:31 `request()` and app/tests/notices.discarded.test.ts:45 `keptRequest()`. The bodies are identical.
- The invitee's read, propose and request flow is `proposeMemberFromInvite` in app/tests/invite.confirm.test.ts:29, and is written out again inline in notices.admission.test.ts (around :186-198 and :208-220).
- The creation-form fill is `proposeFromTheForm` in app/tests/founding.propose.test.ts:30-39, and is written out again in app/tests/verification.view.test.ts:281-285.
- The `sources` glob is defined at notices.discarded.test.ts:26 and again at verification.view.test.ts:41.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is the webview tests beside `app/tests/page.ts`.

**finding-7**: guideline, low — app+on-chain-client: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays". The admission confirmation's two texts appear in two components:
- The mismatch warning "The name in the request is not the name you typed for this Invite." at app/src/lib/components/Admit.svelte:87 and app/src/lib/components/Membership.svelte:340.
- The confirmation label "This request is from the person I invited" at Admit.svelte:95 and Membership.svelte:344.

They state one rule, the REQ-88npkt confirmation, which both files already import from `$lib/notices`. That module is where one copy could live.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `app/src/lib/components/Admit.svelte` and `app/src/lib/components/Membership.svelte`.

**finding-8**: guideline, low — app+on-chain-client: Simplicity 2, "no interface, base class, factory or wrapper without a second caller". app/src-tauri/src/commands.rs:694 `verify_all_and_send` wraps `verified_entries` and drops each entry's org id. Its only caller is commands.rs:710 (`verify_organisation`). No test calls it.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `app/src-tauri/src/commands.rs`, where `verify_all_and_send` still has `verify_organisation` as its only caller.

**finding-9**: guideline, low — app+on-chain-client: Simplicity 2, "No parameter, flag, setting or hook without a caller that uses it now". app/src/lib/notices.ts:65 and :73: `RequestMessage.severity` is always `'error'`. No source reads it: Invite.svelte:201 renders only `.text`. Only the test notices.admission.test.ts:170 reads it.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `app/src/lib/notices.ts`, whose `severity` is still always `'error'`.

**finding-10**: guideline, low — app+on-chain-client: Simplicity 3, "Code no path reaches ... and commented-out code are deleted, not kept for later". on-chain-client/src/verify.rs:1-5 was rewritten in this change to say it is "Reserved for a verifier ... It holds no code". The module, still declared `pub mod verify;` in lib.rs:28, is kept for later use.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `on-chain-client/src/verify.rs`, still declared and holding no code.

**finding-11**: guideline, low — app+on-chain-client: Comments 1, "the code already says what it does, and a comment that repeats it drifts from it at the next edit". Comments that describe the call graph have already drifted:
- app/src-tauri/src/commands.rs:684-685 says `verify_and_send` is "The body every verification runs: `verify_organisation`'s, the receiver loop's, the write's and the periodic run's". The receiver loop (`verify_received`) and the periodic run reach `verified_entry` or `verified_entries` through `verification_outcomes`, never `verify_and_send`.
- app/src-tauri/src/periodic.rs:4-5 ("`commands::verification_outcomes`, over `verify_and_send`") and periodic.rs:54 ("verified through `verify_and_send`") repeat the same error.
- commands.rs:691-693 says `verify_all_and_send` returns "each outcome beside the target it verified", but it returns the entries alone and discards the target.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `app/src-tauri/src/commands.rs` and `app/src-tauri/src/periodic.rs`.

**finding-12**: guideline, low — app+on-chain-client: Errors 1, "An error is reported where it is detected ... and the code stops or returns an error its caller must handle". These handlers return silently:
- app/src/lib/components/Admit.svelte:46 `if (selected === null) return;` runs after `gate.ok` was checked.
- app/src/lib/components/Membership.svelte:125 `if (confirming === null) return;` does the same.

Each is unreachable in normal use (the gate refuses a null request, and the button renders only while `confirming` is set), so the severity is low. Each would still swallow the state without a word, and neither says why that is safe.

note: app/src-tauri/src/state.rs:70-73. The doc of `verification_interval` says twice that `run`'s setup hands the interval to the periodic verification. That is a repetition, not a clause breach.

note: Two commands discard org-io's `ProvisionalChangeSummary` through `.map(|_| ())` without saying why: `propose_admission` (commands.rs, `.map(|_| ())` before `map_err`) and `propose_revocation`. Errors 2 names error values and return codes, and this is a success value, so it is not a finding.

note: Each of these is a 1:1 mapping one could fold:
- `events::FollowUp` is a one-variant enum where `Option<String>` would do.
- `events::Arrival` maps 1:1 onto `ReceiverOutcome`.
- `verify_received` re-parses an org id that `events` wrote as hex from an `OrgId`.

Both types have tests of their own, so Simplicity 2's "second caller" bar is arguably met.

note: These tests restate a production constant as an independent oracle rather than importing it. I read them as a test oracle stating the rule independently, not as duplication:
- tests/fuzz_signatory_set/fuzz_target.rs restates `ACCOUNT_INFO_BYTES`.
- app/src-tauri/tests/ipc.rs restates `EXACTLY_ONE_TARGET`, which is pub as `commands::EXACTLY_ONE_TARGET`.

note: These are outside the changed lines, so I did not review them:
- Revoke.svelte's unchanged context line "Revocation submitted. Waiting for the remote peer to receive + self-delete…" no longer matches the command, which only proposes a revocation.
- Unchanged context lines keep single-letter names (`catch (e)`, `(p: ...)`, `|e|`, `classify_receive_error(e: ...)`).

note: `ex` (ipc.rs, the `Exchange` fixture) is an abbreviation, not a single character, so Names 1 does not reach it.

Clause checks: no clause contradicts the floor. No clause refers to another guidelines file or to a missing document. Each clause's reason still fits its rule.

disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 5 section closes this review's loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `app/src/lib/components/Admit.svelte` and `app/src/lib/components/Membership.svelte`. The notes under this finding raise no clause breach and called for no change.

**finding-13**: guideline, medium — org-io: Errors 2, "Where an error is expected and handled, the handling says why that is safe" — org-io/src/send.rs:115-133
`kept_for` (131-132) drops `addresses_for`'s refusal with `.ok().flatten()`, and it has no comment of its own. The comment that explains the drop ("a refusal never reaches `None` here: `None` is 'no address kept'", 115-119) sits on top of `outgoing_with_kept` (121), merged with that function's own one-line doc (120). The function that handles the error does not say why, and rustdoc shows `kept_for`'s contract on `outgoing_with_kept`, which misleads a reader. Move lines 115-119 down onto `kept_for`.
disposition: fixed in task `o5` (C5-O1): the comment saying why dropping `addresses_for`'s refusal is safe moved off `outgoing_with_kept` onto `kept_for` itself in `org-io/src/send.rs`, so rustdoc shows each function its own contract (`docs/plans/2026-10-21-org-io-domain-api.md`, round 5 fix — TEST and CODE org-io).

**finding-14**: guideline, low — org-io: Names 1, "Every name the change writes, a loop index and a callback parameter included, says what it holds" — org-io/tests/acknowledgements.rs:200 (and 25 more `let t =`), :111, :161 (`_a`, `_r`, `_c`, `_n`)
The added tests use one-letter names in many places:
- `t` for the verification time in acknowledgements.rs (26 times) and open.rs:99.
- `a`, `b`, `c` for handles in chain_write.rs:29, :80-81 and onwards; verification.rs:32, :45, :57, :71, :525; propose.rs:41, :69, :93, :111, :151, :169; invitation_exchange.rs:36, :55-56, :65 and onwards; person.rs:114-115; expectations_met.rs:318, :326, :346.
- `|c: char|` in tests/common/chain_writer_contract.rs:44.
- Held endpoints bound to `_a`, `_r`, `_c`, `_n`, for example acknowledgements.rs:111 and :161.

Names the language fixes are exempt. None of these is.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; the handles `a`/`b`/`c`, `t` and `_a`/`_r` are renamed, for example in `org-io/tests/acknowledgements.rs`.

**finding-15**: guideline, low — org-io: Names 3, "A name says what the thing is in the project's domain, not its type or a role in general" — org-io/src/acknowledge.rs:302
These names send the reader into the code to learn what they hold:
- `Ok(still)` in acknowledge.rs:302 is the count of acknowledgements still kept.
- `own` in send.rs:156, :269, :297 is the sender's own addresses or sockets.
- `at` in tests/acknowledgements.rs:210, :364 and tests/receive_step.rs:580 is the addresses an endpoint holds.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; the names `still`, `own` and `at` are renamed to what they hold, in `org-io/src/acknowledge.rs`, `org-io/src/send.rs` and the tests.

**finding-16**: guideline, low — org-io: Names 4, "A concept has one name in every file, test and task of the change, and where the codebase already names it, that name" — org-io/src/send.rs:263, :292
The codebase names a DevicePublicKey `device_key`, but parts of this change name it `device`: `let device` in send.rs:263 and :292, and the `device` field of `Admin` and `Joining` in tests/common/journey.rs:38 and :47. Three lookups also use three names:
- tests/endpoints.rs:30 `device_of` and tests/store_files.rs:75 `device_key_of` do the same Persona-to-key lookup.
- tests/expectations_met.rs:97 `persona_with` does the reverse lookup that tests/common/handles.rs:99 names `persona_of`.

Also, tests/acknowledgement_inbound.rs:31 adds `LIMIT` (30 s, "every wait on the network"), which is the concept that tests/common/handles.rs:28 already names `NET`.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; the Device key is `device_key` and the lookups go through `common::handles` (`persona_of`, `device_key_of`) in `org-io/tests/common/handles.rs`.

**finding-17**: guideline, low — org-io: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it" — org-io/tests/acknowledgements.rs:1114
The change adds the read-only toggle five times: once in `common::handles` (`make_read_only` and `make_writable`, tests/common/handles.rs:112-124) and four more times outside it:
- `ack_read_only` in acknowledgements.rs:1114.
- An inline copy in acknowledgements.rs:1039-1048.
- Inline copies in persona_creation.rs:94-102 and :347-355. That file already imports `names_in` from the same module.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; the read-only toggle is the one `make_read_only`/`make_writable` pair in `org-io/tests/common/handles.rs`.

**finding-18**: guideline, low — org-io: Simplicity 4 (as finding-5) — org-io/tests/common/journey.rs:132
`stranger_key(seed)` and tests/common/chain_contract.rs:50 `key(seed)` were both added in `tests/common`, and both are `OrgPrivateKey::from([seed; 32]).x25519_keypair().org_public_key().unwrap()`.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; one `stranger_key` in `org-io/tests/common/journey.rs` stays, and `org-io/tests/common/chain_contract.rs` imports it.

**finding-19**: guideline, low — org-io: Simplicity 4 (as finding-5) — org-io/tests/receive_step.rs:154
`dialled(to, device, body)` is line for line the same as tests/acknowledgement_inbound.rs:57 `ack_dialled`: `journey::address_of`, a join of `journey::next` with a timed `dial_direct_as`, then `expect`. Both are new in this change. The helper belongs in `common::journey`.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; the one `dialled` helper now lives in `org-io/tests/common/journey.rs`.

**finding-20**: guideline, low — org-io: Simplicity 4 (as finding-5) — org-io/tests/open.rs:132, :137
Two helpers here repeat helpers added in acknowledgements.rs:
- `dir_entry(dir, prefix, device)` builds the same `<prefix><hex>.store` paths as acknowledgements.rs:687 `ack_files`. Its Persona half is also `common::handles::file_of`.
- `item_held(device)` is acknowledgements.rs:680 `ack_item_held`.

open.rs:121-122 also lists directory names inline, which `common::handles::names_in` (added by this change) already does.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; `org-io/tests/open.rs` imports `item_held`, `file_of` and `names_in` from `org-io/tests/common/handles.rs` instead of its own copies.

**finding-21**: guideline, low — org-io: Simplicity 4 (as finding-5) — org-io/tests/expectations_met.rs:97
`persona_with` is the same lookup as `common::handles::persona_of` (tests/common/handles.rs:99), which this change also adds. tests/endpoints.rs:30 `device_of` and tests/store_files.rs:75 `device_key_of` are one more pair of the same lookup.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; `persona_with` is gone and the reverse lookup is `persona_of` in `org-io/tests/common/handles.rs`.

**finding-22**: guideline, low — org-io: Simplicity 4 (as finding-5) — org-io/tests/verification.rs:32, :626
`epoch_kept(a, org, b)` and `settle_deliver(admin, joining)` in the same file do one check. Each calls `journey::deliver_epoch`, asserts `ProvisionalOrganisationReceived | RevocationNoticeKept`, and asserts every delivery `Delivered`.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; `settle_deliver` is gone and `epoch_kept` in `org-io/tests/verification.rs` is the one copy.

**finding-23**: guideline, low — org-io: Simplicity 4 (as finding-5) — org-io/tests/endpoints.rs:119
`admitted(io, chain, org, tag, party)` re-implements a sequence that this change already adds to `tests/common` as `journey::invited` + `journey::admit` and `handles::requested` + `handles::proposed`. The sequence is: issue an Invite, propose the Member, request admission and keep the request, propose the admission and the updated Organisation, write, verify.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; `admitted` in `org-io/tests/endpoints.rs` now builds on `common::handles::proposed` instead of re-implementing the sequence.

**finding-24**: guideline, low — org-io: Simplicity 3, "Code no path reaches, an unused function, import or variable, and commented-out code are deleted" — org-io/src/test_support.rs:428
`TestClock::advance` has no caller in org-io/tests, org-io/src or the app. The tests move the clock with `TestClock::set`.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; `TestClock::advance` is deleted from `org-io/src/test_support.rs`.

**finding-25**: guideline, low — org-io: Simplicity 3 (as finding-12) — org-io/tests/rng_source.rs:76, org-io/tests/store_files.rs:96
Two doc lines are left over from helpers that moved to `common::handles::names_in`:
- rng_source.rs:76 says "/// The names in `dir`, sorted." and now opens the doc of an unrelated test.
- store_files.rs:96 says "/// The directory's file names, sorted." and documents nothing.

Both mislead the reader and should be deleted.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; the left-over doc lines are deleted from `org-io/tests/rng_source.rs` and `org-io/tests/store_files.rs`.

**finding-26**: guideline, low — org-io: Simplicity 1, "The change does what its tests and requirements ask and nothing more: no extra feature, no case no test exercises" — org-io/src/lib.rs:381, :385, :435
The code itself says that no test reaches these branches:
- `OperationError::AdmissionNotListed`, `OperationError::ProposalNotListed` and `CreateError::FoundingNotListed`, raised at propose.rs:49, propose.rs:80 and persona.rs:67. tests/persona_creation.rs:130 says of `FoundingNotListed` that it "is reached by no test".
- `ListingFailure::Failed` (acknowledge.rs:416-420, "so `Failed` is reached by no test"), reached through acknowledge.rs:375 and retention.rs:207.
- The `map_or` fallback at receiver.rs:170-174 ("the fallback is never taken").
- The `None => Step::Silent` arm at receiver.rs:208-214 ("`Silent` here is not reached").

This is a gap where the code is right: returning a typed error beats panicking under the crate's `deny(clippy::panic)`, and Errors 1 asks for it. The resolution is the owner's: a seam that reaches each branch, or a recorded waiver.
disposition: not fixed — the plan's round 5 fix (task `o5`) records that this low, the unreachable typed branches, needs the owner's call between a seam and a waiver; the untested-clause notes on LLR-un47j8 and LLR-5fchwt cover `AdmissionNotListed` and `ProposalNotListed`, and LLR-4yte89's note covers `FoundingNotListed`. No ruling followed, so it is carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'), `docs/plans/2026-10-21-org-io-domain-api.md`; the `org-io/src/receiver.rs` fallbacks are re-reported as round 6 low 8.

**finding-27**: guideline, low — org-io: Comments 2, "A comment describes the code as it is, never the change that wrote it" — org-io/tests/chain_write.rs:473-474
- chain_write.rs:473-474 tells the history of the fix: "Reproduces PR-8ghk9s: the timeout recorded an unknown write that verification of a founding with no proxy could never settle, refusing every later write for the founding." The problem-report ID belongs on the test. The account of what the code used to do belongs in the PR record.
- tests/receive_step.rs:51, "LLR-eeh4pq (superseding LLR-9arvqj)", records which requirement replaced which. That is history, not the code as it is.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; the fix-history wording is gone from `org-io/tests/chain_write.rs` and `org-io/tests/receive_step.rs`, which cite the item the test answers.

**finding-28**: guideline, low — org-io: Comments 3, "A change comments at about the rate of the file it edits" — org-io/src/write.rs:265-268
In a file whose enums document every variant (`ChainWriteError`, `WriteReport`, `WriteFailure`, and `WriteRefused`'s own `Node` and `Store`), `WriteRefused::NotHeld`, `NoProxyAccount`, `Stale` and `OutcomeUnknown` have no doc comment.
disposition: fixed in task `o5` (the plan's round 5 fix records CODE lows 2–13, 15 and 16 fixed in one pass over org-io/tests, the renames done by a script that skips strings and comments and compiles after each step), `docs/plans/2026-10-21-org-io-domain-api.md`; `WriteRefused::NotHeld`, `NoProxyAccount`, `Stale` and `OutcomeUnknown` each carry a doc comment in `org-io/src/write.rs`.

**finding-29**: guideline, low — org-node: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-node/src/service.rs:621
- `last_committed_epoch: record_epoch.unwrap_or_default()` picks epoch 0 for a store with no record by way of `Epoch`'s `Default`.
- No requirement is named there. The code this replaced wrote `Epoch::new(0)` with a comment tying it to the first admission.
- The value is right, but it is now a silent default.
disposition: fixed in task `n5` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 5 fix — CODE and TEST org-node, CODE src lows fixed): the epoch default is written out with a comment citing `VerifyContext` instead of `Epoch`'s `Default`, in `org-node/src/service.rs`.

**finding-30**: guideline, low — org-node: Errors 3, "No silent fallback" — org-node/src/service.rs:539
- `base_root: base_root.unwrap_or(root)` puts the answer's own root in place of a missing base.
- The comment above says why (no adoption reads it), but no requirement names this default.
- A stand-in that looks like a real base root misleads anyone who later reads `base_root` of an `Answer`.
disposition: not fixed — the plan's round 5 fix (task `n5`) lists it among the CODE src lows not fixed: removing the stand-in `base_root` would change behaviour without an LLR, so `base_root.unwrap_or(root)` stays in `org-node/src/service.rs`; re-reported in later rounds and carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-31**: guideline, low — org-node: Errors 3, "No silent fallback" — org-node/src/service.rs:1172
- `admission_requests` returns `""` as the invitee name when no outstanding invite matches (`.map_or("", …)`).
- The doc says that "never" happens. If the store is inconsistent, it shows up as an empty name rather than an error.
disposition: not fixed — the plan's round 5 fix (task `n5`) lists it among the CODE src lows not fixed: refusing instead of returning `""` would change behaviour without an LLR, so the `map_or("", …)` stays in `org-node/src/service.rs`; re-reported in later rounds and carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-32**: guideline, low — org-node: Errors 2, "no discarded return code" / Simplicity 3 — org-node/src/store.rs:986; callers org-node/src/service.rs:478, org-node/src/verify_org.rs:221, :275
- `StoreData::set_address` returns "true exactly when the entry changed". All three callers throw that result away, the first with `let (_, write)`, and none says why.
- service.rs:475 instead redoes the same "unchanged?" test by hand before calling it. Either the bool or the hand-written check should go.
disposition: fixed in task `n5` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 5 fix — CODE and TEST org-node, CODE src lows fixed): `StoreData::set_address` returns nothing now, so no caller discards a result, in `org-node/src/store.rs`.

**finding-33**: guideline, low — org-node: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays" — org-node/src/service.rs
- The change repeats several checks inline:
  - "Sender listed in the record" at :386, :447 and :509. This replaced the single `sender_listed` helper the change deleted.
  - "Identity already held" (`held.root == root && held.org_pub_key == org_pub_key`) at :531 and :635.
  - "Change is an `Admit` naming `invite_id`" at :916 and :1511.
  - "Kept request for `invite_id`" lookup at :908, :1459 and :1508, and again at verify_org.rs:314.
  - "Record's Devices other than own" at :875 and :1104.
- These state one rule each, so they will drift.
disposition: fixed in task `n5` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 5 fix — CODE and TEST org-node, CODE src lows fixed): each repeated check is one helper in `org-node/src/store.rs`: `lists_device`, `devices_other_than`, `kept_request`, `holds_provisional_identity` and `admits`.

**finding-34**: guideline, low — org-node: Names 3, "no `data` … alone" — org-node/src/person_store.rs:169
- `PersonStore::data()` is a new public accessor named `data`. It returns `PersonStoreContent`, so the name says less than the type does.
- Unlike every other function in the file, it has no doc comment (Comments 3).
- Only tests call it. org-io reads the Person through `view()`.
disposition: fixed in task `n5` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 5 fix — CODE and TEST org-node, CODE src lows fixed): the accessor is `PersonStore::content()`, documented, in `org-node/src/person_store.rs`.

**finding-35**: guideline, low — org-node: Simplicity 2, "No parameter, flag, setting or hook without a caller that uses it now" — org-node/src/ids.rs:68
- `EndpointAddresses` derives `Default`, but nothing in org-node or org-io calls `EndpointAddresses::default()` or uses it through a containing `Default`.
- It is also a second way to build the type besides `parse`, although the type's doc says it is "Constructed only through `parse`".
disposition: fixed in task `n5` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 5 fix — CODE and TEST org-node, CODE src lows fixed): `EndpointAddresses` no longer derives `Default`, so `parse` is its only constructor, in `org-node/src/ids.rs`.

**finding-36**: guideline, low — org-node: Names 4, "One name per concept across the change" — org-node/src/wire.rs:25, :65
- The concept the change calls an "organisation answer" (`keep_organisation_answer`, `Checked::OrganisationAnswerKept`, `Origin::Answer`, the wire doc "the answer to an organisation request") is named `WireMessage::Organisation` and `ORGANISATION_INDEX` on the wire.
disposition: not fixed — the plan's round 5 fix (task `n5`) lists it among the CODE src lows not fixed: renaming `WireMessage::Organisation` is a public wire rename, so `org-node/src/wire.rs` keeps the name; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-37**: guideline, low — org-node: Names 3, "A name says what the thing is in the project's domain" — org-node/src/service.rs:1408
- `fn mark(data, verified)` sets or clears the behind marker, but the name does not say what it marks. Something like `update_behind_marker` would.
disposition: fixed in task `n5` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 5 fix — CODE and TEST org-node, CODE src lows fixed): `mark` is renamed `update_behind_marker` in `org-node/src/service.rs`.

**finding-38**: guideline, low — org-node: Simplicity 3, "Code no path reaches … deleted" — org-node/src/service.rs:492-501
- `keep_organisation_answer` takes a whole `WireMessage` and refuses anything but `Organisation` with `MalformedMessage`.
- Both callers (:420, :467) pass only an `Organisation`, so that branch is unreachable. It exists only because of how the parameter was chosen.
disposition: fixed in task `n5` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 5 fix — CODE and TEST org-node, CODE src lows fixed): `keep_organisation_answer` takes the answer's fields instead of a whole `WireMessage`, so the unreachable `MalformedMessage` branch is gone, in `org-node/src/service.rs`.

**finding-39**: guideline, medium — org-node: Simplicity 3, "an unused function, import or variable … deleted, not kept for later" — org-node/tests/support/mod.rs
- This change removed the last caller of these helpers (I checked: zero callers outside support/), yet kept them, and edited some:
  - `reopen_store` :152 and `Reopened` :182-205, whose body was rewritten at :194-197
  - `disk_rec_of` :555, `id_by_handle` :560, `device_by_handle` :609-615
  - `spawn_recv_one` :565, `Inbox`/`Post`/`Deferred` :394-432 and `Relay::send` :455
  - `deliver` :711, `deliver_from`, `deliver_raw`
  - `Node::receive_posted` :366-370 (signature rewritten), `sender_for`/`sending_as`/`sends_as`
  - `trie_without` :752-760 (rewritten) and `store_dir` :526
- `persona_id_for` :373-376 is new and has no caller at all.
- `#![allow(dead_code)]` at :22 hides all of this.
disposition: fixed in task `n5` (C5-N11): every support helper no target calls is deleted, found by compiling every target with the dead-code allows removed; the `allow(dead_code)` in `org-node/tests/support/mod.rs` stays with a stated reason (each target uses a subset of support), `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-40**: guideline, medium — org-node: Comments 1, "It states a reason, a constraint …"; a false constraint misleads — org-node/tests/support/mod.rs:4-7, :18-21
- The module doc says the story steps "are the only place a test performs one". That is untrue:
  - support/sweep.rs adds a second set (`found`, `founded`, `admission`, `revocation`, `request_admission`).
  - Tests call the operations directly, for example service_lifecycle.rs:41 and :88, sealed_store.rs:55, and verification.rs:373 and :434.
- :20-21 present `reopen_store`, which no test calls (finding-11), as how "reached the disk" is read.
disposition: fixed in task `n5` (C5-N12): the module doc of `org-node/tests/support/mod.rs` is rewritten to match what support holds and how tests use it, `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-41**: guideline, medium — org-node: Simplicity 4, "one copy stays, where both can reach it" — org-node/tests/support/mod.rs vs org-node/tests/support/sweep.rs
- `Node` and `Side` model the same thing (one Persona's service plus what org-io wrote). Both modules are loaded together in six targets.
- Their helpers repeat one for one:

  | What | support/mod.rs | support/sweep.rs |
  |---|---|---|
  | verification time | `story_time` :384 | `at` :30 |
  | panicking secret source | `never_asked` :390 | `no_secrets` :36 |
  | proxy account | `test_proxy` :577 | `the_proxy` :25 |
  | adopt step | `Node::adopt` :315 | `adopt` :142 |
  | admission request step | `admit_b` :796-800 | `request_admission` :201-207 |
  | seed-bytes extraction | `Keychain::keep` :54-60 | `seed_bytes` :41-50 |

- Each pair is also two names for one concept (Names 4).
disposition: fixed in task `n5` (C5-N13): sweep is now `support::sweep` (`org-node/tests/support/sweep.rs`), a `Side` is a `Node` plus its Device, and each concept has one step and one name, `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-42**: guideline, medium — org-node: Simplicity 4, "one copy stays, where both can reach it" — fixture builders copied across test files
- `snapshot(id, handle, member, device)` has copies at:
  - invitation.rs:284 and organisation_request.rs:36
  - own_removal.rs:49 and proposals.rs:591
  - outgoing_bodies.rs:45, receive_path.rs:666, sealed_store.rs:447, secret_redaction.rs:371, store_at_rest.rs:276 and verification.rs:61
- The record builder over `OrgRecordParts` has copies at:
  - invitation.rs:295-309 and organisation_request.rs:47-64
  - own_removal.rs:67 and proposals.rs:614
  - receive_path.rs:691, verification.rs:107, sealed_store.rs:461, secret_redaction.rs:411 and store_at_rest.rs:301
- Smaller fixtures are identical in more than one file:
  - `OWN`/`org()`/`addresses()`: invitation.rs:259-264 and organisation_request.rs:21-29
  - `org()`: own_removal.rs:31 and proposals.rs:585
  - `persona`/`service`/`plaintext`: receive_path.rs:708/737/741 (`nf`) and verification.rs:124/146/155 (`fx`)
  - `snapshots_of`: store_at_rest.rs:287 and verification.rs:99
- `MemberLeaf`-from-snapshot code repeats support `trie_of_members` (:769) at verification.rs:86, secret_redaction.rs:389-402, store_at_rest.rs:330 and outgoing_bodies.rs:69-84.
- Each test file is reachable from support/, so one copy could serve all of them.
disposition: fixed in task `n5` (C5-N14): the new `org-node/tests/support/fixtures.rs` holds every shared builder, the test files build through it, and the ob/rc/nf/fx modules are renamed (`docs/plans/2026-10-21-org-io-domain-api.md`). Round 7 found hand-built sites remaining (C7-N1, a recurrence), fixed by owner ruling 2026-10-29 in task `n7`.

**finding-43**: guideline, medium — org-node: Simplicity 4, "two copies of one rule drift" — org-node/tests/absences.rs:648, :688
- Two tests each define their own `removing` list of removing methods, and the copies have already drifted: only the :688 list has `"dedup"`.
- As a result, `a_kept_acknowledgement_is_removed_only_on_acceptance` would not catch an `acknowledgements.dedup(` added outside `acknowledgement_accepted`, so a check that should fail would pass.
disposition: fixed in task `n5` (C5-N15): `org-node/tests/absences.rs` has one `REMOVING_METHODS` list, with `dedup`, which `a_kept_acknowledgement_is_removed_only_on_acceptance` uses; the plan's red -> green record shows it failing with a probe calling `acknowledgements.dedup_by` (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-44**: guideline, medium — org-node: Errors 2, "no discarded return code or error value" — org-node/tests/proposals.rs:478, :491
- `let _ = sweep::adopt(&mut founder, chain);` throws away the `Verified` result, and `sweep::adopt` (sweep.rs:142) does not check that the result is `Adopted`.
- The asserts that follow ("verify's adoption", "verify's adoption of an update") would still pass on `Behind` or `Inconsistent`, because then nothing changed. The result is the thing the step claims, so it should be asserted.
disposition: fixed in task `n5` (C5-N16): `sweep::adopt` in `org-node/tests/support/sweep.rs` asserts the result is `Adopted`, so the proposals.rs steps can no longer pass on `Behind` or `Inconsistent`, `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-45**: guideline, medium — org-node: Simplicity 4, "one copy stays, where both can reach it" — the "lend the seed to the keychain, reopen it" helper
- Copies: proposals.rs:29-38 (`copy_of`), proposals.rs:119-124 (inline), person_store.rs:16-25 (`reopened_secret`) and sealed_store.rs:641-647 (inline, although `keychain_bytes` is at :34 of the same file).
- `sweep::seed_bytes` (sweep.rs:40, new) already does this, and proposals.rs already declares `mod sweep`.
- The helper also has three names: `copy_of`, `reopened_secret` and `keychain_bytes` (Names 4).
disposition: fixed in task `n5` (C5-N17): `seed_bytes` and `secret_copy` in `org-node/tests/support/mod.rs` replace the copies and their several names, `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-46**: guideline, medium — org-node: Names 4, "where the codebase already names it, that name" — org-node/tests/outgoing_bodies.rs:167, :257
- `pub fn joiner()` and `let (joiner, key) = ob::joiner();` call a provisional Member a "joiner".
- The change's own absence tests rule that concept name out ("no Joiner type", absences.rs:72 and epoch_store.rs:233/251). The change everywhere else calls it a provisional Member.
disposition: fixed in task `n5` (C5-N18): `joiner` is renamed `provisional_member`, now `fixtures::provisional_member` in `org-node/tests/support/fixtures.rs`, `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-47**: guideline, low — org-node: Simplicity 4 — further duplicated helpers
- Panicking secret source: own_removal.rs:197 `never_asked`, in addition to the pair in finding-13. sealed_store.rs:503 has a `no_secrets` that returns an empty list instead of panicking, so one name covers two behaviours.
- Bound measurement: two different measurements of the LLR-m76r4k tuple, proposals.rs:712 `rc::bound_len` and provisional_store.rs:39 `measured`. The search loop at proposals.rs:1105-1118 also repeats `rc::filled_before` :733-753.
- `epoch_store.rs:29-31 fn handle` duplicates support `handle` :465.
- receive_path.rs:84-85 `device(seed)` is `test_fixtures::device_key`, and receive_path.rs:30 `device_of` duplicates the new `Node::device_of` (support/mod.rs:308).
- Within verification.rs `fx`, these pairs are identical apart from one argument: `verify` :169 / `verify_as` :343, `receive` :205 / `receive_into` :335, and `plaintext` :155 / `plaintext_of` :355.
- Pub-signature scanners: sealed_store.rs:669-716 and secret_redaction.rs:441-474.
- store_at_rest.rs:82 `contains` duplicates sweep `contains` :53.
- service_lifecycle.rs:38 `hex16` duplicates support `persona_id_for`.
- absences.rs repeats `sources().into_iter().find(…ends_with(X))…` at :123, :156, :286 and :290, and reads single files a second way at :689, :784 and :803.
- secret_redaction.rs strips whitespace three times (:47, :71, :214).
disposition: fixed in task `n5` (the plan's round 5 fix records CODE test lows 19–27 fixed), `docs/plans/2026-10-21-org-io-domain-api.md`; the duplicated helpers moved into `org-node/tests/support/mod.rs` and `org-node/tests/support/fixtures.rs`.

**finding-48**: guideline, low — org-node: Errors 2, "Where an error is expected and handled, the handling says why that is safe" — discarded results with no reason given
- Bare `let _ = …unwrap()`:
  - proposals.rs:209, :432, :434, :494, :503, :515, :522, :549 and :1277
  - provisional_store.rs:70, where `filler_for` drops the `Checked` without confirming it was a keep
- `#[must_use]` `StoreChange` values dropped:
  - invitation.rs:459, organisation_request.rs:168, outgoing_bodies.rs:151/:154/:156
  - sealed_store.rs:70, store_at_rest.rs:358
  - verification.rs:262-263, :327, :435, :627-628, :686-690, :794 and :865-866
- verification.rs:506 `let _outcome = fx::verify(...)` drops a `Result`. The `_none` bindings at verification.rs:430, :482, :485 and :525 are never asserted to be `None`.
disposition: fixed in task `n5` (the plan's round 5 fix records CODE test lows 19–27 fixed), `docs/plans/2026-10-21-org-io-domain-api.md`; the discards were found by grepping every `let _` binding, and dropped `StoreChange` values now go through `support::unread` in `org-node/tests/support/mod.rs`.

**finding-49**: guideline, low — org-node: Names 3, "no `data`, `info`, `item`, `value`, `result` … alone"; opaque abbreviations
- Functions named `data`: sweep.rs:63, verification.rs:150 (`fx::data`), outgoing_bodies.rs:142 and proposals.rs:689.
- Bindings and parameters named `data`:
  - own_removal.rs: 89, 101, 137, 142, 146, 204, 365, 431, 460, 479, 591, 613, 621, 659, 783
  - proposals.rs: 79, 155, 191, 379, 669, 712, 733, 853, 869, 1008, 1261, 1333
  - epoch_store.rs: 164, 177, 209, 220, 241
  - organisation_request.rs: 76, 85, 114, 250, 314, 446
  - receive_path.rs: 473, 589, 724, 732, 737, 770, 791
  - verification.rs: 719, 828, 862, 950, 975, 1034, 1053
  - persona_records.rs: 147, 156, 442
  - provisional_store.rs: 39, 50, 120, 128
  - receive_chain_reads.rs: 126, 178
  - invitation.rs: 333, 432
  - person_store.rs: 36, 149
  - secret_redaction.rs: 139, 230
  - support/mod.rs: 175, 341
  - encoding_golden.rs: 102
  - outgoing_bodies.rs: 115, 134
- Other generic names: `value` at absences.rs:709 and node_value_types.rs:39, and `info` at encoding_golden.rs:179.
- Abbreviated module names: `mod ob` outgoing_bodies.rs:17, `mod rc` proposals.rs:560, `mod nf` receive_path.rs:644 and `mod fx` verification.rs:23.
- Short bindings and closures:
  - `dk`: invitation.rs:77, :97, :119 and own_removal.rs:327
  - `po`: persona_records.rs:409
  - `per`: proposals.rs:742, :1111 and receive_path.rs:798
  - `one`: proposals.rs:1104 and own_removal.rs:216
  - `some`: own_removal.rs:248
  - `of`: proposals.rs:629
  - `with`: receive_path.rs:792
  - `what`: receive_path.rs:512
- Other unclear names:
  - `long` beside `long_surname` at organisation_request.rs:251
  - sweep.rs:30 `at()`, which does not name a time
  - support/mod.rs:468 and :471 `nm` and `sn`
disposition: fixed in task `n5` (the plan's round 5 fix records CODE test lows 19–27 fixed), `docs/plans/2026-10-21-org-io-domain-api.md`; the generic `data`/`value`/`info` names, the ob/rc/nf/fx module names and the short bindings were renamed after a grep of generic and short names across org-node/tests, for example in `org-node/tests/proposals.rs`.

**finding-50**: guideline, low — org-node: Names 1, "Every name the change writes … says what it holds" — single-character names
- org-node/tests/person_store.rs:66 has the pattern binding `field: f`. persona_records.rs:477 and :522 write the same match as `field: named`.
- org-node/tests/node_value_types.rs:39 uses the type parameter `T`.
disposition: fixed in task `n5` (the plan's round 5 fix records CODE test lows 19–27 fixed), `docs/plans/2026-10-21-org-io-domain-api.md`; the single-character names in `org-node/tests/person_store.rs` and `org-node/tests/node_value_types.rs` are renamed.

**finding-51**: guideline, low — org-node: Names 4, "One name per concept across the change"
- B's MemberId is `b_member_id` at admission_sender.rs:37 but `b_id` at :170, :313 and :351.
- A `Node` is bound as `svc` at organisation_key.rs:43 but as `node` at invitation.rs:41.
- The store's own Device is `me()` at outgoing_bodies.rs:138 but `own()` at organisation_request.rs:89 and invitation.rs:427.
- `addresses` returns `EndpointAddresses` at epoch_store.rs:48 but `Vec<String>` at invitation.rs:264 and organisation_request.rs:29.
- The third slot of `propose_member` is `_created` at invitation.rs:370 and organisation_request.rs:146 but `_sealed` at outgoing_bodies.rs:177.
- A store's encoded plaintext is called `encoded` (own_removal.rs:150, sealed_store.rs:80), `plaintext(s)` (proposals.rs:52, :693) and `bytes_of` (sweep.rs:69).
- `chain` names two different builders: own_removal.rs:360 and proposals.rs:701.
- The invite byte is `invite_byte` at proposals.rs:648 but `invite` at proposals.rs:869.
- sealed_store.rs:648 shadows `seed` and `secret`.
disposition: fixed in task `n5` (the plan's round 5 fix records CODE test lows 19–27 fixed), `docs/plans/2026-10-21-org-io-domain-api.md`; each concept the finding lists has one name, for example in `org-node/tests/admission_sender.rs`.

**finding-52**: guideline, low — org-node: Simplicity 3, "an unused function … deleted" — org-node/tests/persona_records.rs:231-232
- The `#[allow(dead_code)] Answer { from: [u8; 32] }` variant of `WireOrigin` is never built.
- It is the last variant, so deleting it changes no index.
disposition: fixed in task `n5` (the plan's round 5 fix records CODE test lows 19–27 fixed), `docs/plans/2026-10-21-org-io-domain-api.md`; the never-built `WireOrigin::Answer` variant is deleted from `org-node/tests/persona_records.rs`.

**finding-53**: guideline, low — org-node: Comments 2, "A comment describes the code as it is, never the change that wrote it"
- Comments that describe earlier code:
  - absences.rs:261-262, "the removed judging operations are gone"
  - epoch_store.rs:200, "no provisional update survives"
  - epoch_store.rs:232-233, :207 and :251, "no `Joiner` type remains" / "still names"
  - organisation_key.rs:161 and :188, "(ported, design gap G2)"
  - organisation_key.rs:6-7, "which superseded LLR-3fwykc"
  - receive_path.rs:430, "PR-q8r32t stays resolved"
  - verification.rs:907, "the successor of LLR-qezw3n"
  - value_types.rs:114, :121, :136, :149 and :176 ("gone", "removed", "adds")
- Test names that use "new" or "added": value_types.rs:150, :185 and :189.
disposition: fixed in task `n5` (the plan's round 5 fix records CODE test lows 19–27 fixed), `docs/plans/2026-10-21-org-io-domain-api.md`; the comments and test names that narrated earlier code are rewritten, for example in `org-node/tests/value_types.rs`.

**finding-54**: guideline, low — org-node: Comments 1, a comment that says something false about the code
- own_removal.rs:177 says "`content`'s plaintext", but the parameter is `plaintext`.
- proposals.rs:647 says "for `invite`", but the parameter is `invite_byte`.
- support/mod.rs:711 says "to `addr`", but the parameter is `to`.
disposition: fixed in task `n5` (the plan's round 5 fix records CODE test lows 19–27 fixed), `docs/plans/2026-10-21-org-io-domain-api.md`; each comment names the parameter it means, for example in `org-node/tests/own_removal.rs`.

**finding-55**: guideline, low — org-node: Errors 4, "An edit … that matches nothing is an error" — org-node/tests/store_at_rest.rs:254, org-node/tests/receive_path.rs:394-398
- A tamper written as `if let Some(record) = data.record.as_mut()` silently does nothing when there is no record.
- At store_at_rest.rs, the `kept = None` case could then pass without ever editing anything. This is contrived, because the fixture always has a record.
disposition: fixed in task `n5` (the plan's round 5 fix records CODE test lows 19–27 fixed), `docs/plans/2026-10-21-org-io-domain-api.md`; the `if let Some(record) = … as_mut()` tamper that could edit nothing is gone from `org-node/tests/store_at_rest.rs` and `org-node/tests/receive_path.rs`.

### Round 6

**finding-1**: guideline, low — org-io: Names 1, "No single-character names. Every name the change writes, a loop index and a callback parameter included, says what it holds" — tests/common/journey.rs:232, tests/common/journey.rs:292-294, tests/common/chain_contract.rs:45-48, tests/common/chain_contract.rs:58-61, tests/receive_step.rs:46
The generic parameters `T` (`while_receiving<T>`), `F` (`all<F: Future>`, `F::Output`), `F` beside `Fut` (`reader_and_writer_contract<F, Fut>`) and `F` (`received_while<F: …>`), and the lifetime `'a` (`ChainUnderTest<'a>`), are one-letter names the change chose. The language does not fix them. The crate's own code shows the clause's form: `Cause<Failure>` (src/lib.rs:330) and `bounded<Written>` (src/write.rs:351). Better names would be `Output`, `Work` or `Finalise`, and `'chain`.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 6 section closes the CODE org-io loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-io/tests/common/journey.rs` and `org-io/tests/common/chain_contract.rs`.

**finding-2**: guideline, low — org-io: Names 4, "One name per concept across the change. A concept has one name in every file, test and task of the change" — src/lib.rs:96, src/write.rs:327, src/write.rs:332, src/write.rs:380, src/write.rs:429, src/write.rs:450, src/verify.rs:165, tests/absences.rs:375
The change replaces `publish.rs` and `submit.rs` with `write.rs`, and names the concept "the write": `WriteReport`, `WriteFailure`, `unknown_writes`, `write_organisation_to_chain`. The state that holds the unknown writes and the verifications is still `PublishState`, held in the field `publish`. It is read through locals called `publish`, and the comment at write.rs:429 calls it "the publish lock". In tests/absences.rs:375 the text of `src/write.rs` is bound to `submit`. So one concept goes by three names. A reader of `publish` looks for a publish step that no longer exists.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 6 section closes the CODE org-io loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-io/src/write.rs`, where the state is still `PublishState` in the field `publish`.

**finding-3**: guideline, low — org-io: Names 4, "a concept has one name in every file … not `account_id` in one file and `customer_id` in another for the same value" — tests/acknowledgements.rs:41, tests/acknowledgements.rs:288, tests/chain_write.rs:35, tests/chain_write.rs:44, tests/common/journey.rs:37, tests/common/journey.rs:138, tests/common/journey.rs:257, tests/common/journey.rs:417, tests/endpoints.rs:114, tests/endpoints.rs:126, tests/verification.rs:32, tests/verification.rs:57 (against tests/common/handles.rs:244, tests/common/chain_contract.rs:98, tests/persona_creation.rs:30, tests/verification.rs:475 and all of `src/`)
An `OrgId` is `org_id` in every file under `src/` and in four of the added test signatures, but `org` in twelve others (`Admin::org`, `ack_report_gone(…, org: OrgId, …)`, `epoch_kept(…, org: OrgId, …)`). Both names appear in tests/verification.rs alone (lines 32 and 57 against line 475).
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 6 section closes the CODE org-io loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-io/tests/common/journey.rs` and the test files the finding lists.

**finding-4**: guideline, low — org-io: Names 4, "One name per concept across the change" — tests/acknowledgements.rs:223, tests/acknowledgements.rs:306, tests/acknowledgements.rs:1363, tests/common/journey.rs:110, tests/common/journey.rs:154, tests/common/journey.rs:187, tests/common/journey.rs:201, tests/common/journey.rs:429, tests/invitation_exchange.rs:41
Nine doc comments name the Device key parameter `` `device` `` ("an endpoint of `device`", "the Device `device`", "`device`'s store keeps"). Each function they document calls the parameter `device_key`, and no `device` is in scope. The same value has two names, and one of them is a name the reader cannot find in the signature.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 6 section closes the CODE org-io loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is the doc comments in `org-io/tests/common/journey.rs` and `org-io/tests/acknowledgements.rs`.

**finding-5**: guideline, low — org-io: Names 4, "One name per concept across the change" — tests/verification.rs:57, tests/verification.rs:218-229 (with tests/verification.rs:30, 41, 56, 208)
Alice's handle is `alice` in code, "A" in the comments, and the `a_` of `a_clock` (Alice's clock). Bob's clock, a few lines up, is plain `clock`. Bob is `bob`, "B" and the `_b` of `remove_b`. Bob's Device key is `joiner` here, while `joiner` names the joining `OrgIo` in tests/common/handles.rs:231 and 244. `alice_clock`, `bob_clock` and `remove_bob` would give each party one name.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 6 section closes the CODE org-io loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-io/tests/verification.rs`.

**finding-6**: guideline, low — org-io: Simplicity 4, "No duplication between tasks. Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it" — tests/app_boundary.rs:224-238, tests/persona_creation.rs:273-287, tests/chain_read_rule.rs:57-75, tests/rng_source.rs:52-74, tests/acknowledgements.rs:99-112
The change adds five copies of the walk that collects every `.rs` file under a directory:
- `rust_files_under` twice, the two bodies identical;
- `org_io_code`'s inner `walk`;
- `rng_source.rs`'s `sources`, whose inner `walk` and `code_lines` repeat what tests/absences.rs already has as `rust_files` / `sources` / `code_lines`, under the same names;
- an inline stack walk in acknowledgements.rs.

Four of them repeat the same `line.split("//").next().unwrap_or("")` comment strip. All the files can reach `tests/common/`, so one copy should stay there.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 6 section closes the CODE org-io loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is the walks in `org-io/tests/app_boundary.rs`, `org-io/tests/persona_creation.rs`, `org-io/tests/chain_read_rule.rs`, `org-io/tests/rng_source.rs` and `org-io/tests/acknowledgements.rs`.

**finding-7**: guideline, low — org-io: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays" — tests/common/handles.rs:231-240, tests/common/journey.rs:79-85, tests/common/journey.rs:272-276
Three copies of one check: the joining side asks for admission over the transport while the issuer's receive step keeps it, then the code asserts `Received::AdmissionRequestKept` and `Delivery::Delivered`. They are `handles::requested`, `journey::keep_request` and the start of `journey::admit_among`. The three differ only in which side is told to `start_receiving`. A later change to the rule (for example, a new verdict) would have to be made three times.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 6 section closes the CODE org-io loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-io/tests/common/handles.rs` and `org-io/tests/common/journey.rs`.

**finding-8**: guideline, low — org-io: Errors 3, "No silent fallback. A missing file, setting or value is an error unless a requirement names the default"; and Simplicity 1, "no case no test exercises" — src/receiver.rs:169-174, src/receiver.rs:207-215
Two places stand in a made-up value for a missing one, and their comments call that value unreachable:
- When `send_all`'s report holds no entry, the answer's delivery becomes `Delivery::NotDelivered { reason: SendError::NoAddress }` ("the fallback is never taken"). `SendError::NoAddress` is documented as the acknowledgement delivery's own reason (src/send.rs:49), so if it were ever reached the app would get a false cause.
- When the listing does not hold the epoch org-node just kept, the outcome is `(Verdict::Accepted, Step::Silent)` ("`Silent` here is not reached"), which drops the report silently.

No requirement names either default. A state the code believes cannot happen should fail loudly, naming what was missing, not turn into a normal-looking outcome.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 6 section closes the CODE org-io loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-io/src/receiver.rs`. These are the branches round 5 low 14 left for the owner's call between a seam and a waiver.

**finding-9**: guideline, low — org-io: Errors 3, "No silent fallback. A missing … value is an error unless a requirement names the default" — src/acknowledge.rs:236-244
When the removal is no longer kept but a deletion failure is still being retried, `report` returns an `AcknowledgementReport` with `deadline: 0`. The field is documented as "The kept verification time plus 7 days" (src/view.rs:229-230). LLR-rav7dr says this report has "no recipient and that failure", but it names no deadline, let alone 0. A caller that reads `deadline` gets the Unix epoch as a real-looking deadline. The type should say the deadline is unknown (for example `Option<u64>`), or a requirement should name the value.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; its round 6 section closes the CODE org-io loop with nothing above low), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-io/src/acknowledge.rs`; LLR-rav7dr still names no deadline value.

**finding-10**: guideline, low — org-node: Errors 1, "naming what failed and on what input" — org-node/src/person_store.rs:140, org-node/src/store.rs:433, org-node/src/store.rs:1379, org-node/src/service.rs:1435
- **What:** postcard encode failures (person store, epoch-state bound, removed store, record snapshot) are reported as `OrgNodeError::Chain(..)`. That variant renders as "chain read failed: …".
- **Why:** the error names a chain read that never took place, not the encoding that failed.
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed as a behaviour or public-error change, `docs/plans/2026-10-21-org-io-domain-api.md`; the encode failures in `org-node/src/store.rs` and `org-node/src/person_store.rs` still map to `OrgNodeError::Chain`, which the plan's round 9 section names among the open lows ('OrgNodeError::Chain carries local encode failures, which org-node did before this change'); carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-11**: guideline, low — org-node: Errors 1, "naming what failed" — org-node/src/removal.rs:132, org-node/src/service.rs:1354
- **What:** a store that holds no record is refused with `OrgNotOnChain` ("no on-chain state found for org").
- **Why:** what failed is a missing record in the store. Nothing was read from the chain.
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed as a behaviour or public-error change, `docs/plans/2026-10-21-org-io-domain-api.md`; `OrgNotOnChain` stays in `org-node/src/removal.rs` and `org-node/src/service.rs`; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-12**: guideline, low — org-node: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-node/src/service.rs:1170
- **What:** `admission_requests` uses `.map_or("", …)` to return an empty invitee name when no outstanding invite matches.
- **Why:** LLR-8ru6mv (draft SDD) names no such default. The doc says it "is never returned in practice", so the code is right, but the rule is unmet.
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed as a behaviour or public-error change, `docs/plans/2026-10-21-org-io-domain-api.md`; the `map_or("", …)` in `org-node/src/service.rs` stays and LLR-8ru6mv names no default; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-13**: guideline, low — org-node: Errors 3, "unless a requirement names the default" — org-node/src/service.rs:543
- **What:** `base_root: base_root.unwrap_or(root)` puts the answer's own root in place of a base it does not have.
- **Why:** the comment explains it, but no requirement names that value.
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed as a behaviour or public-error change, `docs/plans/2026-10-21-org-io-domain-api.md`; `base_root.unwrap_or(root)` stays in `org-node/src/service.rs`; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-14**: guideline, low — org-node: Errors 2, "no discarded return code or error value" — org-node/src/invitation.rs:137
- **What:** `Invite::wire_for_test` was rewritten in this change and still ends in `.unwrap_or_default()`. An `armour` failure becomes an empty blob.
- **Why:** a test would then see an `invite` refusal for the wrong reason.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): `Invite::wire_for_test` returns a `Result`, so an `armour` failure is no longer an empty blob, in `org-node/src/invitation.rs`.

**finding-15**: guideline, low — org-node: Simplicity 2, "no interface, base class, factory or wrapper without a second caller" — org-node/src/store.rs:37-44
- **What:** the new generic `parse_each` has one caller (store.rs:408).
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): the single-caller `parse_each` is inlined in `org-node/src/store.rs`.

**finding-16**: guideline, low — org-node: Names 3, "no `data`, `info`, … alone" — org-node/src/store.rs:1194, 1211, 1233
- **What:** the new HKDF-label parameter is named `info` in `derive_key`, `labelled_cipher` and `open_sealed`.
- **Why:** the docs themselves call it "the label `info`", and the function is `labelled_cipher`, so `label` is the concrete name.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): the HKDF-label parameter `info` is renamed `label` in `org-node/src/store.rs`.

**finding-17**: guideline, low — org-node: Names 2, "no chained conditional expression standing in for an `if`" — org-node/src/verify_org.rs:312, 316
- **What:** `(!included).then(|| …)` and `still_held.then_some(…)` stand in for an `if` in `discarded_changes`.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): `discarded_changes` uses an `if` in place of `then`/`then_some` in `org-node/src/verify_org.rs`.

**finding-18**: guideline, medium — org-node: Comments 2, "A comment describes the code as it is" — org-node/tests/epoch_store.rs:2, 4, 180, 213, 236, 251
- **What:** a rename of the local `one` to `one_entry` also rewrote the word "one" in comments. They now read:
  - "The epoch state of one_entry Organisation"
  - "the endpoint addresses one_entry Device keeps"
  - "the store holds one_entry record"
  - "the one_entry name for that concept"
  - "zero, one_entry and sixteen entries"
  - "one_entry entry of 257 bytes"
- **Why:** the module doc and the test intents no longer say what the code is. I confirmed every line.
disposition: fixed in task `n6` (C6-N9): the comments in `org-node/tests/epoch_store.rs` say "one" again, and a check of the whole round 5 rename diff found no other comment or string a rename rewrote, `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-19**: guideline, medium — org-node: Names 4, "One name per concept across the change … where the codebase already names it, that name" — org-node/tests/admission_sender.rs:351, 489, 531 (also 433)
- **What:** these moved tests keep names from the retired vocabulary:
  - `a_failed_revocation_send_leaves_the_committed_removal_in_place`
  - `a_committed_update_gives_the_record_the_new_key_pair_on_both_paths`
  - `pr_u4c2vp_…_refused_on_the_self_delete_path`
- The code and these tests' own comments say adopt/adopted (`verify`, `adopt`, `adopt_first`). No commit path, no two paths and no self-delete path exist.
- **Why:** a reader is pointed at code that is not there. I confirmed the names.
disposition: fixed in task `n6` (C6-N10): the admission_sender.rs tests are renamed to what the code does now (adopted removal, relayed update, adopted update), e.g. `building_the_revocation_messages_leaves_the_adopted_removal_in_place` and `each_adopted_update_gives_the_record_the_new_key_pair` in `org-node/tests/admission_sender.rs`, `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-20**: guideline, low — org-node: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays" — org-node/tests/absences.rs:213-217, 667-671, 751-757
- **What:** the `is_public(f)` push / `public_entry_points_reaching(f)` extend step is written three times.
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed (Simplicity 4 duplicates, not selected), `docs/plans/2026-10-21-org-io-domain-api.md`; the step stays in `org-node/tests/absences.rs`; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-21**: guideline, low — org-node: Simplicity 4, "the same … check, one copy stays" — org-node/tests/absences.rs:188, 289
- **What:** the absence checks for `fn submit_` and `async fn submit_` overlap each other and the existing `org_node_writes_nothing_to_the_chain` assertion.
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed (Simplicity 4 duplicates, not selected), `docs/plans/2026-10-21-org-io-domain-api.md`; the overlapping checks stay in `org-node/tests/absences.rs`; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-22**: guideline, low — org-node: Simplicity 4, "one copy stays, where both can reach it" — org-node/tests/absences.rs:440-457 vs org-node/tests/verification.rs:797-829, and verification.rs:798 vs sealed_store.rs:238
- **What:** the LLR-ds228z source scan for `EndpointAddr`/`peer_addr`/`node_addr_for_dial` appears twice.
- verification.rs also redefines a `.rs` walker (`rust_files`) and an identifier matcher (`whole_word`). absences.rs (`sources`, `is_identifier_char`) and sealed_store.rs (`rust_sources`) already have these.
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed (Simplicity 4 duplicates, not selected), `docs/plans/2026-10-21-org-io-domain-api.md`; the scans stay in `org-node/tests/absences.rs` and `org-node/tests/verification.rs` (LLR-ds228z); re-reported as round 9 low 11 and carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-23**: guideline, low — org-node: Simplicity 4, "the same helper … one copy stays" — org-node/tests/encoding_golden.rs:79-87, key_custody.rs:21-25, organisation_key.rs:44, node_value_types.rs:191, organisation_request.rs:157
- **What:** hex encoding is written five times and hex decoding twice. `tests/support` is in reach of all five files.
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed (Simplicity 4 duplicates, not selected), `docs/plans/2026-10-21-org-io-domain-api.md`; later task `n8` (round 8 low 10) moved `support::hex` into `org-node/tests/support/mod.rs`, but the copies in targets whose features exclude support remain; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-24**: guideline, low — org-node: Simplicity 4, "the same helper … one copy stays" — org-node/tests/organisation_request.rs:52-58, 100-107, 367-374; invitation.rs:246-257; epoch_store.rs:102; outgoing_bodies.rs:62; proposals.rs:576
- **What:**
  - `encoded(svc)` re-wraps `support::fixtures::encoded`.
  - `own()` is defined again in several modules: outgoing_bodies, proposals, own_removal and sealed_store.
  - The Acme Invite fixture is written out twice in organisation_request.rs and again in invitation.rs, and the issuer addresses already differ between them.
  - `kept_request` exists in both invitation.rs and epoch_store.rs.
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed (Simplicity 4 duplicates, not selected), `docs/plans/2026-10-21-org-io-domain-api.md`; later tasks `n7` and `n8` (C7-N1, C8-N1 and round 8 lows 4–6) replaced the local `own()` copies, `kept_request` and the Invite fixture with shared ones in `org-node/tests/support/fixtures.rs`.

**finding-25**: guideline, low — org-node: Simplicity 4 / Names 4, "A concept has one name in every file" — org-node/tests/receive_path.rs:688-692
- **What:** `notices::DEE` / `dee()` repeat `fixtures::DAVE` / `dave()` (0x04, 0x64, [0xd1]) under a second name. Only the Handle differs.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): `notices::DEE`/`dee()` no longer repeat `fixtures::DAVE`/`dave()` in `org-node/tests/receive_path.rs`.

**finding-26**: guideline, low — org-node: Names 4, "One name per concept" — org-node/tests/outgoing_bodies.rs:34, 206
- **What:** `bodies::eve()` shadows the glob-imported `fixtures::eve()` with a different Eve that has two Devices.
- Line 34 writes the literal 0xe1 instead of `EVE`, and line 206 writes `device_key(0xc1)` instead of `CAROL`.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): the shadowing `bodies::eve()` is renamed `eve_with_two_devices` and the literal seeds use the fixtures' names, in `org-node/tests/outgoing_bodies.rs`.

**finding-27**: guideline, low — org-node: Names 4, "One name per concept" — org-node/tests/receive_path.rs:323
- **What:** this line uses `device_a`. The same file (:43, :455) and the other test files use `a_device`.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): `device_a` is `a_device` in `org-node/tests/receive_path.rs`.

**finding-28**: guideline, low — org-node: Names 4 / Names 3 — org-node/tests/proposals.rs:21, 660, 907
- **What:** the local `changes` (a `Vec<ProvisionalChange>`) has the same name as the `changes` fixtures module used on the next lines.
- The file-local `fn service()` and `changes::service(..)` also share one name for two concepts.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): the local `changes` and the file-local `service()` no longer share names with the `changes` module in `org-node/tests/proposals.rs`.

**finding-29**: guideline, low — org-node: Names 3, "A name says what the thing is in the project's domain, not … a role in general" — org-node/tests/own_removal.rs:783, 790
- **What:** two `OrgService` values are named `check` and `cancel`, after the operations later run on them.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): the `OrgService` values named `check` and `cancel` are renamed to what they hold in `org-node/tests/own_removal.rs`.

**finding-30**: guideline, low — org-node: Simplicity 4, "the same … check, one copy stays" — duplicated test checks under one LLR:
- receive_path.rs:83-85 repeats :96-98 (malformed bodies, LLR-94rzrk);
- proposals.rs:99 repeats :111 (`DuplicateOrgPublicKey`, LLR-9qavm7);
- proposals.rs:270 repeats :130 (first invalid field, LLR-rjphn4);
- proposals.rs:857 repeats :833-837 (`MemberNotHeld`);
- proposals.rs:250 repeats persona_records.rs:119 (NFC details);
- own_removal.rs:569 repeats :474 (`DeviceSecretNotSupplied`, nothing changed);
- revocation.rs:592-604 repeats :155-163 (acknowledgement signed bytes, LLR-bs6yxf).
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed (Simplicity 4 duplicates, not selected), `docs/plans/2026-10-21-org-io-domain-api.md`; the duplicated checks stay, for example in `org-node/tests/proposals.rs`; re-reported as round 7 low 3 and round 9 low 13 and carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-31**: guideline, low — org-node: Simplicity 4, "the same helper … one copy stays" — org-node/tests/own_removal.rs:442; provisional_store.rs:40-49 vs receive_path.rs:452-458; store_at_rest.rs:270 vs verification.rs:84 vs support/mod.rs:785
- **What:**
  - own_removal.rs:442 builds members inline although `others(count)` (:489) already does this.
  - `b_with_filler` and `information_for_b` build the same thing.
  - Three builders exist for "Organisation information for the next epoch admitting one member".
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed (Simplicity 4 duplicates, not selected), `docs/plans/2026-10-21-org-io-domain-api.md`; the builders stay in `org-node/tests/own_removal.rs`, `org-node/tests/provisional_store.rs` and `org-node/tests/receive_path.rs`; the information builder is re-reported as round 9 low 10 and carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-32**: guideline, low — org-node: Simplicity 4, "one copy stays, where both can reach it" — org-node/tests/service_lifecycle.rs:139; sealed_store.rs:53, 58, 528, 585; verification.rs:213-216, 216, 296-298, 298, 943
- **What:**
  - The Organisation public key is re-derived by hand in places where `support::fixtures::public_key` exists.
  - The proxy is hand-written as `[0x5a;32]`, which is `support::the_proxy()`, and verification.rs adds a second proxy value, `[7;32]`, written three times.
  - The founding sequence is written out inline, repeating `stores::founding()`, `found_on_chain` and `sweep::founded_at`.
disposition: not fixed — the plan's round 6 fix (task `n6`) lists it among the CODE lows not fixed (Simplicity 4 duplicates, not selected), `docs/plans/2026-10-21-org-io-domain-api.md`; task `n7` later made `stores::founding` record `support::the_proxy()` (`a_founding_store_adopts_its_founding_at_epoch_one_with_its_proxy` in `org-node/tests/verification.rs`); the rest is carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-33**: guideline, low — org-node: Names 4, "One name per concept across the change" — org-node/tests/support/mod.rs:580, 585, 605-606, 615, 635, 644, 664-665, 690, 700; support/sweep.rs:76, 87; service_lifecycle.rs:193-194
- **What:**
  - The Persona identifier is `pid` here and `persona_id` in the neighbouring helpers.
  - The device key pair is `device_kp` and also `device_keypair`.
  - The current record is read through `rec_of` in one module and `record_of` in another, and revocation.rs:37 uses `record_of` for a builder.
  - A Wire message is `msg` here and `message` elsewhere.
  - `Side::clone_store` only calls `Node::store_on_disk`, and its doc describes a different operation.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): the Persona-identifier, key-pair, record-reader and message names are made one per concept, and `Side::clone_store`'s doc is corrected, in `org-node/tests/support/mod.rs` and `org-node/tests/support/sweep.rs`.

**finding-34**: guideline, low — org-node: Names 4, "One name per concept across the change" — org-node/tests/support/fixtures.rs:141; verification.rs:45, 50, 57-77, 258-382 vs 851-891
- **What:**
  - `invite(byte)` returns an `InviteId`, while `Node::invite` and `sweep::invite` return an `Invite`.
  - `stores::record` and `stores::ada_store` redefine the glob-imported fixtures of the same name with different member order.
  - `key` names an Organisation public key in some tests and a DevicePublicKey in others of the same file.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): `invite(byte)`, the redefined `stores::record`/`stores::ada_store` and the two meanings of `key` are renamed, in `org-node/tests/support/fixtures.rs` and `org-node/tests/verification.rs`.

**finding-35**: guideline, low — org-node: Names 4, "where the codebase already names it, that name" — org-node/tests/store_at_rest.rs:296, 306
- **What:** the literal seeds 0xa1 and 0xb1 are written where `ADA` and `BOB` exist, in a file whose `mod adopted` already imports them.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): the literal seeds 0xa1 and 0xb1 are written as `ADA` and `BOB` in `org-node/tests/store_at_rest.rs`.

**finding-36**: guideline, low — org-node: Comments 1, "A comment says why, not what" — org-node/tests/organisation_key.rs:58, org-node/tests/verify_against_chain.rs:179
- **What:** `// small order` and `// ignored` restate the code and give no reason.
disposition: fixed in task `n6` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 6 fix — CODE org-node, CODE lows fixed): the `// small order` and `// ignored` comments that restated the code are gone from `org-node/tests/organisation_key.rs` and the other cited file.

### Round 7

**finding-1**: guideline, medium — org-node: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it" — org-node/tests/support/fixtures.rs:1-4; org-node/tests/revocation.rs:37-50, 204-214, 336-349; org-node/tests/secret_redaction.rs:83-97, 100-113, 396-418; org-node/tests/store_at_rest.rs:28-40, 55, 67-80, 181, 213-235; org-node/tests/sealed_store.rs:429-440
- The change adds `support/fixtures.rs` and states its rule there: "Every target that builds a record, a Member snapshot or a one-store service from parts uses these, so the shape of each is stated once."
- These targets still build `OrgRecord`s, Personas and one-store services by hand:
  - `revocation.rs`: `record_at` and the blocks at 204-214 and 336-349.
  - `secret_redaction.rs`, `store_at_rest.rs` and `sealed_store.rs`: the sites listed above.
- `sealed_store.rs:429-440` (`admin()`) and `secret_redaction.rs:402-406` repeat the same Persona-binding block, and each defines `const OWN: u8 = 4`.
- So the rule the change states is not met, and the fixtures doc misleads its reader. Either the fixtures take the parameters these sites need, or the doc stops claiming "every target".
disposition: fixed in task `n7` (C7-N1, a recurrence of C5-N14): owner ruling 2026-10-29, one targeted fix. After one grep inventory of every hand-built record, Member snapshot, Persona and one-store service in org-node/tests, `org-node/tests/support/fixtures.rs` gained `record_parts`, `persona_parts`, `OWN`, `alice()`, `own()`, `alice_persona()`, `acme_store`, `service_under`, `own_service` and `own_encoded`, and each listed site is a fixture plus the fields it varies; the hand-built values that remain each say why in a comment, and the fixtures header states the rule and the exception (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-2**: guideline, low — org-node: Simplicity 4, "one copy stays, where both can reach it" — per-file copies of shared fixtures and helpers that `support` could hold once:
- org-node/tests/epoch_store.rs:26, 33-45 (`DEVICE`, `persona`) are the same as org-node/tests/persona_records.rs:31, 57-69: Alice Smith, "p1", seed 0x11.
- epoch_store.rs:102-114 and org-node/tests/invitation.rs:257-269 have the same `kept_request`.
- org-node/tests/own_removal.rs:29-33, 114-120 and org-node/tests/organisation_request.rs:22, 48-58 have the same `OWN`, `own()`, `service()` and `encoded()`. Also, `admin_data()` at organisation_request.rs:36 is the same as `member_data()` at own_removal.rs:52.
- org-node/tests/outgoing_bodies.rs:62 `bodies::own()` and org-node/tests/proposals.rs:579 `changes::own()` are the same.
- organisation_request.rs:100-107 and 367-374 write the same `Invite` literal, although proposals.rs:319 `invite_to` exists.
- `verification.rs` and `sealed_store.rs`:
  - verification.rs:798-808 `rust_files` is the same as sealed_store.rs:238-247 `rust_sources`.
  - verification.rs:235-240 and 296-299 repeat `stores::founding()` (213-219).
  - sealed_store.rs:58 and verification.rs:216, 298, 943 write `ChainAccount::new(..)` literals beside `support::the_proxy()`.
- Hand-written copies of `support` helpers:
  - org-node/tests/receive_path.rs:71-74 scans signatures by hand beside `support::public_signatures`.
  - org-node/tests/value_types.rs:140-142 `some_device` has the same body as `support::relay_device`.
  - org-node/tests/secret_redaction.rs:472, 489, 491 use `body.windows(32).any(..)` beside `support::contains`.
- Hex encode/decode is written several times:
  - encoding_golden.rs:79-89 and key_custody.rs:21-27 are two decoders.
  - The inline `format!("{byte:02x}")` rendering is repeated at node_value_types.rs:191, organisation_key.rs:44, organisation_request.rs:157 and outgoing_bodies.rs:243.
- The 129-character over-long name is rebuilt:
  - persona_records.rs:591 rebuilds it beside `too_long()` at 380.
  - proposals.rs:134, 223 and 274 build it three times.
  - person_store.rs:50 and 120 build it under two names.
disposition: partly fixed in task `n7`, in the files the C7-N1 fix edited only (`docs/plans/2026-10-21-org-io-domain-api.md`, round 7 fix); task `n8` then replaced the local `own()` copies (C8-N1), `kept_request`, the Invite literal, the src walk and the signature scans with shared helpers in `org-node/tests/support/fixtures.rs` and `org-node/tests/support/mod.rs`; the remainder is carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-3**: guideline, low — org-node: Simplicity 4, "the same helper, constant or check" — the same check is asserted twice:
- org-node/tests/organisation_key.rs:143 and proposals.rs:102, 114 assert LLR-9qavm7's `DuplicateOrgPublicKey` refusal (no store added).
- proposals.rs:273 is a subset of the cases at :133 and :222.
- proposals.rs:865 repeats the `MemberNotHeld` case at :835.
- org-node/tests/revocation.rs:268-285 and 292-311 make nearly identical `accept` refusal-order assertions.
- revocation.rs:154-163 and 591-604 both pin the `ACK_DOMAIN` signed bytes.
- receive_path.rs:83-85 and 97-99 run the same undecodable-body loop.
- wire_frame_bound.rs checks `body[0]` against the index and the round trip three times (144-156, 211-217, 243-247), and checks `org_id()` at both 154 and 304-307.
- value_types.rs:109-116 is covered by the exact-text test at 243-247.
disposition: not fixed — the plan's round 7 fix (task `n7`) lists it among the lows not fixed: the duplicated checks sit in tests the plan's red -> green records name, so which test keeps each check is the TEST review's decision, `docs/plans/2026-10-21-org-io-domain-api.md`; re-reported as round 9 low 13 and carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'). The site is e.g. `org-node/tests/proposals.rs`.

**finding-4**: guideline, low — org-node: Simplicity 2, "No parameter, flag, setting or hook without a caller that uses it now" (and Simplicity 3, "an unused function, import or variable … deleted") — org-node/src/invitation.rs:17
- `invitation` re-exports `MAX_ENDPOINT_ADDRESSES` and `MAX_ENDPOINT_ADDRESS_BYTES`.
- Nothing in org-node, org-io or app imports them through `invitation`. Their only uses are inside `ids.rs`, where they are defined, and `ids` is already `pub`.
disposition: fixed in task `n7` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 7 fix, lows fixed): the unused re-export of `MAX_ENDPOINT_ADDRESSES` and `MAX_ENDPOINT_ADDRESS_BYTES` is gone from `org-node/src/invitation.rs`.

**finding-5**: guideline, low — org-node: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-node/src/service.rs:1170
- `admission_requests` gives the empty string `""` as the invitee name when a kept request has no outstanding invite.
- The doc (1155-1156) says this "is never returned in practice", but no requirement names `""` as the default.
- An inconsistent store would therefore read as a request with an empty invitee name, not as an error.
disposition: not fixed — the plan's round 7 fix (task `n7`) lists it among the lows not fixed (src), `docs/plans/2026-10-21-org-io-domain-api.md`; the `map_or("", …)` stays in `org-node/src/service.rs`; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-6**: guideline, low — org-node: Errors 3, "A missing … value is an error unless a requirement names the default" — org-node/src/service.rs:543
- `base_root: base_root.unwrap_or(root)` stores the answer's own root as `base_root` for a store with no record.
- The comment explains that no adoption reads it, but no requirement names that stand-in, and the field then holds a value that is not a base root.
disposition: not fixed — the plan's round 7 fix (task `n7`) lists it among the lows not fixed (src), `docs/plans/2026-10-21-org-io-domain-api.md`; `base_root.unwrap_or(root)` stays in `org-node/src/service.rs`; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-7**: guideline, low — org-node: Errors 1, "An error is reported where it is detected, naming what failed" — org-node/src/service.rs:1354
- `remove_own` refuses a store with no record as `OrgNotOnChain`.
- Every other no-record refusal in the file goes through `current_record` (`OrgNotHeld` / `FoundingNotWritten`).
- The chain state is present at that point, so the error names the wrong failure. The case is unreachable in practice, which is why this is low.
disposition: not fixed — the plan's round 7 fix (task `n7`) lists it among the lows not fixed (src), `docs/plans/2026-10-21-org-io-domain-api.md`; `remove_own` still refuses as `OrgNotOnChain` in `org-node/src/service.rs`; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-8**: guideline, low — org-node: Errors 1, "naming what failed and on what input" — org-node/src/error.rs:68
- The doc of `ProvisionalLimit` (65-67) covers provisional changes, provisional organisations and revocation notices, and `ensure_within_bound` measures all three.
- The `#[error]` text says "provisional changes and organisations" only. A keep refused by the size of its kept notices is therefore reported as something else.
disposition: not fixed — the plan's round 7 fix (task `n7`) lists it among the lows not fixed (src), `docs/plans/2026-10-21-org-io-domain-api.md`; the `ProvisionalLimit` text in `org-node/src/error.rs` is unchanged; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-9**: guideline, low — org-node: Errors 2, "Where an error is expected and handled, the handling says why that is safe" — the returned `#[must_use]` `StoreChange` is discarded with no reason given:
- Sites, bound to `_sealed`, `_` or `_none`:
  - org-node/tests/verification.rs:236, 293, 297, 390, 445, 498, 535, 560, 677, 688, 778
  - org-node/tests/store_at_rest.rs:298
  - org-node/tests/receive_path.rs:553, 667
  - own_removal.rs:793 (bare `.unwrap()`)
- The suite has `support::unread`, which states the reason.
- `_none` at verification.rs:293 and 390 also claims `None` without asserting it.
disposition: fixed in task `n7` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 7 fix, lows fixed): every dropped `StoreChange` goes through `support::unread` (`org-node/tests/support/mod.rs`), which states the reason.

**finding-10**: guideline, low — org-node: Errors 1, "naming what failed and on what input" — receive_path.rs:472, 502; wire_frame_bound.rs:285
- A fixture destructure falls through to a bare `unreachable!()`.
- receive_path.rs:639 gives a message for the same pattern.
disposition: fixed in task `n7` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 7 fix, lows fixed): every `unreachable!` and `panic!` carries a message, in `org-node/tests/receive_path.rs` and `org-node/tests/wire_frame_bound.rs`.

**finding-11**: guideline, low — org-node: Names 4, "A concept has one name in every file, test and task of the change" — test files:
- **An unlisted Device** is `rogue()` (receive_path.rs:25), `stranger` (receive_path.rs:155, receive_chain_reads.rs:76), `unrelated` (receive_chain_reads.rs:104), `relay_device()` (support/mod.rs:500) and `some_device` (value_types.rs:140).
- **The founder `Side`:**
  - It is `admin` (receive_chain_reads.rs:113, 140, 167; service_stories.rs:30), `founder` (receive_chain_reads.rs:103, service_lifecycle.rs:90, verification.rs:1098, 1128) and `inviter` (service_lifecycle.rs:59).
  - An `Admission` is `pending`, `admission` or `bob` (receive_chain_reads.rs:114, 141; verification.rs:1100, 1129).
  - admission_sender.rs uses `bob`/`b_id`/"B's" (315-317, 353-355) beside `persona_b`, `persona_a` and `bob_store` (64, 137, 293).
- **`org_key`** is an `OrgPublicKey` in epoch_store.rs:28 and an `(OrgPrivateKey, OrgPublicKey)` pair in own_removal.rs:37. src names the public key `org_pub_key`.
- **`record()`** at epoch_store.rs:47 is a fixed record, while `support::fixtures::record(epoch, members)` takes parts. invitation.rs:249 imports the latter.
- **`chain`** is an `OrgState` in fixtures.rs:136, sweep.rs and verification.rs:1052, and a `ChainSlots` in support/mod.rs:288, 605, 660. verification.rs elsewhere names the `OrgState` `state`.
- **`key`** in verification.rs is a `DevicePublicKey` (70, 110, 192, 198, 215, 851ff.), private key bytes (88) and an `OrgPrivateKey` (1108).
- **One name for two values in a file:**
  - In sealed_store.rs, `alice` is a `DevicePublicKey` at 324 and a `PersonaId` at 361.
  - In receive_path.rs, `bob` is a `MemberSnapshot` at 697 and a Device key at 736-737 and 811-813, beside `a_device`/`b_device` elsewhere.
- **Store changes:** in verification.rs the store change is `change`, `sealed`, `written` and `_sealed`.
- **Services:** store_at_rest.rs has `service` at 55 and 181 against `svc` at 273 and 296.
- **Smaller drifts:**
  - organisation_key.rs names the Organisation private key `secret` at 38 and `key` at 165 and 192.
  - In proposals.rs, Bob's MemberId is `changes::bob().id` at 779 and the literal `MemberId::new([0x02; 32])` at 662, 820, 837, 888, 963, 1025 and 1110.
  - persona_records.rs:391 names a `StoreData` `content`, which `StoreContent` names elsewhere.
  - invitation.rs:153 and 171 bind `org` where `founder` binds `org_id`, and invitation.rs:43 has the parameter `invitee` for the field `invitee_name`.
  - `org` holds an `OrgId` at service_lifecycle.rs:194 and receive_path.rs:208 and 216. At receive_path.rs:208 and 216, `invite` also holds an `InviteId`, while `support` uses `invite` for an `Invite`.
  - support/mod.rs has `party` at 128 and 135 against `party_name` at 140 and 147, and `handle()` at 508 against `parsed_name()`/`parsed_surname()` at 512 and 516.
disposition: partly fixed in task `n7`, in the files the C7-N1 fix edited only (`docs/plans/2026-10-21-org-io-domain-api.md`, round 7 fix); the remainder is carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'). The site is e.g. `org-node/tests/verification.rs`.

**finding-12**: guideline, low — org-node: Names 3, "A name says what the thing is in the project's domain … no `data`, `info`, `item`, `value`, `result` … alone" — test files:
- own_removal.rs:732 `for (field, value) in before`.
- revocation.rs:168 `value: &T`.
- invitation.rs:108 `pick` holds "this field is valid yet".
- support/mod.rs:674 `prepare: bool` means "B proposes to join A's Invite".
- fixtures.rs:185 `per`.
- receive_path.rs:762 `another_member` holds a `RevocationNotice`.
- person_store.rs:110-111 `device` and `person` are key bytes, while `person` at 153 is a `PersonStore`.
- provisional_store.rs:23, 40, 84, 98, 106 use `bytes` for a length.
disposition: partly fixed in task `n7`, in the files the C7-N1 fix edited only (`docs/plans/2026-10-21-org-io-domain-api.md`, round 7 fix); task `n8` later renamed `per` to `bytes_per_revoke` in `org-node/tests/support/fixtures.rs` (round 8 low 11); the remainder is carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-13**: guideline, low — org-node: Simplicity 3, "an unused function, import or variable … deleted" — redundant imports:
- proposals.rs:716, 836 and 952 repeat `use org_node::error::OrgNodeError;` inside test bodies, although line 11 already imports it.
- receive_chain_reads.rs:18 `use support::sweep;` repeats what the glob at line 16 brings in.
disposition: fixed in task `n7` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 7 fix, lows fixed): the duplicate imports are gone from `org-node/tests/proposals.rs` and `org-node/tests/receive_chain_reads.rs`.

### Round 8

**finding-1**: guideline, medium — org-node: Names 4, "A concept has one name in every file, test and task of the change" — org-node/tests/outgoing_bodies.rs:61-63, org-node/tests/proposals.rs:579-581 (against org-node/tests/support/fixtures.rs:64-67)
The shared fixture `support::fixtures::own()` is Alice's Device (`device_key(OWN)`), used 66 times
in own_removal.rs. outgoing_bodies.rs and proposals.rs each glob-import `support::fixtures::*`
(outgoing_bodies.rs:29, proposals.rs:543) and then define a local `pub fn own()` returning Ada's
Device (`device_key(ADA)`), shadowing the shared one silently. One name, two Devices, across ~72
call sites: a reader who knows `own()` from fixtures misreads every one of them. The Ada copy is
also written twice (Simplicity 4).
disposition: fixed in task `n8` (C8-N1): one shared `fixtures::ada_device()` in `org-node/tests/support/fixtures.rs` replaces the local `own()` in `org-node/tests/outgoing_bodies.rs` and `org-node/tests/proposals.rs`; a grep comparing every public support/fixtures item name with the items each glob-importing file defines found only those sites before the fix and none after, `docs/plans/2026-10-21-org-io-domain-api.md`.

**finding-2**: guideline, low — org-node: Simplicity 3, "an unused function ... [is] deleted" — org-node/tests/absences.rs:51
`pub fn assert_absent_in(suffix, needle, why)` has no caller left (grep over tests/ finds only the
definition). Its callers on master were the two tests this change rewrote; being `pub` it raises no
dead-code warning.
disposition: fixed in task `n8` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 8 fix, lows fixed): the dead `assert_absent_in` is deleted from `org-node/tests/absences.rs`.

**finding-3**: guideline, low — org-node: Simplicity 3, "an unused function, import or variable ... [is] deleted" — org-node/tests/admission_sender.rs:24
`use support::sweep;` repeats what `use support::*;` (line 16) already brings in
(support/mod.rs:28 `pub mod sweep`). Same pattern round 7 removed from proposals.rs and
receive_chain_reads.rs.
disposition: fixed in task `n8` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 8 fix, lows fixed): the redundant `use support::sweep;` is removed from `org-node/tests/admission_sender.rs`.

**finding-4**: guideline, low — org-node: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it" — org-node/tests/invitation.rs:360, :391
`let own = device_key(OWN);` re-derives `support::fixtures::own()` (fixtures.rs:65) in a module
that already imports from `support::fixtures` (line 233).
disposition: fixed in task `n8` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 8 fix, lows fixed): `org-node/tests/invitation.rs` uses `own()` from the fixtures.

**finding-5**: guideline, low — org-node: Simplicity 4 (as finding-4) — org-node/tests/epoch_store.rs:94, org-node/tests/invitation.rs:250
Two new `kept_request` builders of `KeptAdmissionRequest`; the parametrised one in invitation.rs
(or one in support::fixtures) serves both. Values differ only in names and addresses.
disposition: fixed in task `n8` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 8 fix, lows fixed): one `fixtures::kept_request` in `org-node/tests/support/fixtures.rs` serves both files.

**finding-6**: guideline, low — org-node: Simplicity 4 (as finding-4) and Names 4 — org-node/tests/invitation.rs:239 (`invite()`), org-node/tests/organisation_request.rs:45 (`invite_to_bob()`)
Both new helpers build the same Invite (Bob to `the_org`, issuer Device 5, `InviteId::new([3; 32])`),
differing only in invitee name text and address list: one concept, two helpers, two names.
disposition: fixed in task `n8` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 8 fix, lows fixed): one `fixtures::invite_to_bob` in `org-node/tests/support/fixtures.rs` replaces both helpers.

**finding-7**: guideline, low — org-node: Simplicity 4 (as finding-4) — org-node/tests/outgoing_bodies.rs:89-102, org-node/tests/verification.rs:187-193
Two near-identical `provisional_member()` builders (Carol's provisional Member store from Ada's
Invite, same `propose_member` call, same return type), differing only in `issuer_addresses`. One
copy in support::fixtures taking the addresses would do.
disposition: fixed in task `n8` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 8 fix, lows fixed): `fixtures::provisional_member` and `invite_to_carol` moved into `org-node/tests/support/fixtures.rs`.

**finding-8**: guideline, low — org-node: Simplicity 4 (as finding-4) — org-node/tests/revocation.rs:597-603
A new hand-written walk of org-node's src/ (`read_dir`, extension filter, `read_to_string`) beside
`support::src_rust_files()` (support/mod.rs:425-441), which the same change adds and which
revocation.rs can reach (`mod support;` line 8). The copy also skips subdirectories, so the two can
drift.
disposition: fixed in task `n8` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 8 fix, lows fixed): `org-node/tests/revocation.rs` uses `support::src_rust_files()`.

**finding-9**: guideline, low — org-node: Simplicity 4 (as finding-4) — org-node/tests/proposals.rs:79-81, org-node/tests/receive_path.rs:71-73
The same signature-slicing lines (`source.find("pub fn …<")` then slice to the first `{`) added
twice, beside `support::public_signatures(file)` (support/mod.rs:389-421) added by this change for
exactly that.
disposition: fixed in task `n8` for `org-node/tests/proposals.rs` and `org-node/tests/receive_path.rs`, which use `support::public_signatures`; not fixed at key_custody.rs, because `public_signatures` cuts at the `[u8; 32]` in that signature, `docs/plans/2026-10-21-org-io-domain-api.md`; that part is carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-10**: guideline, low — org-node: Simplicity 4 (as finding-4) — org-node/tests/organisation_request.rs:145-150
A hand-built "key bytes absent from Debug" check (`key_hex` by `{byte:02x}`, `key_decimal` by
`{:?}` trimmed) beside secret_redaction.rs's `renderings(secret)` helper, changed in this diff for
that check. The hex one-liner also recurs at organisation_key.rs:44 and node_value_types.rs:191,
and encoding_golden.rs:79 has its own `hex()`.
disposition: fixed in task `n8` where support reaches: `support::hex`, `squash` and `assert_not_rendered` moved from secret_redaction.rs into `org-node/tests/support/mod.rs`, and `organisation_request.rs` uses `assert_not_rendered`; not fixed for `org-node/tests/node_value_types.rs` and `org-node/tests/encoding_golden.rs`, whose feature gates exclude support, `docs/plans/2026-10-21-org-io-domain-api.md`; that part is carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-11**: guideline, low — org-node: Names 3, "A name says what the thing is in the project's domain" — org-node/tests/support/fixtures.rs:264
`let per = bound_len_with(1) - bound_len_with(0);` holds the encoded bytes of one padding change;
`per` says nothing of that (e.g. `bytes_per_revoke`).
disposition: fixed in task `n8` (`docs/plans/2026-10-21-org-io-domain-api.md`, round 8 fix, lows fixed): `per` is now `bytes_per_revoke` in `org-node/tests/support/fixtures.rs`.

**finding-12**: guideline, low — org-node: Errors 1, "An error is reported where it is detected, naming what failed" — org-node/src/service.rs:1354, org-node/src/removal.rs:132
A store with no record is refused as `OrgNotOnChain` ("no on-chain state found for org") in
`remove_own` (service.rs:1354), and a store bound to no Organisation as `OrgNotOnChain` in
`StoreData::into_removed` (removal.rs:132). Neither failure is about chain state, and
`service::current_record` (service.rs:1417-1423) refuses the very same states as `OrgNotHeld` /
`FoundingNotWritten`. Low: both callers reach these only with a record held.
disposition: not fixed — the plan's round 8 fix (task `n8`) lists it among the lows not fixed (src), `docs/plans/2026-10-21-org-io-domain-api.md`; `OrgNotOnChain` stays in `org-node/src/service.rs` and `org-node/src/removal.rs`; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-13**: guideline, low — org-node: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-node/src/service.rs:1170
`admission_requests` returns `""` as the invitee name when no outstanding invite matches a kept
request (`.map_or("", ...)`). The doc argues the case cannot arise, which makes it rare, but if the
invariant breaks the caller gets an empty name that looks like a real one instead of an error.
disposition: not fixed — the plan's round 8 fix (task `n8`) lists it among the lows not fixed (src), `docs/plans/2026-10-21-org-io-domain-api.md`; the `map_or("", …)` stays in `org-node/src/service.rs`; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

**finding-14**: guideline, low — org-node: Errors 3 (as finding-13) — org-node/src/service.rs:543
In a store with no record, an organisation answer's `base_root` is filled with the answer's own
`root` (`base_root.unwrap_or(root)`). The comment gives the reason (no adoption reads an Answer's
`base_root`), but no requirement names this default and the stored value states a false base.
Low: no reader of the field takes it today.
disposition: not fixed — the plan's round 8 fix (task `n8`) lists it among the lows not fixed (src), `docs/plans/2026-10-21-org-io-domain-api.md`; `base_root.unwrap_or(root)` stays in `org-node/src/service.rs`; carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking').

### Round 9

**finding-1**: guideline, low — org-node: Errors 1, "An error is reported where it is detected, naming what failed" — org-node/src/store.rs:428, org-node/src/store.rs:1374, org-node/src/person_store.rs:140
New encode failures (epoch-state bound measure, removed-store seal, Person store seal) are reported as `OrgNodeError::Chain(..)`, whose Display is "chain read failed: {0}". The message says a chain read failed when an in-memory postcard encode failed; the variant names the wrong failure. Low: postcard encoding of these values fails only in a contrived state.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-node/src/store.rs` and `org-node/src/person_store.rs`. The round 9 section names this one among the noted open lows: 'OrgNodeError::Chain carries local encode failures, which org-node did before this change'.

**finding-2**: guideline, low — org-node: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-node/src/service.rs:1170
`admission_requests` pairs each kept request with `.map_or("", ..)`: a kept request with no outstanding invite gets an empty invitee name, silently. The doc says it "is never returned in practice", which is the case for an error, not a default. Low: needs a store whose invariant is already broken.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-node/src/service.rs`.

**finding-3**: guideline, low — org-node: Errors 3, "A missing file, setting or value is an error unless a requirement names the default" — org-node/src/service.rs:543
`keep_organisation_answer` stores `base_root: base_root.unwrap_or(root)` for a record-less store: the answer's own root stands in for a base that does not exist. The comment justifies it by "no adoption reads base_root of an Answer", but no requirement names this default and the field then holds a value that means something else (`ProvisionalOrganisationView` callers and any later reader see a base root equal to the root). Low.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-node/src/service.rs`.

**finding-4**: guideline, low — org-node: Errors 3, "A missing ... value is an error unless a requirement names the default" — org-node/src/service.rs:270
`HANDLE_FIELDS.get(index).copied().unwrap_or("member.preferred_handles")`: a fourth or later handle that fails to parse is reported under a fallback field name instead of its own index; the count refusal comes only after. Low.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-node/src/service.rs`.

**finding-5**: guideline, low — org-node: Simplicity 3, "Code no path reaches, an unused function, import or variable ... are deleted" — org-node/src/wire.rs:19, org-node/src/wire.rs:25
`ORG_INFORMATION_INDEX` and `ORGANISATION_INDEX` are read only by `service.rs` (gated on `app`); `wire` is not gated, so a build without `app` warns both constants are never used. Gate them with their only reader, or make them private to the place that reads them.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-node/src/wire.rs`.

**finding-6**: guideline, low — org-node: Names 4, "A concept has one name in every file" — org-node/src/store.rs:1199, :1203-1213, :1229
`StoreCipher` is declared "the one name for the library's type outside this module", yet `labelled_cipher`, `cipher` and `open_sealed`, all called from person_store.rs, still return `XChaCha20Poly1305`. Readers of person_store see both names for the one cipher.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-node/src/store.rs`.

**finding-7**: guideline, low — org-node: Names 1, "Every name the change writes ... says what it holds" — org-node/tests/person_store.rs:178 (`same<T>`), org-node/tests/support/mod.rs:211, :227 (`<R: RngCore + CryptoRng>`), :362 (`held_store<'s>`), org-node/tests/revocation.rs:162 (`postcard_of<T>`)
One-letter type parameters and lifetimes written by the change; they are chosen, not fixed by the language. The same tests name other generics (`CloneProbe<Probed>`, `model<Encodable>`).
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-node/tests/person_store.rs`, `org-node/tests/support/mod.rs` and `org-node/tests/revocation.rs`.

**finding-8**: guideline, low — org-node: Names 3, "A name says what the thing is in the project's domain, not its type or a role in general" — org-node/tests/outgoing_bodies.rs:73, org-node/tests/organisation_request.rs:107, :206, org-node/tests/admission_sender.rs:417, :453
`held` names five different things (a provisional organisation, store data, a record, an Organisation private key, a record); at outgoing_bodies.rs:73 it also shadows the helper `held(..)` it is computed from.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-node/tests/outgoing_bodies.rs`, `org-node/tests/organisation_request.rs` and `org-node/tests/admission_sender.rs`.

**finding-9**: guideline, low — org-node: Names 4, "A concept has one name in every file, test and task of the change" — org-node/tests/store_at_rest.rs:8-12
The added module comment calls the Device secret "the wrong passphrase", while sealed_store.rs:240-250 asserts the passphrase concept is gone and the code names it the Device secret.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-node/tests/store_at_rest.rs`.

**finding-10**: guideline, low — org-node: Simplicity 4, "Where two parts of one change add the same helper, constant or check, one copy stays, where both can reach it" — test fixtures rebuilt beside support/fixtures.rs:
- org-node/tests/outgoing_bodies.rs:45-55 builds Carol's `KeptAdmissionRequest` field by field; equals `support::fixtures::kept_request(invite_id(), 0x63, CAROL, "carol")` but for an address no test reads.
- org-node/tests/verification.rs:115-140 (`stores::request`, `stores::admin_store`) and org-node/tests/proposals.rs:547-575 (`changes::request`, `changes::outstanding`, `changes::admin_store`) build the same Ada-at-epoch-3 store with Carol's and Dave's requests; their follow-ons verification.rs:145-152 `own_proposal` and proposals.rs:639-644 `changes::proposed` are the same too; both overlap `fixtures::kept_request` (fixtures.rs:267-279).
- Organisation information after one admission is built three times: support/mod.rs:849-870 `information_after`, verification.rs:82-98 `stores::information`, store_at_rest.rs `adopted::setup` (about 233-262).
- `WireMessage::AdmissionRequest` is spelled out literally at receive_path.rs:50-61, sealed_store.rs:419-432, secret_redaction.rs:297-306, wire_frame_bound.rs:52-63; Invite builders proposals.rs:318-327 `invite_to` and sealed_store.rs:478-485 restate `fixtures::invite_to_bob` (fixtures.rs:231-240).
- The "no curve point" constant (`no_point[0] = 2`, same comment) at own_removal.rs:173-175 and person_store.rs:121-123 (also revocation.rs:555).
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is the fixtures beside `org-node/tests/support/fixtures.rs`.

**finding-11**: guideline, low — org-node: Simplicity 4, "one copy stays, where both can reach it" — support helpers rewritten inline in files that declare `mod support`:
- org-node/tests/service_lifecycle.rs:37 `hex16` = `support::hex(&bytes[..16])` (support/mod.rs:519).
- org-node/tests/proposals.rs:91-92 `windows(32).any(..)` = `support::contains` (support/mod.rs:514).
- org-node/tests/secret_redaction.rs:393 hand probe redoing `support::assert_not_rendered` (support/mod.rs:556), used elsewhere in the same file.
- org-node/tests/receive_path.rs:24-27 `rogue()` = `support::relay_device()` (support/mod.rs:563-566).
- org-node/tests/verification.rs:785-805 `whole_word`/`ident` closures re-implement absences.rs:476-488, and the test repeats the LLR-ds228z scan absences.rs:437-451 already runs; revocation.rs:598-601 re-implements absences.rs `code_of` (461-469); support/mod.rs:524 `squash` = absences.rs:472 `squeezed`.
(Hex encoders in encoding_golden.rs:79-81 and node_value_types.rs:191 duplicate `support::hex` too, but those targets cannot reach `support` under their features; noted, not counted.)
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is the inline copies beside `org-node/tests/support/mod.rs`.

**finding-12**: guideline, low — org-node: Simplicity 4, "the same helper, constant or check" — org-node/tests/support/mod.rs:473-476 vs org-node/tests/verification.rs:72, :222, :281
`story_time()` is documented as "The time every verification of these tests is made at", but verification.rs uses its own `VerificationTime(1_760_000_000)` and `VerificationTime(1)`: the constant is stated three ways and the shared helper's doc is false for the change's own tests.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is `org-node/tests/support/mod.rs` and `org-node/tests/verification.rs`.

**finding-13**: guideline, low — org-node: Simplicity 4, "the same ... check" — duplicate checks within the change's tests:
- Acknowledgement outcomes (`Acknowledged`, `AcknowledgementNotFromItsDevice`, `AcknowledgementForListedDevice`): receive_chain_reads.rs:29-57, receive_path.rs:368-399, receive_path.rs:813-834.
- Unlisted sender is `SenderNotListed` and changes nothing (LLR-283x6f): receive_chain_reads.rs:66-78, receive_path.rs:116, :335-344, :408-417; the same three undecodable bodies at receive_path.rs:82-84 and :96-98.
- revocation.rs:581-591 repeats revocation.rs:148-157 (same bytes, LLR-bs6yxf).
- store_at_rest.rs about 264-285 (LLR-p8vpgz) repeats verification.rs:619-628 on the same fixture.
- Wire kind round-trip with index three times: wire_frame_bound.rs:144-156 (still named for "three kinds"), :211-237, :243-260; the `org_id()` assert at :304-312 repeats :154; value_types.rs:109-121 and :243-250 both pin the `ProvisionalLimit` message.
disposition: not fixed — carried open, not blocking (the plan's round 9 section: 'Open lows, carried and not blocking'; the round 9 CODE org-node review had nothing above low, so its loop is done), `docs/plans/2026-10-21-org-io-domain-api.md`; the site is the duplicate checks in `org-node/tests/receive_path.rs` and the other files listed.

### .guardrails/templates/ARCHITECTURE_GUIDELINES.md

Governs: the change's code and architecture documents in on-chain-client, org-node, org-io and app, plus the root docs/CONTEXT.md and the change's ADRs (the reports state their scope as every changed path in app, on-chain-client, org-io, org-node and root docs/, checked against each unit's design draft, `.guardrails/units.yaml` and each unit's `.guardrails/config.yaml`).

### Round 1

Round 1's report text survives only as the plan's summaries (`docs/plans/2026-10-21-org-io-domain-api.md`, "### Guideline reviews — round 1"); the finding text below is quoted from there, and for the lows A3 and A9, which the round summary does not word, from the fix record of task `arch`.

**finding-1**: guideline, medium — A1 — "A1: org-io production reads `list_orgs()` (`&OrgRecord`) and `PersonStore::data()`, values that carry key fields, instead of key-free views (LLR-8ru6mv)."
disposition: fixed in task `arch` (merged 2026-10-28): org-node gained the key-free views `OrgService::persona_records()` and `PersonStore::view()`, org-io reads org-node stores only through them, LLR-8ru6mv and LLR-5uf4u2 were amended and SDD-27p679 names `PersonView`; the new architecture test `org_io_reads_org_node_stores_only_through_key_free_views` (LLR-jqb5z7, org-io/tests/absences.rs) was mutation-checked red with a planted `service.list_orgs()`.

**finding-2**: guideline, medium — A2 — "A2: org-io's error enums carry `OrgNodeError`, and the app matches it exhaustively."
disposition: no change — owner ruling 2026-10-28 (task `rul`): `OrgNodeError` is part of org-io's public interface through `org_io::node`, and the app's exhaustive `classify_receive_error` match and the `InvalidField` match in commands.rs are a deliberate tripwire; this is recorded as an exception to ARCHITECTURE Boundaries 2 in a dated note under SDD-z3ychz in org-io/docs/architecture/2026-10-16-domain-api.md and on the app's LLR-wgr46u. Round 2's task `o2` stated the exception's limit (refusals org-io decides itself get org-io's own variants), round 4's task `o4` added the remaining carrier types to its list, and round 5 recorded every public org-io type carrying `OrgNodeError` as on the exception's list.

**finding-3**: guideline, low — A3 — "A3 partly fixed: `WriteRefused::source()` keeps the cause (LLR-48me7m). `ChainWriter`'s `String` errors are left, since typing them is not cheap (low)."
disposition: partly fixed in task `arch`: `WriteRefused::source()` keeps the cause (LLR-48me7m), proven by `a_write_refusal_keeps_its_cause_as_its_source` (org-io/tests/chain_write.rs), and the `ChainWriter` `String` errors were left as low; those were typed later under round 3's AR3-2 in task `o3` (`ChainWriteError` in org-io/src/write.rs, LLR-5tpdxw amended).

**finding-4**: guideline, medium — A4 — "A4: \"Device key\" in user-facing labels."
disposition: fixed in task `arch`: the user labels read "DevicePublicKey"; the rendered Admit panel test in app/tests/notices.admission.test.ts (LLR-e63yrs) gained a label assertion, mutation-checked red with the label set back to "Device key".

**finding-5**: guideline, medium — A5 — "A5: \"not executed\" is a glossary _Avoid_ synonym of Deprecated, used for a different outcome."
disposition: fixed by owner ruling 2026-10-28 (task `rul`, documentation only): docs/CONTEXT.md now defines **Not executed** (of a chain write) as a glossary term, takes it off Deprecated's _Avoid_ list and lists it among the Unknown write's settlements; LLR-zey4mg carries a dated note.

**finding-6**: guideline, medium — A6 — "A6: public terms missing from docs/CONTEXT.md (revocation notice, unknown write, removal acknowledgement, kept admission request)."
disposition: fixed in task `arch`: docs/CONTEXT.md defines Revocation notice, Unknown write, Removal acknowledgement and Kept admission request.

**finding-7**: guideline, medium — A7 — "A7: the removed-Persona deletion rule is written three times inline in IO code."
disposition: fixed in task `arch`: the deletion rule is one pure function, `retention::deletion_due` in the new org-io/src/retention.rs, used at open, at the resume and in the delivery loop; LLR-vvd8dn and LLR-k3zmqb were amended and `org_io_removes_nothing_itself_but_in_the_persona_deletion` was rewritten for retention.rs and mutation-checked.

**finding-8**: guideline, medium — A8 — "A8: SDD-83un8g states two responsibilities (Boundaries 3)."
disposition: fixed in task `arch`: SDD-83un8g is superseded by SDD-3my66m (acknowledgement delivery, org-io/src/acknowledge.rs) and SDD-wyn7b6 (retention and deletion, org-io/src/retention.rs), with the README table, soup.md and three scans updated.

**finding-9**: guideline, low — A9 — "A1 + A9 fixed: org-node gains two key-free views: `OrgService::persona_records()` (every `PersonaRecord` field but the member seed) and `PersonStore::view()` (the Person and its public keys, no Person seed)."
disposition: fixed in task `arch` together with A1, as the plan records ("A1 + A9 fixed"): the key-free `PersonStore::view()` in org-node/src/person_store.rs, LLR-5uf4u2 amended and SDD-27p679 naming `PersonView`; the inherited test `the_person_and_member_types_are_named_under_org_node_paths` gained `PersonView`.

### Round 2

**finding-1**: guideline, medium — Boundaries 3, "A software item has one responsibility, stated in one sentence without \"and\"" — app/docs/architecture/DRAFT-worktree-org-io-domain-api-domain-api.md:96-104 (SDD-6pjvhc; repeated at app/docs/architecture/README.md:73)
- SDD-6pjvhc is the IPC translation surface: parse, call org-io, map to a DTO, return the refusal text.
- It also holds the follow-up policy: "It decides when the app proposes, writes, verifies, sends and requests: verification follows an included write, an adopted epoch this node built is sent once … behind is requested …".
- That is two responsibilities, each with its own reason to change.
disposition: fixed as AR2-1 in task `a2` (merged 2026-10-28): the follow-up decision moved out of SDD-6pjvhc into the pure `follow_up.rs` (app/src-tauri/src/follow_up.rs) under the new SDD-6zqqm8 with LLR-292r26 and LLR-bbfgk7, and SDD-6pjvhc was reworded in place to the IPC translation surface alone (reworded again with one responsibility in round 3's task `a3`).

**finding-2**: guideline, medium — Core and edges 1, "Decisions, calculations, validation and state transitions are in code that reads and writes no file, network …" — app/src-tauri/src/commands.rs:782-809 (`send_and_request`) and :887-896 (`write_then_verify`)
- Two follow-up decisions are written inside the adapter code that makes the networked org-io calls (`send_organisation`, `request_organisation`, `verify_organisation`):
  - after an own adoption, send; after `behind`, request; otherwise nothing;
  - verify only after an `Included` write.
- No pure function holds them. The receiver side already has the split: `events::follow_up_for` is pure and `receive_iteration` executes it.
disposition: fixed as AR2-2 in task `a2`: the send-after-own-adoption, request-after-behind and verify-after-Included decisions are the pure function in app/src-tauri/src/follow_up.rs (SDD-6zqqm8, LLR-292r26, LLR-bbfgk7), watched fail first by `an_own_adoption_asks_a_send_and_behind_a_request_and_nothing_else_asks_anything` and `only_an_included_write_asks_a_verification` (app/src-tauri/tests/verification.rs).

**finding-3**: guideline, medium — Boundaries 1, "no reliance on its storage layout or file format" — app/src-tauri/src/events.rs:339-347 and :359 (`VerifiedKind::of`), with the producer at app/src-tauri/src/commands.rs:473
- The events item reads the commands item's typed `EntryResultDto` only through its serialised JSON: `serde_json::to_value(&entry)`, then a match on the strings `"result"` == `"adopted"`/`"removed"`/… and `"org_id"`.
- A renamed variant or serde tag on the commands side still compiles. The entry then becomes `Unknown`, which silently drops the `Updated`/`SelfDeleted` events.
disposition: fixed as AR2-3 in task `a2`: a typed hand-off in app/src-tauri/src/entry.rs replaces the JSON round trip, and `VerifiedKind::Unknown` is removed, so a renamed variant on the commands side no longer compiles silently into a dropped event (LLR-73by2q).

**finding-4**: guideline, medium — Boundaries 2, "An interface reports a failure in its own terms … it does not pass a dependency's error type through to its caller" — app/src/lib/founding.ts:8 and :38, with app/src-tauri/src/commands.rs:209-220 and :242
- `propose_person` returns org-io's `CreateError` Display text unchanged.
- The founding flow branches on that wording: `String(refusal).startsWith('a Person is held already')`.
- A rewording in org-io breaks the founding with no type error. Owner ruling A2 covers naming `OrgNodeError`, not depending on a dependency's message text.
disposition: fixed as AR2-4 in task `a2`: `propose_person` refuses with `{kind, message}` and app/src/lib/founding.ts branches on `kind`, not on org-io's text; LLR-k57uqv amended, watched fail first by `a_second_person_is_refused_by_its_kind_beside_org_ios_text` (app/src-tauri/tests/ipc.rs).

**finding-5**: guideline, medium — Boundaries 2, "keeps the error it received as the cause" — org-io/src/receiver.rs:192; the same pattern at org-io/src/propose.rs:38, 48, 55, 68, 79, org-io/src/exchange.rs:48, 77 and org-io/src/persona.rs:67
- org-io builds `OrgNodeError` values for refusals it decides itself, not ones org-node returned:
  - `OrgNotHeld` comes from org-io's own lookup.
  - `NoAdmissionRequestKept` is raised after `record_admission` already succeeded.
  - `NoProvisionalChange` and `NoProvisionalFounding` are org-io's own post-checks.
  - At receiver.rs:192, `PersonaNotHeld` is reported for a Persona that is held but bound to no Organisation yet. The comment directly above says so, so the refusal misleads.
- The A2 exception (org-io draft :1624-1647) covers only wrapping an `OrgNodeError` "it received".
disposition: fixed as AR2-5 in task `o2` (merged 2026-10-28): org-io has its own refusal variants for refusals it decides itself (`OperationError::{NotHeld, AdmissionNotListed, ProposalNotListed}`, `ExchangeError::NotHeld`, `CreateError::FoundingNotListed`, and `ReceiveError::NoOrganisation` in org-io/src/receiver.rs replacing the misreported `PersonaNotHeld`); the app's classification gained the case and the A2 exception text under SDD-z3ychz states its limit.

**finding-6**: guideline, medium — Boundaries 3, "One responsibility per item … without \"and\"" — org-io/docs/architecture/DRAFT-worktree-org-io-domain-api-domain-api.md:293 (SDD-6gb59p)
- The item states four operations: "issuing an Invite …, reading an imported Invite, and cancelling an outstanding invite …; and listing the admission requests …".
- Round 1 split SDD-83un8g on this same clause.
disposition: fixed as AR2-6 in task `o2`: SDD-6gb59p is split into SDD-zqv3v6 (outstanding invites), SDD-9rbm8t (an imported Invite read) and SDD-cxh9d8 (kept admission requests listed), in org-io/docs/architecture/2026-10-16-domain-api.md.

**finding-7**: guideline, medium — Domain 2, "A term a change adds to a public interface that `docs/CONTEXT.md` does not define is defined there by that change" (with Domain 1, "A public interface speaks the glossary") — org-io/src/transport/mod.rs:52-76; also `Delivery` at org-io/src/send.rs:20-31
- The change adds `Verdict::AlreadyReceived` (0x02) and `Verdict::InviteUsedByAnotherDevice` (0x03).
- docs/CONTEXT.md:317-320 still defines Verdict as the one byte saying "whether the message was acted on (accepted) or not (refused)". The change reworded only its _Avoid_ line.
disposition: fixed as AR2-7 in task `n2` (merged 2026-10-28): docs/CONTEXT.md's Verdict entry now defines the admission answers `AlreadyReceived` and `InviteUsedByAnotherDevice` beside accepted and refused.

**finding-8**: guideline, medium — Boundaries 2, "it does not pass a dependency's error type through to its caller" — org-node/src/service.rs:1480
- A new line builds `OrgNodeError::Trie(OrgMembersError::DuplicateKey)` for org-node's own refusal: a freshly drawn Organisation public key equal to the record's.
- A caller has to match org-members' variant to handle it. LLR-d4gpxm and LLR-qghs95 in the org-node draft state the same refusal.
- Other new call sites pass org-members' errors through unchanged with `.map_err(OrgNodeError::Trie)`: service.rs:176-178, 469, 529, 630-641, 1466-1473 and verify_org.rs:137.
- The A2 exception concerns org-io's surface and does not cover this.
disposition: fixed as AR2-8 in task `n2`: the new `OrgNodeError::DuplicateOrgPublicKey` in org-node/src/error.rs is org-node's own refusal for a drawn Organisation public key equal to the record's, with LLR-9qavm7 and LLR-qghs95 amended; the remaining `Trie(..)` sites pass on failures that org-members itself detects.

**finding-9**: guideline, medium — Domain 2, "A term a change adds to a public interface … is defined there by that change" — org-node/src/person_store.rs:94
- `PersonStore`, `PersonData` and `PersonView` are re-exported at org-node/src/lib.rs:37 and named in org-io's interface (LLR-5uf4u2).
- No glossary defines "Person store". The related "Person file" (org-io/src/store_file.rs:94 `person_path`, :140 `list_person_stores`, and `OpenError::MoreThanOnePerson`'s text) is undefined too.
- docs/CONTEXT.md:312-315 still says a Store file is "One per Persona".
disposition: fixed as AR2-9 in task `n2`: docs/CONTEXT.md defines **Person store** and **Person file**.

**finding-10**: guideline, medium — Boundaries 3, "One responsibility per item … without \"and\"" — org-node/docs/architecture/DRAFT-worktree-org-io-domain-api-domain-api.md:1355 (SDD-xexu3k)
- The item covers "the Invite and the admission request". It also says "the endpoint addresses one Device keeps for another … are this item's too".
disposition: fixed as AR2-10 in task `n2`: SDD-xexu3k is split into SDD-n2x3gb (invitation) and SDD-42cjqk (address book), in org-node/docs/architecture/2026-10-16-domain-api.md.

**finding-11**: guideline, medium — Boundaries 3, same clause — org-node/docs/architecture/DRAFT-worktree-org-io-domain-api-domain-api.md:1036 (SDD-73emgx)
- The item is "revocation notices and acknowledgements". It builds and checks notices, decides which are kept, signs acknowledgements and checks acknowledgements.
disposition: fixed as AR2-11 in task `n2`: SDD-73emgx is split into SDD-gatf4h (revocation notices) and SDD-8zzmav (removal acknowledgement signature).

**finding-12**: guideline, low — Boundaries 3, same clause — several SDD items whose responsibility sentence joins two concerns with "and":
- app draft :919-924 (SDD-xg4nj7);
- app draft :1035-1038 (SDD-ckqyh3);
- app draft :838-841 (SDD-ktsgh5, "… and when it asks for one");
- app draft :798-805 (SDD-tgu7dp, "…; and it announces each result");
- app/docs/architecture/README.md:76 (Webview row);
- on-chain-client/docs/architecture/README.md:105 (SDD-rxfu6h, "the pure decoders … and their two fetches");
- org-io draft :642 (SDD-t4z8sz, verification "… and settling an unknown chain write", shared with SDD-mk39e5);
- org-io draft :384 (SDD-frsa73, "the Admin's provisional changes and the proposal");
- org-node draft :1517 (SDD-pkqc2j, "the organisation request and its answer");
- org-node draft :917 (SDD-z4m57w);
- org-node draft :47 (SDD-undd7q, "holds …, bounds it, and seals it");
- org-node draft :546 (SDD-kegag2, "decides exactly one outcome and, on adoption, replaces …").
disposition: fixed by the round 3 sweeps of every SDD item (dispatcher's note, round 3): in task `a3` SDD-ckqyh3 and SDD-ktsgh5 were split and SDD-xg4nj7, SDD-tgu7dp, SDD-undd7q, SDD-kegag2, SDD-z4m57w and SDD-pkqc2j reworded with one responsibility; in task `o3` SDD-frsa73 was superseded and SDD-t4z8sz and SDD-mk39e5 restated (`docs/plans/2026-10-21-org-io-domain-api.md`, round 3). The plan records no rewording of the app README's Webview row or on-chain-client's SDD-rxfu6h (on-chain-client/docs/architecture/README.md); those two sites are not fixed — carried open, not blocking.

**finding-13**: guideline, low — Boundaries 2, "keeps the error it received as the cause" — new code that keeps the cause only as text:
- org-io/src/verify.rs:232, 236, 265, 269 (`Verified::Store { reason: failure.to_string() }`, `Verified::Refused`, `UnverifiedReason::Read`);
- org-io/src/send.rs:186 (`SendError::SenderEndpoint(String)` from `TransportError`);
- org-io/src/acknowledge.rs:347;
- org-io/src/write.rs:38-54 with 88, 104-107, 111, 134 (the `ChainWriter` seam turns on-chain-client's `WriteError` into `String`; write.rs is new code, so Existing code 1 applies);
- org-node/src/store.rs:420, :1352, service.rs:1434, person_store.rs:141 (encode failures reported as `OrgNodeError::Chain(format!(…))`, which displays "chain read failed: …").
disposition: partly fixed: round 3's AR3-2 (task `o3`) typed the causes at org-io/src/write.rs (`ChainWriteError`), org-io/src/verify.rs and org-io/src/view.rs with `org_io::Cause` (LLR-48me7m, LLR-5tpdxw, LLR-s7rq5x amended); `SendError::SenderEndpoint(String)` in org-io/src/send.rs and org-node's encode failures reported as `OrgNodeError::Chain(format!(…))` remain, and recur as round 4 finding-6 and round 5 findings 1 and 2 — not fixed — carried open, not blocking.

**finding-14**: guideline, low — Boundaries 1, "no reliance on its storage layout or file format" (with Boundaries 2) — app/src-tauri/src/commands.rs:209-216 (`field_refusal`)
- It strips org-node's internal field-path prefixes `"person."`, `"persona."` and `"member."` from `OrgNodeError::InvalidField` reached through org-io.
- Naming the type is covered by A2 (app draft :720-729). Relying on its field-text format is not.
disposition: no change — the plan records no fix of `field_refusal` (app/src-tauri/src/commands.rs); round 3 folded its match into AR3-1, which owner ruling 2026-10-28 keeps as the recorded A2 exception (SDD-z3ychz, LLR-wgr46u), and the reliance on org-node's field-prefix format recurs open as round 5 finding-10; carried open, not blocking (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-15**: guideline, low — Core and edges 2, "Each access to a … clock … is in an adapter the project owns, and the core reaches it through an interface the core defines" — org-io/src/acknowledge.rs:315
- The delivery schedule reads time through `Clock` but waits with `tokio::time::sleep` directly: a second time source outside the seam.
- The test-only `wake` Notify (acknowledge.rs:69-71) works around it. LLR-fzf2n3 says org-io "gives no other time".
disposition: not fixed — carried open, not blocking; the plan's round 2 records no disposition for it and org-io/src/acknowledge.rs still waits with `tokio::time::sleep` beside the `Clock` seam (LLR-fzf2n3) (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-16**: guideline, low — Constraints 1, "A structural rule that a tool can check … is written as an LLR on the software item it binds and verified by an architecture test" — org-io/src/clock.rs:1-2 and org-node/tests/absences.rs:756
- org-io: "org-io keeps no time of its own" (only clock.rs reads `SystemTime`) has no architecture scan, while the rng rule does (LLR-ctm86b, org-io/tests/absences.rs:1309-1329).
- org-node: the clock-read scan for LLR-g2jeqd covers revocation.rs, verify_org.rs, removal.rs and service.rs, but not the new person_store.rs, nor store.rs or invitation.rs.
- The code is right today; enforcement is missing.
disposition: not fixed — carried open, not blocking; the plan's round 2 records no new architecture scan for org-io's clock rule or for widening org-node's LLR-g2jeqd clock scan in org-node/tests/absences.rs (task `n2` changed that scan only to read `pub fn`), and round 4's reviewer notes the same gap without a finding.

**finding-17**: guideline, low — Domain 1, "never a synonym it lists under `_Avoid_` … and the words the user reads" — words the user reads and new public names that use an _Avoid_ synonym:
- org-io/src/write.rs:273: the Unknown write's reason says "the write timed out after {} s". CONTEXT.md:262 avoids "timed-out write".
- app/src-tauri/src/commands.rs:176: "the user has not confirmed the admission request". The app's Operator term and the root Member/Person terms avoid "user".
- org-node/src/removal.rs:52: `KeptAcknowledgement.ack`. Removal acknowledgement avoids "ack".
- org-node/src/store.rs:558: `ProvisionalOrganisationView.current_proposal`. org-node/docs/CONTEXT.md:40 still avoids "proposal"; the entry is marked replaced, but its _Avoid_ list is not retired.
- org-node/src/wire.rs:55, store.rs:813 and :862: new public `member_key` fields. CONTEXT.md:59-61 exempts only `person_key`/`group_key` and avoids "member key". The identifier is already widespread on master, which is why this is low.
disposition: not fixed — carried open, not blocking: task `n2` declined renaming `ack` because it changes encoding labels, and the plan records no fix for the other words (org-io/src/write.rs's "timed out" Unknown-write reason, commands.rs's "user", `current_proposal`, `member_key`); round 4's AR4-4 renamed only `CreateProxyTimedOut` (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-18**: guideline, low — Domain 2, "A term a change adds to a public interface … is defined there by that change" — other new public terms no glossary defines:
- "removed store": org-node/src/removal.rs:42 `RemovedStore`. It is used in the Removal acknowledgement definition but never defined.
- "acknowledgement report": org-io/src/view.rs:139. Only named in passing.
- "Organisation answer": org-node/src/service.rs:1577 `Checked::OrganisationAnswerKept`, store.rs:464 `Origin::Answer`.
- `OutstandingInvite` and `RevokedDevice`: org-node/src/store.rs:907, :915.
- `VerificationTime`: org-node/src/verify_org.rs:23.
- `HeldTarget` and `WriteRefused::Stale`: org-io/src/write.rs:163, :185.
- "founder", "discarded change" and "not checked": app/src-tauri/src/commands.rs:161, :584-586, :624, :91, shown at PersonaList.svelte:163 and Membership.svelte:323.
disposition: partly fixed: round 3's AR3-9 (task `a3`) defined Discarded change, Founder, Held target, Removed store and Verification time in docs/CONTEXT.md; the plan records no definition of "acknowledgement report", "Organisation answer", `OutstandingInvite`, `RevokedDevice` or `WriteRefused::Stale`, which are not fixed — carried open, not blocking.

**finding-19**: guideline, low — Domain 3, "Inside one unit a term means one thing" — app/docs/CONTEXT.md:97-102 (Invite reply), with :81-85, :42, :75-79 and :118-122
- The change replaced the Invite reply with the Admission request, and the root glossary now lists "Invite reply" under _Avoid_ for it. The app glossary still defines Invite reply for that same message.
- The Blob, Command ("Twelve exist"), Peer address and Verification log entries contradict the changed interface: seventeen commands, no command takes a peer address, and verification comes from org-io.
- app/docs/CONTEXT.md is not in the diff.
disposition: fixed as round 3's AR3-7 in task `a3`: app/docs/CONTEXT.md now has the Invite answered by an Admission request, Invite reply retired, the Command count at 17, Peer address and Operator corrected.

**finding-20**: guideline, low — Core and edges 4, "A library … is called from one adapter the project owns, not from the core and not from many places" — org-node/src/person_store.rs:7, :95 (chacha20poly1305), :134, :157 (postcard)
- The AEAD library is now named in a second module beside store.rs, and postcard gains new call sites.
disposition: partly fixed as round 4's AR4-7 in task `n4`: org-node/src/person_store.rs no longer names chacha20poly1305 (`StoreCipher` lives in org-node/src/store.rs); the new postcard call sites stay because one encoding adapter would be a design change — not fixed — carried open, not blocking.

### Round 3

**finding-1**: guideline, medium — Boundaries 2, "An interface reports a failure in its own terms and keeps the error it received as the cause; it does not pass a dependency's error type through to its caller" — org-io/src/lib.rs:329, org-io/src/lib.rs:377, org-io/src/receiver.rs:63, org-io/src/write.rs:188, org-io/src/send.rs:86, org-io/src/send.rs:232, org-io/src/exchange.rs:18; app/src-tauri/src/events.rs:109-172; app/src-tauri/src/commands.rs:214-224.
org-io's public errors (`OperationError::Node`, `CreateError::Node`, `ReceiveError::Refused`, `WriteRefused::Node`, `SendRefused::Node`, `RequestError::Node`, `ExchangeError::Node`) carry org-node's `OrgNodeError` out to the app. The app then handles the dependency's own type, which is the failure the clause's "Why" names:
- `classify_receive_error` matches every `OrgNodeError` variant. This change adds 19 variants to that match and removes 5, so each org-node refusal added for this change edited the app.
- `field_refusal` matches `CreateError::Node(OrgNodeError::InvalidField { field, .. })` and strips org-node's internal field prefixes `"person."`, `"persona."` and `"member."`. The change added `person.` and `member.`, so the app now relies on org-node's field-naming format.
disposition: no change — AR3-1 is A2, and owner ruling 2026-10-28 keeps it accepted as a recorded exception to Boundaries 2 (SDD-z3ychz in org-io/docs/architecture/2026-10-16-domain-api.md, LLR-wgr46u), so the plan records no fix; the exception's limit was stated by AR2-5's fix in task `o2`.

**finding-2**: guideline, medium — Boundaries 2, "keeps the error it received as the cause" — org-io/src/write.rs:35-55, 88, 104-107, 111, 134; org-io/src/verify.rs:22, 85, 89, 232, 236, 262, 265, 269; org-io/src/view.rs:133, 146-150.
The new chain-write seam `ChainWriter` returns `Result<_, String>`. `OnChainWriter` turns on-chain-client's `WriteError` into text (`error.to_string()`, `format!("genesis step {step:?} ...")`), so `WriteReport::Refused` and `WriteReport::Unknown` carry only a string. Verification does the same:
- `Verified::Refused { reason: String }` holds `error.to_string()` of an `OrgNodeError`.
- `Verified::Store { reason: String }` holds `failure.to_string()` of org-io's own `StoreFailure`.
- `UnverifiedReason::Read(String)` holds the reader's error as text.

`AcknowledgementOutcome::NotDelivered { reason: String }` and the report's `last_deletion_failure` / `last_round_failure` do the same. In each case the received error is gone, so no caller can tell one cause from another except by parsing text.
disposition: fixed as AR3-2 in task `o3` (merged 2026-10-28): `ChainWriteError { NotConfigured, Write, Window }` for `ChainWriter`, `WriteReport::Refused { failure: WriteFailure }`, and `org_io::Cause<E>` for `Verified::Refused`, `Verified::Store`, `UnverifiedReason::Read` and `NotDelivered`, with typed `DeletionFailure` and `RoundFailure`; each `Display` keeps its text and `source()` returns the cause (LLR-s7rq5x, LLR-48me7m, LLR-5tpdxw and others amended), and task `a4` updated LLR-jf63e4 to name `Verified::Refused { error }`.

**finding-3**: guideline, low — Boundaries 2, "An interface reports a failure in its own terms" — org-node/src/person_store.rs:141, org-node/src/store.rs:420, org-node/src/store.rs:1352, org-node/src/service.rs:1437; org-io/src/connect.rs:84, org-io/src/connect.rs:88.
- New org-node code reports postcard encode failures (person store, epoch state, removed store, record snapshot) as `OrgNodeError::Chain(format!(...))`, which is the chain-read variant. The app's `classify_receive_error` files `Chain(_)` as a chain read.
- org-io's new `StateReaderNotConfigured::finalised_block`, and the edited `read_state`, report org-io's own "chain not configured" refusal by building org-node's `OrgNodeError::Chain`.

Neither failure is given in the reporting interface's own terms. The trigger is rare (an encode failure) or the meaning is close (a chain that is not configured), so this is low.
disposition: not fixed — carried open, not blocking; the plan records no fix of AR3-3, and the same encode sites recur as round 4's AR4-6, which task `n4` declined because adding or renaming an `OrgNodeError` variant (org-node/src/error.rs) changes a public enum the app matches exhaustively, and as round 5 finding-1.

**finding-4**: guideline, medium — Boundaries 3, "A software item has one responsibility, stated in one sentence without 'and'" — org-io/docs/architecture/DRAFT-worktree-org-io-domain-api-domain-api.md:451.
SDD-frsa73 reads "the Admin's provisional changes and the proposal of the updated Organisation". These are two responsibilities:
- recording an admission or a revocation (`propose_admission`, `propose_revocation`);
- having org-node calculate one provisional organisation from all of them (`propose_updated_organisation`).

They change for different reasons. The same split was already made for SDD-6gb59p, SDD-83un8g, SDD-73emgx and SDD-xexu3k in this change.
disposition: fixed as AR3-4 in task `o3`: SDD-frsa73 is superseded by SDD-7mhqaz (provisional changes recorded) and SDD-4drrpr (updated Organisation proposed), and the sweep restated the other org-io SDD items with one responsibility each.

**finding-5**: guideline, medium — Boundaries 3, "stated in one sentence without 'and'" — app/docs/architecture/DRAFT-worktree-org-io-domain-api-domain-api.md:1122.
SDD-ckqyh3 reads "the decisions ... whether a revocation may be proposed, and what the invitee is told before the first admission request and whether it may be sent". It joins two unrelated gates, the Admin's revocation input (`revoke.ts`) and the invitee's admission-request warning (`invite.ts`), in one item.
disposition: fixed as AR3-5 in task `a3` (merged 2026-10-28): SDD-ckqyh3 is split into SDD-czjf2c (revoke.ts) and SDD-7aq69w (invite.ts), and the sweep also split SDD-ktsgh5 into SDD-swt9fs and SDD-ua2d9b.

**finding-6**: guideline, medium — Core and edges 1, "Decisions, calculations, validation and state transitions are in code that reads and writes no file, network ..."; Core and edges 3, "The core imports no adapter" — app/src/lib/founding.ts:1-7, 32-43.
`proposeFounding` holds a decision: pass over org-io's `person_already_held` refusal, throw any other, then propose the founding (LLR-k57uqv, LLR-6ff735). That decision sits inside the function that makes the two IPC calls, and the module imports the IPC adapter's functions (`proposePerson`, `proposeFoundingOrganisation` from `./api`). The webview's other decision modules (`notices.ts`, `verification.ts`, `invite.ts`, `revoke.ts`) import only types or nothing, and are called by the components, which are the adapters. `founding.ts` reverses that direction, and its test (`tests/founding.propose.test.ts`) needs the IPC fake (`tests/fake-tauri.ts`).
disposition: fixed as AR3-6 in task `a3`: the founding decision is the pure `personRefusalStep` in app/src/lib/founding.ts, which imports only types, and CreateOrg.svelte makes the IPC calls (SDD-g5wmxt, LLR-d8u93q, LLR-5sxa6c); app/tests/founding.decision.test.ts watched it fail first, including the types-only import check.

**finding-7**: guideline, medium — Domain 3, "Inside one unit a term means one thing" — app/docs/CONTEXT.md:31-37, 39-44, 75-79, 87-95, 97-101, against app/src-tauri/src/lib.rs:52-68, app/src-tauri/src/commands.rs:172-178, 334-387, app/src/lib/components/Invite.svelte:148-171.
The app's glossary is not touched by this change, and its terms now mean something other than the app's changed interface:
- "Invite" names "the inviter's DevicePublicKeys" and is outstanding "until a reply naming both has been acted on". The Invite now names one issuing Device (`InviteDto.issuer_device_key`) and is answered by an Admission request.
- "Invite reply" is still defined, though the root glossary lists it under _Avoid_ for Admission request, and the app now sends `request_admission`.
- "Command" says "Twelve exist"; the invoke handler now registers 17.
- "Peer address" says it is "optional in every request that takes one"; this change retired every pasted address (SDD-ckqyh3's note, REQ-vgr7s2).
- "Operator" lists "admin" under _Avoid_, while the changed UI text says "Issue Invite (Admin)" and "the Admin will see your request".
disposition: fixed as AR3-7 in task `a3`: app/docs/CONTEXT.md now matches the interface for Invite, Invite reply (retired), Command (17), Peer address and Operator.

**finding-8**: guideline, medium — Domain 3, "Inside one unit a term means one thing" — org-node/docs/CONTEXT.md:108-112 and 139-150, against org-node/src/service.rs:351-363 (`check_received`) and org-node/src/verify_org.rs.
This change edits org-node's glossary (it marks "Provisional update" replaced) but leaves two definitions that its code contradicts:
- "Persona store" is still "the encrypted file ... the provisional updates it has built and not yet committed or discarded". It now holds provisional changes and provisional organisations, and the file is org-io's Store file, sealed as a SealedStore.
- "Receive operation" still includes "the node's own read of the Organisation state from the chain", and its first kind "commits a membership it verified". The new receive path is chain-free and never adopts (SDD-busuj3); only verification adopts (SDD-kegag2).
disposition: fixed as AR3-8 in task `a3`: org-node/docs/CONTEXT.md restates Persona store and Receive operation for the new store content and the chain-free receive path.

**finding-9**: guideline, medium — Domain 2, "A term a change adds to a public interface that `docs/CONTEXT.md` does not define is defined there by that change" — org-io/src/verify.rs:70 (`DiscardedChangeSummary`), org-io/src/persona.rs:15 and app/src-tauri/src/commands.rs:166, app/src-tauri/src/entry.rs:110 (`founder`), org-io/src/write.rs:163 (`HeldTarget`), org-node/src/removal.rs:42 (`RemovedStore`), org-node/src/verify_org.rs:23 (`VerificationTime`), app/src/lib/api.ts:75 and app/src/lib/notices.ts:82 (`DiscardedDto`, `DiscardedNotice`).
Each term below crosses a unit boundary and is new in this change, and `docs/CONTEXT.md` defines none of them:
- the discarded (left-out) change, which goes from org-node through org-io's verification outcome to the app's notices;
- the founder, which is `HeldTarget::Founding`'s key and the IPC `founder` field that `write_organisation_to_chain` takes;
- the held target;
- the removed store (org-node's `RemovedStore`, org-io's `removed-<hex>.store`), which the glossary's own "Removal acknowledgement" entry uses undefined;
- the verification time.
disposition: fixed as AR3-9 in task `a3`: docs/CONTEXT.md defines Discarded change, Founder, Held target, Removed store and Verification time.

**finding-10**: guideline, low — Domain 1, "the words the user reads" are glossary terms, "never a synonym it lists under `_Avoid_`" — app/src-tauri/src/commands.rs:236; app/src/lib/components/Revoke.svelte:54.
- `persona_id_for` returns "org-io lists no Persona with Device key ..." to the webview. "device key" is under DevicePublicKey's _Avoid_, which the glossary says governs user-facing text.
- The changed notice "member self-deleted (received revoke, removed self)" tells the user the removal came from a received revoke. Under this change a removal comes only from a Verification acting on a kept Revocation notice, and "revoke" / "self-deleted" are not the glossary's words for it.
disposition: fixed: low AR3-10's Revoke.svelte removal notice was reworded in task `a3` (app/src/lib/components/Revoke.svelte), and `persona_id_for`'s refusal in app/src-tauri/src/commands.rs says "DevicePublicKey" since task `a4` (not reachable through IPC without faking org-io, so untested).

**finding-11**: guideline, low — Domain 1, "never a synonym it lists under `_Avoid_`: exported functions and types ..." — org-node/src/removal.rs:47.
`KeptAcknowledgement`, a public type in the public module `org_node::removal`, names its Removal acknowledgement field `ack`. The glossary lists "ack" under _Avoid_ for Removal acknowledgement, and for Verdict.
disposition: not fixed — carried open, not blocking; the plan records no fix of AR3-11, and round 2's task `n2` had already declined renaming `ack` in org-node/src/removal.rs because it changes encoding labels.

**finding-12**: guideline, low — Domain 1, "never a synonym it lists under `_Avoid_`" — on-chain-client/src/write/mod.rs:121 (and subxt_ops.rs:131, ceremony.rs:63-64).
The new public seam method `WriteOps::mapped_account(&self, admin: OrgAdmin)` names the Organisation's 20-byte identifier `admin`. The root glossary lists "Organisation admin (on-chain-client's term for the Organisation's 20-byte identifier, which is not a user)" under Admin's _Avoid_. The type `OrgAdmin` predates this change; the new parameter name does not.
disposition: not fixed — carried open, not blocking; the plan's round 3 records no fix of AR3-12, and `WriteOps::mapped_account` in on-chain-client/src/write/mod.rs still names its parameter `admin` (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-13**: guideline, low — Core and edges 4, "A library, service or SDK outside the language's standard library is called from one adapter the project owns, not from the core and not from many places" — org-io/src/rng.rs:33, org-io/src/store_file.rs:18, 211; org-node/src/person_store.rs:7, 134, 157; org-node/src/removal.rs:34.
- `HandleRng::lock` hands its callers the raw `Box<dyn rand_core::CryptoRngCore>`, so `store_file.rs` imports `rand_core::RngCore` and calls `try_fill_bytes` itself instead of going through the random-source item (SDD-3qwjs2).
- In org-node, the new `person_store.rs` names `chacha20poly1305::XChaCha20Poly1305` beside `store.rs`, and it and `removal.rs` add postcard call sites, rather than going through one owner.
- The new org-io files `acknowledge.rs`, `retention.rs` and `write.rs` add further direct `tokio` and `hex` call sites.
disposition: partly fixed: round 4's AR4-7 in task `n4` removed chacha20poly1305 from org-node/src/person_store.rs; the `rand_core` call in org-io/src/store_file.rs, the postcard sites and the `tokio`/`hex` call sites are not fixed — carried open, not blocking (they recur as round 5 findings 4 and 5).

**finding-14**: guideline, low — Constraints 1, "A structural rule that a tool can check, such as 'this item performs no I/O' ... is written as an LLR on the software item it binds and verified by an architecture test" — app/docs/architecture/DRAFT-worktree-org-io-domain-api-domain-api.md:592-594 (and app/src-tauri/src/follow_up.rs:2).
SDD-6zqqm8 states that its file "reads org-io's outcome and performs no I/O" and that it is "Pure". No LLR states this, and no app test checks `follow_up.rs`'s imports. LLR-292r26 and LLR-bbfgk7 state only its results. The code is pure today, so this is a gap where the code is right.
disposition: not fixed — carried open, not blocking; the plan records no LLR or architecture test for app/src-tauri/src/follow_up.rs's purity (SDD-6zqqm8 states it; LLR-292r26 and LLR-bbfgk7 state only its results) (`docs/plans/2026-10-21-org-io-domain-api.md`).

### Round 4

**finding-1**: guideline, medium — Boundaries 2, "An error crosses a boundary as the interface's own error ... it
does not pass a dependency's error type through to its caller". At `org-io/src/verify.rs:22` (with :45, :235,
:239), the new public `UnverifiedReason::Read(Cause<OrgNodeError>)` reports org-io's own chain-read failure (the
block read or state read in `StateReader`, which `chain_read.rs`/`connect.rs` encode as
`OrgNodeError::Chain(String)`) in org-node's error type. The recorded exception covers "only the refusals org-node
returned". It says a failure org-io decides itself "is its own variant". A failed RPC read is org-io's I/O, not an
org-node refusal, and it reaches the app in every verification while the chain is unreachable. It needs an org-io
read error that keeps the reader's error as its cause. The `StateReader` seam returning `OrgNodeError` is existing
code (Existing code 1). The new public variant is not.
disposition: fixed as AR4-1 in task `o4` (merged 2026-10-28): the new `verify::ChainReadError` in org-io/src/verify.rs holds the reader's failure privately (Display is its text, `source()` the typed cause), so `UnverifiedReason::Read(ChainReadError)` no longer names `OrgNodeError`; the A2 exception text under SDD-z3ychz now says a failed chain read is outside it, and the app, which matches no `Read`, is unchanged.

**finding-2**: guideline, low — Boundaries 2, same words. `org-io/src/verify.rs:85`
(`Verified::Refused { error: Cause<OrgNodeError> }`), `org-io/src/view.rs:152`
(`DeletionFailure::Listing(Cause<OrgNodeError>)`) and `org-io/src/view.rs:190`
(`RoundFailure::Listing(Cause<OrgNodeError>)`) are new public org-io types that carry `OrgNodeError`. They hold
refusals org-node returned, so they fit the exception's principle. But the exception lists its enums by name
(`WriteRefused`, `SendRefused`, `RequestError`, `CreateError`, `OperationError`, the `exchange.rs` error,
`ReceiveError::Refused`, `OpenError::Unseal`/`Service`, `KeychainError::Store`), and these three are not on that
list. The fix is to record them in the exception or wrap them.
disposition: fixed as AR4-2 in task `o4`: the A2 exception text under SDD-z3ychz (org-io/docs/architecture/2026-10-16-domain-api.md) lists `Verified::Refused`, `DeletionFailure::Listing` and `RoundFailure::Listing`; round 5 records every public org-io type that carries `OrgNodeError` as on the exception's list.

**finding-3**: guideline, medium — Domain 1, "The names in a unit's or a software item's public interface are the
terms `docs/CONTEXT.md` defines, never a synonym it lists under `_Avoid_`". At `org-node/src/person_store.rs:28`,
re-exported at `org-node/src/lib.rs:37`, the new public type `PersonData`, returned by `PersonStore::data()`, uses
"person data". The root glossary lists that under **Person definition** `_Avoid_: person record, person data,
profile` (`docs/CONTEXT.md:90`). The glossary terms for what it holds are Person store and Person.
disposition: fixed as AR4-3 in task `n4` (merged 2026-10-28): `PersonData` is renamed `PersonStoreContent` in org-node/src/person_store.rs, with SDD-27p679, LLR-wqu26s and LLR-5uf4u2 amended in place and LLR-jqb5z7 renamed to match in task `o4`.

**finding-4**: guideline, low — Domain 1, same words, including "the words the user reads". At
`org-io/src/write.rs:220` and :235-238, the new public `WriteFailure::CreateProxyTimedOut` and its text "creating
the pure proxy timed out after 90 s" name a chain write whose bound elapsed as "timed out". The app shows that
text through `WriteDto::Refused { reason }`. The root glossary's **Unknown write** lists `_Avoid_: pending write,
failed write, timed-out write`, and **Not executed** lists `timed out`. If a proxy creation whose bound elapsed is
deliberately not an Unknown write (PR-8ghk9s), it needs a glossary term of its own (Domain 2), not an avoided
word. (`WindowReadTimedOut` names a read, so it is not affected.)
disposition: fixed as AR4-4 in task `o4`: the root glossary docs/CONTEXT.md gains **Unanswered proxy creation**, and `CreateProxyTimedOut` is renamed `CreateProxyUnanswered` in org-io/src/write.rs with new text.

**finding-5**: guideline, low — Boundaries 1, "no reliance on its storage layout or file format". At
`app/src-tauri/tests/ipc.rs:407-411` (new in this diff), the app test lists org-io's data directory and filters
names by the `"person-"` prefix. That prefix is org-io's private `PERSON_PREFIX` (`org-io/src/store_file.rs:23`).
The app now depends on org-io's file naming. org-io exposes `store_file::list_person_stores` for this.
disposition: fixed as ARCH low 5 in task `a5` (merged 2026-10-28): app/src-tauri/tests/ipc.rs lists Person files through `org_io::store_file::list_person_stores` instead of org-io's private prefix.

**finding-6**: guideline, low — Domain 3, "Inside one unit a term means one thing". In org-node,
`OrgNodeError::Chain(String)` renders as "chain read failed: ..." (`org-node/src/error.rs:44-45`). New code in
this change raises it for encoding failures that involve no chain:
- `org-node/src/person_store.rs:141` ("person store encode")
- `org-node/src/store.rs:420` ("epoch state encode")
- `org-node/src/store.rs:1352` (removed store encode)
- `org-node/src/service.rs:1437` ("record snapshot encode")

So "Chain" means both a chain read and a local encode, and the app classifies `Chain(_)` as a chain failure. The
same misuse at older call sites is existing code (Existing code 1). Severity is low because postcard encoding of
these types fails only in contrived states.
disposition: not fixed — task `n4` declined AR4-6: renaming or adding an `OrgNodeError` variant (org-node/src/error.rs) changes a public enum that the app matches exhaustively under the A2 tripwire (LLR-wgr46u); carried open, not blocking, and seen again as round 5 finding-1.

**finding-7**: guideline, low — Core and edges 4, "A library ... outside the language's standard library is called
from one adapter the project owns, not from the core and not from many places". New org-node modules call
`postcard` directly:
- `org-node/src/person_store.rs:134` and :157
- `org-node/src/removal.rs:34` (equality by encoding)
- new calls in `org-node/src/service.rs` (`:65`, `:73`, `:1437`) and `org-node/src/store.rs` (`:419`, `:1271`,
  `:1280`, `:1352`)

`person_store.rs:7`, :95 and :101 also name `chacha20poly1305`'s `XChaCha20Poly1305`, although `store.rs`
already wraps the cipher (`labelled_cipher`, `seal_bytes`, `open_sealed`). Under Existing code 1, the older
postcard call sites stay. The new modules add more places that call the library.

note: `on-chain-client`'s `WriteOps` adds `free_balance`, `mapped_account` and `WriteError::MalformedStorage`
(`on-chain-client/src/write/mod.rs:48-50`, :116-120). It also adds public `create_proxy` and `found_at`
(`ceremony.rs`). "pure proxy", "free balance" and "mapped account" are Substrate vocabulary that no glossary
defines. I did not raise this under Domain 2 because they are chain-mechanism terms in a chain-client crate, not
domain terms. A stricter reading would want "pure proxy" defined, since the root glossary uses it (Admin,
Provisional founding).

note: org-node's existing `verify.rs` (`VerifyContext`, the chain-free receive check) sits beside the new
`verify_org.rs` (Verification as the glossary defines it). "verify" then names two things in one unit (Domain 3),
but `verify.rs` is not edited by this change (Existing code 1).

note: Constraints 1. `org_node_is_sans_io` checks files, environment, transport and runtime across the crate. It
checks the clock (`SystemTime`, `Instant`) only in `revocation.rs`, `verify_org.rs` and `removal.rs`. SDD-z4m57w
says "It reads no clock" of code that includes `OrgService::acknowledgement_accepted` and `kept_acknowledgements`
in `service.rs`, which no clock check scans. I would have flagged it as a gap in the tool check. The file does not
say how wide the check must be.

note: Boundaries 3. I checked the SDD items in the four DRAFT architectures. The two-subject items (SDD-73emgx,
SDD-xexu3k, SDD-frsa73, SDD-6gb59p, app SDD-ktsgh5 and SDD-ckqyh3) are superseded by single-responsibility
successors. I found no current item stated with two responsibilities.

Checked with no finding:
- Core and edges 1-3: org-node gains no file, environment, clock, random or network access. org-io keeps its
  clock behind `Clock` and its random source behind `HandleRng`. The app's TS pure modules (`founding.ts`,
  `notices.ts`, `verification.ts`) import only types from `api.ts`, the one module that imports `@tauri-apps`.
  The app holds no rand.
- Boundaries 1: the app reaches org-io only through its public modules, and never `org_node` directly.
- `ChainWriteError` keeps on-chain-client's `WriteError`/`ClientError` as its typed cause (Boundaries 2 met).

disposition: partly fixed as AR4-7 in task `n4`: org-node/src/person_store.rs no longer names chacha20poly1305, because `StoreCipher` is in org-node/src/store.rs; the postcard call sites stay, because one encoding adapter would be a design change — not fixed — carried open, not blocking. The report's notes raise no finding and the plan records no action on them.

### Round 5

**finding-1**: guideline, low — Boundaries 2, "An interface reports a failure in its own terms and keeps the error it received as the cause" — org-node/src/person_store.rs:140, org-node/src/service.rs:1437, org-node/src/store.rs:420, org-node/src/store.rs:1356
These lines are new. Each reports a postcard *encode* failure (person store, record snapshot, epoch-state
bound, removed store) as `OrgNodeError::Chain(format!(..))`. That is the chain-failure variant, so the
failure is not in org-node's own terms for what happened. The postcard error is also flattened to text
rather than kept as the cause. Downstream, the app's `classify_receive_error` files any `Chain(_)` as a
chain-side receive error. An encode failure should get its own variant (e.g. a `StoreRefused` or
`Encode` cause) that holds the error. Low: postcard encoding of these types fails only in a contrived
state.
disposition: not fixed — carried open, not blocking; round 5's ARCHITECTURE review had nothing above low, so its loop ended, and the plan records no fix; the same encode sites were declined as AR4-6 in task `n4` because a new `OrgNodeError` variant (org-node/src/error.rs) changes the enum the app matches exhaustively (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-2**: guideline, low — Boundaries 2, "keeps the error it received as the cause" — org-io/src/acknowledge.rs:381, org-io/src/send.rs:197, org-io/src/transport/endpoint.rs:298,302
- acknowledge.rs:381 is new. It turns org-io's own typed `TransportError` (a failed direct bind) into
  text with `SendError::SenderEndpoint(format!("the direct bind failed: {error}"))`.
- send.rs:197 is in the rewritten `send_all`. It does the same with `error.to_string()`.
- `send_for_verdict` (endpoint.rs:298,302) is new. It keeps iroh's connect and stream errors only as
  `TransportError::Connect/Stream(String)`.

In each case the cause is gone and only its text remains. The crate's own `Cause` type says it exists so
that "the cause is kept, not turned into text" (lib.rs:325). The fix is to carry
`SenderEndpoint(Cause<TransportError>)`, or a boxed source in place of the String. Low: only the
diagnostic chain is lost.
disposition: not fixed — carried open, not blocking; the plan's round 5 records no fix, and `SendError::SenderEndpoint(String)` in org-io/src/send.rs still keeps the cause only as text (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-3**: guideline, low — Boundaries 2, "keeps the error it received as the cause; it does not pass a dependency's error type through" — on-chain-client/src/write/subxt_ops.rs:88,94,98
The new `fetch_finalised` (the founding step's two reads) wraps subxt's errors as
`WriteError::Subxt(format!(..))`. That is on-chain-client's own variant, but the subxt error survives
only as text. Lines 56–74 already follow this pattern and are not edited (Existing code 1); the new
lines copy it.
disposition: not fixed — carried open, not blocking; the plan's round 5 records no fix of `fetch_finalised`'s text-only subxt causes in on-chain-client/src/write/subxt_ops.rs (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-4**: guideline, low — Core and edges 4, "A library ... is called from one adapter the project owns, not from the core and not from many places" — org-io/src/store_file.rs:18,211
SDD-3qwjs2 places `rand_core` in `rng.rs` (`HandleRng`), and that is its adapter. `store_file.rs` now
imports `rand_core::RngCore` and calls `try_fill_bytes` directly on the guard that `HandleRng::lock`
returns, so `rand_core` is called from a second module. Give `HandleRng` a `try_fill` method and have
store_file call that.
disposition: not fixed — carried open, not blocking; the plan's round 5 records no fix, and org-io/src/store_file.rs still imports `rand_core::RngCore` beside `HandleRng` in org-io/src/rng.rs (SDD-3qwjs2) (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-5**: guideline, low — Core and edges 4, "A library ... is called from one adapter the project owns, not from the core and not from many places" — org-node/src/person_store.rs:133,156, org-node/src/removal.rs:36, org-node/src/service.rs:65,73,1437, org-node/src/store.rs:419,1275,1284,1356
The change adds postcard calls in two new core files (person_store.rs, removal.rs) and new lines in
service.rs and store.rs. That brings postcard to eight org-node modules, with no codec module of
org-node's own between them and the library. The older call sites are left alone (Existing code 1), but
the clause binds the new ones. Low: the code is correct, and postcard does no I/O.
disposition: not fixed — carried open, not blocking; task `n4` left the postcard call sites under AR4-7 because one encoding adapter would be a design change, and the plan's round 5 records no further action (org-node/src/store.rs, `docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-6**: guideline, low — Core and edges 1, "Decisions, calculations, validation and state transitions are in code that reads and writes no file, network ..." — on-chain-client/src/write/ceremony.rs:54-~98
The new `found_at` decides what to do from two chain reads: refuse a proxy mapped elsewhere, fund only
below `FUND_AMOUNT`, map only when unmapped. It makes that decision in the same function that calls the
seam to read, fund, dispatch and record. The rule can therefore be tested only through a `WriteOps`
double, not with plain values (the clause's Why). A pure `steps_for(mapped, balance) -> …` beside a
thin sequencing shell would meet the clause.
disposition: not fixed — carried open, not blocking; the plan's round 5 records no split of `found_at`'s decision in on-chain-client/src/write/ceremony.rs into a pure step (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-7**: guideline, low — Domain 1, "The names in a unit's ... public interface are the terms docs/CONTEXT.md defines, never a synonym it lists under _Avoid_ ... endpoints, events and messages" — app/src-tauri/src/commands.rs:174 (with app/src/lib/api.ts:49)
The new IPC field `InviteDto.issuer_device_key` uses "device key". The app glossary lists that term
under _Avoid_ ("using it in new text; DevicePublicKey is the term"). The fields beside it in the same
change are named `device_public_key` (PersonDto, AdmissionRequestDto, the admission-requested payload).
disposition: not fixed — carried open, not blocking; the plan's round 5 records no rename of `InviteDto.issuer_device_key` in app/src-tauri/src/commands.rs (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-8**: guideline, low — Domain 2, "A term a change adds to a public interface that docs/CONTEXT.md does not define is defined there by that change" — org-node/src/lib.rs:35 (also org-io/src/view.rs:114 `issuer_addresses`)
`EndpointAddresses` is newly re-exported from org-node's crate root and crosses org-node → org-io. The
root glossary uses "endpoint addresses" (in Organisation request) but does not define the term. The
app glossary defines the same thing as "Peer address", which org-io keeps per DevicePublicKey. The
result is one undefined interface term and a second name for the same thing in another unit.
disposition: not fixed — carried open, not blocking; the plan's round 5 records no glossary entry for `EndpointAddresses` (org-node/src/lib.rs); task `n5` only removed its `Default` (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-9**: guideline, low — Boundaries 3, "A software item has one responsibility, stated in one sentence without 'and'" — org-io/docs/architecture/DRAFT-worktree-org-io-domain-api-domain-api.md:1253-1255
SDD-dhh3pw's responsibility sentence still reads "each built by org-node from the sending Persona's
store **and** sent through the send path". The amendment note directly below it (line 1262) says the
responsibility "is restated in one sentence without 'and'". So the note claims a compliance the
sentence does not have.
disposition: not fixed — carried open, not blocking; the plan's round 5 records no rewording of SDD-dhh3pw's responsibility sentence in org-io/docs/architecture/2026-10-16-domain-api.md (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-10**: guideline, low — Boundaries 2, "a caller that handles a dependency's error depends on that dependency" — app/src-tauri/src/commands.rs:212-222
The recorded exception covers the app *matching* `OrgNodeError::InvalidField`. `field_refusal` goes
further. It strips org-node's record prefixes `"person."`, `"persona."`, `"member."` from the variant's
`field` string. That makes the app depend on how org-node names its fields inside the error, which the
exception does not record. If org-node renames a record prefix, the app's user text changes silently,
with no build break.
disposition: not fixed — carried open, not blocking; the A2 exception (owner ruling 2026-10-28, SDD-z3ychz, LLR-wgr46u) covers matching `OrgNodeError::InvalidField`, and the plan records no ruling or fix for `field_refusal`'s reliance on org-node's field prefixes in app/src-tauri/src/commands.rs (`docs/plans/2026-10-21-org-io-domain-api.md`).

**finding-11**: guideline, low — Boundaries 1, "Code outside a software item calls only what the item's interface exposes: no import of its internal modules, no access to its private state" — app/src-tauri/src/events.rs:247 (via org-io/src/receiver.rs:42)
The new `Received::Acknowledged` carries org-node's `org_node::revocation::VerifiedAcknowledgement`
unchanged. The app calls `verified.into_inner().org_id`, which reaches org-node's `Acknowledgement`
record through org-io. org-io's interface says no org-node record is reachable through it ("Nothing
else: org-node's service, its stores and their records ... are not reachable through org-io",
lib.rs:31-34, LLR-3zdw8v), and `org_io::node` does not re-export the type. org-io should report what
it decided (the Organisation and the acknowledging Device) as its own fields, as every other `Received`
variant does.
disposition: not fixed — carried open, not blocking; the plan's round 5 records no fix, and `Received::Acknowledged` in org-io/src/receiver.rs still carries org-node's `VerifiedAcknowledgement` against LLR-3zdw8v (`docs/plans/2026-10-21-org-io-domain-api.md`).

## Problem reports

- Resolves (each `status: resolved` in its ledger, resolution written in this
  change): PR-vt244s (org-node,
  `org-node/docs/problems/2026-09-09-org-node-problems.md`); PR-8sgsb5
  (org-node, `org-node/docs/problems/2026-10-25-domain-api.md`); PR-8ghk9s,
  PR-yng83g and PR-wch3zu (org-io, `org-io/docs/problems/2026-10-24-domain-api.md`).
- Answers in part: PR-k5tdxj item 2 (org-io,
  `org-io/docs/problems/2026-10-15-review-round-2.md`), with items 4 and part
  of 5 answered incidentally; the report stays `open` for items 1, 3 and the
  rest of 5.
- Accepts (status `accepted`, owner ruling 2026-10-28, re-confirmed
  2026-10-29: the defect is real; its fix belongs to a separate
  on-chain-client change): PR-w5sk5k, PR-h4mb8y, PR-uq5r97, PR-qpp28h
  (`on-chain-client/docs/problems/2026-09-10-on-chain-client-problems.md`).
- Opens (each `status: open`): PR-2t6ssm (app); PR-bbu24f, PR-hh4xvx (org-io);
  PR-ug8q68, PR-d4r6te, PR-xkp5bv, PR-y6ay44, PR-7ud5fv (org-node); PR-hj4wwd
  (on-chain-client). Each is listed under Gaps.

## Evidence outside the gate

### Evidence carried from test comments removed in this change

Task o4 (round 4, Comments 2) removed dated history from `org-io/tests/keychain_contract.rs` in commit 517ed79. The evidence it held is kept here; recover the full text with `git show 517ed79 -- org-io/tests/keychain_contract.rs`.

- **The real OS-keychain run of the credential contract** (`real_run`, verifies LLR-36jer8). It is opt-in and `#[ignore]`d: it runs with `ODS_REAL_KEYCHAIN_TESTS=1` and `--ignored`.
  - On 2026-10-15, during S4 (`e2a65d8`, before this change), it ran in the agent sandbox on macOS. The two missing-item checks passed against Keychain Services. The sandbox refused the first write (`PlatformFailure`, "UNIX[Operation not permitted]").
  - The owner ran it to the end outside the sandbox the same day, against Keychain Services: **2 passed**.
  - **Rerun on this change's tree, 2026-10-29, by the owner**, in the change worktree at `82f6a3c` (the review-dispatched commit; working tree clean), against macOS Keychain Services:
    `ODS_REAL_KEYCHAIN_TESTS=1 cargo test -p org-io --features test-support --test keychain_contract -- --ignored --exact the_os_keychain_keeps_the_credential_contract`
    → `test the_os_keychain_keeps_the_credential_contract ... ok`; `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out`.
- **Mutation checks, 2026-10-15** (S4):
  - `MemoryCredential::delete_credential` returning `Ok(())` for a missing item failed the fake's run.
  - `MemoryKeychain::build` giving each credential its own map failed it at the read through a second credential.
  - `Refusal::Locked` answering `PlatformFailure` failed the locked-store contract test.
- **The locked native keychain test** (`#[ignore]`d) needs the store locked by hand, for example with `security lock-keychain`. It is not reachable in the test environment, and no run of it is recorded.

## Gaps

- **Chopsticks targets never ran in the gate.** org-io's
  `the_production_writer_meets_the_founding_contract`
  (`org-io/tests/chain_writer_contract.rs`, the real half of the founding
  contract) and on-chain-client's `write_genesis_e2e`, including
  `subxt_reads_answer_the_write_reads_contract`, need a chopsticks fork and
  were never run, so they have no red evidence. `SubxtWriteOps::free_balance`,
  `mapped_account` and `fetch_finalised` (key encoding through
  `Value::from_bytes`, the `NoValueFound` → `None` mapping) have no executed
  evidence anywhere; this is SDD-3b8zef's recorded deviation (the
  chain-facing shell carries no LLRs). org-io's `write.rs` `h160_of` and its
  `WriteError::Step` mapping are reachable only through that chopsticks test.
- **SDD-z85ux9's preflight test** (`org-io/tests/preflight.rs`, the
  chain.live check) verifies no ID by design: no REQ states the check and the
  SDD carries no LLRs by its recorded deviation (owner ruling 2026-10-29,
  T5-O5).
- **The locked native keychain test** needs the keychain locked by hand and
  has no recorded run (above).
- **`fuzz_check_received`** runs about 50 inputs per run with no corpus; its
  header states the rate. Its perturbation now reaches every byte and its
  store comparison includes the behind marker, but the evidence per run is
  thin.
- **The app's `init`** reads the environment and connects org-io to the chain;
  that production path needs a real chain and is untested. The tests reach
  `startup_chain` (`startup_chain_builds_the_settings_with_twice_the_interval_it_decides`)
  but not `init` itself or `connect`'s conversion of the ahead limit to
  seconds.
- **A doubled send of an own adoption** cannot be seen at the app's interface
  (PR-2t6ssm): the request side is pinned, the send side is not.
- **CI does not run on-chain-client's writer set** of `verify_commands`
  (PR-hj4wwd, opened by this record; the gap predates this change). The local
  gate runs it.
- **Residual risk the owner accepted** (re-accepted 2026-10-31, RC-d4fc4s,
  `org-node/docs/risk/2026-10-15-domain-api.md`): a Device that knows the
  Organisation identifier and keeps flooding a store once per ahead limit can
  hold off an admission or an update indefinitely, and it is not identified
  (the sender is neither logged nor shown, PR-hh4xvx); S1/P2.
- **Problem reports this change opens and leaves open:**
  - PR-2t6ssm (app): the doubled send is unobservable at the app's interface.
  - PR-bbu24f (org-io; guardrails tooling): check-trace counts a superseded
    REQ's `expects:` as an open expectation (REQ-tzr3dk, REQ-jxyh9c,
    REQ-txkm94 advisories).
  - PR-hh4xvx (org-io): no part of the software logs or shows the sender of
    received Organisation information or answers; a refused body's sender is
    not kept.
  - PR-ug8q68 (org-node): any listed Device can fill the 1 MiB bound with
    revocation notices, after which the store can never adopt again.
  - PR-d4r6te (org-node): Organisation information received with a current
    record keeps its carried `record_snapshot` as `base_snapshot` unchecked.
  - PR-xkp5bv (org-node): a snapshot encode failure is reported as
    `OrgNodeError::Chain` ("chain read failed").
  - PR-y6ay44 (org-node): `Acknowledgement::signed_bytes` falls back to
    `ACK_DOMAIN` alone if encoding fails.
  - PR-7ud5fv (org-node): a failed write of a removal leaves the Persona
    Active with no behind marker until the next verification.
  - PR-hj4wwd (on-chain-client): CI does not run the writer set.
- **PR-k5tdxj stays open** for item 1 (no keyring call runs under
  `spawn_blocking` or behind a per-item lock), item 3 (an unknown write is
  recorded only on `WriteNotDone::TimedOut`) and the rest of item 5 (a
  failing recipient at a random fan-out position, LLR-murjr6's reloading call
  sites, LLR-b9bm36's `sync_all`/`create_new` mutation, LLR-z35vbh's bind
  observation).
- **Accepted on-chain-client reports**: PR-w5sk5k, PR-h4mb8y, PR-uq5r97 and
  PR-qpp28h are `accepted` (owner ruling 2026-10-28); their fixes belong to a
  separate on-chain-client change.
- **Open lows of the conformance reviews** are carried, not blocking, as
  their dispositions above say; among them, LLR-f3hru6 says `record_revocation`
  "draws nothing" while a successful one draws its seal's 24-byte nonce, and
  `OrgNodeError::Chain` still carries local encode failures, as before this
  change.
