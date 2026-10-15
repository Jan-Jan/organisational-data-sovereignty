# Problems — open items from S4's independent review

Recorded 2026-10-14 from the independent review of change
`worktree-org-io-transport` (S4, round 1, finding-10). Both predate S4 and
are left open by it: this change records them and fixes neither.

**PR-6gf4t2**: `PersonaStore::data_mut` is `pub` (`org-node/src/store.rs`),
so any holder of a `PersonaStore` — a dependent crate included — can change
the store's data without a `StoreWrite`: nothing is sealed and nothing is
returned for the caller to write, so memory and the sealed bytes on disk can
differ with no write pending.
affects: LLR-bvt5up, SDD-asxwp8
opened: 2026-10-14
status: open

Where: `PersonaStore::data_mut` (`org-node/src/store.rs`, beside `data`).
Its only users are inside org-node: `OrgService::adopt` and the store
swap in `service.rs`, which seal the change themselves, and
`test_fixtures.rs`. LLR-bvt5up states that every `OrgService` operation that
changes a store returns exactly one `StoreWrite`; a store opened by
`PersonaStore::unseal` and changed through `data_mut` before or outside the
service is changed by no such operation. A crate-private `data_mut` (with a
`test-support` door for the fixtures, if they still need one) would close
it.

**PR-gq83cs**: `SigningKeypair` derives `Clone` and is re-exported publicly
(`org-node/src/keys.rs`, `pub use keys::{DeviceSecret, SigningKeypair}` in
`org-node/src/lib.rs`), so the device signing key pair built from a
`DeviceSecret` for one signature can be copied, and the copies outlive the
operation the device secret was lent for.
affects: REQ-26mjqa, SDD-fysk96
opened: 2026-10-14
status: open

Where: `#[derive(Clone, Debug)] pub struct SigningKeypair(SigningKey)`
(`org-node/src/keys.rs`). `DeviceSecret::signing_keypair` is crate-private
and no public operation returns a `SigningKeypair` from a secret
(LLR-37k2s3), so a dependent crate cannot reach the device's own key pair
through `DeviceSecret`; but inside org-node nothing stops a clone of it, and
outside it `SigningKeypair::generate` and `Clone` give dependents a cloneable
ed25519 key pair type. REQ-26mjqa requires the device secret to be used
within one operation and overwritten when it ends; a cloneable key pair
built from it does not uphold that by construction. Related: PR-4b2v6p
(`SigningKeypair` carries no role).
