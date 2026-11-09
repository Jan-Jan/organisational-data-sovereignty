# Risk analysis — the commit workflow (org-node)

Run 2026-10-06 for change `worktree-org-io-commit-workflow` (stage S3), under IEC 62304 Class C and the
acceptability matrix in this ledger's README. The requirements are in
`org-node/docs/requirements/2026-10-07-commit-workflow.md`;
the org-io half of the workflow is org-io's. This analysis builds on
change `worktree-org-node-org-key-pair` (a fresh key pair per update;
`WireMessage` of two kinds), merged to master as `1f52c36` and into this
branch on 2026-10-07.

Residuals below are stated as severity and probability. The README's matrix
governs: S3 is unacceptable at every probability, P1 included (owner ruling D
of 2026-10-06 on the S2–S4 design drafts). No S3 residual in this file is
therefore "acceptable under the matrix". Each one that remains is stated
with the benefit-risk case for accepting it rather than refusing the
function that carries it. The owner reviewed each residual individually at
the S3a close-out residual review on 2026-10-07 (rulings R1 to R6, decision
16); each ruling is
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
sender rule is RC-eydn8t's.
mitigates: HAZ-bedm57, HAZ-p6xkuz, HAZ-ep6uzs

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place:
"before it reads the chain" reads "before it takes any chain state, so that
none is read for it" — org-node no longer reads the chain; the order of
org-io's read is stated in org-io's architecture ledger. The risk reasoning
is unchanged.

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
an earlier form of REQ-uk9rw7 (as amended 2026-10-07) and REQ-ea4qs5. *(Added 2026-10-07,
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
revoked Device's own Member (org-members REQ-yyuxh8).
mitigates: HAZ-vxabf9

What it changes for HAZ-vxabf9. Before S3 the revocation carried the
committed Envelope. The key-pair change already removed the member snapshots
and the key from it, but the Change set stayed: with batched updates it
carries every other leaf the batch upserts, and the revoked Device could
verify it only from the record the Change set was based on, while the chain
still held that update's root (an earlier form of REQ-dwb9bq). A Device that missed one update
could therefore never verify its own revocation. An absence proof verifies
against whatever root the chain holds now, so a Device any number of epochs
behind can act on it. This closes the resolved problem report on the membership snapshot sent to a revoked device.

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
refuses to reconcile against such an older state.
mitigates: HAZ-p6xkuz

This is the consumer's half of org-members' caller duty §12 and of the
verification control in `org-members/docs/risk/2026-10-06-absence-proofs.md`
(exported as REQ-65xrgh): org-members verifies a proof against the root it is
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
`reconcile` already did (an earlier form of LLR-rhsfw9 as amended, an earlier form of LLR-rr558z). At an equal
epoch only the record's own root passes, and under it the proof is refused,
so a revocation is acted on only against a chain state of a later epoch.
The control's statement above is unchanged; the deletion path is now no
laxer than reconcile.

org-node takes the chain state as a value its caller read (ruling B,
2026-10-06); it reads no chain itself. A caller that cannot reach the chain
supplies no state, and with no state org-node deletes and commits nothing:
the device keeps its last verified record, which org-io reports to the app as
unverified (owner ruling of 2026-10-06 on the plan's decision 5).

*Assessed 2026-10-13 (S4, task T9;
owner ruling of that day, option A): `OrgService::restore_store` and
`OrgService::drop_store` against this control.* org-io writes each store
change to the Persona's store file. When that write fails, it puts the
Persona back as the file holds it (`restore_store`). When it cannot do even
that, or when the keychain refuses a new Persona's secret, it takes the
Persona's store out of the in-memory service (`drop_store`). Neither is a
trigger this control has to refuse:
- Neither is reachable from a received message, a chain state or any peer
  input. Each runs only after a local IO failure in org-io: a refused
  keychain write for a new Persona, which holds no Organisation data and has
  no file, or a failed reload after a failed store write.
- Neither deletes anything on disk. The store file and the keychain item
  stay, and the Persona returns on the next `OrgIo::open`.
- `restore_store` adds or replaces one store with the sealed copy on disk;
  it removes no store.
So the data the device holds, the sealed stores on disk (an earlier form of REQ-y9p27b's
clarification of 2026-10-13), is deleted only as before. The exempt removal
places are named in the superseding item (an earlier form of LLR-9jn3z8), and the `absences`
scan pins them. What it breaks: after a failed reload, the Persona is
missing from the running node until the next open. That is the conservative
direction, since a store that cannot be reread is not acted on. No new
hazard, and no change to HAZ-p6xkuz's ratings or to this control's
effectiveness.

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
  — chosen by the chain's root and Organisation public key (an earlier form of REQ-dwb9bq) —
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
an earlier form of REQ-dwb9bq). Benefit-risk case put to
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
the revocation.
mitigates: HAZ-7ubhwz

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
Device's statement, not evidence of erasure; an earlier form of RC-5jkcxz is cooperative and so
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

## Derived requirements assessment

**REQ-ps2gy2** realises RC-r8bp43; assessed above.

**REQ-qrtsc9** (a revocation that does not decode, names an unheld
Organisation, or names another Device is refused before any chain state is
used): keeps HAZ-bedm57 where RC-mj6gjq put it. *(2026-10-07: with
REQ-ea4qs5 a stranger no longer costs a chain read at all; only a Device the
record lists reaches the read.)* No new hazard.

**REQ-ea4qs5** (a revocation is acted on only from a Device the current
record lists, before any chain read) and **an earlier form of REQ-uk9rw7** (amended 2026-10-07:
an update for a held Organisation is committed only from such a Device)
realise RC-u7kdam; assessed above under "A message is acted on only when a
member sent it", including what they break. No new hazard: the refusal
writes nothing and reads no chain, so its worst case is a genuine message
from a Device the receiver's record does not yet list, which waits for a
Device both records list (a delay, not a loss). An earlier form of REQ-eqt23j is unchanged (a
first admission from any sender), and its residual — a new joiner held up by
strangers — is owner-accepted (R1 and R6, 2026-10-07).

**REQ-m2xh8q, earlier forms of REQ-y9p27b and REQ-dwb9bq** realise RC-ub82my; assessed above.
An earlier form of REQ-dwb9bq's commit chooses the provisional update by the chain's root and
Organisation public key, as `commit_update` does after the key-pair change,
so a reconcile commits only what the chain holds. Its "behind" outcome
deletes and commits nothing (an earlier form of REQ-y9p27b).

**An earlier form of REQ-h29c8e** realises an earlier form of RC-45k3zg; assessed above. Taking the Device secret
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

**An earlier form of REQ-x2dnee** (amended: the record keeps the Change set that produced it):
the kept Change set holds the leaves it upserted, which is Member data
already in the record it produced; it lives in the encrypted store
(RC-jjsz97), is bounded by the 1 MiB frame, and is sent to nobody in this
change (stage S6 sends it only to a requester in good standing). No new
hazard.

**An earlier form of REQ-48zc3e** (amended: deletion covers everything, the Persona with its
keys included, on either verified path): narrows HAZ-vxabf9 and HAZ-45ucqx on
a cooperating revoked Device, which no longer keeps keys that could act as
it. What it breaks: a wrong deletion is now unrecoverable, which is why
HAZ-p6xkuz rates S3 and why RC-ub82my and RC-u7kdam are the only doors to it.

assesses: REQ-ps2gy2, REQ-qrtsc9, REQ-m2xh8q, REQ-b462sh, REQ-3dsweu, REQ-ea4qs5

*Added 2026-10-08 (change `worktree-org-io-create`, owner's hybrid ruling on
the resolved problem report on amendment in place instead of supersession).* **REQ-uk9rw7** supersedes the item it replaced: it states the rule
that item carried after its 2026-10-07 amendment (an update for a held
Organisation is committed only from a Device the current record lists),
under its own ID, with ruling B's wording of the order ("before it takes any
chain state, so that none is read for it"). It realises RC-u7kdam as
that item did, and the assessment above holds for it unchanged: the refusal
writes nothing and asks for no chain state, and its worst case is a delay.
No new hazard.

assesses: REQ-uk9rw7
