#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Device and member key custody (SDD-sxp8hb).
//!
//! Relocated from the `#[cfg(test)]` module in `org-node/src/keys.rs` so the
//! `verifies:` annotations sit under `test_paths`.

use org_node::keys::verify;
use org_node::{DeviceSeed, MemberSeed};

// Ported 2026-10-05 to the org-node type-safety change: `SigningKeypair` no
// longer takes or returns a plain-array seed (LLR-56hc77), so each keypair here is
// built from a typed seed, and the rebuild below goes through the seed type the
// keypair hands back (LLR-e58j8m, amended).

// verifies: REQ-ag6kqm, LLR-e58j8m
#[test]
fn a_keypair_rebuilt_from_its_seed_signs_identically() {
    let original = MemberSeed::from([3u8; 32]).signing_keypair();
    let msg = b"the transcript that gets signed";
    // Both seed roles a keypair hands its seed back as.
    for rebuilt in [original.member_seed().signing_keypair(), original.device_seed().signing_keypair()] {
        assert_eq!(original.verifying_key(), rebuilt.verifying_key());
        assert_eq!(
            original.sign(msg).to_bytes(),
            rebuilt.sign(msg).to_bytes(),
            "ed25519 is deterministic; a rebuilt keypair must produce the same signature"
        );
    }
}

// verifies: REQ-ztdza4, REQ-xa6smf, LLR-ctzkv7
#[test]
fn member_key_and_device_key_wrap_this_keypairs_one_verifying_key() {
    let kp = MemberSeed::from([5u8; 32]).signing_keypair();
    let vk = kp.verifying_key();
    assert_eq!(kp.member_key().as_bytes(), vk.as_bytes());
    assert_eq!(kp.device_key().as_bytes(), vk.as_bytes());
    // The two roles are the same key, which is the property the trie and the
    // transport both depend on.
    assert_eq!(kp.member_key().as_bytes(), kp.device_key().as_bytes());
}

// verifies: REQ-ztdza4, LLR-ctzkv7
#[test]
fn two_different_keypairs_do_not_share_a_device_key() {
    let a = DeviceSeed::from([5u8; 32]).signing_keypair();
    let b = DeviceSeed::from([6u8; 32]).signing_keypair();
    assert_ne!(a.device_key().as_bytes(), b.device_key().as_bytes());
}

// verifies: REQ-ag6kqm, LLR-na7p4w
#[test]
fn a_signature_verifies_under_the_key_that_made_it() {
    let kp = MemberSeed::from([9u8; 32]).signing_keypair();
    let msg = b"org-bound, seq-bound delta";
    let sig = kp.sign(msg);
    assert!(verify(&kp.verifying_key(), msg, &sig));
}

// verifies: REQ-ag6kqm, LLR-9fvb3y
#[test]
fn a_signature_does_not_verify_under_another_key() {
    let signer = MemberSeed::from([9u8; 32]).signing_keypair();
    let other = MemberSeed::from([10u8; 32]).signing_keypair();
    let msg = b"org-bound, seq-bound delta";
    let sig = signer.sign(msg);
    assert!(!verify(&other.verifying_key(), msg, &sig));
}

// verifies: REQ-ag6kqm, LLR-na7p4w
#[test]
fn a_signature_does_not_verify_over_a_different_message() {
    let kp = MemberSeed::from([9u8; 32]).signing_keypair();
    let sig = kp.sign(b"the original message");
    assert!(!verify(&kp.verifying_key(), b"a different message", &sig));
}
