//! Whether a decoded log belongs to the Organisation a reader watches — the
//! caller's half of the contract-address filter, reached as the pure function
//! `on_chain_client::test_support::log_is_ours`.
//!
//! Why this target exists, and why it is not
//! `on-chain-client/tests/contract_address_filter.rs`. That target verifies
//! the *decoder's* half: the emitting contract's address is carried out of
//! `parse_revive_event` in `EmittedEvent` rather than dropped, and reported
//! byte for byte. The half that actually discards an impostor — comparing
//! that address against the one the reader was constructed for — used to sit
//! inline inside `async fn decode_contract_events`, which needs a live chain,
//! so no gated test reached it. That was measured, not assumed — and the
//! measurement below is dated: it is the historical reading that motivated
//! this target, not a description of what the suite does now. **On
//! 2026-09-10, before review round 1's finding 3 was fixed**, when
//! `contract_address_filter.rs` still reimplemented the predicate locally
//! instead of importing it, the comparison could be switched off in
//! `internals::log_is_ours` and `--test contract_address_filter` still
//! reported 3 passed. A fix whose decisive line can be deleted with every gate
//! staying green is the same false-green shape this tooth exists to remove, so
//! the decision was extracted into `log_is_ours` and is asserted here, where a
//! mutation of it reds a named case.
//!
//! **That reading no longer reproduces, and deliberately so.** Later the same
//! day, finding 3 deleted the local copy and pointed
//! `contract_address_filter.rs` at the real
//! `on_chain_client::test_support::log_is_ours` as well, so it now shares this
//! target's fate. Re-measured after that fix, on 2026-09-10, with the same
//! mutation — the contract comparison removed from `internals::log_is_ours` —
//! `--test contract_address_filter` reports 1 passed, 2 failed, and
//! `--test log_ownership` reports 4 passed, 3 failed. The 3-passed figure is
//! kept because it is the evidence for why the extraction happened; read it as
//! history, and never as current behaviour.
//!
//! What this target covers, and what it does not. It covers the decision:
//! given a decoded log and the reader's configuration, is this log ours. It
//! does not cover the plumbing the decision sits in — that subxt's event
//! iteration reaches `log_is_ours` for every `Revive::ContractEmitted` in a
//! block, and that a rejected log is dropped rather than surfaced as an
//! error, is exercised only by the chain-dependent chopsticks targets beside
//! it, which run in no CI lane and no `verify_commands` line here. And none of
//! those deploys a second contract, so the spoof below has no chain-level
//! counterpart at all. That delivery residual belongs to HAZ-werm85; the rule
//! is what is gated.
//!
//! The logs are built and decoded rather than hand-constructed, so the spoof
//! is a genuine one: well-formed SCALE, a valid OrgRegistry signature hash in
//! `topics[0]`, and the victim's valid indexed admin in `topics[1]`. Every
//! discriminator except the emitting address agrees with a real event.
//!
//! verifies: REQ-9vwcwc, REQ-nygs7k
//!
//! Deliberately carries no `#![cfg(...)]` guard, unlike the chain-dependent
//! targets beside it (corrected 2026-09-28: this said "the nine
//! chain-dependent targets". Nine run at no gate; the eight that carry a
//! crate-level `#![cfg(...)]` guard are exactly the eight that are chain-dependent
//! — seven need a fork, `smoldot_smoke` live Paseo, `regenerate_corpus` neither): the
//! `test-support` feature this target needs is declared as a
//! `required-features` entry in `Cargo.toml`, so an
//! explicit `--test log_ownership` with the feature off is refused by cargo
//! rather than compiled down to an empty binary that reports success having
//! run nothing.

use on_chain_client::decode::dispatch::{PASEO_AH_SPEC_VERSION, for_runtime};
use on_chain_client::state::EmittedEvent;
use on_chain_client::test_support::log_is_ours;
use on_chain_client::{Event, OrgAdmin};

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
const WANTED_ADMIN: [u8; 20] = [0x11; 20];
/// Another organisation's admin, in the same registry.
const OTHER_ADMIN: [u8; 20] = [0x22; 20];

/// A well-formed `GenesisInitialized` log emitted by `contract` for `admin`,
/// decoded the way the client decodes it.
fn genesis(contract: [u8; 20], admin: [u8; 20]) -> EmittedEvent {
    let mut data = Vec::with_capacity(64);
    data.extend_from_slice(&[0xaau8; 32]);
    data.extend_from_slice(&[0xbbu8; 32]);
    let topics = vec![sig_genesis(), padded_address(admin)];
    decode(&encode_contract_emitted(contract, data, topics))
}

/// A well-formed `RootUpdated` log emitted by `contract` for `admin`.
fn update(contract: [u8; 20], admin: [u8; 20]) -> EmittedEvent {
    let mut data = Vec::with_capacity(96);
    data.extend_from_slice(&[0xccu8; 32]);
    data.extend_from_slice(&[0xddu8; 32]);
    data.extend_from_slice(&[0xeeu8; 32]);
    let topics = vec![sig_root_updated(), padded_address(admin), uint256_be(7)];
    decode(&encode_contract_emitted(contract, data, topics))
}

fn decode(bytes: &[u8]) -> EmittedEvent {
    let decoder = for_runtime(PASEO_AH_SPEC_VERSION).expect("pinned Paseo AH decoder must resolve");
    decoder
        .parse_revive_event(bytes)
        .expect("a canonical ContractEmitted payload must decode without error")
        .expect("a known signature must decode to an event")
}

/// verifies: REQ-9vwcwc, LLR-2znra8
/// Normal case: a log from the configured contract, with no admin filter set,
/// is ours. This is the acceptance path every other case here is measured
/// against.
#[test]
fn a_log_from_the_configured_contract_with_no_filter_is_ours() {
    assert!(
        log_is_ours(&genesis(CONFIGURED, WANTED_ADMIN), &CONFIGURED, None),
        "a GenesisInitialized from the configured contract must be accepted",
    );
    assert!(
        log_is_ours(&update(CONFIGURED, WANTED_ADMIN), &CONFIGURED, None),
        "a RootUpdated from the configured contract must be accepted",
    );
}

/// verifies: REQ-9vwcwc, LLR-2znra8
/// Abnormal input — the spoofing attempt this control exists for. A different
/// contract emits a log with a valid OrgRegistry signature hash and the
/// victim's valid indexed admin, carrying a root of the attacker's choosing.
/// Nothing but the emitting address distinguishes it, so nothing but the
/// address comparison can reject it.
#[test]
fn the_spoof_a_valid_log_from_another_contract_is_not_ours() {
    let spoof = genesis(IMPOSTOR, WANTED_ADMIN);
    let genuine = genesis(CONFIGURED, WANTED_ADMIN);

    // Every discriminator except the address agrees with a genuine event.
    assert_eq!(
        spoof.event, genuine.event,
        "the spoofed payload must be structurally indistinguishable from a genuine one, \
         or this case is not testing what it claims",
    );

    assert!(
        !log_is_ours(&spoof, &CONFIGURED, None),
        "a log from any contract other than the configured one must be rejected",
    );
    assert!(
        !log_is_ours(&update(IMPOSTOR, WANTED_ADMIN), &CONFIGURED, None),
        "the rejection must not depend on the event shape",
    );
}

/// verifies: REQ-9vwcwc, LLR-2znra8
/// Boundary: the comparison is over all 20 bytes. An address differing in
/// exactly one byte — at the first position and at the last, the two a
/// truncated or shifted comparison would miss — is not ours.
#[test]
fn an_address_differing_in_one_byte_at_either_end_is_not_ours() {
    let mut first = CONFIGURED;
    first[0] ^= 0x01;
    let mut last = CONFIGURED;
    last[19] ^= 0x01;

    assert!(
        !log_is_ours(&genesis(first, WANTED_ADMIN), &CONFIGURED, None),
        "a one-bit difference in the first byte must be rejected",
    );
    assert!(
        !log_is_ours(&genesis(last, WANTED_ADMIN), &CONFIGURED, None),
        "a one-bit difference in the last byte must be rejected",
    );
}

/// verifies: REQ-9vwcwc, LLR-2znra8
/// The contract check dominates the admin filter: an impostor whose event
/// carries exactly the admin the subscriber filtered on is still rejected. A
/// filter that could rehabilitate a foreign log would defeat the control.
///
/// This is the case that gates REQ-9vwcwc's second clause — "so that no other
/// filter the subscription carries can admit a log from another contract". The
/// admin filter is the only other filter `log_is_ours` takes, and this case
/// runs it at the one input where it comes closest to admitting a foreign log:
/// the filter's own predicate is satisfied, and the log is still rejected. A
/// filter that could rehabilitate a foreign log would defeat the control, and
/// the assertion above the rejection pins the fixture so the case cannot pass
/// vacuously.
///
/// **What no case here gates, and why no case could.** Until review round 3 the
/// requirement also said the contract comparison is made "before, and
/// independently of" any other filter, and this comment claimed to gate that.
/// It never did. Round 3 rewrote `internals::log_is_ours` to test the admin
/// filter first and return the contract comparison last — the exact inversion
/// of the stated ordering — and `--test log_ownership` still reported 7 passed,
/// 0 failed. That is not a gap in the cases: `log_is_ours` is a conjunction of
/// two pure, total tests, so on every input the two orders return the same
/// bool, and a caller holding only the result can never tell them apart. The
/// ordering was therefore unfalsifiable as written, and fix round 8a dropped it
/// from REQ-9vwcwc and from RC-5e3bdk, keeping the consequence — which is
/// observable, and reds here. Do not restore an ordering claim to this comment
/// or to the requirement expecting a test to hold it up; nothing in this file
/// can.
///
/// The second clause used to live in REQ-nygs7k, which bundled it with that
/// requirement's delivery rule; review round 2 (finding-11) folded it here,
/// where the contract check and this test already were.
#[test]
fn the_contract_check_dominates_a_matching_admin_filter() {
    let filter = Some(OrgAdmin(WANTED_ADMIN));
    let spoof = genesis(IMPOSTOR, WANTED_ADMIN);

    assert_eq!(
        event_admin(&spoof.event),
        OrgAdmin(WANTED_ADMIN),
        "the spoof must carry the filtered admin, or this case proves nothing",
    );
    assert!(
        !log_is_ours(&spoof, &CONFIGURED, filter),
        "a matching admin must not rescue a log from a foreign contract",
    );
}

/// verifies: REQ-nygs7k, LLR-kfr75c, LLR-9qp3k7
/// With a filter set, a log for that admin from the configured contract is
/// ours — for both event shapes, since the two carry the admin in different
/// positions of different structs.
#[test]
fn with_a_filter_set_a_matching_admin_is_ours() {
    let filter = Some(OrgAdmin(WANTED_ADMIN));

    assert!(
        log_is_ours(&genesis(CONFIGURED, WANTED_ADMIN), &CONFIGURED, filter),
        "a GenesisInitialized for the filtered admin must be accepted",
    );
    assert!(
        log_is_ours(&update(CONFIGURED, WANTED_ADMIN), &CONFIGURED, filter),
        "a RootUpdated for the filtered admin must be accepted",
    );
}

/// verifies: REQ-nygs7k, LLR-kfr75c
/// Abnormal input for the filter: another Organisation's event, from the same
/// genuine registry. A subscriber that filtered on one admin and received
/// this would form a membership belief about the wrong Organisation.
#[test]
fn with_a_filter_set_a_non_matching_admin_is_not_ours() {
    let filter = Some(OrgAdmin(WANTED_ADMIN));

    assert!(
        !log_is_ours(&genesis(CONFIGURED, OTHER_ADMIN), &CONFIGURED, filter),
        "a GenesisInitialized for another admin must be rejected",
    );
    assert!(
        !log_is_ours(&update(CONFIGURED, OTHER_ADMIN), &CONFIGURED, filter),
        "a RootUpdated for another admin must be rejected",
    );
}

/// verifies: REQ-9vwcwc, LLR-kfr75c
/// `None` means no filter, not "match nothing": every admin from the
/// configured contract is ours. This is what a subscriber watching a whole
/// registry relies on.
///
/// Annotated to REQ-9vwcwc, not REQ-nygs7k. REQ-nygs7k is conditioned on a
/// subscription that names an admin to filter on, and this case names none, so
/// the requirement has nothing to say about it. What it is a case of is
/// REQ-9vwcwc's delivery rule — a log from the configured contract is
/// delivered — read across the whole admin space rather than the single admin
/// `a_log_from_the_configured_contract_with_no_filter_is_ours` uses. It moved
/// here at review round 2 (finding-11), when the clause about other filters was
/// folded into REQ-9vwcwc and REQ-nygs7k was reworded to the filter rule alone.
/// That clause named an ordering when it was folded; review round 3 found the
/// ordering unfalsifiable and fix round 8a cut it back to the consequence, as
/// `the_contract_check_dominates_a_matching_admin_filter` records.
#[test]
fn with_no_filter_any_admin_from_the_configured_contract_is_ours() {
    for admin in [WANTED_ADMIN, OTHER_ADMIN, [0x00; 20], [0xff; 20]] {
        assert!(
            log_is_ours(&genesis(CONFIGURED, admin), &CONFIGURED, None),
            "with no filter, admin {admin:02x?} must be accepted",
        );
        assert!(
            log_is_ours(&update(CONFIGURED, admin), &CONFIGURED, None),
            "with no filter, admin {admin:02x?} must be accepted for RootUpdated",
        );
    }
}

/// The admin an event carries, for the assertions above that need to state
/// what the fixture built. Mirrors `client.rs`'s `event_admin`; kept local
/// because a test that borrowed the implementation's accessor would assert
/// nothing about the fixture.
fn event_admin(ev: &Event) -> OrgAdmin {
    match ev {
        Event::Genesis { admin, .. } => *admin,
        Event::Update { admin, .. } => *admin,
    }
}
