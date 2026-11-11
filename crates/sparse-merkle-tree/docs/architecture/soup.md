# SOUP Inventory

<!--
Software Of Unknown Provenance (IEC 62304 §8.1.2): every third-party
component the software depends on. Keep versions exact; review this file
whenever a dependency changes. Note functional/performance requirements the
SOUP must meet and known anomalies relevant to safety.
-->

**As built on 2026-10-25**: diffed against
`crates/sparse-merkle-tree/Cargo.toml` and the workspace `Cargo.lock`; the
versions below are the locked ones. The dev-dependencies (`blake3`,
`postcard`, `serde_json`, `proptest`) are test tooling and ship in no build.

| Name | Version | Role in system | Requirements it supports | Risk considerations / known anomalies |
|---|---|---|---|---|
| `spin` | 0.9.9 | `spin::Once`, the write-once cell holding a node's computed hash. Moves here from org-members, whose store this unit takes over. | REQ-uy4euz, REQ-3szczq (SDD-rh9u6z; LLR-3y5pq6) | **Safety-relevant.** LLR-3y5pq6 requires that a node's hash be a write-once cell and that the root be refused while any cell is unset; `Once` is what makes "write-once" true, including under concurrent access. A defect in its synchronisation would show as a stale or torn hash: a root, or a proof, made from hashes that are not the tree's. Spinlock-based, so a contended cell busy-waits rather than parking; acceptable at one short hash computation per node, and unavoidable in `no_std`. 0.9.9 is pinned while later versions exist, as in org-members; the first candidate for a reviewed bump. `default-features = false, features = ["once"]`. |
| `serde` | 1.0.229 | Serialisation of `NodeHash` and of both proof types, every `Deserialize` routed through the validating `from_parts`. Optional, behind the `serde` feature. | REQ-dyjq33, REQ-ymgg6b (SDD-ur6uvu, SDD-x43d2z; LLR-77zy97, LLR-jd4phg) | A derived `Deserialize` on a proof would bypass its bounds; the design forbids it (LLR-77zy97), and the decoding tests check each type. `default-features = false, features = ["derive", "alloc"]`. |
