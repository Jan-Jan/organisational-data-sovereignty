//! Fuzz target: `Envelope` postcard decode must never panic on arbitrary
//! bytes, and neither must `decode_delta` on arbitrary delta bytes.
//!
//! **Two shapes per input.** Shape 1 decodes the bytes as a whole envelope
//! and, if that succeeds, calls `decode_delta` on the result; whether a
//! fuzz input reaches that inner call is not measured. Shape 2 puts the fuzz
//! bytes where the delta goes, so the inner decode runs on every iteration:
//! it is what evidences `decode_delta`.
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run a single target with
//! `cargo test -p org-node --test fuzz_envelope_decode`; deep-fuzz with
//! `cargo bolero test fuzz_envelope_decode --engine libfuzzer`.
//!
//! verifies: REQ-9g6as6, LLR-v2y6sw

use bolero::check;
use org_node::envelope::Envelope;
use org_node::ids::OrgId;
use org_node::SequenceNumber;

fn main() {
    check!().for_each(|bytes: &[u8]| {
        // Shape 1 — the bytes are the whole envelope: postcard's own decode
        // surface.
        if let Ok(env) = postcard::from_bytes::<Envelope>(bytes) {
            let _ = env.decode_delta();
        }

        // Shape 2 — the bytes are the DELTA. `decode_delta` reads only
        // `delta_bytes`. Decoding arbitrary bytes as a `Delta` must only ever
        // return Ok or Err, never panic.
        let env = Envelope {
            org_id: OrgId::new([5u8; 20]),
            parent_seq: SequenceNumber::new(1),
            delta_bytes: bytes.to_vec(),
        };
        let _ = env.decode_delta();
    });
}
