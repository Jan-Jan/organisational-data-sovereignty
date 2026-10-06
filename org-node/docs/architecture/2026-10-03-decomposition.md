# org-node — software decomposition, items and low-level requirements

The first architecture ledger for the `org-node` unit: nineteen software items
and the low-level requirements that refine the **twenty** high-level
requirements in `org-node/docs/requirements/`. The unit's SOUP inventory is
`org-node/docs/architecture/soup.md`.

*Corrected 2026-10-04 by review round 2. This sentence read "nineteen" against
a unit that holds twenty, and the one it omitted was the one that mattered:
**REQ-d9g6nt** arrived with `master` `0f85cb9` while this change was open, is
verified directly by `fuzz_first_admission_base` and four `admission_sender`
tests, and was traced by no item here and refined by no low-level requirement.
No gate could have caught that — `check-trace.sh` has an UNTRACED-DESIGN rule
and no UNTRACED-REQUIREMENT rule — so this ledger was the only place it could
be. It is now refined by LLR-ag9mgm, LLR-rjg3m2 and LLR-j6j95z below. Two of
the twenty are traced by nothing by design: REQ-ysyu9g and REQ-q92yac are
expectations on provider units, not behaviours of this one.*

Safety class C throughout (`org-node/.guardrails/config.yaml`, no per-item
override), so every item below carries LLRs — **with one exception, SDD-z85ux9,
which carries none.** That is a deviation, not an override, and the section
"The item with no low-level requirements" at the end of this file records it in
full rather than leaving a reader to notice the absence.

## Overview

The unit's overview — the decomposition picture and the segregation position —
lives in the Overview section of `org-node/docs/architecture/README.md`, which
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
SDD-89es4z, SDD-rx2yvy, SDD-8cpyfa, SDD-72ddm6, SDD-b8tuv3 and SDD-z85ux9.
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
root, the apply, the chain read with its epoch, and the root match. LLR-mcdh85
now states the absence of the signature check, and LLR-fuq379 states that the
four chain-free checks run before the chain is read.

## SDD-swtd3w — Value types and the rejection vocabulary

`org-node/src/ids.rs`, `org-node/src/error.rs`, `org-node/src/types.rs`

**SDD-swtd3w**: the identifier that keys an Organisation's on-chain slot,
together with the vocabulary in which every refusal is reported — a typed
variant per rejection path **on the receive-and-commit path**, so a caller can
state *why* a change was refused rather than only that it was.
traces: REQ-gju89b, REQ-9g6as6, REQ-bcxz96, REQ-y7tsft, REQ-8jb4ny, REQ-ech45n, REQ-fwfku9, REQ-8amu2a, REQ-tqap3r, REQ-kt877x, REQ-yp75u9, REQ-c29s93, REQ-bwx7eg, REQ-vxqc5g, REQ-szq3ud, REQ-3dsweu

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
`types.rs` loses `OrgSecret` and `InviteId` (LLR-qsjde3, LLR-ms8njy), so the
secrets it holds are the member seed, the device seed and the Organisation
private key; and `OrgNodeError` gains `MalformedMessage`, `OrgKeyMismatch`,
`RevocationNotHeld` and `RevocationNotForThisDevice` (LLR-j5vbqj, in
`2026-10-07-org-key-pair.md`). The item traces
REQ-c29s93, REQ-bwx7eg, REQ-vxqc5g, REQ-szq3ud and REQ-3dsweu too. Notes below that list
the Organisation secret among `types.rs`'s secrets are history.

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`, independent
review round 1, finding-1):* traces REQ-yp75u9 too, for the refusal
`PersonaAlreadyBound` that LLR-mxskg9 adds.

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`):* traces
REQ-kt877x too, for the refusal `AdmissionNotOurs` that LLR-mxskg9 adds.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
`OrgNodeError::BadSignature` is removed with the Envelope signature, and the
item gains the refusals of the provisional-update and expected-admission paths
— `ProvisionalLimit`, `AdmissionNotExpected` and `NoProvisionalUpdate` — stated
by LLR-mxskg9 in `2026-10-06-chain-authority.md`,
so it traces REQ-fwfku9, REQ-8amu2a and REQ-tqap3r too.

*Amended 2026-10-05 by the org-node type-safety change, which re-homed its
value types here at its merge of this file.* The item also owns
`org-node/src/types.rs`: the secrets (member seed, device seed, Organisation
secret), the Organisation public key and the tag types for the chain account,
the Persona identifier, the epoch and the Sequence number, with their
construction, formatting and serialised form — so it traces REQ-y7tsft too —
and the two variants that change adds to `OrgNodeError`, `InvalidKey` and
`InvalidField { field, reason }`. Its low-level requirements for them,
LLR-sz4xhc, LLR-s7whrn and LLR-ayrdr8, are in `2026-10-04-type-safety.md`.
The count of `OrgNodeError::Chain(String)` construction sites below is fifty
on the tree it was measured on; after that change it is **forty-five**
(`grep -o 'OrgNodeError::Chain(' -r org-node/src | wc -l`, 2026-10-05), the
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
`worktree-person-shared-types`; reworded the same day by
docs/plans/2026-10-05-switch-trim.md: on that branch `types.rs` also holds
`OrgPrivateKey`, and `OrgPublicKey` is an X25519 key. LLR-mmdu38 and
LLR-ayrdr8 are amended in place, in `2026-10-04-type-safety.md`. LLR-322xfu
(`OrgPrivateKey`), LLR-3jjgtw (`OrgPublicKey`'s parse) and LLR-sj7cd5
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
satisfies: REQ-9g6as6, REQ-bcxz96

*Narrowed 2026-10-04 by review round 3, because the clause as written was
**false of the tree**. It said "each rejection path of this unit", and the unit
has **fifty** `OrgNodeError::Chain(String)` construction sites carrying about
thirty-five distinct messages behind one variant whose `Display` is
`"chain read failed: {0}"`. A wrong store passphrase, a malformed invite blob,
a missing persona, a Loopback revocation with no peer address and an iroh send
failure are all the same variant as one another and as an actual chain read
failure. `org-node/src/error.rs`'s own doc comment has always carried the
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

`org-node/src/keys.rs`

**SDD-sxp8hb**: the key pairs a node holds and how each is made from its
secret: the ed25519 device keypair, whose verifying key is the node's
DevicePublicKey in the trie and its transport identity, and the X25519 key
pairs behind a Member-as-a-group key and behind the Organisation public key,
each from its own secret (the member seed and the Organisation private key).
The Organisation public key a node publishes is built from its key pair only
through `OrgPublicKey`'s parse, `person`'s X25519 validity check. No key here
signs or verifies anything.
traces: REQ-ag6kqm, REQ-8jb4ny, REQ-ech45n, REQ-y7tsft

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item described one ed25519 keypair whose verifying key both signed changes to
membership as the `P2pMemberKey` and served as the `P2pDeviceKey`. Nothing
signs now, and a Member-as-a-group key is an X25519 key from its own seed.
LLR-3jjgtw and LLR-98ufry also constrain this item; they are in
`2026-10-05-unsigned-envelope.md`. *(Corrected 2026-10-05 by review round 3,
finding-9: this also named LLR-sj7cd5 and LLR-322xfu, which constrain
`create_organisation` (SDD-89es4z) and `types.rs` (SDD-swtd3w) and sit under
those items.)* LLR-56hc77 and LLR-bwb9pu,
amended in place, are in `2026-10-04-type-safety.md`.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said the `P2pMemberKey` "signs a change to membership". An Envelope
carries no signature, so `SigningKeypair::sign` and `keys::verify`, whose only
caller was the Envelope, are removed; the item still traces REQ-ag6kqm, which
now requires that no signature be checked, and LLR-na7p4w states that nothing
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
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item also required the same signature over the same message. Nothing in
org-node signs. It is derived now, since no requirement asks for a signature;
it is assessed in `org-node/docs/risk/2026-10-05-envelope-authenticity.md`.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This also required "the same signature over the same message" and satisfied
REQ-ag6kqm, the signature requirement. Nothing in org-node signs now, so the
signature clause has nothing to range over; what remains is that a Persona's
keys survive the round trip through its stored seeds, which no requirement
states, so it is derived and assessed in
`org-node/docs/risk/2026-10-06-chain-authority.md`.

*(Amended 2026-10-05 by the org-node type-safety change: this read "a keypair
rebuilt from `to_seed`". That change removed `to_seed` and `from_seed`
(LLR-56hc77, in `2026-10-04-type-safety.md`), so the seed comes back typed, as
a `MemberSeed` or a `DeviceSeed`. `a_keypair_rebuilt_from_its_seed_signs_identically`
checks both roles.)*

**LLR-ctzkv7**: a Persona's two public keys come from two seeds by two
algorithms. `X25519Keypair::member_key` is the X25519 public key of the member
seed (RFC 7748: the clamped seed times the base point) and is always a valid
`PersonPublicKey`. `SigningKeypair::device_key` wraps the ed25519 verifying
key of the device seed.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said `member_key()` and `device_key()` wrap one verifying key. A
Member-as-a-group key is now an X25519 key from its own seed.

*Parent stale, noted 2026-10-05 by review round 4 (finding-5).* Since the
amendment of 2026-10-05, REQ-ztdza4 and REQ-xa6smf check nothing about a
sender's keys, so neither implies this item, and `satisfies:` above names
them only until it is re-traced. By owner ruling of 2026-10-05 the re-trace is
left to chain-authority's change 1.

*Re-traced 2026-10-06 (change `worktree-org-node-chain-authority`, after the
merge of master `5f7c177`).* No requirement of this unit states how a
Persona's keys are derived: the X25519 rule is `person`'s, and the
DevicePublicKey is the iroh identity. The item is derived, and assessed in
`org-node/docs/risk/2026-10-06-chain-authority.md`. *(Amended 2026-10-06,
independent review round 2, finding-10: "the DevicePublicKey" read "the
device key", the term `org-node/docs/CONTEXT.md` retires for new text.)*

**LLR-na7p4w**: `VerifyContext` holds the expected Organisation, the
Sequence-number guard and the last committed epoch, and no key:
`verify_envelope_against_chain` decides from the Envelope, that context and
the chain reader alone.
satisfies: REQ-ag6kqm

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item stated that `verify` accepts a signature the keypair produced. org-node
verifies no signature (REQ-ag6kqm); `keys::verify` is gone.

**LLR-9fvb3y**: a genuine Envelope relayed by a device other than the one that
built it is committed on both Receive operations when it verifies against the
chain: the delivering device is not an input to verification.
satisfies: REQ-ag6kqm

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item stated that `verify` refuses a signature by any other key. Nothing is
signed, and the device that delivers an Envelope decides nothing (REQ-ag6kqm,
REQ-xa6smf, REQ-ztdza4).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-na7p4w said `verify` accepts this keypair's own signature, and LLR-9fvb3y
that it refuses another key's. `keys::verify` and `SigningKeypair::sign` are
removed with the Envelope signature (their only caller). LLR-na7p4w now states
that absence, which REQ-ag6kqm requires and the compiler enforces, as for
LLR-836z24. LLR-9fvb3y is rewritten to the nearest true statement about the
keys that remain — the other side of LLR-e58j8m — and is derived, assessed in
`org-node/docs/risk/2026-10-06-chain-authority.md`.
*(Merged 2026-10-06 with master `5f7c177`, which amended LLR-na7p4w and
LLR-9fvb3y in place for the same ruling: master's texts stand, and this note
describes this change's texts before that merge.)*

## SDD-kk2y3e — The Envelope

`org-node/src/envelope.rs`

**SDD-kk2y3e**: the wire form of a change to membership. It binds the
Organisation identifier, the Sequence number and the encoded Change set, and
carries no signature. The item also owns the decode a receiver performs on it.
traces: REQ-ag6kqm, REQ-gju89b, REQ-9g6as6

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item described a signed transcript and the signature check a receiver ran
before anything else. (The section heading read "The signed Envelope".)

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This item was "the authenticated wire form": the transcript that is signed, and
the signature check a receiver ran before anything else. The owner ruled that
an Envelope carries no signature and that the on-chain root at a newer epoch is
the sole authority over a received Change set (REQ-ag6kqm as amended).
`SignedDeltaEnvelope` is renamed `Envelope`; its `signature` field,
`verify_signature`, the `sig_bytes` serde helper and `build`'s key argument
are removed. The heading keeps its old title because a heading is part of the
merged item's text; the item's own sentence above is what it now is.
*(Merged 2026-10-06 with master `5f7c177`: the heading and the item text are
master's, which retitled the section "The Envelope".)*


**LLR-e7s4ye**: an Envelope's wire form is the postcard encoding of the
Organisation identifier's twenty bytes, then the Sequence number, then the
Change set bytes, in that order and nothing else; no transcript is signed.
satisfies: REQ-ag6kqm

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item described the signed transcript, which no longer exists. The three fields
it bound are what the wire form carries.

**LLR-p8uu47**: `Envelope::build` sets the Organisation identifier and the
Sequence number it is given, and sets as the Change set bytes the postcard
encoding of the delta. That encoding is canonical: decoding the transmitted
bytes and encoding them again reproduces them exactly.
satisfies: REQ-gju89b

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
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
satisfies: REQ-ag6kqm, REQ-gju89b

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said altering the Organisation identifier breaks the signature. With no
signature, the receiver's expected Organisation is what refuses it.

**LLR-cs4mpb**: an Envelope whose Sequence number is at or below the
receiver's mark is refused with `StaleSeq` before its Change set is decoded,
and one whose Sequence number is above the mark but not the epoch of the chain
state it is verified against is refused with `SeqNotEpoch`.
satisfies: REQ-ag6kqm

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said altering the Sequence number breaks the signature. The mark
(REQ-6yu72z) and the chain's epoch (REQ-txvtm9) refuse it now.

**LLR-9sknpa**: Change set bytes that do not decode are refused with
`MalformedDelta`, and a Change set that decodes but does not extend the
receiver's record is refused with `DeltaBaseMismatch`.
satisfies: REQ-ag6kqm

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said altering the delta bytes breaks the signature. The decode and the
base-root check refuse such bytes now, and the root match refuses any that
survive both.

**LLR-pzde8b**: an `Envelope` holds exactly the Organisation identifier, the
Sequence number and the Change set bytes; it has no signature field and no
`verify_signature`.
satisfies: REQ-ag6kqm

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item stated that `verify_signature` refuses any key but the signer's. It is
gone with the signature.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-ybn5pr, LLR-cs4mpb and LLR-9sknpa said that altering the identifier, the
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

`org-node/src/sequence.rs`

**SDD-d8ktxa**: the monotonic high-water mark that refuses a sequence number
already seen, and the separation of asking from committing that lets the
decisive root check run between the two.
traces: REQ-6yu72z, REQ-mr5abb

**LLR-wx3php**: `check` accepts a sequence number **strictly** greater than the
mark and refuses every other with `StaleSeq` carrying both the offered number
and the mark.
satisfies: REQ-6yu72z

**LLR-f5kq88**: `check` does not move the mark.
satisfies: REQ-mr5abb

**LLR-uc7cej**: `advance` moves the mark to the offered number only when that
number is greater than the mark, and otherwise leaves it where it was.
satisfies: REQ-mr5abb

**LLR-duwz79**: `from_last_seen` starts the guard at the given mark, and
`last_seen` reports the mark currently held.
satisfies: REQ-mr5abb

## SDD-na9nc3 — Verify-against-chain

`org-node/src/verify.rs`

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
traces: REQ-wp2nyc, REQ-bvh8v6, REQ-8gz8bu, REQ-gju89b, REQ-ag6kqm, REQ-6yu72z, REQ-mr5abb, REQ-nhe2zu, REQ-txvtm9, REQ-bcxz96, REQ-f2k4tr

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item put an authenticity check first, on an unauthenticated sender. Nothing
about the sender is checked (REQ-ag6kqm). LLR-9f5hmr, in
`2026-10-05-unsigned-envelope.md`, also refines this item.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said "the cheap authenticity checks come first so that untrusted bytes
are never decoded on behalf of an unauthenticated sender". There is no
authenticity check now: an Envelope carries no signature, and bytes from any
peer reach the decoder, which REQ-9g6as6 and LLR-hs7g6j already require to be
total. What the order protects instead is the chain read (RC-mj6gjq,
REQ-f2k4tr), so the item traces REQ-f2k4tr and LLR-fuq379, in
`2026-10-06-chain-authority.md`, exposes the
chain-free half as `verify::check_chain_free`.

**LLR-4fbuy8**: an envelope naming another Organisation is refused with
`OrgIdMismatch` **before the Change set it carries is decoded**.
satisfies: REQ-gju89b

**LLR-mcdh85**: `verify_envelope_against_chain` checks no signature and no
sender: after the Organisation binding, the only check before the Change set
bytes are decoded is the Sequence-number mark, and bytes that do not decode
are refused with `MalformedDelta`.
satisfies: REQ-ag6kqm

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item required the signature to be checked before the delta bytes are decoded,
refusing with `BadSignature`.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said the signature is checked before the delta bytes are decoded and a
failure refused with `BadSignature`. The owner ruled that an Envelope carries
no signature and that the decision rests on the Organisation, the Sequence
number, the Change set and the chain alone (REQ-ag6kqm as amended);
`VerifyContext.author_member_key` and `BadSignature` are removed.

**LLR-xpbkp5**: the sequence check runs **before the delta bytes are decoded**,
so a replayed envelope is refused without its Change set bytes being parsed.
satisfies: REQ-6yu72z

**LLR-992mbf**: the delta's base root must equal the local trie's current root,
and a mismatch is refused with `DeltaBaseMismatch` before the delta is applied.
satisfies: REQ-wp2nyc

**LLR-8m99q2**: an Organisation for which the chain reports no state is refused
with `OrgNotOnChain`, distinctly from a chain read that failed, which is
refused with `Chain`.
satisfies: REQ-bvh8v6

**LLR-vf5mjx**: the epoch the chain reports must be **strictly** greater than
the last committed epoch, and otherwise the change is refused with `StaleEpoch`
carrying both.
satisfies: REQ-8gz8bu

**LLR-8n95rf**: the candidate trie produced by applying the delta must verify
against the root the chain reported, refused with `RootMismatch`; this is the
**last** check, so no earlier step can have committed anything on its behalf.
satisfies: REQ-wp2nyc, REQ-nhe2zu, REQ-txvtm9

**LLR-d6kvbx**: a verification that succeeds returns a sequence guard whose
mark is the envelope's sequence number, and a verification that fails returns
no guard at all — so a refused envelope cannot move any mark.
satisfies: REQ-mr5abb

**LLR-8hwqru**: the local trie passed in is not mutated on any path; a
successful verification returns a new trie and leaves the caller's untouched.
satisfies: REQ-wp2nyc

**LLR-hs7g6j**: no sequence of bytes offered as an envelope to this function
causes a panic or an abort.
satisfies: REQ-bcxz96

## SDD-pa6p7w — The chain as a read oracle

`org-node/src/chain.rs`; in `org-node/src/chain_read.rs`, `org_state_from_chain`
and `OrgStateCache` *(amended 2026-10-05, see below)*

**SDD-pa6p7w**: the read-only view of what the chain says an Organisation's
membership root and epoch are, as an interface rather than a client — the root
returned here must come from a path the sender of a change does not control,
and expressing it as a trait is what makes that substitutable and testable.
traces: REQ-bvh8v6, REQ-nhe2zu, REQ-txvtm9, REQ-8jb4ny

*Amended 2026-10-05 by the org-node type-safety change.* That change split the
parse of a chain state and its cache out of `OnChainReader` into
`org_state_from_chain` and `OrgStateCache`, an implementation of
`ChainReader` that a gated test can drive without a chain, and stated what it
serves when the chain's state is refused (LLR-mmdu38, in
`2026-10-04-type-safety.md`). Those two parts of `chain_read.rs` are this
item's; the rest of the file — `OnChainReader` and its `refresh`, which talk
to a chain — stays SDD-z85ux9's.

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
Organisation state once, after the chain-free checks (LLR-9ew26y), where they
used to read it first to obtain the signing key; `ChainOpsReader` wraps that
one state. Nothing read here is used as a key: the Organisation state
contributes its root and epoch to the decision and nothing else (LLR-37cj3n).

**LLR-rm9x4z**: `get_org_state` returns `None` for an Organisation with no
on-chain slot and `Some` for one that has state, distinguishing **absence from
failure** — a failure is the `Err` arm — with one exception, stated by
LLR-mmdu38: after a refresh whose fetched state holds an Organisation public
key the parse refuses, `OrgStateCache` (and `OnChainReader` through it) returns
`Ok(None)` for that Organisation, which is on chain, until a refresh succeeds.
That refusal is the `Err` of the refresh, not of `get_org_state`.
satisfies: REQ-bvh8v6

*(Amended 2026-10-05 by the org-node type-safety change, review round 7: this
said `None` only for an Organisation with no on-chain slot. By owner ruling
(2026-10-05, LLR-mmdu38: fail closed) the cache now answers `Ok(None)` for an
Organisation that is on chain once a fetched state is refused at parse, so
verify-against-chain refuses with `OrgNotOnChain`. That reports a refused
state as absence. It rejects the Change set, as REQ-bvh8v6 requires for
absence, and it is recorded in the type-safety design's "Observable changes"
under "Chain read". The production Receive path is not affected:
`SubxtChainOps::read_state` caches nothing and returns a refusal at parse as
an error, and `ChainOpsReader` only wraps a state that read has just
returned.)*

## SDD-kwncn7 — The wire frame and its bound

`org-node/src/transport/wire.rs`, `org-node/src/transport/mod.rs`
(`TransportError`, `MAX_FRAME`)

**SDD-kwncn7**: the framing of one Wire message on the channel — what a frame is
made of, the one-mebibyte ceiling on a body, and the refusal of anything over
it on both the sending and the receiving side.
traces: REQ-eg5j8u, REQ-9g6as6, REQ-y7tsft, REQ-8amu2a, REQ-c29s93, REQ-3dsweu

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
A frame carried one shape of Wire message, with an optional Organisation
secret and an optional invite identifier. The Wire message is now an enum of
two kinds, Organisation information (always carrying the record snapshot and
the Organisation private key) and revocation (carrying neither), and no kind
carries an invite identifier (LLR-js9dsu, LLR-ecxc76 in
`2026-10-07-org-key-pair.md`; LLR-ms8njy amended).
A body of the first kind without its key does not decode, so the item traces
REQ-c29s93 and REQ-3dsweu too. The notes below that speak of the Organisation
secret or the invite identifier on the wire are history.

*Amended 2026-10-06 (owner ruling on pre-emption, change
`worktree-org-node-chain-authority`):* a Wire message carries the invite
identifier of the admission it delivers (LLR-ms8njy, in
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
`MAX_FRAME` (LLR-jq7qh7, under SDD-af5vnt), by the owner's ruling that the
bound is the wire frame's. The two constants are separate so a change to one
is a decision about the other.

**LLR-fa7jt8**: `encode_frame` emits the body's length as four little-endian
bytes followed by the encoded body, and nothing else.
satisfies: REQ-eg5j8u

**LLR-er2x8n**: a Wire message encoded by `encode_frame` and decoded by
`decode_body` yields a Wire message equal to the original, so a frame within the
bound is delivered unchanged.
satisfies: REQ-eg5j8u

**LLR-sc6zuh**: `encode_frame` refuses a body larger than `MAX_FRAME` with
`FrameTooLarge` carrying the measured length, rather than emitting a frame.
satisfies: REQ-eg5j8u

**LLR-8kh3zf**: `decode_body` refuses a body larger than `MAX_FRAME` with
`FrameTooLarge` **before decoding any of its bytes**.
satisfies: REQ-eg5j8u

**LLR-pkruy8**: `decode_body` refuses bytes that are not a valid encoded
Wire message with `Malformed`, and does not panic.
satisfies: REQ-9g6as6

## SDD-8uyg4s — The authenticated endpoint

`org-node/src/transport/endpoint.rs`

**SDD-8uyg4s**: the iroh endpoint whose identity *is* the device's key, so a
completed QUIC handshake is cryptographic evidence of which device is on the
other end — together with the two bind modes and what each one is allowed to
reach.
traces: REQ-ztdza4, REQ-xa6smf, REQ-db6s7q, REQ-2wzfzv, REQ-eg5j8u

**LLR-ygn78w**: the endpoint's iroh `EndpointId` is byte-identical to the
device key of the keypair it was bound with.
satisfies: REQ-ztdza4, REQ-xa6smf

**LLR-v873fx**: `recv_one` returns the remote device key taken from the key
authenticated by the QUIC handshake, **never from the Wire message body**, so the
sender identity a caller cross-checks cannot be chosen by the sender.
satisfies: REQ-ztdza4, REQ-xa6smf

*Note 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* No
caller in org-node compares this key with anything since that ruling
(REQ-xa6smf, REQ-ztdza4 as amended); the item still holds of the key
`recv_one` returns, and "a caller cross-checks" names a use no receive path
makes.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The requirement stands as a property of `recv_one`, but no caller cross-checks
the sender now: the owner ruled that nothing about an Envelope's sender is
checked (REQ-xa6smf and REQ-ztdza4 as amended, LLR-j83kc8 and LLR-u6rq4s).
The authenticated key is returned and not consulted.

**LLR-wwunf4**: in Loopback mode the endpoint binds only loopback addresses and
offers a peer only loopback addresses for dialling.
satisfies: REQ-db6s7q

**LLR-6adc99**: in Loopback mode an IPv4 loopback socket is required and an
IPv6 loopback socket is optional, so the endpoint is provided when only the
latter cannot be bound.
satisfies: REQ-2wzfzv

**LLR-wx77j5**: `node_addr_for_dial` reports the endpoint's identity together
with its currently bound sockets, rather than with addresses from asynchronous
discovery that may not have run.
satisfies: REQ-db6s7q

**LLR-k2y6nn**: `recv_one` reads at most `MAX_FRAME + 4` bytes from a stream,
so a peer cannot cause unbounded allocation by never finishing.
satisfies: REQ-eg5j8u

**LLR-2smrvx**: `recv_one` discards the four-byte length prefix without
comparing it with the body, and a stream shorter than four bytes is decoded
whole. The frame ends where the QUIC stream ends, its size is bounded by
LLR-k2y6nn's read limit and by `decode_body`, and the prefix is not
consulted.
satisfies: derived

*Added 2026-10-05 by review round 8, which measured that checking the prefix against the body left the gate green.
`the_length_prefix_is_not_checked_against_the_body` pins it, and since review
round 9 also sends three bytes and sees them refused as Malformed by the
decoder rather than by the read.*

## SDD-af5vnt — The encrypted persona store

`org-node/src/store.rs`

**SDD-af5vnt**: where every secret this node holds rests — the member and
device seeds and the Organisation private key — and the form the file takes,
which is ciphertext under a passphrase-derived key and nothing else.
traces: REQ-hzm4kt, REQ-qn2erx, REQ-y7tsft, REQ-ech45n, REQ-xs4ab8, REQ-fwfku9, REQ-8amu2a, REQ-hhva9d, REQ-ju6vn2, REQ-stx9v3

*Amended 2026-10-06 (owner ruling on rotation, change
`worktree-org-node-org-key-pair`):* traces REQ-stx9v3. Every provisional
update the store holds now holds the private key of the fresh Organisation
key pair drawn for it, not only a genesis update, and a provisional update is
identified by its public key as well as its root (LLR-qjz3q4, LLR-95753m and
LLR-7cmp38 amended).

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This named "the Organisation secret" among the secrets the store holds. The
Organisation secret is removed; every Organisation record now holds the
Organisation private key, required, on the creating node and on every
admitted node alike (LLR-byjvd9 in
`2026-10-07-org-key-pair.md`), so the item traces
REQ-ju6vn2 too. An expectation names the Organisation alone again
(LLR-95753m amended).

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`):* an
expectation the store holds names an invite identifier (LLR-95753m), so the
item traces REQ-8amu2a; and the genesis provisional update holds the
Organisation private key until it commits (LLR-qjz3q4, REQ-ech45n).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The store holds no pending Invites (`PendingInvite`,
`StoreData.pending_invites`), because the invitation exchange left org-node for
the app, and no administrator key (`OrgRecord.admin_member_key`). It now holds
the provisional updates a Persona has built and the Organisations the app has
declared it expects to join, with the 1 MiB bound on the first. The
requirements stating them, LLR-95753m, LLR-jq7qh7 and LLR-nvn3wk, are in
`2026-10-06-chain-authority.md`, so the item
traces REQ-xs4ab8 and REQ-fwfku9 too. The record of an Organisation keeps its
proxy account as opaque data (LLR-dzte8x).

*Amended 2026-10-05 by the org-node type-safety change:* the item also owns
the store encryption key's type, the records the store holds — each field in
its typed form, parsed when the store is opened — the Persona details a
Persona is created from, and what the debug rendering of a value holding a
secret may show, so it traces REQ-qn2erx and REQ-y7tsft too. Its low-level
requirements for them, LLR-bwb9pu, LLR-scgk5j, LLR-g76zqd and LLR-q6n25z, are
in `2026-10-04-type-safety.md`.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
LLR-8bum44 — a value its type's parse refuses is refused, naming the field —
now sits under this item too, moved from SDD-vee2fq. Most of what it
constrains is in `store.rs`: `PersonaStore::open`, the `Raw…` mirrors of the
four records and of `StoreData`, `parse_field`, and the member-snapshot parse
that `first_admission_base` decodes through. The Join request and Invite
decode is named by SDD-vee2fq, the import operations by SDD-rx2yvy, and
`first_admission_base` by SDD-8cpyfa. LLR-g76zqd and LLR-bwb9pu also
constrain code outside `store.rs`: LLR-g76zqd is named by SDD-vee2fq
(`JoinRequest`) and SDD-89es4z (`create_persona`), and LLR-bwb9pu by
SDD-kwncn7 (`WireMessage`) and SDD-sxp8hb (`SigningKeypair`).)*

*(Note 2026-10-05, at the merge of master `1feb608` into
`worktree-person-shared-types`; reworded the same day by
docs/plans/2026-10-05-switch-trim.md: on that branch the store also holds the
Organisation private key, in `OrgRecord.org_private_key` (LLR-3fwykc,
LLR-2dvhz8, in `2026-10-05-unsigned-envelope.md`). LLR-bwb9pu and LLR-8bum44
are amended in place, in `2026-10-04-type-safety.md`.)*

*(Re-traced 2026-10-05 by review round 3, finding-9, change
`worktree-person-shared-types`: the store keeps the Organisation private key,
so the item traces REQ-ech45n. LLR-3fwykc and LLR-2dvhz8 sit under this item
in `2026-10-05-unsigned-envelope.md`.)*

**LLR-wusj89**: the store file is exactly a twenty-four byte nonce followed by
the AEAD ciphertext of the encoded store, with no cleartext header.
satisfies: REQ-hzm4kt

**LLR-8mfjey**: a fresh twenty-four byte nonce is drawn from the caller's
random source on every save.
satisfies: REQ-hzm4kt

**LLR-t4u66w**: opening a store file with a passphrase other than the one it
was written under yields a typed error and no data.
satisfies: REQ-hzm4kt

**LLR-q5n28x**: a store file shorter than the nonce is refused as too short
rather than read as a nonce and an empty ciphertext.
satisfies: REQ-hzm4kt

**LLR-s78sh7**: no member seed, device seed or Organisation private key the
store holds appears anywhere in the written file in clear.
satisfies: REQ-hzm4kt

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This named the Organisation secret, which is removed; REQ-hzm4kt, amended the
same day, names the Organisation private key in its place.

## SDD-vee2fq — Out-of-band exchange blobs

`org-node/src/blobs.rs`

**SDD-vee2fq**: the boundary at which org-node stops: it holds no
copy-pasteable blob. The Invite and the reply to it, and the armour they travel
in, are the app's; what reaches org-node from an admission's other party is a
`Joiner` of parsed values and, on the joiner's side, a declared expectation of
admission. `org-node/src/blobs.rs` is removed.
traces: REQ-xa6smf, REQ-9g6as6, REQ-qn2erx

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This item was "the two copy-pasteable blobs that bootstrap a relationship
before any channel exists — the Invite an administrator hands out, and the
Join request a prospective member hands back — and the armour they travel in".
The owner ruled that the invitation exchange is outside org-node and happens
in the app (`app/docs/requirements/2026-10-06-invitation.md`),
and that Invites, Join requests, pending Invites and the Invite's dialling
address leave org-node. The item is kept, as merged items are, and now states
that boundary; its four requirements are rewritten to it.

*Amended 2026-10-05 by the org-node type-safety change:* the blobs' fields are
held typed, and a decoded value that does not parse is refused, naming the
field for a Join request (`decode_join_request`); the requirement stating it,
LLR-8bum44, is in `2026-10-04-type-safety.md`, so the item traces REQ-qn2erx
too.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
LLR-8bum44 now sits under SDD-af5vnt, which owns most of what it constrains.
This item is also constrained by it, for `decode_join_request`,
`RawJoinRequest` and the Invite decode, and by LLR-g76zqd, under SDD-af5vnt,
for `JoinRequest`'s typed fields.)*

**LLR-g9vmbx**: org-node defines no Invite and offers no operation that
exports, imports or consults one: `OrgService` has no `export_invite` or
`import_invite`, and a first admission is committed on the chain anchor
without reference to any Invite.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the Invite carries the Published signing key; the chain's key is the
Organisation public key.

*Parent stale, noted 2026-10-05 by review round 4 (finding-5).* Since the
amendment of 2026-10-05, REQ-xa6smf requires no Invite, so it does not imply
this item, and `satisfies:` above names it only until the item is re-traced.
By owner ruling of 2026-10-05 the re-trace is left to chain-authority's
change 1.

*Re-traced 2026-10-06 (change `worktree-org-node-chain-authority`).* That
org-node defines no Invite at all is the owner's placement of the invitation
exchange in the app (ruling of 2026-10-05), not a requirement of this unit;
REQ-xa6smf asks only that a first admission need none. The item is derived,
and assessed in
`org-node/docs/risk/2026-10-06-chain-authority.md`.
What a joining node keeps in the Invite's place is an expectation
(LLR-9zfnmb, REQ-8amu2a).

**LLR-kkj64b**: `admit_member` takes the joiner as a `Joiner` holding a parsed
`Handle`, `Name`, `Surname`, Member-as-a-group key (`PersonPublicKey`) and
`DevicePublicKey`, and the Member it adds to the record carries exactly those
five values; org-node defines no Join request.
satisfies: REQ-xa6smf

**LLR-8qxwst**: no operation of org-node takes armoured text: every value that
arrives from another device reaches it as bytes inside a Wire message frame
(`decode_body`) or as typed values from the app.
satisfies: REQ-9g6as6

**LLR-tcft2r**: `first_admission_base` refuses bytes that are not a valid
encoded record snapshot with a typed error rather than panicking, and extends
nothing.
satisfies: REQ-9g6as6

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-g9vmbx and LLR-kkj64b said an Invite and a Join request round-trip
through their encoding; LLR-8qxwst and LLR-tcft2r that bad Base64 and a bad
encoded blob are refused without panicking. All four types and their armour
leave org-node for the app, which parses the Invite and its reply at its own
edge. LLR-g9vmbx and LLR-8qxwst now state the absence, which the compiler
enforces as for LLR-836z24. LLR-kkj64b states what replaces the Join request
at `admit_member`. LLR-tcft2r is rewritten to the remaining place where bytes
another device composed are decoded into a record — the snapshot a first
admission extends — which `fuzz_first_admission_base` already exercises.

## SDD-msb6xh — Update calldata

`org-node/src/chain_write/calldata.rs` (`build_update_calldata`,
`UPDATE_SELECTOR`, `revive_update_runtime_call`, `update_calldata` *(added
2026-10-05, see below)*)

**SDD-msb6xh**: the absence of update calldata from org-node: org-node builds
no bytes that ask the contract to move a Membership root. The calldata, its
selector and the `Revive.call` runtime call are built in on-chain-client's
chain writer
(`on-chain-client/docs/architecture/2026-10-06-chain-write.md`);
what org-node hands the app for a submission is a provisional update's
resulting root, Organisation public key and the epoch its record holds.
traces: REQ-nhe2zu, REQ-txvtm9

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This item was "the exact bytes that ask the contract to move an Organisation's
Membership root forward — the one part of the write path that is pure". The
owner ruled that org-node does not write to the chain: `chain_write/`, of
which `calldata.rs` is part, moves to on-chain-client, which states the
calldata's layout under its own chain-writer item. The item is kept, as merged
items are, and its four requirements now state that org-node holds none of it.
They are compile-enforced, as LLR-836z24 is. The tests that carried them —
`chain_write_pure` and `calldata_typed` — move with the code and are
re-annotated to on-chain-client's requirements.

**LLR-rv4vux**: org-node has no `build_update_calldata` and emits no contract
calldata; the hundred-byte `update` layout is on-chain-client's.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* The
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
and `submit::submit_update` (SDD-z85ux9's) calls it too. `calldata.rs` now
holds three functions, all this item's. A typed-calldata golden test in
`org-node/tests/calldata_typed.rs` pinned its output (LLR-ayrdr8).)*

*Added 2026-10-04 by review round 6, which found this function inside
SDD-z85ux9, the item with no low-level requirements, under a list headed "the
asynchronous functions only". It is neither asynchronous nor unreachable, and a
test in `org-node/tests/chain_write_pure.rs` pinned it.*

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
(`on-chain-client/docs/architecture/2026-10-06-chain-write.md`);
here each now states that org-node holds none of it.

## SDD-rq6nv4 — Threshold-1 multisig account derivation

`org-node/src/chain_write/multisig.rs` (`multi_account_id`,
`build_dispatch_tx`)

**SDD-rq6nv4**: the absence of the multisig from org-node: org-node derives no
pseudo-account, holds no multisig signatory key and dispatches nothing. The
derivation of the account that controls an Organisation's slot, and the choice
between dispatching directly and through it, are on-chain-client's chain writer
(`on-chain-client/docs/architecture/2026-10-06-chain-write.md`).
traces: REQ-nhe2zu, REQ-txvtm9

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

`org-node/src/service.rs` (`ChainOps`, `MockChainOps`, `MockChainInner`,
`ChainOpsReader`)

**SDD-ueh4tm**: the boundary between the five user stories and the chain,
expressed as a trait rather than a client. It exists so the stories can be
exercised in full without a chain, and it is the mechanism that makes
SDD-rx2yvy, SDD-8cpyfa and SDD-72ddm6 verifiable items instead of parts of the
I/O shell.
traces: REQ-nhe2zu, REQ-txvtm9, REQ-bvh8v6

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The seam is now read-only: `ChainOps` keeps `read_state` and loses
`submit_genesis` and `submit_update`, because org-node does not write to the
chain — the app submits through on-chain-client and then asks org-node to
commit. `MockChainOps` keeps a stand-in for that write, `apply_genesis` and
`apply_update`, inherent methods a test calls in the app's place, so a story
can still run from provisional update to commit without a chain.

**LLR-65py3d**: `ChainOps` presents reading an Organisation's state as an
asynchronous operation on a trait object, and presents no operation that
writes to the chain, so a substitute may be injected without changing the
service and the service cannot submit an update through it.
satisfies: derived

**LLR-hg3xzf**: clones of `MockChainOps` share one chain state, so two services
under test observe the same chain as each other.
satisfies: derived

**LLR-ryzr8m**: `MockChainOps::apply_update(org_id, root, key, expected_epoch)`
refuses an update whose expected epoch is not its slot's current epoch, with a
chain error and the slot unchanged, and an update at the current epoch
advances the slot by one; `apply_genesis(root, key)` creates a slot at epoch
one and returns its org id. This is the contract's compare-and-swap as the
mock imitates it, so a story exercised against the mock cannot commit an
update the chain would refuse.
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-65py3d said `ChainOps` presents "submitting an update" too, and LLR-ryzr8m
stated the compare-and-swap on `MockChainOps`'s `submit_update`. The write
leaves the trait; the mock's imitation of it stays, as inherent methods a test
calls where the app would submit through on-chain-client.

*Added 2026-10-05 by review round 8, which measured that disabling the check left the gate green: every story submits at
the right epoch. `the_mock_chain_refuses_an_update_at_the_wrong_epoch` carries
it.*

## SDD-89es4z — Persona and Organisation genesis

`org-node/src/service.rs` (`create_persona`, `create_organisation`,
`commit_genesis` *(added 2026-10-05, see below)*)

**SDD-89es4z**: how a node acquires an identity and how an Organisation comes
into existence — two keypairs drawn once, a genesis provisional update, and
the Organisation record created only once the chain carries its root.
traces: REQ-hzm4kt, REQ-nhe2zu, REQ-txvtm9, REQ-d9g6nt, REQ-qn2erx, REQ-ech45n, REQ-xs4ab8, REQ-tqap3r, REQ-yp75u9, REQ-stx9v3

*Amended 2026-10-06 (owner ruling on rotation, change
`worktree-org-node-org-key-pair`):* traces REQ-stx9v3. Genesis draws the
first Organisation key pair after its record's root is calculated, as every
provisional update now does (LLR-s6qnht, LLR-sj7cd5 amended).

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`, independent
review round 1, finding-1):* traces REQ-yp75u9, for `create_organisation`'s
refusal of a Persona already bound (LLR-6z5xya, in
`2026-10-06-chain-authority.md`).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said "a genesis trie, and the first on-chain slot": `create_organisation`
submitted the genesis to the chain and created the record. The owner ruled
that org-node does not write to the chain and that what it builds is
provisional until the chain agrees. `create_organisation` now stores a genesis
provisional update and nothing else (LLR-s6qnht); the app runs the genesis
ceremony through on-chain-client and calls `commit_genesis` with the org id
and proxy account it returned, which verifies the update against the chain and
creates the record (LLR-wzqqg9). Both requirements are in
`2026-10-06-chain-authority.md`, so the item
traces REQ-xs4ab8 and REQ-tqap3r too.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
also constrained by LLR-g76zqd, under SDD-af5vnt in
`2026-10-04-type-safety.md`. `create_persona` takes the handle, name and
surname as a parsed `Handle`, `Name` and `Surname`, so a Persona record is
built only from parsed values. So the item traces REQ-qn2erx too.)*

*(Re-traced 2026-10-05 by review round 3, finding-8 and finding-9, change
`worktree-person-shared-types`: `create_organisation` draws the Organisation
private key, publishes its X25519 public key with the genesis root and
refuses one equal to a genesis key, so the item traces REQ-ech45n.
LLR-sj7cd5, in `2026-10-05-unsigned-envelope.md`, refines it.)*

**LLR-tev8h8**: `create_persona` draws an independent member keypair and device
keypair from the caller's random source and persists their seeds rather than
the keys.
satisfies: derived

**LLR-s7yu4k**: a Persona's identifier is the first sixteen bytes of its
Member-as-a-group key, the X25519 public key of its member seed, written as
thirty-two lowercase hexadecimal digits. Two Personas share an identifier only
if their member keys share those bytes.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item derived the identifier from the member verifying key.

**LLR-v82xds**: a newly created persona is recorded as Proposed, with no
Organisation and no member identifier.
satisfies: derived

**LLR-68yd3j**: `create_organisation` writes the store before it returns and
creates no Organisation record: the record list and every Persona's status and
binding are as they were, and the only addition is the genesis provisional
update.
satisfies: REQ-xs4ab8

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said `create_organisation` persists the Organisation record and marks the
Persona Active, and was derived. Creation now builds a provisional update
without changing any record (REQ-xs4ab8); the record is created, and the
Persona marked Active, by `commit_genesis` (LLR-wzqqg9, LLR-w3fhhg).

**LLR-rjg3m2**: the founding Member's `MemberId` is drawn from the caller's
random source before the genesis provisional update's Membership record is
built and is never computed from a key, so the same persona building two
genesis provisional updates receives two different identifiers.
satisfies: REQ-d9g6nt

*Amended 2026-10-06 (REQ-yp75u9, independent review round 1 of change
`worktree-org-node-chain-authority`).* This read "the same persona founding two
Organisations". A Persona is now bound to at most one Organisation, so it can
found only one (LLR-6z5xya, LLR-eyc4ud); while unbound it can still build
several genesis provisional updates, and those are where the same keys meet
two drawn identifiers.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
"The founding administrator" reads "the founding Member", as REQ-d9g6nt does
after the same ruling: org-node has no concept of an administrator. The
identifier is drawn when the provisional update is built.

**LLR-w3fhhg**: `commit_genesis` marks **the named Persona** Active and binds
it to the Organisation it commits, leaving every other Persona in the store as
it was. A Persona already bound to an Organisation is refused before this
step and never rebound (LLR-eyc4ud).
satisfies: derived

*Amended 2026-10-06 (REQ-yp75u9, independent review round 1 of change
`worktree-org-node-chain-authority`).* The second and third sentences read
"Where that Persona was already bound to another Organisation, the binding is
overwritten. That is PR-mdv38y's second writer, and it is stated here and not
endorsed." PR-mdv38y is resolved: `commit_genesis` refuses such a Persona.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This named `create_organisation`, which no longer changes a Persona; the
binding moves with the record's creation to `commit_genesis`.

*Added 2026-10-04 by review round 3 with LLR-vdyu65, and for the same
measurement: `update_persona_status` could mark whichever Persona came first
in the store, with the gate green, because no gated test had two.*

**LLR-dzte8x**: `commit_genesis` stores in the new Organisation record, as
opaque data, the proxy account its caller hands over, and the account reaches
the disk with the record; org-node reads it only to hand it back
(`OrgService::proxy_account`, LLR-3v5nu9) and never interprets it.
satisfies: derived

**LLR-q3aj8z**: `commit_genesis` leaves the founding Persona's member id as it
was, which is none: only an unbound Persona commits a genesis (LLR-eyc4ud), and
the founding Member's `MemberId` is held only in the record's member snapshots.
satisfies: derived

*Amended 2026-10-06 (REQ-yp75u9, independent review round 1 of change
`worktree-org-node-chain-authority`).* The last two sentences described
PR-mdv38y's second-writer path, where the Persona kept the member id of the
Organisation it was bound to before. That path is closed: `commit_genesis`
refuses a bound Persona.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
Both named `create_organisation`, and LLR-dzte8x said the proxy account was
what "the chain returned at genesis", kept "so the production chain client can
find the proxy after a restart". org-node no longer submits to the chain: the
app runs the genesis ceremony through on-chain-client, which returns the proxy
account, and hands it to `commit_genesis`; the app reads it back to submit
each later update. org-node holds it opaquely and never uses it (owner ruling
6). Both requirements move with the record's creation to `commit_genesis`.

*Added 2026-10-05 by review round 8, which measured that not storing the proxy account and setting the member id were
both green. `MockChainOps` returns no proxy account, so no test could see the
first. LLR-dzte8x is carried by
`the_proxy_account_from_genesis_is_kept_and_handed_back` and
`commit_genesis_creates_the_record_once_the_chain_carries_the_root`, which
hand `commit_genesis` a proxy account. LLR-q3aj8z is carried by
`commit_genesis_creates_the_record_once_the_chain_carries_the_root` and,
for the second-writer path, by
`pr_mdv38y_founding_an_organisation_rebinds_a_member_persona`.* *(Test names
updated 2026-10-06, change `worktree-org-node-chain-authority`: the first was
`…_kept_and_passed_on_every_update`, and
LLR-q3aj8z was carried by a `create_organisation` test that now asserts only the
genesis update.)*

*Annotated 2026-10-06 (independent review round 2, finding-3).* The
second-writer path the note above names is closed: R1a of change
`worktree-org-node-chain-authority` rewrote
`pr_mdv38y_founding_an_organisation_rebinds_a_member_persona` as
`pr_mdv38y_a_member_persona_cannot_found_another_organisation`, which verifies
that a bound Persona cannot found another Organisation (LLR-6z5xya,
REQ-yp75u9), not LLR-q3aj8z. LLR-q3aj8z is now carried by
`commit_genesis_creates_the_record_once_the_chain_carries_the_root` alone.

## SDD-rx2yvy — Admission

`org-node/src/service.rs` (`admit_member`, `export_invite`, `import_invite`,
`export_join_request`, `import_join_request`; since 2026-10-05 `admit_member`,
`Joiner`, `expect_admission`, `persona_public_keys`, `proxy_account`,
`send_update` and `OutgoingUpdate`, see below)

**SDD-rx2yvy**: the two sides of letting someone in, as org-node sees them —
on the inviting node, minting the new Member's leaf into an admission
provisional update and, once that update has committed, sending the joiner the
committed Envelope and everything they need to check it; on the joining node,
recording that the app expects that admission.
traces: REQ-xa6smf, REQ-ztdza4, REQ-nhe2zu, REQ-txvtm9, REQ-d9g6nt, REQ-qn2erx, REQ-xs4ab8, REQ-tqap3r, REQ-8amu2a, REQ-szq3ud, REQ-3dsweu, REQ-stx9v3

*Amended 2026-10-06 (owner ruling on rotation, change
`worktree-org-node-org-key-pair`):* traces REQ-stx9v3. An admission
provisional update, built through `keep_change_set`, draws a fresh
Organisation key pair once its root is calculated, keeps the private key and
publishes the public key (LLR-e2b7gv; LLR-ghja3x amended).

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
"Everything they need to check it" now includes the Organisation private key
of the epoch the update reaches, as the node's record holds it, which
`send_update` puts in every Organisation-information message without taking
it from its caller; and `send_update` chooses the kind of Wire message by
whether the node's committed record lists the recipient Device (LLR-6ymd6d,
in `2026-10-07-org-key-pair.md`; LLR-2xzys9,
LLR-48jakr, LLR-bg3vsw, LLR-jn5jeh and LLR-9zfnmb amended). On the joining
node the expectation names the Organisation alone. The item traces
REQ-szq3ud and REQ-3dsweu too.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item handed the new member the signed change. LLR-8bum44, amended in place,
still constrains it.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This was "the administrator's side of letting someone in — minting the new
member's leaf, moving the on-chain root forward, and handing the new member the
signed change". The owner ruled that org-node has no administrator, does not
write to the chain, builds provisional updates, and that the invitation
exchange belongs to the app. `admit_member` now builds and stores a provisional
update and nothing more; the app submits it through on-chain-client and calls
`commit_update` (SDD-8cpyfa); sending is `send_update`, a separate operation
that a commit never waits on. `export_invite`, `import_invite`,
`export_join_request` and `import_join_request` are removed; the app reads a
Persona's public keys through `persona_public_keys`, and declares an expected
admission through `expect_admission`. The item traces REQ-xs4ab8, REQ-tqap3r
and REQ-8amu2a for these, and LLR-2xzys9 in
`2026-10-06-chain-authority.md` sits under it.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
also constrained by LLR-8bum44, under SDD-af5vnt in
`2026-10-04-type-safety.md`. `import_join_request` refuses a Join request
holding a value its type's parse refuses, with `InvalidField` naming the
field, and `import_invite` stores no pending Invite whose keys are not curve
points. So the item traces REQ-qn2erx too.)*

**LLR-rb8r65**: `admit_member(rng, org_id, &Joiner) -> Result<ProvisionalUpdate,
OrgNodeError>` adds the joiner to the trie rebuilt from the record and stores,
through `insert_provisional`, an admission provisional update; it calls no
chain operation, binds no endpoint, sends nothing and leaves the record
unchanged, so a second admission before the first commits is built on the same
record.
satisfies: REQ-xs4ab8

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the signed change.

**LLR-ghja3x**: an admission or revocation provisional update carries the
record's root as its base root, as its Sequence number the epoch the update
produces on the chain — the record's epoch plus one (REQ-txvtm9) — as the
value to publish the public key of the fresh Organisation key pair drawn for
it once that root is calculated (LLR-e2b7gv), never the record's current
Organisation public key, the postcard-encoded Change set with that pair's
private key, and the root that Change set produces; nothing in it is signed.
satisfies: REQ-xs4ab8, REQ-stx9v3

*Amended 2026-10-06 (owner ruling on rotation, change
`worktree-org-node-org-key-pair`).* This said the update carries "the
record's Organisation public key as the value to publish". The owner ruled
that every provisional update draws a fresh Organisation key pair when its
resulting root is calculated and publishes its public key (REQ-stx9v3), so
the key changes with every update.

*Amended 2026-10-06 (merge of master `5f7c177` into change
`worktree-org-node-chain-authority`).* This said "one greater than the
record's mark". That equals the record's epoch plus one only while the mark
equals the epoch, which master's `create_organisation` broke (it wrote mark 0
at epoch 1). The contract moves a record at epoch E to E + 1, so the item now
states that; `commit_genesis` writes mark 1 at epoch 1 (LLR-wzqqg9), after
which the two readings agree on every record.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the Envelope is signed by the administrator's Member-as-a-group key
and carries the record's last sequence number plus one. Nothing signs, and the
Sequence number is the epoch the update produced (REQ-txvtm9).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-rb8r65 said `admit_member` submits the new root on chain at the next epoch
and only then sends the signed change; LLR-ghja3x that the Envelope sent is
signed by the administrator's Member key. Both were derived. org-node no longer
writes to the chain or signs, and admission builds a provisional update
(REQ-xs4ab8). The anchor-before-notice order LLR-rb8r65 protected now holds
by construction: nothing can be sent before `commit_update` has verified the
update against the chain.

**LLR-bg3vsw**: the outgoing update `commit_update` returns, and so every
Organisation-information Wire message `send_update` sends, carries the member
snapshots as they were **before** the committed update, so a joiner holding
no record of the Organisation can rebuild the trie the change applies to; a
revocation carries no snapshot.
satisfies: REQ-xa6smf

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This said "the Wire message `send_update` sends", of a message of one kind.
A revocation goes only to a Device the committed record no longer lists, which
never rebuilds from a snapshot, and carries none (LLR-js9dsu, LLR-6ymd6d).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said "the Wire message carries", of the message `admit_member` sent. The
send is now `send_update`, of the `OutgoingUpdate` the commit returned; the
property is unchanged.

**LLR-jn5jeh**: in Loopback mode `send_update` dials the recipient at the full
`EndpointAddr` its caller passes, not by endpoint identity alone. Nothing
checks that the address names the recipient's DevicePublicKey, so an
Organisation-information message and the Organisation private key it carries
go wherever the address points. That is PR-2dmjzj's
defect, stated here and not endorsed. One of its cures (dial by key in both
modes) would replace this requirement.
satisfies: REQ-ztdza4

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This said "an admission and its Organisation secret". The secret is gone, and
every Organisation-information message, for any operation, carries the
Organisation private key (REQ-szq3ud), which the wrong address now receives.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This named `admit_member`, which no longer sends; the dial moves with the send
to `send_update`, unchanged.

*Review round 7 measured that adding the missing check reddens sixteen tests,
including `five_stories_full_e2e` and the PR-mdv38y and PR-xwek5e pins. The
fix change should expect those fixtures to depend on the defect.*

*Reworded 2026-10-04 by review round 2. The clause said "the full address
carried in the join request", which is not what the code does — `admit_member`
dials the `peer_addr` argument, and the caller is what decodes that from the
join request's `node_addr`. The two tests credited with this requirement were
also both wrong for it: neither calls `admit_member` at all, and one is the
dial-by-id case the Loopback clause excludes. It is now carried by
`in_loopback_mode_the_joiner_is_dialled_at_the_full_address`, which calls
`admit_member` and sees the admission arrive at the endpoint the address it
carried names. The negative — dialling the same identity with its transports
stripped — is deliberately **not** asserted there: iroh retries a live identity
past any timeout a gated test can afford (measured at over thirty seconds), so
"it does not arrive" is not observable at this gate and the falsification is
the mutation instead. The verification record sets that out in full.*

**LLR-t4znbk**: `send_update` writes nothing to the store and changes no
record: the Organisation record has already been updated by the commit the
outgoing update came from, and a send that fails returns a typed error and
leaves that committed record in place.
satisfies: REQ-tqap3r

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said the record is updated and the store saved only after the Wire
message has been sent — the publish-before-persist order PR-vt244s books as a
defect — and was derived. The owner moved the chain write out of org-node and
ruled that the sender commits its own update only once it verifies against the
chain (REQ-tqap3r), and the risk assessment of that requirement made it a
constraint that the commit must not wait on the send. The record is now
written by the commit (LLR-4tcxsu) and never by the send, so a failed send no
longer leaves the record an epoch behind the chain. PR-vt244s stays open by
owner ruling.

**LLR-ag9mgm**: `admit_member` draws the new member's `MemberId` from the
caller's random source before the leaf is built and never computes it from a
key, so admitting the same Member-as-a-group key and device key twice yields two
different identifiers and a deleted identifier is never reissued.
satisfies: REQ-d9g6nt

**LLR-vdyu65**: `admit_member` builds on the Organisation record the `org_id`
argument names and stores the provisional update under **that** Organisation,
so a device holding more than one Organisation admits into the one the caller
asked for and leaves every other record and every other Organisation's
provisional updates where they were; no Persona is looked up by an
administrator key.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item also said `admit_member` signs with that Organisation's administrator
Persona. Nothing is signed; the Persona it looks up names the endpoint to bind
(LLR-cns6q6, PR-8qsnhx).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said `admit_member` "signs with the administrator Persona of that
Organisation" and leaves "every other on-chain root" where it was. Nothing is
signed, org-node has no administrator (`admin_persona_for_org` is removed),
and org-node writes no root to the chain; what the clause still protects is
that the right record and the right Organisation's provisional updates are
touched.

*Added 2026-10-04 by review round 3, which found four record lookups —
`find_org`, `find_org_mut`, `admin_persona_for_org` and
`update_persona_status` — able to ignore the identifier they were given and
return whatever record came first in the store, with the whole gate green. Every
gated test until now gave its service one Organisation and one Persona. This is
round 2's finding-3 one level over: that finding convicted `ensure_endpoint`
for the same assumption, and the fix wrote a two-persona test for that one call
site rather than sweeping the lookups. The failure it allows is not small — an
administrator holding two Organisations calls `admit_member(org_2, …)`, and the
joiner is minted into org_1's record, org_1's root is submitted on chain for
the epoch, and the envelope is signed by org_1's administrator.*

**LLR-3v5nu9**: `OrgService::proxy_account(org_id)` returns the proxy account
the named Organisation's record holds, unchanged — `Some` as `commit_genesis`
stored it, `None` on a record created by a first admission — and no operation
of org-node passes it to the chain or reads anything else from it.
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said `admit_member` submits the update with the proxy account its record
holds. The app now submits through on-chain-client, so the account goes to the
app: org-node keeps it as opaque data and hands it back (owner ruling 6).

*Added 2026-10-05 by review round 8, which measured that passing none was green. It is carried by
`the_proxy_account_from_genesis_is_kept_and_handed_back` (renamed 2026-10-06
from `…_kept_and_passed_on_every_update`).*

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
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item ended ", so the joiner has a key to pin the first admission's sender
against". Nothing on the joiner's side compares that key with a sender since
the owner's ruling.

**LLR-qezw3n**: no value org-node returns carries a dialling address for
another device to reach it by: the Invite's dialling address, which nothing
read, leaves with the Invite, and the Join request's with the Join request.
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-zj88e6 said `export_invite` carries the administrator Persona's device key
so the joiner can pin the first admission's sender, and LLR-qezw3n that it
carries the bound endpoint's address (PR-8qsnhx's reading side). The owner
ruled that the invitation exchange belongs to the app, that nothing about a
sender is checked, and that the Invite's dialling address leaves org-node. Both
now state the absence, compile-enforced. PR-8qsnhx's reading side goes with
them; its sending side remains (LLR-6zjzn2).

*Added 2026-10-05 by review round 8, which measured that emptying the address
left the gate green. Both requirements now state an absence and are carried by
`org_node_holds_no_invitation_exchange` (`org-node/tests/absences.rs`). The
tests that carried the earlier statements — one round-tripping the out-of-band
blobs, and review round 9's PR-8qsnhx pin on the Invite — were deleted
2026-10-06 with the blobs (change `worktree-org-node-chain-authority`, T7).*

**LLR-9zfnmb**: `expect_admission(rng, org_id)` records the Organisation in
`expected_admissions` at most once — a repeated declaration for the same
Organisation leaves one entry rather than appending, and another Organisation
is a second entry — and the store is written before it returns, so the
refusal LLR-s8xp7m performs has one answer and survives a restart.
satisfies: REQ-8amu2a

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This said `expect_admission(rng, org_id, invite_id)` recorded the pair of
Organisation and invite identifier. The invite identifier no longer travels
between peers (REQ-8amu2a as amended again), so the expectation is the
Organisation alone, as before the re-trace note below.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item ended ", so the cross-check LLR-j83kc8 performs has one answer and
survives a restart". The Invite is no longer a cross-check.

*Parent stale, noted 2026-10-05 by review round 4 (finding-5).* Since the
amendment of 2026-10-05, REQ-xa6smf requires no Invite, so it does not imply
this item, and `satisfies:` above names it only until the item is re-traced.
By owner ruling of 2026-10-05 the re-trace is left to chain-authority's
change 1.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said `import_invite` records at most one pending Invite per Organisation,
for the sender cross-check of LLR-j83kc8, and satisfied REQ-xa6smf. Invites
leave org-node and the sender is not checked; what the joining node keeps
instead is the app's declaration that it expects to join that Organisation
(REQ-8amu2a, RC-2ferct), with the same at-most-once and written-before-return
properties.

*Re-traced and amended 2026-10-06 (change `worktree-org-node-chain-authority`,
after the merge of master `5f7c177`).* The stale-parent note above is
answered: the item already names REQ-8amu2a, the requirement it refines, and
no longer REQ-xa6smf. By the owner's ruling on pre-emption (2026-10-06,
REQ-8amu2a as amended) an expectation names the invite identifier of the
Invite the user replied to as well as its Organisation; this said
`expect_admission(rng, org_id)` recorded the Organisation alone.

**LLR-437fvx**: `persona_public_keys(persona_id)` returns the Persona's
Member-as-a-group key and its DevicePublicKey as two distinct keys, each derived
from that Persona's own seed, and no dialling address.
satisfies: derived

**LLR-836z24**: org-node has no `import_join_request` and decodes no Join
request, so nothing the joiner composes reaches the inviter's store except
through `admit_member`'s parsed `Joiner`.
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-437fvx said `export_join_request` carries the Persona's two keys and the
bound endpoint's address (PR-8qsnhx's reading side); LLR-836z24 that
`import_join_request` stores nothing. The Join request leaves org-node for the
app's Invite reply. The keys the reply needs come from `persona_public_keys`,
which carries no address; LLR-836z24 states the absence, compile-enforced as
it was before.

## SDD-8cpyfa — The receive-and-commit path

`org-node/src/service.rs` (`receive_and_verify`, `ReceiveOutcome`,
`first_admission_base` *(added 2026-10-05, see below)*)

**SDD-8cpyfa**: the commit path — accept one Wire message, or take one of the
node's own provisional updates, run every check that needs no chain, read the
chain once, verify against it, and commit, discarding the provisional updates
the commit orphans. A first admission is attempted only for an Organisation the
app declared it expects. Nothing about the sender is checked.
traces: REQ-xa6smf, REQ-ztdza4, REQ-nhe2zu, REQ-txvtm9, REQ-bvh8v6, REQ-d9g6nt, REQ-qn2erx, REQ-f2k4tr, REQ-8amu2a, REQ-tqap3r, REQ-uv3v5w, REQ-uxv2x2, REQ-kt877x, REQ-yp75u9, REQ-c29s93, REQ-bwx7eg, REQ-ju6vn2, REQ-3dsweu, REQ-vxqc5g, REQ-jy6ybw

*Amended 2026-10-06 (owner ruling on rotation, change
`worktree-org-node-org-key-pair`):* traces REQ-jy6ybw. `commit_update`
selects the node's provisional update by its root and its fresh Organisation
public key, and on commit replaces the record's Organisation private key and
public key with the update's (LLR-6s785x; LLR-cmdrp9 amended); a committed
Organisation-information message sets the record's public key to the chain's
(LLR-4kh9w9).

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The path now receives two kinds of Wire message. An Organisation-information
message that does not decode is refused with a typed error before anything
else (LLR-xn5pwc); one that verifies is refused when the X25519 public half
of the Organisation private key it carries is not the `org_pub_key` read from
the chain at verification (LLR-ba2ejp), and on commit its key is stored
(LLR-ckk5nz amended). A revocation about an Organisation the node holds no
record of is refused before the chain is read (LLR-38e2kn); one about a held
Organisation is verified and then refused with `RevocationNotForThisDevice`
unless it removes this node's own Device, in which case the record is deleted
(LLR-pt32fx, owner ruling on relabelling), so no revocation commits into a
record the node keeps. A first admission is matched by Organisation alone
(LLR-s8xp7m amended). LLR-xn5pwc, LLR-ba2ejp, LLR-38e2kn and LLR-pt32fx are
in
`2026-10-07-org-key-pair.md`. The item traces
REQ-c29s93, REQ-bwx7eg, REQ-ju6vn2, REQ-3dsweu and REQ-vxqc5g too.

*Amended 2026-10-06 (change `worktree-org-node-chain-authority`, independent
review round 1, finding-1):* traces REQ-yp75u9, for the rule that no commit
path rebinds a Persona bound to another Organisation (LLR-eyc4ud, in
`2026-10-06-chain-authority.md`).

*Amended 2026-10-06 (owner ruling on pre-emption, change
`worktree-org-node-chain-authority`):* traces REQ-kt877x, for the own-Persona
rule on a first admission (LLR-3f5h7b).

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item owned the two sender cross-checks, the only place the transport's
authenticated identity was compared with the membership record. Both are
removed (REQ-xa6smf, REQ-ztdza4). LLR-rys5nx, in
`2026-10-05-unsigned-envelope.md`, also refines this item.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This was "the member's side — accept one Wire message, establish who sent it,
verify it against the chain, and commit", and owned "the two sender
cross-checks". The owner ruled that nothing about an Envelope's sender is
checked, that a node commits its own provisional update only once it verifies
by the checks a received update passes, and that a commit discards the old
record and the provisional updates it orphans. The item now also owns
`commit_update` and the commit step the three commit paths share; its seven
new requirements — LLR-9ew26y, LLR-s8xp7m, LLR-cmdrp9, LLR-ewkg85, LLR-4tcxsu,
LLR-mkj4bz and LLR-b27jr6 — are in
`2026-10-06-chain-authority.md`, so it traces
REQ-f2k4tr, REQ-8amu2a, REQ-tqap3r, REQ-uv3v5w and REQ-uxv2x2 too.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
`first_admission_base`, which only `receive_and_verify` calls and which no
item named, is this item's. It is also constrained by LLR-8bum44, under
SDD-af5vnt in `2026-10-04-type-safety.md`. A record snapshot holding a value
its type's parse refuses fails as a whole with `InvalidField` naming the
`member.…` field, and extends nothing. So the item traces REQ-qn2erx too.)*

**LLR-j6j95z**: a first admission to an Organisation the node holds no record
of is attempted only from an Organisation-information Wire message, whose
record snapshot is required, so it always rebuilds from the snapshot of the
Membership record the change extends (`first_admission_base(&[u8])`) and never
from a reconstructed one; a body of that kind without a snapshot does not
decode (LLR-js9dsu), and a revocation, which carries none, is refused
(LLR-38e2kn).
satisfies: REQ-d9g6nt

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This said a first admission is refused "when the Wire message carries no
snapshot", because the snapshot was optional. The Organisation-information
kind requires it, so the absence is a decode refusal or a revocation's
refusal, and `first_admission_base` takes the snapshot bytes, not an
`Option`.

**LLR-j83kc8**: a first admission to an expected Organisation that verifies
against the chain and lists one of the node's own Personas is committed
whichever DevicePublicKey the connection authenticated: no Invite and no
sender key is consulted.
satisfies: REQ-xa6smf

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item refused a first admission whose sender was not the Invite's administrator
device.

*Amended 2026-10-06 (owner ruling on pre-emption, change
`worktree-org-node-chain-authority`).* "and lists one of the node's own
Personas" is added: REQ-kt877x refuses a verified first admission that lists
none (LLR-3f5h7b), and this item states only that the sender is not what
decides.

**LLR-u6rq4s**: on a Wire message about an Organisation already held, the
Device key the connection authenticated is compared with no Membership record:
an Organisation-information update that verifies against the chain is
committed whether that key is in the record before the update, after it, or in
neither, and a revocation is accepted or refused (LLR-pt32fx) by the receiving
node's own Device, never by the sender's.
satisfies: REQ-ztdza4

*Amended 2026-10-06 (owner ruling on relabelling, change
`worktree-org-node-org-key-pair`).* This said "an update that verifies
against the chain is committed". A revocation that leaves the receiving
Device listed is now refused (LLR-pt32fx); what the item protects, that the
sender's key decides nothing, is unchanged.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item required that key to be in the trie the Envelope verified into, checked
after verification.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-j83kc8 required a first admission's sender to equal the administrator's
DevicePublicKey a pending Invite named; LLR-u6rq4s required an update's sender to
be in the verified trie. The owner ruled that nothing about the sender is
checked (REQ-xa6smf and REQ-ztdza4 as amended): authority is the chain's, and a
chain-valid update relayed by any peer is harmless. Both are rewritten to that
rule. The two `BadSignature` refusals they described are removed.

**LLR-37cj3n**: which Change set `receive_and_verify` commits, and the
record it produces, are decided by the Organisation state it reads from the
chain itself and by its own record, never by a value carried in the Wire
message other than its Envelope, its Organisation private key (which becomes
the record's key, and only when its public half is the chain's Organisation
public key, LLR-ba2ejp, REQ-ju6vn2) and, on a first admission, its snapshot,
and never by the device that delivered it. The Organisation state's
Membership root and epoch decide the Change set; its Organisation public key
decides only whether an Organisation-information message commits at all,
through the check of the carried Organisation private key against it
(LLR-ba2ejp).
satisfies: REQ-nhe2zu

*Amended 2026-10-07 (change `worktree-org-node-org-key-pair`, review round 1,
finding-3).* This named the Envelope and, on a first admission, the snapshot
as the only carried values that decide the record. Since this change the
record's Organisation private key is the carried key, constrained by the
chain's public key, so the carried key is named as the third input.

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This said a value carried in the Wire message never decides what is
committed, and that the Organisation state contributes only its root and
epoch, "so a state holding any Organisation public key commits the same
Change set". The receipt check RC-9cefcn makes the chain's public key, and
the private key the message carries, decide whether an Organisation-information
message commits; neither changes which Change set does.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item named the author key the signature was verified under. There is no author
key.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said the signature is verified under the Published signing key read from
the chain. No signature is verified now; what remains true and testable is
that the key field is not an input to the decision.

**LLR-3wb7th**: an Organisation the chain holds no state for is refused with
`OrgNotOnChain`, record unchanged — for an Organisation held, only after the
chain-free checks have passed (LLR-9ew26y), and for a first admission only
after the expectation check, the snapshot decode and the chain-free checks
(LLR-s8xp7m).
satisfies: REQ-bvh8v6

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said the refusal comes "before any local record is consulted", because the
Receive operation read the chain first to obtain the signing key. RC-mj6gjq and
RC-2ferct require the chain-free checks, which consult the local record, before
any chain read, so the order is reversed.

**LLR-cja9zv**: the record, the epoch, the sequence mark and the member
snapshots are written together after verification succeeds, and the store is
saved before the outcome is returned.
satisfies: REQ-nhe2zu, REQ-txvtm9

*Annotated 2026-10-05 by review round 8.* This holds for a change that
removes this node's own device key too: on this path that change is committed
as an update, with nothing deleted. That is PR-322qst's defect, stated here
and not endorsed, and the PR-322qst pin carried this requirement.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The annotation above no longer holds: a change that removes this node's own
DevicePublicKey is not committed as an update on this path. Owner ruling 7 (an
empty record on removal, REQ-uxv2x2) applies it on every commit path, so the
record is deleted and the Personas revoked (LLR-b27jr6). The pin for PR-322qst
asserted the opposite; it reddened when this was implemented and was rewritten
2026-10-06 as `an_own_revocation_on_the_ordinary_path_deletes_the_record`,
which carries this requirement now.

**LLR-mbjfq8**: a first admission to an Organisation the app declared it
expects is accepted on the chain anchor, the check of the Organisation private
key it carries against the chain (LLR-ba2ejp) and the own-Persona rule
(LLR-3f5h7b) alone: no Invite, no invite identifier, no signature and no
sender key is consulted.
satisfies: REQ-xa6smf, REQ-8amu2a

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This said "under the invite identifier its Wire message carries". The invite
identifier no longer travels between peers, and the receipt check (RC-9cefcn)
is added to what the admission rests on.

*Amended 2026-10-06 (owner ruling on pre-emption, change
`worktree-org-node-chain-authority`).* This said "on the chain anchor alone"
of an Organisation the app expects. The expectation now names an invite
identifier too (REQ-8amu2a as amended), and a verified admission that lists
none of the node's Personas is refused (REQ-kt877x).

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said such an admission rests on the chain anchor and the signature alone,
and that the Invite cross-check applies when an Invite exists. There is no
signature and no cross-check.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said a first admission with no imported Invite is accepted "on the chain
anchor and the signature alone", and was derived. The signature and the
Invite are gone; what now separates a first admission the node takes from one
it refuses is the app's declared expectation (REQ-8amu2a, LLR-s8xp7m).

*Added 2026-10-04 by review round 7, which measured that refusing such an
admission left the gate green. It is stated as the code behaves, and
`a_first_admission_to_an_expected_organisation_rests_on_the_chain_alone` pins it
(renamed 2026-10-06 from `…_with_no_imported_invite_rests_on_the_chain_alone`).
The trade-off is assessed under RC-b6mydy in the hazard analysis and in the
derived-requirements risk file.*

*Abnormal case added 2026-10-05 by review round 4 (finding-4).* "On the chain
anchor alone" cuts both ways: such an admission that does not reach the
chain's Membership root is refused with `RootMismatch` and commits nothing,
marks no Persona and writes nothing to disk.
`a_first_admission_that_misses_the_chain_root_commits_nothing` pins it (renamed
2026-10-06, without "with no imported invite").

**LLR-y2v8v2**: every record this path reads or writes is the one the received
change's Organisation identifier names — the local Membership record it
verifies against, the expectation it checks and clears, the record it commits
into and the provisional updates it discards — so a node holding more than one
Organisation leaves every other Organisation's record, Organisation private
key, expectation and provisional updates exactly as they were.
satisfies: REQ-nhe2zu, REQ-txvtm9, REQ-xa6smf

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
"secret" read the Organisation secret, which is removed; the record's
Organisation private key takes its place.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item also listed "the pending invite it cross-checks the sender against". The
pending Invite is no longer compared with the sender; the one consumed is
still the named Organisation's.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This named "the pending invite it cross-checks the sender against" and "the
invite it consumes". Invites leave org-node and the sender is not checked; the
records of the same kind this path now touches are the expected admission
(LLR-s8xp7m, LLR-q8emds) and the provisional updates a commit discards
(LLR-mkj4bz).

*Narrowed 2026-10-04 by review round 5.* This requirement also claimed "the
Persona it marks Active", and that clause was false. The Persona is chosen
from every Persona whose device key is in the verified trie, whatever
Organisation it is bound to, and is then rebound. A device holding two
Organisations can have one Organisation's Persona moved to the other. That is
PR-mdv38y, pinned by
`pr_mdv38y_the_receive_path_rebinds_another_organisations_persona`. The clause
is withdrawn here rather than left standing over a test that asserts its
opposite. LLR-e5c9ud states what the selection does today.
*Annotated 2026-10-06 (REQ-yp75u9).* PR-mdv38y is resolved: the selection is
scoped by LLR-eyc4ud, and the pin is rewritten as
`pr_mdv38y_an_update_enrolling_another_organisations_persona_does_not_rebind_it`.

**LLR-xq9nrq**: on first admission the new Organisation record holds, as its
Organisation public key, the value read from the chain, never a value carried
in the Wire message, and holds no administrator key: `OrgRecord` has no
`admin_member_key`.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the field holds the Published signing key, which was the
administrator's Member-as-a-group key. The chain's key is now the Organisation
public key, an X25519 key distinct from every member key (REQ-ech45n). A first
admission no longer needs an Invite (REQ-xa6smf); by the owner's answer Q1 the
field takes the imported Invite's administrator key when there is one, and the
chain's key only when there is none. The field stays, as master left it, until
chain-authority's change 1 removes every administrator field.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said the record holds the Published signing key read from the chain "as
its administrator's Member-as-a-group key". The owner ruled that there is no
`admin_member_key` anywhere in org-node; the value read from the chain is kept
in the record's `org_pub_key`, where it already was.

**LLR-ckk5nz**: when `receive_and_verify` or
`receive_and_self_delete_if_revoked` commits an Organisation-information Wire
message that has passed the key check (LLR-ba2ejp), the record's
`org_private_key` is the Organisation private key the message carries: the
first admission creates the record with it, and an update replaces the stored
key with it, in the same store save as the rest of the commit. A revocation
never commits into a record the node keeps: it is refused unless it removes
this node's own Device (LLR-pt32fx), and then, like any commit that is the
node's own removal, it deletes the record, key included (LLR-b27jr6,
LLR-6p4pj2). `commit_update`
replaces the record's `org_private_key` with its provisional update's
(LLR-6s785x), and `commit_genesis` stores the genesis provisional update's
key (LLR-wzqqg9). `commit_held` draws no key of its own: it takes the
Organisation key pair to store from its caller, as
`commit_held(org_id, verified, org_private_key, org_pub_key)` —
`commit_update` passes its provisional update's pair, and
`commit_received` the carried private key with the Organisation public key
of the chain state read. The record keeps only the current key.
satisfies: REQ-ju6vn2, REQ-3dsweu

*Amended 2026-10-07 (change `worktree-org-node-org-key-pair`, review round 1,
finding-2).* This said "`commit_held` takes no key or secret argument". Since
rotation, `commit_held` writes the key pair its caller passes, so the item
states that interface and where each caller's pair comes from.

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This stated, without endorsing it, that a later received update overwrote the
stored Organisation secret with whatever the Wire message carried, including
nothing (PR-xwek5e), and was derived. The owner ruled that the secret is
replaced by the Organisation private key, which every Organisation-information
message carries and a revocation never does (REQ-ju6vn2, REQ-3dsweu).
PR-xwek5e is resolved by that ruling, and the item now satisfies those
requirements. By the owner's later rulings the same day, `commit_update`
replaces the record's key with its provisional update's (rotation,
REQ-jy6ybw), where it had left the stored secret as it was; and a revocation
is accepted only as the node's own removal (relabelling, REQ-3dsweu as
rewritten), so no received revocation commits into a kept record, where an
earlier draft of this change had it keep the stored key.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The last sentence is added: `commit_update` is a new commit path, and it
carries no Wire message to take a secret from. The received-update clause is
unchanged; the keep-or-clear rule that resolves PR-xwek5e belongs to the
Organisation key-pair change.

**LLR-e5c9ud**: the Persona marked Active is the first in the store whose
DevicePublicKey is in the verified trie, among the Personas the commit may
bind (LLR-eyc4ud: on a first admission the unbound ones, otherwise those bound
to the received change's Organisation), and it is given the member id of the
member holding that DevicePublicKey and the received change's Organisation.
satisfies: derived

*Amended 2026-10-06 (REQ-yp75u9, independent review round 1 of change
`worktree-org-node-chain-authority`).* The selection ran over every Persona and
the last sentences read "Which Organisation that Persona was bound to before
is not consulted. That is PR-mdv38y." It is now scoped by LLR-eyc4ud, and
PR-mdv38y is resolved.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item compared the Persona's device key with the administrator's member key,
correct only while both were one ed25519 key. The code compares the two
Member-as-a-group keys. The last sentence follows the owner's answer Q1 on
where the record's administrator key comes from.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This also excluded a Persona whose member was "the administrator's
Member-as-a-group key" — the member whose key equalled the chain's Organisation
public key. org-node has no administrator, so the exclusion goes; the
evidence note below records that it was never evidenced.

*LLR-xq9nrq, LLR-ckk5nz and LLR-e5c9ud were added 2026-10-04 by review round
5, which found these writes stated by no requirement. Zeroing the administrator's
Member-as-a-group key was green. The secret on first admission and the member id
were already evidenced, by `a_receiver_holding_two_organisations…`,
`five_stories_full_e2e` and `a_self_delete_reaches_the_revoked_nodes_disk`
among others. All three are now carried by
`a_first_admission_records_the_signing_key_the_secret_and_the_member`. The
administrator-key exclusion in LLR-e5c9ud is part of PR-mdv38y's site and is
**not** evidenced: removing it leaves the gate green. It is stated so that the
fix has something to keep.*

*(Note 2026-10-05, review round 3, finding-15, change
`worktree-person-shared-types`: the test named above is now
`a_first_admission_records_the_invites_administrator_the_chains_key_the_secret_and_the_member`,
renamed by review round 2. The administrator-key exclusion is evidenced since
review round 1 of that change, by
`the_administrators_own_persona_is_never_the_one_a_receive_marks`
(`org-node/tests/admission_sender.rs`), which review round 3's mutation M14,
removing the exclusion, turned red. PR-mdv38y, the unbound Persona selection, is
unchanged.)*

*(Note 2026-10-06, change `worktree-org-node-chain-authority`: with the
administrator gone the first test is now
`a_first_admission_records_the_chains_key_the_secret_and_the_member` and
`a_first_admission_records_the_chains_organisation_public_key` carries
LLR-xq9nrq; the exclusion is removed, and its pin is rewritten to the amended
LLR-e5c9ud as
`the_persona_marked_active_is_the_first_whose_device_is_in_the_record`.)*

*(Note 2026-10-06, change `worktree-org-node-org-key-pair`: the first test
named in the note above is now
`a_first_admission_records_the_chains_key_the_private_key_and_the_member`,
renamed with the Organisation secret's removal.)*

*Re-traced 2026-10-04 by review round 6. All three said `satisfies:
REQ-xa6smf`, which states only the first-admission sender check against the
invite. None of the three refines that check: the secret overwrite on later
updates, the Persona selection on every receive, and the administrator's Member-as-a-group key on the member's record. Tracing them there took them out of the derived-behaviour
risk assessment. They are `derived` now, and assessed in
`org-node/docs/risk/2026-10-03-architecture-derived.md`.*

*Also noted by review round 6: REQ-uxv2x2, what a node does on committing a
Change set that removes its own Device key, is not scoped to a receive
operation, and this item does not apply it. On this path the node's own
removal is committed as an ordinary update. That is PR-322qst.* *(Resolved
2026-10-06 by change `worktree-org-node-chain-authority`: REQ-uxv2x2 applies on
every commit path (LLR-b27jr6), and the pin is rewritten as
`an_own_revocation_on_the_ordinary_path_deletes_the_record`.)*

*Added 2026-10-04 by review round 4. Round 3 wrote LLR-vdyu65 for this clause
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
stayed. An expectation names the Organisation alone now (LLR-9zfnmb), and at
most one is held per Organisation.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said a pending Invite is discarded only once the first admission has
committed, and satisfied REQ-xa6smf. The Invite leaves org-node; the
expectation the app declares takes its place (owner ruling 5), cleared at the
same point.

*Amended 2026-10-06 (owner ruling on pre-emption, REQ-8amu2a as amended).*
This said "an expected admission is cleared … once a first admission to that
Organisation has committed". An expectation names an invite identifier too,
so only the matching one is cleared; another expectation for the same
Organisation (a second Invite) stays.

## SDD-72ddm6 — Revocation and self-delete

`org-node/src/service.rs` (`revoke_member`,
`receive_and_self_delete_if_revoked`, `SelfDeleteOutcome`)

**SDD-72ddm6**: what a node does when the change it receives removes **it** —
the one path where committing a verified change means deleting the record
rather than updating it. The item also owns the revoking side, `revoke_member`,
which builds that change. Nothing about the sender is checked on this path.
traces: REQ-uxv2x2, REQ-nhe2zu, REQ-txvtm9, REQ-xs4ab8, REQ-tqap3r, REQ-ztdza4, REQ-bwx7eg, REQ-ju6vn2, REQ-3dsweu, REQ-vxqc5g, REQ-stx9v3

*Amended 2026-10-06 (owner ruling on rotation, change
`worktree-org-node-org-key-pair`):* traces REQ-stx9v3. `revoke_member` builds
through `keep_change_set`, so a revocation draws a fresh Organisation key pair
once its root is calculated (LLR-e2b7gv; LLR-tax3pm amended); the removed
Device keeps only the key of the epoch before its removal.

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The receiving half now takes either kind of Wire message. On an
Organisation-information message it applies the key check (LLR-ba2ejp) and,
on an update branch, stores the received key (LLR-ckk5nz amended). A
revocation it accepts only as this node's own removal, which deletes the
record; one after whose verified record this node's Device is still listed
is refused with `RevocationNotForThisDevice`, store unchanged (LLR-pt32fx,
owner ruling on relabelling); and one about an Organisation not held is
refused before the chain is read, as any such message is (LLR-379hnv,
LLR-38e2kn). The revocation `send_update` sends a removed Device carries no
Organisation private key (LLR-8hdu9x amended). The item traces REQ-bwx7eg,
REQ-ju6vn2, REQ-3dsweu and REQ-vxqc5g too.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item did not name the administrator's side, which its interface
(`revoke_member`) already held, and said nothing of the sender. The owner
ruled that nothing about the sender is checked on any Receive operation.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The receiving half is unchanged in what it decides, but it now reads the chain
only after the chain-free checks (LLR-9ew26y) and checks nothing about the
sender on either branch (LLR-3q63zv), and the deletion it performs is no longer
its alone: every commit path deletes the node's record on its own removal
(LLR-b27jr6). The revoking half, `revoke_member`, builds a revocation
provisional update and nothing more; the app submits it through
on-chain-client, `commit_update` commits it, and `send_update` sends it. The
item traces REQ-xs4ab8, REQ-tqap3r and REQ-ztdza4 for those.

**LLR-vw2jn6**: the change is verified against the chain **before** anything is
decided about the node's own membership.
satisfies: REQ-uxv2x2

**LLR-6p4pj2**: when the node's own device key is absent from the verified
trie, the Organisation record is deleted and every persona bound to that
Organisation is marked revoked.
satisfies: REQ-uxv2x2

**LLR-jsx922**: when the device key of a Persona bound to this Organisation is
still present after an Organisation-information message, the change is
committed as an ordinary update and reported as such, with nothing deleted;
after a revocation it is refused with `RevocationNotForThisDevice`, nothing
committed or deleted (LLR-pt32fx).
satisfies: REQ-uxv2x2, REQ-3dsweu

*Amended 2026-10-06 (owner ruling on relabelling, change
`worktree-org-node-org-key-pair`).* This committed every verified change that
left the node's Device listed. The owner ruled that a revocation is accepted
only as the receiving Device's own removal, so a relabelled
Organisation-information message cannot be committed as a keyless update; the
item now scopes the update branch to Organisation information.

*Clarified 2026-10-04 by review round 5. This read "the node's own device
key", which is the property a reader wants and not the one the code checks.
The code consults the Personas whose binding names this Organisation, and
PR-mdv38y shows that binding can be rewritten by the other receive path, after
which a node still in the trie self-deletes.*

**LLR-sxd3tg**: in that update the record the change names takes the new root,
epoch, sequence mark and member snapshots, and the store is saved before the
outcome is returned.
satisfies: REQ-uxv2x2

*Added 2026-10-04 by review round 5, which measured that dropping the
sequence-mark write from this branch left the gate green. "Committed as an
ordinary update" covered it in words only.*

**LLR-3q63zv**: `receive_and_self_delete_if_revoked` does not check the
sender's authenticated Device key, on the branch that deletes the record and
on the branch that updates it. A removal or an update relayed by any device is
acted on when it verifies against the chain.
satisfies: REQ-ztdza4

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the path does not cross-check the sender because a node being
removed cannot be required to find the remover in a record it is no longer
part of, and booked the update branch's missing check as PR-u4c2vp. The owner
ruled that nothing about the sender is checked on either branch, so PR-u4c2vp
is resolved by that ruling.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said the path does not cross-check the sender because a node being
removed cannot find the remover in a record it has left, and that the missing
check on the not-removed branch was PR-u4c2vp's defect; it was derived. The
owner ruled that nothing about the sender is checked on any path
(REQ-ztdza4 as amended), so the behaviour PR-u4c2vp pinned is now the required
behaviour on both branches.

**LLR-6qmq2g**: a change that fails verification leaves the Organisation record
in place.
satisfies: REQ-uxv2x2

**LLR-379hnv**: a change about an Organisation the node holds no record of is
refused with `OrgNotOnChain` without reading the chain, and nothing is
written: no record, and no expected admission cleared. The error's name is
wrong — the refusal is about the local record, not the chain. That is recorded
for the fix change and not endorsed.
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This said "no pending invite consumed" and that the chain "was read just
before". Invites leave org-node, and the chain read that came first existed
only to obtain the signing key; with it gone, an unheld Organisation is refused
before any chain read, as RC-mj6gjq asks of every cheap refusal.

*Added 2026-10-04 by review round 7, which found this refusal stated by no
requirement and declared nowhere. It is the mechanism behind the verification
record's Gap 20: the shipped app receives only on this path, so a joiner's
first admission cannot complete there.
`the_self_delete_path_refuses_an_organisation_it_holds_no_record_of` pins it.*

**LLR-jwhzh3**: this path acts only on the Organisation the received change
names — the record it verifies against, the Personas whose binding names that
Organisation (and only those) to decide whether the node is still a member,
the record it deletes or updates, and the Personas it marks Revoked — so a node
revoked from one Organisation keeps every other Organisation's record,
Organisation private key and Personas.
satisfies: REQ-uxv2x2

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
"secret" read the Organisation secret, which is removed; the record's
Organisation private key takes its place.

*Narrowed 2026-10-04 by review round 5.* This ended "and a Persona bound to
another Organisation cannot keep it in this one". That is true of this path
and false of the device. The binding this path consults is rewritten by
`receive_and_verify` (PR-mdv38y), and once it has been, a Persona that was
bound to another Organisation **does** keep the node in this one. Round 4's
test delivered the enrolment through this path, which never rebinds, so it
could not see that. The clause is withdrawn until PR-mdv38y is fixed.

*Added 2026-10-04 by review round 4 with LLR-y2v8v2, and this is the half that
was worst: `self.store.data_mut().orgs.retain(|_o| false)` — delete **every**
Organisation record the node holds, rather than the one it was revoked from —
was green, as were the lookups for the local Membership record, the update
branch's record, the Personas to revoke, and the "am I still a member" test
itself. The last of these needs a scenario to be observable at all: org 2's
administrator enrolling the device key of B's **org 1** Persona, so that a test
consulting every Persona finds B still present in an Organisation it was just
removed from. That is also why the clause matters — without it an
administrator could pin a member in place by enrolling a key the member uses
elsewhere.*

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

**LLR-6dc598**: `revoke_member(rng, org_id, member_id) ->
Result<ProvisionalUpdate, OrgNodeError>` deletes the member from the trie
rebuilt from the record and stores, through `insert_provisional`, a revocation
provisional update; it calls no chain operation, binds no endpoint, sends
nothing and leaves the record unchanged.
satisfies: REQ-xs4ab8

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the signed revocation.

**LLR-tax3pm**: a revocation provisional update carries what LLR-ghja3x
states for every provisional update — the record's root as base, the
Sequence number the epoch the update produces on the chain, the record's
epoch plus one (REQ-txvtm9), and the public key of a fresh Organisation key
pair, whose private key it holds (LLR-e2b7gv) — and is signed by no key.
satisfies: REQ-xs4ab8, REQ-stx9v3

*Amended 2026-10-06 (owner ruling on rotation, change
`worktree-org-node-org-key-pair`).* This said the revocation carries "the
record's Organisation public key", as LLR-ghja3x did. A revocation now
rotates the Organisation key pair like every other update (REQ-stx9v3), so
the removed Device's copy of the earlier key is not the Organisation's key
once the revocation commits.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the revocation is signed by the administrator's Member-as-a-group
key and carries the last sequence number plus one.

*Amended 2026-10-06 (merge of master `5f7c177`):* "one greater than the
record's mark" reads "the record's epoch plus one", for the reason given
under LLR-ghja3x.

**LLR-qg9utu**: the revoking node's Organisation record takes the removal only
through `commit_update`, once the revocation has verified against the chain,
and is never changed by `revoke_member` or by `send_update`.
satisfies: REQ-tqap3r

**LLR-pw369n**: in Loopback mode `send_update` refuses with a typed error, and
sends nothing, when its caller passes no `EndpointAddr`, rather than attempting
a peer it cannot name; in Loopback mode with an address it dials that full
address (LLR-jn5jeh).
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-6dc598 said `revoke_member` submits the new root on chain before the
signed revocation is sent; LLR-tax3pm that the revocation is signed by the
administrator's Member key; LLR-qg9utu that the administrator's record is
updated after the send (PR-vt244s's order); LLR-pw369n that a Loopback
revocation with no address is refused. All four were derived. org-node no
longer writes to the chain, signs nothing and has no administrator; revocation
builds a provisional update, the commit writes the record, and the dial and
its refusal move to `send_update`. PR-b9wab3's mechanism — the address check
sitting after `submit_update`, so a call with no address burned an epoch — no
longer exists in org-node: the chain write is the app's, and the refusal now
comes after a commit the chain already agrees with.

*Added 2026-10-04 by review round 3. This is the fourth clause of the same
interface, and round 2 wrote the other three: LLR-6dc598, LLR-tax3pm and
LLR-qg9utu are the twins of LLR-rb8r65, LLR-ghja3x and LLR-t4znbk, and the
twin of LLR-jn5jeh was left unwritten. The behaviour was **evidenced but
unstated** — replacing the Loopback arm's `ep.send(addr, …)` with a dial by
endpoint identity reddens six tests including
`a_revocation_reaches_the_administrators_disk` — so this closes an undeclared
partial item rather than an unevidenced clause. `revoke_member` also carries a
refusal `admit_member` has no counterpart for: a Loopback revocation with no
peer address cannot fall back to discovery, so it is refused rather than
attempted.*

*Corrected 2026-10-04 by review round 4. The sentence above ended "so it is
refused **before anything is sent**", which reads as though nothing had
happened yet. Nothing has been *sent*, but the chain has already moved:
PR-b9wab3, opened by this change, records that the check sits after
`submit_update`, so a call with no address burns an on-chain epoch and then
refuses. The shipped test asserts the epoch advanced, pinning the defect so
that correcting the order reddens it.*

**LLR-8hdu9x**: the Wire message `send_update` sends to the Device a
committed revocation removed — which the node's committed record no longer
lists — is a `Revocation` carrying the committed Envelope and nothing else:
no member snapshot and no Organisation private key. Every Device the record
still lists is sent an `OrgInformation` carrying the committed Envelope, the
member snapshots as they were **before** the committed update, and the
Organisation private key of the epoch the update reaches, as the node's
record holds it (LLR-6ymd6d). A receiver accepts the revocation only as its
own removal and deletes its record; one whose Device is still listed refuses
it (LLR-pt32fx).
satisfies: REQ-3dsweu, REQ-szq3ud

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
This said the message carries exactly the Organisation secret the caller
passes — none for a revocation — and that a receiver stored that absence
over the secret it held (PR-xwek5e, stated and not endorsed); it was derived.
The owner ruled that the kind of message depends on the recipient, that a
revocation carries no key, and (later the same day, on relabelling) that a
receiver accepts a revocation only as its own removal (REQ-3dsweu,
REQ-szq3ud). PR-xwek5e is resolved by those rulings.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the signed envelope.

**LLR-drgdy8**: `revoke_member` neither reads nor changes the proxy account its
Organisation record holds; the app reads it through `proxy_account`
(LLR-3v5nu9) to submit the revocation.
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
LLR-8hdu9x described the message `revoke_member` sent, with a signed
Envelope; the send is now `send_update`, the Envelope is unsigned, and the
secret is whatever the caller passes (unchanged from `admit_member`'s
`org_secret` argument until the Organisation key-pair change). LLR-drgdy8 said
`revoke_member` submits with the proxy account; the app submits now, and
org-node holds the account opaquely (owner ruling 6).

*Added 2026-10-05 by review round 8, which measured that emptying the snapshot and passing no proxy account were both
green, and that the absent secret was observed only by the PR-xwek5e pin.
LLR-8hdu9x is carried by `a_revocation_reaches_the_administrators_disk`, and
its receiver clause by the PR-xwek5e pin (review round 9), and
LLR-drgdy8 by `the_proxy_account_from_genesis_is_kept_and_handed_back` (renamed
2026-10-06 from `…_kept_and_passed_on_every_update`).*

## SDD-b8tuv3 — Endpoint lifecycle in the service

`org-node/src/service.rs` (`ensure_endpoint`, `set_transport_mode`,
`with_endpoint`, `endpoint`)

**SDD-b8tuv3**: when the transport comes into existence and whose key it
carries — bound lazily, at most once, from the device seed of a named persona.
traces: REQ-db6s7q, REQ-ztdza4

**LLR-6zjzn2**: `ensure_endpoint` binds at most one endpoint per service and
returns that same endpoint on every later call. On a service holding more than
one Persona this is the defect PR-8qsnhx books, and it is stated here and not
endorsed.
satisfies: REQ-ztdza4

**LLR-cns6q6**: the endpoint is bound from the device seed of the persona named
by the call that first binds it, so the identity the transport authenticates
is that persona's device key for every later send, whichever persona a later
call names.
satisfies: REQ-ztdza4

*Narrowed 2026-10-04 by review round 5. "The named persona's device key" was
true of the first call only. With LLR-6zjzn2's bind-once rule, an
administrator holding two Organisations through two Personas sends the second
Organisation's admissions under the first Persona's key, and the joiner
refuses them. That is PR-8qsnhx, pinned by
`pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key`.*

**LLR-ecz9a6**: in Loopback mode the endpoint binds loopback addresses and
nothing else.
satisfies: REQ-db6s7q

*Narrowed 2026-10-04 by review round 2, exactly as LLR-jn5jeh was narrowed by
round 1 and for the same reason. The clause used to read "the endpoint is bound
in the transport mode the service holds at the time of binding", and replacing
`self.transport_mode` in `ensure_endpoint` with a hard-coded
`TransportMode::Loopback` left the whole gate green: `OrgService::new` already
sets Loopback, and no gated test may set Networked, because `bind_with_mode`
then uses `presets::N0` and would reach the public internet. That the function
reads the field rather than a constant is therefore unevidenced at this gate,
and it is recorded below among the behaviours that are deliberately not
refined. The `test-support` relay-injecting constructor already booked for the
Networked arm of `admit_member` is what closes both.*

## SDD-z85ux9 — The chain-facing I/O shell

`org-node/src/chain_read.rs` (all but `org_state_from_chain` and
`OrgStateCache`, which are SDD-pa6p7w's since 2026-10-05),
`org-node/src/chain_write/proxy.rs`, `org-node/src/chain_write/submit.rs`, `org-node/src/chain_write/mod.rs`,
`org-node/src/ceremony.rs`, `org-node/src/preflight.rs`,
`org-node/src/bin/preflight.rs`, and in `org-node/src/service.rs`
`connect_chain_client` and the `SubxtChainOps` implementation of `ChainOps`;
in `org-node/src/chain_write/multisig.rs`, the asynchronous functions only —
`dispatch_org_call`, `fund` and `submit_and_watch`. *Corrected 2026-10-05 by
review round 8: this also named `calldata.rs`, which holds no asynchronous
function. Its two functions are SDD-msb6xh's.* *(Amended 2026-10-05 by the
org-node type-safety change, review round 7: three since that change added
`update_calldata`, also SDD-msb6xh's. LLR-mmdu38, under SDD-swtd3w, names
`OnChainReader::refresh` only as the caller of `OrgStateCache::store_fetched`.
The clause is carried by the cache, SDD-pa6p7w's, and this item still
carries no low-level requirement.)*

*Corrected 2026-10-04 by review round 6. This list named
`revive_update_runtime_call` as one of "the asynchronous functions" and left
out `submit_and_watch`. The first is a pure, synchronous `Value` builder that
a test can pin without a chain, so the deviation's argument never applied to
it. It now belongs to SDD-msb6xh, with LLR-rc74nq. The second is async and
belongs here.*

**SDD-z85ux9**: everything that actually talks to a chain — opening an RPC
connection, reading an Organisation's state through on-chain-client, and the
operator-facing preflight checks. It submits nothing: org-node builds, signs
and dispatches no extrinsic. It is the unit's whole supplier-facing surface
and it carries **no low-level requirements**.
traces: REQ-nhe2zu, REQ-txvtm9, REQ-bvh8v6

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This listed "building and signing an extrinsic, submitting it, driving block
production, creating and rotating a pure proxy, running the genesis ceremony".
The owner ruled that org-node does not write to the chain: `chain_write/`
(`proxy.rs`, `submit.rs`, `mod.rs`, and the asynchronous `dispatch_org_call`,
`fund` and `submit_and_watch` in `multisig.rs`), `ceremony.rs`,
`FinalitySink` and the write half of `SubxtChainOps` move to on-chain-client's
chain writer
(`on-chain-client/docs/architecture/2026-10-06-chain-write.md`),
and org-node drops `subxt-signer` and the sr25519 key. What stays here is
`chain_read.rs` (but for SDD-pa6p7w's parts), `preflight.rs`,
`bin/preflight.rs`, `connect_chain_client` and `SubxtChainOps::read_state`.
The size measured below is of the shell before this change; it is to be
re-measured when the change is implemented, and what the chopsticks targets
that exercised the write (`chain_genesis_e2e`, `finality_polling`) still
exercise here is for the implementation plan to settle.

## The item with no low-level requirements

SDD-z85ux9 carries none, under a class that requires them. This section is the
deviation, recorded here rather than left for a reader to notice.

**What it is not.** The `on-chain-client` unit has an item of the same shape —
its chain-facing transport shell, the one item its decomposition records as
carrying no low-level requirements, described in the section "The item with no
low-level requirements" of
`on-chain-client/docs/architecture/2026-09-28-decomposition.md` — and **its
argument does not transfer.** That item is *unreachable*: nothing exercises it
at all, so there is no behaviour anyone has watched. This one is exercised.
`org-node/tests/chain_genesis_e2e.rs` is
507 lines over **two** tests, `finality_polling.rs` 129 lines over two more,
`preflight.rs` 85 lines over three; **seven in all**. Together they drive the
ceremony, the proxy, the submission path and the health checks end to end
against a real runtime.

*Corrected 2026-10-04 by review round 1*, which found this sentence claiming
eight, four and six — eighteen tests, two and a half times what the tree holds.
The line counts were right and the test counts were taken from a `grep` that
counted helper functions as tests. The argument below does not turn on the
number, but a deviation this large should not be argued from a figure nobody
measured.

**What it is.** Those three targets are excluded from this unit's
`verify_commands`, by a decision recorded in `org-node/.guardrails/config.yaml`:
they spawn a chopsticks fork and need `on-chain/scripts/node_modules`, neither
of which the gate provides. So the honest statement is narrower than
on-chain-client's, and worse:

> A low-level requirement written here could be true, could be well-tested, and
> **no mutation to the code it describes would redden this unit's gate.** It
> would be an assertion carrying no evidence that this gate can produce.

That is precisely the failure this repository has now recorded three times — a
property credited to an assertion that cannot observe it — and writing
eighty-odd lines of LLR into the one part of the unit where the gate is blind
would be committing it deliberately, at scale, in the ledger rather than in a
test.

**What follows from it.** The remedy is not to write the LLRs. It is to bring
the three chopsticks targets into the gate, which is an infrastructure change
of its own — a hermetic chopsticks fixture, or a recorded-transcript substitute
for the RPC layer — and it is booked in
`docs/plans/2026-09-05-ratchet-setup.md` rather than attempted here. Until then
this unit's architecture is **described** across its whole surface and
**evidenced** across about seventy per cent of it, and the rest is named.

**Its size, measured.** `chain_read.rs` 67, `chain_write/proxy.rs` 225,
`chain_write/submit.rs` 77, `chain_write/mod.rs` 70, `ceremony.rs` 66,
`preflight.rs` 138, `bin/preflight.rs` 97 — 740 lines of whole files, plus
the parts: `mod subxt_impl` in `service.rs` (the `SubxtChainOps`
implementation) 260, `connect_chain_client` 40, and in `multisig.rs`
`submit_and_watch` 46, `fund` 24 and `dispatch_org_call` 15. Each part is
counted from its first doc comment or attribute to its closing brace. **1125
lines in all, against a unit of 3768 lines over 24 files: about 30%.**

*Re-measured 2026-10-05 by the org-node type-safety change, at its merge of
this file, with the same rule.* `chain_read.rs` 55 (its 101 lines less the 46
of `org_state_from_chain` and `OrgStateCache`, now SDD-pa6p7w's),
`chain_write/proxy.rs` 234, `chain_write/submit.rs` 80, `chain_write/mod.rs`
70, `ceremony.rs` 68, `preflight.rs` 138, `bin/preflight.rs` 97 — 742 lines
of whole files and the remainder of one; the parts `mod subxt_impl` 249,
`connect_chain_client` 40, `submit_and_watch` 46, `fund` 24 and
`dispatch_org_call` 15 — 374. **1116 lines in all, against a unit of 4391
lines over 25 files: about 25%.** The share fell because that change added
`types.rs` and typed parts the gate does reach, not because the shell shrank.

*Measured 2026-10-04 by review round 6, which found this paragraph counting
only the whole files and calling the result "a fifth", while the parts were
left uncounted. The parts are a further 385 lines. The ledger said "a fifth",
and the README and the plan carried "roughly 815". Neither was measured. The
figure above is, and `revive_update_runtime_call`'s 45 lines are no longer in
it.*

*Re-measured 2026-10-06 by change `worktree-org-node-chain-authority` (T17),
after the chain write left org-node, with the same rule: whole files plus the
named parts of `service.rs`, each part from its first doc comment or attribute
to its closing brace.* `chain_read.rs` 55 (its 97 lines less the 42 of
`org_state_from_chain`, `OrgStateCache` and the cache's `ChainReader` impl,
SDD-pa6p7w's), `preflight.rs` 138, `bin/preflight.rs` 97 — 290 lines of whole
files and the remainder of one; the parts `mod subxt_impl` 36 (`read_state`
only) and `connect_chain_client` 41 — 77. `chain_write/` (`proxy.rs`,
`submit.rs`, `mod.rs`, `multisig.rs`'s asynchronous functions) and
`ceremony.rs` are deleted. **367 lines in all, against a unit of 3370 lines
over 18 files: about 11%.** The chopsticks targets `chain_genesis_e2e` and
`finality_polling` are deleted with the write they drove (its evidence moved
to on-chain-client's `write_genesis_e2e`); `preflight` (85 lines, three
tests) is the one chopsticks target left, so the shell this section describes
is now exercised only by the health checks, still outside the gate.

**What is still true of it.** It is class C like everything else here, it is
described by a software item with an interface and a trace, and its
requirements are the unit's. What it lacks is refinement into separately
mutable claims, and the reason is stated above rather than implied.

## What is not an item

`org-node/src/lib.rs` is seventy-eight lines *(eighty-five since the org-node
type-safety change, 2026-10-05, which re-exports its value types there)* and
is named by no software item: the
crate doc comment, the module declarations, the feature gates and the public
re-export surface. Every behaviour it exposes belongs to the item that defines
it. Naming no item for it is not a claim that it decides nothing — the feature
gates decide whether `service.rs`, the chain modules and the transport compile
at all, which is a segregation decision and is stated in the README's Overview.

`org-node/src/test_fixtures.rs` is seventy-five lines *(145 since the org-node
type-safety change, 2026-10-05, which added typed key fixtures and the
`Display`/`Copy` probes)* behind the `test-support`
feature, which `org-node/Cargo.toml` states is never enabled in a production
build. It is the route by which four of this unit's gated targets reach the
constructors they need. It is a **test seam, not a software item**: it adds no
behaviour of its own. But as with on-chain-client's `test_support`, if it were
deleted several items would lose their evidence at once, and that is worth a
reader knowing rather than discovering.

`org-node/src/chain.rs`'s `MockChain` and `service.rs`'s `MockChainOps` are
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
PR-8qsnhx.*

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

**The Networked arm of `admit_member` and `revoke_member`.** LLR-jn5jeh used to
claim both dialling modes — "in Loopback mode … and in Networked mode by the
endpoint identity derived from the joiner's device key". Review round 1 found
the Networked half credited to tests that cannot observe it: no gated test ever
puts an `OrgService` into `TransportMode::Networked` (only `preflight.rs` does,
and that target is chopsticks-excluded), and replacing the whole Networked arm
with the Loopback body leaves the entire gate green. The requirement now states
only the Loopback clause.

The Networked arm is therefore **SDD-z85ux9's problem in miniature, inside an
item that is otherwise well evidenced**: it is real code on a real path, and
this gate is blind to it. It cannot be brought into the gate the way the
Loopback path was, because `bind_with_mode(Networked)` uses `presets::N0` —
n0's relay servers and DNS discovery — so a gated test of it would reach the
public internet. What would close it is a `test-support` constructor taking an
injected relay, the same seam `transport_networked` already uses at the
endpoint layer but which `OrgService` does not expose. Booked in
`docs/plans/2026-09-05-ratchet-setup.md`.

**That `ensure_endpoint` binds in the mode the service holds.** The third
behaviour, found by review round 2 and the same gap one layer down: replacing
`self.transport_mode` with a hard-coded `TransportMode::Loopback` leaves the
entire gate green, because `OrgService::new` already sets Loopback and no gated
test may set the only other value. LLR-ecz9a6 has been narrowed to the clause
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
(`org-node/src/transport/endpoint.rs`). A low-level requirement for that arm was
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
| LLR-p8uu47 (build sets the fields it is given, in a canonical encoding) | LLR-ybn5pr, LLR-cs4mpb, LLR-9sknpa (the receiver's own refusals of an altered field) |
| LLR-er2x8n (a frame within the bound round-trips) | LLR-sc6zuh, LLR-8kh3zf (over the bound, both directions) |
| LLR-fa7jt8 (the frame layout) | LLR-pkruy8 (bytes that are not a frame) |
| LLR-kkj64b (the parsed `Joiner` is what is admitted) | LLR-tcft2r (a record snapshot that does not decode) |
| LLR-wusj89 (the file's layout) | LLR-q5n28x (a file too short to have one) |
| LLR-t4u66w (the wrong passphrase yields an error) | LLR-wusj89 (the right one yields the data) |
| LLR-wx3php (strictly greater is accepted) | — the same LLR states both sides |
| LLR-jsx922 (still a member: update) | LLR-6p4pj2 (no longer a member: delete) |
| LLR-6qmq2g (verification fails: record stands) | LLR-vw2jn6 (verification runs first) |
| LLR-rm9x4z (absence from failure) | — the same LLR states both sides |

*Amended 2026-10-05; reworded the same day by
docs/plans/2026-10-05-switch-trim.md. The Envelope carries no signature
(REQ-ag6kqm), so the first two rows above, and "SDD-sxp8hb's abnormal side is
LLR-9fvb3y" below, describe texts since amended in place. LLR-9fvb3y, amended
in place, now states that a relay by another device is not refused for that;
LLR-ybn5pr, LLR-cs4mpb and LLR-9sknpa state the receiver's own refusals of an
altered field. SDD-sxp8hb's abnormal side is LLR-3jjgtw's refusals, in
`org-node/docs/architecture/2026-10-05-unsigned-envelope.md`.*

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
Three rows changed with the requirements they pair. The signature pair
(LLR-na7p4w, LLR-9fvb3y) and the signed-transcript pair (LLR-p8uu47 against
the three tampering requirements) are now the seed round trip against another
seed's keys, and the Change set `build` transmits against each field's
alteration reaching the decoded value. The blob pair (LLR-g9vmbx and
LLR-kkj64b against LLR-8qxwst and LLR-tcft2r) is now the parsed `Joiner`
against a record snapshot that does not decode; LLR-g9vmbx and LLR-8qxwst
state absences and have no other side. The new requirements of
`2026-10-06-chain-authority.md` pair as follows:
LLR-cmdrp9 (a commit that verifies) against LLR-ewkg85 (one refused at any
step); LLR-95753m (a provisional update kept) against LLR-jq7qh7 (one over the
bound refused); LLR-s8xp7m states both sides of the expectation.

*Recomputed 2026-10-06 at the merge of master `5f7c177`.* The first two rows
are re-derived from the merged texts, master's for both pairs: LLR-e58j8m has
no other side in this file (SDD-sxp8hb's is LLR-3jjgtw's refusals), and
LLR-p8uu47's other side is the receiver's refusals of an altered field. The
`Joiner` row and this change's new pairs stand as written above. SDD-na9nc3
still has ten LLRs here.

**Items whose abnormal side is carried by another item.** SDD-na9nc3's ten
LLRs in this file are *all* refusals except LLR-8n95rf, LLR-d6kvbx and, since
2026-10-05, LLR-mcdh85; its normal case is
REQ-nhe2zu's accept path, carried by `happy_path_commits_when_root_matches_chain`.
SDD-sxp8hb's abnormal side is LLR-9fvb3y. SDD-swtd3w's constructor
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
| SDD-89es4z | a Persona created with a handle org-members will not accept. *Amended 2026-10-05 by the org-node type-safety change: `create_persona` now takes a parsed `Handle`, `Name` and `Surname` (LLR-g76zqd), so this input can no longer be offered to it; the refusal moved to `PersonaDetails::parse` (LLR-q6n25z, under SDD-af5vnt in `2026-10-04-type-safety.md`), whose tests carry the abnormal case. The item's own interface still has none.* |
| SDD-b8tuv3 | an endpoint bind that fails |
| SDD-rx2yvy | `admit_member` given a malformed member or device key (→ `OrgNodeError::Chain("bad member key")`) or a handle already held (→ `OrgNodeError::Trie`) — **the most material of the seven**, because both are reachable from a join request a stranger composes. *Amended 2026-10-05 by the org-node type-safety change: `admit_member` now takes the parsed `JoinRequest`, and a malformed key or handle in a Join request is refused at `import_join_request` with `InvalidField` naming it (LLR-8bum44, under SDD-af5vnt in `2026-10-04-type-safety.md` — *amended 2026-10-05 by the org-node type-safety change, review round 7: it read "under SDD-vee2fq"*), so the first clause cannot reach `admit_member` and the `Chain("bad member key")` refusal no longer exists. A handle already held is still untested here.* *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): `admit_member` takes a parsed `Joiner` built by the app from the Invite reply it parsed; a handle already held is still untested, and now reaches `admit_member` from the app's parse rather than from `import_join_request`.* |
| SDD-ueh4tm | a `ChainOps` implementation whose `read_state` or `submit_update` fails. *Since review round 8 the item has one refusal case, the mock's wrong-epoch update (LLR-ryzr8m), but that is the mock refusing, not an implementation failing, so the line stands.* *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): `submit_update` leaves the trait, so the owed case is a `read_state` that fails, as it reaches a commit (LLR-ewkg85 names it among the refusals).* |
| SDD-z85ux9 | every abnormal case it has. The item has no gated test of any kind — that is the deviation recorded at the end of this file — so it has no abnormal one either, and listing it here is the honest bookkeeping |
| SDD-msb6xh | none — `build_update_calldata` is total over its argument types, so there is no abnormal input to offer it. *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): the item now states that org-node holds no calldata; with no interface left it has no input, normal or abnormal.* |
| SDD-rq6nv4 | `multi_account_id` is total over its argument types. `build_dispatch_tx` is **not**: `runtime_call_to_tx` has three `WriteError::MalformedCall` arms, none of them tested, and this change made the function publicly reachable through `org_node::test_support`, so the abnormal input is now one line of test away. *Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`): both functions leave org-node, and the untested `MalformedCall` arms go with them to on-chain-client's chain writer, where they are owed.* |

**SDD-pa6p7w is no longer on this list.** Round 2 found it missing from it —
nothing reachable at this gate could make `ChainReader::get_org_state` return
`Err`, so the failure arm of LLR-8m99q2 and LLR-rm9x4z was untested and
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

Every LLR above, excepting SDD-z85ux9's absent ones, is carried by a test **in
`org-node/tests`** — one of the thirteen harnessed targets there that
`verify_commands` names, or one of its three bolero targets. With `--lib`,
which carries no low-level requirement, the command runs fourteen harnessed
targets. *(Nineteen and twenty since the org-node type-safety change,
2026-10-05, which added `encoding_golden`, `node_value_types`,
`chain_read_state`, `persona_records`, `secret_redaction` and
`calldata_typed`; its requirements' red-to-green record is in its own plan,
`docs/plans/2026-10-04-org-node-type-safety.md`.)* *Corrected 2026-10-05 by review round 8, which counted thirteen in
`org-node/tests`.*

*Corrected 2026-10-04 by review round 1.* This paragraph used to say "eight
harnessed targets" and to allow evidence to sit "in a `#[cfg(test)]` module
under `org-node/src`". Both were wrong: the count is fourteen, and `test_paths`
reads `verifies:` annotations **only** from `org-node/tests`, which is the whole
reason this change relocated twenty-two tests. One `#[cfg(test)]` module
survives in `src` — `chain.rs`'s `mock_chain_returns_set_state` — and it carries
no annotation and no low-level requirement; LLR-rm9x4z is evidenced from
`tests/` instead.

**A test that is already green has never been watched failing.** So each LLR is
discharged by red by mutation: a named mutation to the source, a named test
failing for that reason, the mutation reverted, and the file proved
byte-identical afterwards by SHA-256. Those attestations are in this change's
verification record, not here.

*Two exceptions, both stated rather than counted, and both found by review
round 2 when it checked this claim against the appendix.* **LLR-836z24** —
`import_join_request` stores nothing — and **LLR-65py3d** — the chain seam is a
trait object a substitute can stand in for — are true by the type system rather
than by a red. The first takes no `&mut self`, the second is checked by
`Box<dyn ChainOps>` compiling at all; no mutation can falsify either without
changing a signature, which is a different change and not a measurement. The
compiler is the evidence, and saying so is better than a row in the appendix
that implies a mutation nobody could write. LLR-f5kq88 and LLR-z8fubr's
distinctness clause are compile-enforced in the same way; all four are listed
together in Gap 4 of the verification record.

**One clause of LLR-u6rq4s is not discharged, and the requirement keeps it
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
  ruling: removing it would make one rule apply to both paths. Booked in
  `docs/plans/2026-09-05-ratchet-setup.md`.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
The owner ruled the sender check out on both paths, so LLR-u6rq4s now states
that an update is committed whoever relayed it, and both clauses above — the
discharged one and the undischarged first-admission skip — describe a check
that no longer exists. The new requirement is to be discharged by red in the
usual way: reinstating the membership check must redden its test. The same
holds for the administrator-key exclusion LLR-e5c9ud carried, which was never
evidenced and is removed.

**One low-level requirement cannot be discharged that way, and it is named
rather than quietly counted.** LLR-6adc99 refines REQ-2wzfzv — IPv4 loopback
required, IPv6 optional — and both clauses are about a bind *failure* that no
gated test can produce. `check-trace.sh` is green over it because MISSING-TEST
is satisfied by the annotation alone, which is the third time this repository
has recorded that property, now in a second unit. It is kept by the owner
ruling that kept REQ-2wzfzv itself, and the `test-support` bind-address
constructor that would close it is booked in
`docs/plans/2026-09-05-ratchet-setup.md`.
