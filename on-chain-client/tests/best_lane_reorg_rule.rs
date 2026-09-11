//! The best lane's per-notification decision — the first tests this logic
//! has ever had. It is reached as the pure function
//! `on_chain_client::test_support::scan_step`, extracted verbatim out of
//! `best_lane`'s `.scan()` closure so the decision can be exercised without
//! a chain; `ScanStep` is what the closure hands downstream, where
//! `reorged: Some(prev)` becomes `SubscribedEvent::Reorged { discarded: prev }`
//! and `from ..= to` becomes the inclusive by-number backfill the caller
//! iterates.
//!
//! **What this target covers, and what it does not.** It covers the
//! *decision*: given the last best head actually processed and a new head's
//! (number, hash, parent), what the lane emits and how `last` advances.
//! Whether subxt delivers the head notifications the decision is made on —
//! that a depth-1 reorg produces a head whose parent is not the previous
//! best, and that the discarded hash reaches the consumer — is checked only
//! by `on-chain-client/tests/reorg_cancels_proposed.rs`, which is
//! `#![cfg(feature = "dev-rpc")]` against a live chopsticks fork and runs in
//! no CI lane and no `verify_commands` line in this repository. So the
//! delivery half of this behaviour has no gated evidence; only the rule does.
//!
//! **Two things the rule cannot do**, recorded here because a reader of these
//! assertions would otherwise mistake them for guarantees. Both are open
//! problem reports, not assertions in this file:
//!
//! - A reorg deeper than the notification gap is invisible to it. For a jump
//!   `n > last.number + 1` no reorg signal is derivable from the head
//!   notifications, and the by-number backfill that fills the gap can only
//!   ever see canonical blocks — so the discarded fork is never reported
//!   (PR-qpp28h). Where a jump appears below, its `reorged: None` is an
//!   assertion of *today's behaviour under that report*, not a claim that no
//!   reorg occurred.
//! - The span returned for the first notification after the seed is
//!   unbounded. The seed is the latest finalised block, which on a live chain
//!   lags the best tip, so `from ..= to` can be arbitrarily long and costs one
//!   `at_block` round-trip per height (PR-uq5r97). The `from` arithmetic below
//!   is asserted over short spans; nothing here bounds a long one.
//!
//! verifies: REQ-gr2ver, REQ-ntn4ss, REQ-5zux82
//!
//! Deliberately carries no `#![cfg(...)]` guard, unlike the nine
//! chain-dependent targets beside it: the `test-support` feature this target
//! needs is declared as a `required-features` entry in `Cargo.toml`, so an
//! explicit `--test best_lane_reorg_rule` with the feature off is refused by
//! cargo rather than compiled down to an empty binary that reports success
//! having run nothing.

use on_chain_client::state::{BlockHash, BlockRef};
use on_chain_client::test_support::{ScanStep, scan_step};

/// A distinct block hash per tag byte. Hashes are compared, never hashed, so
/// any injective mapping will do.
fn hash(tag: u8) -> BlockHash {
    BlockHash([tag; 32])
}

/// The last best head actually processed, at `number` with hash tag `tag`.
fn head(number: u64, tag: u8) -> BlockRef {
    BlockRef {
        hash: hash(tag),
        number,
    }
}

// ───────────────────────── REQ-gr2ver — dedup ─────────────────────────

/// verifies: REQ-gr2ver
///
/// A repeated head — same hash as the last one processed — yields nothing and
/// does not advance `last`. This is the guard that stops a depth-1 reorg
/// re-emitting the replacement block's events twice: the reorg is signalled
/// once, on the notification that actually changed the head, and a
/// re-delivery of that same head is silent.
#[test]
fn repeated_head_is_skipped_and_last_is_unchanged() {
    let mut last = Some(head(10, 0xAA));

    let step = scan_step(&mut last, 10, hash(0xAA), hash(0x09));

    assert_eq!(step, ScanStep::Skip);
    assert_eq!(
        last,
        Some(head(10, 0xAA)),
        "a skipped head must not advance `last`, or the next notification \
         would be judged against a head that was never processed"
    );
}

/// verifies: REQ-gr2ver
///
/// Abnormal input: the same hash arriving with a *different* claimed number
/// and a *different* claimed parent. The hash identifies the block, so the
/// rest of the notification cannot make it a new one — the step is still
/// `Skip` and `last` still does not move. A rule that keyed dedup on the
/// number instead would emit this block's events a second time.
#[test]
fn repeated_hash_with_inconsistent_number_is_still_skipped() {
    let mut last = Some(head(10, 0xAA));

    let step = scan_step(&mut last, 42, hash(0xAA), hash(0xBB));

    assert_eq!(step, ScanStep::Skip);
    assert_eq!(last, Some(head(10, 0xAA)));
}

/// verifies: REQ-gr2ver
///
/// A head that differs from the last processed one only in hash is not a
/// repeat: it is the replacement block of a depth-1 reorg, and must be
/// processed. The dedup guard has to be exactly hash equality — no wider.
#[test]
fn same_height_different_hash_is_not_a_repeat() {
    let mut last = Some(head(10, 0xAA));

    let step = scan_step(&mut last, 10, hash(0xA1), hash(0x09));

    assert_ne!(step, ScanStep::Skip);
    assert_eq!(
        last,
        Some(head(10, 0xA1)),
        "a processed head must become the new `last`"
    );
}

// ──────────────────── REQ-ntn4ss — reorg detection ────────────────────

/// verifies: REQ-ntn4ss
///
/// The ordinary case: a head at the next height whose parent *is* the last
/// processed head. REQ-ntn4ss reports a reorg at the next height only for a
/// parent *other* than that head, so nothing was discarded here.
#[test]
fn child_of_the_last_head_reports_no_reorg() {
    let mut last = Some(head(10, 0xAA));

    let step = scan_step(&mut last, 11, hash(0x0B), hash(0xAA));

    assert_eq!(
        step,
        ScanStep::Block {
            reorged: None,
            from: 11,
            to: 11
        }
    );
}

/// verifies: REQ-ntn4ss
///
/// Same height, different parent: the new head sits one above the last
/// processed but does not descend from it, so the last processed head was
/// discarded and is reported. This is the depth-1 reorg the parent-hash
/// tracking exists to catch — the by-number backfill can never see it,
/// because the discarded block is no longer canonical at any height.
#[test]
fn next_height_with_a_foreign_parent_reports_the_discarded_head() {
    let mut last = Some(head(10, 0xAA));

    let step = scan_step(&mut last, 11, hash(0x0B), hash(0xA1));

    assert_eq!(
        step,
        ScanStep::Block {
            reorged: Some(head(10, 0xAA)),
            from: 11,
            to: 11
        }
    );
}

/// verifies: REQ-ntn4ss
///
/// A rewind to the same height — the boundary case, and the one a
/// strictly-less-than comparison would miss. The head has the last
/// processed head's number but a different hash, so the last processed head
/// is no longer canonical and must be reported discarded.
#[test]
fn a_rewind_to_the_same_height_reports_the_discarded_head() {
    let mut last = Some(head(10, 0xAA));

    let step = scan_step(&mut last, 10, hash(0xA1), hash(0x09));

    assert_eq!(
        step,
        ScanStep::Block {
            reorged: Some(head(10, 0xAA)),
            from: 10,
            to: 10
        }
    );
}

/// verifies: REQ-ntn4ss
///
/// A rewind *below* the last processed height reports it too — including the
/// deep rewind all the way to height 0, the widest a `u64` height can be
/// rewound.
#[test]
fn a_rewind_below_the_last_height_reports_the_discarded_head() {
    for (n, parent_tag) in [(9u64, 0x08u8), (1, 0x00), (0, 0xFF)] {
        let mut last = Some(head(10, 0xAA));

        let step = scan_step(&mut last, n, hash(0x0C), hash(parent_tag));

        assert_eq!(
            step,
            ScanStep::Block {
                reorged: Some(head(10, 0xAA)),
                from: n,
                to: n
            },
            "a rewind to height {n} must report the discarded head"
        );
    }
}

/// verifies: REQ-ntn4ss
///
/// The discarded reference carries the previous head's hash **and** its
/// number, not just the hash: a consumer keyed on the hash still needs the
/// number to know how far back its optimistic state must be rolled, and the
/// notification is the only place it appears. Asserted against a
/// deliberately unrelated hash/number pair so a swapped or defaulted field
/// could not pass.
#[test]
fn the_discarded_reference_carries_both_hash_and_number() {
    let mut last = Some(head(7_654_321, 0x5E));

    let step = scan_step(&mut last, 7_654_322, hash(0x0D), hash(0x5F));

    match step {
        ScanStep::Block {
            reorged: Some(discarded),
            ..
        } => {
            assert_eq!(discarded.hash, hash(0x5E));
            assert_eq!(discarded.number, 7_654_321);
        }
        other => panic!("expected a reorg report, got {other:?}"),
    }
}

/// verifies: REQ-ntn4ss
///
/// Abnormal input: the very first notification, with no last processed head
/// at all. There is nothing that could have been discarded, so no reorg is
/// reported however the head's parent looks. (`best_lane` seeds `last` from
/// the finalised head before subscribing, so `None` is not reached there;
/// the rule must still be defined for it.)
#[test]
fn the_first_notification_reports_no_reorg() {
    let mut last: Option<BlockRef> = None;

    let step = scan_step(&mut last, 10, hash(0xAA), hash(0x09));

    assert_eq!(
        step,
        ScanStep::Block {
            reorged: None,
            from: 10,
            to: 10
        }
    );
    assert_eq!(last, Some(head(10, 0xAA)));
}

// ───────────────────── REQ-5zux82 — the backfill span ─────────────────

/// verifies: REQ-5zux82
///
/// From a seed at height 10, a head at 13 backfills `11 ..= 13` — every
/// height above the last processed one up to and including the new head, so
/// the two intermediate blocks the notification skipped are still read.
///
/// The `reorged: None` asserted here is today's behaviour under PR-qpp28h,
/// not a claim that nothing was discarded: for a jump of more than one
/// height no reorg signal is derivable from the notifications at all.
#[test]
fn a_jump_backfills_every_skipped_height() {
    let mut last = Some(head(10, 0xAA));

    let step = scan_step(&mut last, 13, hash(0x0D), hash(0x0C));

    assert_eq!(
        step,
        ScanStep::Block {
            reorged: None,
            from: 11,
            to: 13
        }
    );
    assert_eq!(last, Some(head(13, 0x0D)));
}

/// verifies: REQ-5zux82
///
/// The one-block step is the boundary of the same arithmetic: `11 ..= 11`,
/// the new head alone and nothing below it. An off-by-one in `from` shows up
/// here as the last processed height being read a second time.
#[test]
fn a_single_step_backfills_that_height_alone() {
    let mut last = Some(head(10, 0xAA));

    let step = scan_step(&mut last, 11, hash(0x0B), hash(0xAA));

    assert_eq!(
        step,
        ScanStep::Block {
            reorged: None,
            from: 11,
            to: 11
        }
    );
}

/// verifies: REQ-5zux82
///
/// A rewind backfills the new head's height alone — never a descending or
/// empty range. The previous head was higher, so `prev.number + 1` is not a
/// legal `from`; the span collapses onto `n`.
#[test]
fn a_rewind_backfills_the_new_height_alone() {
    let mut last = Some(head(10, 0xAA));

    let step = scan_step(&mut last, 9, hash(0x09), hash(0x08));

    assert_eq!(
        step,
        ScanStep::Block {
            reorged: Some(head(10, 0xAA)),
            from: 9,
            to: 9
        }
    );
}

/// verifies: REQ-5zux82
///
/// The first head after a seed at the *same* height backfills that height
/// alone — the seed's hash is a finalised hash, so it never matches a best
/// head for dedup, and the replacement at that height still has to be read.
#[test]
fn the_first_head_at_the_seed_height_backfills_that_height_alone() {
    let mut last = Some(head(10, 0xAA));

    let step = scan_step(&mut last, 10, hash(0xA1), hash(0x09));

    assert_eq!(
        step,
        ScanStep::Block {
            reorged: Some(head(10, 0xAA)),
            from: 10,
            to: 10
        }
    );
}

/// verifies: REQ-5zux82
///
/// `from ..= to` is the contract the caller iterates, so it must never be
/// descending — a `from > to` range iterates zero times and would silently
/// read no blocks at all. Checked across first notifications with no last
/// head, single steps, jumps, same-height replacements and rewinds,
/// including at the `u64` extremes where the arithmetic could wrap.
#[test]
fn the_backfill_span_is_always_ascending() {
    let cases: [(Option<BlockRef>, u64); 8] = [
        (None, 0),
        (None, u64::MAX),
        (Some(head(10, 0xAA)), 11),
        (Some(head(10, 0xAA)), 13),
        (Some(head(10, 0xAA)), 10),
        (Some(head(10, 0xAA)), 9),
        (Some(head(10, 0xAA)), 0),
        (Some(head(u64::MAX - 1, 0xAA)), u64::MAX),
    ];

    for (seed, n) in cases {
        let mut last = seed;
        let step = scan_step(&mut last, n, hash(0x0E), hash(0x0F));
        match step {
            ScanStep::Block { from, to, .. } => {
                assert!(
                    from <= to,
                    "span {from}..={to} is descending for seed {seed:?} and head {n}"
                );
                assert_eq!(to, n, "the span must end at the new head");
            }
            other => panic!("expected a block step for seed {seed:?} and head {n}, got {other:?}"),
        }
    }
}
