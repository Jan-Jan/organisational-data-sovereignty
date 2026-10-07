# Guardrails verification targets.
#
# `coverage-org-members`, `coverage-on-chain-client` and `coverage-person` are
# the targets named by `coverage_command` in org-members/.guardrails/config.yaml,
# on-chain-client/.guardrails/config.yaml and person/.guardrails/config.yaml
# (per-unit configs since 2026-09-05; person's since 2026-10-04; `coverage`
# runs all three and is what CI calls). Since 2026-10-06 each of those configs
# also names its `coverage-branch-<unit>` twin, the decision-coverage half, run
# by `coverage-branch` and CI's job of the same name (nightly; see
# BRANCH_TOOLCHAIN). Note what that does and does not mean: NO check
# script reads `coverage_command` — `grep -rn coverage_command
# .guardrails/scripts/` finds it only in lib.sh's schema key list. The gate is
# an instruction to whoever runs `verify-before-merge` (step 5: run it and
# judge the report against the class target), plus the `coverage` and
# `coverage-branch` jobs in .github/workflows/rust.yml, which are the part that
# is mechanical.
#
# Requires: cargo-llvm-cov and the llvm-tools-preview component; the branch
# targets also need jq and the pinned nightly with its own llvm-tools-preview.
#   cargo install cargo-llvm-cov && rustup component add llvm-tools-preview
#   rustup toolchain install $(BRANCH_TOOLCHAIN) --component llvm-tools-preview

.PHONY: coverage coverage-org-members coverage-on-chain-client coverage-person \
	coverage-branch coverage-branch-org-members coverage-branch-on-chain-client \
	coverage-branch-person

# TWO metrics, because the STATEMENT-coverage half of the class target (class
# C: statement AND decision) is measured two ways here and the two
# are not the same thing. LLVM line coverage credits a whole source line if
# any region on it executed, so `?`-chains, match arms and multi-statement
# lines over-credit. Region coverage is the closer analogue of statement
# coverage, so it is enforced too — and for on-chain-client the two diverge by
# more than five points, in opposite directions between the crates:
#
#   measured 2026-08-26        lines     regions
#   org-members                93.50%     92.20%
#   on-chain-client            53.57%     59.19%
#
# Floors sit ONE POINT below the measurement rather than at it. Two reasons,
# both measured rather than assumed: at 1000 lines, six lines of new uncovered
# code drops org-members under a floor of 93, and on-chain-client has only
# five covered lines of slack against a floor of 53 — a floor that tight
# converts an ordinary edit into a build failure and creates pressure to lower
# it, which the rule below forbids. And the floors are calibrated on
# aarch64-darwin but enforced in CI on x86_64-linux against a floating
# toolchain, where LLVM's line mapping need not agree to the line.
#
# The floors RATCHET: raise one when the real figure rises, never lower one to
# make a merge pass. Lowering a floor is a decision that belongs in a
# verification record with a reason, not in a hurry.
#
# RESOLVED, 2026-09-10 — the on-chain-client floors below are RECALIBRATED onto
# a new measurement basis, on the owner's explicit decision. The reasoning is
# the rest of this note, which is where it is recorded while the change is on
# its branch; the decision is carried into this change's verification record,
# docs/verification/2026-09-10-worktree-guardrails-on-chain-client-risk.md,
# which merge-change writes at step 6b — after review — so do not expect to
# find that file until this change merges. Read this note before concluding
# that a floor was lowered to make a merge pass: the quantity being measured
# changed, and the old floors measured a figure that can never honestly be met
# again.
#
# The distinction that makes this a re-baselining and not a lowering. Until
# this change, twenty-three of this crate's tests lived in `#[cfg(test)]`
# modules INSIDE on-chain-client/src, so llvm-cov counted their bodies as
# library lines — and, being executed, as 100%-covered library lines. They
# inflated the numerator and the denominator together. Relocating them into
# on-chain-client/tests (where this unit's `test_paths` actually is, so that
# `check-trace.sh` can read their `verifies:` annotations at all) removed 214
# such lines from the measurement. Netting them out of the old figure, real
# library coverage ROSE across this change: ~169 covered library lines before,
# 215 after. The floors are recalibrated one point below the new measurement,
# which is the same rule the paragraph above the old figures states.
#
# The ratchet rule itself is untouched and still binds: from this basis
# forward, raise a floor when the real figure rises and never lower one to make
# a merge pass. What follows is the measurement the new floors sit on.
#
# Measured on the change branch of
# docs/plans/2026-09-10-on-chain-client-risk-analysis.md, after T3–T7 relocated
# twenty-three tests out of `#[cfg(test)]` modules in on-chain-client/src and
# T10 added the eight relocated targets to the recipe below:
#
#   recipe                                       lines     regions   denominator
#   old (--lib + 3 fuzz), before the relocation  53.57%     59.19%   715 lines
#   old (--lib + 3 fuzz), after  the relocation  23.75%     24.08%   501 lines
#   new (+ the 8 relocated targets)              42.91%     43.47%   501 lines
#   + the 9th (type_widths), re-measured         41.75%     42.45%   491 lines
#
# The last row is review round 1's finding 1 landing: the final two
# `#[cfg(test)]` tests left `src/types.rs` for `tests/type_widths.rs`. It is
# the same arithmetic as the row above it, one more time and much smaller —
# ten fully-covered library lines left the measurement (types.rs 13 → 3), so
# 215 covered of 501 became 205 of 491. The headline fell by about a point
# while nothing stopped being tested; both tests still run, in the ninth
# target of this recipe. THE FLOORS ARE NOT MOVED for it: 41 / 42 still hold,
# now 0.75 / 0.45 below the measurement rather than one point. That is tighter
# slack than the rule above prefers, and it is left alone deliberately —
# raising a floor is a ratchet step to take on a settled basis, not in the
# same breath as a re-baselining.
#
# The eight targets recover nineteen points of the thirty the relocation cost,
# and the residue is arithmetic rather than lost testing: the 214 lines that
# left src were test bodies, counted as library lines and 100%-covered, so they
# were inflating BOTH sides of the old ratio. Netting them out: ~169 of the 383
# lines the old figure counted as covered were real library lines; 215 were on
# the eight-target basis, and **205 of 491** are on the nine-target basis this
# recipe measures today. So the crate is MORE covered in absolute terms and the
# denominator is honest now.
#
# THE DENOMINATOR IS 491, NOT 501. Every "of 501" in the rows above is a figure
# at the eight-target basis and is kept as history; since `type_widths` joined
# the recipe the measurement spans 491 library lines, because ten
# fully-covered lines left src/types.rs with the last two `#[cfg(test)]` tests.
# client.rs alone is 336 of those 491 lines at **16.37%** (measured 2026-09-10:
# 336 lines, 281 missed) — client.rs itself did not change across the
# re-measurement; only the denominator did. It is exercised almost entirely by
# the chopsticks/anvil targets that sit outside this measurement — that
# shortfall, unchanged since 2026-08-26, is what the figure is really
# reporting.
#
# Per-file floors are now UNBLOCKED, and deliberately not taken up here. The
# note below the floors says `--fail-under-file-lines` "cannot be enabled yet:
# decode/mod.rs is at 0.00%". That file is at 64.29% as of this change, so the
# one blocker it names is gone. Enabling per-file floors is the fix for the
# known limit that same note describes, and it needs a number chosen
# deliberately for client.rs at 16.37% — its own change, not this one.
ORG_MEMBERS_LINES := 92
ORG_MEMBERS_REGIONS := 91
# Recalibrated 2026-09-10 from 52/58 — see the RESOLVED note above for why the
# basis changed. Set one point below the then-measured 42.91% / 43.47%; the
# measurement is 41.75% / 42.45% since `type_widths` joined the recipe, and
# these floors are deliberately unchanged under it.
ON_CHAIN_CLIENT_LINES := 41
ON_CHAIN_CLIENT_REGIONS := 42
# person, first floors, 2026-10-04 — same rule: one point below the
# measurement, rounded down to an integer. Measured on the task branch that
# added this target, with `cargo llvm-cov -p person --summary-only` (stable
# rustc 1.99.0 (b940084d7 2026-09-28), cargo-llvm-cov 0.9.0, aarch64-darwin):
#
#   measured 2026-10-04        lines     regions
#   person                     81.03%     82.37%   (158 of 195 lines; 243 of 295 regions)
#
# Raised 2026-10-04 by the review-findings task (worktree-person-unit-findings1),
# same toolchain, after the derived-LLR and property tests:
#
#   person                    100.00%    100.00%   (215 of 215 lines; 324 of 324 regions)
#
# Re-measured 2026-10-05 by the fourth review-findings task
# (worktree-person-unit-findings4), same toolchain, after the torsion-free
# device-key check:
#
#   person                    100.00%    100.00%   (215 of 215 lines; 331 of 331 regions)
#
# Historical (before the coverage-branch-* targets): the decision half was
# measured by hand, `cargo +nightly llvm-cov -p person --branch --summary-only`
# (cargo 1.101.0-nightly (f3865b2a4 2026-09-29), cargo-llvm-cov 0.9.0),
# re-measured 2026-10-05 by the fourth review-findings task
# (worktree-person-unit-findings4): 28 of 28 branches, 100% decision coverage.
#
# Re-measured 2026-10-06 by the Person definition change
# (worktree-person-requirements), same toolchains, floors unchanged:
#
#   person                    100.00%    100.00%   (344 of 344 lines; 535 of 535 regions)
#
# and with nightly --branch: 36 of 36 branches, 100% decision coverage.
# Enforced since 2026-10-06 by `coverage-branch-person` (PERSON_BRANCHES below).
# The 344/344 and 535/535 figures are stable cargo 1.99.0, cargo-llvm-cov 0.9.0.
# For stale nightly binaries and `cargo llvm-cov clean`, see BRANCH_TARGET_DIR.
PERSON_LINES := 99
PERSON_REGIONS := 99

# DECISION coverage, the other half of the class C target, added 2026-10-06
# (worktree-guardrails-branch-coverage, docs/plans/2026-10-06-branch-coverage.md).
# rustc's branch instrumentation is unstable (`-Z coverage-options=branch`), so
# `cargo llvm-cov --branch` runs only on nightly. The nightly is PINNED, by the
# date of its channel manifest, so a nightly regression cannot move the gate and
# moving the pin is a reviewed edit. Install it once:
#   rustup toolchain install $(BRANCH_TOOLCHAIN) --component llvm-tools-preview
# Override it on the command line (`make coverage-branch BRANCH_TOOLCHAIN=nightly`)
# only where the floating nightly is known to be the same build.
#
# What it counts: both outcomes of each `if`/`while` condition and of each `&&`
# and `||` operand. rustc's support for `match` arms as branches is partial, so
# match arms are guarded chiefly by the region floor above, not by this one.
#
# Floors are branch PERCENTAGES, one point below the measurement rounded down,
# ratcheting upward only — the same rule as the line and region floors.
# cargo-llvm-cov 0.9.0 has no --fail-under-branches, so the recipe exports the
# JSON summary and `jq -e` judges it. A summary with no branches at all reads
# 0% or null and fails either way: an instrumentation that counted nothing is
# not a pass.
#
# Builds go to their own target directory, target/branch-coverage, so nightly
# binaries never land in target/llvm-cov-target, where they fail the stable
# floors. A HAND-RUN `cargo +nightly llvm-cov --branch` still lands there: run
# a plain `cargo llvm-cov clean` (not `--workspace`) after one.
#
#   measured 2026-10-06        branches                   (lines, stable recipe)
#   org-members                123 of 124   99.19%        96.55%
#   on-chain-client             34 of  34  100.00%        41.75%
#   person                      36 of  36  100.00%       100.00%
#   (nightly-2026-10-03 = rustc 1.101.0-nightly 0abfedbc7 2026-10-02,
#   cargo-llvm-cov 0.9.0, aarch64-darwin)
#
# READ THE DENOMINATOR. A branch inside a function that never ran is reported
# as "[Folded - Ignored]" and is left out of the count, so this figure is the
# decision coverage OF THE CODE THAT EXECUTED. Code that never ran is caught by
# the line and region floors, not here. That is why on-chain-client reads 100%
# beside 41.75% lines: client.rs's chain-facing async paths (its Revive event
# filter, for one) never run in this suite, and their conditions are not
# counted. on-chain-client's class C shortfall therefore stands; this floor
# does not close it. The one uncovered org-members branch is the
# `!old.is_calculated()` operand of `calculate_delta`'s guard in
# org-members/src/trie.rs: no test has a calculated `self` with an
# uncalculated `old`. It predates the absence-proofs change (S1).
BRANCH_TOOLCHAIN := nightly-2026-10-03
BRANCH_TARGET_DIR := $(CURDIR)/target/branch-coverage
BRANCH_REPORT_DIR := $(BRANCH_TARGET_DIR)/reports
ORG_MEMBERS_BRANCHES := 98
ON_CHAIN_CLIENT_BRANCHES := 99
PERSON_BRANCHES := 99

# Per-crate, so a regression in the well-covered crate cannot hide behind the
# poorly-covered one. Known limit, unfixable today: each floor is still an
# average over its crate, so coverage moving BETWEEN files inside one crate is
# invisible. In on-chain-client, client.rs is 336 of 491 lines, so a collapse
# in decode/v_paseo_ah.rs (100.00% as of 2026-09-10, the code the fuzz corpora
# protect) could be absorbed by a rise in client.rs. The fix is
# --fail-under-file-lines. It was blocked while decode/mod.rs sat at 0.00%,
# because any per-file floor above zero failed immediately; that file is now at
# 64.29%, so the block is gone and enabling it is available work (see the
# RESOLVED note above). Figures updated 2026-09-10; the pre-relocation ones
# were 353 of 715 and 96.23%.
coverage: coverage-org-members coverage-on-chain-client coverage-person

# Scope note: this omits `mbt_conformance` (`newtypes` and `encoding_golden`,
# added 2026-10-04, and `person_error_mapping` and `absence_proofs`, both
# added 2026-10-06, are measured), and the floors are calibrated against the
# reduced measurement — so
# the gate is conservative relative to the crate's true coverage, but NOT
# relative to its own floor. The reason for omitting it is that it shells out
# to the quint CLI and, measured with quint off PATH, FAILS rather than skips
# ("Failed to execute Quint command"), despite its own header comment claiming
# otherwise. Including it would make the coverage gate pass or fail on whether
# the machine has quint and a writable ~/.quint.
#
# quint.yml's `mbt` job and the `test` job in rust.yml both RUN that lane with
# quint installed, which keeps the conformance argument honest. Neither
# measures its coverage contribution; that remains unknown.
ORG_MEMBERS_COVERAGE_ARGS := -p org-members \
	--lib --test integration_test --test fuzz_tests \
	--test newtypes --test encoding_golden --test person_error_mapping \
	--test absence_proofs

coverage-org-members:
	cargo llvm-cov $(ORG_MEMBERS_COVERAGE_ARGS) \
		--summary-only \
		--fail-under-lines $(ORG_MEMBERS_LINES) \
		--fail-under-regions $(ORG_MEMBERS_REGIONS)

# Mirrors the on-chain-client entry in verify_commands, fuzz targets included:
# the corpora are part of what covers the decoders.
#
# This floor is LOW, and it is a shortfall rather than a target met:
# client.rs sits at 16.37% — 281 of its 336 lines unexercised, of the 491 this
# recipe measures — because it is exercised almost entirely by the
# chopsticks/anvil integration tests, which need on-chain/scripts/node_modules
# and are outside both verify_commands and this measurement. (This note read
# 12.75% until 2026-09-10; that was the PRE-RELOCATION figure, left in place
# when the basis changed, and it contradicted the 16.37% in the RESOLVED note
# above. 16.37% is the measured figure on the current recipe and the two now
# agree.) The floor records what is true today so a regression is still
# caught; it does not claim the
# class target is reached. The shortfall is accepted explicitly in this
# change's verification record — written at merge-change step 6b, after
# review — and the provisioning item that raises it is in
# docs/plans/2026-09-05-ratchet-setup.md.
#
# 2026-09-10: the nine annotated targets below were added alongside
# `--features test-support`, in the same order as verify_commands, so this
# recipe and the gate keep measuring the same suite. (Eight of them with the
# risk analysis; the ninth, `type_widths`, with review round 1's finding 1,
# which relocated the last two `#[cfg(test)]` tests out of src/types.rs.) They had to move together:
# docs/plans/2026-09-10-on-chain-client-risk-analysis.md relocated twenty-three
# tests out of `#[cfg(test)]` modules inside on-chain-client/src, which drops the
# `--lib`-only figure twice over (the relocated bodies were themselves covered
# library lines, and the branches they exercised stop being reached at all).
ON_CHAIN_CLIENT_COVERAGE_ARGS := --manifest-path on-chain-client/Cargo.toml \
	--features test-support \
	--lib \
	--test fuzz_decode_org_state \
	--test fuzz_parse_revive_event \
	--test fuzz_event_round_trip \
	--test contract_address_filter \
	--test log_ownership \
	--test decode_revive_event \
	--test decode_org_state \
	--test runtime_version_dispatch \
	--test h160_mapping \
	--test storage_slot_layout \
	--test best_lane_reorg_rule \
	--test type_widths

coverage-on-chain-client:
	cargo llvm-cov $(ON_CHAIN_CLIENT_COVERAGE_ARGS) \
		--summary-only \
		--fail-under-lines $(ON_CHAIN_CLIENT_LINES) \
		--fail-under-regions $(ON_CHAIN_CLIENT_REGIONS)

# Every person test target, none excluded: the crate has no lane that needs a
# tool outside cargo, so the measurement and `cargo test -p person` (its
# verify_commands entry) run the same suite.
PERSON_COVERAGE_ARGS := -p person

coverage-person:
	cargo llvm-cov $(PERSON_COVERAGE_ARGS) \
		--summary-only \
		--fail-under-lines $(PERSON_LINES) \
		--fail-under-regions $(PERSON_REGIONS)

# Decision coverage per unit, on the pinned nightly (see BRANCH_TOOLCHAIN). Each
# recipe measures EXACTLY the test set of its stable twin above: both read the
# same *_COVERAGE_ARGS variable, so the statement and decision figures always
# describe one suite.
coverage-branch: coverage-branch-org-members coverage-branch-on-chain-client coverage-branch-person

# $(1) report name, $(2) floor. Prints the figure, then fails below the floor.
define judge_branches
	@jq -r '.data[0].totals.branches | "$(1): \(.covered) of \(.count) branches, \(.percent)% (floor $(2)%)"' $(BRANCH_REPORT_DIR)/$(1).json
	@jq -e --argjson floor $(2) '.data[0].totals.branches.percent >= $$floor' $(BRANCH_REPORT_DIR)/$(1).json > /dev/null \
		|| { echo "$(1): branch coverage is below its floor of $(2)%"; exit 1; }
endef

coverage-branch-org-members:
	mkdir -p $(BRANCH_REPORT_DIR)
	CARGO_TARGET_DIR=$(BRANCH_TARGET_DIR) cargo +$(BRANCH_TOOLCHAIN) llvm-cov \
		$(ORG_MEMBERS_COVERAGE_ARGS) \
		--branch --json --summary-only \
		--output-path $(BRANCH_REPORT_DIR)/org-members.json
	$(call judge_branches,org-members,$(ORG_MEMBERS_BRANCHES))

coverage-branch-on-chain-client:
	mkdir -p $(BRANCH_REPORT_DIR)
	CARGO_TARGET_DIR=$(BRANCH_TARGET_DIR) cargo +$(BRANCH_TOOLCHAIN) llvm-cov \
		$(ON_CHAIN_CLIENT_COVERAGE_ARGS) \
		--branch --json --summary-only \
		--output-path $(BRANCH_REPORT_DIR)/on-chain-client.json
	$(call judge_branches,on-chain-client,$(ON_CHAIN_CLIENT_BRANCHES))

coverage-branch-person:
	mkdir -p $(BRANCH_REPORT_DIR)
	CARGO_TARGET_DIR=$(BRANCH_TARGET_DIR) cargo +$(BRANCH_TOOLCHAIN) llvm-cov \
		$(PERSON_COVERAGE_ARGS) \
		--branch --json --summary-only \
		--output-path $(BRANCH_REPORT_DIR)/person.json
	$(call judge_branches,person,$(PERSON_BRANCHES))
