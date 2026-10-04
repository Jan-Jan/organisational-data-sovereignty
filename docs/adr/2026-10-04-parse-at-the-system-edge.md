# ADR: Parse at the system's edge; every signature takes a newtype

- **Date:** 2026-10-04
- **Status:** accepted (2026-10-04)
- **Relates to:** REQ-t46uad (handle lookups), PR-hqwpg9 (secrets printed by
  `Debug`, fixed in the org-node follow-up)
- **Decision maker:** Jan-Jan (project owner), interviewed by `grill-requirements`

## Context

Invariants lived in free functions and in comments: `validate_handle` returned
a `String`, `MemberLeaf::with_handle` relied on "callers must validate first",
the name/surname NFC-and-length check was repeated in three places, and the key
index was keyed by naked `[u8; 32]`. Nothing in the types stopped an
unvalidated value from reaching the trie.

The first proposal was plain types (`&str`, `[u8; 32]`) at each crate's public
API and newtypes only inside. Rejected: org-members is a library whose caller
is org-node, so a crate-edge rule either hides the newtype from org-node (which
then stays stringly-typed or duplicates validation) or, once exported, forces
two functions per operation (`get_by_handle(&str)` and a typed twin) or a
`Result` that can never be `Err` when the caller already holds the newtype.
Rust has no overloading, and trait-based overloading was judged too opaque.

## Decision

1. **Parse at the system's edge.** Plain types appear only where data enters
   the system: user input, wire and storage decoding (serde, via `TryFrom`),
   and chain reads. They are parsed once into a newtype there; from then on
   every signature, public or internal, in every crate, takes and returns the
   newtype.
2. **Two kinds of newtype.** A *validated* type (`Handle`, `Name`, `Surname`,
   `P2pDeviceSlots`, keys built from bytes) has `parse(..) -> Result<Self, E>`
   and `TryFrom` delegating to it, and may canonicalize (NFC). A *tag* type
   carries no invariant and has no `Result`. The public tag types
   (`MemberId`, `NodeHash`, `RootHash`) have an infallible `new` and
   `From<[u8; 32]>`. The crate-internal tag types accept no raw bytes:
   `HeldKey` is built only `From` a member or device key, and `HandleSkeleton`
   only from a `&Handle` (`HandleSkeleton::of`). A tag type becomes a
   validated type when an invariant appears — e.g. hashes under a Poseidon
   hasher must be canonical field elements.
   *Amended 2026-10-04 after independent review: tag-type constructors
   corrected to match the code (`HeldKey` has no `new` or bytes `From`).*
   *Status 2026-10-04: `P2pDeviceSlots` still constructs with a fallible
   `new`, and the key types have no bytes constructor; both are brought under
   this decision in the org-node type-safety follow-up change (as
   `org-members/AGENTS.md` records).*
3. **Validated types are exported**, field private; `parse`/`TryFrom` is the
   only way in. The error type is the crate's own (`OrgMembersError` in
   org-members), not one enum per type.
4. **Accessors return the newtype.** `Debug` is redacted for personal data and
   secrets; `Display`/`as_str()` are for deliberate output. Secret types get no
   `Display`.

## Consequences

- Every public signature in org-members changes; callers (org-node, tests)
  parse before calling. Tests use small helpers.
- Handle lookups take `&Handle`; an invalid handle is reported by
  `Handle::parse`, which is how REQ-t46uad's invalid outcome is met.
- Wire format and hashing are unchanged: the newtypes serialize exactly as the
  plain types they wrap, and `canonical_bytes` is unchanged.
- org-node adopts the rule in a separate follow-up change (store, blobs,
  service, and a redacted secret type for PR-hqwpg9).
