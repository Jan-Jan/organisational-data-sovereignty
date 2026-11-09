# Verification — S3b-io part 2: the revoked Device's side, on the domain API (2026-11-09)

One record per change, written at `merge-change` step 5 and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line and carries the gate's figures in its own trailers; this
file is the rest of the evidence that travels with the change.

branch: worktree-org-io-publish-2
reviewer: four fresh subagent reviewers, one per scope (org-node; org-io; app; on-chain-client and repository-level files), at `merge-change` step 4 on commit b97eff4, each with the skill's review checklist, the diff, the plan and the claimed items, and with no implementation narrative
verdict: merge review round 1 is the last review round: no `code` or `requirement` finding above low in any scope (org-node: "No high or medium findings; every code and requirement finding is low"; org-io: "Mergeable for org-io … All findings are low or record, so this is the last review round"; app: "Mergeable after this round … all low or record"; on-chain-client and root: "No code findings … By the severities this is a last-round result"). The mechanical lows were fixed by fixers m-n, m-o, m-a and m-c; MR1-N1 was opened as PR-gg4zwz; three items were recorded for the owner: the `.guardrails/units.yaml` text (MR1-C2), the org-members Makefile coverage list (MR1-C9), and the mismatch between the skill's review checklist and this tree's guardrails version (MR1-N7, MR1-O7). Every finding is dispositioned below.
reproduced: not applicable for most of the change, which is a feature and tidying change (keep and show acknowledgements, the delivering Device first, the dial by key, the own-admin check removed); there is no defect to reproduce outside these. PR-y6ay44: the fallback path in `Acknowledgement::signed_bytes` was removed; its test `signed_bytes_are_the_domain_then_the_tuple_at_each_varint_length` (org-node/tests/revocation.rs) pins the bytes at each varint length and was watched red at "epoch 128" against a planted `while value > 0x80`. PR-m9betm: already refused before this change (domain-api's fix), and pinned by `a_removal_of_a_persona_with_no_member_id_is_refused_with_nothing_deleted` (org-node/tests/own_removal.rs), watched red against a planted `persona.member_id.unwrap_or(MemberId::new([0; 32]))`. The Q1 (c) design conflict was reproduced by task o1 (the dial by key failed `Connect("No addressing information available")` against the direct-only bind of LLR-cqr26z) and fixed by task q1r, whose test `networked_a_recipient_with_no_kept_address_is_dialled_by_its_key` (org-io/tests/acknowledgements.rs) was red against the direct-only bind.

Units touched: org-node, org-io, app, on-chain-client (class C, no per-item
override). on-chain-client is touched by ledger notes (owner ruling R3) and by
`verifies:` re-annotations of seven tests (T18's Q6 sweep); no
on-chain-client source changed.

Change: a Member keeps each verified acknowledgement it receives from a Device
it holds as revoked and the app shows the Admin which removed Devices
acknowledged; a removed Device sends its acknowledgements first to the Device
it learned the revocation from, and in Networked mode dials a recipient with
no usable address once by its key alone; org-io's unused own-admin check is
removed; the held-as-revoked residual is recorded; PR-y6ay44 is fixed and
PR-m9betm pinned.
Plan: `docs/plans/2026-10-16-org-io-publish-2.md` (§0 rescope, §4 owner
rulings, §5 tasks with their dispatch reports, "Conformance round 1",
"Conformance round 2", "Merge review — round 1, the last round").

## Change IDs

Implements (the plan header's **Implements:** line, and every in-force item
this change's ledger files `*/docs/*/2026-11-02-publish-2.md` define,
LLR-spckm6 among them, minted in task q1r):
- **org-node** — REQ: REQ-h7y5ur, REQ-f8pu8a, REQ-32cp7f; RC: none; SDD: SDD-acnpu4; LLR: LLR-xudkh4, LLR-py43n3, LLR-dsjht5, LLR-f5wrjx, LLR-gb7p7c, LLR-6fsnma.
- **org-io** — REQ: REQ-xghnh3, REQ-kn8zg6; RC: none; SDD: none new; LLR: LLR-3ydu9e, LLR-zhdc9f, LLR-u2j6u8, LLR-spckm6.
- **app** — REQ: REQ-yvzef9; RC: RC-z8cptv (mitigates HAZ-2hqt34); SDD: SDD-g9ugu7; LLR: LLR-ce4hxz, LLR-dmfvv6, LLR-cs9bmw, LLR-x9qbhu.
- **on-chain-client** — none.

Resolves:
- PR-m9betm (org-node) — resolved in T13b by reading the code (the skip is gone: acknowledgements are refused, not skipped); pinned in T18 and by task n1.
- PR-y6ay44 (org-node) — fixed in T18 (owner ruling Q4 of 2026-11-03).
- PR-w88sr9 (org-node) — obsolete (owner ruling 1 of 2026-11-08): change `worktree-org-node-org-key-pair` removed `WireMessage::genesis_snapshot`.

Accepts:
- app PR-5mc4d8, PR-h6xpnh, PR-j9f6kk, PR-w5dae4 (owner ruling 2 of 2026-11-08: to be fixed in a later app change) and PR-eecx3y (owner ruling of 2026-11-09, like the other four).
- org-io PR-8geawr (owner ruling of 2026-11-03; its fix, a contract test against the real chain, waits for the chopsticks or live-chain lane).

Opens:
- PR-gg4zwz (org-node) — merge review MR1-N1: `OrgService::removed_devices` lists a Device the current record lists again, and nothing refuses re-admitting a revoked Device key.

Retires (owner ruling 6 of 2026-10-16; plan §7 and T13b):
- org-io REQ-ntkjk6, LLR-qhyc3n, LLR-2hkqv2, LLR-7qdjqn, LLR-bwerd2.

Supersedes (each `supersedes:` / `superseded-by:` pair this change wrote):
- org-node: LLR-d925uu by LLR-f5wrjx; LLR-vq5d5n by LLR-6fsnma; LLR-5wg8cu by LLR-xudkh4; LLR-r4qtqe by LLR-gb7p7c.
- org-io: LLR-4usfu2 by LLR-zhdc9f; LLR-3w22wc by LLR-u2j6u8; LLR-cqr26z by LLR-spckm6 (owner ruling Q1' of 2026-11-04).
- app: LLR-kk8gc7 by LLR-ce4hxz.
- Plan: §10 of `docs/plans/2026-10-08-org-io-publish.md` (part 1's plan), whose T16c and T16d are dropped and whose other part 2 tasks are reshaped here.

Out of scope, by owner ruling Q4 of 2026-11-03: PR-7ud5fv, left to a change of
its own that supersedes LLR-zk7fx2. PR-vt244s moved to change
`worktree-org-io-domain-api` (owner ruling of 2026-10-16) and was resolved
there.

## Owner rulings that bound this change

From §0 (2026-11-02, after domain-api merged as 7ac0825 and was merged into this branch as c1bf729; domain-api carried ruling Q4', which superseded the 2026-10-16 deletion design):
- **R1 Scope:** all four — T16g, T16h, C4, and the ledger tidying (T18).
- **R2 C4:** the removed Device picks up to three recipients: one is the Device it received the revocation from (the epoch's sender, or the Member that answered with the notice), the rest from its list of last-known Organisation Devices.
- **R3 O2:** keep on-chain-client's signatory-set read and its tests, each with a dated note "no consumer since 2026-11-02; kept for the several-Admins stage".
- **R4 O4:** re-point `dev_seed_env.rs`'s co-signer refusal test to LLR-rgdx22 with a dated note on LLR-rgdx22.

From §4 (2026-11-03; the owner accepted every recommendation):
- **Q1 (c):** no Wire change; in Networked mode an acknowledgement whose recipient has no kept address that may be dialled is dialled once by the recipient's key alone (REQ-kn8zg6, LLR-u2j6u8, superseding LLR-3w22wc); Loopback mode is unchanged and its residual stands.
- **Q2:** no app requirement for `OrgIo::acknowledgement_report` (LLR-rav7dr) in this change; it stays org-io's, exercised by its tests.
- **Q3:** an acknowledgement from a Device the Member does not hold as revoked is accepted (REQ-b462sh unchanged) and nothing is kept (REQ-h7y5ur, LLR-xudkh4).
- **Q4:** PR-y6ay44 is fixed in T18 and resolved by this change; PR-7ud5fv is out of scope.
- **Q5:** the ratings and residuals as drafted: app HAZ-2hqt34 S2/P2 with RC-z8cptv S2/P1, acceptable; the LLR-bgj2ff residual acceptable until roadmap S5; org-io's Loopback residual under Q1 (c) acceptable; no rating of HAZ-h6b34a or HAZ-7ubhwz changes.
- **Q6:** re-point the merged green tests of superseded items to their successors, each as *inherited* (never red-first evidence); each successor also gets its own red-first test.
- PR-8geawr accepted (2026-11-03); org-io's `problem_age_days` raised from 30 to 60 (2026-11-03).

Later rulings:
- **Q1' (2026-11-04),** after task o1 found Q1 (c) unreachable as designed: in Networked mode only, a recipient with no usable address is dialled once from an endpoint bound with the iroh relay server and address lookup (D7 allows the relay server; only a forwarding Device is barred). LLR-cqr26z is superseded for that case only (LLR-spckm6); every dial that has an address keeps the direct-only bind.
- **2026-11-08 (task stl, docs only):** PR-w88sr9 resolved as obsolete; app PR-5mc4d8, PR-h6xpnh, PR-j9f6kk and PR-w5dae4 accepted; the relay and lookup operator's view of a dial by key stays a risk note under REQ-kn8zg6 (`org-io/docs/risk/2026-11-02-publish-2.md`), with no new hazard, to be revisited when self-hosted relays are planned.
- **2026-11-09:** app PR-eecx3y accepted like the other four app reports.

## The gate

The gate's figures are not in this record. The squash commit carries them as
trailers (`merge-change` step 7): `Gate-tree:` names the tree the step 6a gate
measured, a `Gate:` line gives each command's totals, and `Base:` names the
base and the commit step 1 merged. `finish-merge.sh` refuses the cleanup
unless `Gate-tree:` is the commit's own tree.

## Red → green

One row per ID this change implements. A REQ, RC or SDD is verified through
the LLRs that satisfy or implement it; its own red is theirs. The subagent
running each task loop is the only party that saw these tests fail; the rows
summarise, and the lines below the table copy verbatim, every `red -> green:`
and `inherited:` line of the plan.

| Item | Test | Watched red |
| --- | --- | --- |
| `LLR-xudkh4` | `an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed`, `an_acknowledgement_from_a_device_not_held_as_revoked_is_accepted_and_not_kept` (org-node/tests/receive_path.rs) | compile red: no `acknowledged`, no `KeptAcknowledgementFrom`; the first, strengthened by m-n, red against `member_id: ack.member_id` |
| `LLR-py43n3` | `an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo` (org-node/tests/own_removal.rs), and the kept-once test above | compile red; re-red against a planted `acknowledged.clear()`; strengthened by m-n, red with the TODO(S5) above `AcknowledgedEntry` removed |
| `LLR-dsjht5` | `removed_devices_lists_each_device_held_as_revoked_with_its_acknowledgement` (receive_path.rs), `the_removed_devices_of_a_removed_store_are_organisation_removed` (own_removal.rs) | compile red: no `removed_devices`, no `RemovedDeviceView`; n2: red against `FoundingNotWritten` for `OrgNotHeld`, and against `Ok(vec![])` for a removed store |
| `LLR-f5wrjx` | `store_data_holds_its_fourteen_fields_in_order_with_the_notices_senders_and_the_acknowledged_record`, `store_data_holds_no_provisional_updates_field` (org-node/tests/epoch_store.rs); `unseal_refuses_an_invalid_field_of_each_epoch_state_kind_naming_it` (persona_records.rs) | compile red: no `acknowledged`; n2: red against a planted `provisional_updates` field; m-n: red against the KeptNotice parse naming "org.revoked_devices" |
| `LLR-gb7p7c` | `a_notice_check_notice_passes_is_kept_and_nothing_is_deleted` (receive_path.rs), `the_delivering_device_of_the_accepted_notice_is_the_first_recipient` (own_removal.rs; written as `the_sender_of_the_accepted_notice_is_the_first_recipient`) | compile red: `KeptNotice` did not exist; left `[all[0], all[1], all[2]]` right `[all[3], all[0], all[1]]` |
| `LLR-6fsnma` | `the_device_a_match_came_from_is_the_first_recipient`, `the_delivering_device_of_the_accepted_notice_is_the_first_recipient` (own_removal.rs) | left `[all[0], all[1], all[2]]` right `[all[4], all[0], all[1]]` |
| `REQ-h7y5ur`, `REQ-f8pu8a`, `REQ-32cp7f`, `SDD-acnpu4` | through LLR-xudkh4, LLR-py43n3, LLR-f5wrjx (REQ-h7y5ur); LLR-dsjht5 (REQ-f8pu8a); LLR-gb7p7c, LLR-6fsnma (REQ-32cp7f); SDD-acnpu4 traces REQ-h7y5ur and REQ-f8pu8a | as their LLRs |
| `LLR-3ydu9e` | `the_admin_lists_a_removed_device_and_then_its_acknowledgement` (org-io/tests/acknowledgement_inbound.rs) | compile red: no `removed_devices`, `RemovedDevice`, `RemovedDevicesRefused`; strengthened by m-o, red against four mutants (a fixed Epoch(1), a wrong MemberId, a chain read in the query, a store write in the query) |
| `LLR-zhdc9f` | `the_handle_offers_none_of_the_removed_methods` (org-io/tests/absences.rs; replaced `org_io_offers_no_own_admin_check`) | red: `src/signatory.rs is removed`; re-red, and red for its replacement, against a planted `pub fn is_own_admin` |
| `LLR-u2j6u8` | `networked_a_recipient_with_no_kept_address_is_dialled_by_its_key`, `networked_a_recipient_whose_only_kept_address_does_not_parse_is_dialled_by_its_key`, `networked_a_recipient_whose_kept_addresses_all_fail_is_not_dialled_by_its_key` (org-io/tests/acknowledgements.rs) | first written as `a_recipient_with_no_dialable_address_is_dialled_by_key_in_networked_mode_only`, red on "one dial by key" (left `[]`), deleted by o1; then red (NoAddress) with the Networked branch of `dial_plan` disabled; red against a `dial_plan` mutant that always appends the dial by key |
| `LLR-spckm6` | `networked_a_recipient_with_no_kept_address_is_dialled_by_its_key`, `networked_a_failed_relay_bind_is_the_recipients_outcome_and_the_next_round_binds_again`, `networked_after_a_failed_relay_bind_the_other_recipients_are_dialled_as_before` (acknowledgements.rs) | red against the direct-only bind (`NotDelivered { Transport(Connect("No addressing information available")) }`); red with src reverted (timed out waiting for `SenderEndpoint`); red (left 0, right 1) against a mutant ending the round after a relay-bind failure |
| `REQ-xghnh3`, `REQ-kn8zg6` | through LLR-3ydu9e, LLR-zhdc9f (REQ-xghnh3); LLR-u2j6u8, LLR-spckm6 (REQ-kn8zg6) | as their LLRs |
| `LLR-ce4hxz` | `product_registers_the_same_commands_as_the_harness` (app/src-tauri/tests/ipc.rs) | compile red: no `commands::list_removed_devices` (17 ≠ 18 as planned) |
| `LLR-dmfvv6` | `list_removed_devices_refuses_an_unheld_or_malformed_organisation`, `the_removed_devices_answers_have_the_fakes_shapes`, `list_removed_devices_answers_two_devices_in_org_ios_order` (ipc.rs) | compile red: no `commands::list_removed_devices`; a2: red against a planted `.rev()` |
| `LLR-cs9bmw` | removed.view.test.ts 'words an acknowledgement as the Device statement, never as deletion', 'says so when no Device is removed, shows a refusal as its text, and neither shows a deletion word', 'while the first answer is in flight it says nothing of the list, then shows it' | red: `RemovedDevices.svelte` and `removed.ts` did not exist; red against planted deletion words; red with `loaded &&` removed |
| `LLR-x9qbhu` | removed.view.test.ts 'reads the list when shown, on Refresh, and after a verification result for its Organisation only', 'a refused subscription is shown on the panel, and the rows still are', 'an earlier read answered after a later one is dropped, so the latest answer stays shown', and others below | red: no component; red before the C2-A3 fix; red before the `latestRead` counter |
| `REQ-yvzef9`, `RC-z8cptv`, `SDD-g9ugu7` | through LLR-dmfvv6, LLR-cs9bmw, LLR-x9qbhu (REQ-yvzef9, SDD-g9ugu7); RC-z8cptv's wording is LLR-cs9bmw's, tested there | as their LLRs |
| `PR-y6ay44` | `signed_bytes_are_the_domain_then_the_tuple_at_each_varint_length` (org-node/tests/revocation.rs) | red at "epoch 128" against a planted `while value > 0x80` |
| `PR-m9betm` | `a_removal_of_a_persona_with_no_member_id_is_refused_with_nothing_deleted` (own_removal.rs) | green on arrival; red against a planted default MemberId |

Every `red -> green:` and `inherited:` line of the plan, copied verbatim and
grouped by where the plan records it. Lines naming tests later renamed,
replaced or deleted are kept as written: the observation was made.

#### T16h — the plan's task text

```
  red -> green: `org-io/tests/absences.rs::org_io_offers_no_own_admin_check (verifies: LLR-zhdc9f)` — red: `src/signatory.rs is removed`; green after steps 2–6; re-red against a planted `pub fn is_own_admin`.
  inherited: `dev_seed_env.rs::connect_refuses_a_co_signer_equal_to_the_own_account` — from LLR-qhyc3n (to LLR-rgdx22, ruling R4).
  inherited: `absences.rs::the_handles_production_methods_are_exactly_the_domain_surface` — from LLR-4usfu2 (to LLR-zhdc9f).
```

#### T16c4 — the plan's task text

```
  red -> green: `org-node/tests/receive_path.rs::a_notice_check_notice_passes_is_kept_and_nothing_is_deleted (verifies: LLR-gb7p7c)` — compile red: no `KeptNotice`. (Rewritten for the successor; its old body is inherited coverage, the kept-form assertions the new evidence.)
  red -> green: `org-node/tests/own_removal.rs::the_device_a_match_came_from_is_the_first_recipient (verifies: LLR-6fsnma)` — red: left `[all[0], all[1], all[2]]`.
  red -> green: `org-node/tests/own_removal.rs::the_sender_of_the_accepted_notice_is_the_first_recipient (verifies: LLR-6fsnma, LLR-gb7p7c)` — red: left `[all[0], all[1], all[2]]`.
  red -> green: `org-io/tests/acknowledgements.rs::a_recipient_with_no_dialable_address_is_dialled_by_key_in_networked_mode_only (verifies: LLR-u2j6u8)` — red: `one dial by key`, left `[]`.
  inherited: `own_removal.rs::a_removal_by_a_match_keeps_three_two_one_or_no_recipients`, `the_still_listed_come_first_on_a_match_and_all_ascending_on_a_notice` — from LLR-vq5d5n; `acknowledgements.rs::a_round_tries_each_recipients_addresses_in_order_until_accepted`, `a_recipient_with_no_address_of_its_own_is_not_delivered_for_no_address` — from LLR-3w22wc.
```

#### T16c4 — dispatch report (done 2026-11-03)

```
  red -> green: org-node/tests/receive_path.rs::a_notice_check_notice_passes_is_kept_and_nothing_is_deleted (verifies: LLR-gb7p7c) — red: compile error, KeptNotice did not exist
  red -> green: org-node/tests/own_removal.rs::the_device_a_match_came_from_is_the_first_recipient (verifies: LLR-6fsnma) — red: left [all[0], all[1], all[2]] right [all[4], all[0], all[1]]
  red -> green: org-node/tests/own_removal.rs::the_sender_of_the_accepted_notice_is_the_first_recipient (verifies: LLR-6fsnma, LLR-gb7p7c) — red: left [all[0], all[1], all[2]] right [all[3], all[0], all[1]]
  red -> green: org-io/tests/acknowledgements.rs::a_recipient_with_no_dialable_address_is_dialled_by_key_in_networked_mode_only (verifies: LLR-u2j6u8) — red: no acknowledgement_dial_plan, then "one dial by key" left [] right [[]]
  inherited: own_removal.rs::a_removal_by_a_match_keeps_three_two_one_or_no_recipients — from LLR-vq5d5n (to LLR-6fsnma)
  inherited: own_removal.rs::the_still_listed_come_first_on_a_match_and_all_ascending_on_a_notice — from LLR-vq5d5n (to LLR-6fsnma)
  inherited: acknowledgements.rs::a_round_tries_each_recipients_addresses_in_order_until_accepted — from LLR-3w22wc (to LLR-u2j6u8)
  inherited: acknowledgements.rs::a_recipient_with_no_address_of_its_own_is_not_delivered_for_no_address — from LLR-3w22wc (to LLR-u2j6u8)
```

#### T16g — the plan's task text

```
  red -> green: `org-node/tests/receive_path.rs::an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed (verifies: LLR-xudkh4, LLR-py43n3)` — compile red: no `acknowledged`.
  red -> green: `org-node/tests/receive_path.rs::an_acknowledgement_from_a_device_not_held_as_revoked_is_accepted_and_not_kept (verifies: LLR-xudkh4)` — compile red: no `acknowledged`.
  red -> green: `org-node/tests/receive_path.rs::removed_devices_lists_each_device_held_as_revoked_with_its_acknowledgement (verifies: LLR-dsjht5)` — compile red: no `removed_devices`.
  red -> green: `org-node/tests/own_removal.rs::an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo (verifies: LLR-py43n3)` — compile red; then re-red against a planted `acknowledged.clear()` in `prune_after_adoption`.
  red -> green: `org-node/tests/epoch_store.rs::store_data_holds_its_fourteen_fields_in_order_with_the_notices_senders_and_the_acknowledged_record (verifies: LLR-f5wrjx)` — compile red: no `acknowledged`.
  red -> green: `org-io/tests/acknowledgement_inbound.rs::the_admin_lists_a_removed_device_and_then_its_acknowledgement (verifies: LLR-3ydu9e)` — compile red: no `removed_devices`.
  red -> green: `app/src-tauri/tests/ipc.rs::product_registers_the_same_commands_as_the_harness (verifies: LLR-ce4hxz)` — red: 17 ≠ 18.
  red -> green: `app/src-tauri/tests/ipc.rs::list_removed_devices_refuses_an_unheld_or_malformed_organisation (verifies: LLR-dmfvv6)` — red: unknown command.
  red -> green: `app/tests/removed.view.test.ts` ×3 (verifies: LLR-cs9bmw; LLR-cs9bmw; LLR-x9qbhu) — red: no component.
  inherited: `receive_path.rs::an_acknowledgement_is_decided_by_check_acknowledgement_with_the_sender_and_no_store_change` — from LLR-5wg8cu; `epoch_store.rs` ×3 — from LLR-d925uu; `ipc.rs` ×3 — from LLR-kk8gc7.
```

#### T16g — dispatch report (done 2026-11-04)

```
  red -> green: org-node/tests/receive_path.rs::an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed (verifies: LLR-xudkh4, LLR-py43n3) — compile red: no `acknowledged`, no `KeptAcknowledgementFrom`
  red -> green: org-node/tests/receive_path.rs::an_acknowledgement_from_a_device_not_held_as_revoked_is_accepted_and_not_kept (verifies: LLR-xudkh4) — compile red: no `acknowledged`
  red -> green: org-node/tests/receive_path.rs::removed_devices_lists_each_device_held_as_revoked_with_its_acknowledgement (verifies: LLR-dsjht5) — compile red: no `removed_devices`, no `RemovedDeviceView`
  red -> green: org-node/tests/own_removal.rs::an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo (verifies: LLR-py43n3) — compile red; re-red against a planted `acknowledged.clear()` (left [] right [entry])
  red -> green: org-node/tests/epoch_store.rs::store_data_holds_its_fourteen_fields_in_order_with_the_notices_senders_and_the_acknowledged_record (verifies: LLR-f5wrjx) — compile red: no `acknowledged`
  red -> green: org-io/tests/acknowledgement_inbound.rs::the_admin_lists_a_removed_device_and_then_its_acknowledgement (verifies: LLR-3ydu9e) — compile red: no `removed_devices`, `RemovedDevice`, `RemovedDevicesRefused`
  red -> green: app/src-tauri/tests/ipc.rs::product_registers_the_same_commands_as_the_harness (verifies: LLR-ce4hxz) — compile red: no `commands::list_removed_devices`
  red -> green: app/src-tauri/tests/ipc.rs::list_removed_devices_refuses_an_unheld_or_malformed_organisation (verifies: LLR-dmfvv6) — compile red: no `commands::list_removed_devices`
  red -> green: app/src-tauri/tests/ipc.rs::the_removed_devices_answers_have_the_fakes_shapes (verifies: LLR-dmfvv6) — compile red: no `commands::list_removed_devices`
  red -> green: app/tests/removed.view.test.ts ×3 (verifies: LLR-cs9bmw; LLR-cs9bmw; LLR-x9qbhu) — red: RemovedDevices.svelte and removed.ts did not exist (import failed)
  inherited: receive_path.rs::an_acknowledgement_is_decided_by_check_acknowledgement_with_the_sender_and_no_store_change — from LLR-5wg8cu (to LLR-xudkh4)
  inherited: receive_path.rs::an_acknowledgement_is_decided_in_the_check_from_its_named_device_only — from LLR-5wg8cu (to LLR-xudkh4)
  inherited: receive_path.rs::a_held_organisations_messages_from_an_unlisted_device_are_refused_before_use — from LLR-5wg8cu (to LLR-xudkh4)
  inherited: receive_chain_reads.rs::an_acknowledgement_is_checked_and_reported — from LLR-5wg8cu (to LLR-xudkh4)
  inherited: epoch_store.rs::store_data_holds_its_fields_in_encoding_order_and_round_trips_through_the_sealed_store — from LLR-d925uu (to LLR-f5wrjx)
  inherited: epoch_store.rs::store_data_holds_no_provisional_updates_field — from LLR-d925uu (to LLR-f5wrjx)
  inherited: epoch_store.rs::an_outstanding_invite_keeps_its_invitee_name — from LLR-d925uu (to LLR-f5wrjx)
  inherited: ipc.rs::list_personas_returns_an_empty_list_on_a_fresh_store — from LLR-kk8gc7 (to LLR-ce4hxz)
  inherited: ipc.rs::the_harness_registers_exactly_the_eighteen_commands_and_refuses_the_old_names — from LLR-kk8gc7 (to LLR-ce4hxz)
  inherited: ipc.rs::every_backend_entry_has_a_real_producer_here — from LLR-kk8gc7 (to LLR-ce4hxz)
```

#### T18 — the plan's task text

```
  red -> green: `own_removal.rs::sign_acknowledgement_refuses_a_persona_with_no_member_id (verifies: LLR-5tvn7g, PR-m9betm)` — green on arrival; red against the planted default MemberId.
  red -> green: `revocation.rs::signed_bytes_are_the_domain_then_the_tuple_with_no_fallback (verifies: LLR-bs6yxf, PR-y6ay44)` — red: `no fallback path in signed_bytes`.
```

#### T18 — dispatch report (done 2026-11-04)

```
  red -> green: org-node/tests/own_removal.rs::sign_acknowledgement_refuses_a_persona_with_no_member_id (verifies: LLR-5tvn7g, PR-m9betm) — green on arrival (domain-api's fix); watched red on the OrgNotHeld assertion against a planted `persona.member_id.unwrap_or(MemberId::new([0; 32]))`, plant removed
  red -> green: org-node/tests/revocation.rs::signed_bytes_are_the_domain_then_the_tuple_with_no_fallback (verifies: LLR-bs6yxf, PR-y6ay44) — red: "no fallback path in signed_bytes"; green after signed_bytes builds ACK_DOMAIN + tuple by hand (push_varint), equal to postcard's tuple encoding at epoch 300
  inherited: org-io/tests/absences.rs::the_handle_returns_only_allowlisted_public_types — from LLR-4usfu2 (to LLR-zhdc9f)
  inherited: org-node/tests/receive_path.rs::a_kept_notice_counts_against_the_bound — from LLR-r4qtqe (to LLR-gb7p7c)
  inherited: org-io/tests/acknowledgements.rs::already_received_or_invite_used_keeps_the_acknowledgement — from LLR-3w22wc (to LLR-u2j6u8)
  inherited: org-node/tests/encoding_golden.rs::persona_store_plaintext_is_pinned_in_its_active_form — from LLR-d925uu (to LLR-f5wrjx)
  inherited: org-node/tests/value_types.rs::the_first_admission_refusals_name_their_organisation — from LLR-mxskg9 (dropped; successor LLR-3sddey does not state AdmissionNotExpected/AdmissionNotOurs; REQ-8amu2a, REQ-kt877x stay)
  inherited: app/src-tauri/tests/ipc.rs::connection_status_is_registered_under_its_name — from LLR-kk8gc7 (to LLR-ce4hxz)
  inherited: app/src-tauri/tests/ipc.rs::unknown_command_is_rejected — from LLR-kk8gc7 (to LLR-ce4hxz)
  inherited: app/src-tauri/tests/receiver_events.rs::a_chain_failure_emits_no_verification_event_for_any_message — from LLR-7bk6qh (to LLR-wgr46u)
  inherited: app/src-tauri/tests/receiver_events.rs::every_real_terminal_message_stops_the_loop — from LLR-a9rjtf (dropped; LLR-zw6jqy cited)
  inherited: app/src-tauri/tests/receiver_events.rs::a_non_terminal_failure_does_not_stop_the_loop — from LLR-a9rjtf (dropped; LLR-zw6jqy cited)
  inherited: app/src-tauri/tests/receiver_events.rs::the_stop_is_announced_after_the_failure_for_the_same_error — from LLR-a9rjtf (dropped; LLR-zw6jqy cited)
  inherited: on-chain-client/tests/decode_org_state.rs::exactly_ninety_six_bytes_decodes_every_field — from LLR-nq7nhg (dropped; LLR-hezpr7 cited)
  inherited: on-chain-client/tests/write_pure.rs::proxied_wraps_any_call_unchanged_even_another_proxied_call — from LLR-ywhd23 (dropped; LLR-59kxsr cited)
  inherited: on-chain-client/tests/write_pure.rs::proxied_forces_the_any_proxy_type_by_name — from LLR-ywhd23 (dropped; LLR-59kxsr cited)
  inherited: on-chain-client/tests/write_pure.rs::proxied_never_leaves_the_proxy_type_unforced_or_forced_by_index — from LLR-ywhd23 (dropped; LLR-59kxsr cited)
  inherited: on-chain-client/tests/type_widths.rs::newtypes_have_the_widths_the_abi_gives_them — from REQ-2qa5r5 (dropped; REQ-54txzh cited)
  inherited: on-chain-client/tests/signatory_set_decode.rs::definitions_decode_with_type_and_delay_in_stored_order — from LLR-a9bb7b (dropped; LLR-n2vx5q cited)
  inherited: on-chain-client/tests/signatory_set_decode.rs::a_malformed_definitions_value_is_refused — from LLR-a9bb7b (dropped; LLR-n2vx5q cited)
```

#### Conformance round 1 — fixer a1 (app)

```
  red -> green: app/tests/removed.view.test.ts::while the first answer is in flight it says nothing of the list, then shows it (verifies: LLR-cs9bmw) — green on arrival; red against a mutant with `loaded &&` removed
  red -> green: app/tests/removed.view.test.ts::a refused Refresh replaces the rows with its text, and the next answer replaces the text with rows (verifies: LLR-cs9bmw, LLR-x9qbhu) — green on arrival; red against a mutant keeping the refusal text after a successful load
  red -> green: app/tests/removed.view.test.ts::shows the selected Organisation's removed Devices, and reads them again for another selected (verifies: LLR-x9qbhu) — green on arrival; red with the panel removed from Revoke.svelte
  red -> green: app/tests/removed.view.test.ts::reads the list when shown, on Refresh, and after a verification result for its Organisation only (edited, verifies: LLR-x9qbhu) — new asserts red against an admission-requested subscription that loads, and against a merging (non-replacing) load
```

#### Conformance round 1 — fixer n1 (org-node)

```
  red -> green: org-node/tests/own_removal.rs::a_removal_of_a_persona_with_no_member_id_is_refused_with_nothing_deleted (verifies: LLR-5tvn7g, PR-m9betm) — green on arrival; red against a planted `persona.member_id.unwrap_or(MemberId::new([0; 32]))` (got Ok(Removed { acknowledgements_kept: 1 })); replaces sign_acknowledgement_refuses_a_persona_with_no_member_id
  red -> green: org-node/tests/revocation.rs::signed_bytes_are_the_domain_then_the_tuple_at_each_varint_length (verifies: LLR-bs6yxf, PR-y6ay44) — green on arrival; red at "epoch 128" against a planted `while value > 0x80`; replaces signed_bytes_are_the_domain_then_the_tuple_with_no_fallback
```

#### Conformance round 1 — fixer o1 (org-io)

```
  red -> green: org-io/tests/acknowledgements.rs::networked_a_recipient_with_no_kept_address_is_dialled_by_its_key (verifies: LLR-u2j6u8) — green on arrival; red (NotDelivered { reason: NoAddress }) with the Networked branch of dial_plan disabled
```

#### Conformance round 1 — task q1r (owner ruling Q1')

```
  red -> green: org-io/tests/acknowledgements.rs::networked_a_recipient_with_no_kept_address_is_dialled_by_its_key (verifies: LLR-spckm6, LLR-u2j6u8) — red against the direct-only bind (NotDelivered { Transport(Connect("No addressing information available")) }); green: the Admin's receive step reports Acknowledged
  red -> green: org-io/tests/acknowledgements.rs::networked_a_recipient_whose_only_kept_address_does_not_parse_is_dialled_by_its_key (verifies: LLR-spckm6, LLR-u2j6u8) — red the same way before the relay bind
  red -> green: org-io/tests/acknowledgements.rs::networked_a_failed_relay_bind_is_the_recipients_outcome_and_the_next_round_binds_again (verifies: LLR-spckm6) — written after the code; red with src reverted to HEAD (timed out waiting for SenderEndpoint), green restored
  inherited: org-io/tests/acknowledgements.rs::acknowledgements_go_from_the_removed_personas_own_identity_over_a_direct_bind — from LLR-cqr26z (to LLR-spckm6)
  inherited: org-io/tests/acknowledgements.rs::a_failed_direct_bind_is_every_recipients_outcome_and_the_next_round_binds_again — from LLR-cqr26z (to LLR-spckm6)
```

#### Conformance round 2 — fixer a2 (app)

```
  red -> green: app/src-tauri/tests/ipc.rs::list_removed_devices_answers_two_devices_in_org_ios_order (verifies: LLR-dmfvv6) — green on arrival; red against a planted `.rev()` in list_removed_devices
  red -> green: app/tests/removed.view.test.ts::says so when no Device is removed, shows a refusal as its text, and neither shows a deletion word (verifies: LLR-cs9bmw) — green on arrival; red against "wiped" planted in the empty branch and "erased" in the refusal branch
  red -> green: app/tests/removed.view.test.ts::a refused subscription is shown on the panel, and the rows still are (verifies: LLR-x9qbhu) — red before the C2-A3 fix (no refusal text, unhandled rejection)
```

#### Conformance round 2 — fixer n2 (org-node)

```
  red -> green: org-node/tests/own_removal.rs::the_removed_devices_of_a_removed_store_are_organisation_removed (verifies: LLR-dsjht5) — green on arrival; red against removed_devices returning Ok(vec![]) for a removed store
  red -> green: org-node/tests/epoch_store.rs::store_data_holds_no_provisional_updates_field (rewritten at the encoding seam, verifies: LLR-f5wrjx) — red against a planted provisional_updates field in StoreDataEncoding
  red -> green: org-node/tests/receive_path.rs::removed_devices_lists_each_device_held_as_revoked_with_its_acknowledgement (OrgNotHeld pinned, verifies: LLR-dsjht5) — red against removed_devices returning FoundingNotWritten
```

#### Conformance round 2 — fixer o2 (org-io)

```
  red -> green: org-io/tests/absences.rs::the_handle_offers_none_of_the_removed_methods (verifies: LLR-zhdc9f) — green on arrival; red against a planted `pub fn is_own_admin` on OrgIo; replaces org_io_offers_no_own_admin_check
  red -> green: org-io/tests/acknowledgements.rs::networked_a_recipient_whose_kept_addresses_all_fail_is_not_dialled_by_its_key (verifies: LLR-u2j6u8) — green on arrival; red against a dial_plan mutant that always appends the dial by key
  red -> green: org-io/tests/acknowledgements.rs::networked_after_a_failed_relay_bind_the_other_recipients_are_dialled_as_before (verifies: LLR-spckm6) — green on arrival; red (left 0, right 1) against a mutant ending the round after a relay-bind failure
```

#### Merge review round 1 — fixers m-a, m-n, m-o

```
  red -> green: app/tests/removed.view.test.ts::an earlier read answered after a later one is dropped, so the latest answer stays shown (verifies: LLR-x9qbhu) — red before the counter (the stale answer showed "no acknowledgement received by this Device")
  red -> green: app/tests/removed.view.test.ts::an earlier read refused after a later one was answered leaves the latest rows shown (verifies: LLR-x9qbhu) — red before the counter (the stale refusal replaced the rows)
  red -> green: org-node/tests/persona_records.rs::unseal_refuses_an_invalid_field_of_each_epoch_state_kind_naming_it (new case, verifies: LLR-7yu2xy, LLR-f5wrjx) — red against the KeptNotice parse naming "org.revoked_devices"
  red -> green: org-node/tests/receive_path.rs::an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed (strengthened, verifies: LLR-xudkh4, LLR-py43n3) — red against `member_id: ack.member_id`
  red -> green: org-node/tests/own_removal.rs::an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo (strengthened, verifies: LLR-py43n3) — red with the TODO(S5) above `AcknowledgedEntry` removed
  red -> green: org-io/tests/acknowledgement_inbound.rs::the_admin_lists_a_removed_device_and_then_its_acknowledgement (strengthened, verifies: LLR-3ydu9e) — red against four mutants: a fixed Epoch(1), a wrong MemberId, a chain read in the query, a store write in the query
```

## What was wrong, and what was built

What was wrong. After change `worktree-org-io-domain-api` merged, a Member
checked a received acknowledgement (`Checked::Acknowledged`) and kept nothing,
and the app's `AcknowledgementReceived` showed nothing, so an Admin could not
see which removed Devices had acknowledged their removal. A removed Device
chose its acknowledgement recipients by key bytes, still-listed first, so the
Device it had learned the revocation from was first only by chance; and the
acknowledgement's delivery dialled only kept addresses, so a recipient
without one was never reached. org-io still carried `OrgIo::is_own_admin`,
its `signatory` module and REQ-ntkjk6, with no caller since part 1's T16c was
dropped. Two org-node problem reports were open: PR-y6ay44
(`Acknowledgement::signed_bytes` fell back to `ACK_DOMAIN` alone if the
tuple's serialisation failed) and PR-m9betm (`sign_acknowledgements` skipped,
silently, a Persona with no MemberId; domain-api had already made it a
refusal, unpinned). While building the change, task o1 found the plan's own
design for the dial by key unreachable: LLR-cqr26z bound the delivery endpoint
direct-only, with no relay and no address lookup, so a dial by key alone
failed `Connect("No addressing information available")` with the recipient
online and in the lookup.

What was built, as the change stands at merge.

- **org-node (sans-IO).** The Persona store gains the acknowledged record
  (`StoreData.acknowledged: Vec<AcknowledgedEntry>`, last in the encoding
  order of LLR-f5wrjx): an acknowledgement `check_acknowledgement` accepts
  from a Device the store holds as revoked, that the record does not list and
  that has no entry yet, is appended once with the MemberId the Device was
  held under and sealed; any other accepted acknowledgement keeps nothing
  (owner ruling Q3). Adoption keeps the record whole; the field, its type and
  its append carry the TODO that roadmap S5's revoked-device CRDT replaces
  it, as does `revoked_devices` (LLR-bgj2ff's residual recorded).
  `OrgService::removed_devices` lists each Device held as revoked, in held
  order, with its acknowledgement or none, and changes nothing. A kept
  revocation notice now keeps the Device the transport authenticated for it
  (`KeptNotice { notice, from }`), and on its own removal a Device puts the
  delivering Device first among up to three recipients: the sender of the
  matched provisional organisation, or of the first kept notice that
  `accept` takes; an `Own` match keeps the old order (owner ruling R2).
  `signed_bytes` builds the domain and the tuple by hand with a hand-written
  LEB128 varint, so no failure path exists (PR-y6ay44).
- **org-io.** `OrgIo::removed_devices(org_id)` maps org-node's list to public
  `RemovedDevice` values, reads no chain and writes nothing, and refuses an
  Organisation no Persona holds as `NotHeld`. The acknowledgement's delivery
  dials each kept address alone, in order; in Networked mode a recipient with
  no address that may be dialled is dialled once by its key alone, from an
  endpoint bound with the iroh relay server and address lookup (owner rulings
  Q1 (c) and Q1'; every dial with an address keeps the direct-only bind); a
  failed relay bind is that recipient's outcome and the others are dialled as
  before; Loopback mode is unchanged. The own-admin check is removed whole:
  `signatory.rs`, `is_own_admin`, `with_own_account`, the signatory fields and
  fakes, `signatory_rule.rs`, and their configuration and coverage lines; the
  writer's `CoSignerIsOwnAccount` refusal stays, re-cited to LLR-rgdx22.
- **app and webview.** The eighteenth command, `list_removed_devices`, parses
  `org_id` at the IPC edge and returns org-io's list in its order as hex
  identifiers and an optional acknowledged epoch. The Revoke tab shows the
  "Removed Devices (Admin)" panel for the selected Organisation, wording an
  acknowledgement as the removed Device's own signed statement and never as
  proof of deletion (RC-z8cptv); it reads the list when shown, on Refresh and
  after a verification result for its Organisation only, shows a refusal as
  its text, and keeps only the latest read's answer.
- **on-chain-client.** No source change. The signatory-set read is kept with
  no consumer, each exported item noted (owner ruling R3); seven tests drop a
  superseded ID beside the successor they already cited.
- **Ledgers.** Problem reports resolved, accepted and opened as listed under
  "Change IDs"; superseded and retired items as listed there; each touched
  unit's risk file assesses the new derived requirements, and the relay and
  lookup operator's view of a dial by key is a risk note under REQ-kn8zg6.

## Review

The merge-change step 4 round. Each finding is copied verbatim from its
reviewer's report (quoted under its header; the reviewers' full reports
follow in the next section), renumbered within the round and prefixed with
its scope. The plan's ID for each (MR1-N…, MR1-O…, MR1-A…, MR1-C…) is named
in its header, and its disposition is the plan's. The base was unchanged
(local master 7ac0825) and there was no `shared:` line.

### Round 1

> Reviewers, one fresh reviewer per scope, on commit b97eff4. Verdicts:
> - org-node: "No high or medium findings; every code and requirement finding is low. The org-node part is fit to merge once the lows are dispositioned."
> - org-io: "Mergeable for org-io. Every claimed REQ/LLR is met and tested with abnormal cases, both checkers exit 0, and 152/152 tests pass. All findings are low or record, so this is the last review round."
> - app: "Mergeable after this round: four findings, all low or record, and every claimed REQ and LLR is met and tested, including abnormal inputs."
> - on-chain-client and repository-level files: "No code findings. In on-chain-client and the root files, every `code` or `requirement` finding is low (1–5, 8, 9), plus two `record` findings (6, 7). By the severities this is a last-round result: fix the mechanical lows, record the rest, and dispatch no further reviewer for this scope."
> The finding numbers inside these quotes are each reviewer's own.

**finding-1**: code, low — org-node (reviewer's finding-1; MR1-N1): `OrgService::removed_devices` lists every `revoked_devices` entry, but it does not check that the current record no longer lists that Device.
> finding-1. code low: `OrgService::removed_devices` (`org-node/src/service.rs:1229-1246`) lists every `revoked_devices` entry, but it does not check that the current record no longer lists that Device.
> - LLR-dsjht5 says "for each RevokedDevice the store holds as revoked (LLR-bgj2ff)". LLR-bgj2ff (`2026-10-16-domain-api.md:2043`) says "revoked exactly when `revoked_devices` holds its DevicePublicKey and the current record does not list it".
> - `revoked_devices` is never pruned (`verify_org.rs:345-355` only appends). So a Device key admitted again after its removal would still be shown to the Admin as a removed Device.
> - It is rare: the same Device key would have to be re-admitted. No test covers it.
> - Fix: filter with `!record.lists_device(&revoked.device_key)` and add an abnormal case.
disposition: opened as PR-gg4zwz (org-node, `org-node/docs/problems/2026-11-09-publish-2.md`) by fixer m-n, not fixed in this change: the dispatcher checked that no admission path reads `revoked_devices`, so nothing refuses re-admitting a revoked Device key, and the fix needs a rule first (refuse re-admission, or prune on re-admission), then a test of it as an abnormal case of LLR-dsjht5 and LLR-bgj2ff.

**finding-2**: code, low — org-node (reviewer's finding-2; MR1-N2): no test drives LLR-f5wrjx's refusal of an invalid `KeptNotice.from`.
> finding-2. code low: no test drives LLR-f5wrjx's refusal of an invalid `KeptNotice.from` (`InvalidField` naming `org.kept_revocation_notices`, `org-node/src/store.rs:959`).
> - `persona_records.rs:827-842` (`unseal_refuses_an_invalid_field_of_each_epoch_state_kind_naming_it`) added the `org.acknowledged` case but not this one. `WireStore` already holds the sender as `[u8; 32]` (`persona_records.rs:1737`), so one spoil line adds it:
>   ```rust
>   ("org.kept_revocation_notices", |s| s.kept_revocation_notices[0].1 = off_curve_key())
>   ```
disposition: fixed by m-n: `unseal_refuses_an_invalid_field_of_each_epoch_state_kind_naming_it` in `org-node/tests/persona_records.rs` gained the `org.kept_revocation_notices` case, red against the KeptNotice parse naming "org.revoked_devices".

**finding-3**: code, low — org-node (reviewer's finding-3; MR1-N3): the keep tests cannot tell `member_id` taken from the held `RevokedDevice` apart from one taken from `ack.member_id`; the TODO scan misses `AcknowledgedEntry`.
> finding-3. code low: the keep tests cannot tell `member_id` taken from the held `RevokedDevice` (which LLR-xudkh4 requires) apart from one taken from `ack.member_id`.
> - In `receive_path.rs:2030-2054` and `receive_chain_reads.rs` the two values are equal (`ack_by_dave` signs with `dave().id`, the same as the `RevokedDevice`).
> - Also, `an_adoption_keeps_..._s5_todo` (`own_removal.rs:1686-1705`) does not check the TODO on the `AcknowledgedEntry` type, which LLR-py43n3 also requires. The TODO is present (`store.rs:1105`), so this is only a gap in the test.
> - Fix: give the held `RevokedDevice` a different MemberId from the acknowledgement's.
disposition: fixed by m-n: `an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed` (`org-node/tests/receive_path.rs`) holds the revoked Device under a different MemberId, red against `member_id: ack.member_id`; `an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo` (`org-node/tests/own_removal.rs`) checks the TODO on `AcknowledgedEntry`, red with that TODO removed.

**finding-4**: requirement, low — org-node (reviewer's finding-4; MR1-N4): LLR-f5wrjx silently drops LLR-d925uu's clause "no `ProvisionalUpdate` type exists".
> finding-4. requirement low: LLR-f5wrjx (`org-node/docs/architecture/2026-11-02-publish-2.md:321-342`) silently drops LLR-d925uu's clause "no `ProvisionalUpdate` type exists" (`2026-10-16-domain-api.md:82`).
> - The supersession note on LLR-d925uu says "every other field is as stated here" and does not name the drop. The epoch_store test was rewritten at the encoding seam to match.
> - The exhaustive field list still excludes a `provisional_updates` field, so the behaviour that matters stands.
> - Fix: say in the note that the type-absence clause is dropped.
disposition: fixed by m-n: a dated note on LLR-d925uu (`org-node/docs/architecture/2026-10-16-domain-api.md`) names the dropped type-absence clause.

**finding-5**: requirement, low — org-node (reviewer's finding-5; MR1-N5): LLR-xudkh4 and LLR-f5wrjx carry rationale or history inside the item text.
> finding-5. requirement low: two items carry rationale or history inside the item text.
> - LLR-xudkh4 ends "The append is not measured by LLR-m76r4k's bound: the record holds at most one entry per entry of `revoked_devices`." Proposed: "The append is not measured by LLR-m76r4k's bound."
> - LLR-f5wrjx ends "Stores written before this change do not open (as for every store-format change since plan decision 12 of 2026-10-07)." Proposed: "A store in the format before this one does not open."
disposition: fixed by m-n: LLR-xudkh4 and LLR-f5wrjx amended in place in `org-node/docs/architecture/2026-11-02-publish-2.md`.

**finding-6**: requirement, low — org-node (reviewer's finding-6; MR1-N6): the docs say two different things about which SDD owns LLR-xudkh4.
> finding-6. requirement low: the docs say two different things about which SDD owns LLR-xudkh4.
> - SDD-busuj3's traces note (`2026-10-16-domain-api.md`, the 2026-11-03 note) says "LLR-xudkh4, under SDD-acnpu4".
> - LLR-xudkh4 sits in SDD-acnpu4's section.
> - SDD-acnpu4's amended code claim says the append "is the acknowledgement arm of `check_received`, SDD-busuj3's (LLR-xudkh4)".
> - The plan's n1 entry left this "Open for round 2's ARCH review", and round 2 did not settle it.
> - Fix: pick one owner (the code is SDD-busuj3's arm, the data is SDD-acnpu4's) and word both notes the same way.
disposition: fixed by m-n: LLR-xudkh4 belongs to SDD-busuj3 (its code is `check_received`), stated the same way in notes on SDD-acnpu4 and SDD-busuj3 (`org-node/docs/architecture/2026-11-02-publish-2.md`, `org-node/docs/architecture/2026-10-16-domain-api.md`).

**finding-7**: requirement, low — org-node (reviewer's finding-7; MR1-N7): the review checklist does not match what this tree's guardrails require.
> finding-7. requirement low: the review checklist (`~/.claude/skills/merge-change/references/review-checklist.md`) does not match what this tree's guardrails require.
> - The checklist says no item carries `supersedes:`/`superseded-by:`, a changed behaviour's old item is deleted, a resolved PR is deleted, and regression tests never verify the PR.
> - This tree is pinned to guardrails 0.5.1 (`org-node/.guardrails/config.yaml`). Its `check-trace.sh` requires reciprocal `supersedes:`/`superseded-by:` and `status: resolved` entries.
> - The diff follows the tree. I raised no finding against the diff for this, but a reviewer applying the checklist literally would.
disposition: no change — the diff follows this repository's guardrails (`check-trace.sh` requires reciprocal supersession pairs and `status: resolved`); the mismatch between the skill's review checklist and the guardrails version this tree is pinned to is recorded for the owner, with finding-15.

**finding-8**: record — org-node (reviewer's finding-8; MR1-N8): the plan header's Resolves line names PR-m9betm and PR-y6ay44 but not org-node PR-w88sr9.
> finding-8. record: the plan header's **Resolves:** line (`docs/plans/2026-10-16-org-io-publish-2.md:6`) names PR-m9betm and PR-y6ay44 but not org-node PR-w88sr9. The diff resolves PR-w88sr9 as obsolete under the owner ruling of 2026-11-08, which is recorded only at plan line 1248. The squash message's `Resolves:`/`Problems:` should name all three.
disposition: fixed by the dispatcher: the plan header's Resolves line (`docs/plans/2026-10-16-org-io-publish-2.md`) names PR-w88sr9 (obsolete, owner ruling 2026-11-08), and this record's Resolves list names all three (`docs/verification/2026-11-09-worktree-org-io-publish-2.md`, "Change IDs").

**finding-9**: code, low — org-io (reviewer's finding-1; MR1-O1): `the_admin_lists_a_removed_device_and_then_its_acknowledgement` leaves several clauses of LLR-3ydu9e unchecked.
> **finding-1. code low:** `org-io/tests/acknowledgement_inbound.rs:302-330`. `the_admin_lists_a_removed_device_and_then_its_acknowledgement` (verifies LLR-3ydu9e) leaves several clauses unchecked:
> - It matches `acknowledged_epoch: Some(_)` and `..` for `member_id`. A mutant that maps the wrong epoch, or the wrong MemberId, stays green.
> - It asserts none of the item's "reads no chain … sends nothing and writes nothing". `admin.chain.reads()` and `store_writes_to` are available, as `receive_step.rs:489` uses them.
> - Fix: assert the epoch Bob's acknowledgement names, Bob's MemberId, and unchanged chain-read and store-write counts around the query.
disposition: fixed by m-o: `the_admin_lists_a_removed_device_and_then_its_acknowledgement` (`org-io/tests/acknowledgement_inbound.rs`) asserts the epoch, Bob's MemberId, and no chain read and no store write around the query; red against four mutants (a fixed Epoch(1), a wrong MemberId, a chain read in the query, a store write in the query).

**finding-10**: requirement, low — org-io (reviewer's finding-2; MR1-O2): REQ-k6kcgd's note of 2026-11-03 lets the relay carry every send, where only REQ-kn8zg6's dial by key is relayed.
> **finding-2. requirement low:** `org-io/docs/requirements/2026-10-14-domain-api.md:371-376`. REQ-k6kcgd's note of 2026-11-03 re-reads the item's "over a direct connection with no relay" as "iroh's relay server may carry the connection".
> - That loosens the requirement for every send. The tree still makes every dial that has an address direct-only (LLR-spckm6). Only the dial by key alone, which REQ-kn8zg6 states, is relayed.
> - The note thus states a looser behaviour than the item's text and the code.
> - Proposed wording: "The dial by a recipient's key alone when no kept address may be dialled is REQ-kn8zg6's, outside this item's 'at the endpoint addresses kept with it'; every dial at a kept address stays direct with no relay."
disposition: fixed by m-o: a dated note of 2026-11-09 on REQ-k6kcgd (`org-io/docs/requirements/2026-10-14-domain-api.md`) narrows the relay to REQ-kn8zg6's dial by key; every dial at a kept address stays direct.

**finding-11**: requirement, low — org-io (reviewer's finding-3; MR1-O3): LLR-spckm6's clause "which is closed once the dial's outcome is read" is untestable.
> **finding-3. requirement low:** `org-io/docs/architecture/2026-11-02-publish-2.md:89-108`. LLR-spckm6 states about eight behaviours in one item:
> - the identity;
> - closing a relayed endpoint;
> - the direct bind;
> - the address form;
> - the direct-bind failure;
> - the relay bind for a dial by key;
> - closing after the dial;
> - the relay-bind failure;
> - Loopback never binding the relay.
>
> One of them, "which is closed once the dial's outcome is read", is untestable by the item's own note of 2026-11-04. The checklist asks for one testable behaviour per item. Proposed: drop the closing clause from the item text and keep it as design prose. This is not a request to split the item, which the conformance loop accepted.
disposition: fixed by m-o: a dated note of 2026-11-09 on LLR-spckm6 (`org-io/docs/architecture/2026-11-02-publish-2.md`) moves the closing clause to design prose; the item is not split.

**finding-12**: requirement, low — org-io (reviewer's finding-4; MR1-O4): SDD-3my66m's note cites `Endpoints::plan`, renamed `Endpoints::bind_plan`.
> **finding-4. requirement low:** `org-io/docs/architecture/2026-10-16-domain-api.md:1705`. SDD-3my66m's note of 2026-11-04 says the dial by key binds "the handle's plan (`Endpoints::plan`)".
> - Task o2 renamed it to `Endpoints::bind_plan` (`org-io/src/endpoints.rs:113`, and LLR-spckm6's own o2 note).
> - Fix: replace `Endpoints::plan` with `Endpoints::bind_plan`.
disposition: fixed by m-o: SDD-3my66m's note (`org-io/docs/architecture/2026-10-16-domain-api.md`) names `Endpoints::bind_plan`.

**finding-13**: requirement, low — org-io (reviewer's finding-5; MR1-O5): LLR-7qdjqn's deletion note names the replaced test.
> **finding-5. requirement low:** `org-io/docs/architecture/2026-10-16-domain-api.md:2345-2349`. LLR-7qdjqn's deletion note says the absence is pinned by `absences.rs` `org_io_offers_no_own_admin_check`.
> - o2 replaced that test with `the_handle_offers_none_of_the_removed_methods` (`org-io/tests/absences.rs:679`), and the exact-surface test also pins it.
> - Fix: rename it in the note.
disposition: fixed by m-o: LLR-7qdjqn's note (`org-io/docs/architecture/2026-10-16-domain-api.md`) names `the_handle_offers_none_of_the_removed_methods` and the exact-surface test that replaced the old one.

**finding-14**: record — org-io (reviewer's finding-6; MR1-O6): the plan's Progress row and header "plan draft only" are stale.
> **finding-6. record:** `docs/plans/2026-10-16-org-io-publish-2.md:1276`. The Progress table still reads "T16h, T16c4, T16g, T18, T17b | open", but §5 records T16h, T16c4, T16g and T18 as done and merged, and the conformance loop as complete.
> - The header line "This branch holds the plan draft only (docs, no code)" is equally stale.
disposition: fixed by the dispatcher: the plan's Merge plan line is corrected (MR1-O6) and its Progress table records T16h, T16c4, T16g and T18 done and T17b open (`docs/plans/2026-10-16-org-io-publish-2.md`); this record states the change as merged work (`docs/verification/2026-11-09-worktree-org-io-publish-2.md`).

**finding-15**: requirement, low — org-io (reviewer's finding-7; MR1-O7): the checklist's deletion rule against this repository's supersession rule.
> **finding-7. requirement low:** `/Users/jan-jan/.claude/skills/merge-change/references/review-checklist.md:44-47`. The checklist says an item never carries `supersedes:`, `superseded-by:` or `retired:`, and that a changed behaviour deletes the old item.
> - This tree's guardrails do the opposite. `.guardrails/scripts/check-trace.sh` (header lines 22-169) requires reciprocal `supersedes:` / `superseded-by:` and well-formed `retired:` lines.
> - This change follows the project's rule: LLR-3w22wc, LLR-cqr26z and LLR-4usfu2 are superseded, and REQ-ntkjk6 and four LLRs are retired, each in place.
> - So the checklist misstates what this tree does. The change itself is not at fault.
disposition: no change — the change follows this repository's rule as `.guardrails/scripts/check-trace.sh` enforces it; recorded for the owner with finding-7 as a skill-versus-guardrails-version mismatch.

**finding-16**: requirement, low — app (reviewer's finding-1; MR1-A1): the refused-subscription label is stated by no item.
> **finding-1** — requirement low — `app/src/lib/components/RemovedDevices.svelte:53,70-72`
>
> The panel shows `listening for verification results failed: <text>` when its subscription is
> refused. No item states this:
>
> - LLR-x9qbhu (`app/docs/architecture/2026-11-02-publish-2.md:97`) says only when the panel reads.
> - LLR-cs9bmw lists what the panel shows, and this label is not on that list.
>
> Yet the test `a refused subscription is shown on the panel, and the rows still are` carries
> `verifies: LLR-x9qbhu`. The same gap was found and fixed for LLR-5mw2dj in the domain-api change
> (TEST review round 1, finding-4; `app/docs/architecture/2026-10-16-domain-api.md:1253-1268`).
>
> Proposed wording, appended to LLR-x9qbhu: "a refused subscription is shown on the panel under the
> label `listening for verification results failed:` with its text, and the rows are still read and
> shown."
disposition: fixed by m-a: a dated note on LLR-x9qbhu (`app/docs/architecture/2026-11-02-publish-2.md`) states the refused-subscription label; the test 'a refused subscription is shown on the panel, and the rows still are' (`app/tests/removed.view.test.ts`) verifies it.

**finding-17**: requirement, low — app (reviewer's finding-2; MR1-A2): SDD-6pjvhc and the app glossary still say seventeen commands and cite LLR-kk8gc7.
> **finding-2** — requirement low — `app/docs/architecture/2026-10-16-domain-api.md:90,101-102`;
> `app/docs/CONTEXT.md:45`
>
> The body text still gives the old count:
>
> - The SDD-6pjvhc section preamble says "the seventeen command handlers (eighteen until
>   2026-10-20)".
> - SDD-6pjvhc's own sentence says "Its seventeen commands are the thirteen that call org-io's domain
>   surface".
> - The glossary's **Command** entry says "Seventeen exist (… LLR-kk8gc7)" and cites the superseded
>   item.
>
> Only the dated note added at domain-api.md:111 says eighteen. The tree has eighteen commands, and
> fourteen of them call org-io's domain surface.
>
> Proposed fix:
>
> - Amend SDD-6pjvhc's sentence in place to "Its eighteen commands are the fourteen that call org-io's
>   domain surface under its names, then …".
> - Fix the preamble to match.
> - Change the glossary entry to "Eighteen exist (… LLR-ce4hxz)" with a dated note.
disposition: fixed by m-a: SDD-6pjvhc (`app/docs/architecture/2026-10-16-domain-api.md`) and the Command glossary entry (`app/docs/CONTEXT.md`) say eighteen commands, fourteen domain calls, and cite LLR-ce4hxz.

**finding-18**: code, low — app (reviewer's finding-3; MR1-A3): overlapping loads can land out of order, a stale answer replacing a newer one.
> **finding-3** — code low — `app/src/lib/components/RemovedDevices.svelte:22-31`
>
> `load()` can be in flight several times at once, for example after a Refresh click followed by a
> `verification-result`. Each answer is assigned whenever it arrives. Tauri's async commands do not
> guarantee completion order, so an older answer that arrives last replaces a newer one. The record
> only grows, so a stale answer can show "no acknowledgement received by this Device" for a Device
> that has acknowledged. That is the second misreading in HAZ-2hqt34, until the next read.
>
> LLR-x9qbhu says "Each answer replaces the rows shown", so the letter of the requirement holds.
>
> Fix outline: keep a request counter and drop an answer that is not the latest. Add a test using
> `tauri.defer` twice, answered out of order.
disposition: fixed by m-a: a `latestRead` counter in `app/src/lib/components/RemovedDevices.svelte` drops any answer but the latest read's, stated by a dated note on LLR-x9qbhu; tests 'an earlier read answered after a later one is dropped, so the latest answer stays shown' and 'an earlier read refused after a later one was answered leaves the latest rows shown' (`app/tests/removed.view.test.ts`), each red before the counter.

**finding-19**: record — app (reviewer's finding-4; MR1-A4): T16g's prose puts the panel in `+page.svelte`; the diff mounts it in `Revoke.svelte`.
> **finding-4** — record — `docs/plans/2026-10-16-org-io-publish-2.md:516,862`
>
> The prose of task T16g puts the panel in `app/src/routes/+page.svelte`, in both its files list and
> step 9. The diff mounts it in `app/src/lib/components/Revoke.svelte:149-153` and does not touch
> `+page.svelte`. The T16g done note records this as an adaptation, but the task prose was not
> brought into line.
disposition: fixed by the dispatcher: T16g's Files touched and step 9 in `docs/plans/2026-10-16-org-io-publish-2.md` say the panel is mounted in `app/src/lib/components/Revoke.svelte` (MR1-A4); this record describes it there (`docs/verification/2026-11-09-worktree-org-io-publish-2.md`).

**finding-20**: requirement, low — on-chain-client (reviewer's finding-1; MR1-C1): three notes still say tests carry the superseded IDs.
> finding-1. requirement low — The change dropped superseded IDs from four test annotations and corrected the matching "carries both IDs" statements only on LLR-59kxsr and LLR-n2vx5q. Three other places still say the tests carry the superseded IDs:
> - `on-chain-client/docs/architecture/2026-09-28-decomposition.md:780-782` says the annotation "now reads `verifies: REQ-2qa5r5, REQ-54txzh, LLR-xv7auy`". It now reads `REQ-54txzh, LLR-xv7auy`.
> - `on-chain-client/docs/risk/2026-10-05-organisation-public-key.md:7-8` says the test "now verifies both".
> - `on-chain-client/docs/architecture/2026-10-06-chain-write.md:224` and `:229-231` (LLR-ywhd23's notes) say the `write_pure.rs` tests carry `LLR-59kxsr, LLR-ywhd23`.
>
> Proposed fix: add one dated sentence at each place, for example "Since 2026-11-04 (change `worktree-org-io-publish-2`, task T18) the test cites only REQ-54txzh / LLR-59kxsr."
disposition: fixed by m-c (6cce7d0): dated sentences in `on-chain-client/docs/architecture/2026-09-28-decomposition.md`, `on-chain-client/docs/risk/2026-10-05-organisation-public-key.md` and `on-chain-client/docs/architecture/2026-10-06-chain-write.md` say the tests now cite only the successors.

**finding-21**: requirement, low — on-chain-client (reviewer's finding-2; MR1-C2): two prose statements still describe the removed consumer.
> finding-2. requirement low — Two prose statements still describe the removed consumer:
> - `on-chain-client/docs/architecture/README.md:90`: "org-io also calls the two signatory-set reads of SDD-rxfu6h". That has been false since T16h. The R3 notes fix each item, but not the unit's architecture overview.
> - `.guardrails/units.yaml:63` (root config) still lists "the own-admin check" among org-io's responsibilities.
>
> Proposed fix: add a dated note at README:90, "since 2026-11-02 nothing calls them (owner ruling R3)", and drop "the own-admin check," from units.yaml:63, or add a dated note there.
disposition: README part fixed by m-c: a dated note in `on-chain-client/docs/architecture/README.md` says nothing calls the reads since 2026-11-02 (owner ruling R3). The `.guardrails/units.yaml` text is recorded for the owner, not fixed here: an edit to the root `.guardrails` maps the change into every unit's impact set, so it is left for the next guardrails change.

**finding-22**: requirement, low — on-chain-client and root (reviewer's finding-3; MR1-C3): `Makefile` still names the deleted `signatory.rs` in org-io's coverage shortfall.
> finding-3. requirement low — `Makefile:324-327` is the current statement of org-io's uncovered shell (the 2026-10-25 measurement, above `ORG_IO_LINES := 83`). It still names `signatory.rs 72.92%`, and the change deletes that file. Plan T16h step 6 asked for "the `signatory.rs` shortfall lines (≈288–332)" to go, but only the 288 block was edited. `:311` is a dated 2026-10-14 row and can stay. T17b expects org-io's coverage to rise after the deletion. No re-measurement is recorded, and I did not run `make coverage-org-io`. Proposed fix: drop `signatory.rs 72.92%,` from :326, or add a dated note there, and record the new measurement at 6a.
disposition: fixed by m-c: the `signatory.rs` entry is gone from the current shortfall statement in `Makefile` (a note says the file is deleted with the own-admin check); `make -n coverage-org-io` parses, and the step 6a gate measures org-io's coverage afresh.

**finding-23**: requirement, low — on-chain-client and root (reviewer's finding-4; MR1-C4): permanent ledgers cite the plan file.
> finding-4. requirement low — Permanent ledgers cite the plan file. If `merge-change` step 5 drops `docs/plans/2026-10-16-org-io-publish-2.md`, these references dangle:
> - `docs/adr/2026-10-06-org-io-unit.md:231-232`: "plan task T16h of `docs/plans/2026-10-16-org-io-publish-2.md`".
> - `on-chain-client/docs/requirements/2026-10-08-signatory-reads.md:39` and `on-chain-client/docs/requirements/2026-10-13-publish.md:56`: "plan task T16h".
>
> Master has no "chore: drop plans" commit yet, and earlier plans are still on master, so this applies only if step 5 is followed this time. Proposed fix: cite the change and the ruling ("change `worktree-org-io-publish-2`, owner ruling 6") and not the plan task.
disposition: fixed by m-c: `docs/adr/2026-10-06-org-io-unit.md`, `on-chain-client/docs/requirements/2026-10-08-signatory-reads.md` and `on-chain-client/docs/requirements/2026-10-13-publish.md` cite change `worktree-org-io-publish-2` and owner ruling 6, not the plan task.

**finding-24**: requirement, low — on-chain-client (reviewer's finding-5; MR1-C5): o2's notes under LLR-59kxsr and LLR-n2vx5q are pure history.
> finding-5. requirement low — The two task-o2 notes added under LLR-59kxsr (`on-chain-client/docs/architecture/2026-10-13-publish.md:66-69`) and LLR-n2vx5q (`:98-101`) are pure history. They record that the tests once carried both IDs. The corrected `Test:` line above each one already states what the tree does. The checklist bars history in an edited item, and no ruling asks for these notes, unlike the R3 notes. Proposed fix: delete both notes and keep the corrected `Test:` lines.
disposition: fixed by m-c: both history-only notes deleted from `on-chain-client/docs/architecture/2026-10-13-publish.md`; the corrected `Test:` lines of LLR-59kxsr and LLR-n2vx5q stay.

**finding-25**: record — on-chain-client (reviewer's finding-6; MR1-C6): the plan header says on-chain-client is touched "only if O2 removes its read".
> finding-6. record — The plan header (`docs/plans/2026-10-16-org-io-publish-2.md:7`) says on-chain-client is touched "only if O2 removes its read". O2 was ruled (a), keep (R3). The change still edits four on-chain-client ledger files, adding the R3 notes, and four on-chain-client test files. The header should say on-chain-client is touched for the R3 notes and the T18 re-annotations.
disposition: fixed by the dispatcher: the plan's Safety class line (`docs/plans/2026-10-16-org-io-publish-2.md`) is corrected (MR1-C6): on-chain-client is touched by the R3 ledger notes and T18's re-annotations; this record says the same under "Units touched" (`docs/verification/2026-11-09-worktree-org-io-publish-2.md`).

**finding-26**: record — on-chain-client (reviewer's finding-7; MR1-C7): T18's Files touched lists only org-node files.
> finding-7. record — T18's "Files touched" (plan:907-909) lists only org-node files. Its dispatch report (plan:982-999) re-annotates seven on-chain-client tests (`decode_org_state.rs`, `type_widths.rs`, `write_pure.rs` ×3, `signatory_set_decode.rs` ×2) and org-io and app tests. The task's prose does not describe these edits, so it does not match the diff.
disposition: fixed by the dispatcher: T18's Files touched in `docs/plans/2026-10-16-org-io-publish-2.md` names the Q6 sweep's re-annotations in org-io, app and on-chain-client (MR1-C7); this record copies the sweep's `inherited:` lines under T18 (`docs/verification/2026-11-09-worktree-org-io-publish-2.md`).

**finding-27**: requirement, low — repository-level (reviewer's finding-8; MR1-C8; predates this change): CI runs only on-chain-client's first verify line.
> finding-8. requirement low (predates this change, outside the diff) — `.github/workflows/rust.yml:3-6` says the `test` job runs on-chain-client's cargo entries "verbatim". It runs only the first. The second verify line (`--features test-support,write`, six targets: `write_manifest`, `write_pure`, `write_compose`, `write_events`, `signatory_set_decode`, `fuzz_signatory_set`) runs in CI only inside `make coverage`, as an llvm-cov run with one combined invocation. This change edits two of those targets. Proposed fix: add the second line to the `test` job, or correct the comment.
disposition: no change — it predates this change and is already open as PR-hj4wwd (on-chain-client); no new report.

**finding-28**: requirement, low — repository-level (reviewer's finding-9; MR1-C9; predates this change, org-members): `ORG_MEMBERS_COVERAGE_ARGS` omits `tree_error_mapping`.
> finding-9. requirement low (predates this change, from 641bec1; outside the diff) — `Makefile:494-499` says "Every org-members target the verify line `cargo test -p org-members` runs but `mbt_conformance` is measured." `ORG_MEMBERS_COVERAGE_ARGS` omits `tree_error_mapping` (`org-members/tests/tree_error_mapping.rs`). Proposed fix: add `--test tree_error_mapping`.
disposition: no change — it predates this change and org-members is outside this change's impact set; recorded for the owner, not opened here.

## The merge reviewers' reports

The four merge review reports of round 1, copied verbatim, each quoted whole
so that no line of them reads as a field or a finding of this record. The
only edits: repository-wide figures in the reviewers' checker lines (item
totals, open-problem counts and ages, which describe the ledgers at one
instant rather than this change) are replaced by `[repository figure
elided]`, as the verification template requires.

### The org-node report

> # Merge review round 1: worktree-org-io-publish-2, unit org-node (class C)
>
> Reviewed at b97eff4. Diff: `git diff master...b97eff4 -- org-node docs/CONTEXT.md` (25 files, +1131/−132).
> Plan read: header, §0, §4, §5, the conformance rounds and the 2026-11-08/09 rulings.
> The review worktree was left clean (`git status --porcelain` printed nothing). The build used a scratchpad target directory.
>
> ## Claimed IDs (each was run through find-items show)
>
> - New: REQ-h7y5ur, REQ-f8pu8a, REQ-32cp7f, SDD-acnpu4, LLR-xudkh4, LLR-py43n3, LLR-dsjht5, LLR-f5wrjx, LLR-gb7p7c, LLR-6fsnma.
> - Superseded (`superseded-by:`): LLR-d925uu→f5wrjx, LLR-vq5d5n→6fsnma, LLR-5wg8cu→xudkh4, LLR-r4qtqe→gb7p7c.
> - Amended or noted: SDD-undd7q, SDD-z4m57w, SDD-busuj3 (traces), LLR-2d5a7m (clarified), LLR-bgj2ff (S5 TODO and residual note), REQ-6tepxq (narrowed note).
> - Problem reports resolved: PR-m9betm, PR-y6ay44, PR-w88sr9 (obsolete).
> - Tests touched for these IDs: LLR-5tvn7g, LLR-bs6yxf.
>
> ## Answers
>
> ### 1. Does the code satisfy each claimed REQ/LLR?
>
> Yes, apart from finding-1.
>
> - **LLR-xudkh4.** `service.rs:400-420` computes the entry to keep before `check_acknowledgement` and appends only after acceptance, through `self.change`. A refusal or a failed seal therefore keeps nothing. `check_acknowledgement` refuses a listed Device, so the "record does not list it" half of LLR-bgj2ff holds.
> - **LLR-py43n3.**
>   - Appended once per Device.
>   - Not touched by `prune_after_adoption`.
>   - Absent from `RemovedStore`.
>   - The TODOs are on the field (`store.rs:1145`), the type (`store.rs:1105`) and the append (`service.rs:890`).
> - **LLR-f5wrjx.**
>   - The field order holds, with `acknowledged` last, in `StoreData`, `RawStoreData` and `StoreDataEncoding`.
>   - `KeptNotice.from` is re-parsed under `org.kept_revocation_notices`.
>   - The acknowledgement Device key is re-parsed under `org.acknowledged`.
> - **LLR-gb7p7c.** The notice is kept with `from: sender` (`service.rs:906`). `accepted_notice` returns the first kept notice that `accept` takes (`verify_org.rs:1242`).
> - **LLR-6fsnma / REQ-32cp7f.**
>   - `removal.rs:176-201` moves the delivering Device to the front only when it is a candidate, then truncates to k. That equals "it first, then the first k−1 of the rest".
>   - `service.rs:1006-1027` passes `Origin::Received/Answer.from`, `None` for `Own`, and the accepted notice's `from`.
> - **LLR-dsjht5.** It covers refusals (PersonaNotHeld, OrgNotHeld or FoundingNotWritten via `current_record`, OrganisationRemoved via `data()`), held order, the acknowledgement lookup, and no mutation. The exception is finding-1.
> - **PR-y6ay44.** `push_varint` is a correct LEB128 and the fallback path is gone.
>
> ### 2. Would each test fail if its `verifies:` behaviour broke?
>
> Yes, with two weak spots.
>
> **Tests that would fail:**
> - `the_device_a_match_came_from_is_the_first_recipient`, `the_delivering_device_of_the_accepted_notice_is_the_first_recipient`: the first one would catch a wrong origin. The second would catch "first kept" used in place of "first accepted", because the refused notice's `from` is a candidate.
> - `an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed`: checks sealed bytes, reopen, and the second-ack no-op.
> - `an_acknowledgement_from_a_device_not_held_as_revoked_is_accepted_and_not_kept`.
> - `removed_devices_*`, `the_removed_devices_of_a_removed_store_are_organisation_removed`.
> - `an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo`.
> - The encoding tests and `GOLDEN_ACTIVE_STORE` (2355 bytes, now with `from` and `acknowledged`).
> - The `persona_records` `org.acknowledged` spoil case.
> - `signed_bytes_are_the_domain_then_the_tuple_at_each_varint_length`: covers varint lengths 1, 2 and 10.
> - `a_removal_of_a_persona_with_no_member_id_is_refused_with_nothing_deleted`.
>
> **Inherited tests:** checked against their successors.
> - The `own_removal` pair: the delivering 0x21 is no candidate, so the old order stands.
> - The `receive_path` and `receive_chain_reads` acknowledgement tests: rewritten to expect `Some(_)` and one kept entry.
> - The `epoch_store` ×3 and `encoding_golden` tests.
>
> All inherited tests verify the successor's behaviour.
>
> **Weak spots:** see finding-3.
>
> ### 3. Does every claimed ID have abnormal-input cases?
>
> - **LLR-xudkh4:** not held as revoked, relayed, listed Device, second ack.
> - **LLR-dsjht5:** unknown key, no record, removed store.
> - **LLR-6fsnma:** `Own`, delivering Device not a candidate, k=0 (inherited), k=1.
> - **LLR-gb7p7c:** refused notice keeps nothing, bound (inherited).
> - **LLR-py43n3:** refusals change nothing, second ack not kept.
> - **LLR-f5wrjx:** invalid `org.acknowledged` key. The sender key gap is finding-2.
> - **LLR-xudkh4's failed seal:** recorded as unreachable at n2. Accepted.
>
> ### 4. Is any behaviour unrequired?
>
> No.
> - `RawAcknowledgement::parse` is a behaviour-free refactor.
> - `#[allow(clippy::too_many_arguments)]` is lint only.
> - The `revoked_devices` S5 TODO is required by the LLR-bgj2ff note.
> - REQ-h7y5ur, REQ-f8pu8a and REQ-32cp7f are marked `satisfies: derived` and assessed in `org-node/docs/risk/2026-11-02-publish-2.md`.
>
> ## Findings
>
> finding-1. code low: `OrgService::removed_devices` (`org-node/src/service.rs:1229-1246`) lists every `revoked_devices` entry, but it does not check that the current record no longer lists that Device.
> - LLR-dsjht5 says "for each RevokedDevice the store holds as revoked (LLR-bgj2ff)". LLR-bgj2ff (`2026-10-16-domain-api.md:2043`) says "revoked exactly when `revoked_devices` holds its DevicePublicKey and the current record does not list it".
> - `revoked_devices` is never pruned (`verify_org.rs:345-355` only appends). So a Device key admitted again after its removal would still be shown to the Admin as a removed Device.
> - It is rare: the same Device key would have to be re-admitted. No test covers it.
> - Fix: filter with `!record.lists_device(&revoked.device_key)` and add an abnormal case.
>
> finding-2. code low: no test drives LLR-f5wrjx's refusal of an invalid `KeptNotice.from` (`InvalidField` naming `org.kept_revocation_notices`, `org-node/src/store.rs:959`).
> - `persona_records.rs:827-842` (`unseal_refuses_an_invalid_field_of_each_epoch_state_kind_naming_it`) added the `org.acknowledged` case but not this one. `WireStore` already holds the sender as `[u8; 32]` (`persona_records.rs:1737`), so one spoil line adds it:
>   ```rust
>   ("org.kept_revocation_notices", |s| s.kept_revocation_notices[0].1 = off_curve_key())
>   ```
>
> finding-3. code low: the keep tests cannot tell `member_id` taken from the held `RevokedDevice` (which LLR-xudkh4 requires) apart from one taken from `ack.member_id`.
> - In `receive_path.rs:2030-2054` and `receive_chain_reads.rs` the two values are equal (`ack_by_dave` signs with `dave().id`, the same as the `RevokedDevice`).
> - Also, `an_adoption_keeps_..._s5_todo` (`own_removal.rs:1686-1705`) does not check the TODO on the `AcknowledgedEntry` type, which LLR-py43n3 also requires. The TODO is present (`store.rs:1105`), so this is only a gap in the test.
> - Fix: give the held `RevokedDevice` a different MemberId from the acknowledgement's.
>
> finding-4. requirement low: LLR-f5wrjx (`org-node/docs/architecture/2026-11-02-publish-2.md:321-342`) silently drops LLR-d925uu's clause "no `ProvisionalUpdate` type exists" (`2026-10-16-domain-api.md:82`).
> - The supersession note on LLR-d925uu says "every other field is as stated here" and does not name the drop. The epoch_store test was rewritten at the encoding seam to match.
> - The exhaustive field list still excludes a `provisional_updates` field, so the behaviour that matters stands.
> - Fix: say in the note that the type-absence clause is dropped.
>
> finding-5. requirement low: two items carry rationale or history inside the item text.
> - LLR-xudkh4 ends "The append is not measured by LLR-m76r4k's bound: the record holds at most one entry per entry of `revoked_devices`." Proposed: "The append is not measured by LLR-m76r4k's bound."
> - LLR-f5wrjx ends "Stores written before this change do not open (as for every store-format change since plan decision 12 of 2026-10-07)." Proposed: "A store in the format before this one does not open."
>
> finding-6. requirement low: the docs say two different things about which SDD owns LLR-xudkh4.
> - SDD-busuj3's traces note (`2026-10-16-domain-api.md`, the 2026-11-03 note) says "LLR-xudkh4, under SDD-acnpu4".
> - LLR-xudkh4 sits in SDD-acnpu4's section.
> - SDD-acnpu4's amended code claim says the append "is the acknowledgement arm of `check_received`, SDD-busuj3's (LLR-xudkh4)".
> - The plan's n1 entry left this "Open for round 2's ARCH review", and round 2 did not settle it.
> - Fix: pick one owner (the code is SDD-busuj3's arm, the data is SDD-acnpu4's) and word both notes the same way.
>
> finding-7. requirement low: the review checklist (`~/.claude/skills/merge-change/references/review-checklist.md`) does not match what this tree's guardrails require.
> - The checklist says no item carries `supersedes:`/`superseded-by:`, a changed behaviour's old item is deleted, a resolved PR is deleted, and regression tests never verify the PR.
> - This tree is pinned to guardrails 0.5.1 (`org-node/.guardrails/config.yaml`). Its `check-trace.sh` requires reciprocal `supersedes:`/`superseded-by:` and `status: resolved` entries.
> - The diff follows the tree. I raised no finding against the diff for this, but a reviewer applying the checklist literally would.
>
> finding-8. record: the plan header's **Resolves:** line (`docs/plans/2026-10-16-org-io-publish-2.md:6`) names PR-m9betm and PR-y6ay44 but not org-node PR-w88sr9. The diff resolves PR-w88sr9 as obsolete under the owner ruling of 2026-11-08, which is recorded only at plan line 1248. The squash message's `Resolves:`/`Problems:` should name all three.
>
> ## Tests run
>
> Command: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test …`. All passed, 0 failed, 0 ignored:
>
> | Target | Passed |
> |---|---|
> | encoding_golden | 5 |
> | epoch_store | 7 |
> | own_removal | 28 |
> | persona_records | 14 |
> | provisional_store | 3 |
> | receive_chain_reads | 5 |
> | receive_path | 29 |
> | revocation | 19 |
> | value_types | 15 |
> | verification | 52 |
> | **Total** | **177** |
>
> ## Checkers
>
> - `GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-trace.sh` exits 0. It prints only notices that do not fail the gate:
>   - UNRESOLVED-PR for the [repository figure elided] open reports (oldest [repository figure elided], against limits of 60 days and 15 open).
>   - UNMET-EXPECTATION for org-members REQ-q92yac ([repository figure elided]) and person REQ-and2gz ([repository figure elided]), both within the 90-day limit.
> - `GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-ids.sh` exits 0 and prints nothing. Run without `GR_CONFIG` it refuses, because the repository has several units.
>
> ## Verdict
>
> No high or medium findings; every code and requirement finding is low. The org-node part is fit to merge once the lows are dispositioned.

### The org-io report

> # Merge review, round 1: worktree-org-io-publish-2, unit org-io (class C)
>
> Reviewed at b97eff4 ("review dispatched") in the read-only review worktree.
> Inputs:
> - the diff `git diff master...b97eff4 -- org-io`: 35 files, +955/-664;
> - the plan's header, §0, §4, §5, the conformance rounds and Progress;
> - find-items for each touched ID;
> - the merge-change review checklist.
>
> ## Claimed / touched org-io items
>
> - **New:**
>   - REQ-xghnh3 and REQ-kn8zg6, both derived;
>   - LLR-3ydu9e, the removed-Devices query;
>   - LLR-zhdc9f, which supersedes LLR-4usfu2;
>   - LLR-u2j6u8, which supersedes LLR-3w22wc;
>   - LLR-spckm6, which supersedes LLR-cqr26z.
> - **Retired** (owner ruling 6): REQ-ntkjk6, LLR-qhyc3n, LLR-2hkqv2, LLR-7qdjqn and LLR-bwerd2. SDD-f4khqn is left with no code.
> - **Amended by dated notes:**
>   - SDD-z3ychz, SDD-z85ux9 (its `traces:` drops REQ-ntkjk6) and SDD-3my66m (its `traces:` gains REQ-kn8zg6);
>   - LLR-rgdx22, LLR-gh4rw3, LLR-jqb5z7 and LLR-4k43dm;
>   - REQ-k6kcgd and REQ-f3eu9n;
>   - PR-8geawr, now accepted, and PR-wk7zwq.
>
> ## Checkers
>
> Both run with `GR_CONFIG=org-io/.guardrails/config.yaml`.
> - **check-trace.sh:** exit 0. It counted [repository figure elided]. The only notices are pre-existing UNRESOLVED-PR / UNMET-EXPECTATION lines and ACCEPTED-PR PR-8geawr.
>   - The oldest open PR is [repository figure elided], under the limit of 60. The change raised that limit from 30 by the owner ruling of 2026-11-03, which the plan records.
> - **check-ids.sh:** exit 0.
>
> ## Tests run (`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-io ...`)
>
> | Lane | Target | Result |
> |---|---|---|
> | test-support | acknowledgements | 45 passed, 0 failed (667 s) |
> | test-support | absences | 31 passed |
> | test-support | acknowledgement_inbound | 8 passed |
> | test-support | chain_write | 29 passed |
> | test-support | receive_step | 26 passed |
> | test-support | signatory_key | 5 passed |
> | test-support,dev-seed | dev_seed_env | 3 passed |
> | test-support,dev-seed | signatory_key | 3 passed |
> | test-support,dev-seed | chain_writer_contract | 2 passed (no chopsticks fork here, so they skip-pass; the edit there only drops the signatory clause) |
>
> Total: 152 passed, 0 failed. These cover every `verifies:` test of a claimed ID and every test file the diff edits. `signatory_rule.rs` is deleted.
>
> ## Q1. Does the code satisfy each claimed REQ/LLR?
>
> Yes.
> - **LLR-3ydu9e / REQ-xghnh3:** `org-io/src/removed.rs:42-55` takes the lock once, calls `record_holder_of`, then org-node's `removed_devices`, and maps each entry to `RemovedDevice` in org-node's order.
>   - An Organisation no Persona holds gives `NotHeld`; an org-node error gives `Node`. Nothing is read from the chain, sent or written.
>   - `Node` is unreachable, and a comment says so.
> - **LLR-zhdc9f:** the handle's surface matches the item. `is_own_admin` is gone and `removed_devices` is added.
> - **LLR-u2j6u8 / REQ-kn8zg6:** `acknowledge.rs` `dial_plan` and `try_addresses`.
>   - Each socket is dialled alone, in order. With none, Networked mode makes one dial with an empty socket list and Loopback makes none, which gives `NoAddress`.
>   - The dial stops at `Delivered`.
> - **LLR-spckm6:** `dial_by_key` binds from the keychain with the handle's `bind_plan`, sends once and closes.
>   - A failed relay bind is that recipient's `SenderEndpoint("the relay bind failed…")`, and the other recipients are still dialled.
>   - The direct bind is unchanged. Loopback never reaches `dial_by_key`.
> - **REQ-ntkjk6 retirement:** no `is_own_admin`, `signatory` module, signatory fields or `with_own_account` is left in src. `CoSignerIsOwnAccount` stays and is re-cited to LLR-rgdx22.
>
> ## Q2. Would each test fail if its behaviour broke?
>
> Mostly yes. The red-first evidence for each test is recorded in the plan, and I checked each test's assertions against the code. Two gaps:
> - **acknowledgement_inbound.rs:302 (LLR-3ydu9e):** finding-1.
> - **absences.rs:679 `the_handle_offers_none_of_the_removed_methods`:** strictly weaker than the exact-equality test at :657, so it can only fail when that test also fails. It is harmless and correct, so it is not raised as a finding.
>
> ## Q3. Does every claimed ID have abnormal-input cases (class C)?
>
> - **LLR-3ydu9e:** NotHeld, and the empty list as a boundary. The org-node refusal is unreachable, as documented.
> - **LLR-u2j6u8:**
>   - a kept address that does not parse;
>   - kept addresses that all fail (no dial by key follows);
>   - Loopback with no address (`NoAddress`);
>   - the Refused, NotDelivered and other-verdict cases inherited from earlier changes.
> - **LLR-spckm6:**
>   - a failed direct bind (inherited);
>   - a failed relay bind, both alone and with the other recipients still dialled;
>   - a transport failure while the recipient is not in the lookup.
>   - The "closed once the outcome is read" clause is untestable, as the item's own note says (finding-3).
> - **LLR-zhdc9f:** a structural surface item, so abnormal input does not apply.
> - **REQ-xghnh3 and REQ-kn8zg6:** covered through their LLRs.
>
> ## Q4. Is any behaviour unrequired (unmarked derived work)?
>
> None found.
> - `Endpoints::bind_plan()` and `Context.bind_plan` serve LLR-spckm6.
> - The `problem_age_days` change and PR-8geawr's acceptance are owner rulings, recorded.
> - The `receive_step.rs` edit (one store write on an acknowledgement) follows LLR-kz9hj4 ("writes the store change it returns"), now that org-node keeps an Acknowledged record entry.
>
> ## Findings
>
> **finding-1. code low:** `org-io/tests/acknowledgement_inbound.rs:302-330`. `the_admin_lists_a_removed_device_and_then_its_acknowledgement` (verifies LLR-3ydu9e) leaves several clauses unchecked:
> - It matches `acknowledged_epoch: Some(_)` and `..` for `member_id`. A mutant that maps the wrong epoch, or the wrong MemberId, stays green.
> - It asserts none of the item's "reads no chain … sends nothing and writes nothing". `admin.chain.reads()` and `store_writes_to` are available, as `receive_step.rs:489` uses them.
> - Fix: assert the epoch Bob's acknowledgement names, Bob's MemberId, and unchanged chain-read and store-write counts around the query.
>
> **finding-2. requirement low:** `org-io/docs/requirements/2026-10-14-domain-api.md:371-376`. REQ-k6kcgd's note of 2026-11-03 re-reads the item's "over a direct connection with no relay" as "iroh's relay server may carry the connection".
> - That loosens the requirement for every send. The tree still makes every dial that has an address direct-only (LLR-spckm6). Only the dial by key alone, which REQ-kn8zg6 states, is relayed.
> - The note thus states a looser behaviour than the item's text and the code.
> - Proposed wording: "The dial by a recipient's key alone when no kept address may be dialled is REQ-kn8zg6's, outside this item's 'at the endpoint addresses kept with it'; every dial at a kept address stays direct with no relay."
>
> **finding-3. requirement low:** `org-io/docs/architecture/2026-11-02-publish-2.md:89-108`. LLR-spckm6 states about eight behaviours in one item:
> - the identity;
> - closing a relayed endpoint;
> - the direct bind;
> - the address form;
> - the direct-bind failure;
> - the relay bind for a dial by key;
> - closing after the dial;
> - the relay-bind failure;
> - Loopback never binding the relay.
>
> One of them, "which is closed once the dial's outcome is read", is untestable by the item's own note of 2026-11-04. The checklist asks for one testable behaviour per item. Proposed: drop the closing clause from the item text and keep it as design prose. This is not a request to split the item, which the conformance loop accepted.
>
> **finding-4. requirement low:** `org-io/docs/architecture/2026-10-16-domain-api.md:1705`. SDD-3my66m's note of 2026-11-04 says the dial by key binds "the handle's plan (`Endpoints::plan`)".
> - Task o2 renamed it to `Endpoints::bind_plan` (`org-io/src/endpoints.rs:113`, and LLR-spckm6's own o2 note).
> - Fix: replace `Endpoints::plan` with `Endpoints::bind_plan`.
>
> **finding-5. requirement low:** `org-io/docs/architecture/2026-10-16-domain-api.md:2345-2349`. LLR-7qdjqn's deletion note says the absence is pinned by `absences.rs` `org_io_offers_no_own_admin_check`.
> - o2 replaced that test with `the_handle_offers_none_of_the_removed_methods` (`org-io/tests/absences.rs:679`), and the exact-surface test also pins it.
> - Fix: rename it in the note.
>
> **finding-6. record:** `docs/plans/2026-10-16-org-io-publish-2.md:1276`. The Progress table still reads "T16h, T16c4, T16g, T18, T17b | open", but §5 records T16h, T16c4, T16g and T18 as done and merged, and the conformance loop as complete.
> - The header line "This branch holds the plan draft only (docs, no code)" is equally stale.
>
> **finding-7. requirement low:** `/Users/jan-jan/.claude/skills/merge-change/references/review-checklist.md:44-47`. The checklist says an item never carries `supersedes:`, `superseded-by:` or `retired:`, and that a changed behaviour deletes the old item.
> - This tree's guardrails do the opposite. `.guardrails/scripts/check-trace.sh` (header lines 22-169) requires reciprocal `supersedes:` / `superseded-by:` and well-formed `retired:` lines.
> - This change follows the project's rule: LLR-3w22wc, LLR-cqr26z and LLR-4usfu2 are superseded, and REQ-ntkjk6 and four LLRs are retired, each in place.
> - So the checklist misstates what this tree does. The change itself is not at fault.
>
> ## Verdict
>
> Mergeable for org-io. Every claimed REQ/LLR is met and tested with abnormal cases, both checkers exit 0, and 152/152 tests pass. All findings are low or record, so this is the last review round.

### The app report

> # Merge review, change `worktree-org-io-publish-2`, unit app (class C), round 1
>
> Reviewed: `git diff master...b97eff4 -- app` (17 files, +714/-26), in the review worktree
> `.worktrees/worktree-org-io-publish-2-review` at b97eff4. The checklist used was
> `merge-change/references/review-checklist.md`.
>
> ## Items read (find-items.sh show)
>
> REQ-yvzef9, HAZ-2hqt34, RC-z8cptv, SDD-g9ugu7, SDD-6pjvhc (amended), LLR-ce4hxz (supersedes
> LLR-kk8gc7), LLR-kk8gc7, LLR-dmfvv6, LLR-cs9bmw, LLR-x9qbhu. Also read for the receiver_events.rs
> re-pointing: LLR-wgr46u, LLR-7bk6qh, LLR-zw6jqy, LLR-a9rjtf and LLR-usxk57. The diff also sets
> PR-5mc4d8, PR-h6xpnh, PR-j9f6kk, PR-w5dae4 and PR-eecx3y to `accepted`, each with a dated
> disposition, and adds an RC-wzb48r note.
>
> ## Checks run
>
> - `GR_CONFIG=app/.guardrails/config.yaml check-trace.sh` exits 0. It lists only ACCEPTED-PR ×5
>   (this change's rulings) and the pre-existing UNRESOLVED-PR [repository figure elided] and UNMET-EXPECTATION [repository figure elided], all
>   within limits. Counts: [repository figure elided].
> - `check-ids.sh` exits 0.
> - Rust:
>   - `cargo test --features test-support --test ipc`: 60 passed, 0 failed.
>   - `--test receiver_events`: 50 passed, 0 failed.
> - Webview:
>   - `npm --prefix app run test -- tests/removed.view.test.ts tests/epoch.commands.test.ts`: 2 files,
>     26 passed, 0 failed.
>   - Of those, removed.view.test.ts ran 7 and epoch.commands.test.ts ran 19.
> - Tree hygiene:
>   - `npm ci` and the builds added only gitignored paths: `app/node_modules`, `app/.svelte-kit` and
>     `app/src-tauri/target`.
>   - `app/src-tauri/Cargo.lock` and `app/package-lock.json` are byte-identical to b97eff4.
>   - `git status` could not be run inside the review worktree, because the session guard refuses
>     any git redirect to another worktree. The tree was checked with the file comparisons above.
>
> ## Answers
>
> ### 1. Does the code satisfy each claimed REQ/LLR?
>
> Yes, with one gap in the documents (finding-1).
>
> - **LLR-ce4hxz**:
>   - `lib.rs` and the `ipc.rs` harness register the same 18 names.
>   - The old names are refused.
>   - There is one `generate_handler!` list and one `invoke_handler` call.
> - **LLR-dmfvv6**:
>   - `commands.rs:828-831` calls `parse_org_id` first, so a malformed identifier gets the parser's
>     text.
>   - It calls `OrgIo::removed_devices` once and maps the results in org-io's order.
>   - Both identifiers are lowercase hex (`hex::encode`), and `acknowledged_epoch` is an
>     `Option<u64>`.
>   - The DTO has only these three fields, and org-io's refusal is returned as its `Display` text.
> - **LLR-cs9bmw**:
>   - The panel shows the title and the statement verbatim.
>   - Each row shows the MemberId, the DevicePublicKey and `removedDeviceLine`.
>   - The panel shows the empty-list line only once loaded, and a refusal as its text.
>   - No reachable refusal text (the parser's, or org-io's `no Persona here holds Organisation …`)
>     contains a deletion word.
> - **LLR-x9qbhu**:
>   - The `$effect` loads on mount, and `{#key selectedOrgId}` in `Revoke.svelte` remounts the panel
>     when another Organisation is selected.
>   - The panel loads on Refresh, and on a `verification-result` whose `org_id` matches.
>   - No other event causes a read.
> - **REQ-yvzef9**: met through these three LLRs. The panel is shown for whichever held
>   Organisation is selected on the Revoke tab.
> - **RC-z8cptv**: met by the wording that LLR-cs9bmw's tests check.
> - **SDD-g9ugu7**: the code it claims is the code in the diff.
>
> ### 2. Would each test fail if the behaviour its `verifies:` names broke?
>
> Yes, for every new or edited test:
>
> - **`ipc.rs`**:
>   - The count test goes red at 17 ≠ 18.
>   - The set test compares against `EIGHTEEN`.
>   - The order test compares against org-io's own listing, so a reversed list or uppercase hex
>     fails.
>   - The contract test checks keys strictly, so an extra field fails it.
> - **`removed.view.test.ts`**:
>   - The tests check exact text, call counts and arguments, and that rows are replaced.
>   - The tests for the in-flight answer and for a refusal replacing rows were each shown red
>     against a mutant (the plan's dispatch reports).
> - **Inherited tests** (the five `ipc.rs` tests moved from LLR-kk8gc7 to LLR-ce4hxz): each one
>   checks the successor's set or rule.
> - **`receiver_events.rs`**:
>   - `a_chain_failure_emits_no_verification_event_for_any_message` classifies
>     `OrgNodeError::Chain`. That is LLR-wgr46u's "every other variant → ReceiveError" rule, so the
>     move from LLR-7bk6qh is correct.
>   - Dropping LLR-a9rjtf, which is superseded, leaves LLR-zw6jqy, whose rule these three tests
>     check.
>
> ### 3. Does every claimed ID have abnormal-input cases?
>
> Yes:
>
> - **LLR-ce4hxz**: the old and unknown names are refused.
> - **LLR-dmfvv6**: an unheld Organisation gets org-io's refusal, and every malformed form gets the
>   parser's text.
> - **LLR-cs9bmw**: the empty list, a refusal, and an answer still in flight.
> - **LLR-x9qbhu**: a result for another Organisation, another event, a refused subscription, and a
>   refused then a successful Refresh.
> - **REQ-yvzef9**: covered through its LLRs, the lowest level.
> - **SDD-g9ugu7**: tested at its own interface. `removedDeviceLine` is called directly and the
>   component is rendered alone, as well as end-to-end on the Revoke tab.
>
> ### 4. Is any behaviour unrequired, that is, unmarked derived work?
>
> Yes, the refused-subscription label (finding-1).
>
> ## Findings
>
> **finding-1** — requirement low — `app/src/lib/components/RemovedDevices.svelte:53,70-72`
>
> The panel shows `listening for verification results failed: <text>` when its subscription is
> refused. No item states this:
>
> - LLR-x9qbhu (`app/docs/architecture/2026-11-02-publish-2.md:97`) says only when the panel reads.
> - LLR-cs9bmw lists what the panel shows, and this label is not on that list.
>
> Yet the test `a refused subscription is shown on the panel, and the rows still are` carries
> `verifies: LLR-x9qbhu`. The same gap was found and fixed for LLR-5mw2dj in the domain-api change
> (TEST review round 1, finding-4; `app/docs/architecture/2026-10-16-domain-api.md:1253-1268`).
>
> Proposed wording, appended to LLR-x9qbhu: "a refused subscription is shown on the panel under the
> label `listening for verification results failed:` with its text, and the rows are still read and
> shown."
>
> **finding-2** — requirement low — `app/docs/architecture/2026-10-16-domain-api.md:90,101-102`;
> `app/docs/CONTEXT.md:45`
>
> The body text still gives the old count:
>
> - The SDD-6pjvhc section preamble says "the seventeen command handlers (eighteen until
>   2026-10-20)".
> - SDD-6pjvhc's own sentence says "Its seventeen commands are the thirteen that call org-io's domain
>   surface".
> - The glossary's **Command** entry says "Seventeen exist (… LLR-kk8gc7)" and cites the superseded
>   item.
>
> Only the dated note added at domain-api.md:111 says eighteen. The tree has eighteen commands, and
> fourteen of them call org-io's domain surface.
>
> Proposed fix:
>
> - Amend SDD-6pjvhc's sentence in place to "Its eighteen commands are the fourteen that call org-io's
>   domain surface under its names, then …".
> - Fix the preamble to match.
> - Change the glossary entry to "Eighteen exist (… LLR-ce4hxz)" with a dated note.
>
> **finding-3** — code low — `app/src/lib/components/RemovedDevices.svelte:22-31`
>
> `load()` can be in flight several times at once, for example after a Refresh click followed by a
> `verification-result`. Each answer is assigned whenever it arrives. Tauri's async commands do not
> guarantee completion order, so an older answer that arrives last replaces a newer one. The record
> only grows, so a stale answer can show "no acknowledgement received by this Device" for a Device
> that has acknowledged. That is the second misreading in HAZ-2hqt34, until the next read.
>
> LLR-x9qbhu says "Each answer replaces the rows shown", so the letter of the requirement holds.
>
> Fix outline: keep a request counter and drop an answer that is not the latest. Add a test using
> `tauri.defer` twice, answered out of order.
>
> **finding-4** — record — `docs/plans/2026-10-16-org-io-publish-2.md:516,862`
>
> The prose of task T16g puts the panel in `app/src/routes/+page.svelte`, in both its files list and
> step 9. The diff mounts it in `app/src/lib/components/Revoke.svelte:149-153` and does not touch
> `+page.svelte`. The T16g done note records this as an adaptation, but the task prose was not
> brought into line.
>
> ## Not raised
>
> - **Supersession lines.** LLR-ce4hxz carries `supersedes:` and LLR-kk8gc7 carries `superseded-by:`.
>   The checklist bars these lines, but this project's guardrails scripts still check for reciprocal
>   supersession pairs (`check-trace.sh` NON-RECIPROCAL-SUPERSESSION), and the tree uses them
>   throughout. This is the project's convention at its guardrails version, not this change's error.
> - **Dated notes inside items.** SDD-g9ugu7 and SDD-6pjvhc carry dated notes. This follows the
>   project's convention, which the conformance rounds settled (A1-4, A2-1).
> - **The keyed `{#each … (removed.device_key)}`.** A duplicate key would throw, but org-node pushes
>   a revoked key only when it is not already held (`org-node/src/verify_org.rs:349-351`).
> - **REQ-yvzef9's length.** It states one feature, the view and when it reads. Splitting it would
>   add nothing that a test can tell apart.
>
> ## Verdict
>
> Mergeable after this round: four findings, all low or record, and every claimed REQ and LLR is met
> and tested, including abnormal inputs.

### The on-chain-client and repository-level report

> # Merge review, round 1 — worktree-org-io-publish-2 at b97eff4: on-chain-client and repository-level files
>
> Reviewer: independent, class C. Scope: on-chain-client; docs/adr, docs/CONTEXT.md (occ only), Makefile, .github/workflows/rust.yml, root config.
> Diff read: `git diff master...b97eff4 -- on-chain-client docs/adr Makefile .github` and `docs/CONTEXT.md`. The only root-level files the diff changes are `Makefile` and `.github/workflows/rust.yml`. No root Cargo or `.guardrails` file changes.
>
> ## What the diff does in scope
>
> - **on-chain-client source:** none.
> - **on-chain-client tests:** four files, changing only `verifies:` lines. Each drops a superseded ID and keeps its successor, which it already cited:
>   - `decode_org_state.rs:108` drops LLR-nq7nhg and keeps LLR-hezpr7.
>   - `type_widths.rs:105` drops REQ-2qa5r5 and keeps REQ-54txzh.
>   - `write_pure.rs:175,190,216` drops LLR-ywhd23 and keeps LLR-59kxsr.
>   - `signatory_set_decode.rs:286,308` drops LLR-a9bb7b and keeps LLR-n2vx5q.
> - **on-chain-client ledgers:** R3 notes ("no consumer since 2026-11-02; kept for the several-Admins stage") on REQ-8p2veg, REQ-b8x2py, SDD-rxfu6h, LLR-m3tjvp, LLR-n2vx5q and LLR-sc79gq. Task o2 also corrected the `Test:` lines of LLR-59kxsr and LLR-n2vx5q, and added notes on both.
> - **ADR 2026-10-06-org-io-unit.md:** a dated note on the own-admin bullet.
> - **Makefile and rust.yml:** `--test signatory_rule` is dropped from `ORG_IO_COVERAGE_ARGS` and from CI, and the `signatory.rs` shortfall lines are dropped from one comment block.
>
> ## Answers
>
> **1. Do the code and tests satisfy each claimed REQ or LLR?** Yes, as far as anything in on-chain-client changed. No production code changed. Every ID left on an edited test is in force:
> - LLR-hezpr7 and REQ-4astjb/9wwenn: the exact `OrgState` is asserted from ascending fills.
> - REQ-54txzh and LLR-xv7auy: four widths, via `size_of` and region conversion.
> - LLR-59kxsr: the full `Value` equality of `Proxy.proxy`, with `Some(Any)` by name; `map_account_call`; nesting; and not `None`.
> - LLR-n2vx5q: definitions with type and delay in stored order, plus the u8/u32 maxima and the empty list.
>
> Each one verifies the successor's behaviour, so the plan's `inherited:` entries (plan:993-999) are sound. Owner ruling R3 is applied: every exported item of the signatory-set read has its dated note. §7's wording, "since 2026-10-16 (ruling 6)", is superseded by §0's R3, which says 2026-11-02, and the notes follow R3.
>
> **2. Would each test fail if its behaviour broke?** Yes. All seven edited tests assert exact values: whole-struct, whole-`Value` or exact `Err(Malformed)` equalities. None is a call-only assertion. I re-ran them but did not mutate the code: no source changed, and the bodies are unchanged from master.
>
> **3. Does each claimed ID have abnormal-input cases?**
> - LLR-59kxsr has three: the nesting test, the never-unforced/never-by-index test, and the runtime-without-`Any` encoding test.
> - LLR-n2vx5q has `a_malformed_definitions_value_is_refused` and the bolero target `fuzz_signatory_set`.
> - LLR-sc79gq and LLR-m3tjvp each have their abnormal tests (`a_runtime_without_an_any_proxy_type_is_refused`, `a_failed_read_or_an_undecodable_value_is_an_error`, `an_original_account_of_another_length_is_malformed`).
> - LLR-hezpr7 is one-sided. Its abnormal side is LLR-sq76u3, recorded at `2026-10-05-organisation-public-key.md:18-27` and in the decomposition's one-sided table.
> - REQ-54txzh and LLR-xv7auy carry a recorded exemption: the abnormal side is the type system (`2026-09-28-decomposition.md` ≈606-620).
>
> **4. Is any behaviour unrequired?** None was added. The signatory-set read is kept with no consumer by owner ruling R3. REQ-b8x2py and REQ-8p2veg stay in force (`satisfies: derived`, `exported: yes`), so the code is still required. The owner should know that what justified the derivation, org-io's own-admin check, is gone.
>
> **5. Do the Makefile coverage lists, the CI test lists and `verify_commands` agree?** I compared them with a script (`pr2-mr1-occ-lists.py`) and checked the results by hand.
>
> | Unit | Agreement | Targets in one list and not the others |
> |---|---|---|
> | org-io | `verify_commands` (lines 1+2), `ORG_IO_COVERAGE_ARGS` and the two CI runs agree: 28 target names each. | `signatory_rule` is gone from all three and from `org-io/Cargo.toml`. Two targets are in none of the three: `chain_writer_contract` and `preflight`. Both are documented chopsticks exclusions (`org-io/Cargo.toml:82-90`, `org-io/.guardrails/config.yaml:55`). `absences`, `app_boundary` and `chain_read_rule` have no `[[test]]` entry; cargo auto-discovers them, which is fine. |
> | on-chain-client | `verify_commands` and `ON_CHAIN_CLIENT_COVERAGE_ARGS` agree: 18 targets. | The CI `test` job lacks the whole second verify line: `write_manifest`, `write_pure`, `write_compose`, `write_events`, `signatory_set_decode` and `fuzz_signatory_set`. In CI these six run only through `make coverage`. This predates the change (finding-8). `write_genesis_e2e` is in none of the lists, a documented chopsticks exclusion. |
> | org-node | `verify_commands` and CI agree: 34 targets. | org-node has no Makefile statement-coverage target. |
> | org-members | `verify_commands` (`cargo test -p org-members`) runs everything. | `ORG_MEMBERS_COVERAGE_ARGS` omits `mbt_conformance`, which is documented, and `tree_error_mapping`, which is not. This predates the change (finding-9). |
> | person, sparse-merkle-tree | Whole crate in every list. Consistent. | — |
>
> ## Tests and checks run
>
> Every run used `CARGO_HOME=/tmp/cargo_home_fuzz` and `CARGO_TARGET_DIR` in the scratchpad. The write-feature targets need `--features test-support,write`, because their `required-features` include `write`.
>
> | Target | Result |
> |---|---|
> | `decode_org_state` | 9 passed, 0 failed |
> | `type_widths` | 2 passed, 0 failed |
> | `write_pure` | 25 passed, 0 failed |
> | `signatory_set_decode` | 10 passed, 0 failed |
> | `fuzz_signatory_set` (harness=false, bolero) | 1 s budget, about 267.5k iterations, exit 0 |
>
> - `GR_CONFIG=on-chain-client/.guardrails/config.yaml check-trace.sh` exits 0. It reports [repository figure elided] ACCEPTED-PR, from the 2026-10-28 rulings, and [repository figure elided] UNRESOLVED-PR, PR-hj4wwd, open [repository figure elided] and within the limits. It reported no MISSING-TEST and no OUT-OF-FORCE-VERIFIES.
> - `GR_CONFIG=on-chain-client/.guardrails/config.yaml check-ids.sh` exits 0.
> - I did not run `make coverage-org-io` (see finding-3).
>
> ## Findings
>
> finding-1. requirement low — The change dropped superseded IDs from four test annotations and corrected the matching "carries both IDs" statements only on LLR-59kxsr and LLR-n2vx5q. Three other places still say the tests carry the superseded IDs:
> - `on-chain-client/docs/architecture/2026-09-28-decomposition.md:780-782` says the annotation "now reads `verifies: REQ-2qa5r5, REQ-54txzh, LLR-xv7auy`". It now reads `REQ-54txzh, LLR-xv7auy`.
> - `on-chain-client/docs/risk/2026-10-05-organisation-public-key.md:7-8` says the test "now verifies both".
> - `on-chain-client/docs/architecture/2026-10-06-chain-write.md:224` and `:229-231` (LLR-ywhd23's notes) say the `write_pure.rs` tests carry `LLR-59kxsr, LLR-ywhd23`.
>
> Proposed fix: add one dated sentence at each place, for example "Since 2026-11-04 (change `worktree-org-io-publish-2`, task T18) the test cites only REQ-54txzh / LLR-59kxsr."
>
> finding-2. requirement low — Two prose statements still describe the removed consumer:
> - `on-chain-client/docs/architecture/README.md:90`: "org-io also calls the two signatory-set reads of SDD-rxfu6h". That has been false since T16h. The R3 notes fix each item, but not the unit's architecture overview.
> - `.guardrails/units.yaml:63` (root config) still lists "the own-admin check" among org-io's responsibilities.
>
> Proposed fix: add a dated note at README:90, "since 2026-11-02 nothing calls them (owner ruling R3)", and drop "the own-admin check," from units.yaml:63, or add a dated note there.
>
> finding-3. requirement low — `Makefile:324-327` is the current statement of org-io's uncovered shell (the 2026-10-25 measurement, above `ORG_IO_LINES := 83`). It still names `signatory.rs 72.92%`, and the change deletes that file. Plan T16h step 6 asked for "the `signatory.rs` shortfall lines (≈288–332)" to go, but only the 288 block was edited. `:311` is a dated 2026-10-14 row and can stay. T17b expects org-io's coverage to rise after the deletion. No re-measurement is recorded, and I did not run `make coverage-org-io`. Proposed fix: drop `signatory.rs 72.92%,` from :326, or add a dated note there, and record the new measurement at 6a.
>
> finding-4. requirement low — Permanent ledgers cite the plan file. If `merge-change` step 5 drops `docs/plans/2026-10-16-org-io-publish-2.md`, these references dangle:
> - `docs/adr/2026-10-06-org-io-unit.md:231-232`: "plan task T16h of `docs/plans/2026-10-16-org-io-publish-2.md`".
> - `on-chain-client/docs/requirements/2026-10-08-signatory-reads.md:39` and `on-chain-client/docs/requirements/2026-10-13-publish.md:56`: "plan task T16h".
>
> Master has no "chore: drop plans" commit yet, and earlier plans are still on master, so this applies only if step 5 is followed this time. Proposed fix: cite the change and the ruling ("change `worktree-org-io-publish-2`, owner ruling 6") and not the plan task.
>
> finding-5. requirement low — The two task-o2 notes added under LLR-59kxsr (`on-chain-client/docs/architecture/2026-10-13-publish.md:66-69`) and LLR-n2vx5q (`:98-101`) are pure history. They record that the tests once carried both IDs. The corrected `Test:` line above each one already states what the tree does. The checklist bars history in an edited item, and no ruling asks for these notes, unlike the R3 notes. Proposed fix: delete both notes and keep the corrected `Test:` lines.
>
> finding-6. record — The plan header (`docs/plans/2026-10-16-org-io-publish-2.md:7`) says on-chain-client is touched "only if O2 removes its read". O2 was ruled (a), keep (R3). The change still edits four on-chain-client ledger files, adding the R3 notes, and four on-chain-client test files. The header should say on-chain-client is touched for the R3 notes and the T18 re-annotations.
>
> finding-7. record — T18's "Files touched" (plan:907-909) lists only org-node files. Its dispatch report (plan:982-999) re-annotates seven on-chain-client tests (`decode_org_state.rs`, `type_widths.rs`, `write_pure.rs` ×3, `signatory_set_decode.rs` ×2) and org-io and app tests. The task's prose does not describe these edits, so it does not match the diff.
>
> finding-8. requirement low (predates this change, outside the diff) — `.github/workflows/rust.yml:3-6` says the `test` job runs on-chain-client's cargo entries "verbatim". It runs only the first. The second verify line (`--features test-support,write`, six targets: `write_manifest`, `write_pure`, `write_compose`, `write_events`, `signatory_set_decode`, `fuzz_signatory_set`) runs in CI only inside `make coverage`, as an llvm-cov run with one combined invocation. This change edits two of those targets. Proposed fix: add the second line to the `test` job, or correct the comment.
>
> finding-9. requirement low (predates this change, from 641bec1; outside the diff) — `Makefile:494-499` says "Every org-members target the verify line `cargo test -p org-members` runs but `mbt_conformance` is measured." `ORG_MEMBERS_COVERAGE_ARGS` omits `tree_error_mapping` (`org-members/tests/tree_error_mapping.rs`). Proposed fix: add `--test tree_error_mapping`.
>
> ## Verdict
>
> No code findings. In on-chain-client and the root files, every `code` or `requirement` finding is low (1–5, 8, 9), plus two `record` findings (6, 7). By the severities this is a last-round result: fix the mechanical lows, record the rest, and dispatch no further reviewer for this scope.

## Guideline reviews

The `develop-change` conformance reviews: one review per guidelines file that
`guidelines-file.sh` printed for the change diff, split by unit for depth
(TEST and CODE: org-node; org-io with on-chain-client; app). Each review ran
again in round 2, all of them rerun because task q1r changed org-io code after
round 1. Every path resolved to the installed defaults under
`.guardrails/templates/`, except TEST, which resolved to the root tailoring
`docs/TEST_GUIDELINES.md`. The reviewers' report files were kept only in the
session scratchpad, so each finding below is the plan's record of it
("Conformance round 1" and "Conformance round 2" in
`docs/plans/2026-10-16-org-io-publish-2.md`), with the plan's finding ID, and
its disposition is the plan's, with the fixer that carried it out (n1, o1, a1
and q1r in round 1; n2, o2 and a2 in round 2). Round 2 had no finding above
low in any review, and its lows were fixed with no further reviewer.

### docs/TEST_GUIDELINES.md

Governs: the test files the change adds or modifies — org-node/tests (encoding_golden.rs, epoch_store.rs, own_removal.rs, persona_records.rs, provisional_store.rs, receive_chain_reads.rs, receive_path.rs, revocation.rs, value_types.rs, verification.rs); org-io/tests (absences.rs, acknowledgement_inbound.rs, acknowledgements.rs, chain_write.rs, chain_writer_contract.rs, common/chain_contract.rs, dev_seed_env.rs, receive_step.rs, signatory_key.rs, the deleted signatory_rule.rs) with org-io/src/test_support.rs; app/src-tauri/tests (ipc.rs, receiver_events.rs) and app/tests (removed.view.test.ts, epoch.commands.test.ts, contract/backend.json); on-chain-client/tests (decode_org_state.rs, signatory_set_decode.rs, type_widths.rs, write_pure.rs) — and the LLRs those tests verify, as the review reports state their scope.

### Round 1

**finding-1**: guideline, medium — Seams 2 — org-node/tests/own_removal.rs:699-705 (T1-N1): `sign_acknowledgement_refuses_a_persona_with_no_member_id` calls `revocation::sign_acknowledgement` directly; no LLR states that refusal; its `verifies: LLR-5tvn7g` contract (store unchanged) is unchecked.
disposition: fixed by n1: replaced by `a_removal_of_a_persona_with_no_member_id_is_refused_with_nothing_deleted` (`org-node/tests/own_removal.rs`), which drives `verify` with a Persona with no MemberId and asserts the refusal and an unchanged store; red against a planted `persona.member_id.unwrap_or(MemberId::new([0; 32]))`.

**finding-2**: guideline, low — Seams 1 — org-node/tests/revocation.rs:176-179 (T1-N2): the source scan for `if let Ok` is brittle and incomplete.
disposition: fixed by n1: the scan is dropped; the equality assertion is the seam test, at boundary epochs 127, 128 and u64::MAX, in `signed_bytes_are_the_domain_then_the_tuple_at_each_varint_length` (`org-node/tests/revocation.rs`), red at "epoch 128" against a planted `while value > 0x80`.

**finding-3**: guideline, medium — Seams 1/2 — org-io/tests/acknowledgements.rs:502-524 (T1-O1): tests crate-private `dial_plan` through the one-caller wrapper `test_support::acknowledgement_dial_plan`; stays green if `try_addresses` stops using it; LLR-u2j6u8's Networked rule and REQ-kn8zg6 have no test observing a delivery.
disposition: fixed by o1 and q1r: the plan test and the wrapper are deleted; handle-level Networked tests observe the delivery: `networked_a_recipient_with_no_kept_address_is_dialled_by_its_key` and `networked_a_recipient_whose_only_kept_address_does_not_parse_is_dialled_by_its_key` (`org-io/tests/acknowledgements.rs`). o1's first version could assert only a `Transport(_)` failure, which exposed the design conflict (the direct-only bind of LLR-cqr26z); after owner ruling Q1', task q1r bound the dial by key with the relay and lookup (LLR-spckm6), and the test asserts the Admin's receive step reports Acknowledged, red against the direct-only bind.

**finding-4**: guideline, medium — UI 4 — app/tests/removed.view.test.ts (T1-A1): the loading state is untested (`loaded &&` removable); rows → refusal → rows on Refresh is untested.
disposition: fixed by a1: 'while the first answer is in flight it says nothing of the list, then shows it' (red with `loaded &&` removed), 'a refused Refresh replaces the rows with its text, and the next answer replaces the text with rows', and 'shows the selected Organisation's removed Devices, and reads them again for another selected' in `app/tests/removed.view.test.ts`, with the edited reads test asserting that the replaced row is gone and that another event reads nothing.

**finding-5**: guideline, low — UI 1 vs Seams 2.4 — app/tests/removed.view.test.ts (T1-A2): the clause itself, not the diff.
disposition: no change — the finding is against the guideline clause, not the diff; a guideline tailoring is the owner's (as on 2026-10-28), so it is recorded and not fixed here.

### Round 2

TEST app raised no finding in round 2; its notes were taken up by a2 (a two-Device answer checking org-io's order, LLR-cs9bmw's wording rule on the empty list and a refusal, stale LLR-kk8gc7/LLR-7bk6qh comments).

**finding-1**: guideline, low — Existing suites 1 / Seams 1 — org-node/tests/epoch_store.rs:194-197 (T2-N1): `store_data_holds_no_provisional_updates_field` was re-pointed in place but stays a source scan.
disposition: fixed by n2: `store_data_holds_no_provisional_updates_field` (`org-node/tests/epoch_store.rs`) is rewritten at the encoding seam, red against a planted `provisional_updates` field in `StoreDataEncoding`. The notes were taken up too: `OrgNotHeld` pinned in `removed_devices_lists_each_device_held_as_revoked_with_its_acknowledgement`, and LLR-dsjht5's removed-store case added as `the_removed_devices_of_a_removed_store_are_organisation_removed`; LLR-xudkh4's failed-seal case is unreachable (see Gaps).

**finding-2**: guideline, low — Seams 1 — org-io/tests/absences.rs:680-686 (T2-O1): the scan checks names no live LLR states.
disposition: fixed by o2: narrowed to LLR-zhdc9f as `the_handle_offers_none_of_the_removed_methods` (`org-io/tests/absences.rs`), red against a planted `pub fn is_own_admin`. Notes taken up by o2: `networked_after_a_failed_relay_bind_the_other_recipients_are_dialled_as_before` and `networked_a_recipient_whose_kept_addresses_all_fail_is_not_dialled_by_its_key` (`org-io/tests/acknowledgements.rs`), and the stale "new tests carry both IDs" lines in `on-chain-client/docs/architecture/2026-10-13-publish.md`; LLR-spckm6's close clause is unobservable (see Gaps).

### .guardrails/templates/CODE_GUIDELINES.md

Governs: every code path of the diff in org-node, org-io, app and on-chain-client — org-node/src (error.rs, removal.rs, revocation.rs, service.rs, store.rs, verify_org.rs) and its tests; org-io/src (acknowledge.rs, connect.rs, endpoints.rs, lib.rs, removed.rs, the deleted signatory.rs, test_support.rs, transport/endpoint.rs, verify.rs), its tests, org-io/Cargo.toml and org-io/.guardrails/config.yaml; app/src-tauri/src (commands.rs, lib.rs), app/src/lib (api.ts, removed.ts, components/RemovedDevices.svelte, components/Revoke.svelte) and the app tests; on-chain-client/tests; the Makefile and .github/workflows/rust.yml.

### Round 1

**finding-1**: guideline, medium — Comments 2 — org-node/tests/encoding_golden.rs:71, :75 (C1-N1): "added 2026-11-03, plan task T16c4/T16g … 2108 → 2140 → 2355".
disposition: fixed by n1: the narration is removed from `org-node/tests/encoding_golden.rs`.

**finding-2**: guideline, medium — Comments 2 — org-node/tests/own_removal.rs:696 (C1-N2): "(PR-m9betm: the skip is gone)".
disposition: fixed by n1: the comment is gone with the test it sat on, replaced by `a_removal_of_a_persona_with_no_member_id_is_refused_with_nothing_deleted` (`org-node/tests/own_removal.rs`).

**finding-3**: guideline, low — Comments 2 — org-node/tests/own_removal.rs:634 (C1-N3): the assertion message "the old order".
disposition: fixed by n1 in `org-node/tests/own_removal.rs`.

**finding-4**: guideline, medium — Names 4 — org-node `removed_devices` / `RemovedDeviceView` (C1-N4): against the codebase's `revoked_devices`, "held as revoked".
disposition: fixed by n1 under a dispatcher ruling: the glossary's user-facing term is "removed Device", so the names stay (LLR-dsjht5, app `list_removed_devices`); n1 defined **Removed Device** in `docs/CONTEXT.md`, relating it to "held as revoked" and `revoked_devices`, and the view's doc comment uses it.

**finding-5**: guideline, low — Names 2 — org-node/src/service.rs:404-409 (C1-N5): `.filter(|_| …)` standing in for an `if`.
disposition: fixed by n1 in `org-node/src/service.rs`.

**finding-6**: guideline, low — Names 3 — org-node/tests/own_removal.rs:629 (C1-N6): `two`.
disposition: fixed by n1 in `org-node/tests/own_removal.rs`.

**finding-7**: guideline, low — org-io/.guardrails/config.yaml:65 (C1-O1): "(was 30)".
disposition: fixed by o1 in `org-io/.guardrails/config.yaml`.

**finding-8**: guideline, low — org-io/src/test_support.rs:4 (C1-O2): "former `FakeWriter`".
disposition: fixed by o1 in `org-io/src/test_support.rs`.

**finding-9**: guideline, low — org-io/src/test_support.rs:391-399 (C1-O3): a one-caller wrapper.
disposition: fixed by o1 with T1-O1: `test_support::acknowledgement_dial_plan` is deleted from `org-io/src/test_support.rs`.

**finding-10**: guideline, low — org-io/src/removed.rs (C1-O4): `RemovedDevicesRefused::Node` is unreached and `Error::source` is unasked.
disposition: fixed by o1: `RemovedDevicesRefused::Node` is kept with a comment why it is unreachable, and `Error::source` is dropped (`org-io/src/removed.rs`).

**finding-11**: guideline, low — org-io/src/removed.rs:50-55 (C1-O5): `views` / `view`.
disposition: fixed by o1 in `org-io/src/removed.rs`.

**finding-12**: guideline, low — org-io/tests/acknowledgements.rs:512, :516 (C1-O6): `two`.
disposition: fixed by o1 in `org-io/tests/acknowledgements.rs`.

**finding-13**: guideline, medium — Names 1/4 — app/src/lib/components/RemovedDevices.svelte:25 (C1-A1): `catch (e)` → `refusal`.
disposition: fixed by a1 in `app/src/lib/components/RemovedDevices.svelte` (the state variable renamed `refusalText` to avoid shadowing).

**finding-14**: guideline, medium — Names 3/4 — app/src/lib/components/RemovedDevices.svelte:42, app/src/lib/removed.ts:14 (C1-A2): `result` → `entry`.
disposition: fixed by a1 in `app/src/lib/components/RemovedDevices.svelte` and `app/src/lib/removed.ts`.

**finding-15**: guideline, low — app/src/lib/components/RemovedDevices.svelte:46 (C1-A3): `fn`.
disposition: fixed by a1 in `app/src/lib/components/RemovedDevices.svelte`.

**finding-16**: guideline, low — Names 4 — app/tests/removed.view.test.ts (C1-A4): the test's `SILENT` against the fixture's `one_unacknowledged`.
disposition: fixed by a1 in `app/tests/removed.view.test.ts`.

**finding-17**: guideline, low — Simplicity 4 — app/tests/removed.view.test.ts (C1-A5): the test constants repeat backend.json.
disposition: fixed by a1: the tests read the fixture `app/tests/contract/backend.json`.

**finding-18**: guideline, low — Errors 2 — app/src-tauri/tests/ipc.rs:1447 (C1-A6): `is_ok_and` swallows a refusal into a 30 s timeout.
disposition: fixed by a1 in `app/src-tauri/tests/ipc.rs`, with the :1383 "Normal" and :1402 "two shapes" comments.

### Round 2

**finding-1**: guideline, low — Names — org-node `removed_devices` against `revoked_devices` (C2-N1): two names, repeating C1-N4 at low.
disposition: no change — the dispatcher's round 1 ruling stands: the glossary's **Removed Device** entry in docs/CONTEXT.md ties the two names; at low it does not count as a recurrence.

**finding-2**: guideline, low — Names — org-node/src/service.rs:1011, org-node/tests/own_removal.rs:642 (C2-N2): "sender" for the Delivering Device.
disposition: fixed by n2: renamed in `org-node/src/service.rs` and `org-node/tests/own_removal.rs`; the test `the_sender_of_the_accepted_notice_is_the_first_recipient` is now `the_delivering_device_of_the_accepted_notice_is_the_first_recipient`.

**finding-3**: guideline, low — Names — org-node (C2-N3): `value`, `entry`.
disposition: fixed by n2 in `org-node/src/service.rs`.

**finding-4**: guideline, low — org-node/src/removal.rs:196-197 (C2-N4): one statement doing two things, and `delivering` rebound.
disposition: fixed by n2 in `org-node/src/removal.rs`.

**finding-5**: guideline, low — Names — org-io (C2-O1): `relay_plan` / `plan()` / "Networked plan" for the handle's bind plan.
disposition: fixed by o2: one name, `bind_plan` (`org-io/src/endpoints.rs`), with a note on LLR-spckm6.

**finding-6**: guideline, low — org-io/tests/receive_step.rs:484 (C2-O2): the store tag "receive-ack-no-write" contradicts its test.
disposition: fixed by o2 in `org-io/tests/receive_step.rs`.

**finding-7**: guideline, low — Names — app (C2-A1): `removed` for one Device and for the list.
disposition: fixed by a2 in `app/src/lib/components/RemovedDevices.svelte`.

**finding-8**: guideline, low — app (C2-A2): the one-caller `rereadsAfter`.
disposition: fixed by a2: inlined into `app/src/lib/components/RemovedDevices.svelte`.

**finding-9**: guideline, low — app/src/lib/components/RemovedDevices.svelte:46 (C2-A3): `subscribeAll(...).then` with no rejection handler.
disposition: fixed by a2: the refusal is shown on the panel; test 'a refused subscription is shown on the panel, and the rows still are' (`app/tests/removed.view.test.ts`), red before the fix.

**finding-10**: guideline, low — app/src-tauri/tests/ipc.rs:1402 (C2-A4): the comment retells the test.
disposition: fixed by a2 in `app/src-tauri/tests/ipc.rs`.

### .guardrails/templates/ARCHITECTURE_GUIDELINES.md

Governs: every path of the diff in org-node, org-io, app and on-chain-client, with the root docs/CONTEXT.md, docs/adr/2026-10-06-org-io-unit.md, the Makefile and .github/workflows/rust.yml, checked against each unit's architecture ledgers.

### Round 1

**finding-1**: guideline, medium — Domain 2 — docs/CONTEXT.md (A1-1): "removed Device" is public in three units and the UI, and not in docs/CONTEXT.md.
disposition: fixed by n1 with C1-N4: the **Removed Device** entry in `docs/CONTEXT.md`.

**finding-2**: guideline, low — Domain 1 — org-node/src/store.rs:993 (A1-2): `KeptAcknowledgementFrom.ack` uses an _Avoid_ synonym.
disposition: fixed by n1: the new field is renamed `acknowledgement` (`org-node/src/store.rs`; the encoding is positional, so the golden bytes are unchanged; dated notes on LLR-py43n3, LLR-xudkh4, LLR-dsjht5). The existing `KeptAcknowledgement.ack` is not this change's.

**finding-3**: guideline, low — Boundaries 3 — SDD-acnpu4 (A1-3): "kept … and listed …"; its code claim overlaps SDD-busuj3.
disposition: fixed by n1: SDD-acnpu4 has a one-sentence purpose and a code claim without the overlap, by dated note (`org-node/docs/architecture/2026-11-02-publish-2.md`); the ownership of LLR-xudkh4's append was settled by merge review MR1-N6.

**finding-4**: guideline, low — Boundaries 3 — SDD-g9ugu7 (A1-4): "what … shows … and when it asks".
disposition: fixed by a1: SDD-g9ugu7 has a one-sentence purpose, by dated note (`app/docs/architecture/2026-11-02-publish-2.md`).

### Round 2

**finding-1**: guideline, low — Boundaries 3 — SDD-g9ugu7 (A2-1): its text still has "and".
disposition: fixed by a2: SDD-g9ugu7 amended in place to the note's sentence, purpose and code claim (`app/docs/architecture/2026-11-02-publish-2.md`).

**finding-2**: guideline, low — Domain 3 — org-node and org-io (A2-2): "kept acknowledgement" means the outgoing one; the incoming entry is `KeptAcknowledgementFrom`.
disposition: fixed by n2 and o2: n2 renamed `KeptAcknowledgementFrom` to `AcknowledgedEntry` in org-node code and tests (`org-node/src/store.rs`), with SDD-acnpu4, LLR-xudkh4, LLR-py43n3 and LLR-f5wrjx amended in place (golden bytes unchanged); o2 reworded `org-io/src/removed.rs` and LLR-3ydu9e to "an entry of the Acknowledged record"; the dispatcher reworded the remaining "the acknowledgement kept from it" comments in `org-node/src/service.rs` and `org-node/tests/receive_path.rs`.

**finding-3**: guideline, low — Existing code 1 — app (A2-3): two more `hex::encode` sites.
disposition: no change — it is the existing pattern, a pure codec used throughout the app, which Existing code 1 asks the change to follow.

## Gaps

What this change did not establish:

- **LLR-xudkh4's failed-seal case is unreachable.** A failed seal keeps
  nothing by construction (the entry is computed before
  `check_acknowledgement` and appended through `self.change` only after
  acceptance), but no test drives a failed seal: postcard cannot fail for
  these types, and XChaCha20-Poly1305 fails only past its length limit
  (recorded by task n2).
- **LLR-spckm6's close clause is unobservable.** "Closed once the dial's
  outcome is read" cannot be observed at any seam (iroh 0.98 aborts an
  unclosed endpoint on drop; the report and the receive step are the same
  either way). The clause was moved to design prose by a dated note (merge
  review MR1-O3); no test verifies it.
- **The PR-y6ay44 test cannot fail if the fallback returned.** The guarded
  serialisation cannot fail for these types, so a reintroduced fallback
  would never be taken. The fix removed the path; the test,
  `signed_bytes_are_the_domain_then_the_tuple_at_each_varint_length`, pins
  the bytes (LLR-bs6yxf) at each varint length.
- **Coverage.** No coverage gap has been accepted for this change, because
  none has been measured yet: the step 6a gate measures coverage, org-io's
  included (the deleted `signatory.rs` shell no longer counts). Any shortfall
  it finds is a decision still to be made, and is not stated here.
- **Open and residual.** PR-gg4zwz is open: the Admin's removed-Devices list
  shows a Device key re-admitted after its removal, and nothing refuses that
  re-admission. The LLR-bgj2ff residual stands (a Device admitted and removed
  between two epochs a Member adopted is never held as revoked, so its
  acknowledgement is not kept), acceptable until roadmap S5 (owner ruling
  Q5); org-io's Loopback residual stands (a recipient with no kept address is
  not dialled in Loopback mode). PR-7ud5fv is out of scope (owner ruling Q4).
- **Not run here.** `chain_writer_contract` skip-passes without a chopsticks
  fork, and PR-8geawr (no contract test of `FakeChain` against the real
  chain) is accepted, not fixed. `RemovedDevicesRefused::Node` is
  unreachable and untested, with a comment saying why.
- **For the owner.** The `.guardrails/units.yaml` text naming "the own-admin
  check" (MR1-C2), the org-members Makefile coverage list (MR1-C9), and the
  review checklist against this tree's guardrails version (MR1-N7, MR1-O7)
  are recorded, not changed. on-chain-client's signatory-set read is kept
  with no consumer (owner ruling R3), and the derivation that justified
  REQ-b8x2py and REQ-8p2veg, org-io's own-admin check, is gone.
