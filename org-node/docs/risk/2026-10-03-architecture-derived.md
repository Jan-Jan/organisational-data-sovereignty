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
74. The three problem reports round 5 opened (PR-mdv38y, PR-8qsnhx, PR-xwek5e)
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

## LLR-3q63zv — the revocation path does not cross-check the sender

assesses: LLR-3q63zv

**What it records.** `receive_and_self_delete_if_revoked` authenticates the
remote device key through the QUIC handshake like every other receive path, and
then deliberately does not compare it against the Membership record — the
source says so at the line (`let _ = remote_device_key;`).

**Why it is derived rather than a gap.** It is not an omission this change
discovered; it is a carve-out **RC-b6mydy already states**. That control
requires the sender check "for every later Wire message about that Organisation
received through the Receive operation that admits the node or updates its
record — **as distinct from the Receive operation that acts on the node's own
removal**". REQ-ztdza4 carries the same exclusion in the same words. So the
code, the requirement and the control agree, and what was missing was only a
design-level statement of the agreement. This LLR is that statement.

**Why the carve-out is sound.** The check it omits is *the sender's device key
must be present in the Membership record its Envelope verified into*. On the
removal path the node has just been removed from that record. If the remover is
still a member the check could in principle be made against the new record —
but the node cannot distinguish "I was removed and the remover is a member"
from "I was removed and the record no longer lists either of us" without
trusting a record it is no longer part of. Requiring the check would make a
node's ability to learn of its own removal depend on membership it no longer
has, which inverts the property.

**The residual risk, stated rather than implied.** Everything the Wire message claims is still verified: SDD-na9nc3's eight checks all run first, including
the decisive match against the root the chain reports at a newer epoch
(LLR-vw2jn6 fixes that ordering). So a Wire message on this path cannot cause a
self-delete unless the chain itself says the node is out, **or** unless
PR-mdv38y has rewritten the Persona binding this path consults. In that case a
genuine update that leaves the node in the record makes it self-delete, and
`pr_mdv38y_the_receive_path_rebinds_another_organisations_persona` pins
exactly that. *The exception was added 2026-10-05 by review round 8, which
found this sentence contradicted by a pinned report it did not name.* What is **not**
defended is *who told the node*: any peer that can reach the endpoint and relay
a genuine, chain-anchored revocation envelope will cause the node to act on it.
That is a denial-of-timing rather than a forgery — the removal is real and the
node would learn of it eventually — and it is already within HAZ-ep6uzs's
description ("a device the record does not list pushes an update the node acts
on").

**Assessment.** HAZ-ep6uzs keeps severity S3 and probability P2. No new hazard;
no new control; RC-b6mydy needs no amendment because it already excludes this
path in terms. The residual risk above is **newly written down** rather than
newly created, and it belongs with the P2 re-derivation this unit already owes
for HAZ-ep6uzs, booked 2026-10-03 in
`docs/plans/2026-09-05-ratchet-setup.md`.

*Corrected 2026-10-04 by review round 7. "The code, the requirement and the
control agree" holds only for the branch where the node is removed, which is
the only branch this section argues. On the `UpdatedNotRevoked` branch the
node is not being removed, the reason given does not apply, and the missing
cross-check is PR-u4c2vp, open since 2026-09-09 and named nowhere in this
file until now. The hazard analysis already counts it against RC-b6mydy, so
the assessment above stands for the delete branch, and the update branch's
risk is the one PR-u4c2vp carries. LLR-3q63zv now names the report, and a pin
test reddens when it is fixed.*

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
`[u8; 32]`, so `{:?}` of a record prints them in clear. That is PR-hqwpg9,
which arrived with the `24317e3` merge after this sentence was written. It
does not change this LLR's assessment, because `OrgId` holds no secret, but
the sentence must not be read as saying the secrets cannot reach a log.)*
*(Re-checked 2026-10-05 by the org-node type-safety change, at its merge of
this file. PR-hqwpg9 is resolved there: the records now hold the seeds and the
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
high-level requirement that does not state them: thirteen cited REQ-nhe2zu,
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

That gap is booked in `docs/plans/2026-09-05-ratchet-setup.md` for a
`grill-requirements` pass of its own. It is not closed here, because writing
high-level requirements is a requirements change with its own review, not a
sub-task of an architecture tooth.

### A. The write path — LLR-rv4vux, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb, LLR-rb8r65, LLR-ghja3x, LLR-t4znbk

assesses: LLR-rv4vux, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb, LLR-rb8r65, LLR-ghja3x, LLR-t4znbk

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

**One member of the group is not fail-closed, and it is named.** LLR-t4znbk's
ordering clause — the record is updated only after the Wire message has been sent —
guards a real divergence: the chain submission happens *first*, so a push that
fails leaves the Organisation's root advanced on chain while the administrator's
own record still describes the previous membership. Updating the record anyway
would make the administrator believe a member was admitted who was never told.
**That order is PR-vt244s's defect, and this file does not endorse it.** The
hazard register calls it the publish-before-persist hazard, S3/P2
(`org-node/docs/risk/2026-09-09-org-node-hazards.md`), and its not-minted
control is "persist before push". A failed send leaves the chain one epoch
ahead of the administrator's record, and the next submission from that record
is refused by the contract's compare-and-swap. LLR-t4znbk and LLR-qg9utu state
the order and name PR-vt244s, and
`a_failed_push_leaves_the_administrators_record_where_it_was` pins it.
*Corrected 2026-10-05 by review round 8. This paragraph said that keeping the
record behind the chain "is the right direction of error: it under-claims
rather than over-claims", which endorsed the order PR-vt244s books as a
defect and did not name the report. Round 6 corrected the requirements and
left this paragraph.*

**Assessment.** No new hazard; no new control. HAZ-tawvm2 keeps S3/P2.

### B. The chain-operations seam — LLR-65py3d, LLR-hg3xzf

assesses: LLR-65py3d, LLR-hg3xzf

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

### C. Persona and Organisation construction — LLR-tev8h8, LLR-s7yu4k, LLR-v82xds, LLR-68yd3j

assesses: LLR-tev8h8, LLR-s7yu4k, LLR-v82xds, LLR-68yd3j

**What they do.** Draw two independent keypairs per Persona and persist their
seeds; derive the Persona identifier from the member verifying key; record a new
Persona as Proposed with no Organisation; persist the Organisation record and
activate the Persona on genesis.

**Assessment, item by item.**

- **LLR-tev8h8** — two *independent* keypairs matter more since master
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
- **LLR-v82xds** — a Persona wrongly recorded Active would claim a membership
  the trie does not grant. It is bounded: every access decision reads the
  Membership record, never the Persona's status, so the error is cosmetic at
  the surface and cannot grant access. Touches HAZ-45ucqx only in that the
  store holds it.
- **LLR-68yd3j** — persistence on genesis. A founder whose record never reached
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
that genuinely states them (LLR-9zfnmb under REQ-xa6smf, LLR-ag9mgm and
LLR-rjg3m2 and LLR-j6j95z under REQ-d9g6nt). **Six are derived**, and for the
same reason the fifteen in the section above are: *org-node has no high-level
requirement for its administrator-side write path.* That is the SRS gap stated
at the top of the previous section, and finding it a second time — on the
revocation half of the same path, a day after finding it on the admission half
— is the strongest evidence available that it is a gap in the requirements and
not an oversight in the ledger.

### D. The administrator's revocation path — LLR-6dc598, LLR-tax3pm, LLR-qg9utu

assesses: LLR-6dc598, LLR-tax3pm, LLR-qg9utu

These are the exact twins, on the removal side, of LLR-rb8r65, LLR-ghja3x and
LLR-t4znbk on the admission side, and they are derived for the same reason:
REQ-uxv2x2 governs what a node does when it *receives* its own removal, and
nothing above the design layer says what the administrator does to cause one.

**What they state.** The chain root moves before the revocation is sent
(LLR-6dc598); the envelope is signed by the administrator's Member-as-a-group key and
carries the mark one past the record's last (LLR-tax3pm); the record is updated
and the store written after the send (LLR-qg9utu).

**The hazard they bear on.** HAZ-tawvm2 — a Change set accepted on its sender's
word — is a receiver-side hazard, and these are sender-side behaviours that
make the receiver's checks *satisfiable*: a revocation whose anchor is not yet
on chain is one RC-6a2dke will refuse, and a revocation signed with the device
key is one RC-b6mydy's author check will refuse. So the failure mode here is
**a revocation that does not take effect**, not one that takes effect
wrongly — availability of the removal, not integrity of the record.

**That is not negligible.** A revocation that silently fails is a member who
believes they have been removed and has not been, and the administrator has no
signal: `revoke_member` returned `Ok`. But it is a *fail-closed* failure on the
receiver's side — the revoked node refuses a malformed revocation rather than
acting on a forged one — and the existing controls are what refuse it.

**The one that is not fail-closed is LLR-qg9utu**, and it is why the sweep
mattered: with the `save` deleted, the administrator's own record diverges from
the chain. The chain says the member is gone; A's record says they are present;
A's next change is built on a trie that no longer matches the on-chain root,
and every subsequent change A signs is refused by every receiver. That is a
denial of service against the Organisation, caused by the administrator, with
no error anywhere. It was unevidenced until this round.

**Assessment.** No new hazard and no new risk control. LLR-qg9utu's failure
mode is a new *path* to the integrity question RC-6a2dke already covers — the
local record and the on-chain root disagreeing — and the remedy is the test,
not a control.

*Clarified 2026-10-04 by review round 5, which read this against PR-b9wab3's
"a liveness failure … not an integrity one" and found the two documents
apparently disagreeing about the same divergence. They describe its two
sides. RC-6a2dke is what keeps **integrity**: every receiver checks against
the chain, so the stale administrator record is never accepted anywhere. The
**cost** of the divergence is liveness: the administrator's next change is
refused by everyone. "A path to the integrity question" names the control that
holds. PR-b9wab3 names the consequence that remains. Neither document claims
integrity is lost.* Probability unchanged: the behaviour is in the code and always
has been; what changed is that a mutation to it now reddens
`a_revocation_reaches_the_administrators_disk`.

### E. The out-of-band blobs — LLR-zj88e6, LLR-437fvx, LLR-836z24

assesses: LLR-zj88e6, LLR-437fvx, LLR-836z24

**What they state.** The Invite carries the device key of the administrator
persona of the Organisation it names (LLR-zj88e6); the Join request carries the
persona's Member-as-a-group key and device key as two distinct keys from that persona's
own seeds (LLR-437fvx); importing a Join request stores nothing (LLR-836z24).

**LLR-zj88e6 is the sender-side half of a control.** RC-b6mydy and REQ-xa6smf
put the first-admission cross-check on the *receiver*: the authenticated sender
must equal the administrator's device key the Invite names. The receiver can
only perform that check if the Invite names the right key, and nothing above
the design layer says the Invite must. If `export_invite` named the Member-as-a-group key
instead, the cross-check would compare two things that are never equal and
every first admission would be refused — fail-closed again, and loudly, which
is why this is derived rather than a gap in the control.

The direction that would *not* be fail-closed — an Invite naming a key an
attacker controls — is not reachable by changing this function: the Invite is
built from the administrator's own persona and carried out of band, so an
attacker who can rewrite it can equally substitute the whole Invite. That is
the out-of-band channel's trust assumption (S-numbered PoC simplification, and
HAZ-tawvm2's stated boundary), not something this requirement can defend.

**LLR-437fvx and LLR-836z24 carry no hazard.** The first is a statement that
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

### F. The lookups act on the record the caller named — LLR-vdyu65, LLR-w3fhhg

assesses: LLR-vdyu65, LLR-w3fhhg

**What they state.** `admit_member` acts on the Organisation record the
`org_id` argument names and signs with that Organisation's administrator
Persona (LLR-vdyu65); `create_organisation` marks the named Persona Active and
binds it to the Organisation it founded, leaving every other Persona alone
(LLR-w3fhhg).

**The failure they bear on is a confused-deputy, and it is the most serious
thing this change's reviews found.** An administrator holding two Organisations
calls `admit_member(org_2, …)`. With the lookups as the gate measured them, the
joiner is minted into **org_1's** Membership record, org_1's new root is
submitted on chain at the next epoch, and the envelope is signed by org_1's
administrator Persona. A member is admitted to an Organisation nobody asked to
admit them to, with a valid on-chain anchor and a valid signature, and every
receiver in org_1 will correctly accept it. There is no malformed input here
and no attacker: the administrator's own call does it.

**Against which control.** HAZ-tawvm2 is "a Change set accepted on its sender's
word", and RC-6a2dke's answer is the independent on-chain root. **Neither
helps**, because the root this writes *is* independent and *is* correct for the
change it actually made. The control assumes the administrator's intent is
faithfully carried to the record; nothing above the design layer says it is.
That is the gap, and it is why these are derived rather than tracing to
REQ-nhe2zu or REQ-xa6smf, neither of which mentions *which* Organisation a
write lands on.

**Assessment.** No new hazard is raised and no new control is written here, but
this is the one place in these three sections where that conclusion is close.
The reason it stays no-new-hazard is scope: the PoC's app binds one Persona to
one Organisation per device (Phase 2), so the two-Organisation state these
requirements govern is reachable through the library and not through the
shipped application. **That is a property of today's caller, not of this
unit**, and it is exactly the kind of argument this change has repeatedly
convicted elsewhere — so it is written down as the assumption it is. If the app
ever holds two Organisations on one device, this needs a risk control and a
high-level requirement, and it should be raised at the `grill-requirements`
pass already booked for the write path rather than inherited silently.

*Corrected 2026-10-04 by review round 5, which flagged the scope argument as
unmeasured. Measured: **the premise is false.** Nothing in `app` binds one
Persona to one Organisation. `create_persona`, `create_organisation` and
`import_invite` in `app/src-tauri/src/commands.rs` take no account of what
the device already holds, and the UI's `CreateOrg` and `Invite` flows can each
be run again. The flows create one Persona per Organisation by habit, and
nothing enforces it. So the two-Organisation state is reachable through the
shipped application.*

*The conclusion still holds, now for a reason that can be checked. The lookups
these requirements govern are evidenced on a two-Organisation store on both
sides: `a_second_organisation_is_admitted_into_without_touching_the_first` for
the administrator, and the three two-Organisation receiver tests round 4
wrote. The multi-Organisation defects review round 5 found are booked with
their own hazard notes, not assessed here: PR-mdv38y, PR-8qsnhx and
PR-xwek5e. Of those, PR-8qsnhx is reachable through the app, whose admission
command calls `admit_member`. PR-mdv38y's rebinding step is not, because the
app receives only on `receive_and_self_delete_if_revoked`
(`app/src-tauri/src/commands.rs:339`).*
Probability unchanged today; the behaviour is in the code and always has been,
and what changed is that a mutation to it now reddens
`a_second_organisation_is_admitted_into_without_touching_the_first`.

### G. The revocation dial — LLR-pw369n

assesses: LLR-pw369n

**What it states.** In Loopback mode the Member being revoked is dialled at the full
`EndpointAddr` the call carries, and a Loopback revocation offered no address
is refused with a typed error rather than attempted against a peer it cannot
name.

*Rewritten 2026-10-04 by review round 4. The first version of this section
said the refusal "is strictly fail-closed and is the better of the two
outcomes — a revocation that cannot be addressed is refused to the
administrator's face rather than silently dropped". **That was wrong, and it
was contradicted by a problem report this same change opened.** PR-b9wab3
records that the `peer_addr` check sits at `service.rs:1174` while
`submit_update` is at `:1129` *(read `:1215` and `:1170` before the org-node
type-safety change's edits to `service.rs`; corrected by its review round 7)*:
the call burns an on-chain epoch and then
refuses. The shipped test pins it —
`a_loopback_revocation_with_no_peer_address_is_refused_and_records_nothing`
asserts `chain epoch == epoch_before + 1`. Writing an assessment from the
requirement's wording instead of from the measurement, in a file whose
neighbouring section already described this exact divergence as harmful, is
the same defect this change keeps finding in its own tests, committed in prose
instead of in code.*

**The hazard it bears on.** The same availability-of-removal question as
LLR-6dc598 and LLR-qg9utu in section D, **and the same divergence section D
names**: the refusal leaves the published root without the member while the
administrator's record still holds them. Section D calls that "a denial of
service against the Organisation, caused by the administrator, with no error
anywhere", and that judgement applies here with one mitigation — here there
*is* an error, returned to the caller, so the administrator is told the
revocation failed even though the chain has already moved.

**Assessment.** No new hazard and no new control, but **not** because the
behaviour is benign. It is the hazard section D already assesses, reached by a
second path, and the remedy is PR-b9wab3 rather than a control: `peer_addr`
is `None` or not at the function's first line, with no I/O, so this is a
precondition checked late and not a trade-off. Probability unchanged — the
ordering is as it has always been — and what this change adds is that the
ordering is now written down, measured, and pinned by a test that reddens when
it is corrected.

## Four derived low-level requirements — re-traced or added 2026-10-04 by review round 6

LLR-xq9nrq, LLR-ckk5nz and LLR-e5c9ud were written by review round 5 under
`satisfies: REQ-xa6smf`, and this file's note above said they "change no
assessment". Review round 6 found that REQ-xa6smf states only the
first-admission sender check, which none of the three refines. They are
derived, and are assessed here.

### LLR-xq9nrq — the administrator's Member-as-a-group key on a member's record

assesses: LLR-xq9nrq

**What it states.** On first admission the new record holds, as its
administrator's Member-as-a-group key, the Published signing key read from the
chain, never a value from the Wire message.

**The hazard it bears on.** HAZ-tawvm2, a Change set accepted on its sender's
word. The stored key is read by `admin_persona_for_org`
(`org-node/src/service.rs:1403`), which decides whether this device
administers the Organisation. `export_invite` reads it too (`:682`), but only
after that lookup (`:667`) *(read `:1473`, `:697` and `:682` before the
org-node type-safety change's edits to `service.rs`; corrected by its review
round 7)*, so on a member's device it refuses with "admin
persona not found for org" and never hands an invite on. *Corrected
2026-10-05 by review round 8, which measured that refusal: this said the key
was read "when a member re-shares an invite".* A key taken from the Wire message would let a sender make a member believe it administers, or
hand on an invite naming the sender. Taking it from the chain is the same
independent source RC-6a2dke relies on.

**Assessment.** No new hazard and no new control: the behaviour is the safe
one, and is now evidenced. Zeroing the write was green until review round 5,
and `a_first_admission_records_the_signing_key_the_secret_and_the_member`
reddens it now.

### LLR-ckk5nz — the Organisation secret, stored and overwritten

assesses: LLR-ckk5nz

**What it states.** First admission stores the secret the Wire message
carries. A later update overwrites it with whatever the Wire message carries, including nothing.

**The hazard it bears on.** HAZ-ep6uzs, the organisation secret in transit and
at rest, and through it the revoked-member exposure in HAZ-vxabf9. The
overwrite is PR-xwek5e: a remaining member that receives another member's
revocation loses the secret. That is an availability loss to the member, not a
confidentiality one. Nothing leaks, because the secret is replaced by nothing.

**Assessment.** No new hazard. The confidentiality question HAZ-ep6uzs
assesses is unchanged by an overwrite with `None`. The availability loss is
booked as PR-xwek5e, and **no control is minted here**, because the intended
behaviour is unruled. Keep, clear and rotate each imply a different control,
and rotation is what `revokedExcludedFromOrgSecret` in
`org-node/quint/protocol.qnt` points at. The control follows the owner's
ruling.

### LLR-e5c9ud — which Persona a receive marks Active

assesses: LLR-e5c9ud

**What it states.** The Persona marked Active is one whose device key is in
the verified trie and is not the administrator's. It is given the member id
and the received change's Organisation, whatever it was bound to before.

**The hazard it bears on.** HAZ-vxabf9 through RC-wqgm2p, the cooperative
self-delete. PR-mdv38y and its second writer show the binding this sets being
used by the self-delete path to decide membership. A rebound device can be
pinned in an Organisation it was removed from, or delete one it is still in.

**Assessment.** **No new hazard, but the existing control is weakened**, and
that is recorded here rather than in a new control. RC-wqgm2p already carries
residual risk "not acceptable" in the hazard analysis, and PR-mdv38y is a
further way it fails. The remedy is the fix PR-mdv38y describes, after the
owner decides the Persona-to-Organisation model. Probability is not
re-estimated here. The precondition is an administrator enrolling a device key
the member uses in another Organisation, or the member founding one, and that
estimate belongs with the fix.

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
SDD-z85ux9 by review round 6, because it is pure and testable, and it is now
pinned.

*(Re-checked 2026-10-05 against LLR-rc74nq as amended by the org-node
type-safety change: the function now takes a `RootHash`, an `OrgPublicKey` and
an `Epoch`, and the calldata it wraps is the same bytes for the same values,
pinned by `tests/encoding_golden.rs` and `tests/calldata_typed.rs`
(LLR-ayrdr8). A swap of an epoch for another number or of a key for a root no
longer compiles, which removes a route to a wrong publish rather than adding
one. The assessment stands.)*

## Two derived low-level requirements — added 2026-10-04 by review round 7

### LLR-mbjfq8 — a first admission with no invite

assesses: LLR-mbjfq8

**What it states.** With no imported invite, a first admission is accepted on
the chain anchor and the signature alone.

**The hazard it bears on.** HAZ-ep6uzs and RC-b6mydy. The hazard analysis
already records this as the first of three places where RC-b6mydy is weaker
than its wording. Without an invite there is no expected sender to compare
against, and making the invite mandatory is a not-minted control there.

**Assessment.** No new hazard and no new control. The behaviour and its
weakness were assessed on 2026-09-09. What changes is that the ledger now
states it and a test pins it, so a future change that makes the invite
mandatory reddens a named test instead of passing silently.

### LLR-379hnv — the self-delete path and a missing record

assesses: LLR-379hnv

**What it states.** A change about an Organisation the node holds no record of
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

### LLR-8hdu9x — what a revocation's Wire message carries

assesses: LLR-8hdu9x

**What it states.** `revoke_member` sends the signed envelope, the member
snapshots from before the removal, and no Organisation secret.

**The hazard it bears on.** Two. RC-gfn6kr's frame-bound argument and
PR-w88sr9 both rest on the snapshot being sent, so a revocation's frame is as
large as the record it removes from. That was already assessed; this states it.
The absent secret is what PR-xwek5e's receiver writes over the secret it held.
That is an availability loss on one path, already booked, with its intended
behaviour unruled.

**Assessment.** No new hazard and no new control.

### LLR-qezw3n — the Invite's dialling address

assesses: LLR-qezw3n

**What it states.** `export_invite` carries the bound endpoint's address,
whichever Persona bound it, or none.

**The hazard it bears on.** None of integrity. The joiner pins the first
admission's sender to the Invite's device key (LLR-j83kc8), not to its address,
so a wrong address cannot make a joiner accept a wrong sender. A wrong address
makes the joiner's dial-back fail, which is availability. Where the endpoint
was bound from another Persona, that failure is PR-8qsnhx's, already booked.

**Assessment.** No new hazard and no new control.

### LLR-dzte8x, LLR-3v5nu9, LLR-drgdy8 — the pure-proxy account

assesses: LLR-dzte8x, LLR-3v5nu9, LLR-drgdy8

**What they state.** `create_organisation` keeps the proxy account the chain
returned, on disk, and `admit_member` and `revoke_member` hand it to every
update.

**The hazard it bears on.** None of integrity. The proxy account decides
**whether** an update can be submitted, not what a receiver accepts.
SDD-na9nc3 re-derives every root independently. A wrong or missing account
makes the production client fail to find the proxy after a restart, or address
the wrong one, and the chain refuses the extrinsic. That is the availability
cost already assessed for the write path in section A.

**Assessment.** No new hazard and no new control.

### LLR-q3aj8z — the founding Persona's member id

assesses: LLR-q3aj8z

**What it states.** `create_organisation` leaves the Persona's member id as it
was: none on a new Persona, and a stale one on PR-mdv38y's second-writer path.

**The hazard it bears on.** None of integrity. The member id on a Persona
record is not consulted by any decision the unit makes about membership. The
sender cross-checks, the self-delete decision and administration all use keys
and the record's snapshots. A stale id is a display error on the PR-mdv38y
path, which is already booked.

**Assessment.** No new hazard and no new control.

### LLR-ryzr8m — the mock chain's compare-and-swap

assesses: LLR-ryzr8m

**What it states.** `MockChainOps` refuses an update at the wrong epoch and
leaves its slot unchanged.

**The hazard it bears on.** None in production: the mock is test-support code.
What it bears on is the **evidence**. A mock without the check would let a
story test pass with an update the real contract would refuse, such as the
retry PR-vt244s makes impossible. The requirement keeps the mock as strict as
the contract on that one property.

**Assessment.** No new hazard and no new control.

### LLR-2smrvx — the unchecked length prefix

assesses: LLR-2smrvx

**What it states.** `recv_one` discards the four-byte prefix without
comparing it with the body, and a stream shorter than four bytes is decoded
whole.

**The hazard it bears on.** HAZ-5f9jcm, the malformed or oversized frame, and it
does not weaken it. The read is bounded before anything is decoded
(LLR-k2y6nn), and `decode_body` refuses an oversize or malformed body with a
typed error. A prefix that disagrees with the body changes neither the bytes
read nor the bytes decoded. A peer cannot use it to make the receiver
allocate more, accept a body it would otherwise refuse, or read a second
message from one stream.

**Assessment.** No new hazard and no new control. Checking the prefix would
add a refusal, not a defence.

*Counts after this section: 111 low-level requirements, 40 derived, 71
inherited. `grep -c '^satisfies: derived'` and `grep -c '^\*\*LLR-'` against
the decomposition, measured 2026-10-05.*
