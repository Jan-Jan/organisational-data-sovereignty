# Ratchet gap analysis — 2026-08-26

Retrofit of guardrails (IEC 62304 / ISO 14971 discipline) onto the two-tier
access-control project. Source toolkit: `~/Coding/guardrails` at
`guardrails_version: 0.4.0`. Mode: **retrofit** — 75 commits on `master`,
seven Rust crates, an existing `AGENTS.md`, an existing `docs/` tree, and a
quint CI workflow. Nothing existing was migrated or overwritten in this change.

The seven crates: `org-members`, `org-node`, `spike-common`, `spike-keyhive`,
`spike-p2panda` (root workspace members), `on-chain-client` (deliberately
outside the root workspace, its own `[workspace]`), and `app/src-tauri`.

## What exists

| Area | State before this change |
|---|---|
| Rules file | `AGENTS.md` (project-wide) + `org-members/AGENTS.md` (crate). Terse, opinionated, with sticky user preferences and a lessons-learned list. No `CLAUDE.md`. |
| Requirements | None in ID'd form. Behaviour is specified in prose across `docs/superpowers/specs/*` (20 design specs) and the crate `AGENTS.md` files (domain vocabulary, public-API style, "critical invariants (verified by tests)"). |
| Risk management | None. No hazard analysis, no risk controls, no acceptability matrix. Safety-relevant reasoning exists but is scattered through the specs (fork safety, revocation safety, τ-window, convergence). |
| Architecture | No SDD/LLR items. Decomposition is described in `docs/superpowers/specs/*-design.md` per phase, plus `docs/phase-1d/*` (spike comparison, gap matrix, decisions). |
| SOUP inventory | None as a document. Dependency facts live in `Cargo.toml` and in lockfiles — see the caveat under tooth 6: the root `Cargo.lock` is **gitignored**, so the pins that matter most are not in a controlled configuration item. |
| Problem log | None. Bugs are recorded as commit messages (`fix(app): …`, `fix(chain): …`) and as "Known follow-ups" / "Lessons learned" sections in `AGENTS.md`. |
| Verification evidence | Not archived. Evidence is live-only: `cargo test`, proptest and libfuzzer suites, quint `test`/`run`/`verify` invariants, MBT conformance (`org-members/tests/mbt_conformance.rs`), Apalache bounded verify. Results are in CI logs and in `docs/phase-1d/*-results.md`. |
| Tests | Per-crate `tests/` (`org-members`, `org-node`, `on-chain-client`, `spike-*`, and `app/src-tauri`). No `verifies:` annotations anywhere in the tree. |
| CI | `.github/workflows/quint.yml` only, with three jobs: `quint` (typecheck/test/run invariants), `apalache` (bounded verify, `continue-on-error`), and `mbt`, which installs a Rust toolchain (`dtolnay/rust-toolchain@stable`) and runs `cargo test --test mbt_conformance` in `org-members`. So there **is** a Rust job; what is missing is a general Rust build/test/clippy job over the workspace. |
| Signing | The last 20 commits on `master` are all `%G? = N` (unsigned). The cause is a scope override, not an absent key: `commit.gpgsign=true` is set globally in `~/.gitconfig`, and `commit.gpgsign=false` is set in the **shared repository config**, which wins. A GPG `user.signingkey` (`0x224A50748E1745B8E1CC9A284FD03A5A7780F634`) is configured. No `gpg.ssh.allowedSignersFile`. |
| Worktrees | Already the working practice: five live worktrees under `.claude/worktrees/`, which was already gitignored before this change. |
| Item IDs | None. A tree-wide search for `(REQ|HAZ|RC|SDD|LLR|PR)-…` returns nothing, so the ID namespace starts clean and no legacy sequential IDs need to be honoured. |

## Conflicts with the guardrails block (flagged, not deleted)

These existing rules in `AGENTS.md` sit outside the managed block and were left
exactly as written — the append added 86 lines and deleted none. Where they
disagree, the managed block governs.

1. **"Always work in a git worktree for feature work."** Narrower than
   guardrails non-negotiable 1, which covers documentation changes too. Under
   guardrails a doc change is a change and takes a worktree and a squash merge.
   What the history actually evidences is weaker than a process claim:
   `master` is linear and every commit on it is a single commit, which is what
   a squash merge and a direct commit look like alike. So the record cannot
   tell us whether the last 20 commits bypassed a worktree — only that they
   are all unsigned, which point 2 covers.
2. **`master` receives unsigned commits.** Guardrails non-negotiable 2 requires
   the squash commit to be signed and verified by `check-signing.sh`, with no
   unsigned fallback. This is the largest single gap and is tooth 2. Note what
   the fix actually is: signing is already enabled globally and disabled by a
   line in the shared repository config. Writing `commit.gpgsign` into the
   shared repo config is exactly what this project's own rule (`AGENTS.md`,
   user preferences) forbids doing from a worktree — so tooth 2 starts by
   removing that line, not by configuring a key.
3. **"Disable gpg signing for git commits in worktrees, UNLESS merging into
   master."** No conflict — this is the guardrails rule stated in the
   project's own words, and it is what this change followed (a per-worktree
   `commit.gpgsign=false` in the worktree's own config, never in the shared
   config).
4. **"Always include fuzz testing."** Compatible, and it shaped
   `verify_commands`: `on-chain-client`'s three libfuzzer targets (`[[test]]`
   entries with `harness = false`: `fuzz_decode_org_state`,
   `fuzz_parse_revive_event`, `fuzz_event_round_trip`) are named explicitly so
   that restricting that crate to `--lib` does not silently drop them, and
   `org-members`' proptest suite (`tests/fuzz_tests.rs`) is inside
   `cargo test -p org-members`. Each fuzz target runs its corpus for ~1s under
   plain `cargo test`. The gap that remains: no fuzz lane exists for
   `org-node`, `app/src-tauri` or the spike crates.
5. **"`/superpowers:brainstorming` before any non-trivial design work."**
   Unresolved composition question, flagged rather than answered. The managed
   block's workflow map prescribes `grill-requirements` → `plan-change` →
   `design-architecture` for that same phase and does not mention
   brainstorming. They plausibly compose (brainstorm to find the shape, then
   grill it into REQ items), but nothing here decides that, and the first
   Phase 3 change under guardrails will have to.
6. **"Don't use `sed` on Rust test files."** No conflict; still binding.

## Safety class

**Class B** — see `docs/adr/2026-08-26-safety-class-b.md`. Written into
`.guardrails/config.yaml` as `safety_class: B`.

## Configuration chosen

```
safety_class: B
strict_paths:  org-members/src, on-chain-client/src
test_paths:    org-members/tests, on-chain-client/tests
doc_verification: docs/verification
verify_commands:
  cargo test -p org-members
  cargo test --manifest-path on-chain-client/Cargo.toml --lib \
      --test fuzz_decode_org_state --test fuzz_parse_revive_event \
      --test fuzz_event_round_trip
  quint typecheck quint/membership.qnt
  quint typecheck quint/protocol.qnt
problem_age_days: 30
problem_open_max: 10
coverage_command: NOT SET
```

`strict_paths` deliberately excludes `org-node`, `app`, `on-chain`, and the
three `spike-*` crates: untraced code is grandfathered, and each area comes
under discipline as its own tooth. `coverage_command` is unset even though
Class B targets statement coverage — `cargo-llvm-cov` is not installed on this
machine (`cargo llvm-cov --version` → "no such command"). That is a gap, not a
decision; it is tooth 3, and it is recorded here because the guardrails block
requires a project without a coverage command to say why.

### What `verify_commands` does and does not cover

The two `cargo test` entries are what runs at every merge. Precisely:

- **The MBT conformance test is inside the gate, not exempt from it.**
  `org-members/tests/mbt_conformance.rs` is a test target of `org-members`, so
  `cargo test -p org-members` runs it. Its behaviour depends on the machine:
  with no `quint` on `PATH` it skips at runtime (by design — see the test's
  own header); with `quint` present but unable to install its rust evaluator
  it **fails**, which is what happens under the agent sandbox that installed
  this change (`EPERM: mkdir '~/.quint/rust-evaluator-v0.6.0'`). So this entry
  is green on a provisioned machine and red in a sandboxed session, and the
  provisioning item in the setup checklist is what closes that.
- **`--lib` on `on-chain-client` drops its integration targets**, all of which
  spawn a chopsticks or anvil fork and need `on-chain/scripts/node_modules`:
  `00_chopsticks_sanity`, `01_multisig_sanity`, `off_chain_genesis_ceremony`,
  `p_address_is_orgid`, `regenerate_corpus`, `reorg_cancels_proposed`,
  `scenario_a_full`, `smoldot_smoke`, `two_orgs_one_watcher`. The three fuzz
  targets are named back in explicitly (see conflict 4). Those nine targets
  are a real hole in the merge gate until provisioning is scripted.
- **Left to CI on purpose:** the quint invariant simulations and the Apalache
  bounded verify, which run for minutes. CI already runs them.

## Adoption order

Each tooth is its own worktree change. A tooth only ever tightens.

| # | Tooth | Why this order |
|---|---|---|
| 1 | **This change.** Install `.guardrails/` — `config.yaml`, the seven v0.4.0 scripts, and `templates/verification.md` (the record template, which must live there and never among the records). Append the managed `AGENTS.md` block; add `CLAUDE.md`; create the four ledgers with their READMEs, plus `docs/adr/`, `docs/verification/` and the skeleton `docs/CONTEXT.md`; extend `.gitignore`; record the safety class as an ADR. No existing doc migrated. | The machinery has to exist before anything can be checked against it. |
| 2 | **Signing.** ~~Later tooth~~ — **reclassified 2026-09-01 as a prerequisite, not a tooth.** From guardrails 0.5.1 every merge ends with `finish-merge.sh`, which runs `check-signing.sh --strict` before removing the worktree and deleting the branch, so signing is enforced from the first merge rather than adopted on a schedule. The shared-config override was removed on 2026-08-26; what remains is tracked in the setup checklist, which the toolkit now treats as a gate on the ratchet itself. | The upgrade moved this out of the adoption order. A tooth is something you schedule; this is something the next merge demands. |
| 3 | **Rust CI + coverage.** Add a general Rust job (`cargo test` over the workspace, `cargo clippy`) alongside the existing `quint`/`apalache`/`mbt` jobs; install `cargo-llvm-cov` + `llvm-tools-preview` and set `coverage_command` to reach the Class B statement-coverage target. | Class B's verification rigour is not met until coverage is measured, and today's only Rust CI job runs one test target in one crate. |
| 4 | **Requirements migration.** Run `grill-requirements` over `org-members` first: the crate's "Critical invariants (verified by tests)" and public-API table are already requirement-shaped, and its tests already exist to carry `verifies:` annotations. Land as the ledger's first dated file. | Smallest, best-specified, already-tested area. Establishes the ledger's grain before the harder crates. |
| 5 | **Risk analysis.** Run `analyze-risks`: fill the acceptability matrix in `docs/risk/README.md` (it ships as `TBD` and is a quality-manual decision, not an agent's), then enumerate hazards — including the injury pathway the safety-class ADR asserts but does not yet analyse. The quint invariants (fork safety, revocation safety, `revokedExcludedFromOrgSecret`, τ-window, convergence) are the natural first hazard/control candidates: each is already a mechanised argument looking for a risk control to be traced to. | Needs the matrix decision, and the ADR leaves an explicit open item that only this tooth can close. |
| 6 | **Architecture + SOUP.** Run `design-architecture` for SDD items over the crate decomposition, and populate `docs/architecture/soup.md`. **First problem to solve: the root `Cargo.lock` is gitignored**, and only `org-members/Cargo.lock` and `on-chain-client/Cargo.lock` are tracked — neither of which contains `iroh`, `swarm-discovery` or `hickory-*`. IEC 62304 §8.1.2 wants exact versions from a controlled configuration item, so either the root lockfile becomes tracked or the SOUP table carries the versions itself. The interesting rows are the pinned and pre-release dependencies (iroh 0.98.2, subxt 0.50.1, Keyhive, p2panda, and the `-rc`/`-beta`/`-alpha` pins). | SOUP review is where the pre-release pins stop being tribal knowledge — but not from a file git does not track. |
| 7 | **Widen `strict_paths`.** Add `org-node/src`, then the Phase 3 `org-acl` crate as it is created (born under discipline, never grandfathered), then `app`. | Follows the work: Phase 3 is next, and new code should never need a retrofit. |
| 8 | **Problem-log backfill.** Move the live "Known follow-ups" list in `org-members/AGENTS.md` and any open items from the `fix(...)` commit history into `docs/problems/` as PR items with `opened:`/`status:`. (`owner:` was
      in this instruction until 2026-09-01; guardrails 0.5.1 removed the field —
      `git blame` answers authorship, and problems are not personally owned.) Re-check `problem_age_days: 30` / `problem_open_max: 10` against the backlog this produces before it starts failing merges. | Deliberately late: these two limits fail a merge once exceeded, so the backfill and the limits must be sized together. |
| 9 | **Glossary + context.** Fill `docs/CONTEXT.md` from the domain vocabulary already written in `org-members/AGENTS.md` (Organisation, Member, Handle, MemberId, P2pMemberKey, …, and its "do NOT introduce" list, which is exactly the `_Avoid_` field). | Pure migration of text that already exists and is already good; no gate depends on it. |

## Notes on this change's own contents

- **`docs/verification/` becomes a tracked directory only via its first
  record.** Git cannot commit an empty directory, so `mkdir` alone leaves it
  present for the author and absent from the commit — and `doc_verification`
  pointing at a directory that does not exist is exit 2 for `check-review.sh`
  on a fresh clone. This change's own verification record,
  `docs/verification/2026-08-26-worktree-guardrails-ratchet.md`, is what makes
  the directory real in the repository. The same is true of `docs/adr/`, which
  the safety-class ADR populates.
- **`.gitignore`: the substantive addition is `*.bak`.** The `.worktrees/`
  entry is the guardrails default location and is inert in this project, which
  uses `.claude/worktrees/` — already ignored before this change. It is kept so
  that a worktree created at the toolkit's default path is also covered.
- **Two files this change adds that are easy to miss:** `docs/CONTEXT.md` (the
  glossary skeleton, filled at tooth 9) and `.guardrails/templates/`
  `verification.md` (the record template, read by `merge-change` step 6b).

## Not in scope for this change

Migrating any existing document; annotating any existing test with
`verifies:`; touching `org-node`, `app`, `on-chain`, or the `spike-*` crates;
changing CI; enabling signing; filling the risk acceptability matrix.
