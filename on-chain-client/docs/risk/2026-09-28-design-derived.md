# Derived design items — risk assessment for on-chain-client's first architecture ledger

This change (`worktree-guardrails-on-chain-client-arch`) adds nine software
items and thirty-two low-level requirements to
`on-chain-client/docs/architecture/`. It mints no hazard and no risk control:
the decomposition describes code that already exists and already carries this
unit's controls, and nothing in it changes behaviour. One test is added and no
production code is written.

**One** of the thirty-two low-level requirements is marked
`satisfies: derived` — LLR-2y9qdc, which exists because of how the reader was
built, with no parent in the unit's twenty-one high-level requirements.
`check-trace.sh` reports UNANALYZED-DERIVED for a derived item this ledger never
mentions, and silence is not an assessment, so it is assessed below by name.

**This section said *two* when it was written, and review round 2 found the
second was mis-marked.** LLR-b4p32h carried `satisfies: derived` although
REQ-ntn4ss states it verbatim; the annotation is corrected in the decomposition
and the entry below is kept, rewritten as a hazard note rather than as a
derived-item assessment.

The other thirty-one refine a high-level requirement each, and those requirements
were themselves assessed in this unit's hazard analysis
(`on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md`, "Derived
requirements assessment"). Refining an assessed requirement into directly
codeable behaviour introduces no hazard the parent's assessment did not already
consider: the refinement is a statement about the same behaviour at a finer
grain, verified by the same tests. That is recorded here once rather than
repeated thirty-one times.

## The item marked derived in error, and the one derived item

**LLR-b4p32h — not a derived item; marked derived in error, and now
`satisfies: REQ-ntn4ss`.** *A head is carried as a reference holding both its
hash and its number, so that a discarded head can be named by both.* The
justification this entry originally rested on — that the requirements state
*when* a reorg is reported and not what shape a head reference has, so the shape
is design data by definition — **is false, and is withdrawn.** REQ-ntn4ss states
the shape in its own words: "report a Reorg notification, **carrying both the
hash and the number of the head that was discarded**, when and only when …". The
item refines that clause; it invents nothing the requirement does not ask for.

The assessment is kept rather than deleted, as a **hazard note on behaviour now
traced to REQ-ntn4ss** — whose own assessment sits in this unit's hazard
analysis with the other high-level requirements — and not as the assessment of a
derived item. Assessed against HAZ-xd4urb, the hazard the reorg notification
exists for: a consumer holding a membership belief taken from a block the chain
then discarded. The item is a *widening* of what a reorg notification carries,
not a narrowing — RC-kemv75's control is that a discarded head is reported, and
reporting it by hash alone would leave a consumer unable to tell which of
several observations at the same height it had acted on, while reporting it by
number alone would not identify the block at all. Carrying both is what makes
the notification actionable. **It introduces no new hazard**: the pair is
derived from data the notification already had to carry, nothing decides on it
inside this unit, and a consumer that ignores either field is in exactly the
position it would have been in had the field never been carried.

**LLR-2y9qdc** — *an increment that carries out of the most significant byte
wraps to zero rather than panicking or saturating.* This is the unit's one
derived low-level requirement, it can bear on safety, and it is assessed against
HAZ-v2cmtx and its control RC-5ejucb. HAZ-v2cmtx is the hazard that what this
reader reads is derived offline from conventions the crate does not own — among
them Solidity's mapping-slot formula with consecutive struct slots, which is
this item's own arithmetic.

Three behaviours were available at the top of the slot-key space: panic,
saturate, or wrap. **Panicking is refused** because it would make a reachable
input abort the reader, which REQ-sx5b6g forbids for the decoders and which
would be no better here — and because `on-chain-client/Cargo.toml` denies
`clippy::panic`, `unwrap_used` and `expect_used` crate-wide, so the crate's
stated posture is that a bad input is an error value, never an abort.
**Saturating is refused** because it is the genuinely dangerous option: two
distinct base slot keys near the top of the space would saturate to the *same*
key, and the reader would return one organisation's field as another's —
HAZ-v2cmtx's harm exactly, reached without an adversary. Wrapping preserves
injectivity, which is the property the derivation needs.

**Residual risk: acceptable, and the argument does not rest on the arithmetic.**
The base Slot key is the keccak-256 of the Organisation admin and the map slot
index, so reaching a base key within three of `2^256 - 1` requires finding a
keccak preimage in a window of three out of `2^256` — not a bound the reader
enforces, but a work factor no participant has. Wrapping is therefore
unreachable in operation, and the behaviour matters as a *choice against
saturation* rather than as a path anyone will take. The gated test
`an_all_ones_slot_wraps_to_zero` pins it so the choice cannot be silently
reversed by a later edit, which is the whole reason it is a low-level
requirement and not a comment.

It has no parent requirement because REQ-xudf25 states the carry propagation
"through every byte it reaches" and stops there: what happens when the carry
reaches past the last byte is beyond the requirement's words. Rather than widen
the requirement after the fact, the behaviour is recorded as derived and
assessed here.

assesses: LLR-2y9qdc

## The deviation this change records, and why it is not a new hazard

SDD-3b8zef — the chain-facing transport shell, the majority of `client.rs`, of
which llvm-cov analyses 336 lines out of the 491 it analyses across this crate,
at 16.37% line coverage — carries no low-level requirements, because nothing at
this unit's gate can verify one. (The hand-computed "roughly 580 of 699" that
stood here was dropped 2026-09-28 by this change's review sweep as
unreproducible; the decomposition file states why.) That is recorded in full in
the architecture ledger, and as an owner item in
`docs/plans/2026-09-05-ratchet-setup.md`.

It is named here because a reader of the risk ledger should not have to learn
it elsewhere, but it is **not** a new hazard and mints no control. The code it
covers is unchanged by this change, the hazards it can realise are the ones
this unit's register already analysed on 2026-09-10, and **two of the four
problem reports open against this unit are already in exactly that code**:
PR-w5sk5k, the decoder that `client.rs`'s `from_client` pins once and never
re-checks, and PR-uq5r97, the seed `client.rs`'s `subscribe` takes for the best
lane's first backfill span. The other two open reports are elsewhere and are not
this item's: PR-qpp28h is the reorg predicate in `internals::scan_step`, which
is **SDD-m59zrg**, an item fully refined into low-level requirements and gated;
and PR-h4mb8y is `src/verify.rs`, which this change records as **no software
item at all**. (Five reports exist in this unit's ledger; PR-p5ngya is resolved,
so four are open. This paragraph said "four of the five problem reports open",
which both mis-stated that arithmetic and over-counted how many sit in
SDD-3b8zef's code — review round 2 caught both.) What this change adds is not
a new risk but a *measurement* of an existing one: the residual that the register
recorded per-hazard is now also visible as a structural fact about the unit's
decomposition — the majority of its source has no verifiable design refinement.
Closing it is a code or gate change, and both are named in the ledger.
