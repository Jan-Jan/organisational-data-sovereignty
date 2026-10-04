# Design — parse at the system's edge

New low-level requirements for the change recorded in
`docs/adr/2026-10-04-parse-at-the-system-edge.md`. The five defined here
belong to software items defined in `2026-09-17-decomposition.md`: LLR-f3zrwd
to SDD-k5wa4n; and LLR-377ddr, LLR-c5tzyp, LLR-yz2gmc and LLR-v9evtu to
SDD-4yr9ge, which owns `types.rs`, where `NodeHash` and `RootHash` are defined.
The amendments that change
handle, name and surname construction to newtypes (LLR-xzqs9r, LLR-5w2jx8,
LLR-w5nkbu, LLR-4czn8t, LLR-68tka5, LLR-ub6dw9, LLR-mmst86, LLR-g6arcs) are
edited there, where those items are defined.

## SDD-k5wa4n — Membership operations

Handle lookups.

**LLR-f3zrwd**: `get_by_handle` takes a `&Handle` and returns the record of
the member holding exactly that handle, or `None` when no member holds it;
`contains_handle` takes a `&Handle` and returns whether a member holds it. A
string that is not a valid handle is refused with `InvalidHandle` by
`Handle::parse` (LLR-xzqs9r) before either is reached.
satisfies: REQ-t46uad

## SDD-4yr9ge — Member record and validation

Encoding stability, deliberate output of the personal fields, and tag types
for identifiers and hash values.

**LLR-377ddr**: the postcard encoding of a fixed member record and the
Blake3 root of a fixed member set are pinned values: wrapping fields in
newtypes changes neither the wire bytes nor the bytes a member leaf hashes.
satisfies: derived

**LLR-c5tzyp**: `Handle`, `Name` and `Surname` expose their stored NFC value,
unredacted, through `as_str()`, `Display`, `From<_> for String` and their serde
`Serialize` (which encodes the plain string — the wire form of the member
record), for deliberate output and replication; their `Debug` stays redacted
(LLR-4czn8t).
satisfies: derived

**LLR-yz2gmc**: `MemberId` is constructed from 32 bytes infallibly by `new`
and by `From<[u8; 32]>`, holding exactly those bytes; `as_bytes` returns them
unchanged.
satisfies: derived

**LLR-v9evtu**: `NodeHash` and `RootHash` are constructed from 32 bytes
infallibly by `new` and by `From<[u8; 32]>`, each holding exactly those bytes;
`as_bytes` returns them unchanged.
satisfies: derived

### Public API removals and renames

Recorded 2026-10-04 after independent review. This change removes or renames
these public items of the crate:

- `validate_handle` and `handle_skeleton` are removed, replaced by
  `Handle::parse` (LLR-xzqs9r) and the crate-internal `HandleSkeleton::of`
  (LLR-5w2jx8).
- `RootHash::from_bytes` is renamed `new` (LLR-v9evtu).
- `get_by_handle`, `contains_handle`, `update_handle`, `update_name_surname`
  and `MemberLeaf::new` take the validated types (`Handle`, `Name`,
  `Surname`) instead of strings; the record's accessors return them.

### Error order — five observable changes

Recorded 2026-10-04 after independent review; extended the same day after the
second review round, again after the third, and again after the fourth. Every
field-taking entry point now takes a `Handle`, a `Name` or a `Surname`, so a
field error surfaces at `Handle::parse`, `Name::parse` or `Surname::parse` at
the call site, before the operation runs any check of its own. These are the
changes identified by review, observable through error values; for input with
two defects, four entry points in this crate, and one call site in the
consumer org-node, report a different error than before:

- `MemberLeaf::new`: a field error now precedes the `EmptyDeviceList` check
  (LLR-paxj7b). An invalid handle with no devices reported `EmptyDeviceList`;
  it now reports `InvalidHandle`.
- `update_handle`: the operation used to look the id up first, then validate
  the handle. An unknown id with an invalid handle reported `IdNotFound`; it
  now reports `InvalidHandle`.
- `update_name_surname`: likewise, an unknown id with an oversized name or
  surname reported `IdNotFound`; it now reports `FieldTooLong`.
- Decoding a `MemberLeaf`: the record now derives `Deserialize`, so handle,
  name and surname are parsed field by field as they are decoded; previously
  all six fields were decoded as plain values, then validated. A record with
  an invalid handle, name or surname followed by a later defect (truncation, a
  bad key, a bad device list) reported the later structural error (for
  example postcard `DeserializeUnexpectedEnd`); it now reports the field's
  validation error (postcard `SerdeDeCustom`). Still refused either way.
- `trie_from_snapshots` in the consumer org-node (`org-node/src/service.rs`),
  which rebuilds a trie from member snapshots: the call to `MemberLeaf::new`
  now evaluates `Handle::parse`, `Name::parse` and `Surname::parse` in its
  arguments before the `device_keys?` argument that follows them. A snapshot
  with both an invalid handle and an invalid device key reported
  `Chain("bad device key: …")`; it now reports `Trie(InvalidHandle)` (an
  invalid name or surname likewise reports `Trie(FieldTooLong)`). A consequence
  in org-node, not a change to this crate's interface; reached when org-node
  loads member snapshots from its encrypted local store, and when a first
  admission decodes the record snapshot the admin sent
  (`first_admission_base`).

Every outcome is still a typed error. The operations themselves still refuse
an unknown id with `IdNotFound` for every well-typed call, so LLR-v3jqau holds
for every call the type system admits — an invalid field can no longer reach
the operation. The mbt conformance driver parses before calling, so
conformance does not exercise the old order. Assessed in
`../risk/2026-10-04-type-safety.md`.
