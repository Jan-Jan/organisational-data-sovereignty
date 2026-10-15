# org-io: transport, storage IO and the invitation exchange (stage S4)

Change `worktree-org-io-transport`, stage S4 of
`docs/plans/2026-10-06-org-io-roadmap.md`. Plan:
`docs/plans/2026-10-06-org-io-transport.md` (§12 lists these items). Drafted
2026-10-06, before any code. The key-pair change those rulings waited for
merged on 2026-10-07 (`1f52c36`); S4's code starts on top of S2's crate, once
S2 has merged (plan, task T0). Until then each requirement below is reported
MISSING-TEST, which is the truth.

These are the requirements S4 makes org-io newly responsible for: the iroh
endpoint and the stream it reads, the receive loop, writing the sealed
Persona store to disk, running the invitation exchange, and sending a
committed update to every Device. The owner's rulings of 2026-10-06 on the
S2–S4 drafts (the roadmap's section of that name) are binding here and are
not re-argued.

*Reconciled 2026-10-08 with S2 as merged (`e41cb09`).* org-io's crate,
`OrgIo` (`open`, `connect`, `view`, `node_mut`, `found_organisation`,
`submit_commit_send`, `receive_and_verify`,
`receive_and_self_delete_if_revoked`, `is_own_admin`) and its ledgers
(`2026-10-08-org-io.md`) are merged. S2's never-expose requirement
(REQ-v4tfap) and its scans bind every item below: any public method S4 adds
to `OrgIo` or its view returns only types on S2's reviewed allowlist
(`the_handle_returns_only_allowlisted_public_types`), and no device or
signatory private key, and no X25519 secret, is reachable through a public
operation; public keys may be. The owner's rulings of 2026-10-08 on the
plan's §16.4 are applied below (REQ-ur7z4a, REQ-hw82ue) and in the risk
draft (HAZ-abm3xs).

## What moves here rather than being written here

By ruling A an item that moves into org-io keeps its ID, and its definition
moves into this unit's ledgers with a dated note, in the develop phase. The
list, by ID and source, is the plan's §3.1 and §3.5: org-node's loopback
binding-scope requirements and the endpoint, framing and dialling design
items; the app's invitation requirements (producing an Invite, producing a
reply, refusing a reply that is not outstanding) with their design items;
the app's peer-address parse items; and the open problem reports on the
endpoint bound once and on the unchecked loopback address, which S4 resolves
here. Until those definitions arrive this file names none of their IDs:
org-node exports nothing yet, and org-io does not depend on the app.
*(2026-10-13, task T10: the binding-scope requirements have arrived, under
"The binding scope of the transport" below, and the endpoint-bound-once
report in org-io's problems ledger.)* *(2026-10-14, task T13: the app's
three invitation requirements have arrived, under "The invitation exchange"
below.)*

## Transport

**REQ-bupq5y**: The software shall bind at most one transport endpoint per
Persona, with that Persona's Device key as the endpoint identity, and shall
send each message from the endpoint of the Persona bound to the Organisation
the message is about, never from another Persona's endpoint.
satisfies: derived

Ruled 2026-10-06 (one endpoint per Persona). It replaces org-node's
bind-once-per-service, under which a device holding two Personas sent the
second Organisation's messages under the first Persona's Device key, and the
recipient refused them. Because the endpoint identity is the Device key, a
completed handshake proves custody of that key; nothing on the receiving side
checks it (the "no sender checks" rulings), but the recipient's own dial and
the Organisation's peers rely on it.

*Amended 2026-10-07 before first merge (reconciled with S3a, `60c7d58`):*
the receiving side does check it now. Owner ruling R1 of 2026-10-07 (S3a's
member-sender rule, a control in org-node's risk file) acts on an update for a held
Organisation and on a revocation only from a Device the record lists, and
on an acknowledgement only from the Device it names; org-io hands org-node
the authenticated key with the body (REQ-fnsjb6).

**REQ-xuzw32**: The software shall bind a Persona's endpoint from that
Persona's device seed in one function: it shall read the seed from the
Persona's OS keychain item (REQ-hw82ue) into a zeroising, non-cloneable
buffer, build iroh's secret key from it, move that key into the endpoint
builder, and zeroise the buffer before the function returns; it shall not
clone the seed or the key, render either through `Debug` or `Display`,
serialise, log or store either outside the endpoint, call the endpoint's own
secret-key accessor, or hand an endpoint to the app. (implements: RC-pt49zn)
satisfies: derived

*Rewritten 2026-10-06 before first merge (owner ruling of that day,
superseding this draft's first design):* org-io builds iroh's key itself;
org-node has no iroh dependency and no endpoint-secret type, and the
expectation that asked org-node for one is dropped. How this squares with the
never-expose ruling: the iroh endpoint id is the Device key, so the endpoint
must hold the device's ed25519 secret (iroh signs its TLS handshake with it),
and the endpoint is org-io's. This item is the whole of org-io's contact with
the seed on that path: read, converted and moved into the SOUP that must hold
it, inside one function, with the buffer zeroised on the way out. That is
the first of two named move-only paths (the second is the keychain path to
org-node, REQ-hw82ue). The S2 never-expose item gets its dated note in
develop: the device and signatory private keys never leave the process (no
output, log, error, `Debug` or `Display`, file, or IPC to the app), and in
memory they are only moved, never cloned, and zeroised after use. Verified
by a source scan in org-io's absence test (iroh's secret-key type and its
constructor are named in that one function only; no `to_bytes`, `secret`,
`clone` or `{:?}` on it; no call of the endpoint's secret-key accessor; no
endpoint type in an IPC-facing type), and by a test that binding twice for
one Persona yields the same endpoint id.

**REQ-szt6ea**: The software shall carry each message on its own stream as a
four-byte little-endian length followed by the body, shall read at most
1 MiB plus those four bytes from one inbound stream, and shall refuse a
longer stream with a typed error without buffering beyond that bound and
without handing any of it to org-node; the length field shall be discarded,
not trusted, and the body handed on unchanged. (implements: RC-5wz559)
satisfies: derived

The wire format is today's, so S4 and pre-S4 nodes interoperate (the ALPN is
unchanged). The 1 MiB bound on the body itself stays org-node's, on encode
and decode; this is the stream read in front of it, which is now the first
thing a hostile peer reaches.

**REQ-mnd9rj**: The software shall read each inbound connection apart from
every other, shall drop a connection whose handshake, stream and body have
not completed within a fixed time limit, reporting it as that connection's
failure, and shall read at most a fixed number of one endpoint's
connections at once, so that a connection that stalls cannot stop another
peer's message being received. (implements: RC-96w7nf)

*Added 2026-10-15 before first merge (S4 review round 2, finding 1;
PR-d2p5y2).* The limit and the count are LLR-8uf9a5's constants.

### The binding scope of the transport (moved from org-node)

*Moved 2026-10-13 from org-node
(`requirements/2026-10-03-transport-binding-scope.md`), plan task T10
(ruling A):* the two Loopback binding requirements, minted while resolving
org-node's loopback-timeout report, with their verification notes. The
history of how they were found, the gaps' measurements and the RMF's answer
stay in that file as prose; the assessments of both moved to org-io's S4 risk
draft. Their tests moved to `org-io/tests/transport_handshake.rs` and
`org-io/tests/endpoints.rs`, where every endpoint is bound from a keychain
item.

**REQ-db6s7q**: The software shall, when its transport is configured in
Loopback mode, bind its endpoint only to loopback addresses, and shall offer a
peer for dialling only loopback addresses.
satisfies: derived

Verified by two tests in `org-io/tests/transport_handshake.rs`, both of which
assert the bound sockets and the advertised address set directly rather than
inferring the binding from a delivery timeout:
`loopback_mode_binds_and_advertises_loopback_only` (the normal path) and
`loopback_mode_holds_under_repeated_and_colliding_binds` (the robustness case:
eight endpoints alive at once in one process, two of them claiming the same
Device key; exactly one IPv4 loopback socket, required; whatever IPv6 socket
exists, loopback), and by `a_service_in_loopback_mode_binds_loopback_sockets_only`
in `org-io/tests/endpoints.rs`. One clause cannot fail independently of the
other: `node_addr_for_dial()` is built from `bound_sockets()`.

Binding loopback is not the whole of "offline": an endpoint bound entirely to
loopback can still reach the network by a relay or by publishing its address
to a discovery service. The tests close address publication
(`address_lookup()` must be empty); **the relay is not asserted**, because a
home relay is acquired only after `online()`. So `RelayMode::Disabled` on the
Loopback arm is load-bearing and unverified, a gap recorded in org-node's
verification record of 2026-10-03 and carried with the requirement.

**REQ-2wzfzv**: The software shall, when its transport is configured in
Loopback mode, fail to provide an endpoint when it cannot bind an IPv4
loopback socket, and shall provide one when it cannot bind an IPv6 loopback
socket.
satisfies: derived

The asymmetry is deliberate: `BindOpts::is_required` defaults to true, so a
mandatory `[::1]:0` would abort the whole endpoint on an IPv6-less host;
IPv4 loopback stays required, because without it there is no usable Loopback
transport and failing closed is correct. **Neither clause is verified by any
gated test**: both are about bind failure, which no test can produce on a host
where both loopback sockets bind (measured by org-node's review round 3: the
exact negation of the first clause left the tests green). What it rests on is
a reading of iroh's source (`BindOpts::is_required` is honoured in
`socket/transports/ip.rs`). Making it testable needs a `test-support`
constructor that accepts bind addresses; booked, not added.

**REQ-8e57wm**: The software shall report a transport failure that ends the
receive loop (the endpoint closed, or binding failed) as a typed condition
distinct from every other receive failure, so that its caller decides to stop
from the condition's type and never from the text of a message.
satisfies: REQ-x3c8n2
exported: yes

The app's expectation on org-node for a typed terminal error is re-targeted
to org-io in develop, when the app's receiver calls org-io; this item is what
answers it there.

*Amended 2026-10-14 (S4, task T16).* Exported, and `satisfies:` names the
app's expectation (was `derived`): the app's receiver now stops on
`ReceiveError::Terminal` alone. The requirement's text is unchanged.

**REQ-qkrj55**: The software shall hold no lock on the node's service state
or store while it waits for an inbound connection or reads an inbound stream,
and shall take that state only for the synchronous org-node calls that judge
and commit a received message and for the store write that follows them.
(implements: RC-97kdfv)

Ruled 2026-10-06 (the receive loop holds no lock while it waits). Fixes the
app's problem report on the receiver holding the service lock (filed
2026-10-06 on this branch), whose loop moves here. Verified by a test that
runs a command needing the service while the loop is waiting, with no peer
sending, and sees it complete within a bound.

## Receiving

**REQ-fnsjb6**: The software shall pass every received message body through
org-node's single receive path, whatever the message is (a first admission
the user declared, an update to an Organisation this node holds, or the
node's own revocation), handing org-node exactly the body bytes and nothing
derived from the connection, and shall read the chain for a message only
after org-node's chain-free checks have accepted it, at most once per
message, then hand org-node the state it read.
satisfies: derived

Ruled 2026-10-06 (one receive path). Fixes the app's problem report that an
invitee never commits its own first admission, because the app's loop called
only the self-delete entry point. The chain read's position after the
chain-free checks is what keeps a peer from driving chain reads at will; the
moved first-admission gate is org-node's, and this item is org-io's half.

*Amended 2026-10-07 before first merge (reconciled with S3a, `60c7d58`):*
"handing org-node exactly the body bytes and nothing derived from the
connection" becomes "handing org-node exactly the body bytes and the
DevicePublicKey the transport authenticated for the connection, and nothing
else derived from it". S3a's member-sender rule (owner ruling R1, org-node's
org-node's update and acknowledgement requirements as amended) decides on that key, so org-node must
receive it; org-io neither checks it nor alters it. "Whatever the message is"
now also covers a revocation notice and a revoked Device's acknowledgement
(S3a's two new Wire message kinds); an acknowledgement is settled by
org-node's check alone and costs no chain read.

## Acknowledgements from revoked Devices

**REQ-ur7z4a**: The software shall accept an inbound connection from a
Device that a held Organisation's record no longer lists (a Device the node
holds as revoked) and act on a message from it only when it is an
acknowledgement, handed to org-node's check with the connection's
authenticated DevicePublicKey; it shall refuse every other message kind
from such a Device before any chain read or store write; and it shall write
on any inbound connection nothing but a single verdict that the message was
accepted or refused, serving no Organisation information, record, key,
address or other Organisation data to the connecting Device.
(implements: RC-b4m5sj)
satisfies: derived

Owner ruling of 2026-10-07 (plan §header, ruling 8): "members accept
INBOUND connections from devices they hold as revoked, for acknowledgements
ONLY". A revoked Device connects directly to up to three distinct Member
Devices at addresses it kept, with no relay, and tries the next address when
one refuses; that delivery, and keeping the addresses, are S3b-io's (S3 plan
T13–T17, its proposed 8 and 9). This item is the receiving side S4 provides.
Which messages from such a Device are acted on is org-node's member-sender
rule (S3a, owner ruling R1: an update and a revocation only from a listed
Device, an acknowledgement only from the Device it names, which the
receiver holds as removed); this item keeps the transport from narrowing
that (no allow-list at accept time, no relay-only acceptance) or widening it
(nothing served back). The verdict is what tells the revoked Device to try
its next address; a Member that has not yet committed the revocation still
lists the Device and refuses (`AcknowledgementForListedDevice`).

*Clarified 2026-10-13 (S4, task T11a):* a Member behind the revocation
refuses as stated, but org-node judges the acknowledgement's epoch before
the listing, so the refusal it reports is `AcknowledgementFromFuture`; the
verdict is `Refused` either way. Tests, through the design items
(`org-io/tests/acknowledgement_inbound.rs`):
`a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted`,
`a_revoked_device_sending_anything_but_an_acknowledgement_is_refused_and_served_nothing`,
`an_acknowledgement_a_member_still_listing_the_device_refuses_is_answered_refused`,
`an_acknowledgement_relayed_by_another_device_is_refused`,
`a_dropped_responder_writes_nothing`, `the_accept_path_has_no_allow_list`,
`nothing_but_the_verdict_is_written_on_an_inbound_stream`.

*Ruled 2026-10-08 (owner, plan §16.4 point 1):* the verdict is one byte on
the message's own stream, accepted or refused, and nothing else; the ALPN is
unchanged, and a pre-S4 sender, which does not read it, ignores it.

## Storage

**REQ-g9cu2c**: The software shall write to local storage only the sealed
store bytes an org-node operation returned, unmodified and uninterpreted,
and shall write them before it reports that operation's success; when the
write fails it shall report the failure, shall not report success, and shall
discard the in-memory node state that was not persisted, so that what it
holds never runs ahead of what is on disk.
satisfies: derived

Ruling B: org-node returns the sealed bytes, org-io writes them. The "store
is written before it returns" promises of org-node's commit and create paths
are kept at org-io's API by this item.

**REQ-zmtmd9**: The software shall replace the store file atomically: it
shall write the new sealed bytes to a new file in the store's directory,
flush it to stable storage, and rename it over the store file, so that an
interrupted write leaves either the previous sealed store or the new one in
place and never a truncated or mixed file. (implements: RC-37y46c)

Ruled 2026-10-06 (store writes atomic). Fixes org-node's problem report on
non-atomic store writes, filed on this branch.

*An owner-only file-permission requirement drafted here was dropped before
first merge (owner ruling, 2026-10-07): "I'm ok with anyone being able to
read the files on disk, because they should be encrypted (i.e. simple file
ACL is too weak a protection IMO)." The store file's confidentiality rests
on the keychain-derived store key alone (RC-d8dbxs); org-io sets no file
mode or ACL on any platform. Its ID is not reused.*

## The device key in the OS keychain

Owner rulings of 2026-10-06 (on this draft's residual for the store file
readable by other accounts, which the owner did not accept): the local store
is encrypted under a key derived from the device ed25519 key; the device's
private seed lives **only** in the OS keychain (or a secure enclave or
hardware key behind it), one item per Persona's Device key; org-node's sealed
store keeps the Device **public** key only; org-node uses the private seed
transiently, per operation (deriving the store key, signing), and persists
neither it nor anything derived from it; org-io builds iroh's key from it
itself, for the endpoint (REQ-xuzw32); the
signing key is never reused as an encryption key, because the store key comes
from a separate, domain-separated derivation. No passphrase-sealed file
remains to attack offline. Other secrets (the Organisation's X25519 private
key, each Persona's X25519 member seed) stay in the sealed store, now under
the keychain-derived key.

Keychain access is IO, so it is org-io's (ruling B); the seed is a device
private key, so org-io must not hold it in the forbidden sense (the
never-expose ruling). The path to org-node is therefore the **second named
move-only path**, built like the endpoint path: org-io handles the item only
as a zeroising, non-cloneable value that it moves between the keychain call
and its one use (org-node's operation, or the endpoint builder of
REQ-xuzw32) in one function.

**Platforms** (owner ruling, 2026-10-07): the software has to work on
Windows, macOS, Linux, Android and iOS, each through that platform's OS
keychain (Windows Credential Manager, macOS and iOS Keychain Services, Linux
Secret Service, Android Keystore). Interacting with the OS keychain happens
in org-io only, to protect the lower layers from it. Which platforms S4's
keychain crate covers, and which are left to a later stage (Android, whose
keychain the crate is expected to lack; Windows, if it adds complexity), is
decided and recorded by plan task T7; a platform without a keychain backend
refuses under REQ-6d9xv6 and never falls back.

**REQ-hw82ue**: The software shall read a Persona's device secret only from
the OS keychain item for that Persona's Device key, and shall move the bytes
the keychain call returns, in the same function and without binding them to
any other use, either into org-node's opaque device-secret value, which it
then moves into the one org-node operation that needs it, or into the
endpoint build of REQ-xuzw32; it shall not clone, compare, render, log,
serialise or store those bytes or that value, shall keep neither beyond the
operation it was read for, and shall zeroise the bytes when that operation
ends. org-io shall be the only unit that calls the OS keychain: org-node and
every unit below it receive a device secret only as org-node's opaque value
and never touch the keychain. (implements: RC-jgyd6m)

*Clarified 2026-10-08 before first merge (owner ruling of that day, plan
§16.4 point 4):* "only from the OS keychain item" covers every reader of a
device secret in org-io, the operator preflight included: no device key or
seed is read from an environment variable, in any build (org-io's tests
supply device secrets through the in-memory keychain of `test-support`, not
the environment). The owner:
"ODE_DEVICE_KEY is only for testing environments (and only used for on-chain
signing)" — environment-variable keys and seeds belong to test environments
only, and the one org-io reads, `ODS_ADMIN_SEED`, is development-only
(`dev-seed`) and signs on-chain only (S2's REQ-8zuka3). How a user comes to
hold a device key (created automatically into the OS keychain or a hardware
key, or an existing key the user imports into one) is future work (roadmap,
"(later)" rows); until then `create_persona` creates it (REQ-rz93uz).

*Amended 2026-10-07 before first merge (owner ruling of that day):* the last
sentence makes "keychain access in org-io only" explicit. Verified by
org-io's absence test, which also reads the manifests of org-node, person,
org-members and on-chain-client and finds no keychain crate among their
dependencies.

**REQ-rz93uz**: The software shall write a new Persona's device secret only
to the OS keychain, as one item named for that Persona's Device key, by
handing org-node's opaque device-secret value to the keychain call through
the value's single scoped write operation in one function, and shall write
the device secret to no other place: not the sealed store, not any other
file, not the environment, not the app. (implements: RC-jgyd6m)

Both are verified by a source scan in org-io's absence test (the keychain
crate is named in one module; its read result flows only into org-node's
constructor or the endpoint build; its write receives bytes only inside the scoped operation; no
`Clone`, `{:?}`, `to_vec` or `Serialize` reaches either), and by tests
against an in-memory credential store compiled only under `test-support` and
never into a release build: the keychain crate's own mock if it shares an
item between handles for the same name, otherwise org-io's own stand-in
(owner ruling, 2026-10-07; checked in plan task T7).

One sealed store per Persona, each keyed by its own Persona's Device key
(owner ruling, 2026-10-06): org-io keeps one store file per Persona and one
keychain item per Persona. On a Linux host with no Secret Service (headless,
CI, servers) the node refuses to start under REQ-6d9xv6 (owner ruling,
2026-10-06); the in-memory credential store tests use exists only under
`test-support`.

**REQ-6d9xv6**: The software shall refuse, with a typed error naming the
Persona and the condition, each operation that needs that Persona's OS
keychain item (creating the Persona, opening or reloading its store, binding
its endpoint, and signing with its device key) when the OS keychain is
locked, unavailable, or refuses access to that item, and shall never fall
back to a passphrase, a plaintext file, an environment variable or any other
key source. (implements: RC-d8dbxs)

*Narrowed 2026-10-07 before first merge (owner ruling of that day):* the
first wording refused to "open, create or write" a store. Saving a change to
a store that is already open needs no keychain: it seals under the store key
the open store holds in memory (org-node's REQ-qthum6 design), so an
ordinary save neither reads the keychain nor refuses when it has since been
locked. Only the four operations named above read or write the item.

**REQ-fh5gsq**: The software shall, when a Persona's sealed store exists but
the keychain holds no item for its Device key, refuse to open it with a typed
error naming the Persona, leave the store file untouched, and create no new
device secret, store or Persona in its place. (implements: RC-hd5des)

## The invitation exchange

**REQ-a2a6rm**: The software shall act on an Invite reply the user chose to
admit in this order: have org-node build the admission as a provisional
update, submit the chain write, have org-node commit the update against the
state read back from the chain, send it to every recipient REQ-6skn67 names,
and settle the reply's invite identifier; it shall settle once the admission
has committed whether or not any send succeeded, shall stop before the chain
write if building the admission fails, and shall write the sealed store
after each step that changes it.
satisfies: derived

Ruled 2026-10-06 (the invitation exchange moves in S4: org-node builds and
serialises, org-io holds the store and runs the exchange, the app keeps the
UI). The rules on which replies may be acted on (outstanding, for the
Organisation the Invite was issued for) move here with their IDs; this item
is the sequence that joins them to the chain write and the send.

**REQ-pnr6kh**: The software shall keep invite identifiers it holds as
outstanding only inside the sealed Persona store, write no other file for
them, and never read a separate outstanding-invites file.
satisfies: derived

Ruled 2026-10-06 (outstanding invites move into the encrypted store, no
migration). An invite outstanding in the old plain file at upgrade is lost:
its reply is refused as not outstanding, and the inviter issues a new Invite.

*Moved 2026-10-14 from app (`app/docs/requirements/2026-10-06-invitation.md`),
plan task T13 of docs/plans/2026-10-06-org-io-transport.md (ruling A).* The three requirements below — producing an Invite, producing a
reply, refusing a reply that is not outstanding and settling — came with
their IDs, their text and their notes, as written in the app's change
`worktree-org-node-chain-authority` and amended since; their low-level items
moved with them, to this unit's architecture draft under the invitation
exchange runner (SDD-rr7dtw). "org-node" in them is still org-node: org-io
declares the expectation and keeps the outstanding pairs through org-node's
operations (REQ-txkm94), in the sealed store (REQ-pnr6kh). Each is marked
`exported: yes`, as the submission requirement S2 moved here is, because the
app still decides when to issue, reply and admit and cites them.

**REQ-prjja8**: The software shall produce, for an Organisation the user holds,
an Invite as a Blob carrying the Organisation's name as the inviter typed it, the
Organisation identifier, the inviter's guess of the invitee's full name, the
inviter's DevicePublicKeys, and an invite identifier of 32 bytes drawn from its
cryptographic random source, and shall keep that identifier as outstanding.
satisfies: derived
exported: yes

The Organisation's name is free text for display. It is stored nowhere and
verified by nothing; the Organisation identifier is what the invitee's node
reads the chain with (owner ruling 2026-10-05). The inviter's DevicePublicKeys are
where the reply will be sent once the transport moves to the app; the invite
identifier lets the inviter discard any reply it did not ask for (owner
journey, 2026-10-05). *(2026-10-14, moved: the transport moved to org-io, not
the app; the Device public keys are those of the Persona bound to the
Organisation, which org-node's `build_invite` reads.)*

**REQ-tcutr6**: The software shall produce, from an Invite the user imported
and a Persona the user chose, an Invite reply as a Blob carrying that
Persona's Member-as-a-group key, DevicePublicKey, handle, name and surname, and the Organisation identifier and
invite identifier the Invite named; and shall declare to org-node that it
expects a first admission to that Organisation.
satisfies: derived
exported: yes

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The declaration named the invite identifier as well, which the admission's
Wire message carried. The owner ruled that the invite identifier travels only
in the Invite and its reply, never between peers; org-node matches a first
admission against the Organisation alone
(`org-node/docs/requirements/2026-10-06-chain-authority.md`).

**REQ-65xqp8**: The software shall refuse an imported Invite reply whose invite
identifier is not one it holds as outstanding for the Organisation the reply
names, acting on nothing from it; shall act on a reply only for the
Organisation the Invite was issued for; and shall stop holding an invite
identifier as outstanding once a reply naming it has been acted on.
satisfies: derived
exported: yes

*Amended 2026-10-06 (independent review round 2, finding-1).* An invite
identifier is outstanding for the Organisation it was issued for, not for any
Organisation: before this amendment a reply to an Invite for one Organisation
could name another Organisation this device holds, and the first Invite's
identifier let it through to an admission there.

## Fan-out

**REQ-6skn67**: The software shall, in Networked mode, send a committed
update to every Device the committed record lists other than its own, and a
revocation to every Device the update removed, dialling each by the
endpoint identity equal to its Device key; it shall attempt every recipient
whatever happened to the others, report the outcome per Device, and leave
the committed record in place when any send fails. (implements: RC-wbs9ch)

Ruled 2026-10-06 (the fan-out lands in S4, Networked mode). Before S4 a
committed update reached one Device only (the app's problem report on the
update fan-out, merged with the key-pair change on 2026-10-07; PR-3ue4va,
moved here and resolved 2026-10-13, S4 task T12). Which message each recipient
gets is org-node's (REQ-jxyh9c). In Loopback mode, which has no address book,
the single addressed recipient stays, under REQ-8trkps.

## Peer addresses

**REQ-n82q47**: The software shall parse a peer address given as text with a
total function: it shall accept exactly the hexadecimal encoding of a
serialised endpoint address, refuse any other input with a typed error that
names the field, and panic on no input.
satisfies: derived

The app passes the address as text and no longer names iroh. The parse
moves from the app with its items (§3.5 of the plan); this item adds the
totality claim, verified by a fuzz target over arbitrary text (always fuzz,
`AGENTS.md`).

*Clarified 2026-10-14 (S4 review round 1, finding 6), REQ-n82q47:* "exactly
the hexadecimal encoding" admits two forms the parse has accepted since it
moved from the app, as LLR-2b2ghe states them: blank or whitespace-only text
is no address (`Ok(None)`, the caller's "no address given"), not a refusal;
and the hexadecimal, read after its surrounding whitespace is trimmed, may
carry one leading `0x`. Every other input is refused as the item says.

**REQ-8trkps**: The software shall, in Loopback mode, send to a recipient
only at an address whose endpoint identity equals that recipient's Device
key, and shall refuse with a typed error, sending nothing, when it holds no
address for the recipient or the address names a different identity.
(implements: RC-d6f25u)

Fixes org-node's open problem report on loopback sends to an unchecked
address (opened 2026-09-09), which moves here (PR-2dmjzj, moved and resolved
2026-10-13, S4 task T12). Since the key-pair change, the
message to a listed Device carries the Organisation private key, so a
misdirected send hands that key to a device the record does not name.

## What org-io relies on org-node for

Ruling B makes org-node the provider of every value org-io moves. Each
expectation below is met when org-node exports a requirement that names it
in `satisfies:`; org-node's draft
`org-node/docs/requirements/2026-10-06-sans-io.md` does
so for all six.

**REQ-x6kdse**: The software shall rely on org-node to open a Persona store
from sealed bytes and that Persona's transient device secret (starting a new
store only when it creates that Persona, one store per Persona), and to return, from every operation that changes the store, the
complete sealed store as bytes, so that org-io writes exactly what org-node
sealed and needs no access to the store's contents.
expects: org-node
opened: 2026-10-06
satisfies: derived

**REQ-tzr3dk**: The software shall rely on org-node to offer exactly one
receive path, driven by values: a chain-free check of a received message
body that names the Organisation to read, then a commit of the checked
message against the state org-io read, returning the outcome and the sealed
store; covering a first admission, an update and the node's own revocation.
expects: org-node
opened: 2026-10-06
satisfies: derived

*Amended 2026-10-07 before first merge (reconciled with S3a, `60c7d58`):*
the check takes the connection's authenticated DevicePublicKey with the body
(S3a's member-sender rule), settles a revoked Device's acknowledgement
itself with no chain state, and the path also covers a revocation notice
accepted against the state org-io read, signing with the transient device
secret org-io supplies from the keychain (org-node's acknowledgement-signing requirement).

**REQ-jxyh9c**: The software shall rely on org-node to return, for a
committed update, every recipient Device and the encoded message for each,
writing nothing, so that org-io decides nothing about who receives what.
expects: org-node
opened: 2026-10-06
satisfies: derived

**REQ-txkm94**: The software shall rely on org-node to build and parse the
Invite and Invite reply Blobs, to record and settle outstanding invite
identifiers and declared expectations inside the store, and to return the
sealed store from each of those operations.
expects: org-node
opened: 2026-10-06
satisfies: derived

**REQ-v6r2gd**: The software shall rely on org-node to accept a Persona's
device secret only as an opaque value that cannot be cloned, renders no
bytes in `Debug`, has no `Display`, cannot be serialised and offers no
operation returning its bytes except one scoped write for the keychain call;
to consume it within a single operation (deriving the store key, or
signing) and zeroise it when that operation ends; and
to persist neither the secret nor anything derived from it.
(implements: RC-jgyd6m)
expects: org-node
opened: 2026-10-06
satisfies: derived

**REQ-qza96g**: The software shall rely on org-node to seal and unseal a
Persona's store only under a key derived from that Persona's device secret by
a domain-separated derivation that is never the signing key itself, to keep
in the store the Device public key and never the device private seed, and to
offer no passphrase-sealed store. (implements: RC-d8dbxs)
expects: org-node
opened: 2026-10-06
satisfies: derived
