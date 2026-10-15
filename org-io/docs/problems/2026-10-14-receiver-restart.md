# A closed endpoint is kept, so the receiver cannot be restarted

Opened 2026-10-14 by task T19 of `docs/plans/2026-10-06-org-io-transport.md`
(the owner's ruling 2 of that day: lower the app's dead-receiver hazard only
after a test shows the real receive loop stops and restarts).

**PR-tcd2ta**: once a Persona's endpoint has closed, `OrgIo::start_receiving`
neither binds a new endpoint for that Persona nor starts a new accept task on
it, so every later `next_received` returns `ReceiveError::Terminal(TerminalCause::EndpointClosed)`
at once and the node receives nothing more for the life of the handle.
affects: LLR-z35vbh, LLR-d2pzzk, SDD-xwft92
opened: 2026-10-14
status: resolved
resolution: fixed 2026-10-14 (change `worktree-org-io-transport`, plan task
T20): root cause, `Endpoints` kept a closed endpoint and gave each endpoint
one accept task, once. Now an endpoint that reports closed is dropped and
bound again from its keychain item on its Persona's next send or
`start_receiving`, and `start_receiving` gives a new accept task to an
endpoint whose task has ended (LLR-d2pzzk, LLR-z35vbh as amended that day).
Reproduced and fixed under
`after_its_endpoint_closes_start_receiving_binds_it_again_and_receives`,
`after_its_endpoint_closes_a_send_binds_it_again` and
`a_refused_rebind_is_a_failed_bind_and_the_next_start_binds`
(`org-io/tests/endpoints.rs`); the app's
`after_a_stop_start_receiver_starts_a_loop_that_runs` runs un-ignored and
passes.

## What was observed

*Corrected 2026-10-14 (S4 review round 1, finding 15):* the test named here
was split before T19 merged; the real tests are
`a_closed_endpoint_stops_the_loop_announces_it_once_and_releases_the_guard`
(the first point below) and `after_a_stop_start_receiver_starts_a_loop_that_runs`
(the second and third), both in `app/src-tauri/tests/receiver_guard.rs`.

The app's receiver tests (`app/src-tauri/tests/receiver_guard.rs`, named
above) run the app's real receiver loop
over org-io's receive step on a bound Loopback endpoint, then closes the
endpoint:

- the loop stops, announces `receiver-stopped` once, naming "endpoint
  closed", and releases its start guard (as the app's controls require);
- `start_receiver` claims the guard again and spawns a new loop;
- that loop stops at once, announcing a second `receiver-stopped`: the
  assertion "the restarted loop runs: it announces no stop" fails.

## Cause

`Endpoints` (`org-io/src/endpoints.rs`) keeps each bound endpoint for the
life of the handle (`bound_or_bind` binds only a Device key it does not hold)
and gives it one accept task, once (`start_accepting` skips an entry whose
`accept_task` is `Some`, even when that task has ended). When the endpoint
closes, its accept task counts itself out of `accepting` and ends; from then
on `next_inbound` finds `accepting == 0` and returns `EndpointClosed`, and
nothing rebinds or re-accepts. A send from that Persona also reuses the closed
endpoint.

## Why it matters

The app's dead-receiver hazard (its risk file, §4.6) is reduced only when the
stop is announced *and a restart works*. With this defect a restart claims the
guard and announces a second stop, so the state is still absorbing for the
life of the process: inbound updates and revocations addressed to this node
are never received again until the app is restarted. The app's hazard
estimate is therefore not lowered by T19.

## Fix direction (not made by T19)

Drop a closed endpoint from the map when its accept task ends (or when
`start_accepting` finds the task finished), so the next `start_receiving`
binds it again from the keychain item and starts a new accept task. Whether a
send that finds its endpoint closed should rebind too is part of the same
fix.
