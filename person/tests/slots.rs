//! DeviceSlots: bound, order, duplicates, add and remove.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use ed25519_dalek::SigningKey;
use person::{DevicePublicKey, DeviceSlots, IdentityError, MAX_DEVICES};

fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
        .expect("valid key")
}

/// verifies: LLR-9p34cv, REQ-4szc22
#[test]
fn parse_sorts_and_accepts_zero_to_the_bound() {
    assert_eq!(MAX_DEVICES, 4);
    assert_eq!(DeviceSlots::parse(vec![]).expect("empty").device_count(), 0);
    let given = vec![device(4), device(1), device(3), device(2)];
    let slots = DeviceSlots::parse(given.clone()).expect("four devices");
    let mut sorted = given;
    sorted.sort();
    assert_eq!(slots.devices(), sorted.as_slice());
}

/// verifies: LLR-9p34cv, REQ-4szc22
#[test]
fn parse_rejects_over_the_bound_and_duplicates() {
    let five = vec![device(1), device(2), device(3), device(4), device(5)];
    assert_eq!(
        DeviceSlots::parse(five),
        Err(IdentityError::DeviceSlotsFull)
    );
    assert_eq!(
        DeviceSlots::parse(vec![device(1), device(1)]),
        Err(IdentityError::DuplicateDevice)
    );
}

/// verifies: LLR-9p34cv, REQ-4szc22
#[test]
fn add_and_remove_return_new_sets_and_reject_by_rule() {
    let one = DeviceSlots::parse(vec![device(2)]).expect("one");
    let two = one.add_device(device(1)).expect("add");
    assert_eq!(one.device_count(), 1);
    let mut expected = vec![device(1), device(2)];
    expected.sort();
    assert_eq!(two.devices(), expected.as_slice());
    assert_eq!(
        two.add_device(device(1)),
        Err(IdentityError::DuplicateDevice)
    );
    let full = DeviceSlots::parse(vec![device(1), device(2), device(3), device(4)]).expect("full");
    assert_eq!(
        full.add_device(device(5)),
        Err(IdentityError::DeviceSlotsFull)
    );
    assert_eq!(
        one.remove_device(&device(9)),
        Err(IdentityError::DeviceNotFound)
    );
    let none = one.remove_device(&device(2)).expect("remove");
    assert_eq!(none.device_count(), 0);
    assert_eq!(one.device_count(), 1);
}

/// Each add inserts a key that sorts before, after or between the keys
/// already held, by byte order; the set stays sorted and every key held is
/// found by `has_device` and `remove_device`.
///
/// verifies: LLR-9p34cv, REQ-4szc22
#[test]
fn add_keeps_the_set_sorted_wherever_the_key_falls() {
    let mut by_bytes: Vec<DevicePublicKey> = (1..=4).map(device).collect();
    by_bytes.sort_by(|first, second| first.as_bytes().cmp(second.as_bytes()));
    let [lowest, low, high, highest]: [DevicePublicKey; 4] =
        by_bytes.clone().try_into().expect("four keys");
    let mut slots = DeviceSlots::parse(vec![low]).expect("one");
    // before, after, then between the keys already held
    for added in [lowest, highest, high] {
        slots = slots.add_device(added).expect("within bound");
        let held = slots.devices();
        assert!(
            held.windows(2).all(
                |pair| matches!(pair, [first, second] if first.as_bytes() < second.as_bytes())
            ),
            "devices() not in increasing byte order after an add"
        );
        for device in held {
            assert!(slots.has_device(device));
            let removed = slots.remove_device(device).expect("held key is found");
            assert_eq!(removed.device_count(), held.len() - 1);
            assert!(!removed.has_device(device));
        }
    }
    assert_eq!(slots.devices(), by_bytes.as_slice());
}

/// verifies: LLR-9p34cv, REQ-4szc22
#[test]
fn has_device_is_membership_in_the_set() {
    let empty = DeviceSlots::parse(vec![]).expect("empty");
    assert!(!empty.has_device(&device(1)));
    let slots = DeviceSlots::parse(vec![device(3), device(1), device(2)]).expect("three");
    for seed in 1..=3 {
        assert!(slots.has_device(&device(seed)), "seed {seed}");
    }
    for seed in 4..=6 {
        assert!(!slots.has_device(&device(seed)), "seed {seed}");
    }
}

/// verifies: LLR-9p34cv, REQ-4szc22
#[test]
fn try_from_goes_through_parse() {
    let inputs = [
        vec![device(2), device(1)],
        vec![device(1), device(1)],
        vec![device(1), device(2), device(3), device(4), device(5)],
    ];
    for devices in inputs {
        assert_eq!(
            DeviceSlots::try_from(devices.clone()),
            DeviceSlots::parse(devices)
        );
    }
    assert!(DeviceSlots::try_from(vec![device(3)]).is_ok());
}

/// verifies: LLR-a6krbh
#[test]
fn debug_is_the_count_and_names_no_key() {
    for count in 0..=MAX_DEVICES {
        let seeds = 1..=u8::try_from(count).expect("small");
        let slots = DeviceSlots::parse(seeds.map(device).collect()).expect("within bound");
        assert_eq!(format!("{slots:?}"), format!("DeviceSlots({count})"));
    }
}
