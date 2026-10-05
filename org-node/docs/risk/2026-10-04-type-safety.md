# Risk — org-node type safety: secrets in diagnostic output, parsed persona details

Analysed 2026-10-04 for the change that brings org-node under the
parse-at-the-system-edge rule (ADR `docs/adr/2026-10-04-parse-at-the-system-edge.md`).
Severity follows this unit's register (`2026-09-09-org-node-hazards.md`,
"Method"): disclosure and unavailability are both S3 at worst, and the matrix
makes S3 unacceptable at every probability.

## Secret key material in diagnostic output

**HAZ-uy8sxm**: secret key material (a member seed, a device seed, an
Organisation secret or the store encryption key) is written in clear to a
diagnostic channel — a log line, a panic or assertion message, a crash report,
an error string returned to the app — through the debug formatting of a value
that holds it; that output lives outside the encrypted store or is shared (a
bug report, a support request, a CI log), and whoever reads it can impersonate
the Member on the transport, sign as the administrator if the persona is the
administrator's, and hold the Organisation secret. Severity: S3. Probability: P1.

P1, agreed with the owner (2026-10-04): no code path formats a record, the
store or a wire message today (PR-hqwpg9 records the latent leak); the
hazardous situation needs a later edit — a debug log added during an incident,
a failing `assert_eq!` on a wire message, an error that embeds a record.

**RC-8a4xjb**: each secret is held in its own type whose debug formatting
prints a fixed redaction marker and never the bytes, which has no display
formatting, and which gives its bytes up only through one explicitly named
accessor, so that every record, store and wire message holding a secret
redacts it in debug output by construction. mitigates: HAZ-uy8sxm

This is inherent safety by design, ISO 14971's first priority: the formatting
route is closed for every value that holds a secret, including values added
after this change, because a holder's derived `Debug` delegates to the secret
type's. Implemented by REQ-y7tsft.

**What the control deliberately does not do.** By owner ruling (2026-10-04),
secrets shared at Organisation level must stay serialisable: the Organisation
secret is carried to a Member in the Wire message and kept in the Persona
store, so the secret types serialise as the plain bytes they wrap. Serialising
is not formatting — the output goes to the store, which RC-jjsz97 encrypts, and
to the transport. Protecting the Organisation secret in transit is CGKA's
responsibility, outside org-node.

**Residual risk: S3/P1, not acceptable under this project's matrix**, as for
every S3 residual in this unit's register, whose overall conclusion
(UNACCEPTABLE) this does not change. The control is bounded by owner ruling
(2026-10-04): it closes the formatting route, and two routes are left to future
work. (Amended 2026-10-05 after independent review round 6: this residual was
first judged "acceptable for the formatting route", which contradicts the
matrix — S3 is unacceptable at every probability — and the register; closing
the formatting route reduces the risk but does not make an S3 residual
acceptable.) (1) The named accessor still
yields the bytes, and code that passes them to a formatter, or to a third-party
type with a revealing `Debug`, defeats the control; the accessor's name makes
each use searchable, and review is the control. (2) The secret is not wiped
from memory on drop, and copies pass through buffers no secret type owns (the
decrypted store, the encoded wire frame, the iroh secret key); a crash dump or
swapped page can hold them. That is the zeroise shortfall already recorded
against RC-jjsz97 in this unit's register and is not re-scored here.

**What the control breaks.** Nothing observable: no test, log or message reads
a secret through debug formatting today, and the serialised bytes are pinned
unchanged by the golden test (REQ-y7tsft's change). A future diagnostic that
needs to identify a secret must use something derived from it (a public key),
never the secret.

## A Persona store refused because it holds a value the rules refuse

**HAZ-vfjy32**: a Persona store written before this change holds a handle,
name or surname the member-record rules refuse, or a member, device or
Organisation public key that is not a curve point (in a member snapshot, an
Organisation record or a pending Invite); once Persona details and keys are
parsed on load, the store fails to open as a whole — for a key, where it was
previously never checked at open — and the Member cannot reach any
Organisation's material on that device until they re-create every Persona and
are re-admitted. Severity: S3. Probability: P1.

P1, agreed with the owner (2026-10-04): no Persona store has been deployed, and
after this change every value written to a store has already been parsed, so
only a store written by a development build before the change can hold one.

**RC-zutc67**: Persona details are parsed when the Persona is created, and
every key is parsed where it enters the software (Join request and Invite
import, the chain read, a record snapshot), so a store written by this
software holds only values that load, and a store, Join request, Invite or
record snapshot that is refused is refused as a whole, reporting which field
failed (an Invite, that it failed to decode). mitigates: HAZ-vfjy32

(Amended 2026-10-05 after independent review round 3: by owner ruling the
hazardous situation and the control include keys. Before this change a pending
Invite's keys and an Organisation record's keys were stored as unchecked bytes,
so a development store can hold an off-curve key there — for instance from an
imported Invite — and such a store now fails to open as a whole.)

Implemented by REQ-qn2erx. Migration or quarantine of an invalid Persona was
considered and ruled out by the owner (2026-10-04): it would be machinery for
data that does not exist.

**Residual risk: S3/P1, not acceptable under this project's matrix**, as for
every S3 residual in this unit's register, whose overall conclusion
(UNACCEPTABLE) this does not change. The control is bounded by owner ruling
(2026-10-04): no Persona store has been deployed, so no migration is provided,
and the break for development stores is known and is the owner's decision.
(Amended 2026-10-05 after independent review round 6: this residual was first
judged "acceptable while no Persona store has been deployed", which contradicts
the matrix — S3 is unacceptable at every probability — and the register; that
no store has been deployed is the reason for P1 and for providing no
migration, not a reason the residual is acceptable.) If a store is ever
distributed to users before this rule applies to it, this hazard must be
re-assessed — the probability is no longer P1 and the control would then need
a migration path.

**What the control breaks.** An invalid handle is now refused on the Member's
own device when the Persona is created, rather than on the administrator's
device at admission. The error moves earlier and to the person who can fix it;
no input that previously led to a successful admission is refused.

## Derived requirements assessment

assesses: REQ-y7tsft

REQ-y7tsft is the requirement for RC-8a4xjb, assessed above under
HAZ-uy8sxm. It adds no hazard: it removes an output and changes no serialised
byte.

assesses: REQ-qn2erx

REQ-qn2erx is the requirement for RC-zutc67, and it introduces HAZ-vfjy32,
assessed above. It changes when an invalid handle, name or surname is refused
(at Persona creation, store open and Join-request import, before any
admission), not which values are refused: the rules are org-members' member
record rules, unchanged.

(Amended 2026-10-05 after independent review round 3: the owner widened
REQ-qn2erx to keys, so this assessment now covers key refusals.) A member, device or Organisation public key
that is not a curve point is now refused when a store is opened, a Join
request or Invite is imported, or a record snapshot is decoded, where before
it was refused only when used (`Chain("bad member key: …")` and the like) or,
for a pending Invite's or an administrator's key compared only as bytes, never.
This adds no hazard beyond HAZ-vfjy32's: such a key decompresses to no point,
so no signature verifies under it and no device binds to it — nothing that
worked with it before is refused. The one new effect is that the refusal is of
the whole store, which is HAZ-vfjy32's situation, and its residual (S3/P1, not
acceptable) is unchanged, under the P1 given there: no Persona store has been
deployed, and
after this change no key reaches a store unparsed (each is parsed at Invite or
Join request import, at the chain read, or is the public half of a key pair
this software generated), so only a development store written before the
change — for instance one holding a crafted Invite — can hold one. (Amended
2026-10-05 after independent review round 6: this said the residual "stays
acceptable"; what holds is that the key refusals leave HAZ-vfjy32's residual
unchanged.)

(Amended 2026-10-05 after independent review round 4: REQ-qn2erx was narrowed
to each key's curve-point parse.) A member snapshot whose device key set breaks
org-members' rules (a repeated key, more than four) still opens, and is refused
when the members are rebuilt for an admission, a revocation or a Receive, as
before this change. That adds no hazard: the refusal happens before any Change
set is built or applied, and its effect is the unavailability HAZ-vfjy32
already covers, at a later step.

Parsing at creation also canonicalises a non-NFC handle, name or surname to
NFC there (LLR-q6n25z), so the Persona as stored, listed and exported in its
Join request holds the NFC form, and a non-NFC Persona in a development store
is rewritten in NFC on its next save (design ledger, "Observable changes").
This adds no hazard (assessed 2026-10-04 after independent review round 2):
the administrator's node already canonicalised these values at admission
through org-members' parse, so the admitted member record — what the root
hash commits to and what access is decided on — is byte for byte what it was.
What changes is only where the canonical form is first produced and what the
Member's own device shows for it, which is canonically equivalent text.

## The refactor itself

The rest of the change replaces plain values with types and is behaviour
preserving apart from the observable changes listed here. Two hazards could
follow from it; neither is new to this register, and each is controlled by
verification rather than by a runtime control.

- **Stored or transmitted bytes change.** A changed encoding would make a
  store unreadable (unavailability, HAZ-vfjy32's harm) or make peers refuse each
  other's messages and blobs (HAZ-5f9jcm's situation). The change pins the
  postcard bytes of the store plaintext, the Wire message, the Invite and Join
  request blobs, and the EVM calldata in a golden test written before the
  refactor and never edited afterwards.
- **A key parsed earlier than before.** The Organisation public key read from
  the chain is parsed into a curve point when it is read, so an invalid key
  there is reported as an invalid key at the chain read instead of as a failed
  Envelope verification later. Nothing that verified before is refused; the
  Envelope could not have verified under that key. Recorded as an observable
  change in error order. Member and device keys accept exactly what
  deserialisation accepted before (owner ruling); refusing weak and
  non-canonical keys is an open problem in org-members' ledger.
- **Refusals that move earlier.** The observable changes are listed in the
  design ledger (`org-node/docs/architecture/2026-10-04-type-safety.md`,
  "Observable changes"). Beyond those assessed here, an Invite holding a key
  that is not a curve point is refused at import rather than stored; such an
  Invite named a key no Envelope or device could match, so nothing that worked
  before is refused. Likewise a Persona store holding such a key in an
  Organisation record or a pending Invite is now refused at open (HAZ-vfjy32's
  situation, under the same P1: this software never wrote one), and a record
  snapshot or Join request with two defects reports the first in record order
  under its field's name; neither refuses a value that was used before.

## Derived design requirements assessment

assesses: LLR-56hc77

LLR-56hc77 (seeds become key pairs only through their own type) adds no hazard.
What it does: no seed passes through a plain byte array inside org-node, and a
`MemberSeed` and a `DeviceSeed` cannot be swapped for each other at the seed
level, because each is its own type; every seed it touches is a secret type, so
it keeps RC-8a4xjb's formatting control intact. The iroh secret key is built
from `DeviceSeed::expose_secret`, one of the deliberate uses that HAZ-uy8sxm's
residual names.

What it does not do (corrected 2026-10-04 after independent review, which
found the earlier text here claiming more): it does not remove the route by
which a Member key pair is used as a device key pair or the reverse — signing
or binding as the wrong identity. `SigningKeypair` carries no role, and both
`member_seed()` and `device_seed()` are public on it, so
`member_kp.device_seed()` compiles, and a key pair of the wrong role can still
be passed wherever a `&SigningKeypair` is taken — `OrgEndpoint::bind` /
`bind_with_mode` (reached from `OrgService::ensure_endpoint`, which today
passes the device key pair) or `SignedDeltaEnvelope::build`. Residual: that
swap is prevented by review and by the tests of the call sites, not by the
type system. Recorded as PR-4b2v6p; role-typed key pairs are a follow-up
change by owner ruling (2026-10-04), not this one.

assesses: LLR-mmdu38

LLR-mmdu38 (the Organisation public key parsed as a curve point) adds no
hazard; the earlier refusal it causes at the chain read is assessed under "The
refactor itself" above. Naming the type for the key's intended role does not
change which key Envelopes verify under; that conflation is PR-szkat6.
Its `Debug` clause (the type name and the first four bytes in hex) adds no
hazard either: the Organisation public key is not secret, and truncation only
shortens diagnostics; the full key stays available through `as_bytes`.
(Amended 2026-10-05 by the org-node type-safety change, review round 8: the
`Debug` clause was added to LLR-mmdu38 and is assessed here.)

(Amended 2026-10-05 after independent review round 3: the earlier refusal
introduced a defect, now fixed — `OnChainReader::refresh` returned on the parse
failure before updating its cache, so the state of the last successful refresh
stayed cached and verify-against-chain went on verifying against a root and
epoch the chain had superseded — HAZ-tawvm2's situation (an Organisation
state older than the current one): an Envelope the current state would refuse,
one reversing a removal for instance, could be accepted.) By owner ruling a
chain state refused at parse now leaves no cached state: the reader returns no
state until a refresh succeeds, so verification refuses with `OrgNotOnChain`.
That fails closed. Its cost is unavailability of verification through this
reader while the chain holds an Organisation public key that is not a curve
point — a state under which no Envelope could verify anyway, so nothing that
would have been accepted correctly is refused. `SubxtChainOps::read_state`,
which the production Receive operation uses, caches nothing: each Receive
operation reads afresh and a refusal at parse aborts it, so it has no
superseded state to serve.

assesses: LLR-s7whrn

LLR-s7whrn (tag types for chain accounts, Persona identifiers, epochs and
Sequence numbers) adds no hazard: every value any of them accepts was accepted
before. It removes the swap of an epoch for a Sequence number, which would
have defeated RC-m4r75s or the on-chain epoch check silently, and the swap of a
chain account for a root or a key. Its `Debug` clause adds no hazard: a chain
account, a Persona identifier, an epoch and a Sequence number are not secret,
and each renders in full, as the plain value did before, wrapped in its type
name. (Amended 2026-10-05 by the org-node type-safety change, review round 8:
the `Debug` clause was added to LLR-s7whrn and is assessed here.)

assesses: LLR-ayrdr8

LLR-ayrdr8 (pinned bytes) is the verification obligation named under "The
refactor itself": it is what shows that the store, the Wire message, the blobs
and the calldata are unchanged. It adds no hazard. Its limit: it pins fixed
values, so it shows that each type serialises as before for those values, not
for every value.
