//! Fixtures shared by the integration test crates. Each test crate uses a
//! subset, hence the `dead_code` allowance.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError};

use ed25519_dalek::SigningKey;
use org_members::types::{DevicePublicKey, MemberId, PersonPublicKey};

pub fn member_id(seed: &str) -> MemberId {
    MemberId::new(blake3::hash(seed.as_bytes()).into())
}

/// blake3 keyed by `key` over `parts` concatenated.
pub fn keyed(key: &[u8; 32], parts: &[&[u8]]) -> org_members::types::NodeHash {
    let mut hasher = blake3::Hasher::new_keyed(key);
    for part in parts {
        hasher.update(part);
    }
    org_members::types::NodeHash::new(hasher.finalize().into())
}

/// LLR-t465mb's content bytes: the handle, the member key's 32 bytes, the
/// name, the surname (each string prefixed by its u32 little-endian byte
/// length), then the device root.
pub fn member_content_bytes(leaf: &org_members::types::MemberLeaf, device_root: &[u8; 32]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let handle = leaf.handle().as_str().as_bytes();
    bytes.extend_from_slice(&(handle.len() as u32).to_le_bytes());
    bytes.extend_from_slice(handle);
    bytes.extend_from_slice(leaf.p2p_key().as_bytes());
    let name = leaf.name().as_str().as_bytes();
    bytes.extend_from_slice(&(name.len() as u32).to_le_bytes());
    bytes.extend_from_slice(name);
    let surname = leaf.surname().as_str().as_bytes();
    bytes.extend_from_slice(&(surname.len() as u32).to_le_bytes());
    bytes.extend_from_slice(surname);
    bytes.extend_from_slice(device_root);
    bytes
}

/// The ed25519 public key bytes of the first variant of `seed` that
/// `accept` takes.
fn first_key_bytes(seed: &str, accept: impl Fn(&[u8; 32]) -> bool) -> [u8; 32] {
    (0u32..)
        .map(|variant| {
            let secret: [u8; 32] = blake3::hash(format!("{seed}/{variant}").as_bytes()).into();
            *SigningKey::from_bytes(&secret).verifying_key().as_bytes()
        })
        .find(|bytes| accept(bytes))
        .expect("a qualifying variant exists")
}

/// 32 bytes valid both as an ed25519 public key and as a canonical,
/// non-small-order X25519 public key: the first seed variant whose ed25519
/// encoding has its top bit clear.
pub fn dual_key_bytes(seed: &str) -> [u8; 32] {
    first_key_bytes(seed, |bytes| PersonPublicKey::parse(bytes).is_ok())
}

pub fn member_key(seed: &str) -> PersonPublicKey {
    PersonPublicKey::parse(&dual_key_bytes(seed)).expect("dual-valid bytes")
}

/// The same bytes as `member_key(seed)`, so one seed names one key in either role.
pub fn device_key(seed: &str) -> DevicePublicKey {
    DevicePublicKey::parse(&dual_key_bytes(seed)).expect("dual-valid bytes")
}

/// 32 bytes valid as an ed25519 and as an X25519 public key, whose 4-bit
/// device position is `position`: the first variant of `seed` that is both.
/// The search is slow, so each is derived once per test process.
pub fn positioned_key_bytes(seed: &str, position: u8) -> [u8; 32] {
    type Derived = Mutex<HashMap<(String, u8), [u8; 32]>>;
    static DERIVED: OnceLock<Derived> = OnceLock::new();
    // A poisoned memo is still correct: an entry is inserted only once its
    // search has returned, so a panic leaves an entry missing, never wrong.
    let mut derived = DERIVED.get_or_init(Default::default).lock().unwrap_or_else(PoisonError::into_inner);
    *derived.entry((seed.to_owned(), position)).or_insert_with(|| {
        first_key_bytes(seed, |bytes| {
            PersonPublicKey::parse(bytes).is_ok()
                && DevicePublicKey::parse(bytes).is_ok_and(|key| person::device_position_bytes(&key)[0] >> 4 == position % 16)
        })
    })
}

/// The device key of `positioned_key_bytes(seed, position)`.
pub fn positioned_device_key(seed: &str, position: u8) -> DevicePublicKey {
    DevicePublicKey::parse(&positioned_key_bytes(seed, position)).expect("dual-valid bytes")
}

/// `count` device keys of `seed` at positions none of `held` has, nor one
/// another: the keys that fill a member's set without a collision.
pub fn free_position_keys(seed: &str, held: &[DevicePublicKey], count: usize) -> Vec<DevicePublicKey> {
    let taken: Vec<u8> = held.iter().map(|key| person::device_position_bytes(key)[0] >> 4).collect();
    (0u8..16)
        .filter(|position| !taken.contains(position))
        .take(count)
        .map(|position| positioned_device_key(&format!("{seed}-{position}"), position))
        .collect()
}
