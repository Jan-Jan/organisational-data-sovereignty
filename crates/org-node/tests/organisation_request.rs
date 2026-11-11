#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The organisation request and its answer (SDD-pkqc2j): built for the Devices of the current record or the
//! Invite's issuer, answered with the record and its key to a listed Device and with its notice to a
//! revoked one, and an answer kept only while behind.

mod support;

use org_members::hasher::Blake3Hasher;
use org_members::{MemberId, Name, Surname};
use org_node::ids::OrgId;
use org_node::service::{Checked, OrgService, StoreChange};
use org_node::store::{MemberSnapshot, Origin, RevokedDevice, StoreData};
use org_node::test_fixtures::device_key;
use org_node::wire::{decode_message, encode_message, WireMessage, MAX_MESSAGE};
use org_node::{ChainAccount, Epoch, OrgNodeError, OrgPrivateKey, RootHash};
use rand::rngs::OsRng;
use support::fixtures::{
    acme_store, address_strings, addresses, alice, alice_persona, invite_to_bob, own, own_encoded, own_service, record, service_under,
    snapshot, RECORD_KEY,
};
use support::{assert_not_rendered, the_org, trie_of_members};

const CAROL: u8 = 0x22;
const BOB: u8 = 0x21;
const STRANGER: u8 = 0x2f;

fn record_key() -> OrgPrivateKey {
    OrgPrivateKey::from(RECORD_KEY)
}

fn listed_members() -> Vec<MemberSnapshot> {
    vec![alice(), snapshot(3, "carol", 0x12, &[CAROL])]
}

/// The Admin's store: the record at epoch 3 listing this Device and Carol's; Bob's Device held as revoked.
fn admin_data() -> StoreData {
    let mut store_data = acme_store(alice_persona(), record(3, &listed_members()));
    store_data.revoked_devices = vec![RevokedDevice { member_id: MemberId::new([2; 32]), device_key: device_key(BOB) }];
    store_data
}

fn request(list: &[&str]) -> Vec<u8> {
    encode_message(&WireMessage::OrganisationRequest { org_id: the_org(), addresses: address_strings(list) }).unwrap()
}

fn answer_body(epoch: u64, members: &[MemberSnapshot], key: [u8; 32]) -> Vec<u8> {
    encode_message(&WireMessage::Organisation {
        org_id: the_org(),
        epoch: Epoch::new(epoch),
        record_snapshot: postcard::to_allocvec(members).unwrap(),
        org_private_key: OrgPrivateKey::from(key),
    })
    .unwrap()
}

// verifies: LLR-vyank2
#[test]
fn build_organisation_request_addresses_every_listed_device_or_the_invites_issuer() {
    let mut store_data = admin_data();
    let mut members = listed_members();
    members.push(snapshot(5, "erin", 0x14, &[0x25]));
    store_data.record = Some(record(3, &members));
    let svc = own_service(store_data);
    let before = own_encoded(&svc);
    let given = address_strings(&["198.51.100.4:7000"]);
    let requests = svc.build_organisation_request(&own(), given.clone()).unwrap();
    assert_eq!(
        requests.iter().map(|message| message.recipient).collect::<Vec<_>>(),
        vec![device_key(CAROL), device_key(0x25)],
        "every listed Device but this one, in record order"
    );
    for message in &requests {
        assert_eq!(
            decode_message(&message.body).unwrap(),
            WireMessage::OrganisationRequest { org_id: the_org(), addresses: given.clone() }
        );
    }
    assert_eq!(own_encoded(&svc), before, "building changes nothing");

    // A provisional Member, holding no record, asks its Invite's issuer.
    let mut svc = OrgService::new(vec![]).unwrap();
    let invite = invite_to_bob();
    let (member, _secret, _sealed) = svc.propose_member(&mut OsRng, &invite, "Bob", "Builder", &["bob"]).unwrap();
    let requests = svc.build_organisation_request(&member, vec![]).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].recipient, device_key(5));

    // Refusals: bad addresses (nothing built); a founding store with no proxy; an unknown Device.
    assert!(matches!(
        svc.build_organisation_request(&member, vec!["a".repeat(257)]),
        Err(OrgNodeError::InvalidField { field: "request.addresses", .. })
    ));
    let (founder, _secret, _sealed) = svc.propose_founding(&mut OsRng, "Alice", "Smith", "alice", "Acme".into()).unwrap();
    assert!(matches!(svc.build_organisation_request(&founder, vec![]), Err(OrgNodeError::FoundingNotWritten)));
    assert!(matches!(svc.build_organisation_request(&device_key(0x40), vec![]), Err(OrgNodeError::PersonaNotHeld { .. })));
}

// Abnormal: a founding store whose proxy is recorded and not yet adopted holds neither a record nor a
// provisional Member: it is refused with `OrgNotHeld` naming the Organisation, and nothing changes.
// verifies: LLR-vyank2
#[test]
fn build_organisation_request_refuses_a_recorded_founding_not_yet_adopted_as_org_not_held() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (founder, _secret, _sealed) = svc.propose_founding(&mut OsRng, "Alice", "Smith", "alice", "Acme".into()).unwrap();
    crate::support::unread(svc.record_founding_proxy(&mut OsRng, &founder, the_org(), ChainAccount::new([0x5a; 32])).unwrap());
    let held = svc.store_data(&founder).unwrap();
    assert!(held.record.is_none() && held.member.is_none(), "fixture: neither a record nor a provisional Member");
    let before = postcard::to_allocvec(held).unwrap();
    assert_eq!(
        svc.build_organisation_request(&founder, address_strings(&["198.51.100.4:7000"])).err(),
        Some(OrgNodeError::OrgNotHeld { org_id: the_org() })
    );
    assert_eq!(postcard::to_allocvec(svc.store_data(&founder).unwrap()).unwrap(), before, "nothing changes");
}

// verifies: LLR-7kdhst
#[test]
fn a_listed_requester_is_answered_with_the_full_record_and_its_key() {
    let mut svc = own_service(admin_data());
    let held_before = svc.store_data(&own()).unwrap().clone();
    let (checked, change) = svc.check_received(&mut OsRng, &own(), device_key(CAROL), &request(&["192.0.2.22:4433"])).unwrap();
    let Checked::OrganisationRequest { answer } = checked else { panic!("an answer, got {checked:?}") };
    assert_eq!(answer.recipient, device_key(CAROL), "answered to the requester");
    let WireMessage::Organisation { org_id, epoch, record_snapshot, org_private_key } = decode_message(&answer.body).unwrap() else {
        panic!("the Organisation kind")
    };
    assert_eq!((org_id, epoch), (the_org(), Epoch::new(3)));
    let snapshots: Vec<MemberSnapshot> = postcard::from_bytes(&record_snapshot).unwrap();
    assert_eq!(snapshots, listed_members(), "the full current record");
    assert_eq!(org_private_key, record_key(), "with the record's Organisation private key");
    let rendered = format!("{:?}", decode_message(&answer.body).unwrap());
    assert!(rendered.contains("[REDACTED]"), "the key renders as its marker: {rendered}");
    assert_not_rendered(&rendered, &RECORD_KEY, "the answer's Organisation private key");
    let StoreChange::Sealed(write) = change.expect("the requester's address-book entry seals the store");
    assert_eq!(write.device_key, own(), "this Persona's store is sealed");
    let mut after = svc.store_data(&own()).unwrap().clone();
    assert_eq!(after.address_book, vec![(device_key(CAROL), addresses(&["192.0.2.22:4433"]))], "the requester's entry");
    after.address_book = held_before.address_book.clone();
    assert_eq!(
        postcard::to_allocvec(&after).unwrap(),
        postcard::to_allocvec(&held_before).unwrap(),
        "the address-book entry is the one change"
    );
}

// Abnormal: a listed requester whose addresses do not parse is refused naming the field; nothing is
// kept and nothing is answered.
// verifies: LLR-7kdhst
#[test]
fn a_listed_requesters_invalid_addresses_are_refused_and_answered_nothing() {
    let mut svc = own_service(admin_data());
    let before = own_encoded(&svc);
    let refused = svc.check_received(&mut OsRng, &own(), device_key(CAROL), &request(&[&"a".repeat(257)]));
    assert!(matches!(refused, Err(OrgNodeError::InvalidField { field: "request.addresses", .. })), "{refused:?}");
    assert_eq!(own_encoded(&svc), before, "a refused request keeps nothing and answers nothing");
}

// verifies: LLR-j338wq
#[test]
fn a_revoked_requester_is_answered_with_its_notice_and_nothing_else() {
    let mut svc = own_service(admin_data());
    let root = svc.store_data(&own()).unwrap().record.as_ref().unwrap().root_hash;
    let (checked, change) = svc.check_received(&mut OsRng, &own(), device_key(BOB), &request(&[])).unwrap();
    let Checked::OrganisationRequest { answer } = checked else { panic!("an answer") };
    assert_eq!(answer.recipient, device_key(BOB));
    let WireMessage::Revocation(notice) = decode_message(&answer.body).unwrap() else { panic!("its notice, nothing else") };
    assert_eq!((notice.org_id, notice.member_id, notice.device), (the_org(), MemberId::new([2; 32]), device_key(BOB)));
    notice.proof.verify::<Blake3Hasher>(&root, &notice.member_id, &notice.device).unwrap();
    assert!(change.is_some(), "the sender's address-book entry is set");
    let book = &svc.store_data(&own()).unwrap().address_book;
    assert_eq!(book, &vec![(device_key(BOB), addresses(&[]))]);
}

// verifies: LLR-283x6f, LLR-j338wq
#[test]
fn a_never_listed_requester_is_refused_and_answered_nothing() {
    let mut svc = own_service(admin_data());
    let before = own_encoded(&svc);
    assert!(matches!(
        svc.check_received(&mut OsRng, &own(), device_key(STRANGER), &request(&["192.0.2.47:4433"])),
        Err(OrgNodeError::SenderNotListed { org_id }) if org_id == the_org()
    ));
    assert_eq!(own_encoded(&svc), before, "nothing kept, not even its addresses");
}

// verifies: LLR-xc9c6w
#[test]
fn an_answer_over_the_message_bound_is_refused() {
    let mut store_data = admin_data();
    let long_name = Name::parse(&"N".repeat(128)).unwrap();
    let long_surname = Surname::parse(&"S".repeat(128)).unwrap();
    let mut members = listed_members();
    for index in 0..3000u32 {
        let mut id = [0x70; 32];
        id[..4].copy_from_slice(&index.to_be_bytes());
        members.push(MemberSnapshot {
            id: MemberId::new(id),
            name: long_name.clone(),
            surname: long_surname.clone(),
            ..snapshot(0x70, "m", 0x15, &[0x26])
        });
    }
    assert!(postcard::to_allocvec(&members).unwrap().len() > MAX_MESSAGE, "fixture: the record is over the bound");
    // The record is held as given; its root is not recalculated for a listed requester's answer.
    let mut held = record(3, &[]);
    held.trie_members = members;
    held.root_hash = RootHash::new([0x33; 32]);
    store_data.record = Some(held);
    let mut svc = own_service(store_data);
    let before = own_encoded(&svc);
    let refused = svc.check_received(&mut OsRng, &own(), device_key(CAROL), &request(&["192.0.2.22:4433"]));
    assert!(matches!(refused, Err(OrgNodeError::MessageTooLarge { length }) if length > MAX_MESSAGE));
    assert_eq!(own_encoded(&svc), before, "answered whole or not at all: no address-book change either");
}

// verifies: LLR-kd7u2r
#[test]
fn an_answer_is_kept_only_from_a_listed_device_while_behind_and_for_a_later_epoch() {
    let next = vec![alice(), snapshot(3, "carol", 0x12, &[CAROL]), snapshot(6, "fay", 0x16, &[0x27])];
    let next_root = trie_of_members(&next).root_hash().unwrap();

    // Not behind: refused, nothing kept.
    let mut svc = own_service(admin_data());
    assert!(matches!(
        svc.check_received(&mut OsRng, &own(), device_key(CAROL), &answer_body(5, &next, [0x61; 32])),
        Err(OrgNodeError::OrganisationNotRequested { org_id }) if org_id == the_org()
    ));

    let mut behind = admin_data();
    behind.behind = Some(Epoch::new(5));
    let mut svc = own_service(behind);
    let before = own_encoded(&svc);
    // From a Device the record does not list (a revoked one included).
    for sender in [STRANGER, BOB] {
        assert!(matches!(
            svc.check_received(&mut OsRng, &own(), device_key(sender), &answer_body(5, &next, [0x61; 32])),
            Err(OrgNodeError::SenderNotListed { .. })
        ));
    }
    // For an epoch not after the record's.
    for epoch in [2, 3] {
        assert!(matches!(
            svc.check_received(&mut OsRng, &own(), device_key(CAROL), &answer_body(epoch, &next, [0x61; 32])),
            Err(OrgNodeError::StaleChainState { .. })
        ));
    }
    assert_eq!(own_encoded(&svc), before, "every refusal keeps nothing");

    // Listed, behind, later epoch: kept as an answer, the key derived, no Change set.
    let (checked, change) =
        svc.check_received(&mut OsRng, &own(), device_key(CAROL), &answer_body(5, &next, [0x61; 32])).unwrap();
    assert!(matches!(checked, Checked::OrganisationAnswerKept { epoch } if epoch == Epoch::new(5)));
    assert!(change.is_some());
    let store_data = svc.store_data(&own()).unwrap();
    let kept = store_data.provisional_organisations.last().unwrap();
    assert_eq!(kept.origin, Origin::Answer { from: device_key(CAROL) });
    assert_eq!((kept.epoch, kept.root), (Epoch::new(5), next_root));
    assert_eq!(kept.org_pub_key, OrgPrivateKey::from([0x61; 32]).x25519_keypair().org_public_key().unwrap());
    assert!(kept.change_set.is_none() && kept.applied.is_empty());
    assert_eq!(store_data.record.as_ref().unwrap().epoch, Epoch::new(3), "kept, never adopted on receipt");

    // The same identity again: already held, no second copy.
    let (checked, change) =
        svc.check_received(&mut OsRng, &own(), device_key(CAROL), &answer_body(5, &next, [0x61; 32])).unwrap();
    assert!(matches!(checked, Checked::AlreadyHeld));
    assert!(change.is_none());
}

// Abnormal: an answer whose record snapshot
// does not decode, or holds a member value its type's parse refuses, is refused (`InvalidField`, the
// member's field named `member.*`), nothing kept, no change returned, from a listed Device while behind
// and for a later epoch, so only the snapshot is at fault.
// verifies: LLR-kd7u2r
#[test]
fn an_answer_whose_snapshot_does_not_decode_or_parse_is_refused_and_keeps_nothing() {
    let next = vec![alice(), snapshot(3, "carol", 0x12, &[CAROL]), snapshot(6, "fayfay", 0x16, &[0x27])];
    let mut behind = admin_data();
    behind.behind = Some(Epoch::new(5));
    let mut svc = own_service(behind);
    let before = own_encoded(&svc);
    let with_snapshot = |record_snapshot: Vec<u8>| {
        encode_message(&WireMessage::Organisation {
            org_id: the_org(),
            epoch: Epoch::new(5),
            record_snapshot,
            org_private_key: OrgPrivateKey::from([0x61; 32]),
        })
        .unwrap()
    };
    let good = postcard::to_allocvec(&next).unwrap();
    // A Handle the parse refuses (upper case), in bytes that otherwise decode.
    let at = good.windows(6).position(|window| window == b"fayfay").expect("fixture: the handle's bytes");
    let mut upper = good.clone();
    upper[at..at + 6].copy_from_slice(b"FAYFAY");
    let refused = svc.check_received(&mut OsRng, &own(), device_key(CAROL), &with_snapshot(upper)).map(|_| ());
    assert!(matches!(refused, Err(OrgNodeError::InvalidField { field: "member.handle", .. })), "{refused:?}");
    // Bytes that do not decode as a snapshot: cut short, and garbage.
    for bytes in [good[..good.len() - 3].to_vec(), vec![0xff; 7]] {
        let refused = svc.check_received(&mut OsRng, &own(), device_key(CAROL), &with_snapshot(bytes)).map(|_| ());
        assert!(matches!(refused, Err(OrgNodeError::InvalidField { .. })), "{refused:?}");
    }
    assert_eq!(own_encoded(&svc), before, "every refusal keeps nothing");
    assert!(svc.provisional_organisations(&own()).unwrap().is_empty());
    // The same answer with its snapshot intact is kept: the refusals are the snapshot's alone.
    let (checked, _) = svc.check_received(&mut OsRng, &own(), device_key(CAROL), &with_snapshot(good)).unwrap();
    assert!(matches!(checked, Checked::OrganisationAnswerKept { .. }), "{checked:?}");
}

// verifies: LLR-h9d65p
#[test]
fn a_request_sets_its_senders_address_book_entry_and_seals_only_on_change() {
    let mut svc = own_service(admin_data());
    let first = ["192.0.2.22:4433"];
    let (_, change) = svc.check_received(&mut OsRng, &own(), device_key(CAROL), &request(&first)).unwrap();
    let StoreChange::Sealed(write) = change.expect("a new entry seals the store");
    assert_eq!(write.device_key, own());
    let (_, change) = svc.check_received(&mut OsRng, &own(), device_key(CAROL), &request(&first)).unwrap();
    assert!(change.is_none(), "the same addresses again change nothing");
    let second = ["192.0.2.23:4433", "198.51.100.23:4433"];
    let (_, change) = svc.check_received(&mut OsRng, &own(), device_key(CAROL), &request(&second)).unwrap();
    assert!(change.is_some(), "other addresses replace the entry");
    let (_, change) = svc.check_received(&mut OsRng, &own(), device_key(BOB), &request(&first)).unwrap();
    assert!(change.is_some(), "a revoked requester's entry is set too");
    assert_eq!(
        svc.store_data(&own()).unwrap().address_book,
        vec![(device_key(CAROL), addresses(&second)), (device_key(BOB), addresses(&first))],
        "one entry per Device key"
    );
}

// In a store with no record: a provisional Member keeps the answer to the request it sent its issuer,
// for the Organisation its expected admission names, from whichever Device sent it, only while behind;
// an answer for another Organisation, and any answer in a founding store, is `OrgNotHeld`.
// verifies: LLR-e6w3nr
#[test]
fn a_provisional_member_keeps_an_expected_answer_only_while_behind() {
    let invite = invite_to_bob();
    let mut proposing = OrgService::new(vec![]).unwrap();
    let (member, secret, _sealed) =
        proposing.propose_member(&mut OsRng, &invite, "Bob", "Builder", &["bob"]).unwrap();
    let created = proposing.store_data(&member).unwrap().clone();
    assert!(created.record.is_none() && created.member.is_some(), "fixture: a provisional Member's store");
    let members = vec![alice(), snapshot(7, "bob", 0x17, &[0x28])];
    let root = trie_of_members(&members).root_hash().unwrap();

    // Not behind: refused, nothing kept.
    let mut svc = service_under(&secret, created.clone());
    let before = postcard::to_allocvec(svc.store_data(&member).unwrap()).unwrap();
    assert!(matches!(
        svc.check_received(&mut OsRng, &member, device_key(5), &answer_body(1, &members, [0x61; 32])),
        Err(OrgNodeError::OrganisationNotRequested { org_id }) if org_id == the_org()
    ));
    assert_eq!(postcard::to_allocvec(svc.store_data(&member).unwrap()).unwrap(), before);

    // Behind: an answer for an Organisation no expected admission names is refused.
    let mut behind = created.clone();
    behind.behind = Some(Epoch::new(1));
    let expected_before = behind.expected_admissions.clone();
    let mut svc = service_under(&secret, behind);
    let elsewhere = OrgId::new([0xbb; 20]);
    let other = encode_message(&WireMessage::Organisation {
        org_id: elsewhere,
        epoch: Epoch::new(1),
        record_snapshot: postcard::to_allocvec(&members).unwrap(),
        org_private_key: OrgPrivateKey::from([0x61; 32]),
    })
    .unwrap();
    assert!(matches!(
        svc.check_received(&mut OsRng, &member, device_key(5), &other),
        Err(OrgNodeError::OrgNotHeld { org_id }) if org_id == elsewhere
    ));

    // Behind, the expected Organisation, from a Device nothing lists: kept for verification.
    let (checked, change) =
        svc.check_received(&mut OsRng, &member, device_key(STRANGER), &answer_body(1, &members, [0x61; 32])).unwrap();
    assert!(matches!(checked, Checked::OrganisationAnswerKept { epoch } if epoch == Epoch::new(1)));
    let StoreChange::Sealed(write) = change.expect("the keep seals the store");
    assert_eq!(write.device_key, member);
    let store_data = svc.store_data(&member).unwrap();
    let kept = store_data.provisional_organisations.last().unwrap();
    assert_eq!(kept.origin, Origin::Answer { from: device_key(STRANGER) });
    assert_eq!((kept.epoch, kept.root), (Epoch::new(1), root));
    assert_eq!(kept.org_pub_key, OrgPrivateKey::from([0x61; 32]).x25519_keypair().org_public_key().unwrap());
    assert!(kept.change_set.is_none() && kept.applied.is_empty());
    assert!(store_data.record.is_none(), "kept, never adopted on receipt");
    assert_eq!(store_data.expected_admissions, expected_before, "the expectations are untouched");

    // A founding store whose proxy is not recorded holds no expected admission: it keeps no answer.
    let (founder, _secret, _sealed) =
        proposing.propose_founding(&mut OsRng, "Alice", "Smith", "alice", "Acme".into()).unwrap();
    assert!(matches!(
        proposing.check_received(&mut OsRng, &founder, device_key(5), &answer_body(1, &members, [0x61; 32])),
        Err(OrgNodeError::OrgNotHeld { org_id }) if org_id == the_org()
    ));
}
