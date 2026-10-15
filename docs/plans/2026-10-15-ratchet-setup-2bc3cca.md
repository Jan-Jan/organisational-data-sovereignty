# Ratchet setup — guardrails upgrade to 2bc3cca — 2026-10-15

The upgrade record for `docs/plans/2026-10-15-guardrails-upgrade-2bc3cca.md`.
The standing checklist is `docs/plans/2026-09-05-ratchet-setup.md`; this file
re-records only what an upgrade must re-record.

## Tool qualification (DO-330-lite)

- [x] **Basis.** The guardrails bats suite at `guardrails_version: 0.5.1`
      (the upstream template still reads 0.5.1), upstream commit
      `2bc3cca04b75e1205d1c79a8e7b40c302036bf6f`, recorded as
      `guardrails_commit` in all six unit configs. The scripts in
      `.guardrails/scripts/` were copied from a `git clone -q` of the
      guardrails checkout at that commit, in a scratch directory outside this
      project (`ratchet` step 0), and are not modified here.
- [x] **Result: all 1494 tests pass at `2bc3cca`.** One qualifying run, in
      the clone, with no bats installed in this project, under GNU
      `parallel`: `tests/run-tests.sh > <tmp>/ratchet.log 2>&1; rc=$?`, the
      log beside the clone, not inside it. TAP plan `1..1494` complete,
      **1494 ok, 0 not ok, rc=0**. The log's one `run-tests:` line is
      `run-tests: testing commit 2bc3cca04b75e1205d1c79a8e7b40c302036bf6f`,
      so the result describes that commit and no other tree. The clone being
      a git checkout, `tests/mutations.bats` ran inside the same run (the
      `344faa3` record needed a second run for it).
- [x] **Where the log is kept.** `ratchet.log` lives in the agent session's
      scratchpad (`<tmp>` above), outside the clone and outside this
      repository, and is **not kept beyond that session**. Its figures are
      the ones quoted above: plan `1..1494`, 1494 `ok`, 0 `not ok`, and the
      single `run-tests:` line naming `2bc3cca`. To re-qualify, clone the
      guardrails checkout at that commit and rerun the suite.
- [x] **Installed files compared with the clone:** all thirteen
      `scripts/*.sh`, the four `.guardrails/templates/` files and the
      AGENTS.md managed block are byte-identical (`cmp`).
- [x] **Re-record** version, commit and result whenever `/ratchet` updates the
      scripts. Do not modify the scripts in this project; change them
      upstream, where the tests live.

## Existing verification records under the new scripts

`check-review.sh` changed between `344faa3` and `2bc3cca` in comments and
messages only (the merge-change step numbers), so upgrade-notes item 6 needs
no rerun. Records merged before this change name the old step numbers; read
them through the table in upgrade-notes "The merge steps are renumbered".

## Signing

`check-signing.sh --setup` cannot pass inside the agent sandbox (gpg signing
times out: `gpg: signing failed: Timeout`). That is the environment, not the
configuration. The owner's `finish-merge.sh`, which runs
`check-signing.sh --strict` and the `Gate-tree:` guard in their own
terminal, is the proof, as it has been for every merge since 2026-09-05.
