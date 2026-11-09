# org-io: the demo journey's domain surface

Change `worktree-org-io-domain-api`, decided 2026-10-14 in the
`grill-requirements` interview for the demo journey (a user founds an
Organisation, invites and admits a second user over iroh, and later revokes
them). The function names the owner chose for org-io's surface are design and
are recorded in the plan and the architecture ledger, not here.

This change is written before stage S4 (`worktree-org-io-transport`) merges
and merges after it; references to S4's items resolve once master carries
them.

Every requirement is `satisfies: derived`: each exists because of how the
owner shaped the demo journey's surface. Per this unit's citation rule,
org-node's items are named by description only.

## Superseded after S4 merged

S4 (`worktree-org-io-transport`) merged into this branch on 2026-10-15. The
owner then ruled (2026-10-15): R1, each provisional founding and each
provisional Member is a new Persona, with its own Device seed keychain item,
sealed store file and endpoint, so S4's layout stays, and the Person has its
own keychain item and a small sealed Person file; R2, received Organisation
information is kept as a provisional organisation and reported, and the app
asks for its verification at once, with no chain read and no adoption on
receipt; R3, an invitee's first admission takes the same path, behind
org-node's gate on expected first admissions, and is adopted only by a
verification that finds the chain matching and the record listing the
provisional Member's keys; R4, the Invite carries each inviter Device's
endpoint addresses in the current mode, and the Loopback identity check
(REQ-8trkps) still applies; R5, a write overtaken on the chain by another
root is reported deprecated, not "not executed"; R6, answers to organisation
requests go on a new connection to the requester's Device key through the
send path, the inbound connection carries only a verdict, and the verdict
gains "already received" and "refused because the invite was used by another
Device"; R7, a Device no held record lists may send an admission request, and
the acknowledgement S3 allows a Device held as revoked, and nothing else —
its organisation request is refused — and an admission request is accepted
only by the Device that issued the Invite, the one Device the Invite names,
which expects it because it holds the invite as outstanding. The items
below replace S4's requirements for the Invite, inbound connections and
Invite replies, the receive path, the admission flow, the Invite reply, the
fan-out, and S4's three expectations on org-node, which are deleted. The app supersedes its own warning before replying and its
submission decision in its ledger.

## The Person

**REQ-ggjuu6**: The software shall keep on the device a Person the user
proposes apart from any Organisation — a name, a surname, the Person's own
DevicePublicKey and PersonPublicKey, and no Handle — and shall show that
Person as provisional, never as verified, until its Person hash has been
written to the chain and verified against it. It shall keep one Person per
device: a proposal of a Person while one is held shall be refused with an
error naming the Person already held, writing nothing.
satisfies: derived

Writing a Person hash to the chain is future scope (owner ruling 2026-10-14),
so in this change every Person stays provisional.

*Owner ruling R1, 2026-10-15 (change `worktree-org-io-domain-api`):* the
Person has its own keychain item and a small sealed Person file, apart from
every Persona's; each provisional founding and each provisional Member is a
new Persona with its own keychain item, sealed store file and endpoint.

*Amended in place 2026-10-22 (unmerged draft; plan detail pass — owner
ruling of 2026-10-22):* one Person per device. A second proposal is refused
with a typed error naming the Person already held, and nothing is written:
no keychain item, no Person file, and the Person held is unchanged.

## Write, verify, send

Three steps the app calls in turn, each when it decides to (owner ruling
2026-10-14), replacing the single submit-commit-send flow.

**REQ-3pjmub**: The software shall, at the app's request, write one
provisional organisation it holds to the chain through on-chain-client — a
founding, or the next epoch of an Organisation it holds — and report that the
write was included, was refused, or has an unknown outcome; it shall change
nothing in its store in doing so except the proxy address of a founding
(REQ-qfnx94), and shall refuse, writing nothing, a provisional organisation
whose provisional changes or current record have changed since it was
proposed.
satisfies: derived
exported: yes

**REQ-36gxmg**: The software shall, for an Organisation it holds or a
provisional founding it holds (at the proxy address stored with it), when the
app starts and whenever the app asks, read the Organisation state at a
finalised block and judge it against its current record and the provisional
organisations it holds: at the current epoch with the current Membership
root, change nothing but what verification's discard rules discard or
stamp, and report it in step; at the next epoch with the
Membership root and Organisation public key of a provisional organisation it
holds, adopt that one as current, write it to its store and report it
adopted — or, when that record no longer lists this node, write the store
org-node returns, which keeps for that Organisation only the acknowledgements
of the removal, and report it removed; when org-node accepts, against that
state, a revocation notice it holds for this node — one pushed by a Member's
send or received in answer to its request for the Organisation — write the
store that acceptance returns, which keeps the same, and report it removed;
in either case giving org-node the time of the verification and, for the
signing, the Persona's Device secret key for that one operation; at a later epoch that no provisional
organisation matches, change nothing but what those rules discard or stamp,
and report it behind with the chain's epoch; at an earlier epoch, or at the current epoch with another Membership
root, change nothing but what those rules discard or stamp, and report it
inconsistent with the reason; and when the
read fails or does not parse, change nothing and report it unverified with
the reason.
satisfies: derived
exported: yes

*Amended in place 2026-10-16 (unmerged draft; owner ruling Q3 of
2026-10-16):* a revoked Device that asks for the Organisation is answered
with its revocation notice (org-node's answer to an organisation request),
and that notice, checked against the chain's root, is a second way this
verification reports the Organisation removed.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* "forget the Organisation" becomes writing the store org-node
returns on the removal, which deletes at once, with no user step and no app
confirmation, everything for that Organisation but its acknowledgements
(org-node's requirement on its own removal); sending them, deleting each on
acceptance and the 7-day deadline are REQ-k6kcgd, REQ-sd7ayv and
REQ-98a3ty. Verification deletes nothing in any other case. Owner ruling Q4
(verification deletes nothing; deletion only after the app confirms), which
the note above recorded as pending until this amendment, was withdrawn by
the owner and replaced by Q4'; that pending note is removed.

*Amended in place 2026-10-30 (unmerged draft; merge review round 4,
MR4-O1):* org-node's verification, on the outcomes its own design names,
discards the provisional organisations received from other Devices that no
later chain state can adopt, stamps the first time it finds one one epoch
ahead of the chain and discards it once held there past the ahead limit
(org-node's requirement on keeping Organisation information from any
Device, as amended 2026-10-30), and returns that store change. On the in-step,
behind and inconsistent outcomes this software writes its store only to keep
what those rules discarded or stamped, and otherwise writes nothing; the
record, the provisional changes and this node's own proposals are not
touched. "Verification deletes nothing in any other case" (the note of
2026-10-20) stands but for these discards. When the write of that change
fails, the verification reports the store failure, not the outcome, as a
failed write of an adoption or removal does (LLR-zcw3xv): the Persona is
reloaded from its file, which is as it was, and for a behind Organisation
the app makes no organisation request that round; the next verification
discards or stamps again and reports the outcome.

**REQ-nactq6**: The software shall, at the app's request, send an
Organisation's current epoch to every Device it concerns — Organisation
information to each Device the current record lists other than this node's
own, and a revocation notice to each Device that epoch removed — at the
endpoint addresses org-node keeps for each Device's key, attempting
each Device whatever happened to the others, report the outcome for each
Device, change nothing in its store, and send the same again whenever the app
asks again.
implements: RC-wbs9ch
satisfies: derived
exported: yes

*Amended in place 2026-10-15 (unmerged draft; change
`worktree-org-io-domain-api`):* now that S4 has merged, this item also
replaces S4's fan-out and its admission flow requirements;
it carries the fan-out's control, so "attempting each Device whatever
happened to the others" is added from it. The current epoch is the adopted
one: an epoch becomes current only when verification adopts it.

*Amended in place 2026-10-16 (unmerged draft; owner ruling Q1 and owner
answer A2 of 2026-10-16):* the app asks for this send once per adopted epoch
this node wrote and offers no retry (the app's ledger); this item keeps its
per-Device outcome report, and sending the same again on a later request
stays harmless. A Device the single send missed is behind at its next
periodic verification and requests the Organisation (REQ-z73ctw); a revoked
one is answered with its notice when it asks (REQ-wpq4fp). Each Device is
reached at the addresses its own admission request or organisation request
carried, which org-node keeps for its key; the Loopback identity check
(REQ-8trkps) still applies, and in Networked mode they are a hint ahead of
discovery. No address is taken from an inbound connection's source.

**REQ-bm4ppg**: The software shall, when a chain write of a provisional
organisation has an unknown outcome, keep that provisional organisation,
write no other provisional organisation for that Organisation, and end the
unknown state only when verification (REQ-36gxmg) settles it: reporting the
write adopted when verification adopted that provisional organisation;
not executed only when a finalised block later than the last block at which
that write could be included shows the Organisation still at the epoch the
write was based on; and deprecated, with the Organisation reported behind,
when the chain's current epoch is at or past the epoch the write expected
with a Membership root other than that provisional organisation's; until
then it shall report the write's outcome as still unknown.
satisfies: derived

*Amended in place 2026-10-15 (unmerged draft; owner ruling R5, change
`worktree-org-io-domain-api`):* the end of the unknown state now names its
report. A chain past the expected epoch with another root does not decide
whether this write executed (it may have executed and been followed by
another), so it is reported deprecated rather than not executed; this
answers the review round 2 problem whose settle reported such a write as
not executed from the local epoch alone. PR-k5tdxj's "after a restart the reconcile sends to no one" (its item
2) is answered by the app asking org-io to send an adopted epoch this node
wrote at once (the app's ledger), since sending is now a separate step.

*Note 2026-10-16 (dispatcher's resolution C7 and owner ruling Q6, change
`worktree-org-io-domain-api`):* the unknown state is kept in memory only, as
the publish state is today. After a restart the provisional organisation
stays in the store and the next verification judges it against the chain;
a second write for the same epoch is refused by the chain, because the
Organisation slot's update requires the expected epoch to equal the slot's
current epoch and every executed update advances it (the contract's
`EpochMismatch` check, which on-chain-client's update submission reports as
a failed inner call), so at most one write per epoch executes. A crash after
adoption and before the send is closed by the pull rather than by a resend:
each Device the epoch concerns is behind at its next periodic verification
and requests the Organisation (REQ-z73ctw), a revoked one being answered with
its notice (REQ-wpq4fp); this is how PR-k5tdxj's item 2 is answered.

*Amended in place 2026-10-24 (unmerged draft; owner decision on the
founding's `create_proxy` timeout):*
for a founding, the write whose outcome can be unknown is the founding at
its stored proxy (REQ-qfnx94). Creating the proxy, before any proxy address
is known, has no unknown outcome under this requirement: when it times out,
or otherwise fails, nothing is kept for verification to settle and the next
write for that founding creates the proxy again. Verification of a founding
with no proxy stored can read nothing, so an unknown state held for it could
never end. The cost, if the timed-out creation did execute, is an orphan
pure proxy holding only its proxy deposit (funding happens only when the
Organisation is founded at the proxy), the cost already assessed for a
retried founding.

## Admission requests

**REQ-sjqxg2**: The software shall, at the app's request, send a provisional
Member's admission request — the Invite's Organisation identifier and invite
id, and the Member's name, surname, preferred Handles, Member-as-a-group key
and DevicePublicKey, with the endpoint addresses the provisional Member's own
endpoint holds in the current mode — to the Device the Invite names as its issuer, at the
addresses the Invite gives for it, report the outcome (delivered, already received, refused
because the invite was already used by another Device, or not delivered with
the reason), change nothing in its store, and send the same again whenever
the app asks again.
satisfies: derived

*Amended in place 2026-10-16 (unmerged draft; owner answer A2 of
2026-10-16):* the request carries the sender's own addresses, as the Invite
does (R4), so that the issuing Device can send the admitting epoch to the
new Device without an address being pasted or taken from the inbound
connection.

**REQ-wb93ch**: The software shall, at the app's request, return for each
Organisation it holds every admission request it keeps — its invite id, the
name, surname, preferred Handles and DevicePublicKey it carries, and the
invitee name the Admin typed for that Invite — changing nothing.
satisfies: derived
exported: yes

Owner answer A3, 2026-10-16: kept admission requests survive a restart in
the store, and the app shows them at start beside those reported as they
arrive.

## Requesting the Organisation

**REQ-z73ctw**: The software shall, at the app's request, ask each Device of
an Organisation's current record other than its own for the Organisation —
or, for a provisional Member, which holds no record, the Device its Invite
names as issuer, at the addresses the Invite gives for it — with a request
carrying the endpoint addresses its own endpoint holds in the current mode,
report the outcome for each Device, change nothing in its store, and ask
again whenever the app asks again.
satisfies: derived

*Amended in place 2026-10-16 (unmerged draft; owner answer A2 and
dispatcher's resolution C3 of 2026-10-16):* the request carries the sender's
own addresses, so the answer can be sent to them (REQ-wpq4fp). A provisional
Member's store has no record, so "each Device of the current record" would
be empty; it asks the Invite's issuer, which answers once its current record
lists the requester. The app asks for this with no user step when a
verification reports the Organisation behind, and again each interval (the
app's ledger, owner ruling Q2).

**REQ-wpq4fp**: The software shall answer every request for an Organisation
it receives as org-node decides, with no request from the app and without
asking its user — the full record to a Device the current record lists, the
revocation notice alone to a Device it holds as revoked — sending the answer
to the requesting Device's key, at the endpoint addresses its request
carried, on a new connection through its send path and never on the
connection the request arrived on, and shall report each answer to the app
with the requesting Device.
satisfies: derived

*Amended in place 2026-10-16 (unmerged draft; owner ruling Q3 and owner
answer A2 of 2026-10-16):* a revoked Device that asks is answered with its
revocation notice and nothing else, as org-node decides; the answer goes to
the addresses the request carried, never to the inbound connection's
source.

*Amended in place 2026-10-15 (unmerged draft; owner ruling R6, change
`worktree-org-io-domain-api`):* the answer goes out as a send to the
requester's Device key, so the inbound connection carries only a verdict
(REQ-xd5a2d) and a requester whose key is not the connection's never
receives it.

## Discarding

**REQ-7jh4rh**: The software shall itself, with no request from the app,
discard a provisional organisation it holds — its own or one it received —
with the Organisation private key it holds, once verification against the
chain shows it can never be adopted: the chain holds another Membership root
at its epoch or at a later one, and no chain write of it may still execute.
It shall never discard a provisional organisation whose chain write may
still execute, and shall offer the app no command that discards a
provisional organisation or a provisional change.
satisfies: derived

**REQ-kh2uth**: The software shall delete no data it holds, or has org-node
hold, for an Organisation — a record, a Member, a provisional organisation or
provisional change, a file, or a peer address — except a provisional
organisation REQ-7jh4rh discards, a provisional change an adopted epoch left
out, a deletion org-node returns as its acceptance of a verified
revocation or its adoption of a verified epoch that removes this node, an
acknowledgement org-node deletes once its recipient accepted it
(REQ-sd7ayv), and the Persona's store file and keychain item 7 days after
the verification of its removal (REQ-98a3ty); a
failed or refused chain read, a chain state reported as behind or
inconsistent, a write whose outcome is unknown, and a refused message shall
delete nothing else.
satisfies: derived

Discarding is automatic only (owner, 2026-10-14): the provisional changes an
adopted epoch left out are discarded with it, each reported (org-node's
ledger); handling several Admins is a future roadmap stage.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* the exceptions now include the two deletions that follow this
node's own removal — an acknowledgement its recipient accepted, and what is
left 7 days after the removal was verified. The deletion on the removal
itself keeps the acknowledgements, and none of the three asks the app.

## Acting on this node's removal

*Owner ruling Q4', 2026-10-16 (replacing ruling Q4).* After verification
reports this node removed (REQ-36gxmg), org-node keeps, for that
Organisation, only the signed acknowledgements of the removal — three, two,
one or none by the Devices of the Organisation it knew — each with its
recipient's key and addresses, and the time of the verification. These items
are org-io's half: sending them, acting on the answer, and the deadline.
S3b-io part 2's acknowledgement delivery, which no item in force stated
(S4's inbound item recorded it only as S3b-io's), is stated here.

**REQ-k6kcgd**: The software shall, once verification has reported this node
removed from an Organisation, send each acknowledgement org-node keeps for
that Organisation to its recipient Device, at the endpoint addresses kept
with it, over a direct connection with no relay under the Persona's own
Device key, trying the next address kept for that recipient when one refuses
or cannot be reached, and report the outcome for each recipient.
satisfies: derived

A Member accepts an acknowledgement only over a connection authenticated as
the Device it names (org-node's sender rule), so it goes under the revoked
Device's own endpoint identity, never from a fresh one, and the Persona's
keychain item stays until REQ-98a3ty removes it. A Member that has not yet
adopted the removing epoch still lists the Device and refuses. When an
acknowledgement that has not been accepted is sent again is set by REQ-jqmpz5
(owner ruling of 2026-10-20: a backoff schedule).

*Note 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; owner
ruling Q1 (c) of 2026-11-03, as recommended; a clarification, the ID is kept):* REQ-kn8zg6 adds what
happens to a recipient with no kept address that may be dialled: in
Networked mode it is sent to by its Device key alone. "With no relay" here
is ruling D7's: no other Device forwards the acknowledgement; iroh's relay
server may carry the connection.

*Note 2026-11-09 (change `worktree-org-io-publish-2`, merge review round 1,
MR1-O2; a clarification narrowing the note of 2026-11-03, the ID and the
item's text are kept):* the relay server's part is confined to REQ-kn8zg6's
dial. The dial by a recipient's key alone when no kept address may be
dialled is REQ-kn8zg6's, outside this item's "at the endpoint addresses
kept with it"; every dial at a kept address stays direct with no relay
(LLR-spckm6), as this item's text states.

**REQ-sd7ayv**: The software shall, when a recipient answers an
acknowledgement accepted, have org-node delete that acknowledgement with its
recipient's key and addresses and write the sealed store it returns; and on
any other answer, or none, delete nothing.
satisfies: derived

**REQ-98a3ty**: The software shall, 7 days after the time org-node keeps as
that of the verification of this node's removal from an Organisation, delete
the store file and the OS keychain item of the Persona bound to that
Organisation, whatever was delivered; shall delete them earlier, at once,
when org-node keeps no acknowledgement for that Organisation, because every
one was accepted or none was kept; and shall, at start, delete them at once
for a Persona whose deadline has already passed or that keeps no
acknowledgement.
satisfies: derived

The deadline is kept as a time in the store (org-node's requirement on its
own removal), so it survives a restart; org-node reads no clock.
*Amended in place 2026-10-20 (unmerged draft; owner ruling of 2026-10-20):*
the store file and keychain item go as soon as no acknowledgement is left, so
the Device secret leaves the keychain as early as possible; 7 days is the
upper bound.

**REQ-jqmpz5**: The software shall send again each acknowledgement org-node
keeps for an Organisation that its recipient has not answered accepted, 1
minute, 5 minutes and 30 minutes after the first send and every hour after
that, each time to that recipient as REQ-k6kcgd states, until it is accepted
or REQ-98a3ty deletes it, and shall resume that schedule at start from the
time org-node keeps as that of the verification of the removal.
satisfies: derived

Owner ruling of 2026-10-20: a backoff schedule, not the verification
interval. A Member that refused because it had not yet adopted the removing
epoch will have pulled it by a later send.

## Founding an Organisation

**REQ-qfnx94**: The software shall, when the chain write of a provisional
founding has created the Organisation's pure proxy, store the proxy's
address with that provisional founding before it takes any further step of
the write, so that the founding can be verified against the chain after the
app restarts; and shall, when the app asks again to write a provisional
founding that already has a stored proxy address, found the Organisation at
that proxy and create no other.
satisfies: derived

This is the one store write a chain write makes (owner ruling 2026-10-14):
the Organisation identifier is the proxy's address, known only once the
proxy exists, and without it a founding interrupted before verification
could never be found on the chain again.

*Amended in place 2026-10-16 (unmerged draft; owner answer A4 of
2026-10-16):* on-chain-client now offers the genesis ceremony in two steps,
creating the proxy and then founding at it (REQ-wd683g); org-io stores the
proxy between the two, and a retry reuses the stored proxy, where
on-chain-client skips what the chain shows already done.

**REQ-wd683g**: The software shall rely on on-chain-client to offer the
genesis ceremony in two steps — one that creates the pure proxy under the
controller and returns its address, and one that, given that proxy, the
genesis Membership root and the Organisation public key, funds it, maps it
and records the genesis root and key at expected epoch zero, each step only
after the previous one succeeded and a failure naming its step — and to let
the second be run again on a proxy the chain shows already funded or mapped,
skipping what is done as the chain shows it and never from what it holds in
memory.
expects: on-chain-client
opened: 2026-10-16
satisfies: derived

## The Organisation name

**REQ-4bvmwc**: The software shall keep with each Organisation it holds the
Organisation's name as free text, in its local store only — on the founder's
device the name the founder gave when proposing the founding, and on a
joining device the name the Invite carried — and shall send it in no message
other than an Invite, write it to no chain, and verify it against nothing.
satisfies: derived

The name is a local display label (owner, 2026-10-15). Publishing it through
dotNS, the Polkadot name service, pointing at the Organisation's on-chain
account, is future scope. The Invite requirement S4 left (whose note recorded the 2026-10-05 ruling
that the name is "stored nowhere") is replaced by REQ-ek652a so that an
Invite carries the stored name.

## The Invite

**REQ-ek652a**: The software shall produce, for an Organisation the user
holds, an Invite as a Blob carrying the Organisation name stored for it
(REQ-4bvmwc), the Organisation identifier, the inviter's guess of the
invitee's full name, the DevicePublicKey of the Device issuing it with the
endpoint addresses that Device's endpoint holds in the current mode, and an invite identifier of 32 bytes drawn from its
cryptographic random source, and shall keep that identifier as outstanding.
satisfies: derived
exported: yes

Owner rulings 2026-10-15 (change `worktree-org-io-domain-api`): the name is
the stored one (REQ-4bvmwc), no longer the text the inviter types; (R7) the
Invite names only the issuing Device, which alone holds the invite as
outstanding; and (R4) that Device comes with the addresses its endpoint holds — loopback
sockets in Loopback mode, direct addresses in Networked mode as a hint ahead
of discovery — so that the invitee's admission request can reach it. The
identity check of a Loopback send (REQ-8trkps) still applies to these
addresses.

## Receiving

**REQ-xd5a2d**: The software shall accept an inbound connection from any
Device whose handshake completes; hand every message on it to org-node's
receive path with the connection's authenticated DevicePublicKey; act on a
message from a Device no record it holds lists only when org-node accepts it
as an acknowledgement or an organisation request from a Device it holds as
revoked, or as an admission
request for an Invite this Device issued and holds as outstanding; refuse
every other message from such a Device before any chain read or store
write; and write on any inbound connection nothing but a single verdict —
accepted, refused, already received, or refused because the invite was used
by another Device — serving on it no Organisation information, record, key,
address or other Organisation data.
implements: RC-6m89tr
satisfies: derived

Owner rulings 2026-10-15 (R6, change `worktree-org-io-domain-api`): an
admission request comes from a Device no record lists yet, so it joins the
acknowledgement as a message such a Device may send; org-node decides each.
*Narrowed the same day (owner ruling R7):* an organisation request from an
unlisted Device is refused, and an admission request is acted on only by the
Device that issued its Invite. Every answer that carries anything goes out through the send
path to a Device key (REQ-wpq4fp), never on the inbound connection. "Already
received" and "refused because the invite was used by another Device" are
the two verdicts of org-node's first-request rule, refining accepted and
refused; they tell the sender only about its own request. Invite replies,
which S4 refused when not outstanding, are replaced by admission
requests over iroh, checked by org-node against the outstanding pair.

*Amended in place 2026-10-16 (unmerged draft; owner ruling Q3 of
2026-10-16):* a Device this node holds as revoked may also send an
organisation request; org-node answers it with that Device's revocation
notice and nothing else, which goes out through the send path to that
Device's key (REQ-wpq4fp), never on the inbound connection. An organisation
request from a Device no record has ever listed is still refused.

**REQ-cj9u3y**: The software shall hand every received message body to
org-node's one receive path, keep in the sealed store what org-node returns
— a provisional organisation, a kept admission request, a kept revocation
notice, a settled acknowledgement — report each to the app, and read no
chain and adopt nothing on receipt.
satisfies: derived
exported: yes

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* a revocation notice org-node keeps for this node is named
among what is reported, and the item is exported: the app asks for the
Organisation's verification at once when org-io reports a provisional
organisation received or a revocation notice kept (the app's ledger), so
the report of each is part of org-io's interface to the app.

Owner rulings R2 and R3, 2026-10-15 (change `worktree-org-io-domain-api`):
received Organisation information, an invitee's first admission included, is
kept as a provisional organisation and reported; the app then asks for its
verification at once (REQ-36gxmg), and only verification reads the chain and
adopts. No peer can drive a chain read by sending.

## What org-io relies on org-node for, after S4

**REQ-2yw5y3**: The software shall rely on org-node to offer one receive
path driven by values: a chain-free check of a received body with the
connection's authenticated DevicePublicKey, returning it as a provisional
organisation, an admission request with its verdict, an organisation request
with its answer, a settled acknowledgement or a refusal, covering an
invitee's first admission and the node's own revocation, and returning the
sealed store.
expects: org-node
opened: 2026-10-15
satisfies: derived

**REQ-u894ty**: The software shall rely on org-node to return, for the
adopted current epoch of an Organisation, on each request, every recipient
Device and the encoded message for each, writing nothing.
expects: org-node
opened: 2026-10-15
satisfies: derived

**REQ-2xgsbn**: The software shall rely on org-node to build and parse the
Invite, to build the admission request and the organisation request, each
carrying the sending Device's endpoint addresses, to check the admission
request, to record, keep and
settle outstanding invite identifiers, kept admission requests and expected
admissions inside the store, and to keep and return the endpoint addresses
accepted requests carried for each sending Device's key, returning the
sealed store from each.
expects: org-node
opened: 2026-10-15
satisfies: derived

*Amended in place 2026-10-16 (unmerged draft; owner answer A2 of
2026-10-16):* the requests carry their sender's addresses, which org-node
keeps per Device key for this unit's send path.

**REQ-bg5whd**: The software shall rely on org-node to return, at its
request, every acknowledgement it keeps of this node's removal from an
Organisation, each with its recipient Device's DevicePublicKey and endpoint
addresses, and the time kept as that of the verification of the removal,
changing nothing; and to delete, at its request, a kept acknowledgement
whose recipient answered it accepted, with that recipient's key and
addresses, returning the sealed store and whether any acknowledgement is
still kept for that Organisation.
expects: org-node
opened: 2026-10-20
satisfies: derived

Owner ruling Q4' (2026-10-16): REQ-k6kcgd, REQ-sd7ayv and REQ-98a3ty rest
on it.

Owner rulings 2026-10-15 (change `worktree-org-io-domain-api`): the receive
path no longer commits against a chain state org-io read (R2, R3); the send
is of the adopted current epoch, asked for by the app; and the Invite reply
is replaced by the admission request.
