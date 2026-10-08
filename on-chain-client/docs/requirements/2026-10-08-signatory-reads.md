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

**REQ-v8jczx**: The software shall provide a read of an account's proxy
delegates (`Proxy.Proxies`) at the latest finalised block, returning every
delegate account the stored definitions name, an empty list when the account
has no proxies, and an error when the read fails or the stored value does not
decode as the runtime's proxy definitions.
satisfies: derived
exported: yes
