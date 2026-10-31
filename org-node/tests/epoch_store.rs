#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The epoch state of one Organisation in its Persona store (SDD-undd7q):
//! `StoreData`'s fields and their encoding order (LLR-d925uu), the Persona
//! record built from parsed types (LLR-pv2rdy) and the endpoint addresses one
//! Device keeps for another (LLR-h4gwns).

mod support;

use org_members::{Handle, MemberId, Name, RootHash, Surname};
use org_node::invitation::{EndpointAddresses, EndpointAddressesError, InviteId};
use org_node::store::{
    ExpectedAdmission, OrgRecord, OrgRecordParts, Origin, OutstandingInvite,
    PersonaRecord, PersonaRecordParts, PersonaStore, ProvisionalChange, ProvisionalFounding,
    ProvisionalFoundingParts, ProvisionalMember, ProvisionalOrganisation, ProvisionalOrganisationParts, RevokedDevice,
    StoreData,
};
use org_node::test_fixtures::{
    bob_absence_notice, device_key, device_secret, member_key, seal_for_test, store_for_test,
};
use org_node::{ChainAccount, Epoch, OrgPrivateKey, OrgPublicKey};
use rand::rngs::OsRng;
use support::fixtures::{addresses, bob, kept_request, persona_parts, record_parts};
use support::{handle, the_org};

/// The device seed byte of the Persona every store in this file holds.
const DEVICE: u8 = 0x12;

fn org_pub_key(seed: u8) -> OrgPublicKey {
    OrgPublicKey::parse(member_key(seed).as_bytes()).unwrap()
}

/// A Persona whose `handle` is `handle`.
fn persona(handle: Option<Handle>) -> PersonaRecord {
    PersonaRecord::from(PersonaRecordParts {
        org_id: Some(the_org()),
        handle,
        ..persona_parts("p1", [0x11; 32], device_key(DEVICE))
    })
}

/// A record at epoch 1 listing Bob, with a value in every field.
fn full_record() -> OrgRecord {
    OrgRecord::from(OrgRecordParts {
        root_hash: RootHash::new([0x11; 32]),
        org_pub_key: org_pub_key(0x31),
        proxy_account: Some(ChainAccount::new([0x5a; 32])),
        org_private_key: OrgPrivateKey::from([0x5d; 32]),
        kept_change_set: Some(vec![7, 7]),
        kept_base_snapshot: Some(vec![8]),
        ..record_parts(1, &[bob()])
    })
}

fn provisional_organisation() -> ProvisionalOrganisation {
    ProvisionalOrganisation::from(ProvisionalOrganisationParts {
        epoch: Epoch::new(2),
        base_root: RootHash::new([0x11; 32]),
        members: vec![bob()],
        root: RootHash::new([0x66; 32]),
        org_pub_key: org_pub_key(0x33),
        org_private_key: OrgPrivateKey::from([0x88; 32]),
        change_set: Some(vec![1, 2, 3]),
        base_snapshot: Some(vec![4, 5]),
        origin: Origin::Own,
        applied: vec![ProvisionalChange::Revoke { member_id: MemberId::new([1; 32]) }],
        first_ahead_at: None,
    })
}

fn founding() -> ProvisionalFounding {
    ProvisionalFounding::from(ProvisionalFoundingParts {
        epoch: Epoch::new(1),
        members: vec![bob()],
        root: RootHash::new([0x67; 32]),
        org_pub_key: org_pub_key(0x34),
        org_private_key: OrgPrivateKey::from([0x89; 32]),
        proxy: None,
    })
}

fn provisional_member() -> ProvisionalMember {
    ProvisionalMember {
        org_id: the_org(),
        invite_id: InviteId::new([0x62; 32]),
        issuer: device_key(0x23),
        issuer_addresses: addresses(&["192.0.2.1:7000"]),
        name: Name::parse("Alice").unwrap(),
        surname: Surname::parse("Smith").unwrap(),
        preferred_handles: vec![handle("alice")],
        member_key: member_key(0x35),
        device_key: device_key(DEVICE),
    }
}

/// A store holding a value in every field of LLR-d925uu.
fn full_store() -> StoreData {
    StoreData {
        persona: persona(Some(handle("alice"))),
        record: Some(full_record()),
        org_name: "Acme".into(),
        provisional_changes: vec![
            ProvisionalChange::Admit { invite_id: InviteId::new([0x61; 32]), handle: handle("carol") },
            ProvisionalChange::Revoke { member_id: MemberId::new([1; 32]) },
        ],
        provisional_organisations: vec![provisional_organisation()],
        founding: Some(founding()),
        member: Some(provisional_member()),
        kept_admission_requests: vec![kept_request(InviteId::new([0x61; 32]), 0x36, 0x24, "carol")],
        kept_revocation_notices: vec![bob_absence_notice(the_org())],
        expected_admissions: vec![ExpectedAdmission { org_id: the_org() }],
        outstanding_invites: vec![OutstandingInvite {
            org_id: the_org(),
            invite_id: InviteId::new([0x61; 32]),
            invitee_name: "Carol Coder".into(),
        }],
        revoked_devices: vec![RevokedDevice { member_id: MemberId::new([3; 32]), device_key: device_key(0x25) }],
        address_book: vec![(device_key(0x22), addresses(&["203.0.113.9:7000"]))],
        behind: Some(Epoch::new(9)),
    }
}

/// `store_data` sealed as an operation seals it, and opened again with its secret.
fn round_trip(store_data: StoreData) -> StoreData {
    let store = store_for_test(&device_secret([DEVICE; 32]), store_data);
    let write = seal_for_test(&store, &mut OsRng);
    PersonaStore::unseal(&write.sealed, write.device_key, device_secret([DEVICE; 32])).unwrap().data().unwrap().clone()
}

// Normal: the store's encoding is its thirteen encoded fields, in LLR-d925uu's
// order, each encoded as its own type encodes; sealed and opened again it is
// the same value in every encoded field. The behind marker is not encoded:
// set, it changes no byte, and it opens `None`.
// verifies: LLR-d925uu
#[test]
fn store_data_holds_its_fields_in_encoding_order_and_round_trips_through_the_sealed_store() {
    let store_data = full_store();
    let in_order = postcard::to_allocvec(&(
        (&store_data.persona, &store_data.record, &store_data.org_name, &store_data.provisional_changes, &store_data.provisional_organisations),
        (&store_data.founding, &store_data.member, &store_data.kept_admission_requests, &store_data.kept_revocation_notices),
        (&store_data.expected_admissions, &store_data.outstanding_invites, &store_data.revoked_devices, &store_data.address_book),
    ))
    .unwrap();
    let encoded = postcard::to_allocvec(&store_data).unwrap();
    assert_eq!(encoded, in_order, "the fields in LLR-d925uu's order and nothing else");
    let unmarked = StoreData { behind: None, ..store_data.clone() };
    assert_eq!(postcard::to_allocvec(&unmarked).unwrap(), encoded, "the behind marker is not encoded");

    let reopened = round_trip(store_data);
    assert_eq!(postcard::to_allocvec(&reopened).unwrap(), encoded, "every encoded field reads back unchanged");
    assert_eq!(reopened.behind, None, "the behind marker opens None");
    assert_eq!(reopened.record.as_ref().unwrap().org_private_key_for_test(), &OrgPrivateKey::from([0x5d; 32]));
    assert_eq!(reopened.founding.as_ref().unwrap().org_private_key_for_test(), &OrgPrivateKey::from([0x89; 32]));
    assert_eq!(
        reopened.provisional_organisations[0].org_private_key_for_test(),
        &OrgPrivateKey::from([0x88; 32])
    );
}

// Abnormal: the store holds one record and its epoch state: its source
// names no provisional-update field or type and no `orgs` list of records.
// verifies: LLR-d925uu
#[test]
fn store_data_holds_no_provisional_updates_field() {
    let source = include_str!("../src/store.rs");
    for absent in ["provisional_updates", "ProvisionalUpdate", "pub orgs:", "insert_provisional", "forget_organisation"] {
        assert!(!source.contains(absent), "store.rs names `{absent}`");
    }
    let store_data = StoreData::new(persona(None));
    assert!(store_data.record.is_none() && store_data.provisional_changes.is_empty() && store_data.provisional_organisations.is_empty());
}

// Normal: an outstanding invite keeps the invitee name beside its pair, and
// reopens with it; abnormal: two invites differing only in that name are
// two values.
// verifies: LLR-d925uu
#[test]
fn an_outstanding_invite_keeps_its_invitee_name() {
    let invite = |name: &str| OutstandingInvite { org_id: the_org(), invite_id: InviteId::new([0x61; 32]), invitee_name: name.into() };
    let store_data = StoreData { outstanding_invites: vec![invite("Carol Coder")], ..StoreData::new(persona(None)) };
    let reopened = round_trip(store_data);
    assert_eq!(reopened.outstanding_invites, vec![invite("Carol Coder")]);
    assert_eq!(
        (reopened.outstanding_invites[0].org_id, reopened.outstanding_invites[0].invite_id),
        (the_org(), InviteId::new([0x61; 32]))
    );
    assert_ne!(invite("Carol Coder"), invite("Dan Driver"));
}

// Normal: a Persona record and a provisional Member hold their values as the
// parsed types (a provisional Member's Persona holds no Handle; its
// preferred Handles are `Handle`s); both reopen unchanged. Abnormal:
// org-node's source names no `Joiner` type: a provisional Member is the one
// name for that concept.
// verifies: LLR-pv2rdy
#[test]
fn a_persona_record_is_built_from_parsed_types_and_a_provisional_member_holds_no_handle() {
    let member_persona = persona(None);
    let name: &Name = &member_persona.name;
    let surname: &Surname = &member_persona.surname;
    assert_eq!((name.as_str(), surname.as_str(), member_persona.handle.clone()), ("Alice", "Smith", None));
    let store_data = StoreData { member: Some(provisional_member()), ..StoreData::new(member_persona) };
    let reopened = round_trip(store_data);
    assert_eq!(reopened.persona.handle, None, "a provisional Member's Persona holds no Handle");
    let member = reopened.member.as_ref().unwrap();
    assert_eq!(member, &provisional_member());
    let preferred: &Vec<Handle> = &member.preferred_handles;
    assert_eq!(preferred, &vec![handle("alice")]);
    let founder = round_trip(StoreData::new(persona(Some(handle("alice")))));
    assert_eq!(founder.persona.handle, Some(handle("alice")), "a founder's Persona holds its Handle");
    for source in [include_str!("../src/service.rs"), include_str!("../src/store.rs"), include_str!("../src/lib.rs")] {
        assert!(!source.contains("Joiner"), "org-node names a Joiner");
    }
}

// Normal: zero, one and sixteen entries, each at most 256 bytes of UTF-8,
// parse and keep their order and content.
// verifies: LLR-h4gwns
#[test]
fn endpoint_addresses_accept_zero_to_sixteen_entries_of_at_most_256_bytes() {
    assert!(EndpointAddresses::parse(vec![]).unwrap().as_slice().is_empty());
    let one_entry = EndpointAddresses::parse(vec!["192.0.2.1:7000".into()]).unwrap();
    assert_eq!(one_entry.as_slice(), ["192.0.2.1:7000".to_string()].as_slice());
    let sixteen: Vec<String> = (0..16).map(|index| format!("{index}")).collect();
    assert_eq!(EndpointAddresses::parse(sixteen.clone()).unwrap().as_slice(), sixteen.as_slice());
    let longest = "é".repeat(128);
    assert_eq!(longest.len(), 256);
    assert!(EndpointAddresses::parse(vec![longest]).is_ok(), "256 bytes is within the bound");
}

// Abnormal: seventeen entries, or one entry of 257 bytes, are refused, each
// with its own reason.
// verifies: LLR-h4gwns
#[test]
fn endpoint_addresses_refuse_seventeen_entries_or_one_over_256_bytes() {
    assert_eq!(EndpointAddresses::parse(vec![String::new(); 17]), Err(EndpointAddressesError::TooMany { count: 17 }));
    let over = "a".repeat(257);
    assert_eq!(
        EndpointAddresses::parse(vec!["ok".into(), over]),
        Err(EndpointAddressesError::TooLong { index: 1, length: 257 })
    );
    assert_ne!(
        EndpointAddressesError::TooMany { count: 17 }.to_string(),
        EndpointAddressesError::TooLong { index: 0, length: 257 }.to_string()
    );
}
