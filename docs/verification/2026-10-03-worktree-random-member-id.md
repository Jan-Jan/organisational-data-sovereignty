# Verification — random member identifiers in org-node (2026-10-03)

branch: worktree-random-member-id
reviewer: three rounds of independent subagent review at `merge-change` step 6a. Each round had a fresh reviewer, who was given the diff, the ledgers and the plan, but no author narrative. Each reviewer re-ran the org-node suite (round 3 the app suite too) in its own nested worktree and measured its own mutations.
verdict: merge. Round 1 found three code findings (two surviving mutants, B2 and C3, and an encode-after-submit ordering) together with record findings. All were fixed, and round 2 killed every non-equivalent mutant (13 caught; 3 equivalent survivors). Rounds 2 and 3 raised record findings only. Under the convergence rule, round 2 was already the last review round. Round 3 was dispatched anyway, raised record findings only, and those were fixed. No reviewer read the round 3 corrections.
reproduced: yes — every refusal and property was watched failing before it held. `member_ids_are_not_derived_from_keys` failed with "the admin's id is its member key". `readmission_with_same_keys_gets_a_fresh_member_id` failed with "round 0: id is B's member key". `fuzz_first_admission_base`'s explicit-error assertion panicked with "first admission without a snapshot refused for the wrong reason: Trie(DuplicateKey)". `same_persona_founding_two_organisations_gets_two_admin_ids` failed under mutant B2 with "the same keys founding two organisations produced the same admin id". `first_admission_without_a_record_snapshot_is_refused` failed under mutant C3 with "got Err(Trie(DuplicateKey))".

Change: org-node gives every Member it places in a Membership record a `MemberId` drawn from its cryptographic random source before the record is built: the founding administrator in `create_organisation`, and each joiner in `admit_member`. It never computes one from a key, and it refuses a first admission that arrives without the snapshot of the record it extends (REQ-d9g6nt). `member_id_from_key` is deleted. On the send side the record snapshot is encoded before the chain submit, and an encode failure is an error. Branched from `master` at `a547ff3`; `master` (`f5e16d2`) merged in after the first squash attempt — see Base.
Plan: `docs/plans/2026-10-03-random-member-id.md`.

Units touched: `org-node` (code, tests, fuzz target, config, ledgers), `org-members` (README item 11, `trie.rs` doc-comments, requirements/risk/problems prose — no code) and `app` (comments only, in `events.rs` and `receiver_events.rs`). Impact set, from `check-units.sh --impact master..HEAD`: all three are touched.

Base: `git fetch origin` succeeded on every round, but `origin/master` was `a547ff3`. Local `master` was at `f5e16d2` (the dropped-errors change, committed locally and not pushed). Step 1 merged `origin/master`, which did nothing, so rounds 1–4 of the gate and all three reviews ran without `f5e16d2`. **This record first said `origin/master` equalled local `master`; that was wrong.** The author read `git log -1 origin/master master`, which prints only the newer commit, as both refs being equal. `finish-merge.sh` caught it after the owner signed the squash: "HEAD and worktree-random-member-id differ". Local `master` (`f5e16d2`) was then merged into the branch. The two changes share no file, and the resulting tree is identical to the signed squash's. The gate below was re-run on that combined tree. No review read `f5e16d2` together with this change; `f5e16d2` had its own verification record.

## The gate

Final run (r5) on tree `b33f7b3ec215c4df67f9578bcc4181a1c93c2246`: this change with `f5e16d2` merged in, identical to the first signed squash `4c66400`. The record's own correction is the only difference from it. Each unit's commands were read from its config as merged. The r4 run on `755c123`, without `f5e16d2`, had the same counts for every unit.

| Gate | Result |
| --- | --- |
| `check-units.sh` | exit 0 — `units: 4, disclaimed 6; tracked paths 452`; `--impact a547ff3..HEAD`: org-members, org-node, app all touched |
| org-node `cargo test -p org-node --features app,test-support ...` | **57 passed, 0 failed** — lib 23, admission_sender 7, service_stories 3, store_at_rest 4, transport_handshake 3, transport_networked 1, verify_against_chain 13, wire_frame_bound 3; three bolero targets ran their 1 s budget without panic (fuzz_envelope_decode 205,724 inputs, fuzz_verify_against_chain 200,401, fuzz_first_admission_base 137,586) |
| org-node quint 0.33.0 | typecheck `protocol`, `ods_instances` exit 0; `forkSafety`, `revocationSafety`, `revokedExcludedFromOrgSecret`, `tauWindow`, `convergence` no violation |
| org-members `cargo test -p org-members` | **194 passed, 0 failed**, 1 ignored by design (`preflight_probe`) — integration 153, mbt_conformance 32, fuzz 9 |
| org-members quint 0.33.0 | typecheck of `membership_types`, `membership`, `membership_mbt` exit 0; `quint test membership.qnt` **63 passing**; `quint run membership_mbt --invariant=mbtInv` no violation |
| org-members `make coverage-org-members` | lines **94.67%** (floor 92), regions **93.51%** (floor 91), functions 90.00%; decision coverage unmeasured |
| app cargo, 6 targets | **78 passed, 0 failed** |
| app `npm run check` / `npm run test` | 194 files, **0 errors**, 1 pre-existing warning (`@types/node`) / vitest **30 passed** |
| `check-ids.sh` | exit 0, silent, in all three units |
| `check-trace.sh` | exit 0 in all three units. org-node `REQ 20, HAZ 6, RC 9, PR 7`; org-members `REQ 13, HAZ 8, RC 10, SDD 6, LLR 39, PR 7` (two problem reports from `f5e16d2`); app `REQ 22, HAZ 6, RC 12, PR 6`. Only notices already present, within limits |
| Implements map / robustness | REQ-d9g6nt: normal — `member_ids_are_not_derived_from_keys`, `same_persona_founding_two_organisations_gets_two_admin_ids`; abnormal — `readmission_with_same_keys_gets_a_fresh_member_id`, `first_admission_without_a_record_snapshot_is_refused`, `fuzz_first_admission_base` |
| `check-review.sh` | exit 0 — `checked: records 21, for worktree-random-member-id 1, findings 23` |
| Working tree | clean |

Environment: every quint run used `QUINT_HOME` set to a scratch directory, because `~/.quint` is deliberately read-only to agents. `CARGO_HOME=/tmp/cargo_home_fuzz`.

**Coverage, gap carried forward.** org-members meets its line and region floors, but its decision coverage is unmeasured. org-node and app have no `coverage_command`. These are the class C gaps recorded in `docs/plans/2026-09-05-ratchet-setup.md`. The owner accepted the same gap for `27dab89` and carried it forward through `a547ff3`; this record carries that acceptance forward again, and the owner's signature on the squash confirms it. This change adds code to org-node, whose coverage remains unmeasured.

## Red → green

| Item | Test | Watched red |
| --- | --- | --- |
| `REQ-d9g6nt` | `member_ids_are_not_derived_from_keys` | "the admin's id is its member key", before `fresh_member_id` |
| `REQ-d9g6nt` | `readmission_with_same_keys_gets_a_fresh_member_id` | "round 0: id is B's member key", before `fresh_member_id` |
| `REQ-d9g6nt` | `fuzz_first_admission_base` | "refused for the wrong reason: Trie(DuplicateKey)", before the explicit refusal (the plain `is_err()` would have passed on the old fallback, so the stronger assertion was written first) |
| `REQ-d9g6nt` | `same_persona_founding_two_organisations_gets_two_admin_ids` | under mutant B2 (admin id = key XOR 0xff): "the same keys founding two organisations produced the same admin id" |
| `REQ-d9g6nt` | `first_admission_without_a_record_snapshot_is_refused` | under mutant C3 (old fallback re-inlined in `receive_and_verify`): "got Err(Trie(DuplicateKey))" |

Mutation measurement, round 2 reviewer: 15 mutants, 13 caught, 3 surviving. All three survivors are equivalent: the snapshot encode moved back after `submit_update` in admit and in revoke, and an encode failure turned back into `None`. Postcard encoding of `Vec<MemberSnapshot>` cannot fail, so no test can reach them.

## What was wrong, and what was built

PR-g7cfns, opened by the previous change's round 4 review: org-node made a `MemberId` by copying the member key's bytes. So a revoked person who rejoined with the same keypair got their deleted identifier back, which org-members' contract forbids, and after a key rotation the identifier still equalled the retired key.

Owner rulings, 2026-10-03:
- The admin draws the identifier at random before calling org-members, so `genesis` and `add_member` stay pure.
- Nothing relates a `MemberId` to any key.
- A re-admitted person is a new Member under a fresh `MemberId`, and may bring the keys their previous membership held when it was deleted. This is the one exception. Nothing granted to the old id carries over. If a removed device was compromised, fresh keys are the joiner's choice, an accepted residual.
- The duty not to supply other keys no longer held stands, and a deleted member's keys are never given to anyone else. Neither crate enforces it; the administrator carries it.
- Revocation is by `MemberId`, never by key.

Built:
- `fresh_member_id(rng)` in `create_organisation` and `admit_member`.
- `first_admission_base`, which decodes the admin's snapshot or refuses explicitly. It replaces the reconstructed-record fallback, which could never succeed.
- Snapshot encoding before the chain submit.
- Four service-level tests and a bolero target.

The org-members README, the `trie.rs` doc-comments and the requirement, risk and problem prose now state the ruling, and `docs/CONTEXT.md` says how a `MemberId` is chosen. Existing identifiers in stored records stay valid as opaque values; no migration is needed.

Problem-ledger delta: **resolves** PR-g7cfns (org-node).

## Review

**finding-1**: code — (round 1) The founding admin's id was checked only against "id equals key"; mutant B2 (admin id = key XOR 0xff) survived.
disposition: Added `same_persona_founding_two_organisations_gets_two_admin_ids`, which kills B2. Round 2 confirmed it also kills an id hashed from the key.

**finding-2**: code — (round 1) The snapshot refusal was tested only on `first_admission_base`; mutant C3 (the old fallback re-inlined in `receive_and_verify`) survived.
disposition: Added `first_admission_without_a_record_snapshot_is_refused`, at the service API, which kills C3 and asserts the store is unchanged.

**finding-3**: code — (round 1) The snapshot encode ran after the chain submit in `admit_member` and `revoke_member`, so an encode failure would leave the chain ahead of the local record.
disposition: Moved before `submit_update` in both. This is an equivalent mutant (postcard encoding cannot fail), so it is untested by necessity, and the code comment says why it runs first.

**finding-4**: record — (round 1) The `add_member` doc-comment narrowed "a key no longer held" to "a key replaced earlier".
disposition: Restored, with the ruling's one exception named.

**finding-5**: record — (round 1) The REQ-ewdg2q note rewrote the earlier dated parenthetical.
disposition: Restored word for word from master. The appended 2026-10-03 note carries the exception.

**finding-6**: record — (round 1) The org-node hazards and problems ledgers cited `service.rs` line numbers that this change shifted.
disposition: Every citation from `create_organisation` onward is now a name citation.

**finding-7**: record — (round 1) The REQ statement, the PR-g7cfns resolution and the risk file said the old fallback "completed".
disposition: All three now say it is refused explicitly rather than attempted, and that the attempt never succeeded.

**finding-8**: record — (round 1) The requirements file framed "keeps their identifier on rotation" as the defect.
disposition: Reworded: the defect is that the identifier still equals the retired key.

**finding-9**: record — (round 1) The PR-g7cfns body stated the pre-ruling contract as if it were current.
disposition: Marked as the contract as stated before the ruling.

**finding-10**: record — (round 1) The CONTEXT.md MemberId addition was wrong for the founding admin and for records made before this change.
disposition: Reworded to "chosen at random when the Member is placed in the Membership record". Older identifiers may equal key bytes and are opaque either way.

**finding-11**: code — (round 1) `first_admission_base` was public in the shipped API only for the fuzz target.
disposition: `#[doc(hidden)]`, with a comment stating why it is public.

**finding-12**: requirement — (round 1) Revocation by MemberId was in no `shall` and not declared out of scope.
disposition: The requirements file now states it as existing behaviour that the ruling confirms, outside REQ-d9g6nt's scope.

**finding-13**: record — (round 2) This change made numeric `service.rs` citations stale in the transport-binding SRS and in app comments.
disposition: Converted to name citations, each checked against the code (round 3 confirmed).

**finding-14**: record — (round 2) README item 11 stated the ruling wider than given, with no carve-out against its own bullets.
disposition: The heading and text now state the one exception exactly.

**finding-15**: record — (round 2) The `delete_member` doc-comment dropped "never to anyone else", and both comments said "caller's choice".
disposition: Restored, and both comments now say "the joiner's choice".

**finding-16**: record — (round 2) The PR-g7cfns resolution and the requirement omitted the two fix-round tests.
disposition: Both now list all five verifying tests.

**finding-17**: record — (round 2) Terms: "the record" stood in for "Membership record", and bare "member key" is an Avoid term.
disposition: Glossary terms are now used in the requirement, the risk file and CONTEXT.md.

**finding-18**: record — (round 2) A new statement in the org-node hazards register had no dated marker.
disposition: Now marked "(Amended 2026-10-03, REQ-d9g6nt: …)".

**finding-19**: record — (round 2, pre-existing) The org-node problems ledger cited hazard-register lines that never pointed at RC-gfn6kr.
disposition: Now a name citation of the RC-gfn6kr bullet and its verdict.

**finding-20**: record — (round 3) The REQ-ewdg2q "Narrowed" note used "lost or stolen" and omitted two clauses of the ruling.
disposition: Restated to the ruling's six clauses in glossary terms.

**finding-21**: record — (round 3) The membership-hazards correction under RC-mqtks7 forbade the ruling's exception, and the HAZ-s39gbh row called it caller-carried.
disposition: Both amended in place, dated, to the ruling.

**finding-22**: record — (round 3) The org-node requirement's ruling paragraph omitted the standing duty and who carries it.
disposition: Added. Neither crate enforces the duty; the administrator carries it. The "met here or nowhere" sentence now names the MemberId duty, which org-node meets by construction.

**finding-23**: record — (round 3) README item 11's new text used "member key".
disposition: Replaced with "Member-as-a-group key" and "Device key".

## Gaps

- **A key no longer held is not refused**, and neither crate keeps key history, by owner ruling. Re-admission with the previous membership's keys is allowed. Where a removed device was compromised, fresh keys are the joiner's choice: owner-accepted residual risk on HAZ-vxabf9, not re-scored.
- **The Revoke screen has no member list.** An operator now finds a `MemberId` only on the Admit result screen, because revoking by pasting a member key no longer works. The owner confirmed that revocation is by `MemberId`; delete-by-handle may be added later.
- **Re-admission is tested on the admin side.** The receivers in those tests are sinks, so B's full receive path on a re-admission is not exercised.
- **Decision coverage unmeasured**; org-node and app coverage unmeasured.
- **Round 3's corrections were not reviewed**, under the convergence rule.
- Three test `panic!` calls trip `clippy::panic`. CI lints only the org-members and on-chain-client libraries, so nothing gates it.
