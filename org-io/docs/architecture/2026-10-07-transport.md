# Design — org-io takes the transport, the store files, the keychain and the invitation exchange (stage S4)

Software items and low-level requirements for change
`worktree-org-io-transport`, stage S4 of
`docs/plans/2026-10-06-org-io-roadmap.md`. They refine org-io's S4
requirements (`org-io/docs/requirements/2026-10-06-transport.md`)
and realise its S4 controls
(`org-io/docs/risk/2026-10-06-transport.md`). Plan:
`docs/plans/2026-10-06-org-io-transport.md` ("## Tasks"). Drafted 2026-10-07,
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
org-io/src/
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
`peer_addr.rs` moved from `org-io/src/` into `transport/`, so iroh is named
in the transport module alone (LLR-3vb4k9); `lib.rs` re-exports it as
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
`the_handle_returns_only_allowlisted_public_types` (`org-io/tests/absences.rs`)
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

**SDD-xwft92**: one endpoint per Persona. Code: `org-io/src/endpoints.rs`, `org-io/src/transport/endpoint.rs`. Moved beside
it: the authenticated endpoint and the endpoint lifecycle (org-node's
decomposition), with their low-level items.
traces: REQ-bupq5y, REQ-xuzw32, REQ-hw82ue, REQ-mnd9rj

*Amended 2026-10-15 before first merge (S4 review round 2, finding 1;
PR-d2p5y2):* traces REQ-mnd9rj, the per-connection time limit (LLR-8uf9a5).

*Note 2026-10-14 (S4, task T20; PR-tcd2ta):* an endpoint is kept only while
it is open. One that has closed is bound again on its Persona's next send or
`start_receiving` (LLR-d2pzzk and LLR-z35vbh as amended that day).

**LLR-d5y7ks**: `bind_from_keychain(device_key: &DevicePublicKey, plan:
BindPlan) -> Result<OrgEndpoint, TransportError>` is the only function in
`org-io/src` that names `iroh::SecretKey`: it calls
`keychain::with_endpoint_seed(device_key, |seed| iroh::SecretKey::from_bytes(seed))`
(LLR-677fqq), so the seed is lent to the key's constructor once and its
buffer is dropped and zeroised before `with_endpoint_seed` returns, moves the
key into the endpoint builder for the plan (`Loopback`, `Networked`, or under
`test-support` only `Relay(RelayMap, MemoryLookup)`), and maps a keychain
refusal to `TransportError::Keychain(KeychainError)`; the bound endpoint's
id equals `device_key`, and a mismatch is refused
(`TransportError::KeychainKeyMismatch`) with the endpoint closed. Nothing in
`org-io/src` calls `SecretKey::to_bytes`, `SecretKey::secret`, `clone` on
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

**LLR-z35vbh**: `OrgIo::start_receiving` binds an endpoint for every Persona
the node holds and accepts on every one of them: each bound endpoint has one
running accept task that forwards each received body to the receive step's
channel, so a message to any Persona's Device reaches the one receive path.
Called again, it binds again every endpoint that has closed and gives a new
accept task to every endpoint whose task has ended. Once it has been called,
every endpoint bound later, by a send included, gets its accept task too.
satisfies: REQ-bupq5y

*Amended 2026-10-14 before first merge (S4 review round 1, finding 2;
resolves PR-y6t5kt), LLR-z35vbh:* a Persona created while the node receives
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
(`org-io/tests/endpoints.rs`).

*Amended 2026-10-14 before first merge (S4, task T21; resolves PR-47aycj),
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
(`org-io/tests/endpoints.rs`).

*Amended 2026-10-14 before first merge (S4, task T20; resolves PR-tcd2ta),
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
(`org-io/tests/endpoints.rs`).

*Amended 2026-10-07 before first merge (reconciled with S3a, `60c7d58`):*
each accept task forwards the body together with the DevicePublicKey the
handshake authenticated for its connection (S3a's member-sender rule needs
it, REQ-fnsjb6 as amended).

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
`CONNECTION_TIME_LIMIT` (30 s, `org-io/src/transport/endpoint.rs`: several
times what a handshake and a body at the 1 MiB bound take over a slow
relayed path, and short enough that a stalled peer holds its slot for half
a minute at most); past it the connection is dropped, which closes it, and
`TransportError::TimedOut(limit)` goes to the receive channel as that one
connection's failure (`ReceiveError::Connection`, not terminal,
LLR-szhe7j). An endpoint reads at most `CONNECTIONS_AT_ONCE` (16,
`org-io/src/endpoints.rs`) connections at once: while every slot is taken
it accepts no further connection, so the bodies in flight stay bounded
(RC-5wz559's per-stream bound times sixteen). When the endpoint closes, its
accept task waits for its connections' tasks before it counts itself out
of the running tasks and forwards the closure, so the count stays exact
(LLR-szhe7j); the verdict path is unchanged (LLR-unqfj5).
satisfies: REQ-mnd9rj

*Added 2026-10-15 before first merge (S4 review round 2, finding 1;
resolves PR-d2p5y2):* each endpoint had one accept task reading one
connection at a time with no limit, so a peer that completed the handshake
and then opened no stream stopped the endpoint receiving (HAZ-4hj5m9,
RC-96w7nf). Tests (`org-io/tests/endpoints.rs`):
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
names or carries a device key or seed: no file in `org-io/src` reads
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
`send::send_committed` (SDD-v2bq89), and with it
`TransportError::NoLoopbackAddress`: a Loopback send without an address is
refused per recipient as `SendError::NoLoopbackAddress`, still before any
endpoint is bound (LLR-r9sh6k).

*Note 2026-10-13 (S4, task T11):* `next_received` (`org-io/src/receiver.rs`)
replaces `receive_and_verify`. The channel carries `Inbound { sender, body
}` (task T11a added the verdict responder, `verdict`; see LLR-unqfj5's
note) or a connection's failure; the
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
their notes named is named in prose. Code: `org-io/src/transport/`
(`endpoint.rs`, `frame.rs`, `mod.rs`) and `org-io/src/endpoints.rs`; tests:
`org-io/tests/transport_handshake.rs`, `transport_networked.rs` and
`endpoints.rs`.

### SDD-8uyg4s — The authenticated endpoint

`org-io/src/transport/endpoint.rs`

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
satisfies: REQ-bupq5y, REQ-fnsjb6

*Moved 2026-10-13 (S4, task T10):* satisfied org-node's custody and sender
requirements. Since S3a's member-sender rule the key is compared with the
record again, in org-node's `check_received`; org-io hands it over with the
body (REQ-fnsjb6, LLR-z35vbh). The notes of 2026-10-05 that said no caller
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

**LLR-3vb4k9**: iroh is named only in `org-io/src/transport/` (and, under
`test-support`, in `org-io/src/test_support.rs`): no other file of
`org-io/src` names `iroh::` in code, and no function outside the transport
module returns or takes an iroh type. The rest of org-io reaches the
transport through org-io's and org-node's own types: it dials a recipient by
its `DevicePublicKey` (`OrgEndpoint::send_body_to_device`) or at a
transport-owned `PeerAddr` (`OrgEndpoint::send_body_at`; the parse is
SDD-wr8m5n's), and asks an endpoint whether it has a relay home
(`OrgEndpoint::has_relay_home`), not for its iroh address.
satisfies: REQ-xuzw32

*Added 2026-10-15 (S4, ARCHITECTURE conformance review finding-1; owner
ruling of that day):* iroh is one adapter's dependency, so its failure modes
and upgrades are handled in one module. It satisfies REQ-xuzw32 because the
endpoint that holds the Device seed is iroh's, and confining iroh to this
module keeps that endpoint, and every type that reaches it, out of the rest
of org-io and the app.

### The framing and the stream read cap (beside SDD-aw3nka)

`org-io/src/transport/frame.rs`

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

`org-io/src/endpoints.rs` (`Endpoints`), `org-io/src/lib.rs`
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
the defect PR-8qsnhx books, stated and not endorsed. It now states the rule
of LLR-d2pzzk, which it sits beside; its test
(`the_endpoint_is_bound_once_and_the_same_one_is_returned_after`, moved to
`org-io/tests/endpoints.rs`) observes the same endpoint by its sockets.

*Clarified 2026-10-14 (S4, task T20; PR-tcd2ta):* "that same endpoint on
every later call" holds while the endpoint is open; a closed one is bound
again, as LLR-d2pzzk states since that day.

**LLR-cns6q6**: the endpoint of a Persona is bound from the keychain item of
that Persona's Device key, so the identity the transport authenticates for a
send is the sending Persona's Device key, whichever Persona was bound first.
satisfies: REQ-xuzw32, REQ-bupq5y

*Amended 2026-10-13 (S4, task T10; plan §3.1, "rewritten: from the keychain
item"):* the item stated that the endpoint is bound from the device seed of
the Persona named by the first call, true of that Persona only (narrowed
2026-10-04 to PR-8qsnhx; on 2026-10-08 bound from a `DeviceSecret` the caller
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

**SDD-aw3nka**: the receive step. Code: `org-io/src/receiver.rs`, `org-io/src/transport/frame.rs`. Moved beside it:
the framing and the stream read cap (org-node's decomposition), whose
`satisfies:` lines are re-parented to REQ-szt6ea when they move.
traces: REQ-qkrj55, REQ-fnsjb6, REQ-8e57wm, REQ-szt6ea, REQ-ur7z4a

**LLR-9arvqj**: `OrgIo::next_received(&self) -> Result<ReceiveOutcome,
ReceiveError>` waits for the next body on the accept channel and reads no
stream while holding the `OrgService` lock or any store file; it takes the
lock only around `check_received`, releases it for the chain read, and takes
it again only around `commit_checked` and the store write that follows; every
other `OrgIo` method that takes the lock completes while `next_received` is
waiting.
satisfies: REQ-qkrj55

*Note 2026-10-13 (S4, task T11):* as landed, `OrgIo { node:
tokio::sync::Mutex<OrgService>, .. }`, and every method takes `&self`;
`next_received<R: RngCore + CryptoRng + Send>(&self, rng: &mut R) ->
Result<Received, ReceiveError>` (the rng is `commit_checked`'s; `Received`
is `Committed(ReceiveOutcome)` or `Acknowledged(VerifiedAcknowledgement)`).
It waits on the receive channel's own lock (`Endpoints::next_inbound`),
never the node's. The submission (`found_organisation`,
`submit_commit_send`) holds the node's lock for its whole sequence, chain
write included: the item covers the receive step only. Test:
`a_command_completes_while_the_receiver_waits`
(`org-io/tests/receive_step.rs`), mutation-checked with the lock taken
before the wait.

**LLR-cvyyr8**: for each body `next_received` calls `check_received` first;
only when it accepts does it read the chain once, for
`CheckedMessage::org_id()`, through S2's chain read, then calls
`commit_checked` with that state and writes the returned `StoreWrite`
(LLR-b9bm36) before returning the outcome; a refusal at any step returns that
step's typed error and reads no chain after a refusal of `check_received`.
The connection's authenticated key is not passed to org-node.
satisfies: REQ-fnsjb6

*Amended 2026-10-07 before first merge (reconciled with S3a, `60c7d58`):*
the last sentence is reversed. `next_received` calls
`check_received(sender, body)` with the DevicePublicKey the connection
authenticated (S3a's member-sender rule); a `Checked::Acknowledged` result
reads no chain and writes nothing; for a `ReadChain` result it reads the
chain once and calls `commit_checked` with that state and a
`device_secrets` closure that reads each named Device key's keychain item
(LLR-677fqq), then applies the returned `StoreChange` (`write_store`, or
`remove_store` for `Forgotten`, LLR-amv4mb) before returning the outcome.

*Noted 2026-10-08 (reconciled with S2, `e41cb09`):* S2's merged LLR-7pj5af
(`2026-10-08-org-io.md`) already states the order "chain-free phase, then
at most one read, then apply" for `receive_and_verify` and its self-delete
twin, with tests in `receive_order.rs`. This item does not restate it: it
adds what S4 changes around it (the undecoded body and its sender,
`Checked::Acknowledged`, the `device_secrets` closure, applying the
`StoreChange`). LLR-7pj5af gets a dated note in task T4 (one method; the
self-delete twin retired) and T11 (the method is `next_received`).

*Note 2026-10-13 (S4, task T11):* as landed, a refusal of
`check_received` or `commit_checked` is `ReceiveError::Refused`, a failed
read or a refused parse of the state read `ReceiveError::Chain`, a store
change not written `ReceiveError::Store` (the Persona reloaded,
LLR-murjr6), and a keychain refusal of a secret the commit signs with
`ReceiveError::Keychain`: the closure keeps the first refusal
(`Keychain::for_signing_noting_refusal`), and org-node's
`DeviceSecretNotSupplied` that follows is reported as that refusal
(REQ-6d9xv6). Tests (`org-io/tests/receive_step.rs`):
`a_declared_first_admission_is_committed_by_the_one_path`,
`a_refused_check_reads_no_chain`, `an_accepted_body_reads_the_chain_once`,
`the_sender_org_node_sees_is_the_handshakes`,
`an_acknowledgement_is_settled_with_no_chain_read_and_no_store_write`,
`an_accepted_revocation_signs_with_the_keychain_secret_and_removes_the_store`,
`a_locked_keychain_refuses_the_revocation_and_changes_nothing`,
`the_store_write_lands_before_the_outcome_returns`.

*Amended 2026-10-14 before first merge (S4, task T22):* the node is
released for the read, so a local commit (a submission, a reconcile) can
change the record the check read before `commit_checked` runs. org-node
then refuses the commit with `RecordMovedSinceCheck` and nothing is
committed (org-node's problem report on the stale check, in
`org-node/docs/problems/`). On that refusal, and only on it,
`next_received` repeats the step once from `check_received`, under the same
rules (a refused check reads no chain; an accepted one reads once), and
returns the result of the repeat, a second `RecordMovedSinceCheck`
included. Test: `a_check_that_went_stale_is_checked_again_once`
(`org-io/tests/receive_step.rs`), with the chain read held open by
`FakeChain::hold_next_read` while the local commit lands.

**LLR-wvgec4**: the accept path takes every inbound connection whose
handshake completes, whether its path is direct or relayed, applies no
allow-list of remote ids at accept time, and passes each message with its
sender to `check_received` (LLR-cvyyr8), so a Device a held record no longer
lists reaches org-node's member-sender rule and only its `Acknowledgement`
can be acted on (`Received::Acknowledged`); its `OrgInformation` or
`Revocation` for that Organisation is refused by `check_received`
(`SenderNotListed`) before any chain read or store write, and bytes that do
not decode are refused as `MalformedMessage`.
satisfies: REQ-ur7z4a

*Note 2026-10-13 (S4, task T11a):* the revocation the tests send from the
revoked Device names the receiving Member's own Device (a notice carries
no signature, so the revoked Device can build one from its own notice):
that is the notice org-node's member-sender rule refuses as
`SenderNotListed`; a notice naming another Device is refused earlier, as
not for this Device. Tests (`org-io/tests/acknowledgement_inbound.rs`):
`a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted`,
`a_revoked_device_sending_anything_but_an_acknowledgement_is_refused_and_served_nothing`,
`an_acknowledgement_a_member_still_listing_the_device_refuses_is_answered_refused`,
`an_acknowledgement_relayed_by_another_device_is_refused`,
`the_accept_path_has_no_allow_list`.

**LLR-unqfj5**: for each inbound message the accept task keeps the
stream's send half until the receive step answers through the message's
`oneshot` responder, then writes exactly one byte, `Verdict::Accepted`
(`0x00`) when the message was acted on and its store change (if any)
written, `Verdict::Refused` (`0x01`) for any refusal or failure, and
finishes the stream; when the responder is dropped it writes nothing. No
other byte is written on an inbound connection by any code in `org-io/src`.
satisfies: REQ-ur7z4a

*Ruled 2026-10-08 (owner, plan §16.4 point 1):* one verdict byte,
accepted or refused, on ack streams, as stated; the item is no longer
pending a ruling.

*Note 2026-10-13 (S4, task T11a):* as landed, `transport::Verdict`
(`as_byte`); `OrgEndpoint::recv_answerable` returns the body with an
`InboundReply` (the connection and the stream's send half), whose
`answer(verdict)` writes the byte, finishes, and holds the connection
until the sender closes it or ten seconds pass. Each accept task gives
every body a responder (`Inbound::verdict`) whose waiting task holds the
reply apart from the accept loop; `next_received` answers `Accepted` when
it returns a message and `Refused` for every error, a failed store write
included; `recv_body` (and a responder dropped unanswered) writes nothing.
The test dialler is `test_support::dial_direct_as(device_key, addr, body)
-> Result<Vec<u8>, TransportError>`: it binds a Loopback endpoint from the
Device's keychain item (no seed in hand, so iroh's secret key stays named
in `bind_from_keychain` alone), dials the direct addresses only and
returns every byte read back. Tests
(`org-io/tests/acknowledgement_inbound.rs`):
`a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted`,
`a_revoked_device_sending_anything_but_an_acknowledgement_is_refused_and_served_nothing`,
`an_acknowledgement_a_member_still_listing_the_device_refuses_is_answered_refused`,
`a_dropped_responder_writes_nothing`,
`nothing_but_the_verdict_is_written_on_an_inbound_stream`.

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
(LLR-cvyyr8's note) and `Connection(TransportError)`, one inbound
connection that failed before it carried a body, after which the loop goes
on. Tests (`org-io/tests/receive_step.rs`):
`a_closed_endpoint_is_terminal_endpoint_closed`,
`a_failed_bind_is_terminal_bind_failed`, `a_malformed_body_is_not_terminal`.

*Note 2026-10-15 (S4 review round 2, finding 7):* `Connection` is driven
through `next_received` by `a_connection_failing_before_a_body_is_not_terminal`
(`org-io/tests/receive_step.rs`): a peer that closes before opening a
stream, after which the next body is received. A connection dropped at
LLR-8uf9a5's time limit is `Connection(TransportError::TimedOut)`. The
receive-order tests (`org-io/tests/receive_order.rs`) now assert `Refused`
and `Chain` apart.

## The store files

**SDD-egx8rv**: the store files. Code: `org-io/src/store_file.rs` and the store-opening part of `OrgIo`'s
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
they satisfied; the atomic replace, which LLR-cvyyr8 relies on, stays.

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
would put iroh types in org-io's public API (LLR-3vb4k9, LLR-d5y7ks).

**LLR-amv4mb**: when an `OrgService` operation returns
`StoreChange::Forgotten { device_key }` (S3a's one removal step: an
accepted revocation, or a commit that removes the node, on a one-Persona
store), `remove_store(data_dir, device_key)` removes that Persona's store
file and, on Unix, syncs the directory, before the method reports success;
the next listing (LLR-nt8dja) holds no file for that key, no other
Persona's file is touched, and the Persona's keychain item is left in place
for the acknowledgement delivery (S3b-io). When the removal fails the
method reports it and not success, and reloads the Persona from its file as
LLR-murjr6 does.
satisfies: REQ-g9cu2c

*Added 2026-10-07 (reconciled with S3a, `60c7d58`):* with one store per
Persona, S3a's `forget_organisation` empties the store it is applied to, so
org-node returns a removal instead of sealed bytes.

*Ruled 2026-10-08 (owner, plan §16.4 point 3):* a forgotten Persona's
keychain item is kept until S3b-io's acknowledgement delivery for it ends or
the Organisation's acknowledgement retry limit passes (roadmap ruling C,
default two months), and is then deleted. S4 only removes the store file and
leaves the item; deleting it, and recording when the Persona was forgotten so
the limit can be measured, are S3b-io's. Until S3b-io merges the item stays
(no acknowledgement is delivered before then either).

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

*Moved 2026-10-13 from app (`app/docs/architecture/2026-10-05-decomposition.md`),
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

**LLR-t4v69b**: no code in `org-io/src` opens, reads, writes or names
`outstanding_invites.json` or `persona_store.bin`; outstanding invites exist
only inside the sealed stores.
satisfies: REQ-pnr6kh

## The keychain

**SDD-4fvypt**: the keychain. Code: `org-io/src/keychain.rs`. SOUP: `keyring` (inventory row written in task T7).
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

**LLR-n9dnm4**: `write_device_secret(device_key, secret: &DeviceSecret) ->
Result<(), KeychainError>` calls `secret.lend_to_keychain(|bytes|
entry.set_secret(bytes))` and nothing else with the bytes; `OrgIo::create_persona`
writes the keychain item before the store file, and when the keychain write
fails it drops the new Persona's store from the service, writes no file and
returns the refusal.
satisfies: REQ-rz93uz

*Note 2026-10-13 (S4, task T9):* as landed, `OrgIo::create_persona(rng,
handle, name, surname) -> Result<PersonaId, CreateError>` replaces S2's
`NodeBuilders::create_persona`. `CreateError::Keychain` is the refusal.
When the keychain write succeeds and the store file is then not written,
the new Persona has no file to reload from: it is dropped too
(`CreateError::Store(StoreFailure::Dropped)`, LLR-murjr6), and its keychain
item stays. Both drops use org-node's `OrgService::drop_store`.

**LLR-8erfbu**: `keyring::Error::NoEntry` maps to
`KeychainError::ItemMissing { device_key }`; `NoStorageAccess` to
`KeychainError::Locked { device_key }`; every other error to
`KeychainError::Unavailable { device_key, reason }`, where `reason` is the
error's kind name and never a byte of a secret; no path through `org-io/src`
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

## The invitation exchange runner

**SDD-rr7dtw**: the invitation exchange runner. Code: `org-io/src/exchange.rs`. Moved beside it: the app's invitation operations
and requirements (producing an Invite, producing a reply, refusing a reply
that is not outstanding, settling), with their low-level items.
traces: REQ-a2a6rm, REQ-pnr6kh, REQ-prjja8, REQ-tcutr6, REQ-65xqp8

*Amended 2026-10-14 (S4, task T13):* the three moved requirements joined
`traces:` when they arrived from the app with their low-level items, which
sit below.

**LLR-h9qf3s**: `OrgIo::issue_invite(org_id, org_name, invitee_name) ->
Result<String, ExchangeError>` calls `build_invite`, writes the returned
`StoreWrite`, and only then returns the Invite's Blob; a failed write returns
the failure and no Blob.
satisfies: REQ-pnr6kh, REQ-a2a6rm

**LLR-9cb443**: `OrgIo::produce_reply(invite_blob, persona_id, confirmed:
bool) -> Result<String, ExchangeError>` refuses with
`ExchangeError::NotConfirmed`, parsing nothing and writing nothing, unless
`confirmed`; otherwise parses the Invite, calls `build_reply`, writes the
returned `StoreWrite`, and only then returns the reply's Blob.
satisfies: REQ-a2a6rm

**LLR-y4dg74**: `OrgIo::admit_reply(org_id, reply_blob, peer_addr) ->
Result<AdmitReport, ExchangeError>` runs, stopping at the first refusal:
parse the reply; `check_reply(reply, org_id)`; `admit_member`, writing its
`StoreWrite`; the chain write (S2's submit path); `commit_update` with the
state read back, writing its `StoreWrite`; the send of every
`OutgoingMessage` (SDD-v2bq89); `settle(org_id, invite_id)`, writing its
`StoreWrite`. Once `commit_update` has succeeded it settles whatever the
sends reported, and `AdmitReport { member_id, sends }` carries the per-Device
outcomes; a failure before the chain write submits nothing.
satisfies: REQ-a2a6rm

*As landed 2026-10-14 (S4, task T13), LLR-h9qf3s, LLR-9cb443 and
LLR-y4dg74 (clarification):* each operation takes the caller's random source
(`rng`) first, as every building method of `OrgIo` does, and takes the node's
lock inside; `admit_reply` takes the lock for the parse-free check, then
calls the handle's own `admit_member` and `submit_commit_send` (S2's submit
path as T12 reshaped it), then takes it again for `settle`, so the chain
write and the sends run as they do for any submission. `ExchangeError` is
`NotConfirmed`; `Node(OrgNodeError)` (a Blob that does not parse, naming the
field; `check_reply`'s `InviteNotOutstanding` and
`ReplyForAnotherOrganisation`; a build refused); `Store(StoreFailure)` (the
operation's store write failed, the Persona reloaded or dropped); `Submit`
(the submission's report as text: nothing committed, or committed but not
stored, the Persona reloaded from a file that holds no commit, so the pair
stays outstanding); and `Settle { admitted: AdmitReport, error:
OperationError }` (committed and sent, the settle refused or not written).
`AdmitReport`'s `member_id` is read from the committed record by the
reply's member key. The fourth operation, `OrgIo::import_reply(reply_blob)
-> Result<InviteReply, ExchangeError>`, parses the reply and runs
`check_reply` for the Organisation the reply names, writing nothing; it is
LLR-gha5f6's check, which the app's reply panel shows before admitting.
`InviteReply`, `AdmitReport` and `ExchangeError` joined the handle's return
allowlist. `OrgIo::expect_admission` is compiled in test builds only: a
caller declares an expectation through `produce_reply`, behind the
confirmation; org-io's tests that need a bare expectation keep it.

*Moved 2026-10-14 from app (`app/docs/architecture/2026-10-06-invitation.md`,
under the app's command-surface item), plan task T13 (ruling A).* The five low-level
requirements below came with their IDs, their notes and their fourteen tests
(the app's `tests/invitation.rs`, now `org-io/tests/invitation_exchange.rs`,
`verifies:` lines unchanged). They were written for the app's functions over
org-node's service and an outstanding-invites file; each is amended below
for the runner, whose operations are the ones above. Where their text names
"this device", it is the node org-io runs.

**LLR-9sraks**: `issue_invite(svc, outstanding, rng, org_id, org_name,
invitee_name)` returns an Invite Blob carrying `org_name` and `invitee_name`
as typed, `org_id`, the DevicePublicKey of every Persona of this device bound
to that Organisation, and an invite identifier of 32 bytes drawn from `rng`,
which it adds to the outstanding identifiers before returning. For an
Organisation no Persona of this device is bound to it refuses, issuing no
Blob and adding no identifier.
satisfies: REQ-prjja8

Normal: `an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id`.
Abnormal: `no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to`.

*Amended 2026-10-14 (S4, task T13; clarification):* the operation is
`OrgIo::issue_invite(rng, org_id, org_name, invitee_name)` (LLR-h9qf3s);
org-node's `build_invite` draws the identifier and keeps the pair in the
store of the Persona bound to the Organisation (one Persona, one
Organisation), so "every Persona bound to it" is that one Persona's
DevicePublicKey. The refusal is org-node's (`OrgNotOnChain`).

**LLR-f35pda**: `OutstandingInvites` keeps each outstanding Invite as the pair
(Organisation identifier, invite identifier) it was issued for, in one file,
`outstanding_invites.json` in the data directory, as a JSON list of objects
`{"org_id": <40 hex>, "invite_id": <64 hex>}`, written before `issue` or
`settle` returns, so that reopening the file yields exactly the pairs issued
and not yet settled. `holds(org_id, invite_id)` is true only for a pair it
keeps — an identifier is not outstanding for any other Organisation — and
`settle(org_id, invite_id)` removes that one pair and leaves the others. A
file that is not such a list is refused when opened, naming the file; that
includes the earlier format, a list of bare invite identifiers, which is not
migrated (a PoC device deletes the file and issues its Invites again).
satisfies: REQ-65xqp8, REQ-prjja8

*Amended 2026-10-06 (independent review round 2, finding-1).* The file held
bare invite identifiers, so an identifier was outstanding for every
Organisation; it now holds the Organisation each was issued for, as
org-node keys its expected admissions by (Organisation, invite identifier).

*Note 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The requirement's text holds. The reason the note above gives does not:
org-node now keys its expected admissions by Organisation alone. The pair is
still what binds a reply to the Invite this device issued (REQ-65xqp8).

*Amended 2026-10-14 (S4, task T13; rewritten as plan §3.5 rules, owner
ruling of 2026-10-06 on the encrypted store, REQ-pnr6kh):* each outstanding
Invite is the pair (Organisation identifier, invite identifier) it was
issued for, kept in the sealed store of the Persona that issued it (org-node's
`outstanding_invites`), written to that Persona's store file before
`issue_invite` or `admit_reply`'s settle returns, so that reopening the data
directory yields exactly the pairs issued and not yet settled. A pair is
outstanding for its own Organisation only, and the settle removes that one
pair and leaves the others. No `outstanding_invites.json` is read or
written: a file of that name in the data directory, well formed or not, is
ignored and refuses nothing, and its pairs are not outstanding (no
migration). The file-format refusal above is withdrawn with the file.
Normal: `outstanding_invite_ids_survive_a_reopen_and_settle_one_at_a_time`.
Abnormal: `an_outstanding_invites_file_in_the_data_dir_is_never_read`
(rewritten from the file-format refusal test),
`a_reply_naming_another_held_organisation_with_an_outstanding_invite_id_is_refused`.

**LLR-w4mhd4**: `produce_reply(svc, rng, invite_blob, persona_id, confirmed)`
refuses, producing nothing and declaring nothing, when `confirmed` is false —
with a message that begins `confirm first` — when `persona_id` names no
Persona of this device, and when it names one bound to an Organisation, as
LLR-rt8gdz states (amended 2026-10-06, review round 1). Otherwise it returns an Invite reply Blob carrying that
Persona's Member-as-a-group key, DevicePublicKey, handle, name and surname and
the Invite's Organisation identifier and invite identifier, and before
returning declares to org-node, through `expect_admission(org_id)`, the
first admission to that Organisation it expects.
satisfies: REQ-tcutr6

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The declaration was `expect_admission(org_id, invite_id)`. REQ-tcutr6 as
amended declares the Organisation alone, because org-node matches a first
admission against the Organisation alone; the reply still carries the invite
identifier back to the inviter.

Normal: `a_confirmed_reply_carries_the_persona_and_declares_the_expected_admission`.
Abnormal: `no_reply_is_produced_and_nothing_declared_without_confirmation`.

*Amended 2026-10-14 (S4, task T13; clarification):* the operation is
`OrgIo::produce_reply(rng, invite_blob, persona_id, confirmed)`
(LLR-9cb443); the unconfirmed refusal is `ExchangeError::NotConfirmed`,
whose message begins `confirm first`; the declaration is org-node's
`build_reply`, which records the expectation in the Persona's store, written
before the Blob returns; the other refusals are org-node's. `satisfies:` was
REQ-tcutr6 and the app's confirmation requirement, which stays in the app
and is not exported, so it is named in prose; the app's IPC test of the
refusal stays in the app.

**LLR-rt8gdz**: a Persona bound to an Organisation is not offered, and is
refused, as the Persona to found an Organisation with or to reply to an
Invite as. `produce_reply` refuses a `persona_id` that names a Persona of this
device bound to an Organisation, with org-node's `PersonaAlreadyBound` message
for that Persona, after the confirmation check and before declaring any
expectation, producing no reply. `found_organisation` refuses such a Persona
with the same message, from org-node's `create_organisation`, before calling
the chain writer. The webview offers, as the Personas to reply as, only those
`unboundPersonas` (`app/src/lib/invite.ts`) keeps — the Personas whose
`org_id` is null, in the order given; the founding panel offers no choice, and
founds with the Persona it has just created.
satisfies: derived

Normal: `a_bound_persona_cannot_reply_and_an_unbound_one_on_the_same_device_can`.
Abnormal: `no_reply_is_produced_and_nothing_declared_for_a_bound_persona`,
`a_bound_persona_founds_nothing_and_the_chain_is_not_written`.

Review round 1, finding-1 (2026-10-06): one Persona, one Organisation (owner
ruling). Derived, because the founding half has no parent among this unit's
requirements; the reply half narrows REQ-tcutr6's "a Persona the user chose".
The `unboundPersonas` filter sits in `invite.ts`, the file of the app's
revocation-input design item, beside `replyGate`; the backend refusal is
what holds when the webview is bypassed.

*Amended 2026-10-14 (S4, task T13; narrowing):* the item moved with the
backend refusals, which are org-io's `produce_reply` and
`found_organisation` over org-node's `build_reply` and `create_organisation`.
The webview's offer — the `unboundPersonas` filter and the founding panel's
lack of a choice — is the app's code and stays the app's; this unit does
not verify it, and the app's tests of that filter stay in the app. The
app's design item the text named by ID is named in prose.

**LLR-gha5f6**: `check_reply(outstanding, reply_blob)` refuses, with `this
reply names no Invite this device has outstanding`, a reply whose pair
(its `org_id`, its invite identifier) is not outstanding (LLR-f35pda).
`admit_reply(svc, writer, outstanding, rng, org_id, reply_blob, peer_addr)`
refuses, acting on nothing, a reply `check_reply` refuses, and
one whose outstanding pair's Organisation is not `org_id`, with `this reply is
for another Organisation`. The target Organisation is the one the outstanding
pair names: `org_id` is kept as the operator's selection in the Admit panel,
which preselects it from the reply, and is not trusted — it can only confirm
that Organisation, never choose another. Otherwise `admit_reply` builds
org-node's `Joiner` from the reply's five values, has org-node build the
admission to that Organisation, submits it as org-io's write-then-commit-then-send
order states (the submission item) with the reply's DevicePublicKey as the
one recipient (the one-send-per-command item, retired 2026-10-13 in S4 task
T12: the submission now sends to every recipient the commit names), passing
org-node no Organisation secret, key or invite identifier, and settles that
pair once the admission has committed — even when the send that follows
fails; when the submission fails the pair stays outstanding.
satisfies: REQ-65xqp8

Normal: `a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding`.
Abnormal: the same test's reply to an Invite this device never issued and its
second use; `a_reply_whose_admission_fails_on_chain_stays_outstanding`,
`a_reply_admitted_under_another_selected_organisation_is_refused` (a true
reply, with a second Organisation the inviter does hold selected),
`a_reply_naming_another_held_organisation_with_an_outstanding_invite_id_is_refused`
(the reply itself names that second Organisation, and is admitted as the
shipped panel would, under its own `org_id`),
`a_reply_is_settled_once_committed_even_when_the_send_fails`.
Review round 1, finding-4 (2026-10-06): the refusal test used to name an
Organisation the inviter holds no record of, so admission failed without the
check; and the "even when the send fails" clause had no test.

*Amended 2026-10-08 (change `worktree-org-io-create`, task T9).* The
signature was `admit_reply(svc, writer, outstanding, rng, org_id, reply_blob, peer_addr)`
and is `admit_reply(io, outstanding, rng, org_id, reply_blob, peer_addr)`:
the service and the chain writer are both behind the org-io handle, whose
`submit_commit_send` it calls. `satisfies:` was REQ-65xqp8 and REQ-nfr3n2;
REQ-nfr3n2 moved to org-io, and an app low-level requirement does not
decompose another unit's requirement, so it was replaced by the app's
requirement on the decision to admit, handed to org-io. Classification under
the hybrid rule: a clarification, not a change of meaning; the behaviour
and the tests are unchanged. *(Carried 2026-10-14 with the item; the app's
requirement is named in prose.)*

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
`admit_reply` took an `org_secret` and passed it, with the reply's invite
identifier "in the Wire message", to `send_update`. org-node's `send_update`
now takes neither: the Organisation private key comes from org-node's record,
and the invite identifier never travels between peers (REQ-tcutr6 as
amended). The invite identifier still settles the outstanding pair here.

*Amended 2026-10-06 (independent review round 2, finding-1).* The
Organisation check compared the reply's `org_id` with the `org_id` the panel
had preselected from that same reply, and the outstanding check looked at the
invite identifier only, so a reply to an Invite for one Organisation that
named another this device holds was admitted there. The check is now on the
pair, and the test `a_reply_for_another_organisation_is_refused` is rewritten
as `a_reply_admitted_under_another_selected_organisation_is_refused`.

*Amended 2026-10-14 (S4, task T13; clarification):* the operations are
`OrgIo::import_reply(reply_blob)`, which is the check, and
`OrgIo::admit_reply(rng, org_id, reply_blob, peer_addr)` (LLR-y4dg74). The
check is org-node's `check_reply(reply, selected)`: a reply whose
Organisation is not the selected one is refused as
`ReplyForAnotherOrganisation`, and one whose pair is not outstanding as
`InviteNotOutstanding`, both in `ExchangeError::Node`, before anything is
built (`import_reply` selects the Organisation the reply names). As before,
only the pair's Organisation can be the target. The send reaches every
recipient the commit names (LLR-znhmh2, LLR-r9sh6k) and its outcome is
reported per Device in the `AdmitReport`, not as the call's error; the
settle follows the commit whatever the sends reported. `satisfies:` was
REQ-65xqp8 and the app's requirement on the decision to admit, which stays
in the app and is not exported, so it is named in prose.

## A refused update changes nothing (beside SDD-erzj3m)

*Added 2026-10-15 (S4, owner ruling of that day).* SDD-erzj3m, the
submission item, is in `2026-10-08-org-io.md`; this item states what
happens on the submission path when org-node refuses to build the update
at all, so there is nothing to submit. It gives the moved test
`a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing`
(submit_flow), which ran with no `verifies:` line since task T12, an item
of its own.

**LLR-nqdr97**: When org-node refuses to build an update `OrgIo::revoke_member`
asks it for (a revocation of a Member id the Organisation's record does not
hold, or any other refusal org-node returns before an update exists),
`revoke_member` returns the refusal as its error and nothing on the
submission path happens: the record is not committed (its epoch is
unchanged), org-node holds no provisional update for the Organisation,
nothing is written to the chain (the Organisation's on-chain epoch is
unchanged), and no body is sent to any Device.
satisfies: derived

Normal: the revocation in
`each_command_sends_once_to_one_device_and_the_kind_follows_the_record`
(submit_flow), where `revoke_member` of a held Member builds an update that
`submit_commit_send` commits and sends.
Abnormal: `a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing`
(submit_flow): an unknown Member id, refused, with the record's epoch, the
held provisional updates, the chain's epoch and a listening Device all
checked afterwards.

*Clarified 2026-10-15 (S4 TEST review round 3, notes):* the "Normal:"
pointer above names a test that verifies LLR-r9sh6k, not this item. This
item has no normal-case test of its own, and needs none: it states only
what happens when org-node refuses, and the normal path of
`revoke_member` (a held Member's revocation builds an update, which is
submitted, committed and sent) is a successful submission, verified under
LLR-r9sh6k (the Loopback send,
`each_command_sends_once_to_one_device_and_the_kind_follows_the_record`)
and LLR-znhmh2 (the Networked fan-out, `tests/send_fanout.rs`). The
refusal for a Member id the record does not hold is org-node's
`OrgNodeError::Trie(OrgMembersError::IdNotFound)`, returned as
`OperationError::Node`; the abnormal test asserts that variant.

## Sending, and the peer address

### SDD-v2bq89 — Sending committed bodies

**SDD-v2bq89**: the fan-out that sends each body of a committed update to its
recipient. Code: `org-io/src/send.rs`. Moved beside it: org-node's loopback
dialling items.
traces: REQ-6skn67, REQ-8trkps

*Narrowed 2026-10-15 (S4, ARCHITECTURE conformance review finding-5; owner
ruling of that day):* it read "sending, and the peer address", with
`org-io/src/peer_addr.rs` beside `send.rs` and REQ-n82q47 among its traces:
two responsibilities. The peer-address parse is SDD-wr8m5n's (below), with
REQ-n82q47 and LLR-2b2ghe; this item keeps the fan-out, LLR-znhmh2 and
LLR-r9sh6k.

*Restructured 2026-10-15 (S4, ARCHITECTURE conformance review finding-2; no
change of behaviour):* the per-recipient decision is taken by
`send::plan_send(mode, recipients, addressed)`, a pure function returning
the whole-send refusal (`NoLoopbackAddress`, `AddressNotARecipient`) or one
step per recipient (dial by Device, dial at the address, not attempted);
`send_committed` binds the sender's endpoint and carries the plan out.

**LLR-znhmh2**: in Networked mode `OrgIo::send_committed(outcome: &CommitOutcome,
peer_addr: Option<PeerAddr>) -> SendReport` dials every
`OutgoingMessage::recipient` by the endpoint id equal to its Device key,
sends its body, attempts every recipient whatever happened to the others, and
returns `SendReport { per_device: Vec<(DevicePublicKey, Result<(),
SendError>)> }` in `outcome.outgoing`'s order; it ignores `peer_addr`, writes
nothing, and never undoes the commit.
satisfies: REQ-6skn67

**LLR-r9sh6k**: in Loopback mode `send_committed` refuses with
`SendError::NoLoopbackAddress`, sending nothing, when `peer_addr` is `None`,
and with `SendError::AddressNotARecipient`, sending nothing, when the
address's endpoint id is the Device key of no recipient; otherwise it sends
only to that recipient, at that address, and reports every other recipient
as `SendError::NotAttemptedInLoopback`.
satisfies: REQ-8trkps

*As landed 2026-10-13 (S4, task T12), LLR-znhmh2 and LLR-r9sh6k
(clarification):* `send_committed` is crate-private
(`org-io/src/send.rs`): the handle's `submit_commit_send(rng, update,
peer_addr: Option<PeerAddr>)` calls it after the commit's store write, with
the Device of the Persona the commit names as its sender (LLR-d2pzzk), and
returns `Submitted { org_id, epoch, root, sends: SendReport }`; no public
method takes or returns a `CommitOutcome`. `SendError` carries, beyond the
three named, `Transport(TransportError)` for a failed dial or stream and
`SenderEndpoint(String)` when the sending Persona's endpoint cannot be had
(every recipient then reports it, nothing sent). Each refusal of the whole
send (`NoLoopbackAddress`, `AddressNotARecipient`) is reported against every
recipient. Recipients are dialled one after another, in the commit's order.
The sender does not read the verdict byte a receiver writes back
(LLR-unqfj5): a send is reported delivered once the receiver has
acknowledged the stream's bytes, as before the verdict existed.

### Moved from org-node: the Loopback dial (S4, task T12)

*Moved 2026-10-13 from org-node's design ledger
(`org-node/docs/architecture/2026-10-03-decomposition.md`, which keeps a prose
pointer), plan task T12 of `docs/plans/2026-10-06-org-io-transport.md`
(ruling A), with org-node's report of the unchecked Loopback address (now in
`org-io/docs/problems/2026-10-13-loopback-address.md`,
resolved by LLR-r9sh6k).* Both items stated org-node's `send_update`, which
the commit's bodies replaced (S4, task T5); the send is org-io's
`send_committed`. The first was parented to org-node's member-sender
requirement and the second was derived; both are re-parented to REQ-8trkps,
which states the Loopback send, and the second is no longer derived. Their
history notes named org-node items, which are named in prose here; the full
notes stay in org-node's ledger history.

**LLR-jn5jeh**: in Loopback mode `send_update` dials the recipient at the full
`EndpointAddr` its caller passes, not by endpoint identity alone. Nothing
checks that the address names the recipient's DevicePublicKey, so an
Organisation-information message and the Organisation private key it carries
go wherever the address points. That is PR-2dmjzj's
defect, stated here and not endorsed. One of its cures (dial by key in both
modes) would replace this requirement.
satisfies: REQ-8trkps

*Amended 2026-10-13 (S4, task T12; narrowing):* `send_committed` dials the
addressed recipient at the full address the caller passes, not by endpoint
identity alone. The defect clause ("nothing checks that the address names
the recipient's DevicePublicKey") is withdrawn: the address is now sent to
only when its identity is a recipient's Device key (LLR-r9sh6k), which
resolves PR-2dmjzj. Re-parented from org-node's member-sender requirement to
REQ-8trkps.

*Amended 2026-10-06 and 2026-10-05 in org-node:* the Organisation secret it
named became the Organisation private key every Organisation-information
message carries, and the dial moved from `admit_member` to `send_update`.
*Reworded 2026-10-04 by org-node's review round 2:* the test that carried it
called `admit_member` in Loopback mode and saw the admission arrive at the
endpoint the address named; its negative (the same identity with no
transports) was a mutation, because iroh retries a live identity past any
gated timeout. In org-io it is carried by
`loopback_sends_to_the_matching_recipient_only_and_reports_the_rest_not_attempted`
(`tests/send_fanout.rs`).

**LLR-pw369n**: in Loopback mode `send_update` refuses with a typed error, and
sends nothing, when its caller passes no `EndpointAddr`, rather than attempting
a peer it cannot name; in Loopback mode with an address it dials that full
address (LLR-jn5jeh).
satisfies: REQ-8trkps

*Amended 2026-10-13 (S4, task T12; clarification):* the send is
`send_committed`, and its typed refusal is `SendError::NoLoopbackAddress`,
reported against every recipient, before any endpoint is bound. Re-parented
from derived to REQ-8trkps, which states it; its derived assessment moved to
org-io's risk draft with it.

*Added 2026-10-04 by org-node's review round 3*, as the revocation twin of
the Loopback dial, which had been evidenced but unstated; *amended 2026-10-05*
when the chain write left org-node: the refusal then came after a commit the
chain already agreed with, so a missing address no longer cost an on-chain
epoch (org-node's revocation-order report).

### SDD-wr8m5n — The peer-address parse

**SDD-wr8m5n**: the total parse of a peer address given as text into the
transport-owned `PeerAddr`. Code: `org-io/src/transport/peer_addr.rs`
(re-exported as `org_io::peer_addr`).
traces: REQ-n82q47

*Added 2026-10-15 (S4, ARCHITECTURE conformance review finding-5; owner
ruling of that day):* split from SDD-v2bq89, which held the parse beside the
fan-out. The parse lives in the transport module since the same day's
finding-1 (LLR-3vb4k9), because `PeerAddr` wraps iroh's address type.

**LLR-2b2ghe**: `parse_peer_addr(text: &str) -> Result<Option<PeerAddr>,
PeerAddrError>` is total: blank or whitespace-only text is `None`; text that
is, after at most one `0x`, the hexadecimal of a postcard-encoded
`iroh::EndpointAddr` is `Some`; every other input is
`PeerAddrError::{Hex, Decode}` naming the field `peer_addr`; it never panics,
for any input.
satisfies: REQ-n82q47

*Re-traced 2026-10-15 (S4, ARCHITECTURE conformance review finding-5):* it
was SDD-v2bq89's; it is SDD-wr8m5n's, the item the parse now is. The
statement and its `satisfies:` are unchanged, and so is the parse's
behaviour (`tests/peer_addr.rs`, `fuzz_peer_addr`).

*As landed 2026-10-13 (S4, task T12), LLR-2b2ghe (clarification):* the
refusals read `peer_addr_blob hex: …` and `peer_addr decode: …`, the texts
the app's parse gave, which the moved items below state. Exactly one address
is accepted: bytes left over after the address are a `Decode` refusal, and a
bare `0x` (the hexadecimal of no bytes) is not blank, so it is a `Decode`
refusal too, where the app's parse, which stripped every leading `0x`, read
it as no address. `PeerAddr` is an opaque newtype: `endpoint_id()` gives the
identity's bytes and `to_hex()` the text the parse takes; only a test build
makes one from an `iroh::EndpointAddr`, so org-io's public surface names no
iroh type for it.

### Moved from the app: the peer-address parse (S4, task T12)

*Moved 2026-10-13 from the app's design ledger
(`app/docs/architecture/2026-10-05-decomposition.md`, which keeps a prose
pointer), plan task T12 (ruling A).* The three items stated the parse inside
the app's `revoke_member` and `admit_member` commands; the parse is org-io's
`parse_peer_addr` now, which the commands call (task T15 rewires them), and
the items' tests moved to `org-io/tests/peer_addr.rs`, `verifies:`
unchanged. The first was parented to the app's requirement that an empty
peer address is not a ground for refusal, and the other two were derived;
all three are re-parented to REQ-n82q47, which states the parse, so none is
derived any more.

*Moved back to the app 2026-10-15 (S4, TEST conformance review finding-4;
ruling A's move rule, IDs kept).* The three items state what the app's
`revoke_member` and `admit_member` commands do with the peer-address text
(blank is absent; a non-blank address is decoded from hexadecimal and then
as an endpoint address, and refused naming the step that failed). org-io's
handle takes no peer-address text — its `submit_commit_send` and
`admit_reply` take an already parsed `Option<PeerAddr>` — and the app's
commands are the only callers
of `parse_peer_addr`, so the seam that observes the items is the app's IPC
commands, not this unit's parse. Their definitions, with their T12
clarifications, are in the app's design ledger
(`app/docs/architecture/2026-10-05-decomposition.md`, under the
command-boundary parsing item), parented to the app's requirement that an
empty peer address is not a ground for refusal and, for the two decode
items, derived again; their tests are the app's IPC tests. The parse itself
stays here: REQ-n82q47 and LLR-2b2ghe, with `tests/peer_addr.rs` and the
`fuzz_peer_addr` target.

*Corrected in place 2026-10-15 (S4 review round 2, finding-13):* the
paragraph above said "org-io's handle takes no peer address", which was
stale: `submit_commit_send` and `admit_reply` take `Option<PeerAddr>`. It
now says the handle takes no peer-address text, only a parsed address.
