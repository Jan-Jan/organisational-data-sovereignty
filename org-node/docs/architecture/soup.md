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

*Corrected 2026-10-05 by the app architecture change: the sentence above is
wrong about two of the four. `app/src-tauri` and `on-chain-client` each open
their manifest with an empty `[workspace]` table, so each is its own workspace
root and cargo reads each one's own lock, `app/src-tauri/Cargo.lock` and
`on-chain-client/Cargo.lock`. The root lock has no `ods-poc` entry. Its entries
cover the root workspace's members, and on-chain-client only as org-node's path
dependency, at the root lock's versions. The app's own inventory,
`app/docs/architecture/soup.md`, records the app lock and the version skew
against on-chain-client's lock.*

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

*Re-measured 2026-10-06 on change `worktree-org-node-chain-authority` (T17),
after the chain write and the invitation exchange left org-node, with the same
feature set and count — `cargo tree -p org-node --features app,test-support
--edges <kinds> --prefix none --no-dedupe | awk '{print $1" "$2}' | sort -u |
wc -l`: `normal` **435**, `normal,build,dev` **471**, so dev and build edges
still add 36. Four of the 435 are not SOUP — `org-node`, `org-members`,
`on-chain-client` and `person` (a unit since master's person change) — leaving
**431 third-party crates**. The direct dependencies measured with `--depth 1`
are the seventeen the amendment under "Direct dependencies" predicts.*

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

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The chain write leaves org-node for on-chain-client's chain writer, and the
invitation exchange for the app. With them leave four direct dependencies:
`subxt-signer` (the sr25519 signing key), `blake2` and `parity-scale-codec`
(the multisig account derivation in `chain_write/multisig.rs`, their only
user), and `base64` (the armour in `blobs.rs`, its only user). Their rows below
are kept, marked as leaving, so the history of what this unit shipped stays
readable; `subxt-signer` and `blake2` are now SOUP of on-chain-client
(`on-chain-client/docs/architecture/soup.md`). After the change the count is
**seventeen direct dependencies, of which fourteen are SOUP**, and the closure
figures above are to be re-measured when it is implemented (done 2026-10-06,
see the note under the table). *(Recomputed
2026-10-06 at the merge of master `5f7c177`: this said fourteen and twelve,
counted from eighteen and sixteen; master adds `curve25519-dalek`, `zeroize`
and the `person` unit, so the four leaving take twenty-one and eighteen to
seventeen and fourteen.)* The rows for
`ed25519-dalek`, `serde`, `subxt`, `tokio`, `async-trait` and `hex` are
amended for what their users become.

*Amended 2026-10-08 (ruling B, change `worktree-org-io-create`).* org-node
reads no chain: org-io holds the chain connection, the reader and the
writer, and hands org-node each Organisation's state as a value. With the
`chain` feature leave the direct dependencies `subxt`, `async-trait` and
`hex` (their rows below are kept and marked as leaving) and the path
dependency `on-chain-client` (Not SOUP, below); the chopsticks
dev-dependencies leave with the preflight's chain checks. They are org-io's
SOUP now (`org-io/docs/architecture/soup.md`). `tokio` stays, behind
`transport`. After the change the count is **thirteen direct dependencies,
of which eleven are SOUP** (`org-members` and `person` are units).

Twenty-one, of which eighteen are SOUP (eighteen and sixteen on master; this
branch adds `curve25519-dalek`, `zeroize` and the `person` unit). *(Corrected
2026-10-05 by review round 3, finding-16: this said twenty and seventeen,
leaving out `zeroize`, which has its row below.)* Each row's "requirements it supports" names
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
| `ed25519-dalek` | 2.2.0 | The DevicePublicKey: the ed25519 verifying key that is both a device's identity in the trie and the iroh endpoint identity. `keys.rs`, `transport/endpoint.rs` (*corrected 2026-10-05 after the merge of master `1feb608`: this listed `service.rs` too, which reaches the library only through `keys.rs`*). *Merged 2026-10-05 into worktree-person-shared-types:* this row used to begin "Every signature in the unit: the Member-as-a-group key that signs a delta transcript"; on this branch the Envelope carries no signature (REQ-ag6kqm, amended in place), the Member-as-a-group key is X25519 (`curve25519-dalek`, below), and org-node signs and verifies nothing with this library — `envelope.rs` and `verify.rs` no longer use it. | REQ-ag6kqm, REQ-xa6smf, REQ-ztdza4 | Built with `zeroize`. The iroh handshake authenticates the DevicePublicKey, converted at `transport/endpoint.rs` by `VerifyingKey::from_bytes`; nothing compares it with a record (REQ-ag6kqm). That conversion is the library's point-decompression only, and no signature path of this library is reached from org-node. *Merged 2026-10-05 (master `1feb608`):* master's row records the non-strict `VerifyingKey::verify` in `keys.rs` (PR-vkw22m); `keys::verify` does not exist on this branch, so no signature check of either kind is reached, and PR-vkw22m is resolved as moot by this branch. Master's row also records two facts that hold here: `SigningKeypair` derives `Debug` over `SigningKey`, so org-node relies on `SigningKey`'s own `Debug` omitting the secret (2.2.0 renders only the verifying key) for LLR-bwb9pu (amended in place, keeping the reliance) — `a_signing_key_pair_never_renders_its_seed` in `tests/secret_redaction.rs` guards it, and an upgrade must keep that test green or give `SigningKeypair` a redacting `Debug` of its own; and the pre-release `ed25519-dalek` 3.0.0-pre.6 is also in the normal closure, reached only through `iroh` 0.98.2 / `iroh-base` 0.98.0 (transport). *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): org-node signs and verifies nothing now — the Envelope carries no signature, and `keys::verify` and `SigningKeypair::sign` are removed — so the crate's role is key generation, the DevicePublicKey a seed derives, and the endpoint identity; `envelope.rs` and `verify.rs` no longer use it, and REQ-ag6kqm and REQ-gju89b no longer rest on it. The `verify`/`verify_strict` divergence (PR-vkw22m) has no caller left in org-node; the `Debug` reliance for LLR-bwb9pu stands.* |
| `curve25519-dalek` | 4.1.3 | Direct dependency of org-node: `MontgomeryPoint::mul_base_clamped`, which `X25519Keypair::public_bytes` (`org-node/src/keys.rs`) calls to derive the X25519 public key of a member seed and of the Organisation private key (RFC 7748 clamping, then multiplication by the base point). Also reached through `ed25519-dalek` for every DevicePublicKey. | REQ-ech45n | **Safety-relevant.** A wrong multiplication would place a Member-as-a-group key, or publish an Organisation public key, that does not belong to the secret the node holds: key agreement against that member key fails, and a member could not confirm an Organisation private key shared with it (not yet implemented in org-node). Every derived key is re-checked before use by `person`'s X25519 validity rule (`PersonPublicKey::parse`, and from REQ-8jb4ny `OrgPublicKey::parse`). The RFC 7748 §6.1 vectors are a gated test in `tests/key_custody.rs` (relocated from `keys.rs` when master's architecture tooth moved every `src` test module out). The 5.0.0-pre.6 pre-release in the lockfile is not reached from org-node. |
| `zeroize` | 1.9.0 | Direct dependency of org-node since 2026-10-05 (review round 2, finding-22): `X25519Keypair` (`org-node/src/keys.rs`) implements `Zeroize` and `ZeroizeOnDrop`, so a member seed or an Organisation private key held in one is overwritten with zeros when it is dropped. The same version the lockfile already resolved through `ed25519-dalek`'s `zeroize` feature. | LLR-98ufry | Best effort, as the library states: it wipes the value it is given, not copies the compiler or the caller made. The persisted seeds — `PersonaRecord.member_seed` and `OrgRecord.org_private_key`, held since the merge of master `1feb608` in the redacted secret types `MemberSeed` and `OrgPrivateKey` (`types.rs`), and the copies `X25519Keypair::member_seed` / `org_private_key` return — are `Clone` and not wiped on drop (owner ruling 2026-10-04 on the secret types, RC-jjsz97 residual). |
| `iroh` | 0.98.2 | The QUIC transport. The endpoint's `EndpointId` *is* the device's verifying key, so a completed handshake is custody evidence. `transport/endpoint.rs`, `service.rs` | REQ-db6s7q, REQ-2wzfzv, REQ-ztdza4, REQ-nhe2zu, REQ-txvtm9 (*added 2026-10-05 by review round 2; amended the same day by docs/plans/2026-10-05-switch-trim.md: REQ-txvtm9 states the epoch rule beside REQ-nhe2zu and supersedes nothing*) | Pinned to 0.98 deliberately (Phase 2 note). Pulls `ed25519-dalek` 3.0.0-pre.6, a pre-release cryptographic dependency, via `iroh` and `iroh-base` 0.98.0. **The builder binds pre-configured wildcard sockets when no bind address is named** — this was PR-d4nye8, fixed in `f635acc`, and it is a property of the library's defaults rather than a bug in it. `RelayMode::Disabled` on the Loopback arm is load-bearing and unverified: a home relay is acquired only after `online()`, so an assertion at bind time is vacuous. `0.98` exposes no relay-map accessor. `test-support` pulls `iroh/test-utils`, which brings `CaRootsConfig::insecure_skip_verify()` into any build enabling that feature. |
| `chacha20poly1305` | 0.10.1 | XChaCha20-Poly1305 AEAD over the whole persona store file. `store.rs` | REQ-hzm4kt | A fresh 24-byte nonce is drawn per save from the caller's RNG; nonce reuse would be catastrophic and is not defended by the library. The derived key is held in `PersonaStore` as plain bytes and is **not** zeroized on drop. |
| `argon2` | 0.5.3 | Passphrase-to-key derivation for the store. `store.rs` | REQ-hzm4kt | Uses `Argon2::default()` — the library's default parameters, not parameters this project chose or recorded. The salt is a **fixed 32-byte application constant**, not per-store random (PoC simplification S9, stated in `store.rs`). Both are deliberate PoC choices and both weaken the KDF against a precomputation attack across installations. |
| `postcard` | 1.1.3 | The wire and at-rest encoding for every structured value: envelopes, deltas, store data, blobs, ids, wire frames. `envelope.rs`, `store.rs`, `blobs.rs`, `ids.rs`, `transport/wire.rs` | REQ-9g6as6, REQ-bcxz96, REQ-eg5j8u | The decode path is the unit's largest untrusted-input surface. Three bolero fuzz targets at this unit's gate assert it neither panics nor aborts on arbitrary bytes. Postcard is not self-describing, so a length or variant mismatch is a decode error rather than silent reinterpretation — which is what REQ-9g6as6 and REQ-bcxz96 rest on. `de::Error::custom` discards its message (`SerdeDeCustom`), so a refusal inside a nested `Deserialize` cannot name its field; `store.rs`, `blobs.rs` and `service.rs` therefore decode `Raw…` mirrors and parse them (LLR-8bum44). The stored, sent, blob and calldata bytes are pinned by `tests/encoding_golden.rs` (LLR-ayrdr8). *(Amended 2026-10-05 after the merge of master `1feb608`: LLR-8bum44 and LLR-ayrdr8 are amended in place, docs/plans/2026-10-05-switch-trim.md.)* *Amended 2026-10-06 (independent review round 2, finding-9): `blobs.rs` and the blob bytes left org-node in T7 of change `worktree-org-node-chain-authority` (owner ruling 2026-10-05: the invitation exchange is the app's), and the EVM calldata left in T13 for on-chain-client. The row's role and file list above predate that; postcard's users now are `envelope.rs`, `store.rs`, `service.rs` and `transport/wire.rs`, and `tests/encoding_golden.rs` pins the Persona store plaintext and the admission Wire message with its record snapshot.* |
| `serde` | 1.0.229 | Derive layer under `postcard`. `blobs.rs`, `store.rs`, `envelope.rs`, `ids.rs`, `types.rs`, `transport/wire.rs` | REQ-9g6as6, REQ-bcxz96 | The risk is in use — a derived `Deserialize` on a validated type bypasses its constructor — so the secret and tag types in `types.rs` use `serde(transparent)`, `OrgPublicKey` a hand-written `Deserialize` through its `parse`, and the records with a fallible field `serde(try_from = "Raw…")` (`store.rs`'s four records and `StoreData`, `blobs.rs`'s `JoinRequest`); those maintain decode invariants by hand, through the types' own `parse`. *Merged 2026-10-05 into worktree-person-shared-types:* master's row also names a hand-written `sig_bytes` `Visitor` for the 64-byte Envelope signature, whose `visit_seq` entry point did not enforce the length. The Envelope on this branch has no signature field, and `sig_bytes` is gone with it, so no hand-written `Visitor` remains in this unit. *(Corrected 2026-10-05 after the merge of master `1feb608`: of the types listed above, only the four records in `store.rs` and `JoinRequest` carry `serde(try_from = "Raw…")`; `StoreData` derives `Deserialize`, and `PersonaStore::open` decodes its `RawStoreData` mirror and converts it by `TryFrom`, so a refusal names its field. The `Invite` has no mirror: its three keys are parsed by their own types' `Deserialize` — `OrgPublicKey`'s hand-written one (LLR-mmdu38) and `person`'s for the Member-as-a-group key and the DevicePublicKey — so a refused key fails the decode and `import_invite` reports `Chain("blob decode: …")`, without the field's name (LLR-8bum44).)* *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): the Envelope's `signature` field and its `sig_bytes` helper are removed, so the hand-written `Visitor` and its weaker `visit_seq` entry point leave the unit; `blobs.rs` leaves too. The remaining hand-maintained decode invariants are `OrgPublicKey`'s `Deserialize` and the `Raw…` mirrors in `store.rs`, which now include the provisional updates' records.* |
| `base64` | 0.22.1 | Copy-pasteable armour around `postcard` for out-of-band invites and join requests. `blobs.rs` | REQ-9g6as6 | Standard alphabet with padding. A decode failure is a typed error, not a panic. *Left org-node 2026-10-06 (T7; owner ruling 2026-10-05, change `worktree-org-node-chain-authority`): `blobs.rs`, its only user, is removed with the invitation exchange, which the app owns.* |
| `thiserror` | 2.0.20 | The typed rejection vocabulary. One variant per rejection path **on the receive-and-commit path**; elsewhere in the unit forty-five sites share `OrgNodeError::Chain(String)` (fifty before the org-node type-safety change). `error.rs`, `transport/mod.rs`, `chain_write/mod.rs` | REQ-9g6as6, REQ-bcxz96, REQ-eg5j8u | Derive-only; generates `Display`/`Error` and no control flow. **Corrected 2026-10-04 by review round 3**, which measured the fifty `Chain(String)` sites: this row said "one variant per rejection path" flatly, as did the ledger, and that is true only of verify-against-chain. `Chain`'s `Display` is `"chain read failed: {0}"`, so a wrong store passphrase and a malformed invite blob both render as chain read failures. Recorded as a gap, not closed here. *(Amended 2026-10-05 by the org-node type-safety change, review round 7: re-measured on that change's tree as forty-five sites, `grep -o 'OrgNodeError::Chain(' -r org-node/src \| wc -l`, as SDD-swtd3w's amendment states. Five refusals now name the field (`InvalidField`) or the key (`InvalidKey`).)* *(Merged 2026-10-05 into worktree-person-shared-types: `InvalidKey` is not on this branch; the Organisation public key is refused as `InvalidOrgPublicKey`.)* |
| `rand_core` | 0.6.4 | The RNG trait boundary. Key generation and store nonces take `R: RngCore + CryptoRng` from the caller rather than sampling the OS directly, which is what makes those paths deterministic under test. `keys.rs`, `store.rs`, `service.rs` | REQ-hzm4kt | 0.6, one major behind `rand` 0.9's `rand_core` 0.9 — pinned to match `ed25519-dalek` 2.x. **`CryptoRng` is a marker trait the compiler cannot enforce the meaning of**: a caller may pass a seeded test RNG into a production path and nothing in this unit objects. |
| `subxt` | 0.50.3 | *Leaves 2026-10-08 (ruling B; org-io's now).* Every chain read and write: RPC client, extrinsic construction, event decoding. `chain_write/*`, `ceremony.rs`, `preflight.rs`, `service.rs` | — (supports org-io's chain-connection item (moved 2026-10-07), which carries no LLRs) | Features mirror `on-chain-client`'s pins so the `subxt` types unify across the two crates; they must be changed together or the units stop composing. Everything built on it sits in the I/O shell that this unit's gate does not reach. *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): org-node builds and submits no extrinsic now; it uses `subxt` to open the RPC connection (`connect_chain_client`), for the preflight checks, and through on-chain-client's reader for `read_state`. `chain_write/*` and `ceremony.rs` leave for on-chain-client.* |
| `subxt-signer` | 0.50.3 | sr25519 signing for extrinsic submission and the dev accounts the ceremony uses. `chain_write/*`, `ceremony.rs`, `service.rs` | — (supports org-io's chain-connection item (moved 2026-10-07)) | Pulls a bip39/schnorrkel stack for a key type **this unit does not otherwise use**: org-node's DevicePublicKeys are ed25519 and its Member-as-a-group keys and Organisation public key X25519, and sr25519 appears only on the chain-submission side. *Left org-node 2026-10-06 (T13; owner ruling 2026-10-05, change `worktree-org-node-chain-authority`): org-node drops its sr25519 multisig key with the chain write; the crate is now SOUP of on-chain-client, behind its `write` feature. No chopsticks target that signs remains in org-node, so it is not an org-node dev-dependency either.* |
| `parity-scale-codec` | 3.7.5 | SCALE encoding of the multisig pseudo-account preimage. `chain_write/multisig.rs` | — | Used for exactly one derivation. A SCALE layout change would silently move the derived account rather than fail, and a gated ordering test pinned the ordering property but not the encoding itself. *Left org-node 2026-10-06 (T13; owner ruling 2026-10-05, change `worktree-org-node-chain-authority`): `multisig.rs`, its only user, moves to on-chain-client, which already depends on the crate; the ordering test went with it as on-chain-client's `write_pure::the_derived_account_does_not_depend_on_signatory_order`.* |
| `blake2` | 0.10.6 | `blake2_256` over that preimage — the pallet-multisig pseudo-account derivation. `chain_write/multisig.rs` | — | Via `Blake2bVar`, whose constructor is fallible; 32 is always valid and the code says so at the call site. Must track the runtime's hasher: a divergence yields a valid-looking address that holds no funds. *Left org-node 2026-10-06 (T13; owner ruling 2026-10-05, change `worktree-org-node-chain-authority`): `multisig.rs`, its only user, moves to on-chain-client; the crate is now SOUP of on-chain-client, behind its `write` feature.* |
| `tokio` | 1.53.1 | The async runtime behind the chain and transport paths. `transport/endpoint.rs`, `service.rs`, `preflight.rs`, `chain_write/multisig.rs`, `bin/preflight.rs` | — | `rt-multi-thread` + `macros`; `time` only as a dev-dependency, so a production build has no timer driver. Nothing in this unit's gated set depends on scheduling order. *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): `chain_write/multisig.rs` leaves the users listed.* |
| `async-trait` | 0.1.92 | *Leaves 2026-10-08 (ruling B: `ChainOps` is deleted).* Object-safe async methods on the `ChainOps` seam, which is what makes the service headless-testable against `MockChainOps`. `service.rs`, `chain_write/proxy.rs` | REQ-wp2nyc, REQ-8gz8bu, REQ-nhe2zu, REQ-txvtm9 (via the seam the mock substitutes at; REQ-txvtm9, which supersedes REQ-nhe2zu, *added 2026-10-05 by review round 2*) | Boxes every call's future. A proc-macro shim for a language feature that is landing natively; removing it later is a mechanical change to the trait, not to callers. *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): `ChainOps` keeps `read_state` only and `chain_write/proxy.rs` leaves, so `service.rs` is the one user.* |
| `hex` | 0.4.3 | *Leaves 2026-10-08 (ruling B, with the chain checks of the preflight).* Operator-facing display of hashes and addresses. `bin/preflight.rs`, `chain_write/submit.rs` | — | Presentation only; no value is parsed back from hex in this unit. *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): `chain_write/submit.rs` leaves, so `bin/preflight.rs` is the one user — and it does parse hex back, from its environment variables, which this cell's "no value is parsed back" did not record.* |

## Not SOUP

| Name | Why |
|---|---|
| `org-members` | A unit of this repository (`.guardrails/units.yaml`), class C, with its own ledgers and its own SOUP inventory. org-node's reliance on it is declared as `depends_on` and what it relies on beyond that unit's stated requirements is REQ-q92yac. |
| `on-chain-client` | Likewise a unit of this repository, class C, declared in `depends_on`; the unstated reliance is org-io's finalised-block expectation (moved 2026-10-07). *Leaves 2026-10-08 (ruling B, change `worktree-org-io-create`): no longer a dependency or in `depends_on`; org-io holds this edge.* |
| `person` | Likewise a unit of this repository, class C, declared in `depends_on`: the identity types and the X25519 validity rule (`person::x25519`) that `PersonPublicKey::parse` and `OrgPublicKey::parse` apply. |

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
| `jsonrpsee`, `serde_json`, `libc`, `subxt`, `on-chain-client`, `async-trait`, `tokio` (`time`) | see manifest | *Leave 2026-10-08 (ruling B), except `serde_json`, kept for `tests/encoding_golden.rs`, and `tokio` (`time`), kept for the async tests; the chopsticks harness went to org-io with the preflight's chain checks.* The chopsticks harness for `preflight` | These support the chopsticks target **excluded** from `verify_commands`, which is the reason org-io's chain-connection item (moved 2026-10-07) carries no low-level requirements. *Amended 2026-10-06 (change `worktree-org-node-chain-authority`, T13 and T17): `chain_genesis_e2e` and `finality_polling` are deleted with the chain write, leaving `preflight` the one chopsticks target; `subxt-signer` left with them, and the `hex` dev-dependency, which no test used, is removed (`hex` stays a normal dependency of `bin/preflight.rs`; the lockfile is unchanged).* |
| `org-members` | path | Re-declared as a dev-dependency | An integration-test crate cannot see the library's normal dependencies, so `fuzz_verify_against_chain` could not otherwise name `org_members::trie::OrgTrie`. |

## What a reader should check when a dependency changes

1. `subxt` and `subxt-signer` move **together with `on-chain-client`'s pins**, or
   the two units' `subxt` types stop unifying. *(Amended 2026-10-05, owner
   ruling, change `worktree-org-node-chain-authority`: `subxt-signer` leaves
   org-node; `subxt` alone must still move with on-chain-client's pin.)*
   *(Amended 2026-10-08, ruling B: `subxt` leaves org-node too; the coupling
   is org-io's.)*
2. `ed25519-dalek` and `rand_core` move together — the RNG trait version is the
   coupling, not the signature algorithm. `curve25519-dalek` moves with
   `ed25519-dalek`, which depends on it; a bump re-runs the RFC 7748 vectors in
   `key_custody`.
3. An `iroh` minor bump re-opens PR-d4nye8's question, because the defect there
   was the library's default bind behaviour and not this unit's code. The two
   loopback tests in `transport_handshake` are what answer it.
4. Anything touching `argon2`, `chacha20poly1305` or `postcard` reaches the
   persona store, which holds every secret this node has (HAZ-45ucqx).
