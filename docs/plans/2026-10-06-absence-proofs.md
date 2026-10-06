# Absence Proofs (S1) Implementation Plan

**Goal:** Let any device holding the current members trie produce, and a revoked device verify against the on-chain root, a proof that its Member or its Device key is absent, without panics anywhere on the path.
**Implements:** REQ-535jcd, REQ-yyuxh8, REQ-tk2qqj, RC-j2znx8, RC-qa2758, SDD-57vaj4 (LLR-4xz255, LLR-25tpdp, LLR-4rju5r, LLR-dgzy7e, LLR-p2p8qy, LLR-2dcnbp), SDD-d9svdj (LLR-utp6x4, LLR-7jkcba); resolves PR-jq43gx.
**Safety class:** C (org-members, no per-item override).
**Verification:** `org-members/.guardrails/config.yaml` `verify_commands`: `cargo test -p org-members`, `quint --version`, the three `quint typecheck` commands, `quint test org-members/quint/membership.qnt --backend=typescript`, `quint run org-members/quint/membership_mbt.qnt --invariant=mbtInv --max-steps=15 --max-samples=1000`. Set `QUINT_HOME` to a writable directory, such as `$TMPDIR/quint_home`. On a fresh `QUINT_HOME`, the first `mbt_conformance` run fails while the evaluator downloads; run it again. Every task also runs `cargo clippy -p org-members --all-targets -- -D warnings` and `cargo build -p org-members --no-default-features` (the no_std + alloc build).

Context: roadmap `docs/plans/2026-10-06-org-io-roadmap.md` (stage S1). Requirements: `org-members/docs/requirements/2026-10-06-absence-proofs.md`. Risk: `org-members/docs/risk/2026-10-06-absence-proofs.md`. Design: `org-members/docs/architecture/2026-10-06-absence-proofs.md`.

Quint: no impact. The model covers membership operations and change sets, and this change adds neither. The conformance suite must stay green, unchanged.

Path conventions used in every task (from `org-members/src/smt.rs`):

- depth `d` counts from the root (0) to the leaf (256). Level `l = 256 - d` counts from the leaf (0) to the root (256). The sibling met at depth `d` (0..=255) sits at level `255 - d`.
- the bit that steers at depth `d` is the identifier's bit `d`, most significant first (LLR-zbe553).
- the default-sibling map: bit `l` of the 32-byte map is `map[l / 8] >> (l % 8) & 1`, which is set when the sibling at level `l` is that level's default hash.

---

### T1 — PR-jq43gx: `bit` and `at_level` return errors, the store walks with `path_bits`

**Files touched:** org-members/src/error.rs, org-members/src/types.rs, org-members/src/smt.rs, org-members/src/trie.rs, org-members/tests/integration_test.rs, org-members/docs/problems/2026-09-17-review-fixes.md
**Parallel:** no (first)

1. Write the reproduction tests in `org-members/tests/integration_test.rs`, directly after `member_id_bit_indexes_msb_first`. They compile against the current API (each closure returns whatever the method returns), and fail today because the calls panic:

```rust
/// verifies: LLR-7jkcba
///
/// PR-jq43gx's reproduction: an index the type admits must not panic.
#[test]
fn member_id_bit_out_of_range_does_not_panic() {
    let id = MemberId::new([0xff; 32]);
    let outcome = std::panic::catch_unwind(|| {
        let _ = id.bit(256);
        let _ = id.bit(u16::MAX);
    });
    assert!(outcome.is_ok(), "MemberId::bit panicked on an out-of-range index (PR-jq43gx)");
}

/// verifies: LLR-7jkcba
///
/// PR-jq43gx's second site.
#[test]
fn default_hashes_at_level_out_of_range_does_not_panic() {
    let defaults = org_members::smt::DefaultHashes::compute::<Blake3Hasher>();
    let outcome = std::panic::catch_unwind(|| {
        let _ = defaults.at_level(257).is_ok();
        let _ = defaults.at_level(u16::MAX).is_ok();
    });
    assert!(outcome.is_ok(), "DefaultHashes::at_level panicked on an out-of-range level (PR-jq43gx)");
}
```

   The second closure calls `.is_ok()`, which does not compile against today's `&NodeHash`. For the red run, write that closure as `let _ = defaults.at_level(257);` and `let _ = defaults.at_level(u16::MAX);`, then restore the `.is_ok()` form at step 4.

2. Run it and watch it fail:

```
cargo test -p org-members --test integration_test out_of_range_does_not_panic
```

   Expected: `2 failed`, each panic message an index out of bounds (`index out of bounds: the len is 32 but the index is 32`, and the same with `len is 257`). Record the output as the red evidence.

3. Implement.

   `org-members/src/error.rs`: add, before `InvariantViolated`:

```rust
    /// A bit index of 256 or more given to `MemberId::bit`, or a level of 257
    /// or more given to `DefaultHashes::at_level`. LLR-7jkcba (PR-jq43gx).
    #[error("index out of range")]
    IndexOutOfRange,
```

   `org-members/src/types.rs`: replace `MemberId::bit` and add `path_bits`:

```rust
    /// Returns the bit at the given index (0 = MSB of byte 0, 255 = LSB of
    /// byte 31), or `IndexOutOfRange` for an index of 256 or more
    /// (LLR-zbe553, LLR-7jkcba).
    pub fn bit(&self, index: u16) -> Result<bool, OrgMembersError> {
        let byte = self
            .0
            .get(usize::from(index / 8))
            .ok_or(OrgMembersError::IndexOutOfRange)?;
        Ok((byte >> (7 - (index % 8))) & 1 == 1)
    }

    /// The identifier's 256 bits, most significant first: the path the
    /// store walks (LLR-zbe553). It cannot go out of range (LLR-7jkcba).
    pub(crate) fn path_bits(&self) -> impl Iterator<Item = bool> + '_ {
        self.0
            .iter()
            .flat_map(|byte| (0..8u8).rev().map(move |i| (byte >> i) & 1 == 1))
    }
```

   `org-members/src/smt.rs`:

```rust
    /// The default hash at `level` (0 = a leaf, 256 = the root), or
    /// `IndexOutOfRange` for a level of 257 or more (LLR-wm5hpc, LLR-7jkcba).
    pub fn at_level(&self, level: u16) -> Result<&NodeHash, OrgMembersError> {
        self.hashes
            .get(usize::from(level))
            .ok_or(OrgMembersError::IndexOutOfRange)
    }
```

```rust
pub fn empty_root(defaults: &DefaultHashes) -> Result<Arc<Node>, OrgMembersError> {
    Ok(Arc::new(Node::empty(*defaults.at_level(SMT_DEPTH)?)))
}
```

   `insert` and `remove` return `Result<Arc<Node>, OrgMembersError>`, and pass `id.path_bits()` to `insert_at`:

```rust
pub fn insert<H: TrieHasher>(
    root: &Arc<Node>,
    member: MemberLeaf,
    defaults: &DefaultHashes,
) -> Result<Arc<Node>, OrgMembersError> {
    let id = *member.id();
    let device_root = compute_device_root::<H>(member.p2p_device_slots());
    let new_leaf = Arc::new(Node::leaf(member, device_root));
    insert_at(root, &mut id.path_bits(), new_leaf, 0, defaults)
}

pub fn remove(
    root: &Arc<Node>,
    id: &MemberId,
    defaults: &DefaultHashes,
) -> Result<Arc<Node>, OrgMembersError> {
    let empty_leaf = Arc::new(Node::empty(*defaults.empty_leaf()));
    insert_at(root, &mut id.path_bits(), empty_leaf, 0, defaults)
}

fn insert_at(
    node: &Arc<Node>,
    bits: &mut dyn Iterator<Item = bool>,
    new_leaf: Arc<Node>,
    depth: u16,
    defaults: &DefaultHashes,
) -> Result<Arc<Node>, OrgMembersError> {
    if depth == SMT_DEPTH {
        return Ok(new_leaf);
    }
    let go_right = bits.next().ok_or(OrgMembersError::InvariantViolated)?;
    let (left, right) = match &node.kind {
        NodeKind::Internal { left, right } => (left.clone(), right.clone()),
        NodeKind::Empty | NodeKind::Leaf(_) => {
            let default_child = Arc::new(Node::empty(*defaults.at_level(SMT_DEPTH - depth - 1)?));
            (default_child.clone(), default_child)
        }
    };
    let (new_left, new_right) = if go_right {
        (left, insert_at(&right, bits, new_leaf, depth + 1, defaults)?)
    } else {
        (insert_at(&left, bits, new_leaf, depth + 1, defaults)?, right)
    };
    Ok(Arc::new(Node::internal(new_left, new_right)))
}
```

   `get_member` keeps its signature and steers with `path_bits`:

```rust
pub fn get_member(root: &Arc<Node>, id: &MemberId) -> Option<MemberLeaf> {
    let mut current = root.clone();
    for go_right in id.path_bits() {
        match &current.kind {
            NodeKind::Internal { left, right } => {
                current = if go_right { right.clone() } else { left.clone() };
            }
            NodeKind::Empty | NodeKind::Leaf(_) => return None,
        }
    }
    match &current.kind {
        NodeKind::Leaf(payload) => Some(payload.member.clone()),
        _ => None,
    }
}
```

   `org-members/src/trie.rs`: append `?` to every `smt::empty_root(…)`, `smt::insert::<H>(…)` and `smt::remove(…)` call (lines 98, 124, 402, 458, 476, 695, 733 at the time of writing). Each sits in a function that already returns `Result<_, OrgMembersError>`. Run `grep -n "smt::\(empty_root\|insert\|remove\)" org-members/src/trie.rs` and confirm every hit ends in `)?`.

   `org-members/tests/integration_test.rs`: in `member_id_bit_indexes_msb_first`, the four assertions become `assert_eq!(id.bit(0), Ok(true), …)`, `assert_eq!(id.bit(1), Ok(false))`, `assert_eq!(id.bit(255), Ok(true), …)`, `assert_eq!(id.bit(254), Ok(false))`. Restore step 1's `.is_ok()` form.

4. Add the exact-value test after the reproduction tests:

```rust
/// verifies: LLR-7jkcba
#[test]
fn out_of_range_path_index_is_index_out_of_range() {
    let id = MemberId::new([0xff; 32]);
    assert_eq!(id.bit(255), Ok(true));
    assert_eq!(id.bit(256), Err(OrgMembersError::IndexOutOfRange));
    assert_eq!(id.bit(u16::MAX), Err(OrgMembersError::IndexOutOfRange));

    let defaults = org_members::smt::DefaultHashes::compute::<Blake3Hasher>();
    assert!(defaults.at_level(256).is_ok());
    assert!(matches!(defaults.at_level(257), Err(OrgMembersError::IndexOutOfRange)));
    assert!(matches!(defaults.at_level(u16::MAX), Err(OrgMembersError::IndexOutOfRange)));
}
```

5. Run it: `cargo test -p org-members`. Expected: every suite `ok`, including the three new tests and the 32 conformance tests (with `QUINT_HOME` set). Also run `cargo clippy -p org-members --all-targets -- -D warnings` (expect exit 0) and `cargo build -p org-members --no-default-features` (expect exit 0).

6. Resolve PR-jq43gx in `org-members/docs/problems/2026-09-17-review-fixes.md`: change `status: open` to `status: resolved`, and add directly under the status line:

```
resolution: `MemberId::bit` and `DefaultHashes::at_level` returned an unchecked index into a fixed array; both now return `IndexOutOfRange` (LLR-7jkcba), and the store steers by `MemberId::path_bits`. Reproduced by `member_id_bit_out_of_range_does_not_panic` and `default_hashes_at_level_out_of_range_does_not_panic` (`org-members/tests/integration_test.rs`).
```

7. Commit: `fix(org-members): PR-jq43gx — bit and at_level return IndexOutOfRange (LLR-7jkcba)`.

---

### T2 — Path walk, the proof type and `prove_absent`

**Files touched:** org-members/src/error.rs, org-members/src/smt.rs, org-members/src/proof.rs, org-members/src/lib.rs, org-members/src/trie.rs, org-members/tests/absence_proofs.rs
**Parallel:** no (serial, after T1)

1. Create `org-members/tests/absence_proofs.rs` with the fixtures and the production tests:

```rust
mod common;

use common::{device_key, member_key};
use org_members::hasher::Blake3Hasher;
use org_members::proof::ProofEnding;
use org_members::trie::OrgTrie;
use org_members::types::{Handle, MemberId, MemberLeaf, Name, Surname};
use org_members::OrgMembersError;

type TestTrie = OrgTrie<Blake3Hasher>;

fn id(seed: &str) -> MemberId {
    MemberId::new(blake3::hash(seed.as_bytes()).into())
}

fn leaf(name: &str, devices: &[&str]) -> MemberLeaf {
    MemberLeaf::new(
        id(name),
        Handle::parse(name).unwrap(),
        member_key(&format!("{name}-mk")),
        Name::parse("Test").unwrap(),
        Surname::parse("User").unwrap(),
        devices.iter().map(|d| device_key(d)).collect(),
    )
    .unwrap()
}

/// A calculated three-member organisation.
fn org() -> TestTrie {
    TestTrie::genesis(vec![
        leaf("alice", &["alice-d1", "alice-d2"]),
        leaf("bob", &["bob-d1"]),
        leaf("carol", &["carol-d1"]),
    ])
    .unwrap()
}

/// `org()` after alice's second device is removed and recalculated.
fn org_without_alice_d2() -> TestTrie {
    let changed = org()
        .delete_p2p_device(&id("alice"), &device_key("alice-d2"), member_key("alice-mk2"))
        .unwrap();
    changed.recalculate().unwrap().0
}

fn popcount(map: &[u8; 32]) -> usize {
    map.iter().map(|b| b.count_ones() as usize).sum()
}

/// verifies: LLR-4xz255, REQ-535jcd
#[test]
fn prove_absent_refuses_an_uncalculated_trie() {
    let pending = org().add_member(leaf("dave", &["dave-d1"])).unwrap();
    assert_eq!(
        pending.prove_absent(&id("zed"), &device_key("zed-d1")).unwrap_err(),
        OrgMembersError::HashesNotCalculated
    );
}

/// verifies: LLR-4xz255, LLR-2dcnbp, REQ-535jcd
#[test]
fn prove_absent_refuses_a_device_the_member_holds() {
    assert_eq!(
        org().prove_absent(&id("alice"), &device_key("alice-d1")).unwrap_err(),
        OrgMembersError::DeviceStillHeld
    );
}

/// verifies: LLR-4xz255, REQ-535jcd
///
/// A key held by a different Member does not stop the proof (owner ruling
/// 2026-10-06; the never-again rule belongs to the unit holding the
/// revoked-device list).
#[test]
fn prove_absent_ignores_a_key_another_member_holds() {
    assert!(org().prove_absent(&id("alice"), &device_key("bob-d1")).is_ok());
}

/// verifies: LLR-25tpdp, REQ-yyuxh8
#[test]
fn proof_for_an_absent_member_carries_no_member_data() {
    let proof = org().prove_absent(&id("zed"), &device_key("zed-d1")).unwrap();
    assert_eq!(proof.ending(), &ProofEnding::Empty);
    assert_eq!(proof.siblings().len() + popcount(proof.default_map()), 256);
}

/// verifies: LLR-25tpdp, REQ-yyuxh8
#[test]
fn proof_for_a_removed_device_carries_only_that_member() {
    let trie = org_without_alice_d2();
    let proof = trie.prove_absent(&id("alice"), &device_key("alice-d2")).unwrap();
    match proof.ending() {
        ProofEnding::Leaf(member) => {
            assert_eq!(member.id(), &id("alice"));
            assert_eq!(member, &trie.get(&id("alice")).unwrap());
        }
        ProofEnding::Empty => panic!("expected alice's leaf"),
    }
    assert_eq!(proof.siblings().len() + popcount(proof.default_map()), 256);
}
```

2. Run `cargo test -p org-members --test absence_proofs`. Expected: compile errors, unresolved `org_members::proof` and no method `prove_absent`. Record them as the red evidence.

3. Implement.

   `org-members/src/error.rs`, after `IndexOutOfRange`:

```rust
    /// The Member under an absence proof's MemberId holds the Device key.
    /// LLR-4xz255, LLR-p2p8qy.
    #[error("device key still held by the member")]
    DeviceStillHeld,

    /// An absence proof whose sibling hashes do not match its default map.
    /// LLR-4rju5r.
    #[error("malformed absence proof")]
    AbsenceProofMalformed,

    /// An absence proof that does not resolve to the root it was checked
    /// against. LLR-dgzy7e.
    #[error("absence proof does not resolve to the root")]
    AbsenceProofRootMismatch,
```

   `org-members/src/smt.rs`, after `get_member`:

```rust
/// What a member's path ends in (LLR-utp6x4).
pub(crate) enum PathEnd {
    Empty,
    Leaf(MemberLeaf),
}

/// The 256 sibling hashes along `id`'s path, index `l` holding the sibling at
/// level `l` (0 = the leaf's level), and what the path ends in. Below an empty
/// subtree every sibling is its level's default (LLR-wm5hpc). Refuses with
/// `HashesNotCalculated` when a sibling's hash is unset (LLR-utp6x4).
pub(crate) fn path(
    root: &Arc<Node>,
    id: &MemberId,
    defaults: &DefaultHashes,
) -> Result<(Vec<NodeHash>, PathEnd), OrgMembersError> {
    let mut top_down: Vec<NodeHash> = Vec::with_capacity(usize::from(SMT_DEPTH));
    let mut bits = id.path_bits();
    let mut current = root.clone();
    let mut depth: u16 = 0;
    let end = loop {
        if depth == SMT_DEPTH {
            break match &current.kind {
                NodeKind::Leaf(payload) => PathEnd::Leaf(payload.member.clone()),
                NodeKind::Empty => PathEnd::Empty,
                NodeKind::Internal { .. } => return Err(OrgMembersError::InvariantViolated),
            };
        }
        match &current.kind {
            NodeKind::Internal { left, right } => {
                let go_right = bits.next().ok_or(OrgMembersError::InvariantViolated)?;
                let (next, sibling) = if go_right { (right, left) } else { (left, right) };
                top_down.push(*sibling.hash().ok_or(OrgMembersError::HashesNotCalculated)?);
                current = next.clone();
                depth += 1;
            }
            NodeKind::Empty => {
                while depth < SMT_DEPTH {
                    top_down.push(*defaults.at_level(SMT_DEPTH - depth - 1)?);
                    depth += 1;
                }
                break PathEnd::Empty;
            }
            NodeKind::Leaf(_) => return Err(OrgMembersError::InvariantViolated),
        }
    };
    top_down.reverse();
    Ok((top_down, end))
}
```

   Add `use alloc::vec::Vec;` to `smt.rs` if it is not already imported (it is, for `collect_members`).

   Create `org-members/src/proof.rs`:

```rust
//! Absence proofs (SDD-57vaj4): a proof that a MemberId holds no Member, or
//! that the Member it holds lacks a Device key, checked against a membership
//! root. The only code that interprets a leaf for this purpose.

use alloc::vec::Vec;

use crate::error::OrgMembersError;
use crate::smt::{DefaultHashes, PathEnd, SMT_DEPTH};
use crate::types::{MemberLeaf, NodeHash};

/// What an absence proof's path ends in (LLR-25tpdp).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofEnding {
    /// The MemberId holds no Member: the proof carries no Member data.
    Empty,
    /// The Member under the proof's MemberId, and no one else.
    Leaf(MemberLeaf),
}

/// A proof of absence (LLR-25tpdp). Fields are private; the only ways to get
/// one are `OrgTrie::prove_absent` and parsing (LLR-4rju5r).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsenceProof {
    /// Bit `l` set: the sibling at level `l` is that level's default hash.
    default_map: [u8; 32],
    /// The non-default siblings, from the leaf's level up.
    siblings: Vec<NodeHash>,
    ending: ProofEnding,
}

pub(crate) fn is_default(map: &[u8; 32], level: u16) -> bool {
    map.get(usize::from(level / 8))
        .is_some_and(|byte| (byte >> (level % 8)) & 1 == 1)
}

impl AbsenceProof {
    /// Builds a proof from a path walk (LLR-utp6x4), sending only the
    /// siblings that are not their level's default (LLR-25tpdp).
    pub(crate) fn from_path(
        siblings: Vec<NodeHash>,
        end: PathEnd,
        defaults: &DefaultHashes,
    ) -> Result<Self, OrgMembersError> {
        let mut default_map = [0u8; 32];
        let mut explicit = Vec::new();
        for (level, hash) in (0..SMT_DEPTH).zip(siblings) {
            if hash == *defaults.at_level(level)? {
                let byte = default_map
                    .get_mut(usize::from(level / 8))
                    .ok_or(OrgMembersError::InvariantViolated)?;
                *byte |= 1 << (level % 8);
            } else {
                explicit.push(hash);
            }
        }
        let ending = match end {
            PathEnd::Empty => ProofEnding::Empty,
            PathEnd::Leaf(member) => ProofEnding::Leaf(member),
        };
        Ok(Self { default_map, siblings: explicit, ending })
    }

    /// The default-sibling map.
    pub fn default_map(&self) -> &[u8; 32] {
        &self.default_map
    }

    /// The siblings that are not their level's default, from the leaf's
    /// level up.
    pub fn siblings(&self) -> &[NodeHash] {
        &self.siblings
    }

    /// What the path ends in.
    pub fn ending(&self) -> &ProofEnding {
        &self.ending
    }
}
```

   `org-members/src/lib.rs`: add `pub mod proof;` after `pub mod normalize;`, and `pub use proof::{AbsenceProof, ProofEnding};` after `pub use hasher::TrieHasher;`.

   `org-members/src/trie.rs`: add `use crate::proof::AbsenceProof;` and, after `get_by_handle`:

```rust
    /// A proof that `device` is absent under `id`: `id` holds no Member, or
    /// the Member it holds lacks `device`. Refuses with `HashesNotCalculated`
    /// on an uncalculated trie and with `DeviceStillHeld` when that Member
    /// holds `device`. A key another Member holds does not refuse
    /// (LLR-4xz255).
    pub fn prove_absent(
        &self,
        id: &MemberId,
        device: &DevicePublicKey,
    ) -> Result<AbsenceProof, OrgMembersError> {
        if !self.is_calculated() {
            return Err(OrgMembersError::HashesNotCalculated);
        }
        let (siblings, end) = smt::path(&self.root, id, &self.defaults)?;
        if let smt::PathEnd::Leaf(member) = &end {
            if member.has_p2p_device(device) {
                return Err(OrgMembersError::DeviceStillHeld);
            }
        }
        AbsenceProof::from_path(siblings, end, &self.defaults)
    }
```

   (`DevicePublicKey` is already imported in `trie.rs`; if the compiler says otherwise, add it to the `crate::types` import.)

4. Run `cargo test -p org-members --test absence_proofs`. Expected: `5 passed`. Then run `cargo test -p org-members`, clippy and the no_std build as in T1, expecting all green.

5. Commit: `feat(org-members): absence proof type and prove_absent (LLR-utp6x4, LLR-4xz255, LLR-25tpdp)`.

---

### T3 — `verify`

**Files touched:** org-members/src/proof.rs, org-members/tests/absence_proofs.rs
**Parallel:** no (serial, after T2)

1. Append to `org-members/tests/absence_proofs.rs`:

```rust
/// verifies: LLR-utp6x4, LLR-dgzy7e, LLR-p2p8qy, REQ-tk2qqj
#[test]
fn proof_for_an_absent_member_verifies() {
    let trie = org();
    let root = trie.root_hash().unwrap();
    let proof = trie.prove_absent(&id("zed"), &device_key("zed-d1")).unwrap();
    assert_eq!(proof.verify::<Blake3Hasher>(&root, &id("zed"), &device_key("zed-d1")), Ok(()));
}

/// verifies: LLR-utp6x4, LLR-dgzy7e, LLR-p2p8qy, REQ-tk2qqj
///
/// A deleted member's path ends in an empty leaf at depth 256, not an empty
/// subtree higher up: the walk must handle both.
#[test]
fn proof_for_a_deleted_member_verifies() {
    let trie = org().delete_member(&id("bob")).unwrap().recalculate().unwrap().0;
    let root = trie.root_hash().unwrap();
    let proof = trie.prove_absent(&id("bob"), &device_key("bob-d1")).unwrap();
    assert_eq!(proof.ending(), &ProofEnding::Empty);
    assert_eq!(proof.verify::<Blake3Hasher>(&root, &id("bob"), &device_key("bob-d1")), Ok(()));
}

/// verifies: LLR-utp6x4, LLR-dgzy7e, LLR-p2p8qy, REQ-tk2qqj
#[test]
fn proof_for_a_removed_device_verifies() {
    let trie = org_without_alice_d2();
    let root = trie.root_hash().unwrap();
    let proof = trie.prove_absent(&id("alice"), &device_key("alice-d2")).unwrap();
    assert_eq!(proof.verify::<Blake3Hasher>(&root, &id("alice"), &device_key("alice-d2")), Ok(()));
}

/// verifies: LLR-dgzy7e, LLR-2dcnbp, REQ-tk2qqj, RC-j2znx8
///
/// A proof genuine under the new root fails against the old one: the
/// stale-root case org-io must prevent (HAZ-adm7gv) is at least never
/// accepted across roots.
#[test]
fn proof_is_refused_against_another_root() {
    let old_root = org().root_hash().unwrap();
    let trie = org_without_alice_d2();
    let proof = trie.prove_absent(&id("alice"), &device_key("alice-d2")).unwrap();
    assert_eq!(
        proof.verify::<Blake3Hasher>(&old_root, &id("alice"), &device_key("alice-d2")),
        Err(OrgMembersError::AbsenceProofRootMismatch)
    );
}

/// verifies: LLR-dgzy7e, LLR-2dcnbp, REQ-tk2qqj
#[test]
fn proof_is_refused_for_another_member_id() {
    let trie = org();
    let root = trie.root_hash().unwrap();
    let proof = trie.prove_absent(&id("zed"), &device_key("zed-d1")).unwrap();
    assert_eq!(
        proof.verify::<Blake3Hasher>(&root, &id("bob"), &device_key("zed-d1")),
        Err(OrgMembersError::AbsenceProofRootMismatch)
    );
}

/// verifies: LLR-p2p8qy, LLR-2dcnbp, REQ-tk2qqj
///
/// The root matches, but the leaf holds the device asked about.
#[test]
fn proof_is_refused_when_the_member_holds_the_device() {
    let trie = org();
    let root = trie.root_hash().unwrap();
    let proof = trie.prove_absent(&id("alice"), &device_key("alice-d9")).unwrap();
    assert_eq!(
        proof.verify::<Blake3Hasher>(&root, &id("alice"), &device_key("alice-d1")),
        Err(OrgMembersError::DeviceStillHeld)
    );
}
```

2. Run `cargo test -p org-members --test absence_proofs`. Expected: compile error, no method `verify` on `AbsenceProof`. That is the red evidence.

3. Implement in `org-members/src/proof.rs`. Add the imports `use person::compute_device_root;`, `use crate::hasher::TrieHasher;` and `use crate::types::{DevicePublicKey, MemberId, RootHash};`, then add to `impl AbsenceProof`:

```rust
    /// Accepts the proof only when it resolves to `root` along `id`'s path
    /// (LLR-dgzy7e) and its ending shows `device` absent (LLR-p2p8qy).
    pub fn verify<H: TrieHasher>(
        &self,
        root: &RootHash,
        id: &MemberId,
        device: &DevicePublicKey,
    ) -> Result<(), OrgMembersError> {
        let defaults = DefaultHashes::compute::<H>();
        let mut running = match &self.ending {
            ProofEnding::Empty => *defaults.empty_leaf(),
            ProofEnding::Leaf(member) => {
                let device_root = compute_device_root::<H>(member.p2p_device_slots());
                H::hash_member_leaf(&member.canonical_bytes(&device_root))
            }
        };
        let steering: Vec<bool> = id.path_bits().collect();
        let mut explicit = self.siblings.iter();
        for level in 0..SMT_DEPTH {
            let sibling = if is_default(&self.default_map, level) {
                *defaults.at_level(level)?
            } else {
                *explicit.next().ok_or(OrgMembersError::AbsenceProofMalformed)?
            };
            let depth = usize::from(SMT_DEPTH - 1 - level);
            let went_right = *steering.get(depth).ok_or(OrgMembersError::InvariantViolated)?;
            running = if went_right {
                H::hash_member_node(&sibling, &running)
            } else {
                H::hash_member_node(&running, &sibling)
            };
        }
        if explicit.next().is_some() {
            return Err(OrgMembersError::AbsenceProofMalformed);
        }
        if running.as_bytes() != root.as_bytes() {
            return Err(OrgMembersError::AbsenceProofRootMismatch);
        }
        if let ProofEnding::Leaf(member) = &self.ending {
            if member.has_p2p_device(device) {
                return Err(OrgMembersError::DeviceStillHeld);
            }
        }
        Ok(())
    }
```

4. Run `cargo test -p org-members --test absence_proofs`. Expected: `11 passed`. Then run the full `cargo test -p org-members`, clippy and the no_std build, all green.

5. Commit: `feat(org-members): AbsenceProof::verify (LLR-dgzy7e, LLR-p2p8qy)`.

---

### T4 — Parsing: `from_parts` and the bounded serde form

**Files touched:** org-members/src/proof.rs, org-members/tests/absence_proofs.rs
**Parallel:** no (serial, after T3)

1. Append to `org-members/tests/absence_proofs.rs`:

```rust
use org_members::proof::AbsenceProof;
use org_members::types::NodeHash;

/// verifies: LLR-4rju5r, LLR-2dcnbp
#[test]
fn from_parts_refuses_a_count_that_does_not_match_the_map() {
    let all_default = [0xffu8; 32];
    assert_eq!(
        AbsenceProof::from_parts(all_default, vec![NodeHash::new([7; 32])], ProofEnding::Empty)
            .unwrap_err(),
        OrgMembersError::AbsenceProofMalformed
    );
    assert_eq!(
        AbsenceProof::from_parts([0u8; 32], vec![], ProofEnding::Empty).unwrap_err(),
        OrgMembersError::AbsenceProofMalformed
    );
}

/// verifies: LLR-4rju5r, LLR-dgzy7e, REQ-tk2qqj
#[test]
fn a_tampered_sibling_is_refused_by_root() {
    let trie = org_without_alice_d2();
    let root = trie.root_hash().unwrap();
    let proof = trie.prove_absent(&id("alice"), &device_key("alice-d2")).unwrap();
    let mut siblings = proof.siblings().to_vec();
    siblings[0] = NodeHash::new([0xab; 32]);
    let tampered =
        AbsenceProof::from_parts(*proof.default_map(), siblings, proof.ending().clone()).unwrap();
    assert_eq!(
        tampered.verify::<Blake3Hasher>(&root, &id("alice"), &device_key("alice-d2")),
        Err(OrgMembersError::AbsenceProofRootMismatch)
    );
}

/// verifies: LLR-4rju5r, REQ-tk2qqj
#[test]
fn a_proof_survives_the_wire_and_still_verifies() {
    let trie = org_without_alice_d2();
    let root = trie.root_hash().unwrap();
    let proof = trie.prove_absent(&id("alice"), &device_key("alice-d2")).unwrap();
    let bytes = postcard::to_allocvec(&proof).unwrap();
    let back: AbsenceProof = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(back, proof);
    assert_eq!(back.verify::<Blake3Hasher>(&root, &id("alice"), &device_key("alice-d2")), Ok(()));
}

/// The wire shape, written independently of the crate, so a test can send
/// what the crate itself would never produce.
#[derive(serde::Serialize)]
struct RawProof {
    default_map: [u8; 32],
    siblings: Vec<[u8; 32]>,
    ending: Option<MemberLeaf>,
}

/// verifies: LLR-4rju5r, LLR-2dcnbp, REQ-ds8ryr
#[test]
fn the_wire_refuses_inconsistent_or_oversized_proofs() {
    let mismatch = RawProof { default_map: [0xff; 32], siblings: vec![[1; 32]], ending: None };
    let err = postcard::from_bytes::<AbsenceProof>(&postcard::to_allocvec(&mismatch).unwrap())
        .unwrap_err();
    assert!(matches!(err, postcard::Error::SerdeDeCustom), "{err:?}");

    let oversized = RawProof { default_map: [0; 32], siblings: vec![[1; 32]; 257], ending: None };
    assert!(postcard::from_bytes::<AbsenceProof>(&postcard::to_allocvec(&oversized).unwrap()).is_err());
}
```

2. Run `cargo test -p org-members --test absence_proofs`. Expected: compile errors, no function `from_parts`, and `AbsenceProof: Serialize`/`Deserialize` not satisfied. That is the red evidence.

3. Implement in `org-members/src/proof.rs`. Add to `impl AbsenceProof`:

```rust
    /// Parses a proof from its parts: refuses with `AbsenceProofMalformed`
    /// unless the number of siblings is the number of levels the map does not
    /// mark default, which caps it at 256 (LLR-4rju5r).
    pub fn from_parts(
        default_map: [u8; 32],
        siblings: Vec<NodeHash>,
        ending: ProofEnding,
    ) -> Result<Self, OrgMembersError> {
        let defaults = default_map.iter().map(|b| b.count_ones() as usize).sum::<usize>();
        if siblings.len() + defaults != usize::from(SMT_DEPTH) {
            return Err(OrgMembersError::AbsenceProofMalformed);
        }
        Ok(Self { default_map, siblings, ending })
    }
```

   Then the serde form. Its field order is the wire order, and it matches `RawProof` in the test:

```rust
#[cfg(feature = "serde")]
mod wire {
    use alloc::vec::Vec;
    use core::fmt;

    use serde::de::{Error as _, SeqAccess, Visitor};
    use serde::{Deserialize, Deserializer, Serialize};

    use super::{AbsenceProof, ProofEnding};
    use crate::error::OrgMembersError;
    use crate::smt::SMT_DEPTH;
    use crate::types::{MemberLeaf, NodeHash};

    #[derive(Serialize, Deserialize)]
    pub(super) struct RawAbsenceProof {
        default_map: [u8; 32],
        #[serde(deserialize_with = "at_most_256")]
        siblings: Vec<[u8; 32]>,
        ending: Option<MemberLeaf>,
    }

    /// Refuses a 257th sibling while decoding, so memory never grows past
    /// the bound (LLR-4rju5r).
    fn at_most_256<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<[u8; 32]>, D::Error> {
        struct Bounded;
        impl<'de> Visitor<'de> for Bounded {
            type Value = Vec<[u8; 32]>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("at most 256 sibling hashes")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut out = Vec::new();
                while let Some(hash) = seq.next_element::<[u8; 32]>()? {
                    if out.len() == usize::from(SMT_DEPTH) {
                        return Err(A::Error::custom(OrgMembersError::AbsenceProofMalformed));
                    }
                    out.push(hash);
                }
                Ok(out)
            }
        }
        d.deserialize_seq(Bounded)
    }

    impl From<AbsenceProof> for RawAbsenceProof {
        fn from(p: AbsenceProof) -> Self {
            Self {
                default_map: p.default_map,
                siblings: p.siblings.iter().map(|h| *h.as_bytes()).collect(),
                ending: match p.ending {
                    ProofEnding::Empty => None,
                    ProofEnding::Leaf(member) => Some(member),
                },
            }
        }
    }

    impl TryFrom<RawAbsenceProof> for AbsenceProof {
        type Error = OrgMembersError;
        fn try_from(raw: RawAbsenceProof) -> Result<Self, Self::Error> {
            AbsenceProof::from_parts(
                raw.default_map,
                raw.siblings.into_iter().map(NodeHash::new).collect(),
                match raw.ending {
                    None => ProofEnding::Empty,
                    Some(member) => ProofEnding::Leaf(member),
                },
            )
        }
    }
}
```

   On `AbsenceProof`, add the derive attributes:

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "wire::RawAbsenceProof", into = "wire::RawAbsenceProof"))]
```

   `MemberLeaf`'s own `Deserialize` validates the leaf in the ending (REQ-shk82j; its refusals are tested under LLR-pys2ek and LLR-xyv6p9), so this task adds no leaf validation.

4. Run `cargo test -p org-members --test absence_proofs`. Expected: `15 passed`. Then run the full `cargo test -p org-members`, clippy and the no_std build (`--no-default-features` compiles without `wire`), all green.

5. Commit: `feat(org-members): parse absence proofs at the edge, bounded serde form (LLR-4rju5r)`.

---

### T5 — Property tests: no panics, and every honest proof verifies

**Files touched:** org-members/tests/fuzz_tests.rs
**Parallel:** no (serial, after T4)

1. Append to `org-members/tests/fuzz_tests.rs`:

```rust
// ============================================================
// Absence proofs (SDD-57vaj4)
// ============================================================

proptest! {
    /// verifies: REQ-ds8ryr, LLR-2dcnbp, LLR-4rju5r
    #[test]
    fn absence_proof_decoding_and_verify_never_panic(
        bytes in proptest::collection::vec(any::<u8>(), 0..2048),
        target in any::<[u8; 32]>(),
    ) {
        if let Ok(proof) = postcard::from_bytes::<org_members::AbsenceProof>(&bytes) {
            let root = org_members::types::RootHash::new(target);
            let _ = proof.verify::<Blake3Hasher>(&root, &MemberId::new(target), &device_key("probe"));
        }
    }

    /// verifies: LLR-utp6x4, LLR-dgzy7e, LLR-4xz255, REQ-535jcd, REQ-tk2qqj
    #[test]
    fn every_honest_absence_proof_verifies(
        idxs in proptest::collection::btree_set(0usize..HANDLES.len(), 1..6),
        absent_seed in any::<u64>(),
    ) {
        let members: Vec<MemberLeaf> = idxs.iter().filter_map(|&i| make_member(HANDLES[i], 0)).collect();
        let trie = TestTrie::genesis(members).map_err(|e| TestCaseError::fail(format!("{e:?}")))?;
        let root = trie.root_hash().map_err(|e| TestCaseError::fail(format!("{e:?}")))?;
        let absent = member_id(&format!("absent-{absent_seed}"));
        let probe = device_key("probe-device");
        let proof = trie.prove_absent(&absent, &probe).map_err(|e| TestCaseError::fail(format!("{e:?}")))?;
        prop_assert_eq!(proof.verify::<Blake3Hasher>(&root, &absent, &probe), Ok(()));
    }
}
```

2. Red evidence: temporarily change `if hash == *defaults.at_level(level)?` in `proof.rs` `from_path` to `if hash != *defaults.at_level(level)?` and run `cargo test -p org-members --test fuzz_tests every_honest_absence_proof_verifies`. Expected: `1 failed`, `AbsenceProofMalformed` or `AbsenceProofRootMismatch`. Revert the mutation and record it.

3. Run `cargo test -p org-members --test fuzz_tests`. Expected: every property `ok`.

4. Commit: `test(org-members): absence proof properties`.

---

### T6 — Caller duties in the README

**Files touched:** org-members/README.md
**Parallel:** no (serial, after T4: it names the API that T4 completes)

1. Append under "Security checks the caller MUST perform", after section 11:

```markdown
### 12. Check an absence proof only against the current on-chain root, and give one only to the device it is about

`AbsenceProof::verify` accepts a proof that resolves to the root you pass. It cannot know whether that root is current. A proof that was genuine under an older root fails against a newer one. But a proof checked against a stale root that predates a device's admission says "absent" truthfully, and a device that acts on it deletes its data while it is still a member (HAZ-adm7gv). Pass only the latest finalised on-chain root, read by the caller at the moment of checking.

A proof that a Device key is absent from a Member who remains carries that Member's leaf: MemberId, handle, name, surname and keys (HAZ-gwbn5n). `prove_absent` cannot know who is asking. Produce a proof only for a requester whose authenticated Device key and supplied MemberId match a pair on your revoked-device list, and send it only over the connection that authenticated that key.

`prove_absent` does not refuse a Device key that another Member holds. Keeping a revoked Device key from ever being admitted again, to any Member, is the duty of whoever keeps the revoked-device list.
```

2. No test: this is documentation. Its controls are tested where they are implemented (S3 and S5 of the roadmap).

3. Commit: `docs(org-members): caller duties for absence proofs`.

---

## Self-review (plan-change step 9)

1. **Every Implements ID has a verifying test.**
   - REQ-535jcd: T2, T5. REQ-yyuxh8: T2. REQ-tk2qqj: T3, T4, T5.
   - RC-j2znx8: T3 `proof_is_refused_against_another_root`. RC-qa2758: through REQ-yyuxh8.
   - LLR-utp6x4: T3, T5. LLR-4xz255: T2, T5. LLR-25tpdp: T2. LLR-4rju5r: T4, T5. LLR-dgzy7e: T3, T4, T5. LLR-p2p8qy: T3. LLR-2dcnbp: T2, T3, T4, T5. LLR-7jkcba: T1.
2. **Real code, commands and expected output:** every code step shows the code, and every run step names its command and expected result.
3. **Consistent names:**
   - methods: `prove_absent`, `verify::<H>`, `from_parts`, `from_path`, `default_map()`, `siblings()`, `ending()`;
   - types: `ProofEnding::{Empty, Leaf}`, `PathEnd::{Empty, Leaf}`;
   - errors: `IndexOutOfRange`, `DeviceStillHeld`, `AbsenceProofMalformed`, `AbsenceProofRootMismatch`;
   - path helpers: `path_bits`, `smt::path`.
4. **Files touched and parallelism:** every task states both. All tasks are serial: T1 to T4 share `src/` files, T5 needs T4's serde form, and T6 names T4's API.

## Progress

| Task | State | red → green |
|---|---|---|
| T1 | done (0e093d3, merged) | `member_id_bit_out_of_range_does_not_panic`: watched fail for the right reason before the implementation existed (panic at types.rs:58, "index out of bounds: the len is 32 but the index is 32"). `default_hashes_at_level_out_of_range_does_not_panic`: watched fail for the right reason (panic at smt.rs:35, "the len is 257 but the index is 257"). `out_of_range_path_index_is_index_out_of_range`: the exact-value companion added at step 4, which could not compile before the fix (it names `IndexOutOfRange`); its red evidence is the two reproductions above. Result: 244 passed, clippy and the no_std build clean. Deviation: `insert` binds `path_bits()` to a local (E0597 on the tail-expression temporary) |
| T2 | done (6c3dd96, merged) | Each of `prove_absent_refuses_an_uncalculated_trie`, `prove_absent_refuses_a_device_the_member_holds`, `prove_absent_ignores_a_key_another_member_holds`, `proof_for_an_absent_member_carries_no_member_data` and `proof_for_a_removed_device_carries_only_that_member` was watched fail for the right reason before the implementation existed: E0599 no method `prove_absent`, E0432 unresolved `org_members::proof`, and no variant `DeviceStillHeld`. Result: 249 passed, clippy and the no_std build clean. Deviation: `is_default` moved to T3, its first caller (dead code otherwise fails clippy) |
| T3 | done (5c8aef8, merged) | Each of `proof_for_an_absent_member_verifies`, `proof_for_a_deleted_member_verifies`, `proof_for_a_removed_device_verifies`, `proof_is_refused_against_another_root`, `proof_is_refused_for_another_member_id` and `proof_is_refused_when_the_member_holds_the_device` was watched fail for the right reason before the implementation existed (E0599 no method `verify` on `AbsenceProof`). Result: 255 passed, clippy and the no_std build clean. `is_default` was added here, as T2 deferred |
| T4 | done (e0b8c6c, e800abe, merged) | `from_parts_refuses_a_count_that_does_not_match_the_map`, `a_tampered_sibling_is_refused_by_root`: watched fail for the right reason before the implementation existed (E0599 no `from_parts`). `a_proof_survives_the_wire_and_still_verifies`, `the_wire_refuses_inconsistent_or_oversized_proofs`: watched fail for the right reason (E0277 `AbsenceProof` not `Serialize`/`Deserialize`). Added after review of the report: the oversized assertion could not detect a missing bound, because `from_parts` also refuses 257 hashes. So `the_wire_stops_reading_siblings_at_the_bound` (verifies LLR-4rju5r, REQ-ds8ryr) was added: a length prefix of 1,000,000 followed by 257 hashes. Watched fail with the visitor's bound deleted (`DeserializeUnexpectedEnd`), and pass with it restored (`SerdeDeCustom`). Result: 260 passed, clippy and the no_std build clean |
| T5 | done (6bcc72e, merged) | `absence_proof_decoding_and_verify_never_panic`, red by mutation: `verify`'s explicit-sibling read was replaced with `self.siblings[usize::from(level)]`. The planned random-bytes-only property passed under it, because random bytes almost never decode, so a structured generator `arb_wire_proof` was added: the right sibling count, or any count up to 299. With it, the property failed with a panic at proof.rs:136 and passes once the mutation is reverted. This departs from the plan text. `every_honest_absence_proof_verifies`, red by the plan's step 2 mutation (`==` → `!=` in `from_path`): failed with `Err(AbsenceProofRootMismatch)`, and passes once reverted. Result: 262 passed, clippy clean |
| gatefix | done (676a3b8, 02fb928, 974d472, merged) | Gate findings of 2026-10-06. `an_empty_ending_proof_survives_the_wire_and_still_verifies`: watched fail under mutation M3 (the wire turns an `Empty` ending into `AbsenceProofMalformed`), passes once reverted. `a_proof_next_to_a_neighbour_carries_none_of_its_data`: watched fail under M4 (`path` returns the neighbouring sibling's leaf), with "proof bytes carry the neighbour's handle"; it is the only test that catches M4. `proofs_at_the_lowest_and_highest_ids_verify_when_absent`, `proofs_at_the_lowest_and_highest_ids_verify_for_a_removed_device`, `proofs_in_a_one_member_organisation_verify`: watched fail under M5a and M5b (one arm of `path`'s sibling choice swapped) with `AbsenceProofRootMismatch`. The Makefile now measures `absence_proofs`, so proof.rs line coverage rose from 82.17% to 97.62%. `verify`'s unreachable surplus-sibling check was removed, citing LLR-4rju5r |
| review1fix2 | done (a3f4735, merged) | For the independent review's findings 1 to 7 (2026-10-06). `a_proof_sends_only_non_default_siblings`: watched fail under (i) `from_path` sending every level explicitly (the map came out all 0x00 where [0xff;31]+0x7f was expected) and (ii) the map bit written at `7 - level % 8` (last byte 0xfe against 0x7f); passes once both are reverted. `prove_absent_refuses_an_uncalculated_trie_even_on_a_hashed_path`: watched fail with `prove_absent`'s `is_calculated()` guard deleted (it got `Ok`), while the older uncalculated test still passed; passes with the guard restored. Wording fixes: LLR-4rju5r and LLR-2dcnbp (finding-3), the risk file's "Hostile input" (finding-4), `verifies:` additions (PR-jq43gx, finding-5; RC-qa2758, finding-7), and the roadmap's stale rejoin sentence plus the S1 row (finding-6) |
| T6 | done (d07abf6, merged) | Documentation only, so no test (step 2). README section 12 now names `AbsenceProofRootMismatch` as the error `verify` returns. `check-trace.sh` exits 0, with no DANGLING-REF or MALFORMED-ID |
