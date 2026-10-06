#![cfg(feature = "chain")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The chain read parses the Organisation state into org-node's types
//! (LLR-mmdu38, LLR-s7whrn): an Organisation public key that is not a curve
//! point is refused at the read as `InvalidKey`, before any Envelope is
//! verified under it.
//!
//! *Merged 2026-10-05 into worktree-person-shared-types.* On this branch the
//! Organisation public key is an X25519 key parsed by `person`'s rule
//! (REQ-8jb4ny, LLR-3jjgtw) and refused as `InvalidOrgPublicKey`; master's
//! Edwards-point rule and `InvalidKey` (LLR-mmdu38's parse clause) do not
//! hold here. The cases below test the X25519 rule at the read; the
//! fail-closed cache clause of LLR-mmdu38 holds unchanged.

use on_chain_client::{Epoch as ChainEpoch, OnChainRootHash, OrgPubKey, OrgState as ChainOrgState};
use org_members::RootHash;
use org_node::chain_read::org_state_from_chain;
use org_node::{Epoch, OrgNodeError, OrgPrivateKey};

/// A valid X25519 public key: the Organisation public key of a fixed secret.
fn x25519_key(seed: u8) -> [u8; 32] {
    *OrgPrivateKey::from([seed; 32]).x25519_keypair().org_public_key().unwrap().as_bytes()
}

fn chain_state(key: [u8; 32]) -> ChainOrgState {
    ChainOrgState {
        root_hash: OnChainRootHash([0x33; 32]),
        org_pub_key: OrgPubKey(key),
        epoch: ChainEpoch(7),
    }
}

// Adapted at the merge of master `1feb608`: the key is a valid X25519 key,
// where master read an ed25519 verifying key and also accepted the weak key
// [0; 32] (LLR-mmdu38's Edwards rule, which does not hold on this branch).
// verifies: LLR-s7whrn, LLR-3jjgtw, LLR-mmdu38
#[test]
fn chain_state_is_read_into_typed_values() {
    let key = x25519_key(0x11);
    let state = org_state_from_chain(chain_state(key)).unwrap();
    assert_eq!(state.root_hash, RootHash::new([0x33; 32]));
    assert_eq!(state.org_pub_key.as_bytes(), &key);
    assert_eq!(state.epoch, Epoch::new(7));
}

// Adapted at the merge of master `1feb608`: master refused y = 2 (off the
// Edwards curve) as `InvalidKey`. Under the X25519 rule u = 2 is a canonical
// twist point and is accepted (the rule `person::x25519` exports);
// the small-order u = 0 is what the read refuses, as `InvalidOrgPublicKey`.
// verifies: REQ-8jb4ny, LLR-3jjgtw, LLR-mmdu38
#[test]
fn chain_state_with_an_invalid_x25519_key_is_refused_as_invalid_org_public_key() {
    assert_eq!(
        org_state_from_chain(chain_state([0u8; 32])).unwrap_err(),
        OrgNodeError::InvalidOrgPublicKey
    );
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
    let mut epoch1 = chain_state(x25519_key(0x11));
    epoch1.epoch = ChainEpoch(1);
    cache.store_fetched(Some(epoch1)).unwrap();
    assert_eq!(cache.get_org_state(&org).unwrap().unwrap().epoch, Epoch::new(1));

    // Refused at parse on this branch: u = 0, of small order (master used
    // y = 2, off the Edwards curve, which the X25519 rule accepts).
    let mut epoch2 = chain_state([0u8; 32]);
    epoch2.epoch = ChainEpoch(2);
    assert!(cache.store_fetched(Some(epoch2)).is_err());
    assert_eq!(cache.get_org_state(&org).unwrap(), None, "a refused refresh must not leave the superseded state cached");
}
