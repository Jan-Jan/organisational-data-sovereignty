# org-io stage S4 — transport, the invitation exchange and storage IO leave org-node

Design survey and draft for stage S4 of the org-io roadmap
(`docs/plans/2026-10-06-org-io-roadmap.md`): iroh (point-to-point transport),
the invitation exchange, and storage file IO move out of org-node into the new
org-io unit, and org-node becomes IO-free. Change worktree
`worktree-org-io-transport`, off local master `c036fab`. Written 2026-10-06.
**Survey and draft only: no code.** When first written, org-io did not exist
on this branch. Stage S2 (`worktree-org-io-create`) creates it.

*Revised 2026-10-06, after the owner ruled on every open decision of §11
(roadmap `docs/plans/2026-10-06-org-io-roadmap.md`, section "Owner's rulings
on the S2–S4 design drafts", and its "Two keys, and no private signing key is
ever exposed" ruling).* S2's org-io skeleton is merged into this branch
(`worktree-org-io-create` at `31368b0`). The design below is adjusted to the
rulings, §11 records each decision as ruled, and §12 lists the requirements
and risk items S4 has drafted, with their minted IDs. The code still starts
only after the key-pair change (`worktree-org-node-org-key-pair`) merges.

Settled rulings this draft builds on (owner, 2026-10-06; not re-asked):

- org-io is *the* IO unit. It covers the chain (through on-chain-client),
  other nodes over iroh (point-to-point now, gossip later), and local storage
  of **opaque bytes only**.
- org-node only serialises, deserialises, verifies and encrypts.
- The app does the UI and decides *when*. It depends only on org-io.
- The iroh endpoint id is the Device key.
- The invitation exchange moves in S4 (roadmap, owner's ruling 1). org-node
  builds and serialises the messages, org-io holds the store and runs the
  exchange, and the app keeps the UI.
- S2 comes first. PR-kwwap5 is re-targeted to S4.

Rulings of 2026-10-06 on this draft's own decisions (roadmap, "Owner's rulings
on the S2–S4 design drafts"; recorded per decision in §11):

- Moved items keep their IDs (ruling A).
- org-node meets org-io by values in, values out: no IO traits, no async in
  org-node; org-node returns the sealed store bytes for org-io to write
  (ruling B).
- One receive path; `receive_and_self_delete_if_revoked` is retired (fixes
  PR-kwwap5).
- Outstanding invites move into the encrypted store, with no migration.
- The PR-3ue4va fan-out to every Device lands in S4, in Networked mode.
- One iroh endpoint per Persona (PR-8qsnhx).
- Store writes are atomic and owner-only (PR-n696wv). *(Owner-only dropped
  2026-10-07; atomic stays.)*
- The receive loop holds no lock while it waits (PR-7v8anh).
- org-io is class C.
- S4's code starts after the key-pair change merges.
- No private signing key is ever exposed: not the user's sr25519 signatory
  key, not any device ed25519 key (roadmap, "Two keys" ruling; org-io's
  REQ-v4tfap). §2.2 resolves how the endpoint gets the device secret under it.

*Reconciled 2026-10-07 against the merged key-pair change.* Master `1f52c36`
(the key-pair change, `worktree-org-node-org-key-pair`) is merged into this
branch at `f51de1a`. §1, §3, §5 and §8 are re-surveyed at that commit: the
key-pair overlaps of §8 are merged facts now, not pending, and the line
numbers below are from `f51de1a` (the first survey's were from `c036fab`).
What the reconciliation changed is listed in §8.1.

*Owner rulings of 2026-10-07, on the reconciled draft:*

1. **S4 resolves PR-gnh3j2.** The commit builds every recipient's body
   inside the commit, with that commit's key (org-node LLR-gdfqn9, task T5).
2. **REQ-6d9xv6 is narrowed.** "Refuse to read or write" applies only to the
   operations that need the keychain item: creating a Persona, opening or
   reloading a store, binding an endpoint, and signing. An ordinary store save
   seals under the store key the open store holds in memory and reads no
   keychain (org-io requirement draft amended; org-node LLR-bvt5up).
3. **The keychain test double.** Task T7 verifies whether `keyring`'s mock
   shares one item between two `Entry` handles for the same service and user.
   If it does not, org-io supplies its own in-memory credential store under
   `test-support` only (org-io LLR-36jer8).
4. **No file-permission control, on any platform.** The owner: "I'm ok with
   anyone being able to read the files on disk, because they should be
   encrypted (i.e. simple file ACL is too weak a protection IMO)." The
   owner-only requirement and control drafted for S4 (REQ-hvkz4c,
   RC-hnxhw2; never merged) are dropped, their IDs not reused; LLR-b9bm36
   keeps the atomic replace and loses its mode and ACL clauses; HAZ-5n56w2
   is controlled by RC-d8dbxs (the keychain-derived store key) alone, its
   residual already accepted. No Windows ACL SOUP item.
5. **Platforms: Windows, macOS, Linux, Android and iOS.** "This software has
   to work on Windows, Mac, Linux, Android and iPhone." T7 checks the
   keychain crate's backend for each and records the gaps. Android (the
   crate is expected to lack an Android Keystore backend) is a later stage,
   not S4, unless T7 finds it trivial. "If this adds extra complexity on
   Windows, we can punt solving Windows to later": T7 decides, and a
   deferred Windows gets an unminted later-stage row in the roadmap.
6. **The OS keychain is touched in org-io only**, "to protect the lower
   layers from that". org-io's REQ-hw82ue and org-node's REQ-26mjqa now say
   so; T8 and T14 scan the lower units' manifests for a keychain crate.

*Reconciled 2026-10-07 against the merged S3a change (`60c7d58`, plan
`docs/plans/2026-10-06-org-io-commit-workflow.md`; ledgers
`org-node/docs/{requirements,risk,architecture}/2026-10-07-commit-workflow.md`),
merged into this branch at `d4a97ab`/`e0e4dac`.* §16 lists what S3a changed
under S4 and what each task now does about it; the tasks below are edited in
place. **S2 has not merged yet**: S4's develop phase starts only after it, and
the first step of T0 is to re-reconcile every task against S2's merged code,
as this pass did against S3a's. Nothing here tries to match S2's in-progress
code.

*Owner rulings of 2026-10-07, after the S3a merge:*

7. **The transport-only preflight moves into org-io in S4.** Until S4,
   org-node keeps `preflight.rs` and `bin/preflight.rs` reduced to the
   transport check, reading `ODS_DEVICE_SEED` only, gated on the `transport`
   feature (S2 moves the chain checks). S2's LLR-3zdw8v bars `DeviceSeed`
   from org-io, so S4's move does not carry `ODS_DEVICE_SEED` across: org-io's
   preflight binds through the keychain path and the move-only key handling
   (`bind_from_keychain`, LLR-d5y7ks) for a named Device key, and the
   environment variable goes. Task T10, LLR-h9tcat.
8. **Members accept inbound connections from Devices they hold as revoked,
   for acknowledgements only.** No other message kind from such a Device is
   acted on, and no Organisation data is served on the connection. A revoked
   Device connects directly to up to three distinct Member Devices at
   addresses it kept, with no relay, and tries the next address when one
   refuses. The revoked Device's side of that delivery (keeping the
   addresses, the direct dial, the next-address loop) is S3b-io's (S3 plan
   T13–T17); S4 provides the acceptance on the receiving side and the
   verdict a refusal is told by. REQ-ur7z4a, HAZ-abm3xs, RC-b4m5sj,
   LLR-wvgec4, LLR-unqfj5; task T11a.

*Reconciled 2026-10-08 against the merged S2 change (`e41cb09`, plan
`docs/plans/2026-10-06-org-io-create.md`, record
`docs/verification/2026-10-08-worktree-org-io-create.md`), merged into this
branch at `24ca24c`.* §17 lists what S2 landed under S4, the S2 rules S4
must keep, and what each task now does about it; the tasks below are edited
in place, and T0's steps 1–4 are done by this pass. Where §1–§16 disagree
with §17, §17 and the tasks win.

*Owner rulings of 2026-10-08, on §16.4 (applied in §16.4, §17 and the
drafts):*

9. **One verdict byte (accepted/refused) on ack streams** (§16.4 point 1),
   as LLR-unqfj5 states; the ALPN is unchanged.
10. **HAZ-abm3xs's residual is accepted** (§16.4 point 2), with its
    benefit-risk reasoning in org-io's risk draft.
11. **A forgotten Persona's keychain key is kept** until S3b-io's
    acknowledgement delivery ends or the Organisation's limit (roadmap ruling
    C, default two months) passes, **then deleted** (§16.4 point 3). S4
    removes the store file and leaves the item; the deletion is S3b-io's
    (LLR-amv4mb's note).
12. **The preflight's key is not an environment variable** (§16.4 point 4).
    There is no `ODS_DEVICE_KEY`. Environment-variable keys and seeds are for
    test environments only, and `ODS_ADMIN_SEED` is development-only, for
    on-chain signing. The transport preflight gets the device key through
    the same keychain path as the node (T10, LLR-h9tcat). The owner:
    "ODE_DEVICE_KEY is only for testing environments (and only used for
    on-chain signing). In future the user should choose between either
    allowing an automatic key creation (stored in OS keychain, HW key, etc)
    or importing an existing key they own (into OS keychain, HW key, etc.)."
    That choice is recorded as an unminted "(later)" row in the roadmap.

The design drafts that realise the requirements are
`org-node/docs/architecture/2026-10-07-sans-io.md`
(SDD-asxwp8, SDD-fysk96, SDD-esg2kc, SDD-bm27xt, SDD-xshv5b) and
`org-io/docs/architecture/2026-10-07-transport.md`
(SDD-xwft92, SDD-aw3nka, SDD-egx8rv, SDD-4fvypt, SDD-rr7dtw, SDD-v2bq89).
The implementation plan is "## Tasks" at the end of this file.

## 1. Survey: every IO site S4 concerns

### 1.1 org-node — transport (`org-node/src/transport/`, feature `transport`)

| Site | What it does |
|---|---|
| `transport/mod.rs:18-24` `TransportMode` | Loopback (relay off, loopback binds, dial by `EndpointAddr`) or Networked (`presets::N0`, dial by `EndpointId`) |
| `transport/mod.rs:27` `ALPN` | `/ods/org-node/1` |
| `transport/mod.rs:29-47` `TransportError`, `MAX_FRAME` | Bind/Connect/Accept/Stream/FrameTooLarge/Malformed; 1 MiB body cap |
| `transport/wire.rs:16-31` `WireMessage` | The key-pair change's enum: `OrgInformation { envelope, record_snapshot, org_private_key }` (index 0) or `Revocation { envelope }` (index 1); `envelope()` accessor. No secret, no invite id (LLR-js9dsu, LLR-ms8njy) |
| `transport/wire.rs:34-43` `encode_frame` | `len(u32 LE) ‖ postcard(msg)`; refuses a body over `MAX_FRAME` |
| `transport/wire.rs:46-51` `decode_body` | Refuses a body over `MAX_FRAME` before decoding; postcard decode → `Malformed` |
| `transport/endpoint.rs:54-56` `OrgEndpoint::bind` | Loopback bind from a `&SigningKeypair` |
| `transport/endpoint.rs:74-166` `bind_with_mode` | **IO.** iroh endpoint with `SecretKey::from_bytes(device.device_seed().expose_secret())` (`:78`). Loopback: `clear_ip_transports`, `127.0.0.1:0` (required), `[::1]:0` (optional). Networked: N0 relay + Pkarr/DNS |
| `transport/endpoint.rs:190-212` `bind_with_relay` (`test-support`) | **IO.** In-process relay plus `MemoryLookup`, relay-only paths; also `SecretKey::from_bytes(..expose_secret())` (`:196`) |
| `transport/endpoint.rs:229-236` `node_addr_for_dial` | `EndpointAddr` from id plus bound sockets |
| `transport/endpoint.rs:240-242` `inner` | The raw iroh endpoint, public (tests) — org-io must not hand this on (REQ-xuzw32) |
| `transport/endpoint.rs:254-275` `send`, `send_to_id` | **IO.** Dial by address or by id |
| `transport/endpoint.rs:279-307` `send_conn` | **IO.** connect, `open_bi`, `encode_frame`, write, `finish`, `stopped`, close(0) |
| `transport/endpoint.rs:314-341` `recv_one` | **IO.** accept, handshake, `remote_id` → `DevicePublicKey` via `ed25519_dalek::VerifyingKey`, `read_to_end(MAX_FRAME+4)`, strip the 4-byte prefix unchecked, `decode_body` |

### 1.2 org-node — the service's transport use (`org-node/src/service.rs`, features `app`)

| Site | What it does |
|---|---|
| `service.rs:25-27` | Imports `TransportError`, `TransportMode`, `OrgEndpoint`, `WireMessage` |
| `service.rs:283-291` `OrgService { store, chain, endpoint, transport_mode }` | One store for every Persona; the service owns the endpoint lazily |
| `service.rs:304-317` `set_transport_mode`, `with_endpoint`, `endpoint` | Endpoint configuration and injection |
| `service.rs:718-756` `send_update(outgoing, recipient, peer_addr: Option<iroh::EndpointAddr>)` | Chooses the kind per recipient from the record (LLR-6ymd6d): `OrgInformation` with the record's **current** `org_private_key` (a clone; PR-gnh3j2) or `Revocation`; sends from the first Persona bound to the Organisation (LLR-2xzys9); Loopback without an address refused before binding (LLR-pw369n); Networked dials `EndpointId::from_bytes(recipient)` |
| `service.rs:763-886` `receive_and_verify` | **Starts with IO** (`receive_one().await`, `:767`), then the chain-free gates (revocation not held `:783`, LLR-38e2kn; first admission not expected `:790-793`, LLR-s8xp7m), `verify_received` (chain read), the carried-key check (LLR-ba2ejp), then first admission / held update (`commit_received`, LLR-pt32fx) / own removal (`forget_organisation`, LLR-b27jr6). Holds the two unreachable `RevocationNotHeld` arms of PR-q8r32t (`:800`, `:841`) |
| `service.rs:914-956` `receive_and_self_delete_if_revoked` | Same IO start; refuses an unheld Organisation with `OrgNotOnChain` (LLR-379hnv), not `RevocationNotHeld`; returns `SelfDeleteOutcome` |
| `service.rs:961-979` `receive_one` | Binds the **first** Persona's endpoint and `recv_one`; maps `TransportError::Malformed` → `MalformedMessage` (LLR-xn5pwc), everything else to `Chain(String)`. Discards the authenticated sender |
| `service.rs:984-994` `verify_received` | `check_chain_free`, then `chain.read_state(..).await` (S2 moves the read), then verify |
| `service.rs:1043-1058` `ensure_endpoint` | **IO.** Binds at most once per service (PR-8qsnhx), from `find_persona(..).device_seed.signing_keypair()` |
| `service.rs:663-674` `commit_received` (private) | The held-update commit of a received message. **Name clash:** the first draft's public `commit_received` (§2.4) is renamed `commit_checked` to avoid it |
| `service.rs:249-264` `first_admission_base`, `encode_record_snapshot` | Pure. They stay |

### 1.3 org-node — storage file IO and the device seed (`org-node/src/store.rs`, `keys.rs`, `types.rs`; feature `app`)

| Site | What it does |
|---|---|
| `store.rs:3` `std::path::PathBuf`; `:397-401` `PersonaStore { path, key, data }` | The store owns its file path; one store holds every Persona |
| `store.rs:323-331` `StoreData { personas, orgs, provisional_updates, expected_admissions }`; `:374-394` `RawStoreData` | The sealed plaintext; no outstanding invites |
| `store.rs:58-68` `PersonaRecord.device_seed: DeviceSeed` (`:65`), raw mirror `:79` | **The device private seed is stored**; the keychain ruling (§13) moves it out |
| `store.rs:405-449` `StoreKey`, `derive_key` (Argon2, `:419-426`), `APP_SALT` (`:442`), `cipher` | Passphrase KDF: retired by §13 |
| `store.rs:451-465` `seal` | Pure AEAD with a fresh 24-byte nonce. It stays |
| `store.rs:467-479` `seal_for_test` (`test-support`) | **IO.** Writes sealed test plaintext to a path |
| `store.rs:486-507` `PersonaStore::open(path, passphrase)` | **IO.** `path.exists()`, `std::fs::read`, length check, decrypt, postcard `RawStoreData`, parse |
| `store.rs:518-523` `PersonaStore::save` | **IO.** postcard, `seal`, `std::fs::write` (not atomic, PR-n696wv) |
| `types.rs:52-56` `DeviceSeed` (`secret_type!`, `Clone`) ; `keys.rs:39-55` `DeviceSeed::signing_keypair`, `SigningKeypair::device_seed` | The cloneable seed type the never-expose ruling forbids org-io to hold |
| `service.rs` `self.store.save(rng)` at `:348, 398, 451, 479, 581, 627, 868, 882, 947, 953, 1020` | Every mutating operation writes the file before it returns (LLR-68yd3j, LLR-9zfnmb, LLR-7cmp38) |
| `service.rs` device-seed reads at `:507, 688-690, 826, 1048, 1109` | `persona_public_keys`, `still_member`, the receive path's own-Persona match, `ensure_endpoint`, `persona_keys` — each derives the Device public key from the stored seed; in S4 they read the stored public key |

### 1.4 org-node — chain IO (S2's work, listed for the boundary only)

`service.rs:36-174` (`ChainOps`, `MockChainOps`, `SubxtChainOps`),
`service.rs:182-216` `connect_chain_client`, `chain_read.rs`, `preflight.rs`,
`bin/preflight.rs` (which also builds a `DeviceSeed`, `:50`). The
`chain.read_state(..).await` calls at `service.rs:419, 597, 991` sit **inside**
the operations S4 makes value-driven (§2.4). S2 makes `commit_genesis` and
`commit_update` take the `OrgState` org-io read; S4 starts from that.

### 1.5 org-node — the invitation exchange

None. org-node holds only the joining side's *expectation*:
`expect_admission` (`service.rs:1010-1021`), `ExpectedAdmission { org_id }`
(`store.rs:365-368`, Organisation alone since the key-pair change), the
first-admission gate (`service.rs:790-793`), and `persona_public_keys`
(`service.rs:505-508`). The absences are pinned by
`org_node_holds_no_invitation_exchange` (`org-node/tests/absences.rs:57-79`)
and, new with the key-pair change, `org_node_holds_no_invite_identifier`
(`absences.rs:102-108`, LLR-ms8njy, LLR-48jakr: no `InviteId` and no
`invite_id` anywhere in `org-node/src`). S4's return of the Invite to
org-node must narrow the second (§3.2).

### 1.6 app (`app/src-tauri/src/`)

| Site | What it does |
|---|---|
| `invitation.rs:18-190` `InviteId`, `Invite`, `InviteReply`, `WireInvite`, `WireReply`, `armour`/`unarmour`, `parse`/`encode`, `wire_for_test` | Blob format (Base64 of postcard), parsed at the edge, naming the field |
| `invitation.rs:194-268` `OutstandingInvites` | **IO.** `outstanding_invites.json` (plain JSON hex pairs): `std::fs::read_to_string` (`:213`), `std::fs::write` (`:239`). Saved before `issue` and `settle` return |
| `invitation.rs:272-292` `issue_invite` | Collects the device keys of bound Personas, draws an invite id, persists it, encodes the Invite |
| `invitation.rs:298-332` `produce_reply` | Confirmation gate, refuses a bound Persona, encodes the reply, `svc.expect_admission(rng, org_id)` (store write) |
| `invitation.rs:336-342` `check_reply` | Parses and checks that the pair is outstanding |
| `invitation.rs:349-384` `admit_reply` | `admit_member`, then `submit_commit_send` to the reply's Device alone (LLR-q225ws), then `settle` once committed even if the send failed |
| `submit.rs:116-142` `submit_commit_send` | Chain write (S2), `commit_update`, then one `svc.send_update(&outcome.outgoing, recipient, peer_addr)` |
| `commands.rs:242-250` `parse_peer_addr` | Hex, then `postcard::from_bytes::<iroh::EndpointAddr>`. The app's only iroh use besides the type in signatures |
| `commands.rs:216-237, 263-312` `admit_member`, `revoke_member`, `revoke_and_send` | Parse the peer address; one recipient each — the reply's Device, or `device_keys.first()` of the revoked Member (LLR-q225ws, PR-3ue4va) |
| `commands.rs:369-415` `next_outcomes` | **Locks the service (`:374`), then awaits `receive_and_self_delete_if_revoked` (`:375`), which awaits `accept()` with the lock held** (PR-7v8anh) |
| `commands.rs:433-463` `start_receiver` | Spawns the loop; one-at-a-time guard |
| `events.rs:153-162` `TERMINAL_ERRORS`, `is_terminal_error` | Substring match on org-node's formatted iroh errors (`endpoint bind:`, `endpoint closed`) |
| `state.rs:99-104` `transport_mode_for` | Maps the app's mode name to org-node's `TransportMode` |
| `state.rs:117-158` `init`; `policy.rs:73` `resolve_passphrase` | Reads `ODS_PASSPHRASE` (REQ-7g3k9a, LLR-tpkmh3): retired by §13 |
| `state.rs:162-191` `assemble` | **IO.** `create_dir_all(data_dir)`, `PersonaStore::open(data_dir/persona_store.bin, passphrase)`, `set_transport_mode`, `OutstandingInvites::open(data_dir/outstanding_invites.json)` |
| `Cargo.toml` | `iroh = "0.98"`, `base64`, `postcard` (blobs and peer address), `org-node` with `features = ["app"]` |

## 2. The proposed split

### 2.1 Principle: org-node is sans-IO

org-node takes values in and gives values out. It has no endpoint, no path,
no `std::fs`, no `iroh`, and (once S2 moves the chain read) no `tokio` and no
`async`. org-io owns every socket, file and RPC, and calls org-node
synchronously between IO steps. The `absences` target gains a check that
`org-node/src` names none of `iroh`, `std::fs`, `tokio`, `PathBuf`.

### 2.2 Transport

**Stays in org-node** (pure):

- `WireMessage`, in the key-pair change's two-kind form (§8).
- `encode_message(&WireMessage) -> Result<Vec<u8>, OrgNodeError>` and
  `decode_message(&[u8]) -> Result<WireMessage, OrgNodeError>`. These are
  today's `encode_frame` **without** the length prefix, and `decode_body`.
  Both keep the 1 MiB bound: refuse before decoding; never panic.
- The outgoing builder, today's `send_update` minus the dial:
  `OrgService::outgoing_message(&OutgoingUpdate, recipient: DevicePublicKey)
  -> Result<Vec<u8>, OrgNodeError>`. It chooses the kind per recipient
  (key-pair LLR-6ymd6d) and writes nothing.
- The device identity org-io binds with. *Superseded 2026-10-06 by §14: org-io
  builds iroh's key itself from the keychain-held seed; there is no
  `EndpointSecret` and no `iroh-base` in org-node. Kept as history.* *Revised 2026-10-06 to the
  never-expose ruling.* The draft had `OrgService::device_seed(&PersonaId)
  -> DeviceSeed`, but org-node's `DeviceSeed` is `Clone` and gives its bytes
  up through a named accessor, so org-io would hold a device ed25519 private
  key in the sense REQ-v4tfap forbids. Instead org-node returns
  `OrgService::endpoint_secret(&PersonaId) -> EndpointSecret`, an opaque
  move-only type: no `Clone`, no `Copy`, a fixed-marker `Debug`, no
  `Display`, no `Serialize`, no accessor returning bytes, and exactly one
  consuming conversion, `EndpointSecret::into_endpoint_key(self) ->
  iroh_base::SecretKey`, written in org-node. That puts an `iroh-base`
  dependency (key types only, no IO, no async) on org-node; the
  conversion from seed bytes to iroh's key therefore happens inside
  org-node, and no line of org-io names the bytes. org-io calls
  `endpoint_secret(..).into_endpoint_key()` as the argument of iroh's
  endpoint builder, in one function, and nowhere else names
  `iroh::SecretKey`, calls its byte accessors (`to_bytes`, `secret`), clones
  it, or calls the endpoint's own secret-key accessor. A source scan in
  org-io's `absences` test pins all of that. The secret then lives inside
  the iroh endpoint (SOUP), which is unavoidable because the endpoint id is
  the Device key; org-io never hands the endpoint to the app. This is the
  one place a device private key passes through org-io, and it passes by
  move, opaque, into the SOUP that must hold it: org-io's REQ (§12,
  "device secret") states it, and its control RC (§12) carries it. REQ-v4tfap's
  "never receive, hold or output any device ed25519 private key" is read
  with that item; in develop, after S2 has merged, REQ-v4tfap gets a dated
  note naming it.
- The receive path takes bytes (§2.4).

**Moves to org-io:**

- `OrgEndpoint` (all of `transport/endpoint.rs`), `TransportMode`, `ALPN`.
  The ALPN string is unchanged, so S4 and pre-S4 nodes interoperate.
- Framing: the length prefix, the `MAX_FRAME + 4` read cap, and discarding
  the prefix unchecked.
- Endpoint lifecycle (`ensure_endpoint`, bind-once, the Persona whose device
  binds it).
- Dialling: Loopback by address with refusal when there is none, Networked by
  id.
- The receive loop, and `parse_peer_addr` (the app keeps passing a hex
  string; org-io parses it, so the app stops naming `iroh`).
- A typed transport error, so the app's terminal-error check no longer matches
  substrings (REQ-x3c8n2's expectation re-targeted to org-io).

**Byte-level interface:** one QUIC bi-stream per message, carrying
`len(u32 LE) ‖ body`. `body` is exactly what `encode_message` returns and what
`decode_message` takes. org-io hands org-node `(body: &[u8])`, and the
authenticated sender key is not passed: nothing about the sender is checked
(REQ-xa6smf, REQ-ztdza4). *(Superseded 2026-10-07 by S3a's member-sender
rule, owner ruling R1: org-io hands org-node `(sender: DevicePublicKey,
body: &[u8])`; §16.1, task T4.)* org-node hands org-io
`(recipient: DevicePublicKey, body: Vec<u8>)`. These are the on-wire bytes of
today, so the change needs no wire migration.

### 2.3 Storage

*Superseded in part on 2026-10-06 by §13–§15 (noted 2026-10-07):* the store
is one per Persona and keyed by an HKDF derivation of that Persona's
keychain-held device seed, not by a passphrase; `unseal` takes the sealed
bytes, the Device public key and a move-only `DeviceSecret`; a new store is
started only by `create_persona`; the store holds the Device public key, not
the seed. The design items are SDD-asxwp8 and SDD-fysk96 (org-node) and
SDD-egx8rv and SDD-4fvypt (org-io). The text below is the first draft, kept
as history; where it differs, those items win.

**Stays in org-node:** `StoreData`, its records and `Raw…` mirrors, the
Argon2 key, XChaCha20-Poly1305, and the nonce. `PersonaStore` loses its
`path` and gains:

- `PersonaStore::unseal(sealed: Option<&[u8]>, passphrase: &str)`, where
  `None` means a new store. Today's `open` minus `fs::read` and `exists`.
- `PersonaStore::seal(&self, rng) -> Result<SealedStore, OrgNodeError>`,
  where `SealedStore(Vec<u8>)` is `nonce(24) ‖ ciphertext`. Today's `save`
  minus `fs::write`.

**Moves to org-io:** the file (`persona_store.bin` in the data directory),
`create_dir_all`, read, and write. org-io stores the `SealedStore` bytes and
never reads inside them.

**Persistence obligation.** Every org-node operation that today calls
`store.save(rng)` instead returns, beside its result, a `#[must_use]
SealedStore` (or `None` when nothing changed). The org-io wrapper writes it
before it returns to the app. That keeps LLR-68yd3j's and LLR-9zfnmb's "the
store is written before it returns", re-stated at org-io's API, and makes
"forgot to persist" a compiler warning that the workspace lints deny. (Ruled
2026-10-06, ruling B: values in, values out; the injected-trait alternative
is rejected.)

*(2026-10-07: the owner-only mode below is dropped by owner ruling; the
file is created with default permissions and protected by its encryption
alone.)* **The write itself (ruled: atomic and owner-only, PR-n696wv).** org-io writes
the sealed bytes to a temporary file in the store's directory, created with
mode `0o600` (owner read and write only), syncs it, renames it over the
store file and syncs the directory. A crash leaves the old sealed store or
the new one, never a torn file. When the write fails org-io reports it and
drops the in-memory service state that was not persisted, re-unsealing from
the file on the next call, so memory never runs ahead of disk.

**Outstanding invites** move into `StoreData` as
`outstanding_invites: Vec<(OrgId, InviteId)>`. They are then sealed with
everything else, and org-io stores one opaque blob, as the ruling requires.
`outstanding_invites.json` is retired with no migration (ruled 2026-10-06):
org-io never reads it, and an invite outstanding in it at upgrade is lost,
so its reply is refused and the inviter issues a new Invite.

### 2.4 The receive path, byte-driven

Today a receive operation does IO three times in one async call: recv, a
chain read in the middle, and a store write at the end. In S4 the order
becomes:

1. org-io receives the bytes (no service lock held, which fixes F1).
2. org-io calls `OrgService::check_received(&[u8]) -> Result<Received,
   OrgNodeError>`. This decodes and runs the chain-free checks (LLR-9ew26y),
   including the first-admission expectation gate (LLR-s8xp7m, RC-2ferct).
3. org-io reads the chain for `received.org_id()` (S2's reader).
4. org-io calls `OrgService::commit_checked(checked, OrgState) ->
   Result<(ReceiveOutcome, Option<StoreWrite>), OrgNodeError>`. *(Renamed
   2026-10-07 from `commit_received`, which since the key-pair change names a
   private helper at `service.rs:663`; `Received` becomes `CheckedMessage` and
   `SealedStore` becomes `StoreWrite`, per SDD-esg2kc and SDD-asxwp8.)*
5. org-io writes the sealed store.

`commit_update` and `commit_genesis` split the same way:
`commit_update(org_id, OrgState)`.

The chain read happening only after the chain-free checks is now a property
of org-io's sequencing plus the fact that `Received` can only be obtained from
`check_received`. That is a two-unit statement of RC-mj6gjq and RC-2ferct
(HAZ-bedm57). **This split is shared with S2**, which owns the chain read
(§10).

**One receive path (PR-kwwap5).** org-io's loop sends every message through
the full path, today's `receive_and_verify` (first admission, update, or the
node's own removal; LLR-b27jr6). The app's loop calling only
`receive_and_self_delete_if_revoked` is PR-kwwap5's defect, so this fixes it.
The self-delete entry point is retired (ruled 2026-10-06, D5). Its revocation
behaviour that differs from `receive_and_verify`'s (key-pair LLR-38e2kn,
LLR-pt32fx) keeps the `receive_and_verify` branch.

**The receive loop holds no lock while it waits (ruled, PR-7v8anh).**
org-io's loop awaits the inbound connection and reads the stream with no
lock held on the service or the store. It takes the service only for the
synchronous `check_received` call, releases it for the chain read, and takes
it again for `commit_received` and the store write. Every other command can
run while the loop waits.

### 2.5 Invitation exchange

**Moves to org-node** (build and serialise):

- `Invite`, `InviteReply`, and the Blob format (Base64 of postcard).
- `Invite::parse` and `InviteReply::parse`, naming the field (today's app
  LLR-b7wgpf).
- `OrgService::build_invite(org_id, org_name, invitee_name, rng) -> (Invite,
  SealedStore)`. It collects the device keys of bound Personas, draws the
  invite id and records it as outstanding.
- `build_reply(invite, persona_id) -> (InviteReply, SealedStore)`. It refuses
  a bound Persona (LLR-rt8gdz) and records the expectation.
- `check_reply(&InviteReply) -> Result<..>`, against outstanding pairs.
- `settle(org_id, invite_id)`.

All of these are pure over `StoreData`.

**Moves to org-io** (runs the exchange): the operations the app calls,
`issue_invite`, `produce_reply(confirmed)`, `import_reply` and
`admit_reply`. `admit_reply` is the sequence `admit_member` → chain write
(S2) → `commit_update` → send to each recipient → `settle` → persist. It
settles once the admission has committed, even when the send fails. org-io
writes the store after each step that changes it.

**Fan-out (ruled: in S4, Networked mode; PR-3ue4va).** org-node returns, for
a committed update, the recipients and the bytes for each: an
`OrgInformation` message for every Device the committed record lists other
than the sender's own, and a `Revocation` message for every Device the update
removed (key-pair LLR-6ymd6d chooses the kind). In Networked mode org-io
dials each by its Device key, tries every one whatever happened to the
others, and reports the outcome per Device. In Loopback mode, which has no
address book, org-io keeps the single addressed recipient, and refuses an
address whose endpoint id is not that recipient's Device key, which fixes
PR-2dmjzj.

**Endpoints (ruled: one per Persona; PR-8qsnhx).** org-io binds at most one
endpoint per Persona, keyed by `PersonaId`, from that Persona's
`EndpointSecret` (§2.2). A send is made from the endpoint of the Persona
bound to the Organisation the message is about, and the receive loop accepts
on every bound endpoint.

**Stays in the app:** the panels, the copy-paste text fields, the
confirmation dialog and `REPLY_WARNING` (REQ-ab2mfz, LLR-n2u4uf,
RC-wzb48r), and the decision to call each operation. The app passes
`confirmed` through.

### 2.6 Resulting dependency edges

- org-node loses `iroh`, `tokio` (with S2), `async-trait` (with S2), and the
  `transport` and `test-support → iroh/test-utils` features. It gains
  `base64` back (invitation Blobs). *(Superseded by §14: no `iroh-base`.)* It was to gain `iroh-base` (key types only, no IO,
  no async) for the one conversion `EndpointSecret::into_endpoint_key`
  (§2.2). The `iroh-base` version is the one `iroh` 0.98 depends on; it is a
  new SOUP row in org-node, and the absence scan forbids `iroh::` (not
  `iroh_base::SecretKey`) in `org-node/src`.
- org-io gains `iroh` 0.98 (pinned), `iroh/test-utils` (dev and
  `test-support`), `tokio`, and `ed25519-dalek` or iroh's key type for the
  remote-key conversion.
- The app loses `iroh`, `base64`, and `postcard` (peer address and Blobs).
  `app → org-node` goes away if S2 makes org-io re-export the value types.

## 3. Items S4 must move, amend or retire

"Move" means the item leaves org-node's or the app's ledger for org-io's,
keeping its ID (ruling A, 2026-10-06): the definition moves with a dated
"moved from" note, and the source ledger keeps a prose pointer to org-io's
ledger file without naming the ID (ADR `docs/adr/2026-10-06-org-io-unit.md`).
The moves are done in the develop phase, with the code; this section is
their list. "Amend" means it stays and gets a dated note.
"Retire" means it is superseded in place. Paths are relative to the unit's
`docs/`. *Re-surveyed 2026-10-07 at `f51de1a`: every file:line below is the
item's definition line after the key-pair merge, and the items the key-pair
change added that S4 touches are in the tables (marked "key-pair").*

### 3.1 org-node — move to org-io

| ID | file:line | Why |
|---|---|---|
| REQ-db6s7q | requirements/2026-10-03-transport-binding-scope.md:25 | Loopback binding scope: the endpoint is org-io's |
| REQ-2wzfzv | requirements/2026-10-03-transport-binding-scope.md:86 | IPv4 required / IPv6 optional |
| SDD-8uyg4s | architecture/2026-10-03-decomposition.md:718 | The authenticated endpoint |
| LLR-ygn78w | decomposition.md:724 | EndpointId = Device key |
| LLR-v873fx | decomposition.md:728 | `recv_one` sender key from the handshake |
| LLR-wwunf4 | decomposition.md:747 | Loopback binds and advertises loopback only |
| LLR-6adc99 | decomposition.md:751 | IPv4 loopback required |
| LLR-wx77j5 | decomposition.md:756 | `node_addr_for_dial` |
| LLR-k2y6nn | decomposition.md:761 | Stream read cap `MAX_FRAME + 4` |
| LLR-2smrvx | decomposition.md:765 | Length prefix discarded |
| LLR-fa7jt8 | decomposition.md:693 | Length-prefix framing |
| SDD-b8tuv3 | decomposition.md:2412 | Endpoint lifecycle |
| LLR-6zjzn2 | decomposition.md:2416 | Bind once (states PR-8qsnhx); rewritten to one per Persona (LLR-d2pzzk) |
| LLR-cns6q6 | decomposition.md:2422 | Bound from the named Persona's device seed; rewritten: from the keychain item (LLR-d5y7ks) |
| LLR-ecz9a6 | decomposition.md:2435 | Service in Loopback binds loopback |
| LLR-jn5jeh | decomposition.md:1426 | Loopback dials the full address (states PR-2dmjzj); key-pair amended |
| LLR-pw369n | decomposition.md:2322 | Loopback send without an address refused |
| PR-8qsnhx (open) | problems/2026-10-04-endpoint-bound-once.md:6 | Endpoint bound once. Resolved in org-io by one endpoint per Persona (ruled) |
| PR-2dmjzj (open) | problems/2026-09-09-org-node-problems.md:167 | Loopback sends to an unchecked address. Resolved in org-io by refusing an address whose endpoint id is not the recipient's Device key (REQ-8trkps) |
| PR-n696wv (open, this branch) | org-io/docs/problems/2026-10-13-non-atomic-store-write.md:8 | Store writes not atomic. Resolved in org-io (ruled: atomic and owner-only) |

### 3.2 org-node — amend

| ID | file:line | Amendment |
|---|---|---|
| REQ-eg5j8u | requirements/2026-09-09-verify-and-commit.md:101 | "wire frame" becomes "Wire message body". The stream bound becomes org-io's (REQ-szt6ea) |
| RC-gfn6kr | risk/2026-09-09-org-node-hazards.md:355 | Same rewording; org-io's stream cap added as a second realisation |
| HAZ-5f9jcm | risk/2026-09-09-org-node-hazards.md:340 | The "QUIC-reachable decoder" pathway now crosses org-io first; the probability argument is unchanged |
| SDD-kwncn7 | decomposition.md:656 | "framing on the channel" becomes "the Wire message's encoding and bound". Code list `transport/wire.rs` becomes `wire.rs` (outside `transport/`) |
| LLR-er2x8n, LLR-sc6zuh, LLR-8kh3zf, LLR-pkruy8 | decomposition.md:697, 702, 706, 710 | `encode_frame`/`decode_body` renamed to `encode_message`/`decode_message`. No length prefix; the error is `OrgNodeError` |
| LLR-js9dsu, LLR-ecxc76 (key-pair) | architecture/2026-10-07-org-key-pair.md:101, 113 | Unchanged in substance; code reference moves from `transport/wire.rs` to `wire.rs` |
| SDD-rx2yvy | decomposition.md:1304 | `send_update` goes; the commit returns the bodies (SDD-bm27xt, LLR-gdfqn9) |
| LLR-6ymd6d (key-pair) | architecture/2026-10-07-org-key-pair.md:181 | "`send_update(outgoing, recipient, peer_addr)` … sends" becomes "the commit builds, for each recipient, …" with the key captured **at commit** (resolves PR-gnh3j2, §8.1) |
| PR-gnh3j2 (key-pair, open) | problems/2026-10-07-review-findings.md:7 | The key the send carries is read when `send_update` runs, not at the commit. Resolved by LLR-gdfqn9 (bodies built inside the commit) |
| LLR-2xzys9 | architecture/2026-10-06-chain-authority.md:385 | The sending Persona's device is org-io's binding (LLR-d2pzzk); org-node only names the Persona bound to the Organisation |
| LLR-8hdu9x, LLR-bg3vsw | decomposition.md:2361, 1409 | "the Wire message `send_update` sends" becomes "the body the commit returns for that recipient" |
| LLR-t4znbk | decomposition.md:1462 | Splits: building the bodies writes nothing (org-node); a failed send leaves the committed record (org-io, LLR-znhmh2) |
| LLR-48jakr (key-pair) | architecture/2026-10-06-chain-authority.md:402 | "`send_update` takes no invite identifier" becomes "no outgoing body carries one" |
| LLR-ms8njy (key-pair) | architecture/2026-10-06-chain-authority.md:173 | **Narrowed:** no Wire message and no expectation carries an invite identifier; the clause "org-node defines no `InviteId` type" is withdrawn, because the Invite returns to org-node (owner ruling 1 on the sweep). `org_node_holds_no_invite_identifier` (`tests/absences.rs:102-108`) is narrowed to `wire.rs` and `ExpectedAdmission` |
| LLR-4tcxsu | architecture/2026-10-06-chain-authority.md:494 | "bind no endpoint and send nothing" becomes an absence (org-node has no endpoint) |
| LLR-qezw3n | decomposition.md:1556 | Unchanged claim; the evidence moves to the extended absence test |
| LLR-3q63zv | decomposition.md:2191 | The sender key never reaches org-node; the "no check" holds by construction |
| LLR-xn5pwc (key-pair) | architecture/2026-10-07-org-key-pair.md:229 | "`receive_one` returns `MalformedMessage` when `recv_one` refuses …" becomes "`check_received` returns `MalformedMessage` when `decode_message` refuses …" |
| LLR-ba2ejp (key-pair) | architecture/2026-10-07-org-key-pair.md:238 | "on both receive paths" becomes "in `commit_checked`" |
| LLR-38e2kn, LLR-pt32fx (key-pair) | architecture/2026-10-07-org-key-pair.md:250, 260 | The two paths become one: `check_received` refuses an unheld revocation (`RevocationNotHeld`), `commit_checked` refuses one that leaves the Device listed |
| LLR-379hnv | decomposition.md:2218 | **Retired** with the self-delete path: an unheld Organisation is refused as LLR-38e2kn states (`RevocationNotHeld` for a revocation; a first admission otherwise) |
| PR-q8r32t (key-pair, open) | problems/2026-10-07-review-findings.md:25 | The unreachable `RevocationNotHeld` arms go with the split (LLR-zetn7a); resolved |
| SDD-8cpyfa, SDD-72ddm6 | decomposition.md:1641, 2111 | Interfaces take bytes plus `OrgState` (§2.4); code lists drop `receive_one` |
| LLR-9ew26y, LLR-s8xp7m | chain-authority.md:418, 427 | Restated as `check_received` before `commit_checked` (shared with S2) |
| LLR-b27jr6, LLR-eyc4ud | chain-authority.md:509, 517 | "`receive_and_verify`" becomes "`commit_checked`"; the self-delete path is gone |
| RC-mj6gjq, RC-2ferct, HAZ-bedm57 | risk/2026-10-06-chain-authority.md:49, 55, 39 | Realisation spans org-node and org-io sequencing (shared with S2) |
| SDD-af5vnt | decomposition.md:781 | "the form the file takes" becomes "the sealed form"; one store per Persona; outstanding invites join `StoreData`; the Device public key replaces the seed |
| LLR-wusj89, LLR-t4u66w, LLR-q5n28x, LLR-s78sh7 | decomposition.md:850, 858, 862, 866 | "store file" becomes "sealed store"; "passphrase" becomes "device secret"; the device seed leaves the list |
| LLR-8mfjey | decomposition.md:854 | "on every save" becomes "on every seal" |
| LLR-scgk5j | architecture/2026-10-04-type-safety.md:297 | `StoreKey` is now HKDF output (LLR-2qqqx6); traits unchanged |
| LLR-tev8h8, LLR-ctzkv7 | decomposition.md:1178, 280 | `create_persona` returns the device seed as a `DeviceSecret` and keeps the public key |
| LLR-sz4xhc, LLR-56hc77 | type-safety.md:65, 217 | `DeviceSeed` becomes the move-only `DeviceSecret` (LLR-37k2s3) |
| REQ-hzm4kt | requirements/2026-09-09-verify-and-commit.md:139 | "write the Persona store only as ciphertext" becomes "give the Persona store out only as ciphertext", keyed from the device secret |
| REQ-y7tsft | requirements/2026-10-04-type-safety.md:39 | Unchanged in substance; it covers `DeviceSecret` |
| HAZ-45ucqx, RC-jjsz97 | risk/2026-09-09-org-node-hazards.md:628, 638 | Re-assessed: the passphrase and fixed-salt residuals go; no file-permission control (dropped by owner ruling 2026-10-07); the keychain-derived key (org-io RC-d8dbxs) protects the file |
| REQ-qn2erx | requirements/2026-10-04-type-safety.md:63 | "open a Persona store" becomes "unseal"; the Invite/reply parse returns |
| LLR-8bum44 | type-safety.md:342 | Covers `unseal` and the Invite/reply parse |
| LLR-68yd3j, LLR-9zfnmb, LLR-7cmp38 | decomposition.md:1198, 1577; chain-authority.md:268 | "writes the store before it returns" becomes "returns the store write"; org-io writes it |
| LLR-95753m | chain-authority.md:191 | `StoreData` gains `outstanding_invites`; one Persona per store |
| LLR-ayrdr8 | type-safety.md:117 | Golden plaintext changes (single Persona, Device public key, outstanding invites); re-pinned |
| `soup.md` rows `iroh`, `ed25519-dalek`, `argon2`, `postcard`, `serde`, `base64`, `tokio`, `async-trait`, dev `iroh (test-utils)`, checklist item 3 | architecture/soup.md:132, 129, 134, 135, 136, 137, 144, 145, 172, 186 | §4; `hkdf` and `sha2` added |
| `org-node/AGENTS.md:12-15` "edges" paragraph | AGENTS.md | The edges become Wire message decode, unseal and Blob parse. "The invitation Blobs are the app's edge" is reversed |
| Chain-authority "Sequencing", change 2 | requirements/2026-10-06-chain-authority.md:233-234 | "iroh moves to the app" becomes "superseded: roadmap S4, org-io" |

### 3.3 org-node — retire or rewrite (the invitation boundary returns)

| ID | file:line | Action |
|---|---|---|
| SDD-vee2fq | decomposition.md:878 | Rewritten: "the out-of-band Invite and Invite reply, built and serialised here; exchanged by org-io". Code moves to `invitation.rs` in org-node (SDD-xshv5b holds the build and bookkeeping) |
| LLR-g9vmbx | decomposition.md:907 | Retired (org-node defines an Invite again). Its replacement is the moved LLR-b7wgpf |
| LLR-8qxwst | decomposition.md:940 | Rewritten: armoured text reaches org-node only through `Invite::parse` and `InviteReply::parse` |
| LLR-zj88e6 | decomposition.md:1544 | Rewritten: `build_invite` carries the bound Personas' device keys (LLR-nr8df3) |
| LLR-836z24, LLR-kkj64b, LLR-tcft2r | decomposition.md:1623, 934, 945 | Unchanged |

### 3.4 app — move to org-node

| ID | file:line | Why |
|---|---|---|
| LLR-b7wgpf | architecture/2026-10-06-invitation.md:32 | Blob parse, naming the field |
| REQ-yazum3 | requirements/2026-10-06-invitation.md:67 | Refuse an unparsable Invite or reply |

### 3.5 app — move to org-io

| ID | file:line | Why |
|---|---|---|
| REQ-prjja8 | requirements/2026-10-06-invitation.md:19 | Produce an Invite; keep the id outstanding |
| REQ-tcutr6 | invitation.md:33 | Produce a reply; declare the expectation (key-pair amended: Organisation only) |
| REQ-65xqp8 | invitation.md:47 | Refuse a reply not outstanding; settle |
| LLR-9sraks | architecture/2026-10-06-invitation.md:53 | `issue_invite` |
| LLR-f35pda | invitation.md:67 | Outstanding pairs. **Rewritten**: they live in the sealed store (REQ-pnr6kh) |
| LLR-w4mhd4 | invitation.md:96 | `produce_reply` (confirmation gate) |
| LLR-rt8gdz | invitation.md:118 | A bound Persona is refused (the build half goes to org-node, LLR-7q9g9z) |
| LLR-gha5f6 | invitation.md:145 | `check_reply` / `admit_reply` |
| LLR-q225ws (key-pair) | architecture/2026-10-07-org-key-pair.md:21 | One send per command, to one Device. **Retired** by the fan-out (LLR-znhmh2), which PR-3ue4va asks for; its tests in `tests/submit_flow.rs` are rewritten |
| LLR-ty85xv, LLR-n6twt7, LLR-ctrfz4 | architecture/2026-10-05-decomposition.md:294, 305, 351 | Peer-address parse (`iroh::EndpointAddr`); parented to LLR-2b2ghe's SDD |
| LLR-85zque | decomposition.md:119 | Create the data dir and open the store in it; rewritten per Persona (LLR-uzax55) |
| PR-kwwap5 (open) | problems/2026-10-06-first-admission.md:3 | Resolved by S4 (§2.4) |
| PR-7v8anh (open, this branch) | app/docs/problems/2026-10-06-receiver-holds-lock.md:7 | Resolved by org-io's lock-free receive loop (ruled) |
| PR-3ue4va (key-pair, open) | problems/2026-10-07-update-fan-out.md:3 | Re-homed to org-io and resolved by the fan-out. Its text "the `org-io` session's work, which moves peer communication into the app" is stale (F3): corrected with a dated note when it moves |

### 3.6 app — amend (stays in the app)

| ID | file:line | Amendment |
|---|---|---|
| REQ-nfr3n2 | requirements/2026-10-06-invitation.md:72 | Split three ways: the app decides; org-io writes (S2), commits and sends to every recipient (S4) |
| LLR-qhjp6g, LLR-be3zv9 | architecture/2026-10-06-invitation.md:192, 207 | S2 moves the write. S4 moves "then send" |
| LLR-8krgzj (key-pair) | architecture/2026-10-05-decomposition.md:321 | `admit_member` still takes no secret or key; it calls org-io |
| LLR-vzf8j2 | decomposition.md:238 | Parse order: org id in the app; the peer address in org-io |
| SDD-aq7m6b, LLR-vqkr5t | decomposition.md:110, 132 | `assemble` hands the data dir to org-io; it still reports the dir; no passphrase |
| SDD-mwqf6x, LLR-7bk6qh, LLR-a9rjtf | decomposition.md:525, 536, 598 | Receive errors arrive as org-io's typed error; `TERMINAL_ERRORS` becomes a match on its variant; the outcome kinds come from `ReceiveOutcome`'s kind |
| SDD-rmbr3t | decomposition.md:403 | The twelve commands lock org-io's handle, not the service; the receiver takes no lock while it waits |
| REQ-x3c8n2 | requirements/2026-09-14-tauri-shell.md:391 | The expectation is re-targeted from org-node to org-io (REQ-8e57wm) |
| REQ-645jq9, RC-8ygnjd, LLR-j5pacp | tauri-shell.md:35; risk/2026-09-14-app-hazards.md:420; decomposition.md:96 | The mode name is still the app's decision. `transport_mode_for` maps to org-io's type |
| REQ-7g3k9a, RC-xt4qr3, LLR-tpkmh3 | tauri-shell.md:336; app-hazards.md:667; decomposition.md:50 | Passphrase prompt retired: "fail to start, naming the condition, when the keychain refuses" (org-io's REQ-6d9xv6) |
| REQ-vgr7s2, RC-jrkn7w, HAZ-n97v5g, PR-7zz76t | tauri-shell.md:114; app-hazards.md:425, 373; problems/2026-10-05-problems.md:139 | Unchanged in substance. PR-7zz76t's premise (a Join request carrying an address) is gone, so it may close as obsolete |
| `app/docs/architecture/soup.md` rows `iroh`, `postcard`, `base64` | soup.md:96, 97, 93 | `iroh` removed. `postcard` and `base64` lose the peer address and Blob uses |
| `app/AGENTS.md` / `org-node/AGENTS.md` mention of `app/src-tauri/src/invitation.rs` as the Blob edge | — | Reversed |

### 3.7 Counts

- **Move to org-io:** 20 org-node items (2 REQ, 2 SDD, 13 LLR, 3 PR) and 16
  app items (3 REQ; 10 LLR, among them key-pair's LLR-q225ws, which is
  retired there in favour of the fan-out; PR-kwwap5, PR-7v8anh and key-pair's
  PR-3ue4va, each resolved there).
- **Move app → org-node:** 2 (LLR-b7wgpf, REQ-yazum3).
- **Amend in org-node:** about 55 items (the key-pair merge added LLR-6ymd6d,
  LLR-48jakr, LLR-ms8njy, LLR-xn5pwc, LLR-ba2ejp, LLR-38e2kn, LLR-pt32fx,
  LLR-js9dsu, LLR-ecxc76, and the keychain rulings LLR-scgk5j, LLR-tev8h8,
  LLR-ctzkv7, LLR-sz4xhc, LLR-56hc77, REQ-y7tsft), plus SOUP and AGENTS text.
- **Amend in the app:** about 22.
- **Retire or rewrite in org-node:** 5 (SDD-vee2fq, LLR-g9vmbx, LLR-8qxwst,
  LLR-zj88e6, LLR-379hnv).
- **Resolved by S4:** PR-kwwap5, PR-7v8anh, PR-n696wv, PR-8qsnhx, PR-2dmjzj,
  PR-3ue4va, PR-q8r32t and PR-gnh3j2 (the last two key-pair reports, §8.1);
  possibly PR-7zz76t (obsolete).

## 4. SOUP move

*Reconciled 2026-10-07: line numbers at `f51de1a`; the keychain rulings of
§13 add `keyring` (org-io) and `hkdf`/`sha2` (org-node) and remove
`argon2`.*

- **iroh 0.98.2** (`org-node/docs/architecture/soup.md:132`) moves with
  its whole risk cell: the PR-d4nye8 default-wildcard-bind note,
  `RelayMode::Disabled` load-bearing and unverified, no relay-map accessor,
  the `ed25519-dalek` 3.0.0-pre.6 pre-release, and `test-utils` bringing
  `insecure_skip_verify` into `test-support`. It goes into
  `org-io/docs/architecture/soup.md`, **pinned 0.98**. The dev row
  (`:172`) and checklist item 3 (`:186`, "an iroh minor bump re-opens
  PR-d4nye8") move too. The app's iroh row (`app/docs/architecture/soup.md:97`)
  is deleted.
- **keyring** (new, org-io): the OS keychain (macOS Keychain Services,
  Windows Credential Manager, Linux Secret Service over D-Bus). Assessed in
  task T7 (the version is the current 3.x at that date, pinned exactly;
  features `apple-native`, `windows-native`, `sync-secret-service`; its
  `mock` credential builder only under org-io's `test-support`, and only if
  T7 finds that it shares one item between `Entry` handles; otherwise org-io's
  own in-memory stand-in, ruled 2026-10-07). Risk cell:
  a backend that silently stores in plaintext (keyring's `mock` or a
  file-backed fallback in a release build) would defeat RC-d8dbxs; the
  absence scan (LLR-36jer8) pins that only `test-support` names the mock.
- `ed25519-dalek` (`:129`): the endpoint-identity conversion
  (`VerifyingKey::from_bytes` at `endpoint.rs:323`) moves to org-io. org-node
  keeps key generation, the `Debug` reliance for LLR-bwb9pu, and the
  public-key derivation that `DeviceSecret` offers (LLR-37k2s3).
- `argon2` (`:134`): **removed** from org-node (§13; the store key is HKDF).
- `hkdf` and `sha2` (new, org-node): the store-key derivation (LLR-2qqqx6).
- `base64` (`:137`): returns to org-node (Invite Blobs) and leaves the app.
- `postcard` and `serde` (`:135-136`): the users list gains org-node's
  `invitation.rs` and `wire.rs` (renamed from `transport/wire.rs`).
- `tokio` and `async-trait` (`:144-145`): leave org-node once S2 and S4 are
  both in (sans-IO).
- `zeroize` (org-io, new row): the seed buffer of LLR-d5y7ks and LLR-677fqq.
- The dependency-edge assessments needed: org-io → iroh and org-io →
  keyring (SOUP, T7), org-io → org-node (unit edge, created by S2), and
  app → org-io (S2). `.guardrails/units.yaml` has `org-io` already.

## 5. Test migration

*Recounted 2026-10-07 at `f51de1a`.* The key-pair change rewrote most of
org-node's receive tests and added `receive_chain_reads.rs` (9),
`expected_admission.rs` (8), `organisation_key.rs` (11) and
`fuzz_wire_decode`. Counts below are `#[test]`/`#[tokio::test]` functions.

**Move to org-io (networked and endpoint tests):**

- `org-node/tests/transport_handshake.rs`, all 6 tests:
  `delivers_and_verifies_admit_over_iroh` (LLR-v873fx),
  `loopback_mode_binds_and_advertises_loopback_only` (REQ-db6s7q),
  `loopback_mode_holds_under_repeated_and_colliding_binds` (REQ-db6s7q,
  REQ-2wzfzv), `endpoint_id_equals_device_key` (LLR-ygn78w),
  `a_stream_longer_than_the_read_bound_is_refused_rather_than_buffered`
  (LLR-k2y6nn), `the_length_prefix_is_not_checked_against_the_body`
  (LLR-2smrvx).
- `org-node/tests/transport_networked.rs`, 1 test:
  `delivers_and_verifies_admit_over_relay_by_id` (needs `bind_with_relay` in
  org-io's `test-support`).
- `service_lifecycle.rs`, 3 of 9 tests:
  `the_endpoint_is_bound_once_and_the_same_one_is_returned_after`
  (LLR-6zjzn2, rewritten to one per Persona),
  `the_endpoint_binds_the_named_personas_device_not_the_first_personas`
  (LLR-cns6q6), `a_service_in_loopback_mode_binds_loopback_sockets_only`
  (LLR-ecz9a6).
- `admission_sender.rs`, 5 of 52 dial and send tests:
  `in_loopback_mode_the_joiner_is_dialled_at_the_full_address` (LLR-jn5jeh),
  `a_loopback_revocation_without_an_address_burns_no_epoch` (LLR-pw369n),
  `pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key`
  (PR-8qsnhx; inverted when it is fixed), `a_failed_send_leaves_the_committed_record_in_place`
  and `a_failed_revocation_send_leaves_the_committed_removal_in_place`
  (LLR-t4znbk, org-io half).
- `commit_paths.rs`, 2 of 32 (the dial and refusal halves):
  `send_update_refuses_without_a_bound_persona_or_a_loopback_address`, and
  the dial half of
  `send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other`
  (its kind half stays in org-node, over the commit's bodies).
  `send_update_refuses_an_organisation_it_holds_no_record_of` becomes an
  org-node test of the commit (no record, no bodies).
- `wire_frame_bound.rs`: `frame_round_trips` (LLR-fa7jt8), the prefix half.
- `service_stories.rs`: `five_stories_full_e2e` is **duplicated** as org-io's
  end-to-end test over two loopback endpoints and the mock chain; the
  org-node copy is rewritten value-driven.
- app `tests/invitation.rs`: 14 of 17 tests move (LLR-9sraks, LLR-f35pda,
  LLR-w4mhd4, LLR-rt8gdz, LLR-gha5f6). The 3 LLR-b7wgpf parse tests move to
  org-node.
- app `tests/state_assembly.rs`: 3 tests (LLR-85zque), rewritten per
  Persona and over the keychain mock.
- app `tests/ipc.rs`: the peer-address tests (LLR-ty85xv, LLR-n6twt7,
  LLR-ctrfz4) become org-io unit tests of `parse_peer_addr`. The IPC-level
  assertion that the command surfaces the error stays.
- app `tests/submit_flow.rs`, 7 tests: S2 moves the file whole; S4 rewrites
  the two LLR-q225ws tests (`each_command_sends_once_to_one_device_and_the_kind_follows_the_record`,
  `a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing`)
  as fan-out tests (LLR-znhmh2).

That is about **44 tests** moving to org-io, 5 of them split or duplicated,
plus 3 moving from the app to org-node.

**Stay in org-node, rewritten value-driven (no iroh).** 81 tests at
`f51de1a` drive the transport through `tests/support/mod.rs`'s helpers
(`spawn_receive` `:109`, `spawn_self_delete` `:132`, `spawn_recv_one`
`:161`, `dead_addr` `:100`, `setup_over` `:352`, `captured_admission`
`:418`, `deliver` `:432`, `deliver_raw` `:439`, `revoke_and_tell` `:262`):
`admission_sender.rs` 47 (52 less the 5 moved), `receive_chain_reads.rs` 9,
`expected_admission.rs` 5, `commit_paths.rs` 5 (7 less the 2 moved),
`service_stories.rs` 3, `organisation_key.rs` 1. Of those, 21 drive the
self-delete path (`spawn_self_delete`: `admission_sender.rs` 15,
`receive_chain_reads.rs` 3, `service_stories.rs` 3); its retirement folds
them onto the one path, and a test that pinned a difference between the two
paths (`…_on_both_paths`, LLR-379hnv's `OrgNotOnChain`) keeps only
`receive_and_verify`'s behaviour. The helpers are replaced: "send" becomes the commit's bodies,
"receive" becomes `check_received` then `commit_checked` against the mock
chain's state. This removes iroh's latency and the PR-d4nye8 class of
flakiness from org-node's gate.

**Stay in org-node, store tests rewritten from path to sealed bytes:** 34
`PersonaStore::open` call sites across 7 files (`store_at_rest.rs` 10,
`persona_records.rs` 12, `organisation_key.rs` 3, `service_lifecycle.rs` 3,
`commit_paths.rs` 2, `provisional_store.rs` 2, `tests/support/mod.rs` 2;
`open_store`/`reopen_store`/`store_bytes` at `support/mod.rs:52-68` carry
most callers). The `.personas` uses in 9 test files move to the
per-Persona shape. "Reaches the disk" assertions become "reaches the
returned store write". org-io adds a file round-trip test and a test that
the file holds exactly the sealed bytes.

**Fuzz:** `fuzz_envelope_decode`, `fuzz_first_admission_base`,
`fuzz_verify_against_chain` and the key-pair change's `fuzz_wire_decode`
stay (the last re-targeted at `decode_message`, so no new wire target). New
(AGENTS.md: always fuzz):

- org-node: `fuzz_invitation_parse` over `Invite::parse` and
  `InviteReply::parse`; `fuzz_unseal` over `PersonaStore::unseal` on
  arbitrary bytes (never panics, refuses with a typed error).
- org-io: `fuzz_peer_addr` over `parse_peer_addr` on arbitrary text.

`wire_frame_bound.rs` keeps 6 of its 7 tests in org-node, under the renamed
functions.

## 6. Proposed requirements for org-io (prose, no IDs)

*Superseded 2026-10-06 by §12, which lists the minted items as drafted in
org-io's and org-node's ledgers. The prose below is kept as the survey's
proposal; where §12 differs (the device secret, the address check, the
expectations on org-node), §12 wins.* org-io is class C (ruled 2026-10-06).

**Transport (moved substance).** org-io carries the moved REQ-db6s7q and
REQ-2wzfzv unchanged, and adds:

1. The software shall bind its endpoint with the Device key of the Persona it
   acts for as the endpoint identity, so that a completed handshake proves
   custody of that Device key.
2. The software shall read at most 1 MiB plus the four-byte length prefix
   from one inbound stream, and shall refuse a longer stream with a typed
   error without buffering beyond that bound.
3. The software shall hand org-node exactly the body bytes of a received
   message and nothing derived from the connection, and shall not refuse,
   reorder or alter a message because of the Device key the connection
   authenticated.
4. The software shall, in Loopback mode, refuse with a typed error and send
   nothing when asked to send to a recipient for which it holds no address;
   and in Networked mode shall dial the recipient by the endpoint identity
   equal to its Device key.
5. The software shall bind at most one endpoint per Persona and shall send
   each message from the endpoint of a Persona bound to the Organisation the
   message is about. (This replaces bind-once; it cures PR-8qsnhx's sending
   side if the owner accepts D8.)
6. The software shall report a transport failure that ends the receive loop
   (endpoint closed, bind failure) as a typed condition distinct from every
   other receive failure.
7. The software shall not hold the node's state lock while waiting for an
   inbound connection or message (finding F1).

**Receive and commit sequencing.**

8. The software shall read the chain for a received message only after
   org-node's chain-free checks have accepted it, and at most once per
   message (with S2).
9. The software shall pass every received message to org-node's single
   receive path, so that a first admission the app declared it expects is
   committed (PR-kwwap5).

**Storage.**

10. The software shall write to local storage only bytes org-node returned
    as a sealed store, unmodified, and shall interpret none of them.
11. The software shall write the sealed store an operation returned before
    reporting that operation's success to its caller. When the write fails it
    shall report the failure and shall not report success, and it shall
    discard the in-memory state that was not persisted.
12. The software shall replace the store file atomically, so that an
    interrupted write leaves the previous sealed store or the new one in
    place, never a mixture (decision D3).
13. The software shall create the store file readable and writable by the
    owning user only (realises part of RC-jjsz97's residual; decision D3).
14. The software shall refuse to start, naming the data directory, when that
    directory cannot be created or the store in it cannot be unsealed with
    the given passphrase (moved substance of LLR-85zque).

**Invitation exchange (moved substance of REQ-prjja8, REQ-tcutr6,
REQ-65xqp8).**

15. The software shall produce an Invite for an Organisation one of its
    Personas is bound to, keeping its invite identifier outstanding in the
    sealed store before returning the Invite.
16. The software shall produce an Invite reply only when its caller states
    that the user confirmed, and shall persist the declared expectation
    before returning the reply.
17. The software shall act on an Invite reply only when its (Organisation,
    invite identifier) pair is outstanding and names the Organisation the
    caller selected. It shall admit through the chain write, commit, and
    then send, and shall stop holding the pair once the admission has
    committed, whether or not the send succeeded.
18. The software shall send a committed update to every Device the
    committed Membership record lists, and a revocation to every Device of
    each Member the update removed (PR-3ue4va, if D6 places it in S4).
    Failure to reach one Device shall not prevent the attempt on the others,
    and shall be reported per Device.

**Risk notes for org-io's analysis** (to be run with `analyze-risks`):

- Persisted and in-memory state can diverge (items 11 and 12).
- A torn store write loses every secret: the availability face of
  HAZ-45ucqx.
- The device seed crosses the unit boundary (REQ-y7tsft and RC-8a4xjb apply
  to org-io's handling).
- HAZ-5f9jcm's network-reachable decoder: org-io is now the first thing a
  hostile peer touches.
- HAZ-bedm57's request-driven chain reads (item 8).

## 7. Order of tasks

*Superseded 2026-10-07 by "## Tasks" below, which orders org-node's sans-IO
tasks first, then org-io's, then the app's. Kept as history.*

Gate: S2 merged (org-io exists, the chain read is in org-io), and the
key-pair change merged (ruled 2026-10-06: S4's code starts after it).
Step 1's requirements and risk are drafted already (§12); in develop they are
completed with the moves of §3.

1. **Requirements and risk** (grill, analyze-risks). Mint org-io's items from
   §6. Move the items of §3.1 and §3.5 into org-io draft ledgers (D1). Write
   the amendments of §3.2, §3.3 and §3.6. Write the dependency-edge
   assessment for org-io → iroh.
2. **Design.** Update org-io's decomposition (the endpoint, framing, store
   file, exchange runner and receive loop as SDD items) and org-node's.
   Write the SOUP rows (§4). Write an ADR for sans-IO org-node (D2).
3. **org-node, store sealed form.** `unseal`/`seal`, `SealedStore`, the
   operations return it. Rewrite the store tests over bytes, and re-pin the
   golden.
4. **org-node, Wire message codec.** Move `wire.rs` out of `transport/`
   (prefix dropped), and rename the tests.
5. **org-node, outgoing builder and byte-driven receive.** `outgoing_message`
   and `check_received`/`commit_received`, plus a single receive path (D5).
   Rewrite `admission_sender`, `service_stories` and `tests/support` without
   iroh.
6. **org-io, transport.** Move `OrgEndpoint`, `TransportMode`, framing,
   `bind_with_relay`, and the 6+1+3 transport tests. Add the typed transport
   error, per-Persona endpoints (D8) and the lock-free receive loop.
7. **org-io, store file.** Atomic write, permissions, the data dir, and the
   startup refusal. Move the `state_assembly` tests.
8. **org-node, invitation types.** `Invite`, `InviteReply`, Blob
   parse/encode, `build_invite`/`build_reply`/`check_reply`/`settle`,
   `outstanding_invites` in `StoreData`, and the fuzz target. Move the
   LLR-b7wgpf tests.
9. **org-io, exchange runner.** `issue_invite`, `produce_reply`,
   `import_reply`, `admit_reply`, and fan-out (D6). Move the invitation tests.
10. **app.** Drop `iroh`, `base64`, `postcard` and `invitation.rs`. Commands
    call org-io. The receiver loop calls org-io's loop step. Typed
    terminal-error classification. Update the app SOUP.
11. **Remove** `org-node/src/transport/` and the `transport` feature. Extend
    `absences.rs` (no `iroh`, `std::fs`, `PathBuf`, `tokio`) and
    `org_node_holds_no_invitation_exchange` (inverted).
12. Trace, coverage and verification record, then merge.

## 8. Overlap with the key-pair change (merged `1f52c36`, 2026-10-07)

*Rewritten 2026-10-07.* The key-pair change merged to master as `1f52c36`
and into this branch at `f51de1a`. What was pending is now fact:

| S4 touches | What the merged key-pair change did |
|---|---|
| `WireMessage` (moves out of `transport/`, codec renamed) | Is the enum `OrgInformation { envelope, record_snapshot, org_private_key }` / `Revocation { envelope }` (LLR-js9dsu, LLR-ecxc76); no secret, no `genesis_snapshot: Option`, no `invite_id`. `fuzz_wire_decode` fuzzes `decode_body` |
| `send_update` becomes the commit's bodies | `send_update(outgoing, recipient, peer_addr)` chooses the kind per recipient from the record (LLR-6ymd6d), with the record's current key — open PR-gnh3j2 (the key of the epoch the Envelope reaches is not guaranteed) |
| `receive_one` / `decode_body` error mapping | LLR-xn5pwc maps `TransportError::Malformed` to `OrgNodeError::MalformedMessage`; S4 maps `decode_message`'s refusal directly |
| The two receive entry points (D5) | They differ on a revocation: `receive_and_verify` refuses an unheld one with `RevocationNotHeld` before the expectations (LLR-38e2kn), the self-delete path with `OrgNotOnChain` (LLR-379hnv); both refuse a revocation leaving the Device listed (LLR-pt32fx). One path keeps `receive_and_verify`'s. Open PR-q8r32t: two unreachable `RevocationNotHeld` arms in it |
| `ExpectedAdmission`, `expect_admission`, the invitation exchange | `ExpectedAdmission { org_id }` only; no `InviteId` anywhere in org-node (LLR-ms8njy, pinned by `org_node_holds_no_invite_identifier`). S4 returns the Invite to org-node, so LLR-ms8njy is narrowed (§3.2) |
| `admit_reply` / `admit_member` command | No `OrgSecret` (LLR-qsjde3); `admit_member` takes no secret or key (LLR-8krgzj) |
| App send commands | LLR-q225ws: one send per command, to one Device; PR-3ue4va (open, `app/docs/problems/2026-10-07-update-fan-out.md:3`) asks for the fan-out, which S4 owns |
| `StoreData`, `OrgRecord`, golden bytes | `OrgRecord.org_private_key` required, `org_secret` gone, `ProvisionalChange` carries keys; pre-change stores unsupported. S4's per-Persona store, Device public key and `outstanding_invites` re-pin the golden on top (LLR-ayrdr8) |
| `org-node/src/error.rs` | Added `MalformedMessage`, `OrgKeyMismatch`, `RevocationNotHeld`, `RevocationNotForThisDevice`. S4 adds store and invitation variants (T3, T6) |

### 8.1 What the 2026-10-07 reconciliation changed

- **Line numbers** of every survey site (§1) and every item (§3) re-read at
  `f51de1a`.
- **New items to amend** from the key-pair merge: LLR-6ymd6d, LLR-48jakr,
  LLR-ms8njy (narrowed: org-node defines an `InviteId` again, only outside the
  Wire message and the expectation), LLR-xn5pwc, LLR-ba2ejp, LLR-38e2kn,
  LLR-pt32fx, LLR-js9dsu, LLR-ecxc76, LLR-8krgzj; LLR-379hnv **retired** with
  the self-delete path; LLR-q225ws moves to org-io and is retired there by
  the fan-out.
- **Two key-pair problem reports S4 resolves:** PR-q8r32t (the split removes
  the unreachable arms) and PR-gnh3j2 (the commit builds every recipient's
  body with that commit's key, so a later commit cannot change the key an
  earlier update carries). PR-gnh3j2 was in S4's scope only by this design
  choice; the owner ruled on 2026-10-07 that S4 resolves it.
- **A name clash:** the draft's public `commit_received` collides with the
  key-pair change's private helper; renamed `commit_checked`.
- **`OrgEndpoint` binds from `&SigningKeypair`** built from the stored
  `DeviceSeed`, the path §13–§14 replace with the keychain.
- **The keychain rulings** (§13–§15) are carried into §1.3, §2.3, §3 and §4:
  `argon2` goes, `hkdf`/`sha2`/`keyring` arrive, `DeviceSeed` becomes the
  move-only `DeviceSecret`, the store is one per Persona, the app's
  passphrase items are amended.
- **Test counts** (§5) recounted: 44 tests move to org-io (was 42), 81
  org-node tests are rewritten value-driven (was 44), 34 store call sites
  (was 32).
- **Draft wording:** the S4 drafts in org-io and org-node no longer say the
  key-pair change is unmerged, and PR-3ue4va is cited at its merged path.

## 9. Findings during the survey

*Booked 2026-10-06:* F1 as app PR-7v8anh
(`app/docs/problems/2026-10-06-receiver-holds-lock.md`),
F2 as org-node PR-n696wv
(`org-io/docs/problems/2026-10-13-non-atomic-store-write.md (moved from org-node in T9)`).
F3 is a wording fix on PR-3ue4va, now merged at
`app/docs/problems/2026-10-07-update-fan-out.md:3`; it is applied with a
dated note when S4 re-homes that report (T12).

- **F1 — the receiver holds the service lock while it waits.**
  `app/src-tauri/src/commands.rs:366-370` locks `state.service`, then awaits
  `receive_and_self_delete_if_revoked`, which awaits `OrgEndpoint::recv_one`
  → `accept()` (`org-node/src/service.rs:883-893`). While the receiver is
  running, every other command that locks the service (create, admit,
  revoke, list) waits until a peer connects. The comment says the lock "is
  held only for the duration of one receive+verify", but that includes the
  idle wait. It is not recorded in any ledger. S4's lock-free loop
  (requirement 7) cures it.
- **F2 — store writes are not atomic.** `store.rs:516` and `invitation.rs:222`
  use `std::fs::write`. An interrupted write can leave an unreadable store,
  and every secret is lost. Not recorded. Addressed by requirement 12 if D3
  includes it in S4.
- **F3 — stale wording in PR-3ue4va** (key-pair branch): "the `org-io`
  session's work, which moves peer communication into the app". Under the
  roadmap it goes into org-io.

## 10. Overlap with S2 (`worktree-org-io-create`) and S3

*Status 2026-10-07:* S2's guardrails skeleton, ADR and requirements drafts
are merged into this branch (`113be59`, from `worktree-org-io-create` at
`31368b0`); S2's develop phase (the crate, the chain read as values, the
submit path) is being planned in parallel on `worktree-org-io-create` and
merges to master first. **S4's develop starts by merging S2's merged result
(T0)**; ruling B means S2 removes `ChainOps` from org-node, so the tasks
below assume `commit_genesis`/`commit_update` take the `OrgState` org-io
read. T0 records any name S2 lands differently.

S2 creates the org-io crate, `units.yaml` entry, `depends_on` edges, SOUP
file and ADR. S4 cannot start implementation before that. Specific overlaps:

- **The chain-read seam.** S4's byte-driven receive (§2.4) needs the chain
  read outside org-node. If S2 keeps a `ChainOps` trait injected into
  org-node, org-node stays async, and S4's sans-IO split of
  `receive_*`/`commit_*` either waits or is done by S4 (D2).
- `service.rs`: `ChainOps`, `MockChainOps`, `SubxtChainOps` and
  `connect_chain_client` (S2) sit in the same file and the same functions
  (`commit_genesis`, `commit_update`, `verify_received`) that S4 reshapes.
- `app/src-tauri/src/submit.rs` `submit_commit_send` and
  `invitation.rs` `admit_reply`: S2 moves the write, S4 the send and the
  exchange. REQ-nfr3n2, LLR-qhjp6g and LLR-be3zv9 are amended by both.
- `app/src-tauri/src/state.rs` `assemble`: S2 rewires the chain, S4 the store
  and the transport mode.
- "The app depends only on org-io" (S2): whether org-io re-exports org-node's
  value types decides whether S4's app changes are import-only.
- RC-mj6gjq, RC-2ferct, HAZ-bedm57, LLR-9ew26y and LLR-s8xp7m (chain read
  after the chain-free checks) are realised across the boundary S2 draws.
- **S3** (`worktree-org-io-commit-workflow`, runs alongside S4) reworks the
  commit paths (persist → write → read and verify → promote, keep the
  delta, reconcile at startup). It touches `commit_update`, `commit_genesis`,
  `receive_and_verify`, LLR-mkj4bz, LLR-b27jr6, the provisional store and
  PR-924ftr. S4's split of the same functions conflicts textually. Whichever
  merges second rebases.

## 11. Decisions (all ruled 2026-10-06)

Every decision this draft opened was ruled by the owner on 2026-10-06
(roadmap `docs/plans/2026-10-06-org-io-roadmap.md`, "Owner's rulings on the
S2–S4 design drafts"). The recommendation was accepted in each case except
D2, where the ruling is the recommendation stated as a general rule (B).

- **D1. Moving items between units.** Ruled (A): **moved items keep their
  IDs**; the definition moves to org-io's ledger with a dated note, in the
  develop phase. §3 is the list.
- **D2. org-node's seam style.** Ruled (B): **values in, values out.**
  org-node returns the sealed store bytes for org-io to write; no IO traits,
  no async in org-node. §2.3, §2.4.
- **D3. Atomic and owner-only store writes.** Ruled: **included in S4**
  (PR-n696wv). §2.3. *(2026-10-07: the owner-only half is dropped; the
  atomic write stays.)*
- **D4. Outstanding invites.** Ruled: **in the encrypted store, no
  migration.** §2.3.
- **D5. One receive path.** Ruled: **yes**; `receive_and_self_delete_if_revoked`
  is retired, fixing PR-kwwap5. §2.4.
- **D6. PR-3ue4va fan-out.** Ruled: **in S4, Networked mode.** §2.5.
- **D7. org-io's safety class.** Ruled: **C.**
- **D8. Endpoints.** Ruled: **one per Persona** (PR-8qsnhx). §2.5.
- **D9. Sequencing with the key-pair change.** Ruled: **S4's code starts
  after the key-pair change merges.**
- **Also ruled, from the survey's findings:** the receive loop holds no
  lock while it waits (PR-7v8anh, §2.4); no private signing key is ever
  exposed (roadmap "Two keys" ruling), which §2.2 resolves for the endpoint
  by an opaque, move-only `EndpointSecret` converted inside org-node.

## 12. Requirements and risk drafted for S4 (2026-10-06)

Minted with `.guardrails/scripts/new-id.sh --unit <unit>`. The moved items of
§3 are not restated; they move with their IDs in develop.

**org-io requirements** (`org-io/docs/requirements/2026-10-06-transport.md`):

| ID | Subject | Parent |
|---|---|---|
| REQ-bupq5y | One endpoint per Persona, identity = Device key; send from the Organisation's Persona | derived |
| REQ-xuzw32 | Device secret only as org-node's opaque value, by move into the endpoint builder; never cloned, read, rendered, stored, handed out | RC-pt49zn |
| REQ-szt6ea | Length-prefix framing; read at most 1 MiB + 4 per stream; prefix discarded | RC-5wz559 |
| REQ-8e57wm | Typed terminal transport condition | derived |
| REQ-qkrj55 | No lock held while waiting or reading (PR-7v8anh) | RC-97kdfv |
| REQ-fnsjb6 | Every body through org-node's single receive path; chain read after the chain-free checks, at most once (PR-kwwap5) | derived |
| REQ-g9cu2c | Write only org-node's sealed bytes, before reporting success; discard unpersisted state on failure | derived |
| REQ-zmtmd9 | Atomic replace of the store file (PR-n696wv) | RC-37y46c |
| ~~REQ-hvkz4c~~ | Dropped before merge (owner ruling, 2026-10-07: no file-permission control) | — |
| REQ-a2a6rm | Admission sequence: build, chain write, commit, send, settle; settle even if a send fails | derived |
| REQ-pnr6kh | Outstanding invites only in the sealed store; no migration | derived |
| REQ-6skn67 | Fan-out to every listed Device and every removed Device, Networked mode (PR-3ue4va) | RC-wbs9ch |
| REQ-n82q47 | Total peer-address parse, fuzzed | derived |
| REQ-8trkps | Loopback send only to an address whose identity is the recipient's Device key (PR-2dmjzj) | RC-d6f25u |
| REQ-x6kdse | expects org-node: unseal from bytes, sealed bytes out | derived |
| ~~REQ-38desf~~ | Dropped before merge (§14) | — |
| REQ-tzr3dk | expects org-node: one value-driven receive path | derived |
| REQ-jxyh9c | expects org-node: recipients and bodies for a committed update | derived |
| REQ-txkm94 | expects org-node: Invite build/parse and store bookkeeping | derived |

**org-io risk** (`org-io/docs/risk/2026-10-06-transport.md`):

| Hazard | Control |
|---|---|
| HAZ-v9bu8b device secret leaks through the endpoint path (S3/P2) | RC-pt49zn opaque move-only secret, builder only |
| HAZ-4n7pss unbounded inbound stream (S3/P2) | RC-5wz559 read cap, prefix discarded |
| HAZ-esd966 receive loop holds the service lock (S3/P3) | RC-97kdfv lock-free wait |
| HAZ-dd78f9 torn store write loses every key (S3/P2) | RC-37y46c atomic replace, success only after write |
| HAZ-5n56w2 store file readable by other accounts (S3/P2) | ~~RC-hnxhw2~~ dropped 2026-10-07; controlled by RC-d8dbxs alone (§13) |
| HAZ-5qk64w update reaches one Device only (S3/P3) | RC-wbs9ch fan-out |
| HAZ-ztkex9 loopback send to an altered address (S3/P2) | RC-d6f25u address identity must equal the Device key |

Residuals at S3 other than the device secret (which the never-expose ruling
makes a requirement, not a risk) are recorded as not accepted, pending the
owner's benefit-risk ruling before S4 merges.

**org-node requirements** (`org-node/docs/requirements/2026-10-06-sans-io.md`),
each `exported: yes` and answering one org-io expectation:

| ID | Subject | Satisfies |
|---|---|---|
| REQ-x3jrh8 | Unseal from bytes; every changing operation returns the sealed store; no file IO | REQ-x6kdse |
| ~~REQ-rsb4we~~ | Dropped before merge (§14) | — |
| REQ-2ugh2f | One value-driven receive path (check, then commit) | REQ-tzr3dk |
| REQ-6ehufy | Recipients and bodies for a committed update; writes and sends nothing | REQ-jxyh9c |
| REQ-58tzex | Invite/reply build and parse; outstanding ids and expectations in the store | REQ-txkm94 |

## 13. Owner rulings on §12's open points, and the keychain-keyed store (2026-10-06)

Ruled by the owner on 2026-10-06, after §12 was drafted:

1. **The endpoint path is ruled as designed.** The device ed25519 key reaches
   iroh by the move-only path of §2.2: org-node converts, org-io moves the
   result into the endpoint builder, and a source scan pins it. REQ-v4tfap
   gets its dated note in develop, after S2 merges, naming both move-only
   paths (this one and the keychain path below).
2. **org-node may depend on `iroh-base`, key types only.** The org-node SOUP
   row is written in develop. *(Superseded the same day by §14: org-node has
   no `iroh-base`.)*
3. **Residuals accepted** under ruling D, each with the benefit-risk
   reasoning in org-io's risk draft ("Residuals the owner accepted"):
   HAZ-4n7pss, HAZ-esd966, HAZ-dd78f9, HAZ-5qk64w (at P2 until S6 and S7),
   HAZ-ztkex9.
4. **HAZ-5n56w2 is not accepted.** The owner: "I would like the local store
   to be encrypted with the ed25519 device key." Refined the same day: "If
   the private ed25519 key is in the OS keychain, do we really need to also
   store it in org-node? Won't it suffice to just keep the public key in
   org-node?" Agreed. The design:
   - The device ed25519 private seed lives **only** in the OS keychain (or a
     secure enclave or hardware key behind it), one item per Persona's Device
     key. org-node's sealed store keeps the Device **public** key only.
   - org-node uses the private seed transiently, per operation, received from
     org-io as an opaque move-only value and zeroised after use: (a) deriving
     the store key when unsealing or sealing, (b) signing (S3's signed
     acknowledgement), ~~(c) producing the `EndpointSecret`~~ (superseded by §14: org-io builds the endpoint key). Nothing derived
     from it is persisted; the store key is re-derived on each open.
     *(Ruled 2026-10-07: "sealing" here means the first seal of a new
     store; later saves seal under the store key the open store holds in
     memory, so an ordinary save needs no keychain read.)*
   - The store key is HKDF-SHA-256 over the seed, with the Device public key
     as salt and a fixed, versioned ODS store-key label as info, so the
     signing key is never reused as the encryption key.
   - The X25519 Organisation private key, each Persona's X25519 member seed
     and every other secret stay in the sealed store, under the new key.
   - Passphrase sealing (Argon2, the fixed salt) is retired, and the app
     stops prompting for a passphrase.
   - Keychain access is IO, so org-io does it, by the **second named
     move-only path**: org-io moves the keychain's returned bytes, in one
     function, straight into org-node's opaque device-secret value, and
     writes a new seed to the keychain only through that value's single
     scoped write. The path is source-scanned.
   - When the keychain is locked, missing or unavailable, org-io refuses and
     never falls back to a passphrase or a plaintext file. When the store
     exists but its item is missing, it refuses and never overwrites the
     store.
   - **Platforms** for develop are those of the keychain backends: Windows
     (Credential Manager), macOS (Keychain Services) and Linux (Secret
     Service over D-Bus, with the kernel keyutils as an alternative). This
     settles §12's Windows-permissions point: REQ-hvkz4c applies the
     owner-only ACL there, as defence in depth. *(Superseded 2026-10-07:
     the platforms are Windows, macOS, Linux, Android and iOS, and no
     file-permission control exists on any of them; see the rulings block
     at the top.)*
   - **SOUP candidate, assessed in develop:** the `keyring` crate (v3), with
     its `apple-native`, `windows-native` and `sync-secret-service` (or
     `linux-native`) backend features. Its mock credential builder is used
     only under `test-support`, never in a release build. In org-node, `hkdf`
     and `sha2` are added and `argon2` is removed.
   - **Migration (proposal, not a ruling):** none. There is no production
     install, and the key-pair change already drops support for older
     stores. A pre-S4 passphrase store is left in place and not read; the
     node starts as a new install.
   - **S8 reuse:** production custody of the user's sr25519 signatory key
     (roadmap S8) can use the same keychain mechanism and the same move-only
     pattern. S8's requirements are minted when it starts.

**Items added to the drafts** (minted 2026-10-06):

| ID | Unit | Subject |
|---|---|---|
| REQ-hw82ue | org-io | Read the device secret only from its keychain item, moved straight into org-node's opaque value (RC-jgyd6m) |
| REQ-rz93uz | org-io | Write a new device secret only to the keychain, through the value's scoped write (RC-jgyd6m) |
| REQ-6d9xv6 | org-io | Keychain locked, missing or unavailable: refuse, never fall back (RC-d8dbxs) |
| REQ-fh5gsq | org-io | Store present but item missing: refuse, leave the file untouched, create nothing (RC-hd5des) |
| REQ-v6r2gd | org-io, expects org-node | Transient opaque device-secret value, one operation, zeroised, nothing persisted (RC-jgyd6m) |
| REQ-qza96g | org-io, expects org-node | Store sealed under a domain-separated device-derived key; public key only; no passphrase (RC-d8dbxs) |
| REQ-26mjqa | org-node, exported | Answers REQ-v6r2gd |
| REQ-qthum6 | org-node, exported | Answers REQ-qza96g (HKDF-SHA-256, versioned ODS label) |
| HAZ-eyene4 | org-io | Keychain item missing → store overwritten, every key lost (S3/P2) |
| RC-jgyd6m | org-io | The keychain move-only path; mitigates HAZ-v9bu8b (widened to both paths) |
| RC-d8dbxs | org-io | The keychain-keyed store, no fallback; mitigates HAZ-5n56w2 |
| RC-hd5des | org-io | Refuse, never overwrite, when the item is missing; mitigates HAZ-eyene4 |

REQ-x6kdse and REQ-x3jrh8 now take the transient device secret where they
took a passphrase.

**Amendments for develop (IDs kept, ruling A).** org-node's are listed with
file:line in its sans-IO draft: REQ-hzm4kt, SDD-af5vnt, LLR-t4u66w,
LLR-s78sh7, LLR-tev8h8, LLR-ctzkv7, LLR-cns6q6, LLR-sz4xhc, LLR-ayrdr8,
REQ-y7tsft, HAZ-45ucqx, RC-jjsz97 and the `argon2` SOUP row. The person unit
holds no private seed and is unaffected. The app's passphrase items are
retired or amended:

- REQ-7g3k9a (`app/docs/requirements/2026-09-14-tauri-shell.md:336`, fail to
  start without a passphrase);
- RC-xt4qr3 (`app/docs/risk/2026-09-14-app-hazards.md:667`);
- LLR-tpkmh3 (`app/docs/architecture/2026-10-05-decomposition.md:50`,
  `resolve_passphrase`).

They become "fail to start, naming the condition, when the keychain refuses"
(org-io's REQ-6d9xv6). S3's signing requirement for the signed
acknowledgement (REQ-y99c9w, on S3's branch) is amended to take the device
seed as the transient value of REQ-26mjqa, not from the store. That is
coordinated with S3, and whichever change merges second applies the
amendment.

**Open decisions this raises** (recommendations in the S4 report):

- **One sealed store per Persona, or one per device?** Each Persona has its
  own Device key, so a store keyed by "the" device key needs one store per
  Persona. Recommended: one store per Persona, which matches one endpoint per
  Persona. The alternative needs a separate node-level key in the keychain.
- **HAZ-5n56w2's re-assessed residual** (S3/P1: code running as the same user
  while the keychain is unlocked, and a stolen device with an unlocked
  keychain). Recommended: accept. It is the exposure every secret in the
  user's keychain has, and S8 relies on the same boundary.
- **HAZ-eyene4's residual** (S3/P1: a keychain item lost for good loses its
  store, because no second copy of the seed exists by design). Recommended:
  accept for now, and record device-key backup or recovery as a later-stage
  question. A recovery copy would reopen the exposure this ruling closed.
- **Headless Linux without a Secret Service** (CI, servers): the node
  refuses to start. Recommended: accept, and use the `keyring` mock only
  under `test-support` for tests. A file fallback is forbidden by ruling.

## 14. Owner ruling: org-io builds the endpoint key (2026-10-06)

This supersedes the `EndpointSecret` and `iroh-base` design of §2.2, §2.6 and
§13 point 2.

- **org-io builds iroh's `SecretKey` and binds the endpoint itself, in one
  function.** It reads the device seed from the OS keychain into a
  zeroising, non-cloneable buffer, builds the key, moves it into the endpoint
  builder, and zeroises the buffer before returning. A source scan names
  iroh's secret-key type in that function only. This is org-io's REQ-xuzw32
  and RC-pt49zn, both rewritten.
- **org-node leaves the iroh path entirely.** It has no `iroh` or `iroh-base`
  dependency and no endpoint-secret type. The expectation pair that asked for
  one is **dropped**, not repurposed: org-io's REQ-38desf and org-node's
  REQ-rsb4we were never merged and nothing else references them. Their IDs
  are not reused.
- **org-node still receives the seed transiently**, move-only and zeroised,
  for two things only: deriving the store key and signing (REQ-26mjqa
  answering REQ-v6r2gd). S3's signing requirement for the signed
  acknowledgement (REQ-y99c9w) takes it the same way.
- REQ-hw82ue (the keychain read) now feeds either org-node's operation or the
  endpoint build, each in one function.

**Develop list addition: REQ-v4tfap's dated note** (S2's never-expose
requirement, after S2 merges). Its wording "org-io never receives, holds or
outputs a device private key" becomes: *the device and signatory private keys
never leave the process. There is no output, log line, error, `Debug` or
`Display`, file, or IPC to the app that carries them. In memory they are only
moved, never cloned, and zeroised after use.* The note names the two
move-only paths: the endpoint build (REQ-xuzw32) and the keychain-to-org-node
path (REQ-hw82ue, REQ-rz93uz).

## 15. Owner rulings on §13's open decisions (2026-10-06)

All four were ruled as recommended:

1. **One sealed store per Persona**, each keyed by its own Persona's Device
   key. org-io keeps one store file and one keychain item per Persona. This
   is recorded in org-io's keychain section and in REQ-qthum6.
2. **HAZ-5n56w2's re-assessed residual is accepted** under ruling D: code
   running as the same user while the keychain is unlocked, and a stolen
   device with an unlocked keychain.
3. **HAZ-eyene4's residual is accepted** under ruling D: a keychain item lost
   for good makes that Persona's store unrecoverable, and the Member is
   re-admitted with new keys. Device-key backup and recovery is recorded as
   an unminted later-stage item in the roadmap, next to S8.
4. **Headless Linux with no Secret Service:** the node refuses to start.
   `keyring`'s mock is used only under `test-support`. *(Refined 2026-10-07:
   the in-memory credential store is `keyring`'s mock only if T7 shows that
   it shares an item between `Entry` handles; otherwise org-io's own
   stand-in, also under `test-support` only.)*

## 16. Reconciliation with the merged S3a change (2026-10-07)

S3a (`60c7d58`) changed the code S4 reshapes. Line numbers below are at
`e0e4dac`. Where §1–§15 disagree with this section, this section and the
tasks win.

### 16.1 What S3a changed under S4

| S3a fact (file:line at `e0e4dac`) | Effect on S4 |
|---|---|
| `transport/wire.rs:18-41` `WireMessage` has three kinds: `OrgInformation { envelope, record_snapshot, org_private_key }` (0), `Revocation(RevocationNotice)` (1, identity and absence proof, no Envelope; LLR-dc45ur), `Acknowledgement(Acknowledgement)` (2, LLR-378cj4); `WireMessage::org_id()` | T2 moves all three; a revocation body is a notice, not "the Envelope alone" (LLR-gdfqn9 amended) |
| `transport/wire.rs:56-67` `decode_body` refuses leftover bytes for every kind (`postcard::take_from_bytes`, `Ok((message, []))`) | T2's `decode_message` keeps that; `wire_frame_bound.rs` has 10 tests (was 7) |
| `lib.rs:24-26` `revocation` is gated on `transport` (the Wire message carries its types); `test_fixtures::bob_absence_notice` too | T2 ungates the module's wire types; T14 removes the last `transport` gate |
| `transport/endpoint.rs:317` `recv_one -> (DevicePublicKey, WireMessage)`; `service.rs:1005` `receive_one` returns the sender | The authenticated sender **is** passed to org-node from S4 on: `check_received(sender, body)` (T4); org-io hands it over (T10, T11). §2.2's "the authenticated sender key is not passed" and LLR-cvyyr8's last sentence are superseded |
| Member-sender rule R1 (`service.rs:1364` `sender_listed`; LLR-2r2fha, LLR-kzgjz8, LLR-3aysup, LLR-3q63zv; RC-u7kdam): an update for a held Organisation and a revocation only from a Device the current record lists; a first admission from any sender; an acknowledgement only from the Device it names, which the receiver holds as removed (checked before the signature) | T4 restates the three over `check_received`; T11a adds the transport acceptance for a revoked Device's acknowledgement (owner ruling 8) |
| `revocation.rs` (`check_notice`, `accept`, `sign_acknowledgements`, `check_acknowledgement`, `VerifiedAcknowledgement`, `persona_device`) and `reconcile.rs` (`reconcile`, `Reconciled`), pure over values; `accept`, `reconcile` and `commit_step` take `device_seeds: impl FnOnce() -> Vec<DeviceSeed>`, called only on the path that signs | T1/T3: the seed source becomes `impl FnOnce(&[DevicePublicKey]) -> Vec<DeviceSecret>`, which org-io fills from the keychain (a signing operation under REQ-6d9xv6); `persona_device` reads the stored public key (its own comment says S4 does this) |
| `service.rs:1383` `device_seeds_bound_to` clones each bound Persona's stored `DeviceSeed` (comment "S4: org-io moves the seed in from the OS keychain") | Deleted in T3 |
| `keys.rs` `SigningKeypair::sign` (public) | Stays; `DeviceSecret` gains no public signing operation (T1's four-operation scan holds); signing builds a `SigningKeypair` crate-internally (T3) |
| `service.rs:1216` `ReceiveOutcome { org_id, epoch, root, acknowledgements, acknowledged: Option<VerifiedAcknowledgement> }`; `SelfDeleteOutcome::{SelfDeleted { acknowledgements }, UpdatedNotRevoked, Acknowledged }` | T4's `ReceiveOutcome` keeps `acknowledgements`; an acknowledgement is decided in `check_received` with no chain read and returned as `Checked::Acknowledged` |
| `service.rs:1236` `CommitOutcome { org_id, epoch, root, outgoing: OutgoingUpdate, revocations: Vec<RevocationNotice>, acknowledgements }` (LLR-a8z7r5); `send_update(&CommitOutcome, recipient, peer_addr)` refuses a recipient that is neither listed nor noticed (`NoRevocationForRecipient`) | T5 folds `revocations` into the bodies and keeps `acknowledgements`; a commit that removes this node returns no bodies; `NoRevocationForRecipient` goes with `send_update` (T5; the app's match in T15) |
| `service.rs:1300` `commit_step` and `:1349` `removal_step`, shared by `commit_update`, `reconcile` and the receive paths (LLR-23sfdh, LLR-fm38ww) | T3–T5 keep the one removal step |
| `store.rs:384` `StoreData::forget_organisation` (LLR-pba7yu) removes the Organisation's record, Personas, provisional updates and expectations; `absences.rs` pins it as the only remover | With one store per Persona (T3) a removal empties that Persona's store: the three removing operations return `StoreChange::Forgotten { device_key }` and org-io deletes the file (T9, LLR-LLR-amv4mb); T6 extends `forget_organisation` to the outstanding invites |
| `store.rs` `OrgRecord.kept_change_set`, `StoreData::held_update_for`; golden re-pinned by S3a | T3's re-pin is on top of S3a's |
| `service.rs:1062` `OrgService::reconcile(rng, org_id)`, async, reads the chain (the interim caller until org-io's startup reconcile, S3b-io T15) | Value-driven in T4 (state in, `StoreChange` out) unless S2 already did it (T0's table) |
| PR-q8r32t **resolved by S3a** (`RevocationNotHeld` is raised only in `revocation.rs`) | T4 no longer resolves it; its scan stays, re-targeted |
| New org-node tests: `revocation.rs` (20), `reconcile.rs` (10), `commit_workflow_errors.rs` (3); `admission_sender.rs` 52, `commit_paths.rs` 36, `receive_chain_reads.rs` 15, `service_stories.rs` 8, `absences.rs` 9 | Counts in T3–T5 updated below |
| New support helpers (`tests/support/mod.rs`): `ROGUE_SEED` `:30`, `deliver_from` `:461`, `CountingChain` `:500`, `setup_counted` `:537`, `setup_counting` `:550`, `receiving_addr` `:566`, `send_from` `:576`, `deliver_to_self_delete` `:587`, `deliver_to_receive` `:597`, `deliver_from_to_self_delete` `:603`, `deliver_from_to_receive` `:618`, `device_seed_of` `:633`, `trie_without` `:639` | T3 and T4 replace them |
| app `events.rs:16-21` `ReceiverOutcome::AcknowledgementReceived`; `:70-150` `classify_receive_error` lists the S3a variants; `TERMINAL_ERRORS` now `:179`; `commands.rs:369` `next_outcomes` matches `SelfDeleteOutcome::Acknowledged`; `submit.rs:138` passes `&outcome` to `send_update` | T15, T16 |
| `preflight.rs`, `bin/preflight.rs` unchanged by S3a (still chain and transport) | S2 reduces them to the transport check (owner ruling 7); T10 moves that check into org-io; T14 deletes org-node's copy |

### 16.2 Transport-driven org-node tests, recounted at `e0e4dac`

Tests that reach the transport through `tests/support` (spawn, deliver,
receiving address, `send_update`, `ensure_endpoint`): `admission_sender.rs`
44 of 52, `receive_chain_reads.rs` 15 of 15, `commit_paths.rs` 11 of 36,
`service_stories.rs` 7 of 8, `expected_admission.rs` 5 of 8,
`service_lifecycle.rs` 3 of 9, `organisation_key.rs` 1 of 11, `revocation.rs`
1 of 20. Less the 10 that move to org-io (§5: 5 in `admission_sender.rs`, 2
in `commit_paths.rs`, 3 in `service_lifecycle.rs`), about **77** are
rewritten value-driven (was 81). Of those, 28 drive the self-delete path
(`admission_sender.rs` 15, `receive_chain_reads.rs` 9, `service_stories.rs`
4; was 21). `PersonaStore::open` call sites: 37 across 7 files
(`store_at_rest.rs` 12, `persona_records.rs` 12, `commit_paths.rs` 3,
`organisation_key.rs` 3, `service_lifecycle.rs` 3, `provisional_store.rs` 2,
`tests/support/mod.rs` 2; was 34).

### 16.3 What this means for the drafts

Amended before first merge, each with a dated note in its file: org-io's
REQ-fnsjb6 and REQ-tzr3dk (the sender is handed over; the acknowledgement and
the revocation notice are on the one path), REQ-bupq5y's note (no longer "no
sender checks"), LLR-cvyyr8 and LLR-z35vbh (the accept task forwards the
sender with the body); org-node's REQ-2ugh2f, LLR-zetn7a, LLR-6qjmhf and
LLR-gdfqn9. Added: owner rulings 7 and 8 above, with REQ-ur7z4a, HAZ-abm3xs,
RC-b4m5sj, LLR-wvgec4, LLR-unqfj5, LLR-h9tcat and LLR-amv4mb in org-io's drafts.

### 16.4 Open for the owner (raised by this reconciliation) — all ruled 2026-10-08

*Ruled by the owner on 2026-10-08 (rulings 9–12 at the top of this file):
1 as recommended; 2 accepted; 3 as recommended, with the item deleted when
delivery ends or the Organisation's limit passes; 4 **not** as recommended:
no `ODS_DEVICE_KEY`, the preflight finds the device key through the keychain
path as the node does. The recommendations below are kept as history.*

1. **The verdict byte (T11a, LLR-unqfj5).** "Tries the next address on
   refusal" needs the receiver to say "refused". Recommended: the receiver
   answers each inbound stream with one byte, accepted or refused, and
   nothing else; the ALPN is unchanged and pre-S4 senders ignore it. It is a
   wire-protocol addition, so it is put to the owner rather than assumed.
2. **HAZ-abm3xs's residual** (org-io risk draft): recorded as not accepted,
   pending the owner's benefit-risk ruling, as for the other S3 residuals.
3. **A forgotten Persona's keychain item (T9, LLR-amv4mb).** S3's proposed
   7 and 8 keep it for the acknowledgement delivery and delete it when that
   ends, which is S3b-io's. Recommended: S4 removes the store file and
   leaves the item; if S4 merges before S3b-io, the item stays until S3b-io
   lands (no acknowledgement is sent before then either).
4. **The preflight's Device key (T10, LLR-h9tcat).** Recommended:
   `ODS_DEVICE_KEY` names the Device **public** key of a Persona whose
   keychain item exists; the alternative is "the first store in the data
   directory".

## 17. Reconciliation with the merged S2 change (2026-10-08)

S2 merged to master as `e41cb09` and into this branch at `24ca24c`. This
section is T0's step 4 ("S2 names as landed"), done now, plus the S2 rules
S4 must keep and what each task changes because of them. Line numbers are at
`24ca24c`.

### 17.1 S2 names as landed

| S4 planned | Landed in S2 (`e41cb09`) | S4 uses |
|---|---|---|
| `OrgService::new(..)` | `OrgService::new(store: PersonaStore)`: one store, no chain argument; `set_transport_mode`, `with_endpoint`, `endpoint()` (public), `transport_mode()` (`test-support`) remain | T3 changes it to `new(stores: Vec<PersonaStore>)`; T14 removes the transport members |
| `commit_genesis`/`commit_update(.., chain_state: Option<OrgState>)` | As planned, synchronous: `commit_genesis(rng, persona_id, org_id, proxy_account, chain_state) -> ReceiveOutcome`, `commit_update(rng, org_id, chain_state) -> CommitOutcome`; the signing seeds come from the private `device_seeds_bound_to` (`service.rs:1383`) | T3 adds `device_secrets` and the `StoreChange` |
| `reconcile` async (S3a's interim) | **Value-driven already**: `reconcile(rng, org_id, chain_state: Option<OrgState>) -> Reconciled`, synchronous (`service.rs:1001`) | T4 no longer converts it; T3 adds `device_secrets` and the `StoreChange` |
| Receive split `receive_message`, `prepare_receive` → `PendingReceive` → `apply_receive`, `prepare_self_delete`/`apply_self_delete` | As planned: `receive_message() async -> (DevicePublicKey, WireMessage)` (transport-only, binds the **first** Persona's endpoint, decodes, maps `Malformed` to `MalformedMessage`, `service.rs:958`); `prepare_receive(sender, message: WireMessage) -> Prepared<ReceiveOutcome>` with `Prepared::{Done(O), NeedsChain(PendingReceive<O>)}` and `PendingReceive::org_id()` (`:682`, `:1213`); `apply_receive(rng, pending, chain_state)` (`:746`); `prepare_self_delete`/`apply_self_delete` → `SelfDeleteOutcome::{SelfDeleted { org_id, acknowledgements }, UpdatedNotRevoked { org_id }, Acknowledged(VerifiedAcknowledgement)}` (`:876`, `:915`, `:1193`) | T4: `prepare_receive` → `check_received(sender, body: &[u8])` (decoding moves in), `Prepared` → `Checked`, `PendingReceive` → `CheckedMessage`, `apply_receive` → `commit_checked`; the self-delete pair, `SelfDeleteOutcome` and `receive_message` go (`receive_body` is the transitional replacement) |
| `ReceiveOutcome` | `{ org_id, epoch, root, acknowledgements, acknowledged: Option<VerifiedAcknowledgement> }` (`:1157`) | T4 adds `kind: ReceiveKind`; an acknowledgement is `Checked::Acknowledged`, so `acknowledged` goes |
| `CommitOutcome` | S3a's `{ org_id, epoch, root, outgoing: OutgoingUpdate, revocations, acknowledgements }`, `Clone` (`:1177`) | T5 as planned (bodies, not `Clone`) |
| `send_update` | Still org-node's: `send_update(&CommitOutcome, recipient, peer_addr: Option<iroh::EndpointAddr>)` async (`:625`), called by org-io's `submit::submit_commit_send` | T5 replaces it by `send_body` (transitional); T12 by org-io's `send_committed` |
| `ensure_endpoint` | Still org-node's, async, bind-once per service from the named Persona's stored seed (`:1057`); reachable from org-io only through the `test-support` `NodeBuilders::ensure_endpoint` | T3 makes it the transitional `bind_endpoint(persona_id, secret)`; T10 replaces it |
| — | **New:** `OrgService::holds_provisional(org_id, base_root, resulting_root, org_pub_key) -> bool` (LLR-gwk4nk, `:336`), org-io's pre-write guard | Kept; T3 routes it to the bound Persona's store |
| `OrgIo` fields | `{ service: OrgService, reader: Box<dyn StateReader>, writer: Box<dyn ChainWriter>, signatory_reader: Box<dyn SignatorySetReader>, own_account, co_signers }` (`org-io/src/lib.rs:46`) | T11 puts `service` in a `tokio::sync::Mutex` |
| `OrgIo` methods (`not_configured`, `connect`, `for_test`, `node`, `node_mut`, `found_organisation`, `submit_commit_send`, `receive_and_verify`, `receive_and_self_delete_if_revoked`, `is_own_admin`) | `not_configured(service)`; **`open(data_dir, passphrase, transport_mode) -> Result<Self, String>`** (creates the data dir, opens `persona_store.bin`, sets the mode; LLR-rgdx22's clause of 2026-10-08); `connect(self, ChainSettings)` (`dev-seed`; refusal `Box<ConnectFailed>` hands the service back); `for_test(service, &FakeChain)`, `with_own_account` (`test-support`); `is_own_admin`; **`view() -> NodeView`** (no `node()`); `node_mut() -> NodeBuilders`; `node_for_test`/`node_mut_for_test` (`test-support`); `found_organisation(rng, persona_id) -> Result<OrgId, String>`; `submit_commit_send(rng, &BuiltUpdate, recipient, peer_addr) -> Result<CommitOutcome, String>`; `receive_and_verify(rng)`; `receive_and_self_delete_if_revoked(rng)` | T9 changes `open` in place (no passphrase; LLR-uzax55); T4 deletes the self-delete method; T11 renames the receive to `next_received`; T5/T12 change what `submit_commit_send` returns (§17.2 rule 2) |
| `node()` view | `NodeView` (`view.rs`): `personas() -> Vec<PersonaSummary>`, `organisations() -> Vec<OrgSummary>`, `organisation(org_id)`, `persona_public_keys(persona_id)`; `NodeBuilders`: `view()`, `create_persona -> PersonaId`, `admit_member`/`revoke_member -> BuiltUpdate`, `expect_admission`, `ensure_endpoint` (`test-support`); `BuiltUpdate { org_id, base_root, resulting_root, org_pub_key }` | T3 keeps `create_persona -> PersonaId` (the `DeviceSecret` never leaves org-io); T6/T13 replace `expect_admission` by the exchange |
| `StateReader`, `ChainWriter`, `test_support::FakeChain` | As planned: `chain_read::StateReader::read_state(org_id) -> Result<Option<OrgState>, OrgNodeError>`; `submit::ChainWriter::{genesis, update}`; `FakeChain` with read counter `reads()`, switches `write_failing`, `write_hanging`, `read_failing`, `parse_refusing`, `state_hidden`, `epoch_ahead`, and `sharing_slots_with` | Used as named |
| org-node features | `transport = [iroh, tokio]`, `app = [transport, chacha20poly1305, argon2]`, `test-support = [transport, iroh/test-utils]`; no `chain` feature and no `subxt`, `on-chain-client`, `async-trait` or `hex` (LLR-mn5c2q) | T14's list loses `async-trait` (already gone) |
| org-node preflight (ruling 7) | `org-node/src/preflight.rs`: `check_transport(&SigningKeypair, mode, timeout)`; `org-node/src/bin/preflight.rs` reads `ODS_DEVICE_SEED` (default all-zero) and `ODS_TRANSPORT`; both `#![cfg(feature = "transport")]`; `org-node/tests/preflight.rs` (2 tests, no `verifies:`, not in the verify line) | T3 adapts, T10 replaces, T14 deletes (all three files and the `[[bin]]`/`[[test]]`) |
| org-io preflight | `org-io/src/preflight.rs` (chain checks `check_chain_live`, `check_contract`, `render`, `CheckResult`) and `org-io/src/bin/preflight.rs` (`ODS_CHAIN_WS`, `ODS_CONTRACT_H160`, `ODS_ADMIN_H160`); `org-io/tests/preflight.rs` is chopsticks-only | T10 adds `check_transport` |
| — | `pub use org_node as node` in `org-io/src/lib.rs:18` (transitional; roadmap: "narrows them in S4"); the app names `org_io::node::transport::TransportMode` (`state.rs:39`) and its tests name `org_io::node::{store::PersonaStore, service::OrgService, transport::…, DeviceSeed}` | T15 narrows the re-export (§17.2 rule 3) |
| org-io test targets, verify line | `key_custody_config`, `fuzz_seed_parse`, `signatory_key`, `absences`, `chain_read_rule`, `receive_order` (12 tests), `submit_flow` (14), `signatory_rule`, `app_boundary`, `open` (4) on `--features test-support`; `key_custody_config`, `signatory_key`, `dev_seed_env` on `test-support,dev-seed`; `cargo build -p org-io`; clippy with `test-support,dev-seed`; `preflight` excluded (chopsticks) | Plan header updated |
| Coverage | `make coverage-org-io`, `make coverage-branch-org-io`; `ORG_IO_COVERAGE_ARGS` (`Makefile:436`), floors `ORG_IO_LINES` 70, `ORG_IO_REGIONS` 68, `ORG_IO_BRANCHES` 64 | T18 as planned |
| REQ-v4tfap, LLR-3zdw8v | `org-io/docs/requirements/2026-10-08-org-io.md:122`; `org-io/docs/architecture/2026-10-08-org-io.md:626`. Also merged: LLR-7pj5af (`:49`, the receive order), LLR-rgdx22 (`:497`, with `OrgIo::open`), SDD-z3ychz (`:849`, the handle) | §17.3 |
| app | `AppState.org_io: Mutex<OrgIo>` and a separate `outstanding: Mutex<OutstandingInvites>`; `app/src-tauri/src/submit.rs` **deleted** (moved to `org-io/src/submit.rs`); `app/src-tauri/tests/submit_flow.rs` keeps only the 2 LLR-q225ws tests; `invitation.rs` still holds the Blobs, `OutstandingInvites` and `admit_reply`, which calls `OrgIo::submit_commit_send`; `next_outcomes` locks `state.org_io` and awaits `receive_and_self_delete_if_revoked` (PR-7v8anh still present); REQ-x3c8n2 is **already** `expects: org-io` (S2 T9) | T12, T15, T16 adjusted |
| Superseded IDs (PR-zf924s) | REQ-ztdza4 → REQ-uk9rw7 (`org-node/docs/requirements/2026-10-08-sender-rule.md`); LLR-js9dsu → LLR-tajh9d (`org-node/docs/architecture/2026-10-08-wire-kinds.md`; S2's test `exactly_three_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused` names both); LLR-65py3d → LLR-mn5c2q | S4's drafts and tasks name the new IDs |
| S3b-io | **Not merged** at `e41cb09` | T11a drives a test double (`dial_direct_as`) |

### 17.2 S2 rules S4 must keep

1. **The handle's allowlist.** `the_handle_returns_only_allowlisted_public_types`
   (`org-io/tests/absences.rs:497`, LLR-3zdw8v, LLR-7pj5af, LLR-9r2bxd)
   requires every production public method of `OrgIo`, `NodeBuilders`,
   `NodeView` and `ConnectFailed` to return only types on
   `HANDLE_RETURN_ALLOWLIST` (`:456`), and no handle type to `Deref`. Every
   public method S4 adds returns only allowlisted public-data types; a type
   S4 adds to the list (`Received`, `ReceiveKind`, `SendReport`,
   `AdmitReport`, `PeerAddr`, the typed errors, …) is reviewed in the task
   that adds it and holds no private key or secret byte. `SelfDeleteOutcome`
   leaves the list in T4. No `with_node(|service| ..)` closure: it would hand
   `&OrgService` to the caller (T11).
2. **No private key through a public API.** The device ed25519 seed, the
   sr25519 signatory seed and every X25519 secret (Member seed, Organisation
   private key) are never reachable through a public operation of org-io;
   they are moved, never cloned, and zeroised; public keys may be exposed.
   Two S4 consequences: (a) org-node's `create_persona` returns a
   `DeviceSecret` (T3), so `NodeBuilders::create_persona` (and later
   `OrgIo::create_persona`) consumes it inside org-io and returns only the
   `PersonaId`; (b) from T5 `CommitOutcome.outgoing` holds encoded bodies,
   and an `OrgInformation` body carries the Organisation private key, so
   **`CommitOutcome` leaves the allowlist in T5**: `submit_commit_send`
   (and, from T12–T13, the fan-out and `admit_reply`) return a report of
   public data only (epoch, root, per-Device send outcomes), never the
   outcome or its bodies.
3. **The `node` re-export narrows in S4** (roadmap, rulings on the S2–S4
   drafts: "org-io re-exports the org-node types the app uses, transitionally,
   and narrows them in S4"). T15 replaces `pub use org_node as node` by an
   explicit list of the value types the app names (ids, names, public keys,
   errors), so neither `OrgService`, `PersonaStore`, `DeviceSecret` nor the
   Wire types are reachable through org-io.
4. **OS keychain access only in org-io** (owner ruling 6), and
   environment-variable keys and seeds only in test environments (ruling
   12): org-io's only one, `ODS_ADMIN_SEED`, stays behind `dev-seed`.
5. **S2's scans stay green.** LLR-3zdw8v's scans
   (`org_io_names_no_device_or_member_private_key`,
   `org_node_hands_out_no_stored_device_private_key`,
   `org_node_hands_out_no_stored_x25519_secret`) are extended, not
   loosened: T3 adds `DeviceSecret` to the names they forbid outside its
   sanctioned places (org-node's `keys.rs`, the `create_persona`/`unseal`
   signatures, org-io's `keychain.rs` and the transitional holder), by a
   dated clarification of LLR-3zdw8v. `org_io_hands_out_no_bare_service`
   and `the_handle_offers_no_chain_judging_operation_outside_its_own_sequence`
   (LLR-7pj5af, LLR-9r2bxd) keep `commit_*`/`reconcile`/`prepare_*`/
   `apply_*` off the handle; T4 adds `check_received`/`commit_checked` to
   that list. `the_pre_write_guard_copies_no_record_and_no_held_update`
   (LLR-qhjp6g) holds for T5's reshaped `submit_commit_send`.

### 17.3 Merged items S4 must amend (hybrid rule)

Merged items get a dated note for a narrowing or clarification, or a new
superseding ID for a change of meaning; new red-first tests carry both IDs,
and green tests are never re-annotated. Items minted only in S4's unmerged
drafts are edited in place.

| Item (merged) | Task | Change | Class |
|---|---|---|---|
| REQ-v4tfap (org-io) | **T3** (was T10) | The dated note of §14: org-io now receives a device private key, on two named move-only paths (the keychain path, REQ-hw82ue/REQ-rz93uz, and the endpoint build, REQ-xuzw32); "neither receive nor output" becomes "never output; receive only by those moves; never clone; zeroise". T3 is the first commit in which org-io code receives a `DeviceSecret` (from `create_persona`) | narrowing (the "never receive" clause withdrawn; the new obligations are S4's own REQs) |
| LLR-3zdw8v (org-io) | T3; T10 | T3: the scans name `DeviceSecret` too (rule 5). T10: the transport preflight came in by the keychain path (LLR-h9tcat) | clarification |
| LLR-rgdx22 (org-io), its `OrgIo::open` clause of 2026-10-08 | T9 | The clause is withdrawn (opening is LLR-uzax55's: per-Persona files, keychain-keyed, no passphrase); the development-seed clauses stay. `open.rs`'s passphrase test is deleted with the behaviour, its deletion recorded; the others keep their annotation and gain LLR-uzax55 tests beside them | narrowing |
| LLR-7pj5af (org-io) | T4; T11 | T4: one method (the self-delete twin retired, ruling D5). T11: the method is `next_received`; the order it states is unchanged | narrowing; clarification |
| SDD-z3ychz (org-io) | T11 | The handle holds the service behind its own lock and its methods take `&self` | clarification |
| LLR-mn5c2q (org-node, S2) | T4 | Its list of chain-judging operations names `commit_checked` in place of `apply_receive`/`apply_self_delete` | clarification |
| LLR-tajh9d (org-node, S2) | T2 | `decode_body`/`Malformed` become `decode_message`/`MalformedMessage` | clarification |
| LLR-gwk4nk (org-node, S2) | T3 | "the store" becomes "the store of the Persona bound to `org_id`" | clarification |
| PR-jav8zn (org-node, S2) | T3 + T9 | **Resolved by S4**: org-node no longer saves (a refusal changes nothing; the change is returned), and org-io writes atomically, reports a failed write as a storage failure, and reloads (LLR-murjr6); `submit.rs`'s "nothing committed" message after a failed write is corrected | resolution |

S2's other open items (PR-ksm2ua, PR-8geawr, PR-z3gg2b, PR-wk7zwq) are not
S4's; S4 adds `FakeChain` uses that PR-8geawr already covers.

### 17.4 Raised by this reconciliation

- **The preflight's data directory** (ruling 12 leaves it open). The
  preflight lists the store files of a data directory, as `OrgIo::open`
  does. Recommended: the same `ODS_DATA_DIR` the app reads (a path, not key
  material), with no default; T10 records the choice. Put to the owner in
  the report of 2026-10-08. **RULED 2026-10-08 (owner): `ODS_DATA_DIR`, no
  default** — the preflight refuses to run when it is unset.

## Tasks

*Written 2026-10-07 with the `plan-change` skill, after the owner's rulings
of that day and the review of the two S4 design drafts. Execute with
`develop-change`, one task per subagent, in the order of "Execution order"
below; a task marked **Parallel: yes** may run in its own task worktree
(`.guardrails/scripts/task-worktree.sh start <tag>`) beside the tasks named
with it. §1–§15 above are the survey and the rulings; where a task and a
section disagree, the task (and the design items it cites) wins.*

### Plan header

**Goal:** Make org-node free of IO (no file, endpoint, keychain or async) and
give org-io the transport, one sealed store file and one OS keychain item per
Persona, the receive loop and the invitation exchange, so that the app
depends on org-io alone and starts with no passphrase, with every committed
update sent to every listed Device in Networked mode.

**Implements:**
- org-io (drafted on this branch): REQ-bupq5y, REQ-xuzw32, REQ-szt6ea,
  REQ-8e57wm, REQ-qkrj55, REQ-fnsjb6, REQ-g9cu2c, REQ-zmtmd9,
  REQ-a2a6rm, REQ-pnr6kh, REQ-6skn67, REQ-n82q47, REQ-8trkps, REQ-hw82ue,
  REQ-rz93uz, REQ-6d9xv6, REQ-fh5gsq; the expectations REQ-x6kdse,
  REQ-tzr3dk, REQ-jxyh9c, REQ-txkm94, REQ-v6r2gd, REQ-qza96g; RC-pt49zn,
  RC-jgyd6m, RC-5wz559, RC-97kdfv, RC-37y46c, RC-d8dbxs,
  RC-hd5des, RC-wbs9ch, RC-d6f25u; REQ-ur7z4a, RC-b4m5sj (owner ruling 8,
  added 2026-10-07); SDD-xwft92 (LLR-d5y7ks, LLR-d2pzzk,
  LLR-z35vbh, LLR-h9tcat), SDD-aw3nka (LLR-9arvqj, LLR-cvyyr8, LLR-szhe7j,
  LLR-wvgec4, LLR-unqfj5), SDD-egx8rv
  (LLR-nt8dja, LLR-b9bm36, LLR-murjr6, LLR-uzax55, LLR-t4v69b, LLR-amv4mb), SDD-4fvypt
  (LLR-677fqq, LLR-n9dnm4, LLR-8erfbu, LLR-36jer8), SDD-rr7dtw (LLR-h9qf3s,
  LLR-9cb443, LLR-y4dg74), SDD-v2bq89 (LLR-znhmh2, LLR-r9sh6k, LLR-2b2ghe).
- org-node (drafted on this branch): REQ-x3jrh8, REQ-26mjqa, REQ-qthum6,
  REQ-2ugh2f, REQ-6ehufy, REQ-58tzex; SDD-asxwp8 (LLR-bg538c, LLR-2qqqx6,
  LLR-bvt5up, LLR-rdda86), SDD-fysk96 (LLR-37k2s3, LLR-yp69jg, LLR-e5kqhg),
  SDD-esg2kc (LLR-zetn7a, LLR-6qjmhf), SDD-bm27xt (LLR-gdfqn9), SDD-xshv5b
  (LLR-nr8df3, LLR-7q9g9z, LLR-d55brn).
- Moved with their IDs (ruling A): §3.1 and §3.5 to org-io, §3.4 to
  org-node. Amended in place: §3.2, §3.3, §3.6. Retired: LLR-379hnv,
  LLR-g9vmbx, LLR-q225ws (after its move).
- Restated over the one value-driven receive path (T4): S3a's member-sender
  items LLR-2r2fha, LLR-kzgjz8, LLR-3aysup, LLR-3q63zv, and its receive
  items LLR-r7zm39, LLR-tx8ruv, LLR-uw7nmv, LLR-5azhry (dated notes; no new
  behaviour).
- Resolves PR-kwwap5, PR-7v8anh, PR-n696wv, PR-8qsnhx, PR-2dmjzj,
  PR-3ue4va and PR-gnh3j2; PR-7zz76t closes as obsolete (T15). (PR-q8r32t
  was resolved by S3a, `60c7d58`.) *Added 2026-10-08 (§17.3):* resolves
  S2's PR-jav8zn (T3, T9).
- Amends S2's merged items (§17.3, hybrid rule): REQ-v4tfap, LLR-3zdw8v,
  LLR-rgdx22, LLR-7pj5af, SDD-z3ychz (org-io); LLR-mn5c2q, LLR-tajh9d,
  LLR-gwk4nk (org-node).
- Hazards analysed: HAZ-v9bu8b, HAZ-4n7pss, HAZ-esd966, HAZ-dd78f9,
  HAZ-eyene4, HAZ-5n56w2, HAZ-5qk64w, HAZ-ztkex9, HAZ-abm3xs (org-io risk draft); the
  re-assessment of HAZ-45ucqx and RC-jjsz97 (org-node, T3).

**Safety class:** C in every unit touched (org-io, org-node, app); no
per-item override.

**Verification:** each unit's `verify_commands` after this change (final
form written by T18; each task extends its unit's line as it adds a target):
- org-io: S2's lines (recorded 2026-10-08, §17.1: `cargo test -p org-io
  --features test-support --lib --test key_custody_config --test
  fuzz_seed_parse --test signatory_key --test absences --test chain_read_rule
  --test receive_order --test submit_flow --test signatory_rule --test
  app_boundary --test open`; `cargo test -p org-io --features
  test-support,dev-seed --test key_custody_config --test signatory_key
  --test dev_seed_env`) plus, on the `--features test-support`
  line, `--test keychain --test store_files --test transport_handshake
  --test transport_networked --test endpoints --test preflight_transport
  --test receive_step --test acknowledgement_inbound --test
  send_fanout --test peer_addr --test fuzz_peer_addr --test
  invitation_exchange --test node_stories`; `cargo build -p org-io`;
  `cargo clippy -p org-io --all-targets --features test-support,dev-seed --
  -D warnings`.
- org-node: `cargo test -p org-node --features app,test-support --lib` with
  today's targets (S3a added `revocation`, `reconcile` and
  `commit_workflow_errors`) less `transport_handshake`, `transport_networked` (moved,
  T10), starting from the line as it stands at `24ca24c` after S2, plus `--test sealed_store --test
  receive_path --test outgoing_bodies --test invitation --test
  fuzz_invitation_parse --test fuzz_unseal`; the five `quint` lines
  unchanged; `cargo clippy -p org-node --all-targets --features
  app,test-support -- -D warnings`.
- app: today's three lines less `--test invitation` (moved, T13) and
  `--test submit_flow` (its two LLR-q225ws tests move to org-io in T12; S2
  moved the rest of it).
- Coverage: `make coverage-org-io` and `make coverage-branch-org-io` (S2's
  targets; S4 extends their target list and re-sets the floors in T18), and
  every other unit's targets unchanged.
- Trace: `.guardrails/scripts/check-units.sh`; for each of the six units
  `GR_CONFIG=<unit>/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`
  and `… check-ids.sh --allow-draft-files`. Expected at the end: exit 0;
  UNRESOLVED-PR and UNMET-EXPECTATION advisories only for items S4 does not
  touch.

### Conventions for every task

- Work in this change worktree (or the task worktree the dispatcher gives
  you). Commit with plain `git commit`; **never** a `Co-Authored-By` line.
  One plain git command per shell call.
- Never edit a file with `sed`; use the editor (AGENTS.md, "Lessons
  learned"). Rust test files especially.
- Red first: write the test, run it, paste the failure into the task's
  report, then implement, then paste the green run. A test that cannot be red
  at runtime (a new target or an API that does not exist yet) is red at
  compile time; say so and paste the compiler error.
- Every test carries `// verifies: <IDs>` on the line directly above
  `#[test]`/`#[tokio::test]`, naming the lowest level that exists (the LLR
  named in the step). A test that reproduces a problem report also names it
  (`// verifies: LLR-gdfqn9, PR-gnh3j2`).
- Clippy denies `unwrap_used`, `expect_used` and `panic` in library code.
  Test files open with
  `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- Names are concrete; no single-character names.
- A moved item (ruling A) moves its whole definition, `satisfies:`/`traces:`
  lines included, into the destination ledger under a dated "*Moved
  2026-10-xx from <unit> (`<file>`), plan task Tn*" note; the source ledger
  keeps a prose pointer to the destination file that does not name the ID
  (ADR `docs/adr/2026-10-06-org-io-unit.md`). An amended item gets a dated
  "*Amended 2026-10-xx (S4, task Tn):*" note under it with the new text.
  Its tests move in the same task, `verifies:` lines unchanged.
- A resolved problem report is closed in the task that lands its fix, in
  the form the `resolve-problem` skill states (copy the fields from a
  resolved report in the same ledger).
- Re-pinned golden bytes (`org-node/tests/encoding_golden.rs`) are derived by
  the format rule and written with a dated comment, never captured from the
  code, as the earlier re-pins in that file are.
- **Transitional states.** From T3 until T15–T17 the app does not compile
  against the new org-node and org-io API; its verify line is not a gate
  until T15. org-io must compile and pass its verify line after **every**
  task: each org-node task (T3, T4, T5, T6, T14) adapts org-io's call sites
  in the same commit. Until T9, org-io persists each `StoreWrite` through a
  private transitional `persist` that T3 writes (plain `std::fs::write` to
  `persona-<hex>.store` in its data directory, the behaviour org-node's
  `save` has today); T9 replaces its body. Until T10, org-node's transport
  binds from a `DeviceSecret` handed to the transitional
  `OrgService::bind_endpoint` (T3); T14 deletes it.
- `check-trace.sh` will report the moved items as DANGLING-REF or
  MISSING-TEST between the task that moves a test and the task that moves
  its item; it must be clean (MISSING-TEST for not-yet-implemented S4 items
  and pre-existing advisories only) after T6 for org-node, after T13 for
  org-io and after T17 for the app.
- `CARGO_HOME=/tmp/cargo_home_fuzz` when a task fetches a crate not yet in
  the lockfile (`~/.cargo` is read-only in the agent sandbox).

### T0 — Base: master with S2 merged

**Files touched:** none by hand except this plan's Progress section (a merge commit, then one plan commit)
**Parallel:** no (first; every task depends on it)

*Status 2026-10-08:* steps 0–4 are done. Master `e41cb09` (S2) was merged
into this branch at `24ca24c`; the re-reconciliation of step 0 and the
table of step 4 are §17 (commit "plan(org-io S4): reconcile with merged S2
(e41cb09) …"). S3b-io has not merged, so T11a drives its test double.
Remaining: step 5 (the baseline verify run). The `check-ids.sh
--allow-draft-files` and `check-trace.sh` part of step 5 was run on
2026-10-08 for org-io, org-node and the app (Progress).

0. **Re-reconcile against S2's merged code first** (added 2026-10-07). This
   plan was reconciled with S3a (§16) while S2 was still unmerged; S2 lands
   on top of S3a and will reshape the same receive, commit and store code.
   After step 3's merge, re-read every task's "Files touched", interfaces,
   tests and steps against the merged tree, as §16 did for S3a, and edit the
   tasks in place before any code is written. Check in particular: whether
   S2 left `OrgService::reconcile`, `commit_update` and the receive split
   async or value-driven; how S2 reduced org-node's `preflight.rs` and
   `bin/preflight.rs` (owner ruling 7: the transport check only,
   `ODS_DEVICE_SEED` only, gated on `transport`); where LLR-3zdw8v lives;
   and whether S3b-io (S3 plan T13–T17) has merged, in which case T11a
   meets its acknowledgement sender rather than a test double.
1. Confirm S2 (`worktree-org-io-create`) is merged into local master:
   `git log --oneline -3 master -- org-io/Cargo.toml` must show S2's squash
   commit. If it does not, stop and report: every task needs S2's crate.
2. `git merge-base --is-ancestor origin/master master` (local master is
   often ahead of origin; merge the local one).
3. `git merge master`. Expected conflicts: none in code (this branch has
   changed only docs); possibly `docs/plans/2026-10-06-org-io-roadmap.md`
   and `org-io/docs/*/README.md`: keep both sides' text. This branch's
   ledger files (named `<date>-<slug>.md` since 2026-10-14) never collide with S2's dated
   files.
4. Record the names S2 landed, in a table under "## Progress" headed "S2
   names as landed (T0)", by reading the merged code: `OrgService::new`'s
   argument; `commit_genesis` and `commit_update` signatures (S2 planned
   `chain_state: Option<OrgState>`); the receive split (S2 planned
   `receive_message`, `prepare_receive` → `PendingReceive` →
   `apply_receive`, `prepare_self_delete`/`apply_self_delete`); `OrgIo`'s
   fields and public methods (planned: `not_configured`, `connect`,
   `for_test`, `node`, `node_mut`, `found_organisation`,
   `submit_commit_send`, `receive_and_verify`,
   `receive_and_self_delete_if_revoked`, `is_own_admin`), and how S2
   carries S3a's sender, acknowledgement and `reconcile` paths through
   them (§16.1); `StateReader`,
   `ChainWriter`, `test_support::FakeChain`; org-node's remaining features;
   org-io's test targets, `verify_commands` and Makefile coverage targets;
   and the file and line of REQ-v4tfap. For every name that differs from
   this plan, write one dated line under the table: "S4 uses <landed name>
   where the plan says <planned name>". Later tasks use the landed names.
5. Baseline: run every unit's `verify_commands`, and
   `check-units.sh` with per-unit `check-trace.sh`/`check-ids.sh
   --allow-draft-files`. Expected: verify green; trace exit 0 for every
   unit but org-node and org-io, whose only findings are MISSING-TEST for
   the S4 drafts (and anything S2 left open, listed in its Progress), plus
   pre-existing UNRESOLVED-PR/UNMET-EXPECTATION advisories.
6. Commit the plan: `plan(org-io S4): T0 — S2 names as landed`.

### T1 — org-node: `DeviceSecret`, the move-only device seed

**Files touched:** org-node/src/keys.rs, org-node/tests/key_custody.rs
**Parallel:** yes, with T2 (T1 touches neither `lib.rs` nor `Cargo.toml`; tests reach the type as `org_node::keys::DeviceSecret`; the crate-root re-export comes in T3)
**IDs verified:** LLR-37k2s3, LLR-yp69jg

1. Red tests, appended to `org-node/tests/key_custody.rs`:

*(Code pruned at merge: 74 lines. Files touched: org-node/src/keys.rs, org-node/tests/key_custody.rs.)*

   Add a Clone/Copy/Display/Serialize probe in the form
   `an_x25519_key_pair_cannot_be_cloned` already uses in this file (the same
   technique, one assertion per trait), annotated `// verifies: LLR-37k2s3`.
   (`declares_zeroize_on_drop` exists in the file; use the `DevicePublicKey`
   accessor `device_key_wraps_the_verifying_key` uses if `as_bytes` is not
   it.)
2. Run `cargo test -p org-node --features app,test-support --test key_custody`.
   Expected red: `E0432 unresolved import org_node::keys::DeviceSecret`.
3. Implement in `org-node/src/keys.rs`:

*(Code pruned at merge: 22 lines. Files touched: org-node/src/keys.rs, org-node/tests/key_custody.rs.)*

   No `pub(crate)` byte accessor yet (T3 adds the one the store key and
   S3a's acknowledgement signing need). No public signing operation either:
   S3a signs through `SigningKeypair::sign` (`keys.rs`), and T3 builds that
   key pair from a `DeviceSecret` crate-internally, so the four-operation
   scan above holds. `DeviceSeed` and `SigningKeypair::device_seed` stay
   until T3 (S3a's `revocation.rs`, `reconcile.rs` and `commit_step` still
   take `DeviceSeed`s).
4. Green: the same command; then the org-node verify line and
   `cargo clippy -p org-node --all-targets --features app,test-support -- -D warnings`.
5. Commit: `feat(org-node): DeviceSecret, the move-only device seed (LLR-37k2s3, LLR-yp69jg)`.

### T2 — org-node: the Wire message codec leaves `transport/`

**Files touched:** org-node/src/wire.rs (new; the moved `transport/wire.rs`), org-node/src/transport/mod.rs, org-node/src/transport/endpoint.rs, org-node/src/transport/wire.rs (deleted), org-node/src/lib.rs, org-node/src/error.rs, org-node/src/revocation.rs (gates only), org-node/src/test_fixtures.rs (gate of `bob_absence_notice`), org-node/src/service.rs (imports only), org-node/Cargo.toml, org-node/tests/wire_frame_bound.rs, org-node/tests/fuzz_wire_decode/fuzz_target.rs, org-node/tests/revocation.rs (imports only), org-node/tests/encoding_golden.rs (imports only), org-node/tests/support/mod.rs (imports only), org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-07-org-key-pair.md, org-node/docs/architecture/2026-10-07-commit-workflow.md (LLR-dc45ur, LLR-378cj4 code paths), org-node/docs/requirements/2026-09-09-verify-and-commit.md, org-node/docs/risk/2026-09-09-org-node-hazards.md, org-node/docs/architecture/2026-10-08-wire-kinds.md (LLR-tajh9d), org-node/src/service.rs (`receive_message`'s mapping), org-io/tests/receive_order.rs and org-io/tests/common/handles.rs (imports only: `org_io::node::transport::wire::WireMessage` → `org_io::node::wire::WireMessage`), app/src-tauri/tests/submit_flow.rs and app/src-tauri/tests/invitation.rs (imports only, the same path; the app still compiles at T2)
**Parallel:** yes, with T1
**IDs verified:** LLR-er2x8n, LLR-sc6zuh, LLR-8kh3zf, LLR-pkruy8 (amended), LLR-tajh9d (S2's, superseding LLR-js9dsu), LLR-ecxc76, LLR-dc45ur, LLR-378cj4 (unchanged; S3a's three kinds)

*Reconciled with S2 (`e41cb09`, §17), 2026-10-08:* LLR-js9dsu is
superseded by LLR-tajh9d, and S2 already added the leftover-bytes test
for all three kinds (`exactly_three_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused`,
`verifies: LLR-tajh9d, LLR-js9dsu`), so step 1's planned
`bytes_left_over_after_any_kind_are_malformed` is **dropped**; that S2 test
is re-pointed to `decode_message` with its annotation unchanged.
`wire_frame_bound.rs` has 11 tests at `24ca24c`. org-io names the codec
only through the `node` re-export (`org_io::node::transport::wire`), in
two test files; `OrgService::receive_message` maps the transport's
`Malformed` to `MalformedMessage`, and keeps doing so over `decode_message`.

*Reconciled with S3a (§16.1):* the codec moves with its three kinds and
`WireMessage::org_id()`, and `decode_message` keeps S3a's refusal of bytes
left over after any kind (`postcard::take_from_bytes`, `Ok((message, []))`).
`revocation.rs` is gated on `transport` today because the Wire message
carries its `RevocationNotice` and `Acknowledgement`; once `wire.rs` leaves
`transport/`, those two types (and `Signature64`, `ACK_DOMAIN`) are
ungated, and the module's `app`-gated functions keep their gate.
`wire_frame_bound.rs` has 10 tests at `e0e4dac`.

1. Red tests in `org-node/tests/wire_frame_bound.rs` (the file stays; its
   `required-features` keeps `transport` until T10 moves `frame_round_trips`):

*(Code pruned at merge: 23 lines. Files touched: org-node/src/wire.rs (new; the moved `transport/wire.rs`), org-node/src/transport/mod.rs, org-node/src/transport/endpoint.rs, org-node/src/transport/wire.rs (deleted), org-node/src/lib.rs, org-node/src/error.rs, org-node/src/revocation.rs (gates only), org-node/src/test_fixtures.rs (gate of `bob_absence_notice`), org-node/src/service.rs (imports only), org-node/Cargo.toml, org-node/tests/wire_frame_bound.rs, org-node/tests/fuzz_wire_decode/fuzz_target.rs, org-node/tests/revocation.rs (imports only), org-node/tests/encoding_golden.rs (imports only), org-node/tests/support/mod.rs (imports only), org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-07-org-key-pair.md, org-node/docs/architecture/2026-10-07-commit-workflow.md (LLR-dc45ur, LLR-378cj4 code paths), org-node/docs/requirements/2026-09-09-verify-and-commit.md, org-node/docs/risk/2026-09-09-org-node-hazards.md, org-node/docs/architecture/2026-10-08-wire-kinds.md (LLR-tajh9d), org-node/src/service.rs (`receive_message`'s mapping), org-io/tests/receive_order.rs and org-io/tests/common/handles.rs (imports only: `org_io::node::transport::wire::WireMessage` → `org_io::node::wire::WireMessage`), app/src-tauri/tests/submit_flow.rs and app/src-tauri/tests/invitation.rs (imports only, the same path; the app still compiles at T2).)*

   Re-point every other test in the file from `encode_frame`/`decode_body` to
   `encode_message`/`decode_message` (and `TransportError::FrameTooLarge`
   to `OrgNodeError::MessageTooLarge { length }`, `TransportError::Malformed`
   to `OrgNodeError::MalformedMessage`), except
   `frame_round_trips` (LLR-fa7jt8), which keeps testing the transport's
   framing until T10 moves it. S3a's kind tests keep their names and
   `verifies:` lines: `the_three_kinds_round_trip_with_their_indices`,
   `malformed_revocations_and_acknowledgements_are_refused`,
   `an_org_information_body_with_trailing_bytes_is_refused`,
   `an_acknowledgement_signature_decodes_only_at_exactly_64_bytes`,
   `an_organisation_information_body_without_its_snapshot_or_key_does_not_decode`,
   and S2's `exactly_three_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused`.
   ~~Add `// verifies: LLR-js9dsu` `bytes_left_over_after_any_kind_are_malformed`~~
   (dropped 2026-10-08: S2's test above covers it, §17).
2. Run `cargo test -p org-node --features app,test-support --test wire_frame_bound`.
   Expected red: `E0432 unresolved import org_node::wire`.
3. Implement: move `transport/wire.rs` to `src/wire.rs` (no feature gate;
   `postcard` and `serde` are normal dependencies), `pub const MAX_MESSAGE:
   usize = 1024 * 1024;`, `encode_message` (refuses an encoding over the
   bound with `MessageTooLarge`, no prefix) and `decode_message` (refuses
   over the bound before decoding, `MalformedMessage` otherwise). Add
   `OrgNodeError::MessageTooLarge { length: usize }`. In `transport/`, keep
   `MAX_FRAME` as `crate::wire::MAX_MESSAGE` and move the prefix into
   `endpoint.rs`'s send (`len(u32 LE) ‖ encode_message(msg)?`) and its
   receive (`decode_message` on the body after the 4 bytes; `recv_one` still
   returns the authenticated sender with the message, as S3a made it). `lib.rs`:
   `pub mod wire;`, `pub use wire::WireMessage;`, and `pub mod revocation;`
   without the `transport` gate (its items that need the store stay under
   `app`); `test_fixtures::bob_absence_notice` loses its `transport` gate.
   `Cargo.toml`: `fuzz_wire_decode` loses `required-features` and
   re-targets `decode_message` in both of its parts (arbitrary bytes, and
   bytes after an Organisation-information prefix; the bytes it feeds are
   unchanged).
4. `encoding_golden.rs`: imports only. The pinned Wire message bytes must
   not change; if a pinned value was the framed form, stop and report.
5. Docs (dated notes, "S4, task T2"): SDD-kwncn7 (code list `transport/wire.rs`
   → `wire.rs`, "framing on the channel" → "the Wire message's encoding and
   bound"), LLR-er2x8n, LLR-sc6zuh, LLR-8kh3zf, LLR-pkruy8 (new names, no
   prefix, `OrgNodeError`), LLR-tajh9d (`decode_body`/`Malformed` →
   `decode_message`/`MalformedMessage`; S2's item, clarification),
   LLR-ecxc76, LLR-dc45ur and LLR-378cj4
   (code path; LLR-js9dsu, superseded, is left as written), REQ-eg5j8u
   and RC-gfn6kr ("wire frame" → "Wire message body"; the stream bound is
   org-io's REQ-szt6ea), HAZ-5f9jcm (the decoder is reached through org-io's
   stream read first; probability argument unchanged).
6. Green: org-node verify line, clippy, and org-io's verify line.
7. Commit: `refactor(org-node): the Wire message codec leaves transport (encode_message, decode_message)`.

### T3 — org-node: the sealed per-Persona store

*Reconciled with S2 (`e41cb09`, §17), 2026-10-08:* S2 left `commit_genesis`,
`commit_update` and `reconcile` synchronous with `chain_state:
Option<OrgState>`, so the "unless S2 already did it" branch below is moot;
org-io's store construction is `OrgIo::open` (see the org-io bullet of step
3); REQ-v4tfap's dated note and LLR-3zdw8v's `DeviceSecret` scan land here
(first org-io code to receive a `DeviceSecret`); S2's org-io files touched:
`org-io/src/lib.rs`, `org-io/src/view.rs` (if a summary changes),
`org-io/src/submit.rs`, `org-io/tests/{open,signatory_key,submit_flow,receive_order,absences}.rs`,
`org-io/tests/common/handles.rs`, `org-io/docs/requirements/2026-10-08-org-io.md`
(REQ-v4tfap), `org-io/docs/architecture/2026-10-08-org-io.md` (LLR-3zdw8v,
LLR-rgdx22), `org-node/docs/architecture/2026-10-08-values.md` (LLR-gwk4nk),
`org-node/docs/problems/2026-10-08-save-after-mutate.md` (PR-jav8zn, a
dated line).

**Files touched:** Cargo.toml (root: `[workspace.lints.rust] unused_must_use = "deny"`), Cargo.lock, org-node/Cargo.toml, org-node/src/store.rs, org-node/src/keys.rs, org-node/src/types.rs, org-node/src/lib.rs, org-node/src/error.rs, org-node/src/service.rs, org-node/src/revocation.rs, org-node/src/reconcile.rs, org-node/src/transport/endpoint.rs, org-node/src/test_fixtures.rs, org-node/src/preflight.rs and org-node/src/bin/preflight.rs (the transport-only preflight S2 leaves, owner ruling 7: it builds its key pair from `ODS_DEVICE_SEED` through `DeviceSecret::from_keychain_bytes` until T14 deletes it), org-node/tests/support/mod.rs, org-node/tests/revocation.rs, org-node/tests/reconcile.rs, org-node/tests/commit_workflow_errors.rs, org-node/tests/absences.rs, org-node/tests/sealed_store.rs (new), org-node/tests/fuzz_unseal/fuzz_target.rs (new), org-node/tests/store_at_rest.rs, org-node/tests/persona_records.rs, org-node/tests/organisation_key.rs, org-node/tests/service_lifecycle.rs, org-node/tests/commit_paths.rs, org-node/tests/provisional_store.rs, org-node/tests/encoding_golden.rs, org-node/tests/secret_redaction.rs, org-node/tests/key_custody.rs, every other org-node test file naming `.personas`, `PersonaStore::open`, `DeviceSeed` or `device_seed`, org-node/.guardrails/config.yaml, org-node/docs/architecture/soup.md, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-04-type-safety.md, org-node/docs/architecture/2026-10-06-chain-authority.md, org-node/docs/requirements/2026-09-09-verify-and-commit.md, org-node/docs/requirements/2026-10-04-type-safety.md, org-node/docs/risk/2026-09-09-org-node-hazards.md, org-io/docs/problems/2026-10-13-non-atomic-store-write.md (moved from org-node in T9) (pointer only), org-io/src/** and org-io/tests/** (call sites; the transitional `persist`)
**Parallel:** no (after T1 and T2)
**IDs verified:** LLR-bg538c, LLR-2qqqx6, LLR-bvt5up, LLR-rdda86, LLR-e5kqhg (new); LLR-8bum44, LLR-ayrdr8, LLR-scgk5j, LLR-wusj89, LLR-t4u66w, LLR-q5n28x, LLR-s78sh7, LLR-8mfjey, LLR-tev8h8, LLR-ctzkv7, LLR-68yd3j, LLR-9zfnmb, LLR-7cmp38, and S3a's LLR-hby4jr, LLR-uw7nmv, LLR-pba7yu, LLR-gr8x3r (amended, annotations kept); S2's LLR-3zdw8v (clarified: the `DeviceSecret` scan), LLR-rgdx22 (narrowed: no passphrase open), LLR-gwk4nk (clarified), REQ-v4tfap (narrowed, §17.3)

The interface (write it exactly; T4–T9 call it):

*(Code pruned at merge: 45 lines. Files touched: Cargo.toml (root: `[workspace.lints.rust] unused_must_use = "deny"`), Cargo.lock, org-node/Cargo.toml, org-node/src/store.rs, org-node/src/keys.rs, org-node/src/types.rs, org-node/src/lib.rs, org-node/src/error.rs, org-node/src/service.rs, org-node/src/revocation.rs, org-node/src/reconcile.rs, org-node/src/transport/endpoint.rs, org-node/src/test_fixtures.rs, org-node/src/preflight.rs and org-node/src/bin/preflight.rs (the transport-only preflight S2 leaves, owner ruling 7: it builds its key pair from `ODS_DEVICE_SEED` through `DeviceSecret::from_keychain_bytes` until T14 deletes it), org-node/tests/support/mod.rs, org-node/tests/revocation.rs, org-node/tests/reconcile.rs, org-node/tests/commit_workflow_errors.rs, org-node/tests/absences.rs, org-node/tests/sealed_store.rs (new), org-node/tests/fuzz_unseal/fuzz_target.rs (new), org-node/tests/store_at_rest.rs, org-node/tests/persona_records.rs, org-node/tests/organisation_key.rs, org-node/tests/service_lifecycle.rs, org-node/tests/commit_paths.rs, org-node/tests/provisional_store.rs, org-node/tests/encoding_golden.rs, org-node/tests/secret_redaction.rs, org-node/tests/key_custody.rs, every other org-node test file naming `.personas`, `PersonaStore::open`, `DeviceSeed` or `device_seed`, org-node/.guardrails/config.yaml, org-node/docs/architecture/soup.md, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-04-type-safety.md, org-node/docs/architecture/2026-10-06-chain-authority.md, org-node/docs/requirements/2026-09-09-verify-and-commit.md, org-node/docs/requirements/2026-10-04-type-safety.md, org-node/docs/risk/2026-09-09-org-node-hazards.md, org-io/docs/problems/2026-10-13-non-atomic-store-write.md (moved from org-node in T9) (pointer only), org-io/src/** and org-io/tests/** (call sites; the transitional `persist`).)*

   `expect_admission` on a node with several Personas needs to know which
   store records it: give it `persona_id` as T6's `build_reply` does
   (`expect_admission(rng, persona_id, org_id)`), and note the signature
   change in the task report.

1. Red tests, new target `org-node/tests/sealed_store.rs`
   (`required-features = ["app", "test-support"]`):
   - `// verifies: LLR-bg538c` `a_sealed_store_unseals_with_its_secret_and_device_key`: create a
     Persona through `OrgService::new(vec![])` + `create_persona`, then
     `PersonaStore::unseal(&write.sealed, write.device_key, secret)` gives a
     store whose `device_key()` is `write.device_key` and whose Persona's
     handle is the one created. (Keep the `DeviceSecret` by re-making it
     from bytes captured with `lend_to_keychain` before handing it over — the
     test plays the keychain.)
   - `// verifies: LLR-bg538c` `unseal_refuses_each_malformed_input_with_its_own_error`: 39
     bytes → `StoreRefused(StoreRefusal::TooShort)`; one ciphertext bit
     flipped → `StoreRefused(NotAuthentic)`; another Persona's secret →
     `DeviceSecretMismatch` (checked before decryption); a secret whose key
     matches but a store sealed (with `test_fixtures::seal_plaintext_for_test`)
     over bytes that do not decode → `StoreRefused(Undecodable)`; a parsed
     `persona.device_key` other than the argument → `DeviceSecretMismatch`.
   - `// verifies: LLR-2qqqx6` `the_store_key_is_hkdf_sha256_of_the_seed_under_the_versioned_label`:
     for seed `[0x07; 32]`, compute in the test
     `Hkdf::<Sha256>::new(Some(device_key_bytes), &seed).expand(b"ods/persona-store/v1", &mut key)`
     and decrypt the sealed bytes with `XChaCha20Poly1305::new(&key)`
     directly: the plaintext decodes. And the key is not the seed and not the
     ed25519 secret scalar.
   - `// verifies: LLR-2qqqx6` `no_argon2_and_no_passphrase_remain`: source scan of
     `org-node/src` and `org-node/Cargo.toml`: no `argon2`, `Argon2`,
     `passphrase`, `APP_SALT`.
   - `// verifies: LLR-bvt5up` `every_mutating_operation_returns_one_store_write_with_a_fresh_nonce`:
     run `create_persona`, `create_organisation`, `commit_genesis`,
     `discard_provisional`, `admit_member`, `revoke_member`, `commit_update`,
     `expect_admission` in a story over `test_fixtures::ChainSlots`' states
     (S2 removed `MockChainOps`; the states are values);
     each returns a `StoreWrite` for the Persona it changed, and the 24-byte
     nonces of consecutive writes differ; a refused operation (e.g.
     `create_organisation` for a bound Persona) returns `Err` and the next
     write's plaintext equals the one before the refusal.
   - `// verifies: LLR-bvt5up` `saving_needs_no_device_secret`: source scan:
     the only `pub fn` in `store.rs` and `service.rs` whose parameters name
     `DeviceSecret` are `unseal`, the transitional `bind_endpoint`, and the
     operations that may sign S3a's acknowledgements (`commit_update`,
     `reconcile`; `commit_checked` from T4), where it appears only inside the
     `device_secrets` closure's return type (`create_persona` returns one and
     takes none).
   - `// verifies: LLR-hby4jr, LLR-uw7nmv` (S3a's, restated over
     `DeviceSecret`) `a_commit_that_removes_this_node_signs_with_the_supplied_secret_and_reads_none_otherwise`:
     a commit that keeps this node listed never calls `device_secrets` (a
     closure that panics if called); one that removes it calls it once with
     the bound Persona's Device key, and each acknowledgement verifies under
     that key; a closure returning no secret is refused with
     `DeviceSecretNotSupplied` and returns no `StoreChange`.
   - `// verifies: LLR-pba7yu` `a_removal_returns_store_change_forgotten_for_that_personas_device_key`:
     `commit_update`'s removal and `reconcile`'s `Removed` return
     `StoreChange::Forgotten { device_key }` naming the removed Persona's
     key; the service no longer lists that Persona; every other store is
     unchanged.
   - `// verifies: LLR-bvt5up` `no_sealed_store_holds_the_device_seed`: the
     unsealed plaintext of every write in the story above does not contain
     the 32 seed bytes.
   - `// verifies: LLR-rdda86` `two_stores_with_one_device_key_are_refused`
     (`DuplicatePersonaStore`), `an_operation_changes_only_its_personas_store`
     (two Personas; `create_organisation` for one; the other's next sealed
     plaintext unchanged), `list_personas_returns_one_record_per_store`.
   - `// verifies: LLR-e5kqhg` `create_persona_returns_the_secret_and_keeps_only_the_public_key`:
     the returned secret's `device_key()` equals the record's `device_key`
     and the `StoreWrite`'s.
   - `// verifies: LLR-scgk5j` (amended) `the_store_key_has_a_redacted_debug`:
     keep today's assertion over the HKDF key (`test_fixtures` probe updated).
   - Fuzz target `tests/fuzz_unseal/fuzz_target.rs` (bolero, `harness = false`,
     `required-features = ["app", "test-support"]`), `// verifies: LLR-bg538c`:
     arbitrary bytes into `unseal` with a fixed secret; never panics, always
     `Err` or a store whose `device_key` is the argument.
2. Run `cargo test -p org-node --features app,test-support --test sealed_store`.
   Expected red: compile errors (no `SealedStore`, `StoreWrite`, `unseal`).
3. Implement:
   - `Cargo.toml` (org-node): remove `argon2`; add `hkdf = { version = "0.12", optional = true }`
     and `sha2 = { version = "0.10", optional = true }` to `app`.
   - root `Cargo.toml`: `[workspace.lints.rust] unused_must_use = "deny"`.
     Every crate inheriting the workspace lints then fails to build on an
     ignored `StoreWrite`; fix every site the compiler names.
   - `keys.rs`: `pub(crate) fn seed(&self) -> &[u8; 32]` on `DeviceSecret`
     (crate-private, so LLR-37k2s3's "no public operation" holds); remove
     `SigningKeypair::device_seed`; `types.rs`: remove `DeviceSeed`;
     `lib.rs`: re-export `keys::DeviceSecret`, `store::{SealedStore, StoreWrite}`.
   - `store.rs`: the interface above; the key is
     `Hkdf::<Sha256>::new(Some(device_key.as_bytes()), secret.seed()).expand(b"ods/persona-store/v1", ..)`
     into a `Zeroizing<[u8; 32]>`, then `XChaCha20Poly1305::new`; `seal` is
     today's private `seal` returning `StoreWrite`; `RawStoreData` mirrors
     the one-Persona shape; `seal_for_test` becomes
     `test_fixtures::seal_plaintext_for_test(secret, device_key, plaintext, rng) -> SealedStore`.
     Errors: `StoreRefused(StoreRefusal)` with `TooShort`, `NotAuthentic`,
     `Undecodable`; `DeviceSecretMismatch`; `DuplicatePersonaStore`.
   - `service.rs`: `stores: Vec<PersonaStore>`; route each operation to the
     store of the named Persona, or of the Persona bound to the named
     Organisation (`first_persona_bound_to` becomes `store_bound_to`);
     every `self.store.save(rng)?` becomes `self.store_mut(..).seal(rng)?`
     returned beside the result; a refusal returns before any change (move
     every check before the first mutation, or mutate a clone and swap it
     in on success). The device-seed reads (`persona_public_keys`,
     `still_member`, the receive path's own-Persona match, `persona_keys`,
     and S3a's `revocation::persona_device`, whose comment already names
     S4) read `persona.device_key`. `ensure_endpoint` becomes the transitional
     `bind_endpoint(persona_id, secret)`, which checks
     `secret.device_key() == persona.device_key` and binds from
     `SigningKeypair::from(secret)` built crate-internally.
   - S3a's signing (§16.1): `revocation::accept`, `revocation::sign_acknowledgements`,
     `reconcile::reconcile`, `commit_step` and `removal_step` take
     `Vec<DeviceSecret>` (or the closure above) in place of `Vec<DeviceSeed>`
     and sign through a crate-internal `SigningKeypair::from(&DeviceSecret)`;
     `device_seeds_bound_to` (`service.rs:1383`) is deleted. Every removing
     path returns `StoreChange::Forgotten` (one store per Persona); every
     other change returns `StoreChange::Sealed`.
   - S3a's absence scans in `absences.rs` (LLR-23sfdh):
     `only_forget_organisation_removes_records_and_personas` watches the
     `orgs` and `personas` collections; with one Persona per store the
     Persona is a single field and the collection that can lose a Persona is
     `OrgService`'s `stores`. Extend the scan, with a dated comment, to
     `stores` (removed only where the removal step's `StoreChange::Forgotten`
     is applied), keep `orgs`, and drop `personas`; the scan's
     `verifies: LLR-23sfdh` stays.
   - The transitional preflight (owner ruling 7): `preflight::check_transport`
     takes a `DeviceSecret`, which `bin/preflight.rs` builds from
     `ODS_DEVICE_SEED` with `DeviceSecret::from_keychain_bytes`, and binds
     through the transitional `OrgEndpoint::bind_with_mode(DeviceSecret, mode)`.
     (Ruling 12, 2026-10-08: an environment-variable seed is a
     test-environment path only. This one is org-node's pre-S4 operator
     tool, kept compiling only until T10 replaces it by org-io's keychain
     path and T14 deletes it; no S4 code adds an environment-variable key.)
   - `tests/support/mod.rs`: `open_store`/`reopen_store`/`store_bytes`
     become in-memory helpers over `StoreWrite`s: a `Keychain` test struct
     mapping `DevicePublicKey` to captured seed bytes, and a `Disk` map of
     the latest `SealedStore` per Persona that every helper updates from
     each returned write ("reaches the disk" assertions become "reaches the
     returned store write"). Rewrite the 37 `PersonaStore::open` call sites
     (§16.2) and the `.personas` uses onto them, and S3a's `device_seed_of`
     (`:633`) onto the test keychain; the closures S3a's `revocation.rs`,
     `reconcile.rs` and `commit_workflow_errors.rs` tests pass as seed
     sources return `DeviceSecret`s from it.
   - `encoding_golden.rs`: re-pin the store plaintext (one `persona`, not a
     list; `device_key` in place of `device_seed`; S3a's `kept_change_set`
     kept as S3a pinned it), derived by the format rule with a dated comment.
   - org-io (*reconciled with S2, `e41cb09`, §17*): S2's store construction
     is `OrgIo::open(data_dir, passphrase, transport_mode)`
     (`org-io/src/lib.rs:144`, `PersonaStore::open(data_dir/persona_store.bin,
     passphrase)`) and, in tests, `OrgService::new(PersonaStore::open(..))`
     in `tests/common/handles.rs`, `signatory_key.rs`, `open.rs`,
     `submit_flow.rs` and `receive_order.rs`. T3 makes `OrgIo::open(data_dir,
     transport_mode)` transitional: it creates the directory and starts a
     service with no stores when no `persona-<hex>.store` file exists, and
     refuses (naming the first file) when one does, because there is no
     keychain to unseal it with until T8 (T9 gives it the real listing,
     LLR-uzax55). `open.rs`'s
     `open_refuses_a_store_under_another_passphrase_and_opens_no_other`
     loses its behaviour here and is deleted, with LLR-rgdx22's dated
     narrowing note (its "opens `persona_store.bin` under the passphrase"
     clause withdrawn; §17.3) and the deletion recorded in the task report;
     the other three `open.rs` tests are adapted (call sites only, annotation
     unchanged). Tests replace the rest with `OrgService::new(vec![])`
     + `create_persona`. **`NodeBuilders::create_persona` keeps returning
     only the `PersonaId`** (allowlist, §17.2 rules 1–2): it moves the
     `DeviceSecret` org-node returns into a private, non-`Clone`
     transitional holder inside `OrgIo` (keyed by Device public key; no
     public accessor; `test-support` code may take a secret out of it by
     move to play the keychain), which T8/T9 replace with the keychain
     write. REQ-v4tfap gets its dated note in this task (moved here from
     T10, §17.3), and LLR-3zdw8v's scans (`org-io/tests/absences.rs`) gain
     `DeviceSecret` with its sanctioned places, by a dated clarification and
     a red-first test carrying LLR-3zdw8v; add the transitional private
     `fn persist(&self, write: StoreWrite) -> Result<(), String>` (a plain
     `std::fs::write` of `write.sealed` to
     `<data_dir>/persona-<hex>.store`), and call it for every `StoreWrite`
     an `OrgIo` method receives (for `StoreChange::Forgotten`, a plain
     `std::fs::remove_file` of that Persona's file, until T9); until T8's
     keychain module exists, org-io's `device_secrets` closures return the
     secrets its tests hold in a `test_support` map, and in non-test paths
     return none (a signing removal is then refused as
     `DeviceSecretNotSupplied`; T8 wires the keychain read in); tests that bind endpoints call
     `bind_endpoint` after `create_persona` (S2's `test-support`
     `NodeBuilders::ensure_endpoint` becomes `bind_endpoint`, taking the
     secret out of the transitional holder).
   - S2's `holds_provisional` (LLR-gwk4nk) asks the store of the Persona
     bound to `org_id` (dated clarification). S2's PR-jav8zn (a failed save
     leaves memory ahead of disk) is fixed on org-node's side here: no
     operation saves, and a refusal changes nothing; record the fix in its
     report, which T9 resolves with org-io's half.
4. Docs (dated notes, "S4, task T3"): SDD-af5vnt, LLR-wusj89, LLR-t4u66w,
   LLR-q5n28x, LLR-s78sh7, LLR-8mfjey, LLR-scgk5j, LLR-tev8h8, LLR-ctzkv7,
   LLR-sz4xhc, LLR-56hc77, LLR-ayrdr8, LLR-8bum44, LLR-68yd3j, LLR-9zfnmb,
   LLR-7cmp38, LLR-95753m, S3a's LLR-hby4jr, LLR-uw7nmv, LLR-pba7yu and
   LLR-gr8x3r (the seed source is `DeviceSecret`; a removal is
   `StoreChange::Forgotten`), REQ-y99c9w (S3a's own note says whichever of
   S3 and S4 merges second makes the signing take S4's transient type:
   S4 is second, so its dated note says it now does), REQ-hzm4kt, REQ-y7tsft, REQ-qn2erx, HAZ-45ucqx
   and RC-jjsz97 (re-assessed: passphrase and fixed-salt residuals gone;
   no file-permission control replaces them: the file's confidentiality
   rests on the keychain-derived key, owner ruling 2026-10-07), with the
   text of §3.2. SOUP:
   remove the `argon2` row; add `hkdf` 0.12.4 and `sha2` 0.10.9 rows
   (already in the lockfile through other crates: say which).
   PR-n696wv's report gets a dated line "fix lands in org-io, task T9".
5. Green: org-node verify line with `--test sealed_store --test fuzz_unseal`
   added (and the line written into `org-node/.guardrails/config.yaml`);
   clippy; org-io's verify line.
6. Commit: `feat(org-node): one sealed store per Persona, keyed by HKDF of the device secret; every change returns its StoreWrite`.

### T4 — org-node: one receive path, `check_received` then `commit_checked`

*Reconciled with S2 (`e41cb09`, §17), 2026-10-08:* S2 landed exactly the
planned split: `receive_message()` (transport-only, decodes),
`prepare_receive(sender, message: WireMessage) -> Prepared<ReceiveOutcome>`,
`PendingReceive<O>`, `apply_receive(rng, pending, chain_state)`, and the
self-delete pair. T4 renames the first three as below and moves the decode
into `check_received` (it takes `body: &[u8]`), deletes the self-delete
pair and `SelfDeleteOutcome`, and replaces `receive_message` by the
transitional `receive_body`. `reconcile` is already value-driven (no change
here). On org-io's side: `OrgIo::receive_and_verify` follows the new names,
`OrgIo::receive_and_self_delete_if_revoked` is deleted (LLR-7pj5af's dated
narrowing), `SelfDeleteOutcome` leaves `HANDLE_RETURN_ALLOWLIST`, and
`check_received`/`commit_checked` join the names
`the_handle_offers_no_chain_judging_operation_outside_its_own_sequence`
keeps off the handle. `receive_order.rs` has 12 tests (S2); those on the
self-delete path (`revoke_b`'s callers, and
`an_acknowledgement_on_the_self_delete_path_is_decided_with_no_read`) fold
onto the one path with their `verifies: LLR-7pj5af` unchanged.

**Files touched:** org-node/src/service.rs, org-node/src/revocation.rs, org-node/src/error.rs (if a variant is needed), org-node/src/transport/endpoint.rs, org-node/src/lib.rs, org-node/tests/receive_path.rs (new), org-node/tests/support/mod.rs, org-node/tests/admission_sender.rs, org-node/tests/receive_chain_reads.rs, org-node/tests/expected_admission.rs, org-node/tests/organisation_key.rs, org-node/tests/service_stories.rs, org-node/tests/commit_paths.rs, org-node/tests/revocation.rs, org-node/tests/reconcile.rs, org-node/tests/fuzz_first_admission_base/fuzz_target.rs, org-node/Cargo.toml, org-node/.guardrails/config.yaml, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-06-chain-authority.md, org-node/docs/architecture/2026-10-07-org-key-pair.md, org-node/docs/architecture/2026-10-07-commit-workflow.md, org-node/docs/requirements/*.md (REQ-8amu2a note), org-node/docs/risk/2026-10-06-chain-authority.md, org-node/docs/architecture/2026-10-08-values.md (LLR-mn5c2q), org-io/src/lib.rs, org-io/tests/receive_order.rs, org-io/tests/common/handles.rs, org-io/tests/absences.rs (allowlist and judging-name list), org-io/docs/architecture/2026-10-08-org-io.md (LLR-7pj5af), org-io/src/** and org-io/tests/** (other call sites)
**Parallel:** no (after T3)
**IDs verified:** LLR-zetn7a, LLR-6qjmhf (new); LLR-xn5pwc, LLR-ba2ejp, LLR-38e2kn, LLR-pt32fx, LLR-9ew26y, LLR-s8xp7m, LLR-b27jr6, LLR-eyc4ud, and S3a's LLR-2r2fha, LLR-kzgjz8, LLR-3aysup, LLR-3q63zv, LLR-r7zm39, LLR-tx8ruv, LLR-5azhry (amended); S2's LLR-7pj5af (narrowed), LLR-mn5c2q (clarified)

*Reconciled with S3a (§16.1), 2026-10-07.* S3a's member-sender rule (owner
ruling R1) needs the Device key the transport authenticated, so
`check_received` takes it: org-io passes the sender with the body (this
supersedes §2.2's "the authenticated sender key is not passed" and the old
test `neither_function_can_take_the_senders_key`). S3a's acknowledgement is
decided from the store alone, so `check_received` settles it with no chain
read; S3a's revocation notice is checked (`check_notice`, then the sender)
before the chain read and accepted (`revocation::accept`, signing) after it.
PR-q8r32t was resolved by S3a; `RevocationNotHeld` is raised only in
`revocation.rs`.

*(Code pruned at merge: 19 lines. Files touched: org-node/src/service.rs, org-node/src/revocation.rs, org-node/src/error.rs (if a variant is needed), org-node/src/transport/endpoint.rs, org-node/src/lib.rs, org-node/tests/receive_path.rs (new), org-node/tests/support/mod.rs, org-node/tests/admission_sender.rs, org-node/tests/receive_chain_reads.rs, org-node/tests/expected_admission.rs, org-node/tests/organisation_key.rs, org-node/tests/service_stories.rs, org-node/tests/commit_paths.rs, org-node/tests/revocation.rs, org-node/tests/reconcile.rs, org-node/tests/fuzz_first_admission_base/fuzz_target.rs, org-node/Cargo.toml, org-node/.guardrails/config.yaml, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-06-chain-authority.md, org-node/docs/architecture/2026-10-07-org-key-pair.md, org-node/docs/architecture/2026-10-07-commit-workflow.md, org-node/docs/requirements/*.md (REQ-8amu2a note), org-node/docs/risk/2026-10-06-chain-authority.md, org-node/docs/architecture/2026-10-08-values.md (LLR-mn5c2q), org-io/src/lib.rs, org-io/tests/receive_order.rs, org-io/tests/common/handles.rs, org-io/tests/absences.rs (allowlist and judging-name list), org-io/docs/architecture/2026-10-08-org-io.md (LLR-7pj5af), org-io/src/** and org-io/tests/** (other call sites).)*

1. Red tests, new target `org-node/tests/receive_path.rs`
   (`required-features = ["app", "test-support"]`), all value-driven (no
   endpoint): bodies are built in the test with `encode_message` from the
   sender's `CommitOutcome` and record (helper `body_to(svc, outcome,
   recipient)` in `tests/support`, replaced by the commit's own bodies in T5);
   the sender is a `DevicePublicKey` the test names (a listed Member's, or
   the Device of S3a's `ROGUE_SEED` for an unlisted one).
   - `// verifies: LLR-zetn7a` `a_body_that_does_not_decode_is_malformed_and_changes_nothing`.
   - `// verifies: LLR-zetn7a, LLR-38e2kn` `an_unheld_revocation_is_refused_before_any_expectation_check`.
   - `// verifies: LLR-zetn7a, LLR-s8xp7m` `an_unexpected_first_admission_is_refused`.
   - `// verifies: LLR-zetn7a` `the_checked_message_names_the_organisation_to_read`.
   - `// verifies: LLR-zetn7a` `a_checked_message_has_no_public_constructor`: source scan of
     `service.rs`: `pub struct CheckedMessage {` is followed by no `pub `
     field, and no `pub fn` outside `impl OrgService` returns `CheckedMessage`.
   - `// verifies: LLR-2r2fha` `an_update_for_a_held_organisation_from_an_unlisted_sender_is_refused_in_the_check`
     (`SenderNotListed`, no `Checked`, so no chain read can follow);
     `// verifies: LLR-2r2fha` `a_first_admission_is_accepted_from_any_sender`.
   - `// verifies: LLR-kzgjz8, LLR-r7zm39` `a_revocation_from_an_unlisted_sender_is_refused_in_the_check`.
   - `// verifies: LLR-3aysup, LLR-5azhry` `an_acknowledgement_is_decided_in_the_check_from_its_named_device_only`:
     from the Device it names, which the record no longer lists →
     `Checked::Acknowledged`, no store change; from any other sender →
     `AcknowledgementNotFromItsDevice`; from its Device while the record
     still lists it → `AcknowledgementForListedDevice`.
   - `// verifies: LLR-6qjmhf` one test per arm: `a_first_admission_commits_as_admitted`,
     `an_update_commits_as_updated`, `the_nodes_own_removal_forgets_the_organisation`
     (LLR-b27jr6; `StoreChange::Forgotten`, acknowledgements signed),
     `an_accepted_revocation_commits_as_revoked` (LLR-tx8ruv; acknowledgements
     signed with the supplied secret, `StoreChange::Forgotten`),
     `a_revocation_that_leaves_this_device_listed_is_refused`
     (LLR-pt32fx), `a_carried_key_that_does_not_match_the_chain_is_refused`
     (LLR-ba2ejp), `no_chain_state_is_org_not_on_chain`; each refusal asserts
     no `StoreChange`, `device_secrets` never called, and the stores' sealed
     plaintext unchanged.
   - `// verifies: LLR-6qjmhf` `only_the_check_takes_the_sender`: source scan:
     `check_received`'s parameter list names `DevicePublicKey` once, as
     `sender`; `commit_checked`'s names it only inside the `device_secrets`
     closure; neither names `EndpointId` or iroh's `PublicKey`.
   - `// verifies: LLR-zetn7a` `no_receive_branch_is_unreachable`: source scan:
     `RevocationNotHeld` is constructed only in `revocation.rs` (S3a's
     `check_notice`) and not in `service.rs` (PR-q8r32t, resolved by S3a,
     stays resolved).
2. Run; expected red: compile errors (no `check_received`).
3. Implement: rename S2's `prepare_receive`/`Prepared`/`PendingReceive`/`apply_receive`
   (§17.1) to `check_received`/`Checked`/`CheckedMessage`/`commit_checked`;
   `check_received(sender, body)` decodes with `decode_message` first
   (`MalformedMessage`), then per kind and with no chain state:
   `Acknowledgement` → `revocation::check_acknowledgement(store, sender,
   ack)` → `Checked::Acknowledged`; `Revocation` → `check_notice`, then
   `sender_listed` → `ReadChain`; `OrgInformation` → for a held Organisation
   `sender_listed`, for an unheld one the expectation gate, then the
   snapshot decode and `check_chain_free` → `ReadChain`. `commit_checked`
   runs the rest of today's `receive_revocation` and `receive_and_verify`
   against the state given. Delete `receive_one`, `receive_revocation`,
   `prepare_self_delete`, `apply_self_delete`, `SelfDeleteOutcome`,
   `receive_and_verify`/`receive_and_self_delete_if_revoked` if S2 left
   them, and S2's `receive_message` (replaced by `receive_body`). (S2 left
   `OrgService::reconcile` value-driven already, §17.1: no change.)
4. Rewrite the tests that drove receive through iroh (`spawn_receive`,
   `spawn_self_delete`, `spawn_recv_one`, `deliver`, `deliver_raw`,
   `deliver_from`, `send_from`, `receiving_addr`, `deliver_to_receive`,
   `deliver_to_self_delete`, `deliver_from_to_receive`,
   `deliver_from_to_self_delete`, `captured_admission`, `revoke_and_tell`,
   `setup_over_both`, `setup_counted`, `setup_counting`) onto a support helper
   `receive_from(svc, chain, sender, body) -> Result<(ReceiveOutcome, Option<StoreChange>), OrgNodeError>`
   (= `check_received`, then, for `ReadChain`, the mock chain's state for
   `org_id()` and `commit_checked` with the test keychain's secrets;
   `CountingChain` keeps counting the reads it makes). About 77 tests, 28 of
   them on the self-delete path (§16.2), fold onto the one path:
   where a test pinned a difference between the paths (LLR-379hnv's
   `OrgNotOnChain` for an unheld Organisation), keep only the one-path
   behaviour and say so in a comment. Keep every `verifies:` line, except
   that LLR-379hnv's annotations are removed with the item. Tests that
   exercised the dial itself stay on iroh until T10 moves them
   (`transport_handshake`, `transport_networked`, and the five dial tests of
   §5 in `admission_sender.rs`/`commit_paths.rs`: give them the
   transitional `receive_body` + `check_received`).
5. org-io: S2's `receive_and_verify` becomes `receive_body` →
   `check_received(sender, body)` → (for `ReadChain`) read →
   `commit_checked` → `persist`; S2's `receive_and_self_delete_if_revoked`
   is deleted, its tests folded onto the one path (`receive_order.rs`'s 12
   tests keep their `verifies: LLR-7pj5af`; LLR-7pj5af gets its dated
   narrowing note); `SelfDeleteOutcome` leaves `HANDLE_RETURN_ALLOWLIST`, and
   `fn check_received` joins the judging names
   `the_handle_offers_no_chain_judging_operation_outside_its_own_sequence`
   scans for (its existing `fn commit_` already covers the second; say so).
6. Docs (dated notes, "S4, task T4"): LLR-xn5pwc, LLR-ba2ejp, LLR-38e2kn,
   LLR-pt32fx, LLR-9ew26y, LLR-s8xp7m, LLR-b27jr6, LLR-eyc4ud, LLR-3q63zv,
   S3a's LLR-2r2fha, LLR-kzgjz8, LLR-3aysup (the sender now arrives as
   `check_received`'s argument, from org-io; the rule is unchanged),
   LLR-r7zm39, LLR-tx8ruv, LLR-5azhry (called from `check_received` and
   `commit_checked`), SDD-8cpyfa (its S3a note "afterwards org-io passes it in
   with the message" is now the fact), SDD-72ddm6, REQ-8amu2a, RC-mj6gjq,
   RC-2ferct, HAZ-bedm57 (the two-unit statement, §2.4); LLR-379hnv
   **retired** (superseded in place, pointing to LLR-38e2kn); S2's
   LLR-mn5c2q (`commit_checked` in place of `apply_receive`/`apply_self_delete`)
   and LLR-7pj5af (one method, §17.3).
7. Green: org-node verify line with `--test receive_path`; clippy; org-io's
   verify line.
8. Commit: `feat(org-node): one receive path by values — check_received(sender, body), then commit_checked`.

### T5 — org-node: the commit returns every recipient's body (PR-gnh3j2)

**Files touched:** org-node/src/service.rs, org-node/src/error.rs (`NoRevocationForRecipient` goes), org-node/src/transport/endpoint.rs, org-node/src/lib.rs, org-node/tests/outgoing_bodies.rs (new), org-node/tests/support/mod.rs, org-node/tests/commit_paths.rs, org-node/tests/admission_sender.rs, org-node/tests/service_stories.rs, org-node/tests/organisation_key.rs, org-node/tests/revocation.rs, org-node/tests/commit_workflow_errors.rs, org-node/tests/secret_redaction.rs, org-node/Cargo.toml, org-node/.guardrails/config.yaml, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-07-org-key-pair.md, org-node/docs/architecture/2026-10-06-chain-authority.md, org-node/docs/architecture/2026-10-07-commit-workflow.md, org-node/docs/problems/2026-10-07-review-findings.md, org-io/src/submit.rs, org-io/src/lib.rs, org-io/src/view.rs (`Submitted`), org-io/tests/submit_flow.rs, org-io/tests/absences.rs (allowlist), org-io/src/** and org-io/tests/** (other call sites)
**Parallel:** no (after T4)
**IDs verified:** LLR-gdfqn9 (new); LLR-6ymd6d, LLR-8hdu9x, LLR-bg3vsw, LLR-t4znbk (org-node half), LLR-48jakr, LLR-2xzys9, and S3a's LLR-a8z7r5, LLR-kr5t6f (amended); resolves PR-gnh3j2

*Reconciled with S3a (§16.1), 2026-10-07.* S3a's `CommitOutcome` already
carries the notices of exactly the committed update (`revocations`,
LLR-a8z7r5) and the acknowledgements a self-removing commit signed, and its
`send_update(&CommitOutcome, recipient, peer_addr)` sends a notice to a
Device the outcome holds one for and refuses any other unlisted Device
(`NoRevocationForRecipient`). T5 builds both kinds of body inside the
commit: a revocation body is `WireMessage::Revocation(notice)` with its
absence proof (LLR-dc45ur), not an Envelope. A commit that removes this node
forgets the Organisation (S3a's one removal step), so it returns its
acknowledgements and no bodies. With `send_update` gone there is no
caller-named recipient left to refuse, so `NoRevocationForRecipient` goes
too (the app's match on it in T15).

*(Code pruned at merge: 12 lines. Files touched: org-node/src/service.rs, org-node/src/error.rs (`NoRevocationForRecipient` goes), org-node/src/transport/endpoint.rs, org-node/src/lib.rs, org-node/tests/outgoing_bodies.rs (new), org-node/tests/support/mod.rs, org-node/tests/commit_paths.rs, org-node/tests/admission_sender.rs, org-node/tests/service_stories.rs, org-node/tests/organisation_key.rs, org-node/tests/revocation.rs, org-node/tests/commit_workflow_errors.rs, org-node/tests/secret_redaction.rs, org-node/Cargo.toml, org-node/.guardrails/config.yaml, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-07-org-key-pair.md, org-node/docs/architecture/2026-10-06-chain-authority.md, org-node/docs/architecture/2026-10-07-commit-workflow.md, org-node/docs/problems/2026-10-07-review-findings.md, org-io/src/submit.rs, org-io/src/lib.rs, org-io/src/view.rs (`Submitted`), org-io/tests/submit_flow.rs, org-io/tests/absences.rs (allowlist), org-io/src/** and org-io/tests/** (other call sites).)*

`reconcile`'s `Reconciled::Committed { outcome }` carries the same
`CommitOutcome`, so a commit found at startup fans out the same bodies (S3
proposed 3 and 5, S3b-io).

1. Red tests, new target `org-node/tests/outgoing_bodies.rs`
   (`required-features = ["app", "test-support"]`):
   - `// verifies: LLR-gdfqn9` `an_admission_sends_information_to_every_listed_device_but_the_senders`:
     three Members (A sender, B, C); admit D; `outgoing` recipients are B,
     C, D in record order, each body `decode_message`s to `OrgInformation`
     with the Envelope, the pre-commit snapshot and the key the commit
     stored.
   - `// verifies: LLR-gdfqn9, LLR-a8z7r5` `a_removal_sends_revocations_to_every_removed_device_after_the_listed_ones`:
     each removed Device's body decodes to `Revocation(notice)` naming that
     Device, its MemberId and an absence proof that verifies against the
     committed root (S3a's `notices_for`, LLR-kr5t6f).
   - `// verifies: LLR-gdfqn9, LLR-b27jr6` `a_commit_that_removes_this_node_returns_its_acknowledgements_and_no_bodies`
     (`StoreChange::Forgotten`).
   - `// verifies: LLR-gdfqn9` `commit_genesis_returns_no_recipients`.
   - `// verifies: LLR-gdfqn9` `a_reconciled_commit_returns_the_same_bodies_as_commit_update`.
   - `// verifies: LLR-gdfqn9, PR-gnh3j2` `a_later_commit_cannot_change_an_earlier_body`:
     commit update 1 and keep its outcome; commit update 2 (a new
     Organisation key); update 1's bodies still decode to update 1's key
     (before T5 this was read when `send_update` ran: write the test against
     today's `send_update(&outcome, ..)` first if it still exists, to show the red).
   - `// verifies: LLR-gdfqn9` `outgoing_debug_shows_no_body_byte`: `format!("{:?}", outcome)`
     contains neither the key's bytes nor any 8-byte window of a body.
   - `// verifies: LLR-gdfqn9` `outgoing_types_are_not_clone`: the probe used in T1.
   - `// verifies: LLR-gdfqn9` `building_the_bodies_writes_and_sends_nothing`:
     exactly one `StoreChange` per commit; no endpoint is bound by
     `commit_update` (`endpoint()` still `None`).
2. Run; expected red: compile errors (`outgoing` is an `OutgoingUpdate`).
3. Implement: build the bodies inside `commit_step` (so `commit_update` and
   `reconcile` share them, LLR-fm38ww) from the committed record
   (LLR-6ymd6d's kind rule, the key the commit stored, S3a's notices); no
   bodies when the step removed this node; delete `OutgoingUpdate`, the
   `revocations` field, `send_update` and `NoRevocationForRecipient`; add
   the transitional `send_body`.
4. Rewrite `commit_paths.rs`'s kind tests (S3a's included: a notice to a
   removed Device, the refusal of an unlisted, un-noticed Device, which
   becomes "no body for it") over the bodies; the dial halves (§5) use
   `send_body` until T10/T12 move them. Replace T4's `body_to` helper with
   the outcome's bodies.
5. org-io: S2's `submit_commit_send(rng, &BuiltUpdate, recipient, peer_addr)`
   (`org-io/src/submit.rs:200`) picks `outcome.outgoing`'s body for
   `recipient` and calls `send_body` (the fan-out comes in T12); `persist`
   the commit's `StoreChange` first. *Reconciled with S2, 2026-10-08
   (§17.2 rule 2):* an `OrgInformation` body carries the Organisation
   private key, so from this task `CommitOutcome` is **not** returned
   through the handle: `OrgIo::submit_commit_send` returns a
   `Submitted { org_id, epoch, root }` of public data (T12 adds the
   per-Device report), `CommitOutcome` leaves `HANDLE_RETURN_ALLOWLIST`
   and `Submitted` joins it, and a red-first test in `absences.rs`
   (`// verifies: LLR-3zdw8v, LLR-gdfqn9`) fails if any handle method
   returns a type holding `OutgoingMessage`. S2's pre-write guard
   (`the_pre_write_guard_copies_no_record_and_no_held_update`, LLR-qhjp6g)
   stays green.
6. Docs (dated notes, "S4, task T5"): LLR-6ymd6d ("the commit builds, for
   each recipient, …" with the key captured at commit), SDD-rx2yvy (send
   goes; SDD-bm27xt holds the bodies), LLR-8hdu9x, LLR-bg3vsw, LLR-t4znbk
   (split: org-node half here), LLR-48jakr ("no outgoing body carries an
   invite identifier"), LLR-2xzys9 (org-node names the sending Persona;
   org-io binds), S3a's LLR-a8z7r5 (the notices become bodies; the field
   goes); PR-gnh3j2 resolved (owner ruling, 2026-10-07).
7. Green: org-node verify line with `--test outgoing_bodies`; clippy; org-io's line.
8. Commit: `feat(org-node): the commit returns every recipient's body, built with that commit's key (resolves PR-gnh3j2)`.

### T6 — org-node: the Invite, the reply and their bookkeeping

**Files touched:** org-node/Cargo.toml (`base64`), Cargo.lock, org-node/src/invitation.rs (new), org-node/src/store.rs, org-node/src/service.rs, org-node/src/error.rs, org-node/src/lib.rs, org-node/tests/invitation.rs (new), org-node/tests/fuzz_invitation_parse/fuzz_target.rs (new), org-node/tests/absences.rs, org-node/tests/encoding_golden.rs, org-node/tests/expected_admission.rs, org-node/.guardrails/config.yaml, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-06-chain-authority.md, org-node/docs/architecture/soup.md, org-node/docs/requirements/ (REQ-yazum3 arrives), org-node/docs/architecture/ (LLR-b7wgpf arrives), org-node/AGENTS.md, app/docs/requirements/2026-10-06-invitation.md, app/docs/architecture/2026-10-06-invitation.md, app/src-tauri/tests/invitation.rs (the three LLR-b7wgpf tests leave), org-io (call sites of `expect_admission`: S2's `NodeBuilders::expect_admission`, `org-io/src/lib.rs:111`, which takes the `persona_id` T3 added)
**Parallel:** no (after T5 and T7: both change `Cargo.lock`)
**IDs verified:** LLR-nr8df3, LLR-7q9g9z, LLR-d55brn, LLR-yexf5m (new; LLR-yexf5m added 2026-10-13); LLR-b7wgpf, REQ-yazum3 (moved from the app); LLR-ms8njy, LLR-48jakr, LLR-ayrdr8, LLR-95753m (amended)

1. Move `InviteId`, `Invite`, `InviteReply`, `WireInvite`, `WireReply`,
   `armour`/`unarmour`, `parse`/`encode` and `wire_for_test` from
   `app/src-tauri/src/invitation.rs:18-190` (S2 edited the file; re-read the
   lines at `24ca24c`) to `org-node/src/invitation.rs`
   (`OutstandingInvites` and the IO do not move: the store holds the pairs).
   Move the three LLR-b7wgpf tests (`app/src-tauri/tests/invitation.rs`,
   the ones annotated LLR-b7wgpf at lines 144, 154, 186) to
   `org-node/tests/invitation.rs`, `verifies:` unchanged, with REQ-yazum3
   and LLR-b7wgpf moving into org-node's ledgers (ruling A).
2. Red tests in `org-node/tests/invitation.rs`
   (`required-features = ["app", "test-support"]`):
   - `// verifies: LLR-nr8df3` `build_invite_records_the_pair_and_carries_the_bound_personas_device_key`;
     `build_invite_refuses_an_organisation_no_persona_is_bound_to` (`OrgNotOnChain`, no write).
   - `// verifies: LLR-7q9g9z` `build_reply_records_the_expectation_once_and_carries_the_personas_keys`;
     `build_reply_refuses_a_bound_persona` (`PersonaAlreadyBound`, no write);
     `build_reply_refuses_an_unknown_persona`.
   - `// verifies: LLR-d55brn` `check_reply_refuses_another_organisation_and_an_id_not_outstanding`
     (both errors; an id outstanding for another Organisation included);
     `settle_removes_exactly_the_pair_and_refuses_one_not_held`.
   - *Added 2026-10-13 (owner ruling: an individual is a member of an
     Organisation only once, per node; REQ-uy7kfq):* `// verifies: LLR-yexf5m`
     `build_reply_refuses_an_organisation_another_persona_holds` (a node
     with Personas A and B; A founds or is admitted to the Organisation;
     B's reply to an Invite for it is refused
     `OrganisationHeldByAnotherPersona`, with no `StoreWrite` and both
     stores byte-identical);
     `build_reply_refuses_an_organisation_another_persona_expects` (A's
     reply recorded the expectation; B's reply for the same Invite's
     Organisation is refused, nothing written);
     `the_same_persona_replying_twice_is_not_refused_by_the_one_membership_rule`
     (the boundary: A replies twice and the expectation is recorded once);
     `expect_admission_refuses_the_same_two_cases` (for as long as
     `expect_admission` exists; deleted with it in T13 if T13 removes it);
     and a `Display` check that the error names the Organisation and no
     Persona identifier. Before writing these tests, run `analyze-risks`
     on REQ-uy7kfq: does the stale-copy state it prevents, or the refusal
     itself (a person who wants a second Persona in one Organisation), touch
     an existing hazard in `org-node/docs/risk/`? Record the outcome as an
     `assesses:` entry or a dated note in the risk file.
   - `// verifies: LLR-ayrdr8` the store golden re-pinned with
     `outstanding_invites` (format rule, dated comment).
   - Fuzz `fuzz_invitation_parse` (bolero), `// verifies: LLR-b7wgpf`:
     arbitrary text into `Invite::parse` and `InviteReply::parse`; never
     panics; every `Err` names a field.
3. Run; expected red: compile errors.
4. Implement per LLR-nr8df3/7q9g9z/d55brn/yexf5m (the error
   `OrganisationHeldByAnotherPersona { org_id }`; the app's exhaustive
   `classify_receive_error` match gains its arm in T15, when the app
   compiles again);
   `StoreData.outstanding_invites:
   Vec<(OrgId, InviteId)>`; errors `ReplyForAnotherOrganisation`,
   `InviteNotOutstanding`; `base64` 0.22 (the app's version) under `app`.
   `build_reply` replaces `expect_admission` for the reply path; keep
   `expect_admission` only if org-io still needs it after T13 (otherwise
   remove it in T13). *Reconciled with S3a:* S3a's
   `StoreData::forget_organisation` (LLR-pba7yu) removes everything of an
   Organisation; it now also drops that Organisation's
   `outstanding_invites`. Add `// verifies: LLR-pba7yu`
   `forgetting_an_organisation_drops_its_outstanding_invites` to
   `org-node/tests/invitation.rs`, and a dated note on LLR-pba7yu. (S3a's
   scan `only_forget_organisation_removes_records_and_personas` watches the
   record and Persona collections only, so `settle`'s removal of a pair is
   not caught by it.)
5. `absences.rs`: narrow `org_node_holds_no_invite_identifier` to
   `wire.rs` and `ExpectedAdmission` (`assert_absent_in("wire.rs", "invite_id", ..)`,
   and the `ExpectedAdmission` struct body), annotation unchanged (LLR-ms8njy,
   LLR-48jakr); invert `org_node_holds_no_invitation_exchange`'s
   `export_invite`/`import_invite` lines into a presence check of
   `fn build_invite`/`fn build_reply`, and keep the join-request absences.
6. *Retirement, as the owner ruled 2026-10-13 (T4 report):* for
   **LLR-379hnv** (retired in T4) and **LLR-g9vmbx** (retired here),
   delete the `**LLR-…**:` definition and remove the ID from every
   `assesses:` line, then rewrite every remaining mention, in docs and test
   comments across all units, as prose that does not name the ID (e.g. "the
   retired self-delete-path item (S4 T4; see the S4 verification record)").
   `grep -rn 'LLR-379hnv\|LLR-g9vmbx'` must then find only this plan.
   *Clarified 2026-10-13 (dispatcher, raised by T6): "only this plan" means
   nothing in the units; the repo-level `docs/plans/` and the merged,
   signed `docs/verification/` records are history and keep the IDs.
   check-trace scans neither. The same scope holds for LLR-q225ws in T12.*
   Expected: no DANGLING-REF, and no MISSING-TEST for either.
   Docs (dated notes, "S4, task T6"): SDD-vee2fq rewritten, LLR-g9vmbx
   retired (as above), LLR-8qxwst and LLR-zj88e6 rewritten (§3.3), LLR-ms8njy narrowed,
   LLR-95753m, SDD-xshv5b's `traces:` gains REQ-yazum3; SOUP `base64` row
   (returns to org-node), `postcard`/`serde` users; `org-node/AGENTS.md`
   edges paragraph (§3.2 last rows).
7. Green: org-node verify line with `--test invitation --test fuzz_invitation_parse`;
   clippy; org-io's line; `check-trace.sh` for org-node clean but for
   MISSING-TEST of items later tasks implement.
8. Commit: `feat(org-node): the Invite and its reply return to org-node, outstanding ids in the sealed store`.

### T7 — org-io: SOUP for `keyring` and `zeroize`, and the mock's sharing

**Files touched:** org-io/Cargo.toml, Cargo.lock, org-io/docs/architecture/soup.md, org-io/docs/architecture/2026-10-07-transport.md (LLR-36jer8's outcome line and feature list), docs/plans/2026-10-06-org-io-roadmap.md (the later-stage rows of step 4), this plan's Progress
**Parallel:** yes, with T4 and T5 (org-io manifest and docs only; T4/T5 touch org-io sources, not its manifest)
**IDs verified:** none (assessment); prepares LLR-36jer8

1. Choose the current `keyring` 3.x release (pin exactly, `=3.x.y`), with
   `default-features = false` and `features = ["apple-native",
   "windows-native", "sync-secret-service"]` (narrowed by step 4) plus whatever crypto feature
   that release requires for the Secret Service backend (`crypto-rust` in
   3.x); `zeroize` is already `"1.9"` in org-io's manifest (S2): add
   `features = ["zeroize_derive"]` only if derive is used. Fetch with `CARGO_HOME=/tmp/cargo_home_fuzz`.
2. **The mock-sharing probe** (owner ruling 3, 2026-10-07): in a scratch
   test file (not committed), under `keyring::set_default_credential_builder(keyring::mock::default_credential_builder())`,
   create `Entry::new("ods-device-seed", "probe")` twice, `set_secret(b"x")`
   on the first and `get_secret()` on the second. Record the result and the
   keyring version in the Progress notes and in the SOUP row: "shared" →
   LLR-36jer8 uses keyring's mock; "not shared" (`NoEntry` on the second
   handle) → org-io supplies `test_support::MemoryKeychain` in T8. Edit
   LLR-36jer8's last sentence to state which.
3. SOUP rows (org-io `soup.md`, in the file's format): `keyring` (version,
   licence, the three backends, the risk cell of §4: a plaintext or mock
   backend in a release build defeats RC-d8dbxs, pinned by LLR-36jer8's
   scan; Linux Secret Service exposes an unlocked collection to any
   process of the user, the accepted HAZ-5n56w2 residual; known anomalies
   from its changelog and issue tracker at that version); `zeroize`: S2
   already made it a direct org-io dependency with a SOUP row
   (`org-io/docs/architecture/soup.md:31`, the development seed), so T7
   only widens that row's role to the keychain buffers (and the
   `zeroize_derive` feature if used). S2's `iroh` row (`:38`, the peer
   address type passed through to org-node) is replaced in T10 by
   org-node's whole iroh row. Add the dependency-edge
   assessment org-io → keyring ("Declaring a dependency", as S2's §5 did
   for its edges).
4. **Platform coverage** (owner ruling 5, 2026-10-07: Windows, macOS,
   Linux, Android, iOS). From the chosen release's documentation and
   source, record in the `keyring` SOUP row and in Progress, per platform:
   the backend (feature name) or "none". Expected, to be verified: macOS
   and iOS through `apple-native`, Windows through `windows-native`, Linux
   through `sync-secret-service`, Android **none**. Then:
   - **Android:** if the crate has no backend, record the gap, and add an
     unminted "(later)" row to the roadmap's stage table
     (`docs/plans/2026-10-06-org-io-roadmap.md`, beside S8 and the recovery
     row) proposing an Android Keystore-backed credential builder for
     org-io (a `keyring` `CredentialBuilderApi` over the Keystore through
     JNI, or a separate maintained crate if one is found and passes SOUP
     review). In S4 an Android build has no keychain and refuses under
     REQ-6d9xv6. Only if a maintained backend already exists and drops in as
     one more feature, include it in S4 and say so.
   - **Windows:** build and run T8's tests against `windows-native` only as
     far as this environment allows (cross-check `cargo check --target
     x86_64-pc-windows-msvc -p org-io` if the target is installed). If it
     adds complexity beyond enabling the feature (a second code path, a
     credential-size limit the 32-byte seed hits, a build dependency the
     SOUP review rejects), defer it: drop `windows-native` from the
     features, record the reason, and add an unminted "(later)" roadmap
     row for Windows keychain support beside the Android one. Either way
     LLR-36jer8's feature list is edited to the backends S4 ships.
   - No SOUP item for file ACLs (owner ruling 4).
5. Verify: `cargo build -p org-io`; org-io's verify line (no code change).
6. Commit: `docs(org-io): SOUP for keyring and zeroize; platform coverage and the mock's sharing, probed`.

### T8 — org-io: the keychain module

**Files touched:** org-io/src/keychain.rs (new), org-io/src/lib.rs (module line, and the transitional holder of T3 replaced by the keychain write), org-io/src/test_support.rs (S2's file, `FakeChain`; the keychain facade and, if needed, `MemoryKeychain` join it), org-io/tests/keychain.rs (new), org-io/tests/absences.rs, org-io/Cargo.toml (`[[test]]` entry), org-io/.guardrails/config.yaml
**Parallel:** yes, with T4 and T5 (after T7). It shares only `org-io/src/lib.rs`'s module list with them; keep both lines when merging.
**IDs verified:** LLR-677fqq, LLR-8erfbu, LLR-36jer8, LLR-n9dnm4 (the keychain write)

1. If T7 found keyring's mock does not share items:
   `test_support::MemoryKeychain` — a `keyring::credential::CredentialBuilderApi`
   whose credentials are views onto one `Arc<Mutex<HashMap<(String, String), Vec<u8>>>>`,
   with switches `refuse_as: Mutex<Option<Refusal>>` (`Locked` →
   `keyring::Error::NoStorageAccess`, `Unavailable` →
   `keyring::Error::PlatformFailure`) and `pub fn install(&self)` calling
   `keyring::set_default_credential_builder`. Tests that install it run in
   one test binary, serialised (a `static LOCK: Mutex<()>`), because the
   builder is process-global.
2. Red tests, new target `org-io/tests/keychain.rs`
   (`required-features = ["test-support"]`):
   - `// verifies: LLR-677fqq` `a_written_secret_reads_back_as_the_same_device_key`
     (`write_device_secret` then `read_device_secret`).
   - `// verifies: LLR-677fqq` `the_item_is_named_for_the_device_key`: service
     `ods-device-seed`, user the 64 lower-case hex of the key.
   - `// verifies: LLR-677fqq` `a_secret_under_another_device_key_is_refused_as_key_mismatch`.
   - `// verifies: LLR-677fqq` `with_endpoint_seed_lends_the_seed_once_and_refuses_a_short_item`.
   - `// verifies: LLR-8erfbu` `a_missing_item_is_item_missing`,
     `a_locked_keychain_is_locked`, `any_other_failure_is_unavailable_with_no_secret_byte_in_its_reason`.
   - `// verifies: LLR-n9dnm4` `the_write_lends_the_bytes_only_through_lend_to_keychain`:
     source scan of `keychain.rs`: `set_secret` appears once, inside a
     `lend_to_keychain(` call.
   - In `org-io/tests/absences.rs`: `// verifies: LLR-677fqq, LLR-36jer8`
     `keyring_is_named_only_in_the_keychain_module_and_its_mock_only_under_test_support`:
     scan `org-io/src`: `keyring::` only in `keychain.rs` and
     `test_support.rs`; `mock`/`MemoryKeychain` only in `test_support.rs`,
     which `lib.rs` declares under `#[cfg(feature = "test-support")]`;
     `org-io/Cargo.toml`'s keyring features are exactly the three backends
     (and the crypto feature T7 recorded). And
     `// verifies: LLR-677fqq` `no_lower_unit_depends_on_a_keychain_crate`:
     the `[dependencies]` of `org-node/Cargo.toml`, `person/Cargo.toml`,
     `org-members/Cargo.toml` and `on-chain-client/Cargo.toml` name no
     `keyring`, `security-framework`, `windows` credential or
     `secret-service` crate (REQ-hw82ue, as amended 2026-10-07). And
     `// verifies: LLR-677fqq` `keychain_bytes_reach_no_clone_debug_or_serialise`:
     no `.clone()`, `{:?}`, `to_vec`, `Serialize` on a line of
     `keychain.rs` that names `get_secret` or the lent bytes.
   - *Added 2026-10-13 (T7's finding):* keyring 3.6.3 silently makes its
     non-persistent in-memory mock the default store on any target with no
     native backend enabled (Android among them), so an Android build would
     seem to work and lose the seed at exit (HAZ-eyene4) instead of refusing
     (REQ-6d9xv6). `// verifies: LLR-8erfbu`
     `a_target_without_a_native_backend_refuses_as_unavailable`: a source
     scan of `keychain.rs`. Every keyring call sits under a
     `cfg(any(target_os = "macos", target_os = "ios", target_os = "windows",
     target_os = "linux"))` path, and the complementary `cfg(not(any(…)))`
     path of `read_device_secret`, `write_device_secret` and
     `with_endpoint_seed` returns `KeychainError::Unavailable` without
     calling keyring. (No Android target is installed here, so a scan is
     the only test that can run. The `test-support` mock install is
     unaffected, because it runs on the host.) Add a dated T8 note under
     LLR-8erfbu.
3. Run; expected red: compile errors (no `org_io::keychain`).
4. Implement `org-io/src/keychain.rs`: `KeychainError::{ItemMissing,
   Locked, Unavailable { reason }, KeyMismatch, Malformed}`, each with
   `device_key`; `read_device_secret`, `write_device_secret`,
   `pub(crate) with_endpoint_seed` exactly as LLR-677fqq/LLR-n9dnm4/LLR-8erfbu
   state (`reason` is the `keyring::Error` variant name only).
   *Reconciled with S2, 2026-10-08 (§17.2 rule 2):* `read_device_secret`
   returns a device private key, so it and `write_device_secret` are
   `pub(crate)` (the module is declared `mod keychain;`, not `pub mod`);
   `tests/keychain.rs` reaches them through a `test_support` facade compiled
   only under `test-support`. `KeychainError` is public (it is in the
   handle's errors) and holds only the Device public key and a variant
   name. Here, too, the transitional holder T3 added is replaced: org-io's
   `create_persona` path writes the new secret to the keychain (the store
   file order is T9's, LLR-n9dnm4).
5. Green: org-io's verify line with `--test keychain`; clippy.
6. Commit: `feat(org-io): the keychain module — one item per Device key, move-only reads, typed refusals`.

### T9 — org-io: the store files

**Files touched:** org-io/src/store_file.rs (new), org-io/src/lib.rs (`OrgIo::open` changed in place, `create_persona`), org-io/src/submit.rs (the failed-write message), org-io/src/test_support.rs, org-io/tests/store_files.rs (new), org-io/tests/open.rs (S2's; its remaining tests adapted), org-io/tests/absences.rs, org-io/Cargo.toml, org-io/.guardrails/config.yaml, org-io/docs/requirements/ and org-io/docs/architecture/ (LLR-85zque arrives; LLR-rgdx22 already narrowed in T3), app/docs/architecture/2026-10-05-decomposition.md (pointer), app/src-tauri/tests/state_assembly.rs (four LLR-85zque tests at `24ca24c`: `the_store_is_opened_inside_the_data_dir_it_creates` and `a_data_dir_that_cannot_be_created_is_refused_naming_it` move, rewritten per Persona; `a_store_under_another_passphrase_is_refused` has no behaviour left once the passphrase goes and is deleted in T17, recorded, its place taken by LLR-uzax55's keychain-refusal tests; `init_opens_the_store_once_even_when_the_connect_is_refused`, a scan of the app's `init`, stays in the app), app/src-tauri/tests/ipc.rs (`connection_status_reports_a_data_dir_it_had_to_create_verbatim` keeps its IPC assertion), org-io/docs/problems/2026-10-13-non-atomic-store-write.md (moved from org-node in T9) (moves to org-io's ledger and is resolved), org-node/docs/problems/2026-10-08-save-after-mutate.md (PR-jav8zn resolved)
**Parallel:** no (after T6 and T8)
**IDs verified:** LLR-nt8dja, LLR-b9bm36, LLR-murjr6, LLR-uzax55, LLR-t4v69b, LLR-n9dnm4 (the create order), LLR-amv4mb (added 2026-10-07, S3a's removal); LLR-85zque (moved, rewritten per Persona); REQ-6d9xv6 as narrowed; resolves PR-n696wv and S2's PR-jav8zn (§17.3)

1. *Reconciled with S2 (`e41cb09`, §17), 2026-10-08:* S2's
   `OrgIo::open(data_dir, passphrase, transport_mode)` already creates the
   data directory and opens the store (the app's `init` and `for_test` call
   it since S2; T3 made it transitional). T9 changes it in place, not a new
   `open_node`: `OrgIo::open(data_dir: &Path, transport_mode) ->
   Result<OrgIo, OpenError>` lists the store files, unseals each through
   its keychain item and keeps `data_dir: PathBuf` in the handle
   (LLR-uzax55; the app's `init` calls it in T17). `OpenError` is public
   data only (paths, Device public keys, `KeychainError`) and joins
   `HANDLE_RETURN_ALLOWLIST`. `NodeBuilders::create_persona` becomes
   `OrgIo::create_persona(handle, name, surname) -> Result<PersonaId,
   CreateError>` (keychain write, then the store file, LLR-n9dnm4),
   returning the `PersonaId` only (§17.2 rule 2).
2. Red tests, new target `org-io/tests/store_files.rs`
   (`required-features = ["test-support"]`), each over a fresh temporary
   directory and the T8 in-memory keychain:
   - `// verifies: LLR-nt8dja` `each_persona_has_its_own_file_named_for_its_device_key`;
     `the_listing_ignores_every_other_name` (`persona-xyz.store`,
     `notes.txt`, a 63-hex name).
   - `// verifies: LLR-b9bm36` `the_file_holds_exactly_the_sealed_bytes`;
     `a_write_interrupted_after_the_temporary_leaves_the_old_store_unsealing`
     (inject a hook in `write_store` — a `test-support` callback after the
     temporary is written — that returns an error; the old file still
     unseals, no `.tmp-` file is left). No permission test: no mode or ACL
     is set (owner ruling, 2026-10-07).
   - `// verifies: LLR-murjr6` `a_failed_write_reports_failure_and_reloads_the_persona_from_its_file`
     (make the directory read-only after the first write; `issue`-like
     mutation fails; the service's state equals the file's).
   - `// verifies: LLR-uzax55, LLR-85zque` `opening_creates_the_data_dir_and_unseals_every_store`;
     `a_store_whose_item_is_missing_is_refused_and_left_untouched` (REQ-fh5gsq:
     file bytes equal before and after; no new item, no new file);
     `a_locked_keychain_refuses_opening_naming_the_persona`.
   - `// verifies: LLR-n9dnm4` `create_persona_writes_the_item_before_the_file_and_nothing_when_the_keychain_refuses`.
   - `// verifies: LLR-bvt5up` (ruling 2) `an_ordinary_save_succeeds_while_the_keychain_is_locked`:
     open, lock the in-memory keychain, run a mutating operation: the store
     file changes and no keychain call was made.
   - `// verifies: LLR-t4v69b` in `absences.rs`: `no_code_names_the_old_files`
     (`outstanding_invites.json`, `persona_store.bin` absent from `org-io/src`).
   - `// verifies: LLR-amv4mb` (S3a's removal, §16.1)
     `a_forgotten_store_is_removed_and_listed_no_more`: an accepted revocation
     (org-node's `StoreChange::Forgotten`) removes that Persona's store file,
     the next listing holds no file for its Device key, the other Personas'
     files are byte-for-byte unchanged, and the keychain item is left in
     place; `// verifies: LLR-amv4mb`
     `a_failed_removal_is_reported_and_the_persona_reloaded`: with the
     directory read-only the removal fails, the method reports it, and the
     Persona is reloaded from its file as LLR-murjr6 does for a failed write.
   - Move the app's two movable `state_assembly.rs` tests (LLR-85zque; see
     "Files touched" for the other two) here,
     rewritten per Persona and over the in-memory keychain.
3. Run; expected red: compile errors.
4. Implement LLR-nt8dja, LLR-b9bm36 (`OpenOptions::new().write(true).create_new(true)`,
   default permissions, no mode and no ACL on any platform), LLR-murjr6,
   LLR-uzax55, LLR-amv4mb (`remove_store(data_dir, device_key)`: remove the
   file, then on Unix sync the directory); replace the transitional
   `persist` with `write_store`/`remove_store` + the reload. The keychain
   item of a forgotten Persona is left for S3b-io's acknowledgement delivery
   (S3 proposed 7 and 8). *Ruled 2026-10-08 (ruling 11):* it is kept until
   that delivery ends or the Organisation's limit (roadmap ruling C, default
   two months) passes, then deleted; the deletion, and recording when the
   Persona was forgotten, are S3b-io's. No S4 code deletes a keychain item;
   `a_forgotten_store_is_removed_and_listed_no_more` asserts the item is
   still there.
5. Move LLR-85zque into org-io's ledger (rewritten per Persona, ruling A)
   and PR-n696wv into org-io's problems ledger, resolved. LLR-85zque's S2
   notes (its behaviour done by `open_service`, then by `OrgIo::open`) move
   with it. Resolve S2's PR-jav8zn (org-node's ledger,
   `2026-10-08-save-after-mutate.md`) in `resolve-problem`'s form: T3 made
   org-node return the change instead of saving, and here a failed write is
   reported as a storage failure (`StoreFileError`, not
   `OrgNodeError::Chain`) and the Persona is reloaded (LLR-murjr6); in
   `org-io/src/submit.rs` a commit whose store write failed says "the chain
   write executed and the commit was made, but storing it failed; reloaded
   from the file", not "nothing committed". Its reproducing test carries
   `// verifies: LLR-murjr6, PR-jav8zn`.
6. Green: org-io's verify line with `--test store_files`; clippy.
7. Commit: `feat(org-io): one sealed store file per Persona, written atomically (resolves PR-n696wv)`.

*Amended 2026-10-13 (owner ruling of that day, raised by T9).* LLR-murjr6's
"dropped from the service" and LLR-n9dnm4's "drops the new Persona's store"
need an org-node function that removes one store. Merged LLR-23sfdh
(REQ-em28bq, RC-ub82my) allows removal of a PersonaRecord only on the forget
path, and its scan flags `stores.retain`. The owner ruled to supersede it:

8. Files touched also: org-node/src/service.rs (`OrgService::restore_store`,
   `OrgService::drop_store`), org-node/tests/sealed_store.rs,
   org-node/tests/absences.rs, org-node/docs/requirements/ and
   org-node/docs/architecture/ (the S4 drafts and the file holding
   LLR-23sfdh), org-node/docs/risk/ (the file assessing RC-ub82my),
   org-io/docs/risk/ (the S4 draft).
9. Run `analyze-risks` on the change against RC-ub82my before the tests.
   Expected: no new hazard. `drop_store` runs only after a local IO failure,
   never because of a received message. It deletes nothing on disk: the
   file and the keychain item stay, and the Persona returns on the next open.
10. New item (ID from `new-id.sh`) superseding LLR-23sfdh. It keeps
    LLR-23sfdh's content and names `OrgService::drop_store` as a second
    exempt place. That place takes one Persona's store out of the in-memory
    service, touches no file and no keychain item, and is for org-io's two
    cases only: a refused keychain write for a new Persona, and a failed
    reload after a failed write. `satisfies: REQ-em28bq`. REQ-em28bq gets a
    dated clarification: "data it holds" is the sealed store on disk.
    `restore_store` gets a dated note on LLR-rdda86 (it adds or replaces one
    store and removes nothing).
11. Retire LLR-23sfdh by the 2026-10-13 retirement ruling (T6 step 6):
    delete its definition and `assesses:` entries, and rewrite every
    mention in the units as prose without the ID. Its scan test is red now.
    It is replaced by a new red-first scan test that exempts both `adopt`
    and `drop_store`, verifies the new item, and is watched fail first: run
    it against a temporary `stores.retain` placed in another function. A
    behaviour test also verifies the new item: after `drop_store`, the
    Persona's file and keychain item are unchanged and the next
    `OrgIo::open` lists it again. No green test is re-annotated.
12. LLR-85zque's risk assessment moves too. org-io's S4 risk draft assesses
    it against org-io's hazards (HAZ-eyene4, HAZ-dd78f9), and the
    `assesses: LLR-85zque` block in app/docs/risk/2026-10-05-derived.md is
    removed. The app tests that stay in the app and cite LLR-85zque are
    T17's (see there).

### T10 — org-io: the transport moves in, one endpoint per Persona

*Reconciled with S2 (`e41cb09`, §17) and owner ruling 12, 2026-10-08:*
S2 moved the chain preflight to `org-io/src/preflight.rs` and
`org-io/src/bin/preflight.rs` and left the transport check in org-node
(`ODS_DEVICE_SEED`, default all-zero). The transport check arrives here by
the keychain path, and the binary finds its Device keys by listing the
data directory's store files, as the node does: no `ODS_DEVICE_KEY` and no
key or seed in any environment variable (LLR-h9tcat, rewritten in place).
S2's tests reach org-node's endpoint in `tests/common/handles.rs` and
`receive_order.rs`; they move onto org-io's endpoints here.

**Files touched:** org-io/src/transport/{mod.rs,endpoint.rs,frame.rs} (moved from org-node/src/transport/), org-io/src/endpoints.rs (new), org-io/src/lib.rs, org-io/src/test_support.rs, org-io/Cargo.toml (`iroh/test-utils` under `test-support` and dev), org-io/tests/transport_handshake.rs, org-io/tests/transport_networked.rs, org-io/tests/endpoints.rs (new), org-io/tests/absences.rs, org-io/src/preflight.rs and org-io/src/bin/preflight.rs (S2's chain preflight; the transport check joins it), org-io/tests/preflight_transport.rs (new), org-io/.guardrails/config.yaml, org-io/docs/** (moved items, SOUP iroh row), org-io/docs/architecture/2026-10-08-org-io.md (LLR-3zdw8v's preflight note; REQ-v4tfap's note moved to T3), org-io/tests/common/handles.rs (S2's helpers that bind org-node's `OrgEndpoint`), org-io/tests/receive_order.rs (its sends through org-node's endpoint), org-node/tests/transport_handshake.rs and transport_networked.rs (deleted; moved), org-node/tests/service_lifecycle.rs (3 tests leave), org-node/tests/wire_frame_bound.rs (`frame_round_trips` leaves), org-node/docs/** (pointers, SOUP rows moved), org-node/Cargo.toml (the two `[[test]]` entries)
**Parallel:** no (after T9)
**IDs verified:** LLR-d5y7ks, LLR-d2pzzk, LLR-z35vbh, LLR-h9tcat (new); moved: REQ-db6s7q, REQ-2wzfzv, SDD-8uyg4s, LLR-ygn78w, LLR-v873fx, LLR-wwunf4, LLR-6adc99, LLR-wx77j5, LLR-k2y6nn, LLR-2smrvx, LLR-fa7jt8, SDD-b8tuv3, LLR-6zjzn2, LLR-cns6q6, LLR-ecz9a6, PR-8qsnhx

1. Move (ruling A) the items of §3.1 except LLR-jn5jeh, LLR-pw369n and
   PR-2dmjzj (T12), with LLR-6zjzn2 rewritten to one endpoint per Persona
   (LLR-d2pzzk) and LLR-cns6q6 to the keychain bind (LLR-d5y7ks);
   LLR-k2y6nn, LLR-2smrvx and LLR-fa7jt8 re-parented to REQ-szt6ea
   (SDD-aw3nka). Move the tests: `transport_handshake.rs` (6),
   `transport_networked.rs` (1), the three `service_lifecycle.rs` endpoint
   tests, `frame_round_trips`; rewrite their setup onto org-io's endpoint
   (bound through `bind_from_keychain` over the in-memory keychain).
2. Red tests, new target `org-io/tests/endpoints.rs`:
   - `// verifies: LLR-d5y7ks` `the_bound_endpoint_id_is_the_device_key`;
     `binding_twice_for_one_persona_yields_the_same_id`;
     `a_keychain_item_for_another_key_is_refused_and_the_endpoint_closed`;
     `a_locked_keychain_refuses_the_bind` (REQ-6d9xv6).
   - `// verifies: LLR-d2pzzk, PR-8qsnhx` `a_second_organisations_message_goes_out_under_its_own_personas_key`
     (the old PR-8qsnhx test, inverted).
   - `// verifies: LLR-z35vbh` `a_message_to_either_personas_device_reaches_the_receive_channel`
     (with its authenticated sender: S3a's member-sender rule needs it,
     §16.1).
   - In `absences.rs`, `// verifies: LLR-d5y7ks` `iroh_secret_key_is_named_in_one_function_only`:
     `SecretKey` appears only inside `fn bind_from_keychain`; no
     `to_bytes()`, `.secret()`, `.secret_key()`, `{:?}` on it; no public
     type of `org-io/src` has a field of type `Endpoint` or `OrgEndpoint`.
   - **The transport preflight** (owner ruling 7, 2026-10-07), new target
     `org-io/tests/preflight_transport.rs` (`required-features =
     ["test-support"]`), over the in-memory keychain:
     `// verifies: LLR-h9tcat` `the_transport_check_passes_for_a_device_key_whose_item_exists`
     (Loopback: a bound socket; the check binds through
     `bind_from_keychain`);
     `// verifies: LLR-h9tcat` `a_missing_or_locked_item_fails_the_check_naming_the_key_and_binds_nothing`
     (`ItemMissing` and `Locked` each give one FAIL row naming the Device
     key, and no endpoint is bound);
     ~~`a_device_key_that_does_not_parse_fails_the_check_naming_the_variable`~~
     (dropped 2026-10-08 with `ODS_DEVICE_KEY`, ruling 12); in its place
     `// verifies: LLR-h9tcat` `the_preflight_checks_every_persona_listed_in_the_data_dir`
     (two Personas' store files and items: two rows, one per Device key,
     each bound through its own item) and
     `// verifies: LLR-h9tcat` `a_data_dir_with_no_store_fails_the_check_naming_the_dir_and_binds_nothing`;
     and in `absences.rs`, `// verifies: LLR-h9tcat, LLR-d5y7ks`
     `the_preflight_reads_no_key_or_seed_from_the_environment`:
     `ODS_DEVICE_SEED`, `ODS_DEVICE_KEY`, `DeviceSeed` and
     `from_keychain_bytes` are absent from `org-io/src/preflight.rs` and
     `org-io/src/bin/preflight.rs`, no `std::env::var` call in either names
     a key or seed variable, and `SecretKey` is still named only in
     `bind_from_keychain`.
3. Implement; org-io stops calling org-node's transport (S2's paths now use
   org-io's endpoints and `org_node::wire`), so org-node's transport is dead
   code until T14 removes it. The accept tasks forward `(DevicePublicKey,
   Vec<u8>)`, the sender the handshake authenticated with the body
   (S3a's `recv_one`, LLR-v873fx). The transport preflight: S2's org-io
   preflight (chain checks) gains `check_transport(device_key, mode,
   online_timeout)`, which binds with `bind_from_keychain` and checks the
   bound socket (Loopback) or the relay home (Networked) as org-node's did.
   *Rewritten 2026-10-08 (owner ruling 12; was: read `ODS_DEVICE_KEY`):*
   `bin/preflight.rs` reads **no** device key or seed from the environment.
   It finds the Device keys through the same path as the node: it lists the
   store files of the node's data directory with T9's listing (LLR-nt8dja)
   and runs `check_transport` for each listed Device key, each bound through
   its own keychain item; no store file is one FAIL row naming the
   directory. The data directory is a path, not key material: it is the
   app's `ODS_DATA_DIR`, with no default; unset is a refusal (owner ruling
   2026-10-08, §17.4). `ODS_TRANSPORT`
   (the mode, not a key) stays as org-node's binary read it. org-node's
   transport-only preflight (S2's reduced copy, reading `ODS_DEVICE_SEED`)
   is deleted in T14.
4. Docs: org-io SOUP gets org-node's `iroh` row whole (PR-d4nye8 note,
   `RelayMode::Disabled`, `ed25519-dalek` pre-release, `test-utils`), the
   dev row and checklist item 3 (§4), replacing S2's narrower org-io `iroh`
   row (`org-io/docs/architecture/soup.md:38`, the address type passed
   through); org-node's rows say "moved to org-io".
   ~~REQ-v4tfap gets the dated note of §14~~ (moved to T3, §17.3: T3 is the
   first task in which org-io receives a `DeviceSecret`, and its note names
   both move-only paths as §14 words it; T10 adds nothing to it). LLR-3zdw8v
   (S2's, no `DeviceSeed` in org-io) gets a dated note that the transport
   preflight came in by the keychain path (LLR-h9tcat). S2's
   `test-support` `NodeBuilders::ensure_endpoint` (T3's `bind_endpoint`)
   goes; tests bind through org-io's endpoints.
5. Green: org-io's verify line with the four targets (`transport_handshake`,
   `transport_networked`, `endpoints`, `preflight_transport`); org-node's
   (without the two moved targets); clippy both.
6. Commit: `feat(org-io): the transport moves in; one endpoint per Persona, bound from the keychain; the transport preflight by the keychain path (resolves PR-8qsnhx)`.

### T11 — org-io: the lock-free receive step

**Files touched:** org-io/src/lib.rs (`OrgIo` holds `tokio::sync::Mutex<OrgService>`; methods take `&self`), org-io/src/view.rs (S2's `NodeView`/`NodeBuilders` folded into `OrgIo`'s methods; the summaries stay), org-io/src/receiver.rs (new), org-io/src/test_support.rs, org-io/tests/receive_step.rs (new), org-io/tests/receive_order.rs, org-io/tests/submit_flow.rs, org-io/tests/absences.rs (allowlist), org-io/tests/signatory_key.rs, org-io/tests/common/handles.rs (and every org-io test using `view`/`node_mut`/`node_mut_for_test`), org-io/docs/architecture/2026-10-08-org-io.md (SDD-z3ychz, LLR-7pj5af), org-io/.guardrails/config.yaml, org-io/docs/problems/ (PR-7v8anh and PR-kwwap5 arrive, resolved), app/docs/problems/ (pointers)
**Parallel:** no (after T10)
**IDs verified:** LLR-9arvqj, LLR-cvyyr8 (as amended 2026-10-07: the sender is passed), LLR-szhe7j; S2's LLR-7pj5af (its order, under `next_received`; clarified) and SDD-z3ychz (clarified); resolves PR-7v8anh, PR-kwwap5

*Reconciled with S3a (§16.1):* the receive step hands org-node the sender
with the body, settles an acknowledgement with no chain read, and passes
`commit_checked` a `device_secrets` closure over the keychain (a signing
operation, REQ-6d9xv6), because an accepted revocation or a self-removing
update signs S3a's acknowledgements. Until S3b-io sends them, the outcome
carries them to the caller and the app drops them, as it does today.

1. Red tests, `org-io/tests/receive_step.rs` (`#[tokio::test(flavor = "multi_thread")]`):
   - `// verifies: LLR-9arvqj, PR-7v8anh` `a_command_completes_while_the_receiver_waits`:
     `start_receiving`, spawn `next_received()` with no peer sending, then
     `tokio::time::timeout(Duration::from_secs(2), io.issue_like_command())`
     completes (use the handle's `organisations()` query and one mutating
     call, `create_persona`).
   - `// verifies: LLR-cvyyr8, PR-kwwap5` `a_declared_first_admission_is_committed_by_the_one_path`.
   - `// verifies: LLR-cvyyr8` `a_refused_check_reads_no_chain` (FakeChain read
     counter unchanged; an update from an unlisted sender, S3a's
     `SenderNotListed`, is one of the refusals) and `an_accepted_body_reads_the_chain_once`.
   - `// verifies: LLR-cvyyr8` `the_sender_org_node_sees_is_the_handshakes`:
     the same body delivered from a listed Member's endpoint commits, and
     from an unlisted Device's endpoint is refused as `SenderNotListed`.
   - `// verifies: LLR-cvyyr8` `an_acknowledgement_is_settled_with_no_chain_read_and_no_store_write`.
   - `// verifies: LLR-cvyyr8` `an_accepted_revocation_signs_with_the_keychain_secret_and_removes_the_store`
     (S3a's notice; acknowledgements in the outcome; LLR-amv4mb's removal);
     `a_locked_keychain_refuses_the_revocation_and_changes_nothing`
     (REQ-6d9xv6: the keychain refusal is reported, the store file and the
     record unchanged).
   - `// verifies: LLR-cvyyr8` `the_store_write_lands_before_the_outcome_returns`.
   - `// verifies: LLR-szhe7j` `a_closed_endpoint_is_terminal_endpoint_closed`,
     `a_failed_bind_is_terminal_bind_failed`, `a_malformed_body_is_not_terminal`.
2. Run; expected red: compile errors.
3. Implement `OrgIo { node: tokio::sync::Mutex<OrgService>, .. }`,
   ~~`with_node`~~ (*reconciled with S2, 2026-10-08, §17.2 rule 1:* no
   closure over `&OrgService`; S2's `view()`/`NodeView` and
   `node_mut()`/`NodeBuilders` become `async` methods of `OrgIo` on `&self`
   that take the lock inside and return owned allowlisted values:
   `personas()`, `organisations()`, `organisation(org_id)`,
   `persona_public_keys(persona_id)`, `admit_member`/`revoke_member ->
   BuiltUpdate`; `NodeView`/`NodeBuilders` are removed with
   `HANDLE_RETURN_ALLOWLIST`'s scan extended to the new methods; SDD-z3ychz
   and LLR-7pj5af get their dated notes, §17.3),
   `start_receiving`, `next_received` (the accept channel is an
   `mpsc::Receiver<Inbound>` behind its own mutex, never the node's, where
   `Inbound { sender: DevicePublicKey, body: Vec<u8>, verdict: oneshot::Sender<Verdict> }`;
   T11 answers nothing on the connection yet, T11a adds the verdict),
   `ReceiveError::{Terminal(TerminalCause), Refused(OrgNodeError), Chain(..), Store(..), Keychain(..)}`;
   `next_received` returns `Received::{Committed(ReceiveOutcome), Acknowledged(VerifiedAcknowledgement)}`;
   `Received`, `ReceiveKind`, `ReceiveError`, `TerminalCause` and
   `VerifiedAcknowledgement` join `HANDLE_RETURN_ALLOWLIST` after review
   (public data only: ids, epochs, roots, public keys, signatures; no
   Organisation information, key or body).
4. Move PR-7v8anh (app) and PR-kwwap5 (app) into org-io's problems ledger
   with dated notes and resolve them.
5. Green: org-io's verify line with `--test receive_step`; clippy.
6. Commit: `feat(org-io): the receive step holds no lock while it waits (resolves PR-7v8anh, PR-kwwap5)`.

### T11a — org-io: a revoked Device may connect, for its acknowledgement only

*Added 2026-10-07 (owner ruling 8).* Members accept inbound connections from
Devices they hold as revoked, for acknowledgements only: no other message
kind from such a Device is acted on and no Organisation data is served on
the connection. The revoked Device connects directly (no relay) to up to
three distinct Member Devices at addresses it kept, and tries the next when
one refuses; that sending side is S3b-io's (S3 plan T16). S4 provides the
receiving side: acceptance, the kind gate (org-node's R1 rule, reached over
the real transport) and a verdict that tells the sender whether to try the
next address. If S3b-io has merged first (T0), the normal-case test below
drives its sender instead of the test double. *Reconciled 2026-10-08:*
S3b-io has not merged at `e41cb09`, so the test double stands; the verdict
byte is ruled (ruling 9) and HAZ-abm3xs's residual is accepted (ruling 10,
already written into the risk draft).

**Files touched:** org-io/src/receiver.rs, org-io/src/transport/endpoint.rs (the verdict write), org-io/src/test_support.rs (`dial_direct_as(device_secret, addr, body) -> Result<Verdict, _>`, a relay-free test dialler holding a revoked Device's identity), org-io/tests/acknowledgement_inbound.rs (new), org-io/tests/absences.rs, org-io/Cargo.toml (`[[test]]`), org-io/.guardrails/config.yaml, org-io/docs/requirements/2026-10-06-transport.md, org-io/docs/architecture/2026-10-07-transport.md (test names)
**Parallel:** no (after T11)
**IDs verified:** LLR-wvgec4, LLR-unqfj5 (new); REQ-ur7z4a and RC-b4m5sj through them; S3a's LLR-3aysup and LLR-2r2fha, LLR-kzgjz8 over the real transport

1. Red tests, new target `org-io/tests/acknowledgement_inbound.rs`
   (`required-features = ["test-support"]`, `#[tokio::test(flavor =
   "multi_thread")]`). Setup: Members A and B in one Organisation over
   loopback endpoints; A commits B's removal (so A holds B's Device as
   revoked); B accepts the revocation and signs its acknowledgement (S3a);
   B's Device secret stays in the in-memory keychain for the dial.
   - Normal case, `// verifies: LLR-wvgec4, LLR-unqfj5`
     `a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted`:
     B dials A's direct loopback address with no relay
     (`dial_direct_as`) and sends its `Acknowledgement` body; A's
     `next_received` returns `Received::Acknowledged` naming B's Device,
     nothing is written to A's store file, and B reads exactly one byte,
     `Verdict::Accepted`, then end of stream.
   - Abnormal input, `// verifies: LLR-wvgec4, LLR-unqfj5`
     `a_revoked_device_sending_anything_but_an_acknowledgement_is_refused_and_served_nothing`:
     from B's identity, an `OrgInformation` body for the Organisation
     (S3a's `SenderNotListed`, LLR-2r2fha), a `Revocation` body
     (`SenderNotListed`, LLR-kzgjz8), and bytes that do not decode
     (`MalformedMessage`) are each refused: no chain read (FakeChain
     counter unchanged), A's store file byte-for-byte unchanged, and B
     reads exactly `Verdict::Refused` and nothing else (no record, key,
     snapshot, address or other Organisation data on the connection).
   - Abnormal input, `// verifies: LLR-wvgec4, LLR-unqfj5`
     `an_acknowledgement_a_member_still_listing_the_device_refuses_is_answered_refused`:
     C, which has not committed B's removal, receives B's acknowledgement:
     `AcknowledgementForListedDevice` (S3a), verdict `Refused`, so B's
     sender knows to try its next kept address.
   - Abnormal input, `// verifies: LLR-wvgec4`
     `an_acknowledgement_relayed_by_another_device_is_refused`: C's identity
     sends B's acknowledgement (`AcknowledgementNotFromItsDevice`, LLR-3aysup);
     verdict `Refused`.
   - `// verifies: LLR-wvgec4` `the_accept_path_has_no_allow_list`: source
     scan of `org-io/src/transport` and `receiver.rs`: no accept-time check
     of the remote id against a record (the gate is org-node's
     `check_received`), and `accept` is not filtered by relay or direct path.
   - `// verifies: LLR-unqfj5` `nothing_but_the_verdict_is_written_on_an_inbound_stream`:
     source scan: the receive side's only write to an inbound stream is the
     one-byte verdict (`write_all(&[verdict.as_byte()])`), followed by
     `finish`.
2. Run; expected red: compile errors (no `Verdict`, no `dial_direct_as`),
   then, once they compile, the verdict assertions.
3. Implement LLR-wvgec4 and LLR-unqfj5: the accept task keeps the
   inbound stream's send half until the receive step answers through
   `Inbound::verdict`, writes the one byte and finishes; a refusal at any
   step, a store-write failure included, answers `Refused`; a dropped
   responder writes nothing (the sender treats silence as a refusal). The
   verdict byte follows the body on the same bi-stream; pre-S4 senders,
   which close after `stopped`, ignore it, so the ALPN is unchanged.
4. Docs: REQ-ur7z4a, RC-b4m5sj, HAZ-abm3xs, LLR-wvgec4, LLR-unqfj5 are
   drafted already (2026-10-07); record the test names under them. The
   owner's acceptance of HAZ-abm3xs's residual (2026-10-08) is already in
   the risk draft's "Residuals" section.
5. Green: org-io's verify line with `--test acknowledgement_inbound`; clippy.
6. Commit: `feat(org-io): a revoked Device may connect for its acknowledgement only; inbound streams answer with a verdict byte and nothing else`.

### T12 — org-io: fan-out, the loopback address check, the peer-address parse

**Files touched:** org-io/src/send.rs (new), org-io/src/peer_addr.rs (new), org-io/src/lib.rs, org-io/tests/send_fanout.rs (new), org-io/tests/peer_addr.rs (new), org-io/tests/fuzz_peer_addr/fuzz_target.rs (new), org-io/tests/submit_flow.rs (receives the two LLR-q225ws tests, rewritten), app/src-tauri/tests/submit_flow.rs (S2 left exactly those two tests there; the file is deleted and leaves the app's `[[test]]` list and verify line), app/src-tauri/Cargo.toml (`[[test]] submit_flow` removed), app/.guardrails/config.yaml, org-io/src/submit.rs (`submit_commit_send` sends through `send_committed`), org-io/tests/absences.rs (allowlist), org-io/Cargo.toml, org-io/.guardrails/config.yaml, org-io/docs/** (moved items), app/docs/** (pointers), app/src-tauri/tests/ipc.rs (the peer-address unit cases leave), org-node/docs/** (pointers)
**Parallel:** no (after T11)
**IDs verified:** LLR-znhmh2, LLR-r9sh6k, LLR-2b2ghe (new); moved: LLR-jn5jeh, LLR-pw369n, LLR-ty85xv, LLR-n6twt7, LLR-ctrfz4, LLR-q225ws (then retired), PR-2dmjzj, PR-3ue4va; resolves PR-2dmjzj, PR-3ue4va

*Reconciled with S3a (§16.1):* the recipients `send_committed` dials are
`outcome.outgoing`'s, which since T5 include S3a's revoked Devices with
their `Revocation(notice)` bodies; S3a's `send_update` refusal of an
unlisted, un-noticed Device (`NoRevocationForRecipient`) has no counterpart,
because org-io names no recipient of its own. A sender may read the verdict
byte T11a adds and report `Refused` per Device; a missing verdict (a pre-S4
receiver) counts as delivered. Resending an unacknowledged revocation
(S3 proposed 6) is S3b-io's.

1. Red tests:
   - `send_fanout.rs`: `// verifies: LLR-znhmh2, PR-3ue4va`
     `networked_every_recipient_is_attempted_and_reported_in_order` (three
     recipients over the relay test fixture, one of them never bound: two
     `Ok`, one `Err`, order = `outcome.outgoing`'s);
     `a_failed_send_leaves_the_commit_in_place` (LLR-t4znbk's org-io half);
     `// verifies: LLR-r9sh6k, PR-2dmjzj` `loopback_without_an_address_is_refused_and_sends_nothing`,
     `loopback_to_an_address_of_no_recipient_is_refused_and_sends_nothing`,
     `loopback_sends_to_the_matching_recipient_only_and_reports_the_rest_not_attempted`.
   - `peer_addr.rs`: `// verifies: LLR-2b2ghe` blank → `None`; `0x`-prefixed
     and bare hex of a postcard `EndpointAddr` → `Some`; odd-length hex →
     `Hex`; valid hex of garbage → `Decode`; both errors name `peer_addr`.
     The moved LLR-ty85xv, LLR-n6twt7, LLR-ctrfz4 cases join this file.
   - Fuzz `fuzz_peer_addr` (bolero, `harness = false`), `// verifies: LLR-2b2ghe`:
     arbitrary text, never panics.
   - `submit_flow.rs`: move the two LLR-q225ws tests from
     `app/src-tauri/tests/submit_flow.rs` (where S2 left them,
     `each_command_sends_once_to_one_device_and_the_kind_follows_the_record`
     and `a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing`)
     into org-io's `submit_flow.rs`, and rewrite them as fan-out tests
     annotated LLR-znhmh2; LLR-q225ws moves to org-io and is retired there
     (superseded in place by LLR-znhmh2).
2. Run; expected red: compile errors.
3. Implement `send_committed` and `parse_peer_addr` (`PeerAddr` newtype
   over `iroh::EndpointAddr`); `admit`/`revoke` paths call
   `send_committed` from the sender's endpoint (LLR-d2pzzk). *Reconciled
   with S2, 2026-10-08:* S2's `OrgIo::submit_commit_send(rng, &BuiltUpdate,
   recipient, peer_addr: Option<iroh::EndpointAddr>)` loses `recipient`
   (org-node names every recipient) and takes `peer_addr: Option<PeerAddr>`
   (so org-io's public surface names no `iroh` type); it returns T5's
   `Submitted` extended with `sends: SendReport` (public data: Device
   public keys and typed send errors), and `SendReport`, `SendError` and
   `PeerAddr` join `HANDLE_RETURN_ALLOWLIST`. The parse moves from
   `app/src-tauri/src/commands.rs` (`parse_peer_addr`, re-read the lines at
   `24ca24c`).
4. Move the items; PR-3ue4va gets F3's wording fix (§9) as a dated note
   when it moves; PR-2dmjzj and PR-3ue4va resolved.
5. Green: org-io's verify line with `--test send_fanout --test peer_addr --test fuzz_peer_addr`; clippy.
6. Commit: `feat(org-io): fan-out to every Device, the loopback address check, the total peer-address parse (resolves PR-3ue4va, PR-2dmjzj)`.

### T13 — org-io: the invitation exchange

**Files touched:** org-io/src/exchange.rs (new), org-io/src/lib.rs, org-io/tests/invitation_exchange.rs (new), org-io/tests/node_stories.rs (new), org-io/Cargo.toml, org-io/.guardrails/config.yaml, org-io/docs/** (moved items), app/docs/requirements/2026-10-06-invitation.md, app/docs/architecture/2026-10-06-invitation.md (pointers), app/src-tauri/tests/invitation.rs (the remaining 14 tests leave; the file is deleted)
**Parallel:** yes, with T14 (org-io and app docs/tests here; org-node only there)
**IDs verified:** LLR-h9qf3s, LLR-9cb443, LLR-y4dg74 (new); moved: REQ-prjja8, REQ-tcutr6, REQ-65xqp8, LLR-9sraks, LLR-f35pda (rewritten: the pairs live in the sealed store), LLR-w4mhd4, LLR-rt8gdz, LLR-gha5f6

1. Move the 14 app tests (LLR-9sraks, LLR-f35pda, LLR-w4mhd4, LLR-rt8gdz,
   LLR-gha5f6) to `org-io/tests/invitation_exchange.rs`, onto `OrgIo`'s
   `issue_invite`, `produce_reply`, `import_reply`, `admit_reply`.
2. Red tests in the same file:
   - `// verifies: LLR-h9qf3s` `the_invite_is_returned_only_after_its_store_write`
     (failing write → no Blob, and the pair not held after reload).
   - `// verifies: LLR-9cb443` `an_unconfirmed_reply_parses_and_writes_nothing`.
   - `// verifies: LLR-y4dg74` `admission_runs_check_admit_write_commit_send_settle_in_order`
     (FakeChain write and read counters, the store writes' order);
     `a_failed_send_still_settles`; `a_refused_check_submits_nothing`.
   - `node_stories.rs`: `five_stories_full_e2e` duplicated from org-node's
     `service_stories.rs` (§5) over two loopback endpoints and `FakeChain`,
     `// verifies:` lines as the original.
3. Run; expected red: compile errors.
4. Implement LLR-h9qf3s, LLR-9cb443, LLR-y4dg74 and `import_reply`
   (parse + `check_reply`). Remove org-node's `expect_admission` if nothing
   calls it any more (note it in the report), and S2's
   `NodeBuilders::expect_admission` (by T11 an `OrgIo` method) with it.
   *Reconciled with S2, 2026-10-08:* `admit_reply` runs S2's submit path
   (`submit::submit_commit_send` as T5/T12 reshaped it); `AdmitReport`,
   `ExchangeError` and the reply's public view join
   `HANDLE_RETURN_ALLOWLIST` after review; the Blobs returned as `String`
   carry Device and member public keys and the invite id, by design.
5. Green: org-io's verify line with `--test invitation_exchange --test node_stories`;
   org-io's `check-trace.sh` clean but for pre-existing advisories.
6. Commit: `feat(org-io): the invitation exchange runner`.

### T14 — org-node: no transport, no IO features, no async

**Files touched:** org-node/src/transport/ (deleted), org-node/src/preflight.rs and org-node/src/bin/preflight.rs (S2's transport-only copy, deleted: owner ruling 7, the check now lives in org-io, T10), org-node/tests/preflight.rs (S2's two transport-check tests, no `verifies:`; deleted with the `[[test]] preflight` entry), org-node/src/service.rs, org-node/src/lib.rs, org-node/src/revocation.rs and org-node/src/test_fixtures.rs (any `transport` gate left after T2), org-node/Cargo.toml (`transport` feature and `iroh`, `tokio`, `async-trait` gone; `test-support` no longer pulls `iroh/test-utils`; the `preflight` `[[bin]]` and, if S2 left it, `[[test]]` go), Cargo.lock, org-node/tests/absences.rs, org-node/tests/support/mod.rs, org-node/tests/** (any `#[tokio::test]` left becomes `#[test]`), org-node/.guardrails/config.yaml, org-node/docs/architecture/soup.md, org-node/docs/architecture/2026-10-06-chain-authority.md, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/AGENTS.md, org-node/docs/requirements/2026-10-06-chain-authority.md ("Sequencing", change 2), org-io (call sites of the transitional functions, if any remain)
**Parallel:** yes, with T13
**IDs verified:** LLR-4tcxsu, LLR-qezw3n (amended to absences); REQ-x3jrh8's "no file" (absence)

1. Red tests in `absences.rs`:
   - `// verifies: LLR-4tcxsu, REQ-x3jrh8` `org_node_is_sans_io`:
     `assert_absent` for `iroh`, `std::fs`, `PathBuf`, `tokio`, `async fn`,
     `.await`, `keyring`; `normal_dependencies()` names none of `iroh`,
     `tokio`, `async-trait` (already absent since S2, LLR-mn5c2q), `argon2`, `keyring` (org-node never touches
     the OS keychain, REQ-26mjqa); and `ODS_DEVICE_SEED`, `DeviceSeed` and
     `fn check_transport` (the preflight left with owner ruling 7).
     S3a's `revocation_and_reconcile_do_no_io` scan stays and is now
     subsumed by this one for the whole crate.
   - `// verifies: LLR-qezw3n` the existing claim, evidence moved here.
2. Run; expected red: the transitional `bind_endpoint`, `receive_body`,
   `send_body`, `transport/` and the transport-only preflight make the scan
   fail.
3. Delete them (the preflight's `[[bin]]` entry with it), the `transport`
   feature and its dependencies; `app`
   becomes the store-and-service feature (`chacha20poly1305`, `hkdf`,
   `sha2`, `base64`); remove `required-features = ["transport"]` from the
   remaining targets.
4. Docs: SOUP row `tokio` removed, and `async-trait`'s (S2 marked it
   "Leaves 2026-10-08", `soup.md:156`; the dependency is already gone);
   `iroh` moved in T10; `org-node/AGENTS.md` edges; LLR-4tcxsu, LLR-qezw3n notes; the
   chain-authority "Sequencing" line (§3.2 last row).
5. Green: org-node's final verify line; clippy; `cargo build -p org-node --no-default-features`; org-io's line.
6. Commit: `refactor(org-node): sans-IO — no transport, no file, no async`.

### T15 — app: depends on org-io only, no iroh, no Blob code

*Reconciled with S2 (`e41cb09`, §17), 2026-10-08:* S2 already made org-io
the app's only unit dependency (`org-io/tests/app_boundary.rs`) and deleted
`app/src-tauri/src/submit.rs`; the app still names iroh in `commands.rs`
(`parse_peer_addr`, `Option<iroh::EndpointAddr>`) and reaches org-node's
types through `org_io::node`. This task also **narrows the `node`
re-export** (§17.2 rule 3; roadmap: "narrows them in S4"): `pub use org_node
as node` in `org-io/src/lib.rs` becomes an explicit `pub use` list of the
value types the app names (ids, names, public keys, `OrgNodeError`, the
summaries), so `OrgService`, `PersonaStore`, `DeviceSecret`, the Wire types
and the store records are no longer reachable through org-io; a red-first
test in `org-io/tests/app_boundary.rs` (`// verifies: LLR-3zdw8v`) reads
`lib.rs` and fails on `pub use org_node as` or any of those names. The
app's tests that built `OrgService`/`PersonaStore` through the re-export
(`tests/support/mod.rs`, `ipc.rs`, `org_id_parsing.rs`,
`receiver_events.rs`, `state_assembly.rs`; `invitation.rs` and
`submit_flow.rs` have left by then) move onto
`OrgIo::open`/`for_test`. Files touched gain org-io/src/lib.rs,
org-io/tests/app_boundary.rs and app/src-tauri/tests/support/**.

**Files touched:** app/src-tauri/Cargo.toml (`iroh`, `base64`, `postcard` gone), Cargo.lock (app's), app/src-tauri/src/invitation.rs (deleted), app/src-tauri/src/commands.rs, app/src-tauri/src/lib.rs, app/src-tauri/src/state.rs (`transport_mode_for` maps to org-io's type), app/src-tauri/tests/ipc.rs, app/src-tauri/tests/support/**, app/docs/architecture/soup.md, app/docs/architecture/2026-10-05-decomposition.md, app/docs/requirements/2026-10-06-invitation.md, app/docs/requirements/2026-09-14-tauri-shell.md, app/docs/problems/2026-10-05-problems.md (PR-7zz76t), app/AGENTS.md, app/.guardrails/config.yaml
**Parallel:** no (after T13 and T14)
**IDs verified:** LLR-vzf8j2, SDD-aq7m6b, REQ-645jq9, RC-8ygnjd, LLR-j5pacp, REQ-nfr3n2, LLR-qhjp6g, LLR-be3zv9, LLR-8krgzj (amended, annotations kept)

1. Red test in `app/src-tauri/tests/ipc.rs`, `// verifies: LLR-vzf8j2`
   `the_app_manifest_names_no_iroh_base64_or_postcard` (read
   `app/src-tauri/Cargo.toml`'s `[dependencies]`); expected red: they are
   there.
2. Implement: the commands call org-io's exchange, fan-out and peer-address
   parse; the IPC tests that asserted the peer-address error surfacing keep
   their IPC-level assertion. *Reconciled with S3a:* `submit.rs:138`'s
   `send_update(&outcome, ..)` goes with `submit_commit_send` (org-io's);
   `events.rs`'s `classify_receive_error` (`:70-150`) loses
   `NoRevocationForRecipient` (removed in T5) and keeps S3a's other
   variants in the same class.
3. Docs (dated notes, "S4, task T15"): §3.6's rows for these IDs; app SOUP
   (`iroh` removed; `postcard`, `base64` rows removed or narrowed); PR-7zz76t
   closed as obsolete (its premise, a Join request carrying an address, is
   gone), in `resolve-problem`'s form.
4. Green: the app's verify lines (the first without `--test invitation`).
   *Added 2026-10-14 (dispatcher; left by T6, T12 and T13):* also, in this
   task:
   - `classify_receive_error` gains an arm for
     `OrgNodeError::OrganisationHeldByAnotherPersona` (T6), in the class of
     the other refusals.
   - App citations of items that moved to org-io (LLR-9sraks, LLR-f35pda,
     LLR-w4mhd4, LLR-rt8gdz, LLR-gha5f6) in `src-tauri/src/{invitation,
     commands,state}.rs`, `src/lib/invite.ts` and the Admit / CreateOrg /
     Invite.svelte comments become prose. Green tests whose only ID moved
     run with no `verifies:` line and a dated prose pointer:
     `ipc::produce_invite_reply_refuses_without_confirmation_over_ipc`
     (LLR-w4mhd4) and the three `invite.personas.test.ts` tests (LLR-rt8gdz).
     That is the T9 precedent.
   - LLR-rt8gdz's webview half: mint a new app LLR (ID from `new-id.sh`)
     for the webview offering only unbound Personas (the `unboundPersonas`
     filter and the founding panel). Give it a new vitest test that cites
     it, and record a mutation check, since the behaviour already exists.
     Dispatcher's default, from T13's option (a). The owner may overrule it.
   - REQ-m8sgjk: its only LLR (LLR-gha5f6) moved to org-io, and its text
     still says the app chooses "which Device receives". Narrow it with a
     dated note to what the app still does: it passes the reply to org-io
     and shows the result. Cite it from the IPC-level admit test this task
     rewrites. If no rewritten test fits, stop and ask.
   - Record in the app's PR-5mc4d8 that org-io's `parse_peer_addr` strips
     only one `0x` (T12), so the peer-address site is fixed. Resolve the
     problem only if that was its last site.
   - T13 marked REQ-prjja8, REQ-tcutr6 and REQ-65xqp8 `exported: yes` in
     org-io (S2 precedent, REQ-nfr3n2). Keep the app's citations of them.
5. Commit: `refactor(app): the app names no iroh and no Blob format; org-io runs the exchange`.

### T16 — app: the receiver uses org-io's step and its typed terminal error

**Files touched:** app/src-tauri/src/commands.rs (`next_outcomes`, `start_receiver`), app/src-tauri/src/events.rs (`TERMINAL_ERRORS`, `is_terminal_error`), app/src-tauri/tests/receiver_events.rs, app/src-tauri/tests/receiver_guard.rs, app/docs/architecture/2026-10-05-decomposition.md, app/docs/requirements/2026-09-14-tauri-shell.md (REQ-x3c8n2), org-io/docs/requirements (REQ-8e57wm exported, `satisfies: REQ-x3c8n2`)
**Parallel:** no (after T15)
**IDs verified:** SDD-mwqf6x, LLR-7bk6qh, LLR-a9rjtf, SDD-rmbr3t, LLR-2vg79y (amended); REQ-x3c8n2 (re-targeted to org-io)

*Reconciled with S3a (§16.1):* `next_outcomes` (`commands.rs:369`) today
maps `SelfDeleteOutcome::{SelfDeleted, Acknowledged, UpdatedNotRevoked}`
to `ReceiverOutcome::{SelfDeleted, AcknowledgementReceived, Updated}`
(`events.rs:16-21`; S3a's `AcknowledgementReceived` emits nothing,
LLR-2vg79y). From T16 the map is from org-io's `Received`:
`Acknowledged` → `AcknowledgementReceived`; `Committed` by
`ReceiveKind`: `Removed` and `Revoked` → `SelfDeleted`, `Updated` →
`Updated`, `Admitted` → the admitted outcome. The acknowledgements a
`Removed` or `Revoked` outcome carries are still dropped until S3b-io sends
them (S3a's comment at `commands.rs:383`).

1. Red tests in `receiver_events.rs`: `// verifies: LLR-7bk6qh`
   `terminal_is_decided_by_the_error_variant_not_its_text` (a
   `ReceiveError::Terminal(EndpointClosed)` with an arbitrary message is
   terminal; a non-terminal variant whose text contains "endpoint closed"
   is not); `// verifies: LLR-a9rjtf` `the_outcome_kind_comes_from_receive_kind`
   (all four kinds, and an acknowledgement); `// verifies: LLR-2vg79y`
   `a_revocation_accepted_by_its_notice_is_announced_as_self_deleted`;
   `// verifies: SDD-rmbr3t` (or its LLR) `the_receiver_holds_no_app_lock_while_waiting`.
2. Implement: `next_outcomes` calls `OrgIo::next_received` on the shared
   `Arc<OrgIo>` (no app-side lock; S2's `AppState.org_io: Mutex<OrgIo>`
   becomes `Arc<OrgIo>`, and every command's `state.org_io.lock().await`
   goes); `is_terminal_error` becomes a match on
   `ReceiveError::Terminal(_)`; `TERMINAL_ERRORS` deleted.
3. ~~REQ-x3c8n2's `expects:` changes from org-node to org-io~~ (done by S2,
   task T9: REQ-x3c8n2 is `expects: org-io` since 2026-10-08, still an
   UNMET-EXPECTATION advisory in the app's trace); org-io's
   REQ-8e57wm gains `exported: yes` and `satisfies: REQ-x3c8n2` (the app's
   config already lists org-io in `depends_on`, S2).
4. Green: the app's verify lines; `check-trace.sh` for app and org-io: no
   UNMET-EXPECTATION for REQ-x3c8n2.
5. Commit: `feat(app): the receiver stops on org-io's typed terminal condition and holds no lock`.

### T17 — app: starts with no passphrase, refuses when the keychain refuses

**Files touched:** app/src-tauri/src/state.rs (`init`, `assemble`), app/src-tauri/src/policy.rs (`resolve_passphrase` deleted), app/src-tauri/tests/startup_policy.rs, app/src-tauri/tests/state_assembly.rs, app/src/** (the passphrase prompt, if the Svelte UI has one), app/docs/requirements/2026-09-14-tauri-shell.md, app/docs/risk/2026-09-14-app-hazards.md, app/docs/architecture/2026-10-05-decomposition.md
**Parallel:** no (after T16)
**IDs verified:** REQ-7g3k9a, RC-xt4qr3, LLR-tpkmh3, LLR-vqkr5t (amended)

1. Red tests in `startup_policy.rs`: `// verifies: LLR-tpkmh3`
   `startup_reads_no_passphrase` (source scan: no `ODS_PASSPHRASE` in
   `app/src-tauri/src`); `// verifies: LLR-tpkmh3, REQ-7g3k9a`
   `a_keychain_refusal_fails_startup_naming_the_condition` (the in-memory
   keychain set to `Locked`; `assemble` returns the error naming the
   Persona and "locked").
2. Implement: `init` and `for_test` call org-io's `OrgIo::open(data_dir,
   transport_mode)` (S2's call site, `state.rs:119` and `:194`, without the
   passphrase; reconciled 2026-10-08); no passphrase anywhere; the UI loses
   its prompt. `state_assembly.rs`'s
   `a_store_under_another_passphrase_is_refused` is deleted with the
   behaviour (recorded in the task report; T9).
3. Docs (dated notes): REQ-7g3k9a, RC-xt4qr3, LLR-tpkmh3 become "fail to
   start, naming the condition, when the keychain refuses" (org-io's
   REQ-6d9xv6, narrowed); SDD-aq7m6b, LLR-vqkr5t (no passphrase).
4. Green: the app's three verify lines; `check-trace.sh` for the app clean
   but for pre-existing advisories.
   *Added 2026-10-13 (raised by T9):* once LLR-85zque moves to org-io, three
   app tests still cite it, and only a REQ can be cited across units.
   They are `init_opens_the_store_once_even_when_the_connect_is_refused`,
   `a_store_under_another_passphrase_is_refused` (deleted here) and ipc's
   `connection_status_reports_a_data_dir_it_had_to_create_verbatim`
   (`LLR-85zque, LLR-vqkr5t`). A test this task must edit anyway (the
   passphrase leaves `init`) drops the org-io ID in that edit and keeps or
   gains the app item it verifies (LLR-vqkr5t). If one stays green and
   unedited, stop and ask: the hybrid rule forbids re-annotating it.
5. Commit: `feat(app): start with no passphrase; refuse when the keychain refuses`.

### T18 — Trace, coverage and the verification record

**Files touched:** Makefile, org-io/.guardrails/config.yaml, org-node/.guardrails/config.yaml, app/.guardrails/config.yaml, .github/workflows/rust.yml, docs/verification/<date>-worktree-org-io-transport.md (new), this plan's Progress
**Parallel:** no (last)
**IDs:** none new

1. `Makefile`: add S4's org-io test targets (`keychain`, `store_files`,
   `transport_handshake`, `transport_networked`, `endpoints`,
   `preflight_transport`, `receive_step`, `acknowledgement_inbound`,
   `send_fanout`, `peer_addr`, `fuzz_peer_addr`,
   `invitation_exchange`, `node_stories`) to `ORG_IO_COVERAGE_ARGS` (S2's
   variable, T0's table). Measure with the floors at 0:
   `make coverage-org-io` and `make coverage-branch-org-io`; record the
   figures in a dated comment as S2 did; set `ORG_IO_LINES`,
   `ORG_IO_REGIONS`, `ORG_IO_BRANCHES` one point below each, rounded down;
   re-run both with the floors: exit 0. A shortfall below class C
   expectations (the iroh and keychain shells) is written beside the floor
   for the owner to accept or not at merge.
2. Each unit's `verify_commands` in its final form (plan header);
   `.github/workflows/rust.yml` test and clippy lines follow.
3. Closing checks, output pasted into Progress:
   `.guardrails/scripts/check-units.sh`; for each of the six units
   `check-trace.sh` and `check-ids.sh --allow-draft-files`; every unit's
   `verify_commands`; `make coverage coverage-branch`. Expected: exit 0;
   advisories only for items S4 does not touch.
4. Write the verification record (`docs/verification/`, in the form of the
   latest record there), then hand off to `verify-before-merge` and
   `merge-change`.
5. Commit: `build(org-io): S4 coverage floors; verification record`.

*Added 2026-10-14 (dispatcher): items the task reports left for T18,
gathered here. Each source is in Progress. Do them before step 3's closing
checks.*

6. Coverage arguments:
   - Step 1's list gains `--test publish_path --test startup_reconcile`
     (S3b-io's, already in the Makefile; keep them).
   - Drop the two deleted org-node targets (`transport_handshake`,
     `transport_networked`) and `preflight` from `ORG_NODE_COVERAGE_ARGS`.
   - Measure org-node and the app again too: T14 and T15–T17 changed both.
7. Clippy:
   - Make `cargo clippy -p org-node --all-targets --features
     app,test-support -- -D warnings` clean. The hits are in key_custody,
     value_types, fuzz_verify_against_chain and outgoing_bodies (T5's 4);
     PR-6f3qku covers the older ones.
   - Make `cargo clippy -p org-io --all-targets` with no features clean
     (`same_bytes_as` in key_custody_config.rs).
   - Put both lines into the units' `verify_commands` and the CI workflow.
8. Trace:
   - Resolve org-io's NON-EXPORTED-REF LLR-gdfqn9 (T5): an org-io
     absences test cites org-node's item. Rewrite it in prose and re-point
     the test to org-io's own item, or LLR-3zdw8v. The test is green, so
     only the cross-unit ID is removed, with a dated note.
   - Resolve PR-jgvf3b (S3b-io's, org-node): its cause is gone on the
     merged tree. Name the org-io reproducing test in prose.
9. Risk: run `analyze-risks` to re-estimate HAZ-cfp4jb and HAZ-ny7yvt (T16:
   REQ-x3c8n2 met) and HAZ-8ghmhn (T17: the passphrase situation no longer
   arises). Record the new estimates, or why they stand.
10. The verification record lists the items S4 retired, with their
    replacements: LLR-379hnv (T4), LLR-g9vmbx (T6), LLR-23sfdh → LLR-jd82t6
    (T9, owner ruling), LLR-q225ws (T12); and the item it superseded, kept
    with `superseded-by:`: LLR-a9rjtf → LLR-zw6jqy (T16; corrected
    2026-10-14, review round 1 finding-17). It
    also lists the green tests left with no `verifies:` line because their
    only ID moved or retired (T9, T12, T15, T17 reports).
11. The repo-level demo docs (`docs/superpowers/demo/two-instance-demo.md`,
    `two-laptop-paseo.md`) still set `ODS_PASSPHRASE`. Remove those lines
    with a dated note: the variable is now ignored.

### T19 — org-io: tests for the six met expectations; the receiver loop's stop and restart

*Added 2026-10-14 (owner rulings of that day on T18's two open points).*

**Files touched:** org-io/tests/ (a new target, `expectations_met.rs`, or
the existing target that fits each REQ), org-io/Cargo.toml,
org-io/.guardrails/config.yaml (verify line 1), Makefile
(`ORG_IO_COVERAGE_ARGS`), .github/workflows/rust.yml (the org-io test step),
org-io/docs/requirements/ (dated notes only if a REQ's wording needs
clarifying), app/src-tauri/tests/receiver_guard.rs or receiver_events.rs,
app/docs/architecture/2026-10-05-decomposition.md (an LLR if needed),
app/docs/risk/2026-09-14-app-hazards.md
**Parallel:** no (after T18)

1. Ruling 1, "add org-io tests". For each of org-io's six REQs that had
   `expects: org-node` (REQ-jxyh9c, REQ-qza96g, REQ-txkm94, REQ-tzr3dk,
   REQ-v6r2gd, REQ-x6kdse), write a test at org-io's interface (the
   `OrgIo` handle, or the lowest org-io LLR that satisfies the REQ) that
   shows the REQ holds through org-io. Use `// verifies: <REQ or org-io
   LLR>` and never an org-node ID. The behaviour exists, so each test
   carries a recorded mutation check rather than a red-first run: break
   the org-io path the test depends on and watch it fail. Normal-case and
   abnormal-input cases follow the class C robustness rule.
2. Ruling 2, "lower after a test". A test in the app runs the real receive
   loop and:
   - closes the endpoint;
   - shows the loop stops and announces the stop once;
   - shows the start guard is released;
   - shows `start_receiver` can start the loop again.
   It cites an existing app LLR that states this, or a new one with
   `analyze-risks` noted. It must fail first; if the behaviour is already
   there, record a mutation check instead. Then lower HAZ-cfp4jb to S3/P1
   with a dated note and update §7's row. Acceptability is unchanged.
3. Green: org-io's verify lines and clippy, the app's verify lines, and
   check-trace for org-io (expected exit 0) and the app.
4. Commit: `test(org-io, app): the met expectations through org-io; the
   receiver loop stops and restarts (HAZ-cfp4jb to P1)`.

### T20 — org-io: a closed endpoint is not reused (resolves PR-tcd2ta)

*Added 2026-10-14 (dispatcher; found by T19).* `Endpoints` keeps a closed
endpoint. `start_accepting` skips an entry whose `accept_task` is `Some`
even after that task has ended, so a restarted receive loop stops at once
with `EndpointClosed`, and sends from that Persona would reuse the closed
endpoint. S4 introduced this in T10. It is fixed in S4, under
`resolve-problem`.

**Files touched:** org-io/src/endpoints.rs, org-io/src/transport/ (if
needed), org-io/tests/endpoints.rs, org-io/docs/architecture/DRAFT-…-transport.md
(dated notes on LLR-z35vbh, LLR-d2pzzk, SDD-xwft92),
org-io/docs/problems/2026-10-14-receiver-restart.md,
app/src-tauri/tests/receiver_guard.rs (remove the `#[ignore]`),
app/docs/risk/2026-09-14-app-hazards.md
**Parallel:** no (after T19)

Design (dispatcher's default; the owner may overrule): an endpoint whose
accept task has ended, or which reports closed, is dropped from
`Endpoints`. The next `start_receiving` or send for that Persona binds
again from the keychain, lazily, as LLR-d2pzzk already says for a first
bind. An open endpoint is never bound twice (LLR-d5y7ks, LLR-6zjzn2).

1. A red org-io test in endpoints.rs reproduces PR-tcd2ta at org-io's own
   interface. After the endpoint closes, `start_receiving` followed by
   delivering a message reaches the receive channel. A send from that
   Persona also binds again. Watch it fail first (it does today).
2. Fix as above. The open-endpoint tests stay green.
3. Remove the `#[ignore]` from the app's
   `after_a_stop_start_receiver_starts_a_loop_that_runs`; it now passes.
   Lower HAZ-cfp4jb to S3/P1 (dated note, §7's row; acceptability
   unchanged). Resolve PR-tcd2ta in resolve-problem's form.
4. Green: org-io's verify lines and clippy; the app's verify lines;
   check-trace for org-io and the app.
5. Commit: `fix(org-io): a closed endpoint is dropped and bound again on
   next use (resolves PR-tcd2ta)`.

### T21 — org-io: a rebound endpoint accepts while the receiver runs

*Added 2026-10-14 (dispatcher; found by T20).* If one Persona's endpoint
closes while another Persona's is still accepting, the receive loop does
not end. Then nothing calls `start_receiving` again, and a send that
rebinds the closed endpoint starts no accept task on it. That Persona
receives nothing until the next `start_receiving`. It is a defect in S4's
own code, fixed in S4 under `resolve-problem` with a new problem report.

**Files touched:** org-io/src/endpoints.rs, org-io/tests/endpoints.rs,
org-io/docs/architecture/DRAFT-…-transport.md (dated note on LLR-z35vbh),
org-io/docs/problems/ (new draft report, resolved here)
**Parallel:** no (after T20)

Design (T20's option a, the dispatcher's default): once `start_receiving`
has run, `Endpoints` remembers that it is receiving. Every later rebind, by
a send or otherwise, also starts an accept task on the new endpoint. The
count of accept tasks stays exact.

1. A new problem report, then a red test with two Personas: close A's
   endpoint while B's keeps accepting; a send from A rebinds it; a message
   to A's Device reaches the receive channel. Watch it fail first.
2. Fix; all endpoints and receive tests stay green.
3. Green: org-io's verify lines and clippy; the app's cargo line; check-trace
   for org-io.
4. Commit: `fix(org-io): a rebound endpoint accepts while the receiver
   runs`.

### T22 — a check that went stale before its commit

*Added 2026-10-14 (dispatcher; found by the deslop pass).* Since T11,
org-io's receive step releases the node lock between
`OrgService::check_received` and `commit_checked`, so the chain read runs
unlocked. A local commit (a submission, an admission, a settle or a
reconcile) can land on the same Organisation in between. `CheckedMessage`
carries the record's trie, sequence guard and epoch from check time, and
`commit_checked` verifies against them. What happens then is untested. It
may be a refusal (safe), or a commit built on a stale record (a lost local
commit). It is fixed in S4 under `resolve-problem`.

**Files touched:** org-node/src/service.rs, org-node tests (receive_path.rs
or a new target), org-io/src/receiver.rs (if the step must retry),
org-io/tests/receive_step.rs, both units' docs (dated notes on the
`check_received` / `commit_checked` LLRs, LLR-cvyyr8), a new draft problem
report
**Parallel:** no

1. Record a problem report. Reproduce at org-node's interface: check a
   received body, commit a local change to the same Organisation, then
   `commit_checked` the earlier check. Name exactly what happens in the
   report.
2. If the result is a lost or overwritten local commit, or an accepted
   body that a fresh check would refuse, it is a defect. Make
   `commit_checked` refuse a check whose record moved (epoch, root or
   sequence guard), with a typed error. In org-io, the receive step then
   repeats the check and the read once, under the same rules, so a valid
   message is not lost. A test at each interface must fail first.
3. If the result is already a typed refusal that loses nothing, record it.
   Pin it with a test, then show the message is not lost: either org-io
   retries, or the sender's message is shown to be delivered again. If a
   message *is* lost and there is more than one reasonable fix, stop and
   report options.
4. Green: org-node's verify line (and Quint, if a model input changed);
   org-io's verify lines and clippy; the app's cargo line; check-trace for
   all three units.
5. Commit: `fix(org-node, org-io): a check that went stale before its
   commit is refused and repeated`, or `test(...)` if step 3 applies.

### T23 — robustness: both cases for S4's new items

*Added 2026-10-14 (owner ruling on the gate's check 6).* The gate at
`ae38d45` judged each item by test names. Some items S4 added have only a
normal-case test, or only an abnormal-input test. Items that moved into a
unit with their original tests unchanged keep their existing gaps. Those
are recorded in the verification record as known and outside S4's scope.

**Items (S4-new, from the gate's list):** LLR-9arvqj, LLR-h9qf3s,
LLR-9cb443, LLR-e5kqhg, LLR-5wb4vd, LLR-zw6jqy, LLR-szhe7j (abnormal cases
only), LLR-36jer8, and the source-scan items LLR-jd82t6, LLR-t4v69b,
LLR-qezw3n, LLR-4tcxsu. Also check any other item that S4 *added*, listed
in the self-review's item 1 and the T6–T22 reports, against the gate's file
scratchpad/gate2/s4-robust.txt.

1. For each item, read its tests' bodies, not only their names. If both a
   normal case and an abnormal-input case (bad input, a boundary, a failed
   dependency) are already exercised, record that and add nothing.
2. If one is really missing, add a test at the item's own interface that
   cites only the owning unit's items. The behaviour exists, so record a
   mutation check for each new test (what was broken, and the failure).
   For a source-scan item, the abnormal case is the scan catching a
   planted violation: a mutation check against a temporary violating line.
3. Green: each touched unit's verify lines and clippy; check-trace.
4. Commit: `test(org-node, org-io, app): both cases for S4's new items`.

### Merge notes from the S3b-io session (received 2026-10-14, for merge-change)

1. Base: local master at `68ec432` (already merged as `aac4493`). Pass
   `--local-base` to `merge-preflight.sh` at steps 4 and 6c. Before
   squashing, `git merge-base --is-ancestor master HEAD`. If master moved,
   merge it again and rerun from step 1.
2. S3b-io's `2026-10-13-publish.md` ledgers (org-io, org-node,
   on-chain-client, app) are final on master. If `finalize-docs.sh` offers
   to rename one of them, or any DRAFT file S4 did not create, stop.
3. The dated notes on S3b-io items are the right form; don't change them
   beyond a note. **The change to REQ-by9k5q, from "the Device the
   submission named" to "every recipient", needs the owner's explicit OK.
   Record it in the verification record.** *Approved by the owner
   2026-10-14 as a dated clarification, ID kept.* No merged S3b-io test is
   re-annotated.
4. Problem reports:
   - PR-jgvf3b goes under `Resolves:` in the merge message.
   - PR-vt244s stays open (S3b-io part 2, ruling D1); do not resolve or
     narrow it.
   - PR-6nhrr7 stays open in org-io.
   - Both age limits stay at 60 days.
5. `ORG_IO_COVERAGE_ARGS` keeps `--test publish_path --test
   startup_reconcile`; the floors 80/78/82 must pass with them in.
6. Don't edit docs/plans/2026-10-08-org-io-publish.md §10. For part 2, the
   record lists S4's stand-ins (`dial_direct_as`, the kept keychain item,
   acknowledgements the app drops) and the deferred `SendReport` in
   `Settled::Committed.sent`.
7. Review scope: S3b-io's behaviour survived the restructure:
   - an unknown write is never forgotten;
   - the publish lock is taken after the node lock;
   - LLR-pk6mhv's founding rule holds;
   - no public discard method exists (LLR-cc59ys).

## Self-review (plan-change step 9)

1. **Every new LLR has a task whose test verifies it.** org-node:
   LLR-37k2s3, LLR-yp69jg (T1); LLR-bg538c, LLR-2qqqx6, LLR-bvt5up,
   LLR-rdda86, LLR-e5kqhg (T3, LLR-bvt5up also T9); LLR-zetn7a, LLR-6qjmhf
   (T4); LLR-gdfqn9 (T5); LLR-nr8df3, LLR-7q9g9z, LLR-d55brn (T6). org-io:
   LLR-677fqq, LLR-8erfbu, LLR-36jer8, LLR-n9dnm4 (T8, T9); LLR-nt8dja,
   LLR-b9bm36, LLR-murjr6, LLR-uzax55, LLR-t4v69b, LLR-amv4mb (T9); LLR-d5y7ks,
   LLR-d2pzzk, LLR-z35vbh, LLR-h9tcat (T10); LLR-9arvqj, LLR-cvyyr8,
   LLR-szhe7j (T11); LLR-wvgec4, LLR-unqfj5 (T11a, a normal-case and three
   abnormal-input tests);
   LLR-znhmh2, LLR-r9sh6k, LLR-2b2ghe (T12); LLR-h9qf3s, LLR-9cb443,
   LLR-y4dg74 (T13). The REQs and RCs are verified through them; the six
   expectations are met by org-node's exported REQs (no test owed).
   *Corrected 2026-10-14 (T18): under guardrails 0.5.1 a met expectation
   is an ordinary REQ again and owes an org-io test; see Progress, "T18
   report", owner question 1.* S3a's
   items restated over the new interfaces keep their tests, rewritten
   value-driven (T3–T5) or over org-io's transport (T11, T11a).
2. **Real code, commands and expected output** are given for the new types
   and the tests whose shape a reader could not take from an existing
   file; moved tests name their source and the edit.
3. **Names are consistent** with the design drafts as reviewed on
   2026-10-07 and reconciled with S3a the same day: `DeviceSecret::{generate, from_keychain_bytes, device_key,
   lend_to_keychain}`, `SealedStore`, `StoreWrite`, `StoreChange`,
   `PersonaStore::unseal`,
   `OrgService::new(stores)`, `check_received(sender, body)`, `Checked`,
   `CheckedMessage`, `commit_checked`, `ReceiveKind`, `CommitOutcome {
   sender, outgoing, acknowledgements }`,
   `OutgoingMessage`, `build_invite`, `build_reply`, `check_reply`,
   `settle`; org-io `read_device_secret`, `write_device_secret`,
   `with_endpoint_seed`, `write_store`, `remove_store`, ~~`open_node`~~
   `OrgIo::open(data_dir, transport_mode)` (S2's, changed in place; §17),
   `OrgIo::create_persona`, `Submitted`, `SendReport`,
   `bind_from_keychain`, `check_transport`,
   `Endpoints`, `start_receiving`, `next_received`, `Received`, `Inbound`,
   `Verdict`, `ReceiveError`,
   `send_committed`, `parse_peer_addr`, `issue_invite`, `produce_reply`,
   `import_reply`, `admit_reply`. S2's names are mapped in §17.1
   (2026-10-08).
4. **Files touched and Parallel** are stated for every task. Parallel pairs:
   T1 ∥ T2 (disjoint files); T7 ∥ T4, T5 (org-io manifest and docs vs
   org-node and org-io sources); T8 ∥ T4, T5 (shares `org-io/src/lib.rs`'s
   module list and, since the S2 reconciliation, the transitional secret
   holder T3 put there: keep both sides when merging); T13 ∥ T14 (org-io
   and app vs org-node). Every other pair is serial, T11a included (it
   extends T11's `receiver.rs`).
5. **S2's rules** (§17.2) are checked per task: every new public method of
   the handle returns allowlisted public-data types (T3, T5, T9, T11, T12,
   T13 name what joins or leaves `HANDLE_RETURN_ALLOWLIST`); no private key
   is reachable through a public operation (T3, T5, T8, T15); keychain
   access is org-io's only (T8); no key or seed is read from an environment
   variable outside test environments (T10, ruling 12).

## Progress

| Task | State | Commit | Notes |
|---|---|---|---|
| Reconcile with S3a (`60c7d58`); owner rulings 7 and 8 drafted | done | (this commit) | §16; T0 step 0 re-reconciles against S2 |
| Merge master (S2 `e41cb09`) into this branch | done | `24ca24c` | T0 steps 1–3 |
| Reconcile with S2 (`e41cb09`); owner rulings of 2026-10-08 (9–12) applied | done | (this commit) | §17; T0 steps 0 and 4; see the note below |
| T0 | done | `24ca24c`, (this commit) | step 5 baseline 2026-10-08: see "T0 step 5 baseline" below |
| T1 | done | `fc7dcd4` (merged) | see "T1 report" below |
| T2 | done | (merged, 2 commits) | see "T2 report" below |
| T3 | done | `9705c9b` (merged) | see "T3 report" below; one open point for the owner before T4 |
| T4 | done | `21cc014` (merged) | see "T4 report" below; merged on top of T8, org-io lines re-run green |
| T5 | done | `3096bdb` (merged) | see "T5 report" below; PR-gnh3j2 resolved |
| T6 | done | `c7def0d` (merged) | see "T6 report" below; LLR-379hnv and LLR-g9vmbx retired |
| T7 | done | `67a5c57` (merged) | see "T7 report" below; mock not shared |
| T8 | done | `d5ba412` (merged) | see "T8 report" below |
| T9 | done | `ad2c5f6`, `9b8ea7c`, `6a68207` (merged) | see "T9 report" below; owner ruling: LLR-23sfdh superseded by LLR-jd82t6 and retired; PR-n696wv, PR-jav8zn resolved |
| T10 | done | `494f79a` (merged) | see "T10 report" below; PR-8qsnhx resolved; | the transport preflight by the keychain path (ruling 12; data directory `ODS_DATA_DIR`, no default — ruled 2026-10-08, §17.4); REQ-v4tfap's note moved to T3 |
| T11 | done | `63667aa` (merged) | see "T11 report" below; PR-7v8anh, PR-kwwap5 resolved |
| T11a | done | `97c0022` (merged) | see "T11a report" below; | owner rulings 8, 9; HAZ-abm3xs's residual accepted 2026-10-08 (ruling 10) |
| T12 | done | `b3672cc`, `1472c74`, `a6f3c5d` (merged) | see "T12 report" below; LLR-q225ws retired; PR-2dmjzj, PR-3ue4va resolved |
| T13 | done | `5405f02` (merged) | see "T13 report" below |
| T14 | done | `f06dcc2` (merged) | see "T14 report" below; combined tree after T13+T14 re-verified |
| T15 | done | `9ba5c4c` (merged) | see "T15 report" below; first agent stopped by the owner, a second finished in the same worktree; PR-7zz76t resolved (obsolete) |
| T16 | done | `52980a8` (merged) | see "T16 report" below; LLR-a9rjtf superseded by LLR-zw6jqy; REQ-x3c8n2 met |
| T17 | done | `7b5b441` (merged) | see "T17 report" below; app check-trace exit 0 |
| T19 | done | `5797f7a` (merged) | see "T19 report" below; ruling 1 done; ruling 2's condition not met → T20 |
| T20 | done | `8997cf6` (merged) | see "T20 report" below; PR-tcd2ta resolved; HAZ-cfp4jb S3/P1 |
| T21 | done | `2fe77c2` (merged) | see "T21 report" below; PR-47aycj resolved |
| deslop | done | `11c3153`…`b45d093` | see "Deslop pass" below; one behaviour finding → T22 |
| T22 | done | `dd9669a` (merged) | see "T22 report" below; PR-erskh4 resolved |
| rename | done | `282d4e4` (merged) | 2026-10-14, owner's request: the 15 `DRAFT-` ledger files renamed to `<first-added date>-<slug>.md`, every reference updated (21 ledger files, this plan's 16 lines); no collisions; `git ls-files \| grep -i draft` empty; check-ids (no draft flag), check-trace and check-units exit 0 for org-node, org-io, app. `finalize-docs.sh` therefore has nothing to rename at merge |
| gate | checks 1–3, 5, 7 green at `ae38d45` | logs: scratchpad/gate2/ | 1280 Rust tests passed, 0 failed (1 ignored, org-members); vitest 52; Quint 63 + six invariant runs clean; 13 fuzz targets clean; check-trace, check-ids (no draft flag) and check-units exit 0 for all six units; coverage floors met (org-io 81.56 / 80.42 / 84.51); tree clean. **Owner 2026-10-14:** org-io's class C coverage gap accepted as documented (to be recorded in the verification record); check 6's robustness gaps for S4-new items → T23, moved items' gaps recorded as known |
| merge r1 | review round 1 at `4cf1f0f`: not ready (findings 1–3 medium, 4 owner point, rest low or record) | | pre-flight `--before-review --local-base` passed; findings verbatim go into the verification record. Fix tasks `fa` (org-io: 1, 2, 6, 9, 12, 15, 16, 19; files open PRs for finding-5 and the T22 catch-up point = finding-4, per the owner's explanation that the decline itself is by design), `fb` (org-node: 7, 8, 9, 10 as an open PR, 11, 12, 16), `fc` (app: 3, 13, 14, 15). Plan: finding-17 corrected here. finding-18: the record is written at 6b. finding-20 (terminology, predates S4): recorded in the record as known. Then rerun from step 1 and a second review round |
| r1 fixes | merged: `fa` (e2aab16…bbf4ce7), `fb` 6b44a32, `fc` ede9fd8 | | **finding-1** fixed: the endpoint module private, `OrgEndpoint`/`BindPlan`/`InboundReply` public only under test-support; new scan `the_endpoint_and_its_constructor_are_reachable_only_in_test_builds` (LLR-d5y7ks), red on the unchanged tree. **finding-2** fixed: PR-y6t5kt filed and resolved; `Endpoints::accept_if_receiving` from `create_persona` (a bind failure is swallowed and the creation stands; dispatcher accepts) and `produce_reply` (refuses with `ExchangeError::Bind`); tests `a_persona_created_while_the_receiver_runs_is_received_on`, `no_reply_is_produced_while_receiving_for_a_persona_whose_endpoint_cannot_be_bound` (LLR-z35vbh, PR-y6t5kt), both red first. **finding-3** fixed: REQ-x3c8n2's Met narrowed; new app REQ-q89tjr (`expects: org-io`, the Organisation half), UNMET-EXPECTATION again. **finding-4** → PR-m3tfnu (open, org-io). **finding-5** → PR-7hu3v5 (open, org-io). **finding-6, 7** dated clarifications (REQ-n82q47, REQ-x3jrh8). **finding-8, 9, 11, 12** strengthened scans and 9 new tests, each mutation-checked (`a_device_secrets_seed_is_held_in_zeroizing`, `the_sans_io_scan_catches_grouped_file_imports_and_environment_reads`, `a_record_that_appeared…` / `…went_since_the_check_is_refused_as_moved`, two LLR-rdda86 tests, LLR-2qqqx6 and LLR-gdfqn9 abnormal tests, the NoNativeBackend reason, `the_binary_without_a_data_dir_exits_non_zero_before_any_check`); the over-MAX_MESSAGE body is not built (thousands of Members). **finding-10** → PR-6gf4t2, PR-gq83cs (open, org-node). **finding-13, 14** inherited: the three receiver tests from LLR-a9rjtf (→ LLR-zw6jqy) and the three persona tests from LLR-rt8gdz (→ LLR-pms47e); the founding clause is review-only. **finding-15, 16, 19** fixed. **finding-17** corrected in the plan. **finding-18** at 6b. **finding-20** recorded as known. Results: org-node 376, org-io 241 + 11, app 143 + vitest 52; clippy, check-trace and check-ids exit 0 |
| conformance r1 | CODE: 12 findings (2 medium); TEST: 12 findings (5 medium) at `20ae01c` | | CODE task `cf` (aeee172, 2292d32, 533cf37): findings 1, 2, 7, 9, 10, 11, 12 fixed with no behaviour change; 3, 4, 5, 6, 8 recorded open (PR-43t2zn, PR-u784zs, PR-vy83sd, PR-wayqj3, PR-a63sk3); 2's send failures tracked as PR-b8kmk2 until S3b-io part 2. TEST task `ct` (8431a92…75dfd9d): findings 1, 3, 5–11 fixed; finding-2 no change (owner ruling 2026-10-14: the chain is a boundary for org-io's tests; FakeChain stays); finding-12 no change (the upgraded rule keeps a superseded ID beside its successor); finding-4 → task `c4` (LLR-ty85xv, LLR-n6twt7, LLR-ctrfz4 move back to the app, tested at the IPC commands). finding-1's real-keychain run, by the owner in a normal terminal on 2026-10-15: `ODS_REAL_KEYCHAIN_TESTS=1 cargo test -p org-io --features test-support --test keychain_contract` → `the_in_memory_keychain_keeps_the_credential_contract ... ok`, `the_os_keychain_keeps_the_credential_contract ... ok`, 2 passed, 0 failed (the sandbox blocks keychain writes). org-io stands at its problem_open_max of 10. **c4** merged (4c7a3ab): LLR-ty85xv (`satisfies: REQ-vgr7s2`), LLR-n6twt7 and LLR-ctrfz4 (both `satisfies: derived`, as before T12, with their app assessments restored) moved back to the app under SDD-2pa6h6, IDs kept; org-io mentions them only in prose. Seven app IPC tests added (ipc.rs: `revoke_member_treats_a_blank_or_whitespace_only_peer_addr_as_absent`, `revoke_member_does_not_refuse_an_encoded_endpoint_addr`, `revoke_member_refuses_a_peer_addr_that_is_not_hex`, `revoke_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr`, `admit_member_does_not_refuse_a_blank_or_decodable_peer_addr`, `admit_member_refuses_a_peer_addr_that_is_not_hex`, `admit_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr`), each mutation-checked in commands.rs (blank refused, `0x` mis-stripped, the error swallowed); the seven org-io `parse_peer_addr` copies removed (rewritten, not moved: no inherited lines). org-io 236 + 11, app 150, vitest 52; check-trace and check-ids exit 0 |
| conformance r2 | TEST: 5 findings (2 medium); CODE: 9 low (last CODE round) at `a941d1c` | | CODE task `cf2` (013274e, 4179ff8, 20d50a2): all nine fixed (finding-8 with a test; finding-7 without one, the state is unreachable through the public API). TEST task `ct2` (7a1723d): findings 1, 3, 4, 5 fixed; finding-2 (each now cites only the successor, because the old definitions are deleted or the old item moved to another unit; listed as `inherited:` in the record with that note — corrected 2026-10-15, review round 2 finding-14): `forget_organisation_is_defined_once_…` and `a_commit_that_removes_another_member_forgets_nothing` — from LLR-23sfdh (→ LLR-jd82t6), `each_command_sends_once_to_one_device_and_the_kind_follows_the_record` — from LLR-q225ws (→ LLR-r9sh6k), `produce_invite_reply_refuses_without_confirmation_over_ipc` — from LLR-w4mhd4 (→ app REQ-ab2mfz). **Owner rulings 2026-10-15:** delete `state_assembly::init_opens_the_store_once_even_when_the_connect_is_refused` (covered by org-io's store-opening and the app's startup-refusal tests); mint an org-io LLR ("a submission org-node refuses commits, writes and sends nothing") for `submit_flow::a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing` → task `ct3`. `key_custody::two_different_keypairs_do_not_share_a_device_key` has had no `verifies:` since before S4 (dropped at REQ-ztdza4's amendment); left, recorded as known |
| conformance r3 | TEST: 1 low at `b15ca6c` — the loop ends (CODE ended at round 2) | | Task `ct3` (d4bc66e): `state_assembly.rs` deleted (owner ruling); LLR-nqdr97 minted (org-io, derived, assessed) for the refused-revocation test, mutation-checked. Task `ct4` (0b78bc3): the two locked-keychain contract tests re-pointed to LLR-36jer8 (the fake's fidelity); `locked_contract` doc corrected; LLR-nqdr97's abnormal test asserts org-node's `IdNotFound` refusal (mutation: another error → fails), its Normal pointer clarified; `no_receive_branch_is_unreachable` renamed `an_unheld_organisations_notice_is_refused_as_not_held` (LLR-zetn7a note); LLR-h9tcat's two refusals clarified. Results: org-node 374, org-io 238 (+1 ignored) + 11, app 149; clippy, check-trace, check-ids exit 0 |
| master 344faa3 | merged (261ee88) | | guardrails 344faa3 merged, no conflicts, no new findings; adds ARCHITECTURE guidelines kind, `Gate-tree:` trailer guard in finish-merge.sh, record template without gate figures (they go to squash trailers `Base:`, `Gate-tree:`, `Gate:`). |
| arch r1 | ARCHITECTURE: 8 findings (5 medium); fixes merged (`af`: f0ae72c8…5362fce6) | | Findings and dispositions: scratchpad `conformance-arch-round1.md` (session scratchpad). **Owner ruling 2026-10-15: fix all five now.** Task `af` running: iroh confined to org-io/src/transport/ with a new LLR + scan; pure send plan; "Device key" → glossary term; glossary entries in docs/CONTEXT.md; SDD-v2bq89 split; finding-6 fixed; finding-7 io sources kept, iroh errors stay strings (would leak iroh types); finding-8 accepted. **Next:** merge `af`, ARCHITECTURE round 2 (until nothing above low), then merge-change from step 1: prune-plans (`--base master`), gate, step 3 tree check, pre-flight `--before-review --local-base`, second independent review (round 1's 20 findings: scratchpad `review-round1-findings.md`; conformance rounds: `conformance-{code,test}-round{1,2,3}.md`), record (template `.guardrails/templates/verification.md`; owner acceptances: REQ-by9k5q clarification, org-io coverage gap, chain-as-boundary, two test rulings), final pre-flight, squash with `Base: master at <commit>` (local base, remote out of scope per the S3b-io notes), `Gate-tree:`, `Gate:` trailers; hand the owner `git commit -S -F … && sh .guardrails/scripts/finish-merge.sh worktree-org-io-transport` |
| af result | merged | | finding-1: `peer_addr.rs` moved to `org-io/src/transport/`; `OrgEndpoint::{send_body_to_device, send_body_at, has_relay_home}` replace iroh use in send.rs and preflight.rs, `addr()` removed; new LLR-3vb4k9 (`satisfies: REQ-xuzw32`) and scan `iroh_is_named_only_in_the_transport_module_and_test_support`, red on the old tree (send.rs:124, peer_addr.rs:11/28/35/75). finding-5: SDD-v2bq89 narrowed to the fan-out; new SDD-wr8m5n (the peer-address parse), LLR-2b2ghe re-traced. finding-2: pure `send::plan_send`, no behaviour change. finding-3: "Device key" → "DevicePublicKey" in new error text. finding-4: docs/CONTEXT.md gains Device seed, DeviceSecret, Keychain item, SealedStore, Store file, Verdict ("Device secret" under Avoid). finding-6 fixed. finding-7: `OpenError::CreateDataDir` keeps its `io::Error` as `source()`; `StoreFileError` unchanged (an error field breaks clippy's large-error limit and LLR-b9bm36's stated shape); iroh errors stay strings. Results: org-io 239 (+1 ignored) + 11, org-node 374, app 149; clippy, check-trace, check-ids exit 0 |
| arch r2–r3 | r2: 5 findings (3 medium, wording) → task `af2` (1b4c7394…f1db9591): org-node error text, Revoke.svelte hint, preflight "store file", doc comments use glossary terms; docs/CONTEXT.md gains a dated `device_key` identifier exemption (dispatcher, matching PersonPublicKey's); r2 finding-5 = open PR-u784zs. r3: 1 low — **the conformance loop ends for all three kinds** | | r3 finding-1 accepted: postcard is org-node's one encoding, called from five modules on master; the moved Blob codec follows that pattern; one postcard adapter is a later refactor. Rustdoc still saying "Device key" (~35 sites) recorded as known. Findings and dispositions per round: scratchpad `conformance-{code,test,arch}-round*.md` |
| merge r2 | review round 2 at `12e1147`: not ready (findings 1–3 medium; 4–11 low; 12–14 record; a supplementary pass adds A3 medium, A6/A7 low and test gaps) | | Findings: scratchpad `review-round2-findings.md`. Pre-flight `--before-review` (no `--local-base`: origin/master, at S2, is an ancestor; local master is the base) passed. **Owner ruling 2026-10-15: fix the mediums and mechanical lows; file the bigger lows as one umbrella org-io report; raise org-io's problem_open_max from 10 to 15.** Tasks: `ra` (org-io receive path: finding-1 per-connection accept with a time limit, hazard, LLR; A6; A7; finding-5; finding-7 org-io), `rb` (org-io store/publish/records: finding-2, A3, finding-8 keychain, finding-12 keyring tracker, finding-13 org-io, the limit raise, the umbrella report for findings 4, 6, A8, B(e) and test gaps), `rc` (org-node and app: finding-3, 8, 9 filed, 10, 11, 7 app, 13 app, 14 app). finding-14's plan half fixed here. Then rerun from step 1 and review round 3. **Merged:** `rb` (f5ccc170, a75e25c3, 8e4c3464: store writes pub(crate) + scan; A3 settle fix PR-9q5ems resolved — case (b) reports NotExecuted only at expected+1 without the update, else reads the chain; keyring tracker reviewed 2026-10-15, only #163 Windows concurrency, in the umbrella; problem_open_max 15; umbrella PR-k5tdxj open), `rc` (5f6daebf, 83754a48: `lend_to_keychain` returns `Result<(), E: Error>`, PR-aa7prn resolved; PR-qpam4k and PR-bjaq7j open), `ra` (104cba2b: per-connection tasks, 30 s limit, 16 at once; HAZ-4hj5m9, RC-96w7nf, REQ-mnd9rj, LLR-8uf9a5; PR-d2p5y2 resolved; A6 flag set first; A7 closing endpoint rebound; finding-7 org-io tests). **Owner ruling 2026-10-15:** HAZ-4hj5m9's S3/P1 residual accepted (benefit-risk, as HAZ-4n7pss). The A3 settle fix restores LLR-t98pk9/REQ-by9k5q's own clause; offered to the owner for confirmation |
| T23 | done | `e280baf` (merged) | owner ruling on gate check 6. Three missing cases added, each mutation-checked: org-io `receive_step::a_command_completes_while_the_steps_chain_read_stalls` (LLR-9arvqj; the node lock taken before `read_state` → "Elapsed"); org-node `sealed_store::create_persona_refuses_a_seed_whose_store_is_already_held_and_keeps_nothing` (LLR-e5kqhg; the duplicate check disabled → `Ok(PersonaId…)`); app `receiver_guard::a_refused_and_a_mutating_command_complete_while_the_receiver_waits` (LLR-5wb4vd; the node lock taken before the inbound wait → "Timeout"). By test bodies, both cases already present for LLR-h9qf3s, LLR-9cb443, LLR-szhe7j, LLR-zw6jqy, LLR-jd82t6, LLR-nr8df3, LLR-d55brn, LLR-amv4mb, LLR-murjr6, LLR-wvgec4, LLR-unqfj5, LLR-r9sh6k, LLR-2b2ghe, LLR-pms47e. Scan items with a planted violation recorded in a dated comment: LLR-36jer8, LLR-t4v69b. LLR-qezw3n and LLR-4tcxsu: the plant was refused by the session's permission classifier; their abnormal case is T14's red run against the real violations before the deletions ("src/keys.rs contains `iroh`", "src/transport/endpoint.rs contains `node_addr_for_dial`"). Flagged items that moved with unchanged tests are known gaps outside S4 (verification record). Result: org-node 368, org-io 237 + 11, app cargo 143; clippy, check-trace, check-ids exit 0 |
| T18 | steps 1–3, 5–11 done | `2f4a269` (merged) | see "T18 report" below; step 4 (the record) after the deslop pass; two owner points open |

Execution order: T0 → (T1 ∥ T2) → T3 → T4 ∥ (T7 → T8) → T5 → T6 → T9 →
T10 → T11 → T11a → T12 → (T13 ∥ T14) → T15 → T16 → T17 → T18.

### T0 step 5 baseline, 2026-10-08

Every unit's `verify_commands` at `f11d9eb`: green.

- person: `cargo test` 123 passed; clippy clean.
- org-members: `cargo test` 271 passed, 1 ignored; quint typecheck (3),
  `quint test` 63 passing, `quint run … mbtInv` no violation.
- on-chain-client: `test-support` 73 passed; `test-support,write` 39 passed.
- org-io: `test-support` 67 passed; `test-support,dev-seed` 11 passed;
  build and clippy clean.
- org-node: `app,test-support` (28 targets) 304 passed; quint typecheck (2)
  and the five `quint run` invariants, no violation.
- app: `src-tauri` (10 targets) 158 passed; `npm run check` 0 errors,
  0 warnings; `npm run test` failed in the agent sandbox only (EPERM writing
  `node_modules/.vite-temp` through the symlinked main-checkout
  `node_modules`); the same suite through `npx vitest run --configLoader
  runner`: 44 passed.

Checks: `check-units.sh` exit 0 (6 units, 710 tracked paths);
`check-ids.sh --allow-draft-files` exit 0 for all six units.
`check-trace.sh`:

| Unit | Exit | MISSING-TEST | UNRESOLVED-PR | UNMET-EXPECTATION |
|---|---|---|---|---|
| app | 0 | 0 | 14 | 1 |
| on-chain-client | 0 | 0 | 5 | 0 |
| org-io | 1 | 49 | 3 | 1 |
| org-members | 0 | 0 | 0 | 0 |
| org-node | 1 | 19 | 10 | 1 |
| person | 0 | 0 | 0 | 0 |

No other finding kind. The MISSING-TEST counts are S4's drafts, as expected.

### T1 report, 2026-10-08

Tests added (org-node/tests/key_custody.rs):
- a_device_secret_debug_renders_only_the_marker — verifies: LLR-37k2s3
- a_device_secret_declares_zeroize_on_drop — verifies: LLR-37k2s3
- the_device_key_is_the_ed25519_verifying_key_of_the_seed — verifies: LLR-37k2s3
- a_generated_secret_differs_from_the_next_one — verifies: LLR-37k2s3
- from_keychain_bytes_refuses_any_length_but_32_and_echoes_no_byte — verifies: LLR-yp69jg
- lend_to_keychain_calls_the_writer_exactly_once_with_the_seed — verifies: LLR-yp69jg
- device_secret_offers_only_its_four_public_operations — verifies: LLR-37k2s3
- a_device_secret_cannot_be_cloned_copied_displayed_or_serialised — verifies: LLR-37k2s3

red -> green (all eight red at compile time before `DeviceSecret` existed:
`error[E0432]: unresolved import org_node::keys::DeviceSecret`; green after,
key_custody 18 passed):
- a_device_secret_debug_renders_only_the_marker — watched fail for the right reason before the implementation existed
- a_device_secret_declares_zeroize_on_drop — watched fail for the right reason before the implementation existed
- the_device_key_is_the_ed25519_verifying_key_of_the_seed — watched fail for the right reason before the implementation existed
- a_generated_secret_differs_from_the_next_one — watched fail for the right reason before the implementation existed
- from_keychain_bytes_refuses_any_length_but_32_and_echoes_no_byte — watched fail for the right reason before the implementation existed
- lend_to_keychain_calls_the_writer_exactly_once_with_the_seed — watched fail for the right reason before the implementation existed
- device_secret_offers_only_its_four_public_operations — watched fail for the right reason before the implementation existed
- a_device_secret_cannot_be_cloned_copied_displayed_or_serialised — watched fail for the right reason before the implementation existed

inherited: none. Result: org-node verify line 312 passed, 0 failed.

Notes:
- `DeviceSecret` holds `{ seed: Zeroizing<[u8; 32]>, device_key:
  DevicePublicKey }`, not the sketched single field: person's
  `DevicePublicKey` parse is fallible (canonical, torsion-free, not small
  order), so a private `from_seed` derives and parses the public key once
  at construction and `device_key()` stays infallible without a panic.
  `from_keychain_bytes` maps a parse refusal to `InvalidField { field:
  "device secret" }`; `generate` retries until the parse accepts (always
  the first try for a seed-derived key). The public key may be exposed; the
  seed still only leaves by `lend_to_keychain`.
- **Pre-existing, for T18:** `cargo clippy -p org-node --all-targets
  --features app,test-support -- -D warnings` (the plan header's org-node
  clippy line, not in today's `verify_commands`) is red at `73d06d8`
  before T1: `panic`/`unwrap_used`/`expect_used` in
  `transport_handshake.rs`, `value_types.rs`,
  `fuzz_verify_against_chain/fuzz_target.rs`, and `key_custody.rs`'s
  `?Sized`/`Clone` probe and three `assertions_on_constants`. T1 added none.
  T18 must make it green (test-file allows per the conventions) before the
  line joins `verify_commands`.

### T2 report, 2026-10-08

Tests added:
- an_encoded_message_is_its_postcard_body_with_no_length_prefix (org-node/tests/wire_frame_bound.rs) — verifies: LLR-er2x8n
- a_body_over_the_bound_is_refused_before_decoding (org-node/tests/wire_frame_bound.rs) — verifies: LLR-8kh3zf
- a_body_that_is_not_a_wire_message_is_malformed (org-node/tests/wire_frame_bound.rs) — verifies: LLR-pkruy8
- a_wire_message_over_the_bound_is_classified_as_a_receiver_error (app/src-tauri/tests/receiver_events.rs) — verifies: LLR-7bk6qh

red -> green:
- an_encoded_message_is_its_postcard_body_with_no_length_prefix — watched fail for the right reason before the implementation existed (compile: E0432 unresolved import `org_node::wire`; E0599 no variant `MessageTooLarge`)
- a_body_over_the_bound_is_refused_before_decoding — watched fail for the right reason before the implementation existed (same compile run)
- a_body_that_is_not_a_wire_message_is_malformed — watched fail for the right reason before the implementation existed (same compile run)
- a_wire_message_over_the_bound_is_classified_as_a_receiver_error — watched fail for the right reason before the implementation existed (compile: E0004 `MessageTooLarge` not covered in `src/events.rs`)

inherited: none. The 11 existing `wire_frame_bound.rs` tests were
re-pointed with names and `verifies:` lines unchanged; `frame_round_trips`
(LLR-fa7jt8) still tests the transport's framing.

Result: org-node verify line exit 0 (wire_frame_bound 14 passed);
`fuzz_wire_decode` with no features passes; org-io verify lines, build and
clippy green; app `src-tauri` 159 passed (158 + 1). `check-trace.sh` counts
unchanged from T0 for org-node, org-io and the app; `check-ids.sh
--allow-draft-files` exit 0.

Notes:
- Beyond Files touched: eight more org-node tests imported
  `transport::wire` (imports only); the app's `classify_receive_error`
  (`events.rs`) is an exhaustive match on `OrgNodeError`, so the new
  `MessageTooLarge` variant joined its ReceiveError arm, red first, to keep
  the app compiling at T2.
- The length prefix stays as `pub fn transport::encode_frame` (`len ‖
  encode_message`), used by the send, so `frame_round_trips` keeps a
  function to call. `TransportError` gained `From<OrgNodeError>`
  (`MessageTooLarge` → `FrameTooLarge`, else `Malformed`); `recv_one` and
  `receive_message` behave as before. Both go with the transport in T10/T14.
- REQ-eg5j8u's and RC-gfn6kr's notes point to org-io's stream requirement
  in prose, not by ID (org-node does not depend on org-io:
  UNDECLARED-DEPENDENCY otherwise).
- Golden pins were already body form (`framed[4..]`): unchanged.
- `MessageTooLarge` is documented under LLR-8kh3zf's T2 note;
  LLR-j5vbqj lists only the key-pair change's four variants, so it needs
  no note.
- The org-node all-targets clippy line is red only for the pre-existing
  hits listed under T1's report (T18).

### T3 report, 2026-10-13

Tests added:
- sealed_store::a_sealed_store_unseals_with_its_secret_and_device_key — verifies: LLR-bg538c
- sealed_store::unseal_refuses_each_malformed_input_with_its_own_error — verifies: LLR-bg538c
- sealed_store::the_store_key_is_hkdf_sha256_of_the_seed_under_the_versioned_label — verifies: LLR-2qqqx6
- sealed_store::a_created_store_opens_under_the_hkdf_key_of_its_secret — verifies: LLR-2qqqx6
- sealed_store::no_argon2_and_no_passphrase_remain — verifies: LLR-2qqqx6
- sealed_store::every_mutating_operation_returns_one_store_write_with_a_fresh_nonce — verifies: LLR-bvt5up
- sealed_store::no_sealed_store_holds_the_device_seed — verifies: LLR-bvt5up
- sealed_store::saving_needs_no_device_secret — verifies: LLR-bvt5up
- sealed_store::a_commit_that_removes_this_node_signs_with_the_supplied_secret_and_reads_none_otherwise — verifies: LLR-hby4jr, LLR-uw7nmv
- sealed_store::a_removal_returns_store_change_forgotten_for_that_personas_device_key — verifies: LLR-pba7yu
- sealed_store::two_stores_with_one_device_key_are_refused — verifies: LLR-rdda86
- sealed_store::an_operation_changes_only_its_personas_store — verifies: LLR-rdda86
- sealed_store::list_personas_returns_one_record_per_store — verifies: LLR-rdda86
- sealed_store::create_persona_returns_the_secret_and_keeps_only_the_public_key — verifies: LLR-e5kqhg
- fuzz_unseal (bolero, `harness = false`) — verifies: LLR-bg538c
- org-io absences::device_secret_is_named_only_in_its_sanctioned_places — verifies: LLR-3zdw8v

red -> green:
- the 14 `sealed_store` tests and `fuzz_unseal` — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `org_node::store::{SealedStore, StoreRefusal, StoreWrite}`, `org_node::service::StoreChange`; E0599 no `PersonaStore::unseal`; missing variants `StoreRefused`, `DeviceSecretMismatch`, `DuplicatePersonaStore`; `OrgService::new` returned no `Result`); green after: 14 passed, `fuzz_unseal` 1 s, no panic
- device_secret_is_named_only_in_its_sanctioned_places — watched fail for the right reason before the implementation existed (runtime: "the transitional holder holds the secrets")

inherited: none. Deleted with LLR-rgdx22's dated narrowing note:
org-io `open.rs::open_refuses_a_store_under_another_passphrase_and_opens_no_other`.
Existing tests adapted at their call sites, annotations unchanged, with a
dated comment where behaviour changed (LLR-q5n28x's bound now nonce + tag,
40 bytes; LLR-pba7yu, LLR-r8qhky, LLR-r7zm39 over one-Persona stores;
LLR-sz4xhc's `DeviceSeed` assertions removed; LLR-23sfdh's scan watches
`orgs` and `stores`, `OrgService::adopt` exempt, mutation-checked).

Result: org-node verify line + `sealed_store`, `fuzz_unseal`: 329 passed;
org-io lines 67 and 11 passed, build and clippy clean; org-node all-targets
clippy only the pre-existing hits. `check-ids.sh --allow-draft-files` exit 0
(org-node, org-io, app); `check-trace.sh` org-node MISSING-TEST 19 → 9,
org-io 49, other advisories unchanged; app exit 0 (its build is red until
T15, as planned).

Notes:
- Landed signatures beyond the task text: `expect_admission(rng,
  persona_id, org_id)`; `apply_receive`/`apply_self_delete` take the
  `device_secrets` closure and return `(outcome, StoreChange)` (they sign;
  T4's `commit_checked` replaces them); `list_personas`/`list_orgs` return
  `Vec<&_>`; `reconcile` returns `(Reconciled, Option<StoreChange>)`;
  `Accepted.store` and `Reconciled::Removed.store` are `Option<StoreData>`;
  `PersonaStore::create` borrows the secret; `bind_endpoint` and the
  preflight build the `SigningKeypair` inside the crate from the
  `DeviceSecret`. `transport_handshake`, `preflight` and
  `fuzz_verify_against_chain` now need `test-support`.
- The transitional holder `org-io/src/device_secrets.rs` keeps seed bytes
  in `Zeroizing` and builds a fresh `DeviceSecret` per read (the same
  Persona binds and signs); production supplies none for signing, so a
  signing removal is refused with `DeviceSecretNotSupplied` until T8.
- Sends no longer bind lazily: tests bind through the transitional
  `bind_endpoint`; production `submit_commit_send` refuses to send until
  T10. `ConnectFailed` carries org-io's `LocalStore` (pub type, private
  fields).
- `#[must_use]` texts and two notes cite org-io items in prose, not by ID
  (org-node does not depend on org-io).
- One full org-node run failed two `admission_sender` tests on a transient
  iroh "invalid peer certificate: UnknownIssuer"; 8 isolated reruns and two
  later full runs green. Watch for it; the transport leaves org-node in
  T10/T14.
- Spike crates not built under the new workspace lint (offline deps).
- ~~**Open for the owner:** two Personas on one node bound to the same
  Organisation (REQ-yp75u9 rules only the other direction).~~ *Ruled
  2026-10-13:* "An individual can only be member of an organisation once
  (yes, this is only enforceable on a single device, but that is good
  enough)." Minted REQ-uy7kfq (org-node requirements draft) and LLR-yexf5m
  (under SDD-xshv5b), refusing a second Persona's expectation of an
  Organisation another Persona on the node holds or expects. Its tests are
  in T6. T4 needs no change: a first admission commits only for an
  expected Organisation.

### T7 report, 2026-10-13

Files: org-io/Cargo.toml, Cargo.lock, org-io/docs/architecture/soup.md,
org-io/docs/architecture/2026-10-07-transport.md
(LLR-36jer8 dated T7 note), docs/plans/2026-10-06-org-io-roadmap.md (a new
"(later)" row for the Android keychain). Tests added: none (an assessment;
the probe was a scratch test, deleted). Result: org-io lines 67 and 11
passed, build and clippy clean; `check-ids.sh` exit 0; org-io
`check-trace.sh` counts unchanged.

- keyring pinned `=3.6.3` (the last 3.x), `default-features = false`,
  features `apple-native`, `windows-native`, `sync-secret-service`,
  `crypto-rust`; MSRV 1.75; MIT OR Apache-2.0.
- Mock-sharing probe (macOS aarch64; the mock code is the same on every
  platform): **not shared**. A second `Entry` for the same service and user
  reads `NoEntry` (persistence "EntryOnly"). T8 supplies
  `test_support::MemoryKeychain`.
- Backends: macOS and iOS `apple-native` (security-framework); Windows
  `windows-native` (windows-sys); Linux `sync-secret-service`
  (dbus-secret-service → libdbus-1, needed at build and at run time);
  Android none. The Windows and iOS builds were assessed from source only
  (no target installed; `~/.rustup` is read-only).
- **Finding, put into T8:** with no native backend enabled (Android
  included), keyring 3.6.3 silently defaults to its non-persistent mock.
  T8 refuses explicitly on such targets (T8 step 2, added 2026-10-13).
- Recorded in the SOUP row for T8/T10: keyring calls block (Secret Service
  IPC), so keep them off async worker threads; Windows Credential Manager
  does not serialise concurrent access to one item; the keyring issue
  tracker is to be reviewed before merge.
- keyring 4.x (keyring-core, with an Android store crate; MSRV 1.88 against
  org-io's 1.85) was not taken. The plan settles "current 3.x" and defers
  Android, and the roadmap's Android row names 4.x as the candidate.
- New crates are only in `CARGO_HOME=/tmp/cargo_home_fuzz` until the
  owner's own cargo fetches them.

### T8 report, 2026-10-13

Tests added:
- keychain::a_written_secret_reads_back_as_the_same_device_key — verifies: LLR-677fqq
- keychain::the_item_is_named_for_the_device_key — verifies: LLR-677fqq
- keychain::a_secret_under_another_device_key_is_refused_as_key_mismatch — verifies: LLR-677fqq
- keychain::with_endpoint_seed_lends_the_seed_once_and_refuses_a_short_item — verifies: LLR-677fqq
- keychain::a_missing_item_is_item_missing — verifies: LLR-8erfbu
- keychain::a_locked_keychain_is_locked — verifies: LLR-8erfbu
- keychain::any_other_failure_is_unavailable_with_no_secret_byte_in_its_reason — verifies: LLR-8erfbu
- keychain::a_target_without_a_native_backend_refuses_as_unavailable — verifies: LLR-8erfbu
- keychain::the_write_lends_the_bytes_only_through_lend_to_keychain — verifies: LLR-n9dnm4
- absences::keyring_is_named_only_in_the_keychain_module_and_its_mock_only_under_test_support — verifies: LLR-677fqq, LLR-36jer8
- absences::no_lower_unit_depends_on_a_keychain_crate — verifies: LLR-677fqq
- absences::keychain_bytes_reach_no_clone_debug_or_serialise — verifies: LLR-677fqq

red -> green:
- the nine `tests/keychain.rs` tests — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `org_io::test_support::keychain`, `{MemoryKeychain, Refusal}`, `org_io::KeychainError`)
- keyring_is_named_only_in_the_keychain_module_and_its_mock_only_under_test_support — watched fail for the right reason before the implementation existed (runtime: no `keychain.rs`)
- keychain_bytes_reach_no_clone_debug_or_serialise — watched fail for the right reason before the implementation existed (runtime: `src/keychain.rs` not found)
- no_lower_unit_depends_on_a_keychain_crate — an absence check, green on its first run because no lower unit names a keychain crate; mutation-checked instead (with org-io added to the scanned units it failed: "org-io depends on the keychain crate `keyring`"; mutation reverted)

inherited: none. Adapted, annotations unchanged, dated 2026-10-13:
`absences::device_secret_is_named_only_in_its_sanctioned_places`
(LLR-3zdw8v) names `keychain.rs` for the deleted `device_secrets.rs`;
`open.rs`, `signatory_key.rs` (two tests) and `common/handles.rs` install
`MemoryKeychain::shared()` before creating a Persona.

Result: org-io line 1 (+ `--test keychain`) 79 passed; line 2 11 passed;
build and clippy clean; `check-ids.sh` exit 0; org-io `check-trace.sh`
MISSING-TEST 49 → 42, other advisories unchanged.

Notes:
- **Test safety guard** (beyond the task text, accepted by the dispatcher
  as a default): in a `test-support` build every keychain call is refused
  `Unavailable { reason: "NotInstalledForTest" }` until a `MemoryKeychain`
  is installed in the process, so no test can reach the real OS keychain;
  `open` and `signatory_key` gained `required-features = ["test-support"]`.
  Noted under LLR-8erfbu. The app's Persona-creating tests must install
  `MemoryKeychain` (T15/T17).
- `MemoryKeychain` implements `CredentialBuilderApi`/`CredentialApi` over
  shared items with a `refuse_as` switch (`Locked` → `NoStorageAccess`,
  `Unavailable` → `PlatformFailure`); its `Debug` prints names only.
  `reason` is a `&'static str` naming the keyring variant, or
  `"NoNativeBackend"`. The non-native path is a `cfg(not(any(…)))` module
  that never names keyring. Test facade `keychain::for_test`, re-exported
  as `test_support::keychain`; `OrgIo::device_secret_for_test` reads the
  keychain.
- `LocalStore.device_secrets` keeps its name (T4's call sites), now a
  zero-sized `keychain::Keychain` handle; signing reads the keychain in
  production builds too.
- Transitional: `create_persona` maps a keychain refusal to
  `OrgNodeError::Chain("keychain write failed: …")`; the keychain write
  precedes the store file, so a refusal writes no file. Dropping the new
  Persona from the service on refusal (LLR-n9dnm4) and `OrgIo::open`'s
  refusal text are T9's.
- `#![allow(clippy::result_large_err)]` in `keychain.rs`: every
  `KeychainError` variant carries a `DevicePublicKey` (LLR-8erfbu).
- **For T18:** add `--test keychain` to `ORG_IO_COVERAGE_ARGS`;
  `cargo clippy -p org-io --all-targets` with no features is red before T8
  (`same_bytes_as` in `key_custody_config.rs`).
- `.guardrails/scripts/guidelines-file.sh` is not in this repository
  (guardrails 0.5.1); tasks run without it, as T1–T7 did.

### T4 report, 2026-10-13

Tests added (org-node/tests/receive_path.rs):
- a_body_that_does_not_decode_is_malformed_and_changes_nothing — verifies: LLR-zetn7a
- an_unheld_revocation_is_refused_before_any_expectation_check — verifies: LLR-zetn7a, LLR-38e2kn
- an_unexpected_first_admission_is_refused — verifies: LLR-zetn7a, LLR-s8xp7m
- the_checked_message_names_the_organisation_to_read — verifies: LLR-zetn7a
- a_checked_message_has_no_public_constructor — verifies: LLR-zetn7a
- an_update_for_a_held_organisation_from_an_unlisted_sender_is_refused_in_the_check — verifies: LLR-2r2fha
- a_first_admission_is_accepted_from_any_sender — verifies: LLR-2r2fha
- a_revocation_from_an_unlisted_sender_is_refused_in_the_check — verifies: LLR-kzgjz8, LLR-r7zm39
- an_acknowledgement_is_decided_in_the_check_from_its_named_device_only — verifies: LLR-3aysup, LLR-5azhry
- a_first_admission_commits_as_admitted — verifies: LLR-6qjmhf
- an_update_commits_as_updated — verifies: LLR-6qjmhf
- the_nodes_own_removal_forgets_the_organisation — verifies: LLR-6qjmhf, LLR-b27jr6
- an_accepted_revocation_commits_as_revoked — verifies: LLR-6qjmhf, LLR-tx8ruv
- a_revocation_that_leaves_this_device_listed_is_refused — verifies: LLR-6qjmhf, LLR-pt32fx
- a_carried_key_that_does_not_match_the_chain_is_refused — verifies: LLR-6qjmhf, LLR-ba2ejp
- no_chain_state_is_org_not_on_chain — verifies: LLR-6qjmhf
- only_the_check_takes_the_sender — verifies: LLR-6qjmhf
- no_receive_branch_is_unreachable — verifies: LLR-zetn7a (renamed 2026-10-15, TEST conformance round 3, to `an_unheld_organisations_notice_is_refused_as_not_held`; its source-scan half was dropped in round 2)

red -> green (all 18 red at compile time first: E0432 unresolved
`org_node::service::{Checked, CheckedMessage, ReceiveKind}`; E0599 no
`check_received`/`commit_checked`; E0609 no field `kind` on
`ReceiveOutcome`):
- each of the 18 tests above — watched fail for the right reason before the implementation existed
- no_receive_branch_is_unreachable additionally stayed red at runtime after the first implementation ("service.rs constructs RevocationNotHeld", from T3's `notice_index` fallback) until the store choice moved to `revocation::notice_store`: 18/18

inherited: none. Call sites rewritten with `verifies:` lines unchanged,
except, with the retired LLR-379hnv: its annotation removed from
`receive_chain_reads.rs::the_self_delete_path_refuses_an_unheld_organisation_without_a_chain_read`
(now asserts `AdmissionNotExpected`; keeps LLR-38e2kn, LLR-jwhzh3), and
`admission_sender.rs::the_self_delete_path_refuses_an_organisation_it_holds_no_record_of`
(LLR-379hnv alone) deleted with a dated comment. Scans renamed to the new
names, annotations unchanged: `sealed_store.rs::saving_needs_no_device_secret`
(allowance `commit_checked`), org-io `absences.rs` (allowlist:
`SelfDeleteOutcome` out, `Received` in), `app_boundary.rs`.

Result: org-node verify line + `--test receive_path` 346 passed, 5 fuzz
targets clean, quint typecheck OK; org-node all-targets clippy only the
pre-existing hits. After merging on top of T8: org-io line 1 79 passed,
line 2 11 passed, clippy clean (run by the dispatcher). `check-ids.sh` exit
0; org-node `check-trace.sh` MISSING-TEST 9 (LLR-zetn7a, LLR-6qjmhf now
covered; LLR-379hnv newly listed; the rest T5/T6, REQ-uy7kfq/LLR-yexf5m
included).

Notes:
- `commit_genesis` returns `ReceiveOutcome` with `ReceiveKind::Admitted`;
  `ReceiveOutcome` drops S2's `acknowledged`; org-io `receive_and_verify`
  returns `Received::{Committed(ReceiveOutcome),
  Acknowledged(VerifiedAcknowledgement)}` (T11's name, pulled forward,
  allowlisted). LLR-6qjmhf's draft text on a still-listed notice is
  corrected by a clarification note (`RevocationProofRefused` from
  `accept`, S3a).
- Test support: `Inbox`/`Relay` replace endpoint addresses; `Deferred`
  (`IntoFuture`) keeps `task.await`; `Node::sending_as`; `body_to` (T5
  replaces it); the dial tests use `admit_over_iroh`, `revoke_over_iroh`,
  `spawn_recv_one_over_iroh` until T10. The `deliver_*_to_receive` helpers
  are `async fn` with no await (for the deslop pass).
- Three blocks of `org-node/src/service.rs` were spliced with sed against
  the plan's convention; build and suite green. The deslop pass reviews
  `service.rs`.
- The app still calls `receive_and_self_delete_if_revoked` (T15/T16).
- **LLR-379hnv:** the definition is kept with a dated "Retired" note, and
  is now MISSING-TEST. Removing it would turn its 15 history mentions into
  DANGLING-REF. Put to the owner on 2026-10-13, together with LLR-g9vmbx
  (T6) and LLR-q225ws (T12). *Ruled 2026-10-13:* delete the definition and
  its `assesses:` entries, and rewrite every mention as prose that does not
  name the ID (as for moved items, ADR `docs/adr/2026-10-06-org-io-unit.md`).
  The S4 verification record lists the three retirements by ID. LLR-379hnv
  is done in T6, with LLR-g9vmbx; LLR-q225ws in T12.

### T5 report, 2026-10-13

Tests added:
- outgoing_bodies::an_admission_sends_information_to_every_listed_device_but_the_senders — verifies: LLR-gdfqn9
- outgoing_bodies::a_removal_sends_revocations_to_every_removed_device_after_the_listed_ones — verifies: LLR-gdfqn9, LLR-a8z7r5
- outgoing_bodies::a_commit_that_removes_this_node_returns_its_acknowledgements_and_no_bodies — verifies: LLR-gdfqn9, LLR-b27jr6
- outgoing_bodies::commit_genesis_returns_no_recipients — verifies: LLR-gdfqn9
- outgoing_bodies::a_reconciled_commit_returns_the_same_bodies_as_commit_update — verifies: LLR-gdfqn9
- outgoing_bodies::a_later_commit_cannot_change_an_earlier_body — verifies: LLR-gdfqn9, PR-gnh3j2
- outgoing_bodies::outgoing_debug_shows_no_body_byte — verifies: LLR-gdfqn9
- outgoing_bodies::outgoing_types_are_not_clone — verifies: LLR-gdfqn9
- outgoing_bodies::building_the_bodies_writes_and_sends_nothing — verifies: LLR-gdfqn9
- org-io absences::no_handle_method_returns_a_type_holding_an_outgoing_body — verifies: LLR-3zdw8v, LLR-gdfqn9

red -> green:
- the nine `outgoing_bodies` tests — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `OutgoingMessage`; E0599/E0608 `outgoing` an `OutgoingUpdate`; E0609 no field `sender`)
- a_later_commit_cannot_change_an_earlier_body — also watched fail at runtime before the implementation existed (a temporary copy against the send-time rule: "the first commit's key"; the copy was deleted)
- no_handle_method_returns_a_type_holding_an_outgoing_body — watched fail for the right reason before the implementation existed (runtime: "org-io/src/lib.rs:396: returns CommitOutcome: pub async fn submit_commit_send")

inherited: none. Rewritten at call sites with annotations unchanged and
dated comments: `commit_paths` (the three `send_update_*` tests and five
call sites), `service_stories::send_update_chooses_by_record_and_outcome`,
`reconcile` (two), `receive_path::the_nodes_own_removal_forgets_the_organisation`,
`admission_sender::a_loopback_revocation_without_an_address_burns_no_epoch`,
`commit_workflow_errors` (variant list), two tests that now tell C's Device
(a commit builds no body for its sender's Device); org-io
`absences::the_handle_returns_only_allowlisted_public_types` (`CommitOutcome`
out, `Submitted` in).

Result: org-node verify line + `--test outgoing_bodies` 355 passed, fuzz
clean; org-io line 1 80 passed, line 2 11 passed, build and clippy clean;
`check-ids.sh` exit 0; org-node MISSING-TEST 9 → 7 (T6's items and
LLR-379hnv); org-io 42. PR-gnh3j2 resolved in its ledger.

Notes:
- `OutgoingMessage { recipient, body }` (not Clone; `Debug` shows
  `body_len`); `CommitOutcome { org_id, epoch, root, sender, outgoing,
  acknowledgements }`; a private `outgoing_bodies()` shared by
  `commit_update` and `reconcile`; `OrgEndpoint::send_body` (until T10);
  org-io `Submitted { org_id, epoch, root }`.
- `send_body` does not check that the bound endpoint is the sender
  Persona's Device: the check reddened the green pin of PR-8qsnhx, which
  T10 resolves.
- Test support: `body_for`, `message_for`, `notices_in`,
  `device_by_handle`, `send_over_iroh`.
- Two blocks of `service.rs` were applied with a scripted string replace,
  against the convention, as in T4. The deslop pass reviews them.

### T6 report, 2026-10-13

Tests added:
- invitation::an_invite_and_a_reply_that_parse_encode_back_to_the_same_blob — verifies: LLR-b7wgpf (moved)
- invitation::an_invite_or_reply_that_does_not_parse_is_refused_naming_the_field — verifies: LLR-b7wgpf (moved)
- invitation::a_reply_that_does_not_parse_acts_on_nothing — verifies: LLR-b7wgpf (moved, rewritten for org-node)
- invitation::build_invite_records_the_pair_and_carries_the_bound_personas_device_key — verifies: LLR-nr8df3
- invitation::build_invite_refuses_an_organisation_no_persona_is_bound_to — verifies: LLR-nr8df3
- invitation::build_reply_records_the_expectation_once_and_carries_the_personas_keys — verifies: LLR-7q9g9z
- invitation::build_reply_refuses_a_bound_persona — verifies: LLR-7q9g9z
- invitation::build_reply_refuses_an_unknown_persona — verifies: LLR-7q9g9z
- invitation::check_reply_refuses_another_organisation_and_an_id_not_outstanding — verifies: LLR-d55brn
- invitation::settle_removes_exactly_the_pair_and_refuses_one_not_held — verifies: LLR-d55brn
- invitation::forgetting_an_organisation_drops_its_outstanding_invites — verifies: LLR-pba7yu
- invitation::build_reply_refuses_an_organisation_another_persona_holds — verifies: LLR-yexf5m
- invitation::build_reply_refuses_an_organisation_another_persona_expects — verifies: LLR-yexf5m
- invitation::the_same_persona_replying_twice_is_not_refused_by_the_one_membership_rule — verifies: LLR-yexf5m
- invitation::expect_admission_refuses_the_same_two_cases — verifies: LLR-yexf5m
- invitation::the_refusal_names_the_organisation_and_no_persona — verifies: LLR-yexf5m
- fuzz_invitation_parse (bolero, harness = false) — verifies: LLR-b7wgpf
- encoding_golden::persona_store_plaintext_is_pinned re-pinned (LLR-ayrdr8, annotation unchanged): `01 ‖ aa×20 ‖ ee×32` appended by the format rule; dated comment, previous value kept

red -> green:
- the 16 `invitation` tests — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `org_node::invitation`; E0599 no `build_invite`/`build_reply`/`check_reply`/`settle`; no variants `OrganisationHeldByAnotherPersona`, `InviteNotOutstanding`, `ReplyForAnotherOrganisation`; E0609 no field `outstanding_invites`)
- fuzz_invitation_parse — watched fail for the right reason before the implementation existed (compile: E0433 cannot find `invitation` in `org_node`)
- persona_store_plaintext_is_pinned (re-pin) — watched fail for the right reason before the implementation existed (runtime: round-trip `left == right` at encoding_golden.rs:158; truncated_pinned_values_are_refused also red); green once `outstanding_invites` landed
- absences::org_node_holds_no_invitation_exchange and ::org_node_holds_no_invite_identifier went red at runtime when the implementation landed, as step 5 expects, then were narrowed/inverted per step 5

inherited: the three LLR-b7wgpf tests — from app/src-tauri/tests/invitation.rs
(same ID, moved under ruling A), with a dated module note: the field is read
from `OrgNodeError::InvalidField`, and "acts on nothing" is shown on
org-node's side (no reply, store bytes and outstanding pair unchanged).

Result: org-node verify line + `invitation` + `fuzz_invitation_parse` 371
passed, six bolero targets clean (Quint not re-run: no model input changed);
org-io line 1 80 passed, line 2 11 passed, build and clippy clean; org-node
all-targets clippy no new hits; check-trace org-node exit 0 — no
MISSING-TEST, no DANGLING-REF, no UNANALYZED-DERIVED (UNMET-EXPECTATION 1,
UNRESOLVED-PR 9); app exit 0; org-io MISSING-TEST 42; check-ids
`--allow-draft-files` exit 0 for org-node, org-io, app; check-units exit 0.

Notes:
- Retirement grep: the units are clean; repo-level plans and verification
  records keep the IDs (step 6 clarified the same day, above).
- `Invite::parse`/`InviteReply::parse` return `OrgNodeError::InvalidField
  { field, reason }` (LLR-8bum44) instead of the app's `String`; field names
  unchanged; Display reads "invalid <field>: …". Recorded as a hybrid-rule
  clarification on LLR-b7wgpf. Encode failures are
  `OrgNodeError::Chain("blob encode: …")` — the deslop pass looks at the
  variant choice.
- App edits beyond Files touched: invitation.rs and commands.rs re-export
  the moved types from `org_io::node::invitation` (app still red until
  T15); app mentions of REQ-yazum3/LLR-b7wgpf rewritten as prose; app SOUP
  `base64` row notes the app no longer calls it (row goes in T15).
- `build_reply` order: unknown Persona, `PersonaAlreadyBound`,
  `OrganisationHeldByAnotherPersona` (noted under LLR-yexf5m).
- analyze-risks on REQ-uy7kfq: no new hazard, no rating change; narrows
  HAZ-vxabf9, touches HAZ-tawvm2 only as a delay (RC-6a2dke). Recorded with
  `assesses: REQ-uy7kfq` in org-node/docs/risk/2026-10-06-chain-authority.md.
- Left for later: org-io NON-EXPORTED-REF LLR-gdfqn9 (an org-io absences
  test from T5) and 4 all-targets clippy hits in outgoing_bodies.rs (T5) —
  both T18; `expect_admission` kept, T13 decides.

### T9 report, 2026-10-13

Tests added:
- store_files::each_persona_has_its_own_file_named_for_its_device_key — verifies: LLR-nt8dja
- store_files::the_listing_ignores_every_other_name — verifies: LLR-nt8dja
- store_files::the_file_holds_exactly_the_sealed_bytes — verifies: LLR-b9bm36
- store_files::a_write_interrupted_after_the_temporary_leaves_the_old_store_unsealing — verifies: LLR-b9bm36
- store_files::a_failed_write_reports_failure_and_reloads_the_persona_from_its_file — verifies: LLR-murjr6
- store_files::a_commit_whose_store_write_fails_says_the_commit_was_made_and_reloads — verifies: LLR-murjr6 (the reproduction of PR-jav8zn, named in prose)
- store_files::a_dropped_persona_keeps_its_file_and_item_and_opens_again — verifies: LLR-murjr6
- store_files::opening_creates_the_data_dir_and_unseals_every_store — verifies: LLR-uzax55, LLR-85zque
- store_files::a_store_whose_item_is_missing_is_refused_and_left_untouched — verifies: LLR-uzax55, LLR-85zque
- store_files::a_locked_keychain_refuses_opening_naming_the_persona — verifies: LLR-uzax55, LLR-85zque
- store_files::the_store_is_opened_inside_the_data_dir_it_creates — verifies: LLR-85zque
- store_files::a_data_dir_that_cannot_be_created_is_refused_naming_it — verifies: LLR-85zque
- store_files::create_persona_writes_the_item_before_the_file_and_nothing_when_the_keychain_refuses — verifies: LLR-n9dnm4
- store_files::an_ordinary_save_succeeds_while_the_keychain_is_locked — verifies: REQ-6d9xv6 (as narrowed 2026-10-07; the plan's LLR-bvt5up is org-node's and cannot be cited from org-io)
- store_files::a_forgotten_store_is_removed_and_listed_no_more — verifies: LLR-amv4mb
- store_files::a_failed_removal_is_reported_and_the_persona_reloaded — verifies: LLR-amv4mb
- org-io absences::no_code_names_the_old_files — verifies: LLR-t4v69b
- org-node sealed_store::restore_store_replaces_or_adds_one_store_and_drop_store_removes_one — verifies: LLR-rdda86
- org-node absences::only_the_removal_step_adopt_and_drop_store_remove_records_or_stores — verifies: LLR-jd82t6

red -> green:
- the first 15 `store_files` tests — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `org_io::store_file`, `org_io::{CreateError, OpenError, OperationError, StoreFailure}`; E0425 no `test_support::fail_next_write_after_temporary`; E0599 no `MemoryKeychain::calls`, no `OrgIo::create_persona`); mutation-checked too: without `restore_store` in the reload the three reload tests fail
- restore_store_replaces_or_adds_one_store_and_drop_store_removes_one — watched fail for the right reason before the implementation existed (compile: E0599 no `restore_store`/`drop_store` on `OrgService`)
- only_the_removal_step_adopt_and_drop_store_remove_records_or_stores — watched fail for the right reason before the implementation existed (runtime, against a temporary `self.stores.retain(|_| true)` in `restore_store`: "service.rs calls `stores.retain(` (LLR-jd82t6)")
- a_dropped_persona_keeps_its_file_and_item_and_opens_again — NOT red first: written after `drop_store` existed (step 11 added it then). Mutation-checked instead: with the `drop_store` call removed from org-io's reload-failure branch it fails ("dropped from the running service")
- no_code_names_the_old_files — NOT red first (an absence check, green on its first run). Mutation-checked: with `persona_store.bin` in a `store_file.rs` comment it fails

inherited:
- the_store_is_opened_inside_the_data_dir_it_creates, a_data_dir_that_cannot_be_created_is_refused_naming_it — from LLR-85zque in app/src-tauri/tests/state_assembly.rs (same ID, moved under ruling A, rewritten per Persona); deleted from the app with a dated comment
- org-node absences::only_forget_organisation_removes_records_and_personas — from LLR-23sfdh (retired; red after `drop_store`), deleted and replaced by the LLR-jd82t6 scan

Result: org-io line 1 (with `store_files`) 97 passed, line 2 11 passed,
build and clippy (all targets, test-support,dev-seed) clean; org-node verify
line 372 passed, lib clippy clean (Quint not re-run: no model input
changed); check-ids `--allow-draft-files` exit 0 for org-node, org-io and
app; check-trace org-node exit 0, no MISSING-TEST, no DANGLING-REF,
UNRESOLVED-PR 9 → 7; org-io MISSING-TEST 42 → 32, NON-EXPORTED-REF 1
(LLR-gdfqn9, T5's, T18); app exit 1 on NON-EXPORTED-REF LLR-85zque only
(T17, step 4 note).

Notes:
- Owner ruling 2026-10-13 (option A): LLR-jd82t6 supersedes LLR-23sfdh,
  exempting `adopt` and `drop_store`; LLR-23sfdh retired by the T6 rule.
  No `supersedes:` line (the old definition is gone, so a reciprocal
  `superseded-by:` cannot exist); the supersession is a dated prose note on
  LLR-jd82t6 and a "Retired" note in place. T18 lists LLR-23sfdh in the
  verification record's retirements, with LLR-379hnv, LLR-g9vmbx and
  LLR-q225ws.
- Two green org-node tests verified only the retired item
  (`absences::forget_organisation_is_defined_once_and_called_only_from_accept_and_the_removal_step`,
  `service_stories::a_commit_that_removes_another_member_forgets_nothing`):
  kept, running, with no `verifies:` line and a dated prose pointer to
  LLR-jd82t6 (re-annotating a green test is forbidden). Three green tests
  that also verify other IDs lost the retired ID with a dated note.
- RC-ub82my assessed (dated note in org-node/docs/risk/2026-10-07-commit-workflow.md):
  no new hazard, no rating change. LLR-85zque assessed in org-io's S4 risk
  draft; the app block removed.
- New public types, all on the absences allowlist with a dated comment:
  `OpenError`, `CreateError`, `OperationError` (org-node refusal or store
  failure; the handle's builders and `receive_and_verify` return it),
  `StoreFailure`, `StoreFileError { step, path, reason }`. The submit paths
  still return `String`.
- `rand_core` gains feature `getrandom`. Reasoned allows:
  `clippy::result_large_err` on `open_store`, `OrgIo::open`,
  `OrgIo::create_persona`; `await_holding_lock` in store_files.rs (`LOCK`
  serialises tests over keyring's process-global builder).
- Plan error fixed in the task: the plan had org-io tests cite org-node
  items (LLR-bvt5up, LLR-jd82t6, PR-jav8zn); re-pointed before merge.
- For T18: add `--test store_files` to `ORG_IO_COVERAGE_ARGS`.

### T10 report, 2026-10-13

Tests added:
- endpoints::the_bound_endpoint_id_is_the_device_key — verifies: LLR-d5y7ks
- endpoints::binding_twice_for_one_persona_yields_the_same_id — verifies: LLR-d5y7ks
- endpoints::a_keychain_item_for_another_key_is_refused_and_the_endpoint_closed — verifies: LLR-d5y7ks
- endpoints::a_locked_keychain_refuses_the_bind — verifies: LLR-d5y7ks, REQ-6d9xv6
- endpoints::a_second_organisations_message_goes_out_under_its_own_personas_key — verifies: LLR-d2pzzk, PR-8qsnhx
- endpoints::each_persona_has_one_endpoint_bound_lazily — verifies: LLR-d2pzzk (beyond the plan's list)
- endpoints::a_message_to_either_personas_device_reaches_the_receive_channel — verifies: LLR-z35vbh
- preflight_transport::the_transport_check_passes_for_a_device_key_whose_item_exists — verifies: LLR-h9tcat
- preflight_transport::a_missing_or_locked_item_fails_the_check_naming_the_key_and_binds_nothing — verifies: LLR-h9tcat
- preflight_transport::the_preflight_checks_every_persona_listed_in_the_data_dir — verifies: LLR-h9tcat
- preflight_transport::a_data_dir_with_no_store_fails_the_check_naming_the_dir_and_binds_nothing — verifies: LLR-h9tcat
- absences::iroh_secret_key_is_named_in_one_function_only — verifies: LLR-d5y7ks
- absences::the_preflight_reads_no_key_or_seed_from_the_environment — verifies: LLR-h9tcat, LLR-d5y7ks

red -> green:
- the 7 `endpoints` tests and 4 `preflight_transport` tests — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `org_io::transport`, `org_io::preflight::{check_transport, check_transport_for_data_dir}`; E0599 no `endpoint_for_test`, `bound_endpoints_for_test`, `start_receiving`, `next_inbound_for_test` on `OrgIo`)
- iroh_secret_key_is_named_in_one_function_only — watched fail for the right reason before the implementation existed (runtime: "the scan found bind_from_keychain's use of SecretKey")
- the_preflight_reads_no_key_or_seed_from_the_environment — watched fail for the right reason before the implementation existed (runtime: "preflight.rs runs the transport check")

inherited (moved under ruling A; setups rewritten onto
`OrgEndpoint::bind_from_keychain` over `MemoryKeychain`, dated comments):
- transport_handshake::delivers_and_verifies_admit_over_iroh — from org-node transport_handshake.rs (LLR-v873fx)
- transport_handshake::loopback_mode_binds_and_advertises_loopback_only — same (REQ-db6s7q, LLR-wwunf4, LLR-wx77j5)
- transport_handshake::loopback_mode_holds_under_repeated_and_colliding_binds — same (REQ-db6s7q, REQ-2wzfzv, LLR-wwunf4, LLR-6adc99)
- transport_handshake::endpoint_id_equals_device_key — same (LLR-ygn78w)
- transport_handshake::a_stream_longer_than_the_read_bound_is_refused_rather_than_buffered — same (LLR-k2y6nn; org-node's REQ-eg5j8u dropped, named in prose)
- transport_handshake::the_length_prefix_is_not_checked_against_the_body — same (LLR-2smrvx)
- transport_handshake::frame_round_trips — from org-node wire_frame_bound.rs (LLR-fa7jt8; org-node's REQ-eg5j8u and LLR-er2x8n dropped, named in prose)
- transport_networked::delivers_and_verifies_admit_over_relay_by_id — from org-node transport_networked.rs (LLR-v873fx)
- endpoints::the_endpoint_is_bound_once_and_the_same_one_is_returned_after — from org-node service_lifecycle.rs (LLR-6zjzn2)
- endpoints::the_endpoint_binds_the_named_personas_device_not_the_first_personas — same (LLR-cns6q6)
- endpoints::a_service_in_loopback_mode_binds_loopback_sockets_only — same (REQ-db6s7q, LLR-ecz9a6)

Result: org-io line 1 (four new targets) 121 passed, fuzz clean; line 2 11
passed; build and clippy (all targets, test-support,dev-seed) clean;
org-node verify line 361 passed (372 − 11 moved), six bolero targets clean,
lib clippy clean, all-targets clippy only the pre-existing hits (Quint not
re-run); check-ids exit 0 for all three; check-trace org-node exit 0
(UNRESOLVED-PR 7 → 6), org-io MISSING-TEST 32 → 24 with NON-EXPORTED-REF
LLR-gdfqn9 only, app unchanged (LLR-85zque, T17); check-units clean.

Notes (dispatcher's rulings on the report's points, 2026-10-13):
- 16 transport items moved from org-node to org-io. Their `satisfies:`
  /`traces:` were re-parented onto org-io REQs with dated notes (SDD-8uyg4s,
  SDD-b8tuv3, LLR-ygn78w, LLR-v873fx, LLR-fa7jt8, LLR-k2y6nn, LLR-2smrvx —
  no longer derived —, LLR-6zjzn2, LLR-cns6q6). Every org-node mention is
  prose now.
- LLR-6zjzn2 and LLR-cns6q6 keep their IDs with dated "Amended (S4, T10)"
  notes for the per-Persona and keychain rules. Accepted: this is ruling A's
  "moved, rewritten per Persona" (§3.1), the same route LLR-85zque took in
  T9, not a new change of meaning under the hybrid rule.
- org-node's green `service_stories::five_stories_full_e2e` lost the two
  moved IDs (LLR-6zjzn2, LLR-cns6q6) from its `verifies:` line, with a dated
  note. Accepted: a cross-unit citation cannot stay, and this follows the
  retirement precedent (T4, T9); the test still verifies its other IDs.
- PR-8qsnhx resolved in org-io's ledger; PR-6f3qku's `affects:` lost
  LLR-k2y6nn and LLR-2smrvx (dated note).
- Design choices (dated clarifications in org-io's design draft): boxed
  `TransportError::Keychain` / `KeychainKeyMismatch`; a key mismatch is
  refused before the builder runs; new `EndpointClosed`, `NoLoopbackAddress`;
  `Endpoints` in `LocalStore`; receive channel bounded at 64; accept tasks
  aborted on drop; until T11 `receive_and_verify` reads the channel and
  refuses when nothing accepts; `OrgIo::open` takes
  `org_io::transport::TransportMode`; `TransportError` on the allowlist.
- Preflight binary requires `ODS_DATA_DIR`, runs transport rows before the
  chain checks, names each Device key in hex.
- `OrgEndpoint` stays public; `bind_from_keychain` is its only constructor;
  `inner()` is test-only.
- org-node keeps its `iroh` dev-dependency until T14 (dead
  `bind_with_relay`).
- For T18: add `--test endpoints --test preflight_transport --test
  transport_handshake --test transport_networked` to `ORG_IO_COVERAGE_ARGS`
  as the Makefile's coverage lines need, and drop the two removed org-node
  targets from `ORG_NODE_COVERAGE_ARGS`.

### T11 report, 2026-10-13

Tests added (org-io/tests/receive_step.rs):
- a_command_completes_while_the_receiver_waits — verifies: LLR-9arvqj, PR-7v8anh
- a_declared_first_admission_is_committed_by_the_one_path — verifies: LLR-cvyyr8, PR-kwwap5
- a_refused_check_reads_no_chain — verifies: LLR-cvyyr8
- an_accepted_body_reads_the_chain_once — verifies: LLR-cvyyr8
- the_sender_org_node_sees_is_the_handshakes — verifies: LLR-cvyyr8
- an_acknowledgement_is_settled_with_no_chain_read_and_no_store_write — verifies: LLR-cvyyr8
- an_accepted_revocation_signs_with_the_keychain_secret_and_removes_the_store — verifies: LLR-cvyyr8
- a_locked_keychain_refuses_the_revocation_and_changes_nothing — verifies: LLR-cvyyr8
- the_store_write_lands_before_the_outcome_returns — verifies: LLR-cvyyr8
- a_closed_endpoint_is_terminal_endpoint_closed — verifies: LLR-szhe7j
- a_failed_bind_is_terminal_bind_failed — verifies: LLR-szhe7j
- a_malformed_body_is_not_terminal — verifies: LLR-szhe7j

red -> green (all twelve red at compile before the implementation existed:
E0432 unresolved `org_io::ReceiveError`, `org_io::TerminalCause`; E0599 no
`next_received`, `organisations`, `personas`, `persona_public_keys`,
`admit_member`, `revoke_member`, `expect_admission` on `OrgIo`; E0277
`create_persona` is not a future):
- a_command_completes_while_the_receiver_waits — watched fail for the right reason before the implementation existed; mutation-checked too (node lock taken before the wait: fails at the 2 s timeout)
- a_declared_first_admission_is_committed_by_the_one_path — watched fail for the right reason before the implementation existed
- a_refused_check_reads_no_chain — watched fail for the right reason before the implementation existed
- an_accepted_body_reads_the_chain_once — watched fail for the right reason before the implementation existed
- the_sender_org_node_sees_is_the_handshakes — watched fail for the right reason before the implementation existed
- an_acknowledgement_is_settled_with_no_chain_read_and_no_store_write — watched fail for the right reason before the implementation existed
- an_accepted_revocation_signs_with_the_keychain_secret_and_removes_the_store — watched fail for the right reason before the implementation existed
- a_locked_keychain_refuses_the_revocation_and_changes_nothing — watched fail for the right reason before the implementation existed
- the_store_write_lands_before_the_outcome_returns — watched fail for the right reason before the implementation existed
- a_closed_endpoint_is_terminal_endpoint_closed — watched fail for the right reason before the implementation existed
- a_failed_bind_is_terminal_bind_failed — watched fail for the right reason before the implementation existed
- a_malformed_body_is_not_terminal — watched fail for the right reason before the implementation existed

inherited: none. Green tests rewritten onto the new API with `verifies:`
unchanged: receive_order (12), store_files, submit_flow, endpoints, open
(one test now `#[tokio::test]`), preflight_transport, signatory_key;
absences scans widened (below).

Result: org-io line 1 (with `receive_step`) 133 passed, fuzz clean; line 2
11 passed; build and clippy clean; check-ids exit 0 for org-io, app,
org-node; check-trace org-io MISSING-TEST 24 → 19, NON-EXPORTED-REF
LLR-gdfqn9 only; app unchanged (LLR-85zque, T17); org-node untouched.

Notes:
- `OrgIo { node: tokio::sync::Mutex<OrgService> }`; every method `&self`;
  `NodeBuilders`, `NodeView`, `receive_and_verify`, `view()`, `node_mut()`
  gone; S2's views and builders are async methods of `OrgIo`;
  `node_for_test()` returns the guard (`try_lock`, test builds only).
- `receiver.rs`: `start_receiving`, `next_received(rng)`, `Received`,
  `ReceiveError { Terminal, Refused, Chain, Store, Keychain (boxed),
  Connection }`, `TerminalCause { EndpointClosed, BindFailed(TransportError) }`.
  `Connection` (one inbound connection failed before a body; not terminal)
  is beyond the plan's list; `next_received` takes `rng`. Both are "as
  landed" notes on LLR-szhe7j / LLR-cvyyr8.
- The submission (`found_organisation`, `submit_commit_send`) still holds
  the node lock for its whole sequence, chain write (up to 90 s) and send
  included. LLR-9arvqj covers the receive step only, so the plan is met;
  noted on LLR-9arvqj. A possible later narrowing, offered to the owner.
- `Inbound` carries no verdict responder yet: T11a's.
- absences.rs: `handle_methods` scans every `impl OrgIo` in every file;
  `NodeView`, `NodeBuilders` left the allowlist, `ReceiveError`,
  `TerminalCause`, `VerifiedAcknowledgement` joined it.
- Cross-unit fixes: PR-kwwap5's `affects:` (moved to org-io) names
  REQ-fnsjb6, LLR-cvyyr8, REQ-nfr3n2, the app's REQ in prose; PR-7v8anh
  gained `affects: REQ-qkrj55, LLR-9arvqj`. Dated notes.
- One unattributed failure of verify line 1, not reproduced in 15 reruns;
  likely the iroh flake seen before. Watch at T18.
- For T18: add `--test receive_step` to `ORG_IO_COVERAGE_ARGS`.

### T11a report, 2026-10-13

Tests added (org-io/tests/acknowledgement_inbound.rs):
- a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted — verifies: LLR-wvgec4, LLR-unqfj5
- a_revoked_device_sending_anything_but_an_acknowledgement_is_refused_and_served_nothing — verifies: LLR-wvgec4, LLR-unqfj5
- an_acknowledgement_a_member_still_listing_the_device_refuses_is_answered_refused — verifies: LLR-wvgec4, LLR-unqfj5
- an_acknowledgement_relayed_by_another_device_is_refused — verifies: LLR-wvgec4
- the_accept_path_has_no_allow_list — verifies: LLR-wvgec4
- nothing_but_the_verdict_is_written_on_an_inbound_stream — verifies: LLR-unqfj5
- a_dropped_responder_writes_nothing — verifies: LLR-unqfj5 (beyond the plan's list)

red -> green:
- the first six tests — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `org_io::test_support::dial_direct_as`, `org_io::transport::Verdict`); mutation-checked after green too (always Accepted reddens the three refusal tests; dropping the reply reddens the four network tests; a stray `write_all` or `is_listed` check reddens both scans)
- a_dropped_responder_writes_nothing — NOT red first (written after the implementation). Mutation-checked: a dropped responder answering Refused reddens it ("no byte written, read [1]")

inherited: none. No green test re-annotated.

Result: org-io line 1 (with `acknowledgement_inbound`) 140 passed on two
full runs; line 2 11 passed; build and clippy clean; check-ids exit 0;
check-trace org-io MISSING-TEST 19 → 16, NON-EXPORTED-REF LLR-gdfqn9 only.
org-node untouched.

Notes:
- The still-listed case refuses with `AcknowledgementFromFuture`, not the
  plan's `AcknowledgementForListedDevice`: org-node checks the epoch before
  the listing, and a member still listing the Device has not reached the
  revocation epoch. The test asserts C still lists the Device, the variant,
  and the Refused verdict; dated clarification under REQ-ur7z4a (verdict
  unchanged). Accepted by the dispatcher as a clarification.
- The non-acknowledgement case uses a forged notice naming Alice (refused
  `SenderNotListed`); forwarding A's own notice is refused earlier as
  `RevocationNotForThisDevice`. Dated note on LLR-wvgec4.
- `dial_direct_as(device_key: &DevicePublicKey, addr, body) ->
  Result<Vec<u8>, TransportError>`: binds a relay-less Loopback endpoint
  from the keychain item (no seed in hand), dials direct addresses, returns
  every byte read.
- As landed (dated note on LLR-unqfj5): `transport::Verdict { Accepted =
  0x00, Refused = 0x01 }`; `OrgEndpoint::recv_answerable` returns the body
  and an `InboundReply` (writes the byte, finishes, holds the connection up
  to 10 s); `recv_body` unchanged; a oneshot responder per body, held by a
  spawned task; `next_received` answers Accepted on a message and Refused on
  every error, a failed store write included.
- Cross-unit: org-node's LLR-3aysup, LLR-2r2fha, LLR-kzgjz8 (plan's IDs
  verified "over the real transport") are asserted by variant name, not
  cited.
- For T18: add `--test acknowledgement_inbound` to `ORG_IO_COVERAGE_ARGS`.

### T12 report, 2026-10-14

Tests added:
- send_fanout::networked_every_recipient_is_attempted_and_reported_in_order — verifies: LLR-znhmh2, PR-3ue4va
- send_fanout::a_failed_send_leaves_the_commit_in_place — verifies: LLR-znhmh2
- send_fanout::loopback_without_an_address_is_refused_and_sends_nothing — verifies: LLR-r9sh6k, LLR-pw369n, PR-2dmjzj
- send_fanout::loopback_to_an_address_of_no_recipient_is_refused_and_sends_nothing — verifies: LLR-r9sh6k, PR-2dmjzj
- send_fanout::loopback_sends_to_the_matching_recipient_only_and_reports_the_rest_not_attempted — verifies: LLR-r9sh6k, LLR-jn5jeh, PR-2dmjzj
- peer_addr::blank_text_is_no_address — verifies: LLR-2b2ghe
- peer_addr::bare_and_zero_x_prefixed_hex_of_an_endpoint_addr_is_that_address — verifies: LLR-2b2ghe
- peer_addr::text_that_is_not_hex_is_refused_naming_the_field — verifies: LLR-2b2ghe
- peer_addr::hex_of_bytes_that_are_not_exactly_an_endpoint_addr_is_refused_naming_the_field — verifies: LLR-2b2ghe
- fuzz_peer_addr (bolero) — verifies: LLR-2b2ghe

red -> green:
- the 5 `send_fanout` tests — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `org_io::send`; E0061 `submit_commit_send` takes 4 arguments; E0599 no `with_bind_plan_for_test`; E0609 no field `sends` on `Submitted`); mutation-checked too (dropping the no-recipient check, stopping at the first failed send)
- the 4 new `peer_addr` tests — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `org_io::peer_addr`)
- fuzz_peer_addr — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `org_io::peer_addr`)

inherited (moved under ruling A, rewritten onto `parse_peer_addr` or `OrgIo`):
- peer_addr::revoke_member_treats_a_whitespace_only_peer_addr_as_absent — from LLR-ty85xv (app ipc.rs)
- peer_addr::revoke_member_does_not_refuse_an_encoded_endpoint_addr, ::revoke_member_refuses_a_peer_addr_that_is_not_hex, ::revoke_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr — from LLR-n6twt7 (app ipc.rs)
- peer_addr::admit_member_does_not_refuse_a_blank_or_decodable_peer_addr, ::admit_member_refuses_a_peer_addr_that_is_not_hex, ::admit_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr — from LLR-ctrfz4 (app ipc.rs)
- submit_flow::each_command_sends_once_to_one_device_and_the_kind_follows_the_record, ::a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing — from LLR-q225ws (retired; app submit_flow.rs deleted). They run with no `verifies:` line and a dated prose pointer to LLR-znhmh2 (T9 precedent), not carrying LLR-znhmh2 as step 1 said.

Result: org-io line 1 (with `send_fanout`, `peer_addr`, `fuzz_peer_addr`)
158 passed, both fuzz targets clean; line 2 passed; build and clippy
clean; org-node verify line 361 passed, lib clippy clean (Quint not
re-run); check-ids exit 0 for all three; check-trace org-node clean
(UNRESOLVED-PR 6 → 5); org-io MISSING-TEST 16 → 3 (T13's), NON-EXPORTED-REF
LLR-gdfqn9 only; app NON-EXPORTED-REF LLR-85zque only. Retirement grep:
LLR-q225ws found only in docs/plans and docs/verification.

Notes:
- Moved to org-io: LLR-jn5jeh, LLR-pw369n (now `satisfies: REQ-8trkps`),
  LLR-ty85xv, LLR-n6twt7, LLR-ctrfz4 (now `satisfies: REQ-n82q47`),
  PR-2dmjzj, PR-3ue4va (resolved); derived assessments moved to org-io's
  risk draft. LLR-jn5jeh narrowed: its "nothing checks the address" clause
  withdrawn (LLR-r9sh6k checks it).
- Green tests that lost a moved or retired ID, dated notes (T9/T10
  precedent): org-node commit_paths ×2, admission_sender ×1; app ipc ×2,
  invitation ×1. `admission_sender::in_loopback_mode_the_joiner_is_dialled_at_the_full_address`
  verified only LLR-jn5jeh: unannotated with a prose pointer; T14 deletes it.
- As landed: `send_committed` crate-private, takes the sender's Device;
  `SendError` adds `Transport`, `SenderEndpoint`; a whole-send refusal is
  reported against every recipient; the verdict byte is not read (optional
  in the plan); `TransportError::NoLoopbackAddress` and
  `Endpoints::send_body` removed; `Submitted` holds the `SendReport` (no
  longer Copy/PartialEq); `OrgIo::with_bind_plan_for_test`,
  `PeerAddr::for_test` test-only; `SendReport`, `SendError`, `PeerAddr` on
  the allowlist; `postcard` a direct dependency (SOUP row added).
- Parse changes from the app's version: a bare `0x` and trailing bytes
  after an address are refused; only one leading `0x` is stripped (fixes
  the peer-address site of the app's open PR-5mc4d8 once T15 wires the
  commands; PR-5mc4d8 not edited — T15 records it).
- Loopback sinks in green org-io tests now bind as the recipient's Device;
  two captures read the recipient's inbound channel (dated comments).
- For T15: app REQ-m8sgjk still says the app chooses "which Device
  receives" the update.
- For T18: add `--test send_fanout --test peer_addr` to
  `ORG_IO_COVERAGE_ARGS`.
- **Merge blocker to watch:** on 2026-10-14 the app's check-trace shows five
  problem reports at 30 days (`problem_age_days: 30`). Not S4's; they must
  be resolved, re-dated by the owner, or otherwise handled before
  merge-change.

### T13 report, 2026-10-14

Tests added:
- invitation_exchange::the_invite_is_returned_only_after_its_store_write — verifies: LLR-h9qf3s
- invitation_exchange::an_unconfirmed_reply_parses_and_writes_nothing — verifies: LLR-9cb443
- invitation_exchange::admission_runs_check_admit_write_commit_send_settle_in_order — verifies: LLR-y4dg74
- invitation_exchange::a_failed_send_still_settles — verifies: LLR-y4dg74
- invitation_exchange::a_refused_check_submits_nothing — verifies: LLR-y4dg74
- node_stories::five_stories_full_e2e — verifies: REQ-a2a6rm, LLR-cvyyr8, REQ-2ugh2f

red -> green:
- the_invite_is_returned_only_after_its_store_write — watched fail for the right reason before the implementation existed (compile: E0599 no `issue_invite` on `OrgIo`; E0432 unresolved `org_io::ExchangeError`, `org_io::test_support::store_writes_to`)
- an_unconfirmed_reply_parses_and_writes_nothing — watched fail for the right reason before the implementation existed (compile: E0599 no `produce_reply`; E0432 `ExchangeError`, `store_writes_to`)
- admission_runs_check_admit_write_commit_send_settle_in_order — watched fail for the right reason before the implementation existed (compile: E0599 no `admit_reply`, no `FakeChain::updates`)
- a_failed_send_still_settles — watched fail for the right reason before the implementation existed (compile: E0599 no `admit_reply`)
- a_refused_check_submits_nothing — watched fail for the right reason before the implementation existed (compile: E0599 no `admit_reply`, `FakeChain::updates`)
- five_stories_full_e2e — watched fail for the right reason before the implementation existed (compile: E0599 no `issue_invite`/`produce_reply`/`import_reply`/`admit_reply`)

inherited (moved under ruling A from app/src-tauri/tests/invitation.rs,
now deleted; `verifies:` unchanged; rewritten onto `issue_invite`,
`produce_reply`, `import_reply`, `admit_reply`):
- an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id, no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to — from LLR-9sraks
- outstanding_invite_ids_survive_a_reopen_and_settle_one_at_a_time — from LLR-f35pda (reopen reads the store files back)
- an_outstanding_invites_file_in_the_data_dir_is_never_read — from LLR-f35pda (rewritten from `an_outstanding_invites_file_that_is_not_a_list_of_pairs_is_refused`; LLR-f35pda as amended has no file; dated comment)
- a_confirmed_reply_carries_the_persona_and_declares_the_expected_admission, no_reply_is_produced_and_nothing_declared_without_confirmation — from LLR-w4mhd4
- no_reply_is_produced_and_nothing_declared_for_a_bound_persona, a_bound_persona_cannot_reply_and_an_unbound_one_on_the_same_device_can, a_bound_persona_founds_nothing_and_the_chain_is_not_written — from LLR-rt8gdz
- a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding, a_reply_whose_admission_fails_on_chain_stays_outstanding, a_reply_admitted_under_another_selected_organisation_is_refused, a_reply_is_settled_once_committed_even_when_the_send_fails — from LLR-gha5f6 (the last now asserts Ok with the failed send in `AdmitReport.sends`; dated comment)
- a_reply_naming_another_held_organisation_with_an_outstanding_invite_id_is_refused — from LLR-gha5f6, LLR-f35pda, REQ-65xqp8
- node_stories::five_stories_full_e2e is a rewritten copy of org-node's story test; it cites org-io items, the org-node originals named in prose

Result: org-io line 1 (with `invitation_exchange` 19, `node_stories` 1)
178 passed, both fuzz targets clean; line 2 11 passed; build and clippy
clean; check-ids exit 0 for org-io and app; check-trace org-io: T13's four
MISSING-TEST gone, left are the six `expects: org-node` REQs (unchanged
from the change branch), NON-EXPORTED-REF LLR-gdfqn9, UNMET-EXPECTATION
REQ-ysyu9g, UNRESOLVED-PR ×3. org-node untouched; app not compiled (T15).

Notes:
- `OrgIo::expect_admission` is gated `#[cfg(feature = "test-support")]`
  (dated doc note); production declares an expectation through
  `produce_reply`, behind the confirmation. org-node's
  `OrgService::expect_admission` is kept: org-node and org-io test setups
  call it. The deslop pass or T18 may make it test-support-only too.
- Moved from the app to org-io: LLR-9sraks, LLR-f35pda, LLR-w4mhd4,
  LLR-rt8gdz (narrowed: its webview half stays the app's — T15 step 4
  note), LLR-gha5f6 (amended: a failed send no longer fails `admit_reply`);
  their assessments moved to org-io's risk draft. REQ-prjja8, REQ-tcutr6,
  REQ-65xqp8 marked `exported: yes` (accepted, S2 precedent).
- App files edited beyond the list for the moves (T12 precedent):
  decomposition.md, 2026-10-07-org-key-pair.md, the two risk files,
  src-tauri/Cargo.toml (`[[test]] invitation` gone), app config
  (`--test invitation` gone).
- App check-trace now reports NON-EXPORTED-REF for the five moved LLRs and
  MISSING-TEST REQ-m8sgjk — T15's (step 4 note).
- As landed: every operation takes `rng`; `import_reply(blob) ->
  InviteReply` parses then runs `check_reply`; `ExchangeError { NotConfirmed,
  Node, Store, Submit(String), Settle { admitted, error } }`; `AdmitReport
  { member_id, sends }`; `InviteReply`, `AdmitReport`, `ExchangeError` on
  the allowlist; after a submission error the pair is not settled and the
  Persona reloads. Test seams: `FakeChain::updates()`,
  `test_support::store_writes_to`.
- For T18: add `--test invitation_exchange --test node_stories` to
  `ORG_IO_COVERAGE_ARGS`.

### T14 report, 2026-10-14

Tests added:
- absences::org_node_is_sans_io — verifies: LLR-4tcxsu, REQ-x3jrh8
- absences::org_node_holds_no_dialling_address — verifies: LLR-qezw3n

red -> green:
- org_node_is_sans_io — watched fail for the right reason before the implementation existed (runtime, before the deletions: "src/keys.rs contains `iroh`: LLR-4tcxsu: the transport is org-io's")
- org_node_holds_no_dialling_address — watched fail for the right reason before the implementation existed (runtime, before the deletions: "src/transport/endpoint.rs contains `node_addr_for_dial`: LLR-qezw3n")

inherited: none. Deleted with the transitional send, each with a dated
comment in place: `admission_sender::in_loopback_mode_the_joiner_is_dialled_at_the_full_address`
(unannotated, T12), `admission_sender::pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key`
(unannotated pin, inverted in org-io in T10),
`commit_paths::send_update_refuses_without_a_bound_persona_or_a_loopback_address`
(LLR-2xzys9; every assertion a `send_body` refusal; LLR-2xzys9 narrowed to
"the commit names the sending Persona", now asserted by
`send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other`).
S2's `tests/preflight.rs` (two unannotated tests) deleted with the
transport preflight.

Result (task branch): org-node verify line 360 passed (361 − 3 + 2), six
bolero targets clean; org-io line 1 158, line 2 11; builds with no
features and `--features app` warning-free; lib clippy clean (three feature
sets); all-targets clippy only the T18 hits; check-ids and check-trace
org-node exit 0 (UNRESOLVED-PR 5). Quint not re-run (no model input or
driver changed).

Combined tree after merging T13 and T14 (dispatcher, 2026-10-14): org-io
line 1 178 passed, org-node verify line 360 passed, org-io clippy (all
targets, test-support,dev-seed, -D warnings) clean.

Notes:
- org-node lost `transport`, `iroh`, `tokio` (dev too), `async-trait`, the
  `preflight` bin and test. About 165 `#[tokio::test]` became `#[test]`;
  support's `Deferred` gained `run()` in place of `IntoFuture`;
  `spawn_receive` returns a `Receiving` alias.
- Green tests rewritten at call sites (dated comments, annotations
  unchanged): the failed-send tests now build and drop the body;
  `endpoint().is_none()` assertions point to `org_node_is_sans_io`;
  `provisional_store` compares with `wire::MAX_MESSAGE`. LLR-t4znbk and
  LLR-2xzys9 carry dated "Narrowed 2026-10-14" notes.
- `DeviceSecret::signing_keypair` gated `cfg(any(test, feature = "app",
  feature = "test-support"))`.
- SOUP: tokio and async-trait rows removed (dated paragraph); iroh rows
  kept as "Left 2026-10-14" pointers; 13 direct dependencies, 11 SOUP.
- Editor-tool `replace_all` was repeated until grep showed no `.await`
  left; a race between two parallel edits was restored and redone
  serially. The deslop pass checks indentation in org-node tests.

### T15 report, 2026-10-14

Tests added:
- ipc::the_app_manifest_names_no_iroh_base64_or_postcard — verifies: LLR-vzf8j2
- app_boundary::org_io_reexports_no_service_store_secret_or_wire_type_of_org_node — verifies: LLR-3zdw8v
- ipc::admit_member_hands_the_reply_to_org_io_and_reports_a_committed_admission_whose_send_failed — verifies: REQ-m8sgjk (step 4 note)
- receiver_events::the_store_and_invitation_refusals_are_classified_as_receiver_errors — verifies: LLR-7bk6qh (beyond the list)
- invite.personas "offers a Persona whose Organisation is the empty string as bound" — verifies: LLR-pms47e (new app LLR, step 4 note)

red -> green:
- the_app_manifest_names_no_iroh_base64_or_postcard — watched fail for the right reason before the implementation existed (runtime: "the app's [dependencies] name ["base64", "postcard", "iroh"]")
- org_io_reexports_no_service_store_secret_or_wire_type_of_org_node — watched fail for the right reason with the implementation temporarily reverted (the stopped first agent had already narrowed the re-export; with `pub use org_node as node;` restored: "org-io re-exports org-node's internals: the whole of org-node")
- admit_member_hands_the_reply_to_org_io_and_reports_a_committed_admission_whose_send_failed — NOT red first (the first agent's commands.rs already did it). Mutation-checked: without `report_sends(&admitted.sends)?` it fails
- the_store_and_invitation_refusals_are_classified_as_receiver_errors — NOT red first (the arms were needed to compile). Mutation-checked: `OrganisationHeldByAnotherPersona` in the VerifyFailed arm fails it
- "offers a Persona whose Organisation is the empty string as bound" — NOT red first (behaviour existed, as step 4 expects). Mutation-checked: filter `!p.org_id` fails it

inherited: none. Green tests whose only ID moved, now unannotated with a
dated prose pointer: `ipc::produce_invite_reply_refuses_without_confirmation_over_ipc`
(LLR-w4mhd4), the three earlier invite.personas tests (LLR-rt8gdz). Green
tests rewritten at call sites, annotations unchanged: ipc
`import_invite_reply_*` ×2 (LLR-pmus9f; a real Invite over IPC on
FakeChain; the second malformed case is a truncated reply),
`export_invite_does_not_refuse_a_well_formed_org_id`,
receiver_events `the_commit_workflow_refusals_are_classified_as_receiver_errors`
(lost `NoRevocationForRecipient`).

Result: app cargo verify line 133 passed; vitest 45 passed; svelte-check 0
errors 0 warnings; app builds with no features and `dev-seed`; app clippy
clean; org-io line 1 179 passed, line 2 11, build and clippy clean;
check-ids exit 0 (app, org-io); check-trace app exit 1 only on
NON-EXPORTED-REF LLR-85zque (T17) and the five 30-day PRs (merge blocker,
T12 note); org-io unchanged; check-units clean. org-node not changed.

Notes:
- Ahead of T17 (minimum to compile): `init` still requires
  `ODS_PASSPHRASE` but no longer passes it; `AppState::for_test` keeps an
  unused `_passphrase`; `a_store_under_another_passphrase_is_refused`
  deleted with a dated comment (T17 step 2's deletion, done early);
  `AppState.outstanding` and `outstanding_invites.json` gone.
- Ahead of T16: `next_outcomes` still calls `next_received` under the app's
  Mutex; `Received::Committed` Removed/Revoked → SelfDeleted, else Updated;
  non-refusal errors still classed terminal by message text;
  `start_receiver` calls `start_receiving` first, a failed bind emits error
  and stop.
- Test seams: `AppState::for_test_over(data_dir, &FakeChain, mode)`;
  `for_test`/`for_test_over` install `MemoryKeychain::shared()`; app
  `test-support` enables `org-io/test-support`; `OrgIo::open_for_test`
  (test-support only) documented under LLR-3zdw8v.
- `admit_member` / `revoke_member` return Err "the update was committed,
  but sending it failed: <device>: <error>" when any send failed (assessed
  in app/docs/risk/2026-10-08-submission.md, no new hazard). Alternative,
  Ok plus a warning, needs a DTO change — offered to the owner.
- Fixed org-io's `unused import: TransportError` (gated on test-support).
- The app's npm scripts hit EPERM in the sandbox (vite writes
  `node_modules/.vite-temp` through the symlink); the steps were run
  directly (`svelte-kit sync`, `vitest run --configLoader runner`,
  `svelte-check`). Symlink removed.
- LLR-pms47e defined in app/docs/architecture/2026-10-06-invitation.md
  (`satisfies: derived`, under SDD-jx363y) and assessed in
  app/docs/risk/2026-10-05-derived.md. REQ-m8sgjk narrowed (dated note).
  PR-7zz76t resolved obsolete. PR-5mc4d8 stays open (`ODS_CONTRACT_H160`
  in state.rs is still a site). app/AGENTS.md does not exist.
- Left for T16: Revoke.svelte labels ("hex-encoded postcard EndpointAddr"),
  api.ts comments, iroh message-text fragments in events.rs and
  receiver_events.
- For T18: `OrgNodeError::StoreRefused` cannot be constructed from the app
  (`StoreRefusal` not re-exported); that arm is compile-checked only.

### Reconciled with S3b-io part 1 (`68ec432`), 2026-10-14

Local master moved to S3b-io part 1 during S4 (the owner's note of
2026-10-14). It was merged after T15, in task branch `tm` (merge `aac4493`,
fix `5d798d3`). Sixteen files conflicted, and none was taken wholesale from
one side. The S3b-io features are now expressed on S4's structure:
- settling an unknown-outcome write, and reconcile at start and on request,
  write through org-io's store files;
- `publish` state sits behind its own lock, taken after the node lock;
- a late commit goes out by `send_committed` to every recipient at the
  recorded `PeerAddr`, and `UnknownWrite` holds no recipient;
- a failed store write gives `StillUnknown` or `Refused` and keeps the
  unknown write, which fixes the cause of PR-jgvf3b;
- LLR-pk6mhv's founding rule sits inside S4's `change` closure.

Every S3b-io test was kept with its `verifies:` line, rewritten onto S4's API
with a dated comment: publish_path ×26, startup_reconcile ×6, commit_paths
×3 (sync), app reconcile ×3, app ipc ×3. Two new tests are verified on
mutated code (the write not kept):
- `a_settle_whose_commit_is_not_stored_stays_unknown_and_commits_next_time` (LLR-t98pk9)
- `a_reconcile_whose_commit_is_not_stored_keeps_the_unknown_write_and_sends_next_time` (LLR-k5pzsz)

Dated notes were added to SDD-6mpmsw, LLR-2yjuaj, LLR-t98pk9, LLR-nqvbb6,
LLR-fptp57, LLR-k5pzsz, LLR-cc59ys, REQ-by9k5q, REQ-cz753a and org-node's
LLR-pk6mhv. The REQ-by9k5q risk passage was re-assessed: no new hazard.

Result on the merged tree:
- org-node 363 passed; Quint 0.33.0 typechecks OK and five invariants with
  no violation.
- org-io line 1 218 passed (publish_path 28, startup_reconcile 6), fuzz
  clean; line 2 11; build and clippy clean.
- App cargo line 139, clippy clean, vitest 52, svelte-check 0/0.
- check-ids exit 0 for all three units; check-units clean.

Trace delta against T15:
- org-node gains UNMET-EXPECTATION REQ-and2gz (S3b-io's, ruled 2026-10-13)
  and UNRESOLVED-PR PR-jgvf3b.
- org-io: PR-wk7zwq resolved, PR-6nhrr7 opened (both S3b-io's).
- app: PR-924ftr moved to org-io, resolved.
- The age limits rose to 60 days (S3b-io's ruling), which **clears the
  30-day merge blocker of the T12 note**.

Dispatcher's rulings on the reconciliation:
- A late commit now fans out to every recipient. REQ-by9k5q's "the Device
  the submission named" becomes "the recipients the submission would have
  sent to". This is accepted as a clarification with a dated note, not a
  superseding ID. S3b-io's own plan (§3) expected the move to
  `send_committed`. The requirement's point, that a late update never
  reaches a later call's recipient, is unchanged. Offered to the owner.
- `Settled::Committed.sent` stays `Result<(), String>`, summarising the
  per-Device report, which keeps S3b-io's public type. A `SendReport` is
  for S3b-io part 2.
- The LLR-cc59ys removal scan exempts `store_file::remove_store`, which
  runs only on org-node's `Forgotten`. Accepted as a clarification.

Carried forward:
- T16: `reconcile_organisations` and `list_orgs` still lock the app's
  `Mutex<OrgIo>`; amend app LLR-ekz4zn when that lock goes.
- T18:
  - keep `--test publish_path --test startup_reconcile` in
    `ORG_IO_COVERAGE_ARGS`;
  - PR-jgvf3b can be resolved, naming the org-io reproduction in prose;
  - settle, reconcile and reconcile_held hold the node lock for their whole
    sequence, as the submission does (the T11 note).
- Deslop: the duplicate counter names `updates` and `update_writes` in test
  support.

### T16 report, 2026-10-14

Tests added:
- receiver_events::terminal_is_decided_by_the_error_variant_not_its_text — verifies: LLR-zw6jqy, LLR-a9rjtf
- receiver_events::the_outcome_kind_comes_from_receive_kind — verifies: LLR-bym4md
- receiver_events::a_revocation_accepted_by_its_notice_is_announced_as_self_deleted — verifies: LLR-2vg79y, LLR-bym4md
- receiver_guard::the_receiver_holds_no_app_lock_while_waiting — verifies: LLR-5wb4vd

red -> green:
- terminal_is_decided_by_the_error_variant_not_its_text — watched fail for the right reason before the implementation existed (compile: E0308 `outcomes_for_receive_error` expected `&OrgNodeError`, found `&ReceiveError`)
- the_outcome_kind_comes_from_receive_kind — watched fail for the right reason before the implementation existed (compile: E0432 unresolved `ods_poc_lib::events::Arrival`; E0425 no `outcome_for_arrival`)
- a_revocation_accepted_by_its_notice_is_announced_as_self_deleted — watched fail for the right reason before the implementation existed (compile: E0425 no `outcome_for_arrival`)
- the_receiver_holds_no_app_lock_while_waiting — watched fail for the right reason before the implementation existed (runtime, against the app's `Mutex<OrgIo>`: "list_personas completes while the receiver waits for a message: Timeout" after 5 s)

inherited: none. Green tests rewritten at call sites onto org-io's
`ReceiveError`, annotations unchanged, dated comments:
`every_real_terminal_message_stops_the_loop`,
`a_non_terminal_failure_does_not_stop_the_loop`,
`the_stop_is_announced_after_the_failure_for_the_same_error` (LLR-a9rjtf),
`a_refusal_this_change_added_does_not_stop_the_loop` (REQ-jfxah3,
REQ-kn5rtx); `terminal_cases()` builds org-io's two `TerminalCause`
values; one input string in
`a_chain_failure_emits_no_verification_event_for_any_message` lost its iroh
wording.

Result: app cargo verify line 143 passed (139 + 4); app clippy clean;
vitest 52; svelte-check 0/0; check-ids exit 0 (app, org-io); check-trace
app: no open expectation (REQ-x3c8n2 met), otherwise as before
(LLR-85zque for T17); org-io as before. org-io changed in docs only;
org-node untouched.

Notes:
- LLR-a9rjtf (stop when the message *contains* a fragment) is superseded,
  not amended: T16 reverses it for non-terminal errors with that text.
  New LLR-zw6jqy (`supersedes: LLR-a9rjtf`; `satisfies: REQ-jfxah3,
  REQ-kn5rtx, REQ-x3c8n2`); LLR-a9rjtf carries `superseded-by:` and a dated
  note. The plan's §3.6 had "amend"; the hybrid rule wins.
- Annotations differ from step 1: neither LLR-7bk6qh nor LLR-a9rjtf states
  the kind mapping, so new LLR-bym4md (`outcome_for_arrival`, the `Arrival`
  enum in events.rs; `satisfies: REQ-tw4cb5, REQ-2k7ys4, REQ-dp95pv`);
  LLR-7bk6qh a dated clarification only. The lock test verifies new
  LLR-5wb4vd under SDD-rmbr3t (`satisfies: derived`, assessed in
  app/docs/risk/2026-10-05-derived.md, no new hazard).
- Dated clarifications: SDD-mwqf6x, SDD-rmbr3t, LLR-2vg79y, LLR-ekz4zn
  ("calls the shared handle"; carried from the S3b-io reconciliation),
  REQ-x3c8n2 (met). org-io REQ-8e57wm now `exported: yes`, `satisfies:
  REQ-x3c8n2` (was derived).
- The receive loop holds its own `Arc` of the handle; a failed bind emits
  through `events::outcomes_for_receive_error` (same two events). Admitted
  maps to Updated (the app has no admitted outcome).
- T15 leftovers done: Revoke.svelte and api.ts no longer describe a
  "postcard EndpointAddr"; no iroh fragments or `TERMINAL_ERRORS` in the
  app. org-io's `TransportError` text still says "iroh …" (org-io's).
- **For T18 (dispatcher's default, offered to the owner):** app-hazards.md
  says HAZ-cfp4jb's and HAZ-ny7yvt's unreduced probabilities depend on
  REQ-x3c8n2, now met. T16 left a dated note that the estimates stand.
  T18 re-estimates both under `analyze-risks`.

### T17 report, 2026-10-14

Tests added:
- startup_policy::startup_reads_no_passphrase — verifies: LLR-tpkmh3
- startup_policy::a_keychain_refusal_fails_startup_naming_the_condition — verifies: LLR-tpkmh3, REQ-7g3k9a

red -> green:
- startup_reads_no_passphrase — watched fail for the right reason before the implementation existed (runtime: "the app still names ODS_PASSPHRASE in [.../src/policy.rs, .../src/state.rs]")
- a_keychain_refusal_fails_startup_naming_the_condition — red at compile only (E0061: `for_test` took 4 arguments); once the parameter went it passed first time, since T9/T15 already passed `OrgIo::open`'s error on verbatim. NOT red at runtime first. Mutation-checked: with `for_test`'s `map_err` replaced by a generic "the store could not be opened" it fails ("the refusal names the Persona's store")

inherited: none. Deleted with `resolve_passphrase`, each with a dated
comment: `configured_passphrase_is_used`, `absent_passphrase_is_refused`,
`empty_passphrase_is_refused_not_honoured`,
`absent_passphrase_with_opt_in_uses_dev_default`,
`configured_passphrase_wins_over_opt_in` (LLR-tpkmh3),
`passphrase_refusal_names_both_variables_supplier_first` (LLR-97rww8).
`state_assembly::a_store_under_another_passphrase_is_refused`: T17 step 2's
deletion, done in T15 (comment updated).
`refusals_for_empty_values_name_both_variables_supplier_first` lost its
passphrase half (annotation unchanged).

Result: app cargo verify line 139 passed (143 − 6 + 2); app clippy (all
targets, test-support; plain) clean; dev-seed build OK; svelte-check 0/0;
vitest 52; check-trace app **exit 0** (LLR-85zque cleared; advisory PRs
only); check-ids exit 0; check-units clean. org-io and org-node untouched.

Notes:
- LLR-85zque citations (step 4 note): neither remaining test needed an edit
  for the passphrase, so the moved-ID rule was applied rather than a stop:
  `ipc::connection_status_reports_a_data_dir_it_had_to_create_verbatim`
  keeps LLR-vqkr5t (dated note);
  `state_assembly::init_opens_the_store_once_even_when_the_connect_is_refused`
  had only LLR-85zque and runs unannotated with a dated prose pointer.
  Dispatcher: accepted, the same rule as T9/T15. Tracing that scan to an app
  item would need a new red-first test — offered to the owner.
- App docs point to org-io's REQ-6d9xv6, REQ-fh5gsq, RC-d8dbxs, HAZ-5n56w2
  in prose (not exported to the app).
- Dated notes (plan §13, IDs kept): REQ-7g3k9a, RC-xt4qr3, LLR-tpkmh3
  amended; REQ-bmk2z2, LLR-97rww8 narrowed to the data directory;
  SDD-k95rmp narrowed; SDD-aq7m6b, LLR-vqkr5t clarified.
- No passphrase prompt existed in app/src. app/README.md lost the
  `ODS_PASSPHRASE` row. Demo docs outside the unit still set it — T18 step 11.
- HAZ-8ghmhn's estimate left for T18 (step 9).

### T18 report (steps 1–3, 5–11), 2026-10-14

Coverage:
- `ORG_IO_COVERAGE_ARGS` gains S4's 13 targets; S3b-io's `publish_path`
  and `startup_reconcile` are kept.
- Measured with floors at 0, stable 1.99.0 and cargo-llvm-cov 0.9.0:
  lines 81.24% (1568/1930), regions 79.94% (2805/3509). Branches, measured
  on nightly-2026-10-03: 115/138 (83.33%).
- Floors raised: `ORG_IO_LINES` 70 → 80, `ORG_IO_REGIONS` 68 → 78,
  `ORG_IO_BRANCHES` 64 → 82 (dated Makefile comment).
- The class C shortfall beside the floors is for the owner to accept:
  - the chain shell (preflight, connect, chain_read, signatory);
  - transport/mod.rs 54% (relay / Networked binding);
  - the platform keychain and file-system refusal arms (keychain.rs 91.9%,
    store_file.rs 90.4%).
- Step 6's premise did not hold: org-node and the app have no coverage
  target or floor. Measured for information only: org-node lines 98.18%,
  regions 95.34%; app lines 73.14%, regions 68.04%.

Clippy (both lines now in `verify_commands` and in CI, with dated
comments):
- `cargo clippy -p org-node --all-targets --features app,test-support -- -D
  warnings`: red → clean.
  - value_types: file-level allow gains `clippy::panic`.
  - fuzz_verify_against_chain: test-file allow header; `&*local` →
    `&local`.
  - The `CloneProbe` tests in key_custody and outgoing_bodies: `?Sized`
    dropped, plus targeted reasoned allows.
  - PR-6f3qku resolved by its own outline.
- `cargo clippy -p org-io --all-targets -- -D warnings`: red → clean, via
  `[[test]] key_custody_config` with `required-features = ["test-support"]`.

CI (`.github/workflows/rust.yml`): the clippy job gains both lines; the
org-node and org-io test steps are verbatim again. The org-node step had
listed deleted targets; the org-io step lacked `open`, S4's targets and
S3b-io's.

Trace:
- LLR-gdfqn9 removed from org-io's
  `absences::no_handle_method_returns_a_type_holding_an_outgoing_body` with
  a dated note. The test still verifies LLR-3zdw8v. Four source comments
  are now prose.
- PR-jgvf3b resolved, naming the org-io reproduction in prose.

Risk (analyze-risks; dated notes in app/docs/risk/2026-09-14-app-hazards.md):
- HAZ-8ghmhn stands at S3/P1. The reason for P1 is restated: no code path
  can put key material under a constant.
- HAZ-ny7yvt stands at S3/P2. Its main contributor is open PR-eecx3y.
- HAZ-cfp4jb is not recorded: the evidence points to S3/P1, which changes a
  rating. Owner question 2 below.

Step 10 lists, for the record:
- Retired: LLR-379hnv (T4), LLR-g9vmbx (T6), LLR-23sfdh → LLR-jd82t6 (T9),
  LLR-q225ws (T12). Superseded, not retired (kept with `superseded-by:`):
  LLR-a9rjtf → LLR-zw6jqy (T16). *Corrected 2026-10-14 (review round 1
  finding-17).*
- Green tests with no `verifies:` line:
  - org-node: `absences::forget_organisation_is_defined_once_and_called_only_from_accept_and_the_removal_step`,
    `service_stories::a_commit_that_removes_another_member_forgets_nothing`;
  - org-io: `submit_flow::each_command_sends_once_to_one_device_and_the_kind_follows_the_record`,
    `::a_revocation_of_a_member_the_record_does_not_hold_is_refused_and_sends_nothing`;
  - app: `ipc::produce_invite_reply_refuses_without_confirmation_over_ipc`,
    the three earlier invite.personas tests,
    `state_assembly::init_opens_the_store_once_even_when_the_connect_is_refused`.

Step 11: `ODS_PASSPHRASE` removed from both demo docs, with dated notes.

Closing checks on `2f4a269`:
- check-units exit 0 (6 units, 756 tracked paths).
- check-trace / check-ids per unit:

  | Unit | check-trace / check-ids | Findings |
  |---|---|---|
  | org-members | 0 / 0 | none |
  | on-chain-client | 0 / 0 | UNRESOLVED-PR ×4 |
  | org-node | 0 / 0 | UNMET-EXPECTATION REQ-q92yac, REQ-and2gz; UNRESOLVED-PR ×4 |
  | app | 0 / 0 | UNRESOLVED-PR ×9 |
  | person | 0 / 0 | none |
  | org-io | **1** / 0 | MISSING-TEST on the six `expects: org-node` REQs (jxyh9c, qza96g, txkm94, tzr3dk, v6r2gd, x6kdse); UNMET-EXPECTATION REQ-ysyu9g; UNRESOLVED-PR ×3 |

- Every unit's `verify_commands` exits 0, except the app's
  `npm run test`, which hits the sandbox EPERM; run directly, vitest gives
  52 passed.
  - org-members 271 passed, quint 63 passing;
  - on-chain-client 73 + 57;
  - person 123;
  - org-io 218 + 11, build and both clippy lines clean;
  - org-node 363, Quint typechecks and five invariants with no violation,
    all-targets clippy clean;
  - app cargo 139, svelte-check 0/0.
- `make coverage coverage-branch` exit 0.

**Open for the owner:**
1. org-io check-trace exits 1, and has since T13. Under guardrails 0.5.1 a
   met expectation REQ is "an ordinary REQ again" and owes an org-io test.
   The self-review line "met by org-node's exported REQs (no test owed)" is
   wrong.
2. The HAZ-cfp4jb re-estimate.

CI now runs org-io's networked and keychain targets on Linux (memory
keychain, loopback); a relay flake there is possible.

Owner rulings 2026-10-14: (1) add org-io tests for the six met
expectations; (2) lower HAZ-cfp4jb only after a stop-and-restart test →
T19.

### T19 report, 2026-10-14

Tests added (org-io/tests/expectations_met.rs, plus the app):
- every_store_change_reaches_the_file_whole_and_reopens_as_held — verifies: REQ-x6kdse
- a_store_file_org_node_cannot_unseal_is_refused_naming_it_and_left_untouched — verifies: REQ-x6kdse
- a_store_file_opens_under_its_own_item_and_keeps_the_device_key_not_the_seed — verifies: REQ-qza96g
- a_store_file_opens_under_no_other_personas_secret — verifies: REQ-qza96g
- the_device_seed_is_in_the_keychain_alone_and_renders_no_bytes — verifies: REQ-v6r2gd
- without_its_keychain_item_a_store_file_opens_nothing — verifies: REQ-v6r2gd
- one_receive_path_commits_a_first_admission_an_update_and_the_own_revocation — verifies: REQ-tzr3dk
- a_body_the_check_refuses_reads_no_chain_and_writes_nothing — verifies: REQ-tzr3dk
- each_recipient_receives_the_body_org_node_built_for_it — verifies: REQ-jxyh9c
- the_last_other_member_removed_is_still_sent_its_revocation — verifies: REQ-jxyh9c
- outstanding_invites_and_expectations_are_kept_and_settled_in_the_store_files — verifies: REQ-txkm94
- blobs_that_do_not_parse_and_a_reply_for_another_organisation_change_no_file — verifies: REQ-txkm94
- receiver_guard::a_closed_endpoint_stops_the_loop_announces_it_once_and_releases_the_guard — verifies: LLR-yrb6th (new app LLR under SDD-rmbr3t)
- receiver_guard::after_a_stop_start_receiver_starts_a_loop_that_runs — verifies: LLR-yrb6th; `#[ignore]`d, reproduces PR-tcd2ta (T20)

red -> green: the behaviour existed (owner ruling 1), so each test carries a
mutation check, reverted afterwards:
- every_store_change_… — `issue_invite` without its store write fails ("after issue_invite: the file holds the whole store")
- a_store_file_org_node_cannot_unseal_… — `open` skipping failed stores fails ("opened, and it should have been refused")
- a_store_file_opens_under_its_own_item_… — the seed appended to each store file fails (`Unseal … StoreRefused(NotAuthentic)`)
- a_store_file_opens_under_no_other_personas_secret — no `KeyMismatch` check fails ("not the keychain's key mismatch"); the swapped-file half is also held by org-node
- the_device_seed_is_in_the_keychain_alone_… — the seed appended fails ("… holds the Device seed")
- without_its_keychain_item_… — skipping `KeychainItemMissing` stores fails
- one_receive_path_… — `settle` without its store write fails ("the admission is in B's file")
- a_body_the_check_refuses_… — the refusal mapped to `ReceiveError::Chain` fails. The mutation is weak: no org-io break shows the "no chain read" half
- each_recipient_receives_… — every recipient sent the first body fails ("Bob, removed, gets the revocation")
- the_last_other_member_removed_… — revocations skipped fails (`per_device: []`)
- outstanding_invites_and_expectations_… — `produce_reply` without its store write fails
- blobs_that_do_not_parse_… — `admit_reply` checking `reply.org_id` fails
- a_closed_endpoint_stops_… — `forget(guard)`, `break`→`continue`, and a doubled emit each fail it
- after_a_stop_start_receiver_starts_a_loop_that_runs — watched fail; still fails ("the restarted loop runs: it announces no stop"): PR-tcd2ta

Result: org-io line 1 230 passed (218 + 12), line 2, build, both clippy
lines clean; app cargo line 140 passed, 1 ignored; app clippy clean;
svelte-check 0/0; vitest 52; **check-trace org-io exit 0** (the six
MISSING-TEST gone; UNMET-EXPECTATION REQ-ysyu9g; UNRESOLVED-PR PR-8geawr,
PR-z3gg2b, PR-6nhrr7, PR-tcd2ta); app exit 0; check-ids exit 0.

Notes:
- **Ruling 2's condition not met:** the loop stops, announces once and
  releases the guard, but a restarted loop stops at once. Cause: in
  `org-io/src/endpoints.rs`, `Endpoints` keeps the closed endpoint, and
  `start_accepting` skips an entry whose `accept_task` is `Some` even after
  it ended. Recorded as PR-tcd2ta (org-io draft ledger). HAZ-cfp4jb stays at
  S3/P2 (dated note in §4.6) until T20.
- LLR-yrb6th satisfies REQ-3hfggn and REQ-jfxah3 (not derived; no
  analyze-risks pass).
- `expectations_met` added to org-io's verify line 1,
  `ORG_IO_COVERAGE_ARGS` and CI; floors unchanged.

### T20 report, 2026-10-14

Tests added (org-io/tests/endpoints.rs):
- after_its_endpoint_closes_start_receiving_binds_it_again_and_receives — verifies: LLR-z35vbh, LLR-d2pzzk, PR-tcd2ta
- after_its_endpoint_closes_a_send_binds_it_again — verifies: LLR-d2pzzk, PR-tcd2ta
- a_refused_rebind_is_a_failed_bind_and_the_next_start_binds — verifies: LLR-z35vbh, LLR-d2pzzk, PR-tcd2ta (abnormal case: keychain locked during the rebind)

red -> green:
- after_its_endpoint_closes_start_receiving_binds_it_again_and_receives — watched fail for the right reason before the fix existed ("start_receiving binds the closed endpoint again")
- after_its_endpoint_closes_a_send_binds_it_again — watched fail for the right reason before the fix existed (`Err(Transport(Connect("Endpoint is closed")))`)
- a_refused_rebind_is_a_failed_bind_and_the_next_start_binds — watched fail for the right reason before the fix existed ("the refused rebind was not a failed bind: Ok(())")
- app receiver_guard::after_a_stop_start_receiver_starts_a_loop_that_runs — `#[ignore]` removed (annotation unchanged); watched fail in T19, passes now

inherited: none.

Result: org-io line 1 233 passed, line 2, build and both clippy lines
clean; app cargo line 141 passed, 0 ignored; app clippy clean;
svelte-check 0/0; vitest 52; check-trace org-io exit 0 (PR-tcd2ta resolved),
app exit 0.

Notes:
- The fix is in `bound_or_bind`. An entry whose endpoint `is_closed()` is
  removed, and the endpoint is bound again under the map's mutex, so a
  Device never has two open endpoints. `start_accepting` starts a new
  accept task when the stored one `is_finished()`. The old accept task is
  not aborted: it counts itself out of `accepting`. A send already holding
  the closed `Arc` fails with a transport error.
- HAZ-cfp4jb lowered to S3/P1 under owner ruling 2: a dated note in §4.6,
  §7's row changed, and short notes beside the §3 claim 8 and §7 paragraphs
  and REQ-x3c8n2's T16 note. Acceptability unchanged (No).
- LLR-d2pzzk and LLR-z35vbh (drafts) amended in place before first merge,
  with dated notes. SDD-xwft92 got a dated note. LLR-6zjzn2 gained a dated
  clarification: the same endpoint is reused only while it is open.
- S3b-io constraints (merge notes item 7) untouched.
- Gap found, now T21: a rebind while the receiver runs starts no accept
  task.

### T21 report, 2026-10-14

Tests added (org-io/tests/endpoints.rs):
- a_send_that_rebinds_a_closed_endpoint_while_the_receiver_runs_accepts_on_it — verifies: LLR-z35vbh, LLR-d2pzzk, PR-47aycj
- a_refused_rebind_while_the_receiver_runs_binds_and_accepts_on_the_next_send — verifies: LLR-z35vbh, LLR-d2pzzk, PR-47aycj (abnormal case: keychain locked on the send-path rebind; the accept count stays exact)

red -> green:
- a_send_that_rebinds_a_closed_endpoint_while_the_receiver_runs_accepts_on_it — watched fail for the right reason before the fix existed ("the rebound endpoint has no accept task: nothing reached the receive channel")
- a_refused_rebind_while_the_receiver_runs_binds_and_accepts_on_the_next_send — written after the first fix; red shown with the send-path fix disabled (`if false &&`, restored): same panic. NOT red before the fix existed.

inherited: none.

Result: org-io line 1 235 passed, line 2 11, build and both clippy lines
clean; app cargo line 141 passed; check-trace org-io exit 0 (PR-47aycj
resolved; open PR-8geawr, PR-z3gg2b, PR-6nhrr7); check-ids clean.

Notes:
- `Endpoints` gains a `receiving: AtomicBool`, which `start_accepting` sets
  under the `bound` lock. The send path's `Endpoints::endpoint` calls
  `keep_accepting` (the old `start_accepting` body, factored out) under the
  same lock. `keep_accepting` counts a task into `accepting` before
  spawning it.
- A race was judged sound: `next_inbound` may decide the terminal
  `EndpointClosed` while a concurrent send starts an accept task. That race
  existed already with `start_receiving`. The bodies wait in the channel
  for the app's restart.
- A send never restarts a stopped app receive loop, and the `receiving`
  flag is never cleared. Noted on LLR-z35vbh.
- `new-id.sh` needs `GR_CONFIG` as a repo-relative path; an absolute path
  exits 2.

### Deslop pass, 2026-10-14

Commits `11c3153`, `41d0301`, `a85de9a`, `5782f0b`, `b45d093`, on the
change branch. No behaviour change.

Fixed:
- service.rs, the blocks T4 and T5 edited by script:
  - `CheckedKind::OrgInformation` no longer clones an `existing` record;
    `commit_checked` branches on `if let Some(expectation)`.
  - Lines and docs rewrapped; the `reconcile` and `CheckedMessage` docs
    corrected.
- org-io test support:
  - `updates()` merged into `update_writes()`.
  - Copied helpers moved to tests/common/handles.rs: `create`,
    `create_named`, `persona_named`, `fresh_keychain` (four copies of
    `static LOCK` + `fresh()`), `committed`, `refusal`, `file_of`,
    `held_store`, `forwarding`.
- org-node test support: the dead `_unused` passphrase argument removed
  from `open_store` and `reopen_store` (about 70 call sites).
- The 21 calls T14 left split across lines are rejoined.
- Transitional "until T10/T14" comments now say what happened.

Constraints checked, all hold:
- an unknown write is never forgotten;
- the publish lock is taken after the node lock;
- LLR-pk6mhv holds;
- no public discard method exists;
- secrets are moved and not cloned;
- the handle returns only allowlisted types;
- the keychain is touched only in org-io;
- there are no owner-only file modes.

Suite: org-node 363 passed; all-targets clippy clean; org-io 235 + 11 passed,
build and both clippy lines clean; app 141 passed, clippy clean;
check-trace and check-ids exit 0 for org-node, org-io and app.

Left alone:
- **Behaviour finding:** since T11 the node lock is released between
  `check_received` and `commit_checked`, so a local commit can land in
  between. `CheckedMessage` keeps the record's trie, sequence guard and
  epoch from check time, and no test covers the stale case. Now T22.
- `OrgNodeError::Chain` is org-node's catch-all for encode failures ("blob
  encode", "store encode", "encrypt failed", "persona not found"). It
  renders as "chain read failed: …", which misleads. A dedicated `Encode`
  variant and a `PersonaNotFound` variant would be a behaviour change: a
  follow-up for the owner, not S4.
- The remaining helpers each differ in form or are used by one file.

### T22 report, 2026-10-14

Findings at org-node's interface, before the fix (B's record at epoch 2;
a local commit lands between check and commit):
- **Another Member's update:** the stale commit is refused
  (`RootMismatch`); nothing changes or is lost.
- **This node's own update, relayed back by a listed Member before this
  node commits it: the defect.** The stale commit was accepted as `Updated`.
  It rewrote the record with the same values and gave a false `Updated`
  outcome and an Accepted verdict. A fresh check refuses it (`StaleSeq`).
- **Revocation:** refused (`RevocationProofRefused`), the same as a fresh
  check.
- **First admission:** refused (`AdmissionNotOurs`), the same as a fresh
  check.
- **Acknowledgement:** cannot interleave; it is decided under one lock.
- No local commit was lost or overwritten in any case.

Tests added:
- org-node stale_check::an_update_checked_before_a_local_commit_is_refused_as_moved — verifies: LLR-6qjmhf, PR-erskh4
- org-node stale_check::the_nodes_own_update_relayed_back_before_its_commit_is_refused_as_moved — verifies: LLR-6qjmhf, PR-erskh4
- org-node stale_check::a_revocation_checked_before_a_local_commit_is_refused_as_moved — verifies: LLR-6qjmhf, PR-erskh4
- org-node stale_check::a_first_admission_checked_before_its_persona_is_bound_is_not_ours — verifies: LLR-6qjmhf
- org-io receive_step::a_check_that_went_stale_is_checked_again_once — verifies: LLR-cvyyr8
- app receiver_events::a_record_moved_since_the_check_is_classified_as_a_receiver_error — verifies: LLR-7bk6qh

red -> green:
- an_update_checked_before_a_local_commit_is_refused_as_moved — watched fail for the right reason before the fix existed (`RootMismatch` in place of `RecordMovedSinceCheck`)
- the_nodes_own_update_relayed_back_before_its_commit_is_refused_as_moved — watched fail for the right reason before the fix existed ("committed: ReceiveOutcome{.. kind: Updated ..}")
- a_revocation_checked_before_a_local_commit_is_refused_as_moved — watched fail for the right reason before the fix existed (`RevocationProofRefused{AbsenceProofRootMismatch}`)
- a_first_admission_checked_before_its_persona_is_bound_is_not_ours — pins existing behaviour; mutation check: without the `persona.org_id.is_some()` filter it fails (`Ok(())` vs `AdmissionNotOurs`)
- a_check_that_went_stale_is_checked_again_once — watched fail for the right reason before the retry existed (`RecordMovedSinceCheck` in place of `StaleSeq{3,3}`)
- a_record_moved_since_the_check_is_classified_as_a_receiver_error — mutation check: the variant in the `VerifyFailed` arm fails it

inherited: none.

Result: org-node 367 passed, Quint typechecks and five invariants clean,
all-targets clippy clean; org-io 236 + 11 passed, build and both clippy
lines clean; app cargo line 142 passed; check-ids and check-trace exit 0
for org-node, org-io and app.

Notes:
- The fix:
  - `CheckedMessage` records the record's epoch, root and sequence guard
    at check time.
  - `commit_checked` first refuses with `OrgNodeError::RecordMovedSinceCheck
    { org_id }` if they moved; nothing is written and no secret is asked
    for.
  - org-io's `settle` repeats check, read and commit once, on that refusal
    only.
  - Test seam: `FakeChain::hold_next_read`.
- The app's `classify_receive_error` puts the new variant in the
  receiver-error class, the safe side of RC-3rddh7.
- Dated notes: LLR-6qjmhf amended (draft, before first merge), LLR-zetn7a
  clarified, LLR-cvyyr8 amended. `--test stale_check` added to org-node's
  verify line and CI.
- S3b-io constraints untouched: the retry takes only the node lock.
- **Raised for the owner:** a node whose record is behind the chain (an
  unknown write not yet reconciled) refuses, in the check, a valid update
  built on its own pending epoch (`DeltaBaseMismatch`). The sender gets
  Refused, and nothing in the receive path recovers that update.

### Reconciled with S2 (`e41cb09`), 2026-10-08

Master `e41cb09` (S2) was merged into this branch at `24ca24c`, and the plan
and drafts were re-read against S2's merged code and ledgers (§17). What
changed:

- **§17 added:** S2's names as landed (T0's table), the S2 rules S4 keeps
  (the handle's allowlist, no private key through a public API, the `node`
  re-export narrowed in S4, keychain in org-io only, S2's scans extended
  not loosened), the merged items S4 amends under the hybrid rule, and one
  new open point (§17.4).
- **Owner rulings of 2026-10-08 applied** (rulings 9–12; §16.4 marked
  ruled): the verdict byte (LLR-unqfj5, REQ-ur7z4a notes); HAZ-abm3xs's
  residual accepted (org-io risk draft); a forgotten Persona's keychain item
  kept until delivery ends or the Organisation's limit passes, deletion
  S3b-io's (LLR-amv4mb note, T9); the preflight's key through the keychain
  path, no `ODS_DEVICE_KEY` (LLR-h9tcat rewritten in place, REQ-hw82ue note,
  T10 rewritten). The key-provisioning choice is a new unminted "(later)"
  row in the roadmap.
- **Tasks:** T0 steps 0–4 done. T2: LLR-js9dsu → LLR-tajh9d; its planned
  leftover-bytes test dropped (S2's covers it); org-io and app test imports
  added. T3: `OrgIo::open` made transitional (S2 already opened the store
  there), the passphrase `open.rs` test deleted with LLR-rgdx22's
  narrowing, `NodeBuilders::create_persona` keeps returning `PersonaId` (the
  `DeviceSecret` goes into a private transitional holder), REQ-v4tfap's
  note moved here from T10, LLR-3zdw8v's scans gain `DeviceSecret`,
  `reconcile` already value-driven, `MockChainOps` gone. T4: S2's landed
  split renamed (`Prepared` → `Checked`), the decode moves into
  `check_received`, `receive_order.rs`'s 12 tests folded, LLR-7pj5af and
  LLR-mn5c2q notes, `SelfDeleteOutcome` leaves the allowlist. T5:
  `CommitOutcome` (whose bodies carry the Organisation private key) leaves
  the allowlist; `submit_commit_send` returns `Submitted`. T7: `zeroize`
  and `iroh` rows already exist in org-io's SOUP. T8: the keychain
  functions are `pub(crate)` with a `test-support` facade; the T3 holder is
  replaced. T9: S2's `OrgIo::open` changed in place (no `open_node`);
  `state_assembly.rs`'s four tests split; resolves S2's PR-jav8zn. T10: the
  preflight finds Device keys from the store-file listing; S2's
  `test-support` `ensure_endpoint` goes. T11: no `with_node` closure; the
  view and builders become `&self` methods; SDD-z3ychz note. T11a: S3b-io
  not merged, rulings recorded. T12: the LLR-q225ws tests come from the
  app's `submit_flow.rs`; `submit_commit_send` loses `recipient` and takes
  a `PeerAddr`. T13: `AdmitReport` etc. reviewed into the allowlist. T14:
  org-node's `tests/preflight.rs` deleted; `async-trait` already gone.
  T15: the `node` re-export narrowed. T16: REQ-x3c8n2 already `expects:
  org-io` (S2). T17: `OrgIo::open` without the passphrase.
- **Drafts:** org-node's sans-IO design now traces REQ-uk9rw7 (was the
  superseded REQ-ztdza4) and notes S2's landed names; its requirements
  draft corrects PR-q8r32t's resolver (S3a); the app's PR-7v8anh "Where"
  re-read after S2; PR-n696wv notes S2's related PR-jav8zn; org-io's three
  drafts carry the S2 reconciliation and the 2026-10-08 rulings.
- **Checks (2026-10-08, per unit, `GR_CONFIG=<unit>/.guardrails/config.yaml`):**
  `check-ids.sh --allow-draft-files`: exit 0 for org-io, org-node and the
  app, before and after this pass. `check-trace.sh`: the app exit 0
  (advisories only); org-io and org-node exit 1, with MISSING-TEST only for
  S4's not-yet-implemented draft items (org-io 49, org-node 19) and
  pre-existing UNRESOLVED-PR/UNMET-EXPECTATION advisories. The first
  org-node run also reported UNDECLARED-DEPENDENCY for two org-io IDs this
  pass had written into an org-node problem draft; they were replaced by
  prose and the re-run is clean of it.
