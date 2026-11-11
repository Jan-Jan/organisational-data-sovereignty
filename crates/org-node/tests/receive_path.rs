#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The one receive path, by values (SDD-busuj3): `check_received(rng,
//! device_key, sender, body)` takes no chain state; it bounds and decodes the
//! body, applies the sender rule, and decides what is kept, answered or
//! refused in the store `device_key` names. No endpoint: each body is one
//! built here with `encode_message`, and the sender is a Device key the test
//! names. The check names its store by Device key and applies the sender
//! rule of LLR-283x6f and LLR-w7b9kd before any kind's arm.

mod support;
use support::*;

use org_members::DevicePublicKey;
use org_node::error::OrgNodeError;
use org_node::ids::{InviteId, OrgId};
use org_node::revocation::{sign_acknowledgement, RevocationNotice};
use org_node::service::{AdmissionRequestOutcome, Checked};
use org_node::store::RevokedDevice;
use org_node::test_fixtures::{bob_absence_notice, device_key, device_keypair, member_key};
use org_node::wire::{encode_message, WireMessage, MAX_MESSAGE};
use org_node::{Envelope, Epoch, Handle, MemberId, Name, OrgPrivateKey, PersonaId, SequenceNumber, Surname};
use rand::rngs::OsRng;

/// The Device of the rogue relay: listed in no record.
fn rogue() -> DevicePublicKey {
    device_keypair(ROGUE_SEED).device_key().unwrap()
}

/// B admitted (epoch 2), then A's removal of B (epoch 3, A's record only):
/// the setup and the notice A's removal would send B's Device.
fn removal(tag: &str) -> (Setup, RevocationNotice) {
    let story = admit_b(setup(tag));
    let b_device = story.b_device_keypair.device_key().unwrap();
    let (story, b_id) = remove_b(story);
    let proof = trie_of(&record_of(&story.node_a, story.org_id)).prove_absent(&b_id, &b_device).unwrap();
    let notice = RevocationNotice { org_id: story.org_id, member_id: b_id, device: b_device, proof };
    (story, notice)
}

/// A's Invite, its outstanding invite kept on A's disk: the Organisation and
/// the invite identifier.
fn invited(story: &mut Setup) -> (OrgId, InviteId) {
    let a_device = story.node_a.device_of(&story.a_persona_id);
    let invite = story.node_a.invite(&a_device, "Joe Joiner");
    (invite.org_id, invite.invite_id)
}

/// An admission request for (`org_id`, `invite_id`) naming the Device of
/// `device_seed`, with `addresses`.
fn admission_request(org_id: OrgId, invite_id: InviteId, device_seed: u8, addresses: Vec<String>) -> WireMessage {
    WireMessage::AdmissionRequest {
        org_id,
        invite_id,
        name: Name::parse("Joe").unwrap(),
        surname: Surname::parse("Joiner").unwrap(),
        preferred_handles: vec![Handle::parse("joe").unwrap()],
        member_key: member_key(device_seed),
        device_key: device_keypair([device_seed; 32]).device_key().unwrap(),
        addresses,
    }
}

// ---- check_received ------------------------------------------------------------

// Normal: the check takes no chain state — its signature names none — and
// refuses, with nothing changed, a body over the bound, before it decodes
// it, and a body that does not decode; a store no Persona has is refused.
// verifies: LLR-94rzrk
#[test]
fn check_received_takes_no_chain_state_and_refuses_an_oversized_or_undecodable_body() {
    let signatures = public_signatures("service.rs");
    let signature = signatures.iter().find(|signature| signature.starts_with("pub fn check_received<")).unwrap();
    assert!(!signature.contains("OrgState") && !signature.contains("chain"), "{signature}");
    let mut story = admit_b(setup("rp-no-chain"));
    let b_persona_id = story.b_persona_id.clone();
    let before = (encoded_stores(&story.node_b), store_bytes("rp-no-chain", "b"));
    let oversized = vec![0u8; MAX_MESSAGE + 1];
    assert_eq!(
        check(&mut story.node_b, &b_persona_id, rogue(), &oversized).err(),
        Some(OrgNodeError::MessageTooLarge { length: MAX_MESSAGE + 1 })
    );
    for body in [&[][..], &[0xff, 0x00, 0x01][..], &[0x07; 64][..]] {
        assert_eq!(check(&mut story.node_b, &b_persona_id, rogue(), body).err(), Some(OrgNodeError::MalformedMessage));
    }
    let unknown = story.node_b.svc.check_received(&mut OsRng, &device_key(0x7e), rogue(), &[0x07; 64]);
    assert_eq!(unknown.err(), Some(OrgNodeError::PersonaNotHeld { device_key: Box::new(device_key(0x7e)) }));
    assert_eq!((encoded_stores(&story.node_b), store_bytes("rp-no-chain", "b")), before, "nothing changed");
}

// verifies: LLR-94rzrk
#[test]
fn a_body_that_does_not_decode_is_malformed_and_changes_nothing() {
    let mut story = setup("rp-malformed");
    let b_persona_id = story.b_persona_id.clone();
    let before = (encoded_stores(&story.node_b), store_bytes("rp-malformed", "b"));
    for body in [&[][..], &[0xff, 0x00, 0x01][..], &[0x07; 64][..]] {
        assert!(matches!(check(&mut story.node_b, &b_persona_id, rogue(), body), Err(OrgNodeError::MalformedMessage)));
    }
    assert_eq!((encoded_stores(&story.node_b), store_bytes("rp-malformed", "b")), before);
    assert!(story.node_b.expected_admissions().len() == 1, "the expectation stays");
}

// Abnormal: in a store with a current record, Organisation information, a
// revocation notice and an organisation answer from a Device the record does
// not list, and an organisation request from one it neither lists nor holds
// as revoked, are refused with `SenderNotListed` before their Change set,
// snapshot, notice or addresses are used — even when those would not parse —
// keeping nothing.
// verifies: LLR-283x6f
#[test]
fn a_body_from_a_device_the_record_does_not_list_is_refused_before_use() {
    let mut story = admit_b(setup("rp-not-listed"));
    let b_persona_id = story.b_persona_id.clone();
    let org_id = story.org_id;
    let unusable = [
        carrying(Envelope { org_id, parent_seq: SequenceNumber::new(3), delta_bytes: vec![0xff] }),
        WireMessage::Revocation(bob_absence_notice(org_id)),
        WireMessage::Organisation {
            org_id,
            epoch: Epoch::new(9),
            record_snapshot: vec![0xff],
            org_private_key: OrgPrivateKey::from([0x77; 32]),
        },
        WireMessage::OrganisationRequest { org_id, addresses: vec![String::new(); 17] },
    ];
    let before = (encoded_stores(&story.node_b), store_bytes("rp-not-listed", "b"));
    for message in unusable {
        let refused = check(&mut story.node_b, &b_persona_id, rogue(), &encode_message(&message).unwrap()).map(|_| ());
        assert_eq!(refused, Err(OrgNodeError::SenderNotListed { org_id }), "{message:?}");
    }
    assert_eq!((encoded_stores(&story.node_b), store_bytes("rp-not-listed", "b")), before, "nothing kept");
}

// Normal: from a Device the current record does not list, three kinds are
// acted on — an acknowledgement (held to its own rule), an admission request
// (held to its invite), and an organisation request from a Device held as
// revoked; every other kind, and an organisation request from a Device never
// listed, is `SenderNotListed`. A holds B's Device as revoked through B's
// admission and removal (`support::admit_b`, `remove_b`), and the revoked
// Device's request is answered with its notice, not merely not refused.
// verifies: LLR-w7b9kd
#[test]
fn from_an_unlisted_device_only_three_kinds_are_acted_on() {
    let (mut story, b_id) = remove_b(admit_b(setup("rp-three-kinds")));
    let a_persona_id = story.a_persona_id.clone();
    let org_id = story.org_id;
    // A holds B's Device as revoked.
    let revoked = story.b_device_keypair.device_key().unwrap();
    assert_eq!(
        story.node_a.store_of(&a_persona_id).revoked_devices,
        vec![RevokedDevice { member_id: b_id, device_key: revoked }],
        "fixture: the removal's adoption holds B's Device as revoked"
    );
    let stranger = device_key(0x6b);
    let not_listed = Err(OrgNodeError::SenderNotListed { org_id });

    let ack = WireMessage::Acknowledgement(org_node::revocation::Acknowledgement {
        org_id,
        member_id: MemberId::new([0x6b; 32]),
        device: stranger,
        epoch: Epoch::new(1),
        root: org_members::RootHash::new([0x11; 32]),
        signature: org_node::revocation::Signature64([0; 64]),
    });
    let acted = check(&mut story.node_a, &a_persona_id, stranger, &encode_message(&ack).unwrap()).map(|_| ());
    assert_eq!(acted, Err(OrgNodeError::AcknowledgementSignatureInvalid { org_id }), "held to its own rule");
    let request = admission_request(org_id, InviteId::new([0x61; 32]), 0x6b, vec![]);
    let acted = check(&mut story.node_a, &a_persona_id, stranger, &encode_message(&request).unwrap()).map(|_| ());
    assert_eq!(
        acted,
        Err(OrgNodeError::InviteNotOutstanding { org_id, invite_id: InviteId::new([0x61; 32]) }),
        "held to its invite"
    );
    let organisation_request = encode_message(&WireMessage::OrganisationRequest { org_id, addresses: vec![] }).unwrap();
    match check(&mut story.node_a, &a_persona_id, revoked, &organisation_request) {
        Ok((Checked::OrganisationRequest { answer }, _)) => {
            assert_eq!(answer.recipient, revoked, "answered to the revoked Device");
            assert!(
                matches!(org_node::wire::decode_message(&answer.body), Ok(WireMessage::Revocation(notice)) if notice.member_id == b_id),
                "with the notice of its removal"
            );
        }
        other => panic!("an organisation request from a Device held as revoked is acted on: {other:?}"),
    }
    assert_eq!(check(&mut story.node_a, &a_persona_id, stranger, &organisation_request).map(|_| ()), not_listed);
    for message in [
        carrying(Envelope { org_id, parent_seq: SequenceNumber::new(2), delta_bytes: vec![0x01] }),
        WireMessage::Revocation(bob_absence_notice(org_id)),
        WireMessage::Organisation { org_id, epoch: Epoch::new(2), record_snapshot: vec![], org_private_key: OrgPrivateKey::from([0x77; 32]) },
    ] {
        for sender in [stranger, revoked] {
            assert_eq!(check(&mut story.node_a, &a_persona_id, sender, &encode_message(&message).unwrap()).map(|_| ()), not_listed);
        }
    }
}

// Abnormal: an admission request whose (Organisation, invite identifier)
// pair this store does not hold as outstanding is refused with
// `InviteNotOutstanding` before any later field is decoded — a body that
// would not decode past the identifiers is refused so — and nothing is kept.
// verifies: LLR-6gkxpg
#[test]
fn an_admission_request_for_an_invite_not_outstanding_is_refused_before_its_fields_decode() {
    let mut story = setup("rp-not-outstanding");
    let a_persona_id = story.a_persona_id.clone();
    let (org_id, invite_id) = invited(&mut story);
    let truncated = |org: OrgId, invite: InviteId| {
        let mut body = vec![3u8];
        body.extend(org.as_bytes());
        body.extend(invite.as_bytes());
        body.extend([0xff; 5]);
        body
    };
    let before = encoded_stores(&story.node_a);
    for (org, invite) in [(org_id, InviteId::new([0x77; 32])), (OrgId::new([0x42; 20]), invite_id)] {
        let refused = check(&mut story.node_a, &a_persona_id, device_key(0x6b), &truncated(org, invite)).map(|_| ());
        assert_eq!(refused, Err(OrgNodeError::InviteNotOutstanding { org_id: org, invite_id: invite }));
    }
    let refused = check(&mut story.node_a, &a_persona_id, device_key(0x6b), &truncated(org_id, invite_id)).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::MalformedMessage), "an outstanding pair is decoded, and this body does not decode");
    assert_eq!(encoded_stores(&story.node_a), before, "nothing kept");
}

// Abnormal: an admission request whose DevicePublicKey is not the Device
// that sent it is refused with `AdmissionKeyNotSender`, nothing kept.
// verifies: LLR-6gkxpg
#[test]
fn an_admission_request_whose_key_is_not_the_sender_is_refused() {
    let mut story = setup("rp-key-not-sender");
    let a_persona_id = story.a_persona_id.clone();
    let (org_id, invite_id) = invited(&mut story);
    let body = encode_message(&admission_request(org_id, invite_id, 0x6b, vec![])).unwrap();
    let before = encoded_stores(&story.node_a);
    assert_eq!(
        check(&mut story.node_a, &a_persona_id, device_key(0x6c), &body).map(|_| ()),
        Err(OrgNodeError::AdmissionKeyNotSender { org_id })
    );
    assert_eq!(encoded_stores(&story.node_a), before, "nothing kept");
    assert!(story.node_a.store_of(&a_persona_id).kept_admission_requests.is_empty());
}

// Abnormal: a received admission request naming no preferred Handle, four,
// or one twice is refused naming `request.preferred_handles`, keeping
// nothing.
// verifies: LLR-6gkxpg
#[test]
fn an_admission_request_with_no_four_or_a_repeated_handle_is_refused() {
    let mut story = setup("rp-request-handles");
    let a_persona_id = story.a_persona_id.clone();
    let (org_id, invite_id) = invited(&mut story);
    let before = encoded_stores(&story.node_a);
    for handles in [vec![], vec!["a1", "b1", "c1", "d1"], vec!["joe", "joe"]] {
        let mut message = admission_request(org_id, invite_id, 0x6b, vec![]);
        let WireMessage::AdmissionRequest { preferred_handles, .. } = &mut message else {
            panic!("fixture: an admission request")
        };
        *preferred_handles = handles.iter().map(|text| handle(text)).collect();
        let refused = check(&mut story.node_a, &a_persona_id, device_key(0x6b), &encode_message(&message).unwrap()).map(|_| ());
        assert!(
            matches!(refused, Err(OrgNodeError::InvalidField { field: "request.preferred_handles", .. })),
            "{handles:?}: {refused:?}"
        );
    }
    assert_eq!(encoded_stores(&story.node_a), before, "nothing kept");
}

// Normal: the first admission request for an outstanding invite is kept,
// from its sender, with one store change; abnormal: a repeat from the same
// Device is `AlreadyReceived` and one from another Device
// `InviteUsedByAnotherDevice`, neither changing anything.
// verifies: LLR-6gkxpg, LLR-v3av9t
#[test]
fn the_first_admission_request_is_kept_and_a_repeat_or_another_device_is_not() {
    let tag = "rp-first-request";
    let mut story = setup(tag);
    let a_persona_id = story.a_persona_id.clone();
    let (org_id, invite_id) = invited(&mut story);
    let first = encode_message(&admission_request(org_id, invite_id, 0x6b, vec![])).unwrap();
    let (checked, change) = check(&mut story.node_a, &a_persona_id, device_key(0x6b), &first).unwrap();
    assert!(matches!(checked, Checked::AdmissionRequest { outcome: AdmissionRequestOutcome::Accepted }), "{checked:?}");
    story.node_a.disk.apply(change.expect("the kept request is sealed"));
    let kept = &story.node_a.store_of(&a_persona_id).kept_admission_requests;
    assert_eq!(kept.len(), 1);
    assert_eq!((kept[0].invite_id, kept[0].from, kept[0].device_key), (invite_id, device_key(0x6b), device_key(0x6b)));

    let before = (encoded_stores(&story.node_a), store_bytes(tag, "a"));
    let (checked, change) = check(&mut story.node_a, &a_persona_id, device_key(0x6b), &first).unwrap();
    assert!(matches!(checked, Checked::AdmissionRequest { outcome: AdmissionRequestOutcome::AlreadyReceived }), "{checked:?}");
    assert!(change.is_none());
    let other = encode_message(&admission_request(org_id, invite_id, 0x6c, vec![])).unwrap();
    let (checked, change) = check(&mut story.node_a, &a_persona_id, device_key(0x6c), &other).unwrap();
    assert!(
        matches!(checked, Checked::AdmissionRequest { outcome: AdmissionRequestOutcome::InviteUsedByAnotherDevice }),
        "{checked:?}"
    );
    assert!(change.is_none());
    assert_eq!((encoded_stores(&story.node_a), store_bytes(tag, "a")), before, "neither changed anything");
}

// Normal: the kept request holds the request's values, each parsed, and the
// addresses its sender gave, and reaches the sealed store; abnormal: a
// request whose addresses the parse refuses is refused naming
// `request.addresses`, keeping nothing.
// verifies: LLR-v3av9t
#[test]
fn a_kept_admission_request_holds_its_senders_addresses() {
    let tag = "rp-request-addresses";
    let mut story = setup(tag);
    let a_persona_id = story.a_persona_id.clone();
    let (org_id, invite_id) = invited(&mut story);
    let refused = encode_message(&admission_request(org_id, invite_id, 0x6b, vec!["a".repeat(257)])).unwrap();
    assert!(matches!(
        check(&mut story.node_a, &a_persona_id, device_key(0x6b), &refused),
        Err(OrgNodeError::InvalidField { field: "request.addresses", .. })
    ));
    assert!(story.node_a.store_of(&a_persona_id).kept_admission_requests.is_empty());
    let addresses = vec!["198.51.100.7:7000".to_string(), "[2001:db8::7]:7000".to_string()];
    let body = encode_message(&admission_request(org_id, invite_id, 0x6b, addresses.clone())).unwrap();
    let (_, change) = check(&mut story.node_a, &a_persona_id, device_key(0x6b), &body).unwrap();
    story.node_a.disk.apply(change.unwrap());
    let reopened = Node::reopened(party(tag, "a")).svc;
    let a_device = story.node_a.device_of(&a_persona_id);
    let kept = &reopened.store_data(&a_device).unwrap().kept_admission_requests[0];
    assert_eq!(kept.addresses.as_slice(), addresses.as_slice());
    assert_eq!((kept.org_id, kept.invite_id, kept.from), (org_id, invite_id, device_key(0x6b)));
    assert_eq!((kept.name.as_str(), kept.surname.as_str()), ("Joe", "Joiner"));
    assert_eq!(kept.preferred_handles, vec![Handle::parse("joe").unwrap()]);
    assert_eq!((kept.member_key, kept.device_key), (member_key(0x6b), device_key(0x6b)));
}

// Abnormal: Organisation information for the held Organisation from an
// unlisted Device is refused in the check, nothing changed.
// verifies: LLR-283x6f
#[test]
fn an_update_for_a_held_organisation_from_an_unlisted_sender_is_refused_in_the_check() {
    let mut story = admit_b(setup("rp-update-unlisted"));
    let b_persona_id = story.b_persona_id.clone();
    let update = carrying(Envelope { org_id: story.org_id, parent_seq: SequenceNumber::new(3), delta_bytes: vec![0x01] });
    let body = encode_message(&update).unwrap();
    let before = encoded_stores(&story.node_b);
    let refused = check(&mut story.node_b, &b_persona_id, rogue(), &body).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::SenderNotListed { org_id: story.org_id }));
    assert_eq!(encoded_stores(&story.node_b), before, "nothing changed");
}

// Abnormal: a revocation notice from an unlisted Device is refused by the
// sender rule, and the refusal keeps nothing, in memory or on the disk.
// verifies: LLR-283x6f, LLR-r7zm39
#[test]
fn a_revocation_from_an_unlisted_sender_is_refused_in_the_check() {
    let tag = "rp-notice-unlisted";
    let (mut story, notice) = removal(tag);
    let b_persona_id = story.b_persona_id.clone();
    let body = encode_message(&WireMessage::Revocation(notice)).unwrap();
    let before = (encoded_stores(&story.node_b), store_bytes(tag, "b"));
    let refused = check(&mut story.node_b, &b_persona_id, rogue(), &body).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::SenderNotListed { org_id: story.org_id }));
    assert_eq!((encoded_stores(&story.node_b), store_bytes(tag, "b")), before, "nothing kept");
    assert!(story.node_b.store_of(&b_persona_id).kept_revocation_notices.is_empty());
}

// B's acknowledgement of its removal: decided in the check, from the Device
// it names only, and only when the record no longer lists that Device; A's
// adopted removal holds B's Device as revoked, so it is kept. B's
// acknowledgement is signed with `revocation::sign_acknowledgement` over the
// state of A's removal.
// verifies: LLR-xudkh4, LLR-5azhry
#[test]
fn an_acknowledgement_is_decided_in_the_check_from_its_named_device_only() {
    let tag = "rp-ack";
    let story = admit_b(setup(tag));
    let members_listing_b = record_of(&story.node_a, story.org_id).trie_members;
    let (mut story, _) = remove_b(story);
    let a_persona_id = story.a_persona_id.clone();
    let b_device = story.b_device_keypair.device_key().unwrap();
    let state = story.chain.get(&story.org_id).unwrap();
    let b_store = story.node_b.store_of(&story.b_persona_id).clone();
    let secret = story.node_b.keychain.secret(&b_device).unwrap();
    let acknowledgement = sign_acknowledgement(&b_store.persona, story.org_id, &state, &secret).unwrap();
    let ack_body = encode_message(&WireMessage::Acknowledgement(acknowledgement.clone())).unwrap();

    match check(&mut story.node_a, &a_persona_id, b_device, &ack_body) {
        Ok((Checked::Acknowledged(verified), Some(_))) => assert_eq!(verified.into_inner(), acknowledgement),
        other => panic!("not decided and kept as an acknowledgement: {other:?}"),
    }
    let kept = &story.node_a.store_of(&a_persona_id).acknowledged;
    assert_eq!((kept.len(), &kept[0].acknowledgement), (1, &acknowledgement), "kept once");

    let before = encoded_stores(&story.node_a);
    let relayed = check(&mut story.node_a, &a_persona_id, rogue(), &ack_body).map(|_| ());
    assert_eq!(relayed, Err(OrgNodeError::AcknowledgementNotFromItsDevice { org_id: story.org_id }));
    assert_eq!(encoded_stores(&story.node_a), before, "a refusal changes nothing");

    // A record that still lists B's Device (a store this software never
    // writes, sealed back as org-io would hold it).
    tamper(tag, "a", &a_persona_id, |store| {
        store.record.as_mut().expect("A's store holds a record").trie_members = members_listing_b;
    });
    let mut listing = Node::reopened(party(tag, "a"));
    let listed = check(&mut listing, &a_persona_id, b_device, &ack_body).map(|_| ());
    assert_eq!(listed, Err(OrgNodeError::AcknowledgementForListedDevice { org_id: story.org_id }));
}

// A held Organisation's messages from an unlisted Device are refused before
// use, Organisation information and a revocation notice alike; an
// acknowledgement is held to its own rule — `check_acknowledgement` with the
// sender, no listing — so B's genuine one from B's own, no longer listed,
// Device is decided (and kept, B's Device being held as revoked).
// verifies: LLR-283x6f, LLR-xudkh4
#[test]
fn a_held_organisations_messages_from_an_unlisted_device_are_refused_before_use() {
    let tag = "rp-held-unlisted";
    let (mut story, notice) = removal(tag);
    let a_persona_id = story.a_persona_id.clone();
    let not_listed = Err(OrgNodeError::SenderNotListed { org_id: story.org_id });
    let update = carrying(Envelope { org_id: story.org_id, parent_seq: SequenceNumber::new(4), delta_bytes: vec![0xff] });
    let before = encoded_stores(&story.node_a);
    for message in [update, WireMessage::Revocation(notice)] {
        assert_eq!(check(&mut story.node_a, &a_persona_id, rogue(), &encode_message(&message).unwrap()).map(|_| ()), not_listed);
    }
    assert_eq!(encoded_stores(&story.node_a), before, "nothing kept");
    let b_device = story.b_device_keypair.device_key().unwrap();
    let state = story.chain.get(&story.org_id).unwrap();
    let secret = story.node_b.keychain.secret(&b_device).unwrap();
    let acknowledgement = sign_acknowledgement(&story.node_b.store_of(&story.b_persona_id).persona, story.org_id, &state, &secret).unwrap();
    let body = encode_message(&WireMessage::Acknowledgement(acknowledgement)).unwrap();
    assert!(matches!(check(&mut story.node_a, &a_persona_id, b_device, &body), Ok((Checked::Acknowledged(_), Some(_)))));
}

// The `RevocationNotHeld` refusal is reached through `check_received`, so
// it is no unreachable branch. A store holds one Organisation,
// so the notice is about another Organisation than the one A's store
// holds, from A's own listed Device. B's store, a provisional Member's,
// holds no record and refuses the same notice with `OrgNotHeld`. Neither
// keeps anything.
// verifies: LLR-95av3e
#[test]
fn an_unheld_organisations_notice_is_refused_as_not_held() {
    let mut story = setup("rp-no-unreachable");
    let (a_persona_id, b_persona_id) = (story.a_persona_id.clone(), story.b_persona_id.clone());
    let elsewhere = OrgId::new([0x42; 20]);
    let body = encode_message(&WireMessage::Revocation(bob_absence_notice(elsewhere))).unwrap();
    let a_device = story.node_a.device_of(&a_persona_id);
    let before = (encoded_stores(&story.node_a), encoded_stores(&story.node_b));
    let refused = check(&mut story.node_a, &a_persona_id, a_device, &body).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::RevocationNotHeld { org_id: elsewhere }), "check_received reaches the refusal");
    let refused = check(&mut story.node_b, &b_persona_id, a_device, &body).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::OrgNotHeld { org_id: elsewhere }), "a store with no record");
    assert_eq!((encoded_stores(&story.node_a), encoded_stores(&story.node_b)), before, "nothing kept");
}

/// B admitted (epoch 2) and A's Device: the setup, B's Persona, A's Device
/// and Organisation information for epoch 3 from B's record, with its root.
fn information_for_b(tag: &str) -> (Setup, PersonaId, DevicePublicKey, WireMessage, org_members::RootHash) {
    let story = admit_b(setup(tag));
    let b_persona_id = story.b_persona_id.clone();
    let a_device = story.node_a.device_of(&story.a_persona_id);
    let (message, root) = information_after(&record_of(&story.node_b, story.org_id), 0x70);
    (story, b_persona_id, a_device, message, root)
}

// Normal: Organisation information from a Device the current record lists
// passes the chain-free checks and is kept as a received provisional
// organisation for the next epoch: the record it extends and the next one,
// the Change set and snapshot it carried, the carried private key and the
// public key derived from it, its sender; the current record is unchanged,
// and no chain state is used. Its Sequence number 3 is past the mark 2, and
// all four chain-free checks pass.
// verifies: LLR-76bhw9, LLR-s3vq6d, REQ-f2k4tr, LLR-fuq379, LLR-wx3php
#[test]
fn organisation_information_from_a_listed_device_is_kept_as_received_with_its_derived_key() {
    let tag = "rp-kept";
    let (mut story, b_persona_id, a_device, message, root) = information_for_b(tag);
    let record = record_of(&story.node_b, story.org_id);
    let WireMessage::OrgInformation { envelope, record_snapshot, org_private_key } = message.clone() else {
        panic!("fixture: information")
    };
    let (checked, change) = check(&mut story.node_b, &b_persona_id, a_device, &encode_message(&message).unwrap()).unwrap();
    assert!(matches!(checked, Checked::ProvisionalOrganisationKept { epoch } if epoch == Epoch::new(3)), "{checked:?}");
    story.node_b.disk.apply(change.expect("the keep is sealed"));
    let store_data = story.node_b.store_of(&b_persona_id);
    assert_eq!(store_data.provisional_organisations.len(), 1);
    let kept = &store_data.provisional_organisations[0];
    assert_eq!((kept.epoch, kept.base_root, kept.root), (Epoch::new(3), record.root_hash, root));
    assert_eq!(kept.members.len(), 3, "the next record: A, B and the joiner");
    assert_eq!(kept.org_private_key_for_test(), &org_private_key);
    assert_eq!(kept.org_pub_key, org_private_key.x25519_keypair().org_public_key().unwrap(), "derived, not taken");
    assert_eq!(kept.change_set, Some(envelope.delta_bytes));
    assert_eq!(kept.base_snapshot, Some(record_snapshot));
    assert_eq!(kept.origin, org_node::store::Origin::Received { from: a_device });
    assert!(kept.applied.is_empty());
    let now = record_of(&story.node_b, story.org_id);
    assert_eq!((now.epoch, now.root_hash, now.trie_members), (record.epoch, record.root_hash, record.trie_members));
    let reopened = Node::reopened(party(tag, "b")).svc;
    let b_device = story.node_b.device_of(&b_persona_id);
    assert_eq!(reopened.store_data(&b_device).unwrap().provisional_organisations.len(), 1, "on the disk");
}

// Abnormal: each chain-free check refuses with no chain state — another
// Organisation (`OrgIdMismatch`), a Sequence number at or below the mark
// (`StaleSeq`), a Change set that does not decode (`MalformedDelta`) and one
// built on another base (`DeltaBaseMismatch`) — nothing kept, no chain read.
// verifies: LLR-76bhw9, REQ-f2k4tr, LLR-fuq379
#[test]
fn the_chain_free_checks_refuse_with_no_chain_state() {
    let (mut story, b_persona_id, a_device, message, _) = information_for_b("rp-chain-free");
    let WireMessage::OrgInformation { envelope, .. } = message.clone() else { panic!("fixture: information") };
    let elsewhere = admit_b(setup("rp-chain-free-elsewhere"));
    let (foreign, _) = information_after(&record_of(&elsewhere.node_a, elsewhere.org_id), 0x71);
    let foreign_delta = envelope_of(&foreign).delta_bytes.clone();
    let cases = [
        (Envelope { org_id: OrgId::new([0x42; 20]), ..envelope.clone() }, OrgNodeError::OrgIdMismatch),
        (Envelope { parent_seq: SequenceNumber::new(2), ..envelope.clone() }, OrgNodeError::StaleSeq { got: 2, last_seen: 2 }),
        (Envelope { delta_bytes: vec![0xff, 0xff], ..envelope.clone() }, OrgNodeError::MalformedDelta),
        (Envelope { delta_bytes: foreign_delta, ..envelope.clone() }, OrgNodeError::DeltaBaseMismatch),
    ];
    let before = encoded_stores(&story.node_b);
    for (bad, expected) in cases {
        let body = encode_message(&with_envelope(&message, bad)).unwrap();
        let expected_text = format!("{expected:?}");
        assert_eq!(check(&mut story.node_b, &b_persona_id, a_device, &body).map(|_| ()), Err(expected), "{expected_text}");
    }
    assert_eq!(encoded_stores(&story.node_b), before, "nothing kept");
}

// Abnormal: the Organisation and the Sequence number are judged before the
// Change set is decoded. Change set bytes that do not decode are refused as
// `OrgIdMismatch` when the Envelope names another Organisation, and as
// `StaleSeq` — naming the offered number, then the mark, the record's
// Sequence number 2 — when its number is at or below the mark; only past the
// mark, for this Organisation, are they `MalformedDelta`. Nothing kept, no
// chain state.
// verifies: REQ-gju89b, REQ-6yu72z, REQ-f2k4tr, LLR-4fbuy8, LLR-ybn5pr, LLR-xpbkp5, LLR-wx3php, LLR-fuq379
#[test]
fn the_organisation_and_the_sequence_number_are_judged_before_the_change_set_is_decoded() {
    let (mut story, b_persona_id, a_device, message, _) = information_for_b("rp-before-decode");
    let undecodable = Envelope { delta_bytes: vec![0xff, 0xff], ..envelope_of(&message).clone() };
    let cases = [
        (Envelope { org_id: OrgId::new([0x42; 20]), ..undecodable.clone() }, OrgNodeError::OrgIdMismatch),
        (Envelope { parent_seq: SequenceNumber::new(2), ..undecodable.clone() }, OrgNodeError::StaleSeq { got: 2, last_seen: 2 }),
        (Envelope { parent_seq: SequenceNumber::new(1), ..undecodable.clone() }, OrgNodeError::StaleSeq { got: 1, last_seen: 2 }),
        (undecodable, OrgNodeError::MalformedDelta),
    ];
    let before = encoded_stores(&story.node_b);
    for (bad, expected) in cases {
        let body = encode_message(&with_envelope(&message, bad)).unwrap();
        let expected_text = format!("{expected:?}");
        assert_eq!(check(&mut story.node_b, &b_persona_id, a_device, &body).map(|_| ()), Err(expected), "{expected_text}");
    }
    assert_eq!(encoded_stores(&story.node_b), before, "nothing kept");
}

// Abnormal: a Sequence number that is not the record's epoch plus one — one
// further on — is `SeqNotEpoch`, nothing kept.
// verifies: LLR-76bhw9
#[test]
fn a_sequence_number_other_than_epoch_plus_one_is_seq_not_epoch() {
    let (mut story, b_persona_id, a_device, message, _) = information_for_b("rp-seq-not-epoch");
    let ahead = Envelope { parent_seq: SequenceNumber::new(4), ..envelope_of(&message).clone() };
    let before = encoded_stores(&story.node_b);
    let body = encode_message(&with_envelope(&message, ahead)).unwrap();
    assert_eq!(
        check(&mut story.node_b, &b_persona_id, a_device, &body).map(|_| ()),
        Err(OrgNodeError::SeqNotEpoch { seq: 4, epoch: 3 })
    );
    assert_eq!(encoded_stores(&story.node_b), before);
}

// Normal: the same identity again is `AlreadyHeld`, keeping no second copy
// and returning no change; another identity for the same epoch — the same
// Change set under another key — is kept beside it.
// verifies: LLR-s3vq6d
#[test]
fn an_identity_already_held_is_already_held_and_keeps_no_copy() {
    let (mut story, b_persona_id, a_device, message, _) = information_for_b("rp-already-held");
    let body = encode_message(&message).unwrap();
    let (_, change) = check(&mut story.node_b, &b_persona_id, a_device, &body).unwrap();
    story.node_b.disk.apply(change.unwrap());
    let before = encoded_stores(&story.node_b);
    let (checked, change) = check(&mut story.node_b, &b_persona_id, a_device, &body).unwrap();
    assert!(matches!(checked, Checked::AlreadyHeld), "{checked:?}");
    assert!(change.is_none());
    assert_eq!(encoded_stores(&story.node_b), before, "no second copy");
    let other_key = with_key(&message, OrgPrivateKey::from([0x3c; 32]));
    let (checked, change) = check(&mut story.node_b, &b_persona_id, a_device, &encode_message(&other_key).unwrap()).unwrap();
    crate::support::unread(change);
    assert!(matches!(checked, Checked::ProvisionalOrganisationKept { .. }), "{checked:?}");
    assert_eq!(story.node_b.store_of(&b_persona_id).provisional_organisations.len(), 2, "two identities for one epoch");
}

// Normal and abnormal: in a store with no current record, a body is decoded
// no further than its kind and Organisation identifier unless it is
// Organisation information or an organisation answer an expected admission
// names — a revocation, an acknowledgement, an admission request and an
// organisation request are `OrgNotHeld` even when nothing after the
// identifier decodes, and so is an answer for another Organisation;
// Organisation information for another Organisation is `AdmissionNotExpected`
// so too; an expected answer is decoded further (garbage is
// `MalformedMessage`); the expected information is decoded and kept.
// verifies: LLR-e6w3nr
#[test]
fn in_a_store_with_no_record_only_expected_organisation_information_is_decoded() {
    let mut story = setup("rp-no-record");
    let b_persona_id = story.b_persona_id.clone();
    let a_device = story.node_a.device_of(&story.a_persona_id);
    let garbage_after = |kind: u8, org_id: OrgId| {
        let mut body = vec![kind];
        body.extend(org_id.as_bytes());
        body.extend([0xff; 9]);
        body
    };
    let before = encoded_stores(&story.node_b);
    for kind in 1u8..=4 {
        assert_eq!(
            check(&mut story.node_b, &b_persona_id, a_device, &garbage_after(kind, story.org_id)).map(|_| ()),
            Err(OrgNodeError::OrgNotHeld { org_id: story.org_id }),
            "kind {kind}"
        );
    }
    let elsewhere = OrgId::new([0x42; 20]);
    assert_eq!(
        check(&mut story.node_b, &b_persona_id, a_device, &garbage_after(5, elsewhere)).map(|_| ()),
        Err(OrgNodeError::OrgNotHeld { org_id: elsewhere }),
        "an answer for another Organisation"
    );
    assert_eq!(
        check(&mut story.node_b, &b_persona_id, a_device, &garbage_after(5, story.org_id)).map(|_| ()),
        Err(OrgNodeError::MalformedMessage),
        "an expected answer is decoded further"
    );
    assert_eq!(
        check(&mut story.node_b, &b_persona_id, a_device, &garbage_after(0, elsewhere)).map(|_| ()),
        Err(OrgNodeError::AdmissionNotExpected { org_id: elsewhere })
    );
    assert_eq!(encoded_stores(&story.node_b), before, "nothing kept");
    let (message, root) = information_after(&record_of(&story.node_a, story.org_id), 0x72);
    let (checked, change) = check(&mut story.node_b, &b_persona_id, a_device, &encode_message(&message).unwrap()).unwrap();
    assert!(matches!(checked, Checked::ProvisionalOrganisationKept { epoch } if epoch == Epoch::new(2)), "{checked:?}");
    assert!(change.is_some());
    assert_eq!(story.node_b.store_of(&b_persona_id).provisional_organisations[0].root, root);
}

// Abnormal: in a store with no record and no expected admission, Organisation
// information is refused with `AdmissionNotExpected`, nothing kept.
// verifies: LLR-e6w3nr
#[test]
fn unexpected_organisation_information_in_a_store_with_no_record_is_refused() {
    let mut story = setup_with("rp-unexpected", false);
    let b_persona_id = story.b_persona_id.clone();
    let a_device = story.node_a.device_of(&story.a_persona_id);
    let (message, _) = information_after(&record_of(&story.node_a, story.org_id), 0x73);
    let before = encoded_stores(&story.node_b);
    assert_eq!(
        check(&mut story.node_b, &b_persona_id, a_device, &encode_message(&message).unwrap()).map(|_| ()),
        Err(OrgNodeError::AdmissionNotExpected { org_id: story.org_id })
    );
    assert_eq!(encoded_stores(&story.node_b), before);
}

// Abnormal: expected Organisation information in a store with no record
// whose snapshot does not decode, or whose Change set does not apply to the
// snapshot it carries, is refused as `first_admission_base` and
// `check_chain_free` refuse it, nothing kept.
// verifies: LLR-e6w3nr
#[test]
fn expected_organisation_information_failing_its_snapshot_or_chain_free_checks_is_refused() {
    let mut story = setup("rp-first-refused");
    let b_persona_id = story.b_persona_id.clone();
    let a_device = story.node_a.device_of(&story.a_persona_id);
    let record = record_of(&story.node_a, story.org_id);
    let (message, _) = information_after(&record, 0x75);
    let WireMessage::OrgInformation { envelope, org_private_key, .. } = &message else { panic!("fixture: information") };
    let undecodable =
        WireMessage::OrgInformation { envelope: envelope.clone(), record_snapshot: vec![0xff; 3], org_private_key: org_private_key.clone() };
    let other_base = WireMessage::OrgInformation {
        envelope: envelope.clone(),
        record_snapshot: postcard::to_allocvec(&vec![support::fixtures::eve()]).unwrap(),
        org_private_key: org_private_key.clone(),
    };
    let before = encoded_stores(&story.node_b);
    let refused = check(&mut story.node_b, &b_persona_id, a_device, &encode_message(&undecodable).unwrap()).map(|_| ());
    assert!(
        matches!(refused, Err(OrgNodeError::InvalidField { field: "record_snapshot", .. })),
        "a snapshot that does not decode: {refused:?}"
    );
    let refused = check(&mut story.node_b, &b_persona_id, a_device, &encode_message(&other_base).unwrap()).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::DeltaBaseMismatch), "a Change set from another base");
    assert_eq!(encoded_stores(&story.node_b), before, "nothing kept");
}

// Normal: expected Organisation information is kept from whichever Device sent it —
// here a Device no record lists — with that sender as its origin, and the
// expectation is left in place.
// verifies: LLR-e6w3nr
#[test]
fn expected_organisation_information_is_kept_from_any_sender() {
    let mut story = setup("rp-first-any");
    let b_persona_id = story.b_persona_id.clone();
    let (message, _) = information_after(&record_of(&story.node_a, story.org_id), 0x74);
    let (checked, change) = check(&mut story.node_b, &b_persona_id, rogue(), &encode_message(&message).unwrap()).unwrap();
    crate::support::unread(change);
    assert!(matches!(checked, Checked::ProvisionalOrganisationKept { .. }), "{checked:?}");
    let store_data = story.node_b.store_of(&b_persona_id);
    assert_eq!(store_data.provisional_organisations[0].origin, org_node::store::Origin::Received { from: rogue() });
    assert_eq!(store_data.expected_admissions.len(), 1, "the expectation stays");
    assert!(store_data.record.is_none(), "nothing adopted on receipt");
}

/// Fixtures of the notice and acknowledgement arms: stores holding a
/// record of one Organisation, built from public values, so that no other
/// operation runs before the one under test.
mod notices {
    use org_node::revocation::{Acknowledgement, RevocationNotice, Signature64};
    use org_node::store::StoreData;
    use org_node::test_fixtures::{device_key, device_keypair, empty_store};
    use org_node::Epoch;

    pub use crate::support::fixtures::*;
    use crate::support::{the_org, trie_of_members};

    /// Bob's store: a record at epoch 3 listing Ada and Bob.
    pub fn bob_store() -> StoreData {
        let mut store_data = empty_store(persona(&bob(), 0x62));
        store_data.record = Some(record(3, &[ada(), bob()]));
        store_data
    }

    /// A notice that Bob's Member and Device are absent from a record
    /// listing Ada alone.
    pub fn notice_for_bob() -> RevocationNotice {
        let bob = bob();
        let proof = trie_of_members(&[ada()]).prove_absent(&bob.id, &bob.device_keys[0]).unwrap();
        RevocationNotice { org_id: the_org(), member_id: bob.id, device: bob.device_keys[0], proof }
    }

    /// Dave's acknowledgement of its removal at `epoch`, signed by Dave's Device.
    pub fn ack_by_dave(epoch: u64) -> Acknowledgement {
        let mut ack = Acknowledgement {
            org_id: the_org(),
            member_id: dave().id,
            device: device_key(DAVE),
            epoch: Epoch::new(epoch),
            root: trie_of_members(&[ada()]).root_hash().unwrap(),
            signature: Signature64([0; 64]),
        };
        ack.signature = Signature64(device_keypair([DAVE; 32]).sign(&ack.signed_bytes()).to_bytes());
        ack
    }

    /// The fewest padding changes for which `store_data` holding them and
    /// keeping `notice` exceeds the bound.
    pub fn changes_filling_the_bound(store_data: &StoreData, notice: &RevocationNotice) -> usize {
        fewest_exceeding(|count| {
            let mut filled = store_data.clone();
            filled.provisional_changes = revokes(count);
            filled.kept_revocation_notices.push(org_node::store::KeptNotice {
                notice: notice.clone(),
                from: org_node::test_fixtures::device_key(0x21),
            });
            bound_len(&filled)
        })
    }
}

// A notice from a Device the record lists, about this store's own Persona,
// that `check_notice` passes is kept and sealed, and acts on nothing: the
// record, the Persona and every other field stay as they were, and no
// secret is asked for (`check_received` takes none). A notice
// `check_notice` refuses keeps nothing.
// verifies: LLR-gb7p7c
#[test]
fn a_notice_check_notice_passes_is_kept_and_nothing_is_deleted() {
    let bob_device = org_node::test_fixtures::device_key(notices::BOB);
    let ada_device = org_node::test_fixtures::device_key(notices::ADA);
    let mut svc = notices::service(notices::BOB, notices::bob_store());
    let mut expected = svc.store_data(&bob_device).unwrap().clone();
    let notice = notices::notice_for_bob();
    let body = org_node::wire::encode_message(&org_node::wire::WireMessage::Revocation(notice.clone())).unwrap();

    let (checked, change) = svc.check_received(&mut rand::rngs::OsRng, &bob_device, ada_device, &body).unwrap();
    assert!(matches!(checked, org_node::service::Checked::RevocationNoticeKept));
    expected.kept_revocation_notices.push(org_node::store::KeptNotice { notice: notice.clone(), from: ada_device });
    assert_eq!(
        notices::encoded(&svc, notices::BOB),
        postcard::to_allocvec(&expected).unwrap(),
        "the notice is appended and nothing else changes"
    );
    let Some(org_node::service::StoreChange::Sealed(write)) = change else { panic!("one sealed store change") };
    let reopened = org_node::store::PersonaStore::unseal(
        &write.sealed,
        bob_device,
        org_node::test_fixtures::device_secret([notices::BOB; 32]),
    )
    .unwrap();
    assert_eq!(
        reopened.data().unwrap().kept_revocation_notices,
        vec![org_node::store::KeptNotice { notice: notice.clone(), from: ada_device }]
    );
    assert!(reopened.data().unwrap().record.is_some(), "the record stays");

    let held = notices::encoded(&svc, notices::BOB);
    let mut notice_for_another_member = notice;
    notice_for_another_member.member_id = org_node::MemberId::new([0x09; 32]);
    let body = org_node::wire::encode_message(&org_node::wire::WireMessage::Revocation(notice_for_another_member)).unwrap();
    let refused = svc.check_received(&mut rand::rngs::OsRng, &bob_device, ada_device, &body);
    assert!(matches!(refused, Err(org_node::error::OrgNodeError::RevocationNotForThisDevice { .. })));
    assert_eq!(notices::encoded(&svc, notices::BOB), held, "a refused notice keeps nothing");
}

// The bound counts kept notices: with one padding change fewer than fills
// the bound the notice is kept; at the filling count it is refused with
// `ProvisionalLimit`, no store change, the store as it was.
// verifies: LLR-gb7p7c, LLR-m76r4k
#[test]
fn a_kept_notice_counts_against_the_bound() {
    let bob_device = org_node::test_fixtures::device_key(notices::BOB);
    let ada_device = org_node::test_fixtures::device_key(notices::ADA);
    let notice = notices::notice_for_bob();
    let body = org_node::wire::encode_message(&org_node::wire::WireMessage::Revocation(notice.clone())).unwrap();
    let count = notices::changes_filling_the_bound(&notices::bob_store(), &notice);

    let mut fits = notices::bob_store();
    fits.provisional_changes = notices::revokes(count - 1);
    let mut svc = notices::service(notices::BOB, fits);
    let (checked, change) = svc.check_received(&mut rand::rngs::OsRng, &bob_device, ada_device, &body).unwrap();
    assert!(matches!(checked, org_node::service::Checked::RevocationNoticeKept));
    assert!(change.is_some(), "kept and sealed below the bound");

    let mut full = notices::bob_store();
    full.provisional_changes = notices::revokes(count);
    assert!(notices::bound_len(&full) <= org_node::store::MAX_PROVISIONAL_BYTES, "fixture: the store is within the bound");
    let mut svc = notices::service(notices::BOB, full);
    let before = notices::encoded(&svc, notices::BOB);
    let refused = svc.check_received(&mut rand::rngs::OsRng, &bob_device, ada_device, &body);
    assert!(matches!(
        refused,
        Err(org_node::error::OrgNodeError::ProvisionalLimit { limit }) if limit == org_node::store::MAX_PROVISIONAL_BYTES
    ));
    assert_eq!(notices::encoded(&svc, notices::BOB), before, "nothing kept");
}

// An acknowledgement is decided by `check_acknowledgement` with the sender
// the transport authenticated and no other key: from the Device it names,
// which the record no longer lists, it is `Acknowledged` with no store
// change; relayed by another unlisted Device it is that function's
// refusal, not the listing rule's; for a Device the record lists it is
// that function's own refusal, unchanged.
// verifies: LLR-xudkh4
#[test]
fn an_acknowledgement_is_decided_by_check_acknowledgement_with_the_sender_and_no_store_change() {
    let ada = org_node::test_fixtures::device_key(notices::ADA);
    let dave = org_node::test_fixtures::device_key(notices::DAVE);
    let eve = org_node::test_fixtures::device_key(notices::EVE);
    let ack = notices::ack_by_dave(3);
    let body = org_node::wire::encode_message(&org_node::wire::WireMessage::Acknowledgement(ack.clone())).unwrap();

    let mut svc = notices::service(notices::ADA, notices::ada_store(3, &[notices::ada()]));
    let before = notices::encoded(&svc, notices::ADA);
    let (checked, change) = svc.check_received(&mut rand::rngs::OsRng, &ada, dave, &body).unwrap();
    let org_node::service::Checked::Acknowledged(verified) = checked else { panic!("decided as an acknowledgement") };
    assert_eq!(verified.into_inner(), ack);
    assert!(change.is_none(), "no store change");

    let relayed = svc.check_received(&mut rand::rngs::OsRng, &ada, eve, &body);
    assert!(matches!(relayed, Err(org_node::error::OrgNodeError::AcknowledgementNotFromItsDevice { .. })));
    assert_eq!(notices::encoded(&svc, notices::ADA), before, "the store is unchanged");

    let mut listing = notices::service(notices::ADA, notices::ada_store(3, &[notices::ada(), notices::dave()]));
    let refused = listing.check_received(&mut rand::rngs::OsRng, &ada, dave, &body);
    assert!(matches!(refused, Err(org_node::error::OrgNodeError::AcknowledgementForListedDevice { .. })));
}

// Normal: an acknowledgement from a Device held as revoked is accepted,
// kept once with the MemberId it was listed under (the held
// RevokedDevice's, not the one the acknowledgement names), and sealed; a
// second one from that Device, for another epoch, is accepted and keeps
// nothing.
// verifies: LLR-xudkh4, LLR-py43n3
#[test]
fn an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed() {
    let ada = org_node::test_fixtures::device_key(notices::ADA);
    let dave = org_node::test_fixtures::device_key(notices::DAVE);
    let ack = notices::ack_by_dave(3);
    let body = org_node::wire::encode_message(&org_node::wire::WireMessage::Acknowledgement(ack.clone())).unwrap();
    let mut store_data = notices::ada_store(3, &[notices::ada()]);
    let listed_under = org_node::MemberId::new([0x0d; 32]);
    assert_ne!(listed_under, ack.member_id, "the held MemberId differs from the acknowledgement's");
    store_data.revoked_devices = vec![org_node::store::RevokedDevice { member_id: listed_under, device_key: dave }];
    let mut svc = notices::service(notices::ADA, store_data);

    let (checked, change) = svc.check_received(&mut rand::rngs::OsRng, &ada, dave, &body).unwrap();
    assert!(matches!(checked, org_node::service::Checked::Acknowledged(_)));
    let Some(org_node::service::StoreChange::Sealed(write)) = change else { panic!("kept and sealed") };
    let reopened = org_node::store::PersonaStore::unseal(&write.sealed, ada, org_node::test_fixtures::device_secret([notices::ADA; 32])).unwrap();
    assert_eq!(
        reopened.data().unwrap().acknowledged,
        vec![org_node::store::AcknowledgedEntry { acknowledgement: ack, member_id: listed_under }]
    );

    let again = org_node::wire::encode_message(&org_node::wire::WireMessage::Acknowledgement(notices::ack_by_dave(2))).unwrap();
    let before = notices::encoded(&svc, notices::ADA);
    let (checked, change) = svc.check_received(&mut rand::rngs::OsRng, &ada, dave, &again).unwrap();
    assert!(matches!(checked, org_node::service::Checked::Acknowledged(_)));
    assert!(change.is_none(), "one entry per Device");
    assert_eq!(notices::encoded(&svc, notices::ADA), before);
}

// Abnormal: from a Device the record does not list and the store does
// not hold as revoked, the acknowledgement is accepted and nothing is
// kept (owner ruling Q3 of 2026-11-03).
// verifies: LLR-xudkh4
#[test]
fn an_acknowledgement_from_a_device_not_held_as_revoked_is_accepted_and_not_kept() {
    let ada = org_node::test_fixtures::device_key(notices::ADA);
    let dave = org_node::test_fixtures::device_key(notices::DAVE);
    let body = org_node::wire::encode_message(&org_node::wire::WireMessage::Acknowledgement(notices::ack_by_dave(3))).unwrap();
    let mut svc = notices::service(notices::ADA, notices::ada_store(3, &[notices::ada()]));
    let (checked, change) = svc.check_received(&mut rand::rngs::OsRng, &ada, dave, &body).unwrap();
    assert!(matches!(checked, org_node::service::Checked::Acknowledged(_)));
    assert!(change.is_none());
    assert!(svc.store_data(&ada).unwrap().acknowledged.is_empty());
}

// Normal: each Device held as revoked, in held order, with its entry
// of the Acknowledged record or none. Abnormal: an unknown key is
// `PersonaNotHeld`; a store with no record `OrgNotHeld`. It changes
// nothing.
// verifies: LLR-dsjht5
#[test]
fn removed_devices_lists_each_device_held_as_revoked_with_its_acknowledgement() {
    use org_node::service::RemovedDeviceView;
    use org_node::store::{AcknowledgedEntry, RevokedDevice};
    let ada = org_node::test_fixtures::device_key(notices::ADA);
    let dave = org_node::test_fixtures::device_key(notices::DAVE);
    let erin = org_node::test_fixtures::device_key(0x2e);
    let mut store_data = notices::ada_store(3, &[notices::ada()]);
    store_data.revoked_devices = vec![
        RevokedDevice { member_id: notices::dave().id, device_key: dave },
        RevokedDevice { member_id: org_node::MemberId::new([0x0e; 32]), device_key: erin },
    ];
    store_data.acknowledged = vec![AcknowledgedEntry { acknowledgement: notices::ack_by_dave(3), member_id: notices::dave().id }];
    let svc = notices::service(notices::ADA, store_data);
    let before = notices::encoded(&svc, notices::ADA);
    assert_eq!(
        svc.removed_devices(&ada).unwrap(),
        vec![
            RemovedDeviceView { member_id: notices::dave().id, device_key: dave, acknowledgement: Some(notices::ack_by_dave(3)) },
            RemovedDeviceView { member_id: org_node::MemberId::new([0x0e; 32]), device_key: erin, acknowledgement: None },
        ]
    );
    assert_eq!(notices::encoded(&svc, notices::ADA), before);
    assert!(matches!(
        svc.removed_devices(&org_node::test_fixtures::device_key(0x7f)),
        Err(org_node::error::OrgNodeError::PersonaNotHeld { .. })
    ));
    let unrecorded = notices::service(notices::ADA, org_node::test_fixtures::empty_store(notices::persona(&notices::ada(), 0x61)));
    assert!(matches!(
        unrecorded.removed_devices(&ada),
        Err(org_node::error::OrgNodeError::OrgNotHeld { org_id }) if org_id == support::the_org()
    ));
}
