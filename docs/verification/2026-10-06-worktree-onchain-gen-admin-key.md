# Verification — onchain-gen-admin-key (2026-10-06)

branch: worktree-onchain-gen-admin-key
reviewer: independent subagent (no implementation narrative), 2026-10-06
verdict: approve — the script is correct and reproduces published values; one low comment inaccuracy, fixed
reproduced: yes — `node gen-admin-key.mjs "bottom drive obey lake curtain smoke basket hold race lonely fit walk"` printed seed 0xfac7959dbfe72f052e5a0c3c8d6530f202b02fd8f9f5ca3580ec8deb7797479e and public key 0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a, subkey's published values for that phrase; run by the author and again by the reviewer

Change: commits `on-chain/scripts/gen-admin-key.mjs`, which had sat untracked in
the primary checkout since 2026-06-18. Branched from `master` at `5f7c177`.
Plan: none — one tooling file outside every compliance unit.

## The gate

Measured on: `0d4f293` — tree `55ceca083fa93c39f11e5ba55b5896d5505d514b`, clean
worktree. `on-chain` is `not_a_unit` in `.guardrails/units.yaml`, so
`check-units.sh --impact master..HEAD` prints no unit and no unit's
`verify_commands` applies. The ID and trace gates were run for every unit
anyway, as a check that the change disturbs none.

| Gate | Result |
| --- | --- |
| `check-units.sh --impact master..HEAD` | exit 0, empty impact set |
| `check-units.sh` | exit 0 (units 5, disclaimed 6) |
| `check-ids.sh`, each of the five units | exit 0 |
| `check-trace.sh`, each of the five units | exit 0; this change adds and removes no ID |
| Coverage, against the class target | not applicable — no unit touched |
| Working tree | clean |

## Red → green

| Item | Test | Watched red |
| --- | --- | --- |
| none | the change implements no ID | — |

## What was wrong, and what was built

The app's chain mode needs `ODS_ADMIN_SEED`, a 32-byte sr25519 mini-secret
(`app/src-tauri/src/state.rs`, passed to
`subxt_signer::sr25519::Keypair::from_secret_key`). The script that derives one,
from a fresh or a given BIP39 phrase, existed only as an untracked file. It is
now committed unchanged apart from finding-1's comment.

## Review

**finding-1**: code, low — the comment called the seed the input of
`subxt_signer::sr25519::Keypair::from_seed`, while `state.rs` calls
`Keypair::from_secret_key`.
disposition: comment corrected to `from_secret_key` (`0d4f293`). subxt-signer
0.50.3 `src/sr25519.rs:123` has no `from_seed`; `from_secret_key` expands the
bytes as a `MiniSecretKey` with `ExpansionMode::Ed25519`, which is what
`sr25519PairFromSeed` does, so the values above hold for the app too.

## Gaps

- No automated test runs the script; `on-chain` is outside compliance.
- The script prints the seed and, when generated, the phrase to stdout, and a
  phrase passed as an argument lands in shell history. The header limits it to
  dedicated testnet keys; nothing enforces that.
