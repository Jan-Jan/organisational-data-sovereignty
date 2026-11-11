# Architecture — shared identity types

Design of the validated types `person` owns and org-members uses, from the
design-architecture session of 2026-10-04. Serves the requirements in
`crates/person/docs/requirements/2026-10-04-identity-types.md`.

The code these items describe is taken from org-members, not written afresh;
the behaviour is org-members' existing behaviour except where an item says
otherwise. org-members switches to it in the following change, when its own
design items for that code gain a note naming `person` as the provider.

## Decisions

- **Domains belong to the implementor of the hasher.** `person` defines the
  device sub-trie and the trait it hashes through; it ships no production
  implementation in this change. org-members' implementation keeps its existing
  device domain keys and empty-slot sentinel, which is what keeps every member
  hash byte-identical; the Person definition brings `person`'s own.
- **One crate, one address space, all class C.** No segregation is claimed or
  needed. `person` depends on no unit.

## Software items

**SDD-k5ee9x**: Identity value types — `Name`, `Surname`, `DevicePublicKey`,
`PersonPublicKey` and `DeviceSlots`, each constructible only through a
validating constructor, deserialisation included — and the X25519 validity
check `PersonPublicKey` applies, exported for other units' key types.
traces: REQ-r7mytp, REQ-q6xkna, REQ-3vqs9b, REQ-4szc22, REQ-tq4ms4, REQ-7gz72r, REQ-4e23xr

(Amended 2026-10-24, `worktree-generic-smt`: `DeviceSlots` refuses two keys
of one position, LLR-xsx6gg and LLR-pjf9qn in `2026-10-24-device-root-tree.md`.)

**LLR-ayu93n**: `Name::parse` and `Surname::parse` (and the `TryFrom` impls and
`serde(try_from = "String")` decoding that delegate to them) are the only
constructors of a `Name` and a `Surname`; each stores the NFC form, bounded
after normalisation at `MAX_NAME_LEN` (128) bytes for a `Name` and
`MAX_SURNAME_LEN` (128) bytes for a `Surname`, and a value exceeding its bound
is rejected as `FieldTooLong { field, max }` naming the field and the limit.
`as_str` returns the stored NFC value.
satisfies: REQ-r7mytp

**LLR-7guspr**: A `DevicePublicKey` is constructed only by
`DevicePublicKey::parse(&[u8; 32])`, the type's parse — its `Deserialize`
impl and `TryFrom<[u8; 32]>`, `TryFrom<&[u8; 32]>` and
`TryFrom<ed25519_dalek::VerifyingKey>` delegate to it, the last with the key's
32-byte encoding, so a key dalek has decoded meets the same rule. `parse`
accepts the bytes exactly when `ed25519_dalek::VerifyingKey::from_bytes`
accepts them, re-compressing the decoded point gives the same 32 bytes (the
encoding is canonical), the decoded point's `EdwardsPoint::is_torsion_free` is
true (it has no torsion component), and `VerifyingKey::is_weak` is false (the
point is not of small order). Every rejection is `InvalidDeviceKey`.
ed25519-dalek's `from_bytes` accepts small-order points, points with a torsion
component (the base point plus a small-order point) and non-canonical
encodings, established by tests rather than assumed (see this change's risk
file), which is why `person` checks all three itself. The torsion-free check
does not replace the small-order check: a small-order point other than the
identity is not torsion-free, but the identity is both torsion-free and of
small order, so only `is_weak` rejects it. The canonical-encoding check is
implied by the other two: every non-canonical encoding `from_bytes` accepts
decodes to a point of small order or with a torsion component, so no test
isolates it, and its correctness is verified by inspection. It is kept as
defence in depth against a change in dalek's decoding.
satisfies: REQ-q6xkna

**LLR-vs7etb**: `PersonPublicKey::parse(&[u8; 32])` is the only
constructor of a `PersonPublicKey`, the type's parse — its `Deserialize` impl
and `TryFrom<[u8; 32]>` and `TryFrom<&[u8; 32]>` delegate to it — and the
type contains the 32 bytes it was constructed from; it reads
the bytes as an X25519 u-coordinate and fails with `InvalidPersonKey` when the
top bit is set, when the value is not below 2^255 − 19, or when it is on the
small-order blocklist libsodium applies to X25519 public keys (the
implementation cites the list it encodes). It accepts a canonical
u-coordinate of a point on the quadratic twist (u = 2, 3 and 5, for example),
as RFC 7748 and libsodium do. It accepts the u-coordinate of a mixed-order
point (a prime-order point plus a non-identity point of small order), by the
owner's decision: it does not check that the point is torsion-free.
satisfies: REQ-3vqs9b

**LLR-sjrh7z**: `DeviceSlots`' `Deserialize` impl rejects a sequence longer than
`MAX_DEVICES` before storing a key beyond the bound, and rejects one that is
not strictly increasing, rather than sorting it. That the rejection comes
before a key beyond the bound is stored is verified by inspection: the
decoding visitor checks the length before each push, and no test can tell
that from a push followed by the check.
satisfies: REQ-4szc22, REQ-tq4ms4, REQ-vxx8k3

**LLR-m75m7u**: `person::x25519::is_valid_public_key(&[u8; 32]) -> bool` returns
`true` exactly when the bytes are canonical (top bit clear, value below
2^255 − 19) and not on the small-order blocklist of LLR-vs7etb, so it
returns `true` for a canonical u-coordinate of a point on the quadratic twist
and for the u-coordinate of a mixed-order point;
`PersonPublicKey::parse` accepts exactly the bytes this function accepts,
by calling it.
satisfies: REQ-7gz72r, REQ-3vqs9b

**LLR-fbqs2r**: The `Debug` form of a `Name` is `Name([REDACTED])` and of a
`Surname` is `Surname([REDACTED])`, whatever the value; `Display` writes the
stored NFC value unredacted, and `From<Name> for String` and
`From<Surname> for String` return it.
satisfies: derived

**LLR-a6krbh**: The `Debug` form of a `DevicePublicKey` or a `PersonPublicKey`
is the type name followed by the first four of its 32 bytes in lowercase hex
and `..`, as in `DevicePublicKey(3b6a27bc..)`; the `Debug` form of a
`DeviceSlots` is `DeviceSlots(n)`, `n` the number of keys, and names no key.
satisfies: derived

**LLR-78t363**: `DevicePublicKey` and `PersonPublicKey` compare equal, order
and hash by their 32 bytes: `Ord` is the lexicographic order of `as_bytes()`,
which is the sorted order `DeviceSlots` keeps (LLR-xsx6gg) and the slot order
of the device root rests on. `DevicePublicKey::verifying_key`
returns the decoded ed25519 key whose encoding `as_bytes` returns.
satisfies: derived

**SDD-7r833z**: Device root — the root of the device root tree, the depth-4
sparse Merkle tree that commits to a set of DevicePublicKeys, each at its
position, under hash domains the implementor chooses.
traces: REQ-mu3qgz, REQ-aj6x3n, REQ-9sh8qb

(Amended 2026-10-24, `worktree-generic-smt`: the fixed depth-2 tree of four
slots became a sparse Merkle tree of depth 4 built on the
`sparse-merkle-tree` unit, and the item gained the position and its collision
check; its LLRs are in `2026-10-24-device-root-tree.md`. Its statement was
restated as one responsibility on 2026-10-27, guideline review round 1, and
on 2026-10-28, round 6, to name the tree the device root tree.)

**SDD-u9cddc**: Errors — the error type every constructor in the unit returns,
and the no-panic discipline.
traces: REQ-vxx8k3, REQ-bczz87, REQ-r4keha

*Amended 2026-10-06 (Person definition):* `IdentityError` is also the error
type of every Person operation — `Person::new`, the successor check, the
encoding-version parse, the Person hash and its check — so the unit keeps one
error type; the variants the Person definition adds are LLR-3n3kxx's, and
REQ-r4keha is traced here for them.

**LLR-eeq89n**: The in-process constructors above — each type's `parse`, the
`TryFrom` impls that delegate to it, and `DeviceSlots::add_device` and
`remove_device` — reject with a variant of one `IdentityError` enum:
`FieldTooLong { field, max }`, `InvalidDeviceKey`, `InvalidPersonKey`,
`DeviceSlotsFull`, `DuplicateDevice`, `DeviceNotFound`. Decoding rejects
through the deserializer's own error type, with a message that names the rule:
the `IdentityError` message for a key or a name, `device slots exceed
MAX_DEVICES` or `device slots must be strictly increasing (sorted, no
duplicates)` for a `DeviceSlots`, and the expected form of LLR-cyx9a3 for a
`DeviceSlots` decoded from anything other than a sequence.
satisfies: REQ-bczz87

*Amended 2026-10-06 (Person definition):* the Person definition adds five
variants to `IdentityError` — `KeyWithoutDevice`, `MissingPersonKey`,
`PersonKeyIsDeviceKey`, `PersonKeyNotRotated` and
`UnsupportedEncodingVersion` — stated, with the operations that return them,
in LLR-3n3kxx, whose tests verify them. The variants above are unchanged, and
decoding a `Person` still rejects through the deserializer's error, with the
`IdentityError` message.

**LLR-64muuw**: The crate is `#![no_std]` with `alloc`, builds under the
workspace lints that deny `unwrap`, `expect` and `panic`, and denies
`clippy::indexing_slicing` crate-wide, so it indexes no slice or array with a
value derived from input.
satisfies: REQ-vxx8k3

## Items with no invalid input

REQ-mu3qgz, REQ-aj6x3n, LLR-fbqs2r, LLR-a6krbh and LLR-78t363 take only
already-validated types (a `DeviceSlots`, a `Name`, a key), so there is no
invalid input to give them.
Their tests cover the normal case and the boundaries — an empty and a full
`DeviceSlots`, an empty name — and none is an abnormal-input test.

LLR-64muuw is a build property of the crate, not an operation on input. Its
robustness evidence is the property tests that no input makes a constructor or
a decoder panic (`crates/person/tests/no_panic.rs`) and the lint configuration it
states, which the crate-discipline test reads.
