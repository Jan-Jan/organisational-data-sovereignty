# Hazard analysis — the org-node unit

org-node's first hazard enumeration, run under IEC 62304 **Class C**
(`docs/adr/2026-09-05-units-and-per-unit-classes.md`) and evaluated against
the acceptability matrix in this ledger's README. It is tooth 3 of
`docs/plans/2026-09-05-ratchet-gap-analysis.md`; the plan for the change is
`docs/plans/2026-09-09-org-node-risk-analysis.md`.

## Scope

The capability analysed is **the node that carries an Organisation's
membership between the chain and its devices**: receiving a Change set from a
peer and deciding whether to commit it, publishing a new Membership root to the
chain and telling the affected device, admitting and revoking through the
invite and join-request exchange, authenticating peers on the iroh transport,
and keeping the keys and the record at rest in the persona store. Concretely
that is every module of `org-node/src`: `verify.rs`, `sequence.rs`,
`envelope.rs`, `service.rs`, `chain.rs`, `chain_read.rs`, `chain_write/`,
`ceremony.rs`, `transport/`, `blobs.rs`, `store.rs`, `keys.rs`, `ids.rs`,
`error.rs`, `preflight.rs`. `chain.rs` is not an aside: it declares the
`ChainReader` trait and the `MockChain` behind it, and is the seam RC-6a2dke's
independent chain read arrives through, implemented in production by
`ChainOpsReader` (in `org-node/src/service.rs`). The remaining three
files in the tree are not modules of the capability: `lib.rs` is module
declarations and re-exports, `test_fixtures.rs` is the test-support fixtures
(discussed under RC-e2uvje), and `bin/preflight.rs` is the CLI wrapper around
`preflight.rs`.

The owner decided on 2026-09-08 that this change covers the whole unit rather
than only the eight places org-members' hazard analysis
(`org-members/docs/risk/2026-09-02-membership-hazards.md`) names org-node.
Those eight are re-analysed here as org-node's own hazards. org-members'
register keeps its wording and carries dated 2026-09-09 corrections wherever
this change makes it out of date, each naming what remains open: the residual
risk of its base-matching control, the bullet claiming org-node is under no
gate (which was already wrong on 2026-09-05), the evidence paragraph for its
arbitrary-bytes fuzz target, the residual risk of its replay hazard, the rows
of its residual-risk table that summarise those shortfalls, and the
not-minted controls this change closes in part or in whole. The corrections
add to that text rather than rewriting it; the one word replaced is a problem
report's ID, which now names the file that holds the report instead (see "The
problem report that moved"), because a provider may not cite a consumer's
items. No item in that register is altered.

The hazard analysis and the requirements it mints do not have the same reach,
and the difference is deliberate. The analysis covers the whole unit: every
module listed above was surveyed and its trust boundaries walked. The fifteen
requirements in
`org-node/docs/requirements/2026-09-09-verify-and-commit.md`
realise only the nine controls this analysis chose, so `ceremony.rs`,
`keys.rs`, the blob encoding in `blobs.rs`, `chain_write/`, `chain_read.rs`
(the module PR-hvg2dy is filed against), `preflight.rs`, `ids.rs`, `error.rs`
and the transport handshake are analysed here without acquiring a requirement
of their own. Stating what those modules shall do, and the unit's low-level
requirements throughout, is tooth 4 of
`docs/plans/2026-09-05-ratchet-gap-analysis.md`.

Deliberately **outside** this analysis, and not to be read as assessed:

- **What membership grants.** The CGKA/ACL layer (Phase 3) that turns a
  Member-as-a-group key and the organisation secret into decryption
  capability. Every disclosure pathway below ends at "holds the key or the
  secret"; what that key opens is the next capability's analysis.
- **The membership rules themselves** — handle validity, device bounds,
  base-root matching, the trie. Those are org-members' behaviour, analysed in
  its register and stated in its exported requirements, which this file cites
  by ID where org-node leans on them.
- **What the chain returns.** on-chain-client decodes the Organisation state;
  what org-node needed from it was stated as an expectation, assessed in
  `2026-09-06-dependency-expectations.md`; since 2026-10-07 it is the
  finalised-block expectation now held by org-io (`org-io/docs/requirements/`).
- **The administrator's surface.** How the app shows a member before an act
  on them, and how it collects the passphrase and the invite. The app is its
  own unit and its analysis follows this one.

## Method, and what the harms are

The chain walked is ISO 14971's: hazard, hazardous situation, harm, severity,
probability, evaluation, controls, residual risk. Candidates were sought by
walking every trust boundary of the crate — the wire, the chain, the blob
exchange, the disk — and asking at each what arrives, what is checked before
it is acted on, and what is left behind on failure. The survey of the code
that fed this is recorded in the plan.

The two harms are the two the class C ADR names and org-members' register
argues (`docs/adr/2026-09-01-safety-class-c.md`): **disclosure** — someone
reads organisational material they should not, and in the journalism and
government deployments that material identifies a person — and
**unavailability** — a member cannot reach material at the moment a decision
needs it. Both are **S3** at their worst credible outcome. As in org-members'
register, every probability below is the probability of the **hazardous
situation arising**; the situation-to-harm step is not estimated and no number
carries it.

Severity is uniform and uncomfortable: every hazard below is S3, and the
matrix makes S3 unacceptable at every probability. That is the correct result
for the component that holds the keys and does the I/O, not a defect in the
estimates; what it means for acceptability is settled in the residual-risk
section.

## Where the controls come from, and what they are not

Nine controls are minted below. Each is worded from behaviour org-node already
has, and each is realised by a requirement in
`org-node/docs/requirements/2026-09-09-verify-and-commit.md`
carrying `(implements: RC-…)`. Until this change those behaviours were tested
by unit tests inside `org-node/src`, which no gate read; the change relocates
them into `org-node/tests`, where `test_paths` points, and adds the
abnormal-input cases class C requires. No behaviour of the crate is changed.

Two of org-node's requirements predate this file and are **expectations** on
its providers: the finalised-block expectation now held by org-io
(`org-io/docs/requirements/`) (on-chain-client returns the Finalised Organisation state when
no block is named; org-node's until 2026-10-07) and REQ-q92yac (org-members
rejects a Change set that removes a Device key without replacing the
Member-as-a-group key). Each is named below, and the two stand in different
relations to the controls. The finalised-block expectation's subject *is* the decisive input to
RC-6a2dke: the chain read that control compares against, so the control is
only as good as the expectation. REQ-q92yac's subject is not a half of any
control here — RC-wqgm2p is fully implemented and tested in this unit — but
one of the uncontrolled residual counts of HAZ-vxabf9: a Change set that
removes a Device key and leaves the Member-as-a-group key it held would be
applied, and only org-members can refuse it. Neither carries
`(implements: RC-…)` in this change:
an unmet expectation that implements a control fails every run of this
unit's gate, and the providers' 90 days run to 2026-12-05. When a provider
delivers, the follow-up that closes the expectation adds the annotation. The
owner took that decision on 2026-09-08.

Three hazards are the shape of a **defect** rather than a missing control:
behaviour the code has that is wrong on an ordinary failure — the publish path
writing the chain before the local record (**PR-vt244s**), loopback admission
delivering the organisation secret to an address unbound to the joiner's
Device key (**PR-2dmjzj**), and the revocation receive path committing a
record from a sender it never checked (**PR-u4c2vp**). All three are filed as
problem reports in this unit's ledger
(`org-node/docs/problems/2026-09-09-org-node-problems.md`)
and recorded below in prose, with their fixes under "Controls identified but
not minted" as not-minted controls 1, 2 and 4 respectively. Fixing them here
would mix a hazard analysis with a change to `service.rs`; the owner chose
problem reports on 2026-09-08, the same call org-members' register made for
its own unimplemented clause.

## Hazards, and the controls chosen for them

### A Change set accepted on its sender's word

**HAZ-tawvm2**: the node could commit a Change set whose agreement with the
chain it has not established itself — a Membership root taken from the message
or from a path the sender controls, a signature not checked or checked against
a key the sender chose, an envelope built for a different Organisation, or an
Organisation state older than the one already committed; a peer, or a device
whose keys were taken, delivers an envelope that admits a principal the
Organisation never admitted or reverses a removal, and the node commits it;
that principal holds membership and the access that follows from it, and
material identifying a source or a protected person reaches them. Severity:
S3. Probability: P2.

P2: any device that can reach the node can deliver an envelope, the transport
authenticates a device key and nothing more (`org-node/src/transport/endpoint.rs:5`),
and the envelope is hostile input by construction.

*Amended 2026-10-05 (change `worktree-org-node-chain-authority`):* "a
signature not checked or checked against a key the sender chose" no longer
names a check the node makes; the envelope carries no signature and the chain
is the sole authority (RC-pm9kmx as amended). The hazard is otherwise
unchanged.

**RC-6a2dke**: the node commits a Change set only when the Membership root
recomputed by applying it to the node's own record equals the Membership root
of the Organisation state the node itself read from the chain in the same
receive operation; the root it compares against is never taken from the
message, its sender, or a value the sender can influence, and a Change set for
an Organisation the chain does not know is rejected. mitigates: HAZ-tawvm2

**RC-pm9kmx**: before a received Change set is decoded, the node requires the
Envelope to name the Organisation the node expected, and rejects one that
does not with a typed error and without decoding the Change set; it checks
no signature and no key of the sender or of any Member or device, and
leaves authority over the Change set to the Membership root and epoch it
reads from the chain (RC-6a2dke, RC-e5atck). mitigates: HAZ-tawvm2

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`;
text written by change `worktree-person-shared-types`, which merges first).*
This control first also required a valid signature over the Organisation
identifier, the Sequence number and the Change set bytes by the published
signing key. The right to change an Organisation's data lies in its on-chain
multisig proxy, so the Envelope carries no signature and the chain is the
sole authority.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This control first also required a valid signature by the Published signing
key. The right to change an Organisation lies in its on-chain multisig proxy,
org-node has no administrator, and the signing key was one Member's, so the
signature added nothing RC-6a2dke, RC-e5atck and RC-m4r75s do not. The
assessment is in
`org-node/docs/risk/2026-10-06-chain-authority.md`.

**RC-e5atck**: the node commits a Change set only when the epoch of the
Organisation state it verified against is strictly greater than the epoch of
its last commit for that Organisation, so that a chain state older than one
already acted on is never the basis of a commit. mitigates: HAZ-tawvm2

How the three sit in the code. `verify_envelope_against_chain`
(`org-node/src/verify.rs:45-88`, *re-resolved 2026-10-05 after the trim (docs/plans/2026-10-05-switch-trim.md), review round 3 finding-16; it read `:53-100` after the merge of master `1feb608`, `:52-99` on this
branch before it and `:47-88` on master*) checks Organisation binding, then the
signature, then the sequence number, then decodes the Change set, then
requires its declared base to be the local root, applies it, reads the
Organisation state through the `ChainReader` it was handed, requires the epoch
to be strictly newer, and only then requires the recomputed root to equal the
chain's. The order is deliberate: authenticity is settled before any work is
done on attacker-chosen bytes. The reader it is handed in production is built
by `receive_and_verify` from the node's own `read_state` call, made in the same
operation and before verification (`org-node/src/service.rs`: the
`read_state` call in `receive_and_verify`, the `ChainOpsReader` it builds from
that state and hands to `verify_envelope_against_chain`, and the adapter
`ChainOpsReader` itself); the author key is the `org_pub_key` of that same
Organisation state (`author_vk` in `receive_and_verify`), never a key the
message carries. The
independence org-members' register asks its consumer to supply — its
"independent trusted root" — is therefore structural here, and
RC-6a2dke is the requirement it was missing. Two further facts the requirement
text leans on: `verify_envelope_against_chain` never mutates the local record,
and `receive_and_verify` reaches its store only after it returns `Ok`
(the commit block at the end of `receive_and_verify` in
`org-node/src/service.rs`), so a rejected envelope leaves nothing
behind but a consumed connection.

*Amended 2026-10-05 (RC-pm9kmx amended in place).* Step 2 of the order
above, the signature check, is gone, and nothing replaces it: nothing about
the sender is checked. The order of the rest is unchanged: the cheap checks
still run before any attacker-chosen bytes are decoded, and RC-6a2dke and
RC-e5atck still decide. There is no author key, and `author_vk` no longer
exists: the chain's `org_pub_key` is the Organisation public key, an X25519
key that authenticates nothing the node receives.

The base-root check at step 5 is org-members' rule (REQ-4umsuz, exported) and
is not re-minted here; it is what makes a Change set applicable to exactly one
record state, and the root match at step 8 is what makes that record state the
one the chain committed.

Residual risk: reduced, **not acceptable**, on two counts that are not
org-node's to close alone.

The Organisation state is only as trustworthy as the read that produced it.
`SubxtChainOps::read_state` asks on-chain-client for the state with no block
named (`org-node/src/service.rs:408-418`), which on-chain-client documents as
the latest Finalised block — a doc-comment, not a commitment, which is why
the finalised-block expectation now held by org-io (`org-io/docs/requirements/`) exists.
Until on-chain-client states it as an exported requirement,
the decisive input to RC-6a2dke rests on a comment. The same read is what the
chain-reader problem report that moved into this ledger is about: the
node's own `OnChainReader::refresh` documents it as "current best"
(`org-node/src/chain_read.rs:34-35` **as this paragraph was written**;
corrected 2026-10-03, and line 34 now carries the opposite assertion — see the
resolution note at the end of this hazard), contradicting both the client and
`preflight.rs`. Its reach is narrower than an earlier draft of this paragraph
claimed, and narrower is the honest word: `receive_and_verify` does not use
this reader, and neither does preflight, which calls
`OrgRegistryClient::get_org_state(admin, None)` itself and documents it
correctly as the latest finalised block (`org-node/src/preflight.rs:107`,
`:114`). Outside `org-node/src` the cached reader is constructed in exactly
one place, `org-node/tests/chain_genesis_e2e.rs` — a chopsticks-dependent test
the merge gate does not run — and in no production path and no gated path. So
the defect's blast radius is small: no shipped behaviour depends on the wrong
comment today. The report stays open on the grounds it already gives, which
that does not touch: the doc-comment of a security-relevant control says
"current best" where the read is finalised, and a second reader written from
that comment would implement the weaker read. (Corrected 2026-10-03: the
report is resolved — `refresh()`'s doc-comment now states the finalised read and
the caching, and the line reference above is to the comment as it then stood.)

**What the resolution above does not change**, added here by the transport
change (`worktree-org-node-loopback-timeout`) rather than by the change that
resolved the report: the decisive input to RC-6a2dke still rests on a
doc-comment rather than on an exported requirement, because that comment is
on-chain-client's and the finalised-block expectation now held by org-io
(`org-io/docs/requirements/`) is still open. Correcting org-node's
description of the read did not move that, and the paragraph above is left
standing because it is the assessment made of RC-6a2dke while the contradiction
existed — an assessment that quietly becomes a description of a fixed tree
stops being evidence of anything.

And the published signing key is a single ed25519 member key: `org_pub_key`
is the admin's Member-as-a-group key, derived at genesis
(`create_organisation` in `org-node/src/service.rs`), submitted to the chain
with the genesis root (its `submit_genesis` call) and persisted in the record
(its `OrgRecord`), so exactly one author exists per
Organisation and there is no admin role, quorum or rotation for it. Whoever
holds that seed and the chain account authors membership. That is the
admin-authority hazard in prose below.

*(Note 2026-10-05, review round 3 finding-16, change
`worktree-person-shared-types`: this paragraph no longer holds. `org_pub_key`
is the Organisation public key, the X25519 public half of a secret drawn for
the Organisation alone and distinct from every genesis key (REQ-ech45n), and
no Envelope is signed (REQ-ag6kqm, RC-pm9kmx amended in place). Who may move
the membership root is decided on-chain by the Organisation's multisig proxy;
the paragraph is kept as the assessment made of the signing design.)*

### A superseded envelope applied again

**HAZ-p4gfv9**: the Organisation's committed root can return to a value it
previously held, at which point a retained envelope whose Change set declares
that root as its base becomes applicable again; a peer redelivers an envelope
that re-admits a removed member or restores a Device key; the node applies it
a second time, and the removed member's access returns without any
administrator intending it. Severity: S3. Probability: P1.

P1, for the reason org-members' register gives for the same pathway on its
side: it needs an administrator to republish an earlier root, by mistake or
under coercion, and a retained copy of the superseded envelope. Base-root
matching (REQ-4umsuz) rejects a stale Change set only while the record has
moved past its parent, and this is exactly the case where it has moved back.

**RC-m4r75s**: the node accepts an envelope only if its sequence number is
strictly greater than the highest sequence number it has committed for that
Organisation, checks this before the Change set is decoded, and advances that
high-water mark only after the envelope has fully verified — never on a
rejection, and never backwards. mitigates: HAZ-p4gfv9

`SeqGuard::check` is step 3 of the verify order and does not mutate;
`SeqGuard::advance` is called only after the root match and ignores a value
not greater than the mark (`org-node/src/sequence.rs:28-47`,
`org-node/src/verify.rs:56`, `:86`; *re-resolved 2026-10-05 after the trim, review round 3 finding-16; they read `:68`, `:98` after the merge of master `1feb608`, `:27-46`, `:67`, `:97` on this branch before it and `:28-47`, `:62`, `:86` on master*). The guard's own documentation states
why the placement matters: advancing on `check` rather than on commit would
move the watermark for an envelope still to be rejected, which is a replay
bypass. The mark persists as `OrgRecord.last_seq` and is reloaded on the next
receive (`receive_and_verify` in `org-node/src/service.rs`, which loads
`last_seq` from the existing record and builds its `SeqGuard` from it).

Residual risk: reduced, **not acceptable**, because the control closes the
pathway for one node and not for the Organisation. The sequence number is
per-node state: a device that never saw the envelope the first time has no
mark to refuse it against, and a device whose store was restored from a backup
has an old one. The Organisation-level control is that the chain's epoch only
moves forward (RC-e5atck), which refuses an envelope verified against an older
Organisation state but not one whose root the chain has genuinely returned to.
Closing that is the registry contract's business (a no-op guard that rejects
only the current pair is the whole bar today), which is outside every unit.

### The node brought down by hostile input

**HAZ-5f9jcm**: an envelope, a wire frame or a Change set can be malformed,
hostile or oversized; the node panics inside decode or verification and takes
the device's membership function down with it, or is handed a frame large
enough to exhaust its memory; a member cannot receive a membership change or
act on the record at the moment a decision needs it. Severity: S3.
Probability: P2.

Severity is the worst credible harm of the unavailability pathway, as argued
in "Method". P2: the input is attacker-chosen and the surface is a decoder
reachable by anything that can open a QUIC connection to the node.

**RC-e2uvje**: every rejected envelope and Change set yields a typed error,
and no sequence of envelope bytes, however malformed, causes a panic in
decoding or in verification against the chain. mitigates: HAZ-5f9jcm

**RC-gfn6kr**: a wire frame whose body exceeds 1 MiB is rejected with a typed
error before any of its bytes are decoded, on send as on receive. mitigates:
HAZ-5f9jcm

The structural support for RC-e2uvje is worded the same as org-members' and
carries less weight, so all of it is stated here. The crate denies `unwrap`,
`expect` and `panic` twice over — at the crate root (`org-node/src/lib.rs:2`)
and through the workspace lints it opts into (`Cargo.toml:22-25`,
`org-node/Cargo.toml:70-71`). The survey found the allows to be `#[cfg(test)]`
modules, which never compile into a build of the library, with one exception:
`org-node/src/test_fixtures.rs:10` allows `unwrap` and `expect` for the whole
module, and that module is compiled into the library under the `test-support`
feature (`org-node/src/lib.rs:43-44`), not only under `cfg(test)`. The feature
is never enabled in a production build, so no shipped binary contains it, and
the lib the merge gate tests is therefore not quite the lib that ships.
The larger shortfall is that **nothing checks any of this**: no
`verify_commands` entry runs clippy, and the CI `clippy` job lints org-members
and on-chain-client only (`.github/workflows/rust.yml:80-87`). The denial is
asserted, not enforced, and the proof is this change itself: exposing
`test_fixtures` under `test-support` put `unwrap` and `expect` into library
code in breach of the crate's own deny, every gate passed green, and it took
review round 1 to find it. Running clippy on this crate is a not-minted
control below. The behavioural evidence is the two bolero targets,
`org-node/tests/fuzz_envelope_decode` (arbitrary bytes into the envelope
decoder and then into org-members' Change set decoder) and
`org-node/tests/fuzz_verify_against_chain` (arbitrary envelope bytes through
the whole verify pipeline against an honest record and chain, establishing
that no such byte sequence panics). The second target also asserts the
chain's root on anything it accepts, but acceptance needs fuzzed bytes
carrying a valid signature by the honest key and a candidate root equal to the
seeded one, so that branch is not reached in practice and the target is
evidence of panic-freedom only; the root-match property is established instead
by `rejects_root_mismatch_when_chain_root_differs` and
`happy_path_commits_when_root_matches_chain` in
`org-node/tests/verify_against_chain.rs`. Both have run at every merge since
2026-09-05 through this unit's `verify_commands`; what they lacked was a
`verifies:` annotation, which this change adds. The bound in RC-gfn6kr is
`MAX_FRAME` (`org-node/src/transport/mod.rs:47`), enforced on encode, on
decode, and by the `read_to_end` limit of the receiving stream
(`org-node/src/transport/wire.rs:28-30`, `:39-41`,
`org-node/src/transport/endpoint.rs:334`; *re-resolved 2026-10-04 by the
org-node architecture change's review round 5 — it read `:300`*).

*Correction, 2026-10-04 (org-node architecture change, review round 4): "that
branch is not reached in practice" was true when written and was true for a
second reason no one had noticed — the target seeded its chain with the
pre-update root, so no input could be accepted at all. The target now seeds
the root an honest update produces and builds its envelopes rather than
decoding them from fuzz bytes; a `panic!()` on the accept path and a check 8
weakened to compare against the candidate's own root both redden it. The
target is therefore evidence of the root-match assertion as well as of
panic-freedom. The two deterministic tests named above remain the primary
evidence. Detail in `docs/verification/2026-10-03-worktree-guardrails-org-node-arch.md`.*

Residual risk: reduced, **not acceptable**. The fuzz targets run under
bolero's generative engine for one second each at the merge gate, which finds
shallow crashes and nothing deep; libFuzzer runs are a separate manual
invocation with an empty seed corpus. And the bound is on the frame only: the
base64 blobs of the invite and join-request exchange have no size limit
(`org-node/src/blobs.rs:64-78`, *re-resolved 2026-10-05 at the merge of master `1feb608`; it read `:36-50` on this branch and `:63-76` on master*), the envelope's own Change set bytes are
bounded only by the frame that carries them, and there is no rate limit,
allowlist or read timeout at the accept boundary — an **unauthorised**
stranger, one the record does not name, obtains a chain read, a record rebuild
and a signature check per connection. The handshake does authenticate the
peer's Device key (`org-node/src/transport/endpoint.rs:3-5`, `:322-323`); what
it does not do is decide whether that key belongs to a member before the work
is spent (`org-node/src/transport/endpoint.rs:313-339`, *re-resolved 2026-10-05 from `:313-342`; these two citations
re-resolved 2026-10-04 by the org-node architecture change's review round 5;
they read `:288-291` and `:279-306`, which that change's own edits to
`endpoint.rs` had moved onto `send_conn`*; in
`receive_and_verify` in `org-node/src/service.rs`, the chain read, the record
rebuild — from the node's stored record, or on a first admission the base
record rebuilt from the administrator's snapshot by `first_admission_base` —
and the signature check inside `verify_envelope_against_chain`). (Amended
2026-10-03, REQ-d9g6nt: on a first admission that carries no snapshot,
`first_admission_base` refuses the message after the chain read and before any
rebuild or signature check.) *(Amended 2026-10-05: there is no signature check and no sender check. A
stranger obtains per connection a chain read, a record rebuild and, on a
first admission, the decode of the snapshot it sent.)* Those are
not-minted controls below.
And the structural half of the control is unchecked: no gate runs clippy on
org-node, so the panic-freedom denial the paragraph above cites is a claim
about the source, re-established only by reading it. A regression is
detectable only by the fuzz targets happening to reach the offending line, or
by a reviewer. That too is a not-minted control below.

### Membership material handed to, or taken from, a device the record does not name

**HAZ-ep6uzs**: a Change set and, on admission, the organisation secret are
delivered over a connection whose remote end is authenticated as a Device key
and nothing more, and the secret is not covered by the envelope's signature;
a peer other than the administrator relays a genuine admission envelope with a
substituted secret, or a device the record does not list pushes an update the
node acts on; a device outside the Organisation's record holds the secret its
members share, or a non-member's message shapes a member's record. Severity:
S3. Probability: P2.

P2: relaying is what a network does, and the receiving device on first
admission has no record yet to check the sender against.

*Amended 2026-10-05 (change `worktree-org-node-chain-authority`):* the
envelope no longer carries a signature, and no sender is checked, so a peer
other than the one that built an admission can substitute the secret on any
admission. The hazard text above is unchanged; its residual is **not
acceptable** until the key-pair change checks a received key against the
chain (owner-accepted window, 2026-10-05).

*Amended 2026-10-06 (change `worktree-org-node-org-key-pair`):* the
Organisation secret is replaced by the Organisation private key, and RC-9cefcn
refuses any received key whose public half is not the chain's `org_pub_key`.
A substituted key is refused; the 2026-10-05 window closes. Residual S3, P1,
acceptable (`org-node/docs/risk/2026-10-07-org-key-pair.md`).

*Owner ruling of 2026-10-07 (change `worktree-org-io-commit-workflow`):*
residual accepted. Under owner ruling D of 2026-10-06 the README's matrix
wins and S3 is unacceptable at every probability, so "acceptable" above is
restated as S3/P1 — unacceptable under the matrix (ruling D); the owner
accepted that residual on 2026-10-07, on the benefit-risk case in
`org-node/docs/risk/2026-10-07-org-key-pair.md` (HAZ-ep6uzs, notes of this
date).

**RC-b6mydy**: the node commits a received Wire message only through the
checks of RC-pm9kmx, RC-6a2dke, RC-e5atck, RC-m4r75s and RC-95dgg8. On a
first admission it checks nothing about the Device key the connection
authenticated; on a later update it first requires that key to be listed in
its current committed record (RC-u7kdam), and then commits only what the
chain has published, whichever listed Device delivers it.
mitigates: HAZ-ep6uzs, HAZ-vxabf9

*Amended 2026-10-07 (owner ruling R1 at the S3a close-out residual review,
change `worktree-org-io-commit-workflow`; decision 16 of
`docs/plans/2026-10-06-org-io-commit-workflow.md`).* This said the node
"checks nothing about the Device key the connection authenticated: not on a
first admission, with or without an imported Invite, and not on a later
update, whether or not that key is in the Membership record before or after
it. A chain-valid update is committed whoever delivers it". The owner ruled
that updates and revocations are accepted only from members — "Using iroh a
connection can only be established via mutually known public keys, but I
agree with only accepting updates and revocations from members" — and, the
same day, that a new joiner's first admission stays open to any sender ("to
avoid scenarios where something happens to the admin's device during this
window"). The sender check is RC-u7kdam's
(`org-node/docs/risk/2026-10-07-commit-workflow.md`);
this control keeps the chain checks it always required.

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`;
text written by change `worktree-person-shared-types`, which merges first).*
This control first required the Device key the connection authenticated to be,
on a first admission for which an Invite was imported, the administrator's
Device key that Invite names (the invite cross-check), and, for every later
Wire message through the Receive operation that admits or updates, a Device
key present in the Membership record the message was verified into (the
post-verification membership check). The owner ruled that nothing about the
sender is checked: authority is the chain's, and a chain-valid update
delivered by any peer matches the chain.

**Residual risk: not acceptable.** What the connection delivers beside the
Change set is not covered: the Organisation secret (PR-ve9zw8) can now be
substituted by any peer that relays a genuine admission or update, not only by
a device in the record. Chain-authority's change 2 replaces the secret with the
Organisation private key, checked on receipt against the chain's
`org_pub_key`, and that is the control this hazard waits for. The owner's
earlier ruling that the secret is replaced by CGKA keys a member verifies
against the Organisation public key is recorded beside it, unranked: whether
change 2's receipt check supersedes it was not ruled (Q2 of
docs/plans/2026-10-05-switch-trim.md).

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This control first required a first admission's sender to be the
administrator's DevicePublicKey named by an imported Invite, and a later sender to
be in the verified Membership record. The owner ruled that nothing about the
sender is checked and that org-node has no administrator. What the control now
states mitigates only the part of each hazard the root match already
mitigates: HAZ-ep6uzs's substituted secret is unmitigated until the key-pair
change checks a received key against the chain (owner-accepted window), and
HAZ-vxabf9's non-cooperative half rested on refusing messages the root match
also refuses. The prose below describes the control before this amendment.
The assessment is in
`org-node/docs/risk/2026-10-06-chain-authority.md`.

*Before 2026-10-05:*

The first clause is the invite cross-check (the `is_first_admission` block in
`receive_and_verify`, `org-node/src/service.rs`):
the invite the joiner imported out of band carries the administrator's Device
key, and the connection's authenticated key must equal it. The second is the
post-verification membership check (the `sender_known` check): the sender's Device key
must appear in the record the envelope produced. Both return before the store
is written.

Residual risk: **not acceptable, and the control is weaker than its wording
in three places, each recorded here rather than asserted away.**

First, the first-admission check runs only when an invite was imported for
that Organisation; without one the code falls through to the signature and
chain proof alone, with a comment that this is "log-worthy in production but
not a hard fail" (the invite cross-check in `receive_and_verify`) in a crate
that has no
logging. RC-b6mydy is worded to say what the code does; making the invite
mandatory is a not-minted control.

Second, the revocation receive path (`receive_and_self_delete_if_revoked`)
performs neither clause: it authenticates the sender and discards the result
(`let _ = remote_device_key;` in `org-node/src/service.rs`), relying on the
admin signature and the chain
root. A revocation envelope relayed by anyone is acted on, which for a
revocation is the intended outcome and for the `UpdatedNotRevoked` branch of
the same function is a message from an unchecked sender committing a record.
That second outcome is the reason RC-b6mydy's second clause is scoped to the
Receive operation that admits or updates rather than to every Wire message,
and it is now filed as **PR-u4c2vp**.

Third — and this is the defect filed as PR-2dmjzj — in loopback transport
mode `admit_member` sends the envelope and the organisation secret to the
address the join request carried, without binding that address to the
joiner's Device key (the `TransportMode::Loopback` arm of the send in
`admit_member`, `org-node/src/service.rs`); the networked branch derives the
peer identity from the Device key and so does bind it (the
`TransportMode::Networked` arm). A join-request blob whose address was altered in transit hands
the secret to whoever the altered address names, while the record admits the
joiner's genuine keys. The blob is unsigned and unauthenticated by design
(`org-node/src/blobs.rs:12-36`, *re-resolved 2026-10-05 at the merge of master `1feb608`; it read `:11-34` on this branch and `:13-34` on master, re-measured there by the org-node type-safety change, review round 7*), so nothing upstream catches the alteration.

### A device removed from the record that keeps acting as a member

**HAZ-vxabf9**: a revocation is published to the chain and then sent, as a
Change set, to the device being revoked; a device that never receives it, or
that has been altered not to act on it, keeps its local record, its keys and
the organisation secret, and the other devices' records still list it until
they are told otherwise; the removed device continues to receive and decrypt
material published after its removal, and the identities in it reach whoever
holds the device. Severity: S3. Probability: P2.

P2, and no attacker is needed: a device that is offline at the moment of
revocation is the ordinary case for a lost or stolen device.

*Amended 2026-10-06 (change `worktree-org-node-org-key-pair`):* what a removed
device keeps is now the genuine Organisation private key, held by every
Member's Devices, but the update that removes a Device draws a fresh key pair
that only the new record's Devices receive, so a removed Device holds no key
used after its removal (residual P1, acceptable)
(`org-node/docs/risk/2026-10-07-org-key-pair.md`).

*Restated 2026-10-07 (change `worktree-org-io-commit-workflow`, at its merge
of master `1f52c36`; owner ruling D of 2026-10-06 in
`docs/plans/2026-10-06-org-io-roadmap.md`, section "Owner's rulings on the
S2–S4 design drafts").* "Residual P1, acceptable" above follows the reading
of the matrix that ruling D rejects: this hazard is S3, and S3 is
unacceptable at every probability. The residual of the key a removed Device
keeps — the Organisation private keys of the epochs up to its removal — is
S3/P1 — unacceptable under the matrix (ruling D); owner ruling pending
(flagged 2026-10-07). The owner reviews it individually (ruling of
2026-10-07); the benefit-risk case put to the owner is in
`org-node/docs/risk/2026-10-07-org-key-pair.md` (HAZ-vxabf9, note of this
date). S3 also takes from a removed Device the Change set the key-pair
change's revocation still carried (RC-r8bp43).

*Owner ruling of 2026-10-07:* residual accepted for the key itself — the
Organisation private keys of the epochs up to a Device's removal, S3/P1 as
restated above — on the benefit-risk case in
`org-node/docs/risk/2026-10-07-org-key-pair.md`. The CGKA boundary (forward
and post-compromise secrecy across rotations) is recorded as ODS Phase 3's
to address, not this stage's. The "owner ruling pending" above is settled by
this ruling.

**RC-wqgm2p**: on committing a Membership record that no longer lists its own
Device key, or on accepting an absence proof of its removal checked against
the current on-chain root, the node deletes every piece of data it holds for
that Organisation and every Persona bound to it, so that a cooperating device
stops acting on the Organisation from the moment it learns of its removal.
mitigates: HAZ-vxabf9

*Amended 2026-10-06 (owner rulings 3 and 4 on the sweep of `fdf4e77`,
`docs/plans/2026-10-06-org-io-roadmap.md`; change
`worktree-org-io-commit-workflow`).* This said the node deletes its record
and marks the persona revoked, on a committed Change set only. The absence
proof is the second verified path (stage S3), and the Persona, with its
Device and Member secret keys, is now deleted rather than marked. The control
is still cooperative; what S3 adds around it is assessed in
`org-node/docs/risk/2026-10-07-commit-workflow.md`.

RC-wqgm2p is minted with its weakness stated in its own text: it is a
cooperative control, and a control that depends on the party being excluded
is the weakest kind. It is here because it is real, tested end to end (story
5 of `org-node/tests/service_stories.rs`), and because its absence would be
worse — a device that learns of its own removal and carries on. The
non-cooperative half is RC-b6mydy's second clause on every other device: once
their records no longer list the removed Device key, its messages are refused.
*(Amended 2026-10-05: RC-b6mydy no longer has that clause. Nothing about the
sender is checked, so a removed device's messages are refused only when they
do not match the chain, as anyone's are.)*

Residual risk: **not acceptable**, on three counts, two of them outside this
unit.

The other devices' records are updated only when a Change set reaches them,
and the node pushes a Change set to the affected device alone — the admitted
member on admission, the revoked member on revocation
(the transport send in `admit_member` and in `revoke_member`,
`org-node/src/service.rs`). There is no fan-out and no
periodic re-read of the chain, so every other member's record stays at the
epoch it last received until something happens to it. That is the stale-view
hazard in prose below.

The organisation secret is never rotated. The revocation message carries
`org_secret: None` (`revoke_member` in `org-node/src/service.rs`), and the secret handed to
the member at admission stays valid for everyone who has it. Rotating it, and
distributing the new Member-as-a-group keys the rotation implies, is the CGKA
layer (Phase 3) — the same boundary org-members' register draws for its
missed-rotation hazard.

And the Change set that performs a removal is validated by org-members, on the
wire path, with nothing relating a leaf's device set to its Member-as-a-group
key. That is what REQ-q92yac asks of org-members and is due 2026-12-05; until
then a Change set that removes a Device key and leaves the key it held would
be applied here.

*Amended 2026-10-05 (review round 2, finding-12).* HAZ-vxabf9 has a further
control, RC-95dgg8 (`org-node/docs/risk/2026-10-05-envelope-authenticity.md`):
the node commits an Envelope only when its Sequence number is the epoch of
the Organisation state it read from the chain itself. It closes one route to
this hazard that the unsigned Envelope opened: any peer setting the Sequence
number to `u64::MAX`, after which the receiver refused
every later Envelope, its own removal included. The hazard is not re-scored:
the three counts above are unchanged.

### Keys and record read from the device's storage

**HAZ-45ucqx**: the persona store holds every secret the node has — each
persona's member seed and device seed, each Organisation's secret and its
record — in one file; a lost or stolen device, or another account on the same
machine, reads that file; whoever reads it can impersonate the member on the
transport, sign as the administrator if the persona is one, and hold the
Organisation's secret. Severity: S3. Probability: P2.

P2: loss or theft of a laptop is an ordinary event, and the file sits in the
application's data directory with default permissions.

**RC-jjsz97**: the persona store is written only as ciphertext under a key
derived from a passphrase, so that the file opened without the passphrase
yields an error and not data, and no member seed, device seed or
organisation secret appears in the file in clear. mitigates: HAZ-45ucqx

The store is XChaCha20-Poly1305 over the postcard-encoded data, under an
Argon2-derived key, with a fresh random nonce per save
(`org-node/src/store.rs:309-315`, `:332-355`, `:376-397`, `:408-413`). The wrong passphrase
fails authentication and returns an error.

Residual risk: **not acceptable**, and the shortfalls are the store's own.
The key-derivation salt is a fixed application constant
(`org-node/src/store.rs:332-333`, "PoC simplification S9"), so the same
passphrase derives the same key on every installation and a precomputed
table attacks every store at once. The file is written with the process's
default mode, not restricted to the owner (`:412`). *(Citations in this paragraph re-measured 2026-10-05 by the org-node type-safety change, review round 7, after that change's edits moved them.)* The decrypted data and the
derived key are not zeroised. And the passphrase's strength is the user's;
the store imposes none. The write is also not atomic — a crash between
truncation and completion leaves no readable copy — which is the storage-loss
hazard in prose below rather than a disclosure.

*Amended 2026-10-05 (review round 2, finding-22).* "Not zeroised" now has
one exception. `X25519Keypair`, which holds a member seed or an Organisation
private key in memory, overwrites it with zeros when dropped and cannot be
cloned (LLR-98ufry). Everything else stands: the decrypted store data, among
it every persisted seed and `OrgRecord.org_private_key`, the Organisation
private key, is not zeroised, and nor is the derived key. The residual verdict
does not change. *(Amended at the merge of master `1feb608`: this said those
were "plain `[u8; 32]` … (PR-hqwpg9)". Master resolved PR-hqwpg9; they are now
held in redacted secret types (`MemberSeed`, `DeviceSeed`, `OrgPrivateKey`),
which are `Clone` and not wiped on drop.)*

*Interim limitation, stated 2026-10-07 (change
`worktree-org-io-commit-workflow`, review round 1 finding-8).* Stage S3a
gives the removal paths a Device secret key for one signing operation
(REQ-y99c9w: keep no Device secret key it was given "in the store or anywhere
else once the operation returns"), but until stage S4 that key is still a
`DeviceSeed` read from the store, and copies of it outlive the operation in
freed memory: `DeviceSeed` is not wiped on drop
(`org-node/src/types.rs`, the `secret_type!` doc comment, "Wiping on drop is
not done"); the service clones each bound Persona's seed for every removal
path (`device_seeds_bound_to`, `org-node/src/service.rs`, called by
`commit_update`, `reconcile`, `receive_revocation` and `remove_self`);
`commit_step` clones the whole `StoreData` for a successor, and
`OrgService::reconcile` clones the successor `StoreData` again into the
store, seeds included. Only the ed25519 `SigningKey` derived from the seed is
wiped on drop (ed25519-dalek's `zeroize` feature, `org-node/Cargo.toml`). So
copies of Device secret keys may remain in freed process memory, where a
crash dump, a swapped page or another process reading this one's memory can
find them — this hazard's in-memory route, and REQ-y99c9w's "anywhere else"
is not met in that sense until then. REQ-y99c9w's amendment of 2026-10-07
defers the transient value's own type, move-only and zeroised, to S4, which
takes the seed from the OS keychain rather than the store; **S4 is the stage
that closes this limitation**. Until it merges, the residual above stands
widened by these copies. This interim exposure has not been put to the
owner and is not owner-accepted.

## Hazards introduced by these controls

ISO 14971 requires each control to be checked for what it breaks. Four of the
nine controls have something to record: three have an entry below, and the
fourth (RC-b6mydy) introduces none today and is noted so that it is revisited.
**None of the three is acceptable.** Each entry was defective in an earlier
draft: the frame bound put its refusal at the start of the act, the forgotten
passphrase was assessed S2 / P1 on a false analogy, and the spurious
self-delete was assessed S2 / P1 and called acceptable. Review round 5 found
the first two defects and review round 6 the third; each entry was
re-assessed in the fix round that followed the review that found it — the
reviewer found the defect and did not write the replacement. What the
re-assessments changed:
the frame bound's harm falls inside the publish-before-persist window rather
than being refused at admission; the forgotten passphrase is the same harm,
from the same file, as the keys-lost-on-crash hazard this register already
puts at S3; and the spurious self-delete's severity is the unavailability its
own harm carries, not the recoverability of the fallback. The out-of-order
rejection that an earlier draft of this section attributed to RC-m4r75s and
RC-e5atck belongs to neither and is now part of the stale-view hazard in
prose, for the reason given there.

**How the three are counted.** The test this file applies, here and in the
residual-risk section: an entry is a *distinct hazard* when its hazardous
situation and its harm are ones no hazard already counted covers, and a *new
trigger* when it is another route to a hazardous situation and harm already
counted. That is the same test that keeps two of the prose entries out of the
count below. By it, two of these three are triggers and not hazards.
The frame bound is another way of reaching the publish-before-persist hazard's
situation — the chain an epoch ahead of a record that never caught up — and
the forgotten passphrase another way of reaching the keys-lost-on-crash
hazard's — the store file unopenable and every key on the device with it.
Only the spurious self-delete is a hazardous situation this register does not
otherwise contain: a member who was never removed loses their record. So these
controls introduce **one further hazard and two further triggers**, and the
register's count of distinct hazards is twelve rather than eleven.

- **RC-gfn6kr (1 MiB frame).** A Change set larger than the frame cannot be
  delivered, which bounds the size of one membership change; and because both
  Wire messages the node sends carry the whole pre-change record as a
  snapshot — the admission message (`admit_member`, where it builds the record
  snapshot, `org-node/src/service.rs`) and the revocation message
  (`revoke_member`, likewise) — the bound is also a bound on the size of
  the Organisation either can be delivered for. Where an earlier draft of this
  entry was wrong is *when* that bound bites. It is not a refusal at the start
  of the act. On both paths the chain write comes first (the `submit_update`
  call in each); the
  send frames the message through `encode_frame`, which is where the bound is
  enforced (`org-node/src/transport/endpoint.rs:292`, *re-resolved 2026-10-04,
review round 5; it read `:258`*, the check at
  `org-node/src/transport/wire.rs:28-30`); and the local record is updated only
  after the send has returned (the record update at the end of each
  function). So for an
  Organisation whose snapshot exceeds the frame, the error the administrator
  sees arrives *after* the chain has moved — on the admission path and the
  revocation path alike — with the peer told nothing and the administrator's
  record an epoch behind the chain, which is to say every later publish
  refused. That is the publish-before-persist hazard in prose below, which this
  register assesses **S3 / P2** and files as **PR-vt244s**. By the counting
  test above this entry is a new trigger for that hazard rather than a hazard
  of its own: the hazardous situation and the harm are the ones assessed there.
  Assessed **S3 / P2**, **not acceptable**. The severity is that hazard's. The
  probability is argued here on this trigger's own terms rather than borrowed,
  because this trigger is unlike the others: an offline peer, a stale address
  or a failed bind is intermittent, whereas the frame bound is deterministic
  and permanent — below its threshold it never bites, above it every admission
  and every revocation fails, at the send, after the chain write, until the
  code changes. The threshold is a member count, and it can be worked out. A
  `MemberSnapshot` (`org-node/src/store.rs:172-179`) is a 32-byte id, a 32-byte
  member key, three strings and up to four 32-byte device keys; org-members
  caps each string at 128 bytes and the device slots at four
  (`org-members/src/types.rs`), so postcard encodes one member in at most about
  **583 bytes** — 32 + 32, three length-prefixed strings at 130 each, and
  1 + 128 for the device vector — and in about **130 bytes** for the short
  handle and name of ordinary use. Against `MAX_FRAME` of 1 MiB
  (`org-node/src/transport/mod.rs:47`) that puts the bound at roughly **1,800
  members at the maximum field lengths and about 8,000 at typical ones**:
  order 10³ either way, a few thousand people. Nothing caps a member count —
  org-members states no maximum and neither does org-node — and an
  Organisation of a few thousand people is an ordinary size in every
  deployment the class C ADR names. So **P2** is read here as the probability
  of an Organisation reaching low thousands of members, which is occasional
  rather than remote; it is not the probability of failing once there, which
  is one. That is the right reading of the number and not a borrowing of the
  parent's: the Method section above states every probability in this register
  as the probability of the hazardous situation arising, and the situation here
  is the Organisation crossing the bound. It makes P2 a floor rather than a
  guess — below the bound the trigger is inert, above it certain — and the
  verdict does not turn on the number in any case: S3 is unacceptable at every
  probability. Two controls, neither minted: the ordering fix
  PR-vt244s already specifies (not-minted control 1), which makes a failed send
  recoverable rather than leaving the Organisation wedged; and bounding or
  omitting the membership snapshot on the send path (not-minted control 15),
  which removes the failure mode rather than making it survivable — and which
  on the revocation path costs nothing at all, because
  `receive_and_self_delete_if_revoked` rebuilds its trie from its own store and
  never reads the snapshot the message carries.
- **RC-jjsz97 (passphrase).** A forgotten passphrase is a lost store: the file
  cannot be opened, every persona's keys and every Organisation's record on
  that device are gone, and the member must be re-admitted with new keys.
  Assessed **S3 / P1**, **not acceptable**. An earlier draft put this at
  S2 / P1 by analogy with org-members' device-bound lock-out, and the analogy
  does not hold: that lock-out stops a member enrolling one *more* device while
  every device they already hold keeps working, whereas this loses the keys
  they hold. The harm is unavailability, which the Method section above and the
  class C ADR (`docs/adr/2026-09-01-safety-class-c.md`) put at S3 at its worst
  credible outcome; it is the same loss, of the same file, as the
  keys-lost-on-crash hazard in prose below, which this register already
  assesses S3 / P1, and by the counting test above it is a further trigger for
  that hazard rather than a hazard of its own. Nothing about the recovery
  separates the two, and an earlier draft's claim that this one's recovery is
  "strictly worse … rather than a restored file" was wrong in both directions:
  both end in re-admission with new keys, and no backup of the store exists
  anywhere in this design — not-minted control 10 stops a crash truncating the
  file, it does not restore one. If either is the more recoverable it is this
  one, the ciphertext being intact. P1: the passphrase is one the user chose
  and uses at every open. The control is not minted: a recovery path for a
  lost passphrase — a
  second wrapping of the store key under a recovery secret the user records, or
  an exported key backup — together with a documented re-enrolment procedure
  for when no such path was set up (not-minted control 16). That is the same
  body of unwritten work as the atomic-write control the crash hazard names
  (not-minted control 10): both are the store surviving an event the user did
  not choose.
- **RC-b6mydy (invite cross-check).** A member who genuinely received an
  admission from an administrator other than the one whose invite they
  imported — a second administrator does not exist today, but the check is
  what would refuse one — is rejected. No hazard at present; noted so the
  check is reconsidered when there is more than one author.
  *(Amended 2026-10-05, owner ruling of that day, change
  `worktree-org-node-chain-authority`; written by change
  `worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md: the
  invite cross-check is gone, RC-b6mydy being amended in place to check
  nothing about the sender, so no such admission is rejected and this
  hazard no longer arises.)*
- **RC-wqgm2p (delete the record on one's own removal).** The presence test
  that decides whether the node is still a member filters the store's personas
  by their recorded Organisation (the `my_still_present` test in
  `receive_and_self_delete_if_revoked`, `org-node/src/service.rs`), so a persona
  that is not linked to the Organisation yields no candidate, the test
  reads as absent, and the branch deletes the record — a spurious self-delete
  on an inconsistent store. What the branch then does is the whole of the harm:
  it removes this node's record of the Organisation from the store and marks
  every persona of that Organisation revoked (the self-delete branch of the
  same function), so the member's
  own device no longer holds the record it needs and cannot reach the material
  the Organisation governs. That is unavailability, which the Method section
  above and the class C ADR (`docs/adr/2026-09-01-safety-class-c.md`) put at
  **S3** at its worst credible outcome. Assessed **S3 / P1**, **not
  acceptable**. An earlier draft put this at S2 / P1, arguing the severity down
  from recoverability by re-admission; review round 6 found that defective and
  it was re-assessed in the fix round that followed. Recoverability is not
  severity: severity is a property of the harm, and an organisational fallback
  does not make an injury less severe — the correction the class C ADR made to
  itself on 2026-09-02, and the same move this register refuses two entries
  above for the passphrase lock-out. The store state is where the probability
  belongs: the situation needs a record for the Organisation sitting in the
  store with no persona linked to it, and there are two routes to it.
  **The first** is a commit that writes the record and links no persona: this
  node's own admission does not leave that behind — `receive_and_verify`
  writes the record and sets the persona's `org_id` in one save
  (the commit at the end of `receive_and_verify`, `org-node/src/service.rs`) —
  and the one branch that writes a record while linking nothing is the branch
  where no persona's device key is in the committed trie at all (the
  `my_persona_id` lookup finds none), which is not this node's own
  admission. **The second**, which review round 7 found and this entry missed
  until then, is a link that is taken away again. A persona records a single
  Organisation — `PersonaRecord.org_id` is one `Option<OrgId>`
  (`org-node/src/store.rs:59`) — and the same save overwrites it
  unconditionally (where `receive_and_verify` marks the persona Active), so
  admitting a persona already active in one
  Organisation to a second one repoints it at the second and leaves the first
  Organisation's record in the store with nothing linked to it. Nothing in the
  code prevents that admission: "one per org" is a comment on the type
  (`org-node/src/store.rs:53`), not a check. A later Change set for the first
  Organisation then reaches `receive_and_self_delete_if_revoked`, finds no
  candidate at the presence filter (`my_still_present`), and deletes the
  record.
  This second route is the stronger instance of the harm, and the reason the
  entry is worse than it read before: on the first route the node loses a
  record of an Organisation it was never a member of, while on the second it
  deletes the record of an Organisation it genuinely belongs to and revokes
  every persona it has for it (the self-delete branch) — the member is put out of an
  Organisation nobody removed them from, which is exactly the harm this hazard
  names.
  With both routes on the table the probability is still **P1**, but on
  narrower ground, and the ground has changed. It is no longer "no path leaves
  that store state behind": a path does, and the code does not forbid it. It
  is that the state is reachable only through a use the design does not intend
  — one persona per Organisation is what the type comment, and the flow that
  creates a persona per join, both assume — so a persona in two Organisations
  is a misconfiguration rather than an ordinary event. What is now ordinary is
  everything after it: once such a persona exists, the very next update or
  revocation the node receives on the first Organisation triggers the delete,
  with no further coincidence required. So the margin is thin, and it rests on
  a convention nothing enforces; the day a persona in more than one
  Organisation becomes a supported flow, this is **P2** and must be
  re-assessed. The verdict above does not turn on the margin: S3 is
  unacceptable at every probability the matrix admits.
  By the counting test above this is a hazard the register does not otherwise
  contain rather than a further trigger: no counted hazard covers a member who
  was never removed losing their record. The
  control is not minted (not-minted control 17): make the presence test
  independent of a Persona's recorded Organisation, so that a Change set which
  does not remove this node's Device key can never delete its record, with a
  test. A test is not itself a risk control, which is what an earlier draft of
  this entry offered in place of one.

## Hazards recorded in prose

These carry severity, probability and their controls, and no identifier, for
the reason org-members' register states at length: the gate refuses a hazard
whose control is not implemented and tested, and an identifier would assert a
traced control where there is none. Nothing is lost to a reader; what is lost
is the mechanical trace, and that is reported upstream, not worked around.

**Publish before persist.** `admit_member` and `revoke_member` write the new
root to the chain first and update the local record only after the envelope
has been sent to the peer (`org-node/src/service.rs`: in each function the
`submit_update` call, then the transport send, then the record update). A
failed send — the peer is offline, the address
is stale, the endpoint fails to bind — or a crash between the two returns an
error with the chain one epoch ahead of the administrator's record. Every
later publish then carries the stale epoch and is refused by the contract's
epoch check, so the administrator can neither admit nor revoke; and for a
revocation, the chain records a removal that no device is ever told about,
while every member's record still lists the removed device. Hazardous
situation: a revocation of a stolen device is attempted while that device is
offline. Assessed **S3 / P2** — a send failure is an ordinary event. Filed as
**PR-vt244s**. The control — persist the local record as soon as the chain
confirms, and make delivery a separate, retryable step — is not minted.

**A stale view of the membership.** A member's record is updated only when a
Change set is pushed to it, and the node pushes only to the device an action
affects. Every other device stays at its last epoch: after a revocation, the
remaining members' records still list the removed device until each is
individually the subject of some later action, and nothing in the node reads
the chain on a schedule or compares the local epoch with the chain's except at
the moment of receiving an envelope.

A legitimate envelope that arrives out of order — two administrator actions in
flight, the second delivered first — leaves a device in the same position, and
the check that refuses it is worth naming precisely, because an earlier draft
of this file put it under the ordering controls where it does not belong. Such
an envelope passes RC-m4r75s's sequence check (its Sequence number is higher,
not lower, than the mark). It then fails at step 5 of `verify.rs`, the
requirement that the Change set's declared base be the local root
(`org-node/src/verify.rs:61-63`, *re-resolved 2026-10-05 after the trim, review round 3 finding-16; it read `:73-75` after the merge of master `1feb608`, `:72-74` on this branch before it and `:67-69` on master*) — org-members' rule, exported as REQ-4umsuz,
and the correct answer for a Change set that cannot be applied to the record
the node holds. RC-e5atck's epoch check is never reached: the chain is not read
until step 7 (`:79-82`, the epoch comparison it feeds at `:83-85`; *re-resolved 2026-10-05 at the merge of master `1feb608`; they read `:78-81` and `:82-84` on this branch, `:73-76` and `:77-79` on master*), which is
after the base-root comparison has already returned `DeltaBaseMismatch`.
*(Re-resolved 2026-10-05 after the trim, review round 3 finding-16: with the
signature step gone the code numbers the base-root check step 4 and the chain
read step 6; they are at `verify.rs:61-63`, `:67-70` and `:71-73`. The order
is unchanged.)*
Neither ordering control rejects an out-of-order
envelope — one passes it, the other is never consulted — and neither
introduces this hazard. What org-node contributes is the handling of the
rejection: the connection that carried the envelope is consumed with no queue
and no request to resend, `receive_and_verify` taking one connection per call
and returning the error, so the device is behind until the administrator acts
on it again.
Assessed **S3 / P2**, the same as the rest of this hazard; the control is a
resync or requeue, not minted.

The `protocol.qnt` staleness window that org-members' register discusses has
no counterpart in code: no maximum age is stated and none is enforced.
Assessed **S3 / P2**. Controls, none minted: fan-out of every Change set to
every current device; a periodic chain read that flags a record older than the
chain's epoch; a stated maximum age beyond which the node refuses to act on
its record. The consumer-side
half of one of these already exists as the premise of the finalised-block
expectation now held by org-io (`org-io/docs/requirements/`) — that the state
read is the Finalised one — but nothing reads it on a schedule.

**A revoked device keeps the organisation secret.** Stated under HAZ-vxabf9
and repeated here because it has no control in this unit at all: the secret
handed over at admission (the `WireMessage` `admit_member` sends,
`org-node/src/service.rs`) is never rotated and
never revoked. Assessed **S3 / P2**. The control lies in the CGKA layer, Phase
3, and until it exists a revocation removes a device from the record without
removing anything the device can use.

**Compromise of the administrator's authority.** Authorship of membership is
one ed25519 key (the published `org_pub_key`, which is the administrator's
Member-as-a-group key) plus one chain account, and the chain account acts
through a threshold-1 multisig or directly: `build_dispatch_tx` submits the
administrator's call alone when there are no co-signatories and as
`as_multi_threshold_1` when there are, and the threshold-2-or-more outcome is
dead code that errors if ever reached (`org-node/src/chain_write/multisig.rs:125-143`,
`org-node/src/chain_write/mod.rs:43-69`). Whoever holds the administrator's
store passphrase and the `ODS_ADMIN_SEED` the app supplies
(`app/src-tauri/src/state.rs`) publishes any root and authors any Change set,
and every node's verification then accepts a matching envelope as authentic,
because the chain is the authority it consults. Assessed **S3 / P1**: total
disclosure, needing a compromise of the Organisation's own governance. The
control is key custody and a threshold of two or more, which is a deployment
obligation with no software behind it and no document telling an operator
so — org-members' register lists that document as its ninth not-minted
control, and the write path that would honour a higher threshold is this
unit's to build. Not minted.
*Amended 2026-10-05 (owner ruling): the first sentence no longer
holds. No key authors membership. The published `org_pub_key` is the
Organisation public key, an X25519 key distinct from every member key
(REQ-ech45n), and no Envelope is signed. Authority over membership is the
chain account alone, through the Organisation's multisig proxy, and that is
decided on-chain, not by org-node. A receiver accepts an Envelope from any
peer, and commits it only if it reaches the published root. Whoever controls
the chain account therefore still publishes any root, and any peer can deliver
the matching Change set. The S3 / P1
assessment and the remedy (key custody and a threshold of two or more) are
unchanged.*

**Admission delivered to an address the joiner did not prove.** Stated under
HAZ-ep6uzs; filed as **PR-2dmjzj**. Assessed **S3 / P1** — it needs the
join-request blob altered between the joiner and the administrator, and the
blob travels out of band by copy and paste. The control is to dial the peer by
the Device key the join request carries in loopback mode as the networked
branch already does, or to refuse an address whose identity differs from that
key.

**A root the chain later discards.** The administrator's publish waits for
the update extrinsic to be finalised before the local record moves
(`dispatch_org_call` → `wait_for_finalized_success`,
`org-node/src/chain_write/multisig.rs:90-119`), and the receiving node reads
the Organisation state with no block named, which on-chain-client performs as
a Finalised read. So a reorganisation that discards a published root should
find no node that committed against it. Two qualifications. The receiver's
half is the finalised-block expectation now held by org-io (`org-io/docs/requirements/`), open
until 2026-12-05 with a doc-comment
behind it. The submitter's half is tested only in the chopsticks lane
(`org-node/tests/finality_polling.rs`, `chain_genesis_e2e.rs`), which the
merge gate does not run; and there is no reorganisation test anywhere in the
crate — the helper named `chopsticks_reorg.rs` contains a block-mining
function and nothing that reorganises — and no code path on the submitter
that would notice a discarded root and repair its record. Assessed
**S3 / P1**. Not minted: the control exists, its evidence is not run by any
gate, and minting it would repeat the position org-members' register called
"evidence that exists and counts for nothing". Minted when the chopsticks lane
runs in CI (setup checklist).

**The only copy of the keys lost on a crash.** `PersonaStore::save` writes the
encrypted file in place with `std::fs::write` — no temporary file, no rename,
no fsync (`org-node/src/store.rs:412`). A crash or power loss during the write
leaves a truncated file that fails authentication on the next open, and with
it every persona's keys and every Organisation's record on that device.
Hazardous situation: the device loses power while committing a received
Change set. Harm: unavailability until re-admission. Assessed **S3 / P1**. The
control — write to a temporary file, fsync, rename — is not minted.

## The problem report that moved

The chain-reader finality report — the node's `OnChainReader` documents a
best-block read while the client it calls performs a Finalised one — was
filed in org-members' problems ledger on 2026-09-02, when org-members' was the
only ledger. It is an org-node defect, and under D14 a provider's defect is
filed in the provider's ledger. It now lives in
`org-node/docs/problems/2026-09-09-org-node-problems.md`
with its opened date and, but for two mechanical changes, its original text:
the report ID its last paragraph named is replaced by the file that holds that
report, and the path to org-members' register was updated when the ledgers
moved into their units. No statement of the defect was rewritten. Its
`affects:` line is retargeted from org-members' control to RC-6a2dke, and it
remains open: correcting the doc-comment is a `resolve-problem` change of its
own. Where org-members' register named this report by ID
(`2026-09-02-membership-hazards.md`) it now names the file instead, because a
provider may not cite a consumer's items; that is the one word this change
replaces there, and its other edits are the dated corrections described under
"Scope". No item in it is altered.

## Residual risk, and the conclusion this analysis reaches

Per hazard, after controls:

| Hazard | S/P | Residual | Why |
|---|---|---|---|
| HAZ-tawvm2 | S3/P2 | not acceptable | decisive read rests on a provider's doc-comment until the finalised-block expectation now held by org-io (`org-io/docs/requirements/`) is met; one author key |
| HAZ-p4gfv9 | S3/P1 | not acceptable | per-node mark; the Organisation-level bar is the contract's |
| HAZ-5f9jcm | S3/P2 | not acceptable | shallow fuzzing at the gate; blobs unbounded; no accept-time limits |
| HAZ-ep6uzs | S3/P2 | not acceptable | check skipped without an invite; revocation path unchecked; PR-2dmjzj |
| HAZ-vxabf9 | S3/P2 | not acceptable | cooperative control; no fan-out; secret never rotated; REQ-q92yac open |
| HAZ-45ucqx | S3/P2 | not acceptable | fixed salt, default file mode, no zeroise |
| HAZ-uy8sxm | S3/P1 | not acceptable | formatting route closed (RC-8a4xjb); named accessor and no zeroise left to future work |
| HAZ-vfjy32 | S3/P1 | not acceptable | P1 because no Persona store is deployed; no migration by owner ruling |

(Amended 2026-10-05 after independent review round 6: the last two rows were
added so that this table stays the unit's roll-call. Both hazards were
identified on 2026-10-04 by the type-safety change and are defined, with their
controls and residual reasoning, in `2026-10-04-type-safety.md`. The count and
the overall conclusion below are this analysis's as of 2026-09-09; with these
two the unit counts fourteen distinct hazards, neither of the two is
acceptable, and the overall verdict, UNACCEPTABLE, is unchanged.)

*Amended 2026-10-05 (review round 2, finding-12 and finding-22).* HAZ-vxabf9
is also mitigated by RC-95dgg8, which closes the Sequence-number jam the
unsigned Envelope opened; its three counts stand. HAZ-45ucqx's "no zeroise"
now excepts `X25519Keypair` in memory (LLR-98ufry); the persisted seeds and
the Organisation private key in `OrgRecord` are still not zeroised. Neither
verdict changes. *(Amended at the merge of master `1feb608`: this ended
"(PR-hqwpg9)". Master resolved PR-hqwpg9, which is about `Debug` output, not
zeroising; the persisted seeds and the Organisation private key are now held in
redacted secret types (`MemberSeed`, `OrgPrivateKey`), which are not wiped on
drop either — RC-jjsz97's residual.)*

*Amended 2026-10-05.* Two rows above no longer match the code. For
HAZ-tawvm2, "one author key" is gone: there is no author key, and nothing
about the sender is checked; the decisive read is unchanged. For HAZ-ep6uzs,
the sender checks are gone (RC-b6mydy amended in place); its residual is
recorded there. PR-u4c2vp is resolved by the owner's ruling. PR-2dmjzj is
still open. A re-scoring is the next risk analysis's job.

And the seven prose hazards: publish before persist **not acceptable**
(S3/P2, PR-vt244s); stale view **not acceptable** (S3/P2); secret retained
after revocation **not acceptable** (S3/P2, Phase 3); administrator authority
**not acceptable** (S3/P1, control outside software); admission address
**not acceptable** (S3/P1, PR-2dmjzj); discarded root **not acceptable**
(S3/P1, evidence ungated); keys lost on crash **not acceptable** (S3/P1). The
controls introduced three entries of their own and **none of the three is
acceptable**. The frame bound is **not acceptable** (S3/P2), because its
refusal lands inside the publish-before-persist window on the admission and
the revocation path alike; the forgotten passphrase is **not acceptable**
(S3/P1), the same loss of the same file as the keys-lost-on-crash hazard; and
the spurious self-delete on an inconsistent store is **not acceptable**
(S3/P1), because deleting this node's record of the Organisation is the
unavailability harm at its worst credible outcome whatever the fallback. The
out-of-order rejection is not one of them — the check
that refuses such an envelope is org-members' base-root rule, and the
consequence is assessed under stale view.

The count is six identified hazards, five further distinct hazards in prose,
and one further from the controls themselves — **twelve**. The test is the one
stated under "Hazards introduced by these controls": a distinct hazard is a
hazardous situation and harm no counted hazard covers, and anything else is a
further trigger for one already counted. It excludes two of the seven prose
entries — the retained organisation secret and the admission address restate
HAZ-vxabf9 and HAZ-ep6uzs, and are repeated there because neither has a
control in this unit — and it excludes two of the three introduced entries,
the frame bound being a further trigger for the publish-before-persist hazard
and the forgotten passphrase for the keys lost on a crash. The twelfth is the
spurious self-delete, which no other entry covers. **Of all twelve, and of the
two further triggers with them, not one carries a residual risk this project's
matrix calls acceptable**, and no probability estimate could have changed that
at S3. After this round there is no assessed risk in this register that is
acceptable, and the Method section's "every hazard below is S3" now holds
without qualification. The only S2s left in the file are the withdrawn earlier
drafts of the passphrase entry and the self-delete entry, each recorded as
withdrawn where it stands.

### Overall residual risk

**Against the intended use that makes this software Class C, the overall
residual risk of the org-node unit is UNACCEPTABLE.** That restates, for this
unit, the conclusion org-members' register reached for the membership
capability on 2026-09-02 and does not change it: three defects, on the publish
path, the loopback admission path and the revocation receive path; one control
whose decisive half is an open expectation on a provider (RC-6a2dke, whose
chain read is the subject of the finalised-block expectation now held by
org-io (`org-io/docs/requirements/`)) and, separately, one uncontrolled residual
of HAZ-vxabf9 that a second open expectation would close (REQ-q92yac); one
control that depends on the excluded device's cooperation; a store whose
protection is a passphrase over a fixed salt, with no way back for the member
who forgets it; and all three of the entries the controls themselves
introduced unacceptable in their own right.

What follows is what followed there. The software is a proof of concept and
has not reached its intended use; the restriction on use still has to be
written where a user meets it (setup checklist, owner's item); the class stays
C because it is set by the planned use, and lifting the restriction reopens
this evaluation, not the class. And, as there, the verdict is a reasoned
conclusion rather than the output of a stated criterion, because the project
has no risk management plan defining overall acceptability — the same open
item on the checklist.

## Controls identified but not minted

Each would reduce a residual risk above. None has a requirement, and none is
given an RC identifier until it does.

1. **Persist before push** (publish-before-persist hazard, PR-vt244s). Update
   the local record when the chain confirms the update and make delivery to
   the peer a separate, retryable step, so a failed send leaves the
   administrator able to act and the revocation deliverable. The reproducing
   test is a send to an unreachable address after a successful chain write,
   asserting the record's epoch equals the chain's.
2. **Bind the admission delivery to the joiner's Device key in loopback mode**
   (HAZ-ep6uzs, PR-2dmjzj). Refuse a join-request address whose identity is
   not the Device key the request carries, or dial by the key as the
   networked branch does.
3. **Make the invite mandatory on first admission** (HAZ-ep6uzs). Turn the
   fall-through in `receive_and_verify`'s invite cross-check into a
   rejection.
4. **Cross-check the sender on the revocation receive path** (HAZ-ep6uzs,
   PR-u4c2vp). The `UpdatedNotRevoked` branch commits a record from a sender
   it did not check.

   *(Amended 2026-10-05, owner ruling of that day, change
   `worktree-org-node-chain-authority`; written by change
   `worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md:
   controls 3 and 4 are withdrawn by that ruling. No Invite is required for a
   first admission, and nothing about the sender is checked on either Receive
   operation; PR-u4c2vp is resolved as not a defect.)*
5. **Fan out every Change set to every current device, or read the chain on a
   schedule** (stale-view hazard, HAZ-vxabf9). Either closes the window in
   which the remaining members still list a removed device; a stated maximum
   age a node may act on turns the modelled `TAU` policy into a control.
6. **Requeue or request resend on an out-of-order envelope** (stale view).
7. **Rotate the organisation secret on revocation** (HAZ-vxabf9). Phase 3;
   recorded so the boundary is visible from this side.
8. **Bound and authenticate the blobs, and limit the accept boundary**
   (HAZ-5f9jcm). A size limit on the base64 invite and join request; an
   allowlist or rate limit before a connection costs a chain read.
9. **Per-installation salt, owner-only file mode, zeroisation** (HAZ-45ucqx).
   Replace the fixed salt with one stored beside the ciphertext; create the
   file with mode 0600; zeroise the plaintext and the derived key.
10. **Atomic store write** (keys-lost hazard). Temporary file, fsync, rename.
11. **Threshold of two or more for the administrator account, and the
    operator document requiring it** (administrator-authority hazard). The
    write path's higher-threshold outcome is dead code today.
12. **Run the chopsticks lane in CI, and add a reorganisation test**
    (discarded-root hazard). Then the finalised-publish control can be minted
    with evidence a gate runs.
13. **Deep fuzz with a seed corpus** (HAZ-5f9jcm). The two targets exist; a
    libFuzzer lane and a corpus are what would make RC-e2uvje's claim carry
    weight beyond one second of generation.
14. **Run clippy on org-node and on the app** (HAZ-5f9jcm, RC-e2uvje). Add
    `cargo clippy -p org-node --features app,test-support -- -D warnings` and
    the same under `--features app` alone to this unit's `verify_commands`,
    and org-node to the CI `clippy` job
    (`.github/workflows/rust.yml:80-87`), so the panic-freedom denial this
    register cites is checked at every merge rather than asserted. Two
    feature sets because `app` is what the crate ships as and `test-support`
    is what adds a further module to the library. The app unit needs the same:
    `app/src-tauri/Cargo.toml` has no `[lints]` section, so it does not
    inherit the workspace denial, and its `verify_commands` run no clippy
    either.
15. **Bound or omit the membership snapshot on the send path** (the frame
    bound's introduced trigger for the publish-before-persist hazard, and
    PR-vt244s with it). Both Wire messages carry
    the whole pre-change record (`admit_member` and `revoke_member`, where
    each builds the record snapshot, `org-node/src/service.rs`), which makes
    the 1 MiB frame a bound on the Organisation, and the bound bites after the
    chain write. On the revocation path the snapshot is dead weight —
    `receive_and_self_delete_if_revoked` rebuilds its trie from its own store
    — so omitting it there costs nothing. On
    the admission path the receiver does need a genesis record, so the control
    is to bound it, to page it, or to have the joiner obtain the record by some
    route other than the admission frame.
16. **A recovery path for a lost store passphrase, and a documented
    re-enrolment procedure** (the passphrase's introduced trigger for the
    keys-lost hazard). Wrap the
    store key a second time under a recovery secret the user records, or
    support an exported key backup, so that a forgotten passphrase is not the
    loss of every key on the device; and write down what a member and an
    administrator do when neither exists, which is re-admission with new keys.
    Related to control 10 (atomic write), and worth doing with it: both are the
    store surviving an event the user did not choose.
17. **Make the presence test independent of a Persona's recorded Organisation**
    (the spurious self-delete RC-wqgm2p introduces). The test that decides
    whether this node is still a member filters the store's personas by their
    recorded Organisation first (the `my_still_present` test in
    `receive_and_self_delete_if_revoked`, `org-node/src/service.rs`), so a
    persona
    that is not linked to the Organisation reads as absent and the record is
    deleted — whether it never was linked, or was linked and then repointed by
    a later admission to a second Organisation, `PersonaRecord.org_id` holding
    one Organisation and being overwritten unconditionally
    (`org-node/src/store.rs:59`; `receive_and_verify` in
    `org-node/src/service.rs`, where it marks the persona Active). Decide
    presence from the Device keys the store holds against the Device keys the
    verified trie holds, without consulting a persona's `org_id`, so that a
    Change set which does not remove one of this node's Device keys can never
    delete its record of the Organisation. With a test — two cases, one per
    route: a committed Change set that leaves this node's Device key in place
    must leave the record present both against a store whose persona was never
    linked to the Organisation and against a store whose persona has since
    been admitted to a second Organisation. Minted as an RC
    when that behaviour is implemented and the test runs at the gate; recorded
    here rather than minted because an identifier would assert a traced control
    where there is none, and because a test on its own is evidence, not a
    control.

## Derived requirements assessment

assesses: REQ-wp2nyc, REQ-bvh8v6, REQ-gju89b, REQ-ag6kqm, REQ-8gz8bu, REQ-nhe2zu, REQ-6yu72z, REQ-mr5abb, REQ-9g6as6, REQ-bcxz96, REQ-eg5j8u, REQ-xa6smf, REQ-ztdza4, REQ-uxv2x2, REQ-hzm4kt

Every requirement minted by this change is `satisfies: derived`: each exists
because of how the node was built — a signed envelope, a sequence number, a
passphrase-protected store — and not because a system-needs document asked
for it. Finding the hazard a rule addresses gives it a justification, not a
parent. Each is assessed here, and each has a hazard to point at.

- **REQ-wp2nyc** (reject a Change set whose recomputed root differs from the
  chain's): realises RC-6a2dke against HAZ-tawvm2. Introduces no hazard: a
  rejection leaves the record unchanged.
- **REQ-bvh8v6** (reject a Change set for an Organisation the chain does not
  know): realises RC-6a2dke against HAZ-tawvm2. Introduces one situation worth
  naming: a node whose chain read fails or whose reader has not been refreshed
  sees "absent" and refuses — fail-closed, which is the intended direction,
  and an unavailability that resolves on the next read.
- **REQ-gju89b** (reject an envelope for another Organisation before decoding
  its Change set): realises RC-pm9kmx against HAZ-tawvm2. No hazard impact.
- **REQ-ag6kqm** (reject an envelope whose signature does not verify under
  the published signing key, before decoding): realises RC-pm9kmx against
  HAZ-tawvm2. Its dependence on a single published key is the
  administrator-authority hazard, recorded in prose.
- **REQ-8gz8bu** (reject a commit against an Organisation state whose epoch
  is not newer than the last committed): realises RC-e5atck against
  HAZ-tawvm2. Refuses a genuinely current envelope when the node's own epoch
  is somehow ahead of the chain's — which cannot arise without the chain
  moving backwards, and if it did, refusing is right.
- **REQ-nhe2zu** (commit the applied Change set with the chain's epoch and
  the envelope's sequence number when every check passes): realises
  RC-6a2dke, RC-e5atck and RC-m4r75s together; it is the normal case of
  HAZ-tawvm2's and HAZ-p4gfv9's controls. Its hazard impact is the stale-view
  hazard: a commit updates one device's record and no other's.
- **REQ-6yu72z** (reject an envelope whose sequence number is not greater
  than the highest committed, before decoding): realises RC-m4r75s against
  HAZ-p4gfv9. No hazard impact: it does not refuse an out-of-order envelope,
  whose sequence number is higher than the mark and which fails instead at the
  base-root rule; that case is assessed under the stale-view hazard.
- **REQ-mr5abb** (advance the high-water mark only on commit and never
  backwards): realises RC-m4r75s against HAZ-p4gfv9. No hazard impact; it is
  what keeps REQ-6yu72z from being a replay bypass.
- **REQ-9g6as6** (never panic decoding arbitrary bytes as an envelope and a
  Change set): realises RC-e2uvje against HAZ-5f9jcm. No hazard impact.
- **REQ-bcxz96** (never panic verifying arbitrary envelope bytes): realises
  RC-e2uvje against HAZ-5f9jcm. It does not carry RC-6a2dke: its fuzz target
  cannot reach the accept branch in practice, so it is evidence of
  panic-freedom over the verification path and not of the root match, which
  RC-6a2dke gets from REQ-wp2nyc and REQ-bvh8v6. No hazard impact.
- **REQ-eg5j8u** (reject a frame over 1 MiB before decoding it): realises
  RC-gfn6kr against HAZ-5f9jcm. Introduces the size bound assessed above
  (S3/P2, not acceptable: the refusal lands after the chain write, inside the
  publish-before-persist window).
- **REQ-xa6smf** (on first admission with an imported invite, reject a Wire
  message from a device other than the invite's administrator): realises
  RC-b6mydy against HAZ-ep6uzs. Its scoping to "with an imported invite" is
  the fall-through recorded under that hazard.
- **REQ-ztdza4** (after admission, reject a Wire message from a device not in
  the record it was verified into): realises RC-b6mydy against HAZ-ep6uzs and
  HAZ-vxabf9. No hazard impact.
- **REQ-uxv2x2** (delete the record and mark the persona revoked on a Change
  set that removes the node's own Device key): realises RC-wqgm2p against
  HAZ-vxabf9. Introduces the spurious self-delete on an inconsistent store
  assessed under "Hazards introduced by these controls" (S3/P1, not
  acceptable: deleting this node's record of the Organisation is the
  unavailability harm at its worst credible outcome, and the control that
  would close it is not-minted control 17). Its own abnormal-input case — a
  revocation whose envelope fails verification leaves the record in place — is
  in `org-node/tests/service_stories.rs`, and is the reason a rejected message
  cannot reach the delete branch at all.
- **REQ-hzm4kt** (the store opened with the wrong passphrase yields an error,
  and no seed appears in the file in clear): realises RC-jjsz97 against
  HAZ-45ucqx. Introduces the forgotten-passphrase lock-out assessed above
  (S3/P1, not acceptable: the same loss of the same file as the
  keys-lost-on-crash hazard).

The two expectations, the finalised-block expectation now held by org-io
(`org-io/docs/requirements/`) and REQ-q92yac, were assessed in
`2026-09-06-dependency-expectations.md`, which remains the assessment of
record; this file adds the hazards each was written toward — HAZ-tawvm2 and
HAZ-vxabf9 respectively — and the dates they fall due.

## Added 2026-10-03 — the binding scope of the transport, assessed

assesses: REQ-db6s7q

**REQ-db6s7q** (in Loopback mode, bind only loopback addresses and offer a
peer only loopback addresses) was minted while resolving PR-d4nye8, and is
assessed here because it is `satisfies: derived` and realises no control in
this register. It is recorded as an assessment rather than as a new hazard,
and the reasoning is given in full so that disagreeing with it is cheap.

**What the defect changed about this register's picture.** For as long as
`bind_with_mode` existed, a node in Loopback mode bound the wildcard — `0.0.0.0`
and `[::]` — and offered a dialling peer whichever non-loopback address iroh
discovered, on this machine the LAN one. The mode's own documentation said it
bound `127.0.0.1`.

The wildcard is **every interface the host has**, and naming only the LAN
understates it: a VPN or tunnel interface, a container or VM bridge, and a
public address if the host carries one, are all bound by `0.0.0.0` exactly as
the LAN interface is. So the population that could open an authenticated
connection to a node in Loopback mode was not "processes on this machine" but
"whatever can route to any address this host answers on" — which on a laptop
behind NAT is usually the LAN, and on a host with a public address or a
corporate tunnel is not. No document in this unit said so.

**It mints no new hazard, because the harms it widens access to are already
registered.** The reachable surface is what changed, so the hazards it bears on
are the ones whose probability rationale is about who can reach the node, and
there are **three**, not one. An earlier draft of this section surveyed only
HAZ-ep6uzs and was wrong to; the other two are quoted here because their
rationales are more plainly reachability-scoped than the one it did survey.

- **HAZ-ep6uzs** — a connection "whose remote end is authenticated as a Device
  key and nothing more", the harm being a device the record does not list
  pushing an update the node acts on.
- **HAZ-tawvm2** — rationale, verbatim: "P2: **any device that can reach the
  node** can deliver an envelope, the transport authenticates a device key and
  nothing more, and the envelope is hostile input by construction."
- **HAZ-5f9jcm** — rationale, verbatim: "P2: the input is attacker-chosen and
  the surface is a decoder **reachable by anything that can open a QUIC
  connection to the node**."

Each is an existing hazard whose population of reachers the wildcard bind
widened and this fix narrows. None of them is a *new* harm, which is why
nothing is minted — but all three had their P terms set while the register
believed Loopback meant same-machine, and the probability note below applies to
all three rather than to HAZ-ep6uzs alone.

**The control does not weaken with distance — but it does not run everywhere.**
RC-b6mydy rejects a Wire message from any Device key the record does not list,
after verification and before the store is written, and that is as true of a
LAN peer as of a same-machine one. An earlier draft stopped there, which
overstated it. This register records seven hundred lines above that RC-b6mydy's
residual risk is **not acceptable** and that the control is "weaker than its
wording in three places": the first-admission check runs only when an invite
was imported, falling through to signature and chain proof alone otherwise
(the invite cross-check in `receive_and_verify`, `org-node/src/service.rs`),
and the revocation receive path performs neither clause, its
`UpdatedNotRevoked` branch committing a record from an unchecked sender
(`receive_and_self_delete_if_revoked`, filed as PR-u4c2vp). **On exactly those paths,
distance was the only thing standing between the node and an arbitrary peer**,
and the wildcard bind removed it. That does not change what is minted, and it
does sharpen what the fix is worth.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).*
RC-b6mydy no longer rejects a Wire message from a Device key the record does
not list: it is amended in place to check nothing about the sender, on every
path. Distance now stands between the node and an arbitrary peer on every
receive path, and what an arbitrary peer can deliver is what the chain
already published, apart from the Organisation secret (PR-ve9zw8).

**Probability: three P2 figures rest on a premise this defect falsified, and
all three are owed a re-derivation.** An earlier draft of this paragraph
asserted that HAZ-ep6uzs's P2 was "a judgement made about the Networked path"
and therefore unaffected. The register does not support that. The rationale as
written is, in full: "P2: relaying is what a network does, and the receiving
device on first admission has no record yet to check the sender against." It
carries no mode scoping, and its second clause — first admission with no record
to check against — happens in both modes. The register's own Loopback-mode
instance of this hazard, PR-2dmjzj, is assessed S3/**P1**.

The same applies, and more directly, to HAZ-tawvm2 and HAZ-5f9jcm, whose
rationales are quoted above and are *expressly* about who can reach the node.
If any figure in this register was derived from a picture of the reachable
population, it is those two.

The honest reading is that all three P terms were assessed at a time when every
document in this unit said Loopback meant same-machine, and that premise was
false. **That is a reason to re-derive them, not a reason to assert they are
unchanged**, and retrofitting a Networked-only scope onto a sentence that never
had one was the weakest step in this assessment. The figures are left at P2
here because moving a rated number is not a side effect a defect fix may have;
they are owed a pass of their own, booked as an owner item in
`docs/plans/2026-09-05-ratchet-setup.md`.

**The organisation secret: inbound unchanged, outbound materially narrowed.**
An earlier draft said flatly that the secret "is not newly exposed", reasoning
that the admission Wire message travels inside QUIC under TLS authenticated to
the peer's Device key, so a device on the path that is not the intended peer
holds ciphertext. That remains true, and it disposes of the **inbound**
direction: the wildcard bind widened who could attempt a connection, not who
could read one.

It is wrong about the **outbound** direction, and PR-2dmjzj in this same
register is why. In Loopback mode `admit_member` dials the full `EndpointAddr`
carried in the join-request blob (the `TransportMode::Loopback` arm of the
send in `admit_member`, `org-node/src/service.rs`), which is
unsigned and unauthenticated by design, and does **not** bind that address to
the joiner's Device key — the Networked arm does, deriving the peer from
`join_request.device_key` (the `TransportMode::Networked` arm). An altered blob therefore redirects the
administrator's dial to an attacker-chosen EndpointId, and the TLS this
assessment relies on then authenticates to *that* key. The ciphertext argument
protects the secret from an observer; it does nothing against a recipient the
sender was tricked into choosing.

Under the wildcard bind that dial could reach any destination the host could
route to. With loopback-only sockets it cannot leave the machine **by an IP
path**: every bound socket is loopback, so there is no socket from which an
off-machine destination is reachable.

That argument has a second leg, which an earlier draft left unstated.
`bound_sockets()` reports IP transports only; a relay is a separate transport
and never appears there, so loopback-only sockets do not by themselves confine
an endpoint — traffic could still leave by a relay. The confinement therefore
rests equally on `RelayMode::Disabled` and on no address-lookup service being
configured, neither of which REQ-db6s7q states.

**Of those two, one is asserted by the tests and one is not**, and an earlier
draft of this paragraph claimed both were. Review round 3 measured the
difference. Address lookup is genuinely closed: replacing `presets::Minimal`
with `presets::N0` reddens both tests on `an address-lookup service is
configured`. The relay leg is **not** closed: replacing
`RelayMode::Disabled` with `RelayMode::Custom(default_relay_map())` — real n0
relay servers, every bound socket still loopback — leaves both tests green,
while the endpoint goes on to acquire a home relay
(`relay_urls = [RelayUrl("https://euc1-1.relay.n0.iroh-canary.iroh.link./")]`,
`online()` true in about three seconds) and traffic does leave the machine.

The `relay_urls().next().is_none()` assertion that was supposed to cover this
runs immediately after `bind()`, before any home relay can have been acquired,
so it is vacuous in exactly the way the `TransportAddr::Relay` match arm it
replaced was vacuous. **That is the third time in this change that an
assertion was credited with a property it cannot observe**, which is worth more
than the individual corrections: the shape is not a slip, it is what happens
whenever a property is asserted at a moment when it cannot yet be false.

Closing it properly needs an assertion after `online()` resolves, and that is
host-dependent in the direction this change refuses — on a machine with no
route to a relay the mutant would acquire none and the test would pass, so the
assertion would measure the network rather than the configuration. iroh exposes
no relay map on `Endpoint`, so there is no synchronous alternative. **The
relay leg is therefore stated and unverified**, recorded here and in this
change's verification record rather than asserted.

**With both legs — one verified, one argued — this fix narrows PR-2dmjzj's
reachable destination set from "anything this host can route to" to "this
machine",**
which is a real reduction in that defect's worst case and was not claimed when
the fix was made. It does not resolve PR-2dmjzj: a hostile process on the same
machine still receives the secret, and the control that would close it — dial
the peer by the Device key the join request carries, as the networked branch
does — is still not minted.

**No control is minted.** RC-b6mydy already covers the harm. A control reading
"the node binds only loopback in Loopback mode" would restate REQ-db6s7q in
the register's vocabulary without mitigating anything RC-b6mydy does not
already mitigate, and a control minted only so that a requirement has a parent
to cite is a traceability artifact rather than a risk control.

assesses: REQ-2wzfzv

**REQ-2wzfzv, assessed: a deliberate unavailability, in the direction this
register already prefers.** That requirement states which sockets must come up
— IPv4 loopback required, IPv6 loopback optional — and it is `satisfies:
derived` with no control, for the same reason as REQ-db6s7q.

Its first clause *chooses* an unavailability: on a host that cannot bind IPv4
loopback the node gets no endpoint at all and its membership function is down,
which is the harm pathway HAZ-5f9jcm's severity is drawn from ("a member cannot
receive a membership change or act on the record at the moment a decision needs
it", S3). That is accepted rather than overlooked. The alternative is an
endpoint that came up on something other than loopback, which is PR-d4nye8
again and silently, where this failure is loud and at startup. A class C node
that cannot bind the socket its mode requires should refuse to run.

Its second clause removes an unavailability that the first draft of the fix
would have introduced: `BindOpts::is_required` defaults to true, so a mandatory
`[::1]` bind would have taken the whole endpoint down on every IPv6-less host,
for no safety benefit — the IPv4 socket alone satisfies REQ-db6s7q. Caught by
review round 1.

Neither clause changes a hazard's S or P. The probability note above applies to
this requirement too: it was written while the register's picture of Loopback
reachability was wrong, and the figures it leaves untouched are the same ones
owed a re-derivation.

**What this assessment does not establish.** It is the judgement of the change
that fixed the defect, not the output of a full `analyze-risks` pass over the
unit, and it was made by the author rather than the owner.

Two places it could still be wrong, and the first is not the one an earlier
draft nominated. That draft invited disagreement on whether reachability is
itself a harm — and the claim that actually failed review was a different one,
the inbound-only reading of the secret's exposure, corrected above. A section
that names its own weakest step can name the wrong step, so:

1. **P2 is carried forward on a premise known to be false**, as the
   probability paragraph above now says. Nothing here re-derives it.
2. **Reachability may be a harm in its own right.** A reviewer who holds that a
   class C node listening on interfaces its own documentation disclaims is a
   hazard independent of what the receive path then does with the connection
   should mint it. This paragraph is where to say so.
