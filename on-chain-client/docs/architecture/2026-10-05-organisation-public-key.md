# The state decoder's middle slot, by its name

The owner ruled (2026-10-04) that the second field of an Organisation state is
the Organisation public key, an X25519 key-agreement key, and never a signing
key (root `docs/CONTEXT.md`). This item restates LLR-nq7nhg, in
`2026-09-28-decomposition.md` under the state decoder, with that term; the
decoding is unchanged, and the decoder still treats the slot as opaque bytes.

## SDD-f2s7bx — The Organisation state decoder

LLR-hezpr7 belongs to SDD-f2s7bx, as LLR-nq7nhg and LLR-sq76u3 do.

**LLR-hezpr7**: the ninety-six bytes are read as the Membership root in the
first slot, the Organisation public key in the second and the Epoch in the
third, each at its own thirty-two byte offset. satisfies: REQ-4astjb
supersedes: LLR-nq7nhg

**Robustness.** LLR-hezpr7 is the normal case of the decoder and has no
abnormal side of its own, as LLR-nq7nhg had none: what it reads are offsets
into a blob already known to be ninety-six bytes long. Its abnormal side is
LLR-sq76u3, its sibling under the same software item, which refuses every
other length with a typed error, and LLR-hezpr7 is LLR-sq76u3's normal side in
turn. `exactly_ninety_six_bytes_decodes_every_field`
(`on-chain-client/tests/decode_org_state.rs`) carries LLR-hezpr7; the four
length-refusal tests beside it carry LLR-sq76u3. The pairing is recorded in
`2026-09-28-decomposition.md`'s table of one-sided LLRs.
