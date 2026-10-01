#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Runtime-version → decoder dispatch: `decode::dispatch::for_runtime` is
//! the one place that turns a `spec_version` reported by the chain into a
//! decoder, and it must refuse every version it was not compiled against
//! rather than guess with the nearest layout (pallet-revive's storage and
//! event shapes are not stable across runtimes).
//!
//! The normal case is the pinned version, and it is asserted by *decoding*
//! — a known-good 96-byte storage blob through the returned decoder — so
//! the test proves which decoder was selected, not merely that some
//! `&dyn Decoder` came back. The abnormal-input cases are the two
//! neighbours of the pinned version and both ends of the `u32` range;
//! each must come back as `UnsupportedRuntime` carrying the version that
//! was asked for, since that number is what a caller logs and what tells
//! an operator which runtime it has to add support for.
//!
//! Relocated from the `#[cfg(test)]` module in
//! `on-chain-client/src/decode/dispatch.rs`, which no gate reads
//! (`test_paths` is `on-chain-client/tests`), and extended with the
//! neighbour cases class C requires.

use on_chain_client::decode::dispatch::{PASEO_AH_SPEC_VERSION, for_runtime};
use on_chain_client::decode::DecodeError;
use on_chain_client::{Epoch, OnChainRootHash, OrgPubKey, OrgState};

/// Distinctive, non-repeating slot contents: a decoder reading the three
/// 32-byte slots in the wrong order, or at the wrong offsets, cannot
/// produce the expected `OrgState` by accident.
fn root_hash_bytes() -> [u8; 32] {
    core::array::from_fn(|i| 0x10 + i as u8)
}

fn org_pub_key_bytes() -> [u8; 32] {
    core::array::from_fn(|i| 0x90 + i as u8)
}

const EPOCH: u64 = 0x0102_0304_0506_0708;

/// The three concatenated EVM slots `rootHash || orgPubKey || epoch` the
/// caller hands to `Decoder::decode_org_state`. `epoch` is a big-endian
/// `uint256`, so the `u64` occupies the last eight bytes.
fn known_good_blob() -> [u8; 96] {
    let mut blob = [0u8; 96];
    blob[0..32].copy_from_slice(&root_hash_bytes());
    blob[32..64].copy_from_slice(&org_pub_key_bytes());
    blob[88..96].copy_from_slice(&EPOCH.to_be_bytes());
    blob
}

/// Every unsupported version must be refused, and the error must carry
/// the version that was asked for.
fn assert_refused(spec_version: u32) {
    assert_eq!(
        for_runtime(spec_version).err(),
        Some(DecodeError::UnsupportedRuntime { spec_version }),
        "spec_version {spec_version} must be refused with the version it was asked about"
    );
}

// verifies: REQ-hd6m9d, LLR-b3s7st
#[test]
fn pinned_version_resolves_to_a_decoder_that_decodes_the_pinned_layout() {
    let decoder =
        for_runtime(PASEO_AH_SPEC_VERSION).expect("the pinned spec_version must resolve");
    let state = decoder
        .decode_org_state(&known_good_blob())
        .expect("the selected decoder must decode a known-good 96-byte blob");
    assert_eq!(
        state,
        OrgState {
            root_hash: OnChainRootHash(root_hash_bytes()),
            org_pub_key: OrgPubKey(org_pub_key_bytes()),
            epoch: Epoch(EPOCH),
        },
        "the decoder dispatch handed back does not decode the pinned layout"
    );
}

// verifies: REQ-hd6m9d, LLR-u8ajby
#[test]
fn version_one_below_the_pinned_one_is_refused() {
    assert_refused(PASEO_AH_SPEC_VERSION - 1);
}

// verifies: REQ-hd6m9d, LLR-u8ajby
#[test]
fn version_one_above_the_pinned_one_is_refused() {
    assert_refused(PASEO_AH_SPEC_VERSION + 1);
}

// verifies: REQ-hd6m9d, LLR-u8ajby
#[test]
fn zero_version_is_refused() {
    assert_refused(0);
}

// verifies: REQ-hd6m9d, LLR-u8ajby
#[test]
fn max_version_is_refused() {
    assert_refused(u32::MAX);
}
