//! PersonDeviceHasher: the Person device sub-trie's own domains and sentinel.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

mod common;

use common::{hex, slots};
use person::device_hasher::{PersonDeviceHasher, PERSON_DEVICE_LEAF_KEY, PERSON_DEVICE_NODE_KEY};
use person::{compute_device_root, DeviceTrieHasher, NodeHash};

/// org-members' device domain keys and sentinel, as in tests/device_trie.rs.
struct OrgDomains;
impl DeviceTrieHasher for OrgDomains {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1";
    fn hash_device_leaf(data: &[u8]) -> NodeHash {
        NodeHash::new(blake3::keyed_hash(b"org-members::device-leaf________", data).into())
    }
    fn hash_device_node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        let mut hasher = blake3::Hasher::new_keyed(b"org-members::device-node________");
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        NodeHash::new(hasher.finalize().into())
    }
}

/// verifies: LLR-edn55h
#[test]
fn the_domain_keys_and_sentinel_are_the_documented_bytes() {
    assert_eq!(PERSON_DEVICE_LEAF_KEY, b"person::device-leaf_____________");
    assert_eq!(PERSON_DEVICE_NODE_KEY, b"person::device-node_____________");
    assert_eq!(
        PersonDeviceHasher::DEVICE_EMPTY_SENTINEL,
        b"EMPTY_SENTINEL_PERSON_DEVICE_V1"
    );
    assert_eq!(
        PersonDeviceHasher::hash_device_leaf(b"leaf"),
        NodeHash::new(blake3::keyed_hash(PERSON_DEVICE_LEAF_KEY, b"leaf").into())
    );
    let left = NodeHash::new([1; 32]);
    let right = NodeHash::new([2; 32]);
    let mut concatenated = [1u8; 64];
    concatenated[32..].copy_from_slice(&[2; 32]);
    assert_eq!(
        PersonDeviceHasher::hash_device_node(&left, &right),
        NodeHash::new(blake3::keyed_hash(PERSON_DEVICE_NODE_KEY, &concatenated).into())
    );
}

/// verifies: LLR-edn55h
#[test]
fn the_root_is_the_depth_two_tree_under_the_person_domains() {
    let set = slots(&[2, 1]);
    let held = set.devices();
    let empty = PersonDeviceHasher::hash_device_leaf(PersonDeviceHasher::DEVICE_EMPTY_SENTINEL);
    let leaf0 = PersonDeviceHasher::hash_device_leaf(held[0].as_bytes());
    let leaf1 = PersonDeviceHasher::hash_device_leaf(held[1].as_bytes());
    let expected = PersonDeviceHasher::hash_device_node(
        &PersonDeviceHasher::hash_device_node(&leaf0, &leaf1),
        &PersonDeviceHasher::hash_device_node(&empty, &empty),
    );
    assert_eq!(compute_device_root::<PersonDeviceHasher>(&set), expected);
}

/// Pinned values. Computed 2026-10-06 by this test's first run against the
/// implementation in this plan (docs/plans/2026-10-06-person-definition.md).
/// verifies: LLR-edn55h
#[test]
fn the_person_device_root_is_pinned() {
    assert_eq!(
        hex(compute_device_root::<PersonDeviceHasher>(&slots(&[])).as_bytes()),
        "db36a2843de361cd8fa691c95f305e206fbcdc666e0f1d1a5e494610df4421a5"
    );
    assert_eq!(
        hex(compute_device_root::<PersonDeviceHasher>(&slots(&[1, 2])).as_bytes()),
        "6b8a4eea74c7daf62a0e6b87112f24ad1037bbfdc5e8607eaa2e217bc8e9d4a5"
    );
}

/// The same device set gives another root than org-members' domains give it,
/// the empty set included: neither the domain keys nor the sentinel are shared.
/// verifies: LLR-edn55h
#[test]
fn no_set_shares_its_root_with_org_members() {
    for seeds in [&[][..], &[1], &[1, 2], &[1, 2, 3, 4]] {
        let set = slots(seeds);
        assert_ne!(
            compute_device_root::<PersonDeviceHasher>(&set),
            compute_device_root::<OrgDomains>(&set),
            "{seeds:?}"
        );
    }
    assert_ne!(
        PersonDeviceHasher::DEVICE_EMPTY_SENTINEL,
        OrgDomains::DEVICE_EMPTY_SENTINEL
    );
}

/// Every slot set gives a root distinct from every other, the boundary sets
/// (no device, `MAX_DEVICES` devices) included.
/// verifies: LLR-edn55h
#[test]
fn different_sets_give_different_roots() {
    let sets = [
        slots(&[]),
        slots(&[1]),
        slots(&[2]),
        slots(&[1, 2]),
        slots(&[1, 2, 3]),
        slots(&[1, 2, 3, 4]),
    ];
    for (index, set) in sets.iter().enumerate() {
        for other in &sets[index + 1..] {
            assert_ne!(
                compute_device_root::<PersonDeviceHasher>(set),
                compute_device_root::<PersonDeviceHasher>(other)
            );
        }
    }
}
