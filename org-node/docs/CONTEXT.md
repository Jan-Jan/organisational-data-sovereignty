# Context

org-node is the node that carries an Organisation's membership between the
chain and its devices: it receives Change sets from peers and decides whether
to commit them, publishes new Membership roots to the chain, and keeps the
device's keys and record at rest. It exists so that a device can hold a
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

**Wire message**:
What one device sends another in one Receive operation: an Envelope, a
snapshot of the membership as it stood before the Change set, and, on
admission only, the Organisation secret. Wider than the Envelope, because
neither the secret nor the snapshot is part of it.
The snapshot is set on both paths the node sends on — admission and
revocation — but only an admission's receiver reads it, to rebuild the record
it is being admitted to; the receiver of a revocation rebuilds from its own
store and ignores the snapshot it was sent.
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
`docs/CONTEXT.md`), drawn for one Organisation alone and kept by the node
that created it, in its Persona store. It is not the Organisation secret.
_Avoid_: org key, Organisation key, Organisation secret

**Persona**:
One identity a device holds: a Member's keys and name, and the Organisation
the device belongs to under them. A device may hold several.
_Avoid_: account, profile, user

**Persona store**:
The encrypted file in which a device keeps its Personas, the records of the
Organisations they belong to, and the Invites it has imported.
_Avoid_: keystore, wallet, database

**Organisation secret**:
An opaque value the administrator hands to a Member at admission, shared by
every current Member. What it protects is the next capability's concern. The
design intends it as the secret half of the Organisation key pair; the node
does not yet relate it to the Organisation public key (PR-szkat6).
_Avoid_: org key, group secret, shared key

**Secret**:
A value whose holder can act as someone else: a member seed, a device seed,
the Organisation secret, the Organisation private key, or the key the Persona
store is encrypted under.
Never shown in diagnostic output; given up only where it is deliberately used.
_Avoid_: key material, private key (ambiguous with the public half)

**Invite**:
What an administrator gives a prospective Member out of band before
admission: the Organisation, its Organisation public key, the administrator's
Member-as-a-group key and the administrator's DevicePublicKey. None of it is
checked against the admission when it arrives; the joiner records the
administrator's Member-as-a-group key from it.
_Avoid_: invitation code, link, ticket

**Join request**:
What a prospective Member's device gives the administrator out of band so that
the administrator can admit it: the Member's name and Handle, their
Member-as-a-group key and DevicePublicKey, and where the device can be reached.
_Avoid_: application, enrolment request, JoinRequest (the type name)

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
