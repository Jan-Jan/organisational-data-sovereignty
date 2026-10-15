# app — the invitation exchange and submitting updates: low-level requirements

Low-level requirements for change `worktree-org-node-chain-authority`, which
refine the app requirements of
`app/docs/requirements/2026-10-06-invitation.md`
(REQ-prjja8, REQ-tcutr6, REQ-65xqp8, REQ-ab2mfz, REQ-nfr3n2; the Blob-parse
requirement left for org-node on 2026-10-13, S4 task T6). Written
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
(`app/docs/requirements/2026-10-08-submission.md`), which the reply's
admission item below then satisfied in its place. *(2026-10-14, S4 task
T13: that item moved to org-io, below.)*

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

*Moved 2026-10-13 (S4, task T6 of docs/plans/2026-10-06-org-io-transport.md,
ruling A).* The low-level requirement that stood here — `Invite::parse` and
`InviteReply::parse` take a Blob and refuse the first field that fails,
naming it — moved, with its ID, its three tests and the code, to org-node's
architecture ledger (`org-node/docs/architecture/`, the sans-IO file of
change `worktree-org-io-transport`, under the Invite item). This unit calls
that parse through the org-io handle's re-export and names it in prose.

## Under SDD-rmbr3t — The command surface

*Moved 2026-10-14 (S4, task T13 of docs/plans/2026-10-06-org-io-transport.md,
ruling A).* The five low-level requirements that stood here — `issue_invite`;
the outstanding (Organisation, invite identifier) pairs; `produce_reply` and
its confirmation gate; a bound Persona refused as the one to found with or
reply as; and the reply's check and admission, settled once committed —
moved, with their IDs, their notes, their fourteen tests in
`tests/invitation.rs` (deleted) and the code they describe, to org-io's
architecture ledger (`org-io/docs/architecture/`, the transport file of
change `worktree-org-io-transport`, under the invitation exchange runner),
where org-io runs the exchange and the outstanding pairs live in the sealed
store. org-io does not export them, so this unit names them in prose. What
of them was the app's and stays here, without an item of its own until the
app's rewiring task records one: the webview's offer of only unbound
Personas (`unboundPersonas` in `app/src/lib/invite.ts`, the founding panel's
lack of a choice) and the IPC refusal of an unconfirmed reply, whose tests
(`invite.personas`, `ipc.rs`) stay in the app. *(2026-10-14, S4 task T15:
the webview's offer is recorded as LLR-pms47e, under SDD-jx363y below; the
IPC refusal's test runs with no `verifies:` line.)* *(2026-10-15, S4 TEST
review round 2, finding-2: the IPC refusal's test,
`produce_invite_reply_refuses_without_confirmation_over_ipc` (ipc), now
verifies REQ-ab2mfz directly, whose last clause it checks at the IPC seam;
no app LLR states the IPC refusal.)*

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

**LLR-pms47e**: the webview offers, as the Personas to reply to an Invite
as, only those `unboundPersonas` keeps — the Personas whose `org_id` is null,
in the order given, none when every Persona is bound or there is none — and
the founding panel offers no choice of Persona: it founds with the one it has
just created.
satisfies: derived

Normal: "keeps only the Personas bound to no Organisation, in order"
(invite.personas).
Abnormal: "offers none when every Persona is bound", "offers none from an
empty list", "offers a Persona whose Organisation is the empty string as
bound" (invite.personas).
Review only: the founding clause (dated note below).

Minted 2026-10-14 (S4, task T15 of
`docs/plans/2026-10-06-org-io-transport.md`), from option (a) of task T13's
report: the webview half of the one-Persona-one-Organisation item that moved
to org-io with the backend refusals (task T13, named above in prose). It is
the app's code and stays the app's. Derived, as that item was: the founding
half has no parent among this unit's requirements, and the reply half
narrows org-io's exported reply requirement's "a Persona the user chose". The
filter sits in `invite.ts`, beside `replyGate`; org-io's refusal is what
holds when the webview is bypassed. The three earlier `invite.personas`
tests verified the moved item and keep running with no `verifies:` line
(the T9 precedent); the fourth, written for this item, cites it.

*Review round 1, 2026-10-14:* the three earlier `invite.personas` tests now
cite this item, inherited from the moved one: they test the same filter and
are listed above. The founding clause ("the founding panel offers no choice
of Persona") is **verified by review only**: it is the shape of
`CreateOrg.svelte`'s `run()`, which passes `createOrganisation` the ID
`createPersona` just returned and renders no Persona picker. No helper seam
reaches it without extracting code or faking `$lib/api`, and the unit has no
component test runner.
