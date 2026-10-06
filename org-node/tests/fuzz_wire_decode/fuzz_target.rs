//! Fuzz target: `decode_body` must never panic on arbitrary bytes, and a body
//! that decodes is one of the two kinds and decodes again after re-encoding
//! (LLR-js9dsu). Shape 2 puts the fuzz bytes after a well-formed
//! Organisation-information prefix (variant index 0 and an Envelope), so every
//! iteration reaches the record snapshot and the Organisation private key:
//! a body that ends before the key's 32 bytes is refused as `Malformed`
//! (REQ-c29s93).
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run alone with
//! `cargo test -p org-node --features transport --test fuzz_wire_decode`;
//! deep-fuzz with `cargo bolero test fuzz_wire_decode --engine libfuzzer`.
//!
//! verifies: REQ-9g6as6, LLR-js9dsu, REQ-c29s93
#![allow(clippy::expect_used, clippy::panic)]

use bolero::check;
use org_node::envelope::Envelope;
use org_node::ids::OrgId;
use org_node::transport::wire::{decode_body, encode_frame, WireMessage};
use org_node::transport::TransportError;
use org_node::SequenceNumber;

fn main() {
    let envelope = Envelope { org_id: OrgId::new([5u8; 20]), parent_seq: SequenceNumber::new(1), delta_bytes: vec![1, 2, 3] };
    let mut info_prefix = postcard::to_allocvec(&WireMessage::Revocation { envelope }).expect("encode");
    info_prefix[0] = 0;
    check!().for_each(|bytes: &[u8]| {
        // Shape 1 — the bytes are the whole body.
        if let Ok(msg) = decode_body(bytes) {
            let framed = encode_frame(&msg).expect("a decoded message re-encodes");
            assert_eq!(decode_body(&framed[4..]).expect("and decodes again"), msg);
        }
        // Shape 2 — the bytes follow an Organisation-information prefix.
        let mut body = info_prefix.clone();
        body.extend_from_slice(bytes);
        match decode_body(&body) {
            Ok(WireMessage::OrgInformation { org_private_key, .. }) => {
                assert_eq!(org_private_key.expose_secret().len(), 32);
            }
            Ok(WireMessage::Revocation { .. }) => panic!("variant index 0 decoded as a revocation"),
            Err(TransportError::Malformed) => {}
            Err(other) => panic!("refused as {other:?}, not as Malformed"),
        }
    });
}
