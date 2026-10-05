# SOUP Inventory

<!--
Software Of Unknown Provenance (IEC 62304 §8.1.2): every third-party
component the software depends on. Keep versions exact; review this file
whenever a dependency changes. Note functional/performance requirements the
SOUP must meet and known anomalies relevant to safety.
-->

## Evidence and provenance

Every version below is read from the **repository-root `Cargo.lock`**, measured
on 2026-10-03 on branch `worktree-guardrails-org-node-arch`, with `master`
merged in at `5850ed7` (merge commit `4f4ec19`) — the first of this branch's
three merges of `master`, tabulated in the plan.

*Corrected 2026-10-04 by review round 3. This sentence named `2405ede`, which
is the exact commit review round 2 established is **not** a merge parent of
this branch at all. Round 2's finding-13 said the claim was "fixed in both" —
the plan and the verification record — and this was the third document making
it, in the one file round 2's finding-12 had just convicted as the one nobody
re-measured after a merge. Two of three corrected is the instance, not the
class, for the third round running.*

**The root lockfile is the one cargo reads for this unit, and that is a
difference from the sibling unit rather than a default.** `org-node/Cargo.toml`
declares `[lints] workspace = true` and the root `Cargo.toml` lists `org-node`
in `members`, so `cargo locate-project --workspace` run from inside `org-node/`
names the repository root. There is no `org-node/Cargo.lock` and there should
not be one. Contrast `on-chain-client`, which opens its manifest with an empty
`[workspace]` table on purpose and is its own workspace root with its own live
lockfile — **do not generalise either arrangement to the other unit.**

The root lockfile carries **797 package entries**, which is the whole workspace
— `org-members`, `on-chain-client`, `org-node` and `app`'s Rust side together —
and is not a measurement of this unit.

This unit's own closure was measured with `cargo tree -p org-node --features
app,test-support`, counting unique `name version` pairs:

| Edge kinds | Unique packages |
|---|---|
| `normal` | **446** |
| `normal,build,dev` | **482** |

So dev and build edges add 36. Three of the 446 are not SOUP — `org-node`
itself and the two path siblings `org-members` and `on-chain-client`, which are
units of this repository with their own ledgers — leaving **443 third-party
crates** in the shipped closure.

*Measured twice.* A first pass reported 601 and was wrong: it counted whole
`cargo tree` lines, so `(proc-macro)` suffixes and feature annotations split one
package across several lines. The figures above count `name version` pairs
only. The error is recorded rather than quietly corrected because the same
mistake is easy to repeat.

**`app,test-support` is the feature set measured**, and it is the set this
unit's `verify_commands` builds. It is **not** what ships: `test-support` is
never enabled in a production build (`org-node/Cargo.toml`), and it pulls
`iroh/test-utils`. The shipped set is `app`, which is `chain` + `transport` +
the three persistence crates.

No advisory-database scan (`cargo-audit`, `cargo-deny`) was run — neither tool
is installed in this toolchain. That is a gap in this inventory, not a finding
of no advisories, and it is the same gap `on-chain-client`'s inventory records.
It is booked in `docs/plans/2026-09-05-ratchet-setup.md`.

## Direct dependencies

Eighteen, of which sixteen are SOUP. Each row's "requirements it supports" names
requirements from `org-node/docs/requirements/`; the modules are from a grep of
`org-node/src`, not from the manifest's intent.

*Two of those citations were corrected 2026-10-04 by review round 2, which
found them naming a requirement the component has nothing to do with. `base64`
is the armour around out-of-band Invites and Join requests in `blobs.rs`; it
was credited to REQ-eg5j8u, which is solely about the 1 MiB **wire frame**
bound on send and receive, and now names REQ-9g6as6, which the blob low-level
requirements LLR-8qxwst and LLR-tcft2r satisfy. `thiserror` — described in its
own row as the whole rejection vocabulary — was credited to REQ-eg5j8u alone
and now names the three requirements that vocabulary serves. This is the same
class of mis-citation round 1 found in the architecture ledger, surviving here
in the file round 1 had cited as supporting evidence for that finding.*

| Name | Version | Role in system | Requirements it supports | Risk considerations / known anomalies |
|---|---|---|---|---|
| `ed25519-dalek` | 2.2.0 | Every signature in the unit: the Member-as-a-group key that signs a delta transcript, the device key that is both the trie's `P2pDeviceKey` and the iroh endpoint identity. `keys.rs`, `envelope.rs`, `verify.rs`, `transport/endpoint.rs`, `service.rs` | REQ-ag6kqm, REQ-gju89b, REQ-bvh8v6, REQ-xa6smf, REQ-ztdza4 | Built with `zeroize`. Verification uses the library's default (non-batch) path, so the malleability and cofactor questions that affect batch verification do not arise here. The unit never calls `verify_strict`; signatures are produced and consumed by this same library, so the small-order-key divergence is not reachable through the wire format — **this is an argument, not a test**, and no gated test distinguishes the two entry points. |
| `iroh` | 0.98.2 | The QUIC transport. The endpoint's `EndpointId` *is* the device's verifying key, so a completed handshake is custody evidence. `transport/endpoint.rs`, `service.rs` | REQ-db6s7q, REQ-2wzfzv, REQ-ztdza4, REQ-nhe2zu | Pinned to 0.98 deliberately (Phase 2 note). **The builder binds pre-configured wildcard sockets when no bind address is named** — this was PR-d4nye8, fixed in `f635acc`, and it is a property of the library's defaults rather than a bug in it. `RelayMode::Disabled` on the Loopback arm is load-bearing and unverified: a home relay is acquired only after `online()`, so an assertion at bind time is vacuous. `0.98` exposes no relay-map accessor. `test-support` pulls `iroh/test-utils`, which brings `CaRootsConfig::insecure_skip_verify()` into any build enabling that feature. |
| `chacha20poly1305` | 0.10.1 | XChaCha20-Poly1305 AEAD over the whole persona store file. `store.rs` | REQ-hzm4kt | A fresh 24-byte nonce is drawn per save from the caller's RNG; nonce reuse would be catastrophic and is not defended by the library. The derived key is held in `PersonaStore` as plain bytes and is **not** zeroized on drop. |
| `argon2` | 0.5.3 | Passphrase-to-key derivation for the store. `store.rs` | REQ-hzm4kt | Uses `Argon2::default()` — the library's default parameters, not parameters this project chose or recorded. The salt is a **fixed 32-byte application constant**, not per-store random (PoC simplification S9, stated in `store.rs`). Both are deliberate PoC choices and both weaken the KDF against a precomputation attack across installations. |
| `postcard` | 1.1.3 | The wire and at-rest encoding for every structured value: envelopes, deltas, store data, blobs, ids, wire frames. `envelope.rs`, `store.rs`, `blobs.rs`, `ids.rs`, `transport/wire.rs` | REQ-9g6as6, REQ-bcxz96, REQ-eg5j8u | The decode path is the unit's largest untrusted-input surface. Three bolero fuzz targets at this unit's gate assert it neither panics nor aborts on arbitrary bytes. Postcard is not self-describing, so a length or variant mismatch is a decode error rather than silent reinterpretation — which is what REQ-9g6as6 and REQ-bcxz96 rest on. |
| `serde` | 1.0.229 | Derive layer under `postcard`. `blobs.rs`, `store.rs`, `envelope.rs`, `ids.rs`, `transport/wire.rs` | REQ-9g6as6, REQ-bcxz96 | Derived impls throughout **except one**: `envelope.rs:53` carries `#[serde(with = "sig_bytes")]`, a hand-written `Visitor` for the 64-byte signature with two entry points — `visit_bytes` and `visit_seq`. `visit_bytes` enforces the length exactly, by `try_into`. **`visit_seq` does not**: it fills sixty-four elements and errors only if the sequence runs short, so a longer sequence is not refused by this visitor and whether it is refused at all depends on the format's own framing. The call is `deserialize_bytes`, which postcard answers with `visit_bytes`, so `visit_seq` is not on postcard's path — it is a second entry point whose weaker check no gated test reaches. That is the one place in this unit where a decode invariant is hand-maintained rather than derived, and it is the signature field. It exists because postcard does not hint byte arrays through the derive. |
| `base64` | 0.22.1 | Copy-pasteable armour around `postcard` for out-of-band invites and join requests. `blobs.rs` | REQ-9g6as6 | Standard alphabet with padding. A decode failure is a typed error, not a panic. |
| `thiserror` | 2.0.20 | The typed rejection vocabulary. One variant per rejection path **on the receive-and-commit path**; elsewhere in the unit fifty sites share `OrgNodeError::Chain(String)`. `error.rs`, `transport/mod.rs`, `chain_write/mod.rs` | REQ-9g6as6, REQ-bcxz96, REQ-eg5j8u | Derive-only; generates `Display`/`Error` and no control flow. **Corrected 2026-10-04 by review round 3**, which measured the fifty `Chain(String)` sites: this row said "one variant per rejection path" flatly, as did the ledger, and that is true only of verify-against-chain. `Chain`'s `Display` is `"chain read failed: {0}"`, so a wrong store passphrase and a malformed invite blob both render as chain read failures. Recorded as a gap, not closed here. |
| `rand_core` | 0.6.4 | The RNG trait boundary. Key generation and store nonces take `R: RngCore + CryptoRng` from the caller rather than sampling the OS directly, which is what makes those paths deterministic under test. `keys.rs`, `store.rs`, `service.rs` | REQ-hzm4kt | 0.6, one major behind `rand` 0.9's `rand_core` 0.9 — pinned to match `ed25519-dalek` 2.x. **`CryptoRng` is a marker trait the compiler cannot enforce the meaning of**: a caller may pass a seeded test RNG into a production path and nothing in this unit objects. |
| `subxt` | 0.50.3 | Every chain read and write: RPC client, extrinsic construction, event decoding. `chain_write/*`, `ceremony.rs`, `preflight.rs`, `service.rs` | — (supports SDD-z85ux9, which carries no LLRs) | Features mirror `on-chain-client`'s pins so the `subxt` types unify across the two crates; they must be changed together or the units stop composing. Everything built on it sits in the I/O shell that this unit's gate does not reach. |
| `subxt-signer` | 0.50.3 | sr25519 signing for extrinsic submission and the dev accounts the ceremony uses. `chain_write/*`, `ceremony.rs`, `service.rs` | — (supports SDD-z85ux9) | Pulls a bip39/schnorrkel stack for a key type **this unit does not otherwise use**: org-node's own identity is ed25519 throughout, and sr25519 appears only on the chain-submission side. |
| `parity-scale-codec` | 3.7.5 | SCALE encoding of the multisig pseudo-account preimage. `chain_write/multisig.rs` | — | Used for exactly one derivation. A SCALE layout change would silently move the derived account rather than fail, and the gated test `the_derived_account_does_not_depend_on_signer_order` pins the ordering property but not the encoding itself. |
| `blake2` | 0.10.6 | `blake2_256` over that preimage — the pallet-multisig pseudo-account derivation. `chain_write/multisig.rs` | — | Via `Blake2bVar`, whose constructor is fallible; 32 is always valid and the code says so at the call site. Must track the runtime's hasher: a divergence yields a valid-looking address that holds no funds. |
| `tokio` | 1.53.1 | The async runtime behind the chain and transport paths. `transport/endpoint.rs`, `service.rs`, `preflight.rs`, `chain_write/multisig.rs`, `bin/preflight.rs` | — | `rt-multi-thread` + `macros`; `time` only as a dev-dependency, so a production build has no timer driver. Nothing in this unit's gated set depends on scheduling order. |
| `async-trait` | 0.1.92 | Object-safe async methods on the `ChainOps` seam, which is what makes the service headless-testable against `MockChainOps`. `service.rs`, `chain_write/proxy.rs` | REQ-wp2nyc, REQ-8gz8bu, REQ-nhe2zu (via the seam the mock substitutes at) | Boxes every call's future. A proc-macro shim for a language feature that is landing natively; removing it later is a mechanical change to the trait, not to callers. |
| `hex` | 0.4.3 | Operator-facing display of hashes and addresses. `bin/preflight.rs`, `chain_write/submit.rs` | — | Presentation only; no value is parsed back from hex in this unit. |

## Not SOUP

| Name | Why |
|---|---|
| `org-members` | A unit of this repository (`.guardrails/units.yaml`), class C, with its own ledgers and its own SOUP inventory. org-node's reliance on it is declared as `depends_on` and what it relies on beyond that unit's stated requirements is REQ-q92yac. |
| `on-chain-client` | Likewise a unit of this repository, class C, declared in `depends_on`; the unstated reliance is REQ-ysyu9g. |

## Dev-only dependencies

Not in the shipped closure, listed because two of them run at this unit's gate
and one shapes what the gate can see.

*Corrected 2026-10-04 by review round 2: the `bolero` row said "the two fuzz
targets" and named two of the three. `fuzz_first_admission_base` arrived with
`master` `0f85cb9` while this change was open, is declared in `Cargo.toml`,
named in `verify_commands` and listed in the verification record's gate table
— and this file, the one thing in the change nobody re-measured after that
merge, still described the tree as it was before it.*

| Name | Version | Role | Note |
|---|---|---|---|
| `bolero` | 0.13.4 | The three fuzz targets, `fuzz_envelope_decode`, `fuzz_verify_against_chain` and `fuzz_first_admission_base`, all `harness = false` | Matches `on-chain-client`'s pin. These targets print an iteration total and **no `test result:` line**, so they contribute nothing to this unit's pass count and a reader comparing counts must know that. |
| `rand` | 0.8.8 | Seeded RNGs in tests | 0.8 to match `rand_core` 0.6. |
| `iroh` (`test-utils`) | 0.98.2 | `run_relay_server()` for the hermetic `transport_networked` target | Also reachable from any build enabling this unit's `test-support` feature, which is why that feature is stated never to ship. |
| `jsonrpsee`, `serde_json`, `libc`, `subxt`, `subxt-signer`, `on-chain-client`, `async-trait`, `hex`, `tokio` (`time`) | see manifest | The chopsticks harness for `chain_genesis_e2e` | These support the three targets **excluded** from `verify_commands`, which is the reason SDD-z85ux9 carries no low-level requirements. |
| `org-members` | path | Re-declared as a dev-dependency | An integration-test crate cannot see the library's normal dependencies, so `fuzz_verify_against_chain` could not otherwise name `org_members::trie::OrgTrie`. |

## What a reader should check when a dependency changes

1. `subxt` and `subxt-signer` move **together with `on-chain-client`'s pins**, or
   the two units' `subxt` types stop unifying.
2. `ed25519-dalek` and `rand_core` move together — the RNG trait version is the
   coupling, not the signature algorithm.
3. An `iroh` minor bump re-opens PR-d4nye8's question, because the defect there
   was the library's default bind behaviour and not this unit's code. The two
   loopback tests in `transport_handshake` are what answer it.
4. Anything touching `argon2`, `chacha20poly1305` or `postcard` reaches the
   persona store, which holds every secret this node has (HAZ-45ucqx).
