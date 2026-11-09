# app: the demo journey's domain surface

Change `worktree-org-io-domain-api`, decided 2026-10-14 in the
`grill-requirements` interview for the demo journey. What stays in the app
is the decision when, and what the user is shown and offered (owner rule:
the app decides when; org-io does the IO and the judging). org-io's items
are named by description only.

Every requirement is `satisfies: derived`.

**REQ-akeah6**: The software shall ask org-io to verify every Organisation
and provisional founding it holds when the app starts and whenever the user
asks, show the user each one's result — in step, adopted, removed, behind
with the chain's epoch, inconsistent with its reason, or unverified with its
reason — and, for an Organisation reported behind, ask org-io to request it
from its Members as REQ-vgn96n states, with no user step.
satisfies: derived

From org-io's verify requirement ("when the app starts and whenever the app
asks"): org-io verifies when it is asked, and this is the asking. The app
passes org-io's result through and adds no verdict of its own.

*Amended in place 2026-10-16 (unmerged draft; owner ruling Q2 of
2026-10-16):* the offer to request a behind Organisation becomes automatic,
and verification also runs every interval while the app runs (REQ-5q7nyg).

**REQ-5q7nyg**: The software shall, while it runs, ask org-io to verify
every Organisation and provisional founding it holds once every
verification interval, besides when it starts and whenever the user asks,
and show each result as REQ-akeah6 states.
implements: RC-c5tpre
satisfies: derived

**REQ-vgn96n**: The software shall, when org-io's verification reports an
Organisation behind, ask org-io to request that Organisation from its
Members with no user step, and ask again at each verification interval until
a verification reports it in step or adopted.
implements: RC-c5tpre
satisfies: derived

**REQ-ergn9z**: The software shall read the verification interval from its
configuration when it starts, using 10 minutes when none is configured, and
shall refuse to start, with a typed error naming the verification interval,
when the configured value does not parse or is zero.
satisfies: derived

Owner ruling Q2 and answer A1, 2026-10-16: periodic verification, with an
interval that is configurable and defaults to 10 minutes; the interval is
parsed at the system edge, as every configured value is.

**REQ-88npkt**: The software shall, before asking org-io to propose an
admission, show the Admin the invitee name they typed for that Invite beside
the admission request's name, surname and DevicePublicKey, and ask org-io to
propose it only once the Admin confirms.
implements: RC-rv4xhs
satisfies: derived

**REQ-s3vnmh**: The software shall, when org-io reports that an admission
request was refused because another Device already used the invite, show the
user a prominent error stating that and telling them to contact the inviter.
implements: RC-rv4xhs
satisfies: derived

**REQ-pe2c7u**: The software shall, when org-io reports a provisional change
it discarded because an adopted epoch left it out, show the Admin a notice
naming the Member and the kind of change until the Admin dismisses it, and
offer to propose the change again.
implements: RC-8h344c
satisfies: derived

**REQ-qv4d2z**: The software shall, when org-io's verification reports
adopted an epoch this node wrote, ask org-io once, at once and with no user
step, to send it to every Device it concerns, and show the Admin the outcome
for each Device, offering no retry.
implements: RC-c5tpre
satisfies: derived

*Amended in place 2026-10-16 (unmerged draft; owner ruling Q1 of
2026-10-16):* send once. The way to ask org-io to send again is withdrawn: a
Device the single send missed is behind at its next periodic verification
and requests the Organisation (REQ-vgn96n), and a revoked Device is answered
with its revocation notice when it asks. The control this item implements
is RC-c5tpre, which replaced the earlier control of a first send with
manual retries.

**REQ-y53dqn**: The software shall, when it starts, ask org-io for every
admission request it keeps for each Organisation it holds, and show each
as it shows an admission request org-io reports on arrival.
satisfies: derived

Owner answer A3, 2026-10-16: kept admission requests are shown at start
beside those reported as they arrive, so a request kept before a restart is
not lost from the Admin's view.

## After S4 merged

S4 (`worktree-org-io-transport`) merged into this branch on 2026-10-15; the
owner's rulings of that day (recorded in org-io's draft of this change) make
the invitee's first admission request, not an Invite reply, the step the
warning precedes, and make the app ask for verification of received
Organisation information at once (R2).

**REQ-2yh3c8**: The software shall, before asking org-io to send a
provisional Member's first admission request, show the user that nothing
has verified who sent the Invite or the Organisation name it states, and
that the request will reveal the name, surname and preferred Handles entered
to the Device the Invite names; shall ask org-io to send it only once the user
confirms; and shall ask org-io to retry that same request with no further
confirmation.
implements: RC-dfzb5q
satisfies: derived

The warning is the same information for safety as before, moved to the step
that now reveals the details; a retry reveals nothing the confirmed request
did not.

**REQ-vb5pag**: The software shall decide, from the user's commands, when to
propose a Person, a founding, a Member, an admission or a revocation, when
to propose the updated Organisation and when to write it to the chain; and
shall hand each to org-io and show the outcome org-io returns.
satisfies: derived

The single submission is split into proposing, writing, verifying and
sending (owner, 2026-10-14); sending follows verification as the app's
verification and send items state.

**REQ-k7ay5b**: The software shall, when org-io reports received
Organisation information kept as a provisional organisation, or a
revocation notice kept, ask org-io to verify that Organisation at once with
no user step, and show the result as it shows every verification result
(REQ-akeah6).
satisfies: derived

Owner ruling R2, 2026-10-15: org-io neither reads the chain nor adopts on
receipt, so the app asks for verification at once.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* a revocation notice org-io reports kept (org-io's REQ-cj9u3y,
as amended the same day) also triggers the verification at once, instead of
waiting for the next one at start, on request or at the interval; a Device
whose removal the notice records then learns of it, and deletes what it
holds (org-io's REQ-36gxmg), at once.
