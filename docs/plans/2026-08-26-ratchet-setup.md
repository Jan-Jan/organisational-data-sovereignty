# Ratchet setup checklist

Human actions that `/ratchet` cannot perform. Living document: items are ticked
with their evidence as they close.

- Created 2026-08-26 at `guardrails_version: 0.4.0`.
- **Revised 2026-09-01 for `guardrails_version: 0.5.1`** (upstream
  `bb7eee5`). The signing items moved from "later tooth" to **gate on the
  ratchet itself** — see the box below — and a new proof step was added.

Safety class **B** (`docs/adr/2026-08-26-safety-class-b.md`). Adoption order:
`docs/plans/2026-08-26-ratchet-gap-analysis.md`.

## Signing is a gate, not a later tooth

From 0.5.1 every merge ends with `finish-merge.sh`, which runs
`check-signing.sh --strict` **before** it removes the worktree and deletes the
branch. Strictness begins at the first merge, not in CI. A project that defers
signing completes each merge and is then refused the cleanup, accumulating
worktrees with no obvious cause.

**This project is not there yet.** Measured 2026-09-01:
`check-signing.sh --setup` → **exit 1**, naming two missing pieces. Until it
exits 0, adoption of guardrails is not finished, whatever else is ticked.

- [ ] **`gpg.format` is not set.** Git defaults to openpgp when it is unset, so
      signing works today and `--setup` still refuses it: configuration that is
      merely implied cannot be proved. Set it explicitly —
      `git config --global gpg.format openpgp` — rather than in the shared repo
      config, which is where this project already had one signing key cause
      trouble.
- [ ] **Run `--setup` from the primary checkout, never a worktree.** It also
      reported `commit.gpgsign` missing, which is an artefact of where it ran:
      this project deliberately sets `commit.gpgsign=false` per worktree (its
      own rule, and the right one), so the effective value inside a worktree is
      `false`. `--setup` asks what this machine can do, and only the primary
      checkout answers that question.
- [ ] **Signature verification — now load-bearing.** Create an allowed_signers
      file listing each committer (`<email> <key-type> <public-key>`) and set
      `gpg.ssh.allowedSignersFile`, or make the GPG keyring able to verify the
      committer's own key. Without it every signature reads as unverifiable,
      which `--strict` rejects — and `--strict` is now what stands between a
      merge and its cleanup. Note that `--strict` has **never** passed in this
      project, and that is measured rather than assumed: non-strict runs report
      `WARN-UNVERIFIED` and exit 0, while `check-signing.sh --strict
      067e56c~3..067e56c` over the three existing merges reports three
      `UNVERIFIED` lines and exits 1.
- [ ] **Prove the chain, and only then is the ratchet done:**
      `.guardrails/scripts/check-signing.sh --setup` must exit 0. It checks
      `gpg.format`, `user.signingkey`, `commit.gpgsign` and the format's trust
      root, then makes a real signed commit in a throwaway repository and
      confirms it reads `%G?` = `G`. Configuration being present says nothing
      about whether the key can sign or the signature verifies.

What is already in place: a GPG `user.signingkey`
(`0x224A50748E1745B8E1CC9A284FD03A5A7780F634`), `commit.gpgsign=true` globally
with the shared-repo override removed on 2026-08-26, and three signed merges on
`master` — `a120f14`, `4bb5509`, `067e56c` — with this change due to be the
fourth.

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
- [x] **Coverage tooling (Class B target: statement coverage).** DONE
      2026-08-26 — `cargo-llvm-cov` 0.9.0 and `llvm-tools-preview` installed;
      `coverage_command: make coverage` set. Measured: org-members 93.50% lines
      / 92.20% regions, on-chain-client 53.57% / 59.19%. Both metrics gated,
      floors a point below measurement, ratcheting upward only. on-chain-client
      is an accepted class B shortfall — see the tooth 3 verification record.
- [ ] **Human review policy.** Decide who signs off a merge, and who acts as
      the independent reviewer at `merge-change` step 6a when a human is
      preferred over a fresh agent. Class B calls for one reviewer. Every
      change so far has used a fresh subagent, recorded by name in the record.
- [ ] **Risk acceptability matrix.** `docs/risk/README.md` ships with every
      cell as `TBD`. Fill each with ACCEPTABLE or UNACCEPTABLE per the quality
      manual before `analyze-risks` runs (tooth 5). Tooth 5 also has to
      substantiate or retract the injury pathway the safety-class ADR asserts.
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
