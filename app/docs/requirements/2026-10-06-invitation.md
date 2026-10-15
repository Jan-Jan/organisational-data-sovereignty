# Requirements — the invitation exchange and submitting updates

Decided 2026-10-05 in the `grill-requirements` interview of change
`worktree-org-node-chain-authority`. The owner ruled that the invitation
exchange is outside org-node and happens in the app: a Member who invites
someone sends them an Invite over a third-party channel of their choosing, and
the invitee replies with the public keys and details the inviter needs to
admit them. The owner also moved the chain write out of org-node, so the app
submits each update org-node builds before org-node commits and sends it.

Every requirement is `satisfies: derived`: each exists because of where the
owner placed the invitation exchange and the chain write. Per this unit's
citation rule, nothing below names a provider's identifier; org-node's
provisional updates are described in
`org-node/docs/requirements/2026-10-06-chain-authority.md`
and on-chain-client's submission in
`on-chain-client/docs/requirements/2026-10-06-chain-write.md`.

*Moved 2026-10-14 (S4, task T13 of docs/plans/2026-10-06-org-io-transport.md,
ruling A).* The three requirements that stood here — produce an Invite and
keep its identifier outstanding; produce an Invite reply and declare the
expected first admission; refuse a reply that is not outstanding for the
Organisation it names, act on a reply only for the Organisation its Invite
was issued for, and settle the identifier once acted on — moved, with their
IDs, their text and their notes, to org-io's requirements
(`org-io/docs/requirements/`, the transport file of change
`worktree-org-io-transport`, "The invitation exchange"), with the code that
runs the exchange (`org-io/src/exchange.rs`). org-io exports them, so this
unit may still cite them. What the app keeps is the confirmation below
(REQ-ab2mfz), the panels, and the decision to call each operation.

**REQ-ab2mfz**: The software shall, before it produces an Invite reply, show
the user that nothing has verified who sent the Invite or the Organisation
name it states, and that the reply will reveal the handle, name and surname
entered for the chosen Persona to its sender, and shall produce the reply only once
the user has confirmed. (implements: RC-wzb48r)
satisfies: derived

*Moved 2026-10-13 (S4, task T6 of docs/plans/2026-10-06-org-io-transport.md,
ruling A).* The requirement that stood here — refuse an imported Invite or
Invite reply whose Organisation identifier, keys, handle, name or surname
does not parse, naming the field, and act on nothing from it — moved, with
its ID and its text, to org-node's requirements
(`org-node/docs/requirements/`, the sans-IO file of change
`worktree-org-io-transport`), with the parse that realises it: the Invite and
its reply return to org-node.

*Moved 2026-10-08 (change `worktree-org-io-create`, task T9, ruling A).* The
requirement that stood here — submit org-node's provisional update to the
chain through on-chain-client, and only once the submission has executed
ask org-node to commit and send it, reporting a failure and asking for
neither — moved, with its ID and its text, to org-io's requirements
(`org-io/docs/requirements/2026-10-08-org-io.md`,
"Submission"), with the code that does it. org-io exports it, so this unit
may still cite it. What the app keeps — when to submit, and to which Device
— is this unit's new requirement in
`app/docs/requirements/2026-10-08-submission.md`.
