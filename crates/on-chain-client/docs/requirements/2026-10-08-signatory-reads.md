# Requirements — the signatory-set reads

Added 2026-10-07 by change `worktree-org-io-create`, which creates the org-io
unit. org-io's own-admin check confirms that the user's signatory key belongs
to the multisig that controls an Organisation's pure proxy; it needs two chain
values this unit did not yet read: the account pallet-revive maps the
Organisation's H160 to, and that account's proxy definitions. This file adds the
first read; the second is REQ-b8x2py (`2026-10-13-publish.md`), which reads
both at one finalised block. Their consumer is org-io's own-admin requirement, named here in
prose only: this unit does not depend on org-io, so no org-io ID appears.

The requirement is `satisfies: derived`: it exists because org-io needed
it, not because a system-needs document asked for it. Each read is at the
latest finalised block, like the Organisation slot read, and each returns a
typed error rather than an empty answer when the chain's value cannot be read
or decoded. Terms: *Organisation admin* is defined in
`crates/on-chain-client/docs/CONTEXT.md`.

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
its own block would combine values from two chain states). Narrowing: the pinned read also
fails when the runtime has no `Any` proxy type, so in that runtime this
read gives an error where it gave an answer.

*Note 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; owner
ruling R3 of 2026-11-02):* no consumer since 2026-11-02; kept for the
several-Admins stage. Its only consumer, org-io's own-admin check, is
removed (change `worktree-org-io-publish-2`, org-io's owner ruling 6;
citation corrected 2026-11-09, merge review MR1-C4); this read, its export and
its tests stay as they are.
