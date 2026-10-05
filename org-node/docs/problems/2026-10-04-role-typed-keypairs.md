# Problem reports — signing key pairs carry no Member or device role

**PR-4b2v6p**: `SigningKeypair` (`org-node/src/keys.rs`) carries no role, so a
Member key pair can be passed where a device key pair is meant, or the reverse
(`OrgEndpoint::bind` / `bind_with_mode`, `SignedDeltaEnvelope::build`), and
`member_kp.device_seed()` compiles.
affects: LLR-56hc77
opened: 2026-10-04
status: open

Found 2026-10-04 by the independent review of the org-node type-safety change.
LLR-56hc77 types the seeds — a `MemberSeed` and a `DeviceSeed` cannot be
swapped, and no seed passes through a plain byte array — but the key pair a
seed becomes is one type for both roles, and `SigningKeypair::member_seed()`
and `device_seed()` are both public, so the role a seed was given is lost the
moment it becomes a key pair. Nothing in the code swaps them today: every call
site derives the key pair from the seed type of the role it needs. A swap
would present the wrong identity — bind the transport under the Member key, or
sign an Envelope with a device key — and only review and the call sites' tests
stand against it; the risk assessment of LLR-56hc77 records it as a residual. By owner ruling
(2026-10-04) role-typed key pairs (a `MemberKeypair` and a `DeviceKeypair`,
each giving up only its own seed type) are a follow-up change, not part of
the type-safety change.
