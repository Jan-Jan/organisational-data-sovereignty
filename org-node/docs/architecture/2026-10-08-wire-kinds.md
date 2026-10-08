# Design — the Wire message of three kinds, under its own ID

Low-level requirement for change `worktree-org-io-create` (stage S2 of
`docs/plans/2026-10-06-org-io-roadmap.md`), written under the owner's ruling
of 2026-10-08 on PR-zf924s (`org-node/docs/problems/2026-10-07-close-out.md`):
LLR-js9dsu (`2026-10-07-org-key-pair.md`) changed meaning in S3a, from two
message kinds to three (update, revocation notice, acknowledgement), and its
revocation payload changed from a members trie carried in an Envelope to an
absence proof. Under the hybrid amendment rule a change of meaning mints a
new ID that supersedes the old. The item below sits under SDD-kwncn7 (the
wire frame and its bound, `org-node/src/transport/wire.rs`), as LLR-js9dsu
does, and states the rule that holds now.

**LLR-tajh9d**: `WireMessage` is an enum of exactly three kinds, encoded by
postcard as the kind's variant index followed by its payload's fields in
order: `OrgInformation { envelope: Envelope, record_snapshot: Vec<u8>,
org_private_key: OrgPrivateKey }` (index 0, an update: the committed
Envelope, the record snapshot it extends and the Organisation private key of
the epoch it reaches), `Revocation(RevocationNotice)` (index 1: the revoked
Device's identity and an absence proof, no Envelope, LLR-dc45ur) and
`Acknowledgement(Acknowledgement)` (index 2: a revoked Device's signed
acknowledgement, LLR-378cj4). No field of any kind is optional, and none
carries an invite identifier or any other secret. `WireMessage::org_id(&self)
-> OrgId` returns the Organisation identifier of any kind. `decode_body`
refuses with `Malformed`, and does not panic, a body whose variant index is
not 0, 1 or 2 (an unknown kind), an `OrgInformation` body that ends before
its record snapshot or before the 32 bytes of its Organisation private key,
and a body of any of the three kinds with bytes left over after its last
field.
satisfies: REQ-c29s93, REQ-3dsweu
supersedes: LLR-js9dsu

*Supersession (2026-10-08, owner's ruling on PR-zf924s, hybrid).* The text is
LLR-js9dsu's as amended twice on 2026-10-07 (change
`worktree-org-io-commit-workflow`: S3 T4, which added the third kind and
replaced the revocation's Envelope with a `RevocationNotice`; review
finding-3 of round 1, which stated the leftover-bytes refusal for every
kind). Nothing in the behaviour changes in this change. LLR-js9dsu is kept
where it is, with its dated notes; its tests stay annotated with its ID, and
the new test of this item names both (`wire_frame_bound.rs`'s
`exactly_three_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused`).
No `satisfies:` or `traces:` line names LLR-js9dsu: SDD-kwncn7's `traces:`
names requirements only, so no pointer moves.
