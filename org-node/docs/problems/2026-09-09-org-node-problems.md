# Problem reports — org-node's own ledger opens

Five reports. The first moved here from org-members' ledger, where it was
filed on 2026-09-02 because that was the only ledger; it is an org-node
defect, and D14 puts a provider's defect in the provider's ledger. Its
opened date is the original's, and so is its text but for the two mechanical
changes noted at the foot of the report; its `affects:` line, which named an
org-members control a consumer may not cite, now names org-node's own
control.
Three of the other four were found by the code survey behind org-node's hazard
analysis (`org-node/docs/risk/2026-09-09-org-node-hazards.md`), and the last
(PR-w88sr9, opened 2026-09-10) by review round 7 of the same change, which
found a doc-comment contradicted by an assessment that change makes. All are
recorded before anyone investigates a fix, as `resolve-problem`
requires. None is fixed in this change: it is a hazard analysis, and changing
what `service.rs` does, or what `chain_read.rs` and `transport/wire.rs` say
about themselves, under a register written the same day would mix the two.

## The chain reader documents the wrong finality (moved 2026-09-09)

**PR-hvg2dy**: `OnChainReader::get_org_state` in `org-node/src/chain_read.rs`
is documented as fetching "the latest (current best) state", while the read it
performs passes `at = None` to `OrgRegistryClient::get_org_state`, which the
client documents as reading at the latest **finalised** block. The two
statements cannot both be true, and the doc-comment is the one that is wrong.
affects: RC-6a2dke
opened: 2026-09-02
status: open

Found while writing org-members' hazard analysis
(`org-members/docs/risk/2026-09-02-membership-hazards.md`), not by a failing
test. The register had to state which block the decisive control reads, and
the sources disagree: `on-chain-client/src/client.rs` documents `at = None` as
the latest finalised block, `org-node/src/preflight.rs` treats it as
finalised, and the reader's own doc-comment calls it current best.

Why this matters more than a stale comment usually would. The read is step 7 of
`verify_envelope_against_chain` — the independent trusted root that the whole
anchor rests on, and the reason a membership change is committed at all. Best
and finalised differ exactly when
a reorg is in flight, which is the case the control exists for: a root read
from a best block can be reorged away, and a revocation committed against it
would be believed on evidence the chain later discards. A reader who trusts
this comment concludes the control is weaker than it is and may add a
redundant finality check; a reader who trusts it while *writing* a second
reader may implement the best-block read the comment describes. Both
directions are worse than the truth.

There is a second, smaller inaccuracy in the same place: the method returns a
cached snapshot, refreshed only when a caller awaits `refresh()`, so at verify
time no block is read at all. The freshness of the root therefore depends on
the caller's refresh discipline, which is not documented anywhere near the
control that depends on it.

Not a code defect as far as this analysis established — the finalised read is
the correct behaviour and appears to be what the code does. What is wrong is
the documentation of a security-relevant control, which under IEC 62304 Class
C is not a lesser category of defect. The fix is to correct the doc-comment to
state the finalised read and the caching behaviour, and to say which of the
three descriptions the code is actually contracted to.

Not fixed in this change: this is the risk-analysis change, and it is
documentation-only in the regulated ledgers. Touching crate source under it
would mix a hazard analysis with a code change, the same reason the
device-removal report was left open under the requirements tooth.

Notes added at the move, 2026-09-09. The body above is the 2026-09-02 text
with two mechanical changes. Its last paragraph named org-members' other open
report by ID, which a consumer's ledger may not do, so it names the file that
holds that report instead
(`org-members/docs/problems/2026-08-31-device-removal-key-check.md`); and the
path to org-members' hazard register was updated when the ledgers moved into
their units. No statement of the defect was rewritten. Two
things learned since: the production receive path does not use this reader —
`receive_and_verify` reads the chain itself in the same operation and hands
verification a one-shot adapter (`org-node/src/service.rs:922-926`,
`:1532-1540`); and on the provider's side the same fact is now REQ-ysyu9g, the
expectation this unit holds on on-chain-client.

Reach corrected, 2026-09-10. The note above said the cached reader "serves the
chopsticks tests and the preflight". Preflight does not use it: it calls
`OrgRegistryClient::get_org_state(admin, None)` itself and documents that read
correctly as the latest finalised block (`org-node/src/preflight.rs:107`,
`:114`). Outside `org-node/src` the reader is constructed in exactly one
place — `org-node/tests/chain_genesis_e2e.rs`, a chopsticks-dependent test the
merge gate does not run — and in no production path and no gated path. The
blast radius is therefore smaller than recorded, not larger: no shipped
behaviour depends on the wrong comment today. The report stays open on the
grounds already given, which this does not touch: the doc-comment of a
security-relevant control says "current best" where the read is finalised, and
a second reader written from that comment would implement the weaker read.

Correction, 2026-09-09: the report names the wrong method. In this tree
`OnChainReader::get_org_state` (`org-node/src/chain_read.rs:50-59`) documents
the cached snapshot and its fail-closed behaviour and performs no read at all;
the "Fetch the latest (current best) state" doc-comment is on `refresh()`
(`:34-35`), and the `get_org_state(admin, None)` call that reads at the
latest finalised block is at `:40`. The defect is unchanged in substance —
a doc-comment on a security-relevant control says "current best" where the
read is finalised — and the fix is the same correction, made on `refresh()`.
The 2026-09-02 body above is left as it stands.

## The publish path writes the chain before the record

**PR-vt244s**: `admit_member` and `revoke_member` submit the new Membership
root to the chain and then update the local `OrgRecord` only after the
Envelope has been sent to the peer, so a failed send or a crash between the
two leaves the chain one epoch ahead of the administrator's record, every
later publish refused by the contract's epoch check, and — for a revocation —
a removal on the chain that no device is ever told about.
affects: RC-wqgm2p
opened: 2026-09-09
status: open

Where: `org-node/src/service.rs` — the chain write at `:814` and the record
update at `:864-871` in `admit_member`, with the send at `:836-853` between
them returning early on error; the same shape in `revoke_member` at `:1172`,
`:1218-1238` and `:1254-1261`. `ensure_endpoint` failing to bind returns early
in the same window.

Observable symptom, as a test would show it: after a successful chain write,
a send to an unreachable address returns `Err`, `list_orgs()[0].epoch` still
equals the pre-publish epoch while the chain's slot reads the next one, and a
second `admit_member` fails inside `submit_update` on the epoch check.
Reproduced in this change by reading, not by running: the reproducing test is
the first step of the fix, per `resolve-problem`.

Why `affects: RC-wqgm2p` on a publish-path defect: RC-wqgm2p is the control by
which a device deletes its own record when it is removed, and that control
fires only on the Wire message this defect can stop from ever being sent. A
revocation whose send fails leaves the chain saying the device is out and the
device itself never told, so RC-wqgm2p never runs on the one device the
revocation was about. The publish path is where the defect is; the control it
defeats is the self-delete.

Why it is a hazard and not only a bug: it is the publish-before-persist hazard
in org-node's register, S3/P2. A revocation of a stolen device attempted while
that device is offline is the ordinary case, and it is exactly the case in
which the administrator is then unable to act at all.

The fix, in outline (not-minted control 1 in the register): persist the record
as soon as the chain confirms the update, and make delivery a separate,
retryable step whose failure is reported without unwinding the commit.

## Loopback admission delivers to an address the joiner did not prove

**PR-2dmjzj**: in loopback transport mode `admit_member` sends the admission
Envelope and the Organisation secret to the `EndpointAddr` the caller passes
from the Join request, without checking that the address's identity is the
Device key the Join request carries, so an altered Join request hands the
secret to whoever the altered address names while the record admits the
joiner's genuine keys.
affects: RC-b6mydy
opened: 2026-09-09
status: open

Where: `org-node/src/service.rs:836-842` (loopback branch, dials `peer_addr`
as given) against `:847-849` (networked branch, derives the peer identity
from `join_request.device_key` and so binds delivery to the key). The Join
request is an unsigned, unauthenticated blob by design
(`org-node/src/blobs.rs:9-29`), travelling out of band by copy and paste, so
nothing upstream detects the alteration.

Observable symptom: with a Join request carrying B's keys and C's address,
`admit_member` succeeds, the chain's root admits B, and C's endpoint receives
the `WireMessage` carrying `org_secret`. Reproduced in this change by reading,
not by running; the reproducing test is the first step of the fix.

Why it is a hazard: HAZ-ep6uzs in org-node's register, S3/P2 — the
Organisation secret reaches a device the record does not name. The S3/P1 the
register carries for this defect belongs to its own prose entry, the admission
address, which needs the Join request altered in transit; the hazard it
instantiates is the P2 one.

The fix, in outline (not-minted control 2): refuse a `peer_addr` whose
`EndpointId` differs from the Join request's Device key, or dial by the key in
both modes.

## The revocation receive path commits from a sender it never checks

**PR-u4c2vp**: `receive_and_self_delete_if_revoked` authenticates the remote
Device key and discards it, and its `UpdatedNotRevoked` branch then takes the
same commit as `receive_and_verify` — the record, the epoch and the
high-water mark — with neither the invite cross-check on first admission nor
the membership cross-check afterwards, so a Change set relayed by a device
the record does not name shapes the node's record.
affects: RC-b6mydy
opened: 2026-09-09
status: open

Where: `org-node/src/service.rs:1337` discards the authenticated sender with
the comment "authenticated but not cross-checked here (revocation path)"; the
branch that commits is `:1339-1365`, writing the record at `:1355-1364` and
returning `SelfDeleteOutcome::UpdatedNotRevoked` at `:1365`. The two checks
this path lacks are in `receive_and_verify` at `:985-1000` (the invite) and
`:1018-1027` (the membership record the envelope produced).

Observable symptom, as a test would show it: with B admitted and a third
member C revoked by the administrator, a rogue device R that relays the
administrator's genuine revocation envelope to B is accepted — B returns
`UpdatedNotRevoked` and commits the new epoch — where the same envelope
relayed by R through `receive_and_verify` is refused with `BadSignature`.
Reproduced in this change by reading, not by running: the reproducing test is
the first step of the fix, per `resolve-problem`.

Why it is a hazard and not only a bug: it is HAZ-ep6uzs in org-node's
register, S3/P2 — a non-member's message shapes a member's record, and
relaying is what a network does. It is also the reason RC-b6mydy's second
clause is scoped to the receive operation that admits the node or updates its
record rather than to every message the node acts on.

The fix, in outline (not-minted control 4 in the register): apply the
membership cross-check on this path too, so the `UpdatedNotRevoked` branch
refuses a sender absent from the record the envelope was verified into, while
a revocation of the node's own Device key stays acceptable from any relay.

## The Wire message documents a field the revocation path always sets (2026-09-10)

**PR-w88sr9**: `WireMessage::genesis_snapshot` is documented as being `None`
for non-admission messages, and named revocations as the example, while the
revocation send path sets the field to the whole pre-change membership
snapshot on every revocation. The two statements cannot both be true, and the
doc-comment is the one that is wrong: a reader sizing the wire message, or
reasoning about what a revocation carries, is misled about both.
affects: RC-gfn6kr
opened: 2026-09-10
status: open

Where: `org-node/src/transport/wire.rs:14` ("`None` for non-admission messages
(e.g. revocations)") and the field's own comment at `:19` ("None for
non-admission"), against `revoke_member` in `org-node/src/service.rs:1213-1214`,
which encodes the pre-revoke snapshot and puts it in the `WireMessage` it
sends. The admission path sets it at `:826-832`; there is no send path in the
crate that leaves it `None`.

Observable symptom: encode a revocation's `WireMessage` and its
`genesis_snapshot` is `Some(bytes)` whose length grows with the Organisation,
not `None`. Reproduced in this change by reading, not by running: the
reproducing test is the first step of the fix, per `resolve-problem`.

Why it matters, and why it is filed rather than noted. The fact the comment
denies is load-bearing in this unit's own risk analysis. RC-gfn6kr's entry in
the register argues its introduced trigger for the publish-before-persist
hazard — S3 / P2, not acceptable — precisely from *both* send paths carrying
the whole pre-change record, which is what makes the 1 MiB frame a bound on
the size of the Organisation on the revocation path too
(`org-node/docs/risk/2026-09-09-org-node-hazards.md:545-550`, the verdict at
`:565`). Not-minted
control 15 rests on the same fact, and its cheap half — omit the snapshot on
the revocation path, where the receiver rebuilds from its own store and never
reads it — exists only because the field is set there. So the register and the
unit glossary's *Wire message* entry (`org-node/docs/CONTEXT.md`) both state
the truth while the code's own comment states the opposite. A reader who
trusts the comment concludes the revocation frame is small and the frame bound
harmless on that path, which is the conclusion this register rejects; a reader
who trusts it while writing a second sender may leave the field unset and
break nothing visible, since no revocation receiver reads it — masking the
divergence rather than resolving it.

Found by review round 7 of this change, which noticed that the change
establishes a doc-comment on a path it assesses to be wrong and records it
nowhere. It is the same category of defect as PR-hvg2dy, and filed on that
precedent: under IEC 62304 Class C the documentation of a security-relevant
path is not a lesser kind of defect than the path's behaviour.

The fix, in outline: correct the comment on the field and on the struct to say
that the snapshot is set on both send paths and read only by an admission's
receiver, which is what `org-node/docs/CONTEXT.md` already says. **The
behaviour is right and must not change** — the register's frame-bound
assessment and not-minted control 15 are written against the code as it is,
and silently making the revocation path match the comment would be
not-minted control 15 done without its analysis. Whether to omit the snapshot
on the revocation path is that control's question, not this report's.

Not fixed in this change, for the reason the three above give: this is the
risk-analysis change, and it changes no behaviour of the crate and none of the
doc-comments the reports in this ledger are about. What it does touch under
`org-node/src` is the relocation of four `#[cfg(test)]` test modules into
`org-node/tests`, the module doc-comment and lint allow on the test-support
fixtures, and the module declarations that expose them — `transport/wire.rs`
among them, for the test module only, which is why this report's fix is
deferred rather than folded in.
