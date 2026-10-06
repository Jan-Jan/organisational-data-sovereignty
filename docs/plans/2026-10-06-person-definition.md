# Person Definition Implementation Plan

**Goal:** Build the Person definition in the `person` unit — `Person` and its one constructor, the reusable group-key check, the successor check, the Person device sub-trie, the `V1` encoding, the keyed Person hash and its check — test-first, with serde decoding routed through `Person::new`, no-panic property tests and a bolero fuzz target.
**Implements:** REQ-9m5pq2, REQ-7n4g8b, REQ-wg7z4s, REQ-ht3x78, REQ-7qgx2q, REQ-bhez2u, REQ-7ymek3, REQ-r4keha; SDD-4r2x79, SDD-v32mqh, SDD-9hej83, SDD-u9cddc (amended); LLR-4ku56h, LLR-sjkmr6, LLR-kbhc43, LLR-63pkfc, LLR-rde6tk, LLR-edn55h, LLR-4ebtn4, LLR-tf45kx, LLR-wqha8d, LLR-3n3kxx. Through the REQs it realises RC-qn2r5b, RC-ee994g, RC-5gzfux, RC-bnm8qs and RC-d8n777.
**Safety class:** C (`person/.guardrails/config.yaml`, `safety_class: C`; no per-item overrides — the architecture draft states all three software items are class C).
**Verification:** `cargo test -p person` and `cargo clippy -p person --all-targets -- -D warnings` (the unit's `verify_commands`), each run as `CARGO_HOME=/tmp/cargo_home_fuzz cargo … --offline`; `make coverage-person` (floors 99/99); `GR_CONFIG=person/.guardrails/config.yaml .guardrails/scripts/check-trace.sh` and `… check-ids.sh --allow-draft-files`, both exit 0.

Context. Requirements, risk and design are this branch's drafts:
`person/docs/requirements/2026-10-06-person-definition.md`,
`person/docs/risk/2026-10-06-person-hazards.md` and
`person/docs/architecture/2026-10-06-person-definition.md`.
Decisions they rest on: `docs/plans/2026-10-04-person-sequencing.md` (change 2),
`docs/adr/2026-10-04-person-hash-unsalted.md`,
`docs/adr/2026-10-04-group-key-rules.md`. Every ID above is MISSING-TEST in
`check-trace.sh` today; every test this plan writes is annotated at the LLR
level, so each REQ is covered transitively (REQ-7n4g8b by LLR-tf45kx,
REQ-r4keha by LLR-3n3kxx and LLR-tf45kx, REQ-7ymek3 by LLR-63pkfc, and so on —
the LLRs' `satisfies:` lines).

## Decisions this plan makes

- **Empty-slot sentinel of `PersonDeviceHasher`:** the 31 ASCII bytes
  `EMPTY_SENTINEL_PERSON_DEVICE_V1`, the same shape as org-members'
  `EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1` and distinct from it. T5 amends
  LLR-edn55h to name it.
- **Domain keys, padded with `_` to 32 bytes:**
  `person::device-leaf_____________`, `person::device-node_____________`,
  `person::definition::v1__________` (the `&[u8; 32]` type of each constant
  makes the compiler check the length).
- **The group-key check is one public function,**
  `person::group_key::check_group_key(group_key: Option<&PersonPublicKey>, devices: &DeviceSlots) -> Result<(), IdentityError>`.
  `Option<&PersonPublicKey>` rather than `&Option<PersonPublicKey>`: a caller
  holding `Option<PersonPublicKey>` passes `.as_ref()`, and one holding a
  borrowed key needs no clone. `Person::new` calls it; org-members is not
  touched by this plan.
- **The version a caller states is a `u16`.** `person_hash(person, version: u16)`
  and `verify(person, version: u16, hash)` parse it with
  `EncodingVersion::try_from` first, so `UnsupportedEncodingVersion` is
  reachable from both, as LLR-3n3kxx and LLR-tf45kx require. `encode(person,
  EncodingVersion)` takes the parsed version.
- **`PersonHash`** is a newtype over the 32 bytes (parse-at-the-edge ADR:
  a Person hash is not interchangeable with a `NodeHash`). T5 amends
  LLR-4ebtn4 to state its shape.
- **Length prefixes:** `name.len() as u32`. A `const` assertion in `hash.rs`
  proves `MAX_NAME_LEN` and `MAX_SURNAME_LEN` fit a `u32`, so the cast is
  lossless and no unreachable error branch is added.
- **Module layout** (one file per task, so tasks run in parallel):
  `group_key.rs`, `definition.rs`, `definition_serde.rs` (private, `serde`
  only), `successor.rs`, `device_hasher.rs`, `encoding.rs`, `hash.rs`. T1
  declares every module, each file holding only its module doc; the task that
  owns a file replaces it whole. Root re-exports come last (T10), so tests
  before T10 name items by module path.
- **New dev-dependencies, both already in `Cargo.lock`:** `serde_json` 1.0.151
  (keeps a custom decode error's message, which LLR-63pkfc's tests check;
  postcard maps every custom error to `SerdeDeCustom`) and `bolero` 0.13.4 (the
  fuzz target, as in org-node and on-chain-client). `placeholder.rs` is deleted
  in T1, as its own comment asks once annotated tests exist.

## Golden values and their provenance

Computed 2026-10-06 while writing this plan, by running the exact code of
T1–T11 in the task worktree
`.worktrees/worktree-person-requirements-plan` (scratch, not committed):
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test person_device_trie --test person_hash`
with the expected strings first set to `GOLDEN_*` placeholders, reading the
`left:` value of each failed assertion, pasting it, and re-running to green.
There is no second blake3 implementation on this machine (no `b3sum`, no
Python `blake3`); the structural tests
(`the_root_is_the_depth_two_tree_under_the_person_domains`,
`the_v1_hash_is_blake3_keyed_by_the_v1_domain_key`) recompute each value from
`blake3::keyed_hash` directly, so the pins guard against change, the structure
tests against a wrong construction.

| Value | Hex |
|---|---|
| `compute_device_root::<PersonDeviceHasher>` of no devices | `db36a2843de361cd8fa691c95f305e206fbcdc666e0f1d1a5e494610df4421a5` |
| … of the devices of signing seeds `[1; 32]`, `[2; 32]` | `6b8a4eea74c7daf62a0e6b87112f24ad1037bbfdc5e8607eaa2e217bc8e9d4a5` |
| `person_hash(Alice Example, key u(seed 9), devices 1, 2; version 1)` | `d6177f1cbed18504b63666dd45b5f54834667606fa73de40c2dbaf28a704e292` |
| `person_hash(Alice Example, no key, no device; version 1)` | `225bc8ac0117ae0b3c4aaaec7654b81b3aa45a8f27a58eab7f365dd6fc8183f0` |

The same scratch run measured the finished unit: 122 tests pass (73 today),
`make coverage-person` reports 346 of 346 lines and 534 of 534 regions,
`cargo +nightly llvm-cov -p person --branch --summary-only` 36 of 36 branches,
clippy is clean with and without `--no-default-features`, and `check-trace.sh`
exits 0 with no MISSING-TEST.

## Execution

- Dispatch with `develop-change`, one task worktree per task:
  `.guardrails/scripts/task-worktree.sh start <tag>` from the change worktree
  (`.claude/worktrees/person-requirements`, branch `worktree-person-requirements`),
  `task-worktree.sh merge <tag>` after a green report.
- Every cargo command: `CARGO_HOME=/tmp/cargo_home_fuzz` (the default cargo
  home is read-only) and `--offline` (every crate is in that home and in
  `Cargo.lock`). Commands run from the task worktree's root.
- Commits: `git add <the task's files>` then
  `git -c commit.gpgsign=false commit -m "<message>"`, one plain git command per
  shell call, no `Co-Authored-By` line.
- Never edit a Rust test file with `sed` (AGENTS.md); use the Edit tool.

Waves (at most five tasks at once; a wave starts when every task it names as
"after" is merged):

| Wave | Tasks | After |
|---|---|---|
| 1 | T1 | — |
| 2 | T2, T3, T4, T5 (parallel) | T1 |
| 3 | T6 | T2 |
| 4 | T7, T8, T9 (parallel) | T6; T9 also T3, T4 |
| 5 | T10, T11 (parallel) | T7, T8, T9 |
| 6 | T12 | T10, T11 |

**Baseline (2026-10-06, before T1):** `cargo test -p person` — 73 passed over
11 test targets; clippy clean; `check-trace.sh` lists the 18 MISSING-TEST
items named in the request and nothing else.

---

### T1 — Error variants, blake3 a dependency, module skeleton

**Files touched:** `person/Cargo.toml`, `Cargo.lock`, `person/src/error.rs`, `person/src/lib.rs`, `person/src/definition.rs` (new), `person/src/definition_serde.rs` (new), `person/src/device_hasher.rs` (new), `person/src/encoding.rs` (new), `person/src/group_key.rs` (new), `person/src/hash.rs` (new), `person/src/successor.rs` (new), `person/tests/crate_discipline.rs`, `person/tests/placeholder.rs` (deleted)
**Parallel:** no (serial, first)

Trace: LLR-3n3kxx.

1. Failing test. In `person/tests/crate_discipline.rs`, extend the variant
   list of `every_rejection_is_a_distinct_named_variant` (Edit: replace
   `        IdentityError::DeviceNotFound,\n    ];` with):

```rust
        IdentityError::DeviceNotFound,
        IdentityError::KeyWithoutDevice,
        IdentityError::MissingPersonKey,
        IdentityError::PersonKeyIsDeviceKey,
        IdentityError::PersonKeyNotRotated,
        IdentityError::UnsupportedEncodingVersion(2),
    ];
```

   and append to the end of the file:

```rust

/// The variants the Person operations add, one per rule, each with a message
/// that names its rule.
/// verifies: LLR-3n3kxx
#[test]
fn the_person_variants_name_their_rule() {
    let messages = [
        (
            IdentityError::KeyWithoutDevice,
            "person public key held with no device public key",
        ),
        (
            IdentityError::MissingPersonKey,
            "person public key missing: one or more device public keys are held",
        ),
        (
            IdentityError::PersonKeyIsDeviceKey,
            "person public key equals a device public key",
        ),
        (
            IdentityError::PersonKeyNotRotated,
            "person public key not rotated: the device public keys changed and it did not",
        ),
        (
            IdentityError::UnsupportedEncodingVersion(7),
            "unsupported encoding version 7",
        ),
    ];
    for (variant, message) in messages {
        assert_eq!(variant.to_string(), message);
    }
    assert_ne!(
        IdentityError::UnsupportedEncodingVersion(2),
        IdentityError::UnsupportedEncodingVersion(3)
    );
}
```

2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test crate_discipline`
   → fails to compile:
   `error[E0599]: no variant or associated item named `KeyWithoutDevice` found for enum `IdentityError``
   (and the same for the other four).

3. `person/src/error.rs`, whole file:

```rust
/// Every rejection by a constructor or a Person operation in this crate.
/// Decoding rejects through the deserializer's error type, with this type's
/// message where the rule is one of these. LLR-eeq89n, LLR-3n3kxx.
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

    #[error("person public key held with no device public key")]
    KeyWithoutDevice,

    #[error("person public key missing: one or more device public keys are held")]
    MissingPersonKey,

    #[error("person public key equals a device public key")]
    PersonKeyIsDeviceKey,

    #[error("person public key not rotated: the device public keys changed and it did not")]
    PersonKeyNotRotated,

    #[error("unsupported encoding version {0}")]
    UnsupportedEncodingVersion(u16),
}
```

4. `person/Cargo.toml`, whole file (blake3 moves to `[dependencies]` with
   `default-features = false`; `serde_json` and `bolero` join the
   dev-dependencies):

```toml
[package]
name = "person"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
license = "GPL-3.0-only"
description = "An individual's identity types and Person definition: names, device keys, the PersonPublicKey, device slots, the device sub-trie and the Person hash"

[features]
default = ["serde"]
serde = ["dep:serde"]

[dependencies]
ed25519-dalek = { version = "2", default-features = false, features = ["alloc"] }
unicode-normalization = { version = "0.1", default-features = false }
serde = { version = "1", default-features = false, features = ["derive", "alloc"], optional = true }
thiserror = { version = "2", default-features = false }
blake3 = { version = "1", default-features = false }

[dev-dependencies]
postcard = { version = "1", features = ["alloc"] }
curve25519-dalek = "4"
proptest = "1"
serde_json = "1"
bolero = "0.13"

[lints]
workspace = true
```

5. `person/src/lib.rs`, whole file:

```rust
//! An individual's identity types: names, device keys, the PersonPublicKey a
//! grant is encoded against, device slots and the device sub-trie.
//! org-members uses them. On them this crate builds the Person definition, the
//! check of one definition against its predecessor, and the Person hash.

#![no_std]
#![deny(clippy::indexing_slicing)]

extern crate alloc;

pub mod definition;
#[cfg(feature = "serde")]
mod definition_serde;
pub mod device_hasher;
pub mod device_key;
pub mod device_trie;
pub mod encoding;
pub mod error;
pub mod group_key;
pub mod hash;
pub mod name;
pub mod person_key;
pub mod slots;
pub mod successor;
pub mod x25519;

pub use device_key::DevicePublicKey;
pub use device_trie::{compute_device_root, DeviceTrieHasher, NodeHash};
pub use error::IdentityError;
pub use name::{Name, Surname, MAX_NAME_LEN, MAX_SURNAME_LEN};
pub use person_key::PersonPublicKey;
pub use slots::{DeviceSlots, MAX_DEVICES};

/// Writes `type_name(aabbccdd..)`: the type name and the first four of the 32
/// bytes in hex. The Debug form of the crate's 32-byte key and hash types.
fn write_hex_prefix(
    formatter: &mut core::fmt::Formatter<'_>,
    type_name: &str,
    bytes: &[u8; 32],
) -> core::fmt::Result {
    let [byte0, byte1, byte2, byte3, ..] = *bytes;
    write!(
        formatter,
        "{type_name}({byte0:02x}{byte1:02x}{byte2:02x}{byte3:02x}..)"
    )
}
```

6. The seven module files, each holding only its module doc (the owning task
   replaces the file whole):

   `person/src/definition.rs`:
```rust
//! The Person definition: an individual's own record. SDD-4r2x79.
```
   `person/src/definition_serde.rs`:
```rust
//! `Person`'s serde implementations, compiled only with the `serde` feature.
//! Decoding goes through each field's own decoder and then `Person::new`.
//! LLR-63pkfc.
```
   `person/src/device_hasher.rs`:
```rust
//! The Person device sub-trie's hashing: this unit's own domain keys and
//! empty-slot sentinel, none of them org-members'. LLR-edn55h.
```
   `person/src/encoding.rs`:
```rust
//! The Person hash's encoding versions. LLR-rde6tk.
```
   `person/src/group_key.rs`:
```rust
//! The group-key rules a Person definition, and later a Member record, must
//! satisfy (docs/adr/2026-10-04-group-key-rules.md). LLR-sjkmr6, LLR-kbhc43.
```
   `person/src/hash.rs`:
```rust
//! The Person hash: an encoding of a definition under an encoding version,
//! hashed with blake3 keyed per unit and version, and the check of a supplied
//! hash. Free functions over their arguments; nothing is kept between calls.
//! SDD-v32mqh.
```
   `person/src/successor.rs`:
```rust
//! Whether one Person definition may follow another. SDD-9hej83.
```

7. Delete `person/tests/placeholder.rs` (`git rm person/tests/placeholder.rs`).

8. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test crate_discipline`
   → `test result: ok. 4 passed; 0 failed`. Cargo updates `Cargo.lock`
   offline: `git diff Cargo.lock` shows exactly `+ "bolero",` and
   `+ "serde_json",` in the `person` package's dependency list.

9. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person` → every
   target `ok`, 74 passed in total.
   `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy --offline -p person --all-targets -- -D warnings`
   → `Finished`, no warning.

10. Commit: `git add person/Cargo.toml Cargo.lock person/src/error.rs person/src/lib.rs person/src/definition.rs person/src/definition_serde.rs person/src/device_hasher.rs person/src/encoding.rs person/src/group_key.rs person/src/hash.rs person/src/successor.rs person/tests/crate_discipline.rs`,
    then `git -c commit.gpgsign=false commit -m "feat(person): IdentityError variants for the Person operations, blake3 a dependency, module skeleton"`.

---

### T2 — `check_group_key`

**Files touched:** `person/src/group_key.rs`, `person/tests/group_key.rs` (new)
**Parallel:** yes — after T1, alongside T3, T4, T5

Trace: LLR-sjkmr6, LLR-kbhc43, LLR-3n3kxx.

1. Failing test `person/tests/group_key.rs`:

```rust
//! check_group_key: a group key against the device slots it is held with.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::SigningKey;
use person::group_key::check_group_key;
use person::{DevicePublicKey, DeviceSlots, IdentityError, PersonPublicKey};

fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
        .expect("valid key")
}

fn slots(seeds: &[u8]) -> DeviceSlots {
    DeviceSlots::parse(seeds.iter().map(|seed| device(*seed)).collect()).expect("valid set")
}

fn person_key(seed: u8) -> PersonPublicKey {
    PersonPublicKey::parse(&MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes())
        .expect("valid X25519 key")
}

/// A device key whose 32 bytes are also a valid X25519 public key, and that
/// key: the literal copy LLR-kbhc43 refuses. Found by search, not assumed.
fn device_and_its_bytes_as_a_person_key() -> (DevicePublicKey, PersonPublicKey) {
    (1..=u8::MAX)
        .find_map(|seed| {
            let candidate = device(seed);
            PersonPublicKey::parse(candidate.as_bytes())
                .ok()
                .map(|key| (candidate, key))
        })
        .expect("some device key's bytes are a valid X25519 key")
}

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
```

2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test group_key`
   → `error[E0432]: unresolved import `person::group_key::check_group_key``.

3. `person/src/group_key.rs`, whole file:

```rust
//! The group-key rules a Person definition, and later a Member record, must
//! satisfy (docs/adr/2026-10-04-group-key-rules.md). LLR-sjkmr6, LLR-kbhc43.

use crate::error::IdentityError;
use crate::person_key::PersonPublicKey;
use crate::slots::DeviceSlots;

/// Checks a group key against the device slots it is held with: with no
/// device the key must be absent (`KeyWithoutDevice`); with one or more it
/// must be present (`MissingPersonKey`) and its 32 bytes must differ from
/// every device key's 32 bytes (`PersonKeyIsDeviceKey`).
///
/// The comparison is of raw bytes only (owner, 2026-10-06): it does not catch
/// a device's ed25519 key reused through its X25519 image, nor one X25519 key
/// under another of its encodings. LLR-sjkmr6, LLR-kbhc43.
pub fn check_group_key(
    group_key: Option<&PersonPublicKey>,
    devices: &DeviceSlots,
) -> Result<(), IdentityError> {
    match (group_key, devices.device_count()) {
        (None, 0) => Ok(()),
        (Some(_), 0) => Err(IdentityError::KeyWithoutDevice),
        (None, _) => Err(IdentityError::MissingPersonKey),
        (Some(key), _) => {
            if devices
                .devices()
                .iter()
                .any(|device| device.as_bytes() == key.as_bytes())
            {
                Err(IdentityError::PersonKeyIsDeviceKey)
            } else {
                Ok(())
            }
        }
    }
}
```

4. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test group_key`
   → `test result: ok. 4 passed; 0 failed`.
   `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy --offline -p person --all-targets -- -D warnings`
   → `Finished`, no warning.

5. Commit: `git add person/src/group_key.rs person/tests/group_key.rs`, then
   `git -c commit.gpgsign=false commit -m "feat(person): check_group_key, the group-key rules as one reusable function"`.

---

### T3 — `PersonDeviceHasher`

**Files touched:** `person/src/device_hasher.rs`, `person/tests/person_device_trie.rs` (new)
**Parallel:** yes — after T1, alongside T2, T4, T5

Trace: LLR-edn55h (the device sub-trie half).

1. Failing test `person/tests/person_device_trie.rs`:

```rust
//! PersonDeviceHasher: the Person device sub-trie's own domains and sentinel.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use ed25519_dalek::SigningKey;
use person::device_hasher::{PersonDeviceHasher, PERSON_DEVICE_LEAF_KEY, PERSON_DEVICE_NODE_KEY};
use person::{compute_device_root, DevicePublicKey, DeviceSlots, DeviceTrieHasher, NodeHash};

fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
        .expect("valid key")
}

fn slots(seeds: &[u8]) -> DeviceSlots {
    DeviceSlots::parse(seeds.iter().map(|seed| device(*seed)).collect()).expect("valid set")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// org-members' device domain keys and sentinel, as in tests/device_trie.rs.
struct OrgDomains;
impl DeviceTrieHasher for OrgDomains {
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

/// verifies: LLR-edn55h
#[test]
fn the_domain_keys_and_sentinel_are_the_documented_bytes() {
    assert_eq!(PERSON_DEVICE_LEAF_KEY, b"person::device-leaf_____________");
    assert_eq!(PERSON_DEVICE_NODE_KEY, b"person::device-node_____________");
    assert_eq!(
        PersonDeviceHasher::DEVICE_EMPTY_SENTINEL,
        b"EMPTY_SENTINEL_PERSON_DEVICE_V1"
    );
    assert_eq!(
        PersonDeviceHasher::hash_device_leaf(b"leaf"),
        NodeHash::new(blake3::keyed_hash(PERSON_DEVICE_LEAF_KEY, b"leaf").into())
    );
    let left = NodeHash::new([1; 32]);
    let right = NodeHash::new([2; 32]);
    let mut concatenated = [1u8; 64];
    concatenated[32..].copy_from_slice(&[2; 32]);
    assert_eq!(
        PersonDeviceHasher::hash_device_node(&left, &right),
        NodeHash::new(blake3::keyed_hash(PERSON_DEVICE_NODE_KEY, &concatenated).into())
    );
}

/// verifies: LLR-edn55h
#[test]
fn the_root_is_the_depth_two_tree_under_the_person_domains() {
    let set = slots(&[2, 1]);
    let held = set.devices();
    let empty = PersonDeviceHasher::hash_device_leaf(PersonDeviceHasher::DEVICE_EMPTY_SENTINEL);
    let leaf0 = PersonDeviceHasher::hash_device_leaf(held[0].as_bytes());
    let leaf1 = PersonDeviceHasher::hash_device_leaf(held[1].as_bytes());
    let expected = PersonDeviceHasher::hash_device_node(
        &PersonDeviceHasher::hash_device_node(&leaf0, &leaf1),
        &PersonDeviceHasher::hash_device_node(&empty, &empty),
    );
    assert_eq!(compute_device_root::<PersonDeviceHasher>(&set), expected);
}

/// Pinned values. Computed 2026-10-06 by this test's first run against the
/// implementation in this plan (docs/plans/2026-10-06-person-definition.md).
/// verifies: LLR-edn55h
#[test]
fn the_person_device_root_is_pinned() {
    assert_eq!(
        hex(compute_device_root::<PersonDeviceHasher>(&slots(&[])).as_bytes()),
        "db36a2843de361cd8fa691c95f305e206fbcdc666e0f1d1a5e494610df4421a5"
    );
    assert_eq!(
        hex(compute_device_root::<PersonDeviceHasher>(&slots(&[1, 2])).as_bytes()),
        "6b8a4eea74c7daf62a0e6b87112f24ad1037bbfdc5e8607eaa2e217bc8e9d4a5"
    );
}

/// The same device set gives another root than org-members' domains give it,
/// the empty set included: neither the domain keys nor the sentinel are shared.
/// verifies: LLR-edn55h
#[test]
fn no_set_shares_its_root_with_org_members() {
    for seeds in [&[][..], &[1], &[1, 2], &[1, 2, 3, 4]] {
        let set = slots(seeds);
        assert_ne!(
            compute_device_root::<PersonDeviceHasher>(&set),
            compute_device_root::<OrgDomains>(&set),
            "{seeds:?}"
        );
    }
    assert_ne!(
        PersonDeviceHasher::DEVICE_EMPTY_SENTINEL,
        OrgDomains::DEVICE_EMPTY_SENTINEL
    );
}

/// Every slot set gives a root distinct from every other, the boundary sets
/// (no device, `MAX_DEVICES` devices) included.
/// verifies: LLR-edn55h
#[test]
fn different_sets_give_different_roots() {
    let sets = [
        slots(&[]),
        slots(&[1]),
        slots(&[2]),
        slots(&[1, 2]),
        slots(&[1, 2, 3]),
        slots(&[1, 2, 3, 4]),
    ];
    for (index, set) in sets.iter().enumerate() {
        for other in &sets[index + 1..] {
            assert_ne!(
                compute_device_root::<PersonDeviceHasher>(set),
                compute_device_root::<PersonDeviceHasher>(other)
            );
        }
    }
}
```

2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test person_device_trie`
   → `error[E0432]: unresolved imports `person::device_hasher::PersonDeviceHasher`, `person::device_hasher::PERSON_DEVICE_LEAF_KEY`, `person::device_hasher::PERSON_DEVICE_NODE_KEY``.

3. `person/src/device_hasher.rs`, whole file:

```rust
//! The Person device sub-trie's hashing: this unit's own domain keys and
//! empty-slot sentinel, none of them org-members'. LLR-edn55h.

use crate::device_trie::{DeviceTrieHasher, NodeHash};

/// `person::device-leaf`, padded with `_` to 32 bytes.
pub const PERSON_DEVICE_LEAF_KEY: &[u8; 32] = b"person::device-leaf_____________";

/// `person::device-node`, padded with `_` to 32 bytes.
pub const PERSON_DEVICE_NODE_KEY: &[u8; 32] = b"person::device-node_____________";

/// The Person device sub-trie's hasher: blake3 keyed by the leaf key over a
/// device key's 32 bytes (or the sentinel), and by the node key over two
/// child hashes concatenated. LLR-edn55h.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonDeviceHasher;

impl DeviceTrieHasher for PersonDeviceHasher {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_PERSON_DEVICE_V1";

    fn hash_device_leaf(data: &[u8]) -> NodeHash {
        NodeHash::new(blake3::keyed_hash(PERSON_DEVICE_LEAF_KEY, data).into())
    }

    fn hash_device_node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        let mut hasher = blake3::Hasher::new_keyed(PERSON_DEVICE_NODE_KEY);
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        NodeHash::new(hasher.finalize().into())
    }
}
```

4. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test person_device_trie`
   → `test result: ok. 5 passed; 0 failed`. Clippy as in T2 → clean.

5. Commit: `git add person/src/device_hasher.rs person/tests/person_device_trie.rs`, then
   `git -c commit.gpgsign=false commit -m "feat(person): PersonDeviceHasher, the Person device sub-trie's own domains and sentinel"`.

---

### T4 — `EncodingVersion`

**Files touched:** `person/src/encoding.rs`, `person/tests/encoding_version.rs` (new)
**Parallel:** yes — after T1, alongside T2, T3, T5

Trace: LLR-rde6tk, LLR-3n3kxx.

1. Failing test `person/tests/encoding_version.rs`:

```rust
//! EncodingVersion: the closed set of Person hash encodings this unit implements.

use person::encoding::EncodingVersion;
use person::IdentityError;

/// verifies: LLR-rde6tk
#[test]
fn version_one_is_v1_and_back() {
    assert_eq!(EncodingVersion::try_from(1u16), Ok(EncodingVersion::V1));
    assert_eq!(u16::from(EncodingVersion::V1), 1);
}

/// verifies: LLR-rde6tk, LLR-3n3kxx
#[test]
fn every_other_version_is_refused_and_named() {
    for value in [0u16, 2, 3, 255, 256, u16::MAX] {
        assert_eq!(
            EncodingVersion::try_from(value),
            Err(IdentityError::UnsupportedEncodingVersion(value)),
            "version {value}"
        );
    }
    assert_eq!(
        IdentityError::UnsupportedEncodingVersion(2).to_string(),
        "unsupported encoding version 2"
    );
}
```

2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test encoding_version`
   → `error[E0432]: unresolved import `person::encoding::EncodingVersion``.

3. `person/src/encoding.rs`, whole file:

```rust
//! The Person hash's encoding versions. LLR-rde6tk.

use crate::error::IdentityError;

/// An encoding version this unit implements. Closed: a version is added only
/// with its encoding and its own hash domain key (LLR-4ebtn4). LLR-rde6tk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodingVersion {
    /// Name, surname, optional PersonPublicKey (X25519), the device root over
    /// ed25519 DevicePublicKeys. LLR-edn55h.
    V1,
}

impl TryFrom<u16> for EncodingVersion {
    type Error = IdentityError;

    /// 1 is `V1`; every other value is refused before any encoding or hashing.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::V1),
            other => Err(IdentityError::UnsupportedEncodingVersion(other)),
        }
    }
}

impl From<EncodingVersion> for u16 {
    fn from(version: EncodingVersion) -> Self {
        match version {
            EncodingVersion::V1 => 1,
        }
    }
}
```

4. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test encoding_version`
   → `test result: ok. 2 passed; 0 failed`. Clippy → clean.

5. Commit: `git add person/src/encoding.rs person/tests/encoding_version.rs`, then
   `git -c commit.gpgsign=false commit -m "feat(person): EncodingVersion, refusing every version but 1"`.

---

### T5 — Ledger amendments: sentinel, signatures, SOUP as built

**Files touched:** `person/docs/architecture/DRAFT-worktree-person-requirements-person-definition.md`, `person/docs/architecture/soup.md`
**Parallel:** yes — after T1 (the SOUP row describes T1's `Cargo.toml`), alongside T2, T3, T4

Trace: none minted. Every item edited here is an existing draft item edited
in place in the file that defines it; no new ID.

The replacement paragraphs below are shown indented by two spaces: a
definition form at column one anywhere in the tree, this plan included, is a
second definition to `check-ids.sh` (DUPLICATE-ID). Write them into the
architecture draft flush left, without the indent.

1. In the architecture draft, replace the whole **LLR-sjkmr6** paragraph
   (from `**LLR-sjkmr6**:` to `satisfies: REQ-ht3x78`) with:

```markdown
  **LLR-sjkmr6**: `Person::new` refuses, with `KeyWithoutDevice`, a definition
  whose device slots are empty and whose `PersonPublicKey` is `Some`. It does so
  by calling `check_group_key` (LLR-kbhc43).
  satisfies: REQ-ht3x78
```

2. Replace the whole **LLR-kbhc43** paragraph (from `**LLR-kbhc43**:` to
   `satisfies: REQ-7qgx2q`) with:

```markdown
  **LLR-kbhc43**: `Person::new` refuses a definition with one or more device keys
  whose `PersonPublicKey` is `None` (`MissingPersonKey`) or whose 32 key bytes
  equal the 32 bytes of any of its `DevicePublicKey`s (`PersonKeyIsDeviceKey`).
  The comparison is of raw bytes only. It catches a literal copy of a device
  key's bytes into the PersonPublicKey field; it does not catch a device's
  ed25519 key pair reused through its X25519 (birational) image, nor one X25519
  key presented under another of its up to 8 encodings with a small-order
  component mixed in. Both are accepted residual risk (owner, 2026-10-06; the
  risk file's assessment of REQ-7qgx2q). This rule and LLR-sjkmr6's are one
  public function, `check_group_key(group_key: Option<&PersonPublicKey>, devices:
  &DeviceSlots) -> Result<(), IdentityError>` in `person/src/group_key.rs`, which
  `Person::new` calls and which org-members can call for the member-as-a-group
  key (change 3 of `docs/plans/2026-10-04-person-sequencing.md`).
  satisfies: REQ-7qgx2q
```

3. Replace the whole **LLR-63pkfc** paragraph (from `**LLR-63pkfc**:` to
   `satisfies: REQ-7ymek3`) with:

```markdown
  **LLR-63pkfc**: `Person`'s `Deserialize` implementation, compiled only with
  the `serde` feature (`cfg(feature = "serde")`), decodes the four fields and
  passes them to `Person::new`; `Name` and `Surname` decode through their
  `parse` (LLR-ayu93n), the device slots through LLR-sjrh7z, each
  `DevicePublicKey` through `DevicePublicKey::parse` (LLR-7guspr), and the
  `PersonPublicKey` through `PersonPublicKey::parse` (LLR-vs7etb), which accepts
  a canonical u-coordinate of a point on the quadratic twist (owner,
  2026-10-05). No `Person` value is produced from bytes by any other route, and
  a refusal surfaces through the deserializer's error with the message of the
  `IdentityError` that `Person::new` or the field's `parse` returned; a missing
  or unknown field is refused. With the same feature a `Person` serialises as a
  struct of the four fields in this order — `name`, `surname`, `person_key` (an
  option), `devices` — each in its own type's form (LLR-5za6mp).
  satisfies: REQ-7ymek3
```

4. Replace the whole **LLR-edn55h** paragraph (from `**LLR-edn55h**:` to
   `satisfies: REQ-9m5pq2`) with (this names the sentinel):

```markdown
  **LLR-edn55h**: The `V1` encoding is, in order: the NFC name's byte length as a
  4-byte little-endian `u32`, then its bytes; the same for the surname; one byte,
  `0x00` when the `PersonPublicKey` is absent and `0x01` when present, followed in
  the second case by its 32 bytes; then the 32-byte device root
  `compute_device_root::<PersonDeviceHasher>(&devices)` (LLR-6ezhw7).
  `PersonDeviceHasher` is this unit's own, Person-specific implementation of
  `DeviceTrieHasher` (LLR-4vsm8d): `hash_device_leaf` is `blake3::keyed_hash`
  under the leaf domain key `person::device-leaf` and `hash_device_node` is
  `blake3::keyed_hash`, under the node domain key `person::device-node`, of the
  two children's 32 bytes concatenated, each key padded with `_` to 32 bytes
  (`person::device-leaf_____________`, `person::device-node_____________`); its
  `DEVICE_EMPTY_SENTINEL` is the 31 ASCII bytes `EMPTY_SENTINEL_PERSON_DEVICE_V1`.
  Neither its domain keys nor its sentinel are org-members'
  (`org-members::device-leaf________`, `org-members::device-node________`,
  `EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1`). satisfies: REQ-9m5pq2
```

5. Replace the whole **LLR-4ebtn4** paragraph (from `**LLR-4ebtn4**:` to
   `satisfies: REQ-9m5pq2, REQ-wg7z4s`) with:

```markdown
  **LLR-4ebtn4**: The `V1` hash is `blake3::keyed_hash` of the `V1` encoding under
  the 32-byte key `person::definition::v1`, padded with `_` to 32 bytes
  (`person::definition::v1__________`). A later version gets its own key, so no
  two versions' hashes share a domain. `person_hash(person, version)` takes the
  version as the `u16` published beside the hash, parses it with
  `EncodingVersion::try_from` (LLR-rde6tk), and returns a `PersonHash`: any 32
  bytes, unvalidated, built by `PersonHash::new` and `From<[u8; 32]>` and read
  by `as_bytes`, whose `Debug` form is `PersonHash(` and the first four bytes in
  lowercase hex and `..)`, and which with the `serde` feature serialises as its
  32 bytes. satisfies: REQ-9m5pq2, REQ-wg7z4s
```

6. Replace the first line of the **LLR-tf45kx** paragraph — the line that
   begins with the bold ID and ends with the words `exactly when` — with these
   two lines (the rest of the paragraph is unchanged):

```markdown
  **LLR-tf45kx**: `verify(person, version, hash)` — `version` the `u16` encoding
  version, `hash` a `PersonHash` — returns `Ok(true)` exactly when
```

7. In `person/docs/architecture/soup.md`:

   a. Replace the opening of the line that begins
   `**As built at the merge (2026-10-05, after the torsion-free ruling)**`
   so that it reads:

```markdown
**As built at the merge (2026-10-05, after the torsion-free ruling; the `blake3` row and the dev-dependencies below as built by the Person definition change, 2026-10-06)**, diffed against `person/Cargo.toml` and the workspace
```

   b. After the `thiserror` row of the table, add the row:

```markdown
| `blake3` | 1.8.7 | The Person hash, `blake3::keyed_hash` of the `V1` encoding under `person::definition::v1__________` (SDD-v32mqh; LLR-4ebtn4), and the Person device sub-trie through `PersonDeviceHasher`, keyed under `person::device-leaf_____________` and `person::device-node_____________` (SDD-v32mqh; LLR-edn55h, over SDD-7r833z's LLR-6ezhw7). Each use has its own domain key, so no two share a hash domain, and none is org-members'. | REQ-9m5pq2, REQ-7n4g8b, REQ-wg7z4s (SDD-v32mqh; LLR-edn55h, LLR-4ebtn4, LLR-tf45kx) | **Safety-relevant.** The hash's collision and second-preimage resistance is what makes a match against a stored hash mean "this definition": a collaborator checks a definition against the hash published for a person (REQ-7n4g8b), and a different definition with the same hash would pass that check. Residual risk: that of the hash function itself, which this unit cannot reduce; it is the residual risk the risk file's assessment of REQ-9m5pq2 defers to this SOUP item. Outputs are pinned by `the_person_device_root_is_pinned` and `the_v1_hash_is_pinned`, and recomputed from `blake3::keyed_hash` directly by `the_root_is_the_depth_two_tree_under_the_person_domains` and `the_v1_hash_is_blake3_keyed_by_the_v1_domain_key`, so an upgrade that changed an output fails them. `default-features = false`. |
```

   c. In the paragraph under the table, replace these three lines:

```markdown
the mixed-order points whose u-coordinates the tests check. `blake3` 1.8.7 and
`postcard` 1.1.3 are dev-dependencies too: `person` ships no hasher
implementation and no wire format of its own. `proptest` 1.11.0 is a
```

   with:

```markdown
the mixed-order points whose u-coordinates the tests check. `postcard` 1.1.3
is a dev-dependency too: `person` ships no wire format of its own. `blake3`,
a dev-dependency until the Person definition, is now a normal dependency (row
above); the device-trie tests also name it to build org-members' and a test
domain's hashers. `serde_json` 1.0.151 is a dev-dependency because it keeps a
custom decode error's message, which the Person decoding tests check (postcard
maps every custom error to `SerdeDeCustom`). `bolero` 0.13.4 is a
dev-dependency that drives the `fuzz_person_decode` target, as it drives
org-node's and on-chain-client's fuzz targets. `proptest` 1.11.0 is a
```

   d. Delete the whole section `## Planned by this change (Person definition)`,
   from its heading to the end of the file.

8. Diff the SOUP inventory against the manifest and the lockfile:
   `CARGO_HOME=/tmp/cargo_home_fuzz cargo tree --offline -p person -e normal,dev --depth 1`
   → expected:

```text
person v0.1.0 (<worktree>/person)
├── blake3 v1.8.7
├── ed25519-dalek v2.2.0
├── serde v1.0.229
├── thiserror v2.0.20
└── unicode-normalization v0.1.25
[dev-dependencies]
├── bolero v0.13.4
├── curve25519-dalek v4.1.3
├── postcard v1.1.3
├── proptest v1.11.0
└── serde_json v1.0.151
```

   Every normal dependency has a table row at that exact version (and
   `curve25519-dalek` 4.1.3 its runtime row); every dev-dependency is named
   with its version in the prose. Any mismatch is fixed in `soup.md` before the
   commit.

9. `GR_CONFIG=person/.guardrails/config.yaml .guardrails/scripts/check-ids.sh --allow-draft-files`
   → exit 0, no output.

10. Commit: `git add person/docs/architecture/DRAFT-worktree-person-requirements-person-definition.md person/docs/architecture/soup.md`, then
    `git -c commit.gpgsign=false commit -m "docs(person): name the device sentinel, the hash and verify signatures; blake3 as built in SOUP"`.

---

### T6 — `Person` and `Person::new`

**Files touched:** `person/src/definition.rs`, `person/tests/definition.rs` (new)
**Parallel:** no (serial, after T2)

Trace: LLR-4ku56h, LLR-sjkmr6, LLR-kbhc43, LLR-3n3kxx (and LLR-fbqs2r, the
redacted names, through `Person`'s `Debug`).

1. Failing test `person/tests/definition.rs`:

```rust
//! Person::new: the only way to build a Person definition.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::SigningKey;
use person::definition::Person;
use person::group_key::check_group_key;
use person::{
    DevicePublicKey, DeviceSlots, IdentityError, Name, PersonPublicKey, Surname, MAX_DEVICES,
};

fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
        .expect("valid key")
}

fn slots(seeds: &[u8]) -> DeviceSlots {
    DeviceSlots::parse(seeds.iter().map(|seed| device(*seed)).collect()).expect("valid set")
}

fn person_key(seed: u8) -> PersonPublicKey {
    PersonPublicKey::parse(&MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes())
        .expect("valid X25519 key")
}

fn name() -> Name {
    Name::parse("Alice").expect("valid name")
}

fn surname() -> Surname {
    Surname::parse("Example").expect("valid surname")
}

/// A device key whose 32 bytes are also a valid X25519 public key, and that key.
fn device_and_its_bytes_as_a_person_key() -> (DevicePublicKey, PersonPublicKey) {
    (1..=u8::MAX)
        .find_map(|seed| {
            let candidate = device(seed);
            PersonPublicKey::parse(candidate.as_bytes())
                .ok()
                .map(|key| (candidate, key))
        })
        .expect("some device key's bytes are a valid X25519 key")
}

/// verifies: LLR-4ku56h
#[test]
fn a_person_holds_up_to_max_devices_sorted() {
    assert_eq!(MAX_DEVICES, 4);
    let given = vec![device(4), device(1), device(3), device(2)];
    let devices = DeviceSlots::parse(given.clone()).expect("four");
    let person = Person::new(name(), surname(), Some(person_key(9)), devices.clone())
        .expect("valid definition");
    let mut sorted = given;
    sorted.sort();
    assert_eq!(person.devices().devices(), sorted.as_slice());
    assert_eq!(person.devices(), &devices);
    assert_eq!(person.name(), &name());
    assert_eq!(person.surname(), &surname());
    assert_eq!(person.person_key(), Some(&person_key(9)));
}

/// A fifth key or a key given twice is refused while the slots are built, so
/// no Person holding them can exist.
/// verifies: LLR-4ku56h
#[test]
fn over_the_bound_or_a_duplicate_is_refused_before_any_person_exists() {
    let five: Vec<DevicePublicKey> = (1..=5).map(device).collect();
    assert_eq!(
        DeviceSlots::parse(five),
        Err(IdentityError::DeviceSlotsFull)
    );
    assert_eq!(
        DeviceSlots::parse(vec![device(1), device(1)]),
        Err(IdentityError::DuplicateDevice)
    );
    let full = slots(&[1, 2, 3, 4]);
    assert_eq!(
        full.add_device(device(5)),
        Err(IdentityError::DeviceSlotsFull)
    );
}

/// verifies: LLR-sjkmr6, LLR-3n3kxx
#[test]
fn no_devices_and_no_key_is_accepted_and_a_key_without_devices_is_refused() {
    let revoked = Person::new(name(), surname(), None, slots(&[])).expect("no device, no key");
    assert_eq!(revoked.person_key(), None);
    assert_eq!(revoked.devices().device_count(), 0);
    assert_eq!(
        Person::new(name(), surname(), Some(person_key(9)), slots(&[])),
        Err(IdentityError::KeyWithoutDevice)
    );
}

/// verifies: LLR-kbhc43, LLR-3n3kxx
#[test]
fn devices_without_a_key_are_refused() {
    for seeds in [&[1][..], &[1, 2, 3, 4]] {
        assert_eq!(
            Person::new(name(), surname(), None, slots(seeds)),
            Err(IdentityError::MissingPersonKey),
            "{seeds:?}"
        );
    }
}

/// verifies: LLR-kbhc43, LLR-3n3kxx
#[test]
fn a_key_copied_from_a_device_key_is_refused() {
    let (copied, key) = device_and_its_bytes_as_a_person_key();
    let devices = DeviceSlots::parse(vec![copied, device(101)]).expect("two");
    assert_eq!(
        Person::new(name(), surname(), Some(key), devices),
        Err(IdentityError::PersonKeyIsDeviceKey)
    );
    let others = slots(&[101, 102]);
    assert!(Person::new(name(), surname(), Some(key), others).is_ok());
}

/// Person::new accepts exactly what check_group_key accepts, and reports its
/// error, over every combination of key and device count.
/// verifies: LLR-sjkmr6, LLR-kbhc43
#[test]
fn person_new_applies_check_group_key() {
    let (copied, copied_key) = device_and_its_bytes_as_a_person_key();
    let keys = [None, Some(person_key(9)), Some(copied_key)];
    let device_sets = [
        slots(&[]),
        slots(&[101]),
        DeviceSlots::parse(vec![copied]).expect("one"),
        slots(&[101, 102, 103, 104]),
    ];
    for key in keys {
        for devices in &device_sets {
            let expected = check_group_key(key.as_ref(), devices);
            let built = Person::new(name(), surname(), key, devices.clone());
            assert_eq!(built.map(|_| ()), expected, "{key:?} {devices:?}");
        }
    }
}

/// A Person's Debug form carries its Name and Surname in their redacted form.
/// verifies: LLR-fbqs2r
#[test]
fn debug_redacts_the_names() {
    let person =
        Person::new(name(), surname(), Some(person_key(9)), slots(&[1])).expect("valid definition");
    let debug = format!("{person:?}");
    assert!(
        !debug.contains("Alice") && !debug.contains("Example"),
        "{debug}"
    );
    assert!(debug.contains("Name([REDACTED])"), "{debug}");
}
```

2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test definition`
   → `error[E0432]: unresolved import `person::definition::Person``.

3. `person/src/definition.rs`, whole file:

```rust
//! The Person definition: an individual's own record. SDD-4r2x79.

use crate::error::IdentityError;
use crate::group_key::check_group_key;
use crate::name::{Name, Surname};
use crate::person_key::PersonPublicKey;
use crate::slots::DeviceSlots;

/// A validated Person definition: a name, a surname, an optional
/// PersonPublicKey and at most `MAX_DEVICES` DevicePublicKeys held as device
/// slots (LLR-4ku56h). `Person::new` is the only way to build one, decoding
/// included (LLR-63pkfc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    name: Name,
    surname: Surname,
    person_key: Option<PersonPublicKey>,
    devices: DeviceSlots,
}

impl Person {
    /// Refuses a definition whose PersonPublicKey breaks the group-key rules
    /// against its device slots (`check_group_key`). LLR-sjkmr6, LLR-kbhc43.
    pub fn new(
        name: Name,
        surname: Surname,
        person_key: Option<PersonPublicKey>,
        devices: DeviceSlots,
    ) -> Result<Self, IdentityError> {
        check_group_key(person_key.as_ref(), &devices)?;
        Ok(Self {
            name,
            surname,
            person_key,
            devices,
        })
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn surname(&self) -> &Surname {
        &self.surname
    }

    pub fn person_key(&self) -> Option<&PersonPublicKey> {
        self.person_key.as_ref()
    }

    pub fn devices(&self) -> &DeviceSlots {
        &self.devices
    }
}
```

4. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test definition`
   → `test result: ok. 7 passed; 0 failed`. Clippy → clean.

5. Commit: `git add person/src/definition.rs person/tests/definition.rs`, then
   `git -c commit.gpgsign=false commit -m "feat(person): Person and Person::new, through check_group_key"`.

---

### T7 — `Person` serde, decoding through `Person::new`

**Files touched:** `person/src/definition_serde.rs`, `person/tests/definition_decode.rs` (new)
**Parallel:** yes — after T6, alongside T8, T9

Trace: LLR-63pkfc, LLR-4ku56h (decoding the slots through LLR-sjrh7z),
LLR-sjkmr6, LLR-kbhc43, LLR-3n3kxx.

1. Failing test `person/tests/definition_decode.rs`:

```rust
//! Decoding a Person: each field through its own decoder, then Person::new.
//! serde_json keeps a custom error's message, so the message tests decode
//! JSON; postcard, which maps every custom error to `SerdeDeCustom`, checks
//! the binary route.

#![cfg(feature = "serde")]
// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::SigningKey;
use person::definition::Person;
use person::{DevicePublicKey, DeviceSlots, IdentityError, Name, PersonPublicKey, Surname};
use serde_json::{json, Value};

fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
        .expect("valid key")
}

fn person_key(seed: u8) -> PersonPublicKey {
    PersonPublicKey::parse(&MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes())
        .expect("valid X25519 key")
}

fn sorted_device_bytes(seeds: &[u8]) -> Vec<[u8; 32]> {
    let mut keys: Vec<[u8; 32]> = seeds.iter().map(|seed| *device(*seed).as_bytes()).collect();
    keys.sort();
    keys
}

/// A JSON Person with these fields, keys as arrays of 32 numbers.
fn person_json(name: &str, key: Option<[u8; 32]>, devices: &[[u8; 32]]) -> Value {
    json!({
        "name": name,
        "surname": "Example",
        "person_key": key,
        "devices": devices,
    })
}

fn decode_error(value: Value) -> String {
    serde_json::from_value::<Person>(value)
        .expect_err("refused")
        .to_string()
}

fn alice() -> Person {
    let devices = DeviceSlots::parse(vec![device(2), device(1)]).expect("two");
    Person::new(
        Name::parse("Alice").expect("valid name"),
        Surname::parse("Example").expect("valid surname"),
        Some(person_key(9)),
        devices,
    )
    .expect("valid definition")
}

/// verifies: LLR-63pkfc
#[test]
fn a_person_round_trips_through_both_formats() {
    let person = alice();
    let bytes = postcard::to_allocvec(&person).expect("encode");
    assert_eq!(
        postcard::from_bytes::<Person>(&bytes).expect("decode"),
        person
    );
    let text = serde_json::to_string(&person).expect("encode");
    assert_eq!(
        serde_json::from_str::<Person>(&text).expect("decode"),
        person
    );
    let revoked = Person::new(
        Name::parse("Alice").expect("valid name"),
        Surname::parse("Example").expect("valid surname"),
        None,
        DeviceSlots::parse(vec![]).expect("empty"),
    )
    .expect("no device, no key");
    let bytes = postcard::to_allocvec(&revoked).expect("encode");
    assert_eq!(
        postcard::from_bytes::<Person>(&bytes).expect("decode"),
        revoked
    );
}

/// The postcard form is the four fields in order: name, surname, the key as
/// an option, the slots as a sequence.
/// verifies: LLR-63pkfc
#[test]
fn the_binary_form_is_the_four_fields_in_order() {
    let person = alice();
    let mut expected = postcard::to_allocvec(person.name()).expect("name");
    expected.extend(postcard::to_allocvec(person.surname()).expect("surname"));
    expected.extend(postcard::to_allocvec(&person.person_key()).expect("key"));
    expected.extend(postcard::to_allocvec(person.devices()).expect("devices"));
    assert_eq!(postcard::to_allocvec(&person).expect("encode"), expected);
}

/// A decoded name is stored in NFC, as `Name::parse` stores it.
/// verifies: LLR-63pkfc
#[test]
fn a_decoded_name_is_normalised() {
    let value = person_json(
        "Jose\u{0301}",
        Some(*person_key(9).as_bytes()),
        &sorted_device_bytes(&[1]),
    );
    let person = serde_json::from_value::<Person>(value).expect("valid");
    assert_eq!(person.name().as_str(), "Jos\u{00e9}");
}

/// Each Person::new rule refuses through the decoder, with its message.
/// verifies: LLR-63pkfc, LLR-sjkmr6, LLR-kbhc43, LLR-3n3kxx
#[test]
fn a_definition_breaking_a_person_rule_is_refused_with_its_message() {
    let key = *person_key(9).as_bytes();
    assert_eq!(
        decode_error(person_json("Alice", Some(key), &[])),
        IdentityError::KeyWithoutDevice.to_string()
    );
    assert_eq!(
        decode_error(person_json("Alice", None, &sorted_device_bytes(&[1]))),
        IdentityError::MissingPersonKey.to_string()
    );
    let copied = (1..=u8::MAX)
        .map(|seed| *device(seed).as_bytes())
        .find(|bytes| PersonPublicKey::parse(bytes).is_ok())
        .expect("some device key's bytes are a valid X25519 key");
    assert_eq!(
        decode_error(person_json("Alice", Some(copied), &[copied])),
        IdentityError::PersonKeyIsDeviceKey.to_string()
    );
}

/// Each field's own rule refuses through that field's decoder, with its message.
/// verifies: LLR-63pkfc, LLR-4ku56h
#[test]
fn a_field_breaking_its_own_rule_is_refused_with_its_message() {
    let key = Some(*person_key(9).as_bytes());
    let one = sorted_device_bytes(&[1]);
    assert_eq!(
        decode_error(person_json(&"a".repeat(129), key, &one)),
        "field too long: name exceeds 128 bytes after NFC normalization"
    );
    let mut identity = [0u8; 32];
    identity[0] = 1;
    assert_eq!(
        decode_error(person_json("Alice", key, &[identity])),
        "invalid device public key"
    );
    assert_eq!(
        decode_error(person_json("Alice", Some([0u8; 32]), &one)),
        "invalid person public key"
    );
    assert_eq!(
        decode_error(person_json(
            "Alice",
            key,
            &sorted_device_bytes(&[1, 2, 3, 4, 5])
        )),
        "device slots exceed MAX_DEVICES"
    );
    let mut reversed = sorted_device_bytes(&[1, 2]);
    reversed.reverse();
    assert_eq!(
        decode_error(person_json("Alice", key, &reversed)),
        "device slots must be strictly increasing (sorted, no duplicates)"
    );
}

/// verifies: LLR-63pkfc
#[test]
fn a_missing_or_unknown_field_is_refused() {
    let mut missing = person_json("Alice", None, &[]);
    missing
        .as_object_mut()
        .expect("an object")
        .remove("devices");
    assert!(serde_json::from_value::<Person>(missing).is_err());
    let mut extra = person_json("Alice", None, &[]);
    extra
        .as_object_mut()
        .expect("an object")
        .insert("epoch".into(), json!(1));
    assert!(serde_json::from_value::<Person>(extra).is_err());
}

/// postcard bytes whose every field decodes, but which break a Person rule,
/// are refused: the only custom error such bytes can produce is Person::new's.
/// verifies: LLR-63pkfc, LLR-sjkmr6
#[test]
fn binary_bytes_breaking_a_person_rule_are_refused() {
    let mut bytes = postcard::to_allocvec(&Name::parse("Alice").expect("name")).expect("name");
    bytes.extend(
        postcard::to_allocvec(&Surname::parse("Example").expect("surname")).expect("surname"),
    );
    bytes.extend(postcard::to_allocvec(&Some(person_key(9))).expect("key"));
    bytes.extend(
        postcard::to_allocvec(&DeviceSlots::parse(vec![]).expect("empty")).expect("devices"),
    );
    assert_eq!(
        postcard::from_bytes::<Person>(&bytes),
        Err(postcard::Error::SerdeDeCustom)
    );
}

/// Encoding into a buffer too short for it fails, whichever field the buffer
/// runs out in (JSON's opening brace included), and never writes past it.
/// verifies: LLR-63pkfc
#[test]
fn encoding_into_a_short_buffer_fails_at_every_field() {
    let person = alice();
    let text = serde_json::to_vec(&person).expect("encode");
    for length in 0..text.len() {
        let mut buffer = vec![0u8; length];
        let mut writer: &mut [u8] = &mut buffer;
        assert!(
            serde_json::to_writer(&mut writer, &person).is_err(),
            "{length} of {} bytes",
            text.len()
        );
    }
    let full = postcard::to_allocvec(&person).expect("encode");
    for length in 0..full.len() {
        let mut buffer = vec![0u8; length];
        assert!(
            postcard::to_slice(&person, &mut buffer).is_err(),
            "{length} of {} bytes",
            full.len()
        );
    }
    let mut buffer = vec![0u8; full.len()];
    assert_eq!(
        postcard::to_slice(&person, &mut buffer)
            .expect("fits")
            .to_vec(),
        full
    );
}
```

2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test definition_decode`
   → fails to compile with `error[E0277]`: `Person` implements neither
   `serde::Serialize` nor `serde::Deserialize<'_>`.

3. `person/src/definition_serde.rs`, whole file:

```rust
//! `Person`'s serde implementations, compiled only with the `serde` feature.
//! Decoding goes through each field's own decoder and then `Person::new`.
//! LLR-63pkfc.

use serde::ser::SerializeStruct;

use crate::definition::Person;
use crate::name::{Name, Surname};
use crate::person_key::PersonPublicKey;
use crate::slots::DeviceSlots;

impl serde::Serialize for Person {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut fields = serializer.serialize_struct("Person", 4)?;
        fields.serialize_field("name", self.name())?;
        fields.serialize_field("surname", self.surname())?;
        fields.serialize_field("person_key", &self.person_key())?;
        fields.serialize_field("devices", self.devices())?;
        fields.end()
    }
}

/// The four fields as decoded, each through its own validating decoder, not
/// yet checked against one another.
#[derive(serde::Deserialize)]
#[serde(rename = "Person", deny_unknown_fields)]
struct PersonFields {
    name: Name,
    surname: Surname,
    person_key: Option<PersonPublicKey>,
    devices: DeviceSlots,
}

impl<'de> serde::Deserialize<'de> for Person {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = PersonFields::deserialize(deserializer)?;
        Person::new(
            fields.name,
            fields.surname,
            fields.person_key,
            fields.devices,
        )
        .map_err(serde::de::Error::custom)
    }
}
```

4. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test definition_decode`
   → `test result: ok. 8 passed; 0 failed`. Clippy → clean, and
   `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy --offline -p person --no-default-features --all-targets -- -D warnings`
   → clean (the module and the test file compile away without `serde`).

5. Commit: `git add person/src/definition_serde.rs person/tests/definition_decode.rs`, then
   `git -c commit.gpgsign=false commit -m "feat(person): Person serde, decoding through Person::new"`.

---

### T8 — `check_successor`

**Files touched:** `person/src/successor.rs`, `person/tests/successor.rs` (new)
**Parallel:** yes — after T6, alongside T7, T9

Trace: LLR-wqha8d, LLR-3n3kxx.

1. Failing test `person/tests/successor.rs`:

```rust
//! check_successor: whether one Person definition may follow another.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::SigningKey;
use person::definition::Person;
use person::successor::check_successor;
use person::{DevicePublicKey, DeviceSlots, IdentityError, Name, PersonPublicKey, Surname};

fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
        .expect("valid key")
}

fn person_key(seed: u8) -> PersonPublicKey {
    PersonPublicKey::parse(&MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes())
        .expect("valid X25519 key")
}

/// "Alice Example" holding `key` and the devices of `seeds`.
fn alice(key: Option<u8>, seeds: &[u8]) -> Person {
    let devices =
        DeviceSlots::parse(seeds.iter().map(|seed| device(*seed)).collect()).expect("valid set");
    Person::new(
        Name::parse("Alice").expect("valid name"),
        Surname::parse("Example").expect("valid surname"),
        key.map(person_key),
        devices,
    )
    .expect("valid definition")
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
```

2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test successor`
   → `error[E0432]: unresolved import `person::successor::check_successor``.

3. `person/src/successor.rs`, whole file:

```rust
//! Whether one Person definition may follow another. SDD-9hej83.

use crate::definition::Person;
use crate::error::IdentityError;

/// Refuses, with `PersonKeyNotRotated`, a `next` whose device keys differ
/// from `previous`'s while it holds the same PersonPublicKey (both present
/// and equal). Every other pair is accepted, a key change with unchanged
/// device keys included. Only the immediate predecessor is compared (owner,
/// 2026-10-04). LLR-wqha8d.
pub fn check_successor(previous: &Person, next: &Person) -> Result<(), IdentityError> {
    let devices_changed = previous.devices() != next.devices();
    let key_kept = previous.person_key().is_some() && previous.person_key() == next.person_key();
    if devices_changed && key_kept {
        Err(IdentityError::PersonKeyNotRotated)
    } else {
        Ok(())
    }
}
```

4. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test successor`
   → `test result: ok. 4 passed; 0 failed`. Clippy → clean.

5. Commit: `git add person/src/successor.rs person/tests/successor.rs`, then
   `git -c commit.gpgsign=false commit -m "feat(person): check_successor, refusing a device change that keeps the PersonPublicKey"`.

---

### T9 — `V1` encoding, `person_hash`, `verify`, `PersonHash`

**Files touched:** `person/src/hash.rs`, `person/tests/person_hash.rs` (new)
**Parallel:** yes — after T6, T3 and T4, alongside T7, T8

Trace: LLR-edn55h, LLR-4ebtn4, LLR-tf45kx, LLR-rde6tk, LLR-3n3kxx.

1. Failing test `person/tests/person_hash.rs`:

```rust
//! The Person hash: the V1 encoding, the keyed hash, the check of a hash.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::SigningKey;
use person::definition::Person;
use person::device_hasher::PersonDeviceHasher;
use person::encoding::EncodingVersion;
use person::hash::{encode, person_hash, verify, PersonHash, PERSON_DEFINITION_V1_KEY};
use person::{
    compute_device_root, DevicePublicKey, DeviceSlots, IdentityError, Name, PersonPublicKey,
    Surname, MAX_NAME_LEN, MAX_SURNAME_LEN,
};

fn device(seed: u8) -> DevicePublicKey {
    DevicePublicKey::try_from(SigningKey::from_bytes(&[seed; 32]).verifying_key())
        .expect("valid key")
}

fn slots(seeds: &[u8]) -> DeviceSlots {
    DeviceSlots::parse(seeds.iter().map(|seed| device(*seed)).collect()).expect("valid set")
}

fn person_key(seed: u8) -> PersonPublicKey {
    PersonPublicKey::parse(&MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes())
        .expect("valid X25519 key")
}

fn person(name: &str, surname: &str, key: Option<u8>, seeds: &[u8]) -> Person {
    Person::new(
        Name::parse(name).expect("valid name"),
        Surname::parse(surname).expect("valid surname"),
        key.map(person_key),
        slots(seeds),
    )
    .expect("valid definition")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hash_v1(definition: &Person) -> PersonHash {
    person_hash(definition, 1).expect("version 1 is implemented")
}

/// verifies: LLR-edn55h
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
        .extend_from_slice(compute_device_root::<PersonDeviceHasher>(&slots(&[1, 2])).as_bytes());
    assert_eq!(encode(&alice, EncodingVersion::V1), expected);
}

/// The boundary definitions: empty names, no key, no device; and names at
/// their byte bound, given decomposed and encoded in NFC.
/// verifies: LLR-edn55h
#[test]
fn the_v1_encoding_of_the_boundary_definitions() {
    let empty = person("", "", None, &[]);
    let mut expected = vec![0, 0, 0, 0, 0, 0, 0, 0, 0x00];
    expected.extend_from_slice(compute_device_root::<PersonDeviceHasher>(&slots(&[])).as_bytes());
    assert_eq!(encode(&empty, EncodingVersion::V1), expected);

    let longest = person(
        &"a".repeat(MAX_NAME_LEN),
        &"e\u{0301}".repeat(MAX_SURNAME_LEN / 2),
        Some(9),
        &[1, 2, 3, 4],
    );
    let encoding = encode(&longest, EncodingVersion::V1);
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
/// verifies: LLR-edn55h
#[test]
fn equal_device_sets_encode_equally_whatever_their_order() {
    assert_eq!(
        encode(
            &person("Alice", "Example", Some(9), &[3, 1, 2]),
            EncodingVersion::V1
        ),
        encode(
            &person("Alice", "Example", Some(9), &[1, 2, 3]),
            EncodingVersion::V1
        )
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
    let encoding = encode(&alice, EncodingVersion::V1);
    assert_eq!(
        hash_v1(&alice),
        PersonHash::new(blake3::keyed_hash(PERSON_DEFINITION_V1_KEY, &encoding).into())
    );
    assert_ne!(
        hash_v1(&alice).as_bytes(),
        blake3::hash(&encoding).as_bytes()
    );
}

/// Pinned values. Computed 2026-10-06 against the implementation in
/// docs/plans/2026-10-06-person-definition.md; a change here is a change of
/// the V1 encoding, which needs a new version (REQ-wg7z4s).
/// verifies: LLR-4ebtn4, LLR-edn55h
#[test]
fn the_v1_hash_is_pinned() {
    assert_eq!(
        hex(hash_v1(&person("Alice", "Example", Some(9), &[1, 2])).as_bytes()),
        "d6177f1cbed18504b63666dd45b5f54834667606fa73de40c2dbaf28a704e292"
    );
    assert_eq!(
        hex(hash_v1(&person("Alice", "Example", None, &[])).as_bytes()),
        "225bc8ac0117ae0b3c4aaaec7654b81b3aa45a8f27a58eab7f365dd6fc8183f0"
    );
}

/// A change to any one of the four fields changes the hash, and the length
/// prefixes keep a byte moved between name and surname from colliding.
/// verifies: LLR-4ebtn4, LLR-edn55h
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
        ("device_hasher.rs", include_str!("../src/device_hasher.rs")),
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
```

2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test person_hash`
   → `error[E0432]: unresolved imports `person::hash::encode`, `person::hash::person_hash`, `person::hash::verify`, `person::hash::PersonHash`, `person::hash::PERSON_DEFINITION_V1_KEY``.

3. `person/src/hash.rs`, whole file:

```rust
//! The Person hash: an encoding of a definition under an encoding version,
//! hashed with blake3 keyed per unit and version, and the check of a supplied
//! hash. Free functions over their arguments; nothing is kept between calls.
//! SDD-v32mqh.

use alloc::vec::Vec;
use core::fmt;

use crate::definition::Person;
use crate::device_hasher::PersonDeviceHasher;
use crate::device_trie::compute_device_root;
use crate::encoding::EncodingVersion;
use crate::error::IdentityError;
use crate::name::{MAX_NAME_LEN, MAX_SURNAME_LEN};

/// `person::definition::v1`, padded with `_` to 32 bytes. LLR-4ebtn4.
pub const PERSON_DEFINITION_V1_KEY: &[u8; 32] = b"person::definition::v1__________";

/// A name or surname length always fits the encoding's 4-byte length prefix,
/// so the `as u32` casts in `encode_v1` are lossless.
const _: () = assert!(MAX_NAME_LEN <= u32::MAX as usize && MAX_SURNAME_LEN <= u32::MAX as usize);

/// The 32-byte hash of a Person definition under one encoding version.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PersonHash([u8; 32]);

impl PersonHash {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for PersonHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for PersonHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::write_hex_prefix(formatter, "PersonHash", &self.0)
    }
}

/// The encoding of `person` under `version`. LLR-edn55h.
pub fn encode(person: &Person, version: EncodingVersion) -> Vec<u8> {
    match version {
        EncodingVersion::V1 => encode_v1(person),
    }
}

/// Name length (u32 LE) and NFC bytes; surname likewise; `0x00`, or `0x01`
/// and the PersonPublicKey's 32 bytes; the 32-byte Person device root.
/// LLR-edn55h.
fn encode_v1(person: &Person) -> Vec<u8> {
    let name = person.name().as_str().as_bytes();
    let surname = person.surname().as_str().as_bytes();
    let mut encoding = Vec::with_capacity(4 + name.len() + 4 + surname.len() + 1 + 32 + 32);
    encoding.extend_from_slice(&(name.len() as u32).to_le_bytes());
    encoding.extend_from_slice(name);
    encoding.extend_from_slice(&(surname.len() as u32).to_le_bytes());
    encoding.extend_from_slice(surname);
    match person.person_key() {
        None => encoding.push(0x00),
        Some(key) => {
            encoding.push(0x01);
            encoding.extend_from_slice(key.as_bytes());
        }
    }
    let device_root = compute_device_root::<PersonDeviceHasher>(person.devices());
    encoding.extend_from_slice(device_root.as_bytes());
    encoding
}

/// The hash of `person` under the encoding version `version`, refused with
/// `UnsupportedEncodingVersion` for a version this unit does not implement.
/// LLR-4ebtn4, LLR-rde6tk.
pub fn person_hash(person: &Person, version: u16) -> Result<PersonHash, IdentityError> {
    let version = EncodingVersion::try_from(version)?;
    Ok(hash_under(person, version))
}

fn hash_under(person: &Person, version: EncodingVersion) -> PersonHash {
    let key = match version {
        EncodingVersion::V1 => PERSON_DEFINITION_V1_KEY,
    };
    PersonHash(blake3::keyed_hash(key, &encode(person, version)).into())
}

/// `Ok(true)` exactly when `hash` is the hash of `person` under `version`,
/// `Ok(false)` when it is not, and `Err(UnsupportedEncodingVersion)` only for
/// a version this unit does not implement. LLR-tf45kx.
pub fn verify(person: &Person, version: u16, hash: &PersonHash) -> Result<bool, IdentityError> {
    Ok(person_hash(person, version)? == *hash)
}
```

4. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test person_hash`
   → `test result: ok. 13 passed; 0 failed`. If `the_v1_hash_is_pinned` or
   T3's `the_person_device_root_is_pinned` fails, the code differs from this
   plan's: stop and report — never re-pin to make it pass (a changed pin is a
   changed V1 encoding). Clippy → clean.

5. Commit: `git add person/src/hash.rs person/tests/person_hash.rs`, then
   `git -c commit.gpgsign=false commit -m "feat(person): V1 encoding, person_hash and verify, keyed per unit and version"`.

---

### T10 — Root re-exports and no-panic property tests

**Files touched:** `person/src/lib.rs`, `person/tests/no_panic.rs`
**Parallel:** yes — after T7, T8, T9, alongside T11

Trace: LLR-sjkmr6, LLR-kbhc43, LLR-4ku56h, LLR-wqha8d, LLR-tf45kx,
LLR-4ebtn4, LLR-rde6tk, LLR-63pkfc, LLR-3n3kxx (REQ-r4keha's no-panic half).

1. Failing tests. In `person/tests/no_panic.rs` (Edit tool), replace the
   header and imports:

```rust
//! Property tests: no input makes a constructor or a decoder panic, and every
//! accepted value satisfies its type's rule.

use ed25519_dalek::VerifyingKey;
use person::{x25519, DevicePublicKey, Name, PersonPublicKey, Surname, MAX_NAME_LEN};
#[cfg(feature = "serde")]
use person::{DeviceSlots, MAX_DEVICES};
use proptest::prelude::*;
```

   with:

```rust
//! Property tests: no input makes a constructor, a decoder or a Person
//! operation panic, and every accepted value satisfies its type's rules.

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::{SigningKey, VerifyingKey};
use person::{
    check_group_key, check_successor, person_hash, verify, x25519, DevicePublicKey, DeviceSlots,
    Name, Person, PersonHash, PersonPublicKey, Surname, MAX_DEVICES, MAX_NAME_LEN,
};
use proptest::prelude::*;
```

   and append to the end of the file:

```rust

/// The device key of a signing-key seed: every seed gives a valid key.
fn device_from_seed(seed: &[u8; 32]) -> Option<DevicePublicKey> {
    DevicePublicKey::try_from(SigningKey::from_bytes(seed).verifying_key()).ok()
}

/// A Person from small seeds, so that two draws share devices and keys often:
/// device seeds and the key seed index fixed signing and X25519 keys. `None`
/// when the fields break a rule.
fn person_from_small_seeds(key_seed: Option<u8>, device_seeds: &[u8]) -> Option<Person> {
    let devices: Vec<DevicePublicKey> = device_seeds
        .iter()
        .filter_map(|seed| device_from_seed(&[*seed; 32]))
        .collect();
    let key = key_seed.and_then(|seed| {
        PersonPublicKey::parse(&MontgomeryPoint::mul_base_clamped([seed; 32]).to_bytes()).ok()
    });
    let name = Name::parse("Alice").ok()?;
    let surname = Surname::parse("Example").ok()?;
    Person::new(name, surname, key, DeviceSlots::parse(devices).ok()?).ok()
}

/// The rules every accepted Person satisfies (REQ-ht3x78, REQ-7qgx2q, REQ-4szc22).
fn satisfies_the_person_rules(person: &Person) -> bool {
    let devices = person.devices().devices();
    let sorted = devices
        .windows(2)
        .all(|pair| matches!(pair, [a, b] if a < b));
    let key_rule = match person.person_key() {
        None => devices.is_empty(),
        Some(key) => {
            !devices.is_empty()
                && devices
                    .iter()
                    .all(|device| device.as_bytes() != key.as_bytes())
        }
    };
    devices.len() <= MAX_DEVICES && sorted && key_rule
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    /// Person::new on arbitrary fields never panics, accepts exactly what
    /// check_group_key accepts, and every Person it returns satisfies the rules.
    /// verifies: LLR-sjkmr6, LLR-kbhc43, LLR-4ku56h, LLR-3n3kxx
    #[test]
    fn person_new_on_any_fields(
        name in any::<String>(),
        surname in any::<String>(),
        key_bytes in proptest::option::of(any::<[u8; 32]>()),
        copy_a_device_key in any::<bool>(),
        seeds in proptest::collection::vec(any::<[u8; 32]>(), 0..6),
    ) {
        let devices: Vec<DevicePublicKey> = seeds.iter().filter_map(device_from_seed).collect();
        let key = if copy_a_device_key {
            devices.first().and_then(|device| PersonPublicKey::parse(device.as_bytes()).ok())
        } else {
            key_bytes.and_then(|bytes| PersonPublicKey::parse(&bytes).ok())
        };
        let (Ok(name), Ok(surname), Ok(slots)) =
            (Name::parse(&name), Surname::parse(&surname), DeviceSlots::parse(devices))
        else {
            return Ok(());
        };
        let expected = check_group_key(key.as_ref(), &slots);
        let built = Person::new(name, surname, key, slots);
        prop_assert_eq!(built.clone().map(|_| ()), expected);
        if let Ok(person) = built {
            prop_assert!(satisfies_the_person_rules(&person));
        }
    }

    /// check_successor never panics, and refuses exactly a device change that
    /// keeps a present key.
    /// verifies: LLR-wqha8d, LLR-3n3kxx
    #[test]
    fn check_successor_on_any_pair(
        previous_key in proptest::option::of(1u8..4),
        previous_devices in proptest::collection::vec(1u8..7, 0..5),
        next_key in proptest::option::of(1u8..4),
        next_devices in proptest::collection::vec(1u8..7, 0..5),
    ) {
        let (Some(previous), Some(next)) = (
            person_from_small_seeds(previous_key, &previous_devices),
            person_from_small_seeds(next_key, &next_devices),
        ) else {
            return Ok(());
        };
        let refused = previous.devices() != next.devices()
            && previous.person_key().is_some()
            && previous.person_key() == next.person_key();
        let expected = if refused {
            Err(person::IdentityError::PersonKeyNotRotated)
        } else {
            Ok(())
        };
        prop_assert_eq!(check_successor(&previous, &next), expected);
    }

    /// person_hash and verify never panic for any version and any supplied
    /// hash; only version 1 is implemented, and verify matches exactly the
    /// definition's own hash.
    /// verifies: LLR-tf45kx, LLR-4ebtn4, LLR-rde6tk
    #[test]
    fn hash_and_verify_on_any_version_and_hash(
        key in proptest::option::of(1u8..4),
        devices in proptest::collection::vec(1u8..7, 0..5),
        version in any::<u16>(),
        supplied in any::<[u8; 32]>(),
    ) {
        let Some(definition) = person_from_small_seeds(key, &devices) else {
            return Ok(());
        };
        let supplied = PersonHash::new(supplied);
        match person_hash(&definition, version) {
            Ok(own) => {
                prop_assert_eq!(version, 1);
                prop_assert_eq!(verify(&definition, version, &own), Ok(true));
                prop_assert_eq!(verify(&definition, version, &supplied), Ok(own == supplied));
            }
            Err(error) => {
                prop_assert_ne!(version, 1);
                prop_assert_eq!(&error, &person::IdentityError::UnsupportedEncodingVersion(version));
                prop_assert_eq!(verify(&definition, version, &supplied), Err(error));
            }
        }
    }

    /// Decoding arbitrary bytes as a Person never panics, and every Person it
    /// returns satisfies the rules and re-encodes to the bytes it consumed.
    /// verifies: LLR-63pkfc
    #[cfg(feature = "serde")]
    #[test]
    fn decoding_any_bytes_as_a_person_does_not_panic(
        bytes in proptest::collection::vec(any::<u8>(), 0..400),
    ) {
        if let Ok((person, rest)) = postcard::take_from_bytes::<Person>(&bytes) {
            prop_assert!(satisfies_the_person_rules(&person));
            let consumed = bytes.len() - rest.len();
            prop_assert_eq!(postcard::to_allocvec(&person).ok(), Some(bytes[..consumed].to_vec()));
        }
    }

    /// Byte strings shaped like a Person encoding — two short strings, an
    /// option tag, a key, a slot count and keys from signing seeds — reach
    /// Person::new's rules, which random bytes rarely do.
    /// verifies: LLR-63pkfc, LLR-sjkmr6, LLR-kbhc43
    #[cfg(feature = "serde")]
    #[test]
    fn decoding_person_shaped_bytes_does_not_panic(
        name in "[a-z]{0,8}",
        surname in "[a-z]{0,8}",
        key_tag in 0u8..3,
        key_bytes in any::<[u8; 32]>(),
        copy_a_device_key in any::<bool>(),
        slot_count in 0u8..6,
        seeds in proptest::collection::vec(any::<[u8; 32]>(), 0..6),
    ) {
        let mut devices: Vec<[u8; 32]> = seeds
            .iter()
            .filter_map(device_from_seed)
            .map(|device| *device.as_bytes())
            .collect();
        devices.sort();
        let key = match (copy_a_device_key, devices.first()) {
            (true, Some(first)) => *first,
            _ => key_bytes,
        };
        let mut bytes = postcard::to_allocvec(&name).unwrap_or_default();
        bytes.extend(postcard::to_allocvec(&surname).unwrap_or_default());
        bytes.push(key_tag);
        bytes.extend_from_slice(&key);
        bytes.push(slot_count);
        for device in &devices {
            bytes.extend_from_slice(device);
        }
        if let Ok(person) = postcard::from_bytes::<Person>(&bytes) {
            prop_assert!(satisfies_the_person_rules(&person));
        }
    }
}
```

2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test no_panic`
   → `error[E0432]: unresolved imports `person::check_group_key`, `person::check_successor`, `person::person_hash`, `person::verify`, `person::Person`, `person::PersonHash``.

3. In `person/src/lib.rs`, replace the re-export block

```rust
pub use device_key::DevicePublicKey;
pub use device_trie::{compute_device_root, DeviceTrieHasher, NodeHash};
pub use error::IdentityError;
pub use name::{Name, Surname, MAX_NAME_LEN, MAX_SURNAME_LEN};
pub use person_key::PersonPublicKey;
pub use slots::{DeviceSlots, MAX_DEVICES};
```

   with

```rust
pub use definition::Person;
pub use device_hasher::PersonDeviceHasher;
pub use device_key::DevicePublicKey;
pub use device_trie::{compute_device_root, DeviceTrieHasher, NodeHash};
pub use encoding::EncodingVersion;
pub use error::IdentityError;
pub use group_key::check_group_key;
pub use hash::{person_hash, verify, PersonHash};
pub use name::{Name, Surname, MAX_NAME_LEN, MAX_SURNAME_LEN};
pub use person_key::PersonPublicKey;
pub use slots::{DeviceSlots, MAX_DEVICES};
pub use successor::check_successor;
```

4. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test no_panic`
   → `test result: ok. 10 passed; 0 failed` (about 4 s). Clippy → clean, with
   and without `--no-default-features` (`DeviceSlots` and `MAX_DEVICES` are now
   used outside the `serde` tests, so their import is no longer gated).

5. Commit: `git add person/src/lib.rs person/tests/no_panic.rs`, then
   `git -c commit.gpgsign=false commit -m "test(person): no-panic property tests for the Person operations; root re-exports"`.

---

### T11 — bolero fuzz target over Person decoding and operations

**Files touched:** `person/Cargo.toml`, `person/tests/fuzz_person_decode/fuzz_target.rs` (new)
**Parallel:** yes — after T7, T8, T9, alongside T10 (it names items by module path, not by T10's re-exports)

Trace: LLR-63pkfc, LLR-tf45kx, LLR-wqha8d (the fuzz half of HAZ-h3swmw and
HAZ-y6ft54's open control; AGENTS.md "Always include fuzz testing").

1. Target `person/tests/fuzz_person_decode/fuzz_target.rs`:

```rust
//! Fuzz target: decoding arbitrary bytes as a Person, and every Person
//! operation on what decodes, must never panic; every decoded Person satisfies
//! the Person rules.
//!
//! Two shapes per input. Shape 1 decodes the bytes whole. Shape 2 puts them
//! after a valid name, surname and PersonPublicKey, where the device slots go,
//! so every iteration reaches the slot decoder, and every input whose slots
//! decode (a first byte of 0 always does) reaches Person::new's key rules;
//! random bytes alone rarely form two valid strings first.
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run it alone with
//! `cargo test -p person --test fuzz_person_decode`; deep-fuzz with
//! `cargo bolero test fuzz_person_decode --engine libfuzzer`.
//!
//! verifies: LLR-63pkfc, LLR-tf45kx, LLR-wqha8d

use bolero::check;
use person::definition::Person;
use person::group_key::check_group_key;
use person::hash::{person_hash, verify, PersonHash};
use person::successor::check_successor;

/// The postcard bytes of name "Alice", surname "Example" and a present
/// PersonPublicKey (u = 9): what precedes the device slots in shape 2.
fn prefix_before_the_slots() -> Vec<u8> {
    let mut prefix = vec![5];
    prefix.extend_from_slice(b"Alice");
    prefix.push(7);
    prefix.extend_from_slice(b"Example");
    prefix.push(1);
    let mut base_point = [0u8; 32];
    base_point[0] = 9;
    prefix.extend_from_slice(&base_point);
    prefix
}

fn exercise(person: &Person, bytes: &[u8]) {
    assert_eq!(
        check_group_key(person.person_key(), person.devices()),
        Ok(())
    );
    let mut supplied = [0u8; 32];
    for (slot, byte) in supplied.iter_mut().zip(bytes.iter()) {
        *slot = *byte;
    }
    let supplied = PersonHash::new(supplied);
    let _ = verify(person, 1, &supplied);
    let _ = verify(
        person,
        u16::from(bytes.first().copied().unwrap_or(2)),
        &supplied,
    );
    if let Ok(own) = person_hash(person, 1) {
        assert_eq!(verify(person, 1, &own), Ok(true));
    }
    assert_eq!(check_successor(person, person), Ok(()));
}

fn main() {
    let prefix = prefix_before_the_slots();
    check!().for_each(|bytes: &[u8]| {
        if let Ok(person) = postcard::from_bytes::<Person>(bytes) {
            exercise(&person, bytes);
        }
        let mut framed = prefix.clone();
        framed.extend_from_slice(bytes);
        if let Ok(person) = postcard::from_bytes::<Person>(&framed) {
            exercise(&person, bytes);
        }
    });
}
```

2. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test fuzz_person_decode`
   → `error: no test target named `fuzz_person_decode` in `person` package`
   (cargo discovers only `tests/*.rs` and `tests/*/main.rs`).

3. Append to `person/Cargo.toml` (`required-features`: without `serde` there
   is no `Deserialize` for `Person`, and `--no-default-features` must still
   build every target):

```toml

[[test]]
name = "fuzz_person_decode"
path = "tests/fuzz_person_decode/fuzz_target.rs"
harness = false
required-features = ["serde"]
```

4. `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --test fuzz_person_decode`
   → `test fuzz_person_decode ...	run time: 1.0…s | iterations/s: ~11000 | … | exit reason: max duration (1s - default) exceeded`, exit 0.
   Clippy → clean, with and without `--no-default-features`.

5. Commit: `git add person/Cargo.toml person/tests/fuzz_person_decode/fuzz_target.rs`, then
   `git -c commit.gpgsign=false commit -m "test(person): bolero fuzz target over Person decoding and operations"`.

---

### T12 — Close: risk text, sequencing note, coverage record, full gates

**Files touched:** `person/docs/risk/DRAFT-worktree-person-requirements-person-hazards.md`, `docs/plans/2026-10-04-person-sequencing.md`, `Makefile`
**Parallel:** no (serial, after T10 and T11)

Trace: no new test; this task runs every gate over the whole change.

1. Risk draft: replace

```markdown
Residual risk for HAZ-h3swmw and HAZ-y6ft54: reduced, **not acceptable**, until
the implementation is exercised by a fuzzer over arbitrary bytes. No unit in
this repository has a fuzz harness today, org-members included; one for
`person` is a plan item, not a requirement.
```

   with

```markdown
Residual risk for HAZ-h3swmw and HAZ-y6ft54: reduced, **not acceptable**. The
Person definition change adds a bolero fuzz target,
`person/tests/fuzz_person_decode`, over Person decoding and every Person
operation on what decodes; `cargo test -p person` runs it for one second
(about 11 000 inputs), and `cargo bolero test fuzz_person_decode --engine
libfuzzer` runs it as a coverage-guided campaign, of which none has yet been
run or recorded. (org-node and on-chain-client have bolero targets of their
own; org-members has none.) Whether a recorded campaign makes this residual
acceptable is the owner's evaluation.
```

   and in the residual-risk table replace both cells `no fuzzing yet` with
   `fuzz target in place; no recorded fuzzing campaign`.

2. `docs/plans/2026-10-04-person-sequencing.md`: replace

```markdown
- `person` itself: a fuzz harness over arbitrary bytes (HAZ-h3swmw,
  HAZ-y6ft54). No unit has one yet.
```

   with

```markdown
- `person` itself: a fuzz harness over arbitrary bytes (HAZ-h3swmw,
  HAZ-y6ft54). The target exists (`person/tests/fuzz_person_decode`, change 2);
  a recorded libfuzzer campaign does not.
```

   (Check the exact current wrapping with
   `grep -n -A2 "fuzz harness over arbitrary bytes" docs/plans/2026-10-04-person-sequencing.md`
   before editing; replace the whole bullet.)

3. `CARGO_HOME=/tmp/cargo_home_fuzz make coverage-person` → passes the floors;
   expected summary line
   `TOTAL  534  0  100.00%  73  0  100.00%  346  0  100.00% …` (regions, functions,
   lines). Then
   `CARGO_HOME=/tmp/cargo_home_fuzz cargo +nightly llvm-cov --offline -p person --branch --summary-only`
   → branches `36  0  100.00%` in the TOTAL row. In `Makefile`, after the line
   `# (worktree-person-unit-findings4): 28 of 28 branches, 100% decision coverage.`,
   add (with the figures actually measured, if they differ):

```makefile
#
# Re-measured 2026-10-06 by the Person definition change
# (worktree-person-requirements), same toolchains, floors unchanged:
#
#   person                    100.00%    100.00%   (346 of 346 lines; 534 of 534 regions)
#
# and with nightly --branch: 36 of 36 branches, 100% decision coverage.
```

4. Full gates, each with its expected result:
   - `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person` → every
     target ok; 122 tests passed in total (73 baseline + 49), and the fuzz
     target exits 0.
   - `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --offline -p person --no-default-features` → every target ok.
   - `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy --offline -p person --all-targets -- -D warnings` → `Finished`, no warning; the same with `--no-default-features`.
   - `cargo fmt -p person --check` → no output.
   - `GR_CONFIG=person/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`
     → exit 0; no MISSING-TEST line; summary `checked: REQ 18, HAZ 5, RC 5, SDD 6, LLR 25, PR 0, ADR 0`.
   - `GR_CONFIG=person/.guardrails/config.yaml .guardrails/scripts/check-ids.sh --allow-draft-files` → exit 0.

5. Commit: `git add person/docs/risk/DRAFT-worktree-person-requirements-person-hazards.md docs/plans/2026-10-04-person-sequencing.md Makefile`, then
   `git -c commit.gpgsign=false commit -m "docs(person): fuzz target in the risk file; coverage re-measured"`.

Then hand off: `check-traceability`, `verify-before-merge`, `merge-change`.

---

## Implements coverage

| ID | Task(s) | Tests (`verifies:` at the LLR level) |
|---|---|---|
| LLR-3n3kxx | T1, T2, T4, T6, T7, T8, T9, T10 | `the_person_variants_name_their_rule`; each refusal test names its variant |
| LLR-4ku56h | T6, T7, T10 | `a_person_holds_up_to_max_devices_sorted`, `over_the_bound_or_a_duplicate_is_refused_before_any_person_exists`, `a_field_breaking_its_own_rule_is_refused_with_its_message`, `person_new_on_any_fields` |
| LLR-sjkmr6 | T2, T6, T7, T10 | `with_no_device_the_key_must_be_absent`, `no_devices_and_no_key_is_accepted_and_a_key_without_devices_is_refused`, … |
| LLR-kbhc43 | T2, T6, T7, T10 | `with_one_to_four_devices_the_key_must_be_present`, `a_key_whose_bytes_equal_a_device_key_is_refused`, `the_check_compares_bytes_only`, … |
| LLR-63pkfc | T7, T10, T11 | `definition_decode.rs` (8 tests), the two decoding proptests, `fuzz_person_decode` |
| LLR-rde6tk | T4, T9, T10 | `version_one_is_v1_and_back`, `every_other_version_is_refused_and_named`, `hashing_under_an_unimplemented_version_is_refused` |
| LLR-edn55h | T3, T9 | `person_device_trie.rs` (5 tests), the three encoding tests, the pins |
| LLR-4ebtn4 | T9, T10 | `the_v1_hash_is_blake3_keyed_by_the_v1_domain_key`, `the_v1_hash_is_pinned`, `distinct_definitions_give_distinct_hashes`, `a_person_hash_*` |
| LLR-tf45kx | T9, T10, T11 | `verify_matches_exactly_the_hash_of_the_definition`, `verify_under_an_unimplemented_version_is_an_error`, `verify_keeps_nothing_between_calls`, `the_hash_sources_hold_no_state` |
| LLR-wqha8d | T8, T10, T11 | `successor.rs` (4 tests), `check_successor_on_any_pair` |
| REQ-9m5pq2 | T3, T9 | via LLR-edn55h, LLR-4ebtn4 |
| REQ-7n4g8b | T9 | via LLR-tf45kx |
| REQ-wg7z4s | T4, T9 | via LLR-rde6tk, LLR-4ebtn4 |
| REQ-ht3x78 | T2, T6 | via LLR-sjkmr6 |
| REQ-7qgx2q | T2, T6 | via LLR-kbhc43 |
| REQ-bhez2u | T8 | via LLR-wqha8d |
| REQ-7ymek3 | T7 | via LLR-63pkfc |
| REQ-r4keha | T1, T9, T10, T11 | via LLR-3n3kxx, LLR-tf45kx |
| SDD-4r2x79 | T6, T7 | its LLRs LLR-4ku56h, LLR-sjkmr6, LLR-kbhc43, LLR-63pkfc |
| SDD-v32mqh | T3, T4, T9 | its LLRs LLR-rde6tk, LLR-edn55h, LLR-4ebtn4, LLR-tf45kx |
| SDD-9hej83 | T8 | LLR-wqha8d |
| SDD-u9cddc (amended) | T1 | LLR-3n3kxx |

Normal and abnormal input per LLR (class C robustness): every row above has
at least one accepting test and one refusing or boundary test — LLR-edn55h's
abnormal cases are the boundary definitions (empty names, no key, no device,
names at their byte bound given decomposed) since encoding has no failure
input; LLR-4ebtn4's are the unimplemented versions and the one-field changes.

## Self-review (plan-change step 9)

1. Every ID in **Implements:** has a task whose test verifies it, at the LLR
   level, per the table above; the SDDs and REQs through their LLRs.
2. Every task shows its real code, its commands and their expected output;
   the code was run as written (scratch, 2026-10-06), formatted with
   `cargo fmt`, clippy-clean with and without default features, and removed
   before this plan was committed.
3. Names and signatures agree across tasks: `check_group_key(Option<&PersonPublicKey>, &DeviceSlots)`,
   `Person::new(Name, Surname, Option<PersonPublicKey>, DeviceSlots)`,
   `check_successor(&Person, &Person)`, `EncodingVersion::try_from(u16)`,
   `encode(&Person, EncodingVersion)`, `person_hash(&Person, u16)`,
   `verify(&Person, u16, &PersonHash)`; the IdentityError messages in T1 are
   the strings T4 and T7 compare.
4. Every task states **Files touched:** and **Parallel:**. Parallel sets are
   disjoint: wave 2 — T2 {group_key.rs ×2}, T3 {device_hasher.rs, person_device_trie.rs},
   T4 {encoding.rs, encoding_version.rs}, T5 {architecture draft, soup.md};
   wave 4 — T7 {definition_serde.rs, definition_decode.rs}, T8 {successor.rs ×2},
   T9 {hash.rs, person_hash.rs}; wave 5 — T10 {lib.rs, no_panic.rs},
   T11 {Cargo.toml, fuzz_target.rs}. `lib.rs` is touched by T1 and T10,
   `Cargo.toml` by T1 and T11, each pair serial.

## Progress

- T1 done (290b2c1). red -> green: `the_person_variants_name_their_rule`
  (verifies LLR-3n3kxx) — failed to compile with E0599, no variant
  `KeyWithoutDevice` (and the other four) on `IdentityError`, before
  error.rs had them. Suite 74 passed, 0 failed; clippy clean.
- T4 done (0946dad). red -> green: `version_one_is_v1_and_back` (LLR-rde6tk)
  and `every_other_version_is_refused_and_named` (LLR-rde6tk, LLR-3n3kxx) —
  both failed to compile with E0432, unresolved `person::encoding::EncodingVersion`,
  before it existed. Suite 76 passed, 0 failed; clippy clean.
- T2 done (48447e3). red -> green, each failed to compile with E0432,
  unresolved `person::group_key::check_group_key`, before it existed:
  `with_no_device_the_key_must_be_absent` (LLR-sjkmr6, LLR-3n3kxx),
  `with_one_to_four_devices_the_key_must_be_present` (LLR-kbhc43, LLR-3n3kxx),
  `a_key_whose_bytes_equal_a_device_key_is_refused` (LLR-kbhc43, LLR-3n3kxx),
  `the_check_compares_bytes_only` (LLR-kbhc43). Suite 78 passed on its branch;
  clippy clean.
- T3 done (1a97b70). red -> green, each failed to compile with E0432,
  unresolved `PersonDeviceHasher`, `PERSON_DEVICE_LEAF_KEY`,
  `PERSON_DEVICE_NODE_KEY`, before device_hasher.rs had them (all verify
  LLR-edn55h): `the_domain_keys_and_sentinel_are_the_documented_bytes`,
  `the_root_is_the_depth_two_tree_under_the_person_domains`,
  `the_person_device_root_is_pinned`, `no_set_shares_its_root_with_org_members`,
  `different_sets_give_different_roots`. Golden roots matched the plan.
  Suite 79 passed on its branch; clippy clean with and without default features.
- T5 done (4caa972): amended LLR-sjkmr6, LLR-kbhc43, LLR-63pkfc, LLR-edn55h,
  LLR-4ebtn4, LLR-tf45kx in place; SOUP as built (blake3 row; postcard,
  blake3, serde_json, bolero dev-dependencies) matches `cargo tree`.
  check-ids clean; check-trace only MISSING-TEST on unimplemented drafts.
- T6 done (580cbdc). red -> green, each failed to compile with E0432,
  unresolved `person::definition::Person`, before it existed:
  `a_person_holds_up_to_max_devices_sorted` (LLR-4ku56h),
  `over_the_bound_or_a_duplicate_is_refused_before_any_person_exists` (LLR-4ku56h),
  `no_devices_and_no_key_is_accepted_and_a_key_without_devices_is_refused`
  (LLR-sjkmr6, LLR-3n3kxx), `devices_without_a_key_are_refused` (LLR-kbhc43,
  LLR-3n3kxx), `a_key_copied_from_a_device_key_is_refused` (LLR-kbhc43,
  LLR-3n3kxx), `person_new_applies_check_group_key` (LLR-sjkmr6, LLR-kbhc43),
  `debug_redacts_the_names` (LLR-fbqs2r). Suite 92 passed (--all-features);
  clippy clean.
- T7 done (1f400e9). red -> green, each failed to compile with E0277,
  `Person` implementing neither `Serialize` nor `Deserialize`, before
  definition_serde.rs had them: `a_person_round_trips_through_both_formats`,
  `the_binary_form_is_the_four_fields_in_order`, `a_decoded_name_is_normalised`,
  `a_missing_or_unknown_field_is_refused`,
  `encoding_into_a_short_buffer_fails_at_every_field` (LLR-63pkfc);
  `a_definition_breaking_a_person_rule_is_refused_with_its_message`
  (LLR-63pkfc, LLR-sjkmr6, LLR-kbhc43, LLR-3n3kxx);
  `a_field_breaking_its_own_rule_is_refused_with_its_message` (LLR-63pkfc,
  LLR-4ku56h); `binary_bytes_breaking_a_person_rule_are_refused` (LLR-63pkfc,
  LLR-sjkmr6). Suite 100 passed on its branch; clippy clean both feature sets.
- T8 done (2eaf6ff). red -> green, each failed to compile with E0432,
  unresolved `person::successor::check_successor`:
  `a_device_change_with_a_new_key_is_accepted`,
  `unchanged_devices_accept_a_kept_or_a_rotated_key`,
  `only_the_immediate_predecessor_is_compared` (LLR-wqha8d);
  `a_device_change_that_keeps_the_key_is_refused` (LLR-wqha8d, LLR-3n3kxx).
  Suite 96 passed on its branch; clippy clean.
- T9 done (a5a96eb). red -> green, the whole target failed to compile with
  E0432, unresolved `person::hash::{encode, person_hash, verify, PersonHash,
  PERSON_DEFINITION_V1_KEY}`, before hash.rs had them:
  `the_v1_encoding_is_the_documented_layout`,
  `the_v1_encoding_of_the_boundary_definitions`,
  `equal_device_sets_encode_equally_whatever_their_order` (LLR-edn55h);
  `the_v1_hash_is_blake3_keyed_by_the_v1_domain_key`,
  `a_person_hash_wraps_32_bytes`, `a_person_hash_encodes_as_its_32_bytes`
  (LLR-4ebtn4); `the_v1_hash_is_pinned`, `distinct_definitions_give_distinct_hashes`
  (LLR-4ebtn4, LLR-edn55h); `hashing_under_an_unimplemented_version_is_refused`
  (LLR-4ebtn4, LLR-rde6tk, LLR-3n3kxx);
  `verify_matches_exactly_the_hash_of_the_definition`,
  `verify_keeps_nothing_between_calls`, `the_hash_sources_hold_no_state`
  (LLR-tf45kx); `verify_under_an_unimplemented_version_is_an_error`
  (LLR-tf45kx, LLR-3n3kxx). Both golden hashes matched the plan as written.
  Suite 105 passed on its branch; clippy clean both feature sets.
- T11 done (f54c589). red -> green: `fuzz_person_decode` (LLR-63pkfc,
  LLR-tf45kx, LLR-wqha8d) — first "no test target named fuzz_person_decode";
  once wired it was green on first run, so a panic was injected at the top of
  `check_successor` for any predecessor holding a device: the target failed in
  158 ms on it ("T11 mutation", cargo exit 101); mutation reverted, green again.
  Suite 118 passed on its branch; clippy clean both feature sets.
- T10 done (b2c0c7f). Each test first failed to compile with E0432 on the
  missing root re-exports; then a behavioural red by a temporary mutation,
  reverted before commit:
  `person_new_on_any_fields` (LLR-sjkmr6, LLR-kbhc43, LLR-4ku56h, LLR-3n3kxx) —
  `Person::new` ignoring `check_group_key`'s result;
  `check_successor_on_any_pair` (LLR-wqha8d, LLR-3n3kxx) — `&&` → `||`;
  `hash_and_verify_on_any_version_and_hash` (LLR-tf45kx, LLR-4ebtn4,
  LLR-rde6tk) — `verify` always `Ok(true)`;
  `decoding_any_bytes_as_a_person_does_not_panic` (LLR-63pkfc) — decoder
  `expect()` panicking on a decode error;
  `decoding_person_shaped_bytes_does_not_panic` (LLR-63pkfc, LLR-sjkmr6,
  LLR-kbhc43) — both the skip-check and the `expect()` mutations.
  Deviation from this plan's T10 text: the version strategy is
  `prop_oneof![Just(1u16), any::<u16>()]`, not `any::<u16>()`, which drew
  version 1 about once in 65536 cases and let the always-true `verify`
  mutant survive 2048 cases. Known limit:
  `decoding_any_bytes_as_a_person_does_not_panic` can only catch panics
  (random bytes almost never decode to a Person); the Person-shaped test is
  the one that reaches the rules. Suite 122 passed; clippy clean both
  feature sets.
- T12 done (8d4b91c): risk text corrected (fuzz target in place, no recorded
  campaign; verdict still "not acceptable" pending the owner), sequencing
  note, coverage re-measured 346/346 lines, 534/534 regions, 36/36 branches.
  Gates surfaced a break outside the plan: the five new `IdentityError`
  variants broke org-members' exhaustive `From<person::IdentityError>`
  (E0004), and with it org-node and app.
- Dependents fix (ad3030d, 574256c), not in the original task list: the five
  variants map to `OrgMembersError::InvariantViolated`, match kept exhaustive;
  new LLR-28ekrv (org-members draft). red -> green:
  `person_only_variants_map_to_invariant_violated` (LLR-28ekrv) — E0004, then
  green; a mutation mapping `UnsupportedEncodingVersion` to
  `SerializationError` failed it. `shared_variants_keep_their_names_and_fields`
  (LLR-28ekrv) — the same E0004 only; it restates the six existing arms.
  app/src-tauri/Cargo.lock gains blake3 under person (one line).
