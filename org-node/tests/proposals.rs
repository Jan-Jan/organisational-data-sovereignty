#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The proposal operations (SDD-hwax8v): a founding and its proxy, a
//! Member, the Admin's provisional changes and the updated Organisation —
//! epoch state, never a chain write.

use org_node::service::StoreChange;
use org_node::invitation::{EndpointAddresses, Invite, InviteId};
use org_node::store::{ExpectedAdmission, PersonaStatus, PersonaStore, StoreData};
use org_node::test_fixtures::{device_key, device_secret, persona_record, store_for_test};
use org_node::{
    ChainAccount, DevicePublicKey, DeviceSecret, Epoch, OrgId, OrgNodeError, OrgService, PersonaId,
};
use rand::rngs::OsRng;

use support::sweep;
mod support;

use support::{ConstRng, CountingRng};

/// The bytes sealing a store draws: one XChaCha20-Poly1305 nonce.
const SEAL_NONCE_BYTES: usize = 24;

/// A service with no store.
fn storeless_service() -> OrgService {
    OrgService::new(vec![]).unwrap()
}

/// The store `change` sealed, opened with a copy of `secret`.
fn reopen(change: StoreChange, secret: &DeviceSecret) -> PersonaStore {
    let StoreChange::Sealed(write) = change;
    PersonaStore::unseal(&write.sealed, write.device_key, support::secret_copy(secret)).unwrap()
}

/// A founding proposed by "alice" on `svc`: its Device key, secret and change.
fn alice_founding(svc: &mut OrgService) -> (DevicePublicKey, DeviceSecret, StoreChange) {
    svc.propose_founding(&mut OsRng, "Alice", "Smith", "alice", "Acme".into()).unwrap()
}

// ---- Proposing a founding and recording its proxy --------------------------

// Normal: one new store, holding the Persona (no Organisation yet, its
// Handle set, proposed), the Organisation name and the provisional founding
// at epoch 1 — the founder alone, its root, the key pair and no proxy — and
// no current record; the change is that store's, and it reopens the same.
// verifies: LLR-d4gpxm
#[test]
fn propose_founding_adds_one_store_with_a_provisional_founding_at_epoch_one_and_no_record() {
    let mut svc = storeless_service();
    let (device, secret, change) = alice_founding(&mut svc);
    assert_eq!(svc.list_personas().len(), 1, "one store added");
    let store_data = svc.store_data(&device).unwrap();
    assert_eq!(store_data.persona.org_id, None);
    assert_eq!(store_data.persona.handle.as_ref().map(|handle| handle.as_str()), Some("alice"));
    assert_eq!(store_data.persona.status, PersonaStatus::Proposed);
    assert_eq!(store_data.persona.member_id, None);
    assert_eq!(store_data.org_name, "Acme");
    assert!(store_data.record.is_none(), "no current record");
    let founding = store_data.founding.as_ref().expect("a provisional founding");
    assert_eq!((founding.epoch, founding.proxy), (Epoch::new(1), None));
    assert_eq!(founding.members.len(), 1, "the founder alone");
    let founder = &founding.members[0];
    assert_eq!((founder.handle.as_str(), founder.device_keys.clone()), ("alice", vec![device]));
    assert_eq!(founder.member_key, store_data.persona.member_seed_for_test().x25519_keypair().member_key().unwrap());
    assert_eq!(founding.org_private_key_for_test().x25519_keypair().org_public_key().unwrap(), founding.org_pub_key);
    let reopened = reopen(change, &secret);
    assert_eq!(
        postcard::to_allocvec(reopened.data().unwrap()).unwrap(),
        postcard::to_allocvec(store_data).unwrap(),
        "the change is the store as held"
    );
}

// Normal: the operation takes no chain state and makes no chain call — its
// signature names none and the service holds none — and the Device secret is
// returned by value: the store holds the Device public key, never the seed.
// verifies: LLR-d4gpxm
#[test]
fn propose_founding_calls_no_chain_operation_and_returns_the_secret_by_value() {
    let signatures = support::public_signatures("service.rs");
    let signature = signatures.iter().find(|signature| signature.starts_with("pub fn propose_founding<")).unwrap();
    for chain in ["OrgState", "chain", "Chain"] {
        assert!(!signature.contains(chain), "the signature names {chain}: {signature}");
    }
    let mut svc = storeless_service();
    // The return type is pinned by the compiler: the secret comes back by value.
    let (device, secret, change): (DevicePublicKey, DeviceSecret, StoreChange) = alice_founding(&mut svc);
    assert_eq!(secret.device_key(), device);
    let StoreChange::Sealed(write) = change;
    let plaintext = postcard::to_allocvec(svc.store_data(&device).unwrap()).unwrap();
    let seed = support::seed_bytes(&secret);
    assert!(!plaintext.windows(32).any(|window| window == seed.as_slice()), "the store holds no Device seed");
    assert!(!write.sealed.as_bytes().windows(32).any(|window| window == seed.as_slice()));
}

// Abnormal: a random source that repeats makes the Organisation key pair the
// founder's member key pair, so the Organisation public key would be the
// founder's Member-as-a-group key: refused with `DuplicateOrgPublicKey`,
// adding no store and returning no secret.
// verifies: LLR-9qavm7
#[test]
fn a_founding_key_equal_to_a_founder_key_is_refused_adding_no_store() {
    let mut svc = storeless_service();
    let refused = svc.propose_founding(&mut ConstRng(0x5c), "Alice", "Smith", "alice", "Acme".into());
    assert_eq!(refused.err(), Some(OrgNodeError::DuplicateOrgPublicKey));
    assert!(svc.list_personas().is_empty(), "no store added");
}

// Abnormal, then normal: the key pair is drawn after the root, and one equal to a founder key is
// refused; a fresh draw is then accepted with a key equal to neither founder
// key.
// verifies: LLR-9qavm7
#[test]
fn a_founding_key_equal_to_a_founder_key_is_refused() {
    let mut svc = storeless_service();
    let mut rng = ConstRng(0x4e);
    assert_eq!(
        svc.propose_founding(&mut rng, "Alice", "Smith", "alice", "Acme".into()).err(),
        Some(OrgNodeError::DuplicateOrgPublicKey)
    );
    let (device, _, _) = alice_founding(&mut svc);
    let store_data = svc.store_data(&device).unwrap();
    let founding = store_data.founding.as_ref().unwrap();
    let founder = &founding.members[0];
    assert_ne!(founding.org_pub_key.as_bytes(), founder.member_key.as_bytes());
    assert_ne!(founding.org_pub_key.as_bytes(), founder.device_keys[0].as_bytes());
}

// Abnormal: each detail its type refuses is refused, in order — name,
// surname, Handle — before any draw from the source, adding no store.
// verifies: LLR-rjphn4
#[test]
fn details_are_parsed_in_order_before_any_draw() {
    let over = "a".repeat(129);
    let mut svc = storeless_service();
    for (name, surname, handle, field) in [
        (over.as_str(), over.as_str(), "Alice", "member.name"),
        ("Alice", over.as_str(), "Alice", "member.surname"),
        ("Alice", "Smith", "Alice", "member.handle"),
    ] {
        let mut rng = CountingRng::default();
        let refused = svc.propose_founding(&mut rng, name, surname, handle, "Acme".into()).expect_err("refused");
        assert_eq!(support::refused_field(refused), field);
        assert_eq!(rng.drawn, 0, "{field}: refused before any draw");
    }
    assert!(svc.list_personas().is_empty(), "no store added");
}

// Normal: the pure proxy and the Organisation identifier are recorded, in
// the founding and the Persona, and reach the sealed store.
// verifies: LLR-nx87k5
#[test]
fn record_founding_proxy_sets_the_proxy_and_the_org_id() {
    let mut svc = storeless_service();
    let (device, secret, _) = alice_founding(&mut svc);
    let (org_id, proxy) = (OrgId::new([7; 20]), ChainAccount::new([0x5a; 32]));
    let change = svc.record_founding_proxy(&mut OsRng, &device, org_id, proxy).unwrap();
    let store_data = svc.store_data(&device).unwrap();
    assert_eq!(store_data.founding.as_ref().unwrap().proxy, Some(proxy));
    assert_eq!(store_data.persona.org_id, Some(org_id));
    assert!(store_data.record.is_none(), "still no current record");
    let reopened = reopen(change, &secret);
    assert_eq!(reopened.data().unwrap().founding.as_ref().unwrap().proxy, Some(proxy));
    assert_eq!(reopened.data().unwrap().persona.org_id, Some(org_id));
}

// Abnormal: a store with no provisional founding, another proxy, and
// another Organisation identifier are refused, changing nothing; normal: the
// same proxy and identifier again are accepted, sealed unchanged in content.
// verifies: LLR-nx87k5
#[test]
fn record_founding_proxy_refuses_no_founding_or_another_proxy_and_accepts_the_same_again() {
    let mut svc = storeless_service();
    let (device, _, _) = alice_founding(&mut svc);
    let (org_id, proxy) = (OrgId::new([7; 20]), ChainAccount::new([0x5a; 32]));
    crate::support::unread(svc.record_founding_proxy(&mut OsRng, &device, org_id, proxy).unwrap());
    let before = support::encoded_stores(&svc);
    assert_eq!(
        svc.record_founding_proxy(&mut OsRng, &device, org_id, ChainAccount::new([0x5b; 32])).err(),
        Some(OrgNodeError::FoundingProxyAlreadyRecorded)
    );
    assert_eq!(
        svc.record_founding_proxy(&mut OsRng, &device, OrgId::new([8; 20]), proxy).err(),
        Some(OrgNodeError::FoundingProxyAlreadyRecorded)
    );
    assert_eq!(support::encoded_stores(&svc), before, "nothing changed");
    let again = svc.record_founding_proxy(&mut OsRng, &device, org_id, proxy).unwrap();
    assert_eq!(support::encoded_stores(&svc), before, "the same values again change no content");
    let StoreChange::Sealed(write) = again;
    assert_eq!(write.device_key, device, "and the store is sealed again, for a retried write");
    // A store with no provisional founding: one Persona and nothing else.
    let unfounded = StoreData::new(persona_record("p2", 0x41, 0x42));
    svc.restore_store(store_for_test(&device_secret([0x42; 32]), unfounded));
    assert_eq!(
        svc.record_founding_proxy(&mut OsRng, &device_key(0x42), org_id, proxy).err(),
        Some(OrgNodeError::NoProvisionalFounding)
    );
    assert_eq!(
        svc.record_founding_proxy(&mut OsRng, &device_key(0x43), org_id, proxy).err(),
        Some(OrgNodeError::PersonaNotHeld { device_key: Box::new(device_key(0x43)) })
    );
}

// Normal: the details are held as their parsed types in the new store.
// verifies: LLR-rjphn4
#[test]
fn founding_details_parse_into_their_types_and_add_a_store() {
    let mut svc = storeless_service();
    let (device, _, _) = svc.propose_founding(&mut OsRng, "Bea", "Bloggs", "bea", "Acme".into()).unwrap();
    let persona = &svc.store_data(&device).unwrap().persona;
    let name: &org_node::Name = &persona.name;
    let surname: &org_node::Surname = &persona.surname;
    let handle: &org_node::Handle = persona.handle.as_ref().unwrap();
    assert_eq!((name.as_str(), surname.as_str(), handle.as_str()), ("Bea", "Bloggs", "bea"));
    assert_eq!(svc.list_personas().len(), 1);
}

// Abnormal: each invalid detail of a founding or of a Member is refused
// naming it, and its message names it too.
// verifies: LLR-rjphn4
#[test]
fn founding_and_member_details_refuse_each_invalid_field_naming_it() {
    let over = "a".repeat(129);
    let mut svc = storeless_service();
    for (name, surname, handle, field) in [
        (over.as_str(), "Smith", "alice", "member.name"),
        ("Alice", over.as_str(), "alice", "member.surname"),
        ("Alice", "Smith", "Alice", "member.handle"),
    ] {
        let refused = svc.propose_founding(&mut OsRng, name, surname, handle, "Acme".into()).expect_err("refused");
        assert!(refused.to_string().contains(field), "the message names the field: {refused}");
        assert_eq!(support::refused_field(refused), field);
    }
    let invite = invite_to(OrgId::new([7; 20]), device_key(0x23));
    for (name, surname, handles, field) in [
        (over.as_str(), "Builder", vec!["bob"], "member.name"),
        ("Bob", over.as_str(), vec!["bob"], "member.surname"),
        ("Bob", "Builder", vec!["Bob"], "member.preferred_handles[0]"),
        ("Bob", "Builder", vec!["bob", "bo", "B"], "member.preferred_handles[2]"),
    ] {
        let mut rng = CountingRng::default();
        let refused = svc.propose_member(&mut rng, &invite, name, surname, &handles).expect_err("refused");
        assert!(refused.to_string().contains(field), "the message names the field: {refused}");
        assert_eq!(support::refused_field(refused), field);
        assert_eq!(rng.drawn, 0, "{field}: refused before any draw");
    }
    assert!(svc.list_personas().is_empty());
}

// Normal: details given decomposed are held, and sealed, in NFC.
// verifies: LLR-rjphn4
#[test]
fn non_nfc_details_are_stored_in_nfc() {
    let mut svc = storeless_service();
    let (device, secret, change) =
        svc.propose_founding(&mut OsRng, "Jose\u{0301}", "Nun\u{0303}ez", "jose\u{0301}", "Acme".into()).unwrap();
    let nfc = ("Jos\u{e9}", "Nu\u{f1}ez", "jos\u{e9}");
    let held = &svc.store_data(&device).unwrap().persona;
    assert_eq!((held.name.as_str(), held.surname.as_str(), held.handle.as_ref().unwrap().as_str()), nfc, "held in NFC");
    let reopened = reopen(change, &secret);
    let stored = &reopened.data().unwrap().persona;
    assert_eq!(
        (stored.name.as_str(), stored.surname.as_str(), stored.handle.as_ref().unwrap().as_str()),
        nfc,
        "stored in NFC"
    );
    assert_eq!(reopened.data().unwrap().founding.as_ref().unwrap().members[0].handle.as_str(), "jos\u{e9}");
}

// Abnormal: two invalid details at once report the first in parse order.
// verifies: LLR-rjphn4
#[test]
fn details_with_two_invalid_fields_report_the_first() {
    let over = "a".repeat(129);
    let mut svc = storeless_service();
    for (name, surname, handle, field) in [
        (over.as_str(), "Smith", "Alice", "member.name"),
        ("Alice", over.as_str(), "Alice", "member.surname"),
    ] {
        let refused = svc.propose_founding(&mut OsRng, name, surname, handle, "Acme".into()).expect_err("refused");
        assert_eq!(support::refused_field(refused), field);
    }
}

// Normal: the secret returned is the new store's Device's, and the store keeps its
// public key only.
// verifies: LLR-d4gpxm
#[test]
fn propose_founding_returns_the_secret_and_keeps_only_the_public_key() {
    let mut svc = storeless_service();
    let (device, secret, change) = alice_founding(&mut svc);
    let record = svc.list_personas()[0].clone();
    assert_eq!((secret.device_key(), record.device_key), (device, device));
    let StoreChange::Sealed(write) = change;
    assert_eq!(write.device_key, device);
    let (_, device_key) = svc.persona_public_keys(&record.persona_id).unwrap();
    assert_eq!(device_key, secret.device_key());
}

// Normal: proposing keeps
// a provisional founding and no record, reaching the sealed store; nothing
// is recorded, activated or written to any chain.
// verifies: LLR-d4gpxm, REQ-dhr58q, REQ-4qh2h8
#[test]
fn propose_founding_keeps_a_provisional_founding_and_no_record() {
    let mut svc = storeless_service();
    let (_, secret, change) = alice_founding(&mut svc);
    assert!(svc.list_orgs().is_empty(), "no record");
    assert_eq!(svc.list_personas()[0].status, PersonaStatus::Proposed);
    assert_eq!(svc.list_personas()[0].member_id, None);
    let reopened = reopen(change, &secret);
    assert!(reopened.data().unwrap().record.is_none(), "no record reaches the disk");
    assert!(reopened.data().unwrap().founding.is_some(), "the provisional founding does");
    assert!(reopened.data().unwrap().provisional_changes.is_empty() && reopened.data().unwrap().provisional_organisations.is_empty());
    let _: &PersonaId = &svc.list_personas()[0].persona_id;
}

/// An Invite to `org_id` from the issuing Device `issuer`.
fn invite_to(org_id: OrgId, issuer: DevicePublicKey) -> Invite {
    Invite {
        org_name: "Acme".into(),
        org_id,
        invitee_name: "Bob Builder".into(),
        issuer,
        issuer_addresses: EndpointAddresses::parse(vec!["192.0.2.1:7000".into()]).unwrap(),
        invite_id: InviteId::new([0x61; 32]),
    }
}

/// `svc`'s proposal, as bob, to join the Organisation of `invite`.
fn join(svc: &mut OrgService, invite: &Invite) -> Result<(DevicePublicKey, DeviceSecret, StoreChange), OrgNodeError> {
    svc.propose_member(&mut OsRng, invite, "Bob", "Builder", &["bob", "bobby"])
}

// Normal: one new store holding the Persona (bound to the Invite's
// Organisation, no Handle, proposed), the Invite's Organisation name, the
// provisional Member — the Invite's identifiers, issuer and addresses, the
// parsed details, the Persona's keys — and one expected admission; it
// reopens the same.
// verifies: LLR-6acxu9
#[test]
fn propose_member_adds_one_store_with_a_provisional_member_and_an_expected_admission() {
    let mut svc = storeless_service();
    let org_id = OrgId::new([7; 20]);
    let invite = invite_to(org_id, device_key(0x23));
    let (device, secret, change) = join(&mut svc, &invite).unwrap();
    assert_eq!(svc.list_personas().len(), 1);
    let store_data = svc.store_data(&device).unwrap();
    assert_eq!((store_data.persona.org_id, store_data.persona.handle.clone()), (Some(org_id), None));
    assert_eq!(store_data.persona.status, PersonaStatus::Proposed);
    assert_eq!(store_data.org_name, "Acme");
    assert!(store_data.record.is_none() && store_data.founding.is_none());
    assert_eq!(store_data.expected_admissions, vec![ExpectedAdmission { org_id }]);
    let member = store_data.member.as_ref().expect("the provisional Member");
    assert_eq!((member.org_id, member.invite_id, member.issuer), (org_id, invite.invite_id, invite.issuer));
    assert_eq!(member.issuer_addresses, invite.issuer_addresses);
    assert_eq!((member.name.as_str(), member.surname.as_str()), ("Bob", "Builder"));
    assert_eq!(
        member.preferred_handles.iter().map(|handle| handle.as_str()).collect::<Vec<_>>(),
        ["bob", "bobby"]
    );
    assert_eq!(member.device_key, device);
    assert_eq!(member.member_key, store_data.persona.member_seed_for_test().x25519_keypair().member_key().unwrap());
    assert_eq!(secret.device_key(), device);
    let reopened = reopen(change, &secret);
    assert_eq!(postcard::to_allocvec(reopened.data().unwrap()).unwrap(), postcard::to_allocvec(store_data).unwrap());
}

// Abnormal: no preferred Handle, four, or one repeated is refused naming the
// preferred Handles; an invalid one names its position — all adding no
// store.
// verifies: LLR-6acxu9, LLR-rjphn4
#[test]
fn propose_member_refuses_no_handle_four_handles_or_a_repeated_one() {
    let mut svc = storeless_service();
    let invite = invite_to(OrgId::new([7; 20]), device_key(0x23));
    for handles in [&[][..], &["a1", "b1", "c1", "d1"][..], &["bob", "bob"][..]] {
        let mut rng = CountingRng::default();
        let refused = svc.propose_member(&mut rng, &invite, "Bob", "Builder", handles).expect_err("refused");
        assert_eq!(support::refused_field(refused), "member.preferred_handles", "{handles:?}");
        assert_eq!(rng.drawn, 0, "{handles:?}: refused before any draw");
    }
    let mut rng = CountingRng::default();
    let refused = svc.propose_member(&mut rng, &invite, "Bob", "Builder", &["bob", "Bob"]).expect_err("refused");
    assert_eq!(support::refused_field(refused), "member.preferred_handles[1]");
    assert_eq!(rng.drawn, 0, "refused before any draw");
    assert!(svc.list_personas().is_empty(), "no store added");
    for handles in [&["bob"][..], &["bob", "bo"][..], &["bob", "bo", "b0b"][..]] {
        let (device, _, _) = svc.propose_member(&mut OsRng, &invite, "Bob", "Builder", handles).unwrap();
        assert_eq!(svc.store_data(&device).unwrap().member.as_ref().unwrap().preferred_handles.len(), handles.len());
        assert!(svc.drop_store(&device));
    }
}

// Abnormal: an Invite to an Organisation a held store is bound to (a
// founding with its identifier recorded), holds the record of, or expects
// admission to, is refused before any draw, adding no store; normal: an
// Invite to another Organisation is accepted.
// verifies: LLR-w2uyyh
#[test]
fn propose_member_refuses_an_organisation_a_held_store_is_bound_to_or_expects() {
    let mut svc = storeless_service();
    let (founder, _, _) = alice_founding(&mut svc);
    let bound = OrgId::new([7; 20]);
    crate::support::unread(svc.record_founding_proxy(&mut OsRng, &founder, bound, ChainAccount::new([0x5a; 32])).unwrap());
    let expected = OrgId::new([8; 20]);
    let (_device, _secret, joined) = join(&mut svc, &invite_to(expected, device_key(0x23))).unwrap();
    crate::support::unread(joined);
    for org_id in [bound, expected] {
        let before = support::encoded_stores(&svc);
        let mut rng = CountingRng::default();
        let refused = svc.propose_member(&mut rng, &invite_to(org_id, device_key(0x23)), "Cy", "Coder", &["cy"]);
        assert_eq!(refused.err(), Some(OrgNodeError::OrganisationHeldByAnotherPersona { org_id }));
        assert_eq!(rng.drawn, 0, "refused before any draw");
        assert_eq!(support::encoded_stores(&svc), before, "no store added, none changed");
    }
    let (device, _, _) = join(&mut svc, &invite_to(OrgId::new([9; 20]), device_key(0x23))).unwrap();
    assert_eq!(svc.store_data(&device).unwrap().persona.org_id, Some(OrgId::new([9; 20])));
}

/// The Organisation `side`'s Persona is bound to, as `persona_view` shows it.
fn bound_org(side: &sweep::Side) -> Option<OrgId> {
    side.svc.persona_view(&side.device).unwrap().org_id
}

// Normal: a founding Persona's Organisation is `None` until
// `record_founding_proxy` sets it; a provisional Member's is the Invite's from
// `propose_member` on. Then every store-changing operation the two stores
// take part in — the founder's adoption, an Invite, a kept admission request,
// a recorded admission, a proposal and its adoption, a cancelled invite, a
// recorded revocation and its adoption, and the Member's receipt and adoption
// of the Organisation — leaves each Organisation as it was. Abnormal: a
// second `record_founding_proxy` naming another Organisation is refused and
// changes nothing. (`acknowledgement_accepted` changes only a removed store,
// which keeps no Persona, LLR-2d5a7m.)
// verifies: LLR-qqqg4b
#[test]
fn a_personas_organisation_is_set_once_and_kept_by_every_store_change() {
    let mut svc = storeless_service();
    let (device, secret, change) = alice_founding(&mut svc);
    assert_eq!(svc.persona_view(&device).unwrap().org_id, None, "a founding is bound to no Organisation yet");
    let mut founder = sweep::Side::new(svc, device, secret, change);
    let change = founder.svc.record_founding_proxy(&mut OsRng, &device, support::the_org(), support::the_proxy()).unwrap();
    founder.keep(change);
    let org = Some(support::the_org());
    assert_eq!(bound_org(&founder), org, "record_founding_proxy sets it");
    let refused = founder.svc.record_founding_proxy(&mut OsRng, &device, OrgId::new([0x0b; 20]), support::the_proxy());
    assert_eq!(refused.err(), Some(OrgNodeError::FoundingProxyAlreadyRecorded));
    assert_eq!(bound_org(&founder), org, "a refused second proxy changes nothing");

    let chain = sweep::founding_state(&founder);
    sweep::adopt(&mut founder, chain);
    assert_eq!(bound_org(&founder), org, "verify's adoption");
    let invite = sweep::invite(&mut founder, "Bea");
    assert_eq!(bound_org(&founder), org, "build_invite");
    let mut bea = sweep::propose_member(&invite, "Bea", &["bea"]);
    assert_eq!(bound_org(&bea), org, "propose_member sets it to the Invite's Organisation");
    sweep::request_admission(&mut founder, &bea);
    assert_eq!(bound_org(&founder), org, "check_received of an admission request");
    let change = founder.svc.record_admission(&mut OsRng, &device, invite.invite_id, None).unwrap();
    founder.keep(change);
    assert_eq!(bound_org(&founder), org, "record_admission");
    let chain = sweep::propose_next(&mut founder);
    assert_eq!(bound_org(&founder), org, "propose_updated_organisation");
    sweep::adopt(&mut founder, chain);
    assert_eq!(bound_org(&founder), org, "verify's adoption of an update");
    let body = sweep::message_for(&founder, &bea.device);
    sweep::receive_and_adopt(&mut bea, founder.device, &body, chain);
    assert_eq!(bound_org(&bea), org, "the Member's check_received and adoption");

    let cancelled = sweep::invite(&mut founder, "Cy");
    let cy = sweep::propose_member(&cancelled, "Cy", &["cy"]);
    sweep::request_admission(&mut founder, &cy);
    let change = founder.svc.cancel_invite(&mut OsRng, &device, cancelled.invite_id).unwrap();
    founder.keep(change);
    assert_eq!(bound_org(&founder), org, "cancel_invite");
    sweep::revocation(&mut founder, sweep::member_id_of(&bea));
    assert_eq!(bound_org(&founder), org, "record_revocation, its proposal and adoption");
}

// Abnormal: a second Persona of this node may not join an Organisation another Persona holds;
// both stores are unchanged and nothing is added.
// verifies: LLR-w2uyyh
#[test]
fn propose_member_refuses_an_organisation_another_persona_holds() {
    let mut svc = storeless_service();
    let (founder, _, _) = alice_founding(&mut svc);
    let org_id = OrgId::new([7; 20]);
    crate::support::unread(svc.record_founding_proxy(&mut OsRng, &founder, org_id, ChainAccount::new([0x5a; 32])).unwrap());
    let before = support::encoded_stores(&svc);
    assert_eq!(
        join(&mut svc, &invite_to(org_id, device_key(0x23))).err(),
        Some(OrgNodeError::OrganisationHeldByAnotherPersona { org_id })
    );
    assert_eq!(support::encoded_stores(&svc), before);
    assert!(join(&mut svc, &invite_to(OrgId::new([0x42; 20]), device_key(0x23))).is_ok(), "another Organisation is not held");
}

// Abnormal: an Organisation another Persona of this node already expects is refused; the
// expectation stays the first Persona's alone.
// verifies: LLR-w2uyyh
#[test]
fn propose_member_refuses_an_organisation_another_persona_expects() {
    let mut svc = storeless_service();
    let org_id = OrgId::new([0x42; 20]);
    let (first, _, _) = join(&mut svc, &invite_to(org_id, device_key(0x23))).unwrap();
    let before = support::encoded_stores(&svc);
    assert_eq!(
        join(&mut svc, &invite_to(org_id, device_key(0x23))).err(),
        Some(OrgNodeError::OrganisationHeldByAnotherPersona { org_id })
    );
    assert_eq!(support::encoded_stores(&svc), before, "nothing written");
    assert_eq!(svc.list_personas().len(), 1);
    assert_eq!(svc.store_data(&first).unwrap().expected_admissions, vec![ExpectedAdmission { org_id }]);
}

// Abnormal: the refusal's message names the Organisation and no Persona.
// verifies: LLR-w2uyyh
#[test]
fn the_refusal_names_the_organisation_and_no_persona() {
    let mut svc = storeless_service();
    let org_id = OrgId::new([0x42; 20]);
    let (_device, _secret, joined) = join(&mut svc, &invite_to(org_id, device_key(0x23))).unwrap();
    crate::support::unread(joined);
    let text = join(&mut svc, &invite_to(org_id, device_key(0x23))).expect_err("refused").to_string();
    assert!(text.contains(&format!("{org_id:?}")), "names the Organisation: {text}");
    for persona in svc.list_personas() {
        assert!(!text.contains(persona.persona_id.as_str()), "names no Persona: {text}");
    }
}

/// Fixtures of the Admin's provisional changes and proposals: Ada's store,
/// holding a record at epoch 3 that lists Ada and Bob, and two kept
/// admission requests with their outstanding invites.
mod changes {
    use org_node::invitation::EndpointAddresses;
    use org_node::service::OrgService;
    use org_node::store::{KeptAdmissionRequest, OutstandingInvite, ProvisionalChange, StoreData};
    use org_node::test_fixtures::{device_key, member_key};
    use org_node::{Name, Surname};
    use rand::{CryptoRng, RngCore};

    pub use crate::support::fixtures::*;
    pub use crate::support::{handle, the_org, trie_of_members};

    /// A request kept for the invite of `invite_byte` bytes from the Device
    /// of seed `device`, named `name` Joiner.
    pub fn request(invite_byte: u8, name: &str, member: u8, device: u8, handles: &[&str]) -> KeptAdmissionRequest {
        KeptAdmissionRequest {
            org_id: the_org(),
            invite_id: invite_id_of(invite_byte),
            from: device_key(device),
            name: Name::parse(name).unwrap(),
            surname: Surname::parse("Joiner").unwrap(),
            preferred_handles: handles.iter().map(|text| handle(text)).collect(),
            member_key: member_key(member),
            device_key: device_key(device),
            addresses: EndpointAddresses::parse(vec![format!("10.0.0.{device}:4433")]).unwrap(),
        }
    }

    pub fn outstanding(invite_byte: u8, invitee_name: &str) -> OutstandingInvite {
        OutstandingInvite { org_id: the_org(), invite_id: invite_id_of(invite_byte), invitee_name: invitee_name.to_string() }
    }

    /// Ada's store: the record at epoch 3 listing Ada and Bob; Carol's and
    /// Dave's requests kept, each with its outstanding invite.
    pub fn admin_store() -> StoreData {
        let mut store_data = ada_store(3, &[ada(), bob()]);
        store_data.kept_admission_requests = vec![
            request(0x11, "Carol", 0x63, CAROL, &["bob", "carol", "cee"]),
            request(0x12, "Dave", 0x64, DAVE, &["carol", "dave"]),
        ];
        store_data.outstanding_invites = vec![outstanding(0x11, "Carol"), outstanding(0x12, "Dave")];
        store_data
    }

    /// `store_data` padded with the fewest `Revoke` changes for which keeping
    /// `extra` as well exceeds the bound; the padded store is within it.
    pub fn filled_before(store_data: &StoreData, extra: &ProvisionalChange) -> StoreData {
        let padded = |count: usize, plus: bool| {
            let mut filled = store_data.clone();
            filled.provisional_changes = revokes(count);
            if plus {
                filled.provisional_changes.push(extra.clone());
            }
            filled
        };
        let filled = padded(fewest_exceeding(|count| bound_len(&padded(count, true))), false);
        assert!(
            bound_len(&filled) <= org_node::store::MAX_PROVISIONAL_BYTES,
            "fixture: the padded store is within the bound"
        );
        filled
    }

    /// A random source that serves `first` for its first 32 bytes and
    /// `then` after.
    pub struct TwoRng {
        first: u8,
        then: u8,
        served: usize,
    }

    impl TwoRng {
        pub fn new(first: u8, then: u8) -> Self {
            Self { first, then, served: 0 }
        }
    }

    impl RngCore for TwoRng {
        fn next_u32(&mut self) -> u32 {
            let mut bytes = [0u8; 4];
            self.fill_bytes(&mut bytes);
            u32::from_le_bytes(bytes)
        }
        fn next_u64(&mut self) -> u64 {
            let mut bytes = [0u8; 8];
            self.fill_bytes(&mut bytes);
            u64::from_le_bytes(bytes)
        }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            for byte in dest {
                *byte = if self.served < 32 { self.first } else { self.then };
                self.served += 1;
            }
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    impl CryptoRng for TwoRng {}

    /// Ada's store with Carol's admission recorded and the updated
    /// Organisation proposed: Carol's MemberId of `0x22` bytes, the
    /// Organisation key of `0x4a` bytes. The service holds the store, so
    /// the sealed writes are not read.
    pub fn proposed() -> OrgService {
        let mut svc = service(ADA, admin_store());
        crate::support::unread(svc.record_admission(&mut rand::rngs::OsRng, &ada_device(), invite_id_of(0x11), None).unwrap());
        crate::support::unread(svc.propose_updated_organisation(&mut TwoRng::new(0x22, 0x4a), &ada_device()).unwrap());
        svc
    }
}

// Recording an admission and a revocation appends each, in order, to
// `provisional_changes` and changes nothing else: the record is byte for
// byte as it was, and each returns its store sealed.
// verifies: LLR-b2hgd4
#[test]
fn recording_a_change_appends_it_and_leaves_the_record_byte_for_byte() {
    let mut svc = changes::service(changes::ADA, changes::admin_store());
    let mut rest = changes::held(&svc, changes::ADA).clone();
    let admitted = svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None).unwrap();
    let revoked = svc.record_revocation(&mut rand::rngs::OsRng, &changes::ada_device(), org_node::MemberId::new([0x02; 32])).unwrap();
    let recorded = vec![
        org_node::store::ProvisionalChange::Admit { invite_id: changes::invite_id_of(0x11), handle: changes::handle("carol") },
        org_node::store::ProvisionalChange::Revoke { member_id: org_node::MemberId::new([0x02; 32]) },
    ];
    assert_eq!(changes::held(&svc, changes::ADA).provisional_changes, recorded);
    rest.provisional_changes = recorded.clone();
    assert_eq!(changes::encoded(&svc, changes::ADA), postcard::to_allocvec(&rest).unwrap(), "no other field changes, the record none");
    let write = changes::sealed(revoked);
    let reopened =
        org_node::store::PersonaStore::unseal(&write.sealed, changes::ada_device(), org_node::test_fixtures::device_secret([changes::ADA; 32]))
            .unwrap();
    assert_eq!(reopened.data().unwrap().provisional_changes, recorded);
    let first = changes::sealed(admitted);
    assert_ne!(first.sealed, write.sealed, "each change returns its own sealed store");
}

// Carol prefers bob (held by the record), then carol: she is given
// carol. Dave prefers carol (now held by Carol's Admit), then dave: he is
// given dave.
// verifies: LLR-bwtx3h
#[test]
fn record_admission_gives_the_first_free_preferred_handle_in_ranked_order() {
    let mut svc = changes::service(changes::ADA, changes::admin_store());
    crate::support::unread(svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None).unwrap());
    crate::support::unread(svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x12), None).unwrap());
    assert_eq!(
        changes::held(&svc, changes::ADA).provisional_changes,
        vec![
            org_node::store::ProvisionalChange::Admit { invite_id: changes::invite_id_of(0x11), handle: changes::handle("carol") },
            org_node::store::ProvisionalChange::Admit { invite_id: changes::invite_id_of(0x12), handle: changes::handle("dave") },
        ]
    );
}

// With every preferred Handle held, the chosen one is parsed (NFC) and given.
// verifies: LLR-bwtx3h
#[test]
fn with_no_free_handle_the_chosen_one_is_parsed_and_given() {
    let mut store_data = changes::admin_store();
    store_data.kept_admission_requests.push(changes::request(0x13, "Erin", 0x65, 0xe1, &["ada", "bob"]));
    store_data.outstanding_invites.push(changes::outstanding(0x13, "Erin"));
    let mut svc = changes::service(changes::ADA, store_data);
    crate::support::unread(svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x13), Some("zed")).unwrap());
    assert_eq!(
        changes::held(&svc, changes::ADA).provisional_changes,
        vec![org_node::store::ProvisionalChange::Admit { invite_id: changes::invite_id_of(0x13), handle: changes::handle("zed") }]
    );
}

// Each refusal of LLR-mq7n6u records nothing and changes nothing.
// verifies: LLR-mq7n6u
#[test]
fn record_admission_refuses_each_case_recording_nothing() {
    let refuse = |store_data: org_node::store::StoreData, invite_byte: u8, chosen: Option<&str>| {
        let mut svc = changes::service(changes::ADA, store_data);
        let before = changes::encoded(&svc, changes::ADA);
        let refused = svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(invite_byte), chosen);
        assert_eq!(changes::encoded(&svc, changes::ADA), before, "nothing recorded");
        refused.unwrap_err()
    };
    let mut erin = changes::admin_store();
    erin.kept_admission_requests.push(changes::request(0x13, "Erin", 0x65, 0xe1, &["ada", "bob"]));
    erin.outstanding_invites.push(changes::outstanding(0x13, "Erin"));
    let mut recorded = changes::admin_store();
    recorded
        .provisional_changes
        .push(org_node::store::ProvisionalChange::Admit { invite_id: changes::invite_id_of(0x11), handle: changes::handle("carol") });
    let mut no_record = changes::admin_store();
    no_record.record = None;
    let carol = org_node::store::ProvisionalChange::Admit { invite_id: changes::invite_id_of(0x11), handle: changes::handle("carol") };
    let full = changes::filled_before(&changes::admin_store(), &carol);

    assert_eq!(refuse(changes::admin_store(), 0x99, None), OrgNodeError::NoAdmissionRequestKept { invite_id: changes::invite_id_of(0x99) });
    assert_eq!(refuse(recorded, 0x11, None), OrgNodeError::AdmissionAlreadyRecorded { invite_id: changes::invite_id_of(0x11) });
    assert_eq!(refuse(erin.clone(), 0x13, None), OrgNodeError::NoFreeHandle);
    assert!(matches!(refuse(erin.clone(), 0x13, Some("Zed")), OrgNodeError::InvalidField { field: "admission.handle", .. }));
    assert_eq!(refuse(erin, 0x13, Some("bob")), OrgNodeError::HandleTaken { handle: changes::handle("bob") });
    assert_eq!(refuse(no_record, 0x11, None), OrgNodeError::OrgNotHeld { org_id: changes::the_org() });
    assert_eq!(refuse(full, 0x11, None), OrgNodeError::ProvisionalLimit { limit: org_node::store::MAX_PROVISIONAL_BYTES });
}

// An admission that would take the epoch state over the bound is refused
// with `ProvisionalLimit`: no change is appended, no store change is
// returned and the store's plaintext is as it was. With one padding change
// fewer the same admission is recorded, so the refusal is the bound's.
// verifies: REQ-g2537c, LLR-m76r4k, LLR-mq7n6u
#[test]
fn an_admission_refused_by_the_bound_records_nothing() {
    let carol = org_node::store::ProvisionalChange::Admit { invite_id: changes::invite_id_of(0x11), handle: changes::handle("carol") };
    let full = changes::filled_before(&changes::admin_store(), &carol);
    let held = full.provisional_changes.clone();
    let mut svc = changes::service(changes::ADA, full.clone());
    let before = changes::encoded(&svc, changes::ADA);
    let refused = svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None);
    assert_eq!(
        refused.unwrap_err(),
        org_node::error::OrgNodeError::ProvisionalLimit { limit: org_node::store::MAX_PROVISIONAL_BYTES }
    );
    assert_eq!(changes::held(&svc, changes::ADA).provisional_changes, held, "no change appended");
    assert_eq!(changes::encoded(&svc, changes::ADA), before, "the store is as it was");

    let mut one_fewer = full;
    one_fewer.provisional_changes.pop().expect("the padded store holds padding");
    let mut svc = changes::service(changes::ADA, one_fewer);
    crate::support::unread(svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None).unwrap());
    assert_eq!(changes::held(&svc, changes::ADA).provisional_changes.last(), Some(&carol), "within the bound it is recorded");
}

// A revocation that would take the epoch state over the bound is refused
// with `ProvisionalLimit`: no change is appended, no store change is
// returned and the store's plaintext is as it was. With one padding change
// fewer the same revocation is recorded, so the refusal is the bound's.
// verifies: LLR-m76r4k, LLR-f3hru6
#[test]
fn a_revocation_refused_by_the_bound_records_nothing() {
    let bob = changes::bob().id;
    let revoke = org_node::store::ProvisionalChange::Revoke { member_id: bob };
    let full = changes::filled_before(&changes::admin_store(), &revoke);
    let held = full.provisional_changes.clone();
    let mut svc = changes::service(changes::ADA, full.clone());
    let before = changes::encoded(&svc, changes::ADA);
    let mut rng = CountingRng::default();
    let refused = svc.record_revocation(&mut rng, &changes::ada_device(), bob);
    assert_eq!(
        refused.unwrap_err(),
        org_node::error::OrgNodeError::ProvisionalLimit { limit: org_node::store::MAX_PROVISIONAL_BYTES }
    );
    assert_eq!(rng.drawn, 0, "the refusal draws nothing");
    assert_eq!(changes::held(&svc, changes::ADA).provisional_changes, held, "no change appended");
    assert_eq!(changes::encoded(&svc, changes::ADA), before, "the store is as it was");

    let mut one_fewer = full;
    one_fewer.provisional_changes.pop().expect("the padded store holds padding");
    let mut svc = changes::service(changes::ADA, one_fewer);
    let mut rng = CountingRng::default();
    crate::support::unread(svc.record_revocation(&mut rng, &changes::ada_device(), bob).unwrap());
    assert_eq!(changes::held(&svc, changes::ADA).provisional_changes.last(), Some(&revoke), "within the bound it is recorded");
    assert_eq!(rng.drawn, SEAL_NONCE_BYTES, "recorded, it draws only the sealed store's nonce: no key pair");
}

// Behind, both operations refuse with `OrganisationBehind`, recording
// nothing; verification in step clears the marker, and recording works again.
// verifies: LLR-mq7n6u, LLR-f3hru6, LLR-zk7fx2
#[test]
fn record_admission_and_record_revocation_refuse_while_behind() {
    let mut svc = changes::service(changes::ADA, changes::admin_store());
    let (root, key) = (changes::record_root(&svc, changes::ADA), changes::public_key(changes::RECORD_KEY));
    let (verified, change) = svc
        .verify(&mut rand::rngs::OsRng, &changes::ada_device(), Some(changes::chain(4, root, key)), org_node::VerificationTime(1), org_node::AheadLimit(1200), crate::support::none_held)
        .unwrap();
    assert_eq!(verified, org_node::Verified::Behind { chain_epoch: org_node::Epoch::new(4) });
    assert!(change.is_none(), "Behind writes nothing");
    let before = changes::encoded(&svc, changes::ADA);
    let behind = org_node::error::OrgNodeError::OrganisationBehind { org_id: changes::the_org(), chain_epoch: org_node::Epoch::new(4) };
    assert_eq!(svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None).unwrap_err(), behind);
    let mut rng = CountingRng::default();
    assert_eq!(svc.record_revocation(&mut rng, &changes::ada_device(), org_node::MemberId::new([0x02; 32])).unwrap_err(), behind);
    assert_eq!(rng.drawn, 0, "the refusal draws nothing");
    assert_eq!(changes::encoded(&svc, changes::ADA), before, "nothing recorded");
    let (verified, change) = svc
        .verify(&mut rand::rngs::OsRng, &changes::ada_device(), Some(changes::chain(3, root, key)), org_node::VerificationTime(2), org_node::AheadLimit(1200), crate::support::none_held)
        .unwrap();
    assert_eq!(verified, org_node::Verified::InStep);
    assert!(change.is_none(), "InStep writes nothing");
    crate::support::unread(svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None).unwrap());
}

// Normal and abnormal: a store put back from its file (org-io's reload after
// a failed write) keeps the behind marker the replaced store held, though the
// file never holds it, so recording stays refused until a verification clears
// it; a store put back over one not behind, or added where none was held,
// has no marker.
// verifies: LLR-zk7fx2
#[test]
fn a_store_put_back_from_its_file_keeps_the_behind_marker_the_replaced_store_held() {
    let on_disk = || org_node::test_fixtures::store_for_test(&org_node::test_fixtures::device_secret([changes::ADA; 32]), changes::admin_store());
    let mut svc = changes::service(changes::ADA, changes::admin_store());
    let (root, key) = (changes::record_root(&svc, changes::ADA), changes::public_key(changes::RECORD_KEY));
    let (verified, _) = svc
        .verify(&mut rand::rngs::OsRng, &changes::ada_device(), Some(changes::chain(4, root, key)), org_node::VerificationTime(1), org_node::AheadLimit(1200), crate::support::none_held)
        .unwrap();
    assert_eq!(verified, org_node::Verified::Behind { chain_epoch: org_node::Epoch::new(4) });

    svc.restore_store(on_disk());
    assert_eq!(changes::held(&svc, changes::ADA).behind, Some(org_node::Epoch::new(4)), "the marker survives the reload");
    let behind = org_node::error::OrgNodeError::OrganisationBehind { org_id: changes::the_org(), chain_epoch: org_node::Epoch::new(4) };
    assert_eq!(svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None).unwrap_err(), behind);

    let mut in_step = changes::service(changes::ADA, changes::admin_store());
    in_step.restore_store(on_disk());
    assert_eq!(changes::held(&in_step, changes::ADA).behind, None, "no marker to keep");

    let mut empty = org_node::service::OrgService::new(vec![]).unwrap();
    empty.restore_store(on_disk());
    assert_eq!(changes::held(&empty, changes::ADA).behind, None, "added, not replaced: no marker");
}

// A MemberId the record does not hold, one already named by a `Revoke`,
// and a store with no record are refused, recording nothing.
// verifies: LLR-f3hru6
#[test]
fn record_revocation_refuses_an_unheld_or_already_named_member() {
    let bob = org_node::MemberId::new([0x02; 32]);
    let mut svc = changes::service(changes::ADA, changes::admin_store());
    let unheld = org_node::MemberId::new([0x09; 32]);
    let mut rng = CountingRng::default();
    assert_eq!(
        svc.record_revocation(&mut rng, &changes::ada_device(), unheld).unwrap_err(),
        OrgNodeError::MemberNotHeld { member_id: unheld }
    );
    crate::support::unread(svc.record_revocation(&mut rand::rngs::OsRng, &changes::ada_device(), bob).unwrap());
    let before = changes::encoded(&svc, changes::ADA);
    assert_eq!(
        svc.record_revocation(&mut rng, &changes::ada_device(), bob).unwrap_err(),
        OrgNodeError::RevocationAlreadyRecorded { member_id: bob }
    );
    assert_eq!(changes::encoded(&svc, changes::ADA), before);
    let mut no_record = changes::admin_store();
    no_record.record = None;
    let mut svc = changes::service(changes::ADA, no_record);
    assert_eq!(
        svc.record_revocation(&mut rng, &changes::ada_device(), bob).unwrap_err(),
        OrgNodeError::OrgNotHeld { org_id: changes::the_org() }
    );
    assert_eq!(rng.drawn, 0, "no refusal draws anything");
}

// Abnormal: revoking a member the record does not hold is refused; the refusal records no change and returns no store change.
// verifies: LLR-f3hru6
#[test]
fn revoking_a_member_the_record_does_not_hold_records_nothing() {
    let mut svc = changes::service(changes::ADA, changes::admin_store());
    let before = changes::encoded(&svc, changes::ADA);
    let mut rng = CountingRng::default();
    let refused = svc.record_revocation(&mut rng, &changes::ada_device(), org_node::MemberId::new([0x0e; 32]));
    assert!(matches!(refused, Err(org_node::error::OrgNodeError::MemberNotHeld { .. })));
    assert_eq!(rng.drawn, 0, "the refusal draws nothing");
    assert!(changes::held(&svc, changes::ADA).provisional_changes.is_empty());
    assert_eq!(changes::encoded(&svc, changes::ADA), before);
}

// ---- Proposing the updated Organisation, and the read-only views ---------

// Every provisional change is applied in recorded order: Carol is added,
// Bob deleted, and one `Own` provisional organisation for epoch 4 is kept
// with the resulting members and root, the record's root as its base, the
// record's members as its base snapshot, the Change set from the record,
// the drawn key and a copy of the changes it applied.
// verifies: LLR-va2yqz
#[test]
fn propose_updated_organisation_applies_every_change_in_order_and_keeps_one_own_provisional_organisation() {
    let mut svc = changes::service(changes::ADA, changes::admin_store());
    crate::support::unread(svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None).unwrap());
    crate::support::unread(svc.record_revocation(&mut rand::rngs::OsRng, &changes::ada_device(), org_node::MemberId::new([0x02; 32])).unwrap());
    let record = changes::held(&svc, changes::ADA).record.clone().unwrap();
    crate::support::unread(svc.propose_updated_organisation(&mut changes::TwoRng::new(0x22, 0x4a), &changes::ada_device()).unwrap());

    let store_data = changes::held(&svc, changes::ADA);
    assert_eq!(store_data.provisional_organisations.len(), 1);
    let proposal = &store_data.provisional_organisations[0];
    assert!(matches!(proposal.origin, org_node::store::Origin::Own));
    assert_eq!(proposal.epoch, org_node::Epoch::new(4));
    assert_eq!(proposal.base_root, record.root_hash);
    let mut ids: Vec<_> = proposal.members.iter().map(|member| member.id).collect();
    ids.sort();
    assert_eq!(ids, vec![org_node::MemberId::new([0x01; 32]), org_node::MemberId::new([0x22; 32])], "Ada kept, Carol added, Bob gone");
    assert_eq!(proposal.root, changes::trie_of_members(&proposal.members).root_hash().unwrap());
    assert_eq!(proposal.org_pub_key, changes::public_key([0x4a; 32]));
    let delta: org_members::delta::Delta = postcard::from_bytes(proposal.change_set.as_ref().unwrap()).unwrap();
    assert_eq!(delta.base_root(), &record.root_hash, "the Change set extends the record");
    assert_eq!(proposal.base_snapshot.clone().unwrap(), postcard::to_allocvec(&record.trie_members).unwrap());
    assert_eq!(proposal.applied, store_data.provisional_changes);
}

// The provisional changes stay held, the proposal keeps the changes it
// applied, and the current record is unchanged.
// verifies: LLR-va2yqz
#[test]
fn the_proposal_keeps_its_applied_changes_and_the_changes_stay_held() {
    let mut svc = changes::service(changes::ADA, changes::admin_store());
    crate::support::unread(svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None).unwrap());
    let record = postcard::to_allocvec(changes::held(&svc, changes::ADA).record.as_ref().unwrap()).unwrap();
    let recorded = changes::held(&svc, changes::ADA).provisional_changes.clone();
    crate::support::unread(svc.propose_updated_organisation(&mut rand::rngs::OsRng, &changes::ada_device()).unwrap());
    assert_eq!(changes::held(&svc, changes::ADA).provisional_changes, recorded);
    assert_eq!(changes::held(&svc, changes::ADA).provisional_organisations[0].applied, recorded);
    assert_eq!(postcard::to_allocvec(changes::held(&svc, changes::ADA).record.as_ref().unwrap()).unwrap(), record);
}

// A drawn key whose public half is the record's Organisation public key
// (draw `0x3d`) or Carol's Member-as-a-group key in the result (draw
// `0x63`) is refused with `DuplicateOrgPublicKey`, keeping nothing; a fresh
// draw is then accepted.
// verifies: LLR-qghs95
#[test]
fn a_key_equal_to_the_records_or_to_a_key_of_the_result_is_refused_keeping_nothing() {
    let mut svc = changes::service(changes::ADA, changes::admin_store());
    crate::support::unread(svc.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None).unwrap());
    let before = changes::encoded(&svc, changes::ADA);
    for then in [0x3d, 0x63] {
        let refused = svc.propose_updated_organisation(&mut changes::TwoRng::new(0x22, then), &changes::ada_device());
        assert_eq!(
            refused.unwrap_err(),
            org_node::error::OrgNodeError::DuplicateOrgPublicKey,
            "draw {then:#x}"
        );
        assert_eq!(changes::encoded(&svc, changes::ADA), before, "nothing kept");
    }
    crate::support::unread(svc.propose_updated_organisation(&mut rand::rngs::OsRng, &changes::ada_device()).unwrap());
}

// No provisional change, no record, the behind marker, a change
// org-members refuses (Erin's request carries Bob's Device key) and the
// bound are each refused, keeping nothing.
// verifies: LLR-n6ssue
#[test]
fn propose_updated_organisation_refuses_no_change_no_record_behind_and_the_bound() {
    let refuse = |svc: &mut org_node::service::OrgService| {
        let before = changes::encoded(svc, changes::ADA);
        let refused = svc.propose_updated_organisation(&mut rand::rngs::OsRng, &changes::ada_device()).unwrap_err();
        assert_eq!(changes::encoded(svc, changes::ADA), before, "nothing kept");
        refused
    };
    assert_eq!(refuse(&mut changes::service(changes::ADA, changes::admin_store())), OrgNodeError::NoProvisionalChange);

    let mut no_record = changes::admin_store();
    no_record.record = None;
    no_record.provisional_changes.push(org_node::store::ProvisionalChange::Revoke { member_id: org_node::MemberId::new([0x02; 32]) });
    assert_eq!(refuse(&mut changes::service(changes::ADA, no_record)), OrgNodeError::OrgNotHeld { org_id: changes::the_org() });

    let mut behind = changes::service(changes::ADA, changes::admin_store());
    crate::support::unread(behind.record_admission(&mut rand::rngs::OsRng, &changes::ada_device(), changes::invite_id_of(0x11), None).unwrap());
    let state = changes::chain(4, changes::record_root(&behind, changes::ADA), changes::public_key(changes::RECORD_KEY));
    let (verified, change) = behind
        .verify(&mut rand::rngs::OsRng, &changes::ada_device(), Some(state), org_node::VerificationTime(1), org_node::AheadLimit(1200), crate::support::none_held)
        .unwrap();
    assert!(matches!(verified, org_node::Verified::Behind { .. }) && change.is_none(), "fixture: behind, {verified:?}");
    assert_eq!(
        refuse(&mut behind),
        OrgNodeError::OrganisationBehind { org_id: changes::the_org(), chain_epoch: org_node::Epoch::new(4) }
    );

    let mut duplicate = changes::admin_store();
    duplicate.kept_admission_requests.push(changes::request(0x14, "Erin", 0x65, changes::BOB, &["erin"]));
    duplicate.outstanding_invites.push(changes::outstanding(0x14, "Erin"));
    duplicate.provisional_changes.push(org_node::store::ProvisionalChange::Admit { invite_id: changes::invite_id_of(0x14), handle: changes::handle("erin") });
    assert_eq!(
        refuse(&mut changes::service(changes::ADA, duplicate)),
        OrgNodeError::Trie(org_members::OrgMembersError::DuplicateKey)
    );

    let one = changes::held(&changes::proposed(), changes::ADA).clone();
    let copies = |count: usize| {
        let mut filled = one.clone();
        filled.provisional_organisations = vec![one.provisional_organisations[0].clone(); count];
        filled
    };
    let max = org_node::store::MAX_PROVISIONAL_BYTES;
    let count = changes::fewest_exceeding(|count| changes::bound_len(&copies(count))) - 1;
    assert_eq!(refuse(&mut changes::service(changes::ADA, copies(count))), OrgNodeError::ProvisionalLimit { limit: max });
}

// The views hold no private key and change nothing; the `Own` proposal
// of the current record and changes is marked current.
// verifies: LLR-8ru6mv
#[test]
fn the_views_hold_no_private_key_and_mark_the_current_proposal() {
    let svc = changes::proposed();
    let before = changes::encoded(&svc, changes::ADA);
    let views = svc.provisional_organisations(&changes::ada_device()).unwrap();
    assert_eq!(views.len(), 1);
    let proposal = &changes::held(&svc, changes::ADA).provisional_organisations[0];
    let view = &views[0];
    assert!(view.current_proposal);
    assert!(matches!(view.origin, org_node::store::Origin::Own));
    assert_eq!((view.epoch, view.base_epoch), (org_node::Epoch::new(4), org_node::Epoch::new(3)));
    assert_eq!((view.root, view.org_pub_key), (proposal.root, proposal.org_pub_key));
    let rendered = format!("{views:?}");
    assert!(!rendered.contains("OrgPrivateKey"), "no private key in a view: {rendered}");
    assert_eq!(svc.provisional_changes(&changes::ada_device()).unwrap(), changes::held(&svc, changes::ADA).provisional_changes.as_slice());
    assert_eq!(changes::encoded(&svc, changes::ADA), before);
}

// A change recorded after the proposal, or a record that is no longer
// its base, makes it no longer the current proposal.
// verifies: LLR-8ru6mv
#[test]
fn current_proposal_turns_false_after_a_later_change_or_a_new_record() {
    let mut svc = changes::proposed();
    crate::support::unread(svc.record_revocation(&mut rand::rngs::OsRng, &changes::ada_device(), org_node::MemberId::new([0x02; 32])).unwrap());
    assert!(!svc.provisional_organisations(&changes::ada_device()).unwrap()[0].current_proposal, "a later change");

    let mut moved = changes::held(&changes::proposed(), changes::ADA).clone();
    let record = moved.record.as_mut().unwrap();
    record.epoch = org_node::Epoch::new(4);
    record.last_seq = org_node::SequenceNumber::new(4);
    let svc = changes::service(changes::ADA, moved);
    assert!(!svc.provisional_organisations(&changes::ada_device()).unwrap()[0].current_proposal, "a new record");
}

// Each kept admission request is returned in kept order with the invitee
// name its outstanding invite keeps.
// verifies: LLR-8ru6mv
#[test]
fn admission_requests_pairs_each_kept_request_with_its_invitee_name() {
    let svc = changes::service(changes::ADA, changes::admin_store());
    let pairs: Vec<_> = svc
        .admission_requests(&changes::ada_device())
        .unwrap()
        .into_iter()
        .map(|(request, invitee_name)| (request.invite_id, invitee_name.to_string()))
        .collect();
    assert_eq!(pairs, vec![(changes::invite_id_of(0x11), "Carol".to_string()), (changes::invite_id_of(0x12), "Dave".to_string())]);
}

// The member an `Admit` adds carries the kept request's values, the
// Handle chosen for it and a MemberId drawn from the caller's source.
// verifies: LLR-va2yqz, LLR-bwtx3h
#[test]
fn an_admitted_member_carries_exactly_the_kept_requests_values() {
    let svc = changes::proposed();
    let request = changes::request(0x11, "Carol", 0x63, changes::CAROL, &["bob", "carol", "cee"]);
    let proposal = &changes::held(&svc, changes::ADA).provisional_organisations[0];
    let carol = proposal.members.iter().find(|member| member.id == org_node::MemberId::new([0x22; 32])).unwrap();
    assert_eq!(carol.handle, changes::handle("carol"));
    assert_eq!((&carol.name, &carol.surname), (&request.name, &request.surname));
    assert_eq!(carol.member_key, request.member_key);
    assert_eq!(carol.device_keys, vec![request.device_key]);
}

// The three views read the store and change none of its bytes, and
// repeat.
// verifies: LLR-8ru6mv
#[test]
fn reading_provisional_changes_and_organisations_changes_nothing() {
    let svc = changes::proposed();
    let before = changes::encoded(&svc, changes::ADA);
    let first = (
        svc.provisional_changes(&changes::ada_device()).unwrap().to_vec(),
        svc.provisional_organisations(&changes::ada_device()).unwrap(),
        svc.admission_requests(&changes::ada_device()).unwrap().len(),
    );
    let second = (
        svc.provisional_changes(&changes::ada_device()).unwrap().to_vec(),
        svc.provisional_organisations(&changes::ada_device()).unwrap(),
        svc.admission_requests(&changes::ada_device()).unwrap().len(),
    );
    assert_eq!(first, second);
    assert_eq!(changes::encoded(&svc, changes::ADA), before);
}

// Abnormal: each per-store view refuses a Device key no held store names
// with `PersonaNotHeld` naming that key, and reading changes nothing.
// verifies: LLR-8ru6mv
#[test]
fn every_per_store_view_refuses_an_unknown_device_key_as_persona_not_held() {
    let svc = changes::proposed();
    let before = changes::encoded(&svc, changes::ADA);
    let unknown = device_key(0x77);
    let not_held = Some(OrgNodeError::PersonaNotHeld { device_key: Box::new(unknown) });
    assert_eq!(svc.provisional_changes(&unknown).err(), not_held, "provisional_changes");
    assert_eq!(svc.provisional_organisations(&unknown).err(), not_held, "provisional_organisations");
    assert_eq!(svc.admission_requests(&unknown).err(), not_held, "admission_requests");
    assert_eq!(svc.persona_view(&unknown).err(), not_held, "persona_view");
    assert_eq!(changes::encoded(&svc, changes::ADA), before, "reading changes nothing");
}

// For a revocation, a draw equal to the record's key (`0x3d`) or to
// Ada's Member-as-a-group key, left in the result (`0x61`), is refused,
// the record's keys unchanged; a fresh draw is accepted.
// verifies: LLR-qghs95
#[test]
fn an_updated_organisation_key_equal_to_the_records_or_to_a_key_of_the_result_is_refused() {
    let mut svc = changes::service(changes::ADA, changes::admin_store());
    crate::support::unread(svc.record_revocation(&mut rand::rngs::OsRng, &changes::ada_device(), org_node::MemberId::new([0x02; 32])).unwrap());
    let record = postcard::to_allocvec(changes::held(&svc, changes::ADA).record.as_ref().unwrap()).unwrap();
    for draw in [0x3d, 0x61] {
        let refused = svc.propose_updated_organisation(&mut changes::TwoRng::new(draw, draw), &changes::ada_device());
        assert_eq!(
            refused.unwrap_err(),
            org_node::error::OrgNodeError::DuplicateOrgPublicKey,
            "draw {draw:#x}"
        );
        assert!(changes::held(&svc, changes::ADA).provisional_organisations.is_empty());
        assert_eq!(postcard::to_allocvec(changes::held(&svc, changes::ADA).record.as_ref().unwrap()).unwrap(), record);
    }
    crate::support::unread(svc.propose_updated_organisation(&mut rand::rngs::OsRng, &changes::ada_device()).unwrap());
    assert_ne!(changes::held(&svc, changes::ADA).provisional_organisations[0].org_pub_key, changes::public_key(changes::RECORD_KEY));
}

// The Persona view of a record store, of a founding store with its proxy
// recorded and of a provisional Member's store copies their public values
// (the binding, the Organisation name, the record's, the founding's and
// the Organisation the Member asked to join), holds no seed, secret or
// private key, and changes nothing; an unknown Device key is refused.
// verifies: LLR-8ru6mv
#[test]
fn persona_view_copies_each_stores_public_values_and_no_key() {
    let admin = changes::service(changes::ADA, changes::admin_store());
    let before = changes::encoded(&admin, changes::ADA);
    let view = admin.persona_view(&changes::ada_device()).unwrap();
    let store_data = changes::held(&admin, changes::ADA);
    let record = store_data.record.as_ref().unwrap();
    assert_eq!((view.device_key, view.org_id), (changes::ada_device(), store_data.persona.org_id));
    assert_eq!(view.org_name, store_data.org_name);
    let held = view.record.as_ref().expect("the record's public values");
    assert_eq!(
        (held.org_id, held.root_hash, held.org_pub_key, held.epoch),
        (record.org_id, record.root_hash, record.org_pub_key, record.epoch)
    );
    assert_eq!(held.members, record.trie_members);
    assert!(view.founding.is_none() && view.member_of.is_none());
    assert_eq!(changes::encoded(&admin, changes::ADA), before, "reading changes nothing");

    let mut svc = storeless_service();
    let (founder, _, _) = alice_founding(&mut svc);
    let (org_id, proxy) = (OrgId::new([7; 20]), ChainAccount::new([0x5a; 32]));
    crate::support::unread(svc.record_founding_proxy(&mut OsRng, &founder, org_id, proxy).unwrap());
    let invite = invite_to(OrgId::new([9; 20]), device_key(0x23));
    let (member, _, _) = join(&mut svc, &invite).unwrap();
    let before = support::encoded_stores(&svc);
    let founding_view = svc.persona_view(&founder).unwrap();
    let founding = svc.store_data(&founder).unwrap().founding.clone().unwrap();
    assert_eq!(founding_view.org_id, Some(org_id));
    assert!(founding_view.record.is_none() && founding_view.member_of.is_none());
    let held = founding_view.founding.as_ref().expect("the founding's public values");
    assert_eq!(
        (held.epoch, held.root, held.org_pub_key, held.proxy),
        (founding.epoch, founding.root, founding.org_pub_key, Some(proxy))
    );
    let member_view = svc.persona_view(&member).unwrap();
    assert_eq!((member_view.member_of, member_view.org_name.as_str()), (Some(invite.org_id), "Acme"));
    assert!(member_view.record.is_none() && member_view.founding.is_none());
    for rendered in [format!("{view:?}"), format!("{founding_view:?}"), format!("{member_view:?}")] {
        for secret in ["OrgPrivateKey", "MemberSeed", "DeviceSecret"] {
            assert!(!rendered.contains(secret), "no {secret} in a view: {rendered}");
        }
    }
    assert_eq!(svc.persona_view(&device_key(0x77)).err(), Some(OrgNodeError::PersonaNotHeld { device_key: Box::new(device_key(0x77)) }));
    assert_eq!(support::encoded_stores(&svc), before, "reading changes nothing");
}

// `persona_records` lists every Persona record's public fields in
// `list_personas`'s order, and `PersonStore::view` the Person store's; no
// seed is copied and reading changes nothing.
// verifies: LLR-8ru6mv
#[test]
fn persona_records_and_the_person_view_copy_the_public_fields_and_no_seed() {
    let mut svc = storeless_service();
    let (founder, _, _) = alice_founding(&mut svc);
    let invite = invite_to(OrgId::new([9; 20]), device_key(0x23));
    let (member, _, _) = join(&mut svc, &invite).unwrap();
    let before = support::encoded_stores(&svc);
    let records = svc.persona_records();
    let listed = svc.list_personas();
    assert_eq!(records.iter().map(|record| record.device_key).collect::<Vec<_>>(), vec![founder, member]);
    assert_eq!(records.len(), listed.len());
    for (record, persona) in records.iter().zip(listed) {
        assert_eq!(
            (&record.persona_id, record.org_id, &record.name, &record.surname, &record.handle),
            (&persona.persona_id, persona.org_id, &persona.name, &persona.surname, &persona.handle)
        );
        assert_eq!(
            (record.device_key, record.member_id, record.status),
            (persona.device_key, persona.member_id, persona.status)
        );
        let rendered = format!("{record:?}");
        assert!(!rendered.contains("seed") && !rendered.contains("MemberSeed"), "no seed in a record view: {rendered}");
    }
    assert_eq!(support::encoded_stores(&svc), before, "reading changes nothing");

    let (person, _, _) = org_node::PersonStore::create(&mut OsRng, "Ann", "Lee").unwrap();
    let view = person.view();
    let person_content = person.content();
    assert_eq!((&view.person, view.device_key, view.person_key), (&person_content.person, person_content.device_key, person_content.person_key));
    let rendered = format!("{view:?}");
    assert!(!rendered.contains("seed"), "no Person seed in the Person view: {rendered}");
}
