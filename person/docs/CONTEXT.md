# Context

The `person` unit holds an individual's identity types: names, device keys,
the PersonPublicKey, device slots, the device sub-trie and its root, and the
X25519 public-key validity check. The Person definition built on these types,
and the hash that commits to it, are added later.

## Language

<!--
Glossary of this unit's internal language. Definitions only — no
implementation details, no specs. Terms that cross a unit boundary live in the
root docs/CONTEXT.md. Format:

**Term**:
One or two sentences defining what it IS (not what it does).
_Avoid_: synonym1, synonym2
-->

**Name**:
An individual's given name as the unit stores it: in Unicode NFC, and bounded
in length.
_Avoid_: first name, forename, display name

**Surname**:
An individual's surname as the unit stores it: in Unicode NFC, and bounded in
length.
_Avoid_: last name, family name

**Device slots**:
The bounded, sorted, duplicate-free set of DevicePublicKeys one record holds.
_Avoid_: device list, device set, devices

**Device root**:
The value committing to a set of DevicePublicKeys under one domain.
_Avoid_: device hash, device sub-trie hash
