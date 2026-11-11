# Design — the Organisation key pair reaches every Member

Low-level requirements for change `worktree-org-node-org-key-pair`, change 3
of the chain-authority sequence. They refine the owner rulings and the
requirements of
`crates/org-node/docs/requirements/2026-10-07-org-key-pair.md`
(REQ-szq3ud, REQ-jy6ybw, REQ-ju6vn2, REQ-3dsweu, REQ-vxqc5g and earlier forms
of REQ-3pj7rd, REQ-zxgd46 and REQ-dwb9bq), and
REQ-8amu2a as amended in `2026-10-06-chain-authority.md`. They realise the
receipt check of an earlier form of RC-5gbgwb
(`crates/org-node/docs/risk/2026-10-07-org-key-pair.md`).

No software item is added; the decomposition's count of nineteen stands. Each
requirement below sits under the item that owns the code it constrains, as
`2026-10-06-chain-authority.md` does, and each owning item's `traces:` line is
amended in place in `2026-10-03-decomposition.md` with a dated note:

- SDD-kwncn7, which owns `transport/wire.rs`: LLR-ecxc76 and an earlier form of LLR-xr88a8;
- SDD-swtd3w, which owns `error.rs` and `types.rs`: LLR-qsjde3 and an earlier form of LLR-3sddey;
- SDD-af5vnt, which owns `store.rs`: an earlier form of LLR-78jfrd;
- SDD-rx2yvy, which owns `admit_member`, `keep_change_set` and
  `send_update`: earlier forms of LLR-degg8t and LLR-qghs95. The latter also
  constrained `revoke_member`, owned by an earlier form of SDD-z4m57w;
- the item SDD-busuj3 and SDD-kegag2 replaced, which owned the receive-and-commit
  path and `commit_update`: earlier forms of LLR-94rzrk, LLR-8528ud, LLR-95av3e,
  LLR-jhy6t6 and LLR-34c5y3. Those of LLR-8528ud, LLR-jhy6t6 and LLR-34c5y3 also
  constrained `receive_and_self_delete_if_revoked`, owned by an earlier form of
  SDD-z4m57w.

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

`crates/org-node/src/transport/wire.rs`.

**LLR-ecxc76**: the `Debug` rendering of a `WireMessage` of either variant
contains none of the bytes of the Organisation private key it holds, in any
form: an `OrgInformation` renders its key as `OrgPrivateKey([REDACTED])`
(LLR-322xfu), and a `Revocation` holds no key. `WireMessage` formats no field
itself.
satisfies: REQ-y7tsft

*Amended 2026-10-08 (S4, task T2):*
code path only. `WireMessage` is defined in `crates/org-node/src/wire.rs`, out of
`transport/`; the rule is unchanged.

## SDD-swtd3w — Value types and the rejection vocabulary

`crates/org-node/src/error.rs`, `crates/org-node/src/types.rs`.

**LLR-qsjde3**: org-node defines no `OrgSecret` type and re-exports none: no
record, store plaintext, Wire message, provisional update or `OrgService`
operation holds, takes or returns an Organisation secret, and the only
Organisation key material org-node holds is the `OrgPrivateKey`.
satisfies: REQ-szq3ud

## SDD-af5vnt — The encrypted persona store

`crates/org-node/src/store.rs`.

## SDD-rx2yvy — Admission

`crates/org-node/src/service.rs` (`keep_change_set`, which `admit_member` and
`revoke_member` share, and `send_update`).

## The receive-and-commit path

`crates/org-node/src/service.rs` (`receive_one`, `receive_and_verify`,
`receive_and_self_delete_if_revoked`, `commit_update`).
