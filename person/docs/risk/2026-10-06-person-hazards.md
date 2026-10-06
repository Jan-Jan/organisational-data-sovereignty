# Hazard analysis — Person definition

The first hazard enumeration in this unit's ledger, run under IEC 62304
**Class C** and evaluated against the acceptability matrix in this ledger's
README. Interview of 2026-10-04 (analyze-risks), following the requirements in
`person/docs/requirements/2026-10-06-person-definition.md`.

## Scope

The capability analysed is the **Person definition**: an individual's own
record (name, surname, PersonPublicKey, DevicePublicKeys), the hash that commits to
it, the check of a given hash against a definition, and the check of a
definition against its predecessor.

Deliberately **outside** this analysis, and not to be read as assessed:

- **Everything on-chain** — publishing the hash, reading it back, the entry's
  owning account and epoch. Deferred to a future `on-chain-person` unit
  (`docs/plans/2026-10-04-person-sequencing.md`); hazards that unit must close
  are named below where they arise.
- **Key agreement** — computing the PersonPublicKey (TreeKEM/BeeKEM). This
  unit checks the key's relationship to the DevicePublicKeys, not how it was made.
- **The application surface** — how a definition is shown to the collaborator
  who acts on it.

## Harms

The two harms of the org-members register apply unchanged, both **S3**
(owner, 2026-10-04): **Disclosure** — someone reads material delegated to a
Person that they should not, which in journalism and government deployments
can identify a source; and **Unavailability** — someone cannot reach material
when a decision depends on it. A Person delegates without an Organisation's
fallbacks, so neither harm is less severe here.

Probabilities below are the probability of the hazardous situation arising,
as in the org-members register; the situation-to-harm step is not estimated.

## Hazards and controls

**HAZ-vv2c3p**: a collaborator's copy of a Person definition can be out of
date — still listing a removed device or a replaced PersonPublicKey — while the
collaborator believes it current, so that material is delegated to a key a
removed (possibly stolen) device can use, disclosing it. Severity: S3.
Probability: P2.

**RC-qn2r5b**: a hash check answers only whether the hash supplied with the
request is the hash of the definition supplied with it; the unit keeps no hash
or definition between checks, so it can never report a definition current
against a value it remembered rather than one the caller has just read.
mitigates: HAZ-vv2c3p

Residual risk for HAZ-vv2c3p: **not acceptable for release.** RC-qn2r5b
removes the stale-state pathway inside this unit, but whether the hash a
collaborator supplies is the latest one the chain holds — read from a
finalised block, never from a cache — is the other half of the control, and it
belongs to `on-chain-person`, which does not exist. Until that unit supplies
such reads, the Person hash check is not a releasable capability on its own.

**HAZ-mvavg7**: a Person definition can drop a DevicePublicKey — a stolen device,
say — while its successor keeps the PersonPublicKey the removed device held,
so that material delegated to the Person stays readable by the removed device,
disclosing it. Severity: S3. Probability: P2.

**RC-5gzfux**: a definition offered as the successor of another is refused when
its DevicePublicKeys differ from its predecessor's and its PersonPublicKey does not.
mitigates: HAZ-mvavg7

Residual risk for HAZ-mvavg7: reduced, **not acceptable**, because whether the
removed device can derive the *new* key is decided by key agreement, outside
this unit. Comparing a key only with its immediate predecessor is sufficient by
owner ruling (2026-10-04): a new PersonPublicKey equal to one held several
definitions earlier is not refused, on the assumption that key generation is
random enough that such a repeat does not occur. That assumption is the key
agreement layer's responsibility, not this unit's.

**HAZ-zdwra7**: a Person definition can hold no DevicePublicKey yet still hold a
PersonPublicKey, so that after the individual revokes every device — all of
them stolen — material delegated to the Person stays readable through a key a
revoked device could use, disclosing it. Severity: S3. Probability: P2.

**RC-ee994g**: a definition holding no DevicePublicKey is refused unless it also
holds no PersonPublicKey. mitigates: HAZ-zdwra7

Residual risk for HAZ-zdwra7: within this unit the situation is eliminated —
no definition it accepts can reach it. The matrix scores S3 not acceptable at
every probability, and the route that remains is the revocation reaching
collaborators: publishing the zero-device definition's hash, which belongs to
`on-chain-person`.

**HAZ-h3swmw**: bytes received from anyone a person connects to can claim to be
a Person definition that breaks this unit's rules — a PersonPublicKey with no
DevicePublicKey, more devices than the bound, a duplicate DevicePublicKey — and, hashed
to match what the sender published under their own on-chain id, be treated as
a valid definition whose keys delegation then follows, disclosing material
through a state no honest software produces. Severity: S3. Probability: P2.

**RC-bnm8qs**: every Person definition read from outside the process is
re-validated against every rule a definition must satisfy, and rejected,
before its hash is computed or checked. mitigates: HAZ-h3swmw

**HAZ-y6ft54**: malformed, hostile or oversized input to any Person operation
can crash the process or exhaust its memory, so that the person cannot reach
material delegated to or by them while it is down. Severity: S3. Probability:
P2.

**RC-d8n777**: every rejection by a Person operation — constructing a
definition, checking a successor, computing a Person hash and checking one —
reports a typed error naming the rule broken, and no input to those operations
— malformed, hostile, or beyond a documented limit — causes a panic.
mitigates: HAZ-y6ft54

RC-d8n777 adds nothing for the types a definition is built from: their typed
errors and freedom from panic are REQ-bczz87 and REQ-vxx8k3, already in this
ledger (`2026-10-04-identity-types.md`).

Residual risk for HAZ-h3swmw and HAZ-y6ft54: reduced, **accepted by the
owner** (2026-10-06). The Person definition change adds a bolero fuzz target,
`person/tests/fuzz_person_decode`, over Person decoding and every Person
operation on what decodes; `cargo test -p person` runs it for one second
(about 11 000 inputs), and `cargo bolero test fuzz_person_decode --engine
libfuzzer` runs it as a coverage-guided campaign. No campaign has been run or
recorded, and the owner accepts the residual on the fuzz target alone, with the
no-panic property tests (REQ-vxx8k3) and the typed refusals (RC-bnm8qs,
RC-d8n777) beside it. (org-node and on-chain-client have bolero targets of
their own; org-members has none.)

## Hazards introduced by these controls

**RC-5gzfux makes every device change a new PersonPublicKey, and material
delegated under the previous key does not follow it.** Alice's only phone is
stolen; she revokes it (no device, no key), buys a new phone, and holds a new
key. Everything collaborators delegated to her before was granted against a key
only the stolen phone held, so until each collaborator fetches her new
definition and grants again under the new key, she cannot read it.
Unavailability, S3. The same lag has a disclosure side — a collaborator who has
not fetched the new definition keeps granting new material to the old key —
which is HAZ-vv2c3p. The control is the collaborator's: on seeing a new
definition, re-grant under the new key what was delegated under the previous
one (re-encrypting or re-delegating, in key-agreement terms). It belongs to the
application and the key-agreement layer, not to `person` (owner, 2026-10-04);
recorded here, unminted, for the reason given in the next section, so that the
consuming unit's risk analysis inherits it as a named obligation.

## Hazards whose only control lies outside this unit

**A Person sharing a key with a Member record.** The same key — a DevicePublicKey
or a PersonPublicKey — placed in an individual's Person definition and in one of
their Membership records lets anyone who sees both learn that the Person and
the Member are one individual, and in the journalism and government
deployments that link can expose them. Disclosure, S3. Independently generated
keys collide with negligible probability; the realistic route is wrong usage —
software reusing a key across contexts. `person` cannot check it, since it
never sees a Membership record. The control belongs to the software that uses
`person` and holds both kinds of record — the individual's own application:
generate keys per context, and refuse to place one key in two records (owner,
2026-10-04).

Not minted as a HAZ item: every HAZ needs an RC, and every RC a requirement in
this unit's ledger, but this control's requirement belongs to a consuming unit,
none of which depends on `person` yet. The consuming unit's own risk analysis
mints it when it declares that dependency — the same limitation org-members
records for the compromise of an organisation's admin account.

**Spoofing a Person by name.** Assessed and found not to be a hazard of this
unit (owner, 2026-10-04): a collaborator checks a definition against the hash
stored under the on-chain id they recorded when the two first connected, and
that hash covers the DevicePublicKeys and the PersonPublicKey, so a definition
bearing someone else's name still carries its sender's keys and is checked
against its sender's entry. A Person with no DevicePublicKey cannot be collaborated
with at all. What remains is the first connection itself — recording the right
on-chain id for the right individual — which happens before `person` is
involved and belongs to the application.

## Derived requirements assessment

Every requirement in this change is `satisfies: derived`. Each is assessed
here for whether it introduces a hazard, affects a hazardous situation above,
or changes a control's effectiveness.

REQ-9m5pq2 (distinct definitions, distinct hashes, per encoding version):
required for verification; no hazard impact. A substitution hazard — a
different definition matching a person's published hash, through an ambiguous
encoding or a hash from another domain — was analysed and withdrawn (owner,
2026-10-04): a collaborator looks a person up by the on-chain id stored at
first connection, never by hash, so a substitute must match that exact value,
which needs a second preimage of the hash function; and DevicePublicKeys are of
fixed length, so an encoding ambiguity could at most re-read the person's own
bytes, never introduce another's keys. The hash function's collision
resistance is SOUP residual risk, recorded with the SOUP item when the
architecture names it.
assesses: REQ-9m5pq2

REQ-7n4g8b realises RC-qn2r5b (HAZ-vv2c3p); REQ-ht3x78 realises RC-ee994g
(HAZ-zdwra7); REQ-7ymek3 realises RC-bnm8qs (HAZ-h3swmw); REQ-r4keha realises
RC-d8n777 (HAZ-y6ft54). None introduces a hazard beyond the one it controls:
each narrows what the unit accepts or retains, and a narrower acceptance can
refuse only definitions no honest software produces.
assesses: REQ-7n4g8b, REQ-ht3x78, REQ-7ymek3, REQ-r4keha

REQ-wg7z4s (hash under a stated encoding version; an unknown version is an
error): introduces one hazardous situation and controls it in the same item —
a reader whose software predates the publisher's encoding would otherwise
report a current definition as out of date and send its user to refetch what
they already hold (Unavailability). Reporting the version as unsupported, as a
distinct error, closes that. No disclosure impact: the version is published by
the owner of the on-chain entry, and lookup is by that entry's id.
assesses: REQ-wg7z4s

REQ-7qgx2q (with one or more DevicePublicKeys, a PersonPublicKey present and equal
to none of them): no hazard of its own. A PersonPublicKey equal to a
DevicePublicKey is evidence the definition was not produced by key agreement, so the rule
supports detection of a forged definition (HAZ-h3swmw). It could refuse a
genuine definition only if key agreement produced a key equal to a DevicePublicKey,
which fresh randomness makes negligible.
assesses: REQ-7qgx2q

**What the check does not catch (owner ruling, 2026-10-06).** The
PersonPublicKey ≠ DevicePublicKey check (LLR-kbhc43) is a byte comparison
only. It catches a literal copy of a DevicePublicKey's 32 bytes into the
PersonPublicKey. It does not catch reuse of a device's ed25519 key pair through
its X25519 (birational) image, whose bytes differ from the ed25519 encoding;
nor the up to 8 byte-distinct encodings of one X25519 key with a small-order
component mixed in, which `PersonPublicKey::parse` accepts (LLR-vs7etb) and
which give the same key-agreement output. The owner accepts both as residual
risk for Persons, the same ruling as for Members: org-members'
`org-members/docs/risk/2026-10-03-key-uniqueness.md`, "After the switch to
`person`'s key types (owner, 2026-10-05)", accepts both narrowings of its
byte-comparing key-uniqueness rule — a PersonPublicKey is a fresh key-agreement
key from the CGKA of its holder's devices, never a device's signing key, so only
tooling that departs from the rules of `docs/adr/2026-10-04-group-key-rules.md`
reuses one, and the holder of an X25519 secret
gains nothing by presenting it under a second encoding — and
`person/docs/risk/2026-10-04-identity-types.md` sets out both routes, the
reused keypair and the mixed-order PersonPublicKeys, in its paragraphs on the
behaviour change and on mixed-order keys. The check is therefore evidence
against one kind of forged or misbuilt definition, not a guarantee that no
device key is reused as the PersonPublicKey.

REQ-bhez2u realises RC-5gzfux (HAZ-mvavg7), and introduces the Unavailability
hazard recorded under "Hazards introduced by these controls": material
delegated under a previous PersonPublicKey is unreadable to the person's new
devices until collaborators re-grant it.
assesses: REQ-bhez2u

## Derived design behaviour assessment

LLR-dtwpr8 (a `Person`'s `Debug` form shows neither name nor surname): no
hazard impact, and a disclosure consideration recorded here. A Person
definition is kept private except from the collaborators its owner chooses,
and `Debug` is what diagnostics and panic messages print, so a `Person` in
one never shows the individual's name. The rest of its `Debug` form — the
key prefixes and the device count — is LLR-a6krbh's and the slots' own. The
same reasoning as LLR-fbqs2r's in `person/docs/risk/2026-10-04-identity-types.md`
applies: `Display` and the `String` conversion of a `Name` still write it, and
each unit that uses a `Person` owns that choice.
assesses: LLR-dtwpr8

## Residual risk

| Hazard | S/P | Residual | Why |
|---|---|---|---|
| HAZ-vv2c3p | S3/P2 | not acceptable for release | latest-finalised chain reads belong to `on-chain-person`, which does not exist |
| HAZ-mvavg7 | S3/P2 | reduced, not acceptable | whether a removed device can derive the new key is decided by key agreement |
| HAZ-zdwra7 | S3/P2 | eliminated within this unit; not acceptable on the matrix | the revocation reaching collaborators belongs to `on-chain-person` |
| HAZ-h3swmw | S3/P2 | reduced, accepted by the owner (2026-10-06) | fuzz target in place; no recorded fuzzing campaign |
| HAZ-y6ft54 | S3/P2 | reduced, accepted by the owner (2026-10-06) | fuzz target in place; no recorded fuzzing campaign |

Conclusion: `person` removes every pathway to these hazards that lies inside
it, and none of them is closed by `person` alone. The Person capability is not
releasable until `on-chain-person` provides latest-finalised reads and
publication, the consuming application controls key reuse across records and
re-grants after a key change.
