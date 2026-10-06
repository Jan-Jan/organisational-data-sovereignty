#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The Persona's records: Persona details are held parsed and
//! `create_persona` takes them typed (LLR-g76zqd); a Persona store or record
//! snapshot holding a value its type's parse refuses is refused whole, naming
//! the field (LLR-8bum44). Both requirements sit under SDD-af5vnt. (The Join
//! request and the Invite left org-node in T7 of the chain-authority change.)

use std::path::PathBuf;

use org_members::{Handle, MemberId, Name, RootHash, Surname};
use org_node::service::first_admission_base;
use org_node::ids::OrgId;
use org_node::store::{
    self, ExpectedAdmission, MemberSnapshot, OrgRecord, PersonaDetails, PersonaRecord, PersonaStatus, PersonaStore,
    ProvisionalChange, StoreData,
};
use org_node::test_fixtures::{device_key, member_key};
use org_node::{
    ChainAccount, DeviceSeed, Epoch, MemberSeed, MockChainOps, OrgNodeError, OrgPrivateKey, OrgPublicKey, OrgService,
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
        last_seq: SequenceNumber::new(0),
        trie_members: vec![MemberSnapshot {
            id: MemberId::new([1u8; 32]),
            handle: Handle::parse("bob").unwrap(),
            name: Name::parse("Bob").unwrap(),
            surname: Surname::parse("Jones").unwrap(),
            member_key: member_key(0x21),
            device_keys: vec![device_key(0x22)],
        }],
        proxy_account: None,
        org_private_key: OrgPrivateKey::from([0x5d; 32]),
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
    let held = svc.list_personas()[0].clone();
    assert_eq!(
        (held.handle.as_str(), held.name.as_str(), held.surname.as_str()),
        ("jos\u{e9}", "Jos\u{e9}", "Smith"),
        "held as the parsed values the Persona was created with"
    );
    assert_eq!((&held.handle, &held.name, &held.surname), (&p.handle, &p.name, &p.surname));
    assert_eq!(
        svc.persona_public_keys(&pid).unwrap(),
        (
            held.member_seed.x25519_keypair().member_key().unwrap(),
            held.device_seed.signing_keypair().device_key().unwrap()
        )
    );
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

/// Details given in a decomposed (non-NFC) form are held, saved and reopened
/// in NFC.
/// verifies: LLR-q6n25z
#[test]
fn non_nfc_persona_details_are_stored_in_nfc() {
    let path = tmp_path("details-nfc");
    let mut svc = OrgService::new(PersonaStore::open(path.clone(), "pw").unwrap(), Box::new(MockChainOps::new()));
    let PersonaDetails { handle, name, surname } =
        PersonaDetails::parse("jose\u{0301}", "Jose\u{0301}", "Nun\u{0303}ez").unwrap();
    svc.create_persona(&mut OsRng, handle, name, surname).unwrap();
    let nfc = ("jos\u{e9}", "Jos\u{e9}", "Nu\u{f1}ez");
    let held = &svc.list_personas()[0];
    assert_eq!((held.handle.as_str(), held.name.as_str(), held.surname.as_str()), nfc, "held in NFC");
    let reopened = PersonaStore::open(path, "pw").unwrap();
    let p = &reopened.data().personas[0];
    assert_eq!((p.handle.as_str(), p.name.as_str(), p.surname.as_str()), nfc, "stored in NFC");
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
    let data = StoreData {
        personas: vec![persona("alice")],
        orgs: vec![org_with_member()],
        provisional_updates: vec![],
        expected_admissions: vec![],
    };
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
    provisional_updates: Vec<WireProvisional>,
    expected_admissions: Vec<[u8; 20]>,
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
    last_seq: SequenceNumber,
    trie_members: Vec<WireMember>,
    proxy_account: Option<ChainAccount>,
    org_private_key: OrgPrivateKey,
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
struct WireProvisional {
    org_id: Option<[u8; 20]>,
    persona_id: String,
    base_root: Option<[u8; 32]>,
    resulting_root: [u8; 32],
    seq: u64,
    org_pub_key: [u8; 32],
    change: WireChange,
}

#[derive(serde::Serialize)]
enum WireChange {
    Genesis { members: Vec<WireMember>, org_private_key: [u8; 32] },
    #[allow(dead_code)]
    ChangeSet { change_set: Vec<u8>, org_private_key: [u8; 32] },
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
            last_seq: SequenceNumber::new(0),
            trie_members: vec![wire_bob()],
            proxy_account: None,
            org_private_key: OrgPrivateKey::from([0x5e; 32]),
        }],
        provisional_updates: vec![WireProvisional {
            org_id: None,
            persona_id: "p1".into(),
            base_root: None,
            resulting_root: [0x66; 32],
            seq: 1,
            org_pub_key: *member_key(0x33).as_bytes(),
            change: WireChange::Genesis { members: vec![wire_bob()], org_private_key: [0x88; 32] },
        }],
        expected_admissions: vec![[0x77; 20]],
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
    let m = &d.orgs[0].trie_members[0];
    assert_eq!((m.handle.as_str(), m.name.as_str(), m.surname.as_str()), ("bob", "Bob", "Jones"));
    assert_eq!((m.member_key, m.device_keys.clone()), (member_key(0x21), vec![device_key(0x22)]));
    let pu = &d.provisional_updates[0];
    assert_eq!(pu.org_pub_key, OrgPublicKey::parse(member_key(0x33).as_bytes()).unwrap());
    let ProvisionalChange::Genesis { members, .. } = &pu.change else { panic!("a genesis update") };
    assert_eq!(members[0].handle.as_str(), "bob");
    assert_eq!(
        d.expected_admissions,
        vec![ExpectedAdmission { org_id: OrgId::new([0x77; 20]) }]
    );
}

// LLR-byjvd9: every record carries the Organisation private key, encoded as
// its plain 32 bytes with no option tag, and it is read back. A store written
// before the change worktree-org-node-org-key-pair — the record ending in an
// option tag and no key — is refused, not read or migrated.
// verifies: LLR-byjvd9
#[test]
fn a_record_carries_its_organisation_private_key_and_a_store_without_one_is_refused() {
    let data = StoreData {
        personas: vec![],
        orgs: vec![org_with_member()],
        provisional_updates: vec![],
        expected_admissions: vec![],
    };
    let plaintext = postcard::to_allocvec(&data).unwrap();
    let key = org_with_member().org_private_key;
    // The record's last 32 bytes, before the two empty lists, are the key.
    assert_eq!(&plaintext[plaintext.len() - 34..plaintext.len() - 2], key.expose_secret(), "no option tag");
    let path = tmp_path("with-key");
    store::seal_for_test(&path, "pw", &plaintext, &mut OsRng).unwrap();
    assert_eq!(PersonaStore::open(path, "pw").unwrap().data().orgs[0].org_private_key, key);

    let mut legacy = plaintext[..plaintext.len() - 34].to_vec();
    legacy.extend_from_slice(&[0x00, 0x00, 0x00]); // `org_private_key: None`, then the two empty lists
    let path = tmp_path("without-key");
    store::seal_for_test(&path, "pw", &legacy, &mut OsRng).unwrap();
    assert!(PersonaStore::open(path, "pw").is_err(), "a record without the key is not read");
}

/// Every field the store-open refusal names (design ledger, "Observable
/// changes"), each made invalid alone, is refused naming exactly that field.
/// verifies: LLR-8bum44
#[test]
fn a_store_with_any_one_field_invalid_is_refused_naming_exactly_that_field() {
    type Spoil = fn(&mut WireStore);
    let cases: [(&str, Spoil); 9] = [
        ("persona.handle", |s| s.personas[0].handle = "Alice".into()),
        ("persona.name", |s| s.personas[0].name = too_long()),
        ("persona.surname", |s| s.personas[0].surname = too_long()),
        ("org.org_pub_key", |s| s.orgs[0].org_pub_key = off_curve_key()),
        ("member.handle", |s| s.orgs[0].trie_members[0].handle = "Bob".into()),
        ("member.name", |s| s.orgs[0].trie_members[0].name = too_long()),
        ("member.surname", |s| s.orgs[0].trie_members[0].surname = too_long()),
        ("member.member_key", |s| s.orgs[0].trie_members[0].member_key = off_curve_key()),
        ("member.device_keys", |s| s.orgs[0].trie_members[0].device_keys.push(off_curve_key())),
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
/// Organisation public key and its member's Member key — reports the first in
/// record order, `org.org_pub_key`.
/// verifies: LLR-8bum44
#[test]
fn a_store_with_two_invalid_fields_in_one_record_reports_the_first_in_record_order() {
    let mut wire = wire_store();
    wire.orgs[0].org_pub_key = off_curve_key();
    wire.orgs[0].trie_members[0].member_key = off_curve_key();
    let err = PersonaStore::open(sealed_wire_store("two-invalid", &wire), "pw").err().expect("must be refused");
    assert!(matches!(&err, OrgNodeError::InvalidField { field: "org.org_pub_key", .. }), "got {err:?}");
}

// Abnormal: a stored provisional update whose key or member does not parse
// fails the open as a whole, naming the field.
// verifies: LLR-8bum44
#[test]
fn a_store_with_an_invalid_provisional_update_is_refused_naming_the_field() {
    for (field, corrupt) in [
        ("provisional.org_pub_key", (|s: &mut WireStore| s.provisional_updates[0].org_pub_key = off_curve_key()) as fn(&mut WireStore)),
        ("member.handle", |s: &mut WireStore| match &mut s.provisional_updates[0].change {
            WireChange::Genesis { members, .. } => members[0].handle = "Not A Handle".into(),
            WireChange::ChangeSet { .. } => unreachable!(),
        }),
    ] {
        let mut wire = wire_store();
        corrupt(&mut wire);
        let path = sealed_wire_store(&format!("bad-provisional-{field}"), &wire);
        let err = PersonaStore::open(path, "pw").err().unwrap_or_else(|| panic!("{field}: must be refused"));
        assert!(matches!(&err, OrgNodeError::InvalidField { field: f, .. } if *f == field), "{field}: {err:?}");
    }
}

/// A record snapshot (what a first admission extends) as its wire form: a
/// list of member snapshots, each the tuple of its fields.
fn snapshot_bytes(handle: &str, name: &str, surname: &str, member_key: [u8; 32], device_keys: Vec<[u8; 32]>) -> Vec<u8> {
    postcard::to_allocvec(&vec![(MemberId::new([1u8; 32]), handle, name, surname, member_key, device_keys)]).unwrap()
}

/// verifies: LLR-8bum44, LLR-tcft2r
#[test]
fn a_valid_record_snapshot_decodes_with_every_field_parsed() {
    let bytes = snapshot_bytes("bob", "Bob", "Jones", *member_key(0x21).as_bytes(), vec![*device_key(0x22).as_bytes()]);
    let trie = first_admission_base(&bytes).unwrap();
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
        let err = first_admission_base(&bytes).expect_err("must be refused");
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
