# Problem reports — on-chain-client, the chain writer is not in the coverage run

**PR-b795an**: `make coverage-on-chain-client`, this unit's `coverage_command`,
builds without the `write` feature and runs none of `write_manifest`,
`write_pure`, `write_compose` or `write_events`, so the chain writer (`on-chain-client/src/write/`)
is never measured, and the unit's figures (about 42% of lines and regions,
decisions unmeasured, on 2026-10-06) describe the reader alone.
affects: SDD-yg7n55, REQ-6jefu2, REQ-aat4yt
opened: 2026-10-06
status: open

Found 2026-10-06 by the verification gate of change
`worktree-org-node-chain-authority`, which moved the chain writer into this unit
behind the `write` feature. The owner accepted the unit's coverage shortfall
against class C for that merge (statement coverage well below full, decision
coverage not measured, the writer not measured), on the grounds that the same
shortfall stands on master, that org-node and the app have no coverage command
at all, and that coverage is the next ratchet tooth. Adding the `write` feature
and its four test targets to the coverage target is this report's fix.
