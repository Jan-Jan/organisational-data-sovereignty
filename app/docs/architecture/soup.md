# SOUP Inventory

<!--
Software Of Unknown Provenance (IEC 62304 §8.1.2): every third-party
component the software depends on. Keep versions exact; review this file
whenever a dependency changes. Note functional/performance requirements the
SOUP must meet and known anomalies relevant to safety.
-->

## Evidence and provenance

Measured 2026-10-05 on branch `worktree-guardrails-app-arch`, from local
`master` at `06c357b`, on the host target `aarch64-apple-darwin`. Every cargo
command ran with `--locked`.

**`app/src-tauri` is its own workspace.** Its `Cargo.toml` opens with an empty
`[workspace]` table, and the root `Cargo.toml` neither lists nor excludes it.
`cargo locate-project --workspace` run from `app/src-tauri` names
`app/src-tauri/Cargo.toml`, so cargo reads **`app/src-tauri/Cargo.lock`**. That
lock is tracked and has 806 package entries. The root `Cargo.lock` does not
contain the app: there is no `ods-poc` entry in it. This is the same arrangement
as `on-chain-client`'s and the opposite of `org-node`'s.

**Closure.** The command was:

```
cargo tree --manifest-path app/src-tauri/Cargo.toml -p ods-poc \
  -e <edges> --prefix none --format '{p}' --locked \
  | awk '{print $1,$2}' | sort -u | wc -l
```

It counts unique `name version` pairs only, never whole lines.

| Edge kinds | Host target (`aarch64-apple-darwin`) | `--target all` |
|---|---|---|
| `normal` | 499 | 672 |
| `normal,build,dev` | 525 | 707 |

Four pairs in each list are units of this repository, not SOUP: `ods-poc`
itself, `org-node`, `org-members` and `on-chain-client` (`person` is not in the
closure). Subtracting them leaves **495 third-party crates shipped on the host
target**, and 521 with build and dev edges. The `--target all` figures are the
raw counts and include the same four units. The difference from the host
figures is Tauri's platform stacks: webkit2gtk and gtk on Linux, webview2 and
the `windows-*` crates on Windows. The host figures exclude them.

The `test-support` feature, which the gate builds, gives byte-identical lists.
`test-support = []` enables nothing in any dependency, unlike org-node's, which
pulls `iroh/test-utils`.

Against org-node's closure (`cargo tree -p org-node --features app -e normal`
on the root lock, 387 pairs), the app adds 112 pairs (the Tauri stack). Every
one of org-node's 387 is in the app closure at the same version, so the two
locks do not drift on org-node's closure.

**npm.** `app/package-lock.json` is tracked (lockfileVersion 3) and holds
**119 packages**. One is a runtime dependency (`@tauri-apps/api`) and 118 are
flagged `dev`. **The `dev` flag does not mean unshipped.** The svelte runtime,
the `@sveltejs/kit` client runtime and their runtime dependencies (`devalue`,
`esm-env`, `clsx`) are compiled into `build/`, which Tauri embeds in the
binary. The shipped JS closure is larger than one package. It is best measured
from the built bundle, and this inventory has not done that.

**Advisory scans.** No `cargo-audit` or `cargo-deny` is installed in this
toolchain, so no Rust advisory scan was run. That is a gap, not a finding of no
advisories, and it is the gap org-node's and on-chain-client's inventories
record. `npm audit --omit=dev --prefix app` reports **0** vulnerabilities.
`npm audit --prefix app`, which includes the dev flag, reports **5**: 1 low,
1 moderate, 3 high, 0 critical. Of those, `devalue` 5.8.1 (high) is a runtime
dependency of the kit and svelte client code. `--omit=dev` hides it. It was
not found in the built bundle: after `npm run build` (2026-10-05), none of
devalue's error strings appears in `build/`. Kit uses it in production only to
unflatten server data, which this prerendered `ssr = false` app never fetches,
and to stringify under DEV. It is booked as PR-8vh53d all the same, because no
gate would notice it entering the bundle. The other four are `@sveltejs/kit` 2.65.1
(moderate, server-side paths this static `ssr = false` app does not run),
`cookie` 0.6.0 (low, via kit), and `nanoid` 3.3.12 and `postcss` 8.5.15 (high,
build tools).

## Rust direct dependencies

Usage is from a grep of `app/src-tauri/src`, not from the manifest's comments.
Requirements are from `app/docs/requirements/`.

| Name | Kind | Version | Role in system | Requirements it supports | Risk considerations / known anomalies |
|---|---|---|---|---|---|
| `tauri` | normal | 2.11.5 | Desktop shell: webview window, IPC command dispatch, event emission, `app_data_dir` resolution. `lib.rs`, `commands.rs`, `state.rs` | REQ-645jq9, REQ-e4ah9h, REQ-bvx4nh, REQ-affyf5, REQ-kn5rtx, REQ-dp95pv, REQ-tw4cb5, REQ-jfxah3, REQ-rxc8sp, REQ-cj5jmx (RC-mq2365) | Manifest sets `features = []`; resolved defaults include `dynamic-acl`, `wry`, `compression`. See *Tauri configuration* for the IPC exposure. `lib.rs` panics on a failed `AppState::init` and `.expect`s the run. **Carries RC-mq2365 in part:** Tauri injects `app.security.csp` into the webview and appends hashes of the bundle's inline scripts to `script-src` at build time. The gate checks the configured policy, not what Tauri injects. |
| `tauri-build` | build | 2.6.3 | `tauri_build::build()` in `build.rs`; generates the context and the ACL schemas in `gen/schemas` | — | Called with defaults, so no app ACL manifest is generated for the custom commands. |
| `serde` | normal | 1.0.229 | Derive layer for IPC payloads and event structs. `commands.rs`, `events.rs`, `policy.rs` | REQ-645jq9, REQ-affyf5, REQ-dp95pv, REQ-tw4cb5, REQ-kn5rtx | Derive-only. |
| `serde_json` | normal | 1.0.151 | `serde_json::Value` event payloads emitted to the webview. `events.rs` | REQ-affyf5, REQ-dp95pv, REQ-tw4cb5, REQ-kn5rtx, REQ-jfxah3 | `events.rs:180` `.expect`s `to_value`; it relies on every payload being a plain struct of owned scalars. |
| `tokio` | normal | 1.53.1 | `tokio::sync::Mutex` around `OrgService` (`state.rs`); `tokio::spawn` of the receiver loop (`commands.rs:420`) | REQ-3hfggn, REQ-6hgm8r, REQ-jfxah3 | `sync`, `rt-multi-thread`, `macros`. The mutex serialises every command's access to the service. |
| `hex` | normal | 0.4.3 | Hex encoding and decoding of keys, ids and addresses crossing IPC. `commands.rs`, `parsing.rs`, `state.rs` | REQ-sjkp8z, REQ-vgr7s2 | Unlike org-node, the app **parses** hex from untrusted input: `parsing.rs` and the revoke and admit handlers decode strings from the webview, and `state.rs` decodes `ODS_*` environment values. |
| `base64` | normal | 0.22.1 | Standard-alphabet armour around the postcard bytes of the Invite and Invite-reply Blobs. `invitation.rs` | REQ-yazum3 (and REQ-prjja8, REQ-65xqp8 through the same Blobs) | *Added 2026-10-06 (change `worktree-org-node-chain-authority`).* **Decodes untrusted text** pasted into the webview: a decode failure is a named refusal (`"<Blob>: not a Blob: …"`), never a panic. Was org-node's (`blobs.rs`) until the invitation exchange moved to the app; org-node no longer depends on it. |
| `async-trait` | normal | 0.1.92 | Implements org-node's `ChainOps` seam for `ChainNotConfigured`. `state.rs:72` | — (startup wiring) | Proc-macro shim; boxes every call's future. |
| `rand` | normal | 0.8.8 | `rand::rngs::OsRng`, passed into org-node's key generation and store nonces. `commands.rs:19` and every handler that creates keys or saves the store | REQ-vgr7s2 (revoke), plus the onboarding commands | **Used in production here**; in org-node `rand` is dev-only. This is the one place the shipped product supplies the `CryptoRng` that org-node's `rand_core` row says the compiler cannot vouch for. |
| `postcard` | normal | 1.1.3 | `postcard::from_bytes` of `iroh::EndpointAddr` bytes from a join request (`commands.rs:219`) and from the revoke handler's `peer_addr` (`commands.rs:283`) | REQ-vgr7s2 | **Decodes untrusted bytes** that arrive hex-encoded over IPC from the webview or inside a join-request blob. A decode failure is mapped to an error string, not a panic. No fuzz target in this unit reaches these two calls. *Amended 2026-10-06 (change `worktree-org-node-chain-authority`): the Join request is gone; the peer address is decoded once, by `commands.rs:253` for both the admit and the revoke handlers, and postcard now also encodes and decodes the Invite and Invite-reply Blobs inside their Base64 armour (`invitation.rs`, REQ-yazum3).* |
| `iroh` | normal | 0.98.2 | `EndpointAddr` and `EndpointId::from_bytes` for the peer to dial. `commands.rs:204,215,263` | REQ-vgr7s2 | Types only; the endpoint itself is org-node's. Pinned 0.98 with org-node. |
| `subxt` | normal | 0.50.3 | **No code reference in `app/src-tauri/src`.** Declared only to unify features with org-node and on-chain-client; the client is built by `org_node::service::connect_chain_client` | — (chain setup) | The manifest comment says it is "needed by state.rs connect_chain", which overstates it. Removing it would change feature unification, not code. |
| `subxt-signer` | normal | 0.50.3 | Builds the admin sr25519 keypair from the 32-byte `ODS_ADMIN_SEED`. `state.rs:314` | — (chain setup) | The seed comes from the process environment, hex-decoded in `state.rs`. This path runs only when `ODS_CHAIN_WS` is set and no gated test reaches it. |
| `tauri` (`test`) | dev | 2.11.5 | `tauri::test::mock_builder`, `mock_context`, `get_ipc_response` in `tests/ipc.rs` | — | Feature unification means a dev build of the lib also sees `test`. |
| `tempfile` | dev | 3.27.0 | Temporary data directories in `tests/ipc.rs` and `tests/state_assembly.rs` | — | Test-only. |

`on-chain-client` (normal, path) selects `dev-rpc` and `write`. *Amended
2026-10-06 (change `worktree-org-node-chain-authority`): this said it was
declared only to unify features, with no code reference in
`app/src-tauri/src`. The app now enables on-chain-client's `write` feature and
is the one build that does — it submits every genesis and update through on-chain-client's chain writer
(`submit.rs`, `state.rs`; REQ-nfr3n2) — and it holds the sr25519 signatory key
(`subxt-signer`, from `ODS_ADMIN_SEED`) that it passes to the writer on each
call. The writer stores no key. The `write` feature's two dependencies,
`subxt-signer` and `blake2`, are recorded in on-chain-client's inventory at
the app lock's versions (0.50.3, 0.10.6).*
`on-chain-client`, `org-node` (normal, path) and `org-members` (dev, path) are
units of this repository with their own ledgers, not SOUP.

## npm packages

| Name | Locked | Lock flag | Shipped? | Role | Note |
|---|---|---|---|---|---|
| `@tauri-apps/api` | 2.11.0 | dependency | yes | Webview-to-Rust IPC: `invoke` and `listen` in `src/lib/api.ts` | The only runtime dependency; it has no dependencies of its own. |
| `svelte` | 5.56.3 | dev | **yes** (runtime compiled into `build/`) | UI framework for every component | Pulls `esm-env` 1.2.2 and `clsx` 2.1.1 into the bundle. |
| `@sveltejs/kit` | 2.65.1 | dev | **yes** (client runtime in `build/`) | Router; `ssr = false`, `prerender = true` | Moderate advisories are server-side and not reachable in a static SPA. Its runtime dependency `devalue` 5.8.1 carries a high advisory; not found in today's `build/` (PR-8vh53d). |
| `@sveltejs/adapter-static` | 3.0.10 | dev | no (build) | Emits the static SPA to `build/` | — |
| `vite` | 8.0.16 | dev | no (build) | Bundler and dev server on port 5173 | — |

The remaining dev packages (`vitest`, `typescript`, `svelte-check`,
`@tauri-apps/cli` and their dependencies) are build and test tools and are not
in the bundle.

## Platform webview

Added by review round 1 (finding-8). The webview is not in either lock: Tauri
(through `wry`) uses the one the operating system supplies.

| Name | Supplied by | Version | Role | Requirements it supports | Risk considerations / known anomalies |
|---|---|---|---|---|---|
| Platform webview: WKWebView (macOS), WebView2 (Windows), WebKitGTK (Linux) | The operating system | Not pinned; whatever the host has installed | Renders the frontend and **enforces the Content-Security-Policy** Tauri injects | REQ-cj5jmx (RC-mq2365) | RC-mq2365 holds only as far as this webview enforces CSP. Its CSP Level 3 support differs by engine and version, and no gated test runs one. The policy is kept to directives every engine has long supported (LLR-uus6ar). Whether the bundled app loads under the policy is the owner's smoke test. |

## Inherited SOUP

These app direct crates are also in org-node's inventory at the same versions:
`iroh` 0.98.2, `postcard` 1.1.3, `serde` 1.0.229, `tokio` 1.53.1,
`async-trait` 0.1.92, `hex` 0.4.3, `subxt` 0.50.3, `subxt-signer` 0.50.3 and
`rand` 0.8.8. The crates the app reaches only through org-node
(`ed25519-dalek` 2.2.0 and 3.0.0-pre.6, `chacha20poly1305` 0.10.1, `argon2`
0.5.3, `thiserror` 2.0.20, `rand_core` 0.6.4) are also at org-node's versions.
*(Amended 2026-10-06, change `worktree-org-node-chain-authority`: `base64` is
now a direct dependency of the app (row above), and `parity-scale-codec`
3.7.5 and `blake2` 0.10.6 are reached through on-chain-client's `write`
feature, not org-node; org-node no longer depends on any of the three.)*
Their roles and anomalies are in `org-node/docs/architecture/soup.md` and are not
repeated here. The rows above record only what differs in the app's use.

**Version skew with on-chain-client.** `on-chain-client` has its own lock,
and its inventory records `subxt` and `subxt-signer` **0.50.1**. When the app
builds on-chain-client it uses the app lock, so the app ships `subxt` **0.50.3**.
The same holds for on-chain-client's `tokio` 1.52.3, `serde` 1.0.228 and
`async-trait` 0.1.89. on-chain-client's inventory therefore does not describe
the versions the app ships.

## Tauri configuration

- **No plugins.** There is no `tauri-plugin-*` crate in the manifest or the
  lock, and no plugin is registered in `lib.rs`. There is no filesystem, shell,
  http or dialog access from JavaScript.
- **`core:default` only.** `capabilities/default.json` grants the `main`
  window `core:default`, which expands to the default sets of `path`, `event`,
  `window`, `webview`, `app`, `image`, `resources`, `menu` and `tray`.
- **The 12 app commands are ungated by any ACL.** `generate_handler!` exposes
  `create_persona`, `create_organisation`, `export_invite`, `import_invite`,
  `produce_invite_reply`, `import_invite_reply` (these two replaced the Join
  request commands, 2026-10-06), `admit_member`,
  `revoke_member`, `list_personas`, `list_orgs`, `connection_status` and
  `start_receiver`. `build.rs` calls `tauri_build::build()` with no app
  manifest, so no capability permission gates them. Any script running in the
  webview can call all twelve.
- **A Content-Security-Policy is set.** `app.security.csp` in
  `tauri.conf.json` is the shipped policy (REQ-cj5jmx, implementing RC-mq2365),
  and `app.security.devCsp` is the development policy. Tauri injects the policy
  and appends script hashes at build time; the platform webview enforces it
  (see *Platform webview*). `app/src-tauri/tests/csp_policy.rs` checks both
  configured values.

## What a reader should check when a dependency changes

1. `subxt` and `subxt-signer` move with org-node's and on-chain-client's pins,
   and the app lock must be re-checked against both.
2. A `tauri` bump re-opens the IPC exposure above: check whether the defaults
   still grant the custom commands without an ACL.
3. An npm update can change the bundle even when only dev packages move.
   Re-run `npm audit` **without** `--omit=dev`.
