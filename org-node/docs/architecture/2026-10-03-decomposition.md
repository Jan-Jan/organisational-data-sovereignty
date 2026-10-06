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
`verify_envelope_against_chain` is forty-two lines and performs eight checks in a
security-critical order; SDD-na9nc3 is the item that owns it, and ten of this
ledger's low-level requirements refine that one ordering. If a reader has time
for one item, it is that one.

## SDD-swtd3w — Value types and the rejection vocabulary

`org-node/src/ids.rs`, `org-node/src/error.rs`, `org-node/src/types.rs`

**SDD-swtd3w**: the identifier that keys an Organisation's on-chain slot,
together with the vocabulary in which every refusal is reported — a typed
variant per rejection path **on the receive-and-commit path**, so a caller can
state *why* a change was refused rather than only that it was.
traces: REQ-gju89b, REQ-9g6as6, REQ-bcxz96, REQ-y7tsft, REQ-8jb4ny, REQ-ech45n

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
satisfies: REQ-ztdza4, REQ-xa6smf

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
Sequence number come first, so stale or misaddressed bytes are never decoded,
and the watermark moves only after the decisive check has passed. Nothing
about who delivered the Envelope is checked.
traces: REQ-wp2nyc, REQ-bvh8v6, REQ-8gz8bu, REQ-gju89b, REQ-ag6kqm, REQ-6yu72z, REQ-mr5abb, REQ-nhe2zu, REQ-txvtm9, REQ-bcxz96

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item put an authenticity check first, on an unauthenticated sender. Nothing
about the sender is checked (REQ-ag6kqm). LLR-9f5hmr, in
`2026-10-05-unsigned-envelope.md`, also refines this item.

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
traces: REQ-eg5j8u, REQ-9g6as6, REQ-y7tsft

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
also constrained by LLR-bwb9pu, under SDD-af5vnt in
`2026-10-04-type-safety.md` — `WireMessage` holds the Organisation secret as
an `OrgSecret`, so its `Debug` shows none of it — so the item traces
REQ-y7tsft too.)*

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
device seeds and the Organisation secret — and the form the file takes, which
is ciphertext under a passphrase-derived key and nothing else.
traces: REQ-hzm4kt, REQ-qn2erx, REQ-y7tsft, REQ-ech45n

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

**LLR-s78sh7**: no member seed, device seed or Organisation secret the store
holds appears anywhere in the written file in clear.
satisfies: REQ-hzm4kt

## SDD-vee2fq — Out-of-band exchange blobs

`org-node/src/blobs.rs`

**SDD-vee2fq**: the two copy-pasteable blobs that bootstrap a relationship
before any channel exists — the Invite an administrator hands out, and the
Join request a prospective member hands back — and the armour they travel in.
traces: REQ-xa6smf, REQ-9g6as6, REQ-qn2erx

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

**LLR-g9vmbx**: an Invite encoded and decoded yields an Invite equal to the
original. It carries the Organisation identifier, its Organisation public key,
and the administrator's Member-as-a-group key, DevicePublicKey and dialling
address.
satisfies: REQ-xa6smf

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

**LLR-kkj64b**: a Join request encoded and decoded yields a Join request equal
to the original.
satisfies: REQ-xa6smf

**LLR-8qxwst**: a string that is not valid base64 is refused with a typed error
rather than panicking.
satisfies: REQ-9g6as6

**LLR-tcft2r**: valid base64 whose decoded bytes are not a valid encoded blob
is refused with a typed error rather than panicking.
satisfies: REQ-9g6as6

## SDD-msb6xh — Update calldata

`org-node/src/chain_write/calldata.rs` (`build_update_calldata`,
`UPDATE_SELECTOR`, `revive_update_runtime_call`, `update_calldata` *(added
2026-10-05, see below)*)

**SDD-msb6xh**: the exact bytes that ask the contract to move an
Organisation's Membership root forward — the one part of the write path that
is pure, and therefore the one part a test can pin without a chain.
traces: REQ-nhe2zu, REQ-txvtm9

**LLR-rv4vux**: `build_update_calldata` emits exactly one hundred bytes: the
four-byte selector, then the new Membership root, then the Organisation public
key, then the expected epoch, in that order.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* The
third field was the Published signing key; it is the Organisation public key.

**LLR-rc74nq**: `revive_update_runtime_call` builds the `Revive.call` runtime
call by name: `dest` is the contract's twenty bytes, `value` is zero,
`weight_limit` carries `ref_time` and `proof_size` at the declared constants,
`storage_deposit_limit` is the declared constant, and `data` is exactly
`build_update_calldata`'s bytes for the same arguments. The field names and
constants are matched against runtime metadata, so each is part of the claim.
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
holds three functions, all this item's. `typed_update_calldata_is_the_pinned_calldata`
in `org-node/tests/calldata_typed.rs` pins its output (LLR-ayrdr8).)*

*Added 2026-10-04 by review round 6, which found this function inside
SDD-z85ux9, the item with no low-level requirements, under a list headed "the
asynchronous functions only". It is neither asynchronous nor unreachable, and
`update_call_names_every_field_and_constant_the_runtime_matches` in
`org-node/tests/chain_write_pure.rs` now pins it.*

**LLR-txqmz4**: the expected epoch occupies the low sixteen bytes of a
thirty-two byte big-endian field, with the high sixteen bytes zero.
satisfies: derived

**LLR-66h529**: `UPDATE_SELECTOR` is the first four bytes of the keccak-256 of
`update(bytes32,bytes32,uint256)`, so a signature drift in the deployed
contract is a changed constant here rather than a silently rejected call.
satisfies: derived

## SDD-rq6nv4 — Threshold-1 multisig account derivation

`org-node/src/chain_write/multisig.rs` (`multi_account_id`,
`build_dispatch_tx`)

**SDD-rq6nv4**: the derivation of the pseudo-account that controls an
Organisation's on-chain slot, and the choice between dispatching directly and
dispatching through it — both pure, both mirroring a runtime pallet this unit
does not contain.
traces: REQ-nhe2zu, REQ-txvtm9

**LLR-463d89**: `multi_account_id` sorts the signers before hashing, so the
account it derives does not depend on the order they were supplied in.
satisfies: derived

**LLR-8m3bwj**: the derived account depends on the threshold, so two otherwise
identical signer sets at different thresholds derive different accounts.
satisfies: derived

**LLR-f74xwb**: `build_dispatch_tx` dispatches the call directly when there are
no other signatories and wraps it in a threshold-1 multisig when there are.
satisfies: derived


## SDD-ueh4tm — The chain-operations seam

`org-node/src/service.rs` (`ChainOps`, `MockChainOps`, `MockChainInner`,
`ChainOpsReader`)

**SDD-ueh4tm**: the boundary between the five user stories and the chain,
expressed as a trait rather than a client. It exists so the stories can be
exercised in full without a chain, and it is the mechanism that makes
SDD-rx2yvy, SDD-8cpyfa and SDD-72ddm6 verifiable items instead of parts of the
I/O shell.
traces: REQ-nhe2zu, REQ-txvtm9, REQ-bvh8v6

**LLR-65py3d**: `ChainOps` presents reading an Organisation's state and
submitting an update as asynchronous operations on a trait object, so a
substitute may be injected without changing the service.
satisfies: derived

**LLR-hg3xzf**: clones of `MockChainOps` share one chain state, so two services
under test observe the same chain as each other.
satisfies: derived

**LLR-ryzr8m**: `MockChainOps` refuses an update whose expected epoch is not
its slot's current epoch, with a chain error and the slot unchanged, and an
update at the current epoch advances the slot by one. This is the contract's
compare-and-swap as the mock imitates it, so a story exercised against the
mock cannot commit an update the chain would refuse.
satisfies: derived

*Added 2026-10-05 by review round 8, which measured that disabling the check left the gate green: every story submits at
the right epoch. `the_mock_chain_refuses_an_update_at_the_wrong_epoch` carries
it.*

## SDD-89es4z — Persona and Organisation genesis

`org-node/src/service.rs` (`create_persona`, `create_organisation`)

**SDD-89es4z**: how a node acquires an identity and how an Organisation comes
into existence — two keypairs drawn once, a genesis trie, and the first
on-chain slot.
traces: REQ-hzm4kt, REQ-nhe2zu, REQ-txvtm9, REQ-d9g6nt, REQ-qn2erx, REQ-ech45n

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

**LLR-68yd3j**: `create_organisation` persists the Organisation record and marks
the persona Active, and the store is written before it returns.
satisfies: derived

**LLR-rjg3m2**: the founding administrator's `MemberId` is drawn from the
caller's random source before the Membership record is built and is never
computed from a key, so the same persona founding two Organisations receives
two different identifiers.
satisfies: REQ-d9g6nt

**LLR-w3fhhg**: `create_organisation` marks **the named Persona** Active and
binds it to the Organisation it just founded, leaving every other Persona in
the store as it was. Where that Persona was already bound to another
Organisation, the binding is overwritten. That is PR-mdv38y's second writer,
and it is stated here and not endorsed.
satisfies: derived

*Added 2026-10-04 by review round 3 with LLR-vdyu65, and for the same
measurement: `update_persona_status` could mark whichever Persona came first
in the store, with the gate green, because no gated test had two.*

**LLR-dzte8x**: `create_organisation` stores in the new Organisation record
the pure-proxy account the chain returned at genesis, and the account reaches
the disk with the record, so the production chain client can find the proxy
after a restart.
satisfies: derived

**LLR-q3aj8z**: `create_organisation` leaves the founding Persona's member id
as it was. On a new Persona that is none, and the administrator's `MemberId`
is held only in the record's member snapshots. On PR-mdv38y's second-writer
path it is the member id from the Organisation the Persona was bound to
before, which names no member of the new one. That is stated here and not
endorsed.
satisfies: derived

*Added 2026-10-05 by review round 8, which measured that not storing the proxy account and setting the member id were
both green. `MockChainOps` returns no proxy account, so no test could see the
first. LLR-dzte8x is carried by
`the_proxy_account_from_genesis_is_kept_and_passed_on_every_update`, which
substitutes a chain that returns one. LLR-q3aj8z is carried by
`creating_an_organisation_advances_the_chain_and_activates_the_persona` and,
for the second-writer path, by
`pr_mdv38y_founding_an_organisation_rebinds_a_member_persona`.*

## SDD-rx2yvy — Admission

`org-node/src/service.rs` (`admit_member`, `export_invite`, `import_invite`,
`export_join_request`, `import_join_request`)

**SDD-rx2yvy**: the administrator's side of letting someone in — minting the
new member's leaf, moving the on-chain root forward, and handing the new
member the change, unsigned, with everything they need to check it.
traces: REQ-xa6smf, REQ-ztdza4, REQ-nhe2zu, REQ-txvtm9, REQ-d9g6nt, REQ-qn2erx

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item handed the new member the signed change. LLR-8bum44, amended in place,
still constrains it.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
also constrained by LLR-8bum44, under SDD-af5vnt in
`2026-10-04-type-safety.md`. `import_join_request` refuses a Join request
holding a value its type's parse refuses, with `InvalidField` naming the
field, and `import_invite` stores no pending Invite whose keys are not curve
points. So the item traces REQ-qn2erx too.)*

**LLR-rb8r65**: `admit_member` adds the joiner to the trie and submits the new
Membership root on-chain at the next epoch. Only then does it send the joiner
the Envelope carrying the change, so the anchor a receiver will check against
exists before the receiver is told about it.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the signed change.

**LLR-ghja3x**: the Envelope `admit_member` sends names the Organisation the
call names and carries as its Sequence number the epoch its own chain update
produced, which is greater than the record's last. It carries no signature
(LLR-pzde8b).
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the Envelope is signed by the administrator's Member-as-a-group key
and carries the record's last sequence number plus one. Nothing signs, and the
Sequence number is the epoch the update produced (REQ-txvtm9).

**LLR-bg3vsw**: the Wire message carries the member snapshots as they were **before**
the addition, so a joiner holding no record of the Organisation can rebuild the
trie the change applies to.
satisfies: REQ-xa6smf

**LLR-jn5jeh**: in Loopback mode the joiner is dialled at the full
`EndpointAddr` the call carries, not by endpoint identity alone. Nothing checks
that the address names the joiner's device key, so an admission and its
Organisation secret go wherever the address points. That is PR-2dmjzj's
defect, stated here and not endorsed. One of its cures (dial by key in both
modes) would replace this requirement.
satisfies: REQ-ztdza4

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

**LLR-t4znbk**: the Organisation record is updated and the store saved only
after the Wire message has been sent. This is the order PR-vt244s books as a
defect, because a failed send leaves the chain one epoch ahead of the record.
It is stated here and not endorsed, and the tests that carry it pin it.
satisfies: derived

**LLR-ag9mgm**: `admit_member` draws the new member's `MemberId` from the
caller's random source before the leaf is built and never computes it from a
key, so admitting the same Member-as-a-group key and device key twice yields two
different identifiers and a deleted identifier is never reissued.
satisfies: REQ-d9g6nt

**LLR-vdyu65**: `admit_member` acts on the Organisation record that the
`org_id` argument names. A device holding more than one Organisation therefore
admits into the one the caller asked for, and leaves every other record, and
every other on-chain root, where it was.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item also said `admit_member` signs with that Organisation's administrator
Persona. Nothing is signed; the Persona it looks up names the endpoint to bind
(LLR-cns6q6, PR-8qsnhx).

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

**LLR-3v5nu9**: `admit_member` submits the update with the pure-proxy account
its Organisation record holds.
satisfies: derived

*Added 2026-10-05 by review round 8, which measured that passing none was green. It is carried by
`the_proxy_account_from_genesis_is_kept_and_passed_on_every_update`.*

### The four out-of-band functions

*Added 2026-10-04 by review round 2, which found that four of the five
functions this item's interface names — `export_invite`, `import_invite`,
`export_join_request` and `import_join_request` — were refined by no low-level
requirement at all. The encoding and decoding of the blob **types** is
SDD-vee2fq's and is refined there; what was missing is what these four
functions do with them, and one of those is a trust root.*

**LLR-zj88e6**: `export_invite` carries the device key of the administrator
persona of the Organisation it names.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item ended ", so the joiner has a key to pin the first admission's sender
against". Nothing on the joiner's side compares that key with a sender since
the owner's ruling.

**LLR-qezw3n**: `export_invite` carries, as the administrator's dialling
address, the address of the endpoint the service has bound, whichever Persona
bound it, and an empty address when none is bound. Where the endpoint was
bound from another Persona, the address names a different device from the
Invite's device key. That is PR-8qsnhx's reading side, as for LLR-437fvx, and
it is stated here and not endorsed.
satisfies: derived

*Added 2026-10-05 by review round 8, which measured that emptying the address left the gate green.
`the_out_of_band_blobs_carry_the_keys_their_holders_are_pinned_by` carries it,
for a bound endpoint and for none. Its "whichever Persona bound it" clause is
carried by `pr_8qsnhx_an_invite_advertises_the_bound_endpoint_not_its_administrators`,
written by review round 9.*

**LLR-9zfnmb**: `import_invite` records at most one pending invite per
Organisation — replacing any earlier one rather than appending — and the store
is written before it returns, so the Invite a first admission consumes is the
latest one imported and survives a restart.
satisfies: REQ-xa6smf

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

**LLR-437fvx**: `export_join_request` carries the persona's Member-as-a-group key and its
device key as two distinct keys, each derived from that persona's own seed.
The dialling address it carries is the bound endpoint's, whichever Persona
bound it, and so can name a different device. That is PR-8qsnhx's reading
side.
satisfies: derived

**LLR-836z24**: `import_join_request` decodes and returns; it stores nothing,
so importing a join request leaves the importer's record unchanged.
satisfies: derived

## SDD-8cpyfa — The receive-and-commit path

`org-node/src/service.rs` (`receive_and_verify`, `ReceiveOutcome`,
`first_admission_base` *(added 2026-10-05, see below)*)

**SDD-8cpyfa**: the member's side — accept one Wire message, verify it against
the chain, and commit. It takes the sender from the connection and checks
nothing about it, on a first admission and on every later update.
traces: REQ-xa6smf, REQ-ztdza4, REQ-nhe2zu, REQ-txvtm9, REQ-bvh8v6, REQ-d9g6nt, REQ-qn2erx

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item owned the two sender cross-checks, the only place the transport's
authenticated identity was compared with the membership record. Both are
removed (REQ-xa6smf, REQ-ztdza4). LLR-rys5nx, in
`2026-10-05-unsigned-envelope.md`, also refines this item.

*(Amended 2026-10-05 by the org-node type-safety change, review round 7:
`first_admission_base`, which only `receive_and_verify` calls and which no
item named, is this item's. It is also constrained by LLR-8bum44, under
SDD-af5vnt in `2026-10-04-type-safety.md`. A record snapshot holding a value
its type's parse refuses fails as a whole with `InvalidField` naming the
`member.…` field, and extends nothing. So the item traces REQ-qn2erx too.)*

**LLR-j6j95z**: a first admission to an Organisation the node holds no record
of is refused explicitly when the Wire message carries no snapshot of the Membership
record the change extends, rather than attempted against a reconstructed one.
satisfies: REQ-d9g6nt

**LLR-j83kc8**: on a first admission, the Wire message is committed when it
verifies against the chain whichever Device key the connection authenticated,
whether or not that key is the administrator's Device key an imported Invite
names.
satisfies: REQ-xa6smf

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item refused a first admission whose sender was not the Invite's administrator
device.

**LLR-u6rq4s**: on a Wire message about an Organisation already held, the
Device key the connection authenticated is compared with no Membership record:
an update that verifies against the chain is committed whether that key is in
the record before the update, after it, or in neither.
satisfies: REQ-ztdza4

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item required that key to be in the trie the Envelope verified into, checked
after verification.

**LLR-37cj3n**: what `receive_and_verify` commits is decided by the
Organisation state it reads from the chain itself and by its own record, never
by a key or a value carried in the Wire message, and never by the device that
delivered it.
satisfies: REQ-nhe2zu

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item named the author key the signature was verified under. There is no author
key.

**LLR-3wb7th**: an Organisation the chain holds no state for is refused with
`OrgNotOnChain` before any local record is consulted.
satisfies: REQ-bvh8v6

**LLR-cja9zv**: the record, the epoch, the sequence mark and the member
snapshots are written together after verification succeeds, and the store is
saved before the outcome is returned.
satisfies: REQ-nhe2zu, REQ-txvtm9

*Annotated 2026-10-05 by review round 8.* This holds for a change that
removes this node's own device key too: on this path that change is committed
as an update, with nothing deleted. That is PR-322qst's defect, stated here
and not endorsed, and
`pr_322qst_an_own_revocation_on_the_ordinary_path_is_committed_not_self_deleted`
pins it and carries this requirement.

**LLR-mbjfq8**: a first admission to an Organisation for which no Invite has
been imported is committed on the chain anchor alone; the missing Invite is
not a reason to refuse it.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said such an admission rests on the chain anchor and the signature alone,
and that the Invite cross-check applies when an Invite exists. There is no
signature and no cross-check.

*Added 2026-10-04 by review round 7, which measured that refusing such an
admission left the gate green. It is stated as the code behaves, and
`a_first_admission_with_no_imported_invite_rests_on_the_chain_alone` pins it.
The trade-off is assessed under RC-b6mydy in the hazard analysis and in the
derived-requirements risk file.*

*Abnormal case added 2026-10-05 by review round 4 (finding-4).* "On the chain
anchor alone" cuts both ways: such an admission that does not reach the
chain's Membership root is refused with `RootMismatch` and commits nothing,
marks no Persona and writes nothing to disk.
`a_first_admission_with_no_imported_invite_that_misses_the_chain_root_commits_nothing`
pins it.

**LLR-y2v8v2**: every record this path reads or writes is the one the received
change's Organisation identifier names — the local Membership record it
verifies against, the record it commits into and the invite it consumes —
so a node holding more than one Organisation leaves every other
Organisation's record and secret exactly as they were.
satisfies: REQ-nhe2zu, REQ-txvtm9, REQ-xa6smf

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item also listed "the pending invite it cross-checks the sender against". The
pending Invite is no longer compared with the sender; the one consumed is
still the named Organisation's.

*Narrowed 2026-10-04 by review round 5.* This requirement also claimed "the
Persona it marks Active", and that clause was false. The Persona is chosen
from every Persona whose device key is in the verified trie, whatever
Organisation it is bound to, and is then rebound. A device holding two
Organisations can have one Organisation's Persona moved to the other. That is
PR-mdv38y, pinned by
`pr_mdv38y_the_receive_path_rebinds_another_organisations_persona`. The clause
is withdrawn here rather than left standing over a test that asserts its
opposite. LLR-e5c9ud states what the selection does today.

**LLR-xq9nrq**: on a first admission for which no Invite has been imported,
the new Organisation record holds, as its `admin_member_key`, the Organisation
public key read from the chain in the same operation, as a `PersonPublicKey`,
never a value carried in the Wire message. It names no administrator: the
chain publishes none. When an Invite was imported, the field holds that
Invite's administrator Member-as-a-group key instead (LLR-rys5nx).
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

**LLR-ckk5nz**: on first admission the Organisation secret the Wire message
carries is stored in the new record. On a later update the stored secret is
overwritten with whatever the Wire message carries, including nothing. That is
PR-xwek5e, whose intended behaviour is unruled, and it is stated rather than
endorsed.
satisfies: derived

**LLR-e5c9ud**: the Persona marked Active is one whose Member-as-a-group key
is not the administrator's Member-as-a-group key the record names, and whose
DevicePublicKey is in the verified trie. It is given the member id of the
member holding that DevicePublicKey, and the received change's Organisation.
Which Organisation that Persona was bound to before is not consulted. That is
PR-mdv38y. When no Invite was imported, the key the record names is the
chain's Organisation public key (LLR-xq9nrq), which names no administrator, so
the comparison excludes only a Persona whose Member-as-a-group key equals it.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item compared the Persona's device key with the administrator's member key,
correct only while both were one ed25519 key. The code compares the two
Member-as-a-group keys. The last sentence follows the owner's answer Q1 on
where the record's administrator key comes from.

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

*Re-traced 2026-10-04 by review round 6. All three said `satisfies:
REQ-xa6smf`, which states only the first-admission sender check against the
invite. None of the three refines that check: the secret overwrite on later
updates, the Persona selection on every receive, and the administrator's Member-as-a-group key on the member's record. Tracing them there took them out of the derived-behaviour
risk assessment. They are `derived` now, and assessed in
`org-node/docs/risk/2026-10-03-architecture-derived.md`.*

*Also noted by review round 6: REQ-uxv2x2, what a node does on committing a
Change set that removes its own Device key, is not scoped to a receive
operation, and this item does not apply it. On this path the node's own
removal is committed as an ordinary update. That is PR-322qst, pinned by
`pr_322qst_an_own_revocation_on_the_ordinary_path_is_committed_not_self_deleted`.*

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

**LLR-q8emds**: a pending invite is discarded only once the first admission has
committed.
satisfies: REQ-xa6smf

## SDD-72ddm6 — Revocation and self-delete

`org-node/src/service.rs` (`revoke_member`,
`receive_and_self_delete_if_revoked`, `SelfDeleteOutcome`)

**SDD-72ddm6**: what a node does when the change it receives removes **it** —
the one path where committing a verified change means deleting the record
rather than updating it. The item also owns the administrator's side that
sends that change. Nothing about the sender is checked on this path.
traces: REQ-uxv2x2, REQ-nhe2zu, REQ-txvtm9

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item did not name the administrator's side, which its interface
(`revoke_member`) already held, and said nothing of the sender. The owner
ruled that nothing about the sender is checked on any Receive operation.

**LLR-vw2jn6**: the change is verified against the chain **before** anything is
decided about the node's own membership.
satisfies: REQ-uxv2x2

**LLR-6p4pj2**: when the node's own device key is absent from the verified
trie, the Organisation record is deleted and every persona bound to that
Organisation is marked revoked.
satisfies: REQ-uxv2x2

**LLR-jsx922**: when the device key of a Persona bound to this Organisation is
still present, the change is committed as an ordinary update and reported as
such, with nothing deleted.
satisfies: REQ-uxv2x2

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
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the path does not cross-check the sender because a node being
removed cannot be required to find the remover in a record it is no longer
part of, and booked the update branch's missing check as PR-u4c2vp. The owner
ruled that nothing about the sender is checked on either branch, so PR-u4c2vp
is resolved by that ruling.

**LLR-6qmq2g**: a change that fails verification leaves the Organisation record
in place.
satisfies: REQ-uxv2x2

**LLR-379hnv**: a change about an Organisation the node holds no record of is
refused with `OrgNotOnChain`, and nothing is written: no record, and no
pending invite consumed. The error's name is wrong, because the chain does
hold the Organisation (it was read just before). That is recorded for the fix
change and not endorsed.
satisfies: derived

*Added 2026-10-04 by review round 7, which found this refusal stated by no
requirement and declared nowhere. It is the mechanism behind the verification
record's Gap 20: the shipped app receives only on this path, so a joiner's
first admission cannot complete there.
`the_self_delete_path_refuses_an_organisation_it_holds_no_record_of` pins it.*

**LLR-jwhzh3**: this path acts only on the Organisation the received change
names — the record it verifies against, the Personas whose binding names that
Organisation (and only those) to decide whether the node is still a member,
the record it deletes or updates, and the Personas it marks Revoked — so a node
revoked from one Organisation keeps every other Organisation's record, secret
and Personas.
satisfies: REQ-uxv2x2

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

**LLR-6dc598**: `revoke_member` removes the member from the trie and submits
the new Membership root on-chain at the next epoch **before** the revocation
Envelope is sent. The anchor a receiver will check against therefore exists
before the receiver is told about it.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the signed revocation.

**LLR-tax3pm**: the revocation Envelope carries as its Sequence number the
epoch the revocation's chain update produced, which is greater than the
administrator's record's last. It carries no signature (LLR-pzde8b).
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the revocation is signed by the administrator's Member-as-a-group
key and carries the last sequence number plus one.

**LLR-qg9utu**: the administrator's Organisation record is updated and the
store written after the revocation has been sent, so the removal reaches the
disk rather than only memory. The publish-before-persist order is PR-vt244s's
defect, as for LLR-t4znbk, and it is stated here and not endorsed.
satisfies: derived

**LLR-pw369n**: in Loopback mode the Member being revoked is dialled at the full
`EndpointAddr` the call carries, and a Loopback revocation offered no address
is refused with a typed error rather than attempted against a peer it cannot
name.
satisfies: derived

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

**LLR-8hdu9x**: the Wire message `revoke_member` sends carries the revocation
Envelope, the member snapshots as they were **before** the removal, and no
Organisation secret. A receiver on `receive_and_verify` stores that absence
over the secret it held. That is PR-xwek5e, stated here and not endorsed.
satisfies: derived

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* This
item said the signed envelope.

**LLR-drgdy8**: `revoke_member` submits the update with the pure-proxy
account its Organisation record holds, as `admit_member` does (LLR-3v5nu9).
satisfies: derived

*Added 2026-10-05 by review round 8, which measured that emptying the snapshot and passing no proxy account were both
green, and that the absent secret was observed only by the PR-xwek5e pin.
LLR-8hdu9x is carried by `a_revocation_reaches_the_administrators_disk`, and
its receiver clause by the PR-xwek5e pin (review round 9), and
LLR-drgdy8 by `the_proxy_account_from_genesis_is_kept_and_passed_on_every_update`.*

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
connection, building and signing an extrinsic, submitting it, driving block
production, creating and rotating a pure proxy, running the genesis ceremony,
and the operator-facing preflight checks. It is the unit's whole supplier-facing
surface and it carries **no low-level requirements**.
traces: REQ-nhe2zu, REQ-txvtm9, REQ-bvh8v6

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
| LLR-na7p4w (a good signature verifies) | LLR-9fvb3y (another key's does not) |
| LLR-p8uu47 (build signs what it transmits) | LLR-ybn5pr, LLR-cs4mpb, LLR-9sknpa (each field's tampering is caught) |
| LLR-er2x8n (a frame within the bound round-trips) | LLR-sc6zuh, LLR-8kh3zf (over the bound, both directions) |
| LLR-fa7jt8 (the frame layout) | LLR-pkruy8 (bytes that are not a frame) |
| LLR-g9vmbx, LLR-kkj64b (blobs round-trip) | LLR-8qxwst, LLR-tcft2r (bad armour, bad encoded body) |
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

**Items whose abnormal side is carried by another item.** SDD-na9nc3's ten
LLRs are *all* refusals except LLR-8n95rf and LLR-d6kvbx; its normal case is
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
| SDD-rx2yvy | `admit_member` given a malformed member or device key (→ `OrgNodeError::Chain("bad member key")`) or a handle already held (→ `OrgNodeError::Trie`) — **the most material of the seven**, because both are reachable from a join request a stranger composes. *Amended 2026-10-05 by the org-node type-safety change: `admit_member` now takes the parsed `JoinRequest`, and a malformed key or handle in a Join request is refused at `import_join_request` with `InvalidField` naming it (LLR-8bum44, under SDD-af5vnt in `2026-10-04-type-safety.md` — *amended 2026-10-05 by the org-node type-safety change, review round 7: it read "under SDD-vee2fq"*), so the first clause cannot reach `admit_member` and the `Chain("bad member key")` refusal no longer exists. A handle already held is still untested here.* |
| SDD-ueh4tm | a `ChainOps` implementation whose `read_state` or `submit_update` fails. *Since review round 8 the item has one refusal case, the mock's wrong-epoch update (LLR-ryzr8m), but that is the mock refusing, not an implementation failing, so the line stands.* |
| SDD-z85ux9 | every abnormal case it has. The item has no gated test of any kind — that is the deviation recorded at the end of this file — so it has no abnormal one either, and listing it here is the honest bookkeeping |
| SDD-msb6xh | none — `build_update_calldata` is total over its argument types, so there is no abnormal input to offer it |
| SDD-rq6nv4 | `multi_account_id` is total over its argument types. `build_dispatch_tx` is **not**: `runtime_call_to_tx` has three `WriteError::MalformedCall` arms, none of them tested, and this change made the function publicly reachable through `org_node::test_support`, so the abnormal input is now one line of test away |

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

**One low-level requirement cannot be discharged that way, and it is named
rather than quietly counted.** LLR-6adc99 refines REQ-2wzfzv — IPv4 loopback
required, IPv6 optional — and both clauses are about a bind *failure* that no
gated test can produce. `check-trace.sh` is green over it because MISSING-TEST
is satisfied by the annotation alone, which is the third time this repository
has recorded that property, now in a second unit. It is kept by the owner
ruling that kept REQ-2wzfzv itself, and the `test-support` bind-address
constructor that would close it is booked in
`docs/plans/2026-09-05-ratchet-setup.md`.
