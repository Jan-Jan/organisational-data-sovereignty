//! The device sub-trie: shape, order independence, distinctness, domains.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use ed25519_dalek::SigningKey;
use person::{compute_device_root, DevicePublicKey, DeviceSlots, DeviceTrieHasher, NodeHash};

/// blake3 keyed by `key` over the two child hashes.
fn keyed_node_hash(key: &[u8; 32], left: &NodeHash, right: &NodeHash) -> NodeHash {
    let mut hasher = blake3::Hasher::new_keyed(key);
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    NodeHash::new(hasher.finalize().into())
}

/// org-members' exact device domain keys and empty-slot sentinel.
struct OrgDomains;
impl DeviceTrieHasher for OrgDomains {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1";
    fn hash_device_leaf(data: &[u8]) -> NodeHash {
        NodeHash::new(blake3::keyed_hash(b"org-members::device-leaf________", data).into())
    }
    fn hash_device_node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        keyed_node_hash(b"org-members::device-node________", left, right)
    }
}

struct OtherDomains;
impl DeviceTrieHasher for OtherDomains {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_TEST_OTHER_V1";
    fn hash_device_leaf(data: &[u8]) -> NodeHash {
        NodeHash::new(blake3::keyed_hash(b"test::other-device-leaf_________", data).into())
    }
    fn hash_device_node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        keyed_node_hash(b"test::other-device-node_________", left, right)
    }
}

fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
        .expect("valid key")
}

fn slots(seeds: &[u8]) -> DeviceSlots {
    DeviceSlots::parse(seeds.iter().map(|seed| device(*seed)).collect()).expect("valid set")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// verifies: LLR-6ezhw7, LLR-4vsm8d, REQ-mu3qgz
#[test]
fn the_root_is_the_documented_depth_two_tree() {
    let set = slots(&[2, 1]);
    let devices = set.devices();
    let empty = OrgDomains::hash_device_leaf(OrgDomains::DEVICE_EMPTY_SENTINEL);
    let leaf0 = OrgDomains::hash_device_leaf(devices[0].as_bytes());
    let leaf1 = OrgDomains::hash_device_leaf(devices[1].as_bytes());
    let expected = OrgDomains::hash_device_node(
        &OrgDomains::hash_device_node(&leaf0, &leaf1),
        &OrgDomains::hash_device_node(&empty, &empty),
    );
    assert_eq!(compute_device_root::<OrgDomains>(&set), expected);
}

/// The constant was taken from org-members' `device_trie` before the move:
/// `org_members::device_trie::compute_device_root::<org_members::hasher::Blake3Hasher>`
/// over the equivalent `P2pDeviceSlots` of the same two keys. It pins the
/// device root byte-identical across the move.
///
/// verifies: LLR-6ezhw7, REQ-mu3qgz
#[test]
fn the_device_root_matches_org_members_today() {
    let root = compute_device_root::<OrgDomains>(&slots(&[1, 2]));
    assert_eq!(
        hex(root.as_bytes()),
        "2f45f1f7c6362a0bff09da153031a942c2ba77cc102bb6eb923d7ee154bcd6fd"
    );
}

/// verifies: LLR-6ezhw7, REQ-mu3qgz
#[test]
fn equal_sets_give_equal_roots_whatever_the_construction_order() {
    assert_eq!(
        compute_device_root::<OrgDomains>(&slots(&[1, 2, 3])),
        compute_device_root::<OrgDomains>(&slots(&[3, 1, 2]))
    );
}

/// verifies: LLR-6ezhw7, REQ-mu3qgz
#[test]
fn different_sets_give_different_roots() {
    let sets = [
        slots(&[]),
        slots(&[1]),
        slots(&[2]),
        slots(&[1, 2]),
        slots(&[1, 2, 3, 4]),
    ];
    for (index, set) in sets.iter().enumerate() {
        for other in &sets[index + 1..] {
            assert_ne!(
                compute_device_root::<OrgDomains>(set),
                compute_device_root::<OrgDomains>(other)
            );
        }
    }
}

/// verifies: LLR-6ezhw7, LLR-4vsm8d, REQ-aj6x3n
#[test]
fn the_same_set_under_another_domain_gives_another_root() {
    for set in [slots(&[]), slots(&[1]), slots(&[1, 2, 3, 4])] {
        assert_ne!(
            compute_device_root::<OrgDomains>(&set),
            compute_device_root::<OtherDomains>(&set)
        );
    }
}

/// verifies: LLR-z99qee
#[test]
fn a_node_hash_wraps_any_32_bytes() {
    let mut bytes = [0xff; 32];
    bytes[0] = 0x01;
    bytes[1] = 0xab;
    let hash = NodeHash::from(bytes);
    assert_eq!(hash.as_bytes(), &bytes);
    assert_eq!(hash, NodeHash::new(bytes));
    assert_eq!(format!("{hash:?}"), "NodeHash(01abffff..)");
    assert_eq!(
        format!("{:?}", NodeHash::new([0; 32])),
        "NodeHash(00000000..)"
    );
}

/// verifies: LLR-z99qee
#[cfg(feature = "serde")]
#[test]
fn a_node_hash_encodes_as_its_32_bytes() {
    let hash = NodeHash::new([7; 32]);
    let encoded = postcard::to_allocvec(&hash).expect("encode");
    assert_eq!(encoded, [7u8; 32].to_vec());
    assert_eq!(
        postcard::from_bytes::<NodeHash>(&encoded).expect("decode"),
        hash
    );
}
