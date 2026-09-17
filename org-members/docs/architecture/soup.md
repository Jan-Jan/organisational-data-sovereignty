# SOUP Inventory

<!--
Software Of Unknown Provenance (IEC 62304 §8.1.2): every third-party
component the software depends on. Keep versions exact; review this file
whenever a dependency changes. Note functional/performance requirements the
SOUP must meet and known anomalies relevant to safety.
-->

## Evidence and provenance

Every version below is read from `/Cargo.lock` at the repository root, measured
on 2026-09-15.

`org-members` declares no `[workspace]` of its own, so it is a member of the
**root** workspace: `cargo locate-project --workspace`, run from
`org-members/`, names the repository root. The root lockfile is therefore the
one that governs what this unit builds against, and it is tracked as of this
change so that the versions in this table are reproducible rather than
incidental.

`org-members/Cargo.lock` used to be tracked and was **inert** — cargo never
read it, and it had drifted (`blake3` 1.8.5 against the 1.8.7 that builds, 121
packages against 797). It was deleted in this change. That removing it changes
no build is the measurement, not the claim: `cargo test -p org-members` reports
**109 passed, 0 failed** after the deletion, identical to the baseline before
it.

The closure was measured with
`cargo tree -p org-members --edges normal --prefix depth | sort -u`: **nine
direct dependencies over a transitive closure of 37 crates** (unique
`name@version`, `org-members` itself excluded, default features `std` +
`serde`).

## The `ed25519-dalek` pre-release — measured, not assumed

`/Cargo.lock` carries two versions of the signature library: `ed25519-dalek`
2.2.0 and `ed25519-dalek` 3.0.0-pre.6. **The pre-release is not in
`org-members`' dependency closure.** `cargo tree -i ed25519-dalek@3.0.0-pre.6`
resolves its only parents to `iroh` 0.98.2, `iroh-base` 0.98.0 and
`iroh-gossip` 0.98.0, reached from `p2panda-net` 0.6.0 into `spike-p2panda`,
and from `iroh` directly as a **dev-dependency** of `org-node`. No path reaches
`org-members`.

`org-members` resolves `ed25519-dalek` to **2.2.0** on the normal-edge path,
which is a stable release. The pre-release cryptographic dependency is a real
finding for the `org-node` and spike crates and belongs in their SOUP
inventories; it is recorded here only to close the question for this unit.

## Direct dependencies

| Name | Version | Role in system | Requirements it supports | Risk considerations / known anomalies |
|---|---|---|---|---|
| `blake3` | 1.8.7 | The only hash function in the unit. `org-members/src/hasher.rs` calls `blake3::keyed_hash` and `blake3::Hasher::new_keyed` with four distinct 32-byte domain keys (`member-leaf`, `member-node`, `device-leaf`, `device-node`), which is what keeps a member subtree and a device subtree from colliding. Every membership root value is a BLAKE3 output. | REQ-avmu3j, REQ-4umsuz (SDD-d6x85b, SDD-d9svdj; LLR-72p8bz, LLR-kdhd2v, LLR-wm5hpc, LLR-n7nya3) | **Safety-relevant.** A collision or a domain-separation defect here would let two different membership records report the same root, which is exactly the integrity that REQ-avmu3j and REQ-4umsuz rest on — the root is the only thing compared when a change set is accepted. Used `default-features = false` (no `std`, no Rayon, no SIMD dispatch beyond what the portable path selects), so the multithreading and memory-mapping surface is not compiled in. No advisory-database scan was run in this task (no `cargo-audit`/`cargo-deny` in the toolchain here); the version is pinned by the now-tracked lockfile, so the next dependency bump is a visible diff and is the point at which it should be re-reviewed. Note the drift this file replaces: the deleted `org-members/Cargo.lock` recorded 1.8.5, a version nothing builds with. **Supporting-item references corrected 2026-09-17** (independent review, finding-12): this row used to cite LLR-w5nkbu (the 128-byte `name`/`surname` bounds) and LLR-pys2ek (`MAX_DEVICES` is 4). Both resolve, so no gate complained, but neither is a property a hash function supports — they are a field-length rule and a capacity constant. They are replaced rather than dropped, because the row does support hashing items and naming none would be a worse record: LLR-72p8bz is the domain separation this row's own prose describes, LLR-kdhd2v is the device sub-trie's empty-slot sentinel and sorted keys, LLR-wm5hpc the per-level empty-subtree hashes, and LLR-n7nya3 the write-once hash cell. |
| `ed25519-dalek` | 2.2.0 | Supplies `VerifyingKey` only. `org-members/src/types.rs` wraps it as `P2pMemberKey` and `P2pDeviceKey`, and calls `VerifyingKey::from_bytes` on deserialisation so that 32 bytes off the wire are rejected unless they are a valid Edwards point. | REQ-avmu3j, REQ-4umsuz, REQ-shk82j (SDD-4yr9ge, SDD-d6x85b) | **Safety-relevant, with a narrow surface.** This crate is used for *point validation and byte round-tripping*, not for signing or verification: no `Signer`, no `Verifier`, no signature check anywhere in the crate (`grep` over `org-members/src` finds only `VerifyingKey`). Authentication of a change set is explicitly a non-responsibility of this unit (`org-members/src/delta.rs`, "What this crate does NOT do") and lives above it, so the historical dalek weaknesses in the *signing/verification* API — the pre-2.0 double-public-key oracle class, and the "verify with a caller-supplied public key" footgun — are outside the code paths used here. What remains load-bearing is that `from_bytes` rejects non-canonical and small-order-subgroup encodings; a defect there would let an invalid key enter the record and be hashed into the root. Built `default-features = false, features = ["alloc"]`, so `rand_core`-based key generation is not compiled in. The 3.0.0-pre.6 pre-release present elsewhere in the workspace is **not** reached from here (see above). |
| `unicode-security` | 0.1.2 | UTS#39 confusable skeletons. `org-members/src/types.rs` computes a handle's skeleton so that two handles which render alike cannot both be held in one organisation. | REQ-m8aexh (SDD-4yr9ge; LLR-5w2jx8, LLR-fv75ec, LLR-ch2pkw, LLR-mmst86) | **Safety-relevant, and the weakest link in this table.** REQ-m8aexh is a homograph-impersonation control, and this crate *is* the control — the rejection decision is whatever `unicode-security` says the skeleton is. Two concrete concerns. (1) **Maturity:** 0.1.2 is a pre-1.0, low-traffic crate; its API carries no stability guarantee and its release cadence is slow. (2) **Unicode-version drift:** a confusable skeleton is only as current as the confusables table compiled into the crate. A newer Unicode revision that adds a confusable pair the pinned table does not know about produces two handles that render alike and skeleton differently, and REQ-m8aexh admits both. That is a *residual* risk of the control, not a defect in this crate, and it is the reason the control is layered rather than sole (REQ-h5ret5 already refuses mixed-script and uppercase handles, which removes the largest confusable families before the skeleton is consulted). Re-review on every Unicode revision, and keep it pinned in the interim. |
| `postcard` | 1.1.3 | The wire encoding. Serialises and deserialises member records and change sets in `types.rs`, `trie.rs` and `delta.rs`, over `serde`. | REQ-4umsuz, REQ-shk82j (SDD-55b2zj; LLR-xmpqn2) | **Safety-relevant.** LLR-xmpqn2 requires an accepted change set to be in canonical *structure*: removals strictly increasing, upserts strictly increasing and each observably changing the record, the two sets disjoint. Postcard is a compact, deterministic, non-self-describing format, but **the canonical form is enforced by `org-members`, not by postcard** — postcard will happily decode an unsorted or duplicate-bearing sequence; the ordering and disjointness checks in `delta.rs`/`trie.rs` are what refuse it. **Corrected 2026-09-17** (independent review): an earlier version of this row said `types.rs` "deliberately *rejects* rather than normalises non-canonical wire forms so that the encoding stays injective". That is true only of `P2pDeviceSlots`. `MemberLeaf`'s `Deserialize` impl **normalises** — `to_nfc` over `name` and `surname`, and the NFC form of the handle — so an NFD-encoded leaf and its NFC equivalent are distinct postcard byte strings decoding to one leaf, and **the encoding is not injective**. The dependency is trusted for determinism of *encoding a given value*, not for one-encoding-per-value across the deserialisation path, and not for validity of *decoded content*. Anything upstream needing byte-level identity (dedup by bytes, a replay guard keyed on the encoding) must supply it itself. Being non-self-describing, it is also schema-fragile: a field added or reordered in a serialised type silently reinterprets old bytes, so any change to a `Serialize` type in this unit is a wire-compatibility change. Built `default-features = false, features = ["alloc"]`. |
| `serde` | 1.0.229 | Derive-based serialisation framework that `postcard` encodes through; `types.rs` and `delta.rs` carry the impls, several of them hand-written so that validation runs on the deserialisation path. | REQ-shk82j (SDD-4yr9ge, SDD-55b2zj) | Mature, ubiquitous, and the de-facto standard. The risk it carries is not in the crate but in its use: a `#[derive(Deserialize)]` on a validated type bypasses the constructor and admits a value the constructor would refuse, which is precisely what REQ-shk82j exists to close. This unit's hand-written `Deserialize` impls (e.g. `P2pMemberKey`, `P2pDeviceKey` going through `VerifyingKey::from_bytes`) are the mitigation. `default-features = false, features = ["derive", "alloc"]`. |
| `thiserror` | 2.0.20 | Derives the unit's error enum in `org-members/src/error.rs`. | (SDD-m9gs5g) | Compile-time derive only; nothing of it survives into runtime behaviour beyond the `Display` strings. Contributes no failure mode of its own. The relevant property — that the unit reports errors rather than panicking — is the unit's own (SDD-m9gs5g), not the crate's. `default-features = false`. |
| `unicode-normalization` | 0.1.25 | NFC normalisation of member handles and names, in `normalize.rs` and `types.rs`. | REQ-h5ret5, REQ-m8aexh (SDD-4yr9ge) | Supports the handle rules: normalisation runs before length, script and skeleton checks, so a handle cannot dodge them by arriving decomposed. Same Unicode-version-drift consideration as `unicode-security`, but far milder — NFC is stable by Unicode's own normalisation-stability policy, so a table update cannot change the normal form of a string that already had one. Pre-1.0 version number, but a long-standing, widely-used crate. `default-features = false`. |
| `hashbrown` | 0.15.5 | `HashMap` that works in `no_std` + `alloc`; backs lookup structures in `trie.rs` and `delta.rs`. | (SDD-d9svdj, SDD-55b2zj) | Chosen for `no_std` reach rather than performance — it is the same implementation that backs `std::collections::HashMap`. Configured `features = ["default-hasher", "inline-more"]`, which selects `foldhash`; `foldhash` is **not** DoS-resistant and is not a cryptographic hash. That is acceptable here because no membership root or integrity decision is derived from a `HashMap` iteration or from `foldhash` — hashing that matters is BLAKE3 — but a future use that let untrusted input drive map key distribution would need re-examining. Deliberately not `ahash`, which requires `std`. |
| `spin` | 0.9.9 | `spin::Once`, used in `org-members/src/node.rs` as the write-once lazy cell holding a node's computed hash. | REQ-avmu3j (SDD-d9svdj; LLR-n7nya3) | **Load-bearing for REQ-avmu3j.** LLR-n7nya3 requires that a node's hash be a write-once cell and that the root be refused with `HashesNotCalculated` until every cell is filled; `Once` is what makes "write-once" true, including under concurrent access. A defect in its synchronisation would show up as a stale or torn hash, i.e. a root value reported for a record whose hashes are not current — the exact failure REQ-avmu3j forbids. Spinlock-based, so a contended cell busy-waits rather than parking; acceptable at the granularity used (one short hash computation per node) and unavoidable in `no_std`. Note the version distance: 0.9.9 is pinned while 0.12.3 is available, so this is the direct dependency furthest from upstream head and the first candidate for a reviewed bump. `default-features = false, features = ["once"]`. |

## Transitive closure

The 37-crate normal-edge closure is not enumerated row by row: no crate in it
is reached other than through one of the nine above, and none is selected by
this unit. The ones worth naming because they sit on a safety-relevant path
are `curve25519-dalek` 4.1.3, `ed25519` 2.2.3, `signature` 2.2.0, `sha2`
0.10.9, `subtle` 2.6.1 and `zeroize` 1.9.0 (all beneath `ed25519-dalek`),
`constant_time_eq` 0.4.2 and `arrayvec` 0.7.8 (beneath `blake3`),
`unicode-script` 0.5.8 (beneath `unicode-security`), `cobs` 0.3.0 (beneath
`postcard`), and `foldhash` 0.1.5 (beneath `hashbrown`). Their exact versions
are in `/Cargo.lock`.

No advisory-database scan (`cargo-audit`, `cargo-deny`) was run for this
inventory — neither tool is installed in this toolchain. That is a gap in the
evidence, not a clean result, and it should be closed before release.

## Test-only dependencies

`proptest` 1.11.0, `quint-connect` 0.1.2, `anyhow` 1.0.104, `hex` 0.4.3 and
`serde_json` 1.0.151 are dev-dependencies. They build the verification
evidence and are not part of the delivered software, so they are not SOUP
under IEC 62304 §8.1.2; they are named here so the distinction is recorded
rather than inferred. `postcard` and `serde` also appear as dev-dependencies,
with `std` features enabled, which is how the tests exercise the same wire
format the library encodes without `std`.
