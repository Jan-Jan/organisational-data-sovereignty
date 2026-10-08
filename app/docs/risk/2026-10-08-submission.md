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
update to the other Members' Devices remains the open problem PR-3ue4va
(`app/docs/problems/2026-10-07-update-fan-out.md`), unchanged by this
change. The submission's own order and failure rule, and the custody of the
signatory key that signs it, moved to org-io and are assessed in org-io's
risk ledger with the requirement that states them.

assesses: REQ-m8sgjk
