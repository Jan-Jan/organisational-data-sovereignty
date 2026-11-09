# org-io: risk analysis for stage S4 (transport, storage IO, invitation exchange)

Change `worktree-org-io-transport`. Drafted 2026-10-06 with the requirements
in `org-io/docs/requirements/2026-10-06-transport.md`.
Judged against this unit's acceptability matrix (`README.md`): S3 is
unacceptable at every probability (ruling D, 2026-10-06), so no S3 residual
here is called acceptable under the matrix.

Severity follows the method of org-node's hazard analysis, which this unit
inherits with the code it takes over: the worst credible harm of a pathway
that keeps a removed Member's access, or that leaves a Member unable to
receive or act on a membership change when a decision needs it, is S3, in
the deployments this project serves (journalism and government secrets,
`docs/adr/2026-09-01-safety-class-c.md`). The hazards of the items that move
here with their IDs (the endpoint binding scope, the invitation exchange,
the peer-address parse) stay analysed where they are written until the
develop phase moves them; this file analyses what S4 makes org-io newly
responsible for.

## The device secret at the endpoint and at the keychain

*Widened 2026-10-06, before first merge, by the owner's keychain rulings:*
the device secret now crosses org-io on two named move-only paths, the
endpoint (RC-pt49zn) and the OS keychain (RC-jgyd6m).

**HAZ-v9bu8b**: org-io binds each Persona's iroh endpoint with that Persona's
device ed25519 secret, because the endpoint id is the Device key, and reads
and writes that secret's OS keychain item, because keychain access is IO; a
path in org-io that copies, renders, serialises, stores or returns that
secret (a clone kept beside the endpoint, a `Debug` of a struct holding it,
the endpoint's secret-key accessor called for logging, the keychain's
returned bytes kept in a variable or logged, an endpoint handed to the
app) lets whoever reads it act as that Device: connect as it, receive the
next committed update addressed to it, which carries the Organisation's
private key, and so keep reading the Organisation after the Member or Device
is removed, or before it was ever admitted; a retained or gained access can
expose an identity. Severity: S3. Probability: P2.

P2 before control: the secret must be in org-io's hands for the endpoint
to exist at all, iroh's secret-key type is cloneable and serialisable and has
byte accessors, and the obvious seed type to reuse is cloneable too; one
convenience clone or `{:?}` is enough.

**RC-pt49zn**: org-io binds an endpoint in one function that reads the
Persona's device seed from the keychain into a zeroising, non-cloneable
buffer, builds iroh's secret key from it, moves that key into the endpoint
builder and zeroises the buffer before returning; iroh's secret-key type is
named nowhere else in org-io; org-io never clones, renders, serialises, logs
or stores the seed or the key, never calls the endpoint's secret-key
accessor, and never hands an endpoint to the app.
mitigates: HAZ-v9bu8b

*Rewritten 2026-10-06 before first merge (owner ruling of that day):* the
first design had org-node convert the seed into iroh's key through an
endpoint-secret type, which put `iroh-base` into org-node. The owner ruled
that org-io builds the key itself and org-node leaves the iroh path
entirely. Inherent safety by design all the same: the conversion is confined
to one function, and org-io's source is scanned for any use of the seed or
the key outside it. Implemented by REQ-xuzw32 (with the keychain read of
REQ-hw82ue). Verified in develop by the source scan in org-io's absence test
and by an endpoint test that the bound id is the Device key.

Residual: Severity S3, probability P1 after the control. **Not a risk put to
acceptance**: the owner ruled on 2026-10-06 that no device private signing
key is ever exposed, and this unit's S2 never-expose requirement states it.
Any route found later by which the secret leaves the endpoint through org-io
is a defect for `resolve-problem`. Stated for context only: the secret then
lives inside iroh (SOUP), which holds it for the TLS handshake and exposes an
accessor that org-io is checked never to call; process memory is outside
what this control can reach, and the in-memory wiping of seeds is org-node's
recorded residual on the Persona store. The same holds for the keychain path
under RC-jgyd6m below.

**RC-jgyd6m**: org-io reads and writes a Persona's device secret only as the
OS keychain item for that Persona's Device key; the bytes the keychain read
returns are moved, in the same function, into org-node's opaque,
non-cloneable, non-printable device-secret value, and on write the bytes are
lent to the keychain call only inside that value's single scoped write; org-io
never clones, compares, renders, logs, serialises or stores the bytes or the
value, keeps neither beyond the one org-node operation it was read for, and
writes the secret nowhere but the keychain.
mitigates: HAZ-v9bu8b

The second named move-only path (owner ruling, 2026-10-06), built the same
way as RC-pt49zn: the type removes the routes out, and the source scan pins
that the keychain call's result flows only into org-node's constructor or
RC-pt49zn's endpoint build.
Implemented by REQ-hw82ue, REQ-rz93uz and, on org-node's side, REQ-v6r2gd.
The bytes do exist in org-io's memory between the keychain call and the
constructor; that is the irreducible contact of IO with a secret, kept to one
expression, and it is not "holding" in the forbidden sense. In develop the
S2 never-expose requirement gets a dated note (owner ruling, 2026-10-06): the
device and signatory private keys never leave the process (no output, log,
error, `Debug` or `Display`, file, or IPC to the app), and in memory they are
only moved, never cloned, and zeroised after use.

## The stream a hostile peer reaches first

**HAZ-4n7pss**: org-io's receive loop reads inbound streams from anything
that can open a QUIC connection to the node; a peer that streams more than a
Wire message can hold, or claims a huge length in the prefix, makes the node
buffer without bound and exhaust its memory, or makes it hand org-node bytes
the prefix misdescribes; the node goes down and a Member cannot receive a
membership change, a revocation among them, when a decision needs it.
Severity: S3. Probability: P2.

P2: the input is attacker-chosen and the stream read is the first thing a
peer touches; nothing upstream bounds it.

**RC-5wz559**: org-io reads at most 1 MiB plus the four-byte length prefix
from any one inbound stream, refuses a longer stream with a typed error
without buffering beyond the bound and without handing any of it to
org-node, and discards the prefix rather than trusting it, so org-node sees
only the body and its own 1 MiB bound applies to exactly those bytes.
mitigates: HAZ-4n7pss

Implemented by REQ-szt6ea. Verified by the moved stream-bound tests (a stream
longer than the bound is refused rather than buffered; the prefix is not
checked against the body).

Residual: Severity S3, probability P1. Connection floods (many streams, each
within the bound) are not reduced by this control; iroh's connection limits
are SOUP behaviour, recorded on org-io's SOUP row in develop. **Accepted by
the owner on 2026-10-06** under ruling D (see "Residuals the owner
accepted").

### One stalled connection

*Added 2026-10-15 before first merge (S4 review round 2, finding 1;
owner ruling of that day: fix the mediums).* HAZ-4n7pss bounds
what one stream can make the node buffer; it says nothing of how long one
connection can hold the accept path, and the round-2 review found that one
connection could hold it for ever.

**HAZ-4hj5m9**: a Persona's endpoint reads one inbound connection at a time with no time limit, so any peer that completes the QUIC handshake (the accept path takes every one) and then opens no stream, or never finishes its body, holds the endpoint's only accept loop; while it holds it, no other connection to that Persona's Device is accepted, and a Member does not receive a membership change, a revocation among them, when a decision needs it. Severity: S3. Probability: P2.

S3 as HAZ-4n7pss: the harm is the same, a Member unable to receive a
membership change. P2: it needs no flood and no skill, one idle connection
from anything that can reach the endpoint, and the code before the fix
offered no bound at all.

**RC-96w7nf**: org-io reads each inbound connection in a task of its own and drops it, with a typed failure, when its handshake, its stream's open and its body have not completed within a fixed time limit (`CONNECTION_TIME_LIMIT`, 30 s); an endpoint reads at most a fixed number of connections at once (`CONNECTIONS_AT_ONCE`, 16), so a connection that stalls holds one slot, for at most the limit, and the bodies in flight stay bounded. mitigates: HAZ-4hj5m9

Implemented by REQ-mnd9rj (LLR-8uf9a5). Verified by
`a_stalled_connection_does_not_stop_another_peers_message`,
`a_stalled_connection_is_dropped_after_the_limit` and
`a_seventeenth_connection_waits_for_a_slot_while_sixteen_stall`
(`org-io/tests/endpoints.rs`).

What the control breaks, and why that is acceptable: (a) a sender on a path
too slow to complete the handshake and a 1 MiB body within 30 s is dropped;
it reads silence, which a send already reports as not delivered, and 30 s is
several times what that takes over a relayed path. (b) Reading connections
concurrently could let a flood buffer many bodies at once, which would
weaken RC-5wz559's memory bound; the slot count keeps it at sixteen bodies
of at most 1 MiB per endpoint, with the 64-body receive channel behind them,
so HAZ-4n7pss is not made worse. (c) Messages on different connections may
reach the receive step in another order than their connections arrived;
each is judged against the record on its own (org-node's check), as
messages from different peers always could be.

Residual: Severity S3, probability P1. Sixteen peers that stall together,
and stall again as each is dropped, still deny receipt on one endpoint while
they keep it up; that is a connection flood, HAZ-4n7pss's accepted residual
(denial of service while it lasts; the state stays recoverable from the
chain). Not accepted under the matrix (ruling D): pending the owner's
benefit-risk ruling, with the same reasoning as HAZ-4n7pss's acceptance.
*Accepted 2026-10-15 by the owner (benefit-risk, ruling D), on the same
reasoning as HAZ-4n7pss's acceptance of 2026-10-08: denial of receipt lasts
only while the flood does, and the state stays recoverable from the chain.*

## The receive loop and the service lock

**HAZ-esd966**: the receive loop waits for an inbound connection while it
holds the lock on the node's service state; with no peer sending, the lock
is held indefinitely, and every other command waits behind it, revoking a
Member or publishing an update among them; a Member the admin is trying to
remove keeps access for as long as the receiver waits. Severity: S3.
Probability: P3.

P3: before S4 it happens on every run of the app's receiver whenever no peer
is sending, which is most of the time (the app's problem report on the
receiver holding the lock, filed 2026-10-06).

**RC-97kdfv**: org-io's receive loop waits for connections and reads streams
holding no lock on the service state or the store, and takes that state only
for the synchronous org-node calls that judge and commit a received message
and for the store write after them.
mitigates: HAZ-esd966

Inherent safety by design: there is no lock to hold while waiting, because
the wait and the judgement are separated by ruling B. Implemented by
REQ-qkrj55. Verified by a test that a command needing the service completes
while the loop waits with no peer sending.

Residual: Severity S3, probability P1: the commands still serialise behind a
commit that is in progress, which is bounded by the synchronous org-node
call, the chain read happening outside the lock, and one store write.
**Accepted by the owner on 2026-10-06** under ruling D (see "Residuals the
owner accepted").

## Writing the sealed store

**HAZ-dd78f9**: org-io replaces the sealed Persona store on disk after every
change; a crash, a kill or a full disk during an in-place write leaves a
truncated file that no longer unseals, so the device loses every Persona
and Organisation key and record it holds; the Member's device can no longer
receive or act on membership changes, a revocation among them, until it is
admitted again with new keys. Severity: S3. Probability: P2.

P2: in-place writes are what org-node does today (its problem report on
non-atomic store writes, filed 2026-10-06), and the store is written on
every mutating operation, so the window recurs constantly; a laptop losing
power or a user force-quitting is ordinary.

**RC-37y46c**: org-io replaces the store file atomically: new sealed bytes
go to a new file in the same directory, are flushed to stable storage, and
are renamed over the old file, so an interruption leaves the old sealed
store or the new one, never a torn file; and a failed write is reported,
never success, with the unpersisted in-memory state discarded.
mitigates: HAZ-dd78f9

Implemented by REQ-zmtmd9 (the atomic replace) and REQ-g9cu2c (success only
after the write). Verified by a test that a write interrupted after the
temporary file is written leaves the old store unsealing, and by a failure
injection on the write that sees the error reported and the previous state
reloaded.

Residual: Severity S3, probability P1. Rename atomicity and the flush are
file-system behaviour (SOUP: the OS); a disk that lies about flushing can
still lose the last write, but not the previous good store. **Accepted by
the owner on 2026-10-06** under ruling D (see "Residuals the owner
accepted").

**HAZ-eyene4**: the store is now keyed by a secret held outside it, in the
OS keychain; when a Persona's sealed store exists but its keychain item is
missing (deleted by the user or a keychain reset, not yet restored, or on a
profile that lost it), a node that reads "no item" as "new Persona" draws a
new device secret and writes a new store over the old one, destroying every
Organisation key and record the old store held; the Member's device can no
longer receive or act on membership changes, a revocation among them, and
the old store could have been recovered had the item been restored.
Severity: S3. Probability: P2.

P2: keychain items are user-visible and user-deletable on every platform,
and "create when absent" is the natural first implementation.

**RC-hd5des**: org-io refuses to open a Persona's store whose keychain item
is missing, with a typed error naming the Persona, leaves the store file
untouched, and never creates a new device secret, store or Persona in its
place; a new Persona is created only by the explicit create operation, for
a Persona that has no store.
mitigates: HAZ-eyene4

Implemented by REQ-fh5gsq. Verified by a test against the mock keychain:
remove the item, try to open, see the refusal and the store file's bytes
unchanged.

Residual: Severity S3, probability P1: an item that is gone for good (never
restored) still loses the store, which is the cost of keeping no copy of the
device secret anywhere else; recovering it would need a second copy, which
the keychain ruling deliberately forbids. **Accepted by the owner on
2026-10-06** under ruling D (see "Residuals the owner accepted").

**HAZ-5n56w2**: org-io creates the sealed store file, and before S4 it was
created with the process's default permissions; another account on the same
machine reads it, and attacks the passphrase offline at leisure (the key
derivation salt is a fixed application constant, org-node's recorded
residual); a recovered passphrase yields every device seed and Organisation
private key, so the attacker reads the Organisation as its Member and keeps
reading after the Member is removed. Severity: S3. Probability: P2.

P2: shared machines exist in the deployments served, and default
permissions on common systems make a file in the data directory readable
by other local accounts.

*Restated 2026-10-07 before first merge (owner ruling of that day):*
HAZ-5n56w2 is controlled by RC-d8dbxs alone. An owner-only file-permission
control drafted here first was not accepted on 2026-10-06 as sufficient, and
on 2026-10-07 the owner dropped it altogether, on every platform: "I'm ok
with anyone being able to read the files on disk, because they should be
encrypted (i.e. simple file ACL is too weak a protection IMO)." Its ID is
not reused. Reading the file is therefore assumed possible for any local
account; what protects it is that its contents are sealed under a key the
file does not hold.

**RC-d8dbxs**: a Persona's store is sealed only under a key derived inside
org-node from that Persona's device ed25519 seed by HKDF-SHA-256 with a
fixed, versioned ODS store-key label (domain separation: the signing key is
never used as the encryption key); the device seed lives only in the OS
keychain and the store holds only the Device public key; no passphrase or
passphrase-derived key exists; and when the keychain is locked, missing or
unavailable, org-io refuses rather than fall back to any other key source.
mitigates: HAZ-5n56w2

*Noted 2026-10-07 (owner ruling of that day):* "refuses" applies to the
operations that need the keychain item (creating a Persona, opening or
reloading its store, binding its endpoint, signing); saving a change to an
open store seals under the store key held in memory and reads no keychain.
The control is unchanged: no operation ever takes a key from another source.

Implemented by REQ-6d9xv6 (refuse, never fall back), REQ-qza96g (the
expectation on org-node) and, in org-node, its exported answer. Verified by
org-node's tests that the store key is the HKDF output under the label and
differs from the signing key, that a store sealed under one device secret
does not unseal under another, and that no sealed store contains the device
seed; and by org-io's tests against the mock keychain for each refusal.

Re-assessment of HAZ-5n56w2 with RC-d8dbxs: the file alone is now 256 bits of
key away from its contents, with no passphrase and no shared salt to attack
offline; reading it needs the user's keychain item too, which other local
accounts cannot read. Residual: Severity S3, probability P1. What remains is
code running as the same user while the keychain is unlocked (on Linux's
Secret Service an unlocked collection is readable by any process of that
user; macOS and Windows scope items more narrowly), and a stolen device
whose keychain is unlocked. That residual is the same exposure every secret
the user holds in their keychain has; S8 reuses the mechanism for the
signatory key. **Accepted by the owner on 2026-10-06** under ruling D (see
"Residuals the owner accepted").

## Who receives a committed update

**HAZ-5qk64w**: a committed update is sent to one Device only; every other
Device of the Organisation keeps the previous record, in which a removed
Member's or Device's key is still listed, and goes on treating that key as
in good standing (answering it, sharing with it) until it hears of the
change some other way; the removed Member keeps access they should have
lost. Severity: S3. Probability: P3.

P3: before S4 every committed update reached one Device (the app's problem
report on the update fan-out, merged with the key-pair change); with more than
one other Device the rest are always stale.

**RC-wbs9ch**: in Networked mode, org-io sends every committed update to
every Device the committed record lists other than its own, and a
revocation to every Device the update removed, attempting each whatever
happened to the others and reporting each outcome.
mitigates: HAZ-5qk64w

Implemented by REQ-nactq6, with the per-recipient messages provided by
org-node (REQ-u894ty). Verified by an end-to-end test over two or more
recipient endpoints, one of them unreachable, in which every reachable
Device commits and the unreachable one is reported.

*As verified 2026-10-13 (S4, task T12; clarification):*
`networked_every_recipient_is_attempted_and_reported_in_order`
(`org-io/tests/send_fanout.rs`) sends over an in-process relay to three
recipients, one never bound: the two reachable Devices receive their bodies
from the sender's Device, the third is reported failed, all three in the
commit's order. That a receiver commits a body it receives is the receive
step's, verified there (`receive_step.rs`, `receive_order.rs`); the send test
stops at delivery.

Residual: Severity S3, probability P2 for a Device that is offline when the
update is sent: S4 sends once and does not retry or let a Device ask. That
gap is closed by later stages (S6, pull queries between Devices in good
standing; S7, gossip), not by S4. Loopback mode keeps one addressed
recipient by ruling (2026-10-06, Networked mode only). **Accepted by the
owner on 2026-10-06** under ruling D, at P2 until S6 and S7 land (see
"Residuals the owner accepted").

## Where a loopback send goes

**HAZ-ztkex9**: in Loopback mode org-io dials the peer address it is given;
the address travels out of band and is not authenticated, so an altered
address sends the message meant for a listed Device, which carries the
Organisation private key, to whichever endpoint the address names; a device
the record does not list holds the Organisation's private key and reads its
traffic. Severity: S3. Probability: P2.

P2: the address is copy-pasted between people, and nothing upstream
detects the alteration (org-node's open problem report on loopback sends to
an unchecked address, opened 2026-09-09, which moves here). *(Moved and
resolved 2026-10-13, S4 task T12, by the address check now stated by
LLR-34tfm8; reproduced by
`loopback_to_an_address_of_no_recipient_is_refused_and_sends_nothing`.)*

**RC-d6f25u**: org-io sends in Loopback mode only to an address whose
endpoint identity equals the recipient's Device key, and refuses, sending
nothing, otherwise; QUIC's handshake then proves the far end holds that key.
mitigates: HAZ-ztkex9

Implemented by REQ-8trkps. Verified by a test that a mismatching address is
refused with nothing sent, and that the matching one delivers.

Residual: Severity S3, probability P1: an attacker must now hold the
listed Device's secret, which is HAZ-v9bu8b's territory. **Accepted by the
owner on 2026-10-06** under ruling D (see "Residuals the owner accepted").

## A revoked Device's inbound connection

*Added 2026-10-07 before first merge (owner ruling 8 of that day).* Members accept inbound
connections from Devices they hold as revoked, for acknowledgements only.

**HAZ-abm3xs**: a Device a Member holds as revoked opens a connection to that
Member; if org-io's accept path acts on more than the acknowledgement (an
update or a revocation from it committed, or Organisation information, a
record, a key or a peer address written back on the connection), the removed
Device keeps or regains the Organisation's data and private key after its
removal, or steers a Member's record; and if the accept path instead refuses
such connections outright (an accept-time allow-list of listed Devices, or
acceptance over relayed paths only), the revoked Device cannot deliver its
acknowledgement to any Member, keeps its Device secret and the kept
addresses until its retry limit, and the Organisation never learns it acted
on its removal. Severity: S3. Probability: P2.

P2: the inbound side is the first thing a removed Device reaches, and both
mistakes are easy: a reply written "to be helpful" on the same stream, or an
allow-list added to cut connection floods.

## Residuals the owner accepted (2026-10-06)

By ruling D a residual at S3 is never acceptable under the matrix; it is
either accepted by the owner with its benefit-risk reasoning, or not
accepted. On 2026-10-06 the owner ruled on this draft's S3 residuals:

- **HAZ-4n7pss (stream flood), accepted.** Benefit: the node can receive
  membership changes from any peer at all, which is what keeps a Member's
  view current. Risk: a flood of in-bound streams can still slow or stop
  receipt; memory exhaustion by one stream is removed. A flood is denial of
  service within iroh's connection limits (SOUP), and an Organisation's
  state stays recoverable from the chain once it ends.
- **HAZ-esd966 (lock wait), accepted.** Benefit: received messages are
  judged and committed one at a time against one state. Risk: a command
  waits behind at most one commit in progress, a bounded synchronous call
  plus one store write, never behind the wait for a peer.
- **HAZ-dd78f9 (torn write), accepted.** Benefit: the store survives any
  interruption the OS reports honestly. Risk: hardware that acknowledges a
  flush it did not perform can lose the last write, never the previous good
  store; that is outside what software can control.
- **HAZ-5qk64w (offline Device), accepted at P2 until S6 and S7.** Benefit:
  S4 reaches every listed Device that is online, where before it reached
  one. Risk: a Device offline at the moment of sending stays stale until it
  next hears of the change; S6 (pull between Devices in good standing) and
  S7 (gossip) close this, and the acceptance lapses when they land.
- **HAZ-ztkex9 (loopback address), accepted.** Benefit: Loopback mode stays
  usable for single-host development and tests. Risk: a misdirected send now
  needs the listed Device's secret, which HAZ-v9bu8b's never-expose controls
  guard.

**Not accepted:** HAZ-5n56w2's residual under an owner-only file-permission
control alone; the owner ruled the keychain-keyed store instead (RC-d8dbxs),
and on 2026-10-07 dropped the file-permission control altogether.

Accepted later the same day, on the keychain design:

- **HAZ-5n56w2 under RC-d8dbxs, accepted.** Benefit: no passphrase-sealed
  file remains to attack offline, and other local accounts cannot read the
  keychain item. Risk: code running as the same user while the keychain is
  unlocked, or a stolen device with the keychain unlocked, can reach the
  seed. That is the exposure every secret in the user's keychain has, and S8
  relies on the same boundary for the signatory key.
- **HAZ-eyene4 under RC-hd5des, accepted.** Benefit: no second copy of the
  device seed exists anywhere to be stolen. Risk: a keychain item lost for
  good makes that Persona's store unrecoverable, and the Member is
  re-admitted with new keys. The store is never overwritten while the item
  might still be restored. Device-key backup and recovery is recorded as a
  question for a later stage (roadmap, next to S8).
HAZ-v9bu8b is not a risk put to acceptance (the never-expose ruling).

Accepted 2026-10-08 (owner ruling on plan §16.4 point 2):

- **HAZ-abm3xs under the revoked-Device inbound control (now RC-6m89tr), accepted.** Benefit: a revoked Device can
  deliver its signed acknowledgement directly to a Member, so the
  Organisation learns that the removal was acted on, and the revoked Device
  can end its delivery and delete its Device secret and kept addresses as
  soon as Members have accepted (S3 proposed 8), instead of holding them
  until its retry limit. Risk: what such a connection can achieve rests on
  org-node's member-sender rule (its own hazard and control, in org-node's
  risk file), so a defect there would be reachable from a revoked Device;
  and a revoked Device can open connections a Member must accept, a flood
  that is HAZ-4n7pss's accepted residual. Nothing but one verdict byte is
  written back, so no Organisation data leaves on the connection.

## Derived requirements

### REQ-bupq5y (one endpoint per Persona)

No new hazard; it removes one. Before S4 the second Persona's messages went
out under the first Persona's Device key and were refused, so an admission
moved the chain and the admin's record but never reached the joiner. Binding
per Persona can only send under the right key or fail to bind, which is
reported.

assesses: REQ-bupq5y

### REQ-xuzw32 (the device secret passes by move into the endpoint)

Assessed with RC-pt49zn above, which it implements.

assesses: REQ-xuzw32

### REQ-szt6ea (framing and the stream read bound)

Assessed with RC-5wz559 above, which it implements. The wire format is
unchanged, so no interoperability hazard arises with pre-S4 nodes.

assesses: REQ-szt6ea

### REQ-8e57wm (a typed terminal transport condition)

No new hazard; it removes a fragile one. A caller that classifies by message
text loops forever against a dead endpoint when the text changes, while the
UI reports the receiver as running, so updates stop arriving unnoticed. A
typed condition makes stopping a decision on a type, which a test pins.

assesses: REQ-8e57wm

### REQ-g9cu2c (only sealed bytes, written before success)

Assessed with RC-37y46c above, which it helps implement. Writing only
org-node's sealed bytes, uninterpreted, is what keeps org-io from ever
holding a device private key from the store (the never-expose ruling):
org-io cannot read inside what it writes. Discarding unpersisted state means
a failed write cannot leave the node acting on a commit that a restart would
undo.

assesses: REQ-g9cu2c

### LLR-85zque (the data directory created, every store opened in it)

*Moved 2026-10-13 from the app's derived-requirements assessment
(`app/docs/risk/2026-10-05-derived.md`), plan task T9, with the item, and
assessed again against org-io's hazards.* In the app it was assessed
against the app's hazards on the passphrase and the data directory, as the
step after those were decided: open at a fixed name, refuse rather than
substitute. That substance is unchanged.

Against HAZ-eyene4, opening refuses a store whose keychain item is
missing or whose keychain refuses, naming the store file and its Device
key, and creates no secret, store or Persona in its place (RC-hd5des,
through REQ-fh5gsq and LLR-uzax55). The item adds no path that reads "no
item" as "new Persona": only `create_persona` creates one. Against
HAZ-dd78f9, opening only reads the store files, and ignores every name
that is not a store file, so a leftover `.tmp-` file from an interrupted
write is never opened in place of the store (RC-37y46c, through
LLR-nt8dja). Creating the data directory writes no store. A refusal reaches
the app unchanged; the app's handling of a startup refusal is outside this
item. No new hazard, and no change to either control's effectiveness.

assesses: LLR-85zque

### The binding scope of the transport (REQ-db6s7q, REQ-2wzfzv; moved from org-node)

*Moved 2026-10-13 from org-node's hazard register
(`org-node/docs/risk/2026-09-09-org-node-hazards.md`, the section of
2026-10-03), plan task T10, with the two requirements, and assessed again
against org-io's hazards.* There they were assessed against org-node's
reachability hazards (an unlisted sender's push, a hostile Envelope, a
malformed frame): the wildcard bind they replaced had widened who could reach
a node in Loopback mode from "this machine" to "anything that routes to this
host", and the fix narrowed it back. That assessment minted no hazard and no
control, recorded that `RelayMode::Disabled` on the Loopback arm is
load-bearing and unverified (a home relay is acquired only after `online()`),
and booked a re-derivation of three probability figures written while the
register believed Loopback meant same-machine. Its full reasoning stays there
as history.

Against org-io's hazards: REQ-db6s7q narrows the population that reaches
HAZ-4n7pss (the first stream a hostile peer reaches) in Loopback mode to this
machine, and the stream read bound (RC-5wz559) holds whoever reaches it; and
it confines HAZ-ztkex9's Loopback dial by every IP path, the relay leg
argued, not verified (above). It realises no control of this draft and
weakens none. REQ-2wzfzv chooses a loud unavailability at startup (no IPv4
loopback socket, no endpoint) over an endpoint bound somewhere else, and
removes the needless one an IPv6-less host would have had; it bears on no
org-io hazard's severity or probability. No new hazard, and no new control:
a control reading "bind only loopback in Loopback mode" would restate
REQ-db6s7q without mitigating anything RC-5wz559 and RC-d6f25u do not.

assesses: REQ-db6s7q, REQ-2wzfzv

### LLR-2smrvx (the unchecked length prefix; moved from org-node)

*Moved 2026-10-13 from org-node's architecture-derived assessment
(`org-node/docs/risk/2026-10-03-architecture-derived.md`), plan task T10.*
The item is re-parented to REQ-szt6ea, which states it, so it is no longer
derived; the assessment is carried for the record. The read is bounded
before anything is decoded (LLR-k2y6nn), and org-node's decode refuses an
oversize or malformed body with a typed error; a prefix that disagrees with
the body changes neither the bytes read nor the bytes decoded, so a peer
cannot use it to make the receiver allocate more, accept a body it would
otherwise refuse, or read a second message from one stream (HAZ-4n7pss,
RC-5wz559 unchanged). No new hazard and no new control.

assesses: LLR-2smrvx

### REQ-pnr6kh (outstanding invites only in the sealed store)

No hazard impact beyond a benefit: the plain file disclosed which
Organisations had invitations out, and its writer was the second non-atomic
writer. Dropping it without migration loses invites outstanding at upgrade;
their replies are refused as not outstanding, which fails closed, and the
inviter issues a new Invite.

assesses: REQ-pnr6kh

### The expectations on org-node (REQ-x6kdse, REQ-v6r2gd, REQ-qza96g)

These state what org-io relies on org-node for under ruling B; each is met
by an exported org-node requirement. (A fifth, for an endpoint-secret type in
org-node, was dropped before first merge by the owner's ruling of 2026-10-06
that org-io builds the endpoint key itself.) For REQ-x6kdse, and for S4's three other expectations (since replaced by
the domain API's REQ-2yw5y3, REQ-u894ty and REQ-2xgsbn): no new
hazard. They move
behaviour that org-node already had behind IO calls to the same behaviour
returning values; the hazards of building, sealing, judging and committing
stay org-node's analysis. Their safety value is the split itself: org-io
cannot interpret the store (REQ-x6kdse), cannot choose recipients or message
kinds, cannot bypass the single receive path, and cannot forge an Invite or
settle one outside the store.

The two keychain expectations, added 2026-10-06 with the owner's keychain
rulings: REQ-v6r2gd (the transient device-secret value) is assessed with
RC-jgyd6m, and REQ-qza96g (the device-keyed store, no passphrase) with
RC-d8dbxs, both of which they help implement. REQ-x6kdse's "transient device
secret" replaces its passphrase; its assessment above is unchanged.

assesses: REQ-x6kdse, REQ-v6r2gd, REQ-qza96g
