#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The node's own removal (SDD-z4m57w): the removed store form, the decision and the signing in `verify`,
//! and the kept acknowledgements.

mod support;

use org_members::{DevicePublicKey, MemberId, Name, RootHash, Surname};
use org_node::invitation::{EndpointAddresses, InviteId};
use org_node::removal::{KeptAcknowledgement, RemovedStore, StoreContent};
use org_node::revocation::{Acknowledgement, RevocationNotice, Signature64};
use org_node::service::{KeptAcknowledgements, OrgService, StoreChange};
use org_node::store::{
    KeptAdmissionRequest, KeptNotice, MemberSnapshot, Origin, OutstandingInvite, PersonaStatus,
    PersonaStore, ProvisionalChange, ProvisionalOrganisation, ProvisionalOrganisationParts, RevokedDevice, SealedStore,
    StoreData, StoreWrite,
};
use org_node::test_fixtures::{
    device_key, device_secret, empty_store, member_key, persona_record, seal_plaintext_for_test, store_for_test,
};
use org_node::verify_org::{VerificationTime, Verified};
use org_node::wire::{decode_message, encode_message, WireMessage};
use org_node::{DeviceSecret, Epoch, OrgNodeError, OrgPrivateKey, OrgPublicKey, OrgState};
use rand::rngs::OsRng;
use support::fixtures::{
    acme_store, addresses, alice, alice_persona, own, own_encoded, own_service, record, snapshot, OWN, RECORD_KEY,
};
use support::{handle, never_asked, the_org, trie_of_members};

fn own_secret() -> DeviceSecret {
    device_secret([OWN; 32])
}
fn org_key_pair(seed: u8) -> (OrgPrivateKey, OrgPublicKey) {
    let private = OrgPrivateKey::from([seed; 32]);
    let public = private.x25519_keypair().org_public_key().unwrap();
    (private, public)
}

fn root_of(members: &[MemberSnapshot]) -> RootHash {
    trie_of_members(members).root_hash().unwrap()
}

/// Alice's Active store, whose record (epoch 3) lists this Device and `others`.
fn member_data(others: Vec<MemberSnapshot>) -> StoreData {
    let mut members = vec![alice()];
    members.extend(others);
    acme_store(alice_persona(), record(3, &members))
}

/// The rich Active store LLR-f9mcsq's test drives: a record listing this Device, Bob (0x21) and Carol
/// (0x22); a provisional change and organisation; a kept admission request and revocation notice; an
/// outstanding invite; an address-book entry; a revoked Device.
fn rich_data() -> StoreData {
    let mut store_data = member_data(vec![snapshot(2, "bob", 0x11, &[0x21]), snapshot(3, "carol", 0x12, &[0x22])]);
    let invite_id = InviteId::new([0x31; 32]);
    store_data.provisional_changes = vec![ProvisionalChange::Revoke { member_id: MemberId::new([2; 32]) }];
    let next = vec![alice(), snapshot(3, "carol", 0x12, &[0x22])];
    let (private, public) = org_key_pair(0x62);
    store_data.provisional_organisations = vec![ProvisionalOrganisation::from(ProvisionalOrganisationParts {
        epoch: Epoch::new(4),
        base_root: store_data.record.as_ref().unwrap().root_hash,
        root: root_of(&next),
        members: next,
        org_pub_key: public,
        org_private_key: private,
        change_set: Some(vec![9, 8]),
        base_snapshot: Some(vec![7]),
        origin: Origin::Own,
        applied: store_data.provisional_changes.clone(),
        first_ahead_at: None,
    })];
    store_data.outstanding_invites = vec![OutstandingInvite { org_id: the_org(), invite_id, invitee_name: "Dave".into() }];
    store_data.kept_admission_requests = vec![KeptAdmissionRequest {
        org_id: the_org(),
        invite_id,
        from: device_key(0x23),
        name: Name::parse("Dave").unwrap(),
        surname: Surname::parse("Day").unwrap(),
        preferred_handles: vec![handle("dave")],
        member_key: member_key(0x13),
        device_key: device_key(0x23),
        addresses: addresses(&["192.0.2.23:4433"]),
    }];
    store_data.kept_revocation_notices = vec![KeptNotice { notice: notice_removing_own(&store_data), from: device_key(0x21) }];
    store_data.address_book = vec![(device_key(0x21), addresses(&["192.0.2.21:4433"]))];
    store_data.revoked_devices = vec![RevokedDevice { member_id: MemberId::new([9; 32]), device_key: device_key(0x29) }];
    store_data
}

/// A notice for this Device, its proof from the record without this Persona's Member.
fn notice_removing_own(store_data: &StoreData) -> RevocationNotice {
    let without = removed_members(store_data);
    let proof = trie_of_members(&without).prove_absent(&MemberId::new([1; 32]), &own()).unwrap();
    RevocationNotice { org_id: the_org(), member_id: MemberId::new([1; 32]), device: own(), proof }
}
fn removed_members(store_data: &StoreData) -> Vec<MemberSnapshot> {
    store_data.record.as_ref().unwrap().trie_members.iter().filter(|member| !member.device_keys.contains(&own())).cloned().collect()
}


fn sample_ack(signature: u8) -> Acknowledgement {
    Acknowledgement {
        org_id: the_org(),
        member_id: MemberId::new([1; 32]),
        device: own(),
        epoch: Epoch::new(4),
        root: RootHash::new([0x66; 32]),
        signature: Signature64([signature; 64]),
    }
}

fn kept(recipient: u8, addresses: Option<EndpointAddresses>) -> KeptAcknowledgement {
    KeptAcknowledgement { ack: sample_ack(0x5a), recipient: device_key(recipient), recipient_addresses: addresses }
}

fn removed_store() -> RemovedStore {
    RemovedStore {
        org_id: the_org(),
        verified_at: VerificationTime(1_760_000_000),
        acknowledgements: vec![kept(0x21, Some(addresses(&["198.51.100.7:4433"]))), kept(0x22, None)],
    }
}

/// `plaintext` sealed under this Device's store key, as org-io's file would hold it.
fn sealed(plaintext: &[u8]) -> SealedStore {
    seal_plaintext_for_test(&own_secret(), own(), plaintext, &mut OsRng)
}
fn removed_plaintext(store: &RemovedStore) -> Vec<u8> {
    let mut bytes = vec![1u8];
    bytes.extend(postcard::to_allocvec(store).unwrap());
    bytes
}
/// A service over the removed store `store`, as `unseal` opens it.
fn removed_service(store: &RemovedStore) -> OrgService {
    let opened = PersonaStore::unseal(&sealed(&removed_plaintext(store)), own(), own_secret()).unwrap();
    // Built here: a removed store is opened from sealed bytes, which no StoreData fixture holds.
    OrgService::new(vec![opened]).unwrap()
}

/// The content a write sealed, opened with this Device's secret.
fn content_of(write: &StoreWrite) -> StoreContent {
    PersonaStore::unseal(&write.sealed, write.device_key, own_secret()).unwrap().content().clone()
}

// verifies: LLR-6zaeb6
#[test]
fn store_content_encodes_its_variant_index_then_its_fields() {
    let store_data = member_data(vec![]);
    let active = postcard::to_allocvec(&StoreContent::Active(store_data.clone())).unwrap();
    assert_eq!(active[0], 0, "Active is variant 0");
    assert_eq!(&active[1..], postcard::to_allocvec(&store_data).unwrap().as_slice(), "then the store data");

    let removed = removed_store();
    let bytes = postcard::to_allocvec(&StoreContent::Removed(removed.clone())).unwrap();
    assert_eq!(bytes[0], 1, "Removed is variant 1");
    let mut expected = the_org().as_bytes().to_vec();
    expected.extend(postcard::to_allocvec(&removed.verified_at).unwrap());
    expected.extend(postcard::to_allocvec(&removed.acknowledgements).unwrap());
    assert_eq!(&bytes[1..], expected.as_slice(), "org_id, verified_at, acknowledgements, in that order");
    let second = postcard::to_allocvec(&removed.acknowledgements[1]).unwrap();
    let mut fields = postcard::to_allocvec(&removed.acknowledgements[1].ack).unwrap();
    fields.extend(device_key(0x22).as_bytes());
    fields.push(0); // recipient_addresses: None
    assert_eq!(second, fields, "ack, recipient, recipient_addresses, in that order");
}

// verifies: LLR-6zaeb6
#[test]
fn unseal_reparses_a_removed_forms_keys_and_addresses_and_refuses_an_unknown_variant() {
    let removed = removed_store();
    let svc = removed_service(&removed);
    assert_eq!(svc.store_content(&own()), Some(&StoreContent::Removed(removed.clone())));

    // A recipient key that is no curve point (y = 2 has no x on edwards25519): refused naming the field.
    let mut no_point = [0u8; 32];
    no_point[0] = 2;
    let mut plaintext = removed_plaintext(&removed);
    let recipient = device_key(0x21);
    let at = plaintext.windows(32).position(|window| window == recipient.as_bytes()).unwrap();
    plaintext[at..at + 32].copy_from_slice(&no_point);
    assert!(matches!(
        PersonaStore::unseal(&sealed(&plaintext), own(), own_secret()),
        Err(OrgNodeError::InvalidField { field, .. }) if field.starts_with("removed.")
    ));
    // Addresses its parse refuses (seventeen entries): refused naming the removed field.
    let mut over = removed.clone();
    over.acknowledgements[0].recipient_addresses = None;
    let mut plaintext = removed_plaintext(&over);
    let tag = plaintext.len() - postcard::to_allocvec(&over.acknowledgements[1]).unwrap().len() - 1;
    assert_eq!(plaintext[tag], 0, "fixture: the first recipient_addresses is None");
    let seventeen: Vec<String> = (0..17).map(|port| format!("192.0.2.1:{port}")).collect();
    let mut some_seventeen = vec![1u8];
    some_seventeen.extend(postcard::to_allocvec(&seventeen).unwrap());
    plaintext.splice(tag..tag + 1, some_seventeen);
    assert!(matches!(
        PersonaStore::unseal(&sealed(&plaintext), own(), own_secret()),
        Err(OrgNodeError::InvalidField { field, .. }) if field.starts_with("removed.")
    ));
    // An unknown variant index.
    let mut plaintext = removed_plaintext(&removed);
    plaintext[0] = 2;
    assert!(matches!(
        PersonaStore::unseal(&sealed(&plaintext), own(), own_secret()),
        Err(OrgNodeError::InvalidField { field: "store", .. })
    ));
}

/// The rich store's own removal through `verify`: its kept notice accepted against a chain state at
/// epoch 4 whose root lists Bob and Carol but not this Device, signed with this Persona's secret.
fn remove_rich(svc: &mut OrgService, verified_at: VerificationTime) -> (Verified, Option<StoreChange>) {
    let without = removed_members(svc.store_data(&own()).unwrap());
    svc.verify(&mut OsRng, &own(), Some(chain_listing(&without, 4, 0x63)), verified_at, org_node::AheadLimit(1200), |_: &[DevicePublicKey]| vec![own_secret()])
        .unwrap()
}

// The removed form is the one `verify`'s removal of the rich store seals.
// verifies: LLR-2d5a7m
#[test]
fn into_removed_keeps_only_the_org_id_the_time_and_the_acknowledgements() {
    let mut svc = own_service(rich_data());
    let (_, change) = remove_rich(&mut svc, VerificationTime(42));
    let removed = removed_of(change);
    assert_eq!((removed.org_id, removed.verified_at), (the_org(), VerificationTime(42)));
    assert_eq!(removed.acknowledgements.len(), 1, "two candidates: one recipient");
    let bytes = postcard::to_allocvec(&removed).unwrap();
    let model = serde_json::to_value(&removed).unwrap();
    // serde_json's map is sorted by key; the field order is the first test's.
    let fields: Vec<&str> = model.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(fields, ["acknowledgements", "org_id", "verified_at"], "nothing else is held");
    // The record's Organisation private key and the provisional organisation's are not kept.
    for gone in [RECORD_KEY, [0x62; 32]] {
        assert!(!bytes.windows(32).any(|window| window == gone), "no Organisation private key is kept");
    }
    // The member seed's bytes, [1; 32], are also this Persona's MemberId, which each acknowledgement
    // names once: they occur in the removed form exactly that often, so the seed itself is not kept.
    let member_seed = [1u8; 32];
    let named = removed.acknowledgements.iter().filter(|kept| kept.ack.member_id.as_bytes() == &member_seed).count();
    assert_eq!(bytes.windows(32).filter(|window| *window == member_seed).count(), named, "the member seed is not kept");
    assert!(!bytes.windows(4).any(|window| window == b"Acme"), "the Organisation name is not kept");
}

// A store with no record has nothing to remove: `OrgNotHeld` when it is bound to an Organisation,
// `OrgNotOnChain` when it is bound to none (merge review MR3, finding-7).
// verifies: LLR-2d5a7m
#[test]
fn into_removed_refuses_a_store_with_no_record() {
    let mut bound = member_data(vec![]);
    bound.record = None;
    bound.persona.org_id = Some(the_org());
    assert_eq!(
        bound.into_removed(vec![kept(0x21, None)], VerificationTime(42)).unwrap_err(),
        OrgNodeError::OrgNotHeld { org_id: the_org() }
    );
    let mut unbound = member_data(vec![]);
    unbound.record = None;
    unbound.persona.org_id = None;
    assert_eq!(unbound.into_removed(vec![kept(0x21, None)], VerificationTime(42)).unwrap_err(), OrgNodeError::OrgNotOnChain);
}

// The removal is `verify`'s, which LLR-2d5a7m names.
// verifies: LLR-2d5a7m
#[test]
fn a_removal_seals_a_removed_store_for_that_personas_device_key() {
    let other_secret = device_secret([0x44; 32]);
    let mut other = persona_record("p-other", 0x41, 0x44);
    other.status = PersonaStatus::Proposed;
    let other_data = empty_store(other);
    let mut svc = OrgService::new(vec![
        store_for_test(&own_secret(), rich_data()),
        store_for_test(&other_secret, other_data.clone()),
    ])
    .unwrap();
    let (verified, change) = remove_rich(&mut svc, VerificationTime(7));
    assert!(matches!(verified, Verified::Removed { org_id, acknowledgements_kept: 1 } if org_id == the_org()), "{verified:?}");
    let StoreChange::Sealed(write) = change.expect("the removal's one store change");
    assert_eq!(write.device_key, own(), "sealed for that Persona's Device key");
    let StoreContent::Removed(removed) = content_of(&write) else { panic!("the removed form is sealed") };
    assert_eq!((removed.org_id, removed.verified_at, removed.acknowledgements.len()), (the_org(), VerificationTime(7), 1));
    assert!([device_key(0x21), device_key(0x22)].contains(&removed.acknowledgements[0].recipient), "one of the record's others");
    assert_eq!(svc.store_content(&own()), Some(&StoreContent::Removed(removed)), "the held store is replaced");
    assert_eq!(
        postcard::to_allocvec(svc.store_data(&device_key(0x44)).unwrap()).unwrap(),
        postcard::to_allocvec(&other_data).unwrap(),
        "no other store is read or changed"
    );
}

// Abnormal: a store put back from its file over a held store in the removed
// form (org-io's reload after a failed write of a removal) replaces it and
// has no behind marker: the removed form holds no marker to keep.
// verifies: LLR-zk7fx2
#[test]
fn a_store_put_back_over_a_removed_store_replaces_it_with_no_behind_marker() {
    let mut svc = removed_service(&removed_store());
    assert!(matches!(svc.store_content(&own()), Some(StoreContent::Removed(_))));
    svc.restore_store(store_for_test(&own_secret(), member_data(vec![])));
    assert!(matches!(svc.store_content(&own()), Some(StoreContent::Active(_))), "replaced, not kept");
    assert_eq!(svc.store_data(&own()).unwrap().behind, None, "the removed form held no marker");
}

// verifies: LLR-myfk8w
#[test]
fn every_other_operation_on_a_removed_store_is_organisation_removed() {
    let removed = removed_store();
    let mut svc = removed_service(&removed);
    let removed_error = || OrgNodeError::OrganisationRemoved { org_id: the_org() };
    let is_removed = |error: OrgNodeError| assert_eq!(error, removed_error());
    let removed_device = own();
    let invite_id = InviteId::new([0x31; 32]);
    is_removed(svc.persona_view(&removed_device).unwrap_err());
    let state = OrgState::from_chain([0x66; 32], *org_key_pair(0x63).1.as_bytes(), 5).unwrap();
    let body = encode_message(&WireMessage::Acknowledgement(sample_ack(0x5a))).unwrap();

    is_removed(svc.record_founding_proxy(&mut OsRng, &removed_device, the_org(), org_node::ChainAccount::new([0x55; 32])).unwrap_err());
    is_removed(svc.record_admission(&mut OsRng, &removed_device, invite_id, None).unwrap_err());
    is_removed(svc.record_revocation(&mut OsRng, &removed_device, MemberId::new([2; 32])).unwrap_err());
    is_removed(svc.propose_updated_organisation(&mut OsRng, &removed_device).unwrap_err());
    is_removed(svc.verify(&mut OsRng, &removed_device, Some(state), VerificationTime(9), org_node::AheadLimit(1200), never_asked).unwrap_err());
    is_removed(svc.check_received(&mut OsRng, &removed_device, device_key(0x21), &body).unwrap_err());
    is_removed(svc.build_invite(&mut OsRng, &removed_device, "Eve".into(), vec![]).unwrap_err());
    is_removed(svc.cancel_invite(&mut OsRng, &removed_device, invite_id).unwrap_err());
    is_removed(svc.build_admission_request(&removed_device, vec![]).unwrap_err());
    is_removed(svc.build_organisation_request(&removed_device, vec![]).unwrap_err());
    is_removed(svc.messages_for_current_epoch(&removed_device).unwrap_err());
    is_removed(svc.addresses_for(&removed_device, device_key(0x21)).unwrap_err());
    is_removed(svc.provisional_changes(&removed_device).unwrap_err());
    is_removed(svc.provisional_organisations(&removed_device).unwrap_err());
    is_removed(svc.admission_requests(&removed_device).unwrap_err());

    assert_eq!(svc.store_content(&removed_device), Some(&StoreContent::Removed(removed)), "nothing changed");
    assert!(
        matches!(svc.store_data(&removed_device), Err(OrgNodeError::OrganisationRemoved { org_id }) if org_id == the_org()),
        "a removed store holds no store data"
    );
    assert!(svc.drop_store(&removed_device), "drop_store still removes it");
}

// Abnormal: the removed Devices of a removed store are refused with
// `OrganisationRemoved` for its Organisation, and nothing changes.
// verifies: LLR-dsjht5
#[test]
fn the_removed_devices_of_a_removed_store_are_organisation_removed() {
    let removed = removed_store();
    let svc = removed_service(&removed);
    assert_eq!(svc.removed_devices(&own()).unwrap_err(), OrgNodeError::OrganisationRemoved { org_id: the_org() });
    assert_eq!(svc.store_content(&own()), Some(&StoreContent::Removed(removed)), "nothing changed");
}

// --- Deciding this node's removal and signing its acknowledgements ---

/// A chain state at `epoch` with `members`' root and the key whose private half is `seed`.
fn chain_listing(members: &[MemberSnapshot], epoch: u64, seed: u8) -> OrgState {
    OrgState::from_chain(*root_of(members).as_bytes(), *org_key_pair(seed).1.as_bytes(), epoch).unwrap()
}

/// `store_data` holding one received provisional organisation for epoch 4 whose members are `next`.
fn with_received(mut store_data: StoreData, next: Vec<MemberSnapshot>, seed: u8) -> StoreData {
    let (private, public) = org_key_pair(seed);
    store_data.provisional_organisations.push(ProvisionalOrganisation::from(ProvisionalOrganisationParts {
        epoch: Epoch::new(4),
        base_root: store_data.record.as_ref().unwrap().root_hash,
        root: root_of(&next),
        members: next,
        org_pub_key: public,
        org_private_key: private,
        change_set: Some(vec![1]),
        base_snapshot: Some(vec![2]),
        origin: Origin::Received { from: device_key(0x21) },
        applied: Vec::new(),
        first_ahead_at: None,
    }));
    store_data
}

/// A recording `device_secrets`: each call's keys, answered with `answer`.
fn recording(
    asked: &mut Vec<Vec<DevicePublicKey>>,
    answer: Vec<DeviceSecret>,
) -> impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret> + '_ {
    move |keys: &[DevicePublicKey]| {
        asked.push(keys.to_vec());
        answer
    }
}

fn removed_of(change: Option<StoreChange>) -> RemovedStore {
    let StoreChange::Sealed(write) = change.expect("a removal seals the removed store");
    assert_eq!(write.device_key, own());
    match content_of(&write) {
        StoreContent::Removed(removed) => removed,
        StoreContent::Active(_) => panic!("the removed form"),
    }
}

fn verify_signature(ack: &Acknowledgement) {
    let signature = ed25519_dalek::Signature::from_bytes(&ack.signature.0);
    ack.device.verifying_key().verify_strict(&ack.signed_bytes(), &signature).unwrap();
}

// verifies: LLR-jhy6t6
#[test]
fn a_match_not_listing_this_device_is_removed_and_seals_the_removed_store() {
    let others = vec![snapshot(2, "bob", 0x11, &[0x21]), snapshot(3, "carol", 0x12, &[0x22])];
    let next = others.clone();
    let mut svc = own_service(with_received(member_data(others), next.clone(), 0x63));
    let mut asked = vec![];
    let (verified, change) = svc
        .verify(&mut OsRng, &own(), Some(chain_listing(&next, 4, 0x63)), VerificationTime(100), org_node::AheadLimit(1200), recording(&mut asked, vec![own_secret()]))
        .unwrap();
    assert!(matches!(verified, Verified::Removed { org_id, acknowledgements_kept: 1 } if org_id == the_org()));
    let removed = removed_of(change);
    assert_eq!((removed.org_id, removed.verified_at), (the_org(), VerificationTime(100)));
    assert_eq!(removed.acknowledgements.len(), 1);
    assert_eq!(asked, vec![vec![own()]], "asked once, for this Persona's key");
    assert!(
        matches!(svc.store_data(&own()), Err(OrgNodeError::OrganisationRemoved { .. })),
        "the held store is the removed form"
    );
}

// verifies: LLR-jhy6t6
#[test]
fn a_kept_notice_accepted_against_the_chain_is_removed_after_the_inconsistent_checks() {
    let mut store_data = member_data(vec![snapshot(2, "bob", 0x11, &[0x21])]);
    store_data.kept_revocation_notices = vec![KeptNotice { notice: notice_removing_own(&store_data), from: device_key(0x21) }];
    let without = removed_members(&store_data);
    let mut svc = own_service(store_data);

    // A chain behind the record is Inconsistent first: nothing deleted, no secret asked for.
    let before = own_encoded(&svc);
    let (verified, change) =
        svc.verify(&mut OsRng, &own(), Some(chain_listing(&without, 2, 0x63)), VerificationTime(1), org_node::AheadLimit(1200), never_asked).unwrap();
    assert!(matches!(verified, Verified::Inconsistent { .. }));
    assert!(change.is_none());
    assert_eq!(own_encoded(&svc), before);

    // A later chain state the notice's proof holds under: removed.
    let mut asked = vec![];
    let (verified, change) = svc
        .verify(&mut OsRng, &own(), Some(chain_listing(&without, 4, 0x63)), VerificationTime(2), org_node::AheadLimit(1200), recording(&mut asked, vec![own_secret()]))
        .unwrap();
    assert!(matches!(verified, Verified::Removed { acknowledgements_kept: 1, .. }));
    let removed = removed_of(change);
    assert_eq!(removed.acknowledgements[0].recipient, device_key(0x21));
    assert_eq!(removed.acknowledgements[0].ack.epoch, Epoch::new(4));
    assert_eq!(removed.acknowledgements[0].ack.root, root_of(&without));
    assert_eq!(asked, vec![vec![own()]]);
}

// verifies: LLR-jhy6t6
#[test]
fn a_kept_notice_accept_refuses_stays_kept_and_the_judgement_continues() {
    let mut store_data = member_data(vec![snapshot(2, "bob", 0x11, &[0x21])]);
    store_data.kept_revocation_notices = vec![KeptNotice { notice: notice_removing_own(&store_data), from: device_key(0x21) }];
    let still_listing = vec![alice(), snapshot(4, "dave", 0x14, &[0x24])];
    let mut svc = own_service(store_data);
    let before = own_encoded(&svc);
    // Epoch 4 under a root that still lists this Device: accept refuses the proof; no match is held.
    let (verified, change) =
        svc.verify(&mut OsRng, &own(), Some(chain_listing(&still_listing, 4, 0x63)), VerificationTime(3), org_node::AheadLimit(1200), never_asked).unwrap();
    assert!(matches!(verified, Verified::Behind { chain_epoch } if chain_epoch == Epoch::new(4)));
    assert!(change.is_none());
    assert_eq!(own_encoded(&svc), before, "the notice stays kept");
    assert_eq!(svc.store_data(&own()).unwrap().kept_revocation_notices.len(), 1);
}

// verifies: LLR-5tvn7g
#[test]
fn the_acknowledgement_is_signed_once_with_the_personas_secret_and_kept_per_recipient() {
    let others: Vec<MemberSnapshot> =
        (0..5).map(|index| snapshot(10 + index, &format!("m{index}"), 0x30 + index, &[0x31 + index])).collect();
    let mut store_data = member_data(others.clone());
    store_data.address_book = vec![(device_key(0x31), addresses(&["192.0.2.31:4433"]))];
    let mut svc = own_service(with_received(store_data, others.clone(), 0x63));
    let state = chain_listing(&others, 4, 0x63);
    let mut asked = vec![];
    let (_, change) =
        svc.verify(&mut OsRng, &own(), Some(state), VerificationTime(5), org_node::AheadLimit(1200), recording(&mut asked, vec![own_secret()])).unwrap();
    let removed = removed_of(change);
    assert_eq!(asked.len(), 1, "device_secrets called once");
    assert_eq!(removed.acknowledgements.len(), 3, "five candidates: three recipients");
    let first = &removed.acknowledgements[0].ack;
    assert_eq!(
        (first.org_id, first.member_id, first.device, first.epoch, first.root),
        (the_org(), MemberId::new([1; 32]), own(), state.epoch, state.root_hash)
    );
    for kept in &removed.acknowledgements {
        assert_eq!(&kept.ack, first, "one acknowledgement, kept once per recipient");
        verify_signature(&kept.ack);
    }
    // Device 0x31's key is the second lowest of the five, so it is a recipient.
    let with_book = removed.acknowledgements.iter().find(|kept| kept.recipient == device_key(0x31)).expect("0x31 is a recipient");
    assert_eq!(with_book.recipient_addresses, Some(addresses(&["192.0.2.31:4433"])), "the recipient's address-book entry");
    assert!(removed
        .acknowledgements
        .iter()
        .filter(|kept| kept.recipient != device_key(0x31))
        .all(|kept| kept.recipient_addresses.is_none()));
}

// verifies: LLR-5tvn7g
#[test]
fn no_secret_returned_is_device_secret_not_supplied_and_nothing_changes() {
    for answer in [vec![], vec![device_secret([0x41; 32])]] {
        let others = vec![snapshot(2, "bob", 0x11, &[0x21])];
        let mut svc = own_service(with_received(member_data(others.clone()), others.clone(), 0x63));
        let before = own_encoded(&svc);
        let mut asked = vec![];
        let refused =
            svc.verify(&mut OsRng, &own(), Some(chain_listing(&others, 4, 0x63)), VerificationTime(6), org_node::AheadLimit(1200), recording(&mut asked, answer));
        assert!(matches!(refused, Err(OrgNodeError::DeviceSecretNotSupplied { org_id }) if org_id == the_org()));
        assert_eq!(asked, vec![vec![own()]]);
        assert_eq!(own_encoded(&svc), before, "no store change, the store as it was");
    }
}

/// `count` other members of the record, Devices `0x31..`, in that order.
fn others(count: u8) -> Vec<MemberSnapshot> {
    (0..count).map(|index| snapshot(10 + index, &format!("m{index}"), 0x40 + index, &[0x31 + index])).collect()
}

/// The given Device seeds' keys in ascending order of their bytes.
fn ascending(seeds: &[u8]) -> Vec<DevicePublicKey> {
    let mut keys: Vec<DevicePublicKey> = seeds.iter().map(|seed| device_key(*seed)).collect();
    keys.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    keys
}

/// The recipients `verify` keeps acknowledgements for when this Persona's
/// record lists it and `previous`, and an adopted match lists `next` alone.
fn recipients_by_match(previous: Vec<MemberSnapshot>, next: Vec<MemberSnapshot>) -> Vec<DevicePublicKey> {
    let mut svc = own_service(with_received(member_data(previous), next.clone(), 0x63));
    let (verified, change) = svc
        .verify(&mut OsRng, &own(), Some(chain_listing(&next, 4, 0x63)), VerificationTime(8), org_node::AheadLimit(1200), |_: &[DevicePublicKey]| vec![own_secret()])
        .unwrap();
    let removed = removed_of(change);
    assert!(matches!(verified, Verified::Removed { acknowledgements_kept, .. } if acknowledgements_kept == removed.acknowledgements.len()));
    removed.acknowledgements.iter().map(|kept| kept.recipient).collect()
}

// Normal and boundary: with n candidates, a removal by an adopted match
// keeps acknowledgements for k recipients, k being 0 for none (nothing
// signed, no secret asked for), 1 for one or two, 2 for three or four and 3
// for five or more; the match listing every candidate, they are the first k
// in ascending order. Every case is reached through `verify`, which
// LLR-6fsnma's rule binds.
// verifies: LLR-6fsnma
#[test]
fn a_removal_by_a_match_keeps_three_two_one_or_no_recipients() {
    let gina = vec![snapshot(7, "gina", 0x17, &[0x27])];
    let mut svc = own_service(with_received(member_data(vec![]), gina.clone(), 0x63));
    let (verified, change) =
        svc.verify(&mut OsRng, &own(), Some(chain_listing(&gina, 4, 0x63)), VerificationTime(8), org_node::AheadLimit(1200), never_asked).unwrap();
    assert!(matches!(verified, Verified::Removed { acknowledgements_kept: 0, .. }), "n = 0: k = 0");
    assert!(removed_of(change).acknowledgements.is_empty());
    for (candidates, recipients) in [(1u8, 1usize), (2, 1), (3, 2), (4, 2), (5, 3), (6, 3)] {
        let seeds: Vec<u8> = (0..candidates).map(|index| 0x31 + index).collect();
        let kept = recipients_by_match(others(candidates), others(candidates));
        assert_eq!(kept, ascending(&seeds)[..recipients].to_vec(), "n = {candidates}: k = {recipients}");
    }
}

// Normal: on a removal by an adopted match the candidates the match still
// lists come first, then the others, each group ascending; on a removal by
// a kept notice all candidates are taken ascending. Each fixture puts a
// still-listed candidate where plain ascending order would not take it.
// The delivering Device (0x21, or the kept notice's 0x21) is no candidate,
// so it changes nothing here.
// verifies: LLR-6fsnma
#[test]
fn the_still_listed_come_first_on_a_match_and_all_ascending_on_a_notice() {
    let all = ascending(&[0x31, 0x32, 0x33, 0x34, 0x35]);
    let member_with = |key: &DevicePublicKey| others(5).into_iter().find(|member| member.device_keys.contains(key)).unwrap();

    // The match still lists the highest candidate alone: it comes first, then the two lowest.
    let highest = all[4];
    let kept = recipients_by_match(others(5), vec![member_with(&highest)]);
    assert_eq!(kept, vec![highest, all[0], all[1]], "still listed first, then the others ascending");

    // The match still lists the three highest: they are the three taken, ascending.
    let three: Vec<MemberSnapshot> = all[2..].iter().map(member_with).collect();
    let kept = recipients_by_match(others(5), three);
    assert_eq!(kept, all[2..].to_vec(), "the still-listed, ascending");

    // A kept notice: all five candidates, the three lowest taken.
    let mut store_data = member_data(others(5));
    store_data.kept_revocation_notices = vec![KeptNotice { notice: notice_removing_own(&store_data), from: device_key(0x21) }];
    let without = removed_members(&store_data);
    let mut svc = own_service(store_data);
    let (verified, change) = svc
        .verify(&mut OsRng, &own(), Some(chain_listing(&without, 4, 0x63)), VerificationTime(9), org_node::AheadLimit(1200), |_: &[DevicePublicKey]| vec![own_secret()])
        .unwrap();
    assert!(matches!(verified, Verified::Removed { acknowledgements_kept: 3, .. }), "{verified:?}");
    let kept: Vec<DevicePublicKey> = removed_of(change).acknowledgements.iter().map(|kept| kept.recipient).collect();
    assert_eq!(kept, all[..3].to_vec(), "all candidates, ascending");
}

/// `store_data` holding one provisional organisation for epoch 4 whose members are `next`, from `origin`.
fn with_held_from(mut store_data: StoreData, next: Vec<MemberSnapshot>, seed: u8, origin: Origin) -> StoreData {
    let (private, public) = org_key_pair(seed);
    store_data.provisional_organisations.push(ProvisionalOrganisation::from(ProvisionalOrganisationParts {
        epoch: Epoch::new(4),
        base_root: store_data.record.as_ref().unwrap().root_hash,
        root: root_of(&next),
        members: next,
        org_pub_key: public,
        org_private_key: private,
        change_set: Some(vec![1]),
        base_snapshot: Some(vec![2]),
        origin,
        applied: Vec::new(),
        first_ahead_at: None,
    }));
    store_data
}

/// The recipients `verify` keeps acknowledgements for when `store_data` is removed against `chain`.
fn recipients_after(store_data: StoreData, chain: OrgState) -> Vec<DevicePublicKey> {
    let mut svc = own_service(store_data);
    let (verified, change) = svc
        .verify(&mut OsRng, &own(), Some(chain), VerificationTime(12), org_node::AheadLimit(1200), |_: &[DevicePublicKey]| vec![own_secret()])
        .unwrap();
    assert!(matches!(verified, Verified::Removed { .. }), "{verified:?}");
    removed_of(change).acknowledgements.iter().map(|kept| kept.recipient).collect()
}

// Normal: on a removal by a match the Device the match came from is the
// first recipient, then the others in the still-listed-first order; with
// k = 1 it is the only one. Each fixture picks a delivering Device the
// still-listed-first order would not take first. Boundary: an `Own` match
// has no delivering Device, and that order stands.
// verifies: LLR-6fsnma
#[test]
fn the_device_a_match_came_from_is_the_first_recipient() {
    let all = ascending(&[0x31, 0x32, 0x33, 0x34, 0x35]);
    let next = others(5);
    let from_highest = with_held_from(member_data(others(5)), next.clone(), 0x63, Origin::Received { from: all[4] });
    assert_eq!(recipients_after(from_highest, chain_listing(&next, 4, 0x63)), vec![all[4], all[0], all[1]]);

    let candidates = ascending(&[0x31, 0x32]);
    let answered = with_held_from(member_data(others(2)), others(2), 0x63, Origin::Answer { from: candidates[1] });
    assert_eq!(recipients_after(answered, chain_listing(&others(2), 4, 0x63)), vec![candidates[1]], "k = 1: the delivering Device alone");

    let own_match = with_held_from(member_data(others(5)), next.clone(), 0x63, Origin::Own);
    assert_eq!(recipients_after(own_match, chain_listing(&next, 4, 0x63)), all[..3].to_vec(), "no delivering Device: the still-listed-first order");
}

// Normal: on a removal by a kept notice its Delivering Device is the first
// recipient, then the others ascending; of two kept notices the first
// that `accept` takes names it.
// verifies: LLR-6fsnma, LLR-gb7p7c
#[test]
fn the_delivering_device_of_the_accepted_notice_is_the_first_recipient() {
    let all = ascending(&[0x31, 0x32, 0x33, 0x34, 0x35]);
    let mut store_data = member_data(others(5));
    let notice = notice_removing_own(&store_data);
    let mut refused = notice.clone();
    refused.member_id = MemberId::new([0x0f; 32]);
    store_data.kept_revocation_notices = vec![
        KeptNotice { notice: refused, from: all[2] },
        KeptNotice { notice, from: all[3] },
    ];
    let without = removed_members(&store_data);
    assert_eq!(recipients_after(store_data, chain_listing(&without, 4, 0x63)), vec![all[3], all[0], all[1]]);
}

// verifies: LLR-5tvn7g
#[test]
fn a_removal_that_cannot_sign_deletes_nothing() {
    let others = vec![snapshot(2, "bob", 0x11, &[0x21])];
    let mut svc = own_service(with_received(member_data(others.clone()), others.clone(), 0x63));
    let before = own_encoded(&svc);
    let refused = svc.verify(&mut OsRng, &own(), Some(chain_listing(&others, 4, 0x63)), VerificationTime(9), org_node::AheadLimit(1200), crate::support::none_held);
    assert!(matches!(refused, Err(OrgNodeError::DeviceSecretNotSupplied { .. })));
    let store_data = svc.store_data(&own()).expect("still the Active form");
    assert_eq!(own_encoded(&svc), before);
    assert!(store_data.record.is_some() && store_data.persona.member_id.is_some());
}

// verifies: LLR-5tvn7g
#[test]
fn a_signing_refusal_is_returned_unchanged_with_nothing_deleted() {
    let mut store_data = member_data(vec![snapshot(2, "bob", 0x11, &[0x21])]);
    store_data.kept_revocation_notices = vec![KeptNotice { notice: notice_removing_own(&store_data), from: device_key(0x21) }];
    let without = removed_members(&store_data);
    let mut svc = own_service(store_data);
    let before = own_encoded(&svc);
    let refused = svc.verify(&mut OsRng, &own(), Some(chain_listing(&without, 4, 0x63)), VerificationTime(10), org_node::AheadLimit(1200), |_: &[DevicePublicKey]| {
        vec![device_secret([0x41; 32])]
    });
    assert_eq!(refused.map(|(verified, _)| verified).unwrap_err(), OrgNodeError::DeviceSecretNotSupplied { org_id: the_org() });
    assert_eq!(own_encoded(&svc), before);
}

// verifies: LLR-5tvn7g, LLR-f9mcsq
#[test]
fn a_missing_device_secret_is_refused_before_anything_is_deleted() {
    let mut svc = own_service(rich_data());
    let without = removed_members(svc.store_data(&own()).unwrap());
    let before = own_encoded(&svc);
    let refused = svc.verify(&mut OsRng, &own(), Some(chain_listing(&without, 4, 0x63)), VerificationTime(11), org_node::AheadLimit(1200), crate::support::none_held);
    assert!(matches!(refused, Err(OrgNodeError::DeviceSecretNotSupplied { .. })));
    assert_eq!(own_encoded(&svc), before, "every field as it was: record, key, Persona, epoch state");
}

// Abnormal: a removal of a Persona with no MemberId is refused before
// anything is signed, and the store is left as it was.
// verifies: LLR-5tvn7g
#[test]
fn a_removal_of_a_persona_with_no_member_id_is_refused_with_nothing_deleted() {
    let others = vec![snapshot(2, "bob", 0x11, &[0x21])];
    let mut store_data = with_received(member_data(others.clone()), others.clone(), 0x63);
    store_data.persona.member_id = None;
    let mut svc = own_service(store_data);
    let before = own_encoded(&svc);
    let refused = svc.verify(&mut OsRng, &own(), Some(chain_listing(&others, 4, 0x63)), VerificationTime(9), org_node::AheadLimit(1200), |_: &[DevicePublicKey]| {
        vec![own_secret()]
    });
    assert_eq!(refused.map(|(verified, _)| verified).unwrap_err(), OrgNodeError::OrgNotHeld { org_id: the_org() });
    assert_eq!(own_encoded(&svc), before, "every field as it was");
}

// verifies: LLR-5tvn7g, LLR-f9mcsq
#[test]
fn a_removal_signs_with_the_supplied_secret_and_an_adoption_reads_none() {
    // An adoption whose record still lists this Device never calls device_secrets.
    let others = vec![snapshot(2, "bob", 0x11, &[0x21])];
    let next = vec![alice(), snapshot(3, "carol", 0x12, &[0x22])];
    let mut svc = own_service(with_received(member_data(others), next.clone(), 0x63));
    let (verified, change) =
        svc.verify(&mut OsRng, &own(), Some(chain_listing(&next, 4, 0x63)), VerificationTime(12), org_node::AheadLimit(1200), never_asked).unwrap();
    assert!(matches!(verified, Verified::Adopted { .. }));
    assert!(change.is_some());
    // Then a match leaving this Device out: signed with the supplied secret.
    let later = vec![snapshot(3, "carol", 0x12, &[0x22])];
    let mut store_data = svc.store_data(&own()).unwrap().clone();
    let (private, public) = org_key_pair(0x64);
    store_data.provisional_organisations.push(ProvisionalOrganisation::from(ProvisionalOrganisationParts {
        epoch: Epoch::new(5),
        base_root: store_data.record.as_ref().unwrap().root_hash,
        root: root_of(&later),
        members: later.clone(),
        org_pub_key: public,
        org_private_key: private,
        change_set: Some(vec![1]),
        base_snapshot: Some(vec![2]),
        origin: Origin::Received { from: device_key(0x22) },
        applied: Vec::new(),
        first_ahead_at: None,
    }));
    let mut svc = own_service(store_data);
    let mut asked = vec![];
    let (_, change) = svc
        .verify(&mut OsRng, &own(), Some(chain_listing(&later, 5, 0x64)), VerificationTime(13), org_node::AheadLimit(1200), recording(&mut asked, vec![own_secret()]))
        .unwrap();
    let removed = removed_of(change);
    assert_eq!(asked, vec![vec![own()]]);
    verify_signature(&removed.acknowledgements[0].ack);
    assert_eq!(removed.acknowledgements[0].ack.device, own());
}

// --- The kept acknowledgements listed and an accepted one deleted ---

// verifies: LLR-85kves
#[test]
fn acknowledgement_accepted_removes_that_recipients_acknowledgement_and_returns_the_count_left() {
    let mut svc = removed_service(&removed_store());
    let (left, change) = svc.acknowledgement_accepted(&mut OsRng, &own(), device_key(0x21)).unwrap();
    assert_eq!(left, 1);
    let StoreChange::Sealed(write) = change;
    assert_eq!(write.device_key, own());
    let StoreContent::Removed(removed) = content_of(&write) else { panic!("still the removed form") };
    assert_eq!(removed.acknowledgements, vec![kept(0x22, None)], "that recipient's, with its key and addresses, gone");
    assert_eq!((removed.org_id, removed.verified_at), (the_org(), VerificationTime(1_760_000_000)));

    let (left, _) = svc.acknowledgement_accepted(&mut OsRng, &own(), device_key(0x22)).unwrap();
    assert_eq!(left, 0, "none left: org-io deletes the file and keychain item");
    assert!(svc.store_content(&own()).is_some(), "org-node removes no store");
}

// The recipient accepted is not the first kept: only its acknowledgement goes, and the first stays
// with its key and addresses (merge review MR3, finding-4).
// verifies: LLR-85kves
#[test]
fn acknowledgement_accepted_for_a_later_recipient_leaves_the_earlier_ones() {
    let mut svc = removed_service(&removed_store());
    let (left, change) = svc.acknowledgement_accepted(&mut OsRng, &own(), device_key(0x22)).unwrap();
    assert_eq!(left, 1);
    let StoreChange::Sealed(write) = change;
    let StoreContent::Removed(removed) = content_of(&write) else { panic!("still the removed form") };
    assert_eq!(
        removed.acknowledgements,
        vec![kept(0x21, Some(addresses(&["198.51.100.7:4433"])))],
        "only the second recipient's acknowledgement is gone"
    );
}

// verifies: LLR-85kves
#[test]
fn acknowledgement_accepted_refuses_an_unnamed_recipient_and_an_active_store() {
    let mut svc = removed_service(&removed_store());
    let before = svc.store_content(&own()).cloned();
    assert_eq!(
        svc.acknowledgement_accepted(&mut OsRng, &own(), device_key(0x2f)).map(|(left, _)| left).unwrap_err(),
        OrgNodeError::AcknowledgementNotKept { recipient: Box::new(device_key(0x2f)) }
    );
    assert_eq!(svc.store_content(&own()).cloned(), before, "nothing changed");

    let mut active = own_service(member_data(vec![snapshot(2, "bob", 0x11, &[0x21])]));
    let before = own_encoded(&active);
    assert_eq!(
        active.acknowledgement_accepted(&mut OsRng, &own(), device_key(0x21)).map(|(left, _)| left).unwrap_err(),
        OrgNodeError::AcknowledgementNotKept { recipient: Box::new(device_key(0x21)) }
    );
    assert_eq!(own_encoded(&active), before);
    assert!(matches!(
        active.acknowledgement_accepted(&mut OsRng, &device_key(0x40), device_key(0x21)),
        Err(OrgNodeError::PersonaNotHeld { .. })
    ));
}

// verifies: LLR-75sn9g
#[test]
fn kept_acknowledgements_is_none_for_an_active_store_and_lists_a_removed_ones_in_order() {
    let active = own_service(member_data(vec![]));
    assert!(active.kept_acknowledgements(&own()).unwrap().is_none());
    assert!(matches!(active.kept_acknowledgements(&device_key(0x40)), Err(OrgNodeError::PersonaNotHeld { .. })));

    let removed = removed_store();
    let svc = removed_service(&removed);
    let KeptAcknowledgements { org_id, verified_at, acknowledgements } = svc.kept_acknowledgements(&own()).unwrap().unwrap();
    assert_eq!((org_id, verified_at), (the_org(), VerificationTime(1_760_000_000)));
    assert_eq!(acknowledgements.len(), 2);
    for ((message, addresses), kept) in acknowledgements.iter().zip(&removed.acknowledgements) {
        assert_eq!(message.recipient, kept.recipient, "in kept order");
        assert_eq!(decode_message(&message.body).unwrap(), WireMessage::Acknowledgement(kept.ack.clone()));
        assert_eq!(addresses, &kept.recipient_addresses);
    }
    let again = svc.kept_acknowledgements(&own()).unwrap().unwrap();
    assert_eq!(
        again.acknowledgements.iter().map(|(message, _)| message.body.clone()).collect::<Vec<_>>(),
        acknowledgements.iter().map(|(message, _)| message.body.clone()).collect::<Vec<_>>(),
        "equal stores give equal results; nothing changes"
    );
}

// ---- LLR-f9mcsq: on an Active store, nothing but the removal deletes --------------------------------------

/// Every field of the store's data, by name, as serde's data model renders it.
fn fields(svc: &OrgService) -> serde_json::Map<String, serde_json::Value> {
    serde_json::to_value(svc.store_data(&own()).unwrap()).unwrap().as_object().unwrap().clone()
}

/// `after` equals `before` in every field but `named`.
fn only_changed(
    before: &serde_json::Map<String, serde_json::Value>,
    after: &serde_json::Map<String, serde_json::Value>,
    named: &[&str],
    what: &str,
) {
    for (field, field_before) in before {
        if !named.contains(&field.as_str()) {
            assert_eq!(after.get(field), Some(field_before), "{what} changed `{field}`");
        }
    }
}

// The rich store is driven through each operation's refusal and each non-removal outcome, and every
// field is compared by name. An adoption marks the Persona Active, so `persona` is named for it and
// the two key asserts pin that the Persona's keys survive. Before the adoption the store holds Bob in
// the record and the Revoke for Bob, so the Revoke is included and removed (LLR-d9x6xf); Bob's Device
// goes to `revoked_devices`, and the received notice goes (LLR-x8drb8). A field cleared by an
// operation that does not name it (the address book on `cancel_invite`, the provisional changes on
// an `Inconsistent` mark) fails this test.
// verifies: LLR-f9mcsq
#[test]
fn on_an_active_store_nothing_but_the_removal_deletes_and_each_operation_removes_only_what_it_names() {
    let store_data = rich_data();
    let record = store_data.record.clone().unwrap();
    let invite_id = store_data.kept_admission_requests[0].invite_id;
    let mut svc = own_service(store_data.clone());
    let start = fields(&svc);

    // Every refusal leaves the store as it was.
    let refusals: Vec<Result<(), OrgNodeError>> = vec![
        svc.record_revocation(&mut OsRng, &own(), MemberId::new([0x77; 32])).map(|_| ()),
        svc.record_revocation(&mut OsRng, &own(), MemberId::new([2; 32])).map(|_| ()),
        svc.record_admission(&mut OsRng, &own(), InviteId::new([0x39; 32]), None).map(|_| ()),
        svc.cancel_invite(&mut OsRng, &own(), InviteId::new([0x39; 32])).map(|_| ()),
        svc.build_invite(&mut OsRng, &own(), "x".into(), vec!["a".repeat(257)]).map(|_| ()),
        svc.record_founding_proxy(&mut OsRng, &own(), the_org(), org_node::ChainAccount::new([1; 32])).map(|_| ()),
        svc.verify(&mut OsRng, &own(), None, VerificationTime(1), org_node::AheadLimit(1200), never_asked).map(|_| ()),
        svc.check_received(
            &mut OsRng,
            &own(),
            device_key(0x2f),
            &encode_message(&WireMessage::OrganisationRequest { org_id: the_org(), addresses: vec![] }).unwrap(),
        )
        .map(|_| ()),
        svc.check_received(&mut OsRng, &own(), own(), &[0xff, 0xff]).map(|_| ()),
    ];
    assert!(refusals.iter().all(Result::is_err), "each case is a refusal: {refusals:?}");
    assert_eq!(fields(&svc), start, "every refusal leaves every field as it was");

    // InStep, Behind and Inconsistent remove nothing.
    let root = *record.root_hash.as_bytes();
    let key = *record.org_pub_key.as_bytes();
    for (epoch, root) in [(3, root), (2, root), (3, [0x12; 32]), (9, [0x13; 32])] {
        let (verified, change) = svc
            .verify(&mut OsRng, &own(), Some(OrgState::from_chain(root, key, epoch).unwrap()), VerificationTime(2), org_node::AheadLimit(1200), never_asked)
            .unwrap();
        assert!(matches!(verified, Verified::InStep | Verified::Behind { .. } | Verified::Inconsistent { .. }), "{verified:?}");
        assert!(change.is_none());
        assert_eq!(fields(&svc), start, "{verified:?} removes nothing");
    }

    // check_received removes nothing on any outcome: an answered request only sets its sender's entry.
    {
        let mut svc = own_service(store_data.clone());
        let body =
            encode_message(&WireMessage::OrganisationRequest { org_id: the_org(), addresses: vec!["192.0.2.22:1".into()] }).unwrap();
        let (_answered, change) = svc.check_received(&mut OsRng, &own(), device_key(0x22), &body).unwrap();
        crate::support::unread(change);
        only_changed(&start, &fields(&svc), &["address_book"], "an answered organisation request");
    }

    // cancel_invite removes only its request, pair and Admit (here none).
    {
        let mut svc = own_service(store_data.clone());
        crate::support::unread(svc.cancel_invite(&mut OsRng, &own(), invite_id).unwrap());
        let after = fields(&svc);
        only_changed(&start, &after, &["kept_admission_requests", "outstanding_invites", "provisional_changes"], "cancel_invite");
        assert_eq!(after["provisional_changes"], start["provisional_changes"], "no Admit named it: the Revoke stays");
        assert_eq!(after["kept_admission_requests"], serde_json::json!([]));
    }

    // An adoption whose record lists this Device removes only LLR-d9x6xf's, LLR-x8drb8's and LLR-rzqs2u's.
    let proposal = &store_data.provisional_organisations[0];
    let state = OrgState::from_chain(*proposal.root.as_bytes(), *proposal.org_pub_key.as_bytes(), 4).unwrap();
    let (verified, _) = svc.verify(&mut OsRng, &own(), Some(state), VerificationTime(3), org_node::AheadLimit(1200), never_asked).unwrap();
    assert!(matches!(verified, Verified::Adopted { .. }));
    let after = fields(&svc);
    only_changed(
        &start,
        &after,
        &[
            "persona",
            "record",
            "provisional_changes",
            "provisional_organisations",
            "kept_revocation_notices",
            "revoked_devices",
            "address_book",
            "kept_admission_requests",
            "outstanding_invites",
        ],
        "an adoption",
    );
    assert_eq!(after["persona"]["member_seed"], start["persona"]["member_seed"], "the Persona's keys stay");
    assert_eq!(after["persona"]["device_key"], start["persona"]["device_key"]);
    assert!(after["record"].is_object(), "the record is replaced, never removed");
    assert_eq!(after["kept_admission_requests"], start["kept_admission_requests"], "Dave is not in the adopted record: his request stays");
    assert_eq!(after["outstanding_invites"], start["outstanding_invites"]);
    assert_eq!(after["address_book"][0], start["address_book"][0], "existing entries are kept");
}

// Normal: an adoption keeps the acknowledged record whole. Abnormal (a
// scan): the field, its type and its append carry the S5 TODO.
// verifies: LLR-py43n3
#[test]
fn an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo() {
    let bob = snapshot(2, "bob", 0x11, &[0x21]);
    let mut store_data = member_data(vec![bob.clone()]);
    let acknowledged_entry = org_node::store::AcknowledgedEntry { acknowledgement: sample_ack(0x5b), member_id: MemberId::new([9; 32]) };
    store_data.acknowledged = vec![acknowledged_entry.clone()];
    let next = vec![alice(), bob];
    let mut svc = own_service(with_received(store_data, next.clone(), 0x63));
    let (verified, _) = svc
        .verify(&mut OsRng, &own(), Some(chain_listing(&next, 4, 0x63)), VerificationTime(10), org_node::AheadLimit(1200), never_asked)
        .unwrap();
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    assert_eq!(svc.store_data(&own()).unwrap().acknowledged, vec![acknowledged_entry]);
    let source = include_str!("../src/store.rs");
    let todo_just_before = |item: &str| {
        let at = source.find(item).unwrap_or_else(|| panic!("{item}"));
        source[..at].rfind("TODO(S5): the S5 revoked-device CRDT replaces this record").is_some_and(|todo| at - todo < 400)
    };
    assert!(todo_just_before("pub acknowledged:"), "the TODO sits on the field");
    assert!(todo_just_before("pub struct AcknowledgedEntry"), "and on its type");
    assert!(include_str!("../src/service.rs").contains("TODO(S5): the S5 revoked-device CRDT replaces this record"), "and on the append");
}
