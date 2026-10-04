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
- **Reuse org-members' types** (`Handle`, `Name`, `Surname`, `MemberId`,
  `P2pMemberKey`, `P2pDeviceKey`, `RootHash`). Never re-implement their
  validation here; parse with their `parse`/`TryFrom`.
- **Validated type** (has an invariant): private field, `parse` + `TryFrom`
  delegating to it, `type Error = OrgNodeError`. **Tag type** (any value is
  valid, e.g. `OrgId`): infallible `new` + `From`, no `Result`.
- **Serde parses too.** A persisted/wire struct field is the newtype; its
  `Deserialize` goes through `parse`. Wrapping must not change stored or wire
  bytes -- prove it with a round-trip test against bytes from before the change.
- **Secrets** (member/device seed, Organisation secret) get a newtype with
  redacted `Debug` and no `Display`. Never `derive(Debug)` on a struct that
  holds a secret as a plain array (PR-hqwpg9).

**Status (2026-10-04):** org-node does not yet follow this rule; 82 lines
under `src/` mention `[u8; 32]`: `service.rs` 27, `store.rs` 15,
`chain_write/proxy.rs` 12, `chain_write/multisig.rs` 7, `blobs.rs` 5,
`ceremony.rs` 4, `chain_write/calldata.rs` 4, `keys.rs` 2,
`chain_write/submit.rs` 2, and one each in `chain.rs`, `transport/wire.rs`,
`test_fixtures.rs`, `bin/preflight.rs` (counted with
`grep -c '\[u8; 32\]'`). Some are legitimate edge points (chain and wire
decoding, calldata encoding); the rest are converted in a dedicated follow-up
change. Don't add new ones.

```rust
// Edge: a decoded JoinRequest is parsed once into domain types...
let handle = Handle::parse(&req.handle)?; // OrgMembersError -> OrgNodeError via From
let member_key = VerifyingKey::from_bytes(&req.member_key)  // bytes -> curve point
    .map(P2pMemberKey::new)
    .map_err(|e| OrgNodeError::Chain(format!("bad member key: {e}")))?;

// ...and everything past the edge takes the newtype, never &str / [u8; 32].
fn admit(&mut self, handle: Handle, name: Name, surname: Surname,
         member_key: P2pMemberKey, device: P2pDeviceKey) -> Result<(), OrgNodeError>;

// Secret: no Display, redacted Debug.
pub struct Seed([u8; 32]);
impl fmt::Debug for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Seed([REDACTED])")
    }
}
```
