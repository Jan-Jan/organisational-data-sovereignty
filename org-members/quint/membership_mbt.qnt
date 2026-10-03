// -*- mode: Bluespec; -*-
/// Runnable state machine over `membership`, replayed against the crate by
/// org-members/tests/mbt_conformance.rs (quint-connect). Every model action
/// records itself in `lastAction` — a sum type whose tag is the action's name
/// and whose payload is its nondeterministic picks — because `quint test`
/// traces carry no `mbt::actionTaken`, and named scenarios must drive the
/// same driver as random runs. See docs/adr/2026-10-03-quint-conformance-gate.md.
module membership_mbt {
  import membership.* from "./membership"

  /// One requested member of a genesis ceremony: identifier, handle, and how
  /// many device keys its record is built with (0 and 5 are refused).
  type Seed = { id: str, h: str, devs: int }

  /// One producer-side operation, for honest deltas.
  type Op = { op: str, id: str, h: str, g: int }

  type Action =
    | Init({ seeds: List[Seed] })
    | AddMember({ id: str, h: str })
    | DeleteMember({ id: str })
    | UpdateHandle({ id: str, h: str })
    | UpdateNameSurname({ id: str, nm: str, sn: str })
    | RotateKey({ id: str, g: int })
    | AddDevice({ id: str, g: int })
    | DeleteDevice({ id: str, g: int })
    | Isolate({ id: str, g: int })
    | ApplyDelta({ kind: str, ops: List[Op], removed: Set[str], upserted: Set[Leaf], target: str })

  /// Whether an organisation exists (a failed genesis leaves none).
  var orgExists: bool
  /// The current trie under test (empty when no organisation exists).
  var trie: Snapshot
  /// The trie before the last successful change.
  var prev: Snapshot
  /// The error tag of the last attempted op, or "" if it succeeded.
  var lastError: str
  /// The action taken to reach this state, with its picks.
  var lastAction: Action

  /// Small finite universes so the simulator explores a bounded space.
  pure val IDS = Set("a", "b", "c")
  pure val HANDLES = Set("h1", "h2", "h3")
  /// Four generations, so a member can reach the device cap (MAX_DEVICES).
  pure val GENS = 0.to(3)
  pure val NAMES = Set("n", "n2")
  pure val SURNAMES = Set("s", "s2")

  /// The record a fresh member is admitted with.
  pure def mkLeaf(id: str, h: str): Leaf =
    { id: id, handle: h, skeleton: h, name: "n", surname: "s",
      pKey: { owner: id, gen: 0 }, devices: Set({ owner: id, gen: 0 }) }

  /// A genesis seed's record: `devs` device keys of generations 0..devs-1.
  pure def seedLeaf(s: Seed): Leaf =
    mkLeaf(s.id, s.h).with("devices", 0.to(s.devs - 1).map(g => { owner: s.id, gen: g }))

  /// The six orders of the three handles.
  pure val HANDLE_ORDERS: Set[List[str]] = Set(
    ["h1", "h2", "h3"], ["h1", "h3", "h2"], ["h2", "h1", "h3"],
    ["h2", "h3", "h1"], ["h3", "h1", "h2"], ["h3", "h2", "h1"])

  pure val NO_OPS: List[Op] = []
  pure val NO_IDS: Set[str] = Set()
  pure val NO_LEAVES: Set[Leaf] = Set()

  /// Apply one producer-side operation (the subset honest deltas draw from).
  pure def applyOp(s: Snapshot, o: Op): Result =
    if (o.op == "add") addMember(s, mkLeaf(o.id, o.h))
    else if (o.op == "delete") deleteMember(s, o.id)
    else if (o.op == "handle") updateHandle(s, o.id, o.h, o.h)
    else rotateKey(s, o.id, { owner: o.id, gen: o.g })

  pure def applyOps(s: Snapshot, ops: List[Op]): Result =
    ops.foldl(Ok(s), (acc, o) => match acc { | Err(e) => Err(e) | Ok(t) => applyOp(t, o) })

  /// Apply a Result: on Ok advance the trie and clear error; on Err keep the
  /// trie and record the tag.
  action commit(r: Result, a: Action): bool = all {
    match r {
      | Ok(s2) => all { trie' = s2, prev' = trie, lastError' = "" }
      | Err(e) => all { trie' = trie, prev' = prev, lastError' = e }
    },
    orgExists' = orgExists,
    lastAction' = a,
  }

  /// A mutation on a missing organisation is refused as the driver refuses
  /// it: there is no record to name a member in.
  action mutate(r: Result, a: Action): bool =
    if (orgExists) commit(r, a) else commit(Err("IdNotFound"), a)

  /// Genesis with an explicit member list — the deterministic form, used by
  /// named scenarios.
  action initWith(seeds: List[Seed]): bool =
    match genesis(seeds.foldl([], (acc, s) => acc.append(seedLeaf(s)))) {
      | Ok(s)  => all { orgExists' = true,  trie' = s,     prev' = s,     lastError' = "", lastAction' = Init({ seeds: seeds }) }
      | Err(e) => all { orgExists' = false, trie' = Map(), prev' = Map(), lastError' = e,  lastAction' = Init({ seeds: seeds }) }
    }

  action init = {
    // Member count weighted toward 2 and 3, so most traces have members.
    nondet nR = 0.to(9).oneOf()
    val n = if (nR == 0) 0 else if (nR == 1) 1 else if (nR <= 5) 2 else 3
    // Start from a valid ceremony (distinct identifiers, distinct handles,
    // one device each) and add at most one flaw: 6 in 10 ceremonies are valid.
    nondet hs = HANDLE_ORDERS.oneOf()
    nondet flaw = 0.to(9).oneOf()
    val valid = [{ id: "a", h: hs[0], devs: 1 },
                 { id: "b", h: hs[1], devs: 1 },
                 { id: "c", h: hs[2], devs: 1 }].slice(0, n)
    val seeds =
      if (n == 0) valid
      else if (flaw == 0 and n >= 2) valid.replaceAt(1, valid[1].with("id", valid[0].id))
      else if (flaw == 1 and n >= 2) valid.replaceAt(1, valid[1].with("h", valid[0].h))
      else if (flaw == 2) valid.replaceAt(n - 1, valid[n - 1].with("devs", 0))
      else if (flaw == 3) valid.replaceAt(n - 1, valid[n - 1].with("devs", 5))
      else valid
    initWith(seeds)
  }

  action doAddMember(id: str, h: str): bool =
    mutate(addMember(trie, mkLeaf(id, h)), AddMember({ id: id, h: h }))
  action doDeleteMember(id: str): bool =
    mutate(deleteMember(trie, id), DeleteMember({ id: id }))
  action doUpdateHandle(id: str, h: str): bool =
    mutate(updateHandle(trie, id, h, h), UpdateHandle({ id: id, h: h }))
  action doUpdateNameSurname(id: str, nm: str, sn: str): bool =
    mutate(updateNameSurname(trie, id, nm, sn), UpdateNameSurname({ id: id, nm: nm, sn: sn }))
  action doRotateKey(id: str, g: int): bool =
    mutate(rotateKey(trie, id, { owner: id, gen: g }), RotateKey({ id: id, g: g }))
  action doAddDevice(id: str, g: int): bool =
    mutate(addDevice(trie, id, { owner: id, gen: g }), AddDevice({ id: id, g: g }))
  action doDeleteDevice(id: str, g: int): bool =
    mutate(deleteDevice(trie, id, { owner: id, gen: 0 }, { owner: id, gen: g }), DeleteDevice({ id: id, g: g }))
  action doIsolate(id: str, g: int): bool =
    mutate(isolate(trie, id, { owner: id, gen: g }), Isolate({ id: id, g: g }))

  /// Apply a delta and verify the candidate. `target` "true": the expected
  /// root is the producer's (honest) or the candidate's own (malformed);
  /// "wrong": the expected root is the current record's, which differs from
  /// any successful candidate because every accepted delta changes the record.
  pure def applyAndVerify(s: Snapshot, d: Delta, expected: Snapshot, target: str): Result =
    match applyDelta(s, d) {
      | Err(e) => Err(e)
      | Ok(c)  => if (target == "true" and c == expected) Ok(c)
                  else if (target == "true") Err("VerificationFailed")
                  else if (c == s) Ok(c)
                  else Err("VerificationFailed")
    }

  /// An honest delta: the producer applies `ops` to the current record and
  /// computes the delta; the receiver applies it to the same record.
  action applyHonest(ops: List[Op], target: str): bool = {
    val produced = applyOps(trie, ops)
    val producerOk = match produced { | Ok(_) => true | Err(_) => false }
    val s2 = match produced { | Ok(t) => t | Err(_) => trie }
    all {
      orgExists, producerOk, s2 != trie,
      commit(applyAndVerify(trie, calculateDelta(trie, s2), s2, target),
             ApplyDelta({ kind: "honest", ops: ops, removed: NO_IDS, upserted: NO_LEAVES, target: target })),
    }
  }

  /// A stale delta: computed against the previous record, applied to the current.
  action applyStale(target: str): bool = all {
    orgExists, prev != trie,
    commit(applyAndVerify(trie, calculateDelta(prev, trie), trie, target),
           ApplyDelta({ kind: "stale", ops: NO_OPS, removed: NO_IDS, upserted: NO_LEAVES, target: target })),
  }

  /// A delta built directly from removals and upserts at the current base.
  action applyBuilt(removed: Set[str], upserted: Set[Leaf], target: str): bool = all {
    orgExists,
    // One record per identifier: the crate's canonical order cannot hold two.
    upserted.map(l => l.id).size() == upserted.size(),
    val d = { baseRoot: trie, removed: removed, upserted: upserted }
    val expected = match applyDelta(trie, d) { | Ok(c) => c | Err(_) => trie }
    commit(applyAndVerify(trie, d, expected, target),
           ApplyDelta({ kind: "built", ops: NO_OPS, removed: removed, upserted: upserted, target: target })),
  }

  pure val OPS: Set[Op] =
    tuples(Set("add", "delete", "handle", "rotate"), IDS, HANDLES, GENS)
      .map(((o, i, h, g)) => { op: o, id: i, h: h, g: g })

  action ApplyDeltaStep = {
    // Integer draws, because a Set drops duplicates: honest 2/4, stale 1/4,
    // built 1/4; target "true" 3/4.
    nondet kindR = 0.to(3).oneOf()
    nondet targetR = 0.to(3).oneOf()
    val kind = if (kindR <= 1) "honest" else if (kindR == 2) "stale" else "built"
    val target = if (targetR <= 2) "true" else "wrong"
    nondet nOps = Set(1, 2).oneOf()
    nondet o1 = OPS.oneOf()
    nondet o2 = OPS.oneOf()
    nondet removed = IDS.powerset().filter(r => r.size() <= 2).oneOf()
    nondet upIds = IDS.powerset().filter(u => u.size() <= 2).oneOf()
    nondet upH = HANDLES.oneOf()
    nondet keepCurrent = Set(true, false).oneOf()
    val upserted = upIds.map(i =>
      if (keepCurrent and trie.keys().contains(i)) trie.get(i) else mkLeaf(i, upH))
    if (kind == "honest") applyHonest([o1, o2].slice(0, nOps), target)
    else if (kind == "stale") applyStale(target)
    else applyBuilt(removed, upserted, target)
  }

  action step = any {
    nondet id = IDS.oneOf()
    nondet h = HANDLES.oneOf()
    doAddMember(id, h),
    nondet id = IDS.oneOf()
    doDeleteMember(id),
    nondet id = IDS.oneOf()
    nondet h = HANDLES.oneOf()
    doUpdateHandle(id, h),
    nondet id = IDS.oneOf()
    nondet nm = NAMES.oneOf()
    nondet sn = SURNAMES.oneOf()
    doUpdateNameSurname(id, nm, sn),
    nondet id = IDS.oneOf()
    nondet g = GENS.oneOf()
    doRotateKey(id, g),
    nondet id = IDS.oneOf()
    nondet g = GENS.oneOf()
    doAddDevice(id, g),
    nondet id = IDS.oneOf()
    nondet g = GENS.oneOf()
    doDeleteDevice(id, g),
    nondet id = IDS.oneOf()
    nondet g = GENS.oneOf()
    doIsolate(id, g),
    // Listed three times: deltas are the cross-process path and the richest
    // in error branches, so they get three of eleven draws.
    ApplyDeltaStep,
    ApplyDeltaStep,
    ApplyDeltaStep,
  }

  // ---- Named scenarios (decision 12): replayed against the crate by
  // `run_scenario`, one test function each in mbt_conformance.rs.

  pure val TWO: List[Seed] = [{ id: "a", h: "h1", devs: 1 }, { id: "b", h: "h2", devs: 1 }]
  pure def hop(id: str, h: str): Op = { op: "handle", id: id, h: h, g: 0 }

  // One happy path per model action.
  run scenarioGenesis = initWith(TWO)
  // Every record is built before genesis sees any: a later record's bad
  // device list wins over an earlier duplicate identifier.
  run scenarioGenesisDeviceErrorFirst =
    initWith([{ id: "a", h: "h1", devs: 1 }, { id: "a", h: "h2", devs: 0 }])
  run scenarioAddMember = initWith(TWO).then(doAddMember("c", "h3"))
  run scenarioDeleteMember = initWith(TWO).then(doDeleteMember("b"))
  run scenarioUpdateHandle = initWith(TWO).then(doUpdateHandle("a", "h3"))
  run scenarioUpdateNameSurname = initWith(TWO).then(doUpdateNameSurname("a", "n2", "s2"))
  run scenarioRotateKey = initWith(TWO).then(doRotateKey("a", 1))
  run scenarioAddDevice = initWith(TWO).then(doAddDevice("a", 1))
  run scenarioDeleteDevice = initWith(TWO).then(doAddDevice("a", 1)).then(doDeleteDevice("a", 1))
  run scenarioIsolate = initWith(TWO).then(doIsolate("a", 1))
  run scenarioApplyHonest = initWith(TWO).then(applyHonest([hop("a", "h3")], "true"))
  run scenarioApplyStale = initWith(TWO).then(doRotateKey("a", 1)).then(applyStale("true"))
  run scenarioApplyVerifyFail = initWith(TWO).then(applyHonest([hop("a", "h3")], "wrong"))

  // PR-vf5hdm: a handle moved between two present members in one delta, in
  // both identifier orders, and a two-way swap (REQ-wx3wpv, LLR-n5t6bn).
  run scenarioHandoverAtoB = initWith(TWO).then(applyHonest([hop("a", "h3"), hop("b", "h1")], "true"))
  run scenarioHandoverBtoA = initWith(TWO).then(applyHonest([hop("b", "h3"), hop("a", "h2")], "true"))
  run scenarioHandleSwap = initWith(TWO).then(applyHonest([hop("a", "h3"), hop("b", "h1"), hop("a", "h2")], "true"))
  // The same handover to a member the delta admits (the random run's seed
  // 0xa68bbc85: [handle c->h3, add b h1]).
  run scenarioHandoverToNewMember =
    initWith([{ id: "a", h: "h2", devs: 1 }, { id: "c", h: "h1", devs: 1 }])
      .then(applyHonest([hop("c", "h3"), { op: "add", id: "b", h: "h1", g: 0 }], "true"))

  // Error branches random runs reach rarely (measured in the plan, T6).
  run scenarioDeviceSlotsFull = initWith(TWO)
    .then(doAddDevice("a", 1)).then(doAddDevice("a", 2)).then(doAddDevice("a", 3))
    .then(doAddDevice("a", 0))
  run scenarioRemoveUpsertOverlap = initWith(TWO)
    .then(applyBuilt(Set("a"), Set(mkLeaf("a", "h3")), "true"))
  // A member both removed and re-upserted unchanged: the crate reports the
  // no-op upsert before the overlap.
  run scenarioNoOpBeforeOverlap = initWith(TWO)
    .then(applyBuilt(Set("a"), Set(mkLeaf("a", "h1")), "true"))
  run scenarioBuiltVerifyFail = initWith(TWO)
    .then(applyBuilt(Set("b"), Set(mkLeaf("c", "h3")), "wrong"))

  /// Sanity invariants the simulator should never violate.
  val mbtInv = and {
    skeletonsUnique(trie),
    deviceCapOk(trie),
    orgExists or trie == Map(),
  }
}
