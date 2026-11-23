# Problems found during change `worktree-networked-send-fanout`

Recorded 2026-11-20 from the merge gate of change
`worktree-networked-send-fanout`, which does not cause it: master's own code
measures the same.

**PR-7anw6h**: `make coverage-org-io` fails its region floor (82%): master `1ea5c64c` measures 80.61% of regions (752 of 3878 missed; lines 84.11%) on 2026-11-20, where its gate recorded 83.71% on 2026-11-12, with the same code and Cargo.lock; the host moved to macOS 27.2.1 on 2026-11-18. Which files lost coverage is not known: the earlier run's profile data is gone.
affects: none known; the coverage target of class C (crates/org-io/.guardrails/config.yaml coverage_command)
opened: 2026-11-20
status: open
