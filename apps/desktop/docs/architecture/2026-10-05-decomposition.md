# app — software decomposition, items and low-level requirements

The first architecture ledger for the `app` unit: eleven software items and the
low-level requirements that refine the unit's high-level requirements in
`apps/desktop/docs/requirements/`. The SOUP inventory is
`apps/desktop/docs/architecture/soup.md`.

Safety class C throughout (`apps/desktop/.guardrails/config.yaml`, no per-item
override), so every item carries low-level requirements, **except SDD-6g3wnh,
which carries none.** That is a deviation, not an override. Its section, "The
item with no low-level requirements", records it in full.

Written under an owner ruling of 2026-10-05 (the architecture change):
a low-level requirement is written here only where a gated test reddens when
its behaviour breaks. A behaviour that no gated test reaches is listed in the
deviation item instead. New tests were written only where the existing harness
already reaches the code: the command handlers driven over Tauri's mock IPC on
`AppState::for_test`, the `events.rs` vocabulary, the parsing paths, and the
extracted frontend decisions under vitest.

**How each requirement is verified.** Under each low-level requirement,
"Normal" and "Abnormal" name the gated tests that are its normal-case and
abnormal-input cases. A Rust test is named by function, with its file
(`apps/desktop/src-tauri/tests/<file>.rs`). A vitest test is named by its `it(...)`
title, with its file (`apps/desktop/tests/<file>.test.ts`). Each test carries
`verifies:` with the requirement it is listed under.

Of the unit's twenty-three high-level requirements, twenty-two are traced by
an item here. REQ-x3c8n2 is not: it is an expectation on `org-node`, not a
behaviour of this unit.

## Overview

The unit's overview lives in the Overview section of
`apps/desktop/docs/architecture/README.md`, the standing description every later change
amends.

## SDD-k95rmp — Startup policy

`apps/desktop/src-tauri/src/policy.rs`: `StartupError` and its `Display`,
`DEV_PASSPHRASE`, `DEV_DATA_DIR`, `resolve_passphrase`, `resolve_data_dir`,
`transport_mode_from` and `TransportModeName`.

**SDD-k95rmp**: the startup decisions, made from values handed in: the
passphrase the persona store is opened under, the data directory it lives in,
the transport mode, and the refusal, with its message, when a required value
is missing. Every function is total and none reads the process environment.
traces: REQ-7g3k9a, REQ-rxc8sp, REQ-bmk2z2, REQ-645jq9, REQ-ergn9z

*Amended 2026-10-20 (change `worktree-org-io-domain-api`; owner ruling Q2
and answer A1 of 2026-10-16; an extension, the ID is kept):* the startup
decisions also include the verification interval: `resolve_verification_interval`
and `StartupError::InvalidVerificationInterval` join the file list, and
`traces:` gains REQ-ergn9z. Its two low-level requirements (LLR-2dpn52,
LLR-rvrc25) are in `2026-10-16-domain-api.md`, under
"Under SDD-k95rmp — The verification interval". The functions stay total and
read no environment; `init` (SDD-6g3wnh) reads the variable and hands it in.

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).* The app's
rule that `AppState::init` is the only reader of the process environment
holds for every variable but one: `ODS_ADMIN_SEED`, the development signing
seed, is read by org-io, and only in a build with org-io's `dev-seed`
feature (the app's own `dev-seed` feature enables it); the app never reads
it (accepted recommendation, 2026-10-06). The reason for the rule — cargo
runs a binary's tests as threads of one process, so a test that set a
variable would race the rest — holds for org-io's reader too: org-io's tests
exercise its parser on values, and its one test that sets the variable runs
alone in its own test binary. Classification: a clarification, not a change
of meaning.

**LLR-tpkmh3**: `resolve_passphrase` returns the configured passphrase,
unaltered, when it is non-empty, whether or not development defaults are
enabled. When the value is absent, it returns `DEV_PASSPHRASE` if development
defaults are enabled. When the value is absent or empty and development
defaults are not enabled, it returns `StartupError::MissingPassphrase`.
satisfies: REQ-7g3k9a

Normal: `configured_passphrase_is_used`, `configured_passphrase_wins_over_opt_in`,
`absent_passphrase_with_opt_in_uses_dev_default` (startup_policy).
Abnormal: `absent_passphrase_is_refused`, `empty_passphrase_is_refused_not_honoured`
(startup_policy).

Cut by the falsifiability sweep (task T5 of the plan): that an empty value
with development defaults enabled yields `DEV_PASSPHRASE`. No gated test
passes an empty value with the opt-in, and returning the empty string there
reddened nothing (run T5-A05).

*Amended 2026-10-14 (change `worktree-org-io-transport`, S4 task T17;
REQ-7g3k9a amended the same day, owner ruling of 2026-10-06, IDs kept).* `resolve_passphrase`,
`DEV_PASSPHRASE` and `StartupError::MissingPassphrase` are deleted, and
LLR-tpkmh3 now reads: *no source of the unit reads or names
`ODS_PASSPHRASE`; when org-io's `OrgIo::open` refuses because the OS
keychain refuses a Persona's item, `AppState::init` (and `for_test`, which
shares the open) returns that refusal as the startup failure, its message
naming the Persona's store file and the condition (for a locked keychain,
"locked").* The behaviour now sits in `state.rs` (SDD-aq7m6b's
constructors, `init` being SDD-6g3wnh's); the item keeps its place here
because its ID is kept. The five tests listed above were deleted with
`resolve_passphrase`.

Normal: `startup_reads_no_passphrase` (startup_policy).
Abnormal: `a_keychain_refusal_fails_startup_naming_the_condition`
(startup_policy).

SDD-k95rmp is narrowed with it: its startup decisions no longer include a
passphrase, and `DEV_PASSPHRASE` and `resolve_passphrase` leave its file
list. `traces: REQ-7g3k9a` is kept for the item above.

**LLR-kfmng5**: when development defaults are not enabled, `resolve_data_dir`
returns a non-empty override in preference to the path the platform resolver
returned, that path when there is no override, and otherwise
`StartupError::UnresolvableDataDir`. An empty override counts as absent. When
development defaults are enabled and there is neither, it returns
`DEV_DATA_DIR`.
satisfies: REQ-rxc8sp

Normal: `override_data_dir_is_used`, `resolved_data_dir_is_used_when_no_override`,
`unresolvable_data_dir_with_opt_in_uses_temp` (startup_policy).
Abnormal: `unresolvable_data_dir_is_refused`, `empty_override_is_ignored_not_used_as_path`
(startup_policy).

Narrowed by the sweep: the override and the resolved path taking precedence
over `DEV_DATA_DIR` when development defaults are enabled. No gated test
combines the opt-in with an override or a resolved path, and dropping either
precedence reddened nothing (runs T5-A09, T5-A11).

**LLR-97rww8**: each `StartupError` message names the environment variable that
would supply the value (`ODS_PASSPHRASE`, `ODS_DATA_DIR`) before
`ODS_ALLOW_DEV_DEFAULTS`, the variable that would waive the requirement. This
holds whether the value was unset or set to the empty string.
satisfies: REQ-bmk2z2

Normal: `passphrase_refusal_names_both_variables_supplier_first`,
`data_dir_refusal_names_both_variables_supplier_first` (startup_policy).
Abnormal: `refusals_for_empty_values_name_both_variables_supplier_first`
(startup_policy).

*Narrowed 2026-10-14 (S4, task T17):* `StartupError` has one variant left,
`UnresolvableDataDir`, so the item covers `ODS_DATA_DIR` only.
`passphrase_refusal_names_both_variables_supplier_first` was deleted with
the passphrase refusal, and the abnormal test lost its passphrase half
(its annotation unchanged). Normal: `data_dir_refusal_names_both_variables_supplier_first`.

**LLR-j5pacp**: `transport_mode_from` returns Loopback for exactly the string
`loopback`, and Networked for every other value, including an absent one,
an empty one and `LOOPBACK`.
satisfies: derived

Normal: `loopback_transport_is_reported`, `networked_transport_is_reported`
(connection_status).
Abnormal: `unknown_transport_value_is_networked` (connection_status).

## SDD-aq7m6b — Application state assembly

`apps/desktop/src-tauri/src/state.rs`: `AppState`, `AppState::assemble`, and
`AppState::for_test` (test-support only).

**SDD-aq7m6b**: the Tauri-managed state, and its assembly from decided values.
It creates the data directory, opens the persona store in it, wires
`OrgService` to the store and a chain implementation, and records the
directory, chain endpoint and transport mode the running configuration was
built from. `AppState::init` and `AppState::for_test` both build through
`assemble`. `init` reads the environment and is SDD-6g3wnh's, as is
`transport_mode_for`.
traces: REQ-bvx4nh

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).* The state
holds one org-io handle (`AppState.org_io: Mutex<OrgIo>`), not `OrgService`
wired to a chain implementation plus a separate chain writer: the item wires
the service over the store into the handle, and org-io owns the chain read,
the chain write and the signatory key. The store is opened by
`open_service` (the data directory, the store, the transport mode), which
`init` and `for_test` both call before building the handle; `assemble` then
opens the outstanding Invites beside the store and records the directory,
the chain endpoint and the transport mode. `for_test` builds the handle with
the chain not configured (`OrgIo::not_configured`). Classification under the
hybrid rule: a clarification of the item's code and wiring, not a change of
meaning.

*Moved 2026-10-13 (change `worktree-org-io-transport`, S4 task T9).*
The item that stated "create the data directory and open the store in it,
refusing rather than substituting" moved to org-io, rewritten per Persona
(one store file per Persona, unsealed through its keychain item), with its
S2 notes and its two movable tests: see
`crates/org-io/docs/architecture/2026-10-07-transport.md`,
under the store files. The app keeps its scan of `init` (one open) and its
IPC test of the reported directory; task T17 settles what they cite.

*Amended 2026-10-14 (S4, task T15; a clarification of the wiring, the ID is
kept).* `init` and `for_test` hand the data directory and the transport
mode to org-io's `OrgIo::open`, which opens every Persona's store file with
its keychain item (`for_test` installs org-io's in-memory keychain first;
`for_test_over` opens over org-io's substitute chain). `assemble` no longer
opens outstanding Invites beside the store: they live in the founding
Persona's sealed store, kept by org-io's exchange. It records the
directory, the chain endpoint and the transport mode, as before.
`transport_mode_for` maps the app's mode name to org-io's
`transport::TransportMode` (the transport left org-node). The passphrase is
still read by `init` and refused when absent, but no longer handed on;
task T17 removes it.

*Amended 2026-10-14 (S4, task T17; a clarification, the ID is kept).* No
passphrase anywhere: `init` reads none and `for_test` takes none (its
parameter is gone). A refusal from `OrgIo::open`, a keychain refusal
included, is returned by `init` and `for_test` as the startup failure,
verbatim (LLR-tpkmh3 as amended). The app's scan of `init`
(`state_assembly::init_opens_the_store_once_even_when_the_connect_is_refused`)
cited only the item that moved to org-io and now runs with no `verifies:`
line and a dated pointer in the test file.

*Note 2026-10-15 (S4, owner ruling of that day):* that scan,
`init_opens_the_store_once_even_when_the_connect_is_refused`, was deleted:
no app item states it; it is covered by org-io's store-opening tests and the
app's startup-refusal tests. It was the last test in `tests/state_assembly.rs`,
so the file, its `[[test]]` target and its entry in the app's verify line
went with it. No item changes.

**LLR-vqkr5t**: `AppState.data_dir` is the directory the store was opened in,
and the `connection_status` command reports exactly that directory.
satisfies: REQ-bvx4nh

Normal: `connection_status_reports_the_directory_the_store_was_opened_in` (ipc).
Abnormal: `connection_status_reports_a_data_dir_it_had_to_create_verbatim` (ipc).

*Amended 2026-10-14 (S4, task T17; a clarification, the ID is kept):* "the
store" is every Persona's store file, which org-io opens in that directory
with each Persona's keychain item; no passphrase is involved. The abnormal
test dropped its second ID (the item that moved to org-io in task T9) with
a dated note; it verifies this item alone.

## SDD-q4dpym — Connection-status reporting

`apps/desktop/src-tauri/src/policy.rs`: `ChainEndpoint`, `ConnectionStatus`,
`connection_status_from_state`. `apps/desktop/src-tauri/src/commands.rs`:
`connection_status`.

**SDD-q4dpym**: the projection of the running configuration into the
`ConnectionStatus` wire object, served over IPC without re-reading the
environment.
traces: REQ-645jq9, REQ-e4ah9h, REQ-bvx4nh

**LLR-9x6qrg**: `connection_status_from_state` reports `chain_configured` true
exactly when it is handed a chain endpoint, and reports `chain_ws` and
`contract_h160` exactly when it is; otherwise it reports `chain_configured`
false and both fields absent.
satisfies: REQ-e4ah9h

Normal: `configured_chain_reports_its_endpoint_and_contract` (connection_status).
Abnormal: `unconfigured_chain_reports_neither`,
`absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env`
(connection_status).

**LLR-kze6ak**: the endpoint and contract address it reports are the values it
was handed, verbatim, not normalised. The data directory it reports is the
path it was handed: it is not made absolute, and an empty one is not replaced
by a fallback.
satisfies: REQ-bvx4nh

Normal: `endpoint_comes_from_the_built_configuration`,
`data_dir_comes_from_the_built_configuration`,
`configured_chain_reports_its_endpoint_and_contract` (connection_status).
Abnormal: `abnormal_data_dir_paths_are_reported_verbatim` (connection_status).

`configured_chain_reports_its_endpoint_and_contract` was added to this list
by review round 1 (finding-7): it was the only test the sweep's run T5-A42
(the contract address reported other than verbatim) reddened.

Narrowed by the sweep. Two clauses reddened nothing and were cut. First, that
an empty endpoint or contract address is not replaced by a fallback: no gated
test hands an empty one (runs T5-A46, T5-A47). Second, that the data directory
is not normalised: collapsing its components passes, because no gated path has
a redundant separator or a `.` component (run T5-A43).

**LLR-8tzbzn**: the command registered under the name `connection_status`
returns the fields `chain_configured`, `chain_ws`, `contract_h160`,
`transport_mode` and `data_dir`. `transport_mode` is the configured mode,
serialised as `networked` or `loopback`.
satisfies: REQ-645jq9

Normal: `connection_status_reports_transport_mode_over_ipc`,
`connection_status_is_registered_under_its_name` (ipc);
`networked_transport_is_reported`, `loopback_transport_is_reported`,
`transport_mode_serialises_lowercase` (connection_status).
Abnormal: `unknown_command_is_rejected` (ipc): the near-miss name
`connectionStatus` is refused.

## SDD-2pa6h6 — Command-boundary parsing

`apps/desktop/src-tauri/src/parsing.rs`: `parse_org_id`. In
`apps/desktop/src-tauri/src/commands.rs`: the argument parsing in `create_persona`,
`export_invite`, `admit_member` and `revoke_member`, up to the call into
`OrgService`.

**SDD-2pa6h6**: refusal of malformed operator-supplied identifiers and values
at the IPC edge, before they address a service call.
traces: REQ-sjkp8z

*Traces amended 2026-10-20 (change worktree-org-io-domain-api): the
peer-address precondition's requirement is dropped (retired 2026-10-20, owner
answer A2; no peer address reaches the IPC
edge any more).*

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`).* The
invitation exchange left org-node for this unit, so the Invite and the Invite
reply are parsed here, at this unit's edge: `Invite::parse` and
`InviteReply::parse` in `apps/desktop/src-tauri/src/invitation.rs` (under a Blob-parse
low-level requirement since replaced), and the item traced the Blob-parse
requirement. `admit_member` takes a reply Blob and a
peer address in place of the join-request Blob (the admission's peer-address
item, since retired with the pasted address).

*Amended 2026-10-13 (change `worktree-org-io-transport`, S4 task T6).*
The Invite and its reply return to org-node, which parses them; the Blob-parse
requirement and its low-level requirement moved there with their tests
(ruling A), so `traces:` drops that requirement. The commands still call the
parse, through the org-io handle's re-export.
*(2026-10-14, S4 task T15: `import_invite` calls it through org-io's
narrowed re-export, `org_io::node::Invite`; the reply is parsed inside
org-io's exchange; the peer address by org-io's `parse_peer_addr`.)*

**LLR-ecaw34**: `parse_org_id` strips at most one leading `0x`. It then accepts
exactly forty hexadecimal characters, in either case, as twenty bytes of the
values they encode. Any other length is refused with `org_id must be 40 hex chars, got N`,
where N is the length after the strip. The width is checked before the
alphabet. A non-hexadecimal character is refused with a message beginning
`org_id hex`.
satisfies: REQ-sjkp8z

Normal: `forty_hex_characters_are_accepted`,
`a_zero_x_prefix_is_accepted_and_parses_identically`, `uppercase_hex_is_accepted`,
`a_zero_x_prefix_with_forty_characters_after_it_is_accepted` (org_id_parsing).
Abnormal: `an_empty_string_is_refused`, `thirty_nine_characters_are_refused`,
`forty_one_characters_are_refused`,
`an_odd_length_is_refused_on_width_not_on_hex_decoding`,
`a_non_hex_character_at_the_first_position_is_refused`,
`a_non_hex_character_at_the_last_position_is_refused`,
`forty_non_hex_characters_are_refused`, `a_doubled_zero_x_prefix_is_refused`,
`a_zero_x_in_the_middle_is_refused`, `the_bare_prefix_alone_is_refused`
(org_id_parsing).

Narrowed by the sweep: the order of the twenty bytes is not part of this
requirement. Every gated input repeats a single byte value, so reversing the
bytes reddened nothing (run T5-A77). Changing a value does redden (T5-A78).

## Under SDD-6pjvhc — The command surface

Low-level requirements of the command surface written before SDD-6pjvhc
(`2026-10-16-domain-api.md`) replaced the earlier surface; they are unchanged
and sit under it.

**LLR-5wb4vd**: `AppState` holds the org-io handle as a shared `Arc<OrgIo>`
and no command and not the receiver loop takes an app-side lock on it; the
receiver loop calls `OrgIo::start_receiving` and `OrgIo::next_received` on
its own clone. A command invoked while the receiver loop waits for a message,
with no peer sending, completes, and the loop is still running after it.
satisfies: derived

Normal and abnormal: `the_receiver_holds_no_app_lock_while_waiting`
(receiver_guard): a Persona's endpoint bound, the loop waiting, and
`list_personas` bounded at five seconds; red against the app's
`Mutex<OrgIo>`, where it timed out.
Abnormal: `a_refused_and_a_mutating_command_complete_while_the_receiver_waits`
(receiver_guard): while the loop waits, a `create_organisation` org-io
refuses returns its refusal and a second `create_persona` completes, each
within the bound, and the loop is still running after both. *Added to this
list 2026-10-14 (review round 1); the test is S4 task T23's.*

**LLR-yrb6th**: when org-io's receive step returns its terminal condition
because the Persona's endpoint closed, the receiver loop `start_receiver`
spawned emits `receiver-stopped` exactly once, naming the reason, ends, and
releases the start guard; a later `start_receiver` then starts a loop that
runs, holding the guard and waiting for the next message.
satisfies: REQ-3hfggn, REQ-jfxah3

*Added 2026-10-14 (change `worktree-org-io-transport`, S4 task T19; the
owner's ruling 2 of that day on the dead-receiver hazard).* The first test
reaches the real loop, which until T16 no gated test did (SDD-6g3wnh's
"untested shell" names `start_receiver`; this item states the part a test now
reaches). Not derived: it states, for the real loop, what REQ-3hfggn and
REQ-jfxah3 require, so it needs no derived-requirement assessment.

Normal: `a_closed_endpoint_stops_the_loop_announces_it_once_and_releases_the_guard`
(receiver_guard): the endpoint of a Loopback Persona is closed while the loop
waits. Behaviour that existed, so a mutation check stands in for a red run
(recorded in the plan's T19 report).
Abnormal (the restart after a terminal stop):
`after_a_stop_start_receiver_starts_a_loop_that_runs` (receiver_guard),
`#[ignore]`d. It fails: org-io keeps the closed endpoint and never binds it
again, so the restarted loop finds no accept task and stops at once. That
clause is open under org-io's problem report of 2026-10-14 on the closed
endpoint, and the app's dead-receiver hazard is not lowered until it holds.

*Note 2026-10-14 (review round 1):* the paragraph above is history. S4 task
T20 un-ignored `after_a_stop_start_receiver_starts_a_loop_that_runs`, when
org-io came to bind a closed endpoint again on the next `start_receiving`;
the test now runs and passes, and the restart clause holds.

**LLR-pguhw5**: each persona `list_personas` returns has exactly the fields
`persona_id`, `org_id`, `handle`, `name`, `surname` and `status`. No seed or
other secret crosses IPC. `org_id` is present and null for a persona that
belongs to no Organisation.
satisfies: derived

Normal: `list_personas_reports_exactly_the_persona_fields` (ipc).
Abnormal: `a_persona_in_no_organisation_reports_a_null_org_id` (ipc).

*Clarified 2026-10-28 (change `worktree-org-io-domain-api`, round 3 review,
CODE app note 1; the ID is kept):* `handle` is present and null for a
Persona that holds no Handle yet, such as a provisional Member, as
`org_id` is for a Persona in no Organisation; the webview shows it as
LLR-q735zx states. The rule is unchanged. The fixture of the faked IPC
boundary (`apps/desktop/tests/contract/backend.json`) holds a provisional Member's
entry with `handle` null and a founder's with its Handle, each checked
against the real `list_personas` in ipc.rs.

## SDD-5fchuy — Receiver start guard

`apps/desktop/src-tauri/src/events.rs`: `StartGuard` and its `Drop`.

**SDD-5fchuy**: the single slot that decides whether a receiver loop may
start, and its release when the holder ends. The loop that holds it,
`start_receiver` and `next_outcomes`, is SDD-6g3wnh's.
traces: REQ-6hgm8r, REQ-3hfggn

**LLR-5zd6j8**: `StartGuard::try_claim` returns a guard to exactly one caller
while no guard is held, under concurrent calls too, and returns none while one
is held.
satisfies: REQ-6hgm8r

Normal: `first_claim_succeeds` (receiver_guard).
Abnormal: `second_claim_while_held_fails`, `concurrent_claims_yield_exactly_one_guard`
(receiver_guard).

**LLR-z66f27**: dropping the guard releases the slot, including when it is
dropped by a panic, and a released slot is again claimable by exactly one
caller.
satisfies: REQ-3hfggn

Normal: `claim_succeeds_again_after_the_guard_drops` (receiver_guard).
Abnormal: `guard_releases_when_dropped_by_panic`,
`release_then_concurrent_claims_yield_exactly_one_guard` (receiver_guard).

## SDD-mwqf6x — Receiver outcome classification and event vocabulary

`apps/desktop/src-tauri/src/events.rs`: `ReceiverOutcome`, `classify_receive_error`,
`TERMINAL_ERRORS`, `is_terminal_error`, `outcomes_for_receive_error`,
`Emission`, the payload structs and `emissions_for`.

**SDD-mwqf6x**: the total mapping from a receive-path error to an outcome class
(a verification verdict or a receiver error), the detection of a terminal
error, and the mapping from an outcome to the named events and payloads the
frontend receives. It is the only place the event names and payload shapes
are written down.
traces: REQ-kn5rtx, REQ-affyf5, REQ-jfxah3, REQ-2k7ys4, REQ-dp95pv, REQ-tw4cb5, REQ-k7ay5b, REQ-88npkt

*Amended 2026-10-15 (change `worktree-org-io-domain-api`; an extension, the
ID is kept):* `traces:` gains REQ-k7ay5b and REQ-88npkt. The item also maps
a verification result to outcomes and names the follow-up verification a
received provisional organisation asks for (`outcomes_for_verification`,
`follow_up_for`); its new items are in
`2026-10-16-domain-api.md`.

It depends on REQ-x3c8n2, the expectation that org-node will type the
terminal condition. Until then, `is_terminal_error` matches substrings of
messages org-node formats.

*Amended 2026-10-14 (change `worktree-org-io-transport`, S4 task T16).*
REQ-x3c8n2 is met: org-io's exported REQ-8e57wm types the terminal
condition (`ReceiveError::Terminal`). `TERMINAL_ERRORS` and
`is_terminal_error` are deleted; the item's code gains `Arrival` and
`outcome_for_arrival`, and `outcomes_for_receive_error` takes org-io's
`ReceiveError`. The detection of a terminal error is now a match on that
variant (LLR-zw6jqy), and the outcome of a received
message comes from org-io's `Received` and its `ReceiveKind`.
The paragraph above is history. Classification under the hybrid rule: a
clarification of the item (its responsibilities are unchanged); the change
of meaning is carried by LLR-zw6jqy.

**LLR-usxk57**: a `ReceiveError` outcome is announced as exactly one
`receiver-error` event, whose payload carries the message and no
organisation, epoch, root or verdict, at any depth.
satisfies: REQ-kn5rtx

Normal: `the_receiver_error_payload_carries_a_message_and_no_verification_state`
(receiver_events).
Abnormal: `a_chain_failure_emits_no_verification_event_for_any_message`
(receiver_events).

**LLR-zw6jqy**: `outcomes_for_receive_error` takes org-io's `ReceiveError`.
A `Refused` error gives the one outcome `classify_receive_error` returns for
org-node's refusal it carries. A `Terminal` error gives a `ReceiveError`
outcome carrying its message, followed by a `Stopped` outcome whose reason is
that message. Every other variant (`Connection`, `Chain`, `Store`,
`Keychain`) gives one `ReceiveError` outcome carrying its message and nothing
else. No outcome depends on the text of the message.
satisfies: REQ-jfxah3, REQ-kn5rtx, REQ-x3c8n2

Normal: `terminal_is_decided_by_the_error_variant_not_its_text` (its terminal
half), `every_real_terminal_message_stops_the_loop`,
`the_stop_is_announced_after_the_failure_for_the_same_error`
(receiver_events).
Abnormal: `terminal_is_decided_by_the_error_variant_not_its_text` (non-terminal
variants whose text names an endpoint failure, and an empty terminal
message), `a_non_terminal_failure_does_not_stop_the_loop`,
`a_refusal_this_change_added_does_not_stop_the_loop` (receiver_events).

**LLR-2zmhvs**: a `Stopped` outcome is announced as exactly one
`receiver-stopped` event whose payload carries the reason, even when the
reason is empty.
satisfies: REQ-jfxah3

Normal: `stopped_names_the_reason` (receiver_events).
Abnormal: `stopped_emits_exactly_one_event` (receiver_events).

**LLR-p2nm5a**: an `Updated` outcome is announced as `membership-updated`, then
`incoming-verified`, each carrying the organisation, epoch and root, then
`epoch-changed`, carrying the organisation and epoch. The values are carried
exactly as measured, including epoch 0 and an empty root.
satisfies: derived

Normal: `updated_emits_membership_incoming_and_epoch`,
`updated_payload_carries_the_measured_epoch_and_root` (receiver_events).
Abnormal: `updated_carries_boundary_epochs_and_roots_verbatim` (receiver_events).

Two events carry the same verified update. The frontend files both as rows,
which is booked as PR-bu6mau.

**LLR-hgsdm8**: a `RecordUnreadable` outcome is announced as exactly one
`record-unreadable` event whose payload is the organisation identifier alone,
verbatim, with no membership, verification or epoch event.
satisfies: REQ-2k7ys4, REQ-dp95pv

Normal: `record_unreadable_names_the_organisation`,
`record_unreadable_emits_exactly_one_event` (receiver_events).
Abnormal: `record_unreadable_emits_no_membership_event`,
`record_unreadable_emits_no_epoch_event`,
`record_unreadable_holds_for_any_organisation_id` (receiver_events).

**LLR-2vg79y**: a `SelfDeleted` outcome is announced as exactly one `revoked`
event naming the organisation, whose payload carries no epoch key at any
depth, whatever the organisation identifier. An `AcknowledgementReceived`
outcome is announced by no event.
satisfies: REQ-tw4cb5

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3).*
org-node's self-delete path now also returns a verified acknowledgement from
a revoked Device (`SelfDeleteOutcome::Acknowledged`); the receiver loop maps
it to the new `AcknowledgementReceived { org_id }`, which emits nothing until
org-io keeps acknowledgements (S3b-io). The acknowledgements a self-delete
signs are dropped by the app until S3b-io sends them. Normal:
`a_received_acknowledgement_emits_nothing` (receiver_events).

*Clarified 2026-10-14 (S4, task T16).* The outcomes now come from org-io's
`Received`: `SelfDeleted` from a committed message of kind
`Removed` or `Revoked` (this node's removal by an update, or a revocation
notice accepted), `AcknowledgementReceived` from `Received::Acknowledged`.
org-node's `SelfDeleteOutcome` no longer reaches the app. The announcement
rule above is unchanged; the acknowledgements a self-delete signs are still
dropped by the app. Normal:
`a_revocation_accepted_by_its_notice_is_announced_as_self_deleted`
(receiver_events).

Normal: `self_delete_emits_revoked_naming_the_organisation`,
`self_delete_emits_no_epoch_event`, `self_delete_payload_carries_no_epoch_key_at_any_depth`,
`the_epoch_probe_finds_an_epoch_on_updated_and_none_on_self_delete`
(receiver_events).
Abnormal: `self_delete_carries_no_epoch_for_any_organisation_id` (receiver_events).

**LLR-p38be7**: a `VerifyFailed` outcome is announced as exactly one
`verification-failed` event carrying the message and the organisation, which
is present and null when the outcome names none. No membership event is
emitted with it.
satisfies: REQ-affyf5

Normal: `verify_failure_carries_the_organisation_when_known`,
`verify_failure_carries_the_message`, `verify_failure_emits_no_membership_event`
(receiver_events).
Abnormal: `verify_failure_carries_null_organisation_when_unknown` (receiver_events).

`classify_receive_error` never names an organisation (LLR-wgr46u), so the
known-organisation shape is reachable only from a constructed outcome. The
2026-09-14 risk analysis records this as REQ-affyf5 being vacuous today.
*Note 2026-10-14 (review round 1, finding 3):* REQ-x3c8n2's met half is the
terminal one only; the Organisation of a failure is the open expectation
REQ-q89tjr, and this stays vacuous until org-io meets it.

## SDD-8jw9mn — Verification view model

`apps/desktop/src/lib/verify.ts`: `verifyResultFrom`, `applyReceiverEvent` and the two
list limits. `apps/desktop/src/lib/receiver.ts`: `subscribeAll`.

**SDD-8jw9mn**: the frontend decisions behind the verification view. It builds
a log row from the event alone, files each receiver announcement into exactly
one of two bounded lists, and registers and releases the view's event
listeners as a set.
traces: REQ-a83vqr, REQ-akt4p7, REQ-wu6z9p, REQ-rq8g2v

**LLR-53hayh**: `verifyResultFrom` turns a verified event into a row with
`verified` true and the event's epoch and root, including epoch 0. The row has
a null `detail` and the timestamp it was given, unmodified.
satisfies: REQ-akt4p7

Normal: "renders a verified event as verified", "gives a verified event no
detail" (verify.row).
Abnormal: "preserves epoch 0 on a verified event rather than coercing it to
null", "passes the timestamp through unmodified" (verify.row).

**LLR-a4xwvj**: `verifyResultFrom` turns a failed event into a row with
`verified` false, null epoch and root, the event's message as `detail`, and
the timestamp it was given, unmodified.
satisfies: REQ-a83vqr, REQ-akt4p7

Normal: "renders a failed event as not verified", "gives a failed event no
epoch and no root", "carries the failure message as the row detail"
(verify.row).
Abnormal: "passes the timestamp through unmodified" (verify.row).

**LLR-mzae5q**: a failed row's organisation is the event's, or the text
`(unknown organisation)` when the event's is null.
satisfies: derived

Normal: "keeps the organisation a failed event names" (verify.row).
Abnormal: "renders a placeholder rather than \"null\" when the failure names
no organisation" (verify.row).

**LLR-fb7jp5**: `applyReceiverEvent` files a receiver error only in
`receiverErrors`, as `[<timestamp>] <message>`, leaving the verification log
unchanged. It files a verified or failed event only in the verification log,
leaving `receiverErrors` unchanged. It does not modify the view it was given.
satisfies: REQ-wu6z9p

Normal: "renders a receiver error outside the verification log", "does put a
verification failure in the log and not in the receiver errors", "files the
newest announcement first in each list", "keeps at most 20 receiver errors,
dropping the oldest" (receiver.log).
Abnormal: "produces no verification row for a receiver error", "leaves an
existing verification log untouched", "assigns no verification outcome however
many receiver errors arrive", "does not mutate the view it was given"
(receiver.log).

The last two Normal tests were added by review round 1 (finding-7). They are
the tests that check the `[<timestamp>] <message>` format, and the only ones
the sweep's run T5-C14 reddened.

**LLR-rzx6ks**: each list keeps the newest entry first. The verification log
keeps at most 50 rows and the receiver-error list at most 20 entries; an entry
beyond the limit drops the oldest.
satisfies: derived

Normal: "files the newest announcement first in each list" (receiver.log).
Abnormal: "keeps at most 50 verification rows, dropping the oldest", "keeps at
most 20 receiver errors, dropping the oldest" (receiver.log).

**LLR-z4ky6f**: `subscribeAll` registers every subscription it is given and
resolves only after all of them have registered. It resolves to one cleanup
that cancels each registered listener exactly once, however many times it is
called, including listeners whose registration resolved late. When any
registration fails, it rejects with that failure.
satisfies: REQ-rq8g2v

Normal: "registers every subscription it is given", "unsubscribes exactly the
listeners it registered, including those whose listen call resolved late",
"resolves only after every subscription has registered" (receiver.subscribe).
Abnormal: "is idempotent — calling the cleanup twice unsubscribes once",
"propagates a subscription failure rather than resolving a partial cleanup",
"registers nothing and cleans up cleanly when given no subscriptions"
(receiver.subscribe).

On a rejection, the listeners that had registered are not cancelled. That is
a defect against REQ-rq8g2v, booked as PR-cu2h2g.
This requirement does not claim it.

## Under SDD-czjf2c — Revocation input decision

`apps/desktop/src/lib/revoke.ts`: `validateRevokeInput`.

**LLR-csbs5v**: `validateRevokeInput` trims the member identifier and strips at
most one leading `0x`. It refuses a remainder that is not exactly 64
characters, naming its length, and one that is not hexadecimal.
satisfies: REQ-he8ejb

Normal: "accepts a 0x-prefixed member id", "trims surrounding whitespace before
measuring" (revoke.validate).
Abnormal: "rejects a 63-character member id", "rejects a 65-character member
id", "rejects an empty member id", "rejects a 64-character non-hexadecimal
member id", "rejects a doubled 0x prefix" (revoke.validate).

## SDD-32hath — The webview's Content-Security-Policy

`apps/desktop/src-tauri/tauri.conf.json`: `app.security.csp` and `app.security.devCsp`.

**SDD-32hath**: the policy the webview runs under, which bounds what a script
that reaches the webview can load, run or connect to. It is configuration,
not code: `apps/desktop/src-tauri/tests/csp_policy.rs` holds the checker, the abnormal
cases that show it refuses each weakening, and the normal cases that show the
shipped policy passes it.
traces: REQ-cj5jmx

Each requirement below is one tested clause of REQ-cj5jmx. Its normal case is
the shipped policy (`shipped_csp_meets_every_clause`) and the plan's fixture
policy (`fixture_policy_passes_the_checker`). Its abnormal cases are the
weakenings that the checker is shown to refuse.

**LLR-7ymxtn**: the shipped `csp` is a policy string, not null or empty, whose
`default-src` is exactly `'self'`.
satisfies: REQ-cj5jmx

Normal: `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`
(csp_policy).
Abnormal: `null_policy_is_rejected`, `empty_policy_is_rejected`,
`default_src_other_than_self_is_rejected` (csp_policy).

The sweep found that `default_src_other_than_self_is_rejected` does not
isolate the exactness check. Its `default-src *` is also refused by the
network-origin check (LLR-p3xwx4), so removing the exactness check from the
checker reddened nothing (run T5-D07). The clause itself holds: adding `data:`
to the shipped `default-src` reddens `shipped_csp_meets_every_clause` (T5-D48).
The same is true of `remote_origin_in_script_src_is_rejected` and the
`script-src` check (T5-D16).

**LLR-hdvy6x**: its `script-src` admits `'self'` and no other keyword: no
`'unsafe-inline'` and no `'unsafe-eval'`. It admits no remote origin.
satisfies: REQ-cj5jmx

Normal: `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`
(csp_policy).
Abnormal: `unsafe_inline_in_script_src_is_rejected`,
`unsafe_eval_in_script_src_is_rejected`, `remote_origin_in_script_src_is_rejected`
(csp_policy).

Narrowed by the sweep: that hash and nonce sources are admitted. Neither
policy carries one, and no test offers one, so making the checker refuse them
reddened nothing (run T5-D13). Tauri appends hashes at build time. Whether
the bundled app loads under them is the owner's smoke test.

**LLR-c88jhh**: its `object-src`, `base-uri`, `frame-ancestors` and
`form-action` are each present and exactly `'none'`.
satisfies: REQ-cj5jmx

Normal: `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`
(csp_policy).
Abnormal: `missing_object_src_is_rejected`, `each_none_directive_is_required_to_be_none`
(csp_policy).

**LLR-df7prq**: its `connect-src` is present and admits no source other than
`ipc:` and `http://ipc.localhost`. The development server's origins are not
among them.
satisfies: REQ-cj5jmx

Normal: `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`
(csp_policy).
Abnormal: `connect_src_beyond_ipc_is_rejected`,
`localhost_dev_server_is_rejected_in_shipped_policy` (csp_policy).

Task T2 of the 2026-10-05 architecture change cut the clause that
`connect-src` must name the IPC sources: an empty `connect-src` passes the
checker (T2's mutation M10 reddened nothing).

**LLR-p3xwx4**: no directive names a network origin, wildcard or network
scheme other than the two IPC sources.
satisfies: REQ-cj5jmx

Normal: `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`
(csp_policy).
Abnormal: `wildcard_source_is_rejected_in_any_directive`,
`https_scheme_source_is_rejected_in_any_directive` (csp_policy).

**LLR-ausr5q**: no directive appears twice, whatever the case of its name.
satisfies: REQ-cj5jmx

Normal: `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`
(csp_policy).
Abnormal: `repeated_directive_is_rejected`,
`directive_names_are_matched_case_insensitively` (csp_policy).

**LLR-uus6ar**: it names no directive other than `default-src`, `script-src`,
`style-src`, `img-src`, `connect-src`, `object-src`, `base-uri`,
`frame-ancestors` and `form-action`.
satisfies: REQ-cj5jmx

Normal: `shipped_csp_meets_every_clause`, `fixture_policy_passes_the_checker`
(csp_policy).
Abnormal: `script_src_elem_is_rejected`, `script_src_attr_is_rejected`,
`worker_src_is_rejected` (csp_policy).

Added by review round 1 (finding-1). Under CSP Level 3, `script-src-elem`,
`script-src-attr` and `worker-src` override `script-src` for the scripts they
govern, so a policy that adds one of them can admit inline script past the
`script-src` check. The checker used to refuse only network sources in other
directives. Adding `script-src-elem 'self' 'unsafe-inline'; script-src-attr
'unsafe-inline'` to the shipped `csp` now reddens
`shipped_csp_meets_every_clause`.

**LLR-tc5aax**: the `devCsp` used by development builds names the same
directives as the shipped `csp`, each with the same sources, except that
`script-src` and `connect-src` each add exactly the development server's
origins, `http://localhost:5173` and `ws://localhost:5173`. It therefore meets
the same clauses and admits no other origin.
satisfies: REQ-cj5jmx

Normal: `dev_csp_adds_only_the_dev_server_origins`,
`dev_csp_differs_from_csp_only_by_the_dev_server_origins`,
`dev_fixture_is_a_pure_addition_to_the_fixture` (csp_policy).
Abnormal: `dev_policy_rejects_an_origin_beyond_the_dev_server`,
`dev_policy_that_differs_beyond_the_dev_origins_is_rejected` (csp_policy).

Restored by review round 1 (finding-2) to REQ-cj5jmx's clause that the
development policy differs from the shipped one only by adding the
development server's origins. It used to read "meets the same clauses", and
its test never compared `devCsp` with `csp`. A `devCsp` with `style-src`
removed, or with `blob:` added to `img-src`, passed. The comparison test now
parses both policies and compares them directive by directive, and each of
the reviewer's `devCsp` probes reddens it.

The bundled app's loading under the shipped policy is not covered by any of
these. It is the owner's smoke test, recorded as a verification limitation in
the CSP risk draft.

## SDD-6g3wnh — The untested shell

**SDD-6g3wnh**: everything in the unit that no gated test reaches: the process
entry point and the startup wiring that reads the environment, chain-connection
setup, the receiver loop, the commands whose success needs a chain, the
frontend IPC client, and every Svelte component and route. It carries **no
low-level requirements**.
traces: REQ-7g3k9a, REQ-rxc8sp, REQ-bmk2z2, REQ-645jq9, REQ-e4ah9h, REQ-bvx4nh, REQ-6hgm8r, REQ-3hfggn, REQ-jfxah3, REQ-2k7ys4, REQ-dp95pv, REQ-tw4cb5, REQ-a83vqr, REQ-wu6z9p, REQ-rq8g2v, REQ-he8ejb, REQ-vb5pag, REQ-2yh3c8, REQ-akeah6, REQ-k7ay5b, REQ-5q7nyg, REQ-ergn9z, REQ-y53dqn

*Traces amended 2026-10-20 (change worktree-org-io-domain-api): the
peer-address precondition's requirement is dropped (retired 2026-10-20, owner
answer A2).*

*Amended 2026-10-20 (change `worktree-org-io-domain-api`; owner rulings Q2,
A1 and A3 of 2026-10-16; a clarification, the ID is kept):* `traces:` gains
REQ-5q7nyg, REQ-ergn9z and REQ-y53dqn for three call sites no gated test
reaches: `init`'s read of `ODS_VERIFICATION_INTERVAL_SECS` and its
propagation of the refusal as the startup failure; `run`'s setup spawning
the periodic verification loop (SDD-tgu7dp) with the interval `init`
decided; and the `Admit.svelte` markup that renders the kept admission
requests beside those announced on arrival. The list is in
`2026-10-16-domain-api.md`.

*Amended 2026-10-15 (change `worktree-org-io-domain-api`; a clarification,
the ID is kept):* `traces:` drops the Invite's and the Invite reply's requirements, the Invite reply's
warning and the submission decision, all out of force, and gains their successors as this unit's
call sites realise them — REQ-vb5pag, REQ-2yh3c8, REQ-akeah6 and
REQ-k7ay5b. The commands whose success needs a chain or a peer are now the
renamed surface of SDD-6pjvhc; the receiver loop now also calls
`verify_and_send`; the panels are those listed in
`2026-10-16-domain-api.md`.

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).* The chain
connection leaves this unit for org-io's chain-connection item
(`crates/org-io/docs/architecture/2026-10-08-org-io.md`, the item
moved there from org-node): `build_chain_ops`, `connect_chain`,
`ChainNotConfigured` and `OnChainWriter` are gone from the app, and their
work is `OrgIo::connect`, `OrgIo::not_configured` and org-io's writer. What
stays here of "chain-connection setup" is `init`'s read of `ODS_CHAIN_WS`,
`ODS_CONTRACT_H160` and `ODS_COSIGNER_PUB` and its parse of the contract
address (`chain_settings`, PR-5mc4d8), the `block_on` of `OrgIo::connect`,
the endpoint it records only when that succeeded, and the fallback to
`OrgIo::not_configured`. The signing seed is no longer read here: org-io
reads it, in development builds only. The candidates 12–16 below are
narrowed accordingly: the admin seed and co-signer parse moved to org-io,
where they are tested, and "that stub's three refusals" are org-io's.
`traces:` swaps the submission requirement, now org-io's, for the app's
submission decision (since replaced by REQ-vb5pag), which `create_organisation`, `admit_member` and `revoke_member`
realise at their call sites. Classification under the hybrid rule: a
narrowing of the item's code, not a change of meaning.

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`).* The code
list below changes with the invitation exchange and the chain write moving
into this unit. Chain-connection setup gains the production chain writer —
`OnChainWriter` in `apps/desktop/src-tauri/src/submit.rs`, over on-chain-client's
`write` feature, holding the signatory key `build_chain_ops` parses — and
`ChainNotConfigured` keeps only its read refusal. The commands whose success
needs a chain are now `create_organisation`, the success paths of
`export_invite`, `produce_invite_reply`, `import_invite_reply` and
`admit_member`, `revoke_member`'s call into the service, `list_orgs` and
`OrgDto`; `export_join_request` and `import_join_request` are gone. The
onboarding panels `Invite.svelte` and `Admit.svelte` render the reply warning
and the admission from a reply. So the item traces the four new requirements
those call sites realise.

Its code:

- **Process entry and startup wiring** (survey item B, in part):
  `apps/desktop/src-tauri/src/lib.rs` `run`, `apps/desktop/src-tauri/src/main.rs`, and in
  `apps/desktop/src-tauri/src/state.rs` `AppState::init` and `transport_mode_for`.
- **Chain-connection setup** (survey item C, whole): in `state.rs`,
  `build_chain_ops`, `connect_chain` and `ChainNotConfigured`. *Since
  2026-10-08 (T9 above):* `chain_settings` and `init`'s call of
  `OrgIo::connect`; the rest is org-io's.
- **The receiver loop** (survey item G, in part): in
  `apps/desktop/src-tauri/src/commands.rs`, `start_receiver` and `next_outcomes`.
- **Commands whose success needs a chain**: in `commands.rs`,
  `create_organisation`, `import_invite`, `export_join_request` (exercised
  only as a test fixture), the success path of `export_invite`, the call into
  the service in `admit_member` and `revoke_member`, `list_orgs` and `OrgDto`.
- **The frontend IPC client** (survey item I): `apps/desktop/src/lib/api.ts`.
- **The operator panels** (survey items L and M):
  `apps/desktop/src/lib/components/Membership.svelte`, `Revoke.svelte`,
  `StatusBar.svelte`, `CreateOrg.svelte`, `Invite.svelte`, `Admit.svelte`,
  `PersonaList.svelte`, and `apps/desktop/src/routes/+page.svelte`, `+layout.svelte`
  and `+layout.ts`.

## The item with no low-level requirements

SDD-6g3wnh carries none, under a class that requires them. This section is the
deviation, recorded here rather than left for a reader to notice.

**The deviation.** Class C requires low-level requirements for every item.
SDD-6g3wnh has none. It holds the behaviours that realise, at their call
sites, every requirement in its `traces:` line: `init` applies the startup
policy, the loop emits and stops, and the panels render, subscribe and
validate.

**The reason.** No gated test reaches this code, so a low-level requirement
written for it could not redden when its behaviour broke. The owner's ruling of
2026-10-05 (a low-level requirement only where a gated test reddens) forbids
writing one on those terms. The reach of each part:

- `run` panics on a failed `init`, and the gate never builds a real Tauri
  application. `init`, `build_chain_ops` and `connect_chain` read the process
  environment, and a test that set it would race every other test in its
  binary. `connect_chain` also needs a live chain.
- `transport_mode_for` is reached through `AppState::for_test`, but
  `OrgService` exposes no getter for the mode it was given, so no test can
  observe the mapping. Adding one would be a production edit, which this
  change does not make.
- `start_receiver` spawns a loop that calls
  `receive_and_self_delete_if_revoked`. On the harness's fresh store that
  fails with a message that is not terminal, so the loop never stops. The
  claim, the guard's move into the task, the record re-read in
  `next_outcomes`, the emit-then-break order and the lock's release between
  iterations are therefore all unobserved. The 2026-09-14 risk analysis (§3,
  claims 8, 12 and 13) records the same region as unreached.
- `create_organisation` succeeds only against a chain. `import_invite` needs
  an invite, which needs an Organisation. `list_orgs` and `OrgDto` need a
  stored Organisation. The harness installs `ChainNotConfigured`, so it can
  create none.
- `api.ts` and the components have no test harness. `svelte-check`
  type-checks them, and vitest imports only `verify.ts`, `receiver.ts` and
  `revoke.ts`.

**What is reachable and still left here.** `ChainNotConfigured`'s
`submit_genesis` refusal is reachable today: `create_organisation` on the IPC
harness, for a persona that exists, reaches it. The plan places
`ChainNotConfigured` in this item, so it carries no low-level requirement
here. Its `submit_update` and `read_state` refusals are not reachable, because
both need a stored Organisation first.

**The candidates that moved here.** These are the behaviours the survey
proposed as low-level requirements, which have no gated test. Numbers are the
candidate numbers of the 2026-10-05 decomposition survey (measured at
`06c357b`).

- 7, 10, 11: `init` reads the four `ODS_*` variables, propagates the startup
  policy's refusal, sets the service's transport mode from the recorded one,
  and shows a startup failure to the operator. It currently panics:
  PR-w5dae4.
- 12–16: `build_chain_ops`'s refusals and its parsing of the contract, admin
  seed and co-signer key (PR-5mc4d8), the endpoint it returns beside the ops,
  the fallback to `ChainNotConfigured`, and that stub's three refusals.
- 21: that the status projection performs no environment read. This holds by
  construction (LLR-kze6ak's function takes every input as a parameter), and
  no test can show an absence.
- 26: `parse_org_id`'s whitespace policy, open in PR-5mc4d8.
- 36–38: `OrgDto`'s fields, and the success results of `create_organisation`,
  `import_invite` and `export_invite`.
- 43–48: the receiver loop's runtime: no second spawn while the slot is held,
  the guard's lifetime, the record re-read, the single `SelfDeleted` outcome,
  the emit-then-break order, and the lock's release between iterations.
- 59–60: each `api.ts` wrapper's command name and argument names, and the
  empty `peerAddrBlob` passed through unchanged.
- 72–77: Membership routes every announcement through `applyReceiverEvent`.
  Teardown cancels late registrations. `receiver-stopped` clears the badge,
  and `record-unreadable` renders. Revoke validates against the
  backend-reported mode. StatusBar shows the endpoint only when the chain is
  configured.
- 78–82: the onboarding panels' forwarding, and Admit's truncated key display.

**Booking.** Owner ruling 1 books this item for a later change. Each part
becomes reachable when a change gives it a harness, and that change writes its
low-level requirements:

- the startup wiring and `transport_mode_for`, with the fix for PR-w5dae4;
- the chain setup, with PR-5mc4d8's parser fix and a chain fixture;
- the receiver loop, with a seam that lets a test drive one iteration;
- `api.ts` and the components, with a component-test harness;
- the onboarding panels and commands, with the Gap-20 first-admission app
  change, which mints their requirements (owner ruling 3).
