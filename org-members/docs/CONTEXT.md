# Context

<!-- One or two sentences: what this system is and why it exists. -->

## Language

<!--
Glossary of the project's ubiquitous language. Definitions only — no
implementation details, no specs. Format:

**Term**:
One or two sentences defining what it IS (not what it does).
_Avoid_: synonym1, synonym2

Be opinionated: when several words exist for one concept, pick one and list
the rest under _Avoid_. Only include terms specific to this project's domain.
-->

**Membership model**:
The Quint model of the membership operations (`org-members/quint/`), against
which the crate's behaviour is checked.
_Avoid_: spec, specification, Quint spec

**Model action**:
One named alternative of the membership model's `step`; each corresponds to
one thing a caller does to the trie — a single public trie operation, or, for
receiving a change set, applying it and verifying the result against the
expected root.
_Avoid_: spec action, transition

**Conformance test**:
The test that replays membership-model traces against the crate and requires,
after every model action, the same result, the same error and the same
membership state.
_Avoid_: MBT test, model-based test (as a name for this test)

**Declared abstraction boundary**:
The written list of crate behaviour the membership model deliberately does not
describe, each entry naming the test that carries it instead.
_Avoid_: out of scope, not modelled (unqualified)

**Named scenario**:
A fixed, named run in the membership model, replayed against the crate
deterministically regardless of seed.
_Avoid_: regression trace, quint test (as a noun)

**Exported model interface**:
The part of the membership model another unit's model may import; nothing
outside it is visible across the unit boundary.
_Avoid_: shared types, public model

**Device key**:
The name this unit's requirements and risk controls written before 2026-10-05
use for a DevicePublicKey (root `docs/CONTEXT.md`), among them REQ-xdx2c2,
REQ-shk82j, RC-mqtks7 and RC-4apk6w, which are not reworded for a change of
term.
_Avoid_: using it in new text; DevicePublicKey is the term

**Device key set**:
The name the same items use for a Member's set of DevicePublicKeys, the
`DeviceSlots` a member record holds.
_Avoid_: using it in new text; say a Member's DevicePublicKeys

**Newtype**:
A single-field type that gives a value its own type, so it cannot be confused
with another value of the same representation; either a Validated type or a
Tag type.

**Validated type**:
A value type that can only be constructed by checking an invariant, so input
that breaks the invariant is rejected rather than represented.
_Avoid_: smart constructor, checked type

**Tag type**:
A value type with no invariant beyond its representation, used to keep values
of different meaning apart.
_Avoid_: wrapper, marker type
