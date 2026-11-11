# Design — the commit workflow: revocation by proof, delete only when verified, acknowledgement, reconcile

Software items and low-level requirements for change
`worktree-org-io-commit-workflow` (stage S3). They refine
`crates/org-node/docs/requirements/2026-10-07-commit-workflow.md`
(REQ-ps2gy2, REQ-qrtsc9, REQ-m2xh8q, REQ-b462sh and earlier forms of REQ-y9p27b,
REQ-h29c8e and REQ-dwb9bq) and the amendments of this change to earlier forms of
REQ-x2dnee, REQ-48zc3e and RC-5jkcxz, and they realise RC-r8bp43, RC-ub82my,
RC-eydn8t and an earlier form of RC-45k3zg
(`crates/org-node/docs/risk/2026-10-07-commit-workflow.md`).
The org-io half of the workflow is org-io's, in its own ledgers.

*Added 2026-10-07 (owner rulings at the S3a close-out residual review,
decision 16 of the plan).* The sender rules: two earlier forms of LLR-283x6f (one under an earlier form of
REQ-uk9rw7 as amended, one under REQ-ea4qs5) and an earlier form of LLR-xudkh4 with LLR-5azhry as amended
(REQ-b462sh as amended), realising RC-u7kdam and RC-eydn8t's sender clause;
implemented by plan task T12a.

This design is written on top of two changes: the key-pair change
(`worktree-org-node-org-key-pair`: `WireMessage` of two kinds, the error
variants `MalformedMessage`, `RevocationNotHeld` and
`RevocationNotForThisDevice`, a fresh Organisation key pair per provisional
update, `commit_update` selecting by root and key), merged to master as
`1f52c36` and into this branch on 2026-10-07; and stage S2 (org-io exists;
org-node meets it by values in, values out, owner ruling B of 2026-10-06),
not yet merged. Where an item below names a type or variant those changes
introduce, it names it as they define it.

*Re-surveyed 2026-10-07 against the merged code* (`crates/org-node/src/service.rs`,
`crates/org-node/src/transport/wire.rs`). What that changed here:

- `OrgService` has a private `forget_organisation(&mut self, org_id)` that
  deletes the record and the provisional updates and marks the Personas
  Revoked; it is called by `commit_update`, `receive_and_verify` and
  `receive_and_self_delete_if_revoked`. `StoreData::forget_organisation`
  (an earlier form of LLR-2d5a7m) replaces it, and the removal item of an earlier form of
  SDD-z4m57w (retired and superseded in S4, task T9) names the call sites.
- `WireMessage::envelope()` returns the Envelope of either kind; with a
  revocation that holds no Envelope it gives way to `WireMessage::org_id()`
  (an earlier form of LLR-xr88a8 as amended).
- `send_update(outgoing, recipient, peer_addr)` sends a Device the record
  does not list a `Revocation { envelope }`. Until S2 and S4 move the send
  into org-io, the transport is still org-node's, so `send_update` takes the
  `CommitOutcome` and sends the matching notice (an earlier form of LLR-degg8t as amended).
- The receiver loop the app runs is `receive_and_self_delete_if_revoked`
  (`apps/desktop/src-tauri/src/commands.rs`), so the revocation and acknowledgement
  must be decided on that path and on `receive_and_verify` until S4's one
  receive path (an earlier form of LLR-jhy6t6 as amended).
- `still_member` and the Persona match derive each Persona's
  DevicePublicKey from its stored device seed. Stage S4 stores the public key
  only; the items below speak of a Persona's DevicePublicKey and take any
  device seed they sign with from the caller, so they hold under either
  layout (owner rulings recorded by S4
  §13–§15 on branch `worktree-org-io-transport`).

All items are class C, the unit's class. No dependency changes: the absence
proof is org-members' (`AbsenceProof`, `Trie::prove_absent`), and the
acknowledgement's signature is the Device key's Ed25519, already in
`ed25519-dalek` through `person`; `soup.md` is unchanged.

## Architectures considered

Owner ruling of 2026-10-06 on this draft: **one combined module**.

1. **One new pure module, the rest under the items that own the code
   (chosen by the owner).** `revocation.rs` (the item SDD-gatf4h and SDD-8zzmav later replaced) builds the notices a
   commit produces, decides a received one, and signs and checks the
   acknowledgements. The deletion, the kept Change set, the wire forms, the
   error variants and the reconcile go under the items that already own
   `store.rs`, `transport/wire.rs`, `error.rs` and the commit path. Every new
   function takes `&StoreData` and the Organisation state as values and
   returns a successor `StoreData` by value, so ruling B holds by
   construction and each decision is testable without a chain, a transport
   or a file. One item is added: the decomposition's nineteen becomes
   twenty. The receiver's acknowledgement check uses no chain state while
   the notice's acceptance must; that difference is stated per function
   (LLR-5azhry and an earlier form of LLR-rhsfw9), not by an item boundary.
2. **Everything as `OrgService` methods in `service.rs`.** Matches today's
   shape (`receive_and_self_delete_if_revoked` reads the chain itself), adds
   no item, but puts the new decisions behind the `ChainOps` trait and
   `async` that ruling B removes, so S2's rework would move them again.
   Rejected.
3. **Two modules, `revocation.rs` and `acknowledgement.rs`.** Proposed first
   in this draft, with a second item for the acknowledgement. Not chosen by
   the owner. That item's ID was minted for this draft only, never merged
   and referenced nowhere else, so it is dropped rather than carried as a
   withdrawn item.

Placement:

- the revocation item SDD-gatf4h and SDD-8zzmav later replaced (new, `revocation.rs`):
  LLR-kr5t6f, LLR-r7zm39, LLR-5azhry and earlier forms of LLR-rhsfw9, LLR-5tvn7g,
  LLR-f9mcsq, LLR-g2jeqd and LLR-bs6yxf;
- SDD-kwncn7, which owns `transport/wire.rs`: LLR-dc45ur, LLR-378cj4;
- SDD-af5vnt, which owns `store.rs`: earlier forms of LLR-2d5a7m and LLR-p8vpgz;
- SDD-swtd3w, which owns `error.rs`: an earlier form of LLR-3sddey;
- the item SDD-busuj3 and SDD-kegag2 replaced, which owned the commit path: earlier
  forms of LLR-rr558z, LLR-34c5y3 and LLR-degg8t;
- an earlier form of SDD-z4m57w, which owned the node's own removal: its removal item (retired
  and superseded by an earlier form of LLR-9jn3z8 in S4, task T9, 2026-10-13).

Each existing owning item's `traces:` line is amended in place in
`2026-10-03-decomposition.md` with a dated note naming this change, and so
are the low-level requirements this change falsifies: earlier forms of LLR-2d5a7m (decomposition) and of LLR-34c5y3, LLR-jhy6t6 and LLR-qqqg4b
(`2026-10-06-chain-authority.md`).

Amended in place on 2026-10-07, once the key-pair change had merged, in the
files that define them: an earlier form of LLR-xr88a8 (the `Revocation` variant holds a
`RevocationNotice`, not an Envelope, and a third variant is added:
LLR-dc45ur, LLR-378cj4), of LLR-jhy6t6 (a revocation is no longer verified
"as any update"; the receive paths hand it to `revocation::accept`, an
earlier form of LLR-rhsfw9), of LLR-95av3e (the unheld refusal moves to LLR-r7zm39, on both
receive paths), of LLR-degg8t (`send_update` sends a Device the update removed
its notice from `CommitOutcome.revocations`, another earlier form of LLR-degg8t, and nothing to a
Device neither record lists; org-io sends it once S4 moves the transport),
of LLR-3sddey (what `RevocationNotForThisDevice` now refuses) and of LLR-34c5y3
(in `2026-10-07-org-key-pair.md`); and earlier forms of LLR-degg8t and LLR-f9mcsq (in
`2026-10-03-decomposition.md`).

## The shape of the change, for an implementer

    revocation::RevocationNotice { org_id: OrgId, member_id: MemberId,
                                   device: DevicePublicKey, proof: AbsenceProof }
    revocation::Accepted { store: StoreData, acknowledgements: Vec<Acknowledgement> }
    revocation::Acknowledgement { org_id: OrgId, member_id: MemberId,
                                  device: DevicePublicKey, epoch: Epoch,
                                  root: RootHash, signature: [u8; 64] }
    revocation::VerifiedAcknowledgement(Acknowledgement)        // constructed only by `check_acknowledgement`
    revocation::ACK_DOMAIN: &[u8] = b"ods/org-node/revocation-acknowledgement/v1"
    revocation::accept(store, notice, chain: Option<OrgState>,
                       device_seeds: impl FnOnce() -> Vec<DeviceSeed>)  // lazy: called just before signing (T10c)
    revocation::sign_acknowledgements(store, org_id, verified: &OrgState, device_seeds: Vec<DeviceSeed>)
    reconcile::reconcile(store, org_id, chain: OrgState,
                         device_seeds: impl FnOnce() -> Vec<DeviceSeed>)  // lazy: removal path only (T10b)
    WireMessage::Revocation(RevocationNotice)                   // index 1, replaces { envelope }
    WireMessage::Acknowledgement(Acknowledgement)               // index 2, new
    WireMessage::org_id(&self) -> OrgId                         // replaces envelope()
    OrgRecord.kept_change_set: Option<Vec<u8>>                  // the Change set that produced it
    CommitOutcome.revocations: Vec<RevocationNotice>            // new field
    CommitOutcome.acknowledgements: Vec<Acknowledgement>        // new field, empty unless the commit removed this node (an earlier form of LLR-jhy6t6)
    send_update(outcome: &CommitOutcome, recipient, peer_addr)  // takes the outcome, was &OutgoingUpdate
    ReceiveOutcome.acknowledgements: Vec<Acknowledgement>       // new field, empty unless removed
    ReceiveOutcome.acknowledged: Option<VerifiedAcknowledgement> // new field, a received one
    SelfDeleteOutcome::{ SelfDeleted { org_id, acknowledgements },
                         UpdatedNotRevoked { org_id }, Acknowledged(VerifiedAcknowledgement) }
    reconcile::Reconciled::{ InStep, Committed { store, outcome: CommitOutcome },
                             Removed { store, acknowledgements }, Behind { chain_epoch } }
    OrgNodeError::{ OrgNotHeld { org_id }, StaleChainState { org_id, chain_epoch, record_epoch },
                    ChainStateConflict { org_id }, RevocationProofRefused { org_id, cause },
                    AcknowledgementNotHeld { org_id }, AcknowledgementFromFuture { org_id },
                    AcknowledgementForListedDevice { org_id }, AcknowledgementSignatureInvalid { org_id },
                    DeviceSecretNotSupplied { org_id }, NoRevocationForRecipient { org_id } }

`DeviceSeed` in `accept`, `sign_acknowledgements` and `reconcile` is the
device seed moved in by the caller for that one call (an earlier form of REQ-h29c8e as amended):
today `OrgService` takes it from the Persona it holds, and after stage S4 org-io
moves it in from the OS keychain as S4's transient device-secret type, which
replaces `DeviceSeed` in these signatures and zeroises it. No function here
clones a seed or stores one.

*Amended 2026-10-07 (plan tasks T10b, T10c and T12).* `accept` and
`reconcile` take a seed source, `impl FnOnce() -> Vec<DeviceSeed>`, not the
seeds: `accept` calls it only once the notice is accepted, just before
signing, so a refused notice never reads the seeds (an earlier form of LLR-5tvn7g); `reconcile`,
and `commit_update` through the one commit step, call it only on the path
where the committed record removes this node, so a commit that keeps the
node listed never obtains a seed (earlier forms of LLR-rr558z and LLR-34c5y3).
`sign_acknowledgements` and the removal step take the seeds themselves.
`CommitOutcome` also gains `acknowledgements`: the acknowledgements
`commit_update` or `reconcile` signed when the commit removed this node
(an earlier form of LLR-jhy6t6), empty otherwise; this list had omitted it.

"The store" below is the `StoreData` value. A function returns its successor
by value or returns none; S2's values-out `OrgService` seals a successor into
the store bytes org-io writes, and returns no store bytes with a refusal.

## The revocation a revoked Device receives, the decision it acts on, and its acknowledgement

`crates/org-node/src/revocation.rs`.

**LLR-kr5t6f**: `revocation::notices_for(previous: &OrgRecord, committed:
&OrgRecord, committed_trie: &Trie) -> Result<Vec<RevocationNotice>,
OrgNodeError>`, where `committed_trie` is the calculated trie of
`committed`, returns one notice for each pair (MemberId, DevicePublicKey)
that a member snapshot of `previous` lists and whose DevicePublicKey no
member snapshot of `committed` lists, in the order of `previous`'s member
snapshots and, within one, of its device slots; each notice holds
`committed.org_id`, that MemberId, that DevicePublicKey and
`committed_trie.prove_absent(member_id, device)`, and nothing else. It
returns an empty list when no Device was removed, and refuses with
`Trie(cause)` when `prove_absent` refuses (an uncalculated trie), returning
no notice.
satisfies: REQ-ps2gy2

*Amended 2026-10-13 (S4, task T5):*
unchanged; its caller is now the commit step's body builder, which encodes
each notice as a revocation body (earlier forms of LLR-degg8t).

**LLR-r7zm39**: `revocation::check_notice(store: &StoreData, notice:
&RevocationNotice) -> Result<&PersonaRecord, OrgNodeError>` takes no
Organisation state and refuses, in this order: with `RevocationNotHeld {
org_id }` when the store holds no record of `notice.org_id`; with
`RevocationNotForThisDevice { org_id }` when no Persona bound to that
Organisation has `member_id` equal to `Some(notice.member_id)` and a device
key equal to `notice.device`. Otherwise it returns that Persona. A notice
that does not decode never reaches it: the wire decode refuses it first
(LLR-dc45ur, `MalformedMessage`).
satisfies: REQ-qrtsc9

*Amended 2026-10-13 (S4, task T4).*
Called from `check_received` (through `revocation::notice_store`, which
picks the store whose Persona the notice is for and raises the unheld
refusal) and again from `commit_checked` through `revocation::accept`. The
rule is unchanged.

*Amended 2026-10-08 (S4, task T3):* a store holds one Persona:
`check_notice` returns the store's Persona when it is the one the notice names
and refuses otherwise; the service finds the store a notice is for among its
stores.

### The acknowledgement, in the same module

**LLR-5azhry**: `revocation::check_acknowledgement(store: &StoreData, ack:
Acknowledgement) -> Result<VerifiedAcknowledgement, OrgNodeError>` takes no
Organisation state and refuses, in this order (cheapest first): with
`AcknowledgementNotHeld { org_id }` when the store holds no record of
`ack.org_id`; with `AcknowledgementFromFuture { org_id }` when `ack.epoch`
is greater than that record's epoch; with `AcknowledgementForListedDevice {
org_id }` when a member snapshot of that record lists `ack.device`; with
`AcknowledgementSignatureInvalid { org_id }` when `verify_strict` of
`signature` over `signed_bytes()` under `ack.device` fails. Otherwise it
returns `VerifiedAcknowledgement(ack)`, which only this function constructs.
It produces no successor: the store is unchanged. (2026-10-07) Device bytes
that are not a valid ed25519 key never reach this check: `ack.device` is a
`DevicePublicKey`, which parses its bytes (person crate), so such an
acknowledgement does not decode; were they decoded, the refusal would be
`AcknowledgementSignatureInvalid { org_id }`.
satisfies: REQ-b462sh

*Amended 2026-10-13 (S4, task T4).*
Called from `check_received` with the sender org-io passed in. The rule is
unchanged.

*Amended 2026-10-07 (owner ruling at the S3a close-out residual review:
"Acknowledgements of revocations will be sent from devices on the
revocation list, so their ids can be checked before accepting"; REQ-b462sh
as amended; plan task T12a).* The signature becomes
`check_acknowledgement(store: &StoreData, sender: DevicePublicKey, ack:
Acknowledgement)`, `sender` the DevicePublicKey the transport authenticated
for the connection that delivered `ack`, and the refusal order becomes:
`AcknowledgementNotHeld { org_id }`; then `AcknowledgementNotFromItsDevice {
org_id }` when `sender` is not `ack.device`; then `AcknowledgementFromFuture`,
`AcknowledgementForListedDevice` and `AcknowledgementSignatureInvalid` as
above. The sender check precedes the signature check; the function still
takes no Organisation state and produces no successor.

## SDD-kwncn7 — The wire frame and its bound (additions)

**LLR-dc45ur**: `WireMessage::Revocation(RevocationNotice)` (variant index
1) encodes, by postcard, `org_id`, `member_id`, `device` and `proof` (in
org-members' wire form) in that order and nothing else; `decode_body`
refuses with `Malformed`, and does not panic, a notice whose device bytes
are not a valid Ed25519 public key, whose proof carries more siblings than
its default map allows (org-members' proof parse, at most 256), or whose body
has bytes left over; the frame's 1 MiB bound (`MAX_FRAME`) applies to it as
to every body, and `receive_one` maps `Malformed` to `MalformedMessage`.
satisfies: REQ-ps2gy2, REQ-qrtsc9

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place: a
rename. `receive_one` reads `receive_message`, now public, body unchanged.

*Amended 2026-10-08 (S4, task T2):*
code path only. The decoder is `decode_message` (`crates/org-node/src/wire.rs`),
whose refusal is `OrgNodeError::MalformedMessage`, and the bound is
`MAX_MESSAGE` (1 MiB); `receive_message` still reports a body that does not
decode as `MalformedMessage`. The rule is unchanged.

**LLR-378cj4**: `WireMessage::Acknowledgement(Acknowledgement)` (variant
index 2) encodes, by postcard, `org_id`, `member_id`, `device`, `epoch`,
`root` and the 64 signature bytes in that order; `decode_body` refuses with
`Malformed`, and does not panic, a body whose device is not a valid Ed25519
public key, whose signature is not exactly 64 bytes, or that has bytes left
over. An acknowledgement is received on the same receive path as every
other Wire message (one receive path, owner ruling of 2026-10-06).
satisfies: REQ-b462sh

*Amended 2026-10-08 (S4, task T2):*
code path only. The decoder is `decode_message` (`crates/org-node/src/wire.rs`),
whose refusal is `OrgNodeError::MalformedMessage`. The rule is unchanged.

## SDD-af5vnt — The encrypted persona store (additions)

## SDD-swtd3w — The rejection vocabulary (additions)

## The commit path (additions)

### Who sent the message (owner rulings of 2026-10-07; plan task T12a)

The receive paths `receive_and_verify` and
`receive_and_self_delete_if_revoked` (owned by the items SDD-busuj3, SDD-kegag2 and SDD-z4m57w
replaced) keep the
DevicePublicKey `OrgEndpoint::recv_one` returns: the private `receive_one`
returns `(DevicePublicKey, WireMessage)` instead of discarding it. Until S4
moves receiving into org-io, org-node takes the sender from its own
endpoint; afterwards org-io passes it in with the message (values in).

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* The sender
is already a value: `receive_message` (public, the former `receive_one`)
returns it with the message, and the chain-free phase (`prepare_receive`,
`prepare_self_delete`) takes both as arguments. In S2 org-io passes what
`receive_message` returned; S4 changes only who supplies it.

## The node's own removal (addition)

*Retired 2026-10-13 (S4, task T9;
owner ruling of that day, option A).* The item that stood here said where
`StoreData::forget_organisation` is called (`revocation::accept` and the
commit paths' one removal step) and that no other function in org-node
removes an `OrgRecord` or a Persona's record. A 2026-10-07 revision named
the one removal step, and a 2026-10-08 amendment (S4, task T3) named
`OrgService::adopt` as the one place a store is removed. It is superseded
by a new item in `crates/org-node/docs/architecture/2026-10-07-sans-io.md`
(an earlier form of SDD-z4m57w, S4 supersession). That item keeps this content and adds
`OrgService::drop_store`, which org-io uses after a local IO failure, as a
second exempt place. Per the owner's retirement ruling of 2026-10-13, the
retired ID is listed only in the S4 verification record.
