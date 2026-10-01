//! The Solidity storage-slot derivation — how this unit turns an
//! organisation's admin address into the three EVM storage keys it reads
//! `OrgState` out of. Two pure functions, reached through
//! `on_chain_client::test_support`:
//!
//!   - `solidity_mapping_slot(admin, map_slot)` — the mapping key,
//!     `keccak256(pad12 || admin || u256_be(map_slot))` (REQ-9m2rnd);
//!   - `increment_slot(&mut slot, offset)` — big-endian `+ offset`, which is
//!     how the struct's consecutive fields at `S`, `S + 1`, `S + 2` are
//!     addressed (REQ-xudf25).
//!
//! Both requirements implement RC-5ejucb. `get_org_state` calls
//! `solidity_mapping_slot(admin, 0)` once and then `increment_slot` for
//! offsets 0, 1 and 2. **A wrong key does not fail loudly**: it reads a slot
//! that was never written, which the client reports as `Ok(None)` — "no such
//! organisation". So a derivation defect is indistinguishable, at the client's
//! surface, from an organisation that does not exist. That is why these two
//! functions are pinned rather than merely reviewed.
//!
//! **The layout they assume is confirmed against the contract itself.**
//! `on-chain/src/OrgRegistry.sol` declares `mapping(address => OrgState)
//! private orgs` as the contract's ONLY state variable, so it occupies slot 0
//! — which is what makes `solidity_mapping_slot(admin, 0)` right — and its
//! three-field struct (`bytes32 rootHash`, `bytes32 orgPubKey`,
//! `uint256 epoch`) occupies `S`, `S + 1`, `S + 2`, which is what makes the
//! three `increment_slot` offsets right. Being `private`, `orgs` has no
//! getter, which is why the slot has to be read directly at all.
//!
//! That confirmation is by reading, not by a gate. `on-chain/` is listed under
//! `not_a_unit:` in `.guardrails/units.yaml` — deliberately outside
//! compliance, a later tooth — and its forge tests
//! (`on-chain/test/OrgRegistry.t.sol`) run in no CI lane in this repository:
//! the workflow directory holds `rust.yml` and `quint.yml`, and neither
//! mentions `forge`. A later contract revision could declare a state variable
//! ahead of `orgs`, or reorder the struct, and nothing gated here would
//! notice. These tests pin *our* arithmetic against an independent
//! recomputation of the Solidity layout rule; they cannot pin it against the
//! deployed bytecode.
//!
//! **What this target does NOT establish.** The only test that reads a real
//! contract's real storage through this derivation is
//! `on-chain-client/tests/scenario_a_full.rs`, which asserts `get_org_state`
//! returns the genesis state written by an actual `OrgRegistry` deployment —
//! and it is `#![cfg(feature = "dev-rpc")]` against a live chopsticks fork, so
//! it runs in no CI lane and in no `verify_commands` line here. That target is
//! the one that would catch a layout drift; it is ungated, and the residual
//! that leaves belongs to HAZ-v2cmtx — the hazard about the Organisation's
//! slot being computed differently from the one the chain keeps, which is
//! exactly what a layout drift causes. The register carries this residual
//! there, as its "contract half is confirmed by reading a disclaimed file"
//! paragraph and not-minted control 9. This line named HAZ-xd4urb until
//! review round 2's finding 6; that hazard is the uncommitted-observation
//! and reorg one, and has nothing to do with storage layout.
//!
//! Relocated out of `src/client.rs`'s `#[cfg(test)]` module, which no gate
//! reads — `test_paths` for this unit is `on-chain-client/tests` — and joined
//! by the boundary and abnormal cases class C asks for. That module is deleted
//! in the same commit as this file, so the two never both exist; it held
//! nothing but these three tests, and the `use tiny_keccak::{Hasher, Keccak}`
//! it needed goes with it. `internals` keeps its own separate `tiny_keccak`
//! import, which `solidity_mapping_slot` still uses.
//!
//! Every keccak expectation is recomputed here with `tiny-keccak` (a
//! dev-dependency, so the recomputation does not route through anything under
//! `src`), and every slot-arithmetic expectation is either written out as a
//! literal array or recomputed by `be_add`, which adds over two `u128` limbs
//! rather than the byte-at-a-time loop `increment_slot` uses.
//!
//! verifies: REQ-9m2rnd, REQ-xudf25
//!
//! Deliberately carries no `#![cfg(...)]` guard, unlike the chain-dependent
//! targets beside it: the `test-support` feature this target needs is declared
//! as a `required-features` entry in `Cargo.toml`, so an explicit
//! `--test storage_slot_layout` with the feature off is refused by cargo
//! rather than compiled down to an empty binary that reports success having
//! run nothing.

use on_chain_client::OrgAdmin;
use on_chain_client::test_support::{increment_slot, solidity_mapping_slot};
use tiny_keccak::{Hasher, Keccak};

/// `keccak256(bytes)`, recomputed here rather than taken from anything under
/// `src`.
fn keccak(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    let mut out = [0u8; 32];
    hasher.finalize(&mut out);
    out
}

/// The Solidity mapping-key preimage as this file reads the ABI:
/// `abi.encode(uint256(uint160(admin)), uint256(map_slot))` — the address
/// left-padded into the first word's low twenty bytes, the slot index
/// big-endian in the second word. Written out independently of `src`'s own
/// buffer construction.
fn mapping_key_preimage(admin: &[u8; 20], map_slot: u64) -> [u8; 64] {
    let mut buf = [0u8; 64];
    buf[12..32].copy_from_slice(admin);
    buf[56..64].copy_from_slice(&map_slot.to_be_bytes());
    buf
}

/// The expected mapping key for `orgs[admin]` at mapping slot `map_slot`.
fn expected_mapping_slot(admin: &[u8; 20], map_slot: u64) -> [u8; 32] {
    keccak(&mapping_key_preimage(admin, map_slot))
}

/// A 256-bit big-endian `+ offset`, computed over two `u128` limbs. A
/// deliberately different implementation from `increment_slot`'s
/// byte-at-a-time carry loop, so an expectation computed here is not the same
/// algorithm restated.
fn be_add(slot: [u8; 32], offset: u8) -> [u8; 32] {
    let mut hi_bytes = [0u8; 16];
    let mut lo_bytes = [0u8; 16];
    hi_bytes.copy_from_slice(&slot[..16]);
    lo_bytes.copy_from_slice(&slot[16..]);
    let hi = u128::from_be_bytes(hi_bytes);
    let lo = u128::from_be_bytes(lo_bytes);

    let (lo_sum, carried) = lo.overflowing_add(u128::from(offset));
    let hi_sum = if carried { hi.wrapping_add(1) } else { hi };

    let mut out = [0u8; 32];
    out[..16].copy_from_slice(&hi_sum.to_be_bytes());
    out[16..].copy_from_slice(&lo_sum.to_be_bytes());
    out
}

/// The admin from the relocated reference vector.
const ADMIN_ONES: [u8; 20] = [0x11; 20];

// ---------------------------------------------------------------------------
// REQ-9m2rnd — the mapping key
// ---------------------------------------------------------------------------

// verifies: REQ-9m2rnd, LLR-vktf8w
//
// The relocated reference vector, normal case: a 20-byte admin of all `0x11`
// at mapping slot 0. The expectation is `keccak256(0x00 * 12 || 0x11 * 20 ||
// 0x00 * 32)`, recomputed in this file. The two `assert_ne!`s are what stop
// the `assert_eq!` from also holding for a plausibly mis-encoded preimage:
// right-padding the address instead of left-padding it, and hashing the bare
// twenty bytes with no padding at all, are the two encodings a reader of the
// Solidity ABI could reach for by mistake.
#[test]
fn mapping_slot_matches_the_known_vector() {
    let got = solidity_mapping_slot(OrgAdmin(ADMIN_ONES), 0);

    assert_eq!(got, expected_mapping_slot(&ADMIN_ONES, 0));

    let mut right_padded = [0u8; 64];
    right_padded[..20].copy_from_slice(&ADMIN_ONES);
    assert_ne!(
        got,
        keccak(&right_padded),
        "the address must be LEFT-padded into the first word"
    );
    assert_ne!(
        got,
        keccak(&ADMIN_ONES),
        "the preimage must be two 32-byte words, not the bare address"
    );
}

// verifies: REQ-9m2rnd, LLR-vktf8w
//
// A second admin, so the vector above cannot be satisfied by a function that
// ignores its argument and returns one constant. This admin's twenty bytes are
// all distinct, which also means a mis-ordered (reversed) copy of the address
// would change the answer.
#[test]
fn mapping_slot_for_a_second_admin_is_its_own_key() {
    let mut admin = [0u8; 20];
    for (i, byte) in admin.iter_mut().enumerate() {
        *byte = i as u8;
    }

    let got = solidity_mapping_slot(OrgAdmin(admin), 0);

    assert_eq!(got, expected_mapping_slot(&admin, 0));
    assert_ne!(
        got,
        solidity_mapping_slot(OrgAdmin(ADMIN_ONES), 0),
        "two admins must not share a slot key"
    );

    let mut reversed = admin;
    reversed.reverse();
    assert_ne!(
        got,
        expected_mapping_slot(&reversed, 0),
        "the address bytes must be copied in order"
    );
}

// verifies: REQ-9m2rnd, LLR-62tqnv
//
// The formula's second operand. `get_org_state` only ever passes 0 — `orgs` is
// the contract's only state variable — so a function that dropped `map_slot`
// entirely would still serve this unit correctly today and would break
// silently the moment a second state variable were declared ahead of it.
// Non-zero indices are therefore exercised here rather than assumed.
#[test]
fn a_non_zero_mapping_slot_index_is_part_of_the_key() {
    let at_zero = solidity_mapping_slot(OrgAdmin(ADMIN_ONES), 0);
    let at_one = solidity_mapping_slot(OrgAdmin(ADMIN_ONES), 1);
    let at_seven = solidity_mapping_slot(OrgAdmin(ADMIN_ONES), 7);

    assert_eq!(at_one, expected_mapping_slot(&ADMIN_ONES, 1));
    assert_eq!(at_seven, expected_mapping_slot(&ADMIN_ONES, 7));

    assert_ne!(at_one, at_zero, "mapping slot 1 must not collide with 0");
    assert_ne!(at_seven, at_zero, "mapping slot 7 must not collide with 0");
    assert_ne!(at_seven, at_one, "mapping slot 7 must not collide with 1");
}

// verifies: REQ-9m2rnd, LLR-62tqnv
//
// Boundary: the widest mapping-slot index the signature can express. Solidity's
// mapping key is `uint256`, but `solidity_mapping_slot` takes a `u64` and puts
// it in the second word's LOW eight bytes — bytes [56..64] — so indices above
// `u64::MAX` are unrepresentable. That is not a live limit (the only mapping in
// the contract is at 0) but it is a real narrowing of the ABI, recorded here
// rather than left to be discovered.
//
// `u64::MAX` tells the correct placement apart from the mistake a `u64`
// standing in for a `uint256` invites: writing the eight bytes at the second
// word's HIGH end, [32..40], where a `uint64` field would sit in a packed
// struct but a `uint256` mapping index never does. Byte order is pinned
// separately, with 1 — all-ones is byte-order-symmetric and can pin nothing.
#[test]
fn the_mapping_slot_index_is_big_endian_in_the_low_eight_bytes_of_the_second_word() {
    let got = solidity_mapping_slot(OrgAdmin(ADMIN_ONES), u64::MAX);

    assert_eq!(got, expected_mapping_slot(&ADMIN_ONES, u64::MAX));

    let mut high_eight = mapping_key_preimage(&ADMIN_ONES, 0);
    high_eight[32..40].copy_from_slice(&u64::MAX.to_be_bytes());
    assert_ne!(
        got,
        keccak(&high_eight),
        "the slot index belongs in the second word's LOW eight bytes"
    );

    let mut little_endian_one = mapping_key_preimage(&ADMIN_ONES, 0);
    little_endian_one[56..64].copy_from_slice(&1u64.to_le_bytes());
    assert_ne!(
        solidity_mapping_slot(OrgAdmin(ADMIN_ONES), 1),
        keccak(&little_endian_one),
        "the slot index must be big-endian"
    );
}

// verifies: REQ-9m2rnd, LLR-vktf8w
//
// Abnormal input: the zero address. `OrgAdmin([0; 20])` is not a real
// organisation — no account maps to it — but the derivation must still be the
// keccak of the padded preimage, not a short-circuit to the zero slot. A
// zero-slot short-circuit would put the zero admin on top of the mapping's own
// declaration slot.
#[test]
fn the_zero_admin_is_hashed_like_any_other_address() {
    let zero = [0u8; 20];

    let got = solidity_mapping_slot(OrgAdmin(zero), 0);

    assert_eq!(got, expected_mapping_slot(&zero, 0));
    assert_eq!(got, keccak(&[0u8; 64]), "the preimage is 64 zero bytes");
    assert_ne!(got, [0u8; 32], "a zero admin must not derive the zero slot");
}

// ---------------------------------------------------------------------------
// REQ-xudf25 — the struct-field offset
// ---------------------------------------------------------------------------

// verifies: REQ-xudf25, LLR-bhwsn6
//
// Relocated. Offset 0 is the identity: the first struct field lives at `S`
// itself, so `get_org_state`'s loop over 0..3 must leave the base key alone on
// its first pass. The pattern is deliberately not zero, so an implementation
// that zeroed the slot would be caught.
#[test]
fn offset_zero_is_the_identity() {
    let mut slot = [0x42; 32];

    increment_slot(&mut slot, 0);

    assert_eq!(slot, [0x42; 32]);
    assert_eq!(slot, be_add([0x42; 32], 0));
}

// verifies: REQ-xudf25, LLR-bhwsn6
//
// Relocated. The single-byte carry: `0xfe + 3 = 0x101`, so the low byte
// becomes `0x01` and one carries into byte 30. Expectation written out as a
// literal array.
#[test]
fn offset_three_carries_out_of_the_low_byte() {
    let mut slot = [0u8; 32];
    slot[31] = 0xfe;

    increment_slot(&mut slot, 3);

    let mut expected = [0u8; 32];
    expected[30] = 0x01;
    expected[31] = 0x01;
    assert_eq!(slot, expected);
}

// verifies: REQ-xudf25, LLR-bhwsn6
//
// The carry has to propagate, not merely happen once. Four `0xff` bytes with a
// zero above them: `+ 1` must clear all four and set the fifth. A keccak output
// is uniformly random, so a base key ending in a run of `0xff` bytes is not a
// contrived case — a run this long arrives with probability 2^-32 per
// organisation, and the consequence is that the client reads unrelated slots
// and reports `Ok(None)` for an organisation that exists.
#[test]
fn a_carry_propagates_through_four_bytes() {
    let mut slot = [0u8; 32];
    for byte in slot.iter_mut().skip(28) {
        *byte = 0xff;
    }
    let before = slot;

    increment_slot(&mut slot, 1);

    let mut expected = [0u8; 32];
    expected[27] = 0x01;
    assert_eq!(slot, expected, "four 0xff bytes must all clear");
    assert_eq!(slot, be_add(before, 1));
}

// verifies: REQ-xudf25, LLR-bhwsn6
//
// Boundary: 255 is the widest offset a `u8` can ask for, tested both where it
// does not carry and where it does. The second case pins the full-width
// addition — `0x01 + 0xff` overflows a `u8`, so an implementation that added
// into a `u8` instead of a `u16` would wrap and lose the carry.
#[test]
fn offset_255_is_the_widest_a_u8_can_ask_for() {
    let mut no_carry = [0u8; 32];
    increment_slot(&mut no_carry, 255);
    let mut expected_no_carry = [0u8; 32];
    expected_no_carry[31] = 0xff;
    assert_eq!(no_carry, expected_no_carry);
    assert_eq!(no_carry, be_add([0u8; 32], 255));

    let mut carries = [0u8; 32];
    carries[31] = 0x01;
    let before = carries;
    increment_slot(&mut carries, 255);
    let mut expected_carry = [0u8; 32];
    expected_carry[30] = 0x01;
    expected_carry[31] = 0x00;
    assert_eq!(carries, expected_carry, "0x01 + 0xff must carry");
    assert_eq!(carries, be_add(before, 255));
}

// verifies: REQ-xudf25, LLR-2y9qdc
//
// Abnormal input, and the one case the function's own doc-comment calls
// unreachable in practice: an all-ones slot key. `+ 1` wraps the whole 256-bit
// value to zero. Reaching it needs a keccak output of thirty-two `0xff` bytes,
// so "unreachable" is fair — but the behaviour is asserted rather than left
// undefined, because in a crate whose root denies `panic` the alternative to
// wrapping is a panic on data derived from untrusted input.
#[test]
fn an_all_ones_slot_wraps_to_zero() {
    let mut slot = [0xff; 32];

    increment_slot(&mut slot, 1);

    assert_eq!(slot, [0u8; 32], "256-bit wrap-around is silent");
    assert_eq!(slot, be_add([0xff; 32], 1));
}

// ---------------------------------------------------------------------------
// REQ-9m2rnd + REQ-xudf25 — the two composed, as `get_org_state` composes them
// ---------------------------------------------------------------------------

// verifies: REQ-9m2rnd, REQ-xudf25, LLR-v62yjq
//
// The three keys `get_org_state` actually reads, derived the way it derives
// them: one `solidity_mapping_slot(admin, 0)` and then offsets 0, 1, 2. They
// must be three consecutive, pairwise-distinct keys — consecutive because
// `OrgState`'s three fields sit at `S`, `S + 1`, `S + 2`, distinct because two
// equal keys would read one field's bytes into another field's position and
// hand the decoder a plausible-looking 96-byte blob.
#[test]
fn the_three_struct_field_slots_are_consecutive_and_distinct() {
    let admin = OrgAdmin(ADMIN_ONES);
    let base = solidity_mapping_slot(admin, 0);

    let mut keys = [[0u8; 32]; 3];
    for (offset, key) in keys.iter_mut().enumerate() {
        let mut slot = base;
        increment_slot(&mut slot, offset as u8);
        *key = slot;
    }

    assert_eq!(keys[0], base, "rootHash sits at S itself");
    assert_eq!(keys[1], be_add(base, 1), "orgPubKey sits at S + 1");
    assert_eq!(keys[2], be_add(base, 2), "epoch sits at S + 2");

    assert_ne!(keys[0], keys[1]);
    assert_ne!(keys[1], keys[2]);
    assert_ne!(keys[0], keys[2]);
}
