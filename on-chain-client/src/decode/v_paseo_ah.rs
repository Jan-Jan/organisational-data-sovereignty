//! Decoder for a single pinned Paseo AH runtime version. Reachable via
//! `dispatch::for_runtime` when `spec_version == SPEC_VERSION`.
//!
//! Layouts decoded:
//!
//! - **`decode_org_state`** — takes 96 concatenated bytes (3 × 32-byte EVM
//!   storage slots: `rootHash` || `orgPubKey` || `epoch` big-endian).
//!   The first two slots are copied wholesale; the epoch slot is decoded
//!   as a `uint256` whose high 24 bytes must be zero (a +1-per-update
//!   counter never reaches `u64::MAX`, so a non-zero high half is treated
//!   as `DecodeError::EpochOverflow`).
//! - **`parse_revive_event`** — takes the SCALE-encoded payload of
//!   `pallet_revive::Event::ContractEmitted { contract, data, topics }`.
//!   `topics[0]` is the EVM event signature hash; a first topic matching
//!   no signature this decoder knows yields `Ok(None)`, because the follow
//!   stream carries every contract's events. Filtering by emitting
//!   contract is the caller's check, on the address returned in
//!   `EmittedEvent`. Decoded events return `Event::Genesis` or
//!   `Event::Update` with all indexed and non-indexed fields recovered,
//!   paired with the emitting contract's H160.
//!
//! This module has no `#[cfg(test)]` module of its own, by design. Both
//! decoders are tested from `on-chain-client/tests` —
//! `parse_revive_event`'s shape checks from `decode_revive_event.rs`,
//! `decode_org_state`'s length and epoch bounds from `decode_org_state.rs` —
//! because this unit's gate reads its evidence from `test_paths`
//! (`on-chain-client/tests`), so an annotated test written here would be read
//! by no gate. Both decoders are reached there through the public
//! `dispatch::for_runtime`, the way production reaches them.

use alloc::format;
use alloc::vec::Vec;

use parity_scale_codec::Decode;

use super::{DecodeError, Decoder};
use crate::state::{EmittedEvent, Event, OrgState};
use crate::types::{Epoch, OnChainRootHash, OrgAdmin, OrgPubKey};

/// Paseo AH runtime spec_version this decoder targets. Captured from a
/// chopsticks fork in Task 6 (`state_getRuntimeVersion` reported
/// `specName: "asset-hub-paseo"`, `specVersion: 2002002`). Task 10 may
/// widen `dispatch::for_runtime` to accept a known-good range once we've
/// run against several upstream versions.
pub const SPEC_VERSION: u32 = 2_002_002;

/// `keccak256("GenesisInitialized(address,bytes32,bytes32)")`.
/// Re-derived from the canonical Solidity signature string in
/// `on-chain-client/tests/decode_revive_event.rs`, which is where the gate
/// reads it, to lock the const against ABI drift.
pub(super) const SIG_GENESIS_INITIALIZED: [u8; 32] = [
    0x8e, 0x65, 0xbf, 0x09, 0x54, 0x40, 0x39, 0x7e, 0x54, 0x61, 0x39, 0x32, 0xb7, 0x54, 0x91, 0x7e,
    0x45, 0x22, 0xdd, 0xb0, 0x8a, 0x8e, 0x63, 0x8b, 0xcb, 0x8d, 0xee, 0x69, 0xfe, 0x68, 0x5b, 0x6d,
];

/// `keccak256("RootUpdated(address,uint256,bytes32,bytes32,bytes32)")`.
pub(super) const SIG_ROOT_UPDATED: [u8; 32] = [
    0x24, 0x79, 0x88, 0xcb, 0x06, 0x65, 0x74, 0x6b, 0xde, 0x9b, 0xe0, 0xb7, 0x06, 0x8f, 0x5d, 0x04,
    0x96, 0xe8, 0xe7, 0x5d, 0x1a, 0x4b, 0x26, 0x92, 0xb1, 0x98, 0xf6, 0x77, 0x89, 0xee, 0x5b, 0x6e,
];

pub(super) struct DecoderImpl;

/// Static instance handed out by `dispatch::for_runtime`. Zero-sized so a
/// `&'static dyn Decoder` reference costs nothing.
pub(super) static DECODER: DecoderImpl = DecoderImpl;

impl Decoder for DecoderImpl {
    fn decode_org_state(&self, bytes: &[u8]) -> Result<OrgState, DecodeError> {
        if bytes.len() != 96 {
            return Err(DecodeError::StorageLengthMismatch {
                expected: 96,
                actual: bytes.len(),
            });
        }
        let mut root_hash = [0u8; 32];
        root_hash.copy_from_slice(&bytes[0..32]);
        let mut org_pub_key = [0u8; 32];
        org_pub_key.copy_from_slice(&bytes[32..64]);
        let epoch = decode_uint256_to_u64(&bytes[64..96])?;
        Ok(OrgState {
            root_hash: OnChainRootHash(root_hash),
            org_pub_key: OrgPubKey(org_pub_key),
            epoch: Epoch(epoch),
        })
    }

    fn parse_revive_event(&self, mut bytes: &[u8]) -> Result<Option<EmittedEvent>, DecodeError> {
        // pallet_revive::Event::ContractEmitted {
        //     contract: H160,     // 20 raw bytes
        //     data: Vec<u8>,      // compact_len(data) || data
        //     topics: Vec<H256>,  // compact_len(topics) || topics[0..N]
        // }
        // `contract` is carried out rather than dropped: it is the only
        // field that distinguishes a genuine OrgRegistry log from one any
        // other contract can emit with the same signature hash and the same
        // indexed admin (HAZ-werm85).
        let contract: [u8; 20] = Decode::decode(&mut bytes)
            .map_err(|e| DecodeError::Scale(format!("contract: {e}")))?;
        let data: Vec<u8> =
            Decode::decode(&mut bytes).map_err(|e| DecodeError::Scale(format!("data: {e}")))?;
        let topics: Vec<[u8; 32]> = Decode::decode(&mut bytes)
            .map_err(|e| DecodeError::Scale(format!("topics: {e}")))?;
        if !bytes.is_empty() {
            return Err(DecodeError::Scale(format!(
                "trailing {} bytes after ContractEmitted payload",
                bytes.len()
            )));
        }

        let Some(sig) = topics.first() else {
            return Ok(None);
        };
        let event = match *sig {
            SIG_GENESIS_INITIALIZED => parse_genesis(&data, &topics)?,
            SIG_ROOT_UPDATED => parse_root_updated(&data, &topics)?,
            _ => return Ok(None),
        };
        Ok(Some(EmittedEvent { contract, event }))
    }
}

fn decode_uint256_to_u64(bytes: &[u8]) -> Result<u64, DecodeError> {
    // Solidity uint256 is big-endian. The high 24 bytes must be zero for
    // the counter to fit in u64; the contract increments by 1 per update
    // so this is always the case in practice.
    debug_assert_eq!(bytes.len(), 32);
    if bytes[..24].iter().any(|b| *b != 0) {
        return Err(DecodeError::EpochOverflow);
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[24..32]);
    Ok(u64::from_be_bytes(buf))
}

fn parse_genesis(data: &[u8], topics: &[[u8; 32]]) -> Result<Event, DecodeError> {
    if topics.len() != 2 {
        return Err(DecodeError::InvalidTopicCount {
            event: "GenesisInitialized",
            expected: 2,
            actual: topics.len(),
        });
    }
    if data.len() != 64 {
        return Err(DecodeError::InvalidDataLength {
            event: "GenesisInitialized",
            expected: 64,
            actual: data.len(),
        });
    }
    let admin = unpack_address_topic(&topics[1])?;
    let mut root_hash = [0u8; 32];
    root_hash.copy_from_slice(&data[0..32]);
    let mut org_pub_key = [0u8; 32];
    org_pub_key.copy_from_slice(&data[32..64]);
    Ok(Event::Genesis {
        admin: OrgAdmin(admin),
        root_hash: OnChainRootHash(root_hash),
        org_pub_key: OrgPubKey(org_pub_key),
    })
}

fn parse_root_updated(data: &[u8], topics: &[[u8; 32]]) -> Result<Event, DecodeError> {
    if topics.len() != 3 {
        return Err(DecodeError::InvalidTopicCount {
            event: "RootUpdated",
            expected: 3,
            actual: topics.len(),
        });
    }
    if data.len() != 96 {
        return Err(DecodeError::InvalidDataLength {
            event: "RootUpdated",
            expected: 96,
            actual: data.len(),
        });
    }
    let admin = unpack_address_topic(&topics[1])?;
    let epoch = decode_uint256_to_u64(&topics[2])?;
    let mut root_hash = [0u8; 32];
    root_hash.copy_from_slice(&data[0..32]);
    let mut org_pub_key = [0u8; 32];
    org_pub_key.copy_from_slice(&data[32..64]);
    let mut prev_root_hash = [0u8; 32];
    prev_root_hash.copy_from_slice(&data[64..96]);
    Ok(Event::Update {
        admin: OrgAdmin(admin),
        epoch: Epoch(epoch),
        root_hash: OnChainRootHash(root_hash),
        org_pub_key: OrgPubKey(org_pub_key),
        prev_root_hash: OnChainRootHash(prev_root_hash),
    })
}

fn unpack_address_topic(topic: &[u8; 32]) -> Result<[u8; 20], DecodeError> {
    if topic[..12].iter().any(|b| *b != 0) {
        return Err(DecodeError::InvalidAddressTopic);
    }
    let mut admin = [0u8; 20];
    admin.copy_from_slice(&topic[12..32]);
    Ok(admin)
}
