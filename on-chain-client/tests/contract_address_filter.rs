//! verifies: REQ-5upq6n
//! The decoder surrenders the emitting contract's address with every decoded
//! event, so a log from any other contract can be told apart from this
//! organisation's own.
//!
//! Why this target exists. `parse_revive_event` used to decode the
//! `ContractEmitted` payload's `contract` field into `let _contract` and drop
//! it, and `client.rs`'s `event_matches_contract` was
//! `let _ = (ev, contract); true` — so *any* contract on Asset Hub could emit
//! a log carrying the OrgRegistry signature hash and a victim organisation's
//! indexed admin, and every subscriber received it as a genuine event with an
//! attacker-chosen root (HAZ-werm85, PR-p5ngya). The fix is in two halves:
//! the decoder carries the address out in `EmittedEvent`, and the caller
//! compares it against the address the reader was constructed for.
//!
//! **This target is the DECODER's half — REQ-5upq6n, that the address the
//! decoder reports is the one the log actually carried, byte for byte, for
//! both event shapes.** The caller's half — REQ-9vwcwc, whether a log is ours
//! — is the subject of `tests/log_ownership.rs`, and is asserted here only as
//! the consequence that gives reporting its point: an address reported
//! faithfully is one a comparison can decide on. Those `log_is_ours` calls
//! carry no `verifies:` of their own, because every case they make is already
//! made against the decision itself, with its boundaries, next door; claiming
//! REQ-9vwcwc here would add a second copy of that evidence rather than a
//! second piece of it.
//!
//! What this file used to do, and why that was a defect (review round 1,
//! finding 3). Until 2026-09-10 it did not import the decision at all: it
//! reimplemented it locally as `fn is_from_configured_contract`, commented
//! "The caller's decision, verbatim". A copy of a predicate tests the copy.
//! That was measured, not argued — with the real comparison deleted from
//! `internals::log_is_ours`, this file still reported 3 passed while
//! `log_ownership` reported 3 failed. The local copy is gone and the calls
//! below go through `on_chain_client::test_support::log_is_ours`, the same
//! function `decode_contract_events` applies, so a mutation of the decision
//! reds this file too.
//!
//! What is still NOT covered here. That `decode_contract_events` really
//! reaches `log_is_ours` for every `Revive::ContractEmitted` subxt hands it
//! lives in an `async` function that needs a live chain, and no test in this
//! repository exercises it: every chopsticks target deploys one OrgRegistry
//! and hands its address to `OrgRegistryClient::from_client`, so none has a
//! second contract to spoof from. That delivery residual belongs to
//! HAZ-werm85.
//!
//! Deliberately carries no `#![cfg(...)]` guard: the `test-support` feature
//! this target needs is declared as a `required-features` entry in
//! `Cargo.toml`, so an explicit `--test contract_address_filter` with the
//! feature off is refused by cargo rather than compiled down to an empty
//! binary that reports success having run nothing.

use on_chain_client::decode::dispatch::{PASEO_AH_SPEC_VERSION, for_runtime};
use on_chain_client::state::EmittedEvent;
use on_chain_client::test_support::log_is_ours;
use on_chain_client::{Epoch, Event, OnChainRootHash, OrgAdmin, OrgPubKey};

#[path = "fuzz_support/mod.rs"]
mod support;

use support::{encode_contract_emitted, padded_address, sig_genesis, sig_root_updated, uint256_be};

/// The address the reader under test was constructed for — the real
/// OrgRegistry deployment.
const CONFIGURED: [u8; 20] = [0x55; 20];
/// Any other contract on the same chain. Nothing stops it emitting a log with
/// our signature hash and our admin in `topics[1]`.
const IMPOSTOR: [u8; 20] = [0x99; 20];
/// The organisation whose events the subscriber wants.
const VICTIM_ADMIN: [u8; 20] = [0x11; 20];

/// A well-formed `GenesisInitialized` log emitted by `contract` for `admin`.
fn genesis_log(contract: [u8; 20], admin: [u8; 20]) -> (Vec<u8>, Event) {
    let root = [0xaau8; 32];
    let key = [0xbbu8; 32];
    let mut data = Vec::with_capacity(64);
    data.extend_from_slice(&root);
    data.extend_from_slice(&key);
    let topics = vec![sig_genesis(), padded_address(admin)];
    let bytes = encode_contract_emitted(contract, data, topics);
    let event = Event::Genesis {
        admin: OrgAdmin(admin),
        root_hash: OnChainRootHash(root),
        org_pub_key: OrgPubKey(key),
    };
    (bytes, event)
}

/// A well-formed `RootUpdated` log emitted by `contract` for `admin`.
fn root_updated_log(contract: [u8; 20], admin: [u8; 20]) -> (Vec<u8>, Event) {
    let root = [0xccu8; 32];
    let key = [0xddu8; 32];
    let prev = [0xeeu8; 32];
    let mut data = Vec::with_capacity(96);
    data.extend_from_slice(&root);
    data.extend_from_slice(&key);
    data.extend_from_slice(&prev);
    let topics = vec![sig_root_updated(), padded_address(admin), uint256_be(7)];
    let bytes = encode_contract_emitted(contract, data, topics);
    let event = Event::Update {
        admin: OrgAdmin(admin),
        epoch: Epoch(7),
        root_hash: OnChainRootHash(root),
        org_pub_key: OrgPubKey(key),
        prev_root_hash: OnChainRootHash(prev),
    };
    (bytes, event)
}

fn decode(bytes: &[u8]) -> Option<EmittedEvent> {
    let decoder = for_runtime(PASEO_AH_SPEC_VERSION).expect("pinned Paseo AH decoder must resolve");
    decoder
        .parse_revive_event(bytes)
        .expect("a canonical ContractEmitted payload must decode without error")
}

/// verifies: REQ-5upq6n
/// Normal case: a log from the configured contract decodes, and the address it
/// reports is the configured one, so the caller's comparison accepts it.
#[test]
fn log_from_the_configured_contract_reports_that_address() {
    let (bytes, expected) = genesis_log(CONFIGURED, VICTIM_ADMIN);
    let emitted = decode(&bytes).expect("a known signature must decode to an event");
    assert_eq!(
        emitted,
        EmittedEvent {
            contract: CONFIGURED,
            event: expected,
        },
        "the decoder must report the emitting address alongside the event",
    );
    assert!(
        log_is_ours(&emitted, &CONFIGURED, None),
        "a log from the configured contract must pass the caller's comparison",
    );
}

/// verifies: REQ-5upq6n
/// Abnormal input — the spoofing attempt this control exists for. A different
/// contract emits a log with a *valid* OrgRegistry signature hash and the
/// victim organisation's *valid* indexed admin, carrying a root of the
/// attacker's choosing. Pallet and variant filtering cannot tell it apart, and
/// neither can the signature hash or the admin: only the emitting address can.
/// So it must decode (the bytes are well-formed) and report the impostor's
/// address, and the caller's comparison must reject it.
#[test]
fn log_from_another_contract_with_a_valid_signature_is_rejected() {
    let (bytes, event) = genesis_log(IMPOSTOR, VICTIM_ADMIN);
    let emitted = decode(&bytes).expect("a known signature must decode to an event");

    // Every other discriminator agrees with a genuine event: same signature
    // hash, same indexed admin, same shape.
    assert_eq!(
        emitted.event, event,
        "the spoofed payload is structurally indistinguishable from a genuine one",
    );
    assert_eq!(
        emitted.contract, IMPOSTOR,
        "the decoder must report the impostor's address, not the configured one",
    );
    assert!(
        !log_is_ours(&emitted, &CONFIGURED, None),
        "a log from any other contract must fail the caller's comparison",
    );
}

/// verifies: REQ-5upq6n
/// The address is reported byte for byte, for `RootUpdated` as well as
/// `GenesisInitialized`, and for an address with no repeated-byte structure to
/// hide a truncation or a shifted copy.
#[test]
fn the_emitting_address_is_reported_byte_for_byte_for_both_event_shapes() {
    let odd: [u8; 20] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0xf0, 0xf1, 0xf2, 0xf3, 0xf4,
        0xf5, 0xf6, 0xf7, 0xf8, 0xff,
    ];

    let (genesis_bytes, genesis_event) = genesis_log(odd, VICTIM_ADMIN);
    let emitted = decode(&genesis_bytes).expect("GenesisInitialized must decode");
    assert_eq!(
        emitted,
        EmittedEvent {
            contract: odd,
            event: genesis_event,
        },
    );
    assert!(!log_is_ours(&emitted, &CONFIGURED, None));

    let (update_bytes, update_event) = root_updated_log(odd, VICTIM_ADMIN);
    let emitted = decode(&update_bytes).expect("RootUpdated must decode");
    assert_eq!(
        emitted,
        EmittedEvent {
            contract: odd,
            event: update_event,
        },
    );
    assert!(!log_is_ours(&emitted, &CONFIGURED, None));

    // …and the same RootUpdated from the configured contract is accepted, so
    // the rejection above is about the address and not about the event shape.
    let (ok_bytes, ok_event) = root_updated_log(CONFIGURED, VICTIM_ADMIN);
    let emitted = decode(&ok_bytes).expect("RootUpdated must decode");
    assert_eq!(
        emitted,
        EmittedEvent {
            contract: CONFIGURED,
            event: ok_event,
        },
    );
    assert!(log_is_ours(&emitted, &CONFIGURED, None));
}
