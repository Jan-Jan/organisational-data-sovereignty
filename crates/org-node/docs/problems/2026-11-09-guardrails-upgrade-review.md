# Problems found during change `worktree-guardrails-upgrade`

Recorded 2026-11-09 from the review of change `worktree-guardrails-upgrade`
(the ledger migration to the guardrails d6e285e grammar). Each needs an owner
decision, so it is filed open and not fixed in that change.

**PR-83cqyv**: SDD-ueh4tm names SDD-busuj3 among the items its
`Option<OrgState>` boundary makes verifiable, though SDD-busuj3 takes no
chain state.
affects: SDD-ueh4tm
opened: 2026-11-09
status: open

**PR-fwf3dy**: no in-force item states that an own removal of a Persona with
no MemberId is refused with `OrgNotHeld` and the store unchanged, though
`a_removal_of_a_persona_with_no_member_id_is_refused_with_nothing_deleted`
tests it.
affects: LLR-5tvn7g
opened: 2026-11-09
status: open
