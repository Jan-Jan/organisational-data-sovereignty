# SOUP Inventory

<!--
Software Of Unknown Provenance (IEC 62304 §8.1.2): every third-party
component the software depends on. Keep versions exact; review this file
whenever a dependency changes. Note functional/performance requirements the
SOUP must meet and known anomalies relevant to safety.
-->

## Evidence and provenance

Every version below is read from `on-chain-client/Cargo.lock`, measured on
2026-09-28.

**That lockfile is the one cargo actually reads for this unit**, which is worth
stating because the sibling unit's is not. `on-chain-client/Cargo.toml` opens
with an empty `[workspace]` table, deliberately — the comment there records
why: the repository root carries a workspace that does not list this crate, and
without the empty table cargo walks up to it and refuses to build. So
`cargo locate-project --workspace`, run from `on-chain-client/`, names
`on-chain-client/Cargo.toml` itself. This unit is its own workspace root, its
lockfile governs what it builds against, and the versions in this table are
reproducible rather than incidental. (Contrast `org-members`, which declares no
workspace of its own: its unit-level lockfile was inert, had drifted, and was
deleted in `ec66743`. Do not generalise that remedy to this unit — here the
unit lockfile is the live one.)

The closure was measured with `cargo tree --edges normal`: **five direct
dependencies over a transitive closure of 203 crates** (unique `name@version`,
`on-chain-client` itself excluded, default features `dev-rpc` → `client` →
`std`). The lockfile carries 407 package entries in all. The 204 difference is
**not** the dev-dependency closure: measured on 2026-09-29, adding dev and build
edges takes the tree to 262 crates, so dev and build add 59. The remaining ~145
are entries the lockfile records for features and targets this build does not
compile; **their composition was not enumerated**, and the figure is a
subtraction rather than a measurement. Corrected by review rounds 4 and 5; the
three measured figures were each right and the sentence joining them was not.

No advisory-database scan (`cargo-audit`, `cargo-deny`) was run for this
inventory — neither tool is installed in this toolchain. That is a gap in the
evidence, not a clean result, and it should be closed before release. The same
gap is recorded in `org-members`' inventory; it is a repository-wide omission,
not a property of this unit.

## Direct dependencies

| Name | Version | Role in system | Requirements it supports | Risk considerations / known anomalies |
|---|---|---|---|---|
| `tiny-keccak` | 2.0.2 | The only hash function in the unit, and it is used for two different decisions. `src/h160.rs` keccaks a 32-byte account identifier to derive the Organisation admin on the forward path; `src/client.rs`'s `internals::solidity_mapping_slot` keccaks the 64-byte ABI preimage to derive the Slot key an Organisation's state is read from. | REQ-rz7fja, REQ-tkhe3u, REQ-9m2rnd, REQ-xudf25 (SDD-5b8wxs, SDD-bw7v5x; LLR-3bkhuc, LLR-7pjzjn, LLR-vktf8w, LLR-62tqnv) | **Safety-relevant, and the most load-bearing crate in this table.** Both uses are *identity* derivations, not integrity checks: whatever twenty bytes this crate's keccak produces IS the organisation's on-chain identifier, and whatever thirty-two bytes it produces IS the storage key read. A defect — a wrong padding rule, a truncated absorb, a mismatch with Ethereum's keccak-256 rather than SHA3-256 — does not produce a detectably corrupt answer; it produces a confidently wrong one, silently reading another organisation's slot or deriving an admin no chain agrees with. That is HAZ-v2cmtx's harm directly. The mitigation is not trust in the crate: both derivations are pinned by gated tests that **recompute the expected value independently** (`tests/h160_mapping.rs` re-derives the reading from the documented pallet-revive rule; `tests/storage_slot_layout.rs` recomputes the mapping key from the ABI preimage and checks it against a known vector), so a change in this crate's output reddens the suite rather than passing through. Built `default-features = false, features = ["keccak"]`, so the `sha3` half of the crate is not compiled in. Pure Rust, `no_std`, no unsafe-heavy assembly path; a small, stable, widely-used crate whose last release is old — which for a frozen hash standard is maturity, not neglect. |
| `parity-scale-codec` | 3.7.5 | The SCALE decoder for pallet-revive event payloads. `src/decode/v_paseo_ah.rs` decodes `ContractEmitted`'s three fields — `contract: [u8; 20]`, `data: Vec<u8>`, `topics: Vec<[u8; 32]>` — from the raw payload bytes. | REQ-sx5b6g, REQ-axcxf7, REQ-n6v896, REQ-5upq6n (SDD-d5jh6t; LLR-u2e389, LLR-8242kq, LLR-2v5u4d, LLR-6tjhgk) | **Safety-relevant, on the untrusted-input boundary.** This is the first thing that touches bytes arriving from the chain, and those bytes are attacker-influenced: any account can emit a log. The property relied on is that decoding is total — every input either decodes or returns `Err`, never panics and never reads out of bounds. SCALE's `Vec<T>` decoding reads a compact length prefix and then that many elements, which is the classic allocation-amplification shape; the crate bounds this by checking the remaining input against the element size before allocating, and the unit's own `fuzz_parse_revive_event` bolero target is what pins the whole path as panic-free rather than the claim being taken on trust (LLR-2v5u4d). Note what the crate does *not* give: it will happily decode a payload that leaves trailing bytes, so the "nothing beyond the declared fields" rule (REQ-axcxf7) is enforced by `on-chain-client`'s own explicit `!bytes.is_empty()` check after the three fields, not by the codec. Built `default-features = false, features = ["derive"]`, so the crate compiles in the `no_std + alloc` configuration the decoder is designed for. |
| `subxt` | 0.50.1 | Powers the entire chain-facing client: backend and transport, metadata-aware storage and event access, the `ReviveApi::get_storage` runtime-API call each slot is read through, the best-block and finalised block streams both subscription lanes ride, and (behind the `smoldot` feature) the embedded light client. | REQ-9vwcwc, REQ-5zux82, REQ-hd6m9d (SDD-3b8zef) | **Safety-relevant, and the least verified dependency in this table** — deliberately so, and the reason is recorded rather than hidden. The part of this unit that *uses* it — the transport shell — is SDD-3b8zef, the one software item here carrying **no low-level requirements**, because nothing at this unit's gate exercises it (16.37% line coverage on `client.rs`; the five integration targets that do reach it — `off_chain_genesis_ceremony`, `p_address_is_orgid`, `reorg_cancels_proposed`, `scenario_a_full`, `two_orgs_one_watcher` — need a chopsticks fork and run in no gate). **Corrected 2026-09-28 by this change's review sweep:** this cell previously said "every requirement it supports belongs to SDD-3b8zef", which its own "Requirements it supports" column contradicts. All three are traced by a second item as well, and each of those second items carries gated low-level requirements — REQ-9vwcwc by SDD-4z3k2u (three LLRs), REQ-5zux82 by SDD-m59zrg (three LLRs), REQ-hd6m9d by SDD-v2rtka (two LLRs, both refining that item's version-resolution interface, which is the only one of its three declared responsibilities the decomposition treats as fully gated). Not one of the three belongs to SDD-3b8zef alone. **The conclusion is unchanged and does not rest on that premise**: what those sibling LLRs verify is this unit's *own* chain-free decisions — admission, the best-lane rule, decoder resolution — none of which runs a byte of `subxt`. The only code that exercises `subxt` is the transport shell, and no gated test reaches it. So this crate's behaviour under this unit's own use is asserted, not measured, at every gated run. Three known anomalies. **(1) The wasm32-browser lane is blocked upstream**, investigated 2026-06-05 and recorded in `Cargo.toml`: cargo's cross-target feature unification re-enables `jsonrpsee` for the wasm entry even when that entry drops it, so `subxt-rpcs/web` demands `jsonrpsee-wasm-client ^0.24.11` — a sub-crate of subxt 0.50.1's pinned jsonrpsee that **was never published to crates.io**. 0.50.1 is the newest published subxt; there is no release with jsonrpsee 0.25+. This gates ODS Phase 1.c and is a re-assess trigger, not something to work around with a `[patch]` onto a git-sourced yanked sub-crate. The native `smoldot` lane is unaffected. **(2) The default `CombinedBackend` silently mis-serves chopsticks**, which supports the legacy RPC group fully but only part of the v2 `chainHead`/`transactionWatch` groups — tests therefore pass an explicit `LegacyBackend`, and getting this wrong produces a client that appears to work and does not. **(3) The finalised stream is monotonic by block number** and does not re-emit replacement blocks after a finalised-height rewind, with no "finality reverted" notification; reorg detection therefore rests entirely on this unit's own best-lane rule (SDD-m59zrg) rather than on anything subxt reports. Built `default-features = false, features = ["native", "jsonrpsee"]`. |
| `futures-util` | 0.3.32 | Stream combinators the subscription is built from: `select` merges the best and finalised lanes into one stream, and `then` / `flat_map` lift each block's decoded events into the yielded sequence. | REQ-5zux82 (SDD-3b8zef) | Mature and ubiquitous; contributes no decision of its own. One property is load-bearing and worth naming: `stream::select` is **not** ordering-preserving across the two lanes — it interleaves arbitrarily, which is why the same on-chain event generally arrives twice (once as a best-block observation, once as finalised) and why consumers are told to treat the first as optimistic. That is a designed consequence, documented on `subscribe`, not a defect. Built `default-features = false, features = ["std"]`. |
| `futures-core` | 0.3.32 | Supplies the `Stream` trait that `SubscribedEventStream` is a boxed trait object of. | REQ-5zux82 (SDD-3b8zef drives the two lanes that fill the stream; the `SubscribedEventStream` type itself is SDD-5wamsz, which owns the observation vocabulary) | Trait definitions only; no runtime behaviour, no failure mode of its own. Built `default-features = false, features = ["alloc"]`. |

## Transitive closure

The 203-crate normal-edge closure is not enumerated row by row: no crate in it
is reached other than through one of the five above, and none is selected by
this unit. Essentially all of it sits beneath `subxt` — the SCALE tooling
(`scale-decode` 0.16.2, `scale-encode` 0.10.1, `scale-value` 0.18.2,
`scale-info` 2.11.6, `scale-bits` 0.7.0, `scale-info-legacy` 0.4.2,
`frame-decode` 0.17.2, `frame-metadata` 23.0.1), the RPC stack
(`jsonrpsee` 0.24.11 and its transport crates), and the primitive types
(`primitive-types` 0.13.1, `fixed-hash` 0.8.0, `keccak-hash` 0.11.0). Their
exact versions are in `on-chain-client/Cargo.lock`.

Two observations from that closure that are not visible from `Cargo.toml`:

**`jsonrpsee` is present at two incompatible versions**, 0.24.11 and 0.26.0.
0.24.11 is the one `subxt` 0.50.1 pins internally and is the version the client
actually talks to a node through; 0.26.0 is what this crate's own
`[dev-dependencies]` declares for the integration tests' direct RPC calls. Both
are compiled into a test binary. This is not a conflict cargo will resolve —
they are semver-incompatible, so both are linked — and it is the same pin that
blocks the wasm32 lane, seen from the other side: the dev-dependency was moved
forward while subxt's could not be. Nothing depends on the two agreeing today,
because the test-side client speaks raw JSON-RPC and shares no types with
subxt's. It is recorded because a future change that tries to hand a jsonrpsee
client from one to the other will fail in a confusing way.

**`keccak-hash` 0.11.0 and `keccak` 0.1.6 are both in the closure** — the first
beneath subxt via `primitive-types`, the second via `sha3` 0.10.9 →
`sp-crypto-hashing` → `frame-decode` / `subxt-metadata`. The unit uses only
`tiny-keccak`, by explicit import in `h160.rs` and `client.rs`; nothing derives
an address or a slot key through the subxt-side implementations.

**Corrected 2026-09-29 by review round 4: there are two keccak implementations
in the closure, not three.** This paragraph argued from a count of three, and
the count was wrong in a way that inverts part of its own argument. Measured
with `cargo tree --edges normal -p keccak-hash --depth 1`: `keccak-hash` 0.11.0
depends on **`tiny-keccak` 2.0.2** — the same crate at the same version this
unit imports directly. So `keccak-hash` is a wrapper over this unit's own
implementation, not a second one. The two genuinely distinct implementations are
`tiny-keccak` 2.0.2 and `keccak` 0.1.6, the latter reached through `sha3` 0.10.9
→ `sp-crypto-hashing` → `frame-decode` / `subxt-metadata`, which the earlier
paragraph never named.

The point that survives, and is the one worth keeping: more than one keccak in a
binary is the shape in which someone later "simplifies" by switching to another,
and the gated tests (`h160_mapping`, `storage_slot_layout`) recompute their
expectations from first principles rather than comparing two implementations, so
a switch is caught only where the implementations actually disagree. The
correction sharpens which switches are dangerous. Switching to `keccak-hash`
would call the same underlying implementation, so a defect in `tiny-keccak`
would not be exposed by the switch — whether that wrapper applies the same
keccak-256 parameters is not measured here. Switching to the `sha3`/`keccak`
0.1.6 path is the real hazard, and it is the one a reader should be watching for.

## Test-only dependencies

`jsonrpsee` 0.26.0, `tokio` 1.52.3, `serde_json` 1.0.150, `hex` 0.4.3,
`subxt-signer` 0.50.1, `blake2` 0.10.6, `libc` 0.2.186 and `bolero` 0.13.4 are
dev-dependencies. They build the verification evidence and are not part of the
delivered software, so they are not SOUP under IEC 62304 §8.1.2; they are named
here so the distinction is recorded rather than inferred. `parity-scale-codec`
and `tiny-keccak` also appear as dev-dependencies, because an integration-test
crate cannot see the library's own dependencies and the round-trip target and
the corpus regenerator need both.

Two of them do more than they appear to. `bolero` 0.13.4 drives the three fuzz
targets that carry LLR-2v5u4d and LLR-yhw34z — the only evidence in this unit
that its decoders are total — so while it ships nothing, a defect in its input
generation would weaken a safety argument rather than a convenience. And
`libc` 0.2.186 is used by the chopsticks harness to put the subprocess in its
own process group (`setsid` in `pre_exec`, `killpg` in `Drop`), because
chopsticks forks a worker and killing only the parent orphans it; that is test
hygiene, but it is the reason a stray chopsticks process does not silently
serve a later run stale state.
