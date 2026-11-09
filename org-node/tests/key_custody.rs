#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Device and member key custody (SDD-sxp8hb), tested here so the
//! `verifies:` annotations sit under `test_paths`.
//!
//! A DevicePublicKey is ed25519 and signs nothing, and a Member-as-a-group
//! key is an X25519 key-agreement key drawn from its own seed
//! (`X25519Keypair`); the Envelope carries no signature (LLR-e7s4ye).

use org_node::keys::{DeviceSecret, SigningKeypair, X25519Keypair};
use org_node::MemberSeed;
use rand::rngs::OsRng;
use zeroize::Zeroizing;

mod support;
use support::{CloneProbe, NotCloneFallback};

// No keypair takes or returns a plain-array seed (LLR-56hc77), so each
// keypair here is built from a typed seed, and a rebuild goes through the
// seed type the keypair hands back.

fn hex32(text: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * index..2 * index + 2], 16).unwrap();
    }
    out
}

// The device seed leaves a `DeviceSecret` only through `lend_to_keychain`
// (LLR-37k2s3), so the rebuild goes through those bytes.
// verifies: LLR-e58j8m
#[test]
fn a_device_keypair_rebuilt_from_its_seed_has_the_same_device_key() {
    let original = secret_of(3);
    let mut seed = Zeroizing::new(vec![]);
    original
        .lend_to_keychain(|bytes| {
            seed.extend_from_slice(bytes);
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();
    let rebuilt = DeviceSecret::from_keychain_bytes(seed).unwrap();
    assert_eq!(original.device_key(), rebuilt.device_key());
}

// verifies: LLR-ctzkv7
#[test]
fn device_key_wraps_the_verifying_key() {
    let kp = SigningKeypair::generate(&mut OsRng);
    assert_eq!(kp.device_key().unwrap().as_bytes(), kp.verifying_key().as_bytes());
}

// Abnormal: the DevicePublicKey is the verifying key of the device seed, so
// two different seeds give two different keys; a key that ignored the seed
// would give one.
// verifies: LLR-ctzkv7
#[test]
fn two_device_seeds_give_two_device_keys() {
    assert_ne!(secret_of(5).device_key().as_bytes(), secret_of(6).device_key().as_bytes());
}

// RFC 7748 §6.1: Alice's and Bob's private keys and X25519(k, 9).
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

// verifies: LLR-ctzkv7
#[test]
fn member_key_is_the_x25519_public_key_and_always_valid() {
    for _ in 0..256 {
        let kp = X25519Keypair::generate(&mut OsRng);
        let key = kp.member_key().expect("an X25519 public key is a valid PersonPublicKey");
        assert_eq!(key.as_bytes(), &kp.public_bytes());
    }
}

// Abnormal: degenerate seeds — every byte zero, every byte one — still give a
// valid Member-as-a-group key (the clamp keeps the scalar non-zero, so the
// key is never the identity) and a DevicePublicKey wrapping the verifying key.
// verifies: LLR-ctzkv7
#[test]
fn degenerate_seeds_still_give_valid_keys() {
    for byte in [0x00u8, 0xff] {
        let member = MemberSeed::from([byte; 32]).x25519_keypair();
        let key = member.member_key().expect("a degenerate seed still gives a valid PersonPublicKey");
        assert_eq!(key.as_bytes(), &member.public_bytes());
        let verifying_key = ed25519_dalek::SigningKey::from_bytes(&[byte; 32]).verifying_key();
        assert_eq!(secret_of(byte).device_key().as_bytes(), verifying_key.as_bytes());
    }
}

// LLR-98ufry's normal case: a key pair that cannot be cloned still hands its secret back as
// a seed that rebuilds the same key pair.
// verifies: LLR-e58j8m, LLR-98ufry
#[test]
fn x25519_seed_round_trip_preserves_key() {
    let kp = X25519Keypair::generate(&mut OsRng);
    assert_eq!(kp.member_seed().x25519_keypair().public_bytes(), kp.public_bytes());
}

// Checked when the test target compiles: deriving or implementing `Clone` for
// `X25519Keypair` fails this target's build.
const _: () = assert!(!CloneProbe::<X25519Keypair>::IS_CLONE, "X25519Keypair implements Clone");

// LLR-98ufry's "cannot be cloned" clause. The probe is checked against types whose answer is known, so a probe that
// always answered `false` would fail here: `[u8; 32]` and `MemberSeed` are
// `Clone`, `X25519Keypair` is not.
// verifies: LLR-98ufry
#[test]
#[allow(
    clippy::assertions_on_constants,
    reason = "the probe's answers are compile-time constants by design; asserting them at run time is the test"
)]
fn an_x25519_key_pair_cannot_be_cloned() {
    assert!(CloneProbe::<[u8; 32]>::IS_CLONE, "the probe sees a Clone type");
    assert!(CloneProbe::<MemberSeed>::IS_CLONE, "the probe sees a Clone secret type");
    assert!(!CloneProbe::<X25519Keypair>::IS_CLONE, "an X25519 key pair is not Clone");
}

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
// secret byte. That `Drop` calls `zeroize` is not observed here: no safe test
// can read the memory of a dropped value, so that clause of LLR-98ufry is
// verified by inspection.
// verifies: LLR-98ufry
#[test]
fn an_x25519_key_pair_declares_zeroize_on_drop_and_zeroize_clears_its_secret() {
    use zeroize::Zeroize;
    declares_zeroize_on_drop::<X25519Keypair>();
    let mut kp = MemberSeed::from([0xabu8; 32]).x25519_keypair();
    kp.zeroize();
    assert_eq!(kp.member_seed().expose_secret(), &[0u8; 32], "no secret byte is left");
}

// --- DeviceSecret, the move-only device seed ---

fn secret_of(byte: u8) -> DeviceSecret {
    DeviceSecret::from_keychain_bytes(Zeroizing::new(vec![byte; 32])).unwrap()
}

// verifies: LLR-37k2s3
#[test]
fn a_device_secret_debug_renders_only_the_marker() {
    assert_eq!(format!("{:?}", secret_of(0xab)), "DeviceSecret([REDACTED])");
}

// verifies: LLR-37k2s3
#[test]
fn a_device_secret_declares_zeroize_on_drop() {
    declares_zeroize_on_drop::<DeviceSecret>();
}

/// The text of `src/keys.rs`.
fn keys_source() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/keys.rs")).unwrap()
}

/// Every block of `source` that opens with the line `opening` (at column
/// one), each up to its closing `}` at column one.
fn blocks_opening_with<'a>(source: &'a str, opening: &str) -> Vec<&'a str> {
    let mut blocks = vec![];
    let mut rest = source;
    while let Some(start) = rest.find(&format!("\n{opening}\n")) {
        let block = &rest[start + 1..];
        let end = block.find("\n}\n").unwrap();
        blocks.push(&block[..end]);
        rest = &block[end..];
    }
    blocks
}

// The seed's wipe on drop is structural: the field that holds it is
// `Zeroizing`, whose `Drop` overwrites it with zeros. A dropped value's
// memory cannot be read by a safe test, so this checks the field's type: the
// test fails if the seed is held in any other form.
// verifies: LLR-37k2s3
#[test]
fn a_device_secrets_seed_is_held_in_zeroizing() {
    let source = keys_source();
    let definitions = blocks_opening_with(&source, "pub struct DeviceSecret {");
    assert_eq!(definitions.len(), 1, "one definition of DeviceSecret");
    let fields: Vec<&str> = definitions[0].lines().skip(1).map(str::trim).filter(|line| !line.starts_with("//")).collect();
    let seed_types: Vec<&str> = fields
        .iter()
        .filter(|field| field.contains("[u8"))
        .map(|field| field.split_once(':').unwrap().1.trim().trim_end_matches(','))
        .collect();
    assert_eq!(seed_types.len(), 1, "one field holds seed bytes: {seed_types:?}");
    assert!(
        ["Zeroizing<[u8; 32]>", "zeroize::Zeroizing<[u8; 32]>"].contains(&seed_types[0]),
        "the seed is held directly in Zeroizing: {seed_types:?}"
    );
    assert!(!source.contains("impl Drop for DeviceSecret"), "no hand-written Drop stands beside Zeroizing's");
}

// verifies: LLR-37k2s3
#[test]
fn the_device_key_is_the_ed25519_verifying_key_of_the_seed() {
    let expected = ed25519_dalek::SigningKey::from_bytes(&[0x07; 32]).verifying_key();
    assert_eq!(secret_of(0x07).device_key().as_bytes(), expected.as_bytes());
}

// verifies: LLR-37k2s3
#[test]
fn a_generated_secret_differs_from_the_next_one() {
    let first = DeviceSecret::generate(&mut OsRng).device_key();
    let second = DeviceSecret::generate(&mut OsRng).device_key();
    assert_ne!(first, second);
}

// verifies: LLR-yp69jg
#[test]
fn from_keychain_bytes_refuses_any_length_but_32_and_echoes_no_byte() {
    for length in [0usize, 31, 33, 64] {
        match DeviceSecret::from_keychain_bytes(Zeroizing::new(vec![0xab; length])) {
            Err(org_node::OrgNodeError::InvalidField { field, reason }) => {
                assert_eq!(field, "Device seed");
                assert!(!reason.contains("171") && !reason.to_lowercase().contains("ab, "), "{reason}");
            }
            other => panic!("length {length}: expected InvalidField, got {other:?}"),
        }
    }
}

// verifies: LLR-yp69jg
#[test]
fn lend_to_keychain_calls_the_writer_exactly_once_with_the_seed() {
    // The seed is compared inside the writer, not copied out of the lend
    // (LLR-37k2s3); the writer's refusal comes back.
    let secret = secret_of(0x07);
    let mut calls = 0;
    let mut saw_the_seed = false;
    let lent = secret.lend_to_keychain(|bytes| {
        calls += 1;
        saw_the_seed = *bytes == [0x07; 32];
        Ok::<(), std::io::Error>(())
    });
    assert_eq!((calls, saw_the_seed, lent.is_ok()), (1, true, true));
    let refused = secret.lend_to_keychain(|_| Err(std::io::Error::other("keychain refused")));
    assert_eq!(refused.unwrap_err().to_string(), "keychain refused");
}

// Regression: the lend's return must not be a channel for the bytes. The
// writer returns `Result<(), E>` and the lend returns exactly that, with `E`
// bound to `std::error::Error` (which no byte array implements), so neither
// `|seed| *seed` nor `|seed| Err(*seed)` compiles. A generic return over the
// writer's output fails this scan. The test checks the signature only: a
// caller-defined error type wrapping the bytes is outside it and rests on
// review (LLR-yp69jg).
// verifies: LLR-yp69jg, LLR-37k2s3
#[test]
fn lend_to_keychain_returns_no_value_the_writer_could_carry_the_bytes_in() {
    let source = keys_source();
    let start = source.find("pub fn lend_to_keychain").expect("keys.rs declares lend_to_keychain");
    let end = start + source[start..].find('{').unwrap();
    // Layout-free: whitespace and rustfmt's trailing commas dropped.
    let signature = source[start..end].split_whitespace().collect::<String>().replace(",)", ")");
    assert_eq!(
        signature,
        "pubfnlend_to_keychain<E:std::error::Error>(&self,write:implFnOnce(&[u8;32])->Result<(),E>)->Result<(),E>",
    );
}

// Every `impl DeviceSecret` block is read, not only the first.
// verifies: LLR-37k2s3
#[test]
fn device_secret_offers_only_its_four_public_operations() {
    let source = keys_source();
    let blocks = blocks_opening_with(&source, "impl DeviceSecret {");
    assert!(!blocks.is_empty(), "keys.rs implements DeviceSecret");
    // `pub fn`, `pub const fn`, `pub async fn` …; `pub(crate)` is not public.
    let public: Vec<&str> = blocks
        .iter()
        .flat_map(|block| block.lines())
        .filter(|line| line.trim_start().starts_with("pub ") && line.contains("fn "))
        .collect();
    assert_eq!(public.len(), 4, "{public:#?}");
    for name in ["fn generate", "fn from_keychain_bytes", "fn device_key", "fn lend_to_keychain"] {
        assert!(public.iter().any(|line| line.contains(name)), "{name} missing");
    }
    for line in &public {
        assert!(!line.contains("-> &[u8") && !line.contains("-> [u8") && !line.contains("SigningKeypair"), "{line}");
    }
}

/// Trait probes for `Copy`, `Display` and `Serialize`, in `CloneProbe`'s
/// form: the inherent constant answers `true` when the bound holds, the
/// trait's default `false` otherwise.
struct CopyProbe<T: ?Sized>(std::marker::PhantomData<T>);
trait NotCopyFallback {
    const IS_COPY: bool = false;
}
impl<T: ?Sized> NotCopyFallback for CopyProbe<T> {}
impl<T: Copy> CopyProbe<T> {
    const IS_COPY: bool = true;
}

struct DisplayProbe<T: ?Sized>(std::marker::PhantomData<T>);
trait NotDisplayFallback {
    const IS_DISPLAY: bool = false;
}
impl<T: ?Sized> NotDisplayFallback for DisplayProbe<T> {}
impl<T: ?Sized + std::fmt::Display> DisplayProbe<T> {
    const IS_DISPLAY: bool = true;
}

struct SerializeProbe<T: ?Sized>(std::marker::PhantomData<T>);
trait NotSerializeFallback {
    const IS_SERIALIZE: bool = false;
}
impl<T: ?Sized> NotSerializeFallback for SerializeProbe<T> {}
impl<T: ?Sized + serde::Serialize> SerializeProbe<T> {
    const IS_SERIALIZE: bool = true;
}

struct DeserializeProbe<T: ?Sized>(std::marker::PhantomData<T>);
trait NotDeserializeFallback {
    const IS_DESERIALIZE: bool = false;
}
impl<T: ?Sized> NotDeserializeFallback for DeserializeProbe<T> {}
impl<T: serde::de::DeserializeOwned> DeserializeProbe<T> {
    const IS_DESERIALIZE: bool = true;
}

// Each probe is first checked against a type whose answer is known, so a
// probe that always answered `false` fails here. `Deserialize`, which
// LLR-37k2s3 names, is probed too.
// verifies: LLR-37k2s3
#[test]
fn a_device_secret_cannot_be_cloned_copied_displayed_or_serialised() {
    // (probe answer, expected answer, what is probed), checked in a loop so
    // each answer is read at run time.
    let answers = [
        (CloneProbe::<[u8; 32]>::IS_CLONE, true, "the Clone probe on [u8; 32]"),
        (CopyProbe::<[u8; 32]>::IS_COPY, true, "the Copy probe on [u8; 32]"),
        (DisplayProbe::<u8>::IS_DISPLAY, true, "the Display probe on u8"),
        (SerializeProbe::<[u8; 32]>::IS_SERIALIZE, true, "the Serialize probe on [u8; 32]"),
        (CloneProbe::<DeviceSecret>::IS_CLONE, false, "DeviceSecret: Clone"),
        (CopyProbe::<DeviceSecret>::IS_COPY, false, "DeviceSecret: Copy"),
        (DisplayProbe::<DeviceSecret>::IS_DISPLAY, false, "DeviceSecret: Display"),
        (SerializeProbe::<DeviceSecret>::IS_SERIALIZE, false, "DeviceSecret: Serialize"),
        (DeserializeProbe::<[u8; 32]>::IS_DESERIALIZE, true, "the Deserialize probe on [u8; 32]"),
        (DeserializeProbe::<DeviceSecret>::IS_DESERIALIZE, false, "DeviceSecret: Deserialize"),
    ];
    for (answer, expected, probed) in answers {
        assert_eq!(answer, expected, "{probed}");
    }
}
