# Shared identity types in `person` — Implementation Plan

**Goal:** Build `Name`, `Surname`, `DevicePublicKey`, `PersonPublicKey`, `DeviceSlots` and the device sub-trie in the `person` crate, switch org-members (and org-node, app) to them with every member hash byte-identical, then port `PersonPublicKey` from ed25519 to X25519 test-first.
**Implements:** REQ-r7mytp, REQ-q6xkna, REQ-3vqs9b, REQ-4szc22, REQ-tq4ms4, REQ-mu3qgz, REQ-aj6x3n, REQ-vxx8k3, REQ-bczz87, REQ-7gz72r; SDD-k5ee9x, SDD-7r833z, SDD-u9cddc; LLR-ayu93n, LLR-7guspr, LLR-vs7etb, LLR-9p34cv, LLR-sjrh7z, LLR-4vsm8d, LLR-6ezhw7, LLR-eeq89n, LLR-64muuw, LLR-m75m7u (and the derived LLRs the merge review added)

**Two branches (owner, 2026-10-04).** This plan was executed on
`worktree-worktree-person-shared-types`. The `person` part (T1–T7, T9 steps
1–4, T9c, and the review fixes) is merged first, alone, from
`worktree-person-unit`, where the ledger files are
`person/docs/{requirements,risk,architecture}/2026-10-04-identity-types.md`
rather than the `DRAFT-worktree-worktree-person-shared-types-shared-types.md`
names below. Commit hashes in the task notes are on the first branch; T8,
T9's later steps and T10 are not in the `person` merge.
**Safety class:** C (person, org-members, org-node, app; no per-item overrides)
**Verification:** person: `cargo test -p person`, `cargo clippy -p person --all-targets -- -D warnings`. org-members, org-node, app: their unit configs' `verify_commands`, unchanged (QUINT_HOME set to a scratch directory for org-members' conformance test).

Context: `docs/plans/2026-10-04-person-shared-types.md` (scope, dependency
assessment, order of work, step-2 obligations). Requirements, risk and design
are the `DRAFT-worktree-worktree-person-shared-types-shared-types.md` files in
`person/docs/{requirements,risk,architecture}/`.

Order (owner): T1–T7 build `person` without touching org-members; T8 switches
org-members with `PersonPublicKey` still validated as ed25519, because the
fixture migration is not trivial (tests build a member key and a device key
from the same bytes, and `tests/encoding_golden.rs` pins bytes); T9 ports to
X25519 test-first; T10 closes the documents.

**Baseline (2026-10-04, before T1), all green:** org-members 0+3+9+161+32
(1 ignored)+16+0 passed, with QUINT_HOME a scratch directory seeded by
`cp -R ~/.quint/rust-evaluator-v0.7.0 $QUINT_HOME/` (an empty one fails:
quint tries to download the evaluator); org-node 57 passed and app 78 passed
over their configured cargo targets, with `CARGO_HOME=/tmp/cargo_home_fuzz`
(the default cargo home is read-only and lacks crates they need).

All commands run from the worktree root. Every task ends with
`git add <its files>` and `git commit` (one plain git command per call).

---

### T1 — Crate skeleton, error type, crate-level discipline

**Files touched:** `person/Cargo.toml`, `person/src/lib.rs`, `person/src/error.rs`, `person/tests/placeholder.rs` (deleted), `person/tests/crate_discipline.rs`, `person/.guardrails/config.yaml`
**Parallel:** no (first)

1. Failing test `person/tests/crate_discipline.rs`:

```rust
//! Crate-level discipline of the person unit.

use person::IdentityError;

/// verifies: LLR-64muuw, REQ-vxx8k3
#[test]
fn crate_is_no_std_and_inherits_the_panic_denying_lints() {
    let lib = include_str!("../src/lib.rs");
    assert!(lib.contains("#![no_std]"), "person must be no_std");
    let manifest = include_str!("../Cargo.toml");
    assert!(
        manifest.contains("[lints]") && manifest.contains("workspace = true"),
        "person must inherit the workspace lints (deny unwrap/expect/panic)"
    );
}

/// verifies: LLR-eeq89n, REQ-vxx8k3
#[test]
fn every_refusal_is_a_distinct_named_variant() {
    let all = [
        IdentityError::FieldTooLong { field: "name", max: 128 },
        IdentityError::InvalidDeviceKey,
        IdentityError::InvalidPersonKey,
        IdentityError::DeviceSlotsFull,
        IdentityError::DuplicateDevice,
        IdentityError::DeviceNotFound,
    ];
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a, b);
            assert_ne!(a.to_string(), b.to_string());
        }
    }
}
```

2. `cargo test -p person --test crate_discipline` → fails to compile
   (`unresolved import person::IdentityError`).

3. `person/Cargo.toml`:

```toml
[package]
name = "person"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
license = "GPL-3.0-only"
description = "An individual's identity types and definition, and the hash that commits to it"

[features]
default = ["serde"]
serde = ["dep:serde"]

[dependencies]
ed25519-dalek = { version = "2", default-features = false, features = ["alloc"] }
unicode-normalization = { version = "0.1", default-features = false }
serde = { version = "1", default-features = false, features = ["derive", "alloc"], optional = true }
thiserror = { version = "2", default-features = false }

[dev-dependencies]
blake3 = "1"
postcard = { version = "1", features = ["alloc"] }
proptest = "1"

[lints]
workspace = true
```

`person/src/lib.rs`:

```rust
//! An individual's identity types: names, device keys, the group key, device
//! slots and the device sub-trie. org-members uses them; the Person
//! definition is built on them.

#![no_std]

extern crate alloc;

pub mod error;

pub use error::IdentityError;
```

`person/src/error.rs`:

```rust
/// Every refusal by a constructor in this crate. LLR-eeq89n.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdentityError {
    #[error("field too long: {field} exceeds {max} bytes after NFC normalization")]
    FieldTooLong { field: &'static str, max: usize },

    #[error("invalid device public key")]
    InvalidDeviceKey,

    #[error("invalid person public key")]
    InvalidPersonKey,

    #[error("device slots full (max 4)")]
    DeviceSlotsFull,

    #[error("duplicate device")]
    DuplicateDevice,

    #[error("device not found")]
    DeviceNotFound,
}
```

Delete `person/tests/placeholder.rs`. In `person/.guardrails/config.yaml`,
replace the comment above `test_paths:` (the placeholder note) with
`# Annotated integration tests, one file per software item.` and replace
`verify_commands:` with:

```yaml
verify_commands:
  - cargo test -p person
  - cargo clippy -p person --all-targets -- -D warnings
```

4. `cargo test -p person --test crate_discipline` → `test result: ok. 2 passed`.
5. Commit: `feat(person): crate skeleton and IdentityError`.

**Done (7c94739).** red -> green:
- every_refusal_is_a_distinct_named_variant — watched fail for the right reason
  (E0432 unresolved import `person::IdentityError`) before the implementation existed.
- crate_is_no_std_and_inherits_the_panic_denying_lints — red only through the
  same compile failure; its own assertions already held on the scaffold
  (`#![no_std]`, `[lints] workspace = true`), so it is a regression guard,
  never red on its assertions. Result: 2 passed; clippy clean.

### T2 — Name and Surname

**Files touched:** `person/src/name.rs`, `person/src/lib.rs`, `person/tests/names.rs`
**Parallel:** no (serial, after T1)

1. Failing test `person/tests/names.rs`:

```rust
use person::{IdentityError, Name, Surname, MAX_NAME_LEN, MAX_SURNAME_LEN};

/// verifies: LLR-ayu93n, REQ-r7mytp
#[test]
fn names_are_stored_in_nfc() {
    let decomposed = "Jose\u{0301}"; // e + combining acute
    let name = Name::parse(decomposed).expect("valid name");
    assert_eq!(name.as_str(), "Jos\u{00e9}");
    let surname = Surname::parse(decomposed).expect("valid surname");
    assert_eq!(surname.as_str(), "Jos\u{00e9}");
}

/// verifies: LLR-ayu93n, REQ-r7mytp
#[test]
fn the_bound_applies_after_normalisation_and_names_the_field() {
    let at_bound = "a".repeat(MAX_NAME_LEN);
    assert!(Name::parse(&at_bound).is_ok());
    let over = "a".repeat(MAX_NAME_LEN + 1);
    assert_eq!(
        Name::parse(&over),
        Err(IdentityError::FieldTooLong { field: "name", max: MAX_NAME_LEN })
    );
    let over = "a".repeat(MAX_SURNAME_LEN + 1);
    assert_eq!(
        Surname::parse(&over),
        Err(IdentityError::FieldTooLong { field: "surname", max: MAX_SURNAME_LEN })
    );
}

/// verifies: LLR-ayu93n, REQ-r7mytp
#[test]
fn try_from_and_decoding_go_through_parse() {
    let over = "a".repeat(MAX_NAME_LEN + 1);
    assert!(Name::try_from(over.as_str()).is_err());
    assert!(Name::try_from(over.clone()).is_err());
    let bytes = postcard::to_allocvec(&over).expect("encode string");
    assert!(postcard::from_bytes::<Name>(&bytes).is_err());
    let ok = postcard::to_allocvec(&"Jose\u{0301}").expect("encode string");
    assert_eq!(postcard::from_bytes::<Name>(&ok).expect("decode").as_str(), "Jos\u{00e9}");
}

/// verifies: LLR-ayu93n
#[test]
fn debug_redacts_personal_data() {
    let name = Name::parse("Alice").expect("valid name");
    assert_eq!(format!("{name:?}"), "Name([REDACTED])");
}
```

2. `cargo test -p person --test names` → fails to compile (`no Name in the root`).
3. `person/src/name.rs` — moved from `org-members/src/types.rs` (`nfc_bounded`,
   `Name`, `Surname`, `validated_string_impls!`) with the error type replaced:

```rust
use alloc::string::String;
use core::fmt;

use unicode_normalization::UnicodeNormalization;

use crate::error::IdentityError;

/// Maximum byte length of a name after NFC normalization.
pub const MAX_NAME_LEN: usize = 128;

/// Maximum byte length of a surname after NFC normalization.
pub const MAX_SURNAME_LEN: usize = 128;

/// NFC-normalizes `value` and bounds it at `max` bytes. LLR-ayu93n.
fn nfc_bounded(value: &str, field: &'static str, max: usize) -> Result<String, IdentityError> {
    let nfc: String = value.nfc().collect();
    if nfc.len() > max {
        return Err(IdentityError::FieldTooLong { field, max });
    }
    Ok(nfc)
}

macro_rules! validated_string_impls {
    ($ty:ident) => {
        impl $ty {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<&str> for $ty {
            type Error = IdentityError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl TryFrom<String> for $ty {
            type Error = IdentityError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::parse(&value)
            }
        }

        impl From<$ty> for String {
            fn from(value: $ty) -> Self {
                value.0
            }
        }

        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl fmt::Debug for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($ty), "([REDACTED])"))
            }
        }
    };
}

/// A given name: NFC, at most `MAX_NAME_LEN` bytes. PII.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct Name(String);

impl Name {
    pub fn parse(value: &str) -> Result<Self, IdentityError> {
        nfc_bounded(value, "name", MAX_NAME_LEN).map(Self)
    }
}

validated_string_impls!(Name);

/// A surname: NFC, at most `MAX_SURNAME_LEN` bytes. PII.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct Surname(String);

impl Surname {
    pub fn parse(value: &str) -> Result<Self, IdentityError> {
        nfc_bounded(value, "surname", MAX_SURNAME_LEN).map(Self)
    }
}

validated_string_impls!(Surname);
```

   In `person/src/lib.rs` add `pub mod name;` and
   `pub use name::{Name, Surname, MAX_NAME_LEN, MAX_SURNAME_LEN};`.
4. `cargo test -p person --test names` → `test result: ok. 4 passed`.
5. Commit: `feat(person): Name and Surname`.

**Done (2f4f196).** red -> green: names_are_stored_in_nfc,
the_bound_applies_after_normalisation_and_names_the_field,
try_from_and_decoding_go_through_parse, debug_redacts_personal_data — each
watched fail for the right reason (E0432 unresolved imports `person::Name`,
`Surname`, `MAX_NAME_LEN`, `MAX_SURNAME_LEN`) before the implementation
existed. Result: 6 passed; clippy clean.

**Applies to every test file from here on (found in T2):** the workspace
lints deny `expect_used`/`unwrap_used` in test targets too, so each test file
opens with `#![allow(clippy::expect_used)]` (add `clippy::unwrap_used` if it
unwraps) and the comment "A failed expect is the test failing; the
panic-denying lints guard the library, not its tests." — the convention
org-node's test files already follow.

### T3 — DevicePublicKey

**Files touched:** `person/src/device_key.rs`, `person/src/lib.rs`, `person/tests/device_key.rs`
**Parallel:** no (serial, after T2)

1. Failing test `person/tests/device_key.rs`:

```rust
use ed25519_dalek::{SigningKey, VerifyingKey};
use person::{DevicePublicKey, IdentityError};

fn vk(seed: u8) -> VerifyingKey {
    SigningKey::from_bytes(&[seed; 32]).verifying_key()
}

/// verifies: LLR-7guspr, REQ-q6xkna
#[test]
fn from_bytes_accepts_exactly_what_ed25519_dalek_accepts() {
    let good = *vk(7).as_bytes();
    let key = DevicePublicKey::from_bytes(&good).expect("valid ed25519 point");
    assert_eq!(key.as_bytes(), &good);
    assert_eq!(key, DevicePublicKey::new(vk(7)));

    // y = 2 is not on the curve: decompression fails.
    let mut off_curve = [0u8; 32];
    off_curve[0] = 2;
    assert!(VerifyingKey::from_bytes(&off_curve).is_err());
    assert_eq!(DevicePublicKey::from_bytes(&off_curve), Err(IdentityError::InvalidDeviceKey));
}

/// Records, rather than assumes, whether ed25519-dalek's from_bytes refuses a
/// small-order point (risk file of this change). The identity point encodes
/// as y = 1. If this assertion fails, from_bytes refuses it: record that in
/// person's and org-members' SOUP rows instead.
/// verifies: LLR-7guspr
#[test]
fn small_order_points_are_not_refused_by_from_bytes() {
    let mut identity = [0u8; 32];
    identity[0] = 1;
    let key = VerifyingKey::from_bytes(&identity).expect("dalek accepts the identity point");
    assert!(key.is_weak());
    assert!(DevicePublicKey::from_bytes(&identity).is_ok());
}

/// verifies: LLR-7guspr, REQ-q6xkna
#[test]
fn decoding_goes_through_from_bytes() {
    let mut off_curve = [0u8; 32];
    off_curve[0] = 2;
    let bytes = postcard::to_allocvec(&off_curve).expect("encode");
    assert!(postcard::from_bytes::<DevicePublicKey>(&bytes).is_err());
    let good = DevicePublicKey::new(vk(9));
    let bytes = postcard::to_allocvec(&good).expect("encode");
    assert_eq!(postcard::from_bytes::<DevicePublicKey>(&bytes).expect("decode"), good);
}

/// verifies: LLR-7guspr
#[test]
fn keys_order_by_their_bytes() {
    let (a, b) = (DevicePublicKey::new(vk(1)), DevicePublicKey::new(vk(2)));
    assert_eq!(a.cmp(&b), a.as_bytes().cmp(b.as_bytes()));
}
```

2. `cargo test -p person --test device_key` → fails to compile.
3. `person/src/device_key.rs` — moved from org-members' `P2pDeviceKey`:

```rust
use core::fmt;

use ed25519_dalek::VerifyingKey;

use crate::error::IdentityError;

/// A device's ed25519 public key: the device's identity. LLR-7guspr.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DevicePublicKey(VerifyingKey);

impl DevicePublicKey {
    /// From a key ed25519-dalek has already validated.
    pub fn new(key: VerifyingKey) -> Self {
        Self(key)
    }

    /// Accepts the bytes exactly when `VerifyingKey::from_bytes` does.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, IdentityError> {
        VerifyingKey::from_bytes(bytes)
            .map(Self)
            .map_err(|_| IdentityError::InvalidDeviceKey)
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.0
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

impl PartialOrd for DevicePublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DevicePublicKey {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl fmt::Debug for DevicePublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [b0, b1, b2, b3, ..] = *self.as_bytes();
        write!(f, "DevicePublicKey({b0:02x}{b1:02x}{b2:02x}{b3:02x}..)")
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DevicePublicKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.as_bytes().serialize(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DevicePublicKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let bytes = <[u8; 32]>::deserialize(d)?;
        Self::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}
```

   Add to `person/Cargo.toml` `[dev-dependencies]`:
   `ed25519-dalek = { version = "2", features = ["alloc"] }`.
   In `lib.rs`: `pub mod device_key;` and `pub use device_key::DevicePublicKey;`.
4. `cargo test -p person --test device_key` → `test result: ok. 4 passed`. If
   `small_order_points_are_not_refused_by_from_bytes` fails at the
   `is_ok()` line, stop and report: the SOUP claim is then true and the test is
   rewritten to assert the refusal.
5. Commit: `feat(person): DevicePublicKey`.

**Done (d2699e9).** red -> green: from_bytes_accepts_exactly_what_ed25519_dalek_accepts,
small_order_points_are_not_refused_by_from_bytes, decoding_goes_through_from_bytes,
keys_order_by_their_bytes — each watched fail for the right reason (E0432
unresolved import `person::DevicePublicKey`) before the implementation
existed. Result: 10 passed; clippy clean.
**Finding (for T10):** the characterisation test holds —
`VerifyingKey::from_bytes` accepts the identity point (`is_weak()` true), so
org-members' SOUP claim that it rejects small-order encodings is false.
The `ed25519-dalek` dev-dependency was not added: the normal dependency
already serves the tests.
**For the deslop pass:** `cargo fmt -p person --check` flags
`tests/crate_discipline.rs` (T1); test targets using postcard do not compile
with `--no-default-features` (not a configured command) — gate them on
`#![cfg(feature = "serde")]`.

### T4 — PersonPublicKey (ed25519 validation, interim)

**Files touched:** `person/src/person_key.rs`, `person/src/lib.rs`, `person/tests/person_key.rs`
**Parallel:** no (serial, after T3)

Interim per the order of work: the type holds 32 bytes and validates them as
ed25519, so org-members' fixtures keep working in T8. The tests here verify
no LLR yet: LLR-vs7etb is claimed only in T9, when the X25519 rule holds.

1. Failing test `person/tests/person_key.rs`:

```rust
use ed25519_dalek::SigningKey;
use person::{IdentityError, PersonPublicKey};

/// Interim behaviour, replaced in T9.
#[test]
fn holds_the_bytes_it_was_given() {
    let bytes = *SigningKey::from_bytes(&[3; 32]).verifying_key().as_bytes();
    let key = PersonPublicKey::from_bytes(&bytes).expect("valid key");
    assert_eq!(key.as_bytes(), &bytes);
    let bytes = postcard::to_allocvec(&key).expect("encode");
    assert_eq!(postcard::from_bytes::<PersonPublicKey>(&bytes).expect("decode"), key);
}

/// Interim behaviour, replaced in T9.
#[test]
fn refuses_bytes_that_are_not_a_key() {
    let mut off_curve = [0u8; 32];
    off_curve[0] = 2;
    assert_eq!(PersonPublicKey::from_bytes(&off_curve), Err(IdentityError::InvalidPersonKey));
    let bytes = postcard::to_allocvec(&off_curve).expect("encode");
    assert!(postcard::from_bytes::<PersonPublicKey>(&bytes).is_err());
}
```

2. `cargo test -p person --test person_key` → fails to compile.
3. `person/src/person_key.rs`:

```rust
use core::fmt;

use crate::error::IdentityError;

/// The key a grant to a Person or a Member is encoded against. Holds the 32
/// bytes it was given. Validation is interim (ed25519) until the X25519 step;
/// LLR-vs7etb.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PersonPublicKey([u8; 32]);

impl PersonPublicKey {
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, IdentityError> {
        ed25519_dalek::VerifyingKey::from_bytes(bytes)
            .map(|_| Self(*bytes))
            .map_err(|_| IdentityError::InvalidPersonKey)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for PersonPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [b0, b1, b2, b3, ..] = self.0;
        write!(f, "PersonPublicKey({b0:02x}{b1:02x}{b2:02x}{b3:02x}..)")
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for PersonPublicKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for PersonPublicKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let bytes = <[u8; 32]>::deserialize(d)?;
        Self::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}
```

   `#[derive(PartialOrd, Ord)]` on `[u8; 32]` orders by bytes, as
   `P2pMemberKey`'s hand-written impls did. In `lib.rs`: `pub mod person_key;`
   and `pub use person_key::PersonPublicKey;`.
4. `cargo test -p person --test person_key` → `test result: ok. 2 passed`.
5. Commit: `feat(person): PersonPublicKey, ed25519 validation until the X25519 step`.

**Done (29ed589).** red -> green: holds_the_bytes_it_was_given,
refuses_bytes_that_are_not_a_key — each watched fail for the right reason
(E0432 unresolved import `person::PersonPublicKey`) before the implementation
existed; both interim, verifying no LLR. Result: 12 passed; clippy clean.

### T5 — DeviceSlots

**Files touched:** `person/src/slots.rs`, `person/src/lib.rs`, `person/tests/slots.rs`
**Parallel:** no (serial, after T4)

1. Failing test `person/tests/slots.rs`:

```rust
use ed25519_dalek::SigningKey;
use person::{DevicePublicKey, DeviceSlots, IdentityError, MAX_DEVICES};

fn dev(seed: u8) -> DevicePublicKey {
    DevicePublicKey::new(SigningKey::from_bytes(&[seed; 32]).verifying_key())
}

/// verifies: LLR-9p34cv, REQ-4szc22
#[test]
fn new_sorts_and_accepts_zero_to_the_bound() {
    assert_eq!(MAX_DEVICES, 4);
    assert_eq!(DeviceSlots::new(vec![]).expect("empty").device_count(), 0);
    let given = vec![dev(4), dev(1), dev(3), dev(2)];
    let slots = DeviceSlots::new(given.clone()).expect("four devices");
    let mut sorted = given;
    sorted.sort();
    assert_eq!(slots.devices(), sorted.as_slice());
}

/// verifies: LLR-9p34cv, REQ-4szc22
#[test]
fn new_refuses_over_the_bound_and_duplicates() {
    let five = vec![dev(1), dev(2), dev(3), dev(4), dev(5)];
    assert_eq!(DeviceSlots::new(five), Err(IdentityError::DeviceSlotsFull));
    assert_eq!(DeviceSlots::new(vec![dev(1), dev(1)]), Err(IdentityError::DuplicateDevice));
}

/// verifies: LLR-9p34cv, REQ-4szc22
#[test]
fn add_and_remove_return_new_sets_and_refuse_by_rule() {
    let one = DeviceSlots::new(vec![dev(2)]).expect("one");
    let two = one.add_device(dev(1)).expect("add");
    assert_eq!(one.device_count(), 1);
    let mut expect = vec![dev(1), dev(2)];
    expect.sort();
    assert_eq!(two.devices(), expect.as_slice());
    assert_eq!(two.add_device(dev(1)), Err(IdentityError::DuplicateDevice));
    let full = DeviceSlots::new(vec![dev(1), dev(2), dev(3), dev(4)]).expect("full");
    assert_eq!(full.add_device(dev(5)), Err(IdentityError::DeviceSlotsFull));
    assert_eq!(one.remove_device(&dev(9)), Err(IdentityError::DeviceNotFound));
    let none = one.remove_device(&dev(2)).expect("remove");
    assert_eq!(none.device_count(), 0);
    assert_eq!(one.device_count(), 1);
}
```

2. `cargo test -p person --test slots` → fails to compile.
3. `person/src/slots.rs` — moved from org-members' `P2pDeviceSlots`, with
   `add_device`/`remove_device` made `pub` (org-members is now another crate;
   both return a new set and leave the receiver unchanged, so exposing them
   bypasses nothing org-members' trie enforces) and `to_fixed_slots` kept
   `pub(crate)`:

```rust
use alloc::vec::Vec;
use core::fmt;

use crate::device_key::DevicePublicKey;
use crate::error::IdentityError;

/// Maximum number of devices: the device sub-trie has four slots.
pub const MAX_DEVICES: usize = 4;

/// A bounded, sorted, duplicate-free set of device keys. LLR-9p34cv.
#[derive(Clone, PartialEq, Eq)]
pub struct DeviceSlots {
    slots: Vec<DevicePublicKey>,
}

impl DeviceSlots {
    pub fn new(mut devices: Vec<DevicePublicKey>) -> Result<Self, IdentityError> {
        if devices.len() > MAX_DEVICES {
            return Err(IdentityError::DeviceSlotsFull);
        }
        devices.sort();
        if devices.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(IdentityError::DuplicateDevice);
        }
        Ok(Self { slots: devices })
    }

    pub fn devices(&self) -> &[DevicePublicKey] {
        &self.slots
    }

    pub fn has_device(&self, device: &DevicePublicKey) -> bool {
        self.slots.binary_search(device).is_ok()
    }

    pub fn device_count(&self) -> usize {
        self.slots.len()
    }

    pub fn add_device(&self, device: DevicePublicKey) -> Result<Self, IdentityError> {
        if self.slots.len() >= MAX_DEVICES {
            return Err(IdentityError::DeviceSlotsFull);
        }
        if self.has_device(&device) {
            return Err(IdentityError::DuplicateDevice);
        }
        let mut slots = self.slots.clone();
        slots.push(device);
        slots.sort();
        Ok(Self { slots })
    }

    pub fn remove_device(&self, device: &DevicePublicKey) -> Result<Self, IdentityError> {
        let idx = self
            .slots
            .binary_search(device)
            .map_err(|_| IdentityError::DeviceNotFound)?;
        let mut slots = self.slots.clone();
        slots.remove(idx);
        Ok(Self { slots })
    }

    /// The slots padded with `None` to `MAX_DEVICES`, in sorted order.
    pub(crate) fn to_fixed_slots(&self) -> [Option<DevicePublicKey>; MAX_DEVICES] {
        let mut fixed = [None; MAX_DEVICES];
        for (slot, device) in fixed.iter_mut().zip(self.slots.iter()) {
            *slot = Some(*device);
        }
        fixed
    }
}

impl fmt::Debug for DeviceSlots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeviceSlots({})", self.slots.len())
    }
}
```

   `windows(2)` and `pair[0]`/`pair[1]` index a window of length exactly 2,
   never an input-derived index (LLR-64muuw). In `lib.rs`: `pub mod slots;`
   and `pub use slots::{DeviceSlots, MAX_DEVICES};`.
4. `cargo test -p person --test slots` → `test result: ok. 3 passed`.
5. Commit: `feat(person): DeviceSlots`.

**Done (bcba05f).** red -> green: new_sorts_and_accepts_zero_to_the_bound,
new_refuses_over_the_bound_and_duplicates,
add_and_remove_return_new_sets_and_refuse_by_rule — each watched fail for the
right reason (E0432 unresolved imports `person::DeviceSlots`,
`person::MAX_DEVICES`) before the implementation existed. Result: 15 passed;
clippy clean. `to_fixed_slots` was left out (dead code until T7, which adds
it — see T7).

### T6 — DeviceSlots decoding

**Files touched:** `person/src/slots.rs`, `person/tests/slots_decode.rs`
**Parallel:** no (serial, after T5)

1. Failing test `person/tests/slots_decode.rs`:

```rust
use ed25519_dalek::SigningKey;
use person::{DevicePublicKey, DeviceSlots};

fn dev(seed: u8) -> DevicePublicKey {
    DevicePublicKey::new(SigningKey::from_bytes(&[seed; 32]).verifying_key())
}

fn sorted(mut v: Vec<DevicePublicKey>) -> Vec<DevicePublicKey> {
    v.sort();
    v
}

/// verifies: LLR-sjrh7z, REQ-4szc22
#[test]
fn decoding_accepts_only_strictly_increasing_sets() {
    let canonical = sorted(vec![dev(1), dev(2)]);
    let bytes = postcard::to_allocvec(&canonical).expect("encode");
    let slots = postcard::from_bytes::<DeviceSlots>(&bytes).expect("decode");
    assert_eq!(slots.devices(), canonical.as_slice());

    let reversed: Vec<_> = canonical.iter().rev().copied().collect();
    let bytes = postcard::to_allocvec(&reversed).expect("encode");
    assert!(postcard::from_bytes::<DeviceSlots>(&bytes).is_err());

    let duplicated = vec![dev(1), dev(1)];
    let bytes = postcard::to_allocvec(&duplicated).expect("encode");
    assert!(postcard::from_bytes::<DeviceSlots>(&bytes).is_err());

    let empty: Vec<DevicePublicKey> = vec![];
    let bytes = postcard::to_allocvec(&empty).expect("encode");
    assert_eq!(postcard::from_bytes::<DeviceSlots>(&bytes).expect("decode").device_count(), 0);
}

/// verifies: LLR-sjrh7z, REQ-4szc22, REQ-vxx8k3
#[test]
fn decoding_refuses_a_set_over_the_bound() {
    let five = sorted(vec![dev(1), dev(2), dev(3), dev(4), dev(5)]);
    let bytes = postcard::to_allocvec(&five).expect("encode");
    assert!(postcard::from_bytes::<DeviceSlots>(&bytes).is_err());
}

/// A declared length far beyond the bound is refused without allocating it.
/// verifies: LLR-sjrh7z, REQ-vxx8k3
#[test]
fn decoding_a_huge_declared_length_is_refused_cheaply() {
    // postcard varint for 2^32 - 1 elements, then nothing.
    let bytes = [0xff, 0xff, 0xff, 0xff, 0x0f];
    assert!(postcard::from_bytes::<DeviceSlots>(&bytes).is_err());
}

/// verifies: LLR-sjrh7z
#[test]
fn encoding_round_trips() {
    let slots = DeviceSlots::new(vec![dev(3), dev(1)]).expect("two");
    let bytes = postcard::to_allocvec(&slots).expect("encode");
    assert_eq!(postcard::from_bytes::<DeviceSlots>(&bytes).expect("decode"), slots);
}
```

2. `cargo test -p person --test slots_decode` → fails to compile
   (`DeviceSlots: Deserialize` not satisfied).
3. Append to `person/src/slots.rs` — a visitor, so a sequence is refused at the
   first element beyond `MAX_DEVICES` instead of after collecting it:

```rust
#[cfg(feature = "serde")]
impl serde::Serialize for DeviceSlots {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.slots.serialize(s)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DeviceSlots {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct SlotsVisitor;

        impl<'de> serde::de::Visitor<'de> for SlotsVisitor {
            type Value = DeviceSlots;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "at most {MAX_DEVICES} device keys in strictly increasing order")
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<DeviceSlots, A::Error> {
                let mut slots: Vec<DevicePublicKey> = Vec::with_capacity(MAX_DEVICES);
                while let Some(device) = seq.next_element::<DevicePublicKey>()? {
                    if slots.len() == MAX_DEVICES {
                        return Err(serde::de::Error::custom("device slots exceed MAX_DEVICES"));
                    }
                    if slots.last().is_some_and(|last| *last >= device) {
                        return Err(serde::de::Error::custom(
                            "device slots must be strictly increasing (sorted, no duplicates)",
                        ));
                    }
                    slots.push(device);
                }
                Ok(DeviceSlots { slots })
            }
        }

        d.deserialize_seq(SlotsVisitor)
    }
}
```

4. `cargo test -p person --test slots_decode` → `test result: ok. 4 passed`.
5. Commit: `feat(person): DeviceSlots decoding refuses non-canonical and oversized sets`.

**Done (15e7d17).** red -> green: decoding_accepts_only_strictly_increasing_sets,
decoding_refuses_a_set_over_the_bound,
decoding_a_huge_declared_length_is_refused_cheaply, encoding_round_trips —
each watched fail for the right reason (E0277 `DeviceSlots: Deserialize` not
satisfied) before the implementation existed. Result: 19 passed; clippy clean.
**Weak test, for the deslop pass:** decoding_a_huge_declared_length_is_refused_cheaply
does not discriminate — on `[ff ff ff ff 0f]` postcard fails on the missing
bytes (`DeserializeUnexpectedEnd`) before the visitor's bound check, and a
naive `Vec` decoder passes it too (postcard's `size_hint` is `None` when the
declared length exceeds the remaining bytes; serde caps hints with
`size_hint::cautious`). The discriminating input is a declared length of
2^32 − 1 followed by five sorted valid keys: the visitor refuses at the fifth
(`SerdeDeCustom`) where a naive decoder reads all five and fails on EOF. Add
that test, asserting the custom error, and prove by mutation (a naive
`Vec::deserialize` then check) that it fails without the visitor.

### T7 — DeviceTrieHasher, NodeHash and the device root

**Files touched:** `person/src/device_trie.rs`, `person/src/lib.rs`, `person/src/slots.rs`, `person/tests/device_trie.rs`

(T7 also adds `DeviceSlots::to_fixed_slots` to `person/src/slots.rs`, exactly
as T5 step 3 shows it: T5 left it out because it is dead code until
`compute_device_root` calls it.)
**Parallel:** no (serial, after T6)

1. Failing test `person/tests/device_trie.rs` (two test hashers with different
   domains; the "org" one reproduces org-members' exact keys and sentinel, so
   T8's byte-identity is checked here first):

```rust
use ed25519_dalek::SigningKey;
use person::{compute_device_root, DevicePublicKey, DeviceSlots, DeviceTrieHasher, NodeHash};

#[derive(Clone)]
struct OrgDomains;
impl DeviceTrieHasher for OrgDomains {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1";
    fn hash_device_leaf(data: &[u8]) -> NodeHash {
        NodeHash::new(blake3::keyed_hash(b"org-members::device-leaf________", data).into())
    }
    fn hash_device_node(l: &NodeHash, r: &NodeHash) -> NodeHash {
        let mut h = blake3::Hasher::new_keyed(b"org-members::device-node________");
        h.update(l.as_bytes());
        h.update(r.as_bytes());
        NodeHash::new(h.finalize().into())
    }
}

#[derive(Clone)]
struct OtherDomains;
impl DeviceTrieHasher for OtherDomains {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_TEST_OTHER_V1";
    fn hash_device_leaf(data: &[u8]) -> NodeHash {
        NodeHash::new(blake3::keyed_hash(b"test::other-device-leaf_________", data).into())
    }
    fn hash_device_node(l: &NodeHash, r: &NodeHash) -> NodeHash {
        let mut h = blake3::Hasher::new_keyed(b"test::other-device-node_________");
        h.update(l.as_bytes());
        h.update(r.as_bytes());
        NodeHash::new(h.finalize().into())
    }
}

fn dev(seed: u8) -> DevicePublicKey {
    DevicePublicKey::new(SigningKey::from_bytes(&[seed; 32]).verifying_key())
}

fn slots(seeds: &[u8]) -> DeviceSlots {
    DeviceSlots::new(seeds.iter().map(|s| dev(*s)).collect()).expect("valid set")
}

/// verifies: LLR-6ezhw7, LLR-4vsm8d, REQ-mu3qgz
#[test]
fn the_root_is_the_documented_depth_two_tree() {
    let set = slots(&[2, 1]);
    let d = set.devices();
    let empty = OrgDomains::hash_device_leaf(OrgDomains::DEVICE_EMPTY_SENTINEL);
    let l0 = OrgDomains::hash_device_leaf(d[0].as_bytes());
    let l1 = OrgDomains::hash_device_leaf(d[1].as_bytes());
    let expected = OrgDomains::hash_device_node(
        &OrgDomains::hash_device_node(&l0, &l1),
        &OrgDomains::hash_device_node(&empty, &empty),
    );
    assert_eq!(compute_device_root::<OrgDomains>(&set), expected);
}

/// verifies: LLR-6ezhw7, REQ-mu3qgz
#[test]
fn equal_sets_give_equal_roots_whatever_the_construction_order() {
    assert_eq!(
        compute_device_root::<OrgDomains>(&slots(&[1, 2, 3])),
        compute_device_root::<OrgDomains>(&slots(&[3, 1, 2]))
    );
}

/// verifies: LLR-6ezhw7, REQ-mu3qgz
#[test]
fn different_sets_give_different_roots() {
    let sets = [slots(&[]), slots(&[1]), slots(&[2]), slots(&[1, 2]), slots(&[1, 2, 3, 4])];
    for (i, a) in sets.iter().enumerate() {
        for b in &sets[i + 1..] {
            assert_ne!(compute_device_root::<OrgDomains>(a), compute_device_root::<OrgDomains>(b));
        }
    }
}

/// verifies: LLR-4vsm8d, REQ-mu3qgz
#[test]
fn the_same_set_under_another_domain_gives_another_root() {
    for set in [slots(&[]), slots(&[1]), slots(&[1, 2, 3, 4])] {
        assert_ne!(
            compute_device_root::<OrgDomains>(&set),
            compute_device_root::<OtherDomains>(&set)
        );
    }
}
```

2. `cargo test -p person --test device_trie` → fails to compile.
3. `person/src/device_trie.rs` — `NodeHash` and the sub-trie moved from
   org-members (`types.rs`, `device_trie.rs`), the sentinel now the hasher's:

```rust
use core::fmt;

use crate::slots::DeviceSlots;

/// A 32-byte hash output.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeHash([u8; 32]);

impl NodeHash {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for NodeHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for NodeHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [b0, b1, b2, b3, ..] = self.0;
        write!(f, "NodeHash({b0:02x}{b1:02x}{b2:02x}{b3:02x}..)")
    }
}

/// The hashing a device sub-trie is computed through. The implementor
/// chooses the domains and the empty-slot sentinel. LLR-4vsm8d.
pub trait DeviceTrieHasher {
    /// Hashed with `hash_device_leaf` to fill an unoccupied slot.
    const DEVICE_EMPTY_SENTINEL: &'static [u8];

    /// Hash a device public key (device-leaf domain).
    fn hash_device_leaf(data: &[u8]) -> NodeHash;

    /// Hash two device child hashes into a parent (device-node domain).
    fn hash_device_node(left: &NodeHash, right: &NodeHash) -> NodeHash;
}

/// The root of the depth-2, four-slot device sub-trie over `devices`, keys in
/// sorted order, unoccupied slots hashing the sentinel. LLR-6ezhw7.
pub fn compute_device_root<H: DeviceTrieHasher>(devices: &DeviceSlots) -> NodeHash {
    let empty = H::hash_device_leaf(H::DEVICE_EMPTY_SENTINEL);
    let [s0, s1, s2, s3] = devices.to_fixed_slots();
    let leaf = |slot: Option<crate::DevicePublicKey>| match slot {
        Some(device) => H::hash_device_leaf(device.as_bytes()),
        None => empty,
    };
    let left = H::hash_device_node(&leaf(s0), &leaf(s1));
    let right = H::hash_device_node(&leaf(s2), &leaf(s3));
    H::hash_device_node(&left, &right)
}
```

   The `[s0, s1, s2, s3]` pattern compiles only while `MAX_DEVICES` is 4,
   which ties the tree's shape to the bound at compile time. In `lib.rs`:
   `pub mod device_trie;` and
   `pub use device_trie::{compute_device_root, DeviceTrieHasher, NodeHash};`.
4. `cargo test -p person` → all person tests pass (`2 + 4 + 4 + 2 + 3 + 4 + 4`).
   `cargo clippy -p person --all-targets -- -D warnings` → no warnings.
5. Commit: `feat(person): DeviceTrieHasher, NodeHash and the device root`.

**Done (6561efd).** red -> green: the_root_is_the_documented_depth_two_tree,
the_device_root_matches_org_members_today, equal_sets_give_equal_roots_whatever_the_construction_order,
different_sets_give_different_roots, the_same_set_under_another_domain_gives_another_root
— each watched fail for the right reason (E0432 unresolved imports
`compute_device_root`, `DeviceTrieHasher`, `NodeHash`) before the
implementation existed. Result: 24 passed (2+4+5+4+2+3+4); clippy clean.
Added beyond the plan: the_device_root_matches_org_members_today pins
`2f45f1f7c6362a0bff09da153031a942c2ba77cc102bb6eb923d7ee154bcd6fd`, the root
org-members' own `compute_device_root::<Blake3Hasher>` gave for the [1;32]
and [2;32] device keys before the move (taken from a temporary test module,
not committed) — byte-identity across the move, checked before T8 deletes
the original.

### T8 — Switch org-members, org-node and app to `person`

**Files touched:** `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/types.rs`, `org-members/src/hasher.rs`, `org-members/src/device_trie.rs` (deleted), `org-members/src/smt.rs`, `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/error.rs`, `org-members/src/normalize.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/tests/newtypes.rs`, every `org-node/**/*.rs` and `app/src-tauri/**/*.rs` file the rename touches (listed by step 1's grep), `org-members/.guardrails/config.yaml`, `org-members/docs/architecture/2026-09-17-decomposition.md`, `org-members/docs/architecture/soup.md`, `Cargo.lock`
**Parallel:** no (serial, after T7)

No new test is written: the proof is that every existing suite stays green
and `tests/encoding_golden.rs` passes **unchanged** (member hashes
byte-identical).

1. List the files the rename touches, and record the list in the commit:
   `grep -rlE 'P2pDeviceKey|P2pDeviceSlots|P2pMemberKey|device_trie|NodeHash|Name|Surname' org-members/src org-members/tests org-node app/src-tauri --include='*.rs'`
2. `org-members/Cargo.toml` `[dependencies]`: add
   `person = { path = "../person", default-features = false }` and, in
   `[features]`, make `serde` also enable `person/serde`:
   `serde = ["dep:serde", "dep:postcard", "person/serde"]`.
3. `org-members/src/types.rs`: delete `MAX_DEVICES`, `MAX_NAME_LEN`,
   `MAX_SURNAME_LEN`, `P2pMemberKey` (and its impls), `P2pDeviceKey` (and its
   impls), `nfc_bounded`, `Name`, `Surname`, `NodeHash`, `P2pDeviceSlots` (and
   its impls). Add at the top
   `pub use person::{DevicePublicKey, DeviceSlots, Name, NodeHash, PersonPublicKey, Surname, MAX_DEVICES, MAX_NAME_LEN, MAX_SURNAME_LEN};`.
   `HeldKey`'s two `From` impls take `&PersonPublicKey` and `&DevicePublicKey`.
   `RootHash`'s `From<NodeHash>` becomes `Self(*h.as_bytes())`.
   `validated_string_impls!` stays (it serves `Handle`).
4. `org-members/src/hasher.rs`:

```rust
use person::{DeviceTrieHasher, NodeHash};

/// Pluggable hash function for the Merkle trie: the device domains through
/// `person`'s `DeviceTrieHasher`, plus the member domains.
pub trait TrieHasher: DeviceTrieHasher + Clone + Send + Sync {
    /// Hash a serialized member leaf (domain: MEMBER_LEAF).
    fn hash_member_leaf(data: &[u8]) -> NodeHash;

    /// Hash two child node hashes into a parent (domain: MEMBER_NODE).
    fn hash_member_node(left: &NodeHash, right: &NodeHash) -> NodeHash;
}

/// Blake3-based hasher. Uses domain separation via context strings.
#[derive(Clone, Debug)]
pub struct Blake3Hasher;

impl DeviceTrieHasher for Blake3Hasher {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1";

    fn hash_device_leaf(data: &[u8]) -> NodeHash {
        NodeHash::new(blake3::keyed_hash(b"org-members::device-leaf________", data).into())
    }

    fn hash_device_node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        let mut hasher = blake3::Hasher::new_keyed(b"org-members::device-node________");
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        NodeHash::new(hasher.finalize().into())
    }
}

impl TrieHasher for Blake3Hasher {
    fn hash_member_leaf(data: &[u8]) -> NodeHash {
        NodeHash::new(blake3::keyed_hash(b"org-members::member-leaf________", data).into())
    }

    fn hash_member_node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        let mut hasher = blake3::Hasher::new_keyed(b"org-members::member-node________");
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        NodeHash::new(hasher.finalize().into())
    }
}
```

   Every other `TrieHasher` implementor in the workspace (grep
   `impl TrieHasher for`) gets the same split: its device methods move into an
   `impl DeviceTrieHasher` with `DEVICE_EMPTY_SENTINEL =
   b"EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1"`.
5. Delete `org-members/src/device_trie.rs`; remove `pub mod device_trie;` from
   `lib.rs`; in `smt.rs` replace `use crate::device_trie::compute_device_root;`
   with `use person::compute_device_root;`. Any remaining caller of
   `device_empty_leaf_hash::<H>()` becomes
   `H::hash_device_leaf(H::DEVICE_EMPTY_SENTINEL)`.
6. Errors: in `org-members/src/error.rs` add `InvalidDeviceKey` and
   `InvalidPersonKey` variants and

```rust
impl From<person::IdentityError> for OrgMembersError {
    fn from(e: person::IdentityError) -> Self {
        use person::IdentityError as I;
        match e {
            I::FieldTooLong { field, max } => Self::FieldTooLong { field, max },
            I::InvalidDeviceKey => Self::InvalidDeviceKey,
            I::InvalidPersonKey => Self::InvalidPersonKey,
            I::DeviceSlotsFull => Self::DeviceSlotsFull,
            I::DuplicateDevice => Self::DuplicateDevice,
            I::DeviceNotFound => Self::DeviceNotFound,
        }
    }
}
```

   so every `?` on a `person` call keeps org-members' existing error variants.
   `normalize::to_nfc` stays (handles use it).
7. Rename across `org-members`, `org-node`, `app/src-tauri` (files from step 1):
   `P2pDeviceKey` → `DevicePublicKey`, `P2pDeviceSlots` → `DeviceSlots`,
   `P2pMemberKey` → `PersonPublicKey`:
   `perl -pi -e 's/\bP2pDeviceKey\b/DevicePublicKey/g; s/\bP2pDeviceSlots\b/DeviceSlots/g; s/\bP2pMemberKey\b/PersonPublicKey/g' <files>`.
   Then fix each construction of a member key — `P2pMemberKey::new(vk)` is now
   `PersonPublicKey::from_bytes(vk.as_bytes())?` in library code (mapped
   through step 6) and `.expect("valid key")` in tests; any
   `.verifying_key()` on a member key becomes `.as_bytes()`. `lib.rs`'s
   `pub use types::{...}` re-exports `DevicePublicKey`, `DeviceSlots`,
   `PersonPublicKey` in place of the old names.
8. Tests comparing `Name::parse`/`Surname::parse` errors against
   `OrgMembersError::FieldTooLong` compare against
   `person::IdentityError::FieldTooLong` instead.
9. Config and documents (true from this step on):
   - `org-members/.guardrails/config.yaml`: under the existing `# depends_on:`
     comment block, add
     ```yaml
     # Declared 2026-10-04 after the dependency assessment in
     # docs/plans/2026-10-04-person-shared-types.md. person is class C, so the
     # class floor has nothing to convict.
     depends_on:
       - person
     ```
     (replacing the commented `# depends_on:` / `#   - <unit>` lines).
   - `org-members/docs/architecture/2026-09-17-decomposition.md`: append to
     LLR-w5nkbu, LLR-pys2ek, LLR-kdhd2v and LLR-72p8bz each the paragraph
     `(Amended 2026-10-04: realised through the person unit's types — LLR-ayu93n, LLR-9p34cv, LLR-6ezhw7 and LLR-4vsm8d respectively — which org-members re-exports; org-members' TrieHasher extends person's DeviceTrieHasher, and Blake3Hasher keeps the org-members device domain keys and sentinel, so every hash is unchanged.)`
     with the one matching LLR named in each.
   - `org-members/docs/architecture/soup.md`: add a row for `person` (path
     dependency, class C unit, REQ-r7mytp…REQ-vxx8k3), and in the
     `ed25519-dalek` and `serde` rows replace `P2pMemberKey`/`P2pDeviceKey`
     with "`person`'s `DevicePublicKey` (and, until the X25519 step,
     `PersonPublicKey`)".
10. Run, and paste the outputs into the commit body:
    - `cargo test -p person`
    - `QUINT_HOME=<scratch>/quint cargo test -p org-members` — every test
      passes, `encoding_golden` included and **unedited**
      (`git diff --stat org-members/tests/encoding_golden.rs` is empty)
    - org-members' quint `verify_commands`
    - `cargo test -p org-node --features chain` (org-node's verify command as
      configured)
    - app's `verify_commands`
    - `.guardrails/scripts/check-units.sh`, and `check-trace.sh` with
      `GR_CONFIG` for each of the five units
11. Commit: `refactor(org-members): use person's identity types; member hashes unchanged`.

**Done (5cb04f8, 4416c0d).** Refactor: no new test; the existing suites are
the proof. person 24 passed; org-members 0+3+9+161+32 (1 ignored)+16+0 —
identical to baseline, `encoding_golden` included; org-members' quint
commands green (3 typechecks, 63 passing, no mbtInv violation); org-node 57
passed (baseline 57), every org-node target builds; app 78 passed (baseline
78); check-units rc 0; check-trace rc 0 for org-members, org-node, app, and
for person only the MISSING-TEST owed by T9 (REQ-3vqs9b, LLR-vs7etb).
Deviations, each reviewed by the dispatcher:
- `encoding_golden.rs`: the subagent stopped rather than edit it (the
  dispatch said never to). The dispatcher changed the import and the two
  LEAF-CONSTRUCTION constructors only — the lines the file itself designates
  as changeable by a refactor — and left `GOLDEN_ALICE_LEAF`, `GOLDEN_ROOT`
  and every assertion untouched; 3/3 pass, so member bytes and hashes are
  identical across the move.
- org-members' design amendments cite person's exported REQs (REQ-r7mytp,
  REQ-4szc22, REQ-mu3qgz), not its LLRs: a unit may reference only another
  unit's exported REQs (NON-EXPORTED-REF).
- `SigningKeypair::member_key()` (org-node) now returns a `Result`, since
  `PersonPublicKey::from_bytes` can refuse; org-node maps `Name`/`Surname`
  errors through `OrgMembersError::from` so they still surface as
  `OrgNodeError::Trie(FieldTooLong)`.
- `app/src-tauri/Cargo.lock` was set back to lockfile version 3 after cargo
  rewrote it; `--locked` builds pass.
- For the deslop pass: some import lists are out of order after the rename.

### T9 — PersonPublicKey: port to X25519, test-first

**Files touched:** `person/src/person_key.rs`, `person/tests/person_key.rs`, `person/Cargo.toml`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/newtypes.rs`, the org-node and app test files that construct a `PersonPublicKey` (grep `PersonPublicKey::from_bytes`), `org-members/docs/architecture/soup.md`
**Parallel:** no (serial, after T8)

1. Replace `person/tests/person_key.rs` with the failing tests:

```rust
use curve25519_dalek::montgomery::MontgomeryPoint;
use person::{IdentityError, PersonPublicKey};

/// The small-order X25519 u-coordinates libsodium refuses
/// (crypto_scalarmult/curve25519/ref10/x25519_ref10.c, `has_small_order`).
const SMALL_ORDER: [[u8; 32]; 7] = [
    [0; 32],
    {
        let mut b = [0; 32];
        b[0] = 1;
        b
    },
    [
        0xe0, 0xeb, 0x7a, 0x7c, 0x3b, 0x41, 0xb8, 0xae, 0x16, 0x56, 0xe3, 0xfa, 0xf1, 0x9f, 0xc4, 0x6a,
        0xda, 0x09, 0x8d, 0xeb, 0x9c, 0x32, 0xb1, 0xfd, 0x86, 0x62, 0x05, 0x16, 0x5f, 0x49, 0xb8, 0x00,
    ],
    [
        0x5f, 0x9c, 0x95, 0xbc, 0xa3, 0x50, 0x8c, 0x24, 0xb1, 0xd0, 0xb1, 0x55, 0x9c, 0x83, 0xef, 0x5b,
        0x04, 0x44, 0x5c, 0xc4, 0x58, 0x1c, 0x8e, 0x86, 0xd8, 0x22, 0x4e, 0xdd, 0xd0, 0x9f, 0x11, 0x57,
    ],
    p_plus(-1),
    p_plus(0),
    p_plus(1),
];

/// Little-endian encoding of 2^255 - 19 + delta, for |delta| <= 1.
const fn p_plus(delta: i8) -> [u8; 32] {
    let mut b = [0xff; 32];
    b[31] = 0x7f;
    b[0] = (0xed_i16 + delta as i16) as u8;
    b
}

fn x25519_public(seed: u8) -> [u8; 32] {
    MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes()
}

/// verifies: LLR-vs7etb, REQ-3vqs9b
#[test]
fn accepts_a_canonical_x25519_public_key_and_holds_its_bytes() {
    for seed in 1..=16 {
        let bytes = x25519_public(seed);
        let key = PersonPublicKey::from_bytes(&bytes).expect("valid X25519 key");
        assert_eq!(key.as_bytes(), &bytes);
    }
}

/// verifies: LLR-vs7etb, REQ-3vqs9b
#[test]
fn refuses_every_small_order_u_coordinate() {
    for u in SMALL_ORDER {
        assert_eq!(PersonPublicKey::from_bytes(&u), Err(IdentityError::InvalidPersonKey), "{u:02x?}");
    }
}

/// The list above is checked, not trusted: each canonical entry is a point of
/// small order according to curve25519-dalek.
/// verifies: LLR-vs7etb
#[test]
fn the_small_order_list_is_what_it_claims() {
    for u in &SMALL_ORDER[..4] {
        let point = MontgomeryPoint(*u).to_edwards(0).expect("on the curve");
        assert!(point.is_small_order(), "{u:02x?}");
    }
}

/// verifies: LLR-vs7etb, REQ-3vqs9b
#[test]
fn refuses_non_canonical_encodings() {
    let mut top_bit = x25519_public(5);
    top_bit[31] |= 0x80;
    assert_eq!(PersonPublicKey::from_bytes(&top_bit), Err(IdentityError::InvalidPersonKey));
    for delta in 2..=18_i8 {
        assert_eq!(
            PersonPublicKey::from_bytes(&p_plus(delta)),
            Err(IdentityError::InvalidPersonKey),
            "p + {delta}"
        );
    }
}

/// verifies: LLR-vs7etb, REQ-3vqs9b
#[test]
fn decoding_goes_through_from_bytes() {
    let bytes = postcard::to_allocvec(&SMALL_ORDER[2]).expect("encode");
    assert!(postcard::from_bytes::<PersonPublicKey>(&bytes).is_err());
    let good = PersonPublicKey::from_bytes(&x25519_public(9)).expect("valid");
    let bytes = postcard::to_allocvec(&good).expect("encode");
    assert_eq!(postcard::from_bytes::<PersonPublicKey>(&bytes).expect("decode"), good);
}
```

   Add to `person/Cargo.toml` `[dev-dependencies]`:
   `curve25519-dalek = "4"`.
2. `cargo test -p person --test person_key` → compiles; `refuses_every_small_order_u_coordinate`
   fails (interim ed25519 validation accepts some), and
   `accepts_a_canonical_x25519_public_key...` fails for the seeds whose X25519
   bytes are not ed25519 points. `the_small_order_list_is_what_it_claims`
   passes — if it fails, stop: the constants are wrong.
3. `person/src/person_key.rs`, replacing `from_bytes` (no production
   dependency on curve25519-dalek: the check is exact byte comparison):

```rust
/// Little-endian 2^255 - 19.
const P: [u8; 32] = {
    let mut b = [0xff; 32];
    b[0] = 0xed;
    b[31] = 0x7f;
    b
};

/// Small-order u-coordinates below p (libsodium `has_small_order`; the
/// entries at and above p are refused as non-canonical).
const SMALL_ORDER: [[u8; 32]; 4] = [
    [0; 32],
    {
        let mut b = [0; 32];
        b[0] = 1;
        b
    },
    [
        0xe0, 0xeb, 0x7a, 0x7c, 0x3b, 0x41, 0xb8, 0xae, 0x16, 0x56, 0xe3, 0xfa, 0xf1, 0x9f, 0xc4, 0x6a,
        0xda, 0x09, 0x8d, 0xeb, 0x9c, 0x32, 0xb1, 0xfd, 0x86, 0x62, 0x05, 0x16, 0x5f, 0x49, 0xb8, 0x00,
    ],
    [
        0x5f, 0x9c, 0x95, 0xbc, 0xa3, 0x50, 0x8c, 0x24, 0xb1, 0xd0, 0xb1, 0x55, 0x9c, 0x83, 0xef, 0x5b,
        0x04, 0x44, 0x5c, 0xc4, 0x58, 0x1c, 0x8e, 0x86, 0xd8, 0x22, 0x4e, 0xdd, 0xd0, 0x9f, 0x11, 0x57,
    ],
];

/// `p - 1` is the remaining small-order entry below p.
const P_MINUS_ONE: [u8; 32] = {
    let mut b = P;
    b[0] = 0xec;
    b
};

/// True when the little-endian `u` is below p.
fn below_p(u: &[u8; 32]) -> bool {
    for (a, b) in u.iter().rev().zip(P.iter().rev()) {
        if a != b {
            return a < b;
        }
    }
    false
}

impl PersonPublicKey {
    /// A canonical, non-small-order X25519 public key. LLR-vs7etb.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, IdentityError> {
        let canonical = bytes[31] & 0x80 == 0 && below_p(bytes);
        let small_order = SMALL_ORDER.contains(bytes) || *bytes == P_MINUS_ONE;
        if canonical && !small_order {
            Ok(Self(*bytes))
        } else {
            Err(IdentityError::InvalidPersonKey)
        }
    }
}
```

   (`bytes[31]` indexes a fixed-size array with a constant, not an
   input-derived value.) Update the doc comment on the type: drop "Validation
   is interim (ed25519) until the X25519 step".
4. `cargo test -p person` → all pass.
5. Fixtures across org-members, org-node, app. A member key that must also be
   usable as a device key (the DuplicateKey tests, `scenario_member_key_swap`)
   needs bytes valid as both an ed25519 point and a canonical X25519 key. Add
   to each test file that builds member keys (replacing its `member_key` /
   `real_member_key` helper body):

```rust
/// 32 bytes valid both as an ed25519 public key and as a canonical,
/// non-small-order X25519 public key: the first seed variant whose ed25519
/// encoding has its top bit clear.
fn dual_key_bytes(seed: &str) -> [u8; 32] {
    (0u32..)
        .map(|i| {
            let s: [u8; 32] = blake3::hash(format!("{seed}/{i}").as_bytes()).into();
            *ed25519_dalek::SigningKey::from_bytes(&s).verifying_key().as_bytes()
        })
        .find(|b| PersonPublicKey::from_bytes(b).is_ok())
        .expect("about half of all seeds qualify")
}

fn member_key(seed: &str) -> PersonPublicKey {
    PersonPublicKey::from_bytes(&dual_key_bytes(seed)).expect("dual-valid bytes")
}
```

   Where a test builds a device key from the same seed as a member key, build
   it from `dual_key_bytes(seed)` with `DevicePublicKey::from_bytes`.
6. `tests/encoding_golden.rs`: run it. If its member key's bytes are no longer
   a valid `PersonPublicKey`, change only the seed derivation of the member
   key to `dual_key_bytes`, re-derive `GOLDEN_ALICE_LEAF` and `GOLDEN_ROOT` by
   running the test and copying the printed values, and state in the commit
   body that the golden values changed **because the input key changed**, the
   hashing being untouched (T8 proved that).
7. Run every command of T8 step 10 and paste the outputs.
8. In `org-members/docs/architecture/soup.md`'s `ed25519-dalek` row, drop
   "(and, until the X25519 step, `PersonPublicKey`)".
9. Commit: `feat(person): PersonPublicKey validated as X25519`.

**Split by the owner's order of work (2026-10-04):** steps 1–4 (person) now;
steps 5–9 for org-members next (step 2 of the order); org-node and app
fixtures with the org-node work (step 4 of the order). `role` is out of scope.

**Steps 1–4 done (baf851c).** Historical: the test names and "the four
canonical entries" below record the state at baf851c. The tests were renamed
later (`refuses_*` to `rejects_*`, `holds` to `contains`, `from_bytes` to
`parse`), and p − 1 joined the entries the_small_order_list_is_what_it_claims
checks. red -> green:
accepts_a_canonical_x25519_public_key_and_holds_its_bytes (interim ed25519
check refused an X25519 key), refuses_every_small_order_u_coordinate
(all-zero u accepted), refuses_non_canonical_encodings (p + 3 accepted),
decoding_goes_through_from_bytes (valid X25519 key refused) — each watched
fail for the right reason before the implementation existed.
the_small_order_list_is_what_it_claims checks the constants, not the
implementation, and passed from the start, as step 2 expects; it confirms
the four canonical entries are small-order per curve25519-dalek 4.1.3.
Result: person 27 passed; clippy clean. org-members then failed 180 tests
(golden 3, fuzz 8, integration 137, conformance 30, newtypes 2) — the
ed25519 member-key fixtures, migrated next. LLR-vs7etb's interim sentence is
removed from the design draft.

**Steps 5, 6, 8 for org-members done (9e93bfc).** Fixtures only, no
assertion and no library code changed. red -> green: before, encoding_golden
0/3, fuzz_tests 1/9, integration_test 24/161, mbt_conformance 2/32,
newtypes 14/16 (180 failures); after, 0+3+9+161+32 (1 ignored)+16+0 —
identical to baseline. Quint: 3 typechecks, 63 passing, no mbtInv violation.
check-trace (org-members) rc 0. Golden values re-derived because the old
member key (last byte 0xc8, top bit set) is not a canonical X25519 key: the
two leaf encodings differ only in the 32 member-key bytes
(`8092f31a…98c8` → `bf74dcf9…be78`); `GOLDEN_ROOT` `2e81afbd…a19d` →
`4740435a…efd4`. Hashing is unchanged (T8).

### T9c — Exported X25519 validity check (owner, 2026-10-04)

**Files touched:** `person/src/x25519.rs` (new), `person/src/person_key.rs`, `person/src/lib.rs`, `person/tests/x25519.rs` (new)
**Parallel:** no (serial, after T9)

The organisation's public key is an X25519 key owned by org-node, not
`person`; `person` exports the validity rule so org-node applies it without a
copy (REQ-7gz72r, LLR-m75m7u).

The code in this task is historical, as written before the review rounds:
`from_bytes` was later renamed `parse`, `P_MINUS_ONE` folded into
`SMALL_ORDER`, `P` renamed `FIELD_PRIME`, and `below_p` inlined into
`is_valid_public_key`. `person/src/x25519.rs` and
`person/tests/x25519.rs` are the current form.

1. Failing test `person/tests/x25519.rs` (open with the T2 clippy allow):

```rust
use curve25519_dalek::montgomery::MontgomeryPoint;
use person::{x25519, PersonPublicKey};

fn canonical(seed: u8) -> [u8; 32] {
    MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes()
}

/// verifies: LLR-m75m7u, REQ-7gz72r
#[test]
fn accepts_canonical_non_small_order_keys() {
    for seed in 1..=16 {
        assert!(x25519::is_valid_public_key(&canonical(seed)));
    }
}

/// verifies: LLR-m75m7u, REQ-7gz72r
#[test]
fn refuses_small_order_and_non_canonical_bytes() {
    let mut top_bit = canonical(3);
    top_bit[31] |= 0x80;
    let mut p = [0xff; 32];
    p[0] = 0xed;
    p[31] = 0x7f;
    let mut one = [0u8; 32];
    one[0] = 1;
    for bad in [[0u8; 32], one, top_bit, p] {
        assert!(!x25519::is_valid_public_key(&bad), "{bad:02x?}");
    }
}

/// PersonPublicKey applies exactly this rule.
/// verifies: LLR-m75m7u, REQ-3vqs9b
#[test]
fn person_public_key_accepts_exactly_what_the_check_accepts() {
    let mut cases: Vec<[u8; 32]> = (0u8..=40).map(|b| [b; 32]).collect();
    cases.extend((1..=16).map(canonical));
    for bytes in cases {
        assert_eq!(
            PersonPublicKey::from_bytes(&bytes).is_ok(),
            x25519::is_valid_public_key(&bytes),
            "{bytes:02x?}"
        );
    }
}
```

2. `cargo test -p person --test x25519` → fails to compile (no `person::x25519`).
3. Move `P`, `SMALL_ORDER`, `P_MINUS_ONE` and `below_p` from
   `person/src/person_key.rs` into a new `person/src/x25519.rs` and add:

```rust
/// True exactly when `bytes` are a canonical X25519 public key that is not of
/// small order. LLR-m75m7u.
pub fn is_valid_public_key(bytes: &[u8; 32]) -> bool {
    let canonical = bytes[31] & 0x80 == 0 && below_p(bytes);
    canonical && !SMALL_ORDER.contains(bytes) && *bytes != P_MINUS_ONE
}
```

   `PersonPublicKey::from_bytes` becomes
   `if x25519::is_valid_public_key(bytes) { Ok(Self(*bytes)) } else { Err(IdentityError::InvalidPersonKey) }`.
   In `lib.rs`: `pub mod x25519;`.
4. `cargo test -p person` → all pass (27 + 3); clippy clean.
5. Commit: `feat(person): export the X25519 public-key validity check`.

**Done (aff5e6b on worktree-worktree-person-shared-types).** red -> green:
accepts_canonical_non_small_order_keys, rejects_small_order_and_non_canonical_bytes,
person_public_key_accepts_exactly_what_the_check_accepts — each watched fail
with E0432 (unresolved import `person::x25519`) before the implementation
existed.

### T10 — Close the documents

**Files touched:** `person/docs/architecture/soup.md`, `person/docs/architecture/DRAFT-worktree-worktree-person-shared-types-shared-types.md`, `person/docs/risk/DRAFT-worktree-worktree-person-shared-types-shared-types.md`
**Parallel:** no (serial, after T9)

This task is historical, as written before the review rounds, and is
superseded. Step 1's removal of the `curve25519-dalek` row (dev-dependency
only) does not apply: `curve25519-dalek` is reached at runtime through
`ed25519-dalek`, and its row in `person/docs/architecture/soup.md` is a
runtime row. The task's other outputs — the SOUP inventory as built, the
risk file's result for the small-order claim, and the problem report PR-b7khyw
— were done during the review of the `person` merge.

1. Diff `person/docs/architecture/soup.md` against `person/Cargo.toml` and
   `Cargo.lock`: remove the `curve25519-dalek` row (dev-dependency only), set
   versions to what the lockfile resolves, and replace the "planned" status
   line with "As built 2026-10-04".
2. Record T3's finding about `ed25519-dalek` and small-order points in the
   risk file's "A claim to verify" paragraph, as a result. If `from_bytes`
   accepts small-order points (the expected outcome), raise the problem report
   against org-members' SOUP row with `.guardrails/scripts/new-id.sh PR` in
   `org-members/docs/problems/`, and put to the owner whether a
   `DevicePublicKey` must refuse small-order keys.
3. In the design draft, remove the sentence of LLR-vs7etb about the interim
   ed25519 validation.
4. `GR_CONFIG=person/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`
   → exit 0, no MISSING-TEST. Same for org-members, org-node, app; then
   `check-units.sh` and `check-ids.sh --allow-draft-files` per unit.
5. Commit: `docs(person): SOUP as built; ed25519 small-order finding recorded`.

Then `check-traceability`, `verify-before-merge`, `merge-change`.

## Self-review

1. Every ID in **Implements** is verified by a task's test: REQ-r7mytp (T2),
   REQ-q6xkna (T3), REQ-3vqs9b (T9), REQ-4szc22 (T5, T6), REQ-tq4ms4 (T6),
   REQ-mu3qgz and REQ-aj6x3n (T7), REQ-vxx8k3 (T1, T6), REQ-bczz87 (T1),
   REQ-7gz72r (T9c); LLR-ayu93n (T2), LLR-7guspr (T3),
   LLR-vs7etb (T9), LLR-9p34cv (T5), LLR-sjrh7z (T6), LLR-4vsm8d and
   LLR-6ezhw7 (T7), LLR-eeq89n and LLR-64muuw (T1), LLR-m75m7u (T9c, by
   `person/tests/x25519.rs`). SDD items are covered through their LLRs.
2. Code steps show real code and commands with expected results; T8's rename
   is a stated command over a stated file list.
3. Names are consistent: `IdentityError`, `Name`, `Surname`, `DevicePublicKey`,
   `PersonPublicKey`, `DeviceSlots`, `MAX_DEVICES`, `NodeHash`,
   `DeviceTrieHasher::{DEVICE_EMPTY_SENTINEL, hash_device_leaf, hash_device_node}`,
   `compute_device_root`.
4. Every task states its files and is serial: T2–T7 all edit `person/src/lib.rs`
   (T6 edits `slots.rs` after T5), and T8–T10 depend on their predecessors.
