# Absence proofs

Design for S1 of `docs/plans/2026-10-06-org-io-roadmap.md`. It serves
REQ-535jcd (production), REQ-yyuxh8 (content) and REQ-tk2qqj (verification),
and resolves PR-jq43gx. The owner chose option A on 2026-10-06 over two
alternatives: stopping at the first empty subtree (a variable starting level
for the verifier, with the same size as A, since A's bitmap already
compresses everything under an empty subtree), and sending all 256 siblings
in full (8 KiB, sixteen times A's size for an Organisation of about a
thousand members).

The work is split between the existing store and a new item. SDD-d9svdj
(`2026-09-17-decomposition.md`) holds member records "without interpreting
them", so it gains only the path walk, LLR-utp6x4, which interprets nothing.
The new item is the only code that looks inside a leaf. It uses the existing
hash domains (SDD-d6x85b, LLR-72p8bz) and person's `compute_device_root`
(LLR-kdhd2v). There is no new SOUP. The proof is a new postcard wire type under
the existing `serde` feature (`soup.md`, the postcard and serde rows).

## The store's part (refines SDD-d9svdj)

**LLR-utp6x4**: for a MemberId, the store returns the hash of each of the 256
siblings along that identifier's path, ordered from the leaf's level up to the
root's, together with what the path ends in: the Member's leaf or an empty
slot. Where the path enters an empty subtree, every sibling below it is that
level's default hash (LLR-wm5hpc). This holds at every boundary: a trie
with one member, and the lowest and highest identifiers (all bits 0, all
bits 1). satisfies: REQ-535jcd

The walk runs only on a calculated trie. `prove_absent` refuses an
uncalculated one first (LLR-4xz255). The walk's own `HashesNotCalculated` on an
unset sibling hash is a defensive check that the public interface cannot
reach. The verification gate of 2026-10-06 found the original wording here,
"refuses on a trie that has not been calculated", untestable at this item's
interface, and it was moved to LLR-4xz255, which tests it.

## The absence proof

**SDD-57vaj4**: the absence proof (`org-members/src/proof.rs`): produces,
from a calculated trie, a proof that a MemberId holds no Member or that the
Member it holds lacks a Device key; parses that proof when it arrives from
outside the process; and verifies it against a membership root. It is the
only item that interprets a leaf for this purpose. traces: REQ-535jcd,
REQ-yyuxh8, REQ-tk2qqj

**LLR-4xz255**: `OrgTrie::prove_absent(id, device)` returns an `AbsenceProof`
when `device` is not among the Device keys of the Member under `id`, or when
`id` holds no Member. It refuses with `HashesNotCalculated` when the trie is not
calculated, and with `DeviceStillHeld` when that Member holds `device`. A
Device key held by a different Member does not refuse. satisfies: REQ-535jcd

**LLR-25tpdp**: an `AbsenceProof` holds exactly three things: a 256-bit map
marking which siblings are their level's default hash, the hashes of the
siblings that are not, in path order, and an ending that is either `Empty` or
the leaf of the Member under the proof's MemberId. It holds no other Member's
data, and holds no Member data at all when the ending is `Empty`.
satisfies: REQ-yyuxh8

**LLR-4rju5r**: an `AbsenceProof`'s fields are private, and the only ways to
obtain one are `prove_absent` and parsing. `AbsenceProof::from_parts` refuses
with `AbsenceProofMalformed` when the number of sibling hashes is not the
number of non-default bits in the map. Serde's `Deserialize`, under the `serde`
feature, refuses the same inputs; the refusal reaches a postcard caller as the
format's custom error (`postcard::Error::SerdeDeCustom`) carrying that
variant's message. A proof therefore holds at most 256 hashes, and decoding
refuses a 257th sibling before storing it. A leaf in the ending is decoded
through `MemberLeaf`'s validating `Deserialize`, so it meets every rule a
received record meets (REQ-shk82j). (Amended 2026-10-06, review finding-3.)
satisfies: REQ-tk2qqj, REQ-ds8ryr

**LLR-dgzy7e**: `AbsenceProof::verify::<H>(root, id, device)` recomputes the
root. It starts from the empty-leaf hash for an `Empty` ending, or for a
`Leaf` ending from the member-leaf hash of that leaf's canonical bytes and its
device root. It then folds in one sibling per level, from the leaf's level up,
taking the default hash at each level the map marks. The identifier's bit for
that level says whether the running hash is the left or the right child. When
the result differs from `root`, it refuses with `AbsenceProofRootMismatch`, and
it checks this before anything else. satisfies: REQ-tk2qqj

**LLR-p2p8qy**: once the root matches, `verify` refuses a `Leaf` ending whose
Device keys include `device` with `DeviceStillHeld`. It accepts an `Empty`
ending, and a `Leaf` ending without `device`. satisfies: REQ-tk2qqj

There is no separate check that a `Leaf` ending's MemberId is `id`. The fold
in LLR-dgzy7e takes its left and right turns from `id`'s bits, and a trie
places a leaf only at its own identifier's path (LLR-zbe553). So a leaf for
another Member resolves to the given root only through a blake3 collision,
and a check for it could never be reached by any test. It was dropped from
the design on 2026-10-06 while the plan was being written.

**LLR-2dcnbp**: every refusal named in this file is, from the crate's own
functions (`prove_absent`, `from_parts`, `verify`), its own `OrgMembersError`
variant, so the error says which check failed (`HashesNotCalculated`,
`DeviceStillHeld`, `AbsenceProofMalformed`, `AbsenceProofRootMismatch`).
Through a serde format the variant arrives as that format's custom error. No
input to `prove_absent`, to parsing or to `verify` reaches a panic. (Amended
2026-10-06, review finding-3.) satisfies: REQ-tk2qqj, REQ-535jcd, REQ-ds8ryr

## No panics in the path primitives (resolves PR-jq43gx)

**LLR-7jkcba**: `MemberId::bit(index)` returns `IndexOutOfRange` for an index
of 256 or more, and `DefaultHashes::at_level(level)` returns it for a level of
257 or more, without indexing out of bounds. Inside the crate, the store's
walks (`insert`, `remove`, `get_member`, the path walk and `verify`) take the
identifier's bits from `MemberId::path_bits`, an iterator over its 256 bits,
most significant first, that cannot go out of range. The callers of `at_level`
(`empty_root`, `insert`, `remove`, the path walk, the proof) pass its error on
and never discard it. satisfies: REQ-ds8ryr
