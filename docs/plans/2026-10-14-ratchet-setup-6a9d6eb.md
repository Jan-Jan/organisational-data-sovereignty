# Ratchet setup — guardrails upgrade to 6a9d6eb — 2026-10-14

The upgrade record for `docs/plans/2026-10-14-guardrails-upgrade-6a9d6eb.md`.
The standing checklist is `docs/plans/2026-09-05-ratchet-setup.md`; this file
re-records only what an upgrade must re-record.

## Tool qualification (DO-330-lite)

- [x] **Basis.** The guardrails bats suite at `guardrails_version: 0.5.1`
      (the upstream template still reads 0.5.1), upstream commit
      `6a9d6eb8507752fa4e52fc7519740f69c9113ffe`, recorded as
      `guardrails_commit` in all six unit configs. The scripts in
      `.guardrails/scripts/` were copied from upstream `scripts/` at that
      commit, with the upstream working tree clean, and are not modified
      here.
- [x] **Result: all 1401 tests pass at `6a9d6eb`.** Run outside this
      project, with no bats installed here; bats is the copy the guardrails
      checkout vendored into its gitignored `tests/.bats-core/`.
      - Clean tree: `git archive 6a9d6eb`, extracted into a scratch
        directory no other process writes. `tests/run-tests.sh`, exit
        captured as `rc=$?`: TAP plan `1..1401` complete, **1395 ok, 6 not
        ok, rc=1**. The six are tests 911–915 and 917 of
        `tests/mutations.bats`, each failing with `mutate: not a git
        repository; the scratch tree cannot be built`: the archive has no
        `.git`, which `mutate.sh` needs. That is the environment of this
        run, not the tests.
      - Mutations, in a git checkout: `tests/mutations.bats` alone, in a
        scratch clone of the guardrails repository detached at `6a9d6eb`
        with a clean status: plan `1..7`, **7 ok, rc=0**.
      - The installed files were compared with the same archive: all
        thirteen `scripts/*.sh`, the three `.guardrails/templates/` files
        and the AGENTS.md managed block are byte-identical (`cmp`).

      *Corrected 2026-10-14, before merge.* This bullet first recorded a run
      in the live guardrails checkout as **1398 ok, 3 not ok**, with the
      exit code lost to zsh's read-only `status`, and read the three
      failures (tests 1109, 1153, 1179 of `tests/skills.bats`) as upstream
      defects. They were not. Upstream committed `bb249ca` in that checkout
      at 17:39, during the run (about 17:27–17:46), and the log mixed the
      two trees: test 1109 already carried `bb249ca`'s assertions while
      1153 still bore its `6a9d6eb` name. All three pass on the clean
      archive above, and on `bb249ca`. A run in a checkout someone else can
      write to qualifies no commit; run in a clean archive or clone, as
      above.
- [x] **Re-record** version, commit and result whenever `/ratchet` updates the
      scripts. Do not modify the scripts in this project; change them
      upstream, where the tests live.

## Existing verification records under the new `check-review.sh`

`upgrade-notes.md` item 6 (`check-review.sh --branch` once per existing
record) was done at the `35570e8` upgrade: on macOS the earlier
`check-review.sh` never executed, and since then every record has been read
at its own merge. This upgrade changes what `check-review.sh` checks only
behind `review_cite` (off, and unreadable under a unit manifest) and adds a
list-item `disposition:` to `ORPHAN-ANNOTATION`; the list-item grep over
`docs/` printed nothing.

## Signing

`check-signing.sh --setup` cannot pass inside the agent sandbox (gpg cannot
open `~/.gnupg/trustdb.gpg`). That is the environment, not the configuration;
the owner's `finish-merge.sh`, which runs `check-signing.sh --strict` in their
own terminal, is the proof, as for every merge since 2026-09-05.
