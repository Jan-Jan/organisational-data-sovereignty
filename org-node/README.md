# org-node

ODS Phase 2 node logic — the trust brain that sits above `org-members`.

This crate owns what `org-members` deliberately leaves to the caller: the
`Envelope` wire form, the Organisation binding, monotonic replay protection,
and the **verify-against-chain** flow. The on-chain root at a newer epoch is
the authority for what is committed; for an Organisation the node already
holds, an update or revocation is acted on only when its sender's Device is
listed in the node's current record (owner ruling R1, 2026-10-07; a first
admission is accepted from any sender).

org-node reads and writes no chain (ruling B of 2026-10-06, change
`worktree-org-io-create`): it meets org-io by values in, values out. org-io
owns the `OrgService`, the chain connection, the reader, the writer and the
user's signatory key; it reads each Organisation's state and hands it to the
org-node operations that judge against it (see
`org-io/docs/architecture/` and `docs/adr/2026-10-06-org-io-unit.md`).

## The one property

`verify_envelope_against_chain` commits a received membership change only if,
after checking org binding + sequence, applying the delta to the
local trie reproduces a root that **independently** matches the on-chain root
(the `Option<OrgState>` value org-io read and passed in) at a newer epoch. The
delta and the trusted root must travel different trust paths.

## Status

No chain IO. Every operation that judges against the chain takes the
Organisation's state as a value; tests pass values from
`test_fixtures::ChainSlots`. The iroh transport and the encrypted store stay
here until stage S4 of `docs/plans/2026-10-06-org-io-roadmap.md` moves them
to org-io.

## Layout

- `keys.rs` — `SigningKeypair` (the ed25519 device keypair, whose verifying
  key is the `DevicePublicKey`) and `X25519Keypair` (the secret behind a
  Member-as-a-group key `PersonPublicKey`, or the Organisation private key
  behind the Organisation public key).
- `types.rs` — the value types: the redacted secrets (`MemberSeed`,
  `DeviceSeed`, `OrgPrivateKey`), `OrgPublicKey` (parsed through
  `person`'s X25519 rule), and the tags `ChainAccount`, `PersonaId`, `Epoch`,
  `SequenceNumber`.
- `ids.rs` — `OrgId` (= `h160_of(P)`).
- `chain.rs` — `OrgState` and its parse edge `OrgState::from_chain` (the
  chain read itself is org-io's).
- `envelope.rs` — `Envelope` (org_id, parent_seq, postcard(Delta); no signature).
- `sequence.rs` — `SeqGuard`.
- `verify.rs` — `verify_envelope_against_chain` + `VerifyContext`/`VerifiedUpdate`.

## The chain (values in, values out)

*Rewritten 2026-10-08 (change `worktree-org-io-create`, ruling B).* This
section described a `chain` cargo feature, an `OnChainReader` that cached the
finalised state behind a `ChainReader` view, and a read-only `ChainOps` seam.
All three are gone, with org-node's `subxt` and `on-chain-client`
dependencies; org-node's design ledger states the absence
(`docs/architecture/2026-10-08-values.md`).

- **Reads.** org-io reads the Organisation's state at the latest finalised
  block, parses it through `OrgState::from_chain`, and passes it as an
  `Option<OrgState>` to `commit_genesis`, `commit_update`, `reconcile`,
  `apply_receive`, `apply_self_delete` and `verify_envelope_against_chain`,
  all synchronous. A receive runs in phases: `receive_message` (transport),
  the chain-free `prepare_receive`/`prepare_self_delete`, which either
  finishes or names the Organisation whose state it needs, then org-io's one
  read, then the apply.
- **Writes.** org-node writes nothing to the chain. org-io submits each
  provisional update org-node built through `on-chain-client`'s writer, then
  hands org-node the state it read for the commit.

The chain checks of the operator preflight moved to org-io; org-node keeps a
transport-only `preflight` until S4.

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

Messages are typed as `WireMessage::OrgInformation { envelope, record_snapshot, org_private_key }`
(index 0: the committed Envelope, the record it extends, and the Organisation
private key the sending node's record holds) or `WireMessage::Revocation { envelope }`
(index 1: the Envelope alone). The kind follows the recipient: a Device the
sender's committed record lists receives Organisation information, any other
a revocation. No message carries an invite identifier.

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
via TLS). Authentication proves key custody, not membership. For an
Organisation the node holds, the chain-free phase refuses an update or a
revocation whose sender its current record does not list (`SenderNotListed`;
owner ruling R1, 2026-10-07, REQ-uk9rw7); what is then committed is decided by
the on-chain root at a newer epoch. *(Amended 2026-10-08: this said the
receive paths compare the key with nothing, the owner ruling of 2026-10-05
that R1 reversed.)*

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
