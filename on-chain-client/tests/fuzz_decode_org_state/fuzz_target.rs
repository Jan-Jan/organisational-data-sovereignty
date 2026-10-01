//! verifies: REQ-sx5b6g, LLR-yhw34z
//!
//! Fuzz target: `Decoder::decode_org_state` must never panic on arbitrary
//! bytes. Reaches the decoder through the public `for_runtime` path — the
//! exact surface `OrgRegistryClient::get_org_state` uses.
//!
//! REQ-sx5b6g implements RC-8w9wtp, and this target is its evidence for the
//! *storage* decode path; `fuzz_parse_revive_event` covers the event path and
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
//! row: 182_624 then 261_916 rng inputs, both over the same 4 committed
//! corpus inputs, both ending `exit reason: max duration (1s - default)
//! exceeded`. Depth beyond that second is a separate explicit invocation —
//! `cargo bolero test fuzz_decode_org_state --engine libfuzzer` — which no
//! lane in this repository runs.
//!
//! **Seed corpus: four files, present and used.** `corpus/` holds
//! `valid_epoch_7`, `epoch_u64_max`, `epoch_overflow` and `wrong_length_95`,
//! written by `tests/regenerate_corpus.rs` and counted as `corpus inputs: 4`
//! in the run line above, so the generator mutates outward from real shapes
//! rather than from nothing. `crashes/` holds only `.gitkeep`, i.e. no
//! reproducer has ever been found and committed.
//!
//! Run this target alone with `cargo test --test fuzz_decode_org_state`.

use std::panic::AssertUnwindSafe;

use bolero::check;
use on_chain_client::decode::dispatch::{PASEO_AH_SPEC_VERSION, for_runtime};

fn main() {
    let decoder = for_runtime(PASEO_AH_SPEC_VERSION)
        .expect("pinned Paseo AH decoder must resolve");
    // bolero runs each iteration inside `catch_unwind`, which requires the
    // closure's captures to be `RefUnwindSafe`. `&dyn Decoder` is not
    // `RefUnwindSafe` by default, but every `Decoder` impl is a stateless unit
    // struct, so an unwind cannot leave it observably inconsistent — assert it.
    // Deref the wrapper inside the closure (rather than capturing its `.0`
    // field) so the closure captures the `AssertUnwindSafe` wrapper itself.
    // (`decode_org_state` needs no `use Decoder`: the receiver is the
    // `dyn Decoder` trait object, which already names the trait.)
    let decoder = AssertUnwindSafe(decoder);
    check!().for_each(move |input: &[u8]| {
        // Property: any byte slice yields Ok/Err, never a panic/abort.
        let _ = (*decoder).decode_org_state(input);
    });
}
