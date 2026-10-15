# A Persona created while the node receives is not received on

Opened 2026-10-14 by the fix of finding 2 of the S4 review, round 1
(`docs/plans/2026-10-06-org-io-transport.md`, merge-change step 6a).

**PR-y6t5kt**: a Persona created while the receive loop runs gets no
endpoint and no accept task: `OrgIo::create_persona` binds nothing and does
not tell `Endpoints` to accept for the new Device key, the app's
`start_receiver` returns early while its guard is held, and `produce_reply`
binds nothing either, so an admission sent to the new Persona's Device is
never accepted until the receive loop is started again.
affects: LLR-z35vbh, LLR-9cb443, SDD-xwft92
opened: 2026-10-14
status: resolved
resolution: fixed 2026-10-14 (change `worktree-org-io-transport`, review
round 1, finding 2): root cause, only `start_receiving` and a send's bind
started accept tasks, and creating a Persona binds nothing. Now, once the
node receives, `create_persona` binds the new Persona's endpoint and gives it
an accept task, and `produce_reply` does the same first, refusing with
`ExchangeError::Bind` when the bind fails (LLR-z35vbh as amended that day).
Reproduced and fixed under
`a_persona_created_while_the_receiver_runs_is_received_on` and
`no_reply_is_produced_while_receiving_for_a_persona_whose_endpoint_cannot_be_bound`
(`org-io/tests/endpoints.rs`).

## What was observed

The reviewer read it from the code (review round 1, finding 2): T21's
receiving flag gives an accept task only to an endpoint a send binds
(`Endpoints::endpoint`); a Persona that has never sent, created after
`start_receiving`, has no endpoint at all.

## Why it matters

LLR-z35vbh: once the node receives, a message to any Persona's Device reaches
the one receive path. The invitation exchange depends on it: the new
Persona's reply is produced and handed over out of band, and the admitting
node then sends the admission to that Persona's Device while the joining
node's receive loop is already running.
