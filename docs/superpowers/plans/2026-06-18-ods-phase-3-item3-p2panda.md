# ODS Phase 3 item-3 on p2panda — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the members trie the sole write authority: identity/key state is trie-only (asserted), and document-ACL writes are author-gated (author must be a current trie member; rights enforced natively by `GroupCrdt`), with effective membership computed live as `ledger ∩ resolver.is_member` (advisory ACL / authoritative trie, owner not exempt).

**Architecture:** Revises item-1's `org-acl-core` + `org-acl-p2panda` (on a branch stacked on `worktree-phase-3-item1-p2panda`). Adds a neutral `AclRole` to core; makes `DocAccess` mutations author-aware (`add_member`/`remove_member`/`set_access` take `resolver` + `author`, gated on `is_member`); adds `effective_members`/`is_effective`; keeps `reconcile` as manager-authored lazy materialisation. **No p2panda fork, no `BlockingGroups`** (the raw `GroupCrdt`/`Groups` surface is private — encapsulation already disables library-native mutation).

**Tech Stack:** Rust (workspace), p2panda at rev `41559b0` (`GroupAction::{Create,Add,Remove,Promote,Demote}`, `Access::{read,write,manage}`, `AccessLevel: Ord` = Pull<Read<Write<Manage), `ed25519-dalek`, `bolero`. Build with `CARGO_HOME=/tmp/cargo_home_fuzz`.

Spec: [`docs/superpowers/specs/2026-06-18-ods-phase-3-item3-p2panda-design.md`](../specs/2026-06-18-ods-phase-3-item3-p2panda-design.md).

---

## Worktree (do this first)

Item-1 is unmerged; item-3 stacks on it.
```bash
cd /Users/jan-jan/Coding/2-tier-access-control
git worktree add -b worktree-phase-3-item3-p2panda .claude/worktrees/phase-3-item3-p2panda worktree-phase-3-item1-p2panda
cd .claude/worktrees/phase-3-item3-p2panda
git config extensions.worktreeConfig true && git config --worktree commit.gpgsign false
```
All cargo/git below run from that worktree; prefix cargo with `CARGO_HOME=/tmp/cargo_home_fuzz`.

## File structure (this item)

```
org-acl-core/src/role.rs        # NEW: AclRole { Read, Write, Manage }
org-acl-core/src/lib.rs         # export AclRole
org-acl-core/tests/fuzz_identity_codec/fuzz_target.rs   # extend: AclRole codec
org-acl-p2panda/src/acl.rs      # AclError->enum; author-aware mutators; set_access; effective_members/is_effective; reconcile manager-authored via apply(author,…)
org-acl-p2panda/tests/l3_revocation.rs   # updated to author-aware API + effective-membership assertions
org-acl-p2panda/README.md       # note the authority model
org-acl-core/README.md          # note AclRole + advisory/authoritative model
```

---

## Task 0: Stack the worktree + confirm the access-change API at the pin

**Files:** none (verification only).

- [ ] **Step 1: Create the stacked worktree** (commands in "Worktree" above). Confirm `git log --oneline -1` shows item-1's HEAD (`a7e7e03…`).

- [ ] **Step 2: Confirm baseline is green**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-core --features testing && CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda`
Expected: all pass (item-1 state).

- [ ] **Step 3: Confirm the p2panda-auth access API at rev 41559b0**

The p2panda-auth source is in the cargo git checkout. Confirm (read, don't guess):
```bash
AUTH=$(find /tmp/cargo_home_fuzz/git/checkouts -type d -name p2panda-auth | grep 41559b0 | head -1)
grep -n "Promote\|Demote" "$AUTH/src/group/action.rs"   # expect Promote{member,access} + Demote{member,access}
grep -n "PartialOrd, Ord\|pub enum AccessLevel\|pub level" "$AUTH/src/access.rs"  # AccessLevel: Ord; Access{pub level, pub conditions}
```
Expected: `GroupAction::Promote { member, access }` and `GroupAction::Demote { member, access }` exist; `AccessLevel` derives `Ord` (Pull<Read<Write<Manage); `Access<C>` has `pub level: AccessLevel`. If any differs, reconcile `set_access` (Task 3) accordingly and note it. **No commit this task.**

---

## Task 1: `org-acl-core` — `AclRole`

**Files:** Create `org-acl-core/src/role.rs`; Modify `org-acl-core/src/lib.rs`.

- [ ] **Step 1: Declare the module** — append to `org-acl-core/src/lib.rs`:
```rust
pub mod role;
pub use role::AclRole;
```

- [ ] **Step 2: Write the failing test** — create `org-acl-core/src/role.rs` with the test module first:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acl_role_postcard_roundtrip() {
        for r in [AclRole::Read, AclRole::Write, AclRole::Manage] {
            let bytes = postcard::to_allocvec(&r).unwrap();
            let back: AclRole = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(r, back);
        }
    }
}
```

- [ ] **Step 3: Run to verify it fails**
Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-core --lib role`
Expected: FAIL — `AclRole` not defined.

- [ ] **Step 4: Write the type** — prepend above the test module:
```rust
//! `AclRole` — the neutral, substrate-agnostic document-ACL access level.
//!
//! Shared convergence type: both ACL backends map it to their substrate's access
//! level (p2panda's `Access`, Keyhive's capability). Ordered Read < Write < Manage;
//! only `Manage` may mutate a document's ACL.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AclRole {
    Read,
    Write,
    Manage,
}
```

- [ ] **Step 5: Run to verify it passes**
Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-core --lib role`
Expected: PASS (1 test).

- [ ] **Step 6: Extend the codec fuzz** — in `org-acl-core/tests/fuzz_identity_codec/fuzz_target.rs`, add `AclRole` to the import and add a never-panic decode line inside the existing arbitrary-bytes `check!().for_each(|input: &[u8]| { … })` block:
```rust
        let _ = postcard::from_bytes::<org_acl_core::AclRole>(input);
```
(Import: change the `use org_acl_core::{…}` line to include `AclRole`.) Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-core --test fuzz_identity_codec` → passes.

- [ ] **Step 7: Commit**
```bash
git add org-acl-core/src/role.rs org-acl-core/src/lib.rs org-acl-core/tests/fuzz_identity_codec/fuzz_target.rs
git commit -m "feat(org-acl-core): AclRole — neutral document-ACL access level"
```

---

## Task 2: `org-acl-p2panda` — author-aware, trie-gated mutation (revises item-1)

**Files:** Modify `org-acl-p2panda/src/acl.rs` (and update its in-file tests + `tests/l3_revocation.rs`).

This is the cross-cutting API change: `AclError` becomes an enum; `DocAccess` mutations take `resolver` + `author`; `reconcile` authors via a private `apply(author, …)`. Existing item-1 tests are updated in the same task to keep the crate green.

- [ ] **Step 1: Replace `AclError` + add the role mapping + author-aware `apply`**

In `org-acl-p2panda/src/acl.rs`:
- Change the imports: add `AclRole` and `Epoch`/`RootHash` are already used via `org_acl_core::`. Update the `use org_acl_core::{…}` line to:
```rust
use org_acl_core::{AclRole, DocReconcileMark, MemberId, MemberKeyResolver, MembershipDelta, OrgMember};
```
- Replace the `AclError` struct (lines ~60-63) with an enum:
```rust
/// Why an ACL operation was refused.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AclError {
    /// The op author is not a current member of the org trie (trie write-gate).
    #[error("author {0:?} is not a current member of the org trie")]
    AuthorNotMember(MemberId),
    /// The CRDT refused the op — typically the author lacks `Manage` rights.
    #[error("acl crdt rejected the operation")]
    CrdtRejected,
}
```
- Add a free helper mapping `AclRole` → `Access<()>` (below the type aliases):
```rust
fn access_for(role: AclRole) -> Access<()> {
    match role {
        AclRole::Read => Access::read(),
        AclRole::Write => Access::write(),
        AclRole::Manage => Access::manage(),
    }
}
```
- Change the two existing `.map_err(|_| AclError)?` calls (in `create` and `apply`) to `.map_err(|_| AclError::CrdtRejected)?`.
- Replace the private `apply` so it takes an explicit `author: AuthMemberId`:
```rust
    fn apply(mut self, author: AuthMemberId, action: GroupAction<AuthMemberId, ()>) -> Result<Self, AclError> {
        let id = OpId(self.next_op);
        let op = AclOp { id, author, dependencies: self.last.clone(), group_id: self.group, action };
        self.state = AclCrdt::process(self.state, &op).map_err(|_| AclError::CrdtRejected)?;
        self.next_op = self.next_op.saturating_add(1);
        self.last = vec![id];
        Ok(self)
    }
```
- Add three private helpers to the `impl DocAccess` block:
```rust
    fn gate<R: MemberKeyResolver>(resolver: &R, author: MemberId) -> Result<(), AclError> {
        if resolver.is_member(&author) { Ok(()) } else { Err(AclError::AuthorNotMember(author)) }
    }
    fn am(&self, member: MemberId) -> AuthMemberId {
        AuthMemberId(OrgMember { org: self.mark.org, member })
    }
```

- [ ] **Step 2: Rewrite `create` (founder trie-gated)**
Replace `create`'s body so the founder is trie-gated and is the manager:
```rust
    /// Create a document ACL within the resolver's org. `founder` (a current trie
    /// member) becomes the initial `Manage` holder. The mark is stamped from the
    /// resolver's current trie state.
    pub fn create<R: MemberKeyResolver>(
        doc_id: MemberId,
        founder: MemberId,
        resolver: &R,
    ) -> Result<Self, AclError> {
        Self::gate(resolver, founder)?;
        let org = resolver.org_id();
        let group = AuthMemberId(OrgMember { org, member: doc_id });
        let manager = AuthMemberId(OrgMember { org, member: founder });
        let op = AclOp {
            id: OpId(0),
            author: manager,
            dependencies: vec![],
            group_id: group,
            action: GroupAction::Create {
                initial_members: vec![(GroupMember::Individual(manager), Access::manage())],
            },
        };
        let state = AclCrdt::process(AclCrdt::init(), &op).map_err(|_| AclError::CrdtRejected)?;
        let epoch = resolver.epoch();
        let mark = DocReconcileMark { org, acl_epoch: epoch, cgka_epoch: epoch, root_hash: resolver.root_hash() };
        Ok(Self { group, manager, state, next_op: 1, last: vec![OpId(0)], mark })
    }
```

- [ ] **Step 3: Rewrite `add_member` / `remove_member` (author-aware, trie-gated)**
```rust
    /// `author` (a current trie member holding `Manage`) grants `member` access at
    /// `role`. Trie-gates the author (`AuthorNotMember`); the CRDT enforces the
    /// author's `Manage` rights (`CrdtRejected`). The grantee is NOT gated — a
    /// non-member grantee is simply never *effective* (see `effective_members`).
    pub fn add_member<R: MemberKeyResolver>(self, resolver: &R, author: MemberId, member: MemberId, role: AclRole) -> Result<Self, AclError> {
        Self::gate(resolver, author)?;
        let a = self.am(author);
        let m = self.am(member);
        self.apply(a, GroupAction::Add { member: GroupMember::Individual(m), access: access_for(role) })
    }

    /// `author` (Manage) removes `member`'s ACL delegation. Trie-gates the author.
    pub fn remove_member<R: MemberKeyResolver>(self, resolver: &R, author: MemberId, member: MemberId) -> Result<Self, AclError> {
        Self::gate(resolver, author)?;
        let a = self.am(author);
        let m = self.am(member);
        self.apply(a, GroupAction::Remove { member: GroupMember::Individual(m) })
    }
```

- [ ] **Step 4: Point `reconcile_delta`/`reconcile_full` at the manager-authored `apply`**
They currently call `self.remove_member(*m)` (old signature). Change each removal to author as the manager directly. In `reconcile_delta`, replace the loop body:
```rust
        let current = self.member_ids();
        let mgr = self.manager;
        for m in &d.removed {
            if *m != self.manager.0.member && current.contains(m) {
                let target = self.am(*m);
                self = self.apply(mgr, GroupAction::Remove { member: GroupMember::Individual(target) })?;
            }
        }
```
In `reconcile_full`, replace the loop:
```rust
        let mgr = self.manager;
        for m in self.member_ids() {
            if m != self.manager.0.member && !resolver.is_member(&m) {
                let target = self.am(m);
                self = self.apply(mgr, GroupAction::Remove { member: GroupMember::Individual(target) })?;
            }
        }
```
(The `reconcile` doc comment already explains the manager isn't pruned at the ledger level and `effective_members` covers the revoked-manager case — leave it.)

- [ ] **Step 5: Update item-1's in-file tests to the author-aware API**

In `org-acl-p2panda/src/acl.rs`'s `#[cfg(test)] mod tests`, apply this mechanical transformation (the resolver/consts/`mkey` already exist; add `use org_acl_core::AclRole;` to the test module):
- Anywhere a `DocAccess::create(DOC, ALICE, &r)` runs, ensure ALICE is a trie member in `r` first (add `.add_member(ALICE, mkey(0xa1), vec![])` to the resolver builder if not already present). The founder gate now requires it.
- `…​.add_member(BOB)` → `…​.add_member(&r, ALICE, BOB, AclRole::Write)` (ALICE is the founder/manager/author). For chained builders where `r` is rebound later, use the resolver in scope at that call.
- `…​.remove_member(BOB)` → `…​.remove_member(&r, ALICE, BOB)`.
- `AclError` matches: the `acl_op_embeds_stable_id_bytes_not_ed25519_key` test constructs an `AclOp` directly (unaffected). Any `.unwrap()` on results is unaffected.

Concretely, the four reconcile tests and the grant/remove tests must thread `&r, ALICE`. Example — `grant_stores_stable_ids_not_keys` becomes:
```rust
    #[test]
    fn grant_stores_stable_ids_not_keys() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(0xa1), vec![]);
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap()
            .add_member(&r, ALICE, BOB, AclRole::Write).unwrap();
        let mut ids = doc.member_ids(); ids.sort();
        let mut expected = vec![ALICE, BOB]; expected.sort();
        assert_eq!(ids, expected);
    }
```
And e.g. `reconcile_delta_prunes_removed_member` keeps its delta logic but its setup becomes:
```rust
        let r = StubResolver::new(ORG)
            .add_member(ALICE, mkey(0xa1), vec![])   // epoch 1
            .add_member(BOB, mkey(0xb1), vec![]);     // epoch 2
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap()
            .add_member(&r, ALICE, BOB, AclRole::Write).unwrap();
```
(`create` consumes one epoch's worth of resolver state only by READING it — it does not mutate the resolver, so the epoch/root used for the mark is `r`'s current epoch; the existing assertions on `mark().acl_epoch` still hold because ALICE+BOB were added before `create`.)
Apply the same threading to `create_stamps_mark_from_resolver`, `remove_member_drops_from_acl` (now `remove_member(&r, ALICE, BOB)`), `reconcile_noop_when_epoch_and_root_match`, `reconcile_full_rebuild_on_epoch_gap`, `reconcile_ignores_noncontiguous_delta_and_falls_back_to_full`, `reorg_*`, `delta_with_mismatched_from_root_falls_back_to_full`, `rotation_only_delta_advances_mark_without_membership_change`, `revoked_manager_is_retained_by_reconcile`.

- [ ] **Step 6: Update `tests/l3_revocation.rs` to the author-aware API**
- Add `use org_acl_core::AclRole;`.
- The resolver already adds ALICE+BOB; `DocAccess::create(DOC, ALICE, &resolver)` is fine (ALICE is a member). Change `.add_member(BOB)` → `.add_member(&resolver, ALICE, BOB, AclRole::Write)`.

- [ ] **Step 7: Build + test + clippy**
```
CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda
CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-acl-p2panda --lib -- -D warnings
```
Expected: all pass; clippy clean (the gate helper returns `Result`, no unwrap/expect/panic in lib).

- [ ] **Step 8: Commit**
```bash
git add org-acl-p2panda/src/acl.rs org-acl-p2panda/tests/l3_revocation.rs
git commit -m "feat(org-acl-p2panda)!: author-aware trie-gated ACL mutation (AuthorNotMember vs CrdtRejected)"
```

---

## Task 3: `set_access` (Promote/Demote, trie-gated)

**Files:** Modify `org-acl-p2panda/src/acl.rs`.

- [ ] **Step 1: Write the failing tests** (append to the `tests` module):
```rust
    #[test]
    fn manager_can_relevel_a_member() {
        let r = StubResolver::new(ORG)
            .add_member(ALICE, mkey(0xa1), vec![])
            .add_member(BOB, mkey(0xb1), vec![]);
        // ALICE (founder/Manage) grants BOB Read, then promotes BOB to Write, then demotes to Read.
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap()
            .add_member(&r, ALICE, BOB, AclRole::Read).unwrap()
            .set_access(&r, ALICE, BOB, AclRole::Write).unwrap()
            .set_access(&r, ALICE, BOB, AclRole::Read).unwrap();
        // BOB is still a member after relevel round-trip.
        assert!(doc.member_ids().contains(&BOB));
    }

    #[test]
    fn set_access_same_level_is_noop() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(0xa1), vec![]).add_member(BOB, mkey(0xb1), vec![]);
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap()
            .add_member(&r, ALICE, BOB, AclRole::Write).unwrap()
            .set_access(&r, ALICE, BOB, AclRole::Write).unwrap(); // no-op
        assert!(doc.member_ids().contains(&BOB));
    }

    #[test]
    fn set_access_by_non_member_author_is_rejected() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(0xa1), vec![]).add_member(BOB, mkey(0xb1), vec![]);
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap().add_member(&r, ALICE, BOB, AclRole::Read).unwrap();
        // CAROL is not in the trie → AuthorNotMember.
        const CAROL: MemberId = MemberId([0xc2; 32]);
        assert!(matches!(
            doc.set_access(&r, CAROL, BOB, AclRole::Manage),
            Err(AclError::AuthorNotMember(_))
        ));
    }
```

- [ ] **Step 2: Run to verify it fails**
Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --lib acl`
Expected: FAIL — `set_access` not defined.

- [ ] **Step 3: Implement `set_access`** (add to `impl DocAccess`, after `remove_member`):
```rust
    /// `author` (Manage) changes `member`'s access level. Trie-gates the author;
    /// dispatches the CRDT `Promote`/`Demote` based on the level delta (no-op if
    /// the member already holds `role`). If `member` is not in the ledger, no-op.
    pub fn set_access<R: MemberKeyResolver>(self, resolver: &R, author: MemberId, member: MemberId, role: AclRole) -> Result<Self, AclError> {
        Self::gate(resolver, author)?;
        let target = access_for(role);
        let m = self.am(member);
        let current = self
            .state
            .members(self.group)
            .into_iter()
            .find(|(id, _)| *id == m)
            .map(|(_, acc)| acc.level);
        let a = self.am(author);
        match current {
            None => Ok(self),
            Some(cur) => match target.level.cmp(&cur) {
                std::cmp::Ordering::Equal => Ok(self),
                std::cmp::Ordering::Greater => {
                    self.apply(a, GroupAction::Promote { member: GroupMember::Individual(m), access: target })
                }
                std::cmp::Ordering::Less => {
                    self.apply(a, GroupAction::Demote { member: GroupMember::Individual(m), access: target })
                }
            },
        }
    }
```
(`Access.level` is `AccessLevel` which is `Ord`; `state.members(group)` returns `Vec<(AuthMemberId, Access<()>)>` — confirmed Task 0.)

- [ ] **Step 4: Run to verify it passes**
Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --lib acl`
Expected: PASS (the 3 new tests + all prior acl tests).

- [ ] **Step 5: Commit**
```bash
git add org-acl-p2panda/src/acl.rs
git commit -m "feat(org-acl-p2panda): DocAccess::set_access — trie-gated Promote/Demote"
```

---

## Task 4: `effective_members` / `is_effective` (advisory ∩ authoritative, live)

**Files:** Modify `org-acl-p2panda/src/acl.rs`.

- [ ] **Step 1: Write the failing tests** (append to the `tests` module):
```rust
    #[test]
    fn effective_excludes_trie_revoked_member_before_reconcile() {
        let r = StubResolver::new(ORG)
            .add_member(ALICE, mkey(0xa1), vec![])
            .add_member(BOB, mkey(0xb1), vec![]);
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap().add_member(&r, ALICE, BOB, AclRole::Write).unwrap();
        assert!(doc.member_ids().contains(&BOB));           // advisory ledger still lists BOB
        let r = r.revoke(&BOB);                              // trie revokes BOB
        // No reconcile yet: effective membership already excludes BOB.
        let mut eff = doc.effective_members(&r); eff.sort();
        assert_eq!(eff, vec![ALICE]);
        assert!(!doc.is_effective(&r, BOB));
        assert!(doc.is_effective(&r, ALICE));
    }

    #[test]
    fn effective_excludes_revoked_owner_not_exempt() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(0xa1), vec![]);
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap();   // ALICE is owner/manager
        let r = r.revoke(&ALICE);                                // trie revokes the owner
        assert!(doc.member_ids().contains(&ALICE));              // ledger still lists the owner
        assert!(doc.effective_members(&r).is_empty(), "revoked owner is NOT exempt");
        assert!(!doc.is_effective(&r, ALICE));
    }

    #[test]
    fn effective_is_stable_across_reconcile() {
        let r = StubResolver::new(ORG)
            .add_member(ALICE, mkey(0xa1), vec![])
            .add_member(BOB, mkey(0xb1), vec![]);
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap().add_member(&r, ALICE, BOB, AclRole::Write).unwrap();
        let r = r.revoke(&BOB); // epoch 3
        let before = { let mut e = doc.effective_members(&r); e.sort(); e };
        let doc = doc.reconcile(&r, None).unwrap();              // materialise
        let after = { let mut e = doc.effective_members(&r); e.sort(); e };
        assert_eq!(before, after, "reconcile must not change the live effective answer");
        assert_eq!(after, vec![ALICE]);
    }
```

- [ ] **Step 2: Run to verify it fails**
Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --lib acl`
Expected: FAIL — `effective_members`/`is_effective` not defined.

- [ ] **Step 3: Implement** (add to `impl DocAccess`, after `member_ids`):
```rust
    /// Effective members = the advisory ledger ∩ the authoritative trie, computed
    /// LIVE on every call (no cache, no owner exemption). A member revoked from the
    /// trie is excluded immediately, independent of `reconcile`.
    pub fn effective_members<R: MemberKeyResolver>(&self, resolver: &R) -> Vec<MemberId> {
        self.member_ids().into_iter().filter(|m| resolver.is_member(m)).collect()
    }

    /// True iff `member` is in the ledger AND currently vouched by the trie.
    pub fn is_effective<R: MemberKeyResolver>(&self, resolver: &R, member: MemberId) -> bool {
        resolver.is_member(&member) && self.member_ids().contains(&member)
    }
```

- [ ] **Step 4: Run to verify it passes**
Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --lib acl`
Expected: PASS (3 new + all prior).

- [ ] **Step 5: Commit**
```bash
git add org-acl-p2panda/src/acl.rs
git commit -m "feat(org-acl-p2panda): effective_members/is_effective — live advisory∩authoritative (owner not exempt)"
```

---

## Task 5: Guarantee-1 invariant doc + READMEs + full suite + clippy

**Files:** Modify `org-acl-p2panda/src/acl.rs` (doc), `org-acl-p2panda/src/lib.rs` (doc), `org-acl-core/README.md`, `org-acl-p2panda/README.md`.

- [ ] **Step 1: Record the Guarantee-1 invariant** — append to the `DocAccess` module/struct doc in `acl.rs` (doc-only, no logic):
```rust
//! # Trie is the sole write authority (item-3)
//!
//! Guarantee 1 — identity/key state is trie-only: `DocAccess` stores stable
//! `OrgMember` ids and access levels, never keys; the `MemberKeyResolver` is
//! read-only; the raw `GroupCrdt`/`Groups` mutation surface is private to this
//! crate. No `org-acl` API mutates member/device/org-key state — that flows only
//! from the members trie (org-node / on-chain).
//!
//! Guarantee 2 — ACL writes are trie-authorized: every mutator trie-gates its
//! `author` (`AuthorNotMember`) and relies on the CRDT for `Manage` rights
//! (`CrdtRejected`). Effective access is the live `ledger ∩ is_member`
//! (`effective_members`), owner not exempt; `reconcile` is lazy materialisation,
//! not the boundary.
```

- [ ] **Step 2: Add a compile-level Guarantee-1 assertion test** (append to the `tests` module in `acl.rs`):
```rust
    // Guarantee-1 witness: the only public mutators on DocAccess are the
    // author-gated ACL ops; there is no API that writes member/device/org KEY
    // state (keys are resolver-read-only). This test documents that a revoked
    // member's KEY cannot be re-introduced via any DocAccess call — the ledger
    // holds ids only, and effective access defers to the trie.
    #[test]
    fn doc_access_never_introduces_key_state() {
        let r = StubResolver::new(ORG).add_member(ALICE, mkey(0xa1), vec![]);
        let doc = DocAccess::create(DOC, ALICE, &r).unwrap()
            .add_member(&r, ALICE, BOB, AclRole::Write).unwrap(); // BOB has no key in the trie
        // Granting BOB did not make BOB a trie member / give BOB a key:
        assert!(!r.is_member(&BOB));
        assert!(!doc.is_effective(&r, BOB));
    }
```
Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --lib acl` → passes.

- [ ] **Step 3: Update READMEs**
- `org-acl-p2panda/README.md`: add a short "Write authority (item-3)" paragraph — ACL writes are author-gated (author must be a current trie member; `Manage` enforced by the CRDT); effective access = `ledger ∩ is_member` live (owner not exempt); `reconcile` is lazy materialisation; no fork.
- `org-acl-core/README.md`: add `AclRole` to the listed types and one line that the ACL is advisory over the authoritative trie.

- [ ] **Step 4: Full suite + clippy gate**
```
CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-core --features testing
CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda
CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-acl-core --lib -- -D warnings
CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-acl-p2panda --lib -- -D warnings
CARGO_HOME=/tmp/cargo_home_fuzz cargo build -p org-acl-core -p org-acl-p2panda
```
Expected: all green / clean.

- [ ] **Step 5: Commit**
```bash
git add org-acl-p2panda/src/acl.rs org-acl-p2panda/src/lib.rs org-acl-core/README.md org-acl-p2panda/README.md
git commit -m "docs(org-acl): item-3 authority-model invariants + READMEs"
```

---

## Final review

- [ ] Dispatch a code reviewer over the whole item-3 diff (spec §3 compliance: identity/key state trie-only; author-gated writes `AuthorNotMember` vs native `CrdtRejected`; `effective_members` = live `ledger ∩ is_member` with owner NOT exempt; `reconcile` still manager-authored materialisation; no fork; `set_access` Promote/Demote correct). Confirm item-1's regression tests still pass under the author-aware API.
- [ ] Then **superpowers:finishing-a-development-branch**. Because item-3 is stacked on the unmerged item-1 branch, the squash-merge target is the item-1 branch (or master after item-1 lands) — a single **user-signed** commit per AGENTS.md.

## Notes
- **No p2panda fork / no `BlockingGroups`** (spec §6 S1/S2): the spaces-`AuthStore` layer is never used; the raw `Groups`/`GroupCrdt` surface is private — encapsulation disables library-native mutation.
- **Deferred:** key/device/org-key rotation cascades (CGKA) = item-4; org pseudo-group as an ACL member = item-2; custom access `conditions` (`Access<()>`) out of scope.
- Convergence: `AclRole` + the advisory-ACL/authoritative-trie authority model are shared with the Keyhive track.
