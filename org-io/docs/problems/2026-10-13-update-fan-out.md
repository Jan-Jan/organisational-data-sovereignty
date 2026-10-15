# Problems — a committed update reaches one Device

*Moved 2026-10-13 from the app's problems ledger
(`app/docs/problems/2026-10-07-update-fan-out.md`, which keeps a prose pointer
here), plan task T12 of `docs/plans/2026-10-06-org-io-transport.md`
(ruling A): the send moved to org-io, and the fan-out landed here. Its
`affects:` line named the app's one-send-per-command item, retired in this
task (superseded by LLR-znhmh2; see the S4 verification record); it names the
fan-out's items in its place.*

**PR-3ue4va**: the app sends each committed update to one Device only: an
admission to the joiner's Device (`app/src-tauri/src/invitation.rs`,
`admit_reply`) and a revocation to the revoked Member's first Device
(`app/src-tauri/src/commands.rs`, the revoke command); no other Member's
Device receives the update or its new Organisation private key, and a revoked
Member's other Devices receive no revocation.
affects: REQ-nfr3n2, REQ-6skn67, LLR-znhmh2
opened: 2026-10-06
status: resolved
resolution: fixed 2026-10-13 (change `worktree-org-io-transport`, plan task
T12): org-io's `submit_commit_send` names no recipient; after the commit it
sends every body the commit built — Organisation information to every Device
the committed record lists but the sender's, a revocation to every Device the
update removed (org-node's choice, made inside the commit) — from the sending
Persona's endpoint. In Networked mode every recipient is dialled by its
Device key and attempted whatever happened to the others, and the outcome is
reported per Device, the commit left in place when a send fails
(LLR-znhmh2, REQ-6skn67). The reproduction:
`networked_every_recipient_is_attempted_and_reported_in_order`
(`org-io/tests/send_fanout.rs`), three recipients over the in-process relay,
one never bound: red before (compile-red at the new API: the submission took
one recipient), green after, two delivered and one reported failed, in the
commit's order.

Recorded 2026-10-06 by the design of change `worktree-org-node-org-key-pair`.
The owner's model: a change in Organisation information goes to every Device
of every new and current Member, and a revocation, with its proof, goes only
to the revoked Devices, every Device of a revoked Member. With the key pair
rotated at every update, a Member that receives nothing stays on an earlier
epoch's key. By owner ruling (2026-10-06) the fan-out is the `org-io`
session's work, which moves peer communication into the app; org-node already
chooses the right kind for each recipient.

*Corrected 2026-10-13 (S4, task T12; the plan's finding F3):* "which moves
peer communication into the app" is stale: under the org-io roadmap peer
communication moved into org-io, not the app, and the fan-out is org-io's.

*Noted 2026-10-13 (S4, task T12):* in Loopback mode, which has no address
book, a submission still reaches one Device, the one whose Device key is the
identity of the address the operator gives (LLR-r9sh6k, REQ-8trkps); every
other recipient is reported not attempted. Resending to a Device that did not
receive its body is not part of this fix.
