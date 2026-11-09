# Problems found during change `worktree-guardrails-upgrade`

Recorded 2026-11-09 from the review of change `worktree-guardrails-upgrade`
(the ledger migration to the guardrails d6e285e grammar). It needs an owner
decision, so it is filed open and not fixed in that change.

**PR-af784d**: RC-jrkn7w still requires the revoke form's peer-address
precondition to follow the transport mode, a clause nothing realises since no
peer address is taken.
affects: RC-jrkn7w
opened: 2026-11-09
status: open
