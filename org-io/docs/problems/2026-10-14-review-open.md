# Open problems found by the S4 review, round 1

Opened 2026-10-14 by the fix of the S4 review, round 1
(`docs/plans/2026-10-06-org-io-transport.md`, merge-change step 6a). Neither
is fixed by S4: each is recorded here so that it is not lost.

**PR-7hu3v5**: an invite stays outstanding after an admission whose chain
write timed out and later committed: `OrgIo::admit_reply` returns at the
`submit_commit_send` error, before it settles the reply's invite identifier,
and the late commit lands through `settle_unknown` or `reconcile`, neither of
which settles the invite; re-admitting the same reply is then refused
(`DuplicateKey`), so the pair cannot be settled through the exchange.
affects: REQ-prjja8, LLR-y4dg74, LLR-t98pk9, LLR-k5pzsz
opened: 2026-10-14
status: open

Review round 1, finding 5 (`org-io/src/exchange.rs`, the `admit_reply`
return at the submission's error; `org-io/src/publish.rs`, the late-commit
paths of `settle_unknown` and `reconcile`). It needs a write whose outcome is
unknown; nothing is lost and no one is admitted twice, but the invite stays
outstanding in the inviting Persona's store, contrary to REQ-prjja8's
settle-on-admission.

**PR-m3tfnu**: a received update that was declined because this node's own
update for the same Organisation was still provisional (the node behind the
chain) is never offered again: the receive step checks it against the
current record as designed (owner's explanation of 2026-10-14) and declines
it, and once the node's own update is promoted nothing re-offers or re-reads
the declined update, so the node stays behind until another update arrives.
affects: LLR-cvyyr8, SDD-aw3nka
opened: 2026-10-14
status: open

Raised by task T22's report (plan, the owner point the review's finding 4
names): a node behind the chain refuses (`DeltaBaseMismatch`) a valid update
built on its own pending epoch, and nothing recovers it. The decline itself
is correct; what is missing is catch-up. Stage S3b-io part 2 may address its
delivery (re-sending, or reading the chain for the missed update); this
report stays open until a change does.
