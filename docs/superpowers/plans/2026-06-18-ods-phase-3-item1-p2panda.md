# ODS Phase 3 item-1 on p2panda — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a p2panda-backed stable-ID ACL with trie-lookup key resolution — a neutral `org-acl-core` contract + an `org-acl-p2panda` boundary crate — so an ACL binds to immutable trie identities (`MemberId` = handle@org) while keys resolve live through the trie, blocking identity-takeover and making revocation effective at the ACL layer.

**Architecture:** Two new crates. `org-acl-core` (Apache-2.0, no substrate deps) graduates `spike-common`'s identity types + the minimal `MemberKeyResolver` contract — the convergence anchor `org-node` implements once. `org-acl-p2panda` (GPL-3.0-only, the sole p2panda boundary) productionises `spike-p2panda::s1_stable_id_acl`: `AuthMemberId` + `IdentityHandle`, `ResolverPki` (`IdentityRegistry`), `materialise_actor_id`, and a `GroupCrdt<AuthMemberId>` grant path driven **below** `p2panda-spaces`' eager `Manager` (lazy CGKA). No `IdAdapter` cache (re-resolve gives the Flow-B invariant for free); no trie-change observer (item-4).

**Tech Stack:** Rust (edition 2021, workspace), `ed25519-dalek` v2, `serde`/`postcard`, p2panda (`p2panda-core`/`-auth`/`-encryption`/`-spaces` at rev `41559b0dfc2d7d0e9e4fba251ceb7f8094ff8be1`), `bolero` (fuzz), `ciborium` (CBOR test).

Spec: [`docs/superpowers/specs/2026-06-18-ods-phase-3-item1-p2panda-design.md`](../specs/2026-06-18-ods-phase-3-item1-p2panda-design.md).

---

## EXECUTION GATE (precondition)

- [x] **(a) Phase 2 `org-node` has landed** — on `master` (commit `2bb1c21f5`) with the committed `OrgTrie` mirror. Satisfied.
- [ ] **(b) Substrate maturity** — relaxed to a *risk note* for this parallel track (spec §2). Ride the spike pin `41559b0`; **no fork** required for item-1. Task 0 re-confirms reachability at the pin; it is a sanity check, **not** a decision-reopening go/no-go.

## File structure (this item)

```
org-acl-core/
  Cargo.toml          # Apache-2.0; workspace member; [[test]] fuzz_identity_codec
  src/
    lib.rs            # module decls + crate docs
    identity.rs       # MemberId, P2pMemberKey, P2pDeviceKey, OrgKey, Epoch, Principal
    resolver.rs       # MemberKeyResolver trait, ResolverError
    test_support.rs   # StubResolver (behind feature `testing`)
  tests/
    fuzz_identity_codec/{fuzz_target.rs, corpus/.gitkeep, crashes/.gitkeep}

org-acl-p2panda/
  Cargo.toml          # GPL-3.0-only; workspace member; p2panda pins; [[test]] fuzz_resolver_key_boundary
  src/
    lib.rs            # crate docs: Flow-B invariant + lazy-CGKA two-tier spine
    g1_probe.rs       # #[cfg(test)] reachability probe (unit test; sees [dependencies])
    auth_id.rs        # AuthMemberId(MemberId) + impl IdentityHandle + From<MemberId>
    pki.rs            # ResolverPki<R>: impl IdentityRegistry<MemberId, Self>
    actor.rs          # materialise_actor_id(resolver, principal) -> ActorId
    acl.rs            # OpId, AclOp, OrgAcl: GroupCrdt<AuthMemberId> grant path
  tests/
    l3_revocation.rs              # exit criteria: revocation + identity-takeover blocked
    fuzz_resolver_key_boundary/{fuzz_target.rs, corpus/.gitkeep, crashes/.gitkeep}
```

Workspace `Cargo.toml` gains `"org-acl-core"` and `"org-acl-p2panda"` in `members`.

**Worktree (do this first):** all work happens in a dedicated worktree per AGENTS.md.

```bash
cd /Users/jan-jan/Coding/2-tier-access-control
git worktree add -b worktree-phase-3-item1-p2panda .claude/worktrees/phase-3-item1-p2panda master
cd .claude/worktrees/phase-3-item1-p2panda
git config extensions.worktreeConfig true && git config --worktree commit.gpgsign false
```

All `cargo`/`git` commands below run from inside that worktree. Prefix every cargo invocation with `CARGO_HOME=/tmp/cargo_home_fuzz` (read-only `~/.cargo`).

---

## Task 0: Scaffold both crates + G1 reachability re-confirm probe

**Files:**
- Create: `org-acl-core/Cargo.toml`, `org-acl-core/src/lib.rs`
- Create: `org-acl-p2panda/Cargo.toml`, `org-acl-p2panda/src/lib.rs`
- Create: `org-acl-p2panda/tests/g1_reachability.rs`
- Modify: `Cargo.toml` (workspace `members`)

- [ ] **Step 1: Minimal core crate**

`org-acl-core/Cargo.toml`:

```toml
[package]
name = "org-acl-core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license = "Apache-2.0"
description = "ODS organisation ACL — neutral identity + resolver contract (substrate-agnostic)"
publish = false

[features]
# Exposes StubResolver for integration tests here and in org-acl-p2panda / org-node.
testing = []

[dependencies]
ed25519-dalek = { version = "2", default-features = false, features = ["alloc", "serde"] }
serde = { version = "1", features = ["derive"] }
postcard = { version = "1", features = ["use-std"] }
thiserror = "2"

[dev-dependencies]
bolero = "0.13"
postcard = { version = "1", features = ["use-std"] }

[lints]
workspace = true

[[test]]
name = "fuzz_identity_codec"
path = "tests/fuzz_identity_codec/fuzz_target.rs"
harness = false
```

`org-acl-core/src/lib.rs`:

```rust
//! ODS organisation ACL — the neutral, substrate-agnostic contract.
//!
//! This crate has NO local-first-substrate dependencies. It is the convergence
//! anchor shared by every ACL backend (`org-acl-p2panda` today; a Keyhive
//! `org-acl` later) and implemented once by `org-node` over its trie mirror.
```

- [ ] **Step 2: Minimal p2panda crate (deps mirror the spike exactly)**

`org-acl-p2panda/Cargo.toml`:

```toml
[package]
name = "org-acl-p2panda"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license = "GPL-3.0-only"
description = "ODS organisation ACL — p2panda substitution layer (the GPL boundary crate)"
publish = false

[dependencies]
org-acl-core = { path = "../org-acl-core" }
ed25519-dalek = { version = "2", default-features = false, features = ["alloc"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"

p2panda-core       = { git = "https://github.com/p2panda/p2panda", rev = "41559b0dfc2d7d0e9e4fba251ceb7f8094ff8be1", default-features = false }
p2panda-auth       = { git = "https://github.com/p2panda/p2panda", rev = "41559b0dfc2d7d0e9e4fba251ceb7f8094ff8be1", default-features = false, features = ["serde"] }
p2panda-encryption = { git = "https://github.com/p2panda/p2panda", rev = "41559b0dfc2d7d0e9e4fba251ceb7f8094ff8be1", default-features = false, features = ["data_scheme"] }
p2panda-spaces     = { git = "https://github.com/p2panda/p2panda", rev = "41559b0dfc2d7d0e9e4fba251ceb7f8094ff8be1" }

[dev-dependencies]
org-acl-core = { path = "../org-acl-core", features = ["testing"] }
ed25519-dalek = { version = "2", default-features = false, features = ["alloc"] }
bolero = "0.13"
ciborium = "0.2"
serde_json = "1"

[lints]
workspace = true

[[test]]
name = "fuzz_resolver_key_boundary"
path = "tests/fuzz_resolver_key_boundary/fuzz_target.rs"
harness = false
```

`org-acl-p2panda/src/lib.rs`:

```rust
//! ODS organisation ACL — the p2panda substitution layer.
//!
//! This crate is the ONLY one in the workspace that names `p2panda-*`; it
//! confines p2panda's surface behind one boundary. `org-node` depends on
//! `org-acl-core`, not on this crate directly.
//!
//! # Flow-B invariant (item 1)
//!
//! No code path may read a `VerifyingKey` for a `Principal` except through a
//! `MemberKeyResolver`. There is NO cache: `materialise_actor_id` and
//! `ResolverPki` re-query the resolver on every call, so a rotated key is
//! always picked up and a forged key the trie does not vouch for resolves to
//! nothing. This is the structural enforcement that blocks identity takeover.
//!
//! # Lazy-CGKA two-tier model
//!
//! * ACL tier — trie-anchored stable identity (`MemberId` = handle@org). The
//!   grant runs on `p2panda_auth::GroupCrdt<AuthMemberId>` (see [`acl`]) BELOW
//!   `p2panda-spaces`' eager `Manager`, so adding a member does NOT force
//!   prekey/DCGKA placement at add-time.
//! * CGKA tier — per-document `p2panda-encryption` DCGKA, computed lazily when
//!   a member's client first comes online. `ResolverPki` ([`pki`]) and
//!   `materialise_actor_id` ([`actor`]) feed this tier. The trigger itself is
//!   item-4 and is out of scope here.
```

- [ ] **Step 3: Register both crates in the workspace**

Edit `/Users/jan-jan/Coding/2-tier-access-control/Cargo.toml` `members` array (use the `Edit` tool, not `sed`) to insert the two crates after `"org-node",`:

```toml
members = [
    "org-members",
    "org-node",
    "org-acl-core",
    "org-acl-p2panda",
    "spike-common",
    "spike-keyhive",
    "spike-p2panda",
]
```

- [ ] **Step 4: Write the G1 reachability probe (a `#[cfg(test)]` unit test)**

Declare the probe module in `org-acl-p2panda/src/lib.rs` (a unit test, so it sees the crate's `[dependencies]` — an integration test in `tests/` would not):

```rust
#[cfg(test)]
mod g1_probe;
```

`org-acl-p2panda/src/g1_probe.rs` — re-confirms the spike's gate-1 sub-flows compile and run at the pin, using throwaway local types (independent of the real crate types built in later tasks):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! G1 go/no-go (sanity): the three item-1 sub-flows are reachable at pin 41559b0.
//!  (1) stable-ID ACL — GroupCrdt::process with a newtype ID (Create + Add).
//!  (2) key resolution — IdentityRegistry is externally implementable (x25519).
//!  (3) spaces seam   — ActorId::from(VerifyingKey) constructs.
//! If this regresses at a future re-pin, reconcile before the dependent tasks.

use serde::{Deserialize, Serialize};

use p2panda_auth::group::resolver::StrongRemove;
use p2panda_auth::group::{GroupAction, GroupCrdt, GroupCrdtState, GroupMember};
use p2panda_auth::traits::{IdentityHandle, Operation, OperationId};
use p2panda_auth::Access;
use p2panda_core::identity::VerifyingKey as PandaVerifyingKey;
use p2panda_encryption::crypto::x25519::PublicKey as X25519PublicKey;
use p2panda_encryption::traits::IdentityRegistry;
use p2panda_spaces::ActorId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
struct Id([u8; 32]);
impl IdentityHandle for Id {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
struct Oid(u32);
impl OperationId for Oid {}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Op {
    id: Oid,
    author: Id,
    dependencies: Vec<Oid>,
    group_id: Id,
    action: GroupAction<Id, ()>,
}
impl Operation<Id, Oid, ()> for Op {
    fn id(&self) -> Oid { self.id }
    fn author(&self) -> Id { self.author }
    fn dependencies(&self) -> Vec<Oid> { self.dependencies.clone() }
    fn group_id(&self) -> Id { self.group_id }
    fn action(&self) -> GroupAction<Id, ()> { self.action.clone() }
}

type St = GroupCrdtState<Id, Oid, Op, ()>;
type Crdt = GroupCrdt<Id, Oid, Op, (), StrongRemove<Id, Oid, Op, ()>>;

const ALICE: Id = Id([0xa1; 32]);
const BOB: Id = Id([0xb1; 32]);
const GROUP: Id = Id([0xc1; 32]);

#[test]
fn g1_stable_id_acl_reachable() {
    let y: St = Crdt::init();
    let op0 = Op { id: Oid(0), author: ALICE, dependencies: vec![], group_id: GROUP,
        action: GroupAction::Create { initial_members: vec![(GroupMember::Individual(ALICE), Access::manage())] } };
    let y = Crdt::process(y, &op0).unwrap();
    let op1 = Op { id: Oid(1), author: ALICE, dependencies: vec![op0.id], group_id: GROUP,
        action: GroupAction::Add { member: GroupMember::Individual(BOB), access: Access::manage() } };
    let y = Crdt::process(y, &op1).unwrap();
    let ids: Vec<Id> = y.members(GROUP).into_iter().map(|(id, _)| id).collect();
    assert!(ids.contains(&ALICE) && ids.contains(&BOB));
}

// (2) An external IdentityRegistry impl compiles and returns an x25519 key.
struct Pki;
impl IdentityRegistry<Id, Pki> for Pki {
    type Error = core::convert::Infallible;
    fn identity_key(_y: &Pki, _id: &Id) -> Result<Option<X25519PublicKey>, Self::Error> {
        Ok(Some(X25519PublicKey::from_bytes([7u8; 32])))
    }
}

#[test]
fn g1_key_resolution_and_actor_seam_reachable() {
    assert!(Pki::identity_key(&Pki, &ALICE).unwrap().is_some());
    // (3) ActorId constructs from an ed25519 verifying key (via p2panda-core).
    let vk = ed25519_dalek::SigningKey::from_bytes(&[3u8; 32]).verifying_key();
    let _actor = ActorId::from(PandaVerifyingKey::from(vk));
}
```

(The probe uses `ed25519-dalek`, `serde`, and the `p2panda-*` crates — all in `org-acl-p2panda`'s `[dependencies]`, which a `#[cfg(test)]` unit test can use directly.)

- [ ] **Step 5: Run the probe**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --lib g1`
Expected: PASS (2 tests). If the p2panda API shifted from the spike, reconcile the imports against the rev; if a sub-flow is genuinely gone, STOP and escalate (the spike at `41559b0` proves all three).

- [ ] **Step 6: Commit**

```bash
git add org-acl-core/ org-acl-p2panda/ Cargo.toml
git commit -m "feat(org-acl): scaffold core + p2panda crates; G1 reachability probe (pin 41559b0)"
```

---

## Task 1: `org-acl-core` identity types (simple single-org)

**Files:**
- Create: `org-acl-core/src/identity.rs`
- Modify: `org-acl-core/src/lib.rs`

- [ ] **Step 1: Declare the module**

Append to `org-acl-core/src/lib.rs`:

```rust
pub mod identity;
pub use identity::{Epoch, MemberId, OrgKey, P2pDeviceKey, P2pMemberKey, Principal};
```

- [ ] **Step 2: Write the failing tests**

Create `org-acl-core/src/identity.rs` (tests first; types added next step):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_id_postcard_roundtrip() {
        let id = MemberId([7u8; 32]);
        let bytes = postcard::to_allocvec(&id).unwrap();
        let back: MemberId = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn principal_postcard_roundtrip() {
        for p in [Principal::Member(MemberId([1; 32])), Principal::Org] {
            let bytes = postcard::to_allocvec(&p).unwrap();
            let back: Principal = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(p, back);
        }
    }

    #[test]
    fn epoch_ordering() {
        assert!(Epoch(0) < Epoch(1));
        assert!(Epoch(u64::MAX) > Epoch(u64::MAX - 1));
    }
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-core --lib identity`
Expected: FAIL — types not defined.

- [ ] **Step 4: Write the types (prepend above the test module)**

```rust
//! Identity types for org-acl. PII-free; no handles.
//!
//! `MemberId` is the stable, org-scoped ACL identity (the SMT leaf key); it is
//! the PII-free representation of a member's handle@org. The human handle ↔
//! `MemberId` mapping lives in `org-members`/`org-node`, outside this crate.
//! The resolver instance is bound to one org, so a bare `MemberId` is the
//! delegation target (the "@org" is the resolver's context).

use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

/// 32-byte immutable, org-scoped member identifier (the SMT leaf key).
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MemberId(pub [u8; 32]);

/// Member-as-a-group key (ed25519 verifying key).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct P2pMemberKey(pub VerifyingKey);

/// Per-device verifying key (the CGKA tier).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct P2pDeviceKey(pub VerifyingKey);

/// Organisation-as-a-pseudo-group key (ed25519 verifying key).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrgKey(pub VerifyingKey);

/// Monotonic epoch counter for trie/CGKA versioning.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Epoch(pub u64);

/// Opaque principal — the ACL delegation target. The library obtains a key only
/// via [`MemberKeyResolver`](crate::MemberKeyResolver); never a raw key in an ACL.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Principal {
    Member(MemberId),
    Org,
}
```

- [ ] **Step 5: Run to verify it passes**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-core --lib identity`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add org-acl-core/src/identity.rs org-acl-core/src/lib.rs
git commit -m "feat(org-acl-core): simple single-org identity types"
```

---

## Task 2: `org-acl-core` MemberKeyResolver + StubResolver

**Files:**
- Create: `org-acl-core/src/resolver.rs`
- Create: `org-acl-core/src/test_support.rs`
- Modify: `org-acl-core/src/lib.rs`

- [ ] **Step 1: Declare the modules**

Append to `org-acl-core/src/lib.rs`:

```rust
pub mod resolver;
pub use resolver::{MemberKeyResolver, ResolverError};

// Test-only `MemberKeyResolver` stub — compiled under `cfg(test)` and behind
// `--features testing` (for integration tests here, in `org-acl-p2panda`, and
// in `org-node`). The real resolver is `org-node`'s trie mirror. `test_support`
// is a plain top-level module file (`src/test_support.rs`); `testing` re-exports
// it under a stable public path.
#[cfg(any(test, feature = "testing"))]
mod test_support;

#[cfg(any(test, feature = "testing"))]
pub mod testing {
    pub use crate::test_support::StubResolver;
}
```

- [ ] **Step 2: Write the trait + error**

`org-acl-core/src/resolver.rs`:

```rust
//! The `MemberKeyResolver` contract — org-acl's seam with the trie.
//!
//! This is the MINIMAL, substrate-neutral contract. Each backend extends it by
//! wrapping (p2panda's `ResolverPki`) or sub-trait-ing (a future Keyhive
//! `KeyhiveResolver: MemberKeyResolver` adding `contact_card`), never by adding
//! substrate-specific methods here. Each impl is bound to ONE org's trie mirror;
//! `org-node` holds one resolver per org record.

use crate::identity::{Epoch, MemberId, OrgKey, P2pDeviceKey, P2pMemberKey};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ResolverError {
    /// `id` is not in the trie (not a member). Security-relevant.
    #[error("member not in trie: {0:?}")]
    UnknownMember(MemberId),

    #[error("org key not set")]
    OrgKeyUnset,
}

pub trait MemberKeyResolver {
    /// Current member-as-a-group key for `id`.
    fn p2p_member_key(&self, id: &MemberId) -> Result<P2pMemberKey, ResolverError>;

    /// Current org pseudo-group key.
    fn org_key(&self) -> Result<OrgKey, ResolverError>;

    /// Currently-authorised devices for `id`. `Ok(vec![])` if the member exists
    /// but is isolated (the lazy-onboarding "not online yet" state — NOT a
    /// security failure); `Err(UnknownMember)` if not in the trie.
    fn current_devices(&self, id: &MemberId) -> Result<Vec<P2pDeviceKey>, ResolverError>;

    /// IDs of all current members of the org (item-2/item-5 fan-out).
    fn org_member_ids(&self) -> Vec<MemberId>;

    fn is_member(&self, id: &MemberId) -> bool;

    fn epoch(&self) -> Epoch;
}
```

- [ ] **Step 3: Write the StubResolver (graduated from `spike-common::stub_trie`)**

`org-acl-core/src/test_support.rs`:

```rust
//! In-memory `MemberKeyResolver` for tests — NOT a real SMT (the trie lives in
//! `org-members`). Behind `--features testing`.

use std::collections::HashMap;

use crate::identity::{Epoch, MemberId, OrgKey, P2pDeviceKey, P2pMemberKey};
use crate::resolver::{MemberKeyResolver, ResolverError};

#[derive(Clone, Debug, Default)]
pub struct StubResolver {
    members: HashMap<MemberId, MemberEntry>,
    org: Option<OrgKey>,
    epoch: Epoch,
}

#[derive(Clone, Debug)]
struct MemberEntry {
    p2p_key: P2pMemberKey,
    devices: Vec<P2pDeviceKey>,
}

impl StubResolver {
    pub fn new() -> Self {
        Self::default()
    }

    fn bump(mut self) -> Self {
        self.epoch.0 += 1;
        self
    }

    pub fn add_member(mut self, id: MemberId, p2p_key: P2pMemberKey, devices: Vec<P2pDeviceKey>) -> Self {
        self.members.insert(id, MemberEntry { p2p_key, devices });
        self.bump()
    }

    pub fn with_org_key(mut self, key: OrgKey) -> Self {
        self.org = Some(key);
        self.bump()
    }

    pub fn revoke(mut self, id: &MemberId) -> Self {
        self.members.remove(id);
        self.bump()
    }

    pub fn rotate_member_key(mut self, id: &MemberId, key: P2pMemberKey) -> Self {
        if let Some(e) = self.members.get_mut(id) {
            e.p2p_key = key;
        }
        self.bump()
    }
}

impl MemberKeyResolver for StubResolver {
    fn p2p_member_key(&self, id: &MemberId) -> Result<P2pMemberKey, ResolverError> {
        self.members.get(id).map(|e| e.p2p_key).ok_or(ResolverError::UnknownMember(*id))
    }
    fn org_key(&self) -> Result<OrgKey, ResolverError> {
        self.org.ok_or(ResolverError::OrgKeyUnset)
    }
    fn current_devices(&self, id: &MemberId) -> Result<Vec<P2pDeviceKey>, ResolverError> {
        self.members.get(id).map(|e| e.devices.clone()).ok_or(ResolverError::UnknownMember(*id))
    }
    fn org_member_ids(&self) -> Vec<MemberId> {
        self.members.keys().copied().collect()
    }
    fn is_member(&self, id: &MemberId) -> bool {
        self.members.contains_key(id)
    }
    fn epoch(&self) -> Epoch {
        self.epoch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn mkey(seed: u8) -> P2pMemberKey {
        P2pMemberKey(SigningKey::from_bytes(&[seed; 32]).verifying_key())
    }

    #[test]
    fn unknown_vs_isolated_are_distinct() {
        let alice = MemberId([1; 32]);
        let r = StubResolver::new().add_member(alice, mkey(1), vec![]);
        // In trie, isolated (lazy-pending): Ok(empty), NOT an error.
        assert_eq!(r.current_devices(&alice), Ok(vec![]));
        // Not in trie:
        let bob = MemberId([2; 32]);
        assert_eq!(r.current_devices(&bob), Err(ResolverError::UnknownMember(bob)));
        assert_eq!(r.p2p_member_key(&bob), Err(ResolverError::UnknownMember(bob)));
    }

    #[test]
    fn revoke_removes_member() {
        let alice = MemberId([1; 32]);
        let r = StubResolver::new().add_member(alice, mkey(1), vec![]);
        assert!(r.is_member(&alice));
        let r = r.revoke(&alice);
        assert!(!r.is_member(&alice));
        assert_eq!(r.p2p_member_key(&alice), Err(ResolverError::UnknownMember(alice)));
    }
}
```

`test_support.rs` references `ed25519-dalek` in its `#[cfg(test)]` module; it is already a normal dependency. (When compiled under `--features testing` for downstream crates, the `tests` module is `cfg(test)`-gated and does not build.)

- [ ] **Step 4: Run to verify it passes**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-core --lib`
Expected: PASS (identity 3 + test_support 2).

- [ ] **Step 5: Commit**

```bash
git add org-acl-core/src/resolver.rs org-acl-core/src/test_support.rs org-acl-core/src/lib.rs
git commit -m "feat(org-acl-core): minimal MemberKeyResolver + StubResolver"
```

---

## Task 3: `org-acl-core` fuzz — Principal/MemberId codec

**Files:**
- Create: `org-acl-core/tests/fuzz_identity_codec/fuzz_target.rs`
- Create: `org-acl-core/tests/fuzz_identity_codec/corpus/.gitkeep`, `crashes/.gitkeep`

- [ ] **Step 1: Write the fuzz target**

`org-acl-core/tests/fuzz_identity_codec/fuzz_target.rs`:

```rust
//! Fuzz: the stable ID that indexes the ACL must round-trip exactly, and
//! decoding arbitrary bytes must never panic. `harness = false` — a panic
//! (bolero's failure signal) exits non-zero and fails `cargo test`.
//! Deep-fuzz: `cargo bolero test fuzz_identity_codec --engine libfuzzer`.

use bolero::check;
use org_acl_core::{MemberId, Principal};

fn main() {
    // (1) Structured round-trip.
    check!().with_type::<[u8; 32]>().cloned().for_each(|raw| {
        let id = MemberId(raw);
        let bytes = postcard::to_allocvec(&id).expect("encode MemberId");
        let back: MemberId = postcard::from_bytes(&bytes).expect("decode MemberId");
        assert_eq!(id, back);

        let p = Principal::Member(id);
        let pb = postcard::to_allocvec(&p).expect("encode Principal");
        let pback: Principal = postcard::from_bytes(&pb).expect("decode Principal");
        assert_eq!(p, pback);
    });

    // (2) Never-panic on arbitrary bytes at the decode boundary.
    check!().for_each(|input: &[u8]| {
        let _ = postcard::from_bytes::<MemberId>(input);
        let _ = postcard::from_bytes::<Principal>(input);
    });
}
```

`postcard` must be available to this test. It is already a normal dependency of `org-acl-core`.

- [ ] **Step 2: Create corpus/crashes dirs**

```bash
mkdir -p org-acl-core/tests/fuzz_identity_codec/corpus org-acl-core/tests/fuzz_identity_codec/crashes
touch org-acl-core/tests/fuzz_identity_codec/corpus/.gitkeep org-acl-core/tests/fuzz_identity_codec/crashes/.gitkeep
```

- [ ] **Step 3: Run it**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-core --test fuzz_identity_codec`
Expected: PASS (bounded generated batch + empty corpus replay; no panic).

- [ ] **Step 4: Commit**

```bash
git add org-acl-core/tests/fuzz_identity_codec/
git commit -m "test(org-acl-core): bolero fuzz — Principal/MemberId codec never-panic + round-trip"
```

---

## Task 4: `org-acl-p2panda` AuthMemberId + IdentityHandle

**Files:**
- Create: `org-acl-p2panda/src/auth_id.rs`
- Modify: `org-acl-p2panda/src/lib.rs`

- [ ] **Step 1: Declare the module**

Append to `org-acl-p2panda/src/lib.rs`:

```rust
pub mod auth_id;
pub use auth_id::AuthMemberId;
```

- [ ] **Step 2: Write the failing test**

Create `org-acl-p2panda/src/auth_id.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use org_acl_core::MemberId;
    use p2panda_auth::traits::IdentityHandle;

    fn assert_identity_handle<T: IdentityHandle>(_id: T) {}

    #[test]
    fn auth_member_id_satisfies_identity_handle() {
        let a = AuthMemberId::from(MemberId([0xa1; 32]));
        assert_identity_handle(a);
    }

    #[test]
    fn from_member_id_preserves_bytes() {
        let m = MemberId([7; 32]);
        let a = AuthMemberId::from(m);
        assert_eq!(a.0, m);
    }
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --lib auth_id`
Expected: FAIL — `AuthMemberId` not defined.

- [ ] **Step 4: Write the type (prepend above the test module)**

```rust
//! `AuthMemberId` — the p2panda-auth ACL identity.
//!
//! Orphan-rule newtype: `IdentityHandle` (p2panda-auth) and `MemberId`
//! (org-acl-core) are both foreign to this crate, so the auth-layer ID needs a
//! LOCAL newtype to carry `impl IdentityHandle`. The ACL stores `AuthMemberId`
//! (the stable trie identity) — never a key.

use serde::{Deserialize, Serialize};

use org_acl_core::MemberId;
use p2panda_auth::traits::IdentityHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AuthMemberId(pub MemberId);

impl IdentityHandle for AuthMemberId {}

impl From<MemberId> for AuthMemberId {
    fn from(m: MemberId) -> Self {
        Self(m)
    }
}
```

`MemberId` derives `Serialize`/`Deserialize` and the full `Copy + Debug + Eq + Ord + Hash` set (Task 1), so the derives above compile and satisfy `IdentityHandle`.

- [ ] **Step 5: Run to verify it passes**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --lib auth_id`
Expected: PASS (2 tests).

- [ ] **Step 6: Commit**

```bash
git add org-acl-p2panda/src/auth_id.rs org-acl-p2panda/src/lib.rs
git commit -m "feat(org-acl-p2panda): AuthMemberId + IdentityHandle (orphan-rule newtype)"
```

---

## Task 5: `org-acl-p2panda` ResolverPki (the key-resolution half)

**Files:**
- Create: `org-acl-p2panda/src/pki.rs`
- Modify: `org-acl-p2panda/src/lib.rs`

Productionises `spike-p2panda::s1_stable_id_acl::ResolverPki`.

- [ ] **Step 1: Declare the module**

Append to `org-acl-p2panda/src/lib.rs`:

```rust
pub mod pki;
pub use pki::ResolverPki;
```

- [ ] **Step 2: Write the failing tests**

Create `org-acl-p2panda/src/pki.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use org_acl_core::testing::StubResolver;
    use org_acl_core::{MemberId, OrgKey, P2pMemberKey};
    use p2panda_encryption::traits::IdentityRegistry;

    fn mkey(seed: u8) -> P2pMemberKey {
        P2pMemberKey(SigningKey::from_bytes(&[seed; 32]).verifying_key())
    }

    #[test]
    fn resolves_known_member_to_some_key() {
        let alice = MemberId([1; 32]);
        let r = StubResolver::new().add_member(alice, mkey(1), vec![]);
        let pki = ResolverPki::new(r);
        let got = ResolverPki::identity_key(&pki, &alice).expect("no resolver error");
        assert!(got.is_some());
    }

    #[test]
    fn unknown_member_resolves_to_none_not_error() {
        let pki = ResolverPki::new(StubResolver::new());
        let got = ResolverPki::identity_key(&pki, &MemberId([0xff; 32])).expect("graceful");
        assert!(got.is_none());
    }

    #[test]
    fn org_key_resolves() {
        let r = StubResolver::new().with_org_key(OrgKey(mkey(9).0));
        let pki = ResolverPki::new(r);
        assert!(pki.org_identity_key().expect("graceful").is_some());
    }
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --features org-acl-core/testing --lib pki`
Expected: FAIL — `ResolverPki` not defined.

- [ ] **Step 4: Write ResolverPki (prepend above the test module)**

```rust
//! `ResolverPki<R>` — the trie-lookup key-resolution half of item-1.
//!
//! Wraps a [`MemberKeyResolver`] and implements
//! [`IdentityRegistry<MemberId, ResolverPki<R>>`] (the upstream trait's
//! `identity_key` is a static fn taking the state `Y`; we pass `&self` as `Y`,
//! so the resolver is consulted on every call — no cache, Flow-B holds).
//!
//! Key-type note (carried from the spike): `identity_key` returns an x25519
//! `PublicKey`; `P2pMemberKey` wraps an ed25519 `VerifyingKey` whose compressed
//! form is also 32 bytes. We reinterpret those bytes via `PublicKey::from_bytes`.
//! The identity key here is a STABLE ANCHOR for prekey-bundle lookup; actual
//! ECDH uses separate prekey bundles, so the reinterpretation is safe for the
//! PKI-lookup role.

use org_acl_core::{MemberId, MemberKeyResolver, ResolverError};
use p2panda_encryption::crypto::x25519::PublicKey as X25519PublicKey;
use p2panda_encryption::traits::IdentityRegistry;

pub struct ResolverPki<R> {
    /// The live resolver — consulted on every key-lookup call.
    pub resolver: R,
}

impl<R: MemberKeyResolver> ResolverPki<R> {
    pub fn new(resolver: R) -> Self {
        Self { resolver }
    }

    /// Instance-level identity-key lookup. `Ok(None)` for unknown members
    /// (graceful); `Err` only for non-`UnknownMember` resolver failures.
    pub fn identity_key_with_resolver(
        &self,
        id: &MemberId,
    ) -> Result<Option<X25519PublicKey>, ResolverError> {
        match self.resolver.p2p_member_key(id) {
            Ok(key) => Ok(Some(X25519PublicKey::from_bytes(*key.0.as_bytes()))),
            Err(ResolverError::UnknownMember(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Organisation identity key. `Ok(None)` if the org key is unset.
    pub fn org_identity_key(&self) -> Result<Option<X25519PublicKey>, ResolverError> {
        match self.resolver.org_key() {
            Ok(key) => Ok(Some(X25519PublicKey::from_bytes(*key.0.as_bytes()))),
            Err(ResolverError::OrgKeyUnset) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// `Y = ResolverPki<R>`: the caller passes `&pki` as the static-method state.
impl<R: MemberKeyResolver> IdentityRegistry<MemberId, ResolverPki<R>> for ResolverPki<R> {
    type Error = ResolverError;

    fn identity_key(
        y: &ResolverPki<R>,
        id: &MemberId,
    ) -> Result<Option<X25519PublicKey>, Self::Error> {
        y.identity_key_with_resolver(id)
    }
}
```

- [ ] **Step 5: Run to verify it passes**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --features org-acl-core/testing --lib pki`
Expected: PASS (3 tests). If `X25519PublicKey::from_bytes` signature differs at the pin, reconcile against the spike's `s1_stable_id_acl.rs` (same call).

- [ ] **Step 6: Commit**

```bash
git add org-acl-p2panda/src/pki.rs org-acl-p2panda/src/lib.rs
git commit -m "feat(org-acl-p2panda): ResolverPki — IdentityRegistry over the trie resolver"
```

---

## Task 6: `org-acl-p2panda` materialise_actor_id (Flow-B witness)

**Files:**
- Create: `org-acl-p2panda/src/actor.rs`
- Modify: `org-acl-p2panda/src/lib.rs`

Productionises `spike-p2panda::s1_stable_id_acl::materialise_actor_id`.

- [ ] **Step 1: Declare the module**

Append to `org-acl-p2panda/src/lib.rs`:

```rust
pub mod actor;
pub use actor::materialise_actor_id;
```

- [ ] **Step 2: Write the failing tests**

Create `org-acl-p2panda/src/actor.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use org_acl_core::testing::StubResolver;
    use org_acl_core::{MemberId, OrgKey, P2pMemberKey, Principal};

    fn mkey(seed: u8) -> P2pMemberKey {
        P2pMemberKey(SigningKey::from_bytes(&[seed; 32]).verifying_key())
    }

    #[test]
    fn materialises_member_and_org() {
        let alice = MemberId([1; 32]);
        let r = StubResolver::new().add_member(alice, mkey(1), vec![]).with_org_key(OrgKey(mkey(9).0));
        assert!(materialise_actor_id(&r, &Principal::Member(alice)).is_ok());
        assert!(materialise_actor_id(&r, &Principal::Org).is_ok());
    }

    #[test]
    fn unknown_member_is_error() {
        let r = StubResolver::new();
        assert!(materialise_actor_id(&r, &Principal::Member(MemberId([0xff; 32]))).is_err());
    }

    #[test]
    fn rotation_is_picked_up_no_cache() {
        // Flow-B: re-resolving after a key rotation yields a DIFFERENT ActorId,
        // proving the resolver (not a cache) is authoritative.
        let alice = MemberId([1; 32]);
        let r = StubResolver::new().add_member(alice, mkey(1), vec![]);
        let before = materialise_actor_id(&r, &Principal::Member(alice)).unwrap();
        let r = r.rotate_member_key(&alice, mkey(0xa9));
        let after = materialise_actor_id(&r, &Principal::Member(alice)).unwrap();
        assert_ne!(before, after, "rotation must change the materialised ActorId");
    }
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --features org-acl-core/testing --lib actor`
Expected: FAIL — `materialise_actor_id` not defined.

- [ ] **Step 4: Write the function (prepend above the test module)**

```rust
//! `materialise_actor_id` — the `p2panda-spaces` seam.
//!
//! Resolves a [`Principal`] to a fresh [`ActorId`] at call time. Because the
//! resolver is consulted on EVERY call, the returned `ActorId` always reflects
//! the current trie key: after a rotation the next call returns the new
//! `ActorId` (Flow-B; no cache). Callers managing `Group`/`Space` membership
//! reconcile a rotation by `remove(old)` then `add(new)`.

use org_acl_core::{MemberKeyResolver, Principal, ResolverError};
use p2panda_core::identity::VerifyingKey as PandaVerifyingKey;
use p2panda_spaces::ActorId;

pub fn materialise_actor_id<R: MemberKeyResolver>(
    resolver: &R,
    principal: &Principal,
) -> Result<ActorId, ResolverError> {
    match principal {
        Principal::Member(id) => {
            let key = resolver.p2p_member_key(id)?;
            Ok(ActorId::from(PandaVerifyingKey::from(key.0)))
        }
        Principal::Org => {
            let key = resolver.org_key()?;
            Ok(ActorId::from(PandaVerifyingKey::from(key.0)))
        }
    }
}
```

- [ ] **Step 5: Run to verify it passes**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --features org-acl-core/testing --lib actor`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add org-acl-p2panda/src/actor.rs org-acl-p2panda/src/lib.rs
git commit -m "feat(org-acl-p2panda): materialise_actor_id — re-resolving spaces seam (Flow-B)"
```

---

## Task 7: `org-acl-p2panda` ACL grant path (stable-id ACL, below spaces)

**Files:**
- Create: `org-acl-p2panda/src/acl.rs`
- Modify: `org-acl-p2panda/src/lib.rs`

The stable-id ACL: `GroupCrdt<AuthMemberId>` driven directly (below `p2panda-spaces`' eager `Manager`), so a grant carries only the stable identity — no CGKA/prekey at add-time.

- [ ] **Step 1: Declare the module**

Append to `org-acl-p2panda/src/lib.rs`:

```rust
pub mod acl;
pub use acl::{AclOp, OpId, OrgAcl};
```

- [ ] **Step 2: Write the failing tests**

Create `org-acl-p2panda/src/acl.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use org_acl_core::MemberId;

    const ALICE: MemberId = MemberId([0xa1; 32]);
    const BOB: MemberId = MemberId([0xb1; 32]);
    const GROUP: MemberId = MemberId([0xc1; 32]);

    #[test]
    fn grant_stores_stable_ids_not_keys() {
        // Alice creates a group; adds Bob. Membership is by stable MemberId.
        let acl = OrgAcl::create(GROUP, ALICE).expect("create");
        let acl = acl.add_member(BOB).expect("add bob");
        let mut ids = acl.member_ids();
        ids.sort();
        let mut expected = vec![ALICE, BOB];
        expected.sort();
        assert_eq!(ids, expected);
    }

    #[test]
    fn revoked_member_drops_from_acl() {
        let acl = OrgAcl::create(GROUP, ALICE).expect("create");
        let acl = acl.add_member(BOB).expect("add bob");
        let acl = acl.remove_member(BOB).expect("remove bob");
        assert_eq!(acl.member_ids(), vec![ALICE]);
    }

    #[test]
    fn acl_op_embeds_stable_id_bytes_not_ed25519_key() {
        // CBOR of an Add op must contain Bob's 0xb1 stable-id bytes (and no key).
        let op = AclOp {
            id: OpId(1),
            author: AuthMemberId::from(ALICE),
            dependencies: vec![OpId(0)],
            group_id: AuthMemberId::from(GROUP),
            action: p2panda_auth::group::GroupAction::Add {
                member: p2panda_auth::group::GroupMember::Individual(AuthMemberId::from(BOB)),
                access: p2panda_auth::Access::write(),
            },
        };
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(&op, &mut bytes).expect("cbor");
        assert!(bytes.contains(&0xb1), "Add op CBOR must embed Bob's stable-id byte");
    }
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --lib acl`
Expected: FAIL — `OrgAcl`/`AclOp`/`OpId` not defined.

- [ ] **Step 4: Write the ACL types (prepend above the test module)**

```rust
//! The stable-id ACL — `GroupCrdt<AuthMemberId>` driven directly.
//!
//! This is the lazy-CGKA ACL tier: grants carry only the stable trie identity
//! (`AuthMemberId`), BELOW `p2panda-spaces`' eager `Manager`, so adding a member
//! does NOT force prekey/DCGKA placement. A minimal `AclOp` (no networking,
//! signing, or async) drives `GroupCrdt::process`, matching the spike's gate-1
//! reference (`l1_p2panda_auth` Test 2).

use serde::{Deserialize, Serialize};

use p2panda_auth::group::resolver::StrongRemove;
use p2panda_auth::group::{GroupAction, GroupCrdt, GroupCrdtState, GroupMember};
use p2panda_auth::traits::{Operation, OperationId};
use p2panda_auth::Access;

use org_acl_core::MemberId;

use crate::auth_id::AuthMemberId;

/// ACL operation id (monotonic; supplied by the caller / op log).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OpId(pub u32);
impl OperationId for OpId {}

/// A minimal stable-id ACL operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AclOp {
    pub id: OpId,
    pub author: AuthMemberId,
    pub dependencies: Vec<OpId>,
    pub group_id: AuthMemberId,
    pub action: GroupAction<AuthMemberId, ()>,
}

impl Operation<AuthMemberId, OpId, ()> for AclOp {
    fn id(&self) -> OpId { self.id }
    fn author(&self) -> AuthMemberId { self.author }
    fn dependencies(&self) -> Vec<OpId> { self.dependencies.clone() }
    fn group_id(&self) -> AuthMemberId { self.group_id }
    fn action(&self) -> GroupAction<AuthMemberId, ()> { self.action.clone() }
}

type AclState = GroupCrdtState<AuthMemberId, OpId, AclOp, ()>;
type AclCrdt = GroupCrdt<AuthMemberId, OpId, AclOp, (), StrongRemove<AuthMemberId, OpId, AclOp, ()>>;

/// A stable-id ACL bound to one group id, carrying the CRDT state and the
/// op-sequence cursor. The manager (group creator) authors subsequent ops.
pub struct OrgAcl {
    group: AuthMemberId,
    manager: AuthMemberId,
    state: AclState,
    next_op: u32,
    last: Vec<OpId>,
}

/// ACL construction / processing failed at the CRDT layer.
#[derive(Debug, thiserror::Error)]
#[error("acl crdt rejected the operation")]
pub struct AclError;

impl OrgAcl {
    /// Create a group with `manager` (a stable `MemberId`) as the sole manager.
    pub fn create(group: MemberId, manager: MemberId) -> Result<Self, AclError> {
        let group = AuthMemberId::from(group);
        let manager = AuthMemberId::from(manager);
        let op = AclOp {
            id: OpId(0),
            author: manager,
            dependencies: vec![],
            group_id: group,
            action: GroupAction::Create {
                initial_members: vec![(GroupMember::Individual(manager), Access::manage())],
            },
        };
        let state = AclCrdt::process(AclCrdt::init(), &op).map_err(|_| AclError)?;
        Ok(Self { group, manager, state, next_op: 1, last: vec![OpId(0)] })
    }

    fn apply(mut self, action: GroupAction<AuthMemberId, ()>) -> Result<Self, AclError> {
        let id = OpId(self.next_op);
        let op = AclOp {
            id,
            author: self.manager,
            dependencies: self.last.clone(),
            group_id: self.group,
            action,
        };
        self.state = AclCrdt::process(self.state, &op).map_err(|_| AclError)?;
        self.next_op += 1;
        self.last = vec![id];
        Ok(self)
    }

    /// Delegate Write access to a member (handle@org) by stable id. No key,
    /// no CGKA placement — the lazy ACL tier.
    pub fn add_member(self, member: MemberId) -> Result<Self, AclError> {
        self.apply(GroupAction::Add {
            member: GroupMember::Individual(AuthMemberId::from(member)),
            access: Access::write(),
        })
    }

    /// Remove a member's ACL delegation by stable id (the ACL-tier of revocation).
    pub fn remove_member(self, member: MemberId) -> Result<Self, AclError> {
        self.apply(GroupAction::Remove {
            member: GroupMember::Individual(AuthMemberId::from(member)),
        })
    }

    /// Current transitive members, as bare stable `MemberId`s.
    pub fn member_ids(&self) -> Vec<MemberId> {
        self.state.members(self.group).into_iter().map(|(a, _)| a.0).collect()
    }
}
```

Note on the p2panda-auth API at the pin: the spike exercised `GroupAction::{Create, Add}`, `GroupMember::{Individual, Group}`, and `Access::{manage, read}` (`spike-p2panda/tests/l1_p2panda_auth.rs`). This task additionally uses **`GroupAction::Remove`** and **`Access::write()`** — confirm both exist with these shapes at rev `41559b0` before writing the impl. If `Remove`'s field name differs or `write()` is spelled differently, reconcile here (the Task 8 L3 test also depends on `remove_member`). The spike's gate finding "Group members must use Read or Write, not Manage" implies `Access::write()` is present.

- [ ] **Step 5: Run to verify it passes**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --lib acl`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add org-acl-p2panda/src/acl.rs org-acl-p2panda/src/lib.rs
git commit -m "feat(org-acl-p2panda): stable-id ACL grant path (GroupCrdt, below spaces — lazy CGKA)"
```

---

## Task 8: L3 acceptance — revocation passes + identity-takeover blocked

**Files:**
- Create: `org-acl-p2panda/tests/l3_revocation.rs`

The ODS Roadmap.3 item-1 exit criteria (spec §6), promoted from `spike-p2panda/tests/l3_revocation.rs` (item-1 slice — the ACL/resolver layer; `Dcgka::remove` is item-4).

- [ ] **Step 1: Write the tests**

`org-acl-p2panda/tests/l3_revocation.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Exit criteria for Phase 3 item-1 (spec §6):
//!  - revocation: a removed member resolves to nothing AND drops from the ACL;
//!    no ActorId/key is obtainable for them.
//!  - identity-takeover blocked: a forged/rotated key the trie does not vouch
//!    for is never authoritative (no cache; resolver is the only source).
//!
//! These witness the Phase 1.3 formal model's transitive-trust acceptance rules
//! and the members-trie <-> substrate sync contract. When the Quint model names
//! land on master, cite the exact invariant names here.
//!
//! Scope: the cryptographic forward-security half (`Dcgka::remove`) is item-4.

use ed25519_dalek::SigningKey;

use org_acl_core::testing::StubResolver;
use org_acl_core::{MemberId, MemberKeyResolver, P2pMemberKey, Principal, ResolverError};
use org_acl_p2panda::{materialise_actor_id, OrgAcl};

fn mkey(seed: u8) -> P2pMemberKey {
    P2pMemberKey(SigningKey::from_bytes(&[seed; 32]).verifying_key())
}

const ALICE: MemberId = MemberId([0xa1; 32]);
const BOB: MemberId = MemberId([0xb1; 32]);
const GROUP: MemberId = MemberId([0xc1; 32]);

#[test]
fn revocation_makes_member_unresolvable_and_drops_from_acl() {
    // Bob is a member with an ACL delegation.
    let resolver = StubResolver::new()
        .add_member(ALICE, mkey(0xa1), vec![])
        .add_member(BOB, mkey(0xb1), vec![]);
    let acl = OrgAcl::create(GROUP, ALICE).unwrap().add_member(BOB).unwrap();
    assert!(acl.member_ids().contains(&BOB));
    assert!(materialise_actor_id(&resolver, &Principal::Member(BOB)).is_ok());

    // Trie revokes Bob; the ACL manager removes his delegation.
    let resolver = resolver.revoke(&BOB);
    let acl = acl.remove_member(BOB).unwrap();

    // No key/ActorId is obtainable for the revoked member.
    assert_eq!(resolver.p2p_member_key(&BOB), Err(ResolverError::UnknownMember(BOB)));
    assert!(matches!(
        materialise_actor_id(&resolver, &Principal::Member(BOB)),
        Err(ResolverError::UnknownMember(_))
    ));
    // And he is gone from the ACL.
    assert!(!acl.member_ids().contains(&BOB));
    assert_eq!(acl.member_ids(), vec![ALICE]);
}

#[test]
fn forged_key_is_never_authoritative() {
    // The authoritative trie vouches for Bob's real key.
    let real_resolver = StubResolver::new().add_member(BOB, mkey(0xb1), vec![]);
    let real_actor = materialise_actor_id(&real_resolver, &Principal::Member(BOB)).unwrap();

    // An attacker controls a different key for the same MemberId, but only the
    // real trie is authoritative. Materialising through each resolver shows the
    // forged key yields a DIFFERENT ActorId — it can never stand in for Bob.
    let forged_resolver = StubResolver::new().add_member(BOB, mkey(0xff), vec![]);
    let forged_actor = materialise_actor_id(&forged_resolver, &Principal::Member(BOB)).unwrap();
    assert_ne!(real_actor, forged_actor, "only the trie-vouched key is authoritative");

    // After the real trie revokes Bob, his key no longer resolves at all.
    let revoked = real_resolver.revoke(&BOB);
    assert!(materialise_actor_id(&revoked, &Principal::Member(BOB)).is_err());
}
```

This integration test uses only `ed25519-dalek` (for `mkey`) plus the public API of `org-acl-core` and `org-acl-p2panda` — all available to `tests/` (`ed25519-dalek` is an `org-acl-p2panda` dev-dependency; `StubResolver` comes via the `testing` feature the dev-dependency enables).

- [ ] **Step 2: Run it**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --features org-acl-core/testing --test l3_revocation`
Expected: PASS (2 tests).

- [ ] **Step 3: Commit**

```bash
git add org-acl-p2panda/tests/l3_revocation.rs
git commit -m "test(org-acl-p2panda): L3 acceptance — revocation + identity-takeover blocked"
```

---

## Task 9: `org-acl-p2panda` fuzz — resolver→key boundary

**Files:**
- Create: `org-acl-p2panda/tests/fuzz_resolver_key_boundary/fuzz_target.rs`
- Create: `org-acl-p2panda/tests/fuzz_resolver_key_boundary/corpus/.gitkeep`, `crashes/.gitkeep`

The untrusted boundary for item-1: arbitrary 32-byte key material flowing through the ed25519→x25519 reinterpret (`ResolverPki`) and the `ActorId` construction (`materialise_actor_id`). Both must be total (never panic).

- [ ] **Step 1: Write the fuzz target**

`org-acl-p2panda/tests/fuzz_resolver_key_boundary/fuzz_target.rs`:

```rust
//! Fuzz: resolving a member/org key to an x25519 PublicKey and to an ActorId
//! must never panic for any key bytes. `harness = false`.
//! Deep-fuzz: `cargo bolero test fuzz_resolver_key_boundary --engine libfuzzer`.

use bolero::check;
use ed25519_dalek::VerifyingKey;

use org_acl_core::testing::StubResolver;
use org_acl_core::{MemberId, OrgKey, P2pMemberKey, Principal};
use org_acl_p2panda::{materialise_actor_id, ResolverPki};

fn main() {
    check!().with_type::<[u8; 32]>().cloned().for_each(|raw| {
        // Only valid ed25519 points form a VerifyingKey; invalid bytes are
        // rejected by the constructor (no panic) and skipped.
        let Ok(vk) = VerifyingKey::from_bytes(&raw) else { return };
        let bob = MemberId([1; 32]);

        // (1) materialise_actor_id — ed25519 -> ActorId construction is total.
        let r = StubResolver::new().add_member(bob, P2pMemberKey(vk), vec![]).with_org_key(OrgKey(vk));
        let _ = materialise_actor_id(&r, &Principal::Member(bob));
        let _ = materialise_actor_id(&r, &Principal::Org);

        // (2) ResolverPki — ed25519 -> x25519 reinterpret is total. Use the
        // inherent method so the IdentityRegistry trait need not be imported.
        let pki = ResolverPki::new(r);
        let _ = pki.identity_key_with_resolver(&bob);
        let _ = pki.org_identity_key();
    });
}
```

- [ ] **Step 2: Create corpus/crashes dirs**

```bash
mkdir -p org-acl-p2panda/tests/fuzz_resolver_key_boundary/corpus org-acl-p2panda/tests/fuzz_resolver_key_boundary/crashes
touch org-acl-p2panda/tests/fuzz_resolver_key_boundary/corpus/.gitkeep org-acl-p2panda/tests/fuzz_resolver_key_boundary/crashes/.gitkeep
```

- [ ] **Step 3: Run it**

Run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda --features org-acl-core/testing --test fuzz_resolver_key_boundary`
Expected: PASS (no panic on the generated batch).

- [ ] **Step 4: Commit**

```bash
git add org-acl-p2panda/tests/fuzz_resolver_key_boundary/
git commit -m "test(org-acl-p2panda): bolero fuzz — resolver->key boundary never-panics"
```

---

## Task 10: READMEs + full suite + clippy gate

**Files:**
- Create: `org-acl-core/README.md`, `org-acl-p2panda/README.md`

- [ ] **Step 1: Write `org-acl-core/README.md`**

```markdown
# org-acl-core

ODS organisation ACL — the neutral, substrate-agnostic contract. Identity types
(`MemberId` = handle@org, `Principal`, key newtypes) + the minimal
`MemberKeyResolver` trait. NO local-first-substrate dependencies.

This is the convergence anchor: every ACL backend (`org-acl-p2panda`; a Keyhive
`org-acl` later) shares it, and `org-node` implements `MemberKeyResolver` over
its trie mirror exactly once. Backends extend the contract by wrapping or
sub-trait-ing — never by adding substrate-specific methods here.

## Test helper
`StubResolver` (behind `--features testing`) is an in-memory resolver for tests
in this crate, `org-acl-p2panda`, and `org-node`.

## Fuzzing
`fuzz_identity_codec` (bolero) — `Principal`/`MemberId` postcard round-trip +
never-panic. Deep lane: `cargo bolero test fuzz_identity_codec --engine libfuzzer`.
```

- [ ] **Step 2: Write `org-acl-p2panda/README.md`**

```markdown
# org-acl-p2panda

ODS organisation ACL — the p2panda substitution layer (Phase 3 item-1, parallel
track). The **only** crate that names `p2panda-*` (the GPL-3.0 boundary).
Pinned at p2panda rev `41559b0`; no fork required for item-1.

## Core idea
The ACL binds to the stable `MemberId` (handle@org) via
`GroupCrdt<AuthMemberId>`, driven BELOW `p2panda-spaces`' eager `Manager` so the
CGKA stays lazy (no prekey/DCGKA at grant-time). Keys resolve live through the
trie: `ResolverPki` (`IdentityRegistry`) and `materialise_actor_id` re-query the
resolver on every call — no cache — so a rotated key is picked up and a forged
key the trie does not vouch for resolves to nothing (Flow-B). `Dcgka::remove`
and lazy-CGKA recompute are item-4.

## Fuzzing
`fuzz_resolver_key_boundary` (bolero) — the ed25519→x25519 reinterpret + ActorId
construction never panic for arbitrary key bytes. Deep lane:
`cargo bolero test fuzz_resolver_key_boundary --engine libfuzzer`.
```

- [ ] **Step 3: Run the full suites**

```bash
CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-core --features testing
CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-acl-p2panda```
Expected: `org-acl-core` — identity (3) + test_support (2) + fuzz_identity_codec.
`org-acl-p2panda` — lib: g1_probe (2) + auth_id (2) + pki (3) + actor (3) + acl (3); integration: l3_revocation (2) + fuzz_resolver_key_boundary.

- [ ] **Step 4: Clippy deny-gate (lib only) + default build**

```bash
CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-acl-core --lib -- -D warnings
CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-acl-p2panda --lib -- -D warnings
CARGO_HOME=/tmp/cargo_home_fuzz cargo build -p org-acl-core -p org-acl-p2panda
```
Expected: clean. The `--lib` gate enforces no `unwrap`/`expect`/`panic` in library code; test/fuzz code is exempt. (All library functions return `Result`/`Option`; there are no locks or unwraps to trip the gate.)

- [ ] **Step 5: Commit**

```bash
git add org-acl-core/README.md org-acl-p2panda/README.md
git commit -m "docs(org-acl): READMEs — item-1 status, core idea, fuzzing"
```

---

## Final review

- [ ] Dispatch a code reviewer over the whole item-1 diff (spec §3–§7 compliance: minimal neutral core trait; Flow-B enforced *structurally* by the no-cache re-resolve and asserted by `actor::rotation_is_picked_up_no_cache` + `l3_revocation`; ACL stores `AuthMemberId` not keys; lazy seam = grant carries no key; both fuzz targets wired; clippy `--lib` clean).
- [ ] Confirm `org-node` is untouched (the resolver impl over its trie mirror is a follow-on — it implements `org_acl_core::MemberKeyResolver`, the shared anchor; not part of item-1's two crates).
- [ ] Then use **superpowers:finishing-a-development-branch**. Squash-merge `worktree-phase-3-item1-p2panda` to `master` as a single **user-signed** commit (AGENTS.md): gpg signing is disabled in the worktree, so squash and have the user sign the merge (`git commit --amend -S` from a regular terminal if the agent's merge lands unsigned).

## Notes on convergence (why this mirrors the Keyhive plan)

This plan is the p2panda realisation of the same item-1; per the spec §4.2 SAME/ADAPT/DIVERGE map: Tasks 1–3 (core) are the ADAPTed `org-acl` identity/resolver/fuzz minus the composite and `ContactCard`; Tasks 4–9 are the DIVERGEd p2panda wiring (no `IdAdapter` — re-resolve gives Flow-B; no `ContactCard` — `GroupCrdt` is ID-generic); Task 8 L3 and Task 10 conventions are SAME shape. The single neutral `MemberKeyResolver` (Task 2) is the convergence point both backends and `org-node` share.
