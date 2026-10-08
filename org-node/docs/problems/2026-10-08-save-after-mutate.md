# Problems — a commit whose save fails, from S2's review

Recorded 2026-10-08 by the independent review of change
`worktree-org-io-create` (round 1, finding-9). The file is dated, not a
DRAFT, because that change's ledgers were already finalized when the review
ran.

**PR-jav8zn**: A `commit_genesis` or `commit_update` whose store save fails returns `Err` after it has already changed the in-memory record (the Organisation record pushed and the Persona bound, or the committed store swapped in), so org-io reports the commit as "nothing committed" while the open service holds it, and the failed `std::fs::write` itself is labelled `OrgNodeError::Chain` (in `PersonaStore::save`, `org-node/src/store.rs`; the line reference `:568` was corrected 2026-10-08, review round 2, finding 12).
affects: LLR-ewkg85, LLR-cmdrp9, SDD-af5vnt
opened: 2026-10-08
status: open

Where: `commit_genesis` pushes the `OrgRecord`, drops the provisional update
and binds the Persona, then calls `self.store.save(rng)?`
(`org-node/src/service.rs:275-289`); `commit_update` assigns the committed
store to `data_mut()`, then saves (`:447-449`). On a failed save both return
`Err` with memory already mutated. org-io's `refused_after_write`
(`org-io/src/submit.rs`) then says the chain write executed and "nothing
committed". Until the process exits the record in memory disagrees with that
message and with the file on disk; after a restart the file wins, and it
still holds the provisional update the in-memory path had dropped. LLR-ewkg85 ("nothing written to disk", the record unchanged) is
stated for a refusal at a step before the save, so the defect is in what a
failed save leaves behind, not in a refused check. The gap is old; S2's new
message only states it as fact.

The mislabel: `PersonaStore::save` maps a failed encode and a failed
`std::fs::write` to `OrgNodeError::Chain`, so the app and org-io classify a
local disk failure as a chain failure.

Related, not booked as a problem report: the store write is not atomic (a
crash between truncation and completion leaves no readable copy). That is
recorded only as recommended control 10, "Atomic store write", in
`org-node/docs/risk/2026-09-09-org-node-hazards.md`, with no PR ID. A fix
for this report (save to a copy, then swap memory only on success; a typed
storage error) is worth doing together with it.
