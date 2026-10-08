# Design — org-node value types and the Persona's records

Low-level requirements for the change that brings org-node under the
parse-at-the-system-edge rule (ADR
`docs/adr/2026-10-04-parse-at-the-system-edge.md`). The ten defined here
belong to software items defined in `2026-10-03-decomposition.md`, each placed
under the item that owns most of the code it constrains:

- SDD-swtd3w, which owns `types.rs`: LLR-sz4xhc, LLR-s7whrn, LLR-ayrdr8 and
  LLR-mmdu38;
- SDD-sxp8hb, which owns `keys.rs`: LLR-56hc77;
- SDD-af5vnt, which owns `store.rs`: LLR-bwb9pu, LLR-scgk5j, LLR-g76zqd,
  LLR-q6n25z and LLR-8bum44.

Where a requirement also constrains code another item owns, that item names
it: SDD-pa6p7w names LLR-mmdu38; SDD-vee2fq names LLR-8bum44 and LLR-g76zqd;
SDD-rx2yvy and SDD-8cpyfa name LLR-8bum44; SDD-89es4z names LLR-g76zqd; and
SDD-kwncn7 and SDD-sxp8hb name LLR-bwb9pu. The notes are in
`2026-10-03-decomposition.md`, where those items are defined.

(Amended 2026-10-05 by the org-node type-safety change, review round 7: each
requirement now sits under the item owning most of what it constrains.
LLR-mmdu38 moved from SDD-pa6p7w to SDD-swtd3w, because four of its five
clauses constrain `OrgPublicKey` in `types.rs`. LLR-8bum44 moved from
SDD-vee2fq to SDD-af5vnt, because the store open, four of the five records it
names and `parse_field` are in `store.rs`. The requirements' IDs and text are
unchanged by the move.)

The amendments this change makes to items and low-level requirements defined
in `2026-10-03-decomposition.md` — `types.rs` added to SDD-swtd3w, the
`OrgStateCache` part of `chain_read.rs` moved from org-io's chain-connection item (moved 2026-10-07) to SDD-pa6p7w,
and the requirements that described the untyped API — are edited there, where
those items are defined, each marked "Amended 2026-10-05 by the org-node
type-safety change".
*(Amended 2026-10-08, ruling B, change `worktree-org-io-create`.)*
`chain_read.rs` is deleted: `OrgStateCache` and `OnChainReader` with it,
and `org_state_from_chain` moved to org-io, which reads the chain and hands
org-node the state as a value. org-node keeps the parse edge,
`OrgState::from_chain` in `chain.rs`. The mentions of `chain_read.rs`,
`SubxtChainOps` and `ChainOps` below record the change as it was made.

*Re-homed 2026-10-05 by owner ruling, at the merge of `master` `05f6f04`
(ratchet tooth 4).* This change was written when org-node had no design items,
and it defined two of its own — "org-node value types" and "the Persona's
records", minted as tokens `st9knt` and `zs2uyt` under the SDD prefix — with
the ten requirements below split between them. (The tokens are written here
without their prefix because both items are withdrawn and a prefixed
mention of a withdrawn item is a dangling reference.) Tooth 4 then gave org-node its decomposition of nineteen items,
which own every file these two named. The ten requirements keep their IDs and
their text and moved under the items above; the one change to a requirement's
text is LLR-ayrdr8's "every type of this item", which named the value-types
item and now names `types.rs`. Both items are withdrawn, and their tokens are
not reused. `types.rs`, new in
this change, has a fitting owner in tooth 4's items — SDD-swtd3w is "Value
types and the rejection vocabulary", and already owns `error.rs`, which this
change extends with `InvalidKey` and `InvalidField` — so no item of this
change's own is kept, and the decomposition's count of nineteen stands.

All items are class C, the unit's class.

## SDD-swtd3w — Value types and the rejection vocabulary

`org-node/src/types.rs` (new): the secrets (member seed, device seed,
Organisation secret), the Organisation public key and the tag types for the
chain account, the Persona identifier, the epoch and the Sequence number —
their construction, formatting and serialised form, and the Organisation
public key's parse together with what the chain read view serves when that
parse refuses a fetched state (LLR-mmdu38). The seed-to-key-pair conversion is
stated with the key pair it builds (LLR-56hc77, under SDD-sxp8hb).

**LLR-sz4xhc**: `MemberSeed` and `DeviceSeed` are each built
infallibly from 32 bytes by `From<[u8; 32]>`, are `Clone` and not `Copy`, keep
`PartialEq`/`Eq`, have no `Display`, and render under `Debug` as the type
name and `([REDACTED])` — `MemberSeed([REDACTED])` — whatever bytes they hold,
so no formatting gives their bytes up. Their bytes leave only through
`expose_secret(&self) -> &[u8; 32]` or through serialisation, which writes
exactly the 32 bytes they wrap, as `[u8; 32]` would be written, and reads back
only 32 bytes.
satisfies: REQ-y7tsft

(Amended 2026-10-05 by the org-node type-safety change, review round 7: this
said the bytes leave "only through `expose_secret`". The types derive
`Serialize` and `Deserialize` (`serde(transparent)`), by the owner ruling of
2026-10-04 recorded in `org-node/docs/requirements/2026-10-04-type-safety.md`
that a secret type "serializes as the plain bytes", so that the Organisation
secret can be kept in the store and carried in the Wire message.
`secret_types_serialise_as_the_plain_bytes` verifies that clause, which the
requirement did not state. REQ-y7tsft is about formatting only, and RC-8a4xjb
already records that serialising is not formatting.)

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This named `OrgSecret` as a third such type. The Organisation secret is
removed (LLR-qsjde3); the Organisation private key, which the store keeps and
the Wire message carries in its place, has the same properties as
`OrgPrivateKey` (LLR-322xfu).

**LLR-s7whrn**: `ChainAccount` (32 bytes), `PersonaId` (string), `Epoch`
(`u64`) and `SequenceNumber` (`u64`) are tag types: each is built infallibly by
`new` and by `From` of the value it wraps, holds exactly that value, and
returns it unchanged through its accessor; `ChainAccount` is opaque in
org-node — nothing in org-node converts it to subxt's account type, and its
bytes leave only through `as_bytes` for the app to hand to on-chain-client.
Under `Debug`,
`ChainAccount` renders as `ChainAccount(0x` followed by all 32 bytes in
lower-case hex and `)`, and the other three render as their derived `Debug`
does, the type name wrapping the value's own `Debug`: `PersonaId("p-alice")`,
`Epoch(3)`, `SequenceNumber(2)`.
satisfies: derived

(Amended 2026-10-05 by the org-node type-safety change, review round 8: the
`Debug` rendering, which the "Debug rendering" observable change below
describes, was stated by no requirement and checked by no test.
`ChainAccount`'s is hand-written; the other three are derived and kept so,
since a Persona identifier, an epoch and a Sequence number are not secret and
are short.)

(Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`):
this said `ChainAccount` "is converted to subxt's account type or to raw bytes
only inside `chain_write`". `chain_write/` moves to on-chain-client, and the
proxy account becomes opaque data org-node holds for the app (owner ruling 6),
so nothing in org-node converts it.)

**LLR-ayrdr8**: the postcard encoding of a fixed Persona store plaintext (one
Persona, one Organisation with its Organisation private key, a chain account
and members, one change-set provisional update holding the private key of its
Organisation key pair, and one expected admission naming an Organisation
alone), of a fixed Organisation-information Wire message, whose Envelope
carries no signature, and of a fixed revocation Wire message, which holds a
revocation notice (LLR-dc45ur) and no Envelope, are the
values pinned in `org-node/tests/encoding_golden.rs`: every type `types.rs`
defines, and the org-members and `person` types org-node holds, serialise
exactly as the plain values they replace. In the store plaintext, the
Organisation record has no Organisation-secret bytes after its epoch, and its
chain account is followed by the 32 bytes of its Organisation private key
with no option tag (LLR-byjvd9), then by its kept Change set option
(LLR-d9778a); the expected admission is the Organisation
identifier's bytes alone. Each Wire message begins with its variant index (00
for Organisation information, 01 for revocation), and has no invite
identifier bytes (LLR-ms8njy); the Organisation information has no signature
bytes after its Change set bytes (LLR-e7s4ye).
satisfies: derived

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, S3 T4;
LLR-js9dsu and LLR-dc45ur as amended there).* The pinned revocation Wire
message was `01` followed by the same Envelope as the Organisation
information. Index `01` now precedes a revocation notice: the Organisation
identifier, the Member identifier, the Device key and an org-members wire
absence proof, in that order. The new value was derived from the notice's
field order, not captured from the code; `encoding_golden.rs` records the
previous value beside it. The acknowledgement (index `02`, LLR-378cj4) is not
pinned here.

*Amended 2026-10-07 (owner ruling 12 of
docs/plans/2026-10-06-org-io-commit-workflow.md, change
`worktree-org-io-commit-workflow`, S3 T2).* The Organisation record now ends
with its kept Change set (LLR-d9778a) after its Organisation private key: an
option, `None` (the byte `00`) in the pinned store. The owner accepted the
layout change with a new golden pin and no migration; a store written before
it does not load. The new store value was derived from the previous one by
inserting that byte, not captured from the code; `encoding_golden.rs` records
the previous value beside it.

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This pinned an Organisation with an Organisation secret and no Organisation
private key (the byte `00` after the chain account being `org_private_key:
None`), an expected admission holding an invite identifier, and one admission
Wire message with an optional secret and invite identifier. The secret and the
invite identifier are removed, the record's key is required, and the Wire
message is an enum of two kinds (LLR-js9dsu); by the owner's later ruling
the same day a change-set provisional update holds a private key too
(LLR-e2b7gv). The pinned values are re-derived when the change is
implemented.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* The
pinned values were master's; two changed, each by one of the format changes
this item names. The two new values were derived from master's bytes, not
captured from the code; `encoding_golden.rs` records master's values beside
them.

(Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`):
this pinned also a pending Invite in the store, the Base64 text of a fixed
Invite and Join request, and the EVM calldata of a fixed genesis and update.
Invites, Join requests and their armour leave org-node for the app, and the
calldata leaves with the chain write for on-chain-client, which pins it there;
the store now holds provisional updates and expected admissions instead of
pending Invites, and the Envelope has no signature field. The golden values
are re-derived when the change is implemented.)

**LLR-mmdu38**: `OrgPublicKey::parse(&[u8; 32])` and `TryFrom<[u8; 32]>`
accept a value exactly when `person::x25519::is_valid_public_key` accepts it,
and refuse any other with `OrgNodeError::InvalidOrgPublicKey`. Its
`Deserialize` applies the same parse, and a value the parse refuses fails the
decode. No conversion builds one from a Member-as-a-group key: a node obtains
its own Organisation public key from its Organisation private key
(`X25519Keypair::org_public_key`, LLR-322xfu). `as_bytes` returns the 32 bytes
unchanged. Under `Debug` it renders as `OrgPublicKey(` followed by its first
four bytes in lower-case hex and `..)`, never its full 32 bytes. A chain state
whose Organisation public key the parse refuses leaves no cached state:
`OrgStateCache::store_fetched` clears the cached state and returns the error,
so `get_org_state` returns `Ok(None)` until a refresh succeeds.
satisfies: derived

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place: a
narrowing. The parse and `Debug` clauses stand unchanged. The fail-closed
cache clause (from "A chain state whose Organisation public key the parse
refuses leaves no cached state" to the end) is retired: `OrgStateCache` and
`OnChainReader` are deleted with `chain_read.rs`, and org-node caches no
chain state (PR-k2xxaq resolved by that deletion). The two parse cases of
the deleted `tests/chain_read_state.rs` moved to org-io with
`org_state_from_chain`; org-node's evidence for this item is
`node_value_types.rs`.

*Amended 2026-10-05 (owner answer Q4 of that day to
docs/plans/2026-10-05-switch-trim.md; written by change
`worktree-person-shared-types`).* This item stated the Edwards-point rule,
`InvalidKey` and `From<&P2pMemberKey>`, which its own round-5 amendment below
called interim until PR-szkat6 separated the two keys. This branch separates
them (REQ-ech45n), so the X25519 rule of LLR-3jjgtw applies and the conversion
is gone. The `Debug` and fail-closed cache clauses are this item's, unchanged.
A replacement item that stated this text was withdrawn before merge.

(Amended 2026-10-05 after independent review round 3: `refresh` returned on
the parse failure before updating its cache, so `get_org_state` went on
serving the state of the last successful refresh — a root and epoch the chain
had superseded. By owner ruling a state refused at parse now fails closed.)

(Amended 2026-10-05 after independent review round 5: the Edwards-point rule
is the interim rule, for what the field holds today — the published signing
key, the administrator's Member-as-a-group key (PR-szkat6). It conflicts with
the owner's ruling recorded in the `person` unit
(`person/docs/requirements/2026-10-04-identity-types.md`, 2026-10-04) that the
Organisation public key is an X25519 key belonging to org-node, to be checked
by `person`'s X25519 rule rather than a copy of it. When PR-szkat6 separates the
two keys, `OrgPublicKey` must take that rule: under the Edwards rule about half
of valid X25519 keys would be refused here, and small-order points the X25519
rule rejects would be accepted.)

(Amended 2026-10-05 by the org-node type-safety change, review round 7: moved
here from SDD-pa6p7w, which is named for its last clause. The `Ok(None)` it
serves after a refusal is the exception that org-io's absence-from-failure LLR (moved 2026-10-07), SDD-pa6p7w's split of
absence from failure, now states.)

(Amended 2026-10-05 by the org-node type-safety change, review round 8: the
`Debug` clause is added. The hand-written rendering, which the "Debug
rendering" observable change below describes, was stated by no requirement and
checked by no test.)

## SDD-sxp8hb — Device key and Member-as-a-group key custody

`org-node/src/keys.rs`: how a seed becomes the key pair this item holds, and
how a key pair hands its seed back.

**LLR-56hc77**: a key pair is obtained from a secret only through the secret's
own type, and hands its secret back only as that type, so no secret passes
through a plain byte array inside org-node. `MemberSeed::x25519_keypair` and
`OrgPrivateKey::x25519_keypair` yield an `X25519Keypair`, which hands its
secret back through `member_seed` or `org_private_key`.
`DeviceSeed::signing_keypair` yields a `SigningKeypair`, which hands its
secret back through `device_seed`. A key pair rebuilt from the secret it
handed back has the same public key. Neither key pair has a `from_seed` or a
`to_seed`, and `SigningKeypair` has no `member_seed`.
satisfies: derived

*Amended 2026-10-05 (owner answer Q4 of that day to
docs/plans/2026-10-05-switch-trim.md; written by change
`worktree-person-shared-types`).* This item named
`MemberSeed::signing_keypair` and `SigningKeypair::member_seed`. On this
branch a member seed is the secret of an X25519 key (LLR-ctzkv7), so neither
exists. The rule, a secret typed by its role at every step, is unchanged. A
replacement item that stated this text was withdrawn before merge.

## SDD-pa6p7w — The chain as a read oracle

`org-node/src/chain_read.rs` (`org_state_from_chain`, `OrgStateCache`): what
the read view accepts as an Organisation public key, and what it serves when
the chain's state is refused. Also constrained by LLR-mmdu38, under
SDD-swtd3w above. No requirement of this change sits under it.
*(Amended 2026-10-08, ruling B, change `worktree-org-io-create`: the
location is now `org-node/src/chain.rs`, `OrgState::from_chain`; the read,
`org_state_from_chain`, is org-io's, and the cache is deleted. SDD-pa6p7w
in `2026-10-03-decomposition.md` carries the amendment.)*

(Amended 2026-10-05 by the org-node type-safety change, review round 7: this
section named `org-node/src/types.rs` (`OrgPublicKey::parse`) as the item's.
The decomposition gives `types.rs` wholly to SDD-swtd3w.)

## SDD-af5vnt — The encrypted persona store

`org-node/src/store.rs`: the store encryption key, the records the store holds
— the Persona, Organisation, member snapshot and pending Invite records, each
field in its typed form and parsed when the store is opened — the Persona
details a Persona is created from, what the debug rendering of a value
holding a secret may show, and the field-naming refusal (`parse_field`) that
the store open, the Join request decode and the record-snapshot decode share.

**LLR-bwb9pu**: the debug rendering of every org-node value that holds a
secret contains none of that secret's bytes in any form. Those values are
`PersonaRecord`, `OrgRecord`, `StoreData`, `WireMessage`, `SigningKeypair` and
`X25519Keypair`. The records, the store plaintext and the Wire message (of
either kind) hold each secret in its secret type: `MemberSeed`, `DeviceSeed`
(LLR-sz4xhc) or `OrgPrivateKey` (LLR-322xfu). `SigningKeypair` holds its seed
in ed25519-dalek's `SigningKey`, whose `Debug` omits it (soup.md).
`X25519Keypair` renders as `X25519Keypair(..)` (LLR-98ufry). None of them
formats the bytes itself.
satisfies: REQ-y7tsft

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The secret types listed `OrgSecret`, which is removed (LLR-qsjde3). The Wire
message is now an enum of two kinds, and its `OrgInformation` holds the
Organisation private key as an `OrgPrivateKey` (LLR-ecxc76). REQ-y7tsft,
amended the same day, names the Organisation private key, so the parent note
below no longer puts the `OrgPrivateKey` clauses outside it.

*Amended 2026-10-05 (owner answer Q4 of that day to
docs/plans/2026-10-05-switch-trim.md; written by change
`worktree-person-shared-types`).* This item listed five values and called them
every value that holds a secret. This branch adds `X25519Keypair` and
`OrgPrivateKey`, so the list has six values and names the secret types they
hold. A replacement item that stated this text was withdrawn before merge.

*Parent checked 2026-10-05 by review round 3 (finding-8).* REQ-y7tsft names
the member seed, the device seed, the Organisation secret and the store key,
and the records, store and Wire message that hold them. It covers
`X25519Keypair` where the key pair holds a member seed. It does not name the
Organisation private key, so the clauses on `OrgPrivateKey` and on an
`X25519Keypair` holding it go beyond REQ-y7tsft. Those clauses are stated by
LLR-322xfu, LLR-98ufry and LLR-2dvhz8, all derived and assessed in
`org-node/docs/risk/2026-10-05-envelope-authenticity.md`. This item keeps
`satisfies: REQ-y7tsft` for the rest. *(Corrected 2026-10-05 by review round
4, finding-3: this note placed LLR-2dvhz8 under REQ-ech45n; it is derived.)*

(Amended 2026-10-04 after independent review: `SigningKeypair` holds a seed
too and derives `Debug`, relying on ed25519-dalek 2.2.0's `SigningKey` not
rendering its secret; it was missing from the list, and that reliance was
neither stated nor tested.)

**LLR-scgk5j**: the store encryption key is a `StoreKey`, private to
`store.rs`, returned by `derive_key`, with no `Display`, not `Copy`, and a
`Debug` rendering of `StoreKey([REDACTED])`.
satisfies: REQ-y7tsft

**LLR-g76zqd**: `PersonaRecord`, `MemberSnapshot` and `Joiner` hold the
handle, name and surname as `Handle`, `Name` and `Surname`, so a record or a
joiner is constructed only from parsed values, and `create_persona` takes
them typed; every other field of `PersonaRecord`, `OrgRecord`,
`MemberSnapshot`, `ProvisionalUpdate` and `Joiner` that is an array or counter
holds its typed form (`MemberSeed`, `DeviceSeed`, `OrgPrivateKey`,
`MemberId`, `RootHash`, `OrgPublicKey`, `P2pMemberKey`, `P2pDeviceKey`,
`ChainAccount`, `PersonaId`, `OrgId`, `Epoch`, `SequenceNumber`); `OrgRecord`
has no administrator key and no Organisation secret.
satisfies: REQ-qn2erx

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The typed forms listed `OrgSecret`, the type of `OrgRecord.org_secret`. Both
are removed (LLR-qsjde3, LLR-byjvd9); `OrgRecord.org_private_key` holds an
`OrgPrivateKey`.

(Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`):
this named `JoinRequest` and `PendingInvite`, and `OrgRecord`'s typed fields
included `admin_member_key`. The Join request and the pending Invite leave
org-node with the invitation exchange; the joiner's details reach
`admit_member` as a `Joiner`, and the store holds `ProvisionalUpdate`s; the
owner ruled that there is no administrator key anywhere in org-node.)

**LLR-q6n25z**: `PersonaDetails::parse(handle: &str, name: &str, surname:
&str) -> Result<PersonaDetails, OrgNodeError>` returns the three values parsed
by `Handle::parse`, `Name::parse` and `Surname::parse`, in that order, each
in the NFC form those parses produce — the form in which a Persona created
from them is stored, listed and exported in its Join request — and
refuses the first that does not parse with `OrgNodeError::InvalidField` whose
`field` is `persona.handle`, `persona.name` or `persona.surname`, so a caller
that parses through it before `OrgService::create_persona` creates nothing for
a refused detail; a Persona record loaded from a store parses its details
through it.
satisfies: REQ-qn2erx

(Amended 2026-10-04 after independent review round 2: the item did not say
that the parsed values are in NFC, although that is what is stored, listed
and exported for a Persona whose details were given in another normal form;
see "Observable changes".)

**LLR-8bum44**: opening a Persona store, or decoding a record snapshot
(`first_admission_base`), whose decoded content holds a handle, name, surname
or key that its type's parse refuses fails as a whole with
`OrgNodeError::InvalidField`. Its `field` names the field that failed
(`persona.*`, `org.*`, `member.*`, `provisional.*`), and nothing is opened or
extended. Each key is parsed by its own type: a Member-as-a-group key as a
`PersonPublicKey` (X25519), a DevicePublicKey as a `DevicePublicKey`, and an
Organisation public key as an `OrgPublicKey` (LLR-mmdu38). A member
snapshot's DevicePublicKeys are held as a list of parsed keys, so a repeated
key or more than four is not refused here; org-members refuses them, as a
`Trie` error, when the members are rebuilt.
satisfies: REQ-qn2erx

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said 'curve point' for every key; the keys now have three kinds, each
parsed by its own type. REQ-qn2erx still says "not a curve point", master's
wording; this item reads it as "not a valid key of its kind".

(Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`):
this also covered importing a Join request (`join_request.*`) and an Invite,
and named the pending Invite's fields (`pending_invite.*`). REQ-qn2erx was
amended by the same ruling to drop both imports, which move to the app's edge;
the store's provisional updates hold Organisation public keys and member
snapshots, parsed on open like the rest (`provisional.*`).)

(Amended 2026-10-04 after independent review: the record snapshot decoded by
`first_admission_base` and the Invite import were refused by the code but not
stated here, so nothing required them or traced their tests; both are now
part of the item, the error named as `InvalidField` rather than "an error
whose text names the field".)

(Amended 2026-10-05 after independent review round 4: "key" here is each key's
curve-point parse. A member snapshot's device key set is held as a list of
parsed `P2pDeviceKey`s, not as a `P2pDeviceSlots`, so a repeated key or more
than four keys is not refused here; org-members refuses them when the members
are rebuilt (`Trie(DuplicateDevice)`, `Trie(DeviceSlotsFull)`,
`Trie(EmptyDeviceList)`), by owner ruling narrowing REQ-qn2erx to match.)

(Amended 2026-10-05 by the org-node type-safety change, review round 7: moved
here from SDD-vee2fq, which is named for the Join request and Invite decode,
along with SDD-rx2yvy for the import operations and SDD-8cpyfa for
`first_admission_base`.)

## SDD-vee2fq — Out-of-band exchange blobs

`org-node/src/blobs.rs` (`JoinRequest`, `RawJoinRequest`,
`decode_join_request`): what is refused when a decoded Join request or Invite
does not parse. Also constrained by LLR-8bum44 and LLR-g76zqd, under
SDD-af5vnt above. No requirement of this change sits under it.

(Amended 2026-10-05 by the org-node type-safety change, review round 7: this
section claimed the store open and the record-snapshot decode, which are
`store.rs`'s and `first_admission_base`'s, and held LLR-8bum44. The
requirement moved to SDD-af5vnt.)

(Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`):
`blobs.rs` is removed with the invitation exchange, which moves to the app.
SDD-vee2fq now states that boundary (`2026-10-03-decomposition.md`); the
Join request decode this section named, and LLR-8bum44's and LLR-g76zqd's
clauses for it, are withdrawn there and here.)

### Observable changes

Recorded 2026-10-04, from the code as merged (T1–T5 of
`docs/plans/2026-10-04-org-node-type-safety.md`). Every value that was
accepted and used before is still accepted; these refusals and error values
change:

- **Persona creation** takes a parsed `Handle`, `Name` and `Surname`
  (`OrgService::create_persona`); an invalid one is refused where the Persona
  is created — the app's `create_persona` command parses the three through
  `PersonaDetails::parse` (LLR-q6n25z) and refuses with the field's name as
  prefix (`handle: …`, `name: …`, `surname: …`), creating nothing. It used to be stored, then refused at `create_organisation`
  (`Trie(InvalidHandle)` / `Trie(FieldTooLong)`) or on the administrator's
  device at admission.
- **Persona details in NFC** (added 2026-10-04 after independent review
  round 2): a handle, name or surname given in a non-NFC form is canonicalised
  to NFC when the Persona is created (`PersonaDetails::parse`, LLR-q6n25z), so
  what is stored, listed (the app's `PersonaDto`) and exported in a Join
  request changes for such input — `"jose\u{0301}"` becomes `"jos\u{e9}"`. It
  used to be stored and exported as given, and canonicalised only when it
  entered a member record (at `create_organisation`, or on the
  administrator's device at admission). A non-NFC Persona in an existing
  development store loads (its parse canonicalises it) and is rewritten in NFC
  on the store's next save. The admitted member record is unchanged.
  Join request import canonicalises the same way (extended 2026-10-05 after
  independent review round 3): `OrgService::import_join_request` parses each
  detail through `Handle::parse`, `Name::parse` and `Surname::parse`
  (LLR-8bum44), so a Join request whose details are not in NFC imports with
  them in NFC, and the app's `import_join_request` command shows the NFC text
  (`"jos\u{e9}"` for `"jose\u{0301}"`); it used to show them as received.
- **Persona store open** refuses a store holding an invalid handle, name,
  surname or key, as a whole, with `OrgNodeError::InvalidField { field, .. }`
  naming it (`persona.handle`, `persona.name`, `persona.surname`,
  `org.org_pub_key`, `org.admin_member_key`, `member.handle`, `member.name`,
  `member.surname`, `member.member_key`, `member.device_keys`,
  `pending_invite.admin_device_key`, `pending_invite.admin_member_key`,
  `pending_invite.org_pub_key`). Such a store used to open and fail later, at
  the trie rebuild (`Trie(..)` or `Chain("bad member key: …")` /
  `Chain("bad device key: …")`), at `create_organisation`, or at the Receive
  operation (`Chain("bad org_pub_key: …")`); a pending-Invite or
  administrator key only ever compared as bytes never failed and never matched.
  A store whose plaintext does not decode at all still reports
  `Chain("store decode: …")`.
- **Join request import** (`OrgService::import_join_request`,
  `blobs::decode_join_request`, and through them the app's
  `import_join_request` and `admit_member` commands) refuses an invalid
  handle, name, surname or key with `InvalidField` naming it
  (`join_request.handle`, `join_request.name`, `join_request.surname`,
  `join_request.member_key`, `join_request.device_key`); it used to import and
  be refused at admission (`Trie(..)` or `Chain("bad member key: …")` /
  `Chain("bad device key: …")`). Those two admission refusals no longer exist:
  `admit_member` takes the parsed Join request.
- **Record snapshot decode** (`first_admission_base`): an invalid field now
  reports `InvalidField` (`member.…`) instead of `Trie(InvalidHandle)`,
  `Trie(FieldTooLong)`, `Chain("bad member key: …")` or
  `Chain("bad device key: …")`. Still refused. The order in which two defects
  in one snapshot are reported changes with it: fields are now parsed in
  record order (handle, name, surname, member key, device keys), where the
  member key used to be checked first, so a snapshot with an invalid handle
  and an invalid member key reported `Chain("bad member key: …")` and now
  reports `InvalidField { field: "member.handle", .. }`.
- **Chain read**: an Organisation public key that is not a curve point is
  refused at the read — `InvalidKey` from `SubxtChainOps::read_state`
  (through `chain_read::org_state_from_chain`), its text ("invalid key: the
  bytes are not a point on the curve") from `OnChainReader::refresh` — instead
  of `Chain("bad org_pub_key: …")` at the Receive operation
  (`receive_and_verify`, `receive_and_self_delete_if_revoked`). The app
  classifies `InvalidKey` and `InvalidField` as Receive errors, the class
  `Chain` and `Trie` had, so the outcome the app shows is unchanged in kind.
  A refusal at `OnChainReader::refresh` also clears the reader's cached state
  (added 2026-10-05 after independent review round 3, LLR-mmdu38):
  `get_org_state` returns `Ok(None)`, and verify-against-chain through that
  reader refuses with `OrgNotOnChain`, until a refresh succeeds; it used to go
  on returning the state of the last successful refresh.
  `SubxtChainOps::read_state` caches nothing, so there is no such state there.
  (Amended 2026-10-05 by the org-node type-safety change, review round 7: this
  `Ok(None)` is for an Organisation that is on chain, so it is an exception to
  org-io's absence-from-failure LLR's (moved 2026-10-07) rule that `None` means no on-chain slot. org-io's absence-from-failure LLR (moved 2026-10-07) now states
  the exception.)
- **Invite import** (`OrgService::import_invite`) refuses an Invite whose
  Organisation public key, Member key or Device key is not a curve point
  (`Chain("blob decode: …")`, without the field's name); it used to be stored
  as a pending Invite.
- **Debug rendering** (added 2026-10-05 by the org-node type-safety change,
  review round 7): the `Debug` of `PersonaRecord`, `MemberSnapshot`,
  `JoinRequest`, `OrgRecord`, `PendingInvite`, `Invite` and `OrgState`
  changes with their field types, and so does the `Debug` of the values that
  hold them (`StoreData`, `WireMessage`, `ReceiveOutcome`,
  `VerifiedUpdate`). A handle, name or surname renders `Handle([REDACTED])`,
  `Name([REDACTED])` or `Surname([REDACTED])`. These are org-members' types,
  which redact them as personal data. A Member key, a Device key, a
  `MemberId`, a `RootHash` and an `OrgPublicKey` render as their type name
  and their first four bytes in hex, for example `P2pMemberKey(1a2b3c4d..)`
  (for `OrgPublicKey`, LLR-mmdu38). A chain account renders all 32 bytes,
  `ChainAccount(0x…)`, and the tag types render wrapped, for example
  `Epoch(3)` (LLR-s7whrn). The three secrets render
  `MemberSeed([REDACTED])`, `DeviceSeed([REDACTED])` and
  `OrgSecret([REDACTED])` (LLR-sz4xhc, LLR-bwb9pu), and the store key renders
  `StoreKey([REDACTED])` (LLR-scgk5j). Each of these fields used to render as
  the plain string, byte array or number, with the secrets in clear
  (PR-hqwpg9). Assessment: the secret half is REQ-y7tsft, and the rest
  redacts personal data and shortens identifiers, which supports the same
  aim. No operation's result changes. A diagnostic, log search or test that
  matched a full handle, key or root in `Debug` output no longer finds it.
  The values are still available through `Display` (handle, name, surname)
  and `as_bytes` (keys, roots).
  (Amended 2026-10-05 by the org-node type-safety change, review round 8:
  the `OrgPublicKey` and tag-type renderings now cite the requirements that
  state them, LLR-mmdu38 and LLR-s7whrn, which had no `Debug` clause.)
- **Interface**: every public signature a consumer of org-node absorbs,
  from `git diff master...HEAD -- org-node/src` (the app's commands still
  return the same text):
  - *Errors*: `OrgNodeError` gains `InvalidKey` and
    `InvalidField { field: &'static str, reason: String }`.
  - *New items*: the module `types` with `MemberSeed`, `DeviceSeed`,
    `OrgSecret`, `OrgPublicKey`, `ChainAccount`, `PersonaId`, `Epoch` and
    `SequenceNumber`, re-exported at the crate root;
    `store::PersonaDetails { handle, name, surname }` with
    `PersonaDetails::parse(&str, &str, &str) -> Result<PersonaDetails,
    OrgNodeError>` (LLR-q6n25z); `blobs::decode_join_request`;
    `chain_read::org_state_from_chain(on_chain_client::OrgState) ->
    Result<OrgState, OrgNodeError>` (was the private `map_state`, infallible);
    `chain_read::OrgStateCache` with `new(OrgId)`, `store_fetched(Option<
    on_chain_client::OrgState>) -> Result<(), String>` and `ChainReader`, the
    cache `OnChainReader` now holds (LLR-mmdu38);
    `chain_write::calldata::update_calldata(RootHash, OrgPublicKey, Epoch)`;
    `chain_write::proxy::org_id_of(ChainAccount) -> OrgId`.
  - *Re-exports*: `MemberId` and `RootHash` from org-members, at the crate
    root (`pub use org_members::{MemberId, RootHash}`), so the app's
    production code names them through org-node, the unit its `depends_on`
    lists, and org-members stays among the app's dev-dependencies only (for
    `OrgMembersError` in `tests/receiver_events.rs`). (Amended 2026-10-05
    after independent review round 6: the app had taken org-members as a
    production dependency for these two types.)
  - *Key pairs*: `SigningKeypair::from_seed` and `to_seed` are removed;
    `MemberSeed::signing_keypair`, `DeviceSeed::signing_keypair`,
    `SigningKeypair::member_seed` and `device_seed` replace them.
  - *`OrgService`*: `create_persona(rng, Handle, Name, Surname) ->
    PersonaId` (was `&str` ×3 → `String`); `create_organisation`,
    `export_join_request` and `ensure_endpoint` take `&PersonaId` (was
    `&str`); `admit_member` takes `org_secret: Option<OrgSecret>` and returns
    `MemberId` (was `Option<[u8; 32]>` → `[u8; 32]`); `revoke_member` takes
    `member_id: MemberId` (was `[u8; 32]`); `import_join_request` refuses
    with `InvalidField`.
  - *`ChainOps`* (and `MockChainOps`, `SubxtChainOps`): `submit_genesis(
    RootHash, OrgPublicKey) -> (OrgId, Option<ChainAccount>)` and
    `submit_update(OrgId, RootHash, OrgPublicKey, Epoch,
    Option<ChainAccount>)` (were `[u8; 32]`, `[u8; 32]`, `u64` and
    `Option<[u8; 32]>`); `SubxtChainOps::new(.., others: Vec<ChainAccount>)`
    and its public `others` / `proxy_map` fields hold `ChainAccount`.
  - *Chain write*: `revive_update_runtime_call(h160, RootHash, OrgPublicKey,
    Epoch)` and `submit::submit_update(.., RootHash, OrgPublicKey, Epoch)`
    (were `[u8; 32]`, `[u8; 32]`, `u128`); `ceremony::genesis_ceremony(..,
    others: &[ChainAccount], RootHash, OrgPublicKey)` and
    `GenesisOutcome.p: ChainAccount`; `multisig::multi_account_id(
    &[ChainAccount], u16) -> ChainAccount`, `dispatch_org_call(..,
    &[ChainAccount], ..)`, `fund(.., dest: ChainAccount)`;
    `proxy::create_pure(.., &[ChainAccount]) -> ChainAccount`,
    `proxy::proxied(ChainAccount, Value)`, `proxy::rotate(.., ChainAccount,
    .., &[ChainAccount], ChainAccount, ChainAccount)`.
  - *Records and messages*: `ReceiveOutcome.epoch: Epoch` and
    `.root: RootHash` (were `u64` and `[u8; 32]`); `OrgState.org_pub_key:
    OrgPublicKey` and `.epoch: Epoch`; `VerifyContext.last_committed_epoch`
    and `VerifiedUpdate.epoch` are `Epoch`; `SeqGuard::from_last_seen`,
    `last_seen`, `check` and `advance` take or return `SequenceNumber`;
    `SignedDeltaEnvelope.parent_seq` and the `parent_seq` argument of
    `SignedDeltaEnvelope::build` are `SequenceNumber`;
    `WireMessage.org_secret: Option<OrgSecret>`; `Invite.org_pub_key:
    OrgPublicKey`, `.admin_member_key: P2pMemberKey`, `.admin_device_key:
    P2pDeviceKey`; `JoinRequest.handle/.name/.surname` are `Handle`, `Name`,
    `Surname` and `.member_key/.device_key` `P2pMemberKey`/`P2pDeviceKey`;
    the fields of `PersonaRecord`, `OrgRecord`, `MemberSnapshot` and
    `PendingInvite` hold the types listed in LLR-g76zqd (all were `String`,
    `[u8; 32]` or `u64`).
  - *Test seams* (`test-support` only): `test_fixtures::ADMIN_DEVICE_SEED` is
    removed; `member_key`, `device_key`, the `Display`/`Copy` probes
    (`Probe`, `implements_display!`, `implements_copy!`),
    `store::seal_for_test`, `store::store_key_debug_for_test` and
    `store::store_key_traits_for_test` are added.
    `test_support::build_dispatch_tx` takes `&[ChainAccount]` (was
    `&[[u8; 32]]`), and the probes' four traits, `ImplementsDisplay`,
    `LacksDisplay`, `ImplementsCopy` and `LacksCopy`, are public in
    `test_fixtures`. (Amended 2026-10-05 by the org-node type-safety change,
    review round 7, which found the first missing. This list was re-derived
    from the public signatures in `git diff master...HEAD -- org-node/src`,
    and that pass found the traits missing too. No other omission was found.)

  org-members gains `OrgMembersError::InvalidKey`, and `P2pDeviceSlots::new`
  is renamed `parse` (recorded in org-members' design ledger).

Design note: a record with a fallible field decodes through a crate-private
`Raw…` mirror and `TryFrom`, declared as `#[serde(try_from)]`, because postcard
discards the message of an error raised inside `Deserialize`; one parse serves
the typed decode and the field-naming load. A decode that goes through the
typed record's own `Deserialize` (the generic `blobs::decode::<JoinRequest>`, or
a `StoreData` decoded directly) still refuses the same values, but as a
`Chain("… decode: …")` without the field's name.
