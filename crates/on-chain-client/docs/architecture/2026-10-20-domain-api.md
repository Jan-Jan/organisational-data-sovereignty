# Design — the genesis ceremony in two steps

The low-level requirements for this unit's part of change
`worktree-org-io-domain-api` (owner answer A4, 2026-10-16):
`crates/on-chain-client/docs/requirements/2026-10-16-domain-api.md`,
REQ-v86pyy (create the proxy, then found the Organisation at it) and
REQ-6vh3ht (a founding re-run at the same proxy skips what the chain shows
done). Both are exported and meet org-io's expectation of a two-step
genesis; this unit depends on no other, so the consumer is named in prose
only. Class C, the unit's class, with no per-item override.

No new software item. The two steps belong to the chain writer,
SDD-yg7n55 (`crates/on-chain-client/docs/architecture/2026-10-06-chain-write.md`),
whose `traces:` gains REQ-v86pyy and REQ-6vh3ht and whose seam gains two
reads (its note of 2026-10-20). The one-call ceremony's order is restated
here as LLR-hea6y2, and its step-naming errors as LLR-chy957.

## What changes in the writer

- **Two public steps.** `write/ceremony.rs` gains `create_proxy` and
  `found_at`, re-exported from `write/mod.rs` beside `genesis`.
  `genesis` stays, as their composition (LLR-hea6y2), so REQ-6jefu2's
  one-call ceremony is unchanged for its callers.
- **Two reads on the seam.** `WriteOps` gains
  `free_balance(&self, account: AccountId) -> Result<u128, WriteError>` —
  the account's free balance from `System.Account`, zero for an account the
  chain does not hold — and
  `mapped_account(&self, admin: OrgAdmin) -> Result<Option<AccountId>, WriteError>`
  — the value of `Revive.OriginalAccount` keyed by the H160, `None` when
  the H160 is not mapped. `SubxtWriteOps` reads both at the latest
  finalised block, the same block every dispatch waits for, so a step the
  writer saw execute is a step the next read shows done. The fetch is
  SDD-3b8zef's shell (no low-level requirement, by that item's recorded
  deviation); the decoding is not: `Revive.OriginalAccount`'s value is
  decoded by `decode_original_account` (LLR-m3tjvp, unchanged), and
  `System.Account`'s by the new pure `free_balance_of` (LLR-htp5tk) in
  `write/proxy.rs`.
- **One new error.** `WriteError::MalformedStorage(&'static str)`: a
  storage value that does not decode. `SubxtWriteOps::mapped_account`
  maps `SignatorySetError::Malformed` to it.
- **How the reads are faked.** The gated composition target's substitute
  `WriteOps` (the one that records the steps it is asked for and can fail
  any of them) answers `free_balance` and `mapped_account` from values the
  test sets, records each read in its step log beside the writes, and can
  fail either read. No test needs a chain.

## Which steps are skipped, and why

The founding step decides what is done from the two reads alone, never from
anything held in memory (REQ-6vh3ht):

- **Mapping** is skipped when `mapped_account(h160_of(proxy))` is the proxy
  itself. pallet-revive maps an account only from that account, and only
  this ceremony dispatches the proxy's mapping, so a mapped proxy is one
  this ceremony mapped.
- **Funding** is skipped when the proxy is mapped, or when its free balance
  is at least `FUND_AMOUNT`. The second rule covers a founding interrupted
  between funding and mapping: the transfer credits exactly `FUND_AMOUNT`
  and the signatory, not the proxy, pays every later fee, so the free
  balance is at least `FUND_AMOUNT` until mapping. The first rule is needed
  because mapping holds a deposit from the proxy's free balance, so after
  mapping the balance alone would no longer show the funding done, and a
  retry would fund a second time. A balance below `FUND_AMOUNT` on an
  unmapped proxy (dust sent by anyone) is not taken as funded.
- **Recording the genesis is never skipped**, and the Organisation's slot is
  not read. Once the slot holds a genesis, the contract's own check refuses
  any record carrying an expected epoch of zero (`OrgRegistry.update`
  reverts `EpochMismatch(0, epoch)`), so the inner call fails and the step
  is reported as `GenesisStep::RecordGenesis` not executed — the refusal
  REQ-6vh3ht's note states. A slot holding exactly this root and key is
  refused the same way, not treated as done: the requirement states the
  refusal, the contract's check is atomic with the write where a prior read
  would race it, and the caller can tell its own genesis from another by
  `get_org_state`.

The reads are at the latest finalised block. A step that was submitted but
not finalised when the founding was interrupted is not yet shown done, so a
re-run may repeat it: a repeated mapping fails on the chain and is reported
as `MapAccount` not executed (the next re-run sees it mapped); a repeated
funding transfers `FUND_AMOUNT` once more. That cost is assessed in
`crates/on-chain-client/docs/risk/2026-10-16-domain-api.md`.

## Low-level requirements

**LLR-tge9q8**: `create_proxy(ops, signatory, co_signatories) ->
Result<AccountId, WriteError>` makes exactly one seam call,
`WriteOps::create_pure(signatory, co_signatories)`, and returns the account
it answers; when that call returns an error, `create_proxy` returns
`WriteError::Step { step: GenesisStep::CreatePure, reason }`, the reason the
error's text.
satisfies: REQ-v86pyy

**LLR-q6buam**: `found_at(ops, signatory, co_signatories, contract, proxy:
AccountId, root: OnChainRootHash, key: OrgPubKey) -> Result<Genesis,
WriteError>`, on a proxy the reads show neither mapped nor funded, performs,
in this order and each only after the previous one has succeeded: read
`mapped_account(OrgAdmin(h160_of(proxy)))`; read `free_balance(proxy)`;
fund the proxy from the signatory with `FUND_AMOUNT` (`WriteOps::fund`);
dispatch `proxied(proxy, map_account_call())`; dispatch `proxied(proxy,
revive_update_runtime_call(contract, root, key, Epoch(0)))`; and returns
`Genesis { proxy, admin: OrgAdmin(h160_of(proxy)) }`. It never calls
`create_pure`.
satisfies: REQ-v86pyy

**LLR-chy957**: when a step of `create_proxy` or `found_at` returns an error,
or a dispatched step's outcome is not `Executed`, the function returns
`WriteError::Step { step, reason }` whose `step` names the step and makes no
later seam call. The steps are named: `create_pure` as `CreatePure`; the
balance read and `fund` as `Fund`; the mapping read, a mapping read that
holds an account other than the proxy, and the mapping dispatch as
`MapAccount`; the record dispatch as `RecordGenesis`. `genesis`, their
composition, returns the same error.
satisfies: REQ-v86pyy, REQ-6jefu2

**LLR-jk76wf**: `found_at` makes no `fund` call when `mapped_account`
answers the proxy, or when `free_balance(proxy)` is at least `FUND_AMOUNT`,
and goes on to the mapping step; when the proxy is not mapped and its free
balance is below `FUND_AMOUNT` it funds with `FUND_AMOUNT`. When the proxy is
mapped it does not read the balance.
satisfies: REQ-6vh3ht

**LLR-95puh9**: `found_at` dispatches no mapping call when `mapped_account`
answers the proxy, and goes on to the record step; when it answers `None` it
dispatches the mapping once the funding step is done or skipped.
satisfies: REQ-6vh3ht

**LLR-g8qevz**: `found_at` reads no Organisation slot and always dispatches
the record step once mapping is done or skipped, with an expected epoch of
zero; a record whose inner call fails — as the contract's expected-epoch
check makes it fail on a slot that already holds a genesis — is returned as
`WriteError::Step { step: GenesisStep::RecordGenesis, .. }`, never as
success.
satisfies: REQ-6vh3ht

**LLR-hea6y2**: `genesis(ops, signatory, co_signatories, contract, root,
key) -> Result<Genesis, WriteError>` is `create_proxy(ops, signatory,
co_signatories)` followed, only when that succeeded, by `found_at(ops,
signatory, co_signatories, contract, proxy, root, key)` at the proxy it
returned, and returns `found_at`'s result; on a fresh proxy its seam calls
are LLR-tge9q8's then LLR-q6buam's, in that order.
satisfies: REQ-6jefu2, REQ-v86pyy

**LLR-htp5tk**: `free_balance_of(value: Option<&[u8]>) -> Result<u128,
WriteError>` is zero for `None` (an account the chain does not hold); for
`Some(bytes)` it accepts exactly eighty bytes — `System.Account`'s
`AccountInfo`: four four-byte counters (nonce, consumers, providers,
sufficients), then `free`, `reserved`, `frozen` and `flags` as sixteen-byte
little-endian integers — and returns `free`, the little-endian integer at
bytes sixteen to thirty-one; any other length is
`WriteError::MalformedStorage("System.Account")`, and no input panics.
`SubxtWriteOps::free_balance` returns its answer for the fetched value.
satisfies: REQ-6vh3ht

## Where the evidence comes from

LLR-tge9q8, LLR-q6buam, LLR-chy957, LLR-jk76wf, LLR-95puh9, LLR-g8qevz and
LLR-hea6y2 are composition rules, carried by the gated `write_compose`
target driving the three functions through the substitute `WriteOps`
described above (step log, preset read answers, a failure at any step).
LLR-htp5tk is a pure decoder, carried by `write_pure` with fixed vectors
and by a bolero property in `fuzz_signatory_set` that no input panics. All
run on the existing `--features test-support,write` verify line; no new
target. The chopsticks target `write_genesis_e2e` (ungated, as every
chopsticks target of this unit is) is where `SubxtWriteOps`'s two new
fetches meet a chain.

The tests of the one-call ceremony's order and step errors carry LLR-hea6y2
and LLR-chy957 as inherited coverage of the unchanged fresh-proxy order, not
red-first evidence of the new behaviour.
