# Architecture — Person definition

Design of the `person` unit's Person definition capability, from the
design-architecture session of 2026-10-04. Serves the requirements in
`person/docs/requirements/2026-10-06-person-definition.md`
and the controls in `person/docs/risk/2026-10-06-person-hazards.md`.

## What this builds on

The change that moves the shared types (`docs/plans/2026-10-04-person-sequencing.md`,
change 1, merged as master f688a3c) brought `Name`, `Surname`,
`DevicePublicKey`, `PersonPublicKey`, `DeviceSlots`, the depth-2 device
sub-trie and the `DeviceTrieHasher` trait into this unit, with their own design
items in `person/docs/architecture/2026-10-04-identity-types.md`: SDD-k5ee9x
(the value types; LLR-ayu93n, LLR-7guspr, LLR-vs7etb, LLR-9p34cv, LLR-sjrh7z),
SDD-7r833z (the device sub-trie; LLR-4vsm8d, LLR-6ezhw7) and SDD-u9cddc (the
one error type, `IdentityError`, LLR-eeq89n, and the crate discipline,
LLR-64muuw). The items below use those types by name; they do not redefine
them. The device sub-trie's sentinel and domain keys are already parameters,
supplied by the implementor of `DeviceTrieHasher`, which LLR-edn55h relies on.
Every Person operation rejects with `IdentityError`, extended by this change
(LLR-3n3kxx).

## Decisions

- **Hash construction (owner, 2026-10-04):** a hand-written encoding, hashed
  with BLAKE3 keyed by a 32-byte domain key that names both the unit and the
  encoding version. Rejected: hashing a flat device-key list instead of the
  device sub-trie (gives up per-device proofs and diverges from the member
  side), and hashing postcard bytes (schema-fragile, and not one-to-one once
  names are NFC-normalised on decode — org-members' SOUP inventory, `postcard`
  row).
- **PersonPublicKey is an X25519 public key (owner, 2026-10-04):** grants are
  encoded against the key-agreement tree's root encryption key, which rotates
  with every commit — the rotation semantics REQ-bhez2u and the group-key ADR
  assume. DevicePublicKeys stay ed25519. A different key type is a new
  encoding version (REQ-wg7z4s), not a change to version 1.
- **The encoding version fixes the key types; v1 writes no type tags (owner,
  2026-10-04).** Every key in a v1 definition has its version's type, so a
  definition cannot mix device key types; a device whose key cannot be
  ed25519 (a secure-enclave key, say) is the trigger for a version that adds
  tags. For the same reason the on-chain entry stores the encoding version and
  no separate key-type field.

## Software items

All three items are class C, in one crate, one address space; no segregation
between them is claimed or needed. `person` depends on no unit.

**SDD-4r2x79**: Person definition — the validated `Person` value: a `Name`, a
`Surname`, an optional `PersonPublicKey`, and device slots of `DevicePublicKey`s;
one constructor enforces every rule a definition must satisfy, and it is the
only path to a `Person`, deserialisation included. traces: REQ-ht3x78,
REQ-7qgx2q, REQ-7ymek3, REQ-4szc22, REQ-tq4ms4

**LLR-4ku56h**: `Person` holds its device keys as a `DeviceSlots`, so a
`Person` holds at most `MAX_DEVICES` (4) `DevicePublicKey`s, sorted, with no key
twice: building the slots from more keys is refused with `DeviceSlotsFull` and
from a key given twice with `DuplicateDevice` (LLR-9p34cv), before any `Person`
exists, and decoding a `Person` decodes its slots through LLR-sjrh7z.
satisfies: REQ-4szc22, REQ-tq4ms4

**LLR-sjkmr6**: `Person::new` refuses, with `KeyWithoutDevice`, a definition
whose device slots are empty and whose `PersonPublicKey` is `Some`. It does so
by calling `check_group_key` (LLR-kbhc43).
satisfies: REQ-ht3x78

**LLR-kbhc43**: `Person::new` refuses a definition with one or more device keys
whose `PersonPublicKey` is `None` (`MissingPersonKey`) or whose 32 key bytes
equal the 32 bytes of any of its `DevicePublicKey`s (`PersonKeyIsDeviceKey`).
The comparison is of raw bytes only. It catches a literal copy of a device
key's bytes into the PersonPublicKey field; it does not catch a device's
ed25519 key pair reused through its X25519 (birational) image, nor one X25519
key presented under another of its up to 8 encodings with a small-order
component mixed in. Both are accepted residual risk (owner, 2026-10-06; the
risk file's assessment of REQ-7qgx2q). This rule and LLR-sjkmr6's are one
public function, `check_group_key(group_key: Option<&PersonPublicKey>, devices:
&DeviceSlots) -> Result<(), IdentityError>` in `person/src/group_key.rs`, which
`Person::new` calls and which org-members can call for the member-as-a-group
key (change 3 of `docs/plans/2026-10-04-person-sequencing.md`).
satisfies: REQ-7qgx2q

**LLR-63pkfc**: `Person`'s `Deserialize` implementation, compiled only with
the `serde` feature (`cfg(feature = "serde")`), decodes the four fields and
passes them to `Person::new`; `Name` and `Surname` decode through their
`parse` (LLR-ayu93n), the device slots through LLR-sjrh7z, each
`DevicePublicKey` through `DevicePublicKey::parse` (LLR-7guspr), and the
`PersonPublicKey` through `PersonPublicKey::parse` (LLR-vs7etb), which accepts
a canonical u-coordinate of a point on the quadratic twist (owner,
2026-10-05). No `Person` value is produced from bytes by any other route, and
a refusal surfaces through the deserializer's error with the message of the
`IdentityError` that `Person::new` or the field's `parse` returned; a missing
or unknown field is refused. With the same feature a `Person` serialises as a
struct of the four fields in this order — `name`, `surname`, `person_key` (an
option), `devices` — each in its own type's form (LLR-5za6mp).
satisfies: REQ-7ymek3

**LLR-dtwpr8**: The `Debug` form of a `Person` shows neither its Name nor its
Surname, whatever their value: it carries them in their own redacted `Debug`
form (LLR-fbqs2r). satisfies: derived

**SDD-v32mqh**: Person hash — encodes a `Person` under an `EncodingVersion`,
hashes the encoding, and checks a supplied hash against a definition; holds no
state between calls. traces: REQ-9m5pq2, REQ-7n4g8b, REQ-wg7z4s

**LLR-rde6tk**: `EncodingVersion` is a closed enum whose only variant is `V1`;
`EncodingVersion::try_from(u16)` maps 1 to `V1` and refuses every other value
with `IdentityError::UnsupportedEncodingVersion(value)`, before any encoding or
hashing.
satisfies: REQ-wg7z4s

**LLR-edn55h**: The `V1` encoding is, in order: the NFC name's byte length as a
4-byte little-endian `u32`, then its bytes; the same for the surname; one byte,
`0x00` when the `PersonPublicKey` is absent and `0x01` when present, followed in
the second case by its 32 bytes; then the 32-byte device root
`compute_device_root::<PersonDeviceHasher>(&devices)` (LLR-6ezhw7).
`PersonDeviceHasher` is this unit's own, Person-specific implementation of
`DeviceTrieHasher` (LLR-4vsm8d): `hash_device_leaf` is `blake3::keyed_hash`
under the leaf domain key `person::device-leaf` and `hash_device_node` is
`blake3::keyed_hash`, under the node domain key `person::device-node`, of the
two children's 32 bytes concatenated, each key padded with `_` to 32 bytes
(`person::device-leaf_____________`, `person::device-node_____________`); its
`DEVICE_EMPTY_SENTINEL` is the 31 ASCII bytes `EMPTY_SENTINEL_PERSON_DEVICE_V1`.
Neither its domain keys nor its sentinel are org-members'
(`org-members::device-leaf________`, `org-members::device-node________`,
`EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1`). satisfies: REQ-9m5pq2

**LLR-4ebtn4**: The `V1` hash is `blake3::keyed_hash` of the `V1` encoding under
the 32-byte key `person::definition::v1`, padded with `_` to 32 bytes
(`person::definition::v1__________`). A later version gets its own key, so no
two versions' hashes share a domain. `person_hash(person, version)` takes the
version as the `u16` published beside the hash, parses it with
`EncodingVersion::try_from` (LLR-rde6tk), and returns a `PersonHash`: any 32
bytes, unvalidated, built by `PersonHash::new` and `From<[u8; 32]>` and read
by `as_bytes`, whose `Debug` form is `PersonHash(` and the first four bytes in
lowercase hex and `..)`, and which with the `serde` feature serialises as its
32 bytes. satisfies: REQ-9m5pq2, REQ-wg7z4s

**LLR-tf45kx**: `verify(person, version, hash)` — `version` the `u16` encoding
version, `hash` a `PersonHash` — returns `Ok(true)` exactly when
`hash` equals the hash LLR-4ebtn4 computes for `person` under `version`,
`Ok(false)` otherwise, and `Err(IdentityError::UnsupportedEncodingVersion)` only when the
version is not implemented; it is a free function over its arguments, and the
item has no `static`, global, cache or interior-mutable state.
satisfies: REQ-7n4g8b, REQ-r4keha

**SDD-9hej83**: Successor check — decides whether one definition may follow
another. traces: REQ-bhez2u

**LLR-wqha8d**: `check_successor(previous, next)` refuses, with
`PersonKeyNotRotated`, a `next` whose sorted device keys differ from
`previous`'s while its `PersonPublicKey` equals `previous`'s (both `Some` and
equal), and returns `Ok(())` in every other case — a key change with unchanged
device keys included. satisfies: REQ-bhez2u

### Errors (SDD-u9cddc, amended)

The Person operations have no error type of their own: they return the unit's
one error type, `IdentityError` (SDD-u9cddc, LLR-eeq89n, `person/src/error.rs`),
amended in place for this change. The crate discipline that keeps them from
panicking is LLR-64muuw's, which covers the whole crate.

**LLR-3n3kxx**: `Person::new`, `check_successor`, `EncodingVersion::try_from`,
the hash and `verify` reject with a variant of `IdentityError`. The variants
this change adds, one per rule: `KeyWithoutDevice` (a PersonPublicKey with no
DevicePublicKey), `MissingPersonKey` (no PersonPublicKey with one or more
DevicePublicKeys), `PersonKeyIsDeviceKey` (a PersonPublicKey whose bytes equal
one of the DevicePublicKeys'), `PersonKeyNotRotated` (a successor whose
DevicePublicKeys changed and whose PersonPublicKey did not) and
`UnsupportedEncodingVersion` (an encoding version not implemented). A field's
own refusal keeps its existing variant — `FieldTooLong`, `InvalidDeviceKey`,
`InvalidPersonKey`, `DeviceSlotsFull`, `DuplicateDevice` — and there is no
decoding variant: decoding a `Person` fails through the deserializer's error,
with the `IdentityError` message (LLR-63pkfc). satisfies: REQ-r4keha
