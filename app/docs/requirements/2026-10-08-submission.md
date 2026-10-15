# app — deciding a submission and handing it to org-io (stage S2)

Change `worktree-org-io-create`, stage S2 of
`docs/plans/2026-10-06-org-io-roadmap.md`; plan
`docs/plans/2026-10-06-org-io-create.md`, task T9. Written 2026-10-08, when
the app switched to org-io.

The app's former requirement that it submit org-node's provisional update
to the chain and only then ask org-node to commit and send moved to org-io
with the code (ruling A); org-io exports it as REQ-nfr3n2. What stays in the
app is the decision: two of this unit's low-level requirements describe it,
the choice of the one Device a committed update is sent to and the admission
of an Invite reply, and an app low-level requirement does not decompose
another unit's requirement. This is their parent. Minted 2026-10-07 (plan §6,
"Amended in place", app).

**REQ-m8sgjk**: The software shall decide when an Organisation is
founded and when a provisional update is submitted, and which Device
receives the committed update, and shall hand each to org-io and report
the outcome org-io returns.
satisfies: derived

The app decides when from the operator's commands (create an Organisation,
admit a reply, revoke a Member); org-io writes the chain, reads back the
state the write left, has org-node commit and send, and returns the outcome
or the failure, which the command reports as it did before the move.

*Narrowed 2026-10-14 (S4, task T15 of
`docs/plans/2026-10-06-org-io-transport.md`; hybrid rule: a narrowing, the
ID is kept).* "Which Device receives the committed update" is withdrawn:
org-io sends every committed update to every recipient the commit names
(its fan-out, S4 task T12), and the item that chose one Device was retired
there. For an admission the app passes the Invite reply to org-io's
exchange (`admit_reply`), which checks it, has org-node build and commit
the admission, and sends and settles it (the admission item moved to org-io
in S4 task T13). What the app still does: it decides when, from the
operator's commands, hands each to org-io, and shows the result org-io
returns — the new Member, the refusal, or a committed update whose send
failed, which the command reports as a failure that names the commit. The
derivation note above ("two of this unit's low-level requirements describe
it") is historical: both have left the app, and the requirement is verified
at the IPC boundary by
`admit_member_hands_the_reply_to_org_io_and_reports_a_committed_admission_whose_send_failed`
(ipc).
