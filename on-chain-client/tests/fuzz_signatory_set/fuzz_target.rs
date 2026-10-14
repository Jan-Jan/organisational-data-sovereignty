//! verifies: LLR-m3tjvp, LLR-a9bb7b, LLR-n2vx5q
//!
//! Fuzz target: the signatory-set decoders (SDD-rxfu6h) must never panic on
//! arbitrary bytes. `decode_original_account`, `decode_proxy_delegates` and
//! `decode_proxy_definitions` decode chain storage values the caller does not
//! control, so any byte slice must yield `Ok`/`Err`, never a panic, an abort
//! or an allocation the input's length does not bound (LLR-a9bb7b,
//! LLR-n2vx5q: a length prefix larger than the bytes that follow is
//! `Malformed`, checked before anything is allocated). An accepted value
//! accounts for every byte (LLR-n2vx5q: trailing or missing bytes are
//! `Malformed`), and the delegates are exactly the definitions' delegates.
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
use on_chain_client::write::signatory_set::{
    decode_original_account, decode_proxy_definitions, decode_proxy_delegates,
};
use parity_scale_codec::{Compact, Encode};

/// One definition's bytes and the deposit's, as LLR-n2vx5q lays them out.
const DEFINITION_BYTES: usize = 32 + 1 + 4;
const DEPOSIT_BYTES: usize = 16;

fn main() {
    check!().for_each(|bytes: &[u8]| {
        // Property: any byte slice yields Ok/Err, never a panic/abort.
        let _ = decode_original_account(bytes);
        let delegates = decode_proxy_delegates(bytes);
        let definitions = decode_proxy_definitions(bytes);
        if let Ok(definitions) = &definitions {
            let prefix = Compact(definitions.len() as u32).encoded_size();
            assert_eq!(
                prefix + definitions.len() * DEFINITION_BYTES + DEPOSIT_BYTES,
                bytes.len(),
                "an accepted value accounts for every byte"
            );
        }
        assert_eq!(
            delegates.ok(),
            definitions
                .ok()
                .map(|definitions| definitions.iter().map(|definition| definition.delegate).collect()),
            "the delegates are the definitions' delegates"
        );
    });
}
