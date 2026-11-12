# Problems found during change `worktree-cargo-test-failures`

Recorded 2026-11-11 while running org-io's `preflight` target by hand
(`--features chopstix-tests`) after a fresh install of contracts/scripts.

**PR-msj9ks**: org-io's `preflight` test fails intermittently at `preflight.rs:56` ("live chain.live should pass with a miner"): with a continuous miner, `check_chain_live` over its 12 s gap sometimes sees no new finalised block on the chopsticks fork (1 failure in 5 runs, the run right after a fresh install).
affects: SDD-z85ux9
opened: 2026-11-11
status: open
