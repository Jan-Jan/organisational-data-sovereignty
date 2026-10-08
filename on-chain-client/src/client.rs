//! `OrgRegistryClient` — the public reading API for the deployed
//! `OrgRegistry` contract. Per spec §"Client surface":
//!
//! - [`OrgRegistryClient::get_org_state`] — `(admin, at)` returns the
//!   decoded `OrgState` for an org at a specific block. `Ok(None)` for
//!   a never-written slot. `at = None` reads at the latest finalised
//!   block.
//! - [`OrgRegistryClient::subscribe`] — opens a best-block subscription
//!   and yields `SubscribedEvent`s for the lifetime of the stream.
//!   `admin = None` watches every org; `Some(h160)` filters to one.
//!
//! Implementation:
//!
//! - Uses subxt's `OnlineClient` for the metadata-aware operations
//!   (the `ReviveApi::get_storage` runtime call that reads a contract
//!   slot from pallet-revive's per-contract child trie; `RuntimeEvent`
//!   variant decoding for the event stream).
//!   The `Rpc` trait + `WsRpc` (hand-rolled layer, now deleted) have been
//!   removed; the spec-declared generic `OrgRegistryClient<R: Rpc>` was
//!   dropped here because subxt already
//!   owns the WS connection + metadata cache — having two parallel
//!   connections would just double the bookkeeping. The smoldot path is
//!   subxt's light-client backend (feature `smoldot`), preserving one
//!   connection per client.
//! - Best-block events are surfaced via subxt's best-block subscription
//!   plus per-block event reads (the "best lane"), with gap-fill and
//!   parent-hash-based reorg detection layered on. Finalised events come
//!   from a parallel finalised-block subscription (the "finalised lane").
//!   The two lanes are merged with `futures_util::stream::select`.
//!
//! Backend policy and the ReviveApi-based state read are per the
//! 2026-06-04 subxt-commitment amendment.
//!
//! This module's chain-free decisions are tested from
//! `on-chain-client/tests`, not from a `#[cfg(test)]` module here: this
//! unit's gate reads its evidence from `test_paths`
//! (`on-chain-client/tests`), so an annotated test written in `src` would be
//! read by no gate. `internals::{solidity_mapping_slot, increment_slot}` are
//! covered by `tests/storage_slot_layout.rs`, `internals::scan_step` by
//! `tests/best_lane_reorg_rule.rs`, and `internals::log_is_ours` by
//! `tests/log_ownership.rs` and `tests/contract_address_filter.rs`. All four
//! are reached through the feature-gated `crate::test_support` re-export.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::pin::Pin;

use futures_core::Stream;
use futures_util::StreamExt;
use subxt::OnlineClient;
use subxt::config::PolkadotConfig;
use subxt::dynamic::Value;

use crate::client::internals::{
    ScanStep, increment_slot, log_is_ours, scan_step, solidity_mapping_slot,
};
use crate::decode::{Decoder, DecodeError, dispatch};
use crate::state::{BlockHash, BlockRef, Event, OrgState, SubscribedEvent};
use crate::types::OrgAdmin;

/// Errors produced by `OrgRegistryClient`. Variant names are stable;
/// inner strings are advisory.
#[derive(Debug)]
pub enum ClientError {
    /// subxt RPC / metadata / decode error from the underlying client.
    Subxt(String),
    /// `state_getRuntimeVersion` returned a `spec_version` for which no
    /// compiled-in decoder exists.
    UnsupportedRuntime { spec_version: u32 },
    /// Our decoder rejected an on-chain payload.
    Decode(DecodeError),
}

impl core::fmt::Display for ClientError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Subxt(m) => write!(f, "subxt: {m}"),
            Self::UnsupportedRuntime { spec_version } => {
                write!(f, "no decoder for runtime spec_version {spec_version}")
            }
            Self::Decode(e) => write!(f, "decode: {e}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ClientError {}

impl From<DecodeError> for ClientError {
    fn from(e: DecodeError) -> Self {
        Self::Decode(e)
    }
}

/// Stream of decoded notifications from [`OrgRegistryClient::subscribe`].
/// Boxed so callers don't have to thread the concrete subxt subscription
/// type through their code.
pub type SubscribedEventStream =
    Pin<Box<dyn Stream<Item = Result<SubscribedEvent, ClientError>> + Send>>;

/// Read-only client for one deployed `OrgRegistry` contract. Cheap to
/// `Clone` — subxt's `OnlineClient` is reference-counted.
#[derive(Clone)]
pub struct OrgRegistryClient {
    api: OnlineClient<PolkadotConfig>,
    /// H160 of the deployed `OrgRegistry` contract. Events from other
    /// contract addresses are filtered out before reaching subscribers.
    contract: [u8; 20],
    /// Decoder pinned at construction. We resolve once via the runtime
    /// version reported by metadata; if the runtime upgrades mid-session
    /// the client should be reconstructed.
    decoder: &'static dyn Decoder,
    spec_version: u32,
}

impl OrgRegistryClient {
    /// Wrap an already-connected subxt client for the given contract
    /// address. Resolves the runtime version through metadata and pins
    /// the matching decoder — fails fast with `UnsupportedRuntime` if
    /// the version isn't in `decode::dispatch`.
    ///
    /// Backend choice is the caller's: tests use an explicit
    /// `LegacyBackend` (chopsticks fully supports the legacy RPC group
    /// but only part of the v2 `chainHead`/`transactionWatch` groups,
    /// which silently breaks subxt's default `CombinedBackend`);
    /// production uses `ChainHeadBackend` or the smoldot light client.
    pub async fn from_client(
        api: OnlineClient<PolkadotConfig>,
        contract: [u8; 20],
    ) -> Result<Self, ClientError> {
        let at = api
            .at_current_block()
            .await
            .map_err(|e| ClientError::Subxt(format!("at_current_block: {e}")))?;
        let spec_version = at.spec_version();
        let decoder = dispatch::for_runtime(spec_version)
            .map_err(|_| ClientError::UnsupportedRuntime { spec_version })?;

        Ok(Self {
            api,
            contract,
            decoder,
            spec_version,
        })
    }

    /// Runtime spec_version observed at the time `from_client` ran. Useful
    /// for confirming we're talking to the expected runtime; the
    /// decoder pinned at construction is for this version.
    pub fn spec_version(&self) -> u32 {
        self.spec_version
    }

    /// Read the on-chain state for `admin` at the given block (or the
    /// latest finalised block if `at` is `None`). Returns `Ok(None)`
    /// for a never-written slot.
    ///
    /// Layout:
    /// - Solidity slot S for `orgs[admin]` =
    ///   `keccak256(abi.encode(uint256(admin_padded), uint256(0)))`.
    /// - Slots `S`, `S+1`, `S+2` hold `rootHash`, `orgPubKey`, `epoch`.
    /// - Each is read from pallet-revive's per-contract child trie via
    ///   the `ReviveApi::get_storage` runtime API and concatenated into
    ///   the 96-byte blob the decoder consumes.
    pub async fn get_org_state(
        &self,
        admin: OrgAdmin,
        at: Option<BlockHash>,
    ) -> Result<Option<OrgState>, ClientError> {
        // Resolve the target block once so all three slot reads are
        // guaranteed to be consistent — no torn-reads across advancing
        // finalised heads, and 1 round-trip instead of 3.
        let at_block = match at {
            Some(h) => self
                .api
                .at_block(subxt_block_ref(h))
                .await
                .map_err(|e| ClientError::Subxt(format!("at_block: {e}")))?,
            None => self
                .api
                .at_current_block()
                .await
                .map_err(|e| ClientError::Subxt(format!("at_current_block: {e}")))?,
        };

        let base_slot = solidity_mapping_slot(admin, 0);

        let mut blob = Vec::with_capacity(96);
        for offset in 0u8..3u8 {
            let mut slot = base_slot;
            increment_slot(&mut slot, offset);
            match self.read_contract_slot(&slot, &at_block).await? {
                Some(bytes) if bytes.len() == 32 => blob.extend_from_slice(&bytes),
                // Slot absent → treat the whole org as never-initialised.
                // The contract writes all three slots atomically on
                // genesis so a partial-read state shouldn't appear.
                None => return Ok(None),
                Some(other) => {
                    return Err(ClientError::Decode(DecodeError::StorageLengthMismatch {
                        expected: 32,
                        actual: other.len(),
                    }));
                }
            }
        }

        let state = self.decoder.decode_org_state(&blob)?;
        Ok(Some(state))
    }

    /// The account pallet-revive maps `admin` to (`Revive.OriginalAccount`)
    /// at the latest finalised block (REQ-8p2veg): `Ok(None)` when the H160
    /// is not mapped, `ClientError::Decode` when the stored value is not a
    /// 32-byte account. The decoding is LLR-m3tjvp's.
    ///
    /// SDD-3b8zef's shell; exercised only against a chain.
    #[cfg(feature = "write")]
    pub async fn original_account(
        &self,
        admin: OrgAdmin,
    ) -> Result<Option<crate::write::AccountId>, ClientError> {
        let key = Value::from_bytes(admin.0.as_slice());
        match self.fetch_storage_raw("Revive", "OriginalAccount", key).await? {
            None => Ok(None),
            Some(bytes) => crate::write::signatory_set::decode_original_account(&bytes)
                .map(Some)
                .map_err(signatory_set_decode_error),
        }
    }

    /// `account`'s proxy delegates (`Proxy.Proxies`) at the latest finalised
    /// block, in stored order (REQ-v8jczx): empty when the account has no
    /// proxies, `ClientError::Decode` when the stored value does not decode
    /// as the runtime's proxy definitions. The decoding is LLR-a9bb7b's.
    ///
    /// SDD-3b8zef's shell; exercised only against a chain.
    #[cfg(feature = "write")]
    pub async fn proxy_delegates(
        &self,
        account: crate::write::AccountId,
    ) -> Result<Vec<crate::write::AccountId>, ClientError> {
        let key = Value::from_bytes(account.0.as_slice());
        match self.fetch_storage_raw("Proxy", "Proxies", key).await? {
            None => Ok(Vec::new()),
            Some(bytes) => crate::write::signatory_set::decode_proxy_delegates(&bytes)
                .map_err(signatory_set_decode_error),
        }
    }

    /// The raw value of a single-key storage map entry at the latest
    /// finalised block — the block `get_org_state` reads with `at = None` —
    /// or `None` when no value is stored. The key is fetched raw, not through
    /// subxt's typed fetch, so an entry's metadata default never stands in
    /// for an absent value.
    #[cfg(feature = "write")]
    async fn fetch_storage_raw(
        &self,
        pallet: &str,
        entry: &str,
        key: Value,
    ) -> Result<Option<Vec<u8>>, ClientError> {
        let at_block = self
            .api
            .at_current_block()
            .await
            .map_err(|e| ClientError::Subxt(format!("at_current_block: {e}")))?;
        let storage = at_block.storage();
        let address = subxt::dynamic::storage::<Vec<Value>, Value>(pallet, entry);
        let key_bytes = storage
            .entry(address)
            .and_then(|storage_entry| storage_entry.fetch_key(alloc::vec![key]))
            .map_err(|e| ClientError::Subxt(format!("{pallet}.{entry} key: {e}")))?;
        match storage.fetch_raw(key_bytes).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(subxt::error::StorageError::NoValueFound) => Ok(None),
            Err(e) => Err(ClientError::Subxt(format!("{pallet}.{entry}: {e}"))),
        }
    }

    async fn read_contract_slot(
        &self,
        slot: &[u8; 32],
        at_block: &subxt::OnlineClientAtBlock<PolkadotConfig>,
    ) -> Result<Option<Vec<u8>>, ClientError> {
        // pallet-revive keeps contract slot values in a per-contract
        // child trie — there is no runtime storage map to read. The
        // supported read path is the `ReviveApi::get_storage(address,
        // key)` runtime API (verified present on Paseo AH 2_002_002):
        // it returns Ok(Some(bytes)) / Ok(None) for an existing
        // contract, and Err(ContractAccessError) if `address` has no
        // contract. We surface that Err as ClientError::Subxt — callers
        // construct the client with a known-deployed contract address,
        // so it indicates a wiring bug, not an empty slot.

        let args = (
            Value::from_bytes(self.contract.as_slice()),
            Value::from_bytes(slot.as_slice()),
        );
        let payload = subxt::dynamic::runtime_api_call::<_, Result<Option<Vec<u8>>, Value>>(
            "ReviveApi",
            "get_storage",
            args,
        );
        let result = at_block
            .runtime_apis()
            .call(payload)
            .await
            .map_err(|e| ClientError::Subxt(format!("ReviveApi::get_storage: {e}")))?;
        match result {
            Ok(maybe_bytes) => Ok(maybe_bytes),
            Err(access_error) => Err(ClientError::Subxt(format!(
                "ContractAccessError from ReviveApi::get_storage: {access_error:?}"
            ))),
        }
    }

    /// Subscribe to OrgRegistry events. Yields a `SubscribedEvent` per
    /// matching contract event seen in best blocks (`BestBlockEvent`)
    /// and finalised blocks (`FinalisedEvent`), plus a `Reorged`
    /// notification whenever a previously-best block is discarded.
    ///
    /// Two lanes are merged with `futures_util::stream::select`:
    ///
    /// - **Best lane** (`stream_best_blocks`, legacy
    ///   `chain_subscribe_new_heads`): emits `BestBlockEvent`s plus
    ///   `Reorged` on best-head reorgs (see below).
    /// - **Finalised lane** (`stream_blocks`, the finalised stream):
    ///   emits `FinalisedEvent`s. subxt gap-fills this stream internally
    ///   (`subscribe_to_block_headers_filling_in_gaps`), so no extra
    ///   backfill machinery is needed here.
    ///
    /// The two lanes interleave arbitrarily and the SAME on-chain event
    /// will generally arrive on BOTH lanes (once as best, once as
    /// finalised); consumers should treat `BestBlockEvent` as optimistic
    /// and `FinalisedEvent` as committed.
    ///
    /// Reorg semantics consumers MUST account for:
    ///
    /// - The finalised lane is **monotonic by block number** (subxt's
    ///   internal gap-filler skips heights at or below the last one it
    ///   emitted). After a finalised-height rewind (a dev-tool-only
    ///   situation — e.g. chopsticks `dev_setHead`), it does NOT
    ///   re-emit replacement blocks at already-passed heights, and there
    ///   is no "finality reverted" notification. Rely on the best lane's
    ///   `Reorged` for reorg detection.
    /// - After a reorg, the best lane may re-emit **duplicate
    ///   `BestBlockEvent`s for unchanged ancestor heights** (the rewind
    ///   notification collapses the backfill range onto an
    ///   already-processed height). Consumers should de-duplicate or
    ///   filter by payload rather than assume exactly-once delivery.
    pub async fn subscribe(
        &self,
        admin_filter: Option<OrgAdmin>,
    ) -> Result<SubscribedEventStream, ClientError> {
        let best = self.best_lane(admin_filter).await?;
        let finalised = self.finalised_lane(admin_filter).await?;
        let merged = futures_util::stream::select(best, finalised);

        let boxed: Pin<Box<dyn Stream<Item = Result<SubscribedEvent, ClientError>> + Send>> =
            Box::pin(merged);
        Ok(boxed)
    }

    /// Best-block lane: see `subscribe`. Rides `chain_subscribe_new_heads`
    /// (one Block per *best-head* notification — NOT every imported block),
    /// so two things are layered on top:
    ///
    /// 1. **Gap-fill.** When blocks are produced faster than the
    ///    notification round-trips (e.g. two `dev_newBlock` calls
    ///    back-to-back on a chopsticks fork), intermediate best blocks are
    ///    silently skipped. We track the last best block we processed and,
    ///    on each new head, also fetch and emit events for every skipped
    ///    ancestor in `last.number+1 ..= new`, in ascending (chain) order.
    ///    Backfilled heights are looked up by number via `api.at_block(n)`,
    ///    which ALWAYS resolves the canonical best block at that height —
    ///    so it can never observe a discarded sibling.
    ///    Empirically pinned by `two_orgs_one_watcher` (Task 7): two
    ///    genesis updates mined in consecutive blocks must both reach the
    ///    watcher.
    ///
    /// 2. **Reorg detection via parent-hash tracking.** Because the
    ///    by-number gap-fill can never see a discarded block, reorgs are
    ///    detected from the head notifications themselves. We carry the
    ///    full previous best head as a `BlockRef` (hash+number). On a new
    ///    head B (number `n`, hash `h`, parent `p`):
    ///    - if `h == last.hash` → already processed, skip (dedup); a
    ///      depth-1 reorg would otherwise re-emit the replacement block's
    ///      events twice.
    ///    - if `last` is `Some` AND (`n <= last.number` OR (`n ==
    ///      last.number + 1` AND `p != last.hash`)) → the previous best was
    ///      NOT B's parent, so it was discarded → emit
    ///      `Reorged { discarded: last }` BEFORE B's events. For jumps
    ///      `n > last.number + 1` the intermediates come from canonical
    ///      backfill and no reorg signal is derivable — acceptable for the
    ///      manual-mining scenarios.
    ///
    /// `chain_subscribe_new_heads` does NOT replay the current head on
    /// subscribe — it only pushes FUTURE heads. So the scan is seeded from
    /// a baseline captured *before* subscribing. `at_current_block()`
    /// resolves to the latest *finalised* block (a conservative lower
    /// bound on the best tip, ≤ best); the first head's backfill
    /// `seed.number+1 ..= head` may replay best blocks imported before
    /// `subscribe()` was called. Harmless for this best-effort lane
    /// because callers subscribe before triggering the events they care
    /// about; the seed's hash is the finalised hash so it never spuriously
    /// matches a best head for dedup/reorg purposes.
    async fn best_lane(
        &self,
        admin_filter: Option<OrgAdmin>,
    ) -> Result<SubscribedEventStream, ClientError> {
        let contract = self.contract;
        let decoder = self.decoder;
        let api = self.api.clone();

        let seed_block = self
            .api
            .at_current_block()
            .await
            .map_err(|e| ClientError::Subxt(format!("at_current_block: {e}")))?;
        let seed = BlockRef {
            hash: BlockHash(seed_block.block_ref().hash().0),
            number: seed_block.block_number(),
        };
        let sub = self
            .api
            .stream_best_blocks()
            .await
            .map_err(|e| ClientError::Subxt(format!("stream_best_blocks: {e}")))?;

        // Scan state carries the most recent best head actually processed
        // as a `BlockRef` (hash+number) — needed to detect reorgs the
        // by-number backfill can't see. Each step emits an optional
        // `Reorged` plus an inclusive backfill range `from ..= to`.
        let stream = sub
            .scan(Some(seed), move |last: &mut Option<BlockRef>, block_res| {
                let step: ScanStep = match block_res {
                    Err(e) => ScanStep::Error(format!("block: {e}")),
                    Ok(block) => scan_step(
                        last,
                        block.number(),
                        BlockHash(block.hash().0),
                        BlockHash(block.header().parent_hash.0),
                    ),
                };
                core::future::ready(Some(step))
            })
            .then(move |step| {
                let api = api.clone();
                async move {
                    let mut out: Vec<Result<SubscribedEvent, ClientError>> = Vec::new();
                    match step {
                        ScanStep::Skip => {}
                        ScanStep::Error(msg) => out.push(Err(ClientError::Subxt(msg))),
                        ScanStep::Block { reorged, from, to } => {
                            if let Some(discarded) = reorged {
                                out.push(Ok(SubscribedEvent::Reorged { discarded }));
                            }
                            // NOTE: this span is unbounded. The seed is the
                            // finalised head, which on a live chain can lag
                            // the best tip by many blocks — the FIRST
                            // notification then backfills that whole span,
                            // one `at_block` round-trip per height. Fine for
                            // manual-mining test forks; a long-lived watcher
                            // on a live chain may want a cap + resync
                            // strategy before relying on this lane.
                            for number in from..=to {
                                match events_in_best_block(
                                    &api,
                                    contract,
                                    decoder,
                                    admin_filter,
                                    number,
                                )
                                .await
                                {
                                    Ok(mut items) => out.append(&mut items),
                                    Err(e) => out.push(Err(e)),
                                }
                            }
                        }
                    }
                    out
                }
            })
            .flat_map(futures_util::stream::iter);

        let boxed: Pin<Box<dyn Stream<Item = Result<SubscribedEvent, ClientError>> + Send>> =
            Box::pin(stream);
        Ok(boxed)
    }

    /// Finalised lane: see `subscribe`. Rides subxt's `stream_blocks`
    /// (finalised stream), which already gap-fills internally, so each
    /// finalised block is decoded directly and its matching contract
    /// events lifted into `FinalisedEvent`.
    async fn finalised_lane(
        &self,
        admin_filter: Option<OrgAdmin>,
    ) -> Result<SubscribedEventStream, ClientError> {
        let contract = self.contract;
        let decoder = self.decoder;

        let sub = self
            .api
            .stream_blocks()
            .await
            .map_err(|e| ClientError::Subxt(format!("stream_blocks: {e}")))?;

        let stream = sub
            .then(move |block_res| async move {
                let mut out: Vec<Result<SubscribedEvent, ClientError>> = Vec::new();
                let block = match block_res {
                    Ok(b) => b,
                    Err(e) => {
                        out.push(Err(ClientError::Subxt(format!("finalised block: {e}"))));
                        return out;
                    }
                };
                let at_block = match block.at().await {
                    Ok(a) => a,
                    Err(e) => {
                        out.push(Err(ClientError::Subxt(format!("block.at(): {e}"))));
                        return out;
                    }
                };
                match decode_contract_events(
                    &at_block,
                    contract,
                    decoder,
                    admin_filter,
                    |event, at| SubscribedEvent::FinalisedEvent { event, at },
                )
                .await
                {
                    Ok(mut items) => out.append(&mut items),
                    Err(e) => out.push(Err(e)),
                }
                out
            })
            .flat_map(futures_util::stream::iter);

        let boxed: Pin<Box<dyn Stream<Item = Result<SubscribedEvent, ClientError>> + Send>> =
            Box::pin(stream);
        Ok(boxed)
    }
}

/// The chain-free helpers this module is built on: the best lane's step
/// decision and the Solidity slot arithmetic. They live in their own module
/// so they can be declared `pub` and re-exported by `lib.rs`'s
/// feature-gated `test_support` module — a `pub(crate)` item cannot be
/// re-exported (E0364), and a `pub` item in a non-public module is not
/// reachable from outside the crate, so the default build's public API is
/// unchanged either way.
pub(crate) mod internals {
    use alloc::string::String;

    use tiny_keccak::{Hasher, Keccak};

    use super::event_admin;
    use crate::state::{BlockHash, BlockRef, EmittedEvent};
    use crate::types::OrgAdmin;

    /// Whether a decoded log belongs to the Organisation this reader watches:
    /// the contract that emitted it must be the one the reader was constructed
    /// for, and — when a filter is set — the event's admin must be the one
    /// filtered on. Extracted from `decode_contract_events` so the decision
    /// can be tested without a chain: the comparison it replaces was
    /// unreachable from every gated test, which is HAZ-werm85's residual and
    /// was measured, not assumed.
    ///
    /// The contract check comes first and is decisive: any contract on the
    /// chain can emit a log carrying our signature hash and a victim's
    /// indexed admin, so a matching admin must never rescue a foreign log.
    pub fn log_is_ours(
        emitted: &EmittedEvent,
        configured_contract: &[u8; 20],
        admin_filter: Option<OrgAdmin>,
    ) -> bool {
        if emitted.contract != *configured_contract {
            return false;
        }
        match admin_filter {
            None => true,
            Some(wanted) => *event_admin(&emitted.event) == wanted,
        }
    }

    /// Per-step output of the best-lane scan. `Skip` yields nothing (dedup),
    /// `Error` surfaces an upstream block error, `Block` carries an optional
    /// reorg notification plus the inclusive backfill range `from ..= to`.
    #[derive(Debug, PartialEq, Eq)]
    pub enum ScanStep {
        Skip,
        Error(String),
        Block {
            reorged: Option<BlockRef>,
            from: u64,
            to: u64,
        },
    }

    /// The best lane's per-notification decision, extracted from `best_lane`'s
    /// `scan` closure so it can be tested without a chain. Given the last best
    /// head actually processed and the new head's (number, hash, parent),
    /// returns what to emit and advances `last`. Pure apart from that advance.
    pub fn scan_step(
        last: &mut Option<BlockRef>,
        n: u64,
        h: BlockHash,
        p: BlockHash,
    ) -> ScanStep {
        match *last {
            // Dedup: same hash as the last head we processed — emit nothing,
            // don't advance.
            Some(prev) if prev.hash == h => ScanStep::Skip,
            _ => {
                let reorged = match *last {
                    Some(prev) if n <= prev.number || (n == prev.number + 1 && p != prev.hash) => {
                        Some(prev)
                    }
                    _ => None,
                };
                let from = match *last {
                    Some(prev) if n > prev.number => prev.number + 1,
                    _ => n,
                };
                *last = Some(BlockRef { hash: h, number: n });
                ScanStep::Block { reorged, from, to: n }
            }
        }
    }

    /// Compute the Solidity slot key for `orgs[admin]` where `orgs` is the
    /// `mapping(address => OrgState)` declared at slot 0. Formula:
    /// `keccak256(abi.encode(uint256(admin_padded), uint256(map_slot)))`.
    pub fn solidity_mapping_slot(admin: OrgAdmin, map_slot: u64) -> [u8; 32] {
        let mut buf = [0u8; 64];
        // address is left-padded into bytes [12..32].
        buf[12..32].copy_from_slice(&admin.0);
        // map slot is uint256 big-endian into bytes [32..64]'s low 8.
        buf[56..64].copy_from_slice(&map_slot.to_be_bytes());
        let mut hasher = Keccak::v256();
        hasher.update(&buf);
        let mut out = [0u8; 32];
        hasher.finalize(&mut out);
        out
    }

    /// Increment a 32-byte big-endian slot id by `offset`. Solidity stores
    /// struct fields at consecutive slots, so `S+1`, `S+2` etc. are
    /// computed this way. Wrapping is unreachable in practice (it would
    /// require a 2^256 mapping).
    pub fn increment_slot(slot: &mut [u8; 32], offset: u8) {
        let mut carry = u16::from(offset);
        for byte in slot.iter_mut().rev() {
            let sum = u16::from(*byte) + carry;
            *byte = sum as u8;
            carry = sum >> 8;
            if carry == 0 {
                break;
            }
        }
    }
}

/// Fetch the events of best block `number`, decode every
/// `Revive::ContractEmitted` matching `contract` (and `admin_filter`, if
/// set), and lift each into a `SubscribedEvent::BestBlockEvent`. Used by
/// `subscribe`'s gap-fill: looking the block up by number resolves to the
/// canonical best block at that height.
async fn events_in_best_block(
    api: &OnlineClient<PolkadotConfig>,
    contract: [u8; 20],
    decoder: &'static dyn Decoder,
    admin_filter: Option<OrgAdmin>,
    number: u64,
) -> Result<Vec<Result<SubscribedEvent, ClientError>>, ClientError> {
    let at_block = api
        .at_block(number)
        .await
        .map_err(|e| ClientError::Subxt(format!("at_block({number}): {e}")))?;
    decode_contract_events(&at_block, contract, decoder, admin_filter, |event, at| {
        SubscribedEvent::BestBlockEvent { event, at }
    })
    .await
}

/// Shared per-block decode used by both subscription lanes. Resolves the
/// block's `BlockRef`, fetches its events, decodes every
/// `Revive::ContractEmitted` matching `contract` (and `admin_filter`, if
/// set), and lifts each into a `SubscribedEvent` via `wrap` — the only
/// difference between the best lane (`BestBlockEvent`) and the finalised
/// lane (`FinalisedEvent`).
async fn decode_contract_events(
    at_block: &subxt::OnlineClientAtBlock<PolkadotConfig>,
    contract: [u8; 20],
    decoder: &'static dyn Decoder,
    admin_filter: Option<OrgAdmin>,
    wrap: impl Fn(Event, BlockRef) -> SubscribedEvent,
) -> Result<Vec<Result<SubscribedEvent, ClientError>>, ClientError> {
    let block_ref = BlockRef {
        hash: BlockHash(at_block.block_ref().hash().0),
        number: at_block.block_number(),
    };
    let evs = at_block
        .events()
        .fetch()
        .await
        .map_err(|e| ClientError::Subxt(format!("events.fetch: {e}")))?;

    let mut out: Vec<Result<SubscribedEvent, ClientError>> = Vec::new();
    for ev in evs.iter() {
        let ev = match ev {
            Ok(e) => e,
            Err(e) => {
                out.push(Err(ClientError::Subxt(format!("event iter: {e}"))));
                continue;
            }
        };
        if ev.pallet_name() != "Revive" {
            continue;
        }
        if ev.event_name() != "ContractEmitted" {
            continue;
        }
        // Re-encode → decode via parse_revive_event so we exercise the
        // same decoder the fixture tests pin, even though subxt could
        // surface fields directly via the dynamic Value API.
        let payload = ev.field_bytes();
        let emitted = match decoder.parse_revive_event(payload) {
            Ok(Some(e)) => e,
            Ok(None) => continue,
            Err(e) => {
                out.push(Err(ClientError::Decode(e)));
                continue;
            }
        };
        // The decisive check: a log is this organisation's only if the
        // contract that emitted it is the one this reader was constructed
        // for. Any contract can emit a log carrying our signature hash and a
        // victim's indexed admin (HAZ-werm85), so pallet+variant filtering is
        // not enough. The decision lives in `internals::log_is_ours` so it is
        // reachable by a test without a chain; behaviour is unchanged — the
        // contract check already preceded the admin check, and `None` already
        // meant "no filter".
        if !log_is_ours(&emitted, &contract, admin_filter) {
            continue;
        }
        out.push(Ok(wrap(emitted.event, block_ref)));
    }
    Ok(out)
}

/// A signatory-set value that did not decode is a decode error of the read,
/// reported, never read as "not mapped" or "no delegates".
#[cfg(feature = "write")]
fn signatory_set_decode_error(error: crate::write::signatory_set::SignatorySetError) -> ClientError {
    ClientError::Decode(DecodeError::Scale(format!("{error}")))
}

fn subxt_block_ref(h: BlockHash) -> subxt::utils::H256 {
    subxt::utils::H256(h.0)
}

/// The admin an `OrgRegistry` event carries, borrowed. One match over the two
/// event shapes, so `log_is_ours` does not have to repeat it.
fn event_admin(ev: &Event) -> &OrgAdmin {
    match ev {
        Event::Genesis { admin, .. } => admin,
        Event::Update { admin, .. } => admin,
    }
}
