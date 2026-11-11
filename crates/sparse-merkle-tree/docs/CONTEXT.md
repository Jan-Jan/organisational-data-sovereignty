# Context

The `sparse-merkle-tree` unit holds the one sparse Merkle tree that the
Membership record and the device root tree are built on, and the absence proofs
under its root. It knows nothing of Members, Persons or DevicePublicKeys: each
consumer chooses the leaf, the hash domains and the depth.

## Language

<!--
Glossary of this unit's internal language. Definitions only — no
implementation details, no specs. Terms that cross a unit boundary live in the
root docs/CONTEXT.md. Format:

**Term**:
One or two sentences defining what it IS (not what it does).
_Avoid_: synonym1, synonym2
-->

**Sparse Merkle tree**:
A binary hash tree of fixed depth with one position for every possible value
of that depth's width, almost all of them empty, committing to the leaf
identities it holds, and their leaves, by a single root.
_Avoid_: SMT (in prose), trie, merkle trie, sparse merkle trie

**Depth**:
The number of levels from the root to a leaf of a sparse Merkle tree; a tree
of depth d has 2^d positions.
_Avoid_: size, height, bit width

**Position**:
The one place in a sparse Merkle tree a leaf may be held: the first Depth bits
of the leaf identity's position bytes.
_Avoid_: slot, index, path (for the place itself)

**Position bytes**:
The 32 bytes a leaf identity maps to; the first Depth bits of them are its
Position.
_Avoid_: position (for the bytes), path, key hash

**Leaf**:
What a sparse Merkle tree holds at a position, chosen by the tree's consumer:
a leaf identity and the content its content hash covers, such as a Member's
leaf or a device root tree's leaf.
_Avoid_: entry, record, item (for the tree's side)

**Leaf identity**:
What tells one leaf of a sparse Merkle tree from another and fixes its
position, chosen by the tree's consumer: a MemberId for a Member's leaf, a
DevicePublicKey for a device root tree's leaf.
_Avoid_: key (for the tree's side), leaf key, leaf id

**Collision**:
Two leaves of different identities that have the same position in a sparse
Merkle tree.
_Avoid_: clash, conflict

**Identity hash**:
The hash of a leaf identity, under a domain the tree's consumer chooses; it
stands for the identity wherever the tree commits to a leaf.
_Avoid_: key hash, id hash

**Content hash**:
The hash of everything a leaf holds besides its identity, under a domain the
tree's consumer chooses.
_Avoid_: value hash, data hash, payload hash

**Tree hasher**:
The set of hash domains a sparse Merkle tree is built under, chosen by its
consumer: leaf, node and empty leaf, each separated from the others and from
every other consumer's.
_Avoid_: hash function, digest (for the set), hasher (alone, in prose)

**Empty-subtree hash**:
The hash of a subtree of a given level that holds no leaf; one value per
level for a given set of hash domains.
_Avoid_: zero hash, empty node hash

**Proof ending**:
What a Tree absence proof shows at its Position: Empty, no leaf held there, or
Occupied, a leaf of another identity held there, given by its identity hash
and content hash.
_Avoid_: terminal, end leaf, tail

**Calculated**:
A sparse Merkle tree none of whose node hashes is unset.
_Avoid_: hashed, finalised, committed (for the tree)

**Tree changes**:
Between two sparse Merkle trees: the leaf identities the first holds and the
second does not, and the upserted leaves.
_Avoid_: diff, delta, patch (for the tree's side)

**Upserted leaf**:
A leaf the second of two sparse Merkle trees holds that the first does not
hold as it is: one added or one replaced.
_Avoid_: changed leaf, updated leaf, new leaf

**Level**:
The height of a node above the leaves of a sparse Merkle tree: a leaf's
position is at level 0 and the root at level Depth.
_Avoid_: layer, depth (for one node)

**Node hash**:
The 32-byte hash of a leaf, of a node over two subtrees, or of the root of a
sparse Merkle tree.
_Avoid_: digest, node value

**Empty leaf**:
The hash a sparse Merkle tree gives a position that holds no leaf: the
empty-subtree hash of level 0.
_Avoid_: zero leaf, null leaf, empty slot

**Sibling**:
The node hash beside a proof's node at one level on the way from a position
to the root, with which that node is hashed to give the node a level up.
_Avoid_: neighbour, proof node, audit node

**Proof map**:
The Depth bits of a proof marking the levels whose sibling is not that
level's empty-subtree hash; a proof carries only the marked siblings.
_Avoid_: bitmap, mask, bitfield

**Tree absence proof**:
An absence proof that a leaf identity is not held under the root of a sparse
Merkle tree: a value showing, against that root, that the leaf identity is
not held there; it does not show that the leaf identity was never held.
_Avoid_: exclusion proof, non-inclusion proof, non-membership proof, leaf
absence proof
