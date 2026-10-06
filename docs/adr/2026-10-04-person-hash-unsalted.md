# ADR: The Person definition hash is unsalted

- **Date:** 2026-10-04
- **Status:** accepted (2026-10-04)
- **Relates to:** REQ-9m5pq2, REQ-7n4g8b
- **Decision maker:** Jan-Jan (project owner), interviewed by `grill-requirements`

A hash of a name, a surname and keys, published on-chain, looks like an
invitation to confirm guesses, and a salt would prevent that. The Person hash
has none, deliberately: only the collaborators a person shares the definition
with ever see its keys, so an outsider cannot assemble the inputs to guess, and
those collaborators are meant to recompute the hash and recognise the on-chain
entry as that person's — identifying the owner to them is the purpose, not a
leak.
