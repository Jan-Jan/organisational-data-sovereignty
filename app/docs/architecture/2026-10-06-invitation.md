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

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9, ruling A).*
The submission requirement of that list moved to org-io, which exports it,
and with it the two low-level requirements that stood under "Under
SDD-rmbr3t" below for the write-then-commit-then-send order and the
90-second bound; they are in org-io's architecture ledger
(`org-io/docs/architecture/2026-10-08-org-io.md`, under
the submission item). The decision that stays in the app is REQ-m8sgjk
(`app/docs/requirements/2026-10-08-submission.md`), which
LLR-gha5f6 below now satisfies in its place.

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
*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).*
`app/src-tauri/src/submit.rs` is deleted: the chain write, then org-node's
commit and send, are org-io's (`org-io/src/submit.rs`), reached through the
org-io handle, and the production chain writer over subxt is org-io's
chain-connection item's. What of this unit no gated test reaches is the
Tauri handlers' success paths that need a chain and the Svelte panels.

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

*Note 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The requirement's text holds. The reason the note above gives does not:
org-node now keys its expected admissions by Organisation alone. The pair is
still what binds a reply to the Invite this device issued (REQ-65xqp8).

**LLR-w4mhd4**: `produce_reply(svc, rng, invite_blob, persona_id, confirmed)`
refuses, producing nothing and declaring nothing, when `confirmed` is false —
with a message that begins `confirm first` — when `persona_id` names no
Persona of this device, and when it names one bound to an Organisation, as
LLR-rt8gdz states (amended 2026-10-06, review round 1). Otherwise it returns an Invite reply Blob carrying that
Persona's Member-as-a-group key, DevicePublicKey, handle, name and surname and
the Invite's Organisation identifier and invite identifier, and before
returning declares to org-node, through `expect_admission(org_id)`, the
first admission to that Organisation it expects.
satisfies: REQ-tcutr6, REQ-ab2mfz

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The declaration was `expect_admission(org_id, invite_id)`. REQ-tcutr6 as
amended declares the Organisation alone, because org-node matches a first
admission against the Organisation alone; the reply still carries the invite
identifier back to the inviter.

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
`admit_reply(svc, writer, outstanding, rng, org_id, reply_blob, peer_addr)`
refuses, acting on nothing, a reply `check_reply` refuses, and
one whose outstanding pair's Organisation is not `org_id`, with `this reply is
for another Organisation`. The target Organisation is the one the outstanding
pair names: `org_id` is kept as the operator's selection in the Admit panel,
which preselects it from the reply, and is not trusted — it can only confirm
that Organisation, never choose another. Otherwise `admit_reply` builds
org-node's `Joiner` from the reply's five values, has org-node build the
admission to that Organisation, submits it as org-io's write-then-commit-then-send
order states (org-io's architecture ledger, the submission item) with the
reply's DevicePublicKey as the one recipient (LLR-q225ws), passing org-node no
Organisation secret, key or invite identifier, and settles that pair once the
admission has committed — even when the send that follows fails; when the
submission fails the pair stays outstanding.
satisfies: REQ-65xqp8, REQ-m8sgjk

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).* The
signature was `admit_reply(svc, writer, outstanding, rng, org_id, reply_blob, peer_addr)`
and is `admit_reply(io, outstanding, rng, org_id, reply_blob, peer_addr)`:
the service and the chain writer are both behind the org-io handle, whose
`submit_commit_send` it calls. The text said it "submits it as" the app's
order requirement states; that requirement moved to org-io and is not
exported, so it is named in prose. `satisfies:` was REQ-65xqp8, REQ-nfr3n2;
REQ-nfr3n2 moved to org-io, and an app low-level requirement does not
decompose another unit's requirement, so it is replaced by the app's
REQ-m8sgjk (the decision to admit, handed to org-io). Classification under
the hybrid rule: a clarification, not a change of meaning; the behaviour
and the tests are unchanged.

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
`admit_reply` took an `org_secret` and passed it, with the reply's invite
identifier "in the Wire message", to `send_update`. org-node's `send_update`
now takes neither: the Organisation private key comes from org-node's record,
and the invite identifier never travels between peers (REQ-tcutr6 as
amended). The invite identifier still settles the outstanding pair here.

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

*Moved 2026-10-08 (change `worktree-org-io-create`, task T9, ruling A).* The
two low-level requirements that stood here — the write-then-commit-then-send
order of `found_organisation` and `submit_commit_send` with its refusals, and
the 90-second bound on each chain write, with the note that the bound was the
app's — moved, with their IDs, their five tests in `submit_flow.rs` and the
code, to org-io's architecture ledger
(`org-io/docs/architecture/2026-10-08-org-io.md`, under the
submission item), where the bound is org-io's. org-io does not export them,
so this unit names them in prose.

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
