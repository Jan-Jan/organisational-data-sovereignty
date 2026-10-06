#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The Persona's records and the Join request: Persona details are held
//! parsed and `create_persona` takes them typed (LLR-g76zqd); a Persona store
//! or Join request holding a value its type's parse refuses is refused whole,
//! naming the field (LLR-8bum44). Both requirements sit under SDD-af5vnt;
//! SDD-vee2fq, which owns the Join request's code in `blobs.rs`, names them.

use std::path::PathBuf;

use org_members::{Handle, MemberId, Name, RootHash, Surname};
use org_node::blobs;
use org_node::service::first_admission_base;
use org_node::ids::OrgId;
use org_node::store::{
    self, MemberSnapshot, OrgRecord, PersonaDetails, PersonaRecord, PersonaStatus, PersonaStore, StoreData,
};
use org_node::test_fixtures::{device_key, member_key};
use org_node::{
    ChainAccount, DeviceSeed, Epoch, MemberSeed, MockChainOps, OrgNodeError, OrgPrivateKey, OrgPublicKey, OrgSecret, OrgService,
    PersonaId, SequenceNumber,
};
use rand::rngs::OsRng;

fn tmp_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ods-persona-records-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.bin"));
    let _ = std::fs::remove_file(&path);
    path
}

/// Adapted at the merge of master `1feb608` into worktree-person-shared-types,
/// which kept master's name: master used y = 2, off the Edwards curve. On this
/// branch the Member-as-a-group key and the Organisation public key are X25519
/// keys, for which u = 2 is a valid (twist) point, so the key every field
/// refuses is u = y = 0: of small order under both rules (`person`'s
/// DevicePublicKey and X25519 checks).
fn off_curve_key() -> [u8; 32] {
    [0u8; 32]
}

/// The only construction site of a `PersonaRecord` in this file.
fn persona(handle: &str) -> PersonaRecord {
    PersonaRecord {
        persona_id: PersonaId::new("p1".to_string()),
        org_id: None,
        handle: Handle::parse(handle).unwrap(),
        name: Name::parse("Alice").unwrap(),
        surname: Surname::parse("Smith").unwrap(),
        member_seed: MemberSeed::from([0x11; 32]),
        device_seed: DeviceSeed::from([0x12; 32]),
        member_id: None,
        status: PersonaStatus::Proposed,
    }
}

/// An Organisation record with one member, whose Member key (seed 0x21) is
/// held nowhere else in the record.
fn org_with_member() -> OrgRecord {
    OrgRecord {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: OrgPublicKey::parse(member_key(0x31).as_bytes()).unwrap(),
        epoch: Epoch::new(1),
        org_secret: None,
        last_seq: SequenceNumber::new(0),
        admin_member_key: member_key(0x31),
        trie_members: vec![MemberSnapshot {
            id: MemberId::new([1u8; 32]),
            handle: Handle::parse("bob").unwrap(),
            name: Name::parse("Bob").unwrap(),
            surname: Surname::parse("Jones").unwrap(),
            member_key: member_key(0x21),
            device_keys: vec![device_key(0x22)],
        }],
        proxy_account: None,
        org_private_key: None,
    }
}

/// Replaces the one occurrence of `from` in `bytes` with `to` (same length).
fn replace_once(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    let at: Vec<usize> = bytes.windows(from.len()).enumerate().filter(|(_, w)| *w == from).map(|(i, _)| i).collect();
    assert_eq!(at.len(), 1, "the pattern must occur exactly once");
    bytes[at[0]..at[0] + to.len()].copy_from_slice(to);
}

/// A store file holding `data` with `from` replaced by `to` in its plaintext.
fn sealed_store(name: &str, data: &StoreData, from: &[u8], to: &[u8]) -> PathBuf {
    let path = tmp_path(name);
    let mut plaintext = postcard::to_allocvec(data).unwrap();
    replace_once(&mut plaintext, from, to);
    store::seal_for_test(&path, "pw", &plaintext, &mut OsRng).unwrap();
    path
}

/// verifies: LLR-g76zqd
#[test]
fn create_persona_holds_the_parsed_details_across_a_reopen() {
    let path = tmp_path("create");
    let mut svc = OrgService::new(PersonaStore::open(path.clone(), "pw").unwrap(), Box::new(MockChainOps::new()));
    let pid = svc
        .create_persona(
            &mut OsRng,
            Handle::parse("jose\u{0301}").unwrap(),
            Name::parse("Jose\u{0301}").unwrap(),
            Surname::parse("Smith").unwrap(),
        )
        .unwrap();
    let reopened = PersonaStore::open(path, "pw").unwrap();
    let p = &reopened.data().personas[0];
    assert_eq!(p.persona_id, pid);
    assert_eq!(p.handle.as_str(), "jos\u{e9}", "stored in the NFC form Handle::parse produces");
    assert_eq!(p.name.as_str(), "Jos\u{e9}");
    let jr = OrgService::import_join_request(&svc.export_join_request(&pid).unwrap()).unwrap();
    assert_eq!(jr.handle, p.handle);
    assert_eq!(jr.name, p.name);
    assert_eq!(jr.surname, p.surname);
}

/// The typed record holds a `Handle`, so its own decoding refuses an invalid
/// one: there is no route from bytes to a `PersonaRecord` that skips the parse.
/// verifies: LLR-g76zqd
#[test]
fn a_persona_record_decoded_directly_refuses_an_invalid_handle() {
    let mut bytes = postcard::to_allocvec(&persona("alice")).unwrap();
    assert!(postcard::from_bytes::<PersonaRecord>(&bytes).is_ok());
    replace_once(&mut bytes, b"alice", b"Alice");
    assert!(postcard::from_bytes::<PersonaRecord>(&bytes).is_err());
}

/// verifies: LLR-q6n25z
#[test]
fn persona_details_parse_into_their_types_and_create_a_persona() {
    let path = tmp_path("details-create");
    let mut svc = OrgService::new(PersonaStore::open(path, "pw").unwrap(), Box::new(MockChainOps::new()));
    let details = PersonaDetails::parse("jose\u{0301}", "Jose\u{0301}", "Smith").unwrap();
    assert_eq!(details.handle.as_str(), "jos\u{e9}");
    assert_eq!(details.name.as_str(), "Jos\u{e9}");
    assert_eq!(details.surname.as_str(), "Smith");
    let PersonaDetails { handle, name, surname } = details;
    svc.create_persona(&mut OsRng, handle, name, surname).unwrap();
    assert_eq!(svc.list_personas()[0].handle.as_str(), "jos\u{e9}");
}

/// verifies: LLR-q6n25z
#[test]
fn persona_details_refuse_each_invalid_field_naming_it() {
    let over = "a".repeat(129);
    let cases = [
        ("Alice", "Alice", "Smith", "persona.handle"),
        ("alice", over.as_str(), "Smith", "persona.name"),
        ("alice", "Alice", over.as_str(), "persona.surname"),
    ];
    for (handle, name, surname, field) in cases {
        let err = PersonaDetails::parse(handle, name, surname).expect_err("must be refused");
        assert!(matches!(&err, OrgNodeError::InvalidField { field: f, .. } if *f == field), "got {err:?}");
        assert!(err.to_string().contains(field), "the message names the field: {err}");
    }
}

/// Details given in a decomposed (non-NFC) form are held, saved, reopened and
/// exported in the Join request in NFC: the Join request blob is decoded as
/// its plain wire tuple, so no parse on the reading side normalises it.
/// verifies: LLR-q6n25z
#[test]
fn non_nfc_persona_details_are_stored_and_exported_in_nfc() {
    let path = tmp_path("details-nfc");
    let mut svc = OrgService::new(PersonaStore::open(path.clone(), "pw").unwrap(), Box::new(MockChainOps::new()));
    let PersonaDetails { handle, name, surname } =
        PersonaDetails::parse("jose\u{0301}", "Jose\u{0301}", "Nun\u{0303}ez").unwrap();
    let pid = svc.create_persona(&mut OsRng, handle, name, surname).unwrap();
    let nfc = ("jos\u{e9}", "Jos\u{e9}", "Nu\u{f1}ez");
    let held = &svc.list_personas()[0];
    assert_eq!((held.handle.as_str(), held.name.as_str(), held.surname.as_str()), nfc, "held in NFC");
    let reopened = PersonaStore::open(path, "pw").unwrap();
    let p = &reopened.data().personas[0];
    assert_eq!((p.handle.as_str(), p.name.as_str(), p.surname.as_str()), nfc, "stored in NFC");
    type WireJoinRequest = (String, String, String, [u8; 32], [u8; 32], Vec<u8>);
    let (h, n, s, ..) = blobs::decode::<WireJoinRequest>(&svc.export_join_request(&pid).unwrap()).unwrap();
    assert_eq!((h.as_str(), n.as_str(), s.as_str()), nfc, "exported in NFC");
}

/// Two details invalid at once: the first in parse order is reported.
/// verifies: LLR-q6n25z
#[test]
fn persona_details_with_two_invalid_fields_report_the_first() {
    let over = "a".repeat(129);
    let cases = [
        ("Alice", over.as_str(), "Smith", "persona.handle"),
        ("alice", over.as_str(), over.as_str(), "persona.name"),
    ];
    for (handle, name, surname, field) in cases {
        let err = PersonaDetails::parse(handle, name, surname).expect_err("must be refused");
        assert!(matches!(&err, OrgNodeError::InvalidField { field: f, .. } if *f == field), "got {err:?}");
    }
}

/// verifies: LLR-8bum44
#[test]
fn a_store_of_valid_records_opens_with_every_field_parsed() {
    let data = StoreData { personas: vec![persona("alice")], orgs: vec![org_with_member()], pending_invites: vec![] };
    let path = sealed_store("valid", &data, b"alice", b"alice");
    let opened = PersonaStore::open(path, "pw").unwrap();
    assert_eq!(opened.data().personas[0].handle.as_str(), "alice");
    assert_eq!(opened.data().orgs[0].trie_members[0].member_key, member_key(0x21));
}

/// The store plaintext as its wire form, every fallible field held as the
/// plain value it is decoded from, so a test can make any one of them invalid
/// (postcard encodes a struct as the tuple of its fields; names are not
/// encoded).
#[derive(serde::Serialize)]
struct WireStore {
    personas: Vec<WirePersona>,
    orgs: Vec<WireOrg>,
    pending_invites: Vec<WireInvite>,
}

#[derive(serde::Serialize)]
struct WirePersona {
    persona_id: PersonaId,
    org_id: Option<OrgId>,
    handle: String,
    name: String,
    surname: String,
    member_seed: MemberSeed,
    device_seed: DeviceSeed,
    member_id: Option<MemberId>,
    status: PersonaStatus,
}

#[derive(serde::Serialize)]
struct WireOrg {
    org_id: OrgId,
    root_hash: RootHash,
    org_pub_key: [u8; 32],
    epoch: Epoch,
    org_secret: Option<OrgSecret>,
    last_seq: SequenceNumber,
    admin_member_key: [u8; 32],
    trie_members: Vec<WireMember>,
    proxy_account: Option<ChainAccount>,
    org_private_key: Option<OrgPrivateKey>,
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
struct WireInvite {
    org_id: OrgId,
    admin_device_key: [u8; 32],
    admin_member_key: [u8; 32],
    org_pub_key: [u8; 32],
}

/// A store in which every fallible field is valid and every key distinct.
fn wire_store() -> WireStore {
    WireStore {
        personas: vec![WirePersona {
            persona_id: PersonaId::new("p1".to_string()),
            org_id: None,
            handle: "alice".into(),
            name: "Alice".into(),
            surname: "Smith".into(),
            member_seed: MemberSeed::from([0x11; 32]),
            device_seed: DeviceSeed::from([0x12; 32]),
            member_id: None,
            status: PersonaStatus::Proposed,
        }],
        orgs: vec![WireOrg {
            org_id: OrgId::new([5u8; 20]),
            root_hash: RootHash::new([0x11u8; 32]),
            org_pub_key: *member_key(0x32).as_bytes(),
            epoch: Epoch::new(1),
            org_secret: None,
            last_seq: SequenceNumber::new(0),
            admin_member_key: *member_key(0x31).as_bytes(),
            trie_members: vec![WireMember {
                id: MemberId::new([1u8; 32]),
                handle: "bob".into(),
                name: "Bob".into(),
                surname: "Jones".into(),
                member_key: *member_key(0x21).as_bytes(),
                device_keys: vec![*device_key(0x22).as_bytes()],
            }],
            proxy_account: None,
            org_private_key: None,
        }],
        pending_invites: vec![WireInvite {
            org_id: OrgId::new([6u8; 20]),
            admin_device_key: *device_key(0x41).as_bytes(),
            admin_member_key: *member_key(0x42).as_bytes(),
            org_pub_key: *member_key(0x43).as_bytes(),
        }],
    }
}

fn sealed_wire_store(name: &str, store: &WireStore) -> PathBuf {
    let path = tmp_path(name);
    store::seal_for_test(&path, "pw", &postcard::to_allocvec(store).unwrap(), &mut OsRng).unwrap();
    path
}

fn too_long() -> String {
    "a".repeat(129)
}

/// The wire form of a store opens to exactly the records it holds, so the
/// table below makes one field invalid and nothing else.
/// verifies: LLR-8bum44
#[test]
fn a_store_with_every_record_kind_opens_with_every_field_parsed() {
    let opened = PersonaStore::open(sealed_wire_store("every-kind", &wire_store()), "pw").unwrap();
    let d = opened.data();
    assert_eq!((d.personas[0].handle.as_str(), d.personas[0].name.as_str(), d.personas[0].surname.as_str()), ("alice", "Alice", "Smith"));
    assert_eq!(d.orgs[0].org_pub_key, OrgPublicKey::parse(member_key(0x32).as_bytes()).unwrap());
    assert_eq!(d.orgs[0].admin_member_key, member_key(0x31));
    let m = &d.orgs[0].trie_members[0];
    assert_eq!((m.handle.as_str(), m.name.as_str(), m.surname.as_str()), ("bob", "Bob", "Jones"));
    assert_eq!((m.member_key, m.device_keys.clone()), (member_key(0x21), vec![device_key(0x22)]));
    let i = &d.pending_invites[0];
    assert_eq!((i.admin_device_key, i.admin_member_key), (device_key(0x41), member_key(0x42)));
    assert_eq!(i.org_pub_key, OrgPublicKey::parse(member_key(0x43).as_bytes()).unwrap());
}

/// Every field the store-open refusal names (design ledger, "Observable
/// changes"), each made invalid alone, is refused naming exactly that field.
/// verifies: LLR-8bum44
#[test]
fn a_store_with_any_one_field_invalid_is_refused_naming_exactly_that_field() {
    type Spoil = fn(&mut WireStore);
    let cases: [(&str, Spoil); 13] = [
        ("persona.handle", |s| s.personas[0].handle = "Alice".into()),
        ("persona.name", |s| s.personas[0].name = too_long()),
        ("persona.surname", |s| s.personas[0].surname = too_long()),
        ("org.org_pub_key", |s| s.orgs[0].org_pub_key = off_curve_key()),
        ("org.admin_member_key", |s| s.orgs[0].admin_member_key = off_curve_key()),
        ("member.handle", |s| s.orgs[0].trie_members[0].handle = "Bob".into()),
        ("member.name", |s| s.orgs[0].trie_members[0].name = too_long()),
        ("member.surname", |s| s.orgs[0].trie_members[0].surname = too_long()),
        ("member.member_key", |s| s.orgs[0].trie_members[0].member_key = off_curve_key()),
        ("member.device_keys", |s| s.orgs[0].trie_members[0].device_keys.push(off_curve_key())),
        ("pending_invite.admin_device_key", |s| s.pending_invites[0].admin_device_key = off_curve_key()),
        ("pending_invite.admin_member_key", |s| s.pending_invites[0].admin_member_key = off_curve_key()),
        ("pending_invite.org_pub_key", |s| s.pending_invites[0].org_pub_key = off_curve_key()),
    ];
    for (field, spoil) in cases {
        let mut wire = wire_store();
        spoil(&mut wire);
        let path = sealed_wire_store(field, &wire);
        let before = std::fs::read(&path).unwrap();
        let err = PersonaStore::open(path.clone(), "pw").err().unwrap_or_else(|| panic!("{field}: must be refused"));
        assert!(matches!(&err, OrgNodeError::InvalidField { field: f, .. } if *f == field), "{field}: got {err:?}");
        assert!(err.to_string().contains(field), "the message names the field: {err}");
        assert_eq!(std::fs::read(&path).unwrap(), before, "{field}: a refused store is left as it was");
    }
}

/// Two fields of one record invalid at once — the Organisation record's
/// Organisation public key and administrator Member key — reports the first in
/// record order, `org.org_pub_key`.
/// verifies: LLR-8bum44
#[test]
fn a_store_with_two_invalid_fields_in_one_record_reports_the_first_in_record_order() {
    let mut wire = wire_store();
    wire.orgs[0].org_pub_key = off_curve_key();
    wire.orgs[0].admin_member_key = off_curve_key();
    let err = PersonaStore::open(sealed_wire_store("two-invalid", &wire), "pw").err().expect("must be refused");
    assert!(matches!(&err, OrgNodeError::InvalidField { field: "org.org_pub_key", .. }), "got {err:?}");
}

/// A Join request as its wire form, field by field (postcard encodes a struct
/// as the tuple of its fields).
fn join_request_blob(handle: &str, name: &str, surname: &str, member_key: [u8; 32], device_key: [u8; 32]) -> String {
    blobs::encode(&(handle, name, surname, member_key, device_key, vec![9u8, 10])).unwrap()
}

/// verifies: LLR-8bum44
#[test]
fn a_valid_join_request_imports_with_every_field_parsed() {
    let jr = OrgService::import_join_request(&join_request_blob(
        "bob", "Bob", "Jones", *member_key(0x21).as_bytes(), *device_key(0x22).as_bytes(),
    ))
    .unwrap();
    assert_eq!(jr.handle.as_str(), "bob");
    assert_eq!(jr.member_key, member_key(0x21));
    assert_eq!(jr.device_key, device_key(0x22));
    assert_eq!(jr.node_addr, vec![9, 10]);
}

/// A Join request whose details arrive in a decomposed (non-NFC) form imports
/// with them in NFC: the import parses each detail, and the parse canonicalises.
/// verifies: LLR-8bum44
#[test]
fn a_non_nfc_join_request_imports_with_its_details_in_nfc() {
    let jr = OrgService::import_join_request(&join_request_blob(
        "jose\u{0301}", "Jose\u{0301}", "Nun\u{0303}ez", *member_key(0x21).as_bytes(), *device_key(0x22).as_bytes(),
    ))
    .unwrap();
    assert_eq!(
        (jr.handle.as_str(), jr.name.as_str(), jr.surname.as_str()),
        ("jos\u{e9}", "Jos\u{e9}", "Nu\u{f1}ez"),
        "imported in NFC"
    );
}

/// verifies: LLR-8bum44
#[test]
fn a_join_request_holding_an_invalid_value_is_refused_naming_the_field() {
    let mk = *member_key(0x21).as_bytes();
    let dk = *device_key(0x22).as_bytes();
    let over = "a".repeat(129);
    let cases = [
        (join_request_blob("Bob", "Bob", "Jones", mk, dk), "join_request.handle"),
        (join_request_blob("bob", &over, "Jones", mk, dk), "join_request.name"),
        (join_request_blob("bob", "Bob", &over, mk, dk), "join_request.surname"),
        (join_request_blob("bob", "Bob", "Jones", off_curve_key(), dk), "join_request.member_key"),
        (join_request_blob("bob", "Bob", "Jones", mk, off_curve_key()), "join_request.device_key"),
    ];
    for (blob, field) in cases {
        let err = OrgService::import_join_request(&blob).unwrap_err();
        assert!(matches!(&err, OrgNodeError::InvalidField { field: f, .. } if *f == field), "got {err:?}");
        assert!(err.to_string().contains(field), "the message names the field: {err}");
    }
}

/// A record snapshot (what a first admission extends) as its wire form: a
/// list of member snapshots, each the tuple of its fields.
fn snapshot_bytes(handle: &str, name: &str, surname: &str, member_key: [u8; 32], device_keys: Vec<[u8; 32]>) -> Vec<u8> {
    postcard::to_allocvec(&vec![(MemberId::new([1u8; 32]), handle, name, surname, member_key, device_keys)]).unwrap()
}

/// verifies: LLR-8bum44
#[test]
fn a_valid_record_snapshot_decodes_with_every_field_parsed() {
    let bytes = snapshot_bytes("bob", "Bob", "Jones", *member_key(0x21).as_bytes(), vec![*device_key(0x22).as_bytes()]);
    let trie = first_admission_base(Some(&bytes)).unwrap();
    let leaf = trie.get(&MemberId::new([1u8; 32])).expect("the member is in the record");
    assert_eq!(leaf.handle().as_str(), "bob");
    assert_eq!(*leaf.p2p_key(), member_key(0x21));
    assert_eq!(leaf.p2p_devices(), [device_key(0x22)].as_slice());
}

/// verifies: LLR-8bum44
#[test]
fn a_record_snapshot_holding_an_invalid_value_is_refused_naming_the_field() {
    let mk = *member_key(0x21).as_bytes();
    let dk = *device_key(0x22).as_bytes();
    let over = "a".repeat(129);
    let cases = [
        (snapshot_bytes("Bob", "Bob", "Jones", mk, vec![dk]), "member.handle"),
        (snapshot_bytes("bob", &over, "Jones", mk, vec![dk]), "member.name"),
        (snapshot_bytes("bob", "Bob", &over, mk, vec![dk]), "member.surname"),
        (snapshot_bytes("bob", "Bob", "Jones", off_curve_key(), vec![dk]), "member.member_key"),
        (snapshot_bytes("bob", "Bob", "Jones", mk, vec![dk, off_curve_key()]), "member.device_keys"),
    ];
    for (bytes, field) in cases {
        let err = first_admission_base(Some(&bytes)).expect_err("must be refused");
        assert!(matches!(&err, OrgNodeError::InvalidField { field: f, .. } if *f == field), "got {err:?}");
        assert!(err.to_string().contains(field), "the message names the field: {err}");
    }
}

/// An invalid handle and an invalid Member key in one member snapshot report
/// the handle: fields are parsed in record order (design ledger, "Observable
/// changes").
/// verifies: LLR-8bum44
#[test]
fn a_record_snapshot_with_an_invalid_handle_and_member_key_reports_the_handle() {
    let bytes = snapshot_bytes("Bob", "Bob", "Jones", off_curve_key(), vec![*device_key(0x22).as_bytes()]);
    let err = first_admission_base(Some(&bytes)).expect_err("must be refused");
    assert!(matches!(&err, OrgNodeError::InvalidField { field: "member.handle", .. }), "got {err:?}");
}

/// An Invite as its wire form, field by field.
fn invite_blob(org_pub_key: [u8; 32], admin_member_key: [u8; 32], admin_device_key: [u8; 32]) -> String {
    blobs::encode(&(OrgId::new([5u8; 20]), org_pub_key, admin_member_key, admin_device_key, vec![4u8, 5])).unwrap()
}

/// verifies: LLR-8bum44
#[test]
fn a_valid_invite_imports_as_a_pending_invite() {
    let path = tmp_path("invite-valid");
    let mut svc = OrgService::new(PersonaStore::open(path.clone(), "pw").unwrap(), Box::new(MockChainOps::new()));
    let org_key = *member_key(0x43).as_bytes();
    let blob = invite_blob(org_key, *member_key(0x42).as_bytes(), *device_key(0x41).as_bytes());
    let inv = svc.import_invite(&mut OsRng, &blob).unwrap();
    assert_eq!(inv.org_pub_key.as_bytes(), &org_key);
    let reopened = PersonaStore::open(path, "pw").unwrap();
    let pending = &reopened.data().pending_invites;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].org_pub_key.as_bytes(), &org_key);
    assert_eq!(pending[0].admin_member_key, member_key(0x42));
    assert_eq!(pending[0].admin_device_key, device_key(0x41));
}

/// An Invite whose Organisation public key, administrator's Member-as-a-group
/// key or administrator's DevicePublicKey is not a valid key of its kind fails
/// to decode, as master reports it (`Chain("blob decode…")`), and nothing is
/// stored (LLR-8bum44 as amended 2026-10-05).
/// verifies: LLR-8bum44
#[test]
fn an_invite_holding_a_key_that_is_not_a_curve_point_is_refused_and_nothing_stored() {
    let (ok, mk, dk) = (*member_key(0x43).as_bytes(), *member_key(0x42).as_bytes(), *device_key(0x41).as_bytes());
    let cases = [
        ("org_pub_key", invite_blob(off_curve_key(), mk, dk)),
        ("admin_member_key", invite_blob(ok, off_curve_key(), dk)),
        ("admin_device_key", invite_blob(ok, mk, off_curve_key())),
    ];
    for (what, blob) in cases {
        let path = tmp_path(&format!("invite-{what}"));
        let mut svc = OrgService::new(PersonaStore::open(path.clone(), "pw").unwrap(), Box::new(MockChainOps::new()));
        let err = svc.import_invite(&mut OsRng, &blob).err().unwrap_or_else(|| panic!("{what}: must be refused"));
        assert!(matches!(&err, OrgNodeError::Chain(m) if m.starts_with("blob decode")), "{what}: got {err:?}");
        assert!(!path.exists(), "{what}: nothing was stored");
    }
}
