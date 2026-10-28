# ADR: Sparse Merkle tree leaves are unsalted

- **Date:** 2026-10-23
- **Status:** accepted (2026-10-23)
- **Relates to:** REQ-s3unp5 (sparse-merkle-tree); REQ-sb2frw (org-members);
  REQ-5kvqcf (person); `docs/adr/2026-10-23-one-sparse-merkle-tree.md`
- **Decision maker:** Jan-Jan (project owner), interviewed by `grill-requirements`

An absence proof contains hashes and never another key (REQ-s3unp5), but the
leaves those hashes are made from carry no secret salt. A verifier who already
has a candidate DevicePublicKey — and DevicePublicKeys are public, seen by
peers on the network — can hash it and test whether a proof's hashes were
made from it, so a revocation proof hides the holder's other DevicePublicKeys
only from a verifier who has none of them. The owner chose this over a
per-leaf salt, which would have hidden them from every verifier at the cost of
salts the holder must keep to build any proof, and over padding the proof to
hide how many devices the holder has.

The same holds for the Membership record. Adding salts later changes every
root, so this is recorded as a decision and not a default; the residual
disclosure is the owner's accepted risk, analysed in the risk files of the
units that produce the proofs.
