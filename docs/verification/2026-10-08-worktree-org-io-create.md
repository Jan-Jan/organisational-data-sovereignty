# Verification — S2, create org-io (2026-10-08)

One record per change, written at `merge-change` step 6b and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line, so this file is the evidence that travels with the change.

branch: worktree-org-io-create
reviewer: independent review subagents, four rounds — round 1 at `fb334b6`, round 2 at `50d4c5a`, round 3 at `36f9470`, round 4 at `2c36fac`; round 4 was the last round (all its code and requirement findings low), so the rerun ends at step 6b with no further reviewer
verdict: round 4, verbatim: "approve — every code and requirement finding is low, so by the convergence rule this is the last round; findings 1–2 to be fixed or booked, findings 3–4 recorded." Both of round 4's code and requirement findings were fixed; findings 3–4 were recorded.
reproduced: for each defect this change resolves, and how. PR-b795an (on-chain-client's coverage did not measure the writer) was not reproduced as a red test: it is a measurement, and the writer is now measured (T4; its before and after is the coverage summary). PR-k2xxaq (`OnChainReader::refresh` kept a superseded state after a failed fetch) was not reproduced: the defect was in code this change deletes with `OnChainReader` and `OrgStateCache`, and its absence is LLR-mn5c2q's test, `org_node_names_no_chain_library_and_reads_no_chain` (`org-node/tests/absences.rs`, T6, red with the preflight bin naming `on_chain_client::`). PR-zf924s (in-place amendment versus supersession) is a practice ruling, resolved by the hybrid rule and the supersessions of T10 and T10b; there is nothing to reproduce. Every review finding fixed in code was watched red before its fix; each red is recorded in that finding's disposition below.

Units touched: org-io, org-node, app, on-chain-client, org-members, person —
every unit. Impact set (`check-units.sh --impact master..HEAD`, exit 0): all
six units, each `touched`.

Change: stage S2 of the org-io roadmap creates the org-io unit and moves into
it the chain read, the app's chain-write submission, custody of the user's own
signatory key (the development `ODS_ADMIN_SEED` path) and a new own-admin
check, so that org-node takes and returns values only and the app depends on
org-io alone. Branched from `master` at `60c7d58` (merge base).
Plan: `docs/plans/2026-10-06-org-io-create.md`.

## The gate

Every figure below is measured on one tree, and the table names it, so a later
reader can re-measure the same tree instead of guessing which round produced
these numbers. **No figure here is copied forward from an earlier round.**

Measured on: `e71a12c` — `git rev-parse HEAD` — tree
`8b1c17b4b2b79caa448ddf4ef158a671ac9a738e`, from `git rev-parse HEAD^{tree}`
on a clean worktree (final gate, run 6, 2026-10-08). Agent sandbox,
`CARGO_HOME=/tmp/cargo_home_fuzz`, `QUINT_HOME=/private/tmp/claude-501/quint_home`;
`app/node_modules` symlinked for npm and removed before the working-tree check.
All 50 commands exit 0.

| Gate | Result |
| --- | --- |
| org-io `cargo test -p org-io --features test-support …` (the config's first line) | pass: 10 suites, 67 passed, 0 failed, 0 ignored |
| org-io `cargo test -p org-io --features test-support,dev-seed …` | pass: 3 suites, 11 passed, 0 failed |
| org-io `cargo build -p org-io`; clippy `-D warnings` | pass: build finished; clippy clean |
| org-node `cargo test` and Quint | pass: 25 suites, 304 passed, 0 failed, 0 ignored; quint 0.33.0, 2 typechecks pass; `quint run` of forkSafety, revocationSafety, revokedExcludedFromOrgSecret, tauWindow and convergence each "[ok] No violation found" |
| app `cargo test`; `npm run check`; `npm run test` | pass: 10 suites, 158 passed, 0 failed; svelte-check 199 files, 0 errors, 0 warnings; vitest 7 files, 44 passed |
| app release-shape `cargo check` (no features) | pass: exit 0 |
| on-chain-client, both test lines | pass: 10 suites, 73 passed; 5 suites, 39 passed |
| org-members `cargo test` and Quint | pass: 11 suites, 271 passed, 0 failed, 1 ignored (`preflight_probe`, pre-existing); 3 quint typechecks pass; quint test 63 passing; `quint run` mbtInv no violation |
| person `cargo test`; clippy | pass: 19 suites, 123 passed; clippy clean |
| `check-units.sh` | pass: "units: 6, disclaimed 6; tracked paths 700", exit 0 |
| `check-ids.sh` (without `--allow-draft-files`), all six units | pass: exit 0 |
| `check-trace.sh`, all six units | pass: exit 0, advisories only. This change's IDs entering the roll-call: org-io's REQ, HAZ, RC, SDD and LLR items; LLR-gwk4nk (org-node); REQ-uk9rw7 and LLR-tajh9d (org-node, superseding REQ-ztdza4 and LLR-js9dsu); LLR-mn5c2q (org-node, superseding LLR-65py3d). Leaving the open-problem roll-call: PR-b795an, PR-k2xxaq, PR-zf924s. Entering it: PR-jav8zn, PR-ksm2ua, PR-8geawr, PR-z3gg2b, PR-wk7zwq. Every Implements ID has a verifying test except the expected two: REQ-ysyu9g (an `expects:` item) and SDD-z85ux9 (no LLR, the accepted shortfall) |
| Coverage, against the class target | org-io below the class C target, owner-ACCEPTED 2026-10-08: lines 70.91% (468/660, floor 70), regions 69.07% (708/1025, floor 68), functions 70.34%, branches 21/32, 65.625% (floor 64) — every floor met. The accepted shortfall is the chain shell (SDD-z85ux9), unreachable defensive branches, and items no LLR owes; live-chain coverage comes later through a chopsticks lane. on-chain-client: lines 47.88% (floor 46), regions 48.75% (floor 47), branches 58/58 (floor 99) — floors met; the shortfall predates this change |
| Working tree | clean (`git status` empty) |

Totals on this tree: 1,046 Rust tests passed, 0 failed, 1 ignored; 44 vitest;
63 Quint tests; 6 Quint invariant runs, no violation. Robustness: every
Implements ID has a normal and an abnormal test or a dated robustness N/A note
(LLR-3zdw8v's six scans, LLR-n3zmt6, LLR-u2pk5y, LLR-mn5c2q); LLR-tajh9d and
LLR-gwk4nk each have one test asserting both.

### Problem-ledger delta (this change's IDs only)

- Resolves: PR-b795an (on-chain-client, T4), PR-k2xxaq (org-node, T6),
  PR-zf924s (org-node, T10; merge message `Resolves: PR-zf924s`). Each
  `status:` flips from open to resolved in `git diff master -- '*/docs/problems/*'`.
- Opens: PR-jav8zn (org-node), PR-ksm2ua (on-chain-client), PR-8geawr,
  PR-z3gg2b and PR-wk7zwq (org-io), each `status: open` with `opened: 2026-10-08`.
- Narrowed, stays open: PR-5mc4d8 (app, T9).

## Red → green

One row per ID the plan header Implements, copied from the plan's
`red -> green:` lines (Progress section, tasks T1–T12c and the review-fix
notes). A REQ, RC or SDD row is verified through the LLRs it names; its own red is
theirs.

| Item | Test | Watched red |
| --- | --- | --- |
| REQ-tg9zrn | through LLR-7pj5af, LLR-9r2bxd, LLR-rm9x4z | see those rows |
| REQ-f3eu9n | through LLR-qhyc3n, LLR-vyd5d3, LLR-c9fyun | see those rows |
| REQ-3pxa8f | through LLR-c4bktx, LLR-rgdx22, LLR-3zdw8v, LLR-u2pk5y | see those rows |
| REQ-v4tfap | `app_boundary::no_ipc_facing_type_of_the_app_carries_key_material`; through LLR-9fy622, LLR-gc6kwy, LLR-c4bktx, LLR-3zdw8v, LLR-u2pk5y | red with a planted `key: DeviceSeed` field in `PersonaDto` (review round 1, finding-2) |
| REQ-8zuka3 | through LLR-rgdx22, LLR-3zdw8v | see those rows |
| REQ-qnh9pb | through LLR-4ax2m6, LLR-9fy622, LLR-gc6kwy | see those rows |
| REQ-6fk4qj | through LLR-n3zmt6, LLR-u2pk5y | see those rows |
| RC-2xufsr | `app_boundary::no_ipc_facing_type_of_the_app_carries_key_material`; through LLR-gc6kwy, LLR-c4bktx, LLR-4ax2m6 | as REQ-v4tfap's row |
| SDD-6qz9ms | through LLR-7pj5af | see that row |
| SDD-erzj3m | through LLR-9r2bxd | see that row |
| SDD-789u6d | through LLR-4ax2m6, LLR-9fy622, LLR-gc6kwy, LLR-c4bktx, LLR-rgdx22, LLR-3zdw8v | see those rows |
| SDD-f4khqn | through LLR-qhyc3n, LLR-vyd5d3, LLR-c9fyun | see those rows |
| SDD-z3ychz | through LLR-n3zmt6, LLR-u2pk5y | see those rows |
| LLR-7pj5af | `receive_order`'s nine tests; `a_revocation_whose_read_fails_deletes_nothing`; `a_revocation_whose_read_is_refused_at_parse_deletes_nothing`; `an_acknowledgement_on_the_self_delete_path_is_decided_with_no_read`; scans `the_handle_offers_no_chain_judging_operation_outside_its_own_sequence`, `org_io_hands_out_no_bare_service`, `the_handle_returns_only_allowlisted_public_types`, `the_app_calls_no_chain_judging_operation_and_builds_no_service` | T7: compile-red (E0432), then runtime-red under a read before the chain-free phase (all nine) and a failed read taken as absence; T12b: red with a failed self-delete read taken as absence, and with a read in the Done arm; the scans as their review-finding dispositions record |
| LLR-9r2bxd | `submit_flow::each_submission_reads_the_chain_once_between_the_write_and_the_commit`; the two read-back-failure tests; `a_read_back_that_finds_no_state_is_refused_with_org_not_on_chain`; `a_read_back_that_finds_no_state_says_the_write_executed_and_nothing_was_committed`; `a_commit_refused_for_another_reason_after_a_write_says_the_write_executed_and_nothing_was_committed` | T7: red under a double read, and with a read-back failure mapped to absence; T8: founding returned Ok; T8b: the founding error carried only the refusal's text; T12c: the bare refusal "parent_seq 1 is not the on-chain epoch 2" |
| LLR-4ax2m6 | `key_custody_config::a_seed_of_64_hex_characters_parses_with_or_without_one_prefix_and_in_either_case`; `a_malformed_seed_is_refused_by_rule_and_the_error_carries_none_of_it`; `fuzz_seed_parse` | T1: compile-red (E0432, no custody functions) |
| LLR-9fy622 | `key_custody_config::a_co_signer_of_64_hex_characters_parses_to_its_account`; `a_malformed_co_signer_is_refused_by_rule_and_the_error_carries_none_of_it`; `fuzz_seed_parse` | T1: compile-red (E0432) |
| LLR-gc6kwy | `key_custody_config::each_refusal_names_its_variable_and_rule`; the two malformed-value tests; `fuzz_seed_parse` | T1: compile-red (E0432) |
| LLR-c4bktx | `signatory_key::the_key_answers_its_account_and_renders_only_its_start`; `neither_the_seed_nor_the_key_renders_any_part_of_the_secret`; `absences::the_seed_path_keeps_no_unwiped_copy_of_the_seed`; `absences::seed_bytes_offers_no_comparison_outside_test_builds` | T2: compile-red (E0432); round 1: red with `size_of::<SeedBytes>()` 32, and while `same_bytes_as` was public |
| LLR-rgdx22 | `dev_seed_env::the_development_build_reads_the_seed_once_and_none_when_unset`; `a_seed_set_but_not_utf8_is_refused_as_malformed_not_reported_unset`; `signatory_key::a_refused_connect_hands_back_the_service_it_was_given_and_says_why`; `a_build_without_dev_seed_never_asks_for_the_seed_on_a_write_or_a_read`; `a_build_without_dev_seed_says_why_the_own_admin_check_is_not_configured`; `open::open_creates_the_data_dir_and_opens_the_store_inside_it`; `open_refuses_a_data_dir_it_cannot_create_naming_it`; `open_refuses_a_store_under_another_passphrase_and_opens_no_other`; `open_sets_the_transport_mode_it_was_given` | T2: compile-red (E0432); T9b: compile-red (no `ConnectFailed`); round 1: returned `Ok(None)`, and the refused write said "set … ODS_ADMIN_SEED"; round 2: the message said only "this node holds no signatory key"; round 3: planted `create_dir` and prefixes; round 4: red with `set_transport_mode` deleted (left `Loopback`, right `Networked`) |
| LLR-3zdw8v | scans `the_seed_is_read_once_and_only_behind_dev_seed`, `the_key_holders_derive_no_debug_and_have_no_display`, `org_io_names_no_device_or_member_private_key`, `org_node_hands_out_no_stored_device_private_key`, `org_node_hands_out_no_stored_x25519_secret`, `the_handle_returns_only_allowlisted_public_types`; `submit_flow::the_view_summarises_the_public_fields_org_node_holds` | T2: runtime-red with no read and with the dev-seed cfg removed; red against a planted Debug derive and a planted `DeviceSeed` name; rounds 1–3 as their dispositions record; the view test red against a planted `OrgSummary` that dropped its Members |
| LLR-qhyc3n | `signatory_rule::the_controller_is_the_own_account_alone_or_the_threshold_one_multisig`; `a_co_signer_equal_to_the_own_account_is_not_counted_twice`; `dev_seed_env::connect_refuses_a_co_signer_equal_to_the_own_account` | T8: compile-red (E0432/E0599), then red with the duplicate check removed; round 1: connect ran past the 5 s bound |
| LLR-vyd5d3 | `signatory_rule::admin_when_the_mapped_proxy_lists_the_controller`; `not_admin_for_an_unmapped_h160_no_delegates_or_other_delegates` | T8: compile-red; the second also red with the mapped account ignored |
| LLR-c9fyun | `signatory_rule::a_node_holding_no_signatory_key_is_refused_with_not_configured`; `a_failed_signatory_read_is_an_error_never_not_admin`; `a_configured_account_listed_as_delegate_is_admin_and_delegates_are_read_only_for_a_mapped_h160` | T8: compile-red, then red with a read before the key check, with a failure taken as NotAdmin, and with delegates always read |
| LLR-n3zmt6 | `app_boundary::the_app_depends_on_org_io_and_on_no_other_unit_or_chain_library` | T9: red because `[dependencies]` lacked org-io, then red on "names org-node" |
| LLR-u2pk5y | `absences::the_handle_hands_out_no_key` | T7: red with no `src/submit.rs` and with a planted `pub` field; round 1 (rewritten): red with a planted multi-line method returning `Option<&custody::SignatoryKey>` |
| REQ-nfr3n2 | through LLR-qhjp6g, LLR-be3zv9 (moved from the app) | see those rows |
| LLR-qhjp6g | the moved `submit_flow` tests; `a_handle_with_no_chain_refuses_every_submission_at_the_write`; `an_update_org_node_does_not_hold_on_the_records_root_is_refused_before_any_write`; `an_update_for_an_organisation_the_store_does_not_hold_is_refused_as_not_held`; `absences::the_pre_write_guard_copies_no_record_and_no_held_update` | T7: compile-red, then runtime-red under mutations; T12b: red with `WriterNotConfigured` returning Ok; round 3: the altered and the stale update were each written (the held half's red; the root comparison's own red is LLR-gwk4nk's), and "no on-chain state found for org"; round 4: red with `.cloned()` planted back |
| LLR-be3zv9 | `submit_flow::a_submission_that_never_finishes_times_out_and_nothing_is_committed`; `founding_writes_the_chain_then_commits` | moved with annotations unchanged (ruling A); round 1: red on "chain write failed" |
| REQ-ysyu9g | none — an `expects:` item, moved from org-node | not applicable: an open expectation (Gaps) |
| LLR-rm9x4z | `chain_read_rule::chain_state_is_read_into_typed_values`; `a_chain_state_whose_key_org_node_refuses_is_an_error`; `present_absent_failed_and_refused_reads_are_four_different_answers` | T5: compile-red (E0432, no `org_io::chain_read`); the third also runtime-red with a failure mapped to absence |
| SDD-z85ux9 | none — the chain shell has no LLR | not applicable: the accepted coverage shortfall (Gaps) |
| REQ-8p2veg | through LLR-m3tjvp | see that row |
| REQ-v8jczx | through LLR-m3tjvp, LLR-a9bb7b | see those rows |
| SDD-rxfu6h | through LLR-m3tjvp, LLR-a9bb7b; `fuzz_signatory_set` | T3: compile-red (E0432, no `signatory_set` module) |
| LLR-m3tjvp | `signatory_set_decode::an_original_account_is_its_32_bytes`; `an_original_account_of_another_length_is_malformed`; `fuzz_signatory_set` | T3: compile-red (E0432) |
| LLR-a9bb7b | `signatory_set_decode::proxy_definitions_give_their_delegates_in_order`; `truncated_padded_or_overlong_definitions_are_malformed`; `fuzz_signatory_set` | T3: compile-red (E0432) |
| REQ-m8sgjk | through LLR-q225ws and LLR-gha5f6 (re-parented, T9) | their tests stay green with IDs unchanged; no new red |
| LLR-mn5c2q | `absences::org_node_names_no_chain_library_and_reads_no_chain` (org-node) | T6: red with the preflight bin naming `on_chain_client::` |
| REQ-uk9rw7 | `receive_chain_reads::an_update_from_an_unlisted_device_is_refused_before_any_chain_state_is_asked_for` (org-node) | T10: red under a mutation where `sender_listed` accepted all |
| LLR-tajh9d | `wire_frame_bound::exactly_three_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused` (org-node) | T10b: red with a decoder that ignored leftovers, and with index 3 decoded as a revocation |
| LLR-gwk4nk | `commit_paths::holds_provisional_matches_all_four_public_fields_and_writes_nothing` (org-node) | round 4: red with the method missing (E0599), then red with the base-root comparison removed |

## What was wrong, and what was built

What was wrong: the chain read lived in org-node behind a chain seam
(`ChainOps`, `ChainReader`) and a stale-prone cache (PR-k2xxaq); the app built
the sr25519 signing key itself and wired the chain write; on-chain-client's
coverage recipe never measured the writer (PR-b795an); and items had been
amended in place where their meaning changed (PR-zf924s). The four review
rounds then found key material reachable through the new handle: device
ed25519 seeds (round 1), stored X25519 secrets (round 2), and a received
Organisation private key through `node().endpoint().recv_one()` (round 3),
plus unwiped seed copies, a wrong remedy message in release builds, an
unchecked provisional update on the write path, and a transport mode that no
test observed.

What was built: the org-io unit. It owns the `OrgService`, the chain
connection, the reader, the writer and the signatory key, read only under the
development `dev-seed` feature and parsed by total functions into
self-wiping types. org-node takes and returns values: its chain-judging
operations take the state org-io read, and the receive paths split into a
chain-free phase, one read, and an apply. The app holds one `OrgIo` handle and
depends on no other unit. on-chain-client gains the two signatory-set reads
behind the own-admin check, and its coverage measures the writer. The handle
offers only an allowlisted `NodeView`; no device seed, X25519 secret or
signatory key leaves through it, and source scans pin each absence. A
submission is refused before any write unless org-node holds the update on the
record's root (`holds_provisional`, by reference).

## Review

One block per finding the reviewers raised, verbatim, grouped by round. Each
round's own header lines are quoted above its findings.

### Round 1

> Round 1 review of worktree-org-io-create at fb334b6 (independent subagent, 2026-10-08).
>
> verdict: changes requested. Four medium findings (1-4: device-key reach through the handle, a missing claimed verification, unwiped seed copies, a wrong remedy message in release builds) need fixing or a dated narrowing before merge.
>
> Suite counts the reviewer saw: org-io 48 + 9; org-node 303 + quint 5 invariants; app 158 + vitest 44 + svelte-check 0; on-chain-client 73 + 39; org-members 271 + 6 quint lines; person 123. check-units/check-trace/check-ids exit 0 in all six units. Coverage org-io lines 62.78%, regions 60.41%, branches 17/26.

Round 1 (at fb334b6):

**finding-1**: requirement, medium — The handle hands out device ed25519 private keys, so REQ-v4tfap and RC-2xufsr are not met. Both say org-io "shall never receive, hold or output any device ed25519 private key, handling the Persona store … only as opaque sealed bytes". In fact `OrgIo` owns the opened `OrgService` (org-io/src/lib.rs:46). `OrgIo::node()` and `node_mut()` (lib.rs:150-157), together with `pub use org_node as node` (lib.rs:17), give any caller `node().list_personas()` (org-node/src/service.rs:955). That returns `PersonaRecord` with `pub device_seed: DeviceSeed` (org-node/src/store.rs:66), and `DeviceSeed::signing_keypair()` is public (org-node/src/keys.rs:41). LLR-3zdw8v ends "so org-io holds no device or member private key", but its scan (org-io/tests/absences.rs:99-105) only checks that org-io's source never spells the type names, so it cannot see this path. Either narrow REQ-v4tfap, RC-2xufsr and LLR-3zdw8v with a dated note saying this is transitional until S4, or close the path.
disposition: fixed — `PersonaRecord.device_seed` is `pub(crate)`, and `PersonaRecord` and `StoreData` have no production `Serialize`. Test `org_node_hands_out_no_stored_device_private_key` (`org-io/tests/absences.rs`), red on the original tree: a public `device_seed` field, two production `Serialize` derives (`PersonaRecord`, `StoreData`) and `OrgEndpoint::inner()`.

**finding-2**: requirement, medium — A verification that RC-2xufsr and REQ-v4tfap claim does not exist. Both say they are verified by "a test that no IPC-facing type carries key material" (org-io/docs/risk/2026-10-08-org-io.md:58-70; org-io/docs/requirements/2026-10-08-org-io.md, under REQ-v4tfap). No test in org-io/tests reads an IPC-facing type, and no annotation anywhere names REQ-v4tfap or RC-2xufsr directly. Either write the test or withdraw the claim with a dated note.
disposition: fixed — test `no_ipc_facing_type_of_the_app_carries_key_material` (`org-io/tests/app_boundary.rs`), red with a planted `key: DeviceSeed` field in `PersonaDto`. Its annotation and its name-only reach are round 2's finding-7, booked as PR-z3gg2b.

**finding-3**: code, medium — The sr25519 seed is copied into places that are never zeroised, against the owner's rule that private keys travel only along move-only paths and are zeroised. Three copies: `std::env::var("ODS_ADMIN_SEED")` returns a plain `String` holding the full hex seed, which is dropped without being wiped (org-io/src/custody.rs:173-174); `parse_32_hex` builds the seed in a `Copy` `[u8; 32]` named `out` (custody.rs:100) and copies it into `SeedBytes`; `SignatoryKey::from_seed` copies the bytes again into a by-value array with `*seed.bytes()` (custody.rs:140). Only `SeedBytes` and schnorrkel's `SecretKey` wipe themselves. A fix is `Zeroizing<String>` and `Zeroizing<[u8;32]>`, building `SeedBytes` in place.
disposition: fixed — the seed is read into a `Zeroizing<String>` and built in place in a `Box<[u8; 32]>`. Test `the_seed_path_keeps_no_unwiped_copy_of_the_seed` (`org-io/tests/absences.rs`), red because `size_of::<SeedBytes>()` was 32. The one by-value copy inside subxt-signer's `from_secret_key` was ACCEPTED by the owner on 2026-10-08 as a dev-only third-party limit (LLR-c4bktx's residual; SOUP).

**finding-4**: code, medium — In every release build, the chain-not-configured message tells the user to set a variable that release builds never read. Built without `dev-seed`, the handle is `OrgIo::not_configured`. Every create, admit and revoke then fails with `NOT_CONFIGURED`, "chain not configured: set ODS_CHAIN_WS, ODS_CONTRACT_H160, ODS_ADMIN_SEED" (org-io/src/submit.rs:31, used at :112/:116). Every read fails with the same text (org-io/src/connect.rs:65). REQ-8zuka3 requires the report to say that the signing seed is read only by development builds. Only the startup `eprintln!` says so.
disposition: fixed — a build without `dev-seed` no longer asks for the seed. Test `a_build_without_dev_seed_never_asks_for_the_seed_on_a_write_or_a_read` (`org-io/tests/signatory_key.rs`), red because the refused write said "set … ODS_ADMIN_SEED".

**finding-5**: code, low — `node_mut()` (lib.rs:155) exposes org-node's operations that judge against the chain (`commit_update`, `commit_genesis`, `reconcile`, `prepare_*`/`apply_*`). A caller can pass them any `OrgState`, or apply a `PendingReceive` after the store has changed. `PendingReceive`'s safety rests only on a doc comment, "callers run prepare, the read and apply under one `&mut` borrow" (org-node/src/service.rs:1196-1206). Nothing enforces REQ-tg9zrn's "the state org-io read" outside `OrgIo`'s own methods, and nothing scans the app for direct calls.
disposition: fixed across rounds 1–3. Round 1: `node_mut()` replaced by `NodeBuilders`; test `the_handle_offers_no_chain_judging_operation_outside_its_own_sequence`, red on `node_mut() -> &mut OrgService`. Round 2: `ConnectFailed.service` made private and `OrgIo::open` added; test `org_io_hands_out_no_bare_service`, red on `connect.rs:55 pub service: OrgService`. Round 3: the allowlisted `NodeView`; test `the_handle_returns_only_allowlisted_public_types`, red at `36f9470` (all three in `org-io/tests/absences.rs`).

**finding-6**: code, low — A co-signer equal to the user's own account is accepted, and then the own-admin check and the writer disagree. `controller_of` de-duplicates it to `own` (org-io/src/signatory.rs:45-57). `OrgIo::connect` hands the un-de-duplicated `[own]` to `OnChainWriter` (lib.rs:83-86, 95-100). `build_dispatch_tx` then dispatches `as_multi_threshold_1` with the sender among the other signatories (on-chain-client/src/write/multisig.rs:69-75), which the chain refuses. So in that configuration LLR-qhyc3n's phrase "the account every org-io dispatch is made from" is false. `connect` should refuse that co-signer.
disposition: fixed — `connect` refuses that co-signer (`CoSignerIsOwnAccount`). Test `connect_refuses_a_co_signer_equal_to_the_own_account` (`org-io/tests/dev_seed_env.rs`), red because connect ran past the 5 s bound.

**finding-7**: code, low — `signatory_from_environment` maps every `env::var` error to `Ok(None)` (custody.rs:175). A seed that is set but not valid UTF-8 is therefore reported as `SeedNotSet`, whose message says to set ODS_ADMIN_SEED.
disposition: fixed — a seed that is set but not UTF-8 is refused as malformed. Test `a_seed_set_but_not_utf8_is_refused_as_malformed_not_reported_unset` (`org-io/tests/dev_seed_env.rs`), red because it returned `Ok(None)`.

**finding-8**: code, low — `the_handle_hands_out_no_key` (org-io/tests/absences.rs:112-119) checks only the single line that starts `pub fn` or `pub async fn`. Handle methods already span several lines (lib.rs:161-165, 171-178), so a return type of `&SignatoryKey` on a continuation line would pass. The robustness N/A note for LLR-u2pk5y is honest about the input but overstates what the scan proves.
disposition: fixed — the scan `the_handle_hands_out_no_key` (`org-io/tests/absences.rs`) was rewritten to read whole multi-line signatures; red with a planted multi-line method returning `Option<&custody::SignatoryKey>`.

**finding-9**: code, low — After a successful write, a commit that mutates memory and then fails to save is reported as "nothing committed". `commit_genesis` pushes the record and binds the Persona, then calls `save` (org-node/src/service.rs:275-289). `commit_update` behaves the same (around :447-449). A failed `std::fs::write`, which is itself mislabelled as `OrgNodeError::Chain` (org-node/src/store.rs:568), returns `Err`. `refused_after_write` (org-io/src/submit.rs:125-131) then says "nothing committed", while the in-memory record holds the commit. The underlying gap is old (LLR-ewkg85), but the new message now states it as fact.
disposition: booked as open problem report PR-jav8zn (`org-node/docs/problems/2026-10-08-save-after-mutate.md`).

**finding-10**: requirement, low — A write that times out is reported as "chain write failed; nothing committed" (submit.rs:48, :146, :181), but it may already have executed on chain. LLR-be3zv9 mandates treating it as a failure, so this is per requirement and unchanged from before. It does sit uneasily with the owner rule that a write which executed must be reported as executed. Say "outcome unknown" instead.
disposition: fixed — a timed-out write now reports "outcome unknown", and LLR-be3zv9 was clarified to say so. Test `a_submission_that_never_finishes_times_out_and_nothing_is_committed` (`org-io/tests/submit_flow.rs`), red on "chain write failed".

**finding-11**: code, low — The two signatory-set reads each resolve "latest finalised" separately (`fetch_storage_raw` calls `at_current_block()` per call; on-chain-client/src/client.rs:263-268). `original_account` and `proxy_delegates` can therefore come from different blocks. REQ-f3eu9n does not forbid this, and nothing in S2 acts on the answer.
disposition: booked as open problem report PR-ksm2ua (`on-chain-client/docs/problems/2026-10-08-signatory-reads.md`).

**finding-12**: code, low — `SeedBytes::same_bytes_as` is public in production builds (custody.rs:65). It is a test helper and a non-constant-time equality check on the secret, so it works as a guessing oracle. LLR-c4bktx says `SeedBytes` "has no public accessor". Put it behind `test-support` or remove it.
disposition: fixed — `same_bytes_as` exists only in test builds. Test `seed_bytes_offers_no_comparison_outside_test_builds` (`org-io/tests/absences.rs`), red while it was available outside test builds.

**finding-13**: record — Under the dual-ID rule, the tests that verified superseded items should name both IDs. These still name only the old one: REQ-ztdza4: org-node/tests/admission_sender.rs:88 and :1393. LLR-js9dsu: encoding_golden.rs:191 and :217; wire_frame_bound.rs:81, :100, :130 and :224; fuzz_wire_decode/fuzz_target.rs:16. LLR-65py3d: absences.rs:111. Only receive_chain_reads.rs:487, wire_frame_bound.rs:146 and absences.rs:152 carry both. The plan's convention (a new red-first test or a dated N/A note) explains why the old tests were left alone, but the checklist item as written fails.
disposition: record — dated notes on REQ-ztdza4, LLR-js9dsu and LLR-65py3d state why the older tests keep the old ID alone; the plan's Conventions note records that the owner reported the conflict between the dual-ID rule and a green test upstream.

**finding-14**: record — The ADR is still "Status: Proposed" (docs/adr/2026-10-06-org-io-unit.md:4) while the change that implements it is going to merge.
disposition: record — the ADR is Accepted 2026-10-08 (`docs/adr/2026-10-06-org-io-unit.md`).

**finding-15**: record — My org-io coverage differs from the plan's Progress row for T12c. I measured lines 62.78% (334 of 532) and regions 60.41% (502 of 831). The plan records 63.52% and 61.03% over 549 lines. The floors (62/59/64) are still met, but the recorded figures do not reproduce.
disposition: record — the coverage figures in the plan were corrected (dated note "Corrected 2026-10-08 (review round 1, finding-15)"), since superseded by later measurements; the gate table below gives this tree's figures.

### Round 2

> Round 2 review of worktree-org-io-create at 50d4c5a (independent subagent, 2026-10-08).
>
> verdict: changes requested — one medium (finding-1: X25519 secrets are reachable and production-serialisable through the handle, and REQ-v4tfap/RC-2xufsr omit them); every other code/requirement finding is low.
>
> Round 1 resolution (reviewer): fixed in code with new tests: 1, 2, 3, 4, 6, 7, 8, 10, 12. Finding 5 partly resolved (see finding-2). Booked: 9 as PR-jav8zn, 11 as PR-ksm2ua. Record fixes 13, 14 (stale "Proposed" mentions remain, finding-10), 15 (figures stale again, finding-11).
>
> Suite counts the reviewer saw: org-io 54 + 11, build and clippy clean; org-node 303 + Quint 2 typechecks + 5 invariants; on-chain-client 73 + 39; org-members 271 (1 ignored) + Quint 3 typechecks, 63 tests, mbtInv; person 123, clippy clean; app cargo 158, svelte-check 0, vitest 44. Release-shape cargo check of the app with no features and with dev-seed, and org-io with dev-seed: exit 0. check-units, check-trace and check-ids exit 0 for all six units. Coverage org-io lines 66.89%, regions 64.87%, branches 18/28.

Round 2 (at 50d4c5a):

**finding-1**: requirement, medium — The owner's rule says X25519 secrets must never be exposed, but the handle still exposes them, and REQ-v4tfap and RC-2xufsr do not state the rule. REQ-v4tfap's narrowing note says "the rule is no private key" (org-io/docs/requirements/2026-10-08-org-io.md:141). Its text and its scans cover only the sr25519 key and device ed25519 keys. `OrgIo::node()` (org-io/src/lib.rs:220) returns `&OrgService`. From it, `list_orgs()` (org-node/src/service.rs:959) yields `OrgRecord.org_private_key`, a public field (org-node/src/store.rs:217). `list_personas()` (:955) yields `PersonaRecord.member_seed`, also public (store.rs:69). Both types hand their bytes out through public `expose_secret()` (org-node/src/types.rs:28) and `x25519_keypair()` (keys.rs:27, :34). `OrgRecord` derives `Serialize` in production builds (store.rs:197), and `secret_type!` serialises the key as plain bytes. So the Organisation private key leaves org-node through the same kind of path round 1 closed for `PersonaRecord` and `StoreData`. Mitigation and history: the app's IPC scan bans `OrgRecord`, `MemberSeed` and `OrgPrivateKey` by name, and the exposure predates S2 (the app held `OrgService` directly). Remedy: either close the path as finding 1 was closed (crate-private fields, no production `Serialize` on `OrgRecord`, and a scan), or record a dated narrowing in REQ-v4tfap and RC-2xufsr that names X25519 secrets as deferred to S4.
disposition: fixed — the X25519 secret fields are `pub(crate)`, and `OrgRecord` and `ProvisionalUpdate` have no production `Serialize`. Test `org_node_hands_out_no_stored_x25519_secret` (`org-io/tests/absences.rs`), red listing three public fields and two production `Serialize` derives. The received-key path that remained is round 3's finding-1.

**finding-2**: code, low — Finding 5 is only partly closed: the app can still reach org-node's chain-judging operations without org-io's read. `ConnectFailed` hands back `pub service: OrgService` (org-io/src/connect.rs:55). The app builds `OrgService::new(store)` itself in `open_service` (app/src-tauri/src/state.rs:203-212) and destructures `ConnectFailed` (:143). The `node` re-export (lib.rs:17) gives it every chain-judging method. `the_handle_offers_no_chain_judging_operation_outside_its_own_sequence` (org-io/tests/absences.rs:212) reads only org-io/src/lib.rs, and nothing scans the app for calls to `commit_*`, `reconcile` or `apply_*`. The clause added to LLR-7pj5af and LLR-9r2bxd says "the handle offers no chain-judging org-node operation (commit, reconcile, prepare/apply) outside this … sequence". That is false for prepare: `node()` and `NodeBuilders`' `Deref` expose `prepare_receive` and `prepare_self_delete`, which take `&self` (service.rs:656, :850). Prepare is chain-free, so nothing is unsafe; the clause overclaims.
disposition: fixed — tests `org_io_hands_out_no_bare_service` (`org-io/tests/absences.rs`), red on `connect.rs:55 pub service: OrgService`, and `the_app_calls_no_chain_judging_operation_and_builds_no_service` (`org-io/tests/app_boundary.rs`), red on `state.rs:212 OrgService::new(`. The clause under LLR-7pj5af and LLR-9r2bxd was narrowed to commit, reconcile and apply.

**finding-3**: code, low — FakeChain fakes the chain write and read on the normal path with no contract test, which fails the test checklist's test-double item. `org_io::test_support::FakeChain` (org-io/src/test_support.rs:35) is a fake (genesis returns `(org_id, proxy)`, `ChainSlots`' compare-and-swap, and imitated errors such as `InvalidOrgPublicKey` at :135). It stands in for on-chain-client, which the project owns, on the normal path of every submission and receive test in org-io and the app. No contract test ties it to `OnChainWriter`, `OnChainStateReader` or the contract. LLR-ryzr8m tests the fake itself. The owner accepted that the chain shell is uncovered. The fake's fidelity is a separate gap, and it should be stated as a recorded deviation or booked.
disposition: booked as open problem report PR-8geawr (`org-io/docs/problems/2026-10-08-review.md`).

**finding-4**: requirement, low — LLR-qhyc3n's text does not match its code or its test. The item (org-io/docs/architecture/2026-10-08-org-io.md:475) says `controller_of` returns `own` "when `co_signers` is empty and otherwise `multi_account_id` of `own` and the co-signers, de-duplicated". For `co_signers == [own]` the code returns `own` (org-io/src/signatory.rs:52-54), and the test asserts exactly that: `controller_of(OWN, &[OWN]) == OWN` (org-io/tests/signatory_rule.rs:42). The code's doc comment ("no co-signer other than `own`") is the true rule; the LLR should say it.
disposition: record (ledger) — LLR-qhyc3n's text corrected to the code's rule, "no co-signer other than `own`".

**finding-5**: requirement, low — LLR-c4bktx claims more than its test checks. Its Test paragraph (architecture ledger, under LLR-c4bktx) says the `Debug` renderings contain no four-character window "of the seed's hex or of the secret key's hex". `neither_the_seed_nor_the_key_renders_any_part_of_the_secret` (org-io/tests/signatory_key.rs:40-48) checks only windows of the seed's hex. The expanded schnorrkel secret key is never compared.
disposition: record (ledger) — LLR-c4bktx's Test paragraph narrowed to the seed's hex, which is what the test checks.

**finding-6**: requirement, low — The own-admin check's "not configured" report in a release build does not say that the seed is read only by development builds, as REQ-8zuka3 requires of every not-configured report. `OrgIo::is_own_admin` returns `OwnAdminError::NotConfigured`, which reads "this node holds no signatory key" (org-io/src/signatory.rs:33). `SignatorySetNotConfigured` says only "chain not configured" (:124, :128). Nothing in S2 calls it (owner ruling). It matters at S3b-io.
disposition: fixed — the own-admin check's "not configured" report says the seed is read only by development builds. Test `a_build_without_dev_seed_says_why_the_own_admin_check_is_not_configured` (`org-io/tests/signatory_key.rs`), red because the message said only "this node holds no signatory key".

**finding-7**: code, low — `no_ipc_facing_type_of_the_app_carries_key_material` is annotated `verifies: REQ-v4tfap, RC-2xufsr` (org-io/tests/app_boundary.rs:97). That breaks the "lowest level present" rule: LLR-3zdw8v and LLR-u2pk5y sit under REQ-v4tfap, and no LLR states the IPC property. It is also an org-io test that verifies another unit's source (the app's). It finds carriers only by name. It does not recurse into the field types of a DTO, so a type alias or a nested non-DTO struct holding `OrgPrivateKey` would pass. It does not see hand-written `Serialize` impls either.
disposition: booked as open problem report PR-z3gg2b (`org-io/docs/problems/2026-10-08-review.md`).

**finding-8**: record — Under LLR-rgdx22, the T9b clarification lists `ConnectFailure`'s variants as `DevSeedNotBuilt`, `SeedNotSet`, `Config`, `Chain` (org-io/docs/architecture/2026-10-08-org-io.md, LLR-rgdx22 notes). It omits `CoSignerIsOwnAccount`, which round 1's fix added (org-io/src/connect.rs:29).
disposition: record — LLR-rgdx22's note lists `CoSignerIsOwnAccount` among `ConnectFailure`'s variants.

**finding-9**: record — LLR-3zdw8v's robustness N/A note (org-io/docs/architecture/2026-10-08-org-io.md:454) names only the three T2 scans of org-io's own source. The item now also claims, and `org_node_hands_out_no_stored_device_private_key` checks, the clause about org-node's surface. The note should cover that scan, with the planted violation it was watched red against.
disposition: record — LLR-3zdw8v's robustness N/A note covers `org_node_hands_out_no_stored_device_private_key` and the planted violation it was watched red against.

**finding-10**: record — Stale "Proposed" mentions of the ADR, which is now Accepted: org-io/docs/requirements/2026-10-08-org-io.md:5 says "`docs/adr/2026-10-06-org-io-unit.md` (Proposed)"; the plan header (docs/plans/2026-10-06-org-io-create.md:33) says "**ADR:** … (Proposed; …)".
disposition: record — the stale "Proposed" mentions in the requirements ledger and the plan header now say Accepted 2026-10-08.

**finding-11**: record — org-io's coverage changed after the round-1 fixes, and the plan's Progress section still records the pre-fix figures. Measured with `make coverage-org-io` and `make coverage-branch-org-io` (both exit 0): lines 66.89% (396 of 592); regions 64.87% (591 of 911); branches 18 of 28, 64.29%, against a floor of 64. The plan's corrected note records 62.78% of 532 lines and 17 of 26 branches. The branch margin is now one outcome.
disposition: record — the plan's coverage note was brought to the measurement at `929c828` (dated note "review round 2, finding-11"); the gate table below gives this tree's figures.

**finding-12**: record — PR-jav8zn points at `org-node/src/store.rs:568` for the mislabelled `std::fs::write`, which is now at :683 (org-node/docs/problems/2026-10-08-save-after-mutate.md:8).
disposition: record — PR-jav8zn's line reference for the mislabelled `std::fs::write` corrected (dated in the item).

**finding-13**: record — `.guardrails/scripts/prune-plans.sh`, which merge-change step 1 runs, does not exist in this tree, and docs/plans/2026-10-06-org-io-create.md (3194 lines) is unpruned. The record (6b) must list it as "left whole", and step 1 as written cannot run here.
disposition: record — the plan `docs/plans/2026-10-06-org-io-create.md` is left whole: `.guardrails/scripts/prune-plans.sh` does not exist in this tree, so merge-change step 1's prune cannot run. Listed under Gaps.

### Round 3

> Round 3 review of worktree-org-io-create at 36f9470 (independent subagent, 2026-10-08).
>
> verdict: changes requested — one medium (finding-1: the Organisation private key is reachable through `node().endpoint().recv_one()`); every other code and requirement finding is low.
>
> Round 2 resolution (reviewer): finding 1 fixed for stored secrets (received-key path remains, finding-1 below); finding 2 fixed; findings 3 and 7 booked as PR-8geawr and PR-z3gg2b; findings 4, 5, 8, 9, 10, 12 record notes present and correct; finding 11 figures reproduced exactly (65.57% lines, 63.44% regions, 18/28 branches); finding 6 fixed in code with a test, record note missing (finding-6 below); finding 13 to be recorded at 6b (plan unpruned, 3202 lines).
>
> Suite counts the reviewer saw: org-io 58 + 11, build and clippy clean; on-chain-client 73 + 39; person 123, clippy clean; org-members 271 (1 ignored), Quint 3 typechecks, 63 tests, mbtInv; org-node 303, Quint 2 typechecks, 5 invariants; app cargo 158, svelte-check 0/0, vitest 44. Release-shape cargo check of the app and of org-io with no features: exit 0. check-trace and check-ids exit 0 in all six units. Coverage org-io lines 65.57% (400/610), regions 63.44% (597/941), branches 18/28.

Round 3 (at 36f9470):

**finding-1**: code, medium — The Organisation private key can still leave org-node through org-io in production builds. This breaks the round-2 clarification of REQ-v4tfap, RC-2xufsr and LLR-3zdw8v ("no public function … returns … an `OrgPrivateKey`"; "the output path … is closed"). The path: `OrgIo::node()` (org-io/src/lib.rs:240) returns `&OrgService`. `OrgService::endpoint(&self)` (org-node/src/service.rs:137) is not test-gated. `OrgEndpoint::recv_one(&self)` (org-node/src/transport/endpoint.rs:336) is public. It returns `(DevicePublicKey, WireMessage)`. `WireMessage::OrgInformation { org_private_key: OrgPrivateKey, .. }` (org-node/src/transport/wire.rs:25) is a public variant field. `expose_secret()` is public (org-node/src/types.rs:28). So once the receiver has bound the endpoint, `io.node().endpoint().unwrap().recv_one()` hands the caller the next epoch's Organisation private key. It also takes the message out from under org-io's receive sequence. The tests already use this path for sending: receive_order.rs:264 and :359 call `b.node().endpoint().unwrap().send(...)`. `org_node_hands_out_no_stored_x25519_secret` misses it because it matches the return type's text for `OrgPrivateKey` and similar names, and `WireMessage` is not among them. Remedy: make `endpoint()` test-only, or make `recv_one`/`send` crate-private or test-only, and extend the scan to cover `WireMessage`. Alternatively, record a dated narrowing to S4.
disposition: fixed as a class — the handle offers an allowlisted `NodeView`; `endpoint`, `recv_one` and `send` are not reachable in production builds, and `node_for_test` exists only under `test-support`. Test `the_handle_returns_only_allowlisted_public_types` (`org-io/tests/absences.rs`), red at `36f9470` listing `node() -> &OrgService`, `NodeBuilders`' `Deref`, and `admit_member`/`revoke_member` returning `ProvisionalUpdate`.

**finding-2**: requirement, low — `submit_commit_send` writes whatever `ProvisionalUpdate` value it is given (org-io/src/submit.rs:191-212). It never checks that org-node holds that update with `base_root` equal to the record's root. `ProvisionalUpdate` is `Clone + Deserialize`, and its `resulting_root` and `org_pub_key` fields are public (org-node/src/store.rs:399-418). So a caller-altered or stale update is written at the record's current epoch, and the compare-and-swap passes. `commit_update` then refuses with `NoProvisionalUpdate` (service.rs:440-445, `held_update_for`). The chain is left holding a root that no node holds. A stale update built before a revocation would put the revoked Member's root back on chain. REQ-nfr3n2 says "a provisional update built by org-node", and nothing enforces that. The app does not trigger this today: it passes the value `admit_member`/`revoke_member` just returned, under one lock (invitation.rs:368-369, commands.rs:303-304). That is why this is low. Remedy: refuse before writing unless org-node holds the update.
disposition: fixed — submission takes a `BuiltUpdate` and is refused before any write unless org-node holds the update on the record's root. Test `an_update_org_node_does_not_hold_on_the_records_root_is_refused_before_any_write` (`org-io/tests/submit_flow.rs`), red at `36f9470`: the altered and the stale update were each written. Round 4's finding-2 refined which half of the guard that red belongs to.

**finding-3**: requirement, low — Behaviour with no org-io requirement: `OrgIo::open` (org-io/src/lib.rs:138-149) creates the data directory and opens `persona_store.bin` with the "create data_dir" and "open store: " refusals. No org-io LLR describes it. Only the app's LLR-85zque does, and its item text still says `assemble` (app/docs/architecture/2026-10-05-decomposition.md:144). Its tests run in the app. `submit_commit_send` has two refusals before the write with no LLR and no test (submit.rs:200-206): a genesis update ("a genesis update is founded, not submitted"), and an Organisation the store does not hold. The second is reported as `OrgNotOnChain`, which misleads: the record is missing locally, not on chain. org-node has `OrgNotHeld` for this. The code was moved unchanged from the app.
disposition: fixed — the missing record is reported `OrgNotHeld`; `OrgIo::open` gains its clauses (LLR-rgdx22, with LLR-85zque) and tests in `org-io/tests/open.rs`. Tests `an_update_for_an_organisation_the_store_does_not_hold_is_refused_as_not_held`, red reporting "no on-chain state found for org"; `open_creates_the_data_dir_and_opens_the_store_inside_it` and `open_refuses_a_data_dir_it_cannot_create_naming_it`, red against a planted `create_dir` with the prefix "mkdir"; `open_refuses_a_store_under_another_passphrase_and_opens_no_other`, red against the planted prefix "store: ".

**finding-4**: requirement, low — REQ-f3eu9n and LLR-vyd5d3 count any delegate equal to the controller as admin. They ignore the definition's proxy type and delay: `decode_proxy_delegates` drops both (on-chain-client/src/write/signatory_set.rs:53-62). A delegate with a restricted proxy type, or a non-zero delay, cannot dispatch `revive.call` through the proxy at once, yet it is reported `Admin`. Nothing acts on the answer in S2. S3b-io should settle the rule before relying on it.
disposition: booked as open problem report PR-wk7zwq (`org-io/docs/problems/2026-10-08-review.md`), for S3b-io.

**finding-5**: record — No red-first evidence is recorded for three of the four round-2 tests: `org_io_hands_out_no_bare_service` (absences.rs:240), `the_app_calls_no_chain_judging_operation_and_builds_no_service` (app_boundary.rs:175) and `a_build_without_dev_seed_says_why_the_own_admin_check_is_not_configured` (signatory_key.rs:114). Only the X25519 scan has a "watched red" note (architecture ledger :526). The plan's Progress section has no rows for the round-2 fixes (5351490, fc2fa16). Each would have failed at 50d4c5a by reading of the old code, but the record does not say so.
disposition: record — the plan's Progress section has a dated note with the red-first evidence of the round 1 and round 2 fix tests.

**finding-6**: record — The finding-6 fix (fc2fa16) changed `OwnAdminError::NotConfigured`'s `Display` and `SignatorySetNotConfigured`'s reads. No ledger note records it. LLR-c9fyun says only `Err(OwnAdminError::NotConfigured)`. The new test is annotated LLR-rgdx22, but that item's text (including the round-1 clarification, "every write and read the unconfigured handle refuses") does not cover the own-admin check's report. LLR-rgdx22's Test paragraph does not list the new test.
disposition: record — dated notes on LLR-c9fyun and LLR-rgdx22 record the `Display` change, and LLR-rgdx22's Test paragraph lists the new test.

**finding-7**: record — The robustness N/A note for LLR-3zdw8v (architecture ledger :530-532) covers four of its five scans. The fifth, `org_node_hands_out_no_stored_x25519_secret`, has its red recorded in the clarification at :526-528 but is not named in the N/A note.
disposition: record — LLR-3zdw8v's robustness N/A note names the fifth scan, `org_node_hands_out_no_stored_x25519_secret`.

**finding-8**: record — Under LLR-7pj5af and LLR-9r2bxd, the round-2 narrowing says the handle offers no "commit, reconcile and apply" outside the sequence. Through `node()` it still offers `endpoint()`, whose `recv_one` consumes a message without org-io's sequence (finding-1). It is not chain-judging, but the narrowing note should name it, or it should be closed.
disposition: resolved by the LLR-7pj5af/LLR-9r2bxd clarification: with round 3's finding-1 fixed, `endpoint()` is no longer reachable through the handle, and the clarification says so.

### Round 4

> Round 4 review of worktree-org-io-create at 2c36fac (independent subagent, 2026-10-08).
>
> verdict: approve — every code and requirement finding is low, so by the convergence rule this is the last round; findings 1–2 to be fixed or booked, findings 3–4 recorded.
>
> Round 3 resolution (reviewer): finding-1 resolved as a class (NodeView allowlist; node_for_test only under test-support; structural allowlist scan; outcome types checked by hand; SignatoryKey::keypair pub(crate)); finding-2 resolved (BuiltUpdate; pre-write refusal unless held on the record's root); finding-3 resolved (OrgNotHeld; OrgIo::open clause and tests); finding-4 booked as PR-wk7zwq; findings 5–7 done in the record with one gap (finding-3 below); finding-8 resolved by the LLR-7pj5af/LLR-9r2bxd clarifications.
>
> Suite counts the reviewer saw: org-io 65 + 11, build and clippy clean; on-chain-client 73 + 39; person 123, clippy clean; org-node 303, Quint 2 typechecks, 5 invariants; org-members 271 (1 ignored), Quint 3 typechecks, 63 tests, mbtInv; app cargo 158, svelte-check 0/0, vitest 44. Release-shape cargo check of the app and of org-io with no features: exit 0. check-trace and check-ids exit 0 in all six units. org-io coverage lines 71.26% (476/668), regions 69.16% (711/1028).

Round 4 (at 2c36fac):

**finding-1**: requirement, low — Nothing tests that the clause of LLR-rgdx22 added in round 3 (`OrgIo::open` returns a handle "whose service has `transport_mode` set") is met. The code sets the mode at org-io/src/lib.rs:153, and nothing observes it. `OrgService::new` defaults to `Loopback` (org-node/src/service.rs:117), and every test opens with `Loopback` (open.rs:35/41/59/65, app state.rs:194 via for_test). Deleting `service.set_transport_mode(transport_mode)` would leave the suite green. Production would then ignore `ODS_TRANSPORT=networked`, and every networked send would fail with "Loopback send requires the peer's EndpointAddr". Remedy: open with `TransportMode::Networked` and assert the mode, through a test-only accessor or a send that takes the networked path.
disposition: fixed — test `open_sets_the_transport_mode_it_was_given` (`org-io/tests/open.rs`, through a test-only `OrgService::transport_mode()`), red with `set_transport_mode` deleted from `OrgIo::open` (left `Loopback`, right `Networked`). LLR-rgdx22 has a dated note.

**finding-2**: code, low — Part of the new pre-write guard has no test that isolates it, and its pre-write check copies Organisation private keys. Both are at org-io/src/submit.rs:219-220. Untested part: the `|| update.base_root != record.root_hash` half of the guard. Every path that moves a record's root also calls `discard_orphans` (org-node/src/service.rs:287, :477, :797). So the "stale" case in `an_update_org_node_does_not_hold_on_the_records_root_is_refused_before_any_write` (submit_flow.rs) is refused by the `held` test, not by the root comparison. Removing the comparison leaves the suite green. The clause is defensive and probably unreachable; the ledger's "watched red" note for the stale case is true only of the `held` half. Key copies: `svc.provisional_updates(org_id)` clones every held `ProvisionalUpdate`, including the `ProvisionalChange::ChangeSet` `org_private_key`, only to compare four public fields. `OrgPrivateKey` is `Clone` and is never wiped (org-node/src/types.rs:13-17). This adds unwiped X25519 secret copies, against the "keys only move" ruling. The `.cloned()` of the whole `OrgRecord` at :214, moved from the app, does the same. Remedy: add an org-node query that compares by reference, e.g. `holds_provisional(org_id, base_root, resulting_root, org_pub_key) -> bool`, and read proxy, epoch and root by reference.
disposition: fixed — `OrgService::holds_provisional` compares by reference (new LLR-gwk4nk); org-io's pre-write guard reads by reference and clones no record or held update. Tests `holds_provisional_matches_all_four_public_fields_and_writes_nothing` (`org-node/tests/commit_paths.rs`), red with the method missing (E0599) and again with the base-root comparison removed; `the_pre_write_guard_copies_no_record_and_no_held_update` (`org-io/tests/absences.rs`), red with `.cloned()` planted back. The root-comparison half of the guard is defensive and unreachable today; LLR-qhjp6g's dated note says so.

**finding-3**: record — LLR-3zdw8v now has six source scans, but its robustness N/A notes (org-io/docs/architecture/2026-10-08-org-io.md:678-682) cover five ("four of the item's five scans … the fifth"). The sixth, `the_handle_returns_only_allowlisted_public_types` (absences.rs:497), added in round 3 under LLR-3zdw8v, LLR-7pj5af and LLR-9r2bxd, is named in no N/A note. This is the same gap as round-3 finding-7. The N/A would be honest: it is a source absence with no input.
disposition: record — LLR-3zdw8v's N/A note names the sixth scan, `the_handle_returns_only_allowlisted_public_types`.

**finding-4**: record — The plan does not record round 3's code fix (3f0bfed). The Progress notes in docs/plans/2026-10-06-org-io-create.md (:3195-3229) have no round-3 entry for its red-first tests: `the_handle_returns_only_allowlisted_public_types`, `the_view_summarises_the_public_fields_org_node_holds`, the two new submit_flow refusals and the three open.rs tests. Their red evidence is only in the ledger. The plan's latest coverage note is still round 2's (65.57% / 63.44%, 18/28 branches). The Makefile states round 3's figures, which I reproduced: 71.26% / 69.16%.
disposition: record — the plan's Progress section has the round 3 entry (its red-first tests and coverage) and the round 4 entry.

## Gaps

- Coverage: org-io is below the class C target (lines 70.91%, regions
  69.07%, branches 21/32). Owner-accepted 2026-10-08: the chain shell,
  unreachable defensive branches and items no LLR owes are uncovered;
  live-chain coverage comes later through a chopsticks lane. on-chain-client's
  shortfall predates this change.
- The chain shell (SDD-z85ux9: preflight, connect, the `OnChain*` readers and
  writer) has no LLR and no test of its own.
- Open problem reports from this change: PR-jav8zn (a failed save after an
  in-memory commit is reported "nothing committed"), PR-ksm2ua (the two
  signatory-set reads may come from different blocks), PR-8geawr (`FakeChain`
  has no contract test), PR-z3gg2b (the IPC boundary property has no LLR and a
  name-only scan), PR-wk7zwq (proxy type and delay ignored by the own-admin
  rule).
- The plan `docs/plans/2026-10-06-org-io-create.md` is left whole:
  `.guardrails/scripts/prune-plans.sh` does not exist in this tree.
- REQ-ysyu9g is an open expectation (`expects:`), met by no test in this change.
- The app's own `revoke_and_send` still reports `OrgNotOnChain` for an
  Organisation it does not hold, where org-io now reports `OrgNotHeld`; a
  known low item.
- The X25519 private keys and the device private keys stay in org-node's
  memory until S4; they no longer leave through the handle, but they are not
  wiped there.
