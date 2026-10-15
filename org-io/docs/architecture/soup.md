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

*2026-10-13 (S4 task T7):* diffed against `org-io/Cargo.toml` and the
workspace `Cargo.lock` after adding `keyring`; the `keyring` row is added and
the `zeroize` row's role widened to the keychain buffers. No `zeroize_derive`
feature: nothing derives `Zeroize`.

| Name | Version | Role in system | Requirements it supports | Risk considerations / known anomalies |
|---|---|---|---|---|
| zeroize | 1.9.0 | Wipes the parsed development seed bytes when they are dropped (`SeedBytes`), and the environment value they were parsed from (`Zeroizing<String>`; 2026-10-08, review round 1). *Widened 2026-10-13 (S4 T7):* also the Device seed bytes read from or written to the OS keychain (the `Zeroizing` buffers of the keychain module and of the transitional holder `device_secrets.rs`) | LLR-c4bktx, LLR-677fqq, LLR-d5y7ks | Best-effort: wipes the values it owns, not copies the compiler or a caller made; org-io makes none on the seed path except the one below under subxt-signer. `keyring::Entry::get_secret` returns a plain `Vec<u8>`, which org-io must move into `Zeroizing` at once; copies inside `keyring` and the OS keychain service are out of its reach (see the `keyring` row). No known anomalies relevant to safety. |
| keyring | 3.6.3 (pinned `=3.6.3`) | The OS keychain: one generic-password item per Device key (service `ods-device-seed`, user the key's hex) holding that Device's 32-byte seed. `default-features = false`; features `apple-native` (macOS through Keychain Services, `security-framework` 3.7.0; iOS through the iOS keychain, `security-framework` 2.11.1), `windows-native` (Windows Credential Manager, `windows-sys` 0.60.2), `sync-secret-service` (Linux Secret Service over D-Bus, `dbus-secret-service` 4.1.0 → `dbus` 0.9.12 → `libdbus-sys` 0.2.7) and `crypto-rust` (the Secret Service session is encrypted, `dh-ietf1024-sha256-aes128-cbc-pkcs7`, with RustCrypto `aes`/`cbc`/`hkdf`/`sha2`). Every backend dependency is target-gated in keyring's manifest, so each platform compiles only its own. Licence MIT OR Apache-2.0; MSRV 1.75 | LLR-677fqq, LLR-n9dnm4, LLR-8erfbu, LLR-36jer8 (supports REQ-hw82ue, REQ-6d9xv6, RC-d8dbxs) | **Platform coverage (owner ruling 5, 2026-10-07), from 3.6.3's `lib.rs` and manifest:** macOS `apple-native`; iOS `apple-native`; Windows `windows-native`; Linux `sync-secret-service`; **Android none** (later stage, roadmap). **A plaintext or mock store defeats RC-d8dbxs:** on every target with no enabled native backend (Android, and any platform whose feature is dropped) keyring 3.6.3 makes its in-memory `mock` the *default* credential store silently, with no persistence; an item "written" there is gone at exit, which would make the Persona's store unrecoverable (HAZ-eyene4) instead of refused. So the keychain module must refuse on Android (REQ-6d9xv6) rather than use the default store, and LLR-36jer8's scan pins that only `test-support` names the mock or `MemoryKeychain`. The mock does **not** share an item between two `Entry` handles for one service and user (probed 2026-10-13: `NoEntry` on the second handle), so tests use org-io's `MemoryKeychain`. **Linux:** the Secret Service hands an unlocked collection's items to any process of the same user (the accepted HAZ-5n56w2 residual); headless Linux and WSL have no default collection and the node refuses to start (owner ruling, 2026-10-06); `libdbus-1` must be present at build (pkg-config) and at run time (no `vendored`). **Windows:** Credential Manager does not serialise concurrent access to one credential, so the keychain module (T8) must not touch one item from two threads at once; blob limit 2560 bytes (the seed is 32). **Apple:** empty service or user names are wildcards on lookup (org-io's are never empty). All calls block (IPC on Linux: tens to hundreds of ms), so async callers must not make them on a runtime worker thread (T8, T10). The crate's changelog (through 3.6.2; 3.6.3 has no entry) lists no security fix and no anomaly relevant to safety; the issue tracker was not reviewed in T7 and is to be checked before merge. *Reviewed 2026-10-15 (S4 review round 2, finding-12):* checked the tracker's open issues (https://github.com/hwchen/keyring-rs/issues), its releases page, and the one issue a release note ties to concurrency. Two issues were open: #322 (move every keystore to features; documentation, no defect) and #163 (Windows test instability, open since 2024-01-02, labelled bug/waiting): concurrent `set`/`get`/`delete` from several threads loses updates or leaves a deleted credential readable, traced by the maintainers to the Windows Credential Manager itself (direct `CredWriteW`/`CredReadW`/`CredDeleteW` calls show it), not to keyring, with no failures single-threaded. That is the Windows note above; org-io's keychain calls are not yet serialised (the round-2 umbrella report, PR-k5tdxj). The releases after 3.6.3 are the 4.x line (v4.0.0 to v4.2.0); none notes a fix of a 3.6.3 get/set/delete defect on macOS, Windows or the Secret Service. v4.1.6 fixed #343, a concurrent-initialisation race in 4.x's v1 compatibility layer that the issue states keyring 2 and 3 do not have (the store is chosen at compile time). The repository's `CHANGELOG.md` on its default branch was not reachable (HTTP 404; the default branch now holds the 4.x sample app), so no changelog entry later than the one read in T7 was checked. No defect affecting the features org-io uses was found beyond #163. 3.x is the maintenance line: the current major is 4.x (`keyring-core` 1.0 with per-platform store crates), adopted only by a new SOUP review. |
| bolero (dev) | 0.13.4 | Fuzz harness for the seed and co-signer parse (`tests/fuzz_seed_parse`) | LLR-4ax2m6, LLR-9fy622, LLR-gc6kwy | Test-only; not in any production build. |
| subxt-signer | 0.50.3 | The user's own sr25519 signatory key (`SignatoryKey`, built from the parsed seed; `sr25519` and `subxt` features only) | LLR-c4bktx | Its `Keypair` derives `Clone`, so custody rests on org-io never cloning it out (LLR-u2pk5y's scan); `SignatoryKey` holds it privately with a hand-written `Debug` that shows only the start of the public key. `Keypair::from_secret_key` takes the 32 seed bytes by value, so one copy of the seed (the call's argument) is made that org-io cannot wipe (recorded 2026-10-08, review round 1, finding 3; owner ruling 2026-10-08: accepted as a development-only third-party limit, since the seed exists only in `dev-seed` builds and production signing, through the keychain or a hardware signer at a later stage, must avoid this API; noted on LLR-c4bktx); schnorrkel's `MiniSecretKey` and `SecretKey` built from it wipe themselves. No other known anomaly relevant to safety. |
| subxt | 0.50.3 | The chain connection (`connect.rs`: reconnecting RPC client, `LegacyBackend`, `OnlineClient`) and the preflight's chain checks (`preflight.rs`, `bin/preflight.rs`); reads go through on-chain-client's `OrgRegistryClient` | — (supports SDD-z85ux9, which carries no LLRs) | Features mirror `on-chain-client`'s pins (`native`, `jsonrpsee`, `reconnecting-rpc-client`) so the `subxt` types unify across the crates; they must be changed together or the units stop composing. Everything built on it sits in the I/O shell that this unit's gate does not reach. |
| jsonrpsee (through subxt) | 0.24.11 | The WebSocket RPC transport under subxt's reconnecting client | — (supports SDD-z85ux9) | Not named in org-io's `[dependencies]`; reached through `subxt`'s `jsonrpsee` feature. A dead connection surfaces as a subxt error, which the reconnecting client is there to absorb (the comment on `connect`). |
| tokio | 1.53.1 | The async runtime: `bin/preflight.rs`'s `#[tokio::main]`, the preflight's sample gap (`time`), and the async chain read | — | `rt-multi-thread`, `macros`, `time`. Nothing in this unit's gated set depends on scheduling order. *Widened 2026-10-13 (S4, task T10):* `sync` for the endpoints' map (`tokio::sync::Mutex`) and the bounded receive channel (`mpsc`, 64 bodies) the accept tasks (`tokio::spawn`) feed; an accept task is aborted when its handle drops. |
| async-trait | 0.1.92 | Object-safe async methods on org-io's read seams `StateReader` and `RawStateSource` (`chain_read.rs`), which let the read rule be tested without a chain | LLR-rm9x4z | Boxes every call's future. A proc-macro shim for a language feature that is landing natively; removing it later is a mechanical change to the traits, not to callers. |
| iroh | 0.98.2 (pinned 0.98) | *Replaced 2026-10-13 (S4, task T10) by org-node's row, moved whole with the transport; S2's row read: the peer address type `iroh::EndpointAddr` passed through to org-node's send, no endpoint of org-io's own.* The QUIC transport: one endpoint per Persona (`endpoints.rs`), bound from the Persona's keychain item (`transport/endpoint.rs`, `bind_from_keychain`, the one function naming iroh's `SecretKey`), the framing (`transport/frame.rs`) and the transport preflight (`preflight.rs`). *Narrowed 2026-10-15 (S4, ARCHITECTURE conformance review finding-1):* iroh is called only from `transport/` — the endpoint, the framing and the peer-address parse (`transport/peer_addr.rs`); `endpoints.rs`, `send.rs` and `preflight.rs` call `OrgEndpoint`'s methods and name no iroh type (LLR-3vb4k9). The endpoint's `EndpointId` *is* the Device's public key, so a completed handshake is custody evidence; the endpoint holds the Device's ed25519 secret, which iroh needs to sign its TLS handshake, and is never handed to the app | REQ-db6s7q, REQ-2wzfzv, REQ-bupq5y, REQ-xuzw32, REQ-szt6ea (SDD-8uyg4s, SDD-b8tuv3, SDD-xwft92) | Pinned to 0.98 deliberately (Phase 2 note). Pulls `ed25519-dalek` 3.0.0-pre.6, a pre-release cryptographic dependency, via `iroh` and `iroh-base` 0.98.0; org-io converts the handshake's remote id to a `DevicePublicKey` through `person`'s parse (via org-node), not through that crate. **The builder binds pre-configured wildcard sockets when no bind address is named** — org-node's loopback-timeout report, fixed in `f635acc` before the move — and it is a property of the library's defaults rather than a bug in it; the Loopback arm names `127.0.0.1:0` and `[::1]:0` after `clear_ip_transports()`. `RelayMode::Disabled` on the Loopback arm is load-bearing and unverified: a home relay is acquired only after `online()`, so an assertion at bind time is vacuous. `0.98` exposes no relay-map accessor. `test-support` pulls `iroh/test-utils`, which brings `CaRootsConfig::insecure_skip_verify()` into any build enabling that feature (the test-only `BindPlan::Relay`). `SecretKey::from_bytes` takes the seed by reference and keeps its own copy inside the key, which org-io moves into the builder and cannot wipe; the keychain buffer it was lent from is wiped (LLR-d5y7ks, LLR-677fqq). |
| rand_core | 0.6.4 | The `RngCore + CryptoRng` bounds on the handle's operations, whose random-number source org-io passes through to org-node | — | org-node's version, so the traits unify. org-io draws no randomness itself. |
| hex | 0.4.3 | `bin/preflight.rs` parses its two H160 environment variables (`ODS_CONTRACT_H160`, `ODS_ADMIN_H160`) | — | Operator input only; a bad value is reported by name and the binary exits non-zero. The seed and co-signer parse (`custody.rs`) does not use it, so their errors cannot carry `hex`'s offending-character text (LLR-gc6kwy). *Widened 2026-10-13 (S4, task T12):* also the peer-address parse (`peer_addr.rs`, since 2026-10-15 `transport/peer_addr.rs`, LLR-2b2ghe), whose `Hex` refusal carries `hex`'s reason, which may quote a character of the operator's own address text (not a secret). |
| postcard | 1.1.3 | *Added 2026-10-13 (S4, task T12, with the parse moved from the app):* decodes a peer address, the hexadecimal of a postcard-encoded `iroh::EndpointAddr`, with `take_from_bytes`, refusing bytes left over (`peer_addr.rs`, since 2026-10-15 `transport/peer_addr.rs`), and encodes one back for `PeerAddr::to_hex`. Feature `use-std`; the version org-node and the app already build | LLR-2b2ghe (supports REQ-n82q47; the app's two peer-address decode items, moved back to the app 2026-10-15, rest on it through `parse_peer_addr`) | Operator input only, and only addresses: a malformed one is a typed `Decode` refusal, never a panic (fuzzed, `tests/fuzz_peer_addr`). postcard's `from_bytes` ignores trailing bytes, which is why the parse uses `take_from_bytes` and checks the rest. The decoded `EndpointAddr`'s own serde form is iroh's (the `iroh` row); an address that decodes is dialled only when its identity is a recipient's Device key (LLR-r9sh6k). No known anomaly relevant to safety. |

## Dev-only dependencies

| Name | Version | Role | Note |
|---|---|---|---|
| `tokio` (`test-util`), `rand` | 1.53.1, 0.8.8 | The paused clock of the 90-second bound's test (LLR-be3zv9); `OsRng` for the handle's operations in tests, as org-node's and the app's tests use | Test-only; added 2026-10-08 (T12), when this inventory was diffed against `org-io/Cargo.toml`. |
| `iroh` (`test-utils`) | 0.98.2 | `run_relay_server()` for the hermetic `transport_networked` target (moved from org-node 2026-10-13, S4 task T10) | Also reachable from any build enabling this unit's `test-support` feature, which is why that feature is stated never to ship. |
| `jsonrpsee` (`ws-client`, `http-client`), `serde_json`, `libc` | 0.26.0, 1.0.151, 0.2.189 | The chopsticks harness for `preflight` (`tests/common`), moved from org-node with the preflight 2026-10-07 | These support the chopsticks target **excluded** from `verify_commands`, which is the reason SDD-z85ux9 carries no low-level requirements. The harness's `jsonrpsee` is 0.26, a different major from the 0.24 subxt uses; the two are never mixed in one client. |

## What a reader should check when a dependency changes

1. An `iroh` minor bump re-opens the question of org-node's loopback-timeout
   report (moved here with the transport 2026-10-13, S4 task T10), because the
   defect there was the library's default bind behaviour and not this unit's
   code. The two loopback tests in `transport_handshake`
   (`loopback_mode_binds_and_advertises_loopback_only`,
   `loopback_mode_holds_under_repeated_and_colliding_binds`) are what answer
   it.

## Dependency-edge assessment: org-io → keyring ("Declaring a dependency", S4 task T7, 2026-10-13)

- **Uses:** `keyring::Entry` (`new`, `set_secret`, `get_secret`,
  `delete_credential` if S3b-io deletes items), `keyring::Error` (mapped to
  `KeychainError`: `NoEntry` → `ItemMissing`, `NoStorageAccess` → `Locked`,
  anything else → `Unavailable` carrying the variant name only), and under
  `test-support` only `keyring::credential::{CredentialApi,
  CredentialBuilderApi}` and `keyring::set_default_credential_builder` for
  `MemoryKeychain`. Named only in `org-io/src/keychain.rs` and
  `org-io/src/test_support.rs` (LLR-677fqq, LLR-36jer8's scans, T8).
- **Unit boundary:** org-io is the only unit that names a keychain crate
  (REQ-hw82ue, owner ruling 6, 2026-10-07); org-node, person, org-members and
  on-chain-client name none (T8's scan of their manifests).
- **Functional needs the SOUP must meet:** store and return 32 arbitrary
  bytes under a (service, user) pair, byte-exact (`set_secret`/`get_secret`,
  not the UTF-8 password API); report a missing item distinctly from a
  locked or failing store. 3.6.3 meets both on the three backends enabled.
- **Gaps:** Android (no backend; the default store there is the mock, which
  org-io must not use); Windows not built or run in this environment (no
  `x86_64-pc-windows-msvc` target installed, `~/.rustup` read-only), so its
  backend is assessed from source only; iOS likewise (no iOS target).
- **Risk:** RC-d8dbxs rests on the keychain holding the seed; HAZ-5n56w2 and
  HAZ-eyene4 residuals as accepted (owner rulings, 2026-10-06). No new hazard.
- **Class:** C, as org-io. No segregation.
