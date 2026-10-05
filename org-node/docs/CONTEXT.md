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
(Organisation, Member, Device key, Member-as-a-group key, Membership record,
Membership root, Change set, Organisation state, Finalised block) live in the
root `docs/CONTEXT.md` and are not repeated here.

**Envelope**:
The signed carrier of one Change set between devices: the Organisation it is
for, its Sequence number, the Change set bytes, and a signature over all three
by the published signing key.
_Avoid_: message, packet, delta envelope

**Wire message**:
What one device sends another in one Receive operation: an Envelope, a
snapshot of the membership as it stood before the Change set, and, on
admission only, the Organisation secret. Wider than the Envelope, because
neither the secret nor the snapshot is covered by the Envelope's signature.
The snapshot is set on both paths the node sends on — admission and
revocation — but only an admission's receiver reads it, to rebuild the record
it is being admitted to; the receiver of a revocation rebuilds from its own
store and ignores the snapshot it was sent.
_Avoid_: message, packet, payload

**Sequence number**:
The position of an Envelope in the order its author issued them for one
Organisation.
_Avoid_: seq, parent_seq (the field name), nonce

**Published signing key**:
The key recorded in the Organisation state as the Organisation's signing key,
under which every Envelope for that Organisation must verify. Today it is the
administrator's Member-as-a-group key, held in the Organisation state's
Organisation public key field — a conflation recorded as PR-szkat6.
_Avoid_: org key, admin key

**Organisation public key**:
The public half of the Organisation key pair: agreed by the administrators,
published in the Organisation state, and shared with every current Member, so
that access can be granted to the Organisation as a pseudo-group without
running CGKA across all Members. By owner ruling (recorded in the `person`
unit) it is an X25519 key; the field holds the Published signing key today
(PR-szkat6).
_Avoid_: org key, org_pub_key (the field name)

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
the Organisation secret, or the key the Persona store is encrypted under.
Never shown in diagnostic output; given up only where it is deliberately used.
_Avoid_: key material, private key (ambiguous with the public half)

**Invite**:
What an administrator gives a prospective Member out of band so that the
Member's device can recognise the administrator's device when admission
arrives: the Organisation, its published signing key and the administrator's
Device key.
_Avoid_: invitation code, link, ticket

**Join request**:
What a prospective Member's device gives the administrator out of band so that
the administrator can admit it: the Member's name and Handle, their
Member-as-a-group key and Device key, and where the device can be reached.
_Avoid_: application, enrolment request, JoinRequest (the type name)

**Receive operation**:
One acceptance of one Wire message by the node, from opening the connection to
committing or rejecting the Envelope it carries, including the node's own read
of the Organisation state from the chain.
The node has two, and requirements distinguish them: the one that admits the
node to an Organisation or updates its record of one, and the one that acts on
the node's own removal. They differ in what they may do to the record — the
first commits a membership it verified, the second deletes the node's own
record and revokes its Persona — and therefore in what each must establish
about the sender before acting.
_Avoid_: sync, pull, refresh
