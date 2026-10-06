//! Fuzz target: `Envelope` postcard decode must never panic on arbitrary
//! bytes, and neither must `decode_delta` on arbitrary delta bytes.
//!
//! **Two shapes per input, added 2026-10-04.** The original target decoded the
//! bytes as a whole envelope and then, *if that succeeded*, called
//! `decode_delta` on the result. The guard never passed: bolero's default
//! `&[u8]` generator produces slices far shorter than the ~86 bytes the
//! signed envelope of the time needed (20-byte org id, a varint sequence, a
//! length-prefixed delta, a 64-byte signature), so in 200 000 iterations not
//! one input parsed. The doc comment claimed the target "also drives
//! `decode_delta`"; it never did. Measured by panicking inside `decode_delta`
//! and watching this target pass. The second shape puts the fuzz bytes where
//! the delta goes, so the inner decode runs on every iteration.
//!
//! *Merged 2026-10-05 into worktree-person-shared-types:* the `Envelope` has no
//! signature any more (owner ruling, REQ-ag6kqm), so a whole envelope needs
//! only 22 bytes and the `sig_bytes` visitor no longer exists. Whether shape 1
//! now reaches `decode_delta` has not been re-measured; shape 2 is what
//! evidences the inner decode.
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
        // surface. Review round 4 (on the signed envelope) found the body of
        // the `if let Ok` below never entered; not re-measured for the
        // unsigned `Envelope`.
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
