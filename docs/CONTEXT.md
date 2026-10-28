# Context

Organisational Data Sovereignty (ODS) is a two-tier access-control system: an
organisation's membership is recorded in a hash-committed structure anchored
on-chain, and a local-first collaboration layer derives document access from
that membership. It exists so that an organisation controls who can read its
data without depending on a service operator to enforce it.

## Language

Glossary of the project's ubiquitous language. Definitions only — no
implementation details, no specs.

**Partial.** These are the terms settled while writing the first requirements
(`org-members/docs/requirements/`, moved there from `docs/requirements/` on
2026-09-05), which is why they cluster around membership. The remaining
vocabulary — already well developed in `org-members/AGENTS.md`, and the
authority for crate-level terms until then — is migrated at tooth 9 of
`docs/plans/2026-08-26-ratchet-gap-analysis.md`.

**Scope since 2026-09-05.** This is the repository's *interface* glossary: the
terms that cross a unit boundary (`.guardrails/units.yaml`). Each unit keeps
its own `<unit>/docs/CONTEXT.md` for internal vocabulary, and a term is
escalated here the moment it appears in an exported requirement or an
`expects:` item. Every term below is interface vocabulary — the membership
record and its parts are exactly what org-node and the app consume from
org-members — so nothing moved down when the unit glossaries were created.

**Organisation**:
The entity that owns a membership record and whose data is being governed.
Singular per membership record.

**Member**:
A person belonging to an Organisation.
_Avoid_: user, account

**Handle**:
The human-readable identifier a Member is known by within an Organisation.
Mutable, and changes rarely. Personally identifying, and redacted when
displayed in diagnostics.
_Avoid_: username, nickname, alias

**MemberId**:
The immutable 32-byte identifier of a Member. Independent of the Handle and of
every key, so that a grant made to a Member survives a rename. Chosen at random
when the Member is placed in the Membership record; identifiers in Membership
records made before 2026-10-03 may equal key bytes and are opaque either way.
_Avoid_: node id, member key

**PersonPublicKey**:
The key a grant is encoded against, and from which the holder's devices derive
their own access: an X25519 public key resulting from group key agreement among
those devices. A Member holds one in a Membership record. A Person holding at
least one DevicePublicKey holds one in their own definition; a Person holding
none holds no PersonPublicKey. A Member's and a Person's are never the same
key, a rule no software enforces yet. Until group key agreement exists,
org-node derives a Member's from a seed of the persona's own, a stand-in for
it. In code it is named `person_key` (the field) and `group_key` (as in
`check_group_key`); those identifiers stay, and the _Avoid_ list governs prose.
_Avoid_: P2pMemberKey, P2pPersonKey, Person-as-a-group key, person key, group key,
p2p key, member key

**Member-as-a-group key**:
The PersonPublicKey a Membership record holds for a Member: what an
Organisation's grants to that Member are encoded against. Rotatable, and
rotated whenever a DevicePublicKey is removed.
_Avoid_: group key, p2p key, member key

**DevicePublicKey**:
The ed25519 public key identifying one of a Member's or a Person's devices.
Each holds a bounded number of them. One device is meant to hold a different
DevicePublicKey in each record it appears in, a rule no software enforces yet.
In code a value of this type is named `device_key` (as in
`DeviceSecret::device_key()` and `KeychainError`'s fields); those identifiers
stay, and the _Avoid_ list governs prose and user-facing text (2026-10-15).
_Avoid_: device key, P2pDeviceKey, device id, endpoint key

**Person**:
An individual acting in their own capacity, apart from any Organisation. The
same individual may also be a Member of any number of Organisations; nothing
links their Person to those memberships except what the individual chooses to
reveal.
_Avoid_: user, account, individual member, profile

**Person definition**:
The record a Person shares with the collaborators they choose: name, surname,
DevicePublicKeys and, when it holds at least one DevicePublicKey, a
PersonPublicKey. Kept private otherwise: its content is never published, only
its Person hash and Encoding version, with the epoch they were set at.
_Avoid_: person record, person data, profile

**Person hash**:
The value that commits to one Person definition under one Encoding version,
published in the definition's place so that a collaborator who holds the
definition can recognise it.
_Avoid_: person root, fingerprint

**Encoding version**:
The number that fixes how a Person definition is encoded for computing its
Person hash, and so which key types the definition holds. Published beside
the Person hash, so that an encoding can change without every collaborator
changing at once.
_Avoid_: schema version, format version, hash version

**Isolated member**:
A Member holding zero DevicePublicKeys. Still a member of the Organisation,
and able to be restored by adding a DevicePublicKey. The state an Organisation
puts a Member into when that Member's devices are compromised.
_Avoid_: revoked member, removed member, suspended member

**Membership record**:
The authoritative record of who belongs to an Organisation, from which a
membership root is computed. Never modified in place: a change produces a new
record.
_Avoid_: trie, SMT, membership list (all implementation or shape, not the
concept)

**Membership root**:
The single value that commits to a whole Membership record, and the value
published so that others can verify a membership claim.
_Avoid_: root hash, merkle root (both name the mechanism)

**Device root**:
The value committing to the set of DevicePublicKeys one Member or one Person
holds, under one domain; a Person hash and a Member's leaf in the Membership
record each commit to one.
_Avoid_: device hash, device sub-trie hash, device trie

**Device root tree**:
The sparse Merkle tree of depth 4 whose root is the device root.
_Avoid_: device trie, device sub-trie

**Sparse Merkle tree**:
A binary hash tree of fixed depth with one position for every possible value
of that depth's width, almost all of them empty, committing to the leaf
identities it holds, and their leaves, by a single root. The Membership record
and each device root tree are one.
_Avoid_: SMT (in prose), trie, merkle trie, sparse merkle trie

**Depth**:
The number of levels from the root to a leaf of a sparse Merkle tree; a tree
of depth d has 2^d positions.
_Avoid_: size, height, bit width

**Position**:
The one place in a sparse Merkle tree a leaf may be held: the first Depth bits
of the leaf identity's position bytes.
_Avoid_: slot, index

**Position bytes**:
The 32 bytes a leaf identity maps to; the first Depth bits of them are its
Position.
_Avoid_: position (for the bytes), path, key hash

**Leaf**:
What a sparse Merkle tree holds at a position, chosen by the tree's consumer:
a leaf identity and the content its content hash covers, such as a Member's
leaf or a device root tree's leaf.
_Avoid_: entry, record, item (for the tree's side)

**Leaf identity**:
What tells one leaf of a sparse Merkle tree from another and fixes its
position, chosen by the tree's consumer: a MemberId for a Member's leaf, a
DevicePublicKey for a device root tree's leaf.
_Avoid_: key (for the tree's side), leaf key, leaf id

**Collision**:
Two leaves of different identities that have the same position in a sparse
Merkle tree.
_Avoid_: clash, conflict

**Identity hash**:
The hash of a leaf identity, under a domain the tree's consumer chooses; it
stands for the identity wherever the tree commits to a leaf.
_Avoid_: key hash, id hash

**Content hash**:
The hash of everything a leaf holds besides its identity, under a domain the
tree's consumer chooses.
_Avoid_: value hash, data hash, payload hash

**Tree hasher**:
The set of hash domains a sparse Merkle tree is built under, chosen by its
consumer: leaf, node and empty leaf, each separated from the others and from
every other consumer's.
_Avoid_: hash function, digest (for the set), hasher (alone, in prose)

**Empty-leaf sentinel**:
The bytes whose hash in a Tree hasher's leaf domain is that tree's Empty
leaf, chosen so that they encode no leaf the tree can hold.
_Avoid_: empty marker, null leaf, padding

**Device root hasher**:
The hash domains a device root tree is built under: a Tree hasher, the
domains of a DevicePublicKey's identity hash and of a device leaf's content
hash, and its Empty-leaf sentinel, each chosen by the device root tree's
consumer.
_Avoid_: device hasher, device trie hasher

**Device leaf**:
A leaf of a device root tree: a DevicePublicKey as its leaf identity, holding
no content besides it.
_Avoid_: device node, device entry

**Device position**:
A DevicePublicKey's Position in any device root tree: the first Depth bits of
its position bytes, under the one domain Person and Member device root trees
share.
_Avoid_: device slot, device index

**Empty-subtree hash**:
The hash of a subtree of a given level that holds no leaf; one value per
level for a given set of hash domains.
_Avoid_: zero hash, empty node hash

**Proof ending**:
What a Tree absence proof shows at its Position: Empty, no leaf held there, or
Occupied, a leaf of another identity held there, given by its identity hash
and content hash.
_Avoid_: terminal, end leaf, tail

**Calculated**:
A sparse Merkle tree none of whose node hashes is unset.
_Avoid_: hashed, finalised, committed (for the tree)

**Tree changes**:
Between two sparse Merkle trees: the leaf identities the first holds and the
second does not, and the upserted leaves.
_Avoid_: diff, delta, patch (for the tree's side)

**Upserted leaf**:
A leaf the second of two sparse Merkle trees holds that the first does not
hold as it is: one added or one replaced.
_Avoid_: changed leaf, updated leaf, new leaf

**Level**:
The height of a node above the leaves of a sparse Merkle tree: a leaf's
position is at level 0 and the root at level Depth.
_Avoid_: layer, depth (for one node)

**Node hash**:
The 32-byte hash of a leaf, of a node over two subtrees, or of the root of a
sparse Merkle tree.
_Avoid_: digest, node value

**Empty leaf**:
The hash a sparse Merkle tree gives a position that holds no leaf: the
empty-subtree hash of level 0.
_Avoid_: zero leaf, null leaf, empty slot

**Sibling**:
The node hash beside a proof's node at one level on the way from a position
to the root, with which that node is hashed to give the node a level up.
_Avoid_: neighbour, proof node, audit node

**Proof map**:
The Depth bits of a proof marking the levels whose sibling is not that
level's empty-subtree hash; a proof carries only the marked siblings.
_Avoid_: bitmap, mask, bitfield

**Absence proof**:
A value showing, against one root, that something is not held under that
root; it does not show that it was never held. A Tree absence proof, a Device
absence proof and a Person absence proof are its kinds.
_Avoid_: exclusion proof, non-inclusion proof, non-membership proof,
revocation proof (for the value itself)

**Tree absence proof**:
An absence proof that a leaf identity is not held under the root of a sparse
Merkle tree.
_Avoid_: leaf absence proof, absence proof (alone, where another kind could
be meant)

**Inclusion proof**:
A value showing, against one root, that a leaf identity and its leaf are
held under that root.
_Avoid_: membership proof, existence proof, audit path

**Proven member**:
A Member's leaf without its DevicePublicKeys, as a Held proof carries it: its
MemberId, handle, Member-as-a-group key, name, surname and device root.
_Avoid_: member summary, member header, partial member

**Device absence proof**:
An absence proof that a DevicePublicKey is not held, under a Membership root,
by the Member a MemberId names there: a Vacant proof or a Held proof.
_Avoid_: device proof, device exclusion proof

**Vacant proof**:
A Device absence proof in which the MemberId holds no Member under the
Membership root: a Tree absence proof of that MemberId under the Membership
root, its vacancy.
_Avoid_: empty proof, non-member proof

**Held proof**:
A Device absence proof in which the MemberId holds a Member under the
Membership root and that Member lacks the DevicePublicKey: the Proven member,
its inclusion proof, and a Tree absence proof under the Member's device root,
its device-root absence.
_Avoid_: member proof

**Person absence proof**:
A value showing, against one Person hash, that a DevicePublicKey is not held
in the Person definition it commits to: that definition's fields but its
DevicePublicKeys, its device root, and a Tree absence proof under that root.
_Avoid_: person proof, person exclusion proof, person-hash proof

**Change set**:
A set of membership changes anchored to the Membership record it was computed
against, exchanged between administrators. Rejected if applied to a record
other than the one it declares.
_Avoid_: delta, patch, diff

**Organisation state**:
The record an Organisation publishes on-chain: its current Membership root,
its Organisation public key, and the epoch counter that orders successive
publications. The only thing about an Organisation the chain is asked.
_Avoid_: org state, registry slot, on-chain record

**Organisation public key**:
The Organisation's X25519 public key, published in its Organisation state so
that a member who is given the Organisation's private key can check it is the
real one. It is a key-agreement key, not a signing key: it identifies no admin
and authorises nothing. org-node hands the private key to every Member's
Devices, and a receiver keeps it only if its public half is this key
(org-node's *Organisation private key*).
_Avoid_: signing key, org key, admin key

**Finalised block**:
A block the chain has committed to irrevocably, so that a value read from it
cannot be discarded by a later reorganisation. The Organisation state a
Change set is verified against is read from one.
_Avoid_: best block, latest block, head (each names a block that can still
be discarded)

**Admin**:
A user whose own account is a signatory of the multisig that controls the
Organisation's pure proxy. A Device is an Admin's Device when its user is one.
Admin status is a fact on the chain, not a role in the Membership record.
(Added 2026-10-08, change `worktree-org-io-create`.)
_Avoid_: administrator key, Organisation admin (on-chain-client's term for
the Organisation's 20-byte identifier, which is not a user)

**org-io**:
The unit that does all IO for an Organisation: reading and writing the chain,
holding the user's signatory key, and (from stage S4) peer communication and
local storage, while org-node takes values and returns values. (Added
2026-10-08, change `worktree-org-io-create`;
`docs/adr/2026-10-06-org-io-unit.md`.)

*The six terms below were added 2026-10-15 (change
`worktree-org-io-transport`, ARCHITECTURE conformance review finding-4):*
org-io's stage S4 put them in the public interface between org-node, org-io
and the app.

**Device seed**:
The 32-byte secret of one of a Persona's devices, from which its
DevicePublicKey is derived and with which its transport endpoint proves it
holds that key. Kept at rest only in the device's keychain item, and never
handed to the app.
_Avoid_: Device secret, device private key, endpoint secret

**DeviceSecret**:
org-node's holder of one Device seed for the duration of one operation:
moved, never copied, displayed or serialised, and wiped when dropped; its
DevicePublicKey is derived once, when it is built. The type keeps its name;
prose says Device seed for the value it holds.
_Avoid_: using it for the seed itself

**Keychain item**:
The entry in the operating system's keychain that holds one Persona's Device
seed, named by that Persona's DevicePublicKey. A failure to read or write one
is a **KeychainError**, which names the DevicePublicKey and nothing of the
item's content.
_Avoid_: keystore, credential, wallet

**SealedStore**:
A Persona store (org-node's glossary) encrypted for disk: the bytes org-node
hands org-io to write and takes back to open it. Opened only with the
Persona's Device seed.
_Avoid_: encrypted store, store blob

**Store file**:
The file in the data directory that holds one Persona's SealedStore, named by
that Persona's DevicePublicKey. One per Persona.
_Avoid_: database, persona store file, store blob

**Verdict**:
The one byte a device's receive side writes back after a message: whether the
message was acted on (accepted) or not (refused). A sender reads a refusal,
or silence, as its cue to try another address.
_Avoid_: acknowledgement, ack (an acknowledgement is a signed message about a
committed update, a different thing)
