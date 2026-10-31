#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The Person store (SDD-27p679): the user's own Person, created from a name
//! and a surname with keys drawn from the caller's random source
//! (LLR-wqu26s), sealed and opened under its own key label (LLR-73sgz2), and
//! named under `org_node::` paths (LLR-5uf4u2).

mod support;

use org_node::store::{PersonaStore, SealedStore, StoreData, StoreRefusal};
use org_node::test_fixtures::{device_secret, persona_record, seal_for_test, seal_person_plaintext_for_test, store_for_test};
use org_node::{DevicePublicKey, Handle, Name, OrgNodeError, Person, PersonPublicKey, PersonStore, PersonStoreContent, Surname};
use rand::rngs::OsRng;
use support::CountingRng;

// Normal: the name and surname are parsed (NFC, as stored), the Person holds
// them, its one Device and its Person key, and no Handle; the secret
// returned is the Person's Device's; the keys come from the source given.
// verifies: LLR-wqu26s
#[test]
fn create_parses_the_name_and_surname_and_holds_no_handle() {
    let mut rng = CountingRng { drawn: 0 };
    let (store, secret, _) = PersonStore::create(&mut rng, "Jose\u{0301}", "Nun\u{0303}ez").unwrap();
    assert!(rng.drawn >= 64, "a Device seed and a Person key pair are drawn from the source given");
    let person_content: &PersonStoreContent = store.content();
    assert_eq!((person_content.person.name().as_str(), person_content.person.surname().as_str()), ("Jos\u{e9}", "Nu\u{f1}ez"));
    assert_eq!(person_content.device_key, secret.device_key());
    assert_eq!(store.device_key(), secret.device_key());
    assert_eq!(person_content.person.person_key(), Some(&person_content.person_key));
    assert_eq!(person_content.person.devices().devices(), [person_content.device_key].as_slice());
    assert_ne!(person_content.person_key.as_bytes(), person_content.device_key.as_bytes());
    assert_eq!(
        person_content.person_seed_for_test().x25519_keypair().member_key().unwrap(),
        person_content.person_key,
        "the Person key is the kept Person seed's public half"
    );
    let rendered = format!("{person_content:?}");
    assert!(!rendered.contains("Handle") && !rendered.contains("handle"), "no Handle: {rendered}");
    let seed_bytes = format!("{:?}", person_content.person_seed_for_test().expose_secret());
    assert!(!rendered.contains(&seed_bytes), "the Person seed is redacted: {rendered}");
    let (other, other_secret, _) = PersonStore::create(&mut OsRng, "Ann", "Lee").unwrap();
    assert_ne!(other.content().person_key, person_content.person_key, "drawn, not derived");
    assert_ne!(other_secret.device_key(), secret.device_key());
}

// Abnormal: a name or a surname its type refuses is refused naming the
// field, before any draw from the source; no store is created.
// verifies: LLR-wqu26s
#[test]
fn create_refuses_an_invalid_name_or_surname_naming_the_field_and_creates_nothing() {
    let over = "a".repeat(129);
    for (name, surname, field) in [(over.as_str(), "Lee", "person.name"), ("Ann", over.as_str(), "person.surname")] {
        let mut rng = CountingRng { drawn: 0 };
        let refused = PersonStore::create(&mut rng, name, surname).err().expect("must be refused");
        assert!(matches!(&refused, OrgNodeError::InvalidField { field: named, .. } if *named == field), "{refused:?}");
        assert_eq!(rng.drawn, 0, "{field}: nothing drawn, nothing created");
    }
    let refused = PersonStore::create(&mut OsRng, &over, &over).err().expect("must be refused");
    assert!(matches!(&refused, OrgNodeError::InvalidField { field: "person.name", .. }), "the first in order: {refused:?}");
}

// Normal: a sealed Person store unseals, with its secret, to the same Person
// and keys; each seal draws a fresh nonce. Abnormal: another Device's
// secret, too few bytes and a flipped bit are refused as a Persona store's
// are.
// verifies: LLR-73sgz2
#[test]
fn a_sealed_person_store_unseals_to_the_same_person() {
    let (store, secret, sealed) = PersonStore::create(&mut OsRng, "Ann", "Lee").unwrap();
    let reopened = PersonStore::unseal(&sealed, store.device_key(), support::secret_copy(&secret)).unwrap();
    assert_eq!(reopened.content(), store.content());
    assert_eq!(
        reopened.content().person_seed_for_test(),
        store.content().person_seed_for_test(),
        "the Person seed is sealed with the rest and read back unchanged"
    );
    let again = store.seal(&mut OsRng).unwrap();
    assert_ne!(again.as_bytes()[..24], sealed.as_bytes()[..24], "a fresh nonce");
    assert_eq!(PersonStore::unseal(&again, store.device_key(), support::secret_copy(&secret)).unwrap().content(), store.content());

    let other = device_secret([0x51; 32]);
    assert_eq!(
        PersonStore::unseal(&sealed, store.device_key(), other).err(),
        Some(OrgNodeError::DeviceSecretMismatch)
    );
    let short = SealedStore::from_bytes(vec![0; 39]);
    assert_eq!(
        PersonStore::unseal(&short, store.device_key(), support::secret_copy(&secret)).err(),
        Some(OrgNodeError::StoreRefused(StoreRefusal::TooShort))
    );
    let mut flipped = sealed.as_bytes().to_vec();
    flipped[30] ^= 1;
    assert_eq!(
        PersonStore::unseal(&SealedStore::from_bytes(flipped), store.device_key(), support::secret_copy(&secret)).err(),
        Some(OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic))
    );
}

// Abnormal: a Person store that authenticates under its key is still
// refused when its plaintext does not decode (`Undecodable`), when it holds
// a name, surname, Device key or Person key its type refuses (`InvalidField`
// naming the `person.*` field: every field is parsed again), and when the
// Device key it holds is not the one given (`DeviceSecretMismatch`).
// Normal: the same encoding of the store's own values unseals.
// verifies: LLR-73sgz2
#[test]
fn an_authentic_person_store_holding_a_refused_value_or_another_device_key_is_refused() {
    let (store, secret, _) = PersonStore::create(&mut OsRng, "Ann", "Lee").unwrap();
    let device_key = store.device_key();
    let content = store.content();
    let device = *device_key.as_bytes();
    let person = *content.person_key.as_bytes();
    let seed = content.person_seed_for_test();
    let unseal = |name: &str, surname: &str, device: [u8; 32], person: [u8; 32]| {
        let plaintext = postcard::to_allocvec(&(name, surname, device, person, seed)).unwrap();
        let sealed = seal_person_plaintext_for_test(&secret, device_key, &plaintext, &mut OsRng);
        PersonStore::unseal(&sealed, device_key, support::secret_copy(&secret))
    };
    assert_eq!(unseal("Ann", "Lee", device, person).unwrap().content(), content, "fixture: the encoding is the store's");

    let too_long = "a".repeat(129);
    // y = 2 has no x on edwards25519: no Device key.
    let mut no_point = [0u8; 32];
    no_point[0] = 2;
    for (refused, field) in [
        (unseal(&too_long, "Lee", device, person), "person.name"),
        (unseal("Ann", &too_long, device, person), "person.surname"),
        (unseal("Ann", "Lee", no_point, person), "person.device_key"),
        (unseal("Ann", "Lee", device, [0u8; 32]), "person.person_key"),
    ] {
        assert!(
            matches!(&refused, Err(OrgNodeError::InvalidField { field: named, .. }) if *named == field),
            "{field}: {:?}",
            refused.err()
        );
    }

    let undecodable = seal_person_plaintext_for_test(&secret, device_key, &[0xff; 3], &mut OsRng);
    assert_eq!(
        PersonStore::unseal(&undecodable, device_key, support::secret_copy(&secret)).err(),
        Some(OrgNodeError::StoreRefused(StoreRefusal::Undecodable))
    );

    let other_device = *device_secret([0x51; 32]).device_key().as_bytes();
    assert_eq!(unseal("Ann", "Lee", other_device, person).err(), Some(OrgNodeError::DeviceSecretMismatch));
}

// Abnormal: under one Device secret, a Persona store's sealed bytes do not
// open as a Person store, nor a Person store's as a Persona store: the two
// keys are derived under different labels.
// verifies: LLR-73sgz2
#[test]
fn a_persona_store_never_opens_as_a_person_store_or_the_reverse() {
    let (person, secret, person_sealed) = PersonStore::create(&mut OsRng, "Ann", "Lee").unwrap();
    let device_key: DevicePublicKey = person.device_key();
    let mut persona = persona_record("p1", 0x11, 0);
    persona.device_key = device_key;
    let persona_store = store_for_test(&support::secret_copy(&secret), StoreData::new(persona));
    let persona_sealed = seal_for_test(&persona_store, &mut OsRng).sealed;
    assert_eq!(
        PersonStore::unseal(&persona_sealed, device_key, support::secret_copy(&secret)).err(),
        Some(OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic)),
        "a Persona store is not a Person store"
    );
    assert_eq!(
        PersonaStore::unseal(&person_sealed, device_key, support::secret_copy(&secret)).err(),
        Some(OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic)),
        "a Person store is not a Persona store"
    );
    assert!(PersonaStore::unseal(&persona_sealed, device_key, support::secret_copy(&secret)).is_ok());
}

// Normal: every type org-io needs for Persons and Members is named under an
// `org_node::` path, and each is the very type `person` and `org-members`
// define (so no conversion is needed and org-io names no `person::` path).
// verifies: LLR-5uf4u2
#[test]
fn the_person_and_member_types_are_named_under_org_node_paths() {
    fn same<T>(_: std::marker::PhantomData<T>, _: std::marker::PhantomData<T>) {}
    same(std::marker::PhantomData::<org_node::Name>, std::marker::PhantomData::<person::Name>);
    same(std::marker::PhantomData::<org_node::Surname>, std::marker::PhantomData::<person::Surname>);
    same(std::marker::PhantomData::<org_node::DevicePublicKey>, std::marker::PhantomData::<person::DevicePublicKey>);
    same(std::marker::PhantomData::<org_node::PersonPublicKey>, std::marker::PhantomData::<person::PersonPublicKey>);
    same(std::marker::PhantomData::<org_node::Person>, std::marker::PhantomData::<person::Person>);
    same(std::marker::PhantomData::<org_node::Handle>, std::marker::PhantomData::<org_members::Handle>);
    let (store, _, _): (PersonStore, _, _) = PersonStore::create(&mut OsRng, "Ann", "Lee").unwrap();
    let person_content: &PersonStoreContent = store.content();
    let _: (&Person, &DevicePublicKey, &PersonPublicKey) = (&person_content.person, &person_content.device_key, &person_content.person_key);
    let _: (&Name, &Surname) = (person_content.person.name(), person_content.person.surname());
    let _: org_node::PersonView = store.view();
    assert!(Handle::parse("ann").is_ok());
    let lib = include_str!("../src/lib.rs");
    for reexport in ["PersonStore, PersonStoreContent, PersonView", "pub use person::Person;", "Handle", "Name", "Surname", "DevicePublicKey", "PersonPublicKey"] {
        assert!(lib.contains(reexport), "org_node re-exports {reexport}");
    }
}
