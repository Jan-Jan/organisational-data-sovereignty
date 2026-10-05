# Problem reports — the `app` unit, opened by the 2026-09-14 risk analysis

Six anomalies. Four were found by reading `app/src-tauri/src` and `app/src` for
the hazard analysis in `app/docs/risk/`; PR-5mc4d8 was found by the verification
gate afterwards and widened by the task that fixed the part of it a requirement
covered; PR-j9f6kk was found by the second independent review. One of the six is
resolved by this change; five stay open with dated checklist items in
`docs/plans/2026-09-05-ratchet-setup.md`.

Each is a defect in the product as it stands, not a gap in the process. Where
the analysis found a *missing control* rather than a *wrong behaviour*, it is
recorded in the hazard register's not-minted-control list instead — a need is
not an anomaly.

---

**PR-w5dae4**: `AppState::init` failure terminates the app by panicking inside
Tauri's `setup` hook, so a user whose persona store cannot be opened sees a
process abort rather than a diagnosable message.
affects: RC-xt4qr3, RC-djzms3
opened: 2026-09-14
status: open

`app/src-tauri/src/lib.rs` wraps the call as
`AppState::init(tauri_data_dir).unwrap_or_else(|e| panic!("AppState init failed: {e}"))`.
Every failure `init` can return — a data directory that cannot be created, a
store whose passphrase does not decrypt it, and, after this change, the two
refusals RC-xt4qr3 and RC-djzms3 add — arrives at the user as a panic. The
message reaches stderr, which a windowed desktop build does not show.

This matters more after this change, not less: RC-xt4qr3 and RC-djzms3 turn two
silent degradations into startup failures deliberately, and a startup failure
whose reason is invisible trades a silent wrong state for a silent dead app.
The refusals are still the right control — a dead app cannot mislead an
operator about membership, which is the harm being controlled — but the
diagnosis has to become visible before the fix is complete. The fix is a
dialog, or a window that renders the error, and it is a UI change this analysis
did not scope.

Not fixed here because the remedy is a Tauri window-lifecycle change with no
test path in this unit today: the `tauri::test` mock runtime drives commands
through the IPC boundary and does not exercise the `setup` hook's failure
branch, so a test asserting "the user sees the reason" cannot be written
against the mock. It needs the component-rendering harness the register lists
as a not-minted control.

---

**PR-h6xpnh**: `receiverStarted` is component-local state in
`Membership.svelte`, so the receiver's status badge and its start button report
the component's history rather than the receiver's.
affects: RC-nyy73d, RC-8abufw
opened: 2026-09-14
status: open

Switching away from the "S4: Membership" tab and back destroys and remounts the
component, resetting `receiverStarted` to `false`. The UI then offers "Start
Receiver Loop" for a receiver that is already running; clicking it calls
`start_receiver`, which returns `Ok(())` from its idempotence guard without
spawning anything, and the component sets `receiverStarted = true` and displays
"Receiver running — listening for updates…". The badge is therefore true by
coincidence, and would read identically if the receiver had never started.

RC-nyy73d and RC-8abufw fix the backend half — the guard is released when the
loop exits, and the exit is announced — so a dead receiver now stops claiming
the slot and says that it stopped. What remains is that the *frontend* has no
way to ask "is the receiver running right now?": there is no query command, only
the fire-and-forget `start_receiver`. Closing this needs a
`receiver_status` command, which is a new command and a new requirement rather
than a control on an existing one.

Until then the badge means "this component has called `start_receiver` at least
once since it mounted", which is not what it says.

**PR-h6xpnh addendum, 2026-10-05: two more symptoms of the same root cause.**
Found while decomposing the unit (`docs/plans/2026-10-05-app-architecture.md`,
owner ruling 2). They are booked here rather than as new reports because the
cause is the same: the receiver's state lives in the component, not in the
receiver.

- **The start race.** `doStartReceiver` (`Membership.svelte:165-166`) sets
  `receiverStarted = true` after `await startReceiver()` resolves.
  `start_receiver` spawns the loop and returns. If the loop exits and its
  `receiver-stopped` event reaches the component before the command's reply
  does, the handler sets `receiverStarted = false` and records the reason, and
  then the resumed `doStartReceiver` sets it back to `true`. The badge then
  reads "Receiver running" for a loop that has stopped, and the stop reason is
  hidden, because it renders only while `receiverStarted` is false. This is
  HAZ-cfp4jb's situation, which RC-8abufw's announcement was meant to close.
  Tauri does not order an event against a command's reply, so the window is
  real, though narrow.
- **Events lost on a tab switch.** `+page.svelte:51-63` mounts exactly one
  panel at a time, so `Membership` is destroyed whenever another tab is shown,
  and its listeners go with it (REQ-rq8g2v, by design). Every receiver event
  emitted while another tab is open, whether a ✓ row, a ✗ row, a receiver
  error or `receiver-stopped`, reaches no listener and is never shown. The log
  and error lists are component state too, so returning to the tab also clears
  what was shown before. A failed verification that happens while the operator
  is on the Revoke tab leaves no trace in the interface (HAZ-9fmhm4).

Neither has a pin. Both live in Svelte components and the route, which no
gated test mounts (SDD-6g3wnh in the 2026-10-05 decomposition). The remedy is
the one this report already names, receiver state owned outside the
component, extended to the event history the panel displays.

---

**PR-eecx3y**: `chain_ready` is computed once during `AppState::init` and never
re-evaluated, so a chain connection that drops after startup is reported as
healthy for the remaining life of the process.
affects: RC-a7fenm
opened: 2026-09-14
status: open

`build_chain_ops()` runs exactly once, inside `init`, and its outcome is stored
on `AppState` and never revisited. Nothing re-checks the WS connection, and
nothing invalidates the stored outcome when a chain call subsequently fails. The
status bar's green "Chain OK" therefore reports a fact about startup in the
present tense, indefinitely.

*(Wording corrected 2026-09-14, before this report was first merged. It
originally named the field as `AppState.chain_ready` and said
`connection_status` "returns that stored boolean". This change deleted
`chain_ready` — the state is now `chain_endpoint: Option<ChainEndpoint>`, and
`connection_status` reports `chain_configured: chain.is_some()`, so the verdict
and the endpoint can no longer disagree, which is RC-a7fenm. The **defect is
unchanged**: whichever shape holds it, it is still decided once at startup. Only
the field name was wrong, and a problem report naming a field that does not
exist is one a future reader cannot act on.)*

RC-a7fenm narrows the damage rather than repairing it: after this change the
endpoint and contract shown beside the verdict are the ones the running
configuration actually used, so the bar can no longer display a contract
address beside "Chain NOT configured". The verdict itself is still a startup
fact.

The honest fix is a liveness probe — a cheap chain read on a timer, or the
connection's own disconnect signal surfaced as state — and it is not
implementable in this unit alone: `SubxtChainOps` is `org-node`'s type and the
connection it holds is not exposed. That makes it a requirement on the
provider, and this change records it as one: REQ-x3c8n2 carries
`expects: org-node`.

Named in the hazard register under HAZ-ny7yvt, whose probability rests partly
on this report staying open.

---

**PR-u34uqm**: `Membership.svelte` and `Revoke.svelte` each returned an
`$effect` cleanup function synchronously while the subscription setup it was
meant to clean up ran un-awaited, so event listeners leaked and accumulated
across tab switches.
affects: RC-h7mnfj
opened: 2026-09-14
status: resolved

**Scope widened 2026-09-14, before this report was first merged.** It was
written naming `Membership.svelte` alone, because that is where the analysis
found the defect and where its consequence — duplicated rows in the
verification log — is visible. While rewiring the components, T4 of
`docs/plans/2026-09-14-app-risk-analysis.md` found `Revoke.svelte` carrying the
identical defect in miniature: a single `onRevoked` subscription assigned to
`unlisten` after an un-awaited `await`, so the synchronous teardown had nothing
to call and every visit to the Revoke tab left one live listener behind. Both
are now routed through `subscribeAll`.

The second instance is worth recording rather than folding in silently, because
it says something about the defect's nature: it is not a mistake someone made
once in a complicated component, it is what this `$effect`-plus-`async`-setup
shape does every time it is written. A third component written the same way
would have it too.

Root cause: the effect body called `setup()` — an `async` function that pushes
each resolved `UnlistenFn` into the shared `unlisteners` array — and then
returned `() => { unlisteners.forEach(u => u()); unlisteners = []; }` without
awaiting it. Svelte runs the returned cleanup synchronously on unmount, so on a
tab switch the cleanup could run while `setup()`'s awaits were still pending:
it emptied an array the pending subscriptions then pushed into, leaving five
live listeners attached to a destroyed component and unreachable by any later
cleanup. Each remount added five more. The observable symptom is duplicated
rows in the verification log — the display whose trustworthiness HAZ-9fmhm4 is
about — growing by one copy per visit to the tab.

Fixed by RC-h7mnfj's restructuring: subscription setup moved out of both
components into `subscribeAll()` in `app/src/lib/receiver.ts`, which awaits
every `listen` call and resolves to a single cleanup closure over the completed
array. Each component awaits that promise and stores the closure, so a cleanup
can no longer run against a half-populated array.

Reproduced by `app/tests/receiver.subscribe.test.ts` — `unsubscribes exactly the
listeners it registered, including those whose listen call resolved late`. The
reproduction was confirmed by mutation rather than only by construction: T4
restored the old shape inside `subscribeAll` (cleanup returned before awaiting,
unlisteners pushed on `.then` into an array the synchronous cleanup empties) and
watched that test fail with `expected [ 'fast' ] to deeply equal [ 'fast',
'slow' ]`, plus two further tests in the same file, then reverted to green.

---

**PR-5mc4d8**: Five remaining boundary parsers strip a leading `0x` repeatedly
rather than once, so malformed input carrying a doubled prefix is silently
normalised into a valid value instead of being refused.
affects: REQ-sjkp8z, REQ-he8ejb, REQ-bvx4nh
opened: 2026-09-14
status: open

Found by the verification gate's round 4 in `app/src-tauri/src/parsing.rs`, and
then — while fixing that — found in six more places by the fix task itself,
seven sites in all.
`str::trim_start_matches` removes *every* leading occurrence, so `"0x0x"` plus a
well-formed body is accepted wherever a single `0x` would be.

Two of the seven are fixed in this change, because a requirement governs them and
a test could therefore be annotated: the organisation identifier
(`parsing.rs`, REQ-sjkp8z) and the member identifier (`commands.rs`,
REQ-he8ejb). Both now use `strip_prefix("0x").unwrap_or(…)`, and the doubled
prefix is refused by the width check that follows. The remaining five are listed
here rather than fixed, because no requirement covers them and a fix without a
requirement is a test that cannot be annotated — `check-trace.sh` would report
the change, correctly, as behaviour with no requirement behind it.

**Across the IPC boundary — operator input, the sharper two:**

- `commands.rs`, `org_secret_hex` in `admit_member`. `hex::decode` of the
  stripped string, then `bytes.len() != 32`. `"0x0x"` plus 64 hex is normalised
  into a valid 32-byte organisation secret; one strip would refuse it at the
  decode. This is the one worth fixing first: it is optional key material an
  operator pastes by hand, which is exactly where a transcription error is
  likely and where silent acceptance is least welcome.
- `commands.rs`, `peer_addr_blob` in `revoke_member`. Over-accepted in exactly
  the same way as the other four: `peer_addr_blob.trim().trim_start_matches("0x")`
  removes *both* prefixes, so `"0x0x"` plus the hex of well-formed postcard bytes
  decodes cleanly and the revocation proceeds. Under the single-strip rule the
  fixed sites now follow, the surviving `"0x"` reaches `hex::decode` and is
  refused on the `'x'`.

  *(This bullet originally said the blob "has no width requirement and `postcard`
  is the real check, so a doubled prefix fails at decode either way", and called
  it the weakest of the five. That was wrong — asserted from the shape of the
  code rather than traced through it — and the verification gate's round 5
  measured it. The site is correctly listed; only its justification was false.
  It is still the least consequential of the five, because a peer address is
  opaque dialling information rather than an identity or a secret, but "least
  consequential" is not "unchanged by the fix".)*

**At startup, from the environment — three, all over-acceptance only:**

- `ODS_CONTRACT_H160` (`state.rs`). Doubled prefix plus 40 hex passes the
  `len() != 40` check after both strips; a single strip would leave 42 and
  refuse it. **The visible consequence is not the address but the display**:
  the raw, unstripped string is what is stored into `policy::ChainEndpoint` and
  therefore what `connection_status` reports, so the app would talk to `0x…`
  while showing the operator `0x0x…`. REQ-e4ah9h and REQ-bvx4nh remain
  satisfied — the reported value genuinely is the one the configuration was
  built from — but an operator checking the connection row against their records
  sees a string that is not a valid H160. This is HAZ-ny7yvt's family: the
  status display saying something an operator will misread.
- `ODS_ADMIN_SEED` (`state.rs`). Same shape against `len() != 64`. Accepted,
  never displayed, and the same keypair is derived either way — so the only
  consequence is that a transcription error in admin key material is absorbed
  rather than reported.
- `ODS_COSIGNER_PUB` (`state.rs`). Differs: there is no character-width check,
  the value is decoded first and `bytes.len() != 32` checked after. A doubled
  prefix is still stripped and accepted; a single strip would have the hex
  decoder refuse it (`Invalid character 'x'`). Refused either way under a single
  strip, just by a different message.

**What this is not.** Over-acceptance, never mis-parsing. `x` is outside the hex
alphabet, so `0x` cannot occur inside a well-formed body, and no identifier can
be parsed as a *different* identifier: `"0x0x"` plus 40 hex names the same
organisation as `"0x"` plus 40 hex. Nobody revokes the wrong member because of
this. The defect is a class C boundary parser silently normalising malformed
input, which is a robustness failure rather than a route to harm — which is why
it is a problem report and not a hazard.

**Widened 2026-09-14: the same two parsers also disagree about whitespace.**
Found while fixing the second independent review's findings, and measured rather
than inferred. `validateRevokeInput` (`app/src/lib/revoke.ts`) calls `.trim()`,
so a member identifier padded with spaces and a trailing newline is accepted.
`parse_org_id` (`app/src-tauri/src/parsing.rs`) does not trim: it strips one
optional `0x` and then checks the width, so a padded organisation identifier is
rejected at the width check for a length of 43. A leading space also defeats the
prefix strip, because `strip_prefix` sees `"  0x…"` — so a padded, prefixed
identifier fails for its length rather than for its whitespace.

Neither requirement is wrong about its own parser. REQ-he8ejb was amended to say
"ignoring surrounding whitespace"; REQ-sjkp8z says only "optionally preceded by a
single `0x` prefix", and its implementation agrees with it. What is missing is
any item saying which policy the *unit* intends — and `org_id_parsing.rs` has no
whitespace case at all, so nothing would notice if the answer changed.

This belongs in the same report rather than a new one because it has the same
remedy, and because settling one without the other would leave the pair
inconsistent in the opposite direction. Resolving it either way makes the two
parsers agree: trim in `parse_org_id` and widen REQ-sjkp8z, or drop the trim in
`revoke.ts` and revert REQ-he8ejb's fourth amendment. The first is the likelier
answer — a pasted identifier arrives with a newline — but it is a decision, not a
defect to fix quietly at the end of a long change.

Closing it means either minting a requirement for each boundary and fixing under
it, or one requirement covering prefix **and whitespace** handling at every
operator-supplied identifier in the unit. The second is the better shape and it
is the reason this is one report rather than six.

---

**PR-5mc4d8 addendum — two verdict variants compare the envelope against local
state.** Found by the task that reclassified `OrgNotOnChain` and `Trie(_)`, and
recorded here rather than acted on because it is a requirement decision, not a
classification defect. `DeltaBaseMismatch` and `RootMismatch` are produced by
comparing the incoming envelope against the *local* trie, so a corrupt but
reconstructable local snapshot yields a verdict against a good envelope — the
operator is shown `✗ MISMATCH` for an update that was fine. Both remain
classified as verification verdicts, which is defensible: verification *is* a
verdict on the pair, and an envelope that does not apply to this node's state
has genuinely failed to verify here. But it is the same family as the two
variants just reclassified, and whether the display should distinguish "your
copy is wrong" from "their update is wrong" is a question no item currently
answers. Closing PR-5mc4d8's requirement should settle it.

---

**PR-j9f6kk**: Three frontend behaviours added by this change are gated under
requirements that do not describe them, and the unit glossary declares two terms
as defined in the root glossary where they are not defined at all.
affects: REQ-a83vqr, REQ-jfxah3, REQ-dp95pv
opened: 2026-09-14
status: open

Found by the second independent review. Recorded rather than fixed, on the
owner's decision of 2026-09-14: the change had already run two review rounds and
six gate rounds, the code is sound, and minting requirements for each item is
work better done deliberately than at the end of a long change. Each item below
is a real gap, and the reason it is one report rather than six is that they
share a cause — the frontend grew behaviour faster than the ledger grew items.

**Three behaviours with no requirement of their own.** All three are new in this
change; `git show master:` on each file confirms none existed before.

- `app/src/lib/verify.ts` substitutes `'(unknown organisation)'` when a failure
  event names no organisation. That is a *substitution decision* at exactly the
  place HAZ-5ha5vv's reasoning says substitutions must be deliberate and minted —
  the register argues at length that `0` was the wrong sentinel for a missing
  epoch because `0` is genesis, and then introduces a string sentinel for a
  missing organisation without an item. It is gated under `verifies: REQ-a83vqr`,
  which is about a row's *outcome*, not its identity field.
- The same file's `detail` field and the `Detail` column it feeds in
  `Membership.svelte`. New markup carrying the failure message REQ-affyf5
  produces; gated under REQ-a83vqr, which does not mention it.
- `Membership.svelte` clears the "Receiver running" badge and shows the reason
  when `receiver-stopped` arrives. REQ-jfxah3 is purely an emission requirement
  — "shall emit an event naming the reason" — and nothing requires a consumer.

**The asymmetry is the finding, not the individual items.** For `receiver-error`
this change *did* mint the frontend twin, REQ-wu6z9p, and argued in its own
rationale that it was "separate because it fails separately": a correct
classification still misleads if the consumer files both classes in one list.
That argument applies unchanged to `receiver-stopped` and to `record-unreadable`
(whose "Verified, but Record Unreadable" block is likewise unrequirement-ed),
and §5 of the register explicitly reasons about whether REQ-dp95pv's event is
rendered — while no item requires that it is.

**Two glossary terms declared elsewhere and defined nowhere.** `app/docs/CONTEXT.md`
says Persona and Epoch "live in the root `docs/CONTEXT.md` and are not repeated
here". The root glossary defines neither. Epoch is the subject of REQ-tw4cb5 and
half of HAZ-5ha5vv; Persona is the subject of HAZ-8ghmhn. Two of this register's
most load-bearing terms are undefined in both glossaries, each pointing at the
other.

**And the unit glossary's own term is not used.** `app/docs/CONTEXT.md` mints
**Operator** with `_Avoid_: user, admin, end user`, and this change's ledger then
uses "user" in that sense nine times across the register, the problem reports and
the requirements. A glossary that the ledger written alongside it does not follow
is not yet a glossary.

**Six items, and the reason this is one report rather than six separate ones**
is that they share a cause: the frontend grew behaviour faster than the ledger
grew items, and the glossary was written before the ledger that uses it. The six
are the three behaviours above, Persona undefined, Epoch undefined, and the
`_Avoid_` list the ledger does not follow.

Closing this means minting items for the three behaviours (or recording a
reasoned decision that a consumer requirement is not wanted for events whose only
consumer is a display), defining Persona and Epoch in the root glossary where
both units' documents can cite them, and a pass over the ledger for the
`_Avoid_` list. Age limit 2026-10-14.
