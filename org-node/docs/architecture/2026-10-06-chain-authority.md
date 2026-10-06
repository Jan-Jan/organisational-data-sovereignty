# Design — chain authority: provisional updates, the chain-free checks, and expected admissions

Low-level requirements for change `worktree-org-node-chain-authority`, which
refines the owner rulings of 2026-10-05 recorded in
`org-node/docs/requirements/2026-10-06-chain-authority.md`:
an Envelope carries no signature, nothing about its sender is checked, org-node
has no administrator, it builds provisional updates and commits them only once
they verify against the chain, and it no longer writes to the chain. The
twenty-two requirements defined here belong to software items defined in
`2026-10-03-decomposition.md`, each placed under the item that owns the code it
constrains, as `2026-10-04-type-safety.md` does:

- SDD-swtd3w, which owns `error.rs`: LLR-mxskg9;
- SDD-na9nc3, which owns `verify.rs`: LLR-fuq379;
- SDD-kwncn7, which owns `transport/wire.rs`: LLR-ms8njy;
- SDD-af5vnt, which owns `store.rs`: LLR-95753m, LLR-jq7qh7, LLR-nvn3wk,
  LLR-qjz3q4 and LLR-7cmp38;
- SDD-89es4z, which owns `create_organisation` and now `commit_genesis`:
  LLR-s6qnht, LLR-wzqqg9 and LLR-6z5xya;
- SDD-rx2yvy, which owns `admit_member` and now `send_update`: LLR-2xzys9 and
  LLR-48jakr;
- SDD-8cpyfa, which owns the receive-and-commit path and now `commit_update`:
  LLR-9ew26y, LLR-s8xp7m, LLR-3f5h7b, LLR-cmdrp9, LLR-ewkg85, LLR-4tcxsu,
  LLR-mkj4bz, LLR-b27jr6 and LLR-eyc4ud.

No software item is added; the decomposition's count of nineteen stands. Each
owning item's `traces:` line is amended in place in `2026-10-03-decomposition.md`
for the requirements below, with a dated note, and every item whose behaviour
these rulings remove is rewritten there to a testable statement of the new
behaviour (the inventory is in that file's notes dated 2026-10-05 and naming
this change). The chain write that leaves org-node — `chain_write/` and
`ceremony.rs` — is designed in
`on-chain-client/docs/architecture/2026-10-06-chain-write.md`.

All items are class C, the unit's class.

*Revised 2026-10-06 after the merge of master `5f7c177` (the person-types
switch), `d8b9f9b` and `06c357b` into this change.* Master already removed the
Envelope's signature, `BadSignature`, `VerifyContext`'s key, both sender checks
and the requirement for an Invite on a first admission; it made the
Member-as-a-group key and the Organisation public key X25519 keys, gave
genesis a fresh Organisation key pair whose private key only the creating node
keeps (REQ-ech45n), and set every Envelope's Sequence number to the epoch it
is verified against (REQ-txvtm9). This file was revised against that tree:
LLR-s6qnht and LLR-wzqqg9 now carry the Organisation key pair and the epoch
rule, LLR-95753m and LLR-s8xp7m match an expectation on its invite
identifier as well as its Organisation (REQ-8amu2a as amended 2026-10-06),
LLR-mxskg9 names the refusal REQ-kt877x adds, and four requirements are new:
LLR-ms8njy and LLR-48jakr (the invite identifier on the wire), LLR-3f5h7b
(REQ-kt877x) and LLR-qjz3q4 (the genesis provisional update keeps the
Organisation private key until it commits).

## The shape of the change, for an implementer

- **Envelope.** Done on master: `Envelope` has three fields and no signature,
  `VerifyContext` holds no key, and nothing checks a sender.
- **No administrator.** `OrgRecord.admin_member_key`, `admin_persona_for_org`,
  `blobs.rs` (`Invite`, `JoinRequest`, `encode`, `decode`,
  `decode_join_request`), `PendingInvite`, `StoreData.pending_invites`,
  `list_pending_invites`, `export_invite`, `import_invite`,
  `export_join_request` and `import_join_request` are removed. The joiner's
  details reach `admit_member` as a `Joiner` of parsed values, which the app
  builds from the Invite reply it parsed.
- **No chain write.** `ChainOps` keeps `read_state` only; `submit_genesis`,
  `submit_update`, `SubxtChainOps`'s write half, `FinalitySink`,
  `chain_write/` and `ceremony.rs` leave org-node, and with them the sr25519
  multisig key and the `subxt-signer`, `blake2` and `parity-scale-codec`
  dependencies. `MockChainOps` keeps a test stand-in for the chain write
  (`apply_genesis`, `apply_update`) so a story can be exercised without a
  chain.
- **Provisional updates.** `create_organisation`, `admit_member` and
  `revoke_member` each build and store a `ProvisionalUpdate` and return it;
  none writes to the chain, sends anything or changes a record. The app
  submits the update through on-chain-client and then calls `commit_genesis`
  (with the org id and proxy account the chain write returned) or
  `commit_update`; sending the committed Envelope is `send_update`, a separate
  operation.
- **Epochs.** The contract sets epoch 1 at genesis and moves it by one on each
  update, and an Envelope's Sequence number is the epoch it is verified
  against (REQ-txvtm9). So a genesis provisional update carries Sequence
  number 1, an update built on a record at epoch E carries E + 1, and every
  record's Sequence-number mark equals its epoch: `commit_genesis` writes 1,
  where master's `create_organisation` wrote 0.
- **Expected admissions.** `expect_admission(org_id, invite_id)` records that
  the app expects a first admission to that Organisation carrying that invite
  identifier; the admission's Wire message carries the identifier; a first
  admission whose Organisation and identifier match no expectation is refused
  before the chain is read, and one that verifies is committed only if the
  verified record lists one of the node's own Personas (REQ-kt877x).

The types this change adds, in `store.rs` unless named otherwise:

    ProvisionalUpdate {
        org_id: Option<OrgId>,          // None for genesis: no org id until the chain write
        persona_id: PersonaId,          // the Persona that built it
        base_root: Option<RootHash>,    // the record's root it applies to; None for genesis
        resulting_root: RootHash,       // the Membership root it produces
        seq: SequenceNumber,            // the epoch it produces: the record's epoch + 1; 1 for genesis
        org_pub_key: OrgPublicKey,      // the value the app publishes with the root
        change: ProvisionalChange,
    }
    ProvisionalChange::Genesis { members: Vec<MemberSnapshot>, org_private_key: OrgPrivateKey }
    ProvisionalChange::ChangeSet { change_set: Vec<u8> }   // postcard of the Delta
    ExpectedAdmission { org_id: OrgId, invite_id: InviteId }
    types::InviteId([u8; 32])                              // not secret; the app's Invite carries it
    WireMessage { envelope, org_secret, genesis_snapshot, invite_id: Option<InviteId> }
    service::Joiner { handle, name, surname, member_key, device_key }
    service::OutgoingUpdate { envelope: Envelope, record_snapshot: Vec<u8> }
    service::CommitOutcome { org_id, epoch, root, outgoing: OutgoingUpdate }

## SDD-swtd3w — Value types and the rejection vocabulary

`org-node/src/error.rs`: the refusals the provisional-update and
expected-admission paths add.

**LLR-mxskg9**: `OrgNodeError` gains four variants:
`ProvisionalLimit { limit: usize }`, whose `Display` names the limit in bytes
("provisional updates for one Organisation would exceed {limit} bytes");
`AdmissionNotExpected { org_id: OrgId }`, for a first admission whose
Organisation and invite identifier match no expectation the app declared;
`AdmissionNotOurs { org_id: OrgId }`, for a first admission whose verified
Membership record lists none of the node's own Personas; and
`NoProvisionalUpdate`, for a commit whose Organisation state on the chain
carries a Membership root (and, for genesis, an Organisation public key) that
no stored provisional update produces. Each is distinct from every other
variant.
satisfies: REQ-fwfku9, REQ-8amu2a, REQ-kt877x, REQ-tqap3r, REQ-yp75u9

*Amended 2026-10-06 (merge of master `5f7c177`).* This item also removed
`BadSignature`, which master removed; and it gains `AdmissionNotOurs` for
REQ-kt877x.

*Amended 2026-10-06 (independent review round 1, finding-1; REQ-yp75u9).* It
gains a fifth variant, `PersonaAlreadyBound { persona_id: PersonaId }`, whose
`Display` names the Persona, for a Persona already bound to an Organisation
that `create_organisation` or `commit_genesis` is asked to bind to another
(LLR-6z5xya, LLR-eyc4ud); it is distinct from every other variant, and this
item now also satisfies REQ-yp75u9.

## SDD-na9nc3 — Verify-against-chain

`org-node/src/verify.rs`: the chain-free half of the check, exposed so that a
caller can run it before it reads the chain.

**LLR-fuq379**: `verify::check_chain_free(local_trie, envelope, ctx) ->
Result<(), OrgNodeError>` runs, in this order and touching no `ChainReader`,
the Organisation check (`OrgIdMismatch`), the Sequence number check
(`StaleSeq`), the Change set decode (`MalformedDelta`) and the base-root check
(`DeltaBaseMismatch`), and returns `Ok(())` only when all four pass;
`verify_envelope_against_chain` runs exactly these checks first and calls
`ChainReader::get_org_state` once, and only after they have passed.
satisfies: REQ-f2k4tr

## SDD-kwncn7 — The wire frame and its bound

`org-node/src/transport/wire.rs` and `org-node/src/types.rs` (`InviteId`).

**LLR-ms8njy**: `WireMessage` carries, after its record snapshot, an
`invite_id: Option<InviteId>`, where `InviteId` is exactly 32 bytes (it is
not secret, and travels in plain text in the app's Invite); a frame round-trips its
invite identifier unchanged (LLR-er2x8n), and a body whose invite identifier
is not 32 bytes does not decode (`Malformed`).
satisfies: REQ-8amu2a

## SDD-af5vnt — The encrypted persona store

`org-node/src/store.rs`: the provisional updates and the expected admissions a
Persona store holds, and the bound on the first.

**LLR-95753m**: `StoreData` holds `provisional_updates: Vec<ProvisionalUpdate>`
and `expected_admissions: Vec<ExpectedAdmission>`, each expectation an
Organisation and an invite identifier, and no pending Invites; both reach the
encrypted file with the rest of the store and are read back when it is opened.
A provisional update is identified by its Organisation — its `org_id`, or for a
genesis update the Persona that built it — together with its `resulting_root`:
`StoreData::insert_provisional` replaces a stored provisional update with the
same identity rather than adding a second, and otherwise appends, so several
provisional updates for one Organisation are held at once.
satisfies: REQ-xs4ab8, REQ-8amu2a

*Amended 2026-10-06 (owner ruling on pre-emption, REQ-8amu2a as amended).*
`expected_admissions` held Organisation identifiers alone; an expectation now
names the invite identifier the app's Invite reply carries as well.

**LLR-jq7qh7**: `StoreData::insert_provisional(update) -> Result<(),
OrgNodeError>` measures the postcard encoding of the `Vec<ProvisionalUpdate>`
the Organisation would hold after the insertion — every stored provisional
update with the same `org_id`, or, for a genesis update, every genesis
update of the same Persona, with the new one in place of any it replaces —
and when that length exceeds `MAX_PROVISIONAL_BYTES` (1 MiB, equal to the
transport's `MAX_FRAME`) refuses with `ProvisionalLimit { limit:
MAX_PROVISIONAL_BYTES }` and leaves `StoreData` exactly as it was; the
operation that called it then writes nothing to disk.
satisfies: REQ-fwfku9

**LLR-nvn3wk**: `OrgService::provisional_updates(org_id)` returns the stored
provisional updates whose `org_id` is that Organisation, and
`OrgService::genesis_provisional_updates(persona_id)` those genesis updates
that Persona built, each with its resulting root, base root, Sequence number
and Organisation public key, so the app can submit one after a restart; neither
changes the store.
satisfies: REQ-xs4ab8

**LLR-qjz3q4**: a genesis provisional update holds the Organisation private key
whose X25519 public key is its `org_pub_key`, as an `OrgPrivateKey` inside
`ProvisionalChange::Genesis`; it reaches the disk only inside the encrypted
Persona store, is moved into the new record's `org_private_key` by the
`commit_genesis` that consumes the update, and leaves the store with the update
when the update is removed without being committed. No other provisional
update holds a private key.
satisfies: REQ-ech45n

*Added 2026-10-06 (merge of master `5f7c177`).* Master's `create_organisation`
put the Organisation private key straight into the record it created. Creation
now keeps only a provisional update, so REQ-ech45n's "keep that secret in its
encrypted store" is met by the update until `commit_genesis` creates the
record; the owner ruled (2026-10-06) that an update never committed takes the
key with it.

**LLR-7cmp38**: `OrgService::discard_provisional(rng, target, resulting_root)
-> Result<(), OrgNodeError>` (the `rng` seals the store on save, as every
other mutating operation's does), where `target` is `ProvisionalTarget::Org(org_id)`
or `ProvisionalTarget::Genesis(persona_id)`, removes the one stored provisional
update for that target whose `resulting_root` equals `resulting_root` — a
genesis update together with the Organisation private key it holds — and saves
the store before returning; when no such update is stored it returns
`NoProvisionalUpdate`, leaves the store unchanged and writes nothing. It never
touches a record, an expectation or another provisional update.
satisfies: REQ-hhva9d

## SDD-89es4z — Persona and Organisation genesis

`org-node/src/service.rs` (`create_organisation`, `commit_genesis`).

**LLR-s6qnht**: `create_organisation(rng, persona_id) ->
Result<ProvisionalUpdate, OrgNodeError>` builds the genesis Membership record
with the named Persona as its one Member, under a `MemberId` drawn from `rng`,
draws a fresh Organisation key pair from `rng` (REQ-ech45n), refuses one whose
public key equals a key of the genesis record (LLR-sj7cd5), and stores,
through `insert_provisional`, a provisional update with `org_id` `None`,
`base_root` `None`, `seq` 1 (the epoch genesis produces), `resulting_root` the
genesis record's root, `org_pub_key` that key pair's public key and
`ProvisionalChange::Genesis` holding that Member's snapshot and the key pair's
private key (LLR-qjz3q4); it saves the store before returning, and calls no
chain operation.
satisfies: REQ-xs4ab8

*(Merged 2026-10-06 with master `5f7c177`: this read "`org_pub_key` the
founding Member's Member key (`OrgPublicKey::from(&P2pMemberKey)`,
PR-szkat6)". Master's REQ-ech45n gives genesis its own X25519 Organisation key
pair and removes that conversion. Where the genesis provisional update keeps
the Organisation private key until `commit_genesis` creates the record is not
yet stated.)*

*Amended 2026-10-06 (merge of master `5f7c177`, REQ-txvtm9 and REQ-ech45n).*
The update's Sequence number was zero; it is 1, the epoch the contract sets at
genesis, so the record `commit_genesis` creates has a mark equal to its epoch.
The private key's custody is LLR-qjz3q4, which answers the note above.

**LLR-wzqqg9**: `commit_genesis(rng, persona_id, org_id, proxy_account:
ChainAccount) -> Result<ReceiveOutcome, OrgNodeError>` reads the Organisation
state for `org_id` once (`OrgNotOnChain` for none, `Chain` for a failed read),
selects the genesis provisional update the named Persona built whose
`resulting_root` and `org_pub_key` equal the root and key read
(`NoProvisionalUpdate` for none), refuses with `SeqNotEpoch { seq, epoch }` an
epoch other than the update's Sequence number 1, and with `RootMismatch` a
record rebuilt from its members whose root is not the root read, and only then
creates the Organisation record — `org_id`, that root, the Organisation public
key and epoch read, Sequence number 1, no Organisation secret, the members,
the update's Organisation private key, and `proxy_account`
`Some(proxy_account)` — removes that provisional update, and saves the store;
on any refusal the store is unchanged and not written.
satisfies: REQ-tqap3r

*Amended 2026-10-06 (merge of master `5f7c177`, REQ-txvtm9 and REQ-ech45n).*
This refused with `StaleEpoch` an epoch not greater than zero and created the
record with Sequence number zero and no private key. Genesis produces epoch 1
on the contract, so the check now is the epoch rule master applies to every
received Envelope, against the update's Sequence number 1, and the record's
mark is 1. The selection also matches the Organisation public key, so the
private key the record keeps always has the chain's key as its public half.

*Owner ruling 2026-10-06.* When the chain is already past epoch 1 — another
signatory of a 1-of-N multisig updated the Organisation before the founding
node committed its genesis — `commit_genesis` refuses and changes nothing, and
the founding node obtains the current record the way any Member does, from an
update another Member sends it. Its genesis provisional update stays until the
app discards it (LLR-7cmp38). Which refusal it is follows the check order
above: the update is selected by the chain's root and key first, so a chain
whose later updates changed the root (the usual case) gives
`NoProvisionalUpdate`, and one that is past epoch 1 yet carries the genesis
root and key again (an admission later undone) gives `SeqNotEpoch`.
`a_chain_past_epoch_one_refuses_commit_genesis_and_keeps_the_update` asserts
both.

**LLR-6z5xya**: `create_organisation(rng, persona_id)` refuses with
`PersonaAlreadyBound { persona_id }`, naming the Persona, a Persona whose
`org_id` is set, before it draws anything from `rng`; it keeps no provisional
update, changes no Persona or record, and writes nothing to disk. A Persona
whose `org_id` is unset may build any number of genesis provisional updates
(LLR-95753m); LLR-eyc4ud decides which of them can commit.
satisfies: REQ-yp75u9

*Added 2026-10-06 (owner ruling: one Persona, one Organisation; independent
review round 1, finding-1).* Before it a Persona that had founded or joined
one Organisation could found another, and `commit_genesis` moved its binding,
so the first could no longer be managed (PR-mdv38y's second writer).

## SDD-rx2yvy — Admission

`org-node/src/service.rs` (`send_update`): which Persona's device a committed
update goes out under, and what it carries.

**LLR-2xzys9**: `send_update(outgoing, recipient, peer_addr, org_secret,
invite_id)` binds the endpoint, when none is bound, from the device seed of
the first Persona in the store bound to the Organisation
`outgoing.envelope.org_id` names, and refuses with a typed error, sending
nothing, when no Persona is bound to it; an endpoint already bound is used as
it is (LLR-6zjzn2, PR-8qsnhx).
satisfies: derived

*Amended 2026-10-06:* the signature gains `invite_id` (LLR-48jakr).

**LLR-48jakr**: the Wire message `send_update` sends carries in `invite_id`
exactly the invite identifier its caller passes — the Invite reply's for an
admission, none for any other update — and `send_update` neither reads nor
stores an invite identifier.
satisfies: REQ-8amu2a

## SDD-8cpyfa — The receive-and-commit path

`org-node/src/service.rs` (`receive_and_verify`, `commit_update`, and the
commit step `receive_and_self_delete_if_revoked` shares with them).

**LLR-9ew26y**: on a Wire message about an Organisation the node holds a record
of, `receive_and_verify` and `receive_and_self_delete_if_revoked` call
`verify::check_chain_free` against that record before calling
`ChainOps::read_state`, call `read_state` at most once, and on any refusal of
`check_chain_free` return that error with no `read_state` call and the record,
the provisional updates and the expected admissions unchanged; the early
`read_state` that existed to obtain a signing key is removed.
satisfies: REQ-f2k4tr

**LLR-s8xp7m**: on a Wire message about an Organisation the node holds no
record of, `receive_and_verify` refuses with `AdmissionNotExpected { org_id }`,
before decoding the record snapshot and with no `read_state` call, unless
`expected_admissions` holds an expectation whose Organisation is `org_id` and
whose invite identifier is the Wire message's `invite_id` (a message carrying
none matches nothing); and when one does, decodes the snapshot
(`first_admission_base`), runs `check_chain_free` against it, and only then
reads the chain once. Every refusal leaves every expectation in place.
satisfies: REQ-8amu2a

*Amended 2026-10-06 (owner ruling on pre-emption, REQ-8amu2a as amended).*
The expectation matched on the Organisation alone.

**LLR-3f5h7b**: on a first admission that has verified against the chain,
`receive_and_verify` commits only when the verified Membership record holds the
DevicePublicKey of at least one Persona in the store; otherwise it refuses with
`AdmissionNotOurs { org_id }`, creates no record, marks no Persona, leaves every
expectation in place and writes nothing to disk.
satisfies: REQ-kt877x

*Added 2026-10-06 (owner ruling on pre-emption).* Without it a relay holding a
genuine admission of someone else to an Organisation this node expects could
make the node commit a record it is not in.

*Amended 2026-10-06 (REQ-yp75u9, LLR-eyc4ud).* "At least one Persona in the
store" reads "at least one Persona in the store bound to no Organisation": a
record that lists only Personas bound elsewhere is refused with
`AdmissionNotOurs` like one that lists none.

**LLR-cmdrp9**: `commit_update(rng, org_id) -> Result<CommitOutcome,
OrgNodeError>` reads the Organisation state for `org_id` once, selects the
stored provisional update for `org_id` whose `resulting_root` equals the root
read (`NoProvisionalUpdate` for none), builds from it the Envelope `(org_id,
seq, change_set)`, and runs `verify_envelope_against_chain` on it against the
record, the record's Sequence-number mark and last committed epoch, and a
reader serving that one state — the checks a received update passes, the
epoch rule (REQ-txvtm9) included — and on success commits as a received update
commits (root, epoch, Sequence number and members together, store saved) and
returns the Envelope and the encoded snapshot of the record as it stood before
the commit as `outgoing`.
satisfies: REQ-tqap3r

**LLR-ewkg85**: a `commit_update` or `commit_genesis` refused at any step —
no record, a failed or empty chain read, no matching provisional update, or
any refusal of verify-against-chain — returns the typed error and leaves the
record, every provisional update and the Persona records exactly as they were,
with nothing written to disk.
satisfies: REQ-tqap3r

**LLR-4tcxsu**: `commit_update` and `commit_genesis` bind no endpoint and send
nothing: each returns once the store is saved, so a commit never waits on a
send and its outcome does not depend on whether the committed update is ever
sent.
satisfies: REQ-tqap3r

**LLR-mkj4bz**: whenever an update for an Organisation commits — through
`receive_and_verify`, `receive_and_self_delete_if_revoked`, `commit_update`
or `commit_genesis` — the record's previous root, epoch, Sequence number and
members are replaced, and every stored provisional update for that
Organisation whose `base_root` is not the new record's root is removed, in the
same store save; provisional updates for every other Organisation are left as
they were.
satisfies: REQ-uv3v5w

**LLR-b27jr6**: when a commit on `receive_and_verify` or `commit_update`
produces a Membership record that holds the DevicePublicKey of no Persona bound to
that Organisation, the node deletes its record of that Organisation, removes
every provisional update for it and marks every Persona bound to it Revoked,
as `receive_and_self_delete_if_revoked` does (LLR-6p4pj2), instead of
committing the update.
satisfies: REQ-uxv2x2

**LLR-eyc4ud**: no commit path — `commit_genesis`, `commit_update`,
`receive_and_verify`, `receive_and_self_delete_if_revoked` — changes the
`org_id` of a Persona whose `org_id` is set, and none sets it to anything but
the Organisation committed. `commit_genesis` refuses with
`PersonaAlreadyBound { persona_id }` a named Persona whose `org_id` is set,
before it reads the chain, keeping the genesis provisional update and changing
and writing nothing (LLR-ewkg85). `receive_and_verify` chooses the Persona it
marks Active, and gives the member id of the member holding its
DevicePublicKey, as the first Persona in store order whose DevicePublicKey the
verified Membership record lists, among the Personas this commit may bind: on
a first admission only those whose `org_id` is unset, which it binds to the
Organisation; on an update to a held record only those bound to that
Organisation, whose binding it leaves as it is. A first admission whose record
lists the DevicePublicKey of no unbound Persona — none at all, or only of
Personas bound to other Organisations — is refused with
`AdmissionNotOurs { org_id }` (LLR-3f5h7b), creating no record, marking no
Persona, keeping every expectation and writing nothing. `commit_update` and
`receive_and_self_delete_if_revoked` write no binding; forgetting an
Organisation (LLR-b27jr6, LLR-6p4pj2) marks its Personas Revoked and leaves
their `org_id` as it was.
satisfies: REQ-yp75u9

*Added 2026-10-06 (owner ruling: one Persona, one Organisation; independent
review round 1, finding-1).* `commit_genesis` and both branches of
`receive_and_verify` overwrote the binding of whichever Persona they chose,
whatever Organisation it was bound to (PR-mdv38y). A first admission that may
bind no Persona is refused rather than committed, because a record the node
holds through no Persona of its own is one it cannot act for: the next update
for it would read as the node's removal (LLR-b27jr6).

