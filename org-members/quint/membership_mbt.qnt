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

  /// One producer-side operation, for honest deltas. A rotation's new key is
  /// `{ owner: ko, gen: g }`: `ko` may name another member, so honest deltas
  /// can move keys between members (LLR-gjj6bx).
  type Op = { op: str, id: str, h: str, ko: str, g: int }

  type Action =
    | Init({ seeds: List[Seed] })
    | AddMember({ id: str, h: str })
    | DeleteMember({ id: str })
    | UpdateHandle({ id: str, h: str })
    | UpdateNameSurname({ id: str, nm: str, sn: str })
    | RotateKey({ id: str, k: Key })
    | AddDevice({ id: str, k: Key })
    | DeleteDevice({ id: str, d: Key, k: Key })
    | Isolate({ id: str, k: Key })
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

  /// Every key the random run draws. `owner` only names a key: any member may
  /// be handed a key named after another id, so cross-member and
  /// member-vs-device collisions (DuplicateKey) occur.
  pure val KEYS: Set[Key] = tuples(IDS, GENS).map(((o, g)) => { owner: o, gen: g })

  /// The record a fresh member is admitted with: member key generation 0,
  /// initial device generation 1. A member key and a device key with the same
  /// (owner, gen) are one key, so they must differ.
  pure def mkLeaf(id: str, h: str): Leaf =
    { id: id, handle: h, skeleton: h, name: "n", surname: "s",
      pKey: { owner: id, gen: 0 }, devices: Set({ owner: id, gen: 1 }) }

  /// A genesis seed's record: `devs` device keys of generations 1..devs.
  pure def seedLeaf(s: Seed): Leaf =
    mkLeaf(s.id, s.h).with("devices", 1.to(s.devs).map(g => { owner: s.id, gen: g }))

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
    else rotateKey(s, o.id, { owner: o.ko, gen: o.g })

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
  action doRotateKey(id: str, k: Key): bool =
    mutate(rotateKey(trie, id, k), RotateKey({ id: id, k: k }))
  action doAddDevice(id: str, k: Key): bool =
    mutate(addDevice(trie, id, k), AddDevice({ id: id, k: k }))
  action doDeleteDevice(id: str, d: Key, k: Key): bool =
    mutate(deleteDevice(trie, id, d, k), DeleteDevice({ id: id, d: d, k: k }))
  action doIsolate(id: str, k: Key): bool =
    mutate(isolate(trie, id, k), Isolate({ id: id, k: k }))

  /// DeleteDevice restricted to a device the member holds, so the replacement
  /// key equals the removed device's key often enough to exercise that
  /// DuplicateKey case (uniform draws rarely hit it). Records itself as
  /// DeleteDevice: it is a bias of the generator, not another crate operation.
  action doDeleteEnrolledDevice = {
    val enrolled = IDS.filter(i => trie.keys().contains(i) and trie.get(i).devices != Set())
    all {
      enrolled != Set(),
      nondet id = enrolled.oneOf()
      nondet d = trie.get(id).devices.oneOf()
      nondet k = KEYS.oneOf()
      doDeleteDevice(id, d, k),
    }
  }

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
    tuples(Set("add", "delete", "handle", "rotate"), IDS, HANDLES, IDS, GENS)
      .map(((o, i, h, ko, g)) => { op: o, id: i, h: h, ko: ko, g: g })

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
    nondet k = KEYS.oneOf()
    doRotateKey(id, k),
    nondet id = IDS.oneOf()
    nondet k = KEYS.oneOf()
    doAddDevice(id, k),
    nondet id = IDS.oneOf()
    nondet d = KEYS.oneOf()
    nondet k = KEYS.oneOf()
    doDeleteDevice(id, d, k),
    doDeleteEnrolledDevice,
    nondet id = IDS.oneOf()
    nondet k = KEYS.oneOf()
    doIsolate(id, k),
    // Listed three times: deltas are the cross-process path and the richest
    // in error branches, so they get three of eleven draws.
    ApplyDeltaStep,
    ApplyDeltaStep,
    ApplyDeltaStep,
  }

  // ---- Named scenarios (decision 12): replayed against the crate by
  // `run_scenario`, one test function each in mbt_conformance.rs.

  pure val TWO: List[Seed] = [{ id: "a", h: "h1", devs: 1 }, { id: "b", h: "h2", devs: 1 }]
  pure def hop(id: str, h: str): Op = { op: "handle", id: id, h: h, ko: id, g: 0 }
  /// A rotation of `id`'s member key to `{ owner: ko, gen: g }`.
  pure def rop(id: str, ko: str, g: int): Op = { op: "rotate", id: id, h: "h1", ko: ko, g: g }
  pure def key(o: str, g: int): Key = { owner: o, gen: g }

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
  run scenarioRotateKey = initWith(TWO).then(doRotateKey("a", key("a", 2)))
  run scenarioAddDevice = initWith(TWO).then(doAddDevice("a", key("a", 2)))
  run scenarioDeleteDevice = initWith(TWO).then(doAddDevice("a", key("a", 2)))
    .then(doDeleteDevice("a", key("a", 1), key("a", 3)))
  run scenarioIsolate = initWith(TWO).then(doIsolate("a", key("a", 2)))
  run scenarioApplyHonest = initWith(TWO).then(applyHonest([hop("a", "h3")], "true"))
  run scenarioApplyStale = initWith(TWO).then(doRotateKey("a", key("a", 2))).then(applyStale("true"))
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
      .then(applyHonest([hop("c", "h3"), { op: "add", id: "b", h: "h1", ko: "b", g: 0 }], "true"))

  // Keys move between present members in one delta (LLR-gjj6bx: only the
  // resulting record is checked, so a lawful sequence's delta is accepted).
  // Two members swap member keys: rotate a to an unheld key, b to a's old
  // key, a to b's old key -- each step accepted on its own record.
  run scenarioMemberKeySwap = initWith(TWO)
    .then(applyHonest([rop("a", "a", 2), rop("b", "a", 0), rop("a", "b", 0)], "true"))
  // A device key moves from b to a: the record delete_p2p_device(b, (b,1),
  // (b,3)) then add_p2p_device(a, (b,1)) reaches, sent as one built delta.
  run scenarioDeviceKeyMoves = initWith(TWO)
    .then(doAddDevice("b", key("b", 2)))
    .then(applyBuilt(Set(), Set(
      mkLeaf("a", "h1").with("devices", Set(key("a", 1), key("b", 1))),
      mkLeaf("b", "h2").with("pKey", key("b", 3)).with("devices", Set(key("b", 2)))), "true"))

  // DuplicateKey on each direct path (LLR-v6gfc7, LLR-fym7dy), in the crate's
  // check order.
  run scenarioRotateKeyDuplicate = initWith(TWO).then(doRotateKey("a", key("b", 1)))
  run scenarioAddDeviceDuplicate = initWith(TWO).then(doAddDevice("a", key("b", 0)))
  run scenarioDeleteDeviceRemovedKey = initWith(TWO).then(doAddDevice("a", key("a", 2)))
    .then(doDeleteDevice("a", key("a", 2), key("a", 2)))
  run scenarioIsolateDuplicate = initWith(TWO).then(doIsolate("a", key("a", 1)))
  run scenarioAddMemberDuplicate = initWith(TWO).then(doRotateKey("a", key("c", 1)))
    .then(doAddMember("c", "h3"))
  // A built delta whose resulting record holds one key twice (LLR-gjj6bx).
  run scenarioApplyDeltaDuplicate = initWith(TWO)
    .then(applyBuilt(Set(), Set(mkLeaf("a", "h1").with("pKey", key("b", 0))), "true"))

  // Error branches random runs reach rarely (measured in the plan, T6).
  run scenarioDeviceSlotsFull = initWith(TWO)
    .then(doAddDevice("a", key("a", 2))).then(doAddDevice("a", key("a", 3))).then(doAddDevice("a", key("c", 0)))
    .then(doAddDevice("a", key("a", 0)))
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
    keysUnique(trie),
    orgExists or trie == Map(),
  }
}
