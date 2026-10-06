# Person definition requirements

Scope: an individual's own definition — the record a person shares, in their
individual capacity and apart from any Organisation, with collaborators they
choose — and the hash that commits to it. Written from the grill-requirements
interview of 2026-10-04.

Out of scope here, by owner decision: everything on-chain. Publishing the hash
and its encoding version, the epoch compare-and-swap, the account that owns the
entry, and revoking every device from a foreign device with a hardware key are
deferred to a future `on-chain-person` unit. This unit only computes a hash and
checks one.

Facts the interview settled, which these items rest on:

- A Person is unlinkable to the same individual's memberships: its keys differ
  from every Member record's, even for the same device, and only the individual
  knows which records belong together.
- The hash is deliberately unsalted. Only collaborators the person shared the
  definition with see its keys, and those collaborators are meant to recompute
  the hash and recognise the on-chain entry as the person's.
- The PersonPublicKey is agreed among the person's devices by continuous
  group key agreement; it is computed elsewhere, and this unit checks only its
  relationship to the DevicePublicKeys.
- The key rules below are the rules for a Member too (owner ruling): the
  org-members amendments that follow from it are a separate set of items.

Every item is `satisfies: derived`: the rules come from owner decisions in
the interview, not from a written system-needs document.

## Hash

**REQ-9m5pq2**: The software shall compute, from a Person definition and an
encoding version, a single hash value that differs whenever the name, the
surname, the PersonPublicKey, or the set of DevicePublicKeys differs, and that is
the same for two definitions equal in all four under the same version.
satisfies: derived
exported: yes

**REQ-7n4g8b**: The software shall report whether a given hash value is the
hash of a given Person definition, shall report a match only when the
value equals the hash it computes for that definition, and shall retain no
hash or definition from one check to the next.
(implements: RC-qn2r5b)
satisfies: derived
exported: yes

**REQ-wg7z4s**: The software shall compute and check a Person hash under the
encoding version the request states, and shall report a version it does not
implement as an error, distinct from a non-match.
satisfies: derived
exported: yes

The version a collaborator's hash was computed under is published beside it,
by `on-chain-person`: the publisher's software chooses it, so an encoding can
change without every collaborator changing at once, and a reader whose software
predates the version is told so rather than told its copy is out of date.

## Keys and devices

A Person definition holds its DevicePublicKeys as device slots, so the bound,
the sorted order and the refusal of a key held twice are REQ-4szc22's, and the
strictly increasing order of slots read from outside the process is
REQ-tq4ms4's; this ledger does not restate them.

**REQ-ht3x78**: The software shall reject a Person definition that holds no
DevicePublicKey and holds a PersonPublicKey, so that removing every device
also removes every key a removed device could derive access from.
(implements: RC-ee994g)
satisfies: derived
exported: yes

**REQ-7qgx2q**: The software shall reject a Person definition that holds one or
more DevicePublicKeys and whose PersonPublicKey is absent or equals any one of
them.
satisfies: derived
exported: yes

## Successive definitions

The PersonPublicKey is the result of group key agreement over the DevicePublicKeys,
compatible with MLS's TreeKEM and Keyhive's BeeKEM. It must change whenever the
set of DevicePublicKeys changes — keys added, removed or replaced, one or several at
once — and may also change while the DevicePublicKeys stay the same: a rotation
that recovers from a leaked device secret without replacing the DevicePublicKey.
Requiring a change on an addition is stricter than MLS's minimum, which lets a
commit that only adds members omit its update path; the key agreement layer
must always commit with one.

**REQ-bhez2u**: The software shall reject a Person definition offered as the
successor of another whose set of DevicePublicKeys differs from its predecessor's
and whose PersonPublicKey equals its predecessor's, so that a device added or
removed cannot keep or use the key agreed without it.
(implements: RC-5gzfux)
satisfies: derived
exported: yes

## Definitions from outside the process

**REQ-7ymek3**: The software shall re-validate every Person definition it reads
from outside the process — the validity of its name and surname, the bound,
uniqueness and order of its DevicePublicKeys (REQ-4szc22, REQ-tq4ms4), and the
PersonPublicKey rules of REQ-ht3x78 and
REQ-7qgx2q — and shall reject one that fails before computing or checking its
hash, so that a hash match is reported only for a definition the software
itself would accept.
(implements: RC-bnm8qs)
satisfies: derived
exported: yes

The rules REQ-7ymek3 re-applies are the shared types' own, merged with change 1
of `docs/plans/2026-10-04-person-sequencing.md`
(`person/docs/requirements/2026-10-04-identity-types.md`): names and surnames
REQ-r7mytp, device slots REQ-4szc22 and REQ-tq4ms4, each DevicePublicKey
REQ-q6xkna, and the PersonPublicKey REQ-3vqs9b.

## Robustness

**REQ-r4keha**: The software shall report every rejection by a Person
operation — constructing a Person definition, checking a successor, computing
a Person hash and checking one, an encoding version it does not implement
among them — as an error that identifies the rule broken, shall report a hash
that does not match a valid definition as a non-match rather than as an error,
and shall not panic in any Person operation for any input, including input
that is malformed, hostile, or exceeds a documented limit.
(implements: RC-d8n777)
satisfies: derived
exported: yes

REQ-r4keha covers the Person operations only. The types a definition is built
from — names, surnames, DevicePublicKeys, the PersonPublicKey and device
slots — are already covered by REQ-vxx8k3 (no panic) and REQ-bczz87 (typed
errors), and a Person operation that rejects one of them reports that type's
error.
