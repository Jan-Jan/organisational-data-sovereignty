# ODS Phase 1 — Quint model

Models the Organisational Data Sovereignty Phase 1 protocol around the
`org-members` crate. Design spec:
`docs/superpowers/specs/2026-06-15-quint-protocol-model-design.md`.

## Modules

- `membership.qnt` — pure membership-trie semantics (types, ops, canonical-form
  `applyDelta`, round-trip law). A `RootHash` is modeled as the canonical member
  map itself (snapshot-as-root); collision-resistance is assumed.
- `membership_mbt.qnt` — runnable state machine over `membership`, source for
  model-based test traces.

## Commands

> In a normal environment the default `rust` backend is used. In a sandbox where
> `$HOME` is read-only (so the Quint rust evaluator cannot be fetched to
> `~/.quint`), append `--backend=typescript` to `quint test`/`quint run`.

- Typecheck: `quint typecheck quint/membership.qnt quint/membership_mbt.qnt`
- Unit tests (round-trip law, op semantics): `quint test quint/membership.qnt`
- Simulate against sanity invariants:
  `quint run quint/membership_mbt.qnt --invariant=mbtInv --max-steps=15 --max-samples=200`
- Conformance vs. the real crate (needs `quint` on PATH):
  `cd org-members && cargo test --test mbt_conformance`

## Sandbox cargo recipe

This repo's dev sandbox mounts `~/.cargo` and `$HOME` read-only. To run the MBT
conformance test locally:

```
cd org-members
HOME=/tmp/fakehome RUSTUP_HOME=$HOME/.rustup CARGO_HOME=/tmp/cargo-wt \
  cargo test --test mbt_conformance -- --nocapture
```

(`HOME=/tmp/fakehome` gives quint-connect a writable home for its evaluator;
`CARGO_HOME=/tmp/cargo-wt` avoids the read-only cargo cache. CI uses the
defaults.)

## Caveats (by design)

- SMT / Merkle / hashing mechanics are out of model scope — covered by the
  crate's own tests and the root-hash equality-class check in the MBT harness.
- Crypto is assumed sound; keys are `(owner, gen)` pairs, not bytes.
- Confusables are modeled via an explicit `skeleton` field, not real UTS#39.
- Protocol-layer properties (revocation/replay/τ-window/convergence) and the
  full adversary arrive in Milestone 2 (`protocol.qnt`).

## Protocol layer (Milestone 2)

`protocol.qnt` is the distributed state machine over `membership`: on-chain anchor
(`chain`), per-member belief (`local`), an unordered tagged-envelope `network`,
abstract knowledge sets (`orgKnows`, `tokenKnows`/`objToken`), and revocation /
accepted-write bookkeeping. `ods_instances.qnt` holds vacuity-witness invariants.

Adversaries: network (drop/duplicate), revoked-insider (replay, stale write),
below-threshold rogue admin (off-chain delta).

Properties (checked with the simulator):

- `forkSafety` — honest members at the same epoch hold the same root.
- `revocationSafety` — once an object is settled (current members caught up AND
  its CGKA token was rotated at/after the current epoch), no revoked principal
  holds its token and every accepted current-epoch write is by a current member.
  The epoch-stamped "settled" precondition deliberately excludes the pre-rekey
  transitive-trust window (that is the Milestone-3 τ-window, not a violation).

Commands (append `--backend=typescript` locally; CI uses defaults):

- `quint run quint/protocol.qnt --invariant=forkSafety --max-steps=16 --max-samples=5000`
- `quint run quint/protocol.qnt --invariant=revocationSafety --max-steps=16 --max-samples=5000`
- vacuity witnesses: `quint run quint/ods_instances.qnt --invariant=settledWithRevocationReachable ...` (finds a counterexample = the target state is reachable).

Negative controls (documented, not committed): dropping the `== chain.root` gate
in `memberFetchAndApply` breaks `forkSafety`; leaking a CGKA token to revoked
members in `cgkaRotate` breaks `revocationSafety`. Both produce simulator
counterexamples.

**Apalache `quint verify`** runs in CI (the `apalache` job) and locally with a JVM.
`protocol.qnt` uses an **abstract-root representation** — roots are opaque `int`
tokens with a `rootMembers: int -> Set[str]` side-table, not full trie `Snapshot`
maps (the rich Snapshot/Leaf semantics live in `membership.qnt`, validated by the
simulator + MBT). This keeps the protocol state small enough for Apalache to verify
`forkSafety`/`revocationSafety`/`revokedExcludedFromOrgSecret` to depth ~5-6 (the
concrete-Snapshot model topped out at depth 2). The simulator (`quint run
--invariant`) still covers greater breadth. Roots are fresh monotonic ids, faithful
while membership only shrinks (removals) — revisit if a later milestone adds member
re-addition. Convergence and the τ-window property arrive in Milestone 3.

Local Apalache run (outside CI) needs a JVM and a writable `$HOME`:
`HOME=/tmp/fakehome JAVA_HOME=$(/usr/libexec/java_home) PATH=$JAVA_HOME/bin:$PATH quint verify quint/protocol.qnt --invariant=forkSafety --max-steps=5`.

## Protocol layer (Milestone 3) — clock, τ-window, compromised key, convergence

Milestone 3 adds the time-dependent parts. The model is **device-centric**: a small
`DEVICES` set are the staleness-bearing principals; each device has its own local
trie view (`local`) and `lastChecked` clock reading. Roots now commit to membership
**and** per-member key generations (`rootKeys: int -> (str -> int)`), so both member
removal and key rotation are trie updates a device learns of only when it next syncs.

There is **one taint mechanism**: a device acting on a *stale trie view*. The
τ-window bounds it. Properties (all simulator-checked; Apalache as noted):

- `tauWindow` — a device accepts a *chain-invalid* write (author removed **or**
  author key-gen ≠ current) only if its staleness `< TAU`. Checked under both
  `POLICY = "MAX_AGE"` and `"PAUSE_ON_LEARN"` (flip the `POLICY` constant). The
  **compromised-key** adversary is the gen-mismatch sub-case of this single property.
- `convergence` — `quiescent implies (every current-member device holds the chain
  root)`. Convergence is, by the untimed-model limitation the design notes, a
  **quiescence-safety companion** to its real signal: the `convergedReachable`
  witness in `ods_instances.qnt` (the converged state is reachable). It is not a
  temporal-liveness `eventually`.
- `forkSafety`/`revocationSafety` carry over, re-expressed over devices;
  `revocationSafety`'s write-authorship guarantee moved to `tauWindow` (a stale
  device may legitimately accept an ex-member's write within τ — exactly the taint
  τ bounds), leaving `revocationSafety` as the CGKA-token-exclusion property.

Apalache depths (measured on the M3 model): `forkSafety`, `revocationSafety`,
`tauWindow`, `revokedExcludedFromOrgSecret` verify at **depth 5** (~30–42s each);
`convergence` is heavier (its `reachableRoots` transitive closure) and verifies at
**depth 3** (~21s). CI's `apalache` job uses those depths.

Negative controls (documented, not committed): dropping the staleness guard in
`deviceAcceptWrite` breaks `tauWindow`; dropping the `== chain.root` anchor in
`deviceFetchAndApply` breaks `forkSafety`. Both produce simulator counterexamples.
