# Problem reports — removing a device by delete-and-re-add keeps the key

**PR-fzu25w**: a caller can remove a device from a member without replacing the
member-as-a-group key by calling `delete_member` and then `add_member` with the
same identifier, handle and key and one device fewer; the trie accepts both
calls and the result is a member whose removed device still holds the key.
affects: REQ-ewdg2q
opened: 2026-10-03
status: resolved

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

(Amended 2026-10-05: `P2pDeviceSlots::remove_device` no longer exists. Its
successor, `person::DeviceSlots::remove_device`, is public; what keeps an
external caller from installing a smaller device set on an existing member is
now that `MemberLeaf::with_p2p_device_slots` is `pub(crate)` and `add_member`
refuses an identifier already present (`DuplicateId`). The route this report
describes is unchanged by the move.)

Not fixed in the change that found it: that change resolves PR-zz4exm, whose
scope is the replacement key the two device-removal operations accept. Whether
`add_member` should refuse a key the identifier last held, or whether
re-admission is legitimately a fresh start, is a requirement question for the
owner (`grill-requirements`), alongside PR-z463w5 and the wire path.

Resolved, 2026-10-03, by owner ruling; not a defect, and no code change.
Deleting a member is permanent. A member added later is a new member with a new
`MemberId`, whatever its handle or keys, and every delegation is made to a
`MemberId`, never to a handle, because handles change. `MemberId`s are
caller-generated random values, so a deleted `MemberId` is never legitimately
re-added: doing so is a caller error outside the API's contract, and the trie
keeps no record of deleted identifiers and does not enforce it. (Recorded as
`resolved` because the installed guardrails, 0.5.1, has no `accepted` status.)

Scope of the ruling, stated so it is not read wider: only re-adding a
*deleted* `MemberId` is outside the contract. Re-admitting the person under a
fresh `MemberId` with the old key is inside it, and a removed device holding
that key would reach what is later granted to the new member. By owner ruling
(2026-10-03) a re-admitted person is a new member under a fresh `MemberId`
and may bring the keys their previous membership held when it was deleted —
the one exception; nothing granted to the old id carries over. If a removed
device was compromised, fresh keys are the joiner's choice, not a software
check. This is residual risk accepted by the owner (security check 11 in `org-members/README.md`;
doc-comments of `add_member` and `delete_member`).

The key-uniqueness invariant added later the same day (LLR-v6gfc7) does not
catch this route: once the member is deleted its keys are no longer held, so a
fresh `MemberId` given the old key is accepted. That is the accepted residual
above, not a gap left open.
