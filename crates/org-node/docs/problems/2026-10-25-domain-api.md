# Problems found during change `worktree-org-io-domain-api`

**PR-ug8q68**: any Device the current record lists can fill an
Organisation's 1 MiB bound with revocation notices, after which the store
can keep nothing more and can never adopt again: it stays behind for good.
affects: LLR-gb7p7c, LLR-f9mcsq, LLR-m76r4k, LLR-x8drb8
opened: 2026-10-30
status: open

Found by merge review MR1 of change `worktree-org-io-domain-api` (org-node
finding-2, low; a gap in the requirements, not a code defect). `check_received`
keeps every revocation notice `revocation::check_notice` passes
(`crates/org-node/src/service.rs`, the `Revocation` arm; an earlier form of LLR-gb7p7c), and
`check_notice` judges identity alone (Organisation, MemberId, Device), not
the proof, because receipt reads no chain. Kept notices are never
de-duplicated and count against the bound shared with provisional changes
and organisations (LLR-m76r4k); a notice `revocation::accept` refuses stays
kept (LLR-f9mcsq), and only an adoption clears them (LLR-x8drb8). So one
listed Device, malicious or faulty, can send notices naming it with bogus
proofs until `ProvisionalLimit`. From then on every keep of Organisation
information or an answer is refused, and so are `record_admission`,
`record_revocation` and `propose_updated_organisation`; with no provisional
organisation to match, verification never adopts again. For risk analysis:
at least refuse a notice identical to one already kept, or bound kept
notices apart from provisional organisations.

**PR-d4r6te**: Organisation information received in a store with a current
record keeps the `record_snapshot` it carried as its `base_snapshot`
without checking that the snapshot is the record it extends; after adoption
it is re-sent and revocation notices are built from it.
affects: LLR-76bhw9, LLR-p8vpgz, LLR-degg8t
opened: 2026-10-30
status: open

Found by merge review MR1 of change `worktree-org-io-domain-api` (org-node
finding-3, low). In a store with a record, `keep_information`
(`crates/org-node/src/service.rs`) rebuilds the base from `record.trie_members`
and runs the chain-free checks against it, but keeps the carried snapshot
as `base_snapshot: Some(record_snapshot)` without comparing it with that
base, as LLR-76bhw9 states. On adoption it becomes the record's
`kept_base_snapshot` (`crates/org-node/src/verify_org.rs`, `adopt`), which
`messages_for_current_epoch` re-sends and builds revocation notices from. A
wrong snapshot gives wrong or missing notices, or a refusal of the whole
send. Latent today: org-io and the app send only an own adoption, whose
snapshot org-node built itself. Cheap fix: check that the carried snapshot
rebuilds the record's root, or keep the record's own members instead.

**PR-xkp5bv**: a failure to encode a record snapshot is reported as
`OrgNodeError::Chain`, whose message reads "chain read failed: …", though
no chain is involved.
affects: LLR-7kdhst
opened: 2026-10-30
status: open

Found by merge review round 2 of change `worktree-org-io-domain-api`
(org-node finding-5, low). `encode_record_snapshot`
(`crates/org-node/src/service.rs`) maps a postcard encode failure to
`OrgNodeError::Chain(format!("record snapshot encode: {error}"))`. It is
called when answering an organisation request (LLR-7kdhst) and when
proposing the updated Organisation; neither reads the chain, so the app
would show a chain-read failure for a local encode failure. Encoding the
member snapshots fails only on an allocation failure, so it is rare. Fix: a
variant of its own for a local encode failure.

**PR-7ud5fv**: a failed write of a removal leaves the Persona Active with no
behind marker until the next verification, even when the store held one
before the removal, so recording and proposing are not refused as behind in
that window and an organisation answer is refused as not requested.
affects: LLR-zk7fx2
opened: 2026-10-31
status: open

Found by merge review round 7 of change `worktree-org-io-domain-api` (code,
low). `seal_removal` (`crates/org-node/src/service.rs`) replaces the Active store's
content with the removed form before org-io writes it, so the behind marker
goes with it; when that write fails, org-io puts the store back as its file
holds it, and `restore_store` copies a marker only from a held Active store,
so the Persona returns with none. Round 5 closed the same window for a
`Behind` verification's failed write. It needs a failed disk write, and the
next verification repeats the removal and sets the marker again if the
Organisation is still behind. Fix: keep the Active store's marker until the
removal's write succeeds, and restore it on the failure.
