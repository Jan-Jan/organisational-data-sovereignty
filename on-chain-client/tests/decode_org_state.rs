#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `Decoder::decode_org_state` — the bounds that stand between three raw EVM
//! storage slots read off Asset Hub and the `OrgState` every consumer then
//! treats as the chain's word on an organisation's membership root and epoch.
//!
//! Two requirements are gated here, both reached through the public
//! `decode::dispatch::for_runtime` rather than through anything in `src` that
//! only the crate can see:
//!
//!   - REQ-4astjb — the blob must be exactly 96 bytes (3 × 32-byte slots:
//!     `rootHash || orgPubKey || epoch`), and any other length must be
//!     refused with `StorageLengthMismatch` naming both the length required
//!     and the length actually read. The too-short side is the dangerous one:
//!     the three `copy_from_slice` calls below the guard have no bounds check
//!     of their own, so a guard that let a short blob through would **panic**
//!     inside a `no_std` library whose crate root denies `panic`.
//!   - REQ-9wwenn — the epoch slot is a Solidity `uint256` and the client's
//!     `Epoch` is a `u64`, so the slot's leading 24 bytes must be zero. A
//!     non-zero byte anywhere in them means the value does not fit, and must
//!     be refused as `EpochOverflow` rather than truncated into a smaller
//!     epoch — a truncated epoch would order one root against another
//!     wrongly, which is the whole point of carrying an epoch.
//!
//! Both requirements implement RC-sxjnx9.
//!
//! **Independent derivation.** Nothing here is imported from the decoder. The
//! 96-byte blobs are assembled slot by slot in this file and the big-endian
//! `uint256` encoding is written out locally, so the expectations are a
//! second statement of the layout rather than a round trip through the code
//! under test. Values are distinctive and non-repeating wherever an offset is
//! being pinned, so a decoder reading the right number of bytes from the
//! wrong offset cannot produce the expected answer by accident.
//!
//! Relocated out of the `#[cfg(test)]` module in
//! `on-chain-client/src/decode/v_paseo_ah.rs`, which no gate reads
//! (`test_paths` for this unit is `on-chain-client/tests`), and joined by the
//! abnormal-input cases class C asks for. The relocated assertions are kept
//! verbatim where they stand alone and are absorbed, position for position,
//! where a sweep now covers them — noted at each such test. Relocating them
//! emptied that module, so it is deleted in this task's commit.
//!
//! No `test-support` import is needed: `for_runtime` and the `Decoder` trait
//! are unconditionally public. The target is still named in T10's
//! `required-features = ["test-support"]` list, in common with every other
//! target the gate runs, so that a target named in `verify_commands` can
//! never silently not run.
//!
//! **What this file does NOT establish.** It says nothing about where the 96
//! bytes came from. That three slot reads were issued against the right keys,
//! concatenated in the right order, and that an absent slot means no such
//! organisation, are the caller's business: the slot derivation is gated in
//! `on-chain-client/tests/storage_slot_layout.rs`, and the absent-slot reading
//! is sound only because of a `ZeroValue()` revert in
//! `on-chain/src/OrgRegistry.sol` — a control implemented outside this unit,
//! with no gated evidence here (HAZ-xfg9cz).

use on_chain_client::decode::dispatch::{PASEO_AH_SPEC_VERSION, for_runtime};
use on_chain_client::decode::{DecodeError, Decoder};
use on_chain_client::{Epoch, OnChainRootHash, OrgPubKey, OrgState};

/// The decoder under test, reached the way production reaches it.
fn decoder() -> &'static dyn Decoder {
    for_runtime(PASEO_AH_SPEC_VERSION).expect("the pinned runtime has a compiled-in decoder")
}

fn decode(bytes: &[u8]) -> Result<OrgState, DecodeError> {
    decoder().decode_org_state(bytes)
}

/// The three slots concatenated in the order the caller reads them:
/// `S = rootHash`, `S+1 = orgPubKey`, `S+2 = epoch`.
fn blob(root_hash: [u8; 32], org_pub_key: [u8; 32], epoch_slot: [u8; 32]) -> [u8; 96] {
    let mut out = [0u8; 96];
    out[0..32].copy_from_slice(&root_hash);
    out[32..64].copy_from_slice(&org_pub_key);
    out[64..96].copy_from_slice(&epoch_slot);
    out
}

/// A Solidity `uint256` holding a `u64`: big-endian, so the value sits in the
/// slot's *last* eight bytes and the leading 24 are zero. Written out here
/// rather than imported, which is what makes the epoch expectations below an
/// independent statement of the encoding.
fn uint256_be(value: u64) -> [u8; 32] {
    let mut slot = [0u8; 32];
    slot[24..32].copy_from_slice(&value.to_be_bytes());
    slot
}

/// A rejection has to be *legible*: the operator reading the log needs the
/// length the layout requires and the length the chain actually returned,
/// both. A variant that carried only one of the two would leave them
/// guessing whether they had read too few slots or too many.
fn assert_error_names(err: &DecodeError, expected: usize, actual: usize) {
    let msg = err.to_string();
    assert!(
        msg.contains(&expected.to_string()),
        "error must name the expected length {expected}: {msg}"
    );
    assert!(
        msg.contains(&actual.to_string()),
        "error must name the actual length {actual}: {msg}"
    );
}

// ---------------------------------------------------------------------------
// REQ-4astjb — exactly three slots, and nothing else
// ---------------------------------------------------------------------------

// verifies: REQ-4astjb, REQ-9wwenn, LLR-nq7nhg, LLR-hezpr7
//
// The normal case, and the one that pins every offset: 96 bytes decode, and
// each of the three fields comes back from the slot it was written to.
// Relocated from `src`'s `decode_org_state_round_trip`, whose `0xaa`/`0xbb`
// fills are replaced by ascending runs — a uniform fill cannot tell a decoder
// that read slot 0 twice from one that read slots 0 and 1.
#[test]
fn exactly_ninety_six_bytes_decodes_every_field() {
    let root_hash: [u8; 32] = core::array::from_fn(|i| 0x40 + i as u8);
    let org_pub_key: [u8; 32] = core::array::from_fn(|i| 0x80 + i as u8);

    assert_eq!(
        decode(&blob(root_hash, org_pub_key, uint256_be(7))),
        Ok(OrgState {
            root_hash: OnChainRootHash(root_hash),
            org_pub_key: OrgPubKey(org_pub_key),
            epoch: Epoch(7),
        }),
    );
}

// verifies: REQ-4astjb, LLR-sq76u3
//
// One byte short. This is the near neighbour below the bound, where a `<`
// written in place of a `!=` hides, and it is the direction that matters
// most: with the guard weakened this way the blob reaches
// `copy_from_slice(&bytes[64..96])` and panics. Relocated verbatim from
// `src`'s `decode_org_state_wrong_length_rejected`, extended to assert the
// error names both lengths.
#[test]
fn ninety_five_bytes_is_rejected() {
    let err = decode(&[0u8; 95]).expect_err("95 bytes is not three slots");
    assert_eq!(
        err,
        DecodeError::StorageLengthMismatch {
            expected: 96,
            actual: 95,
        },
    );
    assert_error_names(&err, 96, 95);
}

// verifies: REQ-4astjb, LLR-sq76u3
//
// One byte long — the other side of the bound, so it is pinned from both
// directions rather than only from below. A 97-byte answer is not three slots
// with a harmless extra byte; it is a read this decoder cannot account for,
// and reading the first 96 bytes out of it would be guessing which 96.
#[test]
fn ninety_seven_bytes_is_rejected() {
    let err = decode(&[0u8; 97]).expect_err("97 bytes is not three slots");
    assert_eq!(
        err,
        DecodeError::StorageLengthMismatch {
            expected: 96,
            actual: 97,
        },
    );
    assert_error_names(&err, 96, 97);
}

// verifies: REQ-4astjb, LLR-sq76u3
//
// The empty blob, which is not a hypothetical: it is the shape a storage read
// takes when it returns nothing at all. It is also the widest gap between the
// guard and the code below it — every one of the three `copy_from_slice`
// calls would be out of bounds — so this is the case that turns a wrong
// length guard into a panic rather than a wrong answer, in a `no_std` library
// whose crate root denies `panic`.
#[test]
fn an_empty_blob_is_rejected_rather_than_read_out_of_bounds() {
    let err = decode(&[]).expect_err("no bytes is not three slots");
    assert_eq!(
        err,
        DecodeError::StorageLengthMismatch {
            expected: 96,
            actual: 0,
        },
    );
    assert_error_names(&err, 96, 0);
}

// verifies: REQ-4astjb, LLR-sq76u3
//
// Whole-slot miscounts, the realistic wrong lengths: one slot, two slots,
// four slots and six slots. Two slots (64 bytes) is what a caller that
// forgot the epoch would hand over, and six is what one that concatenated two
// organisations' reads would — each is a plausible caller bug that must
// surface as a refusal here rather than as a plausible-looking `OrgState`.
#[test]
fn a_whole_number_of_slots_other_than_three_is_rejected() {
    for actual in [32usize, 64, 128, 192] {
        let err = decode(&vec![0u8; actual]).expect_err("only three slots is three slots");
        assert_eq!(
            err,
            DecodeError::StorageLengthMismatch {
                expected: 96,
                actual,
            },
            "{actual} bytes is not three slots"
        );
        assert_error_names(&err, 96, actual);
    }
}

// ---------------------------------------------------------------------------
// REQ-9wwenn — the epoch slot's high 24 bytes
// ---------------------------------------------------------------------------

// verifies: REQ-9wwenn, LLR-emp3g9
//
// The sweep. A `uint256` that does not fit in a `u64` has a non-zero byte
// somewhere in the slot's leading 24, and *every* one of those positions must
// be refused — position 0 alone (which is what the relocated `src` test
// asserted, and which is absorbed here at `i == 0`) would leave the guard
// free to check a narrower window than the layout requires. The two edges are
// the ones that matter: position 0 is the most significant byte, and position
// 23 is the byte immediately above the `u64`, where an off-by-one in the
// window's width hides.
#[test]
fn a_non_zero_byte_anywhere_in_the_epoch_slots_leading_twenty_four_bytes_is_refused() {
    for i in 0..24usize {
        let mut epoch_slot = uint256_be(7);
        epoch_slot[i] = 0x01;
        assert_eq!(
            decode(&blob([0xaa; 32], [0xbb; 32], epoch_slot)),
            Err(DecodeError::EpochOverflow),
            "a non-zero byte at epoch-slot position {i} does not fit in a u64 \
             and must not be truncated into an epoch"
        );
    }
}

// verifies: REQ-9wwenn, LLR-emp3g9
//
// The largest epoch that does fit, accepted whole. Relocated from `src`'s
// `decode_org_state_max_u64_epoch_ok`. This is the case that keeps the
// overflow guard from being written as "the epoch must be small": every one
// of the low eight bytes is `0xff` here, and the answer is `u64::MAX`, not a
// refusal.
#[test]
fn the_largest_u64_epoch_is_accepted_whole() {
    let state = decode(&blob([0xaa; 32], [0xbb; 32], uint256_be(u64::MAX))).expect("decode");
    assert_eq!(state.epoch, Epoch(u64::MAX));
}

// verifies: REQ-9wwenn, LLR-emp3g9
//
// Zero. An all-zero epoch slot is inside the range and must decode to
// `Epoch(0)`, not be mistaken for an absent or invalid value: this decoder's
// job is to report the bytes it was given, and the question of what an absent
// slot means belongs to the caller (HAZ-xfg9cz), not here.
#[test]
fn a_zero_epoch_is_accepted() {
    let state = decode(&blob([0xaa; 32], [0xbb; 32], [0u8; 32])).expect("decode");
    assert_eq!(state.epoch, Epoch(0));
}

// verifies: REQ-9wwenn, LLR-emp3g9
//
// The window's lower edge, swept from the other side. For each of the eight
// bytes the `u64` is made of, a slot holding `0x01` at that position alone
// must decode to exactly that byte's big-endian weight. Position 24 is the
// one that matters: it is inside the `u64` and immediately below the checked
// window, so a guard one byte too wide would refuse it, and a decoder reading
// its eight bytes from the wrong offset would report the wrong power of two.
#[test]
fn each_of_the_low_eight_bytes_carries_its_big_endian_weight() {
    for i in 24..32usize {
        let mut epoch_slot = [0u8; 32];
        epoch_slot[i] = 0x01;
        // Byte 24 is the u64's most significant, byte 31 its least.
        let expected = 1u64 << (8 * (31 - i));
        let state = decode(&blob([0xaa; 32], [0xbb; 32], epoch_slot))
            .expect("a value inside u64 range must decode");
        assert_eq!(
            state.epoch,
            Epoch(expected),
            "0x01 at epoch-slot position {i} is {expected} big-endian"
        );
    }
}
