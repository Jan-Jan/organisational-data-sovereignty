//! pallet-revive's AccountId32 -> H160 mapping, which is what fixes an
//! organisation's OrgId and therefore its slot key in `OrgRegistry` storage.
//! `h160_of` has two paths and the whole of this unit's addressing depends on
//! taking the right one:
//!
//!   - reverse: the last twelve bytes are all `0xEE` (the EVM-fallback
//!     marker), so the first twenty bytes ARE the H160 (REQ-tkhe3u);
//!   - forward: anything else is `keccak256(account_id_32)[12..32]`
//!     (REQ-rz7fja).
//!
//! Both requirements implement RC-mugta4. These tests were relocated here out
//! of `src/h160.rs`'s `#[cfg(test)]` module, which no gate reads
//! (`test_paths` for this unit is `on-chain-client/tests`), and joined by the
//! boundary cases class C asks for: the all-`0xEE` AccountId32, where the two
//! paths are hardest to tell apart, and each single non-marker byte at the
//! twelve marker positions, one position at a time.
//!
//! Every expectation is recomputed in this file with `tiny-keccak` (a
//! dev-dependency, so the recomputation does not go through anything in
//! `src`); the marker constant is written out here as `0xEE` rather than
//! imported, so a change to `src`'s `EVM_FALLBACK_MARK` cannot move the test's
//! expectation with it.
//!
//! **What this file does NOT establish.** It pins the implementation against
//! an independent recomputation of *our reading* of pallet-revive, not against
//! the runtime's own answer. The runtime is the only authority on that
//! mapping, and the exact byte position of the `0xEE` prefix has already moved
//! across pallet-revive versions. The test that consults the runtime is
//! `on-chain-client/tests/p_address_is_orgid.rs` — it compares `h160_of(P)`
//! with the admin address the runtime itself puts in the contract event, and
//! across a multisig rotation — but it is `#![cfg(feature = "dev-rpc")]` and
//! needs a live chopsticks fork, so it runs in no gate in this repository.
//! Gating it is not-minted control 1 and the residual it leaves is part of
//! HAZ-v2cmtx's residual risk.

use on_chain_client::h160_of;
use tiny_keccak::{Hasher, Keccak};

/// The EVM-fallback marker byte, twelve copies of which in the AccountId32's
/// last twelve bytes select the reverse path. Written out independently of
/// `src`'s own constant.
const MARK: u8 = 0xEE;

/// `keccak256(account_id_32)[12..32]` — the forward mapping, recomputed here
/// rather than taken from anything under `src`.
fn keccak_low_20(account_id_32: &[u8; 32]) -> [u8; 20] {
    let mut hasher = Keccak::v256();
    hasher.update(account_id_32);
    let mut hash = [0u8; 32];
    hasher.finalize(&mut hash);
    let mut low = [0u8; 20];
    low.copy_from_slice(&hash[12..32]);
    low
}

/// The first twenty bytes, which is what the reverse path must return.
fn first_20(account_id_32: &[u8; 32]) -> [u8; 20] {
    let mut head = [0u8; 20];
    head.copy_from_slice(&account_id_32[..20]);
    head
}

// verifies: REQ-tkhe3u, LLR-2yhra8
//
// Normal case for the reverse path: an EVM-derived account is `H160 ||
// [0xEE; 12]`, and the H160 comes back unchanged.
#[test]
fn reverse_path_returns_the_first_twenty_bytes() {
    let mut id = [MARK; 32];
    for (i, byte) in id.iter_mut().enumerate().take(20) {
        *byte = i as u8;
    }

    let got = h160_of(id);

    let mut expected = [0u8; 20];
    for (i, byte) in expected.iter_mut().enumerate() {
        *byte = i as u8;
    }
    assert_eq!(got, expected, "reverse path must return the first 20 bytes");
    // And it must not be the forward answer — otherwise the assertion above
    // would also hold for an implementation that always keccaked.
    assert_ne!(
        got,
        keccak_low_20(&id),
        "reverse path must not be the keccak answer"
    );
}

// verifies: REQ-tkhe3u, LLR-2yhra8, LLR-7pjzjn
//
// The boundary where the two paths are hardest to tell apart: every one of the
// thirty-two bytes is `0xEE`, so the twelve marker bytes are present AND the
// twenty bytes the reverse path returns are themselves markers. The reverse
// path must still be taken, and the answer is twenty `0xEE` bytes — not the
// keccak of the account.
#[test]
fn reverse_path_taken_when_the_whole_account_is_the_marker_byte() {
    let id = [MARK; 32];

    let got = h160_of(id);

    assert_eq!(got, [MARK; 20], "all-0xEE account maps to twenty 0xEE bytes");
    assert_eq!(got, first_20(&id));
    assert_ne!(
        got,
        keccak_low_20(&id),
        "all-0xEE account must not take the forward path"
    );
}

// verifies: REQ-rz7fja, LLR-3bkhuc
//
// Normal case for the forward path: a Substrate-style account with no marker
// suffix is keccaked and truncated to the low twenty bytes. This is the case
// our pure proxies live in.
#[test]
fn forward_path_keccaks_then_truncates() {
    let id = [0xAA; 32];

    let got = h160_of(id);

    assert_eq!(got, keccak_low_20(&id));
    assert_ne!(
        got,
        first_20(&id),
        "forward path must not return the first 20 bytes"
    );
}

// verifies: REQ-rz7fja, LLR-3bkhuc, LLR-7pjzjn
//
// The near-miss: eleven of the twelve marker positions hold `0xEE` and byte 20
// does not. Eleven is not twelve, so the forward path is taken.
#[test]
fn forward_path_taken_when_only_eleven_marker_bytes_are_present() {
    let mut id = [MARK; 32];
    for byte in id.iter_mut().take(20) {
        *byte = 0u8;
    }
    id[20] = 0u8;

    let got = h160_of(id);

    assert_eq!(got, keccak_low_20(&id));
    assert_ne!(
        got,
        first_20(&id),
        "eleven markers must not select the reverse path"
    );
}

// verifies: REQ-rz7fja, LLR-3bkhuc, LLR-7pjzjn
//
// The same near-miss swept across the whole marker window, one position at a
// time: start from an all-`0xEE` account — the case that DOES take the reverse
// path — and clear exactly one of the twelve marker bytes. Each of the twelve
// must move the account onto the forward path, which is what pins the window
// to bytes 20..32 rather than to any eleven-byte subrange of it.
#[test]
fn forward_path_taken_for_a_single_non_marker_byte_at_each_marker_position() {
    for pos in 20..32usize {
        let mut id = [MARK; 32];
        id[pos] = 0u8;

        let got = h160_of(id);

        assert_eq!(
            got,
            keccak_low_20(&id),
            "byte {pos} cleared: expected the forward path"
        );
        assert_ne!(
            got,
            first_20(&id),
            "byte {pos} cleared: reverse path must not be taken"
        );
    }
}
