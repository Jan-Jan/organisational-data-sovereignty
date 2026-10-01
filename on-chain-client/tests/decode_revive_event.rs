#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `Decoder::parse_revive_event` — the shape checks that stand between a
//! `pallet_revive::Event::ContractEmitted` payload carried on the follow
//! stream and a typed `Event` every consumer then treats as the chain's
//! word on an organisation's membership root.
//!
//! Five requirements are gated here, all five reached through the public
//! `decode::dispatch::for_runtime` rather than through anything in `src`
//! that only the crate can see:
//!
//!   - REQ-52uc8f — the ABI binding: the values this decoder recognises as
//!     event signatures are exactly the keccak of the contract's two
//!     canonical Solidity signature strings, and no other, so a rename or a
//!     reorder in the contract cannot go unnoticed.
//!   - REQ-wnjz9j — a first topic matching no signature this decoder knows,
//!     and a log with no topics at all, yield nothing rather than an event.
//!   - REQ-88fp2h — topic count and `data` length must match the ABI for the
//!     matched signature exactly, and the error must name both the expected
//!     and the actual count, because that number is what tells an operator
//!     which shape the chain actually sent.
//!   - REQ-twdu84 — an indexed `address` topic is a 20-byte address
//!     left-padded to 32 with zeroes; a non-zero byte anywhere in that
//!     twelve-byte padding means the log was not written by the Solidity ABI
//!     and must be refused rather than truncated into some other address.
//!   - REQ-axcxf7 — the payload must be consumed exactly: neither a trailing
//!     byte after a well-formed payload, nor a payload that runs out inside
//!     its own `topics`, nor one that ends inside a field's own compact
//!     length prefix may be accepted, while a payload framed exactly as it
//!     declares must be — the accepting side is carried by the two round-trip
//!     tests, which decode canonical encodings with nothing left over.
//!
//! REQ-52uc8f implements RC-675a3h; REQ-wnjz9j, REQ-88fp2h and REQ-twdu84
//! implement RC-6gfh8d; REQ-axcxf7 implements RC-8w9wtp.
//!
//! **Independent derivation.** Every signature hash here is recomputed from
//! the Solidity canonical signature string by `tests/fuzz_support/mod.rs`'s
//! `sig_genesis()` / `sig_root_updated()`, never imported from the decoder's
//! own `pub(super)` constants — which is what keeps the ABI-drift check
//! honest. The payloads are built by that module's `encode_contract_emitted`,
//! which encodes the `ContractEmitted` fields from the pallet's own field
//! order rather than by asking the decoder to encode anything, so a round
//! trip through it is a cross-check and not a tautology.
//!
//! Relocated out of the `#[cfg(test)]` module in
//! `on-chain-client/src/decode/v_paseo_ah.rs`, which no gate reads
//! (`test_paths` for this unit is `on-chain-client/tests`), and joined by the
//! abnormal-input cases class C asks for. The relocated assertions are kept
//! verbatim where they stand alone and are absorbed, position for position,
//! where a sweep now covers them — noted at each such test.
//!
//! **What this file does NOT establish.** It says nothing about *which
//! contract* emitted a log. A log decoded here is well-formed, not
//! trustworthy: any contract on Asset Hub can emit these signature hashes
//! with a victim's indexed admin. That decision is the caller's, on the
//! address `EmittedEvent::contract` carries, and it is gated in
//! `on-chain-client/tests/log_ownership.rs` (REQ-9vwcwc, HAZ-werm85) — the
//! target that holds the decision itself, `log_is_ours`, with its boundaries.
//! `tests/contract_address_filter.rs` is the *decoder's* half (REQ-5upq6n,
//! that the address is reported byte for byte) and its own header explicitly
//! disclaims carrying REQ-9vwcwc; this line pointed there until review round
//! 2's finding 6, which sent a reader to the file that says it does not
//! verify the thing.

use on_chain_client::decode::dispatch::{PASEO_AH_SPEC_VERSION, for_runtime};
use on_chain_client::decode::{DecodeError, Decoder};
use on_chain_client::state::EmittedEvent;
use on_chain_client::{Epoch, Event, OnChainRootHash, OrgAdmin, OrgPubKey};

#[path = "fuzz_support/mod.rs"]
mod support;

use support::{encode_contract_emitted, padded_address, sig_genesis, sig_root_updated, uint256_be};

/// The emitting contract. Irrelevant to every check in this file — it is
/// carried through untouched and compared by the caller, not here — so one
/// constant serves throughout.
const CONTRACT: [u8; 20] = [0x55; 20];

/// The decoder under test, reached the way production reaches it.
fn decoder() -> &'static dyn Decoder {
    for_runtime(PASEO_AH_SPEC_VERSION).expect("the pinned runtime has a compiled-in decoder")
}

fn parse(bytes: &[u8]) -> Result<Option<EmittedEvent>, DecodeError> {
    decoder().parse_revive_event(bytes)
}

/// `GenesisInitialized`'s non-indexed half: `rootHash || orgPubKey`, 64 bytes.
fn genesis_data(root_hash: [u8; 32], org_pub_key: [u8; 32]) -> Vec<u8> {
    let mut data = Vec::with_capacity(64);
    data.extend_from_slice(&root_hash);
    data.extend_from_slice(&org_pub_key);
    data
}

/// `RootUpdated`'s non-indexed half: `rootHash || orgPubKey || prevRootHash`,
/// 96 bytes.
fn root_updated_data(
    root_hash: [u8; 32],
    org_pub_key: [u8; 32],
    prev_root_hash: [u8; 32],
) -> Vec<u8> {
    let mut data = Vec::with_capacity(96);
    data.extend_from_slice(&root_hash);
    data.extend_from_slice(&org_pub_key);
    data.extend_from_slice(&prev_root_hash);
    data
}

/// A rejection has to be *legible*: the operator reading the log needs the
/// shape the ABI requires and the shape the chain actually sent, both. A
/// variant that carried only one of the two would leave them guessing.
fn assert_error_names(err: &DecodeError, expected: usize, actual: usize) {
    let msg = err.to_string();
    assert!(
        msg.contains(&expected.to_string()),
        "error must name the expected count {expected}: {msg}"
    );
    assert!(
        msg.contains(&actual.to_string()),
        "error must name the actual count {actual}: {msg}"
    );
}

// ---------------------------------------------------------------------------
// REQ-52uc8f — which signature hashes are ours
// REQ-wnjz9j — and what a log carrying any other first topic does
// ---------------------------------------------------------------------------

// verifies: REQ-52uc8f, REQ-wnjz9j, LLR-e6skvu
//
// The ABI-drift guard, relocated from `src`'s
// `event_signatures_match_solidity_abi` and re-expressed as behaviour rather
// than as equality against a constant the test can no longer see. The
// decoder's compiled-in signature hashes are private to the crate, so the
// question is asked of the decoder instead: a log whose first topic is the
// keccak of the contract's canonical signature string must decode, and a log
// whose first topic is the keccak of a *plausibly drifted* string — one
// parameter dropped from each event — must not.
//
// **REQ-52uc8f** — recognise exactly these two values and no other — is what
// the two halves say together. The accepting half is the first: each canonical
// Solidity signature string is written out here as a literal, its keccak is
// recomputed, the two `assert_eq!`s pin `sig_genesis()` / `sig_root_updated()`
// to those literals, and a log carrying that hash in `topics[0]` must decode.
// So the decoder is shown to recognise those two values, named the way the
// requirement names them. If anyone renames or reorders a parameter in the
// Solidity source, the canonical string moves, this derivation moves with it,
// and this half fails before any decoder logic gets a chance to silently
// mismatch real events.
//
// The rejecting half carries "and no other value" — and it is equally
// **REQ-wnjz9j**'s property, since what it asserts is `Ok(None)`: no event and
// no error, not a refusal. "No other value" cannot be swept over 2^256, so it
// is sampled where drift actually occurs: one parameter dropped from each
// event, which is the nearest plausible neighbour of a hash we do accept.
// `an_unknown_first_topic_yields_nothing` below samples it far away instead,
// at `[0xff; 32]`.
//
// That rejecting half is also REQ-52uc8f's abnormal-input case, and no further
// one is owed. The requirement is a recognise-exactly-these rule, so its
// abnormal input *is* a first topic that is not one of the two, and the
// drifted strings are that input at its most adversarial — a value derived
// from the same source by a single plausible edit. Malformed payloads behind a
// *known* signature belong to REQ-88fp2h, REQ-twdu84 and REQ-axcxf7 below;
// arbitrary bytes belong to REQ-sx5b6g's bolero target.
//
// What this does NOT establish: that `on-chain/src/OrgRegistry.sol` still
// declares these two events with these parameter lists. The strings here are
// literals, and REQ-52uc8f names the same two literals in its own text, so the
// test does verify the requirement as written — but nothing gated compares
// either against the Solidity source, which is another unit. That link is held
// by review across the unit boundary, not by this test.
#[test]
fn the_recognised_signature_hashes_are_the_keccak_of_the_canonical_solidity_strings() {
    use tiny_keccak::{Hasher, Keccak};

    fn keccak(s: &str) -> [u8; 32] {
        let mut h = Keccak::v256();
        h.update(s.as_bytes());
        let mut out = [0u8; 32];
        h.finalize(&mut out);
        out
    }

    // The canonical strings, written out here rather than imported.
    assert_eq!(
        sig_genesis(),
        keccak("GenesisInitialized(address,bytes32,bytes32)"),
        "the test's own derivation must be the canonical GenesisInitialized string"
    );
    assert_eq!(
        sig_root_updated(),
        keccak("RootUpdated(address,uint256,bytes32,bytes32,bytes32)"),
        "the test's own derivation must be the canonical RootUpdated string"
    );

    // The decoder recognises exactly those two hashes.
    let genesis = encode_contract_emitted(
        CONTRACT,
        genesis_data([0xaa; 32], [0xbb; 32]),
        vec![sig_genesis(), padded_address([0x11; 20])],
    );
    assert!(
        matches!(parse(&genesis), Ok(Some(_))),
        "a log topic-hashed from the canonical GenesisInitialized string must decode"
    );
    let updated = encode_contract_emitted(
        CONTRACT,
        root_updated_data([0xcc; 32], [0xdd; 32], [0xee; 32]),
        vec![sig_root_updated(), padded_address([0x22; 20]), uint256_be(1)],
    );
    assert!(
        matches!(parse(&updated), Ok(Some(_))),
        "a log topic-hashed from the canonical RootUpdated string must decode"
    );

    // And it recognises no neighbouring string: a dropped parameter is the
    // drift this guard exists to catch, and it must not still match.
    for drifted in [
        "GenesisInitialized(address,bytes32)",
        "RootUpdated(address,uint256,bytes32,bytes32)",
    ] {
        let bytes = encode_contract_emitted(
            CONTRACT,
            genesis_data([0xaa; 32], [0xbb; 32]),
            vec![keccak(drifted), padded_address([0x11; 20])],
        );
        assert_eq!(
            parse(&bytes),
            Ok(None),
            "`{drifted}` is not a signature this decoder knows and must not match one"
        );
    }
}

// verifies: REQ-88fp2h, REQ-twdu84, REQ-axcxf7, LLR-6tjhgk, LLR-u2e389
//
// REQ-wnjz9j is deliberately NOT among those IDs (review round 1, finding 9).
// It states only a refusal — an unknown signature yields no event — so it has
// no accepting side these two could fail on: a decoder that returned `Err` for
// every unknown signature would leave them green. Its three dedicated cases
// below are where it is gated.
//
// The normal case for all three shape checks at once: a well-formed
// `GenesisInitialized` — known signature, two topics, sixty-four bytes of
// data, zero padding in the indexed address — decodes, and every field comes
// back where the ABI put it. Relocated from `src`'s
// `parse_genesis_event_round_trip`. Distinctive, non-repeating values, so a
// decoder reading the right number of bytes from the wrong offset cannot
// produce this answer by accident. It is also REQ-axcxf7's normal case: the
// fixture is the canonical encoding of exactly the three `ContractEmitted`
// fields, so accepting it is the decoder taking the branch where nothing is
// left over and nothing ran short — the boundary the trailing-byte and
// truncated-`topics` refusals below sit against.
#[test]
fn genesis_initialized_round_trips_every_field() {
    let admin = [0x11u8; 20];
    let root_hash: [u8; 32] = core::array::from_fn(|i| 0x40 + i as u8);
    let org_pub_key: [u8; 32] = core::array::from_fn(|i| 0x80 + i as u8);

    let bytes = encode_contract_emitted(
        CONTRACT,
        genesis_data(root_hash, org_pub_key),
        vec![sig_genesis(), padded_address(admin)],
    );

    assert_eq!(
        parse(&bytes),
        Ok(Some(EmittedEvent {
            contract: CONTRACT,
            event: Event::Genesis {
                admin: OrgAdmin(admin),
                root_hash: OnChainRootHash(root_hash),
                org_pub_key: OrgPubKey(org_pub_key),
            },
        })),
    );
}

// verifies: REQ-88fp2h, REQ-twdu84, REQ-axcxf7, LLR-6tjhgk, LLR-u2e389
//
// The same for `RootUpdated`, which has one more indexed field (the epoch)
// and one more non-indexed slot (the previous root). Relocated from `src`'s
// `parse_root_updated_event_round_trip`. It carries REQ-axcxf7's normal case
// at the longer of the two payload shapes: three topics and ninety-six data
// bytes, framed exactly as declared, are accepted — so the exact-consumption
// check is not merely tuned to the two-topic fixture the refusals use.
#[test]
fn root_updated_round_trips_every_field() {
    let admin = [0x22u8; 20];
    let root_hash: [u8; 32] = core::array::from_fn(|i| 0x40 + i as u8);
    let org_pub_key: [u8; 32] = core::array::from_fn(|i| 0x80 + i as u8);
    let prev_root_hash: [u8; 32] = core::array::from_fn(|i| 0xc0 + i as u8);

    let bytes = encode_contract_emitted(
        CONTRACT,
        root_updated_data(root_hash, org_pub_key, prev_root_hash),
        vec![sig_root_updated(), padded_address(admin), uint256_be(42)],
    );

    assert_eq!(
        parse(&bytes),
        Ok(Some(EmittedEvent {
            contract: CONTRACT,
            event: Event::Update {
                admin: OrgAdmin(admin),
                epoch: Epoch(42),
                root_hash: OnChainRootHash(root_hash),
                org_pub_key: OrgPubKey(org_pub_key),
                prev_root_hash: OnChainRootHash(prev_root_hash),
            },
        })),
    );
}

// verifies: REQ-wnjz9j, LLR-rjcqg3
//
// An unknown first topic yields nothing. The follow subscription carries
// every contract's logs, so this is the ordinary case, not an error: a future
// event we have not taught the decoder, or any other contract's log. Note
// what this is NOT: a *known* signature from another contract decodes fine
// here and is refused by the caller on the emitting address (HAZ-werm85,
// REQ-9vwcwc, gated in tests/log_ownership.rs — not in
// tests/contract_address_filter.rs, which gates only the decoder's half and
// disclaims REQ-9vwcwc in its own header). Relocated from `src`'s
// `parse_event_with_unknown_signature_returns_none`.
#[test]
fn an_unknown_first_topic_yields_nothing() {
    let bytes = encode_contract_emitted(CONTRACT, Vec::new(), vec![[0xff; 32]]);
    assert_eq!(parse(&bytes), Ok(None));
}

// verifies: REQ-wnjz9j, LLR-rjcqg3
//
// No topics at all yields nothing. pallet-revive emits `ContractEmitted` with
// an empty topic list when a contract calls `log0(data)`; there is no
// signature to match, so there is nothing to decode and nothing to report.
// Relocated from `src`'s `parse_event_empty_topics_returns_none`.
#[test]
fn no_topics_at_all_yields_nothing() {
    let bytes = encode_contract_emitted(CONTRACT, vec![0xde, 0xad], Vec::new());
    assert_eq!(parse(&bytes), Ok(None));
}

// ---------------------------------------------------------------------------
// REQ-88fp2h — topic count and data length must match the ABI exactly
// ---------------------------------------------------------------------------

// verifies: REQ-88fp2h, LLR-n6gghu
//
// One topic short of `GenesisInitialized`'s two: the signature matched but the
// indexed admin is missing, so there is no admin to attribute the root to.
// Relocated from `src`'s `parse_genesis_wrong_topic_count_rejected`, and
// extended to assert the error names both counts.
#[test]
fn genesis_with_one_topic_is_rejected() {
    let bytes = encode_contract_emitted(CONTRACT, vec![0u8; 64], vec![sig_genesis()]);
    let err = parse(&bytes).expect_err("one topic is not two");
    assert_eq!(
        err,
        DecodeError::InvalidTopicCount {
            event: "GenesisInitialized",
            expected: 2,
            actual: 1,
        },
    );
    assert_error_names(&err, 2, 1);
}

// verifies: REQ-88fp2h, LLR-n6gghu
//
// One topic long. The extra topic is not simply ignored: a log with a third
// indexed field is not the event this decoder was written against, and
// reading the first two out of it would be guessing.
#[test]
fn genesis_with_three_topics_is_rejected() {
    let bytes = encode_contract_emitted(
        CONTRACT,
        vec![0u8; 64],
        vec![sig_genesis(), padded_address([0x11; 20]), uint256_be(7)],
    );
    let err = parse(&bytes).expect_err("three topics is not two");
    assert_eq!(
        err,
        DecodeError::InvalidTopicCount {
            event: "GenesisInitialized",
            expected: 2,
            actual: 3,
        },
    );
    assert_error_names(&err, 2, 3);
}

// verifies: REQ-88fp2h, LLR-n6gghu
//
// One topic short of `RootUpdated`'s three: the indexed epoch is missing, and
// the epoch is what orders one root against another. Accepting this would
// mean inventing an epoch.
#[test]
fn root_updated_with_two_topics_is_rejected() {
    let bytes = encode_contract_emitted(
        CONTRACT,
        vec![0u8; 96],
        vec![sig_root_updated(), padded_address([0x22; 20])],
    );
    let err = parse(&bytes).expect_err("two topics is not three");
    assert_eq!(
        err,
        DecodeError::InvalidTopicCount {
            event: "RootUpdated",
            expected: 3,
            actual: 2,
        },
    );
    assert_error_names(&err, 3, 2);
}

// verifies: REQ-88fp2h, LLR-n6gghu
//
// One topic long, the other side of `RootUpdated`'s boundary — the pair of the
// case above, so the check is pinned from both directions rather than only
// from below.
#[test]
fn root_updated_with_four_topics_is_rejected() {
    let bytes = encode_contract_emitted(
        CONTRACT,
        vec![0u8; 96],
        vec![
            sig_root_updated(),
            padded_address([0x22; 20]),
            uint256_be(42),
            [0x01; 32],
        ],
    );
    let err = parse(&bytes).expect_err("four topics is not three");
    assert_eq!(
        err,
        DecodeError::InvalidTopicCount {
            event: "RootUpdated",
            expected: 3,
            actual: 4,
        },
    );
    assert_error_names(&err, 3, 4);
}

// verifies: REQ-88fp2h, LLR-n6gghu
//
// `GenesisInitialized`'s data is exactly two ABI words. Both neighbours of 64
// are refused — one byte short and one byte long, which is where an `<`
// instead of a `!=` hides — and so is the far-off length the relocated
// `src` test used (32 bytes, i.e. one word instead of two). The error names
// both lengths in every case.
#[test]
fn genesis_data_of_any_length_but_sixty_four_is_rejected() {
    for actual in [0usize, 32, 63, 65, 128] {
        let bytes = encode_contract_emitted(
            CONTRACT,
            vec![0u8; actual],
            vec![sig_genesis(), padded_address([0x11; 20])],
        );
        let err = parse(&bytes).unwrap_err();
        assert_eq!(
            err,
            DecodeError::InvalidDataLength {
                event: "GenesisInitialized",
                expected: 64,
                actual,
            },
            "{actual} bytes of data must be refused"
        );
        assert_error_names(&err, 64, actual);
    }
}

// verifies: REQ-88fp2h, LLR-n6gghu
//
// `RootUpdated`'s data is exactly three ABI words. Same both-neighbours
// treatment: 95 and 97 as well as lengths further out. 64 is included
// deliberately — it is `GenesisInitialized`'s valid length, so it is the
// wrong-event-shape confusion this check has to refuse.
#[test]
fn root_updated_data_of_any_length_but_ninety_six_is_rejected() {
    for actual in [0usize, 64, 95, 97, 192] {
        let bytes = encode_contract_emitted(
            CONTRACT,
            vec![0u8; actual],
            vec![sig_root_updated(), padded_address([0x22; 20]), uint256_be(42)],
        );
        let err = parse(&bytes).unwrap_err();
        assert_eq!(
            err,
            DecodeError::InvalidDataLength {
                event: "RootUpdated",
                expected: 96,
                actual,
            },
            "{actual} bytes of data must be refused"
        );
        assert_error_names(&err, 96, actual);
    }
}

// ---------------------------------------------------------------------------
// REQ-twdu84 — the indexed address topic's twelve-byte padding
// ---------------------------------------------------------------------------

/// A non-repeating address, so a decoder reading twenty bytes from the wrong
/// offset inside the topic cannot land on this answer by accident.
const DISTINCT_ADMIN: [u8; 20] = [
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
    0x30, 0x31, 0x32, 0x33,
];

// verifies: REQ-twdu84, LLR-89pdz9
//
// Solidity always zero-pads an indexed `address` on the left, so a non-zero
// byte anywhere in those twelve positions means the log was not written by the
// ABI this decoder decodes. Every one of the twelve positions is swept, one at
// a time, for both event shapes — the two share `unpack_address_topic`, and
// the sweep is what pins the guarded window to bytes 0..12 rather than to any
// eleven-byte subrange of it. The relocated `src` test
// (`parse_event_with_corrupted_address_topic_rejected`) asserted exactly this
// for position 5, and is absorbed here at that position.
#[test]
fn a_non_zero_byte_at_each_padding_position_of_the_address_topic_is_rejected() {
    for pos in 0..12usize {
        let mut topic = padded_address(DISTINCT_ADMIN);
        topic[pos] = 0xff;

        let genesis = encode_contract_emitted(
            CONTRACT,
            genesis_data([0xaa; 32], [0xbb; 32]),
            vec![sig_genesis(), topic],
        );
        assert_eq!(
            parse(&genesis),
            Err(DecodeError::InvalidAddressTopic),
            "GenesisInitialized: padding byte {pos} non-zero must be refused"
        );

        let updated = encode_contract_emitted(
            CONTRACT,
            root_updated_data([0xcc; 32], [0xdd; 32], [0xee; 32]),
            vec![sig_root_updated(), topic, uint256_be(42)],
        );
        assert_eq!(
            parse(&updated),
            Err(DecodeError::InvalidAddressTopic),
            "RootUpdated: padding byte {pos} non-zero must be refused"
        );
    }
}

// verifies: REQ-twdu84, LLR-89pdz9
//
// The accepted side of the same boundary: all twelve padding bytes zero, and
// the address that comes back is the topic's low twenty bytes exactly — which
// is what makes the sweep above meaningful. `DISTINCT_ADMIN` is non-repeating,
// so a window off by one in either direction would produce a different
// address and fail here.
#[test]
fn all_twelve_padding_bytes_zero_is_accepted_and_the_address_is_the_low_twenty_bytes() {
    let topic = padded_address(DISTINCT_ADMIN);
    assert!(
        topic[..12].iter().all(|b| *b == 0),
        "the fixture must have zero padding"
    );

    let genesis = encode_contract_emitted(
        CONTRACT,
        genesis_data([0xaa; 32], [0xbb; 32]),
        vec![sig_genesis(), topic],
    );
    match parse(&genesis) {
        Ok(Some(EmittedEvent {
            event: Event::Genesis { admin, .. },
            ..
        })) => assert_eq!(admin, OrgAdmin(DISTINCT_ADMIN)),
        other => panic!("expected a Genesis event, got {other:?}"),
    }

    let updated = encode_contract_emitted(
        CONTRACT,
        root_updated_data([0xcc; 32], [0xdd; 32], [0xee; 32]),
        vec![sig_root_updated(), topic, uint256_be(42)],
    );
    match parse(&updated) {
        Ok(Some(EmittedEvent {
            event: Event::Update { admin, .. },
            ..
        })) => assert_eq!(admin, OrgAdmin(DISTINCT_ADMIN)),
        other => panic!("expected an Update event, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// REQ-axcxf7 — the payload must be consumed exactly
// ---------------------------------------------------------------------------

// verifies: REQ-axcxf7, LLR-8242kq
//
// One trailing byte after a well-formed payload. SCALE decoding of the three
// fields succeeds and leaves the byte unread, so nothing but an explicit
// check catches it — and an unread tail means the bytes were not the variant
// they were taken for. The error names the number of bytes left over, since
// that is the only clue an operator gets about what the extra bytes were.
// Relocated from `src`'s `parse_event_trailing_bytes_rejected`, extended
// across a run of trailing lengths and to assert the count is reported.
#[test]
fn any_trailing_bytes_after_a_well_formed_payload_are_rejected() {
    let well_formed = encode_contract_emitted(
        CONTRACT,
        genesis_data([0xaa; 32], [0xbb; 32]),
        vec![sig_genesis(), padded_address([0x11; 20])],
    );
    assert!(
        matches!(parse(&well_formed), Ok(Some(_))),
        "the fixture without a tail must decode, or this test proves nothing"
    );

    for extra in 1..=3usize {
        let mut bytes = well_formed.clone();
        bytes.extend(std::iter::repeat_n(0xde, extra));
        match parse(&bytes) {
            Err(DecodeError::Scale(msg)) => assert!(
                msg.contains(&extra.to_string()) && msg.contains("trailing"),
                "the error must name the {extra} trailing bytes: {msg}"
            ),
            other => panic!("{extra} trailing bytes must be refused, got {other:?}"),
        }
    }
}

// verifies: REQ-axcxf7, LLR-8242kq
//
// The other end: a payload that runs out before it has been read. Cut at the
// byte where `topics`' compact length prefix should begin, and again one byte
// into the second of two promised topics — a `Vec<H256>` whose prefix claims
// more than the buffer holds. Both must be refused by the codec rather than
// yielding a short topic list, which would otherwise reach the topic-count
// check as a *different*, plausible-looking shape.
#[test]
fn a_payload_that_ends_inside_its_topics_is_rejected() {
    let well_formed = encode_contract_emitted(
        CONTRACT,
        genesis_data([0xaa; 32], [0xbb; 32]),
        vec![sig_genesis(), padded_address([0x11; 20])],
    );
    // contract (20 raw) + compact len of 64 (two bytes: single-byte compact
    // mode only reaches 63) + data (64) = 86 bytes before the topics length
    // prefix; then 1 prefix byte for two topics and 2 * 32 topic bytes.
    const BEFORE_TOPICS: usize = 20 + 2 + 64;
    assert_eq!(well_formed.len(), BEFORE_TOPICS + 1 + 64);

    // 86: ends exactly where the topics length prefix should start, so the
    // prefix itself is missing. 119: prefix says two topics, the first is
    // whole and the second is one byte short.
    for cut in [BEFORE_TOPICS, BEFORE_TOPICS + 1 + 32 + 31] {
        let truncated = &well_formed[..cut];
        match parse(truncated) {
            Err(DecodeError::Scale(msg)) => assert!(
                msg.starts_with("topics:"),
                "a payload cut at {cut} must fail decoding topics: {msg}"
            ),
            other => panic!("a payload cut at {cut} must be refused, got {other:?}"),
        }
    }
}

// verifies: REQ-axcxf7, LLR-8242kq
//
// REQ-axcxf7's second clause, literally: a payload that ends **inside a
// field's own length prefix**. Neither cut above lands there — 86 stops
// exactly where `topics`' prefix begins, with zero bytes of it present, and
// 150 stops inside a topic's contents — so until review round 2's finding 12
// the clause was stated and not exercised.
//
// The cut that does it is 21. `data` is 64 bytes, and SCALE's single-byte
// compact mode only reaches 63, so 64 encodes two bytes wide:
// `(64 << 2) | 0b01 == 257`, little-endian `[0x01, 0x01]`. Those occupy
// payload bytes 20 and 21, immediately after the contract's twenty raw bytes.
// Cutting at 21 keeps the first prefix byte — whose low two bits promise a
// second — and removes it, so the decoder runs out of buffer *while reading a
// length*, before any field content exists to be short.
//
// The refusal comes from parity-scale-codec, which is the point: this is a
// robustness case the crate already satisfies through SOUP, recorded so that
// the requirement's second clause has evidence rather than an assumption. The
// error must still arrive as a typed `DecodeError::Scale` naming `data`, not
// as a panic and not as an `Ok` with a shorter `data` than the prefix claimed.
#[test]
fn a_payload_that_ends_inside_a_length_prefix_is_rejected() {
    let well_formed = encode_contract_emitted(
        CONTRACT,
        genesis_data([0xaa; 32], [0xbb; 32]),
        vec![sig_genesis(), padded_address([0x11; 20])],
    );

    // The prefix is where the arithmetic above says it is, and is two bytes
    // wide. Asserted rather than assumed, so a change to the encoding reds
    // this test here rather than silently moving the cut somewhere harmless.
    const PREFIX_START: usize = 20;
    assert_eq!(
        &well_formed[PREFIX_START..PREFIX_START + 2],
        &[0x01, 0x01],
        "`data`'s compact length of 64 must be the two-byte encoding 257"
    );
    assert_eq!(
        well_formed[PREFIX_START] & 0b11,
        0b01,
        "the first prefix byte's mode bits must promise a two-byte compact"
    );

    // Cut with one of the two prefix bytes present: inside the prefix itself.
    let truncated = &well_formed[..PREFIX_START + 1];
    match parse(truncated) {
        Err(DecodeError::Scale(msg)) => assert!(
            msg.starts_with("data:"),
            "a payload cut inside `data`'s length prefix must fail decoding data: {msg}"
        ),
        other => panic!(
            "a payload cut at {} — inside `data`'s length prefix — must be refused, got {other:?}",
            PREFIX_START + 1
        ),
    }
}

// ---------------------------------------------------------------------------
// REQ-9wwenn — the indexed epoch topic is bounded, never truncated
// ---------------------------------------------------------------------------

// verifies: LLR-mzh8df
//
// `RootUpdated` carries the epoch as an indexed `uint256` topic, and this
// reader represents it as a `u64`. The storage path's bound is gated in
// tests/decode_org_state.rs; nothing gated exercised the same bound through
// the *event* path, even though `parse_root_updated` reaches the same
// `uint256`-to-`u64` rule. That gap is what this test closes: a chain that
// reports an epoch above the range this reader can hold must be refused, not
// silently read as its low eight bytes — a truncated epoch is a stale
// membership root accepted as a current one.
//
// Every one of the twenty-four high positions is swept, because a reader that
// checked only some of them would still truncate for the rest. The low eight
// bytes carry 7 throughout, so the value a truncating reader would return is
// a specific, plausible-looking epoch rather than zero.
#[test]
fn root_updated_epoch_above_u64_is_refused_not_truncated() {
    let admin = [0x22u8; 20];
    let data = root_updated_data([0x40; 32], [0x80; 32], [0xc0; 32]);

    // The same log with a representable epoch decodes, or the refusals below
    // would prove nothing about the high half in particular.
    let representable = encode_contract_emitted(
        CONTRACT,
        data.clone(),
        vec![sig_root_updated(), padded_address(admin), uint256_be(7)],
    );
    assert!(
        matches!(parse(&representable), Ok(Some(_))),
        "the fixture with an epoch of 7 must decode, or this test proves nothing"
    );

    for position in 0..24usize {
        let mut epoch_topic = uint256_be(7);
        epoch_topic[position] = 0x01;

        let bytes = encode_contract_emitted(
            CONTRACT,
            data.clone(),
            vec![sig_root_updated(), padded_address(admin), epoch_topic],
        );
        assert_eq!(
            parse(&bytes),
            Err(DecodeError::EpochOverflow),
            "a non-zero byte at high position {position} must overflow, \
             not truncate to the low eight bytes"
        );
    }
}
