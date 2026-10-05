# Risk — shared identity types

Assessment of the derived requirements this change writes in
`person/docs/requirements/2026-10-04-identity-types.md`, and
of the one behaviour org-members' switch to them will change. Class C; the
acceptability matrix is this ledger's README.

## What is and is not new

The types are built here from org-members' code, with its behaviour;
org-members switches to them in the following change. The hazards their
validation controls in org-members' use — a serialised record from another
process introducing a member the software would reject, and malformed or
hostile input — stay analysed in org-members' register (its hazard analysis of
2026-09-02), and org-members' requirements realising those controls are
unchanged and stay verified by org-members' tests. (Cited in words, not by ID:
`person` depends on no unit, so its ledgers name no other unit's items.)
No control in that register validates the member-as-a-group key by its curve:
the register's controls name handle and device-key validation, never
validation of the Member-as-a-group key. One derived rule there does compare
Member-as-a-group keys with device keys by their bytes; how the switch affects
it is the next two paragraphs.
This change alters no org-members behaviour, so it mints no hazard. The Person definition's
hazards are analysed with that capability, on branch
`worktree-person-requirements`.

**The one behaviour change, when org-members switches** — the
member-as-a-group key, then a PersonPublicKey, validated as X25519 rather than
ed25519 — adds a check: small-order keys are rejected (REQ-3vqs9b). Member
hashes over the same bytes are unchanged. It also weakens one existing rule,
a hazard consideration the org-members switch must assess before it merges.
org-members' key-uniqueness rule (every key held once in a record, assessed in
its key-uniqueness risk file as reducing the hazard that a removed device keeps
access) catches a device's own key installed as a Member-as-a-group key by
comparing encoded bytes. Today both keys are ed25519 encodings, so one keypair
in both roles is the same 32 bytes and is refused. Once Member-as-a-group keys
are X25519 u-coordinates (PersonPublicKeys) and device keys ed25519
y-encodings, one keypair reused in both
roles — an ed25519 key and its birational X25519 image — has different bytes in
each, so the byte comparison no longer catches the reuse. Whether that route
needs a check that maps one encoding to the other, or is accepted, is the
switch's to decide; it is not `person`'s, because `person` holds no record of
which keys are in use.

**Mixed-order PersonPublicKeys, a second consideration for the switch.** A
PersonPublicKey accepts the u-coordinate of a mixed-order curve point — a
point of prime order plus a non-identity point of the 8-torsion subgroup —
because the rule rejects only small-order and non-canonical encodings (owner,
2026-10-05), and a mixed-order point is neither. X25519 clamps every secret
scalar to a multiple of 8, so a scalar multiplication sends the torsion
component to the identity: one X25519 secret's public key can be presented as
up to 8 byte-distinct accepted PersonPublicKeys, each giving the same X25519
output as the prime-order one. Within `person`'s rule this is accepted: none
of the 8 gives anyone a shared secret they could not compute from the
prime-order key. But org-members' key-uniqueness rule compares bytes, so one
key could be held twice in a record under two encodings without the rule
catching it. Whether the switch needs a check that maps a key to its
prime-order component, or accepts this, is the switch's to decide, as for the
reused keypair above.

**A claim tested, and closed by `person`.** org-members' SOUP inventory states
that `ed25519-dalek`'s `VerifyingKey::from_bytes` rejects small-order
encodings. The tests of REQ-q6xkna show it does not, and that it also accepts
non-canonical encodings (y at least 2^255 − 19, or x = 0 with the sign bit
set): the identity point decodes from four different 32-byte strings (y = 1
and y = 2^255 − 18, each with the sign bit clear and set), and `is_weak()`
reports it. `person` therefore rejects both itself (owner, 2026-10-04): a
DevicePublicKey is the canonical encoding of a point not of small order, so one
device has one encoding and no device key is the identity or another point of
small order. org-members' device keys accept non-canonical encodings,
small-order points and points with a torsion component (next paragraph) today;
its SOUP row is corrected, and the three narrowings assessed, when it
switches.

**Torsion-free device keys (owner, 2026-10-05).** `from_bytes` also accepts a
mixed-order point — a point of prime order plus a non-identity small-order
point — and `is_weak()` does not report it, because the point is not itself of
small order. Its tests show it. A DevicePublicKey is now torsion-free as well:
of prime order, checked with curve25519-dalek's `is_torsion_free`. That closes
the key's side of the cofactor and malleability question for the device-signing
use the owner has ruled on: a key's torsion component is what lets cofactored
and cofactorless verification disagree about one signature under one key, and
what lets the holder of one secret present several keys that differ only by a
small-order point. With neither possible, what remains is the signature's own
encoding, which is the verifier's to check; `person` verifies no signature. No
new hazard: the rule only narrows which inputs are accepted, and a key
generated as a secret scalar times the base point has no torsion component.

**Twist points accepted as PersonPublicKeys (owner, 2026-10-05).** A canonical
u-coordinate of a point on Curve25519's quadratic twist (u = 2, 3 and 5, for
example) is not on the small-order blocklist, so a PersonPublicKey accepts it,
and so does the exported check. The rule matches RFC 7748, libsodium and the
key-agreement libraries the key interoperates with. X25519 is twist-secure: a
scalar multiplication by a twist point leaks nothing useful about the secret
scalar, so a twist key gives an attacker no leverage in key agreement. No
X25519 secret yields a twist point from the base point, so no honest holder
presents one; a twist key has no usable secret behind it, as a small-order key
has none. What sets the small-order keys apart, and why the rule rejects only
them, is that a key agreement against one yields a shared secret anyone can
predict; against a twist key it does not. Accepted; no hazard.

## Derived requirements assessment

REQ-r7mytp (names NFC-normalised and bounded): no hazard impact; it is
org-members' existing name rule, now owned here, and it bounds the input every
name operation processes.
assesses: REQ-r7mytp

REQ-q6xkna (DevicePublicKey only from a canonical ed25519 encoding of a point
of prime order) and REQ-4szc22 (device slots bounded, duplicate-free,
sorted): no hazard impact; they are the device-key
and device-slots validation org-members' controls for records from outside the
process and for the device bound already rely on. REQ-q6xkna is stricter than
org-members' rule: rejecting non-canonical encodings means one device cannot
take two slots under two encodings of one point, rejecting small-order points
removes keys no device holds a usable secret for, and rejecting points with a
torsion component removes the cofactor ambiguity set out above.
assesses: REQ-q6xkna, REQ-4szc22

REQ-tq4ms4 (device slots read from outside the process only when already
strictly increasing): no hazard impact; it is the validation org-members'
control for records from outside the process already relies on. Rejecting an
unordered or duplicated encoding, rather than sorting it, gives each set one
encoding, so a record read back is byte-identical to the one written.
assesses: REQ-tq4ms4

REQ-3vqs9b (PersonPublicKey only from a canonical, non-small-order X25519
encoding): will change which Member-as-a-group keys org-members accepts, when
it switches. No hazard is introduced by `person`: a Member-as-a-group key is
not authenticated by org-members and no control in its register validates the
key's curve; rejecting small-order keys removes a key with no usable secret
behind it, and twist keys stay accepted, as set out above. The switch's
effect on org-members' byte-comparing key-uniqueness rule, for a reused
keypair and for mixed-order keys, is set out above and is assessed by the
switch. Its cost is a migration of
key fixtures, borne by the switching change's tests, with no adopters to
migrate.
assesses: REQ-3vqs9b

REQ-mu3qgz (device root equal for equal sets, different for different sets
under one domain): no hazard impact; it is the device sub-trie's existing
property. Two different sets share a root only through a collision of the hash
function, so the property holds up to its collision resistance, which is the
hash function's, recorded with its SOUP item.
assesses: REQ-mu3qgz

REQ-aj6x3n (a different device root for the same set under a different
domain): no hazard impact; it makes the domain a parameter so that a Person's
device root and a member's coincide only through a collision of the hash
function, which bounds the separation as for REQ-mu3qgz.
assesses: REQ-aj6x3n

REQ-vxx8k3 (no panic for any input): no hazard impact; it is the robustness
rule org-members' control for malformed and hostile input states, applied to
the types now owned here.
assesses: REQ-vxx8k3

REQ-bczz87 (every rejection an error naming the rule broken): no hazard
impact; it is the typed-error half of the same robustness rule, so that a
caller can tell which rule an input broke and none is reported as another.
assesses: REQ-bczz87

REQ-7gz72r (exported X25519 validity check): no hazard impact; it exposes the
rule REQ-3vqs9b already applies, so that the organisation's key, defined in
org-node, is validated by the same code rather than a copy that could drift.
assesses: REQ-7gz72r

## Derived design behaviour assessment

LLR-fbqs2r (names redacted in `Debug`, unredacted in `Display` and in the
`String` conversion): no hazard impact, and a disclosure consideration recorded
here. A name and a surname are personal data; `Debug` is what diagnostics and
panic messages print, so it never shows them. `Display` and `String::from` do
write the value unredacted: they are how a caller gets a name out to show it
to someone entitled to see it, and a caller that formats a `Name` into a log
with `{}` rather than `{:?}` discloses it. Nothing in `person` does so; each
unit that uses the types owns that choice.
assesses: LLR-fbqs2r

LLR-a6krbh (`Debug` of the key types shows a four-byte hex prefix, of
`DeviceSlots` only the count): no hazard impact; a public key is not secret,
the prefix is enough to tell two keys apart in a diagnostic, and the device
count discloses no key.
assesses: LLR-a6krbh

LLR-78t363 (the key types compare, order and hash by their bytes): no hazard
impact; equality by bytes is the identity of a key, which REQ-q6xkna makes
one encoding per device by requiring the canonical encoding, and the byte
order is what makes a DeviceSlots' order, and so the device root
(REQ-mu3qgz), independent of the order keys were given in.
assesses: LLR-78t363

LLR-5za6mp (serde wire forms: keys as 32 raw bytes, names as strings, slots as
a sequence): no hazard impact; the forms are what decoding reads back through
the validating constructors (REQ-q6xkna, REQ-3vqs9b, REQ-r7mytp, REQ-4szc22),
so a changed or hostile encoding is rejected rather than trusted, and a
DeviceSlots encodes in the strictly increasing order its decoder requires. The
error for a non-sequence names only the expected form and the bound, no key.
assesses: LLR-5za6mp

LLR-z99qee (`NodeHash` wraps any 32 bytes, unvalidated): no hazard impact; a
hash output has no invalid values, and `person` produces a `NodeHash` only
from the implementor's hash functions and trusts none as proof of anything.
assesses: LLR-z99qee
