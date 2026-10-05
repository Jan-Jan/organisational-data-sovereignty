# Hazard analysis — the `app` unit (the Tauri shell)

ISO 14971 analysis of the shipped desktop application: the surface through
which an administrator admits, verifies and revokes members, and through which
a member learns they have been revoked. Class C
(`docs/adr/2026-09-05-units-and-per-unit-classes.md`). The acceptability matrix
is in `app/docs/risk/README.md`; the harm pathway is
`docs/adr/2026-09-01-safety-class-c.md`.

Fourth and last of the per-unit registers begun on 2026-09-05
(`docs/plans/2026-09-05-ratchet-gap-analysis.md`, tooth 3), after
`org-members` (2026-09-02), `org-node` (2026-09-09) and `on-chain-client`
(2026-09-10).

---

## 1. Scope

**In scope.** Everything inside `strict_paths` for this unit, which this change
widens from `app/src-tauri/src` alone to include `app/src`:

- `app/src-tauri/src/state.rs` (281 lines) — `AppState::init`, the environment
  interview that decides the data directory, the store passphrase, the chain
  configuration and the transport mode; `ChainNotConfigured`, the fallback
  `ChainOps`; `connection_status_from_state`.
- `app/src-tauri/src/commands.rs` (449 lines) — twelve `#[tauri::command]`
  handlers, their serialisable DTOs, `parse_org_id`, and `start_receiver`'s
  background loop, which emitted five distinct event names from four payload
  types.
- `app/src-tauri/src/lib.rs` (45 lines) — the builder, the `setup` hook and the
  handler registration.
- `app/src-tauri/src/main.rs` (6 lines).
- `app/src` (12 files, 1,384 lines of TypeScript and Svelte, markup and styles
  included) — `lib/api.ts`, the
  typed wrappers over `invoke` and `listen`; seven components; the page and
  layout.

**Out of scope, and why.**

- The behaviour of `org-node` and `on-chain-client`. Both are declared
  dependencies with their own registers. This unit's hazards are about what it
  *does with* their answers and what it *shows* an operator, never about
  whether their answers are right.
- The contract at `on-chain/` and the formal models at `quint/`, disclaimed in
  `.guardrails/units.yaml`. (Amended 2026-10-03: the formal models moved into
  `org-members/quint/` and `org-node/quint/`
  (`docs/adr/2026-10-03-quint-conformance-gate.md`) and are now inside those
  units, out of scope here like the rest of their behaviour; only `on-chain/`
  is still disclaimed.)
- The Tauri framework, `iroh`, `subxt` and the SvelteKit toolchain, which are
  SOUP. This unit's `app/docs/architecture/soup.md` is still the empty
  template; populating it is tooth 4, and this register's reliance on Tauri's
  event delivery and the platform path resolver is an input to that work.

**The shape of the unit, because it governs what the analysis can find.** This
is glue. There is almost no computation here: the handlers lock a mutex, call
one `OrgService` method, and map `OrgNodeError` to `String`. What this unit
decides is narrow and it is all at the edges — *what to start with*, *what to
show*, *what to pass on*, *when to stop*. Every hazard below is one of those
four, and none is an algorithm being wrong.

That is not a reason to expect few hazards. It is the reason to expect these
ones: a component that computes nothing can still be the component that tells
an operator a revocation succeeded when it was never submitted.

## 2. Method, and the harm pathway this unit sits on

The chain from this unit to a person is short, which is why the unit is class C
despite being thin.

1. An administrator forms an intent — admit this person, revoke that one — or
   forms a belief: *this membership is current and chain-verified*.
2. The app is the only place either happens. There is no other client.
3. The app either carries the intent to `org-node` or does not, and either
   reports the state truthfully or does not.
4. `org-node` and `on-chain-client` do the rest, and their registers cover it.
5. A member who should have lost access and has not can read the
   organisation's documents. In the deployments this project targets —
   journalism and human-rights work, government and inter-organisational
   secrets — those documents identify sources and protected individuals.
6. The harm at the end is what happens to a person whose identity reaches
   someone who should not have it. That is **S3** under this project's matrix,
   and S3 is UNACCEPTABLE at every probability, P1 included.

Two consequences of that chain are worth stating before the hazards, because
they are why several items below are graded harder than a reader might expect
of a user interface.

**A display is a control.** The administrator's decision to stop worrying about
a member is taken on what the screen says. A verification indicator that cannot
show a failure is not a cosmetic defect; it is a removed check, and it is
removed at exactly the step where a human would otherwise have caught what the
software got wrong.

**An unreachable action is an absent action.** A safety function that exists in
the backend and cannot be invoked from the only user interface is not
implemented. HAZ-n97v5g is that case, and it is graded as though the function
were missing, because from the operator's side it is.

**How the hazards were found.** By reading all 781 lines of Rust and all 1,384
lines of frontend against four questions — what does this start with, what does
it show, what does it pass on, when does it stop — and then, for each answer,
asking what an operator would conclude and whether they would be right. No
hazard below was found by a test, because this unit had no tests when the
analysis began. That is stated plainly rather than buried: the analysis is the
first verification this unit has ever had.

**Corrected 2026-09-14.** Both sentences above said "~350 lines of frontend",
an estimate written before the files were counted and never re-measured; the
verification gate's round 4 measured 1,384. The Rust figure was exact
(281 + 449 + 45 + 6), which is what made the mismatch findable. The correction
does not change what was read — the analysis did cover these files, and §6's
prose hazards cite line-level details from three of the seven components — but a
number inside a completeness claim has to be the measured one, or the claim is
worth less than it appears. About half the 1,384 is markup and scoped CSS; the
`<script>` blocks plus `api.ts` and `+layout.ts` are 742 lines, and that is the
part the four questions were asked of.

## 3. Where the evidence is, and where it is not

This unit began this change with **zero tests**. `cargo test` compiled two
targets and ran nothing, which `app/.guardrails/config.yaml` already described
honestly as "a compile proof, no more". `app/src-tauri/tests` held a
`.gitkeep`. There was no frontend test runner at all.

It ends the change with two gated suites and a test path on each side. What
each can and cannot reach is the most important thing in this document, so it
is set out before the hazards rather than after.

### What the evidence now covers

**Extracted decisions, tested directly — but not every decision.** Most of the
decisions named in the requirements below were moved out of the place they could
not be reached from — an `async` Tauri handler, or a Svelte component's
`$effect` — into a free function, and gated there.

**Three were not, and this sentence claimed otherwise until the second
independent review measured it.** In each case the requirement's *consequence*
is gated and the requirement's *decision* is not:

- **REQ-2k7ys4 and REQ-dp95pv.** The decision is "the organisation record for
  this update cannot be read" — the `list_orgs().iter().find(…)` that returns
  `None`, inside `next_outcomes`, an `async fn` over `AppHandle`. It was never
  extracted. What `receiver_events.rs` gates is `emissions_for`, which is
  *handed* an already-chosen `RecordUnreadable` and asked only what to emit. So
  "a record that cannot be read produces no membership event" is gated;
  "a record that cannot be read is recognised as such" is not.
- **REQ-jfxah3.** The decision "the loop is stopping" is the `TERMINAL_ERROR`
  substring match, and the requirement's ordering clause — *before* the loop
  exits — is the emit-then-`break` sequence. Neither is extracted; the two tests
  carrying `verifies: REQ-jfxah3` assert only that a `Stopped` outcome maps to
  one `receiver-stopped` emission. The substring match is separately recorded as
  claim 7 and as REQ-x3c8n2; the *ordering* is recorded nowhere until now.

These are added as claims 12 and 13 below. The distinction matters because §3's
whole purpose is to say what the evidence reaches, and "the decision was
extracted and gated" is the strongest thing this section claims about any
requirement — it should not have been said of requirements where only the
downstream mapping was gated. That is the same structural move `on-chain-client` needed for
`log_is_ours`, made for the same reason and with the same justification: a
decision that no gate can reach is a decision no gate protects, and relocating
it is cheaper and more honest than asserting it by inspection.

- Rust, in `app/src-tauri/tests`, gated behind `required-features =
  ["test-support"]` so that a target named in the gate cannot silently fail to
  run — all six: `startup_policy.rs`, `connection_status.rs`,
  `org_id_parsing.rs`, `receiver_events.rs`, `receiver_guard.rs`, and `ipc.rs`,
  whose different character is the subject of the next two paragraphs.
- Frontend, in `app/tests`, run by vitest: `revoke.validate.test.ts`,
  `verify.row.test.ts`, `receiver.subscribe.test.ts`, `receiver.log.test.ts`.

**Four command handlers, driven across the real IPC boundary; twelve
registered.** `app/src-tauri/tests/ipc.rs` uses `tauri::test::mock_builder`
with the real `invoke_handler` registration from `lib.rs`, so a handler it
invokes is reached the way the frontend reaches it, with the actual argument
names. This is what catches the class of defect no extracted-logic test can: a
command registered under one name and invoked under another, an argument the
frontend spells `peerAddrBlob` and the handler expects as something else, a DTO
field renamed on one side only. Those are wiring defects, they are exactly what
a thin shell is made of, and until this change nothing in the repository would
have caught one.

**But it catches them for four commands, not twelve, and the distinction is
the whole point of the paragraph above.** The suite invokes
`connection_status`, `export_invite`, `revoke_member` and `list_personas`, plus
one deliberately unknown name to show the handler list is closed. The other
eight — `create_persona`, `create_organisation`, `import_invite`,
`export_join_request`, `import_join_request`, `admit_member`, `list_orgs`,
`start_receiver` — are *registered* in the mock application and never invoked,
and registration is not exercise: a renamed argument or a renamed DTO field on
any of those eight would still pass every gate here.

The four were chosen because they are the ones reachable without a chain, a
peer or a populated store. The eight are not an oversight; they are the part of
the boundary this repository cannot currently drive, and they are listed here by
name so that nobody has to infer which is which. This sentence used to read
"twelve command handlers, driven end to end" — the verification gate's round 4
measured it and found four. Corrected rather than softened, because a register
that overstates its own evidence is the exact defect HAZ-9fmhm4 is about.

### What the evidence does not cover, stated as gaps

**No component renders in any test.** The frontend suite tests extracted
functions, not Svelte components. Nothing asserts that `Membership.svelte`
actually calls `verifyResultFrom`, that the ✗ branch is actually wired to the
template, or that `Revoke.svelte` actually calls the validator this change
gives it. A defect that leaves the tested function correct and the component
calling something else would pass every gate here. This is the single largest
hole in the change and it is listed as not-minted control 1, with a date.

**No chain, no peer, no second machine.** Nothing in either suite touches a
live chain or a real `iroh` endpoint. The receiver-event tests drive the event
*construction* from synthesised outcomes; they do not drive `OrgService`. So
every claim of the form "when org-node returns X, the app does Y" is tested on
the app's half and assumed on org-node's.

**No test observes the `setup` hook, and none calls `AppState::init`.**
`tauri::test::mock_builder` enters the application past `setup`, so the startup
refusals RC-xt4qr3 and RC-djzms3 are tested one layer below where a user would
meet them: `startup_policy.rs` calls `policy::resolve_passphrase` and
`policy::resolve_data_dir` directly, and that is where the decision lives —
`init` reads the environment and hands what it read to those two total
functions, which decide. The IPC suite builds its state through
`AppState::for_test`, which calls the shared `assemble` path and skips both
resolvers by construction.

So nothing exercises `init`'s own wiring: that it reads the three variables it
claims to, passes them in the right order, and propagates the refusal rather
than swallowing it. That is a genuine hole and it is the same shape as the
guard-placement negative in claim 8 — the decision is gated, its single call
site is not. PR-w5dae4 records that the consequence, when it does fire, is
currently a panic.

*(This paragraph originally said the refusals "are tested at `AppState::init`,
which is where the decision lives". Both halves were wrong — no test calls
`init`, and `state.rs`'s own module doc says the policy functions are what
decide. Found by the verification gate, round 4.)*

**No coverage figure exists for this unit.** `app/.guardrails/config.yaml` has
no `coverage_command`, and this change deliberately does not add one — that is
tooth 5, and the owner's decision on 2026-09-14 was to keep one measurement
basis for `org-node` and `app` rather than set a floor here in isolation.
Under class C the absence is a gap against a mandatory requirement, and it
remains recorded in `docs/plans/2026-09-05-ratchet-setup.md`. Nothing in this
register should be read as resting on a coverage number, because there is none.

**Decision coverage is unmeasured, here as in every unit.** Class C asks for
statement *and* decision coverage. No unit in this repository measures the
second.

### Claims held by review, not by a gate

Thirteen statements in this document are true as far as reading can establish and
are protected by no test. They are collected here so that a later reader does
not have to infer which is which:

1. That the twelve commands registered in `lib.rs` are the twelve the frontend
   calls in `api.ts`. The IPC suite drives them, but a thirteenth command added
   to one side only would not fail anything.
2. That `ChainNotConfigured`'s three methods are unreachable once
   `build_chain_ops` succeeds.
3. That no code path other than the identified ones assigns a verification
   outcome. *(Corrected 2026-09-14: this read "other than the two identified",
   which described the pre-change component. Post-change there are three — the
   two `kind: 'verified'` sites reached from `onMembershipUpdated` and
   `onIncomingVerified`, and the `kind: 'failed'` site. The claim is unchanged
   in substance; its count was of the code this change replaced.)*
4. That Tauri's `emit` failures are confined to the cases the analysis
   assumed — a closed webview, a serialisation failure — and do not include a
   silently dropped event under load.
5. That the platform path resolver's error cases are the ones the analysis
   assumed, on the three desktop platforms.
6. That the `ODS_*` variables are read in exactly the places this document
   names and nowhere else.
7. That the receiver's terminal markers appear in `org-node`'s error messages
   in exactly the situations the loop treats as terminal. This one is the
   sharpest, is the subject of REQ-x3c8n2, and is discussed at HAZ-cfp4jb.

   **This claim was not merely ungated — until 2026-09-15 it was false, and the
   third independent review measured it.** The loop matched the single substring
   `"endpoint not bound"`, which occurs nowhere in `org-node` or `iroh`. The
   messages that actually arrive are `"endpoint bind: …"`, `"endpoint bind
   failed unexpectedly"` and `"endpoint closed"` (the last reaching the app as
   `chain read failed: iroh recv: iroh accept error: endpoint closed`). So the
   loop had never broken, `receiver-stopped` had never been emitted in
   production, and the guard had never been released by task exit. The mechanism
   was dead on arrival and three statements in this register rested on it
   working.

   It is live now — the markers match the messages that occur, and three tests
   pin them — and it is no sounder for being live: matching a provider's
   formatted text is the defect, and REQ-x3c8n2 remains the fix. What changed is
   that the claim above is now about a mechanism that fires.
8. **That the receiver loop actually holds the start guard for its lifetime.**
   Added after the fact, because it was *measured* rather than assumed — and
   the measurement is the reason it is on this list instead of being taken for
   granted. `StartGuard`'s release-on-drop is gated three ways at the unit
   level, panic included (`receiver_guard.rs` holds six tests, but three of them
   verify REQ-6hgm8r's mutual exclusion rather than REQ-3hfggn's release). Its placement at the only call site is gated by nothing: T2
   deleted the line that moves the guard into the spawned task and **all six
   test targets stayed green**, the sole signal being a compiler warning about
   an unused variable.

   Nothing in this repository can run the receiver loop — it needs a bound iroh
   endpoint, an `OrgService` holding a real organisation, a chain and a peer —
   so no test observes whether the guard is there. REQ-3hfggn is therefore
   verified as a property of `StartGuard` and assumed at the point where it
   matters. This is the second time a prescribed mutation in this project has
   come back green (`on-chain-client`, 2026-09-11), and unlike that one it is
   not remedied by further extraction: what is unreached here is the
   `tokio::spawn` itself, not a decision that could be lifted out of it. It is
   part of why HAZ-cfp4jb's residual probability stays at P2.
9. **That the connection status is not re-read from the process environment**
   — REQ-bvx4nh's second clause, and the half of RC-a7fenm that removes the
   contradiction described at HAZ-ny7yvt. It is true **by construction rather
   than by test**: `connection_status_from_state` takes the data directory, the
   chain endpoint and the transport mode as parameters, and no `std::env::var`
   call is reachable from it. No test establishes it, and one deliberately is
   not written: cargo runs integration tests as threads of a single process, so
   a test that set `ODS_CHAIN_WS` would race every other test in the binary and
   fail intermittently, blaming whichever test happened to read it.

   This entry exists because the gate caught the evidence claiming more than it
   held. A test named `unconfigured_chain_reports_false_even_with_env_set` never
   set the environment — it only read whatever the variable ambiently held, to
   quote in a failure message — so wherever `ODS_CHAIN_WS` is unset, which is CI
   and every run to date, the clause in its name was vacuous. It has been
   renamed to state what it asserts, and the clause it does not reach is
   recorded here instead of being implied by a name.

   *(The independent review pushed back on this, rightly: the new name,
   `absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env`,
   still ends in a clause that reads as coverage over environments, and the
   test still only reads one ambient value to quote in a failure message. The
   body comment is exact; the name is the part a reader scanning the suite
   sees. The name is left as it stands and the overclaim is recorded here
   rather than chased through a third rename — but the pattern is worth
   naming: two successive attempts to name this test honestly both reached
   for a clause about the environment, because the tempting thing to say is
   what the test would establish if it could.)*
10. **That a listener registered after teardown began is unsubscribed** —
   REQ-rq8g2v's operative clause. `subscribeAll` awaits every `listen` call
   before it can return a cleanup, so at the helper's own interface the
   clause is unreachable by construction, and the test named for it awaits
   `subscribeAll` before calling cleanup, so no teardown ever overlaps a
   pending registration. What actually implements the clause is a `torndown`
   flag inside `Membership.svelte` and `Revoke.svelte`, and no test renders a
   component. The test is a valid reproduction of PR-u34uqm's
   array-emptying defect and does not reach the clause its name claims.
   Found by the independent review; another consequence of not-minted
   control 1.
11. **That `verify_failure_carries_the_organisation_when_known` tests a
   behaviour the system exhibits.** It constructs a payload production code
   cannot construct, because no `OrgNodeError` variant carries an
   organisation. It verifies the payload's shape and could not fail if the
   behaviour broke. See HAZ-9fmhm4's residual and REQ-x3c8n2.
12. **That an unreadable organisation record is recognised as one.**
   REQ-2k7ys4 and REQ-dp95pv's decision lives in `next_outcomes` and is reached
   by no test; only its consequence in `emissions_for` is gated. See the
   correction above.
13. **That the receiver announces its stop *before* exiting.** REQ-jfxah3's
   ordering clause is the emit-then-`break` sequence in the spawned task, which
   nothing drives — the same unreachable region as claim 8's guard placement.
   The two tests carrying the ID assert the mapping from a `Stopped` outcome to
   one emission, not the order in which the real loop does it.

---

## 4. Hazards

### 4.1 Revocation cannot be submitted through the only user interface

**HAZ-n97v5g**: A user interface precondition that the protocol cannot satisfy,
where an administrator in the default Networked transport attempts to revoke a
member and the revoke form refuses to submit because it demands a peer address
that a Networked join request never carries, so the revocation is never
submitted and the member retains access to documents identifying sources and
protected individuals. Severity: S3. Probability: P3.

The sharpest item in this register, and the app's counterpart to the spoofing
defect `on-chain-client` fixed on 2026-09-11 — not because the mechanism is
similar but because in both cases a doc-comment described a safety behaviour
the code did not perform.

`revoke_member` in `commands.rs` documents `peer_addr_blob` as optional, in as
many words: "leave it empty in Networked mode, where the revoked member is
reached by EndpointId (derived from its device key in the trie) via iroh
discovery. It is only needed for same-machine Loopback dialing." The handler
implements exactly that, mapping an empty string to `None`.

`Revoke.svelte` then refuses to call it:

    if (!peerAddrBlob.trim()) { revokeErr = 'Peer addr blob (hex) is required.'; return; }

And the value it demands cannot be obtained. In Networked transport
`admit_member`'s own comment records that the blob carries no address —
"in `Networked` transport the joiner is reached by its `EndpointId` … so the
blob carries no embedded address" — so `JoinRequestDto.node_addr_blob` is the
hex encoding of nothing, the empty string. The Admit panel even displays this
to the operator as "No (no p2p addr)", in amber, and calls it a warning.

So the operator is asked for a value the system has just told them does not
exist, by a form that will not proceed without it. There is no path through the
user interface to a Networked revocation.

**Probability P3, and why it is not lower.** This is not a failure mode that
sometimes occurs. `TransportMode::Networked` is the default — `AppState::init`
selects it for anything other than the literal `ODS_TRANSPORT=loopback`, and
its own comment calls it "the right choice for two laptops over the internet on
live Paseo", which is the deployment. Every revocation attempted in the shipped
configuration meets this. The only escapes are running the demo transport, or
an operator typing arbitrary hex into the field to get past the check — which
would produce a `postcard` decode error one layer down, so it does not escape
either.

**Severity S3.** Revocation is the safety action of this system. Its absence is
the exact failure the 2026-09-01 ADR describes: a member whose access should
have ended and has not.

**RC-8ygnjd**: Report the transport mode the running service was configured
with in the connection status handed to the frontend, so that a decision
depending on the transport mode can be made from the reported mode rather than
guessed. mitigates: HAZ-n97v5g

**RC-jrkn7w**: Derive the revoke form's peer-address precondition from the
reported transport mode, requiring an address only in Loopback, and validate
the member identifier the operator does supply. mitigates: HAZ-n97v5g

Realised by REQ-645jq9 (the backend reports the mode), REQ-vgr7s2 (the
precondition follows the mode), REQ-he8ejb (the identifier is checked),
REQ-sjkp8z (the organisation identifier is checked at the backend boundary).

**Why two controls and not one.** The obvious fix — delete the check — is
wrong, because the address genuinely *is* required in Loopback, where discovery
does not apply and there is nothing to dial without it. Deleting the check
would move the failure from "cannot revoke in Networked" to "revocation
silently fails to dial in Loopback", trading a visible block for an invisible
one, which is the worse of the two. The precondition has to become conditional,
and to be conditional it needs a fact the frontend did not have. RC-8ygnjd
supplies the fact; RC-jrkn7w uses it.

**Residual risk: S3 / P1. UNACCEPTABLE.** Probability falls from P3 to P1
because the blocked path is opened and the Loopback case keeps its check.
Severity does not move: severity is a property of the harm, not of the control.
The residual is unacceptable under this project's matrix, as every S3 item is.

What keeps it above zero rather than at zero: the frontend precondition is
tested as a function, and nothing tests that `Revoke.svelte` calls it
(not-minted control 1). A future edit could reintroduce the unconditional check
in the component and no gate would object.

### 4.2 The verification display cannot report a failure

**HAZ-9fmhm4**: A verification indicator structurally incapable of showing a
negative result, where an administrator consults the membership panel to
confirm that inbound updates verified against the on-chain root and every row
reads ✓ MATCH regardless of what happened, so a divergence between local
membership and the chain is believed to have been checked and ruled out when it
was never displayed. Severity: S3. Probability: P3.

`Membership.svelte` describes its table as "THE KEY PoC OUTPUT" and heads it
"Verified Updates (chain root match)". It renders

    {v.verified ? '✓ MATCH' : '✗ MISMATCH'}

and assigns `verified` at exactly two sites, both the literal `true` — once in
the `membership-updated` handler and once in `incoming-verified`. No input to
the component produces `✗ MISMATCH`. The string exists in the source and is
unreachable.

The failure path is not absent, it is elsewhere and unlabelled: a verification
failure arrives as `receiver-error` carrying a single `message: String`, and
the component appends it to a separate list of timestamped strings below the
table. That list carries no organisation, no epoch and no root, so it cannot be
correlated with the table above it even by a reader who knows to try.

The result is a display that answers a question it never asked. The operator's
reasonable reading of "three rows, all ✓ MATCH" is *three updates arrived and
all three verified*. The true reading is *three updates arrived that verified;
how many arrived and failed is not shown here, and the ones that failed are
not identifiable*.

**Probability P3.** The grading is of the control's absence, not of the
frequency of an underlying verification failure. The indicator is incapable of
the negative result on every row it has ever rendered and every row it ever
would render; there is no run in which the control is present. Grading it P2 on
the argument that a true ✓ is harmless when nothing went wrong would be grading
the hazard as though it were "the display is sometimes wrong", which is not
what this is — what this is, is that the check the operator believes they are
reading is not being performed.

**Severity S3.** This display is the last human checkpoint before an
administrator stops examining a membership. Removing it removes the step at
which a wrong membership would have been caught.

**RC-6ajdv4**: Emit verification failures as a typed event carrying the
organisation identifier wherever the failing update names one, rather than as
an untyped message string. mitigates: HAZ-9fmhm4

**RC-h7mnfj**: Derive each verification log row's outcome from the event that
produced it, render the failure outcome, and move the subscription lifecycle
out of the component so that both can be tested. mitigates: HAZ-9fmhm4

**RC-3rddh7**: Classify a receive-path failure by the error's type, and report
a verification verdict only for an error that is one. mitigates: HAZ-9fmhm4

RC-3rddh7 was **not** in the original analysis. It was added on 2026-09-14 after
the independent review found that RC-6ajdv4 and RC-h7mnfj, as first implemented,
had created a second instance of this same hazard pointing the other way — see
§5, which now carries five entries rather than four. Recording it as a control
on HAZ-9fmhm4 rather than as a new hazard is deliberate: the hazard is *the
verification display misrepresenting the verification state*, and "cannot show a
failure" and "shows failures that are not verification verdicts" are two ways for
one display to do that.

Realised by REQ-affyf5 (the typed failure event), REQ-a83vqr (a failure renders
as not-verified), REQ-akt4p7 (the outcome comes from the event and nothing
else), REQ-rq8g2v (the subscription helper unsubscribes what it subscribed).

REQ-rq8g2v is in this control rather than in a control of its own because the
restructuring the first three require is what exposed PR-u34uqm, and a
subscription helper that leaks listeners produces duplicate rows in the very
table this hazard is about. A verification log that shows each result twice is
a different way of being untrustworthy, and it would have been introduced by
the fix had the fix not addressed it.

**Residual risk: S3 / P2. UNACCEPTABLE.** The ✗ state becomes reachable and
derived. Probability falls from P3 to P2 rather than to P1 for a reason that has
to be stated: **the app does not decide whether verification succeeded —
`org-node` does** — and the app's new failure event is only as informative as
the error `org-node` hands it. The display now reports what it is told; whether
it is told enough is the provider's half, and no requirement in this unit can
settle it.

**Corrected 2026-09-14 by the independent review: "and failures gain an
identifier" was false, and is withdrawn.** REQ-affyf5 is worded "where the
failing update names one", and the honest reading of that qualifier turns out to
be *never*. The only construction site in production hard-codes `org_id: None`,
and it has to: no `OrgNodeError` variant carries an organisation identifier, so
there is nothing for the app to pass on. The requirement is satisfied — vacuously
— and the reduction the sentence claimed does not occur in any run.

Two further consequences, both recorded rather than smoothed:

- The gating test `verify_failure_carries_the_organisation_when_known`
  constructs `VerifyFailed { org_id: Some(…) }`, a value production code cannot
  produce. It is a test of the payload's shape, not of a behaviour the system
  exhibits, and it could not fail if the behaviour broke. It is kept — the shape
  is what a future `org-node` change would fill in — but it is named in §3's
  claims list rather than counted as evidence of a reduction.
- **The "failures are unlocatable" half of this hazard is therefore not reduced
  at all.** That was the sharper half: a failure with no organisation, no epoch
  and no root cannot be correlated with the table above it. What this change
  delivers is that a failure is now *visible* and *typed*; making it
  *locatable* needs the provider to say which organisation failed, and that is
  added to REQ-x3c8n2's scope below.

### 4.3 Placeholder values are emitted as measured ones

**HAZ-5ha5vv**: Hard-coded sentinel values emitted through the channel reserved
for measured ones, where the receiver completes a verification but cannot read
back the organisation record and announces epoch 0 with an empty root hash, so
an administrator is shown a verified membership row describing a state that was
never observed and an epoch regression is indistinguishable from genesis.
Severity: S3. Probability: P2.

Two sites in `start_receiver`.

The first, after a successful verify, re-reads the org record to obtain the
epoch and root it is about to announce, and answers a lookup miss with a
substitute:

    .unwrap_or((0, String::new()))

That tuple goes into `MembershipUpdatedPayload` and out over
`membership-updated` and `incoming-verified`, identically to a measured one. The
frontend logs it with `verified: true` and renders the root as
`…{v.root.slice(-16)}` — the last sixteen characters of the empty string, which
is the empty string, so the row shows a root of `…` and an epoch of 0, marked
✓ MATCH.

The second is on the self-delete path, which emits

    EpochChangedPayload { org_id: org_id_hex, epoch: 0 }

a literal zero on an event type whose only content is an epoch. The record has
just been deleted; there is no epoch; zero is standing in for "not applicable"
on a channel with no way to express it.

**Why zero is the wrong sentinel specifically.** Epoch 0 is genesis — a real,
reachable, meaningful value. A consumer cannot distinguish "this organisation
is at its genesis state" from "this value is not available", and the second is
rendered in the affirmative. Had the substitute been an obviously impossible
value the hazard would be smaller; had it been an absence it would not exist.

**Probability P2.** The first site needs a lookup miss, which requires the
record to be absent or renamed between the verify and the read-back — a narrow
window, but one that widens exactly when something else has already gone wrong,
which is when a truthful display matters most. The second fires on every
self-delete, but its consequence is confined to an epoch display rather than a
membership claim, and the accompanying `revoked` event carries what is
genuinely known.

**RC-2t9sgv**: Emit no membership event when the organisation record for that
update cannot be read, and emit instead an event that identifies the
organisation and states that its record was unreadable. mitigates: HAZ-5ha5vv

**RC-9t3kpm**: Emit no epoch value on the self-delete path, where no epoch is
known. mitigates: HAZ-5ha5vv

Realised by REQ-2k7ys4, REQ-dp95pv and REQ-tw4cb5.

RC-2t9sgv is deliberately two behaviours in one control, and REQ-dp95pv is the
half that stops the remedy from becoming a new hazard: suppressing the wrong
announcement without substituting a true one converts "the operator is told
something false" into "the operator is told nothing", and silence about a
membership is not obviously better than a wrong statement about one. It is
better only if the silence is itself announced, which is what REQ-dp95pv
requires.

**Residual risk: S3 / P1. UNACCEPTABLE.** Both fabrication sites are removed and
the first gains a truthful replacement. Probability falls to P1 rather than
lower because the failure is now a rendering question: REQ-dp95pv's event has to
be displayed, and what displays it is a component no test renders (not-minted
control 1).

### 4.4 Key material at rest under a passphrase published in the source

**HAZ-8ghmhn**: Persona key material encrypted under a constant compiled into
the published source, where the application is run without `ODS_PASSPHRASE` —
which is every run, because no part of the user interface sets one — and the
persona store file is later read by anyone who obtains it, so an attacker
recovers the administrator's signing keys, admits themselves to the
organisation and reads documents identifying sources and protected
individuals. Severity: S3. Probability: P3.

`AppState::init`:

    let passphrase = std::env::var("ODS_PASSPHRASE")
        .unwrap_or_else(|_| "ods-dev-default".to_string());

The store at `<data_dir>/persona_store.bin` holds every persona's key material
for this installation. The passphrase protecting it, absent an environment
variable that nothing prompts for and no documentation in the app requires, is
fifteen characters present in this repository.

**Probability P3, and the reasoning is about the path rather than the
attacker.** This is the normal execution path, not a degraded one. There is no
UI affordance for a passphrase, no first-run prompt, no error, no warning —
`init` does not even log the substitution, unlike the chain fallback three
statements later, which at least prints to stderr. An operator running the
application as delivered has key material under a published constant and no
indication of it. The probability being graded is the probability of the
hazardous *situation*, and the situation is every run.

**Severity S3.** The keys are the authority. With them an attacker does not
have to defeat membership verification; they can author a membership change
that verifies correctly.

**RC-xt4qr3**: Refuse to start when no passphrase is configured, unless
development defaults are explicitly enabled, and name in the refusal both the
variable that would supply the value and the variable that would waive the
requirement. mitigates: HAZ-8ghmhn

Realised by REQ-7g3k9a and REQ-bmk2z2.

**This is ISO 14971 priority one, and the alternatives were considered and
rejected.** Inherent safety by design is to remove the unsafe state, not to
warn about it. Three weaker options were available. *Warn on stderr* — which
is what the chain fallback does, and which a windowed desktop build does not
display; it is information for safety, the weakest category, delivered through
a channel the user cannot see. *Generate a random passphrase and store it
beside the store* — which protects against a stolen file only until the
directory is copied, and this project's threat model is a copied directory.
*Prompt the user at first run* — correct, and a UI change with no test path in
this unit (not-minted control 2). The refusal is the control that can be
implemented and gated now; the prompt supersedes it later.

**The opt-in is part of the control, not a hole in it.** `ODS_ALLOW_DEV_DEFAULTS`
keeps CI, the demo and two-instance local testing working without editing
source. What makes it acceptable is that it is explicit, it is named in the
refusal message, and setting it is a recorded act rather than the default. What
would make it unacceptable is a default-on, a config file that could acquire it
silently, or a name that reads as innocuous.

**Residual risk: S3 / P1. UNACCEPTABLE.** The unsafe state becomes unreachable
without a deliberate act. P1 rather than lower: a deployment that sets the
opt-in to get past a startup failure it does not understand puts itself back in
the original state, and nothing in this unit can prevent that. The refusal
message naming the passphrase variable *first* is the mitigation for that, and
it is why REQ-bmk2z2 exists as a requirement rather than as a nicety.

### 4.5 Startup degrades silently, and the status display reports a startup fact in the present tense

**HAZ-ny7yvt**: Startup fallbacks that relocate key material and disable chain
operations without announcement, combined with a status display that reports
the startup-time verdict as current while re-reading its details live, where an
administrator reads "Chain OK" or reads a contract address beside "Chain NOT
configured" and submits a revocation believing it will reach the chain, so the
revocation never lands and the member retains access. Severity: S3.
Probability: P2.

Three distinct degradations, one hazard, because they share a situation: the
operator believes the application is configured as intended and it is not.

**The data directory.** `lib.rs` answers a path-resolver error with

    .unwrap_or_else(|_| std::path::PathBuf::from("/tmp/ods-poc"))

a fixed path that is world-readable on a shared machine, identical for every
user of that machine, and cleared by the operating system. Two harms follow and
they point in opposite directions: key material somewhere it should not be, and
a store that disappears — taking the local membership record with it, which is
the unavailability pathway `org-node`'s register treats at length.

**The chain.** A failure to build `SubxtChainOps` — absent variables, a
malformed contract address, a connect error — falls back to `ChainNotConfigured`
and prints one line to stderr. The application then runs, looking entirely
normal, with every chain operation returning an error string at the moment it
is attempted.

**The display.** `connection_status_from_state` reports `chain_configured` from
`chain_ready`, computed once during `init`, while reading `ODS_CHAIN_WS` and
`ODS_CONTRACT_H160` from the environment at the moment of the call. The two
halves are therefore independent, and the bar can show a specific contract
address beside the words "Chain NOT configured" — which an operator reads as
"configured for this contract, connection pending" rather than "nothing here
will work". It can equally show an endpoint the running configuration never
used, if the variable changed after startup.

**Probability P2.** Each degradation needs a trigger — a resolver error, absent
or wrong chain variables, a failed connect — rather than occurring on every
run. The display inconsistency, by contrast, is certain whenever the chain half
has degraded and the variables are set, which is the common shape of a
misconfiguration.

**RC-djzms3**: Refuse to start when the application data directory cannot be
resolved, unless development defaults are explicitly enabled.
mitigates: HAZ-ny7yvt

**RC-a7fenm**: Report in the connection status only the endpoint and contract
the running configuration was built from, and report both as absent when chain
operations are not configured. mitigates: HAZ-ny7yvt

Realised by REQ-rxc8sp, REQ-e4ah9h and REQ-bvx4nh.

**What these controls do not fix, stated plainly.** The chain *fallback* itself
is untouched: the application still starts without chain operations, still
prints one line to stderr, and still presents a working-looking interface. That
was a deliberate scoping decision — an organisation running deliberately
offline is a legitimate mode, and refusing to start would break it — and what
this change does instead is ensure the display tells the truth about it. And
the chain verdict is still a startup fact: a connection that drops an hour later is
still reported as healthy, because the connection is inside `SubxtChainOps`,
which belongs to `org-node`. That is PR-eecx3y, open, and the second face of
REQ-x3c8n2.

**Residual risk: S3 / P2. UNACCEPTABLE.** Probability does not fall. This is one
of the two hazards in this register whose residual probability is unchanged by
its controls — HAZ-cfp4jb is the other, for the same shape of reason — and
saying so is the point: the controls fix the data-directory
fallback and the display's internal contradiction, and they leave the largest
contributor — a stale liveness verdict — standing, because it cannot be fixed
from inside this unit. Recording a reduction here would be recording a
reduction that was not measured and did not occur.

### 4.6 A dead receiver is indistinguishable from a live one, and cannot be restarted

**HAZ-cfp4jb**: A background task whose termination is unobservable and
irreversible, where the receiver loop exits and its start guard stays claimed,
so inbound membership updates and revocations addressed to this node are never
received, the interface continues to report "Receiver running", and no restart
is possible for the life of the process. Severity: S3. Probability: P2.

Three defects compounding, in `start_receiver`.

**The guard is never released.** `receiver_started` is set by `compare_exchange`
*before* the loop is spawned and is never written again. When the loop `break`s,
the flag still reads `true`, so every later `start_receiver` returns `Ok(())`
from the idempotence branch without spawning anything. The receiver cannot be
restarted without restarting the application.

**The exit is silent.** The `break` ends the task. No event is emitted. The
frontend's badge — "Receiver running — listening for updates…" — is set when the
command returns and is never revisited.

**Termination is decided by substring.** The loop stops on

    if msg.contains("endpoint not bound") { break; }

a match against a message formatted by another unit. Every other error is
treated as transient and the loop continues. So a permanently dead endpoint
whose error is worded differently produces an infinite loop emitting one
`receiver-error` per iteration, and a reworded message in `org-node` — an
ordinary thing to do, protected by no test in either unit — silently converts
the terminal case into the infinite one.

**Why this reaches S3.** The receiver is how a revoked member's own node learns
it has been revoked and self-deletes, and how every member learns of a
membership change. A node whose receiver is dead retains the access it had at
the moment of death, and its user is shown a running receiver.

**Probability P2.** The loop exits only on a permanent endpoint error. That is
not an everyday event — but it is correlated with exactly the circumstances in
which a revocation is urgent, and the compounding is what raises it: each of the
three defects alone would be recoverable, and together they make the state
absorbing.

**RC-nyy73d**: Release the receiver's start guard when its loop exits, while
continuing to prevent a second loop from being spawned while one is running.
mitigates: HAZ-cfp4jb

**RC-8abufw**: Emit an event naming the reason the receiver loop stopped,
before it exits. mitigates: HAZ-cfp4jb

Realised by REQ-3hfggn, REQ-6hgm8r and REQ-jfxah3.

RC-nyy73d is worded with both halves because a fix for one alone is a plausible
mistake in either direction: releasing the guard without preserving exclusion
reintroduces double-spawning, and preserving exclusion without releasing is the
present code. REQ-6hgm8r exists to keep the property the guard was written for
while REQ-3hfggn changes it.

**Residual risk: S3 / P2. UNACCEPTABLE.** The state stops being absorbing — the
exit is announced and a restart works — and probability nonetheless does not
fall below P2, because the third defect is untouched. Detection still rests on
matching strings this unit does not own, and a reworded message still converts
a terminal failure into an infinite loop. That is REQ-x3c8n2, addressed to
`org-node` with a deadline of 2026-12-13, and it is also claim 7 of §3.

What the controls genuinely buy is that the failure becomes *recoverable and
visible when it is detected at all*. What they do not buy is detection.

**Corrected 2026-09-15.** That first sentence was false when written, for a
reason worse than the one the paragraph goes on to give. It was not that
detection *might* fail if a message were reworded — detection had never worked
at all. The loop matched `"endpoint not bound"`, a string `org-node` and `iroh`
never produce, so the exit was never announced, the guard was never released by
task exit, and this hazard's state was exactly as absorbing after the controls
as before them. The third independent review found it; §3's claim 7 carries the
measurement.

Two things follow, and both are worse than the register previously implied:

- **REQ-jfxah3 was satisfied vacuously**, in the same way REQ-affyf5 is
  (HAZ-9fmhm4's residual). Its tests asserted that a `Stopped` outcome produces
  one `receiver-stopped` emission, which was true; nothing produced a `Stopped`
  outcome.
- **The real behaviour was a hot spin.** `ensure_endpoint` caches the endpoint
  and rebinds only when it is `None`, so after `"endpoint closed"` the loop
  reused the dead endpoint indefinitely, emitting one error event per iteration.
  Not a quiet absorbing state — a busy one.

The markers now match the three messages that actually occur and three tests
pin them, so the sentence above is true as of this change. (fix7 added four
tests; the fourth pins variant classification under REQ-kn5rtx and never touches
the markers — a distinction gate round 9 drew and this sentence had blurred.) It is no sounder for
being true: a substring match against a provider's formatted text is the defect,
not the fix, and REQ-x3c8n2 is unchanged.

---

## 5. Hazards introduced by these controls

ISO 14971 requires asking what each control breaks. **Five answers, all real** — and the last to be found was not found by asking. The independent reviewer found it in the implementation, after this section had been written claiming four; it stands fourth below, beside the other display entries.

**Refusing to start is a denial of the safety action.** RC-xt4qr3 and RC-djzms3
convert two silent degradations into startup failures. An application that will
not start cannot revoke anybody, and the operator who most needs to revoke is
the one least able to debug an environment variable. The argument for accepting
this is that the two states being refused are states in which the application is
*already* not doing its job safely — key material under a published constant, or
a store in a directory the operating system will clear — so the choice is
between a visible failure and an invisible one, not between failure and success.
It is accepted on that basis and not on the basis that it costs nothing.

PR-w5dae4 sharpens it and is why REQ-bmk2z2 is a requirement: today the refusal
arrives as a panic inside Tauri's `setup` hook, with the reason on stderr, which
a windowed build does not show. A refusal whose reason is invisible is the worst
form of this trade. Closing PR-w5dae4 is therefore not optional polish; it is
what completes RC-xt4qr3 and RC-djzms3, and it carries the same date.

**Suppressing a fabricated announcement can produce silence.** RC-2t9sgv stops
the `(0, "")` row. If REQ-dp95pv's replacement event is not rendered — and no
test renders any component — the operator sees nothing where they previously saw
something wrong. Whether that is an improvement depends on a component this
change does not gate. Named in not-minted control 1.

**Releasing the guard opens a window.** RC-nyy73d makes `receiver_started`
writable in two places rather than one. A release that is not atomic with
respect to a concurrent `start_receiver` permits two loops, which is the defect
the guard existed to prevent and which would double every emitted event.
REQ-6hgm8r is the requirement that keeps it, and it is gated by
`concurrent_claims_yield_exactly_one_guard`, which races sixteen threads on one
flag and asserts exactly one guard is issued.

**That test races `StartGuard::try_claim`, not `start_receiver`.** The
distinction matters and this paragraph originally elided it by saying "a test
that calls the start path concurrently" — which would have contradicted §3
claim 8 of this same document, where the guard's placement at its only call
site is recorded as gated by nothing. Both statements cannot be true. The
correct one is claim 8's: the guard's *mutual-exclusion property* is gated
sixteen ways; its *use by `start_receiver`* is gated not at all, because nothing
in this repository can run the receiver loop.

**Making the failure visible made non-failures look like failures.** Added
2026-09-14, standing fourth of the five because it belongs beside the other
display entries, and the most instructive one in this document because the
control and the defect are the same edit.

RC-6ajdv4 and RC-h7mnfj were implemented by routing every `Err` from the receive
path into the new verification-failure event. That event is rendered as
`✗ MISMATCH` in the table headed "Verified Updates (chain root match)". So after
the change a chain read failure, or a transport fault, was displayed to the
operator as a root mismatch — a claim about membership the software had not
made and could not support. In the same edit the `receiver-error` event lost its
only emitter, so the panel's own "Receiver Errors (non-fatal)" list became dead
markup and had nowhere to put the errors that were now being misfiled.

HAZ-9fmhm4 is *the verification display misrepresenting the verification state*.
Before this change the display could not report a failure. After the first
implementation of its controls it reported failures that were not verification
verdicts. Those are the same hazard, and the second was created by fixing the
first — which is precisely what §5 exists to catch and precisely what §5 failed
to catch, because its author was assessing the controls as designed rather than
as built.

RC-3rddh7 is the answer: classify by the error's type, and report a verification
verdict only for an error that is one. `OrgNodeError` already distinguished them
and the code was discarding the distinction with `e.to_string()` before deciding
anything. REQ-kn5rtx gates the classification, REQ-wu6z9p gates the consumer,
and `classify_receive_error` carries no wildcard arm so a variant added upstream
is a compile error rather than a silent misfiling.

**Residual after RC-3rddh7: unchanged at S3 / P2 for HAZ-9fmhm4.** The added
control removes a defect this change introduced; it does not reduce the hazard
below where §4.2 already placed it, and recording an improvement here would be
recording one that was not measured.

**Relaxing the revoke precondition weakens the Loopback path.** RC-jrkn7w makes
the peer address optional, and in Loopback it is genuinely required. This is why
the requirement is conditional on the transport mode rather than a deletion, and
why RC-8ygnjd exists to supply the mode. If RC-8ygnjd's report is wrong — if the
status says Networked when the service is in Loopback — the form accepts an
empty address and the revocation fails to dial, silently.

**That single point is gated on the backend only.** Six tests carry
`verifies: REQ-645jq9` and all six are Rust — four in `connection_status.rs`
proving the projection reports the configured mode, two in `ipc.rs` proving it
crosses the boundary under that field name. **No frontend test verifies
REQ-645jq9.** What `StatusBar.svelte` and `Revoke.svelte` do with the value they
receive — the half where a mis-read mode actually produces the silent dial
failure — is gated by nothing, for the same reason everything else about the
components is: no test renders one (not-minted control 1).

This paragraph originally claimed the point was gated "on both sides of the IPC
boundary". It is gated on one, and since this is the argument for accepting a
hazard that RC-jrkn7w *introduces*, the difference is the difference between an
accepted risk and an assumed one. The acceptance stands — the backend half is
where the value is decided, and a wrong value there would fail — but it is
recorded now as resting on an ungated consumer.

---

## 6. Further hazards recorded in prose

Seven more were identified and are recorded without identifiers, because each
is either outside this unit's control, or is a control this change chose not to
mint. They are here so the next analysis does not have to rediscover them.

**Truncated key display defeats out-of-band verification.** `Admit.svelte`
shows the joiner's member and device keys as `…{parsed.member_key.slice(-16)}`
— the last sixteen hex characters. An administrator verifying a join request
over a second channel (a phone call, a signal message) can only compare what is
displayed, so two keys agreeing in their final sixteen characters and differing
in the first forty-eight are indistinguishable. Sixteen hex characters is 64
bits, which is not a security parameter anybody would choose deliberately for a
comparison an attacker can grind against offline. Not minted because the fix —
display the full key, or a fingerprint designed for human comparison — belongs
with the component-rendering work.

**The admin signing key travels in the process environment.** `ODS_ADMIN_SEED`
carries the 32-byte admin secret as hex. Environment variables are inherited by
every child process, are visible to other processes on some platforms, and are
routinely captured by crash reporters and process supervisors. Not minted
because the remedy is a key-storage design decision spanning this unit and
`org-node`.

**Nothing authenticates a pasted blob before the operator acts on it.**
`import_invite` and `import_join_request` accept any string. The Admit panel
shows the decoded fields and offers an "Admit Member" button; a blob crafted by
an attacker decodes to whatever the attacker chose, and the operator's only
defence is recognising the handle and name. This is a social-engineering
pathway into membership. Not minted here because the authentication that would
close it is protocol-level and belongs to `org-node`; recorded so that this
unit's register does not read as though the paste boundary were safe.

**Revocation has no confirmation step and no undo.** A mis-click on the Revoke
button submits immediately. Re-admitting the member requires a fresh join
request from them. The hazard runs the opposite way from the rest of this
register — a member losing access they should have kept, which for a care
organisation is the unavailability pathway.

**The organisation identifier has no checksum.** `parse_org_id` accepts any 40
hex characters, so a transposition produces a well-formed identifier for an
organisation that does not exist, and the operator gets a not-found error rather
than "you mistyped this". REQ-sjkp8z gates the shape; a checksum is a protocol
change.

**Table rows are keyed by array index while rows are prepended.**
`{#each verifyLog as v, i (i)}` with `verifyLog = [newest, ...verifyLog]` gives
every row a new key on each insert. Svelte's keyed-each contract is broken: the
framework re-creates rows it could have moved, and any per-row state would be
misassigned. Today the rows are stateless, so the consequence is confined to
rendering work — which is why it is prose rather than a hazard, and why it will
stop being confined the moment a row gains a control.

**The Tauri run call ends in `expect`.** `lib.rs` ends `.expect("error while
running tauri application")`. Same class as PR-w5dae4 and covered by the same
remedy.

---

## 7. Residual risk, and the conclusion for this unit

| Hazard | Before | After | Acceptable? |
|---|---|---|---|
| HAZ-n97v5g — revocation unreachable in the default transport | S3 / P3 | S3 / P1 | **No** |
| HAZ-9fmhm4 — verification display cannot report a failure | S3 / P3 | S3 / P2 | **No** |
| HAZ-5ha5vv — placeholder values emitted as measured ones | S3 / P2 | S3 / P1 | **No** |
| HAZ-8ghmhn — key material under a published passphrase | S3 / P3 | S3 / P1 | **No** |
| HAZ-ny7yvt — silent startup degradation, stale status verdict | S3 / P2 | S3 / P2 | **No** |
| HAZ-cfp4jb — dead receiver unobservable and irreversible | S3 / P2 | S3 / P2 | **No** |

**Overall residual risk for the `app` unit: UNACCEPTABLE.**

The same conclusion the other three registers reached, and for the same
structural reason: every hazard on this system's harm pathway is S3, and this
project's matrix makes S3 unacceptable at every probability including
improbable. No control in this document could have produced a different verdict,
and none was designed to. The matrix was chosen knowing this
(`app/docs/risk/README.md`, and `docs/adr/2026-09-01-safety-class-c.md`): the
conclusion is a statement that the product is not fit for its intended use
today, which is true, and the register's job is to say what stands between it
and fitness rather than to arrive at a comfortable verdict.

Two of the six do not improve in probability at all — HAZ-ny7yvt and
HAZ-cfp4jb — and both for the same reason: their largest remaining contributor
is state held by `org-node` and not reachable from here. Those two are what
REQ-x3c8n2 is for, and its deadline of 2026-12-13 is the date by which that
half has to move.

**What this change did accomplish, stated separately from the verdict**, since
an UNACCEPTABLE conclusion can otherwise read as though nothing improved:

- A safety action that could not be invoked at all can now be invoked
  (HAZ-n97v5g).
- A verification indicator that could not report a failure now can
  (HAZ-9fmhm4).
- Two sites that fabricated membership data no longer do (HAZ-5ha5vv).
- Key material is no longer protected by a constant in the published source on
  the default path (HAZ-8ghmhn).
- A unit with zero tests now has two gated suites, a frontend test runner, and
  four command handlers driven across the real IPC boundary — of twelve
  registered, the four reachable without a chain, a peer or a populated store
  (§3).

## 8. Controls identified but not minted

Fifteen controls the analysis identified and this change does not implement.
Each is a real reduction that was not taken, with an owner and a date. They are
not deferred work items in the ordinary sense: an unminted control is a risk
accepted for now, and the date is when the acceptance is revisited.

Owner for all fifteen: **Jan-Jan**. Decide by: **2026-12-14**, except where
noted.

1. **Render components in tests.** The single largest gap in this change.
   Nothing asserts that any component calls the functions this change extracted
   and gated. Needs `@testing-library/svelte` or `vitest-browser-svelte`, a DOM
   environment, and a decision about how much of SvelteKit to stand up.
   Referenced by the residual of HAZ-n97v5g, HAZ-5ha5vv and §5.
2. **Prompt for the store passphrase at first run**, superseding RC-xt4qr3's
   refusal with the control ISO 14971 would prefer: make the safe state
   reachable rather than making the unsafe one fatal.
3. **Surface startup failures in a window.** Closes PR-w5dae4 and completes
   RC-xt4qr3 and RC-djzms3. Same date as the problem report's age limit,
   **2026-10-14**.
4. **A receiver liveness query command**, so the frontend can ask whether the
   receiver is running instead of remembering that it once called
   `start_receiver`. Closes PR-h6xpnh.
5. **A chain liveness probe**, so `chain_configured` reports the present rather
   than the startup. Blocked on REQ-x3c8n2; closes PR-eecx3y.
6. **Display full keys, or a fingerprint designed for human comparison**, in
   the Admit panel. See §6.
7. **A confirmation step before revocation.** See §6.
8. **Move the admin signing key out of the process environment.** See §6.
9. **A checksummed organisation identifier**, so a transposition is rejected
   rather than resolved to nothing.
10. **Key the verification table by event identity rather than array index.**
11. **Bound the verification log and the error log.** Both are capped by
    slicing (`slice(0, 49)`, `slice(0, 19)`) in the component, which is a
    display cap rather than a retention policy, and neither is persisted — so a
    restart loses the record of what was verified.
12. **Statement and decision coverage for this unit**, with a floor. Tooth 5.
13. **Populate `app/docs/architecture/soup.md`.** Tauri, iroh, subxt and the
    SvelteKit toolchain are SOUP this register leans on. Tooth 4.
14. **Low-level requirements for the software items in this unit.** Class C
    asks for unit-level verification per item at its own interface; this
    register's requirements are all high-level. Tooth 4.
15. **A second, independent path to revocation** — a command-line tool, or a
    recovery mode — so that a defect in the single user interface cannot remove
    the safety action entirely, as HAZ-n97v5g showed it can.

---

## 9. Derived requirements assessment

`check-trace.sh` reports `UNANALYZED-DERIVED` for any item marked
`satisfies: derived` that this file does not name. All twenty-two requirements
written by this change are derived; each is assessed below.

**Realising a control named above.** REQ-645jq9, REQ-vgr7s2, REQ-he8ejb,
REQ-sjkp8z (RC-8ygnjd, RC-jrkn7w, HAZ-n97v5g); REQ-affyf5, REQ-a83vqr,
REQ-akt4p7, REQ-rq8g2v (RC-6ajdv4, RC-h7mnfj, HAZ-9fmhm4); REQ-2k7ys4,
REQ-dp95pv, REQ-tw4cb5 (RC-2t9sgv, RC-9t3kpm, HAZ-5ha5vv); REQ-7g3k9a,
REQ-bmk2z2 (RC-xt4qr3, HAZ-8ghmhn); REQ-rxc8sp, REQ-e4ah9h, REQ-bvx4nh
(RC-djzms3, RC-a7fenm, HAZ-ny7yvt); REQ-3hfggn, REQ-6hgm8r, REQ-jfxah3
(RC-nyy73d, RC-8abufw, HAZ-cfp4jb).

Each of these exists *because of* a hazard rather than alongside one, so the
assessment is not "does it introduce a hazard" in the abstract — it is whether
realising it introduces one, and §5 answers that for the five that do:
REQ-7g3k9a, REQ-rxc8sp (refusal as denial of the safety action); REQ-2k7ys4
(suppression as silence); REQ-3hfggn (a release that could permit two loops);
REQ-vgr7s2 (a relaxation that could weaken Loopback). The remaining fourteen
introduce no new hazard: they add reported facts (REQ-645jq9, REQ-e4ah9h,
REQ-bvx4nh), stop values being fabricated (REQ-dp95pv, REQ-tw4cb5), tighten
input checks (REQ-he8ejb, REQ-sjkp8z), make an existing outcome visible
(REQ-affyf5, REQ-a83vqr, REQ-akt4p7, REQ-jfxah3), preserve a property that
already held (REQ-6hgm8r), fix a leak (REQ-rq8g2v), or constrain a message's
content (REQ-bmk2z2).

assesses: REQ-7g3k9a, REQ-rxc8sp, REQ-2k7ys4, REQ-3hfggn, REQ-vgr7s2, REQ-645jq9, REQ-e4ah9h, REQ-bvx4nh, REQ-dp95pv, REQ-tw4cb5, REQ-he8ejb, REQ-sjkp8z, REQ-affyf5, REQ-a83vqr, REQ-akt4p7, REQ-jfxah3, REQ-6hgm8r, REQ-rq8g2v, REQ-bmk2z2

**REQ-kn5rtx and REQ-wu6z9p**, added after the independent review, are assessed
here too. Both realise RC-3rddh7, and RC-3rddh7 exists *because* §5's fourth
entry is a hazard the earlier controls introduced — so the assessment is not
whether they add a hazard but whether they remove one, and they do: a failure
that is not a verification verdict is no longer reported as one. Neither
introduces a new hazard of its own. REQ-kn5rtx narrows what may be claimed and
cannot make the display say more than before; REQ-wu6z9p moves a class of
message out of the verification log into a list that already existed to hold it.
The one thing to watch is that `classify_receive_error` is total over
`OrgNodeError` with no wildcard arm, which is deliberate — a variant added
upstream must be classified explicitly rather than defaulting — and that
totality is what a future `org-node` change will meet as a compile error.

assesses: REQ-kn5rtx, REQ-wu6z9p

5 + 14 = 19 for the requirements written by the original analysis, plus
REQ-kn5rtx and REQ-wu6z9p from the review round = 21 ordinary requirements;
REQ-x3c8n2 is the twenty-second and is assessed separately below. *(This paragraph said "four" and "sixteen" until the
verification gate's round 4 counted them: the enumerations were right and both
totals were wrong, "sixteen" having been reached as 20 − 4, which counts
REQ-x3c8n2 into the ordinary set and under-counts the §5 set by one. In a
section whose entire purpose is that every derived item is individually
accounted for, the arithmetic is part of the claim.)*

**REQ-bvx4nh specifically** is worth one further sentence, because it removes a
live read: after it, the status no longer reflects an environment variable
changed after startup. That is the intent — the display is about the running
configuration — but it means an operator who corrects `ODS_CHAIN_WS` and
expects the bar to update will not see it until restart. Assessed as no new
hazard: the previous behaviour showed a value the software was not using, which
is the hazard being removed, and the restart is the same restart the corrected
variable requires anyway.

assesses: REQ-bvx4nh

**REQ-x3c8n2**, the expectation on `org-node`, introduces no hazard in this
unit because it changes no behaviour here: it records a need. Its hazard
relevance is the reverse — it is the item on which HAZ-cfp4jb's and
HAZ-ny7yvt's unreduced probabilities depend, and §4.6 and §4.5 say so. If it
goes unmet past 2026-12-13, neither residual improves and both stay exactly
where this register leaves them.

assesses: REQ-x3c8n2
