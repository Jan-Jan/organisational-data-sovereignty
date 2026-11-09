# Design — the signatory-set reads

The software item and low-level requirements for the two reads change
`worktree-org-io-create` adds to this unit
(`on-chain-client/docs/requirements/2026-10-08-signatory-reads.md`,
REQ-8p2veg; the proxy-definitions read is REQ-b8x2py). The pure decoders live beside the writer's
`multi_account_id` under the `write` feature, because their only consumer
(org-io) already builds this unit with `write` and works in the writer's
`AccountId`; the two fetches are subxt shell in `client.rs`. Class C, the
unit's class, with no per-item override.

**SDD-rxfu6h**: the signatory-set reads: the pure decoders of the two
storage values that let a consumer confirm a multisig signatory set
(`write/signatory_set.rs`, compiled with the `write` feature beside
`multi_account_id`), and their two fetches in `client.rs`, which are the
subxt shell SDD-3b8zef's and carry no low-level requirement by its recorded
deviation.
traces: REQ-8p2veg, REQ-b8x2py

*Note 2026-10-08 (change worktree-org-io-publish, task T16a):* the item also
realises REQ-b8x2py, the one pinned read (LLR-n2vx5q, LLR-sc79gq in
`on-chain-client/docs/architecture/2026-10-13-publish.md`).

*Note 2026-10-13 (change worktree-org-io-publish, review round 1,
finding-8):* "their two fetches in `client.rs`" are gone. The public
`OrgRegistryClient::original_account` and `proxy_delegates`, and the
private `fetch_storage_raw` they shared, were removed: nothing called them
once org-io moved to `signatory_set`, and each resolved its own finalised
block, so a caller pairing them read the two values at two chain states. The
item's one fetch is `signatory_set`'s, at one pinned block (LLR-sc79gq).
The decoders stay.

*Note 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; owner
ruling R3 of 2026-11-02):* no consumer since 2026-11-02; kept for the
several-Admins stage. The item, its decoders, `signatory_set` and
`any_proxy_type_index` stay exported with their tests; the chain write needs
none of them (`proxied()` forces `Any` by variant name, LLR-59kxsr).

**LLR-m3tjvp**: `decode_original_account(bytes: &[u8]) -> Result<AccountId, SignatorySetError>`
accepts exactly 32 bytes and returns them as the account; any other length is
`SignatorySetError::Malformed`.
satisfies: REQ-8p2veg

*Note 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; owner
ruling R3 of 2026-11-02):* no consumer since 2026-11-02; kept for the
several-Admins stage, with its tests.
