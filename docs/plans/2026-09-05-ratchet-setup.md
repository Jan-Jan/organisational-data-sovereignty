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
      `on-chain/scripts` for the seven fork-dependent on-chain-client
      integration targets and the three org-node chopsticks targets
      (**corrected 2026-09-28 by this change's review sweep:** this said
      "nine". Nine on-chain-client targets run at no gate, but `smoldot_smoke`
      needs live Paseo and `regenerate_corpus` needs no chain, so neither needs
      these `node_modules`); (b) a writable `~/.quint`;
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
  (on-chain-client not-minted control 10). Nine targets run at no gate: seven
  need a chopsticks fork, `smoldot_smoke` needs live Paseo, and
  `regenerate_corpus` needs no chain at all. The first eight are where every
  chain-dependent property of the crate is tested. Gating them buys reorg and
  delivery evidence but **not** finality evidence, which needs the live-Paseo
  path. (**Corrected 2026-09-28 by this change's review sweep:** this read
  "nine … and one needs live Paseo; none of the ten", counting ten ungated
  targets where there are nine — `smoldot_smoke` **is** one of the nine — and
  a broken reflow left "It buys / reorg and delivery / evidence" across three
  lines.)
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

## Added 2026-09-17 — the skipped finalisation, and the CI that has never run (`docs/plans/2026-09-10-on-chain-client-risk-analysis.md`)

Four open items, all raised by one defect: `fbf17f0` (2026-09-11, tooth 3's
on-chain-client risk analysis) never ran `merge-change` step 3
(`finalize-docs.sh`), so three ledger files were squashed onto `master` still
carrying their `DRAFT-` names. `on-chain-client`'s `check-ids.sh` has been exit 1
on `master` ever since, and stays that way until the repair
(`worktree-guardrails-on-chain-client-finalize`) merges — the rename itself is
done on that branch, so none of the items below is the repair.

They divide in two. Items (a) and (b) are **structural**: they are about why
nothing convicted, and they are the reason this section exists. The two after
them are **documentary** — collateral the investigation uncovered rather than
causes of it, and each is someone's outstanding work rather than a decision to
take.

No date is given here for when the six-day span ended, deliberately. This
section was first written saying "until the repair on 2026-09-17" and the repair
did not merge that day; a span written against a merge that has not happened yet
is a guess, and guessing a date ahead of a merge is the very mistake this
section is about.

**(a) A skipped step 3 is invisible to the local merge gate.** This is a
structural property of the sequence, not an oversight by the operator.
`verify-before-merge` runs `check-ids.sh` **with** `--allow-draft-files`, which
is correct there: a change's own `DRAFT-<branch>-<slug>.md` files are legitimate
for the whole life of that change. The only local run **without** the flag is
`merge-change` step 4 — and step 4 is skipped by exactly the same operator
omission that skips step 3, because they are adjacent steps of one procedure.
So the local sequence cannot self-detect a skipped finalisation: the gate that
would convict is downstream of the omission and inside its blast radius. CI
*can* detect it — `.github/workflows/rust.yml`'s `guardrails` job runs
`check-ids.sh` bare per unit on every event that is not a pull request — but
only on a push, which brings us to (b).

- [ ] **Decide how a skipped `finalize-docs.sh` gets caught locally.** The
  options are not equivalent and none is obviously right: make step 4's bare run
  independent of step 3 having been reached (a post-merge hook on the base
  branch, which catches it after the fact rather than before); have
  `verify-before-merge` additionally assert that no `DRAFT-*` ledger file is
  *tracked on the base branch*, which is a different question from the one
  `--allow-draft-files` forgives and would have caught this one merge later;
  or accept that CI is the backstop and fix (b) instead. This is the owner's
  call. Not proposed as decided.

**(b) The principal CI workflow has never run at all, and what CI history does
exist stops at 2026-06-17.** "Dormant since 2026-06-17" was this section's first
wording and it was too kind: it reads as gates that used to run and lapsed, and
for most of them there was never anything to lapse from. Measured 2026-09-17:
`origin/master` is at `2bb1c21`, dated 2026-06-17, and local `master` is **45
commits ahead** of it. Nothing has been pushed in three months. Every workflow
in `.github/workflows/` is triggered by push or pull request, so none of them
has executed on any of those 45 commits — which includes all four units'
ratcheting, **all four** of the risk analyses (`630fa0d` org-members,
`50a5254` org-node, `fbf17f0` on-chain-client, `5c0c710` app; each verified
present in `origin/master..master` on 2026-09-17), and the org-members
architecture ledger. But the 45-commit gap is only half the story, and the
smaller half.

**`.github/workflows/rust.yml` has no execution history whatsoever.** The file
was added by `4bb5509` (2026-08-27, "ci(guardrails): Rust + guardrails CI and
the class B coverage gate"), and `4bb5509` is itself one of the 45 unpushed
commits: `git merge-base --is-ancestor 4bb5509 origin/master` exits **1**, and
`git cat-file -e origin/master:.github/workflows/rust.yml` fails with "exists on
disk, but not in `origin/master`". The workflow has therefore never been present
on the remote and has never been dispatched, not once. So the bare
`check-ids.sh`, the clippy denial, the `no_std`/wasm32 compile checks and the
cross-platform coverage re-run are not lapsed gates with a green history behind
them — they have produced no result, ever. Every sentence anywhere in this
repository that reads "CI runs X" about `rust.yml` describes an intention, not
an event.

**`.github/workflows/quint.yml` is the mixed case, and the split is per
invariant.** That workflow *is* present at `2bb1c21`, so its steps as they stood
there did run on pushes up to 2026-06-17. `620b459` ("feat(quint): Milestone 3 —
tau-window, compromised key, convergence", 2026-06-18) is the **only** commit
touching it since (`git log 2bb1c21..master -- .github/workflows/quint.yml`
lists it alone), and it too is unpushed: `git merge-base --is-ancestor 620b459
origin/master` exits **1**. Comparing `git show
2bb1c21:.github/workflows/quint.yml` against the working tree gives the split
exactly:

- **Ran, until 2026-06-17.** The `quint` job's simulator runs of `forkSafety`,
  `revocationSafety` and `revokedExcludedFromOrgSecret` over `protocol.qnt` at
  5000 samples / 16 steps; the `membership_mbt.qnt` `mbtInv` run at 200 samples
  / 15 steps; the four typechecks and `quint test quint/membership.qnt`; the
  `mbt` job; and the `apalache` job's bounded `quint verify` of those same three
  invariants at depth 5.
- **Never ran, not once.** The `tauWindow` and `convergence` simulator runs at
  5000 samples / 16 steps, and the `apalache` job's `quint verify` of
  `tauWindow` at depth 5 and of `convergence` at depth 3 — all four added by
  `620b459`. Two of the five randomised invariant runs and two of the five
  bounded verifies have never executed.

The gates that exist **only** in CI, read with that distinction — "dormant"
fits only the three older quint invariants; everything in `rust.yml` has never
run at all:

- the bare `check-ids.sh` per unit (`rust.yml`, `guardrails` job, the
  `github.event_name != 'pull_request'` branch) — the gate that would have
  convicted `fbf17f0` on the first push to `master`. A local bare run **does**
  exist: `merge-change` step 4, as item (a) above says. What CI supplies that
  step 4 cannot is a bare run **independent of the operator reaching step 4** —
  and step 4 is inside the blast radius of the very omission it would convict.
  So CI's is the only bare run that does not depend on the omitted step;
- `cargo clippy … --lib -- -D warnings` for org-members and on-chain-client
  (`rust.yml`, `clippy` job) — the panic-freedom denial. Precisely: it is not
  that clippy has *no local counterpart*, but that it is **prescribed for
  developers and gated nowhere**. `org-members/AGENTS.md` line 117 gives
  `cargo build && cargo test && cargo clippy` as the crate's default command.
  Nothing enforces it: no Makefile target names clippy, no unit's
  `verify_commands` names it, and no step of `verify-before-merge` or
  `merge-change` names it. A prescription nobody runs leaves less trace than a
  CI job nobody ran, because there is not even a build to be red;
- the `no_std` and `wasm32-unknown-unknown` compile checks for org-members
  (`rust.yml`, `no-std` job) — org-members' documented build matrix, and the
  only thing proving its dependency graph stayed no_std-compatible. Same
  correction as clippy: `org-members/AGENTS.md` lines 24-25 make
  `cargo check --no-default-features --features serde --target
  wasm32-unknown-unknown` mandatory "after any dependency change", and lines
  120-122 list all three configurations under "Build / test commands". Again no
  gate runs any of them, so the accurate charge is prescribed-but-ungated, not
  absent locally;
- **part** of `quint.yml`, not the whole of it. CI-only: `quint typecheck
  quint/membership_mbt.qnt`, `quint typecheck quint/ods_instances.qnt`,
  `quint test quint/membership.qnt` and `quint run quint/membership_mbt.qnt
  --invariant=mbtInv` in the `quint` job; that job's five randomised invariant
  runs over `protocol.qnt` (`forkSafety`, `revocationSafety`,
  `revokedExcludedFromOrgSecret`, `tauWindow`, `convergence`, each at 5000
  samples); and the `apalache` job's five bounded `quint verify` of the same
  invariants. **Covered locally, and therefore not on this list:** `quint
  typecheck quint/membership.qnt` and `quint typecheck quint/protocol.qnt`, both
  verbatim `verify_commands` entries of org-members (and `protocol.qnt` of
  org-node as well), and the `mbt` job's `cd org-members && cargo test --test
  mbt_conformance`, which `cargo test -p org-members` already runs —
  `rust.yml`'s own comment calls that job "now redundant". **And within the
  CI-only part, the history splits**: `forkSafety`, `revocationSafety` and
  `revokedExcludedFromOrgSecret` ran until 2026-06-17 and stopped; `tauWindow`
  and `convergence` — one simulator run and one `quint verify` each — were added
  by the unpushed `620b459` and have never run. The accurate claim is that every
  *model-checking* result this repository cites comes from a lane that has
  either not executed since 2026-06-17 (those three) or never executed at all
  (`tauWindow`, `convergence`); the typechecks run at every merge.

Two gates that look CI-only and are **not**, recorded here because putting them
on the list above understates the local sequence:

- **`check-signing.sh` is not dormant.** `.guardrails/scripts/finish-merge.sh`
  line 130 runs `sh check-signing.sh --strict` unconditionally as guard 1,
  before any branch or worktree is removed, and `AGENTS.md` (non-negotiable 2)
  says the squash commit "is verified with `check-signing.sh` before the
  worktree is cleaned up". The local run is `--strict`; `rust.yml` runs it
  **non-strict**, and its own comment gives the reason — a runner holds neither
  the signer's public key nor an `allowed_signers` file. So the local gate is
  the **stricter** of the two, and what CI would add is an independent backstop
  against a local sequence that was bypassed, not the only enforcement of
  non-negotiable 2. Tightening CI to `--strict` is its own open item below.
- **`make coverage` is not dormant either.** `Makefile` line 139 is
  `coverage: coverage-org-members coverage-on-chain-client`, and those two
  targets are exactly the `coverage_command:` entries in
  `org-members/.guardrails/config.yaml` and
  `on-chain-client/.guardrails/config.yaml`, consumed locally by
  `verify-before-merge` check 5 — on-chain-client's most recent local
  measurement is 41.75% lines / 42.45% regions against floors of 41 and 42
  (`docs/verification/2026-09-11-worktree-guardrails-on-chain-client-risk.md`).
  What is dormant is only the **cross-platform re-run**: the floors are
  calibrated on aarch64-darwin and enforced in CI on x86_64-linux against a
  floating `stable` toolchain and an untracked root `Cargo.lock`, a difference
  `rust.yml`'s `coverage` job comment flags itself. With a 0.45-point margin on
  the region floor, that re-run is the check that has never happened.

`check-review.sh` is a third gate not on the dormant list, for a different
reason again: it fires only on a pull
request and this project merges locally, but guardrails 0.5.1 made it runnable
on macOS and `merge-change` step 6c now runs it on the merging machine. It has
no independent CI backstop, which is a different gap and already recorded.

- [ ] **Decide whether this repository pushes.** The workflows were written and
  ratcheted on the assumption that CI is a real backstop; on the evidence it is
  not one, and — for `rust.yml`, the workflow carrying every unit gate — never
  was one for a single commit. The realistic options are: push `master`, which
  is **not** "let three months of accumulated CI catch up" but a **first
  bring-up** of `rust.yml` on a 45-commit backlog, because no human has ever
  seen its `guardrails`, `clippy`, `no-std`, `test` or `coverage` jobs succeed
  or fail on any commit — red on first contact is close to certain rather than a
  risk to accept, the failures will be the ordinary failures of a workflow's
  first run (toolchain, missing system packages, path assumptions) mixed
  indistinguishably with real findings, and triaging that is its own change and
  probably its own tooth; keep working locally and **delete or disable** the
  workflows so that no record cites a gate that does not run; or adopt pull
  requests, which would additionally bring `check-review.sh` back as an
  independent backstop instead of a local-only step. What is not an option is
  leaving verification records implying these gates ran. This is the owner's
  call, recorded here rather than resolved.

  **The "delete or disable the workflows" option moots three items still open
  in this file**, and a reader choosing it should see what it takes with it:

  - "**Confirm the CI reshaping on the first push**" (under "New with the
    multi-unit conversion", the sixth of that section's seven items — *not* in
    the 2026-09-14 section, as this passage first said; there is only one copy
    of it) — it says `rust.yml`'s `guardrails` job now runs `check-units.sh`
    once and the unit gates per unit over `check-units.sh --list`, that the fix
    round added org-node's verify command to the `test` job (the first CI run of
    `transport_networked` on ubuntu-latest), and that "the first push confirms
    both". With no push there is never a first push, and that item can never be
    closed as written — it would have to be withdrawn, not completed.
  - "**Branch protection on `master`**" (under "Carried over, still open") — "No
    direct pushes, require signed commits." Branch protection is a
    forge-side control on a remote that is not being pushed to; deleting the
    workflows does not by itself moot it, but it removes the required-status-
    check half of it, leaving only the signature requirement, which
    `finish-merge.sh` guard 1 already enforces locally and more strictly.
  - "**CI signing to `--strict`**" (under "Carried over, still open") — "Needs
    the committers' public keys published to the workflow. Runs non-strict on
    pushes to master today." Delete the workflows and this item disappears with
    them; the strict run survives locally in `finish-merge.sh` regardless, which
    is why this one is a loss of redundancy rather than a loss of enforcement.

- [ ] **Correct two safety registers that cite CI evidence which was never
  produced.** The rule above — no verification record may imply these gates ran
  — is not abstract, and it was wrong of the first draft of this section to
  state it without naming a single affected document. Two are known, both found
  and read on 2026-09-17:

  - **`org-members/docs/risk/2026-09-02-membership-hazards.md` line 291**, in a
    **class C** hazard register. Of the `tauWindow` invariant it states: "CI
    runs that invariant under the simulator (5000 samples, 16 steps) and under
    Apalache to depth 5, so it is checked rather than merely written down."
    Every number in that sentence matches `quint.yml` exactly — and every one of
    those steps was added by the unpushed `620b459` and has never executed. The
    sentence is **load-bearing**: it is the clause that carries `tauWindow` from
    *asserted* to *checked*, and the paragraph around it is an argument about how
    much weight the staleness bound can take. Remove the evidence and the
    paragraph's own hedges — the check is bounded, the invariant is entailed by
    the action guard — are all that is left, which is a materially weaker
    position than the register currently states.
  - **`org-node/docs/risk/2026-09-09-org-node-hazards.md` line 813-814**, at the
    end of the reorganisation assessment (S3 / P1, control not minted). Its
    wording is "Minted when the chopsticks lane runs in CI (setup checklist)" —
    a **minting condition**, not an assertion that anything ran, and the
    sentences before it say plainly that the submitter's half "is tested only in
    the chopsticks lane … which the merge gate does not run". So it is not false
    and must not be filed as if it were. What it is is a condition that cannot
    currently be met: `rust.yml` excludes the chopsticks-dependent targets from
    org-node's `test` step by design (`rust.yml` line 72), and `rust.yml` has
    never run at all, so the trigger for minting that control is doubly out of
    reach. A reader of that register should know that.

  **The owner's decision, 2026-09-17: neither register is corrected in this
  change, and that is deliberate, not an oversight.** Two reasons. First,
  changing what a hazard's residual-risk argument rests on is a **risk
  decision** — it belongs to the `analyze-risks` skill, with the acceptability
  matrix in hand and the option to re-assess or mint, not to a prose-repair
  change editing a sentence. Second, the change that found this
  (`worktree-guardrails-on-chain-client-finalize`) is deliberately
  documentation-only and confined to the on-chain-client finalisation; editing
  these two files would pull **org-members' and org-node's** risk ledgers into
  it and oblige it to own both units' gates. So this is filed, open, and carried
  to whoever runs the next risk analysis on either unit. It is outstanding
  because it was deferred on purpose, not because it was missed. No sweep for
  further affected documents was performed; these two are the ones that were
  found, and the general rule stands over the rest.

- [ ] **Fix two stale verification-record references written by `fbf17f0`.**
  Both cite `docs/verification/2026-09-10-worktree-guardrails-on-chain-client-risk.md`,
  a path that does not exist and never did — the record landed as
  `docs/verification/2026-09-11-worktree-guardrails-on-chain-client-risk.md`:

  - **`Makefile` line 47**, inside the "RESOLVED, 2026-09-10" note that records
    the owner's recalibration of on-chain-client's coverage floors;
  - **`on-chain-client/.guardrails/config.yaml` line 106**, in the comment above
    `coverage_command:` that repeats the same pointer.

  Both were written by `fbf17f0` — the same commit that skipped
  `finalize-docs.sh` — and they are the same species of damage: a reference
  written to a file name that the merge was expected to produce and did not.
  They are **not** fixed by the repair change. Editing `Makefile` or a unit's
  `config.yaml` would cost that change its documentation-only property and pull
  gate configuration into a prose repair, which is exactly the trade this
  project declines. Recorded here so they are left knowingly. Neither breaks a
  gate — no script resolves these paths — so the cost is a reader following a
  pointer to nothing.

## Added 2026-09-28 — a class C item with no verifiable low-level requirement (`docs/plans/2026-09-28-on-chain-client-architecture.md`)

Tooth 4's second unit gave `on-chain-client` its first architecture ledger:
nine software items and thirty-two low-level requirements. **Eight of the nine
items carry LLRs. The ninth carries none, and that is a recorded deviation from
class C rather than an omission.**

- [ ] **SDD-3b8zef has no low-level requirements, because nothing at this
      unit's gate can verify one.** The item is the chain-facing transport
      shell in `on-chain-client/src/client.rs` — decoder pinning at
      construction, `get_org_state`'s three-slot read, `read_contract_slot`,
      `subscribe` and both lanes, `decode_contract_events` — the majority of
      that file, and the majority of the unit's source by volume. llvm-cov
      analyses 336 lines of `client.rs` out of the 491 it analyses across this
      crate, at 16.37% line coverage. (The hand-computed "roughly 580 of 699"
      was dropped 2026-09-28 by this change's review sweep as unreproducible;
      the decomposition file states why.)
      IEC 62304 class C requires LLRs per software item, verified at the item's
      own interface; this one has none. Two ways to close it, both code or
      infrastructure changes owed their own red-first cycle: continue the
      extraction that already produced `internals` (which is what made
      SDD-bw7v5x, SDD-4z3k2u and SDD-m59zrg verifiable items) until the
      remaining decisions are chain-free, or give this unit's gate a harness
      that can run the five chopsticks targets that reach it. **The
      decision this item records is which.**

- [ ] **The coverage shortfall and the LLR gap are the same gap measured
      twice.** `client.rs` at 16.37% line coverage, and SDD-3b8zef having no
      verifiable LLR, are one fact seen from two directions — the code no gated
      test reaches. The register has listed the coverage half since 2026-08-26
      (recalibrated 2026-09-10, floors at 41/42) without the other half being
      visible. They are cross-referenced here so that closing one is understood
      to close the other, and so that a future coverage bump obtained by any
      means other than reaching that code is recognised as not closing either.

**What was refused, and why it is written down.** LLRs for SDD-3b8zef could
have been minted and annotated onto any of the nine ungated integration
targets — or onto any other text file under `test_paths`, committed or not —
and `check-trace.sh` would have reported a clean tree. It would, because
MISSING-TEST is satisfied by a `verifies:` reference **in a file under
`test_paths`**, not by an executed assertion — and `on-chain-client/tests` *is*
`test_paths`, so those targets count whether or not anything ever runs them.
That was verified against the script rather than assumed. Taking it would have
produced a green gate standing over evidence that was never produced, which is
the defect the 2026-09-17 section above records against this repository's own
hazard registers. The owner was asked on 2026-09-28 and refused it. The gap is
therefore visible in three places — the item's own text, the architecture
README's Overview, and this register — rather than absent from all three.

**Added 2026-09-29, review round 5: the third option, and why it was declined.**
Rounds 1 through 4 all treated this as a two-way choice — annotate LLRs onto
ungated targets, or mint none. There is a third: **mint them and leave them
unannotated**, so `check-trace.sh` reports MISSING-TEST for each and exits 1
(`check-trace.sh:501-503`). Round 4 raised it and judged it *stronger* than what
was done, on the ground that it makes the gap machine-visible.

Round 5 showed that judgement is wrong, using the very fact recorded two
paragraphs above: MISSING-TEST is discharged by a `verifies:` reference in any
text file under `test_paths`. So the red option three raises is **one line away
from a green**, and typing that line converts a recorded class C deviation into
a clean gate that no longer reads as a deviation. It hands the next person a
one-line erasure that leaves no argument behind, where deleting this section
would at least require deleting the reasons. That is the objection, and it is
sufficient on its own.

Two reasons first given for declining option three are **withdrawn**, and are
recorded as withdrawn rather than quietly dropped. "It reddens the gate on a
tree where nothing is broken" switches criteria: under the criterion this whole
section argues from, a class C item missing its required low-level requirements
*is* a defect and a red gate would be accurate. And "a repository whose gates
are red for recorded reasons teaches its readers to stop reading red" is an
assertion about people that nothing here measured.

**The decision is the author of the architecture change, not the owner, and it
is recorded here so it can be overridden.** The owner was asked on 2026-09-28
about the two-way choice, not about this one. Nothing is owed to close this item
except a decision that the erasability objection is wrong — in which case option
three should be taken and this register entry is where to say so.

**Corrected 2026-09-28, review round 1 finding-4.** Both paragraphs above said
*nine* ungated integration targets. Only **five** of the nine files in
`on-chain-client/tests` that run at no gate exercise `OrgRegistryClient` at all
— `off_chain_genesis_ceremony`, `p_address_is_orgid`, `reorg_cancels_proposed`,
`scenario_a_full`, `two_orgs_one_watcher`. Of the rest, `00_chopsticks_sanity`
and `01_multisig_sanity` import nothing from the crate, `smoldot_smoke` touches
only `decode::dispatch::for_runtime` (SDD-v2rtka, whose version-resolution
interface is fully gated) and needs live Paseo rather than chopsticks, and
`regenerate_corpus` is an `#[ignore]`d corpus writer that constructs no client.
The error was made in this register and in the architecture ledger on the same
day and caught in both by the same finding; it is corrected in both rather than
in one, because two files disagreeing about the same set is worse than either
number alone.

**Both numbers are right, for different questions**, and the first pass at this
correction blurred them. *Nine* is how many targets run at no gate, and so how
many places a `verifies:` annotation could hide — the review established the
hiding place is wider still, since MISSING-TEST is a text scan of `test_paths`
that consults no cargo target and passes `--untracked`, so a README or an
uncommitted file serves equally. *Five* is how many of those targets exercise
`OrgRegistryClient`, and so how much of SDD-3b8zef anything reaches at all.
Replacing nine with five everywhere would have understated the hazard while
fixing the overstatement.

The same finding narrowed the deviation's scope. `ClientError` and
`SubscribedEventStream` were listed under SDD-3b8zef, the item whose whole
claim is that nothing at this unit's gate reaches it. `ClientError` is
chain-free — three variants, a hand-written `Display`, a `From<DecodeError>` —
and is re-exported at the crate root under a feature this unit's gate command
enables by default, so it is reachable from the gated tests today. It has moved
to SDD-5wamsz, and **an LLR for its `Display` and its conversion is writable and
gate-verifiable and is not yet written**: minting it needs a new gated cargo
target, which the architecture tooth deliberately does not add. That is a third
open item on this gap, smaller than the other two and the cheapest of the three
to close.

## Added 2026-09-28 — "anvil" is a phantom tool in this repository's prose

- [ ] **Eighteen occurrences across nine documents, plus two in the `Makefile`,
      describe on-chain-client's ungated targets as needing "a chopsticks fork or
      anvil". No target uses anvil.** The count has now been measured three
      times and stated wrongly twice, which is itself worth recording: "roughly
      twenty documents" quoted occurrences as documents, and round 4's
      replacement ("twenty-one across ten") was measured before its own edits
      landed and named a document that by then contained none. The figures above
      are as of the end of review round 5. Of the nine documents: **four are
      verification records**, left immutable by the decision below; **two are
      this change's own files**, where the phrase survives only as a quotation of
      the error; and **three are live documents still asserting it** —
      `docs/plans/2026-09-10-on-chain-client-risk-analysis.md`,
      `on-chain-client/docs/requirements/2026-09-10-chain-reading.md` and
      `on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md`.
      `docs/plans/2026-08-26-ratchet-gap-analysis.md` was a fourth until this
      change corrected its single use in passing.

      Measured on 2026-09-29: `grep -rli anvil` across `*.rs`, `*.toml`, `*.js`,
      `*.ts`, `*.json`, `*.sh`, `*.yml` and `*.yaml` returns **nothing at all** —
      not merely nothing outside comments, which is how this bullet first put it.
      Every fork-dependent target in `on-chain-client/tests` spawns chopsticks
      through `common::chopsticks_fork::spawn_fork`, including
      `01_multisig_sanity`, whose name suggests otherwise. The phrase is a term
      of art that has propagated through the SRS, the RMF, the problem ledger,
      two plans and several verification records without ever describing
      anything that exists.

      **This change corrected every use it wrote, and two it did not.** The
      propagation sweep found six in its own new documents; review round 4 found
      three more that this change had itself written into this very register —
      one inside the SDD-3b8zef owner item the deviation argument points readers
      to — plus a site in the decomposition where the first correction had
      garbled the sentence it repaired. Review round 5 found two more that round
      4 had newly written into
      `docs/plans/2026-09-10-on-chain-client-risk-analysis.md`. A change that
      declares the phrase a phantom while still adding new uses of it is the
      defect, not untidiness.

      It also, in the course of fixing an unrelated count, corrected the single
      pre-existing use in `docs/plans/2026-08-26-ratchet-gap-analysis.md`. So the
      earlier claim that this change "stopped there, deliberately" is **not
      true** and is withdrawn: it stopped at every *document* it had no other
      reason to open. The deferral below still stands for the three live
      documents it never touched. Replacing the phrase in some files and not
      others would recreate exactly the inconsistency the sweep existed to
      remove — the repository would then disagree with itself about which tool
      its tests need, which is the failure mode three review rounds of this
      change were spent on. The decision owed is whether "chopsticks or anvil"
      is retired repository-wide in one sweep, or kept as a deliberate term of
      art meaning "the ungated chain-dependent lane" and defined once in a
      glossary. Either is defensible; the present state, where it reads as a
      statement of fact and is not one, is not.

- [ ] **`docs/verification/` records carry claims this change measured as
      false** — `2026-09-11-worktree-guardrails-on-chain-client-risk.md:213`
      and `:219`, and two of the 2026-08-26 records, say the nine
      chopsticks/anvil targets exercise `client.rs`, where five do. They were
      **left untouched**: a verification record is the signed evidence of a
      review that happened, and amending it after the fact falsifies the record
      rather than correcting it. Every *live* ledger those records fed now
      carries a dated note pointing the other way. The decision owed is whether
      this project ever amends a verification record, and if so under what
      annotation — the alternative, which this change assumes, is that records
      are immutable and corrections live in the ledgers that are still read.
