# `on-chain/` — ODS Phase 1.b Stage 1

Solidity contract anchoring the off-chain organisation-members trie on Asset
Hub via `pallet-revive`. Multi-tenant: one contract instance serves every
organisation, keyed on the H160 of each org's proxied pure-proxy admin.

See `docs/superpowers/specs/2026-05-13-ods-phase-1b-design.md` for the design.

## Layout

- `src/OrgRegistry.sol` — the contract.
- `test/OrgRegistry.t.sol` — Foundry unit tests (covers §5.1 of the spec).
- `abi/OrgRegistry.json` — pinned ABI artifact for Stage 2 consumers.
- `scripts/chopsticks-sanity.sh` — deploys to a chopsticks-forked Paseo
  Asset Hub and verifies the code hash. Gate criterion for Stage 2.
- `scripts/chopsticks-config.yml` — chopsticks config pinning Paseo's
  endpoint.
- `scripts/sanity-deploy.mjs` — Node.js deploy + verify script using
  `@polkadot/api`. Hard-coded to `//Alice`; works only against the
  chopsticks fork (`mock-signature-host`).
- `scripts/deploy-live.mjs` — live-chain deploy + verify. Reads the signing
  key from `$DEPLOYER_SEED`, waits for finalization, and prints the deployed
  contract H160. Use this for Paseo / Polkadot Asset Hub.
- `scripts/wait-for-rpc.mjs` — WS readiness probe used by the sanity harness.
- `scripts/package.json` / `scripts/package-lock.json` — pinned npm
  dependencies for the sanity script (chopsticks, polkadot-api).

## Quickstart

```bash
# Unit tests (no chain required):
cd on-chain && forge test -vv

# Re-pin ABI after a contract change:
forge clean && forge build
jq '{abi: .abi, contractName: "OrgRegistry"}' \
   out/OrgRegistry.sol/OrgRegistry.json > abi/OrgRegistry.json

# Sanity-script deps (run once to materialise scripts/package-lock.json,
# then commit the lockfile so the gate is reproducible):
(cd scripts && npm install)

# Chopsticks sanity (requires resolc + solc + node + npm + jq for the
# ABI re-pin step):
./scripts/chopsticks-sanity.sh
```

## Compiling and deploying to a live chain

`pallet-revive` runs PolkaVM bytecode, not EVM bytecode, so `forge build`'s
EVM artifact is **not** what gets deployed. The on-chain blob is produced by
`resolc` (the Revive Solidity compiler), and deployment goes through
`revive.instantiateWithCode`, not an EVM `CREATE` transaction.

Both Paseo Asset Hub (testnet) and Polkadot Asset Hub (mainnet) use the same
flow; only the RPC endpoint, the token used for fees/deposit, and how you
fund the deployer differ.

### Prerequisites (once)

```bash
# Toolchain: resolc (Revive Solidity compiler) + a matching solc 0.8.27.
# Stage 1 installed both into ~/bin; confirm they're on PATH:
resolc --version
solc --version          # must be 0.8.27 to match foundry.toml / src pragma

# JS deps for the deploy script (pinned via scripts/package-lock.json):
(cd scripts && npm ci)
```

### Step 1 — Compile to a PolkaVM blob

```bash
cd on-chain
mkdir -p tmp/revive
resolc --bin src/OrgRegistry.sol --solc "$(which solc)" -o tmp/revive/ --overwrite

# resolc v1.1.0 emits the source-prefixed name (note the ':' and .pvm ext):
ls -l "tmp/revive/OrgRegistry.sol:OrgRegistry.pvm"
```

This is the exact blob `chopsticks-sanity.sh` already builds; reusing it
keeps the live deploy byte-identical to what passed the Stage 1 gate.

### Step 2 — Fund a deployer account

The deployer pays transaction fees **and** the code storage deposit (the
script reserves up to `10_000_000_000_000` planck). `//Alice` does not exist
on a live chain — generate or reuse a real account and fund it:

- **Paseo Asset Hub (testnet):** get free PAS from the faucet at
  <https://faucet.polkadot.io/> (select network **Paseo**, then route/teleport
  to Asset Hub if the faucet funds the relay). PAS has no real-world value.
- **Polkadot Asset Hub (mainnet):** fund the account with real DOT (enough to
  cover fees + the storage deposit) and **double-check the address** before
  signing — this spends mainnet funds.

Export the secret for the script. It accepts a 12/24-word mnemonic, a
`//Derivation`, or a `0x`-prefixed seed; default key type is `sr25519`
(override with `KEY_TYPE=ed25519` or `ecdsa` if your account differs):

```bash
export DEPLOYER_SEED='word word word ... word'   # NEVER commit this
# export KEY_TYPE=sr25519                          # default; change if needed
```

### Step 3 — Deploy and verify

**Paseo Asset Hub (testnet):**

```bash
cd on-chain
RPC_URL='wss://asset-hub-paseo-rpc.n.dwellir.com' \
BLOB_PATH='tmp/revive/OrgRegistry.sol:OrgRegistry.pvm' \
DEPLOYER_SEED="$DEPLOYER_SEED" \
  node scripts/deploy-live.mjs
```

**Polkadot Asset Hub (mainnet):**

```bash
cd on-chain
RPC_URL='wss://polkadot-asset-hub-rpc.polkadot.io' \
BLOB_PATH='tmp/revive/OrgRegistry.sol:OrgRegistry.pvm' \
DEPLOYER_SEED="$DEPLOYER_SEED" \
  node scripts/deploy-live.mjs
```

On success the script:

1. checks the deployer has a non-zero balance (fails fast otherwise),
2. submits `instantiateWithCode` and **waits for finalization** (not just
   inclusion — a live deploy must survive a re-org before we trust it),
3. reads `Revive.PristineCode[keccak256(blob)]` back and asserts it equals
   the locally-compiled blob, and
4. prints the deployed contract address as `DEPLOYED_H160=0x…`.

Record that H160 — it is the single contract instance every organisation
shares, and the value Stage 2 / the `on-chain-client` and the Phase 2 app
must be pointed at. (Determinism note: with no `SALT` the address is derived
from the deployer + nonce (CREATE1); pass `SALT=0x…32-bytes` for a
reproducible CREATE2 address.)

### Notes / gotchas

- **Endpoints move.** If a public RPC stops resolving, swap in another Asset
  Hub endpoint for the same chain (e.g. from the Polkadot.js apps endpoint
  list). The script only needs a healthy archive/full-node WS endpoint.
- **`exhaustsResources`?** Lower the `weightLimit` `refTime`/`proofSize` in
  `deploy-live.mjs`, or the storage-deposit limit — the defaults were tuned
  to Paseo AH's per-extrinsic limits and may differ slightly on Polkadot AH.
- **`pallet-revive` availability.** Paseo Asset Hub carries `pallet-revive`
  (the chopsticks fork forks its live runtime). Confirm Polkadot Asset Hub
  exposes `api.tx.revive.instantiateWithCode` before deploying to mainnet; if
  the extrinsic is missing, revive is not yet live there.
- **Secrets.** `DEPLOYER_SEED` must never be committed. `scripts/.env*` and
  `*.log` are already git-ignored; prefer exporting the seed in your shell
  over writing it to a file.

## Stage 1 gate (must all pass before Stage 2 starts)

- `forge test` passes (15 tests; 14 unit + 1 fuzz).
- `on-chain/abi/OrgRegistry.json` exists and matches the latest build.
- `scripts/package-lock.json` is checked in and chopsticks resolves to the
  locked version (no `@latest` at runtime).
- `scripts/chopsticks-sanity.sh` exits 0.
- Commit tagged `v0.1.0-on-chain-stage1`.

## What Stage 2 adds (not in this directory)

A sibling `on-chain-client/` Rust crate that reads contract state and
subscribes to events via smoldot. Tracked in its own plan.
