# Verification — Rust CI and the class B coverage gate (2026-08-26)

branch: worktree-ci-coverage
reviewer: fresh general-purpose subagent, dispatched at merge-change step 6a with only the diff, the acceptance criteria in the repository, and the plan documents — no implementation narrative and no session history
verdict: NOT FIT TO MERGE, 25 findings. Four would have landed CI red on the first push, one committed comment was wrong by a factor of six, and two went to whether the class B coverage obligation was actually discharged. All 25 are dispositioned below; the blocking ones are fixed and re-verified, and the one that is not mine to fix — the coverage shortfall — is accepted by the project owner in writing here.
reproduced: yes, every blocking finding, before fixing it. `cargo test -p org-members --test mbt_conformance` with quint off PATH → `Failed to execute Quint command`, FAILED (the review's central claim, and the opposite of what the test's own header comment says). `check-signing.sh master..HEAD` → `UNSIGNED 1b5cc04`, exit 1. `cargo clippy --manifest-path on-chain-client/Cargo.toml --all-targets --keep-going -- -D warnings` → 144 lint errors, not the 25 I had written. `grep -rn coverage_command .guardrails/scripts/` → one hit, in lib.sh's schema key list, confirming no script enforces it. I did not take the reviewer's word for any of these.

Change: ratchet tooth 3 plus the CI half of tooth 2 — a Rust and guardrails CI
workflow, a `make coverage` target with per-crate floors, and
`coverage_command` set so the class B coverage target has a command behind it.
Branched from `master` at `a120f14`.
Plan: `docs/plans/2026-08-26-ratchet-gap-analysis.md`, tooth 3 (and tooth 2's
CI item).

## The gate

Re-run on the corrected tree (`bb10a9d`), after the review fixes.

| Gate | Result |
| --- | --- |
| `cargo test -p org-members` | Attested by the project owner in their own unsandboxed terminal: all tests passed. Not captured by me — under the agent sandbox quint cannot create `~/.quint/rust-evaluator-v0.6.0` (EPERM) and `mbt_conformance` fails for that reason alone. |
| `cargo test --manifest-path on-chain-client/Cargo.toml --lib --test fuzz_decode_org_state --test fuzz_parse_revive_event --test fuzz_event_round_trip` | 23 passed, 0 failed; three fuzz corpora ran ~1s each with no crashes. |
| `quint typecheck quint/membership.qnt` | exit 0 |
| `quint typecheck quint/protocol.qnt` | exit 0 |
| `make coverage` | exit 0. org-members 93.50% lines / 92.20% regions (floors 92/91); on-chain-client 53.57% / 59.19% (floors 52/58). |
| `check-ids.sh --allow-draft-files` | exit 0 |
| `check-trace.sh` | exit 0 — `REQ 0, HAZ 0, RC 0, SDD 0, LLR 0, PR 0`; `strict 2, tests 2` |
| `check-review.sh` | Could not run — exit 2 on macOS awk, the toolkit portability defect recorded in the tooth 1 record. Hand-checked instead, as there: `branch:` once at column one matching whole-value, all three required fields with values, one column-one `disposition:` per `**finding-N**:` block, no bold `**disposition:**`. |
| Coverage, against the class target | org-members MEETS it. on-chain-client does NOT — see "Accepted shortfall" below. |
| Working tree | clean |

The `.github/workflows/rust.yml` jobs themselves have not been observed
running: this repository has no CI run for them yet, and every command they
contain was instead executed locally. The first push is where that claim is
actually tested, and finding 24 below says what could still differ there.

## What was wrong, and what was built

Before this change, the only CI was `quint.yml`: the quint models, the
Apalache bounded verify, and one Rust lane running a single test target in one
crate. Nothing built the workspace, nothing linted it, nothing checked the
guardrails gates, and the class B statement-coverage target was stated in the
ADR and measured nowhere.

What was built: five CI jobs whose two `cargo` entries are `verify_commands`
verbatim, so CI and the local merge gate cannot drift into disagreeing about
what passing means; a `make coverage` target enforcing line AND region floors
per crate; and `coverage_command` pointing at it.

The more useful part of this change is what the review forced it to stop
claiming. Three assertions I had committed were false, and each was the kind
that reads as reassurance: that the MBT lane skips harmlessly without quint
(it fails), that `--all-targets` clippy failures were 25 `expect_used` errors
all covered by the project's "unwrap is fine in tests" rule (144 errors, three
of them `panic`, plus one ordinary lint in org-members' own tests that the
rule does not excuse), and that setting `coverage_command` means coverage is
"enforced at every merge" (no check script reads it; CI enforces it, locally
it is discipline).

## Review

**finding-1**: The `test` job would be red on every push. `mbt_conformance` does not skip when quint is absent — it panics with `Failed to execute Quint command`. The workflow comment asserting otherwise inherited the claim from the test file's own header without checking the code.
disposition: Reproduced with quint off PATH before fixing. The job now installs quint (`actions/setup-node` + `npm i -g @informalsystems/quint`), matching what quint.yml's `mbt` job already did, so the lane actually executes rather than being nominally present. The comment now states the measured behaviour and flags that quint.yml's `mbt` job is thereby redundant — left alone because this change does not own that file.

**finding-2**: `check-signing.sh` on the pull-request range covers exactly the worktree commits that this project's rules make deliberately unsigned, so it would fail every change for obeying the rules.
disposition: Reproduced (`master..HEAD` → `UNSIGNED`, exit 1). Removed from the pull-request path entirely.

**finding-3**: The signature gate was scoped backwards — running only on pull requests and never on push, so it could never see the squash commit on `master`, the one commit non-negotiable 2 actually requires to be signed.
disposition: Inverted. It now runs on `push` to `master` only, with no range argument, so it checks HEAD — precisely the squash commit. This is the finding that mattered most: as written, CI could never have caught the failure the gap analysis calls the largest single gap.

**finding-4**: `check-ids.sh` ran without `--allow-draft-files` on the pull-request path, which would redden every PR that adds a ledger item, since draft files are legitimate until `finalize-docs.sh` renames them at merge.
disposition: Split into two steps by event — `--allow-draft-files` on pull requests, plain on push, where a surviving draft file is a real defect. The comment explains why the same command answers different questions in the two places.

**finding-5**: Everything else in the workflow passes on this tree (clippy ×2, the three no-std checks, both coverage targets, check-ids, check-trace), verified by the reviewer independently.
disposition: Nothing to change. Recorded because it is the evidence that findings 1-4 were the complete set of red-on-first-push defects, not a sample.

**finding-6**: The `fetch-depth: 0` justification was half wrong — `check-ids.sh` has no base gate and needs no history.
disposition: Corrected to name the real reason: the pull-request steps resolve `origin/<base>` and ask which files the branch touched.

**finding-7**: The workflow header said the conformance lane "is not duplicated here" while line 27 ran it.
disposition: Header rewritten. The duplication is now stated plainly, along with the fact that it makes quint.yml's `mbt` job redundant.

**finding-8**: The workflow asserted it "deliberately mirrors verify_commands rather than inventing a second, wider test contract" — and then added two contracts (clippy, no-std) that exist only in CI, so a change can pass locally and redden `master`.
disposition: The claim is withdrawn rather than the jobs removed. The header now says the two `cargo` jobs run `verify_commands` verbatim and that clippy and no-std are additional CI-only contracts, with the cost noted on each. Not fixed by widening `verify_commands`, which would put a wasm32 target install and a clippy pass into every local merge; that is a real trade and it should be made deliberately, not smuggled in under a slogan. Recorded here as an open divergence.

**finding-9**: The two "identical" `cargo test -p org-members` entries do not verify the same thing, because the local machine has quint and the runner did not — so CI was not weaker but permanently red.
disposition: Dissolved by finding-1's fix: the runner now has quint, so the two run the same lane in the same shape.

**finding-10**: CI covers 2 of 7 crates; the gap analysis specified tooth 3 as "cargo test over the workspace", and the checklist was ticked without recording the narrowing.
disposition: Accepted as a deliberate narrowing and now disclosed. CI covers exactly what `strict_paths` and `verify_commands` cover, which is the contract this project has chosen; `org-node`, `app/src-tauri` and the three spike crates come in at tooth 7 with their trace discipline, not before. Recorded in Gaps.

**finding-11**: The `mbt_conformance` omission from the coverage measurement is disclosed and justified, but "omitting it understates coverage, which is the safe direction" is misleading, since the floors were calibrated against the reduced measurement; and "CI's mbt job keeps that lane honest" conflates running the test with measuring its coverage.
disposition: Both corrected in the Makefile. It now says the gate is conservative relative to the crate's true coverage but NOT relative to its own floor, and that neither CI job measures the lane's coverage contribution, which remains unknown.

**finding-12**: The floors were fragile — 6 uncovered lines would break org-members, 5 would break on-chain-client — creating pressure to lower a floor the Makefile forbids lowering.
disposition: Floors moved one point below the measurement (org-members 92/91, on-chain-client 52/58), with the measured values recorded alongside so drift stays visible. The slack is argued from the counted lines, not chosen by feel.

**finding-13**: A per-crate floor is still an average: coverage moving between files inside a crate is invisible, and `decode/mod.rs` sits at 0.00% inside `strict_paths`.
disposition: Documented in the Makefile, including the specific scenario (a collapse in `decode/v_paseo_ah.rs` absorbed by a rise in `client.rs`). Not fixed: `--fail-under-file-lines` is the mechanism, and it cannot be enabled while a file sits at 0.00%. Raising `decode/mod.rs` off zero is what unlocks it.

**finding-14**: Line coverage is not statement coverage, and the change gated on the weaker of the two metrics available, silently. Region coverage is the closer analogue and the table prints both.
disposition: Both metrics are now gated (`--fail-under-lines` and `--fail-under-regions`), with the divergence recorded — org-members 93.50/92.20, on-chain-client 53.57/59.19, differing in opposite directions between the crates. This was the substitution most needing statement for a class B project, and it was the one thing left unstated.

**finding-15**: "The class B statement-coverage target is now enforced at every merge" was false: no check script reads `coverage_command`; the only consumer is an instruction to an agent at verify-before-merge step 5.
disposition: Verified independently (`grep -rn coverage_command .guardrails/scripts/` → one hit, lib.sh's schema list) and corrected in all three places that carried the claim — the config comment, the Makefile header and the checklist. Each now says the CI job is the mechanical half and that locally it is discipline.

**finding-16**: on-chain-client's floor encodes a class B shortfall as a pass condition, without the explicit acceptance AGENTS.md requires.
disposition: The acceptance is recorded below, given by the project owner, which is whose it is to give. The Makefile now labels that floor a shortfall rather than a target met.

**finding-17**: "25 expect_used errors, all of them expect_used" was wrong by roughly six times and wrong in kind — 144 errors across 13 targets, three of them `clippy::panic`, four inside a lib test module; and `org-members --all-targets` also fails, on one ordinary `manual_contains` lint that the "unwrap is fine in tests" rule does not excuse. The stated mechanism was also wrong: org-members has no `[lints]` section and denies via `#![deny(...)]` in lib.rs.
disposition: All re-measured myself and corrected in the workflow comment, which now separates the principled reason for `--lib` from the concession, names the org-members lint as something that should simply be fixed, and states where each crate's deny rules actually come from. The original number came from reading the last line of a truncated run — the error is mine and the correction is measured, not argued.

**finding-18**: The signing checklist item's description is accurate; no defect.
disposition: Nothing to change.

**finding-19**: `check-review.sh` is correctly excluded from the push path and the macOS claim is true — but the surrounding claim that "CI is the only place it executes" holds only if the project uses pull requests, and this project merges locally without them, in which case the gate runs nowhere.
disposition: Disclosed in both the workflow comment and the checklist item, in those terms: today the class B independent-review evidence gate has no mechanical enforcement anywhere, and either adopting PRs or landing the upstream awk fix is what closes it.

**finding-20**: "Three items closed, eight remain open" is accurate.
disposition: Nothing to change; re-counted after the rewrite (3 and 8).

**finding-21**: "The CI half of tooth 2" overstated what landed — tooth 2 specifies `check-signing --strict` on every merge, and what shipped was non-strict and pull-request-only, i.e. on no merge at all.
disposition: Half fixed by finding-3 (it now runs on every push to `master`, which is every merge). Still non-strict, which stays disclosed in the workflow and depends on the signature-verification checklist item; the claim is no longer that tooth 2's CI half is complete.

**finding-22**: No document accounts for a root `Makefile`, and `org-members/AGENTS.md`'s "Build / test commands" section will not tell a reader that `make coverage` exists or is a merge prerequisite. `Makefile:7` also forward-references a `make provision` target that does not exist.
disposition: Accepted as a real documentation gap, recorded in Gaps rather than fixed here — updating `org-members/AGENTS.md` is a change to a crate this tooth does not otherwise touch, and the forward reference to `make provision` is deliberate signposting toward the provisioning checklist item. Both belong with that item.

**finding-23**: No contradiction of org-members' build-matrix or testing rules; the no-std matrix is run verbatim, the fuzz preference is honoured, and no Rust file is touched.
disposition: Nothing to change.

**finding-24**: Both new gates are hostage to uncontrolled configuration — the root `Cargo.lock` is gitignored and `dtolnay/rust-toolchain@stable` floats, so a new lint or a dependency bump can redden `master` with no repository change; and the floors were calibrated on aarch64-darwin but are enforced on x86_64-linux.
disposition: Mitigated, not solved. Finding-12's one-point slack absorbs small mapping differences, and the coverage job now carries a comment telling whoever sees a confusing failure to check the platform difference first. The underlying fix is tooth 6's lockfile decision; this change does not pull it forward. Recorded in Gaps.

**finding-25**: `verifies:` annotations and robustness cases have no subject — four files, no Rust source, no tests — and the reviewer recorded that as an explicit non-finding rather than manufacturing one.
disposition: Confirmed. Nothing to annotate.

## Accepted shortfall (class B coverage)

`AGENTS.md` requires that a coverage shortfall against the class target be
either fixed or explicitly accepted by the user and documented here. The
project owner was asked, with the alternatives (reorder the plan to provision
the chopsticks lane first, or drop the crate from the gate) and chose to
accept:

> **on-chain-client at 53.57% lines / 59.19% regions is accepted as a known
> class B shortfall, as at 2026-08-26.**

The cause is specific and is not thin testing: `client.rs` (353 of the crate's
715 lines) sits at 12.75% and `decode/mod.rs` at 0.00%, because both are
reached almost entirely through the chopsticks/anvil integration tests, which
need `on-chain/scripts/node_modules` and are excluded from `verify_commands`
by the `--lib` flag. The nine excluded test targets and this coverage figure
are the same hole seen from two directions.

What retires it: the dev-environment provisioning item in
`docs/plans/2026-08-26-ratchet-setup.md`. When those targets can run, they
return to `verify_commands` and the floor is re-measured upward. Until then
the floor catches regressions without pretending the target is met, and the
alternative considered and rejected — dropping the crate from the gate — would
have removed the regression check as well as the pretence.

org-members meets the class target on both metrics and is not covered by this
acceptance.

## Gaps

- **No CI run has been observed.** Every command in the workflow was verified
  locally; the jobs themselves have never executed. Finding 24 names what can
  still differ on the runner — a floating toolchain, an untracked lockfile,
  and floors calibrated on a different architecture.
- **`verify_commands` and CI now hold different contracts** (finding 8):
  clippy and the no-std matrix are CI-only, so a local merge can pass and
  `master` can redden. Widening `verify_commands` would put a wasm32 target
  install into every local merge; that trade is deliberately not made here.
- **CI covers 2 of 7 crates** (finding 10). `org-node`, `app/src-tauri` and
  the three spike crates have no build, test or lint anywhere.
- **The independent-review gate runs nowhere** (finding 19): `check-review.sh`
  is macOS-broken locally and pull-request-only in CI, and this project merges
  without pull requests. This record's own conformance to the schema is
  hand-checked, as the tooth 1 record's was.
- **Coverage of the MBT lane is unknown** (finding 11). It runs in two CI jobs;
  neither measures what it covers.
- **File-level coverage is unguarded** (finding 13), blocked by
  `decode/mod.rs` at 0.00%.
- **`make coverage` is undocumented where a reader would look** (finding 22):
  `org-members/AGENTS.md`'s build-commands section does not mention it, and
  `make provision` is referenced before it exists.
- **`check-signing` is non-strict** (finding 21), so a signature that cannot
  be verified still passes. Closing that is the signature-verification
  checklist item, not this change.
