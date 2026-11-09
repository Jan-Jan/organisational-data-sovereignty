# org-node: risks of the batched epoch and admission requests

Change `worktree-org-io-domain-api`, analysed 2026-10-15 with the owner
(`analyze-risks`), for the requirements in
`org-node/docs/requirements/2026-10-14-batched-epoch.md`.

## An impostor admitted through an invite id

**HAZ-8u3sv8**: an admission request is accepted from any Device holding an
outstanding invite id, and the invite id travels in an Invite pasted over a
third-party channel that others can read; someone who saw the Invite sends an
admission request in the invitee's name from their own Device before the
invitee does, and the Admin admits it; the impostor becomes a Member, receives
the Organisation private key and every grant made to that Member, and can read
the Organisation's data, which in this project's deployments can expose a
source's identity. Severity: S3. Probability: P2.

P2: impersonation on third-party channels is ordinary, and nothing in a
request distinguishes the invitee from someone who knows their name.

**RC-sp46zh**: the first admission request for an outstanding invite is the
only one kept; a later one from the same Device, as the transport
authenticated it, is answered as already received, and one from any other
Device is refused with a refusal its sender is told of, so that an invitee
whose invite another Device used first sees a loud error and can tell the
Admin over another channel.
mitigates: HAZ-8u3sv8

What this control breaks: an invitee whose first request was lost and who
retries from another Device is refused. That is the intended outcome — one
invite, one Device — and the recovery is RC-amgwm9.

**RC-amgwm9**: the Admin can remove an admission request kept for an invite,
which cancels the invite, and issue the invitee a new Invite.
mitigates: HAZ-8u3sv8

What this control breaks: nothing an invitee relies on; a removed request's
Device is refused like any other, and the new Invite carries a new invite id.

The Admin's check, before admitting, that the request's name, surname and
Device match the invitee they named is the app's control on the same hazard,
recorded in the app's risk ledger.

**Residual risk.** S3 / P2 with the controls in place: an impostor who is
first, and whom the Admin's check does not catch, is still admitted. The
matrix holds S3 unacceptable at every probability, so this residual is not
acceptable under it. **The owner accepts it (2026-10-15)** for the demo stage:
the benefit is a working admission over iroh now, and the owner will reduce
the probability in a later change (for example, an out-of-band check of the
invitee's Device key, or an invite secret no third-party channel carries).

## False provisional organisations from any Device

**HAZ-cuj9gn**: Organisation information and organisation answers are kept
as provisional organisations — by a store with no record from any Device,
listed or not, and by a store with a record from a Device that record lists
— and a malicious Device — a Member still listed, or, to a store with no
record, any Device that knows the Organisation identifier — sends a Device
false Organisation information for
the next epoch — one that drops a Member, or carries an Organisation private
key of their choosing — while a genuine update goes out, or floods a Device
with them; were any acted on before verification, a Member could be shown as
removed or data encrypted to a key the attacker holds, and in any case the
receiving Device's store fills. Severity: S3. Probability: P2.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR4 finding-6):* the hazard read "from any Device the current record
lists … a malicious Member", though RC-d4fc4s's MR3-N1 note already found
it is not limited to a listed Device: a store with no record keeps
Organisation information and answers for its expected Organisation from any
Device (LLR-e6w3nr), and LLR-4tsgfh adds no sender restriction. The
statement now says any Device. Severity and probability are unchanged: the
harms are the same, and P2 already allowed for a hostile sender.

*Amended in place 2026-10-31 (change `worktree-org-io-domain-api`, merge
review round 5 finding-5):* "from any Device, listed or not" held only for
a store with no record (LLR-e6w3nr). A store with a record refuses
Organisation information and organisation answers from a Device its record
does not list (`SenderNotListed`, LLR-283x6f, LLR-kd7u2r). The statement is
now qualified by store state, here and in the residual risk below. The
qualification narrows who can reach a store that has a record; severity and
probability are unchanged.

**RC-d4fc4s**: no provisional organisation is adopted, shown as current, or
used before verification against the chain finds its Membership root and
Organisation public key there, the Organisation private key it carries must
be the private half of that public key, a provisional organisation that can
never be adopted is discarded after verification, and the provisional changes
and provisional organisations kept for one Organisation are bounded at 1 MiB.
mitigates: HAZ-cuj9gn

What this control breaks: a genuine update received before the chain carries
it waits, unused, until verification; and once the bound is reached a genuine
one may be refused, after which the Device asks for the Organisation when its
verification reports it behind.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1, owner ruling 2026-10-30):* the hazard is not limited to a
listed Device. A store with no record — a new Member waiting for admission
— keeps Organisation information and organisation answers for its expected
Organisation from any Device (LLR-e6w3nr), so any Device that knows the
Organisation identifier can send false ones or flood the store, and while
the bound is full the genuine information and answer are refused with
`ProvisionalLimit`.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1, owner ruling 2026-10-30):* this control's discard clause is
met by LLR-4tsgfh, through REQ-9ktgwx: every verification, adopting or not,
discards each received or answer provisional organisation below the
chain's epoch C, beyond C + 1, or at C without the chain's root and key,
and keeps one at C + 1 only until a verification more than the caller's
ahead limit (twice the app's verification interval) after the first
verification that saw it there. Before this, a store discarded only at an
adoption, which a store kept full never reached. This node's own proposals
and its provisional changes are exempt. No sender restriction is added.
A verification whose chain read is behind the store's own record discards
and stamps nothing (MR4 finding-2, owner ruling 2026-10-30): a record is
never ahead of the chain, so such a read is stale or faulty, cannot be
true, and throws no information away.

**Residual risk.** S1 / P2 — acceptable under the matrix. A false
provisional organisation reaches no user and no key. What remains:

- A store with no record (a new Member waiting for admission) keeps
  Organisation information and organisation answers from any Device that
  knows the Organisation identifier, listed or not; a store with a record
  keeps them only from a Device its record lists (an unlisted sender is
  refused, `SenderNotListed`). Such a sender can fill the store up to the
  1 MiB bound; while the bound is full, the genuine information or answer
  is refused with `ProvisionalLimit`.
- Each verification clears a flood: bodies that can never be adopted go at
  the first verification against the chain, and a body for the next epoch
  (C + 1) lives at most the ahead limit from its stamp — the first
  verification that saw it there — plus one verification interval.
- Any Device that knows the Organisation identifier, and keeps re-flooding
  once per ahead limit, can therefore hold off an admission indefinitely
  (and a listed Device, the adoption of a genuine update), and is not
  identified: no unit logs the sender, and nothing shows it to a user.
  The sender of each kept body is held in the store, in its origin
  (`Origin::Received { from }` or `Origin::Answer { from }`, LLR-s3vq6d),
  until a verification discards or adopts the body or the store itself is
  dropped (its Persona deleted), and org-io reports it for each kept body;
  a body counts as kept only once its store is written, since a failed
  write puts the store back as its file holds it; a refused body's sender
  is not kept (a problem
  report in org-io's problems file,
  `org-io/docs/problems/2026-10-24-domain-api.md`, asks for logging or
  showing the sender as a later improvement). The attack must be sustained
  for as long as it is to last.
- Through automatic answers to requests for the Organisation, a listed
  Member can make a Device send its record, and any sender makes a Device
  do the work of verifying.

The rating stays S1 / P2: each verification clears the flood, and the
harm is a delayed admission or update — no wrong record, no data loss, no
key exposure. *Residual accepted by the owner, 2026-10-31* (merge review
round 5 finding-1), including the sustained flood from a Device that is
not identified, which holds off an admission indefinitely.

*Amended in place 2026-10-31 (change `worktree-org-io-domain-api`, merge
review round 6, MR6-N1):* the third bullet said "nothing in this software
records or shows the sender". The "records" clause was added by the
dispatcher's task instructions, not by the owner, and was false: org-node
holds the sender of each kept body in its origin until a verification
discards or adopts the body, and org-io reports it for each kept body. The
bullet now states what the software does: no unit logs the sender, nothing
shows it to a user, and a refused body's sender is not kept. The owner's
accepted wording, "is not identified", and the acceptance of 2026-10-31
stand; no new ruling was needed.

*Amended in place 2026-10-31 (change `worktree-org-io-domain-api`, merge
review round 5 finding-1, owner ruling 2026-10-31):* the residual said the
sustained flood "comes from a Device whose key the logs show". That was
false: no unit logs the sender, and the app drops the sender of received
Organisation information (its receiver event carries none). The claim came
from the dispatcher's wording of the question put to the owner, not from
the owner, and the owner's acceptance of 2026-10-30 (MR4 finding-5) rested
on it. The claim is removed; the flooder is stated as not identified. The
owner re-accepted the residual without it on 2026-10-31, keeping S1 / P2,
and asked for a problem report on sender logging as a later improvement
(filed with org-io, where each body's sender is received:
`org-io/docs/problems/2026-10-24-domain-api.md`).

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1, rewritten for MR4 finding-5, owner ruling 2026-10-30):* the
first paragraph named only a listed Member, and the MR3-N1 note that
followed said a flooder "cannot block it for good" and measured a C + 1
body's life without saying from when. A flooder who keeps sending blocks
an admission for as long as it keeps sending, which has no bound, and the
life of a C + 1 body is measured from its stamp, not its arrival. The
paragraph above states both; the owner confirmed the S1 / P2 rating and
accepted the residual on 2026-10-30.

## A revocation discarded because another Admin's epoch came first

**HAZ-b5rudd**: the provisional changes an adopted epoch left out are
discarded automatically; an Admin proposes revoking a Member whose Device was
stolen, and before that Admin's write lands another Admin's epoch is adopted
without the revocation, which is discarded; the Admin believes the Member
revoked, and the Member — or whoever holds the stolen Device — keeps the
Organisation private key and every grant made to them.
Severity: S3. Probability: P1.

P1 because the demo has a single Admin, so no other Admin's epoch can arrive
first; with several Admins it would be P2.

**RC-dv6mka**: every provisional change an adopted epoch left out is reported
to the app when it is discarded, so the Admin is told that their revocation
was not applied and can propose it again.
mitigates: HAZ-b5rudd

What this control breaks: nothing; it adds a report.

**Residual risk.** S3 / P1: an Admin who misses the report leaves the Member
with access. Not acceptable under the matrix; **accepted by the owner
(2026-10-15)** while the demo has a single Admin. Carrying a left-out
revocation over to the new epoch, and Admins who disagree, are a later
roadmap stage.

## Acting on one's own removal

*Owner ruling Q4', 2026-10-16 (replacing ruling Q4 of the same day,
"verification deletes nothing", which the owner withdrew); recorded
2026-10-20.* After a Device verifies its own revocation it deletes at once,
with no user step, everything it holds for that Organisation except the
acknowledgements it must send, each deleted on acceptance and the rest 7
days after the verification. The two controls below supersede an earlier form of RC-5jkcxz
(`2026-09-09-org-node-hazards.md`) and an earlier form of RC-45k3zg
(`2026-10-07-commit-workflow.md`); the hazards they bear on, HAZ-vxabf9 and
HAZ-h6b34a, are unchanged in their text and re-estimated here.

**RC-5jkcxz**: on adopting through verification an epoch whose record no
longer lists its own Device key, or on accepting a revocation notice of its
own Device checked against the Membership root of the Organisation state
read from the chain — whether the send step pushed the notice or it came in
answer to the Device's organisation request — the node deletes at once, in
one sealed store image and without asking its user, every piece of data it
holds for that Organisation and every key the store holds for each Persona
bound to it, keeping only the acknowledgements RC-45k3zg keeps; verification
deletes nothing in any other case.
mitigates: HAZ-vxabf9

Implemented by REQ-48zc3e, with REQ-y9p27b keeping the single trigger
(RC-ub82my). Still a cooperative control, as an earlier form of RC-5jkcxz was: a Device that
never learns of its removal, or has been altered not to act on it, deletes
nothing. What it changes from an earlier form of RC-5jkcxz: the trigger is verification (at
start, on request and each periodic interval, the app's ledger) or a notice
the Device receives, including in answer to its own request, so a
cooperating Device that missed the single send learns of its removal within
one verification interval while it runs; and no user step stands between
learning and deleting.

**RC-45k3zg**: before that deletion the node signs each acknowledgement of
the removal with the Device secret key its caller gives it for that one
operation and keeps nowhere afterwards; it keeps for that Organisation only
the signed acknowledgements — three when the record before the removal lists
five or more Devices other than the Persona's own, two when three or four,
one when one or two, none when none, each for a distinct one of those
Devices — each with its recipient's DevicePublicKey and the endpoint
addresses kept for that key, and the time its caller gives as that of the
verification; it deletes an acknowledgement, with its recipient's key and
addresses, once the recipient answers it accepted; and whatever is left is
deleted by its caller, which removes the Persona's store file and keychain
item as soon as no acknowledgement is left and at the latest 7 days after
that time, delivered or not.
mitigates: HAZ-h6b34a

Implemented by REQ-48zc3e (what is kept), REQ-h29c8e (signing before the
deletion, the Device secret key transient), REQ-6tepxq (how many, for whom)
and REQ-ayyt5f (deletion on acceptance). The deadline and the removal of the
store file and keychain item are org-io's, named here by description;
org-node reads no clock. Sending each acknowledgement directly, under the
revoked Device's own endpoint identity, is org-io's as before, and a Member
accepts it only from the Device it names (RC-eydn8t's sender clause), which
is why the Persona's keychain item stays until the deadline.

### HAZ-vxabf9 re-estimated

What a cooperating removed Device keeps after RC-5jkcxz is no record, no
Change set, no Organisation private key of any epoch, no Member key and no
provisional change, organisation, invite, admission request or other
Device's address beyond those of its acknowledgements' recipients; the
store holds no Device secret key (the OS keychain holds it, as below). Only
the signed acknowledgements of its own removal remain, with at most three
recipients' DevicePublicKeys and addresses and the verification time, for at
most 7 days. None of that lets it receive or decrypt material published
after its removal, which is this hazard's harm, and none of it lists
another Member or names a person. The residual counts that keep this hazard
open are unchanged by this change: a Device that never learns of its removal
or is altered not to act on it (the control is cooperative), and the
Organisation private keys of the epochs up to the removal that such a Device
keeps. Severity S3, probability P1, as restated on 2026-10-07; S3 is
unacceptable at every probability under the matrix in
`org-node/docs/risk/README.md`. The owner's acceptance of 2026-10-07 for the
keys up to the removal still covers it: this change shortens the time a
cooperating Device takes to learn of its removal and removes the user step,
and adds nothing a removed Device keeps.

### HAZ-h6b34a re-estimated

What a revoked Device keeps to send its acknowledgements is now the signed
acknowledgements (the Organisation identifier, its MemberId and
DevicePublicKey, the epoch and root it was removed at), for at most three
recipients — fewer when it knows fewer Devices, none when it knows none —
each recipient's DevicePublicKey and the endpoint addresses kept for it, the
verification time, and, in the OS keychain, its own Device secret key, which
the sender rule requires for the direct connection. Against the residual
the owner accepted on 2026-10-07 (owner ruling R5: up to three recipients'
endpoint ids, the acknowledgement and the Device secret key, until three
accepted or the Organisation's retry limit, 2 months by default, passed) the
window is now at most 7 days and each acknowledgement goes as soon as its
recipient accepts it; the count is never larger. What is new is the
recipients' endpoint addresses (owner answer A2's address book), which
disclose where up to three Devices can be reached: a privacy matter, since a
connection completes only to the endpoint holding the dialled key and every
Member Device accepts nothing but an acknowledgement from the revoked
Device (RC-u7kdam). Whoever holds the lost or stolen Device within those 7
days can still connect to those recipients as the revoked Device and learn
that their keys belong to the Organisation. Severity S3, probability P1:
unacceptable under the matrix. The owner's acceptance of 2026-10-07 covered
a longer window with the same or more recipients but not the kept
addresses. **Accepted by the owner on 2026-10-20** for the demo stage, with
the window shortened further by the same day's ruling: the store file and
keychain item, and with them the Device secret key, go as soon as no
acknowledgement is left (all accepted, or none kept), 7 days being only the
upper bound. Benefit-risk case: without the kept acknowledgements a
revoked Device cannot tell the Organisation it cleared itself, and the
Organisation cannot tell a cleared Device from a silent one; the tiered
count limits what is held to what the Device's own Organisation size
warrants, and the 7-day bound ends the exposure whatever is delivered.

## A received Organisation private key

*Owner decision of 2026-10-20; recorded 2026-10-21.* Receipt reads no chain
(owner ruling R2 of 2026-10-15), so an earlier form of RC-5gbgwb
(`2026-10-07-org-key-pair.md`), which checks a received key against the
chain's `org_pub_key` on receipt, no longer describes the software. The
owner approved this successor, which splits the check between receipt and
verification.

**RC-5gbgwb**: on receipt, the node refuses the whole Wire message, leaving
the store unchanged, unless it carries an Organisation private key that
decodes as 32 bytes, and keeps as the received provisional organisation's
Organisation public key the X25519 public half derived from that private key,
never a public key stated beside it; at verification, that provisional
organisation is adopted only when that derived public key is the one the
chain holds for the Organisation, and otherwise nothing of it is adopted.
mitigates: HAZ-ep6uzs

*Amended in place 2026-10-21 (unmerged draft; owner decision of that day):*
the receipt half no longer compares the key with a public key the
provisional organisation states, because the message carries none and the
public key is derived from the private key, so that comparison could never
fail and no test could exercise it. Deriving it is what makes the chain match
at verification a check on the key actually received.

Implemented by REQ-zxgd46 (the receipt half, with no chain read; it
supersedes an earlier form of REQ-zxgd46, whose "before it reads the chain" assumed a chain
read at receipt) and REQ-dwb9bq (the verification half: a provisional
organisation is adopted only when its Organisation public key is the
chain's). What it breaks: as an earlier form of RC-5gbgwb — a genuine update relayed by a peer
that strips or alters the key is refused, or, if it decodes, is kept unused
and never adopted, and the update arrives again from the next correct sender
or is pulled at the next verification. No new hazard.

**HAZ-ep6uzs re-estimated.** Residual unchanged: S3, P1, the residual the
owner accepted on 2026-10-07 (first admission alone, as restated that day in
`2026-10-07-org-key-pair.md`). Reasoning: moving the chain match from receipt
to verification changes when a substituted key is caught, not whether it is
— nothing of a received provisional organisation is adopted, and so nothing
of its key becomes the record's key, without the match against the chain's
`org_pub_key`. Between receipt and verification a substituted key is only
held, encrypted in the store (RC-jjsz97) and bounded by RC-d4fc4s's limit,
and is discarded when another epoch is adopted.

## Derived requirements assessment

### Founding and proposing a Member

No new hazard. The founder's and a provisional Member's Member-as-a-group
key pair and Device key pair are drawn for one Organisation alone, as the
Persona's were, so the rating of the existing hazard of an Invite from a
non-Member (the keys in a reply expose nothing) still holds; the founding's
Organisation key pair is drawn under the existing distinctness rule.
assesses: REQ-4qh2h8, REQ-hx4r8y

### Batched epochs

No new hazard from the batching itself. An epoch's Change set, root and
Organisation key pair are verified against the chain exactly as one
provisional update's were, and the fresh key pair keeps the distinctness rule
of the item it supersedes. The hazards the batching does bring — a
revocation left out of another Admin's epoch, and false provisional
organisations — are HAZ-b5rudd and HAZ-cuj9gn above.
assesses: REQ-dhr58q, REQ-3pj7rd, REQ-x2dnee

### Received provisional organisations, the judgement and the bound

Controlled under HAZ-cuj9gn (RC-d4fc4s). The judgement keeps RC-ub82my: a
state behind the record, or at its epoch with another root, is still refused.

*Re-assessed 2026-10-20 (consistency pass; REQ-dwb9bq now also supersedes
an earlier form of REQ-dwb9bq):* an earlier form of RC-5gbgwb's refusal of a substituted Organisation private key
on receipt is no longer made there, because receipt reads no chain. Its
effect on HAZ-ep6uzs is kept at verification: the public key of a received
provisional organisation is derived from the private key it carries, and
only one whose public key is the chain's is adopted, so a substituted key is
kept unused until it is discarded and never becomes the record's key.
An earlier form of REQ-zxgd46 (a message carrying no key, or one that does not decode, refused
before anything is kept) still implements an earlier form of RC-5gbgwb. The control's own text,
which says the whole message is refused, now describes more than the code
does at receipt; its rating is unchanged, and restating it is left to the
owner.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* the owner restated the control: RC-5gbgwb supersedes
an earlier form of RC-5gbgwb, its receipt half implemented by REQ-zxgd46 (which supersedes
an earlier form of REQ-zxgd46) and its verification half by REQ-dwb9bq; HAZ-ep6uzs's residual
is unchanged (S3/P1, owner-accepted 2026-10-07), as stated under "A received
Organisation private key" above. REQ-zxgd46 adds no hazard: it refuses at
receipt, with no chain read, what an earlier form of REQ-zxgd46 refused, and a key whose public
half is not the provisional organisation's own stated key.
assesses: REQ-9ktgwx, REQ-dwb9bq, REQ-g2537c, REQ-zxgd46

### Persons, Members and Handles

No new hazard. The parse refusals keep RC-zutc67 and only add the bound on
preferred Handles. A Handle is a display identifier, not a key: an Admin's
choice when none is free is refused when another Member holds it, as every
duplicate Handle already is, and grants follow the MemberId, never the Handle.
assesses: REQ-kd34f5, REQ-2nnsmv

### Admission requests

Controlled under HAZ-8u3sv8 (RC-sp46zh, RC-amgwm9).
assesses: REQ-6qxesy, REQ-nbx4nn, REQ-f8p36u

### Requesting the Organisation

No hazard impact (the owner dropped a proposed hazard here, 2026-10-15): an
answer carries only the record, and the Organisation private key, of an epoch
whose record lists the requester, which the requester already held as a
Member of that epoch; an answerer whose current record does not list the
requester refuses it and answers nothing (owner ruling R7, 2026-10-15); and
the requester adopts nothing the chain does not hold. A stale answer costs a
round trip. The workload a listed Device can cause by requesting is part of
HAZ-cuj9gn's S1 residual. Refusing proposals while behind removes a way to
build on a stale record.
*Re-assessed 2026-10-22 (plan detail pass, dispatcher decision; LLR-e6w3nr
as amended):* a provisional Member now keeps an organisation answer for the
Organisation its expected admission names, from whichever Device delivered
it, while its behind marker is set. No new hazard: what is kept is adopted
only by verification against the chain (REQ-2yduet) and only when its
members list the provisional Member's own keys (LLR-q4puca), so a forged
answer is no match and adopts nothing; what it can cost is store space
within the bound of REQ-g2537c, which a provisional Member's received
Organisation information could already use.
assesses: REQ-p4hwuv, REQ-2yduet, REQ-akfwf3

### After S4: receiving, sending and the invitation

The one receive path (REQ-ncv4sc) now keeps and never adopts (owner rulings
R2 and R3, 2026-10-15): Organisation information from a listed Device and an
expected first admission become provisional organisations, which
verification alone adopts. This moves every adoption to verification and so
strengthens RC-d4fc4s on HAZ-cuj9gn: nothing received is acted on before the
chain is read at a finalised block. REQ-8amu2a's gate (RC-2ferct) stays in
front of the first admission, so a stranger still cannot make the node keep
a first admission it does not expect. REQ-eqt23j keeps RC-b6mydy in force:
a first admission is still accepted whichever Device key the connection
authenticated, and only what the chain has published is adopted, now with
the added requirement that the record lists the provisional Member's keys.
The admission request's verdicts ("already received", "refused because the
invite was used by another Device") tell a sender only what it already knows
about its own request; an impostor learns that the invite was used, which
adds no harm beyond HAZ-8u3sv8, already analysed. The Invite's addresses
(owner ruling R4) disclose the inviter's endpoint addresses to whoever sees
the pasted Invite: a privacy matter with no safety harm, since a send
delivers only to the endpoint whose identity is the Device key named. The
send builder and the narrowed Invite parse refusal change no control.
assesses: REQ-ncv4sc, REQ-eqt23j, REQ-ttv8k7, REQ-ky5tyd, REQ-9xf99q

### Re-assessed 2026-10-16: the revoked Device's request, the requester's addresses, the sender binding and the request discard

*Owner rulings Q3 and A2, dispatcher's resolutions C4 and C5, 2026-10-16.*

A Device held as revoked that asks for the Organisation is answered with its
revocation notice and nothing else (REQ-p4hwuv, REQ-ncv4sc): the notice is
what the send step already delivers to that Device when the removing epoch
is sent, it carries no record, no Organisation private key and no other
Organisation data, and it goes out through the send path to that Device's
key, never on the inbound connection. A Device no record has ever listed is
still refused. The revoked Device acts on the notice only by checking it
against the chain's root (REQ-2yduet, as REQ-m2xh8q accepts a revocation), so
a forged notice is refused. This adds a way for a removed Device to learn of
its removal, which narrows the app's hazard of an adopted epoch nobody
sends; no new hazard.

The sender's endpoint addresses (REQ-ky5tyd, REQ-6qxesy) disclose the
sender's addresses to the inviter, or to the Members it asks: a privacy
matter, as the Invite's addresses are (R4), with no safety harm, since a send
to them reaches only the endpoint whose identity is the Device key named. An
address is kept only from a request this node accepts; a refused request
keeps nothing.

Binding the admission request's DevicePublicKey to the connection's
authenticated key (REQ-6qxesy, C4) closes a way for one Device to request
admission for another key; it strengthens RC-sp46zh, whose "same Device, as
the transport authenticated it" now holds of the key the request names too.

REQ-x2dnee now also supersedes an earlier form of REQ-x2dnee (C5): no command discards a
provisional change or organisation on request; discarding is automatic only,
as already assessed under the batched epochs and in org-io's ledger. No new
hazard.
assesses: REQ-p4hwuv, REQ-2yduet, REQ-ncv4sc, REQ-ky5tyd, REQ-6qxesy, REQ-x2dnee

### Acting on one's own removal (owner ruling Q4')

REQ-48zc3e realises RC-5jkcxz and, with REQ-h29c8e, REQ-6tepxq and
REQ-ayyt5f, RC-45k3zg; their effect on HAZ-vxabf9 and HAZ-h6b34a is
assessed above. REQ-48zc3e and REQ-y9p27b keep RC-ub82my (HAZ-p6xkuz): the
removal still deletes only on a verified epoch or a revocation accepted
against the chain's root, and no refused message, behind or inconsistent
state, or call without chain state deletes anything. The one trigger
REQ-y9p27b adds, deleting a kept acknowledgement once its recipient answered
it accepted, acts only after the removal was verified and deletes only that
acknowledgement and its recipient's key and addresses; a wrong "accepted"
costs at most an acknowledgement not delivered, which is HAZ-h6b34a's
benefit side, not a wrong deletion of Organisation data. REQ-h29c8e is
an earlier form of REQ-h29c8e's signing unchanged, the refusal with nothing deleted when no
Device secret key is given included, so a missing keychain item still never
deletes a Device's data without its acknowledgements. REQ-6tepxq draws the
recipients from the record before the removal, which lists only Devices the
chain verified; which ones are chosen is not ruled and changes no rating.
REQ-35s4eb returns what is kept and changes nothing. Keeping the
verification time adds no exposure: it says when the Device learned of its
removal, which its acknowledgements already imply.
assesses: REQ-48zc3e, REQ-y9p27b, REQ-h29c8e, REQ-6tepxq, REQ-ayyt5f, REQ-35s4eb

### No panic on any received body (merge review round 2, MR2-N1, 2026-10-30)

REQ-rv4dwb supersedes an earlier form of REQ-rv4dwb and realises RC-e2uvje against HAZ-5f9jcm
on receipt. With `verify_envelope_against_chain` deleted, a received Change
set is applied on receipt: Organisation information from a Device the
current record lists is applied to that record, and in a store with no
record, from whichever Device sent it for an expected Organisation, to the
snapshot the body carries. That is the hostile Change set HAZ-5f9jcm names,
reached by any listed Device and, for a provisional Member, by any Device
that knows the Organisation identifier. The requirement adds no behaviour
and no exposure: it bounds the existing receipt to a typed error or an
outcome, with a refusal keeping nothing, and its evidence is a fuzz target
over that path. HAZ-5f9jcm's rating is unchanged: REQ-9g6as6 and REQ-rv4dwb
between them keep the control in force for decoding and for applying.
assesses: REQ-rv4dwb

## Derived low-level requirements of the design

Assessed 2026-10-15 for the derived items of
`org-node/docs/architecture/2026-10-16-domain-api.md`.

### Recording a founding's pure proxy

No hazard impact because recording the proxy and Organisation identifier
only names, in the founder's own store, the Organisation a provisional
founding was written to; it adopts nothing, so the founding becomes a
current record only through verification against the chain (RC-d4fc4s), and
a second, different proxy is refused rather than replacing the first, so a
retried or mistaken call cannot rebind the Persona to another Organisation
(REQ-yp75u9 holds).
assesses: LLR-nx87k5

### The pinned store plaintext

No hazard impact because the pin is a test oracle for the encoding: it
fixes the bytes of a store that holds the new epoch state, so an unintended
change of field order or encoding reddens the gate before a store written by
one build fails to open in another; it adds no behaviour.
*Re-assessed 2026-10-20 (owner answer A2, owner rulings Q3 and Q4'):* the
pin now also fixes the address book, the revoked Devices and the removed
store's form; still a test oracle with no behaviour of its own, so no hazard
impact.
assesses: LLR-mkvx2k

### Building an organisation request

No hazard impact because the request carries only the Organisation
identifier, which every Device the current record lists already holds, goes
only to those Devices, and changes nothing; what an answer may cause is
assessed under "Requesting the Organisation" above (adopted only through
verification, REQ-2yduet).
*Re-assessed 2026-10-20 (owner answer A2, dispatcher's resolution C3):* the
request now also carries the sender's own endpoint addresses, disclosed only
to the Devices it asks — a privacy matter assessed under the 2026-10-16
re-assessment above, with no safety harm — and a provisional Member's
request goes to its Invite's issuer alone, which answers only once its
current record lists the requester (REQ-p4hwuv). No hazard impact.
*Re-assessed 2026-10-22 (plan detail pass, dispatcher decision):* a
founding store whose proxy is not yet recorded is refused with
`FoundingNotWritten` instead of `OrgNotHeld`; a refusal builds and sends
nothing, so no hazard impact.
assesses: LLR-vyank2

### The Person store

No hazard impact because the Person store holds the user's own Person and
public keys, sealed exactly as a Persona store is (the same AEAD, nonce and
refusals) under a key derived with a distinct info string, so neither store's
bytes can be opened as the other; it holds no Organisation key or
Membership record, and naming its types under `org_node::` paths changes no
behaviour, only which unit org-io imports them from (no org-io → person
edge, owner ruling 2026-10-15).
*Re-assessed 2026-10-22 (plan detail pass, dispatcher decision;
LLR-wqu26s, LLR-73sgz2 as amended):* the Person store now also keeps the
Person key pair's secret seed, sealed under the Person's own Device seed as
a Persona's member seed is sealed in its store, and held under a redacting
secret type. No new hazard: no Organisation operation of this change
uses the Person key and the Person is never shown as verified, so
its disclosure would expose no Organisation data; the seed is protected as
the member seed is (the same AEAD, a key only that Device's seed derives),
and it shares the member seed's not-wiped-on-drop residual (RC-jjsz97,
owner ruling 2026-10-04), which this change does not alter.
assesses: LLR-73sgz2, LLR-5uf4u2

### Successors of dated items over removed operations (design gap G2, 2026-10-21)

Three derived items restate, over the operations of this change, dated
derived items whose operations it removes; each was assessed where it was
first written (`2026-10-03-architecture-derived.md`, sections C and E, and
a section F since removed as history) and none changes what those assessments rest on.

- **The Persona's status and binding** (LLR-exaa9u, after an earlier form of LLR-exaa9u): no
  hazard impact because a Persona becomes Active only at the first adoption
  in its own store, which follows verification against the chain (RC-d4fc4s),
  and its Organisation is fixed for the life of the store (REQ-yp75u9); a
  store holds one Persona, so no other Persona's status or binding can be
  written by mistake, the failure section F guarded against.
- **The proxy account held opaquely** (LLR-x5asbx, after earlier forms of LLR-x5asbx): no hazard impact because org-node derives
  nothing from the account and writes nothing to the chain; the account
  only tells org-io where to write the next epoch, and a wrong or missing
  one makes that write refused or its verification fail, which adopts
  nothing (RC-d4fc4s).
- **What a joining Device composes** (LLR-96z2k5, after an earlier form of LLR-96z2k5): no
  hazard impact because the one path is the parsed admission request, whose
  sender binding (REQ-6qxesy) and outstanding-invite check were assessed
  above under the admission request, and a Member it names joins a record
  only through an Admin's proposal and verification; the absence of any
  other import is compile-enforced.

assesses: LLR-exaa9u, LLR-x5asbx, LLR-96z2k5

### Successors and decisions of the plan detail pass (2026-10-22)

Two derived items restate, over the operations of this change, dated
derived items it removes or contradicts; each dated item was assessed where
it was first written (`2026-10-03-architecture-derived.md`, section C and
a dialling-address entry since removed as history).

- **A new Persona's status and binding** (LLR-t2adzg, after an earlier form of LLR-t2adzg): no
  hazard impact because a Persona is created Proposed with no member id and
  becomes Active, with its member id, only at the first adoption in its own
  store, which follows verification against the chain (RC-d4fc4s); a
  provisional Member's Persona is bound at creation to its Invite's
  Organisation, which is fixed for the life of the store (REQ-yp75u9), so
  it cannot be bound to another Organisation later.
- **Which values carry endpoint addresses** (LLR-ds228z, after
  an earlier form of LLR-ds228z): no hazard impact because org-node still dials nothing and
  names no transport address; the addresses it now holds and returns are
  the ones owner answer A2 of 2026-10-16 put in the Invite and the
  requests, a disclosure of where a Device can be reached to the Devices it
  deals with, a privacy matter assessed under the 2026-10-16 re-assessment
  above, with no safety harm. An address cannot impersonate a Device:
  org-node acts on what arrives by the Device key the connection
  authenticated (`sender`), never by an address.

assesses: LLR-t2adzg, LLR-ds228z

### The issuer's addresses kept at the first adoption (the resolved problem report on the issuer's addresses lost on first adoption, 2026-10-25)

LLR-q4puca and LLR-h9d65p, amended in place for the resolved problem report on the issuer's addresses lost on first adoption: a provisional
Member's first adoption moves the Invite issuer's DevicePublicKey and
addresses into the address book. No new hazard: the addresses are the ones
the Invite already disclosed to this Device and the store already held (the
privacy matter assessed under the 2026-10-16 re-assessment above); they are
kept under the issuer's key only, and a send to them still reaches only the
endpoint whose identity is that key, so org-io's identity check of the
Loopback recipient (its risk control and requirement on the dialled
endpoint's key, in org-io's ledgers) is unchanged. The change restores a newly admitted Member's pull
of the Organisation (owner answers Q2 and A2); no rating changes.
