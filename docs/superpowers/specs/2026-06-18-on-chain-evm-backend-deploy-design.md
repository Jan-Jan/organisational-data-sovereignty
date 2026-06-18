# ODS on-chain — Switch the Recommended Deploy to revive's EVM (REVM) Backend

**Status:** design / spec
**Date:** 2026-06-18
**Author:** brainstorming session (Jan-Jan + Claude)
**Affects:** `on-chain/README.md` (only)

## 1. Problem

`pallet-revive` on Asset Hub is a **dual-VM stack**: it executes both PolkaVM
(PVM) bytecode *and* standard EVM bytecode. The EVM path runs on **REVM** (a
Rust EVM); Solidity compiled by the stock `solc` deploys unchanged through the
chain's **Ethereum JSON-RPC proxy** ("eth-rpc") with ordinary Foundry/Hardhat
tooling — no `resolc`, no PolkaVM blob, no `revive.instantiateWithCode`
extrinsic, no `Revive.PristineCode` hash dance.

Today `on-chain/README.md` documents *only* the PVM path: compile with
`resolc` to a `.pvm` blob, deploy via `@polkadot/api`
(`scripts/deploy-live.mjs`), and verify `Revive.PristineCode[keccak256(blob)]`.
That path works but is the harder of the two and demands a bespoke toolchain
(`resolc` + a pinned standalone `solc`) and a bespoke deploy script.

We want the **EVM path to be the recommended, documented default**, while
keeping the PVM path available as a clearly-labeled advanced alternative.

## 2. Goal & non-goals

**Goal.** Rewrite the live-deploy documentation so the recommended flow
deploys the standard `solc` EVM bytecode (the existing `forge build` artifact)
via the eth-rpc endpoint using plain Foundry (`forge create` + `cast` +
`forge verify-contract`). Keep the existing PVM flow intact as an advanced
option.

**Non-goals / explicitly out of scope:**

- **No contract changes.** `src/OrgRegistry.sol` is untouched.
- **No Rust changes.** `on-chain-client` and the Phase 2 `org-node` / preflight
  read contract state at the ABI level through `pallet-revive`'s call API,
  which is backend-agnostic; an EVM-deployed instance reads identically to a
  PVM-deployed one. (See §5.)
- **No new deploy script.** The EVM path uses `forge create` directly — there
  is deliberately no JS/forge-script equivalent of `deploy-live.mjs` to
  maintain.
- **No removal of the PVM path.** `resolc`, `scripts/deploy-live.mjs`,
  `scripts/sanity-deploy.mjs`, `scripts/chopsticks-sanity.sh`, and the
  `PristineCode` verification all stay; they are relabeled as the PVM
  alternative, not deleted.
- **No change to the Stage-1 chopsticks gate.** It continues to validate the
  PVM artifact.

## 3. Background facts (verified against the Polkadot docs, 2026-06-18)

- **Dual VM:** `pallet-revive` runs both EVM (REVM) and PVM bytecode; both share
  RPC interfaces, tooling, and precompiles. (`dual-vm-stack`, `evm-vs-pvm`.)
- **eth-rpc HTTP endpoints** (distinct from the WSS substrate endpoints — the
  WSS URLs are *not* behind the eth-rpc proxy):

  | Network | eth-rpc URL (Parity) | Chain ID | Token | Foundry `--chain` |
  |---|---|---|---|---|
  | Paseo Asset Hub ("Polkadot Hub TestNet") | `https://eth-rpc-testnet.polkadot.io/` | `420420417` | PAS | `polkadot-testnet` |
  | Polkadot Asset Hub ("Polkadot Hub") | `https://eth-rpc.polkadot.io/` | `420420419` | DOT | `polkadot` |

  (The Paseo AH substrate WSS `wss://asset-hub-paseo-rpc.n.dwellir.com` used by
  the current README is the same chain; the EVM path uses the HTTP eth-rpc URL
  instead.)
- **Foundry nightly** is required for the native `--chain polkadot*` shortcuts
  and for Blockscout verification: `foundryup --version nightly`. (Deploy via
  an explicit `--rpc-url` works without nightly; the `--chain` shortcut and
  `forge verify-contract` benefit from it.)
- **Key type:** EVM deploys are signed by a **secp256k1 / Ethereum (H160)**
  private key (`0x…` 32-byte hex), *not* the sr25519 key
  `deploy-live.mjs` defaults to. A native sr25519/ed25519 account would require
  account-mapping; using an Ethereum keypair avoids that entirely.
- **Funding:** PAS from `https://faucet.polkadot.io/` (select Polkadot Hub
  TestNet) for Paseo; real DOT for mainnet. Deployer must hold the existential
  deposit plus fees.
- **Contract specifics:** `OrgRegistry` has **no constructor arguments**
  (default constructor; ABI has no `constructor` entry), so `forge create`
  needs no `--constructor-args`. Compiler settings are already pinned in
  `foundry.toml` (`solc_version = "0.8.27"`, `optimizer_runs = 200`,
  `evm_version = "cancun"`), which `forge verify-contract` reuses.
- **No view function:** the `orgs` mapping is `private` and the only external
  function is `update(bytes32,bytes32,uint256)` (state-changing). There is no
  getter to `cast call`. State is read out-of-band via raw storage slots (see
  §5), so the EVM-path confirm step uses `cast code` + `forge verify-contract`
  rather than a `cast call` view read.
- **Size:** EVM code-size limit is 24 KB on the Hub; `OrgRegistry` is well under.

## 4. The README rewrite

`on-chain/README.md` is the only file changed. The "Compiling and deploying to
a live chain" section is restructured: **EVM (recommended)** first, **PVM
(advanced)** second.

### 4.1 Intro reframe (new short paragraph)

State that `pallet-revive` is a dual-VM stack — REVM runs standard EVM bytecode
(`solc` output) and PolkaVM runs `resolc` output; both share the same RPC and
tooling. EVM is the simpler default; PVM is for performance-critical / RISC-V
workloads. Replaces the current "pallet-revive runs PolkaVM bytecode, not EVM
bytecode" framing, which is now outdated.

### 4.2 EVM path (new, primary)

1. **Prerequisites (once):** Foundry nightly —
   `curl -L https://foundry.paradigm.xyz | bash && foundryup --version nightly`;
   verify `forge --version`. No `resolc`, no standalone `solc` pin (Foundry
   manages `solc 0.8.27` per `foundry.toml`).
2. **Fund a deployer:** generate/reuse an Ethereum-style (secp256k1, H160)
   account. Paseo: PAS from the faucet (no real value). Mainnet: real DOT,
   double-check the address. Note (with link) that a native sr25519 account
   would need account-mapping; an Ethereum keypair sidesteps it. Export
   `PRIVATE_KEY=0x…` (never commit).
3. **Compile:** `forge build` (the existing EVM artifact in `out/`).
4. **Deploy** — both networks, e.g. Paseo:
   ```bash
   cd on-chain
   forge create src/OrgRegistry.sol:OrgRegistry \
     --rpc-url https://eth-rpc-testnet.polkadot.io/ \
     --private-key "$PRIVATE_KEY" \
     --broadcast
   ```
   Mainnet: swap `--rpc-url https://eth-rpc.polkadot.io/`. Mention the
   `--chain polkadot-testnet` / `--chain polkadot` shortcuts (Foundry nightly)
   as an alternative to `--rpc-url`. Output prints `Deployed to: 0x…`.
5. **Confirm + verify:**
   - `cast code <addr> --rpc-url <eth-rpc>` returns non-empty bytecode (proves
     the contract deployed).
   - `forge verify-contract <addr> src/OrgRegistry.sol:OrgRegistry --chain polkadot-testnet`
     (Blockscout; no API key) for source verification on the explorer.
   - There is no view function to `cast call` (the `orgs` mapping is private);
     the functional state read is exercised by `on-chain-client` /
     the Phase 2 preflight (`check_contract` → `get_org_state`) once pointed at
     the new H160 — note this rather than offering a non-existent view read.
6. **Record the H160** — same role as before: the single shared contract
   instance every org uses, and the value `on-chain-client` / the Phase 2 app
   are pointed at.
7. **Gotchas:** endpoints move (swap eth-rpc URLs from the connect docs);
   PAS has no value / DOT is real; existential deposit means the deployer needs
   a small balance buffer beyond fees; `forge create` waits for the tx receipt
   (inclusion) — for mainnet re-check the address survives a few blocks before
   trusting it; never commit `PRIVATE_KEY`.

### 4.3 PVM path (existing content, relabeled)

The current resolc → `tmp/revive/*.pvm` → `deploy-live.mjs` →
`Revive.PristineCode` verification content moves under a heading like
**"Advanced: deploy as a PolkaVM (PVM) contract"**, kept verbatim, with a
one-line lead-in on when to choose it (performance-critical / RISC-V). The
`scripts/deploy-live.mjs` Layout entry is relabeled as the PVM-path deploy
script.

### 4.4 Untouched sections

The **Layout** list (except the relabel above), the **Stage-1 gate**, the
**Quickstart**, and **What Stage 2 adds** sections remain. The Stage-1
chopsticks gate continues to validate the PVM artifact.

## 5. Downstream-impact analysis (why nothing else changes)

- **`on-chain-client` reads (the load-bearing point):** state is read via the
  `ReviveApi::get_storage(address, slot)` runtime call, where the crate computes
  the **Solidity storage slot** itself — `keccak256(abi.encode(uint256(admin),
  uint256(0)))` for `orgs[admin]`, plus consecutive slots `S+1`/`S+2` for the
  struct fields (`on-chain-client/src/client.rs`). This layout is assigned by
  the Solidity frontend and is **identical** whether the contract was compiled
  by `resolc` (PVM) or `solc` (EVM) — slot assignment is a Solidity-language
  concept both honor. Empirical proof: the existing chopsticks e2e already reads
  a `resolc`/PVM-deployed `OrgRegistry` using these EVM-layout slots and passes,
  so an EVM/REVM-deployed instance (EVM layout by definition) reads identically.
  No `on-chain-client` change.
- **Code-hash verification:** the `Revive.PristineCode[keccak256(blob)]` check
  lived *only* in the PVM deploy script (`deploy-live.mjs`). The EVM path's
  equivalent assurance is `cast code` (code present) + `forge verify-contract`
  (source verification on the explorer). No shared code depends on the
  PristineCode check.
- **Phase 2 `org-node` / preflight:** `check_contract` calls `get_org_state`
  through the same backend-agnostic read path; unaffected.
- **Contract address:** still an H160. EVM `CREATE` derives it from
  deployer EOA + nonce (vs the PVM script's CREATE1/CREATE2-by-salt); both yield
  an H160 the rest of the stack consumes identically.

## 6. Testing & verification

This change is documentation; the bar is "the documented commands are correct
and nothing regressed":

- `forge build` succeeds and `forge test` stays green (15 tests) — proves the
  EVM artifact compiles and behaves; this is the artifact the new path deploys.
- Doc self-review: no dead links, no stale "PolkaVM, not EVM" claims left in the
  EVM section, resolc references confined to the PVM section.
- Endpoints + chain IDs cross-checked against the authoritative Polkadot docs
  (done in §3).
- The live `forge create` against Paseo (funded H160 + network) remains a manual
  operator step, exactly as the live deploy was before this change.

## 7. Risks

- **Foundry nightly drift.** `--chain polkadot*` and verification rely on the
  nightly build; the README must say nightly explicitly and show the explicit
  `--rpc-url` form as the version-independent fallback for the deploy itself.
- **Endpoint churn.** Public eth-rpc URLs can move; the README points at the
  connect-docs endpoint list as the source of truth.
- **`evm_version = cancun`.** Already in `foundry.toml` and accepted by REVM on
  the Hub; no change, but verification must compile with the same settings
  (it reads `foundry.toml`, so this is automatic).
