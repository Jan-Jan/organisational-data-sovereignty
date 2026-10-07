# Risk analysis — chain authority (org-node)

Run 2026-10-05 for change `worktree-org-node-chain-authority`, under IEC 62304
Class C and the acceptability matrix in this ledger's README. The owner's
rulings are recorded in
`org-node/docs/requirements/2026-10-06-chain-authority.md`:
an Envelope carries no signature, nothing about its sender is checked, org-node
has no administrator, it builds provisional updates and no longer writes to the
chain. The items this changes in place are HAZ-tawvm2, RC-pm9kmx, HAZ-ep6uzs
and RC-b6mydy, each amended with a dated note in
`org-node/docs/risk/2026-09-09-org-node-hazards.md`.

## What removing the signature and the sender checks does to the existing hazards

**HAZ-tawvm2** (a Change set committed on its sender's word) keeps three of its
four controls unchanged: RC-6a2dke (the root recomputed against a root the node
read from the chain itself), RC-e5atck (a strictly newer epoch) and RC-m4r75s
(a strictly newer Sequence number). The signature RC-pm9kmx required added
nothing those three do not: only the Organisation's multisig can place a root
on the chain, and the signing key was one Member's. RC-pm9kmx keeps its
Organisation-binding clause. Bytes from any peer now reach the decoder, which
RC-e2uvje (decode never panics, fuzzed) already covers. Severity and
probability are unchanged (S3, P2 before controls); the residual is as the
register states it.

**HAZ-vxabf9** loses RC-b6mydy's second clause as its "non-cooperative half".
That clause refused a removed device's messages, but the only messages it
refused that the root match does not are chain-valid updates, which are
harmless whoever delivers them. Its residual is unchanged.

**HAZ-ep6uzs** loses its control. A relay can substitute the Organisation
secret in an admission, on first admission and after, until the key-pair
change checks a received key against the chain. The owner accepted that window
(2026-10-05): the code is not deployed. Residual: **not acceptable** until the
key-pair change lands.

## A new hazard: any peer can make the node read the chain

**HAZ-bedm57**: with no signature to refuse a stranger's Envelope cheaply, any
peer that can reach the node sends Envelopes that each make the node read the
chain before refusing them; the node's single receive loop is occupied while a
genuine update or revocation for a Member waits behind them; a removed device's
holder keeps reading material published after its removal for longer, and the
identities in it reach them. Severity: S3. Probability: P2.

P2: reaching the node needs only its address, and the Envelope is hostile input
by construction.

**RC-mj6gjq**: for an Organisation it holds a record of, the node runs every
check that needs no chain — the Organisation named, the Sequence number, the
Envelope and Change set decoding, and the Change set's base against its
record's Membership root — before it reads the chain, and refuses without
reading the chain an Envelope that fails any of them. mitigates: HAZ-bedm57

**RC-2ferct**: for a first admission, the node reads the chain only when the
Organisation named is one the app has declared it expects to join, because
its user entered an Invite for it, and refuses any other first admission
without reading the chain. mitigates: HAZ-bedm57

What these controls break. RC-mj6gjq reorders the checks so that a Change set
is decoded before the chain is read; that was already the order inside
verification, and only the node's own chain read moves later. RC-2ferct
refuses a chain-valid first admission the user did not ask for; that is the
owner's journey (the user entered the Invite) and not a loss. A user who
cleared the expectation before admission arrived must enter the Invite again.

Residual: with both controls a stranger must know an Organisation's current
Membership root and next Sequence number, or an Organisation this node expects
to join; the first is known to every Member, the second to the inviter.
Severity S3, probability **P1**: acceptable under the matrix.

*Note 2026-10-07 (S3a close-out residual review, change
`worktree-org-io-commit-workflow`; owner rulings R1 and R6, decision 16 of
`docs/plans/2026-10-06-org-io-commit-workflow.md`).* Under owner ruling D of
2026-10-06 the README's matrix governs and S3 is unacceptable at every
probability, so "acceptable under the matrix" above is restated as S3/P1 —
unacceptable under the matrix (ruling D). A new control now stands beside
RC-mj6gjq and RC-2ferct: RC-u7kdam (owner ruling R1) refuses, before any
chain read and without writing, an update or a revocation for an
Organisation the node holds whose sending Device — authenticated by the
transport as its DevicePublicKey — is not listed in the node's current
committed record. Residual with it in place: a stranger can no longer make a
node that holds a record read the chain; only a listed Member Device can,
and a Member's chain reads are what the node does anyway. What remains is the
first admission, which by the owner's amendment of 2026-10-07 is accepted
from any sender ("to avoid scenarios where something happens to the admin's
device during this window"; "The invite id plays no role in the update"):
a stranger can send a node that is waiting to join first-admission updates,
each verified against the chain, and so occupy that one node until it is
admitted. That node holds no Organisation information yet, so nothing of an
Organisation's Members is exposed by it; the harm is a delayed admission (a
denial of service to one joiner), and the S3 harm this hazard names — a
removed device's holder keeping published material for longer — cannot
arise on a node that is not yet a Member. Severity S3, probability P1 for
the held-record part (a listed Member Device is needed); the first-admission
part is a delay to one joiner. *Owner ruling R6 (2026-10-07): residual
accepted.* The owner's words: "Strangers cannot make the node read the
chain, because org updates come from other members (not strangers), even the
new joiner knows the device key of the admin. Furthermore, member devices
will be tracking the chain in any case for updates, so this hazard is
irrelevant." And on the first admission, the same day: "the new joiner has
no org information to disclose, and they verify the org information they
receive on-chain so the risk here is only a new joiner being DoS'ed which is
acceptable" — the denial of service to a new joiner by strangers sending
first-admission updates is owner-accepted on 2026-10-07 on that reasoning.
(Recorded for the owner's attention: the clause "even the new joiner knows
the device key of the admin" predates the same day's amendment that lets a
first admission come from any sender.)

*Note 2026-10-07 on RC-2ferct (same review).* RC-2ferct and REQ-8amu2a
above still state that the node reads the chain for a first admission only
for an Organisation the app declared it expects; that is the code today and
is not changed here. The owner's ruling on the update itself: "The invite id
plays no role in the update." The acceptance of the first-admission residual
above rests on the chain check and on the joiner having nothing to disclose,
not on this control.

*Owner ruling 2026-10-07 on RC-2ferct (the join gate; S3a close-out,
change `worktree-org-io-commit-workflow`).* RC-2ferct and REQ-8amu2a are
**kept**. The node reads the chain for a first admission only for an
Organisation the app expects to join, because its user entered the Invite;
any other first admission is refused without a chain read. The update
itself is judged on the chain alone — "the invite id plays no role in the
update" — so the gate decides only whether the node reads the chain, never
whether a chain-valid update is accepted. Kept because it narrows the
denial of service to a new joiner that the owner accepted above: a stranger
can occupy only a node that is waiting to join, and only for the
Organisation that node's user entered an Invite for.

## Derived requirements assessment

**REQ-ag6kqm, REQ-nhe2zu** (amended: no signature; the chain alone decides):
remove a control's clause, assessed above under HAZ-tawvm2. No new hazard:
the remaining controls cover the hazardous situation the signature did. The
loss of the cheap refusal is HAZ-bedm57.

**REQ-xa6smf, REQ-ztdza4** (amended: commit whoever sent it): remove
RC-b6mydy's two clauses, assessed above under HAZ-ep6uzs and HAZ-vxabf9.
HAZ-ep6uzs's residual worsens until the key-pair change (owner-accepted).

*Note 2026-10-07 (owner ruling R1, change
`worktree-org-io-commit-workflow`).* REQ-ztdza4 is amended again: an update
for an Organisation the node holds is committed only from a Device listed in
its current record (RC-u7kdam); REQ-xa6smf is unchanged. The assessment of
that amendment is in
`org-node/docs/risk/2026-10-07-commit-workflow.md`.

**REQ-d9g6nt** (amended: "founding Member") and **REQ-qn2erx** (amended:
the Join request and Invite imports leave org-node): no hazard impact. The
first is wording; the second moves the parse of those two inputs to the app,
whose register assesses it.

**REQ-xs4ab8** (provisional updates, no chain write): no new hazard. It removes
one: the node no longer holds the multisig signatory key, so a compromise of
its store no longer yields the power to change the chain.

**REQ-tqap3r** (the node commits its own provisional update only after it
verifies against the chain): no new hazard, and it narrows the
publish-before-persist hazard filed as PR-vt244s. A sender's record no longer
runs ahead of the chain, and once the chain carries the root the sender
commits by the same checks a receiver uses, so its record is not left an epoch
behind by a failed send, provided the commit does not wait on the send (a
design constraint for `design-architecture`).

**REQ-uv3v5w** (a commit discards the old record and every provisional update
it orphans) and **REQ-fwfku9** (provisional updates bounded at 1 MiB per
Organisation): no new hazard. Persisting provisional updates (REQ-xs4ab8 as
amended) puts more of a Persona store's contents in the encrypted file, under
RC-jjsz97 like the rest; discarding the orphans keeps a node from offering an
update that can no longer apply, and the bound keeps the store from growing
without limit. Persisting also means a restart between the app's submission
and the node's commit no longer loses the update; PR-vt244s nonetheless stays
open by owner ruling (2026-10-05).

**REQ-f2k4tr, REQ-8amu2a** realise RC-mj6gjq and RC-2ferct; assessed above
under "What these controls break". REQ-8amu2a was amended on 2026-10-06 to
match on the invite identifier as well as the Organisation, and to keep every
expectation when it refuses: a relay holding someone else's genuine admission
for the same Organisation carries another invite identifier, so it is refused
without a chain read and the waiting node's expectation survives. That
strengthens RC-2ferct; it breaks nothing a user relies on.

**REQ-kt877x** (a first admission commits only if the verified record lists
one of this node's own Personas): no new hazard. It closes pre-emption, by
owner ruling (2026-10-06): a node would otherwise commit a record it is not
in, which denies it its own admission until the record is repaired by hand.
Denial of one's own admission delays a Member's access; it exposes nothing.
The refusal keeps the expectation, so the genuine admission still commits
when it arrives.

assesses: REQ-ag6kqm, REQ-nhe2zu, REQ-xa6smf, REQ-ztdza4, REQ-d9g6nt, REQ-qn2erx, REQ-xs4ab8, REQ-tqap3r, REQ-f2k4tr, REQ-8amu2a, REQ-uv3v5w, REQ-fwfku9, REQ-kt877x

## Derived low-level requirements of the design (design-architecture, 2026-10-05)

The design step wrote one new derived low-level requirement and rewrote
twenty-nine derived ones in place
(`org-node/docs/architecture/2026-10-06-chain-authority.md`,
`2026-10-03-decomposition.md`, `2026-10-04-type-safety.md`). Every other new
or rewritten requirement satisfies one of the requirements assessed above.

**LLR-2xzys9** (`send_update` binds the endpoint from the first Persona bound
to the Organisation the Envelope names): no new hazard. It replaces the
administrator-Persona lookup by key, which the owner removed; the endpoint's
identity is not an input to any receiver's decision now (nothing about a
sender is checked), so binding from a different Persona of the same device
could misroute a send but not make a receiver commit anything the chain does
not carry. PR-8qsnhx's bind-once defect is unchanged.

**LLR-e58j8m, LLR-9fvb3y** (a keypair rebuilt from its seed has the same keys;
another seed's has different ones): no hazard impact. They state the custody
property the signature requirements used to carry, now that nothing signs;
they constrain the seeds the store already protects (RC-jjsz97).

**LLR-rv4vux, LLR-rc74nq, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj,
LLR-f74xwb, LLR-65py3d** (org-node holds no calldata, no multisig, no chain
write): no new hazard in this unit; each removes code. The behaviour moves to
on-chain-client, whose register assesses it
(`on-chain-client/docs/risk/2026-10-06-chain-write.md`).
Removing the signatory key from org-node's store is a reduction, as assessed
under REQ-xs4ab8 above.

**LLR-ryzr8m** (the mock's compare-and-swap, now on inherent methods): no
hazard impact; a test stand-in, unchanged in what it imitates.

**LLR-w3fhhg, LLR-dzte8x, LLR-q3aj8z** (the record's creation, Persona
binding and proxy account move from `create_organisation` to
`commit_genesis`): no new hazard. They now happen only after the genesis has
verified against the chain (REQ-tqap3r), which narrows the window in which a
record exists that the chain does not. The proxy account is opaque data the
app supplies and org-node never interprets; a wrong one is a failed
submission in the app, not a wrong commit, because every commit is checked
against the chain. PR-mdv38y's second writer is unchanged *(closed 2026-10-06
by REQ-yp75u9; see below)*.

**LLR-vdyu65, LLR-3v5nu9, LLR-drgdy8** (admission and revocation act on the
named Organisation; the proxy account is handed back, unused): no new hazard;
the wrong-record hazard LLR-vdyu65 was written for is unchanged in kind.

**LLR-zj88e6, LLR-qezw3n, LLR-437fvx, LLR-836z24** (no Invite, no dialling
address, no Join request; `persona_public_keys` returns the two keys): no new
hazard in this unit. The parse of the Invite and its reply moves to the app's
edge, which the app's register assesses; the dialling address that
PR-8qsnhx's reading side concerned no longer exists.

**LLR-xq9nrq, LLR-e5c9ud** (no administrator key in the record; the Persona
marked Active is the first whose DevicePublicKey is in the trie): no new hazard.
The administrator-key exclusion was never evidenced and selected nothing a
device holding one Persona per Organisation could observe; PR-mdv38y's
selection defect is unchanged *(closed 2026-10-06 by REQ-yp75u9; see below)*.

**LLR-ckk5nz** (secret overwritten on a received update; kept on a node's own
commit): HAZ-ep6uzs's residual is as assessed above (owner-accepted until the
key-pair change); keeping the node's own secret on its own commit removes no
control.

**LLR-379hnv** (an unheld Organisation refused on the self-delete path before
any chain read): realises RC-mj6gjq's intent on that path; no new hazard.

**LLR-pw369n, LLR-8hdu9x** (the send refuses a Loopback call with no address;
it carries the secret its caller passes): no new hazard. The refusal now
follows a commit the chain already agrees with, so it can no longer burn an
epoch (PR-b9wab3's mechanism); the secret carried is as it was.

**LLR-s7whrn, LLR-ayrdr8** (the chain account is opaque in org-node; the
golden encodings drop the Invite, the Join request and the calldata): no
hazard impact; both restate the types and encodings that remain.

assesses: LLR-2xzys9, LLR-e58j8m, LLR-9fvb3y, LLR-rv4vux, LLR-rc74nq, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb, LLR-65py3d, LLR-ryzr8m, LLR-w3fhhg, LLR-dzte8x, LLR-q3aj8z, LLR-vdyu65, LLR-3v5nu9, LLR-drgdy8, LLR-zj88e6, LLR-qezw3n, LLR-437fvx, LLR-836z24, LLR-xq9nrq, LLR-e5c9ud, LLR-ckk5nz, LLR-379hnv, LLR-pw369n, LLR-8hdu9x, LLR-s7whrn, LLR-ayrdr8

## Derived low-level requirements re-traced or amended after the merge of master (2026-10-06)

Master `5f7c177` (the person-types switch) was merged into this change; the
design was revised against it
(`org-node/docs/architecture/2026-10-06-chain-authority.md`,
note of 2026-10-06). The four requirements it adds — LLR-ms8njy and LLR-48jakr
(the invite identifier on the wire, REQ-8amu2a), LLR-3f5h7b (REQ-kt877x) and
LLR-qjz3q4 (the genesis provisional update keeps the Organisation private key
until it commits, REQ-ech45n) — each satisfy a requirement assessed above or
in `org-node/docs/risk/2026-10-05-envelope-authenticity.md`. LLR-qjz3q4 puts
the Organisation private key in one more place in the store, under the same
encryption (RC-jjsz97) and the same redacted secret type as the record's copy
(LLR-322xfu); a genesis never committed takes the key with it when its update
is discarded, so no key outlives the Organisation it was drawn for.
LLR-2xzys9, assessed above, only gains the invite identifier among its
arguments; its assessment stands.

**LLR-ctzkv7** (a Persona's member key is the X25519 key of its member seed,
its DevicePublicKey the ed25519 key of its device seed), re-traced to derived: no
hazard impact. It states how two keys the store already protects are
computed; the X25519 rule is `person`'s, assessed there, and no decision of
this unit rests on a sender's key any more (REQ-ztdza4 as amended).

**LLR-g9vmbx** (org-node defines no Invite), re-traced to derived: no new
hazard in this unit. It removes code; the Invite's parse moves to the app's
edge, which the app's register assesses (its hazard of an Invite from someone
who is not a Member), and a joining node keeps
an expectation in its place (RC-2ferct, assessed above).

**LLR-rys5nx** (a first admission's record holds the chain's Organisation
public key and no administrator key), amended: no new hazard. Dropping the
administrator key removes a value nothing verifies against; the key the record
keeps is still the chain's, read in the same operation, never one from the
Wire message, as master's assessment of this item relied on.

assesses: LLR-ctzkv7, LLR-g9vmbx, LLR-rys5nx

**REQ-hhva9d** (discard a provisional update on request, refusing an unknown
one): no new hazard, and it narrows one. Without it a genesis that is never
written to the chain keeps its Organisation private key in the encrypted store
indefinitely, under RC-jjsz97 but for no purpose; discarding removes it. The
operation touches no record and no expectation, so it cannot undo a commit or
cost a node its admission; discarding the update that would have committed
only means the app must build it again. Late genesis (owner ruling
2026-10-06): a founding node whose genesis the chain has moved past refuses to
commit it and joins as any Member does; that delays the founder's own record
and exposes nothing.

assesses: REQ-hhva9d

**REQ-yp75u9** (one Persona, one Organisation; added after independent review
round 1, finding-1): no new hazard, and it removes one this change had
introduced. Once org-node found the acting Persona through its single
Organisation binding rather than by an administrator key, a second founding —
or a reply to an Invite made with a Persona already in use — moved that
binding. The first Organisation then had no Persona to act through, and the
next update received for it failed the own-membership check and deleted its
record, its Organisation private key and its proxy account: the loss of an
Organisation's on-chain write capability and of the key every Member needs,
with no attacker required. Refusing reuse at creation, and never rebinding on
admission or commit, makes that path unreachable. What it breaks: founding a
second Organisation needs a new Persona first, which is the owner's model
(keys per Organisation). It also closes PR-mdv38y.

assesses: REQ-yp75u9

LLR-6z5xya and LLR-eyc4ud refine REQ-yp75u9 and are not derived. The derived
items they change, amended in place on 2026-10-06 (task R1a):

**LLR-w3fhhg, LLR-q3aj8z, LLR-e5c9ud** (`commit_genesis` binds only an
unbound Persona, whose member id is therefore none; a receive marks only a
Persona the commit may bind): no new hazard. Each loses the clause that
stated PR-mdv38y's rebinding, which is the hazardous path REQ-yp75u9 closes;
what remains selects and binds a strict subset of what it did. The one new
outcome is a refusal: a first admission that lists only Personas bound
elsewhere is refused as `AdmissionNotOurs` instead of committed, which leaves
the node out of that Organisation until it replies with an unbound Persona —
a delay the app can show, not a loss.

assesses: LLR-w3fhhg, LLR-q3aj8z, LLR-e5c9ud
