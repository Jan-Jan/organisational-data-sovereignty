# Parse at the system's edge (org-members) — Implementation Plan

**Goal:** every org-members signature takes and returns validated or tag
newtypes instead of `&str`/`String`/`[u8; 32]`, per
`docs/adr/2026-10-04-parse-at-the-system-edge.md`, with org-node's call sites
updated to parse at its edge; wire bytes and root hashes unchanged.
**Implements:** REQ-t46uad; LLR-f3zrwd, LLR-377ddr, LLR-c5tzyp (review
round 1), LLR-yz2gmc (review round 2), LLR-v9evtu (review round 4) (new,
`org-members/docs/architecture/2026-10-04-type-safety.md`);
amended LLR-xzqs9r, LLR-5w2jx8, LLR-w5nkbu, LLR-4czn8t, LLR-68tka5,
LLR-ub6dw9, LLR-mmst86, LLR-g6arcs (in
`org-members/docs/architecture/2026-09-17-decomposition.md`). Derived items
assessed in `org-members/docs/risk/2026-10-04-type-safety.md`.
**Safety class:** C (org-members), C (org-node).
**Verification:** org-members `verify_commands` (`cargo test -p org-members`;
quint typecheck/test/run of `org-members/quint/membership*.qnt`); org-node
`verify_commands` (`cargo test -p org-node --features app,test-support --lib
--test …` as listed in `org-node/.guardrails/config.yaml`; quint runs of
`org-node/quint/protocol.qnt`); `cargo clippy -p org-members --all-targets`;
the `no_std`/wasm checks in `org-members/AGENTS.md`;
`GR_CONFIG=<unit>/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`
for both units, `check-ids.sh`, `check-units.sh`. Quint needs
`QUINT_HOME=<task worktree>/target/quint_home` — not the scratchpad, where the
rust evaluator is SIGKILLed (T2). A fresh `QUINT_HOME` can fail several
`mbt_conformance` tests while the evaluator downloads concurrently; rerun once
before treating it as red. Where GitHub fetches are blocked, copy (read-only)
`~/.quint/rust-evaluator-v0.7.0/quint_evaluator` into
`<QUINT_HOME>/rust-evaluator-v0.7.0/` first (T3). org-node builds need
`CARGO_HOME=/tmp/cargo_home_fuzz` (`~/.cargo` is read-only and lacks
`finito-0.1.0`).

## Owner decisions this plan implements (2026-10-04)

1. Plain types only where data enters the system; every signature, public or
   internal, takes the newtype.
2. Validated types (`Handle`, `Name`, `Surname`) are exported, field private,
   `parse` + `TryFrom` the only way in, `type Error = OrgMembersError`.
3. Tag types (`MemberId`, `NodeHash`, `RootHash`) have infallible `new` +
   `From<[u8; 32]>`. `RootHash::from_bytes` is renamed `new`. Crate-internal
   tag types take no raw bytes: `HeldKey` is built only `From` a member or
   device key, `HandleSkeleton` only via `HandleSkeleton::of(&Handle)`.
   (Corrected 2026-10-04 after independent review finding-4.)
4. `get_by_handle(&Handle) -> Option<MemberLeaf>`,
   `contains_handle(&Handle) -> bool`; an invalid string is refused by
   `Handle::parse` (REQ-t46uad).
5. Accessors return the newtype; `Debug` redacted, `Display`/`as_str()` show
   the value. Crate-private setters keep the `with_` prefix.

## Baseline (2026-10-04, before T1)

`cargo test -p org-members`: 9 + 153 + 32 (1 ignored) passed; quint
`mbtInv` run: no violation. Golden values captured from `master` @ `0f85cb9`
(T1 pins them).

## Order and parallelism

T1 ∥ T2, then T3 → T4 → T5 → T6 serially (T2–T5 all touch
`org-members/src/types.rs` or files that depend on its signatures). Between
T4 and T5 the workspace's org-node does not compile; T4 is verified with
`cargo test -p org-members` only.

---

### T1 — Golden encoding and root (LLR-377ddr)

**Status: done** (task commit `454bd7b`, merged 2026-10-04).
red -> green: member_set_root_is_pinned — characterization; GOLDEN_ROOT last digit d→c FAILED (left = real root …a19d), restored → passed.
red -> green: member_record_wire_bytes_are_pinned — characterization; GOLDEN_ALICE_LEAF last digit 7→8 FAILED (left = real encoding …c567), restored → passed.
Result: `cargo test -p org-members` 9 + 153 + 2 passed, mbt_conformance 32 passed / 1 ignored.

**Files touched:** `org-members/tests/encoding_golden.rs` (new)
**Parallel:** yes (with T2)

A characterization test: green on today's code by construction. Red is
demonstrated by flipping one hex digit of `GOLDEN_ROOT`, seeing the
assertion fail, and restoring it. Its job is to stay green, unmodified, through
T3–T5.

Step 1 — write `org-members/tests/encoding_golden.rs`:

```rust
//! Pins the wire bytes of a member record and the root of a member set, so a
//! type-level refactor cannot silently change what is signed or hashed.
//! Values captured on master @ 0f85cb9 (2026-10-04), before the newtype
//! refactor. Never update them to make this test pass.
use ed25519_dalek::SigningKey;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::types::{MemberId, MemberLeaf, P2pDeviceKey, P2pMemberKey};

const GOLDEN_ALICE_LEAF: &str = "902e667049de9cc485facf64b9a88b7ddf9ce1b5f4e7fb61aa7146e201d644bd05616c6963658092f31ae728a56911fc508958c73fadbe6a9b050aec7114e4412a3dc87298c805416c69636505536d697468015726e5bd4b887fcc929566a772f0fd5eb37d5695b4b60d4b298886b53c0bc567";
const GOLDEN_ROOT: &str = "2e81afbd4e9425357c2c26b6bb485e08d60b1a0606a02a8f8f9e76ac7282a19d";

fn seed(s: &str) -> [u8; 32] {
    blake3::hash(s.as_bytes()).into()
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

// LEAF-CONSTRUCTION: the only lines T4 may change in this file.
fn leaf(tag: &str, handle: &str, name: &str, surname: &str) -> MemberLeaf {
    MemberLeaf::new(
        MemberId::new(seed(&format!("{tag}-id"))),
        handle,
        P2pMemberKey::new(SigningKey::from_bytes(&seed(&format!("{tag}-mk"))).verifying_key()),
        name,
        surname,
        vec![P2pDeviceKey::new(SigningKey::from_bytes(&seed(&format!("{tag}-d1"))).verifying_key())],
    )
    .unwrap()
}

/// verifies: LLR-377ddr
#[test]
fn member_record_wire_bytes_are_pinned() {
    let alice = leaf("alice", "alice", "Alice", "Smith");
    assert_eq!(hex(&postcard::to_allocvec(&alice).unwrap()), GOLDEN_ALICE_LEAF);
    let back: MemberLeaf = postcard::from_bytes(&postcard::to_allocvec(&alice).unwrap()).unwrap();
    assert_eq!(back, alice);
}

/// verifies: LLR-377ddr
#[test]
fn member_set_root_is_pinned() {
    let alice = leaf("alice", "alice", "Alice", "Smith");
    let bob = leaf("bob", "bob", "Bob", "Jones");
    let (trie, _) = OrgTrie::<Blake3Hasher>::genesis(vec![alice, bob])
        .unwrap()
        .recalculate()
        .unwrap();
    assert_eq!(hex(trie.root_hash().unwrap().as_bytes()), GOLDEN_ROOT);
}
```

Step 2 — run, expect green:
`cargo test -p org-members --test encoding_golden` → `test result: ok. 2 passed`.
Step 3 — sanity red: change the last digit of `GOLDEN_ROOT` to `c`, rerun →
`member_set_root_is_pinned ... FAILED`; restore, rerun → 2 passed. Record
both runs as the `red -> green:` attestation.
Step 4 — commit `test(org-members): pin member-record bytes and root (LLR-377ddr)`.

---

### T2 — `Handle`, `Name`, `Surname` (LLR-xzqs9r, LLR-w5nkbu, LLR-4czn8t, LLR-68tka5)

**Status: done** (task commit `8802547`, merged 2026-10-04).
red -> green: handle_parse_stores_nfc — failed E0432 (Handle/Name/Surname unresolved) before the types existed.
red -> green: handle_parse_rejects_every_rule — failed E0432 before the types existed.
red -> green: name_and_surname_parse_nfc_and_bound — failed E0432 before the types existed.
red -> green: newtype_debug_redacts_and_display_shows — failed E0432 before the types existed.
red -> green: decode_parses_newtype_fields — failed E0432 before the types existed.
red -> green: newtypes_encode_as_their_string — failed E0432 before the types existed.
Result: `cargo test -p org-members` 9 + 153 + 32/1 ignored + 6 passed; `no_std` and wasm checks clean.

**Files touched:** `org-members/src/types.rs`, `org-members/src/lib.rs`,
`org-members/tests/newtypes.rs` (new)
**Parallel:** yes (with T1)

The types are added beside the existing code; nothing uses them yet.
`validate_handle` keeps working until T4 deletes it.

Step 1 — write the failing tests, `org-members/tests/newtypes.rs`:

```rust
use org_members::types::{Handle, Name, Surname, MAX_HANDLE_LEN, MAX_NAME_LEN};
use org_members::OrgMembersError;

/// verifies: LLR-xzqs9r
#[test]
fn handle_parse_stores_nfc() {
    let h = Handle::parse("jose\u{0301}").unwrap(); // NFD é
    assert_eq!(h.as_str(), "jos\u{00e9}");
    assert_eq!(Handle::try_from("alice").unwrap(), Handle::parse("alice").unwrap());
    assert_eq!(Handle::try_from(String::from("alice")).unwrap(), Handle::parse("alice").unwrap());
    assert!(Handle::parse("jan-jan").is_ok());
}

/// verifies: LLR-xzqs9r
#[test]
fn handle_parse_rejects_every_rule() {
    let over = "a".repeat(MAX_HANDLE_LEN + 1);
    for bad in ["", "Alice", "a.b", "a\u{0430}", over.as_str()] {
        assert!(
            matches!(Handle::parse(bad), Err(OrgMembersError::InvalidHandle(_))),
            "accepted {bad:?}"
        );
        assert!(matches!(Handle::try_from(bad), Err(OrgMembersError::InvalidHandle(_))));
    }
    assert!(Handle::parse(&"a".repeat(MAX_HANDLE_LEN)).is_ok());
}

/// verifies: LLR-w5nkbu
#[test]
fn name_and_surname_parse_nfc_and_bound() {
    assert_eq!(Name::parse("Jose\u{0301}").unwrap().as_str(), "Jos\u{00e9}");
    assert_eq!(Surname::parse("Smith").unwrap().as_str(), "Smith");
    let over = "a".repeat(MAX_NAME_LEN + 1);
    assert_eq!(
        Name::parse(&over),
        Err(OrgMembersError::FieldTooLong { field: "name", max: 128 })
    );
    assert_eq!(
        Surname::parse(&over),
        Err(OrgMembersError::FieldTooLong { field: "surname", max: 128 })
    );
    assert!(Name::parse(&"a".repeat(MAX_NAME_LEN)).is_ok());
}

/// verifies: LLR-4czn8t
#[test]
fn newtype_debug_redacts_and_display_shows() {
    let h = Handle::parse("alice").unwrap();
    let n = Name::parse("Alice").unwrap();
    let s = Surname::parse("Smith").unwrap();
    assert_eq!(format!("{h:?}"), "Handle([REDACTED])");
    assert_eq!(format!("{n:?}"), "Name([REDACTED])");
    assert_eq!(format!("{s:?}"), "Surname([REDACTED])");
    assert_eq!(format!("{h} {n} {s}"), "alice Alice Smith");
}

/// verifies: LLR-68tka5
#[test]
fn decode_parses_newtype_fields() {
    let enc = |s: &str| postcard::to_allocvec(&String::from(s)).unwrap();
    assert!(postcard::from_bytes::<Handle>(&enc("Alice")).is_err());
    assert!(postcard::from_bytes::<Handle>(&enc("")).is_err());
    assert!(postcard::from_bytes::<Name>(&enc(&"a".repeat(129))).is_err());
    assert!(postcard::from_bytes::<Surname>(&enc(&"a".repeat(129))).is_err());
    let h: Handle = postcard::from_bytes(&enc("jose\u{0301}")).unwrap();
    assert_eq!(h.as_str(), "jos\u{00e9}");
}

/// verifies: LLR-377ddr
#[test]
fn newtypes_encode_as_their_string() {
    let h = Handle::parse("alice").unwrap();
    assert_eq!(postcard::to_allocvec(&h).unwrap(), postcard::to_allocvec(&"alice").unwrap());
    let n = Name::parse("Alice").unwrap();
    assert_eq!(postcard::to_allocvec(&n).unwrap(), postcard::to_allocvec(&"Alice").unwrap());
}
```

Step 2 — run, expect red (does not compile):
`cargo test -p org-members --test newtypes` →
`error[E0432]: unresolved imports org_members::types::Handle, …`.

Step 3 — in `org-members/src/types.rs`, add after the
`validate_handle`/`handle_skeleton` functions:

```rust
/// A validated member handle (REQ-h5ret5): NFC, non-empty, at most
/// `MAX_HANDLE_LEN` bytes, lowercase, no `.`, UTS#39 identifier characters or
/// `-`, single-script. `parse` is the only way in; PII, so `Debug` redacts.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct Handle(String);

impl Handle {
    /// Normalizes to NFC and enforces the handle rules. LLR-xzqs9r.
    pub fn parse(value: &str) -> Result<Self, OrgMembersError> {
        if value.is_empty() {
            return Err(OrgMembersError::InvalidHandle(
                "handle must not be empty".to_string(),
            ));
        }
        let normalized: String = value.nfc().collect();
        if normalized.len() > MAX_HANDLE_LEN {
            return Err(OrgMembersError::InvalidHandle(format!(
                "handle exceeds {} bytes after NFC normalization",
                MAX_HANDLE_LEN
            )));
        }
        for ch in normalized.chars() {
            if ch.is_uppercase() {
                return Err(OrgMembersError::InvalidHandle(
                    "handle must be lowercase".to_string(),
                ));
            }
            if ch == '.' {
                return Err(OrgMembersError::InvalidHandle(
                    "handle must not contain '.'".to_string(),
                ));
            }
            if ch == '-' {
                continue;
            }
            if !ch.identifier_allowed() {
                return Err(OrgMembersError::InvalidHandle(format!(
                    "character {:?} not allowed by UTS#39",
                    ch
                )));
            }
        }
        if !normalized.is_single_script() {
            return Err(OrgMembersError::InvalidHandle(
                "handle must not mix scripts".to_string(),
            ));
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Handle {
    type Error = OrgMembersError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<String> for Handle {
    type Error = OrgMembersError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Handle> for String {
    fn from(h: Handle) -> Self {
        h.0
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Handle([REDACTED])")
    }
}

/// NFC-normalizes `value` and bounds it at `max` bytes. LLR-w5nkbu.
fn nfc_bounded(value: &str, field: &'static str, max: usize) -> Result<String, OrgMembersError> {
    let nfc = to_nfc(value);
    if nfc.len() > max {
        return Err(OrgMembersError::FieldTooLong { field, max });
    }
    Ok(nfc)
}

/// A member's given name: NFC, at most `MAX_NAME_LEN` bytes. PII.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct Name(String);

impl Name {
    pub fn parse(value: &str) -> Result<Self, OrgMembersError> {
        nfc_bounded(value, "name", MAX_NAME_LEN).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Name {
    type Error = OrgMembersError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<String> for Name {
    type Error = OrgMembersError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Name> for String {
    fn from(n: Name) -> Self {
        n.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Name([REDACTED])")
    }
}

/// A member's surname: NFC, at most `MAX_SURNAME_LEN` bytes. PII.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct Surname(String);

impl Surname {
    pub fn parse(value: &str) -> Result<Self, OrgMembersError> {
        nfc_bounded(value, "surname", MAX_SURNAME_LEN).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Surname {
    type Error = OrgMembersError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<String> for Surname {
    type Error = OrgMembersError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Surname> for String {
    fn from(s: Surname) -> Self {
        s.0
    }
}

impl fmt::Display for Surname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for Surname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Surname([REDACTED])")
    }
}
```

Then make `validate_handle` delegate so there is one implementation:

```rust
pub fn validate_handle(handle: &str) -> Result<String, OrgMembersError> {
    Handle::parse(handle).map(String::from)
}
```

Step 4 — in `org-members/src/lib.rs` extend the re-export:

```rust
pub use types::{
    Handle, MemberId, MemberLeaf, Name, P2pDeviceKey, P2pMemberKey, RootHash, Surname,
};
```

Step 5 — `cargo test -p org-members` → all suites green, `newtypes` 6
passed. `cargo check -p org-members --no-default-features` and
`--no-default-features --features serde --target wasm32-unknown-unknown` →
clean (the `serde(try_from)` attribute is gated with the derive).
Step 6 — commit `feat(org-members): Handle, Name, Surname validated newtypes`.

---

### T3 — Tag types and `HeldKey`

**Status: done** (task commit `36b27da`, merged 2026-10-04).
red -> green: n/a — behaviour-preserving refactor; guard is the unchanged suite plus `encoding_golden` (2 passed after the change: bytes and root unchanged).
Result: `cargo test -p org-members` all green (golden 2, fuzz 9, integration 153, mbt 32/1 ignored, newtypes 6); `cargo test -p org-node --features app,test-support --lib --test verify_against_chain` 23 + 13 passed. `trie.rs` edited by asserted exact-string replace (not sed), diff reviewed.

**Files touched:** `org-members/src/types.rs`, `org-members/src/trie.rs`,
`org-members/README.md`, `org-members/tests/integration_test.rs`,
`org-node/src/chain_read.rs`, `org-node/src/chain.rs`,
`org-node/src/service.rs`, `org-node/tests/verify_against_chain.rs`
**Parallel:** no (serial, after T1 and T2)

A refactor with no new behaviour: the guard is the unchanged suite plus T1.

Step 1 — `types.rs`: add `From<[u8; 32]>` for the three tag types, and
rename `RootHash::from_bytes` to `new`:

```rust
impl From<[u8; 32]> for MemberId {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<[u8; 32]> for NodeHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl RootHash {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    // as_bytes unchanged
}

impl From<[u8; 32]> for RootHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}
```

and add the crate-internal key-index type:

```rust
/// The 32 encoded bytes of a key held in the organisation -- member key or
/// device key alike: the same bytes are the same key (LLR-v6gfc7). Tag type.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct HeldKey([u8; 32]);

impl From<&P2pMemberKey> for HeldKey {
    fn from(k: &P2pMemberKey) -> Self {
        Self(*k.as_bytes())
    }
}

impl From<&P2pDeviceKey> for HeldKey {
    fn from(k: &P2pDeviceKey) -> Self {
        Self(*k.as_bytes())
    }
}
```

Step 2 — `trie.rs`:

```rust
pub(crate) type KeyIndex = HashMap<HeldKey, MemberId>;

fn leaf_keys(leaf: &MemberLeaf) -> impl Iterator<Item = HeldKey> + '_ {
    core::iter::once(HeldKey::from(leaf.p2p_key()))
        .chain(leaf.p2p_devices().iter().map(HeldKey::from))
}

fn refuse_held_key(&self, key: HeldKey) -> Result<(), OrgMembersError> {
    if self.key_index.contains_key(&key) {
        return Err(OrgMembersError::DuplicateKey);
    }
    Ok(())
}
```

The four callers (`trie.rs` lines 295, 317, 357, 396 at 0f85cb9) become
`self.refuse_held_key(HeldKey::from(&new_p2p_key))?` and
`self.refuse_held_key(HeldKey::from(&device))?`. Add `HeldKey` to the
`crate::types` import.

Step 3 — replace `RootHash::from_bytes(` with `RootHash::new(` (Edit tool,
never `sed`) at: `org-members/tests/integration_test.rs` (1 site),
`org-node/src/chain_read.rs:19`, `org-node/src/chain.rs:58`,
`org-node/src/service.rs:127`, `:152`, `:198`,
`org-node/tests/verify_against_chain.rs:146`; and the prose in
`org-members/README.md:68` ("`RootHash::new` accepts any 32 bytes").
`grep -rn "RootHash::from_bytes" --include='*.rs' --include='*.md' .` must
then print only dated ledger/plan files.

Step 4 — `cargo test -p org-members` → green, `encoding_golden` 2 passed;
`cargo test -p org-node --features app,test-support --lib` → green.
Step 5 — commit `refactor(org-members): tag types gain From, RootHash::new, HeldKey keys the key index`.

---

### T4 — `MemberLeaf` and `OrgTrie` take newtypes (REQ-t46uad, LLR-f3zrwd, LLR-5w2jx8, LLR-ub6dw9, LLR-mmst86, LLR-g6arcs, LLR-68tka5)

**Files touched:** `org-members/src/types.rs`, `org-members/src/trie.rs`,
`org-members/src/delta.rs`, `org-members/tests/integration_test.rs`,
`org-members/tests/fuzz_tests.rs`, `org-members/tests/mbt_conformance.rs`,
`org-members/tests/encoding_golden.rs` (LEAF-CONSTRUCTION lines only)
**Parallel:** no (serial, after T3)

**Status: done** (task commit `b7cd38c`, merged 2026-10-04).
red -> green: handle_query_reports_invalid_absent_and_held — failed E0308 "expected `&str`, found `&Handle`" at the get_by_handle/contains_handle sites before the typed lookups existed; passes after.
Result: `cargo test -p org-members` golden 2 (constants and bodies unchanged — diff checked by dispatcher), fuzz 9, integration 154, mbt 32/1 ignored (no outcome mismatch), newtypes 6; clippy clean; no_std/wasm clean. Deviations: `fuzz_tests.rs` confusable oracle now calls `unicode_security::confusable_detection::skeleton` directly (`HandleSkeleton` is crate-internal); pre-existing clippy `manual_contains` at `fuzz_tests.rs` fixed; mbt driver parses inline instead of using `h`/`nm`/`sn` helpers.

Step 1 — failing test, appended to `org-members/tests/integration_test.rs`
(add `Handle` to the `org_members::types` import):

```rust
/// verifies: REQ-t46uad, LLR-f3zrwd
#[test]
fn handle_query_reports_invalid_absent_and_held() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();

    // Invalid: refused before any lookup can run.
    assert!(matches!(Handle::parse("Alice"), Err(OrgMembersError::InvalidHandle(_))));

    // Valid, held by no member.
    let absent = Handle::parse("zoe").unwrap();
    assert_eq!(trie.get_by_handle(&absent), None);
    assert!(!trie.contains_handle(&absent));

    // Valid, held.
    let held = Handle::parse("alice").unwrap();
    assert_eq!(trie.get_by_handle(&held).map(|m| *m.id()), Some(member_id("alice-id")));
    assert!(trie.contains_handle(&held));
}
```

Step 2 — `cargo test -p org-members --test integration_test` → red:
`error[E0308]: mismatched types … expected &str, found &Handle`.

Step 3 — `types.rs`:

- Add the crate-internal skeleton type, and delete `validate_handle` and
  `handle_skeleton`:

```rust
/// The UTS#39 skeleton of a handle: handles rendering alike share one
/// (LLR-5w2jx8). Tag type, built only from a `Handle`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct HandleSkeleton(String);

impl HandleSkeleton {
    pub(crate) fn of(handle: &Handle) -> Self {
        use unicode_security::confusable_detection::skeleton;
        Self(skeleton(handle.as_str()).collect())
    }
}
```

- `MemberLeaf` fields become `handle: Handle`, `name: Name`,
  `surname: Surname`. Delete `MemberLeafSerde` and both hand-written serde
  impls of `MemberLeaf`; derive instead (field order unchanged → same bytes,
  T1 proves it; every field's own `Deserialize` validates):

```rust
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemberLeaf {
    id: MemberId,
    handle: Handle,
    p2p_key: P2pMemberKey,
    name: Name,
    surname: Surname,
    p2p_devices: P2pDeviceSlots,
}
```

- Constructor, setters, accessors:

```rust
pub fn new(
    id: MemberId,
    handle: Handle,
    p2p_key: P2pMemberKey,
    name: Name,
    surname: Surname,
    p2p_devices: Vec<P2pDeviceKey>,
) -> Result<Self, OrgMembersError> {
    if p2p_devices.is_empty() {
        return Err(OrgMembersError::EmptyDeviceList);
    }
    let p2p_devices = P2pDeviceSlots::new(p2p_devices)?;
    Ok(Self { id, handle, p2p_key, name, surname, p2p_devices })
}

pub(crate) fn with_name_surname(mut self, name: Name, surname: Surname) -> Self {
    self.name = name;
    self.surname = surname;
    self
}

pub(crate) fn with_handle(mut self, handle: Handle) -> Self {
    self.handle = handle;
    self
}

pub fn handle(&self) -> &Handle { &self.handle }
pub fn name(&self) -> &Name { &self.name }
pub fn surname(&self) -> &Surname { &self.surname }
```

  Replace the modifier block comment's last sentence ("They do NOT validate
  the handle -- callers must validate before calling `with_handle`.") with
  "They do not check trie-level rules (uniqueness, confusables, held keys);
  the newtype arguments carry field validity."

- `canonical_bytes`: `self.handle.as_bytes()` → `self.handle.as_str().as_bytes()`,
  likewise `name`, `surname`.
- `use crate::normalize::to_nfc;` stays (used by `nfc_bounded`);
  `use alloc::string::ToString` stays (used by `Handle::parse`).

Step 4 — `trie.rs`:

- Import `Handle, HandleSkeleton, Name, Surname` from `crate::types`; drop
  `handle_skeleton, validate_handle` and `crate::normalize::to_nfc`.
- Index types: `skeleton_index: HashMap<HandleSkeleton, Handle>`,
  `handle_index: HashMap<Handle, MemberId>` — in `OrgTrie`, in the
  `from_parts`-style constructor at line ~820, and in `delta.rs:128-129`.
- Every `handle_skeleton(x.handle())` → `HandleSkeleton::of(x.handle())`;
  every `x.handle().to_owned()` → `x.handle().clone()`. The
  `existing != member.handle()` comparisons and `remove(x.handle())` calls
  compile unchanged (`&Handle` on both sides).
- Lookups and domain operations:

```rust
/// LLR-f3zrwd.
pub fn contains_handle(&self, handle: &Handle) -> bool {
    self.handle_index.contains_key(handle)
}

/// LLR-f3zrwd.
pub fn get_by_handle(&self, handle: &Handle) -> Option<MemberLeaf> {
    let id = self.handle_index.get(handle)?;
    smt::get_member(&self.root, id)
}

/// Updates a member's name and surname. LLR-g6arcs.
pub fn update_name_surname(
    &self,
    id: &MemberId,
    name: Name,
    surname: Surname,
) -> Result<Self, OrgMembersError> {
    let existing = smt::get_member(&self.root, id).ok_or(OrgMembersError::IdNotFound)?;
    self.update_leaf(existing.with_name_surname(name, surname))
}

/// Updates a member's handle. The handle is valid by construction; it must
/// not be taken by, or confusably collide with, another member's. LLR-mmst86.
pub fn update_handle(&self, id: &MemberId, new_handle: Handle) -> Result<Self, OrgMembersError> {
    let existing = smt::get_member(&self.root, id).ok_or(OrgMembersError::IdNotFound)?;
    self.update_leaf(existing.with_handle(new_handle))
}
```

Step 5 — `delta.rs:38` doc comment: "stores the NFC form `validate_handle`
returns" → "stores the NFC form `Handle::parse` produces".

Step 6 — update the tests (Edit tool only; never `sed` on test files). In each
of `integration_test.rs`, `fuzz_tests.rs`, `mbt_conformance.rs` add, next to
the existing helpers:

```rust
fn h(s: &str) -> Handle { Handle::parse(s).unwrap() }
fn nm(s: &str) -> Name { Name::parse(s).unwrap() }
fn sn(s: &str) -> Surname { Surname::parse(s).unwrap() }
```

and apply these rewrites:

| Before | After |
|---|---|
| `MemberLeaf::new(id, "alice", k, "Alice", "Smith", d)` (valid literals) | `MemberLeaf::new(id, h("alice"), k, nm("Alice"), sn("Smith"), d)` |
| a helper returning `Result` from `MemberLeaf::new(…, handle, …)` with a possibly invalid handle (e.g. `leaf_with_handle`, mbt `new_leaf`/`seed_leaf`) | `MemberLeaf::new(…, Handle::parse(handle)?, …, Name::parse(name)?, Surname::parse(surname)?, …)` — the function's `Result<_, OrgMembersError>` is unchanged, so assertions on `InvalidHandle`/`FieldTooLong` keep passing |
| `trie.update_handle(&id, "x")` expecting success | `trie.update_handle(&id, h("x"))` |
| `trie.update_handle(&id, s)` where `s` may be invalid (mbt driver; tests asserting `InvalidHandle`) | `Handle::parse(s).and_then(\|nh\| trie.update_handle(&id, nh))` |
| `trie.update_name_surname(&id, a, b)` | `Name::parse(a).and_then(\|n\| Ok((n, Surname::parse(b)?))).and_then(\|(n, s)\| trie.update_name_surname(&id, n, s))` where a/b may be invalid; `trie.update_name_surname(&id, nm(a), sn(b))` otherwise |
| `trie.get_by_handle("x")` / `trie.contains_handle("x")` | `trie.get_by_handle(&h("x"))` / `trie.contains_handle(&h("x"))` — where `"x"` is not a valid handle (e.g. NFD-input tests), use `h()` (it normalizes) and keep the assertion |
| `leaf.handle() == "x"`, `leaf.name() == "X"`, `assert_eq!(leaf.handle(), "x")` | `leaf.handle().as_str() == "x"` etc. |
| `validate_handle(&s)` in `fuzz_tests.rs::handle_validation_never_panics` | `Handle::parse(&s)`, body iterates `normalized.as_str().chars()` and `normalized.as_str().contains('.')`; import `Handle` instead of `validate_handle` |
| `find_confusable_pair` using `handle_skeleton` | decide confusability through the trie, keeping `HandleSkeleton` internal (below) |

`find_confusable_pair` replacement (`integration_test.rs`):

```rust
/// Finds two distinct valid handles that the trie refuses as confusable.
fn find_confusable_pair() -> Option<(String, String)> {
    let candidates = [
        "paypal", "paypa1", "h0use", "house", "g00gle", "google", "ab1", "abl", "amaz0n", "amazon",
        "g0t", "got", "0lice", "olice", "01ice", "alice",
    ];
    for (i, &a) in candidates.iter().enumerate() {
        let trie = TestTrie::genesis(vec![leaf_with_handle(a).ok()?]).ok()?;
        for &b in &candidates[i + 1..] {
            let other = MemberLeaf::new(
                member_id("confusable-probe"),
                h(b),
                member_key("confusable-probe-mk"),
                nm("P"),
                sn("Q"),
                vec![device_key("confusable-probe-d1")],
            )
            .ok()?;
            if matches!(trie.add_member(other), Err(OrgMembersError::ConfusableHandle)) {
                return Some((a.to_string(), b.to_string()));
            }
        }
    }
    None
}
```

(`leaf_with_handle` is the existing helper at `integration_test.rs` near line
1060; after this task it parses its argument with `Handle::parse(handle)?`.)

In `encoding_golden.rs` change only the LEAF-CONSTRUCTION function body to
`Handle::parse(handle).unwrap()`, `Name::parse(name).unwrap()`,
`Surname::parse(surname).unwrap()` and its imports; the two constants and both
test bodies stay byte-for-byte.

Step 7 — `cargo test -p org-members` → green, including
`handle_query_reports_invalid_absent_and_held`, `encoding_golden` (2), and
`mbt_conformance` (32 passed, 1 ignored). `cargo clippy -p org-members
--all-targets` → no warnings. The three `no_std`/wasm checks → clean.
`grep -rn "validate_handle\|handle_skeleton" org-members/src org-members/tests`
→ no output.
Step 8 — commit `feat(org-members): MemberLeaf and OrgTrie take Handle, Name, Surname; lookups take &Handle (REQ-t46uad)`.

---

### T5 — org-node parses at its edge

**Files touched:** `org-node/src/service.rs`, `org-node/src/test_fixtures.rs`,
`org-node/tests/transport_handshake.rs`, `org-node/tests/chain_genesis_e2e.rs`,
`org-node/tests/service_stories.rs`, `org-node/tests/transport_networked.rs`,
`org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`
**Parallel:** no (serial, after T4)

**Status: done** (task commit `5a17f44`, merged 2026-10-04).
red -> green: org-node lib build — failed E0308 at `service.rs` :493, :622, :816 and `test_fixtures.rs:29` (a fourth site the plan missed) before parsing at the edge; green after.
Result: org-node cargo gate line all passed (lib 23, admission_sender 7, service_stories 3, store_at_rest 4, transport_handshake 3, transport_networked 1, verify_against_chain 13, wire_frame_bound 3; 3 fuzz targets exit 0); quint typechecks and all 5 invariants "No violation found"; `cargo test -p org-members` green; app `cargo check` clean. Pre-existing `unwrap_used` clippy errors in org-node test modules, untouched lines.

No new behaviour: org-node's stored/wire structs keep `String` for now (the
follow-up change converts them). The guard is the org-node gate.

Step 1 — red: `cargo test -p org-node --features app,test-support --lib` →
`error[E0308]: mismatched types` at the three `MemberLeaf::new` sites.

Step 2 — `service.rs`: import `org_members::{Handle, Name, Surname}`; at each
`MemberLeaf::new` site (lines ~493, ~622, ~816 at 0f85cb9) parse the edge
strings, mapping errors exactly as today (`OrgNodeError::Trie`):

```rust
let leaf = MemberLeaf::new(
    MemberId::new(s.id),
    Handle::parse(&s.handle).map_err(OrgNodeError::Trie)?,
    org_members::P2pMemberKey::new(member_vk),
    Name::parse(&s.name).map_err(OrgNodeError::Trie)?,
    Surname::parse(&s.surname).map_err(OrgNodeError::Trie)?,
    device_keys?,
)
.map_err(OrgNodeError::Trie)?;
```

(the `create_organisation` site uses `&handle`, `&name`, `&surname`; the
`admit_member` site `&join_request.handle`, `&join_request.name`,
`&join_request.surname`). The three snapshot sites
`m.handle().to_string()` etc. compile unchanged via `Display`.

Step 3 — `test_fixtures.rs::member` and the six test sites build their leaves
with `Handle::parse(handle).unwrap()`, `Name::parse("Test").unwrap()`,
`Surname::parse("User").unwrap()` (or the literal each site passes today).

Step 4 — run the full org-node `verify_commands` (cargo line and the quint
runs) → green; `cargo build -p app` is not part of the workspace — run
`cargo check --manifest-path app/src-tauri/Cargo.toml` → clean.
Step 5 — commit `refactor(org-node): parse handle, name, surname at the edge for org-members' typed API`.

---

### T6 — Documentation follow-through

**Files touched:** `org-members/AGENTS.md`, `org-members/README.md`,
`org-members/docs/architecture/soup.md`
**Parallel:** no (serial, after T5)

**Status: done** (task commit `71dbc15`, merged 2026-10-04). Documentation only (red -> green n/a). check-trace: no MISSING-TEST for REQ-t46uad, LLR-f3zrwd, LLR-377ddr in any unit (pre-existing UNRESOLVED-PR / UNMET-EXPECTATION only); check-ids per unit: DRAFT-FILE only (finalized at merge); check-units: no findings. Extra: AGENTS.md "Lessons learned" derived-Deserialize entry updated to the `serde(try_from)` pattern.

Step 1 — `org-members/AGENTS.md`:
- Critical invariant 5 → "Wire-format leaves decode through each field's
  validating `Deserialize` (`Handle`/`Name`/`Surname` via `serde(try_from)`,
  `P2pDeviceSlots`, keys). `derive(Deserialize)` on `MemberLeaf` is safe only
  because every field validates; never derive it on a validated type itself."
- "Where to look first", `src/types.rs` line → "MemberId, keys, Handle/Name/Surname,
  MemberLeaf, RootHash".
- Domain vocabulary `MemberKey`/`DeviceKey` mentions: leave (out of scope).
Step 2 — `org-members/README.md:16`: "the NFC form `validate_handle` returns
for the handle" → "the NFC form `Handle::parse` produces for the handle".
Step 3 — `soup.md` `serde` row: append "`Handle`, `Name` and `Surname` decode
through `serde(try_from = \"String\")`, so their `parse` runs on the
deserialisation path; `MemberLeaf` derives its impls over those fields."
Step 4 — traceability: both units' `check-trace.sh` → no MISSING-TEST for
REQ-t46uad, LLR-f3zrwd, LLR-377ddr; `check-ids.sh`, `check-units.sh` → clean.
Step 5 — commit `docs(org-members): AGENTS, README and SOUP reflect the newtypes`.

---

## Gate follow-up — robustness tests (2026-10-04)

The first verify-before-merge gate found no abnormal-input test for
LLR-4czn8t, LLR-ub6dw9, LLR-377ddr. Added (task commit `840a5f6`, tests only),
red by temporary mutation of the guarded code, reverted before commit:

red -> green: newtype_debug_redacts_at_the_bound_and_beyond_ascii (LLR-4czn8t) — newtype Debug printing the value → failed "Handle(josé)" vs "Handle([REDACTED])"; reverted → pass.
red -> green: decoded_member_record_debug_redacts_pii (LLR-4czn8t) — MemberLeaf Debug printing handle → failed `Debug leaked "zelda"`; reverted → pass.
red -> green: handle_index_forgets_renamed_and_deleted_handles (LLR-ub6dw9) — `update_leaf` not removing the old handle-index entry → failed at `get_by_handle(&h("alice")).is_none()`; reverted → pass.
red -> green: handle_lookup_is_exact_not_confusable (LLR-ub6dw9) — `get_by_handle` falling back to the skeleton index → failed `"paypa1" matched "paypal"`; reverted → pass.
red -> green: tampered_member_record_is_refused_and_handle_is_bound_in_bytes (LLR-377ddr, LLR-68tka5) — (A) `TryFrom<String>` skipping parse → failed at the tampered decode; (B) `From<Handle> for String` constant → failed `assert_ne!`; reverted → pass.

Result: `cargo test -p org-members` 208 passed / 1 ignored (pre-existing); clippy clean.
Finding: under mutation B `member_set_root_is_pinned` stayed green — the root
hashes `canonical_bytes`, not the serde encoding — so the wire golden and the
new `assert_ne!` are the only guards on the serde bytes. Both pins are needed.

Carried, not re-reddened: LLR-5w2jx8, LLR-ub6dw9, LLR-mmst86, LLR-g6arcs keep
their pre-existing annotated tests, rewritten to the typed API in T4. The new
part of each amendment (taking `Handle`/`Name`/`Surname`) is enforced by the
compiler; there is no runtime behaviour for a red to show.

## Merge follow-up — base merge conflict (2026-10-04)

Merging local master (`582a0c2`, recalculate refusal: `recalculate()` on a
calculated trie returns `HashesAlreadyCalculated`) broke
`member_set_root_is_pinned`, which called `genesis(..).recalculate()`;
`genesis` already returns a calculated trie. Fix (task commit `ffbeca8`): read
`root_hash()` from the genesis trie directly. Test body edited because the API
changed on the base, not to match a value — `GOLDEN_ROOT` byte-for-byte
unchanged and still matched.
red -> green: member_set_root_is_pinned — failed `unwrap()` on `Err(HashesAlreadyCalculated)` after the base merge; passes after reading the root from genesis.
Result: `cargo test -p org-members --no-fail-fast` 211 passed / 1 ignored (pre-existing); clippy clean.

## Review round 1 fixes (2026-10-04)

Independent review (6a) returned 7 findings (2 code, 1 requirement, 4 record).
Docs (task commit `27a1089`): LLR-c5tzyp minted and assessed; derived LLRs
re-confirmed; error-order change recorded; CONTEXT terms; ADR/AGENTS tag-type
and org-node count corrected. Code (task commit `692e045`):

red -> green: insert_rejects_confusable_handle (+ genesis_rejects_confusables, update_rejects_confusable_handle, apply_delta_rejects_confusable_in_upsert, apply_delta_rejects_confusable_of_untouched_member_after_release) (LLR-5w2jx8) — `HandleSkeleton::of` made identity → failed at the ConfusableHandle assertion (old circular helper instead panicked "no confusable pair found"); reverted → pass.
red -> green: genesis_multiple_members (LLR-5w2jx8, over-match) — `HandleSkeleton::of` constant → failed `unwrap()` on `Err(ConfusableHandle)`; reverted → pass.
red -> green: update_name_surname_unknown_id_leaves_trie_unchanged (LLR-g6arcs) — `update_name_surname` returning `DuplicateId` → failed "DuplicateId vs IdNotFound"; reverted → pass.
red -> green: newtype_deliberate_output_shows_value (LLR-c5tzyp) — Display writing "[REDACTED]" → failed vs "alice Alice Smith"; reverted → pass.
red -> green: newtype_deliberate_output_is_nfc_form (LLR-c5tzyp) — same mutation → failed vs NFC values; reverted → pass.
Re-pointed, not re-reddened (behaviour unchanged): handle_for_update_rejects_invalid (was update_handle_rejects_invalid; now LLR-xzqs9r), name_for_update_rejects_oversized / surname_for_update_rejects_oversized (were update_name_surname_rejects_oversized_*; now LLR-w5nkbu). Dated history (docs/plans/2026-09-15-org-members-architecture.md, docs/verification/2026-09-17-*.md) keeps the old names.
Result: org-members all green (integration 160, newtypes 10, mbt 32/1 ignored, fuzz 9, golden 3); clippy clean; org-node cargo line green.

## Review round 2 fixes (2026-10-04)

Round 2 returned 9 findings (2 requirement, 3 code, 4 record), none
functional. Docs (task commit `f701799`): three error-order changes recorded
(`MemberLeaf::new`, `update_handle`, `update_name_surname`); LLR-c5tzyp covers
serde `Serialize`; LLR-yz2gmc defined and assessed; API removals listed; ADR
status note; CONTEXT "Newtype". Plan: Implements/self-review list
LLR-c5tzyp, LLR-yz2gmc. Tests (task commit `3a86330`, tests only):

red -> green: tag_types_construct_from_bytes (LLR-yz2gmc) — `RootHash::from` zeroing byte 0 → failed at [0xff;32] (00ffffff.. vs ffffffff..); reverted → pass.
red -> green: decode_member_record_stores_nfc_fields (LLR-68tka5) — behaviour pre-exists, no src mutation can red it without breaking decode; red shown by writing the NFD expectation first → failed (left "josé" NFC vs right NFD), corrected to NFC → pass. Weaker evidence than a mutation, stated as such.
Re-annotated, behaviour unchanged: handle_lookup_is_exact_not_confusable → LLR-f3zrwd (its round-1 red, skeleton-index fallback in `get_by_handle`, is a lookup fault); decode_parses_newtype_fields → LLR-xzqs9r, LLR-w5nkbu; deserialize_rejects_oversized_name/_surname gain LLR-68tka5.
Result: org-members 216 passed / 1 ignored (pre-existing); clippy clean.

## Review round 3 fixes (2026-10-04)

Round 3 returned 4 findings (1 code — test gap, no defect; 3 record). Owner
chose to fix all and run a round-4 reviewer. Docs (task commit `475cf4b`):
fourth error-order change (org-node `trie_from_snapshots`, reached by local
store load and by `first_admission_base` decoding a received snapshot)
recorded in SAD and RMF; CONTEXT "Newtype" reworded. Tests (task commit
`86c3c80`, tests only):

red -> green: name_and_surname_try_from_match_parse (LLR-w5nkbu) — macro `TryFrom<&str>` skipping parse → failed (NFD stored, `try_from` != `parse`); reverted → pass.
red -> green: newtypes_serialize_as_stored_nfc_string (LLR-c5tzyp) — `From<Surname> for String` returning a constant → failed (bytes [8,"constant"] vs [5,"José"]); reverted → pass.
Result: org-members 218 passed / 1 ignored (pre-existing); clippy clean.

## Review round 4 fixes (2026-10-04)

Round 4 returned 4 findings (1 requirement — test gap, no defect; 3 record).
Owner chose to fix all and run a round-5 reviewer. Docs (task commit
`533520a`): fifth error-order change (MemberLeaf decode now fails at the first
invalid field) recorded, lists scoped to "identified by review";
LLR-xzqs9r `satisfies: REQ-h5ret5, REQ-t46uad`; LLR-yz2gmc narrowed to
`MemberId`; LLR-v9evtu (new, `NodeHash`/`RootHash`, under SDD-d6x85b) defined
and assessed. Tests (task commit `5ee60b7`, tests only):

red -> green: update_name_surname_stores_boundary_and_nfd_fields_only (LLR-g6arcs) — `with_name_surname` keeping the old surname → failed ("Smith" vs "Müller"); keeping the old name → failed ("Alice" vs 128-byte name); each reverted → pass.
Re-annotated, behaviour unchanged: update_name_surname_unknown_id_leaves_trie_unchanged → LLR-v3jqau; tag_types_construct_from_bytes → LLR-yz2gmc, LLR-v9evtu.
Result: org-members 219 passed / 1 ignored (pre-existing); clippy clean.

## Review round 5 fixes (2026-10-04) — last review round (owner ruling)

Round 5 returned 5 findings (1 requirement, 1 code — test gap, 3 record).
Owner rulings: amend REQ-h5ret5 now; fix all, then close review (no round 6).
Docs (task commit `1bc2b25`): REQ-h5ret5, LLR-xzqs9r and RC-n2taat amended
in place to state the UTS#39 identifier-character rule the code has applied
since 2026-05-12 (assessed); LLR-v9evtu moved under SDD-4yr9ge (owner of
`types.rs`); stale `validate_handle` passages in the 2026-09-17 SAD and
design-derived RMF updated. Plan self-review lists rounds 3–5 tests. Tests
(task commit `115aca0`, tests only):

red -> green: handle_parse_rejects_non_identifier_characters (LLR-xzqs9r) — `identifier_allowed` branch deleted → failed `accepted "a b"`; restored → pass. ('_' is accepted by UTS#39 and sits on the accepted side.)
red -> green: length_bounds_apply_after_nfc (LLR-xzqs9r, LLR-w5nkbu) — Handle bound measured on raw input → failed InvalidHandle on the 192-raw/128-NFC case; `nfc_bounded` on raw input → failed FieldTooLong for Name; each restored → pass.
Result: org-members 221 passed / 1 ignored (pre-existing); clippy clean.
Out of scope, noted: `cargo fmt --check` fails crate-wide on pre-existing code; no gate runs it.

## Self-review

1. Implements → tests: REQ-t46uad, LLR-f3zrwd (T4
   `handle_query_reports_invalid_absent_and_held`); LLR-377ddr (T1 both tests,
   T2 `newtypes_encode_as_their_string`); LLR-xzqs9r, LLR-w5nkbu, LLR-4czn8t,
   LLR-68tka5 (T2); LLR-5w2jx8, LLR-ub6dw9, LLR-mmst86, LLR-g6arcs keep their
   existing annotated tests, rewritten in T4 to the typed API (confusable,
   index and update tests in `integration_test.rs`). Added after review:
   LLR-c5tzyp (`newtype_deliberate_output_shows_value`,
   `newtype_deliberate_output_is_nfc_form`, round 1); LLR-yz2gmc
   (`tag_types_construct_from_bytes`, round 2); LLR-f3zrwd also carried by
   `handle_lookup_is_exact_not_confusable` (re-annotated, round 2);
   LLR-c5tzyp also `newtypes_serialize_as_stored_nfc_string` and LLR-w5nkbu
   also `name_and_surname_try_from_match_parse` (round 3); LLR-g6arcs
   `update_name_surname_stores_boundary_and_nfd_fields_only` and LLR-v9evtu
   `tag_types_construct_from_bytes` (round 4); LLR-xzqs9r
   `handle_parse_rejects_non_identifier_characters` and LLR-xzqs9r/LLR-w5nkbu
   `length_bounds_apply_after_nfc` (round 5).
2. Code steps carry real code and expected output; the test rewrite is a
   closed table of patterns with the one non-mechanical helper written out.
3. Names consistent: `Handle::parse`, `Name::parse`, `Surname::parse`,
   `HandleSkeleton::of`, `HeldKey::from`, `RootHash::new`, `with_handle`,
   `with_name_surname`, `get_by_handle(&Handle)`, `contains_handle(&Handle)`.
4. Parallel only T1 ∥ T2: file sets `{encoding_golden.rs}` and
   `{types.rs, lib.rs, newtypes.rs}` are disjoint. All others serial.

## Risks to watch while executing

- **Error order at construction.** Field errors now surface at `parse`, before
  `MemberLeaf::new`'s `EmptyDeviceList` check. A test or the mbt driver that
  expects `EmptyDeviceList` for a leaf that also has an invalid handle would
  change outcome. None known; if `mbt_conformance` reports a mismatch, stop
  and report rather than reorder.
- **Golden test edited to pass.** Only the LEAF-CONSTRUCTION function may
  change in T4. Any change to the constants is a stop-and-report.
