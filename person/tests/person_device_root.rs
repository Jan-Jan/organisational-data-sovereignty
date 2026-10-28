//! PersonDeviceRootHasher: the Person device root tree's own domains and sentinel.

#![allow(clippy::expect_used)]

mod common;

use common::{device, hex, keyed, slots};
use person::device_root_hasher::{
    PersonDeviceRootHasher, PERSON_DEVICE_CONTENT_KEY, PERSON_DEVICE_IDENTITY_KEY, PERSON_DEVICE_LEAF_KEY,
    PERSON_DEVICE_NODE_KEY,
};
use person::{compute_device_root, DeviceRootHasher, NodeHash};
use sparse_merkle_tree::TreeHasher;

/// verifies: LLR-qwp8d7, LLR-9mjfjn
#[test]
fn the_domain_keys_and_sentinel_are_the_documented_bytes() {
    assert_eq!(PERSON_DEVICE_LEAF_KEY, b"person::device-leaf_____________");
    assert_eq!(PERSON_DEVICE_NODE_KEY, b"person::device-node_____________");
    assert_eq!(PERSON_DEVICE_IDENTITY_KEY, b"person::device-identity_________");
    assert_eq!(PERSON_DEVICE_CONTENT_KEY, b"person::device-content__________");
    assert_eq!(PersonDeviceRootHasher::DEVICE_EMPTY_SENTINEL, b"EMPTY_SENTINEL_PERSON_DEVICE_V1");
    let first = NodeHash::new([1; 32]);
    let second = NodeHash::new([2; 32]);
    assert_eq!(PersonDeviceRootHasher::leaf(&first, &second), keyed(PERSON_DEVICE_LEAF_KEY, &[&[1; 32], &[2; 32]]));
    assert_eq!(PersonDeviceRootHasher::node(&first, &second), keyed(PERSON_DEVICE_NODE_KEY, &[&[1; 32], &[2; 32]]));
    assert_eq!(
        PersonDeviceRootHasher::empty_leaf(),
        keyed(PERSON_DEVICE_LEAF_KEY, &[b"EMPTY_SENTINEL_PERSON_DEVICE_V1"])
    );
    let key = device(1);
    assert_eq!(PersonDeviceRootHasher::device_identity_hash(key.as_bytes()), keyed(PERSON_DEVICE_IDENTITY_KEY, &[key.as_bytes()]));
    assert_eq!(PersonDeviceRootHasher::device_content_hash(), keyed(PERSON_DEVICE_CONTENT_KEY, &[]));
}

/// Pinned values: see "Golden values" in
/// docs/plans/2026-10-24-sparse-merkle-tree.md (T7). A change here is a
/// change of the V1 encoding (LLR-qwp8d7).
/// verifies: LLR-qwp8d7
#[test]
fn the_person_device_root_is_pinned() {
    assert_eq!(
        hex(compute_device_root::<PersonDeviceRootHasher>(slots(&[]).devices()).expect("empty").as_bytes()),
        "a01d79a58bbd2f5c628513a1c31a338d9f84a623a608dd2979be451f433fdac0"
    );
    assert_eq!(
        hex(compute_device_root::<PersonDeviceRootHasher>(slots(&[1, 2]).devices()).expect("two").as_bytes()),
        "1adf410eb24c2d605b0fff5c81928567158a9100611ce790c565e744b64f4f15"
    );
}

/// None of the four domain keys or the sentinel is one of org-members'
/// device domain keys or its device sentinel, as
/// org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md states
/// them.
/// verifies: LLR-qwp8d7
#[test]
fn no_domain_key_or_the_sentinel_is_org_members() {
    let org_members_keys: [&[u8; 32]; 4] = [
        b"org-members::device-leaf________",
        b"org-members::device-node________",
        b"org-members::device-identity____",
        b"org-members::device-content_____",
    ];
    for key in [PERSON_DEVICE_LEAF_KEY, PERSON_DEVICE_NODE_KEY, PERSON_DEVICE_IDENTITY_KEY, PERSON_DEVICE_CONTENT_KEY] {
        assert!(!org_members_keys.contains(&key), "{}", String::from_utf8_lossy(key));
    }
    assert_ne!(PersonDeviceRootHasher::DEVICE_EMPTY_SENTINEL, b"EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1");
}
