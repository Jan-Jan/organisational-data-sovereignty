# The own-removal refusal for a Persona with no MemberId

Added 2026-11-11 by change `worktree-upgrade-review-problems`, under SDD-kegag2
(verification of one Organisation against the chain) and SDD-z4m57w (the
removed store).

**LLR-f727y8**: when verification finds the Persona removed, the count of LLR-6fsnma is not zero, the Persona's secret is returned and the Persona has no MemberId, `verify` refuses with `OrgNotHeld { org_id }` before anything is signed, returning no `StoreChange` and leaving the store as it was.
satisfies: derived

The service never produces this state: a store holds a record only after an
adoption, and an adoption sets the Persona's MemberId with it. The refusal
guards the state rather than signing an acknowledgement that names no Member.
