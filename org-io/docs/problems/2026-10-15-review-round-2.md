# Problems found by the S4 review, round 2

Opened 2026-10-15 by the fix of the S4 review, round 2
(`docs/plans/2026-10-06-org-io-transport.md`, merge-change step 6a; owner
ruling of 2026-10-15: fix the medium findings and the mechanical lows, file
the larger lows as one umbrella report).

**PR-9q5ems**: `settle_unknown` reports a chain write whose outcome was unknown as not executed (`Settled::NotExecuted`, and a resubmission of its update is told to rebuild it) whenever the held record's epoch is past the epoch the write expected, although the record can be past it because that very write executed: the settle decides from the local record's epoch alone, not from the evidence of which update took the epoch after the expected one.
affects: LLR-t98pk9, LLR-nqvbb6, REQ-by9k5q, SDD-6mpmsw
opened: 2026-10-15
status: resolved
resolution: fixed 2026-10-15 (change `worktree-org-io-transport`, task rb): root cause, case (b) of `settle_unknown` took any record past the expected epoch as proof the write did not execute; now it settles at once only when the record is at the epoch after the expected one with another root and Organisation public key, and otherwise reads the chain, the "still at the record's epoch" cases requiring both record and chain at the expected epoch (LLR-t98pk9 and REQ-by9k5q amended); reproduced by `org-io/tests/publish_path.rs` `a_settle_whose_record_holds_its_executed_update_does_not_report_it_not_executed` and `a_settle_after_a_later_epoch_was_committed_does_not_report_its_executed_write_not_executed`.

Review round 2, supplementary finding A3 (medium; `org-io/src/publish.rs`,
the settle's case (b)). Its comment reasoned that the record "could have
moved past it only by committing it, and every commit of it org-io drives
clears the unknown write", which does not hold for a commit driven out of
the settle's sight. The investigation found the reviewer's path (a later
epoch received by the receive step) refused today: org-node's receive check
requires a received update's base root to be the held record's
(`DeltaBaseMismatch`), and no other node can build on this write's update
before it is sent, which happens only after this node commits it. The
reproductions therefore commit out of org-io's sight through org-node's
`reconcile`; the fix makes the settle independent of how the record moved.

**PR-k5tdxj**: the S4 review, round 2, left larger low-severity gaps in org-io unfixed by owner ruling (2026-10-15): keychain calls block tokio worker threads unserialised, an executed write's bodies are not delivered on several paths, an unknown write is recorded only on a timeout, the allowlist scan is shallow, and several items lack a test at their interface — each listed below.
affects: LLR-677fqq, LLR-8erfbu, REQ-by9k5q, REQ-77rh5a, LLR-k5pzsz, LLR-2yjuaj, LLR-3zdw8v, LLR-znhmh2, LLR-murjr6, LLR-b9bm36, LLR-z35vbh, LLR-t98pk9, LLR-nqvbb6, LLR-2b2ghe, SDD-v2bq89
opened: 2026-10-15
status: open

One umbrella report for the round's larger lows (owner ruling of
2026-10-15: fix the mediums and the mechanical lows, file the rest as one
org-io report). Line numbers are those of the tree after task rb's fixes.

1. **Keychain calls on the runtime's worker threads** (finding-4, against
   the `keyring` SOUP row, `org-io/docs/architecture/soup.md`, which says
   async callers must not make them there and that Windows Credential
   Manager needs one item touched by one thread at a time; keyring issue
   #163). Every keyring call runs synchronously inside async code with no
   `spawn_blocking` and nothing serialises access to one item:
   `create_persona`'s `keep` (`org-io/src/lib.rs:443`, the function at
   `org-io/src/keychain.rs:187`); the commit's and the reconcile's secret
   closures (`org-io/src/submit.rs:319`, `org-io/src/publish.rs:200`,
   through `keychain.rs:229`); the receive step's
   (`org-io/src/receiver.rs:152`, `keychain.rs:201`); the store unseal at
   open (`org-io/src/lib.rs:163`, `keychain.rs:221`); and
   `bind_from_keychain` (`org-io/src/transport/endpoint.rs:108`, called from
   `org-io/src/endpoints.rs:121` and `org-io/src/preflight.rs:90`).
2. **An executed write's bodies are not delivered** (finding-6 and A2).
   `reconcile` sends only when an in-memory unknown write is recorded
   (`org-io/src/publish.rs:367-386`), so: after a restart, the startup
   reconcile commits a write that executed but sends its bodies to no one;
   after a read-back, commit or store failure in `submit_commit_send`
   (`org-io/src/submit.rs:315-325`) no unknown write is recorded, and the
   later reconcile commits without sending; when org-node's `reconcile`
   refuses an executed unknown write (`Err(ReconcileFailed::Node)`), the
   write is cleared (`publish.rs:287`, `:302`; `:367-368` and `:386`) and
   never sent; and when a late commit removes this node, nothing is sent.
   org-node's publish-before-persist report (named in the roadmap plan;
   org-node exports none of its items to org-io) and S3b-io part 2's T16d
   (re-sending, or reading the chain for the missed update) may cover
   these; REQ-by9k5q's note that the startup
   reconcile "settles it" is clarified the same day
   (`org-io/docs/requirements/2026-10-13-publish.md`).
3. **An unknown write is recorded only on `TimedOut`** (A8; predates S4):
   `org-io/src/submit.rs:310-311` records it for `WriteNotDone::TimedOut`
   alone, so a write whose transaction was broadcast and then failed is not
   settled (compare PR-6nhrr7).
4. **The allowlist scan is shallow** (B(e)):
   `the_handle_returns_only_allowlisted_public_types`
   (`org-io/tests/absences.rs:615`, its `named_types` at `:586`) checks the
   type names a handle method returns, not the fields of the allowlisted
   types, so a private key added as a field of an allowlisted type passes.
5. **Tests missing at an item's interface** (supplementary report C):
   - LLR-znhmh2: no test places the failing recipient at a random position
     of the fan-out (`org-io/src/send.rs:151`, `send_committed`).
   - SDD-v2bq89: `plan_send` (`org-io/src/send.rs:126`) is untested at its
     interface (listed here unless task `ra` of the same fix adds it).
   - LLR-murjr6: no test covers each call site of `LocalStore::store`
     (`org-io/src/lib.rs:120`) that reloads after a failed write.
   - LLR-b9bm36: the `sync_all` and `create_new` steps
     (`org-io/src/store_file.rs:132-147`) survive their removal (no
     mutation-killing test).
   - LLR-2b2ghe: the peer-address parse is not exercised with transport
     addresses (direct and relay) in the encoded `EndpointAddr`
     (`org-io/src/transport/peer_addr.rs:70`).
   - LLR-z35vbh: the bind after `create_persona`
     (`org-io/src/lib.rs:450`, `org-io/src/endpoints.rs:149`) is observed
     only through test hooks that bind themselves, which can hide its
     absence.
   - LLR-t98pk9: no test pins the read order, the finalised block before
     the state (`org-io/src/publish.rs:253`, `:257`).
   - LLR-nqvbb6: the `Reconciled(_)` branch other than `InStep`
     (`org-io/src/submit.rs:279-280`) has no test of its report.

*Note 2026-10-16 (change `worktree-org-io-domain-api`, owner ruling Q6 of
that day).* Item 2 is to be resolved by change `worktree-org-io-domain-api`
(owner 2026-10-16): an executed write is adopted by verification, never by
an in-memory unknown write, and its bodies reach every Device it concerns
by the pull rather than by a resend: the app asks for one send of an
adopted epoch this node wrote, and every Device the send missed, after an
app crash between adoption and send included, is behind at its next
periodic verification and requests the Organisation from its Members, a
revoked Device being answered with its revocation notice; by REQ-36gxmg,
REQ-nactq6, REQ-z73ctw and REQ-wpq4fp (with REQ-bm4ppg's note on the
restart). The status
stays `open`; the resolution is written by the merging change
(`resolve-problem`). If `worktree-org-io-domain-api` cannot merge by about
2026-11-05, the owner is to be asked whether to raise org-node's
`problem_age_days`, which its report on the publish path writing the chain
before the record reaches on 2026-11-08.

*Item 2 resolved 2026-10-29 (change `worktree-org-io-domain-api`, plan
`docs/plans/2026-10-21-org-io-domain-api.md` tasks T34, T35, T38, T39,
T42, T51, T55).* Root cause: delivery hung on the in-memory unknown write
that `reconcile` cleared, so every path that committed without that record
(a restart, a failed read-back or store write, a refused node reconcile, a
late commit removing this node) sent nothing. `reconcile`, `submit.rs` and
`publish.rs` are gone: an executed write is adopted only by verification
against a finalised chain state (REQ-36gxmg, REQ-bm4ppg; LLR-t2pve7,
LLR-zey4mg), whatever happened in memory or across a restart; the app asks
once for the send of an adopted epoch this node wrote (REQ-nactq6;
LLR-pxqdy4); and every Device the send missed, after an app crash between
adoption and send included, is behind at its next periodic verification and
requests the Organisation from its Members (REQ-z73ctw; LLR-uf5cyy), which
answer with the record, or a revoked Device with its revocation notice
alone, with no user step (REQ-wpq4fp; LLR-898z24). Verified by
`org-io/tests/verification.rs`
`a_write_that_executed_before_a_restart_is_adopted_by_the_next_verification`
(verifies: LLR-t2pve7) and
`a_verification_that_adopts_an_unknown_write_reports_it_adopted_and_clears_it`
(verifies: LLR-t2pve7, LLR-zey4mg); `org-io/tests/send_fanout.rs`
`send_organisation_sends_org_nodes_messages_in_order_to_every_recipient_and_writes_no_store`
(verifies: LLR-pxqdy4, REQ-u894ty); `org-io/tests/requests.rs`
`request_organisation_asks_every_listed_device_or_the_invites_issuer`
(verifies: LLR-uf5cyy); `org-io/tests/receive_step.rs`
`an_answer_is_written_0x00_then_sent_on_a_new_connection_to_the_kept_addresses`
and `a_revoked_requester_gets_its_notice_through_the_send_path` (verifies:
LLR-898z24); and the app's half by `app/src-tauri/tests/verification.rs`
`an_own_adoption_is_sent_once_and_nothing_else_is` and
`app/src-tauri/tests/periodic.rs`
`a_behind_organisation_is_requested_at_each_run_until_in_step`; all pass in
the change's verify lines.

The report stays `open`: item 1 is not fixed (no keyring call in
`org-io/src` runs under `spawn_blocking` or behind a per-item lock), and
item 3 is not fixed (`write_update` and `write_founding` in
`org-io/src/write.rs` still record an unknown write only on
`WriteNotDone::TimedOut`; a failed `update` is reported `Refused`, though a
write that executed anyway is now adopted by the next verification).
Incidentally answered by the same change, with evidence: item 4, the
shallow allowlist scan — `org-io/tests/absences.rs`
`every_returned_type_is_in_the_handle_return_allowlist_and_holds_no_secret`
(verifies: LLR-gh4rw3) reads the body of every allowlisted org-io type and
of every org-io type it names, to a fixed point, and fails on a field
naming a secret-holding type (the scan does not descend into types org-io
does not define, so a key held as a bare byte array would still pass); and
of item 5: SDD-v2bq89 (`plan_send` gone, superseded by SDD-2bzgtp),
LLR-nqvbb6 (the `Reconciled(_)` branch gone with `submit.rs`, superseded by
LLR-mhn4n5), LLR-2b2ghe (the peer-address parse removed with
`transport/peer_addr.rs`, task T28), and LLR-t98pk9 (superseded by
LLR-zey4mg; the read order is now pinned by `org-io/tests/verification.rs`
`the_finalised_block_is_read_before_the_state`, verifies: LLR-t2pve7). Still
open in item 5: a failing recipient at a random position of the fan-out
(LLR-znhmh2, superseded by LLR-pxqdy4, whose tests place none), LLR-murjr6
(each reloading call site of `LocalStore::store`), LLR-b9bm36 (no
mutation-killing test of `sync_all` and `create_new`) and LLR-z35vbh (the
bind after `create_persona` observed through hooks that bind).
