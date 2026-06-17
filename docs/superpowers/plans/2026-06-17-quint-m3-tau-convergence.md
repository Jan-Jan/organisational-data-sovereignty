# Quint Milestone 3 — τ-window, Compromised Key, Convergence — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the abstract-root `quint/protocol.qnt` with a per-device clock, the unified transitive-trust τ-window property (under MAX_AGE and PAUSE_ON_LEARN policies), a compromised-key adversary, and a convergence property — keeping all checks simulator-green and Apalache-verifiable at depth ≥5.

**Architecture:** In-place extension of `protocol.qnt`. Roots gain key generations (`rootMembers: int -> Set[str]` → `rootKeys: int -> (str -> int)`); a small device layer becomes the staleness-bearing principal (`local`/`lastChecked` keyed by device); a global `clock` drives staleness; writes are accepted per-receiving-device against its stale trie view. One taint mechanism (stale trie view) covers both member removal and key rotation, bounded by τ; compromised-key is its gen-mismatch sub-case.

**Tech Stack:** Quint 0.32. Simulator runs use `--backend=typescript`. Apalache `quint verify` (local) needs the env recipe below; in CI it's the `apalache` job.

---

> **Spec:** `docs/superpowers/specs/2026-06-17-quint-m3-tau-convergence-design.md`.
>
> **Validated by spike (quint 0.32 + Apalache 0.56.1):** a representative M3 core
> (`rootKeys` map-of-maps + 4-device layer + clock + `deviceAcceptWrite` with MAX_AGE
> + `tauWindow`) typechecks; `tauWindow` and `forkSafety` pass the simulator; taint is
> **reachable** (a stale device accepts a chain-invalid write — `tauWindow` is
> non-vacuous); and **Apalache verifies `tauWindow` at depth 5 in ~9s**. So the
> heavier state stays tractable; depth 5 is a viable CI target.
>
> **Local Apalache env (Bash sandbox disabled for these):**
> ```
> export HOME=/tmp/fakehome
> export JAVA_HOME=/Library/Java/JavaVirtualMachines/temurin-26.jdk/Contents/Home
> export PATH="$JAVA_HOME/bin:$PATH"
> ```
> Simulator (`quint run`/`quint test`) needs only `--backend=typescript`.

## Scope

IN: extend `protocol.qnt` (rootKeys, device layer, clock, adminRotateKey,
per-receiver write acceptance, both τ policies, compromised-key adversary,
convergence); update `ods_instances.qnt`; measure Apalache ceiling; negative
controls; CI + README.

OUT (unchanged): `membership.qnt`, `membership_mbt.qnt`, the MBT harness (M1 green).
Also out by design: CRDT taint *reversal* (only bounding the window), temporal
liveness, per-device crypto keys.

## Starting point (abstract-root `protocol.qnt`, 272 lines)

State vars: `chain {epoch,root,orgGen}`, `local: str -> LocalView`, `network`,
`rootMembers: int -> Set[str]`, `nextRoot`, `orgKnows`, `objToken: str -> {id,epoch}`,
`tokenKnows`, `revoked`, `acceptedWrites: Set[{obj,author:Key,epoch}]`, `nextTag`.
Helpers `membersOf(rm,r)=rm.get(r)`, `honestMembers=MEMBERS`, `getOrEmpty`,
`memberKey`. Actions: adminProposeRemoval, memberFetchAndApply, memberReceiveOrgSecret,
cgkaRotate, dataObjectWrite, networkDrop, networkDuplicate, revokedReplay,
revokedAttemptWrite, rogueProposeDelta. Properties: forkSafety, revocationSafety
(with `isSettled`), revokedExcludedFromOrgSecret.

## File Structure

| File | Change |
|------|--------|
| `quint/protocol.qnt` | Extended in place across Tasks 1–8. |
| `quint/ods_instances.qnt` | Witnesses updated/added — Task 9. |
| `.github/workflows/quint.yml` | apalache depth + new tau/convergence invariants — Task 10. |
| `quint/README.md` | M3 section — Task 10. |

**Naming locked:** `rootKeys`, `membersOf(rk,r)=rk.get(r).keys()`, `genOf(rk,r,m)=rk.get(r).get(m)`, `DEVICES`, `deviceOwner`, `clock`, `lastChecked`, `CLOCK_MAX=4`, `TAU=2`, `POLICY` (`"MAX_AGE"`|`"PAUSE_ON_LEARN"`), `staleness(d)=clock - lastChecked.get(d)`, properties `tauWindow`, `convergence`, helper `chainInvalid`, `quiescent`.

---

## Task 1: Migrate `rootMembers` → `rootKeys` (roots commit to key-gens)

**Files:** Modify `quint/protocol.qnt`

- [ ] **Step 1: Change the state var and helper.** Replace `var rootMembers: int -> Set[str]` with:
```quint
  var rootKeys: int -> (str -> int)     // root id -> (member id -> key generation)
```
Replace `pure def membersOf(rm: int -> Set[str], r: int): Set[str] = rm.get(r)` with:
```quint
  pure def membersOf(rk: int -> (str -> int), r: int): Set[str] = rk.get(r).keys()
  pure def genOf(rk: int -> (str -> int), r: int, m: str): int = rk.get(r).get(m)
```

- [ ] **Step 2: Update `init`.** Replace `rootMembers' = Map(0 -> MEMBERS)` with:
```quint
    rootKeys' = Map(0 -> MEMBERS.mapBy(_ => 0)),
```

- [ ] **Step 3: Update every `rootMembers` reference to `rootKeys`.** In each action, the stutter line `rootMembers' = rootMembers` becomes `rootKeys' = rootKeys`. In `adminProposeRemoval` replace
```quint
      rootMembers' = rootMembers.put(nr, membersOf(rootMembers, chain.root).exclude(Set(victim))),
```
with (carry over the surviving members' gens):
```quint
      rootKeys' = rootKeys.put(nr,
        rootKeys.get(chain.root).keys().exclude(Set(victim)).mapBy(k => rootKeys.get(chain.root).get(k))),
```
Do the same substitution in `rogueProposeDelta`. Everywhere else `membersOf(rootMembers, X)` becomes `membersOf(rootKeys, X)`, and `objToken`/`isSettled` references to `rootMembers` become `rootKeys`.

- [ ] **Step 4: Update `isSettled` signature** — its `rm` param type becomes `int -> (str -> int)`; body unchanged (uses `membersOf(rm, ch.root)`). Update its call in `revocationSafety` to pass `rootKeys`.

- [ ] **Step 5: Typecheck + invariants.**
```
quint typecheck quint/protocol.qnt
quint run --backend=typescript quint/protocol.qnt --invariant=forkSafety --max-steps=14 --max-samples=4000
quint run --backend=typescript quint/protocol.qnt --invariant=revocationSafety --max-steps=14 --max-samples=4000
quint run --backend=typescript quint/protocol.qnt --invariant=revokedExcludedFromOrgSecret --max-steps=14 --max-samples=4000
```
Expected: typecheck clean; all three `[ok]`. (This is a pure representation change; properties unchanged in meaning.)

- [ ] **Step 6: Commit** — `git add quint/protocol.qnt && git commit -m "feat(quint): roots commit to key-gens (rootKeys: int -> (str -> int))"`

---

## Task 2: Introduce the device layer

**Files:** Modify `quint/protocol.qnt`

The staleness-bearing principal becomes the device. `local` is re-keyed to devices; `forkSafety`, `revocationSafety`'s "caught up", and the member fetch/observe actions move to devices.

- [ ] **Step 1: Add device constants** (near `MEMBERS`):
```quint
  pure val DEVICES: Set[str] = Set("a1", "a2", "b1", "c1")
  pure def deviceOwner(d: str): str =
    if (d == "a1" or d == "a2") "alice" else if (d == "b1") "bob" else "carol"
  // devices whose owning member is current in root r
  pure def currentDevices(rk: int -> (str -> int), r: int): Set[str] =
    DEVICES.filter(d => membersOf(rk, r).contains(deviceOwner(d)))
```

- [ ] **Step 2: Re-key `local`.** Change `var local: str -> LocalView` to be device-keyed (same `LocalView = {epoch, root, orgGen}` type is fine; `orgGen` per device is unused but harmless — OR simplify to `{epoch, root}`. Use `{epoch: int, root: int}` and define `type DevView = { epoch: int, root: int }`, `var local: str -> DevView`). Update `init`:
```quint
    local' = DEVICES.mapBy(_ => { epoch: 0, root: 0 }),
```

- [ ] **Step 3: Convert `memberFetchAndApply` → `deviceFetchAndApply`** (pick a device, not member):
```quint
  action deviceFetchAndApply = {
    nondet d = DEVICES.oneOf()
    nondet te = network.oneOf()
    all {
      network.size() > 0,
      local' = match te.env {
        | DeltaMsg(dm) =>
          if (dm.base == local.get(d).root and dm.result == chain.root)
            local.put(d, { epoch: chain.epoch, root: dm.result })
          else local
        | _ => local
      },
      chain' = chain, network' = network, orgKnows' = orgKnows,
      objToken' = objToken, tokenKnows' = tokenKnows, revoked' = revoked,
      acceptedWrites' = acceptedWrites, nextTag' = nextTag,
      rootKeys' = rootKeys, nextRoot' = nextRoot,
    }
  }
```
(`lastChecked` is added in Task 3.) Update `step` to use `deviceFetchAndApply`.

- [ ] **Step 4: Re-express `forkSafety` over devices:**
```quint
  val forkSafety =
    DEVICES.forall(d1 => DEVICES.forall(d2 =>
      (local.get(d1).epoch == local.get(d2).epoch)
        implies (local.get(d1).root == local.get(d2).root)))
```

- [ ] **Step 5: Re-express `isSettled` "caught up" over current devices.** Change the first conjunct:
```quint
  pure def isSettled(obj: str, ch: ChainState, lv: str -> DevView,
                     tok: str -> { id: int, epoch: int }, rk: int -> (str -> int)): bool =
    and {
      currentDevices(rk, ch.root).forall(d => lv.get(d).epoch == ch.epoch),
      tok.get(obj).epoch >= ch.epoch,
    }
```
(Call site in `revocationSafety` unchanged in arity — still `isSettled(obj, chain, local, objToken, rootKeys)`.)

- [ ] **Step 6: `memberReceiveOrgSecret` stays member-keyed** (org secret is a member-group secret). It currently picks `m` from `honestMembers`; leave it — but its "is m current" guard uses `membersOf(rootKeys, chain.root)`. Keep as-is (members, not devices). `dataObjectWrite` will be replaced in Task 5; leave it for now (it still typechecks).

- [ ] **Step 7: Typecheck + invariants** (forkSafety, revocationSafety, revokedExcludedFromOrgSecret) — all `[ok]`. Note: with `local` now per-device, `revocationSafety`'s settled predicate quantifies over current devices.

- [ ] **Step 8: Commit** — `git commit -am "feat(quint): device layer — per-device local view; forkSafety/settled over devices"`

---

## Task 3: Clock + per-device `lastChecked`

**Files:** Modify `quint/protocol.qnt`

- [ ] **Step 1: Add state + constant.**
```quint
  pure val CLOCK_MAX: int = 4
  var clock: int
  var lastChecked: str -> int           // per-device: clock at last trie read
```
`init`: add `clock' = 0,` and `lastChecked' = DEVICES.mapBy(_ => 0),`.

- [ ] **Step 2: Add `tick` and `deviceObserveChain`.**
```quint
  action tick = all {
    clock < CLOCK_MAX,
    clock' = clock + 1,
    chain' = chain, local' = local, lastChecked' = lastChecked, rootKeys' = rootKeys,
    network' = network, orgKnows' = orgKnows, objToken' = objToken, tokenKnows' = tokenKnows,
    revoked' = revoked, acceptedWrites' = acceptedWrites, nextRoot' = nextRoot, nextTag' = nextTag,
  }

  action deviceObserveChain = {
    nondet d = DEVICES.oneOf()
    all {
      lastChecked' = lastChecked.put(d, clock),
      chain' = chain, local' = local, clock' = clock, rootKeys' = rootKeys,
      network' = network, orgKnows' = orgKnows, objToken' = objToken, tokenKnows' = tokenKnows,
      revoked' = revoked, acceptedWrites' = acceptedWrites, nextRoot' = nextRoot, nextTag' = nextTag,
    }
  }
```

- [ ] **Step 3: `deviceFetchAndApply` updates `lastChecked`** on a successful apply. Add a `lastChecked'` assignment mirroring the `local'` match:
```quint
      lastChecked' = match te.env {
        | DeltaMsg(dm) =>
          if (dm.base == local.get(d).root and dm.result == chain.root)
            lastChecked.put(d, clock) else lastChecked
        | _ => lastChecked
      },
```
and add `clock' = clock,` to its `all{}`. Every OTHER action must now also stutter `clock' = clock` and `lastChecked' = lastChecked` (add those two lines to each existing action's `all{}`).

- [ ] **Step 4: Add `tick`, `deviceObserveChain` to `step`.**

- [ ] **Step 5: Typecheck + forkSafety/revocationSafety** — `[ok]`. (If quint errors on an action missing `clock'`/`lastChecked'`, add the stutter — every action must assign every var.)

- [ ] **Step 6: Commit** — `git commit -am "feat(quint): global clock + per-device lastChecked + tick/observe"`

---

## Task 4: `adminRotateKey` (key rotation as a trie update)

**Files:** Modify `quint/protocol.qnt`

- [ ] **Step 1: Add the action** (mints a fresh root bumping one member's gen; same anchor/delta shape as removal):
```quint
  action adminRotateKey = {
    nondet m = membersOf(rootKeys, chain.root).oneOf()
    val nr = nextRoot
    val cur = rootKeys.get(chain.root)
    val newChain = { epoch: chain.epoch + 1, root: nr, orgGen: chain.orgGen + 1 }
    all {
      rootKeys' = rootKeys.put(nr, cur.put(m, cur.get(m) + 1)),
      nextRoot' = nr + 1,
      chain' = newChain,
      network' = network
        .union(Set({ tag: nextTag,     env: OnChainUpdate(newChain) }))
        .union(Set({ tag: nextTag + 1, env: DeltaMsg({ base: chain.root, result: nr }) })),
      nextTag' = nextTag + 2,
      revoked' = revoked, local' = local, lastChecked' = lastChecked, clock' = clock,
      orgKnows' = orgKnows, objToken' = objToken, tokenKnows' = tokenKnows,
      acceptedWrites' = acceptedWrites,
    }
  }
```

- [ ] **Step 2: Add `adminRotateKey` to `step`.**

- [ ] **Step 3: Typecheck + forkSafety/revocationSafety** — `[ok]`.

- [ ] **Step 4: Commit** — `git commit -am "feat(quint): adminRotateKey — trie-mediated key rotation bumps gen"`

---

## Task 5: Per-receiver write acceptance + τ-window (MAX_AGE) + compromised-key

**Files:** Modify `quint/protocol.qnt`

This replaces M2's global-chain `dataObjectWrite` with the per-receiver model (validated in the spike).

- [ ] **Step 1: Replace `acceptedWrites` shape.** Change its var to:
```quint
  var acceptedWrites: Set[{ obj: str, owner: str, gen: int, receiver: str, staleness: int }]
```
`init` already sets it to `Set()` — fine.

- [ ] **Step 2: Add a `WriteOp` envelope variant** if not present. The `Envelope` type's `WriteOp` becomes `WriteOp({ obj: str, owner: str, gen: int })` (drop the `Key`/epoch form). Update `revokedAttemptWrite` to emit the new shape (see Step 6).

- [ ] **Step 3: Remove `dataObjectWrite`; add `honestWrite` + `deviceAcceptWrite` + `compromisedKeyWrite`.**
```quint
  // honest current member emits a write with its CURRENT (chain) gen
  action honestWrite = {
    nondet m = membersOf(rootKeys, chain.root).oneOf()
    nondet obj = OBJECTS.oneOf()
    all {
      network' = network.union(Set({ tag: nextTag,
        env: WriteOp({ obj: obj, owner: m, gen: genOf(rootKeys, chain.root, m) }) })),
      nextTag' = nextTag + 1,
      chain' = chain, local' = local, lastChecked' = lastChecked, clock' = clock,
      rootKeys' = rootKeys, orgKnows' = orgKnows, objToken' = objToken,
      tokenKnows' = tokenKnows, revoked' = revoked, acceptedWrites' = acceptedWrites,
      nextRoot' = nextRoot,
    }
  }

  // compromised key: stale-gen write for a still-current member
  action compromisedKeyWrite = {
    nondet m = membersOf(rootKeys, chain.root).oneOf()
    nondet obj = OBJECTS.oneOf()
    all {
      genOf(rootKeys, chain.root, m) > 0,
      network' = network.union(Set({ tag: nextTag,
        env: WriteOp({ obj: obj, owner: m, gen: genOf(rootKeys, chain.root, m) - 1 }) })),
      nextTag' = nextTag + 1,
      chain' = chain, local' = local, lastChecked' = lastChecked, clock' = clock,
      rootKeys' = rootKeys, orgKnows' = orgKnows, objToken' = objToken,
      tokenKnows' = tokenKnows, revoked' = revoked, acceptedWrites' = acceptedWrites,
      nextRoot' = nextRoot,
    }
  }

  // device accepts a write vs its STALE local view; MAX_AGE policy guard
  action deviceAcceptWrite = {
    nondet d = DEVICES.oneOf()
    nondet te = network.oneOf()
    val lv = local.get(d).root
    all {
      network.size() > 0,
      clock - lastChecked.get(d) < TAU,
      match te.env {
        | WriteOp(w) => and {
            membersOf(rootKeys, lv).contains(w.owner),
            genOf(rootKeys, lv, w.owner) == w.gen,
          }
        | _ => false
      },
      acceptedWrites' = match te.env {
        | WriteOp(w) => acceptedWrites.union(Set({ obj: w.obj, owner: w.owner, gen: w.gen,
            receiver: d, staleness: clock - lastChecked.get(d) }))
        | _ => acceptedWrites
      },
      chain' = chain, local' = local, lastChecked' = lastChecked, clock' = clock,
      rootKeys' = rootKeys, network' = network, orgKnows' = orgKnows, objToken' = objToken,
      tokenKnows' = tokenKnows, revoked' = revoked, nextRoot' = nextRoot, nextTag' = nextTag,
    }
  }
```
Add `TAU` constant: `pure val TAU: int = 2`.

- [ ] **Step 4: Update `step`** — remove `dataObjectWrite`, add `honestWrite`, `compromisedKeyWrite`, `deviceAcceptWrite`.

- [ ] **Step 5: Add `chainInvalid` + `tauWindow`:**
```quint
  pure def chainInvalid(w: { obj: str, owner: str, gen: int, receiver: str, staleness: int },
                        ch: ChainState, rk: int -> (str -> int)): bool =
    not(membersOf(rk, ch.root).contains(w.owner)) or genOf(rk, ch.root, w.owner) != w.gen

  val tauWindow =
    acceptedWrites.forall(w => chainInvalid(w, chain, rootKeys) implies w.staleness < TAU)
```

- [ ] **Step 6: Update `revokedAttemptWrite`** to the new WriteOp shape and the per-receiver model — it now just emits a `WriteOp` for a revoked member at the current gen (acceptance is `deviceAcceptWrite`'s job; the revoked author will be chain-invalid):
```quint
  action revokedAttemptWrite = {
    nondet p = revoked.oneOf()
    nondet obj = OBJECTS.oneOf()
    all {
      revoked.size() > 0,
      network' = network.union(Set({ tag: nextTag, env: WriteOp({ obj: obj, owner: p, gen: 0 }) })),
      nextTag' = nextTag + 1,
      chain' = chain, local' = local, lastChecked' = lastChecked, clock' = clock,
      rootKeys' = rootKeys, orgKnows' = orgKnows, objToken' = objToken,
      tokenKnows' = tokenKnows, revoked' = revoked, acceptedWrites' = acceptedWrites,
      nextRoot' = nextRoot,
    }
  }
```

- [ ] **Step 7: Update `revocationSafety`** — its old write-authorship clause referenced the M2 `acceptedWrites` shape (`w.author`, `w.epoch`) which no longer exists. The write-authorship guarantee is now carried by `tauWindow` (a stale device may accept an ex-member's write within τ — exactly the taint `tauWindow` bounds). So `revocationSafety` keeps ONLY the token-exclusion clause:
```quint
  val revocationSafety =
    OBJECTS.forall(obj =>
      isSettled(obj, chain, local, objToken, rootKeys)
        implies tokenKnows.get(objToken.get(obj).id).intersect(revoked) == Set())
```
(Rationale recorded for the reviewer: the write clause moves to `tauWindow`; this is the M3 refinement noted in the spec discussion.)

- [ ] **Step 8: Typecheck + checks.**
```
quint typecheck quint/protocol.qnt
quint run --backend=typescript quint/protocol.qnt --invariant=tauWindow --max-steps=14 --max-samples=5000          # [ok]
quint run --backend=typescript quint/protocol.qnt --invariant=forkSafety --max-steps=14 --max-samples=4000          # [ok]
quint run --backend=typescript quint/protocol.qnt --invariant=revocationSafety --max-steps=14 --max-samples=4000    # [ok]
quint run --backend=typescript quint/protocol.qnt --invariant=revokedExcludedFromOrgSecret --max-steps=14 --max-samples=4000  # [ok]
```
Expected: all `[ok]`. (tauWindow holds because the MAX_AGE guard bounds acceptance staleness; this exact core was spike-validated.)

- [ ] **Step 9: Commit** — `git commit -am "feat(quint): per-receiver write acceptance + tauWindow (MAX_AGE) + compromised-key"`

---

## Task 6: PAUSE_ON_LEARN policy variant

**Files:** Modify `quint/protocol.qnt`

- [ ] **Step 1: Add the policy constant** `pure val POLICY: str = "MAX_AGE"` (near TAU). The default is MAX_AGE; the constant is flipped to test the other variant.

- [ ] **Step 2: Make `deviceAcceptWrite`'s guard policy-dependent.** Replace the single `clock - lastChecked.get(d) < TAU` guard with:
```quint
      // MAX_AGE: bounded by staleness. PAUSE_ON_LEARN: a device that has caught up to
      // the current chain epoch (local epoch == chain.epoch) stops accepting stale-view
      // writes; otherwise (still behind) it may, but only the in-window ones.
      if (POLICY == "PAUSE_ON_LEARN")
        local.get(d).epoch < chain.epoch or (clock - lastChecked.get(d) < TAU)
      else
        clock - lastChecked.get(d) < TAU,
```
> **Modeling note (subtlest part — the implementer should validate and adjust if needed):** PAUSE_ON_LEARN's intent is "once a device observes a chain change past an author's invalidation, it drops that author." A faithful, checkable encoding: a device whose `local.epoch == chain.epoch` (fully caught up) will not accept a write whose author is chain-invalid (it sees the current trie); a device still behind may accept within τ. If the literal guard above does not make `tauWindow` hold under PAUSE_ON_LEARN, refine it so acceptance of a *chain-invalid* write requires `local.get(d).epoch < chain.epoch` (the device hasn't learned) — i.e. a caught-up device never accepts taint. Keep the guard simple and document the final form. The acceptance test is: `tauWindow` `[ok]` under BOTH policies.

- [ ] **Step 3: Check `tauWindow` under MAX_AGE** (POLICY = "MAX_AGE"):
`quint run --backend=typescript quint/protocol.qnt --invariant=tauWindow --max-steps=14 --max-samples=5000` → `[ok]`.

- [ ] **Step 4: Flip POLICY to "PAUSE_ON_LEARN"** (edit the constant), re-run `tauWindow` → `[ok]`. Then set POLICY back to "MAX_AGE" (the committed default). If PAUSE_ON_LEARN needs the refined guard (modeling note), apply it before this passes.

- [ ] **Step 5: Typecheck; confirm forkSafety/revocationSafety still `[ok]` (POLICY="MAX_AGE").**

- [ ] **Step 6: Commit** — `git commit -am "feat(quint): PAUSE_ON_LEARN tau policy variant (POLICY constant)"`

---

## Task 7: Convergence property + witness

**Files:** Modify `quint/protocol.qnt`

- [ ] **Step 1: Add `quiescent` + `convergence`.**
```quint
  // no delta is deliverable to a behind device, and no device is behind the chain
  // via an undelivered update. Quiescence = nothing left to make progress on.
  pure def deliverableDelta(net: Set[TaggedEnv], lc: str -> DevView, ch: ChainState): bool =
    net.exists(te => match te.env {
      | DeltaMsg(dm) => dm.result == ch.root and DEVICES.exists(d => lc.get(d).root == dm.base)
      | _ => false
    })

  val quiescent = not(deliverableDelta(network, local, chain))

  val convergence =
    quiescent implies
      currentDevices(rootKeys, chain.root).forall(d => local.get(d).root == chain.root)
```
> Note: "online" devices are modeled as the current-member devices; a device permanently behind only because its delta was dropped is covered by `deliverableDelta` being false AND it not matching — convergence states that when no delta can advance anyone, all current devices already match the chain. If quint flags `deliverableDelta`'s `lc` param type, it is `str -> DevView` (the `local` var type).

- [ ] **Step 2: Check `convergence` (invariant):**
`quint run --backend=typescript quint/protocol.qnt --invariant=convergence --max-steps=16 --max-samples=6000` → expect `[ok]`. If it reports a violation, capture it: a quiescent state where a current device's root ≠ chain root with no deliverable delta is a real gap — analyze whether `quiescent`/`deliverableDelta` is mis-stated (e.g. a device whose `local.base` no longer matches because the chain moved twice while it was offline — that device legitimately needs a fresh delta; if none exists in `network`, the model's admin should have seeded one). Report BLOCKED with the trace + analysis rather than weakening `convergence`.

- [ ] **Step 3: Commit** — `git commit -am "feat(quint): convergence quiescence invariant"`

---

## Task 8: revokedExcludedFromOrgSecret carry-over check + full simulator sweep

**Files:** none (verification task; `revokedExcludedFromOrgSecret` already carried over unchanged).

- [ ] **Step 1: Confirm the lemma still holds** (member-keyed, unaffected by devices):
`quint run --backend=typescript quint/protocol.qnt --invariant=revokedExcludedFromOrgSecret --max-steps=16 --max-samples=5000` → `[ok]`.

- [ ] **Step 2: Full sweep at the committed POLICY="MAX_AGE":** run forkSafety, revocationSafety, tauWindow, convergence, revokedExcludedFromOrgSecret (max-steps=16, max-samples=5000 each) → all `[ok]`. Record the five results. No commit (verification only).

---

## Task 9: `ods_instances.qnt` — witnesses

**Files:** Modify `quint/ods_instances.qnt`

- [ ] **Step 1: Read the current file** and update existing witnesses for the new vars (`rootKeys`, per-device `local`, the `isSettled` arity is unchanged at 5 args). `membersCanDifferReachable` becomes over DEVICES; `settledWithRevocationReachable` keeps `isSettled(o, chain, local, objToken, rootKeys)`.

- [ ] **Step 2: Add M3 witnesses** (each a negated reachability claim; a `[violation]` proves reachable):
```quint
  // a stale device accepts a chain-invalid write (tauWindow non-vacuous)
  val taintAcceptedReachable =
    not(acceptedWrites.exists(w => chainInvalid(w, chain, rootKeys)))

  // the compromised-key (gen-mismatch) sub-case specifically is reachable
  val keyGenMismatchAcceptedReachable =
    not(acceptedWrites.exists(w =>
      membersOf(rootKeys, chain.root).contains(w.owner) and genOf(rootKeys, chain.root, w.owner) != w.gen))

  // a quiescent fully-converged post-removal state is reachable
  val convergedReachable =
    not(quiescent and chain.epoch > 0
        and currentDevices(rootKeys, chain.root).forall(d => local.get(d).root == chain.root))
```

- [ ] **Step 3: Typecheck + run each witness** (expect `[violation]` = reachable):
```
quint typecheck quint/ods_instances.qnt
quint run --backend=typescript quint/ods_instances.qnt --invariant=taintAcceptedReachable --max-steps=16 --max-samples=8000
quint run --backend=typescript quint/ods_instances.qnt --invariant=keyGenMismatchAcceptedReachable --max-steps=16 --max-samples=10000
quint run --backend=typescript quint/ods_instances.qnt --invariant=convergedReachable --max-steps=16 --max-samples=8000
```
Each: `[violation]`. If `keyGenMismatchAcceptedReachable` doesn't violate within budget, raise `--max-samples` to 20000 (it needs a rotation then a stale device accepting the old gen — a longer trace); if still none, report with analysis.

- [ ] **Step 4: Commit** — `git commit -am "feat(quint): M3 vacuity witnesses (taint, key-gen mismatch, convergence)"`

---

## Task 10: Apalache depth measurement + negative controls + CI + README

**Files:** Modify `.github/workflows/quint.yml`, `quint/README.md`; temporary mutations of `quint/protocol.qnt` (reverted).

- [ ] **Step 1: Apalache depth (env per the header).** Verify each property at depth 5 (sandbox disabled):
```
quint verify quint/protocol.qnt --invariant=tauWindow --max-steps=5
quint verify quint/protocol.qnt --invariant=forkSafety --max-steps=5
quint verify quint/protocol.qnt --invariant=revocationSafety --max-steps=5
quint verify quint/protocol.qnt --invariant=convergence --max-steps=5
```
Expected: `[ok]` (spike showed tauWindow at depth 5 ~9s; the full model may be slower — allow a few min each, `pkill -f apalache.jar` between). Record results + the highest depth that completes for `tauWindow` (probe depth 6/7 with `timeout 300`). The CI depth = `min(ceiling, 5)`.

- [ ] **Step 2: Negative control A — drop the MAX_AGE guard.** In `deviceAcceptWrite` temporarily remove the staleness guard (the `clock - lastChecked.get(d) < TAU` / policy `if`). Run `tauWindow` (simulator) → expect `[violation]` (a too-stale device accepts taint). Revert exactly; `git diff` empty; `tauWindow` `[ok]`.

- [ ] **Step 3: Negative control B — drop the key-gen check.** In `deviceAcceptWrite` temporarily remove the `genOf(rootKeys, lv, w.owner) == w.gen` conjunct (accept on membership alone). Run `tauWindow` → expect `[violation]` (a compromised stale-gen write accepted beyond τ via... actually it makes a gen-mismatch write acceptable regardless — confirm a `[violation]`). Revert exactly; `git diff` empty; `tauWindow` `[ok]`.

- [ ] **Step 4: Update `.github/workflows/quint.yml`.** In the `apalache` job, set all `quint verify` lines to `--max-steps=5` (or the Step-1 ceiling) and add tauWindow + convergence:
```yaml
      - run: quint verify quint/protocol.qnt --invariant=forkSafety --max-steps=5
      - run: quint verify quint/protocol.qnt --invariant=revocationSafety --max-steps=5
      - run: quint verify quint/protocol.qnt --invariant=revokedExcludedFromOrgSecret --max-steps=5
      - run: quint verify quint/protocol.qnt --invariant=tauWindow --max-steps=5
      - run: quint verify quint/protocol.qnt --invariant=convergence --max-steps=5
```
In the `quint` (simulator) job, add `quint run` lines for `tauWindow` and `convergence` (max-steps=16, max-samples=5000), and the M3 witness runs are optional. Preserve all existing steps/jobs.

- [ ] **Step 5: Update `quint/README.md`** — extend the protocol section: note the device layer, the τ-window (both policies), compromised-key as the gen-mismatch sub-case, convergence (quiescence + witness), and that Apalache verifies M3 to depth ~5. Keep prior content.

- [ ] **Step 6: Final green check** (POLICY="MAX_AGE"): typecheck both files; run all five invariants on the simulator → `[ok]`; confirm `git diff quint/protocol.qnt` shows only the intended M3 content (no leftover mutations).

- [ ] **Step 7: Commit** — `git add .github/workflows/quint.yml quint/README.md && git commit -m "ci(quint): M3 tau/convergence invariants + apalache depth 5; README"`

---

## Self-Review (completed by plan author)

**Spec coverage:** clock + per-device lastChecked → Task 3; device layer → Task 2; rootKeys/key-gens → Task 1; adminRotateKey → Task 4; per-receiver acceptance + τ-window (MAX_AGE) + compromised-key → Task 5; PAUSE_ON_LEARN → Task 6; convergence → Task 7; lemma carry-over → Task 8; witnesses → Task 9; Apalache depth + negative controls + CI + README → Task 10. revocationSafety re-expressed (token-only, write story → tauWindow) → Task 5 Step 7, with rationale. ✅

**Placeholder scan:** none. The one judgement point (PAUSE_ON_LEARN guard) carries a concrete starting guard + an explicit acceptance test (`tauWindow` `[ok]` under both policies) and a precise refinement rule — not a TODO. The core τ-mechanism (Task 5) is spike-validated verbatim.

**Type/name consistency:** `rootKeys`/`membersOf`/`genOf`/`DEVICES`/`deviceOwner`/`currentDevices`/`clock`/`lastChecked`/`DevView`/`chainInvalid`/`tauWindow`/`quiescent`/`convergence`/`POLICY`/`TAU`/`CLOCK_MAX` are introduced before use and used consistently. Every action assigns every state var (the plan flags adding `clock'`/`lastChecked'` stutters in Task 3 to all actions — a missing prime is a quint error, mechanical to fix).

**Known risks (validated where it counts):** Task 5's core (rootKeys + devices + clock + deviceAcceptWrite + tauWindow) was spiked: typechecks, `tauWindow` `[ok]`, taint reachable, Apalache depth 5 ~9s. The integrated full model (adding org/CGKA carryover + convergence + PAUSE_ON_LEARN) is assembled incrementally with a simulator gate after each task, so a regression localizes to one task. revocationSafety's clause drop (Task 5 Step 7) is a deliberate refinement; if the reviewer disagrees, the token-only form is still sound and the write story is provably tauWindow's. The PAUSE_ON_LEARN guard (Task 6) is the one piece most likely to need iteration; its acceptance criterion is explicit.
