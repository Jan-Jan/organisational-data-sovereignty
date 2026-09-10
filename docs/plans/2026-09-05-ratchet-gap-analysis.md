# Ratchet gap analysis — 2026-09-05 (multi-unit conversion)

Second `/ratchet` on this repository, invoked as `/ratchet this is monorepo`.
Source toolkit: `~/Coding/guardrails` at `guardrails_version: 0.5.1`, upstream
commit `e2eac86ff8db2157735b9604b7ade190d4f465f3` (2026-09-05, "the skills
learn the units"). The installed copy was `0.5.1` at `bb7eee5`; every one of
the eight installed scripts differs from upstream and one script
(`check-units.sh`) did not exist. Mode: **retrofit** — the first ratchet's
teeth 1–5 are merged (`docs/plans/2026-08-26-ratchet-gap-analysis.md`), the
project is class C, and 32 ID'd items exist (12 REQ, 8 HAZ, 10 RC, 2 PR).

This change does two things that the first ratchet's rule "the first tooth
migrates no existing document" could not both honour:

1. **Upgrades the scripts** to the version that carries the unit machinery.
2. **Converts the repository to multi-unit mode** — a root `units.yaml`, four
   unit configs, and **no root config**.

The second forces a relocation the first ratchet never had to make: a unit's
ledgers must live inside the unit's own directory, and a manifest repository
has no root config to read `docs/requirements` from. So the existing ledger
files moved, with their history, into `org-members/docs/`. Their item IDs, item
text and annotations are unchanged; `check-trace.sh` counts exactly what it
counted before (REQ 12, HAZ 8, RC 10, PR 2) from the new location.

## The interview (D9: facts, not rules)

Asked one question at a time, each with a recommendation. Answers, and where
each was written:

| Question | Answer | Written to |
|---|---|---|
| One system in many packages, or many systems? | **Many systems** | mode: manifest repository |
| Which directories are units? | `org-members`, `on-chain-client`, `org-node`, and — added by the owner over the recommendation of three — `app` | `units:` in `.guardrails/units.yaml` |
| What is outside compliance, and why? | `on-chain` (Foundry, separate toolchain), `quint` (models: evidence, not shipped code), `spike-common`/`spike-keyhive`/`spike-p2panda` (Phase 1.d spikes, salvage-only), `docs` (repository-level records), `.github` | `not_a_unit:` with the reason as a comment beside each entry |
| Safety class per unit? | **C for all four** | `safety_class:` in each unit config; rationale in `docs/adr/2026-09-05-units-and-per-unit-classes.md` |
| What does each unit depend on? | **No edges this tooth.** The code has four (org-node → org-members, on-chain-client; app → org-node, on-chain-client); each is declared later with its dependency assessment | `depends_on:` left commented in each config, edges named in `units.yaml` |
| Any segregated dependency? | Not asked — with no edges and every unit at C there is nothing to segregate | — |

Two untracked directories (`.claire/`, `_apalache-out/`) would have read as
`UNCLAIMED-PATH` because `check-units.sh` scans `git ls-files --others`. Both
are local tool output and were gitignored rather than disclaimed (my default
when the owner did not name a preference).

## What exists (after the first ratchet)

| Area | State before this change |
|---|---|
| Machinery | `.guardrails/config.yaml` (root, single unit), eight scripts at upstream `bb7eee5`, `templates/verification.md`. Managed block in `AGENTS.md` identical to upstream's current template. |
| Ledgers | `docs/requirements` (12 REQ), `docs/risk` (8 HAZ, 10 RC, filled acceptability matrix), `docs/problems` (2 PR, both open), `docs/architecture` (README + soup, no items). All 12 REQ are org-members behaviour; the RMF names org-node eight times and on-chain-client twice; PR-hvg2dy is an org-node defect. |
| Tests | 49 `verifies:` annotations in `org-members/tests`. None elsewhere. |
| CI | `rust.yml` runs `check-trace.sh` and `check-ids.sh` bare at the root, plus `verify_commands`, clippy, no-std, coverage, non-strict signing on master, `check-review.sh` on PRs. `quint.yml` unchanged. |
| Signing | `check-signing.sh --setup` exits 0 on the owner's machine (2026-09-01); four signed merges on master since. Unchanged by this change. |
| Uncommitted work in the primary checkout | `app/package.json` modified, `on-chain/scripts/gen-admin-key.mjs` untracked. Not touched; not part of this change. |

## What was missing

- A manifest: without one, upstream `e2eac86`'s gates run in single-unit mode
  and the four crates share one class, one ledger set and one problem budget.
- Unit configs, and ledgers for the three units that had none.
- A CI shape for a manifest repository: a bare `check-trace.sh` at the root is
  now **exit 2**, deliberately, so the old `guardrails` job would have failed
  on the first push after the upgrade.
- Recording of `guardrails_commit` (the checkable half of the tool-qualification
  basis) — the 0.5.1 config never set it.

## Conflicts (flagged, not deleted)

1. **"The first tooth migrates no existing document."** Broken knowingly and
   minimally: nine ledger files moved into `org-members/docs/` with `git mv`
   semantics (rename detection preserves history); no item was edited. The
   alternative — disclaiming `docs` with the ledgers still in it — would have
   put 32 items where no gate reads them, which is the false green the manifest
   exists to prevent.
2. **"Do not change CI in the first tooth."** The `guardrails` job in
   `rust.yml` was rewritten because the upgrade made its first line exit 2.
   New shape: `check-units.sh` once, then `check-trace.sh` and `check-ids.sh`
   per unit over `check-units.sh --list` — the base-branch union, since there
   is no change under review on a push. The `test` job now also runs
   org-node's cargo entry. app's two entries run nowhere in CI, for two
   different reasons: the cargo entry needs Tauri's Linux system packages
   (webkit2gtk, libsoup, …) the workflow does not install; the npm entry needs
   `npm --prefix app ci` first (app/node_modules is gitignored) — node itself
   is already set up in the `test` job. The setup checklist carries both. No
   other job changed.
3. **D14 (a provider's defect is filed in the provider's ledger) versus this
   tooth's "no edges".** PR-hvg2dy is an org-node defect and was moved to
   `org-node/docs/problems` first. Measured: it drew `UNDECLARED-DEPENDENCY`
   in **both** units' runs — the report cites `RC-9z65hw` and `PR-zz4exm`
   (org-members items), and org-members' RMF cites the report back. Neither
   direction is legal without an edge, and the reverse direction is not legal
   even with one (only `expects:` crosses backwards). It stays in
   org-members' ledger this tooth, exactly where it was, and moves when
   org-node's own risk analysis exists (tooth 3 below).
4. **Verification records and older plans name `docs/requirements/…` paths.**
   Historical; left unedited, per the project's convention of dated
   corrections over rewrites. The two 2026-08-26 plans carry a superseded
   pointer at the top.
5. **`AGENTS.md` prose above the managed block** still says "for crate-specific
   guidance see `org-members/AGENTS.md`". True, and now each unit also has
   `<unit>/docs/CONTEXT.md`. Not edited (human text).

## Configuration written

Per unit, all class C, all six prefixes, problem limits 30/10, expectation
limits 90/10, no `depends_on:`:

| Unit | `strict_paths` | `test_paths` | `verify_commands` | `coverage_command` |
|---|---|---|---|---|
| `org-members` | `org-members/src` | `org-members/tests` | `cargo test -p org-members`; `quint typecheck` membership + protocol | `make coverage-org-members` |
| `on-chain-client` | `on-chain-client/src` | `on-chain-client/tests` | the `--lib` + three fuzz targets line, unchanged | `make coverage-on-chain-client` |
| `org-node` | `org-node/src` (**new**) | `org-node/tests` (**new**) | `cargo test -p org-node --features app,test-support --lib` + five named targets (the three chopsticks targets excluded; the feature flags are demanded by the targets' `required-features`); `quint typecheck quint/protocol.qnt` | none — gap |
| `app` | `app/src-tauri/src` (**new**) | `app/src-tauri/tests` (**new**, `.gitkeep`) | `cargo test --manifest-path app/src-tauri/Cargo.toml`; `npm --prefix app run check` | none — gap |

Nothing was loosened: every path and command of the root config appears in
some unit's config, and `quint typecheck quint/protocol.qnt` appears in two.
What DID narrow, by design (D4/D7), is scan scope: `check-ids.sh`'s
DRAFT-ID/DRAFT-FILE/MALFORMED-ID and `check-trace.sh`'s DANGLING-REF
definition set are now unit-scoped (plus foreign exports), with
`check-units.sh` covering disclaimed paths and root files for draft tokens
(DISCLAIMED-DRAFT) but deliberately not MALFORMED-ID. Measured effect today:
none — no definition-form line exists outside `org-members/docs/` and every
unit reports `foreign 0, reverse 0`.

**org-node's and app's verify_commands were measured at this change's own
gate.** This change's impact set is all four units (a change under the root
`.guardrails/` maps to every unit), so both lists were due at this merge, not
a later one. The first draft of org-node's line omitted
`--features app,test-support` and failed closed at the gate: cargo refuses an
explicit `--test` whose `required-features` are off. The independent review
found the same statically. The corrected line passes (38 passed, 0 failed,
both fuzz targets run). The author's earlier belief that the sandbox could not
build org-node came from a `--offline` attempt against a scratch cargo home
lacking the keyhive git checkout, and was wrong. app's `cargo test` is a
compile proof (0 tests); `npm run check` needs `app/node_modules`, gitignored,
copied into the worktree for the gate.

## Measured on the result (this worktree, 2026-09-05)

```
check-units.sh                 units: 4, disclaimed 7; tracked paths 357      exit 0
check-trace  org-members       REQ 12, HAZ 8, RC 10, PR 2; 2 UNRESOLVED-PR    exit 0
check-trace  on-chain-client   all zero, sources present                       exit 0
check-trace  org-node          all zero, sources present                       exit 0
check-trace  app               all zero, sources present                       exit 0
check-ids    ×4                                                                exit 0
check-trace.sh (bare, root)    "no root config to read"                        exit 2
new-id.sh REQ (root, no unit)  "not inside any declared unit"                  exit 2
new-id.sh --unit app HAZ       a six-character HAZ token (minted, discarded;
                               not reproduced here so it cannot read as a
                               reference)                                      exit 0
```

The two `UNRESOLVED-PR` lines are the same two open problems the root config
reported before the change.

## Adoption order (re-cut per unit)

Each tooth is its own worktree change. A tooth only ever tightens.

| # | Tooth | Why this order |
|---|---|---|
| 1 | **This change.** Scripts to `e2eac86`; manifest; four unit configs and ledger sets; ledgers relocated into `org-members/docs/`; CI reshaped; `.gitignore`; ADR for units and classes; setup checklist re-cut. | Mandatory first tooth of D9: manifest + per-unit configs + disclaimers. |
| 2 | **DONE** (change `worktree-guardrails-edges`, plan `docs/plans/2026-09-06-dependency-edges.md`; merge date in the squash commit). **Declare the four `depends_on:` edges**, one change per edge or per consumer, each preceded by the dependency assessment in `grill-requirements` ("Declaring a dependency"): the provider's exports (none yet — this is where org-members decides what to `exported: yes`), its RMF, its ADRs, its open problems, its SOUP. Gaps become `expects:` items in the consumer's SRS. | Everything downstream (class floor, impact set, expectations, redistribution) reads these edges; nothing does until they exist. |
| 3 | **org-node DONE** (change `worktree-guardrails-org-node-risk`, plan `docs/plans/2026-09-09-org-node-risk-analysis.md`; merge date in the squash commit): six HAZ, nine RC, fifteen REQ with relocated and new tests, PR-hvg2dy moved plus four new reports, five open in the unit; on-chain-client and app remain. **Per-unit risk analysis** (`analyze-risks`) for org-node, then on-chain-client, then app. The eight org-node hazard mentions in org-members' RMF are re-analysed as org-node's own HAZ/RC, with expectations on org-members where a control is owed there. PR-hvg2dy moves to org-node's ledger at the same time (see conflict 3). Re-check `problem_open_max` per unit. | Cross-unit references need edges (tooth 2) and a re-analysis, not a file move — measured on this change. |
| 4 | **Architecture + SOUP + LLRs per unit**, org-members first (old tooth 6, now scoped). Class C requires LLRs per software item. The root `Cargo.lock` is still gitignored — either it becomes tracked or each unit's SOUP table carries versions. | Larger under class C; needs the unit boundaries settled first. |
| 5 | **Coverage for org-node and app**: `coverage-org-node` Makefile target and `coverage_command`; the app has no tests to measure until it has tests. Decision coverage for every unit remains the setup checklist's open item. | Class C mandatory requirement, currently unmet in two of four units and half-met in the other two. |
| 6 | **Widen `strict_paths`** inside units as items arrive: `app/src` (SvelteKit) with its first TS-side item; `org-node/src/bin` is already covered. | Follows the work. |
| 7 | **`on-chain` as a unit** (Foundry, `forge test`), or a dated decision to keep it disclaimed. | Separate toolchain; deciding it needs someone who runs forge. |
| 8 | **Problem-log backfill per unit** (old tooth 8). | Late on purpose: budgets and backlog are sized together. |
| 9 | **Glossaries**: root `docs/CONTEXT.md` is now the interface glossary (D13); `org-members/docs/CONTEXT.md` takes the crate-internal vocabulary from `org-members/AGENTS.md` (old tooth 9). | Pure migration; no gate depends on it. |

## Not in scope for this change

Declaring any dependency edge; editing any item; annotating any test;
touching `on-chain`, `quint`, or the spikes; running app's `verify_commands`
in CI (the cargo entry needs Tauri's Linux system packages, the npm entry
`npm --prefix app ci` first); tightening CI signing to `--strict`;
publishing the restriction on use (owner's item, carried over).
