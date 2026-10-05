# app — architecture, SOUP and low-level requirements

Tooth 4 of the ratchet (`docs/plans/2026-09-05-ratchet-gap-analysis.md`, which
numbers it 6) for the **app** unit: the first architecture ledger, a measured
SOUP inventory, and the low-level requirements class C asks for. org-members,
on-chain-client and org-node are done; app is the last.

Branch: `worktree-guardrails-app-arch`, from local `master` at `06c357b`.
Unit config: `app/.guardrails/config.yaml` (class C).

**Deadline.** The five open app problem reports were opened 2026-09-14 and
pass `problem_age_days: 30` on **2026-10-14**. This change must reach its
squash before then.

## Owner rulings (2026-10-05)

1. **Test scope.** LLRs only where a gated test can redden them. New Rust tests
   only where the existing harness already reaches (command handlers on
   `MockChainOps`). The startup wiring, chain-connection setup, `api.ts` and
   the Svelte components become **one declared no-LLR deviation item**, as
   org-node's SDD-z85ux9 is, booked for a later change.
2. **Defects found.** Booked, not fixed (the 2026-10-04 ruling), and pinned by
   a test asserting today's behaviour where one can be. The badge race and the
   tab-switch event loss are addenda to PR-h6xpnh (same root cause). New
   reports: devalue 5.8.1 advisory, `subscribeAll` listener leak, Revoke's
   `'networked'` default on a failed status fetch, two ✓ rows per update, and
   REQ-vgr7s2's premise (org-node's join request carries an address whenever
   an endpoint is bound). `problem_open_max` is raised to **15**, as org-node's
   was.
3. **Onboarding.** The CreateOrg/Invite/Admit/PersonaList panels and the
   create-organisation, invite and join-request commands go into the deviation
   item. Their REQs are minted by the Gap-20 first-admission app change.
4. **CSP — fixed here.** "Add a CSP." `tauri.conf.json` sets
   `app.security.csp: null`. This is the one production change: a REQ and RC
   from `analyze-risks`, a test that reddens on `csp: null`, then the policy.

## Inputs (scratchpad, measured at `06c357b`)

- `scratchpad/app-arch/decomposition-survey.md` — 13 proposed items (A–M),
  82 candidate LLRs with their tests, the untested list, ten contradictions.
- `scratchpad/app-arch/soup-facts.md` — own workspace and tracked lock
  (`app/src-tauri/Cargo.lock`, 806 entries); 495 third-party crates shipped on
  the host target (521 with build/dev; 672/707 for all targets); npm lock 119
  packages, but the dev flag does not mean unshipped; no tauri plugins;
  `core:default` only; no cargo-audit/cargo-deny; `npm audit` 5 findings
  (3 high), 0 with `--omit=dev`.

## Tasks

Common rules for every task:

- Mint every ID with `.guardrails/scripts/new-id.sh --unit app <PREFIX>`.
  Never invent one.
- Gates for the `app` unit take `GR_CONFIG=app/.guardrails/config.yaml`.
- Use `CARGO_HOME=/tmp/cargo_home_fuzz` (`~/.cargo` is read-only), never
  `--offline`. The npm entries need `npm --prefix app ci` in a fresh worktree.
- In a worktree-pinned session, run one plain git command per shell call.
  Put loops in a script file under the scratchpad and run it with `sh`.
- The survey and facts files live in the shared scratchpad
  `/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/f37ae890-21a2-46ff-8435-6bb5ec6d682a/scratchpad/app-arch/`.

### T1 — SOUP inventory (done)

**Files touched:** `app/docs/architecture/soup.md`,
`org-node/docs/architecture/soup.md` (one dated correction paragraph only)
**Parallel:** yes (with T2)

Write `app/docs/architecture/soup.md` from `soup-facts.md`. Follow the
structure of `org-node/docs/architecture/soup.md` but keep it short. It needs
these sections:

- **Evidence and provenance.**
  - app/src-tauri is its own workspace, and its lock
    `app/src-tauri/Cargo.lock` is tracked.
  - Closure: 495 third-party crates shipped on the host target, 521 with
    build and dev edges. The all-targets figures are 672 and 707 before the
    four units are subtracted, 668 and 703 after (T1 report). Name the
    target, the command, and that only `name version` pairs were counted.
  - Name the four repository units that were subtracted.
  - The npm lock has 119 packages. The `dev` flag does not mean the package
    is unshipped: the svelte and kit runtimes and devalue are compiled into
    `build/`.
  - No cargo-audit or cargo-deny. Give the `npm audit` results: 0 with
    `--omit=dev`, 5 in full.
- **Rust direct dependencies.** Normal, build and dev crates, exact versions,
  role, the modules that use each, the app REQs each supports, and anomalies.
  - Say that subxt and on-chain-client are declared only to unify features.
  - `rand` OsRng is used in production in `commands.rs`.
  - `postcard` decodes untrusted EndpointAddr bytes.
  - `subxt-signer` builds the admin key.
- **npm packages.** The one runtime dependency, plus the dev packages whose
  code is shipped in the bundle.
- **Inherited SOUP.** These rows are at the same versions as org-node's
  inventory: cite that file and don't duplicate it. Name the version skew:
  when the app builds on-chain-client, it uses the app lock's subxt 0.50.3,
  not 0.50.1.
- **Tauri configuration.** No plugins; `core:default` only; the 12 commands
  are ungated by any ACL.

In `org-node/docs/architecture/soup.md`, add one dated paragraph,
"*Corrected 2026-10-05 …*", right after the sentence at lines 34–36. Its
claim that the root lock covers app's Rust side and on-chain-client is wrong:
both have their own locks. Leave the original sentence unedited.

This task writes no test, so it has no red → green.

### T2 — Content-Security-Policy (production fix, owner ruling 4) (done)

**Files touched:**
- `app/docs/requirements/2026-10-05-csp.md` (new)
- `app/docs/risk/2026-10-05-csp.md` (new)
- `app/src-tauri/tests/csp_policy.rs` (new)
- `app/src-tauri/Cargo.toml` (`[[test]]` entry)
- `app/src-tauri/tauri.conf.json`
- `app/.guardrails/config.yaml` (`verify_commands` cargo line and its comment)

**Parallel:** yes (with T1)

Run `analyze-risks` for one new hazard. `csp: null` lets any script that
reaches the webview call all 12 ungated Tauri commands, revoke and admit
among them, through `core:default` IPC. Write the HAZ and its RC in the risk
draft, and the REQ in the requirements draft. The REQ is a shall statement:
the software ships a CSP with these properties.

- `default-src 'self'`.
- `script-src` allows no `'unsafe-inline'`, no `'unsafe-eval'` and no
  remote origin.
- `object-src 'none'`, `base-uri 'none'`, `frame-ancestors 'none'` and
  `form-action 'none'`.
- `connect-src` allows only Tauri IPC (`ipc:` and `http://ipc.localhost`).

Red first. `csp_policy.rs` (`required-features = ["test-support"]`) reads
`tauri.conf.json` with serde_json and parses `app.security.csp` into its
directives. Normal case: the shipped policy meets every clause. Abnormal
cases: the checker rejects `null`, `'unsafe-inline'` or `'unsafe-eval'` in
`script-src`, a `*` or `https:` source, and a missing `object-src`. Watch the
normal-case test fail on `csp: null`, then set the policy:

```
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
img-src 'self' data:; connect-src ipc: http://ipc.localhost;
object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'
```

Set `devCsp` as well, so that `npm run tauri dev` can still reach the Vite
dev server. The dev server needs `http://localhost:5173` and
`ws://localhost:5173` in `connect-src` and `script-src`. Set `devCsp` only
under the dev config key; the test asserts that the shipped `csp` contains
no localhost source.

Every test carries `verifies:` with the new REQ, or the RC where it is the
lowest level. Add `--test csp_policy` to the cargo line in `verify_commands`
and update the count comment. Nothing in the gate shows that the bundled app
still loads under the CSP. Say so in the risk draft as a verification
limitation, and leave it as the owner's smoke test (`npm --prefix app run
tauri build`).

### T3 — Decomposition, LLRs and test annotations (done)

**Files touched:**
- `app/docs/architecture/2026-10-05-decomposition.md` (new)
- `app/docs/risk/2026-10-05-derived.md` (new)
- `app/src-tauri/tests/*.rs`
- `app/tests/*.ts`
- `app/src-tauri/Cargo.toml`
- `app/.guardrails/config.yaml` (`verify_commands` only, if a new target is added)

**Parallel:** no (after T2)

**Items.** Take the items from `decomposition-survey.md` §1:

- A–H, J, K and L, plus one item for the CSP. Its LLRs are T2's tested
  clauses, and T2's tests are re-annotated to them.
- I, M, and the untested wiring and chain setup (`run`/`main`, `init`,
  `build_chain_ops`, `connect_chain`, `ChainNotConfigured`) together make
  one **deviation item, with no LLRs**. It traces to the panels' REQs, and
  its text states the deviation, the reason (no gated test reaches it, by
  owner ruling 1) and the booking for the later change.
- Merge or split survey items only when an item would otherwise trace to no
  REQ.

**LLRs.** An LLR is written only where a gated test reddens when the LLR's
behaviour breaks. A candidate with no test either gets a new test now, if
the existing harness reaches it (command handlers on `MockChainOps`, the
`events.rs` vocabulary, the `parsing.rs` paths), or moves into the deviation
item's text.

- **Do not edit production source.** If a test needs a seam that does not
  exist, record the candidate as deviation and say so in the report.
- Each LLR is `satisfies: REQ-…` or `satisfies: derived`.
- Every derived LLR gets an `assesses:` line in the derived risk draft,
  naming the HAZ it was assessed against, or "no hazard" with the reason.

**Tests.** Every gated test carries `verifies:` with the lowest level that
exists, which is its LLR or LLRs. Keep the REQ annotations only where no LLR
exists. Each LLR needs a normal-case test and an abnormal-input test (class
C). New tests follow red → green: a characterization test is red only by a
deliberate mutation of the code it pins. Record that mutation and restore it
byte-identical, checked by SHA-256.

**Gate.** `check-trace.sh` and `check-ids.sh --allow-draft-files` must exit
0 for the app unit with no findings this task introduced.

### T4 — Problem reports (owner ruling 2) (done)

**Files touched:**
- `app/docs/problems/2026-10-05-problems.md` (new)
- `app/docs/problems/2026-09-14-app-problems.md` (dated addendum to PR-h6xpnh only)
- `app/.guardrails/config.yaml` (`problem_open_max` and its comment)
- the test files a pinning test needs

**Parallel:** no (after T3)

**Handoff from T3.**
- The decomposition and derived-risk drafts refer to "task T4 of the plan"
  for the duplicate incoming-verified event (LLR-p2nm5a) and the
  subscribeAll leak (LLR-z4ky6f). Replace those references with the new PR
  IDs.
- A pin that already exists as an LLR test (LLR-p2nm5a pins the duplicate
  event) needs no second test. Add `PR-…` to that test's `verifies:` line.
- The app has no MockChainOps. `AppState::for_test` installs
  ChainNotConfigured.

Book five new reports, each with `opened: 2026-10-05` and `status: open`:

- devalue 5.8.1 high advisory, in the shipped bundle;
- the `subscribeAll` listener leak when one `listen` rejects
  (`receiver.ts:17-19`);
- Revoke keeping `'networked'` when the status fetch fails
  (`Revoke.svelte:45,62-68`);
- two ✓ rows per update (`events.rs:194-207` emits both events to
  `recordVerified`);
- REQ-vgr7s2's premise, contradicted by `org-node/src/service.rs:723-728`.

Add a dated addendum to PR-h6xpnh covering the receiverStarted race
(`Membership.svelte:165-166`) and the event loss on a tab switch
(`+page.svelte:51-63`).

**Pinning tests.** Pin each report with a test that asserts today's
behaviour and carries `verifies: PR-…`, where the existing harness reaches
it (vitest for `subscribeAll`, cargo for `emissions_for`). For a report with
no pin, state why it has none.

**Limit.** Set `problem_open_max: 15`, with a comment citing owner ruling 2
and its date. Verify the open count with check-trace.

### T5 — Falsifiability sweep (done)

**Files touched:** the decomposition draft (cuts only); the scratchpad
`app-arch/sweep.md` (the attestation table)

**Parallel:** yes (with T6)

Decompose every LLR into clauses that can each be falsified on their own.
Apply one source mutation per clause and run the named tests.

- **RED:** record the tests that failed.
- **GREEN:** cut the clause from the LLR, or narrow it until it is falsifiable.

Copy every file from a pristine copy before mutating it, restore it
afterwards, and prove it byte-identical by SHA-256. GREEN is a verdict about
this search, not about the clause. `sweep.md` holds one row per run: LLR,
clause, mutation, verdict, failing tests. It becomes the record's appendix.

### T6 — Overview (done)

**Files touched:** `app/docs/architecture/README.md` (Overview section only)
**Parallel:** yes (with T5)

Write the standing decomposition picture. It covers:

- the items and their interfaces;
- the IPC boundary between the Rust and frontend halves;
- the deviation item;
- that the unit's figures live in the verification record, not here.

### T7 — Gate, review, merge

`verify-before-merge`, then `merge-change`. Impact set: `app` touched. The
CSP smoke test is the owner's, and must be done before the squash.

## Red → green attestations

(filled in as each task report arrives)

- T1: none — a documentation task, no test.
- T2 (HAZ-e2zupz, RC-mq2365, REQ-cj5jmx; 17 tests in `csp_policy.rs`):
  - shipped_csp_meets_every_clause — watched fail on `csp: null` ("policy must be a string, found null") before the policy was set.
  - dev_csp_adds_only_the_dev_server_origins — watched fail (devCsp absent) before devCsp was set.
  - The 15 checker tests were green on first run (the checker lives in the test file). Each was reddened by a deliberate mutation of the checker, M1–M9, with the file restored byte-identical (SHA-256 72cbb6d8…c1d319). M1 (check → Ok) reddened all 15. M10 (empty connect-src accepted) reddened nothing, so that check was cut.
- T3 (11 SDD, 45 LLR of which 13 derived). Every new test is a characterization test. It passed on the unchanged code, then was watched fail under the named mutation. The file was restored, and SHA-256 proved it byte-identical in every run. Raw records: scratchpad `appt3-mut-results.jsonl`. Four tests were renamed after T5 from `…_hands_…_to_the_service` to `…_does_not_refuse_…`, because the service never uses what they pass (see T5); the raw records carry the old names.
  - refusals_for_empty_values_name_both_variables_supplier_first — LLR-97rww8 — M1 policy.rs message reworded
  - the_store_is_opened_inside_the_data_dir_it_creates — LLR-85zque — M2 store file renamed
  - a_data_dir_that_cannot_be_created_is_refused_naming_it — LLR-85zque — M3 create_dir_all error ignored
  - a_store_under_another_passphrase_is_refused — LLR-85zque — M4 fresh store on open failure
  - connection_status_reports_the_directory_the_store_was_opened_in, …_a_data_dir_it_had_to_create_verbatim — LLR-vqkr5t, LLR-85zque — M5 data_dir.join(".")
  - export_invite_does_not_refuse_a_well_formed_org_id — LLR-vzf8j2 — M6 org id sliced
  - revoke_member_accepts_a_zero_x_prefixed_member_id — LLR-6pmrma — M7 no 0x strip
  - revoke_member_treats_a_whitespace_only_peer_addr_as_absent — LLR-ty85xv — M8 no trim
  - revoke_member_does_not_refuse_an_encoded_endpoint_addr — LLR-n6twt7 — M9 decode offset
  - revoke_member_refuses_a_peer_addr_that_is_not_hex — LLR-n6twt7 — M10 hex error swallowed
  - revoke_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr — LLR-n6twt7 — M11 decode failure dropped
  - admit_member_does_not_refuse_a_32_byte_or_absent_org_secret — LLR-8krgzj, LLR-ctrfz4 — M12 length 31
  - admit_member_refuses_an_org_secret_that_is_not_hex — LLR-8krgzj — M13 zero secret on hex error
  - admit_member_does_not_refuse_a_decodable_node_addr — LLR-ctrfz4 — M14 decode offset
  - admit_member_refuses_a_node_addr_that_is_not_an_endpoint_addr — LLR-ctrfz4 — M15 short addr takes id-only branch
  - list_personas_reports_exactly_the_persona_fields — LLR-pguhw5 — M16 serde rename
  - a_persona_in_no_organisation_reports_a_null_org_id — LLR-pguhw5 — M17 null → ""
  - import_join_request_reports_the_request_it_decodes — LLR-pmus9f — M18 has_node_addr threshold
  - import_join_request_refuses_a_malformed_blob — LLR-pmus9f — M19 decode error replaced
  - updated_carries_boundary_epochs_and_roots_verbatim — LLR-p2nm5a — M20 epoch.max(1)
  - record_unreadable_holds_for_any_organisation_id — LLR-hgsdm8 — M21 org id trimmed
  - dev_policy_rejects_an_origin_beyond_the_dev_server — LLR-tc5aax — M22 checker origin test inverted
  - "keeps the organisation a failed event names" — LLR-mzae5q — M23 placeholder always
  - "files the newest announcement first in each list" — LLR-rzx6ks — M24 appended last
  - "keeps at most 50 verification rows, dropping the oldest" — LLR-rzx6ks — M25 cap 51
  - "keeps at most 20 receiver errors, dropping the oldest" — LLR-rzx6ks — M26 cap 21
  - "rejects a doubled 0x prefix" — LLR-csbs5v — M27 /^(0x)+/
- T4 (PR-8vh53d, PR-cu2h2g, PR-uxj7bg, PR-bu6mau, PR-7zz76t; PR-h6xpnh addendum; open 10, limit 15):
  - "leaves a registered listener attached when another registration fails (PR-cu2h2g)": a characterization test. It was watched fail under a mutation of `receiver.ts` that applies the fix (allSettled, cancel, rethrow). The file was restored byte-identical (SHA-256 041be48d…e9c113f).
  - updated_emits_membership_incoming_and_epoch (now also `verifies: PR-bu6mau`): watched fail with the incoming-verified emission renamed. Restored byte-identical (SHA-256 921a7bf4…71f60df2).
  - Unpinned: PR-8vh53d, because no gate runs an audit. PR-uxj7bg and the PR-h6xpnh addendum, because they are component behaviour and no gated test mounts a component. PR-7zz76t, because the behaviour is org-node's.
- T6: none — a documentation task, no test.
- T5: no test added. 277 clauses: 261 RED, 16 GREEN, all 16 cut or narrowed (10 LLRs). 266 new runs plus 26 reused from T3; 266/266 restored byte-identical (SHA-256). 8 of the GREENs have one cause: on `AppState::for_test`, admit and revoke fail at `find_org` before anything reaches the service. The table is in scratchpad `app-arch/sweep.md`, and it becomes the record's appendix.

### R1 — Review round 1 fixes

**Files touched:**
- `app/docs/requirements/2026-10-05-csp.md`
- `app/docs/architecture/2026-10-05-decomposition.md`
- `app/docs/risk/2026-10-05-derived.md`
- `app/docs/architecture/soup.md`
- `app/.guardrails/config.yaml` (comments only)
- `app/src-tauri/tests/csp_policy.rs`
- `app/src-tauri/tests/ipc.rs`
- `app/src-tauri/tests/connection_status.rs`
- `app/tests/receiver.log.test.ts`

Production source is not changed.

The findings are verbatim in the verification record. Each fix below follows
red → green. A strengthened test must fail under the reviewer's probe
mutation before it is accepted, and every mutated file is restored
byte-identical (SHA-256).

1. **finding-1:** allow-list the directives.
   - REQ-cj5jmx gains a clause: the shipped policy names no directive except
     `default-src`, `script-src`, `style-src`, `img-src`, `connect-src`,
     `object-src`, `base-uri`, `frame-ancestors` and `form-action`.
   - Give that clause a new LLR under SDD-32hath, or fold it into LLR-hdvy6x.
   - The checker rejects any other directive name.
   - Add abnormal tests for `script-src-elem`, `script-src-attr` and
     `worker-src`.
   - Red: run the reviewer's `script-src-elem 'self' 'unsafe-inline'`
     addition against the strengthened checker.
2. **finding-2:** test the dev policy as a difference.
   - Restore LLR-tc5aax to REQ-cj5jmx's dev clause: `devCsp` equals `csp`
     directive for directive, except that `script-src` and `connect-src`
     each add exactly the dev-server origins.
   - Add a test that parses both policies and compares them.
   - Red: run the reviewer's `devCsp` probes.
3. **finding-3:** gate the org-id refusal on the other two commands.
   - Add tests that `admit_member` and `revoke_member` refuse a malformed
     `orgId` with the parser's message (abnormal), and do not refuse a
     well-formed one (normal).
   - Widen LLR-vzf8j2 back to all three commands.
   - Red: run the reviewer's `unwrap_or` probe on each command.
4. **finding-4:** gate the product registration.
   - Add a test that reads `app/src-tauri/src/lib.rs` and asserts that the
     `generate_handler!` list equals the harness's list in `tests/ipc.rs`,
     as a set of names. The test reads source text; it does not edit it.
   - Restate LLR-4wcyqy to say what is gated.
   - Correct the claim in `2026-10-05-derived.md:108-115`.
   - Red: delete `commands::revoke_member` from `lib.rs`.
5. **finding-5:** assert the message itself.
   - `import_join_request_refuses_a_malformed_blob` asserts the message that
     org-node's decoder returns for the input, not `contains("blob")`.
   - Red: run the reviewer's `"invalid blob"` probe.
6. **finding-6:** match the exact message.
   - `revoke_member_rejects_a_short_member_id` asserts LLR-6pmrma's exact
     message.
   - Red: run the reviewer's rewording probe.
7. **finding-7:** annotate the tests that already verify the clauses.
   - The receiver.log tests that check the `[<timestamp>] <message>` format
     gain `LLR-fb7jp5`.
   - `configured_chain_reports_its_endpoint_and_contract` gains
     `LLR-kze6ak`.
   - Each LLR's Normal list in the decomposition names those tests.
8. **finding-8:** inventory the CSP's SOUP.
   - The tauri row cites REQ-cj5jmx and RC-mq2365. Tauri injects the policy
     and appends hashes of the inline scripts.
   - Add a row for the platform webview: WKWebView on macOS, WebView2 on
     Windows, WebKitGTK on Linux. It is supplied by the OS, its version is
     not pinned, and it enforces the CSP.
   - The Tauri configuration section mentions the CSP.
9. **finding-9:**
   - The tempfile row also names `state_assembly.rs`.
   - The `test_paths` comment says eight cargo targets.

Gate: check-trace and check-ids exit 0 for the app unit, and every app
verify command passes.

R1 (done) red → green:
- script_src_elem_is_rejected, script_src_attr_is_rejected, worker_src_is_rejected (LLR-uus6ar): all three failed before the checker's directive allow-list existed.
- shipped_csp_meets_every_clause (now also LLR-uus6ar): reddened by the reviewer's `script-src-elem 'self' 'unsafe-inline'; script-src-attr 'unsafe-inline'` addition.
- dev_policy_that_differs_beyond_the_dev_origins_is_rejected (LLR-tc5aax): failed against a stub comparator that returned `Ok(())`.
- dev_csp_differs_from_csp_only_by_the_dev_server_origins (LLR-tc5aax): reddened by each of the reviewer's devCsp probes.
- dev_fixture_is_a_pure_addition_to_the_fixture (LLR-tc5aax): the comparator's normal case; it has no red of its own.
- admit_member_refuses_a_malformed_org_id_with_the_parsers_message and revoke_member_refuses_a_malformed_org_id_with_the_parsers_message (LLR-vzf8j2): each reddened by the reviewer's `unwrap_or(OrgId::new([0xaa;20]))` probe on its command. Their normal counterparts have no red of their own.
- product_registers_the_same_commands_as_the_harness (LLR-4wcyqy): reddened by deleting `commands::revoke_member` from `lib.rs`.
- import_join_request_refuses_a_malformed_blob (strengthened, LLR-pmus9f): reddened by the `"invalid blob"` probe.
- revoke_member_rejects_a_short_member_id (strengthened, LLR-6pmrma): reddened by the rewording probe.
- Every probe restored its file byte-identical (SHA-256). The scripts are in scratchpad `app-arch/r1/`.

### R2 — Review round 2 fixes (last round: all findings low or record)

**Files touched:**
- `app/src-tauri/tests/ipc.rs`
- `app/docs/architecture/2026-10-05-decomposition.md`
- `app/docs/risk/2026-10-05-derived.md`
- `app/docs/risk/2026-10-05-csp.md`
- `app/.guardrails/config.yaml` (count comment only, if a count moves)

Production source is not changed. The findings are verbatim in the record.

1. **finding-1:** strengthen the `export_invite` refusal tests.
   - `export_invite_rejects_a_short_org_id` and
     `export_invite_rejects_a_non_hex_org_id` assert equality with
     `parser_refusal(&bad)`, as the admit and revoke tests do.
   - Red: run the reviewer's fixed-message probe at `commands.rs:130`.
2. **finding-2:** pin the single registration.
   - `product_registers_the_same_commands_as_the_harness` also asserts that
     `lib.rs` contains exactly one `generate_handler!` and exactly one
     `invoke_handler` outside comments.
   - Red: run the reviewer's second-`invoke_handler` probe.
   - Update LLR-4wcyqy and the limit note in `derived.md` to match.
3. **finding-3:** gate the "names the blob" clause.
   - Keep a `contains("blob")` assertion beside the equality in
     `import_join_request_refuses_a_malformed_blob`, so LLR-pmus9f's clause
     is gated.
   - Red: temporarily reword org-node's import_join_request decode error
     (in `org-node/src`) so it no longer names the blob. The equality stays
     green, so the new `contains` must fail. Restore the file byte-identical.
4. **finding-4:** update the counts in `derived.md`: 46 LLRs, 33 refine a
   requirement, 13 derived. Measure them by grep.
5. **finding-5:** `2026-10-05-csp.md` §3 names the directive allow-list
   (LLR-uus6ar) and the devCsp-versus-csp comparison (LLR-tc5aax).

R2 (done) red → green. No new test was added; three tests were strengthened:
- export_invite_rejects_a_short_org_id and export_invite_rejects_a_non_hex_org_id (LLR-vzf8j2): both reddened by the reviewer's fixed-message probe on `export_invite`.
- product_registers_the_same_commands_as_the_harness (LLR-4wcyqy): reddened by the reviewer's second `invoke_handler` probe in `lib.rs`.
- import_join_request_refuses_a_malformed_blob (LLR-pmus9f): reddened by rewording org-node's blob decode errors so they no longer name the blob. The equality stayed green; the new `contains("blob")` failed.
- Every probe restored its file byte-identical (SHA-256). The scripts are in scratchpad `app-arch/r2/`.
