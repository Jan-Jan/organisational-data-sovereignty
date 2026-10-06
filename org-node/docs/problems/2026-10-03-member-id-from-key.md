# Problem reports — a revoked member who rejoins gets their deleted MemberId back

**PR-g7cfns**: org-node derives a member's `MemberId` from their member key
(`member_id_from_key` copies the key bytes), so after `revoke_member` deletes a
member, an `admit_member` of a Join request carrying the same member key re-adds
the deleted `MemberId` — which org-members states is outside its contract — and
the member returns with the key the removed devices held.
affects: REQ-q92yac
opened: 2026-10-03
status: resolved
resolution: by REQ-d9g6nt. org-node now draws each `MemberId` at random before
the record is built, for the founding administrator in `create_organisation`
and for each admitted member in `admit_member`; `member_id_from_key` is
deleted, and a first admission without a record snapshot is refused explicitly
instead of being attempted against a record rebuilt with a key-derived id
(an attempt that never succeeded). Verified by
`member_ids_are_not_derived_from_keys`,
`readmission_with_same_keys_gets_a_fresh_member_id` and
`first_admission_without_a_record_snapshot_is_refused` in
`org-node/tests/admission_sender.rs`, by
`the_founding_member_id_and_the_organisation_key_are_drawn_not_derived` in
`org-node/tests/commit_paths.rs`, and by the fuzz target
`org-node/tests/fuzz_first_admission_base/fuzz_target.rs`. The second half of
the report — the member returns with the keys the removed devices held — is
resolved by owner ruling (2026-10-03): a re-admitted person is a new member
under a fresh `MemberId` and may bring the keys their previous membership held
when it was deleted — the one exception; nothing granted to the old id carries
over. If a removed device was compromised, fresh keys are the joiner's choice,
not a software check. This is residual risk accepted by the owner.

*Amended 2026-10-06 (independent review round 2, finding-3).* The resolution
named `same_persona_founding_two_organisations_gets_two_member_ids` (named
`…_gets_two_admin_ids` until earlier on 2026-10-06) in
`org-node/tests/admission_sender.rs`. R1a of change
`worktree-org-node-chain-authority` deleted it, because a Persona bound to one
Organisation can no longer found another
(`a_persona_bound_to_an_organisation_cannot_found_another`); the drawn
founding `MemberId` is verified by
`the_founding_member_id_and_the_organisation_key_are_drawn_not_derived`.

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
their previous membership held (the contract as stated before the owner's
ruling of 2026-10-03, which the resolution above records). org-members does not enforce the identifier rule
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
