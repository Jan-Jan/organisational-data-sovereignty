# app — the invitation exchange and submitting updates: low-level requirements

Low-level requirements for change `worktree-org-node-chain-authority`, which
refine the app requirements of
`app/docs/requirements/2026-10-06-invitation.md`
(REQ-prjja8, REQ-tcutr6, REQ-65xqp8, REQ-ab2mfz, REQ-yazum3, REQ-nfr3n2). Written
2026-10-06, after the merge of master `d8b9f9b` gave this unit its first
architecture ledger (`2026-10-05-decomposition.md`). No software item is
added: each requirement below sits under the item of that file that owns its
code, and that item's `traces:` line is amended in place there with a dated
note.

The conventions are that ledger's. Safety class C, no per-item override. A
low-level requirement is written only where a gated test reddens when its
behaviour breaks; the tests named under each are the ones this change's
implementation plan (`docs/plans/2026-10-05-chain-authority.md`, T14–T16)
writes. "Normal" and "Abnormal" name them by function, with their file
(`app/src-tauri/tests/<file>.rs`), or by `it(...)` title for vitest
(`app/tests/<file>.test.ts`). Each test carries `verifies:` with the
requirement it is listed under.

The code is in two new files of the Rust half, `app/src-tauri/src/invitation.rs`
(the Invite, the Invite reply, the outstanding invite identifiers) and
`app/src-tauri/src/submit.rs` (the chain write, then org-node's commit and
send), and one of the webview, `app/src/lib/invite.ts`. What of them no gated
test reaches — the production chain writer over subxt, the Tauri handlers'
success paths that need a chain, and the Svelte panels — is SDD-6g3wnh's, as
that item's amended code list states.

## Under SDD-2pa6h6 — Command-boundary parsing

**LLR-b7wgpf**: `Invite::parse` and `InviteReply::parse` take a Blob — Base64 of
the postcard encoding — and parse each field with its type: the Organisation
identifier as twenty bytes, each key with `person`'s `DevicePublicKey::parse`
or `PersonPublicKey::parse` (as org-node re-exports them), the invite identifier as thirty-two bytes, and the
handle, name and surname with their `person` types. The first field that fails
is refused with a message that begins with its name (`invite.org_id`,
`invite.inviter_device_keys`, `invite.invite_id`; `reply.org_id`,
`reply.invite_id`, `reply.member_key`, `reply.device_key`, `reply.handle`,
`reply.name`, `reply.surname`), and text that is not a Blob with one that
begins `invite` or `reply`. A Blob that parses encodes back to the same text.
A refused Blob acts on nothing: no expectation is declared, no outstanding
identifier is settled and no service call is made.
satisfies: REQ-yazum3

Normal: `an_invite_and_a_reply_that_parse_encode_back_to_the_same_blob`
(invitation).
Abnormal: `an_invite_or_reply_that_does_not_parse_is_refused_naming_the_field`,
`a_reply_that_does_not_parse_acts_on_nothing` (invitation).

## Under SDD-rmbr3t — The command surface

**LLR-9sraks**: `issue_invite(svc, outstanding, rng, org_id, org_name,
invitee_name)` returns an Invite Blob carrying `org_name` and `invitee_name`
as typed, `org_id`, the DevicePublicKey of every Persona of this device bound
to that Organisation, and an invite identifier of 32 bytes drawn from `rng`,
which it adds to the outstanding identifiers before returning. For an
Organisation no Persona of this device is bound to it refuses, issuing no
Blob and adding no identifier.
satisfies: REQ-prjja8

Normal: `an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id`
(invitation).
Abnormal: `no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to`
(invitation).

**LLR-f35pda**: `OutstandingInvites` keeps each outstanding Invite as the pair
(Organisation identifier, invite identifier) it was issued for, in one file,
`outstanding_invites.json` in the data directory, as a JSON list of objects
`{"org_id": <40 hex>, "invite_id": <64 hex>}`, written before `issue` or
`settle` returns, so that reopening the file yields exactly the pairs issued
and not yet settled. `holds(org_id, invite_id)` is true only for a pair it
keeps — an identifier is not outstanding for any other Organisation — and
`settle(org_id, invite_id)` removes that one pair and leaves the others. A
file that is not such a list is refused when opened, naming the file; that
includes the earlier format, a list of bare invite identifiers, which is not
migrated (a PoC device deletes the file and issues its Invites again).
satisfies: REQ-65xqp8, REQ-prjja8

Normal: `outstanding_invite_ids_survive_a_reopen_and_settle_one_at_a_time`
(invitation).
Abnormal: `an_outstanding_invites_file_that_is_not_a_list_of_pairs_is_refused`,
`a_reply_naming_another_held_organisation_with_an_outstanding_invite_id_is_refused`
(invitation).

*Amended 2026-10-06 (independent review round 2, finding-1).* The file held
bare invite identifiers, so an identifier was outstanding for every
Organisation; it now holds the Organisation each was issued for, as
org-node keys its expected admissions by (Organisation, invite identifier).

**LLR-w4mhd4**: `produce_reply(svc, rng, invite_blob, persona_id, confirmed)`
refuses, producing nothing and declaring nothing, when `confirmed` is false —
with a message that begins `confirm first` — when `persona_id` names no
Persona of this device, and when it names one bound to an Organisation, as
LLR-rt8gdz states (amended 2026-10-06, review round 1). Otherwise it returns an Invite reply Blob carrying that
Persona's Member-as-a-group key, DevicePublicKey, handle, name and surname and
the Invite's Organisation identifier and invite identifier, and before
returning declares to org-node, through `expect_admission(org_id, invite_id)`,
the first admission it expects.
satisfies: REQ-tcutr6, REQ-ab2mfz

Normal: `a_confirmed_reply_carries_the_persona_and_declares_the_expected_admission`
(invitation).
Abnormal: `no_reply_is_produced_and_nothing_declared_without_confirmation`
(invitation); `produce_invite_reply_refuses_without_confirmation_over_ipc` (ipc).

**LLR-rt8gdz**: a Persona bound to an Organisation is not offered, and is
refused, as the Persona to found an Organisation with or to reply to an
Invite as. `produce_reply` refuses a `persona_id` that names a Persona of this
device bound to an Organisation, with org-node's `PersonaAlreadyBound` message
for that Persona, after the confirmation check and before declaring any
expectation, producing no reply. `found_organisation` refuses such a Persona
with the same message, from org-node's `create_organisation`, before calling
the chain writer. The webview offers, as the Personas to reply as, only those
`unboundPersonas` (`app/src/lib/invite.ts`) keeps — the Personas whose
`org_id` is null, in the order given; the founding panel offers no choice, and
founds with the Persona it has just created.
satisfies: derived

Normal: `a_bound_persona_cannot_reply_and_an_unbound_one_on_the_same_device_can`
(invitation); "keeps only the Personas bound to no Organisation, in order"
(invite.personas).
Abnormal: `no_reply_is_produced_and_nothing_declared_for_a_bound_persona`,
`a_bound_persona_founds_nothing_and_the_chain_is_not_written` (invitation);
"offers none when every Persona is bound", "offers none from an empty list"
(invite.personas).

Review round 1, finding-1 (2026-10-06): one Persona, one Organisation (owner
ruling). Derived, because the founding half has no parent among this unit's
requirements; the reply half narrows REQ-tcutr6's "a Persona the user chose".
The `unboundPersonas` filter sits in `invite.ts`, SDD-jx363y's file, beside
`replyGate`; the backend refusal is what holds when the webview is bypassed.

**LLR-gha5f6**: `check_reply(outstanding, reply_blob)` refuses, with `this
reply names no Invite this device has outstanding`, a reply whose pair
(its `org_id`, its invite identifier) is not outstanding (LLR-f35pda).
`admit_reply(svc, writer, outstanding, rng, org_id, reply_blob, peer_addr,
org_secret)` refuses, acting on nothing, a reply `check_reply` refuses, and
one whose outstanding pair's Organisation is not `org_id`, with `this reply is
for another Organisation`. The target Organisation is the one the outstanding
pair names: `org_id` is kept as the operator's selection in the Admit panel,
which preselects it from the reply, and is not trusted — it can only confirm
that Organisation, never choose another. Otherwise `admit_reply` builds
org-node's `Joiner` from the reply's five values, has org-node build the
admission to that Organisation, submits it as LLR-qhjp6g states with the
reply's DevicePublicKey as recipient and its invite identifier in the Wire
message, and settles that pair once the admission has committed — even when
the send that follows fails; when the submission fails the pair stays
outstanding.
satisfies: REQ-65xqp8, REQ-nfr3n2

Normal: `a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding`
(invitation).
Abnormal: the same test's reply to an Invite this device never issued and its
second use; `a_reply_whose_admission_fails_on_chain_stays_outstanding`,
`a_reply_admitted_under_another_selected_organisation_is_refused` (a true
reply, with a second Organisation the inviter does hold selected),
`a_reply_naming_another_held_organisation_with_an_outstanding_invite_id_is_refused`
(the reply itself names that second Organisation, and is admitted as the
shipped panel would, under its own `org_id`),
`a_reply_is_settled_once_committed_even_when_the_send_fails` (invitation).
Review round 1, finding-4 (2026-10-06): the refusal test used to name an
Organisation the inviter holds no record of, so admission failed without the
check; and the "even when the send fails" clause had no test.

*Amended 2026-10-06 (independent review round 2, finding-1).* The
Organisation check compared the reply's `org_id` with the `org_id` the panel
had preselected from that same reply, and the outstanding check looked at the
invite identifier only, so a reply to an Invite for one Organisation that
named another this device holds was admitted there. The check is now on the
pair, and the test `a_reply_for_another_organisation_is_refused` is rewritten
as `a_reply_admitted_under_another_selected_organisation_is_refused`.

**LLR-qhjp6g**: `found_organisation` and `submit_commit_send` call the chain
writer first and ask org-node to commit (`commit_genesis`, `commit_update`)
only after the write has returned success, and `submit_commit_send` asks
org-node to send (`send_update`) only after the commit has succeeded. When the
write fails they ask org-node for neither, keep the provisional update, and
return an error that reports the failure. `submit_commit_send` refuses before
writing when the Organisation's record holds no proxy account.
satisfies: REQ-nfr3n2

Normal: `founding_writes_the_chain_then_commits`,
`an_admission_is_written_then_committed_then_sent` (submit_flow).
Abnormal: `a_failed_genesis_write_commits_nothing_and_reports_the_failure`,
`a_failed_update_write_neither_commits_nor_sends`, and the proxy-account
refusal in `an_admission_is_written_then_committed_then_sent` (submit_flow).

**LLR-be3zv9**: each call `found_organisation` and `submit_commit_send` make
to the chain writer is bounded by a 90-second timeout (`tokio::time::timeout`);
a call that has not finished when it elapses is treated as a failed write, so
org-node is asked neither to commit nor to send, the provisional update is
kept, and the error reports that the submission timed out.
satisfies: REQ-nfr3n2

Normal: `founding_writes_the_chain_then_commits` (submit_flow), whose writer
returns at once.
Abnormal: `a_submission_that_never_finishes_times_out_and_nothing_is_committed`
(submit_flow), with tokio's clock paused.

The writer in on-chain-client has no timer of its own (its normal
dependencies exclude tokio), so its wait for finality can wait without end;
the bound is the app's, at the 90 seconds org-node's settle used before
(plan T14, dispatcher addition of 2026-10-06).

## Under SDD-jx363y — Revocation input decision

**LLR-n2u4uf**: `REPLY_WARNING` states that nothing has verified who sent the
Invite, that nothing has verified the Organisation name it states, and that the
reply reveals the handle, name and surname of the chosen Persona to its
sender; `replyGate` refuses, naming what is missing, until a Persona is chosen
(a blank choice counts as none) and the user has confirmed, and allows the
reply only then.
satisfies: REQ-ab2mfz

Normal: "states each thing the user must be told", "allows the reply once a
Persona is chosen and the user confirmed" (invite.confirm).
Abnormal: "refuses until the user has confirmed", "refuses without a chosen
Persona even when confirmed" (invite.confirm).
