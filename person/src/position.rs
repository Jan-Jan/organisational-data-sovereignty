//! A DevicePublicKey's position bytes, under one domain for every device
//! root, Person's and Member's alike. LLR-exwx4n.

use crate::device_key::DevicePublicKey;
use crate::keyed_hash::keyed_hash;

/// `persona::device-position`, padded with `_` to 32 bytes: one position
/// domain for Person and Member device root trees alike. LLR-exwx4n.
pub const DEVICE_POSITION_KEY: &[u8; 32] = b"persona::device-position________";

/// A key's position bytes: `blake3::keyed_hash` of its 32 bytes under
/// `DEVICE_POSITION_KEY`; a device root tree reads its first `DEVICE_ROOT_DEPTH`
/// bits. LLR-exwx4n.
pub fn device_position_bytes(key: &DevicePublicKey) -> [u8; 32] {
    keyed_hash(DEVICE_POSITION_KEY, &[key.as_bytes()])
}
