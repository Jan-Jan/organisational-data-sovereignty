//! The unit's adapter over blake3: the keyed hash of byte parts, taken in
//! order, under a 32-byte domain key.

/// `blake3` keyed by `key` over `parts`, one after another.
pub(crate) fn keyed_hash(key: &[u8; 32], parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_keyed(key);
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}
