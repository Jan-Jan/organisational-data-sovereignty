# Context

org-node is the node that carries an Organisation's membership between the
chain and its devices: it builds provisional updates, sends and receives
Change sets, adopts one only when a Verification against the Organisation
state org-io read from the chain matches it, and keeps the device's keys and
record at rest. It reads and writes no chain itself, and it has no concept of
an administrator: the right to change an Organisation lies in its on-chain
multisig proxy. It exists so that a device can hold a membership record it
verified itself, against the chain, rather than one it was told.
*Amended 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 3, finding-9):* it said org-node decided whether to commit "by reading
the chain"; org-io reads the chain and hands org-node the Organisation state
(owner ruling R2 of 2026-10-15).

## Language

Glossary of org-node's internal vocabulary. Definitions only — no
implementation details, no specs. Terms that cross a unit boundary
(Organisation, Member, PersonPublicKey, DevicePublicKey, Member-as-a-group key,
Membership record, Membership root, Change set, Organisation state,
Organisation public key, Finalised block, Ahead limit) live in the root `docs/CONTEXT.md`
and are not repeated here.

**Envelope**:
The carrier of one Change set between devices: the Organisation it is for, its
Sequence number and the Change set bytes. It carries no signature: the
chain's root at a newer epoch decides, and nothing about the sender is
checked. *(2026-10-07, owner ruling R1: for an Organisation the node holds,
the sender must be a Device its current record lists; a first admission
still checks nothing about the sender.)*
_Avoid_: message, packet, delta envelope

**Provisional update**:
A Change set and the Membership root it produces, built by the node for
creating an Organisation, admitting a Member or revoking one, that no record
holds yet. It becomes the Membership record only once the chain carries its
root and it has verified against the chain, on the node that built it as on
every other.
*Replaced 2026-10-15 (change `worktree-org-io-domain-api`):* an Admin's
admissions and revocations are now Provisional changes, calculated together
into one Provisional organisation per epoch, and a founding is a Provisional
founding (root glossary).
_Avoid_: pending update, draft, proposal

**Wire message**:
What one device sends another in one Receive operation: an Envelope, and
whatever else its kind carries; wider than the Envelope, because none of that
is part of it. It is of one of six kinds: *Organisation information*, which
carries an Envelope, a snapshot of the membership as it stood before the
Change set and the Organisation private key, always; *revocation*, which
carries a revocation notice (the revoked Device's identity and an absence
proof) and no Envelope, snapshot or key; a *Removal acknowledgement*; an
*admission request*; an *organisation request*, by which a Device that is
behind asks for the Organisation; and the *organisation answer* to it, which
carries the current record's epoch, its members as a snapshot and its
Organisation private key. Only the admission request carries an invite
identifier: a node waiting to be admitted matches a first admission against
the Organisation alone.
*Amended 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 3, finding-9):* it said a Wire message was of one of two kinds, a
revocation carried an Envelope, and no Wire message carried an invite
identifier; the change added the acknowledgement, the admission request, the
organisation request and its answer (LLR-xr88a8).
*Amended 2026-10-07 (change `worktree-org-node-org-key-pair`, review round 1,
finding-1):* it said every Wire message carried the snapshot and the invite
identifier of the admission it carried, and that a waiting node matched the
invite identifier; a revocation now carries neither snapshot nor key, and the
invite identifier never travels between peers.
_Avoid_: message, packet, payload

**Sequence number**:
The epoch of the Organisation state the Change set an Envelope carries
reaches. *Amended 2026-10-05 (review round 2, finding-13):* it was "the
position of an Envelope in the order its author issued them for one
Organisation" until REQ-txvtm9 bound it to the chain's epoch.
*Amended 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 3, finding-9):* REQ-txvtm9 is superseded by REQ-dwb9bq; on receipt the
Sequence number is checked against the epoch after the held record, not
against the chain (LLR-76bhw9).
_Avoid_: seq, parent_seq (the field name), nonce

**Published signing key**:
Retired 2026-10-05. The Organisation state records the Organisation public key
(root `docs/CONTEXT.md`), an X25519 key-agreement key, and no Envelope is
signed; the term survives only in the dated notes of items amended in place
that day (REQ-ag6kqm, REQ-nhe2zu, RC-pm9kmx).
_Avoid_: using it for the Organisation public key
(*Merged 2026-10-05 with master `1feb608`, whose glossary defines this term as
the administrator's Member-as-a-group key held in the Organisation public key
field (PR-szkat6), and adds an "Organisation public key" entry saying the
field holds it today. Neither holds on this branch: the field holds the
Organisation's own X25519 key (REQ-ech45n), defined in the root
`docs/CONTEXT.md`.*)

**Device key**:
The name requirements written before 2026-10-05 use for a DevicePublicKey
(root `docs/CONTEXT.md`): the ed25519 public key of a device, its identity in
the Membership record and on the transport. Text written from 2026-10-05 on
says DevicePublicKey. The term survives in master's items, among them
REQ-xa6smf, REQ-ztdza4 and RC-b6mydy, which are not reworded for a change of
term.
_Avoid_: using it in new text; DevicePublicKey is the term

**Organisation private key**:
The X25519 secret behind the Organisation public key (root
`docs/CONTEXT.md`): one value per epoch, shared by the whole Organisation, so
no node has a key of its own. An administrator's node draws a fresh one at
genesis and whenever it calculates the root of a provisional update, and the
chain publishes its public half; every node keeps the Organisation's key in its Persona store and
passes that same key to every Member's Devices with every
Organisation-information Wire message and with every organisation answer; a
receiver keeps it with the provisional organisation it came in, and it
becomes the record's key only when a Verification adopts that provisional
organisation because its public half is the Organisation public key the
chain holds (LLR-8528ud).
*Amended 2026-10-06 (change `worktree-org-node-org-key-pair`):* it was kept
only by the node that created the Organisation, and the Organisation secret
was something else.
*Amended 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 3, finding-9):* it said a receiver kept the key only if it matched the
chain's; receipt reads no chain, so the match is made at Verification.
_Avoid_: org key, Organisation key, Organisation secret, group secret

**Persona**:
One identity a device holds: a Member's keys and name, and the Organisation
the device belongs to under them. A device may hold several.
_Avoid_: account, profile, user

**Persona store**:
What a device keeps of one Persona: the Persona, the record of the
Organisation it belongs to, the Provisional changes and Provisional
organisations held for that Organisation's next epoch (or its Provisional
founding, or its Provisional Member), the admission requests, revocation
notices and addresses it keeps, and the admissions it expects. One per
Persona. org-node seals it as a SealedStore, which org-io writes as that
Persona's Store file (root `docs/CONTEXT.md`); after the Persona's own
removal it becomes a Removed store (root `docs/CONTEXT.md`).
*Amended 2026-10-28 (change `worktree-org-io-domain-api`, ARCHITECTURE
review round 3, AR3-8):* it was "the encrypted file in which a device keeps
its Personas, the records of the Organisations they belong to, the
provisional updates it has built and not yet committed or discarded, and the
admissions it has declared it expects". A store now holds one Persona,
holds Provisional changes and Provisional organisations, and is a value
org-node seals; the file is org-io's.
_Avoid_: keystore, wallet, database, persona store file

**Expected admission**:
An Organisation the app has declared, when its user confirms the reply to an
Invite, that this device expects a first admission to. In a store with no
record, Organisation information or an organisation answer for an
Organisation that matches none is refused on receipt, decoded no further
than its kind and Organisation.
*Amended 2026-10-07 (change `worktree-org-node-org-key-pair`, review round 1,
finding-1):* it was "an Organisation identifier and invite identifier"; the
expectation now names the Organisation alone.
*Amended 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 3, finding-9):* it said a non-matching admission was refused "before
the chain is read"; receipt reads no chain (LLR-e6w3nr).
_Avoid_: pending invite, invitation

*Organisation secret* was defined here until 2026-10-06, when the change
`worktree-org-node-org-key-pair` replaced it with the Organisation private
key: an opaque value shared by Members, unrelated to the Organisation public
key and authenticated by nothing (PR-szkat6, PR-ve9zw8, PR-xwek5e).

**Secret**:
A value whose holder can act as someone else: a member seed, a device seed,
the Organisation private key, or the key the Persona
store is encrypted under.
Never shown in diagnostic output; given up only where it is deliberately used.
_Avoid_: key material, private key (ambiguous with the public half)

*Invite* and *Join request* were defined here until 2026-10-05, when the
invitation exchange left org-node for the app; the app's glossary
(`app/docs/CONTEXT.md`) defines *Invite*, and the root glossary
*Admission request*, which replaced the app's *Invite reply* (2026-10-28).

**Receive operation**:
One check of one Wire message by the node, from the message a connection
delivered and the DevicePublicKey the transport authenticated to what the
node does with it: keep it (a Provisional organisation, an admission
request, an organisation answer, a revocation notice), answer it (an
organisation request), accept it (a Removal acknowledgement) or refuse it.
It reads no chain state and never changes the Membership record: only a
Verification (root `docs/CONTEXT.md`) adopts a membership or acts on the
node's own removal. It establishes nothing about the sender beyond the
authenticated DevicePublicKey (owner ruling, 2026-10-05).
*Amended 2026-10-28 (change `worktree-org-io-domain-api`, ARCHITECTURE
review round 3, AR3-8):* it included the node's own read of the Organisation
state from the chain, and had two kinds, one committing a membership it
verified and one acting on the node's own removal. The receive path is now
chain-free and never adopts (owner ruling R2 of 2026-10-15).
_Avoid_: sync, pull, refresh

**First-ahead stamp**:
The Verification time of the first Verification that found a Provisional
organisation another Device sent at the epoch after the chain's. No later
Verification moves it, and the Ahead limit (root `docs/CONTEXT.md`) is
counted from it, not from the organisation's arrival. This node's own
proposals are never stamped. org-node's field is `first_ahead_at`.
(Added 2026-10-30, change `worktree-org-io-domain-api`, merge review MR4
finding-7.)
_Avoid_: arrival time, received at, expiry
