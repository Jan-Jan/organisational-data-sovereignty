# Context

on-chain-client is the reader that decides what the chain says an Organisation's
membership is: it derives an Organisation's identifier and the storage it lives
in, decodes the Organisation state held there and the events the registry
contract emits, decides which of those events belong to the Organisation being
watched, and tells a consumer which of its observations are provisional and
which are committed. It exists so that every device in the system takes its
belief about the chain from one implementation of these conventions rather than
from its own reading of them.

## Language

Glossary of on-chain-client's internal vocabulary. Definitions only — no
implementation details, no specs. Terms that cross a unit boundary
(Organisation, Organisation state, Membership root, Change set, Finalised block,
Member, Device key) live in the root `docs/CONTEXT.md` and are not repeated
here.

**Organisation slot**:
The storage an Organisation's Organisation state occupies in the registry
contract: the three consecutive fields holding its Membership root, its signing
key and its Epoch. One per Organisation, keyed on the Organisation admin, and
absent until the Organisation is initialised.
_Avoid_: org state slot, registry entry, record

**Slot key**:
The 32-byte address of one field of an Organisation slot. Derived, never given:
this unit computes it from the Organisation admin and the map's declared index,
which is why a change to either convention is a hazard rather than an
inconvenience.
_Avoid_: storage key, slot id, storage address

**Organisation admin**:
The 20-byte identifier an Organisation is keyed on, being what pallet-revive
maps the Organisation's controlling account to. Stable for the Organisation's
lifetime, and public: it appears as an indexed field in every event the
Organisation's registry writes emit.
_Avoid_: OrgId, admin address, H160, org key

**Emitting contract**:
The contract that wrote a log, as reported by the chain in the log itself.
Distinct from the contract a reader was constructed for, and the only thing that
tells a genuine registry event apart from one any other contract can emit
carrying the same Event signature and the same Organisation admin.
_Avoid_: contract address, source, sender

**Event signature**:
The value identifying which of the registry contract's declared events a log
claims to be, carried as the log's first topic and fixed by the event's declared
name and parameter types. A log's claim, not a proof: matching one says which
shape to decode, never who emitted it.
_Avoid_: topic0, event hash, selector

**Runtime spec version**:
The number by which the chain identifies the version of its own runtime, and so
the version of the storage and event layouts a decoder must be compiled for. The
one input that decides which decoder this reader may use.
_Avoid_: runtime version, spec_version (the field name), metadata version

**Epoch**:
The Organisation state's own counter, incremented by the registry contract once
per published Membership root, which orders an Organisation's successive
publications. Zero means no Organisation state has ever been written.
_Avoid_: nonce, version, sequence number (which is org-node's, for Envelopes)

**Best-block observation**:
An observation this reader took from a block the chain has accepted as its
current best but has not committed to. Provisional by definition: the block can
still be discarded, and so can the observation.
_Avoid_: pending event, unconfirmed event, optimistic read

**Finalised observation**:
An observation this reader took from a Finalised block, and so from a block that
cannot be discarded. What a consumer may commit on.
_Avoid_: confirmed event, settled event

**Reorg notification**:
What this reader tells a consumer when a best head it previously reported has
been discarded, naming that head by both its hash and its number so a consumer
keyed on either can find what to undo. Its absence is not a promise that nothing
was discarded — only that nothing detectable was.
_Avoid_: rollback, revert, fork notification
