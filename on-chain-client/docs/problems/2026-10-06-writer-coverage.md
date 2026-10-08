# Problem reports — on-chain-client, the chain writer is not in the coverage run

**PR-b795an**: `make coverage-on-chain-client`, this unit's `coverage_command`,
builds without the `write` feature and runs none of `write_manifest`,
`write_pure`, `write_compose` or `write_events`, so the chain writer (`on-chain-client/src/write/`)
is never measured, and the unit's figures (about 42% of lines and regions,
decisions unmeasured, on 2026-10-06) describe the reader alone.
affects: SDD-yg7n55, REQ-6jefu2, REQ-aat4yt
opened: 2026-10-06
status: resolved
resolution: `ON_CHAIN_CLIENT_COVERAGE_ARGS` in the Makefile, read by both
`make coverage-on-chain-client` and `make coverage-branch-on-chain-client`,
builds with `--features test-support,write` and runs the six writer targets
of the unit's second verify_commands line, so `src/write/` is measured.
Resolved by change `worktree-org-io-create` (T4). Reproduced by the coverage
summary: before the fix it listed no file under `src/write/`, after it every
one.

Found 2026-10-06 by the verification gate of change
`worktree-org-node-chain-authority`, which moved the chain writer into this unit
behind the `write` feature. The owner accepted the unit's coverage shortfall
against class C for that merge (statement coverage well below full, decision
coverage not measured, the writer not measured), on the grounds that the same
shortfall stands on master, that org-node and the app have no coverage command
at all, and that coverage is the next ratchet tooth. Adding the `write` feature
and its four test targets to the coverage target is this report's fix.

Resolved 2026-10-07 by change `worktree-org-io-create` (plan T4). Before: 41.75%
lines (205 of 491) / 42.60% regions, 34 of 34 branches, no file under
`src/write/` in the summary. After: 47.66% lines (428 of 898) / 48.53% regions
(658 of 1356), with calldata.rs and ceremony.rs at 100.00%, events.rs 97.30%,
proxy.rs 96.55%, multisig.rs 90.70%, signatory_set.rs 88.24%, mod.rs 40.00% and
subxt_ops.rs, the chain-only shell, 0.00% over 116 lines. The line and region
floors rose from 41 / 42 to 46 / 47. Decision coverage first fell to 56 of 58
branches (96.55%): the writer brought two unreached outcomes into the count,
`is_proxied`'s non-variant arm (write/proxy.rs:41) and one closure
instantiation of `observed_event`'s `event == "ProxyExecuted"` operand
(write/events.rs:15). The branch floor of 99 was not lowered; both outcomes
were covered by tests alone, with no production change (a non-variant call
passed to `is_proxied`, and one shared closure type for every
`observed_event` call in tests/write_events.rs). Final measurement: 58 of 58
branches, 100% (floor 99%), and 47.88% lines (430 of 898) / 48.75% regions
(661 of 1356); `make coverage-on-chain-client` and
`make coverage-branch-on-chain-client` both exit 0.
