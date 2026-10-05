//! Fuzz target: `SignedDeltaEnvelope` postcard decode must never panic on
//! arbitrary bytes, and neither must `decode_delta` on arbitrary delta bytes.
//!
//! **Two shapes per input, added 2026-10-04.** The original target decoded the
//! bytes as a whole envelope and then, *if that succeeded*, called
//! `decode_delta` on the result. The guard never passed: bolero's default
//! `&[u8]` generator produces slices far shorter than the ~86 bytes a
//! `SignedDeltaEnvelope` needs (20-byte org id, a varint sequence, a
//! length-prefixed delta, a 64-byte signature), so in 200 000 iterations not
//! one input parsed. The doc comment claimed the target "also drives
//! `decode_delta`"; it never did. Measured by panicking inside `decode_delta`
//! and watching this target pass. The second shape puts the fuzz bytes where
//! the delta goes, so the inner decode runs on every iteration.
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run a single target with
//! `cargo test -p org-node --test fuzz_envelope_decode`; deep-fuzz with
//! `cargo bolero test fuzz_envelope_decode --engine libfuzzer`.
//!
//! verifies: REQ-9g6as6, LLR-v2y6sw

use bolero::check;
use org_node::envelope::SignedDeltaEnvelope;
use org_node::ids::OrgId;

fn main() {
    check!().for_each(|bytes: &[u8]| {
        // Shape 1 — the bytes are the whole envelope. This is the hand-written
        // `sig_bytes` visitor's surface as well as postcard's own: the decode
        // ATTEMPT reaches the visitor. The body of the `if let Ok` below is
        // never entered — review round 4 put a `panic!()` in it and 165 787
        // iterations passed — because no fuzz input assembles a whole
        // envelope; `decode_delta` on a fuzz-built envelope is not evidenced
        // here.
        if let Ok(env) = postcard::from_bytes::<SignedDeltaEnvelope>(bytes) {
            let _ = env.decode_delta();
        }

        // Shape 2 — the bytes are the DELTA. No signature is needed: this is
        // the decode, not the verification, and `decode_delta` reads only
        // `delta_bytes`. Decoding arbitrary bytes as a `Delta` must only ever
        // return Ok or Err, never panic.
        let env = SignedDeltaEnvelope {
            org_id: OrgId::new([5u8; 20]),
            parent_seq: 1,
            delta_bytes: bytes.to_vec(),
            signature: [0u8; 64],
        };
        let _ = env.decode_delta();
    });
}
