# org-io: risks of the demo journey's domain surface

Change `worktree-org-io-domain-api`, analysed 2026-10-15 with the owner
(`analyze-risks`), for
`org-io/docs/requirements/2026-10-14-domain-api.md`.
No new hazard is minted in this unit: the hazards this change brings are in
org-node's ledger (the impostor admitted through an invite id, false
provisional organisations, a left-out revocation) and the app's (what the
Admin and the invitee are shown, and sending an adopted epoch). Per this
unit's citation rule, their items are named by description.

## Inbound connections from unlisted Devices

**RC-6m89tr**: org-io accepts an inbound connection from any Device whose
handshake completes, with no accept-time allow-list and whether the path is
direct or relayed; it hands each message to org-node's single receive path
with the connection's authenticated DevicePublicKey, so org-node's rules
decide it, and from a Device no held record lists only an acknowledgement
or an organisation request, from a Device this node holds as revoked, or an
admission request for an Invite this Device issued and holds as outstanding,
can be acted on — an organisation request from a Device no record has ever
listed is refused, and one from a revoked Device is answered with its
revocation notice alone, through the send path; and it writes on an
inbound connection only one verdict byte — accepted, refused, already
received, or refused because the invite was used by another Device — never
Organisation data, every answer to an organisation request from a listed
Device going out on a new connection through the send path.
mitigates: HAZ-abm3xs
supersedes: RC-b4m5sj

Reworded 2026-10-15 (owner rulings R2, R3, R6 and R7 of the post-S4
interview, change `worktree-org-io-domain-api`): the admission request comes
from the provisional Member's new Device, which no record lists. It is the
only new message such a Device may send, and only the Device that issued its
Invite, which expects it, acts on it. org-node judges it, as it judges the
acknowledgement. The two added verdicts tell a sender only what follows from
its own request: an impostor learns that the invite was used, which is inside
the impostor hazard org-node's ledger already analyses. No answer travels on
the inbound connection, so that connection still carries no Organisation
data.

**Residual.** S3/P1, unchanged. The owner accepted it for RC-b4m5sj on
2026-10-08 under ruling D, and on 2026-10-15 **accepted it for this
successor**, narrowed so that unlisted Devices send admission requests only,
never organisation requests, and so that the admission request is one the
receiving Device expects because it issued the invite.

*Amended in place and re-assessed 2026-10-16 (unmerged draft; owner ruling
Q3 of 2026-10-16):* a Device held as revoked may now send an organisation
request, which is answered with its revocation notice and nothing else. No
Organisation data is served: the notice carries no record, no Organisation
private key and no address, and it is what the send step already delivers
to that removed Device when the removing epoch is sent; it goes out through
the send path to the revoked Device's key, never on the inbound connection,
which still carries one verdict. The revoked Device can gain nothing from it
that HAZ-abm3xs fears. A Device no record has ever listed is still refused.
The residual is unchanged at S3/P1; owner ruling Q3 relaxes, for revoked
Devices and the notice alone, the narrowing the 2026-10-15 acceptance named.

## Derived requirements assessment

### The Person

No hazard impact: a Person holds no Organisation's data or keys, and it is
never shown as verified, so nothing can be trusted on its strength.
*Re-assessed 2026-10-22 (plan detail pass; owner ruling of 2026-10-22, one
Person per device):* the refusal of a second Person adds no hazard: the
refusal writes nothing — no keychain item, no Person file — and leaves the
Person held unchanged, so the only effect is that the user cannot hold two
Persons. The Person file now also keeps the Person key pair's secret half,
sealed under the Person's Device seed as a Persona store seals its member
seed; the Person key serves no Organisation in this change, so its
protection is the store's and is assessed with org-node's Person store.
assesses: REQ-ggjuu6

### Write, verify, send

The split changes no check. Verification adopts only what the chain holds,
at a finalised block, by org-node's judgement, which keeps its refusal of a
state behind the record; the write still writes no second provisional
organisation while one's outcome is unknown; and the provisional organisation
is never discarded while its write may still execute. One effectiveness
change: under the single submission flow the commit was always followed by
the send, and now the send is a separate step, so a revocation notice reaches
a removed Device only if the app sends it. The app sends an adopted epoch
this node wrote at once, with no user step (the app's ledger), which keeps
the existing control on a removed Device keeping what it held.
assesses: REQ-3pjmub, REQ-36gxmg, REQ-nactq6, REQ-bm4ppg

### Discarding and deleting

No new hazard: the automatic discard keeps the rule that nothing whose write
may still execute is discarded, and the deletions allowed are only those the
verified chain decides. The report of every discarded provisional change is
org-node's control on a left-out revocation.
assesses: REQ-7jh4rh, REQ-kh2uth

### The founding's proxy address

No hazard impact: it is a public chain address, stored so that a founding
interrupted before verification can still be verified; without it the cost
was a funded proxy no device could find, not a route to harm.
assesses: REQ-qfnx94

### The Organisation name

No new hazard: the name stays unverified free text, as the app's existing
hazard of an Invite from a non-Member assumes, and the warning before replying
to an Invite (carried to the first admission request) still tells the invitee
so. Keeping it locally changes only where it is shown.
assesses: REQ-4bvmwc

### Admission requests and requests for the Organisation

The admission request reveals the invitee's name, surname and preferred
Handles to the Devices the Invite names, as the reply did; the warning and
confirmation before the first request is the existing control. Its outcome
report carries the refusal org-node's first-request rule sends, which the app
shows the invitee. Requests for the Organisation and their automatic answers:
no hazard impact, for the reasons in org-node's assessment (an answer
carries only what the requester already held as a Member of that epoch).
assesses: REQ-sjqxg2, REQ-z73ctw, REQ-wpq4fp

*Re-assessed 2026-10-15 after S4 merged (owner rulings R1–R6).* Answering on
a new connection to the requester's Device key (R6) narrows who can receive
an answer to the holder of that key; nothing is served on the inbound
connection. The admission request's outcome list now matches the verdicts
the inbound side writes. No new hazard.

### Deprecated writes

Reporting a write deprecated, with the Organisation behind, when the chain
has moved to or past the expected epoch with another root (R5) removes the
false "not executed" of PR-9q5ems: a write that may have executed is no
longer reported as one that did not, so no rebuild is invited on that false
ground, and "not executed" is kept for a finalised block past the write's
window with the Organisation still at its base epoch. The provisional
organisation is still never discarded while its write may execute. No new
hazard; it removes a wrong report.
assesses: REQ-bm4ppg

### The Invite with addresses

The Invite now carries the stored Organisation name and the issuing Device's
endpoint addresses in the current mode (R4). The addresses are disclosed to
whoever sees the pasted Invite: a privacy matter, as the DevicePublicKeys
already were, with no safety harm. A send to an address that names another
endpoint identity delivers nothing, by the Loopback identity check
(REQ-8trkps), and in Networked mode a connection completes only to the
endpoint holding the dialled Device key, so a misdirected address cannot hand
the request, or later the Organisation private key, to another Device. The
name stays unverified free text, as assessed under the Organisation name.
assesses: REQ-ek652a

### Inbound connections and the receive path

REQ-xd5a2d implements RC-6m89tr, RC-b4m5sj's successor: every handshake is still accepted with
no allow-list, every message still goes to org-node's receive path with the
authenticated DevicePublicKey, a Device no record lists is still refused
everything org-node does not accept before any chain read or store write,
and the inbound connection still carries one verdict and never Organisation
data. The one message it now lets such a Device send cannot give a removed
Device what HAZ-abm3xs fears: an organisation request from an unlisted
Device is refused (owner ruling R7); an admission request is kept only by the
Device that issued its Invite, only while that invite is outstanding, and
admits nobody until the Admin proposes it and the
chain verifies it. The extended verdict tells a sender only what it already
knows about its own request; an impostor learns that the invite was used,
which adds no harm beyond org-node's analysed hazard of an impostor admitted
through an invite id. Moving the chain read out of receipt (REQ-cj9u3y,
rulings R2 and R3) means no peer can drive a chain read by sending, and
received Organisation information, an invitee's first admission included,
is adopted only through verification, which strengthens org-node's control on
false provisional organisations. The expectations restate what org-node
provides for these steps.
assesses: REQ-xd5a2d, REQ-cj9u3y, REQ-2yw5y3, REQ-u894ty, REQ-2xgsbn

### Re-assessed 2026-10-16: send once, pull, the revoked Device's request and the requesters' addresses

*Owner rulings Q1–Q3, Q6 and answers A2, A3 of 2026-10-16; dispatcher's
resolutions C3 and C7.*

Send once and pull (REQ-nactq6, REQ-z73ctw, REQ-bm4ppg). The app sends an
adopted epoch this node wrote once and no longer retries; a Device the send
missed finds itself behind at its next periodic verification and requests
the Organisation, which a listed Device answers with the full record and a
revoked Device with its notice alone (REQ-wpq4fp, REQ-xd5a2d, under
RC-6m89tr as re-assessed above). The removed Device is thus told at its start or
within one verification interval while it runs, where before it was told only if the Admin
retried; this is the app's control on an adopted epoch nobody sends. A
crash after adoption and before the send (PR-k5tdxj item 2) is closed the
same way. After a restart the unknown write's state is gone, but a second
write for the same epoch is refused by the chain's expected-epoch check, so
at most one write per epoch executes; no new hazard. A provisional Member
asks its Invite's issuer (C3), which answers only once its record lists the
requester, so nothing is served early.

Requesters' addresses (REQ-sjqxg2, REQ-z73ctw, REQ-wpq4fp, REQ-nactq6,
REQ-2xgsbn). An admission request and an organisation request disclose the
sender's endpoint addresses to the inviter or to the Members asked: a
privacy matter, as the Invite's addresses already are (R4), with no safety
harm. A send to a kept address reaches only the endpoint whose identity is
the Device key named — the Loopback identity check (REQ-8trkps) in Loopback
mode, the dialled key in Networked mode — so a false address in a request
delivers nothing to another Device. Taking no address from an inbound
connection's source removes a path a relay or a spoofed source could have
steered. No new hazard.

Verification's removed report (REQ-36gxmg). A revocation notice received in
answer is acted on only once org-node accepts it against the chain's root,
as every received revocation is; a forged notice is refused. No new hazard.
assesses: REQ-nactq6, REQ-z73ctw, REQ-bm4ppg, REQ-sjqxg2, REQ-wpq4fp, REQ-xd5a2d, REQ-2xgsbn, REQ-36gxmg

### Listing kept admission requests

No new hazard: the list changes nothing and shows the Admin only what each
request's arrival already reported; the Admin still compares it with the
invitee name they typed before proposing an admission (the app's control on
the impostor hazard), and a request kept across a restart is now shown
rather than lost from view.
assesses: REQ-wb93ch

### The founding in two steps

No new hazard: storing the proxy before founding at it, and reusing the
stored proxy on a retry (REQ-qfnx94), keeps a retried founding from creating
and funding a second proxy; on-chain-client decides what is already done
from the chain, not memory (REQ-wd683g), so a retry neither repeats a funded
transfer nor skips a step the chain does not show done. A second genesis
record at the same proxy is refused by the chain's expected-epoch check. The
proxy address is a public chain address, as already assessed.
assesses: REQ-qfnx94, REQ-wd683g

### A proxy creation that times out (PR-8ghk9s)

*Assessed 2026-10-24 (owner decision on PR-8ghk9s).* A founding's
`create_proxy` that times out, or otherwise fails before a proxy address is
known, records no unknown write and stores nothing; the next write creates
the proxy again. No new hazard: if the timed-out call did execute, the cost
is an orphan pure proxy holding only its proxy deposit, since funding happens
only when the Organisation is founded at a stored proxy, which is the cost
already assessed for a retried founding; no membership state is written, so
no Device can be misled by it. The founding is no longer blocked by an
unknown write that verification of a founding with no proxy could never
settle. A `found_at` timeout at a stored proxy is still held as unknown, so a
founding that may have executed is still never written twice.
assesses: REQ-bm4ppg, LLR-sq5pbq, LLR-d3farr

### Acting on this node's removal (owner ruling Q4')

*Assessed 2026-10-20 (owner ruling Q4' of 2026-10-16, replacing Q4).*
On a verified removal org-io writes the store org-node returns, which keeps
only the signed acknowledgements, their recipients' keys and addresses and
the verification time (REQ-36gxmg); it then sends each acknowledgement
directly to its recipient (REQ-k6kcgd), has org-node delete each one its
recipient accepts (REQ-sd7ayv), and removes the Persona's store file and
keychain item 7 days after the verification, or at once at start when that
time has passed (REQ-98a3ty); nothing else is deleted (REQ-kh2uth). The
delivery control is unchanged in kind: the acknowledgement still goes over a
direct connection with no relay under the revoked Device's own key, the
receiving Member still accepts it only from the Device it names, and its
inbound side (RC-6m89tr, HAZ-abm3xs) is untouched. What changes is the
retention around it, which narrows: at most 7 days instead of the
Organisation's retry limit, at most three recipients by the Devices known,
and each acknowledgement gone as soon as it is accepted. The exposure of
what a lost or stolen revoked Device holds in that window — the kept
acknowledgements, recipients' keys and addresses, and its own Device secret
key in the keychain — is org-node's hazard of what a revoked Device keeps to
send its acknowledgement, re-estimated in org-node's ledger, where the owner
accepted the restated residual on 2026-10-20. Deleting the store file and
keychain item as soon as no acknowledgement is left (REQ-98a3ty as amended
that day) shortens that window. The resend schedule (REQ-jqmpz5: 1, 5 and 30
minutes, then hourly) sends only what is already kept, to recipients already
kept, so it adds nothing a revoked Device holds and only repeats a message
every Member refuses unless it comes from the Device it names; it caps the
traffic at about 170 sends per recipient over the 7 days. A wrong "accepted" verdict
costs at most an undelivered acknowledgement, never Organisation data. The
deadline taken from a time kept in the store survives a restart, and a
deadline already passed is honoured at start, so a Device that stays off for
more than 7 days holds nothing once it starts. The expectation restates
what org-node provides. No new hazard in this unit.
assesses: REQ-36gxmg, REQ-kh2uth, REQ-k6kcgd, REQ-sd7ayv, REQ-98a3ty, REQ-bg5whd, REQ-jqmpz5

### Derived design item: the handle's clock (design revision of 2026-10-20)

*Assessed 2026-10-20 (design revision of change `worktree-org-io-domain-api`,
`org-io/docs/architecture/2026-10-16-domain-api.md`;
owner ruling Q4' of 2026-10-16 and the rulings of 2026-10-20).*

The clock that times the acknowledgement rounds and the 7-day deletion can
be replaced only in `test-support` builds (LLR-6vtbn6), the feature already
stated never to ship, so no release build can be given a clock that holds a
removed Device's acknowledgements, store file or keychain item past 7 days,
or deletes them early. In a release the clock is the operating system's
wall clock: a clock set back delays the deletion by as much (a user who
controls the clock controls the device, and the data are their own Device's
acknowledgements and secret, inside org-node's hazard of what a revoked
Device keeps, accepted 2026-10-20), and a clock set forward only deletes
sooner. No new hazard; no new control.

The design also stated one limit on the deletion it serves (LLR-3fxxpp, not
derived, assessed here for completeness): when the store file was removed
but the keychain refused to delete the item, a start after that found no
file naming it, so the Device secret could outlive the 7 days on that
device. *Amended in place 2026-10-20 (unmerged draft; consistency pass,
dispatcher's decision on the deletion order):* that limit is closed by the
order of the deletion (LLR-vj2nkb). The store file is first renamed
`removed-<hex>.store`, then the keychain item is deleted, then the file. A
store file left behind after the item is gone is sealed under a Device
secret that no longer exists, so it is unreadable and exposes nothing; and
whichever step fails, a `removed-` file still names the key, so at start the
deletion is finished — the item deleted if it is still there, then the file
(LLR-upqj5g, LLR-3fxxpp). No path leaves a keychain item that no file names.
Only a file the deletion marked removed is deleted at start: a Persona store
file whose keychain item is missing is still refused and left untouched, as
RC-hd5des requires, so no store that could be recovered by restoring its
item is deleted. Nothing is left to report to the owner on this path.
*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* the owner confirmed this design on 2026-10-20 (the rename to
`removed-<hex>.store`, the keychain item deleted, the file removed;
leftover `removed-` files finished at start; a `persona-*.store` file
without its item still refused per RC-hd5des). No behaviour change; the
assessment above stands.
assesses: LLR-6vtbn6

### Derived design items: the handle's random source and the Person's types

*Assessed 2026-10-15 (design of change `worktree-org-io-domain-api`,
`org-io/docs/architecture/2026-10-16-domain-api.md`).*

The handle owns its random source and no public method takes one
(LLR-ctm86b): this removes a way for the app, or a test helper leaking into
a release, to hand org-node a weak or predictable generator for an
Organisation private key, a Device seed or an invite identifier, which is
what the impostor-through-an-invite-id and key-disclosure hazards in
org-node's ledger rely on not happening. The seeded replacement exists only
in `test-support` builds (LLR-5acgfg), the feature already stated never to
ship (it also carries the in-memory keychain and iroh's test utilities), so
it adds no release path. On an OS entropy failure (LLR-94nn6q) org-io
produces no value from any other source: org-io's own draw is a typed
refusal, and a draw inside org-node panics before any store change is
returned, so nothing is written; the cost is availability (the operation
does not complete), never a guessable key. The panic is accepted rather
than mapped, because rand_core 0.6.4's `fill_bytes` gives org-node no error
to return; no hazard follows from it. Naming the Person only through
org-node's types (LLR-b97usg) is a dependency-boundary choice with no
hazard impact: the Person holds no Organisation data and is never shown as
verified (REQ-ggjuu6's assessment).
assesses: LLR-ctm86b, LLR-5acgfg, LLR-94nn6q, LLR-b97usg
