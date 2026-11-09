# Problems — found while developing change `worktree-org-io-domain-api`

Recorded 2026-10-24 during the develop phase of change
`worktree-org-io-domain-api` (anomaly noted at plan task T37). The file is a
DRAFT: the merging change dates it and resolves the items it fixes.

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
