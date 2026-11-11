# org-node — software decomposition, items and low-level requirements

The first architecture ledger for the `org-node` unit: nineteen software items
and the low-level requirements that refine the **twenty** high-level
requirements in `crates/org-node/docs/requirements/`. The unit's SOUP inventory is
`crates/org-node/docs/architecture/soup.md`.

*Corrected 2026-10-04 by review round 2. This sentence read "nineteen" against
a unit that holds twenty, and the one it omitted was the one that mattered:
**REQ-d9g6nt** arrived with `master` `0f85cb9` while this change was open, is
verified directly by `fuzz_first_admission_base` and four `admission_sender`
tests, and was traced by no item here and refined by no low-level requirement.
No gate could have caught that — `check-trace.sh` has an UNTRACED-DESIGN rule
and no UNTRACED-REQUIREMENT rule — so this ledger was the only place it could
be. It is now refined by an earlier form of LLR-4rv4kr, and by LLR-rjg3m2 and LLR-j6j95z, below. Two of
the twenty are traced by nothing by design: org-io's finalised-block expectation (moved 2026-10-07) and REQ-q92yac are
expectations on provider units, not behaviours of this one.*

Safety class C throughout (`crates/org-node/.guardrails/config.yaml`, no per-item
override), so every item below carries LLRs — **with one exception, org-io's chain-connection item (moved 2026-10-07),
which carries none.** That is a deviation, not an override, and the section
"The item with no low-level requirements" at the end of this file records it in
full rather than leaving a reader to notice the absence. *(Amended 2026-10-07,
change `worktree-org-io-create`: that item and its deviation moved to org-io
by ruling A, so every item left here carries LLRs and the exception is
org-io's.)*

*Re-parented 2026-10-08 (change `worktree-org-io-create`, owner's hybrid
ruling on the resolved problem report on amendment in place instead of supersession).* An earlier form of REQ-uk9rw7 is superseded by REQ-uk9rw7, which states
the member-sender rule that earlier form carried after its 2026-10-07 amendment. The
items below that named the earlier form on their `satisfies:` or `traces:` line now
name REQ-uk9rw7 in its place, with no other change: the authenticated
endpoint's design item with its endpoint-identity and authenticated-sender
items, SDD-rx2yvy's line (which also named earlier forms of REQ-eqt23j and
REQ-dwb9bq), the Loopback dial item, the lines of the items SDD-busuj3, SDD-kegag2
and SDD-z4m57w replaced, earlier forms of LLR-283x6f, and the endpoint lifecycle's design item with its bind-once and
named-Persona items. *(2026-10-13, S4 task T10: the endpoint and lifecycle
items moved to org-io, `crates/org-io/docs/architecture/2026-10-07-transport.md`,
re-parented to org-io's requirements; S4 task T12: the Loopback dial item
followed, the same way.)* Prose that cites
that earlier form "as amended" is history and is left as written.

*Amended 2026-10-08 (ruling B, change `worktree-org-io-create`).* "The seam"
below says this unit "performs the chain and peer-to-peer I/O" and that
`on-chain-client` supplies the chain reading, and its item count names the
`ChainOps` seam. org-node now performs no chain I/O: org-io owns the
`OrgService`, the chain connection, the reader, the writer and the user's
signatory key, reads each Organisation's state and passes it in as a value
(`2026-10-08-values.md`, an earlier form of LLR-7en38k). The peer-to-peer I/O
stays here until stage S4. The text below is kept as the record of the
decomposition it described.

## Overview

The unit's overview — the decomposition picture and the segregation position —
lives in the Overview section of `crates/org-node/docs/architecture/README.md`, which
is the standing description every later change amends, rather than in this
per-change file.

## The seam

This unit is the node. It holds the keys, decides whether a received change to
the membership of an Organisation is one it will commit, and performs the chain
and peer-to-peer I/O through which a member's access is granted or withdrawn.
Everything else in this repository either supplies it (`org-members` the trie,
`on-chain-client` the chain reading) or drives it (`app`).

The nineteen items divide it along **what has to be true before a received
change is committed**: what a value is typed as, who holds which key, what the
authenticated wire form binds together, what refuses a replay, what performs
the decisive check, what the chain is allowed to be asked, how bytes reach the
node at all, where secrets rest, and how the five user stories compose those
parts.

Seven of the items live wholly or partly in `service.rs`, which is 1571 lines —
forty-two percent of the unit *(1501 lines, thirty-four per cent of a unit of
4391 lines over 25 files, after the org-node type-safety change, measured
2026-10-05 at its merge of this file; the change added `types.rs`)*.
*(Amended 2026-10-05 by the org-node type-safety change, review round 7: this
said the change "removed `service.rs`'s test module". `master` `05f6f04` had
already relocated it — `service.rs` on `master` is 1571 lines with no
`#[cfg(test)]` module. The drop to 1501 is that change's own refactor: the
member-snapshot construction deduplicated into one `snapshot_of`, the
`VerifyingKey::from_bytes` re-parsing of keys that now arrive typed removed,
and the chain-state mapping moved to `chain_read::org_state_from_chain`.)*
An item here is a responsibility with an interface,
not a file: `receive_and_verify` and `revoke_member` are separate items that
happen to be methods on one struct, and the `ChainOps` seam is a third that
exists precisely so the first two can be exercised without a chain. *(This
said "Eight". Seven items name `service.rs` as their location: SDD-ueh4tm,
SDD-89es4z, SDD-rx2yvy, the items SDD-busuj3 and SDD-kegag2 and SDD-z4m57w replaced, the endpoint lifecycle's item (moved to org-io 2026-10-13) and org-io's chain-connection item (moved 2026-10-07).
Counted by review round 7.)*

**The decisive property of the unit is one function.**
`verify_envelope_against_chain` is forty-two lines and performs seven checks in a
security-critical order; SDD-na9nc3 is the item that owns it, and eleven
low-level requirements refine that one ordering — ten in this file and
LLR-fuq379 in `2026-10-06-chain-authority.md`. If
a reader has time for one item, it is that one.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said "eight checks" and "ten of this ledger's low-level requirements".
The signature check is removed — an Envelope carries no signature, and the
on-chain Membership root at a newer epoch is the sole authority — so seven
remain: the Organisation, the Sequence number, the Change set decode, the base
root, the apply, the chain read with its epoch, and the root match. An earlier form of LLR-283x6f
stated the absence of the signature check, and LLR-fuq379 states that the
four chain-free checks run before the chain is read.

## SDD-swtd3w — Value types and the rejection vocabulary

`crates/org-node/src/ids.rs`, `crates/org-node/src/error.rs`, `crates/org-node/src/types.rs`

**SDD-swtd3w**: the identifier that keys an Organisation's on-chain slot,
together with the vocabulary in which every refusal is reported — a typed
variant per rejection path **on the receive-and-commit path**, so a caller can
state *why* a change was refused rather than only that it was.
traces: REQ-gju89b, REQ-9g6as6, REQ-rv4dwb, REQ-y7tsft, REQ-8jb4ny, REQ-ech45n, REQ-g2537c, REQ-8amu2a, REQ-tqap3r, REQ-kt877x, REQ-yp75u9, REQ-zxgd46, REQ-dwb9bq, REQ-vxqc5g, REQ-szq3ud, REQ-3dsweu, REQ-m2xh8q, REQ-b462sh

*Traces amended 2026-10-20 (change worktree-org-io-domain-api): REQ-g2537c and
REQ-dwb9bq replace the earlier forms the item traced.* The
vocabulary's additions and removals of that change are LLR-3sddey's, which
supersedes the earlier form of LLR-3sddey.

*Amended 2026-10-06 (change `worktree-org-io-commit-workflow`, stage S3).* Traces REQ-m2xh8q, REQ-b462sh and an earlier form of REQ-dwb9bq for the eight refusals of
an earlier form of LLR-3sddey, in `2026-10-07-commit-workflow.md`.
*(Merged 2026-10-07 with the key-pair change's note below, at this branch's
merge of master `1f52c36`: the trace list is the union of both.)*

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
`types.rs` loses `OrgSecret` and `InviteId` (LLR-qsjde3 and an earlier form of LLR-g3wmen), so the
secrets it holds are the member seed, the device seed and the Organisation
private key; and `OrgNodeError` gains `MalformedMessage`, `OrgKeyMismatch`,
`RevocationNotHeld` and `RevocationNotForThisDevice` (an earlier form of LLR-3sddey, in
`2026-10-07-org-key-pair.md`). The item traces
REQ-zxgd46, REQ-dwb9bq, REQ-vxqc5g, REQ-szq3ud and REQ-3dsweu too (the first two then in earlier forms). Notes below that list
the Organisation secret among `types.rs`'s secrets are history.

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`, independent
review round 1, finding-1):* traces REQ-yp75u9 too, for the refusal
`PersonaAlreadyBound` that an earlier form of LLR-3sddey added.

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`):* traces
REQ-kt877x too, for the refusal `AdmissionNotOurs` that an earlier form of LLR-3sddey added.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
`OrgNodeError::BadSignature` is removed with the Envelope signature, and the
item gains the refusals of the provisional-update and expected-admission paths
— `ProvisionalLimit`, `AdmissionNotExpected` and `NoProvisionalUpdate` — stated
by an earlier form of LLR-3sddey in `2026-10-06-chain-authority.md`,
so it traces REQ-g2537c (then in an earlier form), REQ-8amu2a and REQ-tqap3r too.

*Amended 2026-10-05 by the org-node type-safety change, which re-homed its
value types here at its merge of this file.* The item also owns
`crates/org-node/src/types.rs`: the secrets (member seed, device seed, Organisation
secret), the Organisation public key and the tag types for the chain account,
the Persona identifier, the epoch and the Sequence number, with their
construction, formatting and serialised form — so it traces REQ-y7tsft too —
and the two variants that change adds to `OrgNodeError`, `InvalidKey` and
`InvalidField { field, reason }`. Its low-level requirements for them,
LLR-sz4xhc, LLR-s7whrn and an earlier form of LLR-mkvx2k, are in `2026-10-04-type-safety.md`.
The count of `OrgNodeError::Chain(String)` construction sites below is fifty
on the tree it was measured on; after that change it is **forty-five**
(`grep -o 'OrgNodeError::Chain(' -r crates/org-node/src | wc -l`, 2026-10-05), the
five removed being refusals that now name the field (`InvalidField`) or the
key (`InvalidKey`). The catch-all is otherwise unchanged.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
`types.rs` is this item's alone. LLR-mmdu38 — `OrgPublicKey`'s parse, and
what the chain read view serves when that parse refuses a fetched state — now
sits under this item in `2026-10-04-type-safety.md`, because four of its five
clauses constrain `OrgPublicKey` in `types.rs`; its cache clause is named by
SDD-pa6p7w. It had been placed under SDD-pa6p7w, whose section claimed
`types.rs` for it.)*

*(Note 2026-10-05, at the merge of master `1feb608` into
`worktree-person-shared-types`; reworded the same day: on that branch `types.rs` also holds
`OrgPrivateKey`, and `OrgPublicKey` is an X25519 key. LLR-mmdu38 and
an earlier form of LLR-mkvx2k are amended in place, in `2026-10-04-type-safety.md`. LLR-322xfu
(`OrgPrivateKey`), LLR-3jjgtw (`OrgPublicKey`'s parse) and an earlier form of LLR-9qavm7
(`ensure_distinct_from`) also constrain `types.rs`. These three are in
`2026-10-05-unsigned-envelope.md`, LLR-3jjgtw under SDD-sxp8hb, which
builds the Organisation's key pair.)*

*(Re-traced 2026-10-05 by review round 3, finding-9, change
`worktree-person-shared-types`: `types.rs` holds `OrgPublicKey`, whose parse
refuses a key that is not a valid X25519 key, and `OrgPrivateKey`, so the item
traces REQ-8jb4ny and REQ-ech45n. LLR-322xfu sits under this item in
`2026-10-05-unsigned-envelope.md`; it is derived since that round.)*

*Qualified 2026-10-04 by review round 3, with LLR-z8fubr below and for the same
measurement: "one typed variant per rejection path" is true of
verify-against-chain and false of the unit. Fifty sites outside it construct
`OrgNodeError::Chain(String)`, which renders every one of them as "chain read
failed: …" whatever actually went wrong.*

**LLR-7gnrnz**: `OrgId` wraps exactly twenty bytes and round-trips through
postcard unchanged.
satisfies: REQ-gju89b

**LLR-gu6u53**: `OrgId`'s `Debug` renders `OrgId(0x` followed by its twenty
bytes as forty lowercase hexadecimal digits, so an identifier in a log is
readable as the contract storage key it is.
satisfies: derived

**LLR-z8fubr**: each rejection path **of verify-against-chain** yields its own
`OrgNodeError` variant, distinct from every other, and the two that carry the
values that caused them — `StaleSeq` and `StaleEpoch` — carry both the observed
value and the value it was judged against.
satisfies: REQ-9g6as6, REQ-rv4dwb

*Narrowed 2026-10-04 by review round 3, because the clause as written was
**false of the tree**. It said "each rejection path of this unit", and the unit
has **fifty** `OrgNodeError::Chain(String)` construction sites carrying about
thirty-five distinct messages behind one variant whose `Display` is
`"chain read failed: {0}"`. A wrong store passphrase, a malformed invite blob,
a missing persona, a Loopback revocation with no peer address and an iroh send
failure are all the same variant as one another and as an actual chain read
failure. `crates/org-node/src/error.rs`'s own doc comment has always carried the
correct, narrower statement — "Every rejection path **in verify-against-chain**
maps to a distinct variant" — and this ledger generalised it to the whole unit
without checking. The narrowed clause is what the ten declared variants
actually give.

**The `Chain(String)` catch-all is now stated as what it is**, in SDD-swtd3w's
item text and in the SOUP inventory's `thiserror` row, rather than contradicted
by them. It is a real design gap — a caller cannot distinguish "the chain did
not answer" from "your passphrase is wrong" — and it is recorded in the Gaps of
this change's verification record rather than closed here, because collapsing
fifty sites into typed variants is a change to the production error surface
with its own review.*

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The requirement's text stands; what it ranges over changes. `BadSignature`,
the variant of the signature check, is removed with that check, so the
rejection paths of verify-against-chain are now `OrgIdMismatch`, `StaleSeq`,
`MalformedDelta`, `DeltaBaseMismatch`, `Trie` (the apply), `OrgNotOnChain`,
`Chain`, `StaleEpoch` and `RootMismatch`, and the distinctness test drops the
one it no longer has.

**LLR-7cgg8a**: an `org_members::OrgMembersError` is carried into
`OrgNodeError::Trie` without loss, so a provider's refusal reaches the caller
as the provider stated it.
satisfies: REQ-9g6as6

## SDD-sxp8hb — Device key and Member-as-a-group key custody

`crates/org-node/src/keys.rs`

**SDD-sxp8hb**: the key pairs a node holds and how each is made from its
secret: the ed25519 device keypair, whose verifying key is the node's
DevicePublicKey in the trie and its transport identity, and the X25519 key
pairs behind a Member-as-a-group key and behind the Organisation public key,
each from its own secret (the member seed and the Organisation private key).
The Organisation public key a node publishes is built from its key pair only
through `OrgPublicKey`'s parse, `person`'s X25519 validity check. No key here
signs or verifies anything.
traces: REQ-8jb4ny, REQ-ech45n, REQ-y7tsft

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item described one ed25519 keypair whose verifying key both signed changes to
membership as the `P2pMemberKey` and served as the `P2pDeviceKey`. Nothing
signs now, and a Member-as-a-group key is an X25519 key from its own seed.
LLR-3jjgtw and LLR-98ufry also constrain this item; they are in
`2026-10-05-unsigned-envelope.md`. *(Corrected 2026-10-05 by review round 3,
finding-9: this also named an earlier form of LLR-9qavm7, and LLR-322xfu, which constrain
`create_organisation` (SDD-89es4z) and `types.rs` (SDD-swtd3w) and sit under
those items.)* LLR-56hc77 and LLR-bwb9pu,
amended in place, are in `2026-10-04-type-safety.md`.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said the `P2pMemberKey` "signs a change to membership". An Envelope
carries no signature, so `SigningKeypair::sign` and `keys::verify`, whose only
caller was the Envelope, are removed; the item still traced an earlier form of LLR-8528ud, which
then required that no signature be checked and stated that nothing
here produces or checks one.

*Amended 2026-10-05 by the org-node type-safety change:* a keypair is built
from a seed only through the typed seeds `MemberSeed` and `DeviceSeed`, and
hands its seed back only as one of them (LLR-56hc77, in
`2026-10-04-type-safety.md`).

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
also constrained by LLR-bwb9pu, under SDD-af5vnt — `SigningKeypair`'s `Debug`
shows none of its seed, through ed25519-dalek's `SigningKey` — so the item
traces REQ-y7tsft too.)*

**LLR-e58j8m**: a keypair rebuilt from its persisted seed has the same public
key as the keypair it came from: for a `SigningKeypair`, the same verifying
key and the same DevicePublicKey; for an `X25519Keypair`, the same X25519
public key.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item also required the same signature over the same message. Nothing in
org-node signs. It is derived now, since no requirement asks for a signature;
it is assessed in `crates/org-node/docs/risk/2026-10-05-envelope-authenticity.md`.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This also required "the same signature over the same message" and satisfied
an earlier form of LLR-8528ud, the signature requirement. Nothing in org-node signs now, so the
signature clause has nothing to range over; what remains is that a Persona's
keys survive the round trip through its stored seeds, which no requirement
states, so it is derived and assessed in
`crates/org-node/docs/risk/2026-10-06-chain-authority.md`.

*(Amended 2026-10-05 by the org-node type-safety change: this read "a keypair
rebuilt from `to_seed`". That change removed `to_seed` and `from_seed`
(LLR-56hc77, in `2026-10-04-type-safety.md`), so the seed comes back typed, as
a `MemberSeed` or a `DeviceSeed`. `a_keypair_rebuilt_from_its_seed_signs_identically`
checks both roles.)*

*Amended 2026-10-08 (S4, task T3):* a device seed is held as a
`DeviceSecret` and leaves it only through `lend_to_keychain`; a `DeviceSecret`
rebuilt from those bytes (`from_keychain_bytes`) has the same Device public
key. `SigningKeypair::device_seed` is gone (LLR-37k2s3).

**LLR-ctzkv7**: a Persona's two public keys come from two seeds by two
algorithms. `X25519Keypair::member_key` is the X25519 public key of the member
seed (RFC 7748: the clamped seed times the base point) and is always a valid
`PersonPublicKey`. `SigningKeypair::device_key` wraps the ed25519 verifying
key of the device seed.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item said `member_key()` and `device_key()` wrap one verifying key. A
Member-as-a-group key is now an X25519 key from its own seed.

*Parent stale, noted 2026-10-05 by review round 4 (finding-5).* Since the
amendment of 2026-10-05, the earlier forms of REQ-uk9rw7 and REQ-eqt23j checked nothing about a
sender's keys, so neither implies this item, and `satisfies:` above names
them only until it is re-traced. By owner ruling of 2026-10-05 the re-trace is
left to chain-authority's change 1.

*Re-traced 2026-10-06 (change `worktree-org-node-chain-authority`, after the
merge of master `5f7c177`).* No requirement of this unit states how a
Persona's keys are derived: the X25519 rule is `person`'s, and the
DevicePublicKey is the iroh identity. The item is derived, and assessed in
`crates/org-node/docs/risk/2026-10-06-chain-authority.md`. *(Amended 2026-10-06,
independent review round 2, finding-10: "the DevicePublicKey" read "the
device key", the term `crates/org-node/docs/CONTEXT.md` retires for new text.)*

*Amended 2026-10-08 (S4, task T3):* a Persona's Device public key is
the ed25519 verifying key of its device seed as `DeviceSecret::device_key`
derives and parses it when the secret is built (LLR-37k2s3); `create_persona`
returns the seed as that `DeviceSecret` and keeps only the public key.
`SigningKeypair::device_key` still wraps the verifying key of a key pair.

## SDD-kk2y3e — The Envelope

`crates/org-node/src/envelope.rs`

**SDD-kk2y3e**: the wire form of a change to membership. It binds the
Organisation identifier, the Sequence number and the encoded Change set, and
carries no signature. The item also owns the decode a receiver performs on it.
traces: REQ-gju89b, REQ-9g6as6

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item described a signed transcript and the signature check a receiver ran
before anything else. (The section heading read "The signed Envelope".)

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This item was "the authenticated wire form": the transcript that is signed, and
the signature check a receiver ran before anything else. The owner ruled that
an Envelope carries no signature and that the on-chain root at a newer epoch is
the sole authority over a received Change set (an earlier form of LLR-8528ud, as amended).
`SignedDeltaEnvelope` is renamed `Envelope`; its `signature` field,
`verify_signature`, the `sig_bytes` serde helper and `build`'s key argument
are removed. The heading keeps its old title because a heading is part of the
merged item's text; the item's own sentence above is what it now is.
*(Merged 2026-10-06 with master `5f7c177`: the heading and the item text are
master's, which retitled the section "The Envelope".)*


**LLR-e7s4ye**: an Envelope's wire form is the postcard encoding of the
Organisation identifier's twenty bytes, then the Sequence number, then the
Change set bytes, in that order and nothing else; no transcript is signed.
satisfies: REQ-9g6as6, REQ-dwb9bq

*Re-pointed 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 2, finding-4).* This item satisfied only the item LLR-8528ud replaced. It now satisfies REQ-9g6as6, which bounds the decode of this
wire form, and REQ-dwb9bq, whose amendment of that day states that no
signature is checked in the decision.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item described the signed transcript, which no longer exists. The three fields
it bound are what the wire form carries.

**LLR-p8uu47**: `Envelope::build` sets the Organisation identifier and the
Sequence number it is given, and sets as the Change set bytes the postcard
encoding of the delta. That encoding is canonical: decoding the transmitted
bytes and encoding them again reproduces them exactly.
satisfies: REQ-gju89b

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item said `build` signs the transcript over the encoded bytes. Nothing is
signed.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-e7s4ye stated the signed transcript (the identifier, the Sequence number
as eight little-endian bytes, the encoded delta), and LLR-p8uu47 that `build`
signs the transcript over the bytes it transmits. With no signature there is
no transcript; what the two now state is what the Envelope binds and that
`build` transmits the Change set it was given.
*(Merged 2026-10-06 with master `5f7c177`, which amended LLR-e7s4ye and
LLR-p8uu47 in place for the same ruling: master's texts stand, and this note
describes this change's texts before that merge.)*

**LLR-v2y6sw**: `decode_delta` returns `MalformedDelta` for any byte string that
is not a valid encoded delta, and does not panic.
satisfies: REQ-9g6as6

**LLR-ybn5pr**: an Envelope built for one Organisation is refused with
`OrgIdMismatch` by a receiver expecting another, before its Change set is
decoded.
satisfies: REQ-gju89b

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item said altering the Organisation identifier breaks the signature. With no
signature, the receiver's expected Organisation is what refuses it.

**LLR-pzde8b**: an `Envelope` holds exactly the Organisation identifier, the
Sequence number and the Change set bytes; it has no signature field and no
`verify_signature`.
satisfies: REQ-dwb9bq, REQ-9g6as6

*Re-pointed 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 2, finding-4).* This item satisfied only the item LLR-8528ud replaced. It now satisfies REQ-dwb9bq, whose amendment of that day states
that no signature is checked in the decision, and REQ-9g6as6, which bounds
the decode of these three fields.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item stated that `verify_signature` refuses any key but the signer's. It is
gone with the signature.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-ybn5pr and earlier forms of LLR-76bhw9 said that altering the identifier, the
Sequence number or the Change set bytes of a built Envelope makes its
signature fail to verify, and LLR-pzde8b that `verify_signature` refuses any
other key. The signature is gone, so tampering is no longer caught at the
Envelope: it is caught by the checks that follow — the Organisation, the
Sequence number and, decisively, the root match against the chain. Each is
rewritten to the nearest true and testable statement about the unsigned
Envelope: altering a field changes the decoded value and nothing else, and
`build` is the same function whichever node calls it.
*(Merged 2026-10-06 with master `5f7c177`, which amended these four in place
for the same ruling: master's texts stand — each states the receiver's own
refusal of an altered field, and LLR-pzde8b the Envelope's three fields — and
this note describes this change's texts before that merge.)*

## SDD-d8ktxa — The replay watermark

`crates/org-node/src/sequence.rs`

**SDD-d8ktxa**: the monotonic high-water mark that refuses a sequence number
at or below the mark and never moves it; the mark is the current record's
Sequence number, set only by an adoption.
traces: REQ-6yu72z, REQ-mr5abb

*Text reconciled 2026-10-30 (change `worktree-org-io-domain-api`, merge
review round 3, finding-10):* the sentence above said the item also held "the separation
of asking from committing that lets the decisive root check run between the
two", which the note below says is gone. It now states what the note
states.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review round 2, finding-7):* the separation of asking from committing is
gone. `SeqGuard::advance` is deleted with `verify_envelope_against_chain`,
its last caller (SDD-na9nc3's note of the same day). The item now holds the
asking half alone: `SeqGuard::check` refuses a Sequence number at or below
the mark and never moves it (LLR-f5kq88), and the mark is the current
record's Sequence number, set only by an adoption (LLR-34c5y3), so no check
runs between an asking and a committing.

**LLR-wx3php**: `check` accepts a sequence number **strictly** greater than the
mark and refuses every other with `StaleSeq` carrying both the offered number
and the mark.
satisfies: REQ-6yu72z

**LLR-f5kq88**: `check` does not move the mark.
satisfies: REQ-mr5abb

**LLR-duwz79**: `from_last_seen` starts the guard at the given mark, and
`last_seen` reports the mark currently held.
satisfies: REQ-mr5abb

## SDD-na9nc3 — Verify-against-chain

`crates/org-node/src/verify.rs`

**SDD-na9nc3**: the single decisive property of this unit — a received change
is committed only if applying it to the local trie reproduces a root that
independently matches the root the chain reports at an epoch newer than the
last one committed, with the Sequence number equal to that epoch. The item
owns both the checks and **the order they run in**, which is itself
security-critical: the cheap checks on what the Envelope names and its
Sequence number come first, so stale or misaddressed bytes are never decoded;
every check that needs no chain — the Organisation, the Sequence-number mark,
the Change set decode and the base root — runs before the chain is read, so a
stranger's Envelope is refused without costing a chain read; and the watermark
moves only after the decisive check has passed. Nothing about who delivered
the Envelope is checked.
traces: REQ-dwb9bq, REQ-gju89b, REQ-6yu72z, REQ-mr5abb, REQ-rv4dwb, REQ-f2k4tr

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item put an authenticity check first, on an unauthenticated sender. Nothing
about the sender is checked (an earlier form of LLR-8528ud). An earlier form of LLR-76bhw9, in
`2026-10-05-unsigned-envelope.md`, also refined this item.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said "the cheap authenticity checks come first so that untrusted bytes
are never decoded on behalf of an unauthenticated sender". There is no
authenticity check now: an Envelope carries no signature, and bytes from any
peer reach the decoder, which REQ-9g6as6 and an earlier form of LLR-7u3dg7 already required to be
total. What the order protects instead is the chain read (RC-mj6gjq,
REQ-f2k4tr), so the item traces REQ-f2k4tr and LLR-fuq379, in
`2026-10-06-chain-authority.md`, exposes the
chain-free half as `verify::check_chain_free`.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1-N1, owner ruling 2026-10-29):* `verify_envelope_against_chain`,
`VerifiedUpdate` and `VerifyContext.last_committed_epoch` are deleted: after
the domain API no product path called them. The item now holds only the
chain-free half, `check_chain_free`, which the receive path runs before
anything is kept (LLR-fuq379, LLR-76bhw9); the decisive property — a change
becomes current only when the chain holds its root, at a greater epoch — is
`verify`'s match (LLR-rr558z, LLR-8528ud). `SeqGuard::advance` (SDD-d8ktxa) is
deleted with its last caller. The LLRs below that stated the deleted function
are superseded or retired, each with a note; LLR-4fbuy8 and LLR-xpbkp5 stay
in force over `check_chain_free`, tested on the receive path.

**LLR-4fbuy8**: an envelope naming another Organisation is refused with
`OrgIdMismatch` **before the Change set it carries is decoded**.
satisfies: REQ-gju89b

**LLR-xpbkp5**: the sequence check runs **before the delta bytes are decoded**,
so a replayed envelope is refused without its Change set bytes being parsed.
satisfies: REQ-6yu72z

## SDD-pa6p7w — The chain as a read oracle

`crates/org-node/src/chain.rs`; in `crates/org-node/src/chain_read.rs`, `org_state_from_chain`
and `OrgStateCache` *(amended 2026-10-05, see below)*

**SDD-pa6p7w**: the read-only view of what the chain says an Organisation's
membership root and epoch are, as an interface rather than a client — the root
returned here must come from a path the sender of a change does not control,
and expressing it as a trait is what makes that substitutable and testable.
traces: REQ-dwb9bq, REQ-8jb4ny

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place:
the view is now a value, what it decides unchanged. The `ChainReader` trait
and `MockChain` are deleted: `verify_envelope_against_chain` takes the
Organisation's state as `Option<OrgState>`, read by org-io from a path the
sender does not control. `OrgState::from_chain` (the parse edge,
REQ-8jb4ny) stays in `chain.rs`; `org_state_from_chain` left for org-io (task
T5); `chain_read.rs`, with `OrgStateCache` and `OnChainReader`, is deleted,
which resolves the resolved problem report on the stale cache on a fetch error. The notes below that name `chain_read.rs`,
`ChainOps::read_state` or `ChainOpsReader` describe the code before this
change.

*Amended 2026-10-05 by the org-node type-safety change.* That change split the
parse of a chain state and its cache out of `OnChainReader` into
`org_state_from_chain` and `OrgStateCache`, an implementation of
`ChainReader` that a gated test can drive without a chain, and stated what it
serves when the chain's state is refused (LLR-mmdu38, in
`2026-10-04-type-safety.md`). Those two parts of `chain_read.rs` are this
item's; the rest of the file — `OnChainReader` and its `refresh`, which talk
to a chain — stays org-io's chain-connection item's (moved 2026-10-07).

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
also constrained by LLR-mmdu38, which now sits under SDD-swtd3w in
`2026-10-04-type-safety.md`, the item that owns `OrgPublicKey` in `types.rs`.
Its last clause is this item's: `org_state_from_chain` applies the parse, and
`OrgStateCache` serves no state after a fetched state the parse refuses. This
item owns no part of `types.rs`.)*

*(Re-traced 2026-10-05 by review round 3, finding-9, change
`worktree-person-shared-types`: `OrgState::from_chain` in `chain.rs` and
`org_state_from_chain` refuse an Organisation state whose Organisation public
key is not a valid X25519 key, so the item traces REQ-8jb4ny. The parse itself
is LLR-3jjgtw's, under SDD-sxp8hb.)*

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
Unchanged in what it decides: org-node no longer writes to the chain, and keeps
reading it through on-chain-client by this item and `ChainOps::read_state`.
What changes is when the read happens. The Receive operations read the
Organisation state once, after the chain-free checks (an earlier form of LLR-76bhw9), where they
used to read it first to obtain the signing key; `ChainOpsReader` wraps that
one state. Nothing read here is used as a key: the Organisation state
contributes its root and epoch to the decision and nothing else (an earlier form of LLR-8528ud).

*Moved 2026-10-07 to org-io's architecture ledger
(`crates/org-io/docs/architecture/2026-10-08-org-io.md`, finalised
at the merge of change `worktree-org-io-create`), ruling A: the low-level
requirement that the chain read tells absence from failure went with the read
to org-io, re-parented to org-io's chain-read requirement, and its cache
exception retired with the cache (task T6 deletes it).*

## SDD-kwncn7 — The wire frame and its bound

`crates/org-node/src/wire.rs` (`WireMessage`, `MAX_MESSAGE`, `encode_message`,
`decode_message`), `crates/org-node/src/transport/mod.rs` (`TransportError`,
`MAX_FRAME`, `encode_frame`)

**SDD-kwncn7**: the framing of one Wire message on the channel — what a frame is
made of, the one-mebibyte ceiling on a body, and the refusal of anything over
it on both the sending and the receiving side.
traces: REQ-eg5j8u, REQ-9g6as6, REQ-y7tsft, REQ-8amu2a, REQ-zxgd46, REQ-3dsweu, REQ-ps2gy2, REQ-qrtsc9, REQ-b462sh

*Amended 2026-10-08 (S4, task T2):*
the item is the Wire message's encoding and bound, not the framing on the
channel. The codec left `transport/wire.rs` for `crates/org-node/src/wire.rs`, with
no feature gate: `encode_message` produces the postcard body with no length
prefix and refuses one over `MAX_MESSAGE` (1 MiB); `decode_message` refuses a
body over the bound before decoding it and anything that is not a Wire
message; both refuse with `OrgNodeError` (`MessageTooLarge { length }`,
`MalformedMessage`). The four-byte length prefix stays with the transport
(`transport::encode_frame`, the framing item) until task T10 moves it to org-io;
`MAX_FRAME` is `MAX_MESSAGE`.

*Amended 2026-10-06 (change `worktree-org-io-commit-workflow`, stage S3).* Traces REQ-ps2gy2, REQ-qrtsc9 and REQ-b462sh for the revocation notice
and the acknowledgement on the wire (LLR-dc45ur, LLR-378cj4, in
`2026-10-07-commit-workflow.md`). The revocation
kind the key-pair change defines (note below) holds a revocation notice
instead of an Envelope, and a third kind, the acknowledgement, is added.
*(Merged 2026-10-07 with the key-pair change's note below, at this branch's
merge of master `1f52c36`: the trace list is the union of both.)*

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
A frame carried one shape of Wire message, with an optional Organisation
secret and an optional invite identifier. The Wire message is now an enum of
two kinds, Organisation information (always carrying the record snapshot and
the Organisation private key) and revocation (carrying neither), and no kind
carries an invite identifier (an earlier form of LLR-xr88a8, and LLR-ecxc76, in
`2026-10-07-org-key-pair.md`; an earlier form of LLR-g3wmen amended).
A body of the first kind without its key does not decode, so the item traces
REQ-zxgd46 (then in an earlier form) and REQ-3dsweu too. The notes below that speak of the Organisation
secret or the invite identifier on the wire are history.

*Amended 2026-10-06 (owner ruling on pre-emption, change
`worktree-org-node-chain-authority`):* a Wire message carries the invite
identifier of the admission it delivers (an earlier form of LLR-g3wmen, in
`2026-10-06-chain-authority.md`), so the item
traces REQ-8amu2a too.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
also constrained by LLR-bwb9pu, under SDD-af5vnt in
`2026-10-04-type-safety.md` — `WireMessage` holds the Organisation secret as
an `OrgSecret`, so its `Debug` shows none of it — so the item traces
REQ-y7tsft too.)*

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The Wire message's Envelope carries no signature (SDD-kk2y3e), so a frame now
carries an `Envelope` of three fields, the record snapshot and the optional
Organisation secret. Its one-mebibyte bound also sizes the provisional updates
a Persona store holds for one Organisation: `MAX_PROVISIONAL_BYTES` equals
`MAX_FRAME` (an earlier form of LLR-m76r4k, under SDD-af5vnt), by the owner's ruling that the
bound is the wire frame's. The two constants are separate so a change to one
is a decision about the other.

*Moved 2026-10-13 (S4, task T10,
ruling A):* the framing item (the four-byte little-endian length before the
body) moved to org-io with the transport, to
`crates/org-io/docs/architecture/2026-10-07-transport.md`,
re-parented to org-io's stream requirement.

**LLR-er2x8n**: a Wire message encoded by `encode_frame` and decoded by
`decode_body` yields a Wire message equal to the original, so a frame within the
bound is delivered unchanged.
satisfies: REQ-eg5j8u

*Amended 2026-10-08 (S4, task T2):* the functions are `encode_message` and
`decode_message` (`crates/org-node/src/wire.rs`), and the encoding is the Wire
message's postcard body with no length prefix.

**LLR-sc6zuh**: `encode_frame` refuses a body larger than `MAX_FRAME` with
`FrameTooLarge` carrying the measured length, rather than emitting a frame.
satisfies: REQ-eg5j8u

*Amended 2026-10-08 (S4, task T2):* `encode_message` refuses an encoding over
`MAX_MESSAGE` with `OrgNodeError::MessageTooLarge { length }`.

**LLR-8kh3zf**: `decode_body` refuses a body larger than `MAX_FRAME` with
`FrameTooLarge` **before decoding any of its bytes**.
satisfies: REQ-eg5j8u

*Amended 2026-10-08 (S4, task T2):* `decode_message` refuses a body over
`MAX_MESSAGE` with `OrgNodeError::MessageTooLarge { length }`, before
decoding.

**LLR-pkruy8**: `decode_body` refuses bytes that are not a valid encoded
Wire message with `Malformed`, and does not panic.
satisfies: REQ-9g6as6

*Amended 2026-10-08 (S4, task T2):* `decode_message` refuses them with
`OrgNodeError::MalformedMessage`.

## The authenticated endpoint (moved)

*Moved 2026-10-13 to org-io's design ledger
(`crates/org-io/docs/architecture/2026-10-07-transport.md`,
"Moved from org-node with the transport"), S4 task T10, ruling A:* the design item for
the iroh endpoint whose identity is the Device key, with its five low-level
items (the endpoint id is the Device key; the sender comes from the
handshake, never the body; Loopback binds and advertises loopback only; IPv4
loopback required, IPv6 optional; the dialling address is the bound
sockets'), and the stream read cap and the unchecked length prefix. org-io
binds every endpoint from its Persona's keychain item; their `satisfies:`
lines name org-io's requirements there. org-node's own transport module is
dead code until task T14 deletes it. *(Deleted 2026-10-14, S4 task T14, with
the `transport` feature.)*

## SDD-af5vnt — The encrypted persona store

`crates/org-node/src/store.rs`

**SDD-af5vnt**: where every secret this node holds rests — the member and
device seeds and the Organisation private key — and the form the file takes,
which is ciphertext under a passphrase-derived key and nothing else.
traces: REQ-hzm4kt, REQ-kd34f5, REQ-y7tsft, REQ-ech45n, REQ-4qh2h8, REQ-dhr58q, REQ-g2537c, REQ-8amu2a, REQ-x2dnee, REQ-ju6vn2, REQ-3pj7rd, REQ-48zc3e, REQ-y9p27b

*Traces amended 2026-10-20 (change worktree-org-io-domain-api): REQ-kd34f5,
REQ-4qh2h8, REQ-dhr58q, REQ-g2537c, REQ-x2dnee, REQ-3pj7rd, REQ-48zc3e and
REQ-y9p27b replace the earlier forms the item traced.*

*Note 2026-10-20 (change `worktree-org-io-domain-api`, dispatcher's
resolution C6 and owner ruling Q4' of 2026-10-16):* the responsibility —
where the secrets rest, and that the file is ciphertext and nothing else —
is unchanged, so the item stands. What the store holds beside the current
record is now the epoch state (SDD-undd7q), and on the node's own removal
the store keeps only the signed acknowledgements, their recipients' keys and
addresses and the verification time (SDD-z4m57w). The item traces
REQ-48zc3e, REQ-y9p27b, REQ-x2dnee, REQ-g2537c, REQ-dhr58q and REQ-4qh2h8
for where the secrets of that state rest; SDD-undd7q and SDD-z4m57w also
trace REQ-48zc3e, REQ-y9p27b, REQ-x2dnee, REQ-g2537c and REQ-dhr58q for
what the store holds.

*Amended 2026-10-06 (change `worktree-org-io-commit-workflow`, stage S3).* Traces earlier forms of REQ-48zc3e, REQ-y9p27b and REQ-x2dnee for forgetting an
Organisation and for the kept Change set (earlier forms of LLR-2d5a7m and LLR-p8vpgz, in
`2026-10-07-commit-workflow.md`). Forgetting an
Organisation removes its Organisation private key and every provisional
update's key with it.
*(Merged 2026-10-07 with the key-pair change's notes below, at this branch's
merge of master `1f52c36`: the definition takes master's wording, "the
Organisation private key", and the trace list is the union of both.)*

*Amended 2026-10-06 (owner ruling on rotation, change
`worktree-org-node-org-key-pair`):* traces an earlier form of REQ-3pj7rd. Every provisional
update the store holds now holds the private key of the fresh Organisation
key pair drawn for it, not only a genesis update, and a provisional update is
identified by its public key as well as its root (earlier forms of LLR-qw8tyu, LLR-f5wrjx and
LLR-d9x6xf amended).

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This named "the Organisation secret" among the secrets the store holds. The
Organisation secret is removed; every Organisation record now holds the
Organisation private key, required, on the creating node and on every
admitted node alike (an earlier form of LLR-78jfrd in
`2026-10-07-org-key-pair.md`), so the item traces
REQ-ju6vn2 too. An expectation names the Organisation alone again
(an earlier form of LLR-f5wrjx amended).

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`):* an
expectation the store holds names an invite identifier (an earlier form of LLR-f5wrjx), so the
item traces REQ-8amu2a; and the genesis provisional update holds the
Organisation private key until it commits (an earlier form of LLR-qw8tyu; REQ-ech45n).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The store holds no pending Invites (`PendingInvite`,
`StoreData.pending_invites`), because the invitation exchange left org-node for
the app, and no administrator key (`OrgRecord.admin_member_key`). It now holds
the provisional updates a Persona has built and the Organisations the app has
declared it expects to join, with the 1 MiB bound on the first. The
requirements stating them, earlier forms of LLR-f5wrjx, LLR-m76r4k and LLR-8ru6mv, are in
`2026-10-06-chain-authority.md`, so the item
traced the items REQ-dhr58q, REQ-4qh2h8 and REQ-g2537c replaced too. The record of an Organisation keeps its
proxy account as opaque data (an earlier form of LLR-x5asbx).

*Amended 2026-10-05 by the org-node type-safety change:* the item also owns
the store encryption key's type, the records the store holds — each field in
its typed form, parsed when the store is opened — the Persona details a
Persona is created from, and what the debug rendering of a value holding a
secret may show, so it traces REQ-kd34f5 (then in an earlier form) and REQ-y7tsft too. Its low-level
requirements for them, LLR-bwb9pu, LLR-scgk5j and earlier forms of LLR-pv2rdy and LLR-rjphn4, are
in `2026-10-04-type-safety.md`.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
An earlier form of LLR-7yu2xy — a value its type's parse refuses is refused, naming the field —
now sits under this item too, moved from SDD-vee2fq. Most of what it
constrains is in `store.rs`: `PersonaStore::open`, the `Raw…` mirrors of the
four records and of `StoreData`, `parse_field`, and the member-snapshot parse
that `first_admission_base` decodes through. The Join request and Invite
decode is named by SDD-vee2fq, the import operations by SDD-rx2yvy, and
`first_admission_base` by the item SDD-busuj3 and SDD-kegag2 replaced. The earlier form of LLR-pv2rdy and LLR-bwb9pu also
constrain code outside `store.rs`: the first is named by SDD-vee2fq
(`JoinRequest`) and SDD-89es4z (`create_persona`), and LLR-bwb9pu by
SDD-kwncn7 (`WireMessage`) and SDD-sxp8hb (`SigningKeypair`).)*

*(Note 2026-10-05, at the merge of master `1feb608` into
`worktree-person-shared-types`; reworded the same day: on that branch the store also holds the
Organisation private key, in `OrgRecord.org_private_key` (an earlier form of LLR-qw8tyu, and
LLR-2dvhz8, in `2026-10-05-unsigned-envelope.md`). LLR-bwb9pu and an earlier form of LLR-7yu2xy
are amended in place, in `2026-10-04-type-safety.md`.)*

*(Re-traced 2026-10-05 by review round 3, finding-9, change
`worktree-person-shared-types`: the store keeps the Organisation private key,
so the item traces REQ-ech45n. An earlier form of LLR-qw8tyu, and LLR-2dvhz8, sit under this item
in `2026-10-05-unsigned-envelope.md`.)*

*Amended 2026-10-08 (S4, task T3):* the form the store takes is the
sealed form: one store per Persona (owner ruling 2026-10-06), `StoreData`
holding one `persona`, the record of the Organisation it is bound to, its
provisional updates and its expectations, AEAD-sealed under a key derived by
HKDF-SHA-256 from that Persona's device secret (LLR-2qqqx6), not under a
passphrase-derived key. The Persona record holds its Device public key, not
its device seed, which never enters the store (an earlier form of LLR-ay8nvu). org-node writes no
file: it gives each change out as a `StoreWrite` and takes a store back in
through `PersonaStore::unseal` (SDD-asxwp8); org-io writes and reads the
bytes. Outstanding invites join `StoreData` in task T6.

**LLR-wusj89**: the store file is exactly a twenty-four byte nonce followed by
the AEAD ciphertext of the encoded store, with no cleartext header.
satisfies: REQ-hzm4kt

*Amended 2026-10-08 (S4, task T3):* "the store file" is the sealed
store a `StoreWrite` carries (`SealedStore`): exactly a twenty-four byte nonce
followed by the AEAD ciphertext of the encoded store, with no cleartext
header; org-io writes those bytes to the Persona's file.

**LLR-8mfjey**: a fresh twenty-four byte nonce is drawn from the caller's
random source on every save.
satisfies: REQ-hzm4kt

*Amended 2026-10-08 (S4, task T3):* "on every save" becomes "on every
seal": each `StoreWrite` an operation returns is sealed with a fresh
twenty-four byte nonce drawn from the caller's random source (an earlier form of LLR-ay8nvu).

**LLR-t4u66w**: opening a store file with a passphrase other than the one it
was written under yields a typed error and no data.
satisfies: REQ-hzm4kt

*Amended 2026-10-08 (S4, task T3):* there is no passphrase. Unsealing
a store with the device secret of another Device yields `DeviceSecretMismatch`
and no data, before any decryption, and a ciphertext that does not
authenticate under the derived key yields `StoreRefused(NotAuthentic)` and no
data (LLR-bg538c).

**LLR-q5n28x**: a store file shorter than the nonce is refused as too short
rather than read as a nonce and an empty ciphertext.
satisfies: REQ-hzm4kt

*Amended 2026-10-08 (S4, task T3):* the bound is the nonce and the
sixteen-byte authentication tag: sealed bytes shorter than forty are refused
as `StoreRefused(TooShort)` rather than read as a nonce and an empty or
truncated ciphertext; forty bytes or more go to decryption (LLR-bg538c). A
narrowing: inputs refused as undecryptable before are refused as too short
now.

**LLR-s78sh7**: no member seed, device seed or Organisation private key the
store holds appears anywhere in the written file in clear.
satisfies: REQ-hzm4kt

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This named the Organisation secret, which is removed; REQ-hzm4kt, amended the
same day, names the Organisation private key in its place.

*Amended 2026-10-08 (S4, task T3):* "the written file" is the sealed
store a `StoreWrite` carries, and the device seed leaves the list: the store
holds no device seed at all, only the Device public key (an earlier form of LLR-ay8nvu).

## SDD-vee2fq — Out-of-band exchange blobs

`crates/org-node/src/blobs.rs`

**SDD-vee2fq**: the boundary at which org-node stops: it holds no
copy-pasteable blob. The Invite and the reply to it, and the armour they travel
in, are the app's; what reaches org-node from an admission's other party is a
`Joiner` of parsed values and, on the joiner's side, a declared expectation of
admission. `crates/org-node/src/blobs.rs` is removed.
traces: REQ-eqt23j, REQ-9g6as6, REQ-kd34f5

*Traces amended 2026-10-20 (change worktree-org-io-domain-api): REQ-eqt23j and
REQ-kd34f5 replace the earlier forms the item traced.*

*Noted 2026-10-21 (change `worktree-org-io-domain-api`, design gap G2 of
its plan):* the responsibility, the boundary of the out-of-band invitation
exchange in org-node, is unchanged; the parts the prose and the S4 note name
are renamed. `Joiner` and `admit_member` are gone: what reaches org-node from
an admission's other party is the admission request the receive path keeps
parsed (`KeptAdmissionRequest`, LLR-v3av9t, LLR-96z2k5), from which the
admitted Member is built when the next epoch is proposed. The Invite reply is
gone too: the joiner's answer is that admission request, a Wire message sent
over the transport, built and checked under SDD-n2x3gb and SDD-42cjqk, so the Invite is now the one thing exchanged out of band. The
joiner's declared expectation of admission is kept with its provisional
Member (`propose_member`, LLR-6acxu9).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This item was "the two copy-pasteable blobs that bootstrap a relationship
before any channel exists — the Invite an administrator hands out, and the
Join request a prospective member hands back — and the armour they travel in".
The owner ruled that the invitation exchange is outside org-node and happens
in the app (`apps/desktop/docs/requirements/2026-10-06-invitation.md`),
and that Invites, Join requests, pending Invites and the Invite's dialling
address leave org-node. The item is kept, as merged items are, and now states
that boundary; its four requirements are rewritten to it.

*Amended 2026-10-05 by the org-node type-safety change:* the blobs' fields are
held typed, and a decoded value that does not parse is refused, naming the
field for a Join request (`decode_join_request`); the requirement stating it,
an earlier form of LLR-7yu2xy, is in `2026-10-04-type-safety.md`, so the item traces REQ-kd34f5
(then in an earlier form) too.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
The earlier form of LLR-7yu2xy now sits under SDD-af5vnt, which owns most of what it constrains.
This item is also constrained by it, for `decode_join_request`,
`RawJoinRequest` and the Invite decode, and by an earlier form of LLR-pv2rdy, under SDD-af5vnt,
for `JoinRequest`'s typed fields.)*

*Amended 2026-10-13 (S4, task T6;
owner ruling 1 on the chain-authority sweep).* Rewritten: this item is now
the out-of-band Invite and Invite reply, built and serialised in org-node and
exchanged by org-io. The code is `crates/org-node/src/invitation.rs`
(`blobs.rs` stays removed); the design item SDD-n2x3gb and SDD-42cjqk replaced, in this change's sans-IO design file,
holds the build, the parse and the bookkeeping (the outstanding pairs in the
sealed store). What reaches `admit_member` is still a `Joiner` of parsed
values (an earlier form of LLR-bwtx3h), and a joining node still keeps an expectation of
admission, recorded now by the reply build.

*Retired 2026-10-13 (S4, task T6; owner ruling of that day on retired
items).* The low-level requirement that stood here, that org-node defines no
Invite, is retired: the Invite returns to org-node. Its definition and its
`assesses:` entry are deleted, and its ID is listed in the S4 verification
record; its replacement is the Blob-parse requirement under the design item SDD-n2x3gb and SDD-42cjqk replaced.

**LLR-8qxwst**: no operation of org-node takes armoured text: every value that
arrives from another device reaches it as bytes inside a Wire message frame
(`decode_body`) or as typed values from the app.
satisfies: REQ-9g6as6

*Amended 2026-10-13 (S4, task T6):* rewritten. Armoured text reaches
org-node only through `Invite::parse` and `InviteReply::parse`
(`crates/org-node/src/invitation.rs`, the Blob-parse requirement under the design item SDD-n2x3gb and SDD-42cjqk replaced);
every other value from another device arrives as a Wire message body
(`decode_message`) or as typed values. `base64` is named only in
`invitation.rs` (`absences::org_node_holds_no_invitation_exchange`).

**LLR-tcft2r**: `first_admission_base` refuses bytes that are not a valid
encoded record snapshot with a typed error rather than panicking, and extends
nothing.
satisfies: REQ-9g6as6

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The retired no-Invite item (S4 T6; see the S4 verification record) and
an earlier form of LLR-bwtx3h said an Invite and a Join request round-trip
through their encoding; LLR-8qxwst and LLR-tcft2r that bad Base64 and a bad
encoded blob are refused without panicking. All four types and their armour
leave org-node for the app, which parses the Invite and its reply at its own
edge. The retired item and LLR-8qxwst now state the absence, which the compiler
enforces as for an earlier form of LLR-96z2k5. An earlier form of LLR-bwtx3h stated what replaces the Join request
at `admit_member`. LLR-tcft2r is rewritten to the remaining place where bytes
another device composed are decoded into a record — the snapshot a first
admission extends — which `fuzz_first_admission_base` already exercises.

## SDD-msb6xh — Update calldata

`crates/org-node/src/chain_write/calldata.rs` (`build_update_calldata`,
`UPDATE_SELECTOR`, `revive_update_runtime_call`, `update_calldata` *(added
2026-10-05, see below)*)

**SDD-msb6xh**: the absence of update calldata from org-node: org-node builds
no bytes that ask the contract to move a Membership root. The calldata, its
selector and the `Revive.call` runtime call are built in on-chain-client's
chain writer
(`crates/on-chain-client/docs/architecture/2026-10-06-chain-write.md`);
what org-node hands the app for a submission is a provisional update's
resulting root, Organisation public key and the epoch its record holds.
traces: REQ-4qh2h8, REQ-dhr58q, REQ-dwb9bq

*Re-traced 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 2, finding-8):* this item traced only two items REQ-dwb9bq replaced.
It now traces the in-force items that state what
org-node does instead of writing to the chain: a founding and a provisional
change are kept writing nothing to the chain (REQ-4qh2h8, REQ-dhr58q), and
verification judges a chain state the caller read (REQ-dwb9bq).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This item was "the exact bytes that ask the contract to move an Organisation's
Membership root forward — the one part of the write path that is pure". The
owner ruled that org-node does not write to the chain: `chain_write/`, of
which `calldata.rs` is part, moves to on-chain-client, which states the
calldata's layout under its own chain-writer item. The item is kept, as merged
items are, and its four requirements now state that org-node holds none of it.
They are compile-enforced, as an earlier form of LLR-96z2k5 was. The tests that carried them —
`chain_write_pure` and `calldata_typed` — move with the code and are
re-annotated to on-chain-client's requirements.

**LLR-rv4vux**: org-node has no `build_update_calldata` and emits no contract
calldata; the hundred-byte `update` layout is on-chain-client's.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* The
third field was the Published signing key; it is the Organisation public key.

**LLR-rc74nq**: org-node has no `revive_update_runtime_call` and builds no
runtime call; the `Revive.call` it named is on-chain-client's.
satisfies: derived

*Amended 2026-10-05 by the org-node type-safety change:*
`revive_update_runtime_call` takes a `RootHash`, an `OrgPublicKey` and an
`Epoch` where it took two `[u8; 32]` and a `u128`, and "the same arguments"
means their bytes and value: `data` is `build_update_calldata(*root.as_bytes(),
*key.as_bytes(), u128::from(epoch.get()))`, which `build_update_calldata`
itself, the plain-bytes encoder at the edge, still takes.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7: the
item also owns `update_calldata(RootHash, OrgPublicKey, Epoch) -> Vec<u8>`,
new and public in `calldata.rs`, which no item named. It is the one place
both typed write paths unwrap the root, the key and the epoch into calldata:
`revive_update_runtime_call`'s `data` is `update_calldata` of its arguments,
and `submit::submit_update` (org-io's chain-connection item's (moved 2026-10-07)) calls it too. `calldata.rs` now
holds three functions, all this item's. A typed-calldata golden test in
`crates/org-node/tests/calldata_typed.rs` pinned its output (an earlier form of LLR-mkvx2k).)*

*Added 2026-10-04 by review round 6, which found this function inside
org-io's chain-connection item (moved 2026-10-07), the item with no low-level requirements, under a list headed "the
asynchronous functions only". It is neither asynchronous nor unreachable, and a
test in `crates/org-node/tests/chain_write_pure.rs` pinned it.*

*(Note 2026-10-06, change `worktree-org-node-chain-authority`, T13:
`calldata.rs`, `calldata_typed.rs` and `chain_write_pure.rs` are deleted. The
calldata and the runtime call are pinned in on-chain-client by
`write_pure::update_calldata_is_the_pinned_hundred_bytes` and
`write_pure::the_revive_call_names_every_field_and_constant`; org-node's
absence is carried by `absences::org_node_writes_nothing_to_the_chain`.)*

**LLR-txqmz4**: org-node encodes no expected epoch for the contract: the epoch
it holds reaches the app as an `Epoch` value from the record, and its
thirty-two byte calldata field is on-chain-client's.
satisfies: derived

**LLR-66h529**: org-node has no `UPDATE_SELECTOR`; the contract's function
selector is on-chain-client's constant.
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-rv4vux stated the hundred-byte calldata layout, LLR-rc74nq the
`Revive.call` runtime call, LLR-txqmz4 the epoch field and LLR-66h529 the
selector. Each moves, as behaviour, to on-chain-client with `calldata.rs`
(`crates/on-chain-client/docs/architecture/2026-10-06-chain-write.md`);
here each now states that org-node holds none of it.

## SDD-rq6nv4 — Threshold-1 multisig account derivation

`crates/org-node/src/chain_write/multisig.rs` (`multi_account_id`,
`build_dispatch_tx`)

**SDD-rq6nv4**: the absence of the multisig from org-node: org-node derives no
pseudo-account, holds no multisig signatory key and dispatches nothing. The
derivation of the account that controls an Organisation's slot, and the choice
between dispatching directly and through it, are on-chain-client's chain writer
(`crates/on-chain-client/docs/architecture/2026-10-06-chain-write.md`).
traces: REQ-4qh2h8, REQ-dhr58q, REQ-dwb9bq

*Re-traced 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 2, finding-8):* this item traced only two items REQ-dwb9bq replaced.
It now traces the in-force items that state what
org-node does instead of dispatching to the chain: a founding and a
provisional change are kept writing nothing to the chain (REQ-4qh2h8,
REQ-dhr58q), and verification judges a chain state the caller read
(REQ-dwb9bq).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This item was "the derivation of the pseudo-account that controls an
Organisation's on-chain slot, and the choice between dispatching directly and
dispatching through it". The owner ruled that org-node does not write to the
chain and drops its sr25519 multisig key; `multisig.rs` moves to on-chain-client
with the rest of `chain_write/`, and with it the `blake2` and
`parity-scale-codec` dependencies and `test_support::build_dispatch_tx`. The
item is kept and its three requirements state the absence, compile-enforced.

**LLR-463d89**: org-node has no `multi_account_id` and derives no multisig
account.
satisfies: derived

**LLR-8m3bwj**: org-node holds no multisig threshold or signatory key: no
record, store field or `OrgService` argument carries an sr25519 key.
satisfies: derived

**LLR-f74xwb**: org-node has no `build_dispatch_tx` and dispatches no call,
directly or through a multisig.
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-463d89 said `multi_account_id` sorts the signers, LLR-8m3bwj that the
derived account depends on the threshold, and LLR-f74xwb that
`build_dispatch_tx` dispatches directly or through a threshold-1 multisig. All
three behaviours move to on-chain-client; here each states that org-node holds
none of it.


## SDD-ueh4tm — The chain-operations seam

`crates/org-node/src/service.rs` (`ChainOps`, `MockChainOps`, `MockChainInner`,
`ChainOpsReader`)

**SDD-ueh4tm**: the boundary between the five user stories and the chain,
expressed as a value rather than a trait or a client: the `Option<OrgState>`
that `verify` takes, read by its caller. It exists so the stories can be
exercised in full without a chain, and it is the mechanism that makes
SDD-rx2yvy, SDD-busuj3, SDD-kegag2 and SDD-z4m57w verifiable items instead of parts of the
I/O shell.
traces: REQ-dwb9bq, REQ-ncv4sc

*Text reconciled 2026-10-30 (change `worktree-org-io-domain-api`, merge
review round 3, finding-10):* the sentence above said the boundary was "expressed as a
trait rather than a client", which the 2026-10-08 note below says no longer
holds. It now names the value the 2026-10-30 note below names.

*Re-traced 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 2, finding-8):* this item traced only earlier forms of REQ-dwb9bq and
LLR-rr558z, all superseded. It now traces the in-force items that state
the boundary as values: verification takes the Organisation state the
caller read, and refuses with none (REQ-dwb9bq, as amended that day), and
the one receive path takes no chain state (REQ-ncv4sc). The operations the
2026-10-08 note below lists are removed; `verify` is the one that takes an
`Option<OrgState>` (LLR-rr558z, LLR-7en38k).

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place: a
clarification of how the boundary is expressed, not of what it is for. It
said the boundary is "expressed as a trait rather than a client"
(`ChainOps`, `MockChainOps`, `MockChainInner`, `ChainOpsReader` in
`service.rs`). org-node now reads no chain: the boundary is the
`Option<OrgState>` value each chain-judging operation takes
(`commit_genesis`, `commit_update`, `reconcile`, `apply_receive`,
`apply_self_delete`, `verify_envelope_against_chain`), and the receive paths
are split so that the chain-free phase (`prepare_receive`,
`prepare_self_delete`) names the Organisation whose state the apply phase
needs. The stories are still exercised in full without a chain, by passing
values from the test-support store `org_node::test_fixtures::ChainSlots`,
which replaces `MockChainOps` and keeps its `apply_genesis`/`apply_update`
stand-ins. Its code is now `service.rs` (`Prepared`, `PendingReceive`, the
`prepare_*`/`apply_*` split) and `test_fixtures.rs` (`ChainSlots`). The new
low-level requirement, an earlier form of LLR-7en38k (in
`2026-10-08-values.md`), states the absence of any chain
read and supersedes the one before it.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The seam is now read-only: `ChainOps` keeps `read_state` and loses
`submit_genesis` and `submit_update`, because org-node does not write to the
chain — the app submits through on-chain-client and then asks org-node to
commit. `MockChainOps` keeps a stand-in for that write, `apply_genesis` and
`apply_update`, inherent methods a test calls in the app's place, so a story
can still run from provisional update to commit without a chain.

**LLR-hg3xzf**: clones of `MockChainOps` share one chain state, so two services
under test observe the same chain as each other.
satisfies: derived

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place: a
rename, the meaning unchanged. `MockChainOps` is replaced by the test-support
value store `org_node::test_fixtures::ChainSlots`: clones of `ChainSlots`
share one set of slots, so two services under test are given states from
the same chain. The tests are re-pointed to it
(`service_lifecycle.rs`: `clones_of_the_mock_chain_share_one_state` and
`two_services_over_one_set_of_chain_slots_see_the_same_state`).

*Note 2026-10-08 (robustness, N/A):* two services over one set of chain slots see the same state: a sharing property of the values the seam hands out, with no input that can be malformed, so it has no abnormal-input case. It was watched red with clones that did not share their slots (S2 T6).

**LLR-ryzr8m**: `MockChainOps::apply_update(org_id, root, key, expected_epoch)`
refuses an update whose expected epoch is not its slot's current epoch, with a
chain error and the slot unchanged, and an update at the current epoch
advances the slot by one; `apply_genesis(root, key)` creates a slot at epoch
one and returns its org id. This is the contract's compare-and-swap as the
mock imitates it, so a story exercised against the mock cannot commit an
update the chain would refuse.
satisfies: derived

*Amended 2026-10-08 (ruling B, change worktree-org-io-create).* In place: a
rename, the meaning unchanged. `MockChainOps::apply_update` and
`apply_genesis` are now `org_node::test_fixtures::ChainSlots::apply_update`
and `apply_genesis`, unchanged in body: they stand in for the chain write
org-io makes through on-chain-client, and the compare-and-swap is kept. The
tests (`service_lifecycle.rs`) are re-pointed to `ChainSlots`.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
An earlier form of LLR-7en38k said `ChainOps` presents "submitting an update" too, and LLR-ryzr8m
stated the compare-and-swap on `MockChainOps`'s `submit_update`. The write
leaves the trait; the mock's imitation of it stays, as inherent methods a test
calls where the app would submit through on-chain-client.

*Added 2026-10-05 by review round 8, which measured that disabling the check left the gate green: every story submits at
the right epoch. `the_mock_chain_refuses_an_update_at_the_wrong_epoch` carries
it.*

## SDD-89es4z — Persona and Organisation genesis

`crates/org-node/src/service.rs` (`create_persona`, `create_organisation`,
`commit_genesis` *(added 2026-10-05, see below)*)

**SDD-89es4z**: how a node acquires an identity and how an Organisation comes
into existence — two keypairs drawn once, a genesis provisional update, and
the Organisation record created only once the chain carries its root.
traces: REQ-hzm4kt, REQ-dwb9bq, REQ-d9g6nt, REQ-kd34f5, REQ-ech45n, REQ-4qh2h8, REQ-tqap3r, REQ-yp75u9, REQ-3pj7rd

*Traces amended 2026-10-20 (change worktree-org-io-domain-api): REQ-kd34f5,
REQ-4qh2h8 (the founding; REQ-dhr58q, which replaced the same item, records
admissions and revocations) and REQ-3pj7rd replace the earlier forms the item traced.*

*Amended 2026-10-06 (owner ruling on rotation, change
`worktree-org-node-org-key-pair`):* traces an earlier form of REQ-3pj7rd. Genesis draws the
first Organisation key pair after its record's root is calculated, as every
provisional update now does (earlier forms of LLR-d4gpxm and LLR-9qavm7 amended).

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`, independent
review round 1, finding-1):* traces REQ-yp75u9, for `create_organisation`'s
refusal of a Persona already bound (an earlier form of LLR-qqqg4b, in
`2026-10-06-chain-authority.md`).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said "a genesis trie, and the first on-chain slot": `create_organisation`
submitted the genesis to the chain and created the record. The owner ruled
that org-node does not write to the chain and that what it builds is
provisional until the chain agrees. `create_organisation` now stores a genesis
provisional update and nothing else (an earlier form of LLR-d4gpxm); the app runs the genesis
ceremony through on-chain-client and calls `commit_genesis` with the org id
and proxy account it returned, which verifies the update against the chain and
creates the record (an earlier form of LLR-gcm7bq). Both requirements are in
`2026-10-06-chain-authority.md`, so the item
traced the item REQ-dhr58q and REQ-4qh2h8 replaced, and REQ-tqap3r, too.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
also constrained by an earlier form of LLR-pv2rdy, under SDD-af5vnt in
`2026-10-04-type-safety.md`. `create_persona` takes the handle, name and
surname as a parsed `Handle`, `Name` and `Surname`, so a Persona record is
built only from parsed values. So the item traces REQ-kd34f5 (then in an earlier form) too.)*

*(Re-traced 2026-10-05 by review round 3, finding-8 and finding-9, change
`worktree-person-shared-types`: `create_organisation` draws the Organisation
private key, publishes its X25519 public key with the genesis root and
refuses one equal to a genesis key, so the item traces REQ-ech45n.
An earlier form of LLR-9qavm7, in `2026-10-05-unsigned-envelope.md`, refines it.)*

**LLR-s7yu4k**: a Persona's identifier is the first sixteen bytes of its
Member-as-a-group key, the X25519 public key of its member seed, written as
thirty-two lowercase hexadecimal digits. Two Personas share an identifier only
if their member keys share those bytes.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item derived the identifier from the member verifying key.

**LLR-rjg3m2**: the founding Member's `MemberId` is drawn from the caller's
random source before the genesis provisional update's Membership record is
built and is never computed from a key, so the same persona building two
genesis provisional updates receives two different identifiers.
satisfies: REQ-d9g6nt

*Amended 2026-10-06 (REQ-yp75u9, independent review round 1 of change
`worktree-org-node-chain-authority`).* This read "the same persona founding two
Organisations". A Persona is now bound to at most one Organisation, so it can
found only one (earlier forms of LLR-qqqg4b); while unbound it can still build
several genesis provisional updates, and those are where the same keys meet
two drawn identifiers.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
"The founding administrator" reads "the founding Member", as REQ-d9g6nt does
after the same ruling: org-node has no concept of an administrator. The
identifier is drawn when the provisional update is built.

## SDD-rx2yvy — Admission

`crates/org-node/src/service.rs` (`admit_member`, `export_invite`, `import_invite`,
`export_join_request`, `import_join_request`; since 2026-10-05 `admit_member`,
`Joiner`, `expect_admission`, `persona_public_keys`, `proxy_account`,
`send_update` and `OutgoingUpdate`, see below)

**SDD-rx2yvy**: the two sides of letting someone in, as org-node sees them —
on the inviting node, minting the new Member's leaf into an admission
provisional update and, once that update has committed, sending the joiner the
committed Envelope and everything they need to check it; on the joining node,
recording that the app expects that admission.
traces: REQ-eqt23j, REQ-uk9rw7, REQ-dwb9bq, REQ-d9g6nt, REQ-kd34f5, REQ-dhr58q, REQ-tqap3r, REQ-8amu2a, REQ-szq3ud, REQ-3dsweu, REQ-3pj7rd

*Traces amended 2026-10-20 (change worktree-org-io-domain-api): REQ-eqt23j,
REQ-kd34f5, REQ-dhr58q (an admission is a provisional change; REQ-4qh2h8,
which replaced the same item, is the founding) and REQ-3pj7rd replace the
earlier forms the item traced.*

*Amended 2026-10-13 (S4, task T5):*
the send goes. `send_update` and `OutgoingUpdate` are removed; the commit
builds every recipient's body and returns it in `CommitOutcome.outgoing`, an
item of the design the item SDD-qzv5mn replaced (an earlier form of LLR-degg8t), and org-io sends it. Until T14 a
transitional `send_body` sends one body from the bound endpoint. *(Deleted
2026-10-14, S4 task T14.)*

*Amended 2026-10-06 (owner ruling on rotation, change
`worktree-org-node-org-key-pair`):* traces an earlier form of REQ-3pj7rd. An admission
provisional update, built through `keep_change_set`, draws a fresh
Organisation key pair once its root is calculated, keeps the private key and
publishes the public key (an earlier form of LLR-qghs95; an earlier form of LLR-va2yqz amended).

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
"Everything they need to check it" now includes the Organisation private key
of the epoch the update reaches, as the node's record holds it, which
`send_update` puts in every Organisation-information message without taking
it from its caller; and `send_update` chooses the kind of Wire message by
whether the node's committed record lists the recipient Device (an earlier form of LLR-degg8t,
in `2026-10-07-org-key-pair.md`; earlier forms of LLR-degg8t and LLR-6acxu9 and the Loopback dial item —
org-io's since 2026-10-13 — amended). On the joining
node the expectation names the Organisation alone. The item traces
REQ-szq3ud and REQ-3dsweu too.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item handed the new member the signed change. An earlier form of LLR-7yu2xy, amended in place,
still constrained it.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This was "the administrator's side of letting someone in — minting the new
member's leaf, moving the on-chain root forward, and handing the new member the
signed change". The owner ruled that org-node has no administrator, does not
write to the chain, builds provisional updates, and that the invitation
exchange belongs to the app. `admit_member` now builds and stores a provisional
update and nothing more; the app submits it through on-chain-client and calls
`commit_update` (the item SDD-busuj3 and SDD-kegag2 replaced); sending is `send_update`, a separate operation
that a commit never waits on. `export_invite`, `import_invite`,
`export_join_request` and `import_join_request` are removed; the app reads a
Persona's public keys through `persona_public_keys`, and declares an expected
admission through `expect_admission`. The item traced the item REQ-dhr58q and REQ-4qh2h8 replaced, REQ-tqap3r
and REQ-8amu2a for these, and an earlier form of LLR-degg8t in
`2026-10-06-chain-authority.md` sits under it.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
also constrained by an earlier form of LLR-7yu2xy, under SDD-af5vnt in
`2026-10-04-type-safety.md`. `import_join_request` refuses a Join request
holding a value its type's parse refuses, with `InvalidField` naming the
field, and `import_invite` stores no pending Invite whose keys are not curve
points. So the item traces REQ-kd34f5 (then in an earlier form) too.)*

### The four out-of-band functions

*Added 2026-10-04 by review round 2, which found that four of the five
functions this item's interface names — `export_invite`, `import_invite`,
`export_join_request` and `import_join_request` — were refined by no low-level
requirement at all. The encoding and decoding of the blob **types** is
SDD-vee2fq's and is refined there; what was missing is what these four
functions do with them, and one of those is a trust root.*

**LLR-zj88e6**: org-node has no `export_invite` and pins no first admission's
sender to a key: the inviter's DevicePublicKey an Invite carries are read by the
app from `persona_public_keys` (LLR-437fvx).
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* This
item ended ", so the joiner has a key to pin the first admission's sender
against". Nothing on the joiner's side compares that key with a sender since
the owner's ruling.

*Amended 2026-10-13 (S4, task T6):* rewritten. `OrgService::build_invite`
carries the Device public key of the Persona bound to the Organisation
(an earlier form of LLR-h9f5c9), read inside org-node; there is still no `export_invite`, and
still no first admission's sender is pinned to that key. The evidence is
`org_node_holds_no_invitation_exchange`'s presence check of `fn build_invite`
and `fn build_reply`, and the tests of LLR-h9f5c9's earlier form.

**LLR-437fvx**: `persona_public_keys(persona_id)` returns the Persona's
Member-as-a-group key and its DevicePublicKey as two distinct keys, each derived
from that Persona's own seed, and no dialling address.
satisfies: derived

## The receive-and-commit path

`crates/org-node/src/service.rs` (`receive_and_verify`, `ReceiveOutcome`,
`first_admission_base` *(added 2026-10-05, see below)*)

**LLR-j6j95z**: a first admission to an Organisation the node holds no record
of is attempted only from an Organisation-information Wire message, whose
record snapshot is required, so it always rebuilds from the snapshot of the
Membership record the change extends (`first_admission_base(&[u8])`) and never
from a reconstructed one; a body of that kind without a snapshot does not
decode (LLR-xr88a8), and a revocation, which carries none, is refused
(LLR-95av3e).
satisfies: REQ-d9g6nt

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This said a first admission is refused "when the Wire message carries no
snapshot", because the snapshot was optional. The Organisation-information
kind requires it, so the absence is a decode refusal or a revocation's
refusal, and `first_admission_base` takes the snapshot bytes, not an
`Option`.

**LLR-cja9zv**: the record, the epoch, the sequence mark and the member
snapshots are written together on an adoption by `verify`, in the one sealed
store change returned with the outcome.
satisfies: REQ-dwb9bq, REQ-x2dnee

*Text reconciled 2026-10-30 (change `worktree-org-io-domain-api`, merge
review round 3, finding-10):* the sentence above said they were written "after
verification succeeds, and the store is saved before the outcome is
returned". org-node saves no store: `verify` returns the sealed change with
the outcome and org-io writes it. It now reads as the note below says.

*Re-pointed 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 2, finding-4).* This item satisfied only two items REQ-dwb9bq replaced.
It now satisfies REQ-dwb9bq (an adoption
after verification against the chain) and REQ-x2dnee (the adopted record
replaces the previous one); "after verification succeeds" reads "on an
adoption by `verify`", whose one sealed change carries the record, its
epoch and its Sequence number together (LLR-34c5y3).

*Annotated 2026-10-05 by review round 8.* This holds for a change that
removes this node's own device key too: on this path that change is committed
as an update, with nothing deleted. That is the resolved problem report on an own revocation on the ordinary receive path's defect, stated here
and not endorsed, and the resolved problem report on an own revocation on the ordinary receive path pin carried this requirement.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The annotation above no longer holds: a change that removes this node's own
DevicePublicKey is not committed as an update on this path. Owner ruling 7 (an
empty record on removal, an earlier form of REQ-48zc3e) applies it on every commit path, so the
record is deleted and the Personas revoked (an earlier form of LLR-jhy6t6). The pin for the resolved problem report on an own revocation on the ordinary receive path
asserted the opposite; it reddened when this was implemented and was rewritten
2026-10-06 as `an_own_revocation_on_the_ordinary_path_deletes_the_record`,
which carries this requirement now.

**not** evidenced: removing it leaves the gate green. It is stated so that the
fix has something to keep.*

*(Note 2026-10-05, review round 3, finding-15, change
`worktree-person-shared-types`: the test named above is now
`a_first_admission_records_the_invites_administrator_the_chains_key_the_secret_and_the_member`,
renamed by review round 2. The administrator-key exclusion is evidenced since
review round 1 of that change, by
`the_administrators_own_persona_is_never_the_one_a_receive_marks`
(`crates/org-node/tests/admission_sender.rs`), which review round 3's mutation M14,
removing the exclusion, turned red. The resolved problem report on the receive path rebinding Personas, the unbound Persona selection, is
unchanged.)*

*(Note 2026-10-06, change `worktree-org-node-chain-authority`: with the
administrator gone the first test is now
`a_first_admission_records_the_chains_key_the_secret_and_the_member` and
`a_first_admission_records_the_chains_organisation_public_key` carries
an earlier form of LLR-q4puca; the exclusion is removed, and its pin is rewritten to the amended
earlier form of LLR-exaa9u as
`the_persona_marked_active_is_the_first_whose_device_is_in_the_record`.)*

*(Note 2026-10-06, change `worktree-org-node-org-key-pair`: the first test
named in the note above is now
`a_first_admission_records_the_chains_key_the_private_key_and_the_member`,
renamed with the Organisation secret's removal.)*

*Re-traced 2026-10-04 by review round 6. All three said they
satisfied an earlier form of REQ-eqt23j, which stated only the first-admission sender check against the
invite. None of the three refines that check: the secret overwrite on later
updates, the Persona selection on every receive, and the administrator's Member-as-a-group key on the member's record. Tracing them there took them out of the derived-behaviour
risk assessment. They are `derived` now, and assessed in
`crates/org-node/docs/risk/2026-10-03-architecture-derived.md`.*

*Also noted by review round 6: an earlier form of REQ-48zc3e, what a node does on committing a
Change set that removes its own Device key, is not scoped to a receive
operation, and this item does not apply it. On this path the node's own
removal is committed as an ordinary update. That is the resolved problem report on an own revocation on the ordinary receive path.* *(Resolved
2026-10-06 by change `worktree-org-node-chain-authority`: that earlier form applies on
every commit path (an earlier form of LLR-jhy6t6), and the pin is rewritten as
`an_own_revocation_on_the_ordinary_path_deletes_the_record`.)*

*Added 2026-10-04 by review round 4. Round 3 wrote an earlier form of LLR-f9x3vf for this clause
on the administrator's side; round 4 found **thirteen** lookups on the two
receive paths and in `import_invite` able to ignore their identifier with the
whole gate green, because the test round 3 wrote gives the **receiving**
service one Organisation and one Persona. It was a two-Organisation test of
the sender and a one-Organisation test of everyone else. What the gate reported
green included an update for one Organisation overwriting another's root,
epoch, sequence mark, member snapshots and **Organisation secret**, and the
first-admission sender cross-check — RC-b6mydy's explicit trust root —
comparing the authenticated sender against whichever pending invite came
first.*

**LLR-q8emds**: the expectation a first admission matched — the one naming its
Organisation — is cleared, in the same store save, once that admission has
committed, and no expectation for another Organisation is cleared; a refused
first admission leaves every expectation in `expected_admissions`.
satisfies: REQ-8amu2a

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This said the expectation matched was "its Organisation and its invite
identifier", so a second Invite's expectation for the same Organisation
stayed. An expectation names the Organisation alone now (an earlier form of LLR-6acxu9), and at
most one is held per Organisation.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said a pending Invite is discarded only once the first admission has
committed, and satisfied an earlier form of REQ-eqt23j. The Invite leaves org-node; the
expectation the app declares takes its place (owner ruling 5), cleared at the
same point.

*Amended 2026-10-06 (owner ruling on pre-emption, REQ-8amu2a as amended).*
This said "an expected admission is cleared … once a first admission to that
Organisation has committed". An expectation names an invite identifier too,
so only the matching one is cleared; another expectation for the same
Organisation (a second Invite) stays.

## Revocation and self-delete

`crates/org-node/src/service.rs` (`revoke_member`,
`receive_and_self_delete_if_revoked`, `SelfDeleteOutcome`)

### The administrator's half of the same item

*Added 2026-10-04 by review round 2. All five low-level requirements above
refine `receive_and_self_delete_if_revoked`. `revoke_member` — the other
function this item's interface names, and roughly a hundred and twenty lines
that remove a member from the trie, submit the new root on chain, build and
sign the revocation, push it, and then update and persist the administrator's
record — was refined by none of them. Its twin `admit_member` carries the
ordering, sequencing and persistence clauses under SDD-rx2yvy; the revocation
half stated not one of them, and the gate could not see the absence, because
there is no rule that convicts an interface for being partly refined.*

## Endpoint lifecycle (moved)

*Moved 2026-10-13 to org-io's design ledger
(`crates/org-io/docs/architecture/2026-10-07-transport.md`,
"Moved from org-node with the transport"), S4 task T10, ruling A:* the design item for
when the transport comes into existence and whose key it carries, with its
three low-level items (bound once, rewritten there to once per Persona;
bound from the named Persona's Device, rewritten there to its keychain item;
Loopback binds loopback only). The transitional `bind_endpoint`,
`set_transport_mode`, `with_endpoint` and `endpoint` in `service.rs` are
dead code until task T14 deletes them: org-io calls none of them. *(Deleted
2026-10-14, S4 task T14: `OrgService` holds no endpoint and no transport
mode.)*

## The chain-facing I/O shell (moved)

*Moved 2026-10-07 to org-io's architecture ledger
(`crates/org-io/docs/architecture/2026-10-08-org-io.md`, finalised
at the merge of change `worktree-org-io-create`), ruling A: the design item
for the chain connection, the production read and the preflight, with its
recorded deviation (no low-level requirements) and its size measurement, now
re-measured over org-io's files. org-node has no chain-facing shell after
ruling B: it takes the Organisation state as a value and reads no chain.
Until task T6 removes them,
org-node's `chain_read.rs`, `preflight.rs`, `bin/preflight.rs`,
`connect_chain_client` and `SubxtChainOps` are copies of what org-io now
owns, described by no item here.*

## What is not an item

`crates/org-node/src/lib.rs` is seventy-eight lines *(eighty-five since the org-node
type-safety change, 2026-10-05, which re-exports its value types there)* and
is named by no software item: the
crate doc comment, the module declarations, the feature gates and the public
re-export surface. Every behaviour it exposes belongs to the item that defines
it. Naming no item for it is not a claim that it decides nothing — the feature
gates decide whether `service.rs`, the chain modules and the transport compile
at all, which is a segregation decision and is stated in the README's Overview.

`crates/org-node/src/test_fixtures.rs` is seventy-five lines *(145 since the org-node
type-safety change, 2026-10-05, which added typed key fixtures and the
`Display`/`Copy` probes)* behind the `test-support`
feature, which `crates/org-node/Cargo.toml` states is never enabled in a production
build. It is the route by which four of this unit's gated targets reach the
constructors they need. It is a **test seam, not a software item**: it adds no
behaviour of its own. But as with on-chain-client's `test_support`, if it were
deleted several items would lose their evidence at once, and that is worth a
reader knowing rather than discovering.

`crates/org-node/src/chain.rs`'s `MockChain` and `service.rs`'s `MockChainOps` are
**not** test seams in that sense — they are compiled unconditionally and named
by SDD-pa6p7w and SDD-ueh4tm respectively, because substitutability at those
two boundaries is a design decision this unit depends on, not a convenience for
tests.

## Deliberately not refined: which Persona a receive binds its endpoint from

*Headings in this part retitled 2026-10-04 by review round 5. They read "Four
behaviours", "Three further behaviours" and "One behaviour", which adds up to
eight, over sections holding four. Each heading now names its subject, so no
count can drift from the content under it. The one-endpoint-per-service
assumption on the **sending** side is not a deliberate absence. It is
the endpoint-bound-once report (org-io's since S4 task T10, resolved there).*

**Which Persona a receive operation binds its endpoint from.** Both
`receive_and_verify` (`service.rs:879`) and
`receive_and_self_delete_if_revoked` (`:1216`) take `personas.first()` when no
endpoint is already bound. *(Amended 2026-10-05 by the org-node type-safety
change, review round 7: the citations read `:906` and `:1268`, the lines
before that change's edits to `service.rs`.)* Review round 4 found this refined by nothing —
behaviour with no requirement, and the same "one Persona per store" assumption
round 2 convicted `ensure_endpoint` for.

**No low-level requirement is written for it, deliberately, and the reason is
worth stating rather than hiding behind one.** A requirement here would have to
be either a correctness clause, which would be false — a device holding two
Personas receives on whichever it created first, whatever Organisation the
sender is addressing — or a statement of the limitation, which **no gated test
could carry**: every receive test injects its endpoint with `with_endpoint`, so
`ensure_endpoint` returns the injected one and this lookup is never reached.
Writing the requirement anyway would satisfy `check-trace.sh`'s MISSING-TEST
rule with an annotation alone, which is precisely the move this ledger has now
spent four review rounds convicting. It is recorded here instead.

The receive path has no Organisation identifier to work from until it has read the Wire message, so choosing the right Persona would mean the caller naming one —
an API change, booked rather than made inside an architecture tooth.

## Deliberately not refined: the Networked arm, and the transport mode `ensure_endpoint` binds in

**The Networked arm of `admit_member` and `revoke_member`.** The Loopback dial
item (org-io's since 2026-10-13, S4 task T12, where the Networked arm is
gated over an in-process relay) used to
claim both dialling modes — "in Loopback mode … and in Networked mode by the
endpoint identity derived from the joiner's device key". Review round 1 found
the Networked half credited to tests that cannot observe it: no gated test ever
puts an `OrgService` into `TransportMode::Networked` (only `preflight.rs` does,
and that target is chopsticks-excluded), and replacing the whole Networked arm
with the Loopback body leaves the entire gate green. The requirement now states
only the Loopback clause.

The Networked arm is therefore **org-io's chain-connection item's (moved 2026-10-07) problem in miniature, inside an
item that is otherwise well evidenced**: it is real code on a real path, and
this gate is blind to it. It cannot be brought into the gate the way the
Loopback path was, because `bind_with_mode(Networked)` uses `presets::N0` —
n0's relay servers and DNS discovery — so a gated test of it would reach the
public internet. What would close it is a `test-support` constructor taking an
injected relay, the same seam `transport_networked` already uses at the
endpoint layer but which `OrgService` does not expose. It is not yet
written.

**That `ensure_endpoint` binds in the mode the service holds.** The third
behaviour, found by review round 2 and the same gap one layer down: replacing
`self.transport_mode` with a hard-coded `TransportMode::Loopback` leaves the
entire gate green, because `OrgService::new` already sets Loopback and no gated
test may set the only other value. The Loopback-bind item (org-io's since
S4 task T10) has been narrowed to the clause
the gate can see — in Loopback mode the endpoint binds loopback addresses and
nothing else — and that the function consults the field at all is unevidenced
here. The same injected-relay constructor closes it, which is why it is booked
with the arm above rather than separately.

*Round 2's closing observation about this unit is that the three behaviours in
this section were each found the same way — by someone who was not the author
mutating a line the author had not thought to mutate — and that all three are
the same shape: a branch whose other side this gate cannot reach, stated as
though both sides were evidenced.*

## Deliberately not refined: the key-conversion arm in `recv_one`

`recv_one` rebuilds an `ed25519_dalek::VerifyingKey` from the identity the QUIC
handshake authenticated, and maps a failure to `Malformed`
(`crates/org-node/src/transport/endpoint.rs`). A low-level requirement for that arm was
written during this change and then **cut**, because an iroh `EndpointId` is a
valid ed25519 public key by construction: nothing a peer can do makes that
conversion fail, so no mutation to the arm would redden a test and the
requirement would have asserted a property no gate can observe. The arm stays
in the code, where it costs nothing and is correct; it is recorded here so the
next reader knows its absence from the ledger was decided rather than missed.

## Robustness, and the one-sided low-level requirements

Class C asks for a normal-case and an abnormal-input case per item. No script
in `.guardrails/scripts` measures the split — nothing there reads robustness
sidedness — so the count below is a hand count, made while running the gate,
and it is reported here rather than left to be rediscovered.

Where a behaviour has an accept path and a refuse path, this ledger states them
as **two** low-level requirements rather than one, because each is separately
codeable and separately mutable. That makes each of the pair one-sided by
construction while leaving the item's coverage complete. The pairs are:

| One-sided LLR | Its other side |
|---|---|
| LLR-e58j8m (a keypair rebuilt from its persisted seed has the same public key) | — none in this file; SDD-sxp8hb's abnormal side is LLR-3jjgtw's refusals, in `2026-10-05-unsigned-envelope.md` |
| LLR-p8uu47 (build sets the fields it is given, in a canonical encoding) | LLR-ybn5pr and earlier forms of LLR-76bhw9 (the receiver's own refusals of an altered field) |
| LLR-er2x8n (a frame within the bound round-trips) | LLR-sc6zuh, LLR-8kh3zf (over the bound, both directions) |
| the frame layout (org-io's since S4 task T10) | LLR-pkruy8 (bytes that are not a frame) |
| an earlier form of LLR-bwtx3h (the parsed `Joiner` is what is admitted) | LLR-tcft2r (a record snapshot that does not decode) |
| LLR-wusj89 (the file's layout) | LLR-q5n28x (a file too short to have one) |
| LLR-t4u66w (the wrong passphrase yields an error) | LLR-wusj89 (the right one yields the data) |
| LLR-wx3php (strictly greater is accepted) | — the same LLR states both sides |
| an earlier form of LLR-f9mcsq (still a member: update) | an earlier form of LLR-2d5a7m (no longer a member: delete) |
| an earlier form of LLR-f9mcsq (verification fails: record stands) | an earlier form of LLR-9jn3z8 (verification runs first) |
| org-io's absence-from-failure LLR (moved 2026-10-07) (absence from failure) | — the same LLR states both sides |

*Amended 2026-10-05; reworded the same day. The Envelope carries no signature
(an earlier form of LLR-8528ud), so the first two rows above, and "SDD-sxp8hb's abnormal side is
an earlier form of LLR-283x6f" below, describe texts since amended in place. That earlier form of LLR-283x6f, amended
in place, stated that a relay by another device is not refused for that;
LLR-ybn5pr and earlier forms of LLR-76bhw9 state the receiver's own refusals of an
altered field. SDD-sxp8hb's abnormal side is LLR-3jjgtw's refusals, in
`crates/org-node/docs/architecture/2026-10-05-unsigned-envelope.md`.*

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
Three rows changed with the requirements they pair. The signature pair
(earlier forms of LLR-8528ud and LLR-283x6f) and the signed-transcript pair (LLR-p8uu47 against
the three tampering requirements) are now the seed round trip against another
seed's keys, and the Change set `build` transmits against each field's
alteration reaching the decoded value. The blob pair (the no-Invite item,
retired in S4 T6, and an earlier form of LLR-bwtx3h against LLR-8qxwst and LLR-tcft2r) is now
the parsed `Joiner` against a record snapshot that does not decode; the
no-Invite item and LLR-8qxwst stated absences and had no other side.
*(2026-10-13, S4 T6: the Blob-parse requirement under the design item SDD-n2x3gb and SDD-42cjqk replaced pairs its
round trip with its field-named refusals.)* The new requirements of
`2026-10-06-chain-authority.md` pair as follows:
LLR-34c5y3 (a commit that verifies) against LLR-qvp7mq (one refused at any
step), and LLR-f5wrjx (a provisional update kept) against LLR-m76r4k (one over the
bound refused), each then in an earlier form; an earlier form of LLR-e6w3nr stated both sides of the expectation.

*Recomputed 2026-10-06 at the merge of master `5f7c177`.* The first two rows
are re-derived from the merged texts, master's for both pairs: LLR-e58j8m has
no other side in this file (SDD-sxp8hb's is LLR-3jjgtw's refusals), and
LLR-p8uu47's other side is the receiver's refusals of an altered field. The
`Joiner` row and this change's new pairs stand as written above. SDD-na9nc3
still has ten LLRs here.

**Items whose abnormal side is carried by another item.** SDD-na9nc3's ten
LLRs in this file are *all* refusals except earlier forms of LLR-8528ud, LLR-34c5y3 and, since
2026-10-05, LLR-283x6f; its normal case is
the accept path of an earlier form of REQ-dwb9bq, carried by `happy_path_commits_when_root_matches_chain`.
SDD-sxp8hb's abnormal side is an earlier form of LLR-283x6f. SDD-swtd3w's constructor
`OrgId::new` takes exactly twenty bytes and is total over that type, and its
rejection vocabulary is output, not input. The one way untrusted bytes become
an `OrgId` is serde decoding inside a Wire message, and that abnormal input is
SDD-kwncn7's LLR-pkruy8 and `fuzz_envelope_decode`'s. *Added 2026-10-05 by the
merge gate, which found SDD-swtd3w accounted for nowhere in this section.*

**Seven items have no abnormal-input case of their own**, and that is a real
gap rather than an argued exemption. *Extended 2026-10-04 by review round 1,
which found the paragraph accounted for two of them and passed over four in
silence — under class C an uncovered item must be argued, and being unmentioned
is not an argument. Extended again the same day by review round 2, which found
the enumeration itself still incomplete: it skipped two further items, and one
of its two arguments claimed more than it could.*

| Item | The abnormal input it does not test |
|---|---|
| SDD-89es4z | a Persona created with a handle org-members will not accept. *Amended 2026-10-05 by the org-node type-safety change: `create_persona` now takes a parsed `Handle`, `Name` and `Surname` (an earlier form of LLR-pv2rdy), so this input can no longer be offered to it; the refusal moved to `PersonaDetails::parse` (an earlier form of LLR-rjphn4, under SDD-af5vnt in `2026-10-04-type-safety.md`), whose tests carry the abnormal case. The item's own interface still has none.* |
| the endpoint lifecycle (org-io's since S4 task T10) | an endpoint bind that fails *(org-io's keychain-refused bind is tested since T10: `a_locked_keychain_refuses_the_bind`)* |
| SDD-rx2yvy | `admit_member` given a malformed member or device key (→ `OrgNodeError::Chain("bad member key")`) or a handle already held (→ `OrgNodeError::Trie`) — **the most material of the seven**, because both are reachable from a join request a stranger composes. *Amended 2026-10-05 by the org-node type-safety change: `admit_member` now takes the parsed `JoinRequest`, and a malformed key or handle in a Join request is refused at `import_join_request` with `InvalidField` naming it (an earlier form of LLR-7yu2xy, under SDD-af5vnt in `2026-10-04-type-safety.md` — *amended 2026-10-05 by the org-node type-safety change, review round 7: it read "under SDD-vee2fq"*), so the first clause cannot reach `admit_member` and the `Chain("bad member key")` refusal no longer exists. A handle already held is still untested here.* *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): `admit_member` takes a parsed `Joiner` built by the app from the Invite reply it parsed; a handle already held is still untested, and now reaches `admit_member` from the app's parse rather than from `import_join_request`.* |
| SDD-ueh4tm | a `ChainOps` implementation whose `read_state` or `submit_update` fails. *Since review round 8 the item has one refusal case, the mock's wrong-epoch update (LLR-ryzr8m), but that is the mock refusing, not an implementation failing, so the line stands.* *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): `submit_update` leaves the trait, so the owed case is a `read_state` that fails, as it reaches a commit (an earlier form of LLR-qvp7mq names it among the refusals).* |
| org-io's chain-connection item (moved 2026-10-07) | every abnormal case it has. The item has no gated test of any kind — that is the deviation recorded at the end of this file — so it has no abnormal one either, and listing it here is the honest bookkeeping |
| SDD-msb6xh | none — `build_update_calldata` is total over its argument types, so there is no abnormal input to offer it. *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): the item now states that org-node holds no calldata; with no interface left it has no input, normal or abnormal.* |
| SDD-rq6nv4 | `multi_account_id` is total over its argument types. `build_dispatch_tx` is **not**: `runtime_call_to_tx` has three `WriteError::MalformedCall` arms, none of them tested, and this change made the function publicly reachable through `org_node::test_support`, so the abnormal input is now one line of test away. *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): both functions leave org-node, and the untested `MalformedCall` arms go with them to on-chain-client's chain writer, where they are owed.* |

**SDD-pa6p7w is no longer on this list.** Round 2 found it missing from it —
nothing reachable at this gate could make `ChainReader::get_org_state` return
`Err`, so the failure arm of an earlier form of LLR-rr558z and org-io's absence-from-failure LLR (moved 2026-10-07) was untested and
untestable — and this change now carries `FailingChain` in
`tests/verify_against_chain.rs` and
`a_chain_read_that_fails_is_refused_as_chain_not_as_absence`, which pins the
failure to `Chain` and asserts it is a different answer from absence.

Only SDD-msb6xh's line is argued rather than owed: a function total over its
argument types has no abnormal case, and inventing one would be the kind of
decoration this ledger has already cut elsewhere. **SDD-rq6nv4's used to be
argued and is not**: the old wording, "`build_dispatch_tx` can fail only on a
malformed `Value`, which this unit never constructs", was true of this unit's
own call graph and not of the function, and it stopped being a safe reading the
moment this change exported the function for tests. The honest statement is
that it is reachable and untested. The remaining five are owed and are recorded
in the Gaps section of this change's verification record.

## Evidence

Every LLR above, excepting org-io's chain-connection item's (moved 2026-10-07) absent ones, is carried by a test **in
`crates/org-node/tests`** — one of the thirteen harnessed targets there that
`verify_commands` names, or one of its three bolero targets. With `--lib`,
which carries no low-level requirement, the command runs fourteen harnessed
targets. *(Nineteen and twenty since the org-node type-safety change,
2026-10-05, which added `encoding_golden`, `node_value_types`,
`chain_read_state`, `persona_records`, `secret_redaction` and
`calldata_typed`.)* *Corrected 2026-10-05 by review round 8, which counted thirteen in
`crates/org-node/tests`.*

*Corrected 2026-10-04 by review round 1.* This paragraph used to say "eight
harnessed targets" and to allow evidence to sit "in a `#[cfg(test)]` module
under `crates/org-node/src`". Both were wrong: the count is fourteen, and `test_paths`
reads `verifies:` annotations **only** from `crates/org-node/tests`, which is the whole
reason this change relocated twenty-two tests. One `#[cfg(test)]` module
survives in `src` — `chain.rs`'s `mock_chain_returns_set_state` — and it carries
no annotation and no low-level requirement; org-io's absence-from-failure LLR (moved 2026-10-07) is evidenced from
`tests/` instead.

**A test that is already green has never been watched failing.** So each LLR is
discharged by red by mutation: a named mutation to the source, a named test
failing for that reason, the mutation reverted, and the file proved
byte-identical afterwards by SHA-256. Those attestations are in this change's
verification record, not here.

*Two exceptions, both stated rather than counted, and both found by review
round 2 when it checked this claim against the appendix.* **an earlier form of LLR-96z2k5** —
`import_join_request` stores nothing — and **an earlier form of LLR-7en38k** — the chain seam is a
trait object a substitute can stand in for — are true by the type system rather
than by a red. The first takes no `&mut self`, the second is checked by
`Box<dyn ChainOps>` compiling at all; no mutation can falsify either without
changing a signature, which is a different change and not a measurement. The
compiler is the evidence, and saying so is better than a row in the appendix
that implies a mutation nobody could write. LLR-f5kq88 and LLR-z8fubr's
distinctness clause are compile-enforced in the same way; all four are listed
together in Gap 4 of the verification record.

**One clause of an earlier form of LLR-283x6f is not discharged, and the requirement keeps it
anyway.** The sweep found two green; one has since been discharged:

- *"the trie the message **verified into**"* — **discharged 2026-10-04 by
  review round 5.** The sweep found checking the sender against the local
  record instead reddened nothing, and this section argued that no test could
  tell the two apart, because a removal reaches the node through the other
  receive operation. **That argument was false.** A removal of somebody else
  reaches the node through `receive_and_verify`; the gated
  `revocation_of_another_member_is_committed_not_self_deleted` already did
  exactly that. Review round 5 wrote the test the argument said could not be
  written: the Member being removed relays its own removal.
  `a_removal_relayed_by_the_member_it_removes_is_refused` now carries the
  clause.
- *"skipped on first admission"* — forcing the check to run on first admission
  too reddens nothing, because the administrator's device is in the trie it
  just admitted the node into, so the check would pass. The skip is therefore
  **unnecessary rather than merely unverified**, which is worth an owner's
  ruling: removing it would make one rule apply to both paths.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The owner ruled the sender check out on both paths, so an earlier form of LLR-283x6f then stated
that an update is committed whoever relayed it, and both clauses above — the
discharged one and the undischarged first-admission skip — describe a check
that no longer exists. The new requirement is to be discharged by red in the
usual way: reinstating the membership check must redden its test. The same
holds for the administrator-key exclusion an earlier form of LLR-exaa9u carried, which was never
evidenced and is removed.

**One low-level requirement cannot be discharged that way, and it is named
rather than quietly counted.** The IPv4-required item refines the
requirement on which loopback sockets must come up (both org-io's since S4
task T10) — IPv4 loopback
required, IPv6 optional — and both clauses are about a bind *failure* that no
gated test can produce. `check-trace.sh` is green over it because MISSING-TEST
is satisfied by the annotation alone, which is the third time this repository
has recorded that property, now in a second unit. It is kept by the owner
ruling that kept that requirement itself, and the `test-support` bind-address
constructor that would close it is not yet written.
