//! The chain read (SDD-6qz9ms): read, parse through org-node, hand over a value.

use async_trait::async_trait;
use on_chain_client::{OrgAdmin, OrgRegistryClient};
use org_node::{OrgId, OrgNodeError, OrgState};

/// org-io's read seam: the state for an Organisation, parsed.
#[async_trait]
pub trait StateReader: Send + Sync {
    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError>;
}

/// The unparsed read, so the parse rule is tested without a chain.
#[async_trait]
pub trait RawStateSource: Send + Sync {
    async fn fetch(&self, org_id: OrgId) -> Result<Option<on_chain_client::OrgState>, String>;
}

/// Through org-node's one parse edge, which refuses an Organisation public key
/// that is not a valid X25519 key (org-node's rule); moved from org-node's
/// `chain_read.rs` 2026-10-07.
pub fn org_state_from_chain(state: on_chain_client::OrgState) -> Result<OrgState, OrgNodeError> {
    OrgState::from_chain(state.root_hash.0, state.org_pub_key.0, state.epoch.0)
}

/// Present, absent, or an error — a failure is never absence (LLR-rm9x4z).
pub async fn read_org_state(source: &dyn RawStateSource, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
    source
        .fetch(org_id)
        .await
        .map_err(|reason| OrgNodeError::Chain(format!("get_org_state: {reason}")))?
        .map(org_state_from_chain)
        .transpose()
}

/// The production reader (SDD-z85ux9's shell): org-node's former
/// `SubxtChainOps::read_state`. Reads at the latest finalised block
/// (`at = None`), the expectation org-io holds on-chain-client to (REQ-ysyu9g).
pub struct OnChainStateReader {
    registry: OrgRegistryClient,
}

impl OnChainStateReader {
    pub fn new(registry: OrgRegistryClient) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl RawStateSource for OnChainStateReader {
    async fn fetch(&self, org_id: OrgId) -> Result<Option<on_chain_client::OrgState>, String> {
        self.registry.get_org_state(OrgAdmin(*org_id.as_bytes()), None).await.map_err(|error| error.to_string())
    }
}

#[async_trait]
impl StateReader for OnChainStateReader {
    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        read_org_state(self, org_id).await
    }
}
