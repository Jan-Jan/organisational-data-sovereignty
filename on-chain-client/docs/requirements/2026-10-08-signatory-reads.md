# Requirements — the signatory-set reads

Added 2026-10-07 by change `worktree-org-io-create`, which creates the org-io
unit. org-io's own-admin check confirms that the user's signatory key belongs
to the multisig that controls an Organisation's pure proxy; it needs two chain
values this unit did not yet read: the account pallet-revive maps the
Organisation's H160 to, and that account's proxy delegates. This file adds the
two reads. Their consumer is org-io's own-admin requirement, named here in
prose only: this unit does not depend on org-io, so no org-io ID appears.

Both requirements are `satisfies: derived`: they exist because org-io needs
them, not because a system-needs document asked for them. Each read is at the
latest finalised block, like the Organisation slot read, and each returns a
typed error rather than an empty answer when the chain's value cannot be read
or decoded. Terms: *Organisation admin* is defined in
`on-chain-client/docs/CONTEXT.md`.

**REQ-8p2veg**: The software shall provide a read of the account that
pallet-revive maps an Organisation's H160 to (`Revive.OriginalAccount`) at
the latest finalised block, returning the 32-byte account when the H160 is
mapped, nothing when it is not, and an error when the read fails or the
stored value is not a 32-byte account.
satisfies: derived
exported: yes

*Narrowed 2026-10-13 (change worktree-org-io-publish, review round 1,
finding-8):* the read is provided only as part of REQ-b8x2py's one pinned
read (`OrgRegistryClient::signatory_set`, its `original_account` field),
whose block is the latest finalised one and whose answers and errors are
this item's; the separate method `OrgRegistryClient::original_account` is
removed (nothing called it, and a caller pairing it with another read at
its own block rebuilt PR-ksm2ua's shape). Narrowing: the pinned read also
fails when the runtime has no `Any` proxy type, so in that runtime this
read gives an error where it gave an answer.

**REQ-v8jczx**: The software shall provide a read of an account's proxy
delegates (`Proxy.Proxies`) at the latest finalised block, returning every
delegate account the stored definitions name, an empty list when the account
has no proxies, and an error when the read fails or the stored value does not
decode as the runtime's proxy definitions.
satisfies: derived
exported: yes
superseded-by: REQ-b8x2py

*Superseded 2026-10-08 (change worktree-org-io-publish, task T16a; owner
rulings D4 and D5, hybrid: a change of meaning mints a new ID).* This read
returns delegates only, at its own finalised block. REQ-b8x2py
(`on-chain-client/docs/requirements/2026-10-13-publish.md`)
returns every definition with its proxy type and delay, with the mapped
account and the `Any` index, all at one finalised block.

*Note 2026-10-13 (change worktree-org-io-publish, review round 1,
finding-8):* the read this item asked for, `OrgRegistryClient::proxy_delegates`,
is removed (unused, and read at its own block).
