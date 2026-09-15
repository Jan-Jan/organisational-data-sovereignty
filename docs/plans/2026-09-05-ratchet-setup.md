# Ratchet setup checklist — multi-unit (2026-09-05)

Human actions that `/ratchet` cannot perform. Living document: items are ticked
with their evidence as they close. Supersedes
`docs/plans/2026-08-26-ratchet-setup.md`, whose still-open items are carried
below; that file is kept as the record of the single-unit period.

- Created 2026-09-05 at `guardrails_version: 0.5.1`, upstream commit
  `e2eac86ff8db2157735b9604b7ade190d4f465f3`, when the repository became a
  four-unit guardrails project (`docs/adr/2026-09-05-units-and-per-unit-classes.md`).
- Adoption order: `docs/plans/2026-09-05-ratchet-gap-analysis.md`.
- Every unit is class C.

## Signing is a gate, not a later tooth

Unchanged from 2026-09-01: `check-signing.sh --setup` exits 0 on the owner's
machine from the primary checkout (inside a worktree the deliberate
`commit.gpgsign=false` makes it report the key as missing — an artefact of
where it ran). Four signed merges on `master` since. `finish-merge.sh` runs
`check-signing.sh --strict` before removing a worktree, so the worktree
vanishing is the proof the signed merge landed.

- [x] **Commit signing proved end to end.** DONE 2026-09-01 (`--setup` → exit 0).
- [x] **Signature verification works.** DONE 2026-09-01, by the same proof;
      GPG, so no allowed_signers file is needed.

One caution that predates this document and still applies: never generalise a
measurement taken inside an agent sandbox to the owner's machine. Inside the
sandbox gpg cannot open `~/.gnupg/trustdb.gpg` and `~/.cargo` is read-only,
so a valid signature reads as unverifiable and cargo needs a scratch
`CARGO_HOME`. The reverse error is as real: this change's author first
concluded the sandbox could not build org-node at all, from one `--offline`
attempt against a scratch cargo home that lacked a git dependency — the
verification gate then built and ran it. A sandbox measurement is evidence
about the sandbox, in both directions.

## New with the multi-unit conversion

- [x] **org-node's verify_commands measured.** DONE 2026-09-05 at this
      change's gate, after a fix: the first draft omitted
      `--features app,test-support`, which the named targets'
      `required-features` demand, and cargo refused it (exit 101, no tests
      ran). Corrected line: 38 passed, 0 failed (lib 35, service_stories 1,
      transport_handshake 1, transport_networked 1), both fuzz targets to
      their 1s default. The three chopsticks-dependent targets stay excluded.
- [x] **app's `cargo test` measured.** DONE 2026-09-05: compiles, 0 tests — a
      compile proof until the crate has tests.
- [ ] **app's `npm run check` in every environment that merges.** Needs
      `app/node_modules` (gitignored): a fresh worktree lacks it and the
      command fails until `npm install` (or a copy from the primary checkout)
      — the merging machine's responsibility, per worktree-discipline "a fresh
      task worktree holds only tracked files".
- [ ] **app's verify_commands in CI.** **Amended 2026-09-14 — partially done.**
      app now has *three* verify_commands, and **two of them run in CI**: the
      `app-frontend` job added by the app risk analysis runs
      `npm --prefix app ci`, `npm --prefix app run check` and
      `npm --prefix app run test`. What remains is the cargo entry, which needs
      Tauri's Linux system packages (webkit2gtk, libsoup, …) `rust.yml` does not
      install — so for that entry alone the merge gate is still app's only gate,
      and 78 of the unit's 108 tests are behind it. Tracked as its own item in
      the 2026-09-14 section below; this box stays open until the cargo half
      lands.
- [ ] **Coverage for org-node and app.** Neither has a `coverage_command`. Under
      class C statement and decision coverage are mandatory, so these are gaps
      against a requirement, not decisions. Add a `coverage-org-node` Makefile
      target from a first measurement. **Amended 2026-09-14:** the clause "the
      app has nothing to measure until it has tests" is no longer true — app has
      108 tests as of the risk analysis. It is the same stale sentence T5
      corrected in `app/.guardrails/config.yaml`; this copy was missed then and
      is corrected now. The owner's decision stands: one basis for org-node and
      app together, at tooth 5.
- [ ] **Confirm the CI reshaping on the first push.** `rust.yml`'s `guardrails`
      job now runs `check-units.sh` once and the unit gates per unit over
      `check-units.sh --list`. The fix round also added org-node's verify
      command as a step in the `test` job, which is the first CI run of
      `transport_networked` on ubuntu-latest (hermetic via iroh's in-process
      relay, per org-node/Cargo.toml). Neither was exercised in this session
      (no push); the first push confirms both.
- [ ] **Decide `on-chain`.** Disclaimed with a date; tooth 7 makes it a unit
      (`forge test` as its `verify_commands`) or records why it stays outside.

## Carried over, still open

- [ ] **Branch protection on `master`.** No direct pushes, require signed
      commits.
- [ ] **CI signing to `--strict`.** Needs the committers' public keys published
      to the workflow. Runs non-strict on pushes to master today.
- [ ] **Decision coverage — mandatory under class C** for every unit. Not
      measured anywhere: `cargo llvm-cov --branch` is unstable on stable rustc.
      Needs `rustup component add llvm-tools-preview --toolchain nightly`, a
      `--branch` lane in the Makefile and in CI's `coverage` job, floors from
      the first measurement.
- [ ] **Human review policy.** Who signs off a merge; who is the independent
      reviewer at `merge-change` step 6a; whether critical items take two
      reviewers. Every change so far has used one fresh subagent, recorded by
      name, and each review found real defects.
- [ ] **Publish the restriction on use.** The hazard analysis
      (`org-members/docs/risk/2026-09-02-membership-hazards.md`) concludes
      overall residual risk against the intended use is UNACCEPTABLE. The
      restriction — no production deployment in the journalism, clinical or
      government context — has to be written where a user meets it (README and
      crate docs at minimum). Owner's decision.
- [ ] **Overall residual-risk criteria.** ISO 14971 clause 8 wants them in a
      risk management plan; this project has none. Write them into the risk
      ledger README — now `org-members/docs/risk/README.md`, and the same
      matrix was copied into the other three units' risk READMEs, so the
      criteria, once written, belong in all four or in one root document they
      cite.
- [ ] **Problem-report limits per unit.** `problem_age_days: 30`,
      `problem_open_max: 10` in every unit. org-members holds both open items
      (PR-zz4exm opened 2026-08-31, PR-hvg2dy opened 2026-09-02); PR-zz4exm
      hits the age limit on 2026-09-30 if still open. Re-check at tooth 8.
- [ ] **Dev-environment provisioning.** (a) `npm install` in
      `on-chain/scripts` for the nine on-chain-client integration targets and
      the three org-node chopsticks targets; (b) a writable `~/.quint`;
      (c) a writable `CARGO_HOME` holding the keyhive git checkout.
- [x] **Tool qualification (DO-330-lite).** DONE 2026-09-05. Qualification
      basis: the guardrails bats suite at `guardrails_version: 0.5.1`, upstream
      commit `e2eac86ff8db2157735b9604b7ade190d4f465f3`, recorded as
      `guardrails_commit` in all four unit configs. Result: **602 tests, 602
      ok, 0 not ok** (TAP plan `1..602` complete), run via
      `<guardrails>/tests/run-tests.sh` in the guardrails repository — not
      here, and with no bats installed anywhere. Run twice, not once: the
      first run's exit code was not captured — the zsh wrapper wrote it to
      `status`, which zsh keeps read-only, and failed after the suite had
      finished (its TAP output was complete and all ok). The second run, with
      the capture fixed (`rc=$?`), reported **602 ok, 0 not ok, exit 0**.
      Re-record version, commit and result whenever `/ratchet` updates the
      scripts. Do not modify the scripts in this project; change them
      upstream, where the tests live.
- [ ] **Scope note.** Guardrails supports a quality management system; it is
      not itself regulatory compliance. The quality manual, design controls
      and human sign-offs govern.

## Closed by this change

- The eight installed scripts were at upstream `bb7eee5`; `check-units.sh`
  did not exist here. All nine are now at `e2eac86`, and `.guardrails/templates/
  verification.md` was refreshed from the same commit (three lines: a
  `finding-2..4` range header is not a finding).
- `guardrails_commit` was never recorded in the 0.5.1 config. It is now set in
  every unit config, which is what makes the qualification basis checkable.

## Added 2026-09-06 — dependency edges (`docs/plans/2026-09-06-dependency-edges.md`)

- [ ] **Deliver REQ-ysyu9g in on-chain-client** before 2026-12-05: an
  exported requirement stating that `get_org_state` with no block named reads
  the latest finalised block, carrying `satisfies: REQ-ysyu9g`, with a test.
  Until then org-node's run reports `UNMET-EXPECTATION`; after that date it
  fails (`expectation_age_days: 90`). Owner: Jan-Jan.
- [ ] **Deliver REQ-q92yac in org-members** before 2026-12-05: an exported
  requirement that a change set removing a device key without replacing the
  member-as-a-group key is rejected, `satisfies: REQ-q92yac`, with a test —
  or REQ-ewdg2q amended in place with that `satisfies:` line and a wire-path
  test; either meets the expectation. Same clock. This is org-members' not-minted control 2 given a requester.
- [ ] **SOUP tables** are empty in all four units; every consumer inherits its
  providers' SOUP (assessments, all four edges). Tooth 4.

## Added 2026-09-09 — org-node risk analysis (`docs/plans/2026-09-09-org-node-risk-analysis.md`)

**Numbering.** Every reference to a not-minted control anywhere in this file
names its unit in front of the number — "org-node not-minted control 12",
"on-chain-client not-minted control 12" — with no exceptions. The entries below
belong to **org-node's register**
(`org-node/docs/risk/2026-09-09-org-node-hazards.md`), whose list starts at 1
and is numbered independently of on-chain-client's list in the section after
this one, so a number on its own says nothing about which control is meant.

That is a rule, not a list of the numbers that happen to be ambiguous, and
deliberately so: this note used to enumerate the colliding numbers, and the
enumeration went stale twice in consecutive fix rounds — each time a control was
added to either register, by an author with no reason to look here. Qualifying
every reference leaves nothing to keep current, so please do not reintroduce the
list. (Review round 1 of the on-chain-client change, finding-6, found the bare
references ambiguous; a later review round found the enumeration that had
replaced them stale, and fix round 8c replaced it with this rule.)

- [ ] **Fix PR-vt244s (publish before persist)** by a `resolve-problem` change:
  reproducing test first (chain write succeeds, send to an unreachable address
  fails, the record's epoch must equal the chain's), then persist the record
  on chain confirmation and make delivery retryable. Age limit 2026-10-09.
- [ ] **Fix PR-2dmjzj (loopback admission unbound to the joiner's Device
  key)** the same way: refuse a peer address whose identity is not the join
  request's Device key, or dial by the key as the networked branch does. Age
  limit 2026-10-09.
- [ ] **Fix PR-u4c2vp (the revocation receive path cross-checks no sender)**
  the same way: apply RC-b6mydy's membership cross-check on the revocation
  receive path, so the `UpdatedNotRevoked` branch cannot commit a record from
  a sender it did not check; reproducing test first (an update relayed to that
  path by a non-member device must be rejected and commit nothing). Age limit
  2026-10-09.
- [ ] **Resolve PR-hvg2dy** (now in org-node's ledger): correct the
  `chain_read.rs` doc-comment and the README's "current best". Age limit
  2026-10-02 — the oldest open item in any unit after PR-zz4exm.
- [ ] **Resolve PR-w88sr9** (org-node's ledger, opened 2026-09-10 by review
  round 7): the `WireMessage::genesis_snapshot` doc-comment says the field is
  `None` for non-admission messages while the revocation send path sets it.
  Correct the comment on the struct and on the field; the behaviour is right
  and must not change, and whether to omit the snapshot on the revocation path
  is org-node not-minted control 15's question, not this report's. Age limit
  2026-10-10.
- [ ] **Problem budget re-check.** org-node holds five open reports,
  org-members one (PR-zz4exm, limit 2026-09-30). Both under
  `problem_open_max: 10`; both age limits bite within a month.
- [ ] **Run the chopsticks lane in CI and add a reorganisation test**, so the
  finalised-publish control can be minted with gated evidence (org-node
  not-minted control 12). `chopsticks_reorg.rs` reorganises nothing today.
- [ ] **Deep-fuzz lane with a seed corpus** for the two org-node bolero
  targets (org-node not-minted control 13).
- [ ] **Run clippy on org-node and on the app** (org-node not-minted control 14, added
  in fix round 2). No `verify_commands` entry and no CI job lints either
  crate, so org-node's panic-freedom denial — the structural half of
  RC-e2uvje — is asserted and never checked; this change itself broke it and
  passed every gate. Add both feature sets to org-node's `verify_commands`,
  add org-node to the CI `clippy` job, and give `app/src-tauri` a `[lints]`
  section plus a clippy entry of its own.
- [ ] **Bound or omit the membership snapshot on the send path** (org-node
  not-minted control 15, added in fix round 5). Both Wire messages carry the
  whole pre-change record, so the 1 MiB frame is a bound on the Organisation,
  and it bites after the chain write — inside the publish-before-persist
  window, on admission and revocation alike. The revocation half is free: the
  receiver rebuilds from its own store and never reads the snapshot. Do it with
  PR-vt244s.
- [ ] **A recovery path for a lost store passphrase, and a documented
  re-enrolment procedure** (org-node not-minted control 16, added in fix round 5).
  A forgotten passphrase today loses every key on the device; the harm is the
  same as the crash that truncates the store, which is why this belongs with
  the atomic-write item (org-node not-minted control 10).
- [ ] **The restriction on use** (carried over) now has a second register
  behind it: org-node's overall residual risk is UNACCEPTABLE against the
  intended use, for the reasons its register gives — including all three of
  the hazards its own controls introduce, each re-assessed as unacceptable in
  its own right in the fix round that followed the review round which found
  the defect.

## Added 2026-09-10 — on-chain-client risk analysis (`docs/plans/2026-09-10-on-chain-client-risk-analysis.md`)

**Numbering.** The same rule as the section above, and for the same reason:
every reference to a not-minted control anywhere in this file names its unit in
front of the number, with no exceptions and no list here of which numbers are
ambiguous — a list of that kind went stale each time either register grew,
which is why a rule replaced it. The entries below belong to
**on-chain-client's register**
(`on-chain-client/docs/risk/…-on-chain-client-hazards.md`), a list of
**nineteen** that starts at 1 and shares nothing but its numbers with
org-node's list in the section above.

Eighteen of the nineteen have a checklist item below. **on-chain-client
not-minted control 17** — couple the width test to the decoder's regions by
exporting them from `src` — has none, and that is deliberate rather than an
omission: it is the one entry on the list that explicitly reduces **no**
residual risk, because `decode_org_state.rs` already reds on exactly the drift
it would catch. It is a better-engineering proposal, so it is recorded in the
register and not queued here. (It was added in fix round 5, after this note was
first written saying "sixteen"; the count is corrected rather than the entry
dropped. **on-chain-client not-minted controls 18 and 19** were added in fix
round 8a, from review round 3's finding-1, and both have items below — each is
a behaviour a minted control used to assert and now no minted control does,
which is exactly the state in which a queue entry is what stops it being
forgotten.)

- [ ] **Fix PR-w5sk5k (the decoder is pinned at construction and never
  re-checked)** by a `resolve-problem` change: compare the resolved block's
  `spec_version()` against the pinned one on each read and each subscription
  and refuse rather than decode, so a client that outlives a runtime upgrade
  fails loudly instead of decoding post-upgrade bytes with the old decoder.
  The information is already in hand — `get_org_state` resolves a block on
  every call. Age limit 2026-10-10.
- [ ] **Resolve PR-h4mb8y (`on-chain-client/src/verify.rs` is seven lines of
  doc-comment and no code)** either way: implement the module — which means
  taking `org-members` as a dependency and declaring the edge after the
  assessment `docs/plans/2026-09-06-dependency-edges.md` prescribes — or
  delete it and move its promise into the design. What is not acceptable is a
  module whose header describes in the present tense a verification it does
  not perform. Age limit 2026-10-10.
- [ ] **Fix PR-uq5r97 (the best lane's first backfill span is unbounded)**:
  cap the span and resync by an explicit state read rather than by replay when
  the cap is exceeded; the seed's hash must stay the finalised hash for the
  dedup and reorg comparisons to behave. Age limit 2026-10-10.
- [ ] **Fix PR-qpp28h (a reorganisation deeper than the notification gap is
  undetectable)**: detect a discarded *ancestor* rather than a discarded head —
  walk back from the new head's parent to a hash already processed and report
  every processed head off that path — so the detectable depth is a property of
  what the lane remembers rather than of how often it was notified. Age limit
  2026-10-10.
- [ ] **Problem budget re-check.** on-chain-client now holds four open reports
  (all limit 2026-10-10) against `problem_open_max: 10`; org-node five,
  org-members one. Three ledgers, ten open items, and every on-chain-client
  limit falls on the same day.
- [ ] **Pin the H160 mapping against the runtime's own answer at a gate**
  (on-chain-client not-minted control 1). `on-chain-client/tests/p_address_is_orgid.rs`
  already does it and runs in no lane, so RC-mugta4 pins our reading of
  pallet-revive rather than pallet-revive itself. The single most valuable
  ungated test in the crate.
- [ ] **Seed `fuzz_event_round_trip`'s corpus** (on-chain-client not-minted control 2). It is
  the one of the three bolero targets with no seeds, visible in measured output
  since the 2026-08-26 verification record and never written down as a gap
  until now. Its seeding job differs from the other two's: it consumes
  `TypeGenerator` driver bytes, not raw payloads, so `tests/regenerate_corpus.rs`
  does not cover it.
- [ ] **Deliver REQ-ysyu9g in on-chain-client** before 2026-12-05
  (on-chain-client not-minted control 3) — carried over from the entry above
  and now with the fact established: subxt 0.50.1 documents `at_current_block`
  as the current finalised block
  (`subxt-0.50.1/src/client/online_client.rs:269`) and `get_org_state(_, None)`
  calls it, so the behaviour is right and what is
  missing is the exported requirement and evidence a gate can run. Note the
  coupling the interview found: exporting it turns org-node's exempt
  expectation into an ordinary requirement needing a `verifies:` test on
  org-node's side at the same merge, and chopsticks finalises every block it
  mines immediately, so no chopsticks test can be that evidence.
- [ ] **Decide whether `for_runtime` widens to a known-good range**
  (on-chain-client not-minted control 5). Added in fix round 2, from review
  round 1's finding-6, which found this control had no owner, no date and no
  item here. Exactly one `spec_version` is compiled in, so the day Paseo Asset
  Hub upgrades every `from_client` fails and the crate reads nothing at all
  until a decoder module and a match arm ship — the total outage the register
  assesses at S3/P2, not acceptable, as a hazard RC-d7r82e introduces. The code
  contemplates the widening already
  (`on-chain-client/src/decode/dispatch.rs:14-18`); what it needs first is
  several upstream versions round-tripped, which needs on-chain-client
  not-minted control 10. **The decision to take is whether widening is
  acceptable at all** — a range trades
  the outage for the risk of decoding a version nobody round-tripped — so this
  is a decision to record either way, not a task to schedule. Owner: Jan-Jan.
  Decide by 2026-12-10.
- [ ] **Bring the contract's guards inside a gate** (on-chain-client not-minted control 9).
  Two controls of on-chain-client's register are implemented in
  `on-chain/src/OrgRegistry.sol` — the `ZeroValue()` revert and the atomic
  three-field write that make "an absent slot means no such Organisation"
  sound, plus the storage layout the slot derivation hard-codes. Its own tests
  cover the first (`on-chain/test/OrgRegistry.t.sol:67`, `:73`) and run in no
  lane here: the workflow directory holds a Rust workflow and a Quint workflow
  and mentions `forge` in neither. Either run `forge test` in CI or
  re-establish both properties from this side against a deployed contract.
- [ ] **Run on-chain-client's chain-dependent lane in a gate**
  (on-chain-client not-minted control 10). Nine targets need a chopsticks or
  anvil fork and one needs live Paseo; none of the ten runs anywhere, and they
  are where every chain-dependent property of the crate is tested. It buys
  reorg and delivery
  evidence but **not** finality evidence, which needs the live-Paseo path.
  Related: the same item for org-node's chopsticks lane, above.
- [ ] **Gate the best/finalised distinction, or record that it stays ungated**
  (on-chain-client not-minted control 18, added in fix round 8a from review
  round 3's finding-1). An observation from a best block is delivered as
  `SubscribedEvent::BestBlockEvent` and one from a finalised block as
  `FinalisedEvent`, so a consumer can act optimistically on the first and
  commit only on the second. The behaviour is real — the two lanes differ in
  exactly the `wrap` closure each passes into `decode_contract_events`
  (`on-chain-client/src/client.rs:616-618` against `:459-465`) — and it was
  RC-kemv75's opening clause until finding-1 established that none of that
  control's three requirements states it, so **no minted control asserts it
  now**. No gated test reaches it either: both lanes are `async`, and all a
  gated test can see is that three variants are declared. on-chain-client
  not-minted control 10 is necessary and **not sufficient** —
  `reorg_cancels_proposed.rs` would have to gain the optimistic half and
  `smoldot_smoke.rs` the committed half, because chopsticks finalises every
  `dev_newBlock` immediately; neither asserts the distinction today. So the
  decision is: write those two assertions inside on-chain-client not-minted
  control 10's lane, or record that the distinction stays evidenced by nothing
  and why that is accepted. Owner: Jan-Jan. Decide by 2026-12-10.
- [ ] **Gate the construction-time refusal of an unrecognised runtime, or
  record that it stays ungated** (on-chain-client not-minted control 19, added
  in fix round 8a from the same finding). A `spec_version` for which no decoder
  is compiled in is refused **before any read is possible**
  (`on-chain-client/src/client.rs:129-147`, the refusal at `:138-139`), so no
  client exists that could read through a decoder it failed to resolve. This
  was RC-d7r82e's last clause, and like on-chain-client not-minted control 18
  it is now asserted by no minted control. `runtime_version_dispatch.rs` gates
  `dispatch::for_runtime`, the
  decision `from_client` delegates to, and can say nothing about *when* that
  decision is taken or what becomes of the client if it refuses; `from_client`
  is `async` and needs a live chain. The test that would settle it does not
  exist: a chopsticks target — `00_chopsticks_sanity.rs` is the smallest —
  constructed against a fork whose runtime version is overridden to one no
  decoder is compiled for, asserting `UnsupportedRuntime` rather than a client.
  Couple the decision to on-chain-client not-minted control 5's: widening
  `for_runtime` to a known-good range changes what this test asserts.
  Owner: Jan-Jan. Decide by 2026-12-10.
- [ ] **A staleness signal on a state read** (on-chain-client not-minted control
  11). Added in fix round 2, from review round 1's finding-6. **This is the sole
  named control for a prose hazard the register assesses S3/P2 and not
  acceptable** — nothing re-reads on a schedule and nothing reports age, so a
  consumer holds whatever it last read for as long as it holds it, and
  `OrgState` carries a root, a key and an epoch and **no block reference**
  (`on-chain-client/src/state.rs:18-25`), so a value cannot afterwards be asked
  which block it came from. ISO 14971 requires an option for an unacceptable
  risk to be carried to a decision, which is what this item is: return the block
  a state was read at alongside the state, optionally with a re-read on a
  schedule, or record the decision not to and why. Owner: Jan-Jan. Decide by
  2026-12-10.
- [ ] **Establish that the configured address holds the expected contract**
  (on-chain-client not-minted control 14). Added in fix round 2, from review
  round 1's finding-6. **Also the sole named control for a prose hazard assessed
  S3/P1 and not acceptable** — RC-5e3bdk checks that the emitting address equals
  the configured one and nothing else: no code hash, no interface probe, nothing
  that establishes that the contract living at that address is the `OrgRegistry`
  this decoder was written against, and a consumer handed a wrong or stale
  address gets a reader that filters diligently for the wrong thing. It is also
  the larger half of the answer to the silently-deaf reader that RC-5e3bdk
  introduces (S3/P1, not acceptable). Options to decide between: accept a code
  hash alongside the address and check it at construction, or read something at
  construction only the intended contract could answer. Owner: Jan-Jan. Decide
  by 2026-12-10.
- [ ] **A two-contract chopsticks fixture** (on-chain-client not-minted control 13). Every
  existing chopsticks target deploys one `OrgRegistry`, which is why the real
  `decode_contract_events` path has never been exercised against an impostor —
  the measured false green this change found and removed by extraction. Deploy
  a second contract emitting a well-formed registry log with the victim's
  indexed admin, and assert the subscriber receives nothing.
- [ ] **Report a signature-matching log from an unconfigured contract**
  (on-chain-client not-minted control 12). One observation is simultaneously the evidence of a
  spoofing attempt (HAZ-werm85) and of a reader pointed at a stale address (the
  hazard RC-5e3bdk introduces), and today it is `continue`.
- [ ] **Extend on-chain-client's panic-freedom gate past `--lib`, and decide
  about `rustfmt`** (on-chain-client not-minted control 15). Clippy lints `--lib` only, so the
  **nine** new integration targets are outside the denial the crate root
  declares — eight added with the risk analysis and a ninth, `type_widths`,
  added by review round 1's finding 1, with
  `on-chain-client/Cargo.toml`'s `[[test]]` declarations the count's ground
  truth;
  and the crate is not `rustfmt`-clean at baseline with no `cargo fmt` step in
  the workflow. Both are decisions to take deliberately, not defects to fix in
  a hazard analysis.
- [ ] **A deep-fuzz lane for on-chain-client's three bolero targets**
  (on-chain-client not-minted control 16). One second each at the generative engine is what the
  gate runs, and a `harness = false` run reports iterations and an exit reason,
  never a pass count. Do it with on-chain-client not-minted control 2's corpus.
  Related: the same item for org-node's two targets, above.
- [ ] **The restriction on use** now has a third register behind it:
  on-chain-client's overall residual risk is UNACCEPTABLE against the intended
  use, for the reasons its register gives — four open defects, two controls
  whose soundness rests on a disclaimed file tested in no lane here, the
  mapping pinned against our reading of pallet-revive rather than against
  pallet-revive, an unmet consumer expectation, and all four entries its own
  controls introduce unacceptable in their own right.

## Added 2026-09-14 — app risk analysis (`docs/plans/2026-09-14-app-risk-analysis.md`)

The entries below belong to **app's register**
(`app/docs/risk/…-app-hazards.md`), whose not-minted-control list starts at 1
and is numbered independently of the other three. The naming rule stated in the
2026-09-09 section applies here too and without exception: every reference
below says "app not-minted control N".

This completes tooth 3. All four units now have a per-unit ISO 14971 register,
and all four conclude **UNACCEPTABLE** overall residual risk against the
intended use.

**Open problem reports — age limit 2026-10-14** (`problem_age_days: 30`,
opened 2026-09-14). app now holds five open reports — PR-5mc4d8 opened by the verification gate
and PR-j9f6kk by the second independent review, both after the register was
written — taking the repository's total to fifteen across four ledgers.

- [ ] **Resolve PR-w5dae4 (startup failure arrives as a panic in Tauri's
  `setup` hook).** This one is not ordinary backlog: RC-xt4qr3 and RC-djzms3
  deliberately convert two silent degradations into startup failures, and a
  refusal whose reason is invisible trades a silent wrong state for a silent
  dead app. It is app not-minted control 3, it carries the same date, and until
  it lands those two controls are only half delivered. The register says so in
  §5 rather than in a footnote.
- [ ] **Resolve PR-h6xpnh (`receiverStarted` is component-local, so the badge
  reports the component's history rather than the receiver's).** Needs a
  `receiver_status` query command — a new command and a new requirement, not a
  control on an existing one. app not-minted control 4.
- [ ] **Resolve PR-j9f6kk (three frontend behaviours gated under requirements
  that do not describe them; two glossary terms declared in the root glossary
  and defined nowhere).** Opened by the second independent review and recorded
  rather than fixed on the owner's decision of 2026-09-14. The asymmetry is the
  point: this change minted REQ-wu6z9p as the frontend twin of REQ-kn5rtx,
  arguing a consumer requirement is "separate because it fails separately", and
  then did not do the same for `receiver-stopped` or `record-unreadable`. Also
  defines Persona and Epoch in the root glossary — REQ-tw4cb5 and HAZ-8ghmhn
  turn on them and both glossaries currently point at each other. Age limit
  2026-10-14.
- [ ] **Resolve PR-5mc4d8 (five boundary parsers strip a leading `0x`
  repeatedly, so a doubled prefix is normalised instead of refused).** Opened by
  the verification gate's round 4 and widened by the task that fixed the two
  sites a requirement covered. The remaining five have no requirement, which is
  why they are recorded rather than fixed: a fix would carry a test that could
  not be annotated. The sharpest is `ODS_CONTRACT_H160`, where the raw
  unstripped string is what reaches the status display, so the app would talk to
  `0x…` while showing the operator `0x0x…`. Over-acceptance only — no identifier
  can be parsed as a different one — so it is a robustness defect, not a route to
  harm. Closing it wants **one** requirement covering prefix **and whitespace** handling
  at every operator-supplied identifier in the unit, not six — the report was
  widened once the two parsers turned out to disagree about whitespace too. Age
  limit 2026-10-14.
- [ ] **Resolve PR-eecx3y (the chain verdict is a startup fact reported in the
  present tense).** Blocked on REQ-x3c8n2 below: the connection lives inside
  `SubxtChainOps`, which is org-node's. app not-minted control 5.

**Expectation — REQ-x3c8n2, `expects: org-node`, due 2026-12-13**
(`expectation_age_days: 90`, opened 2026-09-14).

- [ ] **Deliver REQ-x3c8n2 in org-node** before 2026-12-13: an exported
  requirement carrying `satisfies: REQ-x3c8n2`, letting a consumer distinguish
  a permanent failure of the transport endpoint from a transient one *from the
  error's type*. Today app's receiver decides by matching three
  substrings of messages org-node formats, which no gate in either unit
  protects. Reword any of them and app loops forever against a dead endpoint
  while its badge reads "Receiver running" — and that is not hypothetical: until
  2026-09-15 the app matched a single marker, `"endpoint not bound"`, that
  org-node and iroh never produce, so the loop had never stopped at all (the
  register's §3, claim 7).

  Two of app's six hazards do not improve in probability at all because of
  this, and both say so in their residual: HAZ-cfp4jb (detection is unsound)
  and HAZ-ny7yvt (liveness is unobservable, the same gap seen from the chain
  side, PR-eecx3y). If a single health-and-error surface on org-node answers
  both faces, so much the better — the expectation is worded around the
  receive path only because that is the one with a live workaround in the code.

  This is the repository's **third** open expectation, alongside REQ-ysyu9g
  (org-node → on-chain-client, 2026-12-05) and REQ-q92yac (org-node →
  org-members, 2026-12-05). `expectation_open_max` is 10 in every unit, so the
  budget is not the constraint; the dates are.

**app's not-minted controls — decide by 2026-12-14**, except where noted.
Fifteen in total; the ones below are the ones that need a decision rather than
a tooth.

- [ ] **Render components in tests** (app not-minted control 1). The largest
  hole in the change and the one a reader should be told about first: *no test
  in this repository renders a Svelte component.* app's frontend suite tests
  extracted functions, so nothing asserts that `Revoke.svelte` calls
  `validateRevokeInput`, that `Membership.svelte` calls `verifyResultFrom`, or
  that the newly-reachable `✗ MISMATCH` branch is wired to anything. A defect
  leaving the tested function correct and the component calling something else
  passes every gate. Needs `@testing-library/svelte` or
  `vitest-browser-svelte`, a DOM environment, and a decision about how much of
  SvelteKit to stand up. The residuals of HAZ-n97v5g and HAZ-5ha5vv both rest
  on this staying open.
- [ ] **Prompt for the store passphrase at first run** (app not-minted control
  2), superseding RC-xt4qr3's refusal with the control ISO 14971 prefers: make
  the safe state reachable rather than the unsafe one fatal. The refusal is
  what could be implemented and gated now; it is not the end state.
- [ ] **Surface startup failures in a window** (app not-minted control 3).
  Same item as PR-w5dae4 above, same date, **2026-10-14** — listed in both
  places on purpose, because it is simultaneously a defect and the completion
  of two controls.
- [ ] **Display full keys, or a human-comparable fingerprint, in the Admit
  panel** (app not-minted control 6). `Admit.svelte` shows the joiner's member
  and device keys as their last sixteen hex characters — 64 bits, which nobody
  would choose deliberately for a comparison an attacker can grind offline. An
  administrator verifying a join request over a second channel can compare only
  what is displayed. Recorded in the register's §6 as prose.
- [ ] **Move the admin signing key out of the process environment** (app
  not-minted control 8). `ODS_ADMIN_SEED` carries the 32-byte admin secret as
  hex; environment variables are inherited by child processes and captured by
  crash reporters and process supervisors. Spans app and org-node, which is
  why it is a decision rather than a fix.
- [ ] **A second, independent path to revocation** (app not-minted control
  15) — a command-line tool or a recovery mode — so a defect in the single user
  interface cannot remove the safety action outright. HAZ-n97v5g is the
  existence proof that it can: revocation was unreachable in the default
  transport, and there was no other way to perform it.

**Coverage, still.** app has no `coverage_command` and this change deliberately
did not add one (owner's decision, 2026-09-14: keep one measurement basis for
org-node and app together). Under class C that is a gap against a mandatory
requirement and it stays recorded here. Tooth 5 now owns: a floor for org-node,
a floor for app, and decision coverage — unmeasured in all four units.

**CI.** app's three `verify_commands` used to run nowhere in CI. After this
change two of the three do: a new `app-frontend` job runs `npm --prefix app run
check` and `npm --prefix app run test`. The cargo entry still runs nowhere,
because it needs Tauri's Linux system packages (webkit2gtk, libsoup, …) the
workflow does not install — so for that entry the merge gate remains the only
gate.

- [ ] **Install Tauri's Linux system packages in CI** so app's cargo entry runs
  there too. The 108 tests this change adds are the first this unit has ever
  had, and 78 of them are behind that gap.

**The restriction on use** now has a fourth register behind it. app's overall
residual risk is UNACCEPTABLE against the intended use: six hazards, every one
S3, two of which do not improve in probability at all; five open defects; an
unmet expectation on org-node; no component rendered by any test; and no
coverage figure of any kind.

**Formatting, now a two-unit question.** `cargo fmt --check` is dirty across
`app/src-tauri` — `commands.rs`, `state.rs`, `lib.rs`, `events.rs` and four test
files — and was dirty before the 2026-09-14 change touched any of them. There is
no `rustfmt.toml`, no fmt entry in any unit's `verify_commands`, and no fmt step
in CI, so nothing converts this into a failure anywhere.

- [ ] **Decide about `rustfmt`, repository-wide.** on-chain-client's register
  raised this for its own crate (on-chain-client not-minted control 15, which
  pairs it with extending clippy past `--lib`); app's risk analysis found the
  same in a second unit. Two units is enough to make it a repository decision
  rather than a per-crate one: either adopt a `rustfmt.toml` and add a `cargo
  fmt --check` step, accepting a one-off reformat across four crates, or record
  that formatting is deliberately unenforced. What is not worth keeping is the
  present state, where every task that touches a Rust file has to decide for
  itself whether to reformat neighbouring lines it does not own — which is a
  live source of merge noise between parallel task worktrees.

**npm audit, now that this repository ships a JavaScript dependency tree.**
`npm ci` in `app` reports 4 vulnerabilities (1 low, 1 moderate, 2 high) as of
2026-09-14. None was introduced by the app risk-analysis change — the tree
predates it — but that change is the first to make the frontend a gated,
CI-installed artefact, which is what makes the absence of any audit gate
visible.

- [ ] **Decide what `npm audit` means for a class C unit.** No
  `verify_command` in any unit runs it and no CI job will surface it, so today
  those four advisories are invisible to every gate. The decision is not
  automatically "add `npm audit` to `verify_commands`" — an advisory feed that
  can redden a build overnight, on a transitive dependency of a build tool, is
  a different kind of gate from the rest of this toolkit, and an unpinned one
  makes the merge gate non-reproducible. The realistic options are a periodic
  reviewed audit recorded in the SOUP, or an `audit-level` threshold with a
  documented allowlist. Related: app's SOUP table is still the empty template
  (app not-minted control 13, tooth 4), and these four advisories are precisely
  the kind of thing it exists to record.
