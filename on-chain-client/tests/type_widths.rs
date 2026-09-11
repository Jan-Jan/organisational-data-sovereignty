//! The byte widths of this crate's public newtypes, and `Epoch`'s `Display`.
//!
//! Why this target exists. Both cases below lived in a `#[cfg(test)] mod
//! tests` inside `on-chain-client/src/types.rs` until 2026-09-10. That is
//! outside this unit's `test_paths` (on-chain-client/tests), so
//! `check-trace.sh` could not read an annotation on them however they were
//! written: they were the last two `#[cfg(test)]` tests anywhere in
//! `on-chain-client/src`, and five documents had already recorded the
//! relocation as complete. Review round 1, finding 1. Relocating them is what
//! makes the width case gateable at all.
//!
//! What the width case is really asserting. Not "20 is 20": each newtype
//! wraps a field of the `OrgRegistry` ABI, and its width is that field's
//! width in the ABI — not a number this crate is free to choose. The ABI
//! passes every value in a 32-byte word, and the three widths below are the
//! three ways `on-chain-client/src/decode/v_paseo_ah.rs` reads one:
//!
//!   * `address indexed admin` is the LOW 20 bytes of an indexed word,
//!     `unpack_address_topic`'s `topic[12..32]`, the high 12 being zero
//!     padding it rejects if non-zero → `OrgAdmin` is 20 bytes.
//!   * `bytes32 rootHash` / `bytes32 orgPubKey` / `bytes32 prevRootHash`
//!     occupy the whole word, copied wholesale → `OnChainRootHash` and
//!     `OrgPubKey` are 32 bytes.
//!   * `uint256 indexed epoch` is narrowed on the way in:
//!     `decode_uint256_to_u64` refuses any word whose high 24 bytes are
//!     non-zero and takes `bytes[24..32]` → `Epoch` is 8 bytes, a `u64`.
//!     Eight is therefore the width the DECODER gives the field, not the
//!     width Solidity declares; the narrowing is deliberate (the counter
//!     increments by one per `update` and cannot reach `u64::MAX`) and the
//!     refusal of a wider value is gated in `tests/decode_org_state.rs` —
//!     there and, as the paragraph below records, nowhere else.
//!
//! So each assertion below states a width as `ABI word minus the offset the
//! decoder reads from`, and then proves the newtype's inner array accepts
//! exactly that region of a word — a truncation or a widened wrapper fails to
//! convert. A bare `assert_eq!(size_of::<OrgAdmin>(), 20)` would pass just as
//! happily if the ABI said something else.
//!
//! What that phrasing does NOT mean, spelled out because an earlier wording of
//! this header implied a coupling that does not exist. `ABI_WORD`,
//! `ABI_ADDRESS_OFFSET` and `ABI_EPOCH_OFFSET` below are `const`s of THIS
//! FILE. Nothing in `src` feeds them: they are a hand-mirrored record of the
//! regions `unpack_address_topic` and `decode_uint256_to_u64` read, kept in
//! step with `src/decode/v_paseo_ah.rs` by review and not by the compiler.
//!
//! So this target gates the four WIDTHS, and reds on each of them — a
//! `#[repr(align(N))]` on a newtype widens `size_of` past the ABI field
//! without changing the inner array's type, and was watched failing once per
//! width (32/20, 64/32, 64/32, 16/8). It does NOT gate the decoder's offsets.
//! Measured, not assumed: shifting both regions in `v_paseo_ah.rs` while
//! keeping their widths leaves this target green at 2 passed, and reds
//! `tests/decode_org_state.rs` at 4 of 9 instead. The offsets are covered
//! there, by a target that decodes real blobs; what this file records about
//! them is documentation. Coupling the two would mean exporting the offsets
//! from `src` — a production change, deliberately not made in the hazard
//! analysis that found this, and proposed there instead (see
//! `on-chain-client/docs/risk/`, RC-sxjnx9 and not-minted control 17).
//!
//! **Where the over-wide-epoch refusal is gated, and where it is not.** Until
//! review round 4's finding 3 the epoch bullet above cited two targets; only
//! one of them gates the refusal. Measured: `DecodeError::EpochOverflow` is
//! asserted in exactly one place in this repository —
//! `tests/decode_org_state.rs`, in
//! `a_non_zero_byte_anywhere_in_the_epoch_slots_leading_twenty_four_bytes_is_refused`.
//! `tests/decode_revive_event.rs` has no over-wide-epoch case at all: every
//! epoch it builds comes from the `uint256_be(u64)` helper in
//! `tests/fuzz_support/mod.rs`, which writes eight big-endian bytes into
//! `out[24..32]` and leaves the high 24 zero, so it cannot construct the word
//! that would trip the guard.
//!
//! The fact the correction exposes is worth more than the citation. The EVENT
//! path's epoch runs through the same `decode_uint256_to_u64` — `parse_root_updated`
//! in `src/decode/v_paseo_ah.rs`, over `topics[2]` — and REQ-9wwenn is worded
//! generally, over "an Epoch whose on-chain value does not fit the range this
//! reader represents". So the requirement is stated over both paths, and only
//! the STORAGE path has a gated case of its own.
//!
//! That is a citation defect, not a coverage hole, and the reason is measured
//! rather than argued: both paths call the one helper, so a real weakening of
//! the guard reds `decode_org_state`. On 2026-09-11, with `bytes[..24]`
//! narrowed to `bytes[..0]` in `decode_uint256_to_u64` — the guard switched
//! off while every other line stands — `--test decode_org_state` reported 8
//! passed, 1 failed, the failure being the named case above. An over-wide case
//! added on the event path would be a second copy of that evidence rather than
//! a second piece of it. The one thing it would add is narrow: that the
//! REFUSAL, and not only the narrowing, is reachable from the event path.
//! That `parse_root_updated` reads `topics[2]` through the helper at all is
//! already gated, by `root_updated_round_trips_every_field` in
//! `tests/decode_revive_event.rs`: a `uint256_be(42)` topic must come out as
//! `Epoch(42)`.
//!
//! `epoch_display_is_the_inner_value` relocates WITHOUT a `verifies:`
//! annotation, deliberately. `Display` for `Epoch` is used in error messages
//! and logs and no requirement states it; requirements need tests, tests do
//! not need requirements, and inventing an ID for a smoke test would put a
//! claim in the ledger that nothing in this system relies on.
//!
//! Deliberately carries no `#![cfg(...)]` guard, like `log_ownership.rs`: the
//! target is declared in `Cargo.toml` with `required-features =
//! ["test-support"]`, so an explicit `--test type_widths` with the feature off
//! is refused by cargo rather than compiled down to an empty binary that
//! reports success having run nothing.

use on_chain_client::{Epoch, OnChainRootHash, OrgAdmin, OrgPubKey};

/// Every value in the `OrgRegistry` ABI — indexed or not — occupies one
/// 32-byte word.
const ABI_WORD: usize = 32;
/// Where `address` starts inside its word. A record, mirrored BY HAND, of
/// `unpack_address_topic`, which reads `topic[12..32]` and rejects a non-zero
/// prefix. Nothing in `src` feeds this value — see the header.
const ABI_ADDRESS_OFFSET: usize = 12;
/// Where the decoder starts reading `uint256 epoch`. A record, mirrored BY
/// HAND, of `decode_uint256_to_u64`, which reads `bytes[24..32]` and rejects a
/// non-zero prefix. Nothing in `src` feeds this value — see the header.
const ABI_EPOCH_OFFSET: usize = 24;

/// A word with no repeated structure at the boundaries, so a wrapper that is
/// one byte too wide or too narrow cannot convert by accident.
fn word() -> [u8; ABI_WORD] {
    core::array::from_fn(|i| 0x40u8.wrapping_add(i as u8))
}

/// verifies: REQ-2qa5r5
///
/// Normal case, and the whole of the requirement: each public newtype is
/// exactly as wide as the ABI field it wraps. Stated twice over for each —
/// once as the arithmetic over this file's own record of the ABI word and the
/// decoder's offset, and once by converting that exact region of a real word
/// into the newtype, which no other width admits. Both statements are about
/// the WIDTH; neither reaches the offsets themselves, which live in `src` and
/// are gated by `tests/decode_org_state.rs`. See the header.
#[test]
fn newtypes_have_the_widths_the_abi_gives_them() {
    let word = word();

    // `address indexed admin`: the low 20 bytes of the word.
    assert_eq!(
        ABI_WORD - ABI_ADDRESS_OFFSET,
        20,
        "the ABI's address occupies the low 20 bytes of its 32-byte word",
    );
    assert_eq!(
        core::mem::size_of::<OrgAdmin>(),
        ABI_WORD - ABI_ADDRESS_OFFSET,
        "OrgAdmin must be exactly the ABI address, with nothing else in it",
    );
    let admin: [u8; 20] = word[ABI_ADDRESS_OFFSET..]
        .try_into()
        .expect("the address region of a word must be exactly OrgAdmin's width");
    assert_eq!(
        OrgAdmin(admin).0.len(),
        ABI_WORD - ABI_ADDRESS_OFFSET,
        "OrgAdmin must wrap the address region and no padding",
    );

    // `bytes32 rootHash` / `bytes32 orgPubKey` / `bytes32 prevRootHash`: the
    // whole word, copied wholesale. Two distinct newtypes of the same width,
    // which is precisely why they are newtypes and not both `[u8; 32]`.
    for (name, len) in [
        ("OnChainRootHash", OnChainRootHash(word).0.len()),
        ("OrgPubKey", OrgPubKey(word).0.len()),
    ] {
        assert_eq!(len, ABI_WORD, "{name} must be the full 32-byte ABI word");
    }
    assert_eq!(core::mem::size_of::<OnChainRootHash>(), ABI_WORD);
    assert_eq!(core::mem::size_of::<OrgPubKey>(), ABI_WORD);

    // `uint256 indexed epoch`, narrowed by the decoder to the low 8 bytes.
    assert_eq!(
        ABI_WORD - ABI_EPOCH_OFFSET,
        core::mem::size_of::<u64>(),
        "the decoder narrows the ABI's uint256 epoch to its low 8 bytes",
    );
    assert_eq!(
        core::mem::size_of::<Epoch>(),
        ABI_WORD - ABI_EPOCH_OFFSET,
        "Epoch must be exactly the region decode_uint256_to_u64 reads",
    );
    let low: [u8; 8] = word[ABI_EPOCH_OFFSET..]
        .try_into()
        .expect("the epoch region of a word must be exactly Epoch's width");
    assert_eq!(
        Epoch(u64::from_be_bytes(low)).0.to_be_bytes().len(),
        ABI_WORD - ABI_EPOCH_OFFSET,
        "Epoch must round-trip the low word big-endian, losing no byte",
    );
}

/// Smoke test, carrying no `verifies:` annotation on purpose — see the header.
/// The `Display` impl is used in error messages and logs, where an `Epoch`
/// that rendered as `Epoch(42)` or as a hash would make a log unreadable.
#[test]
fn epoch_display_is_the_inner_value() {
    let e = Epoch(42);
    assert_eq!(format!("{e}"), "42");
}
