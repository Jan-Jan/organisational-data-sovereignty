//! Adapts on-chain-client's OrgRegistryClient to org-node's synchronous
//! ChainReader. Async fetch refreshes a cached snapshot; the sync trait method
//! returns that snapshot so verify-against-chain stays synchronous.
//!
//! The production receive path does not use this reader: `receive_and_verify`
//! reads the chain itself and hands verification a one-shot adapter.
#![cfg(feature = "chain")]
use std::sync::Mutex;

use on_chain_client::{OrgAdmin, OrgRegistryClient};

use crate::chain::{ChainReader, OrgState};
use crate::error::OrgNodeError;
use crate::ids::OrgId;

/// Parses an Organisation state read from the chain into org-node's types,
/// through the one edge that refuses an invalid Organisation public key:
/// bytes that are not a valid X25519 public key are refused with
/// `InvalidOrgPublicKey` (REQ-8jb4ny), and the cache fails closed
/// (LLR-mmdu38's cache clause).
pub fn org_state_from_chain(s: on_chain_client::OrgState) -> Result<OrgState, OrgNodeError> {
    OrgState::from_chain(s.root_hash.0, s.org_pub_key.0, s.epoch.0)
}

/// The cached Organisation state behind [`OnChainReader`], pinned to one
/// Organisation. Split out of the reader so the cache update can be exercised
/// without a live chain: `refresh()` fetches, then hands the result here.
pub struct OrgStateCache {
    org_id: OrgId,
    cached: Mutex<Option<OrgState>>,
}

impl OrgStateCache {
    /// An empty cache: `get_org_state` returns `Ok(None)` until a store.
    pub fn new(org_id: OrgId) -> Self {
        Self { org_id, cached: Mutex::new(None) }
    }

    /// Parses a state fetched from the chain and caches it (LLR-mmdu38).
    /// A state refused at parse fails closed: the cache is cleared, so
    /// `get_org_state` returns `None` until a refresh succeeds, and the parse
    /// error is still returned. The superseded state is never served.
    pub fn store_fetched(&self, fetched: Option<on_chain_client::OrgState>) -> Result<(), String> {
        let parsed = fetched.map(org_state_from_chain).transpose().map_err(|e| e.to_string());
        // Lock poisoning is unreachable here (no panics while held); map it to a string.
        let mut cached = self.cached.lock().map_err(|_| "cache lock poisoned".to_string())?;
        *cached = parsed.as_ref().ok().copied().flatten();
        parsed.map(|_| ())
    }
}

impl ChainReader for OrgStateCache {
    fn get_org_state(&self, requested: &OrgId) -> Result<Option<OrgState>, String> {
        if requested != &self.org_id {
            return Ok(None); // this cache is pinned to one org
        }
        Ok(*self.cached.lock().map_err(|_| "cache lock poisoned".to_string())?)
    }
}

/// A ChainReader backed by a live OrgRegistryClient, with a cached snapshot.
pub struct OnChainReader {
    client: OrgRegistryClient,
    org_id: OrgId,
    cache: OrgStateCache,
}

impl OnChainReader {
    pub fn new(client: OrgRegistryClient, org_id: OrgId) -> Self {
        Self { client, org_id, cache: OrgStateCache::new(org_id) }
    }

    /// Read the state for `org_id` at the latest **finalised** block and cache
    /// it. `at = None` selects that block; org-node holds on-chain-client to
    /// this as REQ-ysyu9g. Call before invoking verify-against-chain.
    pub async fn refresh(&self) -> Result<(), String> {
        let admin = OrgAdmin(*self.org_id.as_bytes());
        let fetched = self
            .client
            .get_org_state(admin, None)
            .await
            .map_err(|e| format!("{e:?}"))?;
        self.cache.store_fetched(fetched)
    }
}

impl ChainReader for OnChainReader {
    /// Reads no block: returns the snapshot of the last `refresh()`, so the
    /// root is only as fresh as the caller's refresh discipline.
    ///
    /// Returns `Ok(None)` both when the org slot is genuinely empty on-chain
    /// AND when `refresh()` has not yet been called (initial state). This fails
    /// closed — callers MUST call `refresh().await` before relying on this.
    fn get_org_state(&self, requested: &OrgId) -> Result<Option<OrgState>, String> {
        self.cache.get_org_state(requested)
    }
}
