# Quint Milestone 3 — τ-window, Compromised Key, Convergence — Design

**Author(s):** [Jan-Jan van der Vyver](mailto:jan-jan@parity.io)
**Status:** In review
**Created:** 2026-06-17
**Last Updated:** 2026-06-17

## Overview

Milestone 3 extends the abstract-root `quint/protocol.qnt` with the time-dependent
parts of the ODS Phase 1 design: a per-device clock, the transitive-trust **τ-window**
property (under two policy variants), a **compromised-key** adversary, and a
**convergence** property. It is the final milestone of the protocol model begun in
[`2026-06-15-quint-protocol-model-design.md`](2026-06-15-quint-protocol-model-design.md)
(§Properties items 3–4, §Phasing item 3), built on the abstract-root representation
introduced in [`2026-06-16-quint-abstract-root-remodel-design.md`](2026-06-16-quint-abstract-root-remodel-design.md).

The central modeling insight (validated with the user during design): there is **one
taint mechanism** — a device acting on a *stale view of the members trie* — and the
τ-window bounds it. A device only learns of trie changes (member removals **and**
key rotations) when it next reads the trie; until then it may accept writes that are
already invalid on-chain. Compromised-key is the *key-generation* sub-case of this
single mechanism, not a separate instant-defeat story.

## Built on

`protocol.qnt` (abstract-root): roots are opaque `int` tokens; `rootMembers: int ->
Set[str]` maps a root to its member-set; `chain`/`local` carry root ids; honest
actions + network/revoked-insider/rogue-admin adversaries; properties `forkSafety`,
`revocationSafety`, `revokedExcludedFromOrgSecret`. `membership.qnt`,
`membership_mbt.qnt`, and the MBT harness are **out of scope and untouched** (M1/M2
stay green).

## Key design decisions (settled during brainstorming)

1. **Time = global logical clock + per-device `lastChecked`.** One `clock: int`
   advanced by `tick`, bounded by `CLOCK_MAX` (small, e.g. 4) for Apalache. Staleness
   of a device = `clock - lastChecked[d]`. τ is a bound on that.
2. **The staleness-bearing principal is the DEVICE, not the member.** Devices go
   online/offline and read the trie, so `lastChecked` and the believed trie root
   (`local`) are **per-device**. Membership itself (`rootKeys`, below) stays
   member-keyed; each device belongs to a member.
3. **A root commits to membership AND per-member key generation.** Replace
   `rootMembers: int -> Set[str]` with **`rootKeys: int -> (str -> int)`** (member id
   → that member's key generation at that trie version). `membersOf(r) =
   rootKeys.get(r).keys()`. There is no global `keyGen` table — key state is part of
   the trie version a device has synced.
4. **Both member removal and key rotation are trie updates** (admin actions that mint
   a new root, bump the epoch, post the on-chain anchor). A device learns of either
   only by syncing to the new root; the τ-window bounds the lag for both.
5. **Per-receiver acceptance against the device's stale view.** A write is accepted by
   a receiving device `d` iff the author is a current member *and* current key-gen
   **in `d`'s last-synced root** — both possibly stale.
6. **Compromised-key is the gen-mismatch sub-case of the τ-window**, not a separate
   property. The stable-identity/trie-rotation claim holds *once a device syncs past
   the rotation*, which is exactly the τ bound.

## State additions / changes

```quint
// CHANGED: roots now commit to membership + key generations
var rootKeys: int -> (str -> int)   // root id -> (member id -> key generation)
// (replaces `rootMembers: int -> Set[str]`; membersOf(r) = rootKeys.get(r).keys())

// NEW: device layer
// DEVICES is a fixed small set; deviceOwner maps each device to its member.
pure val DEVICES: Set[str] = Set("a1", "a2", "b1", "c1")
pure def deviceOwner(d: str): str = ...   // a1,a2 -> alice; b1 -> bob; c1 -> carol

var local: str -> { epoch: int, root: int }   // CHANGED: keyed by DEVICE now
var lastChecked: str -> int                    // NEW: per-device, clock at last trie read
var clock: int                                 // NEW: global logical clock

// CHANGED: per-receiver, richer
var acceptedWrites: Set[{ obj: str, author: { owner: str, gen: int },
                          receiver: str, staleness: int }]
```

Unchanged: `chain` (`{epoch, root, orgGen}`), `network`, `orgKnows`, `objToken`,
`tokenKnows`, `revoked`, `nextTag`, `nextRoot`. Constants add `TAU` (e.g. 2),
`CLOCK_MAX` (e.g. 4), and `POLICY` (`MAX_AGE` | `PAUSE_ON_LEARN`).

Genesis: root `0` with `rootKeys = Map(0 -> MEMBERS.mapBy(_ => 0))`, all devices
`local = {epoch:0, root:0}`, `lastChecked = DEVICES.mapBy(_ => 0)`, `clock = 0`.

## Actions

Carried over from the abstract-root model, with `rootMembers` → `rootKeys`
membership lookups and the member-level observe/fetch actions **re-keyed to devices**:

- **`adminProposeRemoval`** — mint a fresh root dropping `victim` from `rootKeys`,
  bump epoch/orgGen, anchor + seed delta/secret. (Now over `rootKeys`.)
- **`adminRotateKey`** *(new, a trie update)* — mint a fresh root where one member's
  gen is incremented (`rootKeys[new] = rootKeys[cur] with owner |-> gen+1`), bump
  epoch, anchor + seed delta. Models trie-mediated key rotation (also what device
  removal would trigger).
- **`deviceObserveChain`** — device `d` reads the chain: `lastChecked' =
  lastChecked.put(d, clock)`. Under `PAUSE_ON_LEARN`, this is also where `d` drops
  trust in peers not in the new chain root.
- **`deviceFetchAndApply`** — device `d` adopts a delta's result iff `d.base ==
  local[d].root and d.result == chain.root` (anchor check), updating `local[d]` and
  `lastChecked[d]`.
- **`memberReceiveOrgSecret`**, **`cgkaRotate`**, network / revoked-insider /
  rogue-admin adversaries — carried over (membership via `rootKeys`).
- **`tick`** — `clock' = clock + 1` while `clock < CLOCK_MAX`.
- **`deviceAcceptWrite`** *(new)* — device `d` accepts a `WriteOp{author}` from the
  network iff: `author.owner ∈ membersOf(local[d].root)` **and** `author.gen ==
  rootKeys.get(local[d].root).get(author.owner)` (both against `d`'s stale view),
  **and** the policy guard permits (below). Records
  `{obj, author, receiver: d, staleness: clock - lastChecked[d]}`.
- **`compromisedKeyWrite`** *(new adversary)* — emit a `WriteOp` for a still-current
  member `m` using a **stale** gen (`g < rootKeys[chain.root][m]`), modeling a stolen
  pre-rotation key. (The existing `revokedAttemptWrite` covers the removed-author
  case.)

**Policy guard on `deviceAcceptWrite`** (constant `POLICY`):
- `MAX_AGE`: accept only if `clock - lastChecked[d] < TAU`.
- `PAUSE_ON_LEARN`: accept only if `d` has not observed a chain epoch past the one
  in which the write's author became invalid; i.e. once `d` learns (its `local` epoch
  advances past the invalidation), it stops accepting from that author. Taint
  collapses to writes accepted before that observation.

## Properties

All simulator-checked; Apalache where tractable (depth from the spike, Task in plan).

- **`forkSafety`** — honest **devices** at the same epoch hold the same root id.
- **`revocationSafety`** — meaning unchanged; `membersOf` via `rootKeys`; "settled"
  = current **devices** caught up to `chain.epoch` AND the object token rotated
  at/after the epoch; then no revoked principal holds the token and every accepted
  current-epoch write is by a current member.
- **`tauWindow`** — the unified taint bound: for every accepted write that is
  **chain-invalid** (author removed from `chain.root` **or** `author.gen ≠
  rootKeys[chain.root][author.owner]`), the receiving device's recorded `staleness <
  TAU`. Checked under both `POLICY` settings. Compromised-key is the gen-mismatch
  case; revoked-author taint is the membership case.
- **`revokedExcludedFromOrgSecret`** — carried over (member-keyed).
- **`convergence`** — `quiescent implies (∀ honest online device d: local[d].root ==
  chain.root and local[d].epoch == chain.epoch)`, where `quiescent` = no deliverable
  `DeltaMsg`/`OrgSecretMsg` remains for any behind device and no admin update is
  pending. Plus a `convergesReachable` witness (`quint run`) proving the converged
  quiescent state is reachable (so `convergence` is non-vacuous).

## Instances & witnesses (`ods_instances.qnt`)

`MEMBERS = {alice, bob, carol}`; `DEVICES = {a1, a2, b1, c1}` (alice has two devices,
to exercise same-member devices at different staleness); `TAU = 2`; `CLOCK_MAX = 4`.
Non-vacuity witnesses (each must be reachable, i.e. `quint run --invariant` finds a
violation of the negated witness):
- `taintAcceptedReachable` — a stale device DOES accept a chain-invalid write (so
  `tauWindow` is non-vacuous).
- `keyGenMismatchAcceptedReachable` — the compromised-key (gen-mismatch) sub-case is
  reachable specifically.
- `convergesReachable` — quiescent fully-converged post-revocation state reachable.
- The carried-over `settledWithRevocationReachable`, `membersCanDifferReachable`
  (re-expressed over devices).

## Acceptance criteria

1. **Simulator:** `forkSafety`, `revocationSafety`, `revokedExcludedFromOrgSecret`,
   `convergence` pass at `--max-steps≈16 --max-samples≈5000`; `tauWindow` passes under
   **both** `POLICY` settings.
2. **Witnesses:** all listed witnesses find counterexamples (reachable / non-vacuous).
3. **Negative controls (teeth):** dropping the `MAX_AGE` staleness guard makes
   `tauWindow` fail; gating membership but **not** key-gen in `deviceAcceptWrite` lets
   a compromised-gen write slip through (a `tauWindow` violation); dropping the
   `== chain.root` anchor breaks `forkSafety`.
4. **Apalache:** a depth spike runs first; the device layer + `rootKeys: int -> (str
   -> int)` + clock add state, so the remodel's depth-5 ceiling may drop. The plan's
   first task measures the new ceiling and sets the CI `apalache` job depth to it.
   Target ≥3; if it falls below 3, that is a recorded finding (simulator still covers
   breadth) and the CI depth is set to whatever verifies.
5. **Scope:** `membership.qnt`, `membership_mbt.qnt`, the MBT harness unchanged
   (M1 green). `protocol.qnt` extended in place; `ods_instances.qnt` updated; CI +
   README updated.

## Non-goals

- CRDT operation reversal / "removing the taint" (design §10.b, explicit future
  scope). M3 proves the taint window is *bounded*, not that taint is *undone*.
- True temporal-liveness `eventually` (convergence stays quiescence + witness).
- Per-device cryptographic keys / peer-connection auth modeling (devices are
  staleness-bearing principals; their distinct device keys are not modeled — write
  authority is the member-as-a-group key, which is what `rootKeys` tracks).
- Re-introducing member re-addition (membership still only shrinks via removal; the
  abstract-root fresh-id faithfulness assumption continues to hold — key rotation
  changes a gen, not the member set, so it does not revisit a member-set either).

## Open questions

None blocking. The Apalache depth ceiling under the heavier state is the one
empirical unknown; the plan measures it first (spike) before committing CI depth.
