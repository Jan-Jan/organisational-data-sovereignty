# DevicePublicKey and Member-as-a-group key validity on every route in — SDD-4yr9ge

Added by the change that switches org-members to `person`'s identity types
(owner, 2026-10-05). SDD-4yr9ge and SDD-d6x85b are defined in
`2026-09-17-decomposition.md`; this file refines SDD-4yr9ge for the two keys
that change now validates through `person`, the DevicePublicKey and the
Member-as-a-group key, and SDD-d6x85b for the device empty sentinel those keys
are hashed beside.

## SDD-4yr9ge — Member record and validation

**LLR-z954wj**: a DevicePublicKey that the software receives from outside the
process, whether on its own, in a set of DevicePublicKeys (`DeviceSlots`), in a
member record or in a change set, is decoded only through
`person::DevicePublicKey::parse`, and the public constructors (`parse`, and the
`TryFrom` impls from `[u8; 32]`, `&[u8; 32]` and `VerifyingKey` that delegate
to it) go through the same check, so bytes that do not decode as an Edwards
point, a small-order point (among them the four points of order 8 and
y = p − 1), a non-canonical encoding (among them y = p) or a point with a
torsion component are refused. The constructors report the refusal as
`person::IdentityError::InvalidDeviceKey`, which
`From<person::IdentityError> for OrgMembersError` maps to
`OrgMembersError::InvalidDeviceKey`; each decode path reports it as the
decoder's own error, raised through serde's custom error with the message
"invalid device public key" (postcard reports that as `SerdeDeCustom`, which
keeps no message).
satisfies: derived

**LLR-a645bx**: a Member-as-a-group key that the software receives from outside
the process, whether on its own, in a member record or in a change set, is
decoded only through `person::PersonPublicKey::parse`, and the public
constructor goes through the same check, so a Member-as-a-group key whose
u-coordinate is of small order (libsodium's `has_small_order` list: 0, 1, the
two points of order 8, p − 1) or is not canonical (at least p = 2^255 − 19) is
refused. The public constructors (`parse`, and the `TryFrom` impls from
`[u8; 32]` and `&[u8; 32]` that delegate to it) report the refusal as
`person::IdentityError::InvalidPersonKey`, which
`From<person::IdentityError> for OrgMembersError` maps to
`OrgMembersError::InvalidPersonKey`; each decode path reports it as the
decoder's own error, raised through serde's custom error with the message
"invalid person public key" (postcard reports that as `SerdeDeCustom`, which
keeps no message). A canonical u-coordinate of a point on the quadratic twist,
or of a mixed-order point, is accepted (owner, 2026-10-05). An X25519 key has
no off-curve bytes: every canonical u-coordinate is on the curve or on its
twist.
satisfies: derived

(Reworded 2026-10-05, independent review round 3: both LLRs said the public
constructor reports `InvalidDeviceKey` or `InvalidPersonKey`, but the
constructors are `person`'s and return `person::IdentityError`; org-members'
variants of those names arise only through the `From` mapping. LLR-z954wj
also gained the off-curve case, and both name the small-order and
non-canonical cases the tests feed. No behaviour changed.)

(Noted 2026-10-05, independent review round 3, finding-10: no test isolates
LLR-z954wj's non-canonical clause, and none can. Every non-canonical ed25519
encoding that `VerifyingKey::from_bytes` decodes is also of small order or
has a torsion component, so `person`'s canonicity check never decides alone,
and removing it from `DevicePublicKey::parse` changes no outcome: it is an
equivalent mutant. Measured by enumerating all forty non-canonical encodings
(y = p + k for k from 0 to 18 under both sign bits, and x = 0 with the sign
bit set): dalek decodes twenty-six, and each of those fails the torsion or
small-order check. The non-canonical cases in
`org-members/tests/device_key_decoding.rs` therefore show that such bytes are
refused, not which check refuses them. The clause stays: it states the rule
`person` implements, one encoding per key.)

REQ-shk82j asks that a device key set be sorted, free of duplicates and
within its bound, and says nothing of whether each key is a valid point; no
other org-members requirement asks for it either, so LLR-z954wj is derived
(review round 2, 2026-10-05). REQ-shk82j names the handle and the device key
set, not the Member-as-a-group key, and no other org-members requirement asks
for the key's curve to be checked, so LLR-a645bx is derived. Before the switch the key was an ed25519
`VerifyingKey` and decoded through `VerifyingKey::from_bytes`, which accepted
the all-zero u-coordinate (as y = 0, a point of order 4) and refused the twist
u-coordinate 2; the LLR records the behaviour that replaces it. Both LLRs are assessed in
`../risk/2026-10-05-device-and-member-key-validity.md`.

A decode path cannot return an `OrgMembersError`: org-members implements
`Deserialize` and does not choose the format, so the refusal is the format's
error, and what that error keeps of the message is the format's choice.
`org-members/tests/device_key_decoding.rs` and `member_key_decoding.rs` check
each decode path twice: through postcard, which must report `SerdeDeCustom`,
and through serde_json, whose error must be a data error that begins with the
message.

**LLR-st6j2r**: `DeviceSlots::parse(Vec<DevicePublicKey>)`, `person`'s
constructor that org-members re-exports, and the
`TryFrom<Vec<DevicePublicKey>>` impl that delegates to it, hold the keys
sorted, refuse more than `MAX_DEVICES` keys with
`person::IdentityError::DeviceSlotsFull` and a repeated key with
`DuplicateDevice`, which `From` maps to org-members' variants of the same
names, and accept the empty list.
satisfies: REQ-xdx2c2
supersedes: LLR-t3p9zk

LLR-st6j2r replaces master's LLR-t3p9zk (`2026-10-04-key-parse.md`), which
names `P2pDeviceSlots`, a type this change deletes (owner, 2026-10-05:
supersede and resolve). LLR-st6j2r is LLR-t3p9zk renamed: the behaviour is the
same, and only the error type changed, to `person`'s, mapped by `From`. The
tests that verified the old item carry both IDs
(`org-members/tests/newtypes.rs`).

*Amended 2026-10-05 (owner answer Q4 to docs/plans/2026-10-05-switch-trim.md).*
This section also held an item that superseded master's LLR-k6dhz7, the key
constructors, and narrowed it: decoding no longer accepts a small-order or
non-canonical key, so LLR-k6dhz7's last clause and its single `InvalidKey` no
longer held. No test could verify both texts, so that item was withdrawn
before merge and its text folded into LLR-k6dhz7 in place
(`2026-10-04-key-parse.md`). LLR-k6dhz7 is assessed in
`../risk/2026-10-05-device-and-member-key-validity.md`.

## SDD-d6x85b — Domain-separated hashing and the device sub-trie

**LLR-jfj6pc**: `Blake3Hasher::DEVICE_EMPTY_SENTINEL`, the bytes an unoccupied
device slot hashes in the device-leaf domain, is not the encoding of any
DevicePublicKey: it is not 32 bytes long, or `DevicePublicKey::parse` refuses
it.
satisfies: derived

An occupied slot hashes its DevicePublicKey's 32 bytes in the same domain, so
a sentinel that encoded a DevicePublicKey would make a record holding that key
as its largest hash as the same record without it. `person`'s
`DeviceTrieHasher` leaves the sentinel to the implementor and does not require
this (review round 3, finding 9); org-members' sentinel,
`EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1`, is 36 bytes, so it holds, and
LLR-jfj6pc makes it a requirement a test checks. Assessed in
`../risk/2026-10-05-device-and-member-key-validity.md`.
