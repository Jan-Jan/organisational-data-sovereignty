# Context

org-node is the node that carries an Organisation's membership between the
chain and its devices: it builds provisional updates, sends and receives
Change sets, decides whether to commit them by reading the chain, and keeps the
device's keys and record at rest. It does not write to the chain, and it has no
concept of an administrator: the right to change an Organisation lies in its
on-chain multisig proxy. It exists so that a device can hold a
membership record it verified itself, against the chain, rather than one it
was told.

## Language

Glossary of org-node's internal vocabulary. Definitions only — no
implementation details, no specs. Terms that cross a unit boundary
(Organisation, Member, PersonPublicKey, DevicePublicKey, Member-as-a-group key,
Membership record, Membership root, Change set, Organisation state,
Organisation public key, Finalised block) live in the root `docs/CONTEXT.md`
and are not repeated here.

**Envelope**:
The carrier of one Change set between devices: the Organisation it is for, its
Sequence number and the Change set bytes. It carries no signature: the
chain's root at a newer epoch decides, and nothing about the sender is
checked.
_Avoid_: message, packet, delta envelope

**Provisional update**:
A Change set and the Membership root it produces, built by the node for
creating an Organisation, admitting a Member or revoking one, that no record
holds yet. It becomes the Membership record only once the chain carries its
root and it has verified against the chain, on the node that built it as on
every other.
_Avoid_: pending update, draft, proposal

**Wire message**:
What one device sends another in one Receive operation: an Envelope, and
whatever else its kind carries; wider than the Envelope, because none of that
is part of it. It is of one of two kinds: *Organisation information*, which
also carries a snapshot of the membership as it stood before the Change set
and the Organisation private key, always; and *revocation*, which carries the
Envelope alone, no snapshot and no key. Only a first admission's receiver
reads the snapshot, to rebuild the record it is being admitted to; every other
receiver rebuilds from its own store. No Wire message carries an invite
identifier: a node waiting to be admitted matches a first admission against
the Organisation alone.
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
Organisation-information Wire message; a receiver keeps it only if its public
half is the Organisation public key the chain holds.
*Amended 2026-10-06 (change `worktree-org-node-org-key-pair`):* it was kept
only by the node that created the Organisation, and the Organisation secret
was something else.
_Avoid_: org key, Organisation key, Organisation secret, group secret

**Persona**:
One identity a device holds: a Member's keys and name, and the Organisation
the device belongs to under them. A device may hold several.
_Avoid_: account, profile, user

**Persona store**:
The encrypted file in which a device keeps its Personas, the records of the
Organisations they belong to, the provisional updates it has built and not
yet committed or discarded, and the admissions it has declared it expects.
_Avoid_: keystore, wallet, database

**Expected admission**:
An Organisation the app has declared, when its user confirms the reply to an
Invite, that this device expects a first admission to. A first admission to
an Organisation that matches none is refused before the chain is read.
*Amended 2026-10-07 (change `worktree-org-node-org-key-pair`, review round 1,
finding-1):* it was "an Organisation identifier and invite identifier"; the
expectation now names the Organisation alone.
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
(`app/docs/CONTEXT.md`) defines *Invite* and *Invite reply*.

**Receive operation**:
One acceptance of one Wire message by the node, from opening the connection to
committing or rejecting the Envelope it carries, including the node's own read
of the Organisation state from the chain.
The node has two, and requirements distinguish them: the one that admits the
node to an Organisation or updates its record of one, and the one that acts on
the node's own removal. They differ in what they may do to the record — the
first commits a membership it verified, the second deletes the node's own
record and revokes its Persona. Neither establishes anything about the
sender before acting (owner ruling, 2026-10-05).
_Avoid_: sync, pull, refresh
