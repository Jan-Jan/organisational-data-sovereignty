# Verification — guardrails ratchet tooth 1 (2026-08-26)

branch: worktree-guardrails-ratchet
reviewer: fresh general-purpose subagent, dispatched at merge-change step 6a with only the diff, the two plan documents, the ADR, and the toolkit's own acceptance criteria — no implementation narrative and no session history
verdict: fit to merge with four conditions (a) commit docs/verification/ so the install is reproducible, (b) correct four false or self-contradictory factual claims in the plan documents, (c) restate the A/B class boundary or name an injury pathway, (d) disclose two unlisted files and two unflagged pre-existing rules. Thirteen findings, all dispositioned below; every condition met.
reproduced: n/a for a defect — this change installs machinery and fixes no bug. What WAS reproduced: each of the reviewer's factual findings was re-verified independently against the repository before being accepted (config scopes, tracked lockfiles, crate count, CI job contents, fuzz target declarations), and the one fixable gap (finding 5.2) was reproduced as a passing command before and after the fix. The reviewer's own gate runs are quoted in its report; the runs below are mine, on the tree being merged.

Change: install guardrails 0.4.0 into an existing 75-commit project as retrofit
tooth 1 — machinery, ledger skeletons, safety class, adoption order — with no
existing document migrated. Branched from `master` at `d6de5f6`.
Plan: `docs/plans/2026-08-26-ratchet-gap-analysis.md` (this change is its tooth 1).

## The gate

Every figure below is from the tree under merge (`d881c67`), re-run after the
review fixes.

| Gate | Result |
| --- | --- |
| `cargo test -p org-members` | Attested by the project owner in their own unsandboxed terminal: no errors. Not captured by me — under the agent sandbox this command fails in `mbt_conformance` with `EPERM: mkdir '~/.quint/rust-evaluator-v0.6.0'`, quint being unable to install its rust evaluator. My own partial run: 101 passed in `integration_test` + `fuzz_tests`, `mbt_conformance` red for that reason alone. No Rust, test or quint file was touched by this change or by the review fixes, so the owner's result stands for the merged tree. |
| `cargo test --manifest-path on-chain-client/Cargo.toml --lib --test fuzz_decode_org_state --test fuzz_parse_revive_event --test fuzz_event_round_trip` | 23 passed, 0 failed. Three fuzz targets ran their corpora ~1s each: `fuzz_decode_org_state` 260564 it/s (4 corpus inputs), `fuzz_event_round_trip` 41568 it/s, `fuzz_parse_revive_event` 257363 it/s (6 corpus inputs). No crashes. |
| `quint typecheck quint/membership.qnt` | exit 0 |
| `quint typecheck quint/protocol.qnt` | exit 0 |
| `check-ids.sh` | exit 0, no output. Also clean under `--allow-draft-files`. |
| `check-trace.sh` | exit 0 — `checked: REQ 0, HAZ 0, RC 0, SDD 0, LLR 0, PR 0`; `problems: open 0, oldest n/a; limits age 30, open 10`; `sources: srs 1, rmf 1, sad 2, soup 1, problems 1; strict 2, tests 2` |
| `finalize-docs.sh --dry-run` | exit 0, no renames — this change files no draft ledger item |
| `check-review.sh` | **Could not run — exit 2 before reading any record.** A toolkit portability defect, not a defect in this record: `check-review.sh:255` interpolates the newline-separated `GR_RECORD_FIELDS` into an `awk -v` assignment, and macOS's awk (BWK, `version 20200816`) rejects a literal newline there (`awk: newline in string …`, exit 2). gawk, mawk and busybox awk accept it, which is how it shipped. Minimal repro and an upstream fix prompt were produced; the vendored copy was deliberately NOT patched, to keep the scripts identical to the qualified v0.4.0. See Gaps. |
| Coverage, against the class target | **Not measured.** Class B targets statement coverage; `coverage_command` is unset because `cargo-llvm-cov` is not installed (`cargo llvm-cov --version` → "no such command"). Recorded as a known, accepted gap, closed by tooth 3. See Gaps. |
| Working tree | clean |

## What was wrong, and what was built

Nothing was wrong: this is a bootstrap, not a fix. The project had 75 commits,
seven Rust crates, formal quint models with CI-enforced invariants, proptest and
libfuzzer suites, and 20 design specs — and no requirement, hazard, design or
problem item with an identity, no risk file, no SOUP inventory, no archived
verification evidence, and no signed commit on `master`.

What was built is the machinery that makes those absences visible and
mechanically checkable from the next change on: `.guardrails/` (config plus the
seven v0.4.0 scripts and the record template), the managed block appended to
`AGENTS.md` (86 insertions, zero deletions — every pre-existing rule intact),
the four ledger directories with their grammar READMEs, `docs/adr/` and
`docs/verification/`, a `CLAUDE.md` pointing at `AGENTS.md`, and two `.gitignore`
entries.

Three decisions are the substance rather than the scaffolding, and each is
written down where it can be argued with:

- **Class B** (`docs/adr/2026-08-26-safety-class-b.md`), on an injury pathway
  the owner named and the ADR marks as asserted-not-yet-analysed.
- **What is grandfathered**: `strict_paths` covers `org-members/src` and
  `on-chain-client/src` only; five other crates stay outside trace discipline
  until their own tooth.
- **A nine-tooth adoption order**, each tooth its own change, tightening only.

## Review

**finding-1**: `docs/verification/` is empty and not in the commit at all — git cannot commit an empty directory, so `doc_verification` points at a path absent from a fresh clone, where `check-review.sh` dies at exit 2. Both the commit message and the gap analysis claim the change creates it.
disposition: This record is the file that makes the directory real; it is committed with the change. The gap analysis now states the mechanism explicitly ("Notes on this change's own contents") rather than claiming a `mkdir` reached the repository. Reproduced before the fix: the reviewer's own `check-review.sh` run returned exit 2 with "no verification records in … 'docs/verification'".

**finding-2**: The verification-record template's location was checked against the toolkit rule that it must not be filed among the records. No defect — it landed at `.guardrails/templates/verification.md`.
disposition: Nothing to change. Recorded because a passing check on the toolkit's own most-warned-about foot-gun is worth having in the evidence trail.

**finding-3**: No column-one item-definition form (`**REQ-…**:` etc.) exists anywhere in the `doc_*` directories or the templates; every illustrative form is indented. No defect.
disposition: Nothing to change. This is what keeps `MALFORMED-ID` and `MISPLACED-ITEM` quiet on a ledger that as yet defines no items.

**finding-4**: "There is currently no Rust job in CI" is false — `.github/workflows/quint.yml` has an `mbt` job that installs `dtolnay/rust-toolchain@stable` and runs `cargo test --test mbt_conformance`. The gap analysis contradicted itself in a single table cell, and this false premise underpinned tooth 3.
disposition: Corrected in both documents. The CI row now describes all three jobs and states the defensible claim: a Rust job exists, a *general* build/test/clippy job does not. Tooth 3 is rewritten on that premise, noting that today's Rust CI runs one test target in one crate.

**finding-5**: The claim that the MBT conformance lane is "left in CI rather than in `verify_commands`" is contradicted by its own footnote and by the code: `mbt_conformance` is a test target of `org-members`, so `cargo test -p org-members` runs it. The claim that a machine without quint "sees that command fail" is contradicted by the test's own header, which says it skips at runtime when quint is absent from `PATH`.
disposition: The whole rationale section is rewritten. It now says the MBT test is inside the gate, skips when `quint` is absent from `PATH`, and fails when `quint` is present but cannot install its evaluator — which is the sandbox case I actually observed, and is why the org-members row above rests on the owner's run rather than mine. Both halves of the original claim were wrong and neither survives.

**finding-6**: The signing diagnosis omitted where `commit.gpgsign=false` actually lives. It is in the shared repository config, overriding `commit.gpgsign=true` in `~/.gitconfig` — and writing it there is precisely what the project's own pre-existing rule forbids doing from a worktree.
disposition: Verified independently (`git config --list --show-origin` shows all three scopes) and corrected in both documents. Tooth 2 is re-scoped from "configure signing" to "remove one line from the shared config", and the setup checklist's first item now says so in its title. This was the most consequential finding: it turned a setup project into a one-line fix and surfaced a standing violation of the project's own rule.

**finding-7**: "Six Rust crates" is wrong; there are seven (`app/src-tauri` was missing), and the document's own exclusion list implied eight areas.
disposition: Corrected to seven and all seven are now named in the opening paragraph. The `tests/` row now includes `app/src-tauri`.

**finding-8**: The SOUP pins cited as "visible in the lockfile" come from the root `Cargo.lock`, which is gitignored; only `org-members/Cargo.lock` and `on-chain-client/Cargo.lock` are tracked, and neither contains `iroh`, `swarm-discovery` or `hickory-*`. IEC 62304 §8.1.2 wants exact versions from a controlled configuration item.
disposition: Verified (`git ls-files | grep Cargo.lock` returns the two tracked files) and corrected. Tooth 6 now opens with this as its first problem to solve — either track the root lockfile or have the SOUP table carry the versions itself — and the SOUP row in the inventory table carries the caveat.

**finding-9**: "Every one of the last 20 commits landed there directly" is a process claim the evidence cannot support: a squash merge and a direct commit are indistinguishable in a linear history.
disposition: Softened to what the record actually shows — that `master` is linear and its commits are all unsigned — with the process claim explicitly withdrawn.

**finding-10**: The ADR rejects Class A on the wrong criterion. Class A is "no injury possible", not "no hazardous situation possible", and the ADR's own harm analysis (confidentiality/availability, "a person is affected by the resulting decision rather than by the software directly") argues its way to A before asserting B.
disposition: The ADR now carries a "class boundary, stated correctly" section separating the three classes by harm, and interview step 2 names the injury pathway the class rests on: this layer gates data that feeds a care or safety decision, so a wrongly locked-out member or a revoked member retaining access can contribute to non-serious injury through a human decision. It is marked **asserted, not yet analysed**, mapped to S2 on the shipped severity scale, and tooth 5 is named as where it survives or is revised — with revision down to A stated as a legitimate outcome. The Alternatives section no longer misstates the A criterion.

**finding-11**: "`/ratchet`'s rule is that the higher class governs when in doubt" is unverifiable from the repository.
disposition: Now attributed explicitly to the `/ratchet` skill rather than presented as an in-repo rule, and the sentence makes clear it did not drive this decision: there is no doubt at the B/C boundary, and the doubt at A/B is resolved by the owner's answer, not by a tie-break.

**finding-12**: `docs/CONTEXT.md` and `.guardrails/templates/verification.md` are added by the change but enumerated in neither the commit message nor tooth 1's contents list.
disposition: Both are now named in tooth 1's row and again under "Notes on this change's own contents".

**finding-13**: Two pre-existing `AGENTS.md` rules were absent from the conflicts section. "Always include fuzz testing" is in real tension with `--lib` on `on-chain-client`, which drops its fuzz targets from the merge gate; and "brainstorming before non-trivial design work" overlaps the managed block's workflow map with no stated composition.
disposition: The fuzz tension is **fixed, not merely disclosed**: the three libfuzzer targets (`[[test]]`, `harness = false`) are named explicitly in `verify_commands`, and all three now run their corpora at every merge — the run is in the gate table above. Investigated first whether the dropped integration targets could come back too; they cannot (`p_address_is_orgid` fails with `ChopsticksBinaryMissing`), so the nine chopsticks-dependent targets are now named as a known hole with provisioning as the fix. The brainstorming overlap is added as conflict 5, flagged as an open composition question for the first Phase 3 change rather than answered here.

## Gaps

What this change did NOT establish:

- **Statement coverage is unmeasured**, and Class B requires it. `cargo-llvm-cov`
  is absent, `coverage_command` is unset. Accepted as a documented gap for this
  change on the grounds that tooth 1 installs no code to cover; it is tooth 3,
  and until it lands the class target is stated and unmet.
- **Nine `on-chain-client` integration targets are outside the merge gate**
  (`--lib`), because each spawns a chopsticks or anvil fork needing
  `on-chain/scripts/node_modules`. This is the largest hole in `verify_commands`
  and is bounded only by a provisioning script that does not exist yet.
- **The `org-members` gate row is the owner's word, not my captured output.** The
  sandbox cannot run it. That is a weaker form of evidence than every other row
  here, and it is marked as such rather than dressed up.
- **No item of any kind exists yet.** `check-trace.sh` reports zero REQ, HAZ, RC,
  SDD, LLR and PR, so every traceability gate passed over an empty ledger. A
  green run here proves the machinery is wired, not that anything is traced.
- **The injury pathway behind Class B is asserted, not analysed.** No HAZ item,
  no severity, no probability. Tooth 5.
- **The guardrails bats suite was not run** as the tool-qualification basis for
  v0.4.0. Recorded as an open checklist item.
- **`master` is still unsigned and unprotected.** This change installs the rule
  that forbids that; it does not enforce it. Tooth 2.
- **Step 6c never executed, and this record is the first thing it would have
  read.** The gate that checks this record is unrunnable on macOS (see the gate
  table). What stands in its place is a hand check against the documented field
  grammar: `branch: worktree-guardrails-ratchet` present once at column one and
  matching whole-value; `reviewer:`, `verdict:` and `reproduced:` each at column
  one with a non-empty value; 13 `**finding-N**:` headers and 13 column-one
  `disposition:` lines, one per block, with no bold `**disposition:**` (which
  would close the finding it belongs to); and no other line in the file opening
  with `**`. That is presence-checking of the same shape the gate performs, done
  by a person's tooling rather than the tool, and it is weaker evidence for
  exactly the reason the toolkit gives for having the gate at all: it is the
  author checking their own artefact. The remedy is to re-run the gate over this
  record once the upstream fix lands, and to treat a failure then as this
  record's defect rather than a new one.
