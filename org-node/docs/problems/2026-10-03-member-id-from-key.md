# Problem reports — a revoked member who rejoins gets their deleted MemberId back

**PR-g7cfns**: org-node derives a member's `MemberId` from their member key
(`member_id_from_key` copies the key bytes), so after `revoke_member` deletes a
member, an `admit_member` of a Join request carrying the same member key re-adds
the deleted `MemberId` — which org-members states is outside its contract — and
the member returns with the key the removed devices held.
affects: REQ-q92yac
opened: 2026-10-03
status: open

Found by the independent review of the change that made org-members hold every
key once (branch `worktree-rotate-same-key`), not by a failing test. Where:
`member_id_from_key` in `org-node/src/service.rs` (its own comment calls it a
"PoC choice" and suggests hashing the member key with the organisation id,
which would reproduce the same identifier for the same key), used by
`create_organisation` and `admit_member`; `revoke_member` calls
`delete_member` on that identifier.

The provider's contract, stated in `org-members/README.md` ("Security checks the
caller MUST perform", item 11) and in the doc-comments of `add_member` and
`delete_member`: deleting a member is permanent; a `MemberId` is a fresh random
value; a deleted identifier must never be re-added; a person re-admitted later is
a new member under a new identifier and must not be given a key a device of
their previous membership held. org-members does not enforce the identifier rule
(it keeps no record of deleted identifiers), and its key-uniqueness check does
not catch the old key either, because once the member is deleted that key is no
longer held. So org-node is the only place the duty can be met, and its
derivation guarantees it is broken whenever a revoked person rejoins with the
same member keypair.

Observable symptom, as a test would show it: admit B, revoke B, then admit a
Join request carrying B's member key — the second admission succeeds and the
record holds a member whose `MemberId` and member key equal the revoked
member's. Reproduced by reading, not by running; the reproducing test is the
first step of the fix.

What deciding it involves, for the owner: whether `MemberId` stays derived from
the member key (then a rejoin with the same keypair must be refused, or the
joiner must present a fresh member key), or becomes a fresh random value at
admission (then every place that recomputes an identifier from a key — the
admin lookup in `receive_and_verify`'s fallback among them — must look it up in
the record instead). Not fixed in the change that found it: that change is
org-members' key uniqueness, and this is a design decision in org-node.
