# ODS Phase 3 item-1 on p2panda — stable-ID ACL + trie-lookup (parallel track)

**Author(s):** Jan-Jan van der Vyver (design captured via brainstorming session)
**Status:** In review
**Created:** 2026-06-18
**Spec for:** Phase 3 of [`Organisational Data Sovereignty p1.md`](../../../Organisational%20Data%20Sovereignty%20p1.md) §Roadmap item 3, sub-item 1 — "Stable identities + trie-lookup key resolution: revocation scenario passes, identity-takeover attack provably blocked."
**Substrate:** p2panda, pinned at the Phase 1.d spike rev `41559b0dfc2d7d0e9e4fba251ceb7f8094ff8be1`.
**Relationship to the Keyhive design:** this is a **parallel track** alongside the (unbuilt) Keyhive item-1 design [`2026-06-15-ods-phase-3-design.md`](2026-06-15-ods-phase-3-design.md) §5 / plan [`2026-06-15-ods-phase-3-item1-stable-id-acl.md`](../plans/2026-06-15-ods-phase-3-item1-stable-id-acl.md). The Keyhive pick (`docs/phase-1d/decision.md`) was conditional; this realises the documented p2panda salvage path (decision.md §5). The final substrate pick is **deferred** — both tracks may coexist until then.

---

## 1. Overview

Phase 3 item-1 makes the **stable trie identity** (not a raw public key) the ACL
identity, with the **rotatable key** resolved through the trie on demand. This spec
builds that on **p2panda** as a real, mergeable second implementation, structured so
the two tracks (p2panda here, Keyhive later) **converge cleanly** at a shared neutral
crate.

It delivers exactly the two ODS Roadmap.3 item-1 exit criteria:

1. **Revocation passes** (at the ACL/resolver layer — see scope boundary in §6).
2. **Identity-takeover provably blocked.**

Items 2–5 (org pseudo-group, write-authority lockout, CGKA triggers, p2p policy) are
**out of scope** and untouched.

## 1b. Revision R2 (2026-06-18) — `OrgMember` identity + root-anchored per-doc index

This revision **supersedes** the "simple single-org" identity decision and the bare
`org_epoch` stamp wherever they conflict below.

- **ACL identity is the composite `OrgMember { org: OrgId, member: MemberId }`** (because
  `MemberId` is unique only within one org's trie). `Principal::Member(OrgMember)` /
  `Principal::Org(OrgId)`. `AuthMemberId` wraps `OrgMember`. This makes the p2panda identity
  model identical to the Keyhive track (convergence).
- **Per-document reconciliation index — a shared neutral type in `org-acl-core`:**
  ```rust
  pub struct RootHash(pub [u8; 32]);          // mirrors org_members::RootHash; org-node converts
  pub struct DocReconcileMark {
      pub org: OrgId,
      pub acl_epoch: Epoch,   // trie epoch the ACL tier was last reconciled to (was `org_epoch`)
      pub cgka_epoch: Epoch,  // trie epoch the CGKA tier was last reconciled to — item-4 drives it
      pub root_hash: RootHash,// the on-chain-anchored trie root the ACL was reconciled against
  }
  ```
  Hoisting it into core (not duplicating per backend) makes it the convergence artifact both
  tracks share. Rationale: the two tiers reconcile at different cadences (so two epochs), and
  an epoch is a reorg-able *derived* index whereas `root_hash` is the cryptographic anchor
  (reorg-safe — a changed canonical root at that height ⇒ stored hash mismatch ⇒ re-reconcile).
- **`MemberKeyResolver` gains `org_id() -> OrgId` and `root_hash() -> RootHash`** (trie-level
  queries both backends need; forward key methods still take a bare `MemberId`, org implicit).
- **`MembershipDelta` gains `from_root`/`to_root: RootHash`.** `org-node` derives these from its
  `SignedDeltaEnvelope` (`base_root` + new root) in item-4; item-1 builds them synthetically.
- **`DocAccess` embeds `DocReconcileMark`.** `create(doc_id, manager, resolver)` stamps the mark
  from `resolver.org_id()/epoch()/root_hash()` (cgka_epoch initialised = acl_epoch). `reconcile`'s
  Δ==1 delta path now requires `is_single_step && from_epoch==acl_epoch && from_root==root_hash &&
  to_epoch==target_epoch && to_root==target_root`, and stamps both `acl_epoch` and `root_hash`;
  the full-rebuild path stamps `acl_epoch=resolver.epoch()` + `root_hash=resolver.root_hash()`.
  `cgka_epoch` is untouched in item-1.

Scope unchanged otherwise: item-1 is still the ACL tier only; the CGKA tier (and thus
`cgka_epoch` updates), the auto-observer, and the real `org-node` delta bridge remain item-4.

## 2. The gate (preconditions for execution)

- **(a) Phase 2 `org-node` has landed.** ✅ Satisfied — `org-node` is on `master`
  (commit `2bb1c21f5`) with the committed per-org trie mirror (`OrgTrie` over
  `org-members`) the resolver binds to.
- **(b) Substrate maturity.** Because this is a *parallel track*, not the committed
  substrate, the maturity gate is **relaxed to a risk note** rather than a blocker:
  `p2panda-encryption` has an audit scheduled with Radically Open Security and
  `p2panda-spaces` is on a feature branch. We **ride the spike pin `41559b0`** to stay
  aligned with the Phase 1.d qualification evidence; re-pinning to a later/tagged
  release is follow-on work. **No p2panda fork is required for item-1** — item-1's
  salvage is the `TraitImpl` path (~40 lines), not the `pub use` fork (that is only
  items 2/4; decision.md §2).

## 3. Architecture

### 3.1 Crate topology

```
org-acl-core/            # Apache-2.0; NO substrate deps; WASM-clean. The convergence anchor.
  src/
    identity.rs    # MemberId, P2pMemberKey, P2pDeviceKey, OrgKey, Epoch, Principal
    resolver.rs    # MemberKeyResolver trait (minimal) + ResolverError
    delta.rs       # MembershipDelta — neutral per-epoch change set (from/to epoch, removed/added/rotated)
    test_support.rs# StubResolver (cfg(test)/feature `testing`) — graduated from spike-common
  tests/
    fuzz_identity_codec/ # bolero: Principal/MemberId/MembershipDelta postcard never-panic + round-trip

org-acl-p2panda/         # GPL-3.0-only; the ONLY p2panda boundary crate.
  src/
    lib.rs         # crate docs: Flow-B invariant + lazy-CGKA two-tier spine + epoch reconciliation
    auth_id.rs     # AuthMemberId(MemberId) + impl IdentityHandle (orphan-rule newtype)
    pki.rs         # ResolverPki<R>: impl IdentityRegistry<MemberId, Self>  (key-resolution half)
    actor.rs       # materialise_actor_id(resolver, principal) -> ActorId  (re-resolves; no cache)
    acl.rs         # DocAccess: per-document GroupCrdt<AuthMemberId> grant path + org_epoch stamp + reconcile
  tests/
    l3_revocation.rs           # exit criteria (promoted from spike-p2panda l3_revocation, item-1 slice)
    fuzz_resolver_key_boundary/# bolero: materialise/ResolverPki on arbitrary key bytes never-panic
```

- `org-node` (Phase 2) depends on **`org-acl-core`** and implements
  `MemberKeyResolver` over its committed trie mirror — **once**, neutrally.
- `org-acl-p2panda` depends on `org-acl-core` + the p2panda crates.
- `spike-common` / `spike-p2panda` stay **frozen** as Phase 1.d evidence; their
  contract and gate-1 wiring are **graduated** (re-homed, not re-derived).

### 3.2 Cross-cutting spine — lazy-CGKA two-tier model

| Tier | Source of truth | Cadence | Carries |
|------|-----------------|---------|---------|
| **ACL** (delegation) | Trie (on-chain anchored) | Rare — onboard / rotate / revoke | stable **handle@org** identity → member-as-a-group key (`P2pMemberKey`) |
| **CGKA** (encryption) | Per-document `p2panda-encryption` DCGKA | **Lazy** — computed when a member's client first comes online | x25519 prekey material (online devices only) |

**Delegation is at the `Principal::Member` = handle@org member level** — never a raw
key, never a device. The ACL grants to the member-as-a-group key resolved through the
trie; device keys belong to the *separate* CGKA tier. The "@org" context is supplied by
the **org-bound resolver** (one instance per org record — matches Phase 2's
one-persona-per-org). `MemberId` is the PII-free stable representation of the handle@org
leaf; the human handle↔`MemberId` mapping stays in `org-members`/`org-node`, outside the
crypto boundary. The org-as-pseudo-group (`Principal::Org`) delegation is item-2.

**CGKA stays lazy → the ACL grant runs _below_ `p2panda-spaces`' eager `Manager`.**
The grant is driven on `p2panda-auth::GroupCrdt<AuthMemberId>` directly (stable-id-native;
proven by the spike's gate-1 L1 test 2), so **adding a member to the ACL does not force
prekey/DCGKA placement at add-time**. `ResolverPki` (the `p2panda-encryption` PKI bridge)
and `materialise_actor_id` (the spaces `ActorId` seam) live in item-1 but feed the CGKA
tier, which is computed lazily on first-online. The actual lazy DCGKA compute/trigger is
**item-4**; item-1 delivers the ACL tier + resolver and keeps the two tiers **decoupled**.

### 3.3 Per-document ACL + epoch-stamped lazy reconciliation

ACLs and CGKAs are **per data object (per document)**, so item-1 introduces a per-document
type **`DocAccess`** — one instance per document, holding **both** that document's ACL
(its `GroupCrdt<AuthMemberId>` writer set, built now) and its CGKA (added in item-4),
under a single `org_epoch` stamp.

To avoid **thrashing every document** when the members trie changes, reconciliation is
**lazy and epoch-stamped** (the ODS SKU-window minimization, §Scalability):

- Each `DocAccess` stores an **`org_epoch`** = the members-trie epoch its ACL (and, in
  item-4, its CGKA) was last reconciled against (`MemberKeyResolver::epoch()`).
- On a trie advance, a document is reconciled **on demand** — open/active documents first
  (the scheduling/ordering itself is Phase-5).
- **Δepoch == 1 (contiguous):** apply a **`MembershipDelta`** to patch the ACL in place —
  prune `removed` members present in this doc's ACL, then stamp `to_epoch`. (Member
  *additions* are explicit grants, not org-wide auto-adds; key / org-key *rotations* are
  CGKA-tier, item-4.)
- **Δepoch > 1, non-contiguous, or no delta available:** **full rebuild** — drop any
  current ACL member the resolver no longer vouches for (`is_member`), stamp
  `resolver.epoch()`.

The "merkle delta when Δ==1" is `org-node`'s existing single-step **`SignedDeltaEnvelope`**
(guarded by `SeqGuard`/`parent_seq`); "Δ>1 → full trie" is the rebuild-from-committed-trie
path. To keep the dependency arrow correct (`org-node → org-acl-core`, never the reverse),
item-1's reconciler consumes a **neutral `MembershipDelta`** (defined in `org-acl-core`),
**not** `org-node`'s concrete envelope. `org-node` mapping `SignedDeltaEnvelope →
MembershipDelta` is the item-4 bridge.

**Item-1 delivers:** `DocAccess` with the `org_epoch` stamp and the full `reconcile`
function (both the Δ==1 delta path and the full-rebuild path), tested against synthetic
`MembershipDelta`s and a `StubResolver`. **Deferred (item-4 / Phase-5):** the trie-change
*observer* that auto-fires `reconcile` on the finalised epoch-bump signal; the
open-documents-first scheduling; and the **CGKA-tier** reconcile (key rotations →
`Dcgka` epoch advance / `Dcgka::remove`).

## 4. Convergence with the Keyhive track (the design's central concern)

The convergence point is **`org-acl-core`**: whatever lives there is shared by both
backends. The goal is to maximise what is identical in core, isolate every substrate
difference into the backend crate, and keep task structure parallel.

### 4.1 The key lever — one neutral resolver, consumed by both backends

`org-acl-core::MemberKeyResolver` is the **minimal substrate-neutral contract** (the
`spike-common` set). Each backend meets substrate-specific needs by **wrapping or
sub-trait-ing**, never by bloating core:

- **p2panda:** `ResolverPki<R: MemberKeyResolver>` wraps it; no extra trait methods
  needed for item-1.
- **Keyhive (later):** a `KeyhiveResolver: MemberKeyResolver` sub-trait adds
  `contact_card()`, living in the `org-acl` crate — not core.

Payoff: **`org-node` writes its trie-mirror resolver exactly once**, against the neutral
core, and whichever backend wins (or both) consumes it unchanged. This is the
convergence. (Contrast: the Keyhive plan had `org-node` implement an *enriched* trait
with `contact_card`, which would force `org-node` to mint Keyhive `ContactCard`s and drag
`keyhive_core` types toward the core — actively blocking convergence.)

### 4.2 SAME / ADAPT / DIVERGE map (vs the Keyhive item-1 plan)

| Keyhive plan element | p2panda track | Disposition |
|---|---|---|
| T0 — gate + re-pin + R13 Keyhive-reachability go/no-go | gate confirm + **G1 reachability re-confirm** (sanity, not a decision-reopening gate) | ADAPT |
| T1 — single crate `org-acl` | split **`org-acl-core`** + **`org-acl-p2panda`** | DIVERGE |
| T2 — identity composite (`OrgId`/`OrgMember`) | **simple** `Principal::Member(MemberId)/Org` | ADAPT |
| T3 — resolver *enriched* + StubResolver | resolver **minimal** + StubResolver (the convergence anchor) | ADAPT |
| T4 — `IdAdapter` cache | **skip** (re-resolve gives Flow-B for free) | DIVERGE |
| T5 — `ContactCard` e2e through live Keyhive | `GroupCrdt<AuthMemberId>` grant + `materialise_actor_id` through live p2panda | DIVERGE |
| T6 — L3 revocation + identity-takeover | **same shape**, p2panda resolution path | SAME |
| T7 — fuzz `OrgMember`/`Principal` codec | fuzz `Principal`/`MemberId` codec, in core | ADAPT |
| T8 — fuzz `ContactCard` ingestion | fuzz resolver→key boundary, in p2panda crate | DIVERGE |
| T9 / Final — README, suite, clippy `--lib`, squash user-signed | identical | SAME |

## 5. The p2panda wiring (productionising `spike-p2panda::s1_stable_id_acl`)

### 5.1 `org-acl-core` — identity + resolver (simple single-org)

```rust
// identity.rs — graduated from spike-common::identity, PII-free.
pub struct MemberId(pub [u8; 32]);          // SMT leaf key; opaque ACL principal
pub struct P2pMemberKey(pub VerifyingKey);  // member-as-a-group key
pub struct P2pDeviceKey(pub VerifyingKey);  // per-device key (CGKA tier)
pub struct OrgKey(pub VerifyingKey);         // org pseudo-group key
pub struct Epoch(pub u64);
pub enum Principal { Member(MemberId), Org } // delegation target

// resolver.rs — the minimal neutral contract (spike-common set).
pub trait MemberKeyResolver {
    fn p2p_member_key(&self, id: &MemberId) -> Result<P2pMemberKey, ResolverError>;
    fn org_key(&self) -> Result<OrgKey, ResolverError>;
    fn current_devices(&self, id: &MemberId) -> Result<Vec<P2pDeviceKey>, ResolverError>;
    fn org_member_ids(&self) -> Vec<MemberId>;
    fn is_member(&self, id: &MemberId) -> bool;
    fn epoch(&self) -> Epoch;
}
pub enum ResolverError { UnknownMember(MemberId), OrgKeyUnset }

// delta.rs — neutral per-epoch change set; the Δ==1 reconciliation input.
// org-node maps its SignedDeltaEnvelope onto this (item-4); item-1 tests build
// it synthetically. `removed` drives the ACL-tier reconcile; `rotated`/
// `org_key_rotated` are CGKA-tier signals consumed in item-4.
pub struct MembershipDelta {
    pub from_epoch: Epoch,
    pub to_epoch: Epoch,
    pub removed: Vec<MemberId>,
    pub added: Vec<MemberId>,
    pub rotated: Vec<MemberId>,
    pub org_key_rotated: bool,
}
```

**Lazy-pending signal.** "In the trie but client not online yet" is `current_devices`
→ `Ok(vec![])` (isolated, not a security failure), held distinct from `UnknownMember`
(`Err`, not a member). No `ContactCard`/`NoContactCard` variant — p2panda needs none.

### 5.2 `org-acl-p2panda` — the four seams

- **`AuthMemberId(MemberId)` + `impl IdentityHandle`** (`auth_id.rs`). Orphan-rule
  newtype: `IdentityHandle` (p2panda-auth) and `MemberId` (org-acl-core) are both
  foreign to this crate, so the auth-layer ID needs a local newtype + `From<MemberId>`.
  (This is the one spike escape-hatch that does *not* vanish under the neutral-core
  topology; it is small and contained. The encryption side needs none — `ResolverPki`
  is the local implementing type.)
- **`ResolverPki<R>`** (`pki.rs`) — *the trie-lookup key-resolution half*.
  `impl IdentityRegistry<MemberId, ResolverPki<R>> for ResolverPki<R>`; the caller
  passes `&self` as the static-method state. ed25519→x25519 byte reinterpretation for
  the PKI-lookup role (the identity key is a stable bundle-lookup anchor; ECDH uses
  separate prekey bundles — documented, carried from the spike).
- **`materialise_actor_id(resolver, principal) -> ActorId`** (`actor.rs`) — the
  spaces seam. Re-resolves on **every call** ⇒ no cache ⇒ Flow-B holds structurally;
  rotation is picked up automatically.
- **`DocAccess`** (`acl.rs`) — the per-document ACL: `GroupCrdt<AuthMemberId>::process` for
  Create/Add/Remove ops, below `p2panda-spaces`' eager `Manager` (lazy CGKA). Stores
  `org_epoch` (the trie epoch it was last reconciled to) and exposes
  `reconcile(resolver, Option<&MembershipDelta>)`: Δ==1 contiguous → prune `removed` ∩
  members and stamp `to_epoch`; else → full rebuild against `resolver.is_member` and stamp
  `resolver.epoch()`.

### 5.3 Flow-B invariant

No code path reads a `VerifyingKey` for a `Principal` except through the resolver.
With no cache (re-resolve every call), this holds **structurally** — asserted by a test
that rotates the member key and shows the next `materialise_actor_id` yields the new
`ActorId`.

## 6. Exit criteria & scope boundaries

- **Revocation (item-1 slice).** Member removed from trie → resolver returns
  `UnknownMember` → `materialise_actor_id`/`ResolverPki` resolve to nothing → no ACL
  grant/encryption to the revoked principal is constructible; **and** `DocAccess::reconcile`
  prunes the revoked member from the document's ACL (Δ==1 delta path *or* full rebuild),
  stamping the new org epoch. **The cryptographic forward-security half (`Dcgka::remove`)
  and the lazy-CGKA recompute are item-4, out of scope here** — this is the spike's
  documented `update`-vs-`remove` finding (`spike-p2panda/src/evidence/s3.md`).
- **Identity-takeover blocked.** The ACL binds to `MemberId` (handle@org) and keys
  resolve live through the trie only; an attacker's rotated/forged `VerifyingKey` the
  trie does not vouch for resolves to nothing — never authoritative.

| # | Boundary | Rationale |
|---|----------|-----------|
| P1 | p2panda is a *parallel track*, not the committed substrate | final pick deferred; decision.md §5 path |
| P2 | Ride spike pin `41559b0`; no fork for item-1 | aligns with qualification evidence; fork is items 2/4 |
| P3 | One resolver instance per org record | matches Phase 2 one-persona-per-org |
| P4 | `DocAccess` holds the ACL now; the CGKA half + `Dcgka::remove`/recompute are item-4 | item-1 is the ACL/resolver tier only |
| P5 | No `IdAdapter` cache; `reconcile` is pull-based (no auto-observer); open-docs-first scheduling = Phase-5 | re-resolve gives Flow-B; the observer/scheduler are item-4/Phase-5 |
| P6 | Transport (`p2panda-sync`/WASM gate 0) untouched | transport = item-5; rides Phase 2 iroh |

## 7. Testing (AGENTS.md hard rule: unit + scenario + fuzz)

- **`org-acl-core`** unit: postcard round-trips (`MemberId`/`P2pMemberKey`/`Principal`);
  `StubResolver` behaviour incl. `UnknownMember` vs isolated (`Ok(vec![])`).
- **`org-acl-p2panda`** unit: `ResolverPki` resolves / `None` on unknown;
  `materialise_actor_id` picks up rotation on re-resolve (Flow-B witness);
  `AuthMemberId: IdentityHandle`; ACL stores `AuthMemberId` not a key (CBOR-bytes check,
  promoted from the spike's gate-1 test 3).
- **L3 acceptance** (the exit criteria, promoted from `spike-p2panda/tests/l3_revocation.rs`,
  item-1 slice): revocation makes a member unresolvable; forged key never authoritative.
- **Fuzz** (bolero, `harness = false`, mirroring `on-chain-client`; default `cargo test`
  lane on stable + `cargo bolero` deep lane on nightly; committed corpus + `crashes/`):
  - `fuzz_identity_codec` (core) — `Principal`/`MemberId` postcard never-panic + round-trip.
  - `fuzz_resolver_key_boundary` (p2panda) — `materialise_actor_id`/`ResolverPki` on
    arbitrary `VerifyingKey` bytes never panic (the ed25519→x25519 reinterpret + `ActorId`
    construction boundary).
- **Clippy gate** `--lib` only (`-D unwrap_used/expect_used/panic`); test/fuzz code is
  exempt (that is how bolero signals failure).
- **Formal-model tie-in.** The revocation / identity-takeover properties witness the
  Phase 1.3 model's transitive-trust acceptance rules and the members-trie ↔ substrate
  sync contract; the L3 docstring cites the specific Quint invariant names.

## 8. Workflow / merge

- Work in a new git worktree branch **`worktree-phase-3-item1-p2panda`** (the existing
  empty `worktree-phase-3-item1-org-acl` is the Keyhive placeholder — left untouched).
- `git config extensions.worktreeConfig true && git config --worktree commit.gpgsign false`
  inside the worktree (AGENTS.md).
- Squash-merge to `master` as a single **user-signed** commit at the end.
- `~/.cargo` is read-only here; fetch the p2panda crates with
  `CARGO_HOME=/tmp/cargo_home_fuzz`.

## 9. References

- [`Organisational Data Sovereignty p1.md`](../../../Organisational%20Data%20Sovereignty%20p1.md) — §"Key changes required" item 1; §Scenarios (exit criteria); §lazy-CGKA / SKU.
- [`docs/phase-1d/decision.md`](../../phase-1d/decision.md) — Keyhive pick (conditional); p2panda salvage path §5; gap matrix.
- [`docs/phase-1d/spike-p2panda-decision.md`](../../phase-1d/spike-p2panda-decision.md) — p2panda qualification; what worked / needed work; escape hatches; `update`-vs-`remove` finding.
- [`2026-06-15-ods-phase-3-design.md`](2026-06-15-ods-phase-3-design.md) + [`…-item1-stable-id-acl.md`](../plans/2026-06-15-ods-phase-3-item1-stable-id-acl.md) — the Keyhive track this converges with.
- `spike-common/src/{identity,resolver,stub_trie}.rs` — graduating contract.
- `spike-p2panda/src/s1_stable_id_acl.rs` + `tests/l3_revocation.rs` — reference wiring for item-1.
- `on-chain-client/README.md` §Fuzzing — the bolero pattern.
