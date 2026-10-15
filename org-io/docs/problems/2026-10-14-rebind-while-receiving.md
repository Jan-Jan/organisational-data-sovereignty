# A rebound endpoint does not accept while the receiver runs

Opened 2026-10-14 by task T21 of `docs/plans/2026-10-06-org-io-transport.md`
(found by T20, which fixed the rebind on `start_receiving` and on a send).

**PR-47aycj**: when one Persona's endpoint closes while another Persona's
endpoint is still accepting, the receive loop goes on, so nothing calls
`OrgIo::start_receiving` again; a send from the first Persona then binds its
endpoint again but starts no accept task on it, so a message to that
Persona's Device never reaches the receive channel until the next
`start_receiving`.
affects: LLR-z35vbh, LLR-d2pzzk, SDD-xwft92
opened: 2026-10-14
status: resolved
resolution: fixed 2026-10-14 (change `worktree-org-io-transport`, plan task
T21): root cause, only `start_accepting` started accept tasks, and nothing
recorded that the node was receiving. Now `Endpoints` remembers it from the
first `start_receiving`, and an endpoint bound later by a send gets its
accept task under the map's lock, the running-task count kept exact
(LLR-z35vbh as amended that day). Reproduced and fixed under
`a_send_that_rebinds_a_closed_endpoint_while_the_receiver_runs_accepts_on_it`
and `a_refused_rebind_while_the_receiver_runs_binds_and_accepts_on_the_next_send`
(`org-io/tests/endpoints.rs`).

## What was observed

T20's fix drops a closed endpoint and binds it again on its Persona's next
send or `start_receiving`. Only `start_receiving` (`Endpoints::start_accepting`)
starts accept tasks. A send binds through `Endpoints::endpoint`, which starts
none. While another Persona's endpoint keeps accepting, `next_inbound` never
returns the terminal `EndpointClosed`, the app's receive loop does not stop,
and nothing restarts it: the rebound endpoint has no accept task.

## Why it matters

LLR-z35vbh: once the node receives, every Persona's endpoint has one running
accept task. Inbound updates and revocations addressed to the rebound
Persona's Device are not received while the loop runs, and no stop is
announced to the user.
