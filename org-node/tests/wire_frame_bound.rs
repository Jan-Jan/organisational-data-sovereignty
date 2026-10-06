#![cfg(all(feature = "transport", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Frame bound on the org-node channel: a `WireMessage` round-trips through
//! `encode_frame`/`decode_body`, and bodies above `MAX_FRAME` are refused on
//! both the encode and the decode side. The round trip is the normal case;
//! the two rejections are the abnormal-input cases.

use org_node::ids::OrgId;
use org_node::test_fixtures::admit_member_delta;
use org_node::transport::wire::{decode_body, encode_frame, WireMessage};
use org_node::transport::{TransportError, MAX_FRAME};
use org_node::{Envelope, MemberSeed, OrgPrivateKey, SequenceNumber};

fn sample_envelope() -> Envelope {
    let (delta, _) = admit_member_delta(&MemberSeed::from([1u8; 32]).x25519_keypair());
    Envelope::build(OrgId::new([5u8; 20]), SequenceNumber::new(2), &delta).unwrap()
}

fn sample_msg() -> WireMessage {
    WireMessage::OrgInformation {
        envelope: sample_envelope(),
        record_snapshot: vec![1, 2, 3],
        org_private_key: OrgPrivateKey::from([9u8; 32]),
    }
}

/// The postcard body of `msg`, without the frame's length prefix.
fn body_of(msg: &WireMessage) -> Vec<u8> {
    encode_frame(msg).unwrap()[4..].to_vec()
}

// Normal: each kind round-trips through a frame, begins with its variant
// index (0 Organisation information, 1 revocation) and hands back its
// Envelope. Abnormal: a body whose index is neither 0 nor 1 does not decode.
// verifies: LLR-js9dsu, REQ-3dsweu
#[test]
fn the_two_kinds_round_trip_and_a_body_of_neither_kind_is_refused() {
    let revocation = WireMessage::Revocation { envelope: sample_envelope() };
    for msg in [sample_msg(), revocation.clone()] {
        assert_eq!(decode_body(&body_of(&msg)).unwrap(), msg);
        assert_eq!(msg.envelope(), &sample_envelope());
    }
    assert_eq!(body_of(&sample_msg())[0], 0);
    assert_eq!(body_of(&revocation)[0], 1);
    let mut other = body_of(&revocation);
    for index in [2u8, 3, 0x7f] {
        other[0] = index;
        assert!(matches!(decode_body(&other), Err(TransportError::Malformed)), "index {index}");
    }
}

// Abnormal: an Organisation-information body that ends before its record
// snapshot, or before the 32 bytes of its Organisation private key, does not
// decode, and does not panic.
// verifies: LLR-js9dsu, REQ-c29s93
#[test]
fn an_organisation_information_body_without_its_snapshot_or_key_does_not_decode() {
    let body = body_of(&sample_msg());
    let mut no_snapshot = body_of(&WireMessage::Revocation { envelope: sample_envelope() });
    no_snapshot[0] = 0; // Organisation information's index, then the Envelope and nothing
    for cut in [no_snapshot, body[..body.len() - 32].to_vec(), body[..body.len() - 1].to_vec()] {
        assert!(matches!(decode_body(&cut), Err(TransportError::Malformed)), "{} bytes", cut.len());
    }
}

// verifies: REQ-eg5j8u, LLR-fa7jt8, LLR-er2x8n
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

// verifies: REQ-eg5j8u, LLR-8kh3zf
#[test]
fn oversize_body_is_rejected() {
    // A body claiming > MAX_FRAME must be rejected by decode_body.
    let big = vec![0u8; MAX_FRAME + 1];
    assert!(matches!(decode_body(&big), Err(TransportError::FrameTooLarge(_))));
}

// verifies: REQ-eg5j8u, LLR-sc6zuh
#[test]
fn oversize_message_is_rejected_on_encode() {
    let msg = WireMessage::OrgInformation {
        envelope: sample_envelope(),
        record_snapshot: vec![0u8; MAX_FRAME + 1],
        org_private_key: OrgPrivateKey::from([9u8; 32]),
    };
    assert!(matches!(encode_frame(&msg), Err(TransportError::FrameTooLarge(_))));
}

// ---- added 2026-10-03 by the architecture tooth ----------------------------

// Bytes that are within the bound but are not an encoded message are refused
// with the typed Malformed error rather than panicking. This is the decode
// side's abnormal-input case that is NOT about the size bound.
// verifies: REQ-9g6as6, LLR-pkruy8
#[test]
fn a_body_that_is_not_an_encoded_message_is_refused() {
    assert!(matches!(
        decode_body(b"not a postcard WireMessage"),
        Err(TransportError::Malformed)
    ));
    // Empty input is the boundary of the same case.
    assert!(matches!(decode_body(&[]), Err(TransportError::Malformed)));
    // A truncated prefix of a real body: valid bytes, cut short.
    let framed = encode_frame(&sample_msg()).unwrap();
    let truncated = &framed[4..framed.len() - 1];
    assert!(matches!(decode_body(truncated), Err(TransportError::Malformed)));
}

// A body of exactly MAX_FRAME is within the bound, so the refusal is `>` and
// not `>=` — the boundary itself must not be rejected for being the boundary.
// verifies: REQ-eg5j8u, LLR-8kh3zf
#[test]
fn a_body_of_exactly_the_bound_is_not_refused_for_its_size() {
    // 0xff, not zero: an all-zero body IS a valid message — Organisation
    // information with an empty envelope, an empty snapshot and an all-zero
    // key. A leading run of 0xff bytes is a variant-index varint that
    // overflows, so this body is not.
    let at_bound = vec![0xffu8; MAX_FRAME];
    // It is not a valid message, so it is refused — but as Malformed, which is
    // the decode verdict, never as FrameTooLarge, which is the size verdict.
    assert!(matches!(decode_body(&at_bound), Err(TransportError::Malformed)));
}
