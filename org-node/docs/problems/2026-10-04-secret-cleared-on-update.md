# Problem reports — org-node, an update overwrites the Organisation secret

Opened by the architecture tooth
(`docs/plans/2026-10-03-org-node-architecture.md`) from review round 5, which
found the overwrite stated by no low-level requirement and evidenced by
nothing.

**PR-xwek5e**: when `receive_and_verify` commits into an existing Organisation
record, it overwrites the stored Organisation secret with whatever the Wire
message carries. `revoke_member` always sends `org_secret: None`. So a member
that receives another member's revocation on this path loses the Organisation
secret while remaining a member.
affects: SDD-8cpyfa, SDD-72ddm6, LLR-ckk5nz, LLR-8hdu9x
opened: 2026-10-04
status: open

## What was observed

`existing.org_secret = msg.org_secret` (`org-node/src/service.rs:1075`), and
`WireMessage { envelope, org_secret: None, … }` in `revoke_member`
(`:1208`).

`pr_xwek5e_another_members_revocation_clears_the_receivers_secret`
(`org-node/tests/admission_sender.rs`) admits B, admits C with the change
pushed to B, then revokes C with the change pushed to B. B commits the removal
and its record's secret is `None`. The test asserts that, and correcting the
behaviour reddens it.

The self-delete path's update branch does not touch the secret at all. The
shipped app receives only on that path (`app/src-tauri/src/commands.rs:339`),
so this is reachable through the library and not through the shipped app's
receiver.

## Why it is a problem report and not yet a requirement

Which behaviour is right is **not ruled**, and this report does not rule it.
Three readings are possible:

- keep the stored secret unless a new one arrives;
- clear it, on the ground that a revocation invalidates the secret until it is
  rotated;
- rotate it.

`org-node/quint/protocol.qnt` carries a `revokedExcludedFromOrgSecret`
invariant, which points at rotation. The code implements none of the three on
purpose. It writes through whatever the Wire message carries, and a revocation
happens to carry `None`. The ruling belongs to the owner. The fix and the
requirement follow from it, in a change of their own.

*Annotated 2026-10-05 by review round 8.* The sending side is LLR-8hdu9x:
`revoke_member`'s Wire message carries no secret. Whichever reading is ruled,
the fix is on one side or the other, so both requirements are named.
