#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The Invite (SDD-n2x3gb): the Blob, built and parsed here, naming the
//! field that fails (LLR-9jxwq4), and its bookkeeping in the sealed store — the
//! outstanding invite `build_invite` records with the invitee name the
//! inviter typed (LLR-h9f5c9). org-io runs the exchange; these tests play
//! org-io, holding each returned change as "the disk" (`support::Node`).
//! The Invite names its one issuing Device and its endpoint addresses.

mod support;

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::invitation::{EndpointAddresses, Invite, InviteId};
use org_node::store::OutstandingInvite;
use org_node::test_fixtures::{device_key, ChainSlots};
use org_node::{DevicePublicKey, PersonaId};
use rand::rngs::OsRng;
use support::*;

/// A node over a fresh storage whose one Persona, alice, founded an
/// Organisation on `chain`; its Device key.
fn founder(tag: &str, chain: &ChainSlots) -> (Node, PersonaId, OrgId, DevicePublicKey) {
    let mut node = Node::new(open_store(tag, "a"));
    let persona_id = node.propose_and_keep_founding(&mut OsRng, handle("alice"), parsed_name("Alice"), parsed_surname("Smith")).unwrap();
    let org_id = found_on_chain(&mut node, chain, &persona_id);
    let device = persona_of(&node, &persona_id).device_key;
    (node, persona_id, org_id, device)
}

/// `node`'s Invite from `device`'s store, its change kept on the disk.
fn build_invite(node: &mut Node, device: &DevicePublicKey, invitee_name: &str) -> Invite {
    let (invite, change) =
        node.svc.build_invite(&mut OsRng, device, invitee_name.into(), vec!["192.0.2.1:7000".into()]).unwrap();
    node.disk.apply(change);
    invite
}

/// A Blob of these raw fields, valid or not.
fn blob(
    org_name: &[u8],
    org_id: &[u8],
    invitee_name: &[u8],
    issuer: &[u8],
    addresses: &[&str],
    invite_id: &[u8],
) -> String {
    Invite::wire_for_test(org_name, org_id, invitee_name, issuer, addresses, invite_id).expect("a WireInvite encodes")
}

// ---------------------------------------------------------------------
// The Blob (LLR-9jxwq4).
// ---------------------------------------------------------------------

// Normal: a Blob `Invite::encode` wrote parses back to an equal Invite, and
// a Blob of valid fields parses and encodes back to the same Blob.
// verifies: LLR-9jxwq4
#[test]
fn an_invite_that_parses_encodes_back_to_the_same_blob() {
    let issuer_bytes = *device_key(7).as_bytes();
    let text = blob(b"Acme", &[1; 20], b"Bob", &issuer_bytes, &["192.0.2.1:7000", "[2001:db8::1]:7000"], &[3; 32]);
    let invite = Invite::parse(&text).unwrap();
    assert_eq!(invite.encode().unwrap(), text);
    let built = Invite {
        org_name: "Acme Co-op".into(),
        org_id: OrgId::new([2; 20]),
        invitee_name: "Bob Builder".into(),
        issuer: device_key(8),
        issuer_addresses: EndpointAddresses::parse(vec!["198.51.100.7:7000".into()]).unwrap(),
        invite_id: InviteId::new([4; 32]),
    };
    assert_eq!(Invite::parse(&built.encode().unwrap()).unwrap(), built);
}

// Abnormal: a Blob with one field its type refuses is refused naming that
// field.
// verifies: LLR-9jxwq4
#[test]
fn an_invite_that_does_not_parse_is_refused_naming_the_field() {
    let issuer_bytes = *device_key(7).as_bytes();
    let too_long = "a".repeat(257);
    let many = ["x"; 17];
    for (text, field) in [
        (blob(&[0xff, 0xfe], &[1; 20], b"N", &issuer_bytes, &[], &[3; 32]), "invite.org_name"),
        (blob(b"O", &[1; 19], b"N", &issuer_bytes, &[], &[3; 32]), "invite.org_id"),
        (blob(b"O", &[1; 20], &[0xc3], &issuer_bytes, &[], &[3; 32]), "invite.invitee_name"),
        (blob(b"O", &[1; 20], b"N", &[0xff; 32], &[], &[3; 32]), "invite.issuer"),
        (blob(b"O", &[1; 20], b"N", &issuer_bytes[..31], &[], &[3; 32]), "invite.issuer"),
        (blob(b"O", &[1; 20], b"N", &issuer_bytes, &[too_long.as_str()], &[3; 32]), "invite.issuer_addresses"),
        (blob(b"O", &[1; 20], b"N", &issuer_bytes, &many, &[3; 32]), "invite.issuer_addresses"),
        (blob(b"O", &[1; 20], b"N", &issuer_bytes, &[], &[3; 31]), "invite.invite_id"),
    ] {
        assert_eq!(refused_field(Invite::parse(&text).unwrap_err()), field);
    }
}

// Abnormal: with every field invalid at once, the first in order is named;
// then, field by field made valid, the next.
// verifies: LLR-9jxwq4
#[test]
fn invite_parse_refuses_the_first_failing_field_naming_it() {
    let issuer_bytes = *device_key(7).as_bytes();
    let too_long = "a".repeat(257);
    let bad = (vec![0xffu8], vec![1u8; 19], vec![0xffu8], vec![0xffu8; 32], vec![too_long.as_str()], vec![3u8; 31]);
    let good = (b"O".to_vec(), vec![1u8; 20], b"N".to_vec(), issuer_bytes.to_vec(), Vec::<&str>::new(), vec![3u8; 32]);
    let order = ["invite.org_name", "invite.org_id", "invite.invitee_name", "invite.issuer", "invite.issuer_addresses", "invite.invite_id"];
    for (valid_so_far, expected) in order.iter().enumerate() {
        let is_valid = |field_index: usize| field_index < valid_so_far;
        let text = blob(
            if is_valid(0) { &good.0 } else { &bad.0 },
            if is_valid(1) { &good.1 } else { &bad.1 },
            if is_valid(2) { &good.2 } else { &bad.2 },
            if is_valid(3) { &good.3 } else { &bad.3 },
            if is_valid(4) { &good.4 } else { &bad.4 },
            if is_valid(5) { &good.5 } else { &bad.5 },
        );
        assert_eq!(refused_field(Invite::parse(&text).unwrap_err()), *expected, "{valid_so_far} fields valid");
    }
}

// Normal (boundary): an Invite whose issuer gave no endpoint address — a
// Networked endpoint discovered by its key — parses, with no address.
// verifies: LLR-9jxwq4
#[test]
fn an_invite_with_zero_issuer_addresses_parses() {
    let text = blob(b"Acme", &[1; 20], b"Bob", device_key(7).as_bytes(), &[], &[3; 32]);
    let invite = Invite::parse(&text).unwrap();
    assert!(invite.issuer_addresses.as_slice().is_empty());
    assert_eq!(invite.encode().unwrap(), text);
}

// Abnormal: text that is not a Blob — not Base64, or Base64 of bytes that
// are no Invite — is refused naming `invite`.
// verifies: LLR-9jxwq4
#[test]
fn text_that_is_not_a_blob_is_refused_as_invite() {
    for text in ["not base64 !".to_string(), String::new(), org_node::invitation::armour_for_test(&[0xff; 3])] {
        assert_eq!(refused_field(Invite::parse(&text).unwrap_err()), "invite", "{text:?}");
    }
}

// ---------------------------------------------------------------------
// build_invite (LLR-h9f5c9).
// ---------------------------------------------------------------------

// Normal: each Invite draws a fresh identifier and carries the store's
// Organisation name, the Organisation, the invitee name it was given, the
// issuing Persona's Device and the addresses given.
// verifies: LLR-h9f5c9
#[test]
fn build_invite_draws_a_fresh_id_and_carries_the_stored_name_and_the_issuer_device() {
    let chain = ChainSlots::new();
    let (mut alice, _, org_id, device) = founder("issue-fresh", &chain);
    let first = build_invite(&mut alice, &device, "Bob Builder");
    let second = build_invite(&mut alice, &device, "Bob Builder");
    assert_ne!(first.invite_id, second.invite_id, "drawn at random");
    for invite in [&first, &second] {
        assert_eq!((invite.org_name.as_str(), invite.org_id), ("Acme", org_id), "the stored Organisation name");
        assert_eq!(invite.invitee_name, "Bob Builder");
        assert_eq!(invite.issuer, device, "the issuing Persona's Device");
        assert_eq!(invite.issuer_addresses.as_slice(), ["192.0.2.1:7000".to_string()].as_slice());
    }
}

// Normal: the outstanding invite is recorded with the invitee name and
// reaches the sealed store; the one change is the issuing store's.
// verifies: LLR-h9f5c9
#[test]
fn build_invite_records_the_outstanding_invite_with_its_invitee_name_and_carries_the_issuer() {
    let chain = ChainSlots::new();
    let (mut alice, _, org_id, device) = founder("issue", &chain);
    let (invite, change) =
        alice.svc.build_invite(&mut OsRng, &device, "Bob Builder".into(), vec!["192.0.2.1:7000".into()]).unwrap();
    assert_eq!(invite.issuer, device);
    let org_node::service::StoreChange::Sealed(write) = change;
    assert_eq!(write.device_key, device, "the change is that Persona's store");
    alice.disk.write(write);
    let outstanding = OutstandingInvite { org_id, invite_id: invite.invite_id, invitee_name: "Bob Builder".into() };
    assert_eq!(alice.store_data(&device).unwrap().outstanding_invites, vec![outstanding.clone()]);
    assert_eq!(
        party("issue", "a").disk.unseal(&alice.keychain)[0].data().unwrap().outstanding_invites,
        vec![outstanding],
        "the outstanding invite is in the sealed store"
    );
}

// Abnormal: a store with no current record — a Persona that expects an
// admission — is refused with `OrgNotHeld`, nothing changed or written; so
// is an unknown store (`PersonaNotHeld`) and an issuer address list the
// parse refuses (`invite.issuer_addresses`).
// verifies: LLR-h9f5c9
#[test]
fn build_invite_refuses_a_store_with_no_current_record() {
    let chain = ChainSlots::new();
    let (mut alice, _, _, device) = founder("issue-none", &chain);
    // Carol proposes to join an Organisation no store holds (`propose_member`).
    let elsewhere = Invite {
        org_name: "Elsewhere".into(),
        org_id: OrgId::new([9; 20]),
        invitee_name: "Carol".into(),
        issuer: device_key(0x7d),
        issuer_addresses: org_node::invitation::EndpointAddresses::parse(vec![]).unwrap(),
        invite_id: org_node::invitation::InviteId::new([0x39; 32]),
    };
    let carol = alice.propose_member(&mut OsRng, &elsewhere, "Carol", "Jones", &["carol"]).unwrap();
    let carol_device = persona_of(&alice, &carol).device_key;
    let (before, on_disk) = (encoded_stores(&alice), alice.disk.bytes());
    assert_eq!(
        alice.svc.build_invite(&mut OsRng, &carol_device, "Y".into(), vec![]).err(),
        Some(OrgNodeError::OrgNotHeld { org_id: OrgId::new([9; 20]) })
    );
    assert_eq!(
        alice.svc.build_invite(&mut OsRng, &device_key(0x7e), "Y".into(), vec![]).err(),
        Some(OrgNodeError::PersonaNotHeld { device_key: Box::new(device_key(0x7e)) })
    );
    let refused = alice.svc.build_invite(&mut OsRng, &device, "Y".into(), vec![String::new(); 17]).err().unwrap();
    assert_eq!(refused_field(refused), "invite.issuer_addresses");
    assert_eq!(encoded_stores(&alice), before, "nothing changed");
    assert_eq!(alice.disk.bytes(), on_disk, "nothing written");
    assert!(alice.store_data(&device).unwrap().outstanding_invites.is_empty());
}

mod admission_request_and_cancel {
    use org_members::{MemberId, Name, PersonPublicKey, Surname};
    use org_node::error::OrgNodeError;
    use org_node::invitation::InviteId;
    use org_node::service::{OrgService, StoreChange};
    use org_node::store::{OutstandingInvite, ProvisionalChange};
    use org_node::test_fixtures::{device_key, member_key};
    use org_node::wire::{decode_message, encode_message, WireMessage};
    use rand::rngs::OsRng;

    use super::support::fixtures::{
        acme_store, address_strings, alice, alice_persona, invite_to_bob, kept_request, own, own_encoded, own_service, record, snapshot,
    };
    use super::support::{encoded_store, handle, the_org};

    /// An Admin's store: a record listing this Device (seed 4) and Bob (0x21); two outstanding invites,
    /// each with a kept request; an Admit for each and a Revoke between them.
    fn admin_with_two_requests() -> (OrgService, InviteId, InviteId) {
        let (first, second) = (InviteId::new([0x31; 32]), InviteId::new([0x32; 32]));
        let mut store_data = acme_store(alice_persona(), record(3, &[alice(), snapshot(2, "bob", 0x11, &[0x21])]));
        store_data.outstanding_invites = vec![
            OutstandingInvite { org_id: the_org(), invite_id: first, invitee_name: "Carol".into() },
            OutstandingInvite { org_id: the_org(), invite_id: second, invitee_name: "Dave".into() },
        ];
        store_data.kept_admission_requests = vec![kept_request(first, 0x12, 0x22, "carol"), kept_request(second, 0x13, 0x23, "dave")];
        store_data.provisional_changes = vec![
            ProvisionalChange::Admit { invite_id: first, handle: handle("carol") },
            ProvisionalChange::Revoke { member_id: MemberId::new([2; 32]) },
            ProvisionalChange::Admit { invite_id: second, handle: handle("dave") },
        ];
        let svc = own_service(store_data);
        (svc, first, second)
    }

    /// The admission request Device `device` would send for `invite_id`.
    fn request_body(invite_id: InviteId, member: u8, device: u8) -> Vec<u8> {
        let member_key: PersonPublicKey = member_key(member);
        encode_message(&WireMessage::AdmissionRequest {
            org_id: the_org(),
            invite_id,
            name: Name::parse("Carol").unwrap(),
            surname: Surname::parse("King").unwrap(),
            preferred_handles: vec![handle("carol")],
            member_key,
            device_key: device_key(device),
            addresses: address_strings(&["192.0.2.9:4433"]),
        })
        .unwrap()
    }

    // verifies: LLR-g32hjn
    #[test]
    fn build_admission_request_encodes_the_provisional_member_for_the_issuer_with_the_given_addresses() {
        let mut svc = OrgService::new(vec![]).unwrap();
        let (device, _secret, _sealed) =
            svc.propose_member(&mut OsRng, &invite_to_bob(), "Bob", "Builder", &["bob", "bobby"]).unwrap();
        let before = encoded_store(&svc, &device);
        let given = address_strings(&["198.51.100.1:7000", "[2001:db8::1]:7000"]);

        let message = svc.build_admission_request(&device, given.clone()).unwrap();
        assert_eq!(message.recipient, device_key(5), "the request goes to the Invite's issuer");
        let member = svc.store_data(&device).unwrap().member.clone().expect("a provisional Member");
        let WireMessage::AdmissionRequest { org_id, invite_id, name, surname, preferred_handles, member_key, device_key: sent_device, addresses: sent } =
            decode_message(&message.body).unwrap()
        else {
            panic!("an admission request")
        };
        assert_eq!((org_id, invite_id), (the_org(), InviteId::new([3; 32])));
        assert_eq!((name, surname), (member.name.clone(), member.surname.clone()));
        assert_eq!(preferred_handles, vec![handle("bob"), handle("bobby")]);
        assert_eq!((member_key, sent_device), (member.member_key, member.device_key));
        assert_eq!(sent_device, device, "the request names this Device");
        assert_eq!(sent, given, "the addresses the caller gave, last");

        let again = svc.build_admission_request(&device, given).unwrap();
        assert_eq!(again.body, message.body, "a retry sends the same bytes");
        assert_eq!(encoded_store(&svc, &device), before, "building changes nothing");
    }

    // verifies: LLR-g32hjn, LLR-h4gwns
    #[test]
    fn build_admission_request_refuses_no_provisional_member_and_bad_addresses() {
        let mut svc = OrgService::new(vec![]).unwrap();
        assert!(matches!(
            svc.build_admission_request(&device_key(0x40), vec![]),
            Err(OrgNodeError::PersonaNotHeld { .. })
        ));
        let (founder, _secret, _sealed) =
            svc.propose_founding(&mut OsRng, "Alice", "Smith", "alice", "Acme".into()).unwrap();
        assert!(matches!(
            svc.build_admission_request(&founder, vec![]),
            Err(OrgNodeError::NoProvisionalMember)
        ));
        let (device, _secret, _sealed) = svc.propose_member(&mut OsRng, &invite_to_bob(), "Bob", "Builder", &["bob"]).unwrap();
        let before = encoded_store(&svc, &device);
        let seventeen: Vec<String> = (0..17).map(|port| format!("192.0.2.1:{port}")).collect();
        let too_long = vec!["a".repeat(257)];
        for bad in [seventeen, too_long] {
            assert!(matches!(
                svc.build_admission_request(&device, bad),
                Err(OrgNodeError::InvalidField { field: "request.addresses", .. })
            ));
        }
        assert!(svc.build_admission_request(&device, vec![]).is_ok(), "zero addresses are a list it accepts");
        assert_eq!(encoded_store(&svc, &device), before);
    }

    // verifies: LLR-ag8hjd
    #[test]
    fn cancel_invite_removes_the_request_its_pair_and_its_admit_change() {
        let (mut svc, first, second) = admin_with_two_requests();
        let own = own();
        let record_before = postcard::to_allocvec(&svc.store_data(&own).unwrap().record).unwrap();

        let StoreChange::Sealed(write) = svc.cancel_invite(&mut OsRng, &own, first).unwrap();
        assert_eq!(write.device_key, own);
        let store_data = svc.store_data(&own).unwrap();
        assert_eq!(
            store_data.kept_admission_requests.iter().map(|request| request.invite_id).collect::<Vec<_>>(),
            vec![second]
        );
        assert_eq!(store_data.outstanding_invites.iter().map(|invite| invite.invite_id).collect::<Vec<_>>(), vec![second]);
        assert_eq!(
            store_data.provisional_changes,
            vec![
                ProvisionalChange::Revoke { member_id: MemberId::new([2; 32]) },
                ProvisionalChange::Admit { invite_id: second, handle: handle("dave") },
            ]
        );
        assert_eq!(postcard::to_allocvec(&store_data.record).unwrap(), record_before, "the record is untouched");

        // The cancelled invite is no longer outstanding: a later request carrying it is refused.
        assert!(matches!(
            svc.check_received(&mut OsRng, &own, device_key(0x22), &request_body(first, 0x12, 0x22)),
            Err(OrgNodeError::InviteNotOutstanding { invite_id, .. }) if invite_id == first
        ));
    }

    // verifies: LLR-ag8hjd
    #[test]
    fn cancel_invite_refuses_an_invite_with_no_kept_request() {
        let (mut svc, first, _) = admin_with_two_requests();
        let own = own();
        crate::support::unread(svc.cancel_invite(&mut OsRng, &own, first).unwrap());
        let before = own_encoded(&svc);
        for invite_id in [first, InviteId::new([0x39; 32])] {
            assert!(matches!(
                svc.cancel_invite(&mut OsRng, &own, invite_id),
                Err(OrgNodeError::NoAdmissionRequestKept { invite_id: refused }) if refused == invite_id
            ));
        }
        assert!(matches!(
            svc.cancel_invite(&mut OsRng, &device_key(0x40), first),
            Err(OrgNodeError::PersonaNotHeld { .. })
        ));
        assert_eq!(own_encoded(&svc), before, "a refusal changes nothing");
    }
}
