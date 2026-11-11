//! check_successor: whether one Person definition may follow another.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

mod common;

use person::definition::Person;
use person::successor::check_successor;
use person::IdentityError;

/// "Alice Example" holding `key` and the devices of `seeds`.
fn alice(key: Option<u8>, seeds: &[u8]) -> Person {
    common::person("Alice", "Example", key, seeds)
}

/// verifies: LLR-wqha8d
#[test]
fn a_device_change_with_a_new_key_is_accepted() {
    let previous = alice(Some(9), &[1, 2]);
    for next in [
        alice(Some(10), &[1, 2, 3]), // added
        alice(Some(10), &[1]),       // removed
        alice(Some(10), &[1, 3]),    // replaced
        alice(Some(10), &[3, 4]),    // several at once
        alice(None, &[]),            // every device revoked
    ] {
        assert_eq!(check_successor(&previous, &next), Ok(()), "{next:?}");
    }
    // The first device, from a definition with none.
    assert_eq!(
        check_successor(&alice(None, &[]), &alice(Some(9), &[1])),
        Ok(())
    );
}

/// verifies: LLR-wqha8d
#[test]
fn unchanged_devices_accept_a_kept_or_a_rotated_key() {
    let previous = alice(Some(9), &[1, 2]);
    assert_eq!(check_successor(&previous, &alice(Some(9), &[2, 1])), Ok(()));
    assert_eq!(
        check_successor(&previous, &alice(Some(10), &[1, 2])),
        Ok(())
    );
    let empty = alice(None, &[]);
    assert_eq!(check_successor(&empty, &empty), Ok(()));
}

/// verifies: LLR-wqha8d, LLR-3n3kxx
#[test]
fn a_device_change_that_keeps_the_key_is_refused() {
    let previous = alice(Some(9), &[1, 2]);
    for next in [
        alice(Some(9), &[1, 2, 3]), // added
        alice(Some(9), &[1]),       // removed
        alice(Some(9), &[1, 3]),    // replaced
        alice(Some(9), &[3, 4]),    // several at once
    ] {
        assert_eq!(
            check_successor(&previous, &next),
            Err(IdentityError::PersonKeyNotRotated),
            "{next:?}"
        );
    }
}

/// Only the immediate predecessor is compared (owner, 2026-10-04): a key
/// equal to one held two definitions earlier is accepted.
/// verifies: LLR-wqha8d
#[test]
fn only_the_immediate_predecessor_is_compared() {
    let first = alice(Some(9), &[1]);
    let second = alice(Some(10), &[1, 2]);
    let third = alice(Some(9), &[1, 2, 3]);
    assert_eq!(check_successor(&first, &second), Ok(()));
    assert_eq!(check_successor(&second, &third), Ok(()));
}
