# ODS Phase 3 item-3 on p2panda — trie as sole write authority

**Author(s):** Jan-Jan van der Vyver (design captured via brainstorming session)
**Status:** In review
**Created:** 2026-06-18
**Spec for:** Phase 3 of [`Organisational Data Sovereignty p1.md`](../../../Organisational%20Data%20Sovereignty%20p1.md) §"Key changes required" item 3 — the members trie is the sole write authority for member/device key state; library-native operations that would mutate it are structurally disabled.
**Builds on:** item-1 ([`2026-06-18-ods-phase-3-item1-p2panda-design.md`](2026-06-18-ods-phase-3-item1-p2panda-design.md), incl. §1b R2) — the `org-acl-core` / `org-acl-p2panda` crates and `DocAccess`. **Sequencing 1 → 3 → 4 → 2 → 5.**

---

## 1. Overview

"Trie is the sole write authority" decomposes, on this architecture, into **two complementary
guarantees**:

1. **Identity/key state is trie-only.** Who is an org member, their member-as-a-group key,
   their devices, and the org key are mutated *only* by the members trie (via `org-node` /
   on-chain — Phase 2). No path in `org-acl` can write that state.
2. **Document-ACL writes are trie-authorized.** A document's ACL (who may read/write it, and at
   what level) *is* mutated locally — but only by an actor who is **(a) currently in the trie**
   *and* **(b) holds the rights** on that document (`Manage`). Authority to write the ACL is
   rooted in the trie; a revoked member loses it.

**No p2panda fork is required.** The spike's gate-2 Hard finding was the `p2panda-spaces`
`AuthStore` intercept (the `pub use types::AuthGroupState` fork). Item-1 deliberately drives
`p2panda-auth::GroupCrdt` directly, *below* the `p2panda-spaces` `Manager`, and uses only the
`ActorId` type from spaces — so that layer is never touched. **No `BlockingGroups` either:** the
raw `Groups`/`GroupCrdt` mutation surface is private to `org-acl-p2panda`; consumers see only
`DocAccess`'s curated API, so library-native mutation is unreachable by encapsulation.

## 2. Guarantee 1 — identity/key state is trie-only (asserted invariant)

This already holds by item-1's construction; item-3 makes it explicit and tested:

- The ACL stores stable `OrgMember` ids + access levels — **never keys** (`AuthMemberId(OrgMember)`).
- `MemberKeyResolver` is **read-only** — it has no mutating methods (the production resolver is
  `org-node`'s trie mirror; the test `StubResolver`'s builder mutators exist only behind
  `--features testing`).
- The raw p2panda mutation surface (`GroupCrdt::process`, `AclOp`, the `Groups` trait) is
  **private to `org-acl-p2panda`**; `DocAccess` is the only public mutation API.

**Deliverable:** a crate-level invariant doc statement, plus a test asserting that the only way
to obtain/alter a member/device/org key is the (read-only) resolver — i.e. there is no
`org-acl` API that writes identity/key state. (Compile-level: the resolver trait exposes no
`&mut`/mutating method; documented as the "Flow-A" companion to item-1's Flow-B.)

## 3. Guarantee 2 — trie-authorized document-ACL writes

### 3.0 Effective membership — advisory ACL, authoritative trie (the authority model)

The materialised ACL — the `GroupCrdt` membership ledger — is **advisory**; the **members trie
(via the resolver) is authoritative**. A member's *effective* access to a document is

> `effective = ledger-membership(member) ∧ resolver.is_member(member)`

computed **live** via `DocAccess::effective_members(resolver)` / `is_effective(resolver, member)`,
**independent of `reconcile`**. Consequences:

- A member **revoked from the trie has no effective access immediately**, even before the doc is
  reconciled. `reconcile` (§3.3) only *materialises* the advisory ledger toward the authoritative
  trie to bound staleness — it is **not** the security boundary; the live intersection is.
- **The owner/manager is not exempt.** A revoked manager has no effective access. The raw CRDT
  ledger may still list them (the group root can't be removed), but the live intersection excludes
  them — no owner exemption, no succession, fork-on-loss. (This subsumes item-1's "revoked manager"
  edge: effective membership, not a manager skip, is the answer.)

This matches the Keyhive track's finalised authority model (convergence): the on-chain trie is the
single source of truth; the local-first ACL is a cache over it.

### 3.1 Write authority

An ACL mutation is valid iff its **author**:
- **(a) is a current trie member** — `resolver.is_member(author)` — enforced by `org-acl`; and
- **(b) holds the required access level** (`Manage` to add/remove/change-access) — enforced
  **natively by `p2panda-auth`'s `GroupCrdt`** when it processes the op (the spike confirmed the
  CRDT rejects an op whose author lacks the level; we do not duplicate that check).

The two checks are orthogonal: (a) is "are you a real, non-revoked member" (trie); (b) is "do
you have rights on this doc" (the local-first ACL CRDT). Both must pass.

### 3.2 `DocAccess` becomes author-aware (revises item-1)

Item-1's `DocAccess` had a single implicit `manager` authoring everything and ungated grants.
Item-3 generalises:

```rust
// org-acl-core — new neutral, substrate-agnostic role (shared convergence type)
pub enum AclRole { Read, Write, Manage }

// org-acl-p2panda — DocAccess mutation API (author-aware, trie-gated)
impl DocAccess {
    /// Create a doc ACL; `founder` becomes the initial Manage holder.
    /// Trie-gates the founder. Stamps the DocReconcileMark from the resolver (item-1).
    fn create<R: MemberKeyResolver>(doc_id: MemberId, founder: MemberId, resolver: &R) -> Result<Self, AclError>;

    /// `author` (a current trie member with Manage) grants `member` access at `role`.
    fn add_member<R: MemberKeyResolver>(self, resolver: &R, author: MemberId, member: MemberId, role: AclRole) -> Result<Self, AclError>;

    /// `author` (Manage) removes `member`'s delegation.
    fn remove_member<R: MemberKeyResolver>(self, resolver: &R, author: MemberId, member: MemberId) -> Result<Self, AclError>;

    /// `author` (Manage) changes `member`'s access level.
    fn set_access<R: MemberKeyResolver>(self, resolver: &R, author: MemberId, member: MemberId, role: AclRole) -> Result<Self, AclError>;

    /// Effective members = advisory ledger ∩ authoritative trie, computed LIVE.
    /// Independent of `reconcile`; the owner/manager is NOT exempt.
    fn effective_members<R: MemberKeyResolver>(&self, resolver: &R) -> Vec<MemberId>;

    /// True iff `member` is in the ledger AND currently vouched by the trie.
    fn is_effective<R: MemberKeyResolver>(&self, resolver: &R, member: MemberId) -> bool;
}
```

`effective_members`/`is_effective` are the read-side authority check (§3.0): they intersect the
advisory `member_ids()` ledger with `resolver.is_member` on every call — no caching, no manager
special-case.

Each mutation: **(1)** `if !resolver.is_member(&author) → Err(AclError::AuthorNotMember(author))`;
**(2)** build the `AclOp` authored by `AuthMemberId(OrgMember{org, author})` with the `AclRole`→
`p2panda_auth::Access<()>` mapping (`Read→read()`, `Write→write()`, `Manage→manage()`); **(3)**
`GroupCrdt::process` — if the author lacks the level the CRDT rejects → `Err(AclError::CrdtRejected)`.

`set_access` uses the CRDT's promote/demote path; **the exact `GroupAction` variant for an
access change at the pinned rev must be confirmed first** (`Promote`/`Demote` vs re-`Add`) — the
implementation plan's first task verifies it against `p2panda-auth` source (same approach item-1
used for `GroupAction::Remove`/`Access::write`).

### 3.3 `reconcile` is lazy materialisation, not the boundary

`DocAccess::reconcile` (item-1) prunes trie-revoked members from the advisory ledger as a
**trie-driven materialisation** — it bounds ledger staleness so reads stay cheap, but it is **not**
the security boundary (§3.0's live `effective_members` is). It remains authored by the doc's
manager and is *not* subject to the §3.1(a) author gate (the trie is *enforcing* removal, not a
user). It keeps the item-1 behaviour of not removing the group root at the CRDT level (the root
can't be removed); a revoked manager therefore lingers in the *raw* ledger until a fork, but
`effective_members` already excludes them, so this is harmless. Otherwise unchanged from item-1,
now coexisting with the author-aware mutators.

### 3.4 Errors

`AclError` gains distinguishable variants: `AuthorNotMember(MemberId)` (trie gate failed — the
author is not a current org member) vs `CrdtRejected` (the CRDT refused — typically the author
lacks `Manage`, or an invalid op). Distinct so callers/tests can tell "you're not a member" from
"you lack rights."

## 4. Exit criteria (ODS Roadmap.3 item-3)

- **Library-native membership ops are unreachable from the API surface** — the only way to
  change an ACL is `DocAccess`'s author-gated, rights-checked API; `GroupCrdt`/`Groups` raw
  mutation is private; identity/key state is never mutated by `org-acl` (Guarantee 1).
- **A revoked member cannot write any ACL** — once removed from the trie, `is_member(author)` is
  false, so every mutation by them returns `AuthorNotMember`, even on a doc where they still
  appear in the (stale, pending-reconcile) ACL.
- **A non-`Manage` member cannot add/remove/relevel** — the CRDT rejects it (`CrdtRejected`).
- **Effective access tracks the trie live** — a member (incl. the owner/manager) revoked from the
  trie is absent from `effective_members` *immediately*, before any `reconcile`; effective access
  equals `ledger ∩ is_member` (§3.0).

## 5. Testing

- **Authority (the core of item-3):** founder (trie member) creates + grants; a granted `Manage`
  member (trie member) can add/remove/relevel; a `Write`-only member is rejected (`CrdtRejected`);
  a non-trie-member author is rejected (`AuthorNotMember`); a member **revoked from the trie**
  loses ACL-write authority (`AuthorNotMember`) even while still listed in the doc's ACL.
- **Effective membership (§3.0):** `effective_members` = ledger ∩ `is_member`; a trie-revoked
  member is excluded **before** reconcile; a **revoked owner/manager is excluded too** (not exempt);
  `is_effective` agrees. After `reconcile` the advisory ledger shrinks but `effective_members` is
  unchanged (idempotent w.r.t. the live answer).
- **Guarantee-1 invariant:** a test/doc-assertion that no `org-acl` API mutates identity/key
  state (resolver is read-only; only `DocAccess` mutates, and only ACL membership/levels).
- **Regression:** item-1's grant / revocation / reconcile / identity-takeover tests still pass
  under the author-aware API (updated to pass a trie-member author).
- **Fuzz:** extend or add a bolero target if the new surface warrants it (e.g. `AclRole` codec);
  otherwise item-1's targets stand. (AGENTS.md always-fuzz rule.)
- Clippy `--lib` deny-gate clean; `CARGO_HOME=/tmp/cargo_home_fuzz`.

## 6. Scope boundaries

| # | Boundary | Rationale |
|---|----------|-----------|
| S1 | **No p2panda fork** | we never use the spaces `AuthStore`; the gate-2 fork is moot here |
| S2 | **No `BlockingGroups`** | the raw `Groups`/`GroupCrdt` surface is private — encapsulation disables it |
| S3 | Rights enforced **natively** by `GroupCrdt`; org-acl adds only the trie author-gate | no duplicated authority logic |
| S4 | Key/device/org-key **rotation cascades** (CGKA rekey) | item-4 |
| S5 | The **org pseudo-group** as an ACL member (`Principal::Org`, nested groups) | item-2 |
| S6 | No custom access **conditions** — `Access<()>` | matches item-1 |
| S7 | `set_access` `GroupAction` variant verified at impl time | API drift guard (as in item-1) |

## 7. Convergence

`AclRole` joins `org-acl-core` as another shared neutral type (alongside `MemberKeyResolver`,
`MembershipDelta`, `DocReconcileMark`) — the Keyhive track consumes the same role enum and the
same author-gated authority model. The trie-author-gate + native-rights split is substrate-neutral.

**Authority model converged with the Keyhive track:** advisory ACL / authoritative trie, effective
access = `ledger ∩ is_member` recomputed live and independent of reconcile, owner not exempt (no
root removal / succession; fork-on-loss). Divergence kept deliberate: the Keyhive track stamps a
per-`(doc, org)` map (forward-looking for cross-org docs); the p2panda track keeps item-1's
single-org `DocReconcileMark` (cross-org collaboration is an ODS non-goal).

## 8. Workflow / merge

- Item-1 is unmerged on `worktree-phase-3-item1-p2panda`. Item-3 **stacks on it**: branch
  `worktree-phase-3-item3-p2panda` from the item-1 branch HEAD. (When item-1 lands on master,
  item-3 rebases/merges after.)
- `commit.gpgsign false` in the worktree; squash-merge as one user-signed commit at the end.
- `~/.cargo` read-only → `CARGO_HOME=/tmp/cargo_home_fuzz`.

## 9. References

- [`Organisational Data Sovereignty p1.md`](../../../Organisational%20Data%20Sovereignty%20p1.md) — §"Key changes required" item 3.
- [`2026-06-18-ods-phase-3-item1-p2panda-design.md`](2026-06-18-ods-phase-3-item1-p2panda-design.md) — `DocAccess`, the resolver, `DocReconcileMark`, Flow-B.
- `spike-p2panda/src/s2_membership_intercept.rs` + `src/evidence/s2.md` — gate-2: auth-layer `BlockingGroups` (implementable, no fork) vs the spaces-`AuthStore` Hard gap (the fork we avoid by not using that layer).
- `spike-p2panda/tests/l1_p2panda_auth.rs` — `GroupAction`/`Access` levels; `ManagerGroupsNotAllowed` (managerial actions are individuals-only).
