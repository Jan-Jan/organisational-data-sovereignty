# app: risks of the demo journey's admission over iroh

Change `worktree-org-io-domain-api`, analysed 2026-10-15 with the owner
(`analyze-risks`). org-node records the same hazard from its side (an
impostor's admission request kept for an invite); this item is the app's
part of it: what the Admin is shown before admitting, and what the invitee is
shown when their invite was used by another Device.

**HAZ-ycntg7**: the admission request the Admin admits may come from an
impostor who saw the pasted Invite; the Admin proposes the admission without
comparing the request with the invitee they named, or the invitee whose
invite another Device used is not told and does not alert the Admin; the
impostor becomes a Member and can read the Organisation's data.
Severity: S3. Probability: P2.

**RC-rv4xhs**: before proposing an admission the app shows the Admin the
invitee name they typed for the Invite beside the request's name, surname and
DevicePublicKey, and proposes it only once the Admin confirms; and when the
invitee's admission request is refused because another Device used the
invite, the app shows the invitee a prominent error telling them to contact
the inviter. mitigates: HAZ-ycntg7

What this control breaks: nothing; it adds one confirmation per admission and
one error the invitee only sees when something is wrong.

**Residual risk.** S3 / P2: an impostor who uses the invitee's real name and
is first is caught only if the invitee notices the error and tells the
Admin. Not acceptable under the matrix; **accepted by the owner
(2026-10-15)** for the demo stage, with reducing the probability as future
work (org-node's ledger records the same acceptance).

## The warning before the first admission request

**RC-dfzb5q**: before org-io sends a provisional Member's first admission
request, the app tells the user that nothing has verified who sent the
Invite or the Organisation name it states, and that the request reveals the
name, surname and preferred Handles entered to the Device the Invite names, and has
it sent only once the user confirms; a retry of that same request asks
nothing again. mitigates: HAZ-qfb95k
supersedes: RC-wzb48r

Reworded 2026-10-15 (change `worktree-org-io-domain-api`): the Invite reply
Blob is gone, and the admission request over iroh now carries what the reply
carried (owner ruling of 2026-10-14, "Due once S4 merges": confirm before the
first admission request and not before its retries). The disclosure is the
same, apart from preferred Handles taking the place of the one handle, and so
is the moment: before anything leaves the device. A retry discloses nothing
new to the same Devices, so not asking again leaves the control's effect as
it was. It remains information for safety, chosen knowingly by the owner
(2026-10-05). The residual is as RC-wzb48r recorded it.

## A discarded change the Admin is not shown

**HAZ-x4k8tt**: org-io reports each provisional change an adopted epoch left
out and discarded; the app does not show the report, or shows it where the
Admin misses it, after another Admin's epoch was adopted without the Admin's
revocation; the Admin believes the Member revoked, and the Member keeps the
Organisation private key and every grant made to them.
Severity: S3. Probability: P1.

P1 because the demo has a single Admin (org-node's ledger records the same
hazard from its side and the same owner ruling).

**RC-8h344c**: the app shows the Admin each discarded provisional change
prominently, naming the Member and the kind of change, until the Admin
dismisses it, and offers to propose it again. mitigates: HAZ-x4k8tt

What this control breaks: nothing; it adds a notice that appears only when
another Admin's epoch left a change out.

**Residual risk.** S3 / P1: an Admin who dismisses the notice without acting
leaves the Member with access. Not acceptable under the matrix; **accepted by
the owner (2026-10-15)** while the demo has a single Admin.

## An adopted epoch nobody sends

**HAZ-49x93v**: writing, verifying and sending an Organisation are separate
steps the app calls; the app adopts an epoch this node wrote that removes a
Member and never asks org-io to send it, for instance because the Admin
closes the app; the removed Device is never told, never deletes what it
holds, and a stolen Device keeps the Organisation's local data and goes on
showing itself a Member. Severity: S2. Probability: P2.

S2 because the removed Device still never receives the new epoch's
Organisation private key, so it reads nothing granted after the removal; what
it keeps is what it already held.

**RC-526ayz**: when verification reports adopted an epoch this node wrote,
the app asks org-io to send it at once, with no user step, and shows the
Admin each Device whose send failed with a way to send to it again.
mitigates: HAZ-49x93v
superseded-by: RC-c5tpre

What this control breaks: nothing the journey relies on; the first send
happens without the Admin, and only retries are manual.

**Residual risk.** S2 / P1 — acceptable under the matrix: a removed Device
that is offline throughout, and to which the Admin never retries, stays
untold.

**RC-c5tpre**: when verification reports adopted an epoch this node wrote,
the app asks org-io once, with no user step, to send it to every Device it
concerns, and shows the Admin each Device's outcome; while it runs, the app
asks org-io to verify every Organisation it holds every verification
interval, as well as at start and on request, and when a verification
reports an Organisation behind, asks org-io to request it from its Members
with no user step, again each interval until it is in step or adopted —
a request from a revoked Device being answered with its revocation notice,
as org-io's ledger states. mitigates: HAZ-49x93v
supersedes: RC-526ayz

Re-assessed 2026-10-16 (owner rulings Q1, Q2 and Q5, change
`worktree-org-io-domain-api`): the Admin's manual retries are replaced by
the pull. The send is attempted once; every Device the epoch concerns runs
the same periodic verification, so one the send missed finds its
Organisation behind at its start or within one interval while it runs, and
requests it; a removed Device is answered with its revocation notice, which
it checks against the chain's root (org-io's and org-node's ledgers).

What this control breaks: the Admin can no longer resend to one Device by
hand; nothing the journey relies on, since the pull reaches every Device
that runs and can reach a Member.

**Residual risk.** S2 / P1 — acceptable under the matrix. A Device that
missed the single send is behind and pulls within one interval of running;
a revoked Device gets the notice when it asks. What remains is a removed
Device that never runs again while able to reach a Member Device — a
stolen Device kept offline — which keeps what it already held and, as
before, never the new epoch's Organisation private key. This is no worse
than RC-526ayz's residual, where such a Device also stayed untold, and it
no longer depends on the Admin retrying.

*Realised 2026-10-20 (the app's architecture of this change,
`app/docs/architecture/2026-10-16-domain-api.md`):*
through REQ-qv4d2z, REQ-5q7nyg and REQ-vgn96n, by these low-level
requirements — the single send of an own adoption, with no user step
(LLR-es49wp), its per-Device report (LLR-rywe8f) shown for every Device
with no retry (LLR-fsk4g2); the verification after an included write
(LLR-k4rhbs); the periodic run every interval (LLR-3ampqm) announced as
every verification is (LLR-2267hj); the request of a behind Organisation by
the body every verification runs (LLR-nkg8sb), repeated at each run while
it stays behind and stopped once in step or adopted (LLR-fe7umz), with no
action offered to the user (LLR-zkpsw5); and the interval's default and its
refusal at start (LLR-2dpn52, LLR-rvrc25), so the pull can be neither
disabled nor made to spin by configuration. The answer to a revoked
Device's request is org-io's and org-node's.

## Derived requirements assessment

The batching, the write–verify–send split, the automatic discard and the
admission request are org-io's and org-node's derived items, assessed in
their ledgers; the app's items below follow from them.

### Verification at start and on request

No new hazard: the app passes org-io's verification result through, never
shows an Organisation verified that org-io did not report adopted or in step,
and offers to request the Organisation only when org-io reports it behind.
assesses: REQ-akeah6

*Re-assessed 2026-10-16 (owner ruling Q2):* the request for a behind
Organisation is now made with no user step (REQ-vgn96n). A request asks
Member Devices for what the requester already held as a Member, and
anything answered is adopted only by verification, so asking without the
user adds no hazard.

### Periodic verification, the automatic pull and its interval

Controls of HAZ-49x93v (RC-c5tpre). REQ-ergn9z parses the interval at the
system edge: a value that does not parse, or zero, stops the app with a
typed error naming it, so a mistyped interval can neither disable the pull
nor make it spin; the default of 10 minutes bounds how long a Device that
missed the single send stays untold while it runs. No new hazard.
assesses: REQ-5q7nyg, REQ-vgn96n, REQ-ergn9z

### Kept admission requests shown at start

No new hazard: the app shows what org-io kept, changing nothing, and an
admission is still proposed only after the Admin compares the request with
the invitee name they typed (RC-rv4xhs).
assesses: REQ-y53dqn

### What the Admin and the invitee are shown

Controls of HAZ-ycntg7, HAZ-x4k8tt and HAZ-49x93v above.
assesses: REQ-88npkt, REQ-s3vnmh, REQ-pe2c7u, REQ-qv4d2z

### After S4: the warning, the decisions and verifying on receipt

REQ-2yh3c8 keeps RC-wzb48r in force for HAZ-qfb95k: the user is told,
before anything is revealed, that neither the Invite's sender nor its
Organisation name is verified and what the first admission request reveals,
and the request is sent only once they confirm; a retry of that same request
reveals nothing more, so it needs no second confirmation. REQ-vb5pag makes
the same kind of decisions at the user's commands as the item it supersedes;
the split into proposing, writing, verifying and sending is assessed in
org-io's and org-node's ledgers, and the send of an adopted epoch is
HAZ-49x93v's control. REQ-k7ay5b asks for verification at once and shows its
result as every verification is shown: received Organisation information is
adopted only by verification (owner ruling R2), which strengthens org-node's
control on false provisional organisations, and the app shows nothing as
current that verification did not adopt. No new hazard.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20; REQ-k7ay5b as amended):* a revocation notice org-io reports kept
now also triggers the verification at once. For a removed Device this
shortens the window in which it still holds the Organisation's data: it
deletes when that verification accepts the notice against the chain, not at
the next start, request or interval (org-node's hazard of what a revoked
Device keeps, assessed in org-node's ledger, is only helped). The verification adopts nothing it would not adopt at the
interval, and a notice that does not check against the chain deletes
nothing. No new hazard; no rating changes.
assesses: REQ-2yh3c8, REQ-vb5pag, REQ-k7ay5b

### The architecture's derived low-level requirements

Written 2026-10-15 with the app's architecture of this change
(`app/docs/architecture/2026-10-16-domain-api.md`).

The command surface's renames and parses. The handler list (LLR-kk8gc7)
keeps its predecessor's rule and refuses the five old names, so no stale
webview call reaches an old behaviour. The Invite read stores nothing
(LLR-b7ywy4), so reading an Invite changes no state before the user
proposes a Member. The admission's arguments (LLR-gmb4p8) still carry no
secret or key, as its predecessor's did. The member identifier's parse
(LLR-7n4hgr), the invite identifier's parse (LLR-r8jeu8) and the send
address's decode (LLR-4jggyx) refuse rather than truncate or drop, so a
malformed value addresses no other Member, invite or Device; the decode
keeps the predecessor's rules, and org-io's Loopback identity check still
refuses an address naming another Device. Cancelling an invite
(LLR-vqz4s6) only removes what org-io holds for it; an impostor's kept
request is then gone with it. No new hazard.
assesses: LLR-kk8gc7, LLR-b7ywy4, LLR-gmb4p8, LLR-7n4hgr, LLR-r8jeu8, LLR-4jggyx, LLR-vqz4s6

*Re-assessed 2026-10-20 (owner rulings Q1, Q2 and answers A2, A3 of
2026-10-16):* LLR-4jggyx is retired with the pasted address (requests carry
the sender's own addresses, kept by org-io for the Device's key and still
checked by org-io's Loopback identity check); its assessment stays as
history; the
app now decodes no address at all, which removes the malformed-address
input rather than weakening its refusal. LLR-kk8gc7's set loses
`send_organisation` and `request_organisation`, now refused like the other
old names, and gains `list_admission_requests`, which changes nothing
(LLR-y7nc6x): the only send of an adopted epoch is the automatic one, which
cannot be skipped by a webview that never calls a send, and no webview call
can make the app request or send by hand. No new hazard.

*Assessed 2026-10-30 (unmerged draft; merge review round 2, finding-5):*
the webview's "Cancel invite" (LLR-e7bbm5) is the derived half of
LLR-vqz4s6: it invokes `cancel_invite` once for the request the Admin
pressed it beside, and drops that request from the page's list only once
org-io has cancelled it; a refusal keeps the request and shows org-io's
text. It removes nothing org-io still holds and proposes nothing, so the
assessment of LLR-vqz4s6 carries over. No new hazard.
assesses: LLR-e7bbm5

Verifying after an included write. LLR-k4rhbs asks for verification as soon
as a write is included, so the adoption, and with it the send of an epoch
this node wrote (RC-526ayz, superseded 2026-10-16 by RC-c5tpre), follows the write with no user step; it
narrows HAZ-49x93v's window between writing and sending and adopts nothing
verification would not adopt at start. No new hazard.
assesses: LLR-k4rhbs

No randomness in the app. LLR-vj5s28 removes the app's own CSPRNG; org-io
draws every key, nonce and invite identifier from its own (org-io's ledger
assesses that source), so the app can no longer hand org-io a weak or
fixed rng. No new hazard.
assesses: LLR-vj5s28

An answered organisation request is announced by no event (LLR-t9rdf8).
org-io answers it as org-node decides, with no user step (org-io's
requirement); the app shows nothing that could prompt the user to act on
it, and adopts nothing from it. No new hazard.
assesses: LLR-t9rdf8

*Assessed 2026-10-30 (unmerged draft; merge review round 4, finding-8):*
the ahead limit the app gives org-io (LLR-hgrez9) is how long received
Organisation information one epoch ahead of the chain is kept unadopted
before org-node discards it. A limit shorter than two verification
intervals could discard genuine next-epoch information before a second
chain read had the chance to adopt it, so a Device admitted or updated at
that epoch would wait for a resend. Twice the interval puts at least two
periodic chain reads inside the limit whatever the interval, which
prevents it; a doubled interval the resolver accepts never overflows, and
`init` and the test constructors build the limit with one function, so
the tested limit is the one `init` gives. It discards nothing
verification would adopt and adds no input. No new hazard.
assesses: LLR-hgrez9

*Amended in place 2026-10-31 (unmerged draft; merge review round 5,
finding-4):* "the tested limit is the one `init` gives" was not so when
written: a test called the function `init` builds its settings with, not
`init`'s call of it, so passing it the doubled limit would have quadrupled
the ahead limit with every test green. `init` now hands the variables it
read to one step that decides the interval and builds the settings from
it, and a test calls that step with environment-shaped values, so the
limit tested is the one `init` connects with. Untested remain `init`'s
reading of the variables and its call of `OrgIo::connect`. The assessment
is otherwise unchanged. No new hazard.
