# app — whom a committed update is sent to

Low-level requirement for change `worktree-org-node-org-key-pair`, written
2026-10-06. It refines REQ-nfr3n2 (`app/docs/requirements/2026-10-06-invitation.md`)
*(restated 2026-10-08, change `worktree-org-io-create`, task T9: that
requirement moved to org-io, which exports it; this file's item now refines
the app's REQ-m8sgjk, `app/docs/requirements/2026-10-08-submission.md`,
the decision of which Device receives the committed update)*
for the owner ruling of that change that the kind of Wire message org-node
sends depends on the recipient, not on the operation: a Device the committed
Membership record lists receives Organisation information, which carries the
Organisation private key; any other Device receives a revocation, which
carries none (the owner rulings in
`org-node/docs/requirements/2026-10-07-org-key-pair.md`).
The app does not choose the kind: it chooses only the recipient, and this
item states which recipient each command chooses. No software item is added.
Safety class C, no per-item override, as in `2026-10-05-decomposition.md`.

The amendments this change makes to existing app items (LLR-8krgzj,
LLR-ctrfz4 — org-io's from 2026-10-13, back in this unit since
2026-10-15 —, the reply's
check and admission, `produce_reply`, and a note on the outstanding pairs —
those three org-io's since 2026-10-14, S4 task T13) are in the files that
define them.

## Under SDD-rmbr3t — The command surface

*Retired 2026-10-13 (S4, task T12 of `docs/plans/2026-10-06-org-io-transport.md`;
the owner's retirement ruling of that day, task T6 step 6):* the item this
section defined — each command that submits an admission or a revocation
asks for exactly one send, to exactly one Device (the reply's Device for an
admission; the revoked Member's first Device for a revocation), and no
other Device is sent the committed update — is retired, superseded by
org-io's fan-out: org-io's `submit_commit_send` names no recipient and sends
every body the commit builds, to every Device the committed record lists
but the sender's and a revocation to every Device the update removed
(`org-io/docs/architecture/2026-10-07-transport.md`,
"Sending, and the peer address"). Its definition is deleted; its ID is
listed in the S4 verification record. The defect it stated, the app's
report on the update fan-out, moved to org-io
(`org-io/docs/problems/2026-10-13-update-fan-out.md`)
and is resolved there. Its two tests moved to org-io's
`tests/submit_flow.rs`, rewritten onto org-io's handle, with no `verifies:`
line. History, as it read before: it was re-parented on 2026-10-08 from
the submission requirement org-io took to the app's REQ-m8sgjk, and amended
on 2026-10-07 for S3's revocation notices.
