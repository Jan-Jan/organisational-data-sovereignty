# ADR: One in-house sparse Merkle tree, in its own unit

- **Date:** 2026-10-23
- **Status:** accepted (2026-10-23)
- **Relates to:** REQ-vg9wq4, REQ-6cej5z, REQ-uy4euz, REQ-ymgg6b, REQ-s3unp5,
  REQ-3szczq, REQ-fmx9e7 (sparse-merkle-tree); REQ-sb2frw, REQ-65xrgh,
  REQ-2sz7j8 (org-members); REQ-5kvqcf, REQ-uh2bzg, REQ-9sh8qb, REQ-4e23xr
  (person)
- **Decision maker:** Jan-Jan (project owner), interviewed by `grill-requirements`

The Membership record and every device root tree are built on one sparse Merkle
tree, written in this repository as the class C unit `sparse-merkle-tree`,
generic over its leaf, its hash domains and its depth: 256 for the Membership
record, 4 for a device root tree. No third-party crate is used for it.

## Why not a crate

The Rust candidates surveyed on 2026-10-22 (nervos `sparse-merkle-tree`,
Penumbra `jmt`, `fuel-merkle`, `monotree`, `miden-crypto`, `nomt`) each fail on
at least one of: no published audit of the tree, no release since 2022, a
fixed hash function other than blake3, a heavy dependency set, or a root
format of their own. As SOUP in a class C unit each would need its anomalies
argued, and its root format would make this project's golden vectors and
Quint model follow third-party code. A classic fixed-depth tree is a few
hundred lines that can be specified, branch-covered and audited once, and
org-members already had one.

## Why its own unit

Two consumers need the same tree: the Membership record, and a device root tree
under which a DevicePublicKey's absence must be provable without disclosing
the holder's other DevicePublicKeys (owner ruling 2026-10-22). One unit gives
one implementation to test and audit, and keeps each consumer's code to its
leaf, its key and its domain rules. The leaf decides what its key is, how it
is hashed and what a proof discloses of it; the tree decides the positions,
the hashing to the root and the proof's shape.

## Why depth 4 for a device root tree

An absence proof is sound at any depth when a leaf is told apart by its whole
key; depth only sets how often two keys share a position. A device generates
its own key pair, a different one for every record it is in, so it checks a
new key against the one set it joins and generates another on a collision
(REQ-9sh8qb). At depth 4 a holder of three DevicePublicKeys meets that about
once in eight additions, and the proof is at most four hashes. The bound of 8
DevicePublicKeys is a domain rule held below the 16 positions, so the tree is
never full. Depths that vary per Organisation, published on-chain, are left to
a later change.

## Consequences

- Every device root changes, so every Person hash and every Membership root
  changes. V1 of the Person encoding is redefined over the new device root
  rather than versioned, because no Person hash has been published or used
  outside this repository (owner ruling 2026-10-23).
- org-members and person each gain a dependency edge to
  `sparse-merkle-tree`, declared after its assessment.
- Changing a depth later changes every root at that depth and needs a
  version.
