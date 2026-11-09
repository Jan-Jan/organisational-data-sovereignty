# org-node: provisional changes batched into one next epoch

Change `worktree-org-io-domain-api`, decided 2026-10-14 in the
`grill-requirements` interview for the demo journey's domain-language org-io
surface. The owner ruled that an Admin's admissions and revocations are
provisional changes against the Organisation's current record,
and that the Membership root and the Organisation key pair of the next epoch
are calculated once, over all of them, when the Admin proposes the updated
Organisation. An epoch is adopted only once it has been verified against the
chain, and the provisional changes it did not include are discarded
automatically and each reported, so an Admin whose changes another Admin's
epoch left out can propose them again. Several Admins are not the demo's
case; carrying over a left-out revocation, and Admins who disagree, are a
future roadmap stage (owner, 2026-10-15). Handling Admins who disagree is future scope (owner, 2026-10-14).
Removing one Device of a Member, and rotating a Member-as-a-group key on its
own, are future kinds of provisional change (owner, 2026-10-15); the
Organisation key pair is rotated with every epoch and needs no change of its
own.

This change is written before stage S4 (`worktree-org-io-transport`) merges
and merges after it.

Every requirement is `satisfies: derived`: each exists because of where the
owner placed the calculation of an epoch.

**REQ-4qh2h8**: The software shall, when the user proposes founding an
Organisation with the founder's name, surname and Handle, draw a
Member-as-a-group key pair and a Device key pair used for that Organisation
alone, build the founding record listing the founder with them, calculate its
Membership root, draw its Organisation key pair, and keep the result as a
provisional founding, writing nothing to the chain.
supersedes: REQ-xs4ab8
satisfies: derived

*Owner ruling R1, 2026-10-15 (change `worktree-org-io-domain-api`):* each
provisional founding is a new Persona, with its own Device seed keychain
item, sealed store file and endpoint, as the Persona layout already is.

**REQ-dhr58q**: The software shall record each admission and revocation an
Admin proposes for an Organisation as a provisional change
against the Organisation's current record, keep it in the store with that
Organisation's other provisional changes, and, in doing so, calculate no
Membership root, draw no Organisation key pair, write nothing to the chain
and leave its current record unchanged.
supersedes: REQ-xs4ab8
satisfies: derived

**REQ-3pj7rd**: The software shall, when the app asks it to propose the
updated Organisation, apply every provisional change it holds for that
Organisation to the current record, calculate the resulting Membership root
once, draw one fresh X25519 Organisation key pair, distinct from the
record's current Organisation public key and from every Member-as-a-group
key and DevicePublicKey in the resulting record, and keep the result, with
its private key in the encrypted store, as that Organisation's provisional
organisation for the epoch after its current one.
supersedes: REQ-stx9v3
satisfies: derived

**REQ-x2dnee**: The software shall, when it adopts as current an epoch of an
Organisation that it has verified against the chain — its own provisional
organisation or Organisation information it received — replace its previous
record of that Organisation with the new one, keep with the new record the
Change set that produced it and no earlier one (a record created by a
founding keeps none); and discard every provisional change it holds for that
Organisation that the adopted epoch did not include, reporting each to the
app.
(implements: RC-dv6mka)
supersedes: REQ-uv3v5w REQ-hhva9d
satisfies: derived

*Amended in place 2026-10-16 (unmerged draft; dispatcher's resolution C5 of
2026-10-16):* this item also supersedes REQ-hhva9d (discard a provisional
update on request). Discarding is now automatic only, as this item and
org-io's discard requirement state; no command discards on request, so
REQ-hhva9d had no successor in force.

**REQ-9ktgwx**: The software shall keep Organisation information it receives
from a Device its current record lists, for the epoch after its current one,
as a provisional organisation alongside any other provisional organisation it
holds for that Organisation — its own included — and shall adopt none of them
as current until verification against the chain, and then only the one whose
Membership root and Organisation public key the chain holds.
(implements: RC-d4fc4s)
satisfies: derived

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1, owner ruling 2026-10-30):* the item keeps what it receives
only until verification can tell it will never be adopted. Every
verification discards a received one below the chain's epoch, beyond the
next epoch, or at the chain's epoch without its root and key, and keeps
one for the next epoch only for a time the caller supplies (LLR-4tsgfh).
This is RC-d4fc4s's discard clause; a store with no record keeps Organisation
information from any Device (LLR-e6w3nr) under the same rule.

More than one provisional organisation for an epoch is rare but expected
(owner, 2026-10-14): a malicious Member may send false Organisation
information while a genuine update goes out, and an Admin may hold their own
provisional organisation when another Admin's arrives; either of the latter
two may be the one written to the chain.

**REQ-dwb9bq**: The software shall, given the Organisation state the caller
read from the chain at a finalised block for an Organisation it holds, report
exactly one outcome: in step, when the state's epoch and Membership root equal
its current record's; adopted, when a provisional organisation it holds — its
own or one it received — is for the state's epoch and has the state's
Membership root and Organisation public key, which it then adopts
(REQ-x2dnee); behind, when the state's epoch is greater and no provisional
organisation matches, leaving the store unchanged; and inconsistent, naming
the reason and leaving the store unchanged, when the state's epoch is lower
than its record's or equal to it with a different Membership root. Given no
Organisation state, it shall refuse with a typed error, leaving the store
unchanged. It shall decide each outcome from the state and the provisional
organisations it holds alone, checking no signature and never consulting the
Device that delivered any of them.
(implements: RC-ub82my, RC-5gbgwb, RC-6a2dke, RC-e5atck, RC-95dgg8)
supersedes: REQ-tb4f8p REQ-bwx7eg REQ-wp2nyc REQ-8gz8bu REQ-nhe2zu REQ-txvtm9
satisfies: derived

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1, owner ruling 2026-10-30):* "leaving the store unchanged" for
behind and inconsistent (and in step) is now read with one exception: given
a state, the software discards every provisional organisation it received
that can never be adopted against it, and keeps one for the state's epoch
plus one for at most a time the caller supplies (LLR-4tsgfh, RC-d4fc4s
through REQ-9ktgwx). Its own provisional organisations, its provisional
changes, its record and everything else stay unchanged; given no state,
nothing changes.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review round 2, finding-4):* the last two sentences are added. REQ-bvh8v6
(no chain state: refuse, nothing changed) and REQ-ag6kqm (no signature and
no Member's or Device's key in the decision) were superseded by LLRs
(LLR-rr558z, LLR-8528ud), so no in-force REQ stated either behaviour,
although both are designed and tested (LLR-rr558z, LLR-8528ud, LLR-qvp7mq).
This item now states them, and the LLRs that satisfied only the superseded
REQs (LLR-pzde8b, LLR-e7s4ye, LLR-cja9zv, LLR-7en38k) satisfy it.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1-N1, owner ruling 2026-10-29):* supersedes REQ-wp2nyc, REQ-8gz8bu,
REQ-nhe2zu, REQ-txvtm9, which `verify_envelope_against_chain` alone did; it is
deleted, and this item states what each became (see each one's note). It
implements in their place the controls they implemented: RC-6a2dke (a record
is adopted only when the chain holds its root, and nothing is adopted with no
chain state, LLR-rr558z), RC-e5atck (only a state at a greater epoch adopts)
and RC-95dgg8 (the adopted record's Sequence number is the chain state's
epoch, LLR-34c5y3).

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1, finding-4):* the last clause read "it shall refuse with a typed
error, leaving the store unchanged, a state whose epoch is lower …". `verify`
reports such a state as the outcome `Inconsistent { reason }`
(LLR-rr558z), not as an error, and its tests assert that outcome; the clause
now says so. The store is left unchanged either way.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* this item
also supersedes REQ-bwx7eg (a typed error on receipt when a received key's
public half is not the chain's key). Receipt reads no chain (owner ruling
R2); a provisional organisation whose Organisation public key is not the
chain's is no match here, so it is neither adopted nor refused with an
error, and the chain's state is reported behind or adopts another one.
RC-9cefcn, which REQ-bwx7eg implemented, keeps REQ-c29s93 as its in-force
implementer; this item does not claim it.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* RC-5gbgwb supersedes RC-9cefcn, and this item implements its
verification half: a provisional organisation is adopted only when its
Organisation public key is the one the chain holds, so nothing of one whose
key is not the chain's is adopted. Its receipt half is REQ-zxgd46's, which
supersedes REQ-c29s93; the sentence above about RC-9cefcn and REQ-c29s93 is
replaced by this note.

**REQ-zxgd46**: The software shall refuse with a typed error, leaving the
store unchanged and reading no chain state, a received Wire message of the
Organisation-information kind that carries no Organisation private key, or
whose key does not decode as 32 bytes; and shall keep, as the Organisation
public key of the provisional organisation such a message is kept as, the
X25519 public half derived from the key it carries.
(implements: RC-5gbgwb)
supersedes: REQ-c29s93
satisfies: derived

Whether that derived key is the chain's is decided only at verification
(REQ-dwb9bq). Added 2026-10-21 (owner decision of 2026-10-20); narrowed in
place the same day (owner decision of 2026-10-21): no comparison with a
stated public key, since the message carries none.

**REQ-g2537c**: The software shall refuse, with a typed error naming the
limit, to keep a provisional change or a provisional organisation — its own
or one it received — that would bring the provisional changes and
provisional organisations it holds for one Organisation above 1 MiB in their
stored form, and shall keep nothing in that case.
(implements: RC-d4fc4s)
supersedes: REQ-fwfku9
satisfies: derived

**REQ-rv4dwb**: The software shall, for any sequence of bytes received as a
message body — Organisation information whose Change set it applies, to its
current record or, holding no record, to the record snapshot the body
carries, included — either act on it or refuse it with a typed error, leaving
the store unchanged on a refusal, and shall not panic.
(implements: RC-e2uvje)
supersedes: REQ-bcxz96
satisfies: derived

*Added 2026-10-30 (change `worktree-org-io-domain-api`, merge review round 2,
MR2-N1).* REQ-bcxz96 bounded the verification of an Envelope against a
record and a chain state; that function is deleted, but applying a received
Change set, the step HAZ-5f9jcm's hostile Change set targets, moved to
receipt. REQ-9g6as6 bounds decoding alone; this item bounds the receipt that
decodes, applies and keeps. Its LLR is LLR-7u3dg7.

## Persons, Members and Handles

A Person has no Handle; a Member is proposed from an Invite with one to three
preferred Handles in ranked order, and with keys drawn for that Organisation
alone (owner, 2026-10-14).

**REQ-kd34f5**: The software shall refuse to create a Person or a provisional
Member, unseal a store, or decode the record snapshot a first admission
extends, whose name or surname, or any Handle, the member-record rules would
refuse, or one of whose keys is not a curve point, and a provisional Member
whose preferred Handles are fewer than one, more than three, or not all
distinct, reporting the field that failed, and shall create, store or extend
nothing in that case.
(implements: RC-zutc67)
supersedes: REQ-qn2erx
satisfies: derived

**REQ-2nnsmv**: The software shall, when it records an admission, give the
joiner the first of their preferred Handles, in their ranked order, that is
held neither by a Member of the current record nor by another provisional
admission for that Organisation; when none is free, it shall give the joiner
the Handle the Admin chooses, and shall refuse, recording nothing, a choice
that is held in the same way or that the member-record rules would refuse.
satisfies: derived

**REQ-hx4r8y**: The software shall, when the user proposes a Member from an
Invite with a name, a surname and preferred Handles, draw a Member-as-a-group
key pair and a Device key pair used for that Organisation alone, keep them
with the Invite's Organisation identifier and invite id as a provisional
Member, and record that it expects a first admission to that Organisation.
satisfies: derived

*Owner ruling R1, 2026-10-15 (change `worktree-org-io-domain-api`):* each
provisional Member is a new Persona, with its own Device seed keychain item,
sealed store file and endpoint, as the Persona layout already is.

## Admission requests

An admission request is how a provisional Member asks the Device that issued
its Invite, over iroh, to be admitted; it replaces the Invite reply pasted back by hand
(owner, 2026-10-14). The request comes from a Device no record lists, so
holding an outstanding invite id is what lets it in: the invite id now opens
an inbound path, and the Admin's check that the person matches the invitee's
name is the control (to `analyze-risks`). S4's rule that an unlisted Device
may send acknowledgements alone (org-io's inbound-connection requirement) is
superseded in org-io's ledger to make this exception (2026-10-15).

*Owner ruling R7, 2026-10-15:* an unlisted Device may send an admission
request and nothing else new; S3's acknowledgement from a Device held as
revoked stays. An organisation request from an unlisted Device is refused. An
admission request is accepted only by the Device that issued its Invite,
which expects it because that Device's store holds the invite as
outstanding, so the Invite names that one Device.

*Owner ruling Q3, 2026-10-16:* a Device a held record lists as removed — a
Device this node holds as revoked — may also send an organisation request,
answered with its revocation notice and nothing else (REQ-p4hwuv). A Device no
record has ever listed is still refused.

*Owner answer A2, 2026-10-16:* an admission request and an organisation
request carry the sending Device's own endpoint addresses in the current
mode, as the Invite does (R4); the receiver keeps them for that Device key
and the send path uses them, with the Loopback identity check still applying
and, in Networked mode, as a hint ahead of discovery.

**REQ-6qxesy**: The software shall accept an admission request from any
Device only while the Organisation identifier and invite id it carries are an
outstanding pair of an Invite this Device issued, refusing every other admission request before decoding
anything beyond those two fields; shall refuse, changing nothing, an
admission request whose DevicePublicKey is not the connection's
authenticated DevicePublicKey; shall keep the first admission request for
an outstanding invite, with the endpoint addresses it carries for its
sending Device, and report it to the app for the Admin's decision;
shall answer a later one for that invite from the same Device, as the
transport authenticated it, as already received, changing nothing; and shall
refuse a later one from any other Device, changing nothing, with a refusal
the sending Device is told of, stating that the invite has already been used.
(implements: RC-sp46zh)
satisfies: derived

*Amended in place 2026-10-16 (unmerged draft; dispatcher's resolution C4 and
owner answer A2 of 2026-10-16):* the request's DevicePublicKey must be the
key the connection authenticated, so that one Device cannot request
admission for another key; a mismatch is refused with the verdict refused.
The kept request also keeps the sender's endpoint addresses, which the send
path uses to reach the admitted Device.

**REQ-f8p36u**: The software shall, at the app's request, remove an
admission request it keeps for an outstanding invite and cancel that invite,
so that every later admission request carrying its invite id is refused, and
shall refuse, changing nothing, a request naming no admission request it
keeps.
(implements: RC-amgwm9)
satisfies: derived

**REQ-nbx4nn**: The software shall settle an Invite once an epoch it adopted
after verification against the chain admits the Member that Invite's
admission request named, so that one Invite admits one Member, and shall
refuse every later admission request carrying that invite id.
satisfies: derived

## Requesting the Organisation

When verification finds an Organisation behind — the chain at a later epoch than any provisional organisation it holds — the device asks the Devices of its current record for the Organisation (roadmap stage S6; owner, 2026-10-14). The answer is a full record, because the chain holds only the latest Membership root, Organisation public key and epoch.

**REQ-p4hwuv**: The software shall answer a request for an Organisation it holds, from a Device its current record lists, with the current epoch's full record and the Organisation private key that record holds, without asking its user; shall answer a request from a Device a record it holds lists as removed — a Device it holds as revoked — with that Device's revocation notice and nothing else; and shall refuse a request from any other Device, a Device no record it holds has ever listed included, answering nothing and changing nothing.
satisfies: derived

*Amended 2026-10-15 (owner ruling R7):* a request from an unlisted Device is refused; the revocation-notice answer is withdrawn. A revoked Device learns of its removal from the notice the send step delivers when the removing epoch is sent.

*Amended in place 2026-10-16 (unmerged draft; owner ruling Q3 of 2026-10-16):* the revocation-notice answer returns, for a Device this node holds as revoked only; a Device no record has ever listed is still refused. Every answer goes out through the send path to the requesting Device's key, never on the connection the request arrived on (R6). The notice is what the send step already delivers to a removed Device when the removing epoch is sent; answering a revoked Device's request with it lets a Device that missed that single send (owner ruling Q1) learn of its removal by asking.

**REQ-2yduet**: The software shall adopt a full record received in answer to its request for an Organisation only when verification against the chain finds the chain at that record's epoch with its Membership root and Organisation public key, and the Organisation private key received is the private half of that public key; shall act on a revocation notice received in answer to that request only through verification, accepting it only as REQ-m2xh8q accepts a revocation, against the Membership root of the Organisation state read from the chain, and then report the Organisation removed; and shall refuse anything else, changing nothing.
satisfies: derived

*Amended in place 2026-10-16 (unmerged draft; owner ruling Q3 of 2026-10-16):* the revocation notice a revoked Device receives in answer (REQ-p4hwuv) is checked by that Device against the chain's root before it is acted on. What accepting it deletes is left as the deletion requirements in force state (REQ-uxv2x2, REQ-em28bq); this amendment does not change it.

*Amended in place 2026-10-16 (unmerged draft; owner ruling Q4' of 2026-10-16):* what accepting it deletes, and what it keeps, is now REQ-48zc3e, with its trigger restated by REQ-y9p27b; REQ-uxv2x2 and REQ-em28bq are superseded.

**REQ-akfwf3**: The software shall refuse, changing nothing, to record a provisional change or propose an updated Organisation for an Organisation that its last verification reported behind, until a later verification reports it in step or adopted.
satisfies: derived

## After S4: receiving, sending and the invitation

S4 (`worktree-org-io-transport`) merged into this branch on 2026-10-15; the
items below supersede the sans-IO surface it left (owner rulings R1–R6 of
2026-10-15, recorded in org-io's draft of this change). Received
Organisation information, an invitee's first admission included, is kept as
a provisional organisation and adopted only by verification (R2, R3); the
Invite carries the issuing Device and its addresses (R4, R7); and answers to
organisation requests are sent, never written on the inbound connection (R6).

**REQ-ncv4sc**: The software shall offer one receive path driven by values:
it shall check a received body with no chain state, given the connection's
authenticated DevicePublicKey; return Organisation information from a Device
its current record lists, or a first admission to an Organisation it expects
a first admission to (REQ-8amu2a), as a provisional organisation
(REQ-9ktgwx); return an admission request, checked per REQ-6qxesy, with its
verdict; return an organisation request from a Device its current record
lists, or from a Device a record it holds lists as removed, answered per
REQ-p4hwuv, with its answer; settle an acknowledgement as before; refuse anything else with a
typed error; and return the sealed store.
supersedes: REQ-2ugh2f
satisfies: REQ-2yw5y3
exported: yes

*Amended in place 2026-10-16 (unmerged draft; owner ruling Q3 of
2026-10-16):* an organisation request from a Device this node holds as
revoked is now returned with its answer, the revocation notice (REQ-p4hwuv).

Owner rulings R2 and R3, 2026-10-15: there is no longer a commit against a
chain state the caller read; the receive path keeps, and verification adopts.
REQ-8amu2a's gate stays: a first admission to an Organisation no expected
admission names is refused unread.

**REQ-eqt23j**: The software shall adopt a first admission to an
Organisation only through verification: it shall keep it as a provisional
organisation, whichever Device key the connection authenticated, and adopt
it only when the chain's state matches it and its record lists the
provisional Member's keys, requiring no further Invite import.
(implements: RC-b6mydy)
supersedes: REQ-xa6smf
satisfies: derived

Owner ruling R3, 2026-10-15: RC-b6mydy's intent is kept — a first admission
is accepted whichever listed or unlisted Device delivers it, and only what
the chain has published is adopted; the chain check moves from receipt to
verification.

**REQ-ttv8k7**: The software shall return, for the adopted current epoch of
an Organisation, on each request, every recipient Device and the encoded
message for each, writing nothing.
supersedes: REQ-6ehufy
satisfies: REQ-u894ty
exported: yes

Each request returns the same recipients and messages, so the app's retries
send the same again (owner, 2026-10-14); the per-recipient choice of message
kind and REQ-eg5j8u's body bound are unchanged.

**REQ-ky5tyd**: The software shall build an Invite carrying the Organisation
name stored for it, the DevicePublicKey of the Device issuing it and the
endpoint addresses the caller gives for that Device, and parse one; build a provisional
Member's admission request and an organisation request, each carrying the
endpoint addresses the caller gives for the sending Device, and check a
received admission request per REQ-6qxesy; record,
keep and settle outstanding invite identifiers, each with the invitee name
the inviter typed for that Invite, kept admission requests and expected
admissions inside the store, and return the invitee name with each kept
admission request; keep inside the store, for the sending
Device's key, the endpoint addresses an accepted admission request carries
and those an organisation request carries from a Device its current record
lists or a Device it holds as revoked, and return them for that key; and
return the sealed store from each of those operations that changes it.
supersedes: REQ-58tzex
satisfies: REQ-2xgsbn
exported: yes

*Amended in place 2026-10-20 (unmerged draft):* the invitee name the inviter
typed is kept with the outstanding invite, so the Admin's confirmation can
show it beside the request (the app's confirmation requirement and org-io's
listing of kept admission requests rely on it). It is the same name the
Invite already carries as the inviter's guess of the invitee's full name.

*Amended in place 2026-10-16 (unmerged draft; owner answer A2 of
2026-10-16):* requests carry the sender's own addresses in the current mode,
as the Invite does, and the receiver keeps them per Device key in the
Persona store — with a kept admission request, and in a per-Device address
book for listed Devices and revoked Devices that asked — for org-io's send
path. A request from any other Device is refused before anything is kept.

Owner rulings R4 and R7, 2026-10-15: one Device, the issuer, which alone
holds the invite as outstanding; the addresses are those its
endpoint holds in the current mode, which org-io, holding the endpoints,
supplies.

**REQ-9xf99q**: The software shall refuse an imported Invite that does not
parse, naming the field that failed, and shall act on nothing from it.
supersedes: REQ-yazum3
satisfies: derived

Narrowed from REQ-yazum3 (2026-10-15, change `worktree-org-io-domain-api`):
the Invite reply it also covered is replaced by the admission request, whose
check is REQ-6qxesy.

## Acting on one's own removal

*Owner ruling Q4', 2026-10-16 (replacing ruling Q4 of the same day, "verification
deletes nothing", which the owner withdrew).* After a Device verifies its own
revocation — verification finds that the epoch the chain holds no longer
lists its Device key, or it accepts a revocation notice checked against the
chain's current root, whether the send step pushed it or it came in answer to
the Device's organisation request — it deletes at once, with no user step,
everything it holds for that Organisation except what it needs to send its
acknowledgements: 3 acknowledgements if it knows 5 or more other Devices of
the Organisation, 2 if it knows 3 or 4, 1 if it knows 1 or 2, and none (it
keeps nothing) if it knows none. Each acknowledgement is signed before the
deletion and kept with its recipient's key and addresses; each is deleted
once its recipient answers it accepted, and whatever remains is deleted 7
days after the revocation was verified, delivered or not. Verification
deletes nothing in any other case. No app confirmation is involved.

These items supersede REQ-uxv2x2 (delete everything), REQ-em28bq (delete only
on a verified removal) and REQ-y99c9w (sign, return and keep no
acknowledgement). They are the store's half; sending the acknowledgements,
acting on a recipient's answer, the 7-day deadline and deleting the store
file and keychain item are org-io's.

**REQ-48zc3e**: The software shall, when it adopts through verification an
epoch of an Organisation whose record lists the DevicePublicKey of no Persona
bound to that Organisation, or accepts a revocation of such a Persona's
Device (REQ-m2xh8q) against the Membership root of the Organisation state its
caller read from the chain, delete at once, in one sealed store image and
without asking its user, every piece of data it holds for that Organisation —
its record, the Change set kept with it, the Organisation private key, the
proxy account, every provisional change and provisional organisation, every
expected admission, outstanding invite and kept admission request, every
endpoint address it keeps for another Device, and every key the store holds
for each Persona bound to it — keeping for that Organisation only the
acknowledgements REQ-6tepxq counts, each with its recipient Device's
DevicePublicKey and the endpoint addresses it kept for that key, and the time
its caller gives as that of the verification; and it shall leave the data of
every other Organisation and Persona unchanged.
(implements: RC-5jkcxz, RC-45k3zg)
supersedes: REQ-uxv2x2
satisfies: derived

The time of the verification is kept so that the 7-day deadline, which
org-io keeps, survives a restart; org-node reads no clock.

**REQ-y9p27b**: The software shall delete data it holds for an Organisation
only on its own removal as REQ-48zc3e states — adopting, through verification
against the Organisation state its caller read from the chain, an epoch whose
record lists the DevicePublicKey of no Persona bound to that Organisation, or
accepting against that state a revocation of such a Persona's Device — and,
after that removal, only an acknowledgement it kept, as REQ-ayyt5f states; no
other input — a refused message, a chain state reported behind or
inconsistent, a call that supplies no chain state, a discard of a provisional
change or provisional organisation — shall delete any of it beyond, for a
discard, what is discarded.
(implements: RC-ub82my)
supersedes: REQ-em28bq
satisfies: derived

What is left 7 days after the verification is deleted by org-io removing the
Persona's store file and keychain item; org-node deletes nothing on a clock.

**REQ-h29c8e**: The software shall, before it deletes an Organisation's data
on its own removal (REQ-48zc3e), sign each acknowledgement REQ-6tepxq counts,
naming the Organisation, the Persona's MemberId and DevicePublicKey, and the
epoch and Membership root of the chain state the removal was verified
against, under a domain of its own with that Persona's Device secret key,
which its caller gives it as a transient value for that one operation; shall
refuse with a typed error, deleting nothing, when the caller gives no Device
secret key for a Persona that has an acknowledgement to sign; and shall keep
no Device secret key it was given, in the store or anywhere else, once the
operation returns.
(implements: RC-45k3zg)
supersedes: REQ-y99c9w
satisfies: derived

The signing is REQ-y99c9w's, unchanged, the transient type included (S4's
move-only, zeroised `DeviceSecret`); what changes is that the signed
acknowledgements are kept in the store (REQ-48zc3e) rather than returned and
held by no one.

**REQ-6tepxq**: The software shall, on its own removal from an Organisation
(REQ-48zc3e), count for each Persona bound to that Organisation the Devices
its record held before the removal lists other than that Persona's own, and
keep for that Persona three acknowledgements when they are five or more, two
when they are three or four, one when they are one or two, and none when
there are none, each for a distinct one of those Devices as its recipient.
(implements: RC-45k3zg)
satisfies: derived

Which of those Devices are chosen as recipients is not ruled.

*Narrowed 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; owner
ruling R2 of 2026-11-02):* which Devices are chosen is now ruled by
REQ-32cp7f (`2026-11-02-publish-2.md`): the Device the removal was received
from first, the others from the record's other Devices. The count is
unchanged.

**REQ-ayyt5f**: The software shall, at its caller's request naming an
acknowledgement it keeps for an Organisation whose recipient answered it
accepted, delete that acknowledgement with its recipient's DevicePublicKey
and endpoint addresses, return the sealed store, and report whether any
acknowledgement is still kept for that Organisation; and shall refuse,
changing nothing, a request naming no acknowledgement it keeps.
(implements: RC-45k3zg)
satisfies: REQ-bg5whd
exported: yes

**REQ-35s4eb**: The software shall return, at its caller's request, every
acknowledgement it keeps for an Organisation, each with its recipient
Device's DevicePublicKey and endpoint addresses, and the time kept as that of
the verification of its removal, changing nothing.
satisfies: REQ-bg5whd
exported: yes

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* REQ-ayyt5f and REQ-35s4eb are exported and meet org-io's
expectation that it can list the kept acknowledgements, to send them and to
honour the 7-day deadline, and have one deleted once its recipient accepted
it; they were `satisfies: derived`.
