//! The guard that decides whether a receiver loop may start.
//!
//! Requirements carried here: REQ-6hgm8r (at most one receiver loop runs at a
//! time) and REQ-3hfggn (the slot is released however the loop ends, so a
//! restart is possible).
//!
//! The previous code set a flag before spawning and never cleared it, so an
//! exited loop kept the slot for the life of the process (HAZ-cfp4jb).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};

use ods_poc_lib::events::StartGuard;

fn flag() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

// verifies: REQ-6hgm8r
#[test]
fn first_claim_succeeds() {
    let flag = flag();
    let guard = StartGuard::try_claim(&flag);
    assert!(guard.is_some(), "nothing is running, so a loop may start");
    assert!(flag.load(Ordering::Acquire), "the slot is marked as taken");
}

// verifies: REQ-6hgm8r
#[test]
fn second_claim_while_held_fails() {
    let flag = flag();
    let _first = StartGuard::try_claim(&flag).expect("first claim");
    assert!(
        StartGuard::try_claim(&flag).is_none(),
        "a second receiver loop must not be startable while the first holds the slot"
    );
}

// verifies: REQ-3hfggn
#[test]
fn claim_succeeds_again_after_the_guard_drops() {
    let flag = flag();
    {
        let _first = StartGuard::try_claim(&flag).expect("first claim");
    }
    assert!(
        !flag.load(Ordering::Acquire),
        "dropping the guard releases the slot"
    );
    assert!(
        StartGuard::try_claim(&flag).is_some(),
        "a finished loop must be restartable"
    );
}

// verifies: REQ-3hfggn
#[test]
fn guard_releases_when_dropped_by_panic() {
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let f = std::sync::Arc::clone(&flag);
    let _ = std::panic::catch_unwind(move || {
        let _g = StartGuard::try_claim(&f).expect("first claim");
        panic!("the receiver loop panicked");
    });
    assert!(
        StartGuard::try_claim(&flag).is_some(),
        "a panicking loop must not keep the slot claimed for the life of the process"
    );
}

// verifies: REQ-6hgm8r
#[test]
fn concurrent_claims_yield_exactly_one_guard() {
    // The abnormal-input case for REQ-6hgm8r: sixteen callers racing on one
    // flag. A read-then-write guard would let several through here, which is
    // exactly the double-spawning the rewiring must not reintroduce.
    let flag = flag();
    let barrier = Arc::new(Barrier::new(16));
    let handles: Vec<_> = (0..16)
        .map(|_| {
            let f = Arc::clone(&flag);
            let b = Arc::clone(&barrier);
            std::thread::spawn(move || {
                b.wait();
                StartGuard::try_claim(&f)
            })
        })
        .collect();
    let guards: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("no claimant panics"))
        .collect();
    assert_eq!(
        guards.iter().filter(|g| g.is_some()).count(),
        1,
        "exactly one of sixteen racing callers may start a receiver loop"
    );
}

// verifies: REQ-3hfggn
#[test]
fn release_then_concurrent_claims_yield_exactly_one_guard() {
    // Release and re-claim must compose: after a loop ends, the next race is
    // decided the same way rather than letting every waiting caller through.
    let flag = flag();
    {
        let _first = StartGuard::try_claim(&flag).expect("first claim");
    }
    let barrier = Arc::new(Barrier::new(8));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let f = Arc::clone(&flag);
            let b = Arc::clone(&barrier);
            std::thread::spawn(move || {
                b.wait();
                StartGuard::try_claim(&f)
            })
        })
        .collect();
    let guards: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("no claimant panics"))
        .collect();
    assert_eq!(
        guards.iter().filter(|g| g.is_some()).count(),
        1,
        "a released slot is claimable exactly once"
    );
}
