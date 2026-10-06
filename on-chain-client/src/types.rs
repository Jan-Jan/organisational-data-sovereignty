//! Named types mirroring the on-chain `OrgRegistry` fields. Newtypes around
//! fixed-size byte arrays — chosen over naked primitives so the public API
//! can't accidentally swap `OrgPubKey` for `OnChainRootHash` (both are 32
//! bytes). Matches the surface declared in spec §3.
//!
//! Tested from `on-chain-client/tests/type_widths.rs`, not from a
//! `#[cfg(test)]` module here: this unit's gate reads its evidence from
//! `test_paths` (`on-chain-client/tests`), so an annotated test written in
//! `src` would be read by no gate. The two cases that lived here until
//! 2026-09-10 were the last such tests in this crate's `src` (review round 1,
//! finding 1).

use core::fmt;

/// H160 address of an org's pure proxy `P`. Stable for the lifetime of the
/// org: rotating the controlling multisig `M(signers, threshold)` does not
/// affect `P`, so this value is the on-chain OrgId.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OrgAdmin(pub [u8; 20]);

/// The 32-byte sparse-merkle root from `org-members`, anchored on-chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OnChainRootHash(pub [u8; 32]);

/// The Organisation public key as stored on-chain: the org's X25519
/// key-agreement key, by which a member checks the org private key shared
/// with them. Not a signing key. Opaque 32 bytes here: this crate never
/// checks that they are a valid X25519 key.
///
/// Where the check happens, and where it does not: org-node checks the key
/// when it reads Organisation state (`OrgState::from_chain`, in
/// `org-node/src/chain.rs`). The `GenesisInitialized` and `RootUpdated`
/// events also carry an `OrgPubKey`, and those bytes reach their reader
/// unchecked. Nothing reads them today; a consumer that starts to must parse
/// them first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OrgPubKey(pub [u8; 32]);

/// Monotonic per-org counter. `0` = uninitialised slot; `1` = post-genesis;
/// `+1` per successful `update(...)`. Wraparound is not reachable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Epoch(pub u64);

impl fmt::Display for Epoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

