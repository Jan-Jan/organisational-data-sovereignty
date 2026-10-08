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
LLR-ctrfz4, LLR-gha5f6, LLR-w4mhd4, and a note on LLR-f35pda) are in the files
that define them.

## Under SDD-rmbr3t — The command surface

**LLR-q225ws**: each command that submits an admission or a revocation asks
org-node, once the update has committed, for exactly one `send_update`, to
exactly one DevicePublicKey, and passes it no Organisation secret, key or
invite identifier. `admit_reply` (`admit_member`) sends to the DevicePublicKey
the Invite reply carries, which the committed record lists, so org-node sends
that Device Organisation information. `revoke_member` sends to the first
DevicePublicKey (`device_keys.first()`) of the revoked member's snapshot in
the record as it stood before the revocation, which the committed record no
longer lists, so org-node sends that Device a revocation. Neither command
sends the committed update to any other Device, whether of another Member
or another Device of the same Member.
satisfies: REQ-m8sgjk

*Amended 2026-10-07 (S3): `send_update` takes the `CommitOutcome`; a revoked Device is sent its notice, and a Device neither record lists nothing.*

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).* `satisfies:`
was REQ-nfr3n2, which moved to org-io: an app low-level requirement does not
decompose another unit's requirement, so it is re-parented to the app's
REQ-m8sgjk, the decision of which Device receives the committed update. The
command now hands its one recipient to org-io's `OrgIo::submit_commit_send`,
which asks org-node for that one `send_update` after the commit; the
recipient each command chooses, and that there is exactly one, are
unchanged. Classification under the hybrid rule: a clarification, not a
change of meaning; its two tests stay in `app/src-tauri/tests/submit_flow.rs`
with their annotations unchanged.

This states the app's behaviour as it is, not as the ruling would have it
broadly: the other Members' Devices, which the ruling says receive
Organisation information whatever the change, are sent nothing by the app
today, and a revoked Member's second and later Devices are sent nothing.
That defect is PR-3ue4va: in the owner's model a change in Organisation
information goes to every Device of every new and current Member, and a
revocation to every Device of a revoked Member; by owner ruling the fan-out is
the `org-io` session's work, not this change's.
