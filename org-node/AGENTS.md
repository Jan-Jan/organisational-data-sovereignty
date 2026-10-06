# AGENTS.md (org-node)

Guidance specific to the `org-node` crate. See `../AGENTS.md` for
project-wide guidance and `../org-members/AGENTS.md` for the library it builds
on.

## Type safety: parse at the system's edge (hard rule)

Same rule as org-members; why: `../docs/adr/2026-10-04-parse-at-the-system-edge.md`.

- **Plain types (`&str`, `String`, `[u8; 32]`) only where data enters the
  system**, parsed there, once. org-node's edges: app/Tauri command inputs,
  persona-store decoding (`store.rs`), invite/join blobs (`blobs.rs`),
  transport frames (`transport/wire.rs`), chain reads (`chain_read.rs`).
  Everywhere else -- every signature, struct field and return -- uses the
  newtype.
- **Reuse org-members' types** (`Handle`, `MemberId`, `RootHash`, and
  `person`'s `Name`, `Surname`, `PersonPublicKey`, `DevicePublicKey` it
  re-exports). Never re-implement their validation here; parse with their
  `parse`/`TryFrom`. org-node's own: `OrgPublicKey` (`types.rs`), parsed
  through `person::x25519::is_valid_public_key`.
- **Validated type** (has an invariant): private field, `parse` + `TryFrom`
  delegating to it, `type Error = OrgNodeError`. **Tag type** (any value is
  valid, e.g. `OrgId`): infallible `new` + `From`, no `Result`.
- **Serde parses too.** A persisted/wire struct field is the newtype; its
  `Deserialize` goes through `parse`. Wrapping must not change stored or wire
  bytes -- prove it with a round-trip test against bytes from before the change.
- **Secrets** (member/device seed, Organisation secret, Organisation private
  key, store key) get a newtype with redacted `Debug` and no `Display`. An
  X25519 secret in use is an `X25519Keypair` (not `Clone`, wiped on drop). Never `derive(Debug)` on a
  struct that holds a secret as a plain array (PR-hqwpg9).

**Status (2026-10-04, re-counted 2026-10-05 after the merge of master
`05f6f04`):** org-node follows this rule since the org-node
type-safety change (`docs/plans/2026-10-04-org-node-type-safety.md`). Its
design sits under the owning items of org-node's decomposition — value types
in `types.rs` (including the Organisation public key's parse) under
SDD-swtd3w, seeds to key pairs under SDD-sxp8hb, the store's records and the
refusals on load and import under SDD-af5vnt, each cross-referenced from the
other items it constrains (`docs/architecture/2026-10-04-type-safety.md`). 39 lines
under `src/` still mention `[u8; 32]` (counted with `grep -c '\[u8; 32\]'`,
re-counted 2026-10-05 by review round 3 on `worktree-person-shared-types`),
each an edge, a type's own constructor/accessor, or a buffer:
`types.rs` 15 (the newtypes' constructors, accessors and `Deserialize`),
`store.rs` 9 (the `Raw…` decode mirrors and the private `StoreKey`),
`keys.rs` 2 (`X25519Keypair`'s secret and its `public_bytes`),
`chain.rs` 1 (`OrgState::from_chain`, the chain-read edge),
`test_fixtures.rs` 2 (fixture device seeds, test support only),
`chain_write/proxy.rs` 2 (`BlockSink::settle`, extrinsic-event decoding),
`chain_write/multisig.rs` 2 (signer sort buffer, `blake2_256` output),
`chain_write/calldata.rs` 2 (`build_update_calldata`, the pinned EVM encoder),
`blobs.rs` 2 (the `RawJoinRequest` mirror), `service.rs` 1
(`FinalitySink::settle`, the live-chain `BlockSink`), `bin/preflight.rs` 1
(env parsing). Don't add new
ones outside an edge.

```rust
// Edge: a decoded JoinRequest is parsed once, each field named on refusal...
let jr = blobs::decode_join_request(blob)?; // OrgNodeError::InvalidField { field: "join_request.member_key", .. }
                                            // (person's PersonPublicKey::parse refused it)

// ...and everything past the edge takes the newtype, never &str / [u8; 32].
fn admit_member(&mut self, rng: &mut R, org_id: OrgId, join_request: &JoinRequest,
                peer_addr: EndpointAddr, org_secret: Option<OrgSecret>) -> Result<MemberId, OrgNodeError>;

// Secret: no Display, redacted Debug, bytes only through expose_secret.
let member = persona.member_seed.x25519_keypair(); // X25519Keypair: not Clone, wiped on drop
let device = persona.device_seed.signing_keypair();
```

A record with a fallible field decodes through a `Raw…` mirror and `TryFrom`
(`#[serde(try_from)]`), because postcard drops the message of an error raised
inside `Deserialize`.
