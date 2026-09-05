# Hazard analysis — organisation membership

The first hazard enumeration in this ledger. It is run under IEC 62304
**Class C** (`docs/adr/2026-09-01-safety-class-c.md`) and evaluated against the
acceptability matrix in this ledger's README, both settled in this same change.

## Scope

The capability analysed is **on-chain-anchored organisation membership**: who
belongs to an organisation, which device keys each member holds, and how a
change to that record is proposed, applied and committed. Concretely that is
the `org-members` trie, the `OrgRegistry` contract and the reads and event
lanes of `on-chain-client`, together with the verify-and-commit path in
`org-node`.

Deliberately **outside** this analysis, and not to be read as assessed:

- **Document content protection** — the CGKA/ACL layer that turns membership
  into decryption capability, and with it the whole notion of a *grant*. It is
  Phase 3, it exists only as the `spike-keyhive` and `spike-p2panda` crates and
  the `protocol.qnt` model, and it carries the second half of every disclosure
  pathway below. This analysis stops at the membership decision; what a holder
  of the member-as-a-group key can decrypt is the next capability to analyse.
- **Transport** — iroh connectivity, blob fetch, and the peer discovery path.
- **The application surface** — how an administrator is shown a member before
  acting on them. HAZ-jkc6tj and HAZ-sc7wse below are exactly the hazards that
  surface must address, and neither can be discharged here.

## Method, and what the harms are

The chain walked for each hazard is ISO 14971's: hazard, hazardous situation,
harm, severity, probability, evaluation, controls, residual risk. Candidates
were sought systematically — wrong output, stale data, races on shared state,
authority failures, foreseeable misuse, hostile input, resource exhaustion,
degraded infrastructure — rather than by reading the requirements and asking
what they protect against, which would have found only hazards already
controlled.

Two harms recur, and they are the two the class C ADR names:

- **Disclosure.** Someone reads organisational material they should not. In
  the journalism and government deployments that material identifies a source
  or a protected person, and the harm at the end of the chain is what happens
  to that person. **S3.**
- **Unavailability.** A member cannot reach organisational material at the
  moment a decision needs it. In the clinical deployment the decision is a
  care decision taken without the record, and the worst credible harm of that
  is **S3** — which supersedes the class C ADR's remark that the clinical
  context alone would have sustained Class B. Severity is a property of the
  harm, and an organisational fallback does not make an injury less severe.

**What "probability" means here, stated precisely because the distinction was
mishandled in an earlier draft of this file.** Every probability below is the
probability of the **hazardous situation arising** — the software reaching the
state the hazard describes, given the conditions the pathway needs. ISO 14971
decomposes the probability of harm into that step and a second one, the
probability that the hazardous situation leads to the harm. **This register
does not estimate the second step**, and no number below carries it. That is
where an organisation's fallbacks act — paper records, a phone call, asking the
person, a second administrator noticing — and where the deployment context
would have to be modelled to say anything quantitative. An earlier draft
claimed the fallbacks were "counted in probability, not severity"; they are
counted nowhere, and the honest statement is that the register estimates
occurrence and leaves the situation-to-harm step unquantified.

**A note on severity, because it is the uncomfortable result.** Seven of the
eight hazards below are S3, and the matrix makes S3 unacceptable at every
probability. That is not a defect in the estimate. An access-control system
for protected identities has one dominant harm and it is serious; the matrix
was set knowing that. What it means for acceptability is settled in the
residual-risk section.

## Hazards, and the controls chosen for them

Ten controls are minted below. Each is realised by a requirement in
`docs/requirements/2026-08-31-org-membership.md` that now carries
`(implements: RC-…)`, and each is worded from behaviour that requirement
already states — no requirement text was changed to fit a control.

Two controls are **not fully implemented**, and both are recorded at the point
of use rather than in a footnote: RC-mqtks7's second clause, which the code
does not enforce (PR-zz4exm), and RC-mqtks7's first clause, which holds on the
direct API but not on the wire path. Controls that would reduce these risks
further but have no requirement yet are named in "Controls identified but not
minted", and are deliberately given no RC identifier — an identifier would
assert a traced, tested control where there is none.

The requirements keep `satisfies: derived`. Finding the hazard a rule addresses
does not give the rule a parent in system needs; it gives it a justification.
The derived assessments in `docs/risk/2026-08-31-membership-derived.md` stand,
and each of them now has a hazard to point at.

### Misdirection by name

**HAZ-jkc6tj**: two members of an organisation can be made to bear handles
that a person cannot tell apart — by confusable characters, by case or Unicode
normalisation, or by re-use of a departed member's name; an administrator
acting on an instruction that names a member resolves the name to the wrong
principal and acts on that principal; material identifying a source or a
protected person reaches someone who should not have it. Severity: S3.
Probability: P1.

P1 rather than P2: the attacker must first be admitted to the organisation, or
persuade an administrator to admit them, before a colliding handle is worth
anything. It is not negligible — enrolment under a chosen name is exactly what
an adversary targeting a newsroom would attempt.

**RC-n2taat**: handles form a canonical, unambiguous namespace — every handle
is normalised to NFC, restricted to a form in which comparison is total
(lowercase, no `.`, a single Unicode script), required to be unique within the
organisation, and rejected when its UTS#39 confusable skeleton collides with
one already held. mitigates: HAZ-jkc6tj, HAZ-bmv7cy

**RC-4u22ba**: a member is identified by an identifier that does not change
when that member's handle changes or when any of that member's keys are
replaced, and an attempt to admit a member whose identifier is already present
is rejected. mitigates: HAZ-jkc6tj

RC-4u22ba is the stable-identifier half only. An earlier draft of this control
also asserted that "every grant is made against that identifier rather than
against the name it is displayed under" — which is not implemented anywhere in
this repository, has no counterpart in REQ-crjxk8, and concerns the grant
concept this analysis places out of scope. It is moved to the not-minted list,
where the work actually sits.

Residual risk: the skeleton test is a heuristic over a versioned Unicode data
set, and the last step of the pathway is a person reading a name. Both controls
narrow the namespace; neither makes a name-based instruction unambiguous to a
human. Reduced but **not acceptable** under the matrix, and the further control
is not in this codebase: the administrator's surface must show the stable
identifier, not only the handle, at the moment it matters.

### Revocation that appears to succeed

**HAZ-s39gbh**: a device key can be removed from a member's record without the
member-as-a-group key that device held being effectively replaced; the
organisation, having seen the removal succeed, treats a lost, stolen or
compromised device as cut off and continues to place documents in the
organisation; the removed device decrypts material published after its removal
and the identities in it reach whoever holds that device. Severity: S3.
Probability: P2.

P2, and this is the highest occurrence estimate in the register. It needs no
attacker: `delete_p2p_device` and `emergency_isolate_member` both take the
replacement key from the caller and both keys have the same type, so passing
the outgoing key back is a well-typed call that any integration can make by
accident (PR-zz4exm names both operations).

**RC-mqtks7**: on the direct membership API, removing a device key and
replacing the member's member-as-a-group key are one operation that cannot be
performed by halves, and a removal whose replacement key is the key being
replaced is rejected. mitigates: HAZ-s39gbh

**RC-sq3yhp**: a member whose devices are compromised can be isolated in one
operation that removes every device key and replaces the member-as-a-group key
together, without removing the member from the organisation, and the state is
reversible by adding a device key. mitigates: HAZ-s39gbh, HAZ-sc7wse

Residual risk: **unacceptable, and the control is defective in two distinct
ways.**

The second clause of RC-mqtks7 is not implemented: nothing compares the
replacement key against the outgoing one, so the control's effectiveness is
contingent on the caller getting it right, which is the one property a risk
control must not have. Tracked as **PR-zz4exm**, open since 2026-08-31, whose
stated fix begins with the test that reproduces it.

The first clause holds only where the control says it does, and the scoping is
new to this revision. `P2pDeviceSlots::remove_device` is crate-private, so an
external caller must go through `delete_p2p_device`, which always applies the
replacement key — atomic, and rotation is unconditional rather than at the
caller's discretion. But `apply_delta` accepts any well-formed change set, and
`validate_canonical_delta` checks ordering, presence, observable change and
disjointness only: nothing relates a leaf's device set to its member-as-a-group
key. So a change set may upsert a leaf with one device removed and the key
unchanged, which is this hazard by another route. That is the same shape of gap
REQ-shk82j exists to close for handles, and it is unclosed for this pair.
Recorded here and listed as a not-minted control rather than asserted away.

### Isolation applied to the wrong member

**HAZ-sc7wse**: the one-step isolation that RC-sq3yhp provides removes every
device key a member holds in a single call, and nothing in the software
distinguishes the member the administrator meant from the member they named;
a mistaken or coerced isolation is executed against a legitimate member, who
loses access to every organisational document at once; that member cannot
reach material at the moment a decision needs it. Severity: S3. Probability:
P1.

This hazard is created by a control. That is why it is here — ISO 14971
requires each control to be examined for what it breaks — and RC-sq3yhp is
both its cause and its mitigation, because the control has two halves that
work against each other. Removing every device is the half that creates the
hazard; retaining the member in the organisation and restoring access on the
next device key is the half that bounds it. REQ-r784fu requires both, and it
requires the second precisely so that an emergency response cannot become a
wrongful expulsion that needs a re-admission ceremony to undo.

Residual risk: reduced, **not acceptable**. The harm is bounded by the time to
notice and re-enrol a device rather than by a recovery procedure, which is a
real reduction. What is missing is any confirmation step before the call —
the same gap as HAZ-jkc6tj, in the same place: the administrator's surface.

### A stale or divergent record taken as current

**HAZ-y8h835**: a device's view of the organisation's membership can lag or
diverge from the record the organisation has committed — concurrent
administrators, a delta applied to the wrong base, a root published before it
was recomputed, or a device that has not read the chain recently; that device
treats a member removed by the organisation as current and shares material
with them, or treats a current member as removed; the removed member reads
material published after their removal. Severity: S3. Probability: P2.

P2: divergence needs no adversary, only two administrators acting at once or a
device that was offline. The chain anchor is what makes it recoverable rather
than permanent.

**RC-9z65hw**: a set of membership changes is applied only if its declared base
matches the record it is applied to, and the result of applying it is usable
only if its root matches the root the change set was expected to produce.
mitigates: HAZ-y8h835, HAZ-h58jn6

**RC-ty8qdw**: no root value is reported for a record whose hashes have not
been recomputed since its last modification; the operation reports an error
instead of a value that would attest to a superseded membership.
mitigates: HAZ-y8h835

RC-9z65hw is worded to match REQ-4umsuz exactly, and both stop short of the
property that actually matters. `CandidateTrie::verify_against` compares
against a root the **caller supplies**; nothing in `org-members` enforces that
the root came from a path the attacker does not control. The crate says so
itself, listing "independent trusted root" among the responsibilities that
must be met upstream of it. An earlier draft of this control asserted the
independence; it has been removed and appears in the not-minted list.

Residual risk: reduced, **not acceptable**, because the decisive part of this
control is neither stated as a requirement nor under any gate.

What makes the expected root trustworthy is `verify_envelope_against_chain` in
`org-node/src/verify.rs`. It checks org binding, then the author's signature,
then a strictly-increasing envelope sequence number, then decodes the delta,
then requires the chain epoch to be strictly newer than the last committed one,
and only then requires the recomputed root to equal the root obtained from a
`ChainReader` independent of the envelope. That order is deliberate and it is
correct. Two qualifications, both of which an earlier draft got wrong:

- **It is tested in-tree and run by nothing.** Seven unit tests and two fuzz
  targets exist for this path. `org-node` appears in no `verify_commands`, no
  `Makefile` target and no CI job, and is in neither `strict_paths` nor
  `test_paths`. The code is exercised only by someone who runs it by hand.
- **At verify time no block is read.** The only non-mock reader,
  `OnChainReader`, returns a cached snapshot refreshed when a caller awaits
  `refresh()`. The underlying client read is a finalised-head read, but the
  reader's own doc-comment calls it "current best", contradicting both
  `preflight.rs` and the client's documented `at = None` semantics. That
  contradiction sits on the one control the whole anchor depends on, so it is
  filed as **PR-hvg2dy** rather than noted in passing.

The staleness bound is the other half, and it is weaker than an earlier draft
claimed. `protocol.qnt` models a `MAX_AGE` policy in which a device may accept
a write only if it read the trie within `TAU` ticks, and states `tauWindow`:
every write accepted on a chain-invalid basis was accepted inside that window.
CI runs that invariant under the simulator (5000 samples, 16 steps) and under
Apalache to depth 5, so it is checked rather than merely written down — but
"proved" overstates it in two ways. The check is bounded, as the models' own
README insists, and the invariant is entailed by the action guard: the only
action that admits a write requires the window and then stamps the staleness it
just required, so `tauWindow` cannot fail unless that guard is deleted. It
documents the policy; it does not independently establish anything about it.
And the policy exists only in the model: no requirement states a maximum age,
no implementation enforces one, and no number has been chosen for a deployment.

### A member introduced through the wire

**HAZ-bmv7cy**: a serialised membership record arriving from another process
can carry a member whose handle or device set the software would have refused
to admit; that record is accepted and the member becomes current in the
receiving organisation; a principal the organisation never admitted holds
membership and the access that follows from it. Severity: S3. Probability: P2.

P2: any peer or on-disk store is such a source, and the deserialisation path is
reachable by anything that can hand the node bytes.

**RC-4apk6w**: every member handle and device key set received from outside the
process is re-validated on decode — handle validity, a sorted duplicate-free
device set, and the device bound — so that the rules are properties of the
membership record rather than of the constructor path.
mitigates: HAZ-bmv7cy

Residual risk: reduced, **not acceptable**, on two counts.

The re-validation is deliberately weaker than direct admission: it accepts an
empty device key set where admission requires at least one, and it defers
handle uniqueness and the confusable check to the point where the change set is
applied. REQ-shk82j says so in its own text, which is why the control's wording
follows it rather than claiming equivalence.

Its evidence is narrower than an earlier draft of this file stated. The five
tests annotated `verifies: REQ-shk82j` are example-based unit tests over
hand-constructed postcard payloads, not proptests; the crate's proptests carry
annotations for other requirements. Structured examples on a decoder are the
weakest useful evidence, and the correction matters more than the wording,
because the earlier draft was being self-critical in that very sentence and
still overstated what backed it.

### Unavailability under hostile input

**HAZ-8suua9**: a membership operation can be given malformed, hostile or
oversized input; the library panics inside the node or the application and
takes the caller down with it, or exhausts memory on an unbounded device set;
a member cannot reach organisational material at the moment a decision needs
it, and in the clinical deployment that decision is taken without the record.
Severity: S3. Probability: P2.

Severity is the worst credible harm of the unavailability pathway, argued in
"Method". P2: the input is attacker-chosen and the surface is a decoder.

**RC-c4truv**: every rejected membership operation reports a typed error and no
input, however malformed or hostile, causes a panic. mitigates: HAZ-8suua9

**RC-3ppkf6**: the number of device keys a member may hold is bounded, a
duplicate device key is rejected, and a device key added to a member already at
the bound is rejected. mitigates: HAZ-8suua9

Residual risk: reduced, **not acceptable** — but for a different reason than
the one an earlier draft gave, and the correction is the most consequential in
this revision.

RC-c4truv is the broadest claim in the register: "for any input". Its
structural support is real — `org-members` denies `unwrap`, `expect` and
`panic!` at the crate root and contains none of them in `src/`. Its behavioural
evidence is five bolero targets: two arbitrary-bytes targets and one structured
round-trip target over the chain decode path in `on-chain-client`, and two over
the `org-node` path. All five run under bolero's default generative engine in
the lanes that invoke them, not under libFuzzer, which each target's own header
says is a separate deep-fuzz invocation.

**An earlier draft of this file, and the class C ADR written in the same
change, both stated that no arbitrary-bytes target reaches the `org-members`
deserialisation path. That is false, and it was inherited from tooth 4's
verification record without being re-checked.**
`org-node/tests/fuzz_envelope_decode` feeds arbitrary bytes to a
`SignedDeltaEnvelope` decode and then calls `decode_delta()` on anything that
parses, which deserialises `org_members::delta::Delta` — through
`MemberLeaf::deserialize` and `P2pDeviceSlots::deserialize`, the exact
validation surface this hazard names. `fuzz_verify_against_chain` reaches the
same decoder via step 4 of the verify path. Both predate this change.

So the real gap is not a missing target. It is that the target exists, sits
outside `test_paths`, carries no `verifies:` annotation, has an empty seed
corpus, and is run by no lane — evidence that exists and counts for nothing,
which is a worse position to be in than a known absence, because a known
absence is visible. Annotating and running it is a not-minted control below.

### The record altered under what was decided against it

**HAZ-m2xfrm**: a published membership record can be altered in place rather
than superseded by a new value; a decision already taken against that record —
a grant, a removal, a comparison of two organisation states — can no longer be
reconstructed or checked, and two divergent views cannot be reconciled;
the organisation loses the evidence of what its membership was, with no
person exposed and no access wrongly granted or denied by this failure alone.
Severity: S1. Probability: P1.

S1 deliberately. This is an integrity-of-evidence harm: it destroys the ability
to audit and to converge, which matters, but no injury pathway runs through it
that is not already counted under HAZ-y8h835. Recording it as S3 because it
sounds serious would make the severity scale meaningless. Note that S2/P1 is
equally ACCEPTABLE under the matrix, so S1 is not being chosen to clear a bar.

**RC-3qn5xg**: applying a modification leaves the existing record unchanged and
produces the modified membership as a separate value, so a record already
published cannot be altered in place. mitigates: HAZ-m2xfrm

Residual risk: **acceptable.** S1 is acceptable at every probability under the
matrix, and the control is structural — path-copying persistence rather than a
check that can be skipped.

### A superseded change becoming applicable again

**HAZ-h58jn6**: the organisation's committed root can return to a value it
previously held — the registry contract accepts any root at the next epoch,
including a prior one, its only bar being a no-op guard that rejects
re-publishing the *current* pair — at which point a superseded change set
whose declared base is that root becomes applicable again; a change set that
re-admits a removed member, or restores a device key, is applied a second time
and takes effect; the removed member's access returns without any administrator
intending it. Severity: S3. Probability: P1.

P1: it needs an administrator to republish an earlier root, by mistake or under
coercion, and a retained copy of the superseded change set. The library's own
documentation states this residual explicitly — base-root matching rejects a
stale change set only while the record has moved past its parent.

RC-9z65hw (defined above) mitigates this hazard too: base matching is what
makes a change set applicable to exactly one record state. It does not close the
pathway, because the pathway is precisely the case where the record state
returns.

Residual risk: reduced, **not acceptable** from the analysed code alone. The
control that closes it is the monotonic envelope sequence number, which
`org-node` enforces as `SeqGuard` before the delta is decoded and advances only
after the root match, so a replayed envelope is refused whatever the root says.
Same position as HAZ-y8h835: implemented, tested in-tree, run by no gate.

## Hazards introduced by these controls

ISO 14971 requires each control to be checked for the hazards it creates. Four
do create them, all unavailability pathways. One — isolation applied to the
wrong member — is minted above as HAZ-sc7wse, because RC-sq3yhp genuinely
mitigates it and REQ-r784fu is tested, so the trace closes honestly. The other
three cannot be minted, for the reason in the next section.

- **RC-mqtks7 and RC-sq3yhp (unconditional key replacement).** Every remaining
  device must learn the replacement key. A device that misses the rotation
  holds a superseded key and cannot decrypt material published after it — a
  legitimate member locked out by a revocation aimed at someone else. The
  distribution of the replacement key is the CGKA layer, outside this analysis;
  `protocol.qnt` models it as `cgkaRotate` and states `revocationSafety` only
  for objects that have *settled*, which requires both that every current
  device has reached the chain epoch and that the object's CGKA token was
  rotated at or after that epoch. Before settling, this is real. Assessed
  S3 / P1, and **the control lies in a capability this analysis does not
  cover**.
- **RC-n2taat (single Unicode script, lowercase, no `.`).** A member whose real
  name mixes scripts cannot be admitted under it. The harm is an administrative
  obstruction and, if worked around with a transliteration, a name that reads
  as someone else's — which feeds HAZ-jkc6tj. Assessed S1 / P2. Acceptable, and
  **no control exists**, because none is needed at S1.
- **RC-3ppkf6 (device bound).** A member at the bound cannot enrol another
  device; if that device is the one they have, they are locked out. The bound is
  4. Assessed S2 / P1 — acceptable under the matrix, **no control exists**, and
  this is the reason the number belongs in a low-level requirement where it can
  be reconsidered against this hazard rather than against the sub-trie depth
  alone.

## Hazards whose only control lies outside this codebase

**Compromise of the organisation's admin account.** `OrgRegistry.update`
authorises on `msg.sender`, so whoever controls an organisation's admin account
can publish any root — including one admitting themselves — and every node's
verify-against-chain will then accept a matching delta as authentic, because
the chain is the authority it consults. Assessed S3 / P1: total disclosure,
needing a compromise of the organisation's own governance. It is the single
point on which every other control in this register depends.

**The control is a deployment obligation with nothing behind it yet, and an
earlier draft of this file overstated its standing on two counts.** It claimed
the contract's `msg.sender` authorisation is "deliberately agnostic" — that is
a charitable reading, not a position any project document takes. And it cited
`on-chain/POST_POC.md` for a per-organisation admin multisig; that document
recommends a multisig for the **upgrade** authority (3-of-5 maintainers plus a
timelock) and says nothing about an organisation's admin account. What the
repository actually implements is a **threshold-1** pure-proxy multisig, by
design: the write path's own module documentation says so, and its
threshold-≥2 dispatch path is dead code that errors if reached. A threshold-1
multisig confers no additional authority protection.

So: an organisation that runs this software with a single-key admin account has
no protection from any control above, and no document in this repository yet
tells an operator otherwise. Writing one is a not-minted control below.

Not minted as a HAZ item, for the reason in the next section.

## A limitation of the ledger, recorded rather than worked around

`check-trace.sh` requires every HAZ to have an RC that mitigates it, every RC to
have a requirement that implements it, and every requirement to have a test that
verifies it. The chain is sound for controls that live in software. Its effect
here is that **a hazard can only be recorded once it is already controlled and
tested**, so four hazards above cannot be given identifiers: the admin-account
compromise, whose control is key custody; the missed-rotation lock-out, whose
control is in the CGKA layer; and the two the matrix already calls acceptable,
which have no control because none is required.

ISO 14971 runs the other way round: hazards are identified first, and risk
control is a later activity that acts on what the analysis found. A gate that
refuses an uncontrolled hazard pushes toward recording fewer hazards, which is
the opposite of what the ledger is for.

The scope of that complaint is narrower than an earlier draft of this file
claimed. It asserted five unmintable hazards; one of them — isolation applied
to the wrong member — was mintable all along, because RC-sq3yhp mitigates it
through a requirement that is tested, and it is now HAZ-sc7wse. That was the
author's reasoning failing, not the toolkit's. The remaining four are the
limitation's real instances, and only the first is a hazard anyone would want
an identifier for.

The four are written above in prose, with severity, probability and their
controls, so nothing is lost to a reader. What is lost is the mechanical trace,
and that is a toolkit limitation to report upstream, not a reason to leave a
hazard out.

## Residual risk, and the conclusion this analysis reaches

Per hazard, after controls:

| Hazard | S/P | Residual | Why |
|---|---|---|---|
| HAZ-jkc6tj | S3/P1 | not acceptable | last step is a person reading a name |
| HAZ-s39gbh | S3/P2 | not acceptable | control unimplemented in one clause (PR-zz4exm), bypassed on the wire path in the other |
| HAZ-sc7wse | S3/P1 | not acceptable | bounded by reversibility; no confirmation step exists |
| HAZ-y8h835 | S3/P2 | not acceptable | decisive control unstated, ungated and unrun; staleness bound modelled only |
| HAZ-bmv7cy | S3/P2 | not acceptable | re-validation weaker than admission; evidence is five example tests |
| HAZ-8suua9 | S3/P2 | not acceptable | broadest claim in the register; the target that would support it is unrun |
| HAZ-m2xfrm | S1/P1 | **acceptable** | S1 acceptable at every probability; control structural |
| HAZ-h58jn6 | S3/P1 | not acceptable | closing control ungated and unrun |

And the four prose hazards: admin-account compromise **not acceptable**
(S3/P1, no software control); missed-rotation lock-out **not acceptable**
(S3/P1, control outside this analysis); mixed-script exclusion **acceptable**
(S1/P2); device-bound lock-out **acceptable** (S2/P1).

So of twelve hazards, nine carry residual risk this project's matrix calls
unacceptable, and no probability estimate could have changed that at S3.

### Overall residual risk

**Against the intended use that makes this software Class C — production
deployment in journalism, clinical or government organisations — the overall
residual risk is UNACCEPTABLE.** That is the conclusion, and it is a statement
about readiness rather than about any single hazard: nine unacceptable
residuals, two controls not fully implemented, and the decisive control on the
anchor neither stated as a requirement nor run by any gate.

**What this does not mean.** An earlier draft of this file concluded that
overall residual risk was "acceptable while the intended use excludes those
three contexts". That was circular and is withdrawn. Every S3 severity in this
register is derived *from* those three contexts; excluding them as a control
would remove the basis for the severities it was invoked to discharge. It also
leaned on the weakest control category to discharge nine unacceptable risks,
which this ledger's own README rules out in as many words: information for
safety "does not discharge an unacceptable risk on its own". And the
restriction is nowhere a user would meet it — not in the repository README, not
in crate documentation, not in any operator note — so it is not an implemented
control at all, only an intention.

**What follows instead.** The software is a proof of concept and has not
reached its intended use. That is not a risk control; it is the analysis's
result. Two things follow, and neither is an agent's decision:

1. **A restriction on use must be written where a user encounters it** — the
   repository README and the crate documentation at minimum — so that it can
   be verified as implemented under ISO 14971 clause 7.2 rather than asserted
   here. Until then this register is the only place the conclusion exists.
2. **The class stays C.** The class is set by the *planned* intended use, on
   the reasoning the class C ADR already gives: the classification governs how
   the software is built, and it is being built now. The class and the
   residual-risk evaluation are deliberately measured against different things
   — the planned use and the present state — and lifting the restriction
   re-opens the evaluation, not the class.

**One thing this register cannot supply.** ISO 14971 clause 8 requires the
overall residual-risk evaluation to be made against criteria defined in a risk
management plan. This project has no such plan: `docs/risk/README.md` holds a
per-hazard acceptability matrix and nothing about overall acceptability. So the
verdict above is a reasoned conclusion, not the output of a stated criterion,
and writing those criteria is an open item on the ratchet checklist.

## Controls identified but not minted

Each would reduce a residual risk above. None has a requirement, and none is
given an RC identifier until it does — that is `grill-requirements` work with a
test written before the requirement is claimed, not analysis work.

1. **Enforce the replacement-key check** (HAZ-s39gbh). Not a new control: make
   RC-mqtks7's second clause true. PR-zz4exm, open, with the reproducing test
   already specified. The only item here that closes an unacceptable residual
   risk outright.
2. **Relate a leaf's device set to its member-as-a-group key in delta
   validation** (HAZ-s39gbh). Closes RC-mqtks7's wire-path bypass, and is the
   same kind of rule REQ-shk82j already applies to handles.
3. **Annotate and run the arbitrary-bytes decode target**
   (HAZ-bmv7cy, HAZ-8suua9). `org-node/tests/fuzz_envelope_decode` exists and
   reaches the validation surface. It needs a `verifies:` annotation, a seed
   corpus, and a lane that runs it — not writing from scratch, which is what an
   earlier draft wrongly called for.
4. **Bring `org-node` under the traceability gate** (HAZ-y8h835, HAZ-h58jn6).
   The controls exist and are tested in-tree; the requirements, the `verifies:`
   annotations and a lane that runs them do not. Tooth 7 covers `strict_paths`;
   `test_paths` and a CI job have to come with it.
5. **State the independence of the expected root as a requirement**
   (HAZ-y8h835). `verify_against` compares against whatever the caller passes;
   that the root comes from a path the attacker cannot control is the property
   that matters and the one nothing states.
6. **A staleness bound with a chosen number** (HAZ-y8h835). `TAU` is a model
   constant. A requirement stating the maximum age of a membership view a
   device may act on, and an implementation that refuses to act on an older
   one, is what would turn the modelled policy into a control.
7. **Identity confirmation before an act on a named member** (HAZ-jkc6tj,
   HAZ-sc7wse). The administrator's surface must show the stable identifier
   alongside the handle, and confirm before a one-step isolation. Phase 2.4.
8. **Make grants reference the stable identifier, not the handle** (HAZ-jkc6tj).
   Cut from RC-4u22ba in this revision because nothing implements it and grants
   are out of scope here. It belongs with the ACL capability.
9. **An operator document requiring a threshold-≥2 admin account**, and the
   write-path support for it (the admin-compromise hazard). Nothing in this
   repository asks for this today and the implemented path is threshold-1. A
   precondition on any production use.

## Derived requirements assessment

Every requirement in `docs/requirements/2026-08-31-org-membership.md` is
`satisfies: derived` and was assessed in
`docs/risk/2026-08-31-membership-derived.md`, which remains the assessment of
record. This change adds nothing to that set and introduces no new derived
requirement. What it adds is the hazard each assessment was written toward:
that file said to read every "Hazard impact: mitigates" as shorthand for
"mitigates a hazard that has not been enumerated yet", and the hazards are now
enumerated here.
