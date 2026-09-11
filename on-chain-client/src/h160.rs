//! `h160_of(account_id_32)` — pallet-revive's AccountId32 → H160 mapping.
//! This is what determines an org's slot key in `OrgRegistry` storage:
//! whatever 20 bytes pallet-revive maps the pure proxy's AccountId32 to
//! IS the on-chain OrgId.
//!
//! pallet-revive (recent Polkadot SDK) uses two cases:
//!
//! 1. If the AccountId32 has the EVM-fallback prefix (the last 12 bytes
//!    are all `0xEE`), the *first* 20 bytes are themselves the H160 —
//!    this is the reverse mapping for accounts derived from a known
//!    EVM address.
//! 2. Otherwise, keccak256 of the full 32-byte AccountId, take the last
//!    20 bytes. This is the "stateless forward" mapping for Substrate-
//!    style 32-byte accounts (the case our pure proxies live in).
//!
//! The exact byte position of the `0xEE` prefix has changed across
//! pallet-revive versions. Two tests bear on that: `tests/h160_mapping.rs`
//! pins both paths against an independent recomputation of the reading above
//! and runs in this unit's gate, while `tests/p_address_is_orgid.rs` pins the
//! mapping against the runtime's own answer and needs a live chopsticks fork,
//! so it runs in no gate.

use tiny_keccak::{Hasher, Keccak};

/// Mark byte (12 copies of) used by pallet-revive to identify accounts
/// that are EVM-derived (reverse mapping). Lives in the LAST 12 bytes
/// of the AccountId32 — Substrate accounts that came from an H160 are
/// `H160 || [0xEE; 12]`. Confirm against the live runtime in Task 7.
const EVM_FALLBACK_MARK: u8 = 0xEE;

/// Map a 32-byte AccountId32 to its pallet-revive H160.
///
/// Two paths:
///   - Reverse: if the last 12 bytes are `0xEE`, return the first 20.
///   - Forward: keccak256(account_id_32)[12..32].
pub fn h160_of(account_id_32: [u8; 32]) -> [u8; 20] {
    if account_id_32[20..32].iter().all(|b| *b == EVM_FALLBACK_MARK) {
        let mut h160 = [0u8; 20];
        h160.copy_from_slice(&account_id_32[..20]);
        return h160;
    }
    let mut hasher = Keccak::v256();
    hasher.update(&account_id_32);
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    let mut h160 = [0u8; 20];
    h160.copy_from_slice(&hash[12..32]);
    h160
}
