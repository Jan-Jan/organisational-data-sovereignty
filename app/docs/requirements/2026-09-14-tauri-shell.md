# Requirements — the Tauri shell, from the 2026-09-14 hazard analysis

The `app` unit's first requirements. Every one of them exists because the
hazard analysis in `app/docs/risk/` demanded a control or recorded a need, so
**all twenty-two carry `satisfies: derived`** — there are no exceptions — and every
one is assessed in that register's "Derived requirements assessment" section,
where silence is `UNANALYZED-DERIVED` at this unit's own gate.

*(This header originally said "with two exceptions noted below". There are none,
and none was ever noted below; the register's §9 says "all twenty-two" and is
correct. Found by the verification gate, round 4. The count was twenty when that
note was written; the independent review later added REQ-kn5rtx and REQ-wu6z9p.)*

**What "the software" means here.** This unit is a desktop shell in two
languages: a Rust Tauri backend at `app/src-tauri/src` and a SvelteKit frontend
at `app/src`. Both are inside `strict_paths` as of this change. A requirement
below may therefore be realised on either side, and several are realised on the
frontend alone — that is new, and it is why this change also adds a frontend
test runner. Where it matters which side owns the behaviour, the requirement
says so.

**A note on citations.** This unit depends on `org-node` and
`on-chain-client`. Neither exports any requirement, so nothing below names one
of their REQ, HAZ, RC, SDD, LLR or PR identifiers — a consumer's reference to a
non-exported item is `NON-EXPORTED-REF` on every run, and to an undeclared one
`UNDECLARED-DEPENDENCY`. Where the text leans on a provider's behaviour it
names the file to read. The one exception is REQ-x3c8n2, which is not a
citation but an expectation addressed to `org-node`, and the grammar for that
is `expects:`.

---

## What the operator is shown about the connection

**REQ-645jq9**: The software shall report, in the connection status returned to
the frontend, which transport mode the running service was configured with.
(implements: RC-8ygnjd)
satisfies: derived

The frontend cannot decide whether a peer address is required without knowing
the transport mode, and today it cannot ask: `ConnectionStatus` carries
`chain_configured`, `chain_ws`, `contract_h160` and `data_dir`, and nothing
about transport. `AppState::init` already computes the mode from
`ODS_TRANSPORT` and hands it to the service; this requirement is that the value
be reported rather than discarded. It is the prerequisite for REQ-vgr7s2, and
it is written separately because it is the backend's obligation and REQ-vgr7s2
is the frontend's.

**REQ-e4ah9h**: The software shall report an endpoint and a contract address in
the connection status only when chain operations are configured, and shall
report both as absent otherwise. (implements: RC-a7fenm)
satisfies: derived

**REQ-bvx4nh**: The software shall report, as the connection status's endpoint,
contract address and data directory, the values the running configuration was
built from, and shall not re-read them from the process environment when the
status is requested. (implements: RC-a7fenm)
satisfies: derived

*Checked 2026-10-08 (change `worktree-org-io-create`, task T9); text
unchanged.* The chain is now built by org-io's `OrgIo::connect`. The app
records, as the endpoint, the values it handed that call, and only when the
call succeeded; when it fails the handle is `OrgIo::not_configured` and no
endpoint is recorded. So "configured" (REQ-e4ah9h) and "the values it was
built from" (REQ-bvx4nh) hold as before, and "no endpoint ⇔ not configured"
is unchanged (owner ruling of 2026-10-07: the read stays built with the
write in S2).

**Four amendments made 2026-09-14 by the independent review, before any of
these items was first merged** — so they are edits in place, not supersessions;
nothing here has ever been merged for a `superseded-by:` to point at.

- **REQ-bvx4nh gained "and data directory".** The change stopped the
  `connection_status` handler re-deriving the data directory from the
  environment — master re-resolved it from `ODS_DATA_DIR` / `app_data_dir()`,
  duplicating `init`'s logic so the two could disagree, and the handler now
  reports the directory the store was actually opened in. That is the same
  behaviour REQ-bvx4nh requires of the endpoint and the contract, it was
  implemented, and it was covered by no requirement: unmarked derived work. The
  requirement is widened to name what the code does rather than a derived item
  being minted for a third field of one struct.
- **REQ-sjkp8z and REQ-he8ejb gained "optionally preceded by a single `0x`
  prefix".** Both read "not exactly 40 / 64 hexadecimal characters", and `x` is
  not a hexadecimal character — yet `a_zero_x_prefix_with_forty_characters_after_it_is_accepted`
  asserts a 42-character input is accepted, and `accepts a 0x-prefixed member id`
  asserts a 66-character one is. The permission lived only in the rationale
  prose, and PR-5mc4d8's whole single-versus-repeated-strip argument is anchored
  to that prose. An implementer working from the item form alone would have
  written a parser that rejects what these tests require it to accept. The word
  "single" is load-bearing and is what PR-5mc4d8's two fixed sites now enforce.
- **REQ-he8ejb gained "ignoring surrounding whitespace", 2026-09-14, found by
  the second independent review.** The same defect class as the bullet above,
  and it survived the first fix. The item said "not exactly 64 hexadecimal
  characters, optionally preceded by a single `0x` prefix" — whitespace is
  neither a hexadecimal character nor the permitted prefix — while
  `validateRevokeInput` calls `.trim()` and `trims surrounding whitespace before
  measuring` asserts that `"  bb…bb\n"` is **accepted**. So the item form
  forbade what the implementation and its own test both require. The code
  behaviour is the right one and is not what changed: an operator pastes an
  identifier out of a terminal or a chat window and it arrives with a trailing
  newline, and refusing that teaches nothing and loses the revocation. What
  changed is the wording, because the item form is the thing an implementer
  works from — the permission had been living only in the code and the test.
  Note that this settles the question for **this** requirement only: REQ-sjkp8z
  says nothing about whitespace and `parse_org_id` does not trim, so the two
  identifier parsers still disagree. That disagreement is recorded here and
  left for adjudication rather than resolved by this edit.

These two are separate because they fail separately, and the current code fails
both. `connection_status_from_state` calls `std::env::var("ODS_CHAIN_WS")` and
`std::env::var("ODS_CONTRACT_H160")` at the moment the status is requested,
independently of `chain_ready`. So the status bar can display a WS endpoint and
a contract address beside the words "Chain NOT configured" — the operator sees
a specific contract and reasonably reads the pair as "configured for this
contract, connection pending", when in fact no chain operation will succeed and
that contract address was never used for anything. It can also display an
address the running configuration did not use at all, if the variable changed
after startup. REQ-e4ah9h removes the first failure and REQ-bvx4nh the second;
either alone leaves the other.

## Whether a revocation can be submitted

**REQ-vgr7s2**: The software shall accept a revocation request with no peer
address when the transport mode is not Loopback, and shall require a peer
address only when it is Loopback. (implements: RC-jrkn7w)
satisfies: derived

The frontend's obligation, and the one that reopens the safety action. Today
`Revoke.svelte` rejects an empty `peerAddrBlob` unconditionally
(`if (!peerAddrBlob.trim()) { revokeErr = 'Peer addr blob (hex) is required.'; return; }`)
while `revoke_member`'s own doc-comment states the field is optional and needed
only for same-machine dialing. In Networked transport — the default, and the
mode `AppState::init` describes as "the right choice for two laptops over the
internet on live Paseo" — a join request carries no `node_addr`, so
`node_addr_blob` is the empty string and the operator has nothing to paste.
Revocation is unreachable. See HAZ-n97v5g.

The requirement is written as a property of the *decision*, not of the form, so
it can be tested without rendering: the check is a function of the transport
mode and the two input strings.

**REQ-he8ejb**: The software shall reject a revocation request whose member
identifier is not exactly 64 hexadecimal characters, ignoring surrounding
whitespace and an optional single `0x` prefix, and shall do so before any
command is invoked. (implements: RC-jrkn7w)
satisfies: derived

The abnormal-input half of the same control, and not merely defensive: with
REQ-vgr7s2 relaxing the peer-address precondition, the member identifier
becomes the only field the frontend validates at all. `revoke_member` in Rust
does check the width, so this is a second line rather than the only one — but
the frontend's check is what produces a message the operator can act on instead
of an error string surfaced from a hex decoder.

## What the verification log is allowed to claim

**REQ-a83vqr**: The software shall render a verification-failure event as a log
row whose outcome is "not verified". (implements: RC-h7mnfj)
satisfies: derived

**REQ-akt4p7**: The software shall derive a verification log row's outcome from
the event that produced the row, and from no other source. (implements: RC-h7mnfj)
satisfies: derived

The two halves of making the ✗ state reachable, and again they fail separately.
Today `Membership.svelte` builds its row with the literal `verified: true` at
both of its only two assignment sites, so `✗ MISMATCH` cannot be rendered by any
input — while the panel's own comment calls the ✓/✗ column "THE KEY PoC OUTPUT"
and its heading calls the table "Verified Updates (chain root match)". An
operator reading it cannot distinguish "every update verified" from "failures
happened and are in the other list". REQ-akt4p7 alone would leave nothing to
derive a `false` from, because no failure event reaches the component;
REQ-a83vqr alone would leave the success rows still asserting a constant. See
HAZ-9fmhm4.

**REQ-affyf5**: The software shall emit, when an inbound update fails
verification, an event that carries the organisation identifier where the
failing update names one. (implements: RC-6ajdv4)
satisfies: derived

The backend half that gives REQ-a83vqr something to render. Today a failed
verify becomes `receiver-error` with a single `message: String` — no
organisation, no epoch, no root — and the frontend appends it to a flat list of
timestamped strings. That list cannot be correlated with the verification table
above it, which is why a failure is not merely unmarked but unlocatable.

"Where the failing update names one" is a real qualification and not a hedge:
some failures happen before an organisation is identified (an endpoint receive
error, a malformed envelope). Those carry no identifier and the requirement
does not invent one. What it forbids is discarding an identifier the failure
did have.

**REQ-kn5rtx**: The software shall report a receive-path failure as a
verification failure only when the underlying error is a verification verdict,
and shall report every other failure as a receiver error.
(implements: RC-3rddh7)
satisfies: derived

**REQ-wu6z9p**: The software shall render a receiver error outside the
verification log, and shall assign it no verification outcome.
(implements: RC-3rddh7)
satisfies: derived

Added 2026-09-14 after the independent review, and they exist because the first
implementation of RC-6ajdv4 and RC-h7mnfj got this wrong. Every `Err` from the
receive path was mapped to a verification failure and rendered as `✗ MISMATCH`
in the table headed "Verified Updates (chain root match)" — so a chain read
failure or a transport fault was displayed to the operator as a root mismatch.
The `receiver-error` event the previous code used for recoverable failures lost
its only emitter in the same edit, which left the panel's "Receiver Errors
(non-fatal)" block as dead markup.

The classification was available and was being discarded: `OrgNodeError` is a
typed enum, and the code called `e.to_string()` before deciding anything.

**Which variants are verdicts was got wrong at the first attempt, and the third
independent review corrected it.** The seven that are: `OrgIdMismatch`,
`BadSignature`, `StaleSeq`, `MalformedDelta`, `DeltaBaseMismatch`,
`RootMismatch`, `StaleEpoch` — each produced only inside
`verify_envelope_against_chain`, none reachable without an envelope having
arrived. The two that were wrongly included are `OrgNotOnChain` and `Trie(_)`:
both are *also* raised by purely local conditions in the receive path — the
local store having no record of the organisation, and the local members snapshot
failing to reconstruct — with no bearing on whether anything verified. Left as
verdicts, a local-state problem rendered as `✗ MISMATCH` under "Verified
Updates (chain root match)", which is the exact defect this control was minted
to remove, reintroduced by the control that removed it.

Both are now classified as receiver errors. The app cannot tell the two origins
apart from the variant, and the rule is that a verification verdict must not be
claimed where it cannot be supported: a misfiled verdict is the hazard, while a
misfiled receiver error is only less specific. (`Trie(_)` turned out to have
four origins, two of them local, so the exclusion is if anything under-argued.) `classify_receive_error`
now matches on the variant and carries **no wildcard arm**, so a new variant
added upstream is a compile error here rather than a silent default into
whichever class happens to be listed first.

*(Amended 2026-10-05.)* The list of verdicts above describes the code before
org-node dropped the Envelope signature; `classify_receive_error`
(`app/src-tauri/src/events.rs`) is the current list. The rule is unchanged:
a verdict is a refusal of what the sender delivered, judged against the
chain or the node's own record, and each verdict is still produced only by
`verify_envelope_against_chain`. The seven verdicts are `OrgIdMismatch`,
`StaleSeq`, `MalformedDelta`, `DeltaBaseMismatch`, `RootMismatch`,
`StaleEpoch` and `SeqNotEpoch` (a Sequence number other than the chain's
epoch). `BadSignature` is gone with the signature, and nothing replaces it:
org-node checks nothing about the sender (owner ruling 2026-10-05). The
receiver errors are `Chain(_)`, `OrgNotOnChain`, `Trie(_)`,
`InvalidOrgPublicKey` (the chain's Organisation state carries a key that is
not a valid X25519 key, refused before anything is verified against it) and
`InvalidField { .. }`. `StaleEpoch` stays a verdict for the reason given
below.

How `StaleEpoch` fits: it compares two things the sender did not deliver,
the epoch the chain reports and the node's last committed epoch. It is still
a verdict on the delivered update. An update is a change from the node's
record to a newer chain state, and `StaleEpoch` says the chain holds no state
newer than the node's record for this update to be the change to. That
judges the update against the chain and the record, and no local fault
produces it.

REQ-wu6z9p is the frontend half and is separate because it fails separately: a
correct classification still misleads if the consumer files both classes in one
list. It is gated through `applyReceiverEvent`, extracted from `Membership.svelte`
for the same reason every other frontend decision in this change was — no test
renders a component.

**REQ-rq8g2v**: The software shall unsubscribe exactly the event listeners it
registered when a verification view is torn down, including listeners whose
registration completed after teardown began. (implements: RC-h7mnfj)
satisfies: derived

Written because the restructuring REQ-a83vqr and REQ-akt4p7 need — moving
subscription out of the component so the row-building logic can be tested —
also fixes a defect that was there before, recorded as PR-u34uqm. The clause
about late registration is the whole requirement: the old shape returned its
cleanup synchronously while the `listen` calls it was meant to cancel were
still pending, so the cleanup emptied an array the pending subscriptions then
refilled, and the refilled listeners were never cancelled by anything.

## What the receiver is allowed to report

**REQ-2k7ys4**: The software shall not emit a membership-updated event when the
organisation record for that update cannot be read. (implements: RC-2t9sgv)
satisfies: derived

**REQ-dp95pv**: The software shall emit, when an organisation record cannot be
read after a successful verification, an event identifying that organisation
and stating that its record was unreadable. (implements: RC-2t9sgv)
satisfies: derived

Today `start_receiver` re-reads the org record to fill in the epoch and root it
is about to announce, and on a lookup miss substitutes `(0, String::new())`:

    .unwrap_or((0, String::new()))

That tuple is emitted through the same channel as a measured one, so the
frontend logs a verified membership row at epoch 0 with an empty root hash and
marks it ✓. The operator is shown a fabricated state — and epoch 0 is not an
obviously impossible value, it is genesis. REQ-2k7ys4 stops the fabrication;
REQ-dp95pv is what stops the fix from turning a wrong announcement into no
announcement at all. See HAZ-5ha5vv.

**REQ-tw4cb5**: The software shall not include an epoch value in the event it
emits when a persona self-deletes on revocation. (implements: RC-9t3kpm)
satisfies: derived

The other half of the same hazard. The self-delete path emits
`EpochChangedPayload { org_id, epoch: 0 }` — a hard-coded zero standing in for
"not applicable", on an event type whose entire purpose is to report an epoch.
The record is gone; there is no epoch to report; the requirement is that none be
reported. A consumer that needs to know the persona self-deleted has the
`revoked` event, which carries what is actually known.

**REQ-3hfggn**: The software shall release the receiver's start guard when the
receiver loop exits. (implements: RC-nyy73d)
satisfies: derived

**REQ-6hgm8r**: The software shall spawn no second receiver loop while a
receiver loop is running. (implements: RC-nyy73d)
satisfies: derived

`receiver_started` is an `AtomicBool` set by `compare_exchange` before the loop
is spawned and never written again. It is doing two jobs — "do not double-spawn"
and, by accident, "a receiver exists" — and it is only correct at the first. The
loop can exit (it breaks on a permanent endpoint error), after which the guard
still reads `true`, so `start_receiver` returns `Ok(())` without spawning and
the receiver can never be restarted for the life of the process. REQ-6hgm8r
preserves the property the guard was written for; REQ-3hfggn is what it was
missing. They are separate requirements because a fix for either alone is a
plausible mistake: releasing the guard without preserving exclusion reintroduces
double-spawning, and the current code is the other error.

**REQ-jfxah3**: The software shall emit an event naming the reason the receiver
loop stopped, before that loop exits. (implements: RC-8abufw)
satisfies: derived

Releasing the guard makes a restart *possible*; this is what makes anyone aware
one is needed. Today the loop's exit is silent: it `break`s and the task ends,
the frontend's "Receiver running — listening for updates…" badge stays up, and
no event is emitted. See HAZ-cfp4jb.

## What the app refuses to start without

**REQ-7g3k9a**: The software shall fail to start when no passphrase is
configured for the persona store, unless development defaults are explicitly
enabled. (implements: RC-xt4qr3)
satisfies: derived

**REQ-bmk2z2**: The software shall name, in the failure it returns when a
required startup value is missing, the environment variable that would supply
it and the variable that would waive the requirement. (implements: RC-xt4qr3)
satisfies: derived

Today an unset `ODS_PASSPHRASE` silently yields `"ods-dev-default"` — a literal
in the published source — as the key protecting every persona's key material.
It is not an edge case; it is the normal path, because nothing in the UI sets a
passphrase. This is ISO 14971's first priority, inherent safety by design:
remove the state rather than warn about it. See HAZ-8ghmhn.

REQ-bmk2z2 exists because a refusal the operator cannot act on is a different
failure, not a fixed one — and because this change deliberately converts two
silent degradations into startup failures while PR-w5dae4 records that a startup
failure is currently delivered as a panic. Naming the variable is what makes the
message worth surfacing once PR-w5dae4 is closed.

**REQ-rxc8sp**: The software shall fail to start when the application data
directory cannot be resolved, unless development defaults are explicitly
enabled. (implements: RC-djzms3)
satisfies: derived

The same treatment for the other silent fallback. `lib.rs` currently answers a
path-resolver error with `std::path::PathBuf::from("/tmp/ods-poc")` — a
world-readable location on a shared machine, cleared by the operating system,
and identical for every user of that machine. Two harms follow: key material
somewhere it should not be, and a persona store that vanishes, taking the local
membership record with it. See HAZ-ny7yvt.

## Input at the command boundary

**REQ-sjkp8z**: The software shall reject an organisation identifier that is not
exactly 40 hexadecimal characters, optionally preceded by a single `0x` prefix,
before it is used to address any command. (implements: RC-8ygnjd)
satisfies: derived

The app's boundary check for every organisation-addressed command.
`parse_org_id` already enforces the length and delegates the alphabet to
`hex::decode`; this requirement pins both as behaviour rather than as an
implementation detail, and it is the item under which the abnormal-input cases
class C requires for a boundary parser are gated — the empty string, 39 and 41
characters, non-hexadecimal characters at each position, an odd-length string,
and the `0x` prefix that the code strips and the requirement therefore permits.

It implements RC-8ygnjd rather than standing alone because the control is about
what the backend is willing to act on, and the transport-mode report and the
identifier check are the two halves of that boundary.

## What this unit needs from org-node and does not have

**REQ-x3c8n2**: The software shall rely on org-node to distinguish, in the
error it returns from the receive path, a permanent failure of the transport
endpoint from a transient one, so that the receiver loop's decision to stop can
be made from the error's type rather than from the text of its message.
expects: org-io
opened: 2026-09-14
satisfies: derived

*Re-addressed 2026-10-08 (change `worktree-org-io-create`, task T9).* It was
`expects: org-node`. The app no longer depends on org-node: it reaches the
receive path through org-io, whose `receive_and_self_delete_if_revoked`
returns org-node's error type unchanged, so org-io is the one unit this
expectation can be addressed to (an `expects:` on a unit the app does not
depend on is UNDECLARED-DEPENDENCY). What is expected is unchanged, and so
are the ID and the `opened:` date (deadline 2026-12-13). org-io answers it
when stage S4 moves the transport and the receive loop into it. Read
"org-node" below as the unit the expectation was first addressed to; "Met
when" now means when org-io defines an exported requirement carrying
`satisfies:` this item. Classification under the hybrid rule: a
clarification (the provider the app reaches the same error through), not a
change of meaning — flagged for the owner, since the unit owing the work
changes.

Why this is an expectation and not this unit's own requirement. The receiver
loop must decide, on every error, whether to keep looping or to stop. Today it
decides by substring:

    if msg.contains("endpoint not bound") { break; }

That is a match against a formatted message produced by another unit, which no
gate in either unit protects. Reword the message in `org-node` — a reasonable
thing to do to an error string, with no test anywhere that would notice — and
this app loops forever against a permanently dead endpoint, emitting a
`receiver-error` event per iteration, while the operator's badge reads
"Receiver running". Harden it the other way, by matching more strings, and the
coupling gets worse rather than better.

The app cannot fix this from its own side. `OrgNodeError` is org-node's type;
the receive path is org-node's; and a classification the app derives by
inspecting strings is the defect, not the remedy. What the app needs is a
predicate on the error — a variant, or a method answering "is this terminal?" —
and only the provider can supply one.

RC-8abufw and RC-nyy73d are what this change can do from inside the unit: the
loop's exit is announced (REQ-jfxah3) and the guard is released (REQ-3hfggn), so
that a stop the app *does* detect is visible and recoverable. Neither makes the
detection itself sound. The substring match stays, and stays named, until this
expectation is met.

The same gap has a second face recorded as PR-eecx3y: the chain verdict cannot be
re-evaluated after startup either, because the connection lives inside
`SubxtChainOps`, which is likewise org-node's. Both are the same shape — the app
holds the display and the decision, the provider holds the state — and both are
covered by the deadline this item carries. The expectation is worded around the
receive path because that is the one with a live workaround in the code; if
org-node's answer takes the form of a health-and-error surface covering both, so
much the better.

**Scope widened 2026-09-14 by the independent review, before this item was first
merged.** The expectation was written around error *classification* — permanent
versus transient. The review found a second thing the app needs from the same
surface and cannot get: **which organisation a failed verification belongs to.**
REQ-affyf5 requires the app to carry an organisation identifier on a failure
"where the failing update names one", and no `OrgNodeError` variant carries one,
so the qualifier resolves to never and the requirement is satisfied vacuously.
The consequence is recorded at HAZ-9fmhm4's residual: the "failures are
unlocatable" half of that hazard is not reduced by this change at all.

Both needs are the same shape and the same provider — the app holds the display
and the decision, org-node holds the state — so they are one expectation rather
than two. An answer that gives the receive path a typed error carrying the
organisation where one is known satisfies both halves at once.

Met when org-node defines an exported requirement carrying
`satisfies: REQ-x3c8n2`. Until then this unit's gate reports UNMET-EXPECTATION
and org-node's reports one open expectation standing against it, which is the
intent: the prompt lands on the unit that owes the work. At
`expectation_age_days: 90` the deadline is **2026-12-13**.
