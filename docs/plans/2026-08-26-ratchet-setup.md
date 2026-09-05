# Ratchet setup checklist

Human actions that `/ratchet` cannot perform. Living document: items are ticked
with their evidence as they close.

- Created 2026-08-26 at `guardrails_version: 0.4.0`.
- **Revised 2026-09-01 for `guardrails_version: 0.5.1`** (upstream
  `bb7eee5`). The signing items moved from "later tooth" to **gate on the
  ratchet itself** — see the box below — and a new proof step was added.
- **Superseded 2026-09-05** by `docs/plans/2026-09-05-ratchet-setup.md`,
  written when the repository became multi-unit (upstream `e2eac86`). Open
  items below were carried there; this file is kept as the record of the
  single-unit period and is not updated further.

Safety class **C** as of 2026-09-01
(`docs/adr/2026-09-01-safety-class-c.md`, superseding the Class B ADR).
Adoption order:
`docs/plans/2026-08-26-ratchet-gap-analysis.md`.

## Signing is a gate, not a later tooth

From 0.5.1 every merge ends with `finish-merge.sh`, which runs
`check-signing.sh --strict` **before** it removes the worktree and deletes the
branch. Strictness begins at the first merge, not in CI. A project that defers
signing completes each merge and is then refused the cleanup, accumulating
worktrees with no obvious cause.

**This project is there.** Measured by the project owner on their own
terminal, 2026-09-01: `check-signing.sh --setup` → **exit 0**, and the
`finish-merge.sh` run for the 0.5.1 upgrade passed guard 1
(`check-signing --strict`) and completed cleanup. That is the ratchet's own
completion condition met.

- [x] **Commit signing proved end to end.** DONE 2026-09-01. `--setup` exits 0
      from the primary checkout: signing key present, format resolved, and a
      real signed commit in a throwaway repository reads `%G?` = `G`.
- [x] **Signature verification works.** DONE 2026-09-01, by the same proof.
      `--strict` verifies against the GPG keyring; no allowed_signers file is
      needed because this project signs with GPG rather than SSH.

**A correction, recorded rather than quietly fixed.** An earlier revision of
this document stated that `--strict` "has **never** passed in this project", and
called it measured. Every such measurement had been taken inside an agent
sandbox that cannot open `~/.gnupg/trustdb.gpg`, where gpg fails closed and a
valid signature reads as unverifiable. An independent reviewer measured the same
thing and it was treated as confirmation — it ran in the same sandbox, so it was
one blind spot counted twice. The measurements were real; the scope claimed for
them was not. `docs/verification/2026-09-01-worktree-guardrails-051.md` carries
the same error and is left unedited, because a verification record is evidence
for a completed change: corrections belong in the next record, which is this
change's.

The configuration behind that: a GPG `user.signingkey`
(`0x224A50748E1745B8E1CC9A284FD03A5A7780F634`), `commit.gpgsign=true` globally
with the shared-repo override removed on 2026-08-26, `commit.gpgsign=false` per
worktree by this project's own rule, and four signed merges on `master`
(`a120f14`, `4bb5509`, `067e56c`, `4da12d3`).

One operational note, since it cost a false diagnosis: run `--setup` from the
**primary checkout**. Inside a worktree the deliberate `commit.gpgsign=false`
makes it report that key as missing, which is an artefact of where it ran and
not a finding about the machine.

## Remaining items

- [ ] **Branch protection on `master`.** No direct pushes, require signed
      commits.
- [x] **CI.** DONE 2026-08-26 — `.github/workflows/rust.yml`: `test` (the two
      cargo entries of `verify_commands`, with quint installed because
      `mbt_conformance` fails rather than skips without it), `clippy` (lib
      targets, `-D warnings`), `no-std` (org-members' wasm32 matrix),
      `coverage`, and `guardrails`. **Revised understanding as of 0.5.1:** a
      `check-signing --strict` step in CI would be a *backstop* for what
      `finish-merge.sh` already enforced on the merging machine, not the point
      at which strictness begins. It currently runs non-strict, which is now
      the weaker of the two — worth tightening once the item above closes.
      `check-review.sh` runs on pull requests only, and this project merges
      locally without PRs, so that gate still executes nowhere in CI.
- [x] **Statement coverage.** DONE 2026-08-26 — `cargo-llvm-cov` 0.9.0 and
      `llvm-tools-preview` installed; `coverage_command: make coverage` set.
      Measured: org-members 93.50% lines / 92.20% regions, on-chain-client
      53.57% / 59.19%. Both metrics gated, floors a point below measurement,
      ratcheting upward only. on-chain-client is an accepted shortfall — see the
      tooth 3 verification record.
- [ ] **Decision coverage — NEW, and now mandatory.** Class C requires
      statement **and** decision coverage. Decision coverage is not measured and
      cannot be on the current toolchain: `cargo llvm-cov --branch` is unstable
      and fails under stable rustc. A nightly toolchain is installed but lacks
      the component. Needed:
      `rustup component add llvm-tools-preview --toolchain nightly`, then a
      `--branch` lane in the Makefile and in CI's `coverage` job, with floors
      set from the first measurement. Until then this is a gap against a
      mandatory requirement, not a stretch target — the distinction the class
      change introduces.
- [ ] **Human review policy — re-opened by the class change.** Decide who
      signs off a merge, and who acts as the independent reviewer at
      `merge-change` step 6a when a human is preferred over a fresh agent.
      Class C calls for a **thorough** review, and for considering **two**
      independent reviewers on critical items; Class B's one reviewer is no
      longer the standard. Every change so far has used one fresh subagent,
      recorded by name in the record — and those reviews have each found real
      defects, including annotations that verified nothing, so the mechanism
      works and the question is only how much of it to require.
- [x] **Risk acceptability matrix.** DONE 2026-09-01, by the project owner:
      S3 UNACCEPTABLE at every probability, S2 acceptable only at P1, S1
      acceptable throughout. The injury pathway the class B ADR asserted was
      substantiated harder than that ADR allowed and the class moved to C.
- [ ] **Publish the restriction on use.** The hazard analysis
      (`docs/risk/2026-09-02-membership-hazards.md`) concludes that nine of
      twelve hazards carry residual risk the matrix calls unacceptable, and that
      overall residual risk against the intended use is UNACCEPTABLE: the
      software has not reached the use that makes it Class C. A restriction —
      no production deployment in the journalism, clinical or government
      context — has to be written where a user meets it (repository README and
      crate documentation at minimum) before it counts as an implemented
      control under ISO 14971 clause 7.2. Today it exists only inside the risk
      ledger, which is the one place a user will not look. Owner's decision,
      not an agent's.
- [ ] **Overall residual-risk criteria.** ISO 14971 clause 8 wants the overall
      evaluation made against criteria set out in a risk management plan. This
      project has no plan: `docs/risk/README.md` holds a per-hazard matrix and
      says nothing about the whole. So the register's overall verdict is a
      reasoned conclusion rather than the output of a stated criterion. Write
      the criteria — including what would let the restriction above be lifted —
      into the risk ledger README.
- [ ] **Problem-report limits.** `problem_age_days: 30`, `problem_open_max: 10`.
      One item is open (PR-zz4exm, opened 2026-08-31), so the age limit now has
      a live subject. Re-check both numbers at tooth 8's backfill.
- [ ] **Dev-environment provisioning.** (a) `npm install` in
      `on-chain/scripts`, without which nine `on-chain-client` integration
      targets cannot run — the largest hole in `verify_commands`; (b) a
      writable `~/.quint`, without which the MBT conformance test inside
      `cargo test -p org-members` fails; (c) a writable `CARGO_HOME` — this
      machine's `~/.cargo` is read-only under the agent sandbox, and
      `CARGO_HOME=/tmp/cargo_home_fuzz` is the workaround.
- [x] **Tool qualification (DO-330-lite).** DONE 2026-09-01. Qualification
      basis: the guardrails bats suite at `guardrails_version: 0.5.1`, upstream
      commit `bb7eee5`. Result: **435 tests, 435 ok, 0 not ok, exit 0**, run
      once via `<guardrails>/tests/run-tests.sh` in the guardrails repository —
      not here, and with no bats installed anywhere. Re-record version and
      result whenever `/ratchet` updates the scripts. Do not modify the scripts
      in this project; change them upstream, where the tests live. (The 0.4.0
      install recorded no suite result, which is why the awk defect below went
      unnoticed at install time.)
- [ ] **Scope note.** Guardrails supports a quality management system; it is
      not itself regulatory compliance. The quality manual, design controls and
      human sign-offs govern, and any notified-body or auditor requirements
      stay authoritative over anything written here.

## Closed by the 0.5.1 upgrade

- **`check-review.sh` could not run on this machine at all** under 0.4.0: its
  record scan handed `awk -v` a value carrying literal newlines, which macOS's
  BWK awk rejects outright. Step 6c therefore never executed for the first
  three changes, and their records were hand-checked instead. Reported
  upstream, fixed in `31e2303`, and upstream's `tests/portability.bats`
  now carries nine tests for the platform defects that commit fixed, six of
  them on the awk-newline rule and three on `date` and `sed`. All three existing records were re-run against the fixed gate on
  2026-09-01 and pass: 13, 25 and 32 findings, exit 0 each.
- The same upstream commit fixed `sed` and `date` portability on macOS —
  latent defects in the 0.4.0 copy that this project had not yet triggered.
