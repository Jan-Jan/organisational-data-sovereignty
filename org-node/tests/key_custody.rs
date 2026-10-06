#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Device and member key custody (SDD-sxp8hb).
//!
//! Relocated from the `#[cfg(test)]` module in `org-node/src/keys.rs` so the
//! `verifies:` annotations sit under `test_paths`.
//!
//! *Merged 2026-10-05 into worktree-person-shared-types.* On this branch a
//! DevicePublicKey is ed25519 and signs nothing, and a Member-as-a-group key is an
//! X25519 key-agreement key drawn from its own seed (`X25519Keypair`); the
//! Envelope carries no signature (REQ-ag6kqm, amended in place). Master's
//! cases for `SigningKeypair::sign` and `keys::verify` (LLR-e58j8m's signature
//! clause, LLR-na7p4w, LLR-9fvb3y) and for one verifying key playing both
//! roles (LLR-ctzkv7) have no code left to test: `sign`, `verify` and
//! `member_key` on the device keypair do not exist here. The X25519 cases are
//! this branch's, relocated from `src/keys.rs` with them.
//!
//! *Amended 2026-10-05.* SDD-sxp8hb, LLR-e58j8m and LLR-ctzkv7 are amended in
//! place to this branch's key custody, and the tests below verify them as
//! amended.

use org_node::keys::{SigningKeypair, X25519Keypair};
use org_node::{DeviceSeed, MemberSeed};
use rand::rngs::OsRng;

// Ported 2026-10-05 to master's org-node type-safety change: no keypair takes
// or returns a plain-array seed (LLR-56hc77), so each keypair here is built
// from a typed seed, and a rebuild goes through the seed type the keypair
// hands back.

fn hex32(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
    }
    out
}

// Was `a_keypair_rebuilt_from_its_seed_signs_identically` (REQ-ag6kqm,
// LLR-e58j8m): the signature half is gone with `sign`; the verifying-key half
// is what remains, and LLR-e58j8m, as amended, states it.
// verifies: LLR-e58j8m
#[test]
fn a_device_keypair_rebuilt_from_its_seed_has_the_same_device_key() {
    let original = DeviceSeed::from([3u8; 32]).signing_keypair();
    let rebuilt = original.device_seed().signing_keypair();
    assert_eq!(original.verifying_key(), rebuilt.verifying_key());
    assert_eq!(original.device_key().unwrap(), rebuilt.device_key().unwrap());
}

// Relocated from `src/keys.rs` (`device_key_wraps_the_verifying_key`).
// verifies: LLR-ctzkv7
#[test]
fn device_key_wraps_the_verifying_key() {
    let kp = SigningKeypair::generate(&mut OsRng);
    assert_eq!(kp.device_key().unwrap().as_bytes(), kp.verifying_key().as_bytes());
}

// Was annotated `REQ-ztdza4, LLR-ctzkv7`. LLR-ctzkv7 (member key and device
// key wrap one verifying key) does not hold on this branch; the distinctness
// of two keypairs' DevicePublicKeys does. No requirement states this since
// REQ-ztdza4's amendment; kept as a regression check.
#[test]
fn two_different_keypairs_do_not_share_a_device_key() {
    let a = DeviceSeed::from([5u8; 32]).signing_keypair();
    let b = DeviceSeed::from([6u8; 32]).signing_keypair();
    assert_ne!(a.device_key().unwrap().as_bytes(), b.device_key().unwrap().as_bytes());
}

// RFC 7748 §6.1: Alice's and Bob's private keys and X25519(k, 9).
// Relocated from `src/keys.rs`.
// verifies: LLR-ctzkv7
#[test]
fn x25519_public_key_matches_rfc_7748() {
    let alice = MemberSeed::from(hex32(
        "77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a",
    ))
    .x25519_keypair();
    assert_eq!(
        alice.public_bytes(),
        hex32("8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a")
    );
    let bob = MemberSeed::from(hex32(
        "5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb",
    ))
    .x25519_keypair();
    assert_eq!(
        bob.public_bytes(),
        hex32("de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f")
    );
}

// Relocated from `src/keys.rs`.
// verifies: LLR-ctzkv7
#[test]
fn member_key_is_the_x25519_public_key_and_always_valid() {
    for _ in 0..256 {
        let kp = X25519Keypair::generate(&mut OsRng);
        let key = kp.member_key().expect("an X25519 public key is a valid PersonPublicKey");
        assert_eq!(key.as_bytes(), &kp.public_bytes());
    }
}

// Relocated from `src/keys.rs`.
// *Annotated 2026-10-05 by review round 4 (gate notes).* It is LLR-98ufry's
// normal case: a key pair that cannot be cloned still hands its secret back as
// a seed that rebuilds the same key pair.
// verifies: LLR-e58j8m, LLR-98ufry
#[test]
fn x25519_seed_round_trip_preserves_key() {
    let kp = X25519Keypair::generate(&mut OsRng);
    assert_eq!(kp.member_seed().x25519_keypair().public_bytes(), kp.public_bytes());
}

/// A trait probe for `Clone`, resolved at compile time.
/// `CloneProbe::<T>::IS_CLONE` names the inherent constant when `T: Clone`,
/// which takes precedence over a trait's, and otherwise falls back to the
/// trait's default, `false`. This is the pattern of the `impls` crate.
struct CloneProbe<T: ?Sized>(std::marker::PhantomData<T>);

trait NotCloneFallback {
    const IS_CLONE: bool = false;
}

impl<T: ?Sized> NotCloneFallback for CloneProbe<T> {}

impl<T: ?Sized + Clone> CloneProbe<T> {
    const IS_CLONE: bool = true;
}

// Checked when the test target compiles: deriving or implementing `Clone` for
// `X25519Keypair` fails this target's build.
const _: () = assert!(!CloneProbe::<X25519Keypair>::IS_CLONE, "X25519Keypair implements Clone");

// LLR-98ufry's "cannot be cloned" clause (review round 4, gate notes). The
// probe is checked against types whose answer is known, so a probe that
// always answered `false` would fail here: `[u8; 32]` and `MemberSeed` are
// `Clone`, `X25519Keypair` is not.
// verifies: LLR-98ufry
#[test]
fn an_x25519_key_pair_cannot_be_cloned() {
    assert!(CloneProbe::<[u8; 32]>::IS_CLONE, "the probe sees a Clone type");
    assert!(CloneProbe::<MemberSeed>::IS_CLONE, "the probe sees a Clone secret type");
    assert!(!CloneProbe::<X25519Keypair>::IS_CLONE, "an X25519 key pair is not Clone");
}

// Relocated from `src/keys.rs`. Annotated 2026-10-05 by review round 2
// (finding-22): LLR-98ufry states it.
// verifies: LLR-98ufry
#[test]
fn x25519_debug_does_not_print_the_secret() {
    let kp = MemberSeed::from([0xabu8; 32]).x25519_keypair();
    assert_eq!(format!("{kp:?}"), "X25519Keypair(..)");
}

/// Compiles only for a type that declares the `ZeroizeOnDrop` contract. The
/// declaration is a marker: it does not show that `Drop` calls `zeroize`.
fn declares_zeroize_on_drop<T: zeroize::ZeroizeOnDrop>() {}

// An X25519 key pair, which holds a member seed or an Organisation private key,
// declares the `ZeroizeOnDrop` contract, and an explicit `zeroize` leaves no
// secret byte (review round 2, finding-22). That `Drop` calls `zeroize` is not
// observed here: no safe test can read the memory of a dropped value, so that
// clause of LLR-98ufry is verified by inspection (review round 3, finding-1).
// verifies: LLR-98ufry
#[test]
fn an_x25519_key_pair_declares_zeroize_on_drop_and_zeroize_clears_its_secret() {
    use zeroize::Zeroize;
    declares_zeroize_on_drop::<X25519Keypair>();
    let mut kp = MemberSeed::from([0xabu8; 32]).x25519_keypair();
    kp.zeroize();
    assert_eq!(kp.member_seed().expose_secret(), &[0u8; 32], "no secret byte is left");
}
