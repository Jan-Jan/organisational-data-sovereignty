# Risk analysis — the commit workflow (org-node)

Run 2026-10-06 for change `worktree-org-io-commit-workflow` (stage S3 of
`docs/plans/2026-10-06-org-io-roadmap.md`), under IEC 62304 Class C and the
acceptability matrix in this ledger's README. The requirements are in
`org-node/docs/requirements/2026-10-07-commit-workflow.md`;
the org-io half of the workflow is proposed, unminted, in
`docs/plans/2026-10-06-org-io-commit-workflow.md`. This analysis builds on
change `worktree-org-node-org-key-pair` (a fresh key pair per update;
`WireMessage` of two kinds), merged to master as `1f52c36` and into this
branch on 2026-10-07.

Residuals below are stated as severity and probability. The README's matrix
governs: S3 is unacceptable at every probability, P1 included (owner ruling D
of 2026-10-06, `docs/plans/2026-10-06-org-io-roadmap.md`, section "Owner's
rulings on the S2–S4 design drafts"). No S3 residual in this file is
therefore "acceptable under the matrix". Each one that remains is stated
with the benefit-risk case for accepting it rather than refusing the
function that carries it. The owner reviewed each residual individually at
the S3a close-out residual review on 2026-10-07 (rulings R1 to R6, decision
16 of `docs/plans/2026-10-06-org-io-commit-workflow.md`); each ruling is
recorded below beside the residual it rules on, and the chain-authority
residual's (R6) beside it in `org-node/docs/risk/2026-10-06-chain-authority.md`.

## A message is acted on only when a member sent it

**RC-u7kdam**: the node acts on a received Organisation-information message
about an Organisation it holds a record of, and on a received revocation,
only when the DevicePublicKey the transport authenticated for that connection
(the iroh endpoint id) is listed in a member snapshot of its current
committed record of that Organisation; it makes this check after the message
decodes and the record is found, before it reads the chain, decodes the
Change set or obtains a Device secret key, and refuses without writing. It
does not apply to a first admission, which has no record to check against
and is verified against the chain alone, nor to an acknowledgement, whose
sender rule is RC-eydn8t's. mitigates: HAZ-bedm57, HAZ-p6xkuz, HAZ-ep6uzs

Owner ruling R1 (2026-10-07), with the owner's words: "Using iroh a
connection can only be established via mutually known public keys, but I
agree with only accepting updates and revocations from members. Furthermore,
these are checked to be well formed before acting on them, which gives us
another layer of protection." On revocations: "the receiver still thinks of
the other party as a member (based on the information they have) so nothing
is different here." On a first admission (owner amendments the same day): a
new joiner accepts the admitting update from any sender, "to avoid scenarios
where something happens to the admin's device during this window"; "The
invite id plays no role in the update"; "the new joiner has no org
information to disclose, and they verify the org information they receive
on-chain so the risk here is only a new joiner being DoS'ed which is
acceptable." That last residual — a new joiner held up by strangers sending
it first-admission updates, each costing one chain read and refused unless
the chain carries it — is owner-accepted on 2026-10-07 on that reasoning
(recorded with HAZ-bedm57 in `org-node/docs/risk/2026-10-06-chain-authority.md`).

What it changes. For HAZ-bedm57, a stranger can no longer make a node that
holds a record read the chain: the refusal comes before the read. For
HAZ-p6xkuz, a revocation, and an Organisation-information message that
removes the node, now has to come from a Device the node itself still counts
as a Member, so a deletion needs both a listed sender and a proof (or a
record) that verifies against the chain (RC-ub82my). Implemented by
REQ-ztdza4 (as amended 2026-10-07) and REQ-ea4qs5. *(Added 2026-10-07,
review round 1 finding-4.)* For HAZ-ep6uzs, a non-member's message can no
longer shape the record of a held Organisation: it is refused before
anything is decoded; what remains of that hazard is first admission alone,
restated in `org-node/docs/risk/2026-10-07-org-key-pair.md` (HAZ-ep6uzs).

What it breaks. A relay that is not in the receiver's record can no longer
deliver a genuine update or revocation; under iroh the Devices that talk to a
Member are its fellow Members anyway, and stage S4's fan-out sends from the
committing Device. A Device several epochs behind accepts only from Devices
its own, older record lists: a Member admitted after its last commit cannot
deliver to it, and it waits for a Device both records list. A revoked
Member's Device that relays its own removal is still listed in the
receiver's record and is accepted, as before. A stranger's message that
fails the sender check is refused as `SenderNotListed` (plan task T12a)
rather than by the check that refused it before, which changes which error
some existing tests see.

## Owner ruling R2 — the epoch rule

*Owner ruling R2 (2026-10-07):* keep the current rule — a revocation's proof
is checked against the chain's current root, and a chain state older than
the device's record is refused (`StaleChainState`). RC-ub82my and REQ-m2xh8q
stand unchanged.

## A revoked Device is told by a proof, not by the update

**RC-r8bp43**: a Device the committed record no longer lists is sent only an
absence proof for its own MemberId and DevicePublicKey, produced from the
committed record: no Change set, no record snapshot and no Organisation
private key, so the proof carries no data of any Member other than the
revoked Device's own Member (org-members REQ-yyuxh8). mitigates: HAZ-vxabf9

What it changes for HAZ-vxabf9. Before S3 the revocation carried the
committed Envelope. The key-pair change already removed the member snapshots
and the key from it, but the Change set stayed: with batched updates it
carries every other leaf the batch upserts, and the revoked Device could
verify it only from the record the Change set was based on, while the chain
still held that update's root (REQ-txvtm9). A Device that missed one update
could therefore never verify its own revocation. An absence proof verifies
against whatever root the chain holds now, so a Device any number of epochs
behind can act on it. This closes PR-qmvj83.

What it breaks: nothing a correct node does. The revoked Device learns that
it is out and nothing else.

## Acting on a revocation that is not current

**HAZ-p6xkuz**: a revocation is acted on that does not reflect the current
membership — an absence proof checked against a chain state older than the
device's record (a lagging or hostile chain endpoint), a proof for another
Device, or a deletion triggered by anything other than a verified revocation
(a refused message, an unreachable chain, a chain epoch the device cannot yet
verify); the device deletes everything it holds for the Organisation, its
Persona and Device key included, while its Member still holds that Device
key; the member cannot reach organisational records when a decision needs
them, and the Device cannot be enrolled again with the same keys.
Severity: S3. Probability: P2.

P2: revocations arrive from any peer, and a device that wakes after a long
absence routinely reads a chain endpoint that may lag. *(2026-10-07: from
plan task T12a, only from a peer the device's record lists, RC-u7kdam; the
rating before controls is unchanged.)*

**RC-ub82my**: the node deletes Organisation data only on a committed
Membership record, or an absence proof for its own MemberId and
DevicePublicKey, verified against an Organisation state from the chain whose
epoch is not lower than its record's; it refuses every other trigger, and it
refuses to reconcile against such an older state. mitigates: HAZ-p6xkuz

This is the consumer's half of org-members' caller duty §12 and of the
verification control in `org-members/docs/risk/2026-10-06-absence-proofs.md`
(exported as REQ-tk2qqj): org-members verifies a proof against the root it is
given; org-node makes sure the root is not older than what the device
already knows. An equal epoch is accepted because the chain changes its root
only with a new epoch: at its record's own epoch a listed Device has no valid
proof of absence.

*(Corrected 2026-10-07, review round 2 finding-1.)* That reasoning assumed
the chain's root at the record's epoch is the record's root. A chain state
at that epoch with another root — which only a forged or faulty chain read
supplies — would have passed the epoch check and had the proof checked
against its root, so a removal could be acted on against a state the record
contradicts. Under owner ruling R3 (false deletion extremely unlikely)
`revocation::accept` now refuses that state as `ChainStateConflict`, as
`reconcile` already did (LLR-tx8ruv as amended, LLR-gr8x3r). At an equal
epoch only the record's own root passes, and under it the proof is refused,
so a revocation is acted on only against a chain state of a later epoch.
The control's statement above is unchanged; the deletion path is now no
laxer than reconcile.

org-node takes the chain state as a value its caller read (ruling B,
2026-10-06); it reads no chain itself. A caller that cannot reach the chain
supplies no state, and with no state org-node deletes and commits nothing:
the device keeps its last verified record, which org-io reports to the app as
unverified (owner ruling of 2026-10-06 on the plan's decision 5).

What it breaks: a device whose record is ahead of the chain endpoint it reads
cannot act on a genuine revocation until the endpoint catches up; it keeps
its data meanwhile. That is the direction the owner's ruling prefers (delete
only when verified).

Residual, restated 2026-10-07 with RC-u7kdam in place. *(Corrected
2026-10-07, review round 1 finding-5: this paragraph and the sentence after
the ruling below said a wrong deletion needs "delivery by a Device the
device's own record lists" and that the code imposes "three independent
conditions for any deletion: a listed sender…". That holds of a received
message only. `commit_update` and `reconcile` reach the removal step with no
sender at all, so the residual names both routes. Owner ruling R3 below is
unchanged and stands.)* A deletion is reached by
one of two routes:

- (a) **A received message.** A revocation notice: it must come from a
  sender the device's own record lists as a Member (RC-u7kdam; iroh
  authenticates the endpoint id, so a wrong one means that Member Device
  itself, or its key, is compromised), decode, and carry an absence proof
  for the device's own MemberId and DevicePublicKey that verifies against
  the chain's current root, at an epoch not lower than the record's (the
  epoch rule, RC-ub82my, kept by R2). An Organisation-information message
  whose committed record no longer lists the device passes the same sender
  check and decode, and its root must match the chain's at that update's
  epoch (RC-b6mydy, RC-ub82my). A wrong deletion by this route needs both a
  compromised listed Member Device and a forged chain state at least as new
  as the record.
- (b) **The node's own commit or reconcile** (`commit_update`, `reconcile`
  through `commit_step`). No sender is involved. The precondition is that the
  chain carries a provisional update this node itself built and still holds
  — chosen by the chain's root and Organisation public key (REQ-tb4f8p) —
  and that the record it commits no longer lists the device; reconcile also
  refuses a chain state older than the record (RC-ub82my). A wrong deletion
  by this route needs a forged chain state carrying the root of an update
  this node built that removes its own Device and the chain never took; an
  update the node built that removes itself is its own Member's decision.

Each route ends in a chain state that passes on-chain-client's verifier —
control of the chain endpoint, that unit's boundary, not this one's.
Severity S3, probability P1: unacceptable under the README's matrix
(ruling D).

*Owner ruling R3 (2026-10-07): residual accepted*, on the owner's ordering of
the two failure modes: "a device only deletes everything after they verified
the revocation proof against the on-chain information (from a peer that is as
far as they know still a member of the org). If forced to choose between
failure modes it is better for a device to delete everything about and
related to an org, than it is for it to retain any information after it has
been revoked, BUT the code should ideally make this failure if not impossible
then extremely unlikely." The code meets the "extremely unlikely" half by
independent conditions on each route above: on route (a), three — a listed
sender (RC-u7kdam), a well-formed message (decode), and a proof or record
that verifies against a chain state not older than the record (RC-ub82my);
on route (b), two — a provisional update the node itself built and holds,
and a chain state that carries it, not older than the record (RC-ub82my,
REQ-tb4f8p). Benefit-risk case put to
the owner before the ruling:
the alternative to acting on a verified revocation is never deleting on one,
which leaves every lost or stolen Device holding the Organisation's records
and keys indefinitely (HAZ-vxabf9, S3, P2 and the ordinary case); the
residual needs a forged chain state that passes on-chain-client's verifier,
the boundary every other trust decision in this unit already rests on, so
refusing deletion would trade a common S3 harm for protection against one
already outside this unit's reach.

## The acknowledgement

**HAZ-7ubhwz**: an acknowledgement the named Device did not produce — forged,
or bound to a Device the record still lists — is accepted by a node, or
merged from a peer's copy of the revoked-device list, and added to that list;
a Device still in the record is treated as revoked by every peer that merges
the list, or a Device that never deleted anything is recorded as having done
so; a member loses access to organisational records, or the Organisation
believes a revoked Device cleared that did not. Severity: S3. Probability: P2.

*Revised 2026-10-07 (owner ruling R4).* The hazard said the falsely
acknowledged entry "is pruned under the shorter retention limit for
acknowledged entries, after which its key is outside the never-again rule".
The owner ruled that pruning is time-based and independent of
acknowledgement ("Pruning happens a certain amount of time regardless of
acknowledgement"), so an acknowledgement shortens no entry's life and that
route is gone. It also names the list's gossip, where the owner ruled each
peer checks the signature.

P2: before 2026-10-07 acknowledgements were accepted from any endpoint, by
design (a revoked Device may no longer be reachable under its own identity).

**RC-eydn8t**: the node accepts an acknowledgement only when the Device the
transport authenticated for the connection is the Device the acknowledgement
names, checked before the signature; when the DevicePublicKey it names is not
in the node's current record; and when its signature verifies, under the
acknowledgement's own domain, with that DevicePublicKey, the signed
statement binding the Organisation, the MemberId and the epoch and root of
the revocation. mitigates: HAZ-7ubhwz

*Amended 2026-10-07 (owner ruling at the S3a close-out residual review).*
The sender clause is new: "Acknowledgements of revocations will be sent from
devices on the revocation list, so their ids can be checked before
accepting." Until stage S5's list, a Device "on the revocation list" is one
the acknowledgement names and the node's current record does not list. The
revoked Device therefore sends under its own identity, not from a fresh
endpoint (the plan's proposed 8 as revised the same day), and keeps its
Device secret key until delivery (see HAZ-h6b34a below).

**Constraint on stage S5 (owner ruling R4, 2026-10-07).** "Upon gossiping a
CRDT update to the revocation list containing an acknowledgement, peers
should verify the signature before accepting." Every peer that merges a
revocation-list update carrying an acknowledgement verifies that
acknowledgement's signature, as RC-eydn8t's last clause does (org-node's
`check_acknowledgement`), and drops an update carrying one that does not
verify. A gossiped acknowledgement reaches a peer through other peers, so the
sender clause does not apply there; the signature does. And: "Pruning happens
a certain amount of time regardless of acknowledgement" — S5's list prunes an
entry by its age alone; there is no shorter retention for acknowledged
entries. That the never-again rule for a revoked Device key ends when its
entry is pruned, for every entry alike, is S5's to assess.

What it does not cover, and cannot: a revoked Device altered to sign the
acknowledgement without deleting anything. The acknowledgement is the
Device's statement, not evidence of erasure; RC-wqgm2p is cooperative and so
is this. No retention rule may treat an acknowledgement as proof that the
data is gone, and by ruling R4 none does. A signature by a key that was
never a Member's adds an entry only for that key, which its own holder
chose; no other party is affected.

Residual, restated 2026-10-07: S3, P1 — a revoked Device altered to sign
without deleting is recorded as acknowledged, and the Organisation believes
it cleared; its entry lives exactly as long as an unacknowledged one, since
pruning ignores acknowledgement. A third party can neither forge nor relay
an acknowledgement into a node (sender clause) nor into the gossiped list
(signature). Unacceptable under the README's matrix (ruling D). *Owner
ruling R4 (2026-10-07): residual accepted, with the control changed as
above.* The owner's words: "Upon gossiping a CRDT update to the revocation
list containing an acknowledgement, peers should verify the signature before
accepting. Change the risk file accordingly. Pruning happens a certain
amount of time regardless of acknowledgement." Benefit-risk case put to the
owner before the ruling: the acknowledgement is what lets the Organisation
tell a Device that was told and cleared from one that never answered;
without it every revoked entry is equally unknown.

## What the revoked Device keeps to send its acknowledgement

**HAZ-h6b34a**: the state a revoked Device needs to deliver its
acknowledgement — its Device secret key, the acknowledgement, the addresses
of the peers to send it to — outlives the deletion of everything else on a
lost or stolen Device; whoever holds it learns other Members' Devices (each
address names a DevicePublicKey, which is the iroh endpoint id) and can act
as the revoked Device toward them; the identities reach whoever holds the
Device. Severity: S3. Probability: P2.

P2: a lost or stolen Device is the ordinary case of a revocation
(HAZ-vxabf9).

**RC-44vvjp**: the node signs each acknowledgement before it deletes the
Organisation's data, with the Device secret key its caller gives it for that
one operation and keeps nowhere afterwards; deletes the Persona with every
key the store holds for it in the same sealed store image as the rest; and
returns the acknowledgements to its caller holding no key and no other
Organisation data, leaving in the store no tombstone, marker or copy of an
acknowledgement for that Organisation. mitigates: HAZ-h6b34a

*Revised 2026-10-07 (owner rulings of 2026-10-06 recorded by stage S4,
`docs/plans/2026-10-06-org-io-transport.md` §13–§15 on branch
`worktree-org-io-transport`).* This said the node deletes "the Persona's
Device and Member secret keys" in the store image. The device seed lives only
in the OS keychain and org-node is given it transiently; after S4 the store
holds the Device public key, not the seed. The Device secret key is
destroyed by org-io deleting the Persona's keychain item, in the same step as
it writes the store image and before it sends anything (the plan's proposed
7). Until S4, the seed is in the store and goes with the Persona.

*Ruling B (2026-10-06):* org-node writes no file. "The same sealed store
image" is the one set of store bytes org-node returns, which org-io writes
in one step; a caller that wrote the acknowledgements without that image
would still hold the keys, so org-io's proposed 7 writes the image before it
sends anything.

With RC-44vvjp, org-node keeps nothing: `forget_organisation` leaves no
tombstone, marker or copy of an acknowledgement (LLR-pba7yu). What the caller
(org-io) keeps to deliver the acknowledgement, and for how long, was ruled by
the owner on 2026-10-06 (ruling C, and the plan's decision 1 as corrected by
the owner's ruling on retention: "Once the device has deleted all the local
org related data and sent back the signed acknowledgement, it should keep
nothing with regards to that org anymore."), and revised by the owner on
2026-10-07 (ruling R5 and the acknowledgement sender rule, below). These
become org-io requirements once S2 creates the unit (the plan's proposed 7,
8 and 11), with org-io's own control for this hazard minted there.

*Revised 2026-10-07 (owner ruling R5 and the acknowledgement sender rule,
S3a close-out residual review).* This said the caller keeps only the
acknowledgement, the address of the one peer that delivered the revocation
and the retry limit, retries from a fresh endpoint identity, and that the
Device secret key is gone before the first send. Two rulings change that:

- R5: "After delivering the acknowledgement, the local device should delete
  all the information. To be sure of delivery, the device delivers the
  acknowledgement to 2 more peers (if they have as many peers)." The device
  delivers to up to three distinct Member Devices in all — the one that
  delivered the revocation and up to two more — fewer if fewer exist.
- The sender rule (RC-eydn8t as amended): "Acknowledgements of revocations
  will be sent from devices on the revocation list, so their ids can be
  checked before accepting." A receiver accepts an acknowledgement only over
  a connection authenticated as the Device it names, so the revoked Device
  sends under its own endpoint identity, never from a fresh one, and needs
  its Device secret key until delivery ends.

What the caller keeps, therefore: the acknowledgement, the Device secret key
(the Persona's keychain item, used only to open these connections), the
DevicePublicKeys (iroh endpoint ids) of up to three distinct Member Devices
taken from the record before it is deleted — the delivering peer first — and
the retry limit. It connects directly to each in turn, no relay, so each
receiver sees the revoked Device as the sender; a peer that has not yet
committed the revocation still lists the Device and refuses
(`AcknowledgementForListedDevice`), and the device tries the next retained
address. When three have accepted, or every retained address has been tried
and refused or the Organisation's acknowledgement retry limit has passed
(set by the Organisation, 2 months by default, ruling C), whichever comes
first, it deletes all of it — the Device secret key, the acknowledgement,
the addresses and the limit — and from then on holds nothing about the
Organisation. Everything else is deleted before the first send, as before
(proposed 7). A Member Device must accept an inbound connection from a
Device it holds as removed, for an acknowledgement only; that is a transport
requirement for S4 and S3b-io (the plan's proposed 9).

Residual, restated 2026-10-07 with that in place: between the deletion and
the end of delivery — never longer than the retry limit — a lost or stolen
revoked Device holds up to three other Devices' endpoint ids (each a
DevicePublicKey, which names no person by itself), its own signed
acknowledgement (the Organisation identifier, its own MemberId and the root
it was revoked at), and its own Device secret key. With that key its holder
can connect to those Devices as the revoked Device, and every Member Device
refuses from it anything but an acknowledgement: it is listed in no
receiver's record, so its updates and revocations fail RC-u7kdam, and its
key was removed from the record, so it reaches no Organisation data. The
window ends at delivery to three or at the limit. Severity S3, probability
P1: unacceptable under the README's matrix (ruling D). *Owner ruling R5
(2026-10-07): residual accepted* on the delivery design above, bounded by
the Organisation's limit (ruling C, default 2 months). The owner's words:
"After delivering the acknowledgement, the local device should delete all
the information. To be sure of delivery, the device delivers the
acknowledgement to 2 more peers (if they have as many peers)." Benefit-risk
case put to the owner before the ruling: without the retained addresses and
acknowledgement a revoked Device that loses its connection mid-send can
never report that it cleared itself, and the Organisation cannot tell a
cleared Device from a silent one; three deliveries make the report survive
the loss of one peer, at the cost of two more endpoint ids held for the same
bounded window. An Organisation that weighs the exposure higher sets a
shorter limit.

*Owner ruling 2026-10-07 on the key window (S3a close-out).* Because an
acknowledgement is accepted only from the Device it names (RC-eydn8t's
sender clause), the revoked Device keeps its Device secret key — the
Persona's OS keychain item — until delivery to three peers ends or the
Organisation's limit expires, whichever comes first, and then deletes it
with everything else listed above. Until then, whoever holds the lost or
stolen Device holds that key and can connect as the revoked Device, from
which every Member Device accepts nothing but an acknowledgement
(RC-u7kdam). *Residual accepted by the owner, 2026-10-07*, as part of the
residual restated above.

## Derived requirements assessment

**REQ-ps2gy2** realises RC-r8bp43; assessed above.

**REQ-qrtsc9** (a revocation that does not decode, names an unheld
Organisation, or names another Device is refused before any chain state is
used): keeps HAZ-bedm57 where RC-mj6gjq put it. *(2026-10-07: with
REQ-ea4qs5 a stranger no longer costs a chain read at all; only a Device the
record lists reaches the read.)* No new hazard.

**REQ-ea4qs5** (a revocation is acted on only from a Device the current
record lists, before any chain read) and **REQ-ztdza4** (amended 2026-10-07:
an update for a held Organisation is committed only from such a Device)
realise RC-u7kdam; assessed above under "A message is acted on only when a
member sent it", including what they break. No new hazard: the refusal
writes nothing and reads no chain, so its worst case is a genuine message
from a Device the receiver's record does not yet list, which waits for a
Device both records list (a delay, not a loss). REQ-xa6smf is unchanged (a
first admission from any sender), and its residual — a new joiner held up by
strangers — is owner-accepted (R1 and R6, 2026-10-07).

**REQ-m2xh8q, REQ-em28bq, REQ-tb4f8p** realise RC-ub82my; assessed above.
REQ-tb4f8p's commit chooses the provisional update by the chain's root and
Organisation public key, as `commit_update` does after the key-pair change,
so a reconcile commits only what the chain holds. Its "behind" outcome
deletes and commits nothing (REQ-em28bq).

**REQ-y99c9w** realises RC-44vvjp; assessed above. Taking the Device secret
key from the caller for one operation (revised 2026-10-07) adds no exposure:
org-node holds the seed no longer than the signing, and a caller that gives
no seed gets a refusal with nothing deleted, so a missing keychain item never
deletes a Device's data without its acknowledgement. (That refusal means a
Device whose keychain item is lost keeps its Organisation data until the item
is restored or the Persona is removed by hand; S4's hazard of a lost
keychain item, whose residual the owner accepted on 2026-10-06, already
covers the item being lost.) That org-io keeps the keychain item until
delivery ends (R5) is org-io's retention, assessed under HAZ-h6b34a above.
*(Qualified 2026-10-07, review round 1 finding-8.)* "Holds the seed no
longer than the signing" is true of the store, not of process memory: until
stage S4, `DeviceSeed` is not wiped on drop and the removal paths clone it
(and the whole store data), so copies may remain in freed memory. That
interim limitation is stated under HAZ-45ucqx
(`org-node/docs/risk/2026-09-09-org-node-hazards.md`), with S4 as the stage
that closes it; it is not owner-accepted.

**REQ-3dsweu** (amended 2026-10-07: the revocation kind carries only the
revoked Device's notice, sent only to a Device the update removed; the "still
listed" refusal moves to REQ-m2xh8q): narrows HAZ-vxabf9 by RC-r8bp43, as
assessed above. The relabelling the key-pair change guarded against — a relay
turning Organisation information into a revocation to hold a listed Device on
the previous epoch's key — is no longer expressible: a revocation carries no
Envelope to relabel, and an absence proof for a Device the chain's record
lists does not verify. No new hazard.

**REQ-b462sh** realises RC-eydn8t; assessed above. It uses no chain state, so
an acknowledgement costs a stranger no chain read; since 2026-10-07 one from
any Device but the one it names is refused before the signature check.

**REQ-uv3v5w** (amended: the record keeps the Change set that produced it):
the kept Change set holds the leaves it upserted, which is Member data
already in the record it produced; it lives in the encrypted store
(RC-jjsz97), is bounded by the 1 MiB frame, and is sent to nobody in this
change (stage S6 sends it only to a requester in good standing). No new
hazard.

**REQ-uxv2x2** (amended: deletion covers everything, the Persona with its
keys included, on either verified path): narrows HAZ-vxabf9 and HAZ-45ucqx on
a cooperating revoked Device, which no longer keeps keys that could act as
it. What it breaks: a wrong deletion is now unrecoverable, which is why
HAZ-p6xkuz rates S3 and why RC-ub82my and RC-u7kdam are the only doors to it.

assesses: REQ-ps2gy2, REQ-qrtsc9, REQ-m2xh8q, REQ-em28bq, REQ-y99c9w, REQ-b462sh, REQ-tb4f8p, REQ-uv3v5w, REQ-uxv2x2, REQ-3dsweu, REQ-ea4qs5, REQ-ztdza4
