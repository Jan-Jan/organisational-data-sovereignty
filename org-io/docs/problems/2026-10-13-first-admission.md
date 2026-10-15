# Problem reports — the app never commits its own first admission

*Moved 2026-10-13 from the app's problems ledger
(`app/docs/problems/2026-10-06-first-admission.md`, which keeps a prose
pointer here), plan task T11 of `docs/plans/2026-10-06-org-io-transport.md`
(ruling A): the receive loop moved to org-io, and the fix landed here. Its
`affects:` line named the app's Invite-reply requirement (which declares
the expected first admission to org-node), named here in prose; it now
names org-io's one-receive-path requirement and item besides the
submission requirement it named.*

**PR-kwwap5**: the app's receiver loop calls only org-node's
`receive_and_self_delete_if_revoked` (`app/src-tauri/src/commands.rs`,
`next_outcomes`), which refuses an Envelope for an Organisation the node holds
no record of, so an invitee's app never commits its own first admission and
never joins the Organisation it was admitted to.
affects: REQ-fnsjb6, LLR-cvyyr8, REQ-nfr3n2
opened: 2026-10-06
status: resolved
resolution: fixed 2026-10-13 (change `worktree-org-io-transport`, plan
tasks T4 and T11): org-node has one receive path (`check_received`, then
`commit_checked`; the self-delete entry point retired in T4), and org-io's
one receive step, `OrgIo::next_received`, passes every body through it
(LLR-cvyyr8, REQ-fnsjb6). Reproduced and fixed by
`a_declared_first_admission_is_committed_by_the_one_path`
(`org-io/tests/receive_step.rs`): an invitee that declared the expected
first admission commits it as `ReceiveKind::Admitted` and holds the
Organisation. The app's receiver moves onto `next_received` in task T16.

Found 2026-10-05 while planning change `worktree-org-node-chain-authority`. The
behaviour predates that change. It matters now because the owner's invitation
journey (an invitee enters an Invite, replies, and is admitted) ends in exactly
this first admission, and the expectation the app's Invite-reply requirement
declares to org-node is never consulted by the shipped app. By owner ruling
(2026-10-06) it is fixed in the transport change that follows, which moves
the receive loop into the app and rewrites it. (*Re-targeted 2026-10-06:* the
receive loop moves to org-io instead, stage S4 of
`docs/plans/2026-10-06-org-io-roadmap.md`.)
