# Problem reports — device removal does not enforce key replacement

**PR-zz4exm**: `delete_p2p_device` and `emergency_isolate_member` accept a
replacement member-as-a-group key from the caller and store it without checking
that it differs from the key being replaced, so a caller that passes the
current key back removes the device while leaving its access intact.
affects: REQ-ewdg2q, REQ-r784fu.
owner: Jan-Jan
opened: 2026-08-31
status: resolved

Found while writing REQ-ewdg2q, not by a failing test. The requirement says the
software shall replace the key "so that a removed device cannot derive access
from the key it held while enrolled"; `org-members/src/trie.rs`
(`delete_p2p_device`) does `existing.with_p2p_device_slots(new_slots)
.with_p2p_key(new_p2p_key)` with no comparison against the previous key. The
type system does not help here: both keys are `P2pMemberKey`, so passing the old
one is a well-typed call that silently defeats the revocation.

Why this is a problem report rather than a requirement change: the intended
behaviour is not in doubt. The operation exists to cut off a removed device, the
crate's own API table describes the `new_key` parameter as required "because the
deleted device had access to the old key", and a call that reuses the old key
satisfies neither. What is missing is the check, not the intent.

No test covers the degenerate case, which is why the reviewer had to reason
about the code rather than watch something fail. Per `resolve-problem`, the fix
starts with a test that reproduces it: `delete_p2p_device` called with the
member's current key must return an error, and the same for
`emergency_isolate_member`. That test is what turns REQ-ewdg2q's abnormal-input
coverage from absent into present — see the verification record for this change,
which lists that absence as a known gap.

Not fixed in this change: this is the requirements tooth, and changing library
behaviour under it would mix a requirements migration with a code fix. It is the
first item in this ledger and its age is what the merge gate now watches.

Resolved, 2026-10-03. The owner decided that an unchanged replacement key is an
error and the operation is atomic: `delete_p2p_device` and
`emergency_isolate_member` now refuse a replacement key equal to the member's
current key with the new `OrgMembersError::P2pKeyNotReplaced`, after the member
(and, for `delete_p2p_device`, device) lookup, and change nothing — the device
stays enrolled and the key stays as it was. Fixed on branch
`worktree-fix-stale-problems` (plan `docs/plans/2026-10-03-fix-stale-problems.md`,
verification record `docs/verification/` for that branch); its worktree commits
are squashed away, so the fixing commit is that change's squash commit on
master, which also amends LLR-s97ywt and LLR-w92psx to state the refusal. The
reproducing tests asked for above are `delete_p2p_device_rejects_unchanged_key`
and `emergency_isolate_member_rejects_unchanged_key`
(`org-members/tests/integration_test.rs`), both watched fail with `Ok(..)`
before the guard existed; the property test `device_removal_never_keeps_key`
(`fuzz_tests.rs`) and the Quint model `org-members/quint/membership.qnt`
(at the top-level `quint/` when this fix landed; moved by the quint
conformance gate change, 2026-10-03), whose conformance
run now generates the same refusal, cover it as well. `rotate_p2p_key` was
deliberately left alone: it removes no device, so REQ-ewdg2q does not reach it.
