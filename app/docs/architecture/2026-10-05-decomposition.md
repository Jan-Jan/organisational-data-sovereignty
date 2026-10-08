# app — software decomposition, items and low-level requirements

The first architecture ledger for the `app` unit: eleven software items and the
low-level requirements that refine the unit's high-level requirements in
`app/docs/requirements/`. The SOUP inventory is
`app/docs/architecture/soup.md`.

Safety class C throughout (`app/.guardrails/config.yaml`, no per-item
override), so every item carries low-level requirements, **except SDD-6g3wnh,
which carries none.** That is a deviation, not an override. Its section, "The
item with no low-level requirements", records it in full.

Written under owner ruling 1 of `docs/plans/2026-10-05-app-architecture.md`.
A low-level requirement is written here only where a gated test reddens when
its behaviour breaks. A behaviour that no gated test reaches is listed in the
deviation item instead. New tests were written only where the existing harness
already reaches the code: the command handlers driven over Tauri's mock IPC on
`AppState::for_test`, the `events.rs` vocabulary, the parsing paths, and the
extracted frontend decisions under vitest.

**How each requirement is verified.** Under each low-level requirement,
"Normal" and "Abnormal" name the gated tests that are its normal-case and
abnormal-input cases. A Rust test is named by function, with its file
(`app/src-tauri/tests/<file>.rs`). A vitest test is named by its `it(...)`
title, with its file (`app/tests/<file>.test.ts`). Each test carries
`verifies:` with the requirement it is listed under.

Of the unit's twenty-three high-level requirements, twenty-two are traced by
an item here. REQ-x3c8n2 is not: it is an expectation on `org-node`, not a
behaviour of this unit.

## Overview

The unit's overview lives in the Overview section of
`app/docs/architecture/README.md`, the standing description every later change
amends.

## SDD-k95rmp — Startup policy

`app/src-tauri/src/policy.rs`: `StartupError` and its `Display`,
`DEV_PASSPHRASE`, `DEV_DATA_DIR`, `resolve_passphrase`, `resolve_data_dir`,
`transport_mode_from` and `TransportModeName`.

**SDD-k95rmp**: the startup decisions, made from values handed in: the
passphrase the persona store is opened under, the data directory it lives in,
the transport mode, and the refusal, with its message, when a required value
is missing. Every function is total and none reads the process environment.
traces: REQ-7g3k9a, REQ-rxc8sp, REQ-bmk2z2, REQ-645jq9

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

**LLR-j5pacp**: `transport_mode_from` returns Loopback for exactly the string
`loopback`, and Networked for every other value, including an absent one,
an empty one and `LOOPBACK`.
satisfies: derived

Normal: `loopback_transport_is_reported`, `networked_transport_is_reported`
(connection_status).
Abnormal: `unknown_transport_value_is_networked` (connection_status).

## SDD-aq7m6b — Application state assembly

`app/src-tauri/src/state.rs`: `AppState`, `AppState::assemble`, and
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

**LLR-85zque**: `assemble` creates the data directory, with any missing
parents, and opens the persona store at `persona_store.bin` inside it, under
the given passphrase. When the directory cannot be created, it refuses with a
message beginning `create data_dir <path>`. When an existing store does not
open under the passphrase, it refuses with a message beginning `open store: `,
and opens no other store in its place.
satisfies: derived

Normal: `the_store_is_opened_inside_the_data_dir_it_creates` (state_assembly).
Abnormal: `a_data_dir_that_cannot_be_created_is_refused_naming_it`,
`a_store_under_another_passphrase_is_refused` (state_assembly),
`connection_status_reports_a_data_dir_it_had_to_create_verbatim` (ipc).

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).* What this
states of `assemble` is done since T9 by `open_service`, which `init` and
`for_test` call before `assemble`, with the same messages; the tests are
unchanged. When `init`'s connect to the chain fails, the handle it was given
the service for is dropped and `open_service` opens the same store again for
the unconfigured handle — the same file under the same passphrase, never
another store. Classification: a clarification (the function's name), not a
change of meaning.

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9b).* The note
above said that after a failed connect "`open_service` opens the same store
again for the unconfigured handle". It no longer does: `OrgIo::connect`
hands the service back with its refusal (org-io's development-seed
requirement, which is not exported), the
unconfigured handle is built from it, and `init` opens the store once.
Classification: a narrowing (one open instead of two of the same store),
not a change of meaning. Test: `init_opens_the_store_once_even_when_the_connect_is_refused`
(state_assembly, a scan of `init`'s source, since `init` reads the
environment).

*Amended 2026-10-08 (change `worktree-org-io-create`, review round 2,
finding 2).* `open_service` is gone: the app no longer builds org-node's
service. `init` and `for_test` call `OrgIo::open`, which creates the data
directory, opens `persona_store.bin` in it under the passphrase with the
same two refusal messages, and returns the unconfigured handle; `init`
connects that handle (`connect` takes it), and a refused connect gives it
back through `ConnectFailed::into_not_configured`, so the store is still
opened once. The app's sources construct no `OrgService`. Classification:
a clarification (where the open is done), not a change of meaning; the
tests are unchanged except that the source scan also counts `OrgIo::open(`.

*Clarified 2026-10-08 (change `worktree-org-io-create`, review round 3,
finding 3).* The item's text still names `assemble`. Read it as "the
app's state assembly (`init`, `for_test`), through org-io's `OrgIo::open`".
The open, the two refusals and "opens no other store in its place" are
`OrgIo::open`'s, stated where that code is (org-io's development-seed item,
clause of 2026-10-08) and tested there too. This item keeps the app's side:
the state is assembled over the directory and the store `OrgIo::open`
opened, and its refusals reach the app unchanged. Classification under the
hybrid rule: a clarification (the function's name and where the behaviour
is stated), not a change of meaning; the ID is kept and its tests are
unchanged.

**LLR-vqkr5t**: `AppState.data_dir` is the directory the store was opened in,
and the `connection_status` command reports exactly that directory.
satisfies: REQ-bvx4nh

Normal: `connection_status_reports_the_directory_the_store_was_opened_in` (ipc).
Abnormal: `connection_status_reports_a_data_dir_it_had_to_create_verbatim` (ipc).

## SDD-q4dpym — Connection-status reporting

`app/src-tauri/src/policy.rs`: `ChainEndpoint`, `ConnectionStatus`,
`connection_status_from_state`. `app/src-tauri/src/commands.rs`:
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

`app/src-tauri/src/parsing.rs`: `parse_org_id`. In
`app/src-tauri/src/commands.rs`: the argument parsing in `create_persona`,
`export_invite`, `admit_member` and `revoke_member`, up to the call into
`OrgService`.

**SDD-2pa6h6**: refusal of malformed operator-supplied identifiers and values
at the IPC edge, before they address a service call.
traces: REQ-sjkp8z, REQ-vgr7s2, REQ-yazum3

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`).* The
invitation exchange left org-node for this unit, so the Invite and the Invite
reply are parsed here, at this unit's edge: `Invite::parse` and
`InviteReply::parse` in `app/src-tauri/src/invitation.rs` (LLR-b7wgpf, in
`2026-10-06-invitation.md`), and the item traces
REQ-yazum3. `admit_member` takes a reply Blob and a peer address in place of
the join-request Blob (LLR-ctrfz4, amended below).

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

**LLR-vzf8j2**: `export_invite`, `admit_member` and `revoke_member` each parse
the organisation identifier with `parse_org_id` before they call the service.
On a refusal each returns the parser's message, and none refuses a well-formed
identifier.
satisfies: REQ-sjkp8z

Normal: `export_invite_does_not_refuse_a_well_formed_org_id`,
`admit_member_does_not_refuse_a_well_formed_org_id`,
`revoke_member_does_not_refuse_a_well_formed_org_id` (ipc).
Abnormal: `export_invite_rejects_a_short_org_id`,
`export_invite_rejects_a_non_hex_org_id`,
`admit_member_refuses_a_malformed_org_id_with_the_parsers_message`,
`revoke_member_refuses_a_malformed_org_id_with_the_parsers_message` (ipc).

The sweep narrowed this requirement to `export_invite`: no gated test handed
`admit_member` or `revoke_member` a malformed identifier, so substituting a
fixed identifier for a refusal reddened nothing in either (runs T5-A85,
T5-A86). Review round 1 (finding-3) restored the two commands. Their abnormal
tests require the parser's exact message for an under-width, an over-width and
a non-hexadecimal identifier. The reviewer's substitution probe on each
command now reddens that command's test. Review round 2 (finding-1) did the
same for `export_invite`: its two abnormal tests checked only that the message
contained `40 hex chars` or `hex`, so a fixed message in their place left them
green. They now require the parser's exact message too.

*Note 2026-10-06 (change `worktree-org-node-chain-authority`):* the three
commands keep this requirement with new arguments — `export_invite` takes the
Organisation name and the invitee's name, `admit_member` an Invite reply Blob
and a peer address — and on `AppState::for_test` they now fail after the
parse at the first step that needs a stored Organisation or an outstanding
invite, not at `find_org`; the abnormal tests are unchanged in what they
assert.

Narrowed by the deslop review. The text read "on success it hands the
parsed identifier on". On `AppState::for_test`, all three commands fail at
`find_org` with `OrgNotOnChain` whatever identifier they hand on, so no gated
test observes which identifier reaches the service. T3's M6 (the identifier
sliced) reddened because the slice made the identifier short and the parser
refused it. That shows the refusal clause, not the handing on.

**LLR-6pmrma**: `revoke_member` strips at most one leading `0x` from the member
identifier. It refuses a remainder that is not hexadecimal with a message
beginning `member_id hex`, and one that does not decode to exactly
thirty-two bytes with `member_id must be 32 bytes (64 hex chars)`.
satisfies: derived

Normal: `revoke_member_accepts_a_zero_x_prefixed_member_id`,
`revoke_member_is_not_refused_for_an_empty_peer_addr` (ipc).
Abnormal: `revoke_member_rejects_a_short_member_id`,
`revoke_member_rejects_a_doubled_zero_x_prefix_on_the_member_id` (ipc).

Since review round 1 (finding-6), `revoke_member_rejects_a_short_member_id`
requires the width message exactly. It used to accept either `32 bytes` or
`64 hex`, so rewording the message to `member_id must be 64 hex chars` left
it green.

**LLR-ty85xv**: `revoke_member` treats a peer address that is empty or only
whitespace as absent. It does not decode it and does not refuse the request.
satisfies: REQ-vgr7s2

Normal: `revoke_member_is_not_refused_for_an_empty_peer_addr` (ipc).
Abnormal: `revoke_member_treats_a_whitespace_only_peer_addr_as_absent` (ipc).

Cut by the sweep: "it hands the service no address". Every gated call fails in
the service before the address is used, so handing it a fixed address
reddened nothing (run T5-A92).

**LLR-n6twt7**: `revoke_member` decodes a non-blank peer address from
hexadecimal and then as a postcard-encoded iroh `EndpointAddr`, and does not
refuse one that decodes. Text that is not hexadecimal is refused with a
message beginning `peer_addr_blob hex:`. Bytes that are not an `EndpointAddr`
are refused with a message beginning `peer_addr decode:`; they are not
dropped.
satisfies: derived

Normal: `revoke_member_does_not_refuse_an_encoded_endpoint_addr` (ipc).
Abnormal: `revoke_member_refuses_a_peer_addr_that_is_not_hex`,
`revoke_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr` (ipc).

Cut by the sweep: that the decoded address is the one handed to the service.
Decoding it and then handing none reddened nothing (run T5-A95), for the
reason given under LLR-ty85xv.

**LLR-8krgzj**: `admit_member` takes no Organisation secret or key: its
arguments are `org_id`, `reply_blob` and `peer_addr_blob`, it parses no
secret, and nothing it hands org-node carries an Organisation secret, an
Organisation key or an invite identifier for the Wire message. The webview's
`admitMember(orgId, replyBlob, peerAddrBlob)` (`app/src/lib/api.ts`) invokes
it with those three values alone, and `Admit.svelte` passes no fourth.
satisfies: derived

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This said `admit_member` accepts an optional `org_secret_hex`, refused unless
it decodes from hexadecimal to thirty-two bytes (`org_secret hex:`,
`org_secret must be 32 bytes`). The Organisation secret is removed from
org-node; the Organisation private key an admission's Wire message carries is
the one org-node's record holds, and `send_update` takes none from its caller
(org-node's requirements, change `worktree-org-node-org-key-pair`). The parameter, its parse and its two refusals go, and
the item states their absence. Its tests are rewritten accordingly; the
`0x` note below no longer applies to this command.

Normal and abnormal: `admit_member_takes_no_organisation_secret` (ipc,
a stale `orgSecretHex` of any value is not read);
`api.admit.test.ts` (the webview passes three values).

The `0x` handling is deliberately left out of this requirement: the parse
strips a run of prefixes, not one, and that is PR-5mc4d8's.

Cut by the sweep: which secret reaches the service. Every gated call fails in
the service before the secret is used. Handing a fixed secret when none was
supplied, or in place of the decoded one, reddened nothing (runs T5-A96,
T5-A100).

**LLR-ctrfz4**: `admit_member` treats a peer address that is empty or only
whitespace as absent, and does not decode it or refuse the request. It decodes
a non-blank one from hexadecimal and then as a postcard-encoded
`EndpointAddr`, and does not refuse one that decodes. Text that is not
hexadecimal is refused with a message beginning `peer_addr_blob hex:`, and
bytes that are not an `EndpointAddr` with one beginning `peer_addr decode:`.
satisfies: derived

Normal: `admit_member_takes_no_organisation_secret`,
`admit_member_does_not_refuse_a_blank_or_decodable_peer_addr` (ipc).
Abnormal: `admit_member_refuses_a_peer_addr_that_is_not_hex`,
`admit_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr` (ipc).

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`).* This
stated how `admit_member` read the node address a join request carried. The
join request left org-node, and the Invite reply carries no address
(REQ-tcutr6), so the caller passes the joiner's address as `revoke_member`'s
caller does (LLR-ty85xv, LLR-n6twt7), needed in Loopback transport only.

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The requirement's text is unchanged. Its first Normal test,
`admit_member_does_not_refuse_a_32_byte_or_absent_org_secret`, also exercised
the Organisation secret `admit_member` no longer takes (LLR-8krgzj); it is
rewritten without that argument, and the address clauses it carries stay.

Cut by the sweep: which address reaches the service. That the id-only address
is built from the device key, and that a decoded address is the one handed
on, both reddened nothing: the call fails in the service before the address
is used (runs T5-A102, T5-A104).

The refusal of a device key that is not a valid `EndpointId` is not part of
this requirement. A join request that decodes already carries a parsed device
key, so no gated test can reach that refusal.

**LLR-p7dfxb**: `create_persona` parses the handle, name and surname with
`org_node::store::PersonaDetails::parse` before it calls the service, and
stores the parsed (NFC) form. A refusal names the field it is about (`handle:`,
`name:`, `surname:`) without org-node's `persona.` prefix, and creates nothing.
satisfies: derived

Normal: `create_persona_stores_the_parsed_details` (ipc).
Abnormal: `create_persona_refuses_an_invalid_field_naming_it_and_creates_nothing`
(ipc).

## SDD-rmbr3t — The command surface

`app/src-tauri/src/commands.rs`: `PersonaDto`, `JoinRequestDto`,
`import_join_request`, `list_personas`, and the handler list the IPC suite
registers. `app/src-tauri/src/lib.rs` registers its own copy of that list in
`run`. The rest of `run` is SDD-6g3wnh's, but the list's contents are gated
here (LLR-4wcyqy).

**SDD-rmbr3t**: the twelve IPC commands as a surface. Each one locks the
service, calls `OrgService`, maps its error to a string and returns a DTO
that carries no secret key material. The commands named in SDD-6g3wnh carry
no low-level requirement.
traces: REQ-vgr7s2, REQ-sjkp8z, REQ-645jq9, REQ-prjja8, REQ-tcutr6, REQ-65xqp8, REQ-ab2mfz, REQ-m8sgjk

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`).* The
invitation exchange and the chain write moved into this unit, behind the
commands. The item's code gains `app/src-tauri/src/invitation.rs`
(`issue_invite`, `OutstandingInvites`, `produce_reply`, `check_reply`,
`admit_reply`) and `app/src-tauri/src/submit.rs` (`ChainWriter`,
`WriterNotConfigured`, `found_organisation`, `submit_commit_send`), and loses
`JoinRequestDto` and `import_join_request`, whose place `InviteReplyDto` and
`import_invite_reply` take. The surface is still twelve commands:
`export_join_request` and `import_join_request` leave, `produce_invite_reply`
and `import_invite_reply` arrive. Its low-level requirements for the new
behaviour (LLR-9sraks, LLR-f35pda, LLR-w4mhd4, LLR-gha5f6, and the two
low-level requirements of the chain write's order and bound) are in `2026-10-06-invitation.md`.

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).*
`app/src-tauri/src/submit.rs` leaves the item: it is deleted, and the chain
write, then org-node's commit and send, are org-io's (`org-io/src/submit.rs`).
Each command locks the org-io handle (`AppState.org_io`) and calls org-node's
queries and builders through it (`OrgIo::node`, `OrgIo::node_mut`) and the
submission through it (`OrgIo::found_organisation`,
`OrgIo::submit_commit_send`); `revoke_and_send` and `admit_reply` take the
handle. The two low-level requirements of the chain write's order and its
90-second bound moved with the code to org-io's architecture ledger
(`org-io/docs/architecture/2026-10-08-org-io.md`), which
does not export them; the sentence above that named them is rewritten to
prose. `traces:` swaps the submission requirement, now org-io's, for the
app's REQ-m8sgjk (the decision when to submit and to which Device). The
surface is still twelve commands. Classification under the hybrid rule: a
clarification, not a change of meaning.

**LLR-4wcyqy**: the IPC suite's handler list registers the twelve commands
under their snake_case names, and an invocation of any other name is refused
rather than answered. The `generate_handler!` list in
`app/src-tauri/src/lib.rs` names the same set of commands as the suite's list,
and `lib.rs` has exactly one `generate_handler!` list in exactly one
`invoke_handler` call.
satisfies: derived

Normal: `list_personas_returns_an_empty_list_on_a_fresh_store`,
`connection_status_is_registered_under_its_name`,
`product_registers_the_same_commands_as_the_harness` (ipc).
Abnormal: `unknown_command_is_rejected` (ipc).

Restated by review round 1 (finding-4). The first two clauses are properties
of the suite's own registration: the tests invoke the suite's copy of the
list, not `lib.rs`'s, so deleting a command from `lib.rs` left every test
green. The third clause gates the product: the test reads both files as text
and compares the two lists as sets of names. Deleting
`commands::revoke_member` from `lib.rs` now reddens it.

Extended by review round 2 (finding-2). Tauri's `Builder::invoke_handler`
replaces any earlier handler, so the last call wins, and the test read only the
first list: a second `invoke_handler` registering one command left it green.
The test now also counts `generate_handler!` and `.invoke_handler(` in
`lib.rs`, outside `//` comments, and requires one of each. It does not parse
block comments or string literals.

**LLR-pguhw5**: each persona `list_personas` returns has exactly the fields
`persona_id`, `org_id`, `handle`, `name`, `surname` and `status`. No seed or
other secret crosses IPC. `org_id` is present and null for a persona that
belongs to no Organisation.
satisfies: derived

Normal: `list_personas_reports_exactly_the_persona_fields` (ipc).
Abnormal: `a_persona_in_no_organisation_reports_a_null_org_id` (ipc).

**LLR-pmus9f**: `import_invite_reply` parses an Invite reply Blob (LLR-b7wgpf)
and stores nothing. It returns the Organisation identifier, the handle, name
and surname, and the member and device keys as hexadecimal. A Blob that does
not parse is refused with the parse's message, which names the field, and a
reply whose invite identifier this device does not hold as outstanding for the
Organisation the reply names (LLR-gha5f6's `check_reply`) is refused with a
message saying so.
satisfies: derived

Normal: `import_invite_reply_reports_the_reply_it_parses` (ipc).
Abnormal: `import_invite_reply_refuses_a_malformed_blob_or_an_unknown_invite`
(ipc).

*Amended 2026-10-06 (independent review round 2, finding-1).* "Outstanding"
now means outstanding for the reply's Organisation: the abnormal test also
presents the outstanding invite identifier under another Organisation.

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`).* This
described `import_join_request`, which decoded org-node's join-request blob.
The join request left org-node; the Invite reply the app parses takes its
place in the onboarding flow, and the command that shows it to the inviter
before admission is `import_invite_reply`. The notes below describe the
earlier command's tests and are kept as its history.

Since review round 1 (finding-5), the abnormal test requires the message
that `OrgService::import_join_request` itself returns for each input. It used
to check only that the message contained `blob`, so replacing org-node's
message with `invalid blob` left it green.

Since review round 2 (finding-3), it checks both. The equality moves with
org-node's wording, so it cannot hold the clause "which names the blob". A
`contains("blob")` assertion beside it does: rewording org-node's decode
errors in `org-node/src/blobs.rs` so they no longer name the blob reddens it.

## SDD-5fchuy — Receiver start guard

`app/src-tauri/src/events.rs`: `StartGuard` and its `Drop`.

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

`app/src-tauri/src/events.rs`: `ReceiverOutcome`, `classify_receive_error`,
`TERMINAL_ERRORS`, `is_terminal_error`, `outcomes_for_receive_error`,
`Emission`, the payload structs and `emissions_for`.

**SDD-mwqf6x**: the total mapping from a receive-path error to an outcome class
(a verification verdict or a receiver error), the detection of a terminal
error, and the mapping from an outcome to the named events and payloads the
frontend receives. It is the only place the event names and payload shapes
are written down.
traces: REQ-kn5rtx, REQ-affyf5, REQ-jfxah3, REQ-2k7ys4, REQ-dp95pv, REQ-tw4cb5

It depends on REQ-x3c8n2, the expectation that org-node will type the
terminal condition. Until then, `is_terminal_error` matches substrings of
messages org-node formats.

**LLR-7bk6qh**: `classify_receive_error` classifies the seven verdict variants
(`OrgIdMismatch`, `StaleSeq`, `MalformedDelta`, `DeltaBaseMismatch`,
`RootMismatch`, `StaleEpoch`, `SeqNotEpoch`) as `VerifyFailed`, carrying
the error's message and no organisation. It classifies every other variant
(`Chain`, `OrgNotOnChain`, `Trie`, `InvalidOrgPublicKey`, `InvalidField`,
`AdmissionNotExpected`, `AdmissionNotOurs`, `ProvisionalLimit`,
`NoProvisionalUpdate`, `PersonaAlreadyBound`, `MalformedMessage`,
`OrgKeyMismatch`, `RevocationNotHeld`, `RevocationNotForThisDevice`) as
`ReceiveError`, carrying the error's message.
satisfies: REQ-kn5rtx

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3, task
T12a; owner rulings at the S3a close-out residual review).* org-node gains
two more refusals, each classified as `ReceiveError`: a message for a held
Organisation, or a revocation, from a Device the record does not list
(`SenderNotListed`), and an acknowledgement not sent by the Device it names
(`AcknowledgementNotFromItsDevice`). Both
are refused before anything is verified — before the chain is read — so
neither is a verdict on an update. The abnormal test
`the_sender_refusals_are_classified_as_receiver_errors` lists the two.

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3).*
org-node gains ten refusals (`OrgNotHeld`, `StaleChainState`,
`ChainStateConflict`, `RevocationProofRefused`, the four
`Acknowledgement*`, `DeviceSecretNotSupplied`, `NoRevocationForRecipient`),
each classified as `ReceiveError`. None is produced by
`verify_envelope_against_chain`, the rule for a verdict, and none is a
verdict on an update: a `VerifyFailed` row is shown under "Verified Updates
(chain root match)", and filing a refused revocation proof or acknowledgement
there would claim an update failed a root match when no update was received
— HAZ-9fmhm4's misrepresentation — while filing it as a receiver error is
only less specific. `RevocationProofRefused` is the closest case, since it
checks a proof against the chain's root; it stays a receiver error because
what it refuses is a notice about this node, not a membership update, and
`StaleChainState` is reachable from the node's own record alone. The
abnormal test `the_commit_workflow_refusals_are_classified_as_receiver_errors`
lists the ten.

*Amended 2026-10-06 (change `worktree-org-node-org-key-pair`).* org-node
gains four refusals of a received Wire message: one that does not decode
(`MalformedMessage`), a revocation about an Organisation the node holds no
record of (`RevocationNotHeld`), a carried Organisation private key whose
public half is not the chain's (`OrgKeyMismatch`), and a revocation after
which this node's Device is still listed (`RevocationNotForThisDevice`). None
is produced by `verify_envelope_against_chain`, the rule the note below
states for a verdict: the first two are refused before anything is verified,
the last two after it on a rule about the key or about this node, as
`AdmissionNotOurs` is. Each is a receiver error. `PersonaAlreadyBound`, which
the receiver list already matched, is now named. The abnormal test
`locally_reachable_variants_are_classified_as_receiver_errors` lists the four.

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`).* org-node
gains four refusals: a first admission no expectation matches, a first
admission that lists none of the node's Personas, the bound on provisional
updates, and a commit no provisional update matches. None is a verdict on a
delivered change verified against the chain — the first is refused before
anything is verified, the second after verification but on a rule about this
node, and the other two never arise on receive — so each is a receiver error.
The classification rule and the tests above are unchanged; the abnormal test
`locally_reachable_variants_are_classified_as_receiver_errors` lists the four.

*Amended 2026-10-06 (merge of master `d8b9f9b` into
`worktree-worktree-person-shared-types`):* this item was written against
org-node's error set before that change. org-node no longer has
`BadSignature` (the Envelope carries no signature) or `InvalidKey` (an
Organisation public key that fails its X25519 parse is `InvalidOrgPublicKey`),
and gained `SeqNotEpoch` (a Sequence number other than the chain's epoch, a
refusal of the delivered update). The variant lists now name the set
`classify_receive_error` matches; the classification rule and the tests below
are unchanged.

Normal: `every_verification_verdict_is_classified_as_a_verification_failure`,
`a_verification_verdict_carries_its_own_message_and_no_invented_organisation`,
`the_two_classes_are_actually_distinguished` (receiver_events).
Abnormal: `a_chain_failure_is_classified_as_a_receiver_error`,
`a_chain_failure_emits_no_verification_event_for_any_message`,
`locally_reachable_variants_are_classified_as_receiver_errors`,
`a_refused_key_or_field_is_classified_as_a_receiver_error` (receiver_events).

**LLR-usxk57**: a `ReceiveError` outcome is announced as exactly one
`receiver-error` event, whose payload carries the message and no
organisation, epoch, root or verdict, at any depth.
satisfies: REQ-kn5rtx

Normal: `the_receiver_error_payload_carries_a_message_and_no_verification_state`
(receiver_events).
Abnormal: `a_chain_failure_emits_no_verification_event_for_any_message`
(receiver_events).

**LLR-a9rjtf**: `outcomes_for_receive_error` returns the error's class outcome.
When the rendered message contains `endpoint bind:`,
`endpoint bind failed unexpectedly` or `endpoint closed`, it follows it with a
`Stopped` outcome whose reason is that message; otherwise it returns nothing
else.
satisfies: REQ-jfxah3

Normal: `every_real_terminal_message_stops_the_loop`,
`the_stop_is_announced_after_the_failure_for_the_same_error` (receiver_events).
Abnormal: `a_non_terminal_failure_does_not_stop_the_loop` (receiver_events).

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

`classify_receive_error` never names an organisation (LLR-7bk6qh), so the
known-organisation shape is reachable only from a constructed outcome. The
2026-09-14 risk analysis records this as REQ-affyf5 being vacuous today.

## SDD-8jw9mn — Verification view model

`app/src/lib/verify.ts`: `verifyResultFrom`, `applyReceiverEvent` and the two
list limits. `app/src/lib/receiver.ts`: `subscribeAll`.

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

## SDD-jx363y — Revocation input decision

`app/src/lib/revoke.ts`: `validateRevokeInput`.

**SDD-jx363y**: the decision, made before any command is invoked, whether a
revocation may be submitted, from the transport mode and the operator's
inputs.
traces: REQ-vgr7s2, REQ-he8ejb, REQ-ab2mfz

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`).* The item
also makes the webview's other pre-command decision this change adds: whether
an Invite reply may be produced, and what the user is told first
(`app/src/lib/invite.ts`: `REPLY_WARNING`, `replyGate`; LLR-n2u4uf, in
`2026-10-06-invitation.md`). It is the same kind
of decision — pure, made from the operator's inputs, tested under vitest — so
it sits here rather than in a new item, and the item traces REQ-ab2mfz. The
backend refuses an unconfirmed reply as well (LLR-w4mhd4), as SDD-2pa6h6
re-checks what this item checks.

**LLR-csbs5v**: `validateRevokeInput` trims the member identifier and strips at
most one leading `0x`. It refuses a remainder that is not exactly 64
characters, naming its length, and one that is not hexadecimal.
satisfies: REQ-he8ejb

Normal: "accepts a 0x-prefixed member id", "trims surrounding whitespace before
measuring" (revoke.validate).
Abnormal: "rejects a 63-character member id", "rejects a 65-character member
id", "rejects an empty member id", "rejects a 64-character non-hexadecimal
member id", "rejects a doubled 0x prefix" (revoke.validate).

**LLR-c9r5uf**: `validateRevokeInput` requires a peer address that is not blank
in Loopback transport, refusing with a message naming Loopback, and accepts
any peer address, including none, in Networked transport.
satisfies: REQ-vgr7s2

Normal: "accepts an empty peer address in networked transport", "accepts a
supplied peer address in networked transport", "accepts a supplied peer
address in loopback transport" (revoke.validate).
Abnormal: "rejects an empty peer address in loopback transport", "trims
surrounding whitespace before measuring" (revoke.validate).

## SDD-32hath — The webview's Content-Security-Policy

`app/src-tauri/tauri.conf.json`: `app.security.csp` and `app.security.devCsp`.

**SDD-32hath**: the policy the webview runs under, which bounds what a script
that reaches the webview can load, run or connect to. It is configuration,
not code: `app/src-tauri/tests/csp_policy.rs` holds the checker, the abnormal
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

Task T2 of `docs/plans/2026-10-05-app-architecture.md` cut the clause that
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
traces: REQ-7g3k9a, REQ-rxc8sp, REQ-bmk2z2, REQ-645jq9, REQ-e4ah9h, REQ-bvx4nh, REQ-6hgm8r, REQ-3hfggn, REQ-jfxah3, REQ-2k7ys4, REQ-dp95pv, REQ-tw4cb5, REQ-a83vqr, REQ-wu6z9p, REQ-rq8g2v, REQ-vgr7s2, REQ-he8ejb, REQ-prjja8, REQ-tcutr6, REQ-ab2mfz, REQ-m8sgjk

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).* The chain
connection leaves this unit for org-io's chain-connection item
(`org-io/docs/architecture/2026-10-08-org-io.md`, the item
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
REQ-m8sgjk, which `create_organisation`, `admit_member` and `revoke_member`
realise at their call sites. Classification under the hybrid rule: a
narrowing of the item's code, not a change of meaning.

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`).* The code
list below changes with the invitation exchange and the chain write moving
into this unit. Chain-connection setup gains the production chain writer —
`OnChainWriter` in `app/src-tauri/src/submit.rs`, over on-chain-client's
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
  `app/src-tauri/src/lib.rs` `run`, `app/src-tauri/src/main.rs`, and in
  `app/src-tauri/src/state.rs` `AppState::init` and `transport_mode_for`.
- **Chain-connection setup** (survey item C, whole): in `state.rs`,
  `build_chain_ops`, `connect_chain` and `ChainNotConfigured`. *Since
  2026-10-08 (T9 above):* `chain_settings` and `init`'s call of
  `OrgIo::connect`; the rest is org-io's.
- **The receiver loop** (survey item G, in part): in
  `app/src-tauri/src/commands.rs`, `start_receiver` and `next_outcomes`.
- **Commands whose success needs a chain**: in `commands.rs`,
  `create_organisation`, `import_invite`, `export_join_request` (exercised
  only as a test fixture), the success path of `export_invite`, the call into
  the service in `admit_member` and `revoke_member`, `list_orgs` and `OrgDto`.
- **The frontend IPC client** (survey item I): `app/src/lib/api.ts`.
- **The operator panels** (survey items L and M):
  `app/src/lib/components/Membership.svelte`, `Revoke.svelte`,
  `StatusBar.svelte`, `CreateOrg.svelte`, `Invite.svelte`, `Admit.svelte`,
  `PersonaList.svelte`, and `app/src/routes/+page.svelte`, `+layout.svelte`
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
written for it could not redden when its behaviour broke. Owner ruling 1 of
`docs/plans/2026-10-05-app-architecture.md` forbids writing one on those
terms. The reach of each part:

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
proposed as low-level requirements, which have no gated test. Numbers refer to
§3 of the decomposition survey listed under "Inputs" in
`docs/plans/2026-10-05-app-architecture.md` (measured at `06c357b`).

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
