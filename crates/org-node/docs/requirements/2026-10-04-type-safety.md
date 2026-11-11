# org-node type-safety requirements

Decided 2026-10-04 in the `grill-requirements` interview that brings org-node
under the parse-at-the-system-edge rule (ADR
`docs/adr/2026-10-04-parse-at-the-system-edge.md`, `crates/org-node/AGENTS.md`).
The resolved problem report on secret key material in debug output recorded that the member seed, device seed and Organisation secret
were printed in clear by any `{:?}` of a structure holding them; REQ-hzm4kt
keeps them out of the store file only.

Owner rulings (2026-10-04):

- The secret types close the formatting exit only. Wiping secret memory on
  drop is left with the store-hardening residual of RC-jjsz97, because the
  plaintext also passes through buffers no newtype owns (the decrypted store,
  the encoded wire frame, the iroh secret key).
- Each secret has its own type (member seed, device seed, Organisation
  secret), so one cannot be passed where another is expected; the store
  encryption key is a fourth, private to the store.
- A secret type has a redacted `Debug`, no `Display`, no `Copy`; it is built
  infallibly from 32 bytes, gives them up only through `expose_secret`, keeps
  `PartialEq` (nothing compares a secret with untrusted input), and serializes
  as the plain bytes. Member and device seeds alone yield a signing key pair.
- `P2pMemberKey::parse` and `P2pDeviceKey::parse` accept exactly what
  deserialization accepted before (the bytes decompress to a curve point);
  rejecting weak and non-canonical keys was a problem in org-members'
  ledger, since resolved (a resolved org-members problem report); the non-strict signature check is the resolved problem report on non-strict signature verification.
- `org_pub_key` is the Organisation public key — the public half of a key pair
  shared with every Member — not an administrator key. It gets its own type,
  `OrgPublicKey`; that Envelopes are verified under it is the resolved problem report on the Organisation key held only by its founder.
- Chain accounts (`AccountId32`) get a tag type, `ChainAccount`, converted to
  subxt's type or raw bytes only where a chain call is built.
- Identifiers and counters get tag types: `PersonaId`, `Epoch`,
  `SequenceNumber`. The store passphrase stays plain: it arrives from the user.
- Stored, wire, blob and calldata bytes are pinned by a golden test written
  before the refactor and never edited afterwards.

## Secrets in formatted output

**REQ-y7tsft**: The software shall not include the bytes of a member seed, a
device seed, an Organisation private key or the store encryption key in the debug
formatting of that value or of any record, store or wire message that holds
it, and shall offer no display formatting for these values.
implements: RC-8a4xjb
satisfies: derived

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The item named the Organisation secret, which that change removes in favour of
the Organisation private key (REQ-szq3ud).

*Clarified 2026-10-08 (S4, task T3):* unchanged in substance; it covers
the device secret (`DeviceSecret`, `Debug` `DeviceSecret([REDACTED])`,
LLR-37k2s3) and the HKDF-derived store key (LLR-scgk5j).

## Persona details

Before this change `create_persona` stored the handle, name and surname as
typed, and an invalid handle was refused only when the administrator admitted
the Member, on another device; a member, device or Organisation public key held
in a store, a Join request or an Invite was never checked to be a curve point
until it was used. The owner ruled (2026-10-04) that the Persona holds the
parsed values, so the refusal moves to the Member's own device, and
(2026-10-05) that keys are held to the same rule: a Persona store, Join
request, Invite or record snapshot holding a handle, name, surname or key that
its parse refuses is refused when it is loaded, imported or decoded, with no
migration (no store has been deployed).
