#![cfg(feature = "test-support")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The Wire message's encoding and bound: a `WireMessage` round-trips through
//! `encode_message`/`decode_message`, and bodies above `MAX_MESSAGE` are
//! refused on both the encode and the decode side. The round trip is the
//! normal case; the two rejections are the abnormal-input cases. Also the six
//! kinds of Wire message (LLR-xr88a8), the refusals of a malformed
//! revocation or acknowledgement, and `peek_admission_request`
//! (LLR-g9n9w9). The codec is `org_node::wire`, with no length prefix; the
//! transport's framing is org-io's.

use org_members::{Handle, MemberId, Name, Surname};
use org_node::ids::{InviteId, OrgId};
use org_node::revocation::{Acknowledgement, RevocationNotice, Signature64};
use org_node::test_fixtures::{admit_member_delta, bob_absence_notice, device_key, member_key};
use org_node::wire::{decode_message, encode_message, peek_admission_request, WireMessage, MAX_MESSAGE};
use org_node::{Envelope, Epoch, MemberSeed, OrgNodeError, OrgPrivateKey, RootHash, SequenceNumber};

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

fn sample_admission_request() -> WireMessage {
    WireMessage::AdmissionRequest {
        org_id: the_org(),
        invite_id: InviteId::new([0x61; 32]),
        name: Name::parse("Carol").unwrap(),
        surname: Surname::parse("Coder").unwrap(),
        preferred_handles: vec![Handle::parse("carol").unwrap(), Handle::parse("cc").unwrap()],
        member_key: member_key(0x36),
        device_key: device_key(0x24),
        addresses: vec!["198.51.100.7:7000".into()],
    }
}

fn sample_organisation_request() -> WireMessage {
    WireMessage::OrganisationRequest { org_id: the_org(), addresses: vec!["203.0.113.9:7000".into(), String::new()] }
}

fn sample_organisation() -> WireMessage {
    WireMessage::Organisation {
        org_id: the_org(),
        epoch: Epoch::new(4),
        record_snapshot: vec![4, 5, 6],
        org_private_key: OrgPrivateKey::from([0x77u8; 32]),
    }
}

/// Every kind, with its variant index.
fn the_six_kinds() -> [(WireMessage, u8); 6] {
    [
        (sample_msg(), 0u8),
        (WireMessage::Revocation(sample_notice()), 1),
        (WireMessage::Acknowledgement(sample_acknowledgement()), 2),
        (sample_admission_request(), 3),
        (sample_organisation_request(), 4),
        (sample_organisation(), 5),
    ]
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

/// The postcard body of `msg`: its encoding, which carries no length prefix.
fn body_of(msg: &WireMessage) -> Vec<u8> {
    encode_message(msg).unwrap()
}

// verifies: LLR-er2x8n
#[test]
fn an_encoded_message_is_its_postcard_body_with_no_length_prefix() {
    let message = sample_msg();
    let body = encode_message(&message).unwrap();
    assert_eq!(body, postcard::to_allocvec(&message).unwrap());
    assert_eq!(decode_message(&body).unwrap(), message);
}

// verifies: LLR-8kh3zf
#[test]
fn a_body_over_the_bound_is_refused_before_decoding() {
    let body = vec![0u8; MAX_MESSAGE + 1];
    assert!(matches!(decode_message(&body), Err(OrgNodeError::MessageTooLarge { length }) if length == MAX_MESSAGE + 1));
}

// verifies: LLR-pkruy8
#[test]
fn a_body_that_is_not_a_wire_message_is_malformed() {
    assert!(matches!(decode_message(&[0xff, 0xff, 0xff]), Err(OrgNodeError::MalformedMessage)));
}

/// verifies: LLR-xr88a8, LLR-dc45ur, LLR-378cj4
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
        let body = encode_message(&message).unwrap();
        assert_eq!(body[0], index);
        assert_eq!(decode_message(&body).unwrap(), message);
        assert_eq!(message.org_id(), the_org());
    }
}

/// verifies: LLR-xr88a8, LLR-dc45ur, LLR-378cj4
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
        assert!(matches!(decode_message(&body), Err(OrgNodeError::MalformedMessage)), "{body:02x?}");
    }
}

/// verifies: LLR-xr88a8
///
/// Abnormal: an Organisation-information body
/// with bytes left over after its Organisation private key is `Malformed`,
/// without a panic, as the other two kinds are.
#[test]
fn an_org_information_body_with_trailing_bytes_is_refused() {
    let whole = body_of(&sample_msg());
    assert_eq!(decode_message(&whole).unwrap(), sample_msg(), "the body without trailing bytes decodes");
    for trailing in [vec![0u8], vec![0xFF; 7]] {
        let mut body = whole.clone();
        body.extend(trailing);
        assert!(matches!(decode_message(&body), Err(OrgNodeError::MalformedMessage)), "{body:02x?}");
    }
}

/// verifies: LLR-xr88a8
///
/// Normal: each of the six kinds round-trips with its variant index and its
/// payload, as the same kind. Abnormal: a body of any kind with a leftover
/// byte, or the body of any kind under an unknown variant index (every first
/// byte from 6 to 0xFF), is `Malformed`, without a panic.
#[test]
fn exactly_six_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused() {
    for (message, index) in &the_six_kinds() {
        let body = body_of(message);
        assert_eq!(body[0], *index);
        let decoded = decode_message(&body).unwrap();
        assert_eq!(&decoded, message, "kind {index} decodes to the message it was encoded from");
        assert_eq!(std::mem::discriminant(&decoded), std::mem::discriminant(message), "kind {index} as the same kind");

        let mut leftover = body.clone();
        leftover.push(0);
        assert!(
            matches!(decode_message(&leftover), Err(OrgNodeError::MalformedMessage)),
            "kind {index} with a leftover byte"
        );

        for unknown_index in 6u8..=0xFF {
            let mut unknown_kind = body.clone();
            unknown_kind[0] = unknown_index;
            assert!(
                matches!(decode_message(&unknown_kind), Err(OrgNodeError::MalformedMessage)),
                "kind {index}'s payload under index {unknown_index}"
            );
        }
    }
}

/// verifies: LLR-xr88a8
///
/// Normal: each of the six kinds round-trips with variant index 0 to 5, its
/// fields in order. Abnormal: each body cut short by one byte, anywhere it
/// ends early, is `Malformed`, without a panic.
#[test]
fn the_six_kinds_round_trip_with_their_indices() {
    for (message, index) in the_six_kinds() {
        let body = encode_message(&message).unwrap();
        assert_eq!(body[0], index);
        assert_eq!(decode_message(&body).unwrap(), message);
        for cut in 1..body.len() {
            assert!(
                matches!(decode_message(&body[..cut]), Err(OrgNodeError::MalformedMessage)),
                "kind {index} cut to {cut} bytes"
            );
        }
    }
    // The admission request's fields, in order: the identifiers first, the
    // addresses last.
    let body = encode_message(&sample_admission_request()).unwrap();
    assert_eq!(&body[1..21], &[3u8; 20], "the Organisation right after the index");
    assert_eq!(&body[21..53], &[0x61u8; 32], "then the invite identifier");
    assert!(body.ends_with(b"198.51.100.7:7000"), "the addresses last");
}

/// verifies: LLR-xr88a8, REQ-zxgd46
///
/// Abnormal: in every kind that carries one, a key field of 31 bytes — the
/// Organisation private key of Organisation information and of an
/// organisation answer, an admission request's Member-as-a-group key and
/// DevicePublicKey — makes the body not decode, without a panic. A key is a
/// fixed 32-byte array, so a 33rd byte is a leftover byte after it, which
/// `exactly_six_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused`
/// refuses.
#[test]
fn a_key_field_that_is_not_32_bytes_does_not_decode() {
    // Organisation information and an answer: the key is last; 31 bytes of it.
    for message in [sample_msg(), sample_organisation()] {
        let body = body_of(&message);
        let short = &body[..body.len() - 1];
        assert!(matches!(decode_message(short), Err(OrgNodeError::MalformedMessage)), "31 key bytes");
    }
    // An admission request whose Member-as-a-group key or DevicePublicKey is
    // 31 bytes: the tuple after the Handles is re-encoded with one byte less.
    let WireMessage::AdmissionRequest { org_id, invite_id, name, surname, preferred_handles, member_key, device_key, addresses } =
        sample_admission_request()
    else {
        unreachable!("fixture: an admission request")
    };
    let head = postcard::to_allocvec(&(3u8, org_id, invite_id, name, surname, preferred_handles)).unwrap();
    for (member, device) in [
        (member_key.as_bytes()[..31].to_vec(), device_key.as_bytes().to_vec()),
        (member_key.as_bytes().to_vec(), device_key.as_bytes()[..31].to_vec()),
    ] {
        let mut body = head.clone();
        body.extend(member);
        body.extend(device);
        body.extend(postcard::to_allocvec(&addresses).unwrap());
        assert!(matches!(decode_message(&body), Err(OrgNodeError::MalformedMessage)), "a 31-byte key");
    }
}

/// verifies: LLR-xr88a8
///
/// Normal: `org_id()` returns the Organisation of each of the six kinds.
#[test]
fn org_id_returns_every_kinds_organisation() {
    for (message, index) in the_six_kinds() {
        assert_eq!(message.org_id(), the_org(), "kind {index}");
    }
    let elsewhere = OrgId::new([9; 20]);
    let request = WireMessage::OrganisationRequest { org_id: elsewhere, addresses: vec![] };
    assert_eq!(request.org_id(), elsewhere);
}

/// verifies: LLR-g9n9w9
///
/// Normal: on an admission request, `peek_admission_request` returns its
/// Organisation and invite identifier, and reads no byte after them: the
/// first 53 bytes alone give the same answer, whatever follows.
#[test]
fn peek_admission_request_reads_only_the_index_org_id_and_invite_id() {
    let body = body_of(&sample_admission_request());
    let expected = Some((the_org(), InviteId::new([0x61; 32])));
    assert_eq!(peek_admission_request(&body), expected);
    assert_eq!(peek_admission_request(&body[..53]), expected, "only the index and the two identifiers are read");
    let mut garbage = body[..53].to_vec();
    garbage.extend([0xFF; 9]);
    assert_eq!(peek_admission_request(&garbage), expected, "what follows is not read");
    assert!(decode_message(&garbage).is_err(), "though the whole body does not decode");
}

/// verifies: LLR-g9n9w9
///
/// Abnormal: another variant index, or an admission request shorter than its
/// index and two identifiers, is `None`, without a panic.
#[test]
fn peek_admission_request_is_none_for_another_index_or_a_short_body() {
    for (message, index) in the_six_kinds() {
        if index != 3 {
            assert_eq!(peek_admission_request(&body_of(&message)), None, "kind {index}");
        }
    }
    let body = body_of(&sample_admission_request());
    for cut in [0usize, 1, 21, 52] {
        assert_eq!(peek_admission_request(&body[..cut]), None, "{cut} bytes");
    }
}

/// verifies: LLR-378cj4
///
/// Boundary of `Signature64`'s decode: exactly 64 signature bytes decode;
/// 0, 63 and 65 are `Malformed` (the length, not trailing bytes, refuses 65).
#[test]
fn an_acknowledgement_signature_decodes_only_at_exactly_64_bytes() {
    assert_eq!(
        decode_message(&acknowledgement_with_signature_of(64)).unwrap(),
        WireMessage::Acknowledgement(sample_acknowledgement())
    );
    for length in [0, 63, 65] {
        assert!(
            matches!(decode_message(&acknowledgement_with_signature_of(length)), Err(OrgNodeError::MalformedMessage)),
            "{length} signature bytes"
        );
    }
}

// Abnormal: an Organisation-information body that ends before its record
// snapshot, or before the 32 bytes of its Organisation private key, does not
// decode, and does not panic.
// verifies: LLR-xr88a8, REQ-zxgd46
#[test]
fn an_organisation_information_body_without_its_snapshot_or_key_does_not_decode() {
    let body = body_of(&sample_msg());
    // Organisation information's index, then the Envelope and nothing.
    let mut no_snapshot = vec![0u8];
    no_snapshot.extend(postcard::to_allocvec(&sample_envelope()).unwrap());
    for cut in [no_snapshot, body[..body.len() - 32].to_vec(), body[..body.len() - 1].to_vec()] {
        assert!(matches!(decode_message(&cut), Err(OrgNodeError::MalformedMessage)), "{} bytes", cut.len());
    }
}

// verifies: REQ-eg5j8u, LLR-8kh3zf
#[test]
fn oversize_body_is_rejected() {
    // A body claiming > MAX_MESSAGE must be rejected by decode_message.
    let big = vec![0u8; MAX_MESSAGE + 1];
    assert!(matches!(decode_message(&big), Err(OrgNodeError::MessageTooLarge { .. })));
}

// verifies: REQ-eg5j8u, LLR-sc6zuh
#[test]
fn oversize_message_is_rejected_on_encode() {
    let msg = WireMessage::OrgInformation {
        envelope: sample_envelope(),
        record_snapshot: vec![0u8; MAX_MESSAGE + 1],
        org_private_key: OrgPrivateKey::from([9u8; 32]),
    };
    assert!(matches!(encode_message(&msg), Err(OrgNodeError::MessageTooLarge { .. })));
}

// Bytes that are within the bound but are not an encoded message are refused
// with the typed Malformed error rather than panicking. This is the decode
// side's abnormal-input case that is NOT about the size bound.
// verifies: REQ-9g6as6, LLR-pkruy8
#[test]
fn a_body_that_is_not_an_encoded_message_is_refused() {
    assert!(matches!(
        decode_message(b"not a postcard WireMessage"),
        Err(OrgNodeError::MalformedMessage)
    ));
    // Empty input is the boundary of the same case.
    assert!(matches!(decode_message(&[]), Err(OrgNodeError::MalformedMessage)));
    // A truncated prefix of a real body: valid bytes, cut short.
    let body = encode_message(&sample_msg()).unwrap();
    let truncated = &body[..body.len() - 1];
    assert!(matches!(decode_message(truncated), Err(OrgNodeError::MalformedMessage)));
}

// A body of exactly MAX_MESSAGE is within the bound, so the refusal is `>`
// and not `>=` — the boundary itself must not be rejected for being the
// boundary.
// verifies: REQ-eg5j8u, LLR-8kh3zf
#[test]
fn a_body_of_exactly_the_bound_is_not_refused_for_its_size() {
    // 0xff, not zero: an all-zero body IS a valid message — Organisation
    // information with an empty envelope, an empty snapshot and an all-zero
    // key. A leading run of 0xff bytes is a variant-index varint that
    // overflows, so this body is not.
    let at_bound = vec![0xffu8; MAX_MESSAGE];
    // It is not a valid message, so it is refused — but as Malformed, which is
    // the decode verdict, never as MessageTooLarge, which is the size verdict.
    assert!(matches!(decode_message(&at_bound), Err(OrgNodeError::MalformedMessage)));
}
