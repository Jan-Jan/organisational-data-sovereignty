# The sparse Merkle tree — design

Design of the requirements in
`../requirements/2026-10-23-sparse-merkle-tree.md`, under the risk controls of
`../risk/2026-10-23-sparse-merkle-tree.md` (RC-5zyczk, RC-gy5f5r) and the
decision `docs/adr/2026-10-23-one-sparse-merkle-tree.md`. Sections 1 to 3 of
the design were agreed with the owner on 2026-10-23/24: three software items,
an immutable tree with compact single-leaf subtrees whose root is the classic
fixed-depth tree's, a leaf hash split into an identity hash and a content
hash, absence and inclusion proofs typed by their depth, and a crate that
depends on no hash function.

The crate is `sparse-merkle-tree`. All three items are class C, as the unit
is; they run in the consumer's process, and none of them performs I/O.

## Leaf and hasher interfaces

**SDD-x43d2z**: Consumer interfaces — the types through which a consumer
describes its leaves, under its own hash domains, to the tree.
traces: REQ-vg9wq4, REQ-ymgg6b, REQ-s3unp5

**LLR-jd4phg**: `NodeHash` wraps any 32 bytes, unvalidated: `NodeHash::new`
and `From<[u8; 32]>` construct it, `as_bytes` returns the bytes unchanged, its
`Debug` form is `NodeHash(` and the first four bytes in lowercase hex and
`..)`, and with the `serde` feature it serialises as the 32 bytes. It is the
one `NodeHash` of the workspace; `person` and `org-members` re-export it
(`docs/adr/2026-10-04-parse-at-the-system-edge.md`).
satisfies: derived

**LLR-znpbf5**: `TreeHasher` provides `leaf(&NodeHash, &NodeHash) -> NodeHash`
(an identity hash and a content hash), `node(&NodeHash, &NodeHash) ->
NodeHash` (a left and a right child) and `empty_leaf() -> NodeHash`, each a
pure function the consumer implements under its own domains; the crate
provides no implementation.
satisfies: REQ-vg9wq4, REQ-ymgg6b

**LLR-t3e8pp**: `Leaf<H: TreeHasher>` has an associated `Identity` type and
provides `identity(&self) -> &Self::Identity`, `position_bytes(&Self::Identity)
-> [u8; 32]` (the leaf's position bytes), `identity_hash(&Self::Identity) -> NodeHash` and
`content_hash(&self) -> NodeHash`. Two leaves are the same leaf for the tree
when their identities are equal.
satisfies: REQ-ymgg6b, REQ-s3unp5

**LLR-9z6dc9**: a leaf's hash is `H::leaf(identity_hash(identity),
content_hash(leaf))`, and nothing else of a leaf enters the tree.
satisfies: REQ-ymgg6b, REQ-s3unp5

The identity hash is what lets an absence proof show that a position holds a
*different* leaf without carrying that leaf (LLR-m5c3vw): the verifier hashes
the identity it asks about and compares. What a consumer's identity and
content are is the consumer's: for a device root tree, the DevicePublicKey and
nothing; for the Membership record, the MemberId and the rest of the member
record.

## Tree store

**SDD-rh9u6z**: Tree store — the immutable sparse Merkle tree of a depth fixed
at build, which holds a consumer's leaves without interpreting them.
traces: REQ-vg9wq4, REQ-6cej5z, REQ-uy4euz, REQ-3szczq, REQ-dyjq33

**LLR-ggu9bh**: the tree is `SparseMerkleTree<L, H, const DEPTH: usize>`, and a
`DEPTH` of 0 or above 256 fails to compile (a constant assertion evaluated
when the type is used).
satisfies: REQ-vg9wq4

**LLR-mm45ry**: a leaf's position is the first `DEPTH` bits of its position
bytes `L::position_bytes(identity)`, bit 0 being the most significant bit of the first
byte; at each level, bit 0 says left and bit 1 right. The store takes the bits
from an iterator over exactly `DEPTH` bits, which cannot go out of range.
satisfies: REQ-vg9wq4, REQ-dyjq33

**LLR-yuyn8f**: the hash of an empty subtree at each of the `DEPTH + 1` levels
is computed once when an empty tree is made, shared by every tree derived
from it, and computed again once per proof verification: level 0 is `H::empty_leaf()`, level `i` is
`H::node` of level `i - 1` with itself. Reading a level above `DEPTH`
(`SparseMerkleTree::empty_subtree_hash(level)`) returns
`TreeError::LevelOutOfRange`, without indexing out of bounds.
satisfies: derived

**LLR-kep2ju**: a subtree holding exactly one leaf is stored as one node at
the subtree's top level, and its hash is the leaf's hash folded upwards with
that level's empty-subtree hash as the sibling at every level from the leaf's
up to the node's, taking left or right from the leaf's position bits; a
subtree holding no leaf is the empty node of its level. A tree's root is
therefore the root of the classic tree in which every node is stored.
satisfies: derived

**LLR-8uem8c**: `insert` and `remove` take `&self` and return a new tree,
sharing every unchanged node with the receiver, which is observably
unchanged.
satisfies: REQ-6cej5z

**LLR-ycf27m**: `insert(leaf)` places the leaf at its position; when that
position holds a leaf of a different identity it returns
`TreeError::Collision` and no tree, and when it holds a leaf of the same
identity it replaces it. `remove(identity)` returns `TreeError::LeafNotHeld`
when no leaf of that identity is held.
satisfies: REQ-6cej5z

**LLR-3y5pq6**: each node's hash is a write-once cell. `calculate()` fills
every unset cell bottom-up and returns the calculated tree, or
`TreeError::HashesAlreadyCalculated` when no cell is unset; `root()` returns
`TreeError::HashesNotCalculated` while any cell is unset.
satisfies: REQ-uy4euz, REQ-3szczq

**LLR-3zffc2**: the root of a tree is determined by its set of leaves alone,
not by the order they were inserted or removed in.
satisfies: derived

**LLR-9qncuc**: `from_leaves(leaves)` inserts the leaves, in the order
given, into an empty tree and returns that tree calculated, or the first
`TreeError::Collision` an insertion returns; `root_of(leaves)` returns the
root of the tree `from_leaves` builds, under the same refusal.
satisfies: REQ-vg9wq4, REQ-6cej5z

**LLR-639mz9**: `get(identity)` returns the held leaf of that identity, or
none; `contains(identity)` whether one is held; `leaves()` every held leaf
in strictly increasing position order; `is_calculated()` whether no node
hash is unset. None of them changes the tree.
satisfies: REQ-vg9wq4

**LLR-u2r4vj**: `changes_between(before, after)` walks both trees,
calculated or not, descending left before right, skipping every pair of
subtrees that are one node or whose hashes are both set and equal, and
lowering a compact leaf one level where it faces an internal node; it returns
the tree changes between them, the identities removed and the upserted
leaves, each in strictly increasing position order.
satisfies: derived

**LLR-byz55h**: the crate is `#![no_std]` with `alloc`, performs no I/O, and
builds under the workspace lints that deny `unwrap`, `expect` and `panic`
together with `clippy::indexing_slicing` crate-wide.
satisfies: REQ-dyjq33

**LLR-c4eunm**: the crate's normal dependencies are `spin`, for the
write-once cell, and, under its `serde` feature, `serde`; it depends on no
hash function and on no other unit of the workspace.
satisfies: derived

## Proofs

**SDD-ur6uvu**: Proofs — the fixed-depth proof of a leaf's absence or
inclusion, from its production on a calculated tree to its check against a
root.
traces: REQ-uy4euz, REQ-ymgg6b, REQ-s3unp5, REQ-3szczq, REQ-fmx9e7, REQ-dyjq33

**LLR-m5c3vw**: an `AbsenceProof<DEPTH>` holds exactly three things: a map of
`DEPTH` bits marking which siblings are not their level's empty-subtree hash,
the hashes of those siblings in path order from the leaf's level up, and an
ending that is either `Empty` or `Occupied { identity_hash, content_hash }`.
satisfies: REQ-s3unp5

**LLR-tz3nfz**: an `InclusionProof<DEPTH>` holds exactly the map and the
sibling hashes, in the same form as an absence proof; the leaf it is for is
given to the verifier, not carried.
satisfies: REQ-3szczq, REQ-fmx9e7

**LLR-5tww69**: `prove_absent(identity)` on a tree returns an
`AbsenceProof<DEPTH>` whose ending is `Empty` when the identity's position is
in an empty subtree, and `Occupied` with the occupant's identity hash and
content hash when it holds a leaf of a different identity. It returns
`TreeError::HashesNotCalculated` when the tree is not calculated and
`TreeError::LeafHeld` when a leaf of that identity is held.
satisfies: REQ-uy4euz, REQ-s3unp5

**LLR-88rm99**: `prove_included(identity)` returns the held leaf and an
`InclusionProof<DEPTH>`, or `TreeError::HashesNotCalculated` when the tree is
not calculated and `TreeError::LeafNotHeld` when no leaf of that identity is
held.
satisfies: REQ-3szczq

**LLR-77zy97**: a proof's fields are private, and the only ways to obtain one
are its `prove_` function and parsing. `from_parts` returns
`TreeError::ProofMalformed` when the map is not `DEPTH` bits or the number of
sibling hashes is not the number of set bits in it; `Deserialize`, under the
`serde` feature, refuses the same inputs and refuses a sibling beyond `DEPTH`
before storing it. Its serde form is the tuple (depth, map, siblings, ending)
for an absence proof and (depth, map, siblings) for an inclusion proof, and
`Deserialize` refuses a depth other than `DEPTH` before reading the map. A proof built at one depth therefore parses at no other.
satisfies: REQ-dyjq33, REQ-ymgg6b

**LLR-53ezgd**: `AbsenceProof::verify::<L, H>(root, identity)` starts from
`H::empty_leaf()` for an `Empty` ending and from `H::leaf(identity_hash,
content_hash)` for an `Occupied` one, folds in one sibling per level from the
leaf's level up — the empty-subtree hash where the map marks none — taking
left or right from the identity's position bits, and returns
`TreeError::ProofRootMismatch` when the result differs from `root`, before
any other check. It then returns `TreeError::ProofLeafHeld` when an
`Occupied` ending's identity hash equals `L::identity_hash(identity)`.
satisfies: REQ-ymgg6b

**LLR-c7bdee**: `InclusionProof::verify::<L, H>(root, leaf)` folds from the
leaf's hash by the same rule, along its identity's position, and returns
`TreeError::ProofRootMismatch` when the result differs from `root`.
satisfies: REQ-fmx9e7

**LLR-smf4zq**: every refusal named in this file is its own `TreeError`
variant, so the error says which check failed; through a serde format the
variant arrives as that format's custom error. No input to any function of
the crate reaches a panic.
satisfies: REQ-dyjq33

## Segregation

None: the unit and both of its consumers are class C, and the unit is a
library in its consumer's process.
