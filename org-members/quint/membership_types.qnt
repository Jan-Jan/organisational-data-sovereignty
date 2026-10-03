// -*- mode: Bluespec; -*-
/// Exported model interface of the org-members membership model: the types
/// that describe membership data at the unit boundary. Another unit's model
/// (org-node's protocol model) imports this module and nothing else from
/// org-members. See docs/adr/2026-10-03-quint-conformance-gate.md, decision 2.
module membership_types {
  /// A peer-to-peer key, modeled as (who minted it, rotation generation).
  /// No bytes, no crypto: rotation bumps `gen`. This lets later layers
  /// express "revoked insider still holds an old generation".
  type Key = { owner: str, gen: int }

  /// A member leaf. `skeleton` is the UTS#39 confusable-skeleton of the
  /// handle: two handles collide iff their skeletons are equal.
  type Leaf = {
    id: str,
    handle: str,
    skeleton: str,
    name: str,
    surname: str,
    pKey: Key,
    devices: Set[Key],
  }

  /// The trie. Doubles as the RootHash: equal maps == equal root.
  type Snapshot = str -> Leaf

  /// Result of a mutation. `Err`'s payload is an error tag matching the
  /// crate's `OrgMembersError` variant names where one applies.
  type Result =
    | Ok(Snapshot)
    | Err(str)

  type Delta = {
    baseRoot: Snapshot,
    removed: Set[str],
    upserted: Set[Leaf],
  }
}
