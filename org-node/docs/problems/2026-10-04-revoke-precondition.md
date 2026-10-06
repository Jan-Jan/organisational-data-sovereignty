# Problem reports — org-node, the revocation precondition

Opened by the architecture tooth
(`docs/plans/2026-10-03-org-node-architecture.md`) while writing the test for
LLR-pw369n, after review round 3 found that clause stated in no low-level
requirement.

**PR-b9wab3**: `revoke_member` in Loopback mode submits the new Membership root
on chain before it checks that it was given a peer address, so a call with no
address advances the on-chain epoch and then refuses, leaving the published
root without the member while the administrator's own record still holds them.
affects: SDD-72ddm6, LLR-pw369n, LLR-6dc598, LLR-qg9utu
opened: 2026-10-04
status: open

## What was observed

`a_loopback_revocation_with_no_peer_address_is_refused_and_records_nothing`
was first written to assert that the refusal precedes the chain write. It failed:
the chain epoch was 3 where the test expected 2. The refusal is real and
typed — `OrgNodeError::Chain("Loopback revoke requires the peer's
EndpointAddr")` — but it sits in the `match mode` block at
`org-node/src/service.rs:1171`, after `submit_update` at `:1129`.
(Amended 2026-10-05 by the org-node type-safety change, review round 7: citation correction only — the lines read `:1212` and `:1170` before that change's edits
to `service.rs`.)

The test now pins the defect rather than the intent: it asserts
`epoch_before + 1`, so moving the check earlier reddens it and this report is
closed deliberately rather than drifting shut.

## Why it is a problem and not a documented order

The project has already ruled that **a failed push leaves the chain
advanced** — `a_failed_push_leaves_the_administrators_record_where_it_was`
records that for `admit_member`, and it is unavoidable there: whether the peer
is reachable cannot be known until the send is attempted.

**This is not that case.** `peer_addr.is_none()` is knowable at the function's
first line, with no I/O. The order is a straightforward precondition checked
late, and the cost is an on-chain epoch burned by a caller mistake, plus a
divergence between the published root and the administrator's record that no
other code path produces deliberately.

## What it is not

It is not a security defect. The divergence is fail-closed for every
*receiver*: the chain root is correct for the change it describes, so a node
that reads it sees a consistent Organisation state, and the administrator's
stale record only costs the administrator — the next change it signs is built
on a trie that no longer matches the published root and every receiver refuses
it. That is a liveness failure for the Organisation, not an integrity one.

## What closing it looks like

Move the `peer_addr` check above the trie delete and the chain write, under
TDD: the reproduction is the existing test, inverted. It belongs in a change
of its own — this one is an architecture ledger and must not alter production
ordering under cover of documenting it.

*Annotated 2026-10-05 by review round 1 of `worktree-worktree-person-shared-types`
(finding-6); reworded the same day by docs/plans/2026-10-05-switch-trim.md.* The
items the `affects:` line names are amended in place on that branch rather
than superseded, so the line names them alone.
