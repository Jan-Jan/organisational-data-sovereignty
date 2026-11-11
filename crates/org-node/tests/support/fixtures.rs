//! Fixture builders: one Organisation's values built from parts, for the
//! tests that hold a store state no single operation leaves.
//!
//! The rule: every target that builds a record, a Member snapshot, a Persona
//! or a one-store service from parts starts from these, so the shape of each
//! is stated once. A site that varies a field states only that field, over
//! the fixture's parts: `OrgRecord::from(OrgRecordParts { epoch, ..record_parts(..) })`,
//! `PersonaRecord::from(PersonaRecordParts { status, ..persona_parts(..) })`,
//! `MemberSnapshot { name, ..snapshot(..) }`.
//!
//! The exception: a site whose values no fixture can hold (a snapshot of
//! keys read back from a store, a service reopened from sealed bytes as
//! org-io restarts it) builds its value itself, with a one-line comment
//! saying why.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// As in `support`: each target calls a subset of these builders.
#![allow(dead_code)]

use org_node::chain::OrgState;
use org_node::invitation::{EndpointAddresses, Invite, InviteId};
use org_node::service::{OrgService, StoreChange};
use org_node::store::{
    KeptAdmissionRequest, MemberSnapshot, OrgRecord, OrgRecordParts, PersonaRecord, PersonaRecordParts, PersonaStatus, ProvisionalChange,
    StoreData, StoreWrite,
};
use org_node::test_fixtures::{device_key, device_secret, empty_store, member_key, store_for_test};
use org_node::{
    DevicePublicKey, DeviceSecret, Epoch, Handle, MemberId, MemberSeed, Name, OrgPrivateKey, OrgPublicKey, PersonaId,
    RootHash, SequenceNumber, Surname,
};

use super::{the_org, trie_of_members};

/// Device seeds of the Members these fixtures name.
pub const ADA: u8 = 0xa1;
pub const BOB: u8 = 0xb1;
pub const CAROL: u8 = 0xc1;
pub const DAVE: u8 = 0xd1;
pub const EVE: u8 = 0xe1;
/// The record's Organisation private key.
pub const RECORD_KEY: [u8; 32] = [0x3d; 32];
/// The Device seed of Alice's Persona, the one `alice_persona` binds.
pub const OWN: u8 = 4;

/// A Member's snapshot: MemberId of `id` bytes, `handle`, the member key of
/// seed `member` and the Device keys of seeds `devices`, named Test Member.
pub fn snapshot(id: u8, handle: &str, member: u8, devices: &[u8]) -> MemberSnapshot {
    MemberSnapshot {
        id: MemberId::new([id; 32]),
        handle: Handle::parse(handle).unwrap(),
        name: Name::parse("Test").unwrap(),
        surname: Surname::parse("Member").unwrap(),
        member_key: member_key(member),
        device_keys: devices.iter().map(|seed| device_key(*seed)).collect(),
    }
}

/// Alice as a record lists her: MemberId `[1; 32]`, member seed 1, the
/// Device of seed `OWN`.
pub fn alice() -> MemberSnapshot {
    snapshot(1, "alice", 1, &[OWN])
}

/// The Device of Alice's Persona.
pub fn own() -> DevicePublicKey {
    device_key(OWN)
}

pub fn ada() -> MemberSnapshot {
    snapshot(0x01, "ada", 0x61, &[ADA])
}

/// The Device of Ada's Persona: the store `ada_store` builds is held under it.
pub fn ada_device() -> DevicePublicKey {
    device_key(ADA)
}

pub fn bob() -> MemberSnapshot {
    snapshot(0x02, "bob", 0x62, &[BOB])
}

pub fn carol() -> MemberSnapshot {
    snapshot(0x03, "carol", 0x63, &[CAROL])
}

pub fn dave() -> MemberSnapshot {
    snapshot(0x04, "dave", 0x64, &[DAVE])
}

pub fn eve() -> MemberSnapshot {
    snapshot(0x05, "eve", 0x65, &[EVE])
}

/// The public half of the Organisation private key of `secret` bytes.
pub fn public_key(secret: [u8; 32]) -> OrgPublicKey {
    OrgPrivateKey::from(secret).x25519_keypair().org_public_key().unwrap()
}

/// The parts of the record of `the_org` at `epoch` listing `members` in the
/// order given, its root their trie's and its last sequence number `epoch`,
/// under the key pair of `RECORD_KEY`, with no proxy and nothing kept.
pub fn record_parts(epoch: u64, members: &[MemberSnapshot]) -> OrgRecordParts {
    OrgRecordParts {
        org_id: the_org(),
        root_hash: trie_of_members(members).root_hash().unwrap(),
        org_pub_key: public_key(RECORD_KEY),
        epoch: Epoch::new(epoch),
        last_seq: SequenceNumber::new(epoch),
        trie_members: members.to_vec(),
        proxy_account: None,
        org_private_key: OrgPrivateKey::from(RECORD_KEY),
        kept_change_set: None,
        kept_base_snapshot: None,
    }
}

/// The record `record_parts(epoch, members)` holds.
pub fn record(epoch: u64, members: &[MemberSnapshot]) -> OrgRecord {
    OrgRecord::from(record_parts(epoch, members))
}

/// The parts of an unbound, Proposed Persona `persona_id`, Alice Smith of
/// Handle alice, with the member seed `member_seed` and the Device key
/// `device_key`: the values `test_fixtures::persona_record` gives.
pub fn persona_parts(persona_id: &str, member_seed: [u8; 32], device_key: DevicePublicKey) -> PersonaRecordParts {
    PersonaRecordParts {
        persona_id: PersonaId::new(persona_id.to_string()),
        org_id: None,
        name: Name::parse("Alice").unwrap(),
        surname: Surname::parse("Smith").unwrap(),
        handle: Some(Handle::parse("alice").unwrap()),
        member_seed: MemberSeed::from(member_seed),
        device_key,
        member_id: None,
        status: PersonaStatus::Proposed,
    }
}

/// The Persona `listed` names, Active in `the_org`, with the member seed of
/// `member_seed` bytes.
pub fn persona(listed: &MemberSnapshot, member_seed: u8) -> PersonaRecord {
    PersonaRecord::from(PersonaRecordParts {
        persona_id: PersonaId::new(listed.handle.as_str().to_string()),
        org_id: Some(the_org()),
        name: listed.name.clone(),
        surname: listed.surname.clone(),
        handle: Some(listed.handle.clone()),
        member_seed: MemberSeed::from([member_seed; 32]),
        device_key: listed.device_keys[0],
        member_id: Some(listed.id),
        status: PersonaStatus::Active,
    })
}

/// Alice's Persona "p-alice" (member seed 1, the Device of seed `OWN`),
/// bound Active to `the_org` as Alice's MemberId.
pub fn alice_persona() -> PersonaRecord {
    PersonaRecord::from(PersonaRecordParts {
        org_id: Some(the_org()),
        member_id: Some(alice().id),
        status: PersonaStatus::Active,
        ..persona_parts("p-alice", [1; 32], own())
    })
}

/// The store of `persona` holding `record` as its current record, of the
/// Organisation "Acme".
pub fn acme_store(persona: PersonaRecord, record: OrgRecord) -> StoreData {
    let mut store_data = empty_store(persona);
    store_data.org_name = "Acme".to_string();
    store_data.record = Some(record);
    store_data
}

/// Ada's store: a record at `epoch` listing `members`, of the Organisation
/// "Acme".
pub fn ada_store(epoch: u64, members: &[MemberSnapshot]) -> StoreData {
    acme_store(persona(&ada(), 0x61), record(epoch, members))
}

/// A service holding the one store `store_data`, sealed under `secret`.
pub fn service_under(secret: &DeviceSecret, store_data: StoreData) -> OrgService {
    OrgService::new(vec![store_for_test(secret, store_data)]).unwrap()
}

/// A service holding the one store `store_data`, sealed under the Device secret
/// of `device` bytes.
pub fn service(device: u8, store_data: StoreData) -> OrgService {
    service_under(&device_secret([device; 32]), store_data)
}

/// A service holding Alice's one store `store_data`, under her Device's
/// secret.
pub fn own_service(store_data: StoreData) -> OrgService {
    service(OWN, store_data)
}

/// The encoded store of Alice's Device in `svc`.
pub fn own_encoded(svc: &OrgService) -> Vec<u8> {
    encoded(svc, OWN)
}

/// The store data of the Device of seed `device` in `svc`.
pub fn held(svc: &OrgService, device: u8) -> &StoreData {
    super::held_store(svc, &device_key(device))
}

/// The encoded store of the Device of seed `device` in `svc`
/// (`support::encoded_store`).
pub fn encoded(svc: &OrgService, device: u8) -> Vec<u8> {
    super::encoded_store(svc, &device_key(device))
}

/// The root of the record of the Device of seed `device` in `svc`.
pub fn record_root(svc: &OrgService, device: u8) -> RootHash {
    held(svc, device).record.as_ref().unwrap().root_hash
}

/// The chain state of `root` and `key` at `epoch`.
pub fn chain(epoch: u64, root: RootHash, key: OrgPublicKey) -> OrgState {
    OrgState { root_hash: root, org_pub_key: key, epoch: Epoch::new(epoch) }
}

/// The invite identifier of `byte` bytes.
pub fn invite_id_of(byte: u8) -> InviteId {
    InviteId::new([byte; 32])
}

/// An Invite of Bob Builder to `the_org` (invite identifier `[3; 32]`),
/// issued by Device 5 at one address: the Invite another node issued.
pub fn invite_to_bob() -> Invite {
    Invite {
        org_name: "Acme".into(),
        org_id: the_org(),
        invitee_name: "Bob Builder".into(),
        issuer: device_key(5),
        issuer_addresses: addresses(&["192.0.2.5:4433"]),
        invite_id: InviteId::new([3; 32]),
    }
}

/// The Invite Ada issued to Carol (invite identifier of `0x11` bytes), with
/// Ada's addresses.
pub fn invite_to_carol() -> Invite {
    Invite {
        org_name: "Acme".to_string(),
        org_id: the_org(),
        invitee_name: "Carol".to_string(),
        issuer: ada_device(),
        issuer_addresses: addresses(&["ada.example:4433"]),
        invite_id: invite_id_of(0x11),
    }
}

/// Carol's provisional Member store, which holds no record, made from Ada's
/// Invite; the service holds it, so its sealed write is not read.
pub fn provisional_member() -> (OrgService, DevicePublicKey) {
    let mut svc = OrgService::new(Vec::new()).unwrap();
    let (key, _secret, _sealed) =
        svc.propose_member(&mut rand::rngs::OsRng, &invite_to_carol(), "Carol", "Coder", &["carol"]).unwrap();
    (svc, key)
}

/// Carol Coder's admission request for `invite_id` from the Device of seed
/// `device`, with the member key of seed `member` and the one preferred
/// Handle `handle_text`, as a store keeps it.
pub fn kept_request(invite_id: InviteId, member: u8, device: u8, handle_text: &str) -> KeptAdmissionRequest {
    KeptAdmissionRequest {
        org_id: the_org(),
        invite_id,
        from: device_key(device),
        name: Name::parse("Carol").unwrap(),
        surname: Surname::parse("Coder").unwrap(),
        preferred_handles: vec![super::handle(handle_text)],
        member_key: member_key(member),
        device_key: device_key(device),
        addresses: addresses(&["198.51.100.7:7000"]),
    }
}

/// `entries` as the address strings a caller hands an operation.
pub fn address_strings(entries: &[&str]) -> Vec<String> {
    entries.iter().map(|entry| entry.to_string()).collect()
}

/// `entries`, parsed as endpoint addresses.
pub fn addresses(entries: &[&str]) -> EndpointAddresses {
    EndpointAddresses::parse(address_strings(entries)).unwrap()
}

/// The one store write a change is (LLR-ay8nvu: `Sealed` only).
pub fn sealed(change: StoreChange) -> StoreWrite {
    let StoreChange::Sealed(write) = change;
    write
}

/// The postcard length of the tuple LLR-m76r4k measures, for `store_data`:
/// `(provisional_changes, provisional_organisations, kept_revocation_notices)`.
pub fn bound_len(store_data: &StoreData) -> usize {
    postcard::to_allocvec(&(&store_data.provisional_changes, &store_data.provisional_organisations, &store_data.kept_revocation_notices))
        .unwrap()
        .len()
}

/// `count` distinct `Revoke` changes, as padding.
pub fn revokes(count: usize) -> Vec<ProvisionalChange> {
    (0..count)
        .map(|index| {
            let mut id = [0x77u8; 32];
            id[..8].copy_from_slice(&(index as u64).to_le_bytes());
            ProvisionalChange::Revoke { member_id: MemberId::new(id) }
        })
        .collect()
}

/// The fewest padding changes for which `bound_len_with(count)`, the bound length of
/// a store holding `count` of them and whatever the caller keeps, exceeds
/// the bound.
pub fn fewest_exceeding(bound_len_with: impl Fn(usize) -> usize) -> usize {
    use org_node::store::MAX_PROVISIONAL_BYTES;
    let bytes_per_revoke = bound_len_with(1) - bound_len_with(0);
    let mut count = (MAX_PROVISIONAL_BYTES - bound_len_with(0)) / bytes_per_revoke;
    while bound_len_with(count) <= MAX_PROVISIONAL_BYTES {
        count += 1;
    }
    while count > 0 && bound_len_with(count - 1) > MAX_PROVISIONAL_BYTES {
        count -= 1;
    }
    count
}
