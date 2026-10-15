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
