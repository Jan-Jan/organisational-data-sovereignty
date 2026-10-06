# SOUP Inventory

<!--
Software Of Unknown Provenance (IEC 62304 §8.1.2): every third-party
component the software depends on. Keep versions exact; review this file
whenever a dependency changes. Note functional/performance requirements the
SOUP must meet and known anomalies relevant to safety.
-->

**As built at the merge (2026-10-05, after the torsion-free ruling; the `blake3` row and the dev-dependencies below as built by the Person definition change, 2026-10-06)**, diffed against `person/Cargo.toml` and the workspace
`Cargo.lock`. org-members' inventory (`org-members/docs/architecture/soup.md`)
contains the analysis of the uses the two units share; rows here state what is
particular to `person`.

| Name | Version | Role in system | Requirements it supports | Risk considerations / known anomalies |
|---|---|---|---|---|
| `ed25519-dalek` | 2.2.0 | `DevicePublicKey` validation (`VerifyingKey::from_bytes`, `is_weak`, `to_edwards`). | REQ-q6xkna (SDD-k5ee9x; LLR-7guspr) | **Safety-relevant.** Validation and byte round-tripping only, no signing or verification. `VerifyingKey::from_bytes` does **not** reject small-order points (all eight accepted, `is_weak()` true) nor mixed-order points (the base point plus a non-identity small-order point; `is_weak()` false) nor non-canonical encodings (y ≥ p, or x = 0 with the sign bit set), so `DevicePublicKey::parse` rejects all three itself, through `is_weak()`, through `to_edwards().is_torsion_free()` and by re-compressing the decoded point (`to_edwards().compress()`) — tests `a_small_order_point_dalek_accepts_is_rejected`, `parse_rejects_every_small_order_point_dalek_accepts`, `parse_rejects_a_point_with_a_torsion_component` and `parse_rejects_a_non_canonical_encoding_dalek_accepts`, each of which first asserts that dalek accepts the input. Every non-canonical encoding `from_bytes` accepts decodes to a point of small order or with a torsion component, so the first two checks reject it and the re-compression check is not separately observable: the last test's input, y = p + 3, is rejected by the torsion-free check, as it asserts. The re-compression check is kept as defence in depth against a change in dalek's decoding. The 3.0.0-pre.6 pre-release in the lockfile is not reached from `person`. `default-features = false, features = ["alloc"]`. |
| `curve25519-dalek` | 4.1.3 | Reached at runtime through `ed25519-dalek`, not named in `[dependencies]`: the point decompression behind `VerifyingKey::from_bytes`, the re-compression `to_edwards().compress()` that the canonical-encoding check compares against, `EdwardsPoint::is_torsion_free`, which the torsion-free check calls on the decoded point, and `is_small_order`, which `is_weak()` calls. | REQ-q6xkna (SDD-k5ee9x; LLR-7guspr) | **Safety-relevant.** `DevicePublicKey`'s three own checks rest on it: a wrong re-compression would reject a canonical encoding (a non-canonical one it let through would still be rejected by the torsion-free or small-order check, under today's decoding), a wrong `is_torsion_free` (it multiplies by the group order and compares with the identity) would let a point with a torsion component through or reject a prime-order key, and a wrong `is_small_order` would let a small-order point through. The device-key tests exercise all three through `ed25519-dalek` on fixed inputs. The 5.0.0-pre.6 pre-release in the lockfile is not reached from `person`. Also a direct dev-dependency (below). |
| `unicode-normalization` | 0.1.25 | NFC normalisation of `Name` and `Surname`. | REQ-r7mytp (SDD-k5ee9x; LLR-ayu93n) | NFC is stable under Unicode's normalisation-stability policy. `default-features = false`. |
| `serde` | 1.0.229 | Serialisation and deserialisation of the value types, every `Deserialize` impl routed through its validating constructor. Optional, behind the default `serde` feature. | REQ-r7mytp, REQ-4szc22, REQ-tq4ms4, REQ-q6xkna, REQ-3vqs9b (SDD-k5ee9x; LLR-ayu93n, LLR-sjrh7z, LLR-5za6mp) | A derived `Deserialize` on a validated type would bypass its constructor; the design items forbid it, and the decoding tests check each type. `default-features = false, features = ["derive", "alloc"]`. |
| `thiserror` | 2.0.20 | Derives `IdentityError`. | REQ-bczz87 (SDD-u9cddc; LLR-eeq89n) | Compile-time only. `default-features = false`. |
| `blake3` | 1.8.7 | The Person hash, `blake3::keyed_hash` of the `V1` encoding under `person::definition::v1__________` (SDD-v32mqh; LLR-4ebtn4), and the Person device sub-trie through `PersonDeviceHasher`, keyed under `person::device-leaf_____________` and `person::device-node_____________` (SDD-v32mqh; LLR-edn55h, over SDD-7r833z's LLR-6ezhw7). Each use has its own domain key, so no two share a hash domain, and none is org-members'. | REQ-9m5pq2, REQ-7n4g8b, REQ-wg7z4s (SDD-v32mqh; LLR-edn55h, LLR-4ebtn4, LLR-tf45kx) | **Safety-relevant.** The hash's collision and second-preimage resistance is what makes a match against a stored hash mean "this definition": a collaborator checks a definition against the hash published for a person (REQ-7n4g8b), and a different definition with the same hash would pass that check. Residual risk: that of the hash function itself, which this unit cannot reduce; it is the residual risk the risk file's assessment of REQ-9m5pq2 defers to this SOUP item. Outputs are pinned by `the_person_device_root_is_pinned` and `the_v1_hash_is_pinned`, and recomputed from `blake3::keyed_hash` directly by `the_root_is_the_depth_two_tree_under_the_person_domains` and `the_v1_hash_is_blake3_keyed_by_the_v1_domain_key`, so an upgrade that changed an output fails them. `default-features = false`. |

X25519 validity (REQ-3vqs9b, REQ-7gz72r) needs no library: it is an exact
byte comparison in `person/src/x25519.rs`. `curve25519-dalek` 4.1.3, besides
being reached at runtime through `ed25519-dalek` (row above), is also a direct
dev-dependency: the X25519 tests use it to generate keys and to confirm each
entry of the small-order list with its x-only Montgomery ladder, and to build
the mixed-order points whose u-coordinates the tests check. `postcard` 1.1.3
is a dev-dependency too: `person` ships no wire format of its own. `blake3`,
a dev-dependency until the Person definition, is now a normal dependency (row
above); the device-trie tests also name it to build org-members' and a test
domain's hashers. `serde_json` 1.0.151 is a dev-dependency because it keeps a
custom decode error's message, which the Person decoding tests check (postcard
maps every custom error to `SerdeDeCustom`). `bolero` 0.13.4 is a
dev-dependency that drives the `fuzz_person_decode` target, as it drives
org-node's and on-chain-client's fuzz targets. `proptest` 1.11.0 is a
dev-dependency that generates the arbitrary inputs of the no-panic property
tests (REQ-vxx8k3).
