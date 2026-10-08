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
traces: REQ-8p2veg, REQ-v8jczx

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
