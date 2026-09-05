# Guardrails verification targets.
#
# `coverage-org-members` and `coverage-on-chain-client` are the targets named
# by `coverage_command` in org-members/.guardrails/config.yaml and
# on-chain-client/.guardrails/config.yaml (per-unit configs since 2026-09-05;
# `coverage` runs both and is what CI calls). Note what that does and does not mean: NO check
# script reads `coverage_command` — `grep -rn coverage_command
# .guardrails/scripts/` finds it only in lib.sh's schema key list. The gate is
# an instruction to whoever runs `verify-before-merge` (step 5: run it and
# judge the report against the class target), plus the `coverage` job in
# .github/workflows/rust.yml, which is the part that is mechanical.
#
# Requires: cargo-llvm-cov and the llvm-tools-preview component.
#   cargo install cargo-llvm-cov && rustup component add llvm-tools-preview

.PHONY: coverage coverage-org-members coverage-on-chain-client

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
ORG_MEMBERS_LINES := 92
ORG_MEMBERS_REGIONS := 91
ON_CHAIN_CLIENT_LINES := 52
ON_CHAIN_CLIENT_REGIONS := 58

# Per-crate, so a regression in the well-covered crate cannot hide behind the
# poorly-covered one. Known limit, unfixable today: each floor is still an
# average over its crate, so coverage moving BETWEEN files inside one crate is
# invisible. In on-chain-client, client.rs is 353 of 715 lines, so a collapse
# in decode/v_paseo_ah.rs (96.23%, the code the fuzz corpora protect) could be
# absorbed by a rise in client.rs. The fix is --fail-under-file-lines, which
# cannot be enabled yet: decode/mod.rs is at 0.00%, so any per-file floor
# above zero fails immediately. Raising that file off zero is what unlocks it.
coverage: coverage-org-members coverage-on-chain-client

# Scope note: this omits `mbt_conformance`, the fourth org-members test
# target, and the floors are calibrated against the reduced measurement — so
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
coverage-org-members:
	cargo llvm-cov -p org-members \
		--lib --test integration_test --test fuzz_tests \
		--summary-only \
		--fail-under-lines $(ORG_MEMBERS_LINES) \
		--fail-under-regions $(ORG_MEMBERS_REGIONS)

# Mirrors the on-chain-client entry in verify_commands, fuzz targets included:
# the corpora are part of what covers the decoders.
#
# This floor is LOW, and it is a shortfall rather than a target met:
# client.rs sits at 12.75% because it is exercised almost entirely by the
# chopsticks/anvil integration tests, which need on-chain/scripts/node_modules
# and are outside both verify_commands and this measurement. The floor records
# what is true today so a regression is still caught; it does not claim the
# class target is reached. See the verification record for this change, where
# the shortfall is accepted explicitly, and the provisioning item in
# docs/plans/2026-08-26-ratchet-setup.md, which is what raises it.
coverage-on-chain-client:
	cargo llvm-cov --manifest-path on-chain-client/Cargo.toml \
		--lib \
		--test fuzz_decode_org_state \
		--test fuzz_parse_revive_event \
		--test fuzz_event_round_trip \
		--summary-only \
		--fail-under-lines $(ON_CHAIN_CLIENT_LINES) \
		--fail-under-regions $(ON_CHAIN_CLIENT_REGIONS)
