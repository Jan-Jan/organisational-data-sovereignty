# org-node — the X25519 keys and the Organisation key pair

The owner ruled on 2026-10-04 that a Member-as-a-group key is an X25519 key
and that the on-chain `orgPubKey` is the public half of an Organisation key
pair, an X25519 key belonging to org-node (REQ-8jb4ny, REQ-ech45n). On
2026-10-05 the owner ruled that the Envelope carries no signature and that
nothing about its sender is checked: the on-chain Membership root at a newer
epoch is the sole authority (change `worktree-org-node-chain-authority`).

The items of `2026-10-03-decomposition.md` and `2026-10-04-type-safety.md`
that described a signed Envelope, an ed25519 Member-as-a-group key or a sender
check are amended in place there, each with a dated note
(`docs/plans/2026-10-05-switch-trim.md`). This file holds only the items that
are new on change `worktree-person-shared-types`. Each section names the
software item of the decomposition that the items below it refine.

*Rewritten 2026-10-05 by docs/plans/2026-10-05-switch-trim.md.* This file
first held six software items and twenty-six low-level requirements that
superseded master's. The owner ruled that retired behaviour is amended in
place, not superseded, so those items were withdrawn before merge and their
text, where it still holds, folded into the items they replaced.

Safety class C, as for the decomposition this extends.

## Under SDD-sxp8hb (`org-node/src/keys.rs`)

**LLR-3jjgtw**: `OrgPublicKey::parse` accepts a 32-byte value exactly when
`person::x25519::is_valid_public_key` accepts it, and refuses every other value
with `InvalidOrgPublicKey`.
satisfies: REQ-8jb4ny

`OrgPublicKey` is defined in `types.rs` (SDD-swtd3w). This item sits here,
where the Organisation's key pair is built, as it did before the rewrite.

**LLR-98ufry**: an `X25519Keypair`, which holds a member seed or an
Organisation private key, cannot be cloned, overwrites its 32 secret bytes
with zeros when it is dropped, and prints none of them in its `Debug` output.
satisfies: derived

*Added 2026-10-05 by review round 2 (finding-22).* The ed25519 `SigningKey`
the member key used to be zeroized on drop; the `[u8; 32]` that replaced it
was a `Clone` value that was not. Copies `to_seed` returns, and the seeds
persisted as `[u8; 32]` in `PersonaRecord` and `OrgRecord.org_private_key`,
are not covered: PR-hqwpg9. *(Amended at the merge of master `1feb608`:
`to_seed` and `from_seed` are gone (LLR-56hc77); the key pair hands its secret
back as a `MemberSeed` or an `OrgPrivateKey`, and the persisted seeds are held
in those redacted secret types. PR-hqwpg9 is resolved on master. Neither is
wiped on drop; LLR-98ufry still covers only `X25519Keypair`.)*

*Verification of the drop clause (review round 3, finding-1, 2026-10-05).*
That `X25519Keypair`'s `Drop` calls `zeroize` is verified by inspection only
(`impl Drop for X25519Keypair` in `org-node/src/keys.rs`). No safe test can
read the memory of a dropped value, and deleting the call leaves every test
green (the reviewer's mutation M19). The test checks what can be observed:
the type declares the `ZeroizeOnDrop` contract, and an explicit `zeroize`
leaves no secret byte.

*Verification of the clone clause (review round 4, gate notes, 2026-10-05).*
That `X25519Keypair` cannot be cloned is checked when `key_custody` compiles:
a trait probe resolves to `true` for a `Clone` type and `false` otherwise, and
a `const` assertion requires `false` for `X25519Keypair`. Deriving `Clone` on
it fails that target's build (mutation run, error E0080).
`an_x25519_key_pair_cannot_be_cloned` checks the probe against `[u8; 32]` and
`MemberSeed`, which are `Clone`, so a probe that always answered `false` would
fail. `x25519_seed_round_trip_preserves_key` is the normal case: the key pair
hands its secret back only as a seed.

## Under SDD-na9nc3 (`org-node/src/verify.rs`)

**LLR-9f5hmr**: `verify_envelope_against_chain` refuses with
`SeqNotEpoch { seq, epoch }`, after the stale-epoch check and before the root
match, an Envelope whose Sequence number is not the epoch of the Organisation
state the chain reader returned. A refused Envelope leaves the high-water mark
where it was. An accepted one advances the mark to that epoch.
satisfies: REQ-txvtm9

*Added 2026-10-05 by review round 1 (finding-1), on the owner's amendment of
REQ-txvtm9 that day.* `SeqNotEpoch` is a variant of its own, not `StaleSeq`.
`StaleSeq` says the number is at or below the mark. A number above the mark
can still differ from the chain's epoch, and a caller told `StaleSeq` for it
would be told the wrong thing. Every refusal has a distinct variant
(LLR-z8fubr).

## Under SDD-swtd3w (`org-node/src/types.rs`)

**LLR-322xfu**: `OrgPrivateKey` is a secret type with every property
LLR-sz4xhc states of `MemberSeed`, `DeviceSeed` and `OrgSecret`. It is built
infallibly from 32 bytes by `From<[u8; 32]>`, is `Clone` and not `Copy`, keeps
`PartialEq`/`Eq`, has no `Display`, and renders under `Debug` as
`OrgPrivateKey([REDACTED])` whatever bytes it holds. Its bytes leave only
through `expose_secret` or through serialisation as the plain 32 bytes.
`OrgPrivateKey::x25519_keypair` yields the Organisation's X25519 key pair.
That key pair's `org_public_key` is the X25519 public key of those bytes
(RFC 7748), a valid `OrgPublicKey`, and its `org_private_key` hands the same
bytes back as an `OrgPrivateKey`.
satisfies: derived

No requirement on master stated the Organisation private key's type, so this
item is new.

*Re-traced 2026-10-05 by review round 3 (finding-8).* This item said
`satisfies: REQ-ech45n`. REQ-ech45n requires a fresh secret, its public key
published and the secret kept in the encrypted store; it says nothing of
`Clone`, `Copy`, `Display` or a redacted `Debug`, and REQ-y7tsft, which states
the formatting rule for the other secrets, does not name the Organisation
private key. So the item is derived, and assessed in
`org-node/docs/risk/2026-10-05-envelope-authenticity.md`. REQ-ech45n stays
covered by LLR-sj7cd5 and LLR-3fwykc. *(Corrected 2026-10-05 by review round
4, finding-3: this sentence also named LLR-2dvhz8, which was re-traced to
derived in the same round.)*

## Under SDD-8cpyfa (`org-node/src/service.rs`, `receive_and_verify`)

**LLR-rys5nx**: on first admission the new Organisation record's
Organisation public key is the one in the Organisation state read from the
chain in the same operation, never a value carried in the Wire message, and
the record holds no other key of the Organisation: no administrator key, from
an Invite or from anywhere else.
satisfies: derived

*Amended 2026-10-06 (owner ruling of 2026-10-05, change
`worktree-org-node-chain-authority`).* This said the record also holds an
`admin_member_key`, taken from an imported Invite when there was one and
otherwise from the chain's key. The owner ruled that org-node has no
administrator field and that Invites leave org-node; this item now states
what remains, which LLR-xq9nrq states too. It keeps its ID because retiring an
item is not expressible in the trace gate.

*Amended 2026-10-05 (owner answer Q1 of that day to
docs/plans/2026-10-05-switch-trim.md).* This item took `admin_member_key` from
the imported Invite alone, because a first admission then required one, and
superseded LLR-xq9nrq. A first admission no longer needs an Invite
(REQ-xa6smf), so the item names both sources. The Invite's source stays where
there is an Invite; the chain's key is master's source, kept for the case
without one, because the ruling names no other value. LLR-xq9nrq is amended
in place to state that case, and supersedes nothing and is superseded by
nothing. The field stays until chain-authority's change 1 removes every
administrator field.

## The Organisation private key's custody (REQ-ech45n)

*Added 2026-10-05 by review round 1 (finding-9):* no low-level requirement
refined REQ-ech45n.

*Placed 2026-10-05 by review round 3 (finding-9).* These three items sat
under this heading with no software item. Each now sits under the item that
owns its code, and that item traces REQ-ech45n.

### Under SDD-89es4z (`org-node/src/service.rs`, `create_organisation`)

**LLR-sj7cd5**: `create_organisation` refuses with
`Trie(OrgMembersError::DuplicateKey)`, before any provisional update is kept
and with nothing stored or written, an Organisation public key equal to any
Member-as-a-group key or any DevicePublicKey of the genesis record. The
comparison is `OrgPublicKey::ensure_distinct_from` over the record's members.
satisfies: REQ-ech45n

`ensure_distinct_from` is defined in `types.rs` (SDD-swtd3w).

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`, after the
merge of master `5f7c177`).* This said "before the chain is written and with
nothing recorded". `create_organisation` no longer writes the chain or
creates a record; it keeps a genesis provisional update
(`2026-10-06-chain-authority.md`, LLR-s6qnht), so
the refusal comes before that update is kept.

### Under SDD-af5vnt (`org-node/src/store.rs`, `OrgRecord`)

**LLR-3fwykc**: the Organisation private key is held only in
`OrgRecord.org_private_key` of the creating node's record and, until
`commit_genesis` creates that record, in the genesis provisional update
(LLR-qjz3q4); both reach the disk only through the encrypted Persona store.
Every other node's record holds `None` there.
satisfies: REQ-ech45n

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`, after the
merge of master `5f7c177`).* "and, until `commit_genesis` creates that
record, in the genesis provisional update" is added: creation keeps a
provisional update and the record is created only once the genesis verifies
against the chain.

**LLR-2dvhz8**: `OrgRecord`'s `Debug` names the `org_private_key` field and
prints only whether it is set, never the key's bytes.
satisfies: derived

*Re-traced 2026-10-05 (review round 3, as LLR-322xfu):* REQ-ech45n states that
the secret is kept in the encrypted store, not how a record renders, so this
item is derived and assessed in
`org-node/docs/risk/2026-10-05-envelope-authenticity.md`.

## Robustness

Each item has a normal and an abnormal case:

| Item | Normal case | Abnormal case |
|---|---|---|
| LLR-3jjgtw | `org_public_key_accepts_valid_x25519_keys_unchanged`, `chain_state_is_read_into_typed_values` | `org_public_key_refuses_what_the_x25519_rule_refuses`, `chain_state_with_an_invalid_x25519_key_is_refused_as_invalid_org_public_key` |
| LLR-98ufry | `x25519_seed_round_trip_preserves_key` | `x25519_debug_does_not_print_the_secret`; `an_x25519_key_pair_declares_zeroize_on_drop_and_zeroize_clears_its_secret`; `an_x25519_key_pair_cannot_be_cloned`, with the compile-time assertion beside it. The `Drop` call itself is verified by inspection only. |
| LLR-9f5hmr | `happy_path_commits_when_root_matches_chain` | `a_sequence_number_other_than_the_chain_epoch_is_refused` |
| LLR-322xfu | `the_organisation_private_key_is_a_secret_type_and_the_only_way_to_its_key_pair` | the same test's 31-byte decode and boundary secrets; `the_organisation_private_key_and_its_key_pair_never_render_its_bytes` |
| LLR-rys5nx | `a_first_admission_records_the_chains_key_the_secret_and_the_member` | `a_first_admission_records_the_chains_organisation_public_key`: the record holds the chain's Organisation public key and no other key (LLR-xq9nrq); `org_node_has_no_administrator_key`: no record has a field for one |
| LLR-sj7cd5 | a created Organisation's fresh key | its collisions with a genesis key |
| LLR-3fwykc | `the_organisation_private_key_is_kept_only_in_the_encrypted_store`; `a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals` | the same at-rest test's wrong passphrase, which opens nothing; `a_first_admission_records_the_chains_key_the_secret_and_the_member`: a member's record holds `None` |
| LLR-2dvhz8 | `the_organisation_private_key_is_not_in_the_record_debug_output` | `records_and_wire_messages_never_render_secret_bytes` (sentinel bytes, the `{:#?}` form, the record inside `StoreData`); `a_record_debug_says_whether_the_organisation_private_key_is_set` (the unset key of a member's record) |

*Rows for LLR-3fwykc and LLR-2dvhz8 added, and the LLR-98ufry and LLR-rys5nx
rows amended, 2026-10-05 by review round 4 (finding-2, finding-3 and the gate
notes).*

*The LLR-rys5nx and LLR-3fwykc rows amended 2026-10-06 (change
`worktree-org-node-chain-authority`): the tests they named were renamed with
the Invite's removal, and the Invite-key case was deleted with it (T7, T8).*
