# The commit workflow — org-node's part

Stage S3 of `docs/plans/2026-10-06-org-io-roadmap.md`, change
`worktree-org-io-commit-workflow`. The workflow itself (write, read, retry,
fan-out, startup) is org-io's and is proposed, unminted, in
`docs/plans/2026-10-06-org-io-commit-workflow.md` until stage S2 creates that
unit. The items below are the IO-free half: what org-node builds, verifies and
deletes. Every chain state named here is one the caller supplies, read from
the chain; org-node reads nothing itself once S2 has moved the chain read out.

Values in, values out (owner ruling B, 2026-10-06, in the roadmap's section
"Owner's rulings on the S2–S4 design drafts"): each requirement below takes
the Organisation state as a value and returns values — the outcome, the
messages to send, the acknowledgements and the sealed store bytes org-io
writes. "The store" in these items is the store state org-node holds and
returns sealed; "one sealed store image" (REQ-uxv2x2) is one set of store
bytes, which org-io writes in one step; "leaving the store
unchanged" means org-node returns no new image and its held state is as
before. org-node has no IO traits and no async.

This change is written on top of change `worktree-org-node-org-key-pair`
(the Organisation key pair, a `WireMessage` of two kinds, a fresh key pair per
provisional update), which merged to master as `1f52c36` and was merged into
this branch on 2026-10-07.

The device secret key (owner rulings of 2026-10-06 recorded by stage S4,
`docs/plans/2026-10-06-org-io-transport.md` §13–§15, on branch
`worktree-org-io-transport`): the device ed25519 seed lives only in the OS
keychain, org-node's store will hold only the Device public key, and org-node
receives the seed transiently — moved in for one operation, kept nowhere,
zeroised — to derive the store key and to sign. S4 makes that change to the
store; this change assumes neither layout. Where an item below signs, the
seed is given to it as a transient value, never read from the store.

Owner rulings this change carries (2026-10-06, recorded in the roadmap):

- There is no legacy trie. A device keeps the Change set that produced its
  current record, and no earlier one.
- A device deletes Organisation data only on a verified revocation: a
  committed record that no longer lists its Device, or an absence proof,
  each checked against the current on-chain root.
- "Delete everything" means every piece of data the device holds for the
  Organisation. As its last step the device sends, or answers with, an
  acknowledgement signed by its Device key; the receiver adds it to the
  revoked-device list (the list itself is stage S5).
- A revoked device receives only an absence proof: no record snapshot, no
  Change set, no Organisation private key (PR-qmvj83).
- A returning person gets a new identity; REQ-yp75u9 stays.
- Startup reconciles each record with the chain.

Owner rulings at the S3a close-out residual review (2026-10-07; decision 16
of `docs/plans/2026-10-06-org-io-commit-workflow.md`): R1, the member-sender
rule (REQ-ea4qs5, REQ-ztdza4 as amended; not first admission, not
acknowledgements); R2, the epoch rule unchanged (REQ-m2xh8q); R3 to R6, the
residuals of HAZ-p6xkuz, HAZ-7ubhwz, HAZ-h6b34a and HAZ-bedm57, recorded in
the risk files.

Amended in place by this change: REQ-uv3v5w (keeps the Change set),
REQ-uxv2x2 (deletes everything, the Persona included), RC-wqgm2p, and, since
the key-pair change merged (2026-10-07), REQ-3dsweu (the revocation kind
carries the revocation of REQ-ps2gy2, not an Envelope; its "still listed"
refusal becomes REQ-m2xh8q's) and the LLRs that describe
`WireMessage::Revocation` (LLR-js9dsu, LLR-pt32fx, LLR-38e2kn, LLR-6ymd6d,
LLR-j5vbqj, and LLR-8hdu9x as amended there), in the files that define them.

## The revocation a revoked Device receives

**REQ-ps2gy2**: The software shall build, for each Device that the previous
record of an Organisation lists and the committed record does not, a Wire
message of the revocation kind holding only the Organisation identifier, that
Device's MemberId and DevicePublicKey, and an absence proof for that pair
produced from the committed record; it shall hold no Envelope, Change set,
record snapshot or Organisation private key.
(implements: RC-r8bp43)
satisfies: derived

**REQ-qrtsc9**: The software shall refuse with a typed error, before it uses
any chain state and leaving the store unchanged, a received revocation that
does not decode, that names an Organisation it holds no record of, or whose
MemberId and DevicePublicKey are not those of a Persona bound to that
Organisation.
satisfies: derived

**REQ-m2xh8q**: The software shall accept a received revocation only when its
absence proof verifies, for the MemberId and DevicePublicKey of a Persona
bound to the Organisation, against the Membership root of the Organisation
state the caller supplies as read from the chain, and only when that state's
epoch is not lower than its record's epoch; it shall refuse every other
revocation with a typed error that says which check failed, leaving the store
unchanged. (implements: RC-ub82my)
satisfies: derived

*Owner ruling R2 of 2026-10-07 (S3a close-out residual review):* the epoch
rule stands as written — the proof is checked against the chain's current
root, and a chain state older than the record is refused
(`StaleChainState`). No change.

## Who a revocation came from

**REQ-ea4qs5**: The software shall act on a received revocation only when the
DevicePublicKey the transport authenticated for the connection that delivered
it is listed in a member snapshot of its current committed record of the
Organisation the revocation names; it shall refuse any other revocation with a
typed error naming the Organisation, before it reads the chain or uses any
chain state and without obtaining any Device secret key, leaving the store
unchanged.
(implements: RC-u7kdam)
satisfies: derived

REQ-ea4qs5 follows owner ruling R1 of 2026-10-07 (the member-sender rule,
recorded in `docs/plans/2026-10-06-org-io-commit-workflow.md`, decision 16):
a membership update and a revocation are acted on only when the sending
Device — authenticated by the transport as its DevicePublicKey, the iroh
endpoint id — is listed in the receiver's current committed record. The
owner's words: "Using iroh a connection can only be established via mutually
known public keys, but I agree with only accepting updates and revocations
from members. Furthermore, these are checked to be well formed before acting
on them, which gives us another layer of protection." And, on revocations
specifically: "the receiver still thinks of the other party as a member
(based on the information they have) so nothing is different here." The
update half is REQ-ztdza4 as amended on 2026-10-07
(`org-node/docs/requirements/2026-09-09-verify-and-commit.md`). The rule does
not cover a first admission: a new joiner accepts the admitting update from
any sender and verifies it against the chain (owner amendments of
2026-10-07: "to avoid scenarios where something happens to the admin's
device during this window"; "The invite id plays no role in the update";
"the new joiner has no org information to disclose, and they verify the org
information they receive on-chain so the risk here is only a new joiner
being DoS'ed which is acceptable"). An acknowledgement has a sender rule of
its own, since it comes from a Device the receiver no longer lists: "Acknowledgements of
revocations will be sent from devices on the revocation list, so their ids
can be checked before accepting" (REQ-b462sh as amended 2026-10-07).

## Deletion only on a verified revocation

**REQ-em28bq**: The software shall delete data it holds for an Organisation
only when it accepts a revocation (REQ-m2xh8q) or commits a Membership record,
verified against an Organisation state its caller supplies as read from the
chain, that lists the DevicePublicKey of no Persona bound to that
Organisation; no other input — a refused message, a chain state at a newer
epoch that it has not verified a record or proof against, a call that
supplies no chain state, a discarded provisional update — shall delete any of
it.
(implements: RC-ub82my)
satisfies: derived

**REQ-y99c9w**: The software shall, when it deletes an Organisation's data on
a verified revocation, first produce for each Persona bound to that
Organisation an acknowledgement naming the Organisation, the Persona's
MemberId and DevicePublicKey, and the epoch and Membership root of the chain
state the revocation was verified against, signed under a domain of its own
with that Persona's Device secret key, which its caller gives it as a
transient value for that one operation; it shall refuse with a typed error,
deleting nothing, when the caller gives no Device secret key for a Persona
bound to that Organisation; and it shall return the acknowledgements to its
caller and keep none of them, nor any Device secret key it was given, in the
store or anywhere else once the operation returns.
(implements: RC-44vvjp)
satisfies: derived

*Amended 2026-10-07 (owner rulings of 2026-10-06 recorded by stage S4,
`docs/plans/2026-10-06-org-io-transport.md` §13 point 4 and §14, on branch
`worktree-org-io-transport`).* This said the acknowledgement is signed "with
that Persona's Device secret key", read from the store. The device seed lives
only in the OS keychain, and org-node receives it transiently for signing;
whether the store still holds it (today) or holds only the Device public key
(after S4), this requirement takes it from its caller. The transient value's
own type, move-only and zeroised, is S4's; whichever of S3 and S4 merges
second makes this signing take that type.

## The acknowledgement a receiver checks

**REQ-b462sh**: The software shall accept a received acknowledgement only when
it decodes, names an Organisation it holds a record of, was delivered by the
Device it names (the DevicePublicKey the transport authenticated for the
connection equals the acknowledgement's DevicePublicKey), names a
DevicePublicKey its current record does not list, names an epoch not greater
than its record's epoch, and carries a signature that verifies under the
acknowledgement's domain and the DevicePublicKey it names; it shall check the
sender before the signature, and refuse every other acknowledgement with a
typed error that says which check failed, without using any chain state and
leaving the store unchanged. (implements: RC-eydn8t)
satisfies: derived

*Amended 2026-10-07 (owner ruling at the S3a close-out residual review,
decision 16 of `docs/plans/2026-10-06-org-io-commit-workflow.md`).* This
accepted an acknowledgement from any endpoint. The owner: "Acknowledgements
of revocations will be sent from devices on the revocation list, so their
ids can be checked before accepting." Until stage S5's revocation list, "on
the revocation list" is read as "named by the acknowledgement and absent from
this node's current record"; a node that has not yet committed the
revocation still lists that Device and refuses (the revoked Device then tries
another Member Device, the plan's proposed 8). An acknowledgement relayed by
any other Device is refused; S5's gossip of the list carries acknowledgements
between peers, and there each peer checks the signature (owner ruling R4).

## Reconciling a record with the chain

**REQ-tb4f8p**: The software shall, given the Organisation state the caller
read from the chain for an Organisation it holds a record of, report exactly
one outcome: in step, when the state's epoch and Membership root equal its
record's; committed, when a provisional update it holds has the state's
Membership root and Organisation public key, which it then commits as a
commit of its own update does; behind, when the state's epoch is greater and
no provisional update matches, leaving the store unchanged; and it shall
refuse with a typed error, leaving the store unchanged, a state whose epoch is
lower than its record's or equal to it with a different Membership root.
(implements: RC-ub82my)
satisfies: derived

REQ-tb4f8p is the IO-free half of PR-vt244s's fix: a node that crashed after
the chain write and before its commit finds the written update among its kept
provisional updates at its next start, and commits it from there. Which update
it commits is decided by the chain alone, so the node commits the update the
chain holds whichever the caller last submitted. That is the first half of
the app's report of an update that is never committed or discarded after a
write whose outcome is unknown; `docs/plans/2026-10-06-org-io-commit-workflow.md`
(proposed 2, 4 and 5) names the report and carries its other halves to
org-io.
