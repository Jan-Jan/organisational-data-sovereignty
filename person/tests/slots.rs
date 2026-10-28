//! DeviceSlots: bound, order, duplicates, positions, add and remove.

#![allow(clippy::expect_used)]

mod common;

use common::device;
use person::{DevicePublicKey, DeviceSlots, IdentityError, MAX_DEVICES};

fn eight() -> Vec<DevicePublicKey> {
    (1..=8).map(device).collect()
}

/// verifies: LLR-xsx6gg, REQ-4szc22
#[test]
fn parse_sorts_and_accepts_zero_to_the_bound() {
    assert_eq!(MAX_DEVICES, 8);
    assert_eq!(DeviceSlots::parse(vec![]).expect("empty").device_count(), 0);
    let mut given = eight();
    given.reverse();
    let slots = DeviceSlots::parse(given.clone()).expect("eight devices");
    given.sort();
    assert_eq!(slots.devices(), given.as_slice());
}

/// verifies: LLR-xsx6gg, REQ-4szc22, REQ-4e23xr
#[test]
fn parse_refuses_by_rule_in_order() {
    let mut nine = eight();
    nine.push(device(9));
    assert_eq!(DeviceSlots::parse(nine), Err(IdentityError::DeviceSlotsFull));
    assert_eq!(DeviceSlots::parse(vec![device(1), device(1)]), Err(IdentityError::DuplicateDevice));
    // 17 % 16 == 1: two keys of one position.
    assert_eq!(DeviceSlots::parse(vec![device(1), device(17)]), Err(IdentityError::DevicePositionTaken));
    // Order of the checks: the bound, then duplicates, then positions.
    let mut nine_colliding = eight();
    nine_colliding.push(device(17));
    assert_eq!(DeviceSlots::parse(nine_colliding), Err(IdentityError::DeviceSlotsFull));
    assert_eq!(
        DeviceSlots::parse(vec![device(1), device(17), device(1)]),
        Err(IdentityError::DuplicateDevice)
    );
}

/// verifies: LLR-xsx6gg, LLR-vraz7n, REQ-4szc22, REQ-9sh8qb
#[test]
fn add_and_remove_return_new_sets_and_refuse_by_rule() {
    let one = DeviceSlots::parse(vec![device(2)]).expect("one");
    assert!(one.position_taken(&device(18)), "18 % 16 == 2");
    assert!(!one.position_taken(&device(2)), "a key does not take its own position");
    assert!(!one.position_taken(&device(3)));
    assert_eq!(one.add_device(device(18)), Err(IdentityError::DevicePositionTaken));
    assert_eq!(one.device_count(), 1, "refusals leave the set unchanged");
    let two = one.add_device(device(1)).expect("add");
    let mut expected = vec![device(1), device(2)];
    expected.sort();
    assert_eq!(two.devices(), expected.as_slice());
    assert_eq!(two.add_device(device(1)), Err(IdentityError::DuplicateDevice));
    let full = DeviceSlots::parse(eight()).expect("full");
    assert_eq!(full.add_device(device(9)), Err(IdentityError::DeviceSlotsFull));
    assert_eq!(full.add_device(device(17)), Err(IdentityError::DeviceSlotsFull), "the bound first");
    assert_eq!(two.add_device(device(2)), Err(IdentityError::DuplicateDevice), "a duplicate before its position");
    assert_eq!(one.remove_device(&device(9)), Err(IdentityError::DeviceNotFound));
    let none = one.remove_device(&device(2)).expect("remove");
    assert_eq!(none.device_count(), 0);
    assert_eq!(one.device_count(), 1);
}

/// Each add inserts a key that sorts before, after or between the keys
/// already held; the set stays sorted and every held key is found.
/// verifies: LLR-xsx6gg, REQ-4szc22
#[test]
fn add_keeps_the_set_sorted_wherever_the_key_falls() {
    let mut by_bytes: Vec<DevicePublicKey> = (1..=4).map(device).collect();
    by_bytes.sort_by(|first, second| first.as_bytes().cmp(second.as_bytes()));
    let [lowest, low, high, highest]: [DevicePublicKey; 4] = by_bytes.clone().try_into().expect("four keys");
    let mut slots = DeviceSlots::parse(vec![low]).expect("one");
    for added in [lowest, highest, high] {
        slots = slots.add_device(added).expect("within bound");
        let held = slots.devices();
        assert!(held.windows(2).all(|pair| matches!(pair, [first, second] if first.as_bytes() < second.as_bytes())));
        for device in held {
            assert!(slots.has_device(device));
            let removed = slots.remove_device(device).expect("held key is found");
            assert_eq!(removed.device_count(), held.len() - 1);
            assert!(!removed.has_device(device));
        }
    }
    assert_eq!(slots.devices(), by_bytes.as_slice());
}

/// verifies: LLR-xsx6gg, REQ-4szc22
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

/// verifies: LLR-xsx6gg, REQ-4szc22
#[test]
fn try_from_goes_through_parse() {
    let mut nine = eight();
    nine.push(device(9));
    let inputs = [
        vec![device(2), device(1)],
        vec![device(1), device(1)],
        vec![device(1), device(17)],
        nine,
    ];
    for devices in inputs {
        assert_eq!(DeviceSlots::try_from(devices.clone()), DeviceSlots::parse(devices));
    }
    let valid = vec![device(3)];
    let accepted = DeviceSlots::try_from(valid.clone()).expect("a valid set is accepted");
    assert_eq!(Ok(accepted), DeviceSlots::parse(valid));
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
