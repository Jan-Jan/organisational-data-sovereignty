// -*- mode: Bluespec; -*-
/// Distributed ODS Phase 1 protocol state machine (Milestone 2: protocol safety),
/// abstract-root representation: roots are opaque int tokens with a `rootKeys`
/// side-table, so Apalache `quint verify` is tractable at depth. The rich
/// Snapshot/Leaf/applyDelta semantics live in membership.qnt (simulator + MBT).
///
/// Faithfulness assumption (see spec): fresh monotonic root ids are faithful while
/// membership only ever SHRINKS (removals), so no two reachable roots share a
/// member-set. Revisit this if a future milestone adds member re-addition.
module protocol {
  import membership.* from "./membership"

  type Principal = str
  type ChainState = { epoch: int, root: int, orgGen: int }
  type DevView    = { epoch: int, root: int }

  type Envelope =
    | OnChainUpdate(ChainState)
    | DeltaMsg({ base: int, result: int })
    | OrgSecretMsg({ gen: int, epoch: int })
    | WriteOp({ obj: str, owner: str, gen: int })
  type TaggedEnv = { tag: int, env: Envelope }

  // ---- concrete small instance ----
  pure val MEMBERS: Set[str] = Set("alice", "bob", "carol")
  pure val DEVICES: Set[str] = Set("a1", "a2", "b1", "c1")
  pure def deviceOwner(d: str): str =
    if (d == "a1" or d == "a2") "alice" else if (d == "b1") "bob" else "carol"
  pure def currentDevices(rk: int -> (str -> int), r: int): Set[str] =
    DEVICES.filter(d => membersOf(rk, r).contains(deviceOwner(d)))
  pure val ADMINS: Set[str] = Set("a1", "a2")
  pure val THRESHOLD: int = 2
  pure val OBJECTS: Set[str] = Set("o1")
  pure val ORG: str = "org"

  pure def memberKey(m: str, g: int): Key = { owner: m, gen: g }

  pure val CLOCK_MAX: int = 4
  pure val TAU: int = 2
  pure val POLICY: str = "MAX_AGE"

  // ---- state ----
  var chain: ChainState
  var local: str -> DevView
  var network: Set[TaggedEnv]
  var rootKeys: int -> (str -> int)     // root id -> (member id -> key generation)
  var nextRoot: int                     // fresh-id counter for root tokens
  var orgKnows: int -> Set[Principal]   // org-key gen -> holders
  var objToken: str -> { id: int, epoch: int }  // object -> current CGKA token
  var tokenKnows: int -> Set[Principal] // CGKA token id -> holders
  var revoked: Set[Principal]
  var acceptedWrites: Set[{ obj: str, owner: str, gen: int, receiver: str, staleness: int }]
  var nextTag: int
  var clock: int
  var lastChecked: str -> int           // per-device: clock value at last trie read

  action init = all {
    chain' = { epoch: 0, root: 0, orgGen: 0 },
    local' = DEVICES.mapBy(_ => { epoch: 0, root: 0 }),
    network' = Set(),
    rootKeys' = Map(0 -> MEMBERS.mapBy(_ => 0)),
    nextRoot' = 1,
    orgKnows' = Map(0 -> MEMBERS),
    objToken' = OBJECTS.mapBy(_ => { id: 0, epoch: 0 }),
    tokenKnows' = Map(0 -> MEMBERS),
    revoked' = Set(),
    acceptedWrites' = Set(),
    nextTag' = 0,
    clock' = 0,
    lastChecked' = DEVICES.mapBy(_ => 0),
  }

  // ---- helpers ----
  pure def membersOf(rk: int -> (str -> int), r: int): Set[str] = rk.get(r).keys()
  pure def genOf(rk: int -> (str -> int), r: int, m: str): int = rk.get(r).get(m)
  val honestMembers: Set[str] = MEMBERS
  pure def getOrEmpty(mp: int -> Set[Principal], k: int): Set[Principal] =
    if (mp.keys().contains(k)) mp.get(k) else Set()

  // ADMIN: propose removing `victim`. Mints a fresh root id, records its member
  // set, advances the chain, and seeds on-chain update + delta + org secret.
  action adminProposeRemoval = {
    nondet victim = membersOf(rootKeys, chain.root).oneOf()
    val nr = nextRoot
    val newChain = { epoch: chain.epoch + 1, root: nr, orgGen: chain.orgGen + 1 }
    all {
      membersOf(rootKeys, chain.root).size() > 1,
      rootKeys' = rootKeys.put(nr,
        rootKeys.get(chain.root).keys().exclude(Set(victim)).mapBy(k => rootKeys.get(chain.root).get(k))),
      nextRoot' = nr + 1,
      chain' = newChain,
      network' = network
        .union(Set({ tag: nextTag,     env: OnChainUpdate(newChain) }))
        .union(Set({ tag: nextTag + 1, env: DeltaMsg({ base: chain.root, result: nr }) }))
        .union(Set({ tag: nextTag + 2, env: OrgSecretMsg({ gen: chain.orgGen + 1, epoch: chain.epoch + 1 }) })),
      nextTag' = nextTag + 3,
      revoked' = revoked.union(Set(victim)),
      local' = local, orgKnows' = orgKnows, objToken' = objToken,
      tokenKnows' = tokenKnows, acceptedWrites' = acceptedWrites,
      clock' = clock, lastChecked' = lastChecked,
    }
  }

  // ADMIN rotate member m's key: fresh root with bumped gen, epoch+orgGen bump, anchor + delta.
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
        .union(Set({ tag: nextTag + 1, env: DeltaMsg({ base: chain.root, result: nr }) }))
        .union(Set({ tag: nextTag + 2, env: OrgSecretMsg({ gen: chain.orgGen + 1, epoch: chain.epoch + 1 }) })),
      nextTag' = nextTag + 3,
      revoked' = revoked, local' = local, lastChecked' = lastChecked, clock' = clock,
      orgKnows' = orgKnows, objToken' = objToken, tokenKnows' = tokenKnows,
      acceptedWrites' = acceptedWrites,
    }
  }

  // DEVICE: fetch a DeltaMsg whose base matches my local root id, and adopt its
  // result iff it equals the chain root id (the on-chain anchor check).
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
      lastChecked' = match te.env {
        | DeltaMsg(dm) =>
          if (dm.base == local.get(d).root and dm.result == chain.root)
            lastChecked.put(d, clock) else lastChecked
        | _ => lastChecked
      },
      clock' = clock,
      chain' = chain, network' = network, orgKnows' = orgKnows,
      objToken' = objToken, tokenKnows' = tokenKnows, revoked' = revoked,
      acceptedWrites' = acceptedWrites, nextTag' = nextTag,
      rootKeys' = rootKeys, nextRoot' = nextRoot,
    }
  }

  // MEMBER: receive an OrgSecretMsg; only a current member is added to holders.
  action memberReceiveOrgSecret = {
    nondet m = honestMembers.oneOf()
    nondet te = network.oneOf()
    all {
      network.size() > 0,
      membersOf(rootKeys, chain.root).contains(m),
      orgKnows' = match te.env {
        | OrgSecretMsg(s) => orgKnows.put(s.gen, getOrEmpty(orgKnows, s.gen).union(Set(m)))
        | _ => orgKnows
      },
      chain' = chain, local' = local, network' = network, objToken' = objToken,
      tokenKnows' = tokenKnows, revoked' = revoked,
      acceptedWrites' = acceptedWrites, nextTag' = nextTag,
      rootKeys' = rootKeys, nextRoot' = nextRoot,
      clock' = clock, lastChecked' = lastChecked,
    }
  }

  // CGKA: rotate object token to a fresh id held by current members, stamped with
  // the current epoch.
  action cgkaRotate = {
    nondet obj = OBJECTS.oneOf()
    all {
      objToken' = objToken.put(obj, { id: nextTag, epoch: chain.epoch }),
      tokenKnows' = tokenKnows.put(nextTag, membersOf(rootKeys, chain.root)),
      nextTag' = nextTag + 1,
      chain' = chain, local' = local, network' = network, orgKnows' = orgKnows,
      revoked' = revoked, acceptedWrites' = acceptedWrites,
      rootKeys' = rootKeys, nextRoot' = nextRoot,
      clock' = clock, lastChecked' = lastChecked,
    }
  }

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
      if (POLICY == "PAUSE_ON_LEARN")
        // a caught-up device sees the current trie and won't accept stale-view taint;
        // a behind device may, still bounded by the staleness window
        (local.get(d).epoch < chain.epoch and clock - lastChecked.get(d) < TAU)
      else
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

  // NETWORK ADVERSARY: drop an envelope.
  action networkDrop = {
    nondet te = network.oneOf()
    all {
      network.size() > 0,
      network' = network.exclude(Set(te)),
      chain' = chain, local' = local, orgKnows' = orgKnows, objToken' = objToken,
      tokenKnows' = tokenKnows, revoked' = revoked, acceptedWrites' = acceptedWrites,
      nextTag' = nextTag, rootKeys' = rootKeys, nextRoot' = nextRoot,
      clock' = clock, lastChecked' = lastChecked,
    }
  }

  // NETWORK ADVERSARY: duplicate an envelope under a fresh tag.
  action networkDuplicate = {
    nondet te = network.oneOf()
    all {
      network.size() > 0,
      network' = network.union(Set({ tag: nextTag, env: te.env })),
      nextTag' = nextTag + 1,
      chain' = chain, local' = local, orgKnows' = orgKnows, objToken' = objToken,
      tokenKnows' = tokenKnows, revoked' = revoked, acceptedWrites' = acceptedWrites,
      rootKeys' = rootKeys, nextRoot' = nextRoot,
      clock' = clock, lastChecked' = lastChecked,
    }
  }

  // REVOKED INSIDER: replay an in-flight envelope under a fresh tag.
  action revokedReplay = {
    nondet te = network.oneOf()
    all {
      network.size() > 0,
      revoked.size() > 0,
      network' = network.union(Set({ tag: nextTag, env: te.env })),
      nextTag' = nextTag + 1,
      chain' = chain, local' = local, orgKnows' = orgKnows, objToken' = objToken,
      tokenKnows' = tokenKnows, revoked' = revoked, acceptedWrites' = acceptedWrites,
      rootKeys' = rootKeys, nextRoot' = nextRoot,
      clock' = clock, lastChecked' = lastChecked,
    }
  }

  // REVOKED INSIDER: attempt a write with a stale key; never accepted (gated on
  // current membership), but still emits the WriteOp envelope.
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

  // ROGUE ADMIN (< THRESHOLD): gossip a delta to a fresh (off-chain) root with no
  // on-chain update. Its result id != chain.root, so honest members drop it.
  action rogueProposeDelta = {
    nondet victim = membersOf(rootKeys, chain.root).oneOf()
    val r = nextRoot
    all {
      membersOf(rootKeys, chain.root).size() > 1,
      rootKeys' = rootKeys.put(r, rootKeys.get(chain.root).keys().exclude(Set(victim)).mapBy(k => rootKeys.get(chain.root).get(k))),
      nextRoot' = r + 1,
      network' = network.union(Set({ tag: nextTag, env: DeltaMsg({ base: chain.root, result: r }) })),
      nextTag' = nextTag + 1,
      chain' = chain, local' = local, orgKnows' = orgKnows, objToken' = objToken,
      tokenKnows' = tokenKnows, revoked' = revoked, acceptedWrites' = acceptedWrites,
      clock' = clock, lastChecked' = lastChecked,
    }
  }

  // CLOCK: advance the global clock.
  action tick = all {
    clock < CLOCK_MAX,
    clock' = clock + 1,
    chain' = chain, local' = local, lastChecked' = lastChecked, rootKeys' = rootKeys,
    network' = network, orgKnows' = orgKnows, objToken' = objToken, tokenKnows' = tokenKnows,
    revoked' = revoked, acceptedWrites' = acceptedWrites, nextRoot' = nextRoot, nextTag' = nextTag,
  }

  // DEVICE: observe the chain (trie read) without applying a delta; records the
  // clock value at which the device last looked.
  action deviceObserveChain = {
    nondet d = DEVICES.oneOf()
    all {
      lastChecked' = lastChecked.put(d, clock),
      chain' = chain, local' = local, clock' = clock, rootKeys' = rootKeys,
      network' = network, orgKnows' = orgKnows, objToken' = objToken, tokenKnows' = tokenKnows,
      revoked' = revoked, acceptedWrites' = acceptedWrites, nextRoot' = nextRoot, nextTag' = nextTag,
    }
  }

  action step = any {
    adminProposeRemoval,
    adminRotateKey,
    deviceFetchAndApply,
    memberReceiveOrgSecret,
    cgkaRotate,
    honestWrite,
    compromisedKeyWrite,
    deviceAcceptWrite,
    networkDrop,
    networkDuplicate,
    revokedReplay,
    revokedAttemptWrite,
    rogueProposeDelta,
    tick,
    deviceObserveChain,
  }

  // ---- Property 1: replay / fork safety ----
  val forkSafety =
    DEVICES.forall(d1 => DEVICES.forall(d2 =>
      (local.get(d1).epoch == local.get(d2).epoch)
        implies (local.get(d1).root == local.get(d2).root)))

  // ---- Property 2: revocation safety ----
  pure def isSettled(obj: str, ch: ChainState, lv: str -> DevView,
                     tok: str -> { id: int, epoch: int }, rk: int -> (str -> int)): bool =
    and {
      currentDevices(rk, ch.root).forall(d => lv.get(d).epoch == ch.epoch),
      tok.get(obj).epoch >= ch.epoch,
    }

  pure def chainInvalid(w: { obj: str, owner: str, gen: int, receiver: str, staleness: int },
                        ch: ChainState, rk: int -> (str -> int)): bool =
    not(membersOf(rk, ch.root).contains(w.owner)) or genOf(rk, ch.root, w.owner) != w.gen

  val tauWindow =
    acceptedWrites.forall(w => chainInvalid(w, chain, rootKeys) implies w.staleness < TAU)

  val revocationSafety =
    OBJECTS.forall(obj =>
      isSettled(obj, chain, local, objToken, rootKeys)
        implies tokenKnows.get(objToken.get(obj).id).intersect(revoked) == Set())

  // ---- Lemma: revoked never holds the current org secret ----
  val revokedExcludedFromOrgSecret =
    (chain.orgGen > 0)
      implies (getOrEmpty(orgKnows, chain.orgGen).intersect(revoked) == Set())

  // ---- Property: convergence under quiescence ----
  // A delta is deliverable if it can advance some device to the chain root:
  // its result is the chain root AND some device's local root is its base.
  pure def deliverableDelta(net: Set[TaggedEnv], lc: str -> DevView, ch: ChainState): bool =
    net.exists(te => match te.env {
      | DeltaMsg(dm) => dm.result == ch.root and DEVICES.exists(d => lc.get(d).root == dm.base)
      | _ => false
    })

  // The set of delta hops (base -> result) currently in flight.
  pure def deltaHops(net: Set[TaggedEnv]): Set[{ base: int, result: int }] =
    net.fold(Set(), (acc, te) => match te.env {
      | DeltaMsg(dm) => acc.union(Set({ base: dm.base, result: dm.result }))
      | _ => acc
    })

  // Roots reachable from `start` by chaining in-flight delta hops (transitive
  // closure, bounded by the number of distinct hops). Each step extends the
  // frontier by any hop whose base is already reachable.
  pure def reachableRoots(hops: Set[{ base: int, result: int }], start: int): Set[int] =
    hops.fold(Set(start), (reached, _ignore) =>
      reached.union(hops.filter(h => reached.contains(h.base)).map(h => h.result)))

  // A behind current device still has pending delivery work iff the in-flight
  // delta hops form a path from its local root all the way to the chain root --
  // i.e. it can be carried home by applying the present deltas (possibly multi-
  // hop, as later hops successively become chain.root). This refines the naive
  // single-hop deliverableDelta, which misses multi-hop catch-up: when the chain
  // advances several epochs while a device lags, the delta landing on chain.root
  // has a `base` the device skipped, yet the bridging hops are present.
  pure def pendingDelivery(net: Set[TaggedEnv], lc: str -> DevView,
                           ch: ChainState, rk: int -> (str -> int)): bool =
    val hops = deltaHops(net)
    currentDevices(rk, ch.root).exists(d =>
      lc.get(d).root != ch.root and reachableRoots(hops, lc.get(d).root).contains(ch.root))

  // A behind current device is "stranded" when its bridging delta path to the
  // chain root is NOT present -- the network adversary (networkDrop) irrevocably
  // dropped a delta it needed, and there is no re-broadcast action to recover it.
  // Such a state is NOT a legitimate quiescent terminal state for a convergence
  // (liveness) claim: it represents lost delivery work, not delivered-everything.
  // Convergence is asserted under fair/complete delivery, so we exclude stranded
  // states from `quiescent` rather than weaken the conclusion.
  pure def strandedBehindDevice(net: Set[TaggedEnv], lc: str -> DevView,
                                ch: ChainState, rk: int -> (str -> int)): bool =
    val hops = deltaHops(net)
    currentDevices(rk, ch.root).exists(d =>
      lc.get(d).root != ch.root and not(reachableRoots(hops, lc.get(d).root).contains(ch.root)))

  // Quiescent: a legitimate terminal state under complete delivery -- no delivery
  // is pending (single- or multi-hop) and no behind current device has been
  // stranded by a dropped delta. Equivalently: every current device has either
  // already reached chain.root or has no further in-flight work, and none is
  // wedged. This is NOT vacuous: the fully-converged state (all current devices
  // at chain.root, no advancing deltas) is quiescent and reachable (Task 9).
  val quiescent =
    not(deliverableDelta(network, local, chain))
      and not(pendingDelivery(network, local, chain, rootKeys))
      and not(strandedBehindDevice(network, local, chain, rootKeys))

  // Convergence: in a quiescent state, every current-member device already
  // holds the chain root and epoch (the system cannot wedge diverged-but-quiescent).
  //
  // NOTE: this is a bounded-safety quiescence property, accepted by the design as a
  // substitute for temporal liveness. Because `quiescent` excludes every behind
  // current device (pendingDelivery/strandedBehindDevice are exhaustive), the
  // implication is effectively definitional and carries no independent teeth. The
  // load-bearing proof is the `convergedReachable` witness in ods_instances.qnt,
  // which shows a quiescent fully-converged post-removal state is actually reachable.
  val convergence =
    quiescent implies
      currentDevices(rootKeys, chain.root).forall(d =>
        local.get(d).root == chain.root and local.get(d).epoch == chain.epoch)
}
