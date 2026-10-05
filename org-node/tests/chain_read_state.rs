#![cfg(feature = "chain")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The chain read parses the Organisation state into org-node's types
//! (LLR-mmdu38, LLR-s7whrn): an Organisation public key that is not a curve
//! point is refused at the read as `InvalidKey`, before any Envelope is
//! verified under it.

use on_chain_client::{Epoch as ChainEpoch, OnChainRootHash, OrgPubKey, OrgState as ChainOrgState};
use org_members::RootHash;
use org_node::chain_read::org_state_from_chain;
use org_node::{Epoch, OrgNodeError};

fn chain_state(key: [u8; 32]) -> ChainOrgState {
    ChainOrgState {
        root_hash: OnChainRootHash([0x33; 32]),
        org_pub_key: OrgPubKey(key),
        epoch: ChainEpoch(7),
    }
}

/// verifies: LLR-mmdu38, LLR-s7whrn
#[test]
fn chain_state_is_read_into_typed_values() {
    let key = *ed25519_dalek::SigningKey::from_bytes(&[0x11; 32]).verifying_key().as_bytes();
    let state = org_state_from_chain(chain_state(key)).unwrap();
    assert_eq!(state.root_hash, RootHash::new([0x33; 32]));
    assert_eq!(state.org_pub_key.as_bytes(), &key);
    assert_eq!(state.epoch, Epoch::new(7));
    // A weak key decompresses, so it is read as before (owner ruling).
    assert!(org_state_from_chain(chain_state([0u8; 32])).is_ok());
}

/// verifies: LLR-mmdu38
#[test]
fn chain_state_with_an_off_curve_key_is_refused_as_invalid_key() {
    let mut off_curve = [0u8; 32];
    off_curve[0] = 2;
    assert_eq!(org_state_from_chain(chain_state(off_curve)).unwrap_err(), OrgNodeError::InvalidKey);
}

/// A refresh whose chain state is refused at parse leaves no cached state:
/// the reader fails closed (`None`, so verification refuses as
/// `OrgNotOnChain`) rather than serving the superseded root it cached before.
/// The cache is exercised through `OrgStateCache`, the seam `OnChainReader`
/// hands each fetched state to, so no live chain is needed.
/// verifies: LLR-mmdu38
#[test]
fn refresh_refused_at_parse_clears_the_cached_state() {
    use org_node::chain_read::OrgStateCache;
    use org_node::{ChainReader, OrgId};

    let org = OrgId::new([0x44; 20]);
    let cache = OrgStateCache::new(org);
    let key = *ed25519_dalek::SigningKey::from_bytes(&[0x11; 32]).verifying_key().as_bytes();
    let mut epoch1 = chain_state(key);
    epoch1.epoch = ChainEpoch(1);
    cache.store_fetched(Some(epoch1)).unwrap();
    assert_eq!(cache.get_org_state(&org).unwrap().unwrap().epoch, Epoch::new(1));

    let mut off_curve = [0u8; 32];
    off_curve[0] = 2;
    let mut epoch2 = chain_state(off_curve);
    epoch2.epoch = ChainEpoch(2);
    assert!(cache.store_fetched(Some(epoch2)).is_err());
    assert_eq!(cache.get_org_state(&org).unwrap(), None, "a refused refresh must not leave the superseded state cached");
}
