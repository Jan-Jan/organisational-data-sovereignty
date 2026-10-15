# Store writes are not atomic

*Moved 2026-10-13 from org-node's problems ledger (this change's draft
there, now deleted), plan task T9 of
`docs/plans/2026-10-06-org-io-transport.md`: the store write moved to
org-io, and the fix landed here.*

Found by the S4 survey (`docs/plans/2026-10-06-org-io-transport.md`) on
2026-10-06. Fixed in stage S4 of `docs/plans/2026-10-06-org-io-roadmap.md`,
when storage IO moves to org-io (owner ruling of 2026-10-06: S4 makes store
writes atomic and owner-only; the owner-only part was dropped on 2026-10-07,
because the file's contents are encrypted and a file ACL is too weak a
protection to rely on).

**PR-n696wv**: the sealed store is written in place with `std::fs::write`, so
a crash, a full disk or a kill during the write leaves a truncated file that
no longer unseals, and every secret in it, the Persona and Organisation keys
included, is lost.
affects: SDD-egx8rv, LLR-b9bm36
opened: 2026-10-06
status: resolved
resolution: fixed 2026-10-13 (change `worktree-org-io-transport`, plan task
T9): org-io's `store_file::write_store` writes the sealed bytes to a new
temporary file in the store's directory (`create_new`), syncs it, renames it
over the Persona's store file and, on Unix, syncs the directory; a failure
at any step removes the temporary and names the step (LLR-b9bm36,
REQ-zmtmd9, RC-37y46c). Tests `the_file_holds_exactly_the_sealed_bytes` and
`a_write_interrupted_after_the_temporary_leaves_the_old_store_unsealing`
(`org-io/tests/store_files.rs`). Outstanding invites live inside the sealed
stores since task T6, so the second writer is gone (LLR-t4v69b,
`no_code_names_the_old_files`).

Where (when found): `org-node/src/store.rs`, both `std::fs::write` calls
(creating the store, and saving it). The app's outstanding-invites file,
`app/src-tauri/src/invitation.rs`, was written the same way.

Why it is a defect: an interrupted write destroys the previous good copy
along with the new one, so a single failure loses the node's identity and its
Organisations.

Fix, in S4: org-io writes the sealed bytes to a temporary file in the same
directory, syncs it, and renames it over the old file. Outstanding invites
move inside the encrypted store, so the second writer goes away.

*Noted 2026-10-08 (reconciled with S2, `e41cb09`):* S2 booked the related
save-after-mutate report (org-node's `2026-10-08-save-after-mutate.md`: a failed save
leaves memory ahead of disk, and is labelled a chain error), which names
this atomicity gap as "worth doing together with it". S4's plan resolves
both: org-node stops saving and returns the store change (task T3), and
org-io writes it atomically, reports a failed write as a storage failure
and reloads the Persona from its file (task T9, LLR-murjr6).

*Noted 2026-10-08 (S4, task T3):* org-node writes no file since task T3:
`org-node/src/store.rs` seals and unseals bytes, and the two `std::fs::write`
calls named under "Where" are gone from it. The write moved to org-io's
transitional `persist` (a plain `std::fs::write` of each Persona's sealed
store to its own file), so the defect stood there until task T9 replaced
`persist` with `write_store`.
