# Design — the Organisation key pair reaches every Member

Low-level requirements for change `worktree-org-node-org-key-pair`, change 3
of the chain-authority sequence. They refine the owner rulings and the
requirements of
`org-node/docs/requirements/2026-10-07-org-key-pair.md`
(REQ-szq3ud, REQ-stx9v3, REQ-jy6ybw, REQ-c29s93, REQ-bwx7eg, REQ-ju6vn2,
REQ-3dsweu, REQ-vxqc5g), and
REQ-8amu2a as amended in `2026-10-06-chain-authority.md`. They realise the
receipt check RC-9cefcn
(`org-node/docs/risk/2026-10-07-org-key-pair.md`).

No software item is added; the decomposition's count of nineteen stands. Each
requirement below sits under the item that owns the code it constrains, as
`2026-10-06-chain-authority.md` does, and each owning item's `traces:` line is
amended in place in `2026-10-03-decomposition.md` with a dated note:

- SDD-kwncn7, which owns `transport/wire.rs`: LLR-js9dsu and LLR-ecxc76;
- SDD-swtd3w, which owns `error.rs` and `types.rs`: LLR-j5vbqj and LLR-qsjde3;
- SDD-af5vnt, which owns `store.rs`: LLR-byjvd9;
- SDD-rx2yvy, which owns `admit_member`, `keep_change_set` and
  `send_update`: LLR-6ymd6d and LLR-e2b7gv. LLR-e2b7gv also constrains
  `revoke_member`, SDD-72ddm6's;
- SDD-8cpyfa, which owns the receive-and-commit path and `commit_update`:
  LLR-xn5pwc, LLR-ba2ejp, LLR-38e2kn, LLR-pt32fx, LLR-6s785x and LLR-4kh9w9.
  LLR-ba2ejp, LLR-pt32fx and LLR-4kh9w9 also constrain
  `receive_and_self_delete_if_revoked`, SDD-72ddm6's.

Every item whose text the rulings falsify is amended in place, in the file
that defines it, with a dated note naming this change. All items are class C,
the unit's class. No dependency changes, so `soup.md` is unchanged.

## The shape of the change, for an implementer

- **Wire message.** `WireMessage` becomes an enum of two kinds.
  `OrgInformation { envelope, record_snapshot, org_private_key }` always
  carries the record snapshot and the Organisation private key; a body of that
  kind without either does not decode. `Revocation { envelope }` carries
  neither. No kind carries an invite identifier.
- **No Organisation secret.** `OrgSecret`, `OrgRecord.org_secret` and
  `InviteId` are removed. `OrgRecord.org_private_key` is a required
  `OrgPrivateKey`: the creator's record holds it from `commit_genesis`, a
  joiner's from its first admission. Stores written before this change are
  not supported (owner ruling) and nothing migrates them.
- **Sending.** `send_update(outgoing, recipient, peer_addr)` takes no secret,
  key or invite identifier. It sends Organisation information, carrying the
  Organisation private key of the epoch the update reaches as the node's
  record holds it, to a Device the node's committed record lists, and a
  revocation to any other Device. There is one Organisation private key per
  epoch, shared Organisation-wide; no node has a key of its own. A node draws
  a key pair only when it builds a provisional update and passes on the key
  its record holds.
- **Rotation** (owner ruling of 2026-10-06, later the same day, superseding
  the earlier ruling that rotation is out of scope). A provisional update is a
  batch of changes; when its resulting Membership root is calculated, the
  building node draws a fresh X25519 Organisation key pair. The private key is
  kept in the provisional update (`ProvisionalChange::ChangeSet` gains
  `org_private_key`, as `Genesis` already has), and the public key is the
  update's `org_pub_key`, which the app writes to the chain. Today
  `admit_member` and `revoke_member` are each a batch of one. When
  `commit_update` commits the update, the record's key and Organisation public
  key become the update's; the record keeps only the current key, no history.
  A provisional update is identified by its key as well as its root, so
  rebuilding the same change never replaces an update whose key may already
  be on the chain. Nothing about the kind is kept
  in a provisional update or an outgoing update.
- **Receiving.** An Organisation-information message is checked as before
  (chain-free checks, one chain read, verification) and then the X25519
  public half of the Organisation private key it carries is compared with the
  `org_pub_key` read from the chain at verification; a mismatch is
  refused with `OrgKeyMismatch`. On commit the received key is stored. A
  revocation about an Organisation the node holds no record of is refused
  before the chain is read; with a record it is verified as any update and
  then accepted only if it removes this node's own Device, in which case the
  node deletes its record (self-delete). A revocation after which the node's
  Device is still listed is refused with `RevocationNotForThisDevice`, store
  unchanged (owner ruling on relabelling), so a revocation never commits into
  a record the node keeps. A first admission is matched against the expectations
  by Organisation alone: `expect_admission(rng, org_id)`, and
  `ExpectedAdmission { org_id }`.
- **Commit paths.** `commit_held` takes no secret. `commit_update` replaces
  the record's key with its provisional update's; `commit_genesis` stores the
  genesis update's key, as before. A committed Organisation-information
  message sets the record's key and Organisation public key to the received
  key and the chain's key. No revocation commits into a kept record.

The types this change alters:

    WireMessage::OrgInformation { envelope: Envelope, record_snapshot: Vec<u8>, org_private_key: OrgPrivateKey }
    WireMessage::Revocation { envelope: Envelope }
    OrgRecord { org_id, root_hash, org_pub_key, epoch, last_seq, trie_members, proxy_account, org_private_key: OrgPrivateKey }
    ProvisionalChange::ChangeSet { change_set: Vec<u8>, org_private_key: OrgPrivateKey }
    ExpectedAdmission { org_id: OrgId }
    OrgNodeError::{ MalformedMessage, OrgKeyMismatch { org_id }, RevocationNotHeld { org_id },
                    RevocationNotForThisDevice { org_id } }

## SDD-kwncn7 — The wire frame and its bound

`org-node/src/transport/wire.rs`.

**LLR-js9dsu**: `WireMessage` is an enum with exactly three variants, encoded
by postcard as the variant index followed by the variant's fields in order:
`OrgInformation { envelope: Envelope, record_snapshot: Vec<u8>,
org_private_key: OrgPrivateKey }` (index 0), `Revocation(RevocationNotice)`
(index 1, LLR-dc45ur) and `Acknowledgement(Acknowledgement)` (index 2,
LLR-378cj4). No field of any is optional, and none carries an invite
identifier or any other secret. `WireMessage::org_id(&self) -> OrgId` returns
the Organisation identifier of any variant; there is no accessor for an
Envelope, which only `OrgInformation` holds. `decode_body` refuses with
`Malformed`, and does not panic, a body whose variant index is not 0, 1 or 2,
an `OrgInformation` body that ends before its record snapshot or before
the 32 bytes of its Organisation private key, and a body of any of the three
kinds with bytes left over after its last field.
satisfies: REQ-c29s93, REQ-3dsweu
superseded-by: LLR-tajh9d

*Superseded 2026-10-08 (change worktree-org-io-create; owner's ruling on
PR-zf924s, hybrid: a change of meaning mints a new ID).* The first amendment
below changed this item's meaning under the same ID: from two message kinds
to three, and a revocation payload from an Envelope to an absence proof.
LLR-tajh9d (`2026-10-08-wire-kinds.md`) states the rule
that holds now; the text above is kept as it stood when superseded. Its tests
stay annotated with this ID, and the new test of LLR-tajh9d names both.

*Note 2026-10-08 (review round 1 of change worktree-org-io-create,
finding-13: dual-ID coverage).* These older tests still name only this ID:
`org-node/tests/encoding_golden.rs`'s
`organisation_information_wire_message_is_pinned` (:191) and
`revocation_wire_message_is_pinned` (:217); `wire_frame_bound.rs`'s
`the_three_kinds_round_trip_with_their_indices` (:81),
`malformed_revocations_and_acknowledgements_are_refused` (:100),
`an_org_information_body_with_trailing_bytes_is_refused` (:130) and
`an_organisation_information_body_without_its_snapshot_or_key_does_not_decode`
(:224); and the fuzz target `fuzz_wire_decode/fuzz_target.rs` (:16). They are
already green, and the robustness rule forbids re-annotating a green test
with a new ID: a test gains a new ID only by being watched red against it.
The dual-ID coverage of the supersession is therefore the new red-first test
`wire_frame_bound.rs`'s
`exactly_three_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused`
(:146), which names LLR-tajh9d and LLR-js9dsu both.

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3;
REQ-3dsweu as amended there, REQ-ps2gy2).* This had two variants, the second
`Revocation { envelope: Envelope }`, and an accessor
`WireMessage::envelope()` for either. The revocation now holds a
`RevocationNotice` (the revoked Device's identity and an absence proof, no
Envelope), and the acknowledgement a revoked Device signs is the third
variant (`org-node/docs/architecture/2026-10-07-commit-workflow.md`).

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, review
finding-3 of round 1).* `decode_body` refuses leftover bytes after the
message for every kind (`postcard::take_from_bytes`), since S3 T4; this LLR
did not say so for `OrgInformation`, and LLR-dc45ur and LLR-378cj4 state it
only for the other two kinds. The last clause of the refusals ("a body of any
of the three kinds with bytes left over after its last field") is added; the
behaviour is unchanged.

**LLR-ecxc76**: the `Debug` rendering of a `WireMessage` of either variant
contains none of the bytes of the Organisation private key it holds, in any
form: an `OrgInformation` renders its key as `OrgPrivateKey([REDACTED])`
(LLR-322xfu), and a `Revocation` holds no key. `WireMessage` formats no field
itself.
satisfies: REQ-y7tsft

## SDD-swtd3w — Value types and the rejection vocabulary

`org-node/src/error.rs`, `org-node/src/types.rs`.

**LLR-j5vbqj**: `OrgNodeError` gains four variants, each distinct from every
other variant: `MalformedMessage`, whose `Display` is "received wire message
is malformed", for a received Wire message that does not decode
(LLR-xn5pwc); `OrgKeyMismatch { org_id: OrgId }`, whose `Display` names the
Organisation, for an Organisation-information message whose key's public half
is not the chain's Organisation public key (LLR-ba2ejp);
`RevocationNotHeld { org_id: OrgId }`, whose `Display` names the
Organisation, for a revocation about an Organisation the node holds no record
of (LLR-38e2kn); and `RevocationNotForThisDevice { org_id: OrgId }`, whose
`Display` names the Organisation, for a revocation whose MemberId and
DevicePublicKey are not those of a Persona bound to that Organisation
(LLR-r7zm39).
satisfies: REQ-c29s93, REQ-bwx7eg, REQ-vxqc5g, REQ-3dsweu, REQ-qrtsc9

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3).*
`RevocationNotForThisDevice` was "for a revocation after whose verified
Membership record this node's Device is still listed". A revocation is no
longer verified as an update: it names one Device, and is refused with this
variant, before any chain state is used, when that Device is not one of this
node's Personas (REQ-qrtsc9). A revocation for this node's Device that the
chain's record still lists fails its absence proof and is refused with
`RevocationProofRefused` (LLR-tx8ruv, REQ-m2xh8q).

**LLR-qsjde3**: org-node defines no `OrgSecret` type and re-exports none: no
record, store plaintext, Wire message, provisional update or `OrgService`
operation holds, takes or returns an Organisation secret, and the only
Organisation key material org-node holds is the `OrgPrivateKey`.
satisfies: REQ-szq3ud

## SDD-af5vnt — The encrypted persona store

`org-node/src/store.rs`.

**LLR-byjvd9**: `OrgRecord` (and its decode mirror `RawOrgRecord`) has no
`org_secret` field, and its `org_private_key` is an `OrgPrivateKey`, not an
`Option`, encoded as its plain 32 bytes with no option tag: every record a
store holds carries the Organisation's private key, the one value the whole
Organisation shares. `commit_genesis` fills it
from the genesis provisional update (LLR-wzqqg9, LLR-qjz3q4), a first
admission from the Organisation-information message it commits (LLR-ckk5nz),
and no other operation creates a record; `commit_update` and a committed
Organisation-information message replace it (LLR-6s785x, LLR-ckk5nz). The
record holds only the current key and keeps no earlier one. Stores written before this change are
not supported: nothing reads or migrates their layout.
satisfies: REQ-ju6vn2, REQ-ech45n

## SDD-rx2yvy — Admission

`org-node/src/service.rs` (`keep_change_set`, which `admit_member` and
`revoke_member` share, and `send_update`).

**LLR-e2b7gv**: `admit_member` and `revoke_member`, each a batch of one
change, build their provisional update through `keep_change_set(rng, rec,
persona_id, new_trie, delta)`, which, after the resulting Membership root has
been calculated, draws a fresh Organisation key pair from `rng`
(`X25519Keypair` from 32 random bytes, as `create_organisation` does) and
stores the update with `org_pub_key` that pair's public key and
`ProvisionalChange::ChangeSet { change_set, org_private_key }` holding its
private key. Before storing, it refuses with
`Trie(OrgMembersError::DuplicateKey)` a public key equal to the record's
current `org_pub_key` or to any Member-as-a-group key or DevicePublicKey of
the resulting record (`OrgPublicKey::ensure_distinct_from`), keeping no
provisional update, leaving the record unchanged and writing nothing. The
record's `org_pub_key` and `org_private_key` are not changed by building the
update.
satisfies: REQ-stx9v3

**LLR-6ymd6d**: `send_update(outcome: &CommitOutcome, recipient, peer_addr)`
looks up the node's record of `outcome.org_id`, refusing with `OrgNotOnChain`
and sending nothing when it holds none, and chooses the kind from that record
and the outcome alone: when a member snapshot of the record lists `recipient`
among its DevicePublicKeys it sends `WireMessage::OrgInformation` holding
`outcome.outgoing.envelope`, `outcome.outgoing.record_snapshot` and the
Organisation private key of the epoch the update reaches, as that record's
`org_private_key` holds it (a clone; nothing is drawn); otherwise, when
`outcome.revocations` holds a notice whose `device` is `recipient`, it sends
`WireMessage::Revocation` holding that notice; otherwise it refuses with
`NoRevocationForRecipient { org_id }` and sends nothing. Neither
`ProvisionalUpdate` nor `OutgoingUpdate` holds a kind, a key or a recipient,
and the kind does not depend on which operation built the update.
satisfies: REQ-3dsweu, REQ-szq3ud

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3;
REQ-3dsweu as amended there).* This took the `OutgoingUpdate` and sent any
Device the record does not list `WireMessage::Revocation` holding the
committed Envelope. The revoked Device now receives only its notice
(LLR-a8z7r5, REQ-ps2gy2), which `send_update` takes from the outcome, and a
Device neither record lists receives nothing. Until stages S2 and S4 move the
send into org-io, org-node still sends; after S4, org-io sends the same
messages and this item's kind rule moves with the send (S4's plan).

The record consulted is the one the node holds when `send_update` runs, which
is the committed record the outgoing update came from unless a later commit
has replaced it. Either way the key sent is the one that record holds, and it
never goes to a Device the node's current record does not list: a recipient a
later commit removed is sent a revocation, and one a later commit admitted is
sent the current key, which it may hold.

## SDD-8cpyfa — The receive-and-commit path

`org-node/src/service.rs` (`receive_one`, `receive_and_verify`,
`receive_and_self_delete_if_revoked`, `commit_update`).

**LLR-6s785x**: `commit_update(rng, org_id)` selects the stored provisional
update for `org_id` whose `resulting_root` and `org_pub_key` both equal the
root and the Organisation public key the chain state read carries
(`NoProvisionalUpdate` for none), and, when that update commits, sets the
record's `org_private_key` to the `org_private_key` its
`ProvisionalChange::ChangeSet` holds and the record's `org_pub_key` to the
update's `org_pub_key`, in the same store save as the rest of the commit; the
record keeps no earlier key. When the commit is the node's own removal the
record is deleted instead (LLR-b27jr6) and no key is kept. On any refusal the
record's key and Organisation public key are unchanged (LLR-ewkg85).
satisfies: REQ-jy6ybw

**LLR-4kh9w9**: when `receive_and_verify` or
`receive_and_self_delete_if_revoked` commits an Organisation-information
message, the record's `org_pub_key` is set to the Organisation public key the
chain state read at verification carries, which the key check (LLR-ba2ejp)
has shown to be the public half of the stored `org_private_key`
(LLR-ckk5nz), so the record never pairs a public key with a private key that
is not its own half; a first admission creates the record with it
(LLR-xq9nrq). A revocation never sets either key: it is either refused
or accepted, and an accepted one forgets the Organisation (LLR-pt32fx,
LLR-r8qhky).
satisfies: REQ-ju6vn2

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3).*
This said an accepted revocation "deletes the record (LLR-6p4pj2,
LLR-b27jr6)"; it is now decided by `revocation::accept` and forgets the
Organisation through `forget_organisation`.

**LLR-xn5pwc**: `receive_one` returns `OrgNodeError::MalformedMessage` when
`recv_one` refuses the received body with `TransportError::Malformed`, so an
Organisation-information message that carries no Organisation private key, or
a key shorter than 32 bytes, is refused by `receive_and_verify` and
`receive_and_self_delete_if_revoked` with that typed error before any record
or expectation is consulted and with no `read_state` call, leaving the store
unchanged and unwritten.
satisfies: REQ-c29s93

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place:
`receive_one` is now the public `receive_message` (transport only, body
unchanged), and the refusal reaches org-io's receive sequence before any
chain-free phase runs: "with no `read_state` call" reads "before the
chain-free phase, so org-io is handed no Organisation to read".

**LLR-ba2ejp**: on an `OrgInformation` message, `receive_and_verify` and
`receive_and_self_delete_if_revoked`, once the message has verified against
the chain state read by their one `read_state` call, compare
`org_private_key.x25519_keypair().org_public_key()` with that state's
`org_pub_key`, and when they differ refuse with `OrgKeyMismatch { org_id }`
before any commit, record deletion, Persona change or expectation clearing,
leaving the store unchanged and unwritten. The check applies to a first
admission and to an update to a held record alike, and on a first admission
it runs before the own-Persona check (LLR-3f5h7b). A `Revocation` carries no
key and is not checked.
satisfies: REQ-bwx7eg

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place:
the check runs in the apply phase (`apply_receive`, `apply_self_delete`)
against "the state the apply phase is given", which reads for "the chain
state read by their one `read_state` call"; the comparison, its order and
its refusal are unchanged.

**LLR-38e2kn**: on a `Revocation` about an Organisation the node holds no
record of, `receive_and_verify` and `receive_and_self_delete_if_revoked`
refuse with `RevocationNotHeld { org_id }`, through `revocation::check_notice`
(LLR-r7zm39), before consulting the expected admissions and with no
`read_state` call, creating no record, clearing no expectation and writing
nothing; an expectation for that Organisation stays in place.
satisfies: REQ-vxqc5g

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place:
the refusal is made by the chain-free phase (`prepare_receive`,
`prepare_self_delete`); "with no `read_state` call" reads "refused by the
chain-free phase, which hands org-io no Organisation to read".

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3).*
`receive_and_self_delete_if_revoked` refused such a message with
`OrgNotOnChain`, as any message about an unheld Organisation (LLR-379hnv). A
revocation is now decided by `revocation::check_notice` on both paths, so
both refuse it with `RevocationNotHeld`; LLR-379hnv still governs an
Organisation-information message about an unheld Organisation.

**LLR-pt32fx**: on a `Revocation(notice)`, `receive_and_verify` and
`receive_and_self_delete_if_revoked` run `revocation::check_notice`
(LLR-r7zm39) and return its refusal with no `read_state` call; otherwise they
make their one `read_state` call and pass its state (`None` for an
Organisation with no on-chain state), the notice, and the device seeds of the
Personas bound to that Organisation to `revocation::accept` (LLR-tx8ruv).
On a refusal they return it, leaving the record, its keys, the Personas and
the provisional updates unchanged and writing nothing. On acceptance they
adopt the successor store, save it, and return the acknowledgements:
`receive_and_self_delete_if_revoked` as `SelfDeleteOutcome::SelfDeleted {
org_id, acknowledgements }`, `receive_and_verify` in
`ReceiveOutcome.acknowledgements` with the state's epoch and root. On an
`Acknowledgement(ack)` both run `revocation::check_acknowledgement`
(LLR-5azhry) with no `read_state` call and write nothing:
they return its refusal, or, when it verifies,
`receive_and_self_delete_if_revoked` returns
`SelfDeleteOutcome::Acknowledged(verified)` and `receive_and_verify` returns
it in `ReceiveOutcome.acknowledged`, with the record's epoch and root and no
acknowledgements. A revocation never commits an update into a record the
node keeps, and an Organisation-information message never reaches
`revocation::accept`.
satisfies: REQ-3dsweu, REQ-m2xh8q, REQ-b462sh

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place:
the same decisions, split across the two phases. `check_notice`, the sender
rule and `check_acknowledgement` run in the chain-free phase
(`prepare_receive`, `prepare_self_delete`), whose refusals hand org-io no
Organisation to read; an acknowledgement is decided there
(`Prepared::Done`). A notice that passes returns a pending value, and the
apply phase (`apply_receive`, `apply_self_delete`) passes "the state it is
given" — for "the state of their one `read_state` call" — to
`revocation::accept` with the lazy device-seed source, unchanged.

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3;
REQ-3dsweu and REQ-m2xh8q).* This verified a revocation "as any update"
against its Envelope and then, from the verified record, refused it with
`RevocationNotForThisDevice` when a Persona bound to the Organisation was
still listed, or deleted the record and marked the Personas Revoked. A
revocation now holds an absence proof, checked against the chain's current
root, and an accepted one forgets the Organisation and signs the
acknowledgements first (REQ-uxv2x2, REQ-y99c9w). The device seeds come from
the Personas `OrgService` holds until stage S4 moves the seed to the OS
keychain; then org-io supplies them.
