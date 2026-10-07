# Verification — decision (branch) coverage targets (2026-10-06)

branch: worktree-guardrails-branch-coverage
reviewer: independent review subagent (fresh context, given only the diff, the plan and the review checklist; it ran `make coverage`, `make coverage-branch` and the failing-floor paths itself in its own review worktree), 2026-10-06
verdict: pass. The gate works: a drop below a branch floor fails, an instrumentation with no branches fails, and the stable floors fail exactly as before. Four low code findings and one record finding, all fixed in this change. This was the last review round, because every code finding was low.
reproduced: not applicable — this change adds a measurement, it fixes no defect. The class C gap it closes (decision coverage unmeasured, `docs/adr/2026-09-01-safety-class-c.md` open item 1) was shown directly: before this change no Makefile target or CI job passed `--branch`, and stable cargo-llvm-cov reports no branches. The new floor was shown to fail: `make coverage-branch-person PERSON_BRANCHES=100.5` exits 2 with "below its floor".

Change: decision coverage is measured and enforced for org-members, on-chain-client and person: `make coverage-branch` on the pinned `nightly-2026-10-03`, a branch floor per unit, a `coverage-branch` CI job that checks its pin against the Makefile's, and stable and nightly recipes that share one test list per unit. Branched from `master` at `c036fab`, with local master merged in at `1f52c36` (the key-pair change).
Plan: `docs/plans/2026-10-06-branch-coverage.md`.

Units: org-members, on-chain-client, app and person touched (configs, comments and documents; app only by a one-line amendment to its hazard register); org-node dependent. The impact set is from `check-units.sh --impact refs/heads/master..HEAD`.

## The gate

Measured on: `4811cff` (`git rev-parse HEAD`), tree `704b553dacbb45415c7e6fb95215398d74c9049a` (`git rev-parse HEAD^{tree}` on a clean worktree). This was gate round 4, step 2, run after local master moved to `1f52c36` and was merged in. Rounds 1 to 3 measured the tree before that merge. Step 3 renamed nothing. Logs: the session scratchpad's `gate4-bc-*.log`.

| Gate | Result |
| --- | --- |
| org-members `cargo test -p org-members` | 271 passed, 0 failed, 1 ignored (`preflight_probe`, ignored by design) |
| org-members quint version, 3 typechecks, `quint test membership.qnt` | ok; 63 passing |
| org-members `quint run membership_mbt.qnt --invariant=mbtInv` | no violation found |
| on-chain-client `cargo test` (lib + 12 targets) | 73 passed, 0 failed; the 3 fuzz targets ran their full budget |
| on-chain-client `cargo test` (`write`, 4 targets) | 34 passed, 0 failed |
| person `cargo test -p person` | 123 passed, 0 failed |
| person `cargo clippy -p person --all-targets -- -D warnings` | clean |
| app `cargo test` (10 targets) | 159 passed, 0 failed |
| app `npm run check` | 0 errors, 1 warning ("Cannot find type definition file for 'node'"), the same as on master |
| app `npm run test` | 44 passed |
| org-node `cargo test` (lib + 26 targets) | 247 passed, 0 failed; the 4 fuzz targets ran their full budget |
| org-node quint typechecks and 5 `quint run` invariants | ok; no violation found |
| `check-units.sh` | exit 0 |
| `check-ids.sh --allow-draft-files`, every unit | exit 0 |
| `check-trace.sh`, every unit | exit 0. This change opens, resolves and accepts no items |
| Coverage, stable, `make coverage-<unit>` | org-members 96.55% lines / 94.57% regions (floors 92 / 91); on-chain-client 41.75% / 42.60% (41 / 42); person 100% / 100% (99 / 99). All exit 0 |
| Coverage, decision, `make coverage-branch-<unit>` | org-members 123 of 124 branches, 99.19% (floor 98); on-chain-client 34 of 34, 100% (floor 99); person 36 of 36, 100% (floor 99). All exit 0 |
| Floor fails when unmet | `make coverage-branch-person PERSON_BRANCHES=100.5` exits 2, "person: branch coverage is below its floor of 100.5%" |
| Working tree | clean |

Coverage against class C (statement and decision), with the owner's acceptance of 2026-10-06 of each gap below:

- org-members: met, except one branch outcome: the `!old.is_calculated()` operand of `calculate_delta`'s guard, which predates this change. Accepted as a documented gap.
- on-chain-client: statement coverage is the existing documented SHORTFALL (41.75% lines). Its 100% branch figure counts only functions that ran, and the run excludes the `write` feature (PR-b795an), so it does not close that shortfall. Accepted as a documented gap.
- person: met.
- org-node and app: no coverage target of any kind (tooth 5). Accepted as a documented gap.

## Red → green

This change Implements no ledger item, so there is no row. The red evidence for the one new behaviour, the branch floor, is the failing-floor run in the gate table above.

| Item | Test | Watched red |
| --- | --- | --- |

## What was wrong, and what was built

Class C requires statement and decision coverage. Decision coverage was measured in no unit, because rustc's branch instrumentation is unstable and the coverage targets ran on stable. This change adds `coverage-branch-<unit>` targets for the three units that already had statement coverage. They run on a pinned nightly in their own target directory, so nightly binaries no longer reach the stable recipes' directory, where they used to fail the stable floors. Each target judges its JSON summary against a branch floor with `jq -e`. While measuring, the change found that llvm-cov leaves branches in functions that never ran out of the count. That caveat is recorded once, in the Makefile beside the branch floors, and pointed to from the ADR and on-chain-client's config.

## Review

**finding-1**: code, low — The Makefile comment beside the branch floors and the plan both attribute the one uncovered org-members branch to "the IdNotFound arm of `update_leaf`". rustc's branch instrumentation does not count `?`/`ok_or`, so there is no branch there. Merging all instantiations, the one uncovered outcome is the `!old.is_calculated()` operand of `calculate_delta`'s guard in org-members/src/trie.rs. Fix the Makefile comment and plan; optionally add the missing test.
disposition: the Makefile comment and the plan now name `calculate_delta`'s `!old.is_calculated()` operand, without line numbers. The optional test is not added: this change only measures, and the gap is owner-accepted above.

**finding-2**: record — The claim in finding-1 is copied from the merged S1 record, `docs/verification/2026-10-06-worktree-org-members-absence-proofs.md`, which says "The one miss is trie.rs:449 (`update_leaf`)" — that record is wrong the same way, so the Makefile/plan citation backs nothing.
disposition: a dated, italic correction note is appended in that record's table cell. The original text is kept, and the Makefile and the plan no longer cite that record as backing.

**finding-3**: code, low — The pin is held in two places, `BRANCH_TOOLCHAIN` in the Makefile and `env.BRANCH_TOOLCHAIN` in rust.yml, with nothing checking agreement; the CI comment's "one string in one file" holds only within rust.yml.
disposition: the `coverage-branch` job now has a step, before `make`, that fails unless `grep -qx "BRANCH_TOOLCHAIN := $BRANCH_TOOLCHAIN" Makefile` matches, and the comment says the pin lives in both files. Tested locally: with the pinned value it prints a match, and with `nightly-2026-10-04` it does not.

**finding-4**: code, low — A stale statement remains in `docs/plans/2026-09-05-ratchet-setup.md` (the 2026-09-14 section): "decision coverage — unmeasured in all four units".
disposition: a dated amendment is appended inline. The original line is kept.

**finding-5**: code, low — Neither the ADR open item 1 amendment nor on-chain-client's config mentions PR-b795an: the coverage run builds without `write`, so the chain writer is outside both denominators and the 34 of 34 reads as covering more than it does.
disposition: the ADR amendment now names three limits, the third being the uncompiled `write` feature (PR-b795an), and on-chain-client's config gains one clause pointing to PR-b795an.

Reviewer's observation, not a finding: in the review worktree, `task-worktree.sh` had copied the parent's whole `target/`, and the first run counted those copied binaries: org-members read 28.65% lines and 123 of 220 branches. A clean re-run gave the expected figures. Stale binaries only lowered the figures, so the gate erred towards failing. The separate `target/branch-coverage` does not guard against a whole copied `target/`.

## Gaps

- org-node and app measure no coverage (tooth 5, ratchet-setup plan).
- on-chain-client's statement shortfall stands, and its `write` feature is outside every coverage run (PR-b795an, roadmap stage S2).
- The branch figure excludes functions that never ran, so each unit's branch floor must be read beside its statement floor.
- The floors were calibrated on aarch64-darwin; CI enforces them on x86_64-linux. The new CI job has not run yet, because no push happened in this session.
- The sandbox ran on-chain-client with `CARGO_HOME=/tmp/cargo_home_fuzz`, and app's npm checks with a `node_modules` copied from the `org-node-org-key-pair` worktree. Its `package.json` and `package-lock.json` are byte-identical to this branch's. The main checkout's `app/node_modules` is older than its lockfile (no vitest).
