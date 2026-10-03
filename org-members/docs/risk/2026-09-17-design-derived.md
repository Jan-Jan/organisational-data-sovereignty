# Derived low-level requirement assessments — org-members architecture

The `org-members` architecture ledger written in this change defines thirty-five
low-level requirements. Twelve of them refine no high-level requirement and are
marked `satisfies: derived`: they came from the design, not from the requirements
ledger. ISO 14971 and IEC 62304 both require a derived requirement to be assessed
for hazard impact rather than merely noted, and `check-trace.sh` enforces the
mention (`UNANALYZED-DERIVED`). This file is that assessment.

It also records three prose notes this change owes its reader, and the measured
negatives the change produced while putting the unit's tests behind its LLRs.

## What this file is, and what it is not

It is an impact assessment of twelve derived design requirements against the
hazard register already in this ledger
(`org-members/docs/risk/2026-09-02-membership-hazards.md`) and the derived
requirement assessments that precede it
(`org-members/docs/risk/2026-08-31-membership-derived.md`).

It is **not** a hazard analysis. **No hazard, risk control or problem report is
minted here**, and none of the assessments below assigns a severity or a
probability. Two of them raise questions that only a hazard analysis can answer
— whether the register needs a hazard it does not hold, and whether an existing
probability estimate still reads correctly — and both are written up under
"Findings for the owner" as work for `analyze-risks`, not settled in passing at
the end of an architecture change.

Where an assessment reaches "no hazard impact", it says why. Silence is not an
assessment; neither is a bare "no impact".

The two harms remain the ones the register keeps: a member wrongly denied access
cannot reach organisational data when a decision needs it, and a member wrongly
granted or retaining access acts on data they should not hold.

## Derived requirements assessment

### LLR-w5nkbu — the 128-byte bounds on `name` and `surname`

Affects an existing hazardous situation; introduces no new hazard; changes no
control's stated effectiveness.

The bound is resource protection on the one part of a member record whose size an
outside party chooses. That is HAZ-8suua9's territory — malformed, hostile or
oversized input to a membership operation — and it is the same kind of rule as
RC-3ppkf6, which bounds the device key set. RC-3ppkf6 as worded covers device
keys only, so the personal-field bound is an **uncredited contributor** to the
same hazard rather than part of a minted control. That is a wording observation,
not a gap to fix here: the requirement it refines conceptually, REQ-h5ret5's
length clause, was already assessed as "resource protection, with no direct
hazard path".

The rejection form matters as much as the bound. `FieldTooLong { field, max }` is
a typed error naming which field and which limit, which is an instance of
RC-c4truv (every rejection reports a typed error and no input panics) rather than
a separate control; LLR-sa3ugj carries the rendering.

No effect on any integrity or misdirection pathway: `name` and `surname` are
display fields, no lookup resolves on them, and neither takes part in a
uniqueness or confusability decision. See LLR-g6arcs for why that last point is
worth stating rather than assuming.

One second-order effect, recorded and not minted: a member whose real name
exceeds 128 bytes after NFC normalisation cannot be recorded under it. That is
the same shape of administrative obstruction the register already assessed for
RC-n2taat's single-script rule (S1/P2, acceptable, no control needed), and 128
bytes is a far looser bar than a single Unicode script. It is named here so that
a later analysis finds it already considered.

### LLR-paxj7b — `MemberLeaf::new` rejects a member with no device key

Affects an existing hazardous situation; introduces no new hazard; changes no
control's effectiveness.

Zero devices is the **isolated** state. RC-sq3yhp's isolation removes every
device key while retaining the member, and HAZ-sc7wse is the hazard that control
creates. If the admission path could also produce a zero-device member, the
isolated state would stop being distinguishable from a member admitted without
devices, and the reversibility half of RC-sq3yhp — restore access by adding a
device key — would be reached by members for whom nothing was ever revoked. The
constructor's rule is what keeps "zero devices" a state the software enters
deliberately, through one operation, rather than one it can fall into.

The asymmetry with the wire path is deliberate and already recorded. Decoding
accepts the empty device list (LLR-xyv6p9, and REQ-shk82j's own text), so the
one-device minimum is a property of the direct API rather than of the membership
record. RC-4apk6w's residual risk statement says exactly this — "it accepts an
empty device key set where admission requires at least one". LLR-paxj7b therefore
does not weaken RC-4apk6w; it names the boundary the control already declares.

### LLR-4czn8t — the `Debug` rendering redacts handle, name and surname

**This is a confidentiality control in the code that no hazard in the register
names.** It is recorded as a finding for the owner rather than assessed away.

What the code does: `MemberLeaf`'s `Debug` prints `[REDACTED]` for the handle,
the name and the surname, rendering only the identifier, the member-as-a-group
key and the device key set (`org-members/src/types.rs`). `OrgTrie`'s `Debug`
prints member count, root hash and calculation state, and no handle, so the
crate's own diagnostic surface does not re-expose through the container what the
leaf redacts.

What the register holds: eight hazards — misdirection by name, revocation that
appears to succeed, isolation applied to the wrong member, a stale or divergent
record taken as current, a member introduced through the wire, unavailability
under hostile input, the record altered in place, and a superseded change
becoming applicable again. **Every disclosure pathway among them runs through
access to organisational material**, reached by a wrongly granted or retained
membership. None of them is disclosure of the membership record's own personal
fields — a handle, a name, a surname — through diagnostic output: a log line, a
panic message, a crash dump, an error report carrying a formatted record.

That pathway is not excluded from the analysis either. The register's scope
section excludes document content protection, transport and the application
surface; diagnostics are in none of those. So this is a hole in the hazard
enumeration, not an area deliberately left out.

Assessment: LLR-4czn8t **mitigates a hazard the register does not hold**. It maps
to no HAZ item, and it is not correct to attach it to the nearest one: HAZ-jkc6tj
is misdirection by name, which is about two handles a person cannot tell apart,
not about a handle reaching a reader who should not see it. The mismatch is
recorded below as a finding.

Note that the register's own "limitation" section — a hazard can only be recorded
once it is already controlled and tested — is **not** what blocks minting here.
This one is controlled (the `Debug` implementation) and tested
(`member_leaf_debug_redacts_pii`, annotated in T3). What blocks it is that hazard
identification is the owner's activity under `analyze-risks`, with a severity, a
probability and a matrix evaluation, and an architecture change is not where that
gets decided.

### LLR-72p8bz — four separated hash domains

Changes no control's wording; supplies an unstated premise two controls rest on;
introduces no new hazard.

The plan asked this to be assessed against HAZ-bmv7cy, a member introduced
through the wire. The honest answer is that the relation is indirect. HAZ-bmv7cy's
pathway is a serialised record the decoder accepts, and RC-4apk6w — re-validate
handles and device sets on decode — is the control on it. Nothing in that path
depends on hashing.

Where domain separation is load-bearing is one step further on. RC-9z65hw applies
a change set only if its declared base matches, and makes the result usable only
if its root matches the root expected; RC-ty8qdw refuses to report a root for a
record whose hashes are stale. Both controls decide by **comparing root values**,
and both therefore carry an unstated premise: that equal roots mean equal
membership. Four separated domains are part of what makes that true — a value
hashed as a device leaf cannot be read as a member leaf or as an interior node,
so a crafted structure cannot commit to a membership the organisation never
decided while presenting the root the organisation did decide. Without the
separation, a cross-domain collision would let a candidate verify against an
expected root without being the record that root was computed from, and the wire
path's final bar — root verification, which is the bar RC-4apk6w explicitly does
not claim to be — would pass on the wrong record.

So: no change to what RC-9z65hw or RC-ty8qdw say, or to how effective they are
today. The assessment records that they have a premise which was, until this
ledger, written nowhere, and LLR-72p8bz is now where it is written.

Evidence note: this LLR is one of the two carried by a genuinely new red-first
test (`hasher_domains_are_separated`, T4), which reds by giving all four hasher
methods the same domain context.

### LLR-zbe553 — MSB-first bit addressing of the member identifier

No hazard impact, because the convention is internal, compiled in, and not
negotiated with any counterparty at run time.

Worth stating the reasoning rather than the conclusion, because the obvious
objection is right in principle. The property that matters is **consistency**,
not the choice of direction: any total order over the 256 bits addresses every
member uniquely. Two peers computing roots under different conventions would
disagree on the root of the same member set, which is divergence — HAZ-y8h835's
hazardous situation, reached with no adversary. But that state is unreachable
within a build: the convention is a constant of the code, no protocol negotiates
it, and no requirement in this unit governs compatibility between versions of
this crate.

What follows for a later change: altering this convention is a **root-format
change**, invalidating every previously committed root, and it must be handled as
a migration rather than as a refactor. That is a statement about change control,
not a hazard in the current design.

### LLR-wm5hpc — per-level empty-subtree hashes; an emptied record returns to the empty root

Affects an existing hazardous situation — HAZ-h58jn6 — and does not change the
effectiveness of the control on it. Introduces no new hazard. Raises one question
for the owner.

The property is history independence: absence is represented by the same value
whether a member was never present or was removed, so a record's root is a
function of its member set and nothing else. That is the same property LLR-4n8zqx
states at the whole-record level, and every root comparison in the system needs
it.

It also means, directly and by construction, that **a root can return to a value
it previously held** — which is HAZ-h58jn6's precondition. This is not a defect
of the LLR and not an argument for changing it: a record that could not return to
an earlier root would be one whose root depended on history, and then two
administrators reaching the same membership by different routes would disagree,
which is HAZ-y8h835 with no adversary. The register already places the control
against replay correctly: RC-9z65hw's base matching "does not close the pathway,
because the pathway is precisely the case where the record state returns", and
what closes it is the monotonic envelope sequence number in `org-node`, outside
this unit.

So RC-9z65hw's effectiveness is unchanged by this assessment. What the assessment
adds is that root recurrence is **structural rather than accidental**, and
HAZ-h58jn6's P1 estimate is currently argued from the difficulty of the
surrounding steps — an administrator republishing an earlier root, plus a
retained copy of the superseded change set. Whether that estimate still reads
correctly once recurrence is known to be a design property rather than an unlucky
coincidence is a question for `analyze-risks`. It is raised below as a finding;
it is not re-rated here.

### LLR-4n8zqx — the root is determined by the member set alone

Affects existing hazardous situations HAZ-y8h835 and HAZ-h58jn6; RC-9z65hw's
effectiveness **depends** on it; introduces no new hazard.

This is the premise every root comparison in the register rests on, stated
plainly for the first time. RC-9z65hw compares a declared base root against the
record it is applied to, and the recomputed result against an expected root.
RC-ty8qdw withholds a root until it has been recomputed. Both are decisions made
by comparing 32 bytes, and both are only as meaningful as the claim that two
records with equal roots hold equal membership and two records with different
membership have different roots.

Order independence is the half that matters operationally. Two administrators
applying the same set of admissions in different orders must converge on the same
root; if they did not, HAZ-y8h835 would fire with no adversary, no stale device
and no network partition — merely two people working at once, which is the
situation the hazard's P2 estimate is built on.

Evidence note, and it is a good one: a **stronger carrier for this LLR exists and
is not used**. `org-members/tests/mbt_conformance.rs` implements a root-hash
equality-class check — equal model state implies equal real root, and distinct
model states never share a root — which is direct evidence for exactly this
property, in a form nothing else in the suite provides. T8 deliberately did not
annotate it, because its brief capped that test at four named candidates and a
fifth unmeasured annotation would be unearned evidence. Nothing is owed: the LLR
is carried and proven by `same_members_same_root_hash` and
`different_insertion_order_same_root` (T5). This is a stronger carrier going
unused, not a gap, and it is the best candidate for a measured follow-up in a
later change.

### LLR-8jttpb — the diff walk descends left before right

The plan asked whether RC-9z65hw's effectiveness depends on the traversal order
this LLR fixes. **It does not**, and saying so precisely is more useful than
claiming it does.

RC-9z65hw decides by root: base root against the record, recomputed root against
the expected root. A change set carrying the same removals and upserts in a
different order, applied to a matching base, produces the same resulting root and
passes the same checks. Traversal order is invisible to both halves of that
control.

What LLR-8jttpb buys, together with LLR-xmpqn2, is **canonical structure**:
between two roots there is one change-set *value* `apply_delta` will accept.
Canonicality (LLR-xmpqn2) is a check applied to **received** change sets, and
LLR-8jttpb is why every change set this crate **produces** already satisfies it.
An honest producer therefore never trips an honest receiver's canonical-form
check **specifically** — `validate_canonical_delta` returns `Ok` for it.

**Narrowed 2026-09-17 (round-2 independent review), because the unnarrowed
sentence is false on the reading any reader takes.** It used to stop at "an
honest producer never trips an honest receiver's canonical-form check", and it
went on to say that a violation "would show up as a lawful change refused, which
is an availability pathway of the HAZ-8suua9 kind". A lawful change **is**
refused, it does not come through HAZ-8suua9, and both halves needed correcting:

- The refusal is real and is **PR-vf5hdm** (opened 2026-09-17, open) in
  `../problems/2026-09-17-review2-fixes.md`. A
  handle moving between two members — a swap in either identifier order, or a
  one-way handover where the taker's identifier sorts before the giver's — is
  produced successfully by two sequential `update_handle` calls, emitted as a
  canonical two-leaf change set by `calculate_delta`, and then refused with
  `DuplicateHandle` by a receiver holding the byte-identical base trie.
- The refusal is **not** in the canonical-form check. `validate_canonical_delta`
  passes the delta. It is the handle-uniqueness block a few lines later in
  `apply_delta`, which walks the upserts in a single ascending pass and releases
  a leaf's old handle only when it reaches that leaf, so a handle in flight is
  still indexed when its acquirer is checked. The narrow sentence about the
  canonical-form check survives; the conclusion drawn from it does not.
- **It is not HAZ-8suua9.** That hazard is malformed, hostile or oversized
  input. Here there is no adversary and no malformed input at any step. Nor is it
  HAZ-y8h835, whose causes are all lag-shaped and whose acceptability rests on
  recoverability through the chain anchor — this divergence is not recoverable:
  the receiver has the change, refuses it, and will refuse the identical bytes
  forever. See the finding recorded under "No hazard covers PR-vf5hdm" below.

Nothing about RC-9z65hw changes. The control decides by root and is unaffected;
what is affected is the claim that the producer/receiver pair never disagrees.

**Corrected 2026-09-17, and this correction is the most consequential thing on
this page.** Until the independent review at merge, this assessment said the two
items buy **encoding uniqueness** — "one byte string per transition between two
roots" — and then named the properties that consume it: "the signature over the
encoded envelope, deduplication, and the monotonic sequence guard — and all
three live in `org-node`, whose own register holds them". That **exported a
property this unit does not have** to a unit that would have built on it. A
dedup-by-encoded-bytes or a signed-envelope replay guard in `org-node` resting
on that sentence would rest on nothing.

What is actually true:

- Canonical form constrains the **decoded `Delta` value** — ordering, presence,
  disjointness, no-op rejection — so the set of accepted change sets is
  restricted, and every change set this crate produces is already canonical.
- It does **not** make the encoding injective. `MemberLeaf`'s `Deserialize` impl
  **normalises rather than rejects**: `to_nfc` over `name` and `surname`, and
  the NFC form `validate_handle` returns for the handle. An NFD-encoded leaf and
  its NFC equivalent are two distinct postcard byte strings that decode to one
  `MemberLeaf`, give one `Delta` value and produce one root. Several wire forms
  map to one value; that is exactly what injectivity forbids. (`P2pDeviceSlots`
  is the contrasting case — its `Deserialize` genuinely refuses non-canonical
  slot vectors.)

**What `org-node` must therefore do.** Any property keyed on the bytes has to be
established by `org-node` itself and cannot be imported from here:

- **Deduplication** must key on the decoded `Delta` (or on its `base_root` and
  the resulting root), not on the encoded bytes — two distinct byte strings can
  be the same change.
- **A replay guard** must turn on the envelope's monotonic sequence number and
  the base root, as `RC-9z65hw`'s residual already says; it must not treat "we
  have seen these bytes" as "we have seen this change".
- **A signature over the encoded envelope** still authenticates *those bytes*,
  which is what a signature is for, and remains sound — but the signed bytes are
  not a canonical identifier for the transition, so a signature must not be used
  as a change identity.

No code changed to close this. Making the encoding injective is a behaviour
change needing its own red-first test, and the owner's decision on 2026-09-17
was that correcting the claim is this change's fix and the code change gets its
own change. Recorded so a later reader sees a decision rather than an omission.

On HAZ-h58jn6 specifically: the earlier text argued from encoding uniqueness
that a retained superseded change set is byte-identical to the one that would be
recomputed for the same transition. That argument is withdrawn — it may or may
not be byte-identical, depending on how the fields were encoded. The conclusion
is unchanged and does not depend on it: replay turns on the base root recurring,
not on the bytes matching or differing.

No new hazard. No control's effectiveness changes. The register's position on
RC-9z65hw is unaffected by this item, and that is the finding.

Evidence note, carried from T5 and T8, because it bears on how much this item is
actually worth: both carriers the plan named for it are **blind** to the order it
fixes. Reversing the traversal leaves them green. The mutation is caught by
`apply_delta_rejects_unsorted_removed` and by
`fuzz_tests::delta_canonicality_fuzz`, both of which now carry the annotation. See
the measured-negatives section.

### LLR-ub6dw9 — the handle and skeleton indexes, updated by every operation

The effectiveness of **RC-n2taat** rests on this item, and that is the single
most consequential of the twelve assessments.

RC-n2taat requires handles to form a canonical, unambiguous namespace: unique
within the organisation, and rejected when the UTS#39 confusable skeleton
collides with one already held. Both decisions are made by lookup in
`handle_index` and `skeleton_index` (`org-members/src/trie.rs`), not by walking
the membership store. An operation that fails to insert into either index leaves
the control **silently ineffective**: the check still runs, still passes, and
admits the duplicate or the confusable handle. That is HAZ-jkc6tj's hazardous
situation — two members bearing handles a person cannot tell apart — reached
without any failure in the validation logic itself.

The failure is directional, and both directions matter:

- A missed **insert** weakens RC-n2taat: a later admission that should collide
  does not. Wrong-grant path, mediated by an administrator reading a name.
- A missed **removal** denies a lawful re-use: LLR-j4d38d requires a departed
  member's handle to become available again, and a stale index entry refuses the
  successor. Administrative obstruction rather than injury; the register assesses
  that class of harm at S1.

One structural observation, recorded for the owner rather than assessed here:
**the indexes are derived state that no hash commits to.** They are ordinary
in-memory maps, cloned forward by each operation, rebuilt from the member set
only at `genesis`. The committed root covers the member records; it does not
cover the indexes. So an index that disagreed with the store would not be
detectable by comparing roots — the mechanism RC-9z65hw and RC-ty8qdw rely on for
everything else — and the only thing keeping them in agreement is that every
mutation path updates both. This change does not treat that as a defect: no
disagreement exists today, and the invariant is carried by the tests annotated in
T6a. It is the kind of thing a later hazard analysis should know.

Evidence note: the plan's named mutation for this LLR reddened **neither** of its
carriers, and T6a's supplementary mutation reddened both. See the measured
negatives.

### LLR-k89ahd — `rotate_p2p_key` replaces the key and changes nothing else

Affects an existing hazardous situation; introduces no new hazard; implements no
control.

Two distinct points, and the register would be misread if only the first were
recorded.

First, what this operation is **not**. `rotate_p2p_key` replaces the
member-as-a-group key without touching the device key set
(`org-members/src/trie.rs`). It is not a revocation and it does not implement
RC-mqtks7, which requires removal of a device key and replacement of the member
key to be one operation that cannot be performed by halves — that is
`delete_p2p_device`, carried by LLR-s97ywt, and `emergency_isolate_member`,
carried by LLR-w92psx. Rotating alone leaves every enrolled device still
enrolled, so a device an administrator meant to cut off simply learns the
replacement key through the key-distribution layer. An integration that treated a
rotation as a response to a compromised device would be in HAZ-s39gbh's
situation — a revocation that appears to succeed — by a route RC-mqtks7 does not
cover, because no removal was requested. LLR-k89ahd does not create that
possibility; it makes the operation's exact scope a stated requirement rather
than something inferred from the code.

Second, the "changes no other field, the device set included" clause is itself
safety-relevant, in the opposite direction. If rotation also cleared the device
set, a routine key change would silently isolate a member — every device key
gone, access lost at once, with no administrator intending an isolation. That is
HAZ-sc7wse's harm reached through an operation that is not the isolation
operation, and so outside the bounding half of RC-sq3yhp that makes an isolation
visible and reversible. The clause bounds the blast radius of a routine
operation, which is why it is worth being an LLR at all.

### LLR-g6arcs — `update_name_surname` normalises and applies the field bounds

No hazard impact on the naming pathway, and the reason is worth stating because
the opposite conclusion is tempting.

`name` and `surname` are display fields. No lookup resolves on them, they take
part in no uniqueness or confusability decision, and two members may lawfully
hold identical name/surname pairs — RC-n2taat canonicalises the **handle**
namespace and says nothing about personal names, so confusable characters in them
are unrestricted by design.

That is not an oversight to fix here, but it does sharpen where HAZ-jkc6tj's
residual risk actually sits. The register already says the last step of that
pathway is a person reading a name, that both controls narrow the namespace and
neither makes a name-based instruction unambiguous to a human, and that the
remaining control is the administrator's surface showing the stable identifier
(not-minted control 7). If that surface displays `name` and `surname` rather than
the handle, it is displaying the fields this crate does **not** canonicalise. The
assessment adds that qualification to the existing residual rather than raising a
new hazard, because the control in question is already identified and already
outside this codebase.

The two clauses themselves: NFC normalisation gives one byte sequence per
rendering, which is consistency rather than a safety property here; the bounds are
LLR-w5nkbu's, assessed above under HAZ-8suua9.

### LLR-h7stq2 — `calculate_delta` transforms `old` into the receiver, and is refused on uncomputed hashes

Affects HAZ-y8h835; supports RC-ty8qdw and RC-9z65hw at the producing end;
introduces no new hazard.

The second clause is the **producer-side counterpart of RC-ty8qdw**. That control
refuses to report a root for a record whose hashes have not been recomputed,
precisely so that a stale value cannot attest to a superseded membership. A
change set computed from such a record would carry a base root that is stale or
absent, and would then be anchored to a record state that no longer holds —
delivering the same defect through a different door. Refusing the computation is
what keeps RC-9z65hw's base-root match meaningful before the change set is ever
sent.

The first clause — that the change set actually transforms `old` into the
receiver — is functional correctness. A wrong change set would either fail the
receiver's checks, which is an availability outcome, or, if it did not, install a
membership neither party intended, which is HAZ-y8h835's harm. The integrity
weight there is carried by RC-9z65hw's expected-root verification on the
receiving side, not by this item: the receiver does not trust the producer's
arithmetic, it recomputes and compares. LLR-h7stq2 is therefore a correctness
requirement whose failure is caught by an existing control, and it does not change
that control's effectiveness.

## Findings for the owner — for `analyze-risks`, not minted here

1. **The register holds no hazard for disclosure of personal data through
   diagnostic output.** LLR-4czn8t redacts the handle, name and surname from a
   member record's `Debug` rendering; the code does it, a test proves it, and no
   HAZ item describes the situation it prevents. Every disclosure pathway in the
   register runs through access to organisational material. Diagnostics fall in
   none of the register's excluded areas, so this is an enumeration gap rather
   than a scoping decision. A hazard analysis is what settles whether it becomes a
   hazard with a severity, a probability and a matrix evaluation — and if it does,
   LLR-4czn8t is an existing, tested control ready to be named by it. **No HAZ and
   no RC are minted in this change.**

2. **HAZ-h58jn6's P1 estimate deserves a second reading.** LLR-wm5hpc makes root
   recurrence a structural property of the store — an emptied record returns to
   the empty root, and any return to an earlier member set returns to its root —
   rather than an unlucky coincidence. The other steps of that pathway (an
   administrator republishing an earlier root; a retained superseded change set)
   are unchanged, so the estimate may well stand. Re-rating it is not this
   change's business.

3. **The uniqueness and confusability indexes are outside the committed root.**
   RC-n2taat executes in state (`handle_index`, `skeleton_index`) that no hash
   covers, so a disagreement between an index and the membership store is not
   detectable by the root comparison every other control in the register relies
   on. Nothing is wrong today and nothing is minted; a later analysis should know
   where that control physically runs.

4. **RC-n2taat's homograph half is exactly as current as `unicode-security`
   0.1.2.** T1's SOUP inventory names this as its weakest row: the confusable
   skeleton test is that crate's table, and the table ages with every Unicode
   revision. The register already records that "the skeleton test is a heuristic
   over a versioned Unicode data set"; what the inventory adds is the version
   number and the fact that nothing watches it.

5. **No advisory database backs the SOUP risk assessment.** Neither `cargo-audit`
   nor `cargo-deny` is installed in this toolchain and the `Makefile` has no audit
   target, so every SOUP row's risk column is written from the code's actual use
   of the crate rather than from an advisory feed. T1 recorded this in `soup.md`
   and it is repeated here because it is a limitation of this unit's risk
   evidence, not merely of its inventory.

6. **No hazard covers PR-vf5hdm: the receiver refusing a well-formed honest
   change.** Added 2026-09-17 by the round-2 independent review. `apply_delta`
   refuses a lawful handle handover between two members — a swap in either
   identifier order, a one-way handover whenever the taker's identifier sorts
   before the giver's — so a membership change the producer made successfully
   cannot be replicated, and the two records stay apart. The mechanism, the two
   failing shapes and the reason the producer's every step is lawful are in
   `../problems/2026-09-17-review2-fixes.md`.

   The register has no pathway of this shape:

   - **HAZ-8suua9** is malformed, hostile or oversized input, answered by panic-
     freedom and the field bounds. Here the input is what this crate's own
     `calculate_delta` emitted from its own `update_handle` calls. There is no
     adversary and nothing malformed at any step.
   - **HAZ-y8h835** is staleness and divergence, and it is the nearest miss. Its
     causes are all lag-shaped — concurrent administrators, a delta applied to
     the **wrong** base, a root published before recomputation, a device offline
     — and this delta is applied to the **right** base and refused anyway. Its
     probability and acceptability argument turns on recoverability ("the chain
     anchor is what makes it recoverable"), and that argument does not hold here:
     the receiver has already received the change and will refuse the identical
     bytes on every retry. A hazard whose acceptance rests on an argument that
     fails for a pathway does not cover that pathway.

   The harm is the register's own — a member wrongly denied access, or holding a
   handle the rest of the organisation no longer associates with them, on a
   replica that believes it is current. What is new is the **cause**: a correct
   component refusing a correct input, with no adversary and no fault anywhere
   else. That is the pathway the enumeration does not contain, and it is what
   `analyze-risks` has to rate. **No HAZ and no RC is minted here** — minting
   either owes a severity, a probability and an acceptability evaluation against
   the matrix, which is the interview's work and not a fix round's.

## Three notes this change owes its reader

### LLR-pys2ek discharges REQ-xdx2c2's forward reference

`org-members/docs/requirements/2026-08-31-org-membership.md` records, under
REQ-xdx2c2, that the device bound of 4 "belongs in a low-level requirement under
the software item that owns the sub-trie, which does not exist yet", and that
until it is written the number "lives only in places no gate reads". The item now
exists: LLR-pys2ek, under SDD-4yr9ge, states that a member holds at most
`MAX_DEVICES` device keys and that `MAX_DEVICES` is 4 because the device sub-trie
is a fixed depth-2 binary tree of four slots. It is carried by three annotated
tests and was reddened by raising the constant (T3).

The requirement's own text and its note are **not edited**. This project records
corrections by date rather than by rewrite, and the discharge is recorded here, in
this change's own risk draft, which is where a reader following the note will be
sent.

One connected point the register anticipated: its assessment of RC-3ppkf6's
introduced hazard — a member at the bound cannot enrol another device — says that
this is "the reason the number belongs in a low-level requirement where it can be
reconsidered against this hazard rather than against the sub-trie depth alone".
LLR-pys2ek is now that place. As worded it states the sub-trie justification only;
the lock-out consideration lives in the register. A later change reconsidering the
number has both halves in front of it.

### LLR-s97ywt is worded to the implemented behaviour, not to REQ-ewdg2q

REQ-ewdg2q requires the member-as-a-group key to be replaced when a device key is
removed, **and** requires a replacement key equal to the key being replaced to be
rejected. RC-mqtks7 carries both clauses. The second is not implemented — nothing
compares the replacement against the outgoing key — and that is PR-zz4exm, open
since 2026-08-31.

LLR-s97ywt therefore states what `delete_p2p_device` does: it removes the device
key, replaces the member-as-a-group key in the same operation, and isolates the
member when the device removed was the last. It does not restate the requirement.
The reason is mechanical and deliberate: an LLR restating the unimplemented clause
would demand a `verifies:` annotation naming a test that cannot pass, and the only
ways to make the gate green would be to write a test that does not test it or to
leave a MISSING-TEST standing. Either would put the gap somewhere no reader looks.
Worded as it is, the gap stays where it belongs — in an open problem report and in
the register's residual risk statement for HAZ-s39gbh, which is currently
**unacceptable**.

PR-zz4exm stays open through this change. It is the one item in the register's
not-minted list that closes an unacceptable residual risk outright, and fixing it
is a behaviour change needing its own red-first test. It is not architecture work,
and it is expected to remain the sole `UNRESOLVED-PR` at this merge, at 16 days
against the unit's 30-day limit.

(Corrected 2026-10-03: PR-zz4exm resolved in the change that amends LLR-s97ywt
and LLR-w92psx to state the refusal. The LLRs now carry the requirement's
second clause, and the tests that this section said could not exist are
`delete_p2p_device_rejects_unchanged_key` and its siblings. The claim above
that it "closes an unacceptable residual risk outright" did not hold: HAZ-s39gbh
stays not acceptable, for the wire path, PR-z463w5 and PR-fzu25w — as the
register's own correction to its not-minted item 1 records.)

### The `MAX_DEVICES` coupling — a latent defect, and one test that now detects it

T3 measured that raising `MAX_DEVICES` from 4 to 5 compiles cleanly. Read the two
functions together and that is not a curiosity:

- `types.rs::to_fixed_slots` returns an array sized by `MAX_DEVICES`.
- `device_trie.rs::compute_device_root` builds its leaves as `[_; 4]`, the
  literal.

**Two further sites restate the number and are equally uncoupled** (named
2026-09-17, independent review finding-3; the ledger had mentioned only the two
above):

- `error.rs:26` renders the `DeviceSlotsFull` variant as `"device slots full
  (max 4)"` — a literal 4 inside a format string, which no compiler and no test
  compares against `MAX_DEVICES`. Raise the constant and the crate reports a
  limit it no longer enforces, which is a *reporting* defect under REQ-ds8ryr
  rather than a root-integrity one, but it is the same uncoupling.
- `trie.rs:213`, `add_p2p_device`'s doc comment: "the member already has
  `MAX_DEVICES` (4) devices" — the parenthetical is prose and goes stale
  silently.

Neither is safety-relevant the way `compute_device_root` is: an error string and
a doc comment do not decide what is hashed. They are named because "nothing
couples the two numbers" was an undercount, and a later change that fixes the
coupling should fix all four sites, not two.

They agree today because the numbers happen to match, and nothing makes them
agree. Raise the constant and a fifth device key would never be hashed: it would
live in the member record, serialise, and come back from `p2p_devices()`, while
the membership root stayed unchanged. **A device key present in the record but
absent from the root is a member key set the committed root does not describe.**

Which hazard that falls under, if the coupling ever breaks: **HAZ-y8h835**, a
stale or divergent record taken as current. The pathway fits without stretching —
every verifier downstream decides by comparing roots, so a device invisible to the
root is invisible to all of them, and two parties would agree on a root while
disagreeing on who may act as that member. It is not a new hazard, and it is not
HAZ-bmv7cy: nothing arrives from outside, and the record was built locally by the
software's own admission path.

Nothing is defective today — the numbers agree — so **this is not a problem
report and none is minted**. It is a latent defect, and exactly one test in the
suite detects it: T4's `every_device_slot_reaches_the_root`, which varies each of
the `MAX_DEVICES` slots in turn and requires each to move the root. It is the
only test whose failure message names the defect itself (`device slot 4 does not
reach the root`, with both roots identical).

**How that guard was described, and how it actually behaved until 2026-09-17.**
This section used to call it "a latent defect held shut by exactly one test".
The independent review showed the guard was weaker than the sentence implied: it
was held shut by that test *and by a digest coincidence*. The test substituted a
single fixed key, `device_key("replacement")`, into each slot, and because
`P2pDeviceSlots` stores devices sorted, the `MAX_DEVICES` 4 → 5 mutation reds
only if that key's BLAKE3 digest happens to sort after the fourth key of the base
set. The reviewer changed that one seed string to `device_key("zzz-other-seed")`
and the mutation went **green** — the constant raised, the fifth key unhashed,
and the suite reporting 115 passed. Detection was a property of two digests, not
of the test.

**Fixed in this change, and proved.** The substitute is now the largest key of a
sorted pool of `MAX_DEVICES + 1` keys, so it lands in the last slot by
construction and the invisible substitution is reached whatever the digests are.
The `MAX_DEVICES` 4 → 5 mutation reds at `device slot 4 does not reach the root`
under three unrelated seed families (`dev-*`, `zzz-other-seed-*`, `q7-slot-seed-*`),
and the test is green under all three with the constant restored. The guard is
now one test rather than one test plus luck — which is what this section claimed
before it was true.

The three tests that also red under the constant change —
`add_p2p_device_rejects_when_full`, `member_leaf_too_many_devices`,
`deserialize_rejects_too_many_devices` — each hardcode "a fifth device must be
rejected" and fail because the *cap* moved; none of them observes that the fifth
key goes unhashed.

The coupling is deliberately **left unfixed**. A `const` assertion or a rewrite of
`compute_device_root` is a behaviour change needing its own red-first cycle, and
it does not belong in an architecture change. T4 verified `types.rs` and
`device_trie.rs` byte-identical in its commit, and T11's rewrite of the guard
changed the test only — all ten `org-members/src/*.rs` files were re-verified
byte-identical by `shasum -a 256 -c` after the mutation was reverted.

The same coupling has a second face, found in T6a. With the `MAX_DEVICES` check
removed, `add_p2p_device_rejects_when_full` takes the fifth device and then panics
inside the fixed four-slot encoding — `index out of bounds: the len is 4 but the
index is 4` at `types.rs:421` — before its own assertion runs. The mutation is
unambiguously the cause, so the attestation stands, but the test detects a removed
bound only as a crash. A crash in that position is also RC-c4truv's territory —
"no input, however malformed or hostile, causes a panic" — reachable only if the
bound check were removed, which is why it is recorded here beside the coupling
rather than raised as a defect.

## Measured negatives from T1 and T3–T8

A mutation that reds nothing is evidence about the test, not an absence of
evidence, and this change's method was to record every one. They are collected
here because they are what a reader of the ledger most needs and least expects to
find, and because several of them qualify how much the annotations above are
worth.

### From T1 — the SOUP inventory

- **No advisory database was consulted.** `cargo-audit` and `cargo-deny` are both
  absent and there is no audit target, so every risk column in `soup.md` is
  reasoned from the code's use of the crate. Finding 5 above.
- **`ed25519-dalek` is used for `VerifyingKey` point validation only** — no
  signing and no signature verification anywhere in `org-members/src`, so the
  historical dalek signing-API weaknesses sit outside the compiled paths.
- **`unicode-security` 0.1.2 is the weakest row**, and RC-n2taat's homograph half
  is that crate's table. Finding 4 above.
- **The pre-release `ed25519-dalek` 3.0.0-pre.6 is not in this unit's closure.**
  Its only parents are `iroh` and its siblings, reached from `p2panda-net` into
  `spike-p2panda` and from `iroh` as a dev-dependency of `org-node`. It is a live
  SOUP question for `org-node`'s architecture tooth, recorded so that measuring it
  here does not mean forgetting it there.
- **`org-members/Cargo.lock` was tracked and inert**, recording `blake3` 1.8.5
  while the lockfile that governs the build pins 1.8.7. Deleting it left the suite
  at the same 109 passed, which is the evidence it was never read.

### From T3 — member record and validation

- **No measured negatives in the strict sense**; eight LLRs annotated across 31
  tests. But two limits of the evidence, both recorded rather than counted as a
  pass:
- **LLR-xzqs9r's ten carriers rest on one clause.** Only the uppercase mutation
  was run against them; the other nine carriers assert disjoint clauses (empty,
  dot, script-mixing, NFC, hyphen, length) that a narrow mutation does not reach.
- **LLR-5w2jx8's three carriers red through a setup probe**, not through their
  `ConfusableHandle` assertion: `find_confusable_pair()` itself asserts over
  `handle_skeleton`, so the red is for the mutated reason but arrives as a setup
  panic, at a line no reader would expect.
- **One mutation per LLR was not enough where an LLR has several clauses.** The
  named mutation for LLR-w5nkbu reddened 2 of 7 carriers and for LLR-xyv6p9 2 of
  5. T3 ran three supplementary mutations — both length bounds to 256 and to 127,
  the device-slot comparison inverted, the empty list rejected — until every named
  carrier had been individually reddened.
- **Raising `MAX_DEVICES` to 5 compiles cleanly.** The finding above.

### From T4 — hashing and the device sub-trie

- **The plan's claim that the constant mutation leaves the suite "otherwise green"
  was false**, and T4 corrected it: the mutation reds four tests, three of which
  test the bound rather than the unhashed key.
- **The plan's test sketches did not compile.** `MemberLeaf::new` takes `&str` for
  handle, name and surname, not `String`. Recorded because a plan's code is read
  as measured when it is not.

### From T5 — the sparse Merkle store

- **LLR-tk4qxu's structural-sharing clause could not be reddened, and the item was
  wrong rather than the evidence missing.** As first written the item said a
  modification "copies only the path from the changed leaf to the root, sharing
  every other node". The nearest compilable surrogate — return the shared node
  instead of path-copying whenever it already has a hash — reddened more than
  fifty tests and **neither** of the item's two carriers;
  `delete_does_not_mutate_original` reddened only for an unrelated setup reason.
  Structural sharing is invisible at the item's interface, so the clause was
  **withdrawn from the item** and now stands as design rationale in the ledger.
  What remains is guaranteed by the type system: `Node` has no interior mutability
  beyond its write-once hash cell, and every operation takes `&self`.
- **LLR-8jttpb's two named carriers are blind to the order it fixes.** Descending
  right before left leaves `calculate_delta_returns_removed_and_upserted_leaves`
  green (one removal, one upsert — no order to observe) and
  `calculate_delta_empty_when_tries_identical` green (empty). The mutation is
  caught by two tests the plan never named: `apply_delta_rejects_unsorted_removed`
  and `fuzz_tests::delta_canonicality_fuzz`.
- **`genesis_empty_is_ok` is a weak carrier for LLR-wm5hpc**: it asserts
  `member_count() == 0` and `is_calculated()`, nothing about a hash. The tombstone
  mutation reds six tests and that is not one of them. The real carrier is the new
  `add_then_delete_returns_to_the_empty_root`.
- **LLR-n7nya3's named mutation reds one of its two carriers**; the second clause
  needed a supplementary mutation, and that one is blunt — forty tests.
- **Mutation runs harvest proptest regression seeds.** T5 deleted
  `org-members/tests/fuzz_tests.proptest-regressions` rather than commit seeds
  captured from deliberately broken code; T8 deleted it three more times. A
  committed seed file from a mutation round pins the suite to behaviour that never
  existed.

### From T6, T6a and T6b — membership operations

- **A dispatched task lost eleven attestations to a watchdog kill.** T6 batched a
  whole run of mutation rounds into an uncommitted working tree and stalled twice
  at 600s; its context was the only holder of the attestations. The annotations
  were verified by inspection and kept; the eleven mutation rounds were
  re-established in two smaller dispatches, T6a and T6b, which committed nothing
  because the product of a mutation round is the attestation, not a diff. The
  lesson recorded in the plan: cut a task of that size in two.
- **LLR-ub6dw9's named mutation reds none of its carriers.** "Stop updating
  `handle_index` in `update_leaf`" is unreachable from `get_by_handle` and
  `contains_handle`, which both build with `genesis` and read. A run that stopped
  at the named mutation would have recorded a measured negative for an LLR that is
  in fact well covered; the supplementary — drop the insert in `genesis` — reds
  both.
- **LLR-v3jqau's named mutation reds 0 of 7 carriers, and this one is worse.**
  Every public operation performs its own presence check before `update_leaf` is
  reached, and `delete_member` routes through `delete_by_id`, which guards itself,
  so `update_leaf`'s own guard is dead code from all seven tests' perspective. T6b
  split the item's two clauses instead and proved each across all seven: make the
  guards return `Ok` (not refused at all), and make them return `DuplicateId`
  (refused, wrong error).
- **Three more named mutations reached one carrier of several**: LLR-4phmjf 1 of
  3, LLR-mmst86 1 of 4, LLR-g6arcs 1 of 4; LLR-fv75ec 2 of 3 and LLR-ch2pkw 1 of
  4. Across T3, T5, T6a and T6b the same plan defect recurs — one mutation paired
  with a whole carrier list whose members assert different clauses and are reached
  by different paths. The code is well covered; showing it took a supplementary
  mutation per clause.
- **Carriers that red through a crash rather than their own assertion.**
  `add_p2p_device_rejects_when_full` panics in the four-slot encoding before its
  assertion (the coupling, above). Under the isolation mutation, two of
  LLR-w92psx's carriers red through an `unwrap()` on the now-absent member; only
  `_keeps_member_in_trie` reds through its own assertion (`member_count` 1 vs 2).
  The attestations stand; those carriers do not themselves assert what the item
  says.
- **A carrier that is a carrier in name more than in substance.**
  `update_name_surname_changes_pii` exercises neither NFC normalisation nor the
  length bounds — only the write-through of both fields — though it is listed for
  LLR-g6arcs.

### From T7 — delta exchange

- **Two clauses are guaranteed by the type system and no test can discriminate
  them**, recorded in the ledger beside their items: LLR-y38jfk's "exposes no
  member query" (`CandidateTrie` declares exactly `root_hash()` and
  `verify_against()`; the mutation that would test it is a signature change
  producing seven `E0599` compile errors, which is not a discriminating red) and
  LLR-7tdqv9's "the candidate is consumed either way" (`verify_against` takes
  `self` by value). Unlike LLR-tk4qxu's withdrawn clause these stay in their items:
  they describe the interface rather than internal structure, and a
  compiler-enforced interface property is stronger than a test — just not *test*
  evidence.
- **LLR-juxk9q's handle-uniqueness half has no carrier**, verified independently:
  all three `DuplicateHandle` assertions in the suite reach the check through
  `genesis`, `add_member` or `update_handle`, never through `apply_delta`.
  Removing the whole block reds one test,
  `apply_delta_rejects_confusable_in_upsert`. The confusability clause is carried;
  the uniqueness clause is not. A delta upserting a member whose handle duplicates
  an existing one **is** rejected by the code, and no test would notice if that
  stopped being true. Recorded as a gap rather than closed — this change adds no
  test beyond the five its plan names.
  *Forward note, 2026-10-03:* reversed — the uniqueness half is now carried,
  through `apply_delta`, by `membership_conformance` and three
  `integration_test.rs` tests. See `2026-10-03-lawful-change-replicates.md`,
  "Measured carriers".
- **`_canonical_delta_still_works` stays green under all five relaxing
  canonical-form mutations.** Relaxing a check cannot break an honest delta; it
  reds only under an over-strict polarity flip.
- **`apply_delta_to_wrong_side_after_reversed_calc_fails` is carried by the
  `base_root` mutation, not the diff-direction one** — it never inspects delta
  contents.
- **`_duplicate_in_removed` and `_stale_removal` red for the wrong reason.** The
  error degrades to `InvariantViolated` inside the apply loop rather than the
  canonical check returning `MalformedDelta`: the mutation is the cause, but the
  discriminating assertion is not what fails.

### From T8 — error reporting, the fuzz targets and the model

- **LLR-ch2pkw is not reached by the model, and the reason is structural.**
  Removing `genesis`'s identifier, handle *and* skeleton guards leaves
  `membership_conformance` green: the model's `init` is the empty map and both
  driver call sites are `Trie::genesis(Vec::new())`, so `genesis`'s per-member loop
  body never executes. More samples cannot change that. `mbt_conformance.rs` is
  annotated with three of the four candidates — LLR-fv75ec, LLR-j4d38d and
  LLR-v3jqau — and the reason for the fourth's absence is written into the
  annotation block in the file, where a later reader will look.
  *Forward note, 2026-10-03:* reversed — the model's `init` is now genesis with
  generated seeds, so `genesis`'s per-member loop runs and
  `membership_conformance` carries LLR-ch2pkw. See
  `2026-10-03-lawful-change-replicates.md`, "Measured carriers".
- **The `unwrap` mutation is two kinds of evidence, kept apart.** `cargo test` does
  not run clippy, so the mutation compiles and three carriers red on the real
  behaviour change — `called Result::unwrap() on an Err value: DuplicateId`, a
  rejection reaching a panicking path. Separately, `cargo clippy -p org-members
  --lib` errors citing `#![deny(clippy::unwrap_used)]`. The clippy failure is
  compile-time evidence for LLR-h9gs32's lint-posture clause and proves nothing
  about the tests; the test reds are what discriminate the carriers.
- **That mutation reaches 3 of 4 carriers.** `handle_validation_never_panics` only
  calls `validate_handle` and never reaches `add_member`; a second mutation — a
  panic in the uppercase branch — reds it on input `"Ę"`.
- **More carriers that red through a crash.** `calculate_delta_roundtrip` and
  `trie_ops_never_panic_and_count_consistent` red via a panic propagating out of
  `trie.rs` rather than via their own `prop_assert`s. For a panic-freedom clause
  that is the assertion in substance, but it should be read as "any panic fails
  this" rather than as a targeted check, and `calculate_delta_roundtrip`'s own
  assertions concern the diff roundtrip, not error reporting.
  `membership_conformance` under the handle-guard mutation reds through the
  driver's own `expect(…)` at `mbt_conformance.rs:149` — a harness crash inside
  `commit()`, before the model/implementation comparison runs. The other three
  model mutations red properly, with "Specification and implementation states
  diverge".
- **LLR-8jttpb confirmed independently of T5, and more sharply.** Reversing the
  traversal makes `delta_canonicality_fuzz` fail on `mutator = ReverseUpserted`
  with "apply_delta accepted a non-canonical delta" — because the honest delta now
  emerges in *decreasing* identifier order, so the fuzzer's reversal restores
  canonical order. That is the increasing-by-construction clause failing through
  the test's own path rather than through a crash.
- **A stronger carrier for LLR-4n8zqx exists and is unused.**
  `mbt_conformance.rs:278–301` implements a root-hash equality-class check — equal
  model state implies equal real root, and distinct model states never share a
  root. T8 correctly did not annotate it: its brief capped that test at four named
  candidates, and a fifth unmeasured annotation is the unearned evidence the whole
  method exists to avoid. Nothing is owed; the LLR is carried and proven. This is
  the best measured follow-up available to a later change.
  *Forward note, 2026-10-03:* used — `membership_conformance` now carries
  `verifies: LLR-4n8zqx`, measured red 5/5 under an order-dependent leaf hash
  in `smt::insert`. See `2026-10-03-lawful-change-replicates.md`, "Measured
  carriers".

## Obligations from the independent review — opened 2026-09-17

The independent review at this change's merge (merge-change step 6a) returned
thirteen findings. Six were fixed in the review-fix round; one raised no fault;
the six below the owner disposed of as **record, do not fix** on 2026-09-17.
They are written here rather than in a verification record because a
verification record is read once, at its own merge, and these outlive it. Three
of them sit in the architecture ledger instead, where their subject is:
finding-2 (SDD-55b2zj's boundary has no end-to-end evidence), finding-8 (unmarked
derived work under SDD-k5wa4n) and finding-11 (`org-members/docs/CONTEXT.md`).

**No HAZ, RC, SDD or LLR is minted for any of these.** Where an item looks
wanted, that is said and left to the owner.

### Obligation A (finding-5) — four normalisation sites no test carries

Opened 2026-09-17. **Corrected the same day** (round-2 independent review): this
obligation was written as "three normalisation sites" and its table listed
three, while its own prose already described four — it said the deserialisation
half is uncarried "for `name` as well as for `surname`" and then omitted the
surname row from the table. The two halves disagreed; the prose was right. The
fourth site was verified by mutation before being added here, the same way the
first three were.

Same shape as the LLR-juxk9q handle-uniqueness gap this change already declares,
and found the same way: replace the transformation with the identity and see
whether anything reds.

**Four** substitutions each leave the suite at **115/115 green**:

| Site | What was replaced | Why nothing reds |
|---|---|---|
| `trie.rs:174` | `to_nfc` on `update_name_surname`'s `surname` | the carriers pass an already-NFC surname, and assert only over `name` |
| `types.rs:530` | `to_nfc` on `MemberLeaf::new`'s `surname` | same — no carrier constructs a member with a decomposed surname |
| `types.rs:484` | `to_nfc` on `name`, on the `Deserialize` path | no test deserialises a `MemberLeaf` whose `name` is non-NFC |
| `types.rs:490` | `to_nfc` on `surname`, on the `Deserialize` path | added 2026-09-17; measured with `let surname = raw.surname.clone();` in place of `to_nfc(&raw.surname)` — 0 + 6 + 108 + 1 + 0 = 115 passed, 0 failed, then reverted and verified byte-identical by SHA-256 |

So **both** halves of normalisation are uncarried at **both** of their sites:
`name` and `surname` on the `Deserialize` path, and `surname` on each of the two
construction paths. The only normalisation clause any test carries is `name` in
`update_name_surname`. The behaviour is correct — the code does normalise — but
nothing in the suite would notice if it stopped.
`update_name_surname_nfc_normalizes` is the test a reader would expect to close
this and does not: it asserts the `name` field only, and supplies an
already-normalised surname.

What it bears on: LLR-g6arcs (`update_name_surname` normalises and applies the
field bounds) and LLR-w5nkbu (the 128-byte bounds *after NFC normalisation*) are
both annotated and both partly uncarried on the normalisation clause. It bears
on risk through REQ-h5ret5 and REQ-m8aexh only indirectly — `name` and `surname`
are display fields and are not the confusability decision, which runs on the
handle and *is* carried. That is why this is an obligation and not a hazard.

To act on later: four assertions, not four tests — feed a decomposed name or
surname through each site and compare against the composed form. Cheapest of the
six.

### Obligation B (finding-6) — LLR-h9gs32's rationale is stronger than its lints

Opened 2026-09-17. The item reads: "every rejected operation returns an
`OrgMembersError` variant, and the crate denies `unwrap`, `expect` and `panic`
at the lint level **so that no input reaches a panicking path**."

The first two clauses are true and carried. The conclusion after "so that" is
not established by the premise. `org-members/Cargo.toml` denies `unwrap`,
`expect` and `panic` — and those denials are real, CI-gated, and were measured
by T8, which saw `cargo clippy -p org-members --lib` error on
`#![deny(clippy::unwrap_used)]`. But clippy's `indexing_slicing`,
`arithmetic_side_effects`, `integer_arithmetic` and their relatives are **not**
denied, and slice indexing and arithmetic panic without going anywhere near
`unwrap`. Live indexing sites in the crate include:

- `types.rs:66` — `self.0[byte_idx]` in `MemberId::bit`, which is **PR-jq43gx**:
  a panic reachable from outside the crate. That report is the concrete instance
  of this obligation, and the reason this one is not merely theoretical.
- `types.rs:419–421` — the fixed four-slot array fill in `to_fixed_slots`, whose
  panic T6a already observed (`index out of bounds: the len is 4 but the index
  is 4`) under the removed-bound mutation.
- `smt.rs:34`, and the `windows`/removed-element accesses in `trie.rs`.

The lint posture is a genuine control and nothing here weakens it. What is
overstated is the inference: the lints exclude one family of panics, not all of
them. Two ways to settle it, both for a later change and neither minted here:
**(a)** reword the item so its rationale claims what the lints give — "denies
`unwrap`, `expect` and `panic` at the lint level, so no rejection is reported by
panicking" — and carry panic-freedom-for-any-input on the fuzz targets, which is
where it is actually measured; or **(b)** extend the denials to the indexing and
arithmetic lints and fix what that reddens, which is a real code change and
would subsume PR-jq43gx. **(b)** is the stronger answer and the more expensive
one. The owner chooses.

### Obligation C (finding-7) — the SOUP advisory-database gap is disclosed but untracked

Opened 2026-09-17. `soup.md` states it plainly, twice, and finding 5 of "Findings
for the owner" above repeats it: no `cargo-audit` and no `cargo-deny` is
installed in this toolchain, there is no audit target in the `Makefile`, and
every SOUP risk column is therefore reasoned from the code's use of the crate
rather than from an advisory feed.

The obligation is not the gap — that is honestly disclosed. **It is that the gap
is carried by no item and aged by no gate.** Prose in an inventory and a
narrative finding in a risk file are both invisible to `check-trace.sh`: nothing
prints it at a merge, nothing counts its days, and nothing will fail when it is
still open in six months. Compare PR-zz4exm, which is the same kind of
outstanding obligation and *is* surfaced with its age at every merge because it
is an item with an `opened:` date. (Corrected 2026-10-03: PR-zz4exm is now
resolved; the comparison is with how it was carried while open.)

**Recommendation, not minted here:** give it a tracked item carrying an
`opened:` date — a problem report is the natural shape, since "the risk
assessment for nine SOUP crates rests on no advisory data" is an observable
defect in the evidence, and a PR is what the toolkit ages. The brief for this
round said to mint nothing beyond PR-jq43gx, so nothing is minted; this
paragraph is the recommendation and the date it was made.

Note the scope if it is minted: the gap is the **toolchain's**, not this unit's
alone. `org-node`, `on-chain-client` and `app` have the same absence, and
`on-chain-client`'s closure is the one where a pre-release `ed25519-dalek`
3.0.0-pre.6 actually lands. A per-unit problem report in four ledgers may be
worse than one tracked item at the repository level; that is part of what the
owner decides.
