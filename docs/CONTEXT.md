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
