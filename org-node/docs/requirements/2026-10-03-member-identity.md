# Requirements — where a member's identifier comes from

One requirement, minted while resolving PR-g7cfns. org-node made a member's
`MemberId` by copying the bytes of its Member-as-a-group key
(`member_id_from_key`, a proof-of-concept shortcut), so the identifier was a
function of a key that is meant to change: after a Member rotates that key
their identifier (rightly kept) still equals the retired key, and a revoked person who rejoins with the
same keypair got their deleted identifier back. org-members states the opposite contract — a `MemberId` is a fresh random
value chosen by the caller, and a deleted one is never re-added
(`org-members/README.md`, "Security checks the caller MUST perform", item 11) —
and `docs/CONTEXT.md` defines the identifier as independent of the handle and of
every key. org-node is the caller, so the `MemberId` duty — never re-add a
deleted `MemberId` — is met here or nowhere; with REQ-d9g6nt org-node meets it
by construction, since every `MemberId` it admits is freshly drawn at random.

Owner ruling, 2026-10-03: the admin draws the identifier at random *before*
calling org-members, so `genesis` and `add_member` stay pure functions of their
inputs; nothing relates a `MemberId` to any key; a re-admitted person is a new
member under a fresh `MemberId` and may bring the keys their previous membership
held when it was deleted — the one exception; nothing granted to the old id
carries over; if a removed device was compromised, fresh keys are the joiner's
choice, not a software check (accepted residual); the duty not to supply any
other key no longer held stands — a Member-as-a-group key replaced earlier, and
the Device key of a device removed earlier while the member stayed; a deleted
member's keys are never given to anyone else; and a member is revoked by its
`MemberId` (`delete_member`), never by a key. The software keeps no key
history: org-node passes the joiner's keys from the Join request to
org-members unchanged, and neither org-node nor org-members enforces the two
key duties — the administrator who admits or rotates carries them.

## Member identity

**REQ-d9g6nt**: The software shall give every Member it places in a
Membership record — the founding Member at creation and each joiner at
admission — a `MemberId` of 32 bytes drawn from its cryptographic random source
before the Membership record is built, and shall never compute a `MemberId`
from a key; a first admission that arrives without the snapshot of the
Membership record it extends shall be refused explicitly rather than attempted
against a reconstructed Membership record.
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`):*
"the founding administrator" reads "the founding Member"; org-node has no
concept of an administrator. Creation and admission build provisional
updates, so a `MemberId` is drawn when the provisional update is built.

The last clause is part of the same requirement, not a second one: the only
path that reconstructed a Membership record without the snapshot did so by
computing the administrator's identifier from the Organisation's public key.
That path could not succeed — the identifier it computed was not the
administrator's, so the reconstructed Membership record failed the base-root
check, and since org-members' key uniqueness (2026-10-03) its leaf, which used
one key as both Member-as-a-group key and Device key, is refused at genesis — so refusing it removes no working
behaviour, and keeps the "never from a key" clause true without an exception.

Revoking by `MemberId` — `revoke_member` calls `delete_member` on the
identifier, never on a key — is existing behaviour that the owner's ruling
confirms. It is outside REQ-d9g6nt's scope and changes nothing here.

Verified at the service API, normal and abnormal cases both, by
`member_ids_are_not_derived_from_keys`,
`readmission_with_same_keys_gets_a_fresh_member_id` and
`first_admission_without_a_record_snapshot_is_refused` in
`org-node/tests/admission_sender.rs`, by
`the_founding_member_id_and_the_organisation_key_are_drawn_not_derived` in
`org-node/tests/commit_paths.rs`, and by the fuzz target
`fuzz_first_admission_base`
(`org-node/tests/fuzz_first_admission_base/fuzz_target.rs`).

*Amended 2026-10-06 (independent review round 2, finding-3).* The list named
`same_persona_founding_two_organisations_gets_two_member_ids` (named
`…_gets_two_admin_ids` until earlier on 2026-10-06) in
`org-node/tests/admission_sender.rs`. R1a of change
`worktree-org-node-chain-authority` deleted it: a Persona bound to one
Organisation can no longer found another (REQ-yp75u9, carried by
`a_persona_bound_to_an_organisation_cannot_found_another`), and the drawn
founding `MemberId` is now verified by
`the_founding_member_id_and_the_organisation_key_are_drawn_not_derived`.
