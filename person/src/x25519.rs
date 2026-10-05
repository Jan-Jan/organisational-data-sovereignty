//! The X25519 public-key validity rule, exported so org-node applies the same
//! rule to the organisation's key. LLR-m75m7u.

/// 2^255 - 19, little-endian.
const FIELD_PRIME: [u8; 32] = {
    let mut bytes = [0xff; 32];
    bytes[0] = 0xed;
    bytes[31] = 0x7f;
    bytes
};

/// The small-order u-coordinates below the field prime, from libsodium's
/// `has_small_order`. Its two other entries, p and p + 1, are not below p and
/// are rejected as non-canonical.
const SMALL_ORDER: [[u8; 32]; 5] = [
    [0; 32],
    {
        let mut bytes = [0; 32];
        bytes[0] = 1;
        bytes
    },
    [
        0xe0, 0xeb, 0x7a, 0x7c, 0x3b, 0x41, 0xb8, 0xae, 0x16, 0x56, 0xe3, 0xfa, 0xf1, 0x9f, 0xc4,
        0x6a, 0xda, 0x09, 0x8d, 0xeb, 0x9c, 0x32, 0xb1, 0xfd, 0x86, 0x62, 0x05, 0x16, 0x5f, 0x49,
        0xb8, 0x00,
    ],
    [
        0x5f, 0x9c, 0x95, 0xbc, 0xa3, 0x50, 0x8c, 0x24, 0xb1, 0xd0, 0xb1, 0x55, 0x9c, 0x83, 0xef,
        0x5b, 0x04, 0x44, 0x5c, 0xc4, 0x58, 0x1c, 0x8e, 0x86, 0xd8, 0x22, 0x4e, 0xdd, 0xd0, 0x9f,
        0x11, 0x57,
    ],
    {
        let mut bytes = FIELD_PRIME;
        bytes[0] = 0xec;
        bytes
    },
];

/// True exactly when `bytes` are a canonical X25519 public key that is not of
/// small order. LLR-m75m7u.
///
/// Canonical means the little-endian value is below 2^255 - 19. A value with
/// the top bit set is at least 2^255, so this comparison also rejects it.
pub fn is_valid_public_key(bytes: &[u8; 32]) -> bool {
    let is_canonical = bytes.iter().rev().lt(FIELD_PRIME.iter().rev());
    is_canonical && !SMALL_ORDER.contains(bytes)
}
