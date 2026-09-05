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
- [ ] **app's verify_commands in CI.** Neither runs in CI, for two different
      reasons: the cargo entry needs Tauri's Linux system packages (webkit2gtk,
      libsoup, …) rust.yml does not install; the npm entry needs
      `npm --prefix app ci` first (app/node_modules is gitignored) — node
      itself is already set up in the `test` job. Until both are added, the
      merge gate is app's only gate.
- [ ] **Coverage for org-node and app.** Neither has a `coverage_command`. Under
      class C statement and decision coverage are mandatory, so these are gaps
      against a requirement, not decisions. Add a `coverage-org-node` Makefile
      target from a first measurement; the app has nothing to measure until it
      has tests.
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
