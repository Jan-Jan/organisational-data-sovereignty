# org-node

ODS Phase 2 node logic — the trust brain that sits above `org-members`.

This crate owns what `org-members` deliberately leaves to the caller: the
`Envelope` wire form, the Organisation binding, monotonic replay protection,
and the **verify-against-chain** flow. Nothing about the sender of an Envelope
is checked (owner ruling, 2026-10-05): the on-chain root at a newer epoch is
the sole authority.

## The one property

`verify_envelope_against_chain` commits a received membership change only if,
after checking org binding + sequence, applying the delta to the
local trie reproduces a root that **independently** matches the on-chain root
(read via `ChainReader`) at a newer epoch. The delta and the trusted root must
travel different trust paths.

## Status (Phase 2.1)

Pure core, no network/chain. The chain is abstracted behind `ChainReader`;
`MockChain` drives tests. Later phases wire `on-chain-client`/subxt (reads),
iroh transport, persona/org persistence, and the Tauri/Svelte shell.

## Layout

- `keys.rs` — `SigningKeypair` (the ed25519 device keypair, whose verifying
  key is the `DevicePublicKey`) and `X25519Keypair` (the secret behind a
  Member-as-a-group key `PersonPublicKey`, or the Organisation private key
  behind the Organisation public key).
- `types.rs` — the value types: the redacted secrets (`MemberSeed`,
  `DeviceSeed`, `OrgSecret`, `OrgPrivateKey`), `OrgPublicKey` (parsed through
  `person`'s X25519 rule), and the tags `ChainAccount`, `PersonaId`, `Epoch`,
  `SequenceNumber`.
- `ids.rs` — `OrgId` (= `h160_of(P)`).
- `chain.rs` — `ChainReader`, `OrgState`, `MockChain`.
- `envelope.rs` — `Envelope` (org_id, parent_seq, postcard(Delta); no signature).
- `sequence.rs` — `SeqGuard`.
- `verify.rs` — `verify_envelope_against_chain` + `VerifyContext`/`VerifiedUpdate`.

## Chain integration (Phase 2.2)

All on-chain code is gated behind the `chain` cargo feature. The Phase 2.1 core
(envelope / verify / sequence) compiles and tests without it.

### Feature flag

```toml
# Cargo.toml
[features]
chain = ["dep:subxt", "dep:tokio", "dep:on-chain-client", ...]
```

Enabling `chain` pulls in `subxt` 0.50, `on-chain-client` (path dep,
`dev-rpc` feature for chopsticks), and `tokio`.

### Reads — `OnChainReader`

`chain_read::OnChainReader` implements `ChainReader` over
`on-chain-client`'s `OrgRegistryClient`:

- **Async half:** `OnChainReader::refresh(&self) -> Result<(), String>` reads
  the `OrgState` at the latest **finalised** block (`at = None`, REQ-ysyu9g)
  and caches it in a `Mutex`.
- **Sync half:** `ChainReader::get_org_state` reads the cached snapshot
  synchronously, so `verify_envelope_against_chain` (which is sync) can call it
  without blocking.

Call `refresh` before `verify_envelope_against_chain` to ensure the snapshot is
current. `OrgState` derives `Copy`, so the snapshot is extracted cheaply.

For live Paseo, switch `on-chain-client` to its `smoldot` feature (future work).
The chopsticks tests (`preflight`) use the `dev-rpc` / jsonrpsee transport
with a `LegacyBackend` client.

### Writes

org-node writes nothing to the chain (LLR-65py3d): its `ChainOps` seam only
reads. The app makes every chain write — the genesis ceremony and each
update — through `on-chain-client`'s `write` feature, then asks org-node to
commit the provisional update the chain now carries (`commit_genesis`,
`commit_update`). The writer, its gated tests and the chopsticks genesis test
(`write_genesis_e2e`) live in `on-chain-client`.

## Transport (Phase 2.3)

### Feature flag

```toml
# Cargo.toml
[features]
transport = ["dep:iroh", "dep:tokio", ...]
```

All iroh-based transport code is gated behind the `transport` cargo feature.
The crate's Phase 2.1/2.2 core compiles and tests without it.

**Version note:** iroh is pinned to **0.98.2**, not 1.0. The workspace also
contains `spike-p2panda` / `p2panda-net`, which depend on an
`ed25519-dalek` pre-release that conflicts with iroh ≥ 1.0. Pinning to 0.98
keeps all crates in the workspace compatible.

### `OrgEndpoint`

`OrgEndpoint` wraps an iroh `Endpoint` to provide typed send/receive over the
ODS protocol:

- **Identity:** `EndpointId == DevicePublicKey` — the iroh node key is the device
  ed25519 key. Dialing a peer cryptographically proves device-secret custody;
  the TLS handshake is the authentication step.
- **Relay disabled:** built with `presets::Minimal` + `RelayMode::Disabled` for direct/loopback
  connections. Discovery and relay are left to future phases (see S3 note below).
- **ALPN:** `/ods/org-node/1` — both sides must present this protocol string;
  connections using a different ALPN are rejected.

### Wire protocol

Messages are typed as `WireMessage { envelope: Envelope, org_secret: Option<OrgSecret>, genesis_snapshot: Option<Vec<u8>>, invite_id: Option<InviteId> }`
(`genesis_snapshot` is the record the Envelope extends; the invite
identifier is present only on an admission).

Framing is **length-prefixed**: a 4-byte little-endian `u32` body length precedes
each `postcard`-serialised `WireMessage`. The maximum body size is 1 MiB
(`MAX_FRAME = 1 << 20`); oversized frames are rejected with an error before any
bytes are decoded.

### API

| Function | Description |
|---|---|
| `OrgEndpoint::bind(keypair) -> Result<OrgEndpoint>` | Bind an iroh endpoint on an OS-assigned port, using the given device keypair as the iroh node key. |
| `send(endpoint, addr, msg) -> Result<()>` | Open a QUIC stream to `addr`, frame and send `msg`, then flush. |
| `recv_one(endpoint) -> Result<(DevicePublicKey, WireMessage)>` | Accept one inbound connection, decode the framed message, and return both the message and the **authenticated remote DevicePublicKey**. |

**Security note:** `recv_one` returns the remote DevicePublicKey that was
authenticated by the iroh/QUIC handshake (the key the peer proved ownership of
via TLS). Authentication proves key custody, not membership. org-node's
receive paths do not compare this key with anything (owner ruling,
2026-10-05): what they commit is decided by the on-chain root at a newer
epoch, whoever delivered it.

### Two-node handshake test

```
CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node \
  --features transport --test transport_handshake
```

Spins up two `OrgEndpoint`s in-process, sends a `WireMessage` from node A to
node B over loopback, and asserts that:

1. The decoded message round-trips correctly.
2. The remote DevicePublicKey returned by `recv_one` matches the sender's key.

The test runs fully offline with no external services required.

### S3 note

iroh is used as a transport stand-in for the Phase 2 PoC. The PoC uses
relay-disabled direct connections (loopback / LAN). Real peer discovery,
relay fallback, and NAT traversal are out of scope for Phase 2 and will be
addressed in a future phase.
