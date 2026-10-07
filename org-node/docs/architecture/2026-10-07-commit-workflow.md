# Design — the commit workflow: revocation by proof, delete only when verified, acknowledgement, reconcile

Software items and low-level requirements for change
`worktree-org-io-commit-workflow` (stage S3 of
`docs/plans/2026-10-06-org-io-roadmap.md`). They refine
`org-node/docs/requirements/2026-10-07-commit-workflow.md`
(REQ-ps2gy2, REQ-qrtsc9, REQ-m2xh8q, REQ-em28bq, REQ-y99c9w, REQ-b462sh,
REQ-tb4f8p) and the amendments of this change to REQ-uv3v5w, REQ-uxv2x2 and
RC-wqgm2p, and they realise RC-r8bp43, RC-ub82my, RC-eydn8t and RC-44vvjp
(`org-node/docs/risk/2026-10-07-commit-workflow.md`).
The org-io half of the workflow is proposed, unminted, in
`docs/plans/2026-10-06-org-io-commit-workflow.md`.

*Added 2026-10-07 (owner rulings at the S3a close-out residual review,
decision 16 of the plan).* The sender rules: LLR-2r2fha (REQ-ztdza4 as
amended), LLR-kzgjz8 (REQ-ea4qs5) and LLR-3aysup with LLR-5azhry as amended
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

*Re-surveyed 2026-10-07 against the merged code* (`org-node/src/service.rs`,
`org-node/src/transport/wire.rs`). What that changed here:

- `OrgService` has a private `forget_organisation(&mut self, org_id)` that
  deletes the record and the provisional updates and marks the Personas
  Revoked; it is called by `commit_update`, `receive_and_verify` and
  `receive_and_self_delete_if_revoked`. `StoreData::forget_organisation`
  (LLR-pba7yu) replaces it, and LLR-23sfdh names the call sites.
- `WireMessage::envelope()` returns the Envelope of either kind; with a
  revocation that holds no Envelope it gives way to `WireMessage::org_id()`
  (LLR-js9dsu as amended).
- `send_update(outgoing, recipient, peer_addr)` sends a Device the record
  does not list a `Revocation { envelope }`. Until S2 and S4 move the send
  into org-io, the transport is still org-node's, so `send_update` takes the
  `CommitOutcome` and sends the matching notice (LLR-6ymd6d as amended).
- The receiver loop the app runs is `receive_and_self_delete_if_revoked`
  (`app/src-tauri/src/commands.rs`), so the revocation and acknowledgement
  must be decided on that path and on `receive_and_verify` until S4's one
  receive path (LLR-pt32fx as amended).
- `still_member` and the Persona match derive each Persona's
  DevicePublicKey from its stored device seed. Stage S4 stores the public key
  only; the items below speak of a Persona's DevicePublicKey and take any
  device seed they sign with from the caller, so they hold under either
  layout (owner rulings recorded by S4, `docs/plans/2026-10-06-org-io-transport.md`
  §13–§15 on branch `worktree-org-io-transport`).

All items are class C, the unit's class. No dependency changes: the absence
proof is org-members' (`AbsenceProof`, `Trie::prove_absent`), and the
acknowledgement's signature is the Device key's Ed25519, already in
`ed25519-dalek` through `person`; `soup.md` is unchanged.

## Architectures considered

Owner ruling of 2026-10-06 on this draft: **one combined module**.

1. **One new pure module, the rest under the items that own the code
   (chosen by the owner).** `revocation.rs` (SDD-uck4tz) builds the notices a
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
   (LLR-5azhry, LLR-tx8ruv), not by an item boundary.
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

- SDD-uck4tz (new, `revocation.rs`): LLR-kr5t6f, LLR-r7zm39, LLR-tx8ruv,
  LLR-r8qhky, LLR-uw7nmv, LLR-xgefn8, LLR-gbe9bt, LLR-hby4jr, LLR-5azhry;
- SDD-kwncn7, which owns `transport/wire.rs`: LLR-dc45ur, LLR-378cj4;
- SDD-af5vnt, which owns `store.rs`: LLR-pba7yu, LLR-d9778a;
- SDD-swtd3w, which owns `error.rs`: LLR-n67aw8;
- SDD-8cpyfa, which owns the commit path: LLR-gr8x3r, LLR-fm38ww,
  LLR-a8z7r5;
- SDD-72ddm6, which owns the node's own removal: LLR-23sfdh.

Each existing owning item's `traces:` line is amended in place in
`2026-10-03-decomposition.md` with a dated note naming this change, and so
are the low-level requirements this change falsifies: LLR-6p4pj2 and
LLR-jwhzh3 (decomposition), LLR-mkj4bz, LLR-b27jr6 and LLR-eyc4ud
(`2026-10-06-chain-authority.md`).

Amended in place on 2026-10-07, once the key-pair change had merged, in the
files that define them: LLR-js9dsu (the `Revocation` variant holds a
`RevocationNotice`, not an Envelope, and a third variant is added:
LLR-dc45ur, LLR-378cj4), LLR-pt32fx (a revocation is no longer verified
"as any update"; the receive paths hand it to `revocation::accept`,
LLR-tx8ruv), LLR-38e2kn (the unheld refusal moves to LLR-r7zm39, on both
receive paths), LLR-6ymd6d (`send_update` sends a Device the update removed
its notice from `CommitOutcome.revocations`, LLR-a8z7r5, and nothing to a
Device neither record lists; org-io sends it once S4 moves the transport),
LLR-j5vbqj (what `RevocationNotForThisDevice` now refuses) and LLR-4kh9w9
(in `2026-10-07-org-key-pair.md`); LLR-8hdu9x and LLR-jsx922 (in
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
    CommitOutcome.acknowledgements: Vec<Acknowledgement>        // new field, empty unless the commit removed this node (LLR-b27jr6)
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
device seed moved in by the caller for that one call (REQ-y99c9w as amended):
today `OrgService` takes it from the Persona it holds, and after stage S4 org-io
moves it in from the OS keychain as S4's transient device-secret type, which
replaces `DeviceSeed` in these signatures and zeroises it. No function here
clones a seed or stores one.

*Amended 2026-10-07 (plan tasks T10b, T10c and T12).* `accept` and
`reconcile` take a seed source, `impl FnOnce() -> Vec<DeviceSeed>`, not the
seeds: `accept` calls it only once the notice is accepted, just before
signing, so a refused notice never reads the seeds (LLR-r8qhky); `reconcile`,
and `commit_update` through the one commit step, call it only on the path
where the committed record removes this node, so a commit that keeps the
node listed never obtains a seed (LLR-gr8x3r, LLR-fm38ww).
`sign_acknowledgements` and the removal step take the seeds themselves.
`CommitOutcome` also gains `acknowledgements`: the acknowledgements
`commit_update` or `reconcile` signed when the commit removed this node
(LLR-b27jr6), empty otherwise; this list had omitted it.

"The store" below is the `StoreData` value. A function returns its successor
by value or returns none; S2's values-out `OrgService` seals a successor into
the store bytes org-io writes, and returns no store bytes with a refusal.

## SDD-uck4tz — The revocation a revoked Device receives, the decision it acts on, and its acknowledgement

`org-node/src/revocation.rs`.

**SDD-uck4tz**: the revocation notice — building one for each Device a
commit removes, from the committed record, holding only that Device's
identity and an absence proof; and deciding a received notice against an
Organisation state the caller supplies, refusing it with a typed error or
accepting it into the acknowledgements and the store without that
Organisation; and the acknowledgement a revoked Device signs before it
deletes, with the check every other node runs on one it receives, under a
signing domain of its own and using no chain state. Pure functions over
values: no chain read, no transport, no file.
traces: REQ-ps2gy2, REQ-qrtsc9, REQ-m2xh8q, REQ-em28bq, REQ-y99c9w, REQ-uxv2x2, REQ-b462sh

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

**LLR-tx8ruv**: `revocation::accept(store: &StoreData, notice:
&RevocationNotice, chain: Option<OrgState>, device_seeds: impl FnOnce() ->
Vec<DeviceSeed>) -> Result<Accepted, OrgNodeError>` (the seed source called
only after every refusal check below has passed, just before signing,
LLR-r8qhky; a refused notice never calls it) runs `check_notice` first and
returns its refusal unchanged;
then refuses with `OrgNotOnChain` when `chain` is `None`; with
`StaleChainState { org_id, chain_epoch, record_epoch }` when
`chain.epoch` is lower than the record's epoch; and with
`RevocationProofRefused { org_id, cause }`, `cause` the org-members error,
when `notice.proof.verify(&chain.root_hash, &notice.member_id,
&notice.device)` (the hasher the node's tries use) refuses — root mismatch,
or the Device still held. A `chain.epoch` equal to the record's is accepted
(RC-ub82my: at its own epoch a listed Device has no valid proof).
satisfies: REQ-m2xh8q

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, review
round 2 finding-1; owner ruling R3, false deletion extremely unlikely).*
Between `StaleChainState` and the proof check, `accept` now also refuses
with `ChainStateConflict { org_id }` when `chain.epoch` equals the record's
epoch and `chain.root_hash` is not the record's `root_hash` — the refusal
`reconcile` gives the same state (LLR-gr8x3r). Such a state proves the
chain disagrees with the record; before, `accept` checked the proof against
its root and could delete on it. The seed source is not called and nothing
is forgotten. With this check an equal epoch passes only with the record's
own root, under which the record still lists the Device, so the proof is
then refused: a notice is acted on only against a chain state of a later
epoch. The sentence above saying an equal epoch "is accepted" means it
passes the `StaleChainState` check only.

**LLR-r8qhky**: when every check of `accept` passes, it first signs, through
`revocation::sign_acknowledgements(store, org_id, &chain, device_seeds())`
(LLR-hby4jr), one acknowledgement for each Persona bound to that
Organisation, returning that function's refusal unchanged, and then builds
the successor `forget_organisation(store, org_id)` (LLR-pba7yu); it returns
both as `Accepted { store, acknowledgements }`. The successor holds no
acknowledgement and no key of a Persona bound to that Organisation, and no
device seed passed in outlives the call.
satisfies: REQ-y99c9w, REQ-uxv2x2

**LLR-uw7nmv**: on any refusal, `accept` returns the error alone: no
successor and no acknowledgement is produced, and the `StoreData` passed in
is borrowed immutably and so unchanged.
satisfies: REQ-m2xh8q, REQ-em28bq

**LLR-xgefn8**: every public function of `revocation.rs` and of
`reconcile` (LLR-gr8x3r) takes the store as
`&StoreData` and any chain state as an `OrgState` or `Option<OrgState>`
argument, returns a successor `StoreData` by value or none, is not `async`,
is generic over no IO trait, and calls no `ChainOps`, `ChainReader`,
transport, endpoint or file-system function; the `absences` test reads these
files' source for those names. This LLR also constrains SDD-8cpyfa's
reconcile.
satisfies: REQ-qrtsc9, REQ-m2xh8q, REQ-em28bq, REQ-b462sh, REQ-tb4f8p

### The acknowledgement, in the same module

**LLR-gbe9bt**: the bytes an `Acknowledgement` signs are `ACK_DOMAIN`
(`b"ods/org-node/revocation-acknowledgement/v1"`) followed by the postcard
encoding of the tuple `(org_id, member_id, device, epoch, root)`;
`Acknowledgement::signed_bytes(&self) -> Vec<u8>` returns them, and
`signature` is the 64-byte Ed25519 signature over them. No other message
org-node signs or verifies begins with `ACK_DOMAIN`.
satisfies: REQ-y99c9w, REQ-b462sh

**LLR-hby4jr**: `revocation::sign_acknowledgements(store: &StoreData, org_id: OrgId,
verified: &OrgState, device_seeds: Vec<DeviceSeed>) ->
Result<Vec<Acknowledgement>, OrgNodeError>` returns, in store order, one
acknowledgement for each Persona whose `org_id` is `Some(org_id)` — every
such Persona's `member_id` is set, because each path that binds a Persona
sets it with the `org_id`: `commit_genesis` to the MemberId of the genesis
leaf listing the Persona's DevicePublicKey, and a first admission on
`receive_and_verify` to the admitted leaf's — naming `org_id`, that
`member_id`, that Persona's
DevicePublicKey, `verified.epoch` and `verified.root_hash`, signed with the
signing key pair of the seed in `device_seeds` whose public key is that
Persona's DevicePublicKey; Ed25519 signing is deterministic, so it draws no
randomness. When such a Persona has no matching seed in `device_seeds` it
refuses with `DeviceSecretNotSupplied { org_id }` and returns no
acknowledgement. It reads no seed from the store. `device_seeds` is taken by
value and dropped when the function returns, on success and on refusal; no
seed is cloned, returned or kept, and a seed matching no such Persona is
unused. The returned values hold no secret key and no other Organisation
data.
satisfies: REQ-y99c9w

*Revised 2026-10-07 (owner rulings recorded by stage S4: the device seed
lives only in the OS keychain and org-node receives it transiently for
signing).* This signed with each Persona's stored `device_seed`.

*Amended 2026-10-07 (PR-eqs4fs; change `worktree-org-io-commit-workflow`,
task s3t12b).* This said one
acknowledgement for each bound Persona "whose `member_id` is set", narrower
than REQ-y99c9w's "each Persona bound to that Organisation": the founding
Persona was bound by `commit_genesis` with no `member_id`, so a founder
removed from its Organisation signed no acknowledgement. `commit_genesis` now
sets it, and no bound Persona is without one. No fallback derives a MemberId
for a stored founder without one: stores written before S3 do not load (owner
ruling of 2026-10-07, plan decision 12), so no store this code reads was
written by the defective binding outside this unmerged change.

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

**LLR-378cj4**: `WireMessage::Acknowledgement(Acknowledgement)` (variant
index 2) encodes, by postcard, `org_id`, `member_id`, `device`, `epoch`,
`root` and the 64 signature bytes in that order; `decode_body` refuses with
`Malformed`, and does not panic, a body whose device is not a valid Ed25519
public key, whose signature is not exactly 64 bytes, or that has bytes left
over. An acknowledgement is received on the same receive path as every
other Wire message (one receive path, owner ruling of 2026-10-06).
satisfies: REQ-b462sh

## SDD-af5vnt — The encrypted persona store (additions)

**LLR-pba7yu**: `StoreData::forget_organisation(&self, org_id: OrgId) ->
StoreData` returns a copy with, removed: the `OrgRecord` of `org_id` (and so
its kept Change set, Organisation private key and proxy account); every
provisional update whose `org_id` is `Some(org_id)`; every expected
admission for `org_id`; every Persona whose `org_id` is `Some(org_id)`, with
every key the store holds for it (today its member and device seeds; after
stage S4, its member seed and Device public key); and every genesis
provisional update built by one of those Personas. It adds nothing in their place: no tombstone, no
"was a member" marker, no copy of an acknowledgement, and no other value
that names `org_id` (owner ruling of 2026-10-06 on retention). Every other
record, Persona, provisional update and expected admission is in the copy
unchanged and in its original order.
satisfies: REQ-uxv2x2, REQ-em28bq

*Note 2026-10-07.* It replaces the private `OrgService::forget_organisation`
of the merged key-pair change, which marked the Personas Revoked. Under S4's
one sealed store per Persona (owner ruling of 2026-10-06, S4 plan §15 point
1), the copy for that Persona's store holds no Persona at all; deleting the
store file and the Persona's keychain item is then org-io's IO (the plan's
proposed 7), not this function's. It holds under either layout.

**LLR-d9778a**: `OrgRecord` (and `RawOrgRecord`) gains `kept_change_set:
Option<Vec<u8>>`, encoded in the store with the rest of the record: `None`
on a record `commit_genesis` creates, and on every other commit — a first
admission included — exactly the Change set bytes of the Envelope that commit
verified, replacing any earlier one. It is never larger than `MAX_FRAME`,
because it is the Change set of a received frame or of a provisional update
bounded by `MAX_PROVISIONAL_BYTES`; it is sent to nobody in this change.
satisfies: REQ-uv3v5w

*Amended 2026-10-07 (change `worktree-org-io-commit-workflow`, review
finding-1 of round 1).* This said `None` also "on a record … a first
admission creates", and the code kept none there. REQ-uv3v5w exempts only a
record created by genesis: a first admission is a received update whose
Change set produced the joiner's record, and LLR-mkj4bz already set it for
every commit through `receive_and_verify`. The first-admission record now
keeps the admitting Envelope's Change set; no reader of `kept_change_set`
depends on `None` there. REQ-uv3v5w's text is unchanged.

## SDD-swtd3w — The rejection vocabulary (additions)

**LLR-n67aw8**: `OrgNodeError` gains ten variants, each distinct from
every other variant and each with a `Display` naming the Organisation:
`DeviceSecretNotSupplied { org_id }` (a Persona bound to the Organisation
whose device seed the caller did not give, LLR-hby4jr);
`NoRevocationForRecipient { org_id }` (`send_update` to a Device the record
does not list and the outcome holds no notice for, LLR-6ymd6d as amended);
`OrgNotHeld { org_id }` (a reconcile for an Organisation with no record);
`StaleChainState { org_id, chain_epoch: Epoch, record_epoch: Epoch }`,
whose `Display` also names both epochs (a state older than the record);
`ChainStateConflict { org_id }` (the record's epoch with a different
root); `RevocationProofRefused { org_id, cause: OrgMembersError }`, whose
`Display` also names the cause; `AcknowledgementNotHeld { org_id }`;
`AcknowledgementFromFuture { org_id }`; `AcknowledgementForListedDevice {
org_id }`; and `AcknowledgementSignatureInvalid { org_id }`.
satisfies: REQ-m2xh8q, REQ-b462sh, REQ-tb4f8p, REQ-y99c9w, REQ-3dsweu

*Revised 2026-10-07 (re-survey after the key-pair change merged).* This
listed eight; `DeviceSecretNotSupplied` follows from the transient device
seed and `NoRevocationForRecipient` from `send_update` taking the outcome.

*Note 2026-10-07 (owner rulings at the S3a close-out residual review; plan
task T12a).* Two more variants, each distinct and with a `Display` naming the
Organisation, are added by T12a and specified by the items that raise them:
`SenderNotListed { org_id }` (an update for a held Organisation or a
revocation from a Device the record does not list, LLR-2r2fha, LLR-kzgjz8)
and `AcknowledgementNotFromItsDevice { org_id }` (an acknowledgement
delivered by a Device other than the one it names, LLR-5azhry as amended,
LLR-3aysup). This item's list of ten is unchanged.

## SDD-8cpyfa — The commit path (additions)

**LLR-gr8x3r**: `reconcile(store: &StoreData, org_id: OrgId, chain:
OrgState, device_seeds: impl FnOnce() -> Vec<DeviceSeed>) -> Result<Reconciled, OrgNodeError>`
(the seed source called only by a commit that removes this node, LLR-fm38ww) refuses with `OrgNotHeld {
org_id }` when the store holds no record of `org_id`, and otherwise decides
from the record and `chain` alone: `chain.epoch` lower than the record's →
`StaleChainState`; equal epoch and equal Membership root → `InStep`; equal
epoch and a different root → `ChainStateConflict`; greater epoch and a
stored provisional update for `org_id` whose `resulting_root` and
`org_pub_key` equal `chain.root_hash` and `chain.org_pub_key` → the commit
of LLR-fm38ww; greater epoch and no such update → `Behind { chain_epoch }`.
`InStep`, `Behind` and every refusal carry no successor store.
satisfies: REQ-tb4f8p

**LLR-fm38ww**: the commit `reconcile` makes runs the commit step
`commit_update` runs against the same state (LLR-cmdrp9, LLR-6s785x: the
Envelope built from the selected update, the chain-free checks, the epoch
rule, verification against `chain`), and returns `Committed { store,
outcome }`, or, when the committed record lists no DevicePublicKey of a
Persona bound to that Organisation, `Removed { store, acknowledgements }`
built as LLR-r8qhky builds `Accepted`, with `chain` as the verified state.
A refusal of that step is returned as the error, with no successor.
satisfies: REQ-tb4f8p, REQ-uxv2x2

**LLR-a8z7r5**: `CommitOutcome` gains `revocations: Vec<RevocationNotice>`,
which `commit_update` and `reconcile` fill with `notices_for(previous record,
committed record, committed trie)` (LLR-kr5t6f), so the notices org-io sends
are those of exactly the committed update; a received update's commit
produces none, since only the node that built an update fans it out.
satisfies: REQ-ps2gy2

### Who sent the message (owner rulings of 2026-10-07; plan task T12a)

The receive paths `receive_and_verify` and
`receive_and_self_delete_if_revoked` (SDD-8cpyfa, SDD-72ddm6) keep the
DevicePublicKey `OrgEndpoint::recv_one` returns: the private `receive_one`
returns `(DevicePublicKey, WireMessage)` instead of discarding it. Until S4
moves receiving into org-io, org-node takes the sender from its own
endpoint; afterwards org-io passes it in with the message (values in).

**LLR-2r2fha**: on an `OrgInformation` message whose Envelope names an
Organisation the store holds a record of, `receive_and_verify` and
`receive_and_self_delete_if_revoked` refuse with `SenderNotListed { org_id }`
when the sender `receive_one` returned is in no member snapshot's
`device_keys` of that record. The check comes after the held-record lookup
(an unheld Organisation is refused as before: a first admission on
`receive_and_verify`, `OrgNotOnChain` on the self-delete path, LLR-379hnv)
and before `check_chain_free`, the chain read and any decode of the Change
set or snapshot; the refusal writes nothing, saves nothing and leaves every
expectation in place. A first admission (no record held) is not subject to
it: it is accepted from any sender and verified against the chain
(REQ-xa6smf; owner amendment of 2026-10-07). A sender listed in the record
and absent from the verified update's record (a Member relaying its own
removal) passes.
satisfies: REQ-ztdza4

**LLR-kzgjz8**: on a `Revocation(notice)`, both receive paths, after
`revocation::check_notice` passes (LLR-r7zm39) and before the chain read and
`revocation::accept`, refuse with `SenderNotListed { org_id }` when the
sender `receive_one` returned is in no member snapshot's `device_keys` of the
record of `notice.org_id`; the refusal reads no chain, never calls the seed
source, writes nothing and saves nothing.
satisfies: REQ-ea4qs5

**LLR-3aysup**: on an `Acknowledgement(ack)`, both receive paths call
`revocation::check_acknowledgement(store, sender, ack)` (LLR-5azhry as
amended) with the sender `receive_one` returned, and with no other Device key;
neither path applies LLR-2r2fha's or LLR-kzgjz8's check to an
acknowledgement, whose sender is by design a Device the record no longer
lists. No chain read, no write.
satisfies: REQ-b462sh

## SDD-72ddm6 — The node's own removal (addition)

**LLR-23sfdh**: `StoreData::forget_organisation` is called from exactly two
places — `revocation::accept` on success (LLR-r8qhky) and the one removal
step the commit paths share (`commit_update`, `reconcile`'s commit, and a
committed Organisation-information message on `receive_and_verify` or
`receive_and_self_delete_if_revoked`) when the committed record lists no
DevicePublicKey of a Persona bound to the Organisation (LLR-b27jr6,
LLR-6p4pj2, LLR-fm38ww) — and no other function in org-node removes an
`OrgRecord` or a `PersonaRecord`; `OrgService` has no `forget_organisation`
of its own; the `absences` test reads org-node's source for every other
removal.
satisfies: REQ-em28bq

*Revised 2026-10-07 (re-survey after the key-pair change merged).* The
merged code has a private `OrgService::forget_organisation` called from three
commit paths; this names those paths as the one removal step and removes that
method.
