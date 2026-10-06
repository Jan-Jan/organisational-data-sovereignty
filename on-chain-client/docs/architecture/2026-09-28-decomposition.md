# on-chain-client — software decomposition, items and low-level requirements

The first architecture ledger for the `on-chain-client` unit: nine software
items and the thirty-two low-level requirements that refine the twenty-one
high-level requirements in `on-chain-client/docs/requirements/`. The unit's
SOUP inventory is `on-chain-client/docs/architecture/soup.md`.

Safety class C throughout (`on-chain-client/.guardrails/config.yaml`, no
per-item override), so every item below carries LLRs rather than the class B
"optional per item" — **with one exception, SDD-3b8zef, which carries none.**
That is a deviation, not an override, and the section "The item with no
low-level requirements" at the end of this file records it in full rather than
leaving a reader to notice the absence.

## Overview

The unit's overview — the decomposition picture and the segregation position —
lives in the Overview section of
`on-chain-client/docs/architecture/README.md`, which is the standing
description every later change amends, rather than in this per-change file.

## The seam

This unit is a reader. It decides what the chain says an organisation's
membership is, and every consumer's access decision rests on that reading. The
nine items divide it along the chain of custody of a byte: what it is typed
as, how an address is derived from it, where in contract storage it lives,
which decoder is allowed to read it, what that decoder refuses, who is admitted
to see the result, how provisional observations are ordered, and what the
transport does with all of it.

Five of the items live wholly or partly in `client.rs`, and two share
`decode/v_paseo_ah.rs`. An item here is a responsibility with an interface, not
a file. `decode_org_state` and `parse_revive_event` are two interfaces that
happen to share a module; and `client.rs`'s `internals` block exists precisely
because three of its decisions were extracted so they could be tested without a
chain — that extraction is what makes SDD-bw7v5x, SDD-4z3k2u and SDD-m59zrg
verifiable items rather than parts of the transport shell.

## SDD-5wamsz — ABI value types and the observation vocabulary

`on-chain-client/src/types.rs`, `on-chain-client/src/state.rs`,
`on-chain-client/src/client.rs` (`ClientError`, `SubscribedEventStream`)

**SDD-5wamsz**: the public value types whose widths mirror the deployed
contract's ABI, together with the vocabulary in which an observation is
reported — a block reference, a decoded event, an event paired with the
contract that emitted it, the notification a subscriber receives, the typed
error a caller of the client surface is handed, and the stream those
observations arrive on.
traces: REQ-2qa5r5, REQ-54txzh, REQ-5upq6n, REQ-ntn4ss

(Note, 2026-10-05: REQ-2qa5r5 is superseded by REQ-54txzh, which names the
second field the Organisation public key. It stays in the trace above as
history, beside its replacement.)

`SubscribedEvent` distinguishes an observation taken from a best block from one
taken from a finalised block. That distinction is carried by the type but
**required by nothing** — this unit's SRS explicitly declines to state it (see
the closing section of `on-chain-client/docs/requirements/2026-09-10-chain-reading.md`,
which records that RC-kemv75's first clause was dropped when review round 3
found no requirement realised it) — so it is not refined into a low-level
requirement here. An LLR must refine a requirement; there is none to refine.

**LLR-xv7auy**: each on-chain field is a public, distinct newtype whose width is
the ABI's — `OnChainRootHash` and `OrgPubKey` thirty-two bytes, `Epoch` a
`u64`, `OrgAdmin` twenty — so that two fields of equal width cannot be
substituted for one another. satisfies: REQ-2qa5r5, REQ-54txzh

(Note, 2026-10-05: LLR-xv7auy satisfies REQ-54txzh. REQ-2qa5r5, which
REQ-54txzh supersedes, stays in the line above as history, beside its
replacement.)

**LLR-z8rrkr**: a decoded event is carried together with the twenty-byte
address of the contract that emitted it, as one value, so that the address
cannot be dropped between decoding and the admission decision that needs it.
satisfies: REQ-5upq6n

**LLR-b4p32h**: a head is carried as a reference holding both its hash and its
number, so that a discarded head can be named by both. satisfies: REQ-ntn4ss

## SDD-5b8wxs — Account-to-address derivation

`on-chain-client/src/h160.rs`

**SDD-5b8wxs**: the mapping from a thirty-two byte account identifier to the
twenty-byte Organisation admin that pallet-revive gives it, which is what
decides an organisation's slot in contract storage.
traces: REQ-tkhe3u, REQ-rz7fja

**LLR-2yhra8**: `h160_of` returns the account identifier's first twenty bytes
when all twelve marker positions hold the EVM-fallback marker byte.
satisfies: REQ-tkhe3u

**LLR-3bkhuc**: `h160_of` returns the last twenty bytes of the keccak-256 of
the whole thirty-two byte identifier whenever any one marker position does not
hold the marker byte. satisfies: REQ-rz7fja

**LLR-7pjzjn**: the marker positions are bytes twenty through thirty-one
inclusive and the marker byte is `0xEE`, so the branch is decided by those
twelve bytes and no others. satisfies: REQ-tkhe3u, REQ-rz7fja

## SDD-bw7v5x — Solidity storage-slot derivation

`on-chain-client/src/client.rs` (`internals::solidity_mapping_slot`,
`internals::increment_slot`)

**SDD-bw7v5x**: the arithmetic that turns an Organisation admin into the
storage keys its state is held at — the mapping key for the organisation's slot
and the consecutive keys of the fields within it.
traces: REQ-9m2rnd, REQ-xudf25

**LLR-vktf8w**: `solidity_mapping_slot` returns the keccak-256 of a sixty-four
byte buffer holding the Organisation admin left-padded into bytes twelve
through thirty-one and the map's slot index in the second word.
satisfies: REQ-9m2rnd

**LLR-62tqnv**: the map's slot index occupies the low eight bytes of that
second word, big-endian, so a different declared map slot yields a different
key for the same Organisation admin. satisfies: REQ-9m2rnd

**LLR-bhwsn6**: `increment_slot` adds its offset to a thirty-two byte
big-endian slot identifier, propagating carry from the least significant
byte upward. satisfies: REQ-xudf25

**LLR-2y9qdc**: an increment that carries out of the most significant byte
wraps to zero rather than panicking or saturating. satisfies: derived

**LLR-v62yjq**: the base Slot key and the two keys above it, at which an
Organisation slot's three fields live, are consecutive and pairwise distinct.
satisfies: REQ-9m2rnd, REQ-xudf25

## SDD-v2rtka — Runtime-version decoder selection and the decode error vocabulary

`on-chain-client/src/decode/mod.rs`, `on-chain-client/src/decode/dispatch.rs`

**SDD-v2rtka**: the `Decoder` interface every version-pinned decoder
implements, the typed error vocabulary they all report through, and the single
place a Runtime spec version is resolved to one of them.
traces: REQ-hd6m9d

**LLR-b3s7st**: `for_runtime` resolves a Runtime spec version equal to the
pinned one to the decoder compiled for that version, and to no other.
satisfies: REQ-hd6m9d

**LLR-u8ajby**: `for_runtime` returns `UnsupportedRuntime` naming the version it
was asked about for every other Runtime spec version, including the ones
immediately below and above the pinned one and the extremes of the range —
never a nearest match and never a default. satisfies: REQ-hd6m9d

## SDD-d5jh6t — The event log decoder

`on-chain-client/src/decode/v_paseo_ah.rs` (`parse_revive_event`,
`parse_genesis`, `parse_root_updated`, `unpack_address_topic`, the two
signature constants)

**SDD-d5jh6t**: the decoder that turns the SCALE payload of a
`pallet_revive::Event::ContractEmitted` into a typed `OrgRegistry` event paired
with its Emitting contract, and refuses everything that is not one.
traces: REQ-52uc8f, REQ-5upq6n, REQ-wnjz9j, REQ-n6v896, REQ-88fp2h, REQ-twdu84, REQ-axcxf7, REQ-sx5b6g, REQ-9wwenn

**LLR-e6skvu**: exactly two Event signature values are recognised, each the
keccak-256 of one of the two canonical Solidity signature strings the deployed
contract declares. satisfies: REQ-52uc8f

**LLR-u2e389**: the Emitting contract is read from the `ContractEmitted`
payload and returned to the caller alongside the decoded event rather than
dropped. satisfies: REQ-5upq6n

**LLR-rjcqg3**: a log whose first topic matches neither recognised signature,
and a log carrying no topics at all, yield no event and no error.
satisfies: REQ-wnjz9j

**LLR-n6gghu**: `GenesisInitialized` requires exactly two topics and sixty-four
data bytes and `RootUpdated` exactly three topics and ninety-six; any other
count or length is refused with a typed error naming the event, the expected
value and the actual. satisfies: REQ-88fp2h

**LLR-89pdz9**: an indexed address topic is refused when any of its twelve
leading pad bytes is non-zero, and yields the topic's low twenty bytes when all
twelve are zero. satisfies: REQ-twdu84

**LLR-8242kq**: a payload carrying bytes beyond the three fields it declares is
refused naming the trailing count, and one ending inside a field or inside a
field's own length prefix is refused. satisfies: REQ-axcxf7

**LLR-6tjhgk**: a structurally valid event decodes back to every field it
carried — both event shapes, the Emitting contract included — without loss,
reordering or defaulting. satisfies: REQ-n6v896

**LLR-2v5u4d**: no sequence of bytes offered to this decoder causes a panic or
an abort; every rejection is a typed error. satisfies: REQ-sx5b6g

**LLR-mzh8df**: `RootUpdated`'s indexed Epoch topic is bounded to the range
this reader represents, so an on-chain Epoch above it is refused rather than
truncated to it. satisfies: REQ-9wwenn

## SDD-f2s7bx — The Organisation state decoder

`on-chain-client/src/decode/v_paseo_ah.rs` (`decode_org_state`,
`decode_uint256_to_u64`)

**SDD-f2s7bx**: the decoder that turns the three concatenated storage slots of
an Organisation slot into an Organisation state, and refuses any other width or
any Epoch outside the range this reader represents.
traces: REQ-4astjb, REQ-9wwenn, REQ-sx5b6g

**LLR-sq76u3**: `decode_org_state` accepts exactly ninety-six bytes and refuses
every other length — shorter, longer, empty, or a whole number of slots other
than three — with a typed error naming both the expected and the actual length.
satisfies: REQ-4astjb

**LLR-nq7nhg**: the ninety-six bytes are read as the Membership root in the
first slot, the Organisation's signing key in the second and the Epoch in the
third, each at its own thirty-two byte offset. satisfies: REQ-4astjb
superseded-by: LLR-hezpr7

**LLR-emp3g9**: the Epoch slot is a big-endian `uint256` whose high twenty-four
bytes must all be zero; a non-zero byte anywhere in them is refused, never
truncated to the low eight. satisfies: REQ-9wwenn

**LLR-yhw34z**: no sequence of bytes offered to this decoder causes a panic or
an abort; every rejection is a typed error. satisfies: REQ-sx5b6g

## SDD-4z3k2u — Subscriber admission

`on-chain-client/src/client.rs` (`internals::log_is_ours`, `event_admin`)

**SDD-4z3k2u**: the decision whether a decoded log belongs to the organisation
this reader watches — the contract that emitted it, and the Organisation admin
a subscription filters on. traces: REQ-9vwcwc, REQ-nygs7k

**LLR-2znra8**: a decoded log is refused unless the contract that emitted it is
the one the reader was constructed for, so that a matching Organisation admin
never rescues a foreign log. satisfies: REQ-9vwcwc

**LLR-kfr75c**: with no admin filter set every log from the configured contract
is admitted; with one set, only a log whose Organisation admin equals the named
one is. satisfies: REQ-9vwcwc, REQ-nygs7k

**LLR-9qp3k7**: the Organisation admin compared against the filter is read from
both event shapes alike. satisfies: REQ-nygs7k

## SDD-m59zrg — The best-head scan and reorg rule

`on-chain-client/src/client.rs` (`internals::scan_step`, `internals::ScanStep`)

**SDD-m59zrg**: the per-notification decision of the best lane — whether a head
has already been seen, whether the previous head was discarded, and which
heights must be read to cover the span since the last one processed.
traces: REQ-ntn4ss, REQ-gr2ver, REQ-5zux82

**LLR-d3ef7s**: a notification whose hash equals the last processed head's
yields nothing and leaves the last processed head unchanged, whatever number it
carries. satisfies: REQ-gr2ver

**LLR-56bzcj**: a reorg is reported when a head has already been processed, the
new head's hash differs from that head's, and the new head is either at or
below its height or at the next height with a parent other than it — and not
otherwise, the first notification included. satisfies: REQ-ntn4ss

**LLR-48ygak**: the heights read for a new head are the one after the last
processed head up to and including the new head's, or the new head's height
alone when it is at or below the last processed head's or none has been
processed, and the span is always ascending. satisfies: REQ-5zux82

## SDD-3b8zef — The chain-facing transport shell

`on-chain-client/src/client.rs` (`OrgRegistryClient::from_client`,
`spec_version`, `get_org_state`, `read_contract_slot`, `subscribe`,
`best_lane`, `finalised_lane`, `events_in_best_block`,
`decode_contract_events`)

**SDD-3b8zef**: the subxt-backed shell that pins a decoder at construction,
reads an Organisation slot's three storage keys at one resolved block, and
drives the two subscription lanes that deliver observations to a subscriber.
traces: REQ-9vwcwc, REQ-5zux82, REQ-hd6m9d

This item carries **no low-level requirements**. See the section below; the
absence is a recorded deviation, not an omission and not a per-item class
override.

## The item with no low-level requirements

**The obligation.** IEC 62304 class C requires low-level requirements per
software item, verified at that item's own interface. Eight of the nine items
above carry them; SDD-3b8zef carries none.

That sentence is true on the reading a reader checks by counting — every item
has at least one low-level requirement — and review round 4 was right that the
reading is weaker than the obligation the same sentence states. On the stricter
one, that **every part of an item's declared interface is refined**, **three of
the eight are partial**, and none of the three is glossed.

* **SDD-v2rtka** declares the `Decoder` interface and the typed error vocabulary
  alongside version resolution, and both its low-level requirements refine
  version resolution only.
* **SDD-5wamsz** owns `ClientError`, whose `Display` and `From<DecodeError>` are
  gate-verifiable and unwritten. Writing one is not as cheap as it looks: no
  requirement in this unit's SRS covers either, so the LLR would have to be
  `satisfies: derived` and then be assessed in the risk file, or a requirement
  would have to be minted first. That is stated here because this same file
  refuses an LLR for `SubscribedEvent`'s best/finalised distinction on the
  ground that "an LLR must refine a requirement; there is none to refine", and
  the two cases deserve the same answer rather than different ones.
* **SDD-5wamsz also owns `SubscribedEventStream`, and that gap *is* of
  SDD-3b8zef's kind.** Review round 5 found this and it is the more serious of
  the two. The type (`client.rs:100`) is produced by nothing but
  `subscribe` — `client.rs:287`, `:343`, `:432` — which is SDD-3b8zef. **No
  gated test can construct one**; the only tests that name it are three of the
  five ungated targets. So when this change moved `ClientError` and
  `SubscribedEventStream` together out of SDD-3b8zef and into SDD-5wamsz, the
  argument for the move was made entirely about `ClientError` — chain-free,
  re-exported under a default-on feature, reachable from the gated tests today —
  and `SubscribedEventStream` travelled with it on no argument at all. It shares
  none of those properties. **The relocation narrowed the deviation's stated
  scope without narrowing the unreachable surface**, and that is recorded here
  rather than left for the next reader to discover.

So two of the three partial items are writable requirements nobody has written.
The third is a behaviour no gate can reach, which is exactly SDD-3b8zef's
problem wearing another item's name. Calling all eight settled would overstate
them, and the earlier claim that "neither gap is of SDD-3b8zef's kind" was wrong
about one of them.

**The state.** SDD-3b8zef is the async subxt transport and the **majority of
`client.rs`**, which is itself the majority of this unit's source by volume. The
measured figure is llvm-cov's: `client.rs` is **336 of the 491 lines llvm-cov
analyses** across this crate, at **16.37%** line coverage.

The five integration targets that exercise `OrgRegistryClient` at all —
`off_chain_genesis_ceremony`, `p_address_is_orgid`, `reorg_cancels_proposed`,
`scenario_a_full` and `two_orgs_one_watcher` — each need a chopsticks fork,
and **none of them runs at any gate.** The other four ungated
targets under `on-chain-client/tests` do not touch this item:
`00_chopsticks_sanity` and `01_multisig_sanity` are harness sanity checks that
import nothing from `on_chain_client`; `smoldot_smoke` touches only
`decode::dispatch::for_runtime`, which is SDD-v2rtka, whose version-resolution
interface is fully gated, and needs live Paseo rather than a chopsticks fork;
and `regenerate_corpus` is an `#[ignore]`d corpus writer that constructs no
client. An earlier draft of this paragraph named all nine as exercising the
item; that was wrong, and the narrower five is what was measured.

*The hand-computed "roughly 580 of `client.rs`'s 699 lines" was dropped on
2026-09-28 by this change's review sweep and is not re-derived.* It was
subtracted by hand — 699 minus `internals` minus `event_admin` — **before**
`ClientError` and `SubscribedEventStream` were moved out of this item, so it
overstated the item even on its own method; two independent recounts afterwards
gave ~548 and ~550 and neither reproduces the other. A number no reader can
reproduce is worse than a measured one, so what replaces it is the figure
llvm-cov prints plus a plain qualitative statement, and the same substitution
was made at every other site that carried the arithmetic.

What remains in the item after the extraction described below is async subxt
transport code, and nothing this unit's gate can run reaches it.

**The scope of the deviation is narrower than first written.** What is undone
here is the **async subxt transport**, not every symbol once listed under this
item. `ClientError` is a chain-free public type — three variants, a hand-written
`Display`, and `From<DecodeError>` — re-exported at the crate root under
`feature = "client"`, which this unit's gate command enables by default, so it
is reachable from `on-chain-client/tests/*.rs` today with no chopsticks and no
async runtime. Leaving it inside the item whose whole claim is "nothing at this
gate reaches it" made that claim false; it and `SubscribedEventStream` now sit
under SDD-5wamsz, which already owns the value types and the observation
vocabulary. A low-level requirement for `ClientError`'s `Display` and its
`From<DecodeError>` conversion is writable and verifiable at this unit's gate on
the owner item's own terms: `ClientError` is a chain-free public type, so what
it renders and what it converts from can both be asserted by a test that
constructs no client and needs no async runtime. **No such low-level requirement
exists anywhere in this ledger to follow** — SDD-v2rtka's two are both about
`for_runtime`'s resolution, and no LLR here mentions `Display` at all. An
earlier draft of this paragraph cited SDD-v2rtka as already carrying one for
`DecodeError`; it does not, and the comparison is withdrawn. It is **not
written here**: minting it needs a new gated cargo test target, and this tooth
writes no new cargo targets, so it is filed as an owner item instead, alongside
the open SDD-3b8zef entry in `docs/plans/2026-09-05-ratchet-setup.md`. This was
found by the independent review of this change, not by its author.

**What was already extracted, and why that is the precedent rather than the
excuse.** Three decisions that once lived inside this shell are now items of
their own, testable without a chain: the admission decision (SDD-4z3k2u), the
best-lane step (SDD-m59zrg), and the slot arithmetic (SDD-bw7v5x). That
extraction was made because the comparison in `log_is_ours` "was unreachable
from every gated test, which is HAZ-werm85's residual and was measured, not
assumed". The same reasoning applies to what is left; it has simply not been
done yet.

**Why the obvious alternative was refused.** LLRs could be written for this
item and annotated onto any of the nine ungated targets — not merely the five
that reach the item, because the gate does not care which code a file exercises
— and `check-trace.sh` would report a clean tree.

It would do so because MISSING-TEST is satisfied by a `verifies:` reference
**in a file under `test_paths`**, and nothing more. The independent review
verified this against `check-trace.sh` rather than taking the ledger's word for
it, and found the hazard is wider than stated here first: the resolution is a
plain text scan (`ids_matching 'verifies:' LLR $test_paths`) that consults no
cargo target, no `required-features`, no `#[ignore]`, no `verify_commands` and
no run record. So the annotation need not sit in a chopsticks target, or in a
test target at all — **any** text file under `on-chain-client/tests` satisfies
it, a README or a helper in `common/` included, and because the scan passes
`--untracked`, so does a file that was never committed.

The same scan discharges *requirement* coverage transitively: a tested LLR's
`satisfies:` list is folded into the covered set (`check-trace.sh:506-514`). So
a low-level requirement annotated onto a file that runs nowhere would silently
satisfy MISSING-TEST for its parent requirements as well as for itself. The
false green does not stop at the item — it propagates up into the requirements
ledger.

The count of ungated targets is nine; the count that exercise this item is
five. Both numbers appear above and they answer different questions — how many
places an annotation could hide, and how much of the item anything reaches at
all. Neither is the other. It would be a green gate standing over evidence that
was never produced, which is the defect this repository recorded against itself
on 2026-09-18 — a class C hazard register arguing from CI that has never
executed. Repeating it here, in the change that documents the architecture,
would be worse than recording the gap. The choice was put to the owner on
2026-09-28 and this is the decision taken.

**A third option, which this argument did not weigh.** Review round 4 observed
that the choice above is posed as a dichotomy — mint the low-level requirements
and annotate them onto ungated targets, or mint none — and that a third exists:
**mint them and leave them unannotated.** `check-trace.sh` then reports
`MISSING-TEST` for each and exits 1 (`check-trace.sh:501-503`), so the gap is
raised by a script rather than by prose.

**Round 5 found that option weaker than round 4 stated, and the reason is the
one already given above.** MISSING-TEST is discharged by a `verifies:` reference
in *any* text file under `test_paths` — committed or not, in a test target or
not. So the red that option three raises is **one line away from a green**, and
the person who types that line converts a recorded class C deviation into an
ordinary green gate that no longer reads as a deviation at all. Option three
does not make the gap durable. It makes it durable until the next person wants a
green build, and it arms them with a one-line erasure that leaves no argument
behind. Recording the deviation in prose is harder to erase precisely because
erasing it requires deleting a paragraph that says why it is there.

**What this section concedes, plainly.** Round 4 also observed that "it makes
this unit's gate red on a tree where nothing is broken" — the first reason given
for declining option three — switches criteria at the convenient moment. It
does. Under the criterion this section argues from, a class C item missing its
required low-level requirements **is** a defect, and a red gate would be
accurate rather than spurious. That reason is withdrawn. So is the claim that a
repository red for recorded reasons teaches its readers to stop reading red,
which is an assertion about people that this change measured nothing about and
which would equally forbid ever reddening a gate for any recorded defect.

What remains is the erasability argument above, and it is sufficient on its own.

**Nothing is filed for this, and that is stated rather than implied.** An
earlier draft of this passage said the option was "filed as an owner item rather
than settled by the author". It was not filed, and declining it *is* settling
it. The register entry now exists — `docs/plans/2026-09-05-ratchet-setup.md`,
under the SDD-3b8zef gap — and it records the option, the erasability objection,
and the fact that the author of this change declined it. A reader who thinks the
judgement is wrong should be able to find the judgement, not a claim that
somebody else will make it.

**The deviation is enforced by nothing, and the hole is bigger than the class.**
`check-trace.sh` never reads `safety_class`; the only gate that reads it is
`check-units.sh`, and only to rank classes across a dependency edge — its class
floor, D5. But putting it that way understates it, as review round 5 pointed
out. **No gate anywhere associates a low-level requirement with a software item
at all.** `check-trace.sh`'s model has exactly two edges — `SDD traces: REQ`
(UNTRACED-DESIGN) and `LLR satisfies: REQ` — and nothing between SDD and LLR.
Which item an LLR belongs to is a heading in this file and nothing a script can
read. So no gate of any class *could* require an item to carry low-level
requirements, whatever its `safety_class` said. This unit's gate exits 0 today
at `SDD 9, LLR 32` and will go on exiting 0 for as long as SDD-3b8zef carries
none. The deviation is recorded in three prose locations and enforced in zero. A
reader who wants to know whether it has been closed has to read this section; no
run will ever tell them.

The same is true one level up, and it is worth stating because it is easy to
assume otherwise: **`verify_commands` is validated but never executed.**
`gr_check_flat` does read it — it refuses the key written as a scalar,
duplicated, or empty, and names it in the emptiness diagnostic — so "nothing
reads it" would be wrong; the comment at `lib.sh:1086` saying so is scoped to
one absent-key rule, not global. What is true is that no script in
`.guardrails/scripts` ever *runs* its contents. The commands are run by whoever
follows the merge process. So there is no automated cross-check anywhere tying
an annotation to a target that actually ran — not merely none inside
`check-trace.sh`.

**What closes it.** Either the extraction continues until the remaining
decisions in the shell are chain-free and verifiable at this unit's gate, or
the gate gains a harness that can run the chopsticks targets. Both are code or
infrastructure changes owed their own red-first cycle, and neither belongs in a
documentation tooth. The item is filed in
`docs/plans/2026-09-05-ratchet-setup.md` alongside the coverage shortfall it is
the other face of — 16.37% of `client.rs` uncovered and SDD-3b8zef having no
verifiable LLR are the same gap measured two ways, and the register says so.

## What is not an item

`on-chain-client/src/verify.rs` is declared `pub mod verify;` from `lib.rs` and
is seven lines of doc-comment with no code. It gets no software item, because a
design item no requirement needs is YAGNI: no requirement in this unit's SRS
asks for verification against a candidate trie, and the module exports nothing
for an item to describe an interface of.

It is not passed over in silence either. **PR-h4mb8y** records it — open since
2026-09-10 — and states the defect more precisely than a design item could: the
module's own header claims in the present tense to be the verifier that closes
the loop with `org_members::CandidateTrie::verify_against`, so the unit named
for verification performs none, and a consumer written from the module's
description calls a function that does not exist. This change deliberately
leaves the code alone; removing or filling a public module is owed its own
red-first change.

`on-chain-client/src/lib.rs` is named by no software item either, and review
round 4 was right that its omission was not stated. It is forty-six lines and
carries no *behaviour* of its own: the crate doc comment, `#![cfg_attr]`, the
module declarations, the public re-export surface (`:30-35`), and the
feature gates that decide whether `client.rs` compiles at all. Those gates are a
segregation decision, and naming no item for the file is not a claim that it
decides nothing. Every behaviour
it exposes belongs to the item that defines it, which is why no item claims it
and why a tenth item for it would be a file masquerading as a responsibility.

One part of it is load-bearing for this whole change and deserves naming
rather than passing over. `pub mod test_support` (`:41-46`), compiled only
under `test-support` **and** `client`, re-exports exactly five symbols —
`ScanStep`, `increment_slot`, `log_is_ours`, `scan_step`,
`solidity_mapping_slot`. That module is the **only** route by which this unit's
gated tests reach `internals`, so it is the mechanism that makes SDD-bw7v5x,
SDD-4z3k2u and SDD-m59zrg verifiable items instead of parts of the unreachable
shell. It is a test seam, not a software item: it adds no behaviour, and
everything it exports is refined by a low-level requirement under the item that
owns it. But if it were deleted, three items would lose all of their evidence
at once, which is worth a reader knowing.

## Robustness, and why fourteen low-level requirements are one-sided

Class C asks for both a normal-case and an abnormal-input test per item. The
split is not measured by any script in `.guardrails/scripts` — nothing there
reads robustness sidedness. It is a **hand count made by the verification
subagent while running the gate**, which found **eighteen of the thirty-two
carry both sides and fourteen carry one**. That count is reported here rather
than left for a reader to rediscover, and the fourteen are argued rather than
closed by inventing cases — the same course this unit's requirements ledger
took for REQ-2qa5r5 and REQ-n6v896.
*(Note 2026-10-05, review round 3, finding-13: REQ-2qa5r5 is superseded by
REQ-54txzh, which carries the same exemption in
`../requirements/2026-10-05-organisation-public-key.md`.)*

**Twelve of the fourteen have their other side inside the same software
item, carried by a sibling LLR.** That is an artifact of how these LLRs are
written, not a hole in the evidence: where a behaviour has an accept path and
a refuse path, this ledger states them as *two* low-level requirements rather
than one, because each is separately codeable and separately mutable. Splitting
them makes each LLR one-sided by construction while leaving the item's coverage
exactly as complete.

| One-sided LLR | Its other side |
|---|---|
| LLR-b3s7st (resolves the pinned version) | LLR-u8ajby (refuses every other) — and the reverse |
| LLR-nq7nhg (decodes the three fields) | LLR-sq76u3 (refuses every other length) — and the reverse |
| LLR-hezpr7 (decodes the three fields, the second as the Organisation public key; supersedes LLR-nq7nhg) | LLR-sq76u3 (refuses every other length) — and the reverse |
| LLR-9qp3k7 (the admin is read from both shapes) | LLR-kfr75c (a non-matching admin is refused) |
| LLR-u2e389 (the Emitting contract is returned) | LLR-8242kq, LLR-n6gghu (malformed payloads refused) |
| LLR-rjcqg3 (unknown topic yields nothing) | LLR-6tjhgk (a known one round-trips every field) |
| LLR-n6gghu, LLR-8242kq (shape refusals) | LLR-6tjhgk (the well-formed case) |
| LLR-mzh8df (an Epoch above range is refused) | LLR-6tjhgk's `root_updated_round_trips_every_field` (an in-range Epoch decodes) |
| LLR-2y9qdc (the wrap at the top of the space) | LLR-bhwsn6 (ordinary carry, four cases) |
| LLR-v62yjq (three consecutive distinct keys) | LLR-2y9qdc (the boundary of the same arithmetic) |

(Amended 2026-10-05: LLR-nq7nhg is superseded by LLR-hezpr7
(`2026-10-05-organisation-public-key.md`), which restates the same decoding
with the second field named the Organisation public key. LLR-hezpr7 takes
LLR-nq7nhg's pairing with LLR-sq76u3; the LLR-nq7nhg row is kept as the record
of the count above.)

**The other two have no sibling at all, and their exemptions differ from each
other.**

**LLR-b4p32h has no other side, and the pairing this table used to offer was
wrong.** The row read *"LLR-b4p32h (a head is named by hash and number) | every
passing reorg case under LLR-56bzcj carries the same reference"*, and it failed
twice. LLR-b4p32h is defined under **SDD-5wamsz** and LLR-56bzcj under
**SDD-m59zrg** — different software items, so LLR-56bzcj is not a sibling and
the "inside the same software item" claim above never covered this row; that is
why the count is twelve and not thirteen. And the reorg cases the row pointed at
are further *normal* cases, not the abnormal side: they show the reference being
carried correctly, which is the same side LLR-b4p32h already has. Stated
honestly, **LLR-b4p32h's other side does not exist.** What it states is a
constraint on a *declaration* — that a head travels as a reference holding both
its hash and its number — and the abnormal input would be a head reference
missing one of the two, which the type makes unrepresentable. There is nothing
to malform, so there is no abnormal-input case to write, and this is recorded
rather than paired. The row was tenable only while LLR-b4p32h carried
`satisfies: derived`; when review round 2 found REQ-ntn4ss states the item
verbatim and the annotation moved to `satisfies: REQ-ntn4ss`, it became an
ordinary one-sided LLR whose sidedness this table had to answer, and the answer
it gave was a cross-item pairing. Corrected 2026-09-28 by this change's review
sweep.

**LLR-xv7auy is the other, and its exemption rests on the same ground, argued
here in more detail because its evidence is more directly mutable.** It states
that each on-chain field is a newtype of the ABI's width. That is a constraint
on a *declaration*, not on a behaviour: it has no runtime input domain, so there
is no input to malform. A mutation of it is a mutation of the type, which is
exactly how its evidence reds — `#[repr(align(32))]`
on `OrgAdmin` widens `size_of` to 32 and fails the width assertion at
`type_widths.rs:143`, and the same mutation with `align(64)` on
`OnChainRootHash` and on `OrgPubKey`, and `align(16)` on `Epoch`, reds the same
test at `:166`, `:167` and `:175`. All four were re-run in the 2026-09-28
falsifiability sweep. This is the same argument, on the same grounds, that this
unit's requirements ledger records for REQ-2qa5r5, the high-level requirement
LLR-xv7auy refines.
*(Note 2026-10-05, review round 3, finding-13: LLR-xv7auy also satisfies
REQ-54txzh, which supersedes REQ-2qa5r5 and carries the same argument.)*

The same holds for its **distinctness** clause, which the falsifiability sweep
cut on 2026-09-28 and review round 5 restored on 2026-09-29 after measuring a
probe that reds it. Distinctness is likewise a constraint on declarations with
no runtime input domain, and its evidence is likewise a mutation of the types:
retyping the `org_pub_key` field from `OrgPubKey` to `OnChainRootHash` — the
substitution the clause forbids — leaves the library compiling and reds five
gated targets. Both halves of this LLR are exempt from the robustness rule on
the same ground, and both are evidence-bearing.

**What this does not claim.** It does not claim the unit is exhaustively tested.
The robustness rule is met at the level of the software item, which is the level
IEC 62304 states it at — every one of the eight items carrying LLRs has both
normal-case and abnormal-input evidence — and it is *not* met per-LLR for
fourteen of them, which is stated plainly above rather than argued away. A
reader who wants the per-LLR rule enforced literally will find fourteen places
to add a test, and this section is the list.

## Evidence

Every LLR above is carried by a test in `on-chain-client/tests` that runs at
this unit's gate — one of the nine targets declared with
`required-features = ["test-support"]` in `on-chain-client/Cargo.toml` and named
in `verify_commands`, or one of the three bolero targets. Thirty-one of the
thirty-two were carried by tests that already existed; the annotation is what
this change added.

**A test that is already green has never been watched failing.** So each LLR is
discharged by red by mutation: a named mutation to the source, a named test
failing for that reason, the mutation reverted, and the file proved
byte-identical afterwards by SHA-256. Those attestations are in this change's
verification record, not here.

The one new test is
`root_updated_epoch_above_u64_is_refused_not_truncated`, which carries
LLR-mzh8df. The bound it exercises is shared by both decoders, and until this
change nothing at the gate reached it through the event path — only through
storage. No production code was written for it; the behaviour was already
there.

## The falsifiability sweep of 2026-09-28

**Why there is a sweep at all.** Three independent review rounds had each found
one low-level requirement asserting a property **no gated test can falsify**,
and every one of them was found by mutating the source, never by reading the
requirement against the code. Reading found none of them. So rather than wait
for a fourth round to find a sixth, every clause of every low-level requirement
in this file was put through the same measurement at once.

**The method, so a later reader can repeat it.** Each LLR's text is decomposed
into *independently falsifiable claims*: a conjunct, a "so that" purpose clause,
an "and not otherwise", an ordering or short-circuit claim, a named error
variant, a named boundary — each is its own claim. For each claim:

1. Write the smallest source mutation that makes **exactly that claim** false
   while leaving the rest of the LLR true.
2. Apply it and run this unit's whole `verify_commands` — not just the annotated
   target, so collateral is visible.
3. Revert, and prove the file byte-identical with `shasum -a 256`.
4. Classify: **RED** if a named test fails — at runtime, or by failing to
   compile while the library itself still compiles, because for a claim about a
   declaration the compiler is the verifier and it is this unit's gate that
   reports it. **GREEN** if nothing fails: the clause is unfalsifiable at this
   gate, and it is cut. A mutation that breaks the **library's** own
   compilation measures nothing — no test runs, so nothing is watched failing —
   and must be rewritten rather than recorded.

*Rule corrected 2026-09-29 by review round 4.* It first read that a compile
error was "not a red, and not a green either", and filed two claims
UNTESTABLE-BY-MUTATION under it. That conflated a broken library with a failing
gate. Both claims were re-measured under the corrected rule and both are RED;
the outcome table and the section below record the measurements.

**What was measured.** The thirty-two low-level requirements decompose into **one
hundred** independently falsifiable claims, and all one hundred are now probed by
their own mutation. The first sweep recorded **103 runs over 102 distinct
mutations** — `2v5u4d-panic` was run twice, and the commit message quoted the
distinct count as the run count. Review round 4 added two further mutations over
three further runs, one discarded for breaking the library's compilation and
rewritten. Every run restored all five mutable source files from pristine copies
afterwards, and the five digests in this change's plan hold unchanged across all
of them, re-verified again after round 4's mutations.

| Outcome | Claims |
|---|---|
| **RED (runtime)** — a named test runs and fails | 96 |
| **RED (compile-time)** — a named test target fails to compile while the library still compiles | 3 |
| **GREEN** — no mutation anyone has written makes a test fail; clause cut | 1 |

**The tie-break the rule was missing, added 2026-09-29 by review round 5.**
Falsifiability is **existential**: a clause is evidence-bearing if *some*
mutation that makes it false reds a named test. One green falsifier proves
nothing about the clause; it proves that one mutation was a poor probe. So
**GREEN is a verdict about the search, not about the clause**, and it may only
be recorded after the obvious falsifiers have been tried. This was already the
reasoning applied to LLR-e6skvu below — an arbitrary third signature is green, a
drifted-ABI third signature reds, and the clause is kept — and round 5 found the
sweep had not applied it consistently. It cost a clause its life, as the next
paragraph records.

**The clause cut in error, restored 2026-09-28.** LLR-xv7auy's distinctness
clause — "so that two fields of equal width cannot be substituted for one
another" — was cut on one green mutation. It is **RED**, and back in the
requirement. The sweep's probe replaced `OrgPubKey`'s declaration in
`src/types.rs` with `pub use crate::types::OnChainRootHash as OrgPubKey;`, which
leaves the gate green at 73 passed — but that probe also deletes the *name*, so
the widths assertion it was being judged by can no longer run against it. A
probe that performs the forbidden substitution **and leaves the test able to
observe it** does exist:

> retype the `org_pub_key` field from `OrgPubKey` to `OnChainRootHash` at its
> three sites in `src/state.rs` and its three construction sites in
> `src/decode/v_paseo_ah.rs`, touching neither `src/types.rs` nor `src/lib.rs`.

Measured: the library builds clean (`cargo build --lib --features test-support`,
rc 0, no warnings — publicness, newtype-ness and all four widths stand
untouched), and the gate exits **101** with **eight `E0308`** across five
targets — `contract_address_filter`, `decode_org_state`, `decode_revive_event`,
`runtime_version_dispatch`, `fuzz_event_round_trip`. The two thirty-two byte
fields are not interchangeable, and this unit's gate is what says so.

**The one clause still cut.**

* **LLR-mzh8df** claimed the event path's Epoch bound is "the same
  `uint256`-to-`u64` rule as the storage path". That is a claim about *code
  sharing*, not about behaviour. Replacing the call to `decode_uint256_to_u64`
  in `parse_root_updated` with a byte-for-byte identical check written inline
  leaves the gate at **73 passed, 0 failed** — no test can see whether the two
  paths share a function. Round 5 tried the stronger probe as well, inlining
  byte-equivalent code at *both* call sites so that no function is shared by the
  two paths at all: still **73 passed, 0 failed**. `decode_uint256_to_u64` is a
  private `fn` named by no test target in code, so no compile-time carrier
  exists either, and none can be manufactured without changing behaviour — which
  is this LLR's *other*, already-red clause. The cut stands under the corrected
  rule and the tie-break. The behavioural half reds: substituting
  a direct low-eight-byte read fails
  `root_updated_epoch_above_u64_is_refused_not_truncated`, so "refused rather
  than truncated" stays. (The Evidence section's remark that the bound "is
  shared by both decoders" is a true statement about the code and is kept as
  such; it is no longer a claim any requirement here makes.)

**The two claims once filed UNTESTABLE-BY-MUTATION are both RED.** Review round
4 re-measured them and the original classification was wrong in both cases, for
the same reason: the sweep tried one mutation route per claim, found it broke
the *library's* compilation, and generalised from that to "untestable". A second
route exists for each, and both red this unit's gate.

* **LLR-xv7auy — "public".** The sweep dropped `pub` from `OrgAdmin` alone,
  which reds the library, and stopped. Making the type crate-visible
  *consistently* does not: `pub(crate) struct OrgAdmin(pub [u8; 20]);` in
  `src/types.rs:19`, with `OrgAdmin` taken out of `lib.rs:32`'s `pub use` list
  and re-exported beside it as `pub(crate) use crate::types::OrgAdmin;` — the
  other three types stay public — leaves the library compiling with warnings
  only. The gate then exits **101**, with `E0603: struct OrgAdmin is private` in
  **six** integration targets: `contract_address_filter`, `storage_slot_layout`,
  `decode_revive_event`, `log_ownership`, `fuzz_event_round_trip`, and,
  decisively, **`type_widths`** — the very target annotated
  `verifies: REQ-2qa5r5, LLR-xv7auy`. The claim is evidence-bearing at this
  unit's own gate, through its own carrier.
  *(Note 2026-10-05, review round 3, finding-13: the annotation now reads
  `verifies: REQ-2qa5r5, REQ-54txzh, LLR-xv7auy`; REQ-54txzh supersedes
  REQ-2qa5r5, which the test still verifies under its earlier name.)*

* **LLR-z8rrkr — "as one value", and the mutation round 4 offered for it was
  the wrong one.** Round 4 re-typed `parse_revive_event` to return
  `Result<Option<([u8; 20], Event)>, DecodeError>` and called that two values.
  Review round 5 pointed out that **a 2-tuple is one value**, so that mutation
  does not make the clause false at all; what it falsifies is the tests'
  dependence on the struct's name and field names. It reds the gate, but it reds
  it for the wrong reason, and a red for the wrong reason is not evidence.

  The mutation that does falsify the clause returns the address **out of band**:
  `fn parse_revive_event(&self, event_bytes: &[u8], contract_out: &mut [u8; 20])
  -> Result<Option<Event>, DecodeError>` at `decode/mod.rs:56` and
  `decode/v_paseo_ah.rs:88`, writing `*contract_out = contract;` before
  `Ok(Some(event))` at `v_paseo_ah.rs:119`, with `client.rs` re-assembling the
  struct at the call site so the library's own consumers are unaffected. Now the
  address really is carried separately from the event — exactly what the clause
  forbids. Measured: the library builds clean (rc 0, no errors) and the gate
  exits **101** with `E0308` in five targets — `contract_address_filter`,
  `decode_revive_event`, `fuzz_event_round_trip`, `fuzz_parse_revive_event` and
  `log_ownership`. `internals::log_is_ours` is untouched, so what reds is the
  one-value-ness of the decoder's result and nothing else. **RED**, on this
  mutation rather than round 4's.

**What a compile-time red does and does not establish.** It establishes that
the declaration is *referenced* in the shape the clause states, by targets this
unit's gate compiles — no more. It does not exercise a behaviour, and no
assertion runs. That is weaker than a runtime red and is recorded as its own
row in the table above rather than blended into it. Three of the ninety-nine
reds are of this kind: LLR-xv7auy's "public", LLR-xv7auy's restored
distinctness clause, and LLR-z8rrkr's "as one value". The
alternative — treating declaration-surface properties as unverifiable and
striking them from the requirements — was considered and refused, because the
properties are real, safety-relevant and enforced; but a reader weighing this
ledger should know which three rows rest on the compiler rather than on a test
body.

**One claim reds only against the neighbourhood the suite samples, and that is
recorded rather than cut.** LLR-e6skvu says "exactly two Event signature values
are recognised". Adding a *third* recognised signature at an arbitrary constant
(`[0xAA; 32]`) leaves the gate green at 73 passed: "no third value" cannot be
swept over 2^256. Adding a third at the keccak of a plausibly drifted ABI string
— `GenesisInitialized(address,bytes32)`, one parameter dropped — reds
`the_recognised_signature_hashes_are_the_keccak_of_the_canonical_solidity_strings`.
So the clause is falsifiable where drift actually occurs and unfalsifiable
elsewhere, which is the sampling limit
`on-chain-client/tests/decode_revive_event.rs` already states in its own header.
It is kept on that basis.

**What this section claims, and what it does not.** After the two cuts, the text
of every low-level requirement above is exactly what this unit's gated suite can
red on — one named mutation, one named failing test, per remaining clause — with
the two exceptions named above, both marked rather than assumed. It does **not**
claim the unit is exhaustively tested, and it does not touch the shortfalls this
file already records: SDD-3b8zef's absent LLRs and `client.rs`'s 16.37% line
coverage. No production code was changed by the sweep; its outputs are this
section, the two cuts, and the findings rows in
`docs/plans/2026-09-28-on-chain-client-architecture.md`.
