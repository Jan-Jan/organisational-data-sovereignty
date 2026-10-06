# Problem reports — org-node, an own revocation on the ordinary receive path

Opened by the architecture tooth
(`docs/plans/2026-10-03-org-node-architecture.md`) from review round 6.

**PR-322qst**: a Change set that removes the node's own Device key, when it
arrives through `receive_and_verify` and not through
`receive_and_self_delete_if_revoked`, is committed as an ordinary update. The
node keeps its record of the Organisation it was removed from, and its
Persona stays Active.
affects: REQ-uxv2x2, SDD-8cpyfa, SDD-72ddm6, LLR-cja9zv
opened: 2026-10-04
status: open

## What was observed

`pr_322qst_an_own_revocation_on_the_ordinary_path_is_committed_not_self_deleted`
(`org-node/tests/admission_sender.rs`): B is admitted, then A revokes B and
pushes the change to B's `receive_and_verify`. B returns `Ok` at epoch 3, holds
a record whose trie no longer contains its device, and its Persona is still
Active, in memory and on disk. The author reproduced the reviewer's probe on
this tree before booking it.

## Why it is a defect

REQ-uxv2x2 says what a node does on committing a Change set that removes its
own Device key, and it is **not scoped to a receive operation**. The ledger
allocates it to SDD-72ddm6, the self-delete path, alone. `revoke_member`'s own
doc comment (`org-node/src/service.rs:1084-1087`) expects the Member it removes
to clean up "when they call `receive_and_verify`". (Amended 2026-10-05 by the org-node type-safety change, review round 7: citation correction only — the
range read `:1123-1126` before that change's edits to `service.rs`.) RC-wqgm2p, the cooperative
self-delete control in `org-node/docs/risk/2026-09-09-org-node-hazards.md`,
does not hold on this path.

The shipped app receives only on `receive_and_self_delete_if_revoked`
(`app/src-tauri/src/commands.rs:339`), so the app's receiver does not reach
this. A library caller using the ordinary path does.

## What closing it looks like

Either `receive_and_verify` applies the same own-removal rule as the
self-delete path, or the ordinary path refuses a Change set that removes the
receiving node and directs the caller to the other operation. Which one is a
design decision, made with the fix and under TDD with the pin inverted. It
belongs in the fix change that closes PR-mdv38y, because both concern what
the receive paths do with the node's own membership.

**This is the tenth open problem report in org-node, against
`problem_open_max: 10`.** The gate passes at ten and fails at eleven.

*Annotated 2026-10-05 by review round 8.* LLR-cja9zv states the commit this
report books, and now names it. The pin carries LLR-cja9zv.

*Annotated 2026-10-05 by review round 1 of `worktree-worktree-person-shared-types`
(finding-6); reworded the same day by docs/plans/2026-10-05-switch-trim.md.* The
items the `affects:` line names are amended in place on that branch rather
than superseded, so the line names them alone.
