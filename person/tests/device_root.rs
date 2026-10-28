//! The device root tree: a sparse Merkle tree of depth 4 over a set's keys, its
//! positions, and its hasher interface.

#![allow(clippy::expect_used, clippy::indexing_slicing)]

mod common;

use common::{device, device_position, keyed, slots, OrgMembersDomains};
use person::{
    compute_device_root, device_position_bytes, prove_device_absent_in, DeviceLeaf, DevicePublicKey, DeviceRootHasher,
    IdentityError, NodeHash, DEVICE_POSITION_KEY, DEVICE_ROOT_DEPTH,
};
use sparse_merkle_tree::TreeHasher;

/// Domains of no unit.
struct OtherDomains;
impl TreeHasher for OtherDomains {
    fn leaf(identity_hash: &NodeHash, content_hash: &NodeHash) -> NodeHash {
        keyed(b"test::other-device-leaf_________", &[identity_hash.as_bytes(), content_hash.as_bytes()])
    }
    fn node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        keyed(b"test::other-device-node_________", &[left.as_bytes(), right.as_bytes()])
    }
    fn empty_leaf() -> NodeHash {
        keyed(b"test::other-device-leaf_________", &[Self::DEVICE_EMPTY_SENTINEL])
    }
}
impl DeviceRootHasher for OtherDomains {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_TEST_OTHER_V1";
    fn device_identity_hash(key: &[u8; 32]) -> NodeHash {
        keyed(b"test::other-device-identity_____", &[key])
    }
    fn device_content_hash() -> NodeHash {
        keyed(b"test::other-device-content______", &[])
    }
}

/// The depth-4 tree by its 16 slots, folded pairwise: no code of the tree.
fn root_by_slots<H: DeviceRootHasher>(keys: &[DevicePublicKey]) -> NodeHash {
    let mut level = vec![H::empty_leaf(); 16];
    for key in keys {
        level[device_position(key)] = H::leaf(&H::device_identity_hash(key.as_bytes()), &H::device_content_hash());
    }
    while level.len() > 1 {
        level = level.chunks(2).map(|pair| H::node(&pair[0], &pair[1])).collect();
    }
    level[0]
}

/// verifies: LLR-exwx4n, REQ-9sh8qb
#[test]
fn a_position_is_the_keyed_hash_under_the_persona_domain() {
    assert_eq!(DEVICE_POSITION_KEY, b"persona::device-position________");
    let key = device(3);
    assert_eq!(device_position_bytes(&key), <[u8; 32]>::from(blake3::keyed_hash(DEVICE_POSITION_KEY, key.as_bytes())));
    assert_eq!(DEVICE_ROOT_DEPTH, 4);
}

/// verifies: LLR-8y8tqr, LLR-9mjfjn, REQ-mu3qgz, REQ-aj6x3n
///
/// The root is the classic depth-4 tree of the keys at their positions,
/// under the hasher's domains, for sets of 0 to 8 keys.
#[test]
fn the_root_is_the_depth_four_tree_of_the_keys_at_their_positions() {
    for count in 0..=8u8 {
        let set = slots(&(1..=count).collect::<Vec<_>>());
        assert_eq!(
            compute_device_root::<OrgMembersDomains>(set.devices()),
            Ok(root_by_slots::<OrgMembersDomains>(set.devices())),
            "{count} keys"
        );
    }
}

/// verifies: LLR-8y8tqr, REQ-mu3qgz
#[test]
fn equal_sets_give_equal_roots_whatever_the_order() {
    let forward = [device(1), device(2), device(3)];
    let backward = [device(3), device(1), device(2)];
    assert_eq!(compute_device_root::<OrgMembersDomains>(&forward), compute_device_root::<OrgMembersDomains>(&backward));
}

/// verifies: LLR-8y8tqr, REQ-mu3qgz
#[test]
fn different_sets_give_different_roots() {
    let sets: Vec<Vec<DevicePublicKey>> = vec![
        vec![],
        vec![device(1)],
        vec![device(2)],
        vec![device(1), device(2)],
        (1..=8).map(device).collect(),
    ];
    for (index, set) in sets.iter().enumerate() {
        for other in &sets[index + 1..] {
            assert_ne!(compute_device_root::<OrgMembersDomains>(set), compute_device_root::<OrgMembersDomains>(other));
        }
    }
}

/// verifies: LLR-8y8tqr, LLR-9mjfjn, REQ-aj6x3n
#[test]
fn the_same_set_under_other_domains_gives_another_root() {
    for set in [vec![], vec![device(1)], (1..=8).map(device).collect::<Vec<_>>()] {
        assert_ne!(compute_device_root::<OrgMembersDomains>(&set), compute_device_root::<OtherDomains>(&set));
    }
}

/// verifies: LLR-8y8tqr
///
/// Two keys of one position are refused, by the absence proof and by the
/// root; a proof from distinct keys resolves to their root.
#[test]
fn two_keys_of_one_position_are_refused() {
    let same = device(17); // 17 % 16 == 1
    assert_eq!(device_position(&same), device_position(&device(1)));
    assert_eq!(compute_device_root::<OrgMembersDomains>(&[device(1), same]), Err(IdentityError::DevicePositionTaken));
    assert_eq!(
        prove_device_absent_in::<OrgMembersDomains>(&[device(1), same], &device(3)).err(),
        Some(IdentityError::DevicePositionTaken)
    );
    let proof = prove_device_absent_in::<OrgMembersDomains>(&[device(1), device(2)], &device(3)).expect("distinct");
    let root = compute_device_root::<OrgMembersDomains>(&[device(1), device(2)]).expect("distinct");
    assert_eq!(proof.verify::<DeviceLeaf, OrgMembersDomains>(&root, &device(3)), Ok(()));
}

/// verifies: LLR-8y8tqr
///
/// The tree holds one `DeviceLeaf` per key: each key is refused as held, a
/// key not given is not, and the leaf gives its key back.
#[test]
fn the_tree_holds_one_device_leaf_per_key() {
    let devices = [device(1), device(2)];
    for index in [1, 2] {
        let key = device(index);
        assert_eq!(
            prove_device_absent_in::<OrgMembersDomains>(&devices, &key).err(),
            Some(IdentityError::DeviceStillHeld)
        );
        assert_eq!(DeviceLeaf::new(key).public_key(), &key);
    }
    assert!(prove_device_absent_in::<OrgMembersDomains>(&devices, &device(3)).is_ok());
}

/// verifies: LLR-9mjfjn
///
/// The empty leaf is the sentinel's hash under the leaf domain.
#[test]
fn the_empty_leaf_is_the_sentinel_under_the_leaf_domain() {
    assert_eq!(
        OrgMembersDomains::empty_leaf(),
        keyed(b"org-members::device-leaf________", &[OrgMembersDomains::DEVICE_EMPTY_SENTINEL])
    );
    assert_eq!(compute_device_root::<OrgMembersDomains>(&[]), Ok(root_by_slots::<OrgMembersDomains>(&[])));
}
