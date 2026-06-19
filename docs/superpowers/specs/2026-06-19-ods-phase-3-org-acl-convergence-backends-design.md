# ODS Phase 3 — org-acl convergence backends (Plans 2–4)

**Date:** 2026-06-19
**Status:** Design (approved for plans)
**Builds on:** `2026-06-19-ods-phase-3-org-acl-convergence-design.md` (the convergence design) and its Plan 1
(`org-acl-core` convergence primitives — landed on branch `worktree-phase-3-org-acl-convergence`,
9 commits, `47f8c99..1bf0efe`: unified `AclError`, `gate`, `decide_reconcile`/`ReconcileVerdict`,
R7 `find_member_by_device`, fuzz; 35 tests green, `clippy --lib` clean).

This spec designs the **remaining** convergence as three sequenced sub-projects (Plans 2 → 3 → 4),
each its own implementation plan, all on the same worktree/branch.

## 1. Decisions locked in this brainstorm

1. **No `DocAclBackend` trait.** Plan 1 already put the shared decision in pure core functions
   (`gate::{effective_members,is_effective}`, `authority::decide_reconcile → ReconcileVerdict`). The
   backends are sync/per-doc/move-based (p2panda) vs async/multi-doc/`&mut` (Keyhive); a single Rust
   trait spanning both adds friction for no gain. **Convergence = both backends route every decision
   through the core functions** and keep their idiomatic concrete types. This supersedes the original
   convergence design's `DocAclBackend` trait + "generic-over-trait conformance harness".
2. **Scenario conformance enforces parity.** A declarative fixture set in `org-acl-core::testing`
   pins identical observable outcomes across backends (see §2.3).
3. **"Compile to either" = facade feature selection**, not a runtime trait (§5).
4. **Sequencing 2 → 3 → 4**, all on `worktree-phase-3-org-acl-convergence`.

## 2. Convergence model (shared by all three plans)

### 2.1 Route the decision through core
Neither backend re-implements authority logic. For the live access answer they call
`gate::effective_members` / `gate::is_effective`. For reconcile they call
`decide_reconcile(&mark, resolver, witness, &ledger, &retain) -> Result<ReconcileVerdict, AclError>`
and then *execute* the returned verdict (`revoke`, `restamp_epoch`/`restamp_root`, `cgka_rekey` for
item-4) in their own idiom.

### 2.2 Identity & errors
- The delegation level is `OrgMember` everywhere (handles/`MemberId` are unique only within one org).
- The shared reconcile path returns core `AclError`. Backend-specific authoring that core does not
  model keeps a backend-local error (see Plan 3).

### 2.3 Scenario conformance (parity enforcement)
`org-acl-core::testing` gains a declarative fixture list — each scenario is
`(initial trie members, doc ledger, witness, expected effective set, expected revoked set)`. Each
backend's integration test iterates the **same** fixtures, drives its own concrete type, and asserts
the **post-execution observable state** (effective members via the live gate; membership after
executing the verdict). Because the *decision* is already shared by construction, this specifically
pins that each backend's *execution* of the verdict is faithful — the mechanical "strict parity"
guarantee. Fixtures include at minimum: revoke-on-departure, owner-not-exempt, same-epoch reorg,
delta-vs-full, cross-org exclusion.

## 3. Plan 2 — p2panda refit (depends on Plan 1)

`org-acl-p2panda/src/acl.rs` (`DocAccess`). Keeps its move-based, per-doc, sync shape (no `&mut` churn).

- **Reconcile through core.** Replace `DocAccess::reconcile`'s inline stale/reorg/delta-vs-full/prune
  logic with a call to `decide_reconcile(&self.mark, resolver, witness, &ledger, &retain)` where
  `ledger = self.ledger_members()` and `retain = [manager-as-OrgMember]`; then execute `v.revoke` via
  `GroupAction::Remove` and `self.mark.advance(org, v.restamp_epoch, v.restamp_root)`. **Delete**
  `reconcile_delta` / `reconcile_full`. `v.cgka_rekey` is carried but unused (item-4).
- **Gate the read side.** `effective_members` / `is_effective` delegate to `gate::*` and now return
  `OrgMember`.
- **`OrgMember` delegation surface.** `add_member` / `remove_member` / `set_access` take the grantee
  as `OrgMember`; reject `grantee.org != resolver.org_id()` with `AclError::WrongOrg`. `ledger_members`
  returns `Vec<OrgMember>`. Author gate stays `resolver.is_member`.
- **Adopt core `AclError`.** Delete the local enum; map CRDT rejection to
  `AclError::Backend("acl crdt rejected".into())`. `AuthorNotMember` carries `OrgMember`.
- **R7.** `org-acl-p2panda` consumes `&R: MemberKeyResolver`; tests use core `StubResolver` (already
  implements `find_member_by_device`). The plan verifies whether `pki.rs` / `g1_probe.rs` define an
  in-crate resolver impl needing the method; the real resolver is `org-node`'s (Plan 4).
- **Tests.** Retarget the existing `acl.rs` suite to `OrgMember` (stays green); author the shared
  conformance fixtures in `org-acl-core::testing`; add the p2panda conformance test iterating them.

Net: `DocAccess` becomes a thin p2panda *executor* over the core decision + gate.

## 4. Plan 3 — Keyhive migration (depends on Plan 1; reuses Plan 2 fixtures)

- **Import (this worktree).** Copy `doc-access` from `worktree-phase-3-item1-org-acl` into
  `org-acl-keyhive/`; add to `[workspace] members`. It stays the **only** GPL-3.0-only crate and the
  only one naming `keyhive_core`/`beekem`/`keyhive_crypto`. Its `keyhive_core` pin comes with it.
- **Drop parallel types, adopt core.** Delete duplicate identity types (`OrgId`/`MemberId`/`OrgMember`/
  `Epoch`/`RootHash`/`Principal`/`P2p*`/`OrgKey`) and `ChangeSet`; depend on `org-acl-core`. Keep only
  `DocId` (the `keyhive_core` `DocumentId` re-export). The per-doc stamp becomes core
  `DocReconcileMark`; the Keyhive-only `delegated: HashMap<OrgMember, Identifier>` map stays (the
  `Identifier` is GPL-side, needed to revoke).
- **Swap the witness (closes the serde forge-hole).** Replace the local serde `VerifiedTrieChange`
  with core's non-serde one; update accessors (`org_id→org`, `epoch→to_epoch`, `root_hash→to_root`,
  `change_set→delta`).
- **Reconcile through core (closes the epoch-only reorg bug).** `DocAclWriter::reconcile(doc_handle,
  witness)` drops its `witness.epoch() <= acl_epoch` check and manual prune. New body:
  `decide_reconcile(&stamp.mark, resolver, witness, &delegated_orgmembers, &[owner])`, then execute
  `v.revoke` via `kh.revoke_member(stored Identifier)`, map `v.cgka_rekey →
  ReconcileReport.cgka_rekey_needed`, advance `stamp.mark` to `v.restamp`. `needs_reconcile` is
  reworked to test `witness.delta()` (`MembershipDelta`, `Vec<MemberId>` scoped by `witness.org()`)
  instead of `ChangeSet`.
- **`KeyhiveResolver` sub-trait.**
  `pub trait KeyhiveResolver: MemberKeyResolver { fn contact_card(&self, id: &MemberId) -> Result<ContactCard, ResolverError>; }`.
  `DocAclWriter<R>` binds `R: KeyhiveResolver`. The Keyhive resolver impls core `MemberKeyResolver`
  (incl. R7) + this sub-trait; `NoContactCard` lives on the sub-trait's error path. No GPL type leaks
  into core.
- **Error split.** `reconcile` returns core `AclError` (it now yields `WrongOrg`/`StaleWitness` via
  `decide_reconcile`). Keyhive-specific authoring (`delegate`/`revoke_delegation`/`amend`, which touch
  `ContactCard` + live Keyhive ops) keeps a Keyhive-local error (`NotAMember`/`NoContactCard`/
  `Keyhive(String)`/`UnknownDoc`) — core stays free of Keyhive-only variants.
- **Per-doc unit.** Keyhive's ACL state is already per Keyhive `Document`; the per-doc stamp is now
  `{ mark: DocReconcileMark, delegated }`. The CGKA slot (`cgka_epoch`) is reserved for item-4.
- **Conformance.** Run the Plan 2 fixtures against `DocAclWriter` (async test), incl. a same-epoch
  reorg fixture that would have failed the old epoch-only check.

## 5. Plan 4 — facade + `org-node` repoint + conformance/CI (depends on 2 + 3)

- **`org-acl` facade crate (new, permissive).** Features `p2panda` and `keyhive`, **no default**. A
  `compile_error!` fires unless **exactly one** is enabled (zero → "enable a backend"; both → "enable
  exactly one"). `org-acl-keyhive` is an `optional = true` dependency enabled only by the `keyhive`
  feature, so a p2panda build links **zero** GPL; enabling `keyhive` brings GPL transitively. The
  facade always `pub use org_acl_core::*` and, under the active feature, re-exports that backend's
  concrete types (`DocAccess` / `DocAclWriter`, and `DocId`). Consumers naming only the facade's
  re-exports stay backend-agnostic in source.
- **`org-node` repoint.** `org-node` depends on the facade (selecting a backend feature) and
  implements `MemberKeyResolver` (incl. R7) over its per-org trie mirror; under the `keyhive` feature
  it also implements `KeyhiveResolver` (`contact_card`). It stops naming `keyhive_core` directly. The
  exact current `org-node` ACL wiring is confirmed at plan time; this fixes the dependency direction.
- **Conformance + CI.** CI builds **both** lanes (`--features p2panda`, `--features keyhive`) and runs
  each backend's conformance test over the shared `org-acl-core::testing` fixtures. The keyhive lane
  carries the heavier `keyhive_core` git-pin build cost. A `trybuild` compile-fail check pins the
  facade's zero-feature and both-feature `compile_error!` guards.
- **Keyhive maturity gate.** The `keyhive` feature is **experimental**: its provisional pin
  (`keyhive_core` main, no tagged 0.1.0, no external audit, Phase 1.d gate-b waived) must be resolved
  before production use. Strict parity still holds (both pass the same conformance bar), but the
  `keyhive` feature ships with the documented re-pin-and-audit caveat. The gate blocks Keyhive-feature
  production use, not the p2panda build.

## 6. Scope boundaries (YAGNI)

- No runtime `dyn` backend; no `DocAclBackend` trait; no async wrappers on the sync backend.
- No item-2/4/5 work (org pseudo-group, CGKA triggers, p2p policy). `cgka_rekey` / `cgka_epoch` are
  *carried/reserved* only. iroh connection gating (item-5) composes R7 later; not in scope here.
- No succession/owner-recovery; the intersection gate makes a no-longer-vouched owner inert.
- `IdAdapter` and `ContactCard` stay internal to `org-acl-keyhive`.

## 7. References

- `2026-06-19-ods-phase-3-org-acl-convergence-design.md` — the convergence design (topology, spine).
- `2026-06-19-ods-phase-3-org-acl-convergence.md` — Plan 1 (landed) + the original 2–4 roadmap this
  supersedes with the no-trait model.
- `2026-06-15-ods-phase-3-design.md` — five-sub-project decomposition; item-2/4/5 remain future cycles.
