# Problems — found while developing change `worktree-org-io-domain-api`

Recorded 2026-10-24 during the develop phase of change
`worktree-org-io-domain-api` (anomaly noted at plan task T37). The file is a
DRAFT: the merging change dates it and resolves the items it fixes.

**PR-8ghk9s**: when the founding write's `create_proxy` does not return within `WRITE_TIMEOUT`, org-io records an unknown write for the founding with no proxy stored, which verification can never settle (a founding with no proxy is `NoProxyYet`, so the settlement is always `StillUnknown`), so every later write for that founding is refused as `OutcomeUnknown` on that handle until a restart, after which a retry may create a second proxy.
affects: REQ-bm4ppg, LLR-d3farr, LLR-sq5pbq
opened: 2026-10-24
status: resolved
resolution: fixed 2026-10-29 (change `worktree-org-io-domain-api`, found by
plan task T37, fixed in task commit 9ff88b2, merged 2026-10-24; owner
decision 2026-10-24): root cause, `write_founding` treated a `create_proxy`
timeout like any writer timeout and recorded an unknown write for a
founding with no stored proxy, which verification can only settle as still
unknown; now a `create_proxy` that fails or times out stores and records
nothing and is reported `WriteReport::Refused` (`WriteFailure::CreateProxyUnanswered`
for the timeout, `org-io/src/write.rs`), so the next write creates the
proxy again, while a `found_at` timeout after the proxy is stored still
records the unknown write (REQ-bm4ppg, LLR-d3farr, LLR-sq5pbq amended in
place). Reproduced by `org-io/tests/chain_write.rs`
`a_create_proxy_timeout_records_no_unknown_write_and_the_next_write_creates_again`
(verifies: LLR-sq5pbq, LLR-d3farr, PR-8ghk9s), red before ("not refused:
Unknown { … the write timed out after 90 s … }"), green after; the
unchanged `found_at` case by `a_foundings_unknown_outcome_refuses_a_second_write_of_that_founding_only`
(verifies: LLR-mhn4n5); both pass in org-io's verify line.

Where: `write_founding` in `org-io/src/write.rs` (plan task T34) inserts the
unknown write on `create_proxy`'s timeout, as LLR-d3farr states for every
writer call and REQ-bm4ppg for every chain write with an unknown outcome;
LLR-sq5pbq covers `create_proxy`'s failure but not its timeout. LLR-zey4mg
(a founding's unknown write is settled only as `Adopted` or `StillUnknown`)
and LLR-4qyeut (`NoProxyYet` with no proxy stored) are right on their own;
together with the recorded write they make the trap. LLR-mhn4n5's refusal is
right for a recorded unknown write.

Owner decision 2026-10-24: a `create_proxy` that times out, or otherwise
fails before a proxy address is known, records no unknown write: nothing is
stored, and the next write for that founding calls `create_proxy` again. The
accepted cost: if the timed-out call did execute, an orphan pure proxy holds
only its proxy deposit (funding happens only in `found_at`), the cost already
assessed for a retried founding. A `found_at` timeout after the proxy is
stored is unchanged (unknown write recorded, proxy reused). Reproduced by
`a_create_proxy_timeout_records_no_unknown_write_and_the_next_write_creates_again`
(`org-io/tests/chain_write.rs`).

**PR-yng83g**: when the deletion of a removed Persona that `OrgIo::open` makes (its removal past its deadline, or keeping no acknowledgement) fails at `mark_removed`, the Persona stays in the service with its file and keychain item and is tried again only at the next open: `start_receiving` resumes no removal past its deadline and starts no hourly retry for it, so the Device secret can outlive the deadline for the whole run of the handle, where LLR-3fxxpp states that the deletion is tried again one hour later while the handle runs.
affects: LLR-3fxxpp, LLR-upqj5g
opened: 2026-10-25
status: resolved
resolution: fixed 2026-10-29 (change `worktree-org-io-domain-api`, the
io-followup after plan task T44, commit 1bd8717, merged 2026-10-25): root cause, `resume` started work only for a removal
whose deadline had not passed, and the hourly leftover retry covers only
`removed-` files, so a removal past its deadline whose deletion failed at
open had no retry while the handle ran; now `resume`
(`org-io/src/acknowledge.rs`) starts an hourly deletion retry
(`retry_deletion`, `org-io/src/retention.rs`) for such a removal, with no
delivery rounds, started once and aborted with the other tasks, as
LLR-3fxxpp already stated (not amended). Reproduced by
`org-io/tests/acknowledgements.rs`
`a_failed_deletion_at_open_is_retried_hourly_once_receiving_starts`
(verifies: LLR-3fxxpp, LLR-upqj5g), red before (the wait elapsed with
nothing retried), green after; it passes in org-io's verify line.

Where: `delete_expired_at_open` in `org-io/src/acknowledge.rs` (plan task T44)
records the failure for the report and leaves the Persona held; `resume`
(called by `start_receiving` in `org-io/src/receiver.rs`) starts a delivery
task only for a removal whose deadline the clock has not reached, as
LLR-5rxqr8 states, and the hourly retry T44 added (`start_leftover_retry`)
covers only `removed-` files, which a failed rename never leaves. LLR-5rxqr8
is right on its own (no delivery rounds past the deadline); the retry
LLR-3fxxpp states for every failed `mark_removed`, open's included
(LLR-upqj5g: a deletion that fails there does not refuse the open), was not
started for open's failures. Noted at T44. Reproduced by
`a_failed_deletion_at_open_is_retried_hourly_once_receiving_starts`
(`org-io/tests/acknowledgements.rs`).

**PR-wch3zu**: `finish_leftovers` (at `OrgIo::open` and in the hourly retry) finishes every `removed-<key>.store` it finds, deleting that Device key's keychain item, even when a live `persona-<key>.store` for the same key is also in the data directory, so the Persona that store holds loses its Device secret while its store is live.
affects: LLR-upqj5g, LLR-3fxxpp, RC-hd5des
opened: 2026-10-25
status: resolved
resolution: fixed 2026-10-29 (change `worktree-org-io-domain-api`, found in
the io-followup review, plan task T60 item E4, commit 4c25e49, merged
2026-10-25): root cause, `finish_leftovers` never checked for a live store
of the leftover's key; it now skips such a leftover, leaving both files and
the keychain item, and `OrgIo::open` reports it under the Persona's
Organisation as `DeletionFailure::LeftoverKept` (`kept_leftover_failures`,
`org-io/src/retention.rs`), as LLR-upqj5g states (amended in place
2026-10-25). Reproduced by `org-io/tests/acknowledgements.rs`
`a_removed_file_beside_a_live_store_of_its_key_is_kept_and_reported`
(verifies: LLR-upqj5g, LLR-3fxxpp), red before (the open failed
`KeychainItemMissing`: the live Persona's item had been deleted), green
after; it passes in org-io's verify line.

Where: `finish_leftovers` in `org-io/src/acknowledge.rs`. A real deletion
renames the store file to `removed-<key>.store` before it deletes the item,
so both files never exist together from org-io's own steps; a restored
backup or a copied directory can hold both. RC-hd5des's rule that a
Persona's keychain item is never deleted while its store is live is then
broken by the leftover's finish. Found in the io-followup review (plan T60,
item E4). Defensive fix: a leftover whose `persona-` file exists is skipped,
kept untouched and reported.

Resolution (2026-10-25, plan T60): root cause, `finish_leftovers` never
checked for a live store of the leftover's key; it now skips such a
leftover, leaving both files and the item, and `open` reports it under the
Persona's Organisation (`kept_leftover_failures`). Reproduced by
`a_removed_file_beside_a_live_store_of_its_key_is_kept_and_reported`
(`org-io/tests/acknowledgements.rs`). LLR-upqj5g's text ("finishes every
deletion a `removed-` file records") does not yet state the skip.

*Note, 2026-10-30 (merge review round 1, finding-3):* the last sentence
above is stale. LLR-upqj5g was amended in place on 2026-10-25 (PR-wch3zu,
plan task T60) to state the skip: a leftover whose key still has a live
`persona-` store file is left untouched and reported. `finish_leftovers`
has since moved from `org-io/src/acknowledge.rs` to
`org-io/src/retention.rs`. The `resolution:` line above records the fix in
this file's form; the paragraph above is kept as history.

**PR-bbu24f**: guardrails tooling: `.guardrails/scripts/check-trace.sh` counts the `expects:` line of a superseded (out-of-force) REQ as an open expectation, so it prints `UNMET-EXPECTATION org-node` for org-io's superseded REQ-tzr3dk, REQ-jxyh9c and REQ-txkm94, whose in-force successors carry the expectation org-node answers, and no edit to org-io's docs that keeps the superseded items' history can clear the advisories.
affects: REQ-tzr3dk, REQ-jxyh9c, REQ-txkm94
opened: 2026-10-30
status: open

Where: the expectation loop of `.guardrails/scripts/check-trace.sh` (the
scoped-run block "Unit annotations and expectations", the `for _el in …`
loop over `own_scan`'s `EXPECTS` records). It filters the provider's
answering REQs by force (`oof_scan` over the provider's SRS and SAD; D3 as
extended on review round 2, finding-2), but not the expecting unit's own
REQs: every `EXPECTS` record of `own_scan` is judged, a superseded or
retired one included, so an out-of-force item's `expects:` is reported as
open and counts toward `exp_open`, and so toward `EXPECTATION-BACKLOG`
(`expectation_open_max: 10`) and the age limit (`expectation_age_days: 90`).
Found in the merge review of change `worktree-org-io-domain-api`, round 1
(org-io record finding-5, plan task m3), whose dated notes on the three
REQs say why the advisories stand; opened at merge review round 2 (plan
task o6).

Expected: an out-of-force REQ's `expects:` is history, not an open
expectation: the loop skips an `_eid` the unit's own out-of-force scan
lists, as it already discounts an out-of-force provider REQ. The fix belongs
to the guardrails repository the scripts are copied from and reaches this
project through `ratchet`'s upgrade; org-io's docs change nothing. Until
then the three advisories print on every org-io check-trace run. They do
not fail it today: none implements a risk control, and none is past the
age limit; past 90 days open (from 2027-01-05, opened 2026-10-06) they
would fail the run although nothing is expected.

**PR-hh4xvx**: no part of the software logs the sender of received Organisation information or of an organisation answer, or shows it to a user, and the sender of a body refused because a store's provisional bound is full is not kept, so a Device that keeps flooding a store once per ahead limit (the residual risk of org-node's hazard "False provisional organisations from any Device", `org-node/docs/risk/2026-10-15-domain-api.md`) holds off an admission or an update and is not identified.
affects: LLR-x36szv, REQ-cj9u3y
opened: 2026-10-31
status: open

Where: org-io's receiver (`org-io/src/receiver.rs`) knows each body's
sender, the DevicePublicKey of the connection it arrived on, and hands it
to org-node's check. It reports the sender only for a kept body
(`Received::ProvisionalOrganisationReceived { from }`, LLR-x36szv); a
refused body is reported with no sender, and the app drops the `from` it is
given (its `Arrival::ProvisionalOrganisation` carries none).
None of org-node, org-io or the app has a logging dependency. Found in the
merge review of change `worktree-org-io-domain-api`, round 5 (finding-1):
that residual risk said the flooder's key was shown in the
logs, which was not so; the owner re-accepted the residual without that
claim on 2026-10-31 and asked for this report.

*Amended in place 2026-10-31 (merge review round 6, MR6-N1):* the
statement said no part of the software "records or shows" the sender. The
"records" clause was added by the dispatcher's task instructions, not by
the owner, and was false: org-node holds the sender of each kept body in
its origin (`Origin::Received { from }` or `Origin::Answer { from }`) until
a verification discards or adopts the body or the store itself is dropped,
and org-io reports it for each kept body. The statement and "Expected" now name the gap as logging or
showing the sender, and keeping a refused body's sender, not recording it.

Filed against org-io, not org-node: org-io is where every body's sender is
received and known, whether org-node keeps or refuses the body, and
org-node performs no IO, so a log or a report naming the sender is org-io's
to emit (and the app's to show).

Expected, as a later improvement: the sender of each
Organisation-information body and organisation answer org-io receives,
kept or refused, is logged or shown, so that the Device behind a sustained
flood can be identified. Until then the residual stands as the owner accepted it on
2026-10-31: any Device that knows the Organisation identifier can hold off
an admission indefinitely and is not identified.
