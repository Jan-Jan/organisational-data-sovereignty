# Ratchet setup checklist — 2026-08-26

Human actions that `/ratchet` cannot perform. Installed toolkit:
`guardrails_version: 0.4.0` from `~/Coding/guardrails`
(`e6e8acc feat: an open problem report states its owner, its age, and that it
is open`). Safety class **B** (`docs/adr/2026-08-26-safety-class-b.md`).
Adoption order: `docs/plans/2026-08-26-ratchet-gap-analysis.md`.

- [x] **Commit signing — remove the override, do not add a key.** DONE
      2026-08-26: `git config --unset commit.gpgsign` in the primary checkout,
      so the global `commit.gpgsign=true` now applies and the per-worktree
      `false` stays — the combination this project's own rule prescribes. The
      first guardrails merge (`a120f14`) carries a PGP signature from key
      `0x224A50748E1745B8E1CC9A284FD03A5A7780F634`. Signing is on; VERIFYING it
      is the next item. If a merge ever lands unsigned from an agent session,
      re-sign with `git commit --amend -S` from a regular terminal.
- [ ] **Signature verification.** Create an allowed_signers file listing each
      committer (`<email> <key-type> <public-key>`), then
      `git config gpg.ssh.allowedSignersFile <path>`. Until it exists,
      `check-signing.sh` passes signed commits with `WARN-UNVERIFIED`; after
      it, enable `--strict` in CI. (GPG signing verifies against the keyring
      instead; the allowed_signers file is the SSH-signing equivalent.)
- [ ] **Branch protection on `master`.** No direct pushes, require signed
      commits. This is the mechanical half of guardrails non-negotiable 1.
- [x] **CI.** DONE 2026-08-26 — `.github/workflows/rust.yml` adds five jobs:
      `test` (the two cargo entries of `verify_commands`, with quint installed
      because `mbt_conformance` fails rather than skips without it), `clippy`
      (lib targets, `-D warnings`), `no-std` (org-members' documented wasm32
      matrix), `coverage`, and `guardrails`. Three scoping decisions are
      argued in the workflow's own comments, and each is a limit worth knowing:
      check-signing runs on the BASE BRANCH only and non-strict — on a pull
      request it would fail every change, because worktree commits are
      deliberately unsigned by this project's own rule, and a runner holds no
      key to verify with; check-ids takes `--allow-draft-files` on pull
      requests only, since a draft ledger file is legitimate mid-change and a
      defect once merged; and clippy covers lib targets only, which is partly
      the project's rule ("no exceptions in lib code; `unwrap()` is fine in
      tests" — `--all-targets` reports 144 lint errors across on-chain-client's
      13 test targets) and partly a concession, since it also suppresses one
      ordinary `manual_contains` lint in org-members' own test code that ought
      simply to be fixed.
      **`check-review.sh` runs on pull requests only**, and this project merges
      locally without PRs — so today that gate, the class B independent-review
      evidence check, executes nowhere. Adopting PRs or landing the upstream
      awk fix is what closes it.
- [x] **Coverage tooling (Class B target: statement coverage).** DONE
      2026-08-26 — `cargo-llvm-cov` 0.9.0 and `llvm-tools-preview` installed;
      `coverage_command: make coverage` is set, and the `coverage` job in CI
      is what enforces it mechanically (no guardrails check script reads that
      key; locally it is verify-before-merge step 5, i.e. discipline).
      Measured: org-members 93.50% lines / 92.20% regions; on-chain-client
      53.57% / 59.19%. BOTH metrics are gated, because line coverage
      over-credits multi-statement lines and region coverage is the closer
      analogue of the class B statement target. Floors sit a point below the
      measurements to absorb platform drift, and ratchet upward only.
      on-chain-client's figure is a **class B shortfall, not a target met** —
      `client.rs` is at 12.75% and `decode/mod.rs` at 0.00%, both waiting on
      the chopsticks lane. The shortfall is accepted explicitly in this
      change's verification record; the provisioning item below is what
      retires it.
- [ ] **Human review policy.** Decide who signs off a merge, and who acts as
      the independent reviewer at `merge-change` step 6a when a human is
      preferred over a fresh agent. Class B calls for one reviewer. Whoever it
      is, the name goes in the change's verification record under `reviewer:`
      — the gate does not judge independence, so this policy is the only thing
      that makes the field mean anything.
- [ ] **Risk acceptability matrix.** `docs/risk/README.md` ships with every
      cell as `TBD`. Fill each with ACCEPTABLE or UNACCEPTABLE per the quality
      manual before `analyze-risks` runs (tooth 5). This is a policy decision,
      not an agent's. Tooth 5 also has to substantiate or retract the injury
      pathway the safety-class ADR asserts — the classification rests on it.
- [ ] **Problem-report limits.** `problem_age_days: 30` and
      `problem_open_max: 10` are the shipped defaults and are currently
      harmless — the ledger holds no items. They fail a merge once exceeded,
      so re-check both numbers against the real backlog at tooth 8, when the
      existing "Known follow-ups" lists are backfilled as PR items.
- [ ] **Dev-environment provisioning — this is what makes the merge gate
      whole.** Three things are unprovisioned, and two of them leave holes in
      `verify_commands` rather than merely in convenience:
      (a) `npm install` in `on-chain/scripts`, without which nine
      `on-chain-client` integration targets cannot run — they are excluded via
      `--lib` today, which is the largest hole in the gate;
      (b) a writable `~/.quint`, without which the MBT conformance test inside
      `cargo test -p org-members` fails (it skips cleanly only when `quint` is
      absent from `PATH` altogether);
      (c) a writable `CARGO_HOME` — this machine's `~/.cargo` is read-only
      under the agent sandbox, and `CARGO_HOME=/tmp/cargo_home_fuzz` is the
      workaround. A `make provision` target covering (a) and (b) would let the
      excluded targets come back into `verify_commands`.
- [ ] **Tool qualification (DO-330-lite).** The `.guardrails/scripts/` are
      verification tools: their failure could mask errors. Qualification basis
      is the guardrails bats suite at `guardrails_version: 0.4.0`, the version
      recorded in `.guardrails/config.yaml`. The suite was **not** run as part
      of this install — record its result here when it is, and re-record both
      version and result whenever `/ratchet` updates the scripts. Do not
      modify the scripts in this project; change them upstream in
      `~/Coding/guardrails`, where the tests live.
- [ ] **Scope note.** Guardrails supports a quality management system; it is
      not itself regulatory compliance. The quality manual, design controls
      and human sign-offs govern, and any notified-body or auditor
      requirements stay authoritative over anything written here.
