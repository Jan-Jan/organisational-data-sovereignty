# A checked message committed after its record moved

Found by the S4 deslop pass on 2026-10-14
(`docs/plans/2026-10-06-org-io-transport.md`, "Deslop pass"); investigated
and fixed in plan task T22.

**PR-erskh4**: `commit_checked` decides a checked message against the
record values taken at check time, even when a local commit has changed
that record since the check, so it can accept a body that a fresh check
of the same body would refuse.
affects: LLR-6qjmhf, LLR-zetn7a
opened: 2026-10-14
status: resolved
resolution: fixed 2026-10-14 (change `worktree-org-io-transport`, plan task
T22): `check_received` records the marks (epoch, root, sequence guard) of
the record it read in the `CheckedMessage`, and `commit_checked` refuses
with `RecordMovedSinceCheck` when they moved, before anything else;
org-io's receive step repeats the check, the read and the commit once on
that refusal. Reproduced by `org-node/tests/stale_check.rs` (three tests
watched fail first) and, in org-io, by
`a_check_that_went_stale_is_checked_again_once`.

Where: `OrgService::check_received` keeps the held record's trie, sequence
guard and epoch in `CheckedMessage` (`CheckedKind::OrgInformation`).
`OrgService::commit_checked` verifies the Envelope against those values and
the chain state it is given. Since S4 task T11, org-io's receive step
releases the node between the check and the commit for the one chain read,
so a local commit (a submission's `commit_update`, a `reconcile`) can land on
the same Organisation in between.

What happened before the fix, at org-node's interface (check a received
body at B; commit a local change to the same Organisation; commit the
earlier check against the chain as it then stood; then check the body
afresh). B's record was at epoch 2 in each case.

- Organisation information, another Member's update: A's update for epoch
  3 checked; B's own update took epoch 3 on the chain and B committed it.
  The stale commit was refused (`RootMismatch`); nothing changed. A fresh
  check refuses the body as a replay (`StaleSeq { got: 3, last_seen: 3 }`).
  Nothing was lost: A's update never reached the chain.
- Organisation information, B's own update relayed back to B by a listed
  Member: checked before B's `commit_update` of it. The stale commit was
  **accepted** (`ReceiveKind::Updated`) and rewrote the record with the
  values it already held (the plaintext unchanged), while a fresh check
  refuses it (`StaleSeq`). No local commit was lost, but an accepted body
  that a fresh check refuses is the defect this report names.
- Revocation: A's removal of B (epoch 3) checked; B then committed its own
  update against a state, given as a value, holding it at epoch 3. The
  stale commit was refused (`RevocationProofRefused`), as a fresh check and
  commit refuse it. `revocation::accept` reads the store afresh; only the
  member-sender rule is from check time.
- First admission: checked while B's Persona expected it; B's Persona then
  founded an Organisation of its own. The commit refused it
  (`AdmissionNotOurs`), as a fresh check and commit do. No record of the
  Organisation existed at the check or after, so the marks do not move and
  the fix leaves this outcome as it was (pinned, mutation-checked).
- Acknowledgement: no case. `check_received` decides it against the store
  alone (`Checked::Acknowledged`), under one lock in org-io, and nothing
  commits it later.

In every case the commit wrote onto the store as it stood at the commit
(`OrgService::change` clones the current data), so no local commit was
overwritten. Why a valid message is not lost in practice: every local
commit that moves a record commits the chain's own state for this node's
own update, so a received update checked against the earlier record cannot
also be on the chain at that epoch, unless it is that same update.
