#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The Persona's records: every value held parsed (LLR-pv2rdy); a Persona
//! store or record snapshot holding a value its type's parse refuses is
//! refused whole, naming the field (LLR-7yu2xy); the current record always
//! carries its Organisation private key (LLR-78jfrd).
//!
//! A store is one Persona's view of one Organisation (LLR-f5wrjx), sealed
//! under a key derived from its device secret and handed out as bytes; "a
//! store file" below is those sealed bytes, opened with
//! `PersonaStore::unseal`, and the wire form below is in that layout. The
//! parse of a new Persona's details is `propose_founding`'s and
//! `propose_member`'s (LLR-rjphn4).

mod support;

use org_members::{Handle, MemberId, Name, RootHash, Surname};
use org_node::ids::OrgId;
use org_node::invitation::{EndpointAddresses, InviteId};
use org_node::revocation::{RevocationNotice, Signature64};
use org_node::service::first_admission_base;
use org_node::store::{
    ExpectedAdmission, KeptNotice, MemberSnapshot, OrgRecord, OrgRecordParts, Origin, PersonaRecord, PersonaRecordParts, PersonaStatus, PersonaStore, ProvisionalChange,
    ProvisionalMember, SealedStore, StoreData,
};
use org_node::test_fixtures::{bob_absence_notice, device_key, device_secret, member_key, seal_plaintext_for_test};
use org_node::{
    ChainAccount, Epoch, MemberSeed, OrgNodeError, OrgPrivateKey, OrgPublicKey, PersonaId, SequenceNumber,
};
use rand::rngs::OsRng;
use support::fixtures::{persona_parts, record_parts, snapshot};

/// The device seed byte of the Persona every store in this file holds.
const DEVICE: u8 = 0x12;

/// `plaintext`, the encoded store data, sealed in the Active form (index 0,
/// LLR-6zaeb6) as the store of the Persona whose device seed is
/// `[DEVICE; 32]`.
fn seal(plaintext: &[u8]) -> SealedStore {
    let mut active = vec![0u8];
    active.extend_from_slice(plaintext);
    seal_plaintext_for_test(&device_secret([DEVICE; 32]), device_key(DEVICE), &active, &mut OsRng)
}

/// Open `sealed` as that Persona's store.
fn open(sealed: &SealedStore) -> Result<PersonaStore, OrgNodeError> {
    PersonaStore::unseal(sealed, device_key(DEVICE), device_secret([DEVICE; 32]))
}

/// The key every key field refuses: u = y = 0, of small order under both
/// rules (`person`'s DevicePublicKey and X25519 checks). The Member-as-a-group
/// key and the Organisation public key are X25519 keys, for which u = 2 is a
/// valid (twist) point, so y = 2 would not do.
fn off_curve_key() -> [u8; 32] {
    [0u8; 32]
}

/// The only construction site of a `PersonaRecord` in this file: a founder
/// with a Handle.
fn persona(handle: &str) -> PersonaRecord {
    PersonaRecord::from(PersonaRecordParts {
        handle: Some(Handle::parse(handle).unwrap()),
        ..persona_parts("p1", [0x11; 32], device_key(DEVICE))
    })
}

/// An Organisation record with one member, whose Member key (seed 0x21) is
/// held nowhere else in the record.
fn org_with_member() -> OrgRecord {
    let bob = MemberSnapshot {
        name: Name::parse("Bob").unwrap(),
        surname: Surname::parse("Jones").unwrap(),
        ..snapshot(1, "bob", 0x21, &[0x22])
    };
    OrgRecord::from(OrgRecordParts {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: OrgPublicKey::parse(member_key(0x31).as_bytes()).unwrap(),
        last_seq: SequenceNumber::new(0),
        org_private_key: OrgPrivateKey::from([0x5d; 32]),
        ..record_parts(1, &[bob])
    })
}

/// A store holding `persona("alice")` and `org_with_member()` as its current
/// record, and nothing else.
fn store_with_record() -> StoreData {
    StoreData { record: Some(org_with_member()), ..StoreData::new(persona("alice")) }
}

/// Replaces the one occurrence of `from` in `bytes` with `to` (same length).
fn replace_once(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    let offsets: Vec<usize> =
        bytes.windows(from.len()).enumerate().filter(|(_, window)| *window == from).map(|(offset, _)| offset).collect();
    assert_eq!(offsets.len(), 1, "the pattern must occur exactly once");
    bytes[offsets[0]..offsets[0] + to.len()].copy_from_slice(to);
}

/// A sealed store holding `store_data` with `from` replaced by `to` in its plaintext.
fn sealed_store(store_data: &StoreData, from: &[u8], to: &[u8]) -> SealedStore {
    let mut plaintext = postcard::to_allocvec(store_data).unwrap();
    replace_once(&mut plaintext, from, to);
    seal(&plaintext)
}

/// A founder Persona proposed with details given in decomposed (non-NFC)
/// form holds them parsed, in NFC, as typed values, and its store reopens
/// with the same values; the provisional founding beside it reopens too.
/// The Persona is `propose_founding`'s and the store reopened is the write it
/// returned.
/// verifies: LLR-pv2rdy
#[test]
fn a_proposed_founding_persona_holds_the_parsed_details_across_a_reopen() {
    let mut svc = org_node::OrgService::new(vec![]).unwrap();
    let (device, secret, change) =
        svc.propose_founding(&mut OsRng, "Jose\u{0301}", "Nun\u{0303}ez", "jose\u{0301}", "Acme".into()).unwrap();
    let org_node::StoreChange::Sealed(write) = change;
    let reopened = PersonaStore::unseal(&write.sealed, device, secret).unwrap();
    let persona = &reopened.data().unwrap().persona;
    assert_eq!(persona.device_key, device);
    assert_eq!(persona.handle.as_ref().map(Handle::as_str), Some("jos\u{e9}"), "stored in the NFC form Handle::parse produces");
    assert_eq!((persona.name.as_str(), persona.surname.as_str()), ("Jos\u{e9}", "Nu\u{f1}ez"));
    assert_eq!((persona.status, persona.org_id, persona.member_id), (PersonaStatus::Proposed, None, None));
    assert_eq!(reopened.data().unwrap().org_name, "Acme");
    let founding = reopened.data().unwrap().founding.as_ref().expect("the provisional founding");
    assert_eq!((founding.epoch, founding.proxy), (Epoch::new(1), None));
    assert_eq!(founding.members[0].handle.as_str(), "jos\u{e9}", "the founding record lists the parsed Handle");
    assert!(reopened.data().unwrap().record.is_none(), "a proposed founding holds no current record");
}

/// The typed record holds a `Handle`, so its decoding refuses an invalid
/// one: there is no route from bytes to a `PersonaRecord` that skips the
/// parse. The bytes are a sealed store's, opened with `PersonaStore::unseal`.
/// verifies: LLR-pv2rdy
#[test]
fn a_persona_record_decoded_directly_refuses_an_invalid_handle() {
    let store_data = StoreData::new(persona("alice"));
    assert!(open(&sealed_store(&store_data, b"alice", b"alice")).is_ok());
    let refused = open(&sealed_store(&store_data, b"alice", b"Alice")).map(|_| ());
    assert!(matches!(refused, Err(OrgNodeError::InvalidField { .. })), "{refused:?}");
}

/// verifies: LLR-7yu2xy
#[test]
fn a_store_of_valid_records_opens_with_every_field_parsed() {
    let store_data = store_with_record();
    let opened = open(&sealed_store(&store_data, b"alice", b"alice")).unwrap();
    assert_eq!(opened.data().unwrap().persona.handle.as_ref().map(Handle::as_str), Some("alice"));
    assert_eq!(opened.data().unwrap().record.as_ref().unwrap().trie_members[0].member_key, member_key(0x21));
}

/// The store plaintext as its wire form (LLR-f5wrjx's field order), every
/// fallible field held as the plain value it is decoded from, so a test can
/// make any one of them invalid (postcard encodes a struct as the tuple of
/// its fields; names are not encoded).
#[derive(serde::Serialize)]
struct WireStore {
    persona: WirePersona,
    record: Option<WireOrg>,
    org_name: String,
    provisional_changes: Vec<WireChange>,
    provisional_organisations: Vec<WireProvisionalOrganisation>,
    founding: Option<WireFounding>,
    member: Option<WireProvisionalMember>,
    kept_admission_requests: Vec<WireKeptRequest>,
    kept_revocation_notices: Vec<(RevocationNotice, [u8; 32])>,
    expected_admissions: Vec<[u8; 20]>,
    outstanding_invites: Vec<([u8; 20], [u8; 32], String)>,
    revoked_devices: Vec<(MemberId, [u8; 32])>,
    address_book: Vec<([u8; 32], Vec<String>)>,
    acknowledged: Vec<(WireAcknowledgement, MemberId)>,
}

/// An acknowledgement's fields in order, its Device key as plain bytes.
type WireAcknowledgement = (OrgId, MemberId, [u8; 32], Epoch, RootHash, Signature64);

#[derive(serde::Serialize)]
struct WirePersona {
    persona_id: PersonaId,
    org_id: Option<OrgId>,
    name: String,
    surname: String,
    handle: Option<String>,
    member_seed: MemberSeed,
    device_key: [u8; 32],
    member_id: Option<MemberId>,
    status: PersonaStatus,
}

#[derive(serde::Serialize)]
struct WireOrg {
    org_id: OrgId,
    root_hash: RootHash,
    org_pub_key: [u8; 32],
    epoch: Epoch,
    last_seq: SequenceNumber,
    trie_members: Vec<WireMember>,
    proxy_account: Option<ChainAccount>,
    org_private_key: OrgPrivateKey,
    kept_change_set: Option<Vec<u8>>,
    kept_base_snapshot: Option<Vec<u8>>,
}

#[derive(serde::Serialize)]
struct WireMember {
    id: MemberId,
    handle: String,
    name: String,
    surname: String,
    member_key: [u8; 32],
    device_keys: Vec<[u8; 32]>,
}

#[derive(serde::Serialize)]
enum WireChange {
    Admit { invite_id: [u8; 32], handle: String },
    Revoke { member_id: MemberId },
}

#[derive(serde::Serialize)]
enum WireOrigin {
    #[allow(dead_code, reason = "never built: it holds variant index 0, so `Received` encodes as 1")]
    Own,
    Received { from: [u8; 32] },
}

#[derive(serde::Serialize)]
struct WireProvisionalOrganisation {
    epoch: Epoch,
    base_root: RootHash,
    members: Vec<WireMember>,
    root: RootHash,
    org_pub_key: [u8; 32],
    org_private_key: OrgPrivateKey,
    change_set: Option<Vec<u8>>,
    base_snapshot: Option<Vec<u8>>,
    origin: WireOrigin,
    applied: Vec<WireChange>,
    first_ahead_at: Option<u64>,
}

#[derive(serde::Serialize)]
struct WireFounding {
    epoch: Epoch,
    members: Vec<WireMember>,
    root: RootHash,
    org_pub_key: [u8; 32],
    org_private_key: OrgPrivateKey,
    proxy: Option<ChainAccount>,
}

#[derive(serde::Serialize)]
struct WireProvisionalMember {
    org_id: OrgId,
    invite_id: [u8; 32],
    issuer: [u8; 32],
    issuer_addresses: Vec<String>,
    name: String,
    surname: String,
    preferred_handles: Vec<String>,
    member_key: [u8; 32],
    device_key: [u8; 32],
}

#[derive(serde::Serialize)]
struct WireKeptRequest {
    org_id: OrgId,
    invite_id: [u8; 32],
    from: [u8; 32],
    name: String,
    surname: String,
    preferred_handles: Vec<String>,
    member_key: [u8; 32],
    device_key: [u8; 32],
    addresses: Vec<String>,
}

fn wire_bob() -> WireMember {
    WireMember {
        id: MemberId::new([1u8; 32]),
        handle: "bob".into(),
        name: "Bob".into(),
        surname: "Jones".into(),
        member_key: *member_key(0x21).as_bytes(),
        device_keys: vec![*device_key(0x22).as_bytes()],
    }
}

/// A store in which every fallible field is valid and every key distinct,
/// holding one value of every epoch-state kind (LLR-f5wrjx).
fn wire_store() -> WireStore {
    WireStore {
        persona: WirePersona {
            persona_id: PersonaId::new("p1".to_string()),
            org_id: Some(OrgId::new([5u8; 20])),
            name: "Alice".into(),
            surname: "Smith".into(),
            handle: Some("alice".into()),
            member_seed: MemberSeed::from([0x11; 32]),
            device_key: *device_key(DEVICE).as_bytes(),
            member_id: None,
            status: PersonaStatus::Proposed,
        },
        record: Some(WireOrg {
            org_id: OrgId::new([5u8; 20]),
            root_hash: RootHash::new([0x11u8; 32]),
            org_pub_key: *member_key(0x32).as_bytes(),
            epoch: Epoch::new(1),
            last_seq: SequenceNumber::new(0),
            trie_members: vec![wire_bob()],
            proxy_account: None,
            org_private_key: OrgPrivateKey::from([0x5e; 32]),
            kept_change_set: None,
            kept_base_snapshot: None,
        }),
        org_name: "Acme".into(),
        provisional_changes: vec![
            WireChange::Admit { invite_id: [0x61; 32], handle: "carol".into() },
            WireChange::Revoke { member_id: MemberId::new([1u8; 32]) },
        ],
        provisional_organisations: vec![WireProvisionalOrganisation {
            epoch: Epoch::new(2),
            base_root: RootHash::new([0x11u8; 32]),
            members: vec![wire_bob()],
            root: RootHash::new([0x66; 32]),
            org_pub_key: *member_key(0x33).as_bytes(),
            org_private_key: OrgPrivateKey::from([0x88; 32]),
            change_set: Some(vec![1, 2, 3]),
            base_snapshot: Some(vec![4, 5]),
            origin: WireOrigin::Received { from: *device_key(0x22).as_bytes() },
            applied: vec![],
            first_ahead_at: None,
        }],
        founding: Some(WireFounding {
            epoch: Epoch::new(1),
            members: vec![wire_bob()],
            root: RootHash::new([0x67; 32]),
            org_pub_key: *member_key(0x34).as_bytes(),
            org_private_key: OrgPrivateKey::from([0x89; 32]),
            proxy: Some(ChainAccount::new([0x5a; 32])),
        }),
        member: Some(WireProvisionalMember {
            org_id: OrgId::new([5u8; 20]),
            invite_id: [0x62; 32],
            issuer: *device_key(0x23).as_bytes(),
            issuer_addresses: vec!["192.0.2.1:7000".into()],
            name: "Alice".into(),
            surname: "Smith".into(),
            preferred_handles: vec!["alice".into(), "ali".into()],
            member_key: *member_key(0x35).as_bytes(),
            device_key: *device_key(DEVICE).as_bytes(),
        }),
        kept_admission_requests: vec![WireKeptRequest {
            org_id: OrgId::new([5u8; 20]),
            invite_id: [0x61; 32],
            from: *device_key(0x24).as_bytes(),
            name: "Carol".into(),
            surname: "Coder".into(),
            preferred_handles: vec!["carol".into()],
            member_key: *member_key(0x36).as_bytes(),
            device_key: *device_key(0x24).as_bytes(),
            addresses: vec!["198.51.100.7:7000".into()],
        }],
        kept_revocation_notices: vec![(bob_absence_notice(OrgId::new([5u8; 20])), *device_key(0x21).as_bytes())],
        expected_admissions: vec![[0x77; 20]],
        outstanding_invites: vec![([5u8; 20], [0x61; 32], "Carol Coder".into())],
        revoked_devices: vec![(MemberId::new([3u8; 32]), *device_key(0x25).as_bytes())],
        address_book: vec![(*device_key(0x22).as_bytes(), vec!["203.0.113.9:7000".into()])],
        acknowledged: vec![(
            (OrgId::new([5u8; 20]), MemberId::new([3u8; 32]), *device_key(0x25).as_bytes(), Epoch::new(1), RootHash::new([0x66; 32]), Signature64([0x5b; 64])),
            MemberId::new([3u8; 32]),
        )],
    }
}

fn sealed_wire_store(store: &WireStore) -> SealedStore {
    seal(&postcard::to_allocvec(store).unwrap())
}

fn too_long() -> String {
    "a".repeat(129)
}

/// The wire form of a store with one value of every epoch-state kind opens
/// to exactly the values it holds, each parsed by its type, so the tables
/// below make one field invalid and nothing else.
/// verifies: LLR-7yu2xy
#[test]
fn a_store_with_every_epoch_state_kind_opens_with_every_field_parsed() {
    let opened = open(&sealed_wire_store(&wire_store())).unwrap();
    let content = opened.data().unwrap();
    assert_eq!((content.persona.name.as_str(), content.persona.surname.as_str()), ("Alice", "Smith"));
    assert_eq!(content.persona.handle.as_ref().map(Handle::as_str), Some("alice"));
    let record = content.record.as_ref().unwrap();
    assert_eq!(record.org_pub_key, OrgPublicKey::parse(member_key(0x32).as_bytes()).unwrap());
    let listed = &record.trie_members[0];
    assert_eq!((listed.handle.as_str(), listed.name.as_str(), listed.surname.as_str()), ("bob", "Bob", "Jones"));
    assert_eq!((listed.member_key, listed.device_keys.clone()), (member_key(0x21), vec![device_key(0x22)]));
    assert_eq!(content.org_name, "Acme");
    assert_eq!(
        content.provisional_changes,
        vec![
            ProvisionalChange::Admit { invite_id: InviteId::new([0x61; 32]), handle: Handle::parse("carol").unwrap() },
            ProvisionalChange::Revoke { member_id: MemberId::new([1u8; 32]) },
        ]
    );
    let kept = &content.provisional_organisations[0];
    assert_eq!(kept.org_pub_key, OrgPublicKey::parse(member_key(0x33).as_bytes()).unwrap());
    assert_eq!(kept.origin, Origin::Received { from: device_key(0x22) });
    assert_eq!((kept.change_set.clone(), kept.base_snapshot.clone()), (Some(vec![1, 2, 3]), Some(vec![4, 5])));
    let founding = content.founding.as_ref().unwrap();
    assert_eq!(founding.org_pub_key, OrgPublicKey::parse(member_key(0x34).as_bytes()).unwrap());
    assert_eq!(founding.proxy, Some(ChainAccount::new([0x5a; 32])));
    let member: &ProvisionalMember = content.member.as_ref().unwrap();
    assert_eq!(member.issuer, device_key(0x23));
    assert_eq!(member.issuer_addresses, EndpointAddresses::parse(vec!["192.0.2.1:7000".into()]).unwrap());
    assert_eq!(member.preferred_handles, vec![Handle::parse("alice").unwrap(), Handle::parse("ali").unwrap()]);
    let request = &content.kept_admission_requests[0];
    assert_eq!((request.from, request.member_key), (device_key(0x24), member_key(0x36)));
    assert_eq!(request.addresses.as_slice(), ["198.51.100.7:7000".to_string()].as_slice());
    assert_eq!(
        content.kept_revocation_notices,
        vec![KeptNotice { notice: bob_absence_notice(OrgId::new([5u8; 20])), from: device_key(0x21) }]
    );
    assert_eq!(content.expected_admissions, vec![ExpectedAdmission { org_id: OrgId::new([0x77; 20]) }]);
    assert_eq!(content.outstanding_invites[0].invitee_name, "Carol Coder");
    assert_eq!(content.revoked_devices[0].device_key, device_key(0x25));
    assert_eq!(content.address_book[0].0, device_key(0x22));
    assert_eq!(
        (content.acknowledged[0].acknowledgement.device, content.acknowledged[0].member_id),
        (device_key(0x25), MemberId::new([3u8; 32]))
    );
    assert_eq!(content.behind, None, "the behind marker is not encoded");
}

// LLR-78jfrd: the current record carries the Organisation private key,
// encoded as its plain 32 bytes with no option tag, and it is read back. A
// store whose record ends in an option tag and no key — a legacy layout — is
// refused, not read or migrated. The record is followed by its two kept
// options (`kept_change_set`, `kept_base_snapshot`: `00 00`) and the store
// by twelve empty or absent fields (the Organisation name, ten lists and
// the two options of LLR-f5wrjx), so the key is the 32 bytes ending 14
// bytes before the end.
// verifies: LLR-78jfrd
#[test]
fn a_record_carries_its_organisation_private_key_and_a_store_without_one_is_refused() {
    let store_data = StoreData { org_name: String::new(), ..store_with_record() };
    let plaintext = postcard::to_allocvec(&store_data).unwrap();
    let key = org_with_member().org_private_key_for_test().clone();
    assert_eq!(&plaintext[plaintext.len() - 46..plaintext.len() - 14], key.expose_secret(), "no option tag");
    assert_eq!(&plaintext[plaintext.len() - 14..], &[0u8; 14], "two kept options, then the empty epoch state");
    assert_eq!(open(&seal(&plaintext)).unwrap().data().unwrap().record.as_ref().unwrap().org_private_key_for_test(), &key);

    let mut legacy = plaintext[..plaintext.len() - 46].to_vec();
    legacy.extend_from_slice(&[0x00; 15]); // `org_private_key: None`, then the kept options and the empty epoch state
    assert!(open(&seal(&legacy)).is_err(), "a record without the key is not read");
}

/// Every field the store-open refusal names, each made invalid alone, is
/// refused naming exactly that field.
/// verifies: LLR-7yu2xy
#[test]
fn a_store_with_any_one_field_invalid_is_refused_naming_exactly_that_field() {
    type Spoil = fn(&mut WireStore);
    let cases: [(&str, Spoil); 9] = [
        ("persona.handle", |store| store.persona.handle = Some("Alice".into())),
        ("persona.name", |store| store.persona.name = too_long()),
        ("persona.surname", |store| store.persona.surname = too_long()),
        ("org.org_pub_key", |store| store.record.as_mut().unwrap().org_pub_key = off_curve_key()),
        ("member.handle", |store| store.record.as_mut().unwrap().trie_members[0].handle = "Bob".into()),
        ("member.name", |store| store.record.as_mut().unwrap().trie_members[0].name = too_long()),
        ("member.surname", |store| store.record.as_mut().unwrap().trie_members[0].surname = too_long()),
        ("member.member_key", |store| store.record.as_mut().unwrap().trie_members[0].member_key = off_curve_key()),
        ("member.device_keys", |store| store.record.as_mut().unwrap().trie_members[0].device_keys.push(off_curve_key())),
    ];
    for (field, spoil) in cases {
        let mut wire = wire_store();
        spoil(&mut wire);
        let sealed = sealed_wire_store(&wire);
        let before = sealed.as_bytes().to_vec();
        let err = open(&sealed).err().unwrap_or_else(|| panic!("{field}: must be refused"));
        assert!(matches!(&err, OrgNodeError::InvalidField { field: named, .. } if *named == field), "{field}: got {err:?}");
        assert!(err.to_string().contains(field), "the message names the field: {err}");
        assert_eq!(sealed.as_bytes(), before, "{field}: a refused store is left as it was");
    }
}

/// Two fields of one record invalid at once — the record's Organisation
/// public key and its member's Member key — reports the first in record
/// order, `org.org_pub_key`.
/// verifies: LLR-7yu2xy
#[test]
fn a_store_with_two_invalid_fields_in_one_record_reports_the_first_in_record_order() {
    let mut wire = wire_store();
    let record = wire.record.as_mut().unwrap();
    record.org_pub_key = off_curve_key();
    record.trie_members[0].member_key = off_curve_key();
    let err = open(&sealed_wire_store(&wire)).err().expect("must be refused");
    assert!(matches!(&err, OrgNodeError::InvalidField { field: "org.org_pub_key", .. }), "got {err:?}");
}

/// One invalid field in each epoch-state kind — a provisional change, a
/// provisional organisation (key and origin), the provisional founding, the
/// provisional Member, a kept admission request, a kept notice's sender, a
/// revoked Device, an address-book entry, an acknowledged entry and a member
/// snapshot inside a provisional organisation — is refused naming it, and
/// nothing is opened.
/// verifies: LLR-7yu2xy, LLR-f5wrjx
#[test]
fn unseal_refuses_an_invalid_field_of_each_epoch_state_kind_naming_it() {
    type Spoil = fn(&mut WireStore);
    let cases: [(&str, Spoil); 12] = [
        ("provisional.handle", |store| store.provisional_changes[0] = WireChange::Admit { invite_id: [0x61; 32], handle: "Carol".into() }),
        ("provisional.org_pub_key", |store| store.provisional_organisations[0].org_pub_key = off_curve_key()),
        ("provisional.origin", |store| store.provisional_organisations[0].origin = WireOrigin::Received { from: off_curve_key() }),
        ("member.handle", |store| store.provisional_organisations[0].members[0].handle = "Bob".into()),
        ("founding.org_pub_key", |store| store.founding.as_mut().unwrap().org_pub_key = off_curve_key()),
        ("provisional_member.issuer", |store| store.member.as_mut().unwrap().issuer = off_curve_key()),
        ("admission.member_key", |store| store.kept_admission_requests[0].member_key = off_curve_key()),
        ("admission.addresses", |store| store.kept_admission_requests[0].addresses = vec!["a".repeat(257)]),
        ("org.kept_revocation_notices", |store| store.kept_revocation_notices[0].1 = off_curve_key()),
        ("org.revoked_devices", |store| store.revoked_devices[0].1 = off_curve_key()),
        ("org.address_book", |store| store.address_book[0].1 = vec![String::new(); 17]),
        ("org.acknowledged", |store| store.acknowledged[0].0 .2 = off_curve_key()),
    ];
    for (field, spoil) in cases {
        let mut wire = wire_store();
        spoil(&mut wire);
        let err = open(&sealed_wire_store(&wire)).err().unwrap_or_else(|| panic!("{field}: must be refused"));
        assert!(matches!(&err, OrgNodeError::InvalidField { field: named, .. } if *named == field), "{field}: got {err:?}");
    }
}

/// A provisional Member whose preferred Handles number zero or four, or
/// repeat one, is refused as a whole naming the field; one, two and three
/// distinct Handles open.
/// verifies: LLR-7yu2xy
#[test]
fn unseal_refuses_a_provisional_member_with_zero_four_or_repeated_handles() {
    let with_handles = |handles: &[&str]| {
        let mut wire = wire_store();
        wire.member.as_mut().unwrap().preferred_handles = handles.iter().map(|handle| handle.to_string()).collect();
        open(&sealed_wire_store(&wire))
    };
    for handles in [&[][..], &["a", "b", "c", "d"][..], &["al", "al"][..]] {
        let err = with_handles(handles).err().unwrap_or_else(|| panic!("{handles:?}: must be refused"));
        assert!(
            matches!(&err, OrgNodeError::InvalidField { field: "provisional_member.preferred_handles", .. }),
            "{handles:?}: got {err:?}"
        );
    }
    for handles in [&["al"][..], &["al", "bo"][..], &["al", "bo", "cy"][..]] {
        assert_eq!(with_handles(handles).unwrap().data().unwrap().member.as_ref().unwrap().preferred_handles.len(), handles.len());
    }
}

// Abnormal: a stored provisional Member whose name, surname, Handle, keys or
// issuer addresses do not parse fails the open as a whole, naming the field.
// verifies: LLR-7yu2xy
#[test]
fn a_store_with_an_invalid_provisional_member_is_refused_naming_the_field() {
    type Spoil = fn(&mut WireProvisionalMember);
    let cases: [(&str, Spoil); 6] = [
        ("provisional_member.name", |member| member.name = too_long()),
        ("provisional_member.surname", |member| member.surname = too_long()),
        ("provisional_member.preferred_handles", |member| member.preferred_handles[1] = "Not A Handle".into()),
        ("provisional_member.member_key", |member| member.member_key = off_curve_key()),
        ("provisional_member.device_key", |member| member.device_key = off_curve_key()),
        ("provisional_member.issuer_addresses", |member| member.issuer_addresses = vec![String::new(); 17]),
    ];
    for (field, spoil) in cases {
        let mut wire = wire_store();
        spoil(wire.member.as_mut().unwrap());
        let err = open(&sealed_wire_store(&wire)).err().unwrap_or_else(|| panic!("{field}: must be refused"));
        assert!(matches!(&err, OrgNodeError::InvalidField { field: named, .. } if *named == field), "{field}: {err:?}");
    }
}

/// A record snapshot (what a first admission extends) as its wire form: a
/// list of member snapshots, each the tuple of its fields.
fn snapshot_bytes(handle: &str, name: &str, surname: &str, member_key: [u8; 32], device_keys: Vec<[u8; 32]>) -> Vec<u8> {
    postcard::to_allocvec(&vec![(MemberId::new([1u8; 32]), handle, name, surname, member_key, device_keys)]).unwrap()
}

/// verifies: LLR-7yu2xy, LLR-tcft2r
#[test]
fn a_valid_record_snapshot_decodes_with_every_field_parsed() {
    let bytes = snapshot_bytes("bob", "Bob", "Jones", *member_key(0x21).as_bytes(), vec![*device_key(0x22).as_bytes()]);
    let trie = first_admission_base(&bytes).unwrap();
    let leaf = trie.get(&MemberId::new([1u8; 32])).expect("the member is in the record");
    assert_eq!(leaf.handle().as_str(), "bob");
    assert_eq!(*leaf.p2p_key(), member_key(0x21));
    assert_eq!(leaf.p2p_devices(), [device_key(0x22)].as_slice());
}

/// verifies: LLR-7yu2xy
#[test]
fn a_record_snapshot_holding_an_invalid_value_is_refused_naming_the_field() {
    let mk = *member_key(0x21).as_bytes();
    let device_bytes = *device_key(0x22).as_bytes();
    let over = "a".repeat(129);
    let cases = [
        (snapshot_bytes("Bob", "Bob", "Jones", mk, vec![device_bytes]), "member.handle"),
        (snapshot_bytes("bob", &over, "Jones", mk, vec![device_bytes]), "member.name"),
        (snapshot_bytes("bob", "Bob", &over, mk, vec![device_bytes]), "member.surname"),
        (snapshot_bytes("bob", "Bob", "Jones", off_curve_key(), vec![device_bytes]), "member.member_key"),
        (snapshot_bytes("bob", "Bob", "Jones", mk, vec![device_bytes, off_curve_key()]), "member.device_keys"),
    ];
    for (bytes, field) in cases {
        let err = first_admission_base(&bytes).expect_err("must be refused");
        assert!(matches!(&err, OrgNodeError::InvalidField { field: named, .. } if *named == field), "got {err:?}");
        assert!(err.to_string().contains(field), "the message names the field: {err}");
    }
}

/// An invalid handle and an invalid Member key in one member snapshot report
/// the handle: fields are parsed in record order.
/// verifies: LLR-7yu2xy
#[test]
fn a_record_snapshot_with_an_invalid_handle_and_member_key_reports_the_handle() {
    let bytes = snapshot_bytes("Bob", "Bob", "Jones", off_curve_key(), vec![*device_key(0x22).as_bytes()]);
    let err = first_admission_base(&bytes).expect_err("must be refused");
    assert!(matches!(&err, OrgNodeError::InvalidField { field: "member.handle", .. }), "got {err:?}");
}

// Abnormal: bytes that are not an encoded record snapshot are refused with a
// typed error, not a panic, and extend nothing.
// verifies: LLR-tcft2r, REQ-9g6as6
#[test]
fn bytes_that_are_not_a_record_snapshot_are_refused() {
    for bytes in [&[][..], &[0xff; 8][..], &[0x05, 0x00][..]] {
        assert!(org_node::service::first_admission_base(bytes).is_err(), "{bytes:?}");
    }
}
