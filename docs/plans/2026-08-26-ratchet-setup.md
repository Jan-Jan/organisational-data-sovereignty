# Ratchet setup checklist — 2026-08-26

Human actions that `/ratchet` cannot perform. Installed toolkit:
`guardrails_version: 0.4.0` from `~/Coding/guardrails`
(`e6e8acc feat: an open problem report states its owner, its age, and that it
is open`). Safety class **B** (`docs/adr/2026-08-26-safety-class-b.md`).
Adoption order: `docs/plans/2026-08-26-ratchet-gap-analysis.md`.

- [ ] **Commit signing — remove the override, do not add a key.** A GPG
      `user.signingkey` (`0x224A50748E1745B8E1CC9A284FD03A5A7780F634`) is
      configured and `commit.gpgsign=true` is already set globally in
      `~/.gitconfig`. What makes every commit on `master` unsigned is
      `commit.gpgsign=false` in the **shared repository config**, which
      overrides the global. Remove that line
      (`git config --unset commit.gpgsign` from the primary checkout) and keep
      the per-worktree `false` — that combination is exactly this project's own
      stated rule, which also forbids writing `commit.gpgsign` into the shared
      config from a worktree in the first place. If a merge lands unsigned from
      an agent session, re-sign with `git commit --amend -S` from a regular
      terminal. For SSH signing instead: `git config gpg.format ssh` and
      `git config user.signingkey <path-to-pubkey>`. Hardware keys need a
      physical touch per signature.
- [ ] **Signature verification.** Create an allowed_signers file listing each
      committer (`<email> <key-type> <public-key>`), then
      `git config gpg.ssh.allowedSignersFile <path>`. Until it exists,
      `check-signing.sh` passes signed commits with `WARN-UNVERIFIED`; after
      it, enable `--strict` in CI. (GPG signing verifies against the keyring
      instead; the allowed_signers file is the SSH-signing equivalent.)
- [ ] **Branch protection on `master`.** No direct pushes, require signed
      commits. This is the mechanical half of guardrails non-negotiable 1.
- [ ] **CI.** Add a **general** Rust job. One already exists but is narrow:
      `.github/workflows/quint.yml` has an `mbt` job that installs a Rust
      toolchain and runs `cargo test --test mbt_conformance` in `org-members` —
      one test target in one crate. The new job should run the
      `verify_commands` from `.guardrails/config.yaml` plus
      `.guardrails/scripts/check-ids.sh`, `.guardrails/scripts/check-trace.sh`,
      and `.guardrails/scripts/check-signing.sh --strict <base>..HEAD` on every
      merge, and ideally `cargo clippy` over the workspace (the root
      `Cargo.toml` already denies `unwrap_used`, `expect_used` and `panic` at
      the workspace level, so this is enforcement of a rule already written).
      **Do not add `check-review.sh` to the base-branch job** — it asks about
      the change under merge, so on `master` it exits 2 rather than reporting a
      pass over no question. It runs from the change worktree at
      `merge-change` step 6c; to gate it on a pull request, name the branch:
      `check-review.sh --branch <head-branch>`.
- [ ] **Coverage tooling (Class B target: statement coverage).** Install
      `cargo-llvm-cov` and the `llvm-tools-preview` component, then set
      `coverage_command` in `.guardrails/config.yaml`. It is unset today
      because the tool is absent (`cargo llvm-cov --version` → "no such
      command"), which leaves the class target stated but unmeasured.
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
