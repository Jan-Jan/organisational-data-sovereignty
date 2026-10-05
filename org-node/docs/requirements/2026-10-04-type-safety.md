# org-node type-safety requirements

Decided 2026-10-04 in the `grill-requirements` interview that brings org-node
under the parse-at-the-system-edge rule (ADR
`docs/adr/2026-10-04-parse-at-the-system-edge.md`, `org-node/AGENTS.md`).
PR-hqwpg9 recorded that the member seed, device seed and Organisation secret
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
  rejecting weak and non-canonical keys is an open problem in org-members'
  ledger (`ed25519-small-order`); the non-strict signature check is PR-vkw22m.
- `org_pub_key` is the Organisation public key — the public half of a key pair
  shared with every Member — not an administrator key. It gets its own type,
  `OrgPublicKey`; that Envelopes are verified under it is PR-szkat6.
- Chain accounts (`AccountId32`) get a tag type, `ChainAccount`, converted to
  subxt's type or raw bytes only where a chain call is built.
- Identifiers and counters get tag types: `PersonaId`, `Epoch`,
  `SequenceNumber`. The store passphrase stays plain: it arrives from the user.
- Stored, wire, blob and calldata bytes are pinned by a golden test written
  before the refactor and never edited afterwards.

## Secrets in formatted output

**REQ-y7tsft**: The software shall not include the bytes of a member seed, a
device seed, an Organisation secret or the store encryption key in the debug
formatting of that value or of any record, store or wire message that holds
it, and shall offer no display formatting for these values.
(implements: RC-8a4xjb)
satisfies: derived

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

**REQ-qn2erx**: The software shall refuse to create a Persona, open a Persona
store, import a Join request, import an Invite, or decode the record snapshot
a first admission extends, whose handle, name or surname the member-record
rules would refuse or one of whose keys is not a curve point, reporting the
field that failed (an
Invite's refusal reports that the Invite failed to decode, without the
field's name), and shall create, store, import or extend nothing in that case.
(implements: RC-zutc67)
satisfies: derived

(Amended 2026-10-05 after independent review round 3: by owner ruling the
requirement covers keys as well as the handle, name and surname — a store, Join
request, Invite or record snapshot holding a key that is not a curve point is
refused — and names the Invite import and the record-snapshot decode, which
LLR-8bum44 already required and the code already refused, so the requirement
and its design item now state the same scope.)

(Amended 2026-10-05 after independent review round 4: by owner ruling the key
clause is narrowed to each key's curve-point parse. The rules on a member's
device key set — at most four keys, none repeated, at least one for a member
admitted through `MemberLeaf::new` — are not applied when a record is loaded or
decoded; org-members enforces them when the members are rebuilt
(`trie_from_snapshots`), reporting its own errors.)
