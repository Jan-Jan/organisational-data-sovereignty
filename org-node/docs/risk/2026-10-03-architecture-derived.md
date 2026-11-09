# Risk assessment of the derived low-level requirements — 2026-10-03

org-node's first architecture ledger
(`org-node/docs/architecture/`, this change) introduces one hundred and eleven
low-level requirements. **Seventy-one** of them refine a high-level requirement that the
risk ledger has already assessed, and inherit that assessment unchanged.

*Counts corrected 2026-10-04 by review round 1, twice over: this opening said
"eighty-two … eighty of them", a figure left behind when a requirement was cut,
and the split was wrong again once the round found fifteen further items were
derived. Corrected a third time the same day by review round 2, which found
`revoke_member` and the four out-of-band blob functions refined by no low-level
requirement at all; the ten written to close that are below, six of them
derived. Corrected a fourth time by review round 3, which found four record
lookups able to ignore the identifier they were given and `revoke_member`'s
Loopback dial refined by nothing; three more were written, all derived.
**Twenty-six** were then `satisfies: derived` — the two assessed below, the
fifteen in the section added at the end of this file, the six in the section
after it, and the three in the last.*

*This count has now been wrong four times, always in the same direction and
always for the same reason: it was a summary of a number, re-stated rather than
re-counted. It is `grep -c '^satisfies: derived'` against the decomposition,
which was 26, against `grep -c '^\*\*LLR-'`, which was 94, when this
sentence was written. Review round 6 found it still in the present tense after
both had moved. The current figures are at the end of this file, beside the
section that last changed them.*

*And a fifth time, found by review round 5: the opening said ninety-four and
sixty-eight after round 4 had written two more requirements. Re-counted
2026-10-04 after round 5 wrote four more, none of them derived: 100 and 26, so
74. The three problem reports round 5 opened (the resolved problem report on the receive path rebinding Personas, the endpoint-bound-once report — org-io's since 2026-10-13 — and the resolved problem report on an update clearing the Organisation secret)
carry their own hazard notes and change no assessment in this file. They are
defects against requirements that refine assessed high-level requirements, not
new derived behaviour.* *(That last claim was wrong for three of round 5's
requirements, which were derived. Review round 6 re-traced and assessed them:
see the last section. The count was then 71 of 100.)* *(Both figures put
in the past tense 2026-10-05 by review round 8, which found them still in the
present tense. The current figures are at the end of this file.)*

**Two were marked `satisfies: derived` when this file was written**, and ISO
14971 asks that a behaviour no
requirement demands be assessed rather than assumed harmless. This section is
that assessment. It introduces **no new hazard and no new risk control**, and
it says why for each.

## LLR-gu6u53 — the Debug rendering of an Organisation identifier

assesses: LLR-gu6u53

**What it records.** `OrgId`'s `Debug` renders the twenty bytes as forty
lowercase hexadecimal digits inside `OrgId(0x…)`.

**Assessment.** An Organisation identifier is a public contract storage key —
it is `h160_of(P)`, it is carried in every invite, and it is readable on chain
by anyone. Rendering it is not disclosure. The hazard this would touch if it
held a secret is HAZ-45ucqx (the store holds every secret the node has), and
`OrgId` is not among the values that hazard enumerates; the values that are —
member seed, device seed, Organisation secret — have no `Debug` written for
them by this unit. *(Corrected 2026-10-04, after review round 7, by the author's
check of every open problem report against this file. The sentence is true of
the bare values and false of the records that hold them. `PersonaRecord` and
`OrgRecord` derive `Debug` over those seeds and that secret as plain
`[u8; 32]`, so `{:?}` of a record prints them in clear. That is the resolved problem report on secret key material in debug output,
which arrived with the `24317e3` merge after this sentence was written. It
does not change this LLR's assessment, because `OrgId` holds no secret, but
the sentence must not be read as saying the secrets cannot reach a log.)*
*(Re-checked 2026-10-05 by the org-node type-safety change, at its merge of
this file. The resolved problem report on secret key material in debug output is resolved there: the records now hold the seeds and the
secret in `MemberSeed`, `DeviceSeed` and `OrgSecret`, whose `Debug` renders
only the redaction marker (LLR-sz4xhc), and no value holding a secret renders
its bytes (LLR-bwb9pu); the controls are assessed in
`2026-10-04-type-safety.md`. This LLR's assessment is unchanged.)*

One adjacent note, recorded because it is the kind of thing this check exists
to surface: `VerifiedUpdate` **does** derive `Debug`, and its doc comment warns
that the output includes the full node tree. That is a pre-existing property of
SDD-na9nc3's return type, not a consequence of this LLR, and it is already
stated at the type. It is not re-assessed here.

No new hazard, no new control, no probability change.

## Fifteen further derived low-level requirements — added 2026-10-04

Review round 1 of this change found fifteen low-level requirements citing a
high-level requirement that does not state them: thirteen cited an earlier form of REQ-dwb9bq,
which states only the **receiver's** commit rule for an arriving Envelope, and
two cited REQ-hzm4kt, which is solely about the Persona store being ciphertext
under a passphrase-derived key. Neither says anything about calldata layout,
multisig account derivation, the chain-operations seam, Persona construction or
the administrator's send path. `check-trace.sh` could not catch it because every
reference resolved.

They are now marked `satisfies: derived`, which is what they always were, and
this section is the assessment ISO 14971 asks of behaviour no requirement
demands. **The underlying finding is an SRS gap, not a ledger slip:**

> **org-node has no high-level requirement for its administrator-side write
> path.** The unit's SRS states what a node does with a Change set it
> *receives*. What a node does when it *originates* one — builds the calldata,
> derives the controlling account, submits, and pushes to the new member — is
> stated nowhere above the design layer.

That gap is open and owed a
`grill-requirements` pass of its own. It is not closed here, because writing
high-level requirements is a requirements change with its own review, not a
sub-task of an architecture tooth.

### A. The write path — LLR-rv4vux, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb

assesses: LLR-rv4vux, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb

**What they do.** Build the hundred bytes that move an Organisation's root
forward, derive the pseudo-account entitled to send them, choose direct versus
threshold-1 dispatch, and push the signed Change set to the new member.

**The assessment turns on a fail-closed argument.** Every error in this group
costs *availability*, not *integrity*, because the receiver re-derives
everything independently (SDD-na9nc3) and commits only on a root match:

- wrong calldata layout or a drifted selector → the contract rejects the call,
  or writes a root no receiver can reproduce; either way no receiver commits;
- wrong account derivation → the chain refuses the extrinsic as unauthorised;
- wrong dispatch shape → the same.

So a defect here cannot make a receiver accept a Change set it should refuse.
It can stop a legitimate change from ever landing, which is HAZ-tawvm2's
converse and is not a safety hazard under this unit's hazard analysis.

**One member of the group is not fail-closed, and it is named.** An earlier form of LLR-g2jeqd's
ordering clause — the record is updated only after the Wire message has been sent —
guards a real divergence: the chain submission happens *first*, so a push that
fails leaves the Organisation's root advanced on chain while the administrator's
own record still describes the previous membership. Updating the record anyway
would make the administrator believe a member was admitted who was never told.
**That order is the resolved problem report on the publish path writing the chain before the record's defect, and this file does not endorse it.** The
hazard register calls it the publish-before-persist hazard, S3/P2
(`org-node/docs/risk/2026-09-09-org-node-hazards.md`), and its not-minted
control is "persist before push". A failed send leaves the chain one epoch
ahead of the administrator's record, and the next submission from that record
is refused by the contract's compare-and-swap. Earlier forms of LLR-g2jeqd and LLR-34c5y3 stated
the order and named the resolved problem report on the publish path writing the chain before the record, and a failed-push test in
`org-node/tests/admission_sender.rs` pinned it. *(Note 2026-10-06, change
`worktree-org-node-chain-authority`: the commit now precedes the send
(the item LLR-g2jeqd replaced, amended), so a failed send leaves the record committed with the
chain; that test is rewritten as
`a_failed_send_leaves_the_committed_record_in_place`.)*
*Corrected 2026-10-05 by review round 8. This paragraph said that keeping the
record behind the chain "is the right direction of error: it under-claims
rather than over-claims", which endorsed the order the resolved problem report on the publish path writing the chain before the record books as a
defect and did not name the report. Round 6 corrected the requirements and
left this paragraph.*

**Assessment.** No new hazard; no new control. HAZ-tawvm2 keeps S3/P2.

*Amended 2026-10-05. The Change set is pushed unsigned (RC-pm9kmx
amended in place). LLR-rv4vux and earlier forms of LLR-bwtx3h and LLR-va2yqz are amended in
place: they drop the signing wording and name the Organisation public key in
the calldata. The fail-closed argument above rests on RC-6a2dke, not on a
signature, and it stands. The amended texts are assessed in
`org-node/docs/risk/2026-10-05-envelope-authenticity.md`.*

### B. The chain-operations seam — LLR-hg3xzf

assesses: LLR-hg3xzf

**What they do.** Express the chain as an object-safe trait and give the mock
shared state across clones, so the five user stories can be exercised without a
chain.

**The risk worth recording is not what they do but where they live.**
`MockChainOps` is compiled **unconditionally** — it is not behind
`test-support`, unlike `test_fixtures`. So nothing in the type system stops a
production caller from constructing `OrgService::new(store,
Box::new(MockChainOps::new()))`, which would yield a node that verifies every
Change set against an in-memory chain it controls — defeating SDD-na9nc3's
entire purpose, silently, with no error anywhere.

**Why that is accepted rather than controlled.** `app` is this unit's only
consumer, its wiring is a single construction site, and moving `MockChainOps`
behind `test-support` would be a production change inside an architecture
tooth. It is **recorded as a risk and booked**, not dismissed: the remedy is to
gate it on `test-support` as `test_fixtures` already is.

**Assessment.** No new hazard item: this is a route into HAZ-tawvm2 (committing
a Change set whose agreement with the chain is not established) rather than a
new one, and RC-6a2dke already states the control it would defeat. Probability
unchanged — it requires a deliberate miswiring, not an input.

*Amended 2026-10-08 (ruling B, change `worktree-org-io-create`).* The risk
recorded here is closed by deletion, not by gating. `ChainOps` and
`MockChainOps` are gone (their requirement superseded by an earlier form of LLR-7en38k): org-node
holds no chain seam, and every operation that judges against the chain
takes the Organisation's state as a value that org-io read. Their
replacement for tests, `test_fixtures::ChainSlots` (LLR-hg3xzf as amended),
is compiled only under `test-support`, which no production build enables.
A miswired caller would now have to pass a fabricated state value, which
is the reach org-io's chain read (its own ledgers, `org-io/docs/`) exists to
deny; that is org-io's to assess.

### C. Persona and Organisation construction — LLR-s7yu4k

assesses: LLR-s7yu4k

**What they do.** Draw two independent keypairs per Persona and persist their
seeds; derive the Persona identifier from the member verifying key; record a new
Persona as Proposed with no Organisation; persist the Organisation record and
activate the Persona on genesis.

**Assessment, item by item.**

- **The item LLR-d4gpxm and LLR-6acxu9 replaced** — two *independent* keypairs matter more since master
  `a547ff3`: org-members now refuses an Organisation in which one key is held
  twice, a Member-as-a-group key equal to its own device key included. Drawing them from
  one source would make every Persona unusable at genesis. Fail-closed, and
  caught at the first `add_member`.
- **LLR-s7yu4k** — the Persona identifier is derived from the member verifying
  key, so two Personas collide only if their keys do. This is **not** the
  MemberId question master `0f85cb9` settled (MemberIds are now drawn at
  random, never from a key, REQ-d9g6nt); the Persona identifier is local to one
  node's store and names a keypair the node holds, which is exactly what a
  key-derived handle should name. No cross-node identity rests on it.
  *Amended 2026-10-05:* the member key is now the X25519 public key of the
  member seed, not an ed25519 verifying key. LLR-s7yu4k is amended in
  place to state the sixteen-byte prefix the code uses. The assessment
  stands.
- **An earlier form of LLR-t2adzg** — a Persona wrongly recorded Active would claim a membership
  the trie does not grant. It is bounded: every access decision reads the
  Membership record, never the Persona's status, so the error is cosmetic at
  the surface and cannot grant access. Touches HAZ-45ucqx only in that the
  store holds it.
- **An earlier form of LLR-d4gpxm** — persistence on genesis. A founder whose record never reached
  disk would lose its Organisation on restart while the chain still held it.
  Availability, not integrity.

**Assessment.** No new hazard, no new control, no probability change.

### What this section does not do

It does not make the fifteen requirements traceable to anything above the design
layer — nothing above them exists to trace to. Marking them derived records that
honestly; the SRS gap quoted at the top is the real finding, and closing it is
booked rather than attempted.

## Six further derived low-level requirements — added 2026-10-04

Review round 2 of this change found that `revoke_member` — roughly a hundred
and twenty lines that remove a member from the Membership record, submit the
new root on chain, build and sign the revocation, push it, and then update and
persist the administrator's record — was refined by **no low-level requirement
at all**, and that the same held for `export_invite`, `import_invite`,
`export_join_request` and `import_join_request`. All five are named in the
interface of an item the ledger presented as fully refined.

Ten low-level requirements were written to close that. Four cite a requirement
that genuinely states them (an earlier form of LLR-6acxu9 under an earlier form of REQ-eqt23j, an earlier form of LLR-4rv4kr, and
LLR-rjg3m2 and LLR-j6j95z under REQ-d9g6nt). **Six are derived**, and for the
same reason the fifteen in the section above are: *org-node has no high-level
requirement for its administrator-side write path.* That is the SRS gap stated
at the top of the previous section, and finding it a second time — on the
revocation half of the same path, a day after finding it on the admission half
— is the strongest evidence available that it is a gap in the requirements and
not an oversight in the ledger.

### E. The out-of-band blobs — LLR-zj88e6, LLR-437fvx

assesses: LLR-zj88e6, LLR-437fvx

**What they state.** The Invite carries the device key of the administrator
persona of the Organisation it names (LLR-zj88e6); the Join request carries the
persona's Member-as-a-group key and device key as two distinct keys from that persona's
own seeds (LLR-437fvx); importing a Join request stores nothing (an earlier form of LLR-96z2k5).

**LLR-zj88e6 is the sender-side half of a control.** RC-b6mydy and an earlier form of REQ-eqt23j
put the first-admission cross-check on the *receiver*: the authenticated sender
must equal the administrator's device key the Invite names. The receiver can
only perform that check if the Invite names the right key, and nothing above
the design layer says the Invite must. If `export_invite` named the Member-as-a-group key
instead, the cross-check would compare two things that are never equal and
every first admission would be refused — fail-closed again, and loudly, which
is why this is derived rather than a gap in the control.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* There
is no receiver-side cross-check any more: RC-b6mydy and an earlier form of REQ-eqt23j are
amended in place, and nothing about the sender is checked. The administrator's
device key the Invite names is compared with nothing; a wrong key there would
refuse no admission. The assessment below (no new hazard, no new control)
stands.

The direction that would *not* be fail-closed — an Invite naming a key an
attacker controls — is not reachable by changing this function: the Invite is
built from the administrator's own persona and carried out of band, so an
attacker who can rewrite it can equally substitute the whole Invite. That is
the out-of-band channel's trust assumption (S-numbered PoC simplification, and
HAZ-tawvm2's stated boundary), not something this requirement can defend.

**LLR-437fvx and the earlier form of LLR-96z2k5 carry no hazard.** The first is a statement that
two keys are two keys; getting it wrong makes the joiner's Member-as-a-group key equal its
device key, which org-members' key-uniqueness rule then refuses at admission
(`DuplicateKey`) — fail-closed, and already covered by the owner ruling
recorded in `project_key_identity_rulings`. The second says a decode does not
write, which is true by signature: `import_join_request` takes no `&mut self`.

**Assessment.** No new hazard, no new risk control, no probability change for
any of the three.

### What this section does not do

Like the section above it, it does not make these six traceable to anything
above the design layer, because nothing above them exists to trace to. The SRS
gap is the finding; writing high-level requirements for the administrator's
write path is a requirements change with its own review, and it is booked for a
`grill-requirements` pass rather than attempted inside an architecture tooth.

**What round 2 adds to that booking is its scope.** Round 1 stated the gap
against the admission path. Round 2 found the revocation path, the invite
export and the join-request export all sitting in the same hole. The booked
pass is therefore not "write a requirement for `admit_member`" but *state what
an administrator is required to do*, for the whole write path, in one piece.

## Three further derived low-level requirements — added 2026-10-04

Review round 3 found four of the service's record lookups — `find_org`,
`find_org_mut`, `admin_persona_for_org` and `update_persona_status` — able to
ignore the identifier they are given and return whatever record comes first in
the store, with the entire gate green; and `revoke_member`'s Loopback dial
refined by no low-level requirement, although round 2 had just written the
other three clauses of that interface. Three requirements were written. All
three are derived, for the reason the two sections above give: **org-node has
no high-level requirement for its administrator-side write path.**

### G. The revocation dial (moved)

*Moved 2026-10-13 (S4, task T12,
ruling A):* the item this section assessed (a Loopback revocation is dialled
at the full address the call carries, and one offered no address is refused)
moved to org-io with the send, re-parented there to org-io's Loopback-send
requirement (no longer derived), and this assessment with it, to
`org-io/docs/risk/2026-10-06-transport.md`. Kept below
as history.

**What it states.** In Loopback mode the Member being revoked is dialled at the full
`EndpointAddr` the call carries, and a Loopback revocation offered no address
is refused with a typed error rather than attempted against a peer it cannot
name.

*Rewritten 2026-10-04 by review round 4. The first version of this section
said the refusal "is strictly fail-closed and is the better of the two
outcomes — a revocation that cannot be addressed is refused to the
administrator's face rather than silently dropped". **That was wrong, and it
was contradicted by a problem report this same change opened.** the resolved problem report on the revocation precondition order
records that the `peer_addr` check sits at `service.rs:1174` while
`submit_update` is at `:1129` *(read `:1215` and `:1170` before the org-node
type-safety change's edits to `service.rs`; corrected by its review round 7)*:
the call burns an on-chain epoch and then
refuses. The shipped test pinned it, asserting
`chain epoch == epoch_before + 1`; since 2026-10-06 (the resolved problem report on the revocation precondition order resolved) it is
`a_loopback_revocation_without_an_address_burns_no_epoch` and asserts the
opposite. Writing an assessment from the
requirement's wording instead of from the measurement, in a file whose
neighbouring section already described this exact divergence as harmful, is
the same defect this change keeps finding in its own tests, committed in prose
instead of in code.*

**The hazard it bears on.** The same availability-of-removal question as
the administrator's revocation path (earlier forms of LLR-f3hru6 and
LLR-34c5y3, assessed in a section since removed as history), **and the same
divergence**: the refusal leaves the published root without the member while the
administrator's record still holds them. That assessment called it "a denial of
service against the Organisation, caused by the administrator, with no error
anywhere", and that judgement applies here with one mitigation — here there
*is* an error, returned to the caller, so the administrator is told the
revocation failed even though the chain has already moved.

**Assessment.** No new hazard and no new control, but **not** because the
behaviour is benign. It is the hazard the administrator's revocation path already bore, reached by a
second path, and the remedy is the resolved problem report on the revocation precondition order rather than a control: `peer_addr`
is `None` or not at the function's first line, with no I/O, so this is a
precondition checked late and not a trade-off. Probability unchanged — the
ordering is as it has always been — and what this change adds is that the
ordering is now written down, measured, and pinned by a test that reddens when
it is corrected.

## Four derived low-level requirements — re-traced or added 2026-10-04 by review round 6

Earlier forms of LLR-q4puca, LLR-34c5y3 and LLR-exaa9u were written by review round 5 under
an earlier form of REQ-eqt23j, and this file's note above said they "change no
assessment". Review round 6 found that that requirement stated only the
first-admission sender check, which none of the three refined. They were
derived and were assessed here; those assessments are removed as history,
and LLR-rc74nq's remains.

### LLR-rc74nq — the update call's names and constants

assesses: LLR-rc74nq

**What it states.** `revive_update_runtime_call` builds `Revive.call` with the
field names and constants the runtime matches by name, around
`build_update_calldata`'s bytes.

**The hazard it bears on.** None directly. A wrong name or constant produces a
call the chain rejects, which is a failed publish and an availability loss. It
does not produce a wrong root, because the root is in the calldata, which
SDD-msb6xh's other requirements pin. The chain's own epoch check sits behind
it either way.

**Assessment.** No new hazard and no new control. It was moved here from
org-io's chain-connection item (moved 2026-10-07) by review round 6, because it is pure and testable, and it is now
pinned.

*(Re-checked 2026-10-05 against LLR-rc74nq as amended by the org-node
type-safety change: the function now takes a `RootHash`, an `OrgPublicKey` and
an `Epoch`, and the calldata it wraps is the same bytes for the same values,
pinned by `tests/encoding_golden.rs` and `tests/calldata_typed.rs`
(an earlier form of LLR-mkvx2k). A swap of an epoch for another number or of a key for a root no
longer compiles, which removes a route to a wrong publish rather than adding
one. The assessment stands.)*

## Two derived low-level requirements — added 2026-10-04 by review round 7

### The self-delete path and a missing record

*Retired 2026-10-13 (S4, tasks T4 and T6; owner ruling of that day on retired
items).* The item this section assessed went with the self-delete path; its
definition and its `assesses:` entry are deleted, and its ID is listed in the
S4 verification record. The assessment is kept as history.

**What it stated.** A change about an Organisation the node holds no record of
is refused, and nothing is written.

**The hazard it bears on.** None of safety. The refusal is fail-closed, so
nothing is committed from a change the node cannot verify against a record of
its own. What it costs is availability: through the app, whose only receive
loop is this path, a joiner's first admission cannot complete. That is the
verification record's Gap 20, booked for the app. The error is also misnamed
(`OrgNotOnChain`, when the chain holds the Organisation), and that is recorded
for the fix change.

**Assessment.** No new hazard and no new control.

*Counts after this section: 103 low-level requirements, 32 derived, 71
inherited. `grep -c '^satisfies: derived'` and `grep -c '^\*\*LLR-'` against
the decomposition, measured 2026-10-04.* *(Superseded by the next section.)*

## Eight derived low-level requirements added by review round 8 — 2026-10-05

Review round 8 found eight behaviours stated by no requirement and falsified
by no test. Each is now stated, carried by a test that a named mutation
reddens, and assessed here. **None introduces a hazard or a risk control.**

### LLR-ryzr8m — the mock chain's compare-and-swap

assesses: LLR-ryzr8m

**What it states.** `MockChainOps` refuses an update at the wrong epoch and
leaves its slot unchanged.

**The hazard it bears on.** None in production: the mock is test-support code.
What it bears on is the **evidence**. A mock without the check would let a
story test pass with an update the real contract would refuse, such as the
retry the resolved problem report on the publish path writing the chain before the record makes impossible. The requirement keeps the mock as strict as
the contract on that one property.
*(Amended 2026-10-08, ruling B, change `worktree-org-io-create`: the mock is
now the test-support value store `test_fixtures::ChainSlots`, whose
`apply_update` keeps the compare-and-swap; the assessment stands.)*

**Assessment.** No new hazard and no new control.

### The unchecked length prefix (moved)

*Moved 2026-10-13 (S4, task T10,
ruling A):* the item this section assessed moved to org-io with the
transport, re-parented there to org-io's stream requirement (no longer
derived), and this assessment with it, to
`org-io/docs/risk/2026-10-06-transport.md`. Kept below
as history, the items named in prose.

**What it states.** `recv_one` discards the four-byte prefix without
comparing it with the body, and a stream shorter than four bytes is decoded
whole.

**The hazard it bears on.** HAZ-5f9jcm, the malformed or oversized frame, and it
does not weaken it. The read is bounded before anything is decoded
(the stream read cap, org-io's since 2026-10-13), and `decode_body` refuses an oversize or malformed body with a
typed error. A prefix that disagrees with the body changes neither the bytes
read nor the bytes decoded. A peer cannot use it to make the receiver
allocate more, accept a body it would otherwise refuse, or read a second
message from one stream.

**Assessment.** No new hazard and no new control. Checking the prefix would
add a refusal, not a defence.

*Counts after this section: 111 low-level requirements, 40 derived, 71
inherited. `grep -c '^satisfies: derived'` and `grep -c '^\*\*LLR-'` against
the decomposition, measured 2026-10-05.*
