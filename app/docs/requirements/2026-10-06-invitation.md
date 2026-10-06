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

**REQ-prjja8**: The software shall produce, for an Organisation the user holds,
an Invite as a Blob carrying the Organisation's name as the inviter typed it, the
Organisation identifier, the inviter's guess of the invitee's full name, the
inviter's DevicePublicKeys, and an invite identifier of 32 bytes drawn from its
cryptographic random source, and shall keep that identifier as outstanding.
satisfies: derived

The Organisation's name is free text for display. It is stored nowhere and
verified by nothing; the Organisation identifier is what the invitee's node
reads the chain with (owner ruling 2026-10-05). The inviter's DevicePublicKeys are
where the reply will be sent once the transport moves to the app; the invite
identifier lets the inviter discard any reply it did not ask for (owner
journey, 2026-10-05).

**REQ-tcutr6**: The software shall produce, from an Invite the user imported
and a Persona the user chose, an Invite reply as a Blob carrying that
Persona's Member-as-a-group key, DevicePublicKey, handle, name and surname, and the Organisation identifier and
invite identifier the Invite named; and shall declare to org-node that it
expects a first admission to that Organisation.
satisfies: derived

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The declaration named the invite identifier as well, which the admission's
Wire message carried. The owner ruled that the invite identifier travels only
in the Invite and its reply, never between peers; org-node matches a first
admission against the Organisation alone
(`org-node/docs/requirements/2026-10-06-chain-authority.md`).

**REQ-65xqp8**: The software shall refuse an imported Invite reply whose invite
identifier is not one it holds as outstanding for the Organisation the reply
names, acting on nothing from it; shall act on a reply only for the
Organisation the Invite was issued for; and shall stop holding an invite
identifier as outstanding once a reply naming it has been acted on.
satisfies: derived

*Amended 2026-10-06 (independent review round 2, finding-1).* An invite
identifier is outstanding for the Organisation it was issued for, not for any
Organisation: before this amendment a reply to an Invite for one Organisation
could name another Organisation this device holds, and the first Invite's
identifier let it through to an admission there.

**REQ-ab2mfz**: The software shall, before it produces an Invite reply, show
the user that nothing has verified who sent the Invite or the Organisation
name it states, and that the reply will reveal the handle, name and surname
entered for the chosen Persona to its sender, and shall produce the reply only once
the user has confirmed. (implements: RC-wzb48r)
satisfies: derived

**REQ-yazum3**: The software shall refuse an imported Invite or Invite reply whose
Organisation identifier, keys, handle, name or surname does not parse, naming
the field that failed, and shall act on nothing from it.
satisfies: derived

**REQ-nfr3n2**: The software shall submit a provisional update built by
org-node to the chain through on-chain-client, and only once the submission
has executed ask org-node to commit and send it; when the submission fails it
shall ask org-node to do neither and shall report the failure.
satisfies: derived
