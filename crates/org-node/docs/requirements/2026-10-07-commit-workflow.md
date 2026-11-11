# The commit workflow — org-node's part

Stage S3, change
`worktree-org-io-commit-workflow`. The workflow itself (write, read, retry,
fan-out, startup) is org-io's, and is minted in org-io's ledgers once stage
S2 creates that unit. The items below are the IO-free half: what org-node builds, verifies and
deletes. Every chain state named here is one the caller supplies, read from
the chain; org-node reads nothing itself once S2 has moved the chain read out.

Values in, values out (owner ruling B, 2026-10-06, in the roadmap's section
"Owner's rulings on the S2–S4 design drafts"): each requirement below takes
the Organisation state as a value and returns values — the outcome, the
messages to send, the acknowledgements and the sealed store bytes org-io
writes. "The store" in these items is the store state org-node holds and
returns sealed; "one sealed store image" (an earlier form of REQ-48zc3e) is one set of store
bytes, which org-io writes in one step; "leaving the store
unchanged" means org-node returns no new image and its held state is as
before. org-node has no IO traits and no async.

This change is written on top of change `worktree-org-node-org-key-pair`
(the Organisation key pair, a `WireMessage` of two kinds, a fresh key pair per
provisional update), which merged to master as `1f52c36` and was merged into
this branch on 2026-10-07.

The device secret key (owner rulings of 2026-10-06 recorded by stage S4 §13–§15, on branch
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
  Change set, no Organisation private key (the resolved problem report on the membership snapshot sent to a revoked device).
- A returning person gets a new identity; REQ-yp75u9 stays.
- Startup reconciles each record with the chain.

Owner rulings at the S3a close-out residual review (2026-10-07; decision 16): R1, the member-sender
rule (REQ-ea4qs5, an earlier form of REQ-uk9rw7 as amended; not first admission, not
acknowledgements); R2, the epoch rule unchanged (REQ-m2xh8q); R3 to R6, the
residuals of HAZ-p6xkuz, HAZ-7ubhwz, HAZ-h6b34a and HAZ-bedm57, recorded in
the risk files.

Amended in place by this change: an earlier form of REQ-x2dnee (keeps the Change set),
an earlier form of REQ-48zc3e (deletes everything, the Persona included), an earlier form of RC-5jkcxz, and, since
the key-pair change merged (2026-10-07), REQ-3dsweu (the revocation kind
carries the revocation of REQ-ps2gy2, not an Envelope; its "still listed"
refusal becomes REQ-m2xh8q's) and the LLRs that describe
`WireMessage::Revocation` (earlier forms of LLR-xr88a8, LLR-jhy6t6, LLR-95av3e, LLR-degg8t,
LLR-3sddey, and LLR-degg8t as amended there), in the files that define them.

## The revocation a revoked Device receives

**REQ-ps2gy2**: The software shall build, for each Device that the previous
record of an Organisation lists and the committed record does not, a Wire
message of the revocation kind holding only the Organisation identifier, that
Device's MemberId and DevicePublicKey, and an absence proof for that pair
produced from the committed record; it shall hold no Envelope, Change set,
record snapshot or Organisation private key.
implements: RC-r8bp43
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
unchanged.
implements: RC-ub82my
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
implements: RC-u7kdam
satisfies: derived

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place: a
clarification of who reads. "Before it reads the chain or uses any chain
state" reads "before it takes any chain state, so that none is read for it":
org-node no longer reads the chain, and the order of the read is stated in
org-io's architecture ledger
(`crates/org-io/docs/architecture/2026-10-08-org-io.md`).

REQ-ea4qs5 follows owner ruling R1 of 2026-10-07 (the member-sender rule,
decision 16):
a membership update and a revocation are acted on only when the sending
Device — authenticated by the transport as its DevicePublicKey, the iroh
endpoint id — is listed in the receiver's current committed record. The
owner's words: "Using iroh a connection can only be established via mutually
known public keys, but I agree with only accepting updates and revocations
from members. Furthermore, these are checked to be well formed before acting
on them, which gives us another layer of protection." And, on revocations
specifically: "the receiver still thinks of the other party as a member
(based on the information they have) so nothing is different here." The
update half is an earlier form of REQ-uk9rw7 as amended on 2026-10-07
(`crates/org-node/docs/requirements/2026-09-09-verify-and-commit.md`). The rule does
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
leaving the store unchanged.
implements: RC-eydn8t
satisfies: derived

*Amended 2026-10-07 (owner ruling at the S3a close-out residual review,
decision 16).* This
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
