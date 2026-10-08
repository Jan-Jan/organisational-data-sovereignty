//! verifies: LLR-m3tjvp, LLR-a9bb7b
//!
//! Fuzz target: the signatory-set decoders (SDD-rxfu6h) must never panic on
//! arbitrary bytes. `decode_original_account` and `decode_proxy_delegates`
//! decode chain storage values the caller does not control, so any byte
//! slice must yield `Ok`/`Err`, never a panic, an abort or an allocation the
//! input's length does not bound (LLR-a9bb7b: a length prefix larger than the
//! bytes that follow is `Malformed`, checked before anything is allocated).
//!
//! It is a `harness = false` binary in the shape of `fuzz_decode_org_state`:
//! a green run means bolero exhausted its time budget (one second under the
//! gate's default engine, a smoke depth) without panicking; it reports no
//! pass count. Depth beyond that is
//! `cargo bolero test fuzz_signatory_set --engine libfuzzer`.
//!
//! Run this target alone with
//! `cargo test --features test-support,write --test fuzz_signatory_set`.

use bolero::check;
use on_chain_client::write::signatory_set::{decode_original_account, decode_proxy_delegates};

fn main() {
    check!().for_each(|bytes: &[u8]| {
        // Property: any byte slice yields Ok/Err, never a panic/abort.
        let _ = decode_original_account(bytes);
        let _ = decode_proxy_delegates(bytes);
    });
}
