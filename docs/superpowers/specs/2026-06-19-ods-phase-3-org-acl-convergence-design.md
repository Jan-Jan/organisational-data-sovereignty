# ODS Phase 3 — `org-acl` convergence: one library, two backends

**Date:** 2026-06-19
**Status:** Design (approved for plan)
**Supersedes/extends:** the two parallel item-1/item-3 tracks
(`2026-06-15-ods-phase-3-item1-stable-id-acl.md` + design; `2026-06-18-ods-phase-3-item1-p2panda.md`,
`2026-06-18-ods-phase-3-item3-p2panda.md` + designs). The five-sub-project decomposition in
`2026-06-15-ods-phase-3-design.md` §4 still stands; this spec is the convergence sub-project that
must land before items 4/2/5.

## 1. Overview

Two ACL substrate tracks were built in parallel:

- **p2panda** (`worktree-phase-3-item3-p2panda`): already split into a neutral `org-acl-core` crate
  (`identity`, `delta`, `mark`, `resolver`, `role`, `witness`, `test_support`) plus an
  `org-acl-p2panda` backend (`DocAccess` over `GroupCrdt`).
- **Keyhive** (`worktree-phase-3-item1-org-acl`): a single `doc-access` crate that holds its **own
  parallel copies** of identity/resolver/witness/changeset and names `keyhive_core` directly. It
  carries richer machinery (`IdAdapter`, `ContactCard`, `find_member_by_device`/R7, `DocIndex`,
  active hard-revoke).

A cross-track review found the two implementations had **diverged on the authority spine** (Keyhive's
reconcile stale-check is epoch-only and rejects same-epoch reorgs; p2panda's is reorg-aware) and that
Keyhive's `VerifiedTrieChange` derives `Serialize`/`Deserialize`, making the "unforgeable capability"
forgeable from bytes.

This spec converges both tracks into **one library that compiles to either backend**, closes the
review gaps **by construction** (single shared implementation of the authority logic), and places the
seams items 4/2/5 need so they slot on without a redesign.

### Decisions locked in this brainstorm

1. **Architecture:** shared core + backend crates + facade (not a single feature-flagged crate, not a
   runtime `dyn` backend). Driven by the GPL/permissive license split.
2. **Scope:** this spec designs the unification + gap-closure and *maps* how items 4/2/5 attach. Items
   4/2/5 remain separate spec→plan→impl cycles.
3. **Backend posture:** strict parity, no default backend. Both backends are first-class and held to
   the same behavioural bar via a shared conformance harness; the consumer must choose a backend
   explicitly. Keyhive's provisional-pin/audit gate blocks the Keyhive build, not the p2panda build.

## 2. Architecture — workspace topology & license boundary

Four crates:

```
org-acl-core/        # permissive (Apache-2.0/MIT) — ZERO substrate deps
  identity.rs   # OrgId, MemberId, OrgMember, P2pMemberKey, P2pDeviceKey, OrgKey, Epoch, Principal, RootHash
  delta.rs      # MembershipDelta (the neutral change type)
  mark.rs       # DocReconcileMark (DocId × OrgId matrix), OrgEpochs
  resolver.rs   # MemberKeyResolver (now incl. find_member_by_device / R7)
  role.rs       # AclRole (Read < Write < Manage)
  witness.rs    # VerifiedTrieChange (non-serde capability) + trybuild unforgeability proof
  backend.rs    # NEW: DocAclBackend trait — the unified ACL surface
  authority.rs  # NEW: the shared reconcile driver + intersection gate
  index.rs      # NEW (generalised from Keyhive DocIndex): neutral org→docs reverse index

org-acl-p2panda/     # permissive — impls DocAclBackend over GroupCrdt
  acl (DocAccess), actor, auth_id, pki            # substrate glue retained

org-acl-keyhive/     # GPL-3.0-only — the ONLY crate naming keyhive_core / beekem / keyhive_crypto
  (renamed from doc-access) impls DocAclBackend over Keyhive
  adapter (IdAdapter), khtypes, KeyhiveResolver sub-trait (ContactCard), hard-revoke
  # DELETES its parallel identity/witness/mark/ChangeSet copies → depends on org-acl-core

org-acl/             # facade — re-exports core + exactly ONE backend
  features = ["p2panda"] | ["keyhive"]; NO default feature
  compile_error! if zero backends enabled (strict-parity: consumer must choose)
```

- **License containment.** GPL stays sealed inside `org-acl-keyhive`. A p2panda-only build never
  compiles a GPL line. The facade's GPL exposure is conditional on the `keyhive` feature.
- **"Compile to either" = dependency/feature selection**, not `cfg` soup. `org-node` and the Tauri app
  depend on the `org-acl` facade and pick the backend feature; they only ever name neutral
  `org-acl-core` types.
- **`org-node` impact.** Today the Keyhive `doc-access` is named directly; after the merge `org-node`
  implements `MemberKeyResolver` against the facade and is backend-agnostic.

## 3. The shared authority spine (implemented once)

The authority model is substrate-neutral and becomes a **single implementation** in
`org-acl-core::authority`, parameterised over a small set of backend hooks. Neither backend
reimplements it, so the Keyhive/p2panda divergence becomes impossible by construction.

Two neutral functions:

```rust
// 1. The live gate — effective access = ledger ∩ currently-trie-vouched.
//    Owner/manager NOT exempt. No cache. Recomputed every call.
pub fn effective_members<R: MemberKeyResolver>(
    ledger: impl Iterator<Item = OrgMember>, resolver: &R,
) -> BTreeSet<OrgMember>;   // retain only is_member()==true

// 2. The reconcile driver — owns accept/reject + delta-vs-full + revoke decisions.
pub fn drive_reconcile<R, B>(
    mark: &mut DocReconcileMark, resolver: &R,
    witness: &VerifiedTrieChange, backend: &mut B,
) -> Result<ReconcileReport, AclError>
where R: MemberKeyResolver, B: DocAclBackend;
```

`drive_reconcile` makes the whole decision once:

- **Org match.** Reject `witness.org() != resolver.org_id()` → `WrongOrg`.
- **Monotonicity + reorg.** Reject iff `to_epoch < cell.acl_epoch` **or**
  (`to_epoch == cell.acl_epoch` **and** `to_root == cell.root_hash`). A same-epoch **different-root**
  witness is a reorg and is **accepted** → full rebuild. (This is p2panda's correct logic; adopting it
  centrally auto-fixes Keyhive's epoch-only bug.)
- **Δ==1 chaining.** If the witness carries a single-step delta whose `from_epoch` chains the current
  cell, call `backend.apply_delta(delta)`; otherwise `backend.apply_full_reload(resolver)`.
- **Departed members.** Call `backend.revoke(member)` for each member dropped by the change. Keyhive's
  active hard-revoke (via the stored `Identifier`) and p2panda's CRDT `Remove` are now two
  implementations of the **same** `revoke` hook → revoke parity for free.
- **Advance** the per-org `mark` cell (the `DocId × OrgId` matrix) only on accept.

Backend hooks the driver needs (kept tiny — substrate mechanics only):

```rust
fn ledger_members(&self) -> Vec<OrgMember>;                  // advisory raw membership
fn apply_delta(&mut self, d: &MembershipDelta) -> Result<(), Self::Err>;
fn apply_full_reload<R: MemberKeyResolver>(&mut self, r: &R) -> Result<(), Self::Err>;
fn revoke(&mut self, m: OrgMember) -> Result<(), Self::Err>;
```

The witness is the non-serde `VerifiedTrieChange` from core. Keyhive's serde forge-hole disappears the
moment it stops carrying its own copy.

## 4. The `DocAclBackend` trait

In `org-acl-core::backend` — the merge of p2panda's `DocAccess` and Keyhive's `DocAclWriter`:

```rust
pub trait DocAclBackend: Sized {
    type DocId: Clone + Eq + Hash;   // Keyhive = DocumentId; p2panda = log/hash id (substrate-specific)
    type Err: Into<AclError>;

    // — authoring (all author-gated: AuthorNotMember if !resolver.is_member(author)) —
    fn create(owner: OrgMember) -> Self;
    fn add_member(&mut self, author: OrgMember, m: OrgMember, role: AclRole) -> Result<(), Self::Err>;
    fn remove_member(&mut self, author: OrgMember, m: OrgMember)             -> Result<(), Self::Err>;
    fn set_access(&mut self, author: OrgMember, m: OrgMember, role: AclRole) -> Result<(), Self::Err>;

    // — driver hooks (§3) —
    fn ledger_members(&self) -> Vec<OrgMember>;
    fn apply_delta(&mut self, d: &MembershipDelta) -> Result<(), Self::Err>;
    fn apply_full_reload<R: MemberKeyResolver>(&mut self, r: &R) -> Result<(), Self::Err>;
    fn revoke(&mut self, m: OrgMember) -> Result<(), Self::Err>;
    fn mark(&mut self) -> &mut DocReconcileMark;

    // — convenience, DEFAULTED in core over the spine (§3); backends inherit identical behaviour —
    fn effective_members<R: MemberKeyResolver>(&self, r: &R) -> BTreeSet<OrgMember> { /* core gate */ }
    fn is_effective<R: MemberKeyResolver>(&self, r: &R, m: OrgMember) -> bool { /* core gate */ }
    fn reconcile<R: MemberKeyResolver>(&mut self, r: &R, w: &VerifiedTrieChange)
        -> Result<ReconcileReport, AclError> { /* drive_reconcile */ }
    fn needs_reconcile<R: MemberKeyResolver>(&self, r: &R, w: &VerifiedTrieChange) -> bool { /* mark cmp */ }
}
```

- **Authoring is per-backend** (substrate write mechanics differ) but every method delegates its
  author-gate to the shared `is_member` check.
- **`add_member` always requires an `AclRole`** — every member lands on a doc with an explicit role;
  there is no roleless add.
- **`set_access`** changes an existing member's role (promote/demote by `AclRole` cmp). **`set_access`
  on a principal who is not already a member is an error**, not an implicit add — the two operations
  stay cleanly separated.
- **`reconcile` / `needs_reconcile` / `effective_members` / `is_effective` are defaulted in core** over
  the spine, so both backends get identical behaviour and the lazy pull-check (`needs_reconcile`, which
  p2panda lacked) lands for free.
- **`AclError` is unified in core**: `AuthorNotMember(OrgMember)`, `WrongOrg { expected, got }`,
  `StaleWitness { mark, witness }`, and a backend-wrapped `Backend` variant (CRDT/Keyhive rejection).

## 5. Resolver changes

In `org-acl-core::resolver`:

- **Promote `find_member_by_device` (R7) into the core trait.** It is substrate-neutral, both backends
  need it, and item 5's connection-gating composes it — so the seam belongs in core now. p2panda gains
  an impl; Keyhive keeps its trie-backed one.
- **`ContactCard` stays OUT of core** (it is a GPL `keyhive_core` type). `org-acl-keyhive` defines
  `KeyhiveResolver: MemberKeyResolver { fn contact_card(&self, id: &MemberId) -> Result<ContactCard, ResolverError>; }`
  as a sub-trait — the extension pattern the core resolver doc already prescribes. No GPL leaks into
  core.
- The core `ResolverError` keeps `UnknownMember`, `OrgKeyUnset`, `WrongOrg`; `NoContactCard` lives on
  the Keyhive sub-trait's error surface.

## 6. Type unification

- **`MembershipDelta` wins over Keyhive's `ChangeSet`.** It is the richer neutral type
  (`from/to_epoch`, `from/to_root`, `added/removed/rotated`, `org_key_rotated`) and is already what the
  witness carries. Keyhive drops `ChangeSet`.
- **`DocId` is an associated type, not a core newtype.** Keyhive's is a `DocumentId` (VerifyingKey);
  p2panda's is a log/hash id. Forcing one shape would leak substrate into core.
- **`DocReconcileMark` / `OrgEpochs` / `VerifiedTrieChange` / `AclRole` / `Principal` / `OrgMember`**
  are already neutral and serde-correct (`mark` serde-serializable; `witness` intentionally non-serde).
  Keyhive adopts them verbatim.

## 7. How items 4/2/5 attach (no redesign required)

Each remains its own later spec→plan→impl cycle; the merge only fixes the seams.

- **Item 4 — trie-driven CGKA triggers.** `mark.cgka_epoch` already exists, lockstep with `acl_epoch`
  today. Item 4 *decouples* it: adds one backend hook `force_pcs_update(resolver)` and lets the shared
  driver advance `cgka_epoch` independently on device-rotation witnesses. No trait reshape — an added
  hook plus a driver branch.
- **Item 2 — org pseudo-group + org→docs reverse index.** The neutral `org-acl-core::index`
  (generalised from Keyhive's `DocIndex`) is the shared reverse index that 4's cascade and 2's
  delegation both consume. Org-level delegation is `Principal::Org(org)` flowing through the existing
  authoring methods — the `Principal` enum already models it.
- **Item 5 — p2p connection policy.** Composes `resolver.is_member` + `find_member_by_device` (now in
  core) + the reverse index, over iroh. Out of scope here; the only seam the merge owes it (R7 in core)
  is now placed.

## 8. Migration plan

1. Land `backend.rs` + `authority.rs` + `index.rs` in `org-acl-core`; promote R7 into the resolver.
2. Rename `doc-access` → `org-acl-keyhive`; delete its parallel `identity`/`witness`/`mark`/`ChangeSet`;
   depend on `org-acl-core`; implement `DocAclBackend` + the four hooks; retain `IdAdapter`, `khtypes`,
   the `KeyhiveResolver` sub-trait, and hard-revoke (now the `revoke` hook).
3. Refit `org-acl-p2panda::DocAccess` onto the trait (it already matches closely); implement
   `find_member_by_device`.
4. Add the `org-acl` facade (feature-gated, no default, `compile_error!` guard).
5. Repoint `org-node` at the facade.

## 9. Testing

- **Spine suite (core, once):** gate intersection (owner not exempt), monotonicity rejection,
  same-epoch-reorg accept, delta-vs-full dispatch, revoke-on-departure — run against a `StubBackend`.
- **Trait-conformance harness (generic over `DocAclBackend`):** the **same** behavioural assertions run
  against *both* backends. This is what mechanically enforces strict parity.
- **Per-backend conformance suites:** thin, proving each backend's hooks + authoring + substrate glue.
- **Unforgeability:** the trybuild compile-fail proof for `VerifiedTrieChange` moves to core.
- **Fuzz:** bolero targets per AGENTS.md (mirroring the `on-chain-client` pattern), covering the
  reconcile driver's accept/reject decision and the gate.

## 10. The Keyhive gate (strict-parity consequence)

`org-acl-keyhive` carries a build-level risk gate: its provisional pin (GPL `keyhive_core` main @
d562ffb, no tagged `0.1.0`, no external audit, Phase 1.d gate-b waived) must be resolved before the
`keyhive` facade feature is considered production-ready. CI builds **both** backends and the
conformance harness must pass for both. The gate blocks Keyhive-feature production use; it does not
hold back the p2panda build.

## 11. Scope boundaries (YAGNI)

- No runtime `dyn` backend selection; no single feature-flagged mega-crate.
- No succession/owner-recovery logic — a doc whose sole manager is revoked is lost; survivors fork
  (copy → new doc). The intersection gate makes a no-longer-vouched owner inert.
- Transport (iroh/Beelay) remains out of scope; item 5 owns the policy layer only.
- `IdAdapter` and `ContactCard` stay backend-internal to `org-acl-keyhive`; they are not forced into
  core.

## 12. References

- `2026-06-15-ods-phase-3-design.md` — five-sub-project decomposition (§4), spine (§3.2), observer (§3.3).
- `2026-06-15-ods-phase-3-item1-stable-id-acl.md`, `2026-06-18-ods-phase-3-item1-p2panda.md`,
  `2026-06-18-ods-phase-3-item3-p2panda.md` — the two parallel tracks this converges.
