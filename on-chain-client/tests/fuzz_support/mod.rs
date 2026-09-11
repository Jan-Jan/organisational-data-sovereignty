//! Shared helpers for the structured fuzz target (`fuzz_event_round_trip`)
//! and the corpus regenerator. Pulled in with
//! `#[path = ".../fuzz_support/mod.rs"] mod support;`. It lives in a
//! subdirectory so cargo does NOT auto-discover it as a (zero-test) target.
//!
//! Two pieces:
//!   * `sig_genesis()` / `sig_root_updated()` — the EVM event-signature topic
//!     hashes, derived from their canonical Solidity signature strings rather
//!     than copied from the decoder's `pub(super)` constants. Independent
//!     derivation = a drift guard. The guard itself is
//!     `the_recognised_signature_hashes_are_the_keccak_of_the_canonical_solidity_strings`
//!     in `on-chain-client/tests/decode_revive_event.rs` (verifies REQ-52uc8f
//!     and REQ-wnjz9j), which pins these two functions against the canonical
//!     Solidity strings written out as literals and then asks the decoder to
//!     accept those hashes and reject a drifted neighbour. This line named
//!     the lib's internal `event_signatures_match_solidity_abi` until review
//!     round 2's finding 7; that test was deleted from `src` by this change
//!     and relocated here, re-expressed as behaviour rather than as equality
//!     against a constant a test outside the crate cannot see.
//!   * `encode_contract_emitted` — the canonical SCALE encoding of
//!     `pallet_revive::Event::ContractEmitted { contract, data, topics }`,
//!     mirroring the contract ABI. Independent of the decoder, so a round trip
//!     through it is a genuine cross-check, not a tautology.
//!
//! Not every item is used by every includer; `#[allow(dead_code)]` keeps the
//! shared module warning-free when a consumer uses only part of it.
#![allow(dead_code)]

use parity_scale_codec::Encode;
use tiny_keccak::{Hasher, Keccak};

fn keccak(s: &str) -> [u8; 32] {
    let mut h = Keccak::v256();
    h.update(s.as_bytes());
    let mut out = [0u8; 32];
    h.finalize(&mut out);
    out
}

pub fn sig_genesis() -> [u8; 32] {
    keccak("GenesisInitialized(address,bytes32,bytes32)")
}

pub fn sig_root_updated() -> [u8; 32] {
    keccak("RootUpdated(address,uint256,bytes32,bytes32,bytes32)")
}

/// Left-pad a 20-byte EVM address into a 32-byte indexed topic (Solidity
/// zero-pads on the left).
pub fn padded_address(addr: [u8; 20]) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[12..32].copy_from_slice(&addr);
    out
}

/// Encode a `u64` as a big-endian `uint256` topic.
pub fn uint256_be(n: u64) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..32].copy_from_slice(&n.to_be_bytes());
    out
}

/// Canonical SCALE encoding of `ContractEmitted { contract, data, topics }`:
/// `contract` is 20 raw bytes, then compact-length-prefixed `data`, then
/// compact-length-prefixed `topics`.
pub fn encode_contract_emitted(
    contract: [u8; 20],
    data: Vec<u8>,
    topics: Vec<[u8; 32]>,
) -> Vec<u8> {
    let mut buf = Vec::new();
    contract.encode_to(&mut buf);
    data.encode_to(&mut buf);
    topics.encode_to(&mut buf);
    buf
}
