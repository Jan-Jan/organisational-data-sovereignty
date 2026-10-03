# Quint conformance part 2 — org-node Implementation Plan

**Goal:** move org-node's Quint protocol models into `org-node/quint/` and make their checks (typecheck plus the five simulator invariants with explicit `max_steps`) unconditional entries in org-node's `verify_commands` — ADR `docs/adr/2026-10-03-quint-conformance-gate.md` decisions 1, 3 and 13, part 2.
**Implements:** none (gate configuration and file placement; no REQ/RC/SDD/LLR changes). Decision 13's known gap stays open: no quint-connect conformance test for org-node — that is the separate node-level-model change.
**Safety class:** C (`org-node/.guardrails/config.yaml`; no per-item overrides).
**Verification:** org-node's `verify_commands` as rewritten in T1:

```
cargo test -p org-node --features app,test-support --lib --test service_stories --test transport_handshake --test transport_networked --test fuzz_envelope_decode --test fuzz_verify_against_chain --test verify_against_chain --test wire_frame_bound --test store_at_rest --test admission_sender
quint --version
quint typecheck org-node/quint/protocol.qnt
quint typecheck org-node/quint/ods_instances.qnt
quint run org-node/quint/protocol.qnt --invariant=forkSafety --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=revocationSafety --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=revokedExcludedFromOrgSecret --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=tauWindow --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=convergence --max-steps=16 --max-samples=5000
```

plus org-members' `verify_commands` (its `quint typecheck quint/protocol.qnt` line is removed; org-node is a dependent of org-members, so `check-units.sh --impact` runs org-node's gate — and with it the protocol typecheck — on every org-members change).

Sandbox note: as in part 1, run quint and cargo through the scratchpad `env.sh` wrapper (`$HOME/.quint` is read-only here).

### T1 — Move the protocol models into org-node and gate them

**Files touched:**
- `quint/protocol.qnt` → `org-node/quint/protocol.qnt`
- `quint/ods_instances.qnt` → `org-node/quint/ods_instances.qnt`
- `quint/README.md` → `org-node/quint/README.md`
- `org-node/.guardrails/config.yaml`
- `org-members/.guardrails/config.yaml`
- `.guardrails/units.yaml`
- `.github/workflows/quint.yml`
- `org-members/quint/README.md`
- `docs/adr/2026-10-03-quint-conformance-gate.md`

**Parallel:** no (single task).
**Trace:** none — gate configuration (ADR decisions 1, 3, 13).

1. `git mv` the three files into `org-node/quint/`; the `quint/` directory is then empty and gone.
2. In `org-node/quint/protocol.qnt` the import becomes
   `import membership_types.* from "../../org-members/quint/membership_types"`.
   `ods_instances.qnt`'s `import protocol.* from "./protocol"` is unchanged.
3. **Red → green (path evidence).** Before step 2, `quint typecheck org-node/quint/protocol.qnt` must fail on the unresolved import; after it, pass.
4. `org-node/quint/README.md`: every `quint/protocol.qnt` / `quint/ods_instances.qnt` path becomes `org-node/quint/…`; the opening line states the models live in org-node since part 2 and that org-node has no conformance test yet (ADR decision 13).
5. `org-node/.guardrails/config.yaml`: replace `verify_commands` with the nine commands above; add a dated comment (ADR decisions 3, 7, 13; measured runtime of the five simulator runs).
6. `org-members/.guardrails/config.yaml`: remove `quint typecheck quint/protocol.qnt` and the comment sentence about it; say instead that org-node's gate typechecks the importer as a dependent.
7. `.guardrails/units.yaml`: remove the `quint` entry from `not_a_unit`.
8. `.github/workflows/quint.yml`: repoint every `quint/protocol.qnt` / `quint/ods_instances.qnt` path (simulator and Apalache jobs).
9. `org-members/quint/README.md`: repoint any reference to `quint/protocol.qnt`.
10. ADR: dated note under decision 13 — part 2 delivered; the conformance gap for org-node remains.
11. **Verify.** All org-node and org-members `verify_commands` (counts from output); `check-units.sh` and `check-units.sh --impact master..HEAD`; `check-trace.sh` and `check-ids.sh` per impacted unit; `git grep -n "quint/protocol\|quint/ods_instances" -- . ':!docs/plans' ':!docs/verification' ':!docs/superpowers'` shows only `org-node/quint/…` paths (and the ADR's historical narration).
12. Commit: `build(org-node): protocol models into org-node/quint, gated in org-node's verify_commands`.

## Self-review

1. Implements: none — no ID to verify; evidence is the gate commands and the red→green import-path check.
2. Every step names exact paths and commands.
3. Names consistent with part 1 (`membership_types`, `org-node/quint/`).
4. One task, no parallelism.

## Status

**T1: done** (merged from `worktree-quint-conformance-part2-t1` @ aa87355).
- red -> green: `quint typecheck org-node/quint/protocol.qnt` — after the move, before the import fix: QNT013 "could not load '../org-members/quint/membership_types'", QNT405 "Module 'membership_types' not found", exit 1; exit 0 after the import fix (ods_instances typecheck also 0).
- result: org-node verify_commands 9/9 pass — cargo 53 passed; quint 0.33.0; two typechecks; five simulator invariants "[ok] No violation found", ~1 s each (rust backend; ADR decision 3's ~60 s was quint 0.32.0/typescript — note added). org-members verify_commands 7/7 pass (150 passed, 1 ignored; quint test 24). check-units (4 units, 6 disclaimed), check-trace and check-ids pass for all four units.
- note: `quint/.gitignore` also moved (so `quint/` is gone). Outside T1's file list, `app/docs/risk/2026-09-14-app-hazards.md` got a dated amendment: its scope list still named `quint/` as disclaimed.
- Gate round 1 (tree d014418) PASS; independent review PASS with 2 record findings (README), fixed on the change branch; gate round 2 (tree 5652bc1) PASS. Record: `docs/verification/2026-10-03-worktree-quint-conformance-part2.md`.
