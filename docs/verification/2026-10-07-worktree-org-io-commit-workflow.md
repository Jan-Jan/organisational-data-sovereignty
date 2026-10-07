# Verification — the commit workflow, stage S3a (org-node) (2026-10-07)

Drafted by plan task T12 of `docs/plans/2026-10-06-org-io-commit-workflow.md`;
brought up to date at `merge-change` step 6b after review rounds 1 and 2.
The dispatcher fills the gate counts of round 5.

branch: worktree-org-io-commit-workflow
reviewer: independent review subagent — round 1 at `02fad55` (tree `5fb20e2`); round 2 at `0471cee` (tree `2d35892` plus the record); round 2 was the last round (all code and requirement findings low), so the rerun ends at step 6b with no further reviewer
verdict: round 2, verbatim: "no code or requirement finding above low; findings 1–3 are low and findings 4–5 are record corrections, so this round can be the last per the checklist, once the record is corrected." Round 2's low findings were fixed (1, 3) or booked (2, PR-m9betm); record findings 4 and 5 corrected.
reproduced: yes for every defect but one, each watched red before its fix:
PR-qmvj83 by `a_revoked_device_receives_only_its_notice`
(`org-node/tests/service_stories.rs`, T8); PR-u4c2vp by
`pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused_on_the_self_delete_path`
(`org-node/tests/admission_sender.rs`, T12a); PR-eqs4fs by
`a_founder_that_commits_its_own_removal_signs_its_acknowledgement`
(`org-node/tests/commit_paths.rs`, T12b — red on the unfixed code, no
acknowledgement signed); review finding-1 by
`every_commit_keeps_the_change_set_that_produced_the_record`
(`org-node/tests/commit_paths.rs`, rewritten — red on the old code, which kept
no Change set on a first admission); review round 2 finding-1 by
`a_chain_state_at_the_records_epoch_with_another_root_is_a_conflict`
(`org-node/tests/revocation.rs` — red on the code before the fix, which called
the seed source). PR-q8r32t was NOT reproduced as a failing
test and cannot be: it reported two `RevocationNotHeld` arms no input reaches,
so no test could exercise them; the fix removed both arms, confirmed by
`git grep` on the source (its resolution in
`org-node/docs/problems/2026-10-07-review-findings.md`), not by a red test.

Change: stage S3a of the org-io commit workflow, in org-node — a revoked
Device receives only a notice with an absence proof; a node deletes an
Organisation's data only on a revocation it verified against the chain, after
signing acknowledgements; every commit keeps its Change set; `reconcile`
compares a record with the chain after a crash; the member-sender rule.
Branched from `master` at `a57d760` (merge base).
Plan: `docs/plans/2026-10-06-org-io-commit-workflow.md` (T0–T12b, the
robustness fixes, the review round 1 fixes).

## The gate

Measured on: HEAD `4baab33` (gate round 5, final, 2026-10-07) — tree
`506017e59a63875cf328bbaca6bc4a4000c879f1`. Agent sandbox,
`CARGO_HOME=/tmp/cargo_home_fuzz`, `QUINT_HOME=/private/tmp/claude-501/quint_home`.
No figure in the table below is carried forward from an earlier round.

| Gate | Result |
| --- | --- |
| org-node `cargo test -p org-node --features app,test-support --lib --test … --test commit_workflow_errors` (the config's line, 30 targets) | pass: 307 passed, 0 failed, 0 ignored over 26 libtest targets (absences 9, admission_sender 52, chain_read_state 3, commit_paths 36, commit_workflow_errors 3, encoding_golden 4, envelope_binding 4, expected_admission 8, key_custody 10, lib 1, node_value_types 12, organisation_key 11, persona_records 16, provisional_store 5, receive_chain_reads 15, reconcile 10, revocation 20, secret_redaction 9, service_lifecycle 9, service_stories 8, store_at_rest 11, transport_handshake 6, transport_networked 1, value_types 9, verify_against_chain 25, wire_frame_bound 10); the four bolero fuzz targets each ran about 1 s with no failure |
| `quint typecheck` (protocol.qnt, ods_instances.qnt) and `quint run` of the five invariants (forkSafety, revocationSafety, revokedExcludedFromOrgSecret, tauWindow, convergence) | pass: quint 0.33.0; both typechecks exit 0; all five invariants "[ok] No violation found" |
| app `cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support …` | pass: 162 passed, 0 failed, 0 ignored over 10 targets |
| app `npm --prefix app run check` | pass: svelte-check 199 files, 0 errors, 0 warnings |
| app `npm --prefix app run test` | pass: 7 files, 44 passed |
| `GR_CONFIG=org-node/.guardrails/config.yaml check-ids.sh` (without `--allow-draft-files`; the ledgers are finalized since `02fad55`) | pass: exit 0, no findings |
| `GR_CONFIG=app/.guardrails/config.yaml check-ids.sh` | pass: exit 0, no findings |
| `GR_CONFIG=org-node/.guardrails/config.yaml check-trace.sh` | pass: exit 0 (advisories only: open problems and expectations within limits) |
| `GR_CONFIG=app/.guardrails/config.yaml check-trace.sh` | pass: exit 0 (advisories only: open problems and expectations within limits) |
| Coverage, against the class target | shortfall against class C, owner-accepted 2026-10-07 (no coverage_command for org-node or app; floors at tooth 5). Ad-hoc org-node measurement on this tree: stable 86.59% lines, 83.05% regions, 79.71% functions; nightly-2026-10-03 branches 122/146 (83.56%). revocation.rs 98.63% lines, 18/22 branches (the new equal-epoch conflict check adds 4 branches, 2 covered); reconcile.rs 95.65%, 9/10; service.rs 92.45%, 53/58; store.rs 96.57%, 17/18. Branches inside functions that never ran are not counted |
| Working tree | clean (`git status --porcelain` empty; tree unchanged from start to end) |

Gate history (earlier rounds, each on its own tree, not this table's): round 1
at `5559fe5`, org-node 292 passed; round 2 at `17888a5`, 302 passed; round 3
at `5fb20e2`, 303 passed; round 4 at `2d35892`, 306 passed.

Quint (T12 step 4): S3 changes no modelled transition — revocation content
and acknowledgements are not modelled — and the five invariants pass
unchanged. Whether to model "a revoked Device receives only a proof" is a
later model change, not this one.

Not in `verify_commands`: `cargo clippy -p org-node --features
app,test-support --all-targets -- -D warnings` exits 101 on four existing
targets (fuzz_verify_against_chain 22 errors, key_custody 4,
transport_handshake 2, value_types 1), all red before S3 and none S3 added;
booked as PR-6f3qku. The library and every S3 target are clippy-clean.

### Coverage

org-node has no `coverage_command` (class C gap recorded in its config).
Measured ad hoc on tree `17888a5` and again on `5fb20e2`:

| Measure | org-node |
| --- | --- |
| Lines (stable) | 86.61% |
| Regions (stable) | 83.00–83.03% |
| Functions (stable) | 79.71% |
| Branches (nightly-2026-10-03) | 120/142 (84.51%) |

| File | Lines | Branches |
| --- | --- | --- |
| `org-node/src/revocation.rs` | 99.31% | 16/18 |
| `org-node/src/reconcile.rs` | 95.65% | 9/10 |
| `org-node/src/service.rs` | 92.43% | 53/58 |
| `org-node/src/store.rs` | 96.57% | 17/18 |

The branch figure carries a caveat: it counts only branches that survive into the
instrumented code; branches the compiler folds away are not counted, so it is
not decision coverage in the class-C sense. The owner
ACCEPTED the coverage gap with these figures recorded, floors set at tooth 5
(2026-10-07; plan Progress row "Robustness fix 2").

### Problem-ledger delta (this change's IDs only)

- Resolved: PR-qmvj83 (T8); PR-u4c2vp (re-resolved by T12a's fix); PR-eqs4fs
  (T12b); PR-q8r32t (unreachable arms removed; review finding-6).
- Opened: PR-6f3qku (clippy over all targets, pre-S3 targets);
  PR-zf924s (in-place amendment vs supersession, pending the owner);
  PR-eqs4fs (opened and resolved here: booked at T12, resolved at T12b);
  PR-m9betm (a bound Persona with no MemberId is skipped when signing
  acknowledgements; unreachable today; review round 2 finding-2).
- Noted, stays open: PR-vt244s (org-node half done here; S3b-io's).

## Red → green

Adapted from the plan's Progress rows, not copied verbatim: one bullet per
task row (the plan's non-task rows — requirements, rulings, merges of
`master` — are left out); each bullet adds the IDs the task implements and its
commit; T5's stale note on `SigningKeypair` is replaced; the gate and coverage
figures some rows carry (T12, T12b, Robustness fix 2) are left out, the Gate
and Coverage sections giving the current ones; T12's row is shortened (its
owner rulings are listed under "Owner rulings" below).

- **T0** (—; `4768bff`): register the new test targets (no tests, so no red -> green line; per-task clippy narrowed)
- **T1** (LLR-n67aw8; `602c809`): the ten error variants. red -> green: the_ten_new_variants_are_distinct_and_name_the_organisation; stale_state_names_both_epochs_and_the_proof_refusal_names_its_cause — each watched fail (E0599, no such variant) before the variants existed, green after. Variants print `{org_id:?}` (OrgId and Epoch have no Display). Carried to T8: the app's exhaustive `OrgNodeError` match in app/src-tauri/src/events.rs needs arms for the ten variants before the app builds (T8 runs the app's verify_commands)
- **T3** (LLR-kr5t6f, LLR-gbe9bt; `9406523`): `revocation.rs`: notices and the acknowledgement's signed bytes. red -> green: notices_for_names_each_removed_device_with_a_proof_from_the_committed_record; notices_for_orders_by_member_then_slot_and_is_empty_when_nothing_was_removed; notices_for_refuses_an_uncalculated_trie; acknowledgement_signed_bytes_are_the_domain_then_the_postcard_tuple — each watched fail (E0432, no module; then the assert against a stub) before the implementation, green after. Carried to T4: a length-refusal test for `Signature64`'s decode
- **T2** (LLR-pba7yu, LLR-d9778a; `8b9e085`): `forget_organisation` and `kept_change_set` (golden store re-pinned per ruling 12). red -> green: forget_organisation_removes_everything_of_one_organisation_and_nothing_else; forget_organisation_of_an_unheld_organisation_changes_nothing; forget_organisation_of_the_last_organisation_leaves_an_empty_store — each compile-red (E0599, no method) before the implementation; kept_change_set_round_trips_through_the_store — compile-red (no field); persona_store_plaintext_is_pinned — red against the new pin before the field existed; all green after. Also touched tests/secret_redaction.rs and tests/persona_records.rs (the new field); the merge added the field to tests/revocation.rs's literal
- **T4** (LLR-js9dsu, LLR-dc45ur, LLR-378cj4; `62307d9`): the Wire message of three kinds. red -> green: the_three_kinds_round_trip_with_their_indices; malformed_revocations_and_acknowledgements_are_refused; an_acknowledgement_signature_decodes_only_at_exactly_64_bytes — each compile-red (E0533/E0599, the variants did not exist) before wire.rs changed, green after (the last one was not watched red at runtime: the length check already existed from T3); revocation_wire_message_is_pinned — re-pinned to the new layout, no red recorded. INTERIM: 17 tests `#[ignore = "S3 T8"]` (14, send_update now refuses an unlisted recipient) and `#[ignore = "S3 T9"]` (3, relabelled or forged revocations); T8 and T9 must un-ignore every one, and the gate requires 0 ignored. `revocation` module now `transport`-gated, `notices_for` stays `app`-gated
- **T5** (LLR-r7zm39, LLR-tx8ruv, LLR-r8qhky, LLR-uw7nmv, LLR-hby4jr; `d82fef6`): deciding a notice, signing the acknowledgements. red -> green: check_notice_refuses_an_unheld_organisation_then_another_device; accept_refuses_in_order_and_changes_nothing; accept_signs_then_forgets; a_missing_device_seed_is_refused_before_anything_is_deleted — each compile-red (E0432), then runtime-red against a stub, before the implementation; green after. For review: that the signing key is zeroised on drop (it is: ed25519-dalek's `zeroize` feature; since the deslop `5559fe5` signing goes through `SigningKeypair::sign`). Stub swap done with a scripted string replacement, not the editor (convention deviation, reviewed and tested)
- **T6** (LLR-5azhry; `b100492`): checking a received acknowledgement. red -> green: a_genuine_acknowledgement_verifies; acknowledgements_are_refused_in_order; an_acknowledgement_with_an_edited_field_is_refused — each compile-red (E0432), then runtime-red against a stub, before the implementation; green after
- **T6b** (LLR-5azhry; `89f9da1`): the invalid-Device-key boundary. red -> green: an_acknowledgement_whose_device_is_no_curve_point_does_not_decode — red with a valid key spliced in its place (the acknowledgement decoded), green with the non-point bytes. The `VerifyingKey::from_bytes` arm in `check_acknowledgement` is unreachable (DevicePublicKey parses its bytes); for the deslop pass: verify through `ack.device` directly and drop the redundant parse
- **T7** (LLR-d9778a, LLR-mkj4bz; `cc00353`): every commit keeps its Change set. red -> green: every_commit_keeps_the_change_set_that_produced_the_record; a_refused_commit_keeps_the_earlier_change_set; revocation_of_another_member_is_committed_not_self_deleted (extended) — each runtime-red before `commit_held` set the field, green after
- **T8** (LLR-a8z7r5, LLR-6ymd6d, LLR-8hdu9x; PR-qmvj83; `e862cb1`): commit returns its notices; send_update sends them (PR-qmvj83). red -> green: a_revoked_device_receives_only_its_notice; send_update_chooses_by_record_and_outcome — compile-red, then runtime-red against a stub with no notices, green after; the_commit_workflow_refusals_are_classified_as_receiver_errors (app) — runtime-red with the ten variants sorted as verdicts, green after. 6 of the 14 T8-ignored tests un-ignored; 8 relabelled `S3 T9` (they fail only on the receiver's interim refusal), so T9 un-ignores 11. App builds again (ten variants classified as receiver errors)
- **T9** (LLR-pt32fx, LLR-38e2kn, LLR-b27jr6, LLR-6p4pj2, LLR-jsx922, LLR-23sfdh, LLR-2vg79y, LLR-7bk6qh; `76e9548`, `39b23c9`): receive paths decide revocations and acknowledgements; one removal step. red -> green: a_revoked_node_acknowledges_then_forgets_everything; an_org_information_commit_that_removes_the_node_forgets_through_the_one_step; revocations_are_refused_without_writing; an_acknowledgement_is_checked_and_reported; a_node_that_commits_its_own_removal_forgets_the_organisation (extended); a_received_acknowledgement_emits_nothing (app) — each compile-red, then runtime-red against a stub, green after; the 11 T9-ignored tests rewritten to the design, each runtime-red against the stub, green after. 0 ignored in org-node and app. Ten S3 refusals stay receiver errors (reasoning in LLR-7bk6qh's dated note). Carried to T12: (1) the design's `CommitOutcome` gains `acknowledgements` (LLR-b27jr6 returns them) — amend the SDD shape list; (2) a founding Persona has `member_id: None` (commit_genesis never sets it), so a founder committing its own removal signs no acknowledgement — investigate and book a problem report
- **T10** (LLR-gr8x3r, LLR-fm38ww; `9e64c0d`): `reconcile`. red -> green: reconcile_reports_one_outcome_from_the_record_and_the_state; reconcile_is_behind_when_no_stored_update_matches_both_root_and_key; reconcile_commits_the_update_the_chain_holds_after_a_crash; reconcile_returns_a_refusal_of_the_commit_step_as_the_error (tightened after its first form passed the stub); reconcile_of_a_removal_signs_and_forgets; the_service_reconcile_adopts_and_saves_the_committed_record — each compile-red, then runtime-red against a stub, green after. One removal step (`removal_step`) shared by commit and receive; kept-update selection in `StoreData::held_update_for`
- **T11** (LLR-xgefn8, LLR-23sfdh; `c77728e`): source-scan absences. red -> green: revocation_and_reconcile_do_no_io; only_forget_organisation_removes_records_and_personas; forget_organisation_is_defined_once_and_called_only_from_accept_and_the_removal_step — each watched red against planted violations, green with them removed. Known limit: a removal through a renamed alias is not caught
- **T10b** (LLR-gr8x3r, LLR-fm38ww; `d2a70b1`): seeds only on the removal path; held check before the chain read. red -> green: a_commit_that_does_not_remove_this_node_never_obtains_the_device_seeds — red with the seed source called eagerly, green after; the_service_reconcile_refuses_an_unheld_organisation_without_a_chain_read — red on T10's code (OrgNotOnChain after a read), green after. LLR-gr8x3r's signature now a seed source (`impl FnOnce() -> Vec<DeviceSeed>`)
- **T10c** (LLR-tx8ruv, LLR-r8qhky; `316f43f`): `accept` takes a lazy seed source; a refused notice never reads the seeds. red -> green: a_refused_notice_never_calls_the_seed_source — red with the source called first in `accept`, green with the call just before signing
- **T12a** (LLR-2r2fha, LLR-kzgjz8, LLR-3aysup, LLR-5azhry; PR-u4c2vp; `c2bc250`): the member-sender rule (PR-u4c2vp fixed). red -> green: the_sender_refusals_are_distinct_and_name_the_organisation — compile-red; an_acknowledgement_from_another_device_is_refused_before_its_signature, the_sender_check_comes_after_the_held_lookup_and_before_the_rest — runtime-red against a stub ignoring the sender; update_relayed_by_a_non_member_after_admission_is_refused, a_revocation_relayed_by_a_non_member_is_refused, pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused_on_the_self_delete_path, a_malformed_change_set_from_a_device_outside_the_record_is_refused_before_it_is_decoded, revocation_from_an_unknown_device_leaves_the_record_in_place, revocations_are_refused_without_writing, a_held_organisations_messages_from_an_unlisted_device_cost_no_chain_read — each runtime-red against the pre-change code; an_acknowledgement_is_checked_and_reported — runtime-red against stubbed wiring; the_sender_refusals_are_classified_as_receiver_errors (app) — compile-red (E0004); all green after. Nine sweep rewrites gained a relay-refusal assertion that was not watched red on its own (the classifier refused the no-check stub); that behaviour is red-proven by the tests above
- **T12** (—; the commit that records it, task branch `worktree-org-io-commit-workflow-s3t12`): S3a close-out (no tests, so no red -> green line). PR-qmvj83 resolved; PR-u4c2vp's resolution replaced with T12a's fix; PR-vt244s noted; PR-6f3qku and PR-eqs4fs booked; verification record drafted
- **T12b** (LLR-hby4jr, LLR-b27jr6; PR-eqs4fs; `2b7f920`): `commit_genesis` binds the founder's MemberId. red -> green: a_founder_that_commits_its_own_removal_signs_its_acknowledgement — red on the unfixed code (no acknowledgement signed), green after; commit_genesis_creates_the_record_once_the_chain_carries_the_root re-pinned from `member_id: None` to the genesis MemberId. LLR-hby4jr amended
- **Robustness fix 1** (gate finding; `1ac9991`): ten LLRs gain a new test of the missing kind. red -> green, each watched fail against a temporary mutation of the code under test, green restored: check_notice_returns_the_persona_the_notice_is_for (LLR-r7zm39); an_accepted_notice_produces_a_successor_and_leaves_the_store_passed_in_unchanged (LLR-uw7nmv); a_signing_refusal_is_returned_unchanged_with_nothing_forgotten (LLR-r8qhky); a_signature_without_the_acknowledgement_domain_is_refused (LLR-gbe9bt); a_change_set_that_decodes_and_extends_the_record_is_taken (LLR-9sknpa); an_envelope_past_the_mark_is_decoded_and_verified_with_no_signature_or_sender (LLR-mcdh85); after_an_unheld_revocation_the_expected_admission_is_still_taken (LLR-38e2kn); a_revocation_from_a_listed_device_reaches_accept_on_both_paths (LLR-kzgjz8); the_self_delete_path_acts_on_an_update_and_a_removal_from_a_listed_device (LLR-3q63zv); a_commit_that_removes_another_member_forgets_nothing (LLR-23sfdh)
- **Robustness fix 2** (gate round 2; `a1f5b66`): red -> green: a_removal_that_cannot_sign_forgets_nothing (LLR-b27jr6) — red with `removal_step` ignoring the signing error, green restored. LLR-6p4pj2: abnormal seed case unreachable through the service (every bound Persona supplies its own seed), dated N/A note in its LLR
- **Deslop refactor** (—; `5559fe5`): verify acknowledgements through `DevicePublicKey`, `SigningKeypair::sign`, one `persona_device`, shared notice fixture, leaner test support. No tests added, so no red -> green line; behaviour unchanged, the suite green before and after
- **Review round 1 fixes** (LLR-d9778a, LLR-wzqqg9, LLR-js9dsu, LLR-a8z7r5; `2324cbd`, docs `c454ed5`): code findings 1, 2, 3, 7. red -> green: every_commit_keeps_the_change_set_that_produced_the_record (rewritten: a first admission keeps the admitting Change set) — red on the old code, green after; a_genesis_listing_no_leaf_with_the_personas_device_is_refused (LLR-wzqqg9) — red with the lookup falling back to the first member; an_org_information_body_with_trailing_bytes_is_refused (LLR-js9dsu) — red with leftover bytes accepted; reconcile_of_a_crash_after_removing_another_device_returns_its_notice (LLR-a8z7r5) — red with revocations emptied; all green restored. Docs findings 4, 5, 6, 8, 9: HAZ-ep6uzs and HAZ-p6xkuz residuals restated, PR-q8r32t resolved, interim key exposure stated under HAZ-45ucqx, PR-zf924s booked
- **Review round 2 fixes** (LLR-tx8ruv; PR-m9betm booked; `d15ebf4`): findings 1 and 3 fixed, 2 booked. red -> green: a_chain_state_at_the_records_epoch_with_another_root_is_a_conflict (LLR-tx8ruv) — red on the code before the fix (the seed source was called), green after; accept_signs_then_forgets rewritten to later epochs (the equal-epoch case is now a conflict by design). Stale docs fixed; PR-m9betm booked (a bound Persona with no MemberId is skipped, unreachable today)

## What was wrong, and what was built

PR-qmvj83: a revoked Device received the committed Envelope, whose Change set
carries every leaf the update upserts — with batches, other Members' leaves.
Now it receives a notice holding only its identity and an absence proof
against the chain's root (T3, T8). PR-u4c2vp: the self-delete receive path
committed updates relayed by a Device the record does not list; by owner
ruling R1 an update or revocation for a held Organisation is now acted on
only from a listed Device, checked before any chain read (T12a). PR-eqs4fs:
a founding Persona kept `member_id: None`, so a founder that committed its own
removal signed no acknowledgement; `commit_genesis` now binds the genesis
MemberId (T12b). PR-q8r32t: two unreachable `RevocationNotHeld` arms in
`receive_and_verify` are gone, a revocation leaving the receive match at once
through `receive_revocation`. Review round 1 finding-1: a first admission kept
no Change set, against REQ-uv3v5w; it now keeps the admitting one. The rest is
S3a's design: deciding a notice and signing acknowledgements before deleting
(T5, T6, T9, T10c), every commit keeping its Change set and `reconcile`
(T7, T10, T10b — the org-node half of PR-vt244s), one removal step and
source-scan absences (T9, T11).

## Owner rulings recorded by this change (2026-10-07)

- R1, member-sender rule: decision 16 of the plan; REQ-ea4qs5, RC-u7kdam,
  LLR-2r2fha, LLR-kzgjz8, LLR-3aysup.
- R1 amendment, first admission from any sender (DoS of a new joiner
  owner-accepted): decision 16 of the plan; LLR-3q63zv and LLR-u6rq4s as amended.
- Acknowledgement sender rule (sender is the Device named, no longer in the
  record, checked before the signature): decision 16; LLR-5azhry as amended.
- R2, epoch rule kept: decision 16 of the plan.
- R3 (HAZ-p6xkuz), R4 (HAZ-7ubhwz, S5 constraints), R5 (HAZ-h6b34a,
  three-peer delivery): residuals accepted, in
  `org-node/docs/risk/2026-10-07-commit-workflow.md`.
- R6 (HAZ-bedm57, chain-authority residual): accepted, in
  `org-node/docs/risk/2026-10-06-chain-authority.md`.
- Key window (HAZ-h6b34a): the revoked Device keeps its Device secret key
  until delivery to three peers ends or the Organisation's limit expires;
  residual accepted; `org-node/docs/risk/2026-10-07-commit-workflow.md`, after R5.
- Join gate (RC-2ferct, REQ-8amu2a): kept;
  `org-node/docs/risk/2026-10-06-chain-authority.md`, after the RC-2ferct note.
- Coverage gap accepted with the ad-hoc figures recorded, floors at tooth 5:
  the plan's Progress row "Robustness fix 2"; figures above.

## Review

### Round 1

Reviewed by an independent review subagent, round 1, at HEAD `02fad55` (tree
`5fb20e2`), in a review worktree. Suite it saw, verbatim:

> suite seen: org-node 303 passed 0 failed 0 ignored (26 libtest targets) + 4 fuzz ok; quint 2 typechecks + 5 invariants ok; app cargo 162 passed; svelte-check 0/0; vitest 44 passed; clippy (lib + 11 S3 targets) clean; check-ids/check-trace without --allow-draft-files exit 0 both units

Round 1 verdict, verbatim:

> verdict: not yet mergeable: finding-1 (medium, the first-admission Change set required by REQ-uv3v5w) needs a code or requirement fix and the verification record needs correcting (findings 10-14); every other finding is low, and the owner's 2026-10-07 rulings (R1, R2, the join gate) are implemented as ruled.

Findings, verbatim, each with its disposition:

**finding-1**: requirement, medium — REQ-uv3v5w (org-node/docs/requirements/2026-10-06-chain-authority.md:118-121) says that when any update commits, "one it built or one it received", the record keeps "the Change set that produced it". It exempts only genesis ("a record created by genesis keeps none"). A first admission is a received update whose Change set produced the joiner's record. Yet LLR-d9778a and the code keep None for it (org-node/src/service.rs:890, kept_change_set: None on the first-admission record). A test pins that: org-node/tests/commit_paths.rs:876, "genesis and a first admission keep none". So every new joiner fails the requirement as stated. Fix either by keeping the admitting Change set, or by amending REQ-uv3v5w to exempt first admission and stating why.
disposition: code fixed — a first admission now keeps the admitting Change set (`org-node/src/service.rs`); LLR-d9778a amended; test `every_commit_keeps_the_change_set_that_produced_the_record` (`org-node/tests/commit_paths.rs`) rewritten to assert it, watched red on the old code (`2324cbd`).

**finding-2**: code, low — commit_genesis has a new refusal (org-node/src/service.rs:444-449). When no genesis member lists the Persona's Device, it returns NoProvisionalUpdate. No requirement covers this: LLR-wzqqg9 lists only "no selected update", SeqNotEpoch and RootMismatch, and LLR-hby4jr states only the binding. No test exercises the refusal, and it reuses a variant that means something else. It is only reachable with a crafted genesis update, so it is unmarked derived behaviour.
disposition: LLR-wzqqg9 now lists the refusal; `NoProvisionalUpdate` is kept, the reason stated in the LLR; test `a_genesis_listing_no_leaf_with_the_personas_device_is_refused` (`org-node/tests/commit_paths.rs`), red with the lookup falling back to the first member (`2324cbd`).

**finding-3**: code, low — decode_body now uses postcard::take_from_bytes and refuses leftover bytes for every kind (org-node/src/transport/wire.rs:63), OrgInformation included. LLR-dc45ur and LLR-378cj4 specify this only for revocations and acknowledgements, and LLR-js9dsu does not mention it. wire_frame_bound.rs:102-120 tests leftover bytes for the two new kinds only. So the OrgInformation refusal has no requirement and no test.
disposition: LLR-js9dsu now states that leftover bytes are refused for every kind; test `an_org_information_body_with_trailing_bytes_is_refused` (`org-node/tests/wire_frame_bound.rs`), red with leftover bytes accepted (`2324cbd`).

**finding-4**: requirement, low — HAZ-ep6uzs's residual in org-node/docs/risk/2026-10-07-org-key-pair.md:34-50 was not restated after R1. It still says a non-member's message shaping a member's record is "controlled by the root match (RC-6a2dke, RC-b6mydy)". Its benefit-risk case argues that "refusing every relayed update" would leave Devices stale. The code now does refuse non-member relays for a held Organisation (service.rs:1361). RC-u7kdam's mitigates: (org-node/docs/risk/2026-10-07-commit-workflow.md:41) names HAZ-bedm57 and HAZ-p6xkuz but not HAZ-ep6uzs. What actually remains of the residual is first admission only.
disposition: documentation — HAZ-ep6uzs's residual restated after R1 (first admission only); RC-u7kdam now mitigates HAZ-ep6uzs (`c454ed5`). No test: a risk-file change.

**finding-5**: requirement, low — HAZ-p6xkuz's restated residual (org-node/docs/risk/2026-10-07-commit-workflow.md:146-164) overstates the controls. It says a wrong deletion "now needs … delivery by a Device the device's own record lists", and names "three independent conditions for any deletion: a listed sender…". But commit_update and reconcile reach removal_step with no sender at all (service.rs commit_step, org-node/src/reconcile.rs). There the precondition is the node's own held provisional update matching the chain. The statement should name both routes.
disposition: documentation — HAZ-p6xkuz's residual now names both routes to a deletion: a received message from a listed sender, and the node's own held provisional update matching the chain on commit and reconcile (`c454ed5`). No test: a risk-file change.

**finding-6**: requirement, low — PR-q8r32t is fixed by this change but left status: open (org-node/docs/problems/2026-10-07-review-findings.md:25-31). It reported two unreachable RevocationNotHeld arms in receive_and_verify: the (None, None) arm and the let … else on the first-admission branch. The diff of service.rs removes both. check-trace still lists it as UNRESOLVED-PR PR-q8r32t. It should be resolved with this change named.
disposition: documentation — PR-q8r32t resolved, naming this change (`c454ed5`). No test: the arms were unreachable, so no test could redden on them.

**finding-7**: code, low — the reconcile half of LLR-a8z7r5 is untested. The reconcile tests assert only outcome.revocations.is_empty() (org-node/tests/reconcile.rs:84). No test reconciles a committed update that removes another Device and checks its notice. A mutation that empties revocations in Reconciled::Committed would pass. This matters on PR-vt244s's crash path: without the notice, the revoked Device is never told after recovery.
disposition: test `reconcile_of_a_crash_after_removing_another_device_returns_its_notice` (`org-node/tests/reconcile.rs`), red with revocations emptied in the reconcile outcome (`2324cbd`).

**finding-8**: requirement, low — REQ-y99c9w says the node keeps no Device secret key it was given "in the store or anywhere else once the operation returns". In the interim: the service clones each seed (service.rs:1380-1385, persona.device_seed.clone()); OrgService::reconcile clones the whole successor StoreData; DeviceSeed is not wiped on drop (org-node/src/types.rs:13-17, "Wiping on drop is not done"). Copies therefore stay in freed memory. The derived ed25519 SigningKey is wiped (ed25519-dalek zeroize feature, org-node/Cargo.toml:31). REQ-y99c9w's amendment defers the zeroised transient type to S4, but neither the record's Gaps nor the risk files state this interim exposure.
disposition: documentation — the interim exposure is stated under HAZ-45ucqx, closed by S4, and not owner-accepted (`c454ed5`); also listed under Gaps below. No test: no code changed.

**finding-9**: requirement, low — the documentation checklist item on supersession fails. These items were reworded in place, with dated amendment notes, instead of being superseded: REQ-ztdza4 (meaning reversed), REQ-uxv2x2, REQ-uv3v5w, REQ-3dsweu; RC-wqgm2p, RC-b6mydy; LLR-js9dsu, LLR-pt32fx, LLR-38e2kn, LLR-6ymd6d, LLR-8hdu9x, LLR-hby4jr, LLR-5azhry. None has a supersedes:/superseded-by: pair or dual-ID verifies: annotations. The old text is quoted in each note and the plan rules "amended in place", so the tree is traceable. Still, the checklist's rule is not followed, and the owner should confirm the practice.
disposition: booked PR-zf924s for the owner's ruling on in-place amendment versus supersession (`c454ed5`); open, pending the owner.

**finding-10**: record — The gate table (docs/verification/2026-10-07-worktree-org-io-commit-workflow.md:35) says 291 passed, with per-target counts measured at ce7a9b2, before T12b and both robustness fixes. On HEAD I see 303: commit_paths 35 (record: 34), verify_against_chain 25 (23), revocation 19 (15), receive_chain_reads 15 (12), reconcile 9 (8), service_stories 8 (7).
disposition: the gate table no longer carries T12's counts; it names the tree it is measured on, and its counts are filled at gate round 4; earlier rounds are listed as history only (this commit).

**finding-11**: record — Gaps (record lines 124-125) still says "PR-eqs4fs, open: REQ-y99c9w is not met for the founder". T12b (2b7f920) fixed it and PR-eqs4fs is status: resolved (org-node/docs/problems/2026-10-07-close-out.md:59). The gate row at line 49 also says PR-eqs4fs "entered" the roll-call as open.
disposition: the stale gap and the "entered" wording are removed; PR-eqs4fs is listed as resolved in the problem-ledger delta (this commit).

**finding-12**: record — Red→green (record lines 65-85) stops at T12a. It omits T12b, robustness fix 1 (1ac9991), robustness fix 2 (a1f5b66) and the refactor 5559fe5. The T5 note "SigningKeypair has no sign" is now stale (org-node/src/keys.rs:61-64). reproduced: (lines 10-14) omits PR-eqs4fs's red test a_founder_that_commits_its_own_removal_signs_its_acknowledgement.
disposition: rows for T12, T12b, both robustness fixes, the refactor and the review round 1 fixes added; the stale T5 note replaced; `reproduced:` names PR-eqs4fs's red test (this commit).

**finding-13**: record — The record says coverage is "not measured" (lines 51 and 121). The plan's robustness-fix-2 row records an ad-hoc measurement on tree 17888a5: 86.61% lines, 83.00% regions, 120/142 branches (84.51%). It also records the owner's acceptance of the gap. The record should carry both.
disposition: the Coverage subsection carries the ad-hoc figures (trees `17888a5` and `5fb20e2`, per-file figures) and the owner's acceptance (this commit).

**finding-14**: record — The check-ids rows (lines 47-48) were run with --allow-draft-files. The ledgers have since been finalized (02fad55). I re-ran without the flag: exit 0 for both units. The problem-ledger delta should also mention PR-q8r32t (finding-6).
disposition: the check-ids rows are run without `--allow-draft-files` at gate round 4; PR-q8r32t is in the problem-ledger delta (this commit).

### Round 2

Reviewed by an independent review subagent, round 2, at HEAD `0471cee` (gate
tree `2d35892` plus the record), in a review worktree. Reviewer line and suite
it saw, verbatim:

> reviewer: independent subagent, round 2, at HEAD 0471cee (gate tree 2d35892 plus the record), review worktree

> suite seen: org-node 306 passed 0 failed 0 ignored over 26 libtest targets + 4 fuzz ok; quint 0.33.0, 2 typechecks + 5 invariants ok; app cargo 162 passed (10 targets); svelte-check 199 files 0/0; vitest 44 passed; check-ids/check-trace exit 0 both units; clippy lib + S3 targets clean; --all-targets fails exactly as PR-6f3qku states

Round 2 verdict, verbatim:

> verdict: no code or requirement finding above low; findings 1–3 are low and findings 4–5 are record corrections, so this round can be the last per the checklist, once the record is corrected.

Findings, verbatim, each with its disposition:

**finding-1**: requirement, low — `revocation::accept` refuses only `chain.epoch < record_epoch` (org-node/src/revocation.rs:154). It accepts, and deletes everything, against a chain state at the record's own epoch whose root differs from the record's root. `reconcile` refuses that same state as `ChainStateConflict` (org-node/src/reconcile.rs:39-44). RC-ub82my's reason for accepting an equal epoch ("the chain changes its root only with a new epoch", org-node/docs/risk/2026-10-07-commit-workflow.md, RC-ub82my note) assumes the two roots are equal. The test `accept_signs_then_forgets` pins the acceptance with `chain_at(4, removed_root())` against a record at epoch 4 with a different root (org-node/tests/revocation.rs:317-320). REQ-m2xh8q as written allows this. But under R3 ("the code should ideally make this failure … extremely unlikely"), the deletion path is laxer than reconcile on the one state that proves the chain disagrees with the record. It needs a forged chain state, so it is rare. Fix: refuse it as `ChainStateConflict` in `accept` and amend LLR-tx8ruv. Alternatively, state why the deletion path should accept it.
disposition: `accept` refuses an equal-epoch chain state with another root as `ChainStateConflict`, before the seed source is called or anything is deleted; new test `a_chain_state_at_the_records_epoch_with_another_root_is_a_conflict` (`org-node/tests/revocation.rs`), red on the code before the fix (the seed source was called); `accept_signs_then_forgets` rewritten to later epochs; LLR-tx8ruv amended; RC-ub82my's reasoning corrected (`d15ebf4`).

**finding-2**: code, low — `sign_acknowledgements` still silently skips a bound Persona with no MemberId: `let Some(member_id) = persona.member_id else { continue };` (org-node/src/revocation.rs:182). The removal then goes ahead with no acknowledgement and no refusal. LLR-hby4jr (as amended for PR-eqs4fs) and REQ-y99c9w require one acknowledgement per bound Persona. PR-eqs4fs's own fix outline proposed "refuse rather than skip a bound Persona with none". This state is unreachable today: `bind_persona` always sets both fields, and pre-S3 stores do not load. That is why it is low. Still, it is behaviour no requirement states, and it is the silent failure PR-eqs4fs described. Fix: refuse with a typed error instead of skipping.
disposition: booked as open problem report PR-m9betm (affects LLR-hby4jr, REQ-y99c9w); unreachable today; the fix needs a new error variant (`d15ebf4`). Listed under Gaps.

**finding-3**: code, low — stale documentation in the tree after round-1 finding-1's fix. The `OrgRecord.kept_change_set` doc comment still says "`None` on a record `commit_genesis` or a first admission creates (LLR-d9778a)" (org-node/src/store.rs:134-136). The code now keeps the admitting Change set (service.rs:893), and LLR-d9778a was amended to say so. In the design doc, the heading `## SDD-kwncn7 — The wire frame and its bound (additions)` was merged into the end of LLR-5azhry's amendment paragraph by commit b0db2ef: "…produces no successor. — The wire frame and its bound (additions)" (org-node/docs/architecture/2026-10-07-commit-workflow.md:331). LLR-dc45ur and LLR-378cj4 now sit under SDD-uck4tz's section, against the "Placement" list at lines 93-94. check-trace links LLRs by `satisfies:`, not by heading, so no gate notices. Minor: the test doc comment at org-node/tests/revocation.rs:609 still names a "`VerifyingKey::from_bytes` refusal inside the check", which the deslop removed.
disposition: documentation — the `kept_change_set` doc comment in `org-node/src/store.rs` matches LLR-d9778a; the SDD-kwncn7 heading restored on its own line; the stale test doc comment fixed (`d15ebf4`). Doc-only: no test reddens.

**finding-4**: record — line 106 says the Red → green bullets are "Copied verbatim from the plan's Progress rows". They are not. The T5 bullet replaces the plan's "SigningKeypair has no `sign`" note with a different sentence (record line 114 vs plan line 1988). The T12 bullet drops the plan row's gate figures (record line 125 vs plan line 1999). Every bullet also adds IDs the plan rows do not carry. Either say "adapted from" or copy the rows exactly.
disposition: the Red → green intro now says the bullets are adapted from the plan's Progress rows and states each way they differ: non-task rows left out, IDs and commits added, T5's stale note replaced, the gate and coverage figures of T12, T12b and Robustness fix 2 left out, T12's row shortened (this commit).

**finding-5**: record — the problem-ledger delta (record lines 96-102) lists PR-eqs4fs only under "Resolved". This change also opened it: it was booked at T12 and resolved at T12b, as the `git diff master...HEAD` of org-node/docs/problems/2026-10-07-close-out.md shows. It belongs under "Opened" as well, as "opened and resolved here".
disposition: PR-eqs4fs listed under Opened as "opened and resolved here" as well as under Resolved (this commit).

## Gaps

- Coverage: org-node has no `coverage_command`; the ad-hoc figures above fall
  short of full class-C statement and decision coverage. Owner-accepted
  2026-10-07, floors at tooth 5.
- LLR-6p4pj2's abnormal case (a bound Persona without its seed) is N/A by
  construction: unreachable through the service, every bound Persona supplies
  its own seed; dated note in the LLR. No test exercises it.
- LLR-xgefn8 is verified structurally only: the `absences` test scans source
  for IO names, so a removal or IO call through a renamed alias is not caught.
- Interim Device-key exposure until S4: seed copies are cloned and not wiped
  on drop (review finding-8), stated under HAZ-45ucqx, not owner-accepted;
  closed by S4's zeroised transient type.
- PR-6f3qku: clippy over all org-node targets is red on four pre-existing
  targets; `verify_commands` does not run clippy.
- PR-zf924s: in-place amendment versus supersession, pending the owner's ruling.
- PR-m9betm: `sign_acknowledgements` skips a bound Persona with no MemberId
  instead of refusing (against LLR-hby4jr, REQ-y99c9w); unreachable today
  (`bind_persona` sets both fields, pre-S3 stores do not load); the fix needs
  a new error variant.
