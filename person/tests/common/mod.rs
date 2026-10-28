//! Key, definition and hasher fixtures shared by the integration test
//! crates. Each test crate uses a subset, hence the `dead_code` allowance; a
//! fixture that cannot be built is a broken test, hence `expect`.
#![allow(dead_code, clippy::expect_used)]

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::SigningKey;
use person::{DevicePublicKey, DeviceRootHasher, DeviceSlots, Name, NodeHash, Person, PersonPublicKey, Surname};
use sparse_merkle_tree::TreeHasher;

/// The keyed blake3 hash of `parts`, concatenated, under `key`.
pub fn keyed(key: &[u8; 32], parts: &[&[u8]]) -> NodeHash {
    let mut hasher = blake3::Hasher::new_keyed(key);
    for part in parts {
        hasher.update(part);
    }
    NodeHash::new(hasher.finalize().into())
}

/// org-members' device domains, as that unit's design states them.
pub struct OrgMembersDomains;

impl TreeHasher for OrgMembersDomains {
    fn leaf(identity_hash: &NodeHash, content_hash: &NodeHash) -> NodeHash {
        keyed(b"org-members::device-leaf________", &[identity_hash.as_bytes(), content_hash.as_bytes()])
    }
    fn node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        keyed(b"org-members::device-node________", &[left.as_bytes(), right.as_bytes()])
    }
    fn empty_leaf() -> NodeHash {
        keyed(b"org-members::device-leaf________", &[Self::DEVICE_EMPTY_SENTINEL])
    }
}

impl DeviceRootHasher for OrgMembersDomains {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1";
    fn device_identity_hash(key: &[u8; 32]) -> NodeHash {
        keyed(b"org-members::device-identity____", &[key])
    }
    fn device_content_hash() -> NodeHash {
        keyed(b"org-members::device-content_____", &[])
    }
}

/// The 4-bit device position of `key`: the top four bits of its position
/// bytes, 0..16.
pub fn device_position(key: &DevicePublicKey) -> usize {
    usize::from(person::device_position_bytes(key)[0] >> 4)
}

/// The device key of `seed` whose 4-bit device position is `seed % 16`: the
/// first ed25519 signing key, over the seeds `blake3([seed, i])`, that lands
/// there. Seeds 1..=15 never share a position, so fixture sets never collide.
pub fn device(seed: u8) -> DevicePublicKey {
    (0u8..=u8::MAX)
        .map(|attempt| {
            let signing: [u8; 32] = blake3::hash(&[seed, attempt]).into();
            DevicePublicKey::try_from(SigningKey::from_bytes(&signing).verifying_key())
                .expect("valid key")
        })
        .find(|key| device_position(key) == usize::from(seed % 16))
        .expect("one of 256 attempts lands on the position")
}

/// The 32 bytes of `device(seed)`.
pub fn device_bytes(seed: u8) -> [u8; 32] {
    *device(seed).as_bytes()
}

/// The bytes of `device(seed)` for each seed, in increasing byte order.
pub fn sorted_device_bytes(seeds: &[u8]) -> Vec<[u8; 32]> {
    let mut keys: Vec<[u8; 32]> = seeds.iter().map(|seed| device_bytes(*seed)).collect();
    keys.sort();
    keys
}

pub fn slots(seeds: &[u8]) -> DeviceSlots {
    DeviceSlots::parse(seeds.iter().map(|seed| device(*seed)).collect()).expect("valid set")
}

pub fn person_key(seed: u8) -> PersonPublicKey {
    PersonPublicKey::parse(&MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes())
        .expect("valid X25519 key")
}

/// The Person named `name surname` holding the PersonPublicKey of `key` and
/// the devices of `seeds`.
pub fn person(name: &str, surname: &str, key: Option<u8>, seeds: &[u8]) -> Person {
    Person::new(
        Name::parse(name).expect("valid name"),
        Surname::parse(surname).expect("valid surname"),
        key.map(person_key),
        slots(seeds),
    )
    .expect("valid definition")
}

/// A device key whose 32 bytes are also a valid X25519 public key, and that
/// key: the literal copy LLR-kbhc43 refuses. Found by search, not assumed.
pub fn device_and_its_bytes_as_a_person_key() -> (DevicePublicKey, PersonPublicKey) {
    (1..=u8::MAX)
        .find_map(|seed| {
            let candidate = device(seed);
            PersonPublicKey::parse(candidate.as_bytes())
                .ok()
                .map(|key| (candidate, key))
        })
        .expect("some device key's bytes are a valid X25519 key")
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
