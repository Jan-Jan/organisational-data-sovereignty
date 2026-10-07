#![cfg(all(feature = "transport", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Frame bound on the org-node channel: a `WireMessage` round-trips through
//! `encode_frame`/`decode_body`, and bodies above `MAX_FRAME` are refused on
//! both the encode and the decode side. The round trip is the normal case;
//! the two rejections are the abnormal-input cases. Since S3 T4 also the
//! three kinds of Wire message and the refusals of a malformed revocation
//! or acknowledgement.

use org_members::MemberId;
use org_node::ids::OrgId;
use org_node::revocation::{Acknowledgement, RevocationNotice, Signature64};
use org_node::test_fixtures::{admit_member_delta, bob_absence_notice, device_key};
use org_node::transport::wire::{decode_body, encode_frame, WireMessage};
use org_node::transport::{TransportError, MAX_FRAME};
use org_node::{Envelope, Epoch, MemberSeed, OrgPrivateKey, RootHash, SequenceNumber};

/// The one Organisation every fixture names.
fn the_org() -> OrgId {
    OrgId::new([3u8; 20])
}

fn sample_envelope() -> Envelope {
    let (delta, _) = admit_member_delta(&MemberSeed::from([1u8; 32]).x25519_keypair());
    Envelope::build(the_org(), SequenceNumber::new(2), &delta).unwrap()
}

fn sample_msg() -> WireMessage {
    WireMessage::OrgInformation {
        envelope: sample_envelope(),
        record_snapshot: vec![1, 2, 3],
        org_private_key: OrgPrivateKey::from([9u8; 32]),
    }
}

fn sample_notice() -> RevocationNotice {
    bob_absence_notice(the_org())
}

fn sample_acknowledgement() -> Acknowledgement {
    Acknowledgement {
        org_id: the_org(),
        member_id: MemberId::new([2u8; 32]),
        device: device_key(5),
        epoch: Epoch::new(3),
        root: RootHash::new([0x11; 32]),
        signature: Signature64([0x22; 64]),
    }
}

/// A revocation body: index 1, the sample notice's three identity fields,
/// and an org-members wire proof (`default_map`, `siblings`, `ending`) with
/// `sibling_count` siblings.
fn notice_with_siblings(sibling_count: usize) -> Vec<u8> {
    let notice = sample_notice();
    let mut body = vec![1u8];
    body.extend(postcard::to_allocvec(&(notice.org_id, notice.member_id, notice.device)).unwrap());
    let no_ending: Option<()> = None;
    body.extend(postcard::to_allocvec(&([0u8; 32], vec![[0u8; 32]; sibling_count], no_ending)).unwrap());
    body
}

/// An acknowledgement body: index 2, the sample acknowledgement's signed
/// fields, and `signature_length` signature bytes.
fn acknowledgement_with_signature_of(signature_length: usize) -> Vec<u8> {
    let ack = sample_acknowledgement();
    let mut body = vec![2u8];
    body.extend(postcard::to_allocvec(&(ack.org_id, ack.member_id, ack.device, ack.epoch, ack.root)).unwrap());
    body.extend(postcard::to_allocvec(&vec![0x22u8; signature_length]).unwrap());
    body
}

/// The postcard body of `msg`, without the frame's length prefix.
fn body_of(msg: &WireMessage) -> Vec<u8> {
    encode_frame(msg).unwrap()[4..].to_vec()
}

// *Replaced 2026-10-07 (change worktree-org-io-commit-workflow, S3 T4).* Was
// `the_two_kinds_round_trip_and_a_body_of_neither_kind_is_refused`: the
// revocation held an Envelope and index 2 was refused.
/// verifies: LLR-js9dsu, LLR-dc45ur, LLR-378cj4
///
/// Normal: each kind round-trips with variant index 0, 1 or 2, and
/// `org_id()` names its Organisation.
#[test]
fn the_three_kinds_round_trip_with_their_indices() {
    let kinds = [
        (sample_msg(), 0u8),
        (WireMessage::Revocation(sample_notice()), 1),
        (WireMessage::Acknowledgement(sample_acknowledgement()), 2),
    ];
    for (message, index) in kinds {
        let frame = encode_frame(&message).unwrap();
        assert_eq!(frame[4], index);
        assert_eq!(decode_body(&frame[4..]).unwrap(), message);
        assert_eq!(message.org_id(), the_org());
    }
}

/// verifies: LLR-js9dsu, LLR-dc45ur, LLR-378cj4
///
/// Abnormal: index 3, trailing bytes, a device that is not an Ed25519 point,
/// a proof over 256 siblings, or a signature of 63 bytes is `Malformed`,
/// without a panic.
#[test]
fn malformed_revocations_and_acknowledgements_are_refused() {
    let mut bad_index = body_of(&WireMessage::Revocation(sample_notice()));
    bad_index[0] = 3;
    let mut trailing = body_of(&WireMessage::Acknowledgement(sample_acknowledgement()));
    trailing.push(0);
    let mut trailing_notice = body_of(&WireMessage::Revocation(sample_notice()));
    trailing_notice.push(0);
    let not_a_point = [0xFFu8; 32];
    let mut notice_bad_device = vec![1u8];
    notice_bad_device.extend(postcard::to_allocvec(&(the_org(), sample_notice().member_id, not_a_point)).unwrap());
    notice_bad_device.extend(postcard::to_allocvec(&sample_notice().proof).unwrap());
    let refused = [
        bad_index,
        trailing,
        trailing_notice,
        notice_bad_device,
        notice_with_siblings(257),
        acknowledgement_with_signature_of(63),
    ];
    for body in refused {
        assert!(matches!(decode_body(&body), Err(TransportError::Malformed)), "{body:02x?}");
    }
}

/// verifies: LLR-js9dsu
///
/// Abnormal (review finding-3, 2026-10-07): an Organisation-information body
/// with bytes left over after its Organisation private key is `Malformed`,
/// without a panic, as the other two kinds are.
#[test]
fn an_org_information_body_with_trailing_bytes_is_refused() {
    let whole = body_of(&sample_msg());
    assert_eq!(decode_body(&whole).unwrap(), sample_msg(), "the body without trailing bytes decodes");
    for trailing in [vec![0u8], vec![0xFF; 7]] {
        let mut body = whole.clone();
        body.extend(trailing);
        assert!(matches!(decode_body(&body), Err(TransportError::Malformed)), "{body:02x?}");
    }
}

/// verifies: LLR-378cj4
///
/// Boundary of `Signature64`'s decode: exactly 64 signature bytes decode;
/// 0, 63 and 65 are `Malformed` (the length, not trailing bytes, refuses 65).
#[test]
fn an_acknowledgement_signature_decodes_only_at_exactly_64_bytes() {
    assert_eq!(
        decode_body(&acknowledgement_with_signature_of(64)).unwrap(),
        WireMessage::Acknowledgement(sample_acknowledgement())
    );
    for length in [0, 63, 65] {
        assert!(
            matches!(decode_body(&acknowledgement_with_signature_of(length)), Err(TransportError::Malformed)),
            "{length} signature bytes"
        );
    }
}

// Abnormal: an Organisation-information body that ends before its record
// snapshot, or before the 32 bytes of its Organisation private key, does not
// decode, and does not panic.
// verifies: LLR-js9dsu, REQ-c29s93
#[test]
fn an_organisation_information_body_without_its_snapshot_or_key_does_not_decode() {
    let body = body_of(&sample_msg());
    // Organisation information's index, then the Envelope and nothing.
    let mut no_snapshot = vec![0u8];
    no_snapshot.extend(postcard::to_allocvec(&sample_envelope()).unwrap());
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
