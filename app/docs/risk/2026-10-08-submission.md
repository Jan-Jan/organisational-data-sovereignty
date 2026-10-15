# app — risk assessment of the submission decision (stage S2)

Change `worktree-org-io-create`, task T9 of
`docs/plans/2026-10-06-org-io-create.md`. Assesses the derived requirement
of `app/docs/requirements/2026-10-08-submission.md`.

**REQ-m8sgjk** (decide when to found or submit and which Device receives the
committed update; hand each to org-io; report what it returns): no hazard
impact in S2. These are the same decisions the app made before, made at the
same commands, now handed to org-io instead of carried out by the app's own
chain writer. The recipient rule is unchanged: one send, to one
DevicePublicKey, chosen as the app chose it, and the fan-out of a committed
update to the other Members' Devices remains the open problem on the update
fan-out (`app/docs/problems/2026-10-07-update-fan-out.md`), unchanged by this
change. *(2026-10-13, S4 task T12: that report moved to org-io and is
resolved there by the fan-out, assessed in org-io's risk ledger under its
fan-out control; the app no longer chooses a recipient.)* The submission's own order and failure rule, and the custody of the
signatory key that signs it, moved to org-io and are assessed in org-io's
risk ledger with the requirement that states them.

*Re-assessed 2026-10-14 (S4, task T15, REQ-m8sgjk narrowed): no new hazard,
no rating change.* The app now hands an admission's reply to org-io's
exchange whole and reports what it returns. A committed update whose send
failed is reported as a failure whose message says the update was
committed, so the operator is not told the admission or revocation did not
happen; a retried admission is refused because the Invite was settled, and
a retried revocation finds no Member to remove, so neither is applied
twice. A recipient that did not receive its body learns of the change from
the chain later, a delay already assessed with org-io's fan-out control.

assesses: REQ-m8sgjk
