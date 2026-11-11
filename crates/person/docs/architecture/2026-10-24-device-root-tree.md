# The device root tree, and proofs under a Person hash

Design of `../requirements/2026-10-23-person-absence-proofs.md`, and the
amendments to `2026-10-04-identity-types.md` and
`2026-10-06-person-definition.md` it needs, agreed with the owner on
2026-10-23/24 (`docs/adr/2026-10-23-one-sparse-merkle-tree.md`). The device
root becomes the root of the device root tree, a sparse Merkle tree of depth 4 built on the
`sparse-merkle-tree` unit; the V1 encoding keeps its name and is defined over
the new device root (owner ruling 2026-10-23); the bound on DevicePublicKeys
becomes 8.

SDD-7r833z and SDD-k5ee9x are amended in place.

## Device root

The device root item, SDD-7r833z, is amended in place (its definition stays
in `2026-10-04-identity-types.md`).

**LLR-9mjfjn**: `DeviceRootHasher` replaces `DeviceTrieHasher`. It extends
`sparse_merkle_tree::TreeHasher` (leaf, node and empty-leaf hashes, in
`crates/sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`) and adds `device_identity_hash(&[u8; 32]) -> NodeHash` and
`device_content_hash() -> NodeHash`, each under a domain the implementor chooses,
and an associated `DEVICE_EMPTY_SENTINEL: &'static [u8]` whose hash under the
leaf domain is `empty_leaf()`.
satisfies: REQ-mu3qgz, REQ-aj6x3n

**LLR-8y8tqr**: the device root tree of a `&[DevicePublicKey]` is built,
through `from_leaves` (`crates/sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`),
as a calculated `SparseMerkleTree` of depth `DEVICE_ROOT_DEPTH` (4) holding
one `DeviceLeaf` per key, and is refused with `DevicePositionTaken` when two
keys share a position; the tree itself is not exported.
`compute_device_root::<H>(&[DevicePublicKey])` returns that tree's root, and
`prove_device_absent_in::<H>(&[DevicePublicKey], &DevicePublicKey)` that
tree's `AbsenceProof<4>` for the key, failing with `DeviceStillHeld` when the
key is among them; both under the same refusal. A `DeviceLeaf`'s identity is its
DevicePublicKey, its identity hash `H::device_identity_hash` of the key's 32
bytes, its content hash `H::device_content_hash()`, and its position bytes
`device_position_bytes(key)` (LLR-exwx4n). The keys of a `DeviceSlots`, which
admits no two keys of one position (LLR-xsx6gg), are never refused.
satisfies: REQ-mu3qgz, REQ-aj6x3n

**LLR-exwx4n**: `device_position_bytes(&DevicePublicKey) -> [u8; 32]`, a
key's position bytes, is
`blake3::keyed_hash` of the key's 32 bytes under the 32-byte key
`persona::device-position________` (`persona::device-position` padded with
`_`); one domain for Person and Member device root trees alike (owner ruling
2026-10-24), of which the tree reads the first 4 bits. The device root tree's
`same_device_position(&DevicePublicKey, &DevicePublicKey) -> bool` compares
two keys' device positions: true exactly when the first 4 bits of their
position bytes are equal.
satisfies: REQ-9sh8qb

**LLR-vraz7n**: `DeviceSlots::position_taken(&candidate) -> bool` is true when
a key of the set other than `candidate` has the same first 4 bits of
`device_position_bytes`, and false otherwise; it leaves the set unchanged.
satisfies: REQ-9sh8qb

## Device slots

The identity value types item, SDD-k5ee9x, is amended in place: it also
traces REQ-4e23xr.

**LLR-xsx6gg**: `DeviceSlots::parse(Vec<DevicePublicKey>)` is the type's
parse for values built in the process (`TryFrom<Vec<DevicePublicKey>>`
delegates to it; decoding is LLR-sjrh7z's and LLR-pjf9qn's); it sorts its
input and fails with `DeviceSlotsFull` when given more than `MAX_DEVICES` (8)
keys, with `DuplicateDevice` when given a key twice, and with
`DevicePositionTaken` when two keys share a position (LLR-vraz7n); an empty
set is accepted. `add_device` fails with `DeviceSlotsFull` at the bound,
`DuplicateDevice` for a key already in the set and `DevicePositionTaken` when
`position_taken` holds for it; `remove_device` fails with `DeviceNotFound` for
a key not in the set; each returns a new set and leaves the receiver
unchanged. `devices` returns the keys in sorted order, `device_count` their
number, and `has_device` whether a given key is in the set.
satisfies: REQ-4szc22, REQ-4e23xr

`MAX_DEVICES` is 8 (owner ruling 2026-10-23), held below the tree's 16
positions so the tree is never full. The order of a set no longer changes its
device root — a sparse Merkle tree's root depends on its leaves alone — so the
sorted order is the set's canonical form for storing and decoding, and
nothing more.

**LLR-pjf9qn**: `DeviceSlots`' `Deserialize` impl rejects a sequence two of
whose keys share a position, as `parse` does, besides what LLR-sjrh7z
rejects.
satisfies: REQ-4e23xr, REQ-vxx8k3

**LLR-cyx9a3**: With the `serde` feature, a `DevicePublicKey` and a
`PersonPublicKey` serialise as their 32 bytes (a fixed-length byte array, no
length prefix), a `Name` and a `Surname` as the string of their NFC value, and
a `DeviceSlots` as the sequence of its keys in sorted order. Decoding a
`DeviceSlots` from anything other than a sequence fails with an error that
names the expected form, `at most 8 DevicePublicKeys in strictly increasing
order, at distinct device positions`.
satisfies: derived

## Person hash

The V1 encoding is redefined over the new device root.

**LLR-qwp8d7**: The `V1` encoding is, in order: the NFC name's byte length as a
4-byte little-endian `u32`, then its bytes; the same for the surname; one byte,
`0x00` when the `PersonPublicKey` is absent and `0x01` when present, followed in
the second case by its 32 bytes; then the 32-byte device root
`compute_device_root::<PersonDeviceRootHasher>(devices.devices())` (LLR-8y8tqr),
which makes `encode` return a `Result`.
`PersonDeviceRootHasher` is this unit's own `DeviceRootHasher` (LLR-9mjfjn): each
of its hashes is `blake3::keyed_hash` under a 32-byte key padded with `_` —
leaf `person::device-leaf_____________` over the identity hash and the content
hash concatenated, node `person::device-node_____________` over the two
children concatenated, identity `person::device-identity_________` over the
key's 32 bytes, content `person::device-content__________` over no bytes —
and its `DEVICE_EMPTY_SENTINEL` is the 31 ASCII bytes
`EMPTY_SENTINEL_PERSON_DEVICE_V1`. None of its domain keys or its sentinel is
org-members'.
satisfies: REQ-9m5pq2

## Proofs under a Person hash

**SDD-bx9mm6**: Person absence proof — the stateless proof that a
DevicePublicKey is not held under a Person hash, from its production to its
check.
traces: REQ-5kvqcf, REQ-uh2bzg

**LLR-985v9t**: a `PersonAbsenceProof` holds exactly: the `EncodingVersion`,
the `Name`, the `Surname`, the optional `PersonPublicKey`, the device root,
and a `sparse_merkle_tree::AbsenceProof<4>`; it holds no DevicePublicKey.
Its fields are private, and it is obtained only from
`prove_device_absent` or by parsing, which goes through each field's
validating constructor and the absence proof's bounded parse (in
`crates/sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`).
satisfies: REQ-5kvqcf, REQ-uh2bzg

**LLR-cve3d6**: `prove_device_absent(&Person, &DevicePublicKey)` returns a
`PersonAbsenceProof` under `V1` whose device root is the Person's and whose
absence proof is the device root tree's for that key; it fails with
`DeviceStillHeld` when the Person holds the key.
satisfies: REQ-5kvqcf

**LLR-z8h2w8**: `PersonAbsenceProof::verify(&PersonHash, &DevicePublicKey)`
recomputes the Person hash from the proof's fields and device root under its
encoding version and fails with `PersonHashMismatch` when it differs from the
given one, before any other check; it then verifies the absence proof against
the device root under `PersonDeviceRootHasher` and fails with
`DeviceRootMismatch` when it does not resolve to it and `DeviceStillHeld` when
it shows the key held.
satisfies: REQ-uh2bzg

**LLR-3dvfjb**: every refusal named in this file is its own `IdentityError`
variant (`DevicePositionTaken`, `DeviceStillHeld`, `PersonHashMismatch`,
`DeviceRootMismatch`, and `ProofMalformed` for a malformed absence proof),
named for the check that refused. No function, method, type, re-export,
enum variant or public field of this unit's own names the tree's
`TreeError` or its tree type, so no `TreeError` crosses this unit's own
functions. The `sparse-merkle-tree` unit's proof types, which this unit's
proofs carry and its callers depend on directly, are shared by design
(docs/adr/2026-10-23-one-sparse-merkle-tree.md). No input to these
functions reaches a panic.
satisfies: REQ-uh2bzg, REQ-4e23xr, REQ-bczz87

## NodeHash

`NodeHash` is the `sparse-merkle-tree` unit's, and this
unit re-exports it; this unit defines no `NodeHash` of its own.
