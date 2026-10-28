# The Membership record on the sparse Merkle tree

Design of `../requirements/2026-10-23-two-level-absence-proofs.md` and of the
move of the Membership record's store onto the `sparse-merkle-tree` unit,
agreed with the owner on 2026-10-23/24
(`docs/adr/2026-10-23-one-sparse-merkle-tree.md`). The store's addressing,
path-copying, write-once hashes and tree-changes walk become that unit's; this unit
keeps the member leaf, its hash domains, the domain rules and the absence
proof, which is now composed of that unit's proofs.

Superseded here: LLR-72p8bz, LLR-kdhd2v, LLR-pys2ek (hashing and the device
root); LLR-25tpdp, LLR-4xz255, LLR-4rju5r, LLR-dgzy7e, LLR-p2p8qy,
LLR-2dcnbp (the absence proof); and LLR-28ekrv (the mapping of person's
errors, for person's new variants). Retired: LLR-utp6x4, LLR-7jkcba. SDD-d9svdj
and SDD-57vaj4 are amended in place.

The store's other items keep their meaning at this unit's interface and are
realised through `sparse-merkle-tree` from this change on: LLR-zbe553 (a
Member's position is its MemberId's 256 bits, most significant first),
LLR-wm5hpc (the empty-subtree hashes), LLR-tk4qxu (a modification leaves the
record unchanged), LLR-n7nya3 (write-once hashes, `HashesNotCalculated`),
LLR-4n8zqx (the root depends on the member set alone) and LLR-8jttpb (the tree-changes
walk's order).

## The member leaf and its hashing

**LLR-t465mb**: The Membership record's leaf of the `sparse-merkle-tree`
tree at depth 256 is a `MemberLeaf` held with its device root, which is
computed when the leaf enters the tree: its identity is its `MemberId`, its
position the MemberId's 32 bytes, its identity hash the member-identity domain
hash of those bytes, and its content hash the member-content domain hash of
its canonical bytes without the MemberId — handle, Member-as-a-group key, name, surname
and device root, in that order and form. Its DevicePublicKeys enter the leaf
only through the device root. satisfies: REQ-sb2frw, REQ-65xrgh

**LLR-4rdqyn**: the hash interface provides eight separated domains, each
`blake3::keyed_hash` under a 32-byte key padded with `_`: member leaf
(`org-members::member-leaf________`), member node
(`org-members::member-node________`), member identity
(`org-members::member-identity____`), member content
(`org-members::member-content_____`), device leaf
(`org-members::device-leaf________`), device node
(`org-members::device-node________`), device identity
(`org-members::device-identity____`) and device content
(`org-members::device-content_____`); the same input hashed in two domains
yields two different values. The member domains are this unit's
`TreeHasher`; the device domains are its `DeviceRootHasher` for `person`.
supersedes: LLR-72p8bz
satisfies: derived

**LLR-uudst2**: a Member's device root is `person`'s `compute_device_root`,
the root of the device root tree of depth 4, under this unit's four device
domains, with
this unit's device empty sentinel unchanged
(`EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1`, LLR-jfj6pc).
supersedes: LLR-kdhd2v
satisfies: REQ-xdx2c2

**LLR-utk6uy**: a member holds at most `MAX_DEVICES` DevicePublicKeys, and
`MAX_DEVICES` is 8: `person`'s, which this unit re-exports, a domain rule held
below the device root tree's 16 positions (owner ruling 2026-10-23).
supersedes: LLR-pys2ek
satisfies: REQ-xdx2c2

## Membership operations

Under SDD-k5wa4n, the organisation trie and its operations.

**LLR-5jh6t3**: adding a DevicePublicKey to a Member, and building or
replacing a `MemberLeaf` with a device set, fails with
`OrgMembersError::DevicePositionTaken` when the key's position in the
Member's device root tree is taken by another of the Member's DevicePublicKeys,
and leaves the record unchanged. satisfies: REQ-2sz7j8

**LLR-w5yahk**: every error the `sparse-merkle-tree` unit returns to this
unit's own functions reaches their callers as an `OrgMembersError` variant
named for the check that refused: `HashesNotCalculated` and
`HashesAlreadyCalculated` keep their names, and a tree `Collision`, which a
Member's leaf cannot meet because its position is its MemberId and
`DuplicateId` is checked first, is `InvariantViolated`. No function, method,
type, re-export, enum variant or public field of this unit's own returns or
holds the tree's own error type; the mapping into `OrgMembersError` takes it
as its parameter. The `sparse-merkle-tree` unit's proof types, which this
unit's `AbsenceProof` carries and its callers depend on directly, are shared
by design (docs/adr/2026-10-23-one-sparse-merkle-tree.md).
satisfies: REQ-ds8ryr

**LLR-hbb4je**: `From<person::IdentityError> for OrgMembersError` maps every
`person::IdentityError` variant to an `OrgMembersError` variant, by an
exhaustive match with no wildcard arm, so that a variant `person` adds stops
org-members compiling until its mapping is chosen. `FieldTooLong { field, max }`,
`InvalidDeviceKey`, `InvalidPersonKey`, `DeviceSlotsFull`, `DuplicateDevice`,
`DeviceNotFound`, `DevicePositionTaken` and `DeviceStillHeld` map to
org-members' variants of the same names and fields. `KeyWithoutDevice`,
`MissingPersonKey`, `PersonKeyIsDeviceKey`, `PersonKeyNotRotated`,
`UnsupportedEncodingVersion(_)`, `PersonHashMismatch`, `DeviceRootMismatch`
and `ProofMalformed` map to `InvariantViolated`: only `person`'s Person
operations and its proofs under a Person hash produce them, org-members
calls none of those, so one reaching org-members is an internal invariant
violation.
supersedes: LLR-28ekrv
satisfies: REQ-ds8ryr

## The absence proof

SDD-57vaj4 is amended in place (its definition stays in
`2026-10-06-absence-proofs.md`).

**LLR-v6smww**: an `AbsenceProof` is either `Vacant`, holding the
`sparse-merkle-tree` unit's proof, at depth 256, that the MemberId is vacant,
or `Held`, holding the Member's leaf without its DevicePublicKeys (MemberId,
handle, Member-as-a-group key, name, surname and device root), an inclusion
proof at depth 256 for that leaf, and the tree's proof, at depth 4 under the
device root, that the DevicePublicKey is absent. It holds no DevicePublicKey
and no other Member's data.
supersedes: LLR-25tpdp
satisfies: REQ-yyuxh8, REQ-sb2frw

**LLR-bh53rr**: `OrgTrie::prove_absent(id, device)` returns a `Vacant` proof
when `id` holds no Member and a `Held` proof when the Member under `id` does
not hold `device`. It refuses with `HashesNotCalculated` when the
Membership record's hashes are not calculated, and with `DeviceStillHeld` when that Member holds `device`. A
DevicePublicKey held by a different Member does not refuse.
supersedes: LLR-4xz255
satisfies: REQ-sb2frw

**LLR-9g2kjg**: an `AbsenceProof`'s fields are private, and the only ways to
obtain one are `prove_absent` and parsing. Parsing, under the `serde`
feature, builds each field through its validating constructor (`Handle`,
`Name`, `Surname`, the Member-as-a-group key) and each inner proof through its own
bounded parse, and refuses with `AbsenceProofMalformed` whatever they refuse;
the refusal reaches a postcard caller as the format's custom error.
supersedes: LLR-4rju5r
satisfies: REQ-65xrgh, REQ-ds8ryr

**LLR-54gp2f**: `AbsenceProof::verify::<H>(root, id, device)` checks a `Vacant`
proof as the absence of `id` under `root`, and a `Held` proof as the inclusion
under `root`, at `id`'s position, of the leaf rebuilt from the proof's fields,
followed by the absence of `device` under the proof's device root; a Held
proof whose MemberId is not `id` does not resolve to `root` for `id`; when either
step does not resolve to its root it refuses with `AbsenceProofRootMismatch`,
before any other check.
supersedes: LLR-dgzy7e
satisfies: REQ-65xrgh

**LLR-25nmwc**: once both steps resolve, `verify` refuses with
`DeviceStillHeld` when the device step shows `device` held, refuses with
`AbsenceProofMalformed` when a Vacant proof's step shows `id` held, and
accepts otherwise.
supersedes: LLR-p2p8qy
satisfies: REQ-65xrgh

**LLR-v3cysh**: every refusal named in this file is, from the crate's own
functions (`prove_absent`, parsing, `verify`), its own `OrgMembersError`
variant, so the error says which check failed (`HashesNotCalculated`,
`DeviceStillHeld`, `DevicePositionTaken`, `AbsenceProofMalformed`,
`AbsenceProofRootMismatch`). No input to `prove_absent`, to parsing or to
`verify` reaches a panic.
supersedes: LLR-2dcnbp
satisfies: REQ-65xrgh, REQ-sb2frw, REQ-ds8ryr

## Retired

LLR-utp6x4 (the store's path walk returning 256 siblings) and LLR-7jkcba
(`MemberId::bit` and `DefaultHashes::at_level` returning an error) are
retired in their defining file: the path walk, the position bits and the
empty-subtree levels are `sparse-merkle-tree`'s, which bounds them the same
way, and this unit keeps no bit or level accessor of its own.
