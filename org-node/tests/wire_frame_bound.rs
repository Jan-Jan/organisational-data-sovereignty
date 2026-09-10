#![cfg(all(feature = "transport", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Frame bound on the org-node channel: a `WireMessage` round-trips through
//! `encode_frame`/`decode_body`, and bodies above `MAX_FRAME` are refused on
//! both the encode and the decode side. The round trip is the normal case;
//! the two rejections are the abnormal-input cases.

use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::test_fixtures::admit_member_delta;
use org_node::transport::wire::{decode_body, encode_frame, WireMessage};
use org_node::transport::{TransportError, MAX_FRAME};
use org_node::SignedDeltaEnvelope;

fn sample_msg() -> WireMessage {
    let admin = SigningKeypair::from_seed([1u8; 32]);
    let (delta, _) = admit_member_delta(&admin);
    let env = SignedDeltaEnvelope::build(OrgId::new([5u8; 20]), 2, &delta, &admin).unwrap();
    WireMessage { envelope: env, org_secret: Some([9u8; 32]), genesis_snapshot: None }
}

// verifies: REQ-eg5j8u
#[test]
fn frame_round_trips() {
    let msg = sample_msg();
    let framed = encode_frame(&msg).unwrap();
    // strip the 4-byte length prefix
    let len = u32::from_le_bytes(framed[0..4].try_into().unwrap()) as usize;
    assert_eq!(len, framed.len() - 4);
    let back = decode_body(&framed[4..]).unwrap();
    assert_eq!(back, msg);
}

// verifies: REQ-eg5j8u
#[test]
fn oversize_body_is_rejected() {
    // A body claiming > MAX_FRAME must be rejected by decode_body.
    let big = vec![0u8; MAX_FRAME + 1];
    assert!(matches!(decode_body(&big), Err(TransportError::FrameTooLarge(_))));
}

// verifies: REQ-eg5j8u
#[test]
fn oversize_message_is_rejected_on_encode() {
    let mut msg = sample_msg();
    msg.genesis_snapshot = Some(vec![0u8; MAX_FRAME + 1]);
    assert!(matches!(encode_frame(&msg), Err(TransportError::FrameTooLarge(_))));
}
