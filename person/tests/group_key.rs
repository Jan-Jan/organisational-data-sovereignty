//! check_group_key: a group key against the device slots it is held with.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

mod common;

use common::{device, device_and_its_bytes_as_a_person_key, person_key, slots};
use person::group_key::check_group_key;
use person::{DevicePublicKey, DeviceSlots, IdentityError, PersonPublicKey};

/// verifies: LLR-sjkmr6, LLR-3n3kxx
#[test]
fn with_no_device_the_key_must_be_absent() {
    assert_eq!(check_group_key(None, &slots(&[])), Ok(()));
    for seed in [1, 9, 200] {
        assert_eq!(
            check_group_key(Some(&person_key(seed)), &slots(&[])),
            Err(IdentityError::KeyWithoutDevice),
            "seed {seed}"
        );
    }
}

/// verifies: LLR-kbhc43, LLR-3n3kxx
#[test]
fn with_one_to_four_devices_the_key_must_be_present() {
    for seeds in [&[1][..], &[1, 2], &[1, 2, 3], &[1, 2, 3, 4]] {
        let devices = slots(seeds);
        assert_eq!(
            check_group_key(None, &devices),
            Err(IdentityError::MissingPersonKey),
            "{seeds:?}"
        );
        assert_eq!(
            check_group_key(Some(&person_key(9)), &devices),
            Ok(()),
            "{seeds:?}"
        );
    }
}

/// The copied key is refused wherever it sorts among up to three other keys.
/// verifies: LLR-kbhc43, LLR-3n3kxx
#[test]
fn a_key_whose_bytes_equal_a_device_key_is_refused() {
    let (copied, key) = device_and_its_bytes_as_a_person_key();
    assert_eq!(copied.as_bytes(), key.as_bytes());
    let others: Vec<DevicePublicKey> = (101..=103)
        .map(device)
        .filter(|other| *other != copied)
        .collect();
    assert_eq!(others.len(), 3);
    for count in 0..=others.len() {
        let mut held = vec![copied];
        held.extend_from_slice(&others[..count]);
        let devices = DeviceSlots::parse(held).expect("within the bound");
        assert_eq!(
            check_group_key(Some(&key), &devices),
            Err(IdentityError::PersonKeyIsDeviceKey),
            "{count} other devices"
        );
    }
    // A key equal to no held device's bytes is accepted alongside the same keys.
    let devices = DeviceSlots::parse(others).expect("three");
    assert_eq!(check_group_key(Some(&key), &devices), Ok(()));
}

/// The comparison is of bytes only (owner, 2026-10-06): a device's ed25519
/// key reused through its X25519 (birational) image has other bytes, and is
/// accepted. Accepted residual risk, recorded here so a change is noticed.
/// verifies: LLR-kbhc43
#[test]
fn the_check_compares_bytes_only() {
    let held = device(5);
    let image = held.verifying_key().to_edwards().to_montgomery().to_bytes();
    assert_ne!(&image, held.as_bytes());
    let key = PersonPublicKey::parse(&image).expect("the image is a valid X25519 key");
    assert_eq!(check_group_key(Some(&key), &slots(&[5])), Ok(()));
}
