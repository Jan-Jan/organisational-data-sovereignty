# The receiver holds the service lock while it waits for a peer

*Moved 2026-10-13 from the app's problems ledger
(`app/docs/problems/2026-10-06-receiver-holds-lock.md`,
which keeps a prose pointer here), plan task T11 of
`docs/plans/2026-10-06-org-io-transport.md` (ruling A): the receive loop
moved to org-io, and the fix landed here. The report had no `affects:`
line; it gains org-io's lock-free receive requirement and item, which it
was filed against.*

Found by the S4 survey (`docs/plans/2026-10-06-org-io-transport.md`) on
2026-10-06. Fixed in stage S4 of `docs/plans/2026-10-06-org-io-roadmap.md`,
when the receive loop moves to org-io.

**PR-7v8anh**: while the app's receiver runs, every other command that needs
the service waits until a peer connects, because the receive loop takes the
service lock and then waits for an incoming connection with the lock held.
affects: REQ-qkrj55, LLR-9arvqj
opened: 2026-10-06
status: resolved
resolution: fixed 2026-10-13 (change `worktree-org-io-transport`, plan task
T11): `OrgIo` holds org-node's service behind its own lock and every method
takes `&self`; `OrgIo::next_received` waits for the next body on the
receive channel's own lock, takes the node's only around `check_received`,
releases it for the chain read, and takes it again around `commit_checked`
and the store write (LLR-9arvqj, REQ-qkrj55). Reproduced and fixed by
`a_command_completes_while_the_receiver_waits`
(`org-io/tests/receive_step.rs`): a query and a Persona creation complete
within two seconds while `next_received` waits with no peer sending;
mutation-checked with the lock taken before the wait. The app's receiver
moves onto `next_received` on a shared `Arc<OrgIo>` in task T16.

Where: `next_outcomes` in `app/src-tauri/src/commands.rs` locks
`state.service` and, inside that lock, awaits
`receive_and_self_delete_if_revoked`, which waits in iroh's `accept()` until
a peer connects. The comment above it says the lock is held "only for the
duration of one receive+verify", but the wait for a connection is part of
that call.

*Where, re-read 2026-10-08 after S2 (`e41cb09`):* still present.
`next_outcomes` now locks `state.org_io` (the app's `Mutex<OrgIo>`) and,
inside that lock, awaits `OrgIo::receive_and_self_delete_if_revoked`, which
awaits org-node's `OrgService::receive_message` and so iroh's `accept()`.

Why it is a defect: with no peer sending, the lock is held indefinitely, so
publishing, revoking or reading state from the UI blocks while the receiver
is running.

Fix, in S4: org-io's receive loop waits for a connection without holding any
lock, and takes the service only to hand over the received bytes (the
values-in, values-out split ruled on 2026-10-06).
