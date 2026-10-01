//! verifies: REQ-sx5b6g, LLR-2v5u4d
//!
//! Fuzz target: `Decoder::parse_revive_event` must never panic on arbitrary
//! bytes. The input models the SCALE payload of
//! `pallet_revive::Event::ContractEmitted { contract, data, topics }` — but
//! the point of fuzzing is that we feed *arbitrary* bytes, including
//! truncated / oversized / adversarial length prefixes, and require a clean
//! Ok/Err rather than a panic, abort, or runaway allocation.
//!
//! REQ-sx5b6g implements RC-8w9wtp, and this target is its evidence for the
//! *event* decode path; `fuzz_decode_org_state` covers the storage path and
//! `fuzz_event_round_trip` the inverse property. All three already ran in this
//! unit's `verify_commands` before this annotation existed and carried no
//! `verifies:` reference, so `check-trace.sh` credited them with nothing —
//! evidence that ran at every merge and counted for nothing.
//!
//! **What a green run of this target does and does not say.** It is a
//! `harness = false` binary, so it reports no pass count and no test names.
//! bolero prints one line — run time, iterations/s, corpus inputs, rng inputs
//! and an exit reason — and a panic, which is its failure signal, exits
//! non-zero and fails `cargo test`. "Green" therefore means the process
//! exhausted its time budget without panicking or aborting; it never means
//! "N cases passed". Read the iteration total and the exit reason, not a
//! count.
//!
//! **Depth: one second of generated inputs, which is a smoke depth.** Under
//! the gate's default engine the budget is wall-clock, not a fixed case
//! count, so the total differs on every run and no run's number is
//! reproducible. Measured 2026-09-10 with the gate's own command, twice in a
//! row: 207_437 then 241_990 rng inputs, both over the same 6 committed
//! corpus inputs, both ending `exit reason: max duration (1s - default)
//! exceeded`. Depth beyond that second is a separate explicit invocation —
//! `cargo bolero test fuzz_parse_revive_event --engine libfuzzer` — which no
//! lane in this repository runs.
//!
//! **Seed corpus: six files, present and used.** `corpus/` holds
//! `valid_genesis`, `valid_root_updated`, `wrong_topic_count`,
//! `bad_address_padding`, `trailing_byte` and `empty_topics`, written by
//! `tests/regenerate_corpus.rs` and counted as `corpus inputs: 6` in the run
//! line above, so the generator mutates outward from real shapes rather than
//! from nothing. `crashes/` holds only `.gitkeep`, i.e. no reproducer has
//! ever been found and committed.
//!
//! Note what this target does *not* decide: it establishes only that
//! arbitrary bytes cannot panic the parser. Whether a log that parses
//! belongs to this Organisation is the caller's contract-address check, which
//! is REQ-9vwcwc's business and is gated in `tests/log_ownership.rs` — the
//! target that holds the decision itself, `log_is_ours`, with its boundaries.
//! `tests/contract_address_filter.rs` is the *decoder's* half (REQ-5upq6n,
//! that the emitting address is reported byte for byte) and its own header
//! explicitly declines to claim REQ-9vwcwc; this line pointed there until
//! review round 4's finding 4. Round 2's finding 6 made the identical
//! correction to the identical sentence in `tests/decode_revive_event.rs`;
//! this copy was not swept with it.
//!
//! Run this target alone with `cargo test --test fuzz_parse_revive_event`.

use std::panic::AssertUnwindSafe;

use bolero::check;
use on_chain_client::decode::dispatch::{PASEO_AH_SPEC_VERSION, for_runtime};

fn main() {
    let decoder = for_runtime(PASEO_AH_SPEC_VERSION)
        .expect("pinned Paseo AH decoder must resolve");
    // bolero wraps each iteration in `catch_unwind`, which needs the closure's
    // captures to be `RefUnwindSafe`. `&dyn Decoder` is not, but every impl is
    // a stateless unit struct, so an unwind can't leave it inconsistent —
    // assert it. Deref the wrapper inside the closure (not its `.0` field) so
    // the closure captures the `AssertUnwindSafe` wrapper itself.
    // (`parse_revive_event` needs no `use Decoder`: the receiver is the
    // `dyn Decoder` trait object, which already names the trait.)
    let decoder = AssertUnwindSafe(decoder);
    check!().for_each(move |input: &[u8]| {
        let _ = (*decoder).parse_revive_event(input);
    });
}
