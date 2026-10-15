# Risk analysis — the Organisation key pair (app)

Run 2026-10-06 for change `worktree-org-node-org-key-pair`, under IEC 62304
Class C. The owner's rulings are in
`org-node/docs/requirements/2026-10-07-org-key-pair.md`.

## Derived requirements assessment

*Moved 2026-10-14 (S4, task T13 of docs/plans/2026-10-06-org-io-transport.md,
ruling A).* The assessment that stood here, of the reply requirement as the
key-pair change amended it (the expectation names the Organisation alone; no
new hazard), moved with the requirement to org-io's risk ledger
(`org-io/docs/risk/`, the transport file of change
`worktree-org-io-transport`). What it also recorded stays true of the app:
the app supplies no Organisation secret to org-node at all (the admission's
key comes from org-node's own record), which removed the path by which a
user-typed value became the Members' shared secret
(`app/docs/risk/2026-10-05-derived.md`, "The admission's Organisation
secret").
