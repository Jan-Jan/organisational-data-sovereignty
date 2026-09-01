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
(`docs/requirements/`), which is why they cluster around membership. The
remaining vocabulary — already well developed in `org-members/AGENTS.md`, and
the authority for crate-level terms until then — is migrated at tooth 9 of
`docs/plans/2026-08-26-ratchet-gap-analysis.md`.

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
every key, so that a grant made to a Member survives a rename.
_Avoid_: node id, member key

**Member-as-a-group key**:
The key a grant is encoded against when an Organisation gives a Member access,
and from which that Member's devices derive their own access. Rotatable, and
rotated whenever a device key is removed.
_Avoid_: group key, p2p key, member key

**Device key**:
The key identifying one of a Member's devices. A Member holds a bounded number
of them.
_Avoid_: device id, endpoint key

**Isolated member**:
A Member holding zero device keys. Still a member of the Organisation, and able
to be restored by adding a device key. The state an Organisation puts a Member
into when that Member's devices are compromised.
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
