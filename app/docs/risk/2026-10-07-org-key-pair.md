# Risk analysis — the Organisation key pair (app)

Run 2026-10-06 for change `worktree-org-node-org-key-pair`, under IEC 62304
Class C. The owner's rulings are in
`org-node/docs/requirements/2026-10-07-org-key-pair.md`.

## Derived requirements assessment

**REQ-tcutr6** (amended: the app declares to org-node that it expects a first
admission to an Organisation, without the invite identifier): the invite
identifier still binds an Invite reply to the Invite the administrator issued
(REQ-65xqp8, unchanged), which is where the owner's journey uses it. On the
joiner's side org-node now matches a first admission against the Organisation
alone; its risk analysis
(`org-node/docs/risk/2026-10-07-org-key-pair.md`)
finds the residual of its first-admission control unchanged. The app stops supplying
an Organisation secret to org-node at all (the admission's key now comes from
org-node's own record), which removes the path by which a user-typed value
became the Members' shared secret (`app/docs/risk/2026-10-05-derived.md`, "The
admission's Organisation secret"). No new hazard.

assesses: REQ-tcutr6
