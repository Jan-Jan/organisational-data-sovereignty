//! The chain seen as a read-only oracle. Phase 2.1 uses MockChain; a later
//! phase implements ChainReader over on-chain-client. OrgState mirrors the
//! OrgRegistry slot: (rootHash, orgPubKey, epoch). See spec §4.4.
use std::collections::HashMap;

use org_members::RootHash;

use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::types::{Epoch, OrgPublicKey};

/// The on-chain state of one org, as stored in the OrgRegistry slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrgState {
    pub root_hash: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub epoch: Epoch,
}

impl OrgState {
    /// The edge where an Organisation state read from the chain enters the
    /// node: its Organisation public key must be a valid X25519 public key,
    /// or the state is refused (REQ-8jb4ny).
    pub fn from_chain(root_hash: [u8; 32], org_pub_key: [u8; 32], epoch: u64) -> Result<Self, OrgNodeError> {
        Ok(Self {
            root_hash: RootHash::new(root_hash),
            org_pub_key: OrgPublicKey::parse(&org_pub_key)?,
            epoch: Epoch::new(epoch),
        })
    }
}

/// Read-only access to on-chain org state. The trusted-root oracle: the root
/// returned here MUST come from a path the delta sender does not control.
pub trait ChainReader {
    /// Returns the current OrgState for `org_id`, or None if the slot is empty.
    fn get_org_state(&self, org_id: &OrgId) -> Result<Option<OrgState>, String>;
}

/// In-memory ChainReader for tests. `set` simulates the contract's `update()`
/// moving a slot.
#[derive(Default, Clone)]
pub struct MockChain {
    slots: HashMap<OrgId, OrgState>,
}

impl MockChain {
    pub fn new() -> Self {
        Self::default()
    }

    /// Simulate an on-chain update() landing for `org_id`.
    pub fn set(&mut self, org_id: OrgId, state: OrgState) {
        self.slots.insert(org_id, state);
    }
}

impl ChainReader for MockChain {
    fn get_org_state(&self, org_id: &OrgId) -> Result<Option<OrgState>, String> {
        Ok(self.slots.get(org_id).copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn mock_chain_returns_set_state() {
        let mut chain = MockChain::new();
        let org = OrgId::new([1u8; 20]);
        assert_eq!(chain.get_org_state(&org).unwrap(), None);

        let state = OrgState { root_hash: RootHash::new([9u8; 32]), org_pub_key: crate::test_fixtures::org_public_key(), epoch: Epoch::new(1) };
        chain.set(org, state);
        assert_eq!(chain.get_org_state(&org).unwrap(), Some(state));
    }
}
