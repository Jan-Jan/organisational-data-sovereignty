#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The sealed per-Persona store (SDD-asxwp8): one store per Persona, sealed
//! under a key derived from that Persona's device secret, given out as
//! `StoreWrite` bytes and taken back in by `PersonaStore::unseal`. org-node
//! touches no file: these tests play org-io, holding each returned write as
//! "the disk" and each secret's bytes as "the keychain".
//!
//! A Persona's store is the one `propose_founding` creates, and a founder's
//! the one its founding's adoption by `verify` leaves; each write tested is
//! an operation's own. The stores are named by their Device key
//! (LLR-f9x3vf).

mod support;

use std::path::Path;

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use org_members::{DevicePublicKey, Handle, MemberId};
use org_node::invitation::InviteId;
use org_node::keys::DeviceSecret;
use org_node::service::{OrgService, StoreChange};
use org_node::store::{PersonaStore, SealedStore, StoreData, StoreRefusal, StoreWrite};
use org_node::test_fixtures::{self, ChainSlots};
use org_node::verify_org::{VerificationTime, Verified};
use org_node::wire::{encode_message, WireMessage};
use org_node::{ChainAccount, OrgNodeError, PersonaId};
use rand::rngs::OsRng;
use sha2::Sha256;
use zeroize::Zeroizing;

/// A `DeviceSecret` re-made from keychain bytes: what org-io reads back.
fn from_keychain(bytes: &[u8]) -> DeviceSecret {
    DeviceSecret::from_keychain_bytes(Zeroizing::new(bytes.to_vec())).unwrap()
}

/// A new Persona on `svc`, proposing a founding (`propose_founding`): its
/// id, its seed bytes (the test's keychain) and the write the operation
/// returned: the operation's own write, not a store built here and sealed
/// through the test hook `seal_for_test`.
fn new_persona(svc: &mut OrgService, handle: &str) -> (PersonaId, Vec<u8>, StoreWrite) {
    let (device, secret, change) = svc.propose_founding(&mut OsRng, "Test", "User", handle, "Acme".into()).unwrap();
    let StoreChange::Sealed(write) = change;
    let persona_id = svc.list_personas().iter().find(|persona| persona.device_key == device).unwrap().persona_id.clone();
    (persona_id, support::seed_bytes(&secret), write)
}

/// A new Persona on `svc` whose founding is written on `chain`, its proxy
/// recorded and the founding adopted (`propose_founding`,
/// `record_founding_proxy`, `verify`): its id, seed bytes, the adoption's
/// write and the Organisation.
fn new_founder(svc: &mut OrgService, chain: &ChainSlots, handle: &str) -> (PersonaId, Vec<u8>, StoreWrite, org_node::OrgId) {
    let (persona_id, seed, proposed) = new_persona(svc, handle);
    let device = proposed.device_key;
    let founding = svc.persona_view(&device).unwrap().founding.expect("a provisional founding");
    let org_id = chain.apply_genesis(founding.root, founding.org_pub_key);
    crate::support::unread(svc.record_founding_proxy(&mut OsRng, &device, org_id, support::the_proxy()).unwrap());
    let (verified, change) =
        svc.verify(&mut OsRng, &device, chain.get(&org_id), VerificationTime(1), org_node::AheadLimit(1200), crate::support::none_held).unwrap();
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    let StoreChange::Sealed(write) = change.expect("the adoption's write");
    (persona_id, seed, write, org_id)
}

// ---- LLR-bg538c: unseal ------------------------------------------------------

// verifies: LLR-bg538c
#[test]
fn a_sealed_store_unseals_with_its_secret_and_device_key() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, seed, write) = new_persona(&mut svc, "alice");
    let store = PersonaStore::unseal(&write.sealed, write.device_key, from_keychain(&seed)).unwrap();
    assert_eq!(store.device_key(), write.device_key);
    assert_eq!(store.data().unwrap().persona.handle.as_ref().map(Handle::as_str), Some("alice"));
    assert_eq!(store.data().unwrap().persona.device_key, write.device_key);
}

// verifies: LLR-bg538c
#[test]
fn unseal_refuses_each_malformed_input_with_its_own_error() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, seed, write) = new_persona(&mut svc, "alice");
    let device_key = write.device_key;

    // Below nonce(24) + tag(16): too short, whatever the bytes.
    for length in [0usize, 23, 39] {
        let short = SealedStore::from_bytes(vec![0xab; length]);
        assert!(
            matches!(
                PersonaStore::unseal(&short, device_key, from_keychain(&seed)),
                Err(OrgNodeError::StoreRefused(StoreRefusal::TooShort))
            ),
            "{length} bytes"
        );
    }

    // One ciphertext bit flipped: not authentic.
    let mut flipped = write.sealed.as_bytes().to_vec();
    flipped[30] ^= 0x01;
    assert!(matches!(
        PersonaStore::unseal(&SealedStore::from_bytes(flipped), device_key, from_keychain(&seed)),
        Err(OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic))
    ));

    // Another Persona's secret: a mismatch, found before any decryption
    // (the bytes would not even pass the length check).
    let (_, other_seed, _) = new_persona(&mut svc, "bob");
    assert!(matches!(
        PersonaStore::unseal(&write.sealed, device_key, from_keychain(&other_seed)),
        Err(OrgNodeError::DeviceSecretMismatch)
    ));
    assert!(matches!(
        PersonaStore::unseal(&SealedStore::from_bytes(vec![]), device_key, from_keychain(&other_seed)),
        Err(OrgNodeError::DeviceSecretMismatch)
    ));

    // Sealed under the right key, but bytes that do not decode (after the Active form's index).
    let undecodable =
        test_fixtures::seal_plaintext_for_test(&from_keychain(&seed), device_key, &[0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], &mut OsRng);
    assert!(matches!(
        PersonaStore::unseal(&undecodable, device_key, from_keychain(&seed)),
        Err(OrgNodeError::StoreRefused(StoreRefusal::Undecodable))
    ));

    // Sealed under the right key, decodes, but names another Device.
    let mut store_data: StoreData =
        PersonaStore::unseal(&write.sealed, device_key, from_keychain(&seed)).unwrap().data().unwrap().clone();
    store_data.persona.device_key = test_fixtures::device_key(0x61);
    let mut foreign_plaintext = vec![0u8]; // the Active form's index (LLR-6zaeb6)
    foreign_plaintext.extend(postcard::to_allocvec(&store_data).unwrap());
    let foreign = test_fixtures::seal_plaintext_for_test(&from_keychain(&seed), device_key, &foreign_plaintext, &mut OsRng);
    assert!(matches!(
        PersonaStore::unseal(&foreign, device_key, from_keychain(&seed)),
        Err(OrgNodeError::DeviceSecretMismatch)
    ));
}

// ---- LLR-2qqqx6: the store key -----------------------------------------------

/// The key LLR-2qqqx6 states, computed here from first principles.
fn hkdf_key(seed: &[u8], device_key: &DevicePublicKey) -> [u8; 32] {
    let mut key = [0u8; 32];
    Hkdf::<Sha256>::new(Some(device_key.as_bytes()), seed).expand(b"ods/persona-store/v1", &mut key).unwrap();
    key
}

fn open_directly(sealed: &SealedStore, key: &[u8; 32]) -> Vec<u8> {
    let (nonce, ciphertext) = sealed.as_bytes().split_at(24);
    XChaCha20Poly1305::new(key.into()).decrypt(XNonce::from_slice(nonce), ciphertext).unwrap()
}

// verifies: LLR-2qqqx6
#[test]
fn the_store_key_is_hkdf_sha256_of_the_seed_under_the_versioned_label() {
    // A fixed seed, sealed by the fixture the store's own seal shares.
    let seed = [0x07u8; 32];
    let secret = from_keychain(&seed);
    let device_key = secret.device_key();
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, created_seed, created) = new_persona(&mut svc, "alice");
    let mut store_data: StoreData = PersonaStore::unseal(&created.sealed, created.device_key, from_keychain(&created_seed))
        .unwrap()
        .data()
        .unwrap()
        .clone();
    store_data.persona.device_key = device_key;
    let mut bytes = vec![0u8]; // the Active form's index (LLR-6zaeb6)
    bytes.extend(postcard::to_allocvec(&store_data).unwrap());
    let sealed = test_fixtures::seal_plaintext_for_test(&secret, device_key, &bytes, &mut OsRng);
    let key = hkdf_key(&seed, &device_key);
    assert_eq!(open_directly(&sealed, &key), bytes);
    let unsealed = PersonaStore::unseal(&sealed, device_key, from_keychain(&seed)).unwrap();
    assert_eq!(unsealed.device_key(), device_key);

    // The key is neither the seed nor the ed25519 secret scalar.
    assert_ne!(key, seed);
    assert_ne!(key, ed25519_dalek::SigningKey::from_bytes(&seed).to_scalar_bytes());
}

// verifies: LLR-2qqqx6
#[test]
fn a_created_store_opens_under_the_hkdf_key_of_its_secret() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, seed, write) = new_persona(&mut svc, "alice");
    let plaintext = open_directly(&write.sealed, &hkdf_key(&seed, &write.device_key));
    // After the Active form's index (LLR-6zaeb6), the store data.
    let store_data: StoreData = postcard::from_bytes(&plaintext[1..]).unwrap();
    assert_eq!(store_data.persona.handle.as_ref().map(Handle::as_str), Some("alice"));
}

/// `plaintext` sealed directly under `key`, with a fresh nonce: a store as a
/// sealer using `key` would have written it.
fn seal_directly(plaintext: &[u8], key: &[u8; 32]) -> SealedStore {
    let nonce: [u8; 24] = rand::random();
    let ciphertext = XChaCha20Poly1305::new(key.into()).encrypt(XNonce::from_slice(&nonce), plaintext).unwrap();
    SealedStore::from_bytes([nonce.as_slice(), &ciphertext].concat())
}

// Abnormal: a store sealed under a key
// derived any other way than LLR-2qqqx6's — the seed itself, HKDF without
// the Device-key salt, another salt, another label, the ed25519 secret
// scalar — does not open (`NotAuthentic`); the same plaintext sealed under
// the stated key does, so the refusals are the key's alone.
// verifies: LLR-2qqqx6
#[test]
fn a_store_sealed_under_any_other_key_derivation_is_refused() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, seed, write) = new_persona(&mut svc, "alice");
    let device_key = write.device_key;
    let plaintext = open_directly(&write.sealed, &hkdf_key(&seed, &device_key));
    let expand = |salt: Option<&[u8]>, info: &[u8]| {
        let mut key = [0u8; 32];
        Hkdf::<Sha256>::new(salt, &seed).expand(info, &mut key).unwrap();
        key
    };
    let raw_seed: [u8; 32] = seed.as_slice().try_into().unwrap();
    let scalar = ed25519_dalek::SigningKey::from_bytes(&raw_seed).to_scalar_bytes();
    let other_salt = [0x5au8; 32];
    for (how, key) in [
        ("the seed itself", raw_seed),
        ("HKDF without the Device-key salt", expand(None, b"ods/persona-store/v1")),
        ("HKDF under another salt", expand(Some(&other_salt), b"ods/persona-store/v1")),
        ("HKDF under another label", expand(Some(device_key.as_bytes()), b"ods/persona-store/v2")),
        ("the ed25519 secret scalar", scalar),
    ] {
        let refused = PersonaStore::unseal(&seal_directly(&plaintext, &key), device_key, from_keychain(&seed)).err();
        assert_eq!(refused, Some(OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic)), "{how}");
    }
    let stated = seal_directly(&plaintext, &hkdf_key(&seed, &device_key));
    assert!(PersonaStore::unseal(&stated, device_key, from_keychain(&seed)).is_ok(), "the stated key opens it");
}

fn org_node_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

// verifies: LLR-2qqqx6
#[test]
fn no_argon2_and_no_passphrase_remain() {
    let mut files = support::src_rust_files();
    let manifest = org_node_dir().join("Cargo.toml");
    let manifest_text = std::fs::read_to_string(&manifest).unwrap();
    files.push((manifest, manifest_text));
    for (file, text) in files {
        for needle in ["argon2", "Argon2", "passphrase", "APP_SALT"] {
            assert!(!text.contains(needle), "{} names `{needle}`", file.display());
        }
    }
}

// ---- LLR-f9x3vf: one store per Persona -------------------------------------------

// verifies: LLR-f9x3vf
#[test]
fn two_stores_with_one_device_key_are_refused() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, seed, write) = new_persona(&mut svc, "alice");
    let first = PersonaStore::unseal(&write.sealed, write.device_key, from_keychain(&seed)).unwrap();
    let second = PersonaStore::unseal(&write.sealed, write.device_key, from_keychain(&seed)).unwrap();
    assert!(matches!(OrgService::new(vec![first, second]), Err(OrgNodeError::DuplicatePersonaStore)));
}

// Normal: `build_invite` in one Persona's store changes that store alone.
// verifies: LLR-f9x3vf
#[test]
fn an_operation_changes_only_its_personas_store() {
    let chain = ChainSlots::new();
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, _, first_write, _) = new_founder(&mut svc, &chain, "alice");
    let (_, _, second_write) = new_persona(&mut svc, "bob");
    let second_before = postcard::to_allocvec(svc.store_data(&second_write.device_key).unwrap()).unwrap();
    let (_, change) = svc.build_invite(&mut OsRng, &first_write.device_key, "Carol".into(), vec![]).unwrap();
    let org_node::service::StoreChange::Sealed(write) = change;
    assert_eq!(write.device_key, first_write.device_key, "the write is the first Persona's");
    assert_eq!(postcard::to_allocvec(svc.store_data(&second_write.device_key).unwrap()).unwrap(), second_before);
    assert!(svc.store_data(&second_write.device_key).unwrap().outstanding_invites.is_empty());
    assert_eq!(svc.store_data(&first_write.device_key).unwrap().outstanding_invites.len(), 1);
}

// verifies: LLR-f9x3vf
#[test]
fn list_personas_returns_one_record_per_store() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let mut created = vec![];
    for handle in ["alice", "bob", "carol"] {
        created.push(new_persona(&mut svc, handle));
    }
    let listed: Vec<PersonaId> = svc.list_personas().iter().map(|persona| persona.persona_id.clone()).collect();
    assert_eq!(listed, created.iter().map(|(persona_id, _, _)| persona_id.clone()).collect::<Vec<_>>());

    // Reopened from the writes: the same Personas, one store each.
    let stores = created
        .iter()
        .map(|(_, seed, write)| PersonaStore::unseal(&write.sealed, write.device_key, from_keychain(seed)).unwrap())
        .collect();
    let reopened = OrgService::new(stores).unwrap();
    let relisted: Vec<PersonaId> = reopened.list_personas().iter().map(|persona| persona.persona_id.clone()).collect();
    assert_eq!(relisted, listed);
}

// Normal and abnormal: org-io puts back a Persona's store as its file holds
// it after a failed write, and takes one out when the keychain refused its
// new Persona; one store per Device key either way, and the other stores
// untouched. The store runs ahead of its file by `build_invite`.
// verifies: LLR-f9x3vf
#[test]
fn restore_store_replaces_or_adds_one_store_and_drop_store_removes_one() {
    let chain = ChainSlots::new();
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, alice_seed, alice_created, _) = new_founder(&mut svc, &chain, "alice");
    let (_, bob_seed, bob_created) = new_persona(&mut svc, "bob");
    let (alice, bob) = (alice_created.device_key, bob_created.device_key);
    let (_invite, ahead) = svc.build_invite(&mut OsRng, &alice, "Carol".into(), vec![]).unwrap();
    support::unread(ahead);
    // Compared as encoded bytes, which hold the secret fields `Debug` redacts.
    let bob_before = support::encoded_store(&svc, &bob);

    // The store as its file held it replaces the one that ran ahead.
    let alice_on_disk = PersonaStore::unseal(&alice_created.sealed, alice, from_keychain(&alice_seed)).unwrap();
    svc.restore_store(alice_on_disk);
    assert_eq!(svc.list_personas().len(), 2, "replaced, not added");
    assert!(svc.store_data(&alice).unwrap().outstanding_invites.is_empty(), "the store is the file's again");
    assert_eq!(support::encoded_store(&svc, &bob), bob_before, "another Persona's store is untouched");

    // Dropped, and dropped again: the second finds nothing.
    assert!(svc.drop_store(&bob));
    assert!(!svc.drop_store(&bob));
    assert_eq!(svc.store_data(&bob).err(), Some(OrgNodeError::PersonaNotHeld { device_key: Box::new(bob) }));
    assert_eq!(svc.list_personas().len(), 1);

    // A store the service does not hold is added back.
    let bob_on_disk = PersonaStore::unseal(&bob_created.sealed, bob, from_keychain(&bob_seed)).unwrap();
    svc.restore_store(bob_on_disk);
    assert_eq!(svc.list_personas().len(), 2);
    assert_eq!(support::encoded_store(&svc, &bob), bob_before);
}

// Abnormal: a Device key no held store's Persona has is refused with
// `PersonaNotHeld` by every per-store operation, and so is a Persona no
// store holds; the held stores are read and changed by none of them.
// Normal: each held store is found by its own key. The refusals are the
// operations' own, not the test-support accessor's, and "no store was
// changed" compares encoded bytes.
// verifies: LLR-f9x3vf
#[test]
fn an_unknown_device_key_is_refused_as_persona_not_held() {
    let chain = ChainSlots::new();
    let mut svc = OrgService::new(vec![]).unwrap();
    let (alice, _, alice_write, org_id) = new_founder(&mut svc, &chain, "alice");
    let (_, _, bob_write) = new_persona(&mut svc, "bob");
    let held = [alice_write.device_key, bob_write.device_key];
    let before: Vec<Vec<u8>> = held.iter().map(|key| support::encoded_store(&svc, key)).collect();
    let unknown = test_fixtures::device_key(0x7e);
    let peer = test_fixtures::device_key(0x7d);
    let body = encode_message(&WireMessage::OrganisationRequest { org_id, addresses: vec![] }).unwrap();
    let refusals: Vec<(&str, Result<(), OrgNodeError>)> = vec![
        ("record_founding_proxy", svc.record_founding_proxy(&mut OsRng, &unknown, org_id, ChainAccount::new([1; 32])).map(|_| ())),
        ("check_received", svc.check_received(&mut OsRng, &unknown, peer, &body).map(|_| ())),
        ("verify", svc.verify(&mut OsRng, &unknown, chain.get(&org_id), VerificationTime(2), org_node::AheadLimit(1200), crate::support::none_held).map(|_| ())),
        ("build_invite", svc.build_invite(&mut OsRng, &unknown, "Y".into(), vec![]).map(|_| ())),
        ("build_admission_request", svc.build_admission_request(&unknown, vec![]).map(|_| ())),
        ("build_organisation_request", svc.build_organisation_request(&unknown, vec![]).map(|_| ())),
        ("cancel_invite", svc.cancel_invite(&mut OsRng, &unknown, InviteId::new([1; 32])).map(|_| ())),
        ("record_admission", svc.record_admission(&mut OsRng, &unknown, InviteId::new([1; 32]), None).map(|_| ())),
        ("record_revocation", svc.record_revocation(&mut OsRng, &unknown, MemberId::new([1; 32])).map(|_| ())),
        ("propose_updated_organisation", svc.propose_updated_organisation(&mut OsRng, &unknown).map(|_| ())),
        ("messages_for_current_epoch", svc.messages_for_current_epoch(&unknown).map(|_| ())),
        ("provisional_changes", svc.provisional_changes(&unknown).map(|_| ())),
        ("provisional_organisations", svc.provisional_organisations(&unknown).map(|_| ())),
        ("admission_requests", svc.admission_requests(&unknown).map(|_| ())),
        ("persona_view", svc.persona_view(&unknown).map(|_| ())),
        ("addresses_for", svc.addresses_for(&unknown, peer).map(|_| ())),
        ("acknowledgement_accepted", svc.acknowledgement_accepted(&mut OsRng, &unknown, peer).map(|_| ())),
        ("kept_acknowledgements", svc.kept_acknowledgements(&unknown).map(|_| ())),
    ];
    for (operation, refused) in refusals {
        assert_eq!(refused, Err(OrgNodeError::PersonaNotHeld { device_key: Box::new(unknown) }), "{operation}");
    }
    let nobody = PersonaId::new("nobody".into());
    assert_eq!(
        svc.persona_public_keys(&nobody).map(|_| ()),
        Err(OrgNodeError::PersonaIdNotHeld { persona_id: nobody.clone() }),
        "persona_public_keys"
    );
    assert_eq!(svc.persona_view(&alice_write.device_key).unwrap().device_key, alice_write.device_key);
    assert_eq!(svc.persona_public_keys(&alice).unwrap().1, alice_write.device_key, "each held store by its own key");
    let after: Vec<Vec<u8>> = held.iter().map(|key| support::encoded_store(&svc, key)).collect();
    assert_eq!(after, before, "no store was changed");
}

// ---- LLR-ay8nvu: one store change per operation ----------------------------------
//
// The properties stated over every store-changing operation.
mod store_change_properties {
    use org_members::{DevicePublicKey, MemberId, Name, PersonPublicKey, Surname};
    use org_node::ids::OrgId;
    use org_node::invitation::{EndpointAddresses, Invite, InviteId};
    use org_node::revocation::RevocationNotice;
    use org_node::service::{Checked, OrgService};
    use org_node::store::{MemberSnapshot, OrgRecord, OrgRecordParts, PersonaStore, StoreWrite};
    use org_node::test_fixtures::{device_key, device_secret, member_key};
    use org_node::verify_org::{VerificationTime, Verified};
    use org_node::wire::{encode_message, WireMessage};
    use org_node::{ChainAccount, OrgNodeError, OrgPrivateKey, OrgState};
    use rand::rngs::OsRng;

    use super::support::fixtures::{acme_store, alice, alice_persona, own, record_parts, sealed, service, snapshot, OWN};
    use super::support::{contains, none_held, handle, secret_copy, seed_bytes, the_org, trie_of_members};

    /// An Admin's Active store: a record at epoch 3 listing this Device (Alice) and Bob, with a proxy.
    fn admin() -> OrgService {
        let record = OrgRecord::from(OrgRecordParts {
            proxy_account: Some(ChainAccount::new([0x55; 32])),
            ..record_parts(3, &[alice(), snapshot(2, "bob", 0x11, &[0x21])])
        });
        service(OWN, acme_store(alice_persona(), record))
    }
    fn request(invite: &Invite, member: u8, device: u8) -> Vec<u8> {
        let member_key: PersonPublicKey = member_key(member);
        encode_message(&WireMessage::AdmissionRequest {
            org_id: invite.org_id,
            invite_id: invite.invite_id,
            name: Name::parse("Carol").unwrap(),
            surname: Surname::parse("King").unwrap(),
            preferred_handles: vec![handle("carol")],
            member_key,
            device_key: device_key(device),
            addresses: vec![],
        })
        .unwrap()
    }
    fn nonce(write: &StoreWrite) -> [u8; 24] {
        write.sealed.as_bytes()[..24].try_into().unwrap()
    }


    // verifies: LLR-ay8nvu
    #[test]
    fn every_store_changing_operation_returns_one_sealed_change_with_a_fresh_nonce() {
        let mut svc = admin();
        let mut writes: Vec<(&str, StoreWrite)> = vec![];

        let (invite, change) = svc.build_invite(&mut OsRng, &own(), "Carol".into(), vec![]).unwrap();
        writes.push(("build_invite", sealed(change)));
        let (checked, change) =
            svc.check_received(&mut OsRng, &own(), device_key(0x22), &request(&invite, 0x12, 0x22)).unwrap();
        assert!(matches!(checked, Checked::AdmissionRequest { .. }));
        writes.push(("check_received", sealed(change.expect("an accepted request is kept"))));
        writes.push(("record_admission", sealed(svc.record_admission(&mut OsRng, &own(), invite.invite_id, None).unwrap())));
        writes.push(("record_revocation", sealed(svc.record_revocation(&mut OsRng, &own(), MemberId::new([2; 32])).unwrap())));
        writes.push(("propose_updated_organisation", sealed(svc.propose_updated_organisation(&mut OsRng, &own()).unwrap())));
        let (second, change) = svc.build_invite(&mut OsRng, &own(), "Dave".into(), vec![]).unwrap();
        writes.push(("build_invite", sealed(change)));
        let (_, change) = svc.check_received(&mut OsRng, &own(), device_key(0x23), &request(&second, 0x13, 0x23)).unwrap();
        writes.push(("check_received", sealed(change.unwrap())));
        writes.push(("cancel_invite", sealed(svc.cancel_invite(&mut OsRng, &own(), second.invite_id).unwrap())));
        let proposal =
            svc.provisional_organisations(&own()).unwrap().into_iter().find(|view| view.current_proposal).unwrap();
        let state = OrgState::from_chain(*proposal.root.as_bytes(), *proposal.org_pub_key.as_bytes(), proposal.epoch.get()).unwrap();
        let (verified, change) = svc.verify(&mut OsRng, &own(), Some(state), VerificationTime(1), org_node::AheadLimit(1200), none_held).unwrap();
        assert!(matches!(verified, Verified::Adopted { .. }));
        writes.push(("verify", sealed(change.unwrap())));
        for (name, write) in &writes {
            assert_eq!(write.device_key, own(), "{name}: the changed Persona's store");
        }

        // The founding, its proxy and a provisional Member: one sealed change each, for the new store.
        let (founder, founder_secret, change) = svc.propose_founding(&mut OsRng, "Fay", "Fox", "fay", "Fox Co".into()).unwrap();
        let founded = sealed(change);
        assert_eq!(founded.device_key, founder);
        writes.push(("propose_founding", founded));
        let proxy = sealed(
            svc.record_founding_proxy(&mut OsRng, &founder, OrgId::new([0xbb; 20]), ChainAccount::new([0x56; 32])).unwrap(),
        );
        assert_eq!(proxy.device_key, founder);
        writes.push(("record_founding_proxy", proxy));
        let other = Invite {
            org_name: "Elsewhere".into(),
            org_id: OrgId::new([0xcc; 20]),
            invitee_name: "Gil".into(),
            issuer: device_key(5),
            issuer_addresses: EndpointAddresses::parse(vec![]).unwrap(),
            invite_id: InviteId::new([7; 32]),
        };
        let (member, member_secret, change) = svc.propose_member(&mut OsRng, &other, "Gil", "Gray", &["gil"]).unwrap();
        let joined = sealed(change);
        assert_eq!(joined.device_key, member);
        writes.push(("propose_member", joined));

        // The removal and an accepted acknowledgement. Carol, now listed, sends a notice of this Device's removal, which is kept;
        // `verify` accepts it against a later state whose root lists Carol alone, and seals the removed
        // store with one acknowledgement, for Carol; Carol's acceptance of it seals the removed store again.
        let record = svc.persona_view(&own()).unwrap().record.expect("the adopted record");
        let others: Vec<MemberSnapshot> =
            record.members.iter().filter(|member| !member.device_keys.contains(&own())).cloned().collect();
        let without = trie_of_members(&others);
        let proof = without.prove_absent(&MemberId::new([1; 32]), &own()).unwrap();
        let notice = RevocationNotice { org_id: the_org(), member_id: MemberId::new([1; 32]), device: own(), proof };
        let (checked, change) =
            svc.check_received(&mut OsRng, &own(), device_key(0x22), &encode_message(&WireMessage::Revocation(notice)).unwrap()).unwrap();
        assert!(matches!(checked, Checked::RevocationNoticeKept), "{checked:?}");
        writes.push(("check_received", sealed(change.expect("the kept notice is sealed"))));
        let later = OrgState::from_chain(
            *without.root_hash().unwrap().as_bytes(),
            *OrgPrivateKey::from([0x64; 32]).x25519_keypair().org_public_key().unwrap().as_bytes(),
            record.epoch.get() + 1,
        )
        .unwrap();
        let (verified, change) =
            svc.verify(&mut OsRng, &own(), Some(later), VerificationTime(2), org_node::AheadLimit(1200), |_: &[DevicePublicKey]| vec![device_secret([OWN; 32])]).unwrap();
        assert!(matches!(verified, Verified::Removed { acknowledgements_kept: 1, .. }), "{verified:?}");
        let removal = sealed(change.expect("the removed store is sealed"));
        assert_eq!(removal.device_key, own());
        writes.push(("verify (removal)", removal));
        let (left, change) = svc.acknowledgement_accepted(&mut OsRng, &own(), device_key(0x22)).unwrap();
        assert_eq!(left, 0);
        let accepted = sealed(change);
        assert_eq!(accepted.device_key, own(), "acknowledgement_accepted: the removed Persona's store");
        writes.push(("acknowledgement_accepted", accepted));

        // Each write opens under the key of its own store's Device secret.
        for (name, write) in &writes {
            let secret = match write.device_key {
                device if device == own() => device_secret([OWN; 32]),
                device if device == founder => secret_copy(&founder_secret),
                device if device == member => secret_copy(&member_secret),
                device => panic!("{name}: a write for an unexpected store {device:?}"),
            };
            assert!(PersonaStore::unseal(&write.sealed, write.device_key, secret).is_ok(), "{name}: sealed under its store's key");
        }
        let mut nonces: Vec<[u8; 24]> = writes.iter().map(|(_, write)| nonce(write)).collect();
        nonces.sort();
        nonces.dedup();
        assert_eq!(nonces.len(), writes.len(), "every write draws a fresh 24-byte nonce");
    }

    // verifies: LLR-ay8nvu
    #[test]
    fn store_change_has_no_forgotten_variant_and_a_refusal_returns_none() {
        let service_rs =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/service.rs")).unwrap();
        let start = service_rs.find("pub enum StoreChange").expect("StoreChange is defined in service.rs");
        let body = &service_rs[start..start + service_rs[start..].find('}').unwrap()];
        let variants: Vec<&str> =
            body.lines().skip(1).map(str::trim).filter(|line| !line.is_empty() && !line.starts_with("//")).collect();
        assert_eq!(variants, ["Sealed(StoreWrite),"], "one variant, Sealed; no Forgotten");
        assert!(
            service_rs[..start].trim_end().lines().rev().take(3).any(|line| line.contains("#[must_use")),
            "StoreChange is #[must_use]"
        );

        let mut svc = admin();
        let before = postcard::to_allocvec(svc.store_data(&own()).unwrap()).unwrap();
        assert!(matches!(
            svc.record_revocation(&mut OsRng, &own(), MemberId::new([0x77; 32])),
            Err(OrgNodeError::MemberNotHeld { .. })
        ));
        assert!(svc.propose_updated_organisation(&mut OsRng, &own()).is_err(), "no provisional change");
        assert!(svc.cancel_invite(&mut OsRng, &own(), InviteId::new([0x39; 32])).is_err());
        let in_step = OrgState::from_chain(
            *svc.store_data(&own()).unwrap().record.as_ref().unwrap().root_hash.as_bytes(),
            *OrgPrivateKey::from([0x60; 32]).x25519_keypair().org_public_key().unwrap().as_bytes(),
            3,
        )
        .unwrap();
        let (verified, change) = svc.verify(&mut OsRng, &own(), Some(in_step), VerificationTime(2), org_node::AheadLimit(1200), none_held).unwrap();
        assert!(matches!(verified, Verified::InStep));
        assert!(change.is_none(), "an outcome that changes nothing returns no StoreChange");
        assert_eq!(postcard::to_allocvec(svc.store_data(&own()).unwrap()).unwrap(), before);
    }

    // Normal: neither `build_invite`'s write nor `propose_founding`'s holds
    // the Device seed, sealed or in plaintext.
    // verifies: LLR-ay8nvu
    #[test]
    fn no_sealed_store_holds_the_device_seed() {
        let mut svc = admin();
        let seed = [OWN; 32];
        let (_, change) = svc.build_invite(&mut OsRng, &own(), "Carol".into(), vec![]).unwrap();
        let (founder, secret, founded) = svc.propose_founding(&mut OsRng, "Fay", "Fox", "fay", "Fox Co".into()).unwrap();
        let founder_seed = seed_bytes(&secret);
        for (write, write_seed, write_secret) in [
            (sealed(change), seed.to_vec(), device_secret(seed)),
            (sealed(founded), founder_seed, secret_copy(&secret)),
        ] {
            assert!(!contains(write.sealed.as_bytes(), &write_seed), "no seed in the sealed bytes");
            let opened = PersonaStore::unseal(&write.sealed, write.device_key, write_secret).unwrap();
            let plaintext = postcard::to_allocvec(opened.content()).unwrap();
            assert!(!contains(&plaintext, &write_seed), "nor in the plaintext");
        }
        assert_eq!(svc.store_data(&founder).unwrap().persona.device_key, founder, "the store keeps the public key alone");
    }

    // Normal: the only takers of a DeviceSecret are `unseal`, the two
    // proposals that return a new Device's secret, and `verify`, whose one
    // mention is its device_secrets closure; sealing takes none.
    // verifies: LLR-ay8nvu
    #[test]
    fn saving_needs_no_device_secret() {
        let mut takers = vec![];
        for file in ["store.rs", "service.rs"] {
            for signature in super::support::public_signatures(file) {
                if !signature.contains("DeviceSecret") {
                    continue;
                }
                let name = &signature["pub fn ".len()..];
                takers.push(name[..name.find(['(', '<']).unwrap()].to_string());
                if name.starts_with("verify") {
                    assert_eq!(
                        signature.matches("DeviceSecret").count(),
                        1,
                        "verify names DeviceSecret only in its closure: {signature}"
                    );
                    assert!(signature.contains("-> Vec<DeviceSecret>"));
                }
            }
        }
        takers.sort();
        assert_eq!(
            takers,
            ["propose_founding", "propose_member", "unseal", "verify"],
            "only these take or return a DeviceSecret; sealing takes none"
        );
    }
}
