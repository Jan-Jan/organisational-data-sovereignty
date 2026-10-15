# Ratchet setup — guardrails upgrade to 344faa3 — 2026-10-14

The upgrade record for `docs/plans/2026-10-14-guardrails-upgrade-344faa3.md`.
The standing checklist is `docs/plans/2026-09-05-ratchet-setup.md`; this file
re-records only what an upgrade must re-record.

## Tool qualification (DO-330-lite)

- [x] **Basis.** The guardrails bats suite at `guardrails_version: 0.5.1`
      (the upstream template still reads 0.5.1), upstream commit
      `344faa33a09fdc2a4d381c7ef710e1f5631e24c8`, recorded as
      `guardrails_commit` in all six unit configs. The scripts in
      `.guardrails/scripts/` were copied from a clean `git archive` of that
      commit and are not modified here.
- [x] **Result: all 1450 tests pass at `344faa3`.** Run outside this
      project, with no bats installed here; bats is a copy of the one the
      guardrails checkout vendored into its gitignored `tests/.bats-core/`.
      - Clean tree: `git archive 344faa3`, extracted into a scratch
        directory no other process writes. `tests/run-tests.sh`, exit
        captured as `rc=$?`: TAP plan `1..1450` complete, **1444 ok, 6 not
        ok, rc=1**. The six are tests 943–947 and 949 of
        `tests/mutations.bats`, each failing with `mutate: not a git
        repository; the scratch tree cannot be built`: the archive has no
        `.git`, which `mutate.sh` needs. That is the environment of this
        run, not the tests.
      - Mutations, in a git checkout: `tests/mutations.bats` alone, in a
        scratch clone of the guardrails repository detached at `344faa3`
        with a clean status: plan `1..7`, **7 ok, rc=0**.
      - The installed files were compared with the same archive: all
        thirteen `scripts/*.sh`, the four `.guardrails/templates/` files
        and the AGENTS.md managed block are byte-identical (`cmp`).
- [x] **Re-record** version, commit and result whenever `/ratchet` updates the
      scripts. Do not modify the scripts in this project; change them
      upstream, where the tests live.

## Existing verification records under the new scripts

`check-review.sh` did not change between `6a9d6eb` and `344faa3`, so
upgrade-notes item 6 needs no rerun. Records already merged keep their gate
tables; from this change on, the gate's figures travel as the squash commit's
`Base:`, `Gate-tree:` and `Gate:` trailers, which `finish-merge.sh` guard 5
checks.

## Signing

`check-signing.sh --setup` cannot pass inside the agent sandbox (gpg cannot
open `~/.gnupg/trustdb.gpg`, and signing times out). That is the
environment, not the configuration. The owner's `finish-merge.sh`, which
runs `check-signing.sh --strict` and now the `Gate-tree:` guard in their own
terminal, is the proof, as it has been for every merge since 2026-09-05.
