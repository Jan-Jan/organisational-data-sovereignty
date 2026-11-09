# Problems found during change `worktree-org-io-domain-api`

**PR-8sgsb5**: after a provisional Member's first adoption, `addresses_for`
returns nothing for the Invite's issuer, so in Loopback mode a newly admitted
Member cannot reach the Admin (`request_organisation` reports
`NotDelivered { NoLoopbackAddress }`).
affects: LLR-q4puca, LLR-h9d65p
opened: 2026-10-25
status: resolved
resolution: fixed 2026-10-29 (change `worktree-org-io-domain-api`, found by
plan task T41, fixed in commits ff22e09 and a144777, merged 2026-10-25;
dispatcher decision after owner answer A2): root cause, `adopt_first`
(`org-node/src/verify_org.rs`) removed the provisional Member, the only
holder of the Invite issuer's DevicePublicKey and addresses, without moving
them into the address book, so `addresses_for` had no entry for the Admin;
now the first adoption sets the issuer's addresses in the address book in
the adoption's own sealed change (LLR-q4puca, LLR-h9d65p, both amended in
place). Reproduced by `org-node/tests/verification.rs`
`a_first_adoption_keeps_the_issuers_addresses_in_the_address_book`
(verifies: LLR-q4puca, LLR-h9d65p), red before (the address book was
empty), green after; it passes in org-node's verify line, and org-io's
receive and acknowledgement tests, adjusted test-only as below, pass in
org-io's.

Found by plan task T41 (`org-io/tests/receive_step.rs` had to keep the
Admin's addresses by hand). A provisional Member's store holds the Invite's
issuer DevicePublicKey and addresses, and `addresses_for` (LLR-pgy2rb) falls
back to them only while the store is provisional; `adopt_first` removes the
provisional Member without moving them into the address book, so the
receiver's pull model (periodic verification and an organisation request
when behind, owner rulings Q2 and A2) cannot reach the Admin. LLR-q4puca is
silent on the issuer's addresses and LLR-h9d65p says no operation but an
organisation request and an admitted request's adoption writes the address
book; LLR-pgy2rb's lookup order is right as written. Fix (dispatcher
decision, after owner answer A2): the first adoption moves the issuer's key
and addresses into the address book in the adoption's own sealed change.
Reproduced by `org-node/tests/verification.rs`
`a_first_adoption_keeps_the_issuers_addresses_in_the_address_book`.

Fallout in org-io's tests, fixed test-only in the same task: with the
issuer's addresses kept, `org-io/tests/receive_step.rs`'s hand-kept Admin
addresses (`keep_admin_addresses`) found the entry already set and panicked,
so they are removed; and the removed Member's acknowledgement delivery task
now reaches the Admin by itself, racing the hand-dialled acknowledgements in
`org-io/tests/acknowledgement_inbound.rs` and three acknowledgement tests of
`receive_step.rs`. Those tests now keep dead Admin addresses in the removed
Member's store before its removal (by org-node's check), and
`a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted`
drives the real delivery task instead of a hand dial. Every network wait in
them has a time limit.

**PR-ug8q68**: any Device the current record lists can fill an
Organisation's 1 MiB bound with revocation notices, after which the store
can keep nothing more and can never adopt again: it stays behind for good.
affects: LLR-r4qtqe, LLR-f9mcsq, LLR-m76r4k, LLR-x8drb8
opened: 2026-10-30
status: open

Found by merge review MR1 of change `worktree-org-io-domain-api` (org-node
finding-2, low; a gap in the requirements, not a code defect). `check_received`
keeps every revocation notice `revocation::check_notice` passes
(`org-node/src/service.rs`, the `Revocation` arm; LLR-r4qtqe), and
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
(`org-node/src/service.rs`) rebuilds the base from `record.trie_members`
and runs the chain-free checks against it, but keeps the carried snapshot
as `base_snapshot: Some(record_snapshot)` without comparing it with that
base, as LLR-76bhw9 states. On adoption it becomes the record's
`kept_base_snapshot` (`org-node/src/verify_org.rs`, `adopt`), which
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
(`org-node/src/service.rs`) maps a postcard encode failure to
`OrgNodeError::Chain(format!("record snapshot encode: {error}"))`. It is
called when answering an organisation request (LLR-7kdhst) and when
proposing the updated Organisation; neither reads the chain, so the app
would show a chain-read failure for a local encode failure. Encoding the
member snapshots fails only on an allocation failure, so it is rare. Fix: a
variant of its own for a local encode failure.

**PR-y6ay44**: `Acknowledgement::signed_bytes` falls back to `ACK_DOMAIN`
alone if postcard fails to encode the tuple, so a signature made then would
cover no Organisation, Member, Device, epoch or root.
affects: LLR-bs6yxf
opened: 2026-10-30
status: resolved
resolution: fixed 2026-11-04 (change `worktree-org-io-publish-2`, plan task
T18, owner ruling Q4 of 2026-11-03) by the fix outline's second option:
`signed_bytes` appends the domain, then each fixed-size field's bytes and
the epoch's LEB128 varint written by hand, which is the tuple's postcard
encoding with no step that can fail, so the `pub` signature and LLR-bs6yxf
stay as written. Reproduced first by
`revocation.rs::signed_bytes_are_the_domain_then_the_tuple_with_no_fallback`
(`org-node/tests/revocation.rs`, `verifies: LLR-bs6yxf, PR-y6ay44`), red on
its scan for the fallback before the fix; its first assertion holds the
hand-written bytes equal to postcard's at a two-byte epoch.
*Note 2026-11-04 (conformance round 1, T1-N2):* the source scan is dropped;
the test is now
`revocation.rs::signed_bytes_are_the_domain_then_the_tuple_at_each_varint_length`,
which holds the same equality at epochs 127, 128, 300 and `u64::MAX`.

Found by merge review round 3 of change `worktree-org-io-domain-api`
(org-node finding-6, low). `signed_bytes` (`org-node/src/revocation.rs`)
extends the domain bytes only `if let Ok(tuple) = postcard::to_allocvec(..)`
and otherwise returns the domain bytes alone, which LLR-bs6yxf does not
allow. The state is contrived: the tuple holds fixed-size byte arrays and
an integer, which postcard's allocating encoder cannot fail on short of an
allocation failure. It is not fixed here because removing the fallback
changes the `pub` signature to a `Result` and needs a refusal variant that
`sign_acknowledgement` and `check_acknowledgement` pass on, an LLR change
beyond a round-3 low. Fix: return `Result<Vec<u8>, OrgNodeError>`, or
encode the tuple's fields by hand so no failure path exists.

**PR-7ud5fv**: a failed write of a removal leaves the Persona Active with no
behind marker until the next verification, even when the store held one
before the removal, so recording and proposing are not refused as behind in
that window and an organisation answer is refused as not requested.
affects: LLR-zk7fx2
opened: 2026-10-31
status: open

Found by merge review round 7 of change `worktree-org-io-domain-api` (code,
low). `seal_removal` (`org-node/src/service.rs`) replaces the Active store's
content with the removed form before org-io writes it, so the behind marker
goes with it; when that write fails, org-io puts the store back as its file
holds it, and `restore_store` copies a marker only from a held Active store,
so the Persona returns with none. Round 5 closed the same window for a
`Behind` verification's failed write. It needs a failed disk write, and the
next verification repeats the removal and sets the marker again if the
Organisation is still behind. Fix: keep the Active store's marker until the
removal's write succeeds, and restore it on the failure.
