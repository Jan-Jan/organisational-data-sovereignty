# Context

The `person` unit holds an individual's identity types: names, device keys,
the PersonPublicKey, device slots, the device sub-trie and its root, and the
X25519 public-key validity check. On these types it defines the Person
definition — the record a Person shares with collaborators in their individual
capacity, apart from any Organisation — and computes and checks the hash that
commits to it.

## Language

<!--
Glossary of this unit's internal language. Definitions only — no
implementation details, no specs. Terms that cross a unit boundary live in the
root docs/CONTEXT.md. Format:

**Term**:
One or two sentences defining what it IS (not what it does).
_Avoid_: synonym1, synonym2
-->

**Name**:
An individual's given name as the unit stores it: in Unicode NFC, and bounded
in length.
_Avoid_: first name, forename, display name

**Surname**:
An individual's surname as the unit stores it: in Unicode NFC, and bounded in
length.
_Avoid_: last name, family name

**Device slots**:
The bounded, sorted, duplicate-free set of DevicePublicKeys one record holds.
_Avoid_: device list, device set, devices

**Device root**:
The value committing to a set of DevicePublicKeys under one domain.
_Avoid_: device hash, device sub-trie hash, device trie

**Device root tree**:
The sparse Merkle tree of depth 4 whose root is the device root.
_Avoid_: device trie, device sub-trie

**Device root hasher**:
The hash domains a device root tree is built under: a Tree hasher, the
domains of a DevicePublicKey's identity hash and of a device leaf's content
hash, and its Empty-leaf sentinel, each chosen by the device root tree's
consumer.
_Avoid_: device hasher, device trie hasher

**Device leaf**:
A leaf of a device root tree: a DevicePublicKey as its leaf identity, holding
no content besides it.
_Avoid_: device node, device entry

**Device position**:
A DevicePublicKey's Position in any device root tree: the first Depth bits of
its position bytes, under the one domain Person and Member device root trees
share.
_Avoid_: device slot, device index

**Empty-leaf sentinel**:
The bytes whose hash in a Tree hasher's leaf domain is that tree's Empty
leaf, chosen so that they encode no leaf the tree can hold.
_Avoid_: empty marker, null leaf, padding

**Person absence proof**:
A value showing, against one Person hash, that a DevicePublicKey is not held
in the Person definition it commits to: that definition's fields but its
DevicePublicKeys, its device root, and a Tree absence proof under that root.
_Avoid_: person proof, person exclusion proof, person-hash proof
