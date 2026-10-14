# Design — the signatory-set reads

The software item and low-level requirements for the two reads change
`worktree-org-io-create` adds to this unit
(`on-chain-client/docs/requirements/2026-10-08-signatory-reads.md`,
REQ-8p2veg and REQ-v8jczx). The pure decoders live beside the writer's
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
traces: REQ-8p2veg, REQ-v8jczx, REQ-b8x2py

*Note 2026-10-08 (change worktree-org-io-publish, task T16a):* the item also
realises REQ-b8x2py, the one pinned read (LLR-n2vx5q, LLR-sc79gq in
`on-chain-client/docs/architecture/2026-10-13-publish.md`).

*Note 2026-10-13 (change worktree-org-io-publish, review round 1,
finding-8):* "their two fetches in `client.rs`" are gone. The public
`OrgRegistryClient::original_account` and `proxy_delegates`, and the
private `fetch_storage_raw` they shared, were removed: nothing called them
once org-io moved to `signatory_set`, and each resolved its own finalised
block, so a caller pairing them rebuilt PR-ksm2ua's two-block shape. The
item's one fetch is `signatory_set`'s, at one pinned block (LLR-sc79gq).
The decoders stay.

**LLR-m3tjvp**: `decode_original_account(bytes: &[u8]) -> Result<AccountId, SignatorySetError>`
accepts exactly 32 bytes and returns them as the account; any other length is
`SignatorySetError::Malformed`.
satisfies: REQ-8p2veg

**LLR-a9bb7b**: `decode_proxy_delegates(bytes: &[u8]) -> Result<Vec<AccountId>, SignatorySetError>`
decodes the SCALE value `(BoundedVec<ProxyDefinition<AccountId32, ProxyType, u32>>, u128)`
— a compact length `n`, then `n` definitions of a 32-byte delegate, a
one-byte proxy type and a four-byte little-endian delay, then a 16-byte
deposit — and returns the `n` delegates in stored order; a value with
trailing bytes, too few bytes, or a length prefix larger than the bytes
that follow is `Malformed`, and no input panics or allocates more than the
input's length allows.
satisfies: REQ-v8jczx
superseded-by: LLR-n2vx5q

*Superseded 2026-10-08 (change worktree-org-io-publish, task T16a; owner
rulings D4, D5, hybrid).* LLR-n2vx5q
(`on-chain-client/docs/architecture/2026-10-13-publish.md`)
returns each definition's proxy type and delay with its delegate.

*Note 2026-10-13 (task T16a).* `decode_proxy_delegates` is kept, now the
delegates of `decode_proxy_definitions`' result, and `proxy_delegates` with
it (no consumer calls it since org-io moved to `signatory_set`); no test of
this item was deleted: `proxy_definitions_give_their_delegates_in_order` and
`truncated_padded_or_overlong_definitions_are_malformed` still pass unchanged.
`fuzz_signatory_set`'s header gained LLR-n2vx5q with the new decoder and two
properties (an accepted value accounts for every byte; the delegates are
the definitions' delegates).

*Note 2026-10-13 (change worktree-org-io-publish, review round 1,
finding-8):* `proxy_delegates`, the client read the note above kept, is
removed (see SDD-rxfu6h's note of this date); `decode_proxy_delegates` and
its tests stay as they are.
