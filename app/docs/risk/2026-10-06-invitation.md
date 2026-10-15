# Risk analysis — the invitation exchange and submitting updates (app)

Run 2026-10-05 for change `worktree-org-node-chain-authority`, under IEC 62304
Class C and the acceptability matrix in this ledger's README. The owner moved
the invitation exchange out of org-node into this unit and made this unit
submit each update org-node builds
(`app/docs/requirements/2026-10-06-invitation.md`).

## A new hazard: an Invite from someone who is not a Member

**HAZ-qfb95k**: an Invite's sender, its DevicePublicKeys and the Organisation name
it states are verified by nothing, and the invitee has no record to check them
against; someone who is not a Member of the Organisation sends an Invite
naming that Organisation's real identifier and a familiar name, and the
invitee replies; the handle, name and surname the invitee entered for that
Persona reach a party outside the Organisation. Severity: S2. Probability: P2.

P2: impersonation over the third-party channel an Invite travels on is
ordinary, and nothing in an Invite distinguishes a Member from anyone else.

S2, by owner ruling (2026-10-05): the keys in the reply expose nothing, because
a Persona's keys are generated for that one Organisation and link to no other
identity; what reaches the impostor is the name details the invitee chose to
enter, which may be a pseudonym. The first draft of this item rated it S3 on
the keys and the name together.

**RC-wzb48r**: before producing an Invite reply, the app tells the user that
nothing has verified who sent the Invite or the Organisation name it states,
and that the reply reveals the handle, name and surname entered for the chosen
Persona to that sender, and produces the reply only once the user confirms.
mitigates: HAZ-qfb95k

This is information for safety, the lowest rank in ISO 14971's order, and the
owner chose it knowingly (2026-10-05). A design control would need to show the
invitee that the Invite's sender is a Member or a signatory of the
Organisation's multisig, which nothing in this change can establish; the
multisig signatory read is planned for a later change of the sequence.

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).* The
signatory read now exists, in org-io, and checks this node's own admin
status only: the chain can confirm a signatory set but cannot list one, so
the inviter's status cannot be checked without the inviter's signatory set,
which an Invite does not carry (accepted recommendation, 2026-10-06). The
control therefore stays information for safety, and the app does not call
org-io's own-admin check in S2 (owner ruling of 2026-10-07). The residual
and its acceptance below are unchanged. Classification: a clarification of
the rationale, not a change of meaning.

What this control breaks: nothing a user relies on; it adds one confirmation
to joining.

Residual: Severity S2, probability P2, **accepted by owner ruling
(2026-10-05)** on the grounds that the invitee chose the channel, chose the
name details, and is told before replying what the reply reveals. The matrix
places S2/P2 in the unacceptable region; this is a deliberate acceptance,
recorded as such.

## Derived requirements assessment

*Moved 2026-10-14 (S4, task T13 of docs/plans/2026-10-06-org-io-transport.md,
ruling A).* The assessments of the three invitation requirements that stood
here — producing an Invite with a random identifier, refusing a reply naming
no identifier outstanding for its Organisation (with its review-round-2
amendment), and producing a reply that declares the expected admission —
moved with the requirements to org-io's risk ledger
(`org-io/docs/risk/`, the transport file of change
`worktree-org-io-transport`), where they are assessed again against
org-io's hazards. The reply's disclosure stays this unit's HAZ-qfb95k,
controlled by RC-wzb48r through REQ-ab2mfz.

**REQ-ab2mfz** realises RC-wzb48r; assessed above.

*Moved 2026-10-13 (S4, task T6 of docs/plans/2026-10-06-org-io-transport.md,
ruling A).* The assessment of the Blob-parse requirement that stood here (no
hazard impact: the parse-at-the-system-edge rule, and refusing acts on
nothing) moved with the requirement to org-node's risk ledger
(`org-node/docs/risk/2026-10-06-chain-authority.md`). This file's `assesses:`
line keeps its four other requirements.

*Moved 2026-10-08 (change `worktree-org-io-create`, task T9, ruling A).* The
assessment of the submission requirement that stood here (submit through
on-chain-client, then ask org-node to commit and send) moved with the
requirement to org-io's risk ledger
(`org-io/docs/risk/2026-10-08-org-io.md`), where it is
re-assessed for the signatory key's custody, now org-io's. This file's
`assesses:` line keeps its five other requirements. The app's residue, the
decision when to submit and to which Device, is assessed in
`app/docs/risk/2026-10-08-submission.md`. *(2026-10-14, S4 task T13: three
of those left with the exchange, above; the line keeps REQ-ab2mfz.)*

assesses: REQ-ab2mfz

## Derived low-level requirements amended by the design (2026-10-06)

The design step wrote eight low-level requirements for the requirements above
(`app/docs/architecture/2026-10-06-invitation.md`);
each satisfies one of them, and none is derived. The 90-second bound on each chain
write: a write that is still pending when the bound elapses is
reported as failed and nothing is committed or sent, which is REQ-nfr3n2's
failure case. *Note 2026-10-08 (task T9 of `worktree-org-io-create`):* that
bound, its low-level requirement and the submission requirement moved to
org-io's ledgers, and "the app" below is org-io since then; the rest of this
paragraph stands. A write that executes on the chain after the app gave up leaves
the chain ahead of the node. org-node's `commit_update` could reach it from
the provisional update the failure kept, but no app path retries that commit
today: the only caller is the next submission's `submit_commit_send`, which
can then commit the earlier, timed-out update in place of its own and send it
to the new recipient — the problem report on kept provisional updates
records this.
Either way nothing is committed that the chain does not carry.
Two derived requirements of the 2026-10-05 decomposition are amended in place.

*Amended 2026-10-06 (independent review round 2, finding-8).* This paragraph
said the kept update is reached by `commit_update`, as though the app did so;
it is qualified to what the app does today, with the problem report on kept
provisional updates cross-referenced.

*Note 2026-10-13 (change `worktree-org-io-publish`, task T17a).* That problem
report moved to org-io's problems ledger, with its ID, and is resolved there:
org-io settles a write whose outcome is unknown before any second write for
the Organisation, commits a late write and sends it to its own submission's
recipient, and discards the other kept updates in the background. The
paragraph above describes the code before that change.
`app/docs/problems/2026-10-06-provisional-housekeeping.md` points to it; this
file names the report by description because the app may cite only org-io's
exported items.

**LLR-ctrfz4** (`admit_member` reads the joiner's peer
address from its own argument, blank meaning absent): no new hazard. It
replaces reading the address from a join request; where the admission is sent
is still the operator's input, and in Networked transport the address is not
used. *(2026-10-13, S4 task T12: the item moved to org-io with the parse, its
assessment with it, to `org-io/docs/risk/2026-10-06-transport.md`;
its ID left the `assesses:` line below. org-io now also sends in Loopback
mode only to an address whose identity is a recipient's Device key.)*
*(2026-10-15, S4 TEST conformance review finding-4: the item moved back to
this unit, derived again, with its ID; it is back on the `assesses:` line
below, and org-io's copy of the assessment is a prose pointer.)*

**LLR-pmus9f** (`import_invite_reply` shows a parsed reply and stores
nothing, and refuses one naming no outstanding invite): no new hazard. It
replaces `import_join_request`; refusing a reply that names no outstanding
invite is REQ-65xqp8's control against an unsolicited reply, assessed above.

assesses: LLR-ctrfz4, LLR-pmus9f

## Derived low-level requirement of review round 1 (2026-10-06)

*Moved 2026-10-14 (S4, task T13, ruling A).* The assessment that stood here,
of the low-level requirement that founding and replying offer only Personas
bound to no Organisation and that the backend refuses a bound one (no new
hazard; it removes one), moved with the item to org-io's risk ledger
(`org-io/docs/risk/`, the transport file of change
`worktree-org-io-transport`). The webview's offer of only unbound Personas
stays the app's code; it adds no hazard of its own, since the backend
refusal holds when the webview is bypassed.
