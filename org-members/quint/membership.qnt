// -*- mode: Bluespec; -*-
/// Pure membership-trie semantics for the ODS Phase 1 Quint model.
/// Mirrors the `org-members` crate under the snapshot-as-root abstraction:
/// a RootHash is modeled as the canonical member map itself.
module membership {
  /// The boundary types live in the exported model interface.
  import membership_types.* from "./membership_types"
  export membership_types.*

  /// Crate constant: members cap at 4 devices.
  pure val MAX_DEVICES = 4

  /// Every key a leaf holds: its member key and its devices. A member key and
  /// a device key that are equal records are the same key (crate: same bytes).
  pure def leafKeys(l: Leaf): Set[Key] = l.devices.union(Set(l.pKey))

  /// Every key held anywhere in the organisation.
  pure def heldKeys(s: Snapshot): Set[Key] =
    s.keys().fold(Set(), (acc, k) => acc.union(leafKeys(s.get(k))))

  /// True iff `l` holds one key in two places: its member key among its
  /// devices.
  pure def leafSharesKey(l: Leaf): bool = l.devices.contains(l.pKey)

  /// LLR-v6gfc7's invariant: each key held in exactly one place -- no two
  /// members share a key (member or device), and no member key equals one of
  /// its own devices.
  pure def keysUnique(s: Snapshot): bool =
    s.keys().forall(k =>
      not(leafSharesKey(s.get(k))) and
      s.keys().forall(k2 =>
        k == k2 or leafKeys(s.get(k)).intersect(leafKeys(s.get(k2))) == Set()))

  /// True iff `skel` collides with any existing member's skeleton.
  pure def skeletonTaken(s: Snapshot, skel: str): bool =
    s.keys().exists(k => s.get(k).skeleton == skel)

  /// Insert a brand-new member. Mirrors OrgTrie::add_member: a leaf holding a
  /// key already held, or its member key among its own devices, is refused
  /// with "DuplicateKey" (LLR-v6gfc7). Check order: DuplicateId, handle
  /// checks, (leaf construction: EmptyDeviceList, DeviceSlotsFull),
  /// DuplicateKey.
  pure def addMember(s: Snapshot, l: Leaf): Result =
    if (s.keys().contains(l.id))
      Err("DuplicateId")
    else if (skeletonTaken(s, l.skeleton))
      Err("ConfusableHandle")
    else if (l.devices.size() == 0)
      Err("EmptyDeviceList")
    else if (l.devices.size() > MAX_DEVICES)
      Err("DeviceSlotsFull")
    else if (leafSharesKey(l) or leafKeys(l).intersect(heldKeys(s)) != Set())
      Err("DuplicateKey")
    else
      Ok(s.put(l.id, l))

  /// The device-list error a member record is refused with when it is built,
  /// or "" when its device list is acceptable. Mirrors `MemberLeaf::new`
  /// (empty list) and `P2pDeviceSlots::parse` (over the cap).
  pure def leafDeviceError(l: Leaf): str =
    if (l.devices.size() == 0) "EmptyDeviceList"
    else if (l.devices.size() > MAX_DEVICES) "DeviceSlotsFull"
    else ""

  /// Build a trie from a list of leaves (genesis ceremony). Every record is
  /// built before `OrgTrie::genesis` sees any of them, so the first record in
  /// list order with a bad device list wins over every identifier or handle
  /// collision; only then does the fold over addMember run, and its first
  /// error short-circuits.
  pure def genesis(leaves: List[Leaf]): Result =
    val deviceErr = leaves.foldl("", (acc, l) => if (acc != "") acc else leafDeviceError(l))
    if (deviceErr != "")
      Err(deviceErr)
    else
      leaves.foldl(Ok(Map()), (acc, l) =>
        match acc {
          | Err(e) => Err(e)
          | Ok(s)  => addMember(s, l)
        })

  // ---- helpers for tests ----
  /// A valid leaf: member key `(id, 0)`, one device `(id, 1)`. A member key
  /// and a device key with the same (owner, gen) are the same key.
  pure def leaf(idArg: str, handleArg: str, skel: str): Leaf = {
    id: idArg, handle: handleArg, skeleton: skel,
    name: "n", surname: "s",
    pKey: { owner: idArg, gen: 0 },
    devices: Set({ owner: idArg, gen: 1 }),
  }

  pure val emptyTrie: Snapshot = Map()

  run addMemberOkTest = {
    val r = addMember(emptyTrie, leaf("a", "alice", "alice"))
    assert(r == Ok(Map("a" -> leaf("a", "alice", "alice"))))
  }

  run addMemberDuplicateIdTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    assert(addMember(base, leaf("a", "alice2", "alice2")) == Err("DuplicateId"))
  }

  run addMemberConfusableTest = {
    val base = Map("a" -> leaf("a", "alice", "skel"))
    assert(addMember(base, leaf("b", "bob", "skel")) == Err("ConfusableHandle"))
  }

  pure def deleteMember(s: Snapshot, id: str): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else
      Ok(s.keys().exclude(Set(id)).mapBy(k => s.get(k)))

  /// True iff `skel` collides with any member OTHER than `selfId`.
  pure def skeletonTakenByOther(s: Snapshot, selfId: str, skel: str): bool =
    s.keys().exists(k => k != selfId and s.get(k).skeleton == skel)

  pure def updateHandle(s: Snapshot, id: str, newHandle: str, newSkel: str): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else if (skeletonTakenByOther(s, id, newSkel))
      Err("ConfusableHandle")
    else
      Ok(s.put(id, s.get(id).with("handle", newHandle).with("skeleton", newSkel)))

  pure def updateNameSurname(s: Snapshot, id: str, newName: str, newSurname: str): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else
      Ok(s.put(id, s.get(id).with("name", newName).with("surname", newSurname)))

  run deleteMemberOkTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    assert(deleteMember(base, "a") == Ok(Map()))
  }

  run deleteMemberMissingTest = {
    assert(deleteMember(emptyTrie, "ghost") == Err("IdNotFound"))
  }

  run updateHandleOkTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val exp  = base.put("a", base.get("a").with("handle", "alice2").with("skeleton", "alice2"))
    assert(updateHandle(base, "a", "alice2", "alice2") == Ok(exp))
  }

  run updateHandleConfusableTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"), "b" -> leaf("b", "bob", "bob"))
    assert(updateHandle(base, "a", "bob2", "bob") == Err("ConfusableHandle"))
  }

  run updateNameSurnameOkTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val exp  = base.put("a", base.get("a").with("name", "A").with("surname", "B"))
    assert(updateNameSurname(base, "a", "A", "B") == Ok(exp))
  }

  /// Rotate the member's key. Mirrors OrgTrie::rotate_p2p_key: a replacement
  /// key equal to the current key replaces nothing, so it is refused with
  /// "P2pKeyNotReplaced" (LLR-k89ahd). A replacement key held anywhere in the
  /// organisation before the operation -- another member's key or any enrolled
  /// device key, this member's own devices included -- is refused with
  /// "DuplicateKey" (LLR-fym7dy). Either refusal changes nothing. Check order:
  /// IdNotFound, P2pKeyNotReplaced, DuplicateKey. A key no longer held is not
  /// refused (the model, like the crate, keeps no key history).
  pure def rotateKey(s: Snapshot, id: str, newKey: Key): Result =
    if (not(s.keys().contains(id))) Err("IdNotFound")
    else if (s.get(id).pKey == newKey) Err("P2pKeyNotReplaced")
    else if (heldKeys(s).contains(newKey)) Err("DuplicateKey")
    else Ok(s.put(id, s.get(id).with("pKey", newKey)))

  /// Enrol a device. Mirrors OrgTrie::add_p2p_device: a device key already
  /// held -- under another member, or equal to any member key, this member's
  /// own included -- is refused with "DuplicateKey" (LLR-v6gfc7); the refusal
  /// changes nothing. Check order: IdNotFound, DeviceSlotsFull,
  /// DuplicateDevice, DuplicateKey.
  pure def addDevice(s: Snapshot, id: str, d: Key): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else if (s.get(id).devices.size() >= MAX_DEVICES)
      Err("DeviceSlotsFull")
    else if (s.get(id).devices.contains(d))
      Err("DuplicateDevice")
    else if (heldKeys(s).contains(d))
      Err("DuplicateKey")
    else
      Ok(s.put(id, s.get(id).with("devices", s.get(id).devices.union(Set(d)))))

  /// Remove one device and rotate the member's key. Mirrors
  /// OrgTrie::delete_p2p_device: a replacement key equal to the current key
  /// would leave the removed device holding a live key, so it is refused with
  /// "P2pKeyNotReplaced". A replacement key held anywhere in the organisation
  /// before the operation -- another member's key or any enrolled device key,
  /// the key of the device being removed included -- is refused with
  /// "DuplicateKey" (LLR-fym7dy). Either refusal changes nothing. Check order:
  /// IdNotFound, DeviceNotFound, P2pKeyNotReplaced, DuplicateKey.
  pure def deleteDevice(s: Snapshot, id: str, d: Key, newKey: Key): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else if (not(s.get(id).devices.contains(d)))
      Err("DeviceNotFound")
    else if (s.get(id).pKey == newKey)
      Err("P2pKeyNotReplaced")
    else if (heldKeys(s).contains(newKey))
      Err("DuplicateKey")
    else
      Ok(s.put(id, s.get(id)
        .with("devices", s.get(id).devices.exclude(Set(d)))
        .with("pKey", newKey)))

  /// Remove every device and rotate the member's key. Mirrors
  /// OrgTrie::emergency_isolate_member: a replacement key equal to the current
  /// key would leave the removed devices holding a live key, so it is refused
  /// with "P2pKeyNotReplaced". A replacement key held anywhere in the
  /// organisation before the operation -- another member's key or any enrolled
  /// device key, the keys of the devices being removed included -- is refused
  /// with "DuplicateKey" (LLR-fym7dy). Either refusal changes nothing. Check
  /// order: IdNotFound, P2pKeyNotReplaced, DuplicateKey.
  pure def isolate(s: Snapshot, id: str, newKey: Key): Result =
    if (not(s.keys().contains(id)))
      Err("IdNotFound")
    else if (s.get(id).pKey == newKey)
      Err("P2pKeyNotReplaced")
    else if (heldKeys(s).contains(newKey))
      Err("DuplicateKey")
    else
      Ok(s.put(id, s.get(id).with("devices", Set()).with("pKey", newKey)))

  run rotateKeyOkTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val nk = { owner: "a", gen: 2 }
    assert(rotateKey(base, "a", nk) == Ok(base.put("a", base.get("a").with("pKey", nk))))
  }

  run rotateKeySameKeyRejectedTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    assert(rotateKey(base, "a", base.get("a").pKey) == Err("P2pKeyNotReplaced"))
  }

  run addDeviceOkTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val d = { owner: "a-d2", gen: 0 }
    val exp = base.put("a", base.get("a").with("devices", base.get("a").devices.union(Set(d))))
    assert(addDevice(base, "a", d) == Ok(exp))
  }

  run addDeviceDuplicateTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    assert(addDevice(base, "a", { owner: "a", gen: 1 }) == Err("DuplicateDevice"))
  }

  run deleteDeviceRotatesKeyTest = {
    val d2 = { owner: "a-d2", gen: 0 }
    val base = Map("a" -> leaf("a", "alice", "alice").with("devices", Set({ owner: "a", gen: 1 }, d2)))
    val nk = { owner: "a", gen: 2 }
    val exp = base.put("a", base.get("a").with("devices", Set({ owner: "a", gen: 1 })).with("pKey", nk))
    assert(deleteDevice(base, "a", d2, nk) == Ok(exp))
  }

  run isolateRemovesAllAndRotatesTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val nk = { owner: "a", gen: 2 }
    val exp = base.put("a", base.get("a").with("devices", Set()).with("pKey", nk))
    assert(isolate(base, "a", nk) == Ok(exp))
  }

  run deleteDeviceSameKeyRejectedTest = {
    val d2 = { owner: "a-d2", gen: 0 }
    val base = Map("a" -> leaf("a", "alice", "alice").with("devices", Set({ owner: "a", gen: 1 }, d2)))
    assert(deleteDevice(base, "a", d2, base.get("a").pKey) == Err("P2pKeyNotReplaced"))
  }

  run isolateSameKeyRejectedTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    assert(isolate(base, "a", base.get("a").pKey) == Err("P2pKeyNotReplaced"))
  }

  /// Check order: an unknown device is reported before an unchanged key.
  run deleteDeviceUnknownBeforeSameKeyTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val ghost = { owner: "a-ghost", gen: 0 }
    assert(deleteDevice(base, "a", ghost, base.get("a").pKey) == Err("DeviceNotFound"))
  }

  /// Ids present in `oldS` but absent in `newS`.
  pure def removedIds(oldS: Snapshot, newS: Snapshot): Set[str] =
    oldS.keys().filter(k => not(newS.keys().contains(k)))

  /// Leaves in `newS` that are new or changed vs `oldS` (observable change).
  pure def upsertedLeaves(oldS: Snapshot, newS: Snapshot): Set[Leaf] =
    newS.keys()
      .filter(k => not(oldS.keys().contains(k)) or oldS.get(k) != newS.get(k))
      .map(k => newS.get(k))

  pure def calculateDelta(oldS: Snapshot, newS: Snapshot): Delta = {
    baseRoot: oldS,
    removed: removedIds(oldS, newS),
    upserted: upsertedLeaves(oldS, newS),
  }

  /// Canonical-form acceptance + application. Mirrors apply_delta, in the
  /// crate's check order: base must match; removed subseteq base; upserts
  /// observable; removed/upserted disjoint; post-state skeletons unique (over
  /// the whole set, independent of identifier order — LLR-n5t6bn); device caps
  /// hold; then post-state keys unique, else "DuplicateKey" (LLR-gjj6bx). The
  /// cap branch is unreachable from the crate, whose member records cannot
  /// hold more than MAX_DEVICES keys by type. Only the resulting record is
  /// checked for keys, not keys held in the base: a delta collapses a sequence
  /// of direct operations, each judged on the record it ran on, and a key may
  /// move or swap between members within one delta.
  pure def applyDelta(s: Snapshot, d: Delta): Result =
    val upsertIds = d.upserted.map(l => l.id)
    if (d.baseRoot != s)
      Err("DeltaBaseMismatch")
    else if (not(d.removed.subseteq(s.keys())))
      Err("StaleRemoval")
    else if (d.upserted.exists(l => s.keys().contains(l.id) and s.get(l.id) == l))
      Err("NoOpUpsert")
    else if (d.removed.intersect(upsertIds) != Set())
      Err("RemoveUpsertOverlap")
    else
      val afterRemove = s.keys().exclude(d.removed).mapBy(k => s.get(k))
      val afterUpsert = d.upserted.fold(afterRemove, (acc, l) => acc.put(l.id, l))
      if (afterUpsert.keys().exists(k =>
            afterUpsert.keys().exists(k2 =>
              k != k2 and afterUpsert.get(k).skeleton == afterUpsert.get(k2).skeleton)))
        Err("ConfusableHandle")
      else if (afterUpsert.keys().exists(k => afterUpsert.get(k).devices.size() > MAX_DEVICES))
        Err("DeviceSlotsFull")
      else if (not(keysUnique(afterUpsert)))
        Err("DuplicateKey")
      else
        Ok(afterUpsert)

  run calcDeltaIdentityTest = {
    val s = Map("a" -> leaf("a", "alice", "alice"))
    val d = calculateDelta(s, s)
    assert(d.removed == Set() and d.upserted == Set())
  }

  run roundTripAddTest = {
    val s  = Map("a" -> leaf("a", "alice", "alice"))
    val s2 = Map("a" -> leaf("a", "alice", "alice"), "b" -> leaf("b", "bob", "bob"))
    assert(applyDelta(s, calculateDelta(s, s2)) == Ok(s2))
  }

  run roundTripRemoveTest = {
    val s  = Map("a" -> leaf("a", "alice", "alice"), "b" -> leaf("b", "bob", "bob"))
    val s2 = Map("a" -> leaf("a", "alice", "alice"))
    assert(applyDelta(s, calculateDelta(s, s2)) == Ok(s2))
  }

  run roundTripModifyTest = {
    val s  = Map("a" -> leaf("a", "alice", "alice"))
    val s2 = Map("a" -> leaf("a", "alice", "alice").with("name", "Alice2"))
    assert(applyDelta(s, calculateDelta(s, s2)) == Ok(s2))
  }

  run applyStaleBaseTest = {
    val s  = Map("a" -> leaf("a", "alice", "alice"))
    val other = Map("z" -> leaf("z", "zed", "zed"))
    val d = { baseRoot: other, removed: Set(), upserted: Set(leaf("b","bob","bob")) }
    assert(applyDelta(s, d) == Err("DeltaBaseMismatch"))
  }

  pure def skeletonsUnique(s: Snapshot): bool =
    s.keys().forall(k =>
      s.keys().forall(k2 => k == k2 or s.get(k).skeleton != s.get(k2).skeleton))

  pure def deviceCapOk(s: Snapshot): bool =
    s.keys().forall(k => s.get(k).devices.size() <= MAX_DEVICES)

  run skeletonsUniqueHoldsTest = {
    assert(skeletonsUnique(Map("a" -> leaf("a","alice","alice"), "b" -> leaf("b","bob","bob"))))
  }

  run skeletonsUniqueViolatedTest = {
    assert(not(skeletonsUnique(Map("a" -> leaf("a","alice","x"), "b" -> leaf("b","bob","x")))))
  }

  run deviceCapHoldsTest = {
    assert(deviceCapOk(Map("a" -> leaf("a","alice","alice"))))
  }

  // ---- DuplicateKey: each key held in exactly one place ----

  /// Two members, a and b: keys (a,0) (a,1) (b,0) (b,1) are held.
  pure val twoMembers: Snapshot =
    Map("a" -> leaf("a", "alice", "alice"), "b" -> leaf("b", "bob", "bob"))

  // verifies: LLR-v6gfc7 (addMember, genesis)
  run addMemberDuplicateMemberKeyTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val l = leaf("b", "bob", "bob").with("pKey", { owner: "a", gen: 0 })
    assert(addMember(base, l) == Err("DuplicateKey"))
  }

  run addMemberMemberKeyIsHeldDeviceTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val l = leaf("b", "bob", "bob").with("pKey", { owner: "a", gen: 1 })
    assert(addMember(base, l) == Err("DuplicateKey"))
  }

  run addMemberDeviceIsHeldMemberKeyTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val l = leaf("b", "bob", "bob").with("devices", Set({ owner: "a", gen: 0 }))
    assert(addMember(base, l) == Err("DuplicateKey"))
  }

  run addMemberOwnKeyAsDeviceTest = {
    val l = leaf("a", "alice", "alice").with("devices", Set({ owner: "a", gen: 0 }))
    assert(addMember(emptyTrie, l) == Err("DuplicateKey"))
  }

  /// Check order: DuplicateId before DuplicateKey.
  run addMemberDuplicateIdBeforeDuplicateKeyTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    assert(addMember(base, leaf("a", "alice2", "alice2")) == Err("DuplicateId"))
  }

  /// Check order: handle checks before DuplicateKey.
  run addMemberConfusableBeforeDuplicateKeyTest = {
    val base = Map("a" -> leaf("a", "alice", "alice"))
    val l = leaf("b", "bob", "alice").with("pKey", { owner: "a", gen: 0 })
    assert(addMember(base, l) == Err("ConfusableHandle"))
  }

  run genesisDistinctKeysOkTest = {
    assert(genesis([leaf("a", "alice", "alice"), leaf("b", "bob", "bob")]) == Ok(twoMembers))
  }

  run genesisDuplicateKeyTest = {
    val b = leaf("b", "bob", "bob").with("devices", Set({ owner: "a", gen: 1 }))
    assert(genesis([leaf("a", "alice", "alice"), b]) == Err("DuplicateKey"))
  }

  /// Check order per member: DuplicateId before DuplicateKey.
  run genesisDuplicateIdBeforeDuplicateKeyTest = {
    assert(genesis([leaf("a", "alice", "alice"), leaf("a", "alice2", "alice2")]) == Err("DuplicateId"))
  }

  // verifies: LLR-fym7dy (rotateKey)
  run rotateKeyOtherMemberKeyTest = {
    assert(rotateKey(twoMembers, "a", { owner: "b", gen: 0 }) == Err("DuplicateKey"))
  }

  run rotateKeyOtherMemberDeviceTest = {
    assert(rotateKey(twoMembers, "a", { owner: "b", gen: 1 }) == Err("DuplicateKey"))
  }

  run rotateKeyOwnDeviceTest = {
    assert(rotateKey(twoMembers, "a", { owner: "a", gen: 1 }) == Err("DuplicateKey"))
  }

  /// A key named after another member but held by nobody is accepted.
  run rotateKeyUnheldForeignKeyOkTest = {
    val nk = { owner: "b", gen: 2 }
    assert(rotateKey(twoMembers, "a", nk) == Ok(twoMembers.put("a", twoMembers.get("a").with("pKey", nk))))
  }

  /// Check order: IdNotFound before DuplicateKey.
  run rotateKeyIdNotFoundBeforeDuplicateKeyTest = {
    assert(rotateKey(twoMembers, "ghost", { owner: "a", gen: 0 }) == Err("IdNotFound"))
  }

  // rotateKeySameKeyRejectedTest above also fixes P2pKeyNotReplaced before
  // DuplicateKey: the member's current key is held (by itself).

  // verifies: LLR-v6gfc7 (addDevice)
  run addDeviceOtherMemberDeviceTest = {
    assert(addDevice(twoMembers, "a", { owner: "b", gen: 1 }) == Err("DuplicateKey"))
  }

  run addDeviceOtherMemberKeyTest = {
    assert(addDevice(twoMembers, "a", { owner: "b", gen: 0 }) == Err("DuplicateKey"))
  }

  run addDeviceOwnMemberKeyTest = {
    assert(addDevice(twoMembers, "a", { owner: "a", gen: 0 }) == Err("DuplicateKey"))
  }

  /// Member a with MAX_DEVICES devices.
  pure val fullA: Snapshot = Map("a" -> leaf("a", "alice", "alice").with("devices",
    Set({ owner: "a", gen: 1 }, { owner: "a", gen: 2 }, { owner: "c", gen: 1 }, { owner: "c", gen: 2 })))

  /// Check order: DeviceSlotsFull before DuplicateDevice (the crate's slot
  /// check runs first).
  run addDeviceFullBeforeDuplicateDeviceTest = {
    assert(addDevice(fullA, "a", { owner: "a", gen: 1 }) == Err("DeviceSlotsFull"))
  }

  /// Check order: DeviceSlotsFull before DuplicateKey.
  run addDeviceFullBeforeDuplicateKeyTest = {
    assert(addDevice(fullA, "a", { owner: "a", gen: 0 }) == Err("DeviceSlotsFull"))
  }

  // addDeviceDuplicateTest above fixes DuplicateDevice before DuplicateKey:
  // the member's own device is also a held key.

  // verifies: LLR-fym7dy (deleteDevice)
  /// Member a with devices (a,1) and (a,2), member b as in twoMembers.
  pure val aTwoDevices: Snapshot = twoMembers.put("a",
    twoMembers.get("a").with("devices", Set({ owner: "a", gen: 1 }, { owner: "a", gen: 2 })))

  run deleteDeviceOtherMemberKeyTest = {
    assert(deleteDevice(aTwoDevices, "a", { owner: "a", gen: 2 }, { owner: "b", gen: 0 }) == Err("DuplicateKey"))
  }

  /// The key of the device being removed counts as held.
  run deleteDeviceRemovedDeviceKeyTest = {
    assert(deleteDevice(aTwoDevices, "a", { owner: "a", gen: 2 }, { owner: "a", gen: 2 }) == Err("DuplicateKey"))
  }

  run deleteDeviceRemainingDeviceKeyTest = {
    assert(deleteDevice(aTwoDevices, "a", { owner: "a", gen: 2 }, { owner: "a", gen: 1 }) == Err("DuplicateKey"))
  }

  /// Check order: DeviceNotFound before DuplicateKey.
  run deleteDeviceUnknownBeforeDuplicateKeyTest = {
    assert(deleteDevice(aTwoDevices, "a", { owner: "c", gen: 0 }, { owner: "b", gen: 0 }) == Err("DeviceNotFound"))
  }

  // deleteDeviceSameKeyRejectedTest above fixes P2pKeyNotReplaced before
  // DuplicateKey.

  // verifies: LLR-fym7dy (isolate)
  run isolateOtherMemberKeyTest = {
    assert(isolate(twoMembers, "a", { owner: "b", gen: 1 }) == Err("DuplicateKey"))
  }

  /// The keys of the devices being removed count as held.
  run isolateRemovedDeviceKeyTest = {
    assert(isolate(twoMembers, "a", { owner: "a", gen: 1 }) == Err("DuplicateKey"))
  }

  /// Check order: IdNotFound before DuplicateKey.
  run isolateIdNotFoundBeforeDuplicateKeyTest = {
    assert(isolate(twoMembers, "ghost", { owner: "a", gen: 0 }) == Err("IdNotFound"))
  }

  // isolateSameKeyRejectedTest above fixes P2pKeyNotReplaced before
  // DuplicateKey.

  // verifies: LLR-gjj6bx (applyDelta, resulting record only)
  run applyDeltaDuplicateKeyTest = {
    val s = Map("a" -> leaf("a", "alice", "alice"))
    val b = leaf("b", "bob", "bob").with("pKey", { owner: "a", gen: 0 })
    assert(applyDelta(s, { baseRoot: s, removed: Set(), upserted: Set(b) }) == Err("DuplicateKey"))
  }

  run applyDeltaTwoUpsertsShareKeyTest = {
    val b = leaf("b", "bob", "bob").with("pKey", { owner: "c", gen: 0 })
    val c = leaf("c", "carol", "carol")
    assert(applyDelta(emptyTrie, { baseRoot: emptyTrie, removed: Set(), upserted: Set(b, c) }) == Err("DuplicateKey"))
  }

  run applyDeltaOwnKeyAsDeviceTest = {
    val s = Map("a" -> leaf("a", "alice", "alice"))
    val a2 = s.get("a").with("devices", Set({ owner: "a", gen: 0 }))
    assert(applyDelta(s, { baseRoot: s, removed: Set(), upserted: Set(a2) }) == Err("DuplicateKey"))
  }

  /// Result only: delete device (a,2) rotating to (a,3), then rotate to the
  /// removed device's key (a,2) -- each step accepted on its own record, so
  /// their collapsed delta is accepted too.
  run applyDeltaResultOnlyTest = {
    val s1 = aTwoDevices.put("a", aTwoDevices.get("a")
      .with("devices", Set({ owner: "a", gen: 1 })).with("pKey", { owner: "a", gen: 3 }))
    val s2 = s1.put("a", s1.get("a").with("pKey", { owner: "a", gen: 2 }))
    all {
      assert(deleteDevice(aTwoDevices, "a", { owner: "a", gen: 2 }, { owner: "a", gen: 3 }) == Ok(s1)),
      assert(rotateKey(s1, "a", { owner: "a", gen: 2 }) == Ok(s2)),
      assert(applyDelta(aTwoDevices, calculateDelta(aTwoDevices, s2)) == Ok(s2)),
    }
  }

  /// A key may move between two upserted leaves in one delta.
  run applyDeltaKeyMovesBetweenLeavesTest = {
    val s2 = twoMembers
      .put("a", twoMembers.get("a").with("pKey", { owner: "c", gen: 0 }))
      .put("b", twoMembers.get("b").with("pKey", { owner: "a", gen: 0 }))
    assert(applyDelta(twoMembers, calculateDelta(twoMembers, s2)) == Ok(s2))
  }

  /// Two members swap member keys in one delta: rotate a to an unheld key,
  /// rotate b to a's old key, rotate a to b's old key -- each step accepted on
  /// its own record, so the collapsed delta is accepted too.
  run applyDeltaMemberKeySwapTest = {
    val k3 = { owner: "c", gen: 0 }
    val s1 = twoMembers.put("a", twoMembers.get("a").with("pKey", k3))
    val s2 = s1.put("b", s1.get("b").with("pKey", { owner: "a", gen: 0 }))
    val s3 = s2.put("a", s2.get("a").with("pKey", { owner: "b", gen: 0 }))
    all {
      assert(rotateKey(twoMembers, "a", k3) == Ok(s1)),
      assert(rotateKey(s1, "b", { owner: "a", gen: 0 }) == Ok(s2)),
      assert(rotateKey(s2, "a", { owner: "b", gen: 0 }) == Ok(s3)),
      assert(applyDelta(twoMembers, calculateDelta(twoMembers, s3)) == Ok(s3)),
    }
  }

  /// A device key moves from b to a in one delta: b drops device (b,1)
  /// rotating to (b,2), a enrols (b,1) -- each step accepted on its own record.
  run applyDeltaDeviceKeyMovesTest = {
    val bWithTwo = twoMembers.put("b", twoMembers.get("b").with("devices", Set({ owner: "b", gen: 1 }, { owner: "b", gen: 3 })))
    val s1 = bWithTwo.put("b", bWithTwo.get("b")
      .with("devices", Set({ owner: "b", gen: 3 })).with("pKey", { owner: "b", gen: 2 }))
    val s2 = s1.put("a", s1.get("a").with("devices", Set({ owner: "a", gen: 1 }, { owner: "b", gen: 1 })))
    all {
      assert(deleteDevice(bWithTwo, "b", { owner: "b", gen: 1 }, { owner: "b", gen: 2 }) == Ok(s1)),
      assert(addDevice(s1, "a", { owner: "b", gen: 1 }) == Ok(s2)),
      assert(applyDelta(bWithTwo, calculateDelta(bWithTwo, s2)) == Ok(s2)),
    }
  }

  /// A removed member's keys are free in the resulting record.
  run applyDeltaRemovalFreesKeyTest = {
    val s2 = Map("b" -> leaf("b", "bob", "bob").with("pKey", { owner: "a", gen: 0 }))
    assert(applyDelta(twoMembers, calculateDelta(twoMembers, s2)) == Ok(s2))
  }

  /// Check order: handle checks before DuplicateKey.
  run applyDeltaConfusableBeforeDuplicateKeyTest = {
    val s = Map("a" -> leaf("a", "alice", "alice"))
    val b = leaf("b", "bob", "alice").with("pKey", { owner: "a", gen: 0 })
    assert(applyDelta(s, { baseRoot: s, removed: Set(), upserted: Set(b) }) == Err("ConfusableHandle"))
  }

  // verifies: LLR-v6gfc7 (the invariant)
  run keysUniqueHoldsTest = {
    assert(keysUnique(twoMembers))
  }

  run keysUniqueViolatedAcrossMembersTest = {
    assert(not(keysUnique(twoMembers.put("b", twoMembers.get("b").with("pKey", { owner: "a", gen: 1 })))))
  }

  run keysUniqueViolatedWithinLeafTest = {
    assert(not(keysUnique(Map("a" -> leaf("a", "alice", "alice").with("devices", Set({ owner: "a", gen: 0 }))))))
  }
}
