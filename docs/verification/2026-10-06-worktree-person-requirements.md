# Verification — Person definition (2026-10-06)

branch: worktree-person-requirements
reviewer: independent subagent, two rounds (round 1 on 342bf79, full diff, re-ran every impacted unit's suite; round 2 on dd74489, the round-1 fixes and the merge of master fdf4e77)
verdict: approve with findings, both rounds; five findings, all low, all fixed
reproduced: yes — round 1 re-ran person 122 passed, org-members 243 passed 1 ignored, org-node 186 passed, app 135 Rust and 36 vitest passed, `make coverage-person` 344/344 lines and 535/535 regions; round 2 re-ran person 123 passed. No defect was reproduced: the findings are wording and traceability, not behaviour.

Change: `person` gains the Person definition — `Person` and `Person::new`, the shared `check_group_key`, `check_successor`, a V1 hash under an encoding version with `person_hash` and `verify`, a Person-specific device sub-trie hasher, decoding routed through `Person::new`, no-panic property tests and a bolero fuzz target; org-members maps the five new `IdentityError` variants. Branched from `master` at `2569e56`; master merged in at `0de6586` (f963f93) and `fdf4e77` (dd74489), local and `origin/master` both ancestors.
Plan: `docs/plans/2026-10-06-person-definition.md` (its "Progress" section holds every task's red → green report); sequencing in `docs/plans/2026-10-04-person-sequencing.md`.

Units touched: person, org-members, app (lockfile only). Impact set (`check-units.sh --impact master..HEAD`): person, org-members, app touched; org-node dependent.

## The gate

Measured on: `dd74489` — tree `50508835cf4156390fef16c602161163499dc16a`, clean worktree, after the merge of master `fdf4e77`. The tree then moved by one documentation commit, `d3f72cb` (tree `e80cddb1b77e330debea010c160a9f9bbb9a9289`), the round-2 wording fixes to `docs/CONTEXT.md` and `docs/plans/2026-10-04-person-sequencing.md`. No code, test, build file or Quint model reads either file (the only mention is a doc comment in `org-node/src/types.rs`), so the suite figures below were not re-measured on it; `check-ids.sh`, `check-trace.sh` (all four units) and `check-units.sh` were re-run on `d3f72cb` and exit 0.

| Gate | Result |
| --- | --- |
| person `cargo test -p person` | 123 passed, 0 failed (19 binaries, fuzz target 1 s); clippy `-D warnings` clean |
| org-members `cargo test -p org-members` | 243 passed, 0 failed, 1 ignored (`preflight_probe`, by design); Quint: 3 typechecks, `quint test` 63 passing, `mbtInv` no violation |
| org-node `cargo test` (lib + 25 targets) | 228 passed, 0 failed; 3 bolero targets ran 1 s each, no failure; Quint: 2 typechecks, 5 invariants no violation |
| app `cargo test` (10 targets) | 159 passed, 0 failed |
| app `npm run check` / `npm run test` | 0 errors, 1 warning (missing `@types/node`, present on master too) / 43 passed |
| `check-ids.sh` (GR_CONFIG per unit) | exit 0 for person, org-members, org-node, app |
| `check-trace.sh` (GR_CONFIG per unit) | exit 0 for all four; this change's items all traced and tested (no MISSING-TEST); `check-units.sh` exit 0 |
| Coverage, class C (statement + decision) | person: lines 344/344, regions 535/535 (floors 99/99), nightly branches 36/36; org-members: lines 96.30%, regions 95.33% (floors 92/91), error.rs 100% |
| Working tree | clean |

An earlier gate run on `342bf79` reported person lines 98.01% (344/351). A clean measurement at the same commit gave 344/344: the extra lines came from a nightly `--branch` run's binaries left in `target/llvm-cov-target`, which `cargo llvm-cov clean --workspace` does not remove and a plain `cargo llvm-cov clean` does. The Makefile's person coverage note now says so.

The first org-members run of each fresh `QUINT_HOME` failed 11 `mbt_conformance` tests while the evaluator downloaded; every rerun passed. Not seen on the measured tree.

## Red → green

Every test was watched failing before its implementation existed; the per-test lines are in the plan's "Progress" section. By item:

| Item | Tests | Watched red |
| --- | --- | --- |
| LLR-3n3kxx | `the_person_variants_name_their_rule` and every refusal test below | E0599, no such `IdentityError` variant |
| LLR-rde6tk | `version_one_is_v1_and_back`, `every_other_version_is_refused_and_named`, `hashing_under_an_unimplemented_version_is_refused` | E0432, `EncodingVersion` missing |
| LLR-sjkmr6, LLR-kbhc43 | `group_key.rs` (4), `definition.rs` (5) | E0432, `check_group_key` / `Person` missing |
| LLR-4ku56h | `a_person_holds_up_to_max_devices_sorted`, `over_the_bound_or_a_duplicate_is_refused_before_any_person_exists` | E0432, `Person` missing |
| LLR-edn55h | `person_device_trie.rs` (5), `person_hash.rs` encoding tests (3) | E0432, hasher / `encode` missing |
| LLR-63pkfc | `definition_decode.rs` (8) | E0277, `Person` not `Serialize`/`Deserialize` |
| LLR-wqha8d | `successor.rs` (4) | E0432, `check_successor` missing |
| LLR-4ebtn4, LLR-tf45kx | `person_hash.rs` (13) | E0432, `person_hash`/`verify`/`PersonHash` missing |
| no-panic properties (LLR-sjkmr6, LLR-kbhc43, LLR-4ku56h, LLR-wqha8d, LLR-tf45kx, LLR-4ebtn4, LLR-rde6tk, LLR-63pkfc) | `no_panic.rs` (5 new) | compile red, then a temporary mutation per test (check ignored, `&&`→`||`, `verify` always true, decoder `expect`), reverted |
| LLR-63pkfc, LLR-tf45kx, LLR-wqha8d | `fuzz_person_decode` | missing target, then an injected panic in `check_successor` found in 158 ms, reverted |
| LLR-28ekrv (org-members) | `person_only_variants_map_to_invariant_violated`, `shared_variants_keep_their_names_and_fields` | E0004, non-exhaustive match; a mutation mapping one variant elsewhere failed the first |
| LLR-dtwpr8 | `debug_redacts_the_names`, `debug_redacts_names_debug_would_escape` | a temporary `Debug` impl printing the names failed both, reverted |

REQ-9m5pq2, REQ-7n4g8b, REQ-wg7z4s, REQ-ht3x78, REQ-7qgx2q, REQ-bhez2u, REQ-7ymek3 and REQ-r4keha are verified through these LLRs. LLR-edn55h has no failure input (encoding is total); its abnormal cases are the boundary definitions. LLR-28ekrv is a total mapping over a closed enum and has no abnormal input.

## What was wrong, and what was built

The Person definition requirements (written 2026-10-04) predated change 1, which put the shared identity types on master. Merging master showed five draft items duplicating master's and three conflicting with it (a second error enum, `from_bytes`, a private sentinel); they were deleted or rewritten against master's names before any code (7dc341c). The plan's T1 then broke org-members, org-node and app: five new `IdentityError` variants missed org-members' exhaustive conversion (E0004). They now map to `InvariantViolated`, the match stays exhaustive, and the later Member key-rules change revisits them.

Owner rulings recorded on this branch (2026-10-06): the PersonPublicKey ≠ DevicePublicKey check compares bytes only, and reuse through the X25519 image of an ed25519 key or through mixed-order encodings is accepted residual risk for Persons, as for Members; the fuzzing residual of HAZ-h3swmw and HAZ-y6ft54 is accepted on the fuzz target alone, with no recorded campaign. The group-key check is one public function, `check_group_key`, so the Member side can call the same code later.

## Review

**finding-1**: requirement, low — the root glossary contradicted REQ-ht3x78 ("a Person holds one in their own definition"; the Person definition entry listed the PersonPublicKey as always present), and "encoding version" and "Person hash" were undefined.
disposition: fixed in 2c0850b — both entries now make the PersonPublicKey conditional on at least one DevicePublicKey; "Person hash" and "Encoding version" added to the root glossary. Round 2 confirmed.

**finding-2**: requirement, low — prose used the avoided terms "group key" / "person key" (ADR group-key-rules, person hazards file) while code identifiers `check_group_key` and `person_key` use them.
disposition: fixed in 2c0850b — prose says PersonPublicKey; the glossary names `person_key` and `group_key` as code identifiers outside the _Avoid_ rule. Round 2 confirmed.

**finding-3**: requirement, low — `Person`'s `Debug` form was unmarked derived behaviour; its test traced only to LLR-fbqs2r.
disposition: fixed in 2c0850b — new LLR-dtwpr8 (`satisfies: derived`, assessed in the person hazards file); both tests verify it and fail under a name-printing `Debug` impl. Round 2 confirmed.

**finding-4**: requirement, low — the glossary said "only" the Person hash and Encoding version are published, excluding the epoch the on-chain entry also stores.
disposition: fixed in d3f72cb — the entry says the content is never published, only the hash and encoding version with the epoch they were set at.

**finding-5**: record — the sequencing plan's key rules still said "group key".
disposition: fixed in d3f72cb.

## Gaps

- No coverage-guided fuzzing campaign has been run; the owner accepted the residual on the fuzz target alone.
- The Person capability stays not releasable until `on-chain-person` provides latest-finalised reads and publication (HAZ-vv2c3p, HAZ-zdwra7), and key agreement decides whether a removed device can derive the new key (HAZ-mvavg7).
- org-members does not yet call `check_group_key`; Member and Person key rules share code only after the Member key-rules change.
- The byte-only key check does not catch birational or mixed-order key reuse (accepted).
- The golden hash values were pinned from this implementation's own output; no second blake3 implementation was available to cross-check them, though separate tests rebuild each from `blake3::keyed_hash`.
- org-node and app have no coverage command; their coverage is not measured.
