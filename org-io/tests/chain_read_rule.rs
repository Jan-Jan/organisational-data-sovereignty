#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The chain read hands org-node a value: present, absent, or an error —
//! never absence for a failure (LLR-rm9x4z, moved from org-node 2026-10-07).

use on_chain_client::{Epoch as ChainEpoch, OnChainRootHash, OrgPubKey, OrgState as ChainOrgState};
use org_io::chain_read::{org_state_from_chain, read_org_state};
use org_io::node::{Epoch, OrgId, OrgNodeError, OrgPrivateKey, RootHash};

fn x25519_key(seed: u8) -> [u8; 32] {
    *OrgPrivateKey::from([seed; 32]).x25519_keypair().org_public_key().unwrap().as_bytes()
}

fn chain_state(key: [u8; 32]) -> ChainOrgState {
    ChainOrgState { root_hash: OnChainRootHash([0x33; 32]), org_pub_key: OrgPubKey(key), epoch: ChainEpoch(7) }
}

struct ScriptedReader(Result<Option<ChainOrgState>, String>);

#[async_trait::async_trait]
impl org_io::chain_read::RawStateSource for ScriptedReader {
    async fn fetch(&self, _org_id: OrgId) -> Result<Option<ChainOrgState>, String> {
        self.0.clone()
    }
}

// Moved from org-node/tests/chain_read_state.rs (2026-10-07).
// verifies: LLR-rm9x4z
#[test]
fn chain_state_is_read_into_typed_values() {
    let key = x25519_key(0x11);
    let state = org_state_from_chain(chain_state(key)).unwrap();
    assert_eq!(state.root_hash, RootHash::new([0x33; 32]));
    assert_eq!(state.org_pub_key.as_bytes(), &key);
    assert_eq!(state.epoch, Epoch::new(7));
}

// Moved from org-node/tests/chain_read_state.rs (2026-10-07).
// verifies: LLR-rm9x4z
#[test]
fn a_chain_state_whose_key_org_node_refuses_is_an_error() {
    assert_eq!(org_state_from_chain(chain_state([0u8; 32])).unwrap_err(), OrgNodeError::InvalidOrgPublicKey);
}

// verifies: LLR-rm9x4z
#[tokio::test]
async fn present_absent_failed_and_refused_reads_are_four_different_answers() {
    let org = OrgId::new([1u8; 20]);
    let present = read_org_state(&ScriptedReader(Ok(Some(chain_state(x25519_key(0x11))))), org).await.unwrap();
    assert!(present.is_some());
    assert_eq!(read_org_state(&ScriptedReader(Ok(None)), org).await.unwrap(), None);
    let failed = read_org_state(&ScriptedReader(Err("registry read failed".into())), org).await.unwrap_err();
    assert_eq!(failed, OrgNodeError::Chain("get_org_state: registry read failed".into()));
    let refused = read_org_state(&ScriptedReader(Ok(Some(chain_state([0u8; 32])))), org).await.unwrap_err();
    assert_eq!(refused, OrgNodeError::InvalidOrgPublicKey);
}
