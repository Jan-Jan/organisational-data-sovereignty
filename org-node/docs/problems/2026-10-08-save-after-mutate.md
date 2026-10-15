# Problems — a commit whose save fails, from S2's review

Recorded 2026-10-08 by the independent review of change
`worktree-org-io-create` (round 1, finding-9). The file is dated, not a
DRAFT, because that change's ledgers were already finalized when the review
ran.

**PR-jav8zn**: A `commit_genesis` or `commit_update` whose store save fails returns `Err` after it has already changed the in-memory record (the Organisation record pushed and the Persona bound, or the committed store swapped in), so org-io reports the commit as "nothing committed" while the open service holds it, and the failed `std::fs::write` itself is labelled `OrgNodeError::Chain` (in `PersonaStore::save`, `org-node/src/store.rs`; the line reference `:568` was corrected 2026-10-08, review round 2, finding 12).
affects: LLR-ewkg85, LLR-cmdrp9, SDD-af5vnt
opened: 2026-10-08
status: resolved
resolution: fixed 2026-10-13 (change `worktree-org-io-transport`, plan tasks
T3 and T9). org-node half (T3): org-node no longer saves; each operation
changes a copy, seals it and adopts it only once sealed, returning the
change for org-io to write, so a refusal leaves memory as it was. org-io
half (T9): org-io writes the change atomically; when the write fails it
reports a storage failure, not a chain error (org-io's `StoreFailure`,
naming the failed file step), and puts the Persona back as its file holds
it (org-node's `OrgService::restore_store`), so memory never runs ahead of
the disk. A commit whose write failed is reported as "the chain write
executed and the commit was made, but storing it failed; reloaded from the
file", not "nothing committed". Reproducing test
`a_commit_whose_store_write_fails_says_the_commit_was_made_and_reloads`
(`org-io/tests/store_files.rs`).

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

*Fixed on org-node's side 2026-10-08 (S4, task T3 of
docs/plans/2026-10-06-org-io-transport.md):* org-node no longer saves.
Every operation that changes a store makes the change on a copy of that
Persona's store data, seals the copy, and adopts it only once sealing has
succeeded, returning the sealed bytes as a `StoreWrite` (or
`StoreChange`) for org-io to write; a refusal, of a check or of the seal,
returns no write and leaves memory as it was (`OrgService::change` and
`OrgService::adopt`, `org-node/src/service.rs`; the `StoreWrite` is
`#[must_use]`, and the workspace denies `unused_must_use`). A failed
encode or seal is still labelled `OrgNodeError::Chain` until org-io's half.
org-io's half — write atomically, report a failed write as a storage
failure, reload the Persona from its file — is task T9, which resolves this
report. Until then org-io's transitional `persist` writes in place after
the commit, so a failed file write still leaves the committed record in
memory, and org-io reports it as "committed, but writing the store failed".

*Scope widened 2026-10-13 (owner ruling; change `worktree-org-io-publish`,
plan `docs/plans/2026-10-08-org-io-publish.md` §6a).* `discard_provisional`
(`org-node/src/service.rs`; LLR-7cmp38) has the same shape: it removes the
update from memory, then saves, and on a failed save returns `Err` without
restoring it, so the Organisation private key the update holds stays on
disk while memory says it is gone, until the next successful save. Found by
S3b-io's task T15d, whose background discard (the settle of a write known
not to have executed) reports a failed save as still unknown and retries.
The whole memory-ahead-of-disk class — `commit_genesis`, `commit_update`
and `discard_provisional` — is for S4 (its tasks T3 and T9, where org-node
returns the store change instead of saving and org-io writes it) to fix;
this change does not fix it.

*Clarified 2026-10-14 (master's S3b-io part 1 merged into
`worktree-org-io-transport`):* the widened scope is met by the same fix.
Since S4's task T3, `discard_provisional` changes a copy through
`OrgService::change` and returns a `StoreWrite`, saving nothing; org-io's
settle writes it through its store files, and a failed write reloads the
Persona from its file (task T9), so memory never says the update is gone
while the file still holds it. The settle then reports the write still
unknown, as S3b-io's T15d expects. The "saves" and "failed save" above are
S2's behaviour as the widening found it.

*Note 2026-10-14 (review round 1 of S4, finding-16):* the reproducing test
named in the resolution is org-io's
`store_files::a_commit_whose_store_write_fails_says_the_commit_was_made_and_reloads`
(`org-io/tests/store_files.rs`). It is named here in prose only: its
`verifies:` line cannot cite this report, because a test in one unit may
cite only that unit's items and PR-jav8zn is org-node's, so the test cites
org-io's own LLRs.
