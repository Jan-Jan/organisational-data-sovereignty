//! Person::new: the only way to build a Person definition.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

mod common;

use common::{device, device_and_its_bytes_as_a_person_key, person_key, slots};
use person::definition::Person;
use person::group_key::check_group_key;
use person::{DevicePublicKey, DeviceSlots, IdentityError, Name, Surname, MAX_DEVICES};

fn name() -> Name {
    Name::parse("Alice").expect("valid name")
}

fn surname() -> Surname {
    Surname::parse("Example").expect("valid surname")
}

/// verifies: LLR-4ku56h
#[test]
fn a_person_holds_up_to_max_devices_sorted() {
    assert_eq!(MAX_DEVICES, 4);
    let given = vec![device(4), device(1), device(3), device(2)];
    let devices = DeviceSlots::parse(given.clone()).expect("four");
    let person = Person::new(name(), surname(), Some(person_key(9)), devices.clone())
        .expect("valid definition");
    let mut sorted = given;
    sorted.sort();
    assert_eq!(person.devices().devices(), sorted.as_slice());
    assert_eq!(person.devices(), &devices);
    assert_eq!(person.name(), &name());
    assert_eq!(person.surname(), &surname());
    assert_eq!(person.person_key(), Some(&person_key(9)));
}

/// A fifth key or a key given twice is refused while the slots are built, so
/// no Person holding them can exist.
/// verifies: LLR-4ku56h
#[test]
fn over_the_bound_or_a_duplicate_is_refused_before_any_person_exists() {
    let five: Vec<DevicePublicKey> = (1..=5).map(device).collect();
    assert_eq!(
        DeviceSlots::parse(five),
        Err(IdentityError::DeviceSlotsFull)
    );
    assert_eq!(
        DeviceSlots::parse(vec![device(1), device(1)]),
        Err(IdentityError::DuplicateDevice)
    );
    let full = slots(&[1, 2, 3, 4]);
    assert_eq!(
        full.add_device(device(5)),
        Err(IdentityError::DeviceSlotsFull)
    );
}

/// verifies: LLR-sjkmr6, LLR-3n3kxx
#[test]
fn no_devices_and_no_key_is_accepted_and_a_key_without_devices_is_refused() {
    let revoked = Person::new(name(), surname(), None, slots(&[])).expect("no device, no key");
    assert_eq!(revoked.person_key(), None);
    assert_eq!(revoked.devices().device_count(), 0);
    assert_eq!(
        Person::new(name(), surname(), Some(person_key(9)), slots(&[])),
        Err(IdentityError::KeyWithoutDevice)
    );
}

/// verifies: LLR-kbhc43, LLR-3n3kxx
#[test]
fn devices_without_a_key_are_refused() {
    for seeds in [&[1][..], &[1, 2, 3, 4]] {
        assert_eq!(
            Person::new(name(), surname(), None, slots(seeds)),
            Err(IdentityError::MissingPersonKey),
            "{seeds:?}"
        );
    }
}

/// verifies: LLR-kbhc43, LLR-3n3kxx
#[test]
fn a_key_copied_from_a_device_key_is_refused() {
    let (copied, key) = device_and_its_bytes_as_a_person_key();
    let devices = DeviceSlots::parse(vec![copied, device(101)]).expect("two");
    assert_eq!(
        Person::new(name(), surname(), Some(key), devices),
        Err(IdentityError::PersonKeyIsDeviceKey)
    );
    let others = slots(&[101, 102]);
    assert!(Person::new(name(), surname(), Some(key), others).is_ok());
}

/// Person::new accepts exactly what check_group_key accepts, and reports its
/// error, over every combination of key and device count.
/// verifies: LLR-sjkmr6, LLR-kbhc43
#[test]
fn person_new_applies_check_group_key() {
    let (copied, copied_key) = device_and_its_bytes_as_a_person_key();
    let keys = [None, Some(person_key(9)), Some(copied_key)];
    let device_sets = [
        slots(&[]),
        slots(&[101]),
        DeviceSlots::parse(vec![copied]).expect("one"),
        slots(&[101, 102, 103, 104]),
    ];
    for key in keys {
        for devices in &device_sets {
            let expected = check_group_key(key.as_ref(), devices);
            let built = Person::new(name(), surname(), key, devices.clone());
            assert_eq!(built.map(|_| ()), expected, "{key:?} {devices:?}");
        }
    }
}

/// A Person's Debug form carries its Name and Surname in their redacted form.
/// verifies: LLR-fbqs2r, LLR-dtwpr8
#[test]
fn debug_redacts_the_names() {
    let person =
        Person::new(name(), surname(), Some(person_key(9)), slots(&[1])).expect("valid definition");
    let debug = format!("{person:?}");
    assert!(
        !debug.contains("Alice") && !debug.contains("Example"),
        "{debug}"
    );
    assert!(debug.contains("Name([REDACTED])"), "{debug}");
    assert!(debug.contains("Surname([REDACTED])"), "{debug}");
}

/// Names that Debug would escape — a quote, a backslash, a newline — and a
/// definition with no key and no device show in neither raw nor escaped form.
/// verifies: LLR-dtwpr8
#[test]
fn debug_redacts_names_debug_would_escape() {
    let given_name = "Al\"ice\\";
    let given_surname = "Ex\nample";
    let person = Person::new(
        Name::parse(given_name).expect("valid name"),
        Surname::parse(given_surname).expect("valid surname"),
        None,
        slots(&[]),
    )
    .expect("valid definition");
    let debug = format!("{person:?}");
    for shown in [
        given_name.to_string(),
        given_surname.to_string(),
        format!("{given_name:?}"),
        format!("{given_surname:?}"),
        "Al".to_string(),
        "ample".to_string(),
    ] {
        assert!(!debug.contains(&shown), "{shown:?} in {debug}");
    }
}
