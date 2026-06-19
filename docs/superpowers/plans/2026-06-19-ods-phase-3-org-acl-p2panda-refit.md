# Plan 2 — p2panda refit onto org-acl-core

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `org-acl-p2panda`'s `DocAccess` a thin p2panda *executor* over the Plan-1 core — routing the reconcile decision through `decide_reconcile`, the live access answer through `gate`, adopting the unified `AclError`, and keying its delegation surface on `OrgMember` — and prove cross-backend parity with shared conformance fixtures.

**Architecture:** `DocAccess` keeps its sync, per-doc, move-based shape. It no longer re-implements authority logic: `reconcile` calls `org_acl_core::decide_reconcile` and executes the returned `ReconcileVerdict`; `effective_members`/`is_effective` call `org_acl_core::gate`. Parity is enforced by a declarative fixture set in `org-acl-core::testing` that each backend's integration test runs.

**Tech Stack:** Rust, `org-acl-core` (path dep + `testing` dev-feature), `p2panda-auth` (`GroupCrdt`), `ed25519-dalek`.

## Global Constraints

- **Delegation level is `OrgMember`** — the grantee of `add_member`/`remove_member`/`set_access` is `OrgMember`; reject `grantee.org != resolver.org_id()` with `AclError::WrongOrg`. `ledger_members`/`effective_members` return `OrgMember`. The author gate stays `resolver.is_member` (author is a `MemberId` within the resolver's org).
- **Route every decision through core** — no authority logic re-implemented in `DocAccess`. `reconcile` → `decide_reconcile`; read side → `gate`.
- **Unified error** — use `org_acl_core::AclError`; delete the local enum. Map the CRDT rejection to `AclError::Backend("acl crdt rejected".into())`.
- **`DocAccess` stays move-based** (`self -> Result<Self>`); no `&mut` churn.
- **clippy `--lib` deny-gate** — no `unwrap`/`expect`/`panic!` in library code (test code may use `unwrap`).
- **Commits:** single-purpose, **no `Co-Authored-By` lines**.
- **Worktree:** all work on `worktree-phase-3-org-acl-convergence` (already created).

## Core API consumed (Plan-1, already on this branch)

- `org_acl_core::gate::effective_members<R, I>(ledger: I, resolver: &R) -> BTreeSet<OrgMember>` where `I: IntoIterator<Item = OrgMember>`.
- `org_acl_core::gate::is_effective<R, I>(ledger: I, resolver: &R, member: &OrgMember) -> bool`.
- `org_acl_core::decide_reconcile<R: MemberKeyResolver>(mark: &DocReconcileMark, resolver: &R, witness: &VerifiedTrieChange, ledger: &[OrgMember], retain: &[OrgMember]) -> Result<ReconcileVerdict, AclError>`.
- `ReconcileVerdict { mode: ReconcileMode, revoke: Vec<OrgMember>, cgka_rekey: Vec<OrgMember>, restamp_epoch: Epoch, restamp_root: RootHash }`.
- `AclError { AuthorNotMember(OrgMember), NotAMember(OrgMember), WrongOrg { expected: OrgId, got: OrgId }, StaleWitness { mark: Epoch, witness: Epoch }, Backend(String) }`.
- `DocReconcileMark::advance(&mut self, org: OrgId, epoch: Epoch, root: RootHash)`; `VerifiedTrieChange::{org, to_epoch, to_root}`.
- Test helpers (feature `testing`): `org_acl_core::testing::{StubResolver, root_for_epoch}` (and, after Task 1, `conformance_scenarios`, `ConformanceScenario`). `StubResolver` builder: `new(OrgId)`, `add_member(MemberId, P2pMemberKey, Vec<P2pDeviceKey>) -> Self`, `revoke(&MemberId) -> Self`, `epoch() -> Epoch`, `root_hash() -> RootHash`, `org_id() -> OrgId`, `is_member(&MemberId) -> bool`.

---

### Task 1: Shared conformance fixtures in `org-acl-core::testing`

**Files:**
- Create: `org-acl-core/src/conformance.rs`
- Modify: `org-acl-core/src/lib.rs` (declare the cfg-gated module + re-export under `testing`)
- Test: inline `#[cfg(test)]` self-consistency check in `conformance.rs`

**Interfaces:**
- Produces (under `org_acl_core::testing`):
  - `pub struct ConformanceScenario { pub name: &'static str, pub manager: u8, pub granted: &'static [u8], pub revoked_from_trie: &'static [u8], pub expected_effective: &'static [u8] }`
  - `pub fn conformance_scenarios() -> &'static [ConformanceScenario]`
- Semantics each backend test must honour: all of `manager` + `granted` are added to the trie and granted on the doc; then `revoked_from_trie` are revoked from the trie; after a full-reload reconcile, the live effective set (member seeds) must equal `expected_effective`. Seeds are `u8`; a backend maps seed `s` to `MemberId([s; 32])`.

- [ ] **Step 1: Write the fixtures + self-consistency test**

Create `org-acl-core/src/conformance.rs`:

```rust
//! Cross-backend conformance fixtures (the parity-enforcement artifact).
//!
//! Declarative scenarios that EVERY ACL backend's integration test runs against
//! its own concrete type, asserting identical observable outcomes. Seeds are
//! `u8`; a backend maps seed `s` to `MemberId([s; 32])`. The decision itself is
//! already shared (`decide_reconcile`); these fixtures pin that each backend's
//! EXECUTION of the verdict — and its live gate — agree.

/// One conformance scenario. Setup: add `manager` + every seed in `granted` to
/// the trie and grant them on a fresh doc owned by `manager`; then revoke every
/// seed in `revoked_from_trie` from the trie; then reconcile with a full-reload
/// witness to the trie's current epoch. After that, the live effective-member
/// set (as seeds) MUST equal `expected_effective`.
pub struct ConformanceScenario {
    pub name: &'static str,
    pub manager: u8,
    pub granted: &'static [u8],
    pub revoked_from_trie: &'static [u8],
    pub expected_effective: &'static [u8],
}

/// The shared scenario set. Covers: owner-only, grant-and-keep, revoke a member,
/// and owner-not-exempt (revoking the manager removes them from effective access
/// even though they are retained in the ledger).
pub fn conformance_scenarios() -> &'static [ConformanceScenario] {
    &[
        ConformanceScenario {
            name: "owner_only",
            manager: 0xa1,
            granted: &[],
            revoked_from_trie: &[],
            expected_effective: &[0xa1],
        },
        ConformanceScenario {
            name: "grant_and_keep",
            manager: 0xa1,
            granted: &[0xb1, 0xc1],
            revoked_from_trie: &[],
            expected_effective: &[0xa1, 0xb1, 0xc1],
        },
        ConformanceScenario {
            name: "revoke_one_member",
            manager: 0xa1,
            granted: &[0xb1],
            revoked_from_trie: &[0xb1],
            expected_effective: &[0xa1],
        },
        ConformanceScenario {
            name: "revoke_owner_not_exempt",
            manager: 0xa1,
            granted: &[0xb1],
            revoked_from_trie: &[0xa1],
            expected_effective: &[0xb1],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenarios_are_self_consistent() {
        for s in conformance_scenarios() {
            // expected_effective ⊆ (manager ∪ granted) and disjoint from non-retained revoked.
            let mut universe = vec![s.manager];
            universe.extend_from_slice(s.granted);
            for e in s.expected_effective {
                assert!(universe.contains(e), "{}: effective seed {e:#x} not in universe", s.name);
            }
            // A revoked member that is NOT the manager must not be effective.
            for r in s.revoked_from_trie {
                if *r != s.manager {
                    assert!(!s.expected_effective.contains(r), "{}: revoked {r:#x} still effective", s.name);
                }
            }
            // A revoked manager must not be effective either (owner not exempt).
            if s.revoked_from_trie.contains(&s.manager) {
                assert!(!s.expected_effective.contains(&s.manager), "{}: revoked owner still effective", s.name);
            }
        }
    }
}
```

Modify `org-acl-core/src/lib.rs` — extend the testing wiring. The file currently has:

```rust
#[cfg(any(test, feature = "testing"))]
mod test_support;

#[cfg(any(test, feature = "testing"))]
pub mod testing {
    pub use crate::test_support::{root_for_epoch, StubResolver};
}
```

Change it to:

```rust
#[cfg(any(test, feature = "testing"))]
mod test_support;

#[cfg(any(test, feature = "testing"))]
mod conformance;

#[cfg(any(test, feature = "testing"))]
pub mod testing {
    pub use crate::conformance::{conformance_scenarios, ConformanceScenario};
    pub use crate::test_support::{root_for_epoch, StubResolver};
}
```

- [ ] **Step 2: Run the test to verify it fails (module not yet wired)**

Run: `cargo test -p org-acl-core --features testing conformance::`
Expected: FAIL/compile error before the module + re-exports are in place; then PASS once both edits are present.

- [ ] **Step 3: Confirm it passes**

Run: `cargo test -p org-acl-core --features testing conformance::`
Expected: PASS (1 test).

- [ ] **Step 4: Lint + commit**

```bash
cargo clippy -p org-acl-core --lib --features testing -- -D warnings
git add org-acl-core/src/conformance.rs org-acl-core/src/lib.rs
git commit -m "test(org-acl-core): shared cross-backend conformance fixtures"
```

---

### Task 2: Refit `DocAccess` onto core

Rewrite `org-acl-p2panda/src/acl.rs` so `DocAccess` routes through core and keys delegation on `OrgMember`. The existing `acl.rs` test module is the behavioural spec — retarget it (it must stay green).

**Files:**
- Modify: `org-acl-p2panda/src/acl.rs` (imports, delete local `AclError`, rewrite methods, retarget tests)

**Interfaces:**
- Consumes: the Core API listed above.
- Produces (new public surface of `DocAccess`):
  - `create<R: MemberKeyResolver>(doc_id: MemberId, founder: MemberId, resolver: &R) -> Result<Self, AclError>` (unchanged signature; `AclError` now core's)
  - `add_member<R>(self, resolver: &R, author: MemberId, grantee: OrgMember, role: AclRole) -> Result<Self, AclError>`
  - `remove_member<R>(self, resolver: &R, author: MemberId, grantee: OrgMember) -> Result<Self, AclError>`
  - `set_access<R>(self, resolver: &R, author: MemberId, grantee: OrgMember, role: AclRole) -> Result<Self, AclError>`
  - `ledger_members(&self) -> Vec<OrgMember>`
  - `effective_members<R>(&self, resolver: &R) -> Vec<OrgMember>`
  - `is_effective<R>(&self, resolver: &R, member: OrgMember) -> bool`
  - `reconcile<R>(self, resolver: &R, witness: &VerifiedTrieChange) -> Result<Self, AclError>`
  - `mark(&self) -> &DocReconcileMark` (unchanged)

- [ ] **Step 1: Replace the imports and delete the local `AclError`**

In `org-acl-p2panda/src/acl.rs`, replace the existing `org_acl_core` import line:

```rust
use org_acl_core::{AclRole, DocReconcileMark, Epoch, MemberId, MemberKeyResolver, MembershipDelta, OrgEpochs, OrgId, OrgMember, RootHash, VerifiedTrieChange};
```

with:

```rust
use org_acl_core::{decide_reconcile, gate, AclError, AclRole, DocReconcileMark, MemberId, MemberKeyResolver, OrgMember, VerifiedTrieChange};
```

Delete the entire local `AclError` definition block:

```rust
/// Why an ACL operation was refused.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AclError {
    #[error("author {0:?} is not a current member of the org trie")]
    AuthorNotMember(MemberId),
    #[error("acl crdt rejected the operation")]
    CrdtRejected,
    #[error("witness org {got:?} does not match this resolver's org {expected:?}")]
    WrongOrg { expected: OrgId, got: OrgId },
    #[error("stale/replay witness: mark at {mark:?}, witness to {witness:?}")]
    StaleWitness { mark: Epoch, witness: Epoch },
}
```

(`AclError` is re-exported from `lib.rs` via `pub use acl::{AclError, DocAccess};` — that line keeps working because `AclError` is now imported into `acl.rs` from core and re-exported.)

- [ ] **Step 2: Rewrite the gate, mutators, CRDT-error mapping**

Replace the `gate` helper and the three mutators. `gate` now builds a core `AclError`:

```rust
    fn gate<R: MemberKeyResolver>(resolver: &R, author: MemberId) -> Result<(), AclError> {
        if resolver.is_member(&author) {
            Ok(())
        } else {
            Err(AclError::AuthorNotMember(OrgMember { org: resolver.org_id(), member: author }))
        }
    }
```

Map the CRDT rejection everywhere `apply`/`create` previously used `AclError::CrdtRejected`. Update `create` and `apply`:

```rust
        let state = AclCrdt::process(AclCrdt::init(), &op).map_err(|_| AclError::Backend("acl crdt rejected".into()))?;
```

```rust
    fn apply(mut self, author: AuthMemberId, action: GroupAction<AuthMemberId, ()>) -> Result<Self, AclError> {
        let id = OpId(self.next_op);
        let op = AclOp { id, author, dependencies: self.last.clone(), group_id: self.group, action };
        self.state = AclCrdt::process(self.state, &op).map_err(|_| AclError::Backend("acl crdt rejected".into()))?;
        self.next_op = self.next_op.saturating_add(1);
        self.last = vec![id];
        Ok(self)
    }
```

Rewrite the three mutators to take `grantee: OrgMember` and reject foreign orgs:

```rust
    /// `author` (a current trie member holding `Manage`) grants `grantee` access
    /// at `role`. Trie-gates the author; rejects a grantee in a different org
    /// (`WrongOrg`). The CRDT enforces the author's `Manage` rights (`Backend`).
    pub fn add_member<R: MemberKeyResolver>(self, resolver: &R, author: MemberId, grantee: OrgMember, role: AclRole) -> Result<Self, AclError> {
        Self::gate(resolver, author)?;
        if grantee.org != resolver.org_id() {
            return Err(AclError::WrongOrg { expected: resolver.org_id(), got: grantee.org });
        }
        let a = self.am(resolver.org_id(), author);
        let m = AuthMemberId(grantee);
        self.apply(a, GroupAction::Add { member: GroupMember::Individual(m), access: access_for(role) })
    }

    /// `author` (Manage) removes `grantee`'s ACL delegation. Trie-gates the author.
    pub fn remove_member<R: MemberKeyResolver>(self, resolver: &R, author: MemberId, grantee: OrgMember) -> Result<Self, AclError> {
        Self::gate(resolver, author)?;
        if grantee.org != resolver.org_id() {
            return Err(AclError::WrongOrg { expected: resolver.org_id(), got: grantee.org });
        }
        let a = self.am(resolver.org_id(), author);
        let m = AuthMemberId(grantee);
        self.apply(a, GroupAction::Remove { member: GroupMember::Individual(m) })
    }

    /// `author` (Manage) changes `grantee`'s access level (Promote/Demote by level
    /// delta; no-op if already at `role` or not in the ledger). Trie-gates the author.
    pub fn set_access<R: MemberKeyResolver>(self, resolver: &R, author: MemberId, grantee: OrgMember, role: AclRole) -> Result<Self, AclError> {
        Self::gate(resolver, author)?;
        if grantee.org != resolver.org_id() {
            return Err(AclError::WrongOrg { expected: resolver.org_id(), got: grantee.org });
        }
        let target = access_for(role);
        let m = AuthMemberId(grantee);
        let current = self
            .state
            .members(self.group)
            .into_iter()
            .find(|(id, _)| *id == m)
            .map(|(_, acc)| acc.level);
        let a = self.am(resolver.org_id(), author);
        match current {
            None => Ok(self),
            Some(cur) => match target.level.cmp(&cur) {
                std::cmp::Ordering::Equal => Ok(self),
                std::cmp::Ordering::Greater => self.apply(a, GroupAction::Promote { member: GroupMember::Individual(m), access: target }),
                std::cmp::Ordering::Less => self.apply(a, GroupAction::Demote { member: GroupMember::Individual(m), access: target }),
            },
        }
    }
```

- [ ] **Step 3: Rewrite the read side (gate) and `ledger_members`**

```rust
    /// The advisory CRDT ledger members (org-scoped `OrgMember`s) — NOT the access
    /// answer. A member here may be trie-revoked; use `effective_members` /
    /// `is_effective` for the authoritative live check.
    pub fn ledger_members(&self) -> Vec<OrgMember> {
        self.state.members(self.group).into_iter().map(|(a, _)| a.0).collect()
    }

    /// Effective members = advisory ledger ∩ currently-trie-vouched, computed LIVE
    /// via the shared core gate (no cache, owner not exempt).
    pub fn effective_members<R: MemberKeyResolver>(&self, resolver: &R) -> Vec<OrgMember> {
        gate::effective_members(self.ledger_members(), resolver).into_iter().collect()
    }

    /// True iff `member` is in the ledger AND currently trie-vouched (shared gate).
    pub fn is_effective<R: MemberKeyResolver>(&self, resolver: &R, member: OrgMember) -> bool {
        gate::is_effective(self.ledger_members(), resolver, &member)
    }
```

- [ ] **Step 4: Rewrite `reconcile` through `decide_reconcile`; delete `reconcile_delta`/`reconcile_full`**

Replace the whole `reconcile` method AND delete the `reconcile_delta` and `reconcile_full` helpers:

```rust
    /// Bring this document's ACL up to a VERIFIED on-chain change. The
    /// `VerifiedTrieChange` is the unforgeable capability minted by org-node's
    /// verify-against-chain accept branch. The accept/reject decision (org match,
    /// monotonicity + same-epoch reorg, delta-vs-full, departed-member set) is the
    /// shared `decide_reconcile`; this method only EXECUTES the resulting verdict
    /// against the CRDT and advances the mark. The manager (owner) is in `retain`,
    /// so it is never auto-pruned (a revoked manager is made inert by the gate).
    pub fn reconcile<R: MemberKeyResolver>(
        mut self,
        resolver: &R,
        witness: &VerifiedTrieChange,
    ) -> Result<Self, AclError> {
        let ledger = self.ledger_members();
        let retain = [self.manager.0];
        let verdict = decide_reconcile(&self.mark, resolver, witness, &ledger, &retain)?;
        let org = witness.org();
        let mgr = self.manager;
        for om in verdict.revoke {
            self = self.apply(mgr, GroupAction::Remove { member: GroupMember::Individual(AuthMemberId(om)) })?;
        }
        self.mark.advance(org, verdict.restamp_epoch, verdict.restamp_root);
        Ok(self)
    }
```

After this step the module no longer references `MembershipDelta`, `OrgEpochs`, `Epoch`, `RootHash`, or `OrgId` in non-test code — confirm none remain (the import line in Step 1 already dropped them).

- [ ] **Step 5: Retarget the `#[cfg(test)] mod tests`**

Add an `om` helper next to the existing test consts (the test module already imports `OrgMember`):

```rust
    fn om(m: MemberId) -> OrgMember { OrgMember { org: ORG, member: m } }
```

Apply these mechanical substitutions across EVERY existing test in the module (the tests already exist; transform them in place):

1. `.add_member(&r, AUTHOR, MEMBER, role)` → `.add_member(&r, AUTHOR, om(MEMBER), role)` (wrap the grantee, e.g. `BOB` → `om(BOB)`). `AUTHOR` stays a bare `MemberId`.
2. `.remove_member(&r, AUTHOR, MEMBER)` → `.remove_member(&r, AUTHOR, om(MEMBER))`.
3. `.set_access(&r, AUTHOR, MEMBER, role)` → `.set_access(&r, AUTHOR, om(MEMBER), role)`.
4. `doc.is_effective(&r, MEMBER)` → `doc.is_effective(&r, om(MEMBER))`.
5. `ledger_members()` / `effective_members()` expectations: every expected `MemberId` becomes `om(...)`. E.g. `vec![ALICE, BOB]` → `vec![om(ALICE), om(BOB)]`; `vec![ALICE]` → `vec![om(ALICE)]`; `.contains(&BOB)` → `.contains(&om(BOB))`. Sorting still works (`OrgMember: Ord`).
6. `create(DOC, ALICE, &r)` is unchanged (`founder` stays a `MemberId`).
7. `AclError` variant patterns are unchanged in name — `Err(AclError::AuthorNotMember(_))`, `Err(AclError::WrongOrg { .. })`, `Err(AclError::StaleWitness { .. })` all still compile (the core enum uses the same names; `AuthorNotMember` now wraps `OrgMember`, which `_` still matches).

Worked examples (the transformed form these existing tests must take):

```rust
    #[test]
    fn grant_stores_stable_ids_not_keys() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(0xa1), vec![]);
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap().add_member(&r, ALICE, om(BOB), AclRole::Write).unwrap();
        let mut ids = doc.ledger_members();
        ids.sort();
        let mut expected = vec![om(ALICE), om(BOB)];
        expected.sort();
        assert_eq!(ids, expected);
    }

    #[test]
    fn effective_excludes_revoked_owner_not_exempt() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(0xa1), vec![]);
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap();   // ALICE is owner/manager
        let r = r.revoke(&ALICE);                                // trie revokes the owner
        assert!(doc.ledger_members().contains(&om(ALICE)));      // ledger still lists the owner
        assert!(doc.effective_members(&r).is_empty(), "revoked owner is NOT exempt");
        assert!(!doc.is_effective(&r, om(ALICE)));
    }

    #[test]
    fn set_access_by_non_member_author_is_rejected() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(0xa1), vec![]).add_member(BOB, mkey(0xb1), vec![]);
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap().add_member(&r, ALICE, om(BOB), AclRole::Read).unwrap();
        assert!(matches!(
            doc.set_access(&r, CAROL, om(BOB), AclRole::Manage),
            Err(AclError::AuthorNotMember(_))
        ));
    }
```

The `acl_op_embeds_stable_id_bytes_not_ed25519_key` test builds `GroupAction`/`AuthMemberId` literals directly — it is unaffected (no `DocAccess` method signature involved); leave it as-is.

- [ ] **Step 6: Build, test, lint**

Run: `cargo test -p org-acl-p2panda` (the dev-dep enables `org-acl-core/testing`)
Expected: PASS — every retargeted test green.

Run: `cargo clippy -p org-acl-p2panda --lib -- -D warnings`
Expected: no warnings (confirm no unused imports remain from the dropped types).

If cargo fails on a read-only `~/.cargo`, prefix with `CARGO_HOME=/tmp/cargo_home_fuzz`.

- [ ] **Step 7: Commit**

```bash
git add org-acl-p2panda/src/acl.rs
git commit -m "refit(org-acl-p2panda): DocAccess routes through core gate + decide_reconcile, OrgMember surface"
```

---

### Task 3: p2panda conformance test

Add the integration test that runs the Task-1 fixtures against `DocAccess`, proving the p2panda backend's execution matches the shared expectations.

**Files:**
- Create: `org-acl-p2panda/tests/conformance.rs`

**Interfaces:**
- Consumes: `org_acl_core::testing::{conformance_scenarios, StubResolver}`, `org_acl_core::{AclRole, MemberId, OrgId, OrgMember, P2pMemberKey, VerifiedTrieChange}`, `org_acl_p2panda::DocAccess`.

- [ ] **Step 1: Write the conformance test**

Create `org-acl-p2panda/tests/conformance.rs`:

```rust
//! p2panda backend conformance: run the shared `org-acl-core` scenario fixtures
//! against `DocAccess` and assert the live effective-member set matches. This is
//! the p2panda half of the cross-backend parity guarantee (the Keyhive half runs
//! the same fixtures in Plan 3).

use ed25519_dalek::SigningKey;
use org_acl_core::testing::{conformance_scenarios, StubResolver};
use org_acl_core::{AclRole, MemberId, OrgId, OrgMember, P2pMemberKey, VerifiedTrieChange};
use org_acl_p2panda::DocAccess;

const ORG: OrgId = OrgId([0xaa; 20]);
const DOC: MemberId = MemberId([0xdd; 32]);

fn mid(seed: u8) -> MemberId { MemberId([seed; 32]) }
fn om(seed: u8) -> OrgMember { OrgMember { org: ORG, member: mid(seed) } }
fn mkey(seed: u8) -> P2pMemberKey { P2pMemberKey(SigningKey::from_bytes(&[seed; 32]).verifying_key()) }

#[test]
fn p2panda_matches_conformance_scenarios() {
    for s in conformance_scenarios() {
        // Build a trie containing the manager + every granted member.
        let mut r = StubResolver::new(ORG).add_member(mid(s.manager), mkey(s.manager), vec![]);
        for &g in s.granted {
            r = r.add_member(mid(g), mkey(g), vec![]);
        }
        // Create the doc owned by the manager, then grant each member.
        let mut doc = DocAccess::create(DOC, mid(s.manager), &r).unwrap_or_else(|e| panic!("{}: create: {e:?}", s.name));
        for &g in s.granted {
            doc = doc.add_member(&r, s.manager, om(g), AclRole::Write).unwrap_or_else(|e| panic!("{}: grant {g:#x}: {e:?}", s.name));
        }
        // Revoke the scheduled members from the trie.
        for &rv in s.revoked_from_trie {
            r = r.revoke(&mid(rv));
        }
        // If anything was revoked the trie epoch advanced; reconcile (full-reload to
        // the current epoch). With no revocations the epoch is unchanged, so a witness
        // would be a stale replay — skip reconcile (the gate is live regardless).
        if !s.revoked_from_trie.is_empty() {
            let w = VerifiedTrieChange::accept_full_reload(ORG, r.epoch(), r.root_hash());
            doc = doc.reconcile(&r, &w).unwrap_or_else(|e| panic!("{}: reconcile: {e:?}", s.name));
            // Non-retained revoked members must be pruned from the advisory ledger.
            for &rv in s.revoked_from_trie {
                if rv != s.manager {
                    assert!(!doc.ledger_members().contains(&om(rv)), "{}: {rv:#x} not pruned from ledger", s.name);
                }
            }
        }
        // The authoritative assertion: the live effective set equals the expected set.
        let mut got = doc.effective_members(&r);
        got.sort();
        let mut want: Vec<OrgMember> = s.expected_effective.iter().map(|&seed| om(seed)).collect();
        want.sort();
        assert_eq!(got, want, "{}: effective mismatch", s.name);
    }
}
```

- [ ] **Step 2: Run the conformance test**

Run: `cargo test -p org-acl-p2panda --test conformance`
Expected: PASS (all four scenarios).

If cargo fails on a read-only `~/.cargo`, prefix with `CARGO_HOME=/tmp/cargo_home_fuzz`.

- [ ] **Step 3: Commit**

```bash
git add org-acl-p2panda/tests/conformance.rs
git commit -m "test(org-acl-p2panda): run shared conformance fixtures against DocAccess"
```

---

## Self-Review

- **Spec coverage (§3):** route reconcile through `decide_reconcile` → Task 2 Step 4; effective via `gate` → Task 2 Step 3; adopt core `AclError` → Task 2 Steps 1-2; `OrgMember` delegation surface + `WrongOrg` guard → Task 2 Step 2; author the shared conformance fixtures → Task 1; p2panda conformance test → Task 3. R7: verified no in-crate `MemberKeyResolver` impl exists in `org-acl-p2panda` (`ResolverPki` implements p2panda's `IdentityRegistry`, not our trait), so no R7 work here — `StubResolver` already implements it; the real resolver is `org-node`'s (Plan 4). No gap.
- **Placeholder scan:** none — full code for every production change and the conformance fixtures/test; the test-retarget is a defined mechanical transform of pre-existing tests with worked examples, not a "write tests for the above".
- **Type consistency:** `add_member`/`remove_member`/`set_access` take `grantee: OrgMember`; `ledger_members`/`effective_members` return `Vec<OrgMember>`; `is_effective` takes `OrgMember`; `decide_reconcile(&mark, resolver, witness, &ledger, &retain)` with `retain = [self.manager.0]` (an `OrgMember`); these match Task 3's call sites (`om(seed)`, `effective_members(&r) -> Vec<OrgMember>`). `AclError` variant names (`AuthorNotMember`, `WrongOrg`, `StaleWitness`, `Backend`) match the core enum consumed.
```
