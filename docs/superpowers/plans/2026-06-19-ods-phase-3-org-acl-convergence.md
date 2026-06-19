# org-acl Convergence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Converge the parallel Keyhive and p2panda ACL tracks into one library that compiles to either backend, by extracting the substrate-neutral *authority decision* into a shared `org-acl-core` and having each backend execute it in its own idiom.

**Architecture:** Shared core + backend crates + facade (spec §2). The convergence point is the **pure, sync decision logic** — `gate` (effective access = ledger ∩ trie-vouched) and `decide_reconcile` (monotonicity + same-epoch-reorg + delta-vs-full + who-departed → a `ReconcileVerdict`) — written and tested once in `org-acl-core`. Each backend computes the verdict, then executes it sync (p2panda `GroupCrdt`) or async (Keyhive). This sidesteps the sync/async + per-doc/multi-doc mismatch between the two backends and kills the Keyhive reorg bug and witness forge-hole by construction.

**Tech Stack:** Rust (workspace), `org-acl-core` (no substrate deps), `ed25519-dalek`, `serde`/`postcard`, `thiserror`, `bolero` (fuzz). p2panda backend: `p2panda-auth`. Keyhive backend: `keyhive_core` (GPL, sealed in one crate).

**This document fully specifies Plan 1 (org-acl-core convergence primitives).** Plans 2–4 are sequenced as a roadmap (§"Subsequent plans") and each gets its own full plan once Plan 1 lands, because their exact code depends on Plan 1's final signatures.

## Global Constraints

- **Per-doc unit:** a single ACL (+ a reserved CGKA slot for item-4) per document. Delegation keys on `OrgMember` (handles/`MemberId` are unique only within one org).
- **Lazy reconcile:** docs reconcile on open ("open docs first"); each backend keeps its own reconcile-execution methodology.
- **Authority model (verbatim, spec §3):** effective access = `ledger ∩ currently-trie-vouched`, recomputed live, **owner/manager NOT exempt**, no cache. Epochs never move backward; same-epoch-different-root is a reorg and re-reconciles.
- **Witness:** `VerifiedTrieChange` is **non-serde** (minted in-process, never from the wire) — do not add `Serialize`/`Deserialize`.
- **clippy `--lib` deny-gate:** no `unwrap`/`expect`/`panic!`/`unreachable!`/indexing-panic in library (`--lib`) code. Test code (`#[cfg(test)]`) may use `unwrap`.
- **New-crate fetches:** `~/.cargo` is read-only here — prefix cargo commands that resolve new deps with `CARGO_HOME=/tmp/cargo_home_fuzz`. (Plan 1 adds no new deps; needed for the `bolero` fuzz target if not already vendored.)
- **Fuzz (AGENTS.md hard rule):** security-critical decision functions get a `bolero` target asserting no-panic + the monotonicity invariant.
- **Commits:** single-purpose, no `Co-Authored-By` lines. Work in a git worktree; final integration is a user-signed squash-merge.

---

## Plan 1 — `org-acl-core` convergence primitives

**Target location:** the `org-acl-core` crate (currently in `worktree-phase-3-item3-p2panda`; the unified workspace is assembled in Plan 4). Create the worktree via `superpowers:using-git-worktrees` at execution start if not already in one.

**Existing context (already in `org-acl-core`, do not recreate):**
- `identity.rs` — `OrgId([u8;20])`, `MemberId([u8;32])`, `OrgMember{org,member}`, `RootHash([u8;32])`, `Epoch(u64)`, `P2pMemberKey`, `P2pDeviceKey`, `OrgKey`, `Principal`.
- `delta.rs` — `MembershipDelta { from_epoch, to_epoch, from_root, to_root, removed: Vec<MemberId>, added: Vec<MemberId>, rotated: Vec<MemberId>, org_key_rotated: bool }`, `is_single_step()`.
- `mark.rs` — `OrgEpochs { acl_epoch, cgka_epoch, root_hash }` + `fresh()`; `DocReconcileMark { per_org: BTreeMap<OrgId,OrgEpochs> }` + `new()`/`epochs(&OrgId)`/`advance(org,epoch,root)`.
- `witness.rs` — non-serde `VerifiedTrieChange`; `accept_delta(OrgId, MembershipDelta)`, `accept_full_reload(OrgId, Epoch, RootHash)`; `org()`, `to_epoch()`, `to_root()`, `delta() -> Option<&MembershipDelta>`.
- `role.rs` — `AclRole { Read, Write, Manage }`.
- `resolver.rs` — `MemberKeyResolver` trait + `ResolverError { UnknownMember(MemberId), OrgKeyUnset, WrongOrg{expected,got} }`.
- `test_support.rs` (feature `testing`) — `StubResolver`, `root_for_epoch(Epoch) -> RootHash`.

---

### Task 1: Unified `AclError`

**Files:**
- Create: `org-acl-core/src/error.rs`
- Modify: `org-acl-core/src/lib.rs` (add `pub mod error; pub use error::AclError;`)
- Test: inline `#[cfg(test)]` in `error.rs`

**Interfaces:**
- Produces: `org_acl_core::AclError` — enum with variants `AuthorNotMember(OrgMember)`, `NotAMember(OrgMember)`, `WrongOrg { expected: OrgId, got: OrgId }`, `StaleWitness { mark: Epoch, witness: Epoch }`, `Backend(String)`. Derives `Debug, thiserror::Error, PartialEq, Eq`.

- [ ] **Step 1: Write the failing test**

Append to a new file `org-acl-core/src/error.rs`:

```rust
//! `AclError` — the unified ACL failure type shared by every backend (spec §4).
//! p2panda's `AclError` and Keyhive's `AclError` collapse onto this; backend
//! substrate rejections are wrapped in `Backend(String)`.

use crate::identity::{Epoch, OrgId, OrgMember};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AclError {
    /// The op author is not a current member of the trie (security-relevant).
    #[error("author {0:?} is not a current org-trie member")]
    AuthorNotMember(OrgMember),

    /// A grantee/target is not a current org member.
    #[error("{0:?} is not a current org-trie member")]
    NotAMember(OrgMember),

    /// The witness names a different org than the resolver is bound to.
    #[error("witness org {got:?} does not match resolver org {expected:?}")]
    WrongOrg { expected: OrgId, got: OrgId },

    /// The witness is stale/replayed: not strictly ahead of the doc's mark.
    #[error("stale/replay witness: mark at acl_epoch {mark:?}, witness to {witness:?}")]
    StaleWitness { mark: Epoch, witness: Epoch },

    /// The backend substrate (CRDT / Keyhive) rejected the operation.
    #[error("backend rejected the operation: {0}")]
    Backend(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{MemberId, OrgId, OrgMember};

    #[test]
    fn wrong_org_displays_both_orgs() {
        let e = AclError::WrongOrg { expected: OrgId([1; 20]), got: OrgId([2; 20]) };
        let s = format!("{e}");
        assert!(s.contains("does not match"));
    }

    #[test]
    fn variants_are_eq() {
        let m = OrgMember { org: OrgId([1; 20]), member: MemberId([9; 32]) };
        assert_eq!(AclError::AuthorNotMember(m), AclError::AuthorNotMember(m));
        assert_ne!(AclError::AuthorNotMember(m), AclError::NotAMember(m));
    }
}
```

Add to `org-acl-core/src/lib.rs` after the `delta` re-export block:

```rust
pub mod error;
pub use error::AclError;
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p org-acl-core error::`
Expected: compile error or FAIL until `error.rs` + the `lib.rs` export are in place (red before this task's edits compile).

- [ ] **Step 3: (implementation is the file above — no further code)**

The `error.rs` content in Step 1 is the complete implementation.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p org-acl-core error::`
Expected: PASS (2 tests).

- [ ] **Step 5: Lint + commit**

```bash
cargo clippy -p org-acl-core --lib -- -D warnings
git add org-acl-core/src/error.rs org-acl-core/src/lib.rs
git commit -m "feat(org-acl-core): unified AclError shared by all backends"
```

---

### Task 2: The pure access gate (`gate.rs`)

**Files:**
- Create: `org-acl-core/src/gate.rs`
- Modify: `org-acl-core/src/lib.rs` (add `pub mod gate;`)
- Test: inline `#[cfg(test)]` in `gate.rs` (uses `StubResolver` under `feature = "testing"`)

**Interfaces:**
- Consumes: `MemberKeyResolver` (Task 0/existing), `OrgMember` (existing).
- Produces:
  - `pub fn effective_members<R, I>(ledger: I, resolver: &R) -> std::collections::BTreeSet<OrgMember> where R: MemberKeyResolver, I: IntoIterator<Item = OrgMember>`
  - `pub fn is_effective<R, I>(ledger: I, resolver: &R, member: &OrgMember) -> bool where R: MemberKeyResolver, I: IntoIterator<Item = OrgMember>`

- [ ] **Step 1: Write the failing test**

Create `org-acl-core/src/gate.rs`:

```rust
//! The live access gate (spec §3): effective access = advisory ledger ∩
//! currently-trie-vouched, recomputed on every call. Owner/manager NOT exempt;
//! no cache. A ledger entry for a different org than the resolver, or a member
//! the trie no longer vouches, is excluded. Substrate-neutral and pure.

use std::collections::BTreeSet;

use crate::identity::OrgMember;
use crate::resolver::MemberKeyResolver;

/// Effective members = `ledger ∩ currently-trie-vouched` for the resolver's org.
pub fn effective_members<R, I>(ledger: I, resolver: &R) -> BTreeSet<OrgMember>
where
    R: MemberKeyResolver,
    I: IntoIterator<Item = OrgMember>,
{
    let org = resolver.org_id();
    ledger
        .into_iter()
        .filter(|m| m.org == org && resolver.is_member(&m.member))
        .collect()
}

/// True iff `member` is in `ledger` AND currently trie-vouched in the resolver's org.
pub fn is_effective<R, I>(ledger: I, resolver: &R, member: &OrgMember) -> bool
where
    R: MemberKeyResolver,
    I: IntoIterator<Item = OrgMember>,
{
    member.org == resolver.org_id()
        && resolver.is_member(&member.member)
        && ledger.into_iter().any(|m| m == *member)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{MemberId, OrgId, OrgMember};
    use crate::testing::StubResolver;
    use crate::P2pMemberKey;
    use ed25519_dalek::SigningKey;

    const ORG: OrgId = OrgId([0xaa; 20]);
    fn om(seed: u8) -> OrgMember { OrgMember { org: ORG, member: MemberId([seed; 32]) } }
    fn mkey(seed: u8) -> P2pMemberKey { P2pMemberKey(SigningKey::from_bytes(&[seed; 32]).verifying_key()) }

    #[test]
    fn owner_not_exempt_when_revoked() {
        let r = StubResolver::new(ORG).add_member(MemberId([0xa1; 32]), mkey(0xa1), vec![]);
        let ledger = vec![om(0xa1)];
        assert!(is_effective(ledger.clone(), &r, &om(0xa1)));
        let r = r.revoke(&MemberId([0xa1; 32]));
        assert!(effective_members(ledger.clone(), &r).is_empty(), "revoked owner is excluded");
        assert!(!is_effective(ledger, &r, &om(0xa1)));
    }

    #[test]
    fn excludes_foreign_org_and_unvouched() {
        let r = StubResolver::new(ORG).add_member(MemberId([0xa1; 32]), mkey(0xa1), vec![]);
        let foreign = OrgMember { org: OrgId([0xbb; 20]), member: MemberId([0xa1; 32]) };
        let unvouched = om(0xff);
        let eff = effective_members(vec![om(0xa1), foreign, unvouched], &r);
        assert_eq!(eff.into_iter().collect::<Vec<_>>(), vec![om(0xa1)]);
    }
}
```

Add to `org-acl-core/src/lib.rs`:

```rust
pub mod gate;
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p org-acl-core --features testing gate::`
Expected: FAIL/compile error before the `lib.rs` export exists.

- [ ] **Step 3: (implementation is the file above)**

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p org-acl-core --features testing gate::`
Expected: PASS (2 tests).

- [ ] **Step 5: Lint + commit**

```bash
cargo clippy -p org-acl-core --lib --features testing -- -D warnings
git add org-acl-core/src/gate.rs org-acl-core/src/lib.rs
git commit -m "feat(org-acl-core): pure access gate (effective_members / is_effective)"
```

---

### Task 3: The reconcile decision (`authority.rs`)

This is the shared spine. It reproduces the p2panda reconcile decision (monotonicity, same-epoch-reorg, delta-vs-full, departed-member computation) as a pure verdict, so both backends route through one tested implementation.

**Files:**
- Create: `org-acl-core/src/authority.rs`
- Modify: `org-acl-core/src/lib.rs` (add `pub mod authority; pub use authority::{decide_reconcile, ReconcileMode, ReconcileVerdict};`)
- Test: inline `#[cfg(test)]` in `authority.rs`

**Interfaces:**
- Consumes: `DocReconcileMark`, `OrgEpochs::fresh`, `MemberKeyResolver`, `VerifiedTrieChange`, `MembershipDelta::is_single_step`, `AclError` (Task 1).
- Produces:
  - `pub enum ReconcileMode { Delta, Full }` (derives `Clone, Copy, Debug, PartialEq, Eq`)
  - `pub struct ReconcileVerdict { pub mode: ReconcileMode, pub revoke: Vec<OrgMember>, pub cgka_rekey: Vec<OrgMember>, pub restamp_epoch: Epoch, pub restamp_root: RootHash }` (derives `Clone, Debug, PartialEq, Eq`)
  - `pub fn decide_reconcile<R: MemberKeyResolver>(mark: &DocReconcileMark, resolver: &R, witness: &VerifiedTrieChange, ledger: &[OrgMember], retain: &[OrgMember]) -> Result<ReconcileVerdict, AclError>`
    - `ledger`: the doc's current advisory member set (backend supplies it).
    - `retain`: principals never auto-revoked (e.g. the doc owner/manager).

- [ ] **Step 1: Write the failing tests**

Create `org-acl-core/src/authority.rs`:

```rust
//! The reconcile decision — the substrate-neutral authority spine (spec §3).
//!
//! `decide_reconcile` is pure and sync: given the doc's reconcile mark, the
//! current trie resolver, a verified witness, the doc's advisory ledger, and the
//! never-revoke `retain` set, it returns a `ReconcileVerdict` the backend then
//! executes in its own idiom. All monotonicity / same-epoch-reorg / delta-vs-full
//! logic lives here, written and tested ONCE — so the Keyhive epoch-only stale bug
//! cannot recur and the decision is identical across backends.

use crate::error::AclError;
use crate::identity::{Epoch, OrgMember, RootHash};
use crate::mark::{DocReconcileMark, OrgEpochs};
use crate::resolver::MemberKeyResolver;
use crate::witness::VerifiedTrieChange;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconcileMode {
    /// A contiguous Δ==1 patch that chains the mark.
    Delta,
    /// A full rebuild from the resolver (gap, reorg, or full-reload witness).
    Full,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconcileVerdict {
    pub mode: ReconcileMode,
    /// ACL-tier removals to apply (already excludes `retain` and non-ledger members).
    pub revoke: Vec<OrgMember>,
    /// CGKA-tier rekeys (rotated members still present) — item-4 consumes this.
    pub cgka_rekey: Vec<OrgMember>,
    /// Stamp the doc's mark cell to these on success.
    pub restamp_epoch: Epoch,
    pub restamp_root: RootHash,
}

/// Decide what a verified witness implies for a document's ACL. See module docs.
pub fn decide_reconcile<R: MemberKeyResolver>(
    mark: &DocReconcileMark,
    resolver: &R,
    witness: &VerifiedTrieChange,
    ledger: &[OrgMember],
    retain: &[OrgMember],
) -> Result<ReconcileVerdict, AclError> {
    let org = witness.org();
    if org != resolver.org_id() {
        return Err(AclError::WrongOrg { expected: resolver.org_id(), got: org });
    }
    let cell = mark.epochs(&org).copied().unwrap_or_else(OrgEpochs::fresh);
    let to_epoch = witness.to_epoch();
    let to_root = witness.to_root();

    // Reject a witness not strictly ahead: a regression, or an exact (epoch+root) replay.
    // A same-epoch DIFFERENT-root witness is a reorg and is NOT rejected (falls to Full).
    if to_epoch < cell.acl_epoch || (to_epoch == cell.acl_epoch && to_root == cell.root_hash) {
        return Err(AclError::StaleWitness { mark: cell.acl_epoch, witness: to_epoch });
    }

    let retained = |m: &OrgMember| retain.contains(m);

    // Delta path iff the witness carries a single contiguous step chaining the cell.
    let chaining_delta = witness.delta().filter(|d| {
        d.is_single_step()
            && d.from_epoch == cell.acl_epoch
            && d.from_root == cell.root_hash
            && d.to_epoch == to_epoch
            && d.to_root == to_root
    });

    let verdict = match chaining_delta {
        Some(d) => {
            let revoke: Vec<OrgMember> = d
                .removed
                .iter()
                .map(|m| OrgMember { org, member: *m })
                .filter(|m| ledger.contains(m) && !retained(m))
                .collect();
            let cgka_rekey: Vec<OrgMember> = d
                .rotated
                .iter()
                .map(|m| OrgMember { org, member: *m })
                .filter(|m| ledger.contains(m) && !retained(m) && !revoke.contains(m))
                .collect();
            ReconcileVerdict { mode: ReconcileMode::Delta, revoke, cgka_rekey, restamp_epoch: to_epoch, restamp_root: to_root }
        }
        None => {
            let revoke: Vec<OrgMember> = ledger
                .iter()
                .copied()
                .filter(|m| m.org == org && !resolver.is_member(&m.member) && !retained(m))
                .collect();
            ReconcileVerdict { mode: ReconcileMode::Full, revoke, cgka_rekey: Vec::new(), restamp_epoch: to_epoch, restamp_root: to_root }
        }
    };
    Ok(verdict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delta::MembershipDelta;
    use crate::identity::{MemberId, OrgId, OrgMember};
    use crate::mark::DocReconcileMark;
    use crate::testing::{root_for_epoch, StubResolver};
    use crate::witness::VerifiedTrieChange;
    use crate::P2pMemberKey;
    use ed25519_dalek::SigningKey;

    const ORG: OrgId = OrgId([0xaa; 20]);
    const ALICE: MemberId = MemberId([0xa1; 32]);
    const BOB: MemberId = MemberId([0xb1; 32]);
    fn om(m: MemberId) -> OrgMember { OrgMember { org: ORG, member: m } }
    fn mkey(s: u8) -> P2pMemberKey { P2pMemberKey(SigningKey::from_bytes(&[s; 32]).verifying_key()) }
    fn mark_at(epoch: u64) -> DocReconcileMark {
        let mut m = DocReconcileMark::new();
        m.advance(ORG, Epoch(epoch), root_for_epoch(Epoch(epoch)));
        m
    }
    fn delta(from: u64, to: u64, removed: Vec<MemberId>, rotated: Vec<MemberId>) -> VerifiedTrieChange {
        VerifiedTrieChange::accept_delta(ORG, MembershipDelta {
            from_epoch: Epoch(from), to_epoch: Epoch(to),
            from_root: root_for_epoch(Epoch(from)), to_root: root_for_epoch(Epoch(to)),
            removed, added: vec![], rotated, org_key_rotated: false,
        })
    }

    #[test]
    fn rejects_wrong_org() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(1), vec![]);
        let w = VerifiedTrieChange::accept_full_reload(OrgId([0xbb; 20]), Epoch(2), root_for_epoch(Epoch(2)));
        assert!(matches!(decide_reconcile(&mark_at(1), &r, &w, &[om(ALICE)], &[om(ALICE)]),
            Err(AclError::WrongOrg { .. })));
    }

    #[test]
    fn rejects_regress_and_exact_replay() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(1), vec![]);
        let regress = VerifiedTrieChange::accept_full_reload(ORG, Epoch(1), root_for_epoch(Epoch(1)));
        assert!(matches!(decide_reconcile(&mark_at(2), &r, &regress, &[], &[]), Err(AclError::StaleWitness { .. })));
        let replay = VerifiedTrieChange::accept_full_reload(ORG, Epoch(2), root_for_epoch(Epoch(2)));
        assert!(matches!(decide_reconcile(&mark_at(2), &r, &replay, &[], &[]), Err(AclError::StaleWitness { .. })));
    }

    #[test]
    fn same_epoch_different_root_is_full_reorg_not_stale() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(1), vec![]);
        let reorg = VerifiedTrieChange::accept_full_reload(ORG, Epoch(2), RootHash([0x77; 32]));
        let v = decide_reconcile(&mark_at(2), &r, &reorg, &[om(ALICE)], &[om(ALICE)]).expect("reorg accepted");
        assert_eq!(v.mode, ReconcileMode::Full);
        assert_eq!(v.restamp_root, RootHash([0x77; 32]));
    }

    #[test]
    fn chaining_delta_revokes_removed_in_ledger_excluding_retain() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(1), vec![]); // BOB already gone
        let w = delta(1, 2, vec![BOB, ALICE], vec![]); // ALICE is retained (manager)
        let v = decide_reconcile(&mark_at(1), &r, &w, &[om(ALICE), om(BOB)], &[om(ALICE)]).unwrap();
        assert_eq!(v.mode, ReconcileMode::Delta);
        assert_eq!(v.revoke, vec![om(BOB)], "removed∩ledger minus retained manager");
    }

    #[test]
    fn noncontiguous_delta_falls_back_to_full() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(1), vec![]); // BOB not vouched
        let w = delta(0, 3, vec![], vec![]); // from_epoch 0 ≠ cell epoch 1 → Full
        let v = decide_reconcile(&mark_at(1), &r, &w, &[om(ALICE), om(BOB)], &[om(ALICE)]).unwrap();
        assert_eq!(v.mode, ReconcileMode::Full);
        assert_eq!(v.revoke, vec![om(BOB)], "full path prunes ledger members the trie no longer vouches");
    }

    #[test]
    fn rotation_only_delta_yields_cgka_rekey_not_revoke() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(1), vec![]).add_member(BOB, mkey(2), vec![]);
        let w = delta(1, 2, vec![], vec![BOB]);
        let v = decide_reconcile(&mark_at(1), &r, &w, &[om(ALICE), om(BOB)], &[om(ALICE)]).unwrap();
        assert!(v.revoke.is_empty());
        assert_eq!(v.cgka_rekey, vec![om(BOB)]);
    }
}
```

Add to `org-acl-core/src/lib.rs`:

```rust
pub mod authority;
pub use authority::{decide_reconcile, ReconcileMode, ReconcileVerdict};
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p org-acl-core --features testing authority::`
Expected: FAIL/compile error before the module + exports compile.

- [ ] **Step 3: (implementation is the file above)**

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p org-acl-core --features testing authority::`
Expected: PASS (6 tests).

- [ ] **Step 5: Lint + commit**

```bash
cargo clippy -p org-acl-core --lib --features testing -- -D warnings
git add org-acl-core/src/authority.rs org-acl-core/src/lib.rs
git commit -m "feat(org-acl-core): decide_reconcile — shared reconcile-decision spine"
```

---

### Task 4: Promote `find_member_by_device` (R7) into the resolver

**Files:**
- Modify: `org-acl-core/src/resolver.rs` (add trait method + `OrgMember` import)
- Modify: `org-acl-core/src/test_support.rs` (impl for `StubResolver` + `OrgMember` import)
- Test: inline `#[cfg(test)]` in `test_support.rs`

**Interfaces:**
- Produces (on `MemberKeyResolver`): `fn find_member_by_device(&self, dev: &P2pDeviceKey) -> Option<OrgMember>;`
- This is a **required** trait method (no default): both real backends must implement a trie-backed lookup; a default returning `None` would silently disable item-5 connection gating.

- [ ] **Step 1: Write the failing test**

In `org-acl-core/src/test_support.rs`, add `OrgMember` to the identity import:

```rust
use crate::identity::{Epoch, MemberId, OrgId, OrgKey, OrgMember, P2pDeviceKey, P2pMemberKey, RootHash};
```

Add this test to the `#[cfg(test)] mod tests` block in `test_support.rs`:

```rust
    #[test]
    fn find_member_by_device_round_trips() {
        let dev = P2pDeviceKey(SigningKey::from_bytes(&[0x11; 32]).verifying_key());
        let alice = MemberId([1; 32]);
        let r = StubResolver::new(ORG).add_member(alice, mkey(1), vec![dev]);
        assert_eq!(r.find_member_by_device(&dev), Some(OrgMember { org: ORG, member: alice }));
        let unknown = P2pDeviceKey(SigningKey::from_bytes(&[0x99; 32]).verifying_key());
        assert_eq!(r.find_member_by_device(&unknown), None);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p org-acl-core --features testing test_support::tests::find_member_by_device_round_trips`
Expected: FAIL — `no method named find_member_by_device`.

- [ ] **Step 3: Add the trait method and the StubResolver impl**

In `org-acl-core/src/resolver.rs`, add `OrgMember` to the import:

```rust
use crate::identity::{Epoch, MemberId, OrgId, OrgKey, OrgMember, P2pDeviceKey, P2pMemberKey, RootHash};
```

Add this method to the `MemberKeyResolver` trait (after `current_devices`):

```rust
    /// Cross-org cold reverse lookup (R7): which `OrgMember` owns `dev`? `None` if
    /// no member in this resolver's org owns the device. Substrate-neutral;
    /// item-5 connection gating composes it.
    fn find_member_by_device(&self, dev: &P2pDeviceKey) -> Option<OrgMember>;
```

In `org-acl-core/src/test_support.rs`, add this method to `impl MemberKeyResolver for StubResolver` (after `current_devices`):

```rust
    fn find_member_by_device(&self, dev: &P2pDeviceKey) -> Option<OrgMember> {
        self.members
            .iter()
            .find(|(_, e)| e.devices.contains(dev))
            .map(|(id, _)| OrgMember { org: self.org, member: *id })
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p org-acl-core --features testing test_support::`
Expected: PASS (existing + new).

- [ ] **Step 5: Lint + commit**

```bash
cargo clippy -p org-acl-core --lib --features testing -- -D warnings
git add org-acl-core/src/resolver.rs org-acl-core/src/test_support.rs
git commit -m "feat(org-acl-core): add find_member_by_device (R7) to MemberKeyResolver"
```

> **Note for Plans 2 & 3:** this is a breaking trait change. The p2panda `ResolverPki` (Plan 2) and the Keyhive resolver impl + `org-node`'s real resolver (Plan 3/4) must each add a trie-backed `find_member_by_device`. Listed in those plans' interface-consumption blocks.

---

### Task 5: Fuzz the decision (`bolero`)

**Files:**
- Create: `org-acl-core/tests/fuzz_decide_reconcile.rs` (or extend the crate's existing fuzz module if one exists — mirror the `on-chain-client` bolero pattern)
- Modify: `org-acl-core/Cargo.toml` (add `bolero` dev-dependency if absent)

**Interfaces:**
- Consumes: `decide_reconcile`, `StubResolver`, `VerifiedTrieChange`, `DocReconcileMark`.

- [ ] **Step 1: Write the fuzz target asserting the invariants**

Create `org-acl-core/tests/fuzz_decide_reconcile.rs`:

```rust
//! Fuzz: decide_reconcile never panics, and on success the verdict's restamp
//! epoch is >= the mark's prior acl_epoch (monotonicity is never violated).

#![cfg(feature = "testing")]

use org_acl_core::testing::{root_for_epoch, StubResolver};
use org_acl_core::{decide_reconcile, DocReconcileMark, Epoch, MemberId, OrgId, OrgMember, RootHash, VerifiedTrieChange};

const ORG: OrgId = OrgId([0xaa; 20]);

#[test]
fn decide_reconcile_is_panic_free_and_monotonic() {
    bolero::check!()
        .with_type::<(u64, u64, u8)>()
        .for_each(|(mark_epoch, to_epoch, removed_seed)| {
            let mark_epoch = mark_epoch % 1000;
            let to_epoch = to_epoch % 1000;
            let mut mark = DocReconcileMark::new();
            mark.advance(ORG, Epoch(mark_epoch), root_for_epoch(Epoch(mark_epoch)));
            let r = StubResolver::new(ORG);
            let w = VerifiedTrieChange::accept_full_reload(ORG, Epoch(to_epoch), root_for_epoch(Epoch(to_epoch)));
            let ledger = vec![OrgMember { org: ORG, member: MemberId([*removed_seed; 32]) }];
            if let Ok(v) = decide_reconcile(&mark, &r, &w, &ledger, &[]) {
                assert!(v.restamp_epoch.0 >= mark_epoch, "restamp must never regress below the mark");
                let _ = RootHash::default();
            }
        });
}
```

- [ ] **Step 2: Ensure `bolero` is available**

If `cargo test -p org-acl-core --features testing` reports `bolero` missing, add to `org-acl-core/Cargo.toml` under `[dev-dependencies]` the same `bolero` version the workspace already pins (check `on-chain-client/Cargo.toml`), fetching with:

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo build -p org-acl-core --tests --features testing`

- [ ] **Step 3: Run the fuzz target (bounded)**

Run: `cargo test -p org-acl-core --features testing --test fuzz_decide_reconcile`
Expected: PASS (bolero runs the corpus / a bounded number of iterations under `cargo test`).

- [ ] **Step 4: Commit**

```bash
cargo clippy -p org-acl-core --tests --features testing -- -D warnings
git add org-acl-core/tests/fuzz_decide_reconcile.rs org-acl-core/Cargo.toml
git commit -m "test(org-acl-core): bolero fuzz decide_reconcile (panic-free + monotonic)"
```

---

### Task 6: Crate-level convergence doc + full green gate

**Files:**
- Modify: `org-acl-core/src/lib.rs` (expand the crate doc to state it now owns the shared authority spine)

- [ ] **Step 1: Update the crate doc**

Replace the `org-acl-core/src/lib.rs` top doc comment's first paragraph with:

```rust
//! ODS organisation ACL — the neutral, substrate-agnostic contract AND the shared
//! authority spine. This crate has NO local-first-substrate dependencies. Beyond
//! the convergence types (identity, delta, mark, role, resolver, witness) it owns
//! the pure decision logic both backends route through: `gate` (effective access =
//! ledger ∩ trie-vouched, owner not exempt) and `authority::decide_reconcile`
//! (monotonicity + same-epoch reorg + delta-vs-full + departed-member computation).
//! Backends (`org-acl-p2panda`, `org-acl-keyhive`) execute the verdict in their own
//! sync/async idiom; the decision itself is written and tested once, here.
```

- [ ] **Step 2: Full crate test + lint gate**

Run: `cargo test -p org-acl-core --features testing`
Expected: PASS (all modules).

Run: `cargo clippy -p org-acl-core --lib --features testing -- -D warnings`
Expected: no warnings.

- [ ] **Step 3: Commit**

```bash
git add org-acl-core/src/lib.rs
git commit -m "docs(org-acl-core): crate now owns the shared authority spine"
```

---

## Subsequent plans (2–4) — roadmap

Each becomes its own full plan (own `docs/superpowers/plans/` file) after Plan 1 lands, since their exact code depends on Plan 1's final signatures. Sequencing and scope:

### Plan 2 — p2panda refit onto core (depends on Plan 1)
- **`org-acl-p2panda/src/acl.rs`:** delete the inline reconcile logic in `DocAccess::reconcile`/`reconcile_delta`/`reconcile_full`; replace with `decide_reconcile(&self.mark, resolver, witness, &ledger, &[manager])` then execute the verdict's `revoke` via `GroupAction::Remove` and `self.mark.advance(org, v.restamp_epoch, v.restamp_root)`. Replace `effective_members`/`is_effective` bodies with `gate::effective_members`/`gate::is_effective`. Replace the local `AclError` with `org_acl_core::AclError` (map `CrdtRejected` → `Backend("crdt".into())`).
- **Delegation level:** change `add_member`/`remove_member`/`set_access` to take `OrgMember` (not bare `MemberId`); keep the resolver-org cross-check (`WrongOrg` if `arg.org != resolver.org_id()`).
- **R7:** implement `find_member_by_device` on `ResolverPki` (the p2panda resolver wrapper).
- **Tests:** the existing `acl.rs` test suite must stay green (it already encodes the target behaviour); add a cross-check that `DocAccess` revoke decisions equal `decide_reconcile`'s verdict.

### Plan 3 — Keyhive migration onto core (depends on Plan 1)
- **Rename** crate `doc-access` → `org-acl-keyhive`; depend on `org-acl-core`.
- **Delete** the parallel `identity.rs` types that duplicate core (`OrgId`, `MemberId`, `OrgMember`, `Epoch`, `RootHash`, `Principal`, `P2p*`/`OrgKey`); keep only `DocId` (re-export of `keyhive_core … DocumentId`) and move it to a `khtypes`-adjacent module. Delete `ChangeSet` (use `MembershipDelta`).
- **Replace** the local serde `VerifiedTrieChange` (forge-hole) with `org_acl_core::VerifiedTrieChange` (non-serde). Update call sites: `witness.org_id()`→`witness.org()`, `witness.epoch()`→`witness.to_epoch()`, `witness.root_hash()`→`witness.to_root()`, `witness.change_set()`→`witness.delta()` (now `Option<&MembershipDelta>` with `removed: Vec<MemberId>`, so `changed.removed.contains(&om)` becomes `d.removed.contains(&om.member)` within `witness.org()`).
- **Reroute** `reconcile` through `decide_reconcile` (supplying the doc's delegated set as `ledger` and `[owner]` as `retain`); this **deletes the `witness.epoch() <= acl_epoch` epoch-only stale check** (the reorg bug) — `decide_reconcile` now owns it. Execute the verdict's `revoke` via `kh.revoke_member(stored_identifier, …)`; map `v.cgka_rekey` into `ReconcileReport.cgka_rekey_needed`; `self.index.advance(doc, org, v.restamp_epoch, v.restamp_root)`.
- **`DocReconcileMark`:** replace the local `index::OrgEpochs` with core `OrgEpochs`/`DocReconcileMark` (the per-doc mark); keep the Keyhive-only `delegated: HashMap<OrgMember, Identifier>` map (the `Identifier` is GPL-side and stays).
- **`KeyhiveResolver` sub-trait:** define `pub trait KeyhiveResolver: MemberKeyResolver { fn contact_card(&self, id: &MemberId) -> Result<ContactCard, ResolverError>; }`; move `NoContactCard` onto its error path; implement R7 `find_member_by_device` on the resolver.
- **Per-doc shape:** keep ACL state per Keyhive Document (already per-doc); the writer remains the actor handle. Reserve the CGKA slot (item-4).
- **Tests:** existing `wrapper.rs`/`index.rs` suites stay green after retargeting onto core types; add the same reorg test p2panda has (`reorg_same_epoch_different_root_forces_reconcile`) — it should now pass where the old epoch-only check failed it.

### Plan 4 — facade, `org-node` repoint, conformance (depends on 2 + 3)
- **`org-acl` facade crate:** `features = ["p2panda", "keyhive"]`, **no default**; `compile_error!("enable exactly one backend")` when zero are enabled; re-export core + the selected backend.
- **Workspace assembly:** bring `org-acl-core` + `org-acl-p2panda` + `org-acl-keyhive` + `org-acl` into one workspace (the GPL `org-acl-keyhive` is the only GPL member). Resolve the two backends currently living in separate worktrees.
- **`org-node` repoint:** depend on the `org-acl` facade; implement `MemberKeyResolver` (incl. R7) against its per-org trie mirror; stop naming `keyhive_core` directly.
- **Conformance harness:** a shared scenario suite (revoke, reorg, delta-vs-full, owner-not-exempt) run against BOTH backends, asserting identical `effective_members` + revoke outcomes — the mechanical enforcement of strict parity.
- **Keyhive gate:** CI builds both backends; the `keyhive` feature carries the re-pin-to-0.1.0+audit risk note (spec §10).
- **`DocId`:** if a single neutral handle is wanted across the facade, introduce a backend associated type; otherwise consumers name the backend's `DocId` via the facade.

---

## Self-Review (Plan 1)

- **Spec coverage (Plan 1 slice):** shared spine §3 → Tasks 2+3; unified `AclError` §4 → Task 1; R7 into core resolver §5 → Task 4; non-serde witness §6 → already in core (no task needed; Plan 3 adopts it); fuzz (§9) → Task 5. Topology/facade §2, migration §8, items 4/2/5 §7, gate §10 → Plans 2–4 (roadmapped). No Plan-1 gap.
- **Placeholder scan:** none — every code step carries full content; the only prose-only items are the Plans 2–4 roadmap (explicitly deferred to their own plans).
- **Type consistency:** `decide_reconcile` signature and `ReconcileVerdict` fields are identical in the Task-3 interface block, the implementation, and the Plans 2–3 call-site descriptions. `MembershipDelta.removed`/`.rotated` are `Vec<MemberId>` (per the read of `delta.rs`); the verdict re-wraps them as `OrgMember { org: witness.org(), member }` — consistent across Task 3 and the Plan 3 migration notes. `find_member_by_device(&self, &P2pDeviceKey) -> Option<OrgMember>` matches in Task 4 and the Plan 2/3 consumption notes.
```
