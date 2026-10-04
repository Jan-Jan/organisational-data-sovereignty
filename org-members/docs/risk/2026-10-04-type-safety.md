# Derived requirement assessment — handle lookups and the newtype refactor

Assesses the derived requirement written by the change recorded in
`docs/adr/2026-10-04-parse-at-the-system-edge.md`, and the refactor that change
carries (validated and tag newtypes in every signature). **No hazard and no
risk control is minted here**: the owner decided on 2026-10-04 (`analyze-risks`
interview) that the refactor's own failure modes are threats to existing
controls, answered by two obligations on the plan's tests rather than by new
controls.

## Derived requirements assessment

### REQ-t46uad — a handle query reports invalid, absent or held

assesses: REQ-t46uad

Affects an existing hazardous situation; introduces no new hazard; changes no
control's stated effectiveness.

**HAZ-jkc6tj (handles a person cannot tell apart) — slightly reduced.** A
lookup by handle is how an administrator resolves a name to a member. Before
this change `get_by_handle("Alice")` answered "not found", because an
uppercase handle can never be held; that reads as "no such member" when the
truth is "not a handle". REQ-t46uad makes the case explicit
(`Handle::parse` → `InvalidHandle`), removing one way to misread a lookup.
Lookups stay exact-match: a confusable query (an all-Cyrillic "аlісе" beside a
Latin `alice`) still answers "not found". That is unchanged, and the hazard's
existing residual-risk statement — the administrator's surface shows the stable
identifier, outside this codebase — covers it.

### LLR-377ddr — encoded bytes and root are pinned across the refactor

assesses: LLR-377ddr

Introduces no hazard; it is the carrier of the HAZ-y8h835 obligation below
(the golden test), written as an item so the test has something to verify.
It constrains the implementation and adds no runtime behaviour.

### LLR-c5tzyp — handle, name and surname values are output only deliberately

assesses: LLR-c5tzyp

Added 2026-10-04 after independent review. Introduces no new hazard; changes
no control's stated effectiveness.

`as_str()`, `Display` and `From<_> for String` hand the caller the personal
value it already holds — it parsed the value or read the record that carries
it — so they disclose nothing the caller could not already print. The
disclosure path LLR-4czn8t closes is the incidental one: a log line, a panic
message or an error report formatting a value with `{:?}`. That path goes
through `Debug`, which stays redacted on the standalone newtypes as on the
record. No hazard in the register is disclosure of the record's own personal
fields (the gap recorded under LLR-4czn8t in `2026-09-17-design-derived.md`
stands); for HAZ-jkc6tj the unredacted handle is what an administrator reads
to resolve a name, the output that pathway needs, and it carries no
uniqueness or confusability decision.

Serialisation is the fourth output (extended 2026-10-04 after the second
review round). The newtypes' `Serialize` encodes the plain string because the
wire form of the member record must carry the value: replication of lawful
changes (REQ-wx3wpv) cannot work without it. Exposure of that wire form is
governed outside this crate, by org-node's transport encryption and its
at-rest encryption of the store (org-node's requirement that the Persona
store be written only as ciphertext, in
`org-node/docs/requirements/2026-09-09-verify-and-commit.md`; cited by file
because org-members does not depend on org-node). A caller that hands a
record to a serde-based logger would print the value; that is a caller duty
and not a new hazard path this change opens — the pre-change `String` fields
serialised identically, byte for byte (LLR-377ddr).

### LLR-yz2gmc, LLR-v9evtu — tag types hold exactly the bytes they are built from

assesses: LLR-yz2gmc
assesses: LLR-v9evtu

Added 2026-10-04 after the second review round; split the same day after the
fourth, when the hash types moved to their own item (LLR-v9evtu) and
LLR-yz2gmc kept `MemberId`; both sit under SDD-4yr9ge, which owns `types.rs`
(corrected 2026-10-04 after the fifth review round: LLR-v9evtu was first
placed under SDD-d6x85b). Neither introduces a hazard or
changes a control's stated effectiveness. `MemberId` (LLR-yz2gmc) and
`NodeHash` and `RootHash` (LLR-v9evtu) are tag types: they carry no invariant
— any 32 bytes are a valid value — so an infallible constructor admits
nothing a fallible one would refuse. Each item pins only that construction and
`as_bytes` neither alter nor lose bytes. Where root bytes come from, and
whether they are an independent trusted root, remains the caller's
responsibility, as the crate README already states. Should a hasher with an
invariant on its outputs (a Poseidon hasher, whose outputs are field
elements) be adopted, the hash types become validated types, per the ADR.

### Amended items re-confirmed under the newtype mechanism

assesses: LLR-w5nkbu, LLR-4czn8t, LLR-ub6dw9, LLR-g6arcs

Added 2026-10-04 after independent review. These four items were reworded by
this change (validation moved from free functions into `parse` on a newtype);
their 2026-09-17 assessments in `2026-09-17-design-derived.md` still hold.

- **LLR-w5nkbu** — the 128-byte bounds now live in `Name::parse` and
  `Surname::parse` instead of a repeated check; the rejection is still the
  typed `FieldTooLong { field, max }`, so the HAZ-8suua9 / RC-c4truv
  assessment is unchanged.
- **LLR-4czn8t** — its scope now includes the `Debug` of a standalone
  `Handle`, `Name` or `Surname`, not only the record's. Redaction is widened,
  not narrowed; the risk is not raised, and the unminted-hazard finding stands.
- **LLR-ub6dw9** — the indexes are keyed by `Handle` and `HandleSkeleton`
  instead of strings; every operation still updates both, so RC-n2taat's
  dependence on them, and the assessment of a missed insert or removal, is
  unchanged.
- **LLR-g6arcs** — `update_name_surname` takes a `Name` and a `Surname`, so
  normalisation and bounds hold by construction rather than by a call inside
  the operation; the fields stay display-only and the HAZ-jkc6tj residual
  qualification is unchanged.

### REQ-h5ret5 amended — the identifier-character rule stated

assesses: REQ-h5ret5

Added 2026-10-04 after the fifth review round. By owner ruling, REQ-h5ret5
(defined in `../requirements/2026-08-31-org-membership.md`) and its refining
LLR-xzqs9r now state that a handle containing a character not permitted in
identifiers by UTS#39 (the Unicode General Security Profile), `-` excepted, is
rejected. `Handle::parse` has applied that rule since 2026-05-12; the
amendment records behaviour, it does not change it. **No new hazard.** The rule
is a restriction that already narrowed the handle namespace, in support of
RC-n2taat and so of HAZ-jkc6tj (handles a person cannot tell apart): it keeps
invisible, formatting and symbol characters out of handles, where they could
make two handles render alike. RC-n2taat's wording is amended to match.

## The refactor that carries it

Not a derived requirement; recorded because it rewrites the code behind
existing controls.

**HAZ-bmv7cy / RC-4apk6w (decoded record bypasses validation) — strengthened,
with an obligation.** Re-validation on decode becomes structural: a `Handle`,
`Name` or `Surname` cannot exist except through `parse`, and serde reaches it
through `try_from`. The refactor's own risk is the inverse — a slip such as
`derive(Deserialize)` on the newtype would open the bypass. **Obligation:** the
REQ-shk82j decode tests stay green unmodified in what they assert, and the plan
adds a test that each newtype field decoded from invalid bytes is refused.

**HAZ-y8h835 (views diverge) — a new cause, with an obligation.** If wrapping
changed `canonical_bytes` or the postcard encoding, two builds would compute
different roots for the same membership. **Obligation:** a golden test pinning
the encoded bytes and root hash of a fixed trie, written and green before the
refactor, and green unchanged after it.

**HAZ-8suua9 / RC-c4truv (panic on hostile input) — unaffected.** `parse`
reuses the existing, non-panicking validation; `deny(clippy::unwrap_used)`
still holds over the crate. Five observable behaviour changes, identified by
review and observable through error values — field errors now surface at
`Handle::parse`/`Name::parse`/`Surname::parse`, before the operation's own
checks: an invalid handle with no devices reports
`InvalidHandle` from `MemberLeaf::new`'s call site where it reported
`EmptyDeviceList`; an unknown id with an invalid handle reports
`InvalidHandle` where `update_handle` reported `IdNotFound`; an unknown id with
an oversized name or surname reports `FieldTooLong` where
`update_name_surname` reported `IdNotFound`; decoding a `MemberLeaf` whose
handle, name or surname is invalid and which carries a later defect
(truncation, a bad key, a bad device list) reports the field's validation
error (postcard `SerdeDeCustom`) where it reported the later structural error
(for example `DeserializeUnexpectedEnd`), because the record now derives
`Deserialize` and parses each field as it is decoded; and, in the consumer org-node,
`trie_from_snapshots` (`org-node/src/service.rs`) reports
`Trie(InvalidHandle)` where it reported `Chain("bad device key: …")` for a
member snapshot with both an invalid handle and an invalid device key
(recorded in `../architecture/2026-10-04-type-safety.md`) — affect no control:
each outcome is still a typed error and no input panics (RC-c4truv); the
org-node case still refuses the snapshot, only under a different error, and
no control depends on which of its two defects is named; the decode case
still refuses the record with a typed error and no panic, and RC-4apk6w's
re-validation on decode is unaffected — strengthened, since validation now
runs before the later fields are trusted; LLR-paxj7b still
refuses a zero-device member whose fields are valid; and LLR-v3jqau still
holds for every call the type system admits, since an invalid field can no
longer reach the operation and every well-typed call naming an unknown id is
refused with `IdNotFound`. The mbt conformance driver parses before calling,
so conformance does not exercise the old order.

**Key and device hazards — unaffected.** `HeldKey` renames the key-index key
only; `DuplicateKey` behaviour (LLR-v6gfc7) is unchanged.
