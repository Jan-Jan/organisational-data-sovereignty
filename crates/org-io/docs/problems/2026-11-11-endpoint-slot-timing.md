# Problems found during change `worktree-cargo-test-failures`

Recorded 2026-11-11 from the merge gate of change
`worktree-cargo-test-failures`, which does not touch the code concerned.

**PR-v7zcbh**: org-io's test `a_seventeenth_connection_waits_for_a_slot_while_sixteen_stall` (`crates/org-io/tests/endpoints.rs:540`) fails intermittently: under `make coverage-branch-org-io` the seventeenth peer's body was taken 1.16 s after sixteen connections stalled, before the 1.8 s the test requires for a slot to free under a 2 s connection time limit (1 failure in 4 runs of the target in one gate).
affects: LLR-8uf9a5
opened: 2026-11-11
status: open
