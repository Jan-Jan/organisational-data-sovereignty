# ODS Phase 1 — Quint protocol model

Models the Organisational Data Sovereignty Phase 1 protocol around the
`org-members` crate. Design spec:
`docs/superpowers/specs/2026-06-15-quint-protocol-model-design.md`.

These models have lived in org-node since part 2 of
docs/adr/2026-10-03-quint-conformance-gate.md (2026-10-03): both typechecks and
the five simulator invariants are unconditional entries in org-node's
`verify_commands`. The membership model moved to org-members/quint/ in part 1.

## Protocol layer (Milestones 2–3)

> **Scope disclaimer.** Everything below is verified about the Quint protocol
> *model* only. org-node has **no conformance test** yet — no Rust code is
> replayed against this model (ADR decision 13; the node-level model is a
> separate change). The `org-members` crate is the sole code under conformance
> test, against `org-members/quint/membership_mbt.qnt`. So `forkSafety`/`revocationSafety`/`tauWindow`/`convergence`
> are statements about the model, not about a running system. "verify" here means
> Apalache checked the *model*, not that an implementation was verified.

`protocol.qnt` is the distributed state machine. From org-members it imports only
the exported model interface, `org-members/quint/membership_types.qnt` (the `Key`
record), not the membership model itself. Its state is the on-chain anchor
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

Commands (the default rust backend, as org-node's `verify_commands` and CI run
them; the full list is in `org-node/.guardrails/config.yaml`):

- `quint run org-node/quint/protocol.qnt --invariant=forkSafety --max-steps=16 --max-samples=5000`
- `quint run org-node/quint/protocol.qnt --invariant=revocationSafety --max-steps=16 --max-samples=5000`
- vacuity witnesses: `quint run org-node/quint/ods_instances.qnt --invariant=settledWithRevocationReachable ...` (finds a counterexample = the target state is reachable).

Negative controls (documented, not committed): dropping the `== chain.root` gate
in `deviceFetchAndApply` breaks `forkSafety` (re-measured 2026-10-03); leaking a CGKA token to revoked
members in `cgkaRotate` breaks `revocationSafety`. Both produce simulator
counterexamples.

**Apalache `quint verify`** runs in CI (the `apalache` job) and locally with a JVM.
`protocol.qnt` uses an **abstract-root representation** — roots are opaque `int`
tokens with a `rootMembers: int -> Set[str]` side-table, not full trie `Snapshot`
maps (the rich Snapshot/Leaf semantics live in `org-members/quint/membership.qnt`, validated by the
simulator + MBT). This keeps the protocol state small enough for Apalache to verify
`forkSafety`/`revocationSafety`/`revokedExcludedFromOrgSecret` to depth ~5-6 (the
concrete-Snapshot model topped out at depth 2). The simulator (`quint run
--invariant`) still covers greater breadth. Roots are fresh monotonic ids, faithful
while membership only shrinks (removals) — revisit if a later milestone adds member
re-addition. Convergence and the τ-window property arrive in Milestone 3.

Local Apalache run (outside CI) needs a JVM and a writable `$HOME`:
`HOME=/tmp/fakehome JAVA_HOME=$(/usr/libexec/java_home) PATH=$JAVA_HOME/bin:$PATH quint verify org-node/quint/protocol.qnt --invariant=forkSafety --max-steps=5`.

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
