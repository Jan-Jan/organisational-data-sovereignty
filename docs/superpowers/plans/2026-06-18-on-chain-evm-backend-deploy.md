# on-chain EVM-backend Deploy Docs — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make revive's EVM (REVM) backend the recommended live-deploy path in `on-chain/README.md`, keeping the PolkaVM path as a labeled advanced alternative — docs only, no code changes.

**Architecture:** Single-file documentation edit. The "Compiling and deploying to a live chain" section is restructured into **EVM (recommended)** then **Advanced: PVM**; the intro paragraph and the `deploy-live.mjs` Layout entry are reworded. No changes to `src/`, `scripts/`, `foundry.toml`, `on-chain-client/`, or `org-node/`.

**Tech Stack:** Markdown (`on-chain/README.md`); verification via Foundry (`forge build`, `forge test`).

**Spec:** `docs/superpowers/specs/2026-06-18-on-chain-evm-backend-deploy-design.md`

---

## Context for the implementer

- `pallet-revive` is a **dual-VM** stack: REVM runs standard `solc` EVM bytecode; PolkaVM runs `resolc` output. The repo currently documents only the PVM path.
- The EVM deploy is plain Foundry: `forge build` → `forge create` over the chain's **eth-rpc HTTP** endpoint → `cast code` + `forge verify-contract`. No bespoke script. **No `--constructor-args`** (`OrgRegistry` has a default constructor).
- The deployer key is **secp256k1 / Ethereum (H160)**, not sr25519.
- Verified endpoints (do not alter): Paseo AH eth-rpc `https://eth-rpc-testnet.polkadot.io/` (chain ID `420420417`, token PAS, `--chain polkadot-testnet`); Polkadot AH eth-rpc `https://eth-rpc.polkadot.io/` (chain ID `420420419`, token DOT, `--chain polkadot`).
- The existing PVM content (resolc → `tmp/revive/*.pvm` → `deploy-live.mjs` → `Revive.PristineCode` verify, plus its notes) is **kept verbatim** under a new advanced heading — moved, not rewritten or deleted.
- Reads are backend-agnostic: `on-chain-client` reads via `ReviveApi::get_storage` using Solidity-layout slots, which `resolc` and `solc` assign identically (the existing chopsticks e2e already proves this against a PVM build). So this docs change cannot affect Rust reads — and there is **no view function** to `cast call` (the `orgs` mapping is `private`).

The current README "Compiling and deploying to a live chain" section spans roughly lines 48–158, beginning with `## Compiling and deploying to a live chain` and the paragraph `` `pallet-revive` runs PolkaVM bytecode, not EVM bytecode … `` and ending at the line before `## Stage 1 gate (must all pass before Stage 2 starts)`. The Layout entry to reword is the `scripts/deploy-live.mjs` bullet near the top.

---

## Task 1: Reword the Layout entry for `deploy-live.mjs`

**Files:**
- Modify: `on-chain/README.md` (the `scripts/deploy-live.mjs` bullet in the `## Layout` list)

- [ ] **Step 1: Read the file**

Run: `Read on-chain/README.md` (whole file) to confirm current text before editing.

- [ ] **Step 2: Replace the Layout bullet**

Find:
```markdown
- `scripts/deploy-live.mjs` — live-chain deploy + verify. Reads the signing
  key from `$DEPLOYER_SEED`, waits for finalization, and prints the deployed
  contract H160. Use this for Paseo / Polkadot Asset Hub.
```

Replace with:
```markdown
- `scripts/deploy-live.mjs` — **PVM-path** live-chain deploy + verify. Reads the
  signing key from `$DEPLOYER_SEED`, waits for finalization, and prints the
  deployed contract H160. Used by the advanced PolkaVM deploy path (the
  recommended EVM path uses `forge create` directly — no script).
```

- [ ] **Step 3: Commit**

```bash
git add on-chain/README.md
git commit --no-gpg-sign -m "docs(on-chain): relabel deploy-live.mjs as the PVM-path deploy script"
```

---

## Task 2: Restructure the deploy section — EVM primary, PVM advanced

**Files:**
- Modify: `on-chain/README.md` (the entire `## Compiling and deploying to a live chain` section, ~lines 48–158, up to but not including `## Stage 1 gate …`)

- [ ] **Step 1: Replace the intro paragraph**

Find the section header and its current intro:
```markdown
## Compiling and deploying to a live chain

`pallet-revive` runs PolkaVM bytecode, not EVM bytecode, so `forge build`'s
EVM artifact is **not** what gets deployed. The on-chain blob is produced by
`resolc` (the Revive Solidity compiler), and deployment goes through
`revive.instantiateWithCode`, not an EVM `CREATE` transaction.

Both Paseo Asset Hub (testnet) and Polkadot Asset Hub (mainnet) use the same
flow; only the RPC endpoint, the token used for fees/deposit, and how you
fund the deployer differ.
```

Replace with:
```markdown
## Compiling and deploying to a live chain

`pallet-revive` is a **dual-VM** contract platform: it runs both standard **EVM
bytecode** (via REVM, a Rust EVM) and **PolkaVM (PVM)** bytecode (via `resolc`).
Both share the same chain, RPC, and tooling. EVM is the simpler default — you
deploy the ordinary `forge build` artifact with stock Foundry over the chain's
Ethereum JSON-RPC ("eth-rpc") endpoint. PVM is for performance-critical /
RISC-V workloads and is documented as the advanced path below.

Both Paseo Asset Hub (testnet) and Polkadot Asset Hub (mainnet) expose an
eth-rpc endpoint; only the URL, chain ID, and the token used for fees differ.

### EVM path (recommended)

#### Prerequisites (once)

Foundry **nightly** — the nightly build adds the native `--chain polkadot*`
shortcuts and Blockscout verification:

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup --version nightly
forge --version
```

No `resolc` and no standalone `solc` are needed for this path; Foundry uses the
`solc 0.8.27` pinned in `foundry.toml`.

#### Step 1 — Fund an Ethereum-style deployer

EVM transactions are signed by a **secp256k1 / Ethereum (H160)** key, not the
sr25519 key the PVM script uses. Generate or reuse an Ethereum keypair (e.g.
`cast wallet new`) and fund its address:

- **Paseo Asset Hub (testnet):** free PAS from <https://faucet.polkadot.io/>
  (select **Polkadot Hub TestNet**). PAS has no real-world value.
- **Polkadot Asset Hub (mainnet):** real DOT — enough for fees plus the
  existential deposit — and **double-check the address** before signing.

Because the account is already an Ethereum keypair, no account-mapping is
required. (A native sr25519/ed25519 account would need [account
mapping](https://docs.polkadot.com/smart-contracts/for-eth-devs/accounts/#account-mapping-for-native-polkadot-accounts);
an Ethereum key avoids it.)

Export the key (never commit it):

```bash
export PRIVATE_KEY=0x…            # 32-byte hex secp256k1 secret
```

#### Step 2 — Compile

```bash
cd on-chain
forge build
```

This is the same EVM artifact `forge test` exercises.

#### Step 3 — Deploy

`OrgRegistry` has no constructor arguments, so none are passed.

**Paseo Asset Hub (testnet)** — eth-rpc `https://eth-rpc-testnet.polkadot.io/`,
chain ID `420420417`:

```bash
cd on-chain
forge create src/OrgRegistry.sol:OrgRegistry \
  --rpc-url https://eth-rpc-testnet.polkadot.io/ \
  --private-key "$PRIVATE_KEY" \
  --broadcast
```

**Polkadot Asset Hub (mainnet)** — eth-rpc `https://eth-rpc.polkadot.io/`,
chain ID `420420419`:

```bash
cd on-chain
forge create src/OrgRegistry.sol:OrgRegistry \
  --rpc-url https://eth-rpc.polkadot.io/ \
  --private-key "$PRIVATE_KEY" \
  --broadcast
```

With Foundry nightly you can replace `--rpc-url …` with `--chain
polkadot-testnet` (or `--chain polkadot`). On success `forge create` prints
`Deployed to: 0x…`.

#### Step 4 — Confirm and verify

```bash
# Code is present at the address (non-empty bytecode):
cast code 0x… --rpc-url https://eth-rpc-testnet.polkadot.io/

# Source-verify on the block explorer (Blockscout, no API key):
forge verify-contract 0x… src/OrgRegistry.sol:OrgRegistry --chain polkadot-testnet
```

There is no view function to `cast call` (the `orgs` mapping is `private`); the
functional state read is exercised by `on-chain-client` and the Phase 2
preflight (`check_contract` → `get_org_state`) once pointed at the new H160.

#### Record the address

Record the deployed `0x…` H160 — it is the single contract instance every
organisation shares, and the value Stage 2 / the `on-chain-client` and the
Phase 2 app must be pointed at. (EVM `CREATE` derives the address from the
deployer account + nonce.)

#### Notes / gotchas

- **Endpoints move.** If an eth-rpc URL stops resolving, swap in another from
  the [Connect to Polkadot](https://docs.polkadot.com/smart-contracts/connect/)
  endpoint list for the same chain.
- **PAS vs DOT.** Paseo PAS has no value; Polkadot DOT is real — re-check the
  address before a mainnet deploy.
- **Existential deposit.** The deployer needs a small balance buffer beyond
  fees so its account isn't reaped.
- **Finalization.** `forge create` waits for the transaction receipt
  (inclusion). For mainnet, re-check the address persists across a few blocks
  before trusting it.
- **Secrets.** `PRIVATE_KEY` must never be committed; prefer exporting it in
  your shell over writing it to a file.

### Advanced: deploy as a PolkaVM (PVM) contract

Choose this path for performance-critical / RISC-V workloads. It compiles
Solidity with `resolc` to a PolkaVM blob and deploys via the
`revive.instantiateWithCode` extrinsic (not an EVM transaction), verifying the
on-chain `Revive.PristineCode` hash.
```

- [ ] **Step 2: Demote the existing PVM content to subheadings under the advanced heading**

The current PVM content — `### Prerequisites (once)`, `### Step 1 — Compile to a PolkaVM blob`, `### Step 2 — Fund a deployer account`, `### Step 3 — Deploy and verify`, and `### Notes / gotchas` — is **kept verbatim** but must now sit *below* the new `### Advanced: deploy as a PolkaVM (PVM) contract` heading and not collide with the EVM path's headings. Demote each of those five `###` headings to `####` (so they nest under the advanced `###` section), leaving their body text unchanged. The advanced section ends where `## Stage 1 gate (must all pass before Stage 2 starts)` begins — do not modify the Stage 1 gate section or anything after it.

- [ ] **Step 3: Verify no stale claims remain**

Run:
```bash
cd on-chain
grep -nE 'runs PolkaVM bytecode, not EVM|not \*\*what gets deployed' README.md || echo "OK: no stale PVM-only intro claim"
grep -nc 'resolc' README.md   # resolc mentions should now all sit within the Advanced (PVM) section
```
Expected: the first grep prints `OK: …` (the outdated framing is gone); `resolc` still appears (in the PVM section) — manually confirm every occurrence is under `### Advanced: deploy as a PolkaVM (PVM) contract`.

- [ ] **Step 4: Verify the build/tests still pass (artifact the new path deploys)**

Run:
```bash
cd on-chain && forge build && forge test
```
Expected: build succeeds; `forge test` passes (15 tests: 14 unit + 1 fuzz). This is unchanged by the docs edit — it confirms the EVM artifact the recommended path deploys still compiles and behaves.

- [ ] **Step 5: Doc self-review**

Read the rewritten section end-to-end. Confirm: EVM path appears before PVM; endpoints/chain IDs match the Context block above; the `account-mapping` and `Connect to Polkadot` links are well-formed; no duplicated `### Step 1` headings across the two paths (EVM uses `####`, PVM uses `####` under its own `###`); the Quickstart and Stage-1 gate sections are untouched.

- [ ] **Step 6: Commit**

```bash
git add on-chain/README.md
git commit --no-gpg-sign -m "docs(on-chain): make revive EVM (REVM) backend the recommended deploy path"
```

---

## Self-review (plan vs spec)

- **Spec coverage:** §4.1 intro reframe → Task 2 Step 1; §4.2 EVM path (prereqs/fund/compile/deploy/confirm/record/gotchas) → Task 2 Step 1; §4.3 PVM relabel-and-demote → Task 2 Step 2; §4 Layout relabel → Task 1; §6 verification (`forge build`/`forge test` + self-review + endpoint cross-check) → Task 2 Steps 3–5. All spec sections map to a task.
- **No code tasks:** consistent with the spec's "docs only" scope — no `src/`, `scripts/`, `foundry.toml`, or Rust edits.
- **No placeholders:** the `0x…` and `0x…`-key tokens are deliberate operator-supplied values in command templates, not plan gaps.
- **Consistency:** endpoint URLs, chain IDs, the no-constructor-args fact, the secp256k1 key requirement, and the no-view-function confirm step are identical across the spec and both tasks.
