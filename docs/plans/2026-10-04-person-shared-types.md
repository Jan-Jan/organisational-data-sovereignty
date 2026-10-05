# Shared identity types move into `person` — dependency assessment and order

Change 1 of the Person work (the sequencing and every earlier decision are in
`docs/plans/2026-10-04-person-sequencing.md` on branch
`worktree-person-requirements`, which merges after this change). This change
also contains the `person` unit's registration, cherry-picked from that branch.

**Merged in two parts (owner, 2026-10-04).** At the owner's instruction,
`person` is merged on its own first, from branch `worktree-person-unit`:
the unit, its identity types, their requirements, risk and design. The rest of
this plan — org-members, on-chain(-client) and org-node switching to those
types — follows on branch `worktree-worktree-person-shared-types`. Until it
merges, master holds `person`'s copies of `Name`, `Surname`, the device key,
the device slots and the device sub-trie beside org-members' originals: the
parallel copy `docs/adr/2026-10-04-parse-at-the-system-edge.md` forbids. The
owner accepted that interval explicitly (2026-10-04); it ends when
org-members switches, and nothing on master uses `person` in the meantime.

## Dependency assessment: org-members -> person

Walked with the owner on 2026-10-04 (grill-requirements, "Declaring a
dependency"). Provider `person`, consumer `org-members`, both class C, so the
class floor has nothing to convict and no `segregated_from:` entry is needed.

- **Export surface.** Empty on master. This change writes the exported
  requirements org-members relies on: REQ-r7mytp (names), REQ-q6xkna
  (DevicePublicKey), REQ-3vqs9b (PersonPublicKey), REQ-4szc22 (device slots),
  REQ-tq4ms4 (device slots read from outside the process), REQ-mu3qgz (device
  root per domain), REQ-aj6x3n (device root differs across domains),
  REQ-vxx8k3 (no panic), REQ-bczz87 (errors naming the rule broken), and
  REQ-7gz72r (the X25519 validity check, added later for org-node's
  organisation key).
- **Risk ledger.** Empty on master. The hazards of these types in org-members'
  use stay analysed in org-members' register (HAZ-bmv7cy, HAZ-8suua9); this
  change assesses its requirements in `person`'s ledger and re-examines
  org-members' register for the one behaviour that changes (below).
- **ADRs.** `docs/adr/2026-10-04-parse-at-the-system-edge.md` requires the move
  rather than a copy. No ADR forecloses it.
- **Open problem reports.** None in `person`.
- **SOUP.** Nothing new to the repository: `blake3`, `ed25519-dalek`,
  `curve25519-dalek`, `unicode-normalization`, `serde`, `postcard`,
  `thiserror`, all already resolved for org-members. `person`'s inventory
  lists them as its own.

Decision: declare `depends_on: person` in org-members' config once `person`
exports these requirements, in the change that switches org-members. org-node and the app take no edge: org-members
re-exports the moved types, and neither unit references `person`'s
requirements.

## Scope (owner, 2026-10-04)

Moves into `person`: `Name`, `Surname`, `MAX_NAME_LEN` and the NFC bound;
`P2pDeviceKey` as `DevicePublicKey`; `P2pDeviceSlots` as `DeviceSlots`, with
`MAX_DEVICES`; `NodeHash`; the device sub-trie; a `DeviceTrieHasher` trait
whose device-leaf, device-node and empty-slot sentinel are the implementor's;
and `P2pMemberKey` as `PersonPublicKey`.

Stays in org-members: `Handle`, `MemberId`, `MemberLeaf`, the trie, change
sets, member hashing. org-members' `TrieHasher` extends `DeviceTrieHasher`;
its `Blake3Hasher` keeps today's `org-members::device-*` keys and sentinel, so
member hashes are byte-identical. No deprecated aliases: there are no
adopters.

The one behaviour change: the member-as-a-group key, now a PersonPublicKey,
is validated as X25519 instead of ed25519. Hashes over the same bytes are
unchanged; which bytes are accepted changes (an ed25519 encoding with its top
bit set is not a canonical X25519 one), so key fixtures change.

Constructor names (owner, 2026-10-05): the owner chose `parse`, as the ADR's
Decision 2 states — `DevicePublicKey::parse`, `PersonPublicKey::parse` and
`DeviceSlots::parse`, with `TryFrom<VerifyingKey>` in place of
`DevicePublicKey::new`; the change branch worktree-worktree-person-shared-types,
which calls `from_bytes`/`new`, renames its call sites when it merges master.

## Owner rulings on keys and authority (2026-10-04, during T9 planning)

- `orgPubKey` (on-chain) is to be the organisation's X25519 key. Its only
  purpose is to let members verify that the organisation private key shared
  with them is the real one. It never represented admins.
- Keys used for key agreement (CGKA) determine on-the-wire encryption of
  documents; member, Person and organisation group keys are all X25519.
- Admins are to be identified by a `role` field on the member record
  (`MemberLeaf`), and authenticate — sign anything members must be able to
  attribute to an admin — with one of their own valid DevicePublicKeys. That
  is the only way members verify an admin's approval.
- On-chain updates by admins are authorised by on-chain accounts, not by the
  members trie.

Today's code authenticates change-set envelopes against `orgPubKey`
(`org-node/src/service.rs`, the two `VerifyingKey::from_bytes(&chain_state.org_pub_key)`
sites) and identifies the admin by `member key == orgPubKey`. Both are
replaced by the admin-authority change; until then they stay as they are.

## Order of the remaining work (owner, 2026-10-04, after T8)

1. Finish `person`: PersonPublicKey validated as X25519.
2. Update and migrate org-members (X25519 member keys; the admin `role`).
3. Update on-chain and on-chain-client (`orgPubKey` is the organisation's
   X25519 key).
4. Finally fix org-node (X25519 persona and organisation keys; change sets
   signed by an admin's own DevicePublicKey).

Steps 2–4 are merged together, after step 1 (`person`, merged alone at the
owner's instruction): a change to org-members runs org-node's gates
(`check-units.sh --impact`), so no step merges while org-node is red. The
branch may be red between steps; it merges green. Steps 2–4 each get a
requirements and risk pass before their tasks are planned.

## Order of work (owner, 2026-10-04)

1. Build the types in `person` first, test-first, without touching
   org-members.
2. Switch org-members (and org-node, app call sites) to them, delete the
   org-members originals, keep every member hash byte-identical.
3. X25519: if moving the tests' member-key fixtures to X25519 is trivial, do
   it directly; otherwise keep PersonPublicKey ed25519 until every suite is
   green, then port to X25519 test-first.

Written in step 2, when they become true, not before: `depends_on: person` in
org-members' config; amendment notes, in
`org-members/docs/architecture/2026-09-17-decomposition.md`, on LLR-w5nkbu,
LLR-pys2ek, LLR-kdhd2v and LLR-72p8bz naming `person` as the provider (org-members'
`TrieHasher` extending `person`'s device-trie trait, its device domains
unchanged); org-members' SOUP rows for `ed25519-dalek` and `serde`, which name
`P2pMemberKey` and `P2pDeviceKey`; and the mapping of `person`'s errors onto
org-members' existing variants of the same names, so org-members' reported
errors do not change.

Assessed in step 2, before it merges, against org-members' key-uniqueness rule
(every key held once, by byte comparison), as `person`'s risk file
(`person/docs/risk/2026-10-04-identity-types.md`) sets out:

- one keypair reused as a device key (ed25519 encoding) and a
  Member-as-a-group key (X25519 encoding) has different bytes in each role, so
  the byte comparison no longer catches it;
- a PersonPublicKey accepts mixed-order points, so one X25519 key has up to 8
  byte-distinct accepted encodings with the same X25519 output, and the byte
  comparison does not see them as one key.
