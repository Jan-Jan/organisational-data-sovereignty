//! The Person hash: the V1 encoding, the keyed hash, the check of a hash.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

mod common;

use common::{hex, person, person_key, slots};
use person::definition::Person;
use person::device_root_hasher::PersonDeviceRootHasher;
use person::encoding::EncodingVersion;
use person::hash::{encode, person_hash, verify, PersonHash, PERSON_DEFINITION_V1_KEY};
use person::{compute_device_root, IdentityError, MAX_NAME_LEN, MAX_SURNAME_LEN};

fn hash_v1(definition: &Person) -> PersonHash {
    person_hash(definition, 1).expect("version 1 is implemented")
}

/// verifies: LLR-qwp8d7
#[test]
fn the_v1_encoding_is_the_documented_layout() {
    let alice = person("Alice", "Example", Some(9), &[1, 2]);
    let mut expected = Vec::new();
    expected.extend_from_slice(&5u32.to_le_bytes());
    expected.extend_from_slice(b"Alice");
    expected.extend_from_slice(&7u32.to_le_bytes());
    expected.extend_from_slice(b"Example");
    expected.push(0x01);
    expected.extend_from_slice(person_key(9).as_bytes());
    expected
        .extend_from_slice(compute_device_root::<PersonDeviceRootHasher>(slots(&[1, 2]).devices()).expect("distinct positions").as_bytes());
    assert_eq!(encode(&alice, EncodingVersion::V1).expect("encodes"), expected);
}

/// The boundary definitions: empty names, no key, no device; and names at
/// their byte bound, given decomposed and encoded in NFC.
/// verifies: LLR-qwp8d7
#[test]
fn the_v1_encoding_of_the_boundary_definitions() {
    let empty = person("", "", None, &[]);
    let mut expected = vec![0, 0, 0, 0, 0, 0, 0, 0, 0x00];
    expected.extend_from_slice(compute_device_root::<PersonDeviceRootHasher>(slots(&[]).devices()).expect("distinct positions").as_bytes());
    assert_eq!(encode(&empty, EncodingVersion::V1).expect("encodes"), expected);

    let longest = person(
        &"a".repeat(MAX_NAME_LEN),
        &"e\u{0301}".repeat(MAX_SURNAME_LEN / 2),
        Some(9),
        &[1, 2, 3, 4],
    );
    let encoding = encode(&longest, EncodingVersion::V1).expect("encodes");
    assert_eq!(
        encoding.len(),
        4 + MAX_NAME_LEN + 4 + MAX_SURNAME_LEN + 1 + 32 + 32
    );
    assert_eq!(&encoding[..4], &128u32.to_le_bytes());
    let surname_start = 4 + MAX_NAME_LEN;
    assert_eq!(
        &encoding[surname_start..surname_start + 4],
        &128u32.to_le_bytes()
    );
    assert_eq!(
        &encoding[surname_start + 4..surname_start + 4 + MAX_SURNAME_LEN],
        "\u{00e9}".repeat(MAX_SURNAME_LEN / 2).as_bytes()
    );
}

/// The device keys enter the encoding through the Person device root only.
/// verifies: LLR-qwp8d7
#[test]
fn equal_device_sets_encode_equally_whatever_their_order() {
    assert_eq!(
        encode(
            &person("Alice", "Example", Some(9), &[3, 1, 2]),
            EncodingVersion::V1
        ).expect("encodes"),
        encode(
            &person("Alice", "Example", Some(9), &[1, 2, 3]),
            EncodingVersion::V1
        ).expect("encodes")
    );
}

/// verifies: LLR-4ebtn4
#[test]
fn the_v1_hash_is_blake3_keyed_by_the_v1_domain_key() {
    assert_eq!(
        PERSON_DEFINITION_V1_KEY,
        b"person::definition::v1__________"
    );
    let alice = person("Alice", "Example", Some(9), &[1, 2]);
    let encoding = encode(&alice, EncodingVersion::V1).expect("encodes");
    assert_eq!(
        hash_v1(&alice),
        PersonHash::new(blake3::keyed_hash(PERSON_DEFINITION_V1_KEY, &encoding).into())
    );
    assert_ne!(
        hash_v1(&alice).as_bytes(),
        blake3::hash(&encoding).as_bytes()
    );
}

/// Pinned values: the record of the V1 hash, which the structural tests
/// recompute from `blake3::keyed_hash` alone; never re-pinned to make a red
/// test pass. V1 is defined over the
/// root of the depth-4 device root tree (LLR-qwp8d7); a change here is a change of the V1
/// encoding, which needs a new version (REQ-wg7z4s).
/// verifies: LLR-4ebtn4, LLR-qwp8d7
#[test]
fn the_v1_hash_is_pinned() {
    assert_eq!(
        hex(hash_v1(&person("Alice", "Example", Some(9), &[1, 2])).as_bytes()),
        "0d8019c330b4279c02cf747b911a9cb6f1357dbb164bbd9183265e90ca951c8a"
    );
    assert_eq!(
        hex(hash_v1(&person("Alice", "Example", None, &[])).as_bytes()),
        "943411327947df5cd3f96b581d5c466cb0c8202ff00d72516fda45e4bbdd8bc3"
    );
}

/// A change to any one of the four fields changes the hash, and the length
/// prefixes keep a byte moved between name and surname from colliding.
/// verifies: LLR-4ebtn4, LLR-qwp8d7
#[test]
fn distinct_definitions_give_distinct_hashes() {
    let definitions = [
        person("Alice", "Example", Some(9), &[1, 2]),
        person("Alicia", "Example", Some(9), &[1, 2]),
        person("Alice", "Exemplar", Some(9), &[1, 2]),
        person("Alice", "Example", Some(10), &[1, 2]),
        person("Alice", "Example", Some(9), &[1, 3]),
        person("Alice", "Example", Some(9), &[1]),
        person("Alice", "Example", None, &[]),
        person("ab", "c", None, &[]),
        person("a", "bc", None, &[]),
    ];
    for (index, definition) in definitions.iter().enumerate() {
        for other in &definitions[index + 1..] {
            assert_ne!(
                hash_v1(definition),
                hash_v1(other),
                "{definition:?} {other:?}"
            );
        }
    }
    assert_eq!(
        hash_v1(&person("Alice", "Example", Some(9), &[2, 1])),
        hash_v1(&person("Alice", "Example", Some(9), &[1, 2]))
    );
}

/// verifies: LLR-4ebtn4, LLR-rde6tk, LLR-3n3kxx
#[test]
fn hashing_under_an_unimplemented_version_is_refused() {
    let alice = person("Alice", "Example", Some(9), &[1, 2]);
    for version in [0u16, 2, u16::MAX] {
        assert_eq!(
            person_hash(&alice, version),
            Err(IdentityError::UnsupportedEncodingVersion(version))
        );
    }
}

/// verifies: LLR-4ebtn4
#[test]
fn a_person_hash_wraps_32_bytes() {
    let mut bytes = [0xff; 32];
    bytes[0] = 0x01;
    bytes[1] = 0xab;
    let hash = PersonHash::from(bytes);
    assert_eq!(hash.as_bytes(), &bytes);
    assert_eq!(hash, PersonHash::new(bytes));
    assert_eq!(format!("{hash:?}"), "PersonHash(01abffff..)");
}

/// verifies: LLR-4ebtn4
#[cfg(feature = "serde")]
#[test]
fn a_person_hash_encodes_as_its_32_bytes() {
    let hash = PersonHash::new([7; 32]);
    let encoded = postcard::to_allocvec(&hash).expect("encode");
    assert_eq!(encoded, [7u8; 32].to_vec());
    assert_eq!(
        postcard::from_bytes::<PersonHash>(&encoded).expect("decode"),
        hash
    );
}

/// verifies: LLR-tf45kx
#[test]
fn verify_matches_exactly_the_hash_of_the_definition() {
    let alice = person("Alice", "Example", Some(9), &[1, 2]);
    let own = hash_v1(&alice);
    assert_eq!(verify(&alice, 1, &own), Ok(true));
    let other = hash_v1(&person("Alice", "Example", Some(10), &[1, 2]));
    assert_eq!(verify(&alice, 1, &other), Ok(false));
    for position in 0..32 {
        let mut flipped = *own.as_bytes();
        flipped[position] ^= 0x01;
        assert_eq!(verify(&alice, 1, &PersonHash::new(flipped)), Ok(false));
    }
    assert_eq!(verify(&alice, 1, &PersonHash::new([0; 32])), Ok(false));
}

/// An unimplemented version is an error even when the supplied hash is the
/// definition's V1 hash: it is never a non-match, nor a match.
/// verifies: LLR-tf45kx, LLR-3n3kxx
#[test]
fn verify_under_an_unimplemented_version_is_an_error() {
    let alice = person("Alice", "Example", Some(9), &[1, 2]);
    let own = hash_v1(&alice);
    for version in [0u16, 2, u16::MAX] {
        assert_eq!(
            verify(&alice, version, &own),
            Err(IdentityError::UnsupportedEncodingVersion(version))
        );
    }
}

/// Each answer depends only on the call's own arguments: interleaving checks
/// of two definitions changes none of them.
/// verifies: LLR-tf45kx
#[test]
fn verify_keeps_nothing_between_calls() {
    let alice = person("Alice", "Example", Some(9), &[1, 2]);
    let bob = person("Bob", "Example", Some(10), &[3]);
    let alice_hash = hash_v1(&alice);
    let bob_hash = hash_v1(&bob);
    for _ in 0..3 {
        assert_eq!(verify(&alice, 1, &alice_hash), Ok(true));
        assert_eq!(verify(&bob, 1, &alice_hash), Ok(false));
        assert_eq!(verify(&bob, 1, &bob_hash), Ok(true));
        assert_eq!(verify(&alice, 1, &bob_hash), Ok(false));
    }
}

/// The hash and its check hold no static, global, cache or interior-mutable
/// state: no code line of their sources declares a `static` item or names a
/// construct that would hold one. (`&'static` lifetimes are not items.)
/// verifies: LLR-tf45kx
#[test]
fn the_hash_sources_hold_no_state() {
    let stateful = [
        "Cell<",
        "Mutex",
        "RwLock",
        "Atomic",
        "OnceLock",
        "OnceCell",
        "LazyLock",
        "thread_local",
    ];
    for (file, source) in [
        ("hash.rs", include_str!("../src/hash.rs")),
        ("encoding.rs", include_str!("../src/encoding.rs")),
        ("device_root_hasher.rs", include_str!("../src/device_root_hasher.rs")),
        ("keyed_hash.rs", include_str!("../src/keyed_hash.rs")),
        ("position.rs", include_str!("../src/position.rs")),
    ] {
        let code_lines = source
            .lines()
            .map(str::trim_start)
            .filter(|line| !line.starts_with("//"));
        for line in code_lines {
            let declares_static = line.starts_with("static ") || line.starts_with("pub static ");
            assert!(!declares_static, "{file}: a static item in {line:?}");
            for construct in stateful {
                assert!(
                    !line.contains(construct),
                    "{file}: `{construct}` in {line:?}"
                );
            }
        }
    }
}
