# Design — chain authority: provisional updates, the chain-free checks, and expected admissions

Low-level requirements for change `worktree-org-node-chain-authority`, which
refines the owner rulings of 2026-10-05 recorded in
`crates/org-node/docs/requirements/2026-10-06-chain-authority.md`:
an Envelope carries no signature, nothing about its sender is checked, org-node
has no administrator, it builds provisional updates and commits them only once
they verify against the chain, and it no longer writes to the chain. The
twenty-two requirements defined here belong to software items defined in
`2026-10-03-decomposition.md`, each placed under the item that owns the code it
constrains, as `2026-10-04-type-safety.md` does:

- SDD-swtd3w, which owns `error.rs`: an earlier form of LLR-3sddey;
- SDD-na9nc3, which owns `verify.rs`: LLR-fuq379;
- SDD-kwncn7, which owns `transport/wire.rs`: an earlier form of LLR-g3wmen;
- SDD-af5vnt, which owns `store.rs`: earlier forms of LLR-f5wrjx, LLR-m76r4k, LLR-8ru6mv,
  LLR-qw8tyu and LLR-d9x6xf;
- SDD-89es4z, which owns `create_organisation` and now `commit_genesis`:
  earlier forms of LLR-d4gpxm, LLR-gcm7bq and LLR-qqqg4b;
- SDD-rx2yvy, which owns `admit_member` and now `send_update`: earlier forms of LLR-degg8t;
- the item SDD-busuj3 and SDD-kegag2 replaced, which owned the receive-and-commit path and then `commit_update`:
  earlier forms of LLR-76bhw9, LLR-e6w3nr, LLR-q4puca, LLR-34c5y3, LLR-qvp7mq, LLR-g2jeqd,
  LLR-jhy6t6 and LLR-qqqg4b.

No software item is added; the decomposition's count of nineteen stands. Each
owning item's `traces:` line is amended in place in `2026-10-03-decomposition.md`
for the requirements below, with a dated note, and every item whose behaviour
these rulings remove is rewritten there to a testable statement of the new
behaviour (the inventory is in that file's notes dated 2026-10-05 and naming
this change). The chain write that leaves org-node — `chain_write/` and
`ceremony.rs` — is designed in
`crates/on-chain-client/docs/architecture/2026-10-06-chain-write.md`.

All items are class C, the unit's class.

*Revised 2026-10-06 after the merge of master `5f7c177` (the person-types
switch), `d8b9f9b` and `06c357b` into this change.* Master already removed the
Envelope's signature, `BadSignature`, `VerifyContext`'s key, both sender checks
and the requirement for an Invite on a first admission; it made the
Member-as-a-group key and the Organisation public key X25519 keys, gave
genesis a fresh Organisation key pair whose private key only the creating node
keeps (REQ-ech45n), and set every Envelope's Sequence number to the epoch it
is verified against (an earlier form of REQ-dwb9bq). This file was revised against that tree:
earlier forms of LLR-d4gpxm and LLR-gcm7bq then carried the Organisation key pair and the epoch
rule, those of LLR-f5wrjx and LLR-e6w3nr matched an expectation on its invite
identifier as well as its Organisation (REQ-8amu2a as amended 2026-10-06),
that of LLR-3sddey named the refusal REQ-kt877x adds, and four requirements were new:
earlier forms of LLR-g3wmen and LLR-degg8t (the invite identifier on the wire), of LLR-q4puca
(REQ-kt877x) and of LLR-qw8tyu (the genesis provisional update keeps the
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
  *Superseded 2026-10-08 (ruling B, change worktree-org-io-create):*
  `ChainOps` is gone too; org-node reads no chain and every chain-judging
  operation takes the state as a value (an earlier form of LLR-7en38k, in
  `2026-10-08-values.md`). The test stand-in is
  `test_fixtures::ChainSlots`.
- **Provisional updates.** `create_organisation`, `admit_member` and
  `revoke_member` each build and store a `ProvisionalUpdate` and return it;
  none writes to the chain, sends anything or changes a record. The app
  submits the update through on-chain-client and then calls `commit_genesis`
  (with the org id and proxy account the chain write returned) or
  `commit_update`; sending the committed Envelope is `send_update`, a separate
  operation.
- **Epochs.** The contract sets epoch 1 at genesis and moves it by one on each
  update, and an Envelope's Sequence number is the epoch it is verified
  against (an earlier form of REQ-dwb9bq). So a genesis provisional update carries Sequence
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

*Note 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
`ExpectedAdmission` names the Organisation alone, `InviteId` and the Wire
message's `org_secret` and `invite_id` are removed, and `WireMessage` is an
enum of two kinds; see
`2026-10-07-org-key-pair.md` (finalised under its
merge date) and the items amended below.

*Note 2026-10-07 (change `worktree-org-io-commit-workflow`, plan task T12).*
`CommitOutcome` gains two fields: `revocations: Vec<RevocationNotice>`
(an earlier form of LLR-degg8t) and `acknowledgements: Vec<Acknowledgement>`, the
acknowledgements a commit that removed this node signed (an earlier form of LLR-jhy6t6), empty
otherwise. See that change's design ledger
(`2026-10-07-commit-workflow.md`, finalised under
its merge date).

## SDD-swtd3w — Value types and the rejection vocabulary

`crates/org-node/src/error.rs`: the refusals the provisional-update and
expected-admission paths add.

## SDD-na9nc3 — Verify-against-chain

`crates/org-node/src/verify.rs`: the chain-free half of the check, exposed so that a
caller can run it before it reads the chain.

**LLR-fuq379**: `verify::check_chain_free(local_trie, envelope, ctx) ->
Result<(), OrgNodeError>` runs, in this order and touching no `ChainReader`,
the Organisation check (`OrgIdMismatch`), the Sequence number check
(`StaleSeq`), the Change set decode (`MalformedDelta`) and the base-root check
(`DeltaBaseMismatch`), and returns `Ok(())` only when all four pass.
[Struck 2026-10-30, see the note below: ~~`verify_envelope_against_chain`
runs exactly these checks first and calls `ChainReader::get_org_state` once,
and only after they have passed.~~]
satisfies: REQ-f2k4tr

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1-N1, owner ruling 2026-10-29), a narrowing:* the clause after the
semicolon is struck. `verify_envelope_against_chain` is deleted, and the
receive path, which takes no chain state, calls `check_chain_free` before
anything is kept (LLR-76bhw9, LLR-e6w3nr). The four checks and their order
are unchanged, and `VerifyContext` holds the expected Organisation and the
guard alone. Its tests are now receive-path tests (`receive_path.rs`).
*Tidied 2026-10-30 (merge review round 2, finding-7):* the struck clause is
marked inline above, where it stood after a semicolon.

## SDD-kwncn7 — The wire frame and its bound

`crates/org-node/src/transport/wire.rs`. *(It also named `crates/org-node/src/types.rs`
for the invite identifier's type, removed 2026-10-06, change
`worktree-org-node-org-key-pair`, with an earlier form of LLR-g3wmen.)*

## SDD-af5vnt — The encrypted persona store

`crates/org-node/src/store.rs`: the provisional updates and the expected admissions a
Persona store holds, and the bound on the first.

## SDD-89es4z — Persona and Organisation genesis

`crates/org-node/src/service.rs` (`create_organisation`, `commit_genesis`).

## SDD-rx2yvy — Admission

`crates/org-node/src/service.rs` (`send_update`): which Persona's device a committed
update goes out under, and what it carries.

## The receive-and-commit path

`crates/org-node/src/service.rs` (`receive_and_verify`, `commit_update`, and the
commit step `receive_and_self_delete_if_revoked` shares with them).
