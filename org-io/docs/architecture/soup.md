# SOUP Inventory

<!--
Software Of Unknown Provenance (IEC 62304 §8.1.2): every third-party
component the software depends on. Keep versions exact; review this file
whenever a dependency changes. Note functional/performance requirements the
SOUP must meet and known anomalies relevant to safety.
-->

The unit was created empty on 2026-10-06; its crate exists since task T1 of
S2's develop phase (`docs/plans/2026-10-06-org-io-create.md`). Each task
adds the rows for the components it first depends on, diffed against
`org-io/Cargo.toml` and the workspace `Cargo.lock`.

Rows still expected, taken from what moves into the unit
(`docs/plans/2026-10-06-org-io-create.md` §3): `rand_core`. Task T5
(2026-10-07) added `subxt`, `jsonrpsee`, `tokio`, `async-trait`, `hex` and the
chopsticks harness, with the chain connection, read and preflight moved from
org-node; their roles are copied from org-node's inventory
(`org-node/docs/architecture/soup.md`) and restated for org-io.
on-chain-client and org-node are units, not SOUP; their own inventories
cover what they bring in.

*2026-10-08 (task T12):* diffed against `org-io/Cargo.toml` and the
workspace `Cargo.lock`. The expected `rand_core` row is added, with `iroh`
(added to the manifest at T7) and the dev-only `rand` and `tokio`
`test-util`; no row is still expected.

| Name | Version | Role in system | Requirements it supports | Risk considerations / known anomalies |
|---|---|---|---|---|
| zeroize | 1.9.0 | Wipes the parsed development seed bytes when they are dropped (`SeedBytes`), and the environment value they were parsed from (`Zeroizing<String>`; 2026-10-08, review round 1) | LLR-c4bktx | Best-effort: wipes the values it owns, not copies the compiler or a caller made; org-io makes none on the seed path except the one below under subxt-signer. No known anomalies relevant to safety. |
| bolero (dev) | 0.13.4 | Fuzz harness for the seed and co-signer parse (`tests/fuzz_seed_parse`) | LLR-4ax2m6, LLR-9fy622, LLR-gc6kwy | Test-only; not in any production build. |
| subxt-signer | 0.50.3 | The user's own sr25519 signatory key (`SignatoryKey`, built from the parsed seed; `sr25519` and `subxt` features only) | LLR-c4bktx | Its `Keypair` derives `Clone`, so custody rests on org-io never cloning it out (LLR-u2pk5y's scan); `SignatoryKey` holds it privately with a hand-written `Debug` that shows only the start of the public key. `Keypair::from_secret_key` takes the 32 seed bytes by value, so one copy of the seed (the call's argument) is made that org-io cannot wipe (recorded 2026-10-08, review round 1, finding 3; owner ruling 2026-10-08: accepted as a development-only third-party limit, since the seed exists only in `dev-seed` builds and production signing, through the keychain or a hardware signer at a later stage, must avoid this API; noted on LLR-c4bktx); schnorrkel's `MiniSecretKey` and `SecretKey` built from it wipe themselves. No other known anomaly relevant to safety. |
| subxt | 0.50.3 | The chain connection (`connect.rs`: reconnecting RPC client, `LegacyBackend`, `OnlineClient`) and the preflight's chain checks (`preflight.rs`, `bin/preflight.rs`); reads go through on-chain-client's `OrgRegistryClient` | — (supports SDD-z85ux9, which carries no LLRs) | Features mirror `on-chain-client`'s pins (`native`, `jsonrpsee`, `reconnecting-rpc-client`) so the `subxt` types unify across the crates; they must be changed together or the units stop composing. Everything built on it sits in the I/O shell that this unit's gate does not reach. |
| jsonrpsee (through subxt) | 0.24.11 | The WebSocket RPC transport under subxt's reconnecting client | — (supports SDD-z85ux9) | Not named in org-io's `[dependencies]`; reached through `subxt`'s `jsonrpsee` feature. A dead connection surfaces as a subxt error, which the reconnecting client is there to absorb (the comment on `connect`). |
| tokio | 1.53.1 | The async runtime: `bin/preflight.rs`'s `#[tokio::main]`, the preflight's sample gap (`time`), and the async chain read | — | `rt-multi-thread`, `macros`, `time`. Nothing in this unit's gated set depends on scheduling order. |
| async-trait | 0.1.92 | Object-safe async methods on org-io's read seams `StateReader` and `RawStateSource` (`chain_read.rs`), which let the read rule be tested without a chain | LLR-rm9x4z | Boxes every call's future. A proc-macro shim for a language feature that is landing natively; removing it later is a mechanical change to the traits, not to callers. |
| iroh | 0.98.2 | The peer address type `iroh::EndpointAddr` that `OrgIo::submit_commit_send` passes through to org-node's send; org-io opens no endpoint of its own in S2 | — (supports SDD-erzj3m's submission sequence) | org-node's version, so the type unifies; org-io only forwards the value. Transport moves to org-io in S4, when this row's role grows. |
| rand_core | 0.6.4 | The `RngCore + CryptoRng` bounds on the handle's operations, whose random-number source org-io passes through to org-node | — | org-node's version, so the traits unify. org-io draws no randomness itself. |
| hex | 0.4.3 | `bin/preflight.rs` parses its two H160 environment variables (`ODS_CONTRACT_H160`, `ODS_ADMIN_H160`) | — | Operator input only; a bad value is reported by name and the binary exits non-zero. The seed and co-signer parse (`custody.rs`) does not use it, so their errors cannot carry `hex`'s offending-character text (LLR-gc6kwy). |

## Dev-only dependencies

| Name | Version | Role | Note |
|---|---|---|---|
| `tokio` (`test-util`), `rand` | 1.53.1, 0.8.8 | The paused clock of the 90-second bound's test (LLR-be3zv9); `OsRng` for the handle's operations in tests, as org-node's and the app's tests use | Test-only; added 2026-10-08 (T12), when this inventory was diffed against `org-io/Cargo.toml`. |
| `jsonrpsee` (`ws-client`, `http-client`), `serde_json`, `libc` | 0.26.0, 1.0.151, 0.2.189 | The chopsticks harness for `preflight` (`tests/common`), moved from org-node with the preflight 2026-10-07 | These support the chopsticks target **excluded** from `verify_commands`, which is the reason SDD-z85ux9 carries no low-level requirements. The harness's `jsonrpsee` is 0.26, a different major from the 0.24 subxt uses; the two are never mixed in one client. |

