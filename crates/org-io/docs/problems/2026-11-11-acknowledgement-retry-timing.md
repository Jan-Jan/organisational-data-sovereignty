# Problems found during change `worktree-cargo-test-failures`

Recorded 2026-11-11 from the merge gate of change
`worktree-cargo-test-failures`, which does not touch the code concerned.

**PR-g2ec66**: org-io's test `a_failed_step_is_reported_and_retried_hourly_and_at_open` (`crates/org-io/tests/acknowledgements.rs`) fails intermittently under `make coverage-branch-org-io`: its receive step (`tests/common/journey.rs:221`, `next`) waits past the 30 s `NET` timeout and panics `Elapsed(())`, in a run of the `acknowledgements` target that took 672 s; the same test passed in the uninstrumented run of the same gate.
affects: LLR-3fxxpp
opened: 2026-11-11
status: open
