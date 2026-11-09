# Absence proofs

Design for stage S1 of the org-io roadmap: the absence proof, the only code
that looks inside a member leaf. Its LLRs are in
`2026-10-24-on-sparse-merkle-tree.md`, where the proof is composed of the
`sparse-merkle-tree` unit's proofs. There is no new SOUP. The proof is a
postcard wire type under the existing `serde` feature (`soup.md`, the postcard
and serde rows).

## The absence proof

**SDD-57vaj4**: the absence proof (`org-members/src/proof.rs`), the one
value showing against a Membership root that a MemberId's Member, if any,
lacks a DevicePublicKey.
traces: REQ-yyuxh8, REQ-sb2frw, REQ-65xrgh

(Amended 2026-10-24, `worktree-generic-smt`: the proof is composed of
`sparse-merkle-tree`'s proofs in two steps, under REQ-sb2frw and REQ-65xrgh;
its LLRs are in `2026-10-24-on-sparse-merkle-tree.md`. The statement above was
restated on 2026-10-28, guideline review round 5, as one responsibility;
producing, parsing and verifying the proof are its LLRs.)
