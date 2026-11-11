# Problem reports — org-node, an own revocation on the ordinary receive path

Opened by the architecture tooth from review round 6.

## What was observed

The resolved problem report on an own revocation on the ordinary receive path pin (`crates/org-node/tests/admission_sender.rs`, rewritten 2026-10-06
as `an_own_revocation_on_the_ordinary_path_deletes_the_record`, see the
resolution): B is admitted, then A revokes B and
pushes the change to B's `receive_and_verify`. B returns `Ok` at epoch 3, holds
a record whose trie no longer contains its device, and its Persona is still
Active, in memory and on disk. The author reproduced the reviewer's probe on
this tree before booking it.

## Why it is a defect

An earlier form of REQ-48zc3e said what a node does on committing a Change set that removes its
own Device key, and it is **not scoped to a receive operation**. The ledger
allocates it to an earlier form of SDD-z4m57w, the self-delete path, alone. `revoke_member`'s own
doc comment (`crates/org-node/src/service.rs:1084-1087`) expects the Member it removes
to clean up "when they call `receive_and_verify`". (Amended 2026-10-05 by the org-node type-safety change, review round 7: citation correction only — the
range read `:1123-1126` before that change's edits to `service.rs`.) An earlier form of RC-5jkcxz, the cooperative
self-delete control in `crates/org-node/docs/risk/2026-09-09-org-node-hazards.md`,
does not hold on this path.

The shipped app receives only on `receive_and_self_delete_if_revoked`
(`apps/desktop/src-tauri/src/commands.rs:339`), so the app's receiver does not reach
this. A library caller using the ordinary path does.

## What closing it looks like

Either `receive_and_verify` applies the same own-removal rule as the
self-delete path, or the ordinary path refuses a Change set that removes the
receiving node and directs the caller to the other operation. Which one is a
design decision, made with the fix and under TDD with the pin inverted. It
belongs in the fix change that closes the resolved problem report on the receive path rebinding Personas, because both concern what
the receive paths do with the node's own membership.

**This is the tenth open problem report in org-node, against
`problem_open_max: 10`.** The gate passes at ten and fails at eleven.

*Annotated 2026-10-05 by review round 8.* LLR-cja9zv states the commit this
report books, and now names it. The pin carries LLR-cja9zv.

*Annotated 2026-10-05 by review round 1 of `worktree-worktree-person-shared-types`
(finding-6); reworded the same day.* The
items the `affects:` line names are amended in place on that branch rather
than superseded, so the line names them alone.
