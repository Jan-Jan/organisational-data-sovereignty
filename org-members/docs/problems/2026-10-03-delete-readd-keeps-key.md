# Problem reports — removing a device by delete-and-re-add keeps the key

**PR-fzu25w**: a caller can remove a device from a member without replacing the
member-as-a-group key by calling `delete_member` and then `add_member` with the
same identifier, handle and key and one device fewer; the trie accepts both
calls and the result is a member whose removed device still holds the key.
affects: REQ-ewdg2q
opened: 2026-10-03
status: open

Found by the fourth independent review round of the change that resolved
PR-zz4exm (`worktree-fix-stale-problems`), and reproduced there by a probe test
that passed: `delete_member(id)`, then `add_member` of a leaf with the same
`MemberId`, handle and `P2pMemberKey` and the device set minus one, then
`recalculate()`, leaves the member with one fewer device and the key unchanged.
The probe was not committed; the reproducing test is the first step of the fix,
per `resolve-problem`.

Why it matters: RC-mqtks7's first clause rests on the claim that "an external
caller must go through `delete_p2p_device`" to remove a device
(`org-members/docs/risk/2026-09-02-membership-hazards.md`, HAZ-s39gbh's residual
risk). That is true of `P2pDeviceSlots::remove_device`, which is crate-private,
but not of the membership operations as a whole: off-boarding and re-admitting
the member reaches the same end state with no replacement key. It is the
direct-API counterpart of the wire-path bypass (a change set may upsert a leaf
with a device removed and the key unchanged), and the same rule would close
both — relate a leaf's device set to its key across a change, not only within
one operation.

Not fixed in the change that found it: that change resolves PR-zz4exm, whose
scope is the replacement key the two device-removal operations accept. Whether
`add_member` should refuse a key the identifier last held, or whether
re-admission is legitimately a fresh start, is a requirement question for the
owner (`grill-requirements`), alongside PR-z463w5 and the wire path.
