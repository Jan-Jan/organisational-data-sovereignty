# Design — org-io takes the transport, the store files, the keychain and the invitation exchange (stage S4)

Software items and low-level requirements for change
`worktree-org-io-transport`, stage S4 of the org-io roadmap. They refine
org-io's S4
requirements (`crates/org-io/docs/requirements/2026-10-06-transport.md`)
and realise its S4 controls
(`crates/org-io/docs/risk/2026-10-06-transport.md`). Drafted 2026-10-07,
before any code, on top of S2's crate (`OrgIo`, the chain read as values, the
submit path), which merges first. Every item is class C, the unit's class.

The items that move here from org-node and the app keep their IDs (ruling A)
and arrive in develop with the code (plan §3.1, §3.5): the authenticated
endpoint and its lifecycle, the length-prefix framing and the stream read
cap, the loopback binding scope, the invitation operations and the
peer-address parse. Until they arrive this file names none of their IDs;
each item below says which moved item it will sit beside. *(2026-10-13, task
T10: the endpoint, its lifecycle and the framing items have arrived, in
"Moved from org-node with the transport" below.)*

## The shape of the change, for an implementer

```
crates/org-io/src/
  keychain.rs      # keyring Entry per Persona; read → DeviceSecret; scoped write; typed refusals
  store_file.rs    # persona-<hex>.store per Persona; atomic write; startup listing
  transport/
    mod.rs         # TransportMode, ALPN, TransportError (moved), MAX_FRAME re-used from org-node
    endpoint.rs    # OrgEndpoint (moved); bind_from_keychain — the one fn naming iroh's SecretKey
    frame.rs       # len(u32 LE) ‖ body; read cap MAX_FRAME + 4; prefix discarded (moved)
    peer_addr.rs   # parse_peer_addr (moved from the app), PeerAddr newtype (here since 2026-10-15)
  endpoints.rs     # one endpoint per Persona, keyed by Device public key
  receiver.rs      # the lock-free receive step; typed terminal condition; verdict byte (2026-10-07)
  preflight.rs     # S2's chain preflight; gains check_transport by the keychain path (2026-10-07)
  send.rs          # fan-out (Networked), single checked recipient (Loopback)
  exchange.rs      # issue_invite, produce_reply, import_reply, admit_reply, revoke
```

*Amended 2026-10-15 (S4, ARCHITECTURE conformance review finding-1):*
`peer_addr.rs` moved from `crates/org-io/src/` into `transport/`, so iroh is named
in the transport module alone (LLR-4k43dm); `lib.rs` re-exports it as
`org_io::peer_addr`, so its public path is unchanged.

S2's `OrgIo` holds `service: OrgService` directly and its methods take
`&mut self`, so the app locks the whole handle. S4 (plan task T11) changes it
to hold the node state as `tokio::sync::Mutex<OrgService>` with every other
field outside that lock, and its methods take `&self`, so the app holds an
`Arc<OrgIo>` and no lock of its own. S2's transitional `node()`/`node_mut()`
accessors become `with_node(|service| ..)` for queries.

*Reconciled 2026-10-08 with S2 as merged (`e41cb09`).* S2 landed no
`node()`: `OrgIo::view()` returns a `NodeView` (an allowlist of org-node's
queries returning public data only: `PersonaSummary`, `OrgSummary`, public
keys) and `node_mut()` returns `NodeBuilders` (`create_persona`,
`admit_member`/`revoke_member` returning a `BuiltUpdate`, `expect_admission`,
and a test-only `ensure_endpoint`), neither dereferencing to `OrgService`;
`the_handle_returns_only_allowlisted_public_types` (`crates/org-io/tests/absences.rs`)
pins every production public method of `OrgIo`, `NodeBuilders`, `NodeView`
and `ConnectFailed` to reviewed return types. A `with_node(|service| ..)`
closure would hand org-node's whole service to the caller and is therefore
**not** used: T11 keeps S2's surface as `async` methods on `&self` that take
the lock inside (the view's queries returning owned allowlisted values, the
builders returning `PersonaId`/`BuiltUpdate`), and every S4 method returns
only allowlisted types (extended with S4's public-data types, each reviewed
in the task that adds it). S2's `OrgIo::open(data_dir, passphrase,
transport_mode)` already creates the data directory and opens the store
(LLR-rgdx22's clause of 2026-10-08); S4 changes it in place (LLR-uzax55).
`submit_commit_send`, the receive pair and `holds_provisional`'s pre-write
guard are S2's and are reshaped by T4, T5, T11 and T12.

*Reviewed 2026-10-07 against the plan, the S4 requirements and risk drafts
and S2's planned interface.* Owner rulings of 2026-10-07 applied: an
operation needs the keychain only to create a Persona, open or reload a
store, bind an endpoint or sign (REQ-6d9xv6, narrowed); and the in-memory
credential store tests use is keyring's mock only if task T7 finds that its
items are shared between `Entry` handles, otherwise org-io's own stand-in
(LLR-36jer8). The endpoint build reads the keychain through a scoped lend in
`keychain.rs` (LLR-677fqq), so the keychain crate stays in one module and
iroh's secret-key type in one function (LLR-d5y7ks).

## One endpoint per Persona

**SDD-xwft92**: one endpoint per Persona. Code: `crates/org-io/src/endpoints.rs`, `crates/org-io/src/transport/endpoint.rs`. Moved beside
it: the authenticated endpoint and the endpoint lifecycle (org-node's
decomposition), with their low-level items.
traces: REQ-bupq5y, REQ-xuzw32, REQ-hw82ue, REQ-mnd9rj

*Amended 2026-10-15 before first merge (S4 review round 2, finding 1):* traces REQ-mnd9rj, the per-connection time limit (LLR-8uf9a5).

*Note 2026-10-14 (S4, task T20):* an endpoint is kept only while
it is open. One that has closed is bound again on its Persona's next send or
`start_receiving` (LLR-d2pzzk and LLR-z35vbh as amended that day).

**LLR-d5y7ks**: `bind_from_keychain(device_key: &DevicePublicKey, plan:
BindPlan) -> Result<OrgEndpoint, TransportError>` is the only function in
`crates/org-io/src` that names `iroh::SecretKey`: it calls
`keychain::with_endpoint_seed(device_key, |seed| iroh::SecretKey::from_bytes(seed))`
(LLR-677fqq), so the seed is lent to the key's constructor once and its
buffer is dropped and zeroised before `with_endpoint_seed` returns, moves the
key into the endpoint builder for the plan (`Loopback`, `Networked`, or under
`test-support` only `Relay(RelayMap, MemoryLookup)`), and maps a keychain
refusal to `TransportError::Keychain(KeychainError)`; the bound endpoint's
id equals `device_key`, and a mismatch is refused
(`TransportError::KeychainKeyMismatch`) with the endpoint closed. Nothing in
`crates/org-io/src` calls `SecretKey::to_bytes`, `SecretKey::secret`, `clone` on
it, `Endpoint::secret_key`, or formats it, and no type reachable from the
public API holds an `iroh::Endpoint` or an `OrgEndpoint`.
satisfies: REQ-xuzw32, REQ-hw82ue

**LLR-d2pzzk**: `Endpoints` binds at most one open endpoint per Device public
key, lazily, and returns the same endpoint for that key on every later call
while it is open; an endpoint that has closed is dropped, and the next call
for its key binds again from the keychain item; `OrgIo` sends a committed
update from the endpoint of the Device key of `CommitOutcome::sender` (the
Persona bound to the update's Organisation, its key as `persona_public_keys`
returns it), and never from another Persona's endpoint.
satisfies: REQ-bupq5y

*Amended 2026-10-15 before first merge (S4 review round 2, A7),
LLR-d2pzzk:* an endpoint that has started closing (iroh's
`Endpoint::closed()` future has resolved) counts as closed
(`OrgEndpoint::is_closed`), not only one fully closed, so a closing
endpoint is not handed out again.

*Note 2026-10-15 (change `worktree-org-io-domain-api`), LLR-d2pzzk and
LLR-z35vbh:* `CommitOutcome` and `create_persona` are gone. The sending
Persona is now the one whose store the message came from (LLR-hx5c2y), and
the Persona created by `propose_founding_organisation` or `propose_member`
is bound and accepted on while the node receives (LLR-jvx6j9). The rest of
both items is unchanged.

**LLR-z35vbh**: `OrgIo::start_receiving` binds an endpoint for every Persona
the node holds and accepts on every one of them: each bound endpoint has one
running accept task that forwards each received body to the receive step's
channel, so a message to any Persona's Device reaches the one receive path.
Called again, it binds again every endpoint that has closed and gives a new
accept task to every endpoint whose task has ended. Once it has been called,
every endpoint bound later, by a send included, gets its accept task too.
satisfies: REQ-bupq5y

*Amended 2026-10-14 before first merge (S4 review round 1, finding 2), LLR-z35vbh:* a Persona created while the node receives
had no endpoint and no accept task: `create_persona` bound nothing, the app's
`start_receiver` returns early while its loop runs, and T21's flag reached
only an endpoint a send binds. Once `start_receiving` has been called,
`OrgIo::create_persona` now binds the new Persona's endpoint and gives it an
accept task (`Endpoints::accept_if_receiving`), under the map's lock and
counted as before (LLR-szhe7j); a bind that fails there does not undo the
creation. `OrgIo::produce_reply` makes the same bind first, so a Persona
whose endpoint cannot be bound hands out no reply: the refusal is
`ExchangeError::Bind`, with nothing built or written. Tests:
`a_persona_created_while_the_receiver_runs_is_received_on` and
`no_reply_is_produced_while_receiving_for_a_persona_whose_endpoint_cannot_be_bound`
(`crates/org-io/tests/endpoints.rs`).

*Amended 2026-10-14 before first merge (S4, task T21),
LLR-z35vbh:* only `start_receiving` started accept tasks, so an endpoint a
send bound again while another Persona's endpoint kept the receive loop
running had none, and its Persona received nothing until the next
`start_receiving`. `Endpoints` now remembers, from the first
`start_accepting`, that the node receives, and `Endpoints::endpoint` (the
send path's bind) then gives the endpoint it returns a running accept task
unless it has one. Both decisions are made under the map's lock, and a new
task is counted into the running tasks before it starts, so the count stays
exact (LLR-szhe7j). The flag is never cleared: after the terminal
`EndpointClosed` a send may start an accept task whose bodies wait in the
receive channel for the next `next_received`, as one started by a
concurrent `start_receiving` could before. Tests:
`a_send_that_rebinds_a_closed_endpoint_while_the_receiver_runs_accepts_on_it`
and `a_refused_rebind_while_the_receiver_runs_binds_and_accepts_on_the_next_send`
(`crates/org-io/tests/endpoints.rs`).

*Amended 2026-10-14 before first merge (S4, task T20),
LLR-d2pzzk and LLR-z35vbh:* both items said the endpoint, once bound, is the
same one for the life of the handle, with one accept task, once. A closed
endpoint was therefore kept: every later send from its Persona failed with
"Endpoint is closed", and a restarted receive loop found no accept task
running and ended at once. Both now state the rebind. The closure is read
from the endpoint (`OrgEndpoint::is_closed`), and the drop and the bind
happen under the map's lock, so no Device key has two open endpoints. A
closed endpoint's accept task is not aborted: it sees the closure, counts
itself out of the running tasks (LLR-szhe7j) and ends on its own, and the
closure it forwards is skipped by the receive step as before. Tests:
`after_its_endpoint_closes_start_receiving_binds_it_again_and_receives`,
`after_its_endpoint_closes_a_send_binds_it_again` and
`a_refused_rebind_is_a_failed_bind_and_the_next_start_binds`
(`crates/org-io/tests/endpoints.rs`).

*Amended 2026-10-07 before first merge (reconciled with S3a, `60c7d58`):*
each accept task forwards the body together with the DevicePublicKey the
handshake authenticated for its connection (S3a's member-sender rule needs
it; now REQ-xd5a2d).

*Amended 2026-10-15 before first merge (S4 review round 2, A6),
LLR-z35vbh:* `start_receiving` marks the node receiving
(`Endpoints::mark_receiving`, under the map's lock) before it lists the
Personas, so a Persona created between the listing and the first
`start_accepting` sees the flag in `create_persona` and is accepted on;
before, the flag was set after the listing and such a Persona was missed.
No test drives the interleaving deterministically (it would need a hook
between the listing and the bind); the order is stated in a comment at the
call.

**LLR-8uf9a5**: each endpoint's accept task takes each inbound connection
(`OrgEndpoint::next_connection`) and reads it in a task of its own
(`PendingConnection::receive`), so a connection that stalls cannot stop
another being taken. One connection's handshake, the open of its
bidirectional stream and the read of its body together are bounded by
`CONNECTION_TIME_LIMIT` (30 s, `crates/org-io/src/transport/endpoint.rs`: several
times what a handshake and a body at the 1 MiB bound take over a slow
relayed path, and short enough that a stalled peer holds its slot for half
a minute at most); past it the connection is dropped, which closes it, and
`TransportError::TimedOut(limit)` goes to the receive channel as that one
connection's failure (`ReceiveError::Connection`, not terminal,
LLR-szhe7j). An endpoint reads at most `CONNECTIONS_AT_ONCE` (16,
`crates/org-io/src/endpoints.rs`) connections at once: while every slot is taken
it accepts no further connection, so the bodies in flight stay bounded
(RC-5wz559's per-stream bound times sixteen). When the endpoint closes, its
accept task waits for its connections' tasks before it counts itself out
of the running tasks and forwards the closure, so the count stays exact
(LLR-szhe7j); the verdict path is unchanged (LLR-hhgdk7).
satisfies: REQ-mnd9rj

*Added 2026-10-15 before first merge (S4 review round 2, finding 1):* each endpoint had one accept task reading one
connection at a time with no limit, so a peer that completed the handshake
and then opened no stream stopped the endpoint receiving (HAZ-4hj5m9,
RC-96w7nf). Tests (`crates/org-io/tests/endpoints.rs`):
`a_stalled_connection_does_not_stop_another_peers_message`,
`a_stalled_connection_is_dropped_after_the_limit` (a one-second limit, set
through the test-only `OrgIo::with_connection_time_limit_for_test`) and
`a_seventeenth_connection_waits_for_a_slot_while_sixteen_stall`.

**LLR-h9tcat**: org-io's preflight checks the transport with
`check_transport(device_key, mode, online_timeout) -> CheckResult`, which
binds through `bind_from_keychain` (LLR-d5y7ks) for `device_key` and
reports the bound socket (Loopback) or the relay home (Networked) as
org-node's preflight did; a keychain refusal or a missing item is one FAIL
row naming the Device key, with no endpoint bound. The preflight binary
finds the Device keys the same way the node does: it lists the store files
of the node's data directory (LLR-nt8dja) and runs `check_transport` for
each listed Device key, through that key's keychain item; a data directory
with no store file is one FAIL row naming the directory, and a directory
that cannot be read is one FAIL row naming it. No environment variable
names or carries a device key or seed: no file in `crates/org-io/src` reads
`ODS_DEVICE_SEED` or `ODS_DEVICE_KEY`, names `DeviceSeed`, or builds a
device secret from anything but the keychain item.
satisfies: REQ-xuzw32, REQ-hw82ue

*Added 2026-10-07 (owner ruling of that day):* org-node keeps a
transport-only preflight, reading `ODS_DEVICE_SEED`, until S4; S2's
no-`DeviceSeed`-in-org-io item bars carrying that across, so the check
arrives by the keychain path instead.

*Rewritten 2026-10-08 before first merge (owner ruling of that day on plan
§16.4 point 4):* the draft had the binary read a Device **public** key from
`ODS_DEVICE_KEY`. The owner ruled that the preflight's key is not an
environment variable: environment-variable keys and seeds are for test
environments only (and `ODS_ADMIN_SEED` is development-only, for on-chain
signing), and the transport preflight gets the device key through the same
keychain path as the node. The binary therefore takes the Device keys from
the store-file listing, as `OrgIo::open` does. Which data directory it
lists (the app's `ODS_DATA_DIR`, a path and not a key, or a command-line
argument) is settled in task T10 and recorded there; neither carries key
material.

*Clarified 2026-10-13 (S4, task T10; owner ruling of 2026-10-08, plan
§17.4):* the binary lists `ODS_DATA_DIR`, with no default: unset, it
refuses and exits non-zero before any check. `ODS_TRANSPORT` (the mode, not a
key) is read as org-node's binary read it. The listing and the per-key rows
are `preflight::check_transport_for_data_dir(data_dir, mode,
online_timeout)`; every row of `check_transport` names its Device key as 64
lower-case hex characters.

*Clarified 2026-10-13 (S4, task T10), LLR-d5y7ks:* the keychain refusal is
carried boxed, `TransportError::Keychain(Box<KeychainError>)`, and so is the
mismatch's key, `TransportError::KeychainKeyMismatch(Box<DevicePublicKey>)`
(a Device public key is a parsed point). The mismatch is detected from the
secret key's public half before the builder runs, so it is refused with no
endpoint bound at all, which is the strongest reading of "with the endpoint
closed".

*Clarified 2026-10-13 (S4, task T10), LLR-d2pzzk and LLR-z35vbh:*
`Endpoints` lives in the handle's `LocalStore` (so `connect` carries it), maps
each Device public key to one endpoint behind a `tokio::sync::Mutex`, and is
built in `OrgIo::open`'s transport mode. A Loopback send without an address is
refused before any endpoint is bound (`TransportError::NoLoopbackAddress`).
The receive channel is bounded (64 bodies): a full channel stops the accept
tasks accepting rather than queueing without bound. Each item is a body with
its sender or the reason one connection gave none; an accept task ends only
when its endpoint closes (`TransportError::EndpointClosed`). Until task T11,
S2's `receive_and_verify` reads this channel and refuses at once when no
endpoint is accepting; `start_receiving` called again accepts on the
endpoints of Personas created since.

*Clarified 2026-10-13 (S4, task T12):* the send left `Endpoints` for
`send::send_committed` (then the fan-out item), and with it
`TransportError::NoLoopbackAddress`: a Loopback send without an address is
refused per recipient as `SendError::NoLoopbackAddress`, still before any
endpoint is bound (the Loopback address check, now LLR-34tfm8).

*Note 2026-10-13 (S4, task T11):* `next_received` (`crates/org-io/src/receiver.rs`)
replaces `receive_and_verify`. The channel carries `Inbound { sender, body
}` (task T11a added the verdict responder, `verdict`; see LLR-hhgdk7) or a connection's failure; the
running accept tasks are counted, and when none is left and the channel is
drained `next_inbound` returns `TransportError::EndpointClosed`, which the
receive step reports as `TerminalCause::EndpointClosed` (LLR-szhe7j).
`start_receiving` moved to `receiver.rs` and returns `ReceiveError`. S2's
`view()`/`NodeView` and `node_mut()`/`NodeBuilders` are gone: their
methods (`personas`, `organisations`, `organisation`,
`persona_public_keys`, `admit_member`, `revoke_member`,
`expect_admission`), with `create_persona`, are `async` methods of `OrgIo`
on `&self` that take the lock inside; the test-only `node_for_test` returns
the lock's guard and `node_mut_for_test` is gone.

*Clarified 2026-10-15 (S4 TEST review round 3, notes):* the binary's
refusal of `ODS_DATA_DIR` (the clarification of 2026-10-13 above) is one of
two, each naming the variable on stderr, and each exiting non-zero before
any check: unset, it reports `ODS_DATA_DIR not set`; set to a value that is
not valid Unicode, it reports `ODS_DATA_DIR is not valid Unicode`, rather
than that it is unset. Tests:
`the_binary_without_a_data_dir_exits_non_zero_before_any_check` and
`the_binary_with_a_non_unicode_data_dir_says_so_and_exits_before_any_check`
(preflight_transport).

## Moved from org-node with the transport (S4, task T10)

*Moved 2026-10-13 from org-node (`architecture/2026-10-03-decomposition.md`),
plan task T10 (ruling A):* the authenticated endpoint, the endpoint
lifecycle and the framing items, with their low-level items. Their
`satisfies:` and `traces:` lines named org-node requirements (device-key
custody evidence, the unchecked sender, the Wire message's bound), which
org-io cannot cite; each is re-parented here to the org-io requirement that
states the same property (dated notes below), and every other org-node item
their notes named is named in prose. Code: `crates/org-io/src/transport/`
(`endpoint.rs`, `frame.rs`, `mod.rs`) and `crates/org-io/src/endpoints.rs`; tests:
`crates/org-io/tests/transport_handshake.rs`, `transport_networked.rs` and
`endpoints.rs`.

### SDD-8uyg4s — The authenticated endpoint

`crates/org-io/src/transport/endpoint.rs`

**SDD-8uyg4s**: the iroh endpoint whose identity *is* the device's key, so a
completed QUIC handshake is cryptographic evidence of which device is on the
other end — together with the two bind modes and what each one is allowed to
reach.
traces: REQ-bupq5y, REQ-xuzw32, REQ-db6s7q, REQ-2wzfzv, REQ-szt6ea

*Moved 2026-10-13 (S4, task T10):* the traces were org-node's custody
requirement, its sender requirement and its Wire-message bound, beside the
two binding-scope requirements; they are REQ-bupq5y (the endpoint identity is
the Persona's Device key), REQ-xuzw32 (bound from the keychain item) and
REQ-szt6ea (the stream read in front of the bound) here.

**LLR-ygn78w**: the endpoint's iroh `EndpointId` is byte-identical to the
device key of the keychain item it was bound from.
satisfies: REQ-bupq5y

*Moved 2026-10-13 (S4, task T10):* was "of the keypair it was bound with",
satisfying org-node's custody and sender requirements; org-io binds only from
the keychain item (LLR-d5y7ks), and the item's key is the Device key.

**LLR-v873fx**: `recv_one` (and `recv_body`) returns the remote device key
taken from the key authenticated by the QUIC handshake, **never from the Wire
message body**, so the sender identity a caller cross-checks cannot be chosen
by the sender.
satisfies: REQ-bupq5y, REQ-cj9u3y

*Moved 2026-10-13 (S4, task T10):* satisfied org-node's custody and sender
requirements. Since S3a's member-sender rule the key is compared with the
record again, in org-node's `check_received`; org-io hands it over with the
body (REQ-xd5a2d, LLR-z35vbh). The notes of 2026-10-05 that said no caller
compares the key are history.

**LLR-wwunf4**: in Loopback mode the endpoint binds only loopback addresses and
offers a peer only loopback addresses for dialling.
satisfies: REQ-db6s7q

**LLR-6adc99**: in Loopback mode an IPv4 loopback socket is required and an
IPv6 loopback socket is optional, so the endpoint is provided when only the
latter cannot be bound.
satisfies: REQ-2wzfzv

**LLR-wx77j5**: `node_addr_for_dial` reports the endpoint's identity together
with its currently bound sockets, rather than with addresses from asynchronous
discovery that may not have run.
satisfies: REQ-db6s7q

### The framing and the stream read cap (beside SDD-2fc2cq)

`crates/org-io/src/transport/frame.rs`

**LLR-fa7jt8**: `encode_frame` emits the body's length as four little-endian
bytes followed by the encoded body, and nothing else.
satisfies: REQ-szt6ea

*Moved 2026-10-13 (S4, task T10):* satisfied org-node's Wire-message bound;
re-parented to REQ-szt6ea (plan task T10). `encode_frame` frames org-node's
`encode_message` body; `frame_body` frames an already encoded body and
refuses one over `MAX_FRAME` (org-node's `MAX_MESSAGE`).

**LLR-k2y6nn**: `recv_one` reads at most `MAX_FRAME + 4` bytes from a stream,
so a peer cannot cause unbounded allocation by never finishing.
satisfies: REQ-szt6ea

*Moved 2026-10-13 (S4, task T10):* satisfied org-node's Wire-message bound;
re-parented to REQ-szt6ea. The read is `recv_body`'s, which `recv_one`
decodes; a longer stream is `TransportError::Stream`.

**LLR-2smrvx**: `recv_one` discards the four-byte length prefix without
comparing it with the body, and a stream shorter than four bytes is decoded
whole. The frame ends where the QUIC stream ends, its size is bounded by
LLR-k2y6nn's read limit and by org-node's decode, and the prefix is not
consulted.
satisfies: REQ-szt6ea

*Added 2026-10-05 by review round 8, which measured that checking the prefix against the body left the gate green.
`the_length_prefix_is_not_checked_against_the_body` pins it, and since review
round 9 also sends three bytes and sees them refused as Malformed by the
decoder rather than by the read.*

*Moved 2026-10-13 (S4, task T10):* was `satisfies: derived`, assessed in
org-node's architecture-derived risk file; REQ-szt6ea states it ("the length
field shall be discarded, not trusted"), so it is re-parented there and its
assessment moved with it (org-io's S4 risk draft).

### SDD-b8tuv3 — Endpoint lifecycle

`crates/org-io/src/endpoints.rs` (`Endpoints`), `crates/org-io/src/lib.rs`
(`OrgIo::open`'s transport mode, `start_receiving`)

**SDD-b8tuv3**: when the transport comes into existence and whose key it
carries — bound lazily, at most once per Persona, from the keychain item of
that Persona's Device.
traces: REQ-db6s7q, REQ-bupq5y, REQ-xuzw32

*Moved 2026-10-13 (S4, task T10):* was "at most once, from the device seed of
a named persona", in org-node's service, tracing org-node's custody
requirement; the lifecycle is org-io's `Endpoints` (SDD-xwft92), one endpoint
per Persona.

**LLR-6zjzn2**: `Endpoints` binds at most one endpoint per Device public key
and returns that same endpoint on every later call for it.
satisfies: REQ-bupq5y

*Amended 2026-10-13 (S4, task T10; plan §3.1, "rewritten to one per
Persona"):* the item stated org-node's `ensure_endpoint` binding at most one
endpoint **per service**, which on a node holding more than one Persona was
the defect of the endpoint bound once per service (a resolved problem
report), stated and not endorsed. It now states the rule
of LLR-d2pzzk, which it sits beside; its test
(`the_endpoint_is_bound_once_and_the_same_one_is_returned_after`, moved to
`crates/org-io/tests/endpoints.rs`) observes the same endpoint by its sockets.

*Clarified 2026-10-14 (S4, task T20):* "that same endpoint on
every later call" holds while the endpoint is open; a closed one is bound
again, as LLR-d2pzzk states since that day.

**LLR-cns6q6**: the endpoint of a Persona is bound from the keychain item of
that Persona's Device key, so the identity the transport authenticates for a
send is the sending Persona's Device key, whichever Persona was bound first.
satisfies: REQ-xuzw32, REQ-bupq5y

*Amended 2026-10-13 (S4, task T10; plan §3.1, "rewritten: from the keychain
item"):* the item stated that the endpoint is bound from the device seed of
the Persona named by the first call, true of that Persona only (narrowed
2026-10-04 to that defect; on 2026-10-08 bound from a `DeviceSecret` the caller
passed). It now states LLR-d5y7ks's keychain bind per Persona; its test
(`the_endpoint_binds_the_named_personas_device_not_the_first_personas`,
moved) binds the second Persona's endpoint and sees its key.

**LLR-ecz9a6**: in Loopback mode the endpoint binds loopback addresses and
nothing else.
satisfies: REQ-db6s7q

*Narrowed 2026-10-04 by review round 2 (in org-node):* the clause used to
read "the endpoint is bound in the transport mode the service holds at the
time of binding"; replacing the mode with a hard-coded Loopback left the gate
green, because no gated test may bind Networked (it would reach the public
internet). That the bind reads the configured mode rather than a constant is
unevidenced at this gate. *Moved 2026-10-13 (S4, task T10):* the mode is
`OrgIo::open`'s, held by `Endpoints`; `open_sets_the_transport_mode_it_was_given`
(LLR-rgdx22) reads it back, and the test-only `BindPlan::Relay` exercises a
Networked-style bind against an in-process relay.

## The receive step

**LLR-szhe7j**: `ReceiveError::Terminal(TerminalCause)`, with
`TerminalCause::EndpointClosed` and `TerminalCause::BindFailed`, is returned
when every accept task has ended or an endpoint could not be bound, and for
nothing else; every other failure is a different `ReceiveError` variant, and
no caller needs the text of a message to tell them apart.
satisfies: REQ-8e57wm

*Note 2026-10-13 (S4, task T11):* as landed, `TerminalCause::BindFailed`
carries the `TransportError` and is returned by `start_receiving`, which
binds; `EndpointClosed` by `next_received`, once every accept task has
ended (each counts itself out before it forwards its endpoint's closure)
and the bodies they forwarded are taken, or at once when none was
started. The other variants are `Refused`, `Chain`, `Store`, `Keychain`
(the receive step's keychain refusal, now LLR-kz9hj4) and `Connection(TransportError)`, one inbound
connection that failed before it carried a body, after which the loop goes
on. Tests (`crates/org-io/tests/receive_step.rs`):
`a_closed_endpoint_is_terminal_endpoint_closed`,
`a_failed_bind_is_terminal_bind_failed`, `a_malformed_body_is_not_terminal`.

*Note 2026-10-15 (S4 review round 2, finding 7):* `Connection` is driven
through `next_received` by `a_connection_failing_before_a_body_is_not_terminal`
(`crates/org-io/tests/receive_step.rs`): a peer that closes before opening a
stream, after which the next body is received. A connection dropped at
LLR-8uf9a5's time limit is `Connection(TransportError::TimedOut)`. The
receive-order tests (`crates/org-io/tests/receive_order.rs`) now assert `Refused`
and `Chain` apart.

## The store files

**SDD-egx8rv**: the store files. Code: `crates/org-io/src/store_file.rs` and the store-opening part of `OrgIo`'s
constructor. Moved beside it: the app's "create the data dir and open the
store in it" item, rewritten per Persona.
traces: REQ-g9cu2c, REQ-zmtmd9, REQ-pnr6kh, REQ-fh5gsq

**LLR-nt8dja**: each Persona's sealed store is the file
`<data_dir>/persona-<64 lower-case hex of its Device public key>.store`; the
listing at startup parses each file name of that form into a
`DevicePublicKey` (the disk is an edge), ignores every other name, and holds
no other store file.
satisfies: REQ-g9cu2c

**LLR-b9bm36**: `write_store(data_dir, write: StoreWrite)` creates
`persona-<hex>.store.tmp-<16 random hex>` in the data directory with
`create_new`, with the platform's default permissions (no mode or ACL is
set: owner ruling, 2026-10-07), writes exactly `write.sealed`'s bytes, calls
`sync_all`, renames it over the Persona's store file, and on Unix syncs the
directory; a failure at any step removes the temporary file where it can and
returns a typed `StoreFileError` naming the step.
satisfies: REQ-zmtmd9, REQ-g9cu2c

*Amended 2026-10-07 before first merge (owner ruling of that day):* the
owner-only mode and Windows ACL clauses are dropped with the requirement
they satisfied; the atomic replace, which the receive step (LLR-kz9hj4) relies on, stays.

*Note 2026-10-13 (S4, task T9):* as landed, `write_store(data_dir, write:
&StoreWrite)` and `StoreFileError { step, path, reason }`, `step` one of
`StoreFileStep` (`CreateTemporary`, `Write`, `Sync`, `Rename`,
`SyncDirectory`, and for the listing, read and removal `List`, `Read`,
`Remove`); `reason` is the operating system's message. The temporary's
suffix comes from `OsRng`. Under `test-support` only,
`test_support::fail_next_write_after_temporary` makes the next write on the
thread fail at the `Rename` step, after the temporary is written and synced.

**LLR-murjr6**: when `write_store` fails after an `OrgService` operation
changed a store, the `OrgIo` method reports the failure and not success, and
before releasing the lock replaces that Persona's in-memory store with the
one unsealed again from its file (reading the keychain item once more), so
the node's state never runs ahead of the file; when that reload also fails
the Persona's store is dropped from the service and the method's error says
so.
satisfies: REQ-g9cu2c

*Note 2026-10-13 (S4, task T9):* as landed, the failure is
`StoreFailure::Reloaded { failed: StoreFileError }` or
`StoreFailure::Dropped { failed, reload: Box<OpenError> }`; the handle's
builders and `receive_and_verify` return it as `OperationError::Store`
(org-node's refusals as `OperationError::Node`), `create_persona` as
`CreateError::Store`, and the submit paths in their message ("the chain
write executed and the commit was made, but storing it failed; reloaded
from the file …"), never as "nothing committed" and never as
`OrgNodeError::Chain`. The reload puts the store back with org-node's
`OrgService::restore_store`, and the drop takes it out with
`OrgService::drop_store`, org-node's exempt removal place for this case
(owner ruling of 2026-10-13, option A). The drop deletes nothing on disk.
"Before releasing the lock" applies from task T11, when the handle has one.

**LLR-uzax55**: opening the node creates the data directory if needed
(refusing, naming it, when that fails), lists the store files (LLR-nt8dja),
and for each reads its keychain item and unseals it; it refuses to open,
with a typed error naming the Device public key, when an item is missing
(`OpenError::KeychainItemMissing`), leaving that file untouched and
creating nothing, and when the keychain refuses (LLR-8erfbu).
satisfies: REQ-fh5gsq, REQ-g9cu2c

*Note 2026-10-13 (S4, task T9):* as landed, `OrgIo::open(data_dir,
transport_mode) -> Result<OrgIo, OpenError>`, changed in place from S2's.
`OpenError` holds public data only: `CreateDataDir { path, reason }`,
`ListDataDir` and `ReadStore` (each a `StoreFileError`),
`KeychainItemMissing { device_key, path }`, `Keychain { path, error:
KeychainError }`, `Unseal { device_key, path, error }` and `Service`. A
keychain refusal names the Device key through its `KeychainError`; the
Persona's id is not known before its store is unsealed, so the Device key
is how a refusal names the Persona.

*Note 2026-10-15 (S4, ARCHITECTURE conformance review finding-7):*
`CreateDataDir { path, source }` keeps the operating system's
`std::io::Error` (public data, no key material) where it held its text as
`reason`; its message is unchanged. `OpenError::source()` gives that error,
or the `StoreFileError` of `ListDataDir` and `ReadStore`. `StoreFileError`
keeps `reason` as text: carrying the `std::io::Error` beside it grows
`StoreFailure`, and with it `OperationError` and `ExchangeError`, past the
size the crate's `result_large_err` lint allows, and dropping the public
`reason` field would change the shape LLR-b9bm36's note of 2026-10-13
states. The transport's errors stay text on purpose: keeping iroh's errors
would put iroh types in org-io's public API (LLR-4k43dm, LLR-d5y7ks).

**LLR-85zque**: `OrgIo::open` creates the data directory, with any missing
parents, and opens every Persona's store in it: the file
`persona-<hex Device key>.store` (LLR-nt8dja), unsealed through that
Persona's keychain item (LLR-uzax55). When the directory cannot be created
it refuses with `OpenError::CreateDataDir`, whose message begins
`create data_dir <path>`. When a store does not open, it refuses naming the
store file, and opens no other store in its place.
satisfies: derived

Normal: `the_store_is_opened_inside_the_data_dir_it_creates`,
`opening_creates_the_data_dir_and_unseals_every_store` (store_files).
Abnormal: `a_data_dir_that_cannot_be_created_is_refused_naming_it`,
`a_store_whose_item_is_missing_is_refused_and_left_untouched`,
`a_locked_keychain_refuses_opening_naming_the_persona` (store_files).

*Moved 2026-10-13 from app (`apps/desktop/docs/architecture/2026-10-05-decomposition.md`),
plan task T9; rewritten per Persona (ruling A).* In the app it read:
"`assemble` creates the data directory, with any missing parents, and opens
the persona store at `persona_store.bin` inside it, under the given
passphrase. When the directory cannot be created, it refuses with a message
beginning `create data_dir <path>`. When an existing store does not open
under the passphrase, it refuses with a message beginning `open store: `,
and opens no other store in its place." Its S2 notes, carried with it: in
change `worktree-org-io-create` the behaviour moved from the app's
`assemble` to `open_service` (task T9), the store came to be opened once
even after a refused connect (task T9b, `ConnectFailed::into_not_configured`),
and the open itself moved into org-io's `OrgIo::open` (review round 2,
finding 2), stated there as a clause of org-io's development-seed item
(review round 3, finding 3). S4 drops the passphrase (task T3) and the one
store: there is one store file per Persona, keyed through the keychain. The
two app tests that moved here are rewritten per Persona over the in-memory
keychain; the app keeps its scan of `init` and its IPC test (task T17
settles their citations).

**LLR-t4v69b**: no code in `crates/org-io/src` opens, reads, writes or names
`outstanding_invites.json` or `persona_store.bin`; outstanding invites exist
only inside the sealed stores.
satisfies: REQ-pnr6kh

## The keychain

**SDD-4fvypt**: the keychain. Code: `crates/org-io/src/keychain.rs`. SOUP: `keyring` (inventory row written in task T7).
traces: REQ-hw82ue, REQ-rz93uz, REQ-6d9xv6, REQ-fh5gsq

**LLR-677fqq**: a Persona's keychain item is the `keyring::Entry` with
service `"ods-device-seed"` and user the 64 lower-case hex of its Device
public key; `read_device_secret(device_key) -> Result<DeviceSecret,
KeychainError>` calls `get_secret()` and moves the result, wrapped in
`Zeroizing` in the same expression, into `DeviceSecret::from_keychain_bytes`,
then refuses (`KeychainError::KeyMismatch`) a secret whose Device public key
is not `device_key`. `with_endpoint_seed<R>(device_key, build: impl
FnOnce(&[u8; 32]) -> R) -> Result<R, KeychainError>` (`pub(crate)`, called
only by LLR-d5y7ks's `bind_from_keychain`) calls `get_secret()` wrapped in
`Zeroizing` in the same expression, refuses any length but 32
(`KeychainError::Malformed { device_key }`, echoing no byte), calls `build`
exactly once with the 32 bytes, and drops the buffer before it returns
`build`'s result. Neither function binds the bytes to any other name, and no
`Clone`, `Debug`, `to_vec` or `Serialize` reaches them; `keyring` is named in
`keychain.rs` only.
satisfies: REQ-hw82ue

*Clarified 2026-10-14 (S4 review round 1, finding 15), LLR-677fqq:* "named
in `keychain.rs` only" is of production code. `keyring` is also named in
`test_support.rs`, the test-only in-memory store LLR-36jer8 sanctions
(`MemoryKeychain`, a `keyring::credential::CredentialBuilderApi` compiled
only under `test-support`); the scan
`keyring_is_named_only_in_the_keychain_module_and_its_mock_only_under_test_support`
allows exactly these two files.

**LLR-8erfbu**: `keyring::Error::NoEntry` maps to
`KeychainError::ItemMissing { device_key }`; `NoStorageAccess` to
`KeychainError::Locked { device_key }`; every other error to
`KeychainError::Unavailable { device_key, reason }`, where `reason` is the
error's kind name and never a byte of a secret; no path through `crates/org-io/src`
reads a seed, a key or a passphrase from anywhere but the keychain item.
satisfies: REQ-6d9xv6

*Note 2026-10-13 (S4, task T8; T7's finding):* on a target with no native
keychain backend (any `target_os` but macOS, iOS, Windows and Linux; Android
in S4), `read_device_secret`, `write_device_secret` and `with_endpoint_seed`
return `KeychainError::Unavailable { device_key, reason: "NoNativeBackend" }`
without calling `keyring`, whose 3.6.3 release would otherwise use its
non-persistent in-memory store there and lose the seed at exit (HAZ-eyene4).
Every `keyring` call sits in the module's `cfg(any(...))` backend. No such
target is installed here, so the test is a source scan
(`a_target_without_a_native_backend_refuses_as_unavailable`). In a
`test-support` build the keychain refuses every call
(`reason: "NotInstalledForTest"`) until a test installs
`test_support::MemoryKeychain`, so no test reaches the OS keychain.

**LLR-36jer8**: `org-io` builds `keyring` (pinned `=3.6.3`,
`default-features = false`) with only its platform-native backends and the
Secret Service session cipher: features exactly `apple-native` (macOS and
iOS), `windows-native` (Windows), `sync-secret-service` (Linux) and
`crypto-rust`; Android has no backend in S4 and is a later stage. The
in-memory credential store tests use is compiled only under the
`test-support` feature and installed only by test code, and no release build
contains it. That store is org-io's `test_support::MemoryKeychain`, a
`keyring::credential::CredentialBuilderApi` over one shared map, installed with
`keyring::set_default_credential_builder` (owner ruling, 2026-10-07); it can
be told to refuse every call as locked or unavailable, so the refusals of
LLR-8erfbu are testable.
satisfies: REQ-6d9xv6

*Amended 2026-10-13 (S4, task T7):* the probe of owner ruling 3 ran against
`keyring` 3.6.3: under `keyring::mock::default_credential_builder()`, a
secret set through one `Entry::new("ods-device-seed", "probe")` handle reads
back on that handle and is `NoEntry` on a second handle for the same service
and user (each mock credential keeps its secret in the entry), so the
mock does not share items and org-io supplies `MemoryKeychain`. Windows is
kept: `windows-native` adds only `windows-sys` and `byteorder`, one code path,
and the Credential Manager's 2560-byte blob limit is far above the 32-byte
seed. Android is deferred (roadmap "(later)" row): `keyring` 3.6.3 builds
its mock as the default store on any target without a native backend,
Android included, so an Android build must refuse rather than use the default
store.
