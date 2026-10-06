# Person: decisions and change sequencing

From the grill-requirements interview of 2026-10-04 (branch
`worktree-person-requirements`). Records what was settled for all three changes,
so that the two changes not written in this worktree start from the decisions
rather than from the conversation.

## Settled

- A **Person** is an individual in their own capacity. It is unlinkable to the
  same individual's memberships: separate keys even for the same device, and
  the link is known only to the individual. Members and Persons share code,
  never data.
- The `person` unit computes the hash of a Person definition and checks a given
  hash against a definition. Nothing on-chain: publishing, the hash's encoding
  version (stored beside the hash, chosen by the publisher's software;
  REQ-wg7z4s), the epoch compare-and-swap, the owning account (an entry keyed by
  the creating account, written only by it), and revoking every device from a
  foreign device with a hardware key go to a future `on-chain-person` unit.
- The hash is unsalted: only collaborators see the inputs, and they are meant
  to recognise the on-chain entry as the person's.
- `person` is class C and depends on no unit. org-members may depend on
  `person`, never the reverse, and only where the code is reasonably reused.
  `docs/adr/2026-10-04-parse-at-the-system-edge.md` forbids copying the
  validated types, so they move rather than duplicate.
- Key rules, the same for a Member's member-as-a-group key and a Person's
  PersonPublicKey (owner ruling), compatible with MLS TreeKEM and Keyhive
  BeeKEM:
  - zero DevicePublicKeys: no such key;
  - one or more: such a key, different from every DevicePublicKey;
  - DevicePublicKeys added, removed or replaced: the key changes;
  - DevicePublicKeys unchanged: the key may change (recovery rotation).
- The on-chain entry is `(hash, encoding version, epoch)`, with no separate
  key-type field (owner, 2026-10-04): the encoding version fixes the key types
  (v1: DevicePublicKeys ed25519, PersonPublicKey X25519), so a type field could
  only repeat the version or contradict it, and no reader can use a type
  without the version.
- Fee-payer linkability of a Person's on-chain account is a known limit for the
  PoC (hard-derived accounts unlink keys, not funding).

## Obligations the risk analysis leaves outside `person`

From `person/docs/risk/2026-10-06-person-hazards.md`
(none of these is releasable-blocking for change 2, all are for the Person
capability):

- `on-chain-person`: latest-finalised reads of a person's hash and encoding
  version (HAZ-vv2c3p) and publication of revocations (HAZ-zdwra7).
- The consuming application: never place one key in a Person definition and a
  Membership record; re-grant, on seeing a new definition, what was delegated
  under the previous PersonPublicKey; record the right on-chain id at first
  connection.
- `person` itself: a fuzz harness over arbitrary bytes (HAZ-h3swmw,
  HAZ-y6ft54). The target exists (`person/tests/fuzz_person_decode`, change 2);
  a recorded libfuzzer campaign does not.

## Changes, in order

1. **Shared types move into `person`.** *Merged: master f688a3c (the types in
   `person`), then the switch of org-members, on-chain-client, org-node and the
   app to them, master 5f7c177 (2026-10-06).* `Name`, `Surname`, the DevicePublicKey,
   device slots, the device sub-trie and the hasher trait move from org-members
   to `person`; org-members declares `depends_on: person` after the dependency
   assessment. Member hashes stay byte-identical (the org-specific sentinel and
   domain tags become parameters — done: each implementor of `person`'s
   `DeviceTrieHasher` supplies its own domain keys and `DEVICE_EMPTY_SENTINEL`,
   LLR-4vsm8d, LLR-6ezhw7); every existing org-members test, the Quint
   conformance gate and org-node's suite stay green. Behaviour-preserving: no
   new behaviour requirement, but `person` must define exported requirements
   for the validation it now owns, which org-members' items trace to.
2. **Person definition** (this worktree's requirements:
   `person/docs/requirements/2026-10-06-person-definition.md`).
   Merges with its implementation and tests, after change 1.
3. **Member key rules aligned.** The member-as-a-group key becomes optional;
   members follow the rules above. Supersedes REQ-ewdg2q (rotation on removal
   only) and REQ-r784fu (isolation replaces the key; it now removes it). Member
   record format and hashes change; the Quint spec, the conformance gate and
   org-node follow. The "every key held once" rule (DuplicateKey) is unchanged
   by the key rules. Closed (was this change's first open question): the
   member-as-a-group key no longer is an ed25519 `P2pMemberKey`; the switch
   (5f7c177) made it `person`'s `PersonPublicKey`, an X25519 key, so
   DuplicateKey's index already compares keys of two types, by bytes, with the
   two narrowings the owner accepted in
   `org-members/docs/risk/2026-10-03-key-uniqueness.md`. Own risk analysis. Updates the root glossary's
   Member-as-a-group key entry, which this change leaves as it is.
