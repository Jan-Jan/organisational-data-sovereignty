# Design — the chain writer: genesis ceremony and update submission

The software item and low-level requirements for change
`worktree-org-node-chain-authority`, which moves the chain write into this
unit (`on-chain-client/docs/requirements/2026-10-06-chain-write.md`,
REQ-6jefu2 and REQ-aat4yt). The code moves from org-node — `chain_write/`
(`calldata.rs`, `multisig.rs`, `proxy.rs`, `mod.rs`) and `ceremony.rs` — and
its behaviour is unchanged except where stated below; org-node's design now
states that it holds none of it
(`org-node/docs/architecture/2026-10-06-chain-authority.md`
and the items it amends in `org-node/docs/architecture/2026-10-03-decomposition.md`).

This file adds one software item, SDD-yg7n55, and fourteen low-level
requirements under it (LLR-24mwew added by review round 1). The unit's Overview in `README.md` is amended for it.
Class C, the unit's class, with no per-item override.

## The constraints the owner set

- **Behind a `write` feature.** `write = ["client", "dep:subxt-signer",
  "dep:blake2"]`. With `write` off the crate compiles exactly as it does today;
  `default` does not enable it. The two crates are SOUP rows in `soup.md`.
- **This unit's own value types.** The writer's interface takes and returns
  `OnChainRootHash`, `OrgPubKey`, `Epoch` and `OrgAdmin` from `types.rs`, and a
  new `AccountId([u8; 32])` defined in the writer module for a 32-byte chain
  account (signatory, co-signatory, proxy). `on-chain-client/Cargo.toml` lists
  neither org-node nor org-members, with or without `write`, so the provider
  does not depend on its consumer.
- **No new runtime dependency.** The settle loop that waits for an extrinsic
  to finalise uses `futures-util` (already a dependency) rather than tokio,
  which this unit has only as a dev-dependency, and the seam's async methods
  are native `async fn` in a trait used generically, so `async-trait` is not
  added.
- **Not carried over.** org-node's `proxy::rotate` and the direct
  single-signer `submit::submit_update` have no requirement here and no
  production caller; they do not move. `FUND_AMOUNT` moves as the genesis
  ceremony's funding constant.

## SDD-yg7n55 — The chain writer

`on-chain-client/src/write/` (new, behind the `write` feature): `mod.rs`
(`AccountId`, `WriteError`, `GenesisStep`, `DispatchOutcome`, the `WriteOps`
seam, `Genesis`), `calldata.rs` (`UPDATE_SELECTOR`, `build_update_calldata`,
`revive_update_runtime_call`), `multisig.rs` (`multi_account_id`,
`build_dispatch_tx`), `proxy.rs` (`proxied`, `map_account_call`, `is_proxied`),
`ceremony.rs` (`genesis`, `submit_update`), `events.rs` (`ObservedEvent`,
`outcome_of`, `observed_event`, `dispatch_result`; LLR-24mwew), and
`subxt_ops.rs` (`SubxtWriteOps`, `BlockSink` and `FinalitySink`, the subxt
implementation of the seam).

**SDD-yg7n55**: the chain writer — given a signatory key and the
co-signatories of the multisig that controls an Organisation, it stands up the
Organisation's slot (pure proxy, funding, pallet-revive mapping, the genesis
record at epoch zero) and submits each later update to that slot through the
proxy, returning only once the call has executed and reporting every other
outcome as a typed error. It holds no key and stores nothing: the signatory
key is an argument of every call. The composition is written against a seam,
`WriteOps` (create a pure proxy, fund an account, dispatch a call under the
controller), so the order of the steps, which step failed and whether a
dispatch executed are decided by code a gated test reaches through a
substitute; `SubxtWriteOps` implements the seam over subxt.
traces: REQ-6jefu2, REQ-aat4yt

`SubxtWriteOps` and `FinalitySink` carry no low-level requirement, for the
reason SDD-3b8zef carries none (the section "The item with no low-level
requirements" of `2026-09-28-decomposition.md`): what they do — sign, submit,
wait for finality, read `Proxy.PureCreated`, decode events into
`ObservedEvent`s — is reachable only against a
chain, by targets this unit's gate does not run. That is the same recorded
deviation, extended to the writer's transport, not a new one; the seam exists
so that everything else in the writer is refined and gated.

**LLR-a2acvh**: `genesis(ops, signatory, co_signatories, contract, root:
OnChainRootHash, key: OrgPubKey) -> Result<Genesis, WriteError>` performs, in
this order and each only after the previous one has succeeded: create the pure
proxy under the controller (`WriteOps::create_pure`); fund it from the
signatory with `FUND_AMOUNT` (`WriteOps::fund`); dispatch `proxied(proxy,
map_account_call())`; dispatch `proxied(proxy, revive_update_runtime_call(
contract, root, key, Epoch(0)))`; and returns `Genesis { proxy, admin }` where
`admin` is `OrgAdmin(h160_of(proxy))`.
satisfies: REQ-6jefu2

**LLR-z9vugt**: when a genesis step returns an error, or a dispatched step's
outcome is not `Executed`, `genesis` returns `WriteError::Step { step, reason
}` whose `step` is the `GenesisStep` that failed — `CreatePure`, `Fund`,
`MapAccount` or `RecordGenesis` — and performs no later step.
satisfies: REQ-6jefu2

**LLR-qs8jfw**: `build_dispatch_tx(co_signatories, call)` returns the call as
a top-level extrinsic when there are no co-signatories, and otherwise
`Multisig.as_multi_threshold_1(sorted co_signatories, call)`, the
co-signatories sorted by their bytes; every dispatch the writer makes goes
through it.
satisfies: REQ-6jefu2, REQ-aat4yt

**LLR-kv27gp**: `submit_update(ops, signatory, co_signatories, contract,
proxy: AccountId, root: OnChainRootHash, key: OrgPubKey, expected_epoch:
Epoch) -> Result<(), WriteError>` dispatches exactly one call, `proxied(proxy,
revive_update_runtime_call(contract, root, key, expected_epoch))`, under the
controller, and returns `Ok(())` only when that dispatch's outcome is
`Executed`.
satisfies: REQ-aat4yt

**LLR-hun4wf**: `submit_update` returns `WriteError::PendingApproval` when the
dispatch's outcome is `ApprovalRecorded` — a multisig approval that did not
execute the call — and returns the dispatch's own error when the dispatch
fails, so no outcome other than an executed update is reported as success.
satisfies: REQ-aat4yt

**LLR-24mwew**: `outcome_of(proxied: bool, events: &[ObservedEvent]) ->
Result<DispatchOutcome, WriteError>` decides what a finalised, successful
dispatch did from its events, where `ObservedEvent` is
`ProxyExecuted(Result<(), String>)` — pallet-proxy's `Proxy.ProxyExecuted`
with its inner call's result, the error rendered as text — or `Other`. For a
proxied call (`is_proxied(&call)`: the call is `Proxy.proxy`) it returns
`WriteError::InnerCallFailed(error)` when any `ProxyExecuted` carries an
error, `WriteError::EventNotFound("Proxy.ProxyExecuted")` when there is no
`ProxyExecuted`, and `Ok(DispatchOutcome::Executed)` otherwise; for a call
that is not proxied it returns `Ok(DispatchOutcome::Executed)`.
The events are decoded by two pure functions over the scale-value form subxt
gives. `dispatch_result(fields: &Composite<()>) -> Result<Result<(), String>,
WriteError>` reads `ProxyExecuted`'s `result` field (the named field
`result`, or the first unnamed field): a variant `Ok` is `Ok(Ok(()))`, a
variant `Err` is `Ok(Err(text))` with the error's fields rendered as text, no
`result` field is `WriteError::MalformedEvent("ProxyExecuted has no result")`,
a `result` that is not a variant is `WriteError::MalformedEvent("ProxyExecuted
result is not a variant")`, and any other variant is
`WriteError::MalformedEvent("ProxyExecuted result is neither Ok nor Err")`.
`observed_event(pallet, event, fields)` is `ProxyExecuted(dispatch_result(..))`
for `Proxy.ProxyExecuted`, returning the field decoder's error unchanged, and
`Other` for any other event without decoding its fields.
`SubxtWriteOps::dispatch` only fetches the extrinsic's events, maps each
through `observed_event` and returns `outcome_of`'s answer, so through `genesis` an inner failure is
`WriteError::Step { step, .. }` naming the step (LLR-z9vugt) and through
`submit_update` it is the update's error (LLR-hun4wf).
satisfies: REQ-6jefu2, REQ-aat4yt

*Added 2026-10-06 (independent review round 1, finding-2).* `Proxy.proxy`
succeeds as an extrinsic when its inner call fails, so a reverted `Revive.call`
(a stale expected epoch) or a failed `map_account` was reported as executed.
`DispatchOutcome::ApprovalRecorded` stays in the seam for the composition's
sake, but `SubxtWriteOps` never produces it: `as_multi_threshold_1` dispatches
at once and records no approval.

*Amended 2026-10-06 (independent review round 2, finding-2).* The decoding of
`Proxy.ProxyExecuted` lived in the subxt shell, out of reach of any gated test,
so a decoder that mapped `Err` to `Ok` or missed the `result` field passed
every gate. It moved to `events.rs` as `dispatch_result` and `observed_event`,
stated above as directly testable rules; `SubxtWriteOps` keeps only the fetch.

**LLR-yvq33e**: `build_update_calldata(root, key, expected_epoch)` emits
exactly one hundred bytes: the four-byte selector, then the Membership root,
then the Organisation public key, then the expected epoch, in that order.
satisfies: REQ-aat4yt, REQ-6jefu2

**LLR-5varjf**: the expected epoch occupies the low sixteen bytes of a
thirty-two byte big-endian field, with the high sixteen bytes zero.
satisfies: REQ-aat4yt

**LLR-d4ftc8**: `UPDATE_SELECTOR` is the first four bytes of the keccak-256 of
`update(bytes32,bytes32,uint256)`, so a signature drift in the deployed
contract is a changed constant here rather than a silently rejected call.
satisfies: REQ-aat4yt

**LLR-m7wmmx**: `revive_update_runtime_call(contract, root, key, epoch)` builds
the `Revive.call` runtime call by name: `dest` is the contract's twenty bytes,
`value` is zero, `weight_limit` carries `ref_time` and `proof_size` at the
declared constants, `storage_deposit_limit` is the declared constant, and
`data` is exactly `build_update_calldata`'s bytes for the same arguments. The
field names and constants are matched against runtime metadata, so each is
part of the claim.
satisfies: REQ-aat4yt

**LLR-ywhd23**: `proxied(proxy, call)` builds `Proxy.proxy` with `real` the
`Id` of the proxy's thirty-two bytes, `force_proxy_type` `None` and `call` the
call given, and `map_account_call()` builds `Revive.map_account` with no
arguments.
satisfies: REQ-aat4yt, REQ-6jefu2

**LLR-gjx3jn**: `multi_account_id(signatories, threshold)` returns the
blake2-256 of the SCALE encoding of `("modlpy/utilisuba", sorted signatories,
threshold)`, sorting the signatories before hashing, so the account it derives
does not depend on the order they were supplied in.
satisfies: REQ-6jefu2

**LLR-kqxw9t**: the account `multi_account_id` derives depends on the
threshold, so two otherwise identical signatory sets at different thresholds
derive different accounts.
satisfies: REQ-6jefu2

**LLR-rxs5ec**: the `write` module and everything it exports compile only with
the `write` feature, and enabling it adds exactly `subxt-signer` and `blake2`
to the crate's normal dependencies; `on-chain-client/Cargo.toml` names neither
org-node nor org-members under any feature.
satisfies: derived

## Where the evidence comes from

The pure requirements — LLR-qs8jfw, LLR-yvq33e, LLR-5varjf, LLR-d4ftc8,
LLR-m7wmmx, LLR-ywhd23, LLR-gjx3jn and LLR-kqxw9t — are those org-node's
`chain_write_pure` and `calldata_typed` targets pinned under org-node's
identifiers; the tests move with the code and are re-annotated here. The
composition requirements — LLR-a2acvh, LLR-z9vugt, LLR-kv27gp and LLR-hun4wf —
are new evidence, carried by a gated target driving `genesis` and
`submit_update` through a substitute `WriteOps` that records the steps it is
asked for and can fail or answer `ApprovalRecorded` at any of them. Every new
target joins `verify_commands` with `--features test-support,write`.

`MalformedCall` — `runtime_call_to_tx`'s three arms, untested in org-node
(its robustness table in `org-node/docs/architecture/2026-10-03-decomposition.md`)
— comes with `build_dispatch_tx` and is owed here as that function's abnormal
input.
