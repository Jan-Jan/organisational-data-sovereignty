# org-members — membership model

## Modules

- `membership_types.qnt` — the exported model interface: the boundary types
  (`Key`, `Leaf`, `Snapshot`, `Result`, `Delta`). Another unit's model imports
  this module and nothing else from org-members
  (`docs/adr/2026-10-03-quint-conformance-gate.md`).
- `membership.qnt` — pure membership-trie semantics (types, ops, canonical-form
  `applyDelta`, round-trip law). A `RootHash` is modeled as the canonical member
  map itself (snapshot-as-root); collision-resistance is assumed.
- `membership_mbt.qnt` — runnable state machine over `membership`, source of the
  traces the conformance test replays.

## Commands

> In a normal environment the default `rust` backend is used. In a sandbox where
> `$HOME` is read-only (so the Quint rust evaluator cannot be fetched to
> `~/.quint`), append `--backend=typescript` to `quint test`/`quint run`.

- Typecheck: `quint typecheck org-members/quint/membership_types.qnt`,
  `quint typecheck org-members/quint/membership.qnt`,
  `quint typecheck org-members/quint/membership_mbt.qnt`
- Unit tests (round-trip law, op semantics): `quint test org-members/quint/membership.qnt`
- Simulate against sanity invariants:
  `quint run org-members/quint/membership_mbt.qnt --invariant=mbtInv --max-steps=15 --max-samples=1000`
- Conformance vs. the real crate (needs `quint` on PATH):
  `cargo test -p org-members --test mbt_conformance`

## Sandbox cargo recipe

This repo's dev sandbox mounts `~/.cargo` and `$HOME` read-only. To run the
conformance test locally:

```
RUSTUP_HOME=$HOME/.rustup HOME=/tmp/fakehome CARGO_HOME=/tmp/cargo-wt \
  cargo test -p org-members --test mbt_conformance -- --nocapture
```

(`HOME=/tmp/fakehome` gives quint-connect a writable home for its evaluator;
`CARGO_HOME=/tmp/cargo-wt` avoids the read-only cargo cache. CI uses the
defaults.)

## Caveats (by design)

- SMT / Merkle / hashing mechanics are on the declared abstraction boundary —
  covered by the crate's own tests and the root-hash equality-class check in
  the conformance test.
- Crypto is assumed sound; keys are `(owner, gen)` pairs, not bytes.
- Confusables are modeled via an explicit `skeleton` field, not real UTS#39.
- Protocol-layer properties (revocation/replay/τ-window/convergence) and the
  full adversary live in the protocol model (`org-node/quint/protocol.qnt`).
