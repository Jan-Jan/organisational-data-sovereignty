# Requirements — the member-sender rule for updates, under its own ID

Requirement for change `worktree-org-io-create` (stage S2 of
`docs/plans/2026-10-06-org-io-roadmap.md`), written to resolve PR-zf924s
(`org-node/docs/problems/2026-10-07-close-out.md`) under the owner's hybrid
ruling of 2026-10-08: a change that narrows or clarifies an item is a dated
in-place note; a change of meaning mints a new ID that supersedes the old.
REQ-ztdza4 (`2026-09-09-verify-and-commit.md`) had its meaning reversed in
place on 2026-10-07 (owner ruling R1): from "commit such an update whichever
Device key the connection authenticated" to "commit it only from a Device
the current record lists". The item below states the rule that holds now,
under a new ID.

**REQ-uk9rw7**: The software shall commit an update to an Organisation it holds
a record of only when the Device key the connection authenticated is listed
in a member snapshot of its current committed record of that Organisation and
the update verifies against the chain — a key listed there and absent from
the record after the update included — and shall refuse an update delivered
under any other Device key with a typed error naming the Organisation, before
it takes any chain state, so that none is read for it, leaving the store
unchanged.
(implements: RC-b6mydy, RC-u7kdam)
satisfies: derived
supersedes: REQ-ztdza4

*Supersession (2026-10-08, owner's hybrid ruling on PR-zf924s).* The text is
REQ-ztdza4's as amended on 2026-10-07 (owner ruling R1, decision 16 of
`docs/plans/2026-10-06-org-io-commit-workflow.md`) and on 2026-10-08 (ruling
B: org-node takes the state as a value; the order of the read is stated in
org-io's architecture ledger, `org-io/docs/architecture/`). Nothing in the
rule changes in this change. A first admission is not subject to it
(REQ-xa6smf, unchanged); the revocation half is REQ-ea4qs5. The low-level
requirements and design items that satisfied or traced REQ-ztdza4 now name
this item, each with a dated note. It is assessed in
`org-node/docs/risk/2026-10-07-commit-workflow.md` ("REQ-uk9rw7 supersedes
REQ-ztdza4").

Test: `an_update_from_an_unlisted_device_is_refused_before_any_chain_state_is_asked_for`
(`org-node/tests/receive_chain_reads.rs`), annotated with both IDs: red
under a mutation that made the member-sender check accept every sender,
green restored. The S3a tests of LLR-2r2fha (`admission_sender.rs`,
`receive_chain_reads.rs`) verify the same rule through that LLR.
