//! Key fixtures shared by the integration test crates. Each test crate uses
//! a subset, hence the `dead_code` allowance.
#![allow(dead_code)]

use ed25519_dalek::SigningKey;
use org_members::types::{DevicePublicKey, PersonPublicKey};

/// 32 bytes valid both as an ed25519 public key and as a canonical,
/// non-small-order X25519 public key: the first seed variant whose ed25519
/// encoding has its top bit clear.
pub fn dual_key_bytes(seed: &str) -> [u8; 32] {
    (0u32..)
        .map(|i| {
            let s: [u8; 32] = blake3::hash(format!("{seed}/{i}").as_bytes()).into();
            *SigningKey::from_bytes(&s).verifying_key().as_bytes()
        })
        .find(|b| PersonPublicKey::parse(b).is_ok())
        .expect("about half of all seeds qualify")
}

pub fn member_key(seed: &str) -> PersonPublicKey {
    PersonPublicKey::parse(&dual_key_bytes(seed)).expect("dual-valid bytes")
}

/// The same bytes as `member_key(seed)`, so one seed names one key in either role.
pub fn device_key(seed: &str) -> DevicePublicKey {
    DevicePublicKey::parse(&dual_key_bytes(seed)).expect("dual-valid bytes")
}
