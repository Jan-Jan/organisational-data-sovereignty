# Device and Member-as-a-group key validity — assessment of the derived requirements

The change that switches org-members to `person`'s identity types adds four
derived LLRs, three under SDD-4yr9ge and one under SDD-d6x85b
(`../architecture/2026-10-05-device-key-validity.md`). None has a parent
requirement that states it, so each is assessed here against the existing
register (`2026-09-02-membership-hazards.md`). The
narrowing the switch makes to the key-uniqueness check is an amendment of that
check's own assessment and stays in `2026-10-03-key-uniqueness.md`, "After the
switch to `person`'s key types".

**LLR-z954wj, derived (2026-10-05).** A DevicePublicKey is decoded and
constructed only through `person::DevicePublicKey::parse`, which refuses bytes
off the curve, a small-order point, a non-canonical encoding and a point with
a torsion component (`person::IdentityError::InvalidDeviceKey` from the
constructors, `OrgMembersError::InvalidDeviceKey` through `From`; serde's
custom error, "invalid device public key", on decode). Before the switch a
device key was an ed25519 `VerifyingKey` decoded through
`VerifyingKey::from_bytes`, which refused bytes off the curve and accepted the
other three (`org-members/tests/device_key_decoding.rs`, PR-b7khyw).
No hazard impact, and the change reduces exposure on two existing hazards:

- HAZ-bmv7cy, a serialised record introducing a member the software would have
  refused. RC-4apk6w re-validates the order, duplicates and bound of a
  member's set of DevicePublicKeys on decode; it compared keys by their bytes, so one point under two
  encodings passed as two devices. With one encoding per key, the
  duplicate-free rule RC-4apk6w enforces holds for points as well as bytes.
- HAZ-s39gbh, a removed device keeping access. The "every key held once" rule
  (`2026-10-03-key-uniqueness.md`) compares bytes; a second encoding of a
  removed device's key, or of a held one, escaped it. One encoding per key
  closes that for DevicePublicKeys.

A small-order device key has no secret behind it, and a torsioned one lets
cofactored and cofactorless verification disagree about one signature
(`person/docs/risk/2026-10-04-identity-types.md`); refusing both means no
record names a device whose key nobody holds or whose signatures two verifiers
judge differently. org-members verifies no signature itself. The one new
failure is availability, of the kind this register already accepts: a record
or change set carrying such a key, or an operation given one, is refused where
it used to succeed. A key generated as a secret scalar times the base point is
never refused, so only crafted or corrupted keys reach the refusal. There are
no adopters with stored records to migrate.
assesses: LLR-z954wj

**LLR-a645bx, derived (2026-10-05).** The Member-as-a-group key is decoded
and constructed only through `person::PersonPublicKey::parse`, which refuses a
small-order or non-canonical X25519 u-coordinate
(`person::IdentityError::InvalidPersonKey` from the constructors,
`OrgMembersError::InvalidPersonKey` through `From`; serde's custom error,
"invalid person public key", on decode) and accepts twist and mixed-order
u-coordinates. Before the switch the key was an
ed25519 `VerifyingKey`, which accepted the all-zero u-coordinate and refused
the twist u-coordinate 2 (`org-members/tests/member_key_decoding.rs`, run
against master 05f6f04). No hazard impact. org-members does not authenticate a
Member-as-a-group key and no control in this register validates its curve:
which key the group holds is the administrator's and the group's CGKA's
choice. The refusal reduces exposure rather than adding it: a small-order key
has no usable secret behind it, and key agreement with it yields an output
any party can compute, so a grant encoded against it would be readable by
anyone who reads the grant. Refusing non-canonical u-coordinates gives each
key below p one byte string, which narrows the byte comparison's blind spot to
the mixed-order case accepted in `2026-10-03-key-uniqueness.md`. Twist keys
stay accepted, as RFC 7748 and libsodium accept them (owner, 2026-10-05); that
is `person`'s accepted rule (`person/docs/risk/2026-10-04-identity-types.md`,
REQ-3vqs9b). The one new failure is an availability one, of the kind this
register already accepts: a record or change set carrying such a key, or an
operation given one, is refused where it used to succeed. A record written
before the switch holds ed25519 Member-as-a-group keys, whose bytes were never meant as
X25519 u-coordinates; there are no adopters with stored records to migrate.
assesses: LLR-a645bx

**Residual: a record written before the switch (review round 3, 2026-10-05).**
About half of all ed25519 Member-as-a-group keys have their top bit clear and are below
p, so they parse as X25519 u-coordinates; `PersonPublicKey` accepts them. The
member-leaf hash domain and the leaf's canonical bytes carry no version tag,
and the Member-as-a-group key's bytes are hashed unchanged, so such a record decodes,
hashes to the root it was committed under and verifies against it. The key it
holds is then unusable: its bytes are an Edwards y-encoding, and nobody holds
the X25519 secret of the u-coordinate they spell, so a grant encoded against
it can be read by no one. That is availability, not disclosure: the member is
locked out of what is granted to them until their key is rotated, and no one
else gains access. The other half are refused on decode, also availability.
Accepted, with no control: there are no adopters with stored records, and a
record from before the switch is not expected to reach a node after it. A
format version in the leaf would close it, and belongs to the change that
next changes the record format (the Member key rules change,
`docs/plans/2026-10-04-person-sequencing.md` on `worktree-person-requirements`).

**LLR-k6dhz7, derived, amended in place (2026-10-05).** The 32-byte
constructors of a DevicePublicKey and a PersonPublicKey accept exactly what
decoding accepts and report `person::IdentityError` (`OrgMembersError` through
`From`). Master's assessment of the item (`2026-10-04-key-parse.md`) rests on
the same argument: no value enters by a constructor that could not enter by
decoding, so the constructors add no route in. What each route accepts is
LLR-z954wj's and LLR-a645bx's, assessed above. No hazard impact.
assesses: LLR-k6dhz7

**LLR-jfj6pc, derived (2026-10-05).** The device empty sentinel encodes no
DevicePublicKey. Were it one, a record holding that key as its largest
DevicePublicKey would hash as the same record without it: a device present in
the record and absent from the root, the case
`2026-09-17-design-derived.md` files under HAZ-y8h835 ("a member key set the
committed root does not describe"), here reachable by anyone who can enrol a
DevicePublicKey equal to the sentinel. org-members' sentinel is 36 bytes, so
no DevicePublicKey equals it, and the requirement and its test keep a later
change from choosing a 32-byte one. It reduces exposure on HAZ-y8h835 and adds
no failure. No re-scoring.
assesses: LLR-jfj6pc

**Follow-up for `person` (not in this change's scope).** `person`'s
`DeviceTrieHasher` lets the implementor choose the sentinel and does not
require it to lie outside the DevicePublicKey encodings, so another unit
hashing device sub-trie roots through `person` could choose one that collides.
`person` is merged and signed, and this change does not edit it; the
constraint belongs in `person`'s own requirements for `DeviceTrieHasher`, in
a later change to that unit. Until then each implementor carries it, as
org-members does with LLR-jfj6pc.

The same gap is open inside org-members (noted 2026-10-05 by review round 3,
finding-11). LLR-jfj6pc constrains `Blake3Hasher` alone. `TrieHasher`
(`org-members/src/hasher.rs`) is public and extends `DeviceTrieHasher`, so a
hasher written outside this crate, such as the planned Poseidon one, chooses
its own sentinel and nothing in org-members requires it to encode no
DevicePublicKey. Such a hasher carries the constraint itself until the
follow-up above states it in `person` for every `DeviceTrieHasher`, which then
covers every `TrieHasher` too.
